# Mycelium — Project Foundation

**Version:** r3 (post-research, solidified)
**Date:** June 08, 2026
**Status:** Living document — append-only for decisions

---

## 1. Vision

Mycelium is a programming language that treats **traditional binary**, **balanced ternary**, **dense embeddings**, and **Vector Symbolic Architectures (VSA)** as co-equal, first-class substrates under semantics that are transparent, metadata-native, and formally reasoning-amenable.

The central thesis: **representation-swap** must be an explicit, verifiable, first-class operation. Every approximation must be disclosed, bounded, and tagged by how trustworthy that bound is.

## 2. Non-Negotiables

1. **No hidden / opaque behavior** in core semantics.
2. **Dual intelligibility** — human-readable *and* useful for AI agents.
3. **Formally reasoning-amenable** — "no black boxes" realized as mechanically-checkable invariants.

## 3. Scope (Phase 0–1)

- Core IR + metadata schema
- Representation swap with per-swap certificates
- VSA as optional submodule with honest bounds
- Hybrid execution (interpreter = reference; AOT via MLIR→LLVM)
- Schedule-staged packing for lossless layouts
- Total, auditable selection policies

Out of scope for Phase 1: full term language, module system, surface syntax, advanced projectional editing.

## 4. Requirements

### Functional Requirements (FR)
- **FR-M1** Representation paradigm is part of the type.
- **FR-M3** No implicit conversion between paradigms.
- **FR-M4** Every approximate operation carries a bound tagged by guarantee strength.
- **FR-M5** Metadata survives lowering and is queryable.
- **FR-S2** Definitions are content-addressable (Unison-style).

### Non-Functional Requirements (NFR)
- **NFR-3** No hidden behavior; all approximation is explicit.
- **NFR-6** Kernel must remain single-expert-auditable (KC-3).
- **NFR-7** Interpreter is the reference semantics and trusted base.

### Verification Requirements (VR)
- **VR-1/3** Static provability of declared bounds where feasible.
- **VR-4** Per-swap / per-lowering translation validation.
- **VR-5** Guarantee tags must be honest (`Proven` only when backed by checked theorem).

### Success Criteria (SC)
- **SC-3** Every representation change is an explicit `Swap` with certificate.
- **SC-4** Every IR stage is dumpable/diffable.

## 5. Architecture Anchors

- **MLIR** for progressive, inspectable lowering.
- **Unison** for content-addressed definition identity.
- **Apache Arrow** for self-describing metadata.

## 6. Key Decisions (Summary)

See the ADR log in §8 and the individual RFCs for details.

## 7. Kill Criteria (KC)

- **KC-1** VSA bundling bounds must be honestly statable (PASSED — Clarkson/Thomas proven bounds).
- **KC-2** LLM leverage on novel surface (OPEN — E4 experiment pending).
- **KC-3** Kernel remains single-expert-auditable.
- **KC-4** Avoid full verified compilation cost.

## 8. ADR Log

ADR-001 through ADR-009 live in this section (summary form). ADR-010 is broken out as its own file.

**ADR-001** Guarantee lattice + honesty propagation (`Exact ⊃ Proven ⊃ Empirical ⊃ Declared`).
**ADR-002** Split verification regime (bijective vs bounded).
**ADR-003** Content-addressed identity (Unison style).
**ADR-006** Reified selection policies (anti-opacity).
**ADR-007** Rust kernel + reference interpreter as trusted base.
**ADR-008** VSA as optional submodule.
**ADR-009** Hybrid execution (AOT for stable components).
**ADR-010** Two bound kernels (ε + δ) with shared certificate (Accepted).

## 9. Risk Register (selected)

- **RR-13** HRR/FHRR are the VSA weak link (Empirical tags only).
- **RR-14** Rust VSA ecosystem immature (build cost).

## 10. Recommended Immediate Next Actions

1. Liquid-Haskell `bundle` capacity-refinement probe (RFC-0003 §5).
2. Core IR + reference interpreter implementation.
3. Binary↔ternary `LosslessWithinRange` swap.
4. Single certificate checker.
5. `ternary` MLIR dialect + schedule-staged packing.
6. VSA submodule implementation.
7. LLM-leverage experiment (E4).

See the dependency-ordered list in the Doc-Index for full details.

---

**Meta**
This document is the single source of truth for charter, requirements, and high-level decisions. All RFCs and ADRs trace back to it.
