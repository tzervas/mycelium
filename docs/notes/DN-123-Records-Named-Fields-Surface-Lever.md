# Design Note DN-123 — The Records / Named-Fields Surface Lever (P2): Sugar-over-Positional, Not a Kernel Primitive

| Field | Value |
|---|---|
| **Note** | DN-123 |
| **Status** | **Draft** (2026-07-11). Authored as a **design-forward reasoner note** working DN-121's **P2 lever** — the record / named-field surface (Struct, 80/10% of the `checked_fraction` type-vocabulary class) — forward to a **ranked recommendation**. It **works the decision forward and recommends, ranked**; it **enacts nothing**, **ratifies nothing**, and **moves no other doc's status** (house rule #3, append-only — the maintainer ratifies). It **does not edit** `crates/mycelium-l1/**` (semcore serial lane — read-only), `crates/mycelium-transpile/**`, `issues.yaml`, `CHANGELOG.md` beyond the append-only `[Unreleased]` entry for this note, or the DN-99 register — FLAGGED in §8. Tags are `Empirical` where read against the tree (dev tip `46006994`, 2026-07-11), `Declared` for any design not yet built/ratified (VR-5). |
| **Decides** | *Proposes, for ratification (does not self-ratify):* (1) the **verified state correction** — the record / named-field surface is **already substantially supported at the `checked_fraction` level**: the transpiler already carries a field-name↔index map (`StructLayout = Vec<Option<String>>`, `emit.rs:28`), already desugars Rust struct literals / field-projection / struct-update to positional `Data` (`struct_layout`-gated), and already emits named-field structs positionally with names **recorded, not lost**, as a never-silent `Category::NamedFieldDrop` sub-gap (`emit.rs:2200–2429`, `gap.rs:72`). The genuine residual is **not** expressibility but **(i) faithfulness** (dropped field names) and **(ii) the self-hosted `.myc` surface** (no named-field construction / `.field` access / `Foo { x, y }` pattern — the DN-119 **L3-G1** grammar residual). (2) The **ranked recommendation**: build the named-field surface as a **mechanically-lowering SUGAR** over the existing positional `Ctor`/`Data` machinery plus a field-name↔index map (**Option A**), exactly per DN-106's ratified **General Principle 2** (gap-closure default = the mechanically-lowering sugar, not a kernel primitive) and **General Principle 1** (surface-sugar transparency — reveal on demand). (3) The **do-not-reopen boundary**: a first-class named-field `CtorInfo`/`Ty` variant with its own resolver plus checker/eval/elab semantics (**Option B**) stays **rejected** — DN-106 §3 fork (B), KC-3, value-semantic positional design. (4) The **honest tag boundary + open questions** (§6/§7): content-addressed identity vs field order/names, cross-phylum name metadata (DN-112/DN-113), functional-update affine treatment (M-919), struct-pattern exhaustiveness plus the DN-104 ctor seal. |
| **Feeds** | DN-121 §5 **lever P2** (records/named-field surface, `needs-design`, M-876) — this note is that design; DN-119 **L3-G1** (struct/named-field patterns, the genuine L1-grammar residual) plus DN-119 §"deliberate exclusions" (which listed named-field record *type* surface as excluded citing DN-106 — this note records that DN-106's **ratification** re-scoped the SUGAR into scope, §2); DN-106 (ratified General Principles 1 and 2 — the sugar-over-positional default this note applies); DN-99 register (Struct rows — the transpiler-lane `NamedFieldDrop` sub-gap); M-876 (surface completeness — records / external-trait impls / bounded generics, `needs-design`); DN-112 (M-1036, nodule-qualified home identity) plus DN-113 (M-1060, cross-phylum import/resolution) — the cross-phylum name-metadata interaction (§7 OQ-2); DN-104 (M-1027, per-constructor visibility seal — §7 OQ-4); M-919 (affine tracker — §7 OQ-3); DN-120 / ADR-003 (content-addressed identity — §7 OQ-1). |
| **Grounds on** | **DN-106 GP2** (gap-closure default: build the mechanically-lowering sugar, not a kernel primitive — the ratified basis for Option A over B); **DN-106 GP1** (surface-sugar transparency — the sugar hides nothing, expands/reveals the lower positional grammar on demand; house rule #2, no black boxes); **KC-3** (small auditable kernel — zero L0/`Ty`/eval growth; the positional `Data` model is untouched); **KISS/YAGNI** (the transpiler half already exists; retain-vs-drop names is the only new bit, and runtime name retention is added only if a port driver needs it); **DRY** (reuse the existing `StructLayout` map plus `Match`/`App` destructure-and-reconstruct — no parallel resolver); **value semantics plus ADR-003 content-addressed identity** (identity stays **positional/structural**; names are inert metadata, never in the hash — §7 OQ-1); **G2/never-silent** (name-drop is recorded today; the sugar's desugaring is EXPLAIN-revealable; an unresolved field name is a never-silent refusal); **VR-5** (the whole proposed design is `Declared` until built plus differential-witnessed). |
| **Date** | July 11, 2026 |
| **Task** | Work DN-121's records/named-fields lever (P2) forward — verify-first inventory plus ranked recommendation plus adversarial stress-test plus DoD. Read-only except this DN plus its append-only Doc-Index/CHANGELOG rows. |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This note **works a decision forward and
> recommends, ranked**; it does **not** take the decision (house rule #3 — the maintainer ratifies).
> Its central, potentially-unwelcome finding — reported on the evidence, not softened to manufacture a
> deliverable (house rule #4: *be corrected over being wrongly affirmed*) — is that **records are not the
> large open lever the "P2 / 80 gaps / needs-design" framing suggests.** The transpiler already lowers
> Rust records to positional `Data` faithfully-in-structure and check-clean; the field-*name* drop is a
> recorded sub-gap, not a `checked_fraction` blocker (§2, `Empirical`, read against the tree). So the
> genuine work is **faithfulness plus the self-hosted surface**, and the correct shape for it was already
> **settled by DN-106's ratification** (the mechanically-lowering sugar, GP1/GP2) — this note confirms
> that shape against the code and scopes the residual, rather than re-designing a kernel feature the
> language deliberately omits.

---

## §1 The problem, precisely

Rust structs with named fields and struct-variant enums are among the most common constructs a
mechanical Rust→Mycelium port must express (zero-hand-port north star). The concrete surface obligations:

| # | Rust construct | Example |
|---|---|---|
| C1 | Named-field struct **type** | `struct Foo { x: u8, y: u8 }` |
| C2 | Struct-**variant** enum | `enum E { A { x: u8 }, B(u8) }` |
| C3 | Field **access** (projection) | `foo.x` |
| C4 | Named-field **construction** | `Foo { x: 1, y: 2 }` |
| C5 | Struct **pattern** | `match f { Foo { x, y } => … }`, incl. `Foo { x, .. }` rest |
| C6 | **Functional update** | `Foo { x: 9, ..base }` |

**The Mycelium-native answer (DN-111 / DN-110 taxonomy).** None of these is a *native-equivalent* — Mycelium
has no named-field record type by design. Every one is an **Idiomatic Remapping** (DN-111 §3.2 / DN-110
"Solution") onto the **positional constructor** model: a value is built with a positional `Ctor` and taken
apart / rebuilt by `match`-destructure-and-reconstruct (DN-106 §2). The **underlying problem** each construct
solves — *"name the k-th component so the source is readable and order-insensitive"* — is served natively by
positional components **plus a field-name↔index map**; the map is the whole of the remapping. This is the
same shape DN-106 §2 already established for C6 (functional update) and DN-121 §5 assigned to P2.

## §2 Verify-first — what already exists vs. the gap (`Empirical`, dev tip `46006994`)

**Core AST/checker: positional-only, confirmed.** `ast.rs::Ctor { name, fields: Vec<TypeRef>, sealed }`
(fields are positional `TypeRef`, `ast.rs:355–376`); `checkty.rs::CtorInfo { name, fields: Vec<Ty> }` and
`DataInfo` (positional `Vec<Ty>`, `checkty.rs:237–249`); `Pattern::Ctor(String, Vec<Pattern>)` — positional,
**no** `Struct`/record variant (`ast.rs:994`, matching DN-119 L3-G1). The `Expr` enum has **no**
record-literal and **no** field-projection node (DN-106 §2, re-verified). `Tok::LBrace`/`RBrace`/`Dot` exist
but only for paths / trait bodies / blocks — there is **no** `foo.x` expression form and **no** `Foo { … }`
construction in the self-hosted grammar. **The kernel and the self-hosted surface are positional-only. No
partial named-field support exists at L1.**

**Transpiler: named-field support is largely built — this is the load-bearing correction.**
- `emit.rs:28` `type StructLayout = Vec<Option<String>>` — **the field-name↔index map already exists.**
- `transpile.rs:245` `struct_layouts()` builds a `HashMap<String, Vec<Option<String>>>` for every in-file struct.
- `emit.rs` desugars Rust **struct literals, field-projection, and struct-update** to positional form,
  **gated on `struct_layout(name)` resolving** (`emit.rs:124/152/212/1420/1441/1523`).
- `map_named_fields_positional()` (`emit.rs:2213`) maps a `FieldsNamed` struct to positional field **types**,
  returning the dropped names; the caller records them as a never-silent **`Category::NamedFieldDrop`** sub-gap
  (`emit.rs:2299/2423`, `gap.rs:72/125`). A named-field struct **emits exactly like a tuple one** — structure
  preserved, names dropped-and-recorded. An **unmapped field TYPE** (e.g. `String`) still refuses the whole
  record (`on_type_gap`), never a partial emission (VR-5/G2).

**Net finding (register-lag corrected, mitigation #14).** Records **are not** a large open expressibility
gap. Rust records already **emit positionally and check-clean** whenever their field *types* map; the
`checked_fraction` blocker for a record is an **unmapped field type**, not its named-fieldness. What is
genuinely missing is **two** things, neither of which is "a new kernel primitive":
1. **Faithfulness** — field names are dropped (`NamedFieldDrop`), so ported code loses round-trip
   readability (DN-119 C5, "medium"). The names are *recorded* in the sub-gap but not *carried*.
2. **The self-hosted `.myc` surface** — hand-written / self-hosted Mycelium can only write positional
   constructors; there is no named-field construction / `.field` / `Foo { x, y }` pattern sugar. This is
   the **DN-119 L3-G1** grammar residual (semcore-serial, size S).

**One append-only reconciliation this note records (does not perform).** DN-119 §"deliberate exclusions"
listed *"named-field record type surface"* as excluded, citing DN-106 rejecting the record-update literal.
That was true of DN-106 **at Draft**; DN-106's **ratification note** (Accepted 2026-07-11) re-scoped it:
fork (B) as a *first-class L1 construct* stays rejected, but **as a mechanically-lowering surface sugar it is
"now explicitly in scope to carry"** (GP1), and GP2 makes that the **default** for any missing surface
construct. So the named-field surface is **not** in the deliberate-exclusion set — it is a sugar to build.
This note surfaces that tension and resolves it on the ratified basis, rather than glossing it (house rule #4).

## §3 The real alternatives

### Option A — Mechanically-lowering sugar over positional `Data` plus a field-name↔index map *(recommended)*
Surface named-field construction `Foo { x, y }`, field-projection `foo.x`, struct pattern `Foo { x, y }`
(incl. `..` rest), and functional-update `Foo { x, ..base }` are **surface sugar** that desugars — in the
transpiler (Rust ports, already largely built) and in a new self-hosted parser pre-pass (`.myc`) — to the
existing positional machinery: positional `Ctor` `App` for construction, `match`-destructure-and-reconstruct
for projection/update, positional `Pattern::Ctor` (with wildcards at omitted indices) for patterns. The
field-name↔index map is the `StructLayout` (transpiler) resp. a parse-time table keyed to the type
declaration (self-hosted). **Runtime `Data` value and its identity are purely positional** — names exist only
during desugaring and in the EXPLAIN/reveal trace (GP1: `desugar`/`expand`/`EXPLAIN` shows the dev the lower
positional grammar). **Kernel growth: zero** (no L0 node, no `Ty` variant, no eval/elab path — reuses `Match`
plus `App`, exactly as tuple/or-pattern sugar already does, `ast.rs:999`/`1015`).

### Option B — First-class named-field `CtorInfo` / `Ty` variant in the kernel *(rejected)*
Names become part of the type's **semantic** model: a named-field resolver over the ctor model, named
construction/projection as **core** ops, new checker/usefulness/eval/elab/mono/fmt/lsp handling and a
silent-hole sweep across every walker. **Kernel growth: large.** **Contradicts** the deliberate
positional-constructor / value-semantic design (DN-106 §2/§3). This is DN-106 fork (B) as a first-class
construct — **explicitly rejected at DN-106's ratification** (KC-3 core decision not reopened). Listed only
to be ruled out on the merits (house rule #4).

### Option C — Hybrid: Option A's sugar **plus** field names retained as inert type-registry metadata
Option A, but the field-name↔index map is additionally retained at **runtime** as an **optional, inert**
`field_names: Vec<Option<String>>` on the registry `DataInfo`/`CtorInfo` — **never** on the `Ty` identity,
**never** consulted by type-checking / eval / identity — so EXPLAIN/reveal and **faithful round-trip**
(`Data` value → `Foo { x: … }` rendering) work at *runtime*, not only at compile time. **Kernel growth:
minimal** (one optional metadata field, semantically inert). This is the *faithfulness upgrade* over A, not a
different semantics.

## §4 Evaluation against the house rules plus zero-hand-port goal

| Criterion (objective function) | Weight | Option A (sugar, names compile-time) | Option B (kernel variant) | Option C (sugar plus inert metadata) |
|---|---|---|---|---|
| **Expressibility** (C1–C6 all portable) | high | **Yes** — all six desugar | Yes | **Yes** |
| **Mechanical/reliable lowering** (zero-hand-port) | high | **Yes** — deterministic map→positional | Yes (but bespoke) | **Yes** |
| **Transparency / reveal on demand** (GP1, house rule #2) | high | **Yes** — desugar/EXPLAIN shows positional | Partial — names *are* the semantics, less to reveal | **Yes** (plus runtime reveal) |
| **Small auditable kernel** (KC-3) | high | **Best — zero growth** | **Worst — large subsystem** | Good — one inert metadata field |
| **Value semantics / content-addressed identity** (ADR-003) | high | **Clean** — identity positional | Risk — named identity temptation | **Clean** — names inert, off the hash |
| **DN-106 GP2 gap-closure default** | high | **Matches exactly** | **Violates (rejected fork B)** | Matches (A plus faithfulness) |
| **Faithfulness / round-trip readability** (DN-119 C5) | medium | Compile-time only | Yes | **Yes at runtime** |
| **Reuse of built machinery** (DRY) | medium | **Best** — `StructLayout` exists | None | Best (plus small retain) |
| **Interaction w/ affine (M-919), cross-phylum (DN-112/113), seal (DN-104)** | medium | Bounded (§7) | Amplified (new surface everywhere) | Bounded (§7) plus metadata-export OQ |

## §5 Recommendation — ranked

1. **Option A — the mechanically-lowering sugar (recommended).** Smallest, most mechanical, most
   transparent; matches DN-106 GP2's ratified default exactly; the transpiler half already exists
   (`StructLayout` plus `struct_layout`-gated desugaring), so the marginal work is (a) turning the recorded
   `NamedFieldDrop` into a *reveal-carrying* desugar trace and (b) the self-hosted `.myc` parse-time sugar
   (construction / `.field` / struct pattern — the DN-119 L3-G1 grammar unit). **Transparency (GP1) is
   satisfied by A alone**: reveal is a *compile-time desugar* operation (`expand`/`EXPLAIN` → the positional
   grammar), which needs no runtime name retention.
2. **Option C — adopt only if a port driver needs *runtime* name reveal / faithful round-trip
   (YAGNI-gated).** The one inert `field_names` metadata field is a bounded, append-only upgrade over A; add
   it when DN-119 C5 faithfulness (e.g. a reveal-mode that re-renders a live `Data` as `Foo { x: … }`)
   becomes a real requirement — not preemptively.
3. **Option B — rejected.** DN-106 ratification, KC-3, value-semantic positional design.

**Guarantee tag.** The whole proposed design is **`Declared`** — a proposed sugar not yet built. Each piece
upgrades independently: the transpiler desugaring is `Empirical` *for structure* today (it emits and `myc
check`-passes), and the *name-faithful* upgrade stays `Declared` until a **differential witness** (a
round-trip: Rust record → positional `.myc` → reveal → same field names) exists. No claim is upgraded past a
checked basis (VR-5). This matches DN-121 P2's own "`Declared` projection until re-measured `Empirical`".

## §6 Objective function (explicit)

Maximize, in priority order: **(1) mechanical zero-hand-port expressibility of C1–C6**, subject to a hard
constraint of **(2) zero kernel-semantic growth (KC-3) plus preserved value-semantic content-addressed
identity (ADR-003)**, then **(3) transparency/reveal (GP1)**, then **(4) faithfulness (DN-119 C5)**, then
**(5) reuse of built machinery (DRY)**. Option A dominates on (1)(2)(3)(5); C adds (4) at bounded (2)-cost.

## §7 Adversarial stress-test — where it breaks, and the open questions

**OQ-1 — Content-addressed identity vs. field ORDER and NAMES (the load-bearing one).** Under ADR-003 /
RFC-0001 §4.6 (and DN-120's verdict), a value's identity **is** its content hash; a record lowered to a
positional `Data` hashes by its **positional structure**. Two consequences the sugar must handle correctly,
never silently:
- **Names do NOT participate in identity.** Two records with identical field *types+values* in the same
  order but different field *names* hash **identically**. Correct for value semantics — but it means the
  named surface is *structural*, not *nominal-by-name*; a field **rename** is identity-preserving. (This is
  why Options A/C keep names as inert metadata, off the hash — the only sound choice.)
- **Order IS identity-load-bearing, and Rust literals are order-INSENSITIVE.** `Foo { b: 2, a: 1 }` equals
  `Foo { a: 1, b: 2 }` in Rust, but they lower to *different* positional layouts unless the desugarer
  **canonicalizes literal field order to the declaration order** before positional lowering. **This is a
  hard correctness obligation on the desugarer** (both transpiler and self-hosted): resolve each named field
  to its declaration index and emit in index order. A missing/duplicate name is a **never-silent refusal**,
  not a mis-ordered emit. *Open:* does struct-variant enum (C2) share the obligation per-variant (yes), and
  is declaration order part of a `pub type`'s exported contract (see OQ-2)?

**OQ-2 — Cross-phylum name metadata (DN-112 / DN-113 / M-1060).** `DataInfo` now carries a **home**
(nodule-qualified identity, DN-112 Rank 1); ctor field *types* were just re-homed (M-1060, PR #1506). A
foreign record crossing a phylum boundary exposes its **positional structure plus home identity**; its field
**names** are surface metadata local to the declaring phylum's sugar. *Open:* when an importer uses the named
sugar on a foreign type, is the foreign declaration's name↔index map **part of the cross-phylum export
surface** (a metadata import via DN-113's `use`-resolution), or **phylum-local** (the importer falls back to
positional / must import the metadata explicitly, never-silent)? Recommendation to ratify: **names stay
phylum-local metadata; identity plus the positional layout are the boundary contract** (keeps DN-112 identity
name-agnostic). But this needs DN-113 wiring to *reveal* a foreign record's names at an import site, and it
interacts with DN-122 (external-trait-impls across the home boundary). **FLAG: coordinate with M-1060/DN-113
before the self-hosted sugar claims cross-phylum name reveal.**

**OQ-3 — Functional-update aliasing vs. value semantics plus affine (M-919 / DN-120).** `Foo { x, ..base }`
desugars to destructure-and-reconstruct (`match base { Ctor(f0,…,fN) => Ctor(f0,…,NEW_x,…,fN) }`). The
`..base` spread **reads every non-overridden field** of `base`. *Open:* under the M-919 affine tracker, is
that spread a **consume/move** of `base` or a multi-read? DN-120's rc==1 reuse could reuse `base`'s
allocation for the result **iff no live alias** — the sugar must not silently alias, and must adopt DN-106
fork-A's policy (*never fabricate a mutation the desugarer was not taught*; an in-place mutation stays a
never-silent gap). This is the same affine question DN-106 Part 2 raised; the sugar inherits it and must pin
the spread's affine treatment (recommend: the spread is a structural read consumed by the reconstruct, so
`base` is moved unless the tracker proves rc==1-reuse safe).

**OQ-4 — Struct-pattern exhaustiveness plus the DN-104 ctor seal (the L3-G1 residual).** The struct-pattern
sugar `Foo { x, .. }` desugars to positional `Pattern::Ctor` with wildcards at omitted indices;
usefulness/exhaustiveness then runs on the positional form (already handled). *Open:* (a) the name→index
resolution for a partial pattern must be **total** — every named field resolves or a never-silent refusal;
(b) named-field **construction** sugar must respect the **DN-104 per-constructor visibility seal** (a
`priv`-sealed ctor withholds cross-nodule construction) — it lowers to the same `App`, so the seal holds, but
the name-metadata **reveal** must not leak a sealed ctor's field names cross-nodule beyond what the positional
surface already reveals; (c) a **duplicate field name within one ctor** is a never-silent registration
refusal. Minor but must be specified.

**OQ-5 (minor) — Phase-0 measure.** DN-121's own outstanding Phase-0 re-measure applies: because records
already emit positionally, the *`checked_fraction` delta* of this lever is **mostly faithfulness, not
expressibility** — the numeric leverage claim ("80/10%") should be re-measured as *how many files gain
readable round-trip* vs *how many gain check-cleanness* before any target is set. Do not claim a
`checked_fraction` number for this lever until re-measured (`Declared`, VR-5).

## §8 FLAGs (append-only reconciliations this note does not perform)

- **FLAG-1 (Doc-Index):** add the DN-123 row (this note appends it in the same change as an author
  convenience row; the integrating parent owns final reconciliation).
- **FLAG-2 (CHANGELOG):** the `[Unreleased]` `docs(dn)` entry for DN-123 (appended here, append-only).
- **FLAG-3 (DN-99 register):** the Struct rows should cross-reference DN-123 as the P2 design note and
  record that the residual is *faithfulness plus self-hosted surface*, not expressibility. **Integration-owned
  — not edited here.**
- **FLAG-4 (DN-119):** DN-119 §"deliberate exclusions" row for *named-field record type surface* should be
  annotated (append-only) that DN-106's ratification re-scoped the **sugar** into scope (this note). Not
  edited here (semcore/integration-owned).
- **FLAG-5 (issues.yaml):** M-876's records/named-field sub-scope should point `doc_refs` at DN-123.
  **Orchestrator/integration-owned — not edited here** (mitigation #1/#2, ID plus union-merge hygiene).

## §9 Definition of Done (the gate for this DN)

This DN is **ratifiable** (maintainer moves Draft → Accepted) when:
1. The **verified state correction** (§2 — records already emit positionally; the residual is faithfulness
   plus self-hosted surface) is confirmed against the tree and accepted as the framing.
2. **Option A** is accepted as the mechanism (sugar over positional plus name↔index map), with **Option C**
   as the YAGNI-gated faithfulness upgrade and **Option B** recorded rejected — grounded on DN-106 GP1/GP2
   plus KC-3.
3. The **§7 open questions** are acknowledged, with OQ-1 (canonicalize-to-declaration-order; names off the
   identity hash) and OQ-3 (spread affine treatment) accepted as **build preconditions**, and OQ-2
   (cross-phylum name metadata) accepted as a **coordination dependency on DN-113/M-1060**.
4. The **honest tag boundary** is accepted: the design is `Declared`; the name-faithful upgrade needs a
   **differential witness** (round-trip name preservation) before any `Empirical` claim; no `checked_fraction`
   number is claimed pre-Phase-0-re-measure (VR-5).

**Enacted** (a separate, later transition — never skipped to) requires: the transpiler `NamedFieldDrop`→
reveal-carrying-desugar upgrade landed plus differential-witnessed; the self-hosted `.myc` named-field sugar
(construction / `.field` / `Foo { x, y }` pattern, DN-119 L3-G1) landed with usefulness/exhaustiveness green
and the OQ-4 seal/duplicate refusals pinned as `*_reject` tests; and (if Option C is taken) the inert
`field_names` metadata retention landed without touching `Ty` identity.

## §10 User stories

- *As a **port author**, I want `struct Foo { x, y }`, `foo.x`, `Foo { x, y }` patterns, and `Foo { x,
  ..base }` to port mechanically, so that* the large class of Rust record code lands without hand-rewriting
  to positional constructors — **and** the ported `.myc` stays readable (names not silently lost).
- *As a **Mycelium developer** reading ported code, I want to `expand`/`EXPLAIN` a named-field sugar to see
  the positional grammar it compiles to, so that* the sugar hides nothing (house rule #2 / GP1) and I can
  reason about value identity structurally.
- *As a **maintainer**, I want the named-field surface to be sugar with zero kernel-semantic growth, so
  that* the small auditable kernel (KC-3) and value-semantic content-addressed identity (ADR-003) are
  preserved — a rename is identity-neutral and no named-field resolver enters the trusted base.
- *As a **cross-phylum consumer**, I want a foreign record's boundary contract to be its positional layout
  plus home identity (names phylum-local metadata), so that* importing it is never blocked on name agreement
  and identity stays name-agnostic (DN-112).

---

*DN-123 — Draft. Works the decision forward and recommends, ranked; ratification is the maintainer's
(house rule #3).*

## Changelog

- 2026-07-11 — DN-123 created (**Draft**): records/named-fields surface lever (P2). Verify-first finding
  (records already emit positionally; residual is faithfulness plus self-hosted surface), ranked
  recommendation (Option A sugar-over-positional then C YAGNI-gated then B rejected), adversarial OQ-1..5,
  DoD plus user stories. `Empirical` where read against dev tip `46006994`; `Declared` for the proposed design.
