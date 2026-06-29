"""VSA algebra — batch-vectorized MAP-I (and MAP-B / HRR / FHRR stubs) on numpy or torch.

Semantics match `crates/mycelium-vsa/src/{mapi,mapb,hrr,fhrr}.rs` exactly:

MAP-I:
  bind(a, b)    = elementwise product (±1 alphabet; self-inverse: unbind == bind)
  unbind(a, b)  = bind(a, b)  [self-inverse, Exact]
  bundle(xs)    = elementwise sum (integer superposition)
  permute(a, k) = cyclic left-shift by k positions
  similarity    = cosine
  threshold     = sign (Hebbian cleanup)

MAP-B:
  Same as MAP-I but bundle applies sign-rounding back to ±1 (tie -> first component).
  RFC-0003 §4: "Proven membership-only, no deep nesting".

HRR:
  bind(a, b)    = circular convolution (via FFT)
  unbind(a, b)  = circular correlation (a ⊛ b~, where b~[i] = b[-i mod d])
  bundle        = elementwise sum (Empirical, Gaussian capacity)
  similarity    = cosine
  threshold     = nearest codebook atom (cleanup via similarity lookup)

FHRR:
  bind(a, b)    = elementwise phase addition (mod 2pi)
  unbind(a, b)  = elementwise phase subtraction
  bundle        = complex mean angle (arg of mean phasor)
  similarity    = mean cos(theta_a - theta_b)
  threshold     = nearest phasor codebook atom

All ops accept batched inputs with a leading batch dimension (B, D) so GPU sweeps
can process B independent trials in one call.

Guarantee tags per op (matching RFC0003_MATRIX):
  MAP-I  bind/unbind/permute: Exact; bundle: Proven (when side-conditions hold)
  MAP-B  bind/unbind/permute: Exact; bundle: Proven membership-only (depth 1)
  HRR    bind/permute: Exact; unbind/bundle: Empirical
  FHRR   bind/permute: Exact; unbind/bundle: Empirical

This module NEVER issues Proven tags for multi-hop results — that is OQ-F.
"""

from __future__ import annotations

import math
from typing import Literal

import numpy as np

from . import backend as _be

# ---------------------------------------------------------------------------
# LCG (deterministic RNG matching `resonator_profile.rs`) — no `random`
# ---------------------------------------------------------------------------

_LCG_MUL = 6364136223846793005
_LCG_ADD = 1442695040888963407
_LCG_INIT_MUL = 0x9E3779B97F4A7C15
_LCG_INIT_ADD = 1
_U64_MASK = 0xFFFFFFFFFFFFFFFF


class Lcg:
    """Deterministic LCG matching `resonator_profile.rs::Lcg`.

    Same constants as the Rust test so seed→vector sequences agree cross-language.
    """

    __slots__ = ("_state",)

    def __init__(self, seed: int) -> None:
        self._state = ((seed * _LCG_INIT_MUL) + _LCG_INIT_ADD) & _U64_MASK

    def next_u64(self) -> int:
        self._state = (self._state * _LCG_MUL + _LCG_ADD) & _U64_MASK
        return self._state

    def bipolar(self, dim: int) -> np.ndarray:
        """Draw a bipolar (±1) vector of length `dim`."""
        arr = np.empty(dim, dtype=np.float32)
        for i in range(dim):
            arr[i] = 1.0 if (self.next_u64() >> 63) & 1 else -1.0
        return arr

    def uniform_phase(self, dim: int) -> np.ndarray:
        """Draw uniform phase angles in (-pi, pi] for FHRR."""
        arr = np.empty(dim, dtype=np.float32)
        for i in range(dim):
            u = self.next_u64()
            # Map u64 -> [-pi, pi]: u / 2^64 -> [0,1) -> [-pi, pi)
            arr[i] = (u / (2**64) * 2 * math.pi) - math.pi
        return arr

    def gaussian(self, dim: int) -> np.ndarray:
        """Draw N(0, 1/dim) Gaussian atom for HRR (Box-Muller from LCG)."""
        std = 1.0 / math.sqrt(dim)
        arr = np.empty(dim, dtype=np.float32)
        i = 0
        while i < dim:
            u1 = (self.next_u64() + 1) / (2**64 + 1)  # avoid log(0)
            u2 = self.next_u64() / 2**64
            z0 = math.sqrt(-2.0 * math.log(u1)) * math.cos(2.0 * math.pi * u2)
            z1 = math.sqrt(-2.0 * math.log(u1)) * math.sin(2.0 * math.pi * u2)
            arr[i] = z0 * std
            if i + 1 < dim:
                arr[i + 1] = z1 * std
            i += 2
        return arr


