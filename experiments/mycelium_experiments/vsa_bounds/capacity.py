"""MAP-I bundle capacity formula — Python reimplementation of `capacity.rs`.

Formula (Clarkson-Ubaru-Yang 2023 Thm 6; Thomas-Dasgupta-Rosing 2021):

    required_dim(m, delta) = ceil((2 / mu^2) * ln(m / delta)),  mu = 0.1

Cross-validation: the four (m, delta) pairs from `capacity.rs`'s test
`required_dim_matches_the_m001_probe_table` are reproduced exactly in
`test_vsa_bounds.py::test_required_dim_parity`.

Guarantee tag: `Proven` via cited theorem (same basis as the Rust impl — M-131;
RFC-0003 §5; Clarkson/Thomas).  This module ONLY computes the formula; it does NOT
issue Proven verdicts on multi-hop compositions — that is an open research question (OQ-F).
"""

from __future__ import annotations

import math

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

    Guarantee: `Proven` via cited theorem when mu == MARGIN_MU (the checked instantiation
    from M-001 / proofs/lh-bundle).  For other mu values the formula applies but the
    Proven basis is the raw theorem (not the checked M-001 instantiation).
    """
    import sys  # noqa: PLC0415 — lazy, avoids a module-level import for a sentinel

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
