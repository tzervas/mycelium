# Stage C — Quality & Style

Status: Advisory — report only. Part of the 2026-06 deep review (see `00-summary.md`).
Baseline: HEAD `e2d627e`. Reviewed against the project's own house rules (CONTRIBUTING.md,
CLAUDE.md): the honesty rule + guarantee lattice, "no black boxes"/EXPLAIN, append-only
decisions, KC-3 small auditable kernel, SOLID/DRY/KISS/YAGNI/Law-of-Demeter/SoC, composition
over inheritance, notation `⊐` (not `⊃`).

**Verdict.** The Rust workspace is in unusually good shape for a design-phase repo: `cargo fmt`
and `cargo clippy --workspace --all-targets --all-features -D warnings` are both clean, tests
pass, and the "no black boxes" rule is honored remarkably well — every automatic decision point
(selection, swap, packing, layout, VSA cleanup) is reified and EXPLAIN-able, with refusals as
explicit `Result`/`Option` errors rather than silent coercions. The trusted base is small
(~5,250 LOC across the three checker crates), well within "one expert can audit it." The real
findings are honesty-tag overclaims concentrated in two trusted crates (numerics `ProvenThm` on
unrounded f64; VSA `Proven` capacity without modeling side-conditions). Mechanically, two
doc-content checks (`spell`, `markdown`) fail on **pre-existing committed content**, not on
anything this branch introduced.

## Mechanical results

| Check | Result |
|---|---|
| `cargo fmt --check` | clean (exit 0) |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | clean (exit 0, no warnings) |
| `scripts/checks/format.sh` (ruff format + cargo fmt) | clean (9 py files) |
| `scripts/checks/lint.sh` (clippy + ruff check) | clean |
| `scripts/checks/shell.sh` (shellcheck) | clean (16 scripts) |
| `scripts/checks/links.sh` | clean (relative links resolve) |
| `scripts/checks/structured.sh` (json/yaml/toml) | clean |
| `scripts/checks/schema.sh`, `grammar.sh` | clean |
| `cargo test --workspace` | clean (all pass) |
| `scripts/checks/spell.sh` (codespell) | **FAIL** — pre-existing content (see C1-07) |
| `scripts/checks/markdown.sh` (markdownlint) | **FAIL** — pre-existing content (see C1-06) |

**Skips (local↔CI parity candidates):** `proofs.sh` skips both proofs (z3/cabal/GHC absent
locally; CI installs them); `secrets.sh` skips gitleaks and runs the narrow fallback (gitleaks
absent locally — and, per C1-09, in CI too). These are intended graceful-degradation; CI's
install steps cover the proof tools, so they are not parity *gaps* except for gitleaks.

## Findings (honesty-tag overclaims first)

- **[High] C1-01** `crates/mycelium-numerics/src/error.rs:106-115,142-158,188-225` (basis
  re-emitted at `cert.rs:239-251`) — `AffineForm`/`ErrorBound` `add`/`scale`/`mul` compose the
  radius/ε in plain `f64` **without outward rounding**, yet the doc comments assert "Sound: the
  composed `eps` is a true upper bound" and `composed_basis` re-stamps `ProvenThm`. The bound on
  the *mathematical* affine form is sound, but the *computed* value can fall below the true
  radius by the summation's own rounding error, so `Proven` is not fully backed for the FP
  realization. → Honesty rule (`Proven` only with checked side-conditions; the composition's
  rounding is an unchecked side-condition). → Round outward (`next_up`) or inflate before tagging
  `Proven`, or downgrade the composed basis. Contrast: `mycelium-cert/src/dense.rs` and
  `mycelium-dense/src/lib.rs` bound a *single* rounding and check every per-element
  side-condition — the correct pattern this crate should mirror. (= A2-01.)
- **[High] C1-02** `crates/mycelium-vsa/src/capacity.rs:51-62` — `proven_capacity_bound` emits
  `ProvenThm` whenever `dim ≥ required_dim(items, delta, μ=0.1)`. The cited theorem (Clarkson
  Thm 6) has further side-conditions the function cannot see (it takes only scalars): it never
  verifies the codebook is bipolar/random with the assumed margin μ=0.1 (hardcoded `MARGIN_MU`,
  honestly labelled "illustrative") nor that the bundled items are distinct. Only the
  *arithmetic* instantiation is checked. → `Proven` without all side-conditions checked. →
  Accept μ/bipolarity/distinctness as checked inputs (require a verified μ, assert distinctness)
  or downgrade. Note: bipolarity *is* checked at the `cert/dense_vsa.rs:162` swap call-site —
  meaning the honesty invariant is split across caller and callee (an SRP/Demeter smell: the
  kernel function that stamps `Proven` is not the one that guarantees the precondition). (= A3-03.)
