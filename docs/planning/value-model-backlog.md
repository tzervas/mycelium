# Value-Model Backlog — User Stories, Acceptance, Requirements, DoD

Status: Proposed — companion to RFC-0033 + ADR-025…031 (all Proposed). Pending ratification.

Companion to `value-model-implementation-plan.md` (phases/tasks) and
RFC-0033 (`docs/rfcs/RFC-0033-Value-Model-Collections-and-Precision.md`, normative spec).
Personas: **KE** kernel engineer, **VM** value-model author, **ML** ML/embedding
integrator, **PM** project manager, **EV** evaluator/QA, **AU** auditor of the trusted base.

---

## 1. User stories & acceptance criteria

Acceptance is **Given/When/Then**. Each story names the task(s) it closes.

### US-1 — Indexed sequences without breaking the kernel invariant *(M-760/M-761/M-762)*
*As a* VM author, *I want* a fixed-size `Seq{elem,len}` with a never-silent `get`, *so
that* I can model arrays without introducing variable-length data into the kernel.
- **AC-1.1** Given `Seq{Binary{8}, 4}`, when I read index 2, then I get the element repr
  with `in_bounds=1`.
- **AC-1.2** Given the same, when I read index 9, then `in_bounds=0` and the returned repr
  is a defined zero that the caller must ignore; **no panic**.
- **AC-1.3** When any code needs `Option<Repr>`, then it is produced only by
  `lift_option`; a CI grep finds no other construction site.
- **AC-1.4** No `Repr` variant compiles with a `Vec`/variable-length field (review
  checklist + type audit).

### US-2 — Honest text boundaries *(M-763)*
*As a* VM author, *I want* UTF-8 decode to be never-silent, *so that* invalid bytes are
never silently replaced.
- **AC-2.1** Given valid UTF-8 bytes, decode yields the string with `valid=1`.
- **AC-2.2** Given bytes with an invalid sequence, decode yields `valid=0` and the
  `Result::Err` is surfaced above the kernel; **no `U+FFFD` substitution**.

### US-3 — Large balanced-ternary integers that never silently corrupt *(M-756/M-757)*
*As a* KE, *I want* arbitrary-width balanced-ternary arithmetic with a never-silent
fixed-width boundary, *so that* values beyond ~40 trits are exact, not wrapped.
- **AC-3.1** `3^41` is represented exactly (width 42, value `3^41`), unlike the i64 path.
- **AC-3.2** Growable add/mul never overflow; carry-out becomes a new trit.
- **AC-3.3** For `Ternary{trits:N}`, an op exceeding `±(3^N−1)/2` returns `None`; e.g.
  width-3 `13 + 1 → None`; width-3 `6 + 6 → 12`.
- **AC-3.4** A limbed implementation, if present, is bit-exact to the digit-serial
  reference over a fuzz corpus.
- **AC-3.5** Canonical form is non-redundant (each integer has exactly one trit string).

### US-4 — Integer/bitvector without signedness fragmenting identity *(M-765/M-766/M-767)*
*As a* KE, *I want* `Binary{width}` to be sign-free with signedness in operations, *so
that* a reinterpretation is not a content-address change.
- **AC-4.1** The same bit pattern has **one** content address regardless of signed/
  unsigned interpretation.
- **AC-4.2** `add/sub/mul/neg` are bit-identical across signedness (property test over
  random widths/values).
- **AC-4.3** `div/cmp/shift/overflow-detect` have distinct signed and unsigned variants.
- **AC-4.4** Fixed-width overflow returns `Option`/`Result`; **no two's-complement wrap**.

### US-5 — Quantized embeddings represented honestly *(M-770/M-771/M-772/M-773)*
*As an* ML integrator, *I want* per-channel/per-block quantization expressible, *so that*
real model formats are not silently misrepresented.
- **AC-5.1** A per-block (block=32, MXFP8) value is constructible with a `PerBlock`
  descriptor and a scale **array** in the Payload.
