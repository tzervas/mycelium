# Design Note DN-31 — Delimiter & Operator Deconfliction (`<> () [] {}`, return arrow, trit literals)

| Field | Value |
|---|---|
| **Note** | DN-31 |
| **Status** | **Draft** (2026-06-24; direction capture — advisory, non-committal) |
| **Feeds** | a binding **supersession of RFC-0019 §4.1** (type-parameter bracket) + an update to **RFC-0030** (concrete L3 grammar) + **RFC-0025** (resolves the deferred angle-bracket operators, **M-745**); touches RFC-0001/RFC-0033 (repr-descriptor surface) and the lexer/parser/conformance corpus. |
| **Date** | June 24, 2026 |
| **Decides** | *Nothing normatively* — advisory. Records the maintainer's decided delimiter/operator scheme (2026-06-24): free the **triple-loaded `<>`** to comparison/shift operators only, move type/size arguments onto the **near-empty `[]`**, and reconcile the four bracket families + the return arrow + trit literals into one collision-free allocation. The binding change is a future RFC/supersession; this note captures the scheme + the grounding + the open questions. |
| **Task** | surface syntax (pre-RFC capture); resolves M-745 |

> **Posture (transparency rule / VR-5).** Advisory. **Enacts nothing; moves no status; rewrites no Enacted
> decision.** RFC-0019 §4.1 (Enacted) committed `<…>` for type parameters; this note proposes to **supersede**
> that (the append-only way), not edit it. No claim here is upgraded past `Declared` until the binding RFC lands.

## §1 The problem — measured, not asserted

A grep of the normative grammar (`docs/spec/grammar/mycelium.ebnf`) shows the load is badly imbalanced:

| Delimiter | Current roles (grounded) | Verdict |
|---|---|---|
| **`<>`** | type params (L109) · type args (L147) · **trit literals** `<+-0>` (L252) · **and** the *wanted* comparison/shift operators `< > <= >= << >>` (deferred by RFC-0025 §4.3, **M-745**) | **triple-loaded → the clunkiness** |
| `()` | call/constructor args, grouping, swap/spore (L110/115/185/195/235/239) | heavily used, clean |
| `{}` | trait/match/wild/colony bodies, effects `!{…}`, **repr descriptors** `Binary{8}` (L135-139), ambient | loaded, positionally distinct |
| **`[]`** | list literals only (L248) | **near-empty → free capacity** |

The `<>` overload is *why* M-745 (the comparison/shift operators) is blocked: `a < b` cannot be told from
`f<T>(x)` without contextual lexing or speculative parsing. The fix is to **rebalance off `<>` onto `[]`**, not
to add disambiguation machinery to keep `<>` triple-booked.

## §2 The decided allocation

```
<>   comparison/shift operators ONLY:  <   >   <<   >>
       (the <= >= glyphs are RETIRED → word operators `lte` `gte`)
()   calls · grouping · tuples · constructors
[]   type arguments · sized/repr types · list literals   (List[T], Binary[64], [1,2,3])
       — disambiguated by position (type/operand position vs standalone value position)
{}   blocks · effects !{…} · match bodies · MAP literals
->   retired as the return arrow  →  =>   (bare `=` `-` `>` remain independent operators)
```

Trit literals leave `<>` for a **`0t` literal prefix** (like `0x`/`0b`): `0t+-0`. Repr/size types use `[]`:
`trit[N]`, `tryte[9]`, `Binary[64]`, `Ternary[N]` — and the pattern **extends to bytes** (`byte[N]`).

**Why this works:** `<>` drops from three roles to one (operators) — M-745 dissolves with *no* disambiguation
machinery. `[]` absorbs the type/size args it has spare capacity for; its only ambiguity (type-args vs list
literal) is a clean **position** split, tractable here *specifically because* Mycelium has **no `arr[i]`
indexing** (it's `get(seq, i)`) and **no juxtaposition application** (calls are `()`), so `name[…]` is
unambiguously type/size args and a standalone `[…]` is a list. The maintainer has accepted the position rule
("parser handles type vs value position").

## §3 What this supersedes / touches

- **RFC-0019 §4.1 (Enacted)** — `type_params`/`type_args` move `<…>` → `[…]`. *Supersede, don't rewrite.*
- **RFC-0025 §4.3 / M-745** — the deferred ordering/shift operators become *defined* (`<>` is operators-only;
  `<=`/`>=` are `lte`/`gte` words). M-745 is **resolved by this scheme**, not by speculative parsing.
- **RFC-0030** — the concrete L3 grammar is regenerated (`type_args`, `base_type` repr descriptors, `TritLit`,
  the `fn_sig`/`fn_item` return arrow, `ListLit` vs type-args).
- **RFC-0001 / RFC-0033** — repr-descriptor surface `Binary{8}` → `Binary[64]` (the `{}`→`[]` move); the value
  model's *surface* changes, not its semantics or content-hash (the descriptor's *meaning* is unchanged).
