"""Candidate closed-form multi-hop required-dimension bounds (M-832, OQ-F).

This module hypothesizes and validates parametric generalizations of the single-hop
``required_dim(m, delta)`` formula (Clarkson-Ubaru-Yang 2023 / Thomas-Dasgupta-Rosing 2021)
to multi-hop VSA compositions (bind-chains, bundle-of-binds, nested unbind).

Guarantee discipline (VR-5 / ADR-032):
  - The candidate *forms* are ``Declared`` — hypothesized but not theorem-backed.
  - Empirical validation against sweep data produces ``Empirical`` evidence that a
    candidate *is* or *is not* an upper bound over the swept regime.
  - This module NEVER stamps ``Proven``.  A candidate validated here is a ``Declared``
    theorem + ``Empirical`` envelope.  The ``Proven`` tag requires a proof assistant
    to discharge the concrete instantiation (see ``proof_obligation.py``).

Never-silent (G2):
  - Candidates that fail to upper-bound measured rates in *any* swept point are reported
    as REFUTED for that regime, not silently excluded.
  - Out-of-regime points (where the candidate predicts success but the formula is
    extrapolated beyond its empirically validated range) are flagged, not hidden.

Effective-m models (the core modeling choice — FLAG for maintainer):
  Three parametric ``m_eff`` hypotheses generalize the single-hop effective bundle size
  to composition depth ``h``, codebook size ``k``, and factor count ``F``:

  Model A — ``m_eff = m_0 * k^h`` (exponential growth per hop; conservative worst-case):
    Each bind adds roughly k-fold more candidate solutions → effective confusion set grows
    as k^h.  For bind_chain: m_0 = F (factor count).  For bundle_of_binds: m_0 = h * k.
    This is the model used in the existing ``_naive_extrapolated_m`` in sweeps.py.

  Model B — ``m_eff = m_0 * h * k`` (linear growth per hop; optimistic):
    Each additional hop adds k more items to the effective confusion set rather than
    multiplying.  For bind_chain: m_0 = F * k so m_eff = F * k * h.  For bundle_of_binds:
    m_eff = h * k (same as Model A for this composition).

  Model C — ``m_eff = m_0 * k * sqrt(h)`` (sub-linear; very optimistic):
    Empirically motivated by the observation that noise accumulation may be sub-exponential
    in practice.  m_eff = F * k * sqrt(h) for bind_chain.

  FLAG (open modeling choice): The correct ``m_eff`` model is UNKNOWN without a formal
  derivation from the capacity theorem.  Models A, B, C are hypotheses to be validated
  empirically and confirmed or refuted by a proof assistant.  The maintainer must choose
  which model to pursue for the formal proof after seeing the fit results.

Usage::
    from mycelium_experiments.vsa_bounds.candidate_bound import (
        fit_and_validate,
        CandidateResult,
        required_dim_multihop,
    )
"""

from __future__ import annotations

import dataclasses
import math
from typing import Literal

from .capacity import CAPACITY_CITATION, MARGIN_MU, required_dim
from .sweeps import CompositionKind, MultihopResult

# ---------------------------------------------------------------------------
# Effective-m model names and implementations
# ---------------------------------------------------------------------------

EffMModel = Literal["A_exponential", "B_linear", "C_sqrt"]

_EFFECTIVE_M_DESCRIPTIONS: dict[EffMModel, str] = {
    "A_exponential": (
        "m_eff = F * k^h (bind_chain/nested_unbind) or h*k (bundle_of_binds); "
        "exponential worst-case per hop.  Same as sweeps.py::_naive_extrapolated_m."
    ),
    "B_linear": (
        "m_eff = F*k*h (bind_chain/nested_unbind) or h*k (bundle_of_binds); "
        "linear growth per hop — optimistic hypothesis."
    ),
    "C_sqrt": (
        "m_eff = ceil(F*k*sqrt(h)) (bind_chain/nested_unbind) or ceil(sqrt(h)*k) "
        "(bundle_of_binds); sub-linear growth — very optimistic hypothesis."
    ),
}