- **AC-5.2** Attempting to represent a per-block value with a single scalar scale **fails
  to construct** (type/guard error), not silently.
- **AC-5.3** A quantized dtype with `quant=None` is unconstructable.
- **AC-5.4** Per-tensor and per-block encodings of the same data have **distinct content
  addresses**.
- **AC-5.5** Conformance vectors for ONNX per-axis/blocked, GGUF Q4_K, OCP MXFP8, and NF4
  round-trip through `Repr`+`Payload` without loss of structure.

### US-6 — Full VSA family coverage *(M-775/M-776/M-777/M-778/M-779)*
*As an* ML integrator, *I want* the element space and sparsity explicit, *so that* FHRR
(complex), MAP-I (integer), and SBC/BSDC (block-sparse) are all representable.
- **AC-6.1** A complex model stores into `HypervectorC` with honest `dim` (D complex
  components, not 2D reals).
- **AC-6.2** Storing a complex HV as interleaved reals in `Hypervector` **fails to
  construct** for a complex model.
- **AC-6.3** An SBC vector (`SparseBlock{blocks, active_per_block:1}`) validates;
  a vector violating one-active-per-block is **rejected** (never-silent).
- **AC-6.4** A BSDC vector (`active_per_block>1`) is representable and validatable.
- **AC-6.5** `Vsa.elem` and `Vsa.sparsity` are part of content-address identity.

### US-7 — Coherent swaps & guarantees after the changes *(M-780/M-781/M-782)*
*As an* AU, *I want* the certificate kinds and guarantee lattice reconciled, *so that*
every conversion carries an honest, correctly-typed certificate.
- **AC-7.1** Binary↔ternary is `LosslessWithinRange` with the growable case marked
  lossless and the fixed case range-bounded.
- **AC-7.2** Dequant carries a `Bounded` certificate whose bound is computed from
  `QuantScheme` (per-block bound ≤ per-tensor bound for the same data).
- **AC-7.3** `bound.basis` is a single required field on the shared `Bound` supertype,
  reconciled with the existing ratified **ADR-011 (BoundBasis-Is-Universal)**, and
  includes block structure for dequant bounds (**OQ-3 closed**).
- **AC-7.4** Dequant results are tagged at most `Empirical` (never `Exact`/`Proven`).
- **AC-7.5** M-I bounds are parameterized by granularity.

### US-8 — Trustworthy lift with provenance *(M-754/M-755, M-777, M-783)*
*As an* AU, *I want* every lifted artifact attributed and the trusted/untrusted boundary
explicit, *so that* the trusted base stays minimal and auditable.
- **AC-8.1** `PROVENANCE.md` records source file, commit, license, and every change.
- **AC-8.2** No `unsafe` appears in trusted modules; SIMD lives in the untrusted accel
  crate; removing it leaves correctness intact.
- **AC-8.3** Only the exhaustively-tested `ternary.rs::Trit` is lifted; `dimensional.rs`
  is not.

### US-9 — Perf without paying before it's needed *(M-758/M-759, M-783/M-784)*
*As a* PM, *I want* perf paths gated on benchmarks, *so that* we don't add complexity
(or trusted surface) speculatively (YAGNI).
- **AC-9.1** A limbed/SIMD path is added only after a profile shows the reference path is
  hot, and ships with an equivalence proof/proptest against the reference.

---

## 2. Requirements

### Functional (FR)
- **FR-1** `Repr` MUST express: width-general Binary; width-general Ternary; Dense with
  expanded dtypes + granularity descriptor; VSA with element space + Dense/SparseGlobal/
  SparseBlock; `Seq`; `Bytes`.
- **FR-2** `Payload` MUST include `Bits`, `Trits`, `Scalars`, `Hypervector`,
  `HypervectorC`, and a quantized-Dense arm carrying scale/zero-point arrays.
- **FR-3** Boundaries (`get`, UTF-8, cast/narrow, fixed-width overflow, dequant) MUST be
  never-silent.
