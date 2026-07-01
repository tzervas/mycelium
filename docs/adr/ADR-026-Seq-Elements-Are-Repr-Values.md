# ADR-026 — Seq elements are repr-values only

| Field | Value |
|---|---|
| **ADR** | 026 |
| **Status** | **Accepted** (2026-07-01 — maintainer-ratified, part of the RFC-0033 ratification act). Was **Proposed** (2026-06-24) — value-model decision recommended by the research review. |
| **Decides** | A `Seq`'s element `Repr` MUST be a repr-value (Binary/Ternary/Dense/Vsa/Seq/Bytes) — homogeneous, repr-values only. Heterogeneous/`Vec<Struct>`/`Map` are ADTs **above** the kernel. |
| **Grounds** | RFC-0033 §3.1.1, §3.1.4 (the normative statement); RFC-0001 §4.1 (closed paradigm kinds); KC-3 (avoid a recursive data/type registry in the trusted base); `research/14-value-model-integration-report-RECORD.md` §3 (A2). |
| **Date** | 2026-06-24 |

> **Posture (VR-5).** Now **Accepted**. **One-way door** (admitting general algebraic elements later is
> additive; retracting them is not). No implementation lands with this ADR — the E20-1 V1 `Seq`
> implementation this decision governs is deferred post-1.0, per RFC-0033's 2026-07-01 ratification.

## Context
Where to cut element generality for a minimal kernel that still makes collections real.

## Decision
Homogeneous **repr-value** elements only; heterogeneous / `Vec<Struct>` / `Map` are recursive ADTs
**above** the kernel, using `Seq` for storage. `Map` in particular MUST NOT be a kernel repr (its
hashing/ordering policy is not canonically content-addressable).

## Status
**Accepted** (2026-07-01, maintainer-ratified). **One-way door.**

## Consequences
The kernel stays closed over itself: a `Seq` of repr-values is still a fixed-size `Repr`, swaps compose
element-wise, content-addressing stays structural. No registry entanglement; the clean "swaps are
between paradigms" model is preserved.

## Rejected
**General algebraic-data elements** (`Vec<MyStruct>`, `Map<K,V>` in the kernel) — forces `Repr` into a
recursive container over the whole value model plus a data/type registry: the single largest
trusted-surface explosion available, and it breaks the per-paradigm swap model (no certified swap for
an arbitrary user struct).

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-07-01 | **Accepted** | Maintainer-ratified, as part of the RFC-0033 §1–§8 ratification act (alongside ADR-025/027/028; ADR-029/030/031 already Accepted 2026-06-24). No implementation lands with this transition; the E20-1 V1 implementation is deferred post-1.0. |
| 2026-06-24 | **Proposed** | Initial record. Repr-values-only elements (A2). Grounds RFC-0033 §3.1.1/§3.1.4. |
