# Mycelium

> A programming language that treats **traditional binary**, **balanced ternary**, **dense embeddings**, and **Vector Symbolic Architectures (VSA / hyperdimensional computing)** as co-equal, first-class substrates — under semantics that are **transparent** (no hidden behavior), **metadata-native**, and **amenable to formal reasoning**.

**Status:** design phase. Both research passes are complete and the full design corpus (Foundation + RFC-0001…0005 + ADR-010 + DN-01) is **Accepted/Resolved**. The next step is implementation, plus one confirming proof-of-concept (see [Status & open items](#status--open-items)).

*(Formerly named **Verid** — retained only as a provenance note.)*

---

## Why this exists

Modern computing keeps these four representations in separate worlds: bits for traditional computation, embeddings for ML, hypervectors for symbolic-connectionist work, and balanced ternary as a recurring "what if" in hardware. Moving between them is where correctness quietly leaks — conversions are implicit, lossy in undocumented ways, and impossible to audit.

Mycelium's thesis is that the **representation-swap** should be the explicit, verifiable, first-class operation of the language. The central design problem is therefore **metadata-native, explicit, verifiable swapping between substrates** — with every approximation disclosed, bounded, and tagged by how trustworthy that bound is.

Three non-negotiables shape every decision:
1. **No hidden / opaque behavior** in core semantics.
2. **Human-intelligible *and* useful for AI agents** (a dual-intelligibility goal).
3. **Formally reasoning-amenable** — "no black boxes" is realized as mechanically-checkable invariants, not a slogan.

---

## The core ideas (in one screen)

- **Representation is part of the type.** `Binary{width}`, `Ternary{trits}`, `Dense{dim,dtype}`, `VSA{model,dim,sparsity}` are distinct type families. There is **no implicit conversion** between paradigms.
- **`Swap` is the only representation-changing operation**, and every swap emits a **certificate** describing exactly what the conversion cost.
- **Honesty is a typed, monotone property.** A guarantee lattice — **`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`** — travels with every value and degrades by *meet* through operations, so a disclosed guarantee can never spuriously strengthen.
- **Metadata is self-describing and survives lowering** (Apache-Arrow-grade): provenance, bounds, layout, and reconstruction info are queryable at runtime and exposed to tooling.
- **Split verification regime:** binary↔ternary swaps are *provable/bijective-within-range*; VSA/embedding swaps carry *bounded/probabilistic* per-instance certificates (translation-validation style).
- **Physical packing is a _schedule_, not a type.** Lossless layout (e.g. ternary packing) is chosen at a lowering stage and *recorded* as inspectable metadata — values of the same logical type stay interoperable whether packed or not.

---

## Repository structure

```
mycelium/
├── README.md                  ← you are here
├── LICENSE                    ← MIT
├── CONTRIBUTING.md            ← decision process, honesty rule, dev env, workflow
├── CHANGELOG.md               ← Keep-a-Changelog; the design baseline + edits
├── .gitignore                 ← Rust + Python
├── docs/
│   ├── Mycelium_Project_Foundation.md   ← charter, scope, requirements, ADR-001…010, roadmap, risk register
│   ├── Doc-Index.md                     ← map of the corpus + status + dependency DAG
│   ├── rfcs/
│   │   ├── README.md                                        ← RFC index + process + template
│   │   ├── RFC-0001-Core-IR-and-Metadata-Schema.md        ← value model, Repr, Meta, guarantee lattice, content-addressing
│   │   ├── RFC-0002-Swap-Certificate-and-Split-Regime.md  ← certificate, legal pairs, binary↔ternary bijection, shared checker
│   │   ├── RFC-0003-VSA-Submodule-Boundary.md             ← kernel↔submodule boundary, per-model guarantee matrix, manifest
│   │   ├── RFC-0004-Execution-Model-and-Stable-Component.md ← MLIR→LLVM, reference interpreter, schedule-staged packing
│   │   └── RFC-0005-Selection-Policy-Language.md          ← total cost-based selection policy + mandatory EXPLAIN
│   ├── adr/
│   │   ├── README.md                                        ← ADR index + process + template
│   │   └── ADR-010-Verified-Numerics-Foundation.md        ← two bound kernels (ε / δ) + shared certificate
│   └── notes/
│       └── DN-01-Packing-Placement-Tradeoffs.md           ← in-type vs metadata vs schedule-staged (Resolved)
└── research/
    ├── 01-prior-art-survey-RECORD.md          ← Pass 1: gaps G1–G11, tensions A–E, recommendations R1–R8 + sources
    └── 02-research-findings-RECORD.md         ← Pass 2: T0/T1/T2 results + sources
```

> **Note on ADRs.** ADR-001 through ADR-009 live inside `docs/Mycelium_Project_Foundation.md` §8 (the decision log). ADR-010 is broken out as its own file because of its length. All are append-only with status transitions.

---

## Suggested reading order

1. **`docs/Doc-Index.md`** — the map: every document, its status, and how they depend on each other.
2. **`docs/Mycelium_Project_Foundation.md`** — the charter: vision, requirements (FR/NFR/VR), success & kill criteria, ADRs 001–010, roadmap, risks.
3. **`docs/rfcs/RFC-0001…`** — the Core IR & metadata schema (everything else plugs into this).
4. **RFC-0002 → RFC-0005**, then **ADR-010** and **DN-01** for the deep dives.
5. **`research/`** — the evidence base, if you want the "why" behind a decision.

---

## Key decisions at a glance

| Decision | Where | Summary |
|---|---|---|
| Guarantee lattice + honesty propagation | RFC-0001; ADR-001 | `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`, meet on compose |
| No implicit conversion; explicit `Swap` only | RFC-0001 §3.3; FR-M3 | the central transparency rule |
| Split verification regime | RFC-0002; ADR-002 | provable binary↔ternary; bounded/probabilistic VSA |
| One certificate checker, two uses | RFC-0002 + RFC-0004 | swaps *and* interpreter-vs-compiled equivalence |
| Binary↔ternary = `LosslessWithinRange` | RFC-0002 | total bijection impossible at fixed widths; `Option`-typed, never silent |
| VSA in core, but an optional submodule | RFC-0003; ADR-008 | lean kernel (type slot only); opt-in algebra |
| Per-model × per-op guarantee matrix | RFC-0003 | honest tags; HRR/FHRR unbind is the weak link |
| Sparsity as static refinement | RFC-0001 §4.4; RFC-0003 §5 | declared class checked by SMT; capacity = axiomatized theorem + checked instantiation |
| Verified-numerics: two bound kernels | ADR-010 | `ErrorBound` (ε, affine arith.) + `ProbBound` (δ, union-bound/apRHL), shared certificate |
| Hybrid execution; interpreter = reference | RFC-0004; ADR-009 | AOT (MLIR→LLVM) for stable components; interpret/JIT for dev/dynamic |
| Packing is schedule-staged, not typed | DN-01; RFC-0004 §5 | cost-model selector over a small fixed set (I2_S/TL1/TL2) |
| Selection policy is total + EXPLAIN-mandatory | RFC-0005; ADR-006 | non-learned, deterministic, auditable; no cardinality-estimate black box |

---

## Status & open items

The corpus is Accepted; the work is now **build + confirm**:

- ✅ **KC-1 (the existential risk) passed** — proven non-asymptotic VSA bundling bounds exist (Clarkson 2023; Thomas 2021), so VSA stays in core with honest `Proven` tags.
- 🔜 **One confirming build:** a Liquid-Haskell `bundle` capacity-refinement probe (RFC-0003 §5) that ratifies the "axiomatized theorem + checked instantiation" strategy end-to-end.
- ❓ **One genuinely-open existential question:** **KC-2 / LLM leverage** — whether AI agents can productively read/write Mycelium's novel surface (the E4 experiment). The research did *not* settle this.
- 🧱 **Then build:** Core IR + Rust reference interpreter → the single certificate checker → `ternary` MLIR dialect + schedule-staged packing → VSA submodule.

Residual risks tracked in the Foundation risk register, notably **RR-13** (HRR/FHRR are the VSA weak link) and **RR-14** (Rust VSA ecosystem immature — the submodule is a build).

See `docs/Mycelium_Project_Foundation.md` §10 for the dependency-ordered action list.

---

## Intended technology stack

- **Kernel + reference interpreter:** Rust (MSRV **1.92**). The interpreter is the trusted base and the reference semantics.
- **AOT path:** **MLIR → LLVM** (a `ternary` dialect first; `vsa`/`embedding` dialects deferred), confined to the performance path.
- **VSA submodule:** Rust; reuse the `balanced-ternary` crate; port `torchhd`'s operation set as the reference.
- **Verified numerics:** a Flocq/FloVer-style certificate-checker-in-Rust (two assurance tiers).
- **Tooling / experiments / LLM harness:** Python **3.13 / 3.14**, **UV**, **pytest**, **codecov**.

---

## Glossary

- **Substrate / paradigm** — one of the four representation families (binary, balanced ternary, dense embedding, VSA).
- **Balanced ternary** — base-3 with digits {−1, 0, +1}; symmetric, sign-is-a-digit. Used here as a *logical* substrate, forward-compatible with native-ternary hardware.
- **VSA / HDC** — Vector Symbolic Architectures / hyperdimensional computing: high-dimensional vectors with algebraic operations (bind, bundle, permute) for symbolic-connectionist computation.
- **Swap** — the explicit, certificate-emitting operation that changes a value's representation. The only such operation.
- **Guarantee lattice** — `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`: how trustworthy a value's accuracy claim is; composes by *meet* (weakest wins).
- **Certificate** — a checkable record that a swap (or a compiled artifact vs. the interpreter) meets a claimed `{ε, δ, strength}` bound.
- **Translation validation** — proving each *instance* of a lowering/conversion correct, rather than proving the whole compiler correct once.
- **Schedule-staged packing** — choosing a lossless physical layout at a lowering stage (a "schedule"), recorded as inspectable metadata, not encoded in the type.
- **`ErrorBound` / `ProbBound`** — the two ADR-010 bound kernels: error-magnitude (ε) via affine arithmetic; failure-probability (δ) via the union bound / approximate couplings.
- **Reconstruction manifest** — the explicit recipe (model, codebooks, compositional structure, decoding procedure, bound) needed to recover content from a VSA representation; distinguishes indexed retrieval from true compositional reconstruction.
- **Stable component** — a definition that is content-addressed + spec-ratified + verification-passed, and therefore eligible for AOT compilation.

---

## Conventions for contributing

> Full detail (process, dev environment, workflow) is in [`CONTRIBUTING.md`](./CONTRIBUTING.md). In brief:

- **Decisions are append-only.** Don't silently edit an ADR/RFC decision — supersede it with a new status (`Draft/Proposed → Accepted → Superseded`) and link forward. Every claim cites its grounding (survey labels `G*`/`A–E`/`R*`; research labels `T0.x/T1.x/T2.x`).
- **Honesty rule.** Guarantee tags are assigned **per model and per operation**, never in aggregate. A bound may be tagged `Proven` *only* if it cites a theorem whose side-conditions are checked; otherwise it is `Empirical` (validated) or `Declared` (user-asserted, always flagged). New results may *upgrade* a tag; absence keeps it weaker.
- **No black boxes.** Any feature that introduces opaque behavior (especially "intelligent" automatic selection) must be reified, inspectable, and explainable (`EXPLAIN`).
- **Engineering principles** (the project's house style): SRP, OCP, LSP, ISP, DIP, DRY, KISS, YAGNI, Law of Demeter, separation of concerns; **composition over inheritance**; PEP 8 / Black for Python.
- **Kill criteria** (KC-1…KC-4) are re-checked at every phase gate; a gate that doesn't check them is hiding risk.

---

## Provenance & evidence

Everything in `docs/` traces back to the two research passes recorded in `research/`. The *full narrative* research reports (the prior-art survey and the T0/T1/T2 findings, each with full inline citations) were produced as separate long-form artifacts; `research/` contains structured records of them with the source lists. If you want the full narratives committed here as files, they can be added on request.

---

## License

MIT — Copyright (c) 2026 **Tyler Zervas**. See [`LICENSE`](./LICENSE).