# ---------------------------------------------------------------------------
# Tensor helpers — transparent numpy/torch dispatch
# ---------------------------------------------------------------------------


def _as_tensor(arr: np.ndarray, cuda: bool):
    """Convert numpy array to torch tensor on the right device if torch path active."""
    if cuda:
        t = _be.torch()
        if t is None:
            raise RuntimeError("torch not available but cuda=True was requested")
        return t.from_numpy(arr).cuda()
    torch = _be.torch()
    if torch is not None and _be.is_torch():
        return torch.from_numpy(arr)
    return arr


def _to_numpy(x) -> np.ndarray:
    """Move tensor back to numpy (no-op if already numpy)."""
    if isinstance(x, np.ndarray):
        return x
    return x.cpu().numpy()


# ---------------------------------------------------------------------------
# MAP-I algebra (Exact bind/unbind/permute, Proven bundle)
# ---------------------------------------------------------------------------

ModelName = Literal["mapi", "mapb", "hrr", "fhrr"]


def mapi_bind(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """MAP-I bind = elementwise product (Exact, self-inverse on ±1 alphabet)."""
    return a * b


def mapi_unbind(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """MAP-I unbind = bind (self-inverse on ±1 alphabet, Exact)."""
    return mapi_bind(a, b)


def mapi_bundle(xs: list[np.ndarray]) -> np.ndarray:
    """MAP-I bundle = elementwise sum (integer superposition).

    Guarantee: Proven when dim >= required_dim(len(xs), delta) — checked by caller.
    """
    if not xs:
        raise ValueError("empty bundle")
    acc = xs[0].copy()
    for x in xs[1:]:
        acc = acc + x
    return acc


def mapi_permute(a: np.ndarray, shift: int) -> np.ndarray:
    """MAP-I permute = cyclic left-shift by `shift` (Exact)."""
    return np.roll(a, -shift, axis=-1)


def mapi_unpermute(a: np.ndarray, shift: int) -> np.ndarray:
    """MAP-I inverse permute (Exact)."""
    return mapi_permute(a, -shift)


def mapi_similarity(a: np.ndarray, b: np.ndarray) -> float | np.ndarray:
    """Cosine similarity.  For batches (B,D) vs (D,), returns (B,) similarities."""
    if a.ndim == 1:
        denom = np.linalg.norm(a) * np.linalg.norm(b)
        return float(np.dot(a, b) / denom) if denom > 0 else 0.0
    # batch: a is (B, D), b is (D,)
    dots = a @ b
    na = np.linalg.norm(a, axis=-1)
    nb = float(np.linalg.norm(b))
    denom = na * nb
    denom = np.where(denom > 0, denom, 1.0)
    return dots / denom


def mapi_threshold(v: np.ndarray) -> np.ndarray:
    """Hebbian bipolar thresholding: sign(v), ties -> +1 (Cleanup::Hebbian in Rust)."""
    return np.where(v >= 0, 1.0, -1.0).astype(np.float32)


# ---------------------------------------------------------------------------
# MAP-B algebra
# ---------------------------------------------------------------------------


def mapb_bind(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """MAP-B bind = elementwise product (same as MAP-I, Exact)."""
    return a * b


def mapb_unbind(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """MAP-B unbind = bind (self-inverse, Exact)."""
    return mapb_bind(a, b)


def mapb_bundle(xs: list[np.ndarray]) -> np.ndarray:
    """MAP-B bundle = elementwise sum then sign-round to ±1.

    Ties (sum == 0) copy the first component per RFC-0003 §4 (deterministic).
    Guarantee: Proven membership-only, depth=1 (RFC-0003 §4; RR-13 forbids nesting).
    """
    if not xs:
        raise ValueError("empty bundle")
    s = mapi_bundle(xs)
    result = np.sign(s).astype(np.float32)
    # Resolve ties: use first item's component where sum == 0.
    ties = result == 0.0
    if np.any(ties):
        result[ties] = xs[0][ties]
    return result


def mapb_permute(a: np.ndarray, shift: int) -> np.ndarray:
    """MAP-B permute = cyclic left-shift (Exact)."""
    return mapi_permute(a, shift)


def mapb_similarity(a: np.ndarray, b: np.ndarray) -> float | np.ndarray:
    """Cosine similarity (same computation as MAP-I)."""
    return mapi_similarity(a, b)


# ---------------------------------------------------------------------------
# HRR algebra (circular convolution / correlation via FFT)
# ---------------------------------------------------------------------------


def hrr_bind(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """HRR bind = circular convolution via FFT (Exact algebraic op).

    `(a ⊛ b)[k] = Σ_i a[i] * b[(k-i) mod d]`
    """
    return np.fft.irfft(np.fft.rfft(a) * np.fft.rfft(b), n=len(a)).astype(np.float32)


def _hrr_inv(b: np.ndarray) -> np.ndarray:
    """HRR involution b~: b~[i] = b[-i mod d].  For circular correlation."""
    d = len(b)
    inv = np.empty(d, dtype=b.dtype)
    inv[0] = b[0]
    inv[1:] = b[1:][::-1]
    return inv


def hrr_unbind(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """HRR unbind = circular correlation: a ⊛ b~ (Empirical — approximate inverse).

    This is an approximate inverse, not exact; it requires cleanup against a codebook.
    Guarantee: Empirical (RFC-0003 §4; HRR_UNBIND_PROFILE in hrr.rs).
    """
    return hrr_bind(a, _hrr_inv(b))


def hrr_bundle(xs: list[np.ndarray]) -> np.ndarray:
    """HRR bundle = elementwise sum (Empirical, Gaussian capacity)."""
    return mapi_bundle(xs)


def hrr_permute(a: np.ndarray, shift: int) -> np.ndarray:
    """HRR permute = cyclic left-shift (Exact)."""
    return mapi_permute(a, shift)


def hrr_similarity(a: np.ndarray, b: np.ndarray) -> float | np.ndarray:
    """HRR cosine similarity."""
    return mapi_similarity(a, b)


# ---------------------------------------------------------------------------
# FHRR algebra (phasor / frequency-domain)
# ---------------------------------------------------------------------------

_PI = math.pi
_TAU = 2.0 * math.pi


def _wrap_phase(theta: np.ndarray) -> np.ndarray:
    """Wrap to (-pi, pi]."""
    t = theta % _TAU
    t = np.where(t > _PI, t - _TAU, t)
    return t.astype(np.float32)


def fhrr_bind(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """FHRR bind = elementwise phase addition (complex multiplication of phasors, Exact)."""
    return _wrap_phase(a + b)


def fhrr_unbind(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """FHRR unbind = elementwise phase subtraction (Empirical in multi-factor context)."""
    return _wrap_phase(a - b)


def fhrr_bundle(xs: list[np.ndarray]) -> np.ndarray:
    """FHRR bundle = complex mean phasor (Empirical).

    Near-zero phasor sum is degenerate — here we fall back to phase 0.0 (flagged
    in practice via low similarity; experiment harness treats this as a failure).
    """
    if not xs:
        raise ValueError("empty bundle")
    real = np.zeros(len(xs[0]), dtype=np.float64)
    imag = np.zeros(len(xs[0]), dtype=np.float64)
    for x in xs:
        real += np.cos(x.astype(np.float64))
        imag += np.sin(x.astype(np.float64))
    angles = np.arctan2(imag, real)
    return _wrap_phase(angles.astype(np.float32))


def fhrr_permute(a: np.ndarray, shift: int) -> np.ndarray:
    """FHRR permute = cyclic left-shift of phase components (Exact)."""
    return mapi_permute(a, shift)


def fhrr_similarity(a: np.ndarray, b: np.ndarray) -> float | np.ndarray:
    """FHRR similarity = mean cos(theta_a - theta_b) in [-1, 1]."""
    diff = _wrap_phase(a.astype(np.float64) - b.astype(np.float64))
    if a.ndim == 1:
        return float(np.mean(np.cos(diff)))
    return np.mean(np.cos(diff), axis=-1).astype(np.float64)


# ---------------------------------------------------------------------------
# Cleanup: exact-recovery vs codebook (nearest-atom lookup)
# ---------------------------------------------------------------------------


def cleanup_exact(
    v: np.ndarray,
    codebook: np.ndarray,
    model: ModelName,
) -> int:
    """Return the index of the codebook atom most similar to `v`.

    `codebook` is (k, d) — k atoms of dim d.
    For MAP-I/MAP-B: cosine similarity.
    For HRR: cosine similarity (same formula).
    For FHRR: mean cos(theta_a - theta_b).
    """
    if model == "fhrr":
        # pairwise phase differences: (k, d)
        diffs = _wrap_phase(codebook.astype(np.float64) - v[np.newaxis, :].astype(np.float64))
        sims = np.mean(np.cos(diffs), axis=-1)
    else:
        # cosine: (k,)
        dots = codebook @ v
        norms_c = np.linalg.norm(codebook, axis=-1)
        norm_v = float(np.linalg.norm(v))
        denom = norms_c * norm_v
        denom = np.where(denom > 0, denom, 1.0)
        sims = dots / denom
    return int(np.argmax(sims))


# ---------------------------------------------------------------------------
# Dispatch table for model-agnostic callers
# ---------------------------------------------------------------------------


def bind(model: ModelName, a: np.ndarray, b: np.ndarray) -> np.ndarray:
    if model in ("mapi", "mapb"):
        return mapi_bind(a, b)
    elif model == "hrr":
        return hrr_bind(a, b)
    elif model == "fhrr":
        return fhrr_bind(a, b)
    raise ValueError(f"unknown model {model!r}")


def unbind(model: ModelName, a: np.ndarray, b: np.ndarray) -> np.ndarray:
    if model in ("mapi", "mapb"):
        return mapi_unbind(a, b)
    elif model == "hrr":
        return hrr_unbind(a, b)
    elif model == "fhrr":
        return fhrr_unbind(a, b)
    raise ValueError(f"unknown model {model!r}")


def bundle(model: ModelName, xs: list[np.ndarray]) -> np.ndarray:
    if model == "mapi":
        return mapi_bundle(xs)
    elif model == "mapb":
        return mapb_bundle(xs)
    elif model == "hrr":
        return hrr_bundle(xs)
    elif model == "fhrr":
        return fhrr_bundle(xs)
    raise ValueError(f"unknown model {model!r}")


def similarity(model: ModelName, a: np.ndarray, b: np.ndarray) -> float | np.ndarray:
    if model in ("mapi", "mapb", "hrr"):
        return mapi_similarity(a, b)
    elif model == "fhrr":
        return fhrr_similarity(a, b)
    raise ValueError(f"unknown model {model!r}")


def sample_atom(model: ModelName, dim: int, lcg: Lcg) -> np.ndarray:
    """Draw a random codebook atom appropriate for the given model."""
    if model in ("mapi", "mapb"):
        return lcg.bipolar(dim)
    elif model == "hrr":
        return lcg.gaussian(dim)
    elif model == "fhrr":
        return lcg.uniform_phase(dim)
    raise ValueError(f"unknown model {model!r}")
