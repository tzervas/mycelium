# Remediation Roadmap — From Findings to 1.0.0

Status: Advisory — planning artifact. Part of the 2026-06 deep review (see `00-summary.md`).
This file analyzes the 55 review findings, groups them into shippable workstreams, sequences
them, and proposes an explicit 1.0.0 readiness gate (the project currently has none). It changes
no code and ratifies no decision; the gate and the VSA reconciliation direction are flagged as
maintainer decisions to be ratified append-only (ADR/RFC) before execution.

## 1. Framing: where 1.0.0 actually sits

Three facts from the corpus set the scope:

- **Phase 2's exit gate is met** (`docs/planning/phase-2.md:§5`, 2026-06-12): the core is
  feature-complete for binary/ternary/dense/VSA with verified swaps, a single shared checker,
  selection + EXPLAIN, packing, and reconstruction. KC-1/KC-3/KC-4 were re-confirmed honest.
- **There is no formal 1.0.0 gate.** `CHANGELOG.md:7` — "Versioning will begin when the kernel
  does." No release-criteria document exists. Phase 2 completion is the *practical* 1.0-readiness
  point for the core language; Phase 3 (projections, JIT, factorization, BitNet acceleration,
  native codegen) is explicitly **maturation, post-1.0**.
- **The review found 0 Critical but 11 High issues, all clustered at the honesty-tag/contract
  seams.** The honesty rule (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`, per-op, never silent) is
  this project's entire value proposition. A `Proven` tag that isn't fully backed, or a schema
  "contract" one side doesn't enforce, is therefore not a cosmetic bug — it is the one defect
  class that most undermines the thing being shipped.

**Conclusion.** The road to 1.0.0 is not "more features" — it is closing the honesty-integrity
gap the review exposed and making those guarantees *durable* (mutation/property/fuzz), then
ratifying an explicit release gate. The remaining feature work (KC-2 verdict, native path) is
either an external blocker or Phase-3/post-1.0. This roadmap is therefore the bulk of the
1.0.0 critical path.

## 2. Triage

Every finding was re-checked; all are accepted as valid (no false positives to retract — the
duplicates were already merged in `00-summary.md`). One grading decision needs the maintainer:

> **Borderline Critical (recommend treating as must-fix-first regardless of label).** Under a
> strict reading of the rubric, four Highs graze *Critical*: **H1** (`ProvenThm` on unrounded
> f64) and **H6** (`Proven` capacity without checked side-conditions) are "`Proven` without
> checked side-conditions"; **H3** (`uncertain()` non-finite→exact) and **H11**
> (`EmpiricalFit{trials:0}` accepted) are silent honesty-surface holes. The reviewers held them
> at High because each needs a misbehaving caller / tampered input or is ulp-bounded. The
> sequencing below puts all four in Wave 1 either way, so the label choice does not change the
> plan — but it should be recorded.

The fixes are overwhelmingly **behavior-preserving in the safe direction**: outward rounding only
makes bounds *more conservative*; new validations turn previously-silent acceptance into explicit
refusals; depth guards turn a crash into a clean error. None removes intended functionality — they
make the existing guarantees honest, which is the stated goal ("retain all intended behavior").

## 3. Workstreams

Nine workstreams, each an independently-reviewable PR. Every workstream ships its fixes **plus a
regression test that fails without the fix** (the "mutant-witness" discipline from Stage D) and
re-runs the relevant Stage-A probes to confirm the previously-MISSED mutants are now caught.

### WS1 — Numerics honesty hardening (trusted base) · effort M · risk low · **LANDED (Wave 1)**

> **Status (2026-06-15):** implemented and merged on the working branch. A2-01/02/03/04/06/07/08
> done with regression tests (each citing its finding ID as a mutant-witness); workspace tests,
> `clippy -D warnings`, and `fmt` green. **Deferred (tracked):** A2-05 (private kernel-type fields —
> a cross-crate API change, kept out of the rounding fix) and A2-09 (citation provenance, Nit).

The single most on-theme workstream; it repairs the trusted-base checker the whole honesty story
rests on. Findings: **H1** (A2-01/C1-01), **H2** (A2-02), **H3** (A2-03), A2-04, A2-05, A2-06,
A2-07, A2-08, A2-09.

- Round every bound-increasing composition outward (`f64::next_up`, stable in MSRV 1.92) or
  inflate by `(1 + k·EPSILON)` before tagging `ProvenThm` (`error.rs:66,88,112,152,191,222`,
  `prob.rs:41,80`). Replace the absolute `CHECK_TOL = 1e-12` with a relative tolerance (or strict
  `>=` against an outward-rounded re-derivation) and add a small-ε regression test (`cert.rs:26`).
- Make `AffineForm::uncertain` and `compose_error_bound` fail closed on non-finite values
  (`Option`/explicit top), aligning the `|radius|` doc. Route composed outputs through
  `ErrorBound::new`. Add the freshness `debug_assert!` to `mul`. Make kernel-type fields private.
- Strengthen the property tests: `abs()` on the add-soundness assertion, worst-case-aligned
  sampling, constructor-refusal negative tests.
- **Restores:** VR-3, VR-5, SC-2 honesty for numerics; a non-vacuous tier-i checker (ADR-010).
- **Verify:** the WS1 regression suite + re-run A2 probes at ulp scale (the gap A2-07 flagged);
  `cargo test -p mycelium-numerics` green.

### WS2 — Contract / schema integrity (core) · effort M · risk low

Makes the JSON schema an enforced contract on both sides and closes the tamper vectors. Findings:
**H11** (A6-02/B2-03/A1-02), A6-03, A6-06, A1-01/A6-07, A1-03, A6-08, A6-09, A1-04, A1-05.

- Add `#[serde(deny_unknown_fields)]` to the wire structs (handle `Bound`'s `flatten` manually);
  extend `Bound::well_formed` to basis constraints (`EmpiricalFit.trials ≥ 1`, non-empty
  `method`/`citation`, `eps.is_finite()`).
- Add an emit-then-`jsonschema`-validate test and pin one committed example per bound
  kind / basis / layout (closes the enum-spelling-drift class, A6-03).
- Reconcile the `ReconInfo`/cert conditional drift with the schemas (A6-06, A6-09); fix the stale
  `reconstruction` doc comment (A1-01/A6-07), the `MalformedBound`-for-sparsity variant (A6-08),
  and the SC-3 `assert_validated` strength check (A1-05); express the recon basis check by rank
  (A1-04).
- **Restores:** the schema-as-contract; M-I1…M-I4 in full; tamper resistance for manifests.
- **Verify:** the schema round-trip/emit-validate test + the A6/A1 probes (extra-fields,
  `EmpiricalFit{trials:0}`, tag-rename) now rejected.

### WS3 — VSA tag honesty + RFC-0003 reconciliation · effort M · risk medium · **decision settled; erratum LANDED**

Findings: **H4** (A3-01/C1-03), **H5** (A3-02), **H6** (A3-03/C1-02), A3-04, A3-05, A3-06/C1-04,
A3-07, A3-08, A3-09, A3-10.

> **Decision settled (2026-06-15): option (a) — erratum ratifying Exact, keep the code.** RFC-0003 is
> now Accepted **(r3)** with a §4.1 erratum (append-only) grounding `permute = Exact` for all models
> and the HRR/FHRR `bind Exact` / `unbind Empirical` split on a checked algebraic basis. This closes
> **H4 and H5** with no code tag change (the code already matched the Net line); the code comment's
> non-citable "issue #61" rationale was replaced by the §4.1 citation.
>
> **Remaining WS3 (code):** **H6** (A3-03/C1-02, `Proven` capacity must check bipolar-alphabet +
> distinct-item side-conditions), A3-04 (MAP-I/MAP-B bind alphabet checks), A3-05 (`Bundle.hs` header
> vs README), A3-06/C1-04 (on-expectation `Proven` qualifier), A3-07 (`EmptyCodebook` variant),
> A3-08/09/10 (test gaps).

- After the decision: realign `matrix.rs` (or the RFC) so code and the normative table agree
  cell-by-cell; ground or remove the "issue #61" citation (H5).
- Add the checkable side-conditions to the certified capacity path (`bundle_values_certified`):
  bipolar-alphabet + distinct-item checks; parameterize/record μ in the bound basis so EXPLAIN
  exposes it (H6). Add `check_bipolar` to the MAP-I/MAP-B bind paths (A3-04).
- Reconcile `Bundle.hs` header vs README "Discharged" (A3-05); add a basis/qualifier (or RFC
  note) for on-expectation `Proven` (A3-06); add `EmptyCodebook`/`CodebookMismatch` (A3-07); add
  the missing BSC tie-break test and capacity-tightness comment (A3-08, A3-09); note HRR trial
  thinness (A3-10).
- **Restores:** per-op honesty (VR-5) for VSA; grounding discipline; append-only integrity.
- **Verify:** `tests/matrix.rs` re-pinned to the reconciled table; a Proven-capacity probe on a
  non-bipolar/duplicate input now refused.

### WS4 — L1 soundness + parser hardening · effort M · risk medium

Findings: **H7** (A4-01), **H8** (A4-02/B2-01), A4-03, A4-04, plus the reject-corpus
assertion-strength weakness.

- Fix the totality checker's `Match`-arm shadowing (mirror the `Let`/`For` drop-and-restore at
  `totality.rs:187-204`) so a non-terminating function is no longer classified `Total`/`matured`
  (H7). Add the two probe witnesses as regression tests.
