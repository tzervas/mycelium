---
name: dev-workflow
description: >-
  Mycelium's implementation discipline. Apply when writing or changing code/docs in
  this repo: small auditable steps, tests-first, a property test for every bound,
  honest per-op guarantee tags, no black boxes (EXPLAIN), append-only decisions,
  grounded claims, and local↔CI check parity (`just check`) before committing.
when_to_use: >-
  Use when implementing a task/issue, writing a new RFC/ADR, adding an operation or
  swap, or otherwise changing the corpus or (later) the code — i.e. any authoring work
  that should follow the house rules rather than just answering a question.
allowed-tools: Bash(just:*), Bash(git diff:*), Bash(git status:*), Read, Grep, Glob, Edit, Write
---

# dev-workflow

How we build Mycelium. This operationalizes `CONTRIBUTING.md`; follow it while authoring.

**Workspace prep + scoped PRs (DN-65).** *Before* authoring a unit: **sync off the latest tip**
(`git fetch`; branch from / ff to current `dev`/head — never a stale base) and **pre-install the
toolchain your change-kind needs** (Rust → `just setup`; Python → `uv sync --group <g>`; docs →
markdownlint + `doc_refs_check.py`; proofs → `z3`/LH/Lean) so nothing surprises you mid-flight. Do
the work at whatever scale it takes, but **land it as logical, closely-scoped PRs** (~1–2k-LOC soft
target — cohesion over a line count; a big effort lands as a fan/sequence of small, individually
`/pr-review`'d PRs). When PRs share files, land sequentially and pull the merged base down between
them. Full policy + the change-kind→toolchain map: `docs/notes/DN-65-…md`.

## The loop
1. **Anchor the work.** Find the issue/task (`M-xxx`/`E*`) and the `FR/NFR/VR/SC/KC` it
   advances, and the governing RFC/ADR. If none exists, the design is the first deliverable —
   write the ADR/RFC (or open the question) before the code.
2. **Smallest auditable step.** Keep the kernel small enough for one expert to audit (KC-3).
   Prefer composition over inheritance; SOLID/DRY/KISS/YAGNI/Demeter/SoC. No speculative
   generality.
3. **Tests first.** Write the failing test, then the code. For any **approximate operation**,
   ship its **bound + guarantee tag + a property test that exercises the bound** (SC-2). Round-
   trips/injectivity get a machine-checkable proof where claimed (e.g. swaps → `proof_ref`).
4. **Stay honest.** Tag every guarantee per-model/per-op on the lattice
   `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`. Use `Proven` only with a theorem whose
   side-conditions you *check*; otherwise `Empirical`/`Declared`. Downgrade freely to stay
   honest; never upgrade without a checked basis (VR-5).
5. **No black boxes.** Any selection/conversion/approximation is reified, inspectable, and
   `EXPLAIN`-able. A swap is never silent; out-of-range is an explicit `Option`/error.
6. **Ground every claim.** Normative statements cite `G1–G11 / A–E / R1–R8 / T0.x–T2.x`, or are
   marked open questions. Decisions are **append-only** — supersede, don't rewrite.
7. **Verify locally, with parity.** Run `just check` (the same suite CI runs) before you commit.
   Fix or explicitly skip; never hand off red. Add a property test for any new bound. **Before
   committing, the branch-guard asserts you are on your working branch, not a protected one**
   (`/branch-guard` or `just branch-guard`; also enforced by the pre-commit + PreToolUse hooks) —
   protected branches (`main`/`integration`/`dev`/`claude/head/*`) are PR-only, never a direct commit.
8. **Record it.** Update the `CHANGELOG.md` and any doc's changelog footer / status (use the
   `changelog` skill). If you changed a public API, run `just api-baseline` (updates
   `docs/spec/api/`) AND `just docs-index` (updates `docs/api-index/`). Commit both deltas
   as part of the change.
9. **Commit & PR.** Conventional, imperative subject referencing the issue
   (`feat(swap): …`, `docs(rfc-0003): …`). The PR says which `FR/NFR/VR/SC` it advances and
   **how it was verified**; editorial-only PRs say so.

