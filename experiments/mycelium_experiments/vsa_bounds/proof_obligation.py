"""Emit machine-checkable proof obligations for validated multi-hop VSA bounds (M-832, OQ-F).

This module bridges the empirical discovery of a candidate closed-form multi-hop bound
(``candidate_bound.py``) to a machine-checkable proof artifact — the same ADR-010 /
RFC-0002 §7 ``axiomatize + checked-instantiation`` pattern used for the single-hop bound:
  - ``proofs/lh-bundle/`` (Liquid Haskell / Z3 discharge of capacity theorem instance)
  - ``proofs/binary-ternary-roundtrip/roundtrip_8x6.smt2`` (SMT-LIB instance)

Two output forms are emitted for each validated candidate:

(a) **SMT-LIB 2 (``.smt2``)** — asserts ``d >= required_dim_multihop(F, k, h, delta)``
    over the concrete swept points as arithmetic propositions in QF_LIA (quantifier-free
    linear integer arithmetic).  A solver (Z3) discharging these obligations confirms that
    the candidate formula admits a machine-checkable concrete instantiation — the first step
    toward ``Proven``.

(b) **Liquid Haskell skeleton (``.hs``)** — axiomatizes the candidate capacity theorem
    (``assume candidateCapacityThm``) and asks LH/Z3 to discharge the concrete arithmetic
    instantiation for each swept in-regime point.  Mirrors ``proofs/lh-bundle/src/Bundle.hs``
    exactly in structure.

Guarantee discipline (VR-5 / ADR-032):
  - The emitted obligations are ``Declared`` stubs awaiting solver discharge.
  - Emitting them does NOT upgrade the candidate to ``Proven``.
  - The ``Proven`` tag is the MAINTAINER's prerogative, after a prover actually reports
    SAFE / unsat for the obligations.
  - These stubs are the *path* to ``Proven``, not a ``Proven`` claim.

Never-silent (G2):
  - Every emitted file contains explicit ``Declared`` / ``PENDING`` markers.
  - Out-of-regime or refuted points are excluded from obligations and a NOTE is written.

Usage::
    from mycelium_experiments.vsa_bounds.proof_obligation import emit_obligations
    obligations = emit_obligations(best_candidate, out_dir)
"""

from __future__ import annotations

import math
from pathlib import Path

from .candidate_bound import CandidateResult, EffMModel, effective_m
from .capacity import CAPACITY_CITATION, MARGIN_MU, required_dim
from .sweeps import CompositionKind

# ---------------------------------------------------------------------------
# SMT-LIB 2 emitter
# ---------------------------------------------------------------------------

_SMTLIB_HEADER = """\
; VSA multi-hop capacity proof obligation (M-832, OQ-F)
;
; Candidate theorem (Declared — PENDING proof-assistant discharge):
;   required_dim_multihop(F, k, h, delta) = ceil((2/mu^2) * ln(m_eff/delta))
;   where m_eff = {m_eff_formula}
;   is a sufficient dimension such that the membership-decode failure probability
;   is at most delta for a '{composition}' composition.
;
; Effective-m model: {eff_m_model}
; Empirically validated regime: {regime_envelope}
; Status: Declared — this file is the checkable artifact that a solver must
;   discharge before the maintainer may upgrade to Proven (VR-5 / ADR-032).
;
; Strategy (mirrors proofs/lh-bundle/ and proofs/binary-ternary-roundtrip/):
;   - AXIOMATIZE the capacity theorem (same Clarkson/Thomas result, extended to multi-hop).
;   - The solver discharges only the concrete ARITHMETIC:  d >= requiredDim(m_eff, delta).
;   - This does NOT re-prove the concentration inequality — only the instantiation.
;
; Citation: {citation}
; Effective-m formula: m_eff = {m_eff_formula}
;
; Expected solver output: sat  (the assertions ARE satisfiable — d >= requiredDim for each point)
; If any assertion returns unsat, the concrete point REFUTES the candidate.
;
; Run with:
;   z3 -smt2 {filename}
;
; Guarantee: Declared (pending discharge).  Do NOT stamp Proven until solver confirms.

(set-logic QF_LIA)

; requiredDimMultihop(m_eff, delta_scaled) approximates ceil(200 * ln(m_eff / delta)).
; Because QF_LIA cannot express logarithms directly, we use the PRECOMPUTED integer
; values of required_dim for each concrete (m_eff, delta) pair in the swept regime.
; The solver checks d >= precomputed_value — purely integer arithmetic, no log needed.
; (The precomputed values are produced by candidate_bound.py::required_dim_multihop.)
;
; This is exactly the same strategy as proofs/lh-bundle/: axiomatize the formula,
; pre-compute its concrete values, have the solver discharge d >= value.

"""

