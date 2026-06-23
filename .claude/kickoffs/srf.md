# Kickoff `srf` — remaining L1 surface + runtime vocabulary (`mycelium-l1`)

> Continues the `run` chain (M-673 landed). Read `.claude/agent-context.md` + `CLAUDE.md` (house rules
> win) + `.claude/kickoffs/README.md` (tiered `dev → integration → main`) first. **Serial-on-L1** —
> shares `crates/mycelium-l1` collision files (mitigation #7); **`run` landed first** (M-673 ✓); `srf`
> now branches from the **post-M-673** L1 base.

## ⚡ M-673 run-collisions — rebase notes (MUST read before touching L1)

M-673 changed the L1 base in three additive but compile-breaking ways. Every leaf that touches
`crates/mycelium-l1/` must account for these:

1. **`crates/mycelium-l1/src/mono.rs` added** (new public module). `lib.rs` now has
   `pub mod mono;`. Any branch missing this file will fail to compile.
2. **`elaborate`/`elaborate_colony` now call `crate::mono::monomorphize(env, entry)?` before
   `elab_prelude`** (`elab.rs`). New constructs' L0 lowering flows through the mono pass — do
   **not** insert L0 lowering that bypasses it.
3. **`checkty.rs` — additive `Env.impls` field**: `Env` gained
   `impls: BTreeMap<(String,String), Vec<FnDecl>>`. Every `Env { … }` literal — including in tests —
   **must include `impls: BTreeMap::new()`** (or a populated map) or it will not compile. The field
   is additive; no existing semantics changed.
4. **`Ty::Data` now carries type args** (`Ty::Data(String, Vec<Ty>)`); `FnSig.params` is
   `Vec<TypeParam>`. Pattern-match arms on `Ty::Data` must destructure both fields.
5. **`Residual` sites in `elab.rs` are kept as defensive invariants** — do **not** delete them
   (they are the never-silent fallback for staging; their presence is correct).

**Rebase discipline:** branch M-664 (and M-667 after it) from the post-M-673 `dev` tip (the tip
that carries M-673). Run `cargo test -p mycelium-l1` after every rebase to verify clean.

## ⚡ RESUME HERE

**Branch off `dev`** (post-M-673 tip). **Sonnet swarm — serial-on-L1** (one leaf per L1-touching
task, landed one at a time in dependency order). Promote `dev → integration → main`. **Copilot review
round on every kernel PR (it has caught a real bug each time) — soundness + never-silent G2.**

Swarm layout:
- **M-664 + M-667** — serial Sonnet leaves (each a tightly-scoped changeset on the shared L1
  surface files: `lexer.rs` / `token.rs` / `parse.rs` / `ast.rs` / `checkty.rs` / `elab.rs`).
  M-664 lands first; M-667 rebases onto the post-M-664 L1, then lands.
- **M-668** — a parallel Sonnet leaf (docs-only: design notes under `docs/notes/`; fully disjoint
  from the L1 code; can run concurrently with M-664/M-667 without collision).
- The **orchestrator** owns all shared-wiring reconciliation: `CHANGELOG.md`, `docs/Doc-Index.md`,
  `tools/github/issues.yaml`, `docs/api-index/` (regenerate with `cargo public-api` if the public
  surface changes). Leaves treat these **read-only** and FLAG up if they need a change there.

**▶ FIRST: M-664 (#323) — `consume` / `grow` / `impl` surface keywords** (DN-03 §1 ratified terms,
none yet lexed). Depends on the **M-659 trait checker** (landed) and the **M-673 mono base** (landed).
- reserve `consume` + `grow` in `keyword()` (`token.rs`) — never-silent reject-corpus entries;
- `consume <substrate>` → affine move of a `Substrate` value (never copyable; LR-8);
- `grow <Trait> for <Type> { … }` → the derive-like generated impl;
- `impl <Type> { fn … }` inherent-method block parse + check;
- **also fix the stale `.claude/memory/lang-lexicon-syntax.md` legend (~l.100)** that still lists
  `impl` as reserved-not-lexed.
- New `Env { … }` literals in tests **must include `impls: BTreeMap::new()`** (M-673 collision).
- **Acceptance:** the three are reserved keywords (reject-corpus added); `consume` elaborates to an
  affine move; `grow Debug for MyType {…}` generates the impl; `impl MyType { fn to_bits(self) -> … }`
  type-checks; `cargo fmt` / `clippy -D warnings` / `test -p mycelium-l1` green; api snapshot
  regenerated (`cargo public-api`) if public surface changes.

## Chain (dependency-ordered)
| # | Issue(s) | What | Status |
|---|---|---|---|
| 1 | **M-664** (#323) | `consume`/`grow`/`impl` surface keywords (+ lexicon-legend fix); tightly-scoped Sonnet leaf; serial-on-L1 | **active — ▶ first** |
| 2 | **M-667** (#327) | E7-2 R1 remaining: `fuse` (semilattice merge — RT6), `reclaim` (supervision reclamation — RT7/M-356), `tier` (RFC-0004 `ExecutionMode` switch). After M-665/M-666 (`hypha`/`colony`, landed). Each: L1 surface + check pass + elaboration. Rebase onto post-M-664. Tightly-scoped Sonnet leaf; serial-on-L1 | next |
| 3 | **M-668** (#328) | E7-2 R2 **planning** (docs only): decompose `xloc`/`mesh`/`cyst`/`graft`/`forage`/`backbone` into per-construct implementation RFCs with honest guarantee tags + gates (a design note / DN-11 append). Gated on R1 (M-667). Parallel Sonnet leaf (docs-only; disjoint from L1 code) | last (or parallel with M-664/M-667) |

For M-667: a non-associative `fuse` merge is a `CheckError` (declared semilattice constraint);
`reclaim` elaborates to `Supervisor` calls (M-356); `tier` switches mode with an EXPLAIN-able record;
all three active in the grammar; `just check` green. RFC-0008 §4.6 R1 enactment note updated
(append-only, "implemented Rust-first, pending ratification").

## Ownership / method
- **Owns:** `crates/mycelium-l1/**`, `.claude/memory/lang-lexicon-syntax.md`, the implemented RFC/DN
  append-only notes. **Read-only / FLAG up:** `tools/github/issues.yaml`, `CHANGELOG.md`,
  `docs/Doc-Index.md`, `docs/api-index/`, workspace `Cargo.toml`.
- Per-task loop (each Sonnet leaf): **design-map → FLAG architecturally-significant choices
  (flag-don't-guess) → tightly-scoped changeset → honesty + soundness review → Copilot round →
  land.** Honest `Declared` guarantee tags; a property/soundness test per bound; never-silent
  `Result`/`Option`; `Residual` for staged-not-yet-implemented paths.
- **Done** = M-664 + M-667 + M-668 landed on `main`; the DN-03/RFC-0008 reserved-word + R1 enactment
  cross-refs updated (append-only); every issue body + status updated.
