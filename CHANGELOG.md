# Changelog

All notable changes to this project are recorded here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Dates are ISO-8601.

This project is in the **design phase**; "changes" here are to the documentation
corpus, not released software. Versioning will begin when the kernel does.

## [Unreleased]

### Added
- Initial **design baseline**: project charter (`docs/Mycelium_Project_Foundation.md`, r3),
  document index (`docs/Doc-Index.md`), five RFCs (RFC-0001…0005, all Accepted),
  ADR-010 (Accepted), design note DN-01 (Resolved), and two research records
  (`research/01`, `research/02`).
- Repository scaffolding: `README.md`, `LICENSE` (MIT), `CONTRIBUTING.md`,
  `.gitignore` (Rust + Python), and index/process READMEs for `docs/adr/` and `docs/rfcs/`.

### Changed (baseline-review consistency pass)
- ADR-001 promoted to firmly **Accepted**; the "no statistical approximation vs
  fully-disclosed approximation" definitional question recorded as **settled**
  (fully-disclosed), consistent with the KC-1 pass and the guarantee lattice.
- Foundation §5.2 core-model sketch marked **superseded by RFC-0001** (packing is
  now schedule-staged, not in the type; guarantee lattice is the four-point form).
- Foundation §5.6 updated: **MLIR→LLVM** recorded as the committed AOT path
  (ADR-007 / RFC-0004), not a candidate.
- Foundation §6 Phase 0 annotated with post-research status (largely complete;
  remaining: the Liquid-Haskell `bundle` probe and the KC-2 LLM-leverage experiment).
- `README.md` decisions table: fixed a placeholder reference for the
  "no implicit conversion" rule (grounded in RFC-0001 §3.3 / FR-M3).
- `docs/Doc-Index.md`: the two research rows now point to the in-repo records.

### Open
- One confirming build: the Liquid-Haskell `bundle` capacity-refinement probe (RFC-0003 §5).
- One existential question: **KC-2 / LLM leverage** (the E4 experiment) — not yet settled.
- Decomposed task/issue set and phase planning documents — *forthcoming* (`docs/planning/`).
