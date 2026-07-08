# Deferred-Design CU Gates — Issue-Spec Source

| Field | Value |
|---|---|
| **Status** | **Living snapshot** (2026-07-08) — the deferred design gates surfaced by the kernel prim-gap (CU) wave and the trx2 Lane A1/A2 transpiler work. Regenerate/extend as the corpus moves. |
| **Method** | `Declared` — hand-authored issue specs (title · user story · DoD · `doc_refs` · `depends_on`), grounded in the cited decisions. Not yet minted into `tools/github/issues.yaml`. |
| **Purpose** | Give the PM-sync a **durable, reviewable spec source** to mint each gate into `issues.yaml` with a verified-free id (mitigation #1). This doc does NOT edit `issues.yaml` (script-owned) and mints nothing itself. |

> **Posture (VR-5 / G2).** Every entry below is a **design gate** — it needs a decision / RFC / DN
> *before* implementation, not a ready-to-code task. Statuses are proposed (`needs-design` unless a
> concrete blocker is named); the PM-sync confirms and assigns the real id. No decision is moved by this
> doc (house rule #3). The `depends_on` blockers (memory-model RFC, the E20-1 content-address
> settlement) are the maintainer's to schedule — this doc records them, it does not resolve them.

---

## How to use this

Each `## D<n>` block is one issue spec. At sync time: verify a free `M-xxx` slot
(`grep "id: M-xxx" tools/github/issues.yaml`), mint the row with the proposed `kind`/`status`, copy the
`user story` + `DoD` + `doc_refs` + `depends_on`, and mark this doc as the spec source. The `needs-design`
gates (D1/D4/D5) are actionable now (no external blocker); the `blocked` gates (D2/D3/D6) wait on their
named blocker and should be tracked, not scheduled. **D5 is the highest-leverage** — it unlocks the
transpiler float↔int cast emission that Lane A1 (#1311) deliberately gapped.

---

## D1 — CU-6 rotate / reverse (bit-rotation surface)

- **kind:** task · **status:** needs-design · **family:** kernel prim surface (CU-1/CU-4/CU-6 lineage)
- **user story:** As a kernel/stdlib author, I want `rotl`/`rotr` (and bit-reverse) over `Binary{N}`, so that
  rotation-based algorithms (hashing, crypto, SIMD-lane ops) have a faithful, never-silent surface.
- **the gate:** the naive `or(shl_u, shr_u)` is wrong — it mishandles `n = 0` (a full-width `shr` refuses), so a
  correct rotate needs either dedicated `bit.rotl`/`bit.rotr` kernel prims or a width-reflection construct.
  Decide: new prims vs. a derived width-safe surface (`std.math` FLAG-math-3).
- **doc_refs:** `corpus:DN-34#8.16`, `src:lib/std/math.myc`, `src:crates/mycelium-l1/src/checkty.rs`
- **DoD:** a ratified rotate surface (prim or derived) with never-silent `n = 0` / `n >= N` semantics, plus
  property tests (rotate-by-`N` is identity; `rotl` then `rotr` is identity) exercised three-way (L1/L0/AOT).

## D2 — CU-8 atomics (fetch_add & the memory-model gate)

- **kind:** task · **status:** blocked · **depends_on:** a memory-model RFC (does not exist yet)
- **user story:** As a concurrency author, I want atomic read-modify-write ops (`fetch_add`, CAS, …), so that
  lock-free / shared-state code has defined semantics.
- **the gate:** atomics are meaningless without a **memory model** — this is an RFC, not a prim add. It depends on
  a memory-ordering decision (DN-32 §7 / RFC-0027 §12).
- **doc_refs:** `corpus:DN-32#7`, `corpus:RFC-0027#12`, `corpus:DN-34#8.16`
- **DoD:** a memory-model RFC (Accepted), then an atomics surface with per-op ordering and never-silent semantics.

## D3 — CU-9 Dense dtype / quantization surface

- **kind:** task · **status:** blocked · **depends_on:** the E20-1 content-address settlement (ADR-030)
- **user story:** As a numerics / VSA author, I want a Dense dtype plus a quantization surface, so that dense and
  quantized tensors have a first-class, honesty-tagged representation.
- **the gate:** rides the **E20-1 content-address rehash** (ADR-030, RFC-0033 §4.3.2) — the growable / quantized
  `Repr` payload changes identity. The maintainer's `vsa_checks` branch carries the desktop durability numbers to
  ground it. Coupled to DN-90 §5 (the growable-tier design gate).
- **doc_refs:** `corpus:RFC-0033#4.3.2`, `corpus:ADR-030`, `corpus:DN-90`, `corpus:DN-34#8.16`
- **DoD:** post-E20-1, a Dense dtype / quant surface with honesty-tagged precision and never-silent conversions.

## D4 — CU-3 signed int ↔ float conversions

- **kind:** task · **status:** needs-design
- **user story:** As a stdlib author, I want checked signed-int ↔ float conversions, so that signed numeric code has
  a faithful, never-silent conversion surface (the unsigned `width_cast` slice landed in Lane A1, #1311).
- **the gate:** the CU-3 kernel prims (`flt.to_bin` / `bin.to_flt`) are unsigned / two's-complement-unaware; signed
  conversions need a decided prim or surface. Distinct from the lossy-swap gate (D5).
- **doc_refs:** `corpus:ADR-040#2.4`, `corpus:DN-34#8.17`, `corpus:DN-34#8.18`
- **DoD:** a ratified signed-conversion surface, checked and never-silent, property-tested three-way.

## D5 — CU-3 lossy-swap machinery (the float↔int cast fidelity gate)

- **kind:** task · **status:** needs-design · **raised by:** Lane A1 (#1311)
- **user story:** As a transpiler / stdlib author, I want a reified **lossy conversion swap** (rounding /
  saturating), so that Rust-style `as` float↔int casts have a faithful Mycelium form — today Lane A1 gaps them
  `PENDING-DESIGN(CU-3-fidelity)` because the checked prims refuse where `as` rounds / saturates.
- **the gate:** ADR-040 §2.4/§5 says the lossy conversion is a **reified swap carrying its bound, not a prim** — the
  swap surface / name / `SwapCertificate` shape is undecided. This also folds in **FLAG-cast-narrow-fidelity** (Rust
  wrapping-truncate has no faithful never-refusing prim; a wrapping-narrow needs deciding — prim vs. a `wrapping { }`
  extension vs. a swap).
- **doc_refs:** `corpus:ADR-040#2.4`, `corpus:ADR-040#5`, `corpus:DN-34#8.18`, `src:crates/mycelium-transpile/src/emit.rs`
- **DoD:** a ratified lossy-swap surface (rounding, saturating, wrapping-truncate) with a per-swap honesty tag and an
  EXPLAIN certificate; then the transpiler `Expr::Cast` float↔int and narrow arms flip from gap to emit.

## D6 — CU-7 growable ternary value form (BigTernary surfaced)

- **kind:** task · **status:** blocked · **depends_on:** the E20-1 content-address settlement (ADR-030)
- **user story:** As a ternary author, I want an arbitrary-width (growable) ternary value, so that ternary
  arithmetic is not capped at the ~40-trit fixed width — the digit-serial `ternary::add` / `mul` are already
  arbitrary-width, and `BigTernary` (M-756 / ADR-029) is built but unsurfaced as a value form.
- **the gate:** the growable `Repr::Ternary` payload couples to **E20-1** (ADR-030 one-way doors); held for the
  single content-address rehash. This is DN-90's central decidable-vs-held split (§5 / §6).
- **doc_refs:** `corpus:ADR-029`, `corpus:RFC-0033#4.2.2`, `corpus:DN-90`, `corpus:DN-34#8.16`
- **DoD:** post-E20-1, a surfaced growable ternary value form with never-silent overflow-to-grow and property tests.

---

## Summary map

| Gate | CU | Status | Blocker | Note |
|---|---|---|---|---|
| D1 | CU-6 rotate | needs-design | — | prim vs. derived; `n = 0` correctness |
| D2 | CU-8 atomics | blocked | memory-model RFC | RFC, not a prim add |
| D3 | CU-9 Dense | blocked | E20-1 settlement | DN-90 §5 coupled |
| D4 | CU-3 signed conv | needs-design | — | distinct from D5 |
| D5 | CU-3 lossy-swap | needs-design | — | **highest-leverage** — unlocks A1 cast emission |
| D6 | CU-7 growable ternary | blocked | E20-1 settlement | DN-90 §6 coupled |

DN-90 (#1310) is the umbrella design note for the E20-1-coupled set (D3 / D6) and the size-tier strategy.

## Meta — changelog

- **2026-07-08 — Created (trx2 Session-5).** Durable spec source for the six deferred-design CU gates surfaced by
  the kernel prim-gap wave and the Lane A1/A2 transpiler work, so the PM-sync can mint each into `issues.yaml`
  against a stable reference. `Declared`; mints nothing itself; no `issues.yaml` edit (script-owned). Append-only.
