# Mycelium Lexicon, Syntax, Grammar & Naming Conventions — Memory File

**Status: Empirical/Declared** — source + RFC/ADR/DN are ground truth; this is an orientation aid.
Not normative. Cite from here, verify in source. Ground truth lives in `docs/notes/DN-02`, `DN-03`,
`DN-06`, `docs/Glossary.md`, `docs/spec/grammar/mycelium.ebnf`, and `crates/mycelium-l1/src/token.rs`.

---

## What it is

The Mycelium language vocabulary, grammar, and naming conventions. The language is called
**Mycelium** (shared with the project). Source files use the `.myc` extension. The grammar is
W3C-EBNF, machine-readable, and backed by an accept/reject conformance corpus — the corpus is the
oracle, not any single parser.

The naming philosophy is a **hybrid**: fungal-themed terms where the metaphor is **accurate and
illuminating**; conventional terms (from Rust, ML, etc.) where a borrowed word is clearer for
human and machine readers. The gate is three tests from **DN-02 §1**.

---

## Where it lives

| Document | Role |
|---|---|
| `docs/notes/DN-02-Fungal-Lexicon-and-Reserved-Words.md` | Naming LAW + three-test gate; the original reserved-word set (Resolved 2026-06-10) |
| `docs/notes/DN-03-Lexicon-Amendment-Surface-and-Runtime-Forms.md` | Surface additions (`consume`, `grow`, `thaw`); runtime names (one per concept); one-name-per-term rule (Resolved 2026-06-10) |
| `docs/notes/DN-06-Static-Organization-and-Dynamic-Grouping-Lexicon.md` | `phylum` / `nodule` / `colony` (load-bearing; Resolved 2026-06-16); `// nodule:` header rule; M-358 migration |
| `docs/Glossary.md` | The dedicated term reference (Index + per-term detail, 2026-06-16) |
| `docs/notes/Lexicon-Reference.md` | Terse catalog with mnemonics and tier table (Draft v0.4) |
| `docs/spec/grammar/mycelium.ebnf` | **NORMATIVE grammar oracle** (W3C EBNF, v0, L1-facing) |
| `docs/spec/grammar/README.md` | Grammar discipline, conformance corpus layout |
| `docs/spec/grammar/conformance/` | `accept/` (must parse) and `reject/` (must fail with explicit error) |
| `docs/rfcs/RFC-0006-Surface-Language-and-Term-Layering.md` | The L0→L1→L2/L3 layer cake; invariants S1–S6 (Accepted r5) |
| `docs/rfcs/RFC-0007-L1-Kernel-Calculus.md` | L1 ten-node budget, types, totality (Accepted r4) |
| `docs/rfcs/RFC-0020-L2-Surface-Term-Language.md` | L2 surface — ratification status: FLAG — verify current status in doc |
| `docs/spec/Nodule-Header-and-Project-Manifest.md` | `// @key: value` header schema + `mycelium-proj.toml` manifest (Accepted, Enacted M-359) |
| `docs/spec/Spore-Build-and-Publish-Contract.md` | `spore` as deployable unit (ADR-013) |
| `crates/mycelium-l1/src/token.rs` | **Actual reserved-word set in code** — ground truth for which words lex as keywords today |
| `docs/notes/Example-Programs-Reference.md` | `.myc` example snippets (some pre-ratification; see Gotchas) |

---

## Generic-to-Mycelium term mapping

**THIS IS THE MOST IMPORTANT CALLOUT FOR ORIENTATION.**

