# Kickoff `srf` — remaining L1 surface + runtime vocabulary (`mycelium-l1`)

> Continues the `lex` chain (M-663 landed). Read `.claude/agent-context.md` + `CLAUDE.md` (house rules
> win) + `.claude/kickoffs/README.md` (tiered `dev → integration → main`) first. **Serial-on-L1** —
> shares the `crates/mycelium-l1` collision files with `run` (mitigation #7), so do **not** run `srf`
> concurrently with `run`. Neither blocks the other; run `run` first (it's the critical-path unblock),
> then `srf` in the same serial L1 session.

## ⚡ RESUME HERE

**Branch off `dev`.** Serial-on-L1 Sonnet swarm (Opus orchestrator + an Opus leaf per task, landed one
at a time). Promote `dev → integration → main`. **Copilot review round on every kernel PR (it has
caught a real bug each time) — soundness + never-silent G2.**

**▶ FIRST: M-664 (#323) — `consume` / `grow` / `impl` surface keywords** (DN-03 §1 ratified terms,
none yet lexed). Depends on the **M-659 trait checker** (landed; M-659 itself stays `in-progress`
until `run`/M-673 lands its dictionaries — that does **not** block M-664's surface work).
- reserve `consume` + `grow` in `keyword()` (`token.rs`) — never-silent reject-corpus entries;
- `consume <substrate>` → affine move of a `Substrate` value (never copyable; LR-8);
- `grow <Trait> for <Type> { … }` → the derive-like generated impl;
- `impl <Type> { fn … }` inherent-method block parse + check;
- **also fix the stale `.claude/memory/lang-lexicon-syntax.md` legend (~l.100)** that still lists
  `impl` as reserved-not-lexed.
- **Acceptance:** the three are reserved keywords (reject-corpus added); `consume` elaborates to an
  affine move; `grow Debug for MyType {…}` generates the impl; `impl MyType { fn to_bits(self) -> … }`
  type-checks.

## Chain (dependency-ordered)
| # | Issue(s) | What | Status |
|---|---|---|---|
| 1 | **M-664** (#323) | `consume`/`grow`/`impl` surface keywords (+ lexicon-legend fix) | **active — ▶ first** |
| 2 | **M-667** (#327) | E7-2 R1 remaining: `fuse` (semilattice merge — RT6), `reclaim` (supervision reclamation — RT7/M-356), `tier` (RFC-0004 `ExecutionMode` switch). After M-665/M-666 (`hypha`/`colony`, landed). Each: L1 surface + check pass + elaboration. | next |
| 3 | **M-668** (#328) | E7-2 R2 **planning** (docs): decompose `xloc`/`mesh`/`cyst`/`graft`/`forage`/`backbone` into per-construct implementation RFCs with honest guarantee tags + gates (a design note / DN-11 append). Gated on R1 (M-667). | last |

For M-667: a non-associative `fuse` merge is a `CheckError` (declared semilattice constraint);
`reclaim` elaborates to `Supervisor` calls (M-356); `tier` switches mode with an EXPLAIN-able record;
all three active in the grammar; `just check` green. RFC-0008 §4.6 R1 enactment note updated
(append-only, "implemented Rust-first, pending ratification").

## Ownership / method
- **Owns:** `crates/mycelium-l1/**`, `.claude/memory/lang-lexicon-syntax.md`, the implemented RFC/DN
  append-only notes. **Read-only / FLAG up:** `tools/github/issues.yaml`, `CHANGELOG.md`,
  `docs/Doc-Index.md`, `docs/api-index/`, workspace `Cargo.toml`.
- Per-task loop: **design-map → FLAG architecturally-significant choices (flag-don't-guess) → Opus
  leaf → honesty + soundness review → Copilot round → land.** Honest guarantee tags; a property/
  soundness test per bound; never-silent `Result`/`Option`.
- **Done** = M-664 + M-667 + M-668 landed on `main`; the DN-03/RFC-0008 reserved-word + R1 enactment
  cross-refs updated; every issue body + status updated.
