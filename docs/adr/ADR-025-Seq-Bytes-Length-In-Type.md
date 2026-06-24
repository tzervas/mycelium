# ADR-025 — Seq/Bytes length lives in the type

| Field | Value |
|---|---|
| **ADR** | 025 |
| **Status** | **Proposed** (2026-06-24) — value-model decision recommended by the research review; maintainer ratifies → Accepted (house rule #3). |
| **Decides** | `Repr::Seq { elem, len: u32 }` and `Repr::Bytes { len: u32 }` carry length **in the `Repr` type** (a fixed-size `u32` parameter, not variable-length data). Growables are higher structures over a fixed-capacity `Seq`. |
| **Grounds** | RFC-0033 §3.1 (the normative statement); **RFC-0032 §5 D3/D4 (already decides the `Seq`/`Bytes` shape + placement — this ADR is consistent with and adds the value-model invariant)**; RFC-0001 §4.1 (fixed-size `Repr`); KC-3 (+1 `u32`, already justified by `width`/`dim`); `research/14-value-model-integration-report-RECORD.md` §3 (A1). |
| **Date** | 2026-06-24 |

> **Posture (VR-5).** Records a *decision direction*, Proposed. The `Seq`/`Bytes` shape itself is the
> RFC-0032 D3/D4 decision (in `core` 1.0.0); this ADR adds the value-model invariant (length-in-type)
> and is **one-way-door** on that invariant. No implementation lands with this ADR.

## Context
Adding indexed collections to a flat fixed-size value model. "Every `Repr` is fixed-size" underpins
lowering, codegen, allocation, and content-addressing (RFC-0001).

## Decision
`Seq{elem, len:u32}` / `Bytes{len:u32}` — length in the `Repr` type. Growable `Vec`/`DynamicSeq` are
higher structures over a fixed-capacity `Seq` (capacity + length, chunked, or COW), **not** kernel
reprs. (Aligned with RFC-0032 D3/D4.)

## Status
**Proposed (recommended).** **One-way door** (the invariant is corpus-wide; reversing it later is the
expensive direction — choosing length-in-type is the reversible-upward move).

## Consequences
Fixed-size invariant preserved; push/pop produce a new type; content-addressing stays structural.
LLVM array types (`[N x T]`), Rust `[T; N]`, sized-type languages (Idris/Agda `Vect n`), and MLIR's
static-dimension split all carry length in the type — this is the dominant low-level convention.

## Rejected
**Length-in-payload** (`Seq{elem}`) — a kernel-level growable, but it breaks the fixed-size-`Repr`
invariant every consumer leans on and imports heap/realloc concerns into the trusted base for a
capability (growth) that belongs above the kernel.

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-24 | **Proposed** | Initial record. Seq/Bytes length-in-type (A1). Consistent with RFC-0032 D3/D4 (which decides the shape + `core`-1.0.0 placement); this ADR fixes the value-model invariant. Grounds RFC-0033 §3.1. |
