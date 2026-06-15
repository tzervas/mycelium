# Stage D — QC / PE Improvement Roadmap

Status: Advisory — report only. Part of the 2026-06 deep review (see `00-summary.md`).

This stage answers the user's framing — *"100% coverage is useless if it's not testing the
right things in the right way"* — by recommending process and tooling that would have *caught
the findings in Stages A–C mechanically*, while preserving all intended behavior. Each item is
grounded in a specific finding and tagged with effort/value. Adopt/reject is the user's call;
the recommendation column is this review's opinion.

The headline lesson from Stage A is that the suite is genuinely strong — most mutation probes
were **caught** — but it has two systematic blind spots the probes exposed: (1) it validates the
*real-arithmetic* inequality and is blind to *floating-point* round-off gaps (A2-01/A2-07), and
(2) several negative/differential tests are weaker than they read (reject corpus asserts only
`is_err()` — A4; shape-only baseline oracle — A6-04; unpinned constants — A5-07; missed mutants
at A3-08/A3-09). The roadmap targets exactly those.

## Recommended — high value

1. **cargo-mutants on the trusted base** (`mycelium-core`, `mycelium-cert`, `mycelium-numerics`,
   `mycelium-vsa`). *Effort: medium; value: high.* This is the direct mechanical answer to the
   user's concern: it would have surfaced A2-07 (ulp-scale mul/add gaps the ≥1% probes missed),
   A3-08/A3-09 (the two MISSED VSA probes), A4's `is_err()`-only reject corpus, and A6-03/A6-04.
   Wire it as an opt-in `just mutants` recipe (not in `just check` — runtime is minutes-to-tens).
   Start scoped to `numerics` + `vsa/capacity` where the known blind spots live.
2. **proptest (or quickcheck) migration for the hand-rolled LCG property tests** in
   `mycelium-numerics` and `mycelium-vsa`. *Effort: medium; value: high.* The current tests are
   one fixed-seed deterministic sample (honestly labeled, but A2-07/A3 noted the lack of
   adversarial coverage — denormals, large magnitudes, signed-zero/NaN/inf). proptest adds
   shrinking, `PROPTEST_CASES` scaling, and CI seed-rotation (a different seed per run finds new
   inputs while staying reproducible from the failure seed). Resolves the "20k fixed sample ≠
   statistical procedure" tension without changing the bound math.
3. **Outward-rounding / interval discipline in `mycelium-numerics`.** *Effort: low-medium; value:
   high.* Directly fixes the central honesty hole (C1-01/A2-01) and the vacuous checker
   (A2-02). Either `f64::next_up` after each bound-increasing op, a small `(1 + k·EPSILON)`
   inflation, or an interval/`rug` backend for the checker re-derivation; pair with a relative
   (not absolute `1e-12`) checker tolerance. Add a small-ε regression test (the regime currently
   untested).
4. **cargo-fuzz / libFuzzer targets** for the untrusted-input surfaces: the L1 lexer+parser
   (would have found A4-02/B2-01 — the stack-overflow DoS — immediately), the M-210 cert
   checker, and the schema/manifest deserializers (A6-02/B2-03). *Effort: medium; value: high.*
   Pair with **explicit recursion-depth guards** in parser/checker/elaborator and a
   `reject/09-deep-nesting.myc` corpus fixture so the guard is regression-tested.
5. **schema↔Rust contract test (emit-then-validate + `deny_unknown_fields`).** *Effort: low;
   value: high.* Add `#[serde(deny_unknown_fields)]` to the wire structs and a test that emits
   one example per bound kind / basis / layout and validates it against the JSON schema (and
   round-trips schema examples through Rust). Closes A6-02, A6-03, A6-06, B2-03 and the
   enum-spelling-drift class as a whole, and makes the schema an actual contract.

## Recommended — medium value

1. **"Mutant-witness" convention for differential and negative tests.** *Effort: low (process);
   value: medium.* Require every differential/negative test to carry a one-line comment naming a
   mutation that makes it fail (the reviewers produced these by hand; institutionalize it). This
   is the lightweight, always-on complement to cargo-mutants and directly attacks the
   "coverage ≠ testing the right thing" risk. Apply first to the reject corpus (A4): switch
   `is_err()` to per-file expected-error-substring assertions.
2. **`cargo deny` + `cargo audit` in `scripts/checks/` (skip-if-missing pattern).** *Effort: low;
   value: medium.* `cargo audit` already runs clean (0/32) but is not wired into `just check`;
   `cargo deny` (licenses, banned/duplicate deps, source allowlist) is absent (B1 gap). Add both
   as graceful-skip check scripts, matching the existing tooling idiom.
3. **Actually run gitleaks in CI + locally.** *Effort: low; value: medium.* `secrets.sh` degrades
   to a narrow git-grep fallback because gitleaks is never installed (B1 gap, C1-09) — so the
   "full secret scan" runs in neither environment. Add gitleaks to `install-tools.sh` and the CI
   tool step.
