"""MAP-I bundle capacity formula — Python reimplementation of `capacity.rs`.

Formula (Clarkson-Ubaru-Yang 2023 Thm 6; Thomas-Dasgupta-Rosing 2021):

    required_dim(m, delta) = ceil((2 / mu^2) * ln(m / delta)),  mu = 0.1

Cross-validation: the four (m, delta) pairs from `capacity.rs`'s test
`required_dim_matches_the_m001_probe_table` are reproduced exactly in
`test_vsa_bounds.py::test_required_dim_parity`.

Guarantee tag: `Proven` via cited theorem — but ONLY for single-hop bundle decode when
the returned dimension >= the argument AND mu == MARGIN_MU (same basis as the Rust impl —
M-131; RFC-0003 §5; Clarkson/Thomas; checked instantiation in proofs/lh-bundle/).
Multi-hop callers (e.g. candidate_bound.py) plug an effective_m value derived from
heuristic models — their results are `Declared` (candidate) + `Empirical` (swept), NOT
`Proven`. The `Proven` tag does NOT propagate to multi-hop compositions (OQ-F).
"""

from __future__ import annotations

import math
import sys

# The illustrative margin mu used in the M-001 LH probe and capacity.rs (MARGIN_MU = 0.1).
MARGIN_MU: float = 0.1

# Precomputed: 2 / mu^2 = 200.0
_COEFF: float = 2.0 / (MARGIN_MU**2)

CAPACITY_CITATION: str = "Clarkson-Ubaru-Yang 2023 (Thm 6); Thomas-Dasgupta-Rosing 2021"


def required_dim(items: int, delta: float, mu: float = MARGIN_MU) -> int:
    """Sufficient dimension for bundling `items` with failure probability <= delta.

    Mirrors `capacity.rs::required_dim` exactly:
    - items == 0 or invalid delta/mu -> returns a sentinel MAX (sys.maxsize here).
    - Otherwise: ceil((2/mu^2) * ln(items/delta)).

    Args:
        items: number of items to bundle (m).
        delta: target failure probability, must be in (0, 1].
        mu: margin parameter (default MARGIN_MU = 0.1; use only this value for Proven
            results matching the Rust impl).

    Returns:
        Sufficient dimension as an integer, or sys.maxsize for invalid inputs.

    Guarantee: `Proven` for single-hop bundle decode when the returned value >= `items`
    AND `mu == MARGIN_MU` (the checked instantiation from M-001 / proofs/lh-bundle;
    Clarkson-Ubaru-Yang 2023 Thm 6).  For other mu values, `Proven` follows the raw
    theorem (not the concrete M-001 checked instantiation).  When called from multi-hop
    callers (candidate_bound.py) with an effective_m argument, the result is `Declared`
    (heuristic m_eff model) + `Empirical` (validated by sweep) — NOT inherited `Proven`.
    """
    if items <= 0 or not math.isfinite(delta) or delta <= 0.0 or delta > 1.0:
        return sys.maxsize
    if not math.isfinite(mu) or mu <= 0.0:
        return sys.maxsize

    coeff = 2.0 / (mu * mu)
    val = coeff * math.log(items / delta)
    if not math.isfinite(val) or val < 0.0:
        return 0
    return math.ceil(val)


def proven_bound_holds(items: int, dim: int, delta: float, mu: float = MARGIN_MU) -> bool:
    """True iff dim >= required_dim(items, delta, mu) — the checked side-condition.

    Mirrors `capacity.rs::proven_capacity_bound` returning Some vs None.
    """
    req = required_dim(items, delta, mu)
    return dim >= req
