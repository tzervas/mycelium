# Mycelium Object & Behavior Model + Sigil Scheme — Internal Ground Truth

Repo-grounded research backing a design note on how "objects"/OOP render in Mycelium and the
sigil-category scheme. All citations are `file:line` against `/home/user/mycelium` @ `main`.
Honesty: source + Enacted RFCs are ground truth; tags below are per-claim.

---

## 1. Data / Record / ADT Layer — `type` + `Construct` + flat `Match`

**There is no `struct`/`enum` keyword.** Both surface through a single keyword: **`type`**.

- The lexer reserves `type` (`Tok::Type`) — `crates/mycelium-l1/src/token.rs:83-84,318`. There is
  **no** `struct`, `enum`, `class`, `object`, or `record` keyword anywhere in `keyword()`
  (`token.rs:292-355`) — confirmed by exhaustive read.
- Surface form: `type T = C₁(τ…) | … | Cₙ(τ…)` — a **sum-of-products**. The parser:
  `parse_type_decl` reads `type Name <params>? = ctor ( | ctor )*` (`parse.rs:444-459`); each
  `parse_ctor` is `Name ( '(' field,* ')' )?` (`parse.rs:461-468`). Fields are **positional**
  (`Vec<FieldSpec>`), in declaration order — **no named fields** in v0.
- **`struct`/record = a single-constructor `type`**: `type Point = Mk(Binary{16}, Binary{16})`.
  **`enum`/sum = a multi-constructor `type`**: `type Shape = Circle(..) | Square(..)` (worked
  example, `.claude/memory/lang-lexicon-syntax.md:296`). This is the surface rendering of LR-1
  (`docs/rfcs/RFC-0006-...md:117`).
- Example confirming the surfacing (`lang-lexicon-syntax.md:292-308`).

**L0/kernel representation (the trusted base).** Two nodes carry data
(`crates/mycelium-core/src/node.rs:70-87`):
- `Node::Construct { ctor: CtorRef, args }` — a **saturated** constructor application (WF6); builds
  a product value. `args.len()` == ctor field count, checked above the kernel (`node.rs:69-75`).
- `Node::Match { scrutinee, alts, default }` — a **flat**, single-level, coverage-checked match
  (WF7). Maranget usefulness algorithm (`crates/mycelium-l1/src/usefulness.rs`) checks exhaustiveness
  - redundancy; the nested→flat decision tree stays an untrusted artifact above the kernel
  (`node.rs:76-87`; RFC-0011 §4.4).
- These are L0 nodes as of RFC-0011 **Enacted r3** (`docs/rfcs/RFC-0011-...md:5`,
  `node.rs:12-13`). The 5-node L0 (`Const|Var|Let|Op|Swap`) grew to add `Construct|Match` (r3) and
  `Lam|App|Fix|FixGroup` (r4/r5) — `node.rs:88-139`.

**Data declarations live in a content-addressed registry `Σ`, NOT in the term grammar**
(`crates/mycelium-core/src/data.rs:1-9`; the GHC-Core/Lean/Coq/Unison convergence).
- A `CtorRef` is `#T#i` = declaration content-hash ‖ constructor index (`data.rs:33-40,62-67`).
  Two constructors are the same iff `(decl-hash, index)` agree — **names play no part** (ADR-003,
  `data.rs:34-35`).
- **Equality / value identity = structural / content-addressed.** A declaration is content-addressed
  over its **α-normalized structure**: constructor **order is significant**, field types incl. their
  `Repr` are significant, **names are not identity** (`data.rs:11-17`). Proven by tests:
  `identity_is_structural_not_nominal` (`data.rs:491-514` — `Nat` ≡ `Peano`),
  `constructor_order_is_identity_bearing` (`data.rs:516-534`), `field_repr_is_identity_bearing`
  (`data.rs:536-552`). Self-recursive types hash their own occurrence as a cycle **placeholder**
  (Unison scheme; `data.rs:14-17`); mutual recursion content-addresses canonically/name-independently
  (`data.rs:286-313`, test `data.rs:574-615`).
- Runtime `Value` equality is `#[derive(PartialEq)]` over `(repr, payload, meta)`
  (`crates/mycelium-core/src/value.rs:133-138`) — structural.

