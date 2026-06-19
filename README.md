# Mycelium

> A programming language that treats **traditional binary**, **balanced ternary**, **dense embeddings**, and **Vector Symbolic Architectures (VSA / hyperdimensional computing)** as co-equal, first-class substrates — under semantics that are **transparent** (no hidden behavior), **metadata-native**, and **amenable to formal reasoning**.

**Status:** design + **Rust-first implementation underway.** The design corpus is Accepted/Resolved
(Foundation, RFC-0001…0021, ADR-001…017, DN-01…12), and the kernel is landing as a
Rust workspace of **42 crates** (+ `xtask`) <!-- doc-currency:crate-count --> — a trusted reference interpreter, certified binary↔ternary
swaps, the selection-policy engine, and a Rust-first standard library. Phases **0–3, 5, and 7** are
complete; Phases **4, 6, and 8** are in progress. Per the honesty rule, the stdlib specs read
*"implemented (Rust-first), pending ratification"* — **not** silently `Accepted` — and self-hosting
(M-502) is **not yet established**. See [Status & open items](#status--open-items).

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
- **Physical packing is a *schedule*, not a type.** Lossless layout (e.g. ternary packing) is chosen at a lowering stage and *recorded* as inspectable metadata — values of the same logical type stay interoperable whether packed or not.

---

## Repository structure

```
mycelium/
├── README.md                 ← you are here
├── LICENSE                   ← MIT
├── CONTRIBUTING.md           ← decision process, honesty rule, dev env, workflow
├── CLAUDE.md                 ← operating guide for Claude Code / agents (the house rules)
├── CHANGELOG.md              ← Keep-a-Changelog; design baseline + implementation edits
├── Cargo.toml                ← Rust workspace (42 crates + xtask; MSRV 1.92, ADR-007)
├── rust-toolchain.toml       ← pinned MSRV
├── justfile                  ← one source of truth for local↔CI checks (`just check`)
├── deny.toml                 ← cargo-deny supply-chain policy
├── crates/                   ← the Rust kernel + reference interpreter + stdlib (see "The Rust workspace")
├── docs/
│   ├── Mycelium_Project_Foundation.md   ← charter, requirements, ADR-001…009, roadmap, risks
│   ├── Doc-Index.md                     ← map of the corpus + status + dependency DAG
│   ├── Glossary.md                      ← the fungal lexicon + honesty/architecture terms
│   ├── rfcs/        ← RFC-0001…0021 (normative designs) + index
│   ├── adr/         ← ADR-010…017 as files (ADR-001…009 live in the Foundation §8) + index
│   ├── notes/       ← DN-01…12 design notes + reference material (lexicon, examples, research prompts)
│   ├── spec/        ← per-module + per-tool specs (stdlib/, api/ baselines, swaps/)
│   ├── planning/    ← phase-by-phase build plans (phase-0 … phase-5)
│   └── devlog/      ← append-only development log
├── research/                 ← the evidence base (records 01 … 11)
├── examples/                 ← worked `.myc` programs (hello-phylum, repr-tour)
├── experiments/              ← uv-managed Python experiments (the KC-2 LLM-leverage harness)
├── proofs/                   ← Z3/SMT2 + Liquid-Haskell proof artifacts
├── scripts/                  ← the check tooling (scripts/checks/* behind `just check`)
├── tools/                    ← GitHub issue bootstrap, LLM harness, Termux setup
└── xtask/                    ← cargo-xtask repo-automation entrypoint
```

> **Note on ADRs.** ADR-001 through ADR-009 live inside `docs/Mycelium_Project_Foundation.md` §8 (the decision log); ADR-010 through ADR-017 are broken out as their own files in `docs/adr/`. All are append-only with status transitions. The authoritative, always-current map of the whole corpus (every RFC/ADR/DN with status) is [`docs/Doc-Index.md`](./docs/Doc-Index.md).

---

## The Rust workspace

The kernel and its tooling live in `crates/` as **42 crates** (+ `xtask`), MSRV-pinned to Rust 1.92
(ADR-007). The trusted base is small and the public surface is gated by a committed API baseline
(`docs/spec/api/`, KC-3). Grouped by role:

- **Kernel / trusted base** — `mycelium-core` (Core IR: `Value`/`Repr`/`Meta`, the guarantee
  lattice, content-addressing), `mycelium-numerics` (the ε/δ bound kernels), `mycelium-diag` (the
  structured diagnostic record).
- **Interpreter & certification** — `mycelium-interp` (the reference interpreter — the trusted
  executable semantics), `mycelium-cert` (swap certificates + the certified binary↔ternary swap,
  Z3-backed).
- **Paradigms & selection** — `mycelium-dense`, `mycelium-vsa` (MAP-I algebra), `mycelium-select`
  (the total, EXPLAIN-able selection-policy engine).
- **Language & execution** — `mycelium-l1` (the L1 calculus + the unified swap/interp differential
  checker), `mycelium-mlir` (the AOT path: a `ternary` dialect, an env-machine, native LLVM, JIT,
  hot-inject).
- **Toolchain (above the kernel)** — `mycelium-lsp`, `mycelium-fmt` (`mycfmt`), `mycelium-check`
  (`myc-check`), `mycelium-lint` (`myc-lint`), `mycelium-sec` (`myc-sec`), `mycelium-doc`
  (`myc-doc`), `mycelium-spore` (the `spore` packager), `mycelium-build`, `mycelium-proj`.
- **Standard library `std`** (Rings 0–2, RFC-0016) — `mycelium-std-{core, ternary, dense, vsa,
  swap, select, content, numerics, diag, recover, spore}` (Tier-A differentiators) and
  `mycelium-std-{collections, error, cmp, iter, math, text, fmt, io, fs, time, rand, testing}`
  (Tier-B commons). Each ships its RFC-0016 §4.5 guarantee matrix as data, asserted in tests.

Every operation carries an **honest, per-op guarantee tag** on the `Exact ⊐ Proven ⊐ Empirical ⊐
Declared` lattice; out-of-range is an explicit `Result`/`Option`, never a silent fallback.

---

## Build & checks

```
just            # list recipes
just setup      # best-effort install of the check tools
just check      # the FULL suite — exactly what CI runs (build · clippy · test · docs · proofs · supply-chain)
just fmt        # auto-format (Rust + Python)
```

Checks **skip gracefully** when a tool isn't present. Remote CI (`.github/workflows/checks.yml`) is
**manual-dispatch only and advisory**, running the same `just ci` — see `CONTRIBUTING.md`.

---

## Suggested reading order

1. **`docs/Doc-Index.md`** — the map: every document, its status, and how they depend on each other.
2. **`docs/Mycelium_Project_Foundation.md`** — the charter: vision, requirements (FR/NFR/VR), success & kill criteria, ADRs 001–009, roadmap, risks.
3. **`docs/rfcs/RFC-0001…`** — the Core IR & metadata schema (everything else plugs into this).
4. **RFC-0002 → RFC-0021**, then the ADRs (010…017) and design notes (DN-01…12) for the deep dives — `Doc-Index.md` orders them.
5. **`crates/mycelium-core` and `crates/mycelium-interp`** — the kernel and reference semantics, if you want to read the design as code.
6. **`research/`** — the evidence base (records 01…11), if you want the "why" behind a decision.

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
| Surface/term-language layering (L0–L3) | RFC-0006; RFC-0007 | syntactic honesty invariants; the ten-node L1 kernel calculus; concrete L3 syntax KC-2-gated |
| Runtime & concurrency model | RFC-0008 | RT1–RT7; deterministic-fragment-first; partial failure explicit; honest probabilistic guarantees |
| Structured diagnostics + declarative recovery | RFC-0013; RFC-0014 | additive over the never-silent error (never substitutive); declared, **bounded** effects |
| Standard-library scope + per-op contract | RFC-0016 | C1–C6 (never-silent · honest tags · EXPLAIN · content-addressed · above-the-kernel · bounded effects); ring layering |
| `spore` is the deployable unit | ADR-013 | content-addressed code + values + metadata; identity is the content hash (ADR-003) |
| Interpreted↔compiled ABI + hot-inject | ADR-016; ADR-017 | hash-keyed dispatch; content-addressed dynamic linking; immutable-by-construction |

> The full set (RFC-0006…0021, ADR-011…017, DN-02…12) with status and dependencies is in
> [`docs/Doc-Index.md`](./docs/Doc-Index.md) — the table above is the load-bearing subset.

---

## Status & open items

The design corpus is Accepted and the kernel is **building**:

- ✅ **KC-1 (the existential risk) passed — and confirmed in a build.** Proven non-asymptotic VSA bundling bounds exist (Clarkson 2023; Thomas 2021), and the Liquid-Haskell `bundle` capacity-refinement probe (`proofs/lh-bundle/`, RFC-0003 §5) reports **SAFE** (Z3 discharged), ratifying the "axiomatized theorem + checked instantiation" strategy. VSA stays in core with honest `Proven` tags.
- ✅ **KC-2 / LLM leverage — verdict: proceed (DN-09).** The M-002 experiment measured *weak-but-recoverable* leverage (best arm ≈40% first-attempt → ≈70% eventual), below the "irrecoverable collapse" kill threshold. The follow-up is a committed text syntax + a co-equal projection layer (M-380; RFC-0021); the retention-ratio ablation (RP-1) stays an honest, non-blocking open research prompt.
- 🧱 **Built (Phases 0–3, 5, 7):** the Core IR + Rust reference interpreter, the single certificate checker, the certified binary↔ternary swap (Z3-proved), Dense/VSA breadth, the selection-policy engine + EXPLAIN, the `ternary` MLIR dialect + native LLVM / JIT / hot-inject, the L1 calculus, the runtime/concurrency model (RFC-0008), and the Rust-first standard library (23 `std` crates).
- 🔜 **In progress (Phases 4, 6, 8):** the full interpreted↔compiled ABI + AOT env-machine (mutual recursion, RFC-0012 ambient), native MLIR→LLVM codegen + deployable spores, and the remaining toolchain/release-engineering gates.
- ❓ **Not yet established:** **self-hosting (M-502)** — the stdlib is Rust-first; the Mycelium-lang migration half is open. Stdlib specs read *"implemented (Rust-first), pending ratification"*, never silently `Accepted`.

Residual risks tracked in the Foundation risk register, notably **RR-13** (HRR/FHRR are the VSA weak link). See `docs/Mycelium_Project_Foundation.md` §10 for the dependency-ordered action list and `docs/planning/phase-*.md` for the live phase ladder.

---

## Technology stack

- **Kernel + reference interpreter:** Rust (MSRV **1.92**, ADR-007). The interpreter is the trusted base and the reference semantics (`crates/mycelium-interp`).
- **AOT path:** **MLIR → LLVM** — a `ternary` dialect + env-machine + native lowering, with JIT and hot-inject (`crates/mycelium-mlir`), confined to the performance path; `vsa`/`embedding` dialects deferred.
- **VSA submodule:** Rust (`crates/mycelium-vsa`) — the MAP-I algebra + the per-model guarantee matrix (RFC-0003).
- **Verified numerics:** a Flocq/FloVer-style certificate-checker-in-Rust — two assurance tiers, ε (affine arithmetic) and δ (union-bound) sharing one certificate (`crates/mycelium-numerics`, ADR-010).
- **Toolchain:** Rust crates (`mycfmt` / `myc-check` / `myc-lint` / `myc-sec` / `myc-doc` / `spore`), all routed through `just check`.
- **Experiments / LLM harness:** Python **3.13 / 3.14**, **UV**, **pytest**, **codecov** (`experiments/`, `tools/llm-harness/`).
- **Proofs:** Z3/SMT2 (binary↔ternary injectivity, `proofs/binary-ternary-roundtrip/`) + Liquid Haskell (`proofs/lh-bundle/`).

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

Everything in `docs/` traces back to the research passes recorded in `research/` — now **eleven records** (`01…11`), spanning the prior-art survey and T0/T1/T2 findings through the language layer, runtime/concurrency, error-recovery & bounded effects, automatic-baseline diagnostics, the narrative-authoring pipeline, honest-stdlib prior art, stage-1 grading non-interference, traits/coherence & Repr-polymorphism, and the semantic-projection framework. Each record carries its structured findings + source list; normative claims in `docs/` cite their grounding (survey labels `G*`/`A–E`/`R*`; research labels `T0.x…T11.x`) or are flagged as open questions.

---

## License

MIT — Copyright (c) 2026 **Tyler Zervas**. See [`LICENSE`](./LICENSE).