def effective_m(
    composition: CompositionKind,
    F: int,
    k: int,
    h: int,
    model: EffMModel,
) -> int:
    """Compute the effective bundle size for a given composition and model.

    This is the key modeling choice that determines the candidate required dimension.
    The result feeds directly into ``required_dim(m_eff, delta)`` to produce the
    candidate multi-hop dimension bound.

    Args:
        composition: the composition type (bind_chain, bundle_of_binds, nested_unbind).
        F: number of factor slots.
        k: codebook size per slot.
        h: hop depth.
        model: effective-m hypothesis (A, B, or C).

    Returns:
        Effective bundle size (positive integer).

    Guarantee: ``Declared`` — this is a hypothesis, not a theorem.
    """
    if model == "A_exponential":
        # Same as sweeps.py::_naive_extrapolated_m — the baseline hypothesis.
        if composition == "bundle_of_binds":
            return h * k
        else:
            return F * (k**h)

    elif model == "B_linear":
        # Linear per-hop growth hypothesis.
        if composition == "bundle_of_binds":
            return h * k
        else:
            return F * k * h

    elif model == "C_sqrt":
        # Sub-linear (sqrt) per-hop growth — very optimistic.
        if composition == "bundle_of_binds":
            return max(1, math.ceil(math.sqrt(h) * k))
        else:
            return max(1, math.ceil(F * k * math.sqrt(h)))

    raise ValueError(f"unknown effective-m model: {model!r}")


def required_dim_multihop(
    composition: CompositionKind,
    F: int,
    k: int,
    h: int,
    delta: float,
    eff_m_model: EffMModel,
    mu: float = MARGIN_MU,
) -> int:
    """Candidate multi-hop required dimension.

    Computes ``required_dim(effective_m(F, k, h, model), delta, mu)`` — plugging the
    candidate effective-m into the single-hop Clarkson/Thomas formula.

    This is the candidate closed-form bound the experiment proposes.  It must be validated
    empirically (is it an upper bound on measured failure rates?) and then discharged by a
    proof assistant before any ``Proven`` claim is warranted.

    Args:
        composition: the composition type.
        F: number of factor slots.
        k: codebook size per slot.
        h: hop depth.
        delta: target failure probability.
        eff_m_model: effective-m hypothesis.
        mu: margin parameter (default 0.1; use only MARGIN_MU for Proven-candidate results).

    Returns:
        Candidate sufficient dimension (integer).

    Guarantee: ``Declared`` — formula output; empirical validity must be established
    separately via ``fit_and_validate``.
    """
    m_eff = effective_m(composition, F, k, h, eff_m_model)
    return required_dim(m_eff, delta, mu)


# ---------------------------------------------------------------------------
# Single-hop parity check
# ---------------------------------------------------------------------------


def single_hop_parity(F: int, k: int, delta: float, eff_m_model: EffMModel) -> bool:
    """Check that required_dim_multihop collapses to required_dim for h=1.

    For a bind-chain at h=1 (one hop): the effective m under any model should correspond
    to a dimension that is >= required_dim for a comparable single-hop bundle.  This is a
    necessary (but not sufficient) sanity condition on the candidate.

    Specifically:
      - Model A at h=1: m_eff = F*k^1 = F*k  (bind_chain)
      - Model B at h=1: m_eff = F*k*1 = F*k   (bind_chain) — same as A
      - Model C at h=1: m_eff = ceil(F*k*sqrt(1)) = F*k  (bind_chain) — same as A

    So all three models agree at h=1.  This checks that
    ``required_dim_multihop(h=1) == required_dim(effective_m(h=1))``.

    Returns:
        True iff the h=1 candidate collapses to single-hop required_dim for bind_chain.

    Guarantee: ``Empirical`` (a structural property check, not a proof).
    """
    m_eff_h1 = effective_m("bind_chain", F, k, 1, eff_m_model)
    req_candidate = required_dim_multihop("bind_chain", F, k, 1, delta, eff_m_model)
    req_single = required_dim(m_eff_h1, delta, MARGIN_MU)
    return req_candidate == req_single


# ---------------------------------------------------------------------------
# Per-point validation result
# ---------------------------------------------------------------------------


