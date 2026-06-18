# Research Record 10 — Traits: coherence + Repr-polymorphism soundness (RFC-0019 / RP-3)

> **What this file is.** A durable record discharging the **RP-3** research prompt
> (`docs/notes/research-prompts.md`) and the **RFC-0019 §9** pre-ratification prompts **R1**
> (coherence soundness under content-addressed identity) and **R2** (Repr-polymorphism soundness under
> S1). Conducted 2026-06-18 from the RFC-0019 draft, RFC-0006 §4.1/§4.2 (S1, LR-5/LR-6), RFC-0001
> §4.5/§4.6 (the Prim/Swap rules, content-addressing), RFC-0007 §4.4 (the monomorphic judgment), and
> the typeclass-coherence / representation-polymorphism literature already cited in `research/03`
> (T3.1/T3.3). Findings are labeled **T10.1–T10.9** (continuing the T0–T9 scheme) and map onto
> RFC-0019 §4.5 (coherence/orphan rule), §4.6 (Repr-polymorphism restriction set), §8 (Q-coherence,
> Q-reprpoly, Q-multi-param, Q-associated-types), and §9 (R1, R2).
>
> **Posture (honesty rule / VR-5).** Like `research/09`, this record delivers **stated theorems with
> hand-constructed proof sketches**, grounded in cited prior art; they are **not machine-checked**, so
> the soundness *claims* are tagged **Declared-with-argument**, never **Proven** (mechanization is the
> future `Proven` basis). Design recommendations (reject overlap; reject newtype waivers in v1; the
> Repr-polymorphism restriction set; defer multi-param/associated types) are **recommendations** for
> the maintainer, not ratifications. Append-only.

---

## 1. Scope

RFC-0019 ships a complete trait layer (dictionary-passing elaboration to existing kernel nodes — zero
new nodes; coherence + orphan rule; LR-5 Repr-polymorphism; LR-6 guarantee-indexed methods) and
defers ratification (§10) on two **flagged-novel** research prompts: **R1** (the coherence soundness
proof / Q-coherence) and **R2** (the Repr-polymorphism restriction-set soundness / Q-reprpoly). This
record discharges both as sketches and gives recommendations on the secondary open questions
(Q-multi-param, Q-associated-types). It answers RP-3's three sub-questions: the coherence *mechanism*,
the *restriction set*, and the *soundness statement*.

---

## 2. Findings

### T10.1 — Why content-addressing makes coherence a *correctness* property, not a style rule

Under ADR-003 a definition's identity is its α-normalized structure hash. A dictionary is
content-addressed over `(trait_ref, type_arg, method_bodies)` (RFC-0019 §4.2). If two distinct
instances `impl Ord for Binary{8}` with *different* orderings could both be valid, then the *same*
`(Trait, Type)` resolution would map to *two* dictionary hashes depending on scope — and a generic
caller's content hash would denote two semantics. That is a direct ADR-003 violation. Hence
**incoherence is a soundness failure** in Mycelium, strictly worse than the usability problem it is in
Haskell/Rust. This raises the bar: the resolution function must be not merely "usually unambiguous"
but **total, deterministic, and hash-stable by construction**. Grounding: RFC-0019 §2.2; ADR-003.

### T10.2 — The coherence mechanism: Rust-style orphan rule + global uniqueness + reject-overlap (RP-3 sub-question 1)

Of the three standard mechanisms — Haskell global coherence (one instance per type, globally),
Rust orphan rules, Scala local implicits — **only a Rust-style orphan rule with global uniqueness and
overlap-rejection is consistent with content-addressing.** Reasoning:

- *Scala local implicits* make resolution **scope-dependent**: the same `(Trait, τ)` can resolve to
  different instances in different import scopes. Under ADR-003 that is exactly the two-semantics-one-
  hash failure (T10.1). **Rejected.**
- *Haskell whole-program global coherence* is sound but assumes whole-program instance visibility,
  which breaks separate compilation (a `nodule` must be checkable without the whole program). **Too
  strong for the nodule model.**
