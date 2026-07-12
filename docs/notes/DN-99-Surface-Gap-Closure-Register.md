# Design Note DN-99 — Surface-Gap Closure Register and Plan (the spw / RFC-0031 stdlib-port wave)

| Field | Value |
|---|---|
| **Note** | DN-99 |
| **Status** | **Draft** (2026-07-10). Authored as **READ + a new DN only** for the `spw` stdlib-port wave (RFC-0031). It **enacts nothing** and **moves no other doc's status** (house rule #3, append-only). It **classifies** the surface gaps that block porting the Rust stdlib to self-hosted `.myc` into a single closure register, ranks the closures, and proposes an enabler (`enb`) backlog to file. All rows are `Empirical` where a cited `file:line` / `M-id` was read against the code, and `Declared` for any closure design not yet ratified (VR-5). |
| **Decides** | *Proposes, for ratification:* (1) the canonical **status + closure-layer** of every enumerated surface gap; (2) a **ranked closure plan** grouped by layer (grammar-`enb` / runtime-`enb` / transpiler / accepted-idiom); (3) the **already-closed set** that must not be re-opened (the Float-lesson correction); (4) the **`enb` issue backlog** the integrator should file. It does **not** edit `issues.yaml`, `CHANGELOG.md`, `Doc-Index.md`, `lib/compiler/**`, or `crates/mycelium-l1/**` — the integrator / the cloud semcore lane own those. |
| **Posture (maintainer, 2026-07-10)** | The **kernel + lexicon are UNFROZEN** (a superseding/amending ADR is being drafted). A kernel/grammar/runtime closure is therefore **actionable optimization**, not frozen-deferred. The **north star is ZERO hand-ports** long-term via **mechanical** porting, with an explicit division of labor: the Rust→Mycelium parse/translate **rules live in the transpiler**; the **language's job is gap closure** (expand grammar / kernel / runtime so every construct is *expressible*). The plan is framed as **two coupled tracks** converging on zero-hand-port (§4). Kernel/semcore-lane implementation still **coordinates with the cloud CC session** (M-1013 SCC in flight: `resolve_imports` / `Wrapping` / etc.) — those items are marked **"cloud-lane, coordinate,"** as **unfrozen-actionable, NOT deferred-by-freeze**. |
| **Feeds** | RFC-0031 (the port wave) port-authoring guide; DN-34 §8 (transpiler gap taxonomy); the `enb` backlog (§8, to be filed by the integrator). |
| **Date** | July 10, 2026 |
| **Task** | spw surface-gap synthesis — one closure register + ranked plan + `enb` backlog from the verified, adversarially-swept input register. |