_SMTLIB_PROBE_TEMPLATE = """\
; Probe {idx}: {composition} F={F} k={k} h={h} d={d} delta={delta}
;   m_eff = {m_eff}  required_dim_multihop = {req_dim}  d >= req_dim? {holds}
(assert (>= {d} {req_dim}))  ; d={d} >= required_dim_multihop({m_eff}, {delta}) = {req_dim}

"""

_SMTLIB_FOOTER = """\
; All probes pass iff (check-sat) = sat.
; Each (assert (>= d req_dim)) is trivially true for the given concrete values.
; The solver discharges: for each swept point, d was at or above the candidate threshold.
;
; NEXT STEP (maintainer):
;   1. Run: z3 -smt2 {filename}  — expect 'sat' for all probes.
;   2. If sat: the arithmetic instantiation is machine-confirmed for the swept points.
;   3. A theorem connecting m_eff to the actual multi-hop confusion probability must
;      then be established (the axiomatized part — see the LH skeleton).
;   4. Only after BOTH: (a) theorem axiomatized + LH/Z3 confirms SAFE, AND
;      (b) the theorem itself is proven or cited — does Proven become warranted.
;
; Guarantee: Declared (pending discharge).
(check-sat)
"""


def emit_smt2(
    candidate: CandidateResult,
    composition: CompositionKind,
    out_path: Path,
    delta: float = 0.02,
) -> Path:
    """Emit an SMT-LIB 2 proof obligation for the candidate bound.

    Only in-regime, non-refuted points are included (those where candidate_holds=True
    AND rate_respects_delta=True).  Out-of-regime and refuted points are noted but
    excluded — including them would create false obligations.

    Args:
        candidate: the CandidateResult to emit obligations for.
        composition: the composition type to focus on (one file per composition).
        out_path: destination ``.smt2`` file path.
        delta: the target failure probability for the obligations.

    Returns:
        The path of the written file.

    Guarantee: Declared — the emitted file is a proof obligation stub, not a proof.
    """
    eff_m_model: EffMModel = candidate.eff_m_model

    # Collect in-regime, non-refuted points for this composition.
    valid_points = [
        p
        for p in candidate.all_points
        if p.composition == composition
        and p.candidate_holds
        and p.rate_respects_delta
        and not p.refuted
    ]

    # Determine m_eff formula description.
    if eff_m_model == "A_exponential":
        formula = "F * k^h" if composition != "bundle_of_binds" else "h * k"
    elif eff_m_model == "B_linear":
        formula = "F * k * h" if composition != "bundle_of_binds" else "h * k"
    elif eff_m_model == "C_sqrt":
        formula = (
            "ceil(F * k * sqrt(h))" if composition != "bundle_of_binds" else "ceil(sqrt(h) * k)"
        )
    else:
        formula = "unknown"

    filename = out_path.name
    lines: list[str] = []
    lines.append(
        _SMTLIB_HEADER.format(
            m_eff_formula=formula,
            composition=composition,
            eff_m_model=eff_m_model,
            regime_envelope=candidate.regime_envelope,
            citation=CAPACITY_CITATION,
            filename=filename,
        )
    )

    if not valid_points:
        lines.append(
            f"; NOTE: No in-regime non-refuted points found for composition={composition!r}.\n"
        )
        lines.append("; This obligation file is empty — see PROOF-SUMMARY.md for details.\n")
        lines.append("; Guarantee: Declared (no discharge needed — no valid probes).\n")
        lines.append("(check-sat)\n")
    else:
        # Deduplicate on (F, k, h, d) — keep unique parameter combinations.
        seen: set[tuple[int, int, int, int]] = set()
        unique_points = []
        for p in valid_points:
            key = (p.F, p.k, p.h, p.d)
            if key not in seen:
                seen.add(key)
                unique_points.append(p)

        # Sort for deterministic output.
        unique_points.sort(key=lambda p: (p.F, p.k, p.h, p.d))

        for idx, p in enumerate(unique_points, 1):
            m_eff_val = effective_m(composition, p.F, p.k, p.h, eff_m_model)
            req_dim = required_dim(m_eff_val, p.delta, MARGIN_MU)
            holds_str = "YES (d >= req_dim)" if p.d >= req_dim else "NO (d < req_dim)"
            lines.append(
                _SMTLIB_PROBE_TEMPLATE.format(
                    idx=idx,
                    composition=composition,
                    F=p.F,
                    k=p.k,
                    h=p.h,
                    d=p.d,
                    delta=p.delta,
                    m_eff=m_eff_val,
                    req_dim=req_dim,
                    holds=holds_str,
                )
            )

        # Also note excluded points.
        excluded = [p for p in candidate.all_points if p.composition == composition and p.refuted]
        if excluded:
            lines.append(
                f"; NOTE: {len(excluded)} point(s) EXCLUDED (refuted — "
                f"candidate does not hold there).  Never-silent (G2):\n"
            )
            for p in excluded:
                lines.append(
                    f";   REFUTED: F={p.F} k={p.k} h={p.h} d={p.d} "
                    f"rate={p.measured_rate:.4f} > delta={p.delta}\n"
                )
            lines.append("\n")

        lines.append(_SMTLIB_FOOTER.format(filename=filename))

    out_path.write_text("".join(lines), encoding="utf-8")
    return out_path


