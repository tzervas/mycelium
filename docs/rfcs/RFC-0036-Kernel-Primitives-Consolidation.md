# RFC-0036 — Kernel & Primitives Consolidation (minimal + complete + frozen-toward-1.0)

| Field | Value |
|---|---|
| **RFC** | 0036 |
| **Status** | **Accepted** (2026-06-28) — **ratified by maintainer 2026-06-28 (in-session)**. Kernel position accepted: single frozen L0 kernel (Option A); 9/10 nodes irreducibly primitive; `FixGroup` (FLAG-B) still open — to be examined before freeze (derivability question, not blocking Accepted); zero new VSA/HDC primitives (`Repr::Vsa` + RFC-0003 submodule is the correct split). → Enacted once the FLAG-B question is resolved and the freeze mechanism (OQ-4) is implemented. Algorithm tags stay `Declared` (VR-5). Prior status chain (append-only): **Draft (updated 2026-06-28)** — proposed kernel position for maintainer ratification. The capture-only framing (2026-06-27) has been extended with a concrete proposed position on the three open questions: OQ-1/OQ-2 (primitive-vs-derived boundary), OQ-3 (one-kernel-vs-several), and the VSA/HDC kernel question. The freeze mechanism (OQ-4) and completeness criterion (OQ-5/OQ-6) are addressed in summary terms; the full answers live in DN-56. |
| **Type** | Architecture — kernel boundary, primitive taxonomy, 1.0 freeze strategy. |
| **Date** | June 27, 2026 |
| **Depends on** | KC-3 (small-auditable-kernel constraint — CLAUDE.md house rule #5; `docs/Mycelium_Project_Foundation.md`); DN-39 (`docs/notes/DN-39-Kernel-Promotion-Review-KC3.md` — ratified: zero promotions, boundary UNCHANGED); RFC-0001 (`docs/rfcs/RFC-0001-Core-IR-and-Metadata-Schema.md` — the Core IR the kernel grounds); RFC-0003 (`docs/rfcs/RFC-0003-VSA-Submodule-Boundary.md` — the VSA submodule boundary). |
| **Coupled with** | RFC-0001 (Core IR); RFC-0003 (VSA boundary); RFC-0007 (L1 kernel calculus); RFC-0032 (kernel prims for self-hosting enablement); ADR-033 (FieldSpec::Fn for dynamic dispatch — the last open kernel-touching primitive); DN-39 (the promotion review this RFC extends); DN-55 (`docs/notes/DN-55-Static-Specialization-The-Polymorphism-Model.md` — polymorphism zero-primitive); DN-56 (`docs/notes/DN-56-Kernel-Completeness-And-Freeze-Criterion.md` — the freeze gate this RFC feeds); the spore/phylum packaging model (ADR-013) if a multi-kernel split changes the phylum shape. |
| **Task** | `rsm` kickoff, F5. Deliverables: an RFC + research. |

> **Posture (transparency rule / VR-5 / G2).** All positions in §5 are **`Declared`-with-argument**
> — they are design recommendations grounded in the existing corpus (KC-3, DN-39, DN-55, DN-56,
> RFC-0003, RFC-0007, RFC-0032, ADR-033) and in the enumerated L0 node set + prim table read
> directly from the code (`crates/mycelium-core/src/node.rs`, `prim.rs`, `repr.rs`). None are
> `Proven` — no theorem establishes that the proposed set is minimal-and-complete in a
> machine-checked sense. The code enumeration is `Empirical` (read 2026-06-28; source is ground
> truth; no guarantee against a concurrent change). Tag strengths are held at their supportable
> basis throughout (VR-5); disagreement is flagged explicitly (G2).

---

## §1 The framing (maintainer's intent)

The kernel stays **minimal** — KC-3 (small-auditable-kernel), ratified by DN-39 with zero
promotions and the boundary unchanged. But "minimal" is not "weak": the maintainer's model is:

> **All primitives live in the kernel, and the kernel is the most heavily tested, benchmarked,
> and chaos-engineered artifact in the project — every condition and error mapped and handled —
> so it can be frozen, locked, and pinned as the 1.0 kernel, rarely if ever touched again.**

Small *because targeted*: only primitives, and only doing in primitives what should be done in
primitives; everything else builds on top. The kernel earns its freeze by being exhaustively
verified, not by being minimal in a way that leaves gaps.

**This RFC investigates two things:**

1. **The "what must be a primitive vs built on top" boundary review.** Given the freeze goal,
   the boundary between "belongs in the kernel" and "belongs in the stdlib/phylum layer" must be
   reviewed and settled. The DN-39 bar (four conjunctive clauses — foundational · unverifiable-
   from-outside · net-trust-reducing · small-and-auditable) is the existing tool for this
   boundary; this RFC's boundary review applies it systematically.

2. **One kernel, or several?** Open structural question: is the right shape a single `mycelium`
   kernel, or a family — e.g. a `mycelium` kernel + a binary/ternary kernel + an embedding kernel
   - a VSA/HDC kernel? The best approach is **TBD** (`Declared`); this RFC captures the question
   and the considerations, decides nothing.

## §2 Relationship to existing docs

- **KC-3** (CLAUDE.md house rule #5; `docs/Mycelium_Project_Foundation.md`) is the primary
  constraint: the kernel must remain small and auditable. This RFC works toward a version of KC-3
  that is **proven, not merely aspired to** — a kernel small enough to be exhaustively verified.
- **DN-39** (`docs/notes/DN-39-Kernel-Promotion-Review-KC3.md`, Accepted 2026-06-26) is the
  immediately prior boundary review: it reviewed whether any non-kernel functionality should be
  promoted in, and ratified zero promotions. This RFC extends that work in the outward direction:
  given the freeze goal, what *must* remain in the kernel, and what is better moved out?
- **RFC-0001** (`docs/rfcs/RFC-0001-Core-IR-and-Metadata-Schema.md`) defines the Core IR and
  content-addressing — the substrate the kernel grounds. Any kernel consolidation must preserve
  RFC-0001's invariants.
- **RFC-0003** (`docs/rfcs/RFC-0003-VSA-Submodule-Boundary.md`) draws the VSA submodule
  boundary. The multi-kernel question (OQ-3) directly affects whether the VSA machinery is a
  separate kernel or a phylum.
- **RFC-0007** (`docs/rfcs/RFC-0007-L1-Kernel-Calculus.md`) defines the L1 kernel calculus —
  the ten-node budget, typing, totality. This RFC's boundary review includes L1 as part of the
  kernel surface.
- **RFC-0032** (`docs/rfcs/RFC-0032-Kernel-Self-Hosting-Enablement-Surface.md`, Accepted
  2026-06-23) ratified the comparison prims (D1), binary arithmetic (D2), `Repr::Seq` (D3),
  `Repr::Bytes` (D4), and the width-cast prim (DN-41) as in-`core`-1.0.0 additions.
- **DN-55** (`docs/notes/DN-55-Static-Specialization-The-Polymorphism-Model.md`, Accepted
  2026-06-27) records the **zero-kernel-primitive consequence** of static specialization:
  polymorphism — type parameters, width generics, trait-bounded generics — erases entirely in
  the frontend and adds no L0 node and no kernel prim.
- **DN-56** (`docs/notes/DN-56-Kernel-Completeness-And-Freeze-Criterion.md`, Accepted
  2026-06-27) is the freeze-criterion framework this RFC feeds. It states the five-condition
  freeze gate; this RFC closes OQ-1/OQ-2/OQ-3 as inputs to DN-56 condition #3 (primitive set
  closed) and condition #5 (KC-3 completeness review).
- **ADR-033** (`docs/adr/ADR-033-Abstract-Function-Field-For-Dynamic-Dispatch.md`, Accepted
  2026-06-27) is the **last open kernel-touching primitive**: `FieldSpec::Fn` for dynamic
  dispatch. Its FLAG-1 (arity-only hashing soundness) is an explicit pre-`Enacted` gate and a
  DN-56 condition-#3 dependency.
- **DN-34** (`docs/notes/DN-34-Rust-to-Mycelium-Transpiler-Strategy.md`) and the
  **self-hosting arc** (RFC-0031, RFC-0032) are downstream of a frozen kernel: self-hosting
  becomes tractable once the kernel is stable and pinned.

## §3 Open Questions

**OQ-1.** What is the complete set of primitives that "belong in the kernel" by the DN-39
four-clause bar? Is the current kernel surface already complete for 1.0, or are there gaps?
(The DN-39 review found zero *promotions* needed, but did not audit whether existing kernel
items should be *demoted*.)

**OQ-2.** Is there anything currently in the kernel that fails the DN-39 bar's conjunctive
clauses and should be moved to the stdlib or a phylum instead? The freeze goal makes this
question more urgent: anything that *should not* be frozen must be identified before the pin.

**OQ-3.** One kernel or several? The candidate split (maintainer's framing, `Declared`):
a `mycelium` kernel + a binary/ternary kernel + an embedding kernel + a VSA/HDC kernel.
What are the arguments for and against the split?
- *For:* each kernel can be frozen and verified independently; the VSA/HDC surface has
  different stability guarantees than the core language kernel; separate kernels can be
  combined compositionally.
- *Against:* more kernels = more surface to test and maintain; the split may violate KC-3
  by distributing the trusted base across multiple artifacts; the interaction between kernels
  becomes a new trust surface.
The honest answer is unknown; the research must explore this.

**OQ-4.** What does "frozen, locked, and pinned" mean concretely? A semantic version pin? A
content-hash commitment (RFC-0001 §4.6)? A policy commitment (no new primitives after 1.0
without a superseding RFC)? All three? The mechanism is open.

**OQ-5.** What does "most heavily tested, benchmarked, and chaos-engineered" mean as a
completeness criterion? The goal is "every condition and error mapped and handled" — how is
that criterion discharged? Is it a property-test exhaustion requirement, a chaos-test
specification, or something else?

**OQ-6.** Does the freeze goal interact with the RFC-0034 `fast` / `certified` mode split?
The kernel is the substrate for both modes; freezing it must not inadvertently constrain
future certification machinery. Does the freeze apply to the kernel's interface, its
implementation, or both?

**OQ-7.** What is the sequencing relative to the self-hosting arc (RFC-0031/RFC-0032)?
The kernel freeze is a prerequisite for stable self-hosting (you can't bootstrap reliably on
a moving target). Does this RFC need to land (at least as Accepted) before RFC-0032's
self-hosting work begins in earnest?

## §4 Definition of Done (gate to move Draft → Proposed)

- A **research record** is produced covering: (a) the multi-kernel question (OQ-3) with at
  least one precedent from a language with compositional kernel architecture; (b) the
  completeness criterion (OQ-5) with a grounded definition of what "every condition and error
  mapped" means for the Mycelium kernel.
- OQ-1 and OQ-2 are answered: the current kernel surface is audited against the DN-39 bar,
  with any demotion candidates identified (even if none are found — that is itself the result).
- OQ-3 is answered with a grounded proposal (one kernel or a specified split), with honest
  labels on which claims are `Proven`/`Empirical`/`Declared`.
- The freeze mechanism (OQ-4) is specified.
- The maintainer ratifies the kernel count and freeze mechanism before the note moves to
  Proposed.

---

## §5 Proposed kernel position (2026-06-28 — `Declared`-with-argument; for maintainer ratification)

> **Scope.** This section proposes a concrete position on OQ-1, OQ-2, and OQ-3. It is the
> first substantive design pass, not a final answer. All positions are `Declared` —
> design recommendations grounded in the corpus; none are `Proven`. The maintainer must ratify
> or redirect before this RFC moves to Proposed. OQ-4 (freeze mechanism) is addressed in §5.4;
> OQ-5/OQ-6/OQ-7 are addressed in §5.5.

### §5.1 The actual L0 node set (grounded enumeration)

The complete node grammar as implemented in `crates/mycelium-core/src/node.rs` (read
2026-06-28, `Empirical` — source is ground truth):

**L0 baseline nodes (RFC-0001 §4.5, present from the earliest Core IR):**

| Node | Description | Classification |
|---|---|---|
| `Const` | A literal constant value (repr + payload + Meta). | **Irreducibly primitive.** A ground value; there is nothing below it to lower to. |
| `Var` | A variable reference (a name). | **Irreducibly primitive.** Variable lookup is fundamental to any term language. |
| `Let` | A let-binding (`let x = e₁ in e₂`). | **Irreducibly primitive** (see FLAG-A for the Let/Lam redundancy question). Without a binding form, functions cannot name intermediate results; the current design keeps `Let` for interpreter clarity. |
| `Op` | A paradigm-specific primitive application. | **Irreducibly primitive.** The gateway to all arithmetic and logic — there is no lower-level node to call out to. |
| `Swap` | The only node that changes a value's `Repr`; always carries a `PolicyRef`. | **Irreducibly primitive.** S1 (the never-silent swap rule — WF1/WF2) is enforced *by construction*: `Swap` is the single gated Repr-change node. Moving this to the stdlib would destroy the structural enforcement of WF1/WF2. |

**L1 additions (RFC-0007 §4.1, the five higher-order/data nodes):**

| Node | Description | Classification |
|---|---|---|
| `Lam` | Lambda abstraction: binds one `param` in `body`. First-order in v0 (closed — no heap closures). | **Irreducibly primitive** for a functional language; functions-as-values require a lambda node. |
| `App` | Application: apply `func` to `arg`, call-by-value. | **Irreducibly primitive** (dual of `Lam`). |
| `Fix` | General self-recursion: `Fix{name, body}` binds `name` to itself in `body`. | **Irreducibly primitive** for a general-recursive language. Termination is not enforced in the kernel (the totality checker is untrusted-for-semantics); removing `Fix` would prohibit recursion. |
| `FixGroup` | Mutual recursion group (RFC-0001 r5, R7-Q3). | **Candidate for examination** — see FLAG-B. Operationally distinct from `Fix` but potentially derivable from it via a pairing/CPS encoding over `Construct`. The cost of keeping it is a slightly larger audit surface; the benefit is direct, transparent mutual recursion in the kernel (G2 — no encoding to obscure). `Declared` — the derivability must be verified before a final position is taken. |
| `Construct` | Saturated constructor application: builds a data value (WF6 saturation). | **Irreducibly primitive** given the registry model. Data types live in the registry (`Σ`), not in the term grammar; the kernel must have a way to construct them. |
| `Match` | Flat pattern match: one scrutinee, single-level alternatives, at most one default (WF7). | **Irreducibly primitive.** Destructuring is the dual of `Construct`; the flat/checked invariant is enforced above the kernel (Maranget, RFC-0011 §4.4). |

**Summary.** Of the ten L0/L1 nodes, nine are unambiguously irreducibly primitive by the DN-39
bar. `FixGroup` is the one node to examine for derivability before freeze (FLAG-B). No
promotions are warranted (consistent with DN-39's zero-promotion result in the inward direction).

### §5.2 The actual kernel prim set (grounded enumeration)

The complete default prim table as implemented in `crates/mycelium-core/src/prim.rs`
`PrimTable::builtins()` (read 2026-06-28, `Empirical`):

**Identity:**

| Prim | Signature | Classification |
|---|---|---|
| `core.id` | `Any → Any` (paradigm-polymorphic) | **Irreducibly primitive.** Structural passthrough needed for uniform prim-dispatch; keeping it in the prim table preserves the auditability of every value path through `Op`. |

**Elementwise binary logic:**

| Prim | Signature | Classification |
|---|---|---|
| `bit.not` | `Binary → Binary` | **Irreducibly primitive.** Boolean negation; cannot be derived without circularity. |
| `bit.and` | `Binary × Binary → Binary` | **Irreducibly primitive.** Conjunction; foundational. |
| `bit.or` | `Binary × Binary → Binary` | **Candidate for consolidation** (`bit.or(a,b)` = `bit.not(bit.and(bit.not(a), bit.not(b)))` — De Morgan). Recommend **keep** (`Declared`): the derived encoding introduces elaborator complexity harder to audit than a single named prim; every hardware ISA exposes OR directly. |
| `bit.xor` | `Binary × Binary → Binary` | **Irreducibly primitive.** Not derivable from `not`/`and` without extra infrastructure; foundational for binary arithmetic and cryptographic primitives. |

**Fixed-width balanced-ternary arithmetic:**

| Prim | Signature | Classification |
|---|---|---|
| `trit.neg` | `Ternary → Ternary` | **Irreducibly primitive.** Ternary negation (balanced `{−1,0,+1}`); no ternary De Morgan. |
| `trit.add` | `Ternary × Ternary → Ternary` | **Irreducibly primitive.** Basic ternary addition; cannot be derived from `neg` alone. |
| `trit.sub` | `Ternary × Ternary → Ternary` | **Candidate for consolidation** (`trit.sub(a,b)` = `trit.add(a, trit.neg(b))`). Recommend **keep** (`Declared`): same auditability-over-derivability argument as `bit.or`. |
| `trit.mul` | `Ternary × Ternary → Ternary` | **Irreducibly primitive.** Ternary multiplication is not derivable from add/neg without repeated-addition recursion through `Fix`. |

**Comparison prims (RFC-0032 D1):**

| Prim | Signature | Width | Classification |
|---|---|---|---|
| `cmp.eq` | `Any × Any → Binary{1}` | `Collapse` | **Irreducibly primitive.** Equality across paradigms; reduce-to-Bool. Foundational for conditional constructs. |
| `cmp.lt` | `Any × Any → Binary{1}` | `Collapse` | **Irreducibly primitive.** Total ordering; required for `Ord`, sort, and range-checked access. |

**Binary arithmetic (RFC-0032 D2):**

| Prim | Signature | Classification |
|---|---|---|
| `bit.add` | `Binary × Binary → Binary` | **Irreducibly primitive.** Fixed-width binary addition (never-silent on overflow — explicit error at the interpreter layer). |
| `bit.sub` | `Binary × Binary → Binary` | **Candidate for consolidation** (`bit.sub(a,b)` = `bit.add(a, two_complement_neg(b))`; two's-complement negation is `bit.add(bit.not(b), 1)` — but requires a literal `1`, a bootstrapping dependency). Recommend **keep** (`Declared`): same argument as `trit.sub`; every ISA exposes subtract directly. |

**Width-cast prim (DN-41):**

| Prim | Signature | Classification |
|---|---|---|
| `bit.width_cast` | `Binary × Binary → Binary` | **Irreducibly primitive.** The only mechanism to change a `Binary{n}` width in the kernel. Without this prim, width-generic programs (DN-42) cannot be lowered to concrete widths. See FLAG-C for the `WidthRel` model gap. |

**Sequence prims (RFC-0032 D3):**

| Prim | Signature | Classification |
|---|---|---|
| `seq.len` | `Any → Binary` | **Irreducibly primitive.** The only way to query a `Repr::Seq`'s length. |
| `seq.get` | `Any × Binary → Any` | **Irreducibly primitive.** Indexed element access; the primary operation over `Repr::Seq`. |

**Byte-string prims (RFC-0032 D4):**

| Prim | Signature | Classification |
|---|---|---|
| `bytes.len` | `Any → Binary` | **Irreducibly primitive.** Length of a `Repr::Bytes` value. |
| `bytes.get` | `Any × Binary → Binary` | **Irreducibly primitive.** Byte at index (a `Binary{8}`); needed for UTF-8 decode and general byte access. |
| `bytes.slice` | `Any × Binary × Binary → Any` | **Candidate for consolidation** — derivable from `len`/`get` + `Fix` (loop constructing a new `Repr::Bytes`). Recommend **keep** (`Declared`): a kernel prim for slice is O(1)-allocation-visible and audit-safe; the `Fix`-based derivation obscures allocation behavior and introduces recursion into every use. |
| `bytes.concat` | `Any × Any → Any` | **Candidate for consolidation** — same argument as `bytes.slice`. Recommend **keep** (`Declared`). |

**Value representations (kernel `Repr` variants — `crates/mycelium-core/src/repr.rs`):**

The six `Repr` variants are not prims but are kernel data types whose presence in the frozen
set must be justified:

| Repr | Classification |
|---|---|
| `Binary{width}` | **Irreducibly primitive.** Primary bit-typed paradigm; foundational. |
| `Ternary{trits}` | **Irreducibly primitive.** Balanced-ternary paradigm; foundational. |
| `Seq{elem, len}` | **Irreducibly primitive** (in-1.0.0 per RFC-0032 D3). `seq.len`/`seq.get` bottom out on it. |
| `Bytes` | **Irreducibly primitive** (in-1.0.0 per RFC-0032 D4). The four `bytes.*` prims bottom out on it. |
| `Dense{dim, dtype}` | **In kernel; no demotion recommended** — see FLAG-D. No kernel arithmetic prims operate on `Dense` today; its algebra is above the kernel. It participates as a value-type descriptor for content-addressing. The `ScalarKind` enum (F16, Bf16, F32, F64) is small and stable (standardized formats). |
| `Vsa{model, dim, sparsity}` | **In kernel; no demotion recommended** — see FLAG-E. Same reasoning as `Dense`. The `model: String` is an open registry reference — the kernel is already extensible w.r.t. VSA models without a kernel change. |

### §5.3 The VSA/HDC kernel question

**Position (Declared):** The VSA/HDC value model requires **no new kernel primitives** in the
v0 kernel. The existing architecture already achieves the right split:

1. **`Repr::Vsa` is in the kernel** as a value-type descriptor — it is needed for content-
   addressing (a `Const` value carrying a `Vsa` repr is content-addressed over its `Repr`
   descriptor, which must be in the trusted base). The kernel can create, content-address,
   and swap *to/from* `Vsa` values without understanding their algebra.

2. **VSA algebra (bind, bundle, permute, similarity) is in the submodule** (RFC-0003 §2:
   "the submodule supplies the algebra"). These operations are registered in a `VsaModel`
   interface, not as kernel prims. They carry per-model-per-operation guarantee tags
   (`Proven`/`Empirical` — RFC-0003 §4) that are *not* `Exact` (unlike all kernel prims,
   which are `Exact`). Promoting VSA algebra into the kernel would violate KC-3 (the algebra
   is large) and the honesty rule (the kernel's `Exact` tag cannot absorb `Empirical` — VR-5).

3. **DN-55's zero-primitive consequence** applies here: static-specialization erases all
   VSA type-parameter usage before reaching L0. There is no "polymorphism-over-VSA-models"
   requiring a kernel prim — model-specific dispatch is the submodule's concern, above L0.

4. **The one exception** is `Repr::Vsa` itself (point 1). The multi-kernel architecture
   (OQ-3 Option B) would allow moving `Repr::Vsa` out of the kernel `Repr` enum into a
   registered repr-extension. §5.4 recommends against this split (FLAG-E).

**Conclusion (Declared):** The VSA/HDC value model is correctly placed today. No new kernel
primitives are needed for VSA. The `Repr::Vsa` descriptor belongs in the frozen kernel.

### §5.4 One-kernel-vs-several (OQ-3)

#### Option A: Single frozen L0 kernel (recommended — `Declared`)

Keep the single `mycelium-core` kernel with its current structure. The kernel contains:
- The ten-node L0/L1 calculus (RFC-0007)
- The closed `Repr` enum (Binary, Ternary, Dense, Vsa, Seq, Bytes)
- The closed prim table (core.id, bit.*, trit.*, cmp.*, seq.*, bytes.*)
- The data registry (`Σ`) and constructor references
- The prim table (`Π`) and prim declarations
- The guarantee lattice, content-addressing, and metadata model

Everything else is a frontend lowering, a stdlib phylum, or a verified submodule.

**Arguments for (all `Declared`):**

- **Smallest trusted base.** A single kernel has one audit surface, one test suite, one
  freeze event. KC-3's "small and auditable" is satisfied most cleanly by one auditable kernel,
  not N of them.
- **The interaction problem is a trust leak.** In a multi-kernel world, the cross-kernel
  protocol — "which kernel's prim can appear in which kernel's `Op` node?" — becomes a new
  trust surface harder to audit than the combined single surface. The "N independent kernels,
  N independent freezes" claim breaks down once kernels interact.
- **The existing `Repr` enum is small.** Six variants. `Dense` and `Vsa` are type *descriptors*
  — they participate in content-addressing but have no kernel arithmetic. Their presence in a
  single `Repr` enum is not a KC-3 violation; it is the correct factoring (type descriptor in
  the kernel, algebra in the submodule). The enum does not need to grow for any planned feature.
- **DN-55: polymorphism is zero-primitive.** There is no "polymorphism-over-kernels" concern
  — DN-55 established that all polymorphism erases before reaching L0. A VSA kernel would
  not gain anything from polymorphism that the existing `Repr::Vsa` + submodule model does
  not already provide.
- **Precedent: CakeML, Lean 4, GHC Core.** These languages freeze a single trusted kernel
  (a small calculus + a fixed primitive set) and layer everything above it. None split into
  domain-specific sub-kernels. The compositional-multi-kernel model has no mature precedent
  in production language systems, to this author's knowledge (`Declared` — no systematic
  literature search was conducted; the §4 Definition of Done calls for one).

**Arguments against / for the split (all `Declared`):**

- **`Dense`/`Vsa` have no kernel arithmetic.** Their algebra is entirely above the kernel.
  A split that moves `Repr::Dense` and `Repr::Vsa` into a repr-extension registry would make
  the core kernel (Binary, Ternary, Seq, Bytes) freeze-eligible earlier.
- **`ScalarKind` lock-in.** The `Dense` variant's `ScalarKind` enum (F16, Bf16, F32, F64)
  would be frozen. New scalar formats (e.g., FP8) would require a `core 2.0.0` event. A
  repr-extension registry would allow new scalar kinds post-freeze.
- **VSA descriptor extensibility.** The `Vsa` `model: String` is already open, but the
  descriptor shape (model + dim + sparsity) would be frozen. A repr-extension registry could
  allow new VSA descriptor fields.

#### Option B: Split `Dense`/`Vsa` to repr-extension registry (not recommended at this time)

Move `Repr::Dense` and `Repr::Vsa` out of the closed kernel `Repr` enum into a separately-
registered repr-extension mechanism. The core kernel's `Repr` would be:
`Binary | Ternary | Seq | Bytes`. `Dense` and `Vsa` would be registered as extensions, with
their type descriptors and content-addressing rules in a verified (not trusted) submodule.

**Assessment (Declared):** Architecturally coherent but unfavourable cost-vs-benefit at this
stage. The `Dense`/`Vsa` descriptors are small (each ~30 LOC in `repr.rs`), their content-
addressing is already implemented and tested, and RFC-0003's submodule boundary already cleanly
separates the algebra. Moving the descriptors out introduces a new trust boundary (the extension
registry itself must be specified, verified, and frozen) that does not simplify the freeze
criterion — it replaces one known surface with two (the reduced kernel + the extension registry
protocol).

#### Recommendation (Declared)

**Recommend Option A: a single frozen L0 kernel.** The six `Repr` variants, ten nodes, and
~20 prims are well within KC-3's "small and auditable" bound. The total trusted base is small
enough to audit exhaustively (which is the point — DN-56 §5 condition #5). The interaction-
problem argument is load-bearing: cross-kernel trust surfaces compound audit cost. The
VSA/HDC algebra is already correctly placed in the submodule; no structural split is required.

**The open structural question after Option A** is FLAG-D/FLAG-E (§5.6): whether
`Repr::Dense` and `Repr::Vsa` should be promotable to a registered repr-extension *post-freeze*,
enabling new descriptor variants after 1.0 without a `core 2.0.0`. The recommended answer
(`Declared`): **no** — adding new `Repr` variants after 1.0 *is* a `core 2.0.0` event
(supersession, house rule #3). The ScalarKind variants are standardized (IEEE-754, BF16); the
VSA descriptor shape is stable. The extensibility-via-registry design, while appealing,
introduces a new trust boundary that itself needs specification, verification, and freeze.

### §5.5 Remaining open questions (partially addressed)

**OQ-4 (freeze mechanism — `Declared`).** DN-56 §5 specifies the five-condition freeze gate.
The freeze *event* mechanism: all three of the following, in combination.
- **Semantic version pin:** `core 1.0.0`. Necessary but insufficient (does not prevent silent
  behavioral changes to internals).
- **Content-hash commitment:** RFC-0001 §4.6 content-addressing provides a structural hash
  over each `Const`/`DataDecl`/`PrimDecl`/`CtorRef`. The `PrimTable::builtins()` set, `Repr`
  enum variants, and `Node` enum variants produce a deterministic hash over the kernel's
  structure; that hash is committed in the `core 1.0.0` release note and checked on each
  subsequent build. Any change to the kernel surface changes the hash — a tamper-evident,
  never-silent (G2) freeze seal.
- **Policy commitment:** An RFC (or supersession of this RFC) declares that no new `Node`
  variant, `Repr` variant, or builtin prim is added after `core 1.0.0` without superseding
  this RFC and cutting a `core 2.0.0`. This is the append-only commitment (house rule #3
  applied to the kernel surface).

**OQ-5/OQ-6 (completeness criterion + RFC-0034 interaction — `Declared`).** DN-56 addresses
OQ-5 directly (the five-condition freeze gate). OQ-6: the freeze applies to the kernel's
**interface and semantics**, not to the `CertMode` policy layer above it. The `CertMode` enum
(`crates/mycelium-core/src/cert_mode.rs`) is a separate concern from the node grammar and prim
table; freezing the kernel does not constrain adding new certification modes via `CertMode`
(which is a policy enum, not a kernel node). This interpretation is `Declared`; a full answer
requires RFC-0034's authors to confirm no kernel-level change is anticipated for the `certified`
mode.

**OQ-7 (sequencing with self-hosting — `Declared`).** RFC-0032 is already Accepted and in
flight (E19-1). This RFC should reach Proposed (maintainer ratification) before the E19-1
implementation wave completes, so that any late-breaking primitive-set corrections are caught
before the kernel implementation is committed. It need not be Accepted before E19-1 starts —
the positions taken here do not contradict RFC-0032. It should be Accepted before the freeze
event.

### §5.6 Flags for maintainer review

**FLAG-A — Let/Lam redundancy.** The node set includes both `Let` and `Lam`/`App`. A `Let`
is beta-equivalent to `App(Lam(x, body), bound)`. Keeping both is a deliberate design choice
(direct naming is more readable and more efficient in the interpreter) but means the kernel has
ten structurally distinct nodes rather than nine. The author's position (`Declared`): keep both
— the auditability benefit of the simpler interpreter path outweighs theoretical parsimony.

**FLAG-B — FixGroup as a derived form.** `FixGroup` (mutual recursion) is potentially
derivable from `Fix` + `Construct` via the standard pairing encoding (two mutually recursive
functions `f`/`g` encoded as `Fix(fg, Construct(pair, Lam(x, ...), Lam(y, ...)))`). If this
derivation is verified (a correctness argument, not merely asserted), `FixGroup` could be
removed from the kernel node set, reducing the budget to nine nodes. The cost: an encoding
overhead in the elaborator and a G2 concern (the encoding hides mutual recursion behind a data
structure, making L0 terms harder to read in `reveal`). The benefit: a simpler kernel. The
author's position (`Declared`): **flag for maintainer decision.** The derivation is non-trivial
to verify; removing a node before ratification is less disruptive than after. Recommend: a
brief proof sketch either way, then close before moving to Proposed.

**FLAG-C — WidthRel model gap for `bit.width_cast`.** The `WidthRel` enum (`Uniform`/
`Collapse`) cannot express "result width = second operand's (witness) width" — the correct
typing for `bit.width_cast`. The prim is registered as `Uniform` (a known approximation,
documented in-code). The real constraint is enforced by the interpreter prim and the L1
checker. This means the `PrimTable` does not fully encode `bit.width_cast`'s signature —
an auditability gap (G2). Author's position (`Declared`): add `WidthRel::Cast` before freeze
— an un-encodable constraint in the trusted prim table is a G2 violation.

**FLAG-D — `Repr::Dense` in the frozen kernel.** `Dense{dim, dtype}` has no kernel arithmetic
prims. The `ScalarKind` enum (F16, Bf16, F32, F64) would be frozen. Author's recommendation
(`Declared`): keep in the kernel — the enum is small, variants are stable (IEEE-754 and BF16
are standardized), and moving it out creates more complexity than it removes (see Option B
analysis). A new scalar kind after 1.0 is a `core 2.0.0` event.

**FLAG-E — `Repr::Vsa` in the frozen kernel.** `Vsa{model, dim, sparsity}` has no kernel
arithmetic prims. The `model: String` is an open registry reference — the kernel is already
extensible w.r.t. VSA models. Author's recommendation (`Declared`): keep in the kernel. The
descriptor shape is stable and unlikely to require post-freeze changes.

---

## §6 Summary of positions for ratification

| Question | Position | Tag |
|---|---|---|
| **OQ-1: Kernel surface complete for 1.0?** | Yes — no primitive gaps found. The current L0 node set + prim table + Repr variants are the correct frozen set, subject to the FLAGs. | `Declared` |
| **OQ-2: Anything to demote?** | No demotion candidates among core nodes/prims. `bit.or`, `trit.sub`, `bit.sub`, `bytes.slice`, `bytes.concat` are derivable but kept on auditability grounds. `FixGroup` is the one node to examine (FLAG-B). `Dense`/`Vsa` Reprs stay (FLAGs D/E). | `Declared` |
| **VSA/HDC kernel question** | No new kernel primitives required for VSA. `Repr::Vsa` + RFC-0003 submodule is the correct split. | `Declared` |
| **OQ-3: One kernel or several?** | Single frozen L0 kernel (Option A). | `Declared` |
| **OQ-4: Freeze mechanism** | Version pin + content-hash commitment + policy commitment (all three). | `Declared` |
| **FLAG-A: Let/Lam redundancy** | Keep both — interpreter clarity outweighs parsimony. | `Declared` |
| **FLAG-B: FixGroup derivability** | Verify before freeze; maintainer decision required. | Open |
| **FLAG-C: WidthRel::Cast gap** | Add `WidthRel::Cast` before freeze (G2 — un-encodable constraint). | `Declared` |
| **FLAG-D: Dense in frozen kernel** | Keep — ScalarKind variants are standardized; enum is small. | `Declared` |
| **FLAG-E: Vsa in frozen kernel** | Keep — model field is already open; descriptor shape is stable. | `Declared` |

**Do not move to Proposed until:** the maintainer ratifies or redirects these positions, and
FLAG-B (FixGroup derivability) is resolved by either a brief proof sketch or a "keep, not
pursued" decision.

**FLAG to parent (orchestrator/maintainer) — changes outside this RFC's file ownership:**

- **DN-56 §7 / condition #5:** Once this RFC is ratified, DN-56's condition #5 (KC-3
  completeness review) can be partially discharged by the OQ-1/OQ-2 primitive-set audit above.
  The maintainer should update DN-56 §7 to reflect this when ratifying.
- **CHANGELOG.md:** A changelog entry for this RFC update belongs in `CHANGELOG.md` (owned
  by the orchestrator/maintainer, not this RFC).
- **docs/Doc-Index.md:** No doc-index update required for a status-unchanged RFC update.

---

> **Provenance.** §1–§4 capture the maintainer's `rsm`-kickoff F5 workstream intent (2026-06-27;
> unchanged). §5–§6 are the proposed kernel position, authored 2026-06-28. KC-3 and DN-39 are the
> grounding basis for the boundary constraints; RFC-0001/0003/0007/0032/ADR-033/DN-55/DN-56 are
> the primary references for the current kernel surface. The node-set and prim-table enumerations
> are grounded in `crates/mycelium-core/src/node.rs`, `prim.rs`, and `repr.rs` (read 2026-06-28,
> `Empirical` — source is ground truth). All design positions are `Declared`; the maintainer
> ratifies or redirects before this RFC moves to Proposed.

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-27 | **Draft** | Initial capture (`rsm` kickoff, F5 / RFC-0036). Records the maintainer's kernel-consolidation and freeze intent, the one-vs-many kernel open question, and the "what must be a primitive" boundary review. KC-3 and DN-39 cited as the grounding basis. All design decisions open. Implements nothing. |
| 2026-06-28 | **Draft** (updated) | Proposed kernel position added (§5–§6). Enumerated the actual L0 node set (`node.rs`, ten nodes) and prim table (`prim.rs`, ~20 prims + six `Repr` variants in `repr.rs`) from source (`Empirical`, 2026-06-28). Classified each: nine nodes irreducibly primitive; `FixGroup` flagged for derivability check (FLAG-B). All kernel prims recommended as irreducible or kept on auditability-over-derivability grounds (`bit.or`, `trit.sub`, `bit.sub`, `bytes.slice`, `bytes.concat`). VSA/HDC kernel question closed: zero new kernel primitives required; `Repr::Vsa` + RFC-0003 submodule is the correct split. OQ-3 position: single frozen L0 kernel (Option A recommended; Option B — split Dense/Vsa to repr-extension registry — assessed unfavourable). OQ-4: version pin + content-hash + policy commitment (all three). OQ-5/OQ-6: deferred to DN-56; RFC-0034 `CertMode` is above-kernel. OQ-7: this RFC should be Accepted before freeze, not before E19-1 starts. Five flags (FLAG-A through FLAG-E). Status remains **Draft** — maintainer ratifies or redirects §5 before Proposed. Grounded in DN-55, DN-56, RFC-0003, RFC-0007, RFC-0032, ADR-033. |
| 2026-06-28 | **Accepted** (ratified by maintainer, in-session) | **Kernel position ratified.** Single frozen L0 kernel (Option A) accepted; 9/10 nodes irreducibly primitive; `FixGroup` FLAG-B stays open (derivability to be verified before freeze — not blocking Accepted). Zero new VSA/HDC primitives. OQ-4 freeze mechanism (version pin + content-hash + policy). → Enacted gated on FLAG-B resolution + freeze mechanism implementation. Tags stay `Declared` (VR-5). |