**KEY CONSTRAINT for the object model — `FieldSpec` is `Repr | Data` only**
(`data.rs:98-104`): a field is either a representation-typed value or a reference to another data
declaration. **There is NO function-typed field.** This is exactly why the literal RFC-0019 §4.5
runtime-dictionary `Construct` (a record of method *values*) is **not v0-expressible** (RFC-0019
changelog, `RFC-0019-...md:799,808-811`). Any "object = record of closures" encoding is therefore
**greenfield**, gated on an abstract-parameter `FieldSpec` trusted-core change (a separate ADR).

Guarantee tag: **Built / Enacted** (RFC-0011 r3 enacted; tests green). `Exact` for the
identity-is-structural claim (it is mechanically tested).

---

## 2. Trait / Behavior / Polymorphism Layer — RFC-0019 (Enacted, staged)

`trait` is the typeclass keyword (conventional; `guild` declined — DN-02 §7). Elaboration is
**dictionary-passing in the design, monomorphization + static resolution in the implementation.**

### What's there (surface + checker)
- `trait Name<A> { fn sig … }` — `Tok::Trait` active (`token.rs:85-86,319`); parsed by
  `parse_trait_decl` → a list of **`fn_sig`s only** (`parse.rs:471-492`).
- `impl Trait<args> for T { fn … = body }` — `Tok::Impl` active (`token.rs:87-91,320`);
  `parse.rs:292-293,375-376`.
- Bounded generics `fn f<T: Cmp + Ord<T>>(…)` with self-bound sugar `T: Cmp ≡ T: Cmp<T>`
  (`parse.rs:511-533,630-631`; RFC-0019 §4.1).
- Coherence: **global uniqueness per `(trait, type-head)` + orphan rule**, phylum-wide as of M-662
  (`RFC-0019-...md:813-820`). Instance key erases width/shape (`type_head`,
  `checkty.rs:221-240`); a blanket `impl … for T` over a bare `Ty::Var` is refused explicitly
  (`checkty.rs:225-238`). All refusals never-silent (G2).
- Elaboration = **monomorphization with static instance resolution** (`mono.rs`), reified in a
  queryable EXPLAIN record — NOT the literal runtime dictionary (`RFC-0019-...md:800-812`). Honest
  tradeoff recorded: identity *fragments across specializations* (mangled names are the record).
- `Repr`-polymorphism restriction (LR-5) holds at the bound level; guarantee-indexed methods (LR-6)
  are stage-0 `Declared` contracts (RFC-0019 §4.6/§4.7).

### The trait data structure confirms the v1 scope
`TraitInfo` (`checkty.rs:192-204`) carries **only** `name`, `params: Vec<String>`, and
`sigs: Vec<FnSig>`. There is **NO super-trait field and NO default-method body field**. The parser
reads only `fn_sig`s in the trait body (`parse.rs:475-480`) and a `fn_sig` is a signature with **no
`= body`** (`parse_fn_sig` → `parse_sig_tail`, `parse.rs:489-533` — contrast `parse_fn_decl`
which requires `Tok::Eq` + a body, `parse.rs:497-509`). Grep for `super.?trait|default method|default
body` across `crates/mycelium-l1/src` returns **zero matches**.

### BUILT vs DEFERRED — precise table (per RFC-0019 + source)

