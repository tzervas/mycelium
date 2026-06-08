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
   Fix or explicitly skip; never hand off red. Add a property test for any new bound.
8. **Record it.** Update the `CHANGELOG.md` and any doc's changelog footer / status (use the
   `changelog` skill). 
9. **Commit & PR.** Conventional, imperative subject referencing the issue
   (`feat(swap): …`, `docs(rfc-0003): …`). The PR says which `FR/NFR/VR/SC` it advances and
   **how it was verified**; editorial-only PRs say so.

## Toolchain (CONTRIBUTING)
- **Rust** kernel + reference interpreter; **MSRV 1.92** pinned; `cargo fmt`, `cargo clippy
  -D warnings`, `cargo test`. The interpreter is the trusted base; MLIR→LLVM stays on the
  perf path. (Note any local rustc/Python newer than the pin is fine to build with, but don't
  silently bump the committed MSRV / version pins — that's a decision, not a build detail.)
- **Python 3.13/3.14** via **UV**; `pytest` + codecov; **ruff** + **Black** (PEP 8).
- Reuse the `balanced-ternary` crate; port `torchhd`'s op set as a VSA reference.

## Definition of done
Tests pass · `just check` green (or skips explained) · bounds carry tag+proof/property test ·
claims grounded · decisions append-only · changelog/status updated · conventional commit +
verified-how PR note. When in doubt, prefer the smaller, more auditable change.
