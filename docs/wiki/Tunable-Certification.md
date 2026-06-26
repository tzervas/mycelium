# Tunable Certification & the Transparency Rule

The early design mandated the full certification/hashing machinery **unconditionally** — every value
tagged, every swap certified, everything content-hashed. **ADR-032** (Enacted) and **RFC-0034**
(Accepted) repositioned that: certification depth and *transparency* are separable, so the expensive
machinery becomes a **tunable policy** while transparency survives at every setting.

## The modes (RFC-0034)

Certification is a matrix of independent knobs grouped by compile/runtime phase, preset into
first-class modes:

- **`fast`** (default) — transparent, inspectable, *non-certified* auditability. Structural tags
  only; never overclaims.
- **`balanced`** — an optional intermediate.
- **`certified`** (opt-in per `global` / `phylum` / `nodule`) — reinstates the full machinery: swap
  certificates emitted + checked, bound proofs, pervasive content-hashing.

The **mode itself is never silent (G2)**: every result is mode-tagged, tooling surfaces the active
mode, and cross-mode composition is explicit. Scoping rides the RFC-0012 ambient + `mycelium-proj`
manifest/header resolver (`// @certification: …`), so *where did this level come from?* is always
answerable.

## The transparency & auditability rule

ADR-032 reframes the project's "honesty rule" vocabulary as **transparency & auditability** — a
wording change only; the **mechanism is unchanged**:

- **The guarantee lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`** travels with every value and
  degrades by *meet* through operations.
- **`Proven` is allowed only** with a theorem whose side-conditions are *checked*; otherwise a claim
  is `Empirical` (trials) or `Declared` (asserted, always flagged).
- **VR-5** — downgrade to stay accurate; never upgrade a tag without a checked basis.
- **G2** — never-silent: out-of-range is an explicit `Option`/error; selections/conversions/
  reclamations are reified and `EXPLAIN`-able.

`fast` is a *systematic, flagged downgrade* to structural tags — not a way to hide behavior. The
always-on mandates (SC-3 "zero swaps without a certificate", FR-M3, RFC-0001 §3.4/§4.6, RFC-0002 §2)
now hold **at the active mode**, with the mode never silent.

## Why

A fast, ergonomic default without forfeiting assurance where it matters; deployability decoupled from
certification (spores are mintable with the runtime cert off). Memory-safety, speed, and ergonomics
are first-class goals alongside the transparent-swap thesis.

## References

- [ADR-032](https://github.com/tzervas/mycelium/blob/main/docs/adr/ADR-032-Tunable-Certification-Supersedes-Always-On-and-Transparency-Reframe.md)
  (the charter reframe) ·
  [RFC-0034](https://github.com/tzervas/mycelium/blob/main/docs/rfcs/RFC-0034-Tunable-Certification-and-Transparency-Modes.md)
  (the mechanism) ·
  [Foundation §1](https://github.com/tzervas/mycelium/blob/main/docs/Mycelium_Project_Foundation.md)
  (the charter update).