| Generic / Other-language concept | Mycelium term | Notes |
|---|---|---|
| library / package / crate | **`phylum`** | Content-addressed, versioned, library-scale static unit (DN-06) |
| module / namespace | **`nodule`** | The basic static organizational unit; replaces "module" (DN-06) |
| published artifact / package | **`spore`** | Deployable unit; germinates into a running `colony` of `hypha` |
| thread / task / goroutine | **`hypha`** | Single structurally-scoped concurrent execution unit (DN-03) |
| runtime task-group / process-group | **`colony`** | Dynamic runtime grouping of active `hypha` (DN-06) |
| unsafe block | **`wild`** | Denied-by-default, lexically-marked (DN-02) |
| type-class / interface | **`trait`** | Kept conventional — `guild` was declined (DN-02 §7) |
| representation change / cast | **`swap`** | Native corpus term, never borrowed; always explicit |
| checkpoint / serialized state | **`cyst`** | Content-addressed dormable-resumable computation (DN-03) |
| inherent methods / impl block | **`impl`** | Kept conventional — `embody` was declined (DN-03 §1) |

**Key distinction:** The Rust kernel implementation packages are genuine Rust crates named
`mycelium-*` (e.g. `mycelium-l1`, `mycelium-interp`). Those are Rust crates in the Cargo sense.
The Mycelium-**language** organizational units are **phyla** (library-scale) and **nodules**
(module-scale). Do not call Mycelium-language units "crates" or "modules" — the correct terms
are `phylum`/`nodule`.

---

## The naming law and three-test gate (DN-02 §1)

Every candidate term must pass all three to be themed; failing any means keep the conventional term:

- **T-map (fidelity).** Does the fungal metaphor map *accurately* to the behavior — not just
  decoratively? A metaphor implying behavior the construct does **not** have is *disqualified*.
- **T-illuminate (teaching value).** Does the themed term *teach* something about the behavior
  that the conventional term does not?
- **T-learn (dual readability).** Does keeping a conventional term aid learnability/readability
  for **both** human and machine readers (LLM familiarity) more than theming would?

**Net rule:** theme the Mycelium-unique concepts (no familiar baseline to lose); keep conventional
where a borrowed term is clearer to learn and read. Control-flow and binding keywords (`let`, `if`,
`match`) always score high on T-learn.

**One name per term** (DN-03 §3): no canonical/alias pairs. ADR-012 §7.6's long-form + short-alias
scheme was **rejected** as needless surface area. Each concept has exactly one name; pick the
single clearest.

---

## Full reserved-word table

Verified against `crates/mycelium-l1/src/token.rs` and `docs/spec/grammar/mycelium.ebnf`.

**Status legend (be precise — this is the highest-value correctness surface):**
- **Active** — in the lexer's `keyword()` *and* consumed by a construct (it forms/opens programs).
- **Reserved-not-active** — in `keyword()` (lexes as a keyword, so it can *never* be a silent
  identifier — G2) but no construct consumes it yet: **only `phylum` and `colony`**.
- **Ratified — not yet lexed** — a name ratified in DN-02/DN-03 but **not** in `keyword()`, so it
  currently lexes as an ordinary **identifier** (using it is *not* yet an error). The whole Runtime
  tier plus `consume`/`grow`/`impl` are here — their lexer reservation lags the spec.

**Gap-closure tracking (2026-06-22):** the **Reserved-not-active** and **Ratified — not yet lexed**
gaps are tracked under **E7-1** (L1 language features; issues M-656…M-664, Phase 5) and **E7-2**
(runtime vocabulary; M-665…M-668, Phase 7). Progress: **M-656/657/658** (generics), **M-659** (traits;
`Tok::Trait`/`Tok::Impl` active; elab STAGED → M-673), **M-660** (effects; `Tok::Bang`; checker-only,
no L0 node) — all LANDED. **Remaining E7-1:** M-661 (`wild`/FFI), M-662 (`phylum`/cross-nodule), M-663
(grading), M-664 (`consume`/`grow`/`impl`). **E7-2 remaining:** M-667 (`fuse`/`reclaim`/`tier`), M-668
(R2). A status row flips only when its tracking issue lands and `just check` is green (VR-5).

