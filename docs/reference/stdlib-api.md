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
**E13-1**. That migration is **in progress**: as of 2026-06-27, **eight** stdlib nodules self-host
as `.myc` prototypes (RFC-0031 §5 D4, under the D5 stability bar — *not* the frozen migration), so
the generated `.myc` API reference currently covers:

| Nodule | Source | Public `fn`s | Status |
|---|---|---|---|
| `std.result` | `lib/std/result.myc` | `is_ok`, `is_err`, `unwrap_or`, `map`, `and_then`, `fold`, `map_err`, `or_else` | runs three-way (M-649) |
| `std.option` | `lib/std/option.myc` | `is_some`, `is_none`, `unwrap_or`, `map`, `and_then`, `fold`, `or_else`, `flatten` | runs three-way (Tier-0, M-715) |
| `std.cmp` | `lib/std/cmp.myc` | `is_lt`/`is_eq`/`is_gt`, `reverse`, `bool_eq`/`bool_cmp`, `ord_eq`, **width-generic** `cmp`/`le`/`ge`/`max`/`min` | runs three-way (M-718; width-generic, M-753) |
| `std.math` | `lib/std/math.myc` | **width-generic** `badd`/`bsub`/`band`/`bor`/`bxor`/`bnot` (`Binary{N}`), `tadd`/`tsub`/`tmul`/`tneg` (`Ternary{M}`) | runs three-way (M-718; primitive surface, RFC-0032 D2) |
| `std.collections` | `lib/std/collections.myc` | `empty`/`push_front`/`is_empty`/`head`/`tail`/`len`/`get`/`snoc`/`reverse`, **width-generic** `map_*`/`set_*` (`map_get<N,V>`, `set_contains<N>`) | runs three-way (M-718; unblocks M-716) |
| `std.iter` | `lib/std/iter.myc` | `is_empty_l`, `length`, recursive HOF `map`/`filter`/`foldl`/`any`/`all`/`find` | runs three-way (M-715 recursive-HOF; RFC-0024 §4) |
| `std.text` | `lib/std/text.myc` | UTF-8 validity + decode surface (`byte_len`/`slice`/`is_ascii_byte`/`reject_two..four`/`decode_one..four`, …) | runs three-way (M-717 validity layer) |
| `std.fmt` | `lib/std/fmt.myc` | `hex_digit`, `nibble_lo`/`nibble_hi`, `to_hex` | self-hosted projection surface |

> **Honesty (VR-5).** "Runs three-way" = L1-eval ≡ L0-interp ≡ AOT agreement is `Empirical` (trials,
> `crates/mycelium-l1/tests/std_*.rs` + `std_generic_conformance.rs`); nothing is `Proven`. These are
> **prototypes** — the Rust `mycelium-std-*` crates remain the **differential oracle** (RFC-0031 D6)
> and are **not** retired by them.

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
