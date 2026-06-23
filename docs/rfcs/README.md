# RFCs

An RFC is a detailed, normative design of a subsystem that multiple decisions plug into. RFCs are **append-only** with status transitions and cite their grounding (survey `G*/A–E/R*`, research `T0/T1/T2`, and the ADRs they implement).

## Status set

`Draft/Proposed/Preliminary → Accepted → Enacted → Superseded` (the ratified lattice, #236; `Enacted` = an Accepted design now fully implemented/landed — it must step through `Accepted` first). RFC-0001…0005 are **Accepted** (solidified from the two research passes); **RFC-0006 is Accepted (r5, 2026-06-18)** and **RFC-0007 Accepted (r4)** — the layering / L1 v0 calculus are ratified, and the **KC-2 verdict (DN-09, 2026-06-18) = proceed** has now **committed the concrete L3 text surface** (+ a co-equal projection layer), discharging the prior KC-2 gate (RFC-0006 §10 Q1). **RFC-0017** (maturation scope & de-maturation) is **Enacted** (2026-06-21). The follow-on surface/type-system designs ratified 2026-06-18: **RFC-0018** (stage-1 grading) **Enacted** (2026-06-22); **RFC-0019** (traits/LR-2) **Enacted** (2026-06-23; stage-1 surface complete incl. M-673 monomorphization; literal runtime-dictionary form deferred); **RFC-0020** (L2 surface) **Accepted (scoped)**; **RFC-0021** (projections) **Enacted (framework)** (2026-06-21). **RFC-0016** (Core Library & Standard Library) is **Enacted** (2026-06-21; 25 std crates landed). **RFC-0008** is **Accepted**; **RFC-0009** and **RFC-0010** are **Enacted** (2026-06-23; M-350 done); the error-handling family **RFC-0011/0012/0013/0014/0015** is **Enacted** (Accepted designs landed in code).

## Index

| RFC | Title | Status | Implements / depends on |
|---|---|---|---|
| 0001 | Core IR & Metadata Schema | Accepted (r2) | Foundation FR-M*/VR-*; ADR-001/002/003/006/008; ADR-010 (§4.7); ADR-011 (§4.3 `Bound`) |
| 0002 | Swap Certificate & Split Regime | Accepted | RFC-0001; ADR-002; ADR-010; shares the certificate checker with RFC-0004 |
| 0003 | VSA Submodule Boundary | Accepted (r2) | RFC-0001; ADR-008; ADR-010; ADR-013 (r2 `spore` scope note) |
| 0004 | Execution Model & "Stable Component" | Accepted | RFC-0001; ADR-007; ADR-009; DN-01; shares the checker with RFC-0002 |
| 0005 | Selection-Policy Language | Accepted | RFC-0001; ADR-006; same mechanism used by RFC-0002 & RFC-0004 |
| 0006 | Surface Language, Grammar & Term-Language Layering | **Accepted** (r5; **concrete L3 text surface committed** — DN-09 KC-2 verdict, Q1 discharged; stage-1 grading → RFC-0018) | RFC-0001 §4.5; ADR-003/006/007; KC-2/KC-3; RR-3; DN-09; SPEC §10.1/§10.2 |
| 0007 | The L1 Kernel Calculus | **Accepted** (r4; v0 calculus; surface now committed via DN-09; `matured` granularity → RFC-0017; R7-Q1…Q4 deferred — §10) | RFC-0006; RFC-0001 §4.5/§4.6; RFC-0004 §4; ADR-003; T3.1/T3.4 |
| 0008 | Runtime & Concurrency Execution Model | **Accepted** (2026-06-16, maintainer sign-off; RT1–RT7 + §4 model normative; enactment staged — M-353/354/356/357) | RFC-0004 (extends, per-node model unchanged); RFC-0001/0002/0005/0006/0007; ADR-012 §7.3; T4.1–T4.6 |
| 0009 | Resonator-Network Factorization | **Enacted** (2026-06-23, M-350; `resonator.rs` present; never-silent `NoConverge` verdict; full convergence corpus deferred build-phase) | RFC-0003 §6; RFC-0001; FR-C2/G2/G4; RR-5/VR-5 (M-350) |
| 0010 | Decode-Methodology Selection | **Enacted** (2026-06-23, M-350; `decode_select.rs` present; ADR-015 closes §8; Hebbian default normative; other variants deferred) | RFC-0005 (same selection mechanism, third site); RFC-0009 (§10.3 matrix + regime gate); RFC-0003; G2/G4/VR-5 |
| 0011 | L0 `Match` & the L1-in-Core-IR Revision | **Enacted (r3)** (2026-06-15; r3 folded into RFC-0001 r3 + wired in lockstep — M-320/M-110/M-210) | RFC-0001 §4.5/§4.6 (r3 revision); RFC-0006 §4.4 step 2; RFC-0007 §4.1–4.6; ADR-003; M-320 |
| 0012 | Ambient Representation: Declared Paradigm Defaults & Scoped Overrides | **Enacted** (2026-06-16; §4 design landed in `mycelium-l1`/`mycelium-lsp` — M-344) | RFC-0006 (surface/term-layer; no kernel change); RFC-0005/ADR-006 (reified selection); RFC-0001 §4.5/§4.6 (content-addressing); ADR-016 (cross-module ABI); G2/KC-3/tension A; NFR-7 |
| 0013 | Structured Diagnostics & Reified Error-Handling Policy | **Enacted** (2026-06-16; §4 design landed in `crates/mycelium-lsp/src/diagnostics` — M-345) | DN-04 (basis); RFC-0005/ADR-006 (reified-policy pattern); RFC-0006 (optional surface); RFC-0008 (observability home); RFC-0001/0002 (the never-silent errors it presents); G2/G11/NFR-2/KC-3; ADR-007 |
| 0014 | Declarative Error Recovery & Bounded Effects | **Enacted** (2026-06-16; §4 design landed in `crates/mycelium-lsp/src/recover` — M-352/M-353) | RFC-0001 (error values); RFC-0013 (sibling — shared registry/pattern, presentation vs. recovery); RFC-0005/ADR-006 (reified policy); RFC-0006 (surface); RFC-0004 §4 / RFC-0007 §4.5 / M-347 / DN-05 (the budget discipline bounded effects generalise); G2/VR-5/KC-3/SC-3 |
| 0015 | Automatic Baseline Diagnostics & Recovery | **Enacted** (2026-06-16; baseline policy landed in `crates/mycelium-lsp/src/baseline.rs` — M-362) | RFC-0013 (additive presentation it auto-applies) + RFC-0014 (opt-in declared recovery); RFC-0005/ADR-006 (reified EXPLAIN-able policy); RFC-0008 §4.8/RFC-0013 §8 (sinks); G2/VR-5/KC-3/NFR-2 |
| 0016 | Core Library & Standard Library | **Enacted** (2026-06-21; 25 `mycelium-std-*` crates landed Rust-first, M-501–M-534/540/541; guarantee matrices asserted; G2/VR-5 hold; M-648) **Accepted** (2026-06-17, M-501/DN-07; §4.1 contract + taxonomy + ring layering + migration order; §8 Q3/Q4 deferred-with-direction; 2026-06-20 erratum — `spore` Ring 1) | M-346; RFC-0001; RFC-0002/0003/0005/0008/0013/0014; ADR-003/007/010/013/014; G2/G11/VR-5/KC-2/KC-3 |
| 0017 | Maturation Scope & De-maturation | **Enacted** (2026-06-21; `thaw`/scope-`matured` implemented + tested in `crates/mycelium-l1/`; DN-08 five questions closed; M-648) **Accepted** (2026-06-18; `matured` → nodule/phylum/program scope, `matured fn` retired, `thaw` de-maturation; supersedes RFC-0007 §4.5 *granularity*, gate soundness unchanged) | RFC-0007 §4.5; RFC-0004 §4; DN-06/DN-08; Nodule-Header spec; DN-02/DN-03; ADR-003; KC-3/G2/VR-5 |
| 0018 | Stage-1 Static Guarantee Grading | **Enacted** (2026-06-22, stage 1a landed in `mycelium-l1` — M-663; Accepted 2026-06-18, maintainer ratification; R18-Q1 = Design A; supersedes RFC-0007 §4.3; stages 1b/2 future RFCs; noninterference stays Declared-with-argument) | RFC-0007 §4.3/§4.4 (revision); RFC-0006 §8 Q3; RFC-0002 (Swap certificate = endorsement); LR-6; VR-5; T3.2 |
| 0019 | Traits & Parametric Polymorphism (LR-2) | **Enacted** (2026-06-23, M-657/M-659/M-662/M-673; stage-1 surface complete; monomorphization + static resolution EXPLAIN-reified; literal runtime-dictionary `Construct` form deferred; MPTC/assoc-types deferred to v2; coherence stays Declared-with-argument) · **Accepted** (2026-06-18) | RFC-0007 §4.4 (the deferral); RFC-0006 LR-2/LR-5/LR-6; ADR-003; DN-03; KC-3; T3.3 |
| 0020 | The L2 Surface Term Language | **Accepted (scoped)** (2026-06-18, maintainer ratification, DN-12; §4.1/4.3/4.4/4.6–4.9 core Accepted; §4.2/§4.5 + R20-Q1…Q5 carved out, deferred) | RFC-0006 §3 (L2); RFC-0007 (L1 target); RFC-0011/0012; DN-09 §3.2; LR-1/LR-3; ADR-003/006; KC-3 |
| 0021 | Semantic-Level Projection Framework | **Enacted (framework)** (2026-06-21; M-380 `LlmCanonical` renderer landed in `crates/mycelium-lsp/src/project.rs`, total over 11 L1 node kinds, P1–P6 unit-tested; LLM-leverage stays Declared/open in M-381; M-648) **Accepted (framework)** (2026-06-18; framework + P1–P6 + registry normative; G11 gate discharged; LLM-leverage carved out → RP-1/M-381, not asserted) | RFC-0006 §3/§9 (L3, FR-S5); DN-09 §3.1; RFC-0001 §4.6/ADR-003; ADR-006; FR-C1/G11; RR-3 |
| 0022 | Web-Tooling Phylum (HTTP client/server/routing/JSON) | **Accepted** (2026-06-21, maintainer ratification — RP-10 discharged clean; design agreed, Enacted gated on the `mycelium-web` build + E7-1/E7-2; `web.server` on the Mycelium runtime, IDNA pinned-at-build, HTTP/2-3+TLS+smuggling = v1 non-goals) **Draft** (2026-06-21; gated on RP-10 + E7-1/E7-2 — `web` phylum design; Rust-first plan; not yet built) | RFC-0016 §4.1 C1–C6 (per-op contract); RFC-0001 §4.8 (serialization); RFC-0008 RT1–RT7 (server-as-colony); RFC-0014 §4.5 (effects); ADR-007 (Rust-first); E7-1 (M-657/M-659); E7-2 (M-666) |
| 0023 | Agent Development Kit Phylum (`adk`) | **Accepted** (2026-06-21, maintainer ratification — RP-9 discharged + §3 concept-map repaired via §3.7 [ADK 2.0 graph Workflow Runtime; §3 pinned v2.3.0]; design agreed, Enacted gated on the `mycelium-adk` build + E7-1/E7-2; runtime `mycelium-mlir::runtime`, ToolError budget→`TaskOutcome::BudgetExhausted`, Session snapshot-v0) **Draft** (2026-06-21; gated on RP-9 + E7-1/E7-2 — ADK-port phylum design; Rust-first plan; not yet built) | RFC-0008 RT1–RT7 (colony/hypha model); RFC-0016 §4.1 C1–C6 (per-op contract); RFC-0021 (LlmCanonical projection); DN-09 (honest LLM-leverage posture); ADR-003 (content-addressed state); E7-1 (M-657/M-659); E7-2 (M-666); M-670 (web phylum HTTP/JSON transport) |

Cross-cutting machinery (decided across RFCs): **one** certificate checker (RFC-0002 ⇄ RFC-0004), **one** selection mechanism (RFC-0002 ⇄ RFC-0004), and **two** bound kernels meeting at one shared certificate (ADR-010 → RFC-0001/0002/0003). See `../Doc-Index.md` for the dependency DAG.

## Relationship to ADRs

An **ADR** records *a* decision; an **RFC** designs *a subsystem* and may rest on several ADRs. When an RFC makes a new architectural decision in passing, capture that decision as an ADR too, so it's discoverable from the ADR index.

## Template

```markdown
# RFC-NNNN — <title>

| Field | Value |
|---|---|
| **RFC** | NNNN |
| **Status** | Draft \| Proposed \| Preliminary \| Accepted \| Enacted \| Superseded |
| **Type** | Foundational / normative \| Informational |
| **Date** | YYYY-MM-DD |
| **Depends on** | <RFCs / ADRs / Foundation labels> |

## 1. Summary
## 2. Motivation
## 3. Guide-level explanation
## 4. Reference-level design (normative)
## 5. Drawbacks
## 6. Rationale & alternatives
## 7. Prior art
## 8. Unresolved questions
## 9. Future possibilities

## Meta — changelog
```