| Term | Layer | Status | Meaning | Normative source |
|---|---|---|---|---|
| `nodule` | L2 Surface (static) | **Active** | The basic static organizational unit (replaces "module"); opens a program | DN-06; RFC-0006 |
| `phylum` | L2 Surface (static) | **Reserved-not-active** | Library-scale grouping above nodules; lexes as keyword but no construct consumes it yet | DN-06; RFC-0006 |
| `colony` | Runtime | **Reserved-not-active** | Dynamic runtime grouping of active `hypha`; lexes as keyword, never a silent identifier; reassigned from its former static meaning | DN-06; RFC-0008 §4.7 |
| `use` | L2 Surface | **Active** | Import (conventional) | DN-02 §3 |
| `type` | L2 Surface | **Active** | Data-type (sum) declaration (conventional; `species` declined) | DN-02 §7 |
| `trait` | L2 Surface | **Active** | Typeclass / behavior set (conventional; `guild` declined); `Tok::Trait` active; **trait checker + coherence LANDED M-659** — type-checks; dictionary-passing L0 lowering **STAGED → M-673** (does NOT yet RUN) | DN-02 §7; RFC-0019 |
| `fn` | L2 Surface | **Active** | Function definition (conventional; `spawn`/`grow` rejected on T-map) | DN-02 §3 |
| `thaw` | L2 Surface | **Active** | De-maturation: keeps one `fn` interpreted inside a `matured` scope; `thaw fn f(…)` | RFC-0017 §4.3/§5; DN-03 changelog |
| `let` | L2 Surface | **Active** | Local binding (conventional) | DN-02 §3 |
| `in` | L2 Surface | **Active** | Binding body separator (`let x = e in body`) | grammar |
| `if` / `then` / `else` | L2 Surface | **Active** | Conditional (conventional) | DN-02 §3 |
| `match` | L2 Surface | **Active** | Pattern match (conventional; `sift` declined) | DN-02 §3 |
| `for` | L2 Surface | **Active** | Bounded iteration sugar (structural recursion, Total by construction) | RFC-0007 §4.8 r2; DN-03 §2 |
| `swap` | L1/L0 | **Active** | The never-silent representation change (native corpus term) | RFC-0001 §4.5; RFC-0002 |
| `default` | L2 Surface | **Active** | Opens a nodule-scope ambient declaration (`default paradigm P`) | RFC-0012 §4.2 |
| `paradigm` | L2 Surface | **Active** | Ambient granularity keyword (`default paradigm P` / `with paradigm P`) | RFC-0012 §4.2 |
| `with` | L2 Surface | **Active** | Opens a block-scope ambient override (`with paradigm P { … }`) | RFC-0012 §4.4 |
| `wild` | L2 Surface | **Active** | Denied-by-default unsafe block (themed); the only FFI/raw-memory site | DN-02 §5/§7 |
| `spore` | Surface / Deploy | **Active** | Reconstruction-manifest construction; also the deployable artifact form | DN-02 §2/§7; RFC-0003 §6; ADR-013 |
| `to` | L2 Surface | **Active** | Swap target label (within `swap(…, to: …, policy: …)`) | grammar |
| `policy` | L2 Surface | **Active** | Swap policy label (within `swap(…, to: …, policy: …)`) | grammar |
| `matured` | L2 Surface | **Active (reserved keyword)** | Scope-level promotion to AOT-compiled, stable form; a `matured fn` at item position is a parse error with teaching diagnostic | RFC-0017; DN-02 §7 |
| `impl` | L2 Surface | **Active** (`Tok::Impl` lexed, M-659) | Trait implementation block (`impl Trait<X> for Y { … }`); `Tok::Impl` in the lexer; inherent-impl `impl T { … }` uses the same token; dictionary-passing L0 lowering **STAGED → M-673** | DN-03 §1; RFC-0019; M-659 |
| `Binary` | Type | **Active** | N-bit binary representation type (`Binary{N}`) | RFC-0001; grammar |
| `Ternary` | Type | **Active** | N-trit balanced-ternary type (`Ternary{N}`) | RFC-0001; grammar |
| `Dense` | Type | **Active** | Dense embedding type (`Dense{N, scalar}`) | RFC-0001; grammar |
| `VSA` | Type | **Active** | Hypervector type (`VSA{model, dim, sparsity}`) | RFC-0001; grammar |
| `Substrate` | Type | **Active** | Affine external resource kind (`Substrate{Name}`); consumed exactly once | DN-02 §2; LR-8 |
| `Sparse` | Type qualifier | **Active** | Sparsity qualifier for VSA (`Sparse{N}`) | grammar |
| `F16`, `BF16`, `F32`, `F64` | Scalar kind | **Active** | Scalar type keywords for Dense | grammar |
| `Exact`, `Proven`, `Empirical`, `Declared` | Formal / Honesty | **Active** | Guarantee strength tags; type-level index `T @ Exact` (LR-6) | RFC-0001; DN-02 §7 |
| `consume` | L2 Surface (future) | **Ratified — not yet lexed** | Acquire exclusive ownership of an affine `substrate` (single-use semantics) | DN-03 §1 |
| `grow` | L2 Surface (future) | **Ratified — not yet lexed** | Derive-like generative capability extension (`grow Debug for T`) | DN-03 §1 |
| `hypha` | Runtime (future) | **Ratified — not yet lexed** | Single concurrent execution unit | DN-03 §4; RFC-0008 §4.5 |
| `fuse` | Runtime (future) | **Ratified — not yet lexed** | Lawful state fusion: semilattice merge of two `hypha` states | DN-03 §4; RFC-0008 §4.5/RT6 |
| `mesh` | Runtime (future) | **Ratified — not yet lexed** | Gossip/pub-sub overlay with honest probabilistic guarantees | DN-03 §4; RFC-0008 §4.5/RT5 |
| `graft` | Runtime (future) | **Ratified — not yet lexed** | Capability contract with external infrastructure | DN-03 §4; RFC-0008 §4.5/RT4 |
| `cyst` | Runtime (future) | **Ratified — not yet lexed** | Content-addressed checkpoint of a dormable computation | DN-03 §4; RFC-0008 §4.5/RT2 |
| `xloc` | Runtime (future) | **Ratified — not yet lexed** | Explicit, fallible, Meta-preserving value movement ("trans-locate") | DN-03 §4; RFC-0008 §4.5/RT1/RT4 |
| `forage` | Runtime (future) | **Ratified — not yet lexed** | Adaptive placement policy (reified RFC-0005 selection) | DN-03 §4; RFC-0008 §4.5/RT3 |
| `backbone` | Runtime (future) | **Ratified — not yet lexed** | Declared high-bandwidth transport path (placement artifact, semantics-free) | DN-03 §4; RFC-0008 §4.5/RT3 |
| `tier` | Runtime (future) | **Ratified — not yet lexed** | Execution-mode switch (interpreted vs native); distinct from a `swap` (Repr change) | DN-03 §4; RFC-0008 §4.5 |
| `reclaim` | Runtime (future) | **Ratified — not yet lexed** | Supervision-tree reclamation of stale runtime units (never memory — LR-9 makes memory automatic) | DN-03 §4; RFC-0008 §4.5/RT7 |

