# `lib/compiler/` — the self-hosted L1 frontend (Mycelium-in-Mycelium)

> **Status:** M-740 wave scaffold (2026-07-03). This phylum is the `.myc` port of the Rust L1
> frontend (`crates/mycelium-l1/src/`), per **DN-26** (bootstrap plan) and the maintainer's
> **DN-26 §9** flag decisions. The Rust frontend stays the **trusted differential oracle** until
> **M-741** ratifies the port canonical — nothing here overwrites `crates/mycelium-l1/src/`.

## Phylum layout (DN-26 §9 flag-1: the `compiler` phylum, SCC as one nodule)

`lib/compiler/` is a **phylum** (RFC-0006 §4.3). Nodules:

| Nodule | Ports (Rust) | Role | Stage |
|---|---|---|---|
| `compiler.token` | `token.rs`, `error.rs` | token kinds + lex errors (the small `token↔error` cycle) | 1 |
| `compiler.lex` | `lexer.rs` | source text → token stream | 1 |
| `compiler.nodule` | `nodule.rs` | `// nodule:` header parse (standalone) | 2 |
| `compiler.ast` | `ast.rs` | surface AST data types (pure, no upward deps) | 3 |
| `compiler.parse` | `parse.rs` | token stream → AST / phylum | 3 |
| `compiler.ambient` | `ambient.rs` | ambient-representation resolution (leaf dep of the SCC) | 4 |
| `compiler.totality` | `totality.rs` | totality walk (leaf dep) | 4 |
| `compiler.substrate` | `substrate.rs` | substrate release events (leaf dep) | 4 |
| **`compiler.semcore`** | `checkty.rs` · `elab.rs` · `eval.rs` · `mono.rs` · `fuse.rs` · `decision.rs` · `usefulness.rs` · `grade.rs` · `affine.rs` | **the semantic-core SCC — one nodule** (nodule-wide `FixGroup` mutual recursion, DN-14 row 3) | 5 |

The **semantic core is one nodule** because those nine Rust modules form a single strongly-connected
component (they call each other cyclically — DN-26 §7.1); a single nodule gives them nodule-wide
mutual recursion for free. The leaves (`ambient`/`totality`/`substrate`) and the front stages
(`token`/`lex`/`nodule`/`ast`/`parse`) are **sibling nodules** exporting across nodule boundaries with
`pub` + cross-nodule `use` (DN-14 row 10).

## Frontend / kernel boundary (KC-3)

This phylum is `source text → closed L0`. The **L0 kernel stays Rust** (`mycelium-core`/`interp`/
`cert`/`select`) — it is **not** ported here (KC-3). The self-hosted frontend reaches L0 construction
and the prim registry through the `@std-sys` + `wild` FFI seam (DN-14 row 9, executes; RFC-0028). A
frontend port step that *appears* to need a `mycelium-core` change is a **FLAG-up**, not an in-wave
core edit.

## Verification per stage (DN-26 §7.3 + §9 flag-2)

Each stage lands as a **separate green-`just check` commit** with a **differential** against the Rust
oracle over the L1 conformance corpus (`docs/spec/grammar/conformance/accept|reject/`):

- **Stages 1–5:** `Rust-host ≡ self-host` on the same output for the corpus, graded **`Empirical`**
  (differential agreement across trials — never upgraded to `Proven`, VR-5).
- **Stage 6 (bootstrap gate, M-742):** per **DN-26 §9 flag-2** — **validate on the interpreted `myc`
  first**, then **AOT-compile the same `.myc` and validate that build too**. Both runtimes run the
  identical source and must agree: the stage-2 three-way `Rust-host ≡ self-host-interpreted ≡
  self-host-AOT`. The interpreted pass is the gate; the AOT pass is the never-skipped follow-on (G2).

Differential harnesses live in `crates/mycelium-l1/tests/` (the established `std_*.rs` pattern),
reading both the Rust output and the self-hosted output for the same input.

## Honesty / status discipline

- Nothing here is pre-declared done. Each stage's differential is `Empirical` **only after trials
  run** (VR-5). DN-26 stays **Draft** until **M-741** ratifies the full-toolchain gate.
- The full-language 1.0.0 capstone criterion is **never pre-declared** — this is the
  comprehensive-dogfooding track (ADR-036) that gates the *public-release* milestone, not the
  `lang 1.0.0` tag.

## Wave progress

- [ ] **Stage 1** — `compiler.token` + `compiler.lex` (token-stream differential)
- [ ] **Stage 2** — `compiler.nodule` (header-parse differential)
- [ ] **Stage 3** — `compiler.ast` + `compiler.parse` (AST differential + full conformance corpus)
- [ ] **Stage 4** — `compiler.ambient` + `compiler.totality` + `compiler.substrate` (leaf differentials)
- [ ] **Stage 5** — `compiler.semcore` (L0-output differential; `cargo-mutants` witness)
- [ ] **Stage 6 / M-742** — `just bootstrap`: interpreted-first then AOT, stage-2 three-way

_This README is the M-740 wave map; it is updated as each stage lands. Grounded in DN-26 §7/§9,
DN-14, RFC-0028, KC-3._