4. **Pin GitHub Actions to commit SHAs + add a scoped dependabot config.** *Effort: low; value:
   medium.* Closes B1-01/B1-04. Dependabot raises PRs only (it does not auto-run the advisory
   workflow), so it does not violate the manual-CI policy — but record the choice explicitly.
   Pin `npx markdownlint-cli2` to a version (B1-02) at the same time.

## Code enumeration / call-and-dependency mapping (developer-requested workflow tool)

*Effort: low–medium; value: medium.* A workflow aid to enumerate/trace/map functionality, calls,
paths, and routing. Two layers, both skip-if-missing per the existing tooling idiom:

- **Advisory `just map` recipe** (artifacts under `target/map/`, *not* part of `just check`):
  `cargo depgraph --workspace-only | dot` for the crate-to-crate routing graph (`cargo tree`
  fallback when graphviz is absent); `cargo modules` for per-crate module/item structure with
  internal `use` edges; `cargo doc --workspace --document-private-items` for the browsable item
  graph including internals. **Caveat:** Rust *function-level* static call graphs are partial (trait
  dispatch / generics) — use rust-analyzer's call hierarchy interactively for that, and
  `cargo-call-stack` for the interp/AOT hot paths when needed.
- **API-surface gate** (`scripts/checks/api.sh`, *in* `just check`): commit a `cargo public-api`
  snapshot per crate under `docs/spec/api/<crate>.txt` and diff against it. Doubles as a map ("what
  is callable from outside each crate") and a guardrail that catches accidental `pub` growth —
  directly supporting **KC-3** (small auditable kernel) and the deferred **A2-05** (private fields).
  Pin the snapshots so an intentional surface change is a reviewed diff.
- **Mycelium-lang (forward, Phase 3):** the design already mandates the substrate — SC-4
  dumpable/diffable lowering, content-addressed IR (an operation-hash graph is derivable from the
  `mycelium-lsp` lowering dumps *today*), and EXPLAIN traces. Make a program's call/dataflow graph a
  first-class **reified LSP/EXPLAIN artifact** when the toolchain matures ("no black boxes"), not a
  bolted-on external tracer; a minimal "IR-dump → dot" `just map` sub-target is feasible now against
  the existing dumps.

## Recommended — lower value / hygiene

1. **`overflow-checks = true` for the trusted kernel crates' release profile** (B2-05) — aligns
   release arithmetic with the never-silent ethos; no behavior change today.
2. **Private fields + accessors on the numerics kernel types** (A2-05) — make "never silent"
   structural rather than conventional in the trusted base.
3. **cargo-llvm-cov coverage**, framed explicitly as a **map for mutation testing**, not a
   target. *Effort: low.* CONTRIBUTING already names codecov; a coverage map tells cargo-mutants
   where to look and exposes the few untested error paths (A1-03, A4-04, A2-08). Do **not** set a
   coverage percentage gate — that would reward exactly the vanity coverage the user warned
   against.
4. **An `expect_value` oracle for the KC-2 baseline harness** (A6-01/A6-04) and, optionally,
   **hypothesis** for the Python harness property tests. Resolves the shape-only-oracle blind
   spot and the sign-convention divergence in one move; pin the Python dev-dep ranges while
   there.
5. **Append-only RFC-0003 erratum reconciling the §4 table with its "Net" prose** (A3-01/A3-02/
   C1-03), and **a basis/qualifier for on-expectation `Proven`** (A3-06/C1-04). These are
   decision-record actions, not code — route through the `changelog`/`docs-review` discipline.

## Explicitly considered and deferred

- **SBOM generation / `cargo vet`** — premature for a design-phase repo with a clean `cargo
  audit` and a tiny dependency set; revisit when third-party deps grow or a release is cut.
- **gitleaks in a pre-push hook** — fine once gitleaks is actually installed (item 8); not worth
  a separate workstream before that.
- **A coverage-percentage CI gate** — rejected outright (see item 12); it optimizes the wrong
  metric.

## Suggested sequencing

The cheapest high-value fixes close the two most on-theme honesty holes — **outward rounding +
relative checker tolerance** (high-value #3) and the **schema↔Rust contract test +
`deny_unknown_fields`** (high-value #5) — do them first. The **parser depth guards + fuzzing**
(high-value #4) are a small change that removes the only crash in the codebase. The
**cargo-mutants + proptest** investment (high-value #1 and #2, plus the mutant-witness
convention, medium-value #1) pays back across every future change and most directly answers the
user's framing. The supply-chain items (medium-value #2-4: `cargo deny`/`audit`, gitleaks, action
SHA-pinning + dependabot) are a single short "CI/supply-chain hardening" PR. Everything in the
lower-value/hygiene group can ride along opportunistically.
