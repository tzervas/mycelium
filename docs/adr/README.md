# Architecture Decision Records (ADRs)

An ADR captures a single architectural decision, its context, and its consequences. ADRs are **append-only**: to change one, supersede it with a new ADR and link forward.

## Where ADRs live

- **ADR-001 … ADR-009** are recorded in the project charter, **`../Mycelium_Project_Foundation.md` §8** (they were authored together as the founding decision log).
- **ADR-010 and onward** are **standalone files in this directory**, one per file (`ADR-NNN-short-title.md`).

New ADRs go here as standalone files. (ADR-001…009 may be extracted into individual files later for uniformity; until then, the Foundation §8 is authoritative for them.)

## Status set

`Proposed → Accepted → Enacted → Superseded`. ADRs 001–014/016/017 are **Accepted**; **ADR-015/ADR-018/ADR-019** are **Enacted** (2026-06-23, maintainer-approved); **ADR-020** is **Enacted** (M-521 v0 R1 implementation landed 2026-06-20). **ADR-021** (kernel 1.0.0 gate) is **Superseded by ADR-022** (2026-06-23; kernel gate preserved as track T1). **ADR-022** (full-language 1.0.0 gate, dual-version) is **Accepted** (2026-06-23, supersedes ADR-021). **ADR-023** (stability and API-compatibility guarantees) is **Accepted** (2026-06-23; corrected 2026-06-25 from a stale "Draft" in this index — the ADR-023 header + the table row below are Accepted). **ADR-034** (native AOT re-gated INTO `lang 1.0.0`) is **Accepted** (2026-06-30; amends ADR-022 T6 — reverses §8 Q4 append-only). **ADR-035** (T4 stdlib-in-Mycelium narrowed to the stable-API freeze + core-lib self-host slice) is **Accepted** (2026-07-01; amends ADR-022 T4 — narrows §8 Q1's T4 reading append-only). **ADR-036** (dogfooding + public-release strategy) is **Accepted** (2026-07-01; additive — the `lang 1.0.0` tag is cut on Rust, self-hosting gates it only at ADR-022 §8 Q1's core-lib slice, unchanged; comprehensive dogfooding is a parallel, non-tag-gating track that instead gates the project's separate, later public release). **ADR-037** (spore-registry remote backend: GHCR/OCI dense-map) is **Enacted** (2026-07-01; stepped through Accepted the same day; M-871/E26-1 — this row added 2026-07-01 with ADR-038's indexing, closing an index-coverage gap). **ADR-038** (pragmatic dogfooding — the function-first release strategy) is **Accepted** (2026-07-01; maintainer-ratified — the strategy binds; §2.4 refinement + D1 supersession in force; **FLAG-V1/V2 resolved** — public release sub-`1.0.0`/`0.x` at usability, `1.0.0` = fully-rewritten-where-appropriate; →Enacted only when both phase gates are reached), refining ADR-036 §2.4's public-release gate to functional usability, **version-independent** (public flips at a `0.x`, well before `1.0.0`; the public semver tracks the Mycelium rewrite, `1.0.0` ≡ fully-rewritten + 100% operational — §2.8; version stays `0.0.0` for now, semver scheme deferred), and fixing compiler self-hosting as a deferred, conditional, post-public aspiration, recording ADR-036's supersession of RFC-0031 §5 D1's compiler-forever permanence; execution doctrine reserves Fable-class models solely for planning/complex-design with implementation on Opus/Sonnet/Haiku scoped to complexity).

## Index

