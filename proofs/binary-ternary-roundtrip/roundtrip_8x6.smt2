; M-121 — binary<->ternary round-trip / injectivity proof, fixed widths (8 bits <-> 6 trits).
;
; Obligation (P2, docs/spec/swaps/binary-ternary.md §4): the balanced-ternary value map on T_6 is
; INJECTIVE — two DISTINCT 6-trit vectors never denote the same integer. We assert the NEGATION
; (a collision: two distinct vectors with equal value) and expect `unsat`.
;
; Why injectivity discharges the round-trip (P1) for this pair:
;   * |T_6| = 3^6 = 729 vectors; the value map lands them in the integer range [-364, 364], which
;     has exactly 729 elements. Injective + |domain| = |codomain range| ⟹ BIJECTION onto
;     [-364, 364].
;   * Hence every integer in [-364,364] has a UNIQUE 6-trit representation; `enc` (the §3.1
;     digit-extraction) yields that unique vector and `dec` (= this value map) inverts it.
;   * B_8 = [-128,127] ⊆ [-364,364], so `enc` is total on B_8 and `dec(enc b) = b` for all b ∈ B_8.
; P4 (out-of-range `dec` = None) and P3 (Exact within range) are structural; the Rust exhaustive
; corpus (crates/mycelium-cert, all 256 bytes) additionally decides the concrete round-trip.
;
; This is the SMT-dischargeable fixed-width obligation RFC-0002 §4 anticipated. Run with:
;   z3 -smt2 roundtrip_8x6.smt2      (expected output: unsat)

(set-logic QF_LIA)

; Two candidate 6-trit vectors, MSB-first index by power: t_i is the digit for 3^i.
(declare-const t0 Int) (declare-const t1 Int) (declare-const t2 Int)
(declare-const t3 Int) (declare-const t4 Int) (declare-const t5 Int)
(declare-const s0 Int) (declare-const s1 Int) (declare-const s2 Int)
(declare-const s3 Int) (declare-const s4 Int) (declare-const s5 Int)

; Each digit is a balanced trit in {-1, 0, +1}.
(assert (or (= t0 (- 1)) (= t0 0) (= t0 1)))
(assert (or (= t1 (- 1)) (= t1 0) (= t1 1)))
(assert (or (= t2 (- 1)) (= t2 0) (= t2 1)))
(assert (or (= t3 (- 1)) (= t3 0) (= t3 1)))
(assert (or (= t4 (- 1)) (= t4 0) (= t4 1)))
(assert (or (= t5 (- 1)) (= t5 0) (= t5 1)))
(assert (or (= s0 (- 1)) (= s0 0) (= s0 1)))
(assert (or (= s1 (- 1)) (= s1 0) (= s1 1)))
(assert (or (= s2 (- 1)) (= s2 0) (= s2 1)))
(assert (or (= s3 (- 1)) (= s3 0) (= s3 1)))
(assert (or (= s4 (- 1)) (= s4 0) (= s4 1)))
(assert (or (= s5 (- 1)) (= s5 0) (= s5 1)))

; value(t) = Σ t_i · 3^i  (Horner / positional, binary-ternary.md §1).
(define-fun valt () Int
  (+ t0 (* 3 t1) (* 9 t2) (* 27 t3) (* 81 t4) (* 243 t5)))
(define-fun vals () Int
  (+ s0 (* 3 s1) (* 9 s2) (* 27 s3) (* 81 s4) (* 243 s5)))

; The two vectors denote the same integer ...
(assert (= valt vals))
; ... yet differ in at least one digit (a would-be collision).
(assert (or (not (= t0 s0)) (not (= t1 s1)) (not (= t2 s2))
            (not (= t3 s3)) (not (= t4 s4)) (not (= t5 s5))))

; No such collision exists ⟹ the value map is injective on T_6.
(check-sat)