- **[Medium] C1-03** `crates/mycelium-vsa/src/matrix.rs:36-57` vs `docs/rfcs/RFC-0003-VSA-
  Submodule-Boundary.md:24-32` — the code tags `Permute` as `Exact` for MAP-I/MAP-B/HRR/FHRR/SBC,
  but the §4 **table** (column 4, rows 26-30) lists those cells as `Proven` (only BSC is `Exact`
  there). The matrix.rs doc and the RFC's "Net" paragraph (line 32) side with `Exact` — so the
  RFC's own table and prose contradict each other, and the code resolves it to the *stronger*
  tag. → Code tagging stronger than the normative table is honesty/grounding drift. → Reconcile
  the RFC table with its Net paragraph (append-only supersede/erratum), then cite the reconciled
  cell. (= A3-01/A3-02, permute-row mechanism.)
- **[Medium] C1-04** `crates/mycelium-vsa/src/matrix.rs:44,bsc.rs:9-11` — the matrix tags BSC
  `Bundle` as `Proven`, but the doc qualifies it "`Proven` **on expectation** … weaker than w.p.
  ≥ 1−δ." The `Proven` variant therefore denotes two materially different strengths with no
  distinction, undercutting VR-5's "honest per-op" intent. (Mitigant: BSC's *value* path issues
  `Empirical`, so no value is mis-stamped.) → A distinct annotation for on-expectation
  guarantees, or a matrix-entry note. (= A3-06.)
- **[Low] C1-05** `crates/mycelium-core/src/ternary.rs:35` — `unreachable!("balanced-ternary
  digit out of range")` guards a dependency invariant inside the trusted kernel; if the dep ever
  returns an out-of-range digit, the kernel panics. → A `debug_assert!` + saturating/Result path
  is more in keeping with KC-3's "never silent, never crash"; at minimum, comment *why* it is
  truly unreachable.
- **[Low] C1-06** `docs/verification/M-230..M-260-verified.md` (5 files) — lack a trailing
  newline (markdownlint MD047). Pre-existing; editorial.
- **[Low] C1-07** codespell findings, all in pre-existing committed content. Two are genuine
  hyphenation typos (the correct forms are `reusing`/`reuses`) at
  `crates/mycelium-cert/Cargo.toml:20`, `crates/mycelium-cert/src/dense_vsa.rs:12`, and
  `docs/planning/phase-2.md:501`. Two are false positives — a programming-language proper noun
  and a type-qualifier keyword that codespell misreads as English typos — at
  `research/03-language-layer-RECORD.md:112,138,150`. → Fix the genuine ones; add the two
  false-positive tokens to `.codespellrc` `ignore-words-list` so the check goes green. Editorial.
- **[Nit] C1-08** `scripts/checks/format.sh:25-26`, CI — CONTRIBUTING.md and CLAUDE.md mandate
  **Black** for Python, but the toolchain uses **`ruff format`** exclusively (Black is referenced
  nowhere in `justfile`/`scripts/`/CI). → Doc/tooling drift: update the docs to "ruff format
  (Black-compatible)", or record the toolchain choice as an ADR per CLAUDE.md ("don't silently
  bump committed pins — that's a decision").
