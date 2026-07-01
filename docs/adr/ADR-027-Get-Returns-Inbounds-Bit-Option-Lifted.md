# ADR-027 — `get` returns (Repr, in_bounds bit); Option lifted above the kernel

| Field | Value |
|---|---|
| **ADR** | 027 |
| **Status** | **Accepted** (2026-07-01 — maintainer-ratified, part of the RFC-0033 ratification act). Was **Proposed** (2026-06-24) — value-model decision recommended by the research review. |
| **Decides** | `get(seq, i)` returns `(elem_repr, in_bounds: Binary{1})`; a single blessed `lift_option` adapter assembles `Option<Repr>` immediately above the kernel. `get` carries tag **Exact**. |
| **Grounds** | RFC-0033 §3.1.2 (the normative statement); RFC-0032 §5 D1 (the ratified `eq → Binary{1}` + Bool-lift pattern this mirrors); G2 (never-silent); `research/14-value-model-integration-report-RECORD.md` §3 (A3). |
| **Date** | 2026-06-24 |

> **Posture (VR-5).** Now **Accepted**. The `lift_option` adapter is **non-trusted** (above KC-3) but its
> single-call-site discipline is the mechanical enforcement of never-silent. No implementation lands
> with this ADR — the E20-1 V1 `get`/`lift_option` implementation is deferred post-1.0, per RFC-0033's
> 2026-07-01 ratification.

## Context
Kernel primitives return *representation values*, not algebraic data; `get` must still signal
out-of-bounds. `Option`/`Result` are ADTs that live above the kernel.

## Decision
`get → (elem_repr, in_bounds: Binary{1})`. On `in_bounds = 0` the returned repr is a defined-but-ignored
zero. The `Option<Repr>` is assembled by **one blessed, generated `lift_option(repr, bit)` adapter**,
reused at every call site (CI grep enforces it is the only construction site). This mirrors the
ratified `eq → Binary{1}` then Bool-ADT lift (RFC-0032 D1) — consistency in the most sensitive layer is
itself the argument. It also mirrors CPU overflow/carry flags, SMT-LIB overflow *predicates*
(`bvuaddo`/`bvsaddo` return a Bool beside the wrapping result), and seL4/CompCert TCB-minimization.

## Status
**Accepted** (2026-07-01, maintainer-ratified).

## Consequences
No ADTs leak into KC-3; never-silent via an explicit bit; the pattern scales to UTF-8 decode,
div-by-zero, and cast-overflow.

## Rejected
- **`Option` as a kernel primitive return** — drags a generic tagged sum into KC-3 (the very thing
  ADR-026 excludes) and is circular (you need ADTs to express a kernel op's result).
- **Sentinel/poison value** — a silent in-band signal; violates never-silent.
- **Trap/panic on OOB** — destroys totality; `get` must be total with an explicit status.

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-07-01 | **Accepted** | Maintainer-ratified, as part of the RFC-0033 §1–§8 ratification act (alongside ADR-025/026/028; ADR-029/030/031 already Accepted 2026-06-24). No implementation lands with this transition; the E20-1 V1 implementation is deferred post-1.0. |
| 2026-06-24 | **Proposed** | Initial record. `get → (Repr, in_bounds bit)`, blessed `lift_option` (A3). Mirrors RFC-0032 D1. Grounds RFC-0033 §3.1.2. |