- Thread an explicit recursion-depth budget through parser / typechecker / elaborator returning a
  clean error (H8); add a `reject/09-deep-nesting.myc` fixture. Switch the reject corpus from
  `is_err()` to per-file expected-error-substring assertions.
- Charge eval depth per call-frame (A4-03); add a `Wf`-error-path test or document unreachability
  (A4-04).
- **Restores:** soundness of the `matured`/AOT-promotion gate (RFC-0007 §4.5); the "never a panic"
  promise; `myc-check` robustness as the M-002 oracle.
- **Verify:** the divergent witnesses are now `Partial`; deep-nesting input returns exit-2, not
  SIGABRT; reject corpus fails if a file rejects for the wrong reason.

### WS5 — Selection integrity · effort S · risk low

Findings: **H9** (A5-01/B2-02), A5-02, A5-03, A5-05, A5-06, A5-07, A5-08.

- Reject non-finite `f64` predicate literals at construction/deserialization (recursive walk),
  restoring `PolicyRef` injectivity (H9); add `PolicyError::BadPredicateLiteral`.
- `WrongSiteKind`-style refusal for non-ternary layout records (A5-02); make `unpack_trits`
  return a `Result`/`debug_assert` (A5-03); assert the success count in the dense sweep (A5-05);
  fix the false "f64 exact" comment (A5-06); pin the `*_OP_REL_EPS` constants to their formulas
  (A5-07); cost-model figure cross-reference + `policy_ref` caching (A5-08).
