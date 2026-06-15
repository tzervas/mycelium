# RFC-0003 — VSA Submodule Boundary

| Field | Value |
|---|---|
| **RFC** | 0003 |
| **Status** | **Accepted (r4)** (r2 scope note per ADR-013; r3 §4.1 erratum reconciling the permute / bind-unbind tags with the §4 "Net"; r4 §6.1 resonator decode params, additive, per RFC-0009) |
| **Type** | Foundational / normative |
| **Date** | June 08, 2026 |
| **Depends on** | RFC-0001 (`VSA` Repr kind, `Hypervector` slot, `ModelId` registry, `CapacityBound`/`CrosstalkBound`, lattice); ADR-008 (VSA optional submodule); ADR-010 (bound kernels); Research Findings **T0.2**, **T1.2**, **T1.3**, **T2.2**, **T2.6** |
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

> ⚠️ **The table cells above are superseded for two entries by the §4.1 erratum (r3):** the `permute`
> column is **`Exact`** for every model (not `Proven`), and the HRR/FHRR **bind/unbind** cell splits
> into **bind `Exact`, unbind `Empirical`**. The cells are preserved as written (append-only); §4.1 is
> the authoritative reading and is what `mycelium-vsa` implements.

### 4.1 Erratum (r3) — `permute` is `Exact`; HRR/FHRR `bind` is `Exact`, `unbind` is `Empirical`

The §4 table (r2) contradicts its own **Net** line on two entries, and the contradiction surfaced
during the 2026-06 code review (finding A3-01/A3-02): the code (`mycelium-vsa::matrix.rs`) follows the
Net line, the table cells say otherwise. This erratum resolves the contradiction **in favour of the
Net line**, on a checked algebraic basis (the honesty rule forbids ratifying a stronger tag without
one — here the *weaker-or-equal*, more honest reading is the algebraically exact one):

- **`permute` is `Exact` for all models.** Permutation is a *fixed coordinate permutation* (typically
  a circular shift): a bijection on the hypervector that preserves the alphabet (it relabels
  components, never altering ±1 / bit / real / complex values) and is exactly invertible by the
  inverse permutation. It introduces **no** approximation, so it is `Exact`/algebraic — like `neg` or
  a relabeling. The table's "**Proven**; error grows quadratically in sequence length (Clarkson Thm
  9)" conflated the *operation* with **sequence-decoding error growth**: the quadratic error is a
  property of *retrieving an item from a permutation-encoded, bundled sequence* (a `bundle` + repeated
  `permute` + `unbind`/cleanup composition), arising from the **bundle** superposition crosstalk — not
  from the `permute` step. That composition error remains governed by the `bundle`/`unbind` cells
  (unchanged); the `permute` operation in isolation is `Exact`. Basis: research **T1.2** ("permute
  Exact everywhere") and the bijection argument above.

- **HRR/FHRR `bind` is `Exact`; `unbind` is `Empirical`.** The r2 cell lumped bind and unbind as "at
  most `Empirical` (single-factor)". But `bind` (circular convolution for HRR; elementwise complex
  multiplication for FHRR) is a deterministic, exact algebraic operation; the lossy part is **`unbind`**
  — the *approximate inverse* that is not self-inverse and needs cleanup, which stays `Empirical` and
  remains "the residual weak link" exactly as the Net line says (RR-13, unchanged). Splitting the
  cell makes the per-operation tag honest (VR-5): the operation that is exact is tagged `Exact`, the
  operation that is lossy is tagged `Empirical`. (This supersedes the prior, non-citable "issue #61"
  rationale that the code comment referenced.)

Unchanged by this erratum: every `bundle` tag, the BSC "Proven on expectation" qualifier, the
HRR/FHRR `unbind` `Empirical` weak link, and the capacity-bound side-conditions (§5). The per-model ×
per-op tags asserted in `mycelium-vsa/tests/matrix.rs` are the authoritative encoding of this
corrected matrix.

