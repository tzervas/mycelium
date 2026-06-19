# Phase 5 — Self-Hosting & Core Library (working plan)

| Field | Value |
|---|---|
| **Status** | **Anticipated — decomposition drafted** (2026-06-17; scope firms at the Phase-4 gate. Not yet ratified.) |
| **Owns** | the concrete, issue-coupled decomposition of the Phase-5 **stdlib / Core Library** roadmap into `M-5xx` tasks |
| **Source of truth above this doc** | `docs/rfcs/RFC-0016-Core-Library-and-Standard-Library.md` (**Draft** — the scope + per-op contract + module taxonomy); `docs/Mycelium_Project_Foundation.md` §6 (roadmap); `tools/github/milestones.json` (the Phase-5 charter); the M-346 stdlib epic body (`tools/github/issues.yaml`) |
| **Mirrors** | the GitHub board: each task carries its issue number from `tools/github/idmap.tsv` **once minted** (Phase-5 numbers are assigned on the next `gh-sync-all.sh` run at the gate — the M-364…368 staging precedent) |
| **Companion docs** | `phase-2.md`, `phase-3.md` (predecessors); `RFC-0013/0014/0015` (the diagnostics/recovery/auto-baseline the charter self-hosts); ADR-010 (numerics), RFC-0003/0009 (VSA), RFC-0002 (swap), RFC-0008 (runtime) |

> **Grounding discipline.** This is a planning artifact, not a normative one. It cites the corpus for
> every claim about *what* is required and introduces no new requirements; the normative scope lives in
> **RFC-0016 (Draft)**. Phase 5 is **anticipated** (not yet ratified) — the descriptions say so, and every
> task is `status:needs-design`. Genuine open questions are **FLAGGED** (RFC-0016 §8), never invented into
> false-confident tasks (the planning analogue of never-silent, G2).

---

## 1. What Phase 5 is for