- **Restores:** the content-addressed audit anchor (RFC-0005 §3); never-silent in the codec.
- **Verify:** the NaN/Inf-policy hash-collision probe now produces distinct refs/errors.

### WS6 — Harness oracle fidelity + LSP never-silent · effort S · risk low

Findings: **H10** (A6-01), A6-04, A6-05, A6-10/B2-04, A6-11.

- Make the KC-2 baseline DSL two's-complement to match the kernel/spec (H10), or document the
  divergence and drop the "mirrors" claim; add an `expect_value` oracle to `Task` and assert it
  in the well-posedness test (A6-04) — this is what makes the KC-2 verdict trustworthy *when*
  the LLM API lands.
- Emit an "unsupported-swap-pair" diagnostic for statically-known unhandled pairs in the LSP
  (A6-05, never-silent); gate `BaselineChecker.exec` behind `allow_untrusted=False` / strip
  `__builtins__` (A6-10/B2-04); add the `assert_validated` precheck to `xtask kc4` (A6-11).
- **Restores:** KC-2 experiment validity (so the eventual verdict means something); never-silent
  in the LSP facade.
- **Verify:** baseline and kernel agree on the `kc2-05` worked example; a value-wrong
  reference solution now fails the well-posedness test.

### WS7 — Security / supply-chain hardening (CI/infra) · effort S · risk low