**Reserved-not-active words lex as keywords — they can never be silent identifiers.** Using
`phylum` or `colony` as a function name is a parse error (verified in
`crates/mycelium-l1/src/lib.rs` test `phylum_and_colony_are_reserved_not_active`). No production
consumes them yet, so they do not open a program (see `docs/spec/grammar/conformance/reject/10-reserved-not-active.myc`).

**Words not reserved (DN-02 §6):** `while`, `loop`, `break`, `continue`, `return` —
unbounded iteration undermines the divergence bit. The toolchain emits **teaching diagnostics**
when they appear, pointing at recursion or `for`. (`embody` is also unreserved — it was declined in
**DN-03 §1**; inherent methods keep `impl`.)

---

## The L0 → L1 → L2/L3 layer cake (RFC-0006 §3)

```
L3  Projections / editor surface     ← committed (DN-09 KC-2 verdict: text syntax + structured
                                        projection layer co-equally; M-380 for projections)
L2  Surface term language ("Myc")    ← the Rust-class language: ADTs, traits, nodules, recursion
L1  Kernel calculus                  ← small typed core: λ + data + explicit recursion + Repr types
L0  Core IR (frozen, RFC-0001)       ← Const | Var | Let | Op | Swap + Meta/WF1–WF5
```

- **L0 is the trusted base** (KC-3, ADR-007). It is frozen; changes need their own RFC. The
  reference interpreter runs L0. `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` guarantee tags live here.