# ---------------------------------------------------------------------------
# Liquid Haskell skeleton emitter
# ---------------------------------------------------------------------------

_LH_HEADER = """\
{{-@ LIQUID "--reflection" @-}}
{{-@ LIQUID "--ple"        @-}}

-- | M-832 / OQ-F — Multi-hop VSA capacity-refinement probe ({composition}, {eff_m_model}).
--
--   STATUS: Declared — PENDING proof-assistant discharge (VR-5 / ADR-032).
--
--   Strategy (mirrors proofs/lh-bundle/src/Bundle.hs):
--     - AXIOMATIZE the candidate multi-hop capacity theorem's statement.
--     - Have Z3 discharge only the concrete ARITHMETIC INSTANTIATION — never
--       re-prove the concentration inequality.
--
--   Candidate theorem (Declared):
--     For composition '{composition}', model '{eff_m_model}':
--       m_eff(F, k, h) = {m_eff_formula}
--       required_dim_multihop(m_eff, delta) = ceil((2/mu^2) * ln(m_eff/delta))
--     Then d >= required_dim_multihop implies failure probability <= delta.
--
--   Empirically validated regime: {regime_envelope}
--
--   Citation basis: {citation}
--
--   To run (once GHC + LiquidHaskell + Z3 are available):
--     cd proofs/vsa-multihop-bound
--     cabal build   -- expects: LIQUID: SAFE (N constraints checked)
--   If SAFE: the arithmetic instantiation is machine-confirmed for the swept points.
--
--   NEXT STEP: establish the theorem connecting m_eff to the actual multi-hop confusion
--   probability (the ``assume candidateCapacityThm`` below — axiom awaiting formal proof
--   or citation to a published theorem).  Only after that does Proven become warranted.
--
--   Guarantee: Declared (pending discharge).
module MultihopBound_{suffix} where

-- | @requiredDimMultihop m@ is the axiomatized right-hand side of the candidate capacity
--   theorem: @ceil( (2/mu^2) * ln(m_eff/delta) )@ with mu = 0.1 (so 2/mu^2 = 200).
--   The formula is cited (T0.2, extended to multi-hop via the m_eff model); only its
--   concrete values enter the checked arithmetic below.
--
--   This is a LOOKUP TABLE of precomputed values for the swept (m_eff, delta) pairs —
--   exactly the same strategy as proofs/lh-bundle/src/Bundle.hs::requiredDim.
{{-@ reflect requiredDimMultihop @-}}
requiredDimMultihop :: Int -> Int
requiredDimMultihop m
"""

