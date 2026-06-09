# RFC-0001 — Core IR & Metadata Schema

| Field | Value |
|---|---|
| **RFC** | 0001 |
| **Title** | Core IR & Metadata Schema |
| **Status** | **Accepted** (r2 — §4.3 `Bound` grammar revised per **ADR-011**; supersedes r1) |
| **Type** | Foundational / normative |
| **Date** | June 08, 2026 |
| **Depends on** | *Mycelium Project Foundation* (r3): FR-M1/M3/M4/M5/M8, FR-S2, NFR-3/6/7, VR-1/2/3/4/5, SC-3/4, ADR-001/002/003/006/008 |
| **Refines** | Foundation §5.2 core-model sketch (now superseded by this RFC) |
| **Blocks** | RFC-0002, RFC-0003, RFC-0004, RFC-0005 (all now Accepted) |
| **Resolved dependencies** | ADR-010 (verified-numerics) **Accepted** → §4.7 composition now concrete; DN-01 (packing) **Resolved** → §4.1 now schedule-staged |

---

## 0. Grounding & traceability

Every normative choice below cites the Foundation by its labels (`FR-*`, `NFR-*`, `VR-*`, `SC-*`, `ADR-*`, `§5.x`), and through the Foundation, the prior-art survey (Areas, `G1–G11`, tensions `A–E`). This RFC introduces no new prior art; it is a detailed *design* of slots the Foundation already mandated. Where it **refines** the Foundation's candidate sketch, it says so explicitly (§4.1, §6).

## 1. Summary

Defines the Mycelium **Core IR** — the typed, content-addressed, metadata-bearing intermediate representation that is the single source of truth for a program — and the **metadata schema** that travels with every value. It establishes four normative pillars:

