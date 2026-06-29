"""Monte-Carlo sweeps for VSA compositional capacity (M-832, OQ-F).

Two sweeps:

  single   — bundle capacity: exact-decode failure rate vs dimension d at given (m, delta).
             Anchors the Proven formula parity.  `Empirical` output (trial-measured rates).

  multihop — composition: sweep {model, F, k, d, h, delta} for bind-chains, bundle-of-binds,
             and nested unbind.  Compares measured failure rate against the single-hop
             closed-form bound extrapolated naively to multi-hop.  Locates candidate regimes
             where the formula still upper-bounds measured rate vs where it diverges.

VR-5 / G2:
  - Measured rates only; no Proven verdicts on multi-hop results.
  - Deterministic LCG seeds — no random nondeterminism in recorded runs.
  - All failure counts and exact trial counts are recorded; nothing is silently capped.
"""

from __future__ import annotations

import dataclasses
import sys
import time
from typing import Literal

import numpy as np

from .algebra import Lcg, ModelName, bind, bundle, cleanup_exact, sample_atom, similarity, unbind
from .capacity import MARGIN_MU, required_dim

# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------

CompositionKind = Literal["bind_chain", "bundle_of_binds", "nested_unbind"]


@dataclasses.dataclass
class SingleResult:
    """One (m, d, delta) measurement from the single-hop bundle sweep."""

    m: int  # bundle size
    d: int  # dimension
    delta: float  # target failure probability
    trials: int
    failures: int
    measured_rate: float
    required_dim_proven: int  # required_dim(m, delta) from the Proven formula
    bound_holds: bool  # d >= required_dim (side-condition for Proven tag)
    bound_respected: bool  # measured_rate <= delta  (whether bound is empirically satisfied)
    elapsed_s: float
    # Guarantee tag for THIS measurement: always Empirical (trial-measured rate).
    guarantee: str = "Empirical"


@dataclasses.dataclass
class MultihopResult:
    """One {model, F, k, d, h, delta, composition} measurement from the multihop sweep."""

    model: ModelName
    composition: CompositionKind
    F: int  # number of factors / factor slots
    k: int  # codebook size per slot
    d: int  # dimension
    h: int  # hop depth (bind-chain depth or nesting level)
    delta: float  # comparison target
    trials: int
    failures: int
    measured_rate: float
    # Single-hop Proven bound naive extrapolation:
    #   bind_chain / nested_unbind: effective m = F * k^h (rough worst-case)
    #   bundle_of_binds: m = F, h bundles-of-binds
    # This extrapolation is NOT Proven — it is a heuristic comparison point (Declared).
    naive_extrapolated_m: int
    naive_required_dim: int  # required_dim(naive_extrapolated_m, delta)
    naive_bound_holds: bool  # d >= naive_required_dim
    naive_bound_respected: bool  # measured_rate <= delta when naive bound says it should hold
    bound_diverges: bool  # naive bound predicts OK but measured rate > delta
    elapsed_s: float
    guarantee: str = "Empirical"  # all multihop results are Empirical


# ---------------------------------------------------------------------------
# Single-hop bundle sweep
# ---------------------------------------------------------------------------


def _single_trial(
    m: int,
    d: int,
    model: ModelName,
    lcg: Lcg,
    n_distractors: int = 100,
) -> bool:
    """One trial: bundle m atoms; check membership detection vs n_distractors non-members.

    The Clarkson/Thomas capacity bound (RFC-0003 §5) is about a MEMBERSHIP query:
    given bundle B = a_0 + ... + a_{m-1} and a probe atom q, is q a member?

    Failure = a non-member distractor has higher similarity to B than the true member a_0.

    This matches the semantics of the capacity theorem: if `d >= required_dim(m, delta)`,
    then for each member atom the probability that any distractor beats it in similarity is
    at most delta (Clarkson/Thomas, 2023 Thm 6).

    Args:
        m: bundle size (number of members).
        d: dimension.
        model: VSA model.
        lcg: deterministic RNG.
        n_distractors: number of non-member atoms to compare against.

    Returns:
        True iff membership detection fails for atom_0 (any distractor beats it).

    Guarantee: Empirical — result of one stochastic trial (VR-5).
    """
    # Draw m member atoms
    atoms = [sample_atom(model, d, lcg) for _ in range(m)]

    # Bundle all members
    b = bundle(model, atoms)

    # Similarity of atom_0 (query member) to bundle
    sim_member = similarity(model, atoms[0], b)
    if isinstance(sim_member, np.ndarray):
        sim_member = float(sim_member)

    # Draw n_distractors non-member atoms; failure if any has similarity >= sim_member
    for _ in range(n_distractors):
        distractor = sample_atom(model, d, lcg)
        sim_dist = similarity(model, distractor, b)
        if isinstance(sim_dist, np.ndarray):
            sim_dist = float(sim_dist)
        if sim_dist >= sim_member:
            return True  # failure: distractor beats member
    return False


