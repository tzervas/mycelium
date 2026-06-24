# Value-Model Implementation Plan

Status: Proposed — companion to RFC-0033 + ADR-025…031 (all Proposed). Pending ratification.

Phased, dependency-ordered plan to land the decisions in ADR-025…ADR-031
(`docs/adr/`) and RFC-0033 (`docs/rfcs/RFC-0033-Value-Model-Collections-and-Precision.md`)
into the kernel + corpus.

> **Task IDs are canonical M-754…M-784 under epic E20-1** (was provisional VM-010…VM-071
> in the source bundle). Priorities: **P0** blocks dogfooding / is a content-address
> one-way door; **P1** core; **P2** perf/optional. "KC-3" marks trusted-kernel work
> (extra review + proof obligations).

## Critical path

```
M-754 ─┐                         (ternary substrate: reconcile primitives into core::ternary)
       ├─▶ M-756 ──▶ M-757     (arbitrary-width arithmetic + fixed-width boundary)
M-755 ─┘
M-770 (Dense descriptor) ─┐
M-775 (VSA elem+sparsity) ─┼─▶ M-780 (content-address rehash) ─▶ M-781 (swap/guarantee/M-I reconcile + ADR-011 BoundBasis / OQ-3)
M-765 (Binary signed?) ───┘                                          │
                                                                     ▼
                                                             DOGFOOD GATE: values may be persisted
```

The **content-address one-way doors** (M-770, M-775, optional M-765) and the §6
reconciliation (M-781) MUST complete before any value is persisted. Collections (M-760…M-764)
and the ternary substrate (M-754…M-759) are independently buildable and not on the identity
critical path, so they can proceed in parallel.

---

## Phase V0 — Ternary substrate (foundation) · P0

| ID | Task | KC-3 | Deps | Status |
|---|---|---|---|---|
| M-754 | Reconcile ternary primitives into `core::ternary` (serde optional, contract docs): the canonical `core::Trit` already exists; extract the shared balanced full-adder (`add_with_carry`) DRY from the existing fixed-width `add`. `Limb27`/`PackedTernary` are a YAGNI perf follow-on (M-758/M-759), not lifted now. | ✔ | — | **designed + algorithm-validated (10k fuzz, Python port); implementation reconciled into `core::ternary` lands in the follow-up code PR after the mandatory `cargo +1.92 test/clippy/fmt` gate.** |
| M-755 | Port exhaustive `Trit` truth-table tests from `embeddonator-core` (inline `#[cfg(test)]` in `core`) | ✔ | M-754 | TODO |
| M-756 | Arbitrary-width `BigTernary` (digit-serial) + `from/to_i128`, never-silent | ✔ | M-754 | **designed + algorithm-validated (10k fuzz, Python port); implementation reconciled into `core::ternary` lands in the follow-up code PR after the mandatory `cargo +1.92 test/clippy/fmt` gate.** |
| M-757 | Fixed-width boundary: `FixedWidthTrits`, `checked_add_fixed`, `checked_to_width` | ✔ | M-756 | **designed + algorithm-validated (10k fuzz, Python port); implementation reconciled into `core::ternary` lands in the follow-up code PR after the mandatory `cargo +1.92 test/clippy/fmt` gate.** |
| M-758 | (P2) Limbed `PackedTernary` (40 trits/u64) + differential equivalence vs `BigTernary` | ✔ | M-756 | TODO (gate on bench) |
| M-759 | Multiplication fast path (Karatsuba/Toom) proven equivalent to schoolbook | ✔ | M-756 | TODO (gate on bench) |

**Phase DoD.** `cargo +1.92 test` green incl. ported exhaustive tests + `BigTernary`
fuzz; `clippy -D warnings`; `fmt --check`; `3^41` exactness test present; no `unsafe` in
the trusted ternary modules; PROVENANCE complete.

## Phase V1 — Collections · P1

> Collections (M-760…M-764) **overlap and align with RFC-0032 / E19-1**, which already
> scopes `Repr::Seq` / `Repr::Bytes`. These tasks **extend** RFC-0032, they do not
> re-decide it.

| ID | Task | KC-3 | Deps |
|---|---|---|---|
| M-760 | `Repr::Seq{elem,len}` + `Repr::Bytes{len}`; enforce fixed-size invariant in constructors | ✔ | — |
| M-761 | `get → (Repr, in_bounds:Binary{1})` kernel primitive (Exact tag) | ✔ | M-760 |
| M-762 | Blessed `lift_option(repr,bit) -> Option<Repr>` adapter (single call site, generated) | — | M-761 |
| M-763 | UTF-8 decode primitive `(bytes) -> (str, valid:Binary{1})` + `Result` lift | ✔ | M-760 |
| M-764 | Above-kernel growable `DynamicSeq` (capacity+len) over fixed `Seq` | — | M-760 |

**Phase DoD.** No `Repr` variant carries variable-length data (compile-time check or
review checklist); OOB `get` returns `in_bounds=0` with a defined zero, never panics;
invalid UTF-8 never silently replaced; `lift_option` is the only place `Option<Repr>` is
constructed (grep-enforced in CI).

## Phase V2 — Binary precision · P1