- **FR-4** Arbitrary-width ternary arithmetic MUST be exact (growable) with a never-
  silent fixed-width boundary.
- **FR-5** Construction guards MUST reject: scalar scale for per-block; `quant=None` for a
  quantized dtype; real payload for a complex model; invalid SBC block structure.
- **FR-6** Every cross-paradigm swap MUST carry a certificate of the correct kind with a
  passing bound/range check, and a guarantee tag consistent with the lattice.

### Non-functional (NFR)
- **NFR-1 (trust)** No `unsafe` in KC-3 modules; SIMD isolated and untrusted.
- **NFR-2 (toolchain)** Builds on **MSRV 1.92**; `cargo test`, `clippy -D warnings`,
  `fmt --check` all green; CI on GitHub Actions.
- **NFR-3 (style)** PEP8/Black for any Python tooling; Rust `fmt`; SOLID/DRY/KISS/YAGNI;
  composition over inheritance.
- **NFR-4 (testing)** Exhaustive tests for the trit truth tables; property tests for
  shared-op signedness invariance, block-sparse invariants, and ternary equivalence;
  fuzz for ternary arithmetic vs arbitrary-precision integers; coverage tracked
  (codecov).
- **NFR-5 (identity)** Content-address-affecting changes land in a single rehash before
  any value is persisted.
- **NFR-6 (provenance)** Every lifted file attributed with source + commit + license +
  diff rationale.
- **NFR-7 (determinism)** Canonical forms are unique (non-redundant ternary; defined zero
  for OOB), so content addresses are stable.

---

## 3. Deliverables

| # | Deliverable | Location | State |
|---|---|---|---|
| D-1 | Trit + Limb primitives (reconciled into `core::ternary`) | `crates/mycelium-core/src/ternary/{trit,limb,mod}.rs` | design-complete; lands in code PR |
| D-2 | Arbitrary-width ternary integer arithmetic + fixed-width boundary | `…/ternary/big_ternary.rs` | design-complete; lands in code PR |
| D-3 | Provenance record | `…/ternary/PROVENANCE.md` | design-complete; lands in code PR |
| D-4 | Ported exhaustive trit tests | `crates/mycelium-core/` inline `#[cfg(test)]` (exhaustive trit truth-table tests) | TODO |
| D-5 | Integration report & design | `research/14-value-model-integration-report-RECORD.md` | done |
| D-6 | Embeddonator leverage map | `research/15-embeddonator-leverage-map-RECORD.md` | done |
| D-7 | Normative RFC | `docs/rfcs/RFC-0033-Value-Model-Collections-and-Precision.md` | done |
| D-8 | ADRs (7) | `docs/adr/ADR-025…ADR-031` | done |
| D-9 | Implementation plan | `docs/planning/value-model-implementation-plan.md` | done |
| D-10 | This backlog | `docs/planning/value-model-backlog.md` | done |
| D-11 | Corrected `Repr`/`Payload` in the kernel | `crates/mycelium-core/src/…` | TODO (V1–V4) |
| D-12 | Quant conformance vectors (ONNX/GGUF/MX/NF4) | `crates/mycelium-core/` inline `#[cfg(test)]` (dense conformance vectors) | TODO |
| D-13 | Block-sparse VSA lift + complex carrier | `crates/mycelium-vsa/` | TODO |
| D-14 | Swap-cert / lattice / M-I reconciliation + ADR-011 BoundBasis reconciliation | corpus + kernel | TODO (V5) |
| D-15 | SIMD accel layer (untrusted) | `crates/mycelium-vsa-accel/` | TODO (V6) |

---

## 4. Success criteria (release-level)
- **SC-1** Every recognized data shape in scope is representable **without** silent
  misrepresentation: per-channel/per-block quant, FP8, complex/integer/block-sparse VSA,
  arbitrary-width ternary, fixed-width Binary/Ternary.
- **SC-2** Every boundary in the value model is never-silent (FR-3) — demonstrated by the
  guard tests in §5 failing to construct dishonest values.