def run_single_sweep(
    *,
    m_values: list[int],
    d_values: list[int],
    delta: float = 0.02,
    trials_per_point: int = 1000,
    model: ModelName = "mapi",
    salt: int = 0xDEAD_BEEF,
) -> list[SingleResult]:
    """Sweep bundle size m and dimension d, measuring exact-decode failure rate.

    Anchors the Proven formula: for each (m, d), checks whether
    `measured_rate <= delta` and whether `d >= required_dim(m, delta)`.

    Args:
        m_values: bundle sizes to sweep.
        d_values: dimensions to sweep.
        delta: target failure probability (comparison point).
        trials_per_point: Monte-Carlo trials per (m, d) point.
        model: VSA model (mapi, mapb, hrr, fhrr).
        salt: base LCG seed; each (m, d, trial) gets a unique seed.

    Returns:
        List of SingleResult, one per (m, d) pair.

    Guarantee: Empirical — trial-measured rates only (VR-5).

    EXPLAIN (n_distractors=100): each trial draws 100 non-member atoms and counts
    failure if ANY distractor similarity exceeds the true-member similarity.  This is
    more stringent than the single-distractor Clarkson/Thomas bound; at
    d >> required_dim the probability of even one distractor winning is negligible,
    so 100 distractors amplifies the sensitivity at the boundary without inflating
    the failure rate at comfortable dimensions.  The amplification is why the test
    asserts rate <= 5*delta (not delta) at 2x required_dim (test_proven_formula_anchor).
    """
    results: list[SingleResult] = []
    for m in m_values:
        req = required_dim(m, delta, MARGIN_MU)
        for d in d_values:
            t0 = time.monotonic()
            failures = 0
            for trial in range(trials_per_point):
                seed = salt ^ (m * 100_003 + d * 7 + trial)
                lcg = Lcg(seed)
                if _single_trial(m, d, model, lcg):
                    failures += 1
            elapsed = time.monotonic() - t0
            rate = failures / trials_per_point
            results.append(
                SingleResult(
                    m=m,
                    d=d,
                    delta=delta,
                    trials=trials_per_point,
                    failures=failures,
                    measured_rate=rate,
                    required_dim_proven=req,
                    bound_holds=d >= req,
                    bound_respected=rate <= delta,
                    elapsed_s=elapsed,
                )
            )
    return results


# ---------------------------------------------------------------------------
# Multi-hop composition sweeps
# ---------------------------------------------------------------------------


