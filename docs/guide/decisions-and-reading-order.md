# Key decisions at a glance, and suggested reading order

One-line purpose: the load-bearing decision subset in table form, and a recommended path through
the corpus for a new reader.

## Contents

- [Key decisions at a glance](#key-decisions-at-a-glance)
- [Suggested reading order](#suggested-reading-order)

## Key decisions at a glance

| Decision | Where | Summary |
|---|---|---|
| Guarantee lattice + transparency propagation | RFC-0001; ADR-001 | `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`, meet on compose |
| No implicit conversion; explicit `Swap` only | RFC-0001 §3.3; FR-M3 | the central transparency rule |
| Split verification regime | RFC-0002; ADR-002 | provable binary↔ternary; bounded/probabilistic VSA |
| One certificate checker, two uses | RFC-0002 + RFC-0004 | swaps *and* interpreter-vs-compiled equivalence |
| Binary↔ternary = `LosslessWithinRange` | RFC-0002 | total bijection impossible at fixed widths; `Option`-typed, never silent |
| VSA in core, but an optional submodule | RFC-0003; ADR-008 | lean kernel (type slot only); opt-in algebra |
| Per-model × per-op guarantee matrix | RFC-0003 | tags; HRR/FHRR unbind is the weak link |
| Sparsity as static refinement | RFC-0001 §4.4; RFC-0003 §5 | declared class checked by SMT; capacity = axiomatized theorem + checked instantiation |
| Verified-numerics: two bound kernels | ADR-010 | `ErrorBound` (ε, affine arith.) + `ProbBound` (δ, union-bound/apRHL), shared certificate |
| Hybrid execution; interpreter = reference | RFC-0004; ADR-009 | AOT (MLIR→LLVM) for stable components; interpret/JIT for dev/dynamic |
| Packing is schedule-staged, not typed | DN-01; RFC-0004 §5 | cost-model selector over a small fixed set (I2_S/TL1/TL2) |
| Selection policy is total + EXPLAIN-mandatory | RFC-0005; ADR-006 | non-learned, deterministic, auditable; no cardinality-estimate black box |
| Surface/term-language layering (L0–L3) | RFC-0006; RFC-0007 | syntactic transparency invariants; the ten-node L1 kernel calculus; L3 = committed text syntax and a co-equal projection layer (M-380, KC-2 verdict) |
| Runtime & concurrency model | RFC-0008 | RT1–RT7; deterministic-fragment-first; partial failure explicit; probabilistic guarantees |
| Structured diagnostics + declarative recovery | RFC-0013; RFC-0014 | additive over the never-silent error (never substitutive); declared, **bounded** effects |
| Standard-library scope + per-op contract | RFC-0016 | C1–C6 (never-silent, guarantee tags, EXPLAIN, content-addressed, above-the-kernel, bounded effects); ring layering; 25/25 specs Accepted |
| `spore` is the deployable unit | ADR-013 | content-addressed code + values + metadata; identity is the content hash (ADR-003) |
| Interpreted↔compiled ABI + hot-inject | ADR-016; ADR-017 | hash-keyed dispatch; content-addressed dynamic linking; immutable-by-construction |
| 1.0.0 release-readiness gate | ADR-021 | Gate A (honesty-integrity + durability) + Gate B (decision/external); kernel/core scope; surface → 1.x |
| Full-language 1.0.0 gate, dual-axis versioning | ADR-022 | tracks T1–T9 across surface completeness, runtime, stdlib-in-Mycelium, FFI, toolchain, docs, self-hosting |
| Tunable certification supersedes always-on | ADR-032 | `fast` default · `balanced` · `certified`; memory-safety/speed/ergonomics promoted to first-class goals |
| Native AOT re-gated into `lang 1.0.0` (T6) | ADR-034 | expands E15-1 to full-language native-codegen coverage — see [Status & roadmap](status-and-roadmap.md) |

> The full set (RFC-0001…0039, ADR-001…034, DN-01 onward) with status and dependencies is in
> [`docs/Doc-Index.md`](../Doc-Index.md) — the table above is the load-bearing subset.

## Suggested reading order

**Prefer thematic order over meeting order.** The numbered RFC/ADR/DN tree is the permanent
append-only record; **[`docs/companion/`](../companion/README.md)** is the maintained *historian*
that groups decisions by what they complete (mutation loop, trust axes, native L3, …) with
airlock patterns and diagrams.

1. **[`docs/companion/`](../companion/README.md)** — start at
   [How to read](../companion/00-how-to-read.md): resolution chain, thematic clusters, airlocks.
2. **[`docs/Doc-Index.md`](../Doc-Index.md)** — status oracle: every document and its dependencies.
3. **[`docs/Mycelium_Project_Foundation.md`](../Mycelium_Project_Foundation.md)** — the charter:
   vision, requirements (FR/NFR/VR), success and kill criteria, ADRs 001–009, roadmap, risks.
4. **`docs/rfcs/RFC-0001…`** — Core IR and metadata (everything else plugs into this) — after the
   companion thesis chapter, not as a cold open.
5. **Thematic deep dives** via [companion §05](../companion/05-thematic-decision-map.md), then open
   only the ADRs/DNs that cluster names (avoid reading DN-119…140 as a flat changelog).
6. **`crates/mycelium-core` and `crates/mycelium-interp`** — design-as-code (see the
   [workspace map](workspace-map.md)).
7. **`research/`** — evidence base behind a decision (see
   [Contributing conventions & provenance](contributing-and-provenance.md)).

---

**See also:** [Repository structure](repository-structure.md) · [Status & roadmap](status-and-roadmap.md) ·
[Workspace map](workspace-map.md)

[← Back to README](../../README.md)