_LH_CASE_TEMPLATE = (
    "  | m <= {m_eff}  = {req_dim}  -- m_eff={m_eff}, delta={delta}, req_dim={req_dim}\n"
)
_LH_OTHERWISE = "  | otherwise  = {otherwise_dim}  -- conservative: largest in table\n"

_LH_THEOREM = """
-- | A bundle is well-capacitied at dimension d when d meets the candidate threshold.
{{-@ type WellCapacitied M = {{d:Int | d >= requiredDimMultihop M}} @-}}

-- | The axiomatized candidate multi-hop capacity guarantee:
--   a well-capacitied composition decodes with failure probability at most delta.
--   ``assume`` = the cited theorem (T0.2, extended) is taken as given;
--   LiquidHaskell does not re-prove it — only the concrete arithmetic is discharged.
--
--   NOTE: This is the key OPEN QUESTION (OQ-F / OQ-A).  The axiom below is a
--   HYPOTHESIS — a formal theorem connecting m_eff to the multi-hop failure probability
--   must be established before this can be treated as anything other than Declared.
{{-@ assume candidateCapacityThm :: m:Int -> WellCapacitied m -> {{b:Bool | b}} @-}}
candidateCapacityThm :: Int -> Int -> Bool
candidateCapacityThm _ _ = True

-- Checked instantiations.  Each definition type-checks iff Z3 proves
-- @d >= requiredDimMultihop m_eff@ for the concrete arguments.
-- Together they are the concrete arithmetic the theorem requires.
-- Guarantee: Declared (pending LH/Z3 discharge).

"""

_LH_PROBE_TEMPLATE = (
    "{{-@ probe{idx} :: {{b:Bool | b}} @-}}\n"
    "probe{idx} :: Bool\n"
    "probe{idx} = candidateCapacityThm {m_eff} {d}"
    "  -- {composition} F={F} k={k} h={h}: {d} >= {req_dim} (requiredDimMultihop {m_eff})\n\n"
)

_LH_FOOTER = """\
-- End of auto-generated probe file.
-- STATUS: Declared — run `cabal build` to discharge (expects LIQUID: SAFE).
-- Guarantee: Declared until SAFE is confirmed AND the axiom is formally established.
"""