def _bind_chain_trial(
    F: int,
    k: int,
    d: int,
    h: int,
    model: ModelName,
    lcg: Lcg,
) -> bool:
    """Bind-chain depth h: s = b_1 * b_2 * ... * b_h, each b_i from a codebook of k.

    Recovery: unbind sequentially and check exact atom recovery for the LAST factor.
    `h` bind operations; F parallel factor slots each with codebook of k.
    The bound question: does the single-hop formula cover h-hop compositions?

    Failure = wrong atom recovered for the final factor.
    """
    # Draw F * k codebook atoms (F factor-slots, k atoms each)
    # Then for each slot draw h atom-indices and bind them into a chain.
    codebooks: list[np.ndarray] = []  # (F, k, d) effectively
    for _ in range(F):
        cb = np.stack([sample_atom(model, d, lcg) for _ in range(k)], axis=0)  # (k, d)
        codebooks.append(cb)

    # Ground truth: pick a random index in each slot for each hop depth
    # For h=1: pick one atom per slot and bind them -> single-hop product
    # For h>1: pick h atoms per slot (one per hop), bind chain within each slot, then combine
    truth_indices: list[list[int]] = []
    for f in range(F):
        slot_indices = [int(lcg.next_u64() % k) for _ in range(h)]
        truth_indices.append(slot_indices)

    # Build the h-hop compound atom for each slot: atom_f = a_f_1 * a_f_2 * ... * a_f_h
    slot_atoms: list[np.ndarray] = []
    for f in range(F):
        a = codebooks[f][truth_indices[f][0]].copy()
        for hop in range(1, h):
            a = bind(model, a, codebooks[f][truth_indices[f][hop]])
        slot_atoms.append(a)

    # Bind all slots together: s = slot_0 * slot_1 * ... * slot_{F-1}
    if len(slot_atoms) == 1:
        s = slot_atoms[0].copy()
    else:
        s = bind(model, slot_atoms[0], slot_atoms[1])
        for j in range(2, len(slot_atoms)):
            s = bind(model, s, slot_atoms[j])

    # Recovery: unbind all OTHER slots from s to get an estimate of slot_0,
    # then unbind the h-1 hops within slot_0 to recover truth_indices[0][0].
    est_slot0 = s.copy()
    for f in range(1, F):
        est_slot0 = unbind(model, est_slot0, slot_atoms[f])

    # Now est_slot0 should be close to codebooks[0][truth_indices[0][0]] * (noise from hops)
    # Unbind the hop chain within slot 0 (hops 1..h-1):
    for hop in range(1, h):
        est_slot0 = unbind(model, est_slot0, codebooks[0][truth_indices[0][hop]])

    # Cleanup: find nearest atom in codebook[0]
    best = cleanup_exact(est_slot0, codebooks[0], model)
    return best != truth_indices[0][0]


def _bundle_of_binds_trial(
    F: int,
    k: int,
    d: int,
    h: int,
    model: ModelName,
    lcg: Lcg,
) -> bool:
    """Bundle of h bound pairs: s = bundle(b_1 * c_1, b_2 * c_2, ..., b_h * c_h).

    Each bound pair draws from the same codebook of k atoms.
    Recovery: unbind s by a known factor and check if we recover the paired atom.

    Failure = wrong atom recovered for the second factor of the first pair.
    """
    # Codebook: k atoms shared across all pairs
    cb = np.stack([sample_atom(model, d, lcg) for _ in range(k)], axis=0)  # (k, d)

    # Draw h pairs (a_i, b_i); ground truth for pair 0 is (a0, b0)
    pairs_a: list[np.ndarray] = []
    pairs_b: list[np.ndarray] = []
    truth_b_idx = int(lcg.next_u64() % k)  # we'll test recovery of this
    for i in range(h):
        ia = int(lcg.next_u64() % k)
        if i == 0:
            ib = truth_b_idx
        else:
            ib = int(lcg.next_u64() % k)
        pairs_a.append(cb[ia].copy())
        pairs_b.append(cb[ib].copy())

    # Build h bound products
    products = [bind(model, pairs_a[i], pairs_b[i]) for i in range(h)]

    # Bundle all products
    s = bundle(model, products)

    # Recovery: unbind s by pairs_a[0] to get estimate of pairs_b[0]
    est_b0 = unbind(model, s, pairs_a[0])

    # Cleanup against codebook
    best = cleanup_exact(est_b0, cb, model)
    return best != truth_b_idx