@dataclasses.dataclass
class PointValidation:
    """Validation of one candidate bound against one swept measurement point."""

    # Sweep point identification.
    model_vsa: str
    composition: CompositionKind
    F: int
    k: int
    d: int
    h: int
    delta: float

    # Candidate bound.
    eff_m_model: EffMModel
    m_eff: int
    candidate_dim: int  # required_dim_multihop(...)

    # Measurement.
    measured_rate: float
    trials: int

    # Validation.
    candidate_holds: bool  # d >= candidate_dim (side-condition satisfied)
    rate_respects_delta: bool  # measured_rate <= delta
    candidate_is_upper_bound: bool  # candidate_holds => rate_respects_delta
    # Out-of-regime: candidate says d is too small (candidate_holds=False) AND rate > delta.
    # This is expected failure — does NOT refute the candidate.
    out_of_regime: bool
    # Refutation: candidate says d is sufficient (holds) but rate > delta.
    # This REFUTES the candidate for this point.
    refuted: bool

    # Guarantee: always Empirical for the measurement; Declared for the candidate form.
    guarantee_measurement: str = "Empirical"
    guarantee_candidate: str = "Declared"


# ---------------------------------------------------------------------------
# CandidateResult — aggregate over a full sweep
# ---------------------------------------------------------------------------


@dataclasses.dataclass
class CandidateResult:
    """Aggregate validation of one effective-m candidate model against a multihop sweep.

    Attributes:
        eff_m_model: the effective-m hypothesis tested.
        description: human-readable description of the model.
        all_points: per-point validation results.
        n_total: total points.
        n_candidate_holds: points where d >= candidate_dim (in-regime).
        n_upper_bounded: points where candidate_holds AND rate <= delta.
        n_refuted: points where candidate_holds AND rate > delta (candidate fails here).
        n_out_of_regime: points where NOT candidate_holds (expected failure).
        is_empirical_upper_bound: True iff n_refuted == 0 (no refutation found).
        regime_envelope: descriptive string of the swept regime where the candidate held.
        refuted_points: subset of all_points where refuted=True (non-empty iff refuted).
        min_safety_margin: minimum (delta - measured_rate) across valid in-regime points;
            positive means slack; negative means a near-miss where the candidate holds.
        goodness_of_fit_note: qualitative note on fit quality.

    Guarantee: Empirical (based on sweep data) + Declared (candidate form).
    """

    eff_m_model: EffMModel
    description: str
    all_points: list[PointValidation]
    n_total: int
    n_candidate_holds: int
    n_upper_bounded: int
    n_refuted: int
    n_out_of_regime: int
    is_empirical_upper_bound: bool
    regime_envelope: str
    refuted_points: list[PointValidation]
    min_safety_margin: float  # min(delta - measured_rate) where candidate_holds; negative=near-miss
    goodness_of_fit_note: str

    guarantee: str = "Empirical+Declared"


# ---------------------------------------------------------------------------
# fit_and_validate — the core discovery function
# ---------------------------------------------------------------------------


