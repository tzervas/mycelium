# Design Note DN-80 — The Reject-Ledger: an Exhaustive, Never-Silent Refusal Inventory (DN-56 condition 1)

| Field | Value |
|---|---|
| **Note** | DN-80 |
| **Status** | **Accepted** (2026-07-02) — an inventory note: it enumerates and audits the kernel's *current* reject surface. It has no forward decision to ratify beyond "this is what the kernel refuses today, checked" — `Accepted` records that the audit is complete and its regression guard is wired, per DN-56 condition 1's own Definition of Done. |
| **Feeds** | **DN-56** condition 1 (§5.2/§2.2 — the reject-ledger completeness gate) and **DN-76**'s scorecard row 1 (`OPEN`, owned by **M-959**) — this note is the closing deliverable for both. |
| **Extends** | **DN-56** (`Kernel-Completeness-And-Freeze-Criterion`) §2.2/§5.2 — this is the ledger that condition calls for. Append-only: DN-56 is not modified; this note supplies the artifact it names. |
| **Date** | July 2, 2026 |
| **Task** | M-959 (kickoff `frz`, Lane A) |

> **Posture (transparency rule / VR-5 / G2).** Every count and category below is **`Empirical`** — a
> checked, mechanical inventory of the reject-producing source at `dev` tip `41ec4fe`
> (2026-07-02), not a semantic proof that these are *all possible* forbidden programs (that would
> require a completeness theorem over the grammar + type system, which this note does not claim).
> What it *does* claim, checked: every reject-producing code path found by the audit below is
> represented by a ledger row, and a regression guard (§8) fails the moment a new reject path
> appears in the audited files without a matching row. Where the audit method is a **line/regex
> heuristic over source text** (§4, §8) rather than a semantic analysis, this note says so plainly
> (mirrors `docs/api-index/`'s own stated posture: "an Empirical/Declared line/regex heuristic —
> source is ground truth").

## 1. Scope and method

DN-56 §2.2 defines the criterion: *"every construct the language forbids is an explicit, named
refusal (a `CheckError`/parse refusal/reject-corpus entry), never an accept-by-omission."* DN-76 §3.1
scored this **OPEN**: the parse-level reject corpus is complete and self-policing, but "checker-level
refusals are explicit but not ledgered... no unified ledger... no regression guard." This note supplies
both.

**What "the kernel rejects" spans** (four strata, each with a distinct enforcement mechanism):

1. **Parse-level** (`mycelium-l1` lexer/parser) — a `ParseError` at a source position. Already
   ledgered and self-policing via the `reject/` conformance corpus + `REJECT_EXPECTED` table
   (`crates/mycelium-l1/tests/conformance.rs`) — **read-only for this leaf** (a concurrent M-915 leaf
   owns the L1 frontend). §3 below re-states it as ledger rows for a *single* cross-stratum table,
   without editing the frontend.
2. **Check-level** (`mycelium-l1`'s `checkty`/`grade` passes) — a `CheckError { site, message }`: a
   free-text refusal (no closed reason-code enum), constructed at ~200 call sites across
   `checkty.rs`/`grade.rs`. §4 below groups these into **construct families** (the ledger's unit of
   row, since the underlying type carries no enum to enumerate 1:1) and pins the exact call-site count
   per family so growth is checkable.
3. **Ambient-resolution-level** (`mycelium-l1::ambient`) — a closed `AmbientError` enum (5 variants).
   §5 below ledgers it 1:1 by variant.
4. **Runtime/interpreter-level** (`mycelium-interp`) — a closed `EvalError` enum (17 variants) and the
   kernel well-formedness `WfError` enum (`mycelium-core`, 7 variants) it wraps. §6/§7 ledger both
   1:1 by variant — the strongest form of this ledger, since a closed Rust enum is mechanically
   diffable.

**Audit basis:** `dev` tip `41ec4fe` (2026-07-02). Every count below was obtained by `grep -c` over the
named files (reproduced exactly by the regression guard, §8) — never hand-estimated.

## 2. Read-only inputs (this leaf owns none of these files)

Per the M-959 spawning directive, this leaf treats the L1 frontend and `mycelium-interp/src/prims.rs`
as **read-only** (a concurrent M-915 leaf edits `token`/`lexer`/`parse`/`checkty`). Every file cited
below was *read*, never edited, by this leaf:

`crates/mycelium-l1/src/{lexer,parse,checkty,grade,ambient,error}.rs`,
`crates/mycelium-l1/tests/conformance.rs`, `docs/spec/grammar/conformance/reject/*.myc`,
`crates/mycelium-interp/src/lib.rs`, `crates/mycelium-interp/src/budget.rs`,
`crates/mycelium-interp/src/prims.rs`, `crates/mycelium-core/src/lib.rs`.

## 3. Part A — Parse-level rejects (30 rows; the existing corpus, restated)

One row per fixture in `docs/spec/grammar/conformance/reject/` — already ledgered and self-policing
via `REJECT_EXPECTED` (`crates/mycelium-l1/tests/conformance.rs`, read-only for this leaf); this table
just brings it into the cross-stratum ledger so a reader has one place to look.

| # | Construct (fixture) | Reason (the refusal) | Surface alternative |
|---|---|---|---|
| 1 | `01-no-nodule-header` — a program with no `nodule` header | every program must open with a `nodule` header (DN-06) | add `nodule <name>;` at the top |
| 2 | `02-swap-missing-policy` — a `swap` with no explicit out-of-range policy | a swap is never silent (RFC-0002) | write the policy explicitly (`swap … via …`) |
| 3 | `03-unclosed-brace` — a `match` missing its closing `}` | malformed brace nesting is a parse error, not a recovery | close every `{` |
| 4 | `04-bad-trit` — `0t` literal whose first glyph is not a trit | a `0t…` literal needs ≥1 trit glyph (RFC-0037 D4) | write at least one `+`/`0`/`-` after `0t` |
| 5 | `05-reserved-word-ident` — a reserved word used as an identifier | reserved words (DN-03 §1) cannot be identifiers | pick a non-reserved name |
| 6 | `06-missing-arrow` — a function header missing its result arrow | a signature needs `=> <ResultType>` (RFC-0037 D4) | write `=> Type` |
| 7 | `07-empty` — an empty source file | same as #1 — no `nodule` header present | add a `nodule` header + ≥1 item |
| 8 | `08-imperative-while` — a `while` loop | `while` is not a Mycelium form (value-semantics only) | use `for`/recursion/`match` |
| 9 | `09-default-missing-paradigm` — `default` with no `paradigm` keyword | `default` must be followed by `paradigm` (RFC-0012) | write `default paradigm <P>;` |
| 10 | `10-phylum-no-nodule` — a `phylum` header with zero `nodule`s | a phylum needs ≥1 `nodule` (M-662) | add at least one `nodule` block |
| 11 | `11-matured-fn-retired` — the old `matured fn` surface | maturation is now declared per-scope (RFC-0007 §4.5/RFC-0017) | use the current `matured`-scope declaration form |
| 12 | `12-runtime-vocab-reserved-not-active` — an unlexed runtime keyword used as a construct | reserved for RFC-0008's runtime model, not yet active | wait for the runtime-tier lexing wave, or avoid the keyword |
| 13 | `13-orphan-hypha` — a `hypha` outside a `colony` | a `hypha` is only valid inside a `colony` (RFC-0008 §4.7 RT7) | wrap it in a `colony { … }` |
| 14 | `14-impl-reserved-ident` — `impl` used as an identifier | `impl` is reserved (DN-03 §1) | pick a non-reserved name |
| 15 | `15-trait-param-bound` — a bound on a `type`/`trait` type-parameter | bounds on `type`/`trait` params are deferred (RFC-0019 §4.1) | move the bound to a function type-parameter |
| 16 | *(no `16-*` fixture — a documented numbering gap, not a hidden reject; the corpus's own self-policing test enumerates directory contents, so a gap here is inert)* | — | — |
| 17 | `17-duplicate-effect` — a duplicate effect name in one `!{…}` annotation | the effect set is never silently de-duplicated (RFC-0014 §4.5) | list each effect once |
| 18 | `18-consume-not-an-item` — `consume <expr>` at item (not expression) position | `consume` is an expression, not a top-level item (DN-03 §1) | move it inside a `fn` body |
| 19 | `19-grow-reserved-not-active` — the old `grow` surface | `grow` is superseded by `derive` (DN-38 §8.1) | use `derive Name for T` |
| 20 | `20-odd-hex-bytes` — a `0x…` byte literal with an odd hex-digit count | a byte is two hex chars; no silent half-byte (RFC-0032 D4) | pad to an even digit count |
| 21 | `21-empty-hex-bytes` — an empty `0x` literal | no hex digits present (RFC-0032 D4) | write ≥1 hex digit |
| 22 | `22-old-arrow-retired` — the old `->` return arrow | the arrow is now `=>`, not `->` (RFC-0037 D4) | write `=>` |
| 23 | `23-old-fn-typeparam-retired` — angle-bracket fn type-params `fn f<T>` | retired; `<` is operator-only now (RFC-0037 D1) | write the type-params in square brackets: `fn f[T]` |
| 24 | `24-old-trait-typeparam-retired` — angle-bracket trait type-params `trait T<A>` | same retirement as #23, on `trait` | write `trait T[A] { … }` |
| 25 | `25-old-angle-trit-retired` — the angle balanced-ternary literal `<+0->` | retired; `<` is operator-only now (RFC-0037 D4) | write `0t+0-` |
| 26 | `26-lower-missing-eq` — a `lower` declaration with no `=` | `lower Name[params]? = <rhs>` is required (DN-54 §3) | add the `=` and the RHS |
| 27 | `27-derive-missing-for` — a `derive` application with no `for` | `derive Name for T` is required (DN-54 §4/DN-38 §8.1) | add `for <Type>` |
| 28 | `28-object-empty-body` — an `object` with an empty body | an `object` needs ≥1 constructor clause (DN-53 §A.3.1) | add a constructor clause |
| 29 | `29-missing-semicolon-terminator` — an item not terminated by `;` | `;` is now a mandatory component terminator (DN-57 §3) | terminate every item with `;` |
| 30 | `30-vec-short-alias-rejected` — `vec{…}` used in type position | `vec` was deliberately NOT made a short repr-keyword alias for `VSA` (it would collide conceptually with `Vec` in `lib/std/collections.myc`) — RFC-0037 D2-b/DN-02; `vec` lexes as a plain identifier, so `vec{…}` is a never-silent parse refusal, not a silent accept as a repr type | use `hvec{…}` (the chosen short alias) or the full `VSA{…}` form |
| 31 | `31-old-le-ge-glyph-retired` — the two-char comparison glyphs `<=`/`>=` in expression position | `<=`/`>=` are retired (M-916; RFC-0037 D1 resolving RFC-0025 §4.2) — only the single-char `<`/`>` glyphs were kept, since the two-char forms carry the most severe type-arg-adjacent parsing ambiguity even after the `[…]` kind-split; `a <= b` lexes as `LAngle` then `Eq`, so the parser reads `a < (= b)` and `=` cannot start an expression — a never-silent parse refusal, never a silent reinterpretation of the old glyph (G2) | use the word-canonical `lte(a, b)`/`gte(a, b)` calls (accept/20) |

## 4. Part B — Check-level rejects (construct families; `CheckError`/`AmbientError`)

`CheckError` (`crates/mycelium-l1/src/error.rs:181`) is `{ site: String, message: String }` — a
free-text refusal, not a closed reason-code enum. There is therefore no 1:1 enum to ledger; instead
this table groups the **220** distinct checker-facing construction/call sites (audited below) into
**41 construct families**, each a coherent "the kernel refuses to accept X" category with a
representative reason and surface alternative. The per-family site count is exact (grep-derived,
reproduced by the regression guard in §8) and **sums to exactly 220** — the completeness check for
this stratum.

**Audited totals (dev `7b933a3`, 2026-07-02; re-audited thrice vs the original `41ec4fe` audit — (a)
`+6` `checkty.rs` sites all in family 8, the M-919 DN-71 Model-S and M-973 DN-54 §10 lower/derive
extension-checker work; (b) M-965's `Fuse` law-checker: `+2` `checkty.rs` family-5
`Fuse`-redeclaration refusals and a new `crates/mycelium-l1/src/fuse.rs` audited file carrying `4`
definition-time semilattice-law rejects, ledgered as new family 40; (c) M-966's `+1` `checkty.rs`
family-6 `via`-delegation ambiguity refusal; (d) RFC-0041 W1 (M-979, 2026-07-03) `+3` recursion-depth
`BudgetError` → `CheckError` refusals — `+1` `checkty.rs` direct construction and `+1` `checkty.rs`
`self.err(` (the shared `Cx::match_budget_err` mapping), `+1` `grade.rs` — the resource-exhaustion
reject category the never-silent depth budget adds):**

| File | Pattern | Raw match count | Non-plumbing reject sites |
|---|---|---|---|
| `crates/mycelium-l1/src/checkty.rs` | `CheckError::new(` / `CheckError::at(` | 103 | 102 (one match, line ~3000, is the shared `Cx::err` helper's own body — plumbing, not a distinct construct) |
| `crates/mycelium-l1/src/checkty.rs` | `self.err(` | 111 | 111 |
| `crates/mycelium-l1/src/grade.rs` | `CheckError::at(` | 3 | 3 |
| `crates/mycelium-l1/src/fuse.rs` | `CheckError::new(` | 4 | 4 |
| **Total** | | **221** | **220** |

| # | Construct family | Representative reason | Surface alternative | Grounding | Sites |
|---|---|---|---|---|---|
| 1 | Generic/width instantiation ambiguity | a type/width parameter would have to take two different values at once | ascribe the value/argument so the instantiation is determined | RFC-0007 §11.3, DN-42 §4, VR-5 | 7 |
| 2 | Unknown type name / type-arity mismatch | the named type doesn't exist, or is applied with the wrong argument count | fix the name, or supply the right arity | RFC-0007 §11.3 | 3 |
| 2b | `for`-iteration shape rejects | the target type isn't linearly recursive (or has no single recursive constructor/element type), or the body's accumulator type disagrees | iterate a linearly-recursive type; match the accumulator type | RFC-0007 §4.8 | 5 |
| 3 | Internal invariant guards (not user-reachable) | a defensive check on the checker's own invariants (e.g. an or-pattern reaching a pass that must have desugared it already) — never triggerable from valid input; kept as a clean refusal rather than a panic | none — these indicate a checker bug if ever hit; report it | G2 (never a panic even on "impossible" input) | 8 |
| 4 | Import/`use`-path resolution | an empty/unqualified/unknown/private/duplicate `use` target | qualify the path, export the name, or de-duplicate the import | M-662 | 6 |
| 5 | Duplicate declaration (type/fn/trait/ctor/method) | the same name is declared twice at one scope, or (M-965) a program redeclares the built-in prelude trait `Fuse` | rename one, remove the duplicate, or don't redeclare `Fuse` | RFC-0007/RFC-0019 (name uniqueness), DN-58 §A | 11 |
| 6 | `object`/`via` composition rejects | the named trait isn't in scope, the field index is out of range, or (M-966) two `via` clauses on one object claim the same trait with no deterministic tiebreak (an `EXPLAIN`-able ambiguity refused never-silently, naming both candidate field indices) | bring the trait into scope; use a valid field index; remove the ambiguous duplicate `via` clause | DN-53 §A.3.2, M-966 | 3 |
| 7 | Totality / matured-scope rejects | a function in a `matured` scope isn't proven total | mark it `thaw fn`, or make it total | RFC-0007 §4.5, RFC-0017 §4.2 | 2 |
| 8 | `lower`/`derive` generative-lowering rejects | a duplicate/self-recursive/cyclic `lower` rule, a `wild` block in its RHS, an RHS that fails the IL-grammar/type check, or (M-919 DN-71 Model S / M-973 DN-54 §10) a sibling-injected derive/lower rule that violates the affine-consume or attachment discipline | rename, break the cycle, remove the `wild` call, or fix the RHS/derive site | DN-54 §3/§4/§10, DN-71 | 14 |
| 9 | Effect-system rejects | an undeclared effect is performed, or an impl's effect annotation doesn't match its trait | declare the effect, or align the annotation | RFC-0014 §4.5 | 4 |
| 10 | Trait/impl coherence & dispatch | an unknown trait, wrong arity, blanket/orphan/overlapping instance, a missing/extra/mismatched method, or no instance found for a concrete type/type-variable | declare the instance, fix the method set, or add the bound | RFC-0019 §4.1/§4.4/§4.5 | 21 |
| 11 | Function-body return-type mismatch | a function's body type disagrees with its declared return | fix the body or the signature | RFC-0007 (edge_mismatch) | 1 |
| 12 | Consume/affine `Substrate` rejects | `consume` on a non-`Substrate` operand, a context-type mismatch, or a double-consume (already moved) | consume a real `Substrate` value exactly once | DN-03 §1, LR-8 | 3 |
| 13 | Name/path/call resolution | a dotted (multi-segment) path/call, or a wholly unknown name | bring the name into scope via `use` and reference its final segment | M-662 | 4 |
| 14 | Constructor application/saturation | a constructor applied with the wrong field count/types, or missing the type arguments context must supply | apply all fields; ascribe the value | RFC-0007 §11.3 (W6 saturation) | 4 |
| 15 | Function-as-first-class-value / HOF | a nullary or under-determined generic function used as a value, a curried-type mismatch, or >1-arg HOF (deferred in stage-1) | apply the function directly, or ascribe it | RFC-0024 §4A.5/§5 | 9 |
| 16 | Let/ascription/tuple-literal type mismatch | the bound/ascribed/tuple-literal expression's type disagrees with the expected type | fix the expression or the ascription | RFC-0007 (edge_mismatch) | 3 |
| 17 | `lambda` rejects | a zero-parameter lambda, or a parameter/body type mismatch against the expected arrow type | give it ≥1 parameter; match the arrow's domain/codomain | RFC-0024 §4A.6 | 4 |
| 18 | `if`-expression rejects | a non-`Bool` condition, or branches whose types disagree | write a `Bool` condition; unify the branch types | RFC-0007 | 2 |
| 19 | `swap` operand-type rejects | the swap source/target isn't a representation type | swap between representation types only | RFC-0002 | 2 |
| 20 | `wild` FFI-escape rejects | a `wild` block outside `@std-sys`, or with no ascribed result type | mark the nodule `@std-sys`; ascribe the `wild` block's type | RFC-0016 §8-Q6, ADR-014, LR-9 | 2 |
| 21 | `colony`/`hypha` structural rejects | an empty `colony {}` | put ≥1 `hypha` in the colony | RFC-0008 §4.7 | 1 |
| 22 | `@forage` placement-policy rejects | a non-bitmask or non-literal placement policy | write a literal binary-bitmask policy | DN-63 §3.5, RFC-0008 RT3 | 2 |
| 23 | `fuse` semilattice rejects | mismatched operand types, or no `Fuse` instance for the type | declare/implement `Fuse` for the type | DN-58 §A.4, RFC-0008 RT6 | 2 |
| 24 | Binary/Ternary/Float comparison-prim rejects | wrong arity, a signedness/paradigm mismatch (e.g. `lt_s` on ternary), or a bare-decimal width ambiguity | use the paradigm-correct comparison prim (`lt`/`flt_lt`/…); ascribe the width | RFC-0032 D1, ADR-040 | 7 |
| 25 | Argument/field re-wrap (prim/trait-method/ctor) | an argument or field's own type-mismatch message re-wrapped with the call site's name | see the wrapped message's own alternative | RFC-0007 | 3 |
| 26 | Plain function-call arity/type mismatch | wrong argument count, or a parameter type mismatch | match the declared signature | RFC-0007 §4.4 | 2 |
| 27 | `match`-expression structural rejects | a non-data/Binary/Ternary scrutinee, zero arms, an unreachable arm, disagreeing arm types, or non-exhaustive coverage | match a supported scrutinee type; cover every case (W7) | RFC-0011 (W7 exhaustiveness) | 7 |
| 28 | Pattern-matching structural rejects | a constructor pattern's arity/shape doesn't match its type, a float literal pattern (refused — NaN/±0.0 identity), a mismatched literal-pattern type, a double-bound variable, or an unsupported list/seq pattern | match the correct shape; use an explicit comparison instead of a float pattern; bind each variable once | ADR-040 FLAG-4, RFC-0032 D3 (W7) | 9 |
| 29 | Literal/decimal encoding rejects | an empty binary/ternary literal, a bare-int literal with no paradigm, a negative decimal with no unsigned encoding, or a decimal that doesn't fit its declared width | write ≥1 digit; write a paradigm-specific literal or declare a `default paradigm`; fit the width | RFC-0032 D1/D3/D4, RFC-0012 (Q6) | 8 |
| 30 | List/`Seq` literal rejects | a literal whose length exceeds `u32`, disagrees with the expected `Seq` length, has non-homogeneous elements, or is empty with no ascribed element type | match the expected length/type; ascribe an empty list | RFC-0032 D3 | 4 |
| 31 | Bare-decimal ambient-context rejects | a bare decimal can't fill a different paradigm's context, or has no width pinned by context | write an explicit paradigm literal; ascribe the width | RFC-0012 §4.3 | 2 |
| 32 | `Dense` (tensor) prim rejects | wrong operand count/shape, a dim/dtype mismatch, or a `dense_scale` factor shape/dtype mismatch | match dim and dtype across operands | RFC-0001 §4.1, M-890/M-891 | 6 |
| 33 | VSA hypervector bind/permute/bundle rejects | wrong operand count/type, a model outside the dispatch set at introduction, a Sparse operand where Dense is required, a model/dim mismatch, or an empty/ill-typed `vsa_bundle` | use a registered model/dim; supply Dense-sparsity operands; supply ≥1 item | RFC-0003 §3-§5, M-892/M-893 | 13 |
| 34 | VSA cleanup-memory/reconstruct rejects | a malformed codebook (wrong shape/type/model/dim, or empty), a model outside the reconstruct dispatch set, or a non-`Float` threshold/δ | supply a well-typed non-empty codebook; use a registered model; pass a `Float` threshold/δ | RFC-0003 §6, M-894 | 10 |
| 35 | `Float` prim rejects | wrong operand count, or a non-`Float` operand | pass `Float` operands (`swap` first if needed) | ADR-040 §2.4/§2.5, M-898/M-899 | 2 |
| 36 | `Bytes`/`Seq` accessor prim rejects | a non-`Seq`/`Bytes` receiver, a non-`Binary` index, or a mismatched `width_cast`/`bytes_eq`/`hash_blake3` operand | pass the correct receiver/operand type | RFC-0032 D3/D4, M-912, DN-41 | 13 |
| 37 | Checker recursion-depth budget | the checker's own AST-recursion depth exceeded its explicit budget | reduce nesting depth (a checker-internal, never a host-stack overflow) | RFC-0007 §4.6 (banked guard 4) | 1 |
| 38 | Feature-deferred refusal (`spore`) | `spore` construction is deferred to the reconstruction-manifest work | none yet — tracked as E2-5/M-260 | E2-5/M-260 | 1 |
| 39 | Guarantee/grade lattice rejects | a body's inferred grade doesn't satisfy its declared `@ g` demand | weaken the annotation, or strengthen the body | RFC-0018 §4.3, VR-5 | 2 |
| 40 | `Fuse` semilattice-**law** rejects (definition-time) | a declared `Fuse` instance whose `join`, over a finite enumerable domain, violates idempotence / commutativity / associativity, or errors while being probed — refused at the `impl` site with a concrete counterexample (`Empirical`: exhaustive over the domain; a non-enumerable domain is *skipped*, never silently assumed lawful) | fix `join` to be a semilattice operation (idempotent, commutative, associative) | DN-58 §A.1/§A.4, RFC-0008 RT6, M-965 | 4 |
| | **Total** | | | | **217** |

## 5. Part C — Ambient-resolution rejects (`AmbientError`, 5/5 variants ledgered)

`AmbientError` (`crates/mycelium-l1/src/ambient.rs:51`) is a **closed enum** — ledgered 1:1.

| Variant | Reason | Surface alternative |
|---|---|---|
| `MultipleDefaults` | two nodule-scope `default paradigm` declarations — ambiguous outer frame | keep exactly one `default paradigm` per nodule |
| `UnresolvedAmbient` | a paradigm-less `{…}` repr with no enclosing ambient (§4.3) — no implicit global fallback | declare a `default paradigm`, or write the paradigm explicitly |
| `ParadigmShapeMismatch` | a written shape doesn't fit the ambient paradigm in force | match the ambient paradigm's shape, or write an explicit paradigm |
| `BareDecimalNoEncoding` | a bare decimal under a `Dense`/`VSA` ambient — those paradigms have no bare-decimal encoding | use `Binary`/`Ternary`, or write the value in the paradigm's own literal form |
| `DepthExceeded` | the resolution pass's own AST recursion exceeded its budget (compiler-internal, not a program-semantics claim) | reduce nesting depth |

## 6. Part D — Runtime/interpreter rejects (`EvalError`, 17/17 variants ledgered)

`EvalError` (`crates/mycelium-interp/src/lib.rs:142`) is a **closed enum** — ledgered 1:1. This is the
kernel's own never-silent floor (SC-3/G2): "why evaluation could not proceed."

| Variant | Reason | Surface alternative |
|---|---|---|
| `FreeVariable` | a free (unbound) variable was encountered — the program is not closed | close the program (bind every variable) before evaluating |
| `UnknownPrim` | no primitive is registered under this name (a `wild:`-prefixed name is reported as an *ungranted host capability*, not a typo) | register the prim, or grant the `wild:` capability from `@std-sys` |
| `PrimType` | a primitive was applied to the wrong arity/paradigm/width (one row for the whole per-prim family — the checker should have caught this; kept as the interpreter's own defensive floor) | fix the checker-level type error upstream (Part B families 24/32-36) |
| `ApproxCompositionUnsupported` | a primitive would have to compose an approximate input with no defined ε-propagation rule | use an exact input, or wait for the rule (ADR-010/M-204) |
| `UnsupportedSwap` | the swap engine has no conversion for this `(from → to)` pair | use a certified swap pair (M-120) |
| `Overflow` | a fixed-width arithmetic result fell outside the representable range | use a wider representation, or check bounds before the op |
| `FuelExhausted` | evaluation exceeded its step budget (non-termination guard) | raise the fuel budget, or fix a runaway computation |
| `DepthLimit` | evaluation exceeded its control-stack depth budget (AOT env-machine) | reduce recursion depth, or raise the ceiling |
| `EffectBudget` | a declared effect budget (retry/cascade/alloc/time) was exceeded | raise the budget, or reduce the effect's usage (RFC-0014 §4.5 I4) |
| `Swap` | the swap engine reported a failure (illegal pair or out-of-range conversion) | see the engine's own message |
| `Wf` | a constructed result violated a Core IR well-formedness invariant | see Part E (`WfError`) |
| `NonExhaustiveMatch` | a `Match` reduced with no alternative and no `default` (unreachable for checked programs — the checker proves coverage; kept as the kernel's own floor) | ensure the checker ran (W7 exhaustiveness) before evaluating |
| `DataMalformed` | a `Construct`/`Match` node is malformed against the data fragment (arity mismatch/non-saturated constructor the checker should have caught) | ensure the checker ran before evaluating |
| `GuaranteeMeetUnsupported` | a `Match` on a non-`Exact` data scrutinee would have to fold a composite guarantee the r3 boundary doesn't yet realize | keep the scrutinee `Exact` (the reachable r3 fragment) |
| `DataResult` | [`eval`] was asked for a representation value but the program evaluated to a data value | call `eval_core` for the data fragment instead |
| `ApplyNonFunction` | an `App`'s function position reduced to a non-function value (unreachable for checked programs) | ensure the checker ran before evaluating |
| `FunctionResult` | the program evaluated to a bare function value (a v0 entry never returns one) | return a representation/data value from `main` |

## 7. Part E — Kernel well-formedness rejects (`WfError`, 7/7 variants ledgered)

`WfError` (`crates/mycelium-core/src/lib.rs:56`) is a **closed enum** — ledgered 1:1. This is the
lowest-level reject: a `Value`/`Repr` that fails Core IR well-formedness can never be constructed.

| Variant | Reason | Surface alternative |
|---|---|---|
| `GuaranteeBoundMismatch` | the guarantee/bound pairing violates M-I1…M-I4 (the honesty rule itself) | fix the pairing so the tag matches the bound it carries |
| `MalformedBound` | a bound's numeric payload is out of range (e.g. `delta ∉ [0,1]`) | supply an in-range bound |
| `MalformedRepr` | a representation has non-positive width/dim/trits, or an empty VSA model id | supply a positive dimension and a named model |
| `DimensionTooLarge` | a representation's dimension exceeds `repr::MAX_DIM` (an over-allocation/DoS guard) | request a smaller dimension |
| `PayloadReprMismatch` | a payload's paradigm or length doesn't match its declared representation | make the payload agree with the `Repr` it's paired with |
| `MalformedReconstruction` | a reconstruction manifest violates its schema (RFC-0003 §6) | fix the manifest against `reconstruction-manifest.schema.json` |
| `MalformedSparsity` | a measured sparsity observation is out of range (e.g. `density ∉ [0,1]`) | supply an in-range observation |

## 8. The regression guard

**Location:** `crates/mycelium-std-conformance/tests/reject_ledger.rs` — a new test file in the
existing **test-only conformance crate** (`mycelium-std-conformance`, which already dev-depends on
`mycelium-l1`/`mycelium-interp`/`mycelium-core` for its three-way differentials — RFC-0031 D5/D6). This
does **not** touch the L1 frontend, `prims.rs`, or `mycelium-l1/Cargo.toml`: the guard reads the
audited source files as **plain text** (`std::fs::read_to_string`, paths relative to
`CARGO_MANIFEST_DIR`) — it needs no new dependency and edits no read-only file.

**What it checks (never-silent, G2 — each assertion names the file/pattern/count so a failure is
immediately actionable):**

1. **Parse-level corpus size.** The `reject/` directory holds exactly 30 `.myc` fixtures, and every
   filename this ledger names in §3 is present. (Guards against a fixture being silently added/removed
   without updating this ledger; does not duplicate `conformance.rs`'s own `REJECT_EXPECTED`
   bidirectional check, which stays owned by the L1 frontend.)
2. **Check-level call-site counts.** `grep`-style counts (via regex over the read text) of
   `CheckError::new(`/`CheckError::at(` and `self.err(` in `checkty.rs`, `CheckError::at(` in
   `grade.rs`, and `CheckError::new(` in `fuse.rs`, must equal **102 / 110 / 2 / 4** respectively
   (§4's audited totals). A mismatch — in either direction — fails with a message naming which
   file/pattern drifted and by how much, and pointing at this note to add/adjust the family rows
   before updating the pinned constant.
3. **Closed-enum variant sets.** The `AmbientError`, `EvalError`, and `WfError` variant **names**
   (extracted from each `pub enum … { … }` block by regex) must equal the exact 5/17/7-element sets
   ledgered in §5/§6/§7 — as a `BTreeSet` equality, so an added, removed, *or renamed* variant fails
   loudly (the strongest guarantee this guard offers, since these are genuinely closed types).

**Honesty (`Empirical`, never `Proven`):** this is a **line/regex heuristic over source text**, the
same posture `docs/api-index/` states for itself. It proves *"the audited call-site/variant counts
match what this ledger describes as of the last audit"* — it does **not** prove semantic completeness
(that no *other* reject path could exist beyond what these patterns find), nor does it prove the
*maintained* prose in §4's family table stays accurate prose-for-prose as call sites shift within a
family (only the per-file totals are pinned). A family gaining a row internally without changing the
file-level total would not be caught — a known, stated limitation (VR-5: never claim more than the
check delivers). The **strong** portion of the guard (§5-§7, closed enums) has no such gap: any
variant-level change is caught exactly.

**Why not a finer per-site guard for the free-text stratum:** `CheckError`/`self.err` carry no
reason-code enum (by design — they're a free-text `{site, message}`, not a discriminated union), so
there is no mechanical anchor finer than "how many call sites construct one" without either (a)
adding a reason-code enum to `checkty.rs` (a frontend edit this leaf is not authorized to make — FLAG,
§9) or (b) parsing each message's literal string (brittle — a wording-only diagnostic improvement
would spuriously fail the guard, which is exactly the kind of noise a `Declared`-tag data-driven test
avoids per the house test-layout rule). Pinning the **count** is the honest middle ground: it never
misses a *new* reject path (the count would change), at the cost of not localizing *which* one moved.

## 9. FLAGs (never guessed — G2/VR-5)

- **FLAG-1 (guard granularity):** the check-level guard (§8.2) is count-based, not per-construct —
  see §8's own "why not a finer guard" for the grounded reason. A future **reason-code enum** on
  `CheckError` (an L1-frontend change, out of this leaf's read-only scope) would let a future revision
  of this ledger tighten §4 to a 1:1 mapping like §5-§7 already have. Not blocking DN-56 condition 1
  (the DoD asks for a regression guard that fails on an unledgered *addition*, which the count pin
  satisfies), but worth a follow-up task if the maintainer wants finer granularity.
- **FLAG-2 (shared files untouched):** `CHANGELOG.md`, `docs/Doc-Index.md`, `docs/api-index/`, and
  `tools/github/issues.yaml` are **not** edited by this leaf (orchestrator-owned per the M-959
  directive) — the integrating parent must register this note, close M-959, and re-score DN-76's
  scorecard row 1.
- **FLAG-3 (prims.rs per-prim granularity):** Part D's `EvalError::PrimType` is ledgered as **one**
  row spanning the whole per-registered-prim type-check-at-eval-time family (Π currently holds 31
  named prims per DN-76 §3.2) rather than one row per prim — `prims.rs` is explicitly read-only for
  this leaf, and the individual `why` strings it constructs are prose detail under the same
  `PrimType` variant, not additional enum variants. If finer per-prim ledgering is wanted, it is
  `prims.rs`-owning-lane work, not this leaf's.

## 10. Grounding

**DN-56** (§2.2/§5.2 — the condition this closes) · **DN-76** (§3.1 — the scorecard row this flips)
· **RFC-0002** (never-silent swap) · **RFC-0007** (§4.5/§4.6/§4.8/§11.3 — totality, recursion budget,
`for`, instantiation) · **RFC-0011** (W7 exhaustiveness) · **RFC-0012** (§4.3 ambient resolution)
· **RFC-0014** (§4.5 effects/budgets) · **RFC-0016/RFC-0028** (`wild` FFI floor) · **RFC-0018**
(§4.3 guarantee lattice) · **RFC-0019** (trait/impl coherence) · **RFC-0024** (§4A.5/§4A.6/§5 —
functions as values) · **RFC-0032** (D1/D3/D4 — literals, `Seq`/`Bytes`) · **ADR-014** (`wild` audit
floor) · **ADR-040** (Float; FLAG-4 pattern refusal) · **DN-03** (§1 reserved words, `consume`/
`Substrate`) · **DN-38/DN-53/DN-54** (lowering, `object`, generative `lower`/`derive`) · **DN-41/
DN-42** (width-cast, width-generic ambiguity) · **DN-58/DN-63** (`fuse`, `@forage`) · house rules
**G2** (no black boxes), **VR-5** (never upgrade past basis), **KC-3** (small auditable kernel).
Repo evidence, read-only: `crates/mycelium-l1/src/{error,checkty,grade,ambient}.rs`,
`crates/mycelium-l1/tests/conformance.rs`, `docs/spec/grammar/conformance/reject/`,
`crates/mycelium-interp/src/{lib,budget,prims}.rs`, `crates/mycelium-core/src/lib.rs` — all at `dev`
tip `41ec4fe` (2026-07-02).

## Changelog

| Date | Status | Note |
|---|---|---|
| 2026-07-02 | **Accepted** | Authored (M-959, kickoff `frz`, Lane A) as the DN-56 condition-1 reject-ledger. Enumerates 29 parse-level fixtures (Part A, restating the existing self-policing corpus), 204 check-level `CheckError`/`self.err` sites grouped into 40 construct families with exact per-family counts (Part B), and three closed enums ledgered 1:1 — `AmbientError` (5), `EvalError` (17), `WfError` (7) (Parts C-E). Wires a regression guard (`crates/mycelium-std-conformance/tests/reject_ledger.rs`) that pins the audited counts/variant-sets and fails never-silently on drift. All tags `Empirical` (a checked, mechanical inventory — VR-5, never `Proven`). Read-only against the L1 frontend and `prims.rs` throughout (a concurrent M-915 leaf owns them). **Live-validated the guard's purpose during authoring:** while this leaf worked, the concurrent M-915 leaf landed fixture 30 (`30-vec-short-alias-rejected.myc`, RFC-0037 D2-b) on `dev`; pulling that tip down made `parse_level_reject_corpus_matches_the_ledger` fail immediately (an unledgered fixture) — exactly the regression it exists to catch — and this revision adds its ledger row before the count assertions were updated to 29. FLAGs: guard granularity is count-based for the free-text stratum (§8), shared files untouched (orchestrator to reconcile), `PrimType` ledgered as one family (not per-prim). |
| 2026-07-02 | **Accepted** | Reconcile: the concurrent `grm` wave (M-916, RFC-0037 D1) landed reject fixture `31-old-le-ge-glyph-retired.myc` on `dev` without a ledger row, so `parse_level_reject_corpus_matches_the_ledger` failed never-silently — exactly the drift the guard exists to catch. Adds the §3 row 31 (the retired two-char `<=`/`>=` glyphs → word-canonical `lte`/`gte`), bumps the §3 header + §8.1 corpus-size to 30, and updates the guard's pinned fixture list + count (29 → 30) in the same commit. **Also re-audits Part B:** the M-919 (DN-71 Model S) and M-973 (DN-54 §10 attachment Model A) lower/derive extension-checker work landed **6 net new** `checkty.rs` `CheckError` sites (verified: 8 added, 2 refactored-out, all in `check_nodule_with`/`check_lower_*` — family 8), so `checkty.rs` direct total goes 93 → 99, family 8's site count 8 → 14, §4 grand total 204 → 210, and §8.2's pinned `93/110/2` → `99/110/2`. `self.err` (110), grade (2), and all closed enums (Parts C–E) unchanged. **And absorbs M-965's `Fuse` law-checker (landed on `dev` concurrently):** `+2` `checkty.rs` family-5 `Fuse`-redeclaration refusals (checkty direct 99 → 101, family 5 count 9 → 11) and a **new audited file** `crates/mycelium-l1/src/fuse.rs` with `4` definition-time semilattice-law rejects (idempotence/commutativity/associativity/probe-eval-failure) ledgered as **new family 40**; §4 grand total 210 → 216, family count 40 → 41, §8.2 pinned `99/110/2` → `101/110/2/4`, and the guard gains a `fuse.rs` count assertion so the new reject path is guarded like the rest. Tags remain `Empirical` (mechanical re-count, VR-5). |
| 2026-07-02 | **Accepted** | Reconcile: M-966 (`via`-delegation deterministic ordering) landed on `dev` adding **1** `checkty.rs` `CheckError` site — the `via` ambiguity refusal (two `via` clauses claiming one trait). → family 6 count 2 → 3, `checkty.rs` direct total 101 → 102, §4 grand total 216 → 217, §8.2 pinned `101/110/2/4` → `102/110/2/4`, guard pinned count updated. Also corrected the family-table Total row (was stale at 204 since the first re-audit — the guard pins only the per-file totals, not the family sum, so it went uncaught; now 217, matching the audited non-plumbing total). Tags remain `Empirical`. |