def _nested_unbind_trial(
    F: int,
    k: int,
    d: int,
    h: int,
    model: ModelName,
    lcg: Lcg,
) -> bool:
    """Nested unbind (h levels deep): s = a_1 * ... * a_h, unbind h-1 times.

    This tests how well exact recovery holds when we apply h unbind operations to
    a bundle of F atoms.

    Failure = wrong atom recovered after h nested unbinds.
    """
    # Draw codebook of k atoms
    cb = np.stack([sample_atom(model, d, lcg) for _ in range(k)], axis=0)  # (k, d)

    # Draw F*h atoms: F factor slots, h hops per slot
    truth_final_idx = 0  # the atom index we'll try to recover at the deepest level
    atoms_by_hop: list[list[np.ndarray]] = []  # [hop][slot] -> atom
    for hop in range(h):
        row = []
        for f in range(F):
            idx = int(lcg.next_u64() % k)
            if hop == h - 1 and f == 0:
                truth_final_idx = idx
            row.append(cb[idx].copy())
        atoms_by_hop.append(row)

    # Build the nested product: for each hop, bind all F slots, then bundle over hops?
    # Nested unbind: bind within hops, then unbind one hop at a time.
    # Level 0: s_0 = bind(atoms_by_hop[0])
    # Level 1: s_1 = bind(s_0, bind(atoms_by_hop[1]))
    # ... (but this is getting complex; use a simpler "bind-chain then peel" model)
    #
    # Interpretation: s = a_{0,0} * a_{0,1} * ... * a_{0,F-1}  (hop 0, F factors)
    # Apply h-1 nested unbinds: unbind by a_{0,1}, a_{0,2}, ..., a_{0,F-1} to get a_{0,0},
    # then re-bundle and re-unbind...
    # Actually: nested unbind = apply F-1 unbinds then repeat h times.
    # For simplicity: treat as h successive rounds of (bind F atoms, unbind F-1, recover 1).

    all_products: list[np.ndarray] = []
    for hop in range(h):
        row = atoms_by_hop[hop]
        if len(row) == 1:
            p = row[0].copy()
        else:
            p = bind(model, row[0], row[1])
            for j in range(2, len(row)):
                p = bind(model, p, row[j])
        all_products.append(p)

    # s = bundle of all hop products
    s = bundle(model, all_products)

    # Unbind all hop products except hop h-1 to peel layers
    for hop in range(h - 1):
        s = unbind(model, s, all_products[hop])

    # Now s ≈ atoms_by_hop[h-1] product; unbind all but factor 0
    for f in range(1, F):
        s = unbind(model, s, atoms_by_hop[h - 1][f])

    # Cleanup against codebook
    best = cleanup_exact(s, cb, model)
    return best != truth_final_idx


def _naive_extrapolated_m(composition: CompositionKind, F: int, k: int, h: int) -> int:
    """Rough effective bundle size for naive single-hop bound extrapolation.

    This is a DECLARED heuristic, not a Proven derivation.  The idea: each hop
    introduces roughly k times more interference, so the effective `m` grows as k^h.

    For bind_chain / nested_unbind: m_eff = F * k^h  (each slot binds h atoms from k)
    For bundle_of_binds: m_eff = h * k  (h bundles of k pairs)

    DO NOT use this to stamp Proven — it is a naive worst-case proxy (Declared).
    """
    if composition == "bundle_of_binds":
        return h * k
    else:
        return F * (k**h)


