# Decision Records

Mycelium's design lives in an **append-only** corpus of decision documents. Status moves forward only
(`Draft / Proposed → Accepted → Enacted → Superseded`; notes `→ Resolved`); to change an
Accepted/Enacted decision you **supersede** it, never rewrite history (house rule #3). Every
normative claim cites its grounding or is marked an open question.

## Document kinds

- **RFC** (`docs/rfcs/`) — normative designs (the Core IR, swaps, selection, runtime, stdlib,
  tunable certification, memory management, …).
- **ADR** (`docs/adr/`) — architecture decision records (Rust kernel + MLIR, verified numerics, the
  spore unit, the unsafe-code policy, the north-star reframe, …).
- **DN** (`docs/notes/`) — design notes / deliberations that feed RFCs/ADRs (the lexicon, the memory
  model, tunable-certification deliberation, …).

The authoritative, per-document **status index** is
[`docs/Doc-Index.md`](https://github.com/tzervas/mycelium/blob/main/docs/Doc-Index.md). The
[Foundation](https://github.com/tzervas/mycelium/blob/main/docs/Mycelium_Project_Foundation.md) is the
charter (FR/NFR/VR, SC-*/KC-*, the roadmap).

## Orientation — some load-bearing decisions

- **RFC-0001** — Core IR & metadata schema (the value model). · **RFC-0002** — swap certificate +
  split regime. · **RFC-0005** — selection-policy language. · **RFC-0007** — the L1 kernel calculus.
- **RFC-0008 / ADR-020** — runtime & concurrency execution model.
- **RFC-0016** — core + standard library. · **ADR-010** — verified-numerics foundation.
- **ADR-007** — Rust kernel + MLIR. · **ADR-013** — the spore is the deployable unit. · **ADR-014** —
  unsafe-code policy.
- **ADR-022 / DN-25** — the dual-axis 1.0.0 release gates.
- **Direction (current):** **ADR-032** + **RFC-0034** — tunable certification & the transparency
  reframe (north star → fast, memory-safe, ergonomic; certification optional). See
  [Tunable Certification](Tunable-Certification).
- **Memory model:** **DN-32** (three-layer hybrid) + **RFC-0027** (reclamation) + **DN-33** (MEM-4
  static RC elision). See [Memory Model](Memory-Model).

## Indexes in the repo

- [`docs/rfcs/README.md`](https://github.com/tzervas/mycelium/blob/main/docs/rfcs/README.md) — the RFC
  table. ·
  [`docs/adr/README.md`](https://github.com/tzervas/mycelium/blob/main/docs/adr/README.md) — the ADR
  table. ·
  [`docs/Doc-Index.md`](https://github.com/tzervas/mycelium/blob/main/docs/Doc-Index.md) — everything,
  with status.
