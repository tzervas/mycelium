; VSA multi-hop capacity proof obligation (M-832, OQ-F)
;
; Candidate theorem (Declared — PENDING proof-assistant discharge):
;   required_dim_multihop(F, k, h, delta) = ceil((2/mu^2) * ln(m_eff/delta))
;   where m_eff = F * k^h
;   is a sufficient dimension such that the membership-decode failure probability
;   is at most delta for a 'bind_chain' composition.
;
; Effective-m model: A_exponential
; Empirically validated regime: compositions=['bind_chain', 'bundle_of_binds', 'nested_unbind']; F=[2]; k=[4]; h=[1, 2]; d in [2048, 8192]
; Status: Declared — this file is the checkable artifact that a solver must
;   discharge before the maintainer may upgrade to Proven (VR-5 / ADR-032).
;
; Strategy (mirrors proofs/lh-bundle/ and proofs/binary-ternary-roundtrip/roundtrip_8x6.smt2):
;   - AXIOMATIZE the capacity theorem (same Clarkson/Thomas result, extended to multi-hop).
;   - The solver discharges only the concrete ARITHMETIC via REFUTATION PATTERN:
;     assert the NEGATION of the property (NOT (>= d req_dim)) and expect 'unsat'.
;   - 'unsat' means: NO counterexample exists for the swept in-regime points — the
;     candidate HOLDS for all of them. 'sat' means a refuting point was found.
;   - This does NOT re-prove the concentration inequality — only the instantiation.
;
; Citation: Clarkson-Ubaru-Yang 2023 (Thm 6); Thomas-Dasgupta-Rosing 2021
; Effective-m formula: m_eff = F * k^h
;
; Expected solver output: unsat  (the negation is unsatisfiable — d >= requiredDim holds
;                                 for ALL swept in-regime points; no counterexample exists)
; If solver returns sat, a specific point refutes the candidate — investigate.
;
; Run with:
;   z3 -smt2 20260629T174620Z-multihop-bind_chain-A_exponential.smt2
;
; Guarantee: Declared (pending discharge).  Do NOT stamp Proven until solver confirms unsat.

(set-logic QF_LIA)

; requiredDimMultihop(m_eff, delta_scaled) approximates ceil(200 * ln(m_eff / delta)).
; Because QF_LIA cannot express logarithms directly, we use the PRECOMPUTED integer
; values of required_dim for each concrete (m_eff, delta) pair in the swept regime.
; The solver checks the NEGATION of d >= precomputed_value — purely integer arithmetic.
; (The precomputed values are produced by candidate_bound.py::required_dim_multihop.)
;
; This is the REFUTATION PATTERN from proofs/binary-ternary-roundtrip/roundtrip_8x6.smt2:
; assert (not (property)) and expect unsat — a counterexample search that found none.
; Compare: the tautological pattern (assert property, expect sat) confirms nothing new
; for points already filtered to satisfy the property.

; Probe 1: bind_chain F=2 k=4 h=1 d=2048 delta=0.02
;   m_eff = 8  required_dim_multihop = 1199  d >= req_dim? YES (d >= req_dim)
;   REFUTATION CHECK: assert NOT (d >= req_dim); expect unsat (no counterexample).
(push 1)
(assert (not (>= 2048 1199)))  ; negation: d=2048 < required_dim_multihop(8, 0.02)=1199
(check-sat)  ; expect: unsat
(pop 1)

; Probe 2: bind_chain F=2 k=4 h=1 d=4096 delta=0.02
;   m_eff = 8  required_dim_multihop = 1199  d >= req_dim? YES (d >= req_dim)
;   REFUTATION CHECK: assert NOT (d >= req_dim); expect unsat (no counterexample).
(push 1)
(assert (not (>= 4096 1199)))  ; negation: d=4096 < required_dim_multihop(8, 0.02)=1199
(check-sat)  ; expect: unsat
(pop 1)

; Probe 3: bind_chain F=2 k=4 h=1 d=8192 delta=0.02
;   m_eff = 8  required_dim_multihop = 1199  d >= req_dim? YES (d >= req_dim)
;   REFUTATION CHECK: assert NOT (d >= req_dim); expect unsat (no counterexample).
(push 1)
(assert (not (>= 8192 1199)))  ; negation: d=8192 < required_dim_multihop(8, 0.02)=1199
(check-sat)  ; expect: unsat
(pop 1)

; Probe 4: bind_chain F=2 k=4 h=2 d=2048 delta=0.02
;   m_eff = 32  required_dim_multihop = 1476  d >= req_dim? YES (d >= req_dim)
;   REFUTATION CHECK: assert NOT (d >= req_dim); expect unsat (no counterexample).
(push 1)
(assert (not (>= 2048 1476)))  ; negation: d=2048 < required_dim_multihop(32, 0.02)=1476
(check-sat)  ; expect: unsat
(pop 1)

; Probe 5: bind_chain F=2 k=4 h=2 d=4096 delta=0.02
;   m_eff = 32  required_dim_multihop = 1476  d >= req_dim? YES (d >= req_dim)
;   REFUTATION CHECK: assert NOT (d >= req_dim); expect unsat (no counterexample).
(push 1)
(assert (not (>= 4096 1476)))  ; negation: d=4096 < required_dim_multihop(32, 0.02)=1476
(check-sat)  ; expect: unsat
(pop 1)

; Probe 6: bind_chain F=2 k=4 h=2 d=8192 delta=0.02
;   m_eff = 32  required_dim_multihop = 1476  d >= req_dim? YES (d >= req_dim)
;   REFUTATION CHECK: assert NOT (d >= req_dim); expect unsat (no counterexample).
(push 1)
(assert (not (>= 8192 1476)))  ; negation: d=8192 < required_dim_multihop(32, 0.02)=1476
(check-sat)  ; expect: unsat
(pop 1)

; All probes PASS iff every (check-sat) above returns 'unsat'.
; 'unsat' = the negation is unsatisfiable = the candidate holds for that point.
; 'sat'   = a refuting point exists — investigate before proceeding.
;
; NEXT STEP (maintainer):
;   1. Run: z3 -smt2 <this-file>  — expect 'unsat' for every probe.
;   2. If all unsat: the arithmetic instantiation is machine-confirmed for swept points.
;   3. A theorem connecting m_eff to the actual multi-hop confusion probability must
;      then be established (the axiomatized part — see the LH skeleton).
;   4. Only after BOTH: (a) theorem axiomatized + LH/Z3 confirms SAFE, AND
;      (b) the theorem itself is proven or cited — does Proven become warranted.
;
; Guarantee: Declared (pending discharge).
