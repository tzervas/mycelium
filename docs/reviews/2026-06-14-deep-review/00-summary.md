# Mycelium Deep Review — Summary (2026-06)

Status: Advisory — report only. This review recommends; it does not gate and it changed no
code. Branch `claude/codebase-review-security-audit-n9l2vj`, HEAD `e2d627e`.

A four-stage pass over the whole repository: (A) code-correctness + test-quality, (B) security
audit, (C) quality/style against the project's own house rules, (D) a QC/PE improvement roadmap.
Depth was rubric-adaptive (`.claude/skills/_shared/review-rubric.md`): T2 on the fragile paths
(core IR, certificates, verified numerics, VSA bounds, selection/EXPLAIN, MLIR/packing, JSON
contracts, CI/scripts/supply-chain), T1 elsewhere.

## Verdict

**This is a strong, honesty-disciplined codebase, and its tests largely earn their coverage.**
The two highest-risk mechanisms — the guarantee-lattice meet (weakest-wins) and the single
shared M-210 translation-validation checker — are correct and survive mutation. The "no black
boxes / EXPLAIN" rule is genuinely honored: every automatic decision point is reified and
EXPLAIN-able, swaps are never silent, and the trusted base is small (5,250 LOC, passes KC-3).
`cargo fmt`, `clippy -D warnings`, and all 323 tests are clean; `cargo audit` is clean; there is
zero `unsafe`. Crucially for the user's framing, the test suite is **not** vanity coverage —
across ~25 transient mutation probes the great majority were **caught** by tests asserting real
behavior (independent oracles, hand-computed constants, content assertions, a genuinely
discriminating wrong-layout differential).

**The findings cluster in one theme: the honesty rule is upheld in spirit everywhere but leaks
at a few precise seams** — places where a `Proven`/`Exact` tag or a schema contract is asserted
more strongly than the code actually checks. No Critical issues; **11 distinct High issues**, all
either probe-confirmed or independently source-verified.

> **Grading note for triage.** Under a strict reading of the rubric, four of the Highs graze the
> *Critical* line: the numerics `ProvenThm`-on-unrounded-f64 (H1) and the VSA `Proven`-capacity-
> without-side-conditions (H6) are "`Proven` tag without checked side-conditions"; the numerics
> `uncertain()` non-finite→exact collapse (H3) and the `EmpiricalFit{trials:0}` acceptance (H11)
> are "silent" honesty-surface holes. The reviewers held them at High because each requires a
> misbehaving caller / tampered input or is bounded to ulp scale, but the final severity is the
> maintainer's call — flagged here rather than buried.

## High findings (11 distinct; cross-batch duplicates merged)

| # | Issue | Where | Evidence | Refs |
|---|---|---|---|---|
| H1 | `ProvenThm` emitted on ε/δ bounds composed in round-to-nearest f64 with no outward rounding — not a true upper bound at ulp scale | `mycelium-numerics` `error.rs:190,222`, `cert.rs:242` | source-verified | A2-01, C1-01 |
| H2 | Tier-i checker's absolute `CHECK_TOL=1e-12` is vacuous in the float-round-off regime ADR-010 assigns to this kernel (accepts ε=0 vs 5e-13) | `mycelium-numerics` `cert.rs:26,106,121` | reasoning, untested regime | A2-02 |
| H3 | `AffineForm::uncertain` silently collapses a non-finite radius to an exact constant (∞ uncertainty → claimed exact) | `mycelium-numerics` `error.rs:49-55` | source-verified | A2-03 |
| H4 | 5 `Permute` cells tagged `Exact` where RFC-0003 §4 table says `Proven` (code took the stronger reading of an internally-contradictory RFC) | `mycelium-vsa` `matrix.rs:37,41,49,53,57` | source + RFC verified | A3-01, C1-03 |
| H5 | HRR/FHRR `Bind` tagged `Exact`, grounded in a non-citable "issue #61" rather than the corpus | `mycelium-vsa` `hrr.rs:147`, `fhrr.rs:142` | source-verified | A3-02 |
| H6 | `Proven` capacity bound issued after checking only `dim ≥ requiredDim` — skips checkable bipolar-alphabet + distinct-item side-conditions the cited theorem assumes | `mycelium-vsa` `mapi.rs:121-163`, `capacity.rs:51-62` | reasoning + probe | A3-03, C1-02 |
| H7 | Totality checker classifies a non-terminating function as `Total` and admits it as `matured` (stale smallness across a shadowing rebind) | `mycelium-l1` `totality.rs:187-204` | probe-confirmed (diverges; accepted) | A4-01 |
| H8 | Recursive-descent parser/checker/elaborator have no depth guard → crafted nested input crashes `myc-check` (SIGABRT), bypassing the harness timeout | `mycelium-l1` `parse.rs:352,549,578` | probe-confirmed (≈5k → exit 134) | A4-02, B2-01 |
| H9 | Non-finite `f64` predicate literals canonicalize to JSON `null`, so two opposite selection policies share one `PolicyRef` — the audit anchor becomes non-injective | `mycelium-select` `lib.rs:321-346` | demonstrated (hash collision) | A5-01, B2-02 |
| H10 | KC-2 baseline DSL reads `Bin` as unsigned; kernel/spec use two's-complement — benchmark arms compute different answers (+178 vs −78), invisible to the shape-only oracle | `experiments/.../kc2/baseline.py:38-40,116-123` | probe-confirmed | A6-01 |
| H11 | Wire structs lack `deny_unknown_fields`, and `EmpiricalFit{trials:0}`/infinite `eps` are accepted — a tampered manifest can carry an evidence-free guarantee the schema would reject | `mycelium-core` `bound.rs:99-108`, `meta.rs`, `value.rs`, `recon.rs` | probe-confirmed | A6-02, B2-03, A1-02 |