| Feature | Status | Evidence (honesty tag) |
|---|---|---|
| `trait T<A> { fn sig }` decl | **Built (type-checks)** | `parse.rs:471-492`; `checkty.rs:192-204`. `Empirical` (tested checker). |
| `impl T<args> for U { fn = body }` | **Built (type-checks)** | `parse.rs:292,375`; `checkty.rs:206-219`. `Empirical`. |
| Trait bounds `fn f<T: A + B>` | **Built** | `parse.rs:511-533,630`; self-bound sugar. `Empirical`. |
| Coherence: orphan + global-uniqueness + reject-overlap | **Built (phylum-wide)** | `checkty.rs:221-240`; `RFC-0019-...md:813-820`. Result tag **Declared-with-argument** (not machine-checked; `RFC-0019-...md:6,863`). |
| Monomorphization + static dispatch (elaboration that RUNS) | **Built** | `mono.rs`; `RFC-0019-...md:800-812`. `Empirical`. |
| **Super-traits** (`trait Ord<A>: Eq<A>`) | **DEFERRED — not in v1** | RFC-0019 *designs* the dict layout (§4.3, `RFC-0019-...md:305-308`) but `TraitInfo` has no super-trait field (`checkty.rs:192-204`) and `parse_trait_decl` parses no `:` super-bound (`parse.rs:471-487`). **Not parsed, not checked.** Greenfield-to-implement. |
| **Default method bodies** | **DEFERRED — not in v1** | Trait body is `fn_sig*` (signatures only); a sig has no `= body` (`parse.rs:475-480,489-492`). Every `impl` must supply **every** method (exact method-set conformance, `lang-lexicon-syntax.md:337`). **Not in the grammar.** Greenfield. |
| Runtime-dictionary `Construct` form (dynamic dispatch §4.5) | **DEFERRED (normative target)** | `FieldSpec` is `Repr\|Data` only — no function field (`data.rs:98-104`); literal dict record not v0-expressible (`RFC-0019-...md:799,808-811`). Gated on a trusted-core `FieldSpec` ADR. |
| Associated types (`type Output = …`) | **DEFERRED → v2** | `RFC-0019-...md:663-665,677,861`. Greenfield. |
| Multi-parameter traits (`trait Coerce<A,B>`) | **DEFERRED → v2** | `RFC-0019-...md:658-662,677,861`. Stage-1 is single-parameter (`checkty.rs:192-200`). |
| Guarantee-indexed methods (LR-6) | **Stage-0 Declared only** | `RFC-0019-...md:482-503`. The `@ g` annotation parses; static grading is the future grading RFC. `Declared`. |
| Repr-polymorphism (LR-5) | **Restriction-set Accepted; bodies rejected** | `RFC-0019-...md:397-447,858-859`. `UnresolvedReprPolymorphism` on violation. `Declared-with-argument`. |
| `grow Trait for T` (derive) | **DEFERRED** | `grow` reserved-not-active (`token.rs:71-73,314`); derivation rules unspecified (`RFC-0019-...md:516-532,655-657`). |

**Bottom line for inheritance-emulation primitives available TODAY:** trait + impl + bounded
generics give *interface polymorphism* (Rust-trait-shaped). The two ergonomic-inheritance levers —
**super-traits** (interface refinement) and **default method bodies** (shared behavior) — are
**BOTH absent in v1** (designed in RFC-0019 §4.3 for super-traits, but unimplemented; default bodies
not even designed in the grammar). There is **no `dyn`/trait-object dynamic dispatch** (RFC-0019
§6.2: monomorphization is the only path). So "emulate inheritance" today = **manual composition**:
embed a value-typed field of the "base" `type`, and re-declare/forward methods by hand. No
delegation sugar exists (see §6).

---

## 3. Encapsulation + the Acyclicity Constraint

### Encapsulation / visibility — `pub` over nodules
- Visibility is **cross-nodule**, Rust-like: top-level `fn`/`trait`/`type` are
  **private-to-nodule by default**; `pub` exports the **name** to the other nodules of the phylum
  (`crates/mycelium-l1/src/ast.rs:37-54`; `token.rs:77-82,317`; M-662). The `Vis` enum is
  `Private | Pub` (`ast.rs:42-54`).
- `pub` is **pub-blind for coherence**: it gates the `use` namespace, not coherence authority, which
  spans the whole phylum (`RFC-0019-...md:818`).
- **Opaque types / private fields are NOT yet a separate mechanism.** Because fields are positional
  and a `type` is exported as a whole name, v0 has no field-level visibility — encapsulation today is
  *nodule-level* (export the type + its constructor/accessor functions, keep helpers private). The
  `wild`/FFI body is "trusted/opaque" (`ast.rs:501`) but that is the unsafe floor, not data hiding.
  Field-private/opaque-type encapsulation is **greenfield**.

Guarantee tag: **Built** for nodule-level `pub` (M-662, `Empirical`); **greenfield** for
field-level/opaque encapsulation.