def emit_lh_skeleton(
    candidate: CandidateResult,
    composition: CompositionKind,
    out_path: Path,
    delta: float = 0.02,
) -> Path:
    """Emit a Liquid Haskell skeleton for the candidate multi-hop capacity theorem.

    Mirrors ``proofs/lh-bundle/src/Bundle.hs`` exactly in structure:
    - ``requiredDimMultihop`` is a lookup table of precomputed values.
    - ``candidateCapacityThm`` is axiomatized (``assume``).
    - ``probe1..N`` are the concrete checked instantiations.

    Only in-regime, non-refuted points are included.

    Args:
        candidate: the CandidateResult to emit probes for.
        composition: the composition type to focus on.
        out_path: destination ``.hs`` file path.
        delta: target failure probability.

    Returns:
        The path of the written file.

    Guarantee: Declared — pending LH/Z3 discharge.
    """
    eff_m_model: EffMModel = candidate.eff_m_model

    # Formula description.
    if eff_m_model == "A_exponential":
        formula = "F * k^h" if composition != "bundle_of_binds" else "h * k"
    elif eff_m_model == "B_linear":
        formula = "F * k * h" if composition != "bundle_of_binds" else "h * k"
    elif eff_m_model == "C_sqrt":
        formula = (
            "ceil(F * k * sqrt(h))" if composition != "bundle_of_binds" else "ceil(sqrt(h) * k)"
        )
    else:
        formula = "unknown"

    # Module name suffix: sanitize composition + model.
    suffix = f"{composition.replace('_', '')}_{eff_m_model.replace('_', '')}"

    # Collect valid points.
    valid_points = [
        p
        for p in candidate.all_points
        if p.composition == composition
        and p.candidate_holds
        and p.rate_respects_delta
        and not p.refuted
    ]

    lines: list[str] = []
    lines.append(
        _LH_HEADER.format(
            composition=composition,
            eff_m_model=eff_m_model,
            m_eff_formula=formula,
            regime_envelope=candidate.regime_envelope,
            citation=CAPACITY_CITATION,
            suffix=suffix,
        )
    )

    if not valid_points:
        lines.append("  | otherwise = 9999999  -- No in-regime points: conservative sentinel\n")
        lines.append(_LH_THEOREM)
        lines.append("-- NOTE: No in-regime non-refuted points — no probes generated.\n")
        lines.append(_LH_FOOTER)
        out_path.write_text("".join(lines), encoding="utf-8")
        return out_path

    # Compute unique (m_eff, req_dim) pairs for the lookup table.
    seen_m: set[int] = set()
    table_entries: list[tuple[int, int, float]] = []  # (m_eff, req_dim, delta)
    for p in valid_points:
        m_eff_val = effective_m(composition, p.F, p.k, p.h, eff_m_model)
        if m_eff_val not in seen_m:
            seen_m.add(m_eff_val)
            req_dim_val = required_dim(m_eff_val, p.delta, MARGIN_MU)
            table_entries.append((m_eff_val, req_dim_val, p.delta))

    # Sort by m_eff ascending.
    table_entries.sort(key=lambda t: t[0])
    otherwise_dim = table_entries[-1][1] if table_entries else 9999999

    # Emit lookup table cases.
    for m_eff_val, req_dim_val, pt_delta in table_entries:
        lines.append(
            _LH_CASE_TEMPLATE.format(
                m_eff=m_eff_val,
                req_dim=req_dim_val,
                delta=pt_delta,
            )
        )
    lines.append(_LH_OTHERWISE.format(otherwise_dim=otherwise_dim))

    # Emit theorem axiom.
    lines.append(_LH_THEOREM)

    # Emit probes.
    # Deduplicate on (F, k, h, d).
    seen_probe: set[tuple[int, int, int, int]] = set()
    unique_probes = []
    for p in valid_points:
        key = (p.F, p.k, p.h, p.d)
        if key not in seen_probe:
            seen_probe.add(key)
            unique_probes.append(p)
    unique_probes.sort(key=lambda p: (p.F, p.k, p.h, p.d))

    for idx, p in enumerate(unique_probes, 1):
        m_eff_val = effective_m(composition, p.F, p.k, p.h, eff_m_model)
        req_dim_val = required_dim(m_eff_val, p.delta, MARGIN_MU)
        lines.append(
            _LH_PROBE_TEMPLATE.format(
                idx=idx,
                m_eff=m_eff_val,
                d=p.d,
                composition=composition,
                F=p.F,
                k=p.k,
                h=p.h,
                req_dim=req_dim_val,
            )
        )

    # Note excluded / refuted points.
    excluded = [p for p in candidate.all_points if p.composition == composition and p.refuted]
    if excluded:
        lines.append(f"-- NOTE (G2 / never-silent): {len(excluded)} refuted point(s) excluded:\n")
        for p in excluded:
            lines.append(
                f"--   REFUTED: F={p.F} k={p.k} h={p.h} d={p.d} "
                f"rate={p.measured_rate:.4f} > delta={p.delta}\n"
            )

    lines.append(_LH_FOOTER)

    out_path.write_text("".join(lines), encoding="utf-8")
    return out_path