## Toolchain (CONTRIBUTING)
- **Rust** kernel + reference interpreter; **MSRV 1.96.1** pinned (ADR-041); `cargo fmt`, `cargo clippy
  -D warnings`, `cargo test`. The interpreter is the trusted base; MLIR→LLVM stays on the
  perf path. (Note any local rustc/Python newer than the pin is fine to build with, but don't
  silently bump the committed MSRV / version pins — that's a decision, not a build detail.)
- **Python 3.13/3.14** via **UV**; `pytest` + codecov; **ruff** + **Black** (PEP 8).
- Reuse the `balanced-ternary` crate; port `torchhd`'s op set as a VSA reference.

## Banked guards (lessons from the 2026-06 deep review)

Concrete, mechanical rules so the honesty rule holds *while you write the code*, not just in
audit. Each is grounded in a real finding (ID in parens). Apply the matching one whenever you
touch that kind of code; reviewers grep for these too (`_shared/review-rubric.md` §1.5).

1. **Floating-point bounds round *outward*.** Any ε/δ — or any number that will carry a
   `Proven`/`Empirical` tag — composed in `f64` must be rounded **outward**, so the stored value
   is a *true* upper bound and not a round-to-nearest value that can fall below the real one. Use
   the directed-rounding helpers (`mycelium-numerics::round::{add_up,mul_up}`); they push up only
   when IEEE actually rounded down, so an exact composition stays exact. Re-validation checkers
   compare with a **relative** tolerance, never an absolute slack that goes vacuous for tiny
   bounds. (A2-01 / A2-02)
2. **Guarantee/bound constructors fail closed.** Non-finite or out-of-range inputs return
   `Option`/`Err` — never a silent collapse (e.g. infinite uncertainty must not become "exact").
   Prefer private fields + a validating constructor so an invalid bound is unrepresentable, and
   re-validate a *composed* result before emitting it (overflow to `inf` is a refusal, not a
   bound). (A2-03 / A2-04 / A2-05 / B2-03)
3. **A serde wire struct is a contract on *both* sides.** Every struct mirroring a
   `docs/spec/schemas/*.json` uses `#[serde(deny_unknown_fields)]` **and** re-checks every schema
   constraint on deserialize (e.g. `EmpiricalFit.trials ≥ 1`, non-empty citation). Ship an
   emit-then-validate test that pins one committed example per enum spelling / basis / layout — a
   round-trip test alone cannot catch tag drift. (A6-02 / A6-03 / B2-03)
4. **Recursive descent over untrusted input is depth-guarded.** Parsers, type/totality checkers,
   and elaborators that recurse on input carry an explicit depth budget and return a clean error
   past it — never lean on the host stack. Add a deep-nesting *reject* fixture. `myc-check` is the
   M-002 oracle: a degenerate input must exit-2, not `SIGABRT`. (A4-02 / B2-01)
5. **Content-addressed identity rejects ambiguous encodings.** Anything hashed for identity must
   reject inputs that canonicalize ambiguously (non-finite `f64` serialize to JSON `null`, so two
   different policies collide on one ref). Validate *before* hashing. (A5-01 / B2-02)
6. **Property-tracking analyses handle shadowing.** Any pass that carries a fact across binders
   (totality "smaller-than", scope, linearity) must drop-and-restore that fact on a *shadowing*
   rebind, or a non-terminating/ill-formed program slips the gate. (A4-01)
7. **Tests assert the *specific* failure and ship a mutant-witness.** Negative/reject tests assert
   the exact error variant (and span/message), not just "some `Err`". Every differential, negative,
   or bound test carries a one-line comment naming a mutation that makes it fail (reuse the probe
   that found the bug). Don't present a single fixed-seed sample as a statistical procedure — say
   "one deterministic sample of N", or rotate the seed in CI. (A4 reject corpus / A2-07 / A3-08-09
   / A6-04)

## Definition of done
Tests pass · `just check` green (or skips explained) · bounds carry tag+proof/property test ·
claims grounded · decisions append-only · changelog/status updated · conventional commit +
verified-how PR note · the **banked guards** above applied to any code of their kind. When in
doubt, prefer the smaller, more auditable change.