- **Lexer/parser** (`crates/mycelium-l1`) + the **conformance corpus** + the **editor grammars** (`just
  grammar-gen`) + the **doc-index**. A mechanical but wide migration.

## §4 Open questions — the real tensions (don't paper over them)

1. **`{}` blocks vs maps — the empty case. RESOLVED (2026-06-24, maintainer):** **`{}` = empty block;
   `{:}` = empty map** (the colon marks "map"). Non-empty cases were never ambiguous — `{ e }` (block) vs
   `{ k: v }` (map) already split on the `:` pairs; the rule only had to disambiguate the *empty* case, and
   `{:}` does it minimally (the colon is the same "map" marker, just with no entries). The JS block-vs-object
   trap is avoided without tagging every map literal. *(Q1 closed; the rest of §4 remains open.)*
2. **`[]` carries type-args *and* sized/repr types *and* lists.** In *type* position `Name[…]` is uniformly
   "bracketed type/size params" (so `Binary[64]` and `List[T]` are the *same* form — fine); in *value* position
   `[…]` is a list. The residual edge is a **list literal at statement start right after an expression on the
   prior line** — newline/terminator handling must make `expr` ⏎ `[…]` unambiguous. Bounded, but specify it.
3. **`<=`/`>=` → `lte`/`gte` is asymmetric with `<`/`>`.** Mycelium already treats `a < b` as infix sugar for
   `lt(a, b)`; keeping `<`/`>` as glyph sugar but spelling `<=`/`>=` as words is defensible (those glyphs are
   the most type-arg-ambiguous) but worth stating explicitly so it reads as intentional, not an omission.
4. **Migration is wide and the corpus is the product.** Every `<T>`, `Binary{8}`, `<+-0>`, and `->` in the
   spec, examples, conformance corpus, and (small) code must move together — a single coordinated change, not a
   drip. Cheap *now* (design phase, ~no ecosystem); expensive later. Sequence it as one supersession wave.

## §5 Definition of Done (this note)

A binding RFC (or an RFC-0019 supersession + RFC-0030/0025 update) ratifies §2. §4-Q1 (empty `{}`) is
**resolved** (`{}` = empty block, `{:}` = empty map); §4-Q2 (list-at-statement-start) must be **resolved with
an explicit disambiguation rule** before the grammar commits.
This note **enacts nothing** (VR-5/G2); it is the direction record + the grounding, superseded (append-only) by
that act.

---

