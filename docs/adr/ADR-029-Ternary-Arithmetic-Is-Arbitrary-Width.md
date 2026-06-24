# ADR-029 — Ternary arithmetic is arbitrary-width now

| Field | Value |
|---|---|
| **ADR** | 029 |
| **Status** | **Accepted** (2026-06-24 — maintainer-ratified, owner approval granted). The V0 `BigTernary` reference implementation **landed** (#535, M-754…M-757) — reconciled into `core::ternary` (DRY shared full-adder, never-silent fixed-width boundary), `cargo +1.92` fmt/clippy/test green — and reads *"implemented (Rust-first)"*. Was **Proposed** (2026-06-24). The limbed/Karatsuba perf paths (M-758/M-759) remain YAGNI follow-ons (no status of their own). |
| **Decides** | Balanced-ternary arithmetic is **arbitrary-width**: a digit-serial reference (`BigTernary`) that grows instead of overflowing, an optional limbed perf path proven bit-exact against it, a never-silent fixed-width boundary, and a non-redundant canonical form. Reconciled into **`crates/mycelium-core/src/ternary/`** (no new crate, no duplicate `Trit`). |
| **Grounds** | RFC-0033 §4.2 (the normative statement); the existing M-111 `core::ternary` codec + `docs/spec/swaps/binary-ternary.md` §1; G2 (never-silent fixed-width boundary); `research/14-value-model-integration-report-RECORD.md` §3 (B-Ternary) + `research/15-embeddonator-leverage-map-RECORD.md` §2 (the gap). |
| **Date** | 2026-06-24 |

> **Posture (VR-5 / honesty).** Mycelium's `core::ternary` is **already never-silent** about the
> ~40-trit fixed-width cap (`max_magnitude → None` at `m ≥ 41`; `add`/`mul → None` on overflow). This
> decision **removes the cap by adding a growable path** — it does *not* fix a silent bug in
> Mycelium's code. (The silent-overflow defect is `embeddonator`'s `dimensional::Tryte::max_value`, a
> different upstream codebase; that file is on the do-not-lift list.)

## Context
The current fixed-width path is i64-internal (exact to `m = 40`). A bignum need now exists (the value
model wants width-general ternary). `embeddonator`'s only variable-width path silently corrupts above
40 trits — the cautionary example, not a source.

## Decision
Ship a digit-serial `BigTernary` (obviously correct, never-overflowing oracle) reconciled into
`core::ternary` beside the existing fixed-width codec, reusing a DRY-extracted balanced full-adder
(`add_with_carry`). The growable path never overflows (carry-out becomes a new trit); the fixed-width
type's boundary is never-silent (`checked_to_width` / `checked_add_fixed` return `None`). A limbed
`PackedTernary` (40 trits/u64) is an explicit **YAGNI follow-on** gated on a benchmark and
differentially tested against the reference. The canonical form is non-redundant (no carry-save /
signed-digit redundancy), preserving content-addressing.

## Status
**Proposed (recommended). High priority.** The *capped* choice is the one-way door (it would force a
later `BigTernary` variant or a break).

## Consequences
Closes the precision ceiling. Binary↔ternary base conversion (2,3 coprime — no bit shortcut) lands in
the swap machinery; `LosslessWithinRange` gains a growable-vs-fixed distinction (RFC-0033 §6.1).
Algorithm-validated upstream (Python port + 10k fuzz vs arbitrary-precision int, incl. the `3^41`
case); the Rust lands only after the mandatory `cargo +1.92 test`/`clippy`/`fmt` gate.

## Rejected
**Keep-and-document the cap** (a real fixed-width ceiling once a bignum need exists); **digit-serial as
the production path** (memory/throughput — keep it as the equivalence oracle only); **carry-save /
signed-digit redundancy** (O(1) carry but multiple representations of one number — destroys canonical
content-addressing).

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-24 | **Accepted** | Maintainer-ratified (owner approval). The V0 reference implementation landed in **#535** (M-754…M-757): arbitrary-width `BigTernary` reconciled into `core::ternary` with a DRY shared `add_with_carry` (proven identical over all 27 inputs) + a never-silent fixed-width boundary (`FixedWidthTrits`/`checked_add_fixed`/`checked_to_width`); `cargo +1.92` fmt/clippy/test green, 11 new tests incl. the `3^41` cap-removal witness. `PackedTernary`/Karatsuba (M-758/M-759) stay YAGNI (bench-gated). |
| 2026-06-24 | **Proposed** | Initial record. Arbitrary-width balanced ternary (`BigTernary`), reconciled into `core::ternary`; removes the cap (core already cap-honest), `Limb27`/`PackedTernary` YAGNI follow-on (B-Ternary). Grounds RFC-0033 §4.2. |