def fit_and_validate(
    multihop_results: list[MultihopResult],
    eff_m_models: list[EffMModel] | None = None,
    delta: float | None = None,
) -> list[CandidateResult]:
    """Validate each candidate effective-m model against the multihop sweep data.

    For each model in ``eff_m_models``, for each swept point:
    1. Compute the candidate required dimension.
    2. Check whether ``d >= candidate_dim`` (candidate says d is sufficient).
    3. Check whether ``measured_rate <= delta`` (rate is within target).
    4. Classify as: upper_bounded, refuted, or out_of_regime.

    Reports all refutations — never drops them (G2: never-silent).

    Args:
        multihop_results: results from ``run_multihop_sweep``.
        eff_m_models: list of effective-m models to test (default: all three).
        delta: override the delta from sweep results (default: use per-point delta).

    Returns:
        One CandidateResult per model, sorted worst-to-best by refutation count.

    Guarantee: ``Empirical`` (measurement-based) + ``Declared`` (candidate forms).
    VR-5: no ``Proven`` tag is ever assigned here.
    """
    if not multihop_results:
        return []

    if eff_m_models is None:
        eff_m_models = ["A_exponential", "B_linear", "C_sqrt"]

    results: list[CandidateResult] = []

    for eff_m_model in eff_m_models:
        points: list[PointValidation] = []
        for r in multihop_results:
            d_delta = delta if delta is not None else r.delta
            m_eff_val = effective_m(r.composition, r.F, r.k, r.h, eff_m_model)
            cand_dim = required_dim(m_eff_val, d_delta, MARGIN_MU)
            cand_holds = r.d >= cand_dim
            rate_ok = r.measured_rate <= d_delta
            # Upper bound: IF candidate says d is sufficient, THEN rate must be <= delta.
            upper_bounded = cand_holds and rate_ok
            out_of_regime_pt = (not cand_holds) and (not rate_ok)
            refuted_pt = cand_holds and (not rate_ok)

            points.append(
                PointValidation(
                    model_vsa=r.model,
                    composition=r.composition,
                    F=r.F,
                    k=r.k,
                    d=r.d,
                    h=r.h,
                    delta=d_delta,
                    eff_m_model=eff_m_model,
                    m_eff=m_eff_val,
                    candidate_dim=cand_dim,
                    measured_rate=r.measured_rate,
                    trials=r.trials,
                    candidate_holds=cand_holds,
                    rate_respects_delta=rate_ok,
                    candidate_is_upper_bound=upper_bounded,
                    out_of_regime=out_of_regime_pt,
                    refuted=refuted_pt,
                )
            )

        n_total = len(points)
        n_holds = sum(p.candidate_holds for p in points)
        n_upper = sum(p.candidate_is_upper_bound for p in points)
        n_ref = sum(p.refuted for p in points)
        n_oor = sum(p.out_of_regime for p in points)
        is_ub = n_ref == 0

        refuted_pts = [p for p in points if p.refuted]

        # Safety margin: for genuine in-regime (holds AND not refuted) points only.
        # Excluding refuted points ensures min_margin cannot go negative due to refutations
        # — refutations are already counted in n_refuted / refuted_points (never-silent, G2).
        # Positive = slack; negative = near-miss (candidate barely clears the rate threshold).
        in_regime = [p for p in points if p.candidate_holds and not p.refuted]
        if in_regime:
            safety_margins = [p.delta - p.measured_rate for p in in_regime]
            min_margin = min(safety_margins)
        else:
            min_margin = float("nan")

        # Regime envelope description.
        if points:
            comps = sorted({p.composition for p in points if p.candidate_holds})
            hs = sorted({p.h for p in points if p.candidate_holds})
            ks = sorted({p.k for p in points if p.candidate_holds})
            Fs = sorted({p.F for p in points if p.candidate_holds})
            ds = sorted({p.d for p in points if p.candidate_holds})
            if comps:
                envelope = (
                    f"compositions={comps}; F={Fs}; k={ks}; h={hs}; d in [{min(ds)}, {max(ds)}]"
                )
            else:
                envelope = "no in-regime points found"
        else:
            envelope = "no sweep data"

        # Goodness-of-fit note.
        if n_ref > 0:
            gof = (
                f"REFUTED at {n_ref}/{n_total} points — candidate is NOT a universal upper "
                f"bound over the swept regime.  See refuted_points for details."
            )
        elif n_holds == 0:
            gof = (
                "No in-regime points (d < candidate_dim everywhere) — sweep is entirely "
                "out-of-regime; cannot validate."
            )
        else:
            margin_note = (
                f"min safety margin = {min_margin:.4f}" if math.isfinite(min_margin) else "N/A"
            )
            gof = (
                f"Upper bound HOLDS over all {n_holds} in-regime points "
                f"({n_upper}/{n_total} total). {margin_note}."
            )

        results.append(
            CandidateResult(
                eff_m_model=eff_m_model,
                description=_EFFECTIVE_M_DESCRIPTIONS[eff_m_model],
                all_points=points,
                n_total=n_total,
                n_candidate_holds=n_holds,
                n_upper_bounded=n_upper,
                n_refuted=n_ref,
                n_out_of_regime=n_oor,
                is_empirical_upper_bound=is_ub,
                regime_envelope=envelope,
                refuted_points=refuted_pts,
                min_safety_margin=min_margin,
                goodness_of_fit_note=gof,
            )
        )

    # Sort: non-refuted first, then by fewest refutations, then by most in-regime points.
    results.sort(key=lambda r: (r.n_refuted, -r.n_candidate_holds))
    return results


# ---------------------------------------------------------------------------
# Candidate theorem statements (Declared)
# ---------------------------------------------------------------------------