> **Provenance.** Grounded in `docs/spec/grammar/mycelium.ebnf` (the measured current allocation), RFC-0019
> §4.1 (Enacted type-arg brackets), RFC-0025 §4.3 (M-745 deferred operators), RFC-0030 (concrete grammar),
> RFC-0001/RFC-0033 (repr-descriptor surface), and the maintainer's decided scheme (2026-06-24). Advisory; no
> normative claim (VR-5 / G2).
>
> **Revision history.** *2026-06-24* — initial Draft (the decided delimiter/operator deconfliction; resolves
> M-745 by reallocation; supersession of RFC-0019 §4.1 proposed, not performed). *2026-06-24 (rev.)* — §4-Q1
> (empty `{}`) **resolved** by the maintainer: `{}` = empty block, `{:}` = empty map. §4-Q2 remains open.
> *2026-06-25 (D7 decision record, post corpus-alignment audit; Status unchanged — Draft/advisory)* — the
> maintainer **confirms the direction**: **adopt `[]` for type-arguments**, freeing `<>` for
> comparison/shift operators (`< > << >>`). **Rationale:** this resolves the triple-loaded `<>` vs type-arg
> collision (**M-745**) by *reallocation*, with **no disambiguation machinery** — `[]` has spare capacity and
> the type-arg-vs-list-literal split is a clean position rule (Mycelium has no `arr[i]` indexing and no
> juxtaposition application, §2). **Consequence — a grammar-supersession wave**, flagged as a **tracked epic**
> (forthcoming; *no code is implemented by this note*): it **supersedes** RFC-0019 §4.1's `<…>` type params
> and RFC-0030's current `<>`-based direction, and spans the lexer/parser (`crates/mycelium-l1`), the
> tree-sitter / editor grammars (`just grammar-gen`), the conformance corpus, and the `[]`-vs-list-literal
> disambiguation (§4-Q2, which must be resolved with an explicit rule before the grammar commits). **Maintainer
> design input to fold into that work:** (a) **better support for line-breaks / indentation** so syntax is
> more **human-visible**, and (b) using **`,` to delineate syntax portions**. This is a **recorded direction**,
> not an enactment — DN-31 stays **Draft**; the binding act is the future RFC/supersession (VR-5 / G2).
>
> **Revision — *2026-06-27 (kind-split refinement; Status unchanged — Draft/advisory).*** The maintainer
> **refines §2/§3 in-session: repr/size types STAY on `{}`** (`Binary{8}` is **not** moved to `Binary[64]`),
> adopting a **bracket-by-kind** principle rather than bracket-by-availability:
> **`[T]`** = **type** params/args + list literals; **`{N}`** = **const/width** params **and** repr/size types;
> **`<>`** = operators only; **`0t`** = trit literals; **`=>`** = return arrow; **`<=`/`>=`** → **`lte`/`gte`**.
> A const/width param is declared **`f{N}`** (explicit, per kind — matching its use in `Binary{N}`); a type
> param is `f[T]`; a function generic over both is `f[T]{N}` (e.g. `map_get[V]{N}(m: Map[Binary{N}, V], k:
> Binary{N})`). This is the *smallest* migration from the landed `fn f<N>` (swap the bracket; the pervasive
> `Binary{N}` repr surface — including the landed `Ty::Binary(Width)` — does **not** re-spell).
> **Repr-keyword shortening (proposed, 2026-06-27 — maintainer):** the long paradigm type-keywords gain
> short, ergonomic forms that keep their width/dims on `{}` — **`bin{N}`** (Binary), **`tern{N}`**
> (Ternary), **`emb{…}`** (Dense / embeddings — dims + scalar kind), **`hvec{…}`** (VSA / HDC
> hypervectors — element-space + dims). **`vec` is rejected — it collides with `Vec`** (the
> `std.collections` cons-list, `lib/std/collections.myc`); `hvec` is used. This is a **lexicon amendment**
> to reconcile with DN-02/DN-03 (the type-keyword set) and `crates/mycelium-l1/src/token.rs`
> (`Tok::Binary`/`Ternary`/`Dense`/`Vsa`); captured here, ratification pending that reconciliation.
> **Rationale (second-/third-order effects, recorded):** (1) it eliminates the corpus's **largest** migration
> (repr `{}` is the most pervasive surface); (2) it adds **zero new `{}` ambiguity** — `Binary{8}` (type
> position) already coexists with `{ block }` and `{k: v}` (expr position) by position, and `f{N}` sits in the
> param-declaration slot, unambiguous against equals-bodied `fn f(…) = …`; (3) it is **honest about kind** —
> width *is* a const-generic (DN-42), so `{8}`/`{N}` reading as "const" teaches the `[type]` vs `{const}`
> distinction; (4) it **avoids** the alternative of putting type-args on `{}` (`List{T}`), which would
> reintroduce the Rust `Name{block}`-vs-type-application footgun (types stay square ⇒ `List[T]` never collides
> with a block). **§4-Q2 — resolution DIRECTION fixed: a LAYOUT-INDEPENDENT grammar (maintainer,
> 2026-06-27).** Newlines are **formatting-only, never semantically required**: the *same* program parses
> identically whether written as a dense **stream** ("a series of lambda functions and such") or — as in
> practice — **formatted with line breaks for human readability**. Structural delineation is therefore by
> **explicit delimiters** (the maintainer's **`,`-delineation**) plus the **type-position-vs-value-position**
> distinction — **not** by whitespace. So the type-app-vs-list edge is settled without any newline rule: a
> list `[…]` follows a delimiter / sits in value position; a type-or-size application `Name[…]` has `Name` in
> type position (and Mycelium has no `arr[i]` indexing and no juxtaposition application, so there is no third
> reading). The earlier "newline/adjacency rule" is therefore **withdrawn** — newlines must not be load-bearing.
> The **exact delimiter set + adjacency rules** are the remaining spec detail for the binding grammar RFC; the
> *principle* (layout-independent, explicit-delimiter, human-formatted-in-practice but stream-legal) is fixed.
> **Lambdas are declared with an explicit `lambda` keyword** (not bare arrows), so a streamed lambda-chain stays
> legible and unambiguous — a new reserved word to reconcile with **DN-02/DN-03** (lexicon) + `token.rs`, and
> input to **M-704** (full higher-order functions / closures). **Guarantee model cross-ref:** the width-generic guarantee is
> **per-instance** (DN-51 §2-D4 / the width-arithmetic decision), not a uniform tag. Still a **recorded
> direction** — DN-31 stays **Draft**; the binding RFC/supersession (+ the grammar-supersession epic) is the
> enacting act (VR-5 / G2).