## 5. Sparsity placement (T1.3) — resolved; feeds RFC-0001
- **Declared sparsity *class*** (`Sparse{max_active:k}`) is a **static refinement** `{v | activeCount v ≤ k}`, SMT-discharged exactly as Liquid Haskell checks length/bounds. **Feasible today.**
- **Capacity bounds** (given k, derive δ ≤ target) are encoded as **refinement post-conditions whose soundness is a cited axiom** (the T0.2 theorems); the checker discharges only the **arithmetic instantiation** (concrete d, k, s, m, δ). Soundness of the *formula* is axiomatized from the literature, not proved by the type system.
- **Observed** sparsity remains runtime `Meta`.
- Caveat: how sparsity *evolves* through nonlinear (majority/sign) bundling is genuinely hard → only **conservative declared** post-conditions there.
- *Closest prior art:* **Heim** (Yi & Achour, OOPSLA 2023) does this as static *analysis*, not types — borrow its derivations. No prior system tracks VSA capacity *in types*; this part is partly novel.

**Confirming probe (the one remaining make-or-break build):** encode MAP-I `bundle` in Liquid Haskell as `{v | activeCount ≤ s} → {d | d ≥ ⌈(2/μ²)·ln(m/δ)⌉} → {r | failProb r ≤ δ}` and confirm Z3 discharges it for concrete params. Success ratifies the cited-theorem + checked-instantiation strategy (and KC-1 / ADR-010).

## 6. Reconstruction manifest (T2.2) — normative
Distinguish, explicitly and inspectably:
- **Indexed retrieval** = codebook + similarity + threshold; returns a *stored atom*; bounded-lossy (Clarkson Thm 16). NOT holographic reconstruction.
- **True compositional reconstruction** = requires the **structural recipe / role schema** (which ops combined which slots) + **algebraic inverse operations**; can recover *novel* combinations never stored. This is VSA's defining capability over a hash table.

**Minimal manifest contents:** (1) VSA model + dimension d; (2) codebook(s) — atomic item memory, as **content-addressed references** (Unison identity fits); (3) compositional recipe/role schema (if reconstruction must be compositional); (4) decoding procedure + params (cleanup threshold, or resonator factor structure + iteration budget); (5) the `{ε, δ, strength}` bound certificate.

**Factorization** (resonator networks — Frady/Kent/Olshausen/Sommer, *Neural Computation* 32(12), 2020): needed when a vector is a binding product of *unknown* factors; manifest adds per-factor codebooks + binding op + approximate inverse. **Not guaranteed to converge** (almost always within an operational-capacity regime); reconstruction is **lossy-bounded, best-effort**. Kept **Phase-3 exploratory** with a **probabilistic-only** guarantee (FR-C2).

> **Scope note (r2, per ADR-013).** The surface term **`spore`** now names the
> **content-addressed deployable unit** (code + values + metadata, T4.3/T4.4); the
> reconstruction manifest specified in this section is **one digest-referenced component** of a
> spore, and the surface expression `spore(v)` constructs the degenerate single-value spore
> whose payload is `v`'s manifest. Nothing in this section's manifest contents, schema
> (`reconstruction-manifest`, unchanged), or guarantees changes — only the term's scope is
> reconciled so the narrow and broad senses cannot silently diverge (ADR-012 §7.4).

### 6.1 Resonator decode parameters (r4 — additive, per RFC-0009)

RFC-0009 (Accepted) specifies the resonator factorization semantics and, per its §4, records the
decode-side parameters in this manifest so a run is reproducible. The `decode` object (the
`DecodeSpec` the kernel carries — §2 "its metadata fields") gains, for `procedure == "Resonator"`
only, these **optional, additive** fields:

- `cleanup` — `"Softmax" | "ArgMax"`, the per-slot cleanup projection (default `Softmax`).
- `beta` — the softmax inverse-temperature `β > 0` (required-meaningful only when `cleanup == Softmax`).
- `tau_lock` — the per-slot top-similarity lock threshold `∈ [0, 1]` for the convergence verdict.
- `init` — `"UniformSuperposition" | "SeededGuess"`, the initialisation strategy (default uniform).
- `seed` — `u64 ≥ 0`, the initialisation seed (determinism — RFC-0009 §6).

