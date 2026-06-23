# Standard Library API Reference (generated)

> **Status: Empirical/Declared.** This page describes the **generated** per-module stdlib API
> documentation (M-736). The API docs themselves are *projected from source* by `mycelium-doc` —
> a `.myc` nodule's header `@summary`, its `fn` signatures, and each `fn`'s preceding `//` doc
> comment become API-item entries; an undocumented symbol is rendered "undocumented", **never
> invented** (G2). Source is ground truth.

## What is generated

`crates/mycelium-doc` (`myc-doc`) scans every `.myc` nodule under `lib/std/` and projects it into
the API reference, alongside the corpus and JSON schemas. For each stdlib nodule you get:

- the **nodule** entry (its `@summary` header), and
- one **API item per `fn`** — its signature plus the contiguous `//` doc comment immediately above
  it (or "undocumented" when there is none — an explicit, honest gap), and
- the **whole source** captured as a *checked example* (it is type-checked by the `myc-check`
  pipeline, so the documented code is real, dogfooded code — never an illustrative fiction).

## Build it

```sh
# Project the corpus + stdlib into the doc-IR and emit HTML / Typst / JSON views:
cargo run -p mycelium-doc --bin myc-doc -- build --repo-root . --out target/doc

# Run the §4.1 quality-bar lint (also part of `just check` via scripts/checks/myc-doc.sh):
cargo run -p mycelium-doc --bin myc-doc -- lint --repo-root .
```

The generated views land under `target/doc/` (a build artifact, like rustdoc's output — not
committed). The **lint is part of `just check`**, so the stdlib API docs are *validated on every
check*: the `checked-examples` gate type-checks each stdlib source, and the `no-hallucinated-prose`
gate confirms every API statement traces to source and every gap is flagged.

## Current coverage (honest — gated on E13-1)

The full-language 1.0.0 north star is a stdlib written **fully in Mycelium** (`.myc`) — epic
**E13-1**. That migration is **in progress**: today exactly one stdlib module self-hosts, so the
generated `.myc` API reference currently covers:

| Nodule | Source | Public `fn`s | Doc state |
|---|---|---|---|
| `std.result` | `lib/std/result.myc` | `is_ok`, `is_err`, `unwrap_or`, `map`, `and_then`, `fold` | nodule `@summary` + `map`/`and_then`/`fold` documented from source; `is_ok`/`is_err`/`unwrap_or` flagged undocumented |

The remaining standard-library modules are **Rust-first** today (`crates/mycelium-std-*`, with their
specifications in `docs/spec/stdlib/`). They appear in this generated `.myc` API reference **as they
are ported to `.myc` under E13-1** — the coverage grows with the migration, and the gap is **never
silent** (it is stated here and flagged by the lint), not hidden behind a fake-complete index.

> **Why not "every public symbol documented" yet?** Because "every public symbol" of the *full*
> stdlib presupposes E13-1 is complete (all modules self-hosted). Until then, completeness is bounded
> by what self-hosts; claiming otherwise would violate the honesty rule (VR-5/G2). The *generation
> mechanism* is complete and green; the *content* fills in as E13-1 lands modules.

## See also

- `docs/spec/stdlib/` — the per-module standard-library **specifications** (all 25 ratified).
- `lib/std/result.myc` — the first self-hosted stdlib nodule (M-649).
- `crates/mycelium-doc/src/apiref.rs` — the projection that generates these docs.
- `docs/reference/language-reference.md` — the language surface the stdlib is written in.

## Changelog

- **2026-06-23 — Created (M-736).** Documents the generated per-module stdlib API reference: wired
  `lib/std/` into the `mycelium-doc` apiref build, added per-`fn` source-comment extraction (summaries
  trace to source, never invented), and recorded the honest E13-1-gated coverage (`std.result`
  today, growing with the self-hosting migration). Guarantee: `Declared`. Append-only.
