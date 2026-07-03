# ADR-041 — MSRV Bump: Rust 1.92 → 1.96.1

| Field | Value |
|---|---|
| **ADR** | 041 |
| **Status** | **Accepted** (2026-07-03 — maintainer-authorized). Amends the **MSRV clause of ADR-007** (Foundation charter §ADR-007: *"the reference-semantics interpreter in Rust (MSRV 1.92)"*): the committed minimum supported Rust version moves **1.92 → 1.96.1**. ADR-007's decision (kernel in Rust; MLIR→LLVM for AOT; Python for tooling) is otherwise **unchanged** — this amends only the pinned version number, not the toolchain strategy. `Accepted → Enacted` once the workspace builds + tests green on 1.96.1 and the pins below are landed. |
| **Decides** | The workspace MSRV / pinned toolchain is **Rust 1.96.1** (`rustc 1.96.1`, released 2026-06-26). Every committed pin moves in lockstep: `rust-toolchain.toml` `channel`, `Cargo.toml` `[workspace.package].rust-version`, and the prose MSRV statements in `CLAUDE.md` / `CONTRIBUTING.md`. |
| **Amends** | **ADR-007** (Foundation charter) — its MSRV value only. ADR-007's original text is **preserved, not rewritten** (house rule #3 — a pin change is a decision, recorded via a superseding/amending ADR, never an in-place edit of the Accepted charter); the charter carries only an append-only "MSRV superseded by ADR-041 (1.96.1)" pointer. |
| **Grounds** | Maintainer directive (2026-07-03) to run to the latest stable toolchain where non-breaking, alongside a workspace dependency-freshness pass; **CLAUDE.md / CONTRIBUTING** ("don't silently bump committed version pins — MSRV is a decision, not a build detail") — this ADR is exactly that decision, made explicitly rather than silently; the pin's own guard-comment in `rust-toolchain.toml` ("Do NOT bump without an ADR"). 1.96.1 verified available + installable (`rustup toolchain install 1.96.1` → `rustc 1.96.1 (31fca3adb 2026-06-26)`). |
| **Date** | 2026-07-03 |

> **Posture (transparency rule / VR-5).** This ADR records a *decision*, maintainer-authorized. It
> does not assert the build is green — that is a checked criterion in the Definition of Done below,
> discharged by a full `cargo build`/`cargo test` on 1.96.1 before the pins land. No guarantee tag is
> upgraded; the interpreter remains the reference/trusted base (ADR-007 / KC-3), unchanged by the
> toolchain version.

---

## 1. Why this bump

ADR-007 pinned MSRV at 1.92 (the then-current stable) to keep the trusted base on a single, committed
toolchain. Seven months on (2026-07-03), the maintainer directs a toolchain + dependency freshness
pass: move to a recent stable (1.96.1, released 2026-06-26) so the workspace builds against current
compiler diagnostics, current `std`, and dependency versions that themselves increasingly require a
newer rustc. Nothing in the kernel design depends on staying at 1.92; the pin is a hygiene choice, and
keeping it current reduces the future bump debt.

The bump is a **decision, not a build detail** (CLAUDE.md / CONTRIBUTING) — hence this ADR rather than
an unremarked edit to `rust-toolchain.toml`. The strategy of ADR-007 (Rust kernel + reference
interpreter as trusted base; MLIR→LLVM confined to the AOT perf path; Python for tooling/experiments)
is untouched.

## 2. Scope

**In scope:** the four committed MSRV pins —
`rust-toolchain.toml` (`channel = "1.96.1"`), `Cargo.toml` (`rust-version = "1.96.1"` + the two MSRV
comments), `CLAUDE.md` (§Toolchain), `CONTRIBUTING.md` (§Toolchain). Every workspace crate inherits via
`rust-version.workspace = true`, so no per-crate edit is needed.

**Out of scope:** the AOT/MLIR toolchain versions (LLVM/MLIR pins — their own decision); the Python
(3.13/3.14) pin (unchanged); the dependency-version refresh that rides alongside this bump is
**mechanical** (`cargo update` to latest semver-compatible + two pre-1.0 tooling-dep bumps verified
non-breaking) and does not itself need an ADR — only the MSRV pin does.

## 3. Consequences

- Contributors must have Rust **≥ 1.96.1**; `rust-toolchain.toml` makes `rustup` fetch it automatically
  on first `cargo` invocation (never-silent — the pin is explicit).
- CI jobs that "Set up Rust (pinned to MSRV via rust-toolchain.toml)" pick up 1.96.1 with no workflow
  edit (they read the file).
- Future MSRV moves follow the same path: a superseding/amending ADR + a lockstep pin update. This ADR
  is itself the template.

## 4. Definition of Done

- All four pins read **1.96.1** (grep-verifiable: no committed `1.92` MSRV reference remains outside
  append-only history / this ADR's own citation of the prior value).
- `cargo build --workspace` and `cargo test --workspace` (or `just check`) are **green on 1.96.1** —
  the checked basis for `Accepted → Enacted`. Any new-toolchain warning/deprecation is either fixed or
  explicitly recorded (never-silent, G2).
- The Foundation charter (§ADR-007) carries an append-only pointer to this ADR; the ADR index
  (`docs/adr/README.md`) lists ADR-041; `CHANGELOG.md` records the bump.

## Meta — changelog

- **2026-07-03 — Accepted (maintainer-authorized).** MSRV 1.92 → 1.96.1; amends ADR-007's pin clause
  only (append-only — charter text preserved). Pins updated in lockstep across `rust-toolchain.toml`,
  `Cargo.toml`, `CLAUDE.md`, `CONTRIBUTING.md`. `Accepted → Enacted` on a green `cargo build`/`test`
  at 1.96.1 (DoD §4). Rides alongside a mechanical workspace dependency-freshness pass (out of scope
  here — no separate ADR). (VR-5 / house rule #3.)