This is a **metadata-only** extension: it adds inspectable fields to the manifest record the kernel
already carries (§2), and changes **no** kernel logic, algebra, swap machinery, or guarantee. The
existing invariants are unchanged and still enforced by `ReconInfo::new`/the schema: a `Resonator`
decode still requires non-empty `factors` + `iteration_budget ≥ 1`, and its bound basis **must not
exceed `Empirical`** (probabilistic-only, FR-C2). Out-of-range parameters are an explicit
`MalformedReconstruction`, never silently accepted (the never-silent rule, G2). The thresholds the
*decoder* applies beyond the schema (confidence, ambiguity-margin, oscillation window) live in the
submodule's `ResonatorParams`, not the kernel; promoting any of those into the manifest is a future
decision, not this revision.

## 7. Implementation note (T2.6)
Build the VSA submodule in Rust (no production-grade Rust VSA library exists; torchhd is Python — port its well-documented operation set as the reference). Reuse the `balanced-ternary` crate for trit storage / 5-trit-per-byte & 40-trit packings; evaluate the early "bitsliced ternary + VSA" crate as a reference. Implement MAP/BSC/HRR/FHRR/sparse with the §4 tags.

## 8. Interfaces
Provides `bind/unbind/...` as RFC-0001 prims over `VSA`; populates `Meta.bound` (`CapacityBound`/`CrosstalkBound`) with lattice-correct tags; uses **ADR-010** kernels. §5 sparsity-refinement decision **feeds back into RFC-0001 §4.3/§4.5** (now applied there).

## Meta — changelog

- **2026-06-15 (r4) — §6.1 resonator decode parameters (additive, per RFC-0009).** RFC-0009 is
  Accepted; its §4 records the resonator decode parameters in this manifest. Add the optional,
  additive `decode` fields for `procedure == "Resonator"` — `cleanup` (`Softmax`/`ArgMax`), `beta`
  (`> 0`), `tau_lock` (`∈ [0,1]`), `init` (`UniformSuperposition`/`SeededGuess`), `seed` (`u64`) — to
  `DecodeSpec` and `reconstruction-manifest.schema.json`, with out-of-range values an explicit
  `MalformedReconstruction` (G2). **Metadata-only** (§2): no kernel logic/algebra/guarantee change; the
  `Resonator` invariants (non-empty `factors`, `iteration_budget ≥ 1`, basis ≤ `Empirical`,
  probabilistic-only FR-C2) are unchanged. Also tightens the JSON schema to *require* `factors` +
  `iteration_budget` for `Resonator` (it previously under-enforced vs. the Rust contract). Append-only;
  the r3 §4.1 erratum and all §4 tags are unchanged.
- **2026-06-15 (r3) — §4.1 erratum.** Reconcile the §4 guarantee-tag table with its own "Net" line on
  two entries, on a checked algebraic basis: `permute` is `Exact` for all models (the table's "Proven"
  conflated the operation with sequence-decoding error growth, which belongs to `bundle`/`unbind`);
  and the HRR/FHRR bind/unbind cell splits into bind `Exact` (exact algebraic op) / unbind `Empirical`
  (the lossy approximate inverse — the residual weak link, unchanged). Table cells preserved
  (append-only); §4.1 is authoritative. Aligns the normative matrix with `mycelium-vsa::matrix.rs` /
  `tests/matrix.rs`. Resolves review findings A3-01/A3-02 (H4/H5). No code tag changes (the code
  already followed the Net line); supersedes the non-citable "issue #61" rationale.
- **2026-06-10 (r2) — scope note (ADR-013).** `spore` = the content-addressed deployable unit;
  this RFC's reconstruction manifest is one component of it; `spore(v)` is the degenerate
  single-value case. Manifest contents/schema unchanged. Resolves ADR-012 §7.4.
