# Kickoff `run` — make generics/traits **RUN** + first self-hosted nodule (`mycelium-l1`)

> The critical-path continuation of `lex` (which landed **M-663**). Read `.claude/agent-context.md` +
> `CLAUDE.md` (house rules win) + `.claude/kickoffs/README.md` (the tiered `dev → integration → main`
> workflow) first. **Serial-on-L1** (shares the `crates/mycelium-l1` collision files with `srf` —
> mitigation #7), so do **not** run `run` and `srf` concurrently; `run` goes first (it's the unblock).

## ⚡ RESUME HERE

**Branch off `dev`.** Work lands in `dev` first (messy OK); promote `dev → integration → main` per the
tiered workflow. **Serial-on-L1 Sonnet swarm** (Opus orchestrator + an Opus leaf per L1-touching task,
landed one at a time in dependency order). **Copilot has caught a real soundness bug on every kernel
PR this wave — review hard (soundness + never-silent G2) before each land.**

**▶ FIRST: M-673 (#351) — elaboration that makes generics + traits RUN.** Today `elab.rs` lowers a
generic *instantiation* and a trait-method call to an explicit never-silent `Residual` (staged by
M-657 and M-659). Land the elaboration:
- **Monomorphization** (RFC-0007 §11.3): a pre-pass over the checked `Env` specializes each concrete
  generic use into a monomorphic registry declaration + function instance under a mangled name,
  rewriting ctor/type/call references — so the **trusted elaborator/registry run unchanged** (no
  `mycelium-core` change; KC-3).
- **Trait dictionaries** (RFC-0019 §4.5, staged by M-659): dictionary-passing L0 lowering for
  trait-method calls.
- **Honesty:** a generic's content-addressed identity fragments across specializations — **record it,
  don't hide it** (RFC-0019 §4.4). Never silently insert a `Swap` (S1). Guarantee stays at the
  honestly-supportable strength (VR-5).
- **Acceptance:** a program defining + instantiating `List`/`first_or` (and a trait + `impl`)
  **elaborates to closed L0** (no `Residual`); the **M-210 three-way differential** agrees
  (L1-eval ≡ L0-interp ≡ AOT) on the monomorphized output; `cargo test -p mycelium-l1` green; **DN-14
  §3 rows 6 + 7 → `present`**; and **M-657 (#314) + M-659 (#318) flip to `status:done`** (this lands
  their staged half).

## Chain (dependency-ordered)
| # | Issue(s) | What | Status |
|---|---|---|---|
| 1 | **M-673** (#351) | monomorphization + trait-dictionary elaboration — makes generics/traits **run**; closes the M-657/M-659 staging | **active — ▶ first** |
| 2 | **M-649** (#284) | first self-hosted `.myc` stdlib nodule — now unblocked for a *generic* module (e.g. `std.option`: `Option` + `map`/`unwrap`/`is_some`); a non-generic one (`std.ternary`) was already doable | after M-673 |

For M-649: write the nodule in `.myc` L1 syntax; `myc-check` must exit 0; each exported fn gets a
**differential test** (Mycelium-lang value ≡ Rust reference) via the three-way checker; module meets
the M-501 contract (never-silent G2, honest per-op guarantee tags, EXPLAIN-able selections). If a
needed feature is absent, **flag the gap — don't guess** (VR-5). Then **DN-14 Status → Resolved**
(append-only, honestly recording any remaining gate-fails — no silent upgrade).

## Ownership / method
- **Owns:** `crates/mycelium-l1/**`, the implemented RFC/DN append-only notes, + (M-649) one new
  `.myc` nodule. **Read-only / FLAG up** (the integrating parent reconciles once): `tools/github/
  issues.yaml`, `CHANGELOG.md`, `docs/Doc-Index.md`, `docs/api-index/`, workspace `Cargo.toml`.
- **Serial-on-L1** (one editor of the collision files at a time). Per-task loop: **design-map → FLAG
  the architecturally-significant choices (flag-don't-guess) → Opus leaf → honesty + soundness review
  → Copilot round → land** (feature → `dev` → `integration` → `main`).
- **Done** = M-673 + M-649 landed on `main`; M-657/M-659 → `done`; DN-14 §3 rows 6/7 → `present`,
  DN-14 → Resolved; every issue body + status updated.
