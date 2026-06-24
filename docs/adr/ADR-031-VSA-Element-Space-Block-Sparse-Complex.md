# ADR-031 — VSA element space in Repr; block-sparse + complex carrier

| Field | Value |
|---|---|
| **ADR** | 031 |
| **Status** | **Accepted** (2026-06-24 — maintainer-ratified, owner approval granted). The element-space + block-sparsity + complex-carrier decision is **locked**; the implementation (`VsaElem` / `VsaSparsity::SparseBlock` in `Repr::Vsa`, `HypervectorC` `Payload` arm) is a **content-address one-way door** landing in the single E20-1 rehash (M-775 → M-780) **before any VSA value is persisted** (RFC-0033 §7). Was **Proposed** (2026-06-24); disagrees with the input draft. |
| **Decides** | Add `VsaElem{Binary,Bipolar,Integer,Real,Complex}` and `VsaSparsity{Dense, SparseGlobal{max_active}, SparseBlock{blocks, active_per_block}}` to `Repr::Vsa`, and a `HypervectorC(Vec<Complex<f64>>)` `Payload` arm. `model` selects the algebra over a carrier that can honestly store it. |
| **Grounds** | RFC-0033 §4.4 (the normative statement); RFC-0003 (VSA submodule boundary — models, sparsity); RFC-0001 §4.1 (`Vsa`/`SparsityClass`); KC-3 (the `Payload` enum is trusted — this growth is a correctness condition, §2.3(a)); `research/14-value-model-integration-report-RECORD.md` §3 (B-VSA) + `research/15-embeddonator-leverage-map-RECORD.md` §1 (`block_sparse.rs` reference impl). |
| **Date** | 2026-06-24 |

> **Posture (VR-5).** Proposed decision direction. **Content-address one-way door** (`Vsa.elem` and
> `Vsa.sparsity` are hashed into every VSA value's identity). It lands in the single rehash (RFC-0033
> §7 / M-780) before any VSA value is persisted. No implementation lands with this ADR.

## Context
`Hypervector(Vec<f64>)` + a global `Sparse{max_active}` today. The input draft held that `model:String`
extensibility is enough. But a model string **cannot retrofit a vector space the `Payload`/`Repr`
cannot hold** — the current shape structurally blocks at least three recognized families:
- **Complex (FHRR, MAP-C).** Complex unit-modulus phasors; `Vec<f64>` makes `dim` lie (2D for a D-dim
  HV) or stores only phases (wrong algebra) — either way the kernel silently misrepresents it.
- **Block-sparse (SBC, BSDC).** A *global* `max_active` cap cannot express block structure (which
  blocks, one-hot-per-block), so it cannot distinguish a valid SBC vector from an invalid one.
  `embeddonator`'s `block_sparse.rs` already implements this — proof the family is real.
- **Integer/graded (MAP-I).** Storable in `f64` but the element type can't be *declared* integer, so
  tags/swaps can't reason about exactness.

## Decision
Make the **element space** and **sparsity** explicit in `Repr::Vsa`, and add a complex `Payload`
carrier. The registry then chooses the algebra *on top of* a carrier that can honestly store it.
`SparseBlock{…, active_per_block}` covers both strict SBC (`=1`) and BSDC (`>1`).

## Status
**Proposed (recommended)** — **disagrees with** the input draft. **One-way door.**

## Consequences
Unblocks FHRR/MAP-C (complex), MAP-I (integer), BSDC/SBC (block-sparse) — three standard families the
old shape cannot honestly represent. Enlarges the trusted `Payload` (justified under §2.3(a):
correctness, not a feature gap). Changes content-address identity of every VSA value; needs
complex↔real (`Bounded`) and bipolar↔binary (`LosslessWithinRange`) swap certs. `block_sparse.rs` in
`embeddonator` is the reference impl for the lift (`crates/mycelium-vsa/`).

## Rejected
**Model-strings-only** (strings can't store complex or block structure — families stay
unrepresentable); **interleaved-reals-with-a-lying-`dim`** (never-silent violation); **drop
complex/block families** (FHRR and SBC are not fringe; the "complete coverage" goal fails).

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-24 | **Accepted** | Maintainer-ratified (owner approval). The VSA element-space / sparsity / complex-carrier decision is locked ahead of the single content-address rehash (RFC-0033 §7 / M-780); implementation (M-775…M-779) lands in E20-1 V4 before any VSA value is persisted. |
| 2026-06-24 | **Proposed** | Initial record. VSA element-space + block-sparsity + complex `Payload` carrier (B-VSA, disagrees with the input draft). `embeddonator/block_sparse.rs` is the reference impl. Grounds RFC-0033 §4.4. |