> **Grounding + honesty (transparency rule / VR-5 / G2 / house rule #4).** This note **works up
> classifications and a plan**; it does not take a decision (house rule #3). Each register row's
> **evidence** column is `Empirical` — a `file:line` or `M-id` read against the tree (dev tip
> `6d906b76`, 2026-07-10). Each **closure approach** not yet built/ratified is `Declared`. No tag is
> upgraded past its basis. **No sycophancy:** where the input's `layerGuess` was wrong (a construct the
> code shows is *already landed*), this note says so plainly and flips the row — the recurring
> **Float lesson** (mitigation #14): the tracker/pilot flag lags the code, so a gap must be verified
> against the source before it is called open.

---

## §1 Purpose and grounding

The `spw` wave ports Mycelium's Rust stdlib to self-hosted `.myc` (RFC-0031). A **surface gap** is any
construct that blocks porting. There are **two distinct kinds**, and this register keeps them separate:

1. **Language-surface gaps** — the `.myc` grammar/runtime genuinely cannot express the construct
   (e.g. a per-constructor visibility seal, a `?`-operator, transcendental float numerics,
   cross-nodule *execution*). These close via a **language enabler** (`enb`): grammar / kernel /
   runtime work in `crates/mycelium-l1/**` (the cloud semcore lane).
2. **Transpiler gaps** — the language *can* express it by hand (struct→ADT, `impl`→free fn,
   `use`→local-mirror, `matches!`→`match`) but the transpiler can't yet auto-emit it. These close by
   **improving the transpiler** (`crates/mycelium-transpile/**`, outside the semcore lane) or are
   **accepted as hand-port**.

Under the **unfrozen posture**, a real language closure is *preferred over an accepted-idiom
workaround* wherever impact justifies it (e.g. cross-nodule imports → close the linking /
symbol-resolution *in the language*, not the local-mirror sidestep). The honest coverage metric
remains **`checked_fraction`** (`myc check`-clean), never `expressible_fraction` alone (M-991 /
DN-34 §8.7).

**House-rule anchors:** the transparency lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` (rule #1);
never-silent swaps/gaps (rule #2, G2); append-only status (rule #3); grounded claims (rule #4);
small auditable kernel (rule #5, KC-3).

---

## §2 The full register (92 gaps)

Legend — **Status**: `open` (genuine unresolved language/runtime gap) · `partial` (core landed,
residual open) · `already-closed` (landed on both axes; do not re-open) · `transpiler-only` (language
expresses it; only the auto-emitter is open) · `idiom` (closed by a sanctioned hand-port convention).
**Layer**: `grm` grammar-`enb` · `rt` runtime-`enb` · `tr` transpiler · `ac` accepted-idiom · `cl`
already-closed. **DN?** = needs its own Draft DN before implementation. **Coll.** = collision with the
cloud semcore lane (`crates/mycelium-l1/**` + `lib/compiler/**`): `none` / `low` / `HIGH`. Sz = S/M/L/XL.
**Pri** = backlog-filing priority `P1` (highest) / `P2` / `P3` — the order to file/close a row; a
distinct axis from §4's language-enabler *build* rank (a row may file at `P2` yet be the rank-1 build
unblocker, e.g. #41).

| # | Gap | Status | Layer | Evidence (cite) | Closure approach | DoD (short) | DN? | Tracking | Coll. | Sz | Pri |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | import-use (`use` path) | idiom | ac | `parse.myc:2400`; M-662 landed | local-mirror redeclare; transpiler keeps flagging `Category::Import` | import-less nodule `myc check`-clean; `use` stays flagged | no | M-1001+M-662+M-982 | low | S | P2 |
| 2 | impl-block | closed | cl | `parse.myc:3670`; M-664; `emit.rs:1901` | none — native + auto-emitted | recorded already-closed | no | M-664/M-659/M-673 | none | S | P3 |
| 3 | derive-attr | idiom | ac | `emit.rs:1538/1659`; DN-34:552 | drop Debug/Clone; hand-write structural eq | per-module drop/helper; sub-gap stays never-silent | no | DN-34 §8; DN-54/M-812 | none | S | P3 |
| 4 | struct-def | closed | cl | `emit.rs:1652`; M-1006 | none — struct→single-ctor ADT landed | tuple/named fixtures green | no | M-1006 | none | S | P3 |
| 5 | generic-bound | closed | cl | `parse.rs:1143`; RFC-0019 §4.1; M-673 | none — bounded fn type-params landed | DN-14 §3 rows 6/7 present | no | M-657/659/673 | none | S | P3 |
| 6 | reserved-word (ident collision) | idiom | ac | `token.rs:415`; DN-34 §8.8 | prefix-rename (Kw-/P-/G-…) + FLAG | renamed, `myc check`-clean, differential witnessed | no | DN-34 §8.8 | low | S | P3 |
| 7 | test-item (`#[test]`) | idiom | ac | `gap.rs:29-32`; RFC-0031 §5 | not ported; `Category::TestItem`, denom-excluded | Rust crate stays differential oracle | no | DN-34 §8; RFC-0031 D6 | none | S | P3 |
| 8 | multi-stmt-body (let-chain+tail) | closed | cl | `ast.myc:595`; `emit.rs:518-548` | none — nested `let…in` landed | `emit.rs:62` fixture green | no | RFC-0030/0032 | none | S | P3 |
| 9 | named-field-drop | closed | cl | `emit.rs:1508`; M-1006 | none — positional + never-silent NamedFieldDrop | resolvability-gated fixtures green | no | M-1006 | none | S | P3 |
| 10 | payload-variant | closed | tr | `parse.rs:713`; `emit.rs:1559` | none — tuple + named-field variants emit | census refresh only | no | M-1006 | none | S | P3 |
| 11 | macro-invocation | tr-only | ac | `transpile.rs:300`; DN-56 | hand-expand (`matches!`→`match`, `format!`→encoders) | zero macro tokens in emitted `.myc`; `myc check`-clean | no | M-875; DN-34 §8.3/8.5 | none | M | P2 |
| 12 | trait-decl | closed | cl | `parse.rs:723`; M-1013; `semcore.myc:2352` | none — `trait T{..}` parses+checks+prints | register_traits landed | no | RFC-0019; M-1013 | none | S | P3 |
| 13 | mod-declaration | idiom | ac | `transpile.rs:281`; RFC-0031 layout | nodule-per-file split; cross-nodule *exec* → M-982 | worked multi-module port; per-file `myc check`-clean | no | M-982 (dep) | low | M | **P1** |
| 14 | top-level-const/static | idiom | ac | `totality.myc:273`; 177 nullary-fn defs | `const`→nullary `fn name()=>T=expr` | port guide records mapping | no | untracked | none | S | P2 |
| 15 | slice-array-type | idiom | ac | `ast.myc:320`; RFC-0037 D1 | `&[u8]`→Bytes, `[T;N]`→Seq, `&[T]`→Vec cons-list | 3-way mapping documented + witnessed | no | RFC-0032/0037 | none | S | P3 |
| 16 | qualified-type-path | closed | cl | `map.rs:85`; M-1001; M-662 | none — flag-don't-guess + `use`-then-bare landed | regression guards hold | no | DN-34 §8.7-8.8 | none | S | P3 |
| 17 | unit-type-return | idiom | ac | `error.myc:88`; `parse.rs:872` | `type Unit=U;` local mirror; `Result[Unit,E]` | mirror declared per nodule; honest note | no | DN-73 §D5 | none | S | P3 |
| 18 | qualified-fn-call (`T::m`) | idiom | ac | `map.rs:85`; M-664; M-662 | `Type::m`→`type_m` free fn; `use`+dotted App | differential upgrades Declared→Empirical | no | M-664/M-662 | low | M | P2 |
| 19 | mutable-reference (`&mut`) | idiom | ac | `ebnf:240`; DN-34 §8.11; ADR-003 | value-threading (return new `T`) | per-site differential parity | no | DN-34 §8.11; ADR-003 | low | M | P3 |
| 20 | inner-attribute (`#![...]`) | idiom | ac | `transpile.rs:43-67`; DN-34 §8.3 | drop non-doc, fold `//!` into nodule header | never-silent gap recorded | no | DN-34 §8.3 | none | S | P3 |
| 21 | member-access-expr (non-`self` field / index) | tr-only | tr | `emit.rs:1120`; ADR-027 | widen field-env desugar; index → `get`+lift-Option hand-port | checked_fraction rises; no fabrication | no | DN-34 §8.12; M-1006 | low | M | P2 |
| 22 | usize-width | idiom | ac | `token.rs:477`; ADR-028 | `usize`/`uN`→domain `Binary{N}` + FLAG | width choice recorded never-silent | no | ADR-028; RFC-0033 | none | S | P3 |
| 23 | closure-lambda-expr | closed | cl | `parse.rs:940`; M-704/706/822 | none — `lambda(x)=>e` parses/checks/lowers | three-way differential green | no | M-704/706/822 | none | S | P3 |
| 24 | struct-literal-expr | partial | tr | `emit.rs:1187`; M-1006 | base landed; `..rest`→hand-port per-field copy | struct-update sites hand-ported + witnessed | no | M-1006 | none | S | P3 |
| 25 | char-literal | idiom | ac | `lex.myc:553`; no CharLit token | codepoint `0b…`/Int + `// 'x'` comment | codepoint idiom, differential green | no | untracked | none | S | P3 |
| 26 | signed-int-type | idiom | ac | `ternary.myc:36`; ADR-028 | sign-magnitude ADT (SPos/SNeg over `Binary{N}`) | shared `SInt` helper; M-874 prong resolved | **yes** | M-874; ADR-028 | none | M | P2 |
| 27 | dyn-trait-object | idiom | ac | RFC-0019:606; ADR-033 §11 | bounded generic `[A: T]`; hetero → ADR-033 escape hatch | `&dyn`→`[A:T]` rewrite, `myc check`-clean | no | ADR-033; RFC-0019 §6.2; DN-55 | low | S | P2 |
| 28 | impl-trait-sig (arg-position APIT) | tr-only | ac | `parse.rs:1143`; RFC-0019 §4.1 | `impl Into[S]`→`fn f[R: Into[S]](x:R)` | rewritten sites `myc check`-clean | no | RFC-0019 §4.1; M-876 | low | S | P2 |
| 29 | match-pattern-struct-variant | tr-only | tr | `emit.rs:1482`; `ast.myc:565` | `Pat::Struct` arm → positional, resolvability-gated | fixtures over `Fn{name,..}` / `Lambda{..}` | no | DN-34 §8.12; M-1006 | none | S | P3 |
| 30 | numeric-cast-expr (`x as T`) | idiom | ac | `token.rs:415` (no `as`); `emit.rs:1276` | `width_cast`/`truncate`/`to_flt` prim calls | ledger maps casts to named prims | no | DN-34 §8.18; DN-91; DN-51 | none | S | P2 |
| 31 | conditional-pattern-binding (`if let`/guards) | idiom | ac | `ebnf:292/301`; `semcore.myc:2163` | desugar to `match`+`if/then/else` | idiom recorded w/ fall-through caveat | no | untracked | none | S | P3 |
| 32 | type-alias item (`type X=<type>`) | idiom | ac | `parse.rs:696`; `lex.myc:84` | single-ctor newtype `type X=C(..)` | newtype form `myc check`-clean | no | DN-34 §8.5 | low | S | P3 |
| 33 | top-level-const-item | idiom | ac | `ast.rs:173` (no Const); 133 nullary fns | `const`→top-level nullary fn | ported const bindings as nullary fns | no | untracked | low | S | P3 |
| 34 | unit-type (`()`) | idiom | ac | `ast.rs:553`; DN-73 D5; M-826 | canonical `Unit=U` from std.core | one `Unit` exported + `use`d | no | DN-73 D5; M-826 | none | S | P3 |
| 35 | slice-type (native `[T]`/`&[T]`) | idiom | ac | `parse.rs:1522`; RFC-0033 §3.1.3 | `&[T]`/`Vec<T>`→`Vec[T]` cons-list; `[T;N]`→Seq | port convention + `[…]` sugar | no | RFC-0033 §3.1.3; RFC-0040 | none | S | P3 |
| 36 | reference-type (`&T`/`&mut T`) | idiom | ac | `parse.rs:1522`; NFR-5; DN-32 | drop borrow, pass by value / thread | parser error steers to value/Substrate | no | untracked | none | S | P3 |
| 37 | **sealed-constructor-visibility** | **open** | **grm** | `ast.rs:42/302`; `checkty.rs:1232`; DN-53 §B.5 | per-ctor `priv`/seal marker; export name not construction | conformance accept/reject + differential | **yes** | DN-53 §B.5/B.6 Q1 | **HIGH** | L | P2 |
| 38 | native-integer-type | idiom | ac | `checkty.rs:6840`; DN-42/M-753 | `uN`→`Binary{N}`; ambient-paradigm resolve | width-selection table in port guide | no | RFC-0012; DN-42/M-753 | none | S | P3 |
| 39 | bool-primitive | idiom | ac | `checkty.rs:622`; RFC-0007 | prelude `Bool`/`True`/`False`; `if`-as-Match | ports use `Bool`, `myc check`-clean | no | RFC-0007; RFC-0032 D1 | none | S | P3 |
| 40 | imperative-control-flow | idiom | ac | `ebnf:297`; `parse.myc:2431` | recursion / bounded `for`-fold (RFC-0007 §4.8) | teaching diagnostic already lands | no | RFC-0007 §4.8 | none | S | P3 |
| 41 | **cross-nodule-runtime-exec** | **partial** | **rt** | `checkty.rs:1275` (check landed); `eval.rs:557` | phylum-wide evaluator reusing the check-time import registry | two-nodule phylum witness (interp+AOT) | **yes** | M-982 | **HIGH** | L | P2 |
| 42 | **float-eps-delta-numerics** (transcendentals) | **open** | **rt** | `prims.rs:2063/2071`; `math.myc:46`; M-718 | ADR-gated numerics enabler: prims + ε/δ matrix | prims registered, never-silent domain errors | **yes** | (deferred in M-718) | **HIGH** | XL | P2 |
| 43 | float-arith (scalar) | closed | cl | `prims.rs:210-237`; ADR-040 **Enacted**; M-896-900 | none — **DO NOT RE-OPEN** (Float lesson) | ADR-040 Enacted 2026-07-02 | no | ADR-040; DN-69; M-896-900 | none | S | P3 |
| 44 | signed-integer-first-class-type | partial | ac | `prims.rs:169` (ops landed); ADR-028 | ops closed; typed-view above kernel dispatches to signed ops | DN decides scope; ops need no work | **yes** | E18-1; ADR-028; M-767 | low | L | P3 |
| 45 | **platform-width-and-char-types** | **partial** | **rt** | `repr.rs:102-149`; M-874 | canonicalize `Binary{N}` mapping; char via Bytes/std.text | DN decides usize/isize/char sub-questions | **yes** | M-874; DN-34 §8.4/8.5 | **HIGH** | L | P2 |
| 46 | references-borrow-value-semantics | idiom | ac | `eval.rs:181`; `map.rs:265`; DN-71 | none — `&T` erases; `&mut` gapped never-silent | `&mut`→value-return hand-port | no | DN-34 §8.11; DN-71 | none | S | P3 |
| 47 | collections-seq-substrate | closed | cl | `repr.rs:138`; M-749; RFC-0032 §5 D3 | none — Repr::Seq landed; ops = M-716 port work | three-way differential green | no | M-749; M-716 | none | S | P3 |
| 48 | parsable-not-runnable-frontier | closed | cl | `runnable_gate.rs:1`; DN-52; M-807 | none — census + gate + 3-way harnesses landed | DN-50→Resolved hygiene | no | M-807; DN-50/52/56 | low | S | P3 |
| 49 | transpiler-use-import-poison | idiom | ac | `transpile.rs:374`; M-1001 | landed — `use` gapped not emitted; local-mirror ports | vet re-run confirms no whole-file poison | no | M-1001; DN-34 §8.7-8.8 | none | S | P3 |
| 50 | transpiler-impl-undefined-trait+derive | tr-only | tr | DN-34:351/552; DN-41; DN-54 | gap-or-free-fn emit; width_cast body; std-derive `lower` lib | Widen impls off CheckError; checked_fraction | no | E18-1; DN-34 §8.8; M-1001/2 | low | L | P2 |
| 51 | transpiler-macro-expand + reserved-word | tr-only | tr | `reserved.rs:1` (RW done); M-875 (macro open) | RW landed; macro expand-first pre-pass (toolchain DN) | before/after expressible_fraction on std-cmp | **yes** | M-1001 (done); M-875 | none | M | P2 |
| 52 | try-question-operator (idiom facet) | idiom | ac | `result.myc:28`; `select.myc:518` | hand-desugar to `and_then` / nested `match` | idiom recorded; before/after example | **yes** | M-527 | none | M | P2 |
| 53 | lifetime-annotation | closed | ac | `map.rs:265`; ADR-003; DN-34 §8.14 | none — `&'a T` erases; lifetime-arg gaps named | ledger row: transpiler-handled, not a gap | no | ADR-003; DN-34 §8.14 | none | S | P3 |
| 54 | iterator-adapter-method-chain | idiom | ac | `iter.myc:44`; `parse.rs:1970` | combinators / `for`-fold; extend iter.myc residuals | zip/enumerate/rev/sum land three-way | no | M-526; DN-34 | low | M | P2 |
| 55 | associative-and-deque-collections | partial | ac | `collections.myc:104`; `checkty.rs:103` | assoc-list Map/Set landed; add `Deque` banker's queue | Deque three-way; String-key eq decision | no | M-716; M-867 | low | M | P2 |
| 56 | **shared-ownership / interior-mutability** | **partial** | **rt** | `ast.myc:44`; DN-94; RFC-0031:94 | Box→plain T (done); shared-mut excluded (RT1); tasks Phase-7 | value-field + FLAG; runtime tier post-Ph7 | **yes** | DN-94; RFC-0008 RT1 | **HIGH** | XL | P3 |
| 57 | outer-non-derive-attribute | idiom | ac | `emit.rs:210-219`; `gap.rs:29` | drop-and-record; `#[cfg(test)]`→TestItem denom-excluded | never-silent sub-gaps (landed) | no | DN-34 §8; M-1001 | none | S | P3 |
| 58 | turbofish + inference placeholder (`_`) | idiom | ac | `parse.rs:2253`; `checkty.rs:4600`; M-657/673 | `foo::<T>(x)`→`foo(x):T` ascription; `_`→concrete by hand | idiom + one witnessed site | no | M-657/673; RFC-0007 §11.3 | none | S | P2 |
| 59 | trait-associated-type | idiom | ac | `checkty.rs:264`; RFC-0019 Q-assoc (v2) | monomorphize binding to concrete; free/inherent fn | inlined binding, no `type X=Y`, witnessed | no | RFC-0019 Q-assoc; RFC-0007 §10 | none | S | P2 |
| 60 | **try-operator (`?`) grammar sugar** | **open** | **grm** | `token.rs:267` (no `?`); `parse.rs:1932` | `Question` token + postfix desugar to existing bind | differential `?` ≡ hand-match; three-way | **yes** | untracked (this DN) | **HIGH** | M | **P1** |
| 61 | method-call-expr | idiom | ac | `emit.rs:1013`; `parse.rs:2362`; ADR-003 | `x.m(a)`→`m(x,a)` free fn (transpiler already) | taxonomy row; no postfix surface added | no | DN-34; RFC-0031 | low | S | P3 |
| 62 | lifetime-parameter-declaration | idiom | ac | `map.rs:253`; `ebnf:154`; ADR-003 | erase `<'a>` with the borrow; optional tr erase-arm | register row distinct from GenericBound | no | ADR-003 | low | S | P3 |
| 63 | **generic-parameterized-impl-block** | **partial** | **grm** | `parse.rs:1179` (no impl type-params); DN-34:550 | add impl-level `type_params` slot; interim impls-as-fns | parses `impl[T] Foo[T]`; lifetime decided | **yes** | DN-34 §8.5 | **HIGH** | L | P2 |
| 64 | turbofish-type-args (silent-drop) | tr-only | tr | `emit.rs:1013`; RFC-0020 §4.2 | never-silent: gap load-bearing turbofish (not silent elide) | test asserts gap, not bare `collect(...)` | no | M-1001 | none | S | P3 |
| 65 | tuple-type-and-destructuring | partial | ac | `parse.rs:1611`; M-826; DN-73 | core landed; `.0/.1`→`let (…)=tuple` destructure | 24 projection sites hand-ported | no | DN-73 D5; M-826/921 | none | S | P2 |
| 66 | array-repeat-expr (`[x; N]`) | idiom | ac | `emit.rs:1104`; `ebnf:415` | enumerate N copies into Seq ListLit (small N) | sites as enumerated Seq, three-way | no | DN-34 §8 | none | S | P2 |
| 67 | index-subscript-expr (`arr[i]`) | idiom | ac | RFC-0037:190 **Enacted**; `ast.rs:693` | `get`/`slice_opt`; l-value → functional `set`/fold | `fn set` helper; sites rewritten | no | RFC-0037 §190; DN-31 | low | S | P2 |
| 68 | range-expr (`..`/`..=`) | idiom | ac | `lexer.rs:170` (no `..`); `parse.rs:1970` | `contains`→comparisons; `(0..n)`→`iota` helper | iota lands three-way; sites rewritten | no | untracked | low | S | P2 |
| 69 | restricted-visibility-modifier (`pub(crate)`) | idiom | ac | `ast.rs:42`; DN-53 §B.6 Q1 | collapse to binary; field newtype → seal idiom | no `pub(` survives; genuine sub-scope FLAGged | no | DN-53 §B.6 Q1/B.5 | none | S | P2 |
| 70 | **format-string-mini-language** | **partial** | **ac+rt** | `fmt.myc:19`; `transpile.rs:298`; M-533 | hand-compose over Bytes; Display + `{:.2e}` prim = M-533 | idiom recipe; M-533 DN for float-precision prim | **yes** | M-533; DN-34 §8 | **HIGH** (residual) | L | **P1** |
| 71 | tuple-let-destructure (transpiler) | partial | tr | `emit.rs:569`; M-826 | `let (a,b)=e`→`match e{(a,b)=>…}` in Stmt::Local | vet: `let (` sites off MultiStmtBody-gap | no | M-1006 | low | S | P2 |
| 72 | string-literal-pattern | closed | rt | `emit.rs`; DN-34 §8.21; M-1035; PR #1372 | L1 enabler landed (M-1035/ENB-12) AND the transpiler arm now EMITS (PR #1372): `match s { "yes" => True, _ => False }`, `myc check`-clean — emit only WITH a wildcard/default arm (open-`Bytes` W7), else gaps never-silently (G2/VR-5) | closed: `string_literal_pattern_emits_with_l1_enabler` pins emit+defaultless-gap; corpus win awaits conversion-method mapping (M-1037) | no | M-1035; PR #1372; DN-34 §8.21 | low (residual: M-1037) | S | P2 |
| 73 | or-pattern-in-match-arm | closed | cl | `emit.rs:1466`; M-873; RFC-0020 §9 | none — emits end-to-end (100% witnessed) | one regression fixture (nice-to-have) | no | M-873; RFC-0020 §9 | none | S | P3 |
| 74 | reserved-word-ctor-declaration | idiom | ac | `lex.myc:96-127`; `core.myc:82` | prefix-rename at decl (Kw-/G-/S-) + FLAG | docs enumerate the decl slot | no | DN-02/03; DN-80 §5 | low | S | P3 |
| 75 | drop-trait-raii-destructor | idiom | ac | `checkty.rs:49`; `ambient.myc:362` | rewrite RAII as explicit depth-threading | budget-threading differential parity | no | RFC-0031; DN-84 | HIGH (port site) | S | P3 |
| 76 | associated-const-item | idiom | ac | `parse.rs:1249`; `core.myc:145` | `impl T{const C}`→top-level nullary `fn t_c()` | ported consts as nullary fns, witnessed | no | spw; M-867 | none | S | P2 |
| 77 | self-type-keyword (`Self`) | idiom | ac | `token.rs:415` (no `Self`); zero `.myc` uses | write concrete impl target type; optional tr desugar | idiom recorded; optional emitter accelerator | no | DN-34 §8.5 | none | S | P2 |
| 78 | hex-integer-literal (`0x…`) | idiom | ac | `lexer.rs:205` (`0x`=Bytes); `checkty.rs:6823` | rewrite to width-typed `0b…` Binary{W} (never decimal) | splitmix64 consts as `0b…`, differential | no | untracked | low | S | **P1** |
| 79 | **host-effect-wild-execution** | **partial** | **rt** | `elab.rs:1484`; `eval.rs:1670/1687`; M-720/721 | grant real `wild:` ops from std-sys; fixture differential | one real syscall end-to-end + fixture witness | **yes** | M-720/721; M-722 | **HIGH** | M | P2 |
| 80 | const-fn-qualifier | idiom | ac | `ebnf:169`; `cmp.myc:23` | drop `const`; pure `.myc` fns const-eval by construction | 11 std-time sites port to bare `fn` | no | untracked | none | S | P3 |
| 81 | let-else-statement | idiom | ac | `parse.rs:1990`; `semcore.myc:4070` | `let P=e else{d}`→`match e{P=>…,_=>d}` | register row + honest tr message | no | DN-34 §8.5 | low | S | P3 |
| 82 | assignment-and-mutation-statement | idiom | ac | `ast.rs:693`; `substrate.myc:162`; ADR-003 | functional rewrite (`let` shadow / fold / thread) | catalog row; tr NOT taught to invent mutation | no | DN-34 | none | S | P3 |
| 83 | imperative-loop-construct | idiom | ac | `RFC-0007:257`; `iter.myc:75` | recursion / bounded `for`-fold | playbook row; optional tr fold-emit | no | RFC-0007 §4.8 | partial (l1 sites=boot10) | M | P2 |
| 84 | empty-marker-trait-impl | tr-only | ac | `parse.myc:3640`; `semcore.myc:2656`; FLAG-core-4 | drop (errors-as-values); optional tr empty-ImplDecl | recorded expressible + accepted drop | no | FLAG-core-4; M-535 | none | S | P3 |
| 85 | byte-literal + byte-string (`b'…'`/`b"…"`) | language-enabler | tr | `emit.rs:787`; `lex.myc:513`; DN-34 §8.21 | the byte-literal arm is `myc check`-clean **in isolation**, but corpus closure is gated on the ENB-1 unknown-prim symbol table (M-1024; FLAG-tr-unknown-prim) — landing it alone regresses `checked_fraction` via the co-located blind method-call emit | byte-literal arm lands once the ENB-1 known-prim gate exists; not a standalone transpiler-only win | no | M-1024; DN-34 §8.21 | HIGH | S | P2 |
| 86 | bitwise-and-shift-operator-suite | closed | cl | `token.rs:290`; `parse.rs:2396`; M-745 | none — `<<>>&\|^!` desugar landed; compound-assign=SSA | register row + compound-assign note | no | M-745; RFC-0025 §4.1 | none | S | P3 |
| 87 | phantom-type-marker | idiom | ac | `checkty.rs:1691` (unused params tolerated) | drop PhantomData fields; keep unused type params | DN records contract; regression test FLAGged | no | untracked | low (test) | S | P3 |
| 88 | **never-type-divergence (`-> !`)** | **open** | **rt** | `ast.rs:553` (no bottom); `sys.rs:32` | model as divergent host-effect prim + `diverges` effect | DN: divergence-as-effect + totality interaction | **yes** | untracked | **HIGH** | M | P3 |
| 89 | **statement-sequencing-body** | **partial** | **tr** | `emit.rs:637`; `ebnf:291` | Part1: `let _=e in body`; Part2: mutation→functional (sep gap) | tr emits `let _`; mutation stays separate gap | **yes** | untracked | low | M | P2 |
| 90 | auto-trait Send+Sync bound | idiom | ac | `eval.rs:559`; DN-33 §8.1 Q1; DN-84 | static dispatch rewrite; elide `+Send+Sync` markers | register row; markers dropped, no loss | no | DN-33 §8.1; DN-84; RFC-0019 | none | S | P3 |
| 91 | generic-function-declaration (`fn f<T>`) | closed | cl | `ebnf:162`; `parse.rs:857`; `emit.rs:229` | none — `fn f[T]` landed; `<T>` retired | register corrected to already-closed | no | RFC-0037; RFC-0019 §4.1 | none | S | P3 |
| 92 | box-recursive-indirection-type (`Box<T>`) | idiom | ac | `map.rs:176`; `ast.myc:44`; `checkty.rs:1652` | Box field → plain `T` (value-semantic recursion) | idiom documented; optional tr erase-arm | no | untracked | low | S | P3 |

**Status tally:** `open` = 3 (#42, #60, #88) · `landed-with-residual` = 1 (#37 — M-1027/DN-104, 2026-07-10;
the import-path refusal landed, the capability-gate claim did NOT — see A3 above and M-1036) ·
`partial` = 12 (#24, #41, #44, #45, #55, #56, #63, #65, #70, #71, #79, #89) · `already-closed` = 16 ·
`transpiler-only` = 10 · `idiom` (closed-by-convention) = 50.
**Total = 92.** So **66 are closed today** (16 landed + 50 idiom), **10 need only transpiler work**, and
**16 carry a genuine language/runtime residual** (the 4 `open` + 12 `partial`), of which **15 rows are
tagged `DN? = yes`** (before de-dup; §8 collapses the duplicates into the filable backlog).

---

## §3 ALREADY-CLOSED — do not re-open (the Float-lesson corrections)

These rows were flagged (in the pilot / by `layerGuess`) as open language-surface gaps but are **landed
on both axes**. Re-implementing them would duplicate work and violate append-only (rule #3). Verified
against the tree at dev tip `6d906b76`:

- **Scalar float (#43) — the biggest correction.** The pilot FLAG *".myc has no float surface"* is
  **stale**. `Repr::Float`/binary64 (M-896), float literal (M-897), arithmetic (M-898), comparison
  (M-899), and the certified-mode gate (M-900) all **landed 2026-07-02**; **ADR-040 is Enacted**
  (verified: header reads `Enacted (2026-07-02 …)`), prims registered (`prims.rs:210-237`
  `flt.add`…`flt.to_bin` — verified). Residual `lib/std/select.myc`/`spore.myc` comments that still say
  *"no float surface"* are **stale doc-hygiene** to refresh under the self-hosted-surface workstream,
  **not** a reason to touch ADR-040. Only **transcendentals + ε/δ numerics (#42)** remain genuinely open.
- **impl-block (#2), trait-decl (#12), generic-bound (#5), generic-function-declaration (#91)** — the
  trait/impl/bounded-generic surface is landed (M-664/M-659/M-673, RFC-0019 §4.1); `<T>` is retired in
  favor of `[T]` (RFC-0037). Verified `mod`/`?` absence confirmed real (#60, #13) — those stay open.
- **struct-def (#4), named-field-drop (#9), payload-variant (#10), multi-stmt-body (#8),
  or-pattern (#73), closure-lambda (#23), collections-seq (#47), bitwise-suite (#86),
  qualified-type-path (#16), parsable-not-runnable (#48)** — all landed (M-1006, M-704/706/822, M-749,
  M-745, M-1001, M-807). `lifetime-annotation (#53)` is transpiler-handled by erasure (ADR-003), not a
  language gap.

**Rule for the wave:** count a `#[test]` item, a NamedFieldDrop, a lifetime, or an emitted-but-gapped
`use` as *handled/never-silent*, **not** as a blocker — and verify any "open" flag against the source
before acting on it (mitigation #14).

---

## §4 Ranked closure plan — two coupled tracks converging on zero hand-port

Per the maintainer's north star, the plan is **two tracks**: **(a)** close every language expressibility
gap (grammar / kernel / runtime — now unfrozen), and **(b)** capture the translation rules in the
**transpiler** so it mechanically ports what we hand-port today. Ranked by impact within each layer.

### Track A — language `enb` closures (grammar / runtime; cloud semcore lane — coordinate)

**A0 (rank 1, highest impact) — Cross-nodule symbol resolution + execution (runtime-`enb`, #41; unblocks #13 mod, #1 import, #16/#18 qualified path).** The check-time half is **landed** (`resolve_imports`,
M-662); the runtime half is **open** (`eval.rs` holds a single per-nodule `env`). Under the unfrozen
posture this is the **preferred real close** over the local-mirror sidestep: give the evaluator a
phylum-wide view that reuses the *check-time import registry* as the runtime link table (DRY, KC-3, no
new L0 node), then AOT parity. **Cloud-lane, coordinate** (M-1013 SCC `resolve_imports` in flight) —
unfrozen-actionable, **not** deferred-by-freeze. `HIGH` collision; needs a Draft DN. Until it lands, the
local-mirror idiom (#1) keeps the port unblocked — so this is impact-ranked, not port-blocking.

**A1 — `?` try-operator grammar sugar (grammar-`enb`, #60; P1).** `Question` token + postfix desugar to
the existing `Result`/`Option` bind — surface+lowering over already-present runtime semantics, **no new
kernel semantics**. Pins the error-type unification rule; differential `?` ≡ hand-match. `HIGH`
collision (`token.rs`/`parse.rs`/`elab.rs` + the self-hosted frontend under port). Needs a Draft DN.

**A2 — impl-level generic params (grammar-`enb`, #63).** Add the `type_params` slot to `parse_impl_item`
(mirroring fn/object), thread into the impl AST, monomorphize via the existing path; decide lifetime
erasure. Needed for a *faithful impl-block-preserving* self-host; the stdlib lane is unblocked by
impls-as-functions meanwhile. `HIGH` collision; Draft DN.

**A3 — per-constructor visibility seal (grammar-`enb`, #37).** A `priv`/seal marker on the `constructor`
production so `pub type T = priv Mk(..)` exports the type *name* but withholds cross-nodule
construction; design with named-field visibility (DN-53 §B.5) and resolve §B.6 Q1 first. `HIGH`
collision; Draft DN. **Landed (M-1027, PR #1370, 2026-07-10) — Draft DN-104.** The Rust frontend +
`.myc` surface parity are in; the seal withholds construction **via an imported name only** — it is
**not** an enforced capability/security boundary (a same-named local shadow bypasses it; DN-104 §6
CRITICAL residual). The real fix (nodule-qualified type identity) is tracked as **M-1036**.

**A4 — transcendental + ε/δ float numerics (runtime-`enb`, #42; XL).** ADR-gated: decide kernel-prim vs
libm-behind-`wild` routing, the per-op ε/guarantee matrix, and the float `ApproxRule` wiring; then a
serial prim-lane wave registers prims with never-silent domain errors. `HIGH` collision; Draft DN/ADR.

**A5 — platform-width + char decision (runtime-`enb`/idiom, #45; subsumes #26/#44 signed).** Prefer the
**canonicalized `Binary{N}` mapping** (accepted-idiom, no kernel change) for `usize`/`uN`; route `char`
through the Bytes/std.text bridge; only escalate to a platform-abstract width / char paradigm on
witnessed demand. Draft DN records the decision (resolves M-874).

**A6 — host-effect real syscalls (runtime-`enb`, #79).** Grant real `wild:read/write/get_env/exit` ops
from `mycelium-std-sys` into the `PrimRegistry`; add a **fixture/sandbox** differential for
non-deterministic host ops (equality differential defeated by real syscalls). `HIGH` collision; Draft DN.

**A7 — never-type `-> !` (runtime-`enb`, #88).** Do **not** add a bottom `BaseType` (YAGNI); model
`exit` as a divergent host-effect prim + a `diverges` effect, with the totality-checker interaction
specified. Overlaps A6. Draft DN.

**A8 — shared-ownership / runtime tier (runtime-`enb`, #56; XL, post-Phase-7).** Box→plain-T is done;
shared-mutable-state is excluded by RT1/DN-94; tasks/channels wait for the runtime tier (`hypha`/`mesh`,
ratified-not-lexed). Out of RFC-0031 scope; blocks nothing now. Draft DN maps the value-semantics
runtime substrate before the L1 change.

### Track B — transpiler closures (capture the translation rules; outside the semcore lane)

Ranked by `checked_fraction` impact. All land in `crates/mycelium-transpile` (`low`/`none` collision):

- **B1 — string-literal pattern arm (#72, S)** and **byte-literal/byte-string arms (#85, S)** — trivial,
  high-frequency; reuse existing expression-position helpers.
- **B2 — tuple-let-destructure in `Stmt::Local` (#71, #89 Part 1, S/M)** — `let (a,b)=e`→`match`;
  `let _ = e in body` for value-producing discarded statements.
- **B3 — non-`self` field-access desugar (#21, M)** and **match-pattern-struct-variant arm (#29, S)** —
  reuse StructLayout + the resolvability gate.
- **B4 — turbofish never-silent gap (#64, S)** — stop the silent type-arg drop; gap load-bearing
  turbofish. **B5 — impl/derive-undefined-trait emit policy + std-derive `lower` lib (#50, L)**.
- **B6 — macro expand-first pre-pass (#51, M)** — toolchain-gated (Draft DN); optional bulk lever.
- **B7 — Self→concrete, Box→plain-T, lifetime-erase, iterator-chain auto-desugar** (#77/#92/#62/#54) —
  optional accelerators; each emission stays `Declared` until a differential upgrades it (M-991).

### Track C — accepted-idiom (document once in the RFC-0031 port guide; no code)

The 51 `idiom` rows are closed-by-convention. The port guide must record, once, the sanctioned mappings:
`use`→local-mirror; `const`→nullary fn; `&mut`→value-threading; `impl Trait`(arg)→bounded generic;
`x as T`→named prim; `if let`/guards→`match`; `[x;N]`→enumerated Seq; `arr[i]`→`get`/`slice_opt`;
`..`→`iota`/comparisons; `pub(crate)`→binary vis; `?`→`and_then`; hex-int→width-typed `0b…`;
`Self`→concrete type; assoc-const→nullary fn; `Box<T>`→plain `T`; PhantomData→dropped fields.

---

## §5 Adversarial-completeness attestation

The input register was delivered as **verified + adversarially completeness-swept**. This note adds an
independent **spot-verification** pass rather than re-deriving the whole sweep. Honest scope + residual
uncertainty:

- **Dry rounds performed here:** **1 targeted verification round** (not a full re-sweep). It confirmed,
  against the tree at `6d906b76`: the **Float landing** (ADR-040 Enacted; `flt.*`/`bin.to_flt`/
  `flt.to_bin` registered at `prims.rs:210-237`; M-896 present) — the biggest correction; the **genuine
  absence** of a `mod`/`module` token (`0` hits in `token.rs`) and a `?`/`Question` token (`0` hits) —
  confirming #13/#60 are real open gaps; and the **item-level-only `Vis`** (`ast.rs:42` `enum Vis
  {Private,Pub}`; item `vis` fields present) confirming #37's per-ctor seal is genuinely open.
- **What the critic lens covered (inherited from the input sweep + re-checked on samples):** the
  language-vs-transpiler split for every row; the Float-lesson (mitigation #14) on every "open" flag;
  the collision-with-semcore-lane classification; and the never-silent (G2) posture on each transpiler
  gap.
- **Residual uncertainty (declared honestly, VR-5):** (a) the **frequency/occurrence counts** (e.g. "33
  TestItem", "29 NamedFieldDrop", "152 iterator sites", "427 checkty `?` sites") are quoted from the
  input register and DN-34, **not re-counted here** — treat them as `Empirical`-per-source, not
  re-verified. (b) Only **~6 of 92 rows** were re-read against source in this pass; the other rows carry
  the **input sweep's** `Empirical` basis, which I did not independently reproduce line-by-line. (c) The
  register is **not proven exhaustive** — it enumerates the constructs observed across the RFC-0031 port
  targets + the semcore draft corpus; a not-yet-encountered Rust construct could surface a new row. This
  is a `Declared` completeness claim, **not** a `Proven` one. No tag in §2 is upgraded past its basis.

---

## §6 Definition of Done (for this DN)

1. The register (§2) records every gap with status + evidence cite + layer + closure + DoD + DN-flag +
   tracking + collision + size + priority — **done**.
2. The already-closed set (§3) cites its landings, foregrounding the Float correction — **done**.
3. The ranked plan (§4) is grouped by layer as two coupled tracks toward zero hand-port — **done**.
4. The attestation (§5) is honest about dry-round count and residual uncertainty — **done**.
5. The `enb` backlog (§8) lists issues to **file** (the integrator owns `issues.yaml`) — **done**.
6. **Ratification (maintainer):** confirm the status/layer classifications, the already-closed set, and
   authorize the `enb` backlog. Status stays **Draft** until then — the reasoner does not self-ratify
   (house rule #3 / #4).

---

## §7 Doc-Index + changelog note (FLAGGED up, not applied here)

`docs/Doc-Index.md`, `CHANGELOG.md`, and `tools/github/issues.yaml` are **integration-owned** (the
concurrent-PR pattern: leaves FLAG, the integrating parent applies once). This DN therefore **does not**
edit them. **FLAG to the integrator:** add a Design-Notes row for `DN-99 — Surface-Gap Closure Register
(Draft)` to `Doc-Index.md`, a `CHANGELOG.md` entry, and file the §8 backlog.

---

## §8 Proposed `enb` backlog — issues to FILE (do NOT edit `issues.yaml` here)

De-duplicated from the 15 `DN? = yes` rows (signed #26/#44 collapse into one; the two `?` facets #52/#60 collapse into one; char/width #45 subsumes the signed/char sub-questions). Each is a **new Draft
DN + tracking issue** for the integrator to file. All touch the **cloud semcore lane** unless noted —
**coordinate with the cloud CC session** (M-1013 SCC), unfrozen-actionable, not deferred-by-freeze.

| Proposed | Title | Layer | Closes rows | Coll. | Pri | Notes |
|---|---|---|---|---|---|---|
| ENB-1 | Cross-nodule symbol resolution + runtime execution | runtime-`enb` | #41, #13, #1, #16, #18 | HIGH | P1 | rides M-982; reuse check-time import registry as runtime link table; coordinate M-1013 |
| ENB-2 | `?` try-operator grammar sugar + desugar | grammar-`enb` | #60, #52 | HIGH | P1 | `Question` token + postfix; no new kernel semantics; differential ≡ hand-match |
| ENB-3 | impl-level generic-parameter slot | grammar-`enb` | #63 | HIGH | P2 | `parse_impl_item` `type_params`; lifetime-erase decision |
| ENB-4 | per-constructor visibility seal | grammar-`enb` | #37, #69 (field sub-case) | HIGH | P2 | design with DN-53 §B.5; resolve §B.6 Q1 |
| ENB-5 | transcendental + ε/δ float numerics | runtime-`enb` | #42 | HIGH | P2 | ADR-gated (parallel ADR-040); kernel-vs-libm routing; ε/guarantee matrix |
| ENB-6 | platform-width + char-type decision (subsumes signed) | runtime-`enb`/idiom | #45, #26, #44 | HIGH | P2 | prefer canonical `Binary{N}` idiom; resolves M-874 |
| ENB-7 | host-effect real-syscall registry + fixture differential | runtime-`enb` | #79, #88 | HIGH | P2 | grant real `wild:` ops; sandbox/golden-trace for non-determinism |
| ENB-8 | runtime-tier value-semantics mapping (post-Phase-7) | runtime-`enb` | #56 | HIGH | P3 | `hypha`/`mesh` lex+check+elab; no user-facing Arc/Mutex/Box |
| ENB-9 | macro expand-first transpiler pre-pass (toolchain DN) | transpiler | #51 | none | P2 | `cargo expand` vs vendored; optional bulk lever |
| ENB-10 | statement-sequencing (`let _`) + record-update-mutation split | transpiler + grammar-`enb` | #89 | low/HIGH | P2 | Part 1 tr-only; Part 2 (mutation→functional) separate DN-gated |
| ENB-11 | signed/format ratification notes (idiom-close) | idiom (docs) | #70 (idiom arm), #26 | none | P1 | Display-composition recipe; float `{:.2e}` prim residual = ENB-5-adjacent |

> **ENB-10 triage update (2026-07-10, DN-106 — append-only, this note's decisions unchanged).** M-1033
> (ENB-10) was triaged per mitigation #14 (verify a stale issue's claim against the codebase before
> implementing). Finding: **both** sub-gaps' LANGUAGE side is already closed at L1 — three-way witnessed
> (L1-eval ≡ elaborate→L0-interp ≡ trampoline-AOT) and pinned as four regression witnesses in
> `crates/mycelium-l1/tests/enablement.rs` (PR #1373) — which resolves the tension between **this
> register's own two ENB-10 tags**: row #89 carries layer `tr` / collision `low`, while the §8 ENB-10
> backlog synthesis (above) carried `transpiler + grammar-`enb`` / `low/HIGH` and deferred Part 2's
> classification to "a separate DN." **DN-106 is that DN**, and resolves it in favour of row #89's
> `tr`/`low`: **M-1033's L1/semcore residual is NIL**; the real residual is entirely transpiler-lane
> (Part 1 `let _` emit for value-producing discarded statements; Part 2 the mutation→functional
> destructure-and-reconstruct rewrite). A `{ ..base, field: v }` record-update literal has no Mycelium
> surface by design (positional constructors, DN-106 §2/§3) and its addition to L1 is rejected (fork B).
> See **DN-106** (Draft).
>
> **ENB-2 gap-routing addendum (2026-07-11, maintainer decision on DN-102 — append-only, this
> register's rows are unchanged).** The maintainer declined to ratify DN-102 (the `?` try-operator
> desugar) as originally drafted, judging that DN-102's **FLAG-try-2** — no `From`-error widening; the
> error-type unification rule is exact-match only, since Mycelium has no error-conversion trait wiring —
> is a genuine **language-surface gap** that should have surfaced in this register's own sweep rather
> than only as a residual FLAG on the implementing note. It is recorded here, against the **ENB-2** row
> above, as an explicit sub-gap: **error-conversion-trait wiring for `?`-widening** — no register row
> number is reassigned (append-only; a future sweep may mint one if warranted). Tracked for
> implementation-readiness alongside DN-102's required second research pass, filed as **M-1049**
> (`doc_refs: corpus:DN-102, corpus:DN-99`). See DN-102's "Ratification / Maintainer decision
> (2026-07-11)" section for the maintainer's exact words.
>
> **ENB-2 FLAG-try-2 RESOLUTION addendum (2026-07-11, DN-102 second research pass / M-1049 —
> append-only, this register's rows are unchanged).** The second research pass (DN-102 §SP.2) **resolves**
> the FLAG-try-2 sub-gap and **refines this routing note's wording**: the sub-gap is **not**
> "error-conversion-**trait** wiring … pending." Under the ratified principles (DN-106 gap-closure
> default + native-solution mapping; DN-109 D13), the **implicit, trait-dispatched `From` widening is a
> deliberate exclusion** — the error-channel analogue of an inferred `swap` (a black-box conversion the
> `?` site does not name; house rule #2 / G2 / VR-5). The underlying problem (propagate a sub-error
> across error types) is **mapped to Mycelium's native solution: explicit conversion** via the landed
> `map_err[A,E,F](r, f: E => F) => Result[A,F]` (`lib/std/result.myc:39`; `Empirical`) — `map_err(e,
> conv)?`, visible and never-silent. An **explicit-conversion `?` sugar** (e.g. `e ?with conv`) that
> lowers mechanically (`Err($f) => Err(conv($f))`) is a **deferred, additive** follow-up (YAGNI — the
> combinator already expresses it), and is **not** implicit `From`. The transpiler **flags-with-
> suggested-native-idiom** (DN-109 D6 / M-1045) rather than bare-refusing. **No error-conversion-trait
> subsystem is adopted or needed.** So the ENB-2 sub-gap is **resolved (deliberate-exclusion-of-implicit,
> plus native explicit-conversion), not open-pending.** The maintainer confirms this wording on ratifying
> DN-102 (M-1049).
>
> **ENB-2 wording CONFIRMED (2026-07-10, on DN-102 ratification — append-only, this register's rows
> are unchanged).** DN-102 is ratified → Accepted (maintainer, 2026-07-10), adopting SP.5 Rank 1 in
> full, including the SP.2 FLAG-try-2 resolution recorded in the addendum immediately above. The
> maintainer confirms this addendum's wording stands as the ENB-2 sub-gap's resolved (not open-pending)
> status. See DN-102's "Ratification (maintainer, 2026-07-10)" section for the full ratification
> record; **M-1049** is closed on this basis (see `tools/github/issues.yaml`).

**Transpiler-only closures needing NO new DN** (file as ordinary tracking issues under M-1006 / the trx2
ladder): #72 string-literal pattern, #85 byte-literal/byte-string, #71 tuple-let-destructure, #21
field-access desugar, #29 struct-variant pattern, #64 turbofish-never-silent, #50 impl/derive emit
policy, #10 payload-variant census refresh, #24 struct-literal `..rest`.

---

## §9 Changelog

- **2026-07-11 (later same day) — DN-123 cross-reference addendum (integration close-out).** Rows
  **#4** (struct-def, closed) and **#9** (named-field-drop, closed) now cross-reference **DN-123**
  (`docs/notes/DN-123-Records-Named-Fields-Surface-Lever.md`) as the P2 design note working the
  records/named-fields lever forward — the residual there is **faithfulness** (dropped field names)
  plus the **self-hosted `.myc` surface** (DN-119 L3-G1 struct-pattern grammar), **not**
  expressibility; rows #4/#9's own `closed` status and tally are unchanged (both remain correctly
  closed at the transpiler-emission level DN-123 §2 verified). Append-only — no row content edited,
  a dated cross-reference note only; DN-99 itself stays **Draft**.
- **2026-07-10** — **ENB-2 wording CONFIRMED addendum** (§8): DN-102 is ratified → Accepted
  (maintainer, 2026-07-10), and the maintainer confirms the 2026-07-11 FLAG-try-2 RESOLUTION addendum's
  wording stands. **M-1049** closed on this basis. Append-only — the register's rows and tally are
  unchanged; DN-99 itself stays **Draft** (this note is not itself ratified by DN-102's ratification).
- **2026-07-11** — **ENB-2 FLAG-try-2 RESOLUTION addendum** (§8): the DN-102 second research pass
  (M-1049) resolves the FLAG-try-2 sub-gap and refines the prior routing note — implicit trait-`From`
  widening is a **deliberate exclusion** (inferred-conversion black box, DN-109 D13), mapped to the
  native **explicit `map_err(e, conv)?`** solution (`lib/std/result.myc:39`), with an explicit-
  conversion `?` sugar as a deferred additive follow-up; **no error-conversion-trait subsystem** is
  adopted. Append-only — the register's rows and tally are unchanged; this adds a dated cross-reference
  note only, pending the maintainer's DN-102 ratification.
- **2026-07-11** — **ENB-2 gap-routing addendum** (§8, after the backlog table): the maintainer's
  DN-102 non-ratification decision routes **FLAG-try-2** (`From`-error widening) into this register as
  an explicit sub-gap of the ENB-2 row, tracked via **M-1049**. Append-only — the register's rows and
  tally are unchanged; this adds a dated cross-reference note only.
- **2026-07-10** — DN-99 created (**Draft**). Synthesized the spw / RFC-0031 surface-gap closure register
  (92 gaps), the two-track ranked closure plan, the already-closed set (Float-lesson corrections), the
  adversarial-completeness attestation, and the `enb` backlog. `Empirical` where cited against the tree
  (dev `6d906b76`); `Declared` for unratified closure designs. Authored READ + DN only — no edit to
  `lib/compiler/**`, `crates/mycelium-l1/**`, `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md`
  (integration/cloud-semcore-lane owned; FLAGGED up per §7). Append-only; status advances only by
  maintainer ratification (house rule #3).
- **2026-07-10** — **register correction (mitigation #14, verify-first): rows #72 and #85
  reclassified `tr-only` → `language-enabler`.** Whole-corpus profiling against the real `myc check`
  oracle (DN-34 §8.21) proved the DN-99 Track-B "trivial transpiler-only" B1 label wrong for these two
  literal-pattern closures: **#72** (string-literal `match` pattern) needs an L1 enabler — the checker
  categorically rejects a `match` on a `Bytes` scrutinee — now tracked as **M-1035** (ENB-12); **#85**
  (byte-literal) has a `myc check`-clean transpiler arm *in isolation* but its corpus closure is gated
  on the **ENB-1** unknown-prim symbol table (FLAG-tr-unknown-prim, cross-refed on **M-1024** /
  DN-101), so it is not a standalone transpiler-only win. The prior "add-an-arm / `myc check`-clean"
  DoDs were the stale guesses. Only the two rows' classification + tracking are corrected here
  (append-only spirit — the register's structure is unchanged); see DN-34 §8.21 for the measured basis.
- **2026-07-10** — **row #72 CLOSED (emitted-with-enabler): the M-1035/ENB-12 L1 enabler was
  consumed by the transpiler (PR #1372 → dev).** The trx #72 arm flipped *gapped* → *emitted* — a
  string-literal `match` now lowers to the faithful, `myc check`-clean `match s { "yes" => True, _ =>
  False }` (`&str` → `Bytes`, `true`/`false` → `True`/`False`), but only WITH a wildcard/default arm;
  a defaultless string-literal match still gaps never-silently (an open-`Bytes` non-exhaustive surface
  is check-failing — VR-5/G2). Pinned by `string_literal_pattern_emits_with_l1_enabler`. The measured
  corpus win is small (`checked_fraction` +0.547pp, 6.193% → 6.740%) because most string-`match` arm
  bodies are ownership/identity-conversion no-ops (`.to_owned()`/`.clone()`/…) that PR #1372 now GAPS
  rather than fabricates (the honest never-silent outcome) — the full corpus win awaits the
  conversion-method mapping, filed as **M-1037** (relates to the ENB-1 unknown-prim symbol table /
  FLAG-tr-unknown-prim on **M-1024** / DN-101). Row #72's classification + notes are corrected here;
  the register's structure is unchanged (append-only spirit, house rule #3).
