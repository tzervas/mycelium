# Mycelium

> A **fast, memory-safe, ergonomic** multi-paradigm language that treats **traditional binary**, **balanced ternary**, **dense embeddings**, and **Vector Symbolic Architectures (VSA / hyperdimensional computing)** as co-equal, first-class substrates — under semantics that are **transparent** (no hidden behavior) and **metadata-native**, with **certification & auditability baked in as *optional, tunable* capabilities** (`fast` by default · `certified` on request) rather than a tax on every line.

**Status:** design + **Rust-first implementation underway.** The design corpus spans Foundation,
RFC-0001…0035, ADR-001…032, DN-01…33 — per-document status (Draft / Proposed / Accepted / Enacted /
Resolved) is in [`docs/Doc-Index.md`](docs/Doc-Index.md); the newest decisions range from Draft to Enacted. The Rust workspace has
**50 crates** (+ `xtask`) <!-- doc-currency:crate-count --> — a trusted reference interpreter, explicit
representation **swaps** (certified at the `certified` mode), the selection-policy engine, a
verified-numerics layer, a **Rust-first standard library**, an L1 surface with **generics · traits ·
higher-order functions · operator syntax**, a **runtime** (scheduler, structured concurrency) with a
**three-layer hybrid memory model** (affine ownership → optimized reference counting → region
reclamation; DN-32 / RFC-0027), and the **static RC-elision passes** that build on it (MEM-4 / DN-33).
Versioning is **dual-axis (ADR-022)**: the Rust **core/kernel** is **gate-met / tag-ready** for
`core 1.0.0` (the ratified ADR-021 criteria, carried forward as ADR-022 **track T1**), while the
**full language** (`lang`) targets a broader `1.0.0` — a fully usable language whose **stdlib and
libraries are themselves written in Mycelium**. That program is mapped in **ADR-022** (the gate,
supersedes ADR-021) + **DN-25** (tracks **T1–T9** → epics E10-1…E18-1): surface completeness, runtime,
**stdlib-in-Mycelium** (the long pole), FFI, toolchain, docs, and self-hosting — native-AOT perf is
`1.1`. Per the transparency rule, no claim here is upgraded beyond what a checked basis supports (VR-5).

