# Design Note DN-122 — External-Trait Impls Across the Home Boundary (the top `checked_fraction` unlock)

| Field | Value |
|---|---|
| **Note** | DN-122 |
| **Status** | **Draft** (2026-07-11). A design-reasoner DN working the **M-876** external-trait-impl gap forward to a ranked recommendation for maintainer ratification. It decides the **native mechanism for `impl Trait for Type` where the trait and/or the type is declared in another nodule/phylum** — the single biggest zero-hand-port unlock (DN-121 scoping: ~15% of the gap mass / 119 gaps; "emits-but-fails-check, poisons whole file"). **Recommends, does not ratify** (house rule #3 — the maintainer ratifies; status advances only by their decision). |
| **Decides (proposes, for ratification)** | (1) the **native mechanism** — a *foreign-trait import* that brings an external trait's signature into a phylum as a **home-qualified foreign declaration** (not a local re-declaration), against which `impl ForeignTrait for LocalType` checks and monomorphizes (§4, ranked); (2) the **coherence/orphan analogue** — extend the existing phylum-wide orphan rule + `CoherenceView` to the **import closure**, judged on **home-qualified identity** (§5, §7); (3) the **build split** — a checker-first change with a mono-dispatch fast-follow and a transpiler-rule swap, **zero L0/kernel/runtime** (§6); (4) the **semcore-serial vs disjoint** phasing (§8); (5) the **honest tag boundary** — the ~15% leverage is `Declared` until the Phase-0/Phase-4 re-measure (§9, DN-121's VR-5 boundary). |
| **Feeds** | M-876 (external-trait-impl / trait-Self-body surface — this note is its design gate); DN-121 (kernel-type-vocabulary scoping — P1 lever); DN-34 §8.8 (the transpiler `Widen`-class residue + the FAILED synthetic trait-def; delta-L3 ledger row 89); DN-99 register rows 27/28 (dyn-trait / APIT). |
| **Depends on (load-bearing — BOTH NOW LANDED, re-verified 2026-07-12 `Empirical @dev b36ebdbe`)** | **DN-112** (nodule-qualified type identity — the home model this note's coherence key rests on; impl **M-1036 `status: done`, landed 2026-07-11** — `type_head`/`qualify_type_name` home-qualified, `checkty.rs:296`); **DN-113** (cross-phylum import resolution + **acyclic phylum DAG / cycle-refusal §9.3** — the soundness pillar and the import substrate; impl **M-1060 `status: done`, landed 2026-07-11**, incl. a 3-cycle adversarial-verification HOLE A/A2/B closure, `checkty.rs:1562`/`4307`/`7759`). **The substrate this note planned as its checker "Phase 1" now EXISTS — so the ground moved since Draft-1 (mitigation #14): the common `impl ForeignTrait for LocalType` already checks clean today, and the residual narrowed to a single, already-tracked follow-up (M-1076) plus a transpiler rule-swap. See §13.** |
| **Grounds on** | RFC-0019 (Enacted — traits, dictionary-free static resolution, phylum-wide orphan rule + global uniqueness + reject-overlap, KC-3 node budget unchanged); DN-55 (static specialization — polymorphism = **zero kernel primitives**, monomorphizes away; the one ADR-033 dynamic-dispatch escape does not apply here); DN-112 (home-qualified identity); DN-113 (acyclic import closure); KC-3 (small kernel — check-time + mono only); G2 (never-silent — undefined/ambiguous/orphan = explicit `CheckError`); VR-5 (no tag upgraded past its basis); KISS/YAGNI (reuse the two ratified pillars, add no new coherence philosophy). |
| **Date** | July 11, 2026 |
| **Author** | design-reasoner (Opus). Owns only this note. |
| **Task** | M-876 — external-trait-impl surface (the P1 `checked_fraction` lever, DN-121). |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This is a **design recommendation**, not a
> decision (house rule #3). `Empirical` claims are re-read against the tree at **`dev@b36ebdbe`**
> (2026-07-12 re-verification — §13) and cite `file:line`; the still-unbuilt pieces (the transpiler
> rule-swap, the M-1076 sig-re-homing, the `Self`-in-trait-sig grammar) are `Declared` until
> implemented + differential-witnessed. The fresh Phase-0 re-measure (2026-07-12, `Empirical`) ranks
> the Impl/external-trait gap class at **114 gaps / 12.4 %** (was 119 / ~15 % at Draft-1). **No
> sycophancy:** §4 ranks three real alternatives (plus the failed synthetic precedent) on merit;
> §9/§13 argue *against* the original Draft-1 build framing — since **M-1060 already landed the
> checker substrate**, the honest v1 is a **transpiler-rule-swap + prelude-scoped subset**, far
> smaller than Draft-1's "checker Phase 1", and the whole-closure-coherence-vs-separate-compilation
> tension is narrower than Draft-1 feared. §13 is the build-ready spec that supersedes §8's stale plan.

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

**What is already landed (do not re-decide — mitigation #14; re-verified `Empirical @dev b36ebdbe`
2026-07-12). NOTE: the M-1060 landing closed most of Draft-1's planned "Phase 1" — see §13 for the
narrowed residual.**

- **`impl Trait for Type` surface + dictionary-free static resolution + coherence** are **Enacted**
  (RFC-0019; `crates/mycelium-l1/src/{checkty.rs,mono.rs,elab.rs}`). Same-nodule and *intra-phylum*
  cross-nodule impls check and monomorphize today.
- **The orphan rule is already phylum-wide, pub-blind, on home-qualified identity** (M-662 + M-1036):
  `register_instances` admits `impl T for Ty` iff `T` is declared in *some* nodule of the phylum **or**
  `Ty`'s head is (or `Ty` is a primitive repr) — `checkty.rs:4480`–`4511`; the pub-blind
  `CoherenceView`, `checkty.rs:2231`–`2237`. `type_local` compares `ty_local_name(n)` so the qualified
  identity does not defeat the locality test (`checkty.rs:4485`).
- **Cross-phylum trait *import* is WIRED (M-1060, done 2026-07-11).** `use dep.Trait` resolves through
  the DN-113 manifest DAG + hash-checked link and merges the **one** foreign `TraitInfo` into the
  consumer's `Exports` under a `"{dep}::…"` phylum-qualifier key (`::` is not a legal Mycelium ident
  char, so it can never collide with an intra-phylum `.`-joined name; `checkty.rs:1606`–`1633`). It is
  **not re-minted** — it references the declaring home. The current binding's cross-phylum origin is
  tracked in `NoduleImports::cross_phylum_traits` (`checkty.rs:1584`), re-homed in lockstep at every
  `insert_export` so an intra-phylum shadow clears the marker.
- **The foreign-trait-sig *soundness guard* is landed** (M-1060 fix-cycle-3, HOLE A/A2/B). A foreign
  trait whose signature names a **concrete type beyond its own generic params** is refused
  never-silently at all three sites — register-time (`register_instances`, `checkty.rs:4519`),
  method-call (`check_trait_method_call`, `checkty.rs:7774`), and bound-discharge — via the shared
  `foreign_trait_sig_names_a_concrete_type` recognizer (`checkty.rs:4339`), because re-resolving that
  bare name against the *consumer's* registry would reopen "the M-1036/DN-112 bare-name collapse one
  level up."
- **Polymorphism costs zero kernel primitives** — static specialization erases everything before L0
  (DN-55 §3; `mono.rs` "turns a checked generic-and-trait `Env` into a closed, monomorphic `Env`").

**The genuine RESIDUAL gap** (post-M-1060) is therefore *not* "traits", *not* "no way to name a
foreign trait" (that landed), and *not* a new coherence philosophy. It is three narrow, disjoint items:

1. **The transpiler still emits the FAILED synthetic def / leaves the trait undefined.** `myc check`
   on the emitted `impl Widen[...] for Binary{...}` fails **`impl` for unknown trait `Widen`**
   (`crates/mycelium-transpile/src/tests/vet.rs:69`) — because the emission carries **no `use`** to
   import the trait. This is the single change that moves `checked_fraction`, and it is **disjoint
   from semcore** (§13.1). *(This is what makes the 12.4 % still "open" even though the checker admits
   the pattern.)*
2. **`Self`-in-trait-signature is a grammar gap (M-876 sub-gap).** Mycelium has **no implicit `self`**
   (`ast.rs:227`) and the single-param-trait convention spells the self-type as the trait's own param
   (`Cmp[A] { fn cmp(a: A, b: A) => Binary{2}; }`; lexicon `T: Cmp ≡ T: Cmp<T>`). A **two-type** trait
   like `Widen[To]` (From→To) needs to name the *implementing* type in `fn widen(this: Self) => To`,
   and the grammar has **no slot** for it — `emit_trait` bails honestly (`emit.rs:2489`). So the
   `Widen`-two-type sub-class is **not** free from M-1060; single-param traits (`Cmp`/`Eq`/`Fuse`-shape)
   **are** (§13.2).
3. **The concrete-type-in-foreign-sig re-homing is deferred to M-1076.** A foreign trait/fn signature
   that names a concrete type beyond its params is *correctly refused* today (soundness held), but not
   yet *admitted* by re-homing the sig against its declaring phylum — the DN-113 §7 / DN-122 general
   fix, **already minted as M-1076** (`status: todo`, `depends_on: [M-1060]`). Runtime cross-phylum
   **dispatch** (mono/eval executing a boundary-crossing call) is separately deferred (DN-113 §8) — but
   it does **not** affect `checked_fraction`, which measures `myc check` only.

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

> **SUPERSEDED by §13 (2026-07-12).** This section was written at Draft-1 when DN-112/DN-113 were "0 %
> wired". **Both landed 2026-07-11** (M-1036 + M-1060), which collapsed Draft-1's "Phase 1 (checker)"
> into *already-done*. §13 is the re-verified, build-ready plan — a **transpiler rule-swap + optional
> prelude-trait seeding**, no new serial-semcore checker phase for the single-param class. Kept below
> for lineage (append-only); read §13 for the current work-units.

**[Draft-1, superseded] Ordering is gated by DN-112 (M-1036) and DN-113 (M-1060) landing first — both
are Accepted, 0 % wired. External-trait impls cannot be built or witnessed before their substrate
exists.**

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

> **Read §13 first (2026-07-12).** Since M-1060 landed, DoD items 1–3 below are **already witnessed**
> for the single-param param-only class (the landed `register_instances` + `foreign_trait_sig…` guard);
> items 4–7 (closure coherence, acyclicity, re-homing, mono dispatch) are the **M-1076 residual**, not
> the MVP. The MVP's own DoD is §13.2's WU-A/WU-B property tests (T-A1..T-B2) + §13.3's re-measure.

Ratifying this note = accepting the **mechanism** (the §13 build-ready MVP — single-param param-only
foreign-trait impls via a transpiler rule-swap, prelude-scoped first — with M-876/M-1076 as the named
residual); the implementation must then satisfy:

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

- **OQ-1 (v1 scope, for the maintainer) — ANSWERED in §13 (2026-07-12).** Since DN-113/M-1060 landed,
  the fork is no longer "full Alt-1 vs prelude-subset gated on DN-113"; DN-113 is done. The build-ready
  v1 (§13.1) is the **single-param, param-only-sig** foreign-trait-impl class (prelude-scoped first),
  which the landed checker already admits — a **transpiler rule-swap** (WU-A), not a new checker phase.
  The `Widen`-two-type witness and the concrete-sig class are the **M-876 + M-1076 residual**. The
  maintainer's remaining choice is OQ-6 (prelude-seed vs std-phylum-declare the target traits).
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
- **`crates/mycelium-l1/src/checkty.rs`** (re-read 2026-07-12 `@dev b36ebdbe`, `Empirical`) —
  `type_head`/`qualify_type_name` (`:296`); `NoduleImports::cross_phylum_traits`/`cross_phylum_fns`
  (`:1584`); the DN-113/M-1060 subsystem doc (`:1606`–`:1633`); the `foreign_trait_sig_names_a_concrete_type`
  recognizer plus the MED-closure doc that preserves "impl a foreign trait for your own type"
  (`:4307`–`:4344`); `register_instances` orphan rule and foreign-sig guard (`:4460`–`:4536`); `check_impl_methods`
  Self-free receiver model (`:4620`–`:4694`); `check_trait_method_call` HOLE A/A2 guard (`:7711`–`:7789`);
  prelude/`Fuse` seeding (`:1358`–`:1369`, `PRELUDE_HOME` `:350`). Grounds §1, §5, §9, §13.
- **`crates/mycelium-transpile/src/{emit.rs,vet.rs,tests/vet.rs}`** (read 2026-07-12, `Empirical`) —
  `emit_trait` bails on a `Self`-binding method ("grammar has no slot", `emit.rs:2489`); the impl
  emitter renders `impl {trait}[{targs}] for {self_ty}` with **no `use`** (`emit.rs:2841`); the current
  `myc check` failure is `impl` for unknown trait `Widen` (`tests/vet.rs:69`). Grounds §1, §13.1.
- **`crates/mycelium-l1/src/ast.rs:227`** — "no implicit `self` in v0; the receiver is an explicit
  typed parameter." Grounds the §13.2 `Self`-in-trait-sig gap.
- **`tools/github/issues.yaml`** (read 2026-07-12) — M-1036 `status: done` (`:14008`); M-1060
  `status: done` (`:15839`); **M-1076** `status: todo`, `depends_on: [M-1060]` — "full cross-phylum
  trait/fn signature re-homing (close DN-113 §7 / DN-122's general fix)" (`:16440`). Grounds the
  precondition-landed correction + the residual issue mapping.

---

## §13 Build-ready MVP — re-verified post-M-1060 (2026-07-12, `Empirical @dev b36ebdbe`)

**This section supersedes §8's stale plan.** Since M-1036 + M-1060 landed (2026-07-11), the checker
substrate Draft-1 planned as a serial-semcore "Phase 1" **already exists**. The build-ready v1 is
therefore far smaller than Draft-1 assumed, and it is honest about which sub-class it closes.

### §13.1 The tightened OQ-1 MVP — what CHECKS, what REFUSES, the soundness argument

**Scope (OQ-1 answered).** v1 = the **single-parameter, param-only-signature foreign-trait-impl**
class — a **prelude-scoped subset** as the first increment, generalizing to any acyclic dependency
closure with **no further checker work** because M-1060 already imports cross-phylum traits.

**What the checker ADMITS today (no new checker code for this class — `Empirical`, confirmed by
`register_instances` + the MED-closure doc `checkty.rs:4335`–`4338`):**

- `impl ForeignTrait[targs] for LocalType` where `ForeignTrait` is a **single-parameter** trait
  (`tr.params.len() == 1`, RFC-0019 stage-1) whose method signatures reference **only** the trait's
  own generic param(s) and primitive reprs — e.g. `Cmp[A] { fn cmp(a: A, b: A) => Binary{2}; }`,
  `Eq`, `Fuse[T] { fn join(a: T, b: T) => T; }`. The orphan rule admits it because `LocalType`'s head
  is home-local; the foreign-sig guard does **not** fire (no concrete type beyond the param). This is
  the majority of the 114-gap class *by trait shape* — every "impl a foreign comparison/equality/join
  trait for my own type" case.
- `impl LocalTrait for ForeignType` symmetrically (trait home-local; the imported foreign type's head
  is a legal instance head).
- **Prelude-scoped variant** (the honest first increment): when `ForeignTrait` is a **prelude** trait
  (uniform reserved home `<prelude>`, seeded once per phylum like `Fuse` at `checkty.rs:1358`–`1369`),
  the coherence closure is exactly `{this phylum, <prelude>}` — **no cross-phylum import, no manifest
  DAG, no diamond, no separate-compilation tension** (the prelude is one uniform home, DN-112 §9). The
  transpiler emits the impl against the ambient prelude trait; zero new checker work.

**What the MVP REFUSES — never-silent (G2), leaving an honest transpiler gap, NOT a fabrication:**

1. **Foreign trait whose sig names a concrete type beyond its params** → refused by the landed
   `foreign_trait_sig_names_a_concrete_type` guard (`checkty.rs:4519`/`7774`). Admitting it needs the
   sig re-homing = **M-1076** (todo). *Honest gap, not the MVP.*
2. **Two-type / `Self`-needing traits (the canonical `Widen[To]` witness itself).** `Widen` maps
   From→To; the receiver must name the *implementing* type via `Self`, which the grammar has no slot
   for (`emit.rs:2489`) — the **M-876 `Self`-in-trait-sig sub-gap**. So the *canonical witness*
   `mycelium-std-cmp::Widen` is **in the residual, not the MVP** (stated plainly — the MVP closes the
   single-param majority, the re-measure quantifies the split; see the OQ below). The transpiler keeps
   emitting the honest gap for `Widen` until M-876 lands.
3. **Orphan `impl ForeignTrait for ForeignType`** (neither head home-local) → landed orphan refusal
   (`checkty.rs:4501`). Correct and unchanged.
4. **Overlapping/duplicate `(trait, type_head)`** → landed coherence refusal (`checkty.rs:4542`).

**Soundness argument (one paragraph — `Declared`-with-argument, matching RFC-0019's coherence tag; not
machine-checked).** The MVP adds **no new carrier and no new position** to the cross-home
bare-name-collapse surface that M-1060 already verified. That surface is `{carrier} × {position}` =
`{ctor field, fn signature, trait-method signature} × {register-time, call-time/bound-discharge}`, and
M-1060's fix-cycle-3 closed all of it: ctor fields are **re-homed** (`merge_phyla_exports`/
`qualify_ty_cross_phylum`), fn sigs are **baked-or-refused** (HOLE B, `foreign_fn_sig_names_a_concrete_type`),
trait-method sigs are **refused** at register-time (HOLE A) and bound-discharge (HOLE A2)
(`foreign_trait_sig_names_a_concrete_type`) — the same identity discipline M-1036 hardened intra-phylum,
extended one level up. The MVP admits **exactly the complement** of what that guard refuses (param-only
sigs, which carry **no** foreign concrete-type reference to collapse), so it cannot reopen a bare-name
collapse *by construction* — the guard is the invariant, the MVP lives strictly inside it. Global
uniqueness across the closure holds by the same §5 argument, now standing on **landed** substrate:
DN-112 home-qualified `type_head` keeps `(Cmp, a::T)` and `(Cmp, b::T)` distinct pairs, and DN-113's
**acyclic phylum DAG** (`§9.3`, landed in M-1060) forbids the two-legal-authors non-confluence hole
(for two phyla to each legally impl the same pair, each must name the other's head, i.e. a cycle — refused).

### §13.2 Disjoint implementation work-units (against the real tree) + property tests

Three disjoint units; **none is a serial-semcore checker phase for the MVP class** (that's why this is
now cheap). Sizes are `Declared` estimates.

- **WU-A — Transpiler rule-swap (DISJOINT from semcore; the one that moves `checked_fraction`).**
  In `crates/mycelium-transpile/src/emit.rs`, replace the "emit `impl Trait[...] for T` with no trait
  in scope" behavior (and retire the FAILED synthetic-def path, §2) with: **emit a `use
  <trait-home>.<Trait>` import** ahead of the impl **iff** (i) the trait is single-parameter and (ii)
  its signature is param-only (`foreign_trait_sig_names_a_concrete_type`-clean — mirror the checker's
  own recognizer so emit and check agree). Otherwise **emit the honest gap** (record a `GapReason`,
  do not fabricate a def). Then re-run the M-1006 ladder and **re-measure `checked_fraction`** (§13.3).
  - *Property tests (`crates/mycelium-transpile/src/tests/`):* **(T-A1 positive control)** a
    single-param param-only foreign-trait impl (`impl Cmp for <LocalType>`) emits **with** the `use`
    and `myc check`s **clean** (upgrades `tests/vet.rs:69`'s current "unknown trait" expectation for
    that shape). **(T-A2 negative control / honest-gap)** a two-type `Widen` impl still emits a
    recorded gap (no fabricated `Self` body) — the `Widen` gap of `tests/vet.rs` stays red-but-honest.
    **(T-A3 emit↔check agreement)** the emitter's param-only predicate matches the checker's
    `foreign_trait_sig_names_a_concrete_type` on a shared case table (so emit never ships a `use` for
    a shape the checker will refuse).
- **WU-B — Target-trait availability (DISJOINT; small).** The imported traits must EXIST as Mycelium
  declarations for the `use`/prelude reference to resolve. Either **declare** the single-param
  conversion/comparison traits in an importable std phylum, **or** (prelude-scoped variant) **seed**
  them into the prelude alongside `Fuse` (`checkty.rs:1358`–`1369`) so `impl PreludeTrait for LocalType`
  needs no `use` at all. Choice is OQ-6 (prelude-seed vs std-phylum-declare).
  - *Property tests:* **(T-B1)** a prelude-seeded single-param trait is visible in every nodule's
    `traits` without an import and an `impl` of it for a local type checks clean; **(T-B2)** a
    std-phylum-declared trait resolves via `use` through the M-1060 `"{dep}::…"` key and checks clean.
- **WU-C — (RESIDUAL, NOT MVP) Foreign-sig re-homing = M-1076 + `Self`-in-trait-sig = M-876.** Tracked
  separately; unblocks the `Widen`-two-type class and the concrete-type-in-sig class. Listed here only
  to draw the MVP boundary. **`Declared` / out of v1 scope.**
  - *Property tests it will need (spec'd for M-1076/M-876, not built here):* **(T-C1 coherence-soundness
    / diamond)** two phyla each attempting the same `(ForeignTrait, ForeignType)` pair is refused, and
    two distinct home-qualified pairs both check without false overlap, verified over the **full
    closure**; **(T-C2 acyclicity precondition)** a cyclic-phyla fixture refuses (DN-113 §9.3) before
    any coherence claim; **(T-C3 re-homing positive)** once a foreign sig is re-homed against its
    declaring phylum, `impl Widen[To] for LocalType` checks clean and `Self` resolves to the `for`-type;
    **(T-C4 mono cross-phylum dispatch)** a differential that the resolved instance body direct-calls
    and agrees across L1-eval / L0-interp / AOT (DN-113 §8 runtime dual).

**The DN-113 deferred-separate-compilation tension (Draft-1 §9), re-assessed.** The MVP **sidesteps**
it: the prelude-scoped closure is `{this phylum, <prelude>}` — a single uniform home, so "whole-closure
coherence" is trivially satisfied (there is no transitive closure to be partial about). The general
cross-phylum case still requires whole-closure checking (Draft-1 §9 condition i), but that is M-1076's
concern, and M-1060 already checks each dependency phylum once via the whole-graph `build_phyla_graph`
(not separate compilation), so the closure is whole by construction in v1's linking model.

### §13.3 What I adversarially attacked (VR-5 / house rule #4) and the result

1. **Can the MVP reopen the bare-name collapse across the home boundary?** *Attacked:* a foreign
   single-param trait whose sig sneaks a concrete foreign type (`Cmp[A] { fn cmp(a: A, b: A) => Bar; }`
   where `Bar` is foreign). *Result: HELD.* The landed `foreign_trait_sig_names_a_concrete_type` guard
   refuses it at register-time **and** call-time **and** bound-discharge — I traced all three sites
   (`checkty.rs:4519`, `:7774`, and the HOLE-A2 bound-discharge path). The MVP's param-only predicate
   is the guard's complement, so the MVP cannot admit this shape.
2. **Diamond trait import — two intermediates each impl the same foreign pair.** *Result: HELD for the
   MVP, correctly deferred for the general case.* Prelude-scoped closure has no diamond (one uniform
   home). The general diamond needs whole-closure coherence — that is M-1076 (T-C1), out of MVP scope,
   and flagged, not glossed.
3. **`Self`/two-type traits slipping in as "single-param".** *Result: NARROWED (hole-found-and-fixed
   in the spec).* My first cut of the MVP said "single-param foreign trait impls check clean." That is
   **false for `Widen`** — the canonical witness is two-type and needs `Self`, which is a grammar gap
   (`emit.rs:2489`), so it is **not** single-param-expressible. I narrowed the MVP to *single-param
   **AND** param-only-sig*, moved `Widen` explicitly into the M-876/M-1076 residual, and added T-A2/T-A3
   so the emitter cannot ship a `use` for a shape the checker refuses. This is the sharpest correction
   to Draft-1's framing: the top *named* witness is not in the MVP; the MVP is the single-param *majority*.
4. **Cross-phylum trait method via a generic bound (the M-1060 Hole-A/A2 class).** *Result: HELD.*
   That is exactly HOLE A2, already closed — a bound-discharged call to a cross-phylum trait whose sig
   names a concrete type is refused (`checkty.rs:7759`–`7789`). The MVP adds no new dispatch path.

**Net verdict: the recommendation SURVIVES, with one honest narrowing** — v1 is the *single-param,
param-only-sig* foreign-trait-impl class (prelude-scoped first), a **transpiler rule-swap (WU-A) +
target-trait availability (WU-B)**, riding entirely on landed M-1060 checker soundness. The
`Widen`-two-type witness and the concrete-sig class are the **M-876 + M-1076 residual**, refused
never-silently until they land. **The design is build-ready for the MVP; it is NOT a claim that the
full 12.4 % closes in v1** — the Phase-0 re-measure must report the single-param-vs-two-type split
before the leverage tag moves off `Declared` (VR-5).

**Open question raised by the re-verification:**

- **OQ-6 (target-trait availability).** Prelude-seed the single-param conversion/comparison traits
  (like `Fuse`) vs declare them in an importable std phylum? Prelude-seed is the smallest MVP (no
  `use` emission, no manifest); std-phylum-declare exercises the real M-1060 import path. **Flagged
  for the maintainer** — a real fork on merit, not guessed.
- **OQ-7 (single-param vs two-type split of the 114 gaps).** Unknown from the corpus without the
  re-measure. The MVP's leverage depends on it; the Phase-0 re-measure (WU-A) must report it. **I do
  not know this number** — stated plainly (VR-5), not estimated.

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-07-11 | **Draft** | DN-122 created. A design-reasoner DN working M-876 (external-trait impls — the P1 `checked_fraction` lever, DN-121 ~15 %/119 gaps) forward to a ranked recommendation. Recommends **Alt 1 — foreign-trait import plus closure-extended coherence on home-qualified identity** (Rank 1; Alt 2 hash-checked re-statement second, Alt 3 structural rejected; the synthetic local trait-def is the recorded FAILED precedent, §2). Coherence soundness = phylum-wide orphan rule extended to the import closure, sound **conditioned on** DN-113's acyclic phylum DAG (§9.3) plus DN-112 home identity (§5). Checker-first plus mono fast-follow plus transpiler rule-swap; **zero L0/kernel/runtime** (DN-55). Downstream of DN-112 (M-1036) and DN-113 (M-1060) — both must land first (§8). Sharpest stress-test: whole-closure coherence vs. deferred separate compilation, and the prelude-scoped subset as the honest v1 MVP if DN-113 slips (§9, OQ-1). Leverage figure `Declared` until the Phase-0/Phase-4 re-measure (§9/§10). Grounded against `integration@b36f0ef4`. **Recommends, does not ratify** (house rule #3). Authored the DN only — no edit to `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (integration-owned; FLAGGED up). |
| 2026-07-12 | **Draft** | Advanced toward ratification-ready. **Verify-first re-read against `dev@b36ebdbe` (mitigation #14) found the ground had moved: DN-112/M-1036 and DN-113/M-1060 BOTH landed 2026-07-11** — so Draft-1's "0 % wired / must land first" premise is corrected in the header, banner, and §1. **M-1060 already landed the checker substrate** Draft-1 planned as a serial-semcore "Phase 1": the common `impl ForeignTrait for LocalType` (single-param, param-only sig) **checks clean today** (`register_instances` + the MED-closure preserving that pattern, `checkty.rs:4335`), and the only unsound shapes are **refused never-silently** by the landed `foreign_trait_sig_names_a_concrete_type` guard (HOLE A/A2/B, `checkty.rs:4339`/`4519`/`7774`). New **§13 (build-ready MVP)** supersedes §8's stale plan: v1 = the **single-param param-only foreign-trait-impl class (prelude-scoped first)** via a **transpiler rule-swap (WU-A) + target-trait availability (WU-B)** — no new checker phase; the concrete-sig re-homing = **M-1076** (already minted, todo) and the `Widen`-two-type/`Self`-in-sig case = **M-876** are the named residual (WU-C). Soundness argument re-grounded on the **landed** `{carrier}×{position}` surface M-1060 verified (the MVP is the guard's complement — cannot reopen a bare-name collapse by construction). Adversarial stress-test (§13.3) **narrowed** the MVP: the canonical `Widen` witness is two-type/`Self`-needing (`emit.rs:2489` grammar gap), so it is **in the residual, not the MVP** — the MVP closes the single-param majority; the Phase-0 re-measure must report the split (OQ-7). Fresh Phase-0 figure 114 gaps / 12.4 %. New OQ-6 (prelude-seed vs std-phylum-declare) / OQ-7 (single-param split) flagged. Property tests spec'd per WU (T-A1..T-C4). **Still Draft — recommends, does not ratify** (house rule #3). Re-verified `Empirical` claims cite `file:line @dev b36ebdbe`; unbuilt pieces stay `Declared` (VR-5). Authored the DN only; `issues.yaml`/`CHANGELOG.md`/`Doc-Index.md` FLAGGED up, not edited. |
