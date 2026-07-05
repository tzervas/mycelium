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
| `compiler.nodule_header` | `nodule.rs` | `// nodule:` header parse (standalone) — DN-26 §7.3 named it `compiler.nodule`, which is unspellable (`nodule` is a reserved word; FLAG-nodule-5) | 2 |
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

- [x] **Stage 1** — `compiler.token` + `compiler.lex` (token-stream differential). Landed
      (`lib/compiler/token.myc`, `lib/compiler/lex.myc`; gate: `crates/mycelium-l1/tests/compiler_stage1.rs`).
      `compiler.token`: the full `Tok`/`Pos`/`Spanned`/`keyword` port, all 64 reserved words
      classification-differentialed against the Rust oracle (`token::keyword`) 1:1. `compiler.lex`:
      the full lexer (trivia, all punctuation/operators, identifiers+keywords, `0b`/`0x`/`0t`
      literals, decimal `Int`, `"…"` strings) token-COUNT-differentialed against the Rust oracle
      over **every file in the accept-corpus** (27/27), plus per-token kind/content spot-checks.
      Two real checker findings were surfaced by this port (reported, not silently worked around;
      full detail in the test file): (1) a combined two-level nested match pattern (e.g.
      `Some(Scalar(SF16))`, or `Some(Sp(Ctor, _))`) panics `usefulness::useful_budgeted` when the
      outer type has `Tok`'s ~80 variants — worked around in every test via a split-match idiom,
      never hit by `lex`'s own logic (which only constructs `Tok`, never destructures it); (2)
      `mycelium_interp::Interpreter::eval_core` (the L0 substitution interpreter) does not return
      within minutes for a `lex.myc`-scale elaborated program even though L1-eval finishes in
      well under a second, so the Stage-1 differential compares the **L1-eval** leg only (still a
      complete "Rust-lexer ≡ self-hosted-lexer" comparison; the L0-interp/AOT cross-check used by
      `std_*.rs`'s three-way harness is not currently feasible at this scale). Deliberate, flagged
      scope narrowings vs. the Rust oracle: no float-literal scanning (none in the accept-corpus),
      ASCII-only whitespace, `Int`/literal payloads carry verbatim `Bytes` rather than an eagerly
      converted value (mirrors the Rust lexer's own deferred-conversion precedent for every OTHER
      literal kind). `compiler.token`/`compiler.lex` are mutually self-contained (each redeclares
      the shared types) rather than cross-nodule `use`, because cross-nodule EXECUTION (not just
      type-checking) is still staged in `checkty.rs`'s `check_phylum` — a real, separate finding
      from this leaf, reported upstream for M-741/a future stage to lift.
- [x] **Stage 2** — `compiler.nodule_header` (header-parse differential). Landed
      (`lib/compiler/nodule.myc`; gate: `crates/mycelium-l1/tests/compiler_stage2.rs`). The full
      `nodule.rs` port: `parse_nodule_header` (first-non-blank-line scan, blank-line skipping with
      1-based line tracking), the `//`-comment / bare-`nodule` / `nodule:<dotted>` recogniser, the
      never-silent ill-formed-name errors (empty name, empty segment, non-identifier segment — G2),
      and the `dotted`/`canonical` accessors. Differential vs the live Rust oracle: a 4-way
      classification code (none/bare/named/error) plus the joined dotted name and `canonical`
      spelling (named case) plus the 1-based error line (error case), over a 26-case synthetic edge
      battery transcribed from the oracle's own unit tests AND every real `.myc` file in the
      conformance corpus (accept **and** reject) and `lib/std/` + `lib/compiler/` (66+ files). One
      THREE-WAY run (L1 ≡ L0-interp ≡ AOT) is kept at this stage's small scale; the per-file sweep
      is L1-eval-only (M-981, as in Stage 1). Honest narrowings (flagged in-file): ASCII-only trim
      vs Rust's Unicode `str::trim` (FLAG-nodule-2 — a real classification divergence: a
      non-ASCII-whitespace-only leading line hides a later marker from the port; PINNED as a
      known-divergence test in the gate, per the PR #1165 review finding); static error messages,
      line fidelity kept (FLAG-nodule-3). **One real finding: the DN-26 §7.3 nodule name
      `compiler.nodule` is unspellable** — `nodule` is a reserved word, so the surface declaration
      `nodule compiler.nodule;` cannot parse (the FLAG-token-3 keyword-collision class at the
      nodule-NAME level); renamed `compiler.nodule_header` (FLAG-nodule-5, reported up for DN-26's
      append-only changelog). Every source-length-bounded recursion is direct-tail (the RFC-0041
      §7 W7 amendment-11 TCO acceptance criterion); the non-tail recursions are bounded by a
      name's segment count, never by source length.
- [ ] **Stage 3** — `compiler.ast` + `compiler.parse` (AST differential + full conformance corpus)
- [ ] **Stage 4** — `compiler.ambient` + `compiler.totality` + `compiler.substrate` (leaf differentials)
- [ ] **Stage 5** — `compiler.semcore` (L0-output differential; `cargo-mutants` witness)
- [ ] **Stage 6 / M-742** — `just bootstrap`: interpreted-first then AOT, stage-2 three-way

*This README is the M-740 wave map; it is updated as each stage lands. Grounded in DN-26 §7/§9,
DN-14, RFC-0028, KC-3.*