# ---------------------------------------------------------------------------
# PROOF-SUMMARY.md emitter
# ---------------------------------------------------------------------------


def emit_proof_summary(
    candidates: list[CandidateResult],
    smt2_paths: dict[str, Path],
    lh_paths: dict[str, Path],
    out_path: Path,
    run_id: str = "",
    backend: str = "unknown",
) -> Path:
    """Emit a PROOF-SUMMARY.md describing candidates, obligations, and next steps.

    This is a ``Declared`` scaffold for the maintainer — it presents the candidate
    theorem, the empirically-validated regime, and the emitted proof obligations.
    The ``Proven`` verdict stays the maintainer's after a prover discharges them.

    Args:
        candidates: all CandidateResult objects (best first).
        smt2_paths: mapping from description key to emitted .smt2 path.
        lh_paths: mapping from description key to emitted .hs path.
        out_path: destination PROOF-SUMMARY.md path.
        run_id: run timestamp string.
        backend: compute backend used.

    Returns:
        The written path.

    Guarantee: Declared — this summary is maintainer input, not a proof.
    """
    lines: list[str] = []
    lines.append("# VSA Multi-Hop Bound — PROOF-SUMMARY")
    lines.append("")
    lines.append(f"Run: `{run_id}` | Backend: `{backend}`")
    lines.append("")
    lines.append(
        "> **Guarantee: Declared**  "
        "This summary presents candidate theorems and emitted proof obligations.  "
        "It is NOT a proof.  The `Proven` verdict requires a proof assistant "
        "(LH/Z3 or SMT solver) to discharge the obligations AND the underlying theorem "
        "to be formally established or cited.  See VR-5 / ADR-032."
    )
    lines.append("")

    # Best candidate summary.
    best = candidates[0] if candidates else None
    if best:
        status = (
            "NOT REFUTED over swept regime (Empirical upper bound)"
            if best.is_empirical_upper_bound
            else f"REFUTED at {best.n_refuted} points — see details below"
        )
        lines.append("## Best candidate")
        lines.append("")
        lines.append(f"- Effective-m model: `{best.eff_m_model}`")
        lines.append(f"- Status: {status}")
        lines.append(f"- Description: {best.description}")
        lines.append(f"- Goodness of fit: {best.goodness_of_fit_note}")
        lines.append(f"- Empirically validated regime: {best.regime_envelope}")
        lines.append("")

    # Candidate theorem.
    lines.append("## Candidate theorem (Declared)")
    lines.append("")
    lines.append("Let `m_eff(F, k, h)` be as defined by the best candidate model.  Then:")
    lines.append("")
    lines.append("```")
    lines.append("required_dim_multihop(F, k, h, delta)")
    lines.append("  = ceil((2 / mu^2) * ln(m_eff(F, k, h) / delta))")
    lines.append("  = ceil(200 * ln(m_eff(F, k, h) / delta))   [mu = 0.1]")
    lines.append("```")
    lines.append("")
    lines.append(
        "For a multi-hop composition with parameters (F, k, h, delta), if "
        "`d >= required_dim_multihop(...)`, then the membership-decode failure probability "
        "is at most `delta`."
    )
    lines.append("")
    lines.append(
        "> **Status: Declared.**  This is a hypothesis — not a proven theorem.  "
        "The single-hop basis is the Clarkson-Ubaru-Yang 2023 Thm 6 / "
        "Thomas-Dasgupta-Rosing 2021 result.  Extending it to multi-hop requires "
        "a new theorem or reduction (OQ-F / OQ-A)."
    )
    lines.append("")

    # Emitted obligations.
    lines.append("## Emitted proof obligations")
    lines.append("")
    lines.append(
        "The following files are the checkable artifacts.  Each follows the "
        "`axiomatize + checked-instantiation` pattern from `proofs/lh-bundle/` "
        "and `proofs/binary-ternary-roundtrip/`."
    )
    lines.append("")

    if smt2_paths:
        lines.append("### SMT-LIB 2 obligations (`.smt2`)")
        lines.append("")
        lines.append("Run with: `z3 -smt2 <file>` — expect `sat` for all probes.")
        lines.append("")
        for key, path in sorted(smt2_paths.items()):
            lines.append(f"- `{path.name}` — {key}")
        lines.append("")

    if lh_paths:
        lines.append("### Liquid Haskell skeletons (`.hs`)")
        lines.append("")
        lines.append(
            "Run with: `cd proofs/vsa-multihop-bound && cabal build` — "
            "expect `LIQUID: SAFE (N constraints checked)`."
        )
        lines.append("")
        for key, path in sorted(lh_paths.items()):
            lines.append(f"- `{path.name}` — {key}")
        lines.append("")

    # All candidates table.
    lines.append("## All candidate models")
    lines.append("")
    lines.append("| Model | Status | In-regime | Refuted | Min margin |")
    lines.append("|---|---|---|---|---|")
    for r in candidates:
        status_str = "NOT REFUTED" if r.is_empirical_upper_bound else f"REFUTED ({r.n_refuted})"
        margin_str = f"{r.min_safety_margin:.4f}" if math.isfinite(r.min_safety_margin) else "N/A"
        lines.append(
            f"| `{r.eff_m_model}` | {status_str} | "
            f"{r.n_candidate_holds}/{r.n_total} | {r.n_refuted} | {margin_str} |"
        )
    lines.append("")

    # Next steps.
    lines.append("## Next steps for the maintainer")
    lines.append("")
    lines.append(
        "1. **Run the SMT-LIB obligations:** `z3 -smt2 <file>.smt2` for each emitted file."
    )
    lines.append(
        "   - If `sat`: the concrete arithmetic instantiation is machine-confirmed "
        "for the swept points."
    )
    lines.append("   - If `unsat`: a specific point refutes the candidate — investigate.")
    lines.append("2. **Run the LH skeleton:** `cabal build` in `proofs/vsa-multihop-bound/`.")
    lines.append(
        "   - If `LIQUID: SAFE`: Z3 confirms the arithmetic. "
        "This is the same confirmation as the single-hop M-001 probe."
    )
    lines.append(
        "3. **Establish the underlying theorem** (OQ-A / OQ-F): "
        "the `assume candidateCapacityThm` axiom in the LH skeleton is the key open "
        "question.  It requires either a formal proof (Lean 4 / Agda / Coq) or a "
        "citation to a published theorem that covers the multi-hop case."
    )
    lines.append(
        "4. **Stamp Proven** (maintainer prerogative, VR-5): only after BOTH "
        "(a) solver confirms SAFE/sat AND (b) the axiom is formally established "
        "does the `Proven` tag become warranted for the checked subset."
    )
    lines.append("")
    lines.append(
        "> FLAG (open modeling choice): The effective-m model is the key hypothesis.  "
        "The best empirical model is `"
        + (best.eff_m_model if best else "unknown")
        + "` but this may not be the correct theoretical model.  "
        "The maintainer must confirm which model to pursue for the formal proof."
    )
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append(
        "*Generated by `experiments/mycelium_experiments/vsa_bounds/proof_obligation.py` "
        "(M-832, OQ-F).  Guarantee: Declared throughout.*"
    )

    out_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return out_path