> **Direction note (ADR-032, Enacted 2026-06-24).** The north star has been **repositioned** from the
> original "certified-everything substrate" premise toward **a fast, memory-safe, ergonomic
> multi-paradigm language**, with certification/transparency as **optional, tunable** capabilities
> (RFC-0034: `fast` default · `balanced` · `certified`). Memory-safety, speed, and ergonomics are now
> **first-class goals** alongside the transparent-swap thesis. The "honesty rule" is reframed as the
> **transparency & auditability rule** (mechanism unchanged — the `Exact ⊐ Proven ⊐ Empirical ⊐
> Declared` lattice, never-silent G2, and downgrade-don't-overclaim VR-5 all stand). See ADR-032 +
> RFC-0034, the memory model (DN-32 / RFC-0027 / DN-33), and the Foundation §1 charter update.

---

## Why this exists

Modern computing keeps four representation families in separate worlds: bits for traditional
computation, dense embeddings for ML, hypervectors for symbolic-connectionist work, and balanced
ternary as a recurring "what if" in hardware. Moving between them is where correctness quietly
leaks — conversions are implicit, lossy in undocumented ways, and impossible to audit.

Mycelium's thesis is that the **representation-swap** should be the explicit, verifiable,
first-class operation of the language. The central design problem is therefore
**metadata-native, explicit, verifiable swapping between substrates** — with every approximation
disclosed, bounded, and tagged by how trustworthy that bound is.

Three non-negotiables shape every decision:

1. **No hidden / opaque behavior** in core semantics.
2. **Human-intelligible *and* useful for AI agents** (a dual-intelligibility goal).
3. **Formally reasoning-amenable** — "no black boxes" is realized as mechanically-checkable
   invariants, not a slogan.

---

## The core ideas

- **Representation is part of the type.** `Binary{width}`, `Ternary{trits}`, `Dense{dim,dtype}`,
  `VSA{model,dim,sparsity}` are distinct type families. There is **no implicit conversion**
  between paradigms.
- **`Swap` is the only representation-changing operation**, and every swap emits a **certificate**
  describing exactly what the conversion cost — bijective for binary↔ternary, bounded/probabilistic
  for ↔VSA/embedding (the split verification regime, ADR-002).
- **Transparency is a typed, monotone property.** A guarantee lattice —
  **`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`** — travels with every value and degrades by *meet*
  through operations, so a disclosed guarantee can never spuriously strengthen.
- **Metadata is self-describing and survives lowering** (Apache-Arrow-grade): provenance, bounds,
  layout, and reconstruction info are queryable at runtime and exposed to tooling.
- **All four substrates are co-equal, first-class.** Binary and balanced ternary share the kernel
  type system; dense embeddings and VSA/HDC are not optional add-ons — they participate in the same
  type + swap + certificate machinery, with VSA packaged as an optional-but-first-class submodule
  (ADR-008).
- **Physical packing is a *schedule*, not a type.** Lossless layout (e.g., ternary packing) is
  chosen at a lowering stage and *recorded* as inspectable metadata — values of the same logical
  type stay interoperable whether packed or not (DN-01, RFC-0004 §5).
- **Selection policies are reified, EXPLAIN-able artifacts.** Any policy-driven selection is a
  first-class, queryable value; every swap records the `PolicyRef` it used (ADR-006, RFC-0005).
- **Definitions are content-addressed.** Identity is the content hash; names are metadata
  (Unison-style, ADR-003). A stable component is content-addressed + spec-ratified +
  verification-passed, and only then eligible for AOT compilation.

---

## What is built

### The Rust workspace — 50 crates (+ `xtask`)

The kernel and tooling live in `crates/` under MSRV-pinned Rust 1.92 (ADR-007). The public
surface is gated by a committed API baseline (`docs/spec/api/`, KC-3). **Every crate now carries its
own `README.md`** (linked below) for a 30-second orientation; this map is synthesised from them.
Grouped by role:

#### Kernel / trusted base

| Crate | Role |
|---|---|
| [`mycelium-core`](crates/mycelium-core/README.md) | Core IR: `Value<Repr,Meta>`, the guarantee lattice, content-addressing, the node grammar; the never-silent fallibility contract (RFC-0001) |
| [`mycelium-numerics`](crates/mycelium-numerics/README.md) | Two bound kernels — `ErrorBound` (ε, affine arithmetic) + `ProbBound` (δ, union/apRHL) — meeting at one `{ε, δ, strength}` certificate (ADR-010) |
| [`mycelium-vsa`](crates/mycelium-vsa/README.md) | The VSA submodule: the `VsaModel` trait + MAP-I, dependency-gated so the kernel stays small (RFC-0003; ADR-008) |
| [`mycelium-dense`](crates/mycelium-dense/README.md) | Dense `Dense{dim,dtype}` values + elementwise ops with honest per-op rounding bounds (RFC-0001 §4.1) |
| [`mycelium-select`](crates/mycelium-select/README.md) | The total, EXPLAIN-able selection-policy engine — content-addressed decision tables, no black box (RFC-0005, ADR-006) |

#### Compiler / execution

| Crate | Role |
|---|---|
| [`mycelium-interp`](crates/mycelium-interp/README.md) | The **reference interpreter** — the trusted small-step semantics; AOT/JIT paths are validated against it (RFC-0004; ADR-009) |
| [`mycelium-cert`](crates/mycelium-cert/README.md) | Swap certificates + the certified binary↔ternary swap, and the **one** shared translation-validation checker (RFC-0002) |
| [`mycelium-l1`](crates/mycelium-l1/README.md) | The L1 surface prototype (RFC-0006/0007): lexer, parser, typechecker, totality checker, evaluator, elaborator to Core IR; stage-1 generics/traits/effects landed (E7-1) |
| [`mycelium-stack`](crates/mycelium-stack/README.md) | Host-stack management for the L1 frontend's recursive passes — kept outside the kernel so `mycelium-l1` stays `unsafe`-free (KC-3) |
| [`mycelium-mlir`](crates/mycelium-mlir/README.md) | The AOT path: env-machine, direct-LLVM-IR backend, optional MLIR `ternary` dialect, JIT + hot-inject (RFC-0004; ADR-007) |
| [`mycelium-mir-passes`](crates/mycelium-mir-passes/README.md) | **MEM-4 (DN-33):** the RC-annotated IR + Perceus-style RC emission/elision passes — optimisation-only, **outside** the trusted Core IR (KC-3) |

#### Runtime & memory model

| Crate | Role |
|---|---|
| [`mycelium-std-runtime`](crates/mycelium-std-runtime/README.md) | The fungal concurrency surface (Colony/Scope/Task/Network/scheduler/supervision — ADR-020 / RFC-0008) **and** the landed **three-layer hybrid memory model** runtime — reclamation records, RC cell, regions, live scope/region wiring, and the three triggers (RcZero · ScopeExit · ChannelClose) — DN-32 / RFC-0027 / DN-33 |

#### Toolchain crates

| Crate | Binary | Role |
|---|---|---|
| [`mycelium-cli`](crates/mycelium-cli/README.md) | `myc` | The one-command driver: `myc init\|build\|check\|test\|run` over a phylum, with DN-22 structured diagnostics |
| [`mycelium-check`](crates/mycelium-check/README.md) | `myc-check` | Project-aware type-check driver; aggregates every refusal as an RFC-0013 diagnostic |
| [`mycelium-fmt`](crates/mycelium-fmt/README.md) | `mycfmt` | The canonical formatter — an identity-preserving projection that never changes content-addressed identity |
| [`mycelium-lint`](crates/mycelium-lint/README.md) | `myc-lint` | Lint + auto-fix with a `suggest`/`apply`/`scaffold` boundary (M-141 invariant lints) |
| [`mycelium-sec`](crates/mycelium-sec/README.md) | `myc-sec` | Security checks — the `wild`-block audit + secrets/supply-chain gates |
| [`mycelium-doc`](crates/mycelium-doc/README.md) | `myc-doc` | The doc build pipeline: content-addressed doc-IR, HTML/Typst/JSON renderers, §4.1 quality lint |
| [`mycelium-spore`](crates/mycelium-spore/README.md) | `spore` | Content-addressed packager: builds a deployable `spore` from a project (ADR-013) |
| [`mycelium-lsp`](crates/mycelium-lsp/README.md) | LSP | The semantic-feedback facade — diagnostics, swap certificates, bound/guarantee annotations, EXPLAIN traces over one surface |
| [`mycelium-build`](crates/mycelium-build/README.md) | — | Stable-vs-interpreted classification + content-addressed build certificates (RFC-0004 §4) |
| [`mycelium-proj`](crates/mycelium-proj/README.md) | — | Project metadata: nodule header, `mycelium-proj.toml`, the inheritance resolver, `@certification` scoping |
| [`mycelium-bench`](crates/mycelium-bench/README.md) | — | Honest benchmark harness: a deterministic WIN/LOSS/REGRESSION report over the execution backends |
| [`mycelium-diag`](crates/mycelium-diag/README.md) | — | The canonical RFC-0013 `Diag` record types (the failure-legibility substrate) |
| [`mycelium-cli-common`](crates/mycelium-cli-common/README.md) | — | Small dependency-free helper shared by the toolchain CLIs |

#### Standard library — 26 `mycelium-std-*` crates

The Rust-first standard library implements RFC-0016's three-ring contract. Every exported op carries
a per-op guarantee tag; every fallible op returns an explicit `Result`/`Option`, never a silent
fallback. The RFC-0016 §4.5 guarantee matrix is encoded as data and asserted in tests — never prose
only. Each crate's `README.md` links its `docs/spec/stdlib/<name>.md` spec.

**Tier A — differentiators** (the substrates + Mycelium-specific capabilities):
[`std-core`](crates/mycelium-std-core/README.md) · [`std-swap`](crates/mycelium-std-swap/README.md) ·
[`std-ternary`](crates/mycelium-std-ternary/README.md) · [`std-dense`](crates/mycelium-std-dense/README.md) ·
[`std-vsa`](crates/mycelium-std-vsa/README.md) · [`std-select`](crates/mycelium-std-select/README.md) ·
[`std-content`](crates/mycelium-std-content/README.md) · [`std-numerics`](crates/mycelium-std-numerics/README.md) ·
[`std-diag`](crates/mycelium-std-diag/README.md) · [`std-recover`](crates/mycelium-std-recover/README.md) ·
[`std-spore`](crates/mycelium-std-spore/README.md) · [`std-sys`](crates/mycelium-std-sys/README.md) ·
[`std-sys-host`](crates/mycelium-std-sys-host/README.md) (the `std-runtime` crate is listed under *Runtime & memory model* above).

**Tier B — common / expected** (same C1–C6 contract, above the Tier-A crates):
[`std-collections`](crates/mycelium-std-collections/README.md) · [`std-error`](crates/mycelium-std-error/README.md) ·
[`std-cmp`](crates/mycelium-std-cmp/README.md) · [`std-iter`](crates/mycelium-std-iter/README.md) ·
[`std-math`](crates/mycelium-std-math/README.md) · [`std-text`](crates/mycelium-std-text/README.md) ·
[`std-fmt`](crates/mycelium-std-fmt/README.md) · [`std-io`](crates/mycelium-std-io/README.md) ·
[`std-fs`](crates/mycelium-std-fs/README.md) · [`std-time`](crates/mycelium-std-time/README.md) ·
[`std-rand`](crates/mycelium-std-rand/README.md) · [`std-testing`](crates/mycelium-std-testing/README.md)

**Note on self-hosting.** The stdlib is Rust-first; the Mycelium-lang migration half (M-502)
is not yet established and is explicitly post-1.0 scope (ADR-021 §5).

### The verified proof artifacts

| Artifact | What it proves |
|---|---|
| `proofs/binary-ternary-roundtrip/` (Z3/SMT2) | Bijectivity of binary↔ternary swaps within range |
| `proofs/lh-bundle/` (Liquid-Haskell) | MAP-I `bundle` capacity refinement: types **SAFE**, Z3 discharged — ratifying the axiomatized-theorem + checked-instantiation strategy (RFC-0003 §5; KC-1 confirmed 2026-06-09) |

### The LLM-leverage experiment (KC-2 — Resolved, DN-09)

The M-002 harness (`experiments/`, `tools/llm-harness/`) ran the KC-2 LLM-leverage experiment
and a subsequent multi-arm retention-ratio ablation (M-381). Verdict (DN-09, Resolved
2026-06-18): **proceed** — the surface is learnable-from-context, the failure mode is a
knowledge-surface gap (not irrecoverable collapse), and the grammar-in-context primer reaches
91.7% pass@1 on frontier models. The retention ratio (arm2 grammar-primed vs arm4 LlmCanonical)
is **DETERMINATE** (DN-09 §10, 2026-06-21): 550% for `grok-build-0.1` and 220% for `grok-4.3`
— the RFC-0021 §4.7 promote-to-projection trigger does **not** fire. L3 strategy selected:
committed text syntax + a co-equal structured-projection layer (M-380, RFC-0021).

---

## The guarantee lattice in practice

Every operation in the kernel and standard library carries one of four guarantee tags,
never upgraded without a checked basis (VR-5):

| Tag | Meaning | When it applies |
|---|---|---|
| `Exact` | No accuracy loss; result is the mathematical ideal | Binary arithmetic, `len`, boolean ops, lossless widening |
| `Proven` | Follows from a theorem whose side-conditions are checked | Binary↔ternary bijectivity (Z3); MAP/BSC `bundle` capacity (Clarkson-Ubaru-Yang / Thomas-Dasgupta-Rosing, ratified by the LH probe) |
| `Empirical` | Validated across ≥10⁴ randomized trials; bound stated and measured | FHRR/HRR `unbind` crosstalk; float ε bounds not yet reduced to a Proven basis |
| `Declared` | User-asserted or open research prompt; always flagged | Unverified user bounds; open T3.6 retention ablation |

The lattice composes by *meet* (weakest wins) through operations, so a composed result can never
spuriously claim a stronger guarantee than its inputs. Out-of-range input is an explicit
`Result`/`Option`, never a silent clamp or fallback.

**The split verification regime** (ADR-002):

| Swap class | Guarantee | Mechanism |
|---|---|---|
| binary ↔ ternary | `Proven` bijective | Round-trip proof (Z3) + property tests; `LosslessWithinRange` — `Option`-typed, never silent |
| ↔ dense embedding / VSA | `Proven` or `Empirical` bounded/probabilistic | Per-swap certificate (translation-validation model, VR-4): typed `{ε, δ, strength}` certificate |

---

## How it compares — and why

Mycelium is not trying to be a faster general-purpose systems language, a better ML framework,
or a novel dependently-typed proof assistant. Each comparison is made fairly — shared ground
and genuine differences.

### vs. typed systems languages (Rust, Haskell, ML family)

**Shared:** strong static types, explicit ownership/lifetime reasoning, composition over
inheritance, small auditable kernel (KC-3), no hidden behavior; content-addressed definitions
draw from Unison.

**Different:** none of them treat dense embeddings or VSA/HDC as first-class type families with
a shared type system. In Rust or Haskell, moving a value from a ternary representation to a VSA
hypervector is a user-written, uncertified function call; the type system has no stake in the
accuracy claim. Mycelium's `swap` is the *only* such operation, and it must emit a certificate.
The guarantee lattice is part of the *type* of a value, not a documentation annotation.

**Why:** the survey found no existing system unifies even two of {binary, balanced ternary, dense
embedding, sparse/dense VSA} as co-equal, first-class substrates with verifiable inter-conversion
(G1). The four-way union with certified swaps is the novel integrative contribution.

### vs. ML / array languages and Python scientific stack

**Shared:** first-class dense vector/matrix operations; numeric precision tagging; the accuracy
requirement around float approximation recalls Rosa/Daisy/Gappa.

**Different:** NumPy/PyTorch treat conversion silently — a `.half()` call in PyTorch does not
emit a certificate describing the precision loss, and there is no guarantee lattice ensuring the
accuracy claim propagates correctly through a pipeline. VSA operations (if present at all) are
a library on top of the type system, not a first-class type family. Mycelium's `Dense` and `VSA`
types are at the same level as `Binary` and `Ternary`; swapping between them is the same
certified `swap` operation.

**Why:** ML practitioners routinely suffer from silent precision loss and from the impossibility
of auditing what happened to a pipeline's accuracy claims. Mycelium addresses this structurally,
not through documentation conventions.

### vs. VSA / HDC libraries (torchhd, resonator-network implementations)

**Shared:** the MAP-I algebra (`bind`/`unbind`/`bundle`/`permute`/`cleanup`), per-model
guarantee matrices, with capacity bounds and crosstalk stated.

**Different:** torchhd (and similar libraries) sit above PyTorch as a numeric layer; the host
language's type system knows nothing about the hypervector type or its bounds. Mycelium's `VSA`
type is a first-class type family in the *language's* core type system. The capacity bound is a
`Proven` or `Empirical` guarantee tag on the *value*, not a comment in the source. The `bundle`
probe (`proofs/lh-bundle/`) confirmed that MAP-I capacity admits `Proven` tags under the
Clarkson-Ubaru-Yang / Thomas-Dasgupta-Rosing non-asymptotic bounds — so "bounds exist"
is checked, not declared.

HRR/FHRR are the VSA weak link (RR-13): non-self-inverse bind means unbind is lossy
(`Empirical` only); prefer MAP/BSC for compositional work where `Proven` tags are needed.

**Why:** the survey found no VSA library that integrates with a language-level type system
providing certified inter-substrate swaps (G1). Building the VSA submodule in-language (not just
in a library) is what enables the certified swap infrastructure to cover VSA↔binary/ternary/dense
paths.

### vs. verification-oriented languages (CompCert, Fstar, Lean, Dafny)

**Shared:** translation-validation (per-swap certificate checking, not whole-engine proof, VR-4),
the "no black boxes" principle, explicit about what is and is not proven.

**Different:** CompCert-style verified compilers prove a *compiler* correct once; Mycelium uses
translation-validation to prove each *instance* of a swap or lowering correct. Mycelium does not
require the user to write proofs — the guarantee lattice is inferred by meet-composition from
per-op tags, and proofs live in the implementation, not exposed to the surface language. Mycelium
is also multi-substrate (four representation families), which no existing verification-oriented
language treats as first-class.

**Why:** whole-engine proofs (CompCert-style) are high-cost; per-swap translation validation
(the "certificate checker in Rust" approach from ADR-010) gives guarantees at per-swap
granularity with manageable overhead. The KC-4 gate (cert-overhead budget) confirms the overhead
is within budget: ≤ 2× the swap cost for swaps whose own cost exceeds the check, ≤ 5 µs
absolute (ADR-021 A5, measured — `cargo xtask kc4`).

---

## Repository structure

```
mycelium/
├── README.md                 ← you are here
├── LICENSE                   ← MIT
├── CONTRIBUTING.md           ← decision process, transparency rule, dev env, workflow
├── CLAUDE.md                 ← operating guide for Claude Code / agents (the house rules)
├── CHANGELOG.md              ← Keep-a-Changelog; design baseline + implementation edits
├── Cargo.toml                ← Rust workspace (50 crates + xtask; MSRV 1.92, ADR-007)
├── rust-toolchain.toml       ← pinned MSRV
├── justfile                  ← one source of truth for local↔CI checks (`just check`)
├── deny.toml                 ← cargo-deny supply-chain policy
├── crates/                   ← the Rust kernel + reference interpreter + stdlib (see "What is built")
├── docs/
│   ├── Mycelium_Project_Foundation.md   ← charter, requirements, ADR-001…009, roadmap, risks
│   ├── Doc-Index.md                     ← map of the corpus + status + dependency DAG
│   ├── Glossary.md                      ← the fungal lexicon + transparency/architecture terms
│   ├── rfcs/        ← RFC-0001…0032 (normative designs) + index
│   ├── adr/         ← ADR-010…024 as files (ADR-001…009 live in the Foundation §8) + index
│   ├── notes/       ← DN-01…28 design notes + reference material (lexicon, examples, research prompts)
│   ├── spec/        ← per-module + per-tool specs (stdlib/, api/ baselines, swaps/, grammar/)
│   ├── planning/    ← phase-by-phase build plans (phase-0 … phase-8)
│   └── devlog/      ← append-only development log
├── research/                 ← the evidence base (records 01 … 13)
├── examples/                 ← worked `.myc` programs (hello-phylum, repr-tour)
├── lib/                      ← self-hosted Mycelium-lang stdlib (`.myc`; std.result — M-649/RFC-0024)
├── experiments/              ← uv-managed Python experiments (the KC-2 LLM-leverage harness)
├── fuzz/                     ← cargo-fuzz durability targets (standalone nightly workspace; WS8/M-654)
├── proofs/                   ← Z3/SMT2 + Liquid-Haskell proof artifacts
├── scripts/                  ← the check tooling (scripts/checks/* behind `just check`)
├── tools/                    ← GitHub issue bootstrap, LLM harness, Termux setup
└── xtask/                    ← cargo-xtask repo-automation entrypoint
```

> **Note on ADRs.** ADR-001 through ADR-009 live inside `docs/Mycelium_Project_Foundation.md` §8
> (the decision log); ADR-010 through ADR-024 are broken out as their own files in `docs/adr/`.
> All are append-only with status transitions. The authoritative, always-current map of the whole
> corpus (every RFC/ADR/DN with status) is [`docs/Doc-Index.md`](./docs/Doc-Index.md).

---

## Build & checks

```
just            # list recipes
just setup      # best-effort install of the check tools
just check      # the FULL suite — exactly what CI runs (build · clippy · test · docs · proofs · supply-chain)
just fmt        # auto-format (Rust + Python)
just docs-index # regenerate docs/api-index/ after a public-API change
```

Checks **skip gracefully** when a tool isn't present. Remote CI
(`.github/workflows/checks.yml`) is **manual-dispatch only and advisory**, running the same
`just ci` — see `CONTRIBUTING.md`.

---

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
selection-policy engine + EXPLAIN; the direct-LLVM native path (data/closure/tail-recursion
fragment, M-373/M-378/M-379); JIT (M-340); hot-inject (M-341); the L1 calculus; the
runtime/concurrency model (RFC-0008); the full toolchain suite; and the Rust-first standard
library — **25/25 crate specs ratified to `Accepted`** (2026-06-20/21, DN-07 + maintainer
2026-06-21). The **E7-1 L1 stage-1 surface** is now complete in the type-checker: generics
(M-656/M-657), trait/`impl` checker + coherence (M-658/M-659), effect annotations (M-660),
`wild`/FFI gate (M-661), phyla + cross-nodule model (M-662), and static guarantee grading
(M-663, RFC-0018 Enacted). **M-673 landed the monomorphization elaboration + dictionary-free
static trait resolution**, so generic and trait instantiations now elaborate to closed L0 and run
on all three paths (L1-eval ≡ L0-interp ≡ AOT); DN-14 §3 rows 6 and 7 are now `present`.

**1.0.0 gate defined and ratified (ADR-021, Accepted 2026-06-21).** Gate A1 (zero open High
findings from the 2026-06-14 deep review) and Gate B2 (KC-2 verdict) are met. Open gate rows
are the critical path (DN-19): A2 (Medium-findings ledger), A3 (WS8 durability:
`cargo-mutants`/proptest/fuzz), A4 (`cargo deny`/`cargo audit` wired into `just check`). The
1.0.0 product scope is the kernel/core (interpreter, certified swaps, VSA/dense ops with
bounds, selection + EXPLAIN, the trusted toolchain); surface-language ratification is scoped to
a tracked `1.x`.

**In progress (Phase 4, parts of Phase 6):** the full interpreted↔compiled ABI + AOT env-machine
(mutual recursion, RFC-0012 ambient); the **real `ternary`→arith/vector→LLVM MLIR-dialect
lowering** (M-601, E6-1 — currently a direct-LLVM stand-in; unblocked by libMLIR provisioning
on Linux, M-348/ADR-019) + deployable spores.

**Not yet established:** self-hosting (M-502) — the stdlib is Rust-first; the Mycelium-lang
migration half is open. Surface-language and self-hosting are post-1.0 / 1.x scope (ADR-021 §5).
The native MLIR-dialect lowering (M-601/M-602) stays "not established" until the three-way
NFR-7 differential holds (VR-5), never pre-written.

Residual risks tracked in the Foundation risk register, notably **RR-13** (HRR/FHRR are the VSA
weak link). See `docs/Mycelium_Project_Foundation.md` §10 for the dependency-ordered action list
and `docs/planning/phase-*.md` for the live phase ladder.

---

## Technology stack

- **Kernel + reference interpreter:** Rust (MSRV **1.92**, ADR-007). The interpreter is the
  trusted base and the reference semantics (`crates/mycelium-interp`).
- **AOT path:** **MLIR → LLVM** (`crates/mycelium-mlir`), confined to the performance path.
  Built: the env-machine + the **direct-LLVM** native lowering of the data/closure/tail-recursion
  fragment (M-373/M-378/M-379), with JIT (M-340) and hot-inject (M-341). In progress: the real
  `ternary`→arith/vector→LLVM **MLIR-dialect** lowering (M-601, E6-1 — currently a direct-LLVM
  stand-in; unblocked by libMLIR provisioning on Linux, M-348). `vsa`/`embedding` dialects
  deferred.
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

## Suggested reading order

1. **`docs/Doc-Index.md`** — the map: every document, its status, and how they depend on each
   other.
2. **`docs/Mycelium_Project_Foundation.md`** — the charter: vision, requirements (FR/NFR/VR),
   success & kill criteria, ADRs 001–009, roadmap, risks.
3. **`docs/rfcs/RFC-0001…`** — the Core IR & metadata schema (everything else plugs into this).
4. **RFC-0002 → RFC-0032**, then the ADRs (010…024) and design notes (DN-01…28) for the deep
   dives — `Doc-Index.md` orders them.
5. **`crates/mycelium-core` and `crates/mycelium-interp`** — the kernel and reference semantics,
   if you want to read the design as code.
6. **`research/`** — the evidence base (records 01…13), if you want the "why" behind a decision.

---

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
| Surface/term-language layering (L0–L3) | RFC-0006; RFC-0007 | syntactic transparency invariants; the ten-node L1 kernel calculus; L3 = committed text syntax + co-equal projection layer (M-380, KC-2 verdict) |
| Runtime & concurrency model | RFC-0008 | RT1–RT7; deterministic-fragment-first; partial failure explicit; probabilistic guarantees |
| Structured diagnostics + declarative recovery | RFC-0013; RFC-0014 | additive over the never-silent error (never substitutive); declared, **bounded** effects |
| Standard-library scope + per-op contract | RFC-0016 | C1–C6 (never-silent · guarantee tags · EXPLAIN · content-addressed · above-the-kernel · bounded effects); ring layering; 25/25 specs Accepted |
| `spore` is the deployable unit | ADR-013 | content-addressed code + values + metadata; identity is the content hash (ADR-003) |
| Interpreted↔compiled ABI + hot-inject | ADR-016; ADR-017 | hash-keyed dispatch; content-addressed dynamic linking; immutable-by-construction |
| 1.0.0 release-readiness gate | ADR-021 | Gate A (honesty-integrity + durability) + Gate B (decision/external); kernel/core scope; surface → 1.x |

> The full set (RFC-0006…0023, ADR-011…021, DN-02…22) with status and dependencies is in
> [`docs/Doc-Index.md`](./docs/Doc-Index.md) — the table above is the load-bearing subset.

---

## Glossary

- **Substrate / paradigm** — one of the four representation families (binary, balanced ternary,
  dense embedding, VSA).
- **Balanced ternary** — base-3 with digits {−1, 0, +1}; symmetric, sign-is-a-digit. Used here
  as a *logical* substrate, forward-compatible with native-ternary hardware.
- **VSA / HDC** — Vector Symbolic Architectures / hyperdimensional computing: high-dimensional
  vectors with algebraic operations (bind, bundle, permute) for symbolic-connectionist computation.
- **Swap** — the explicit, certificate-emitting operation that changes a value's representation.
  The only such operation.
- **Guarantee lattice** — `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`: how trustworthy a value's
  accuracy claim is; composes by *meet* (weakest wins).
- **Certificate** — a checkable record that a swap (or a compiled artifact vs. the interpreter)
  meets a claimed `{ε, δ, strength}` bound.
- **Translation validation** — proving each *instance* of a lowering/conversion correct, rather
  than proving the whole compiler correct once.
- **Schedule-staged packing** — choosing a lossless physical layout at a lowering stage (a
  "schedule"), recorded as inspectable metadata, not encoded in the type.
- **`ErrorBound` / `ProbBound`** — the two ADR-010 bound kernels: error-magnitude (ε) via affine
  arithmetic; failure-probability (δ) via the union bound / approximate couplings.
- **Reconstruction manifest** — the explicit recipe (model, codebooks, compositional structure,
  decoding procedure, bound) needed to recover content from a VSA representation; distinguishes
  indexed retrieval from true compositional reconstruction.
- **Stable component** — a definition that is content-addressed + spec-ratified +
  verification-passed, and therefore eligible for AOT compilation.
- **EXPLAIN** — a first-class, queryable artifact that records why a selection, conversion, or
  approximation was made; required for any policy-driven or approximate operation (ADR-006,
  RFC-0005).

---

## Conventions for contributing

> Full detail (process, dev environment, workflow) is in [`CONTRIBUTING.md`](./CONTRIBUTING.md).
> In brief:

- **Decisions are append-only.** Don't silently edit an ADR/RFC decision — supersede it with a
  new status (`Draft/Proposed → Accepted → Enacted → Superseded`) and link forward. Every claim
  cites its grounding (survey labels `G*`/`A–E`/`R*`; research labels `T0.x/T1.x/T2.x`).
- **Transparency rule.** Guarantee tags are assigned **per model and per operation**, never in
  aggregate. A bound may be tagged `Proven` *only* if it cites a theorem whose side-conditions
  are checked; otherwise it is `Empirical` (validated) or `Declared` (user-asserted, always
  flagged). New results may *upgrade* a tag; absence keeps it weaker.
- **No black boxes.** Any feature that introduces opaque behavior (especially "intelligent"
  automatic selection) must be reified, inspectable, and explainable (`EXPLAIN`).
- **Engineering principles** (the project's house style): SRP, OCP, LSP, ISP, DIP, DRY, KISS,
  YAGNI, Law of Demeter, separation of concerns; **composition over inheritance**; PEP 8 /
  `ruff format` for Python.
- **Squash-only into `main`.** Every PR lands as a single curated squash commit (a clean linear,
  bisectable history); internal swarm integration merges (leaf→epic→orch) stay octopus/`--no-ff`
  to preserve lineage. The `/land` skill drives the autonomous self-review → green `just check`
  → curated squash-merge → cleanup loop.
- **Kill criteria** (KC-1…KC-4) are re-checked at every phase gate; a gate that doesn't check
  them is hiding risk.

---

## Provenance & evidence

Everything in `docs/` traces back to the research passes recorded in `research/` — now
**thirteen records** (`01…13`), spanning the prior-art survey and T0/T1/T2 findings through the
language layer, runtime/concurrency, error-recovery & bounded effects, automatic-baseline
diagnostics, the narrative-authoring pipeline, honest-stdlib prior art, stage-1 grading
non-interference, traits/coherence & Repr-polymorphism, the semantic-projection framework,
the web-tooling phylum (RFC-0022), and the ADK phylum (RFC-0023).
Each record carries its structured findings + source list; normative claims in `docs/` cite
their grounding (survey labels `G*`/`A–E`/`R*`; research labels `T0.x…T13.x`) or are flagged
as open questions.

---

## License

MIT — Copyright (c) 2026 **Tyler Zervas**. See [`LICENSE`](./LICENSE).