def run_multihop_sweep(
    *,
    models: list[ModelName] | None = None,
    compositions: list[CompositionKind] | None = None,
    F_values: list[int] | None = None,
    k_values: list[int] | None = None,
    d_values: list[int] | None = None,
    h_values: list[int] | None = None,
    delta: float = 0.02,
    trials_per_point: int = 500,
    salt: int = 0xC0DE_FACE,
    progress: bool = True,
) -> list[MultihopResult]:
    """Sweep {model, composition, F, k, d, h} measuring failure rate vs naive formula.

    For each point:
    1. Run `trials_per_point` Monte-Carlo trials.
    2. Compare measured_rate against delta.
    3. Compute the naive single-hop bound extrapolation (Declared) and record whether
       it upper-bounds the measured rate (candidate Proven-subset signal) or diverges.

    Args:
        models: VSA models to sweep (default: ["mapi", "mapb"]).
        compositions: composition types (default: all three).
        F_values: factor slot counts (default: [2, 3]).
        k_values: codebook sizes (default: [4, 8, 16]).
        d_values: dimensions (default: [512, 1024, 2048, 4096, 8192]).
        h_values: hop depths (default: [1, 2, 3]).
        delta: comparison failure probability.
        trials_per_point: Monte-Carlo trials per parameter combination.
        salt: base LCG seed.
        progress: print progress to stderr.
        Note: the n_distractors=100 used in single-hop trials is not exposed here;
        multi-hop trials do not use the distractor model (they use cleanup_exact on
        a finite codebook). The sweep counts failures as wrong-atom recoveries (EXPLAIN-able).

    Returns:
        List of MultihopResult, one per parameter combination.

    Guarantee: Empirical — all rates are trial-measured (VR-5).
    """
    if models is None:
        models = ["mapi", "mapb"]
    if compositions is None:
        compositions = ["bind_chain", "bundle_of_binds", "nested_unbind"]
    if F_values is None:
        F_values = [2, 3]
    if k_values is None:
        k_values = [4, 8, 16]
    if d_values is None:
        d_values = [512, 1024, 2048, 4096, 8192]
    if h_values is None:
        h_values = [1, 2, 3]

    # Deterministic integer lookup tables for finite string-valued parameters.
    # Using hash() on strings is randomized per process (PYTHONHASHSEED); these
    # lookup tables give fixed, portable integers — G2 / "Deterministic LCG seeds".
    _MODEL_ID: dict[str, int] = {"mapi": 0, "mapb": 1, "hrr": 2, "fhrr": 3}
    _COMP_ID: dict[str, int] = {
        "bind_chain": 0,
        "bundle_of_binds": 1,
        "nested_unbind": 2,
    }

    results: list[MultihopResult] = []
    total = (
        len(models)
        * len(compositions)
        * len(F_values)
        * len(k_values)
        * len(d_values)
        * len(h_values)
    )
    done = 0

    for model in models:
        for comp in compositions:
            for F in F_values:
                for k in k_values:
                    for h in h_values:
                        for d in d_values:
                            t0 = time.monotonic()
                            failures = 0
                            trial_fn = {
                                "bind_chain": _bind_chain_trial,
                                "bundle_of_binds": _bundle_of_binds_trial,
                                "nested_unbind": _nested_unbind_trial,
                            }[comp]
                            for trial in range(trials_per_point):
                                # Deterministic arithmetic seed — no hash() of strings
                                # (Python hash() is randomized per-process unless
                                # PYTHONHASHSEED=0; this lookup + arithmetic is portable).
                                seed = salt ^ (
                                    _MODEL_ID[model] * 1_000_000_007
                                    + _COMP_ID[comp] * 100_003_003
                                    + F * 9_999_991
                                    + k * 999_983
                                    + h * 99_991
                                    + d * 7
                                    + trial
                                )
                                lcg = Lcg(seed)
                                if trial_fn(F, k, d, h, model, lcg):
                                    failures += 1

                            elapsed = time.monotonic() - t0
                            rate = failures / trials_per_point
                            n_m = _naive_extrapolated_m(comp, F, k, h)
                            n_req = required_dim(n_m, delta, MARGIN_MU)
                            n_holds = d >= n_req
                            n_respected = rate <= delta
                            # Diverges = naive bound says OK (n_holds) but rate > delta
                            diverges = n_holds and (rate > delta)

                            results.append(
                                MultihopResult(
                                    model=model,
                                    composition=comp,
                                    F=F,
                                    k=k,
                                    d=d,
                                    h=h,
                                    delta=delta,
                                    trials=trials_per_point,
                                    failures=failures,
                                    measured_rate=rate,
                                    naive_extrapolated_m=n_m,
                                    naive_required_dim=n_req,
                                    naive_bound_holds=n_holds,
                                    naive_bound_respected=n_respected,
                                    bound_diverges=diverges,
                                    elapsed_s=elapsed,
                                )
                            )
                            done += 1
                            if progress and done % 10 == 0:
                                print(
                                    f"  [{done}/{total}] {model} {comp} "
                                    f"F={F} k={k} h={h} d={d}: "
                                    f"rate={rate:.4f} diverges={diverges}",
                                    file=sys.stderr,
                                )
    return results
