"""Compute backend selection — numpy-cpu (always available) or torch-gpu (accelerated).

Never-silent (G2): the chosen backend is printed at startup.  An unavailable GPU falls
back loudly with a printed reason; numpy always works.

Transparency: GPU acceleration provides identical numerical results for all operations
defined in algebra.py (elementwise multiply, sum, cosine) — the numpy and torch paths
agree to floating-point precision.  Backend is `Empirical`: measured on the maintainer's
hardware; GPU detection relies on torch's runtime probing.
"""

from __future__ import annotations

import sys
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    pass

# ---------------------------------------------------------------------------
# Backend state — resolved once at first call to `select()`.
# ---------------------------------------------------------------------------

_BACKEND: str | None = None  # "numpy-cpu" | "torch-cpu" | "torch-cuda:<name>"
_NP = None  # numpy module (always importable)
_TORCH = None  # torch module or None


def _import_numpy():
    global _NP
    if _NP is None:
        import numpy as np  # noqa: PLC0415

        _NP = np
    return _NP


def _try_import_torch():
    global _TORCH
    if _TORCH is not None:
        return _TORCH
    try:
        import torch  # noqa: PLC0415

        _TORCH = torch
        return _TORCH
    except ImportError:
        return None


def select(force_numpy: bool = False) -> str:
    """Detect and fix the compute backend.  Returns the backend string and prints it.

    Call once at experiment startup; subsequent calls are no-ops and return the cached
    backend name.

    Args:
        force_numpy: if True, always select numpy-cpu regardless of torch/CUDA.

    Returns:
        One of ``"numpy-cpu"``, ``"torch-cpu"``, ``"torch-cuda:<device_name>"``.
    """
    global _BACKEND
    if _BACKEND is not None:
        return _BACKEND

    _import_numpy()

    reason: str | None = None  # set to a non-None string to use numpy-cpu with a reason

    if not force_numpy:
        torch = _try_import_torch()
        if torch is not None:
            if torch.cuda.is_available():
                dev = torch.cuda.get_device_name(0)
                _BACKEND = f"torch-cuda:{dev}"
                print(f"[vsa_bounds] backend: {_BACKEND}", file=sys.stderr)
                return _BACKEND
            else:
                # torch present but no CUDA — set backend and return immediately.
                reason_str = "torch present but torch.cuda.is_available() == False"
                _BACKEND = "torch-cpu"
                print(
                    f"[vsa_bounds] backend: torch-cpu  (GPU unavailable — {reason_str})",
                    file=sys.stderr,
                )
                return _BACKEND
        else:
            reason = "torch not installed; run `uv sync --group gpu` for GPU acceleration"

    _BACKEND = "numpy-cpu"
    if reason is not None:
        print(
            f"[vsa_bounds] backend: numpy-cpu  ({reason})",
            file=sys.stderr,
        )
    else:
        print(f"[vsa_bounds] backend: {_BACKEND}", file=sys.stderr)
    return _BACKEND


def is_cuda() -> bool:
    """True iff the resolved backend is CUDA."""
    return (_BACKEND or "").startswith("torch-cuda:")


def is_torch() -> bool:
    """True iff the resolved backend is any torch path (cpu or cuda)."""
    return (_BACKEND or "").startswith("torch-")


def numpy():
    """Return the numpy module (always available)."""
    return _import_numpy()


def torch():
    """Return the torch module, or None if not installed."""
    return _try_import_torch()