- **L1 adds five nodes** to L0: `Lam | App | Construct | Match | Fix` (RFC-0007 §3). Data-type
  declarations live in a content-addressed registry, not in the term language.
- **L2 is elaboration-defined** — every L2 construct has a specified desugaring to L1; there is no
  independent L2 semantics. Elaboration is always `EXPLAIN`-able (S4/ADR-006 — no black box).
- **Content-addressed identity** (ADR-003) is over elaborated **L0**, not the surface keyword. A
  header date-bump does not change identity; a code change does.
- The committed grammar artifacts are `docs/spec/grammar/mycelium.ebnf` — the normative **W3C-EBNF
  grammar oracle** (an LR(1)/LALR(1)-class grammar) — plus the accept/reject conformance corpus under
  `docs/spec/grammar/conformance/`; `scripts/checks/grammar.sh` and the parser conformance tests are
  the machine-verified gates. (No separate LR(1) parser-table artifact is committed — the EBNF is the
  oracle.)

**Invariants every layer must preserve (RFC-0006 §4.1):**

- **S1 (never-silent swap):** A representation change is *lexically visible* at every layer. No
  sugar or inference step may insert a `Swap`. A `swap` always names both `to:` and `policy:`.
- **S2 (honest tags surface):** The guarantee lattice is part of every binding's observable
  interface; a `Declared` value is always visibly flagged.
- **S3 (content-addressed identity):** Definition identity is the structure hash (ADR-003) at
  every layer — names are bindings to hashes, never identity.
- **S4 (inspectable elaboration):** Every L2→L1→L0 step is dumpable and diffable; the elaborator
  is not a black box.
- **S5 (explicit partiality):** Out-of-range, illegal pair, and unsupported composition are
  explicit `Option`/`Result`/diagnostic. No surface construct may erase a kernel refusal.

---

## Surface syntax and usage

### The `// nodule:` header marker (DN-06 §6)

A Mycelium source file declares its nodule on its **first non-blank line**:

```mycelium
// nodule: geometry.shapes
```

or bare (subnodule, inheriting metadata):

```mycelium
// nodule
```

File and directory names stay **conventional** — `nodule` is never forced into a path. Recognised
by `mycelium_l1::parse_nodule_header`; the M-141 linter surfaces malformed markers; the M-142
formatter preserves valid ones. The marker is **not** part of content-addressed identity (metadata
is not identity — ADR-003).

### The structured `// @key: value` header (Nodule-Header spec §3)

On a nodule/phylum root, optional `@`-prefixed metadata lines follow the required marker. Closed
v0 key set (all values are metadata, never part of the content hash):

```mycelium
// nodule: ml.inference
// @license: Apache-2.0
// @authors: Tyler Zervas <...>
// @since: 2026-01-10
// @updated: 2026-06-16
// @version: 0.3.0
// @matured: true
```

Subnodules inherit from the parent/manifest. Unknown keys are an explicit lint error (G2 —
never silently ignored). The `mycelium-proj.toml` manifest is the default inheritance source.

### The `mycelium-proj.toml` manifest (Nodule-Header spec §2)

The project-level manifest (the pyproject.toml / Cargo.toml analogue). `[project]` with
`name`/`kind` is the only required table:

```toml
[project]
name    = "geometry"
kind    = "phylum"       # "phylum" (library) | "program" | "script"
version = "1.2.0"
license = "Apache-2.0"
lang    = "mycelium-0"   # surface-language edition (MSRV-analogue)

[dependencies]
numerics = { phylum = "numerics", version = "^2", hash = "blake3:..." }
```

### Example: a minimal nodule with a swap

