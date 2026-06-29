; VSA multi-hop capacity proof obligation (M-832, OQ-F)
;
; Candidate theorem (Declared — PENDING proof-assistant discharge):
;   required_dim_multihop(F, k, h, delta) = ceil((2/mu^2) * ln(m_eff/delta))
;   where m_eff = F * k^h
;   is a sufficient dimension such that the membership-decode failure probability
;   is at most delta for a 'nested_unbind' composition.
;
; Effective-m model: A_exponential
; Empirically validated regime: no in-regime points found
; Status: Declared — this file is the checkable artifact that a solver must
;   discharge before the maintainer may upgrade to Proven (VR-5 / ADR-032).
;
; Strategy (mirrors proofs/lh-bundle/ and proofs/binary-ternary-roundtrip/):
;   - AXIOMATIZE the capacity theorem (same Clarkson/Thomas result, extended to multi-hop).
;   - The solver discharges only the concrete ARITHMETIC:  d >= requiredDim(m_eff, delta).
;   - This does NOT re-prove the concentration inequality — only the instantiation.
;
; Citation: Clarkson-Ubaru-Yang 2023 (Thm 6); Thomas-Dasgupta-Rosing 2021
; Effective-m formula: m_eff = F * k^h
;
; Expected solver output: sat  (the assertions ARE satisfiable — d >= requiredDim for each point)
; If any assertion returns unsat, the concrete point REFUTES the candidate.
;
; Run with:
;   z3 -smt2 20260629T164202Z-multihop-nested_unbind-A_exponential.smt2
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

; NOTE: No in-regime non-refuted points found for composition='nested_unbind'.
; This obligation file is empty — see PROOF-SUMMARY.md for details.
; Guarantee: Declared (no discharge needed — no valid probes).
(check-sat)
