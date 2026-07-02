# Stage A — Correctness vs Specs + Test Quality

Status: Advisory — report only. Part of the 2026-06 deep review (see `00-summary.md`).
Baseline: HEAD `e2d627e`, branch `claude/codebase-review-security-audit-n9l2vj`. All 323
workspace tests pass; `cargo clippy -D warnings` (all targets/features) and `cargo fmt --check`
clean (re-confirmed this session). LiquidHaskell/z3 proof checks **skip** locally (cabal/z3
absent), so proof claims are unverifiable in this environment and are noted where relevant.

Review depth: T2 on all crates (fragile paths). Findings carry stable IDs (`A1-01`…) so the
later fix-scoping discussion can reference them. Severities per
`.claude/skills/_shared/review-rubric.md`. Every High finding below is either probe-confirmed
(a transient source mutation that was reverted immediately, working tree verified clean) or
independently source-verified by the orchestrator.

---

## A1 — mycelium-core + mycelium-cert (T2)

**Verdict.** High-quality, honesty-disciplined code. The two highest-risk invariants — the
guarantee-lattice meet direction (weakest-wins) and the single shared M-210
translation-validation checker — are implemented correctly per RFC-0001/0002 and are backed by
tests that genuinely fail under mutation (six probes, all CAUGHT). No swap can be silent: every
fallible path returns an explicit typed `Option`/`Err`, the bijective inverse is `Option`-typed
and its out-of-range path is tested, and every `Proven` tag is gated on a side-condition checked
in code (`legal_pair` for binary↔ternary, `dim ≥ requiredDim` for Dense↔VSA). No Critical or High
issues; findings are a stale doc comment, two narrow well-formedness/coverage gaps, and nits.

- **[Low] A1-01** `crates/mycelium-core/src/meta.rs:211-213` — `MetaWire` doc comment says
  `reconstruction` is "deferred (M-130) and not carried", but the field (`:225`) and both
  serde impls (`:238,259-262`) do carry and round-trip it. → A comment contradicting the code
  it documents is a grounding hazard in an honesty-rule codebase. → Update the comment.
- **[Low] A1-02** `crates/mycelium-core/src/bound.rs:99-108` — `Bound::well_formed` accepts
  non-finite magnitudes: `Error { eps: f64::INFINITY }` passes (`inf >= 0.0`), as does
  `Crosstalk { expected: inf, tail: Some(inf) }`. (NaN is correctly rejected.) → An infinite
  ε/crosstalk is a vacuous bound that the M-I2/M-I3 coupling still lets ride as `Proven`/
  `Empirical`; `mycelium_numerics::ErrorBound::new` rejects non-finite eps downstream, so core's
  own predicate is weaker than the numerics layer's. → Add `is_finite()` checks. (See also
  B2-03, A6-02.)
- **[Low] A1-03** `mycelium-core` bound/meta tests — no test feeds an infinite `eps`/`expected`
  to `well_formed`/`Meta::new`; `out_of_range_bound_is_malformed` only covers `delta = 1.5`. →
  The A1-02 gap is untested both directions. → Add `MalformedBound` assertions for infinite eps.
- **[Nit] A1-04** `crates/mycelium-core/src/recon.rs:18,131-135` — resonator-decode "basis must
  not exceed Empirical" is encoded as "is `ProvenThm`" rather than a rank comparison; correct
  today (ProvenThm is the only stronger variant) but fragile if the basis registry grows. →
  Express via `basis_strength(...).rank()`.
- **[Nit] A1-05** `crates/mycelium-cert/tests/sc3.rs:47-52` — the shared `assert_validated`
  helper asserts only `matches!(Validated { .. })`, discarding `strength`; the SC-3 sweep would
  not catch a swap validating at the wrong strength. The `dense_vsa.rs`/`dense.rs` tests pin
  strength, so it is covered elsewhere. → Have the helper assert an expected strength.

**Probe log (all CAUGHT; reverted immediately, scope clean, `cargo test -p mycelium-core -p
mycelium-cert` green afterward):** meet `>=`→`<=` → `meet_is_weakest_for_all_pairs`;
bijection re-derivation `|| true` → `check::tampered_target_diverges`; bounded tier-i gate
no-op → `dense::tampered_conversion_is_caught_by_the_checker`; VR-5 strength gate `if false` →
`check::strength_upgrade_past_the_basis_is_rejected`; `legal_pair` side-condition `if false` →
`check::failed_side_condition_is_a_mismatch`; δ-bound `profile_covers` `true ||` →
`dense_vsa::refusals_are_explicit`.