```mycelium
// nodule: demo
nodule demo
fn f(x: Binary{8}) -> Ternary{6} =
  swap(x, to: Ternary{6}, policy: rt)
```

Both `to:` and `policy:` are mandatory in `swap(…)`. Omitting `policy` is a parse error (S1/WF2).

### Example: matured scope + `thaw` exception

```mycelium
// nodule: ml.inference
// @matured: true
nodule ml.inference

fn inference_pipeline(input: Dense{1024, F32}) -> Dense{1024, F32} = input

thaw fn experimental_kernel(input: Dense{1024, F32}) -> Dense{1024, F32} = input
```

`// @matured: true` in the header (or in `mycelium-proj.toml`) promotes the whole scope to AOT.
`thaw fn` keeps one definition interpreted. `matured fn` at item position is a **parse error**
with a teaching diagnostic (RFC-0017 §4.1).

### Example: type declaration and pattern match

```mycelium
// nodule: geometry.shapes
nodule geometry.shapes

type Shape = Circle(Binary{8}) | Square(Binary{8}) | Triangle(Binary{8}, Binary{8})

fn area(s: Shape) -> Binary{16} =
  match s {
    Circle(r)    => r,
    Square(w)    => w,
    Triangle(b, h) => b,
  }
```

Match coverage is checked by the **Maranget usefulness algorithm** (`crates/mycelium-l1/src/usefulness.rs`)
— both exhaustiveness and redundancy — never assumed.

---

## Grammar overview

The EBNF in `docs/spec/grammar/mycelium.ebnf` is the normative oracle. Key productions:

```ebnf
program        ::= nodule_header item*
nodule_header  ::= 'nodule' path
item           ::= use_item | default_item | type_item | trait_item | impl_item | fn_item
fn_item        ::= 'thaw'? 'fn' Ident type_params? '(' params? ')' '->' type_ref effect_ann? '=' expr
type_params    ::= '<' type_param (',' type_param)* '>'
type_param     ::= Ident (':' bound ('+' bound)*)?   /* bounded type param; bound = trait name */
effect_ann     ::= '!' '{' (Ident (',' Ident)*)? '}'  /* absent = pure; '!{}' = explicit pure */
trait_item     ::= 'trait' Ident type_params? '{' fn_sig* '}'
impl_item      ::= 'impl' Ident type_args? 'for' type_ref '{' fn_item* '}'
type_ref       ::= base_type ('@' strength)?
swap_expr      ::= 'swap' '(' expr ',' 'to' ':' type_ref ',' 'policy' ':' path ')'
wild_expr      ::= 'wild' '{' expr '}'
for_expr       ::= 'for' Ident 'in' app_expr ',' Ident '=' app_expr '=>' expr
```

**Landed surface — honesty notes (VR-5):**
- **Generics** (`type List<A>`, `fn f<A>(…)`) — type-check via unification (`Ty::Var` + `Ty::Data(name,args)`).
  **Elaboration STAGED → M-673**: generics type-check but **do not yet RUN** (explicit `Residual` placeholder).
- **Bounded generics** (`fn f<T: Cmp>(…)`, `fn f<T: Cmp + Eq>(…)`) — self-bound sugar `T: Cmp ≡ T: Cmp<T>`;
  a bound on a `type`/`trait` param that isn't a trait name is an explicit refusal (G2). LANDED M-659.
- **Traits + impls** (`trait T<A> { fn … }`, `impl T<X> for Y { … }`) — coherence = global uniqueness per
  `(trait, type-head)` + single-nodule orphan rule; exact method-set conformance; bounded-call + unqualified
  trait-method resolution. All refusals explicit. **Dictionary-passing L0 lowering STAGED → M-673** (traits
  type-check but do NOT yet RUN). LANDED M-659.