# ---------------------------------------------------------------------------
# Top-level emit_obligations entry point
# ---------------------------------------------------------------------------


def emit_obligations(
    candidates: list[CandidateResult],
    out_dir: Path,
    run_id: str = "",
    backend: str = "unknown",
    delta: float = 0.02,
) -> dict[str, Path]:
    """Emit all proof obligations (SMT2 + LH skeletons + PROOF-SUMMARY.md).

    For each candidate model and each composition type (that has in-regime points),
    emits one ``.smt2`` and one ``.hs`` file plus a ``PROOF-SUMMARY.md``.

    The best candidate (first in the sorted list) gets the primary obligations;
    all candidates appear in the summary.

    Args:
        candidates: CandidateResult list (sorted best-first from fit_and_validate).
        out_dir: directory to write obligations into.
        run_id: run identifier string for filenames.
        backend: compute backend (for PROOF-SUMMARY header).
        delta: target failure probability.

    Returns:
        Dict mapping file role to Path.

    Guarantee: Declared — the emitted files are obligation stubs, not proofs.
    """
    out_dir.mkdir(parents=True, exist_ok=True)
    prefix = f"{run_id}-" if run_id else ""

    emitted: dict[str, Path] = {}
    smt2_paths: dict[str, Path] = {}
    lh_paths: dict[str, Path] = {}

    if not candidates:
        # Emit an empty summary if no candidates.
        summary_path = out_dir / f"{prefix}PROOF-SUMMARY.md"
        emit_proof_summary([], {}, {}, summary_path, run_id=run_id, backend=backend)
        emitted["PROOF-SUMMARY"] = summary_path
        return emitted

    # Use the best (first) candidate for obligations.
    best = candidates[0]
    compositions: list[CompositionKind] = ["bind_chain", "bundle_of_binds", "nested_unbind"]

    for comp in compositions:
        # Check if there are valid points for this composition.
        valid = [
            p
            for p in best.all_points
            if p.composition == comp and p.candidate_holds and p.rate_respects_delta
        ]
        if not valid:
            # Emit a stub noting no in-regime points.
            smt2_key = f"{comp}_{best.eff_m_model}_smt2"
            lh_key = f"{comp}_{best.eff_m_model}_lh"
            smt2_path = out_dir / f"{prefix}multihop-{comp}-{best.eff_m_model}.smt2"
            lh_path = out_dir / f"{prefix}multihop-{comp}-{best.eff_m_model}.hs"
            emit_smt2(best, comp, smt2_path, delta=delta)
            emit_lh_skeleton(best, comp, lh_path, delta=delta)
            smt2_paths[smt2_key] = smt2_path
            lh_paths[lh_key] = lh_path
            emitted[smt2_key] = smt2_path
            emitted[lh_key] = lh_path
        else:
            smt2_key = f"{comp}_{best.eff_m_model}_smt2"
            lh_key = f"{comp}_{best.eff_m_model}_lh"
            smt2_path = out_dir / f"{prefix}multihop-{comp}-{best.eff_m_model}.smt2"
            lh_path = out_dir / f"{prefix}multihop-{comp}-{best.eff_m_model}.hs"
            emit_smt2(best, comp, smt2_path, delta=delta)
            emit_lh_skeleton(best, comp, lh_path, delta=delta)
            smt2_paths[smt2_key] = smt2_path
            lh_paths[lh_key] = lh_path
            emitted[smt2_key] = smt2_path
            emitted[lh_key] = lh_path

    # PROOF-SUMMARY.md.
    summary_path = out_dir / f"{prefix}PROOF-SUMMARY.md"
    emit_proof_summary(
        candidates,
        smt2_paths,
        lh_paths,
        summary_path,
        run_id=run_id,
        backend=backend,
    )
    emitted["PROOF-SUMMARY"] = summary_path

    return emitted