Full per-batch detail, probe logs, and the Medium/Low/Nit findings are in `01-…`/`02-…`/`03-…`.
Counts: Critical 0 · High 11 · Medium ~16 · Low ~21 · Nit ~7 (some IDs are cross-batch
corroborations of the same defect).

## What is notably right

- **Test quality is real, not cosmetic.** Independent oracles (integer-oracle exhaustive codec/
  arithmetic checks; hand-computed numeric constants; reference-interpreter differentials; a
  wrong-layout differential proven to fail if the checker is no-opped). Mutation probes caught
  the meet direction, the shared checker, the VR-5 gate, the legal-pair side-condition, the
  EXPLAIN content, and the layout check.
- **No black boxes.** Every selection/swap/packing/cleanup decision is reified and EXPLAIN-able;
  no silent path found.
- **Small auditable kernel** (KC-3 passes at 5,250 LOC), **zero `unsafe`** (forbid effective),
  **BLAKE3 domain-separated content addressing** with no truncation.
- **Security posture is sound for the threat model** (local research tool): CI is
  `workflow_dispatch`-only with least privilege, scripts are `set -euo pipefail` + shellcheck-
  clean, no `curl|bash`, no command injection, `cargo audit` clean over 32 deps.

## Top recommendations (full roadmap in `04-…`)

1. **Outward-rounding + relative checker tolerance in `mycelium-numerics`** — closes H1, H2 (the
   most on-theme honesty holes; cheap).
2. **`deny_unknown_fields` + an emit-then-validate schema↔Rust contract test** — closes H11 and
   the enum-drift class.
3. **Recursion-depth guards + a deep-nesting reject fixture; fuzz the parser** — closes H8.
4. **cargo-mutants on the trusted base + proptest migration of the LCG property tests** — the
   direct mechanical answer to "coverage that tests the right things"; would have caught the
   probe-MISSED gaps (A2-07, A3-08/09, A4 reject corpus, A6-04).
5. **Append-only RFC-0003 erratum** reconciling the §4 table with its "Net" prose, then realign
   the code tags — closes H4/H5 honestly without rewriting a decision.

## Reproduction appendix

- **Repo state:** branch `claude/codebase-review-security-audit-n9l2vj`, HEAD `e2d627e`, clean
  working tree at review time.
- **Toolchain:** rustc/cargo **1.92.0** (matches `rust-toolchain.toml`); uv 0.8.17; `cargo audit`
  0.22.2. `just` was **absent** — checks were run via `bash scripts/checks/all.sh` and the
  individual `scripts/checks/*.sh`, which are the same entrypoints `just check`/`just ci`/CI use
  (so local↔CI parity holds).
- **Commands run (all read-only except reverted probes):** `cargo test --workspace
  --all-features`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo
  fmt --check`; `(cd experiments && uv run --frozen pytest)`; `bash scripts/checks/*.sh`; `cargo
  audit`; `shellcheck` (via `shell.sh`).
- **Skipped checks (environment, not regressions):** `proofs.sh` (z3/cabal/GHC absent — CI
  installs them); `secrets.sh` gitleaks (absent — ran narrow fallback). Two doc-content checks
  **failed on pre-existing committed content**, not this branch: `markdown.sh` (MD047 trailing
  newline on 5 `docs/verification/M-*-verified.md`) and `spell.sh` (codespell: two genuine
  hyphenation typos plus two domain false-positives — see C1-07 for the exact tokens and
  locations) — these are findings C1-06/C1-07.
- **Probe discipline:** every test-sensitivity claim marked "probe-confirmed" came from a
  transient single-file source mutation, run against the narrowest test, then reverted with
  `git checkout --`; the working tree was verified clean (`git diff --stat` empty for the scope)
  after every probe and at the end of every batch. Nothing was committed except this report.

## Next step

Per the agreed workflow, this is **report-first**. The natural follow-up is to pick a remediation
scope from the High list (the H1/H2 numerics fix and the H8 depth guard are the cheapest
high-value starts) and the `04-…` roadmap, then implement on this branch with the usual
honesty-tag / property-test / changelog discipline. No fixes have been applied.