Phases 1–4 built the trusted base + the capability crates (numerics, swaps, VSA, selection, dense,
diagnostics, recovery) and a surface language. Phase 5 turns that into a **usable language**: a
**standard library** (the Core Library, RFC-0016) and the beginning of **self-hosting** — writing the
library, and eventually the diagnostics/recovery runtime, **in Mycelium-lang itself** (the M-346 "free of
other languages" goal).

Its deliverables map to the `milestones.json` Phase-5 charter and the M-346 epic:

1. The **Core Library RFC** — scope, the per-op guarantee/EXPLAIN contract, module boundaries, the
   Rust→Mycelium migration (**M-501**; RFC-0016). *The keystone — everything below depends on it.*
2. The **self-hosting readiness gate** — make "the surface is self-hosting enough" a *checkable* verdict,
   never pre-declared (**M-502**; the M-346 precondition).
3. The **stdlib modules** — decomposed from RFC-0016's taxonomy into per-module tasks: the Tier-A
   *differentiator* modules and the Tier-B *common* modules (**M-510…M-534**).
4. **Self-hosting the RFC-0013/0014 diagnostics + recovery** in Mycelium-lang (**M-520**; the charter
   names this specifically — the first, most honesty-load-bearing migration target).

### Phase-5 (anticipated) exit gate

Per `milestones.json`: *a stdlib module self-hosts with the guarantee/EXPLAIN contract every op must meet
(G2, VR-5, KC-3, ADR-003).* Concretely, Phase 5 closes when: RFC-0016 is **Accepted** (its §8 questions
resolved, §7 prior art traced into `research/`); the v0 module set lands in Rust meeting the §4.1
contract (each with its checked guarantee matrix); and **≥1 module self-hosts** in Mycelium-lang and passes
its migration differential (§4.6) — proving the contract holds when the library is written in the language
it describes. (Sequencing vs Phases 6–7 is a maintainer gate decision; the runtime track may precede.)

---

## 2. The Phase-5 task set (at a glance)

All `status:needs-design`, `priority:P3` (out-of-gate, anticipated). Issue numbers are minted at the gate.

### Keystone + gate

| Task | Type | Depends on | Maps to | Note |
|---|---|---|---|---|
| **M-501** Core Library RFC | design | M-346 | RFC-0016 (Draft) | the contract + taxonomy keystone |
| **M-502** Self-hosting readiness gate | design | M-346, M-320 | M-346 precondition | makes "self-hosting enough" checkable |

### Tier A — differentiator modules (RFC-0016 §4.3)

| Task | Module | Grounding |
|---|---|---|
| **M-515** | `core` / prelude (the honest value model) | RFC-0001 |
| **M-512** | `numerics` (honest ε/δ bounds) | ADR-010; M-201/202/203 |
| **M-516** | `swap` (certified representation change) | RFC-0002; M-120/210/211/231 |
| **M-513** | `vsa` / `hdc` (hypervectors + reconstruction + resonator) | RFC-0003/0009; M-130/240–242/260/350 |
| **M-517** | `ternary` (balanced ternary, bit/trit, packed) | FR-M2; M-111; RFC-0004 §5 |
| **M-518** | `dense` (typed dim-tracked tensors/embeddings) | RFC-0001 §4.1; M-230 |
| **M-519** | `select` / `explain` (selection DSL + EXPLAIN) | RFC-0005/ADR-006; M-220/221/222 |
| **M-510** | `diag` (structured diagnostics, self-hosted) | RFC-0013; M-345 |
| **M-520** | `recover` + self-host RFC-0013/0014 | RFC-0014; M-352 |
| **M-521** | `runtime` / `colony` (fungal concurrency surface) | RFC-0008; M-355–357 — *Phase-7-gated (Q4)* |
| **M-522** | `spore` (deployable / reconstruction manifest) | ADR-013; M-368 |
| **M-523** | `content` / `hash` (content-addressing primitives) | ADR-003; RFC-0001 §4.6 |

### Tier B — common / expected modules (RFC-0016 §4.4)

| Task | Module | Honesty crux |
|---|---|---|
| **M-511** | `collections` | value-semantic; no silent reorder |
| **M-524** | `text` / `string` | `parse` → `Result`, lossy encoding explicit |
| **M-525** | `math` | rounding/approx ops carry their tag |
| **M-526** | `iter` | total/terminating where the kernel guarantees it |
| **M-527** | `error` / `option` / `result` | propagation is the floor (I1) |
| **M-514** | `io` + `serialize` | substrate single-consumption (LR-8); round-trip checked |
| **M-528** | `fs` | every path/permission failure explicit |
| **M-529** | `time` | monotonic vs wall a typed distinction |
| **M-531** | `rand` | nondeterminism reified/named (RT3) |
| **M-532** | `cmp` / `convert` | lossy convert is explicit + fallible |
| **M-533** | `fmt` | dual human/machine projection (G11) |
| **M-534** | `testing` | a skipped check is reported, never a silent pass |

---

## 3. Batch structure & sequencing

```
 GATE (design-first):
   M-501 Core Library RFC (RFC-0016 Accepted; §8 resolved; §7 research traced)
        │  (everything below depends on the ratified contract + taxonomy)
   M-502 self-hosting readiness verdict  ── gates the Mycelium-lang migration half

 Batch P5-A — Ring 0/1 (capability surfaces, mostly wrap landed crates; Rust-first):
   M-515 core/prelude ─► (M-512 numerics, M-516 swap, M-513 vsa, M-517 ternary,
                          M-518 dense, M-519 select, M-510 diag) — parallel over their crates

 Batch P5-B — Ring 2 commons (depends Ring 0/1; Rust-first):
   M-511 collections, M-524 text, M-525 math, M-526 iter, M-527 error,
   M-514 io+serialize, M-528 fs, M-529 time, M-531 rand, M-532 cmp/convert,
   M-533 fmt, M-534 testing

 Batch P5-C — self-hosting (depends M-502 readiness):
   M-520 self-host diag+recover (the charter's named first migration target) ─►
        differential vs the Rust reference (§4.6) ─► further module migration

 DEFERRED / cross-phase:
   M-521 runtime/colony ── gated on the RFC-0008 constructs (Phase 7); FLAGGED (RFC-0016 §8-Q4)
   M-522 spore, M-523 content ── Ring-1, but spore deploy fully lands with the Phase-6 native path (M-620)
```

**Why M-501 is the keystone.** Every module is held to the §4.1 contract and ships a guarantee matrix
(§4.5); the contract + taxonomy + naming must be ratified before per-module folding, or the modules drift.
M-502 gates only the *Mycelium-lang* half — the Rust-first modules (Batches A/B) can proceed against the
RFC without waiting on self-hosting readiness.

---

## 4. Open questions (FLAGGED — carried from RFC-0016 §8)

| Id | Item | Disposition |
|---|---|---|
| **Q1** | The exact v0 module set + priority order | Maintainer sign-off. The five M-346 early candidates (`diag`/`collections`/`numerics`/`vsa`/`io+serialize`) are the safe v0 floor; the rest are proposed. |
| **Q2** | Naming (`std` phylum? module names vs crate names? themed lexicon?) | A DN-level decision (DN-02/06 lineage), not settled here. |
| **Q3** | Ergonomics vs the contract (tension A) | How much tag/EXPLAIN/effect machinery is implicit-but-inspectable (RFC-0012 lesson) vs always-explicit — a per-ring design pass. |
| **Q4** | `runtime`/`colony` (M-521) sequencing | Depends on RFC-0008 constructs (Phase 7); may live in a separate `runtime` phylum. Cross-phase FLAG. |
| **Q5** | The migration differential's bar (§4.6) | What a self-hosted module must match (observable results? tags + EXPLAIN?) for M-502 graduation — ties to NFR-7 for library code. |
| **Q6** | The `wild`/FFI floor (`io`/`fs`/`time`/`rand`) | The minimal audited FFI surface (ADR-014); whether it lives in `std` or a separate `std-sys` so pure `std` stays leak-free (LR-9). |

---

## 5. How this doc stays honest

- **Append-only with status transitions**, mirroring the ADR/RFC discipline: this file moves
  `Anticipated → Living draft → exit-gate met` only as the work actually progresses; it never pre-records
  a module as done. Task rows update in place as their issues land.
- **Every task traces to RFC-0016 + the cited corpus**; no module is invented — a module whose
  Mycelium-specific intent is unclear is a §4 open question, not a silent task.
- **Issue numbers are minted at the gate**, not faked here (the new-id-resolves-to-a-GitHub-number-only
  discipline; `idmap.tsv` records them on the `gh-sync-all.sh` run).

---

## Meta — changelog & maintenance

- **2026-06-17 — Created (anticipated; decomposition drafted).** Stands up the Phase-5 working plan
  alongside **RFC-0016 (Draft)**: the keystone Core Library RFC (M-501) + the self-hosting readiness gate
  (M-502), and the full stdlib module decomposition — Tier-A differentiator modules (M-510/512/513/515–523)
  and Tier-B common modules (M-511/514/524–534) — each grounded in its Accepted RFC/ADR and held to the
  RFC-0016 §4.1 per-op contract. Records the batch/sequencing plan (§3) and carries the six RFC-0016 §8
  open questions as FLAGs (§4). Anticipated, not ratified; all tasks `status:needs-design`/`P3`. Numbers
  minted at the Phase-5 gate. Append-only.
- **2026-06-17 — First design wave landed (specs, not ratification).** The per-module **design specs** now
  exist under `docs/spec/stdlib/` for the first wave — Tier-A `core` (M-515) / `swap` (M-516) / `ternary`
  (M-517) / `dense` (M-518) / `select` (M-519) / `content` (M-523) and Tier-B `iter` (M-526) / `math`
  (M-525) / `error` (M-527) / `cmp` (M-532) / `fmt` (M-533) — plus the **M-502 self-hosting-readiness gate**
  (honest verdict: *not yet established*). Each ships its load-bearing guarantee matrix (RFC-0016 §4.5) and
  C1–C6 conformance; cross-module seams are reconciled in the stdlib index §5 (the swap↔convert boundary and
  the numerics-ε ownership are *consistent*; the recurring naming/ergonomics items are corroborated RFC-0016
  §8-Q2/Q3 signal for the maintainer). These are **Drafts (needs-design)**, not ratification — RFC-0016
  (M-501) acceptance + the §8/§7 obligations remain the gate. The Rust-first implementations and the
  Mycelium-lang migration half are still downstream (the latter gated by M-502). Append-only.
- **2026-06-18 — Batch P5-A Rust-first enactment landed (code, not ratification).** With RFC-0016
  **Accepted** (M-501) and the readiness gate assessed (M-502), the first **Rust-first** stdlib code
  landed: the seven Ring-0/Tier-A crates `mycelium-std-{core,ternary,swap,dense,select,vsa,content}`
  (M-515/517/516/518/519/513/523), built as a **swarm of six sonnet agents fanned in by a single
  octopus merge** (the orchestrator owns every shared file; the pattern is recorded in `CLAUDE.md`).
  Each crate is a KC-3 consumer of one landed capability crate and **encodes its RFC-0016 §4.5
  guarantee matrix as checked data asserted in tests** (230 tests; workspace fmt/clippy/test green).
  Tags are at the honestly-supportable strength (VR-5) — e.g. `std.dense` accumulation ops are
  **downgraded to `Empirical`** pending M-512, `std.vsa` resonator is never `Proven`. The per-module
  specs move to **"implemented (Rust-first), pending ratification"**, *not* `Accepted`; ratification
  and the Mycelium-lang migration half (M-502-gated) remain. This advances Batch P5-A (Ring 0/1);
  Batch P5-B (Ring-2 commons) and P5-C (self-hosting) are still downstream. Append-only.
- **2026-06-18 — Batch P5-B Rust-first enactment landed (code, not ratification).** The Ring-2 commons
  half lands: the twelve Tier-B crates `mycelium-std-{collections,error,cmp,iter,math,text,fmt,testing,
  io,fs,time,rand}` (M-511/527/532/526/525/524/533/534/514/528/529/531), built as a **swarm of twelve
  sonnet agents fanned in by a single octopus merge** (the orchestrator owns every shared file). Each is
  a KC-3 consumer written **to the §4.1 contract over Ring 0/1** and **encodes its RFC-0016 §4.5 guarantee
  matrix as checked data asserted in tests** (722 tests; workspace fmt/clippy/test green). Tags are at the
  honestly-supportable strength (VR-5) — `std.math` transcendentals **downgraded to `Declared`** (unaudited
  libm floor), `std.rand` samplers `Declared`/`Empirical` and **never `Proven`**, `std.io` `deserialize`
  `Empirical` (proptest round-trip, no theorem). The effectful modules (`io`/`fs`/`time`/`rand`, and
  `math`'s transcendentals) ship a **fully-testable surface over an injectable substrate/source** and
  **FLAG the audited `wild`/FFI floor to the `std-sys` phylum (§8-Q6 / M-541)** rather than inventing it —
  pure `std` stays leak-free (LR-9). Further cross-module FLAGs recorded for the maintainer (not silently
  decided): the `recover` bridge (M-520; `error`/`testing`), the `std.diag` record substrate (M-510;
  `testing`), the `fmt`→`io` canonical-JSON delegation, and the early-termination fold primitive
  (RFC-0007 §4.8; `iter`). The twelve per-module specs move to **"implemented (Rust-first), pending
  ratification"**, *not* `Accepted`. This completes the **Rust-first** half of Batches P5-A+P5-B; the
  Mycelium-lang migration (M-502-gated) and Batch P5-C (self-hosting) remain downstream. Append-only.
- **2026-06-18 — Tier-A completion Rust-first enactment landed (code, not ratification).** The four
  remaining spec'd-but-uncoded **Tier-A Ring-1** modules land as Rust-first crates (a four-sonnet-agent
  swarm, scaffold-first then octopus-merged): `mycelium-std-numerics` (M-512, #153), `mycelium-std-diag`
  (M-510, #151) over a newly-extracted **`mycelium-diag` kernel crate**, `mycelium-std-recover` (M-520,
  #156 — the Rust-first half only; self-hosting stays Batch P5-C, M-502-gated), and `mycelium-std-spore`
  (M-522, #163 — the library/manifest half; native deploy stays Phase-6, M-620). This discharges the three
  P5-B seams (`math`→`numerics` ε carrier; `testing`→`diag` `Fail`-record; `error`→`recover`
  `Outcome`/`PolicyRef`) and **completes the Tier-A differentiator surface** (only the Phase-7-gated
  `runtime` M-521 remains). A **maintainer-resolved FLAG** (scaffold decision #1): the canonical RFC-0013
  record is homed in the extracted `mycelium-diag` kernel crate (a small, deliberate trusted-base growth),
  not inside the std crate. Each crate encodes its RFC-0016 §4.5 guarantee matrix as checked data and tags
  at the honestly-supportable strength (VR-5) — notably `recover` **fixes the P5-B exact-tag bug** (Ok→floor,
  fallback→`Declared`, never laundered up). Cross-module reconciliations are **FLAGGED for fast-follow**, not
  silently made (`testing` `FailRecord`→`Diag`; `error` stub→real `Outcome`; `mycelium-interp`
  `EffectBudget::Io`/`Named`; the `DECLARED_FLOAT_EPS` migration; the `regrow`→`Approx<Value>` wrapper). The
  four per-module specs move to **"Implemented (Rust-first), pending ratification"**, *not* `Accepted`. Full
  `cargo build`/`clippy --all-targets -D warnings`/`test --workspace` green (1883 tests). Append-only.
