# RFC-0037 — Surface Grammar Deconfliction and Layout Independence

| Field | Value |
|---|---|
| **RFC** | 0037 |
| **Status** | **Enacted** (2026-06-27) — *Proposed → Accepted → **Enacted**: the grammar epic landed in `crates/mycelium-l1` + `mycelium-fmt` (full corpus migrated; `mycelium-l1` 615 + `mycelium-fmt` 11 green; `mycelium.ebnf`/editor-grammars/api-index regenerated). **Remaining follow-ons** (not blockers): D2-b short repr keywords, RFC-0025 operator wiring. Prior chain — ratified by the maintainer 2026-06-27 (R1 binding-grammar gate); enactment = the grammar epic in `crates/mycelium-l1` (lexer/parser/`mycelium.ebnf`) — moves to **Enacted** when that lands; FLAG-2 (delimiter rule, rec. Option B) and FLAG-3 (short-keyword status) resolved at epic time. Prior status preserved (append-only):* **Proposed** (2026-06-27) — the bracket kind-split, operator reallocation, return-arrow change, trit-literal reprefix, and layout-independence principle are normative; enacts no code. |
| **Type** | Normative / foundational (once Accepted) — surface grammar supersession; no L0 or L1 kernel change |
| **Date** | 2026-06-27 |
| **Task** | M-809 |
| **Feeds** | E11-1 (surface-language completeness) · the grammar-supersession epic (lexer/parser, tree-sitter grammars, conformance corpus, examples, docs) |
| **Decides** | The committed, collision-free bracket-family allocation; const/width vs type parameter declaration syntax; the layout-independence principle and its consequence for the list-literal-vs-type-app edge; the return-arrow glyph; trit-literal prefix; `lambda` keyword for closures; short paradigm-type keywords; what this supersedes |
| **Depends on** | DN-31 (the decided scheme — primary source; esp. the 2026-06-27 kind-split + layout-independence revision entries); RFC-0019 §4.1 (Enacted type-param brackets — this RFC supersedes that sub-section); RFC-0025 §4.3 (M-745 deferred angle-bracket operators — resolved here); RFC-0030 (the concrete L3 grammar — updated here); DN-51 §2-D4 (per-instance width-generic guarantee tags); DN-02/DN-03 (lexicon and keyword set — `lambda` + short keywords amend this) |
| **Coupled with** | `docs/spec/grammar/mycelium.ebnf` (the EBNF to be updated per §6); `crates/mycelium-l1/src/` (lexer, parser, checker); `crates/mycelium-l1/src/token.rs` (`Tok::Binary`/`Ternary`/`Dense`/`Vsa`; `lambda` reserved word); tree-sitter grammar (`just grammar-gen`); `docs/spec/grammar/conformance/` (corpus fixtures); `lib/std/` (examples) |

