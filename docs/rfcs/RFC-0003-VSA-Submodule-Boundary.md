# RFC-0003 — VSA Submodule Boundary

| Field | Value |
|---|---|
| **RFC** | 0003 |
| **Status** | **Accepted** (solidified from the research pass) |
| **Type** | Foundational / normative |
| **Date** | June 08, 2026 |
| **Depends on** | RFC-0001 (`VSA` Repr kind, `Hypervector` slot, `ModelId` registry, `CapacityBound`/`CrosstalkBound`, guarantee lattice); ADR-008 (VSA optional submodule); ADR-010 (bound kernels); Research Findings **T0.2**, **T1.2**, **T1.3**, **T2.2**, **T2.6** |
| **Coupled with** | RFC-0001 §4.3/§4.5 (sparsity refinement feeds back, now resolved) |

## 1. Scope
The kernel↔submodule boundary; the per-model interface; the **per-model × per-operation guarantee-tag matrix** (T1.2); the **sparsity-as-static-refinement** decision (T1.3); and the **reconstruction manifest** schema (T2.2).

## 2. Boundary (thin kernel)
Kernel carries only: the `VSA` Repr kind, the `Hypervector` type slot, its metadata fields, the swap machinery targeting it, and the `ModelId` registry hook. The submodule supplies the algebra. A kernel built without the submodule type-checks programs that *mention* hypervectors but offers no operations (NFR-6, KC-3; RFC-0001 §5.7).

## 3. Per-model interface (`VsaModel`)
A composition-style trait each registered model implements, supplying: `bind`/`unbind` (+ self-inverse flag), `bundle` (+ capacity-bound derivation), `permute`, `similarity`, clean-up integration, and — per operation — the **guarantee tag + basis** (`Proven` only where a cited theorem applies; else `Empirical`).

## 4. Guarantee-tag matrix (T1.2) — normative
Honest tags per the literature (proven = non-asymptotic concentration bounds; empirical = Gaussian/asymptotic):

| Model | bind / unbind | bundle (superposition) | permute |
|---|---|---|---|
| **MAP-I** (additive bundle, mult. bind) | self-inverse, **Exact** (algebraic) | **Proven** (Clarkson Thm 6; Thomas Thm 2/7). Tighter Frady numbers = **Empirical** | **Proven**; error grows **quadratically in sequence length** (Clarkson Thm 9) |
| **MAP-B** (sign-rounded bundle) | self-inverse, **Exact** | **membership-only, Proven** (Clarkson Thm 16); reliability decays 1/2+1/2^r with depth r → **forbid deep nesting under Proven** | **Proven** |
| **BSC** (XOR bind, majority bundle) | XOR self-inverse, **Exact** | **Proven on expectation** (Heim / Yi & Achour: min size to hit target accuracy in expectation) — weaker than w.p.≥1−δ; tag accordingly | circular shift, **Exact** |
| **HRR / FHRR** (convolution / complex mult.) | **NOT self-inverse** → approximate inverse, lossy, needs cleanup → at most **Empirical** (single-factor); multi-factor needs resonator (§6) | addition; **Empirical** (Gaussian), or **Proven** iff phasor components are sub-Gaussian (Thomas) | **Proven** |
| **Sparse / block codes** (k-active) | algebraic part **Proven** | **Proven** via Bloom / Counting-Bloom analysis (Clarkson Thms 22–23; first rigorous set-*intersection*) | **Proven** |

Net: `bind`/`unbind` (MAP, BSC) and `permute` (all) are **Exact/algebraic**; `bundle` is **Proven** for MAP-I, sparse/Bloom, on-expectation BSC, and **Empirical** for HRR/FHRR; **HRR/FHRR `unbind` is Empirical and is the residual weak link.** Binding arity k blows up capacity super-exponentially → deep compositions cannot honestly carry tight Proven bounds.

## 5. Sparsity placement (T1.3) — resolved; feeds RFC-0001
- **Declared sparsity *class*** (`Sparse{max_active:k}`) is a **static refinement** `{v | activeCount v ≤ k}`, SMT-discharged exactly as Liquid Haskell checks length/bounds. **Feasible today.**
- **Capacity bounds** (given k, derive δ ≤ target) are encoded as **refinement post-conditions whose soundness is a cited axiom** (the T0.2 theorems); the checker discharges only the **arithmetic instantiation** (concrete d, k, s, m, δ). Soundness of the *formula* is axiomatized from the literature, not proved by the type system.
- **Observed** sparsity remains runtime `Meta`.
- Caveat: how sparsity *evolves* through nonlinear (majority/sign) bundling is genuinely hard → only **conservative declared** post-conditions there.
- *Closest prior art:* **Heim** (Yi & Achour, OOPSLA 2023) does this as static *analysis*, not types — borrow its derivations. No prior system tracks VSA capacity *in types*; this part is partly novel.

**Confirming probe (the one remaining make-or-break build):** encode MAP-I `bundle` in Liquid Haskell as `{v | activeCount ≤ s} → {d | d ≥ ⌈(2/μ^{2})·ln(m/δ)⌉} → {r | failProb r ≤ δ}` and confirm Z3 discharges it for concrete params. Success ratifies the cited-theorem + checked-instantiation strategy (and KC-1 / ADR-010).

## 6. Reconstruction manifest (T2.2) — normative
Distinguish, explicitly and inspectably:
- **Indexed retrieval** = codebook + similarity + threshold; returns a *stored atom*; bounded-lossy (Clarkson Thm 16). NOT holographic reconstruction.
- **True compositional reconstruction** = requires the **structural recipe / role schema** (which ops combined which slots) + **algebraic inverse operations**; can recover *novel* combinations never stored. This is VSA's defining capability over a hash table.

**Minimal manifest contents:** (1) VSA model + dimension d; (2) codebook(s) — atomic item memory, as **content-addressed references** (Unison identity fits); (3) compositional recipe/role schema (if reconstruction must be compositional); (4) decoding procedure + params (cleanup threshold, or resonator factor structure + iteration budget); (5) the `{ε, δ, strength}` bound certificate.

**Factorization** (resonator networks — Frady/Kent/Olshausen/Sommer, *Neural Computation* 32(12), 2020): needed when a vector is a binding product of *unknown* factors; manifest adds per-factor codebooks + binding op + approximate inverse. **Not guaranteed to converge** (almost always within an operational-capacity regime); reconstruction is **lossy-bounded, best-effort**. Kept **Phase-3 exploratory** with a **probabilistic-only** guarantee (FR-C2).

## 7. Implementation note (T2.6)
Build the VSA submodule in Rust (no production-grade Rust VSA library exists; torchhd is Python — port its well-documented operation set as the reference). Reuse the `balanced-ternary` crate for trit storage / 5-trit-per-byte & 40-trit packings; evaluate the early "bitsliced ternary + VSA" crate as a reference. Implement MAP/BSC/HRR/FHRR/sparse with the §4 tags.

## 8. Interfaces
Provides `bind/unbind/...` as RFC-0001 prims over `VSA`; populates `Meta.bound` (`CapacityBound`/`CrosstalkBound`) with lattice-correct tags; uses **ADR-010** kernels. §5 sparsity-refinement decision **feeds back into RFC-0001 §4.3/§4.5** (now applied there).