CANDIDATE_THEOREM_TEMPLATE = """\
Candidate theorem (Declared — not yet Proven):

  For a multi-hop VSA composition of type ``{composition}`` with parameters
  (F, k, h, delta), let

    m_eff_{model}(F, k, h) = {m_eff_formula}

  Then:

    required_dim_multihop(F, k, h, delta) = ceil((2 / mu^2) * ln(m_eff / delta))

  is a sufficient dimension such that the membership-decode failure probability
  is at most delta.

  Empirically validated over: {regime_envelope}

  Status: Declared (hypothesis). Empirical: validated (no refutation found) or
  refuted (see refuted_points). Proven: PENDING proof-assistant discharge of the
  concrete arithmetic instantiation (see proof_obligation.py).

Citation basis (if Proven): {citation}
Effective-m model: {model_description}
"""


def candidate_theorem_text(
    result: CandidateResult,
    composition: CompositionKind,
    F_values: list[int],
    k_values: list[int],
    h_values: list[int],
) -> str:
    """Format the candidate theorem statement for a given composition and model.

    Args:
        result: the CandidateResult to summarize.
        composition: the composition type this theorem covers.
        F_values: factor slot counts in the validated regime.
        k_values: codebook sizes in the validated regime.
        h_values: hop depths in the validated regime.

    Returns:
        Human-readable theorem statement (Declared).

    Guarantee: Declared — this is a formatted hypothesis, not a proof.
    """
    model = result.eff_m_model
    if model == "A_exponential":
        if composition == "bundle_of_binds":
            formula = "h * k"
        else:
            formula = "F * k^h"
    elif model == "B_linear":
        if composition == "bundle_of_binds":
            formula = "h * k"
        else:
            formula = "F * k * h"
    elif model == "C_sqrt":
        if composition == "bundle_of_binds":
            formula = "ceil(sqrt(h) * k)"
        else:
            formula = "ceil(F * k * sqrt(h))"
    else:
        formula = "unknown"

    return CANDIDATE_THEOREM_TEMPLATE.format(
        composition=composition,
        model=model,
        m_eff_formula=formula,
        regime_envelope=result.regime_envelope,
        citation=CAPACITY_CITATION,
        model_description=result.description,
    )


# ---------------------------------------------------------------------------
# Public summary of all candidate results
# ---------------------------------------------------------------------------


def summarize_candidates(results: list[CandidateResult]) -> str:
    """Return a human-readable summary of all candidate validation results.

    Guarantee: Empirical (measurement-based) summary. Never stamps Proven.
    """
    lines: list[str] = []
    lines.append("## Candidate multi-hop bound validation summary")
    lines.append("")
    lines.append(
        "**Guarantee: Empirical + Declared**  "
        "A non-refuted candidate is Empirical evidence that the formula "
        "upper-bounds the swept regime; it is NOT Proven.  "
        "Proof requires a proof assistant to discharge the checked instantiation "
        "(see ``proof_obligation.py``)."
    )
    lines.append("")

    for r in results:
        status = (
            "NOT REFUTED (Empirical upper bound over swept regime)"
            if r.is_empirical_upper_bound
            else f"REFUTED at {r.n_refuted} points"
        )
        lines.append(f"### Model {r.eff_m_model}: {status}")
        lines.append("")
        lines.append(f"Description: {r.description}")
        lines.append(f"Goodness of fit: {r.goodness_of_fit_note}")
        lines.append(f"In-regime points: {r.n_candidate_holds}/{r.n_total}")
        lines.append(f"Upper-bounded points: {r.n_upper_bounded}/{r.n_total}")
        lines.append(
            f"Min safety margin (delta - rate, in-regime): {r.min_safety_margin:.4f}"
            if math.isfinite(r.min_safety_margin)
            else "Min safety margin: N/A"
        )
        lines.append(f"Regime envelope: {r.regime_envelope}")
        if r.refuted_points:
            lines.append("")
            lines.append("**REFUTED points (never-silent — G2):**")
            for p in r.refuted_points[:10]:  # cap at 10 for readability
                lines.append(
                    f"  {p.composition} F={p.F} k={p.k} h={p.h} d={p.d}: "
                    f"candidate_dim={p.candidate_dim} rate={p.measured_rate:.4f} "
                    f"delta={p.delta} — REFUTED"
                )
            if len(r.refuted_points) > 10:
                lines.append(f"  ... and {len(r.refuted_points) - 10} more (see all_points).")
        lines.append("")

    return "\n".join(lines)