- **Effect annotations** (`fn f(…) -> T !{retry, alloc}`) — effect names: kernel kinds
  `retry|alloc|io|cascade|time` + user `Named`; absent ⇒ pure; duplicate effect = never-silent parse
  refusal. Checker `check_effect_coverage`: declared ⊇ performed; performed = union of every callee's
  declared effects over fn bodies AND impl-method bodies; under-declaration = explicit `CheckError`,
  over-declaration OK. **No new L0 node (KC-3)** — effects are **checker-only metadata**, do NOT lower or
  run. Guarantee `Declared`. LANDED M-660.

**Literals are representation-typed and universal-until-elaboration** — no defaulting across
representation families (Q6 resolved):

- Binary: `0b1011_0010`
- Balanced ternary: `<+0--0>` (MSB-first over `{+,0,-}`)
- Decimal integer: bare `42` (unresolved until ambient or explicit type supplies a paradigm)
- List: `[1.5, -2.25]`

**Guarantee annotation** at the type level: `Ternary{6} @ Exact` (LR-6; `@` is a reserved
operator in the grammar).

**Singular/plural forms** (prose only — plurals are never reserved identifiers):

| Singular (reserved) | Plural (prose) |
|---|---|
| `phylum` | `phyla` |
| `hypha` | `hyphae` |
| `nodule` | `nodules` |
| `colony` | `colonies` |

---

## Key invariants (honesty)

- **Naming law (DN-02 §1):** theme only where the metaphor is accurate and illuminating; keep
  conventional where borrowed terms are clearer for human and machine readers. The three-test gate
  (T-map / T-illuminate / T-learn) is mandatory; no term bypasses it.
- **One name per term (DN-03 §3):** no canonical/alias pairs, no per-audience projection. Pick
  the single clearest name and stop.
- **Never-silent reservation (G2):** reserved-not-active words lex as keywords — they can never
  be silent identifiers. A reserved word used as a name is a parse error with a diagnostic.
- **Never-silent swap (S1/WF2):** `swap` target and policy are always lexically written. No
  inference or sugar may insert a `Swap`.
- **Append-only decisions:** a term's ratification is in the relevant DN/RFC. To change a
  ratified term, supersede in a new note — never rewrite history.
- **Content-addressed identity is over elaborated L0** (ADR-003), never the surface keyword. A
  rename that does not change the elaborated L0 term is not a new identity.