| ID | Task | KC-3 | Deps |
|---|---|---|---|
| M-765 | Confirm/keep `Binary{width:u32}` sign-free; document op-level signedness split | ✔ | — |
| M-766 | Two's-complement `add/sub/mul/neg` (shared) + never-silent fixed-width overflow | ✔ | M-765 |
| M-767 | Signedness-split ops: `div/cmp/shift/overflow-detect` (signed + unsigned variants) | ✔ | M-765 |
| M-768 | (P0, *only if* a nominal signed type is chosen) content-address decision for `signed` | ✔ | M-765 |
| M-769 | `BigInt` ADT above the kernel (arbitrary precision); kernel `Binary` does not grow | — | M-766 |

**Phase DoD.** Shared ops bit-identical across signedness (property test); overflow
never-silent; if M-768 is taken it is recorded as an explicit superseding ADR before
merge (default is NOT to add `signed` to `Repr`).

## Phase V3 — Dense precision · P0 (content-address one-way door)

| ID | Task | KC-3 | Deps |
|---|---|---|---|
| M-770 | Expand `Dtype` (I8/U8/I16/U16/I4/U4/F8E4M3/F8E5M2/TF32) | ✔ | — |
| M-771 | `QuantDesc{scheme,symmetric,scale_dtype}` + `QuantScheme{PerTensor,PerChannel,PerBlock}` in `Repr` | ✔ | M-770 |
| M-772 | Construction guard: quantized dtype ⇒ `quant=Some`; scalar-scale unconstructable | ✔ | M-771 |
| M-773 | `Payload::QuantDense` (codes + scale/zero-point arrays); per-tensor + per-block paths first | ✔ | M-771 |
| M-774 | Ingest conformance vectors: ONNX per-axis/blocked, GGUF Q4_K, OCP MXFP8, NF4 | — | M-773 |

**Phase DoD.** A per-block value cannot be expressed with a scalar scale (compile/
construction failure); per-tensor and per-block of the same data have **distinct content
addresses**; conformance vectors round-trip; `quant` descriptor is hashed into identity.

## Phase V4 — VSA precision · P0 (content-address one-way door)

| ID | Task | KC-3 | Deps |
|---|---|---|---|
| M-775 | Add `VsaElem` + `VsaSparsity{…,SparseBlock}` to `Repr::Vsa` | ✔ | — |
| M-776 | `Payload::HypervectorC(Vec<Complex<f64>>)` carrier | ✔ | M-775 |
| M-777 | Lift `block_sparse.rs` (BSDC) into `mycelium-vsa` (unit + proptest) | — | M-775 |
| M-778 | Construction guard: complex model ⇒ `HypervectorC`; real payload rejected | ✔ | M-776 |
| M-779 | Registry binding/bundling per `(model, elem)`; FHRR/MAP-C/MAP-I/SBC smoke tests | — | M-776, M-777 |

**Phase DoD.** FHRR/MAP-C representable (complex carrier, honest `dim`); SBC/BSDC
representable and validatable (invalid block vectors rejected, never-silent); complex HV
cannot be stored as interleaved reals; `Vsa.elem`/`sparsity` hashed into identity.

## Phase V5 — Swap / guarantee / M-I reconciliation · P0 (cross-cutting)

| ID | Task | KC-3 | Deps |
|---|---|---|---|
| M-780 | Single content-address rehash landing M-771 + M-775 (+ M-768 if taken) | ✔ | M-771, M-775 |
| M-781 | `LosslessWithinRange` growable-vs-fixed split; `Bounded` bound from `QuantScheme`; **`bound.basis` reconciles with the existing ratified ADR-011 (BoundBasis-Is-Universal) — OQ-3 is *already closed* by ADR-011, not by this work; M-781 only *extends* the dequant basis with block structure**; M-I granularity parameter; dequant ≤ Empirical | ✔ | M-780 |
| M-782 | Swap certs: binary↔ternary, complex↔real (`Bounded`), bipolar↔binary (`LosslessWithinRange`) | ✔ | M-781 |

**Phase DoD.** `bound.basis` defined once on the shared `Bound` supertype per ADR-011
(OQ-3 closed) and includes block structure for dequant; every new swap has a certificate
of the correct kind with a passing bound check; guarantee tags respect the lattice;
**DOGFOOD GATE opens** (values may be persisted).

## Phase V6 — VSA acceleration · P2 (perf, untrusted)

| ID | Task | KC-3 | Deps |
|---|---|---|---|
| M-783 | Lift `PackedTritVec` SIMD (AVX2/AVX-512) into `mycelium-vsa-accel`, isolated `unsafe` | ✘ | M-777 |
| M-784 | Property-test accel layer vs scalar reference; kernel never depends on it for correctness | ✘ | M-783 |

**Phase DoD.** `unsafe` confined to the accel crate; scalar↔SIMD equivalence proptests
green; removing the accel crate leaves correctness intact.

---

## Sequencing notes
- **Parallelizable now:** V0 and V1 (no shared deps, not on the identity path).
- **Must precede persistence:** V3, V4, V5 (and V2/M-768 only if a nominal signed type
  is adopted). Land M-780 as a *single* rehash to avoid multiple address migrations.
- **Gate perf on benchmarks (YAGNI):** M-758, M-759, V6 only after a profile shows the
  reference paths are hot.
- **Phase 2/3 epics** (per the master program plan) decompose further only at their gate
  milestones; this plan covers the value-model slice.