Findings: B1-01, B1-02, B1-03, B1-04, B1-05, B1-06, B2-05, C1-08, C1-09. Plus the Stage-D infra
wiring (cargo-audit/deny, gitleaks) that naturally lives here.

- Pin every GitHub Action `uses:` to a full commit SHA (B1-01); pin `npx markdownlint-cli2` to a
  version (B1-02); add a scoped `dependabot.yml` (`github-actions` + `cargo` + `uv`) (B1-04) —
  raises PRs only, so it does not violate the manual-CI policy; optionally pin z3 (B1-03).
- `mktemp` in `gh-bootstrap-local.sh` (B1-05); tighten the gitleaks allowlist (B1-06); set
  `overflow-checks = true` for the trusted kernel crates' release profile (B2-05).
- Install gitleaks in CI + `install-tools.sh` so the full secret scan actually runs (C1-09,
  B1 gap); wire `cargo audit` + `cargo deny` into `scripts/checks/` (skip-if-missing pattern).
- Resolve the Black-vs-`ruff format` doc/tooling drift (C1-08) — either update the docs or record
  an ADR (per CLAUDE.md "don't silently change committed pins").
- **Restores:** supply-chain posture; local↔CI parity (gitleaks); never-silent in release arithmetic.
- **Verify:** CI run with pinned SHAs green; gitleaks runs (not the fallback); `cargo deny check`
  passes.

### WS8 — Durable test infrastructure (the QC/PE investment) · effort L · risk low

This is what turns "we fixed it once" into "it stays fixed," and is the direct answer to the
"coverage that tests the right thing" concern. Stage-D items.

- **cargo-mutants** on `mycelium-core`/`-cert`/`-numerics`/`-vsa` as an opt-in `just mutants`
  recipe (not in `just check`). It would have found the probe-MISSED gaps (A2-07, A3-08, A3-09,
  A4 reject corpus, A6-04) mechanically.
- **proptest migration** of the hand-rolled fixed-seed LCG property tests in `-numerics`/`-vsa`
  (shrinking, `PROPTEST_CASES`, CI seed-rotation) — resolves the "20k fixed sample ≠ statistical
  procedure" tension without changing the bound math.
- **cargo-fuzz** targets for the L1 lexer+parser, the M-210 checker, and the schema/manifest
  deserializers (would have caught H8 immediately).
- **cargo-llvm-cov** as a *map for mutation testing* — explicitly not a coverage-percentage gate
  (that would reward the vanity coverage the review warned against).
- Institutionalize the **mutant-witness** comment convention on every differential/negative test.
- **Restores:** durability of every WS1–WS6 guarantee; mechanizes the review's probe discipline.
- **Verify:** `just mutants` reports zero surviving mutants on the trusted base (or each survivor
  is triaged); fuzz targets run clean for a fixed budget in CI.

### WS9 — Editorial / mechanical-green · effort S · risk none

Findings: C1-06 (trailing newlines on 5 `M-*-verified.md`), C1-07 (codespell typos + ignore-list).
Trivial; do first so `just check` is green repo-wide and later PRs start from a clean baseline.

## 4. Sequencing (three waves)