- **[Nit] C1-09** CI — `checks.yml` installs z3/GHC/cabal for proofs and runs
  `install-tools.sh`, but does not explicitly install `gitleaks`, so `secrets.sh` runs only its
  narrow fallback in CI too — the "full secrets scan" never actually runs in either environment.
  Advisory parity observation. (Ties to B1's gitleaks gap.)

## KC-3 measurement

Trusted-base (checker-path) source LOC, `wc -l` over `src` (excludes tests):

| Crate | src LOC |
|---|---|
| `mycelium-core` | 3,260 |
| `mycelium-cert` | 1,346 |
| `mycelium-numerics` | 644 |
| **Combined trusted base** | **5,250** |
| Total workspace Rust (incl. tests, 10 crates + xtask) | 20,906 |

**Assessment:** Passes KC-3 comfortably. 5,250 LOC across 3 crates is auditable by a single
domain expert in a sitting. `mycelium-core`'s public surface is 96 `pub fn/struct/enum` across 12
modules (largest `meta.rs`/`lower.rs` at 16 each) — no god-module, no over-exposure. The kernel
contains no perf hacks or heuristics; the one heuristic-shaped thing (the selection cost model)
lives *outside* the kernel in `mycelium-select` by deliberate design (its module doc cites the
KC-3/SoC sequencing). The two honesty findings (C1-01, C1-02) are *correctness* of the kernel's
tagging, not size — the kernel is small *and* the leaks are subtle, exactly the regime KC-3 is
meant to keep auditable.

## Honesty-tag spot checks (18 sites; ≥15 required)

| Site (file:line) | Claimed | Enforced in code? | Verdict |
|---|---|---|---|
| `core/guarantee.rs:61` `meet` | weakest-wins, laws by exhaustion | yes — exhaustive 4×4×4 tests | consistent |
| `core/guarantee.rs:77` `propagate` | result ≤ weakest input (VR-5) | yes — fold of `meet` | consistent |
| `cert/dense.rs:99-122` bf16 swap | `Proven`, side-conds checked/elem | yes — finite/exact-f32/normal/overflow checked | consistent (model pattern) |
| `dense/lib.rs:34,45` F32 op | `Proven`, single-rounding 2⁻²⁴ | yes — per-element grid/finite checks | consistent |
| `dense/lib.rs:39,50` bf16 op | `Proven`, two-rounding 2⁻⁸+2⁻²³ | yes — both roundings checked/elem | consistent |
| `numerics/error.rs:188` `ErrorBound::add` | "Sound … true upper bound" | partial — f64 sum, no outward rounding | **overclaim (C1-01)** |
| `numerics/error.rs:220` `ErrorBound::mul` | "sound first-order" | partial — f64 arithmetic unrounded | **overclaim (C1-01)** |
| `numerics/cert.rs:239` `composed_basis` | re-emits `ProvenThm` affine | partial — inherits C1-01 gap | **overclaim (C1-01)** |
| `numerics/cert.rs:215` `basis_strength` | basis *is* evidence class | yes — pure match | consistent |
| `vsa/capacity.rs:51` `proven_capacity_bound` | `Proven` iff dim sufficient | partial — only arithmetic side-cond | **overclaim (C1-02)** |
| `vsa/matrix.rs:37` MAP-I Permute `Exact` | matches RFC §4 | code stronger than RFC table cell | **drift (C1-03)** |
| `vsa/matrix.rs:44` BSC Bundle `Proven` | on-expectation | tag overloaded; value-path `Empirical` | **caveat (C1-04)** |
| `vsa/bsc.rs:19` value bundle `Empirical` | no Proven value issued | yes — `bundle_values_empirical` | consistent |
| `vsa/mapb.rs:16-21` nesting forbidden | depth>1 refused under Proven | yes — `NestedBundleUnsupported` | consistent |
| `vsa/fhrr.rs:9,98-107` unbind `Empirical` | weak link, never upgraded | yes — profile-bounded refusal | consistent |
| `cert/dense_vsa.rs:84-113` δ bound | `ProvenThm` iff capacity else `EmpiricalFit` else refuse | yes — bipolar checked at `:162`; Proven inherits C1-02 gap | mostly consistent |
| `interp/prims.rs:103-115` builtins | intrinsic `Exact`, result = meet | yes — `propagate(Exact, inputs)` | consistent |
| `interp/swap.rs:30-33` identity swap | cross-paradigm refused, never silent | yes — `UnsupportedSwap` | consistent |

Honesty findings concentrate in numerics composition and VSA capacity; the dense/cert swap
crates are the model of how to do it right.

## Clean areas

fmt, clippy (`-D warnings`), shell, links, structured-data, schema, grammar, and tests are all
clean; notation is correct (`⊐` used, zero `⊃`/`⊂` misuse in code or code-adjacent docs); no
train-wreck Demeter chains (the 4-link hits are idiomatic iterator adapters); no god-modules and
`mycelium-core`'s 96-item public surface is well-scoped; the **"no black boxes / EXPLAIN" rule
is fully honored** — `mycelium-select` returns an `Explanation` on every selection call with no
silent path, swap refuses cross-paradigm conversions explicitly, `lower.rs` uses a fixed
enumerable default schedule (no hidden choice), VSA cleanup thresholds come from the manifest
with below-threshold results surfaced as explicit errors, and a static honesty linter
(`mycelium-lsp/src/lint.rs`) flags `Declared` values and stub policies.
