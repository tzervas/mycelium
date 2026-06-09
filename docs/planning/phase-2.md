# Phase 2 — Honest Approximation & Selection (working plan)

| Field | Value |
|---|---|
| **Status** | **Living draft** (initial cut, 2026-06-09) |
| **Owns** | the concrete, issue-coupled decomposition of the Phase-2 epics (#28–#34) into `M-2xx` build tasks |
| **Source of truth above this doc** | `docs/Mycelium_Project_Foundation.md` §6 (roadmap), `docs/spec/SPECIFICATION.md` §10 (open build items 10.7–10.10), `tools/github/issues.yaml` + `idmap.tsv` (task ids), ADR-010/011 + RFC-0001…0005 + DN-01 (design corpus, all Accepted/Resolved) |
| **Mirrors** | the GitHub board: every task carries its issue number from `tools/github/idmap.tsv` |
| **Companion docs** | `phase-1.md` (predecessor, exit gate met at build level 2026-06-09); `phase-0.md`; `phase-3.md` (forthcoming, epics #35–#41) |

> **Grounding discipline.** This is a planning artifact, not a normative one. It cites the corpus
> (`FR/NFR/VR/SC/KC`, `RFC-xxxx §`, `ADR-0xx`, `Tx.y`, `G#`, `RR-#`) for every claim about *what* is
> required; it introduces no new requirements. Where it records a *decision about sequencing or
> scope* it says so and routes anything normative back to an RFC/ADR. The honesty rule applies to the
> gate verdicts below: a guarantee tag or kill-criterion verdict stays at the strength actually
> *established* by a checked run (VR-5), never pre-written.

---

## 1. What Phase 2 is for

Phase 1 delivered a small, auditable, executable kernel — but with three honest gaps documented
in-code, all by design: **bound composition does not exist** (the interpreter explicitly *refuses*
to compose an approximate input, `EvalError::ApproxCompositionUnsupported`); only the **bijective**
binary↔ternary swap exists (no lossy/`Bounded` swaps); and there is **no selection policy / EXPLAIN**
(packing is a default schedule, not selected). Phase 2 closes those gaps — it makes Mycelium's
*honest approximation* and *inspectable selection* real, in dependency order.

Its deliverables map to SPEC §10.7–§10.10 and Foundation §6 Phase-2:

1. The **verified-numerics foundation** (ADR-010): two bound kernels — `ErrorBound` (ε, affine
   arithmetic) and `ProbBound` (δ, union-bound / apRHL) — meeting at one shared `{ε, δ, strength}`
   certificate with a tier-i Rust checker, then wired so the interpreter composes approximate inputs
   honestly (**M-201…M-204**; E2-4 / #31). *Foundational — unblocks everything below.*
2. The **full swap surface + the single shared certificate checker** (translation validation, shared
   interp↔AOT): the split regime, the first `Bounded`/lossy swap, KC-4 overhead, SC-3 global
   (**M-210…M-212**; E2-3 / #30).
3. The **selection-policy language + mandatory EXPLAIN** (RFC-0005): one total, non-learned,
   content-addressed decision-table mechanism, two sites (swap-target + packing) (**M-220…M-222**;
   E2-6 / #33, P0).
4. **Dense embeddings + Dense↔VSA swaps** with ε/δ bounds (**M-230/M-231**; E2-1 / #28).
5. The **remaining VSA models** (MAP-B, BSC, HRR, FHRR, sparse/SBC) under the RFC-0003 §4 honest tag
   matrix (**M-240…M-242**; E2-2 / #29).
6. The **schedule-staged packing selector** + the E3 wrong-layout soundness differential
   (**M-250/M-251**; E2-7 / #34).
7. The **reconstruction manifest** (**M-260**; E2-5 / #32).

### Phase-2 exit gate (what "done" means)

Phase 2 closes when **all** of:

- **Numerics foundation** — the ε/δ kernels compose with **Soundness / Monotonicity / Determinism**
  property-tested (RFC-0001 §4.7); the tier-i checker re-validates example certificates and rejects
  a too-tight one (ADR-010); and the interpreter **composes approximate inputs honestly** (the
  `ApproxCompositionUnsupported` refusal is retired for composable inputs).
- **Full swap + shared checker** — the single translation-validation checker validates both swaps and
  the interp↔AOT differential; ≥1 `Bounded`/lossy swap ships with an honestly-derived bound; SC-3
  holds globally (every swap certified, never silent); KC-4 overhead is **measured** and recorded.
- **Selection + EXPLAIN** — every automatic selection emits a valid, deterministic EXPLAIN record;
  one mechanism serves both the swap-target and packing sites; determinism + overrides tested
  (RFC-0005).
- **Dense + VSA breadth** — Dense↔VSA swaps satisfy SC-2 with tagged ε/δ bounds; the remaining VSA
  models implement the trait with tags matching the RFC-0003 §4 matrix (HRR/FHRR unbind stays
  `Empirical`).
- **Packing + reconstruction** — the packing selector records `meta.physical` and the E3 differential
  catches a mislabeled layout (NFR-7); the reconstruction manifest recovers a novel compositional
  combination.

Maps to Foundation §6 Phase-2 success metrics: SC-2 (new swaps), SC-3 (global), the KC-4 first
measurement, NFR-7 (wrong-layout), and the SC-5 EXPLAIN channel.

---

## 2. The Phase-2 task set (readiness at a glance)

All Phase-2 tasks, with issue number (`idmap.tsv`), priority, dependency, and **build readiness**.

| Task | Issue | Pri | Depends on | Maps to | Readiness |
|---|---|---|---|---|---|
| **M-201** ErrorBound (ε) affine kernel | [#48](https://github.com/tzervas/mycelium/issues/48) | P0 | M-101 (bound) | ADR-010 §1 / RFC-0001 §4.7 | **In progress** — keystone (§4) |
| **M-202** ProbBound (δ) union/apRHL kernel | [#49](https://github.com/tzervas/mycelium/issues/49) | P0 | M-101 (bound) | ADR-010 §2 / RFC-0001 §4.7 | **In progress** |
| **M-203** Shared `{ε,δ,strength}` cert + tier-i checker | [#50](https://github.com/tzervas/mycelium/issues/50) | P0 | M-201, M-202 | ADR-010 §3/§4 + Trusted base | **Ready after M-201/M-202** |
| **M-204** Interp honest approximate composition | [#51](https://github.com/tzervas/mycelium/issues/51) | P0 | M-201…M-203 | RFC-0001 §4.7 | **Ready after M-203** |
| **M-210** Shared TV certificate checker | [#52](https://github.com/tzervas/mycelium/issues/52) | P0 | E2-4, M-120/M-151 | RFC-0002 §2 / RFC-0004 §3 | Ready after E2-4 |
| **M-211** Bounded/lossy swap (F32→BF16) | [#53](https://github.com/tzervas/mycelium/issues/53) | P1 | E2-4, M-210, M-230 | RFC-0002 §5 / ADR-010 §1 | Ready after M-210 + M-230 |
| **M-212** KC-4 overhead + SC-3 global | [#54](https://github.com/tzervas/mycelium/issues/54) | P1 | M-210, M-211 | KC-4 / SC-3 | Ready after M-211 |
| **M-220** Decision-table SelectionPolicy | [#55](https://github.com/tzervas/mycelium/issues/55) | P0 | M-101…M-103 | RFC-0005 §2/§3 | Ready (parallel to E2-4) |
| **M-221** Mandatory EXPLAIN + LSP surfacing | [#56](https://github.com/tzervas/mycelium/issues/56) | P0 | M-220, M-140 | RFC-0005 §2.2/§4 / SC-5 | Ready after M-220 |
| **M-222** Wire selection into swap/packing sites | [#57](https://github.com/tzervas/mycelium/issues/57) | P1 | M-220, M-221 | RFC-0005 §4 | Ready after M-221 |
| **M-230** Dense{dim,dtype} ops | [#58](https://github.com/tzervas/mycelium/issues/58) | P1 | M-101 (Dense repr) | RFC-0001 §4.1 / RFC-0002 §5 | Ready after E2-4 (float bounds) |
| **M-231** Dense↔VSA swaps (ε/δ) | [#59](https://github.com/tzervas/mycelium/issues/59) | P1 | E2-4, M-210, M-230, VSA | RFC-0002 §5 / RFC-0003 | Ready after M-210 + M-230 |
| **M-240** VSA: MAP-B + BSC (Exact) | [#60](https://github.com/tzervas/mycelium/issues/60) | P1 | M-130 | RFC-0003 §4 | Ready after E2-4 (tags) |
| **M-241** VSA: HRR + FHRR (Empirical unbind) | [#61](https://github.com/tzervas/mycelium/issues/61) | P1 | M-130/M-132, E2-4 | RFC-0003 §4 / T1.2 | Ready after M-240 |
| **M-242** Sparse/SBC + §4 matrix + MAP-B nesting | [#62](https://github.com/tzervas/mycelium/issues/62) | P1 | M-240, M-241 | RFC-0003 §4 / RR-13 | Ready after M-241 |
| **M-250** Packing selector (I2_S/TL1/TL2) | [#63](https://github.com/tzervas/mycelium/issues/63) | P1 | E2-6 (M-222), M-112 | RFC-0004 §5 / DN-01 | Ready after E2-6 |
| **M-251** E3 wrong-layout differential | [#64](https://github.com/tzervas/mycelium/issues/64) | P1 | M-250, M-151 | RFC-0004 §8 / NFR-7 | Ready after M-250 |
| **M-260** Reconstruction manifest (ReconInfo) | [#65](https://github.com/tzervas/mycelium/issues/65) | P1 | VSA, E2-4 | RFC-0003 §6 | Ready after E2-4 + VSA |

Legend — **Ready**: can start now from the corpus + landed deps. **Ready after X**: a hard
dependency is open. **In progress / Done**: as the issue progresses; **Done** = landed, tests green,
issue closed.

---

## 3. Batch structure (the parallelization plan)

Phase 2 sequences into four batches; tasks **within** a batch touch different modules/crates and
parallelize, while batches serialize on real dependencies.

- **Batch E — verified numerics** (`mycelium-numerics`, new crate): **M-201** (ε) and **M-202** (δ)
  are independent kernels (different monoids — ADR-010/T0.1c) and parallelize; **M-203** (shared
  certificate + tier-i checker) joins them; **M-204** wires them into `mycelium-interp`. The
  selection track (**M-220/M-221**, `mycelium-select`) is independent of numerics and runs *alongside*
  Batch E.
- **Batch F — full swap** (depends on E): **M-210** (shared TV checker) → **M-230** (Dense ops, also
  needs nothing from F beyond E) → **M-211** (the first `Bounded` swap) → **M-212** (KC-4 + SC-3).
- **Batch G — breadth** (depends on E, partly F): the VSA models **M-240 → M-241 → M-242**, the
  Dense↔VSA swaps **M-231** (needs F's M-210), and the reconstruction manifest **M-260**.
- **Batch H — packing** (depends on E2-6 + lowering): **M-250** (selector) → **M-251** (E3
  differential).

---

## 4. Critical path & sequencing

```
 Batch E (mycelium-numerics + mycelium-select)
   M-201 ErrorBound (ε, affine) ─┐
   M-202 ProbBound (δ, union)  ──┤ (independent monoids — parallel)
                                 ▼
   M-203 shared {ε,δ,strength} cert + tier-i checker
                                 │
   CRITICAL PATH ▼
   M-204 interp composes approximate inputs honestly  ── retires ApproxCompositionUnsupported

   PARALLEL (independent of numerics):
   M-220 decision-table policy ─► M-221 EXPLAIN+LSP ─► M-222 wire (swap + packing sites)

 Batch F (depends on E):
   M-210 shared TV checker ─► M-230 Dense ops ─► M-211 Bounded swap (F32→BF16) ─► M-212 KC-4 + SC-3

 Batch G (depends on E, partly F):
   M-240 MAP-B/BSC ─► M-241 HRR/FHRR (Empirical) ─► M-242 sparse + §4 matrix + RR-13
   M-231 Dense↔VSA (needs M-210)      M-260 reconstruction manifest

 Batch H (depends on E2-6 + M-112):
   M-250 packing selector (I2_S/TL1/TL2) ─► M-251 E3 wrong-layout differential (NFR-7)
```

**Why M-201/M-202/M-203 are the keystone.** Every honest approximation in Phase 2 routes through the
two bound kernels and their shared certificate: the interpreter's approximate composition (M-204),
the `Bounded` swap's ε (M-211) and the checker that consumes it (M-210), the Dense↔VSA ε/δ (M-231),
the VSA `bundle`/unbind tags (M-240…M-242), and the reconstruction bound (M-260). So E2-4 is built
first; the selection track (E2-6) runs in parallel since it needs only the Core IR's `Meta`.

---

## 5. Gate verdicts — Phase-1→2 re-run of KC-1…KC-4 (honest status)

Per the honesty rule and VR-5, kill-criterion status is tracked at the strength actually
*established*. Re-run at the Phase-1→2 gate (Foundation Meta).

| Gate | Question | Phase-1→2 verdict (2026-06-09) | What moves it in Phase 2 |
|---|---|---|---|
| **KC-1** | Honest, usefully-tight bound for a core VSA op? | ✅ **confirmed (build)** — carried from Phase 1: M-001 LH probe SAFE; M-131 ships a `Proven` capacity bound via checked instantiation + ≥1e4-trial validation. No regression. | Phase 2 *extends* the pattern to MAP-B/BSC/HRR/FHRR/sparse (M-240…M-242) — each tagged at the strength its basis supports, never upgraded. |
| **KC-2** | LLM code-gen/reasoning survives the Mycelium surface? | **open — blocked (external)** — unchanged; M-002 (#3) needs LLM API access. *Structurally* unblocked by the M-110 interpreter + M-141 linter (a type-check-pass-rate harness now exists). | Out of Phase-2 scope to *run*; remains the open Phase-0 experiment. Honest verdict: not yet established. |
| **KC-3** | Kernel stays single-expert auditable? | **holding** — `mycelium-core` stayed small and by-construction-correct through Phase 1; VSA is behind the ADR-008 submodule boundary. | Phase 2 adds surface (numerics, swaps, selection, more VSA). Decision: keep numerics in a *separate* `mycelium-numerics` crate and selection in `mycelium-select` (SoC) so the core kernel does not balloon. Re-assess at the Phase-2 gate. |
| **KC-4** | Per-swap certificate-check overhead within budget? | **n/a yet** — first *measurable* when the shared checker (M-210) lands; the only swap today (M-120 bijective) references a cached lemma, no per-value proof. | M-212 measures it across the now-complete swap set and records an honest verdict vs the budget (the measured number, never pre-written). |

**KC-3 decision (sequencing/scope, 2026-06-09).** The two bound kernels and the selection mechanism
land as their own crates (`mycelium-numerics`, `mycelium-select`), *not* inside `mycelium-core`. This
keeps the trusted kernel auditable (KC-3 / SoC / ADR-010 "small trusted base") while the numerics
checker is a certificate *consumer*. Routed back to ADR-010 (trusted-base tiers) for the normative
basis.

---

## 6. Per-task detail (filled as tasks land)

### 6.1 M-201 — ErrorBound (ε) affine-arithmetic kernel · #48 · P0

*(to be filled on landing — goal/acceptance from the issue, what was delivered, honesty note)*

### 6.2 M-202 — ProbBound (δ) union-bound kernel · #49 · P0

*(to be filled on landing)*

### 6.3 M-203 — Shared `{ε,δ,strength}` certificate + tier-i checker · #50 · P0

*(to be filled on landing)*

### 6.4 M-204 — Interpreter honest approximate composition · #51 · P0

*(to be filled on landing)*

---

## 7. Risks & open questions

| Id | Item | Disposition |
|---|---|---|
| **T0.1c** | ε and δ do **not** share one composition algebra (settled negative). | Accepted as inherent (ADR-010): two kernels, one certificate. The crate exposes them as separate monoids meeting at `{ε,δ,strength}`; `strength` composes by `meet`. |
| **RR-12** | Dual-path semantic divergence (interpreter vs AOT). | Carried from Phase 1; the M-210 shared checker now folds the M-151 differential into one translation-validation surface, and M-251's E3 extends it to wrong-layout. |
| **RR-13** | MAP-B accuracy degrades past a nesting depth. | M-242 enforces/flags the limit explicitly — never a silent accuracy loss (G2). |
| **KC-3** | Integrative complexity → un-auditable kernel. | §5 decision: numerics + selection in separate crates; VSA stays behind ADR-008. Re-run KC-3 at the gate. |
| **KC-4** | Cert-check overhead unknown until the checker exists. | First measured by M-212; recorded honestly, not pre-budgeted. |
| **OQ (naming)** | Issue E2-5 (#32) says `recon-info.schema.json`; the ratified file is `reconstruction-manifest.schema.json`. | The ratified name is authoritative (SPEC §10 note); M-260 reconciles the issue text. |

---

## 8. How this doc stays honest

- **Append-only with status transitions**, mirroring the ADR/RFC discipline: this file moves
  `Living draft → ratified` only when the Phase-2 exit gate (§1) is met; task rows update in place as
  their issues progress, but gate verdicts (§5) never pre-record an upgrade.
- **Every task row carries its issue number** (`idmap.tsv` is the join key) so the board and this doc
  cannot silently diverge.
- **Progress is reported back to the issues** — each task's substantive output links its artifact from
  the GitHub issue, and the issue is closed when its acceptance is met (or left open with an honest
  note if blocked).

---

## Meta — changelog & maintenance

- **2026-06-09 (initial draft):** decomposed Phase-2 epics #28–#34 into 18 `M-2xx` tasks
  (#48–#65), created as sub-issues of their epics and appended to `idmap.tsv`. Records the readiness
  table (§2), the batch/parallelization plan (§3), the critical path with the E2-4 numerics kernels
  as keystone (§4), the honest Phase-1→2 KC-1…KC-4 re-run (§5), and a per-task detail skeleton (§6)
  to fill as tasks land. KC-3 sequencing decision: numerics + selection as separate crates.
- Maintain append-only; supersede, don't rewrite. Re-run KC-1…KC-4 at the phase gate (Foundation
  Meta). Keep `Proven|Empirical|Declared` verdicts honest per VR-5.
</content>
