# Design Note DN-106 — Statement-Sequencing (`let _`) + Record-Update / Mutation Split: a Triage (ENB-10)

| Field | Value |
|---|---|
| **Note** | DN-106 |
| **Status** | **Draft** (2026-07-10). Authored as the **triage** of M-1033 (ENB-10) / DN-99 register row **#89**, per mitigation #14 (*verify a stale issue's claim against the codebase before implementing*). It records what is **genuinely open** in the L1/semcore lane versus what is **already closed** or belongs to the **transpiler lane** — and finds, on investigation, that **both** sub-gaps' language side is already closed, three-way witnessed. It **enacts nothing** and **moves no other doc's status** (house rule #3, append-only). Tags are `Empirical` where read against the code / witnessed by a running differential, `Declared` for any design not yet ratified (VR-5). |
| **Decides** | *Proposes, for ratification:* (1) **Part 1 (statement-sequencing, `let _ = e in body`) is ALREADY CLOSED at the L1 level** — grammatical (`ebnf:291` `let_expr ::= 'let' Ident … 'in' expr`; `Ident` admits `_`), parsed (`parse.rs::parse_let` → `ident()`), checked (`checkty.rs::check_let`), evaluated + elaborated (three-way witnessed, §7) — and is moreover the **established affine drop/use-once surface** (`src/tests/affine.rs`; DN-71/M-903). Its sole residual is the **transpiler emit** of `let _ = e in body` for value-producing discarded statements, which lives in `crates/mycelium-transpile` (Part 1 of the issue), **not** in the semcore lane. (2) **Part 2 (record-update / mutation→functional) needs NO new L1 grammar.** The functional-update **target form** — `match base { Ctor(f0, …, fN) => Ctor(f0, …, NEW_fk, …, fN) }` (destructure-and-reconstruct) — is **already expressible** (three-way witnessed, §7). Mycelium has **no named-field record literal and no field-projection expression by design** (value-semantic **positional** constructors; §2), so a `{ ..base, field: v }` record-update literal is **intentionally absent** and its addition to L1 is **rejected** (§3, fork B). The mutation→functional rewrite and the struct-update→reconstruct rewrite are **transpiler translation rules** (`crates/mycelium-transpile`), and the *split* between a functional update and an in-place mutation is a transpiler translation **policy** that must never fabricate mutation the transpiler was not taught (G2). (3) **Correction (mitigation #14):** the M-1033 issue body's framing of Part 2 as *"grammar-`enb`, HIGH collision, touches `crates/mycelium-l1/**`, coordinate M-1013"* is **over-scoped**; it disagrees with DN-99's own register row #89, which already tags #89 **layer=`tr`, collision=`low`**. **M-1033's L1/semcore residual is NIL**; the real residual is entirely in the transpiler lane. It does **not** edit `issues.yaml`, `CHANGELOG.md`, `Doc-Index.md`, or the DN-99 register (the integrating session owns those — §8 lists the reconciliations to apply). |
| **Feeds** | DN-99 §8 / register row **#89** (statement-sequencing-body) — the L1 side confirmed **already-closed**, the residual confirmed **transpiler-lane** (mitigation #14 correction of the M-1033 issue-body over-scope; the register's own row #89 tags already agree — layer `tr`, collision `low`). ENB-10 / M-1033 — L1/semcore residual scoped to **NIL**; Part 1 (`let _` emit) and Part 2 (mutation→functional rewrite) hand off to `crates/mycelium-transpile`. DN-71 / M-903 (`let _` as the affine drop/use-once surface). DN-26 (SCC self-hosting — the `.myc` mirror already carries `let`/`match`/ctor-application; **no mirror change**). KC-3 (the value-semantic positional-constructor design that makes a record-update literal unnecessary). |
| **Grounds on** | Mitigation #14 (the tracker is `Declared`; the codebase is ground truth — verify before building), VR-5 (no closure claimed past its basis; the "already-closed" claims are `Empirical`, witnessed by the §7 three-way differentials, not asserted), KC-3 / KISS / YAGNI (do not add a record-update literal + field-projection subsystem the language deliberately omits, when destructure-and-reconstruct already expresses the target), G2 (never-silent — the transpiler *gaps* an untranslatable mutation rather than fabricate one; `{ ..base, … }` is an explicit parse refusal today, §7), house rule #4 (no sycophancy — §3 confronts the real fork on its merits and §6 states the residuals plainly, including that this note ships **no** L1 code change because none is warranted). |
| **Date** | July 10, 2026 |
| **Task** | M-1033 (ENB-10) — statement-sequencing (`let _`) + record-update / mutation split. |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This note records a **triage**, not an
> implementation. Its central, potentially-unwelcome finding is that **there is no L1/semcore work to do
> for M-1033** — both sub-gaps are already closed at the language level, and the real residual is in the
> transpiler lane the issue body mis-assigned. That is a mitigation-#14 correction reported on the
> evidence, not softened to manufacture a deliverable (house rule #4: *be corrected over being wrongly
> affirmed*). The "already-closed" claims are `Empirical` — witnessed by the running three-way
> differentials in §7 (pinned as regression tests so the closure cannot silently regress). The fork
> resolution (§3) is `Declared` until the maintainer ratifies.

---

## §1 Purpose

Triage M-1033 (ENB-10), which DN-99 §8 splits into two parts against register row **#89**
(statement-sequencing-body):

- **Part 1 — statement-sequencing `let _ = e in body`** — discard a value-producing (side-effecting)
  expression's result in sequence, then yield the body. The issue assigns this to `crates/mycelium-transpile`
  (emit) with `low` collision and **no** semcore coordination.
- **Part 2 — record-update / mutation→functional split** — the issue body frames this as a *"grammar-`enb`,
  HIGH collision"* gap *"that touches `crates/mycelium-l1/**`"* needing this Draft DN and M-1013
  coordination.

Per mitigation #14, before building anything the claim is checked against the checker/grammar/evaluator.
The finding (§7): **both parts' language side is already closed.** This note records that, corrects the
over-scope, and scopes M-1033's L1 residual to nil — pinning the closure with regression witnesses so it
cannot silently regress (VR-5: the codebase is ground truth).

## §2 The language shape that settles the triage — positional constructors, no field projection

Mycelium's data surface is **positional constructors**, not named-field records
(`ast.rs::Ctor { name, fields: Vec<TypeRef> }`; `type Pair = Mk(Binary{8}, Binary{8})`). The `Expr` enum
has **no** record-literal / struct-literal node and **no** field-projection (`x.field` / `x.0`) node — the
full variant set is `Let · If · Match · For · Swap · WithParadigm · Wild · Spore · Wrapping · Consume ·
Try · Colony · Lambda · App · Fuse · Reclaim · Path · Lit · Ascribe · TupleLit`. Field **read** and field
**update** are therefore both done by **`match`-destructuring**: a value is taken apart with a
`Pattern::Ctor` and rebuilt by applying the same constructor. This is a deliberate value-semantic /
small-kernel design (KC-3), not a missing feature. Two consequences fix the whole triage:

1. **Statement-sequencing needs no dedicated construct.** `let _ = e in body` is an ordinary `let` whose
   binder happens to be the identifier `_` (a legal `Ident` — `ebnf:447` `Ident ::= (Letter | '_') …`). The
   checker binds it and never enforces "must use" on a plain binding, so the value is simply discarded.
2. **Functional field-update needs no dedicated construct.** Updating field *k* of an *N*-field value is
   `match base { Ctor(f0, …, fN) => Ctor(f0, …, NEW_fk, …, fN) }` — existing `Match` + constructor `App`.
   There is nothing for a `{ ..base, field: v }` literal to *add* except surface sugar, and the language
   has deliberately chosen not to carry field-named surface at all.

## §3 The genuine fork for Part 2 — transpiler rewrite vs. an L1 record-update sugar

Two real designs could "close" Part 2; they are very different sizes and only one is in the semcore lane:

- **(A) Transpiler translation rules only *(proposed)*.** The transpiler
  (`crates/mycelium-transpile`) translates Rust struct-update (`Foo { field: v, ..base }`) and Rust
  in-place mutation (`base.field = v; …`) into the already-expressible Mycelium target: a
  destructure-and-reconstruct `match` (functional update) resp. a functional **rebind**
  (`let base = Foo(v, base.1, …) in …`). It needs the struct's field arity + the field's positional index
  (transpiler-side `StructLayout` info) and reuses existing L1 `Match`/`App`. **Zero L1 change.** The
  *split* — never conflating a functional update with a mutation, and never fabricating a mutation the
  transpiler was not taught — is a transpiler **policy** (G2: an untranslatable mutation is *gapped*
  never-silently, not faked). This is exactly what DN-99 #89's own register tags already say (layer `tr`,
  collision `low`), and what the string-dispatch / port targets actually need.