### Acyclicity (LR-9) — the key OOP constraint
- **LR-9 is "memory safety by construction, leaks structurally excluded"** — value semantics
  (immutable values, no aliased mutable state), no manual alloc/free, automatic deterministic
  reclamation (Perceus-style refcounting + region inference). The *guarantee*: "in safe Mycelium a
  memory leak is not expressible." (`docs/rfcs/RFC-0006-...md:125`, the LR-9 row.)
- **The acyclicity is the value-graph consequence, and it is an explicitly-flagged open mechanism,
  not yet a hard rule in the corpus.** RFC-0006 Q8 records it directly: "Perceus-style RC handles
  cycles by … restricting them … *does the language forbid value cycles, detect them, or fall back
  to a tracing pass?*" — listed as **open** (`docs/rfcs/RFC-0006-...md:246-254`). So:
  - **Immutability (LR-8/value-semantics) is the load-bearing fact** that makes cyclic *runtime
    value* graphs hard to even construct (you cannot mutate a field to point back at a parent — there
    is no aliased mutable state; `RFC-0006-...md:124-125`). This is the corpus statement the
    object-model note should cite for "no parent↔child back-pointers / observer back-refs."
  - **Recursive *types* are explicitly allowed** (self- and mutual-recursion in the data registry,
    `data.rs:14-25`) — what's excluded is a cyclic *value* (a value that contains itself), which an
    immutable, bottom-up `Construct` cannot build.
  - The *resolution* of value cycles (forbid / detect / trace) is **Q8 open** (`RFC-0006-...md:251`).
- Practical consequence for OOP: the classic object-graph patterns that *require* mutable
  back-references — bidirectional parent/child links, the Observer pattern's subject→observer
  back-list, doubly-linked structures — **cannot be expressed as value cycles**. They must be
  re-expressed acyclically (IDs/indices into an owning collection, one-directional ownership, a
  re-derived view).

