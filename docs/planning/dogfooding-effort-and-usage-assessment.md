# Dogfooding Effort & Usage Assessment — how much will replacing all Rust with Mycelium cost?

| Field | Value |
|---|---|
| **Status** | **Declared** planning estimate (2026-07-01). A model built from **one** measured productivity sample + LOC heuristics — **not** `Empirical` (it is not a measured distribution) and **not** `Proven`. Wide error bars; assumptions are stated so they can be adjusted. |
| **Purpose** | Size the comprehensive-dogfooding track (ADR-036): reimplementing **all** of Mycelium *in* Mycelium, beside the Rust reference, until the Rust artifacts are superseded. Answers "how much usage (tokens) is likely required?" and "how much per nodule/module?" |
| **Feeds** | ADR-036 (dogfooding/public-release strategy), `docs/planning/self-hosting-port-ledger.md`, DN-66 (stdlib self-host readiness), E18-1 (self-hosting capstone) |

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

The 968 `.myc` lines vs 126,539 Rust lines is the headline: **the reference is ~99.5% Rust**; DN-66's
per-crate survey independently found **zero of 26 `mycelium-std-*` crates** clear even the narrower
RFC-0031 self-host bar today. The dogfood is a near-greenfield reimplementation effort, not a finishing pass.

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
  tokens all-in. At that throughput, the **~45M-token floor** is on the order of **dozens of
  feature-sized sessions**; the realistic all-in (~70–120M) is **many months** of sustained
  agent-driven work — consistent with the ~23-day, 1,274-commit velocity that produced the Rust
  reference in the first place, i.e. **dogfooding is comparable in size to building the reference again.**
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
