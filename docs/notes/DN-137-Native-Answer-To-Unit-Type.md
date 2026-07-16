# Design Note DN-137 — Mycelium's Native Answer to the Problem Rust `()` (Unit) Solves

| Field | Value |
|---|---|
| **Note** | DN-137 |
| **Status** | **Accepted** (2026-07-13) — ratified through the strict 9-criterion DN-review gate on a **clean re-pass** after the criteria-5/6/9 patch (native-solution / KC-3 / M-826-consistency): Alt D adopted, Alt A (`base_type`) rejected, the arity-0-of-M-826 grounding verified against `mycelium.ebnf:151/156/255-256` + `lib/compiler/ast.myc:132/154/335`. Ratification is the **delegated DN-review gate** (maintainer standing automation), not a self-ratify by the note (house rule #3 preserved — the note *recommends*; the independent gate *ratifies*). **Accepted authorizes the Phase-2 wave-2 `Unit` build leaf** (§6 build DoD): a prelude `type Unit = Unit;` (or the equivalent nullary-constructor seed) + one `type_map::TABLE` row (`() → Unit`), **no `mycelium-core` edit**, differential-witnessed before any `Empirical` upgrade past `Declared` (VR-5). It remains **design only** — the build lands under its FLAGGED issue (M-1102). Prior status: Draft (2026-07-13, patched post-gate). |
| **Task** | Scope Mycelium's native answer to Rust's unit type `()` — the single largest specific "Other"-class gap reason in the current DN-136 Phase-2 re-measure (26/203 instances, `docs/planning/DN-136-phase2-bulk-gap-close-worklist.md` §3 item D1): `unit type () has no representable value in this grammar fragment`. A genuinely new, un-owned language question — not a transpiler oversight. |
| **Decides** | *Proposes, for ratification (revised at the strict-gate patch — the earlier Draft's base_type recommendation is dropped; see §7):* a prelude **`type Unit = Unit;`** — a **nullary-constructor algebraic data type** (the arity-0 member of the M-826 tuple/product family), reusing the **existing** ADT machinery with **no new kernel node and no grammar change**. `constructor ::= Ident ('(' … ')')?` (`mycelium.ebnf:156`) already makes the field-parens optional, so a payload-free constructor is grammatical *today* and is used pervasively in the self-hosted compiler (`type Vis = Private \| Pub;`, `type Paradigm = …`, `type ExecutionMode = Interpreted \| Compiled;` — `lib/compiler/ast.myc:132/154/335`). A named type is already a valid `type_ref`, so `fn f() => Unit = Unit;` is grammatical today with `Unit` in the prelude. This honestly satisfies the mandatory `'=>' type_ref` production (`fn_sig`/`fn_item`, `mycelium.ebnf:162/168`) — Mycelium has no implicit-return-type sugar, unlike Rust, so *some* `type_ref` must be written. |
| **Feeds** | `docs/planning/DN-136-phase2-bulk-gap-close-worklist.md` §3/§5 (D1); the `type_map::TABLE` axis (DN-136 §4.2, frozen), which would carry the `() → Unit` row once built. |
| **Grounds on** | **KC-3** (smallest possible addition — a prelude ADT declaration reusing the *existing* nullary-constructor parse/check/eval machinery, **zero** new kernel/`BaseType`/grammar surface; the strictly-more-minimal answer the strict gate correctly identified); **DRY** (the same ADT + M-826 product-family desugar path, not a second parallel type-family); **G2/never-silent** (the alternative of silently reusing an existing type as a sentinel is rejected below precisely because it would fabricate a value where none exists); **VR-5** (every claim tagged; the corpus count is `Empirical`, the design is `Declared` until built); **M-826** (the tuple/product family "desugars to a synthetic single-ctor data type … mycelium-core untouched" — `Unit` is that family's arity-0 member, so a new `base_type` would *touch* the kernel M-826 deliberately avoids); DN-111 (native-translation taxonomy — classifies the *problem* `()` solves, not the Rust token). |
| **Definition of Done** | §6. In one line: **Accepted** requires the maintainer/gate to confirm (a) a prelude nullary-constructor `type Unit = Unit;` (the M-826 arity-0 product-family member, mycelium-core untouched) is the minimal answer over the rejected base_type / sentinel-reuse / grammar-loosening alternatives — after which a `type_map::TABLE` row (`() → Unit`) is a same-shape Phase-2 additive leaf. The value's spelling is resolved *for free* by Alt D (the existing constructor-application expression — no new literal token; OQ-1 dissolved). |

---

## §1 The problem, precisely located

**Verify-first (mitigation #14).** Read directly against `docs/spec/grammar/mycelium.ebnf` at
`dev@b1755fa6`:

- `fn_sig ::= 'fn' Ident type_params? const_params? '(' params? ')' '=>' type_ref effects?` (line
  162) and `fn_item`'s two productions (line 168–169) both require `'=>' type_ref` **unconditionally**
  — there is no "omit the arrow for a void function" sugar anywhere in the grammar. Every function,
  including one whose only job is a side effect, must write a real `type_ref` after `=>`.
- `type_ref ::= base_type (...)` and `base_type` (line 240–257) enumerates every nullary/compound
  type Mycelium has today: `Binary{N}`/`Ternary{N}`/`Dense{...}`/`VSA{...}` (all parametrized),
  `Substrate{Ident}`, `Seq{type_ref, Int}`, `Bytes` (nullary), `Float` (nullary), a **tuple type
  requiring arity ≥ 2** (`M-826` — explicitly not 0 or 1), and a named type/type-variable. **None
  of these is a legitimate stand-in for "no value."**
- So a Rust `fn f() { ... }` (implicit `-> ()`) or an explicit `fn g() -> ()` has **no** `type_ref`
  it can honestly carry today. The transpiler's current behavior (an explicit `Category::Other`
  gap, never a guess) is exactly correct under G2 — this is confirmed live in 26 of the current
  corpus's 203 `Other`-class instances, several of which are whole functions whose *signature*
  alone gaps (before any body content is even considered), which is disproportionately high
  leverage per gap-inventory's "file/item-gating" leverage framing.

This is a **genuinely new question**, not a residual of any existing DN: DN-127 (Display/formatting),
DN-128 (derive library), DN-125 (`&mut self`), DN-129 (`Default`/`Error`) all assume a *real* return
value exists; none of them touches the "there is no return value at all" case.

## §2 Alternatives, evaluation, recommendation

### Objective function

| Criterion | Weight | Why it matters here |
|---|---|---|
| **Honesty (G2/VR-5)** | critical (veto) | Must not fabricate a meaningful value where Rust's `()` genuinely carries none |
| **KC-3/KISS** | high | Smallest addition; **prefer reusing existing machinery over any new kernel/grammar surface** (the strict-gate correction — see Alt D) |
| **Grammar/M-826-consistency** | high | Must respect the mandatory `=> type_ref` production *and* not touch the kernel/grammar that M-826's tuple/product family deliberately left untouched |
| **Coverage leverage** | high | 26/203 current "Other" instances, several signature-gating (whole-item leverage, not just body leverage) |

### Alt D — a prelude nullary-constructor data type `type Unit = Unit;` (RECOMMENDED)

Declare, in the prelude, a payload-free algebraic data type: `type Unit = Unit;` — a `type_item`
(`mycelium.ebnf:151`) whose single `constructor` (`mycelium.ebnf:156`) carries no field-parens. This
is the **arity-0 member of the M-826 tuple/product family** (`mycelium.ebnf:255-256`: the
`(type_ref, …)` tuple type "desugars to a synthetic single-ctor data type … no new L0 node"; M-826
scoped its *surface* to arity ≥ 2, but the underlying single-ctor-data-type mechanism is exactly what
a nullary product is). **Nothing new is built at the kernel or grammar layer** — the parse/check/eval
path for a nullary constructor already exists and is exercised pervasively in the self-hosted compiler
(`type Vis = Private | Pub;`, `type Paradigm = PBinary | …`, `type ExecutionMode = Interpreted |
Compiled;` — `lib/compiler/ast.myc:132/154/335`, all payload-free constructors used as first-class
values and returns).

- **Honesty:** exact — `Unit` denotes precisely "no informative value," matching `()`'s semantics
  1:1. **Native Equivalent** under DN-111 (a first-class native construct fills the role directly,
  exact, structure-preserving) — no reform of shape; `()` and `Unit` are the *same* concept.
- **KC-3/KISS:** **minimal — this is the smallest possible answer.** Zero new `BaseType` variant,
  zero grammar change, zero new evaluator machinery: a prelude *declaration* over already-landed ADT
  support. The `type_map::TABLE` (DN-136 §4.2, frozen) gets exactly one new row (`() → Unit`), the
  same additive shape as every other type-mapping row — and, unlike Alt A, the build leaf itself is a
  pure prelude-declaration + one table row, **no `mycelium-core` edit at all**.
- **Value spelling — resolved for free.** The one value is the ordinary constructor-application
  expression `Unit` (a nullary constructor *is* its own value, exactly as `Private`/`Pub` are in
  `ast.myc`). No new literal token, no `()`-expression production needed — which is why OQ-1 (the
  earlier Draft's open sub-question) **dissolves** under Alt D (§3).
- **Grammar/M-826-consistency:** exact fit — the mandatory `'=>' type_ref` is honestly satisfiable
  (`Unit` is a named `type_ref`), *and* the kernel/grammar stays untouched, consistent with M-826's
  own "mycelium-core untouched" posture (see Alt A's rejection for why a `base_type` would violate
  this).
- **Verdict:** **Rank 1.**

### Alt A — a new nullary `Unit` `base_type` (REJECTED — not minimal; inconsistent with M-826)

Add `| 'Unit'` to `base_type` (`mycelium.ebnf:240ff`), parallel to the nullary `Bytes`/`Float`
entries.

- **KC-3/KISS:** **fails relative to Alt D.** A new `base_type` variant **touches the kernel and the
  grammar** — precisely the surface M-826's tuple/product family was designed to *avoid* ("desugars
  to a synthetic single-ctor data type … no new L0 node"). `Unit` is the arity-0 member of that same
  family, so it should ride the same desugar-to-ADT path, not open a new scalar/base-type slot.
- **The `Bytes`/`Float` analogy is false (the strict-gate correction).** `Bytes` and `Float` are
  `base_type`s **because they carry representation the kernel manages** — widths, IEEE-754 layout,
  representation-swaps. `Unit` carries **no representation at all** (nothing to swap, no width, no
  paradigm) — which is exactly why it belongs with the *desugar/ADT* family, not the scalar/base-type
  family. The earlier Draft's "add a fourth nullary base_type, the mechanism is proven" reasoning
  conflated "nullary" with "scalar-repr-carrying"; dropped.
- **Grammar-consistency:** the earlier Draft claimed a base_type is a "perfect fit" for the mandatory
  `=> type_ref`. It satisfies that one production, but it is **inconsistent with M-826** — it adds
  kernel/grammar surface where an existing ADT path suffices. Corrected.
- **Verdict:** **Rank 2 — rejected in favor of Alt D.** (No grounded reason was found to require a
  base_type over Alt D, given `lib/compiler/`'s pervasive named nullary-constructor data types used
  as first-class values/returns; if a future need for kernel-managed unit *representation* ever
  arose — none is known — that would be a fresh, grounded decision, not this note's.)

### Alt B — reuse an existing nullary type as a sentinel (e.g. always-`false` `Bool`, or always-empty `Bytes`)

- **Honesty:** **fails the veto.** A `Bool` return that is *always* `false` is not "no value" — it
  is a **fabricated** value that happens to never vary, indistinguishable at the type level from a
  real boolean result. This is exactly the "plausible but wrong" emission class DN-34/`map_type`
  already refuses elsewhere (qualified-path collapsing, `Widen`/`Narrow` bodies) — reusing an
  existing type here would be the same category of dishonesty this codebase has consistently
  rejected. **Rejected outright**, not merely lower-ranked.

### Alt C — special-case the grammar so `=> type_ref` becomes optional (Rust-style implicit unit)

- **Grammar-consistency:** **fails.** Mycelium's `fn_sig` mandates an explicit return-type
  annotation on *every* function — this is a deliberate design posture (no implicit types anywhere
  in the surface grammar, consistent with the project's explicit-annotation stance throughout
  `mycelium.ebnf`), not an oversight Rust's implicit `-> ()` sugar should be imported to "fix."
  Overriding it would be a broader grammar-philosophy decision this narrow note has no mandate to
  make — and is moot anyway, since Alt D satisfies the mandatory production honestly with `Unit`.
- **Verdict:** **Rejected** — out of scope, and unnecessary given Alt D.

### Ranked recommendation

**Alt D ≻ Alt A (rejected: not minimal, M-826-inconsistent) ≻ Alt C (out of scope) ≻ Alt B (rejected
on honesty).** Adopt **Alt D — a prelude nullary-constructor `type Unit = Unit;`**: the strictly-most-
minimal answer (KC-3), an exact semantic match (DN-111 Native Equivalent), **zero new kernel/grammar
surface** (reuses the existing ADT + M-826 product-family desugar path, `mycelium-core` untouched),
and it satisfies the mandatory-return-type production honestly rather than fighting it — with the value
spelling resolved for free by the existing constructor-application expression.

## §3 Open questions (not guessed — VR-5)

- **OQ-1 (value spelling) — DISSOLVED by Alt D (was open under the earlier base_type Draft).** The
  earlier Draft left the unit *value*'s surface spelling open (a `unit` keyword? reuse `()`?). Alt D
  removes the question entirely: `Unit`'s one value is the ordinary **constructor-application
  expression `Unit`** — a nullary constructor *is* its own value (exactly as `Private`/`Pub` are in
  `lib/compiler/ast.myc`), so no new literal token or `()`-expression production is introduced.
  Nothing to defer.
- **OQ-2 (effects interaction).** `fn_sig` already carries a separate `effects?` production distinct
  from the return type. Does a genuinely side-effecting, `Unit`-returning function's `EXPLAIN`
  surface distinguish "no value, but an effect occurred" from "no value, pure"? Likely already
  handled by the existing `effects` clause without any `Unit`-specific change — flagged for the
  build to confirm, not re-designed here (out of this note's scope).

## §4 Adversarial stress-test (house rule #4)

**Does adding `Unit` reopen any landed soundness gate?** No — `Unit` is nullary (no width, no
paradigm, no representation choice), so it participates in **none** of the swap/EXPLAIN/certified-mode
machinery those gates protect (RFC-0034/ADR-032's guarantees are about *representation* choices;
a type with exactly one value has nothing to represent-swap). And under Alt D it is a **prelude ADT
declaration**, not a kernel/grammar change, so it cannot perturb any parse/check/eval invariant the
existing nullary-constructor path (`ast.myc`'s `Vis`/`Paradigm`/`ExecutionMode`) does not already
exercise. The struct-derive library (DN-128), the emit-hook interfaces (DN-136), and the
pattern/derive/call axes are all unaffected — `Unit` enters `type_map::TABLE` as an ordinary additive
row, identical in shape to `bool`/`f64`/`char`.

**Is Alt D's "arity-0 member of the M-826 family" claim sound, or a stretch?** Checked: M-826's
grammar comment (`mycelium.ebnf:255-256`) states the tuple/product type "desugars to a synthetic
single-ctor data type (KC-3 — no new L0 node)" and scopes its *surface* to arity ≥ 2 (a 1-tuple and a
0-tuple were excluded from the *surface* to avoid `(x)`-grouping ambiguity and because no need had
arisen). The *underlying mechanism* — a single-ctor data type — is representation-agnostic in arity;
a nullary product (one ctor, zero fields) is a well-formed instance of it, and is exactly what
`type Unit = Unit;` declares. So Alt D does not *reinterpret* M-826; it instantiates the same
single-ctor-data-type target at arity 0 via the ordinary `type_item` path (no new surface tuple
syntax needed, since a named prelude type suffices). The claim is grounded, not a stretch.

**Does Alt B's rejection undercut any already-landed pattern?** No — checked against `Clone`/`Copy`'s
`DeriveSatisfied` no-op handling (which records a true structural fact: value semantics genuinely
satisfy `Clone`/`Copy` with no generated code) versus Alt B's proposal (fabricating a *specific,
wrong-shaped* value where none exists) — these are not analogous; `DeriveSatisfied` never invents a
false value, it records a true equivalence.

## §5 Python carry-forward (flagged, not scoped — VR-5)

Python has no unit type — `None` plays a partially-overlapping role (both "no return value" and
"absence of a value" more generally, e.g. `Optional`). Whether Mycelium's `Unit` and `Option[T]`'s
`None` case should be unified or kept distinct is a Python-reframe question, out of this note's
Rust-sourced scope (per DN-119 §11's standing policy — Python gets its own DN-111 pass).

## §6 Definition of Done

**This note reaches Accepted only when the maintainer/gate confirms:**
1. **Alt D** (a prelude nullary-constructor `type Unit = Unit;`, the M-826 arity-0 product-family
   member, `mycelium-core` untouched) is ratified as the minimal answer over Alt A (rejected: a new
   `base_type` is not minimal and is M-826-inconsistent), Alt B (rejected, honesty veto), and Alt C
   (out of scope).
2. (OQ-1 is dissolved by Alt D — nothing to resolve; the value is the constructor-application
   expression `Unit`.)

**Then, the build DoD:**
- A prelude `type Unit = Unit;` declaration (or the equivalent seed) in `mycelium-l1`'s prelude —
  **no new `BaseType` variant, no grammar change** — reusing the existing nullary-constructor
  parse/check/eval path (the one `ast.myc`'s `Vis`/`Paradigm`/`ExecutionMode` already exercise);
  `Exact` guarantee by construction (one value, nothing to measure or swap).
- One new `type_map::TABLE` row (`() → Unit`) in `crates/mycelium-transpile` — a same-shape
  Phase-2 additive leaf against the frozen DN-136 interface, differential-witnessed against the
  26-instance corpus sample before any `Empirical` upgrade past `Declared` (VR-5).
- No `mycelium-core` edit at all (the Alt-D dividend over the rejected base_type path).
- No claim is tagged `Proven` (no checked theorem here); everything stays `Declared` until built.

## §7 Changelog

- **2026-07-13** — **Accepted** via the strict 9-criterion DN-review gate (delegated ratification;
  maintainer standing automation). Clean re-pass after the criteria-5/6/9 patch. Grounding
  re-verified against source: `constructor ::= Ident ('(' … ')')?`'s optional field-parens
  (`mycelium.ebnf:156`), `type_item` (`:151`), the M-826 desugar comment "synthetic single-ctor data
  type … no new L0 node" (`:255-256`), the pervasive payload-free constructors `Vis`/`Paradigm`/
  `ExecutionMode` (`lib/compiler/ast.myc:132/154/335`), and the mandatory `'=>' type_ref` in
  `fn_sig`/`fn_item` (`:162/168`). **Native-solution / KC-3 / consistency (5/6/9) now pass:** Alt D
  (prelude nullary-constructor `type Unit = Unit;`) is the strictly-minimal answer — no kernel/grammar
  addition, reuses the existing ADT parse/check/eval machinery (KC-3/DRY) — and Alt A (a new
  `base_type`) is correctly rejected as non-minimal and M-826-inconsistent (the `Bytes`/`Float`
  analogy is false: those are base_types because they carry kernel-managed *representation*; `Unit`
  carries none). Alt B (sentinel) / Alt C (grammar-loosening) rejections sound. **Adversarial re-run:**
  the nullary-constructor `Unit` reopens no swap/EXPLAIN/certified-mode gate (nothing to
  represent-swap) and cannot perturb any parse/check/eval invariant the existing nullary-ctor path
  does not already exercise; the "arity-0 member of the M-826 family" claim is grounded (M-826's
  arity ≥ 2 is a *surface* scoping; the *mechanism* — a single-ctor data type — is arity-agnostic).
  One minor build-leaf note (not gate-blocking): the same-name `type Unit = Unit;` spelling is
  unwitnessed in the corpus, so the sole constructor may need a distinct spelling if the language does
  not separate the type and value namespaces — §6's "(or the equivalent seed)" already covers this,
  and OQ-1's dissolution holds regardless (the value is a constructor-application expression, not a new
  literal token). Authorizes the wave-2 `Unit` build leaf (**M-1102**). Status Draft→Accepted; no prior
  content rewritten (append-only, house rule #3).
- **2026-07-13** — **Draft patch (post strict-gate, criteria 5/6/9 — native-solution / KC-3 /
  M-826-consistency).** The gate accepted the semantic direction (nullary `Unit` is the honest
  Native Equivalent; Alt B sentinel-reuse correctly rejected on G2/VR-5; Alt C correctly rejected)
  but found the recommendation **not minimal**: a strictly-smaller, precedent-grounded option was not
  enumerated. **Enumerated + adopted Alt D — a prelude nullary-constructor data type
  `type Unit = Unit;`** (the arity-0 member of the M-826 tuple/product family), reusing the existing
  ADT parse/check/eval machinery with **no new kernel node and no grammar change**, grounded in
  `constructor ::= Ident ('(' … ')')?`'s optional field-parens (`mycelium.ebnf:156`) and its
  pervasive use in `lib/compiler/ast.myc` (`Vis`/`Paradigm`/`ExecutionMode`). **Demoted Alt A (new
  `base_type`) to Rank 2 / rejected**: corrected the false `Bytes`/`Float` analogy (those are
  base_types because they carry kernel-managed *representation*; `Unit` carries none, so it belongs
  with the desugar/ADT family) and the "grammar-consistency perfect fit" claim (a base_type is
  *inconsistent* with M-826's "mycelium-core untouched" posture). **Dissolved OQ-1** (value spelling)
  — Alt D's value is the ordinary constructor-application expression `Unit`, no new literal token.
  Rewrote the Decides/Grounds/DoD header rows, §2 (Alt D added + adopted, Alt A rejected with the two
  corrections), §3 (OQ-1 dissolved), §4 (added the M-826-arity-0 soundness check), and §6 DoD (prelude
  ADT, no `mycelium-core` edit). Status stays **Draft** (re-gate pending; no self-ratify — house rule
  #3).
- **2026-07-13** — initial Draft. Scoped Mycelium's native answer to Rust's unit type `()`,
  triggered by the DN-136 Phase-2 re-measure finding it the single largest specific "Other"-class
  gap reason (26/203 instances). Verified against `mycelium.ebnf` that `fn_sig`/`fn_item` mandate
  an explicit `=> type_ref` on every function (no implicit-unit sugar), so omitting the return type
  is not grammatically available — recommends a minimal nullary `Unit` `base_type` (Alt A, DN-111
  Native Equivalent) over reusing an existing type as a dishonest sentinel (Alt B, rejected on the
  G2 veto) and over loosening the mandatory-annotation grammar (Alt C, out of scope). Two open
  questions (literal spelling, effects interaction) left unresolved, not guessed (VR-5). Recommends,
  does not ratify (house rule #3).
