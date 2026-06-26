# ADR-028 — Binary is sign-free; signedness is operations

| Field | Value |
|---|---|
| **ADR** | 028 |
| **Status** | **Proposed** (2026-06-24) — value-model decision recommended by the research review; **amends the input draft** (which placed `signed:bool` in the `Repr`). Maintainer ratifies → Accepted (house rule #3). |
| **Decides** | `Binary{width:u32}` stores a bitvector with **no signedness**; signedness is carried by **operations**, not the `Repr`. Shared ops (`add`/`sub`/`mul`/`neg`) are signedness-agnostic; signedness-dependent ops (`div`/`cmp`/`shift`/overflow-detect) are distinct named ops. |
| **Grounds** | RFC-0033 §4.1 (the normative statement); RFC-0032 D2 (the never-silent binary arithmetic prims already landing); SMT-LIB/Z3 (no signed/unsigned distinction in the bitvector *value* — only in the *operations*); G2 (never-silent overflow); `research/14-value-model-integration-report-RECORD.md` §3 (B-Binary). |
| **Date** | 2026-06-24 |

> **Posture (VR-5).** Proposed decision direction. The default is **not** to add `signed` to the
> `Repr`. Doing so would be a separate, explicit content-address one-way-door decision (§4.1.4). No
> implementation lands with this ADR.

## Context
General fixed-width integer/bitvector semantics. The input draft proposed `signed:bool` in the `Repr`.
Two's-complement `add/sub/mul/neg` are bit-identical for signed/unsigned; only division, comparison,
right-shift, and overflow detection differ.

## Decision
`Binary` **is** the bitvector; "signed integer" is an *interpretation* imposed by the op set (or a
higher typed view), not a property of the stored value. Two's-complement arithmetic and the
signedness-split ops are distinct named ops; the shared ones are signedness-agnostic. Never-silent
fixed-width overflow; arbitrary-precision integers are a `BigInt` ADT above the kernel (`Binary` does
not grow).

## Status
**Proposed (recommended)** — **amends** the input draft.

## Consequences
Smaller trusted surface; no address-space fragmentation (the *same bit pattern* keeps **one** content
address regardless of signed/unsigned interpretation). Matches SMT-LIB's shared `bvadd/bvsub/bvmul`
and split `bvsdiv/bvudiv`, `bvslt/bvult`, `bvashr/bvlshr`.

## Rejected
- **`signed` in `Repr`** — changes the content-address identity of every integer value and doubles the
  swap matrix; permitted only by an explicit superseding decision (**one-way door**, §4.1.4).
- **Two's-complement wrap on overflow** (never-silent violation); **bignum-growable `Binary`** (imports
  allocation into the kernel); a **separate `Int{width,signed}` repr** (redundant; doubles the matrix).

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-24 | **Proposed** | Initial record. Binary sign-free; signedness-as-operations (B-Binary, amends the input draft). Grounds RFC-0033 §4.1. |