- **Guarantee tags (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`):** every accuracy claim is tagged.
  Downgrade to stay honest; never upgrade without a checked basis (VR-5). `Proven` requires a
  theorem with *checked* side-conditions. `Declared` is always flagged.
- **Honesty of this file:** the reserved-word table above is verified against
  `crates/mycelium-l1/src/token.rs` (the `keyword()` function + `Tok` enum). Confirmed: `keyword()`
  reserves only the **Active** and **Reserved-not-active** rows — `nodule`, `phylum`, `colony`, and
  the L1/L2/type/scalar/strength words. The Runtime-tier words (`hypha`, `fuse`, `mesh`, …) **and**
  `consume`/`grow`/`impl` are **not** in `keyword()`, so they currently lex as ordinary identifiers
  and are marked **Ratified — not yet lexed** (their lexer reservation lags the DN-03 spec; using one
  as an identifier is not yet an error). Source is ground truth — re-verify against `token.rs` after
  any lexer change.

---

## Read-more entry points

- `docs/notes/DN-02-Fungal-Lexicon-and-Reserved-Words.md` — naming LAW, three-test gate, original lexicon
- `docs/notes/DN-03-Lexicon-Amendment-Surface-and-Runtime-Forms.md` — runtime names, one-name-per-term
- `docs/notes/DN-06-Static-Organization-and-Dynamic-Grouping-Lexicon.md` — phylum/nodule/colony; the `// nodule:` header
- `docs/Glossary.md` — per-term definitions with normative citations
- `docs/spec/grammar/mycelium.ebnf` — the normative grammar oracle
- `docs/spec/grammar/README.md` — conformance corpus layout and checking
- `docs/spec/Nodule-Header-and-Project-Manifest.md` — header schema + `mycelium-proj.toml`
- `crates/mycelium-l1/src/token.rs` — the actual `keyword()` function (source-of-truth for what the lexer reserves today)
- `crates/mycelium-l1/src/lib.rs` — parser tests; the `phylum_and_colony_are_reserved_not_active` test
- `docs/rfcs/RFC-0006-Surface-Language-and-Term-Layering.md` — layer cake; invariants S1–S6; grammar discipline
- `docs/rfcs/RFC-0007-L1-Kernel-Calculus.md` — ten-node budget; typing; totality gate
- `docs/notes/Example-Programs-Reference.md` — code examples (read §Grounding notes — pre-ratification names used)

---

## Gotchas

- **`phylum` and `colony` are reserved-not-active.** They lex as keywords so using them as
  identifiers is a parse error — but no construct consumes them yet. Neither opens a program.
  `docs/spec/grammar/conformance/reject/10-reserved-not-active.myc` tests this.
- **`colony` was reassigned (DN-06).** DN-02 §2 originally gave `colony` the static "module"
  meaning; DN-06 (2026-06-16) superseded that and reassigned `colony` to the dynamic runtime
  grouping of `hypha`. The static role moved to `nodule`. The M-358 migration completed this in
  code. Do not use `colony` to mean "module."
- **`matured fn` at item position is a parse error.** `matured` is declared at scope (nodule/phylum
  header `// @matured: true` or `mycelium-proj.toml`), not per function. The parser emits a
  teaching diagnostic pointing at `// @matured: true` and `thaw fn` (RFC-0017 §4.1).
- **Runtime words (`hypha`, `fuse`, `mesh`, etc.) are ratified names but not yet active syntax.**
  They have one ratified name each (DN-03 §4 — the source of truth). The Example Programs
  Reference (§Grounding notes) lists the old draft spellings that appear in some examples and have
  since been superseded: `anas` → `fuse`, `sclrt`/`Sclerotium` → `cyst`, `myco` → `graft`,
  `cmn` → `mesh`, `rhizo`/`rhizomorph` → `backbone`, `dimorph` → `tier`.
- **`reclaim` reclaims *runtime units*, never memory.** LR-9 makes memory reclamation automatic;
  a memory-`reclaim` would contradict it (DN-03 §4 scope clarification).
- **`thaw` is conventional-clearest, not themed.** The intuitive inverse `germinate` is taken by
  spore-activation/deployment (ADR-013). DN-03 changelog records this (2026-06-18).
- **The `// nodule:` marker is not grammar** — it is a comment (lexer trivia) recognised by
  `mycelium_l1::parse_nodule_header`. It is never part of content-addressed identity.
- **Example Programs Reference is partially pre-ratification.** Examples #1–#7 use syntax
  (`embody`, `hyph`, `anas`, `sclrt`, `rhizo`) that has been superseded. Read the Grounding notes
  at the end of that file. Examples #8 (matured scope) and #18 (multi-repr pipeline) are the most
  current.
- **The grammar is W3C EBNF, not ISO 14977.** Notation: `A?` optional; `A*` zero-or-more; `A+`
  one-or-more; `A | B` alternation; `(A B)` grouping; `/* */` comments (RFC-0006 §4.3 / T3.1-B).
- **`tier` is an execution-mode switch, not a representation change.** Interpreted ↔ native is a
  `tier`. Dense ↔ sparse is a `swap` (S1). These are distinct operations (RFC-0008 §4.5 / DN-03 §4).
- **Generics and traits type-check but do NOT yet run (elaboration STAGED → M-673).** `Ty::Var` unification
  and trait coherence are implemented (M-656/M-657/M-659); the monomorphization + dictionary-passing L0
  lowering is an explicit `Residual` placeholder pending M-673. Do not claim generics or traits execute.
- **Effect annotations are checker-only (no L0 lowering).** `fn f() -> T !{alloc}` is parsed and
  coverage-checked (M-660); the `!{…}` annotation does NOT add an L0 node and does NOT wire to the
  interpreter budget yet (that is M-677). Never state that effects "run" or "enforce at runtime" —
  the guarantee is `Declared`.