- **(B) Add a record-update literal (+ field projection) to L1.** Introduce `{ ..base, field: v }` (and,
  to be coherent, named-field construction and `x.field` projection) as first-class L1 surface. This is a
  **large, separate** subsystem: new AST nodes, a named-field resolver over the positional `Ctor` model,
  new checker/usefulness/eval/elab/mono/fmt/lsp handling, a full silent-hole sweep across every walker —
  and it **contradicts the deliberate positional-constructor / value-semantic design** (§2, KC-3). It buys
  only surface ergonomics over the already-working destructure-and-reconstruct.

**Resolution: (A).** KISS / YAGNI / KC-3 — (A) delivers the whole of Part 2 with **no** L1 growth over
machinery that already exists and is already three-way witnessed (§7); (B) is a design in its own right
that runs against the language's chosen data model, with no port driver that destructure-and-reconstruct
does not already serve. (A) forecloses nothing: were a named-field record surface ever wanted, it would be
an append-only extension, not a prerequisite for the port. **(B) is rejected**, recorded as a
non-adopted alternative (§6), not left implicit (house rule #4).

## §4 Part 1 is the affine drop surface (why "already closed" is load-bearing, not incidental)

`let _ = e in body` is not merely tolerated — it is the **canonical affine `Substrate` drop/use-once
surface** (DN-71 Model S / M-903, exercised throughout `src/tests/affine.rs`):
`let _ = s in True` consumes `s` exactly once and checks; `let _ = s in s` is a never-silent
`double-consume` refusal; `let _ = a in let _ = b in True` drops two distinct substrates independently.
The consume is charged by the **reference in the bound position** (`use_at`), and the `_` binding then
holds the moved value; an unconsumed `_`-bound value is not a "must-use" error. This is working,
tested, load-bearing behavior. Any change to the `let _` construct (e.g. making `_` a non-binding
discard so `let _ = e in _` refuses) would be an unwarranted, risk-bearing edit to a working affine
primitive for negligible benefit — the referenceable-`_` behavior is harmless and even consistent (`_` is
just an `Ident`), never a soundness issue. **It is therefore explicitly NOT changed** (YAGNI; §6.3).

## §5 The DN-26 dual — no L1 change, so no `.myc` mirror change

Because the triage closes with **no L1 code change**, there is **no** silent-hole sweep to run (no AST
node/field added → the exhaustive walkers, the fingerprint walker, the three `classify_expr` copies, the
five `.myc` mirror encoders, the stage-3/4/5 encoders, and cross-crate `mycelium-fmt`/`mycelium-lsp` are
**untouched**, and correctly so). The `.myc` self-hosted frontend (`lib/compiler/parse.myc`,
`semcore.myc`) already carries `let`, `match`, and constructor-application as core forms
(`lib/compiler/ast.myc`), so `let _ = e in body` and destructure-and-reconstruct are already expressible
in the mirror too. The DN-26 dual is satisfied by the **existing** dual surfaces; this note adds only the
**pinning witnesses** (§7) on the Rust leg.

## §6 Residuals (stated plainly, not hidden)

1. **Part 1 transpiler emit (`crates/mycelium-transpile`).** Emit `let _ = e in body` for value-producing
   discarded statements in `Stmt::Local` — `low` collision, no semcore coordination. Owned by the
   transpiler lane; out of scope for this L1/semcore triage.
2. **Part 2 transpiler translation rule (`crates/mycelium-transpile`).** The mutation→functional rewrite
   (struct-update → destructure-and-reconstruct; in-place mutation → functional rebind), with the
   never-fabricate split as a G2 policy. Owned by the transpiler lane; needs a **transpiler-lane** DN if a
   formal rule is wanted (this note supplies the target-form semantics it targets). Fork (A) above.
3. **The referenceable-`_` behavior is intentionally left as-is** (§4). Making `_` a non-binding discard is
   *not* adopted — it is a behavior change to the load-bearing affine drop surface with regression risk and
   no benefit. Non-defect; recorded so the choice is never-silent.
4. **Record-update sugar / named-field surface (fork B, §3)** — not adopted; contradicts the positional
   constructor design (KC-3). An append-only future extension with no current driver.

## §7 Definition of Done + witnesses

**Definition of Done (from M-1033).** Part 1: the transpiler emits `let _ = e in body` (transpiler lane —
this triage confirms the L1 target is valid and pins it); Part 2: the mutation→functional rewrite is
scoped to a separate (transpiler-lane) design, **not** conflated, and neither part fabricates mutation the
transpiler was not taught (never-silent). This note additionally records, per its own DoD, *what "done"
means for the L1/semcore lane*: **nil residual, witnessed closed.**

**Witnesses (this increment — `crates/mycelium-l1/tests/enablement.rs`, `assert_three_way`: L1-eval ≡
elaborate→L0-interp ≡ trampoline-AOT).**

- **Statement-sequencing three-way.** `fn main() => Binary{8} = let _ = not(0b0000_0000) in 0b0000_0001;`
  — the discarded `not(…)` is evaluated for effect and its value dropped; all three paths yield the body's
  `0b0000_0001`, identically. Confirms Part 1's L1 target is a valid, value-correct Mycelium program.
- **Functional field-update three-way.** A destructure-and-reconstruct
  `match Mk(a, b) { Mk(a, b) => Mk(a, NEW) }` observed through a projector `fn snd` — all three paths
  yield the updated field. Confirms Part 2's target form is already expressible and value-correct.
- **Never-silent record-update refusal (parse).** `{ ..p, 1: v }` is an explicit `ParseError`
  ("expected an expression, found `{`") — the `{ ..base, field: v }` surface is intentionally absent, and
  its absence is a never-silent refusal, not a silent mis-parse (G2). Pinned as a `*_reject`.

Honesty tags: the "already-closed" closures are `Empirical` (witnessed by the running three-way
differentials above); the §3 fork resolution and the §6.3 non-change decision are `Declared` until the
maintainer ratifies (VR-5).

## §8 Reconciliations for the integrating session (this note does not apply them)

Per the enb-DN convention (cf. DN-105), this note does **not** edit the shared collision surface. The
integrating session should, on ratification:

- **`tools/github/issues.yaml`** — record on M-1033 the mitigation-#14 triage: L1/semcore residual **NIL**
  (both sub-gaps closed at the language level, three-way witnessed); the real residual is transpiler-lane
  (Part 1 emit + Part 2 mutation→functional rewrite in `crates/mycelium-transpile`). Add `corpus:DN-106`
  to `doc_refs`. If M-1033 is retitled/re-scoped to the transpiler residual, note the split from the
  (already `low`/`tr`) register row #89.
- **`docs/notes/DN-99-Surface-Gap-Closure-Register.md`** — annotate row #89 that the L1 side is
  confirmed already-closed (three-way witnessed, DN-106), the residual is transpiler-only (as the row's own
  `tr`/`low` tags already indicate), and the M-1033 issue-body "grammar-`enb`/HIGH/mycelium-l1" framing was
  an over-scope corrected here.
- **`CHANGELOG.md`** — an entry for DN-106 (Draft): the ENB-10 triage + the `enablement.rs` pinning
  witnesses.
- **`docs/Doc-Index.md`** — register DN-106.

---

## Changelog

- **2026-07-10** — DN-106 created (**Draft**): the ENB-10 triage. Finds (mitigation #14) that **both**
  M-1033 sub-gaps' language side is already closed — Part 1 (`let _ = e in body` statement-sequencing,
  also the affine drop surface) and Part 2 (functional field-update via destructure-and-reconstruct) are
  three-way witnessed today — so **M-1033's L1/semcore residual is NIL**; the real residual is
  transpiler-lane (`let _` emit + the mutation→functional rewrite). Corrects the M-1033 issue-body
  over-scope of Part 2 (the DN-99 register's own row #89 already tags it `tr`/`low`), rejects an L1
  record-update literal (fork B — contradicts the positional-constructor design, KC-3), and adds the
  pinning three-way witnesses to `enablement.rs`. Enacts nothing, moves no other doc's status
  (append-only); does not edit `issues.yaml`/`CHANGELOG`/`Doc-Index`/DN-99 (§8 — integrating session owns).