1. **Representation paradigm is part of the type.** `Binary`, `Ternary`, `Dense`, `VSA` are distinct, parameterized type families (FR-M1).
2. **No implicit conversion.** The kernel has *no* coercion rule between paradigms; the only node that changes a value's representation is an explicit `Swap` (FR-M3, FR-W2; cross-cutting **A.1**).
3. **Honesty is a typed, monotone property.** A `GuaranteeStrength` lattice (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`) propagates by *meet* through every operation, so an approximation's disclosed strength can only degrade, never spuriously upgrade (Tension **B**; VR-3/VR-5).
4. **Metadata is self-describing, inspectable, and survives lowering.** Provenance, bounds, layout, and reconstruction info travel with values and are queryable (Arrow-grade self-description; dimensional persistence, contra units-erasure) (FR-M5; NFR-3; Area 4).

It defines the extension hooks consumed by RFC-0002/0003/0004/0005 and defers their internals.

## 2. Motivation

The Core IR is the foundational artifact: the swap-certificate format, the VSA algebra, the lowering/execution pipeline, and the policy language all reference the value model, the type discipline, and the metadata schema. Specifying it first (the chosen drill-down order) means the dependent RFCs plug into stable slots rather than re-litigating the data model. It is also where the project's non-negotiables become *mechanically checkable* rather than aspirational: "no black boxes" is realized here as a small set of well-formedness invariants (§4.5) that the linter (Foundation §5.8) enforces.

## 3. Guide-level explanation

### 3.1 Values, paradigms, and types

A Mycelium **value** is a payload plus a representation descriptor plus metadata. Its **type** carries the representation paradigm and the semantically-significant parameters:

- `Value<Binary{width: 8}>` — an 8-bit binary value.
- `Value<Ternary{trits: 6}>` — six balanced trits {−1, 0, +1} (Area 3).
- `Value<Dense{dim: 768, dtype: F32}>` — a 768-d dense embedding (Area 2/4).
- `Value<VSA{model: MAP, dim: 10000, sparsity: Dense}>` — a hypervector; the *type slot* lives in the kernel, its *operations* in the VSA submodule (ADR-008; §4.9).

Two values are the same *type* only if their paradigm and semantic parameters match. `Binary{8}` and `Ternary{6}` are unrelated types with no subtype relationship.

### 3.2 Metadata: what travels with a value

Beyond the type, each value carries runtime **metadata** (`Meta`, §4.3): where it came from (`provenance`), how trustworthy its representation is (`guarantee` + `bound`), its measured `sparsity`, its concrete in-memory `physical` layout, any `reconstruction` info for lossy/holographic forms (the generalized Embeddenator "Manifest" idea, treated as provisional — **G6**), and, if a swap produced it, the `policy_used` (ADR-006; **G2**). Metadata is queryable at runtime and is the data the LSP surfaces to humans and AI alike (Foundation §5.8).

### 3.3 No implicit conversions (the central rule)

You cannot add a `Binary` value to a `Ternary` value. The type checker rejects it — there is no hidden coercion. To combine them you write an explicit `swap`, which yields a value in the target paradigm *and* a certificate describing what the conversion cost (RFC-0002). This is the mechanical form of "no black boxes" (cross-cutting **A.1**; FR-M3).

### 3.4 Guarantee strength and honesty propagation

Every value carries a `GuaranteeStrength`:

- **`Exact`** — no approximation (pure binary/ternary arithmetic; a bijective binary↔ternary swap). `bound == None`.
- **`Proven`** — approximate, with a *machine-checked* error/probability bound (e.g., a capacity bound from a proven theorem such as Clarkson-Ubaru-Yang 2023).
- **`Empirical`** — approximate, with an *empirically-validated* bound (e.g., a Frady-Sommer Gaussian-approximation capacity estimate). Honest about its weaker basis (**G5**; VR-5).
- **`Declared`** — approximate, with a *user-asserted* bound not yet validated. Tooling must always flag it; it is never silently trusted.

These form a lattice, `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`. Any operation's result takes the **meet** (weakest) of its inputs' strengths and the operation's own intrinsic strength. So combining an `Empirical` hypervector into an otherwise `Exact` computation yields an `Empirical` result — disclosure can only degrade. This is the type-level realization of the Tension-B resolution (ADR-001): approximation is first-class but never hidden, and its trustworthiness is conserved downward.

### 3.5 Worked example

```text
let a: Value<Binary{width:8}>  = const 0b1011_0010        // guarantee = Exact
let b: Value<Ternary{trits:6}> = const ⟨+,0,-,+,0,-⟩       // guarantee = Exact

// add_binary(a, b)
//   ✗ TYPE ERROR: operand 2 has paradigm Ternary, expected Binary.
//     No implicit conversion (WF1). Insert an explicit `swap`.

let b8: Value<Binary{width:8}> =
        swap(b, target = Binary{width:8}, policy = round_trip_safe)
//   ⇒ SwapCertificate::Bijective(proof)            (per RFC-0002)
//     b8.meta.guarantee   = Exact
//     b8.meta.policy_used = Some(round_trip_safe)

let s = add_binary(a, b8)                 // meet(Exact, Exact) = Exact

// --- VSA side (requires the VSA submodule, RFC-0003) ---
let bundled = bundle([hv1, hv2, hv3])     // hvN : Value<VSA{MAP,10000,Dense}>
//     bundled.meta.guarantee = Empirical
//     bundled.meta.bound     = Some(CapacityBound{
//                                items: 3, dim: 10000,
//                                basis: EmpiricalFit{ trials: 10_000,
//                                                     method: "Frady-Sommer Gaussian" }})

let r = combine(s, bundled)               // meet(Exact, Empirical) = Empirical
//     r.meta.guarantee = Empirical        // honesty degrades, as required
```

## 4. Reference-level explanation (normative)

### 4.1 Representation descriptors (`Repr`)

```ebnf
Repr          ::= "Binary"  "{" "width:"  Nat "}"
                | "Ternary" "{" "trits:"  Nat "}"
                | "Dense"   "{" "dim:"    Nat "," "dtype:" ScalarKind "}"
                | "VSA"     "{" "model:"  ModelId "," "dim:" Nat "," "sparsity:" SparsityClass "}"
SparsityClass ::= "Dense" | "Sparse" "{" "max_active:" Nat "}"
ScalarKind    ::= "F16" | "BF16" | "F32" | "F64"        // extensible registry
ModelId         = identifier resolved against the VSA model registry
```

**Closed kinds, open registries (decision).** The four paradigm *kinds* are **closed** in the kernel: adding a fifth requires an RFC + ADR. This trades the survey's "open/extensible type system" point (MLIR, Area 1) against auditability (`KC-3`, NFR-6); a small fixed set of kinds keeps the kernel inspectable, while the *parameter* registries (`ScalarKind`, `ModelId`, and `PackScheme` in §4.3) remain extensible. The `ModelId` registry is populated by the VSA submodule (ADR-008), not the kernel.

**Type captures semantics; physical layout is schedule-staged, not typed (decision; refines Foundation §5.2, confirmed by DN-01 + research T1.4).** The Foundation sketch placed ternary *packing* inside `Repr`; DN-01's tradeoff study and the research pass instead establish that **lossless physical packing is neither in the type nor a free-floating runtime tag — it is a *schedule* chosen at a lowering stage** (RFC-0004 §5) and *recorded* (not decided) in `Meta.physical`. Rule: *the type carries the logical paradigm and any semantically-significant (potentially lossy) parameter; lossless physical layout is a schedule artifact, recorded as inspectable metadata.* Rationale (also §6): two ternary values of the same logical `Ternary{trits:6}` may be packed differently (unpacked in development vs. TL2-packed in a stable AOT component, ADR-009) yet must remain the **same type** so they interoperate; packing is a lossless re-encoding chosen for performance, so it belongs to the schedule, with `Meta.physical` as its auditable record. T1.4 confirms the small fixed packing set (≈5 schemes) makes the schedule selection tractable (not Halide's hard general problem). `dtype` stays in the `Dense` type because precision is semantically significant (it bounds embedding error).

### 4.2 The value model

```text
Value<R: Repr> = {
    payload: Payload,      // bits | trits | scalar vector (representation-specific)
    meta:    Meta,         // §4.3 — runtime, queryable, survives lowering
}
```

The static type is `Value<R>`; `meta` is runtime data. The split between what is in the type vs. in `meta` is normative (§4.4). `Payload` encodings are representation-specific and defined per paradigm (binary words; trit sequences; scalar arrays; hypervector storage — sparse index/value pairs or dense arrays).

### 4.3 Metadata schema (`Meta`)

```text
Meta = {
    provenance:     Provenance,             // §4.6; NOT part of code identity
    guarantee:      GuaranteeStrength,      // Exact | Proven | Empirical | Declared
    bound:          Option<Bound>,          // None  iff  guarantee == Exact
    sparsity:       Option<SparsityObs>,    // measured: { active: Nat, density: Real }
    physical:       Option<PhysicalLayout>, // RECORD of the schedule-staged packing (RFC-0004 §5); inspectable, not the decision locus (FR-M5, NFR-4)
    reconstruction: Option<ReconInfo>,      // Manifest-style; for lossy/holographic forms (G6, provisional)
    policy_used:    Option<PolicyRef>,      // set iff produced by a Swap (ADR-006)
}

GuaranteeStrength ::= Exact | Proven | Empirical | Declared

Bound      ::= { kind: BoundKind, basis: BoundBasis }  // r2 (ADR-011): `basis` is a companion of EVERY bound; supersedes r1, where only CapacityBound carried it
BoundKind  ::= ErrorBound      { eps: Real, norm: NormKind }
             | ProbabilityBound { delta: Real }                       // failure probability
             | CrosstalkBound   { expected: Real, tail: Option<Real> }
             | CapacityBound    { items: Nat, dim: Nat }
NormKind   ::= L1 | L2 | Linf | Rel                                   // extensible registry (r2)
BoundBasis ::= ProvenThm    { citation: Text }      // e.g. "Clarkson-Ubaru-Yang 2023"
             | EmpiricalFit { trials: Nat, method: Text }  // e.g. "Frady-Sommer Gaussian"
             | UserDeclared

PhysicalLayout ::= BinaryWords | TritPacked { scheme: PackScheme } | DenseArray | VsaStore { sparse: Bool }
PackScheme     ::= Unpacked | TwoBitPerTrit | FiveTritPerByte | I2S | TL1 | TL2  // extensible registry
```

> **r2 note (ADR-011).** `basis` is a required companion of **every** `Bound`, not just
> `CapacityBound` (the r1 grammar, now superseded). The strength tag derives from `basis` for all
> bound kinds, which is what M-I2/M-I3/M-I4 below already require and what RFC-0002 §3 (a certificate
> carries `Bound` + `BoundBasis`) assumes. An `Exact` value carries no `Bound` (M-I1), so this adds
> nothing there. `NormKind` is enumerated `L1|L2|Linf|Rel` as an extensible registry (§4.1).

**Schema invariants (normative).**
- **M-I1.** `guarantee == Exact  ⟺  bound == None`.
- **M-I2.** `guarantee == Proven  ⟹  bound.basis == ProvenThm{..}`.
- **M-I3.** `guarantee == Empirical ⟹ bound.basis == EmpiricalFit{..}`.
- **M-I4.** `guarantee == Declared ⟹ bound.basis == UserDeclared`, and any tool presenting the value MUST surface a "declared, unverified" marker (VR-5; honesty).
- **M-I5.** `physical` is always a *lossless* encoding of `payload`; changing `physical` MUST NOT change the value's type or its `guarantee`.

### 4.4 Static contract vs. dynamic metadata; what is part of code identity

Two distinct notions, deliberately separated (SRP):

- **Static contract** — the type `Value<R>` plus any *declared* static bounds attached to a definition's signature (e.g., a function annotated to return a `Sparse{max_active: k}` VSA value, or a declared error tolerance). The static contract **is** part of code identity (§4.6). The declared sparsity *class* (`Sparse{max_active:k}`) is a **static refinement** discharged by SMT (RFC-0003 §5, research T1.3); resulting **capacity bounds** are refinement post-conditions whose soundness is an axiomatized cited theorem (T0.2), with the checker discharging only the arithmetic instantiation. *Observed* sparsity stays in dynamic metadata.
- **Dynamic metadata** — the runtime `Meta` of a particular value (its provenance DAG, measured sparsity, realized bound, `policy_used`). Dynamic metadata is **not** part of code identity; it is per-value data produced during execution.

This separation lets the type checker reason about contracts statically while runtime metadata records what actually happened — both inspectable, neither conflated.

### 4.5 Core IR & typing discipline

**Node grammar (core subset; the full term language — abstraction/application, recursion, modules — is layered above this and is out of scope here):**

```ebnf
Node ::= Const { value: Value }
       | Var   { id: VarId }
       | Let   { id: VarId, bound: Node, body: Node }
       | Op    { prim: Prim, args: [Node] }                    // paradigm-specific primitive
       | Swap  { src: Node, target: Repr, policy: PolicyRef }  // the ONLY Repr-changing node
```

**Typing judgment.** `Γ ⊢ e : Value<R>`, where `R: Repr`. Selected rules:

- *(Prim)* Each `Prim` declares operand and result paradigms. `Op{prim, args}` is well-typed only if each argument's `Repr` matches the prim's declared operand paradigm exactly. **There is no coercion/subsumption rule between paradigms.** (FR-M3, FR-W2.)
- *(Swap)* `Swap{src, target, policy}` is well-typed if `Γ ⊢ src : Value<R_src>`, `target : Repr`, and `policy : PolicyRef`; its result type is `Value<target>`. It is the *only* rule under which result `Repr ≠ operand Repr`. The certificate it produces and the legality of the specific `(R_src → target)` pair are RFC-0002's concern (split regime, ADR-002); the Core IR only requires the node be present and carry a policy.
- *(Guarantee elaboration)* Elaboration annotates each result with `guarantee = meet(args.guarantee ⊕ prim.intrinsic_guarantee)` and composes `bound` per §4.7.

**Well-formedness invariants (the mechanical "no black boxes"; enforced by the linter, Foundation §5.8):**
- **WF1.** Every change of a value's `Repr` is a `Swap` node. *(no hidden conversion — SC-3)*
- **WF2.** Every `Swap` carries a `PolicyRef`. *(G2; ADR-006)*
- **WF3.** Every result with `guarantee != Exact` carries a `bound` consistent with M-I2/3/4. *(VR-3)*
- **WF4.** Every node is content-addressable: its hash is a pure function of its normalized structure and types (§4.6). *(FR-S2)*
- **WF5.** Lowering preserves `Meta` semantics (the lowering contract; *asserted* here, *enforced* in RFC-0004). *(FR-M5; dimensional persistence)*

### 4.6 Content-addressing

Definition identity follows Unison (Area 1):

```
hash(def) = H( normalize(structure(def)) ‖ types_with_repr(def) ‖ static_contract(def) )
```

- **Hashed (identity-bearing):** normalized node structure (α-renamed, position-independent), result/operand types *including `Repr`*, and the static contract (§4.4).
- **Not hashed (metadata):** human names, source spans, comments, formatting, and *all dynamic value metadata* (provenance, measured sparsity, realized bounds, `policy_used`). Names are stored separately as a `hash ↔ name` map; renaming does not change identity.

Consequence: two definitions differing only in representation paradigm have different hashes; a definition and its reformatting have the *same* hash (formatting is a projection, ADR-003; §4.8). Provenance references *are* content hashes, forming an acyclic derivation DAG (`Provenance ::= Root | Derived{ op: ContentHash, inputs: [ProvenanceRef] }`).

### 4.7 Guarantee lattice & bound composition

**Lattice (normative).** `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`, with `meet` = weakest. For an operation `f` with intrinsic guarantee `g_f` over inputs `v_1..v_n`:

```
guarantee(result) = meet(guarantee(v_1), …, guarantee(v_n), g_f)
```

This is monotone-downward: no operation can produce a result stronger than its weakest input or than its own intrinsic guarantee (the formal heart of ADR-001 at the value level).

**Bound composition (now concrete per ADR-010, Accepted).** Approximate `bound`s compose under three normative properties — **Soundness** (a true bound on deviation from the ideal-real spec), **Monotonicity** (never tighter than inputs justify), **Determinism** (identical inputs → identical composed bounds, so bounds are content-addressable). The arithmetic is supplied by **ADR-010's two kernels**: an `ErrorBound` kernel composing ε via **affine arithmetic** (Daisy/FloVer style), and a `ProbBound` kernel composing δ via the **union bound** (with apRHL-style couplings for relational certificates). The two do **not** share one algebra (a settled negative result, ADR-010/T0.1c); they meet at the shared certificate `{ε, δ, strength}`, and `strength` composes by `meet` as above. The one sanctioned cross-kernel inference is accuracy→probability (an `ErrorBound` may feed a `ProbBound`). Composed approximate results carry `Proven`/`Empirical` per the cited-theorem-with-checked-instantiation pattern — they no longer default to `Declared`.

### 4.8 Serialization & inspectability

- **Self-describing wire form (Arrow-grade, Area 4).** A serialized value is `[Repr descriptor] ‖ [Meta] ‖ [payload]`, faithfully round-trippable: `deserialize(serialize(v)) ≡ v` including `Meta`. This is the data model's analogue of Arrow's schema-travels-with-data property.
- **Canonical text dump (diffable, SC-4).** Every IR node has a canonical textual rendering; two structurally-identical nodes render identically (enabling lowering-stage diffs). Rendering is a projection and does not affect identity (§4.6).
- **Runtime query surface (consumed by the LSP, Foundation §5.8).** At minimum: `repr_of(v) → Repr`, `meta_of(v) → Meta`, `provenance_of(v) → Provenance`, `guarantee_of(v) → GuaranteeStrength`, `bound_of(v) → Option<Bound>`. These are the artifacts SC-5's dual-intelligibility channel delivers.

### 4.9 Extension points (hooks for dependent RFCs)

| Hook (defined here) | Consumed by | What that RFC supplies |
|---|---|---|
| `Swap` node + `Meta.policy_used` + `guarantee/bound` slots + the split-regime requirement | **RFC-0002** | `SwapCertificate` content (`Bijective(proof) \| Bounded{ε,…}`), legal `(R_src→target)` pairs, per-swap translation-validation (ADR-002, VR-4) |
| `VSA` paradigm kind + `Hypervector` type slot + `ModelId` registry + `CapacityBound`/`CrosstalkBound` | **RFC-0003** | `bind/unbind/bundle/permute/cleanup` as `Prim`s; MAP/BSC/HRR/FHRR/SBC implementations; how their bounds populate `Meta.bound` (ADR-008) |
| WF5 metadata-preservation contract + `PhysicalLayout` + `ExecutionMode` | **RFC-0004** | Core IR → Substrate IR → backend lowering; the "stable component" definition; interpreter-as-reference equivalence (ADR-009, NFR-7) |
| `PolicyRef` | **RFC-0005** | the `SelectionPolicy` language; how policies are reified, queried, and recorded (G2) |
| §4.7 bound-composition contract | **ADR-010** (Accepted) | two kernels (ε affine-arithmetic, δ union-bound/apRHL) meeting at the shared `{ε,δ,strength}` certificate |

## 5. Drawbacks

- **Verbosity.** Mandatory explicit `swap`s and `PolicyRef`s make representation changes wordy (mitigated by tooling/projections; the cost is intentional — it is the price of SC-3 auditability).
- **Closed paradigm kinds.** Adding a future substrate is a kernel change behind an RFC, not a library extension (deliberate, for `KC-3`; revisitable).
- **Two bound kernels, not one** (ADR-010 / T0.1c): ε and δ compose under different algebras meeting at a shared certificate — more surface than a single algebra, accepted as inherent and settled.
- **`reconstruction`/`ReconInfo`** is now specified in **RFC-0003 §6** (manifest = model+dim, content-addressed codebooks, compositional recipe, decoding procedure, bound certificate), and distinguishes indexed retrieval from true compositional reconstruction.

## 6. Rationale & alternatives

- **Why paradigm-in-the-type (vs. a single dynamic "tagged value" type)?** A dynamic tag would push paradigm errors to runtime and defeat static auditability (FR-M1, NFR-3). Units-of-measure (F#) shows compile-time representation tracking works; its erasure (Area 4/5) is the *anti*-pattern this RFC avoids via §4.4/WF5.
- **Why move packing to metadata (refining Foundation §5.2)?** So that lossless re-packing does not fork the type and break interoperability between dev (unpacked) and stable-AOT (packed) forms (ADR-009). Keeping `dtype` in the `Dense` type but packing in metadata draws the line at *semantic significance* (M-I5).
- **Why a four-point guarantee lattice (vs. a boolean exact/approximate)?** The survey is explicit that some VSA bounds are proven (Clarkson-Ubaru-Yang) and others only Gaussian-approximate (Frady-Sommer) (**G5**); collapsing these would force dishonesty. `Declared` is separated from `Empirical` so unverified user assertions can never masquerade as measured ones (VR-5).
- **Why content-address the static contract but not dynamic metadata?** Code identity must be stable across renames/reformatting (Unison) yet sensitive to contract changes; per-value runtime facts are not "the code" and would make identity non-deterministic.
- **Alternative considered — open paradigm registry in the kernel.** Rejected for now: maximal MLIR-style extensibility conflicts with a single-expert-auditable kernel (`KC-3`). Parameter registries give most of the practical extensibility without opening the kind set.

## 7. Prior art (from the survey; no new sources)

Apache Arrow — self-describing schema/metadata travelling with data, faithful round-trips (§4.8). Unison — content-addressed definition identity, names-as-metadata (§4.6). MLIR — typed, inspectable, multi-level IR; its open type system is the extensibility point this RFC deliberately constrains (§4.1). F# units-of-measure — compile-time representation tracking, whose *erasure* this RFC rejects (§4.4). Refinement types (Liquid Haskell, F*) — the basis for declared static bounds in the contract (§4.4) and the eventual bound checking. Rosa/Daisy + verified numerics — the ideal-real-spec-plus-certified-error pattern the bound model encodes, and the source of the composition arithmetic now concrete in ADR-010 (§4.7).

## 8. Unresolved questions

- ~~The concrete bound-composition arithmetic~~ → **resolved** by ADR-010 (two kernels; §4.7 updated).
- ~~Legal swap pairs and certificate content~~ → **resolved** in RFC-0002 (incl. `LosslessWithinRange` binary↔ternary, T2.1).
- ~~The full `ReconInfo` schema~~ → **resolved** in RFC-0003 §6.
- ~~Whether sparsity should be a static refinement~~ → **resolved**: declared class is a static refinement; observed sparsity stays in `Meta` (RFC-0003 §5, §4.4 here).
- **Still open:** the full term language (abstraction, recursion, modules, the VSA-submodule import mechanism) layered over §4.5 → a later RFC. One **confirming build** remains: the Liquid-Haskell `bundle` capacity-refinement instantiation (RFC-0003 §5) that ratifies the cited-theorem + checked-instantiation strategy.

## 9. Future possibilities

A fifth paradigm kind (e.g., a future native-ternary-hardware representation, or a residue/fractional-power encoding from the survey) could be added behind an RFC once the four-kind model is validated. The provenance DAG (§4.6) could support full W3C-PROV-style export (Area 4) for external audit. The self-describing wire form could become the interchange format between Mycelium and external systems (e.g., Arrow ↔ Mycelium bridges).

---

## Meta — changelog & maintenance

- **r0 (initial draft):** initial Core IR & metadata schema; refines Foundation §5.2; introduces the four-point guarantee lattice and honesty-propagation rule; fixes the bound-composition *contract* and defers its arithmetic to ADR-010.
- **r1 (solidified, this version):** **Accepted.** Packing moved from metadata to **schedule-staged** (§4.1, per DN-01 + T1.4); sparsity-as-static-refinement **resolved** (§4.4); §4.7 bound composition made **concrete** via ADR-010's two kernels (Accepted); §8 unresolved questions resolved with pointers. Remaining: the full term language, and the one confirming Liquid-Haskell `bundle` probe.
- **r2 (this revision):** **Accepted.** §4.3 — `BoundBasis` factored out to a required companion of *every* `Bound` (r1 attached it to `CapacityBound` only), per **ADR-011**, reconciling the grammar with invariants M-I2/M-I3/M-I4 and RFC-0002 §3; `NormKind` registry enumerated `L1|L2|Linf|Rel`. **Supersedes** the r1 §4.3 `Bound` grammar. Surfaced as OQ-3/OQ-4 during M-010 schema ratification (#5).
- Maintain as append-only with status transitions (Draft → Accepted → Superseded), mirroring the ADR discipline (Foundation Meta).
- On acceptance, add a one-line forward-pointer in Foundation §5.2 noting that RFC-0001 supersedes that sketch's packing placement, to prevent divergence.
- Re-validate §4.7 once ADR-010 is ratified; promote composed-result default from `Declared` to the foundation's actual composition rules.
