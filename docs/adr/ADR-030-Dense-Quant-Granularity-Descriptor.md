# ADR-030 — Dense quant is a granularity descriptor in Repr; scales in Payload

| Field | Value |
|---|---|
| **ADR** | 030 |
| **Status** | **Proposed** (2026-06-24) — value-model decision recommended by the research review; **disagrees with the input draft** (which put a scalar `{scale, zero_point}` in the `Repr`). Maintainer ratifies → Accepted (house rule #3). |
| **Decides** | Expand `Dtype` (I8/U8/I16/U16/I4/U4/F8E4M3/F8E5M2/TF32; MX FP6/FP4 later). Quant = `QuantDesc{scheme: PerTensor\|PerChannel{axis}\|PerBlock{axis,block}, symmetric, scale_dtype}` **in the `Repr`** (part of content-address identity); scale/zero-point **arrays in the `Payload`**. Quantized dtype ⇒ `quant = Some` (unconstructable otherwise). |
| **Grounds** | RFC-0033 §4.3 (the normative statement); RFC-0001 §4.1 (`Dense`/`ScalarKind` extensible registry); **ADR-011 (BoundBasis universal — the dequant `Bounded` bound extends the basis with block structure; OQ-3 already closed, not reopened)**; ONNX QuantizeLinear, GGUF K-quants, OCP MX (E8M0/32), NF4 (block 64); `research/14-value-model-integration-report-RECORD.md` §3 (B-Dense). |
| **Date** | 2026-06-24 |

> **Posture (VR-5).** Proposed decision direction. **Content-address one-way door** (the `quant`
> descriptor is hashed into every quantized value's identity). It lands in the single rehash (RFC-0033
> §7 / M-780) before any Dense value is persisted. No implementation lands with this ADR.

## Context
Float-only dtypes today. The input draft proposed a *single scalar* `{scale, zero_point}` pair per
value — honest **only** for per-tensor INT8/UINT8. Every mainstream sub-8-bit / modern embedding-quant
format is per-channel or per-block (ONNX per-axis/blocked; GGUF Q4_K 256-weight super-blocks with
6-bit sub-scales; OCP MX one E8M0 scale per 32-element block; NF4 blocksize 64), and a scalar literally
cannot represent them — using one is **silently wrong** at the `Repr` level.

## Decision
A granularity descriptor `QuantDesc{scheme, symmetric, scale_dtype}` goes **in the `Repr`** (small,
fixed-size, determines layout, part of content-address identity so per-tensor and per-block values
cannot collide). The scale/zero-point **arrays** are `O(dim/block)`, so they are **`Payload`**, not
`Repr` (variable-length scale vectors in `Repr` would re-break the fixed-size invariant of ADR-025).
The draft's "metadata in `Repr`" instinct is right for the *descriptor*, wrong for the *data*.

## Status
**Proposed (recommended)** — **disagrees with** the input draft. **One-way door.**

## Consequences
Honest for ONNX per-axis/blocked, GGUF K-quants, OCP MX, NF4. Dequant is a `Bounded` swap whose error
bound depends on granularity → touches the `Bounded` cert and the dequant `bound.basis` (extends
ADR-011 with block structure), the guarantee lattice (dequant ≤ `Empirical`), and the M-I bound series
(gains a granularity parameter). The descriptor is part of content-address identity of every quantized
value — ship it before anyone stores Dense values.

## Rejected
**Scalar-per-tensor-only** (dishonest/under-general); **all-meta-in-Payload** (per-tensor and per-block
then share an address despite being non-interchangeable — silent aliasing); **defer quant entirely**
(the content-address door makes deferral *more* expensive — ship the descriptor now, stage the payload
paths: `PerTensor` + `PerBlock` first).

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-24 | **Proposed** | Initial record. Granularity-descriptor quant in `Repr` + scale arrays in `Payload` (B-Dense, disagrees with the input draft). Dequant basis extends ADR-011 (OQ-3 not reopened). Grounds RFC-0033 §4.3. |