Guarantee tag: immutability/value-semantics = **Built/Enacted** (`Exact` for "no aliased mutable
state" — it's structural). The *acyclicity-forbidding mechanism* = **flagged open (Q8)**, tag
`Declared`. Do not claim the compiler *rejects* a value cycle today — it isn't reachable to build
one in safe code, but the explicit check is unwritten.

---

## 4. Complete Current Sigil / Symbol Lexicon

Source of truth: `crates/mycelium-l1/src/token.rs` (the `Tok` enum + `keyword()`) and
`crates/mycelium-l1/src/lexer.rs` (the `run()` dispatch). Comments are `//` (line comments, lexer
trivia; `lexer.rs:393`), so **`#` is free**. The lexer's default arm is a **never-silent**
`unexpected character` error (`lexer.rs:200-205`) — every free sigil hits it.

### TAKEN — single/compound glyph tokens (each with `lexer.rs` lex site + `token.rs` defn)

| Glyph | Token | Meaning | Cite |
|---|---|---|---|
| `(` `)` | `LParen`/`RParen` | grouping, call/param lists | `token.rs:163-165`; `lexer.rs:152-153` |
| `{` `}` | `LBrace`/`RBrace` | blocks, repr size `Binary{8}`, effect set, trait/impl/match bodies | `token.rs:166-169`; `lexer.rs:154-155` |
| `[` `]` | `LBracket`/`RBracket` | **list literals only** today (type-args move here per DN-31/D7) | `token.rs:170-173`; `lexer.rs:156-157` |
| `<` | `LAngle` (or trit-lit) | type-args open; 1-char lookahead → balanced-ternary literal `<+0->` | `token.rs:174`; `lexer.rs:194,306-337` |
| `>` | `RAngle` | type-args close | `token.rs:175`; `lexer.rs:158` |
| `@` | `At` | **guarantee annotation** `T @ Exact` (LR-6) | `token.rs:178-179`; `lexer.rs:163,217-237` |
| `@std-sys` | `AtStdSys` | atomic nodule-header FFI-floor marker (M-661) — whole-word only | `token.rs:180-188`; `lexer.rs:217-237` |
| `:` | `Colon` | type ascription, bound, `swap`/header labels | `token.rs:189-190`; `lexer.rs:164` |
| `,` | `Comma` | separator | `token.rs:191-192`; `lexer.rs:165` |
| `.` | `Dot` | path/field projection | `token.rs:193-194`; `lexer.rs:166` |
| `\|` | `Pipe` | sum-type ctor separator **and** bitwise `bor` (RFC-0025) | `token.rs:195-196,247-250`; `lexer.rs:171,285-293` |
| `\|\|` | `PipePipe` | logical `or` | `token.rs:245-250`; `lexer.rs:287` |
| `+` | `Plus` | trait-bound separator `T: A + B` **and** infix `add` | `token.rs:197-201`; `lexer.rs:181` |
| `-` | `Minus` | infix `sub` / unary `neg` | `token.rs:202-204`; `lexer.rs:196,260-271` |
| `*` | `Star` | glob import tail `use a.b.*` **and** infix `mul` | `token.rs:205-209`; `lexer.rs:185` |
| `/` | `Slash` | infix `div` (`//` is a comment, consumed as trivia) | `token.rs:210-213`; `lexer.rs:188` |
| `%` | `Percent` | infix `rem` | `token.rs:214-216`; `lexer.rs:190` |
| `^` | `Caret` | infix bitwise `xor` | `token.rs:217-219`; `lexer.rs:191` |
| `&` | `Amp` | infix bitwise `band` | `token.rs:220-222`; `lexer.rs:193,274-282` |
| `&&` | `AmpAmp` | infix logical `and` | `token.rs:223-225`; `lexer.rs:276` |
| `=` | `Eq` | binder / definition glyph | `token.rs:226-227`; `lexer.rs:195,244-258` |
| `==` | `EqEq` | infix `eq` | `token.rs:228-231`; `lexer.rs:252` |
| `!` | `Bang` | effect-set opener `!{…}` **and** unary `not` | `token.rs:236-241`; `lexer.rs:176,296-304` |
| `!=` | `BangEq` | infix `ne` | `token.rs:242-244`; `lexer.rs:298` |
| `->` | `Arrow` | function-type / return arrow | `token.rs:232-233`; `lexer.rs:262-266` |
| `=>` | `FatArrow` | match-arm / `for` body arrow | `token.rs:234-235`; `lexer.rs:247-250` |

Literal prefixes: `0b…` binary (`lexer.rs:197,339-352`); `<+0->` trit (`lexer.rs:306-337`); bare
decimal int (`lexer.rs:198`). Operators above are RFC-0025/M-705 surface sugar that desugars to
named ops.

### FREE — sigils that hit the never-silent `unexpected character` path (`lexer.rs:200-205`)

| Glyph | Status | Note |
|---|---|---|
| `$` | **FREE** | no match in `run()` dispatch (`lexer.rs:151-206`) → error |
| `#` | **FREE** | comments are `//` not `#` (`lexer.rs:393`); `#` only appears inside `CtorRef::Display` strings, never lexed |
| `~` | **FREE** | not dispatched → error |
| `?` | **FREE** | not dispatched → error |
| `` ` `` (backtick) | **FREE** | not dispatched → error |
| `\` (backslash) | **FREE** | not dispatched → error |

Confirmed: the expected free set `$ # ~ ? backtick \` is exactly correct. `@` is **taken**
(guarantee annotation), `!` is **taken** (effect/not), so they are NOT available. Note `<>` is
slated to be freed for comparison/shift operators with type-args moving to `[]` (DN-31/D7,
advisory — not yet landed; see §6).

Guarantee tag: **Exact** for the taken/free split — verified by exhaustive read of the lexer
dispatch (`lexer.rs:151-206`) and `keyword()` (`token.rs:292-355`).

---

## 5. Composition-over-Inheritance + Delegation; Grammar Home for a Sigil Scheme

- **Delegation / forwarding sugar: GREENFIELD.** Grep for `delegat|forward|composition over
  inheritance` across `docs/` and `crates/mycelium-l1/src` finds the *principle* (CLAUDE.md house
  rule) but **no surface construct**. No `delegate`/`forward`/`use … as` member-forwarding exists;
  there is no keyword for it in `token.rs`. Composition today = a value-typed `Data` field
  (`data.rs:98-104`) + hand-written forwarding `fn`s. The fungal lexicon has reserved-not-active
  surface words `consume`/`grow` (`token.rs:68-73`) but neither is delegation.
- **Grammar home for any new sigil/delimiter scheme = DN-31 + RFC-0030 + RFC-0025, the "epic #27"
  []-for-type-args wave.**
  - **DN-31** (`docs/notes/DN-31-Delimiter-and-Operator-Deconfliction.md`) — advisory, Draft. The
    maintainer's decided scheme (2026-06-24, reconfirmed 2026-06-25 D7): free the triple-loaded `<>`
    to comparison/shift operators, move type/size args onto **`[]`**, trit literals to a `0t` prefix
    (`DN-31:9,22,25,37,43,101-102`). It **supersedes RFC-0019 §4.1** for the type-arg bracket and
    **updates RFC-0030/RFC-0025** (`DN-31:7,55-60`).
  - **RFC-0030** — the concrete L3 surface grammar (ratification home for the bracket move,
    `DN-31:58-59`).
  - **Epic #27** (task list) — "[]-for-type-args grammar wave (D7) — supersede RFC-0030 `<>`
    direction" — the implementation vehicle.
  - This is where a sigil-category scheme (e.g. assigning the free `$ # ~ ? \` `` ` `` glyphs to
    object/delegation/projection categories) would land normatively.

Guarantee tag: **greenfield** for delegation surface (`Declared` — absence verified by grep);
DN-31 scheme is **Draft/advisory** (`Declared`).

---

## 6. What an Object/Behavior Model + Sigil Scheme Can Build On — tagged

| Capability needed | What exists today | Tag |
|---|---|---|
| Product/record type | single-ctor `type T = Mk(τ…)` (positional fields) | **Built/Enacted** (`Exact`) |
| Sum/variant type | multi-ctor `type T = A \| B` | **Built/Enacted** (`Exact`) |
| Structural value identity | content-addressed registry; names ≠ identity | **Built/Enacted** (`Exact`, tested) |
| Pattern dispatch (vtable analogue) | flat `Match`, exhaustiveness-checked | **Built/Enacted** (`Empirical`) |
| Interface polymorphism | `trait` + `impl` + bounded generics, coherent | **Built/Enacted** (`Empirical`; coherence `Declared-with-argument`) |
| Static dispatch that runs | monomorphization + static resolution (`mono.rs`) | **Built** (`Empirical`) |
| Nodule-level encapsulation | `pub` over nodules, private-by-default | **Built** (`Empirical`) |
| Acyclicity / no back-pointers | immutable value semantics (cyclic values unbuildable) | **Built** invariant; *explicit forbid/detect mechanism* **flagged-open Q8** (`Declared`) |
| **Super-traits** (interface refinement) | designed in RFC-0019 §4.3, **not implemented** | **DEFERRED** (greenfield to build) |
| **Default method bodies** (shared behavior) | **not in grammar** | **DEFERRED/greenfield** |
| Dynamic dispatch (`dyn`/trait objects) | none — monomorphization only | **DEFERRED (normative target)**; needs `FieldSpec` function field |
| Object = record-of-closures | **not expressible** — `FieldSpec` is `Repr\|Data` only | **Greenfield** (trusted-core ADR gated) |
| Field-level visibility / opaque types | none (nodule-level only) | **Greenfield** |
| Delegation / forwarding sugar | none | **Greenfield** (DN-31/RFC-0030/epic#27 grammar home) |
| Associated types / multi-param traits | deferred to v2 | **DEFERRED → v2** |
| Free sigils for a scheme | `$ # ~ ? \` `` ` `` all free, never-silent | **Built** lexer capacity (`Exact`) |

**Honest synthesis.** The object model can build firmly on Mycelium-as-it-is for *data* (sum-of-
products `type` + content-addressed structural identity) and *interface polymorphism* (traits +
impls + coherent bounded generics, monomorphized). The OOP-shaped gaps are all on the *behavior-
sharing and reference* axes: no super-traits, no default bodies, no dynamic dispatch, no
record-of-closures objects, no delegation sugar, no field privacy — and the acyclicity that defines
the constraint envelope is a value-semantics *consequence* (built) whose explicit enforcement
mechanism is still an open question (Q8). A sigil scheme has clean room: `$ # ~ ? \` `` ` `` are free
and the grammar deconfliction wave (DN-31/RFC-0030/epic#27) is the place to spend them.