**Test quality:** assertions are strong (concrete values/variants); SC-2 bounds are
re-derived by an independent measurement (not the code under test); negative tests assert exact
variants; the `LosslessWithinRange` inverse `None` path is tested; every `Proven` is gated in
code and every `Empirical` has its validating trial. One weak helper (A1-05). The
`AmbiguousDecode` (dot==0) decode path has no dedicated test (structurally guarded).

---

## A2 — mycelium-numerics (T2)

**Verdict.** The kernel algebra is correct *in real arithmetic*: affine `mul` includes the full
`rad(x)·rad(y)` remainder, the δ path adds (never multiplies) and saturates at 1, apRHL `[SEQ]`
adds ε and clamps δ, and strength composes by weakest-wins meet (all four mutation probes
CAUGHT; oracles are independent hand-computed constants, not kernel re-derivation; trial counts
match docs). The honest gap is at the **floating-point layer**: every bound-combining site uses
round-to-nearest f64 with no outward rounding, while the rustdoc claims unqualified Soundness
and `compose_error_bound` emits `ProvenThm` on that basis — and the tier-i checker's absolute
`1e-12` tolerance makes it vacuous in exactly the float-round-off regime (ε ≲ 1e-12) ADR-010
assigns to this kernel.

- **[High] A2-01** `crates/mycelium-numerics/src/error.rs:66,88,112,152,191,222` (and
  `src/prob.rs:41,80`) — every ε/δ accumulation uses round-to-nearest f64 with no directed
  rounding or inflation; RN addition/multiplication can round the computed bound *below* the
  true mathematical bound, so the doc claim "the composed `eps` is a true upper bound"
  (`error.rs:13-15`) is false at ulp scale, yet `compose_error_bound` emits
  `BoundBasis::ProvenThm` (`cert.rs:242-244`) citing affine-arithmetic soundness whose FP
  side-condition is neither checked nor satisfied. → Under the rubric's strict reading
  ("`Proven` without checked side-conditions") this **borders Critical**; magnitude is ~n ulps
  but the honesty rule has no de-minimis clause. → Round bound-increasing results outward
  (`f64::next_up()`, stable in MSRV 1.92) or inflate by `(1 + k·EPSILON)`, *or* qualify the
  Soundness claim and reconsider the `ProvenThm` basis. (Verified: `error.rs:190` composes
  `eps: self.eps + other.eps` in plain f64; `cert.rs:242` returns `ProvenThm`.)
- **[High] A2-02** `crates/mycelium-numerics/src/cert.rs:26,106,121` — `CHECK_TOL = 1e-12` is an
  *absolute* slack (`claimed + 1e-12 >= recomputed`); for any composition whose re-derived bound
  is ≤ ~1e-12 the checker accepts *every* claim, including `ε = 0` against a recomputed `5e-13`
  (claiming exactness for an approximate result). f64 round-off bounds live at ~1.1e-16, so the
  trusted-base checker is vacuous in the very regime ADR-010 §1 assigns to this kernel; no test
  exercises the small-ε regime, so the hole is silent. → Use a relative tolerance, or strict
  `>=` against an outward-rounded re-derivation.