- **SC-3** The trusted base grew only by justified, minimal additions (NFR-1) and every
  lifted line is attributed (NFR-6).
- **SC-4** Content addresses are stable and unique; the identity rehash happened once
  before persistence (NFR-5/7).
- **SC-5** Toolchain gates green on MSRV 1.92 (NFR-2); coverage does not regress.
- **SC-6** OQ-3 is closed by reconciliation with ADR-011 (BoundBasis-Is-Universal);
  OQ-4/OQ-5 remain explicitly declared-open (not silently resolved).

## 5. Global Definition of Done
A unit of work is **Done** when:
1. Code compiles on **MSRV 1.92**; `cargo test`, `cargo clippy -- -D warnings`,
   `cargo fmt --check` pass in CI.
2. Tests exist at the right level: **exhaustive** for finite truth tables, **property/
   fuzz** for algebraic laws and equivalence, **conformance vectors** for external
   formats; coverage tracked.
3. Never-silent is demonstrated by a **negative** test: the dishonest construction
   (scalar scale per-block; `quant=None`; real payload for complex; invalid SBC; silent
   overflow) **fails**.
4. Any KC-3 change has a recorded justification (ADR or PROVENANCE) and adds no `unsafe`.
5. Any content-address-affecting change is sequenced into the single rehash (M-780) and
   not merged piecemeal ahead of it.
6. Docs updated: the RFC/ADR/plan reflect the as-built; PROVENANCE current.
7. Conventional-commit messages (`feat(value-model):`, `test(value-model):`,
   `perf(vsa-accel):`, `docs(value-model):`).

---

## 6. Conformance test matrix (RFC → tests)

| RFC clause | Requirement | Test(s) | Story |
|---|---|---|---|
| §2.1 | no variable-length `Repr` | type audit / review checklist | US-1 |
| §2.2 | never-silent boundaries | negative guard tests (§5.3) | US-2,3,5,6 |
| §3.1.2 | `get → (Repr,bit)` | `seq_get_inbounds`, `seq_get_oob_no_panic` | US-1 |
| §3.1.2 | `lift_option` sole site | CI grep | US-1 |
| §3.2.1 | UTF-8 never-silent | `utf8_valid`, `utf8_invalid_surfaced` | US-2 |
| §4.1.1/.4 | sign-free Binary; never-silent overflow | `binary_one_address`, `binary_shared_ops_signless`, `binary_overflow_none` | US-4 |
| §4.2.2/.3 | arbitrary-width; fixed-width boundary | `bt_3pow41_exact`, `bt_fixed_overflow_none`, `bt_fuzz_vs_bigint` | US-3 |
| §4.2.2 | limbed ≡ reference | `bt_packed_equivalence` | US-3 |
| §4.3.4/.5 | descriptor in Repr; arrays in Payload | `dense_perblock_constructs`, `dense_scalar_perblock_rejected` | US-5 |
| §4.3.3 | quantized ⇒ `Some` | `dense_quant_none_rejected` | US-5 |
| §7 | distinct addresses | `dense_pertensor_vs_perblock_distinct_addr` | US-5 |
| §4.4.2/.4 | complex carrier | `vsa_complex_constructs`, `vsa_complex_as_real_rejected` | US-6 |
| §4.4.3 | block-sparse validate | `sbc_valid`, `sbc_invalid_rejected`, `bsdc_valid` | US-6 |
| §6.1 | `LosslessWithinRange` split | `swap_bin_ter_growable_lossless`, `swap_bin_ter_fixed_bounded` | US-7 |
| §6.2/§6 | `Bounded` from scheme; §4.3 basis; OQ-3 | `dequant_bound_from_scheme`, `bound_basis_includes_blocks` | US-7 |
| §6.3 | dequant ≤ Empirical | `dequant_tag_not_exact` | US-7 |
| NFR-1 | no `unsafe` in KC-3 | `grep_unsafe_kernel` CI gate | US-8 |
