# Kickoff `run` — make generics/traits **RUN** + first self-hosted nodule (`mycelium-l1`)

> **LANDED.** M-673 (monomorphization + dictionary-free static trait resolution → generics/traits run
> to closed L0) is on `main` via `claude/int-docs-mono-wave`. The remaining task (M-649 — first
> self-hosted `.myc` stdlib nodule) is captured in its own next-wave kickoff: **`std`** (`std.md`).

## Status

| # | Issue(s) | What | Status |
|---|---|---|---|
| 1 | **M-673** (#351) | monomorphization + trait-dictionary elaboration — makes generics/traits **run**; DN-14 §3 rows 6+7 → `present`; M-657/M-659 → `done` | ✓ **LANDED** on `main` |
| 2 | **M-649** (#284) | first self-hosted `.myc` stdlib nodule — unblocked by M-673 | → **`std` kickoff** (`std.md`) |

## What M-673 delivered (context for `srf` + `std`)

- **New file:** `crates/mycelium-l1/src/mono.rs` (`pub mod mono;` in `lib.rs`).
- **Elaboration path:** `elaborate`/`elaborate_colony` now call
  `crate::mono::monomorphize(env, entry)?` **before** `elab_prelude`.
- **`checkty.rs` additive change:** `Env` gained `impls: BTreeMap<(String,String), Vec<FnDecl>>` —
  every `Env { … }` literal (including tests) must include `impls`.
- **Type shape:** `Ty::Data(String, Vec<Ty>)`; `FnSig.params: Vec<TypeParam>`.
- **`Residual` sites in `elab.rs` kept** as defensive invariants — do not delete.
- Three-way differential (`L1-eval ≡ L0-interp ≡ AOT`) passes on the monomorphized output.

See `srf.md` §M-673 run-collisions for the rebase checklist every `srf` leaf must follow.
