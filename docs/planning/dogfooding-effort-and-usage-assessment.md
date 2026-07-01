# Dogfooding Effort & Usage Assessment — how much will replacing all Rust with Mycelium cost?

| Field | Value |
|---|---|
| **Status** | **Declared** planning estimate (2026-07-01). A model built from **one** measured productivity sample + LOC heuristics — **not** `Empirical` (it is not a measured distribution) and **not** `Proven`. Wide error bars; assumptions are stated so they can be adjusted. |
| **Purpose** | Size the comprehensive-dogfooding track (ADR-036): reimplementing **all** of Mycelium *in* Mycelium, beside the Rust reference, until the Rust artifacts are superseded. Answers "how much usage (tokens) is likely required?" and "how much per nodule/module?" |
| **Feeds** | ADR-036 (dogfooding/public-release strategy), `docs/planning/self-hosting-port-ledger.md`, DN-66 (stdlib self-host readiness), E18-1 (self-hosting capstone), **DN-34 (Rust→Mycelium transpiler strategy — the intended bulk mechanism, §5a; Draft, not implemented)**, DN-26 (self-hosting bootstrap plan) |

> **Transparency (VR-5).** This is a *forecast*, tagged `Declared`. Its central figure rests on a
> **single** measured code-production sample (this session's M-871 remote-registry agent) extrapolated
> by lines-of-code — a thin basis. Treat the ranges, not the point estimates, as the message; the
> dominant cost driver (language-capability readiness) is **not** LOC-estimable at all and is called
> out explicitly in §6. Two things this assessment **cannot** do, stated plainly (house rule #4):
> (1) it **cannot read the Claude Code weekly usage meter** — there is no programmatic surface for it
> from inside a session (`/usage` is interactive; no local quota file is accessible), so "usage
> remaining this week" must be read by the maintainer from `/usage` or the Anthropic Console and
> divided into the token estimate here; (2) it does not know the maintainer's plan limits, so it
> reports the work in **tokens**, not in "weeks", except as an illustrative division in §7.

## 1. What exists today (the footprint to supersede)

Measured on the workspace at `dev` (2026-07-01), Rust reference implementation:

| Metric | Value |
|---|---|
| Workspace crates (`mycelium-*`) | **51** |
| Non-test Rust | **126,539 LOC** across **287 modules**, **7,439 `fn`s** |
| Test Rust | **63,087 LOC** |
| Public API items (api-index) | **2,193** |
| Mycelium (`.myc`) written so far | **968 LOC** (≈ 0.5% — the dogfood has effectively not started) |
| Repo lifespan to reach this | **~23 days**, **1,274 commits** (2026-06-08 → 07-01), largely agent-driven |

The 968 `.myc` lines vs 126,539 Rust lines is the headline: **the reference is ~99.5% Rust**. But the
0.5% is not noise — **~877 of those `.myc` lines are the core-lib self-host slice** (`lib/std/`:
`option`/`result`/`cmp`/`iter`/`collections`/`text`/`fmt`/`math`), and **M-714–M-719 are all `done`**:
that slice **passes differential conformance** against its Rust twin, and **M-502 (surface-sufficiency
gate) is `done`** — i.e. the language is *proven* able to express core stdlib. What DN-66 found is the
next, stricter bar: **zero of 26 crates clear the RFC-0031 §5 D6 trigger to *retire* the Rust crate** —
the `.myc` core modules are conformance-passing **prototypes that coexist with** (do not yet replace) the
load-bearing Rust. So the dogfood is **seeded and de-risked at the core, not begun at the leaves.**

## 2. Productivity baseline (grounded, single sample)

The one clean, instrumented code-production sample this session:

- **M-871 remote-registry leaf agent (Sonnet):** **206,004 tokens** consumed → **~1,874 delivered lines**
  of production Rust + tests (`remote.rs` 1,226 + `remote_tests.rs` 598 + wiring ~50), green under
  `fmt`/`clippy -D warnings`/`test`. ⇒ **≈ 110 tokens / delivered line** at the subagent level.

This is the *closest available analog to a port*: the agent worked from a tight spec (ADR-037) the way
a port works from existing Rust. But it **understates** the full cost, which also includes:

- **Orchestration** (the parent designing ADR-037, the dense-map format, the PR, integration) —
  not in the 206k.
- **Review + differential validation** (ADR-036 mandates Rust≡Mycelium differential validation of every
  ported component — a second axis of work with no Rust-side analog).
- **Language-gap iteration** (see §6) — where Mycelium can't yet express a construct, cost balloons.

Rolling those in, a **fully-loaded** rate of **~180–500 tokens / line** is used below, tiered by how
much new language capability a crate's port demands. (Sanity check: the *whole* M-871 feature —
Explore agents + impl agent + orchestrator turns + 2 PRs — plausibly ran ~600–800k tokens for ~2,500
net new lines ⇒ ~250–320 tokens/net-line fully-loaded, consistent with the tier rates.)

## 3. The model

`tokens(crate) ≈ LOC_nontest × ρ × rate_tier`, where **ρ** (Mycelium-reimpl LOC ÷ Rust LOC) is taken as
**≈ 1.0** (value-semantics may cut boilerplate; the paradigm shift and explicit provenance add it back —
assumed to wash, ±30%), and `rate_tier` is the fully-loaded tokens/line:

| Tier | Crates | Rate (tok/line) | Why |
|---|---|---|---|
| **A — stdlib** (`mycelium-std-*`) | 26 | 180 | Lowest capability bar; most mechanical once the surface language + core I/O land. |
| **B — toolchain** (fmt, lint, check, sec, doc, lsp, cli, spore, proj, build, bench, diag) | 12 | 240 | Needs richer language features (process, FS, structured data); more integration. |
| **C — kernel + numerics** (core, l1, interp, mir-passes, cert, select, numerics, dense, vsa, stack, sched) | 11 | 340 | Self-hosting frontend is `needs-design`; the trusted base — highest correctness + differential-validation burden. |
| **D — AOT/MLIR** (`mycelium-mlir`) | 1 | 500 | Binds C++ `libMLIR`/LLVM; "rewrite in Mycelium" likely means a **native codegen** (RFC-0039), not a port — highest uncertainty. |

## 4. The estimate (per tier)

| Tier | Non-test LOC | Central tokens |
|---|---|---|
| A — stdlib | 41,123 | **7.4M** |
| B — toolchain | 25,322 | **6.1M** |
| C — kernel + numerics | 40,712 | **13.8M** |
| D — AOT/MLIR | 19,382 | **9.7M** |
| **Non-test port total** | **126,539** | **≈ 37.0M** |
| + Test port (63,087 LOC @ ~120/line, more mechanical) | 63,087 | **≈ 7.6M** |
| **Grand central (code + tests)** | | **≈ 45M tokens** |

**Range: ≈ 30M – 70M tokens** for the full LOC port (applying a 0.6×–1.6× band over the central for the
compounding ρ and rate uncertainty). Per-module average ≈ **126,539 ÷ 287 ≈ 441 LOC/module**, i.e.
**~80k–220k tokens per module** at tier rates — roughly *one M-871-sized agent run per module*, of which
there are **287**.

## 5. Per-crate breakdown (central estimate, top items)

| Crate | Tier | Non-test LOC | Central tokens |
|---|---|---|---|
| `mycelium-mlir` | D | 19,382 | 9.7M |
| `mycelium-l1` | C | 18,824 | 6.4M |
| `mycelium-core` | C | 6,755 | 2.3M |
| `mycelium-lsp` | B | 6,417 | 1.5M |
| `mycelium-vsa` | C | 4,596 | 1.6M |
| `mycelium-doc` | B | 3,976 | 1.0M |
| `mycelium-bench` | B | 3,910 | 0.9M |
| `mycelium-interp` | C | 3,560 | 1.2M |
| `mycelium-std-fs` | A | 3,243 | 0.6M |
| `mycelium-std-runtime` | A | 2,982 | 0.5M |
| *(41 more crates)* | A/B/C | ~48,894 | ~11.3M |

The full 51-crate table is reproducible from `just`/the workspace (`find crates -name '*.rs'`); the tail
is dominated by the 26 stdlib crates at 0.03M–0.6M each.

## 5a. Two rewrite mechanisms — this estimate is the *agent-port* (upper-bound) model

**The §4 figure prices per-module *agent* porting. That is the pessimistic mechanism.** The project's
intended bulk-rewrite mechanism is a **Rust→Mycelium transpiler** (**DN-34**, Draft/advisory — *not
implemented*, no code; seeded from the maintainer's `py2rust` + `py-rust-bridge` projects: AST-walk
transpilation + a never-silent "flag, don't guess" compatibility analyzer). It reshapes the cost curve:

| Mechanism | Cost shape | Where it applies |
|---|---|---|
| **Agent port (this doc's §4)** | ~LOC × rate, paid **per module, every time** | the fallback / refinement mechanism; the only option where no transpiler exists |
| **Transpiler (DN-34)** | a **bounded one-time build** of the transpiler, then **cheap deterministic transpile** + a smaller agent **refinement** pass per crate | the intended bulk mechanism once the Mycelium surface is a viable *target* |

Under the transpiler path the marginal cost of the mechanical bulk collapses (transpilation is a
compiler run, not an agent run); what remains is (a) the one-time transpiler build (itself an agent
effort, but sized like *one* toolchain crate — order ~1–5M tokens, `Declared`), (b) a **refinement +
differential-validation** pass per crate (a fraction of the §4 per-crate figure — call it 0.2–0.5×), and
(c) the same **language-capability gate** (§6), which the transpiler does **not** remove. So a
transpiler-accelerated all-in is plausibly **~0.3–0.5× the §4 LOC-port line** for the mechanical portion
(≈ **15–25M** instead of ~45M), **plus** the unchanged capability build. **Caveats (VR-5):** DN-34 is
Draft and gated on surface maturity; a transpiler's output still needs human/agent refinement to be
idiomatic + to clear the never-silent/guarantee-tag bar; and the transpiler cannot emit constructs the
language can't yet express — so it *accelerates* the port, it does not *bypass* §6. This row is
`Declared` and coarser than §4 (no measured transpiler sample exists).

**Cost to stand up + run the transpiler (Declared, order-of-magnitude):**

| Step | Estimate | Notes |
|---|---|---|
| **Build the transpiler** | **~2–5M tokens** | Sized as *one large toolchain crate* (~5–15k LOC: a Rust-AST→Mycelium construct mapper + never-silent "flag, don't guess" analyzer + FFI-bridge glue), **seeded** from the maintainer's `py2rust`/`py-rust-bridge` (not greenfield). Bounded, self-contained. |
| **Execute (transpile) the codebase** | **~0 agent tokens** | A deterministic compiler run per crate — near-zero *agent* cost once built. |
| **Refine + differential-validate output** | **~15–25M tokens** | 0.2–0.5× the §4 per-crate figure across the workspace — the real remaining work after transpilation. |
| **Language-capability gate (§6)** | **un-estimable (design-bound)** | Unchanged; the transpiler cannot emit what the surface can't express. |
| **Recommended first spike (PoC)** | **~1–3M tokens** | Map a handful of Rust constructs → Mycelium, transpile **one** small Tier-A stdlib crate, refine + differential-validate it end-to-end. De-risks DN-34 and yields the **first `Empirical` rate** to replace every `Declared` figure here. |

**"Do we have enough usage to build + execute it?"** — the honest split: the **build** (~2–5M) and a
**PoC spike** (~1–3M) are small, bounded increments that very likely fit a normal usage window; the
**full execute + refine** (~15–25M, atop the capability gate) is a **multi-session, multi-week** effort,
not a single budget window. But I **cannot read the weekly meter** from a session (Posture note) — check
`/usage` or the Console for tokens remaining and divide: if you have low-single-digit millions this week,
the **PoC spike is the right-sized, highest-value move** (it also converts these estimates from
`Declared` to `Empirical`); the full sweep should be planned across many windows and is explicitly a
non-tag-gating, within-1.0.0 track (ADR-036), so it need never be rushed into one budget period.

## 5b. The 26 stdlib crates + the full path (crates → transpiler → self-hosted)

**Are the 26 `mycelium-std-*` crates the Rust implementation?** Yes — all 26 (**41,123 non-test LOC**,
Tier A) are the load-bearing Rust. **~8 have partial `.myc` prototypes** (the M-714–M-719 core slice,
~877 lines) that pass differential conformance but do **not** yet retire their Rust twin (DN-66 / D6).

**Cost to fully self-host the stdlib (all 26 crates clear D6, Rust retired):**

| Model | Stdlib cost | Notes |
|---|---|---|
| **Agent port** | **~7.4M tokens** (range ~4.5–12M) | §4 Tier-A figure; the ~877 done lines barely dent it. |
| **Transpiler-accelerated** | **~3–5M tokens** for the stdlib portion | + the shared transpiler build (~2–5M, §5a). The core slice is the hand-written **seed**. |

**The full path — and the good news: its hardest gate is already partly cleared.** DN-34 gates the
transpiler on the surface being a viable *target*; **M-502 + the M-714–719 core slice prove the surface
can already express + differentially-validate core stdlib.** So for stdlib the path is unusually clear:

1. **Core-lib slice** (`option`/`result`/`cmp`/`iter`/`collections`/`text`/`fmt`/`math` in `.myc`,
   conformance-passing) — **DONE** (M-714–719). This is the transpiler's seed *and* its proof-of-target.
2. **Build the transpiler** (~2–5M, §5a), targeting the now-proven surface, seeded from `py2rust`.
3. **Transpile + refine the remaining ~18 stdlib crates + bring the 8 prototypes to D6-clearing parity**
   (~3–5M) — retiring the Rust stdlib crate-by-crate under Rust≡Mycelium differential validation.
4. **Surface features the leaf crates still need** (`fs`/`io`/`sys`/`time`/`rand` want runtime/effect
   capabilities the pure-value core slice didn't exercise) — incremental, some `needs-design` (part of §6).

The **stdlib is the cheapest, most de-risked tier to finish** (~3–8M tokens all-in depending on
mechanism, atop the transpiler build) and is the natural **first full-self-host milestone** after the
core slice. The **toolchain → kernel → AOT** tiers follow and carry the heavier §6 capability gate
(E18-1 is `needs-design`). *(All figures `Declared`; the first transpiled-and-retired stdlib crate
converts them to `Empirical`.)*

## 6. The dominant uncertainty (read this before the numbers)

**§4's estimate assumes the language can already express each crate. It largely cannot yet** — and that
gap, not the LOC port, is the real cost driver:

- **Self-hosting frontend** (E18-1 children M-739–M-742) is `status:needs-design`. The kernel crates
  (Tier C) cannot be ported *at all* until the language can compile itself; that capability build is
  **design-bound**, not LOC-bound, and is not in the §4 figure.
- **DN-66:** 0/26 stdlib crates clear the self-host bar today; several depend on runtime capabilities
  (`mycelium-std-runtime` is load-bearing) that need language features first.
- **AOT/MLIR (Tier D):** `mycelium-mlir` binds C++ `libMLIR`. Dogfooding it almost certainly means a
  **Mycelium-native codegen** (RFC-0039 direction), i.e. *new design*, not a translation of 19k lines —
  the 9.7M figure is a placeholder for "a large, mostly-new effort," not a port estimate.

**Implication:** the realistic total is the §4 LOC-port figure (**~45M tokens central**) **plus** a
capability-build cost that is currently un-estimable because the enabling epics are `needs-design`. A
defensible planning posture is to treat **~45M as a floor** for the mechanical reimplementation and
expect the capability build + differential-validation iteration to **multiply the kernel/AOT tiers by
2–4×**, pushing a realistic **all-in central toward ~70M–120M tokens**, spread across **hundreds of
agent runs** and many sessions. This is `Declared`; refine it as real port samples land.

## 7. Framing against weekly usage

I cannot read the weekly meter from here (see the Posture note). To convert:

- Read **tokens remaining this week** from `/usage` (or the Anthropic Console).
- This session produced **one** ~2,500-net-line feature (the registry + M-872) at a plausible ~1M+
  tokens all-in. At that throughput, the **agent-port ~45M-token floor** is on the order of **dozens of
  feature-sized sessions**; the realistic agent-port all-in (~70–120M) is **many months** of sustained
  agent-driven work — comparable in size to building the Rust reference again (the ~23-day, 1,274-commit
  velocity). **The transpiler path (§5a) is the lever that shrinks this** — plausibly to a ~15–25M
  mechanical portion + the capability build — which is why DN-34 names it the intended bulk mechanism.
  Neither path removes the language-capability gate (§6).
- Because dogfooding is a **within-1.0.0, non-tag-gating** track (ADR-036), this cost does **not** block
  the v1.0.0 tag or the artifact publish — it paces the road to the *public* release.

## 8. Recommended sequencing (cheapest-capability-first)

1. **Build the enabling capability** (self-hosting frontend, core I/O/process, effect surface) — the
   gate; port nothing kernel-side until it exists.
2. **Port Tier A stdlib leaves** (`std-cmp`, `std-core`, `std-iter`, `std-math`, …) as the first
   differential-validated dogfood — smallest per-crate cost, immediate confidence, exercises the
   registry (publish each ported phylum to GHCR — the M-871 path).
3. **Tier B toolchain**, then **Tier C kernel** (highest validation burden last).
4. **Tier D AOT** via native codegen (RFC-0039), treated as new design.
5. Replace each Rust crate only when its Mycelium twin is differential-validated and satisfies the
   maintainer (ADR-036); the repo flips public when the set is complete.

Track actuals against this in `docs/planning/self-hosting-port-ledger.md` and **replace this
`Declared` model with `Empirical` per-crate rates as real ports land** — the first few Tier-A ports will
sharply narrow every range above.