> **Posture (transparency rule / VR-5 / G2).** This RFC **decides** the surface grammar
> realignment normatively. It **enacts no code** and moves no other decision's status in place.
> The bracket-family allocation, the layout-independence principle, and every design claim below
> are tagged `Declared` — no claim is `Proven`; mechanization is the basis for a future `Proven`
> upgrade. The supersessions recorded in §5 are append-only (house rule #3): the superseded
> sub-section texts are **not** rewritten; this RFC carries the forward references. Where open
> sub-details remain (§4.3, §4.4, §7), they are named, not buried (G2).

---

## 1. Problem / Goal

DN-31 (Draft, advisory) measured the load on the four bracket families and found `<>` triple-loaded:

| Delimiter | Roles before this RFC | Verdict |
|---|---|---|
| `<>` | type params/args (`List<T>`, `fn f<N>`) · trit literals (`<+-0>`) · comparison/shift operators (`< > << >>`) — but the last group is **blocked** (M-745: `a < b` cannot be told from `f<T>(x)` without contextual lexing) | **triple-loaded — M-745 unresolvable** |
| `()` | calls, grouping, tuples, constructors | heavily used, clean |
| `{}` | blocks, match bodies, effects `!{…}`, repr descriptors `Binary{8}` | loaded, positionally distinct |
| `[]` | list literals only | **near-empty — spare capacity** |

The overload is structural: M-745 (comparison and shift operators, RFC-0025 §4.3) cannot be resolved
without removing one of the `<>` roles first. The fix is **reallocation off `<>` onto `[]`**, not
disambiguation machinery.

RFC-0025 §4.3 deferred `< <= > >= << >>` explicitly pending this resolution. RFC-0030's move
to Proposed is gated on M-745 landing (RFC-0030 §4.3). This RFC is that landing: it reallocates the
bracket families, resolves M-745 by reallocation, and fixes the layout-independence principle that
settles the remaining list-literal-vs-type-application edge — all normatively, as a single
coherent supersession wave.

---

## 2. User stories

- As a **language user** writing numeric code, I want `a < b` and `a << b` to work as infix
  operators without fighting type-parameter syntax, so that comparison-heavy and bit-manipulation
  code does not require verbose word-call workarounds.
- As a **compiler engineer** maintaining `mycelium-l1`, I want a collision-free bracket allocation
  with explicit rules for every ambiguous edge, so that the lexer and parser require no contextual
  state to distinguish type application from operator use.
- As a **stdlib author** writing width-generic functions, I want `f[T]{N}` to distinguish type
  parameters from const/width parameters by kind — `[T]` = type, `{N}` = const/width — so that the
  distinction the type system already makes (DN-42) is visible at the surface.
- As a **tool author** writing a formatter or syntax highlighter, I want newlines to be
  semantically inert, so that a formatter can reflow a program without changing its parse tree.
- As a **language user** writing lambda expressions, I want an explicit `lambda` keyword, so that
  a chain of lambdas in a dense stream is legible and unambiguous.
- As a **downstream app developer** targeting the Mycelium AOT compiler, I want the repr-keyword
  short forms `bin{N}`/`tern{N}`/`emb{…}`/`hvec{…}` for ergonomic width-annotated type
  literals, so that I do not write `Binary{8}` in every declaration while still keeping the
  const bracket `{}` consistent.
- As a **maintainer**, I want a single normative RFC that is the reference for the
  grammar-supersession epic, so that M-809 and its implementation children cite one document.

---

## 3. Scope and decision space

### In scope

- The complete **bracket kind-split allocation** (`[]`, `{}`, `<>`, and closing `->` / `=>`).
- The **const/width parameter declaration** syntax (`f{N}`, `f[T]`, `f[T]{N}`).
- The **layout-independence principle** and its consequence for list-literal-vs-type-app
  disambiguation.
- The **return-arrow glyph** change (`->` retired, `=>` used for return type).
- The **trit-literal prefix** change (`<+-0>` retired, `0t+-0` adopted).
- The **`lambda` keyword** for closures (reserved word).
- The **short paradigm-type keywords** (`bin{N}`, `tern{N}`, `emb{…}`, `hvec{…}`).
- **EBNF-level production sketches** (§6) for the changed productions.
- **Supersession records** (§5) — what this supersedes and what it leaves intact.
- **Migration scope** (§8) — which implementation artifacts move.

### Out of scope

- L0 or L1 kernel changes — this is grammar and surface only; no `mycelium-core` or
  `mycelium-interp` change (KC-3).
- The full updated `docs/spec/grammar/mycelium.ebnf` text — that is the implementation artifact
  produced by the grammar-supersession epic, not this RFC.
- The closures / higher-order semantics — `lambda` syntax is reserved here; full HOF semantics
  are RFC-0024's concern and M-704's implementation target.
- Width arithmetic promotion and narrowing semantics — those are DN-51; this RFC is surface syntax
  only.
- Operator overloading or user-defined precedence — deferred per RFC-0025.

---

## 4. Decisions (D1–D6) — normative as proposed

### D1 — Bracket kind-split allocation

The four bracket families are realigned by **semantic kind**, not by availability:

```
[T]   →  type parameters / type arguments + list literals
{N}   →  const / width parameters  AND  repr/size types  (unchanged surface)
<>    →  comparison and shift operators ONLY: <  >  <<  >>
()    →  calls · grouping · tuples · constructors  (unchanged)
```

**Repr/size types stay on `{}`** (`Binary{8}` is unchanged; `Ternary{N}`, `Dense{D,S}`,
`VSA{E,D,Sp}` all remain). This is the bracket-by-kind principle: `{}` already carries const/width
(size descriptors are const-generic width values, DN-42); keeping them there avoids a corpus
migration of the most pervasive surface form. `[T]` absorbs type-parameter brackets, replacing
`<T>` in every type parameter list and type argument list.

**`<=` and `>=` are retired as glyphs.** The word-operator aliases `lte` and `gte` are their
replacements. The asymmetry — keeping `<` and `>` as glyph operators while retiring `<=` and `>=`
— is intentional and grounded: the two-character glyphs are precisely the ones that produce the
most severe type-arg parsing ambiguity in a scanner (a `<=` following a type argument could be
misread as the end of a type-arg list followed by `=`). Single-character `<` and `>` are
unambiguous at the glyph level once they are operators-only. Word forms `lt(a,b)` / `lte(a,b)` /
`gt(a,b)` / `gte(a,b)` remain valid everywhere and are the word-canonical desugaring targets
(RFC-0025 §4.2 — add `lte`/`gte` to the desugaring map; existing `lt`/`gt` entries unchanged).

**Consequence — M-745 resolved.** With `<>` carrying no type-argument role, `a < b` is
unambiguously a comparison expression. No contextual lexing and no speculative parsing are needed.
The RFC-0025 §4.3 deferral is closed by this reallocation.

**Guarantee tag (D1): `Declared`.** The collision-freedom claim follows from the allocation by
construction; it is not yet mechanically verified.

### D2 — Const/width parameter declaration

A function generic over a **type** parameter uses `[T]`; generic over a **const/width** parameter
uses `{N}`; generic over both uses `[T]{N}`. The bracket matches the kind of the parameter:

```mycelium
fn identity[T](x: T) -> T = x
fn zero_extend{N}(x: Binary{N}) -> Binary{N} = x
fn map_get[V]{N}(m: Map[Binary{N}, V], k: Binary{N}) -> Option[V] = ...
```

This replaces the current `fn f<T>` / `fn f<N>` form where both kinds share `<>` (as landed in
M-753 / DN-42). The migration is mechanical: `fn f<T>` becomes `fn f[T]`; `fn f<N>` becomes
`fn f{N}`; `fn f<T, N>` becomes `fn f[T]{N}`. The pervasive `Binary{N}` repr surface in **type
position** (not in a parameter declaration) does **not** change: `Binary{N}` with a concrete or
width-variable `N` is unchanged in type references, call sites, and struct fields.

**Short repr keywords (D2-b).** The paradigm type-keywords gain short ergonomic aliases that keep
their width/dims argument on `{}`:

| Short form | Long form | Meaning |
|---|---|---|
| `bin{N}` | `Binary{N}` | N-trit binary value |
| `tern{N}` | `Ternary{N}` | N-trit balanced-ternary value |
| `emb{D,S}` | `Dense{D,S}` | Dense embedding (dims D, scalar kind S) |
| `hvec{E,D,Sp}` | `VSA{E,D,Sp}` | Hypervector (element-space E, dims D, sparsity Sp) |

`vec` is **rejected** — it collides with `Vec` in `std.collections` (the cons-list alias in
`lib/std/collections.myc`). `hvec` is used instead. The short forms are syntactic aliases only;
they elaborate identically to their long forms. The lexer must reconcile `bin`/`tern`/`emb`/`hvec`
with `crates/mycelium-l1/src/token.rs`; see §5 (lexicon amendment) and §7 FLAG-3.

**Guarantee tag (D2): `Declared`.** Short-form aliasing is a definitional claim; elaboration
identity is `Declared` pending implementation.

### D3 — Layout-independent grammar (load-bearing)

Newlines are **formatting-only, never semantically required**. The same program parses identically
whether written as a dense character stream or with line breaks for human readability. Structural
delineation is by:

1. **Explicit delimiters** — the maintainer's `,`-delineation principle: syntactic sub-parts are
   delimited by commas (`,`), brackets, or keywords, not by whitespace or newline position.
2. **Type-position vs value-position** — whether a bracketed form appears where a type is expected
   (type application) or where a value is expected (list literal).

**Consequence for list-literal-vs-type-app.** The earlier "newline/adjacency" disambiguation
rule (DN-31 §4-Q2, tentative) is **withdrawn** — newlines must not be load-bearing. Instead, the
disambiguation is structural and newline-free:

- `Name[…]` in **type position** is a type application (`List[T]`, `Option[Binary{N}]`).
- `[…]` in **value position** (standalone, or following a delimiter or in an argument slot) is a
  list literal (`Seq{T, N}`).
- These cases are mutually exclusive because Mycelium has **no `arr[i]` indexing** (indexing is
  `get(seq, i)`) and **no juxtaposition application** (all calls use `()` — `f(x)`, never `f x`).
  Therefore `Name[…]` has exactly one reading (type application), and `[…]` in value position has
  exactly one reading (list literal). No ambiguity survives.

The **exact delimiter set and adjacency rules** are the remaining implementation detail to be
specified during the grammar-supersession epic. The principle (layout-independent, explicit-delimiter,
stream-legal) is fixed by this RFC; the fine grain is an open sub-detail in §7 (FLAG-2).

**Guarantee tag (D3): `Declared`.** The no-ambiguity claim follows from the structural argument
(`Declared`-with-argument); mechanization would raise it to `Proven`.

### D4 — Return arrow and trit-literal prefix

**Return arrow.** The fn-signature return arrow `->` is retired and replaced by `=>`. Rationale:
freeing `-` and `>` as independent operator tokens eliminates the need for the lexer to speculatively
consume `->` before deciding whether to emit it as the return arrow or two separate tokens. `=>`
is already used in match arms; making it the return-type arrow as well is a minimal surface change.
Current `fn_sig` and `fn_item` in `mycelium.ebnf` use `'->'`; those productions change to `'=>'`
(see §6). The for-expression `=>` already used in `for_expr` is unchanged.

**Trit literals.** The `<…>` trit literal form (`<+-0>`, `TritLit ::= '<' Trit+ '>'`) is
retired. The replacement is a **`0t` prefix**, analogous to `0b` (binary) and `0x` (bytes):
`0t+-0`. This frees `<>` for exclusive use as comparison/shift operators and makes trit literals
visually consistent with binary (`0b…`) and hex (`0x…`) prefixes. The lexer's `TritLit` terminal
changes; the `literal` production gains `TritLit` in the `0t…` form.

**Guarantee tag (D4): `Declared`.**

### D5 — `lambda` keyword for closures

Closures are introduced with the explicit keyword `lambda`. The keyword makes a lambda-chain in a
dense stream legible and unambiguous:

```mycelium
lambda (x: bin{8}) => add(x, x)
lambda [T](x: T, y: T) => pair(x, y)
```

`lambda` is a new **reserved word** to be added to DN-02/DN-03 (the lexicon and keyword set) and
to `crates/mycelium-l1/src/token.rs`. It is input to M-704 (full higher-order functions / closures
design); this RFC reserves the keyword and specifies its role in the grammar without committing the
full closure semantics (which belong to M-704 and RFC-0024's successor). The `lambda` body uses
`=>` (the return arrow, D4) as the separator between parameter list and body, consistent with fn
signatures.

**Guarantee tag (D5): `Declared`.** The full closure semantics are not specified here; the keyword
reservation is a definitional commitment.

### D6 — Per-instance width-generic guarantee model (cross-reference, not new decision)

DN-51 §2-D4 specifies that per-instance guarantee tags — not a uniform tag on the generic source
function — are the model for width-generic code. This RFC references that decision because it
affects how grammar productions involving `{N}` parameters are annotated: a `fn f{N}(…) => …`
carries no guarantee tag at the source; its monomorphized instances each carry their own tag.
This is a **cross-reference only** (the decision is DN-51's; no new commitment here).

---

## 5. Supersessions and touches (append-only)

This RFC records the supersessions normatively. It does **not** edit the superseded texts;
the orchestrator adds the corresponding back-references to those documents per the append-only rule.

### S1 — RFC-0019 §4.1 (Enacted) — type-parameter bracket syntax

RFC-0019 §4.1 committed `<…>` for `type_params` and `type_args`
(`type_params ::= '<' Ident … '>'`, `type_args ::= '<' type_ref … '>'`). This RFC **supersedes**
that sub-section: `[…]` replaces `<…>` in all type-parameter and type-argument positions. The
remainder of RFC-0019 (trait declarations, bounded generics, instance resolution, coherence,
LR-5/LR-6) is unchanged and remains Enacted.

FLAG to orchestrator: add an append-only "superseded by RFC-0037 §5-S1 (type-param brackets
`<…>` → `[…]`)" note to RFC-0019 §4.1.

### S2 — RFC-0025 §4.3 (Proposed) — deferred angle-bracket operators (M-745)

RFC-0025 §4.3 deferred `< <= > >= << >>` pending type-arg disambiguation. This RFC **resolves**
M-745: `<>` is operators-only (D1), so the deferral condition no longer holds. The desugaring
map (RFC-0025 §4.2) gains `<` → `lt`, `>` → `gt`, `<<` → `shl`, `>>` → `shr` as bindings;
`<=` and `>=` are retired as glyphs (word aliases `lte`/`gte` only). The operator grammar gains
a comparison tier between the equality tier and the bitwise-or tier (see §6). RFC-0025's
Proposed status is unchanged (it awaits maintainer ratification independently); this RFC's
ratification is a prerequisite for RFC-0025 reaching Accepted (the M-745 gate).

FLAG to orchestrator: add an append-only "M-745 resolved by RFC-0037 §5-S2; desugaring map
extended to include `<`/`>`/`<<`/`>>`; `<=`/`>=` retired" note to RFC-0025 §4.3.

### S3 — RFC-0030 (Draft) — concrete L3 grammar

RFC-0030 §4.3 gates move-to-Proposed on M-745 landing. This RFC is that landing. After
ratification of RFC-0037 and the grammar-supersession implementation epic, RFC-0030 can proceed
to Proposed with the updated `mycelium.ebnf` (incorporating all D1–D5 changes). The §4.3 gate
condition ("M-745 — the angle-bracket operators so the operator grammar is whole") is satisfied.

FLAG to orchestrator: add an append-only "M-745 gate satisfied by RFC-0037; RFC-0030 may now
proceed to Proposed once the grammar-supersession epic lands" note to RFC-0030 §4.3.

### S4 — DN-31 (Draft, advisory) — the decided scheme

DN-31 remains Draft (advisory; it enacts nothing by design). This RFC is the **binding enactment**
of DN-31's decided scheme. After ratification, DN-31's §5 (Definition of Done) is satisfied:
"a binding RFC ratifies §2." DN-31 itself does not change status — it is superseded-as-advisory
by this normative RFC.

### What this does NOT touch

- RFC-0001 / RFC-0033 — the value model and repr semantics are unchanged. `Binary{8}` stays
  `Binary{8}` in type-reference position; only the parameter-declaration bracket changes.
- RFC-0024 — HOF semantics; `lambda` syntax is reserved here, semantics there.
- `crates/mycelium-core` / `crates/mycelium-interp` — no kernel change (KC-3).
- The guarantee lattice or guarantee-tag machinery — unchanged.

---

## 6. EBNF production changes (illustrative sketch)

These are the **changed or new productions** relative to `docs/spec/grammar/mycelium.ebnf`.
They are illustrative; the normative EBNF artifact is updated by the grammar-supersession
implementation epic. All other productions are unchanged.

```ebnf
(* D1/D2: type_params and type_args move from <…> to […].
 * const_params are the new {…} parameter declarations — was folded into
 * type_params as <N> in the M-753/DN-42 era. *)
type_params    ::= '[' type_param (',' type_param)* ']'
type_param     ::= Ident (':' bound)?
type_args      ::= '[' type_ref (',' type_ref)* ']'
const_params   ::= '{' Ident (',' Ident)* '}'       (* const/width param declarations *)

(* Function item: type_params? const_params? — [T]{N} or [T] or {N} or neither.
 * Return arrow -> becomes =>. *)
fn_item        ::= 'pub'? 'thaw'? 'fn' Ident type_params? const_params?
                   '(' params? ')' '=>' type_ref effects? '=' expr

(* Trait item: type params only (traits are type-level, not const-level in v1). *)
trait_item     ::= 'pub'? 'trait' Ident type_params? '{' fn_sig* '}'
fn_sig         ::= 'fn' Ident type_params? const_params? '(' params? ')' '=>' type_ref effects?

(* Type item: type params only. *)
type_item      ::= 'pub'? 'type' Ident type_params? '=' constructor ('|' constructor)*

(* D1: base_type — repr descriptors stay on {}, type args move to [].
 * D2-b: short aliases bin/tern/emb/hvec added.
 * Note: Seq now uses [...] for its type arg (kind correction, see §6 prose). *)
base_type      ::= 'Binary'   '{' Int '}'
                 | 'Ternary'  '{' Int '}'
                 | 'Dense'    '{' Int ',' scalar '}'
                 | 'VSA'      '{' Ident ',' Int ',' sparsity '}'
                 | 'bin'      '{' Int '}'                          (* D2-b short alias *)
                 | 'tern'     '{' Int '}'                          (* D2-b short alias *)
                 | 'emb'      '{' Int ',' scalar '}'               (* D2-b short alias *)
                 | 'hvec'     '{' Ident ',' Int ',' sparsity '}'   (* D2-b short alias *)
                 | 'Substrate' '{' Ident '}'
                 | 'Seq'      '[' type_ref ',' Int ']'             (* {} -> [] for type arg *)
                 | 'Bytes'
                 | Ident type_args?
                 | '{' ambient_params '}'

(* D4: trit literal prefix changes from <…> to 0t…. *)
TritLit        ::= '0t' Trit+
Trit           ::= '+' | '0' | '-'

(* D5: lambda expression (keyword reserved; full semantics = M-704).
 * Added to the expr alternatives alongside let_expr, if_expr, etc. *)
lambda_expr    ::= 'lambda' type_params? const_params? '(' params? ')' '=>' expr

(* D1/S2: operator grammar gains comparison and shift tiers (resolves M-745).
 * Inserted between eq_expr and bor_expr; lte/gte are word-only, not glyph operators. *)
eq_expr        ::= cmp_expr (('==' | '!=') cmp_expr)*
cmp_expr       ::= shift_expr (('<' | '>') shift_expr)*
shift_expr     ::= bor_expr (('<<' | '>>') bor_expr)*

(* impl_item: type_args use new [] form. *)
impl_item      ::= 'impl' Ident type_args? 'for' type_ref '{' impl_method* '}'
```

**Note on `Seq` type syntax.** The current `Seq '{' type_ref ',' Int '}'` form uses `{}` because
it predates this RFC. Under the bracket kind-split (D1), `{}` in `Seq{T,N}` is semantically wrong:
`T` is a type argument, not a const/width argument. The production above corrects this to
`Seq '[' type_ref ',' Int ']'` — the list element type `T` is a type arg (square brackets), and `N`
is a size value also in square brackets as a literal integer (not a const-parameter binding). This
is a minor correction to the RFC-0032 D3 surface that the kind-split makes visible; the semantics
are unchanged.

---

## 7. Open sub-details (FLAGGED — not resolved here)

**FLAG-2 — Exact delimiter set and adjacency rules.** The layout-independence principle (D3) is
fixed: newlines are formatting-only, structural delineation is by explicit delimiters and
type-vs-value position. The remaining specification work is the **exact delimiter set and any
adjacency rules** for the grammar-supersession implementation. Three options exist on the spectrum:

- **Option A (mandatory `,` everywhere).** Every list of items — parameter lists, type argument
  lists, expression sequences — requires a `,` between elements with no exceptions.
  Maximally explicit; zero adjacency ambiguity; slightly verbose for single-element forms
  (though `f[T](x: T)` is unaffected since `(x: T)` already has explicit brackets).
- **Option B (`,` required between same-kind siblings; optional trailing).** Commas are required
  between items of the same syntactic kind (e.g., between two type arguments `[T, U]`, between
  two parameters `(x: T, y: U)`) but trailing commas are optional (as currently). Structurally
  equivalent to Option A for parsing; slightly more forgiving to write.
- **Option C (`,` required only where ambiguous without it).** Commas are omitted in positions
  where the bracket structure alone makes the boundary unambiguous (e.g., a single-argument
  type app `List[T]` needs no comma; a two-argument `Map[K, V]` does). Reduces syntactic noise
  in common single-argument forms but requires the spec to enumerate the unambiguous positions.

**Recommendation (Declared): Option B** — it is the least surprising to a programmer familiar with
Rust/ML-family languages, it matches the current grammar's conventions for parameter lists and match
arms (trailing comma tolerated), and it leaves no position where adjacency alone must be the
delimiter. Decide at grammar-supersession epic time; do not over-commit in this RFC.

**FLAG-3 — Short keyword lexer status.** `bin`/`tern`/`emb`/`hvec` are proposed as short repr
aliases (D2-b). An open question is whether they should be **full reserved keywords** (like
`Binary`/`Ternary`/`Dense`/`VSA`) or **soft keywords** (lexed as identifiers, resolved to the
short-form repr type only in type position). The reserved-keyword path avoids user confusion (a
user cannot shadow `bin` as an identifier); the soft-keyword path is more conservative and avoids
growing the reserved set. Decide at implementation; reconcile with `crates/mycelium-l1/src/token.rs`
and DN-02/DN-03.

**FLAG-4 — `lambda` full surface.** This RFC reserves the keyword and gives an illustrative
production (D5). The full closure semantics — capture rules, type inference for closures, HOF
interaction — belong to M-704 and RFC-0024's successor. Do not implement closure semantics from
this RFC alone.

---

## 8. Migration scope

The grammar-supersession epic (gated on this RFC's ratification) spans:

1. **Lexer/parser** (`crates/mycelium-l1/src/`): update `token.rs` to add `lambda`, retire the
   `<…>` type-arg lexer path, add `0t…` TritLit, update `=>` as return arrow. Parser productions
   follow §6.
2. **Type checker and elaborator** (`crates/mycelium-l1/src/checkty.rs`, `elaborate.rs`): update
   type-param and const-param handling to match the D2 bracket split.
3. **Tree-sitter / editor grammars** (`just grammar-gen`, RFC-0026): regenerate from the updated
   EBNF.
4. **Conformance corpus** (`docs/spec/grammar/conformance/`): update all accept/reject fixtures
   that use `<T>` type args, `<+-0>` trit literals, or `->` return arrows. Add new fixtures for
   `[T]`, `{N}`, `0t…`, `lambda`, and `<`/`>`/`<<`/`>>` as operators.
5. **Examples and docs** (`examples/`, `lib/std/`, every RFC/ADR/DN that shows surface syntax):
   mechanical replacement of `<T>` with `[T]`, `<+-0>` with `0t+-0`, and `->` with `=>` in
   surface-syntax contexts. **`Binary{8}` / `Ternary{N}` / repr `{}` forms are NOT changed** (D1).
6. **`docs/spec/grammar/mycelium.ebnf`**: updated to incorporate all D1–D5 changes, resolving
   RFC-0030's M-745 gate condition (§5-S3).

This migration is wide but mechanical. It is cheap now (design phase, minimal ecosystem);
sequencing it as one coordinated wave (DN-31 §4-Q4) avoids incremental confusion about which
form is current.

---

## 9. Grounding

- **DN-31** (Draft, 2026-06-27 kind-split + layout-independence revision) — the primary source for
  D1–D5. The `Declared` grounding strength reflects DN-31's own status: a decided direction, not a
  mechanically checked result.
- **RFC-0019 §4.1** (Enacted) — the sub-section this RFC supersedes (S1). The remainder of
  RFC-0019 is unaffected.
- **RFC-0025 §4.3 / M-745** (Proposed) — the deferred angle-bracket operators, resolved by D1/S2.
  The desugaring map extension (`lt`/`gt`/`shl`/`shr`; retired `lte`/`gte` glyphs) is additive
  to RFC-0025 §4.2.
- **RFC-0030** (Draft) — the concrete grammar; S3 records the gate condition being met.
- **DN-51 §2-D4** — per-instance width-generic guarantee tags; cross-referenced in D6 (not a new
  commitment here).
- **DN-42** (Accepted) — width-generics (`fn f<N>`); the const-param bracket change (D2) is the
  surface amendment to DN-42's landed `fn f<N>` form.
- **`docs/spec/grammar/mycelium.ebnf`** (checked 2026-06-27) — the measured current allocation
  (triple-loaded `<>`, near-empty `[]`); ground truth for the problem statement in §1.
- **DN-02/DN-03** (Resolved) — the lexicon and keyword set; `lambda` + short keywords amend these
  (FLAG-3 and FLAG-4 for the reconciliation).
- House rules: **VR-5** (per-op `Declared` tags; nothing upgraded); **G2** (never-silent — every
  new form must have an explicit refusal for malformed input, not a silent fallback); **KC-3** (no
  kernel change); **house rule #3** (append-only — supersessions are forward-references, not
  in-place rewrites).

---

## 10. Definition of Done

This RFC reaches **Accepted** when:

- [ ] The maintainer ratifies D1–D5 (the bracket kind-split, const/width param syntax,
  layout-independence principle, return arrow, trit prefix, `lambda` keyword).
- [ ] FLAG-2 (delimiter set and adjacency rules) is resolved with a specific option (A/B/C).
- [ ] FLAG-3 (short keyword lexer status) is decided (reserved keyword vs soft keyword).

The grammar-supersession **epic (implementation)** is complete when:

- [ ] `docs/spec/grammar/mycelium.ebnf` is updated to incorporate D1–D5 (§6) and the RFC-0030
  M-745 gate condition is satisfied.
- [ ] `crates/mycelium-l1` lexer/parser/checker are updated; `just check` green.
- [ ] Tree-sitter / editor grammars regenerated (`just grammar-gen`).
- [ ] Conformance corpus updated with new accept/reject fixtures for every changed production.
- [ ] Examples, docs, and `lib/std/` migrated (mechanical; `Binary{8}` unchanged).
- [ ] RFC-0030 moves Draft → Proposed (gated on this epic completion + M-707).
- [ ] RFC-0025 moves Proposed → Accepted (gated on this epic's M-745 closure).
- [ ] Append-only back-references added to RFC-0019 §4.1, RFC-0025 §4.3, and RFC-0030 §4.3.

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-27 | **Proposed** | Initial RFC. D1–D5 proposed (kind-split, const/width params, layout-independence, return arrow, trit prefix, `lambda`); supersessions S1–S4 recorded; EBNF sketch in §6; open sub-details in §7 (FLAG-2 delimiter set, FLAG-3 short keyword lexer status, FLAG-4 lambda semantics). Task: M-809. |
