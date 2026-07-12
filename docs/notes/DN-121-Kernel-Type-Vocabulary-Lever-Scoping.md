# Design Note DN-121 — The Type-Vocabulary Lever: Scoping the Dominant `checked_fraction` Class

| Field | Value |
|---|---|
| **Note** | DN-121 |
| **Status** | **Draft** (2026-07-11). Authored as a **design-forward reasoner note** scoping the *type-vocabulary* class — the ~40% "Other / type-coverage" gap the zero-hand-port delta-ledger names the dominant `checked_fraction` lever (`zero-hand-port-delta-ledger.md` §2; `delta-L3-transpiler.md` §3, row #1: **322 gaps / 40%**). It **works the decision forward to a ranked, phased build plan and recommends**; it **enacts nothing**, **ratifies nothing**, and **moves no other doc's status** (house rule #3, append-only). It **does not edit** `crates/mycelium-l1/**` (semcore serial lane — read-only), `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` — FLAGGED in §8. Tags are `Empirical` where read against the tree (dev tip `b36f0ef4`, 2026-07-11), `Declared` for any design not yet built/ratified (VR-5). |
| **Decides** | *Proposes, for ratification (does not self-ratify):* (1) a **corrected framing** — the dominant lever is the language's **type-reference-closure** layer, of which only a *bounded* residual needs a **new kernel primitive `Ty` variant**; most of the 322-gap class closes on the **existing kernel `Data`/`Bytes`/`Binary{N}` vocabulary** via **std ADTs + transpiler emit rules + idiom ratification** (per DN-99's already-verified rows); (2) a **phased, leverage-ranked build plan** decomposing the class into individually-buildable units, each with its guarantee model and honest `checked_fraction`-impact tag; (3) the **already-closed / do-not-reopen set** (signed-order ops, closures, string-literal pattern); (4) the **honest tag boundary** and the outstanding **Phase-0 re-measure** that must precede any numeric target. |
| **Feeds** | The delta-ledger (`docs/planning/zero-hand-port-delta-ledger.md` §2/§6 Phase-2); DN-99 (Surface-Gap Closure Register — rows #22/#25/#26/#42/#44/#45/#70/#72); DN-108 (transcendentals, ENB-5, Accepted 2026-07-11); DN-107 (never-type, ENB-7); M-874 (surface type-coverage, needs-design); M-876 (surface completeness — records / external-trait impls / bounded generics, needs-design); M-1028/M-1029/M-1034 (enb E28-1 fast-follows, todo); ADR-028 (Binary is sign-free). |
| **Grounds on** | KC-3 (small kernel — prefer std ADTs on the existing `Ty::Data` variant over new primitive `Ty` variants); DRY (one shared `SInt`/`Char` std type, not the per-file `SInt` now duplicated in `lib/std/ternary.myc`); G2/never-silent (every idiom mapping is EXPLAIN-recorded + FLAG-carried); VR-5 (each phase's `checked_fraction` impact is tagged at its real basis — `Declared` projection until re-measured `Empirical`); KISS/YAGNI (idiom-on-existing-types beats a new kernel primitive wherever it preserves semantics). |
| **Date** | July 11, 2026 |
| **Task** | Scope the kernel type-vocabulary lever (the dominant `checked_fraction` class) — verified inventory + phased plan + adversarial stress-test. Read-only except this DN. |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This note **works a decision forward and
> recommends, ranked**; it does **not** take the decision (house rule #3 — the maintainer ratifies).
> Every claim about existing machinery carries a `file:line` read against dev tip `b36f0ef4`
> (`Empirical`); every design not yet built is `Declared`; every `checked_fraction` number is a
> **pre-enb-wave, file-gated** measurement (`Empirical` as history) whose forward projection is
> `Declared`. **No sycophancy (§4):** the task's own framing — "the lever is *missing kernel
> type-vocabulary*" — is **corrected here on the evidence**, because DN-99's verified rows and the
> live `lib/std/ternary.myc` `SInt` ADT show most of the class closes **without** touching the kernel
> primitive vocabulary. Following the evidence over the framing is the point of the persona (VR-5
> applied to assent).

---

## §1 Purpose and frame

The zero-hand-port delta-ledger measured that faithful **transpiler** rules move `checked_fraction`
by **0**; every real gain came from **language/kernel surface** (`delta-L3-transpiler.md` §4,
`Empirical`: §8.10/§8.11/§8.12 each moved the metric by 0; §8.14 String→`Bytes` and §8.17 kernel
`and`/`or` prims moved it). The single largest gap class is **"Other / type-coverage": 322 instances,
40%** (`delta-L3-transpiler.md` §3 row #1, `Empirical`, DN-34 §8.9 basis). Its members
(cited in that row): **signed ints, `usize`/`isize` platform-width, `char`, `String` [partly closed],
closures [closed], refs [closed]**.

This note inventories that class **against the actual kernel type system**, separates
genuinely-missing from register-stale-already-closed (mitigation #14 — three "gaps" this session were
already closed), and ranks the residual by `checked_fraction` leverage into a phased plan.

## §2 The actual kernel type vocabulary today (`Empirical`, read against dev `b36f0ef4`)

The checker's type universe is the `Ty` enum (`crates/mycelium-l1/src/checkty.rs:78-139`) — **eleven
variants**:

`Binary(Width)` · `Ternary(Width)` · `Dense(u32, Scalar)` · `Vsa{model,dim,sparsity}` ·
`Data(String, Vec<Ty>)` · `Substrate(String)` · `Seq(Box<Ty>, u32)` · `Bytes` · `Float` ·
`Var(String)` · `Fn(Box<Ty>, Box<Ty>)`.

The surface `BaseType` (`ast.rs:590-652`) adds `Named`, `Ambient`, and `Tuple(Vec<TypeRef>)` — but
`Tuple` **desugars to a synthetic single-constructor `Data`** (`ast.rs:646-651`, M-826, `Empirical`),
so it is **not** a new kernel primitive. Every guarantee index rides beside the base as a `TypeRef`
(`ast.rs:556-586`), orthogonal to the type vocabulary.

**Key structural fact (load-bearing for the whole plan):** `Ty::Data(String, Vec<Ty>)` is a
**general parametric ADT constructor** — sum-of-products, applied to type arguments. Any new
*nominal* type a Rust program needs (a signed integer, a `char`, a record) can in principle be a
**registered `Data` declaration in `lib/std`**, requiring **no new `Ty` variant** and therefore
**no edit to the read-only semcore lane**. This is already exploited in the tree (§3.1).

## §3 Verified inventory of the 322 class — genuinely-missing vs already-closed

Ranked by `checked_fraction` leverage (how many transpiled programs each blocks — file-gated, so a
class that "poisons a whole file" outranks one that gaps a single item).

### §3.1 Signed integers — **CLOSEABLE ON EXISTING KERNEL; a std ADT already exists** (`Empirical`)

- `Binary{N}` is **sign-free** by decision (ADR-028; `Empirical`). There is **no primitive signed
  `Ty` variant** and `FLAG-ternary-2` confirms it: "`.myc` has no signed-integer surface type"
  (`lib/std/ternary.myc:36-37`).
- **But the signed *operations* already landed** — `lt_s` two's-complement order
  (`checkty.rs:5270-5330`, DN-72/M-767) and `bin.mul` reads Binary as signed two's-complement
  (`lib/std/ternary.myc:74`, M-887). DN-99 row #44 (`Empirical`): "ops landed … ops need no work."
- **And a signed *type* already exists as a std ADT:** `type SInt = SPos(Binary{16}) | SNeg(Binary{16})`
  (`lib/std/ternary.myc:98`, sign-magnitude over the existing `Data` variant). It is **ad-hoc and
  per-file** (DRY debt), not a shared std type.
- **Verdict:** the closure is **(a) a shared `std.int.SInt`/`INum` ADT** (uses existing `Ty::Data` —
  no kernel change) **+ (b) a transpiler emit rule** mapping Rust `iN`/`isize` onto it **+ (c) idiom
  ratification** (DN-99 §4 A5 / ENB-6 / M-1029; ENB-11 / M-1034 docs). **`Declared` closure design;
  no new kernel primitive.** DN-99 row #26 sizes it M/P2.

### §3.2 Platform-width `usize`/`isize` — **IDIOM ON EXISTING KERNEL** (`Empirical`)

DN-99 row #22 (`Empirical`, `token.rs:477`; ADR-028): map `usize`/`uN` → domain `Binary{N}` **+ a
never-silent width-choice FLAG**. No new `Ty` variant — the escalation to a *platform-abstract width*
is **conditional, only if the fixed-`Binary{N}` idiom proves insufficient under measurement**
(DN-99 §4 A5). Tracked M-1029 (todo). **`Declared`; idiom-on-existing-kernel.**

### §3.3 `char` — **IDIOM ON EXISTING KERNEL** (`Empirical`)

No `Char` type exists (`grep` of `checkty.rs`/`ast.rs` for `char`/`codepoint`/`unicode` → **empty**,
`Empirical`). DN-99 row #25 (`Empirical`, `lex.myc:553`): the codepoint idiom — a Unicode scalar as
`Binary{32}` with a `// 'x'` provenance comment — or route through `Bytes`/`std.text`. **No new kernel
primitive** unless the idiom is rejected. Tracked M-1029 (subsumes char). **`Declared`; idiom.**

### §3.4 `String`/`str` — **CLOSED (the ledger's largest single win)** (`Empirical`)

`String`/string-literals → the **existing `Bytes` kernel type** (`delta-L3-transpiler.md` row
"String literal / `String`", §8.14 "ladder's largest win", `Empirical`). String-literal *patterns*
also closed (DN-99 row #72, M-1035/PR #1372, `Empirical`). **Do not reopen.** Residual: conversion-
method mapping (M-1037) — a transpiler rule, not a type.

### §3.5 Closures / lambdas — **CLOSED** (`Empirical`)

DN-99 row #23 (`Empirical`): `lambda(x)=>e` parses/checks/lowers (M-704/706/822); the checker builds
`Ty::Fn(A,B)` (`checkty.rs:982`, `:4113`). **Do not reopen** as a *type-vocabulary* gap. (Rust
*capturing* closures with environment are a separate L4/L5 **Judgment**-bucket lowering — DN-109 §4 —
not a missing kernel type; out of this note's scope.)

### §3.6 References `&T` — **CLOSED** (`Empirical`)

Erased under value semantics (ADR-003; `delta-L3-transpiler.md` row "`Expr::Reference`", §8.11,
`Empirical`). **Do not reopen.**

### §3.7 The genuinely-open kernel-touching residual (small)

- **Transcendental float numerics** (`sqrt`/`exp`/`ln`/`sin`/`cos`/`pow`) — DN-99 row #42, the one
  **`open`** numeric gap. **Already designed**: DN-108 (Accepted 2026-07-11) resolves it as **`flt.*`
  prims returning the existing `Ty::Float`, no new numeric type, no new kernel node** — impl-open
  (M-1028, todo). So even this is **not a new type**, it is new *prims* over the existing `Float`.
- **Never-type `-> !`** — DN-107 (designed, M-1030). Low type-vocabulary leverage: DN-107 argues
  general-`?` does **not** need it. Deprioritize for `checked_fraction`.

### §3.8 The adjacent classes that ARE genuine type-*system* surface (M-876) — see §4

Records (Struct, 80 / 10%), external-trait impls (Impl, 119 / 15%), bounded generics (GenericBound,
59 / 7%) are **separate ledger rows**, not part of the 322 — but they are the genuine
kernel/language **type-system** extensions (named-field surface; trait-impl surface; a *bounded*
`Var`). §4 argues they out-rank most of the 322 on `checked_fraction`.

## §4 Adversarial stress-test — is "missing kernel type-vocabulary" really the dominant lever? (VR-5, no sycophancy)

**The task's framing is directionally right but the word "kernel" overstates it, and the leverage
ranking partly inverts.** Three findings, each grounded:

1. **Most of the 322 needs NO new kernel primitive.** §3.1–§3.6 show signed/char/usize/string/
   closures/refs close on the **existing** `Ty::Data`/`Bytes`/`Binary{N}`/`Fn` vocabulary — via std
   ADTs + transpiler emit + idiom ratification (DN-99's verified `idiom`/`closed`/`partial` rows;
   the live `SInt` ADT is the proof of concept). The kernel primitive `Ty` enum is **near-complete**.
   Calling the lever "missing *kernel* type-vocabulary" mis-locates the work into the forbidden
   semcore lane when most of it is **std-lib + transpiler + a ratification note**.

2. **Reconciling with "transpiler rules moved the metric by 0."** This is the sharpest objection to
   my own finding, so I confront it: if signed/char are "std ADT + transpiler rule," why did bare
   transpiler rules (§8.10-8.12) move `checked_fraction` by 0 while String→`Bytes` (§8.14) moved it?
   Because those bare rules **emitted forms that still referenced uncheckable types**; §8.14 worked
   precisely because the target type (`Bytes`) **existed and was check-clean**. So the lever is
   *"make the referenced type check-clean end-to-end"* — which for signed/char requires the std ADT
   to **exist AND be emitted onto** (a *coupled* library+transpiler+idiom move), **not** a bare
   transpiler rule and **not** necessarily a kernel primitive. The measurement does not distinguish
   "new kernel primitive" from "new std ADT on the existing `Data` variant" — **it only shows the
   *type-reference* must resolve to something the checker accepts.** DN-99's verified rows settle
   which side of that line each member falls on, and it is mostly the std-ADT/idiom side. This is the
   central honest correction (`Empirical` for the citations; the reconciliation itself is `Declared`
   reasoning).

3. **The leverage ranking partly inverts the premise.** The 322 is the highest-*frequency* class, but
   for `checked_fraction` specifically (file-gated — one poison item zeros a whole file) the
   **M-876 surface trio out-ranks most of the 322**: external-trait **Impl** (119) *"emits but fails
   check, poisons whole file"* (`delta-L3-transpiler.md` row `Item::Impl`, `Empirical`) is the worst
   per-instance damage; **records** (80) and **bounded generics** (59) pervade every stdlib crate and
   need genuine *type-system* surface (named fields; a bounded `Var`), which idiom cannot supply.
   These are the **true "kernel/language type-system" additions** the premise is reaching for — more
   than the idiom-closeable 322. **So if the maintainer wants the single biggest honest
   `checked_fraction` unlock, it is the M-876 surface trio, not the signed/char/width idioms.**

4. **The load-bearing caveat that could defeat the whole ranking (`Empirical`).** The 322/119/80/59
   split is **pre-enb-wave, file-gated** (DN-34 §8.9; `checked_fraction` 7.8% baseline). The enb wave
   (string-literal pattern M-1035, `?`, generic-slot, visibility-seal) has **landed and NOT been
   re-measured on the full 17-target corpus** — the delta-ledger's **Phase-0 re-measure is
   outstanding** (`zero-hand-port-delta-ledger.md` §8, §6 Phase-0; the partial 5-target sample gave
   4.3% union but is not comparable). **No numeric target in §5 may be committed before that
   re-measure.** The ranking below is therefore ordered by *grounded blast-radius reasoning*, with
   its numeric impact explicitly `Declared`.

**Net verdict:** kernel *type-vocabulary* — read as *"the set of types a transpiled program can
reference check-clean"* — **is** the dominant lever (the framing survives). But **most of it is not
kernel-primitive work**: it is std ADTs on the existing `Data` variant + transpiler emit + idiom
ratification, plus a **bounded** genuine-surface set (M-876 records/impl/generics; transcendental
*prims* over the existing `Float`). The persona's job is to say this even though the task framed it as
kernel work: **follow the evidence, not the framing.**

## §5 Recommendation — the phased build plan, leverage-ranked

**Objective function (the criteria table).** Each phase scored on: **Lev** (`checked_fraction`
blast-radius: file-gating damage × frequency), **New-kernel?** (does it touch the read-only L1
primitive `Ty` enum — lower is better, KISS/KC-3), **Design-ready?** (has a ratified/Accepted design
vs needs-new), **Guarantee model preserved** (value semantics · totality · VR-5 tags), **Size**.

| # | Phase (buildable unit) | Lev | New kernel primitive? | Design status | Guarantee model | Size | `checked_fraction` impact (`Declared`) |
|---|---|---|---|---|---|---|---|
| **P1** | **External-trait impl surface** (Impl, 119/15%) | **Highest** (poisons whole files) | **Maybe** — impl-of-external-trait / trait Self-body surface (needs decision: std-side vs kernel) | **needs-design** (M-876) | coherence over concrete types (as `Ty::Var`/`Fn`); totality preserved | L | High — un-poisons whole files across every stdlib crate |
| **P2** | **Record (named-field) surface** (Struct, 80/10%) | High (pervasive) | **Likely small** — named-field projection above the positional `Data` ctor (today `NamedFieldDrop` drops names) | **needs-design** (M-876) | value semantics; positional↔named is a checked view, never silent | M | High — records pervade stdlib |
| **P3** | **Bounded-generic surface** (GenericBound, 59/7%) | High (pervasive) | **Yes, contained** — a *bounded* `Ty::Var` (today `Var` is unbounded, `checkty.rs:131`) | **needs-design** (M-876) | parametric/repr-opaque discipline (S1) preserved under the bound | L | Medium-High — unblocks generic stdlib APIs |
| **P4** | **Signed-int std type + emit + idiom** (§3.1) | Medium-High (frequency) | **No** — shared `std.int.SInt` ADT on existing `Ty::Data` | **idiom, mostly settled** (DN-99 #26/#44; M-1029/M-1034) | sign-magnitude, explicit-`None` out-of-range totality; VR-5 tags on ops | M | Medium — very frequent, but ops already landed |
| **P5** | **Platform-width `usize`/`isize` + `char` idiom** (§3.2/§3.3) | Medium | **No** (conditional escalation only) | **idiom** (DN-99 #22/#25; M-1029) | never-silent width/codepoint FLAG; EXPLAIN-recorded | S | Medium |
| **P6** | **Transcendental prims** (§3.7) | Low-Medium (numeric crates) | **No** — `flt.*` prims over existing `Ty::Float` | **Accepted** (DN-108) impl-open (M-1028) | `Declared` ε; never-silent domain `Result`; compose-refuse | XL | Low-Medium (numeric-heavy targets only) |
| **P7** | **Never-type `-> !`** (§3.7) | Lowest | **No** (divergence model) | designed (DN-107) impl-open (M-1030) | never-silent divergence; `?` decoupled | M | Low (DN-107: `?` doesn't need it) |

**Ranked recommendation:** build in **P1 → P2 → P3** order first (the genuine type-*system* surface
= the biggest honest `checked_fraction` unlock, file-gating-aware), then **P4 → P5** (the
high-frequency idiom closures that need no kernel primitive and no semcore-lane edit), then **P6/P7**
(already-designed, lower/numeric-scoped leverage). **P4/P5 can run in parallel with P1-P3** because
they touch **std-lib + transpiler**, disjoint from the L1 primitive surface P1-P3 touch (the
disjoint-ownership seam — DN-97/mitigation #11).

**Guarantee-model note binding every phase (KC-3/VR-5/G2):** each unit must (1) preserve value
semantics; (2) keep every op **total** or explicitly `Result`/`Option` (never-silent out-of-range);
(3) tag each op at its real basis and **never upgrade** (signed ops `Empirical` where tested,
transcendental ε `Declared`); (4) be **EXPLAIN-recorded** for any idiom mapping; (5) insert **no
`swap`** (S1). An auto-fire idiom fires only under DN-109 §4's conjunctive ratchet.

## §6 Which are already-designed (impl-open) vs need new design

- **Impl-open (design exists, build pending):** P6 transcendentals (**DN-108 Accepted**), P7
  never-type (DN-107), P4 signed (DN-99 idiom + `SInt` ADT precedent), P5 width/char (DN-99 idiom).
- **Need new design (`needs-design`):** **P1/P2/P3 — the M-876 surface trio.** These are the genuine
  design gap: external-trait-impl surface, named-field record surface, bounded-generic surface. **A
  synthetic trait-def was tried and FAILED** (`delta-L3-transpiler.md` row `Item::Impl`, `Empirical`)
  — so P1 in particular needs a real design decision (std-side vs a contained kernel surface), which
  is the natural next DN after this one.

## §7 Honest tag boundary + `checked_fraction` impact

- **`Empirical`:** the `Ty` enum inventory (§2), every `file:line` citation, the pre-enb-wave 322/119/
  80/59 split, the live `SInt` ADT and `lt_s`/`bin.mul` signed ops, the "transpiler rules moved 0 /
  String→`Bytes` moved it" measurements.
- **`Declared`:** every forward `checked_fraction` projection in §5's last column (the split is
  **pre-enb-wave and file-gated**; §4.4); the closure *designs* for P1-P5; the §4.2 reconciliation
  reasoning.
- **Hard boundary (VR-5):** **no numeric `checked_fraction` target may be committed** until the
  delta-ledger **Phase-0 full 17-target re-measure** lands (`zero-hand-port-delta-ledger.md` §6/§8).
  This note ranks by grounded blast-radius, not by a projected percentage.

## §8 Definition of Done (for maintainer ratification — this note moves to Accepted only when)

1. The maintainer **confirms or corrects the §4 reframing** — that the dominant lever is
   *type-reference closure* (mostly std-ADT/idiom/transpiler on the existing kernel), **not** new
   kernel primitives — or rejects it with grounded reason.
2. The maintainer **ratifies the P1-P7 ranking and the P1→P2→P3-first ordering** (or reorders), in
   particular endorsing that **the M-876 surface trio out-ranks the idiom-closeable 322 on
   `checked_fraction`** (§4.3).
3. **P1 (external-trait impl surface)** is assigned its own **new design DN** (the §6 open decision:
   std-side vs contained kernel surface) — this note scopes, it does not design P1.
4. The **Phase-0 re-measure** is scheduled as the gate before any numeric target (§7).
5. The FLAGs in §8.1 are actioned by the owning integrator/lane (this note edits none of them).

### §8.1 FLAGs (append-only, dated — owned elsewhere, this note edits none)

- **FLAG-DN-index (Doc-Index.md):** add a Design-Notes row `DN-121 — The Type-Vocabulary Lever
  (Draft, 2026-07-11)`. *(Doc-Index is orchestrator-owned — read-only here.)*
- **FLAG-CHANGELOG:** add an append-only entry for DN-121 (Draft). *(CHANGELOG is orchestrator-owned.)*
- **FLAG-issues (`issues.yaml`):** (a) M-1029 body already names the `Binary{N}` idiom + char/signed
  subsumption — align its `doc_refs` to add `corpus:DN-121`; (b) M-876 (records/impl/generics) is the
  **P1-P3 build spine** — this note recommends it be **split into three tracked units** (impl / record
  / bounded-generic) per §5, and re-prioritized to reflect its §4.3 top-of-`checked_fraction` leverage
  (currently P2/P3). *(issues.yaml is orchestrator-owned — FLAG only.)*
- **FLAG-DN-slot (mitigation #1/#14):** on-disk notes max at **DN-113** on this dev tip; the task
  states **DN-114-120 are in-flight in other sessions** (not visible here). **DN-121 is confirmed free
  on this tree** (no `DN-121` reference anywhere in `docs/`/`tools/`), but the integrator **must
  confirm 114-120 landed** before ratifying, in case a slot was skipped.

## §9 Changelog

- 2026-07-11 — initial Draft. Scopes the type-vocabulary lever; corrects the "missing *kernel*
  type-vocabulary" framing to *type-reference closure* on the existing kernel (VR-5, §4); ranks P1-P7
  by file-gating-aware `checked_fraction` leverage; flags the outstanding Phase-0 re-measure as the
  numeric-target gate. Grounded on dev tip `b36f0ef4`.
