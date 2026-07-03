# Status & roadmap

One-line purpose: what's built, what's in progress, what's explicitly not-yet-established, and
the technology stack underneath it — kept honest per the transparency rule (VR-5: never upgrade a
claim past its checked basis).

## Contents

- [Status & open items](#status--open-items)
- [Native AOT — current state (epic E25-1 / ADR-034)](#native-aot--current-state-epic-e25-1--adr-034)
- [Technology stack](#technology-stack)

## Status & open items

**KC-1 (existential VSA risk): passed — and confirmed in a build.** Proven non-asymptotic VSA
bundling bounds exist (Clarkson-Ubaru-Yang 2023; Thomas-Dasgupta-Rosing 2021), and the
Liquid-Haskell `bundle` capacity-refinement probe (`proofs/lh-bundle/`, RFC-0003 §5) reports
**SAFE** (Z3 discharged), ratifying the axiomatized-theorem + checked-instantiation strategy.
VSA stays in core with `Proven` tags.

**KC-2 / LLM leverage: verdict Proceed (DN-09, Resolved 2026-06-18).** The experiment measured
weak-but-recoverable leverage (best local arm: 40% first-attempt → 70% eventual); the
frontier-model follow-up (DN-09 §§8–10) confirmed the grammar-in-context primer reaches 91.7%
pass@1 across 3 seeds, and the retention ratio (550%/220% for the two models) does not trigger
the RFC-0021 §4.7 projection-promotion threshold. The T3.6 full ablation (arms 3/5) is a
non-blocking research follow-up (M-381, backlogged per ADR-021 §5).

**Built (Phases 0–3, 5, 7, 8 complete):** the Core IR + Rust reference interpreter; the single
certificate checker; the certified binary↔ternary swap (Z3-proved); the verified-numerics layer
(ε/δ, `mycelium-numerics`); Dense/VSA breadth with per-model guarantee matrices; the
selection-policy engine + EXPLAIN; the direct-LLVM native path (recursion/closures/`Swap`/Dense/VSA,
see below); JIT (M-340) including dynamic-VSA/HDC JIT (M-855); hot-inject (M-341); the L1 calculus;
the runtime/concurrency model (RFC-0008); the full toolchain suite; and the Rust-first standard
library — **25/25 crate specs ratified to `Accepted`** (2026-06-20/21, DN-07 + maintainer
2026-06-21). The **E7-1 L1 stage-1 surface** is complete in the type-checker: generics
(M-656/M-657), trait/`impl` checker + coherence (M-658/M-659), effect annotations (M-660),
`wild`/FFI gate (M-661), phyla + cross-nodule model (M-662), and static guarantee grading
(M-663, RFC-0018 Enacted). **M-673 landed the monomorphization elaboration + dictionary-free
static trait resolution**, so generic and trait instantiations now elaborate to closed L0 and run
on all three paths (L1-eval ≡ L0-interp ≡ AOT); DN-14 §3 rows 6 and 7 are `present`.

**1.0.0 gate defined and ratified (ADR-021, Accepted 2026-06-21; amended by ADR-024 and
ADR-034).** Gate A1 (zero open High findings from the 2026-06-14 deep review) and Gate B2 (KC-2
verdict) are met. **ADR-034 (Accepted, 2026-06-30)** re-gates native AOT maturity (track T6 /
epic E15-1) back into the `lang 1.0.0` Definition of Done, with scope expanded to full-language
native-codegen coverage — see the next section for the honest current state of that work. Open
gate rows otherwise remain the critical path (DN-19): A2 (Medium-findings ledger), A3 (WS8
durability: `cargo-mutants`/proptest/fuzz), A4 (`cargo deny`/`cargo audit` wired into `just
check`). The 1.0.0 product scope is the kernel/core (interpreter, certified swaps, VSA/dense ops
with bounds, selection + EXPLAIN, the trusted toolchain); surface-language ratification is scoped
to a tracked `1.x`.

**Not yet established:** self-hosting (M-502) — the stdlib is Rust-first; the Mycelium-lang
migration half is open. Surface-language and self-hosting are post-1.0 / 1.x scope (ADR-021 §5).

Residual risks tracked in the Foundation risk register, notably **RR-13** (HRR/FHRR are the VSA
weak link). See `docs/Mycelium_Project_Foundation.md` §10 for the dependency-ordered action list
and `docs/planning/phase-*.md` for the live phase ladder.

## Native AOT — current state (epic E25-1 / ADR-034)

**Direction (maintainer decision, 2026-06-30, ADR-034 Accepted).** The Rust `1.0.0` release is to
be "fully implemented and fully featured," with native AOT completed and a **hard gate row** on
the `lang 1.0.0` Definition of Done (reversing ADR-022 §8 Q4's earlier `1.1` deferral) — scope
expanded to full-language native-codegen coverage, not only the bit/trit + bounded-data subset
that shipped at the earlier waveN2 milestone.

**Landed on `main` (epic E25-1, wave closing 2026-06-30/07-01) — each `Empirical` via a checked
three-way differential (interp ≡ AOT ≡ JIT), never `Proven` beyond the one checked single-op
MAP-I capacity bound:**

- **M-850** — direct-LLVM full recursion: non-tail `Fix` + mutual `FixGroup` via a heap-allocated
  control-stack trampoline, bounded by the shared `AutoDepthBudget`.
- **M-851** — direct-LLVM closure-ABI widening: closures over any repr/width, currying, and
  closure-valued results.
- **M-852** — direct-LLVM `Swap` native codegen, certificate-preserving.
- **M-853** — native Dense lowering: un-quantized `Repr::Dense` elementwise ops.
- **M-854** — native VSA lowering: MAP-I/BSC/HRR/FHRR `bind`/`unbind`/`bundle`/`permute`/`similarity`.
- **M-855** — JIT for dynamic VSA/HDC workloads (cleanup/resonator loops deferred).
- **M-856** — MLIR-dialect coverage catch-up: `Construct`/`Match`/`Swap` (libMLIR-gated, ADR-019).
- **M-857** — `trit.mul` through the real MLIR-dialect path (libMLIR-gated).
- **M-858** — the unified mutant-witnessed three-way differential over all the new fragments.

**Still open** (tracked, not silently dropped — G2): **M-856b** (MLIR-dialect Dense/VSA coverage —
the dialect path still refuses these; direct-LLVM already covers them), **M-859** (`mycelium-bench`
single- and multi-core scaling curves + baseline-regression gating — landed via PR #845; the
`issues.yaml` status field lagged the merge as of this writing, flagged rather than silently
corrected here), **M-860** (parallel AOT codegen: per-function independent lowering,
byte-identical emit), **M-861** (scheduler multicore work-stealing), **M-862** (parallel
pure-fragment interpreter/env-machine eval + differential), and **M-863** (the ratification act:
flipping ADR-034 → Enacted, RFC-0029 → Enacted, DN-15 → Resolved once the rest lands).

**What this means concretely:** the native path now natively compiles recursion, closures, the
`Swap` node, Dense, and VSA — not just the bit/trit + bounded-data fragment — and a JIT exists for
dynamic VSA/HDC workloads. **Full-language coverage per ADR-034's Definition of Done is not yet
met**: the MLIR-dialect path still refuses Dense/VSA (M-856b), and the parallelism/perf-eval
increments (M-860–M-862) plus the formal ratification (M-863) remain open. The interpreter
remains the trusted base and reference throughout (ADR-007/NFR-7); the native path is validated
against it, never the source of meaning. RFC-0029 stays `Accepted` (not `Enacted`) until the path
is complete and stable (house rule #3) — this section does not upgrade that status.

See [`docs/adr/ADR-034-Full-Language-1.0.0-Gate-T6-AOT-Re-Gating.md`](../adr/ADR-034-Full-Language-1.0.0-Gate-T6-AOT-Re-Gating.md)
for the full Definition of Done, and the [workspace map](workspace-map.md#compiler--execution) for
the `mycelium-mlir` and `mycelium-bench` crate summaries.

## Technology stack

- **Kernel + reference interpreter:** Rust (MSRV **1.96.1**, ADR-007/ADR-041). The interpreter is the
  trusted base and the reference semantics (`crates/mycelium-interp`).
- **AOT path:** **MLIR → LLVM** (`crates/mycelium-mlir`), confined to the performance path.
  Landed: the env-machine, the **direct-LLVM** native lowering across recursion, closures,
  `Swap`, Dense, and VSA (M-850…M-854), JIT (M-340) including dynamic-VSA/HDC JIT (M-855), and
  hot-inject (M-341). In progress toward ADR-034's full-language gate: MLIR-dialect Dense/VSA
  coverage (M-856b) and the parallel-codegen/perf increments (M-860–M-863) — see the AOT status
  section above.
- **VSA submodule:** Rust (`crates/mycelium-vsa`) — the MAP-I algebra + the per-model guarantee
  matrix (RFC-0003). Built as a first-class submodule with `Proven`/`Empirical` tags per
  model and per operation.
- **Verified numerics:** a FloVer-style certificate-checker-in-Rust — two assurance tiers, ε
  (affine arithmetic) and δ (union-bound/apRHL) sharing one `{ε, δ, strength}` certificate
  (`crates/mycelium-numerics`, ADR-010).
- **Proofs:** Z3/SMT2 (binary↔ternary injectivity, `proofs/binary-ternary-roundtrip/`) +
  Liquid Haskell (`proofs/lh-bundle/`, KC-1 confirmed).
- **Experiments / LLM harness:** Python **3.13 / 3.14**, **UV**, **pytest**, **codecov**
  (`experiments/`, `tools/llm-harness/`).

---

**See also:** [Workspace map](workspace-map.md) · [Decisions & reading order](decisions-and-reading-order.md) ·
[Guarantees & verification](guarantees-and-verification.md)

[← Back to README](../../README.md)
