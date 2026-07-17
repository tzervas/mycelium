# L2-B brief — Import non-type residual (G-α Rank 2)

## Identity

- **Leaf:** G-alpha-L2B-import-non-type
- **Branch:** `claude/leaf/G-alpha-import-non-type`
- **Model assigned:** `grok-composer-2.5-fast` (record actual if unavailable)
- **Base:** `origin/dev` @ `67090f4a` — **prefer serial after L2-A** if re-export depends on Clean `io`, or parallel if only `transpile.rs`/`symtab.rs`
- **PR base:** `dev` — **do not merge**

## Problem (Empirical)

std-io `lib.myc` single-file first poison:

```text
use std.io.io.read_all`: no such name `std.io.io.read_all` in the phylum
```

L2-B type co-include already handles **type** leaves (`type_defs` + `cross_nodule_type_def_closure`). Non-type resolved leaves still emit full-path `use` (`dispatch_use` else branch) — residual FLAG in `symtab.rs` module docs.

Phylum mode currently fails both `io` and `lib` on **`unknown type Result`** first; after L2-A, re-measure whether phylum `use` of `read_all` becomes Clean. Single-file `lib` may still need a non-type lever.

## Goal

Close or honestly EXPLAIN the Import **non-type** residual for free-fn re-exports (e.g. `read_all`):

- **Preferred closable path:** extract sibling **fn** surface from baseline `.myc` (mirror `extract_type_defs`) and co-include free-fn defs for resolved non-type leaves (Declared + EXPLAIN home path), **or**
- If co-include is unsafe/wrong: stop emitting oracle-false-failing `use` for non-type under single-file and record a precise Import gap (never silent success) — only if that is the honest residual after Result.

## Ownership (write)

- `crates/mycelium-transpile/src/transpile.rs` (`dispatch_use` non-type branch)
- `crates/mycelium-transpile/src/symtab.rs` (fn def extract / NoduleSymbols if needed)
- Tests under `src/tests/`
- Avoid editing L2-A Result ambient in `emit.rs` unless merge conflict forces a tiny pin — prefer FLAG
- **Do not** edit shared CHANGELOG/issues/Doc-Index/api-index

## Acceptance

1. Documented behavior for non-type batch-resolved imports (co-include **or** honest gap) with EXPLAIN.
2. `cargo fmt` + `clippy -D warnings` + `cargo test -p mycelium-transpile` green.
3. Before/after metrics on std-io if practical (`lib` CheckError residual).
4. Open PR → `dev`, **do not merge**. Report **PR# + SHA + FLAGs**.

## Out of scope

- Result ambient (L2-A)
- Macro expand (M-875)
- One-shot / SemVer claims