- **[High] A2-03** `crates/mycelium-numerics/src/error.rs:49-55` — `AffineForm::uncertain(c, s,
  radius)` silently collapses a non-finite radius to the exact constant: `uncertain(c, s,
  f64::INFINITY)` yields radius 0 — infinite uncertainty becomes a claimed-exact form with no
  flag, inverting soundness and violating house rule 2 ("out-of-range is an explicit
  `Option`/error"). The doc says "`center ± |radius|`" but a negative radius is *dropped*, not
  absolute-valued. → Return `Option`/saturate to an explicit top; align the doc.
- **[Medium] A2-04** `crates/mycelium-numerics/src/cert.rs:268-293` with `error.rs:209-214,
  220-225` — op parameters and composed outputs are never re-validated: `Scale(f64::NAN)` or
  `Mul { x0_mag: NAN }` emits a `Bound` with `eps = NaN`, and `1e308 + 1e308 → inf` breaks the
  documented finite invariant; `compose_error_bound` fails open. → Route the result through
  `ErrorBound::new` (returns `None` on NaN/inf).
- **[Medium] A2-05** `error.rs:165-170`, `prob.rs:16-19,57-63`, `cert.rs:148-156` —
  `ErrorBound`, `ProbBound`, `ApRhlJudgment`, `Certificate` all have fully `pub` fields, so the
  range-checked `new()` constructors are bypassable (`ProbBound { delta: -0.5 }` compiles);
  field docs assert invariants the types do not enforce. → Private fields + accessors (the
  kernel is the trusted base; "never silent" should be structural).
- **[Medium] A2-06** `error.rs:139-142,153` — `mul`'s freshness precondition is documented but
  unchecked; an aliased `fresh` lets the positive remainder cancel an existing negative
  coefficient, shrinking the radius. → `debug_assert!` the freshness, or return `Option`.
- **[Low] A2-07** `tests/properties.rs:110,124,134` — comment "sampled over worst-case-aligned
  deviations" overclaims (`dx = rng.unit() * ex` is uniform); the add assertion tests `dx + dy
  ≤ eps` without `abs()` (only the positive side); mul slack is `1e-6`. Together the suite
  structurally cannot detect the ulp-scale unsoundness of A2-01.
- **[Low] A2-08** `tests/properties.rs` — constructor-refusal paths largely untested
  (`ErrorBound::new(NaN/-1.0)`, `ProbBound::new(1.5/-0.1/NaN)`, arity-`Malformed`).
- **[Nit] A2-09** `cert.rs:22,242-244` — a `Proven⊕Proven` composition replaces input theorem
  citations with the single generic `AFFINE_CITATION`, losing provenance.

**Probe log:** add `*0.99` → CAUGHT (3 tests); strength fold takes stronger element → CAUGHT
(`compose_meets_strength_down`); union `Σδ`→`Πδ` → CAUGHT (4 tests); mul remainder `*0.5` →
CAUGHT (`affine_mul_is_sound`). **All 4 caught — but all probes were ≥1% perturbations; the
ulp-scale gap of A2-01/A2-07 would not be caught.** Reverted; 17/17 green afterward.

**Trial-count note.** `TRIALS = 20_000` matches the docs; the union test's 200,000 matches.
No doc presents these as confidence intervals; the header honestly calls them "de-facto
property tests" over a deterministic LCG. **No trial-count overclaim.** The LCG (MMIX
constants, top ~53 bits) is adequate for the ranges used, but fixed seeds mean reruns never
explore new inputs (one deterministic sample, honestly labeled), and coverage omits denormals,
large magnitudes, and signed-zero/NaN/inf adversarial inputs.

---

## A3 — mycelium-vsa + lh-bundle proofs (T2)

**Verdict.** A faithful, well-tested implementation of the crate's *own reading* of RFC-0003:
the tag matrix is mechanically self-enforced (`tests/matrix.rs` catches drift, probe-verified),
Empirical tags are backed by trial suites that run the declared counts, and the Proven capacity
bound is issued only behind the checked `dim ≥ requiredDim`. The problems are at the code↔RFC
seam: the code's matrix diverges from the RFC §4 *table* on 7 cells, and the MAP-I certified
`Proven` bundle checks only the dimension side-condition while skipping checkable preconditions
(bipolar alphabet, distinct items) the cited theorem assumes. The LiquidHaskell proof is
unverifiable here (cabal/z3 absent) and `Bundle.hs` carries a "NOT yet type-checked" header that
contradicts the README's "Discharged" status.

- **[High] A3-01** `crates/mycelium-vsa/src/matrix.rs:37,41,49,53,57` vs
  `docs/rfcs/RFC-0003-VSA-Submodule-Boundary.md:26-30` — five permute cells are tagged `Exact`
  where the normative §4 *table* says `Proven` (MAP-I, MAP-B, HRR, FHRR, SBC). Since `Exact ⊐
  Proven`, this is an upgrade past the normative row — the direction VR-5 forbids — justified
  only by the RFC's own contradictory "Net" summary (line 32, "permute (all) are
  Exact/algebraic"). The code silently picked the stronger reading. → Fix doc-side: an
  append-only erratum/r3 reconciling the table with the Net line, *or* downgrade the code tags
  until it lands. (Verified: `matrix.rs` tags all six models' Permute as `Exact`; RFC table
  rows 26-30 say `Proven` for all but BSC, while line 32 says all Exact — the RFC is internally
  inconsistent.)
- **[High] A3-02** `crates/mycelium-vsa/src/hrr.rs:147`, `crates/mycelium-vsa/src/fhrr.rs:142` —
  HRR/FHRR `Bind` is tagged `Exact`, but the §4 bind/unbind cell grants "at most Empirical"
  jointly and the Net line lists only MAP/BSC bind as Exact; the upgrade is grounded in "issue
  #61", which is not in `tools/github/issues.yaml` and not citable from the corpus. → Ratify in
  an RFC erratum or downgrade. (The critical cell, unbind, is correctly `Empirical` everywhere
  and pinned by `tests/matrix.rs:52-58`.)
- **[High] A3-03** `crates/mycelium-vsa/src/mapi.rs:121-163` with `capacity.rs:21,51-62` —
  `bundle_values_certified` issues a `Proven` `CapacityBound` after checking *only* `dim ≥
  requiredDim`. The cited Clarkson/Thomas theorems assume i.i.d. random bipolar atoms with a
  similarity margin; the code never checks the **checkable** proxies (MAP-I has no
  `check_bipolar` at all; duplicate items are not rejected though content hashes are computed at
  `mapi.rs:140`), and `MARGIN_MU = 0.1` is self-described as "illustrative". A `Proven` value
  with violated-but-checkable side-conditions (M-I2/VR-5); graded High rather than Critical only
  because the dimension side-condition *is* checked and a misbehaving caller is required. → Add
  alphabet + distinct-input checks to the certified path; measure/parameterize μ or record it in
  the bound basis so EXPLAIN exposes it.
- **[Medium] A3-04** `mapi.rs:185-194`, `mapb.rs:78-105,173-182` — MAP-I/MAP-B
  `bind_values`/`unbind_values` stamp `Exact` without enforcing the ±1 alphabet on which the
  self-inverse property holds; BSC/FHRR/SBC enforce their alphabets in `bind`. → Add
  `check_bipolar` to the MAP-I/MAP-B bind paths.
- **[Medium] A3-05** `proofs/lh-bundle/src/Bundle.hs:6-9` vs `proofs/lh-bundle/README.md` — the
  source header says "DRAFT / SCAFFOLD … NOT yet type-checked" while the README records "✅
  Discharged … LIQUID: SAFE (16 constraints)". The proof cannot be re-run here, so the claim
  rests on that README and the contradicting header undermines it. Proved-statement scope
  (verified by reading): `capacityThm` is `assume`d; Z3 discharges four integer inequalities
  against a hard-coded lookup table — the `⌈(2/μ²)·ln(m/δ)⌉` formula is outside the checked
  artifact. The Rust side describes this accurately and cross-pins the same constants, so no
  overclaim — but the README's displayed refinement signature is aspirational. → Update the
  `Bundle.hs` header to match the discharged status (with toolchain pin).
- **[Medium] A3-06** `bsc.rs:151-158` — BSC bundle's intrinsic tag is plain `Proven`,
  indistinguishable in the lattice from MAP-I's tail-bound Proven, though the RFC says "Proven
  **on expectation** … tag accordingly". Mitigated (no Proven *value* is issued; value path is
  Empirical). → A basis/qualifier field, or an RFC note that the lattice cannot carry it.
- **[Medium] A3-07** `recon.rs:58` — an empty/dim-mismatched `CleanupMemory` surfaces as
  `VsaError::EmptyBundle` ("bundle requires at least one item") — a semantically wrong variant;
  `reconstruct_role` also never verifies the supplied memory against the manifest's
  content-addressed codebook hashes. → Add `EmptyCodebook`/`CodebookMismatch`.
- **[Low] A3-08** `tests/capacity_validation.rs:44-45,72-76` — the SC-2 empirical assertion
  alone cannot detect bound weakening (probe P2b: at dim=115, 10× below the formula, the 10⁴
  failure rate is still ≤ δ because μ=0.1 is hugely conservative); tamper protection comes from
  the hard-coded sentinel `assert!(dim >= 1141)` and the pinned-table unit test. Fine as-is, but
  the comment should state tightness is not tested.
- **[Low] A3-09** `bsc.rs:189-203` — the documented BSC even-count tie-break is untested (probe
  P4: `n > half` → `n >= half` passed all BSC tests; ties impossible at odd m). → Add the
  BSC twin of MAP-B's tie test.
- **[Low] A3-10** `hrr.rs:35` — `HRR_UNBIND_PROFILE.trials = 2_000` (vs 10 000 elsewhere) at
  δ=1e-2 is thin resolution; codebook-size side-conditions live only in free-text `method`
  strings that `EmpiricalProfile::check` cannot gate on.
- **[Nit]** `cleanup.rs:72-99` — an all-NaN-similarity query returns `Match{index:0,
  confidence:-inf}` rather than `None`; variant inconsistency between `OutsideEmpiricalProfile`
  and `EmptyBundle` for the empty-codebook case.

**Probe log:** unbind drops involution → CAUGHT (3 tests); capacity formula 10× weaker →
CAUGHT (pinned-table test + sentinel); same with sentinel neutralized → **MISSED** (empirical
assertion has no power against weakening → A3-08); intrinsic Unbind `Empirical`→`Exact` →
CAUGHT (`every_model_matches_the_rfc0003_matrix`); BSC tie-break `>`→`>=` → **MISSED** (→ A3-09).
Reverted; full crate suite green afterward.

**Test quality:** `VsaError` variant coverage is essentially total (11/12 negatively tested
with the correct variant); capacity validation runs at the boundary dim with an independent
oracle (not a tautology) but can only validate non-vacuity, not tightness; Empirical ⇒
validating-trials holds everywhere (the strongest part of the crate); Proven ⇒ side-conditions
only partially checked (A3-03). Boundary gaps: no singleton-bundle, no dim-0/1 extremes.

---

## A4 — mycelium-l1 + mycelium-interp (T2)

**Verdict.** The L1 pipeline (lexer → recursive-descent parser → monomorphic checker → totality
checker → fuel/depth-guarded evaluator → L0 elaborator) and the M-110 reference interpreter are
unusually disciplined: explicit errors everywhere, no silent swaps, honest guarantee handling,
a genuinely discriminating 3-way differential, and golden values that are hand-computed, not
self-generated. Two real holes: a totality-checker soundness bug (a non-terminating function is
classified `Total` and admitted as `matured`), and an unbounded recursive-descent parser that a
crafted ~5k-deep input crashes (SIGABRT). The evaluator's fuel/depth guards still refuse to run
the non-total function, so meaning is safe (per RFC-0007 §4.5 "mis-gate affects packaging, not
meaning") — which caps the first at High rather than Critical.

- **[High] A4-01** `crates/mycelium-l1/src/totality.rs:187-204` — the totality checker
  classifies a **non-terminating** function as `Total` (and admits it as `matured`). At a
  `Match`, arm binders are added to `smaller` only when `scrut_small` is true, but **shadowing
  is never handled**: an arm binding a name already in `smaller` from an outer arm inherits the
  stale smallness. Witness (probe-verified to diverge and to be accepted as `matured`): `fn
  f(n,p) = match n { Z=>Z, S(m)=> match p { Z=>Z, S(m)=> f(m,p) } }` — `f(3,2)→f(1,2)→f(1,2)…`.
  Generalizes to constructor sub-binder rebinds. → Unsoundness in the gate the AOT-promotion
  trusts. → Mirror the `Let`/`For` shadow discipline (`totality.rs:206-241`) at `Match` arms:
  drop-and-restore prior `smaller` membership around each arm body.
- **[High] A4-02** `crates/mycelium-l1/src/parse.rs:352,549,578` — the recursive-descent parser
  has **no recursion-depth guard**; crafted deeply-nested input overflows the host stack and
  aborts (SIGABRT, exit 134). Measured: ~2000-deep parses, ~5000-deep aborts (≈10 KB input).
  The conformance corpus has no deep-nesting reject case, so the gap is invisible to CI.
  `check_colony` and `elaborate` recurse unboundedly over the same AST. → Contradicts the
  module's "never a panic" promise; `myc-check` is the M-002 oracle and an adversarial/
  LLM-generated input crashes it instead of returning exit 2. → Thread a depth budget (as
  `eval.rs` does) returning an explicit `ParseError`; add a `reject/09-deep-nesting.myc`
  fixture. (See B2-01 — corroborated independently from the security/DoS angle.)
- **[Low] A4-03** `crates/mycelium-l1/src/eval.rs:262-264` — the depth guard is charged per
  AST-node-entry rather than per call-frame, so a wide-but-shallow expression can hit
  `DEFAULT_DEPTH=64`. Correct and well-tested, but consider charging depth at `invoke` only, or
  document the 64-node nesting ceiling.
- **[Low] A4-04** `crates/mycelium-interp/tests/golden.rs` — no fuel-exhaustion-at-depth or
  `Wf`-error-path test beyond `fuel(0)`; `EvalError::Wf` is constructed but never asserted. →
  Add one test forcing a `Wf` violation if reachable, else note it's structurally unreachable.

**Test-quality — REJECT CORPUS (notable weakness).** `tests/conformance.rs:43-54` asserts only
`is_err()`, not *why*. A reject file failing for an unintended reason (e.g. a lexer error
masking the intended grammar violation) would silently pass. Hand-checked all 8 via `myc-check`:
all 8 currently reject for their documented reason, so the corpus is healthy *today*, but the
gate does not enforce it. → Assert a per-file expected-substring (and ideally position).

**Probe log:** totality always-`Total` → CAUGHT (`tests/check.rs`); drop per-node fuel charge →
CAUGHT (`differential.rs`); totality shadow-rebind → **MISSED** (→ A4-01); parser depth 5k →
SIGABRT (→ A4-02). Differential proven discriminating (`the_differential_distinguishes_
different_programs`). Golden values independently derived (not interpreter-generated). The 3-way
differential covers 10 programs but has no explicit `let`-shadowing/multi-arg-order case — a
wrong-order elaborator on ≥2 args is only indirectly covered.

---

## A5 — mycelium-select + mycelium-mlir + mycelium-dense (T2)

**Verdict.** All three crates are sound on their headline obligations. `mycelium-select` has
**no EXPLAIN-free selection path**: all five public entry points route through one `select`,
which builds the `Explanation` unconditionally — no black box (probe 2 confirms the content
assertions, not just presence). The **E3 wrong-layout differential is genuine, not vacuous**: it
injects a real `packed_as ≠ tag` misread, asserts the payload diverged (`assert_ne`), asserts
the specific `NotValidated{Diverged}` variant, and the mandatory probe confirms a no-op'd layout
check fails 2 of its 3 tests. `mycelium-dense`'s bound constants and Higham side-condition checks
are mathematically correct and the 20k-pair sweep catches a worsened op. Residual issues: one
content-addressing collision in select and several test-robustness items in dense.

- **[High] A5-01** `crates/mycelium-select/src/lib.rs:321-346` (with `:117,:387-391`) —
  `SelectionPolicy::new` validates `cost.storage_weight` for finiteness but never validates
  `Predicate::ErrorEpsAtMost(f64)` literals; serde_json canonicalizes both `NaN` and `+Inf` to
  `null`, so **two semantically opposite policies (never-matches vs matches-any) share one
  `PolicyRef`** — verified empirically (both hash to `blake3:d1052c…`). Content-addressing is the
  audit anchor recorded in `Meta.policy_used` (RFC-0005 §3); a registry lookup on the colliding
  ref resolves to whichever policy registered first, silently. Such a policy also fails its own
  JSON round-trip. → Reject non-finite predicate literals in `new` (recursive walk through
  `All`/`Any`/`Not`); add `PolicyError::BadPredicateLiteral`. (See B2-02.)
- **[Low] A5-02** `lib.rs:684-706` + `tests/packing.rs:148-162` —
  `select_layout`/`record_packing_layout` records `TritPacked{…}` onto a **non-ternary** value's
  `Meta`; nothing ties a `TritPacked` record to a `Ternary` repr (currently unobservable because
  `run_with_layout` ignores layout on non-ternary results). → A `WrongSiteKind`-style refusal.
- **[Low] A5-03** `crates/mycelium-mlir/src/pack.rs:108-134` — `unpack_trits` **silently
  truncates** when `bytes` is too short for `count`; it is `pub` and re-exported. → `Result` or
  a debug assertion (house "never silent").
- **[Low] A5-05** `crates/mycelium-dense/tests/rounding_bound.rs:63` — `let Ok(v) = result else
  { continue }`: if the ops regressed to always-refuse, both sweeps would pass **vacuously** (no
  success count asserted). The `continue` is dead code today. → Assert `successes == 3 * PAIRS`.
- **[Low] A5-06** `tests/rounding_bound.rs:5-6` — the header claim that the ±20 exponent window
  is "exact for f64 reference arithmetic" is false for add/sub (operands can span ~64 bits). The
  test is still valid (f64 roundoff ≪ slack), but in this repo stated groundings must be true. →
  Fix the comment ("exact for products; within 2⁻⁵³ ≪ ε for sums").
- **[Low] A5-07** `crates/mycelium-dense/src/lib.rs:34,39` — no test pins
  `F32_OP_REL_EPS`/`BF16_OP_REL_EPS` to their formulas; probe 4 (loosening F32 to `1e-3`)
  passed the whole dense suite (the sweep uses the same constant as tolerance and expected eps).
  Loosening is the sound direction, but the constant can silently drift from its `ProvenThm`
  citation. → `assert_eq!(F32_OP_REL_EPS, 2f64.powi(-24))` etc.
- **[Nit] A5-08** `lib.rs:223-230` vs `pack.rs:25-31` — the cost model prices TL2 at 1.67 b/w
  while the stand-in codec achieves 1.6; selection outcome unaffected. Also `select` recomputes
  `policy_ref()` (full serialize + hash) on every call.

**Probe log:** layout-tag no-op (`aot.rs:82`) → CAUGHT (`wrong_layout` `assert_ne` + tag-flip
test) — **"if the layout checker were a no-op, this test would fail: yes"**; EXPLAIN costs
emptied → CAUGHT by content assertions in all three suites; dense F32 ops worsened → CAUGHT;
F32 eps loosened to `1e-3` → **MISSED** (→ A5-07); serde non-finite collision → demonstrated
(→ A5-01). Reverted; 12/12 suites green afterward.

**Test quality:** assertions are strong (exact variants/values/hashes); the dense sweep fails on
a worsened op and on a bound tightened past truth but does not pin against loosening (A5-07);
MLIR differentials compare AOT against the independent reference interpreter (properly oracled,
shared-primitive blind spot documented); `Proven` on dense ops is backed by per-element checked
Higham side-conditions; EXPLAIN traces are tested for content, not mere presence.

---

## A6 — contracts + KC-2 harness + periphery (T1)

**Verdict.** The core serde types mirror the ratified schemas closely (field names, tag
spellings, presence-vs-null, M-I1…M-I4 enforced on construct and deserialize, probe-confirmed),
and the LSP/xtask periphery is honest and well-tested — but the contract is leaky at its edges:
Rust deserializers accept wire data every schema rejects (unknown fields; `EmpiricalFit{trials:
0}`), enum wire-spellings outside the four pinned examples are unprotected (a `TL2`→`Tl2x` drift
passes all core tests), and the KC-2 baseline DSL silently disagrees with the kernel on the sign
convention of the very swap the benchmark exercises (+178 vs −78), invisible because the
baseline oracle checks only result shape, never value.

- **[High] A6-01** `experiments/mycelium_experiments/kc2/baseline.py:38-40,116-123` — the
  baseline DSL interprets `Bin` as **unsigned**, but the kernel/spec interpret binary as
  **two's-complement** (`crates/mycelium-core/src/binary.rs:10-24`; `docs/spec/swaps/
  binary-ternary.md`). For task `kc2-05` the two arms compute different answers from the same
  prompt: baseline `+178`, kernel `−78` (the spec's own worked example) — probe-confirmed. The
  module docstring claims it "mirrors the minimal Mycelium surface fragment". → Oracle
  infidelity: the benchmark arms are not solving the same computation, and the shape-only
  checker cannot notice. → Make `Bin.to_int()` two's-complement (matching `binary.rs`), or drop
  the "mirrors" claim and document the divergence.
- **[High] A6-02** `crates/mycelium-core/src/{value.rs:185-190, meta.rs:214-228, recon.rs:
  190-200, bound.rs:95-108}` — Rust accepts wire data the schemas reject (probe-confirmed): (a)
  **unknown fields** — every schema declares `additionalProperties: false`, but the wire structs
  lack `#[serde(deny_unknown_fields)]`, so extra fields deserialize and are silently dropped; (b)
  `bound.schema.json` requires `EmpiricalFit.trials ≥ 1` and non-empty `method`/`citation`, but
  `Bound::well_formed()` checks only the kind payload, so `EmpiricalFit{trials: 0, method: ""}`
  is accepted — an Empirical tag backed by zero trials (M-I3 in letter, not spirit), and since
  `Bound`'s fields are `pub` the kernel can also *emit* that schema-invalid JSON. → The schema
  isn't a contract if one side doesn't enforce it. → Add `deny_unknown_fields` to the wire
  structs (Bound's `flatten` precludes it there — handle manually) and extend `well_formed()` to
  basis constraints. (See B2-03, A1-02.)
- **[Medium] A6-03** `crates/mycelium-core/tests/serde_roundtrip.rs:219-287` — schema-agreement
  pinning covers only the four `value/valid` examples; the wire spellings of
  `ErrorBound`/`ProbabilityBound`/`CrosstalkBound`, `NormKind`, `PackScheme` (`TL1`/`TL2`),
  `TritPacked`, `EmpiricalFit`/`UserDeclared` are exercised only by symmetric round-trips, which
  can't detect spelling drift. Probe: renaming the `TL2` tag to `Tl2x` passed all 65 core tests.
  → Extend the pinned corpus with one committed example per bound kind/basis/layout, or add an
  emit-then-`jsonschema`-validate test.
- **[Medium] A6-04** `experiments/mycelium_experiments/kc2/checkers.py:149-158` — the baseline
  oracle checks only `isinstance` + `width`; any wrong-valued result of the right shape passes
  (probe: mutating `tadd` to drop its second operand passed all 8 tests, including the
  well-posedness test). Symmetric with the Mycelium arm (typecheck-only), so the SC-5b metric is
  consistent — but reference solutions are never verified to compute correct values, which hides
  A6-01. → Add an `expect_value` to `Task` and assert it in the reference-solution test.
- **[Medium] A6-05** `crates/mycelium-lsp/src/feedback.rs:48-51` vs `:157-184` —
  `SwapSite::certificate` doc says `None` means "the reason is surfaced as a diagnostic, never
  silent", but the `_ => None` arm (statically-known unhandled pair, e.g. Binary→Dense) produces
  `certificate: None` with **no** diagnostic. → Emit an "unsupported-swap-pair" diagnostic, or
  narrow the doc.
- **[Medium] A6-06** `crates/mycelium-core/src/recon.rs:39-45,59-72,120-135` vs
  `reconstruction-manifest.schema.json` — bidirectional conditional drift: schema's `recipe`
  requires neither `roles` nor `structure` and leaves `structure` free-form, but Rust requires
  both and types `structure` as `ContentHash` (schema-valid manifests rejected); Rust requires
  `cleanup_threshold` for `Cleanup` (schema doesn't) but does not range-check it for `Resonator`
  (schema does). → Encode the conditionals in the schema and range-check stray optional params.
- **[Low] A6-07** `meta.rs:213` — stale doc: "`reconstruction` … not carried" yet `MetaWire`
  carries and round-trips it. (Same as A1-01.)
- **[Low] A6-08** `meta.rs:120-123` — an out-of-range `sparsity.density` returns
  `WfError::MalformedBound`, a misleading variant (it's an observation, not a bound).
- **[Low] A6-09** `crates/mycelium-cert/src/lib.rs:49-62` vs `swap-certificate.schema.json` —
  schema lists `params` as optional free-form; Rust `Bijective` requires typed `BinTernParams`,
  rejecting schema-valid certs without `params`. Minor while only one bijective kind exists.
- **[Low] A6-10** `checkers.py:111-117` — the "exec() is fixtures-only" invariant is
  docstring-only: untested and unenforced (and the exec namespace, lacking a `__builtins__` key,
  gets full builtins injected). → An explicit `allow_untrusted: bool = False` guard would make
  it structural. (See B2-04 for the security framing.)
- **[Nit] A6-11** `xtask/src/kc4.rs:152-167` — the bijective-dec path is timed without the
  `assert_validated` precheck the other paths get; a silently failing check would still be
  "measured".

**Probe log:** `tadd` drops 2nd operand → **MISSED** (shape-only oracle → A6-04); `swap` emits
width+1 → CAUGHT; sign-convention divergence demonstrated (→ A6-01); `MetaWire.bound` renamed →
CAUGHT; `PackScheme` `TL2`→`Tl2x` → **MISSED** (→ A6-03); extra-fields + `EmpiricalFit{trials:0}`
accepted by Rust, rejected by schema (→ A6-02). Reverted; scope clean.

**Test quality:** strong for the Rust core (exhaustive enumeration over finite spaces — lattice
laws 4×4×4, codec/arithmetic vs integer oracles; negative tests assert specific `WfError`
variants; the example-pinning idea is right, its weakness is coverage breadth — A6-03). The LSP
scripted client asserts real diagnostic/certificate/EXPLAIN content. The KC-2 Python tests
genuinely exercise harness logic (iteration counting, feedback threading, metric arithmetic,
VR-5 no-pre-written-verdict — probe-verified) but share the harness's blind spot: the baseline
oracle is shape-only, so semantic mutations (probe 1) and the A6-01 divergence are invisible.

---

## Stage A consolidated count

| Severity | IDs |
|---|---|
| Critical | (none) |
| High | A2-01, A2-02, A2-03, A3-01, A3-02, A3-03, A4-01, A4-02, A5-01, A6-01, A6-02 |
| Medium | A2-04, A2-05, A2-06, A3-04, A3-05, A3-06, A3-07, A6-03, A6-04, A6-05, A6-06 |
| Low | A1-01, A1-02, A1-03, A2-07, A2-08, A3-08, A3-09, A3-10, A4-03, A4-04, A5-02, A5-03, A5-05, A5-06, A5-07, A6-07, A6-08, A6-09, A6-10 |
| Nit | A1-04, A1-05, A2-09, A5-08, A6-11, plus the vsa `cleanup.rs` nit |

Cross-batch corroboration: A4-02≈B2-01 (parser DoS), A5-01≈B2-02 (PolicyRef collision),
A6-02≈B2-03≈A1-02 (manifest/bound well-formedness), A2-01≈C1-01 (numerics `Proven`/FP),
A3-01/02≈C1-03 (VSA matrix drift), A3-03≈C1-02 (VSA capacity side-conditions).
