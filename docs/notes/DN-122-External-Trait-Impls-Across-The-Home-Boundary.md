# Design Note DN-122 — External-Trait Impls Across the Home Boundary (the top `checked_fraction` unlock)

| Field | Value |
|---|---|
| **Note** | DN-122 |
| **Status** | **Draft** (2026-07-11). A design-reasoner DN working the **M-876** external-trait-impl gap forward to a ranked recommendation for maintainer ratification. It decides the **native mechanism for `impl Trait for Type` where the trait and/or the type is declared in another nodule/phylum** — the single biggest zero-hand-port unlock (DN-121 scoping: ~15% of the gap mass / 119 gaps; "emits-but-fails-check, poisons whole file"). **Recommends, does not ratify** (house rule #3 — the maintainer ratifies; status advances only by their decision). |
| **Decides (proposes, for ratification)** | (1) the **native mechanism** — a *foreign-trait import* that brings an external trait's signature into a phylum as a **home-qualified foreign declaration** (not a local re-declaration), against which `impl ForeignTrait for LocalType` checks and monomorphizes (§4, ranked); (2) the **coherence/orphan analogue** — extend the existing phylum-wide orphan rule + `CoherenceView` to the **import closure**, judged on **home-qualified identity** (§5, §7); (3) the **build split** — a checker-first change with a mono-dispatch fast-follow and a transpiler-rule swap, **zero L0/kernel/runtime** (§6); (4) the **semcore-serial vs disjoint** phasing (§8); (5) the **honest tag boundary** — the ~15% leverage is `Declared` until the Phase-0/Phase-4 re-measure (§9, DN-121's VR-5 boundary). |
| **Feeds** | M-876 (external-trait-impl / trait-Self-body surface — this note is its design gate); DN-121 (kernel-type-vocabulary scoping — P1 lever); DN-34 §8.8 (the transpiler `Widen`-class residue + the FAILED synthetic trait-def; delta-L3 ledger row 89); DN-99 register rows 27/28 (dyn-trait / APIT). |
| **Depends on (load-bearing, both Accepted-not-Enacted)** | **DN-112** (nodule-qualified type identity — the home model this note's coherence key rests on; impl M-1036, 0 % wired); **DN-113** (cross-phylum import resolution + **acyclic phylum DAG / cycle-refusal §9.3** — the soundness pillar and the import substrate; impl M-1060, 0 % wired). **This note is downstream of both — they must land before external-trait impls can be built or witnessed (§8, §10).** |
| **Grounds on** | RFC-0019 (Enacted — traits, dictionary-free static resolution, phylum-wide orphan rule + global uniqueness + reject-overlap, KC-3 node budget unchanged); DN-55 (static specialization — polymorphism = **zero kernel primitives**, monomorphizes away; the one ADR-033 dynamic-dispatch escape does not apply here); DN-112 (home-qualified identity); DN-113 (acyclic import closure); KC-3 (small kernel — check-time + mono only); G2 (never-silent — undefined/ambiguous/orphan = explicit `CheckError`); VR-5 (no tag upgraded past its basis); KISS/YAGNI (reuse the two ratified pillars, add no new coherence philosophy). |
| **Date** | July 11, 2026 |
| **Author** | design-reasoner (Opus). Owns only this note. |
| **Task** | M-876 — external-trait-impl surface (the P1 `checked_fraction` lever, DN-121). |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This is a **design recommendation**, not a
> decision (house rule #3). `Empirical` claims are read against the tree at `integration@b36f0ef4`
> and cite `file:line`; the recommended mechanism, the coherence-closure extension, and the leverage
> figure are `Declared` until implemented + differential-witnessed. **No sycophancy:** §4 ranks three
> real alternatives (plus the failed synthetic precedent) on merit; §9 argues *against* the Rank-1
> recommendation — the whole-closure-coherence-vs-separate-compilation tension is a genuine hard edge
> that could reasonably narrow v1 to a prelude-scoped subset, and it is surfaced, not glossed.

---

## §0 The question, in one line

**How does Mycelium natively express `impl Trait for Type` when the trait and/or the type is declared
in another nodule/phylum — soundly, under value semantics, home-qualified nominal identity (DN-112),
and static defunctionalization (DN-55) — so a file that uses one stops file-gating on `myc check`?**

## §1 The problem, verified against the code and the ledger

A file that contains one external-trait `impl` currently **fails `myc check` whole-file**
(delta-L3 ledger row 89, `docs/planning/zero-hand-port/delta-L3-transpiler.md:89`): *"Impl
(external-trait / whole-impl, `Widen`-class) — #2, 119 (15 %); emits-but-fails-check, poisons whole
file — DOWNSTREAM (LANGUAGE), no impl-of-external-trait / trait Self-body surface."* The canonical
witness is `mycelium-std-cmp`: 10 `impl Widen for …` blocks each fail with **`impl` for an undefined
external trait** (DN-34 §8 lines 297/336/351).

**What is already landed (do not re-decide — mitigation #14):**

- **`impl Trait for Type` surface + dictionary-free static resolution + coherence** are **Enacted**
  (RFC-0019; `crates/mycelium-l1/src/{checkty.rs,mono.rs,elab.rs}`). Same-nodule and *intra-phylum*
  cross-nodule impls check and monomorphize today.
- **The orphan rule is already phylum-wide** (M-662): an `impl` is legal iff its trait **or** its
  `for`-type head is declared in *some* nodule of the phylum, or the type is a primitive repr
  (`checkty.rs:3449`–`3451`; the pub-blind `CoherenceView`, `checkty.rs:1588`).
- **Polymorphism costs zero kernel primitives** — static specialization erases everything before L0
  (DN-55 §3; `mono.rs` "turns a checked generic-and-trait `Env` into a closed, monomorphic `Env` …
  No `mycelium-core` change").
- **Type identity is now home-qualified** (DN-112, Accepted): `Ty::Data` carries the declaring
  nodule's home; `type_head` (the coherence key) is qualified with it (`checkty.rs:296`).

**The genuine gap** is therefore *not* "traits" and *not* a new coherence philosophy. It is precisely
the **cross-home boundary**:

1. **No way to name a foreign trait.** The `CoherenceView` is built from the nodules *physically in
   the phylum* (`checkty.rs:1588`–`1600`); it contains no imported/dependency-phylum trait. So `Widen`
   — declared in another phylum — is an **undefined trait** at the impl site. There is no
   `use widen_phylum.Widen` that registers a *foreign* trait signature to `impl` against.
2. **Trait Self-bodies are unsupported** (M-876; ledger row 73). A foreign trait's signatures use
   `Self`; the checker does not yet resolve `Self` in trait method signatures — the direct cause of
   the failed synthetic attempt (§2).
3. **Cross-phylum instance dispatch is unwired.** Mono resolves a trait-method call only via a
   *resolved instance* in the merged `Env` (`mono.rs:212`, `:891`). Instances imported across a
   phylum boundary are not in that `Env` today (DN-113 is 0 % wired) — an *independently* open
   dispatch gap that this note's Phase 2 must close, not assume closed.

## §2 Why the synthetic trait-def attempt FAILED (recorded — do not repeat)

The transpiler tried to **fabricate a local `trait Widen { … }`** in each file that impls it, so the
`impl` would resolve. It **failed** (DN-34 §8.8, `DN-34…:351`): *"a synthetic trait-def was tried and
**fails** (`unknown type Self` / arg-type mismatch) — a real trait-surface gap, not cheaply
closeable,"* with a fabricated `from(self)` body (`DN-34…:183`). Three distinct reasons, all
structural — this is the anti-pattern the recommended mechanism must avoid:

1. **`unknown type Self`** — the fabricated trait body references `Self`, and trait Self-bodies are
   unsupported (M-876). A synthetic def cannot dodge the missing Self surface; it *depends* on it.
2. **Arg-type mismatch** — the fabricated signature is a guess; it does not necessarily match the real
   `Widen`'s method types, so even when it parses, the `impl` fails to satisfy it.
3. **Identity fork (the deep reason).** A *local* `trait Widen` re-declared per file is, under
   home-qualified identity (DN-112) + content-addressing (ADR-003), a **different trait** in every
   file — `a::Widen ≠ b::Widen`. Coherence, EXPLAIN, and mono mangling would all see N distinct
   traits, not one. Re-declaration is not import; it forks identity. **The mechanism must reference
   the one true home-qualified trait, never re-mint it.**

## §3 KC-3 framing + blast radius (verified)

**This is a check-time (L1) + monomorphization (L1) change — no L0 node, no kernel prim, no runtime /
representation change (DN-55 §3; RFC-0019 KC-3-unchanged).** External-trait impls are pure static
specialization: once the impl resolves at check time, mono rewrites each call to a **direct call** to
the concrete instance body (dictionary-free — DN-55 §2.3). No `FieldSpec` change; the ADR-033
dynamic-dispatch escape (heterogeneous collections) is **not** in play here — every external-trait
impl call site is at a statically known concrete type.

Blast radius concentrates in the **semcore serial lane** (`crates/mycelium-l1`, read-only for this
note): the `CoherenceView`/orphan pass and the foreign-trait registry in `checkty.rs`, and the
instance-resolution path in `mono.rs`. The grammar surface (a foreign-trait `use`) and the transpiler
emission rule are **disjoint** from semcore (§8).

## §4 The native mechanism — three alternatives, ranked

### Alt 1 — Foreign-trait import + closure-extended coherence (RECOMMENDED, Rank 1)

**Reuse DN-113's cross-phylum `use` to import a trait's *signature* as a home-qualified FOREIGN trait
declaration**, and **extend the phylum-wide `CoherenceView` + orphan rule to the import closure**.

- **Mechanism.** `use widen_phylum.Widen` resolves (through DN-113's manifest DAG + hash-checked link)
  to the **one** `TraitInfo` at its declaring home. It is registered into the checking phylum's
  coherence view as a **foreign** trait whose identity is `widen_phylum::Widen` (home-qualified,
  DN-112-style) — **not** a local declaration and **not** re-minted. `impl Widen for LocalType` then
  checks against that imported signature; the orphan rule admits it because **the `for`-type head is
  home-local** (home-qualified). Dispatch is unchanged static specialization (DN-55): mono resolves
  `(widen_phylum::Widen, thisphylum::LocalType)` to the concrete instance body and emits a direct
  call.
- **Coherence key.** `type_head` already yields a home-qualified key for the type (DN-112 §5); this
  note qualifies the **trait** side symmetrically, so the coherence pair is
  `(widen_phylum::Widen, thisphylum::LocalType)` — globally unique across the closure by §5's argument.
- **Self-bodies.** A small checker addition: resolve `Self` in the imported trait's method signatures
  to the impl's `for`-type at check time (closes the M-876 sub-gap that sank the synthetic attempt).
- **What changes.** (a) `CoherenceView` construction — union in the import closure's foreign traits
  (and their impls, for global-uniqueness, §5); (b) `resolve_ty`/trait-name resolution — resolve a
  `use`d foreign trait to its home-qualified `TraitInfo`; (c) orphan rule — judge trait-local /
  type-local on home-qualified identity; (d) `Self` resolution in trait sigs; (e) mono — cross-phylum
  instance lookup (§6 Phase 2); (f) transpiler — emit the `use` instead of the synthetic def (§8).
- **KC-3 verdict.** Smallest mechanism that closes the gap **and reuses the two ratified pillars**
  (DN-112 home identity, DN-113 acyclic import) rather than inventing a new coherence philosophy. It
  is the value-semantics-native reading of Rust's "impl a foreign trait for a local type." **Rank 1.**

### Alt 2 — Explicit `foreign trait` / `extern trait` re-statement (hash-checked) (Rank 2)

Let a phylum **re-state** the external trait's signature under an explicit `foreign trait Widen { … }`
form that **must hash-match** the real declaration (DN-113 already refuses an import on hash mismatch).

- **Pro.** No dependency on DN-113's *name-resolution* surface for traits specifically — the signature
  is written locally; identity is pinned by the hash check, so it does **not** fork (unlike the failed
  synthetic def, which had no hash gate).
- **Con.** Author re-writes every external trait's signature (DX cost + drift risk), and it still
  needs Self-body support and the hash-link. It is the *disciplined* cousin of the failed synthetic
  attempt — safe only because of the hash gate. More surface, more churn, worse ergonomics than an
  import. **Rank 2** — the honest fallback if DN-113's trait-import surface proves out of scope for v1
  (OQ-2).

### Alt 3 — Structural / duck typing for external impls (Rank 3, REJECTED)

Drop nominal trait identity for external impls: accept any impl whose method *shapes* match.

- **Con.** Directly violates nominal home-qualified identity (DN-112), content-addressing (ADR-003 —
  the RFC-0019 §2.2 argument: same hash must not map to two semantics), and **G2** (structural match
  admits silent ambiguity — two shape-compatible traits become indistinguishable). It abandons
  coherence, the one property RFC-0019 calls non-negotiable. **Rejected — unsound for this substrate.**

### Rejected precedent — synthetic local trait-def (the FAILED attempt, §2)

Per-file fabricated `trait Widen`: fails on `unknown Self`, arg-type mismatch, and identity fork.
Documented so it is not retried; Alt 2 is its *only* safe form (hash-checked, not fabricated).

### Objective function (criteria table)

| Criterion | Alt 1 — foreign import | Alt 2 — hash-checked re-statement | Alt 3 — structural |
|---|---|---|---|
| Closes the file-gating gap (must-have) | Yes | Yes | Yes |
| Preserves nominal identity (DN-112 / ADR-003) | **Yes** (references the one home trait) | Yes (hash-pinned) | **No** — forks/duck |
| Coherence soundness (§5) | **Yes** (closure + acyclic DAG) | Yes | **No** (silent ambiguity) |
| Reuses ratified pillars (KISS/YAGNI) | **DN-112 + DN-113** | DN-112 + hash-link | none — new philosophy |
| KC-3 / zero kernel prim (DN-55) | **Yes** | Yes | Yes |
| Author ergonomics (DX) | **Best** (`use` the trait) | Weak (re-write sigs) | n/a (unsound) |
| Blast radius (semcore serial) | Checker + mono, bounded | Checker + mono + surface | Broad, guts coherence |
| Honest tag reachable | `Empirical` post-witness | `Empirical` post-witness | never (unsound) |

**Ranked recommendation: Alt 1, then Alt 2, then (reject) Alt 3.** Alt 1 is the value-semantics-native
mechanism, reuses both ratified pillars, and closes the gap with the smallest new coherence philosophy
(none — it *extends* the existing one to the closure). Alt 2 is the honest fallback if DN-113 trait
import is deferred. Alt 3 is unsound.

## §5 The coherence / orphan-rule soundness story (the classic hard part)

**Claim (Declared-with-argument — VR-5, not machine-checked, matching RFC-0019's own coherence tag):
the orphan-rule analogue extended to the import closure is sound — instance resolution for any
`(Trait, Type)` pair over a well-formed phylum closure is total, deterministic, and hash-stable —
*conditioned on two properties that must be relied upon explicitly and tested*.**

**The orphan rule, restated on home-qualified identity.** `impl T for Ty` is legal iff `T` is
home-local **or** `Ty`'s head is home-local (or `Ty` is a primitive repr) — judged on DN-112
home-qualified identity. So:

- `impl ForeignTrait for LocalType` — legal (type home-local). *This is the `Widen`-class case.*
- `impl LocalTrait for ForeignType` — legal (trait home-local).
- `impl ForeignTrait for ForeignType` — **orphan → explicit `CheckError`** (neither home-local),
  matching Rust. G2-refused, never silent.

**Global uniqueness across the closure.** For `(ForeignTrait, LocalType)`: the type is home-local, so
**no other phylum may impl it** (any other phylum sees `LocalType` as foreign → its impl would be an
orphan → refused). Home-qualified identity (DN-112) makes `(Widen, a::T)` and `(Widen, b::T)` **distinct
pairs**, so unrelated same-named types never false-conflict. This is where DN-112 is load-bearing.

**The genuine hazard — could two phyla both legally impl the *same* pair?** Consider
`impl Widen for a::MyType` written both by phylum `a` (owns `MyType`) **and** by `Widen`'s home phylum
(owns `Widen`). Both satisfy the orphan rule. That is the classic non-confluence hole. **It cannot
occur, because of DN-113's acyclic phylum DAG (§9.3 — cycles are refused never-silently with the cycle
path).** For `Widen`'s home to name `a::MyType` it must depend on `a`; for `a` to name `Widen` it must
depend on `Widen`'s home — a **cycle**, refused by DN-113. So at most one of {trait-home, type-home}
can name both, so at most one legal impl site, so global uniqueness holds. **Acyclicity is the
soundness pillar** — exactly Rust's acyclic-crate-graph guarantee, here provided by DN-113 §9.3.

**The two conditions the implementation MUST satisfy (else unsound — enumerated as DoD witnesses,
§10):**

1. **Coherence is checked over the *whole import closure*, not one phylum.** The current
   `CoherenceView` is intra-phylum (`checkty.rs:1588`). External-trait impls force it to span the
   transitive closure — otherwise a diamond dependency where two intermediate phyla each ship an
   impl for the same `(ForeignTrait, ForeignType)` pair (each individually illegal-orphan, but a
   partial check might see only one) slips through. **v1 must check coherence over the full closure**
   — this is the tension with DN-113's *deferred* separate compilation (§9).
2. **Acyclic phylum DAG is a hard precondition.** If DN-113's acyclicity were ever relaxed (a future
   separate-compilation/linking story), the non-confluence hole reopens. **FLAG forward:** external-
   trait coherence is only sound while the phylum graph is a DAG.

**Overlapping impls stay rejected** (RFC-0019 §4.5 — no `OverlappingInstances`); nothing here relaxes
that. Newtype-derived waivers remain rejected (RFC-0019 Q-coherence — needs roles).

## §6 Checker-only, or surface/mono/runtime? The build split

| Layer | Needed? | What |
|---|---|---|
| **L0 kernel / runtime** | **No** | Static specialization erases before L0 (DN-55 §3); zero kernel prim, no `FieldSpec` change, value semantics unaffected. |
| **Surface / grammar** | **Small** | A foreign-trait `use` (reuse DN-113 `use phylum.Trait` if in scope; else add — OQ-2) plus `Self` in trait method signatures (M-876 sub-gap). |
| **Checker (L1, semcore)** | **Yes (core)** | Foreign-trait registration into a closure-extended `CoherenceView`; orphan rule on home-qualified identity; `Self` resolution; symmetric home-qualification of the trait side of `type_head`. |
| **Mono (L1, semcore)** | **Yes (fast-follow)** | Cross-phylum **instance resolution** so dispatch finds the imported instance (the independently-open gap, §1.3). Verify claim (`Declared`): I did not reproduce cross-phylum dispatch failure directly — flagged for Phase-0 empirical confirmation. |
| **Transpiler** | **Yes (rule swap)** | Replace the failed synthetic trait-def rule with a foreign-trait-import emission; re-run the ladder; re-measure `checked_fraction`. |

**Verdict: checker-first, mono fast-follow, transpiler rule-swap — no runtime/kernel piece.**

## §7 Composition with DN-112 (home identity) and DN-55 (mono)

- **DN-112.** The coherence pair and mono mangling are already home-qualified for the *type*; this
  note qualifies the *trait* symmetrically. `mangle_ty`'s separator-safety edit (DN-112 §7) covers the
  trait home too. No new identity philosophy — the same home-qualified string carries the trait's home.
- **DN-55.** Nothing dynamic is added. An external-trait impl is ordinary static specialization: one
  concrete instance body, resolved statically, direct-called, EXPLAIN-reified in `MonoSelections`
  (`mono.rs:63`–`130`). The ADR-033 escape is untouched.

## §8 Phased build plan + semcore-vs-disjoint split

**Ordering is gated by DN-112 (M-1036) and DN-113 (M-1060) landing first — both are Accepted, 0 %
wired. External-trait impls cannot be built or witnessed before their substrate exists.**

- **Phase 0 — re-measure (VR-5 boundary, DN-121).** Establish the *current* `checked_fraction` on the
  `Widen`-class corpus (`mycelium-std-cmp` plus the 119-gap set) and **empirically confirm** the
  cross-phylum-dispatch claim (§1.3). Converts the ~15 % leverage from `Declared` to a measured
  baseline. **Disjoint** (transpile-vet harness).
- **Phase 1 — checker (SEMCORE SERIAL).** Foreign-trait registration plus closure-extended
  `CoherenceView` plus orphan rule on home-qualified identity plus `Self`-in-trait-sig resolution
  (`checkty.rs`). Serial lane — one owner, no parallel semcore edits.
- **Phase 2 — mono (SEMCORE SERIAL).** Cross-phylum instance resolution so dispatch finds imported
  instances (`mono.rs`). Serial, immediately after Phase 1.
- **Phase 3 — surface (DISJOINT).** Confirm/extend DN-113 trait `use` surface plus grammar; the
  Self-body grammar line. Parallelizable with Phase 4 authoring.
- **Phase 4 — transpiler (DISJOINT) plus RE-MEASURE.** Emit the foreign-trait import in place of the
  synthetic def; re-run the M-1006 ladder; **re-measure `checked_fraction`**, which upgrades the
  leverage tag from `Declared` toward `Empirical` (§9).

**Semcore vs disjoint:** Phases 1–2 are the **serial semcore lane** (`mycelium-l1`,
`crates/mycelium-l1/` — read-only for this note); Phases 0/3/4 are **disjoint** and can proceed in
parallel around the serial core.

## §9 Adversarial stress-test (VR-5 / house rule #4 — argue against the recommendation)

**Sharpest disconfirming finding — whole-closure coherence vs. separate compilation.** §5's soundness
rests on checking coherence over the **entire import closure**. But DN-113 **defers separate
compilation** (its §12 "Deferred, never-silently: separate compilation (B3)"). These are in tension:
if a phylum is ever `myc check`ed against a *partial* closure (only the phyla physically present, not
the full transitive set), a diamond-dependency orphan conflict can escape the check. **The orphan rule
plus acyclicity guarantee at most one *legal* author, but a partial-closure check cannot *witness* the
uniqueness it relies on.** So v1 external-trait coherence is honest **only** if it mandates
whole-closure checking (accepting: no separately-compiled coherence in v1). This is a real constraint
the maintainer must ratify explicitly, not a free consequence.

**The strongest argument for narrowing v1 (a legitimate Rank-1 flip).** Alt 1 depends on **two**
unbuilt pillars (DN-112 *and* DN-113, both 0 % wired) plus a coherence-closure extension. That is a lot
of substrate before the first `Widen` checks. A maintainer could reasonably prefer a **prelude-scoped
subset first**: the dominant `Widen`-class traits are *stdlib/prelude* conversions with a **single,
known home** (`@prelude`). Registering prelude traits as foreign-impl targets needs **only** prelude-
trait registration plus Self-bodies — **no full cross-phylum import, no closure coherence** (the
prelude is one uniform home, already exempt per DN-112 §9). This subset may capture much of the 15 %
*before* DN-113 lands, and is strictly a special case of Alt 1 (same mechanism, closure = {this phylum,
prelude}). **Recommendation stands (Alt 1 full), but the prelude-scoped subset is the honest v1 MVP if
DN-113 slips** — surfaced as OQ-1, not buried.

**Where the mechanism otherwise holds (reasoned against the code):**

1. **Re-exports / `use` chains.** The foreign trait's home must be the **declaring** phylum, resolved
   through DN-113's hash-checked link — never the use-site. A mis-resolution that stamps the use-site
   re-forks identity (the §2.3 failure). Load-bearing invariant, DoD-tested.
2. **Blanket/generic external impls** (`impl Widen for Binary{N}` at many widths). Each width is a
   distinct monomorphic instance (DN-55 §2.2); the coherence key is per (trait-home, type-head), width
   erased — matching RFC-0019's M-659 "keys per (trait, type-head)." No new hazard; DN-42 Q5
   (width-generic *instances*) remains its own open question (DN-55 §10.4) and is **out of scope** here.
3. **Prelude/builtin uniform home.** Prelude traits stay under one reserved home (DN-112 §9 invariant)
   — the exact exemption the subset in the paragraph above leans on.

**Net verdict:** the recommendation **survives**, conditioned on (i) **whole-closure coherence
checking** (accept: no separate-compilation coherence in v1), (ii) the **acyclic phylum DAG** (DN-113
§9.3) as a hard precondition, and (iii) **home = declaring phylum, resolved through the hash-checked
link, never the use-site**. All three are enumerated as DoD witnesses (§10), not left to care. The
prelude-scoped subset is the honest fallback ordering if DN-113 is not yet Enacted.

## §10 Definition of Done (what "Accepted" requires of the maintainer)

Ratifying this note = accepting the **mechanism** (Alt 1, or the prelude-scoped subset as v1 MVP per
OQ-1); the implementation (M-876) must then satisfy:

1. **Foreign-trait reference.** `use phylum.Trait` (or Alt 2's hash-checked re-statement) registers
   the **one home-qualified** `TraitInfo` — never a re-minted local trait (the §2.3 fork is a tested
   negative).
2. **`impl ForeignTrait for LocalType` checks** against the imported signature; `Self` resolves to the
   `for`-type (the M-876 sub-gap that sank the synthetic attempt is closed — a positive witness).
3. **Orphan rule on home-qualified identity.** `impl ForeignTrait for ForeignType` is a never-silent
   `CheckError` (orphan); `impl ForeignTrait for LocalType` and `impl LocalTrait for ForeignType` are
   admitted — a dedicated cross-home orphan witness.
4. **Closure coherence (§5 / §9 condition i).** A **diamond-dependency** fixture: two phyla each
   attempting the same `(ForeignTrait, ForeignType)` pair is refused; two distinct home-qualified
   pairs (`(Widen, a::T)`, `(Widen, b::T)`) both check without false overlap. Coherence is verified
   over the **full closure**.
5. **Acyclicity precondition (§9 condition ii).** A regression pinning that external-trait coherence
   assumes DN-113 cycle-refusal (§9.3); a cyclic fixture refuses before any coherence claim is made.
6. **Home = declaring phylum (§9 condition iii).** A re-export/`use`-chain fixture: a foreign trait
   used through an intermediate phylum keeps its **declaring** home; a use-site stamp is caught.
7. **Mono cross-phylum dispatch.** A differential: an external-trait method call resolves to the
   imported instance's body (direct call, `MonoSelections`-reified) and agrees across L1-eval /
   L0-interp / AOT.
8. **Re-measure (§8 Phase 4).** `checked_fraction` re-measured on the `Widen`-class corpus **before
   and after** — the leverage claim moves from `Declared` to `Empirical` with the delta shown (VR-5).
9. **`.myc` parity** noted (DN-26): the self-hosted frontend mirrors foreign-trait resolution as the
   checkty cross-phylum port progresses.
10. **Guarantee tag = `Empirical`** for the landed mechanism (witnessed by items 4/7/8); `Declared`
    for any prelude-scoped-subset-only v1 that does not yet witness the full closure (OQ-1). No tag
    upgraded past its basis (VR-5). No `Proven` — no unforgeability/confluence theorem is discharged.

## §11 Open questions (FLAGGED — never guessed, G2/VR-5)

- **OQ-1 (v1 scope, for the maintainer).** Full Alt 1 (cross-phylum, gated on DN-113 Enacted) vs the
  **prelude-scoped subset MVP** (closure = {this phylum, `@prelude`}, buildable before DN-113)?
  §9's dependency-substrate argument is a genuine reason to ship the subset first. This note
  recommends Alt 1 as the target with the subset as the honest first increment — a real fork, stated
  on merits.
- **OQ-2 (surface).** Is a *trait* `use` in DN-113's cross-phylum import scope, or must it be added
  (or fall to Alt 2's `foreign trait` hash-checked form)? Confirm against DN-113's resolved surface
  before Phase 3.
- **OQ-3 (whole-closure vs separate compilation).** §9 condition (i): v1 mandates whole-closure
  coherence checking. Is that acceptable to the maintainer, or is a coherence-registry/linking story
  (deferred with DN-113 B3) required first? Load-bearing for soundness.
- **OQ-4 (associated types / multi-param).** External traits with associated types or multiple type
  parameters are **out of scope** (RFC-0019 defers both to v2); the `Widen`-class is single-parameter.
  Confirm the corpus's external traits are all single-parameter, or scope the residual.
- **OQ-5 (Self-body surface, M-876).** The `Self`-in-trait-signature addition is shared with M-876's
  general trait-Self-body work — confirm one owner so it is not built twice.

## §12 Grounding

- **`docs/rfcs/RFC-0019-Traits-and-Parametric-Polymorphism.md`** (read 2026-07-11, full) — Enacted;
  `impl … for …`, dictionary-free static resolution, phylum-wide orphan rule plus global uniqueness
  plus reject-overlap (§4.5), coherence = the ADR-003 content-addressing consequence (§2.2), KC-3 node
  budget unchanged, Q-coherence (reject newtype waivers — needs roles), multi-param/assoc-types
  deferred to v2. Grounds §1, §5, §11.
- **`docs/notes/DN-55-Static-Specialization-The-Polymorphism-Model.md`** (read 2026-07-11, full) —
  static specialization = zero kernel primitives; dictionary-free static resolution; the one ADR-033
  dynamic-dispatch escape (not in play here); `MonoSelections` EXPLAIN record. Grounds §3, §6, §7.
- **`docs/notes/DN-112-Nodule-Qualified-Type-Identity.md`** (read 2026-07-11, full) — home-qualified
  `Ty::Data`; `type_head` coherence key qualified; prelude uniform-home invariant (§9);
  `mangle_ty` separator-safety; Accepted-not-Enacted (M-1036). Grounds §4, §5, §7, §9.
- **`docs/notes/DN-113-Cross-Phylum-Import-Resolution-Subsystem.md`** (read 2026-07-11, §0/§9.3
  header) — cross-phylum import DAG; **§9.3 cyclic-phyla refusal** (the acyclicity pillar); hash-
  checked link; separate compilation deferred (B3); Accepted-not-Enacted (M-1060, 0 % wired). Grounds
  §4, §5, §8, §9.
- **`docs/planning/zero-hand-port/delta-L3-transpiler.md`** (read 2026-07-11, §2–§5) — ledger row 89
  (external-trait Impl, 119/15 %, poisons whole file, synthetic def FAILED); row 72–73 (emits-but-
  fails-check, trait Self-bodies unsupported); the downstream-language vs transpiler split. Grounds
  §1, §2, §8.
- **`docs/notes/DN-34-Rust-to-Mycelium-Transpiler-Strategy.md`** (read 2026-07-11, grep §8) — the
  `Widen`-class witness (`mycelium-std-cmp`, lines 297/336/351); the synthetic trait-def failure
  (`unknown Self` / arg mismatch, line 351; fabricated `from(self)`, line 183). Grounds §2.
- **`crates/mycelium-l1/src/checkty.rs`** (read 2026-07-11: `:296` `type_head`, `:1588`–`1625`
  `CoherenceView`/`register_nodule_decls`, `:3449`–`3451` orphan rule) — the intra-phylum coherence
  view external-trait impls must extend to the closure. Grounds §1, §3, §5.
- **`crates/mycelium-l1/src/mono.rs`** (read 2026-07-11: `:63`–`130` `MonoSelections`, `:212`, `:891`
  instance-required dispatch) — static resolution requires a resolved instance in the merged `Env`;
  cross-phylum instances unwired. Grounds §1.3, §6.
- **House rules:** KC-3, G2, VR-5, KISS/YAGNI, house rules #2/#3/#4.

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-07-11 | **Draft** | DN-122 created. A design-reasoner DN working M-876 (external-trait impls — the P1 `checked_fraction` lever, DN-121 ~15 %/119 gaps) forward to a ranked recommendation. Recommends **Alt 1 — foreign-trait import plus closure-extended coherence on home-qualified identity** (Rank 1; Alt 2 hash-checked re-statement second, Alt 3 structural rejected; the synthetic local trait-def is the recorded FAILED precedent, §2). Coherence soundness = phylum-wide orphan rule extended to the import closure, sound **conditioned on** DN-113's acyclic phylum DAG (§9.3) plus DN-112 home identity (§5). Checker-first plus mono fast-follow plus transpiler rule-swap; **zero L0/kernel/runtime** (DN-55). Downstream of DN-112 (M-1036) and DN-113 (M-1060) — both must land first (§8). Sharpest stress-test: whole-closure coherence vs. deferred separate compilation, and the prelude-scoped subset as the honest v1 MVP if DN-113 slips (§9, OQ-1). Leverage figure `Declared` until the Phase-0/Phase-4 re-measure (§9/§10). Grounded against `integration@b36f0ef4`. **Recommends, does not ratify** (house rule #3). Authored the DN only — no edit to `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (integration-owned; FLAGGED up). |