**Wave 0 — clear the decks (days).** WS9 (editorial green) + the WS3 *decision* (RFC-0003
reconciliation direction) raised for ratification, since it blocks WS3's code. Land the
mutant-witness convention and one or two proptest conversions from WS8 so Wave 1 lands with
durable tests rather than retrofitted ones.

**Wave 1 — honesty integrity (the 1.0 critical path).** WS1 (numerics) → WS2 (contracts) → WS4
(L1 soundness) → WS5 (selection) → WS3 (VSA, after its decision) → WS6 (harness/LSP). These close
all 11 Highs and most Mediums. Order WS1/WS2 first (trusted base), WS4 next (only crash in the
tree), then WS5/WS3/WS6. Each is an independent PR; WS1, WS2, WS4, WS5 have no interdependencies
and can parallelize across contributors.

**Wave 2 — hardening + durability.** WS7 (security/CI, one PR) + the remainder of WS8
(cargo-mutants/fuzz/coverage). These can overlap Wave 1; WS7 is fully independent.

## 5. Proposed 1.0.0 readiness gate (recommend ratifying as an ADR)

No release gate exists; defining one is itself a decision the maintainer should ratify
append-only. Recommended criteria, partitioned by what is in our control now vs. external:

**Gate A — resolvable now (this roadmap):**

1. Zero open **High** findings (Waves 0–1 complete).
2. Every **Medium** resolved or explicitly deferred with a one-line ADR rationale.
3. The honesty surface is *durable*: cargo-mutants green on the trusted base, the LCG property
   tests migrated to proptest with seed rotation, fuzz targets in CI (WS8).
4. `just check` green repo-wide including gitleaks + `cargo deny` (WS7, WS9).
5. KC-4's *numeric* cert-overhead budget ratified (the review noted it is measured but the
   threshold is still a maintainer decision).

**Gate B — decision/external (track explicitly; may gate the surface, not the kernel):**

1. RFC-0003 §4 reconciled (WS3 decision) and RFC-0006/RFC-0007 moved `Draft → Accepted` — but
   their concrete-syntax ratification is **KC-2-gated**.
2. **KC-2 (LLM-leverage) verdict recorded.** Genuine external blocker: the M-002 harness is ready
   but running it needs LLM API access. Either obtain access and record the verdict, or make an
   explicit decision to ship 1.0.0 of the *kernel/core* with KC-2 still "not established" and the
   surface language gated to a later minor.

**Out of scope for 1.0.0 (Phase 3 / post-1.0, confirmed from the roadmap):** native libMLIR/LLVM
codegen, JIT, semantic projections (FR-C1), resonator factorization (FR-C2), BitNet packed-ternary
acceleration (FR-C3), native ternary-hardware path. These are maturation, not release blockers.

**Recommended framing:** ship a **1.0.0 of the kernel/core** once Gate A is met and Gate B's
RFC-0003 reconciliation lands, explicitly scoping the concrete surface language and the KC-2
verdict to a tracked follow-up (a `1.x` once the LLM probe runs). This lets the honest, verified
substrate — the actual product — reach 1.0.0 without being held hostage to an external API
dependency, while keeping the surface-language claims gated until KC-2 is real.

## 6. Cross-cutting verification discipline

Applied to every workstream, mirroring how the review itself was conducted:

- Each fix ships with a test that **fails without it** (reuse the exact probe that exposed the
  finding as the regression's mutant-witness; cite the finding ID in the test comment).
- Re-run the Stage-A probe set after each Wave-1 PR; the previously-MISSED mutants (A2-07, A3-08,
  A3-09, A4 reject corpus, A5-07, A6-03, A6-04) must flip to CAUGHT — that is the objective
  measure that the test suite now tests the right things.
- Honesty-tag changes go through the `docs-review`/`changelog` discipline; any guarantee
  downgrade is recorded, any upgrade requires a checked basis (VR-5).
- `just check` (via `scripts/checks/all.sh`) must be green locally before each commit; CI parity
  holds because both route through the same scripts.