| ADR | Title | Status | Location |
|---|---|---|---|
| 001 | Tension-B framing: exact-spec + proven/declared bound (no *hidden* approximations) | Accepted | Foundation §8 |
| 002 | Split verification regime (bijective binary↔ternary; bounded/probabilistic VSA) | Accepted | Foundation §8 |
| 003 | Architecture anchors = MLIR ⊕ Unison ⊕ Arrow | Accepted | Foundation §8 |
| 004 | Embeddenator = provisional inspiration, not reference | Accepted | Foundation §8 |
| 005 | Ternary = logical substrate now, native hardware later | Accepted | Foundation §8 |
| 006 | Representation-selection policies are reified, inspectable artifacts | Accepted | Foundation §8 |
| 007 | Kernel in Rust (reference interpreter); MLIR→LLVM for AOT; tooling in Python | Accepted | Foundation §8 |
| 008 | VSA is in core semantics but packaged as an optional submodule | Accepted | Foundation §8 |
| 009 | Hybrid execution; AOT preferred for stable components; interpreter is reference | Accepted | Foundation §8 |
| 010 | Verified-numerics foundation: two bound kernels (ε / δ) + shared certificate | Accepted | `ADR-010-Verified-Numerics-Foundation.md` |
| 011 | `BoundBasis` is a property of every `Bound` (not just `CapacityBound`) | Accepted | `ADR-011-BoundBasis-Is-Universal.md` |
| 012 | Layered lexicon (Surface/Runtime/Formal tiers) + fungal runtime vocabulary | Accepted | `ADR-012-Layered-Lexicon-and-Fungal-Runtime-Model.md` |
| 013 | `spore` is the deployable unit; the reconstruction manifest is one component | Accepted | `ADR-013-Spore-Is-The-Deployable-Unit.md` |
| 014 | `unsafe` Rust: permitted-but-warned (explicit, justified, dev-warned, release-silenceable), not forbidden | Accepted | `ADR-014-Unsafe-Code-Policy.md` |
| 015 | `DEFAULT_ENUM_BUDGET = 4096` (guarantee-maximal): the RFC-0010 decode selector defaults to the `Exact` arm across the whole validated envelope, not the cost-optimal ≈128 | **Enacted** (2026-06-23, M-350; applied in `decode_select.rs`) | `ADR-015-decode-enum-budget-default.md` |
| 016 | The interpreted↔compiled ABI: dispatch a compiled definition by its content hash; cross values in the self-describing wire form (RFC-0004 §10 OQ-1) | Accepted | `ADR-016-Interpreted-Compiled-ABI.md` |
| 017 | Hot-inject recompiled definitions: hash-keyed dispatch + content-addressed dynamic linking, immutable-by-construction (RFC-0004 §10 OQ-2) | Accepted | `ADR-017-Hot-Inject-Recompiled-Definitions.md` |
| 018 | Versioning policy: per-crate `0.x` SemVer + source-only distribution (no crates.io publish in the design phase); CHANGELOG `[Unreleased]` → release-cut mapping | **Enacted** (2026-06-23, M-383/M-384; workspace pinned `0.0.0`; release-plz dry-run wired; first release cut deferred) | `ADR-018-Versioning-Policy.md` |
| 019 | libMLIR toolchain: the version-matched build dependency of the off-by-default `mlir-dialect` feature; provisioned via `scripts/setup-mlir.sh` (`just setup-mlir`); resolves M-348 on Linux | **Enacted** (2026-06-23, M-348/M-603; `setup-mlir.sh` present; `mlir-dialect` OFF by default; real dialect lowering present M-601) | `ADR-019-libMLIR-Toolchain.md` |
| 020 | `runtime`/`colony` phylum placement: dedicated `runtime` phylum + thin `std.runtime` facade; construct-by-construct activation at the Phase-7 gate; v0 API surface for the landed R1 slice (M-521) | **Enacted** | `ADR-020-Runtime-Colony-Phylum-Placement.md` |
| 021 | 1.0.0 release-readiness gate: kernel/core 1.0.0 criteria (Gate A honesty-integrity/durability; Gate B decision/external — KC-2 now met); surface language scoped to `1.x`; Phase-3+ maturation out of scope | **Superseded by ADR-022** (2026-06-23; kernel gate preserved as track T1 within the full-language program) — archived at `docs/archive/adr/ADR-021-1.0.0-Release-Readiness-Gate.md` (2026-07) | `ADR-021-1.0.0-Release-Readiness-Gate.md` |
| 022 | Full-Language 1.0.0 Release-Readiness Gate: dual-version core⟂lang — kernel track T1 (preserves ADR-021 Gate A/B) + full-language program tracks T2–T9; 9 epics E10-1…E18-1; DN-25 program map | **Accepted** (2026-06-23; supersedes ADR-021; **T1 scope amended by ADR-024**; **T4 scope amended by ADR-035**; **T6 re-gated by ADR-034**) | `ADR-022-Full-Language-1.0.0-Release-Gate.md` |
| 023 | Stability and API-Compatibility Guarantees: four-layer stability scope (surface · Core-IR · LSP wire · Rust crate API) with explicit carve-outs; dual-version model (full-language 1.0.0 = a release-event, not a crate version); release-based never-silent deprecation; MIT-only legal-readiness gate | **Accepted** (2026-06-23; M-737 — was Proposed/Draft 2026-06-23; →Enacted at the full-language 1.0.0 tag) | `ADR-023-Stability-and-API-Compatibility-Guarantees.md` |
| 024 | Core 1.0.0 Gate (Track T1) Scope Amendment: adds epic E19-1 (the RFC-0032 self-hosting-enablement surface — `eq`/`lt` prims, binary arithmetic, `Repr::Seq`, `Repr::Bytes`) to T1's Definition of Done, so the stdlib is fully `.myc`-self-hosted at the `core 1.0.0` tag; ADR-021 Gate A/B rows unchanged; M-703 waits on E19-1 | **Accepted** (2026-06-23; amends ADR-022 T1; →Enacted with T1 at the `core 1.0.0` tag) | `ADR-024-Core-1.0.0-Gate-T1-Scope-Amendment.md` |
| 034 | Full-Language 1.0.0 Gate (Track T6) Re-Gating: **reverses ADR-022 §8 Q4** — re-gates epic E15-1 (native AOT) INTO `lang 1.0.0` as a hard gate row, scope expanded to full-language native-codegen coverage (closures · recursion · `trit.mul` · `Swap` · Dense · VSA · dynamic-VSA JIT); umbrella E25-1 (M-850…M-863); M-738 waits on E15-1; ADR-022 §3/§5/§8 Q4 carry append-only re-gating pointers | **Accepted** (2026-06-30; amends ADR-022 T6; →Enacted with ADR-022 at the `lang 1.0.0` tag) | `ADR-034-Full-Language-1.0.0-Gate-T6-AOT-Re-Gating.md` |
| 035 | Full-Language 1.0.0 Gate (Track T4) Scope Amendment: **narrows** ADR-022 T4 to the documented stable-API freeze (DN-66) + the core-lib self-host slice (M-714…M-719); full RFC-0031 §5 D6 Rust-crate retirement for all 26 `mycelium-std-*` crates is deferred to the post-1.0 long-term arc (§10); ADR-022 §5 T4/§8 Q1 carry append-only "narrowed by ADR-035" pointers | **Accepted** (2026-07-01; amends ADR-022 T4; →Enacted with ADR-022 T4 at the `lang 1.0.0` tag) | `ADR-035-T4-Stdlib-in-Mycelium-Scope-Amendment.md` |
| 036 | Dogfooding and Public-Release Strategy: the `lang 1.0.0` **tag** is cut on the Rust reference implementation (self-hosting gates it only at the existing core-lib slice, ADR-022 §8 Q1 — unchanged); **comprehensive dogfooding** (all of Mycelium rewritten in Mycelium, beside the Rust originals) is a first-class within-1.0.0, non-tag-gating, parallel track (E18-1); each reimplementation is Rust≡Mycelium differential-validated and replaces its Rust original only once maintainer-satisfied; the repo stays **private** until dogfooding is complete + validated — the **public release** is a distinct, later milestone from the tag. Additive (does not amend ADR-022 §5/§8 Q1 criteria); append-only pointers only at §8 Q1/§10. **§2.4's release gate refined by ADR-038 (Accepted, 2026-07-01)** — append-only pointer at §2.4, in force | **Accepted** (2026-07-01; additive — does not amend ADR-022 §5; →Enacted at the public-release milestone, not the tag) | `ADR-036-Dogfooding-and-Public-Release-Strategy.md` |
| 037 | Spore Registry v0 Remote Backend: a published spore is an **OCI 1.1 artifact** in **GHCR** (`ghcr.io/<owner>/<phylum>:<version>`), decomposed into the DN-28 dense-map (source object → BLAKE3-addressed OCI blob; dense-map DAG → config blob; `name@version` → tag); resolve is fetch-and-verify (never-silent, G2); `oras` as the v0 transport behind an `OciTransport` trait. Realizes DN-28's direction; M-871/E26-1 | **Enacted** (2026-07-01; stepped through Accepted same day; local + live-GHCR round-trip demonstrated) | `ADR-037-Spore-Registry-v0-Remote-Backend-GHCR-OCI-Dense-Map.md` |
| 038 | Pragmatic Dogfooding — the Function-First Release Strategy: North Star **"Rust where appropriate, Mycelium everywhere else"** (pragmatic, not zero-Rust dogmatism); **Phase I → reach functional usability, then go public at a `0.x`** (the public release is gated on usability and is **version-independent** — happens well before `1.0.0`, §2.8; below-grammar enabler closure; private + `0.0.0` + `publish = false` until the flip); **versioning axis (§2.8):** the public semver **tracks the Mycelium rewrite**, `0.x → 1.0.0` in the open, **`1.0.0` ≡ fully-rewritten + 100% operational** (Phase II terminal); **for now stay `0.0.0`, semver scheme deferred to publish-time**; **Phase II → post-public progressive rewrite** (compiler self-hosting deferred/conditional — only if stability/perf-proven, after transpiler polish); refines ADR-036 §2.4 (release gate: usability + version-independent, not Rust-replacement; §2.1–§2.3 unchanged); records ADR-036's supersession of RFC-0031 §5 D1's compiler-forever permanence; transpiler doctrine (progressive hardening · pre-port polish · manifest-transcoding only where ROI-positive); float route (ii) (scalar-float `Repr` via future float ADR + DN-39 review, single rehash with ADR-030/031, deferred to first value-persistence); **execution doctrine — Fable reserved solely for planning/complex-design, implementation on Opus/Sonnet/Haiku scoped to complexity**, mandatory PM prep; **FLAG-V1** (`lang 1.0.0` label collision) + **FLAG-V2** (`1.0.0` vs compiler self-hosting) resolved by the maintainer 2026-07-01 (public release sub-`1.0.0`/`0.x` at usability; `1.0.0` = fully-rewritten-where-appropriate). Companion roadmap: `docs/planning/road-to-1.0.0-and-mycelium-rewrite.md` | **Accepted** (2026-07-01; maintainer-ratified — the strategy binds; §2.4 refinement + D1 supersession in force; →Enacted only when both phase gates are reached and checked) | `ADR-038-Pragmatic-Dogfooding-Function-First-Release-Strategy.md` |

## Template

```markdown
# ADR-NNN — <short title>

| Field | Value |
|---|---|
| **ADR** | NNN |
| **Status** | Proposed \| Accepted \| Superseded |
| **Date** | YYYY-MM-DD |
| **Supersedes / Superseded by** | (links, if any) |

## Context
<the forces at play; cite grounding labels: G*, A–E, R*, T0/T1/T2>

## Decision
<the decision, stated plainly>

## Consequences
<positive and negative; what this enables and what it costs>

## Grounding
<survey/research references>
```