- *Rust orphan rule* ties each instance to a **definition site** (the trait's home `nodule`/`phylum`
  or the type's home), which is itself content-addressed and fixed. The candidate-instance set for
  `(Trait, τ)` is therefore determined by two content-addressed locations and **cannot be enlarged by
  adding an unrelated dependency** (orphan instances are rejected at check time). This is the
  mechanism that gives stable, definition-site-determined resolution under separate compilation.

*Position.* Adopt three rules together: the orphan rule (RFC-0019 §4.5), global uniqueness (≤1
instance per `(Trait, τ)`), and **rejection of overlapping instances** (Haskell-98 / Rust coherence,
*not* GHC `OverlappingInstances`). Grounding: Wadler & Blott (POPL 1989, typeclasses); Rust RFC 1023
(coherence/orphan); RFC-0019 §4.5.

### T10.3 — R1: coherence soundness — the resolution invariant, stated and argued

> **Theorem (coherence soundness).** For any well-formed registry `Σ` satisfying the orphan rule,
> global uniqueness, and overlap-rejection, the instance resolution `inst(Trait, τ) ↝ dict` is
> **total** (resolves to the unique instance or fails with an explicit, never-silent error),
> **deterministic** (a pure function of `Σ`, `Trait`, `τ`), and **hash-stable** (the resolved `dict`
> hash is independent of the order in which `InstanceDecl`s were added to `Σ`).

*Proof sketch.* (a) **Determinism + stability:** the candidate set `C(Trait, τ)` is exactly the
instances declared in the trait's home or `τ`'s home `nodule` (orphan rule); both homes are
content-addressed and fixed, so `C` is a pure function of `Σ` independent of insertion order. Global
uniqueness + overlap-rejection give `|C| ≤ 1` after a *confluent* check (no specificity ordering is
consulted — overlap is rejected, so there is never a "more specific" tie-break to compute). Hence
`inst` returns the single element of `C` or fails — deterministically, and the returned `dict`'s hash
is its content hash (independent of order). (b) **Totality:** `|C| = 0` is an explicit
`MissingInstance` error; `|C| ≥ 2` is impossible for a *well-formed* `Σ` (the uniqueness/overlap check
rejects it at registration with both conflicting hashes named — never a silent shadowing). ∎-sketch

*Why reject overlap (and not adopt specialization).* Rust's specialization (RFC 1210) admits
overlapping instances ordered by a global specificity lattice; it has well-known soundness hazards
(lifetime-dependent specialization unsoundness; non-confluence under associated types) and remains
unstable years on. For a language where the dictionary is **content-addressed**, a non-confluent
resolution is a *correctness* failure (T10.1), so overlap-rejection is not a conservatism but a
requirement. Grounding: Rust RFC 1210 (specialization) and its documented soundness holes; RFC-0019
§4.5.

### T10.4 — Q-coherence (newtype-derived waivers): reject in v1; admitting them safely needs *roles*

`GeneralizedNewtypeDeriving`-style waivers copy a wrapped type's dictionary onto a newtype via a
representational coercion. This is **sound only under a roles discipline** (Weirich, Vytiniotis,
Peyton Jones et al., *Safe Zero-cost Coercions for Haskell*, ICFP 2014 / JFP 2016): without roles, a
method whose signature uses the type parameter in a *role-sensitive* position (e.g. inside a type
family / associated type) can be coerced across a representational boundary it must not cross,
producing unsoundness. Verdict for Mycelium v1:

- v1 has **no associated types** (T10.8) and **no roles mechanism**. Admitting newtype waivers now
  would either be unsound (no roles) or require importing the whole roles apparatus (kernel-growth
  pressure against KC-3).
- **Recommendation: reject all coherence waivers in v1** (the conservative, sound choice, consistent
  with overlap-rejection). Revisit with a dedicated *roles* RFC if/when associated types land — at
  which point a representational-coercion waiver becomes admissible *with* the role guard. Grounding:
  Weirich et al. ICFP 2014; RFC-0019 §4.5 Q-coherence.

### T10.5 — R2: the Repr-polymorphism restriction set, made precise (RP-3 sub-question 2)

LR-5 asks for `∀ R: Repr` abstraction that never violates **S1** (no elaboration/instantiation step
inserts a `Swap` — RFC-0006 §4.1; RFC-0001 WF1/WF8). The restriction set (consolidating RFC-0019 §4.6
(1)–(4) and RP-3's candidate restriction):

1. **Kind stratification.** A `Repr` kind with sub-kinds `BinaryRepr`/`TernaryRepr`/`DenseRepr`/
   `VSARepr` and super-kind `AnyRepr`. A variable `R: K` ranges only over reprs of kind `K`.
2. **No paradigm-specific `Op` on a Repr-abstract argument** (the load-bearing restriction). A value
   of an abstract type `R` may be: **(i)** passed through (bound, returned, stored), **(ii)** an
   argument to a method of `R`'s declared trait bound (the dictionary supplies it), or **(iii)**
   `Swap`-ed by a **lexically-present** `swap(…, to: R₂, policy: p)`. It may **not** be an operand of
   a primitive `Op` whose signature demands a concrete paradigm (RFC-0001 §4.5 *(Prim)*: there is no
   paradigm coercion). Paradigm-specific work goes through the trait interface, never a bare prim on
   an abstract value.
3. **Swap-explicitness at the generic level (S1 restated).** Every representation change inside a
   generic body is a lexical `Swap` node; specialization performs only substitution, never insertion.
   A specializer that *discovers* a needed conversion rejects with an explicit
   `UnresolvedReprPolymorphism` / `MissingConversion` diagnostic — never a silent insert.
4. **`ReprDesc` witness for runtime dispatch.** A generic receives a runtime paradigm descriptor
   (RFC-0001 §3.3) in its dictionary; the body may branch on it only through an explicit `match`,
   never an implicit coercion.

*Local checkability (RP-3 sub-question 2).* Restriction (2) is a **purely local syntactic check** on
the generic body against its trait bound's interface — no whole-program analysis: every use of an
`R`-typed value is checked to be passthrough, a bound-method call, or a lexical `Swap`. This answers
RP-3's "is it checkable locally?" — **yes**. Grounding: RFC-0019 §4.6; RFC-0001 §4.5; research/03 T3.3.

### T10.6 — R2: S1-preservation soundness — theorem and sketch (RP-3 sub-question 3)

> **Theorem (Repr-polymorphism S1-soundness).** Let `g` be a generic definition well-typed under the
> §4.6/T10.5 restriction set. For any kind-correct substitution `R := r` (a concrete repr) and the
> coherent dictionary `d = inst(Bound, r)` (well-defined by T10.3), the specialization `g[R := r, d]`
> is **(a)** well-typed under RFC-0007 §4.4's monomorphic judgment, and **(b)** contains **no `Swap`
> node that was not lexically present in `g`**.

*Proof sketch.* Structural induction on `g`'s body. Specialization is the *substitution* `[R := r, d]`
— it introduces no new term constructors, only replaces the abstract type and supplies the dictionary.

- **`Var` / passthrough:** substitution renames a type; no node added; (b) holds trivially.
- **Bound-method call (`App` of a dictionary field):** the method body is the instance's *monomorphic*
  definition, already S1-checked (no inserted `Swap`); supplying it via the dictionary adds no `Swap`.
  Its type is the instance's method type at `r` — well-typed by the instance's existence (T10.3). (a),(b) hold.
- **Lexical `Swap`:** present in `g`'s source; substitution fixes its `target` to a concrete repr; it
  is *counted*, not inserted; well-typed per RFC-0002 at `r`. (b) holds (it was already there).
- **`Construct`/`Match`/`Let`/`Lam`/`App`:** Repr-transparent (WF8); recurse; no `Swap` introduced.
- **The excluded case (the crux):** the *only* place a specializer could be forced to insert a
  conversion is a paradigm-specific `Op` applied to an `R`-abstract value (e.g. `bit.and` needs
  `Binary` but `R` might specialize to `Ternary`). **Restriction (2) forbids exactly this** — such an
  `Op` is rejected at the generic check, before any specialization. So the tempting insertion site
  never arises. ∎-sketch

*Confirmation/falsification (RP-3).* The restriction set is **sufficient**: every restricted generic,
specialized to any concrete repr, type-checks monomorphically with no inserted `Swap` (the theorem).
It would be **insufficient** if a counterexample generic, obeying (1)–(4), forced an inserted swap on
some instantiation — the induction shows the only such site (paradigm-specific `Op` on abstract `R`)
is excluded, so no counterexample exists within the restriction set. The set is also **non-empty/
useful**: any function over the trait interface (e.g. `fn dup<R: Clone>(x: R) -> (R, R)`,
`fn id<R: Repr>(x: R) -> R`) is expressible; cross-paradigm work is expressible by naming the `Swap`.

### T10.7 — Relationship to GHC levity polymorphism (the precedent, dualized)

GHC's levity/representation polymorphism (Eisenberg & Peyton Jones, *Levity Polymorphism*, PLDI 2017
— production-grade) is sound under exactly two restrictions: **no representation-polymorphic binders**
and **no representation-polymorphic function arguments** — because the *code generator* must know a
value's representation to move/store it. Mycelium's setting differs in two ways recorded in
research/03 T3.3: (i) reprs are tracked at **runtime** (a `ReprDesc` witness / dictionary can move a
value repr-abstractly), which *relaxes* GHC's two restrictions; (ii) Mycelium adds the **S1**
constraint (no inserted swap), which GHC has no analogue of. The T10.5 restriction set is the
**dual**: where GHC forbids repr-poly *binders/args* to keep **codegen** sound, Mycelium forbids
*paradigm-specific `Op`s on repr-abstract values* to keep **S1** sound. This is why the restriction is
about *operations*, not *binders* — and why a runtime witness is admissible here but a compile-time-
erased one is what GHC needs. Grounding: Eisenberg & Peyton Jones PLDI 2017; research/03 T3.3.

### T10.8 — Q-multi-param and Q-associated-types: recommend defer to v2

- **Multi-parameter traits** (`trait Coerce<A, B>`) reintroduce the orphan-rule subtlety (which
  parameter's home counts) and the coherence-overlap surface that Rust handles with extra machinery.
  v1's single-parameter traits keep T10.3's coherence proof clean. **Recommend: defer to v2.**
- **Associated types** are the feature whose interaction with coercions *requires* the roles
  discipline (T10.4). Deferring them keeps v1 free of roles (KC-3). **Recommend: defer to v2**; bundle
  with the roles RFC that would also unlock newtype waivers (T10.4). Grounding: RFC-0019 §8.

### T10.9 — Dictionary-passing is the right elaboration (kernel-budget + content-addressing)

Confirmed: dictionary-passing elaborates traits to **existing** kernel nodes (`Construct` a dictionary
record, `Match`/projection, `Lam`/`App` for method calls, `Let` for bindings) — the kernel node budget
(RFC-0007 §4.1) does **not** grow (KC-3). It also gives a generic *one* content-addressed hash
regardless of instantiation count (ADR-003) and supports separate compilation directly (the consuming
`nodule` does not re-elaborate). Monomorphization is recorded as an *optional, non-normative* AOT
specialization (outside the trusted kernel), under which the Repr-polymorphism restrictions partly
relax (à la GHC, T10.7) — a performance promotion, not a semantic requirement. Grounding: RFC-0019
§4.3/§4.4; research/03 T3.1 (the GHC-Core/Lean/Unison convergence on a small term grammar + a
separate declaration layer).

---

## 3. Decisions this record supports

- **R1 (coherence soundness) → discharged** by T10.2 (mechanism: orphan rule + global uniqueness +
  reject-overlap) and T10.3 (the total/deterministic/hash-stable resolution theorem + sketch).
- **Q-coherence (newtype waivers) → recommend reject in v1** (T10.4); safe admission needs a roles
  mechanism (Weirich et al.), bundled with associated types in a future RFC.
- **R2 (Repr-polymorphism soundness) → discharged** by T10.5 (the locally-checkable restriction set)
  and T10.6 (the S1-preservation theorem + sketch), with T10.7 grounding it as the dual of GHC levity
  polymorphism.
- **RP-3's three sub-questions → answered:** mechanism = Rust-style orphan coherence (T10.2);
  restriction set = "no paradigm-specific `Op` on Repr-abstract args; passthrough/trait-interface/
  lexical-swap only", locally checkable (T10.5); soundness statement = the T10.6 theorem.
- **Q-multi-param / Q-associated-types → recommend defer to v2** (T10.8).
- **Elaboration → confirm dictionary-passing**, kernel budget unchanged (T10.9).

All soundness results are tagged **Declared-with-argument** (sketches, not machine-checked);
mechanization is the future `Proven` basis. The **decisions** (reject overlap, reject v1 waivers, the
restriction set, the deferrals) are the maintainer's to record; this record discharges the *research*
that was gating them.

---

## 4. Key sources

- **Wadler & Blott**, *How to make ad-hoc polymorphism less ad hoc* (POPL 1989) — typeclasses +
  dictionary-passing translation.
- **Rust RFC 1023 (coherence / orphan rule)** and **RFC 1210 (specialization)** — the orphan-rule
  mechanism adopted (T10.2) and the overlapping-specialization soundness hazards avoided (T10.3).
- **Weirich, Vytiniotis, Peyton Jones, et al.**, *Safe Zero-cost Coercions for Haskell* (ICFP 2014;
  JFP 2016) — roles; why `GeneralizedNewtypeDeriving`-style waivers need a role guard (T10.4).
- **Eisenberg & Peyton Jones**, *Levity Polymorphism* (PLDI 2017) — the representation-polymorphism
  precedent and its two restrictions; Mycelium's restriction set is the S1-dual (T10.7).
- **Futhark / Dex** typed-array systems (T3.3) — closest relatives for abstracting over data
  representation; neither enforces a never-silent-swap rule (the Mycelium novelty, flagged).
- In-repo: RFC-0019 (the trait layer, §4.2–§4.6, §8–§9); RFC-0006 §4.1 S1 / §4.2 LR-5/LR-6; RFC-0001
  §4.5 (Prim/Swap rules), §4.6 (content-addressing), WF1/WF8; RFC-0007 §4.1/§4.4 (node budget, the
  monomorphic judgment); RFC-0002 (per-swap certificate); ADR-003 (Unison identity); `research/03`
  T3.1/T3.3 (the convergence + Repr-polymorphism position).

---

## 5. Honest-uncertainty register

- **Both soundness results are stated theorems with proof *sketches*, not machine-checked.** Tagged
  **Declared-with-argument** (VR-5); a `Proven` upgrade needs mechanization (the elaborator is the
  untrusted layer, so the obligation is on the *meta-theory*, not the kernel).
- **"No found mainstream analogue" for S1-enforced kind-stratified Repr-polymorphism** (RFC-0019
  §4.6; research/03 T3.3) is *absence of evidence*, not proof of novelty — flagged.
- **The restriction set's *completeness* (expressiveness) is argued by examples, not characterized.**
  T10.6 shows it is non-empty and useful, but does not prove it captures *every* desirable generic; a
  future relaxation (e.g. admitting some multi-paradigm dispatch via the `ReprDesc` witness) may be
  sound and is left open.
- **The coherence proof assumes the orphan rule is *enforced* as specified.** It is a meta-theorem
  about a well-formed `Σ`; the enforcing check (reject orphan/overlap at registration) is the
  trusted obligation — its implementation correctness is a separate (testable) concern.
- **Q-coherence verdict (reject waivers) is conservative, not forced.** A roles mechanism *could*
  admit them soundly; "reject in v1" is the KC-3-minimal choice, not the only sound one.

---

## Meta — changelog

- **2026-06-18 — Created.** Discharges RP-3 / RFC-0019 §9 R1+R2: the coherence mechanism (Rust-style
  orphan rule + global uniqueness + reject-overlap, T10.2) and its total/deterministic/hash-stable
  resolution theorem (T10.3); the Q-coherence verdict (reject newtype waivers in v1 — needs roles,
  T10.4); the Repr-polymorphism restriction set, locally checkable (T10.5) and its S1-preservation
  theorem + sketch (T10.6), grounded as the dual of GHC levity polymorphism (T10.7); and the
  recommend-defer verdicts for multi-param/associated types (T10.8). Confirms dictionary-passing keeps
  the kernel budget unchanged (T10.9). All soundness results tagged Declared-with-argument (not
  machine-checked — VR-5). Grounds RFC-0019 toward ratification; the RFC stays Draft pending the
  maintainer's append-only decisions (all recommended above). Append-only.
