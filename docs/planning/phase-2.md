# Phase 2 — Honest Approximation & Selection (working plan)

| Field | Value |
|---|---|
| **Status** | **Exit gate met** (2026-06-12 — all five §1 build conditions; was Living draft 2026-06-09) |
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

**Status (2026-06-12): all five conditions met.** Batches E/F (numerics, full swap + shared
checker), E2-6 (selection + EXPLAIN), Batch G (Dense + VSA breadth, reconstruction), and Batch H
(M-250 packing selector → M-251 E3 wrong-layout differential) have all landed at the task level. The
gate is **met**; the KC re-run is in §5 (KC-2 stays open as an explicitly out-of-scope external
probe — it is not one of the five gate conditions).

Maps to Foundation §6 Phase-2 success metrics: SC-2 (new swaps), SC-3 (global), the KC-4 first
measurement, NFR-7 (wrong-layout), and the SC-5 EXPLAIN channel.

---

## 2. The Phase-2 task set (readiness at a glance)

All Phase-2 tasks, with issue number (`idmap.tsv`), priority, dependency, and **build readiness**.

| Task | Issue | Pri | Depends on | Maps to | Readiness |
|---|---|---|---|---|---|
| **M-201** ErrorBound (ε) affine kernel | [#48](https://github.com/tzervas/mycelium/issues/48) | P0 | M-101 (bound) | ADR-010 §1 / RFC-0001 §4.7 | **Done (2026-06-09)** — `mycelium-numerics::error` |
| **M-202** ProbBound (δ) union/apRHL kernel | [#49](https://github.com/tzervas/mycelium/issues/49) | P0 | M-101 (bound) | ADR-010 §2 / RFC-0001 §4.7 | **Done (2026-06-09)** — `mycelium-numerics::prob` |
| **M-203** Shared `{ε,δ,strength}` cert + tier-i checker | [#50](https://github.com/tzervas/mycelium/issues/50) | P0 | M-201, M-202 | ADR-010 §3/§4 + Trusted base | **Done (2026-06-09)** — `mycelium-numerics::cert` |
| **M-204** Interp honest approximate composition | [#51](https://github.com/tzervas/mycelium/issues/51) | P0 | M-201…M-203 | RFC-0001 §4.7 | **Done (2026-06-09)** — refusal retired for additive arithmetic |
| **M-210** Shared TV certificate checker | [#52](https://github.com/tzervas/mycelium/issues/52) | P0 | E2-4, M-120/M-151 | RFC-0002 §2 / RFC-0004 §3 | **Done (2026-06-10)** — `mycelium-cert::check` |
| **M-211** Bounded/lossy swap (F32→BF16) | [#53](https://github.com/tzervas/mycelium/issues/53) | P1 | E2-4, M-210, M-230 | RFC-0002 §5 / ADR-010 §1 | **Done (2026-06-10)** — `mycelium-cert::dense` (M-101's `Dense` repr sufficed; M-230's *ops* remain open) |
| **M-212** KC-4 overhead + SC-3 global | [#54](https://github.com/tzervas/mycelium/issues/54) | P1 | M-210, M-211 | KC-4 / SC-3 | **Done (2026-06-10)** — `xtask kc4` + `tests/sc3.rs`; measured verdict in §6.7 |
| **M-220** Decision-table SelectionPolicy | [#55](https://github.com/tzervas/mycelium/issues/55) | P0 | M-101…M-103 | RFC-0005 §2/§3 | **Done (2026-06-10)** — `mycelium-select` |
| **M-221** Mandatory EXPLAIN + LSP surfacing | [#56](https://github.com/tzervas/mycelium/issues/56) | P0 | M-220, M-140 | RFC-0005 §2.2/§4 / SC-5 | **Done (2026-06-10)** — `Explanation` + LSP channel |
| **M-222** Wire selection into swap/packing sites | [#57](https://github.com/tzervas/mycelium/issues/57) | P1 | M-220, M-221 | RFC-0005 §4 | **Done (2026-06-10)** — swap site wired; packing adapter ready for E2-7 |
| **M-230** Dense{dim,dtype} ops | [#58](https://github.com/tzervas/mycelium/issues/58) | P1 | M-101 (Dense repr) | RFC-0001 §4.1 / RFC-0002 §5 | **Done (2026-06-11)** — `mycelium-dense` |
| **M-231** Dense↔VSA swaps (ε/δ) | [#59](https://github.com/tzervas/mycelium/issues/59) | P1 | E2-4, M-210, M-230, VSA | RFC-0002 §5 / RFC-0003 | **Done (2026-06-11)** — `mycelium-cert::dense_vsa` + the checker's δ-side |
| **M-240** VSA: MAP-B + BSC (Exact) | [#60](https://github.com/tzervas/mycelium/issues/60) | P1 | M-130 | RFC-0003 §4 | **Done (2026-06-11)** — `mycelium-vsa::{mapb,bsc}` |
| **M-241** VSA: HRR + FHRR (Empirical unbind) | [#61](https://github.com/tzervas/mycelium/issues/61) | P1 | M-130/M-132, E2-4 | RFC-0003 §4 / T1.2 | **Done (2026-06-11)** — `mycelium-vsa::{hrr,fhrr}` |
| **M-242** Sparse/SBC + §4 matrix + MAP-B nesting | [#62](https://github.com/tzervas/mycelium/issues/62) | P1 | M-240, M-241 | RFC-0003 §4 / RR-13 | **Done (2026-06-11)** — `mycelium-vsa::{sbc,matrix}` + RR-13 refusal |
| **M-250** Packing selector (I2_S/TL1/TL2) | [#63](https://github.com/tzervas/mycelium/issues/63) | P1 | E2-6 (M-222), M-112 | RFC-0004 §5 / DN-01 | **Done (2026-06-12)** — `mycelium-select::{bitnet_packing_policy,select_layout}` + `Meta::with_physical` |
| **M-251** E3 wrong-layout differential | [#64](https://github.com/tzervas/mycelium/issues/64) | P1 | M-250, M-151 | RFC-0004 §8 / NFR-7 | **Done (2026-06-12)** — `mycelium-mlir::{pack,run_with_layout}` + `tests/wrong_layout.rs` |
| **M-260** Reconstruction manifest (ReconInfo) | [#65](https://github.com/tzervas/mycelium/issues/65) | P1 | VSA, E2-4 | RFC-0003 §6 | **Done (2026-06-11)** — `mycelium-core::recon` + `mycelium-vsa::recon` |

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
| **KC-2** | LLM code-gen/reasoning survives the Mycelium surface? | **open — blocked (external)** — *running* M-002 (#3) still needs LLM API access. **Harness landed (2026-06-10)**: `experiments/mycelium_experiments/kc2/` — the fixed 8-task benchmark (minimal Mycelium fragment vs a Python-embedded DSL baseline, both arms with checked reference solutions proving well-posedness), the `myc-check` oracle (parse / typecheck / task-signature exit codes), and the generate→check→feedback loop measuring syntactic validity, first-attempt type-check pass rate (the SC-5b number), and edit-to-fix iterations. The report hard-codes "verdict: not established" (VR-5 — never pre-written). | Out of Phase-2 scope to *run*; plug an LLM generator into the `Generator` protocol when API access exists. Honest verdict: not yet established. |
| **KC-3** | Kernel stays single-expert auditable? | **holding** — `mycelium-core` stayed small and by-construction-correct through Phase 1; VSA is behind the ADR-008 submodule boundary. | Phase 2 adds surface (numerics, swaps, selection, more VSA). Decision: keep numerics in a *separate* `mycelium-numerics` crate and selection in `mycelium-select` (SoC) so the core kernel does not balloon. Re-assess at the Phase-2 gate. |
| **KC-4** | Per-swap certificate-check overhead within budget? | **measured (2026-06-10, M-212)** — cert checks cost the same order as the swap itself (bijective ≈1.3× of a ~1.3 µs swap; bounded ≈0.13× of a ~16 µs swap; observational ≈10 ns) → the downgrade path is **not** triggered on this evidence. See §6.7 for the numbers + caveats. | A *ratified numeric budget* is still pending (Foundation says "an agreed budget" — a maintainer decision); re-measure on representative hardware when one is set. |

**KC-3 decision (sequencing/scope, 2026-06-09).** The two bound kernels and the selection mechanism
land as their own crates (`mycelium-numerics`, `mycelium-select`), *not* inside `mycelium-core`. This
keeps the trusted kernel auditable (KC-3 / SoC / ADR-010 "small trusted base") while the numerics
checker is a certificate *consumer*. Routed back to ADR-010 (trusted-base tiers) for the normative
basis.

**Phase-2 exit-gate KC re-run (2026-06-12, after Batch H).** Re-run at the Phase-2 gate per the
honesty rule (VR-5 — strength as actually *established*):

- **KC-1 — ✅ confirmed (build), no regression.** Carried from Phase 1 (M-001 LH probe SAFE; M-131
  `Proven` capacity bound by checked instantiation). Batch G/H added MAP-B/BSC/HRR/FHRR/SBC each
  tagged at the strength its basis supports (the §4 matrix as a checked table) and the
  trial-validated `EmpiricalProfile` pattern — extended, never upgraded.
- **KC-2 — open, blocked (external).** Unchanged: the M-002 harness landed (2026-06-10) but *running*
  it needs LLM API access; the report hard-codes "not established". **Out of the Phase-2 exit-gate
  scope** (the gate is the five §1 build conditions, which map to SC-2/SC-3/KC-4/NFR-7/SC-5 — not
  KC-2). Flagged, not silently closed.
- **KC-3 — ✅ holds at the gate.** The trusted kernel stayed single-expert auditable through all four
  batches: numerics, selection, and the **M-251 packing-byte codec** all live *outside* `mycelium-core`
  (`mycelium-numerics`, `mycelium-select`, `mycelium-mlir`); core gained only small by-construction
  records (`Meta.reconstruction`, `Meta.with_physical`) and the existing VSA submodule boundary
  (ADR-008) held. The §6.9-decision crate split did its job.
- **KC-4 — measured (2026-06-10), unchanged by Batch H.** The packing layout check rides the existing
  `ObservationalEquiv` instance (≈10 ns in §6.7) — no new per-swap certificate, so the M-212 verdict
  (cert-check cost is the same order as the swap; downgrade path **not** triggered) stands. The
  numeric-budget ratification remains a pending maintainer decision (re-measure on representative
  hardware when set).

**Verdict:** the **five Phase-2 exit-gate build conditions (§1) are met**; KC-1/KC-3/KC-4 are honestly
re-confirmed at the gate, KC-2 stays open as an explicitly out-of-scope external probe. The doc moves
to **exit-gate met** (§8) on this basis.

---

## 6. Per-task detail (filled as tasks land)

### 6.1 M-201 — ErrorBound (ε) affine-arithmetic kernel · #48 · P0 · done 2026-06-09

- **Goal / acceptance (from issue).** Affine-form ε composition (`add`/`sub`/`mul`/`neg`/`scale`)
  with a radius→`eps` projection; Soundness/Monotonicity/Determinism each property-tested.
- **Delivered.** `mycelium-numerics::error`: `AffineForm` (`x₀ + Σxᵢ·εᵢ`, noise symbols `εᵢ∈[−1,1]`),
  exact linear ops (shared symbols cancel — the correlation advantage over interval arithmetic) and a
  sound `mul` (second-order remainder `≤ rad(x)·rad(y)` onto a fresh symbol). The scalar
  `ErrorBound{eps,norm}` projection carries the conservative magnitude composition used when the
  affine structure isn't available (the interpreter's case). Property tests: linear ops are *exact*
  for every noise assignment; `mul` is sound (true product ∈ `[center±radius]`); scalar `add/sub/
  scale/mul` upper-bound true deviations; monotone; deterministic; norm-mismatch refused (`None`,
  never silent). 20k trials each.
- **Honesty.** The kernel only ever *degrades* strength (composition is monotone-downward); norm
  mixing is an explicit `None`, not a coercion (G2).

### 6.2 M-202 — ProbBound (δ) union-bound kernel · #49 · P0 · done 2026-06-09

- **Goal / acceptance (from issue).** `union(δ₁..δₙ)=min(1,Σδ)`; apRHL `[SEQ]` `⟨ε,δ⟩`; Soundness/
  Monotonicity/Determinism tested; does *not* supply VSA crosstalk content (ADR-010 §5).
- **Delivered.** `mycelium-numerics::prob`: `ProbBound{delta}` with `union` (saturating at 1 — a sound
  over-approximation) and `or`; `ApRhlJudgment{eps,delta}` with `seq` (ε adds as the `e^ε` factors
  multiply, δ adds, both clamped — ADR-010 §2). Tests: union over-estimates the empirical "any fails"
  rate of independent events (200k Monte-Carlo trials); monotone + saturates; deterministic; `[SEQ]`
  composes and saturates.
- **Honesty.** A different monoid from ε by construction (T0.1c); no VSA capacity/crosstalk content
  lives here (that stays in RFC-0003's cited-theorem path).

### 6.3 M-203 — Shared `{ε,δ,strength}` certificate + tier-i checker · #50 · P0 · done 2026-06-09

- **Goal / acceptance (from issue).** `Certificate{eps,delta,strength}` (strength by `meet`,
  serializable); tier-i checker rejects a too-tight certificate; `accuracy_to_probability` the single
  legal cross-kernel rule.
- **Delivered.** `mycelium-numerics::cert`: `Certificate` (serde round-trips; range-checked
  constructor — out-of-range δ refused); `recompute_error` (the kernel re-derivation), the tier-i
  `check_error_claim`/`check_union_claim` returning `Valid` / `Rejected{recomputed,claimed}` /
  `Malformed` — a claim *tighter* than the re-derivation is **rejected**, a looser one is `Valid`
  (sound, allowed); `accuracy_to_probability` (within tolerance ⇒ inherits the accuracy confidence,
  outside ⇒ honest worst case δ=1). Also `compose_error_bound` (the M-204 entry, §6.4): composes
  approximate inputs' `Error` bounds, meeting `strength` to the weakest input and deriving a matching
  basis (Proven⊕Proven stays Proven under the affine-composition citation; Proven⊕Empirical→Empirical
  carrying the fewest trials; …⊕Declared→Declared); a non-`Error` input ⇒ `None` (refuse, never
  fabricate).
- **Honesty.** Incompleteness of the checker is an explicit `Rejected`/`Malformed`, never a silent
  pass (RFC-0002 §2); strength is never upgraded without a checked basis (VR-5).

### 6.4 M-204 — Interpreter honest approximate composition · #51 · P0 · done 2026-06-09

- **Goal / acceptance (from issue).** Retire `EvalError::ApproxCompositionUnsupported` for composable
  approximate inputs; an exact-over-exact op stays `Exact`/`bound=None`; an op over approximate inputs
  carries the kernel-composed `Bound` + meet-strength; a golden test checks the propagated
  `{bound, guarantee}` against the kernel's direct composition; M-I1…M-I4 stay enforced.
- **Delivered.** `mycelium-interp::prims`: `exact_result` generalized to `compose_result`, which
  short-circuits to `Exact`/`bound=None` when all inputs are exact (M-I1) and otherwise composes per a
  per-prim `ApproxRule`: `core.id` → `Passthrough` (the bound is preserved verbatim, citation
  included); `trit.add`/`trit.sub`/`trit.neg` → `Error(Add|Sub|Neg)` (sound 1-Lipschitz affine ε
  propagation via `mycelium_numerics::compose_error_bound`); `bit.*` and `trit.mul` → `Refuse` (no
  defined ε rule — `trit.mul` needs the central-operand magnitudes that land with the Dense numerics,
  E2-1). Strength is the `meet` of the inputs' bases (Proven⊕Proven stays Proven; …⊕Declared →
  Declared), and the basis is re-derived to match (so M-I2…M-I4 hold through `Meta::new`). Five new
  golden tests: additive composition sums ε and keeps Proven; neg preserves ε; `core.id` passes the
  bound through; the meet degrades to Declared; `trit.mul` still refuses (explicit, never silent). The
  Phase-1 `bit.not` refusal test still holds (bit ops keep `Refuse`).
- **Honesty.** Refusing was the honest Phase-1 choice; composing-with-a-checked-kernel is the honest
  Phase-2 upgrade — but only where a *sound* propagation rule exists; the rest still refuse rather
  than fabricate (G2/VR-5). This closes the documented Phase-1 honesty gap (the interpreter could not
  compose approximate inputs).

### 6.5 M-210 — Shared TV certificate checker · #52 · P0 · done 2026-06-10

- **Goal / acceptance (from issue).** One `check(A, B, R, claimed-bound, certificate)` with a
  `RefinementRelation` (bijection | bounded-similarity | observational-equiv); Exact instances
  discharge by equality, bounded instances consume the E2-4 certificate; TV incompleteness is an
  explicit fallback, never a silent pass; the M-120 cert and the M-151 differential both validate
  through the one checker.
- **Delivered.** `mycelium-cert::check` (module `check.rs`): `check(a, b, relation, claimed:
  numerics::Certificate, evidence)` → `CheckVerdict::Validated{strength}` or
  `NotValidated{reason, fallback}`. **Bijection** re-checks the lemma reference
  (`roundtrip_lemma_ref`) and the `legal_pair(n, m)` side-condition (the honesty rule — `Proven`
  only with checked side-conditions), then discharges by *structural re-derivation equality*
  (re-run `enc`/`dec` on A, compare payloads with B — the computational analogue of the SMT/
  `decide` discharge, per-instance and cheap; no per-value proof objects). **BoundedSimilarity**
  measures the actual A↔B deviation in the certificate's own norm and re-validates twice through
  the E2-4 tier-i kernel (`check_error_claim`): the certificate ε must cover the measured instance,
  and the claimed ε must not be tighter than the certificate (VR-5 — a claim never outruns its
  checked evidence); a claimed *strength* above `basis_strength(basis)` is likewise rejected.
  δ-side and non-`Error` bounds are explicit `Incomplete` (lands with M-231). **ObservationalEquiv**
  discharges by structural equality of the NFR-7 observable `(repr, payload, guarantee)`; the M-151
  differential now validates every corpus pair through this instance (and its control test asserts
  the checker rejects a genuinely divergent pair). Every non-validation carries
  `Fallback::UseReference` (refuse the swap / run the trusted interpreter, ADR-007).
- **Honesty.** TV incompleteness is a typed `Incomplete{detail}` verdict — distinct from a
  `Diverged` counterexample — and never a pass (RFC-0002 §2). Theorem citations in a `ProvenThm`
  basis are accepted axiomatically; only the arithmetic instantiation is re-checked (RFC-0002 §7).

### 6.6 M-211 — Bounded/lossy swap (Dense F32→BF16) · #53 · P1 · done 2026-06-10

- **Goal / acceptance (from issue).** `dense_f32_to_bf16` emits the converted value + a `Bounded`
  cert with a basis-derived ε bound; the cert validates through M-210; NaN/Inf handling explicit.
- **Scope note (dependency).** The issue lists M-230 (Dense ops) as a dependency; what M-211
  actually needs is the `Dense{dim, dtype}` *representation* + `Scalars` payload, which landed with
  M-101/M-103 in Phase 1 — so this was built against that, and M-230 (Dense *operations*) stays
  open and independent.
- **Delivered.** `mycelium-cert::dense`: round-to-nearest-even `F32 → BF16` (bit-level, ties to
  even), emitting `SwapCertificate::Bounded` with the proven per-element relative bound
  `{eps: 2^−8, norm: Rel}` and a `ProvenThm` basis citing the standard round-to-nearest theorem
  (Higham 2002, Thm 2.2 instantiated at β=2, p=8) — side-conditions **checked per element** (finite;
  exactly an `f32`; zero or normal; no overflow on rounding), each violation a typed explicit
  `SwapError`. The result value discloses `Proven` + the same bound (M-I2) and records
  `policy_used`. Validates through the M-210 checker under `BoundedSimilarity`; a tampered
  conversion is caught by the tier-i measured-deviation check. `CertifiedSwapEngine` now serves
  the complete certified surface (bijective binary↔ternary + bounded F32→BF16 + identity), with
  explicit `UnsupportedSwap` elsewhere. Property test: 20k-sweep relative-bound soundness +
  idempotence (output on the bf16 grid).
- **Honesty.** Subnormal inputs and approximate sources are *refused* (explicit errors), not
  tagged with a bound the theorem doesn't cover (VR-5; RFC-0002 §5 "type error, not a `Declared`
  gamble"): the subnormal absolute-spacing bound and the input-bound composition rule (E2-1) are
  honest future work, recorded here.

### 6.7 M-212 — KC-4 cert-overhead measurement + SC-3 global exit · #54 · P1 · done 2026-06-10

- **Goal / acceptance (from issue).** A bench harness reporting cert-check cost per swap kind with
  an honest measured verdict vs the KC-4 budget; a test asserting every swap in the legal-pair
  table emits and validates a certificate (SC-3 global).
- **Delivered.**
  - **KC-4 harness:** `cargo run --release -p xtask -- kc4` times every implemented swap kind and
    its M-210 check (warmup + minimum-mean-of-5-batches; refuses to measure a debug build — its
    numbers would be dishonest to record). No bench dependency (house style).
  - **Measured (2026-06-10, containerized x86-64 CI runner, single run — indicative, not a
    calibrated benchmark):**

    | Swap kind | swap | cert check | check/swap |
    |---|---|---|---|
    | bijective enc `Binary{8}→Ternary{6}` | ≈1.3 µs | ≈1.7 µs | ≈1.3× |
    | bijective dec `Ternary{6}→Binary{8}` | ≈1.3 µs | ≈1.6 µs | ≈1.3× |
    | bounded `Dense{768} F32→BF16` | ≈16 µs | ≈2.0 µs | ≈0.13× |
    | observational interp↔AOT pair | — | ≈10 ns | — |

  - **Honest KC-4 verdict (this evidence):** per-swap certificate checking costs the *same order
    as the swap itself* (the bijective check re-derives the swap, hence ~1.3×; the bounded check
    is ~13% of its swap) — microseconds, not CompCert-level effort. On this evidence the KC-4
    downgrade path (certified → declared-and-property-tested) is **not** triggered. **Caveat:**
    the Foundation specifies "an agreed budget" but none is ratified; ratifying a numeric budget
    (and re-measuring on representative hardware) is a maintainer decision — this records the
    measured number, not a pre-written "within budget".
  - **SC-3 global:** `crates/mycelium-cert/tests/sc3.rs` — every *implemented* legal-pair row
    (bijective enc/dec over four `(n, m)` pairs; bounded F32→BF16) emits a certificate that
    validates through the one checker, and every rejected/unimplemented row (out-of-range,
    illegal pair, Dense↔VSA, cross-paradigm without a rule) is an explicit error through
    `CertifiedSwapEngine` — never silent, anywhere on the surface.
- **Honesty.** The unimplemented table rows are *part of* the SC-3 statement: SC-3 demands they
  fail explicitly until their swaps exist (M-231/M-242), and the test pins exactly that.

### 6.8 M-220 — Decision-table SelectionPolicy + cost function · #55 · P0 · done 2026-06-10

- **Goal / acceptance (from issue).** Ordered `(predicate over queryable Meta) → candidate` rules +
  explicit cost; total (default arm) and terminating by construction; deterministic;
  content-addressed; first-class override; fixed declared precedence.
- **Delivered.** New crate `mycelium-select` (the §5 KC-3 decision — selection stays out of the
  trusted kernel; depends on `mycelium-core` only). `SelectionPolicy{name, candidates, rules,
  default_choice, cost}` with private fields and a validating constructor (empty candidate set,
  dangling `Choose(i)`/default indices, degenerate cost weights are construction errors; the wire
  form re-validates on deserialize). `Predicate` is a small closed non-Turing-complete language
  (`Always | SrcKindIs | DtypeIs | GuaranteeAtLeast | ErrorEpsAtMost | DeclaredSparse | All | Any |
  Not`) — structural recursion on finite data, so evaluation is total and terminating. `CostModel`:
  cost = `storage_weight ×` storage **bits** (a real declared unit, not the "arbitrary internal
  units" failure mode RFC-0005 §2 documents; packing bits/element per RFC-0004 §5 / DN-01).
  `Action::Choose(i) | Cheapest` (ties → lowest index); first matching rule wins (fixed declared
  precedence); `policy_ref()` content-addresses the canonical serialization (RFC-0005 §3);
  overrides are a first-class `forced` argument, out-of-range → explicit error.
- **Honesty.** The "statistics" are the kernel's exact metadata — the cardinality-estimation
  opacity trap does not arise (RFC-0005 §2.5); every failure mode is a typed explicit error.

### 6.9 M-221 — Mandatory EXPLAIN trace + LSP surfacing · #56 · P0 · done 2026-06-10

- **Goal / acceptance (from issue).** A serializable `Explanation` populated on every selection
  (candidates + per-candidate cost + chosen + override state); `explain(policy, meta)` total and
  deterministic; the LSP facade exposes EXPLAIN as a surfaced artifact kind; a ranking test.
- **Delivered.** `Explanation{policy, policy_name, inputs, costs, matched_rule, chosen_index,
  chosen, overridden}` — emitted by `select` on **every** call (there is no selection without an
  EXPLAIN) and serde round-trips. `explain(policy, inputs)` is total (un-overridden selection on a
  validated policy cannot fail) and deterministic. `mycelium-lsp::analyze_with(node,
  &PolicyRegistry)` adds the **fifth artifact kind** to the M-140 facade: at every swap site whose
  `PolicyRef` resolves and whose source is statically known, the trace is re-derived and surfaced;
  a `policy-divergence` warning fires when the node's recorded target disagrees with the policy's
  choice (override or stale policy — visible either way). The ranking test hand-computes the
  64-vs-128-bit costs and pins the full trace.
- **Honesty.** Mandatory EXPLAIN is the operational form of "no black boxes" (G2/ADR-006); the
  divergence warning keeps even *overridden* selections inspectable.

### 6.10 M-222 — Wire selection into swap-target (and packing) sites · #57 · P1 · done 2026-06-10

- **Goal / acceptance (from issue).** Swap path records `Meta.policy_used = PolicyRef` + EXPLAIN
  on auto-selection; a single `select(policy, candidates, meta) → (choice, explanation)` serves
  both sites; override forces the alternate target deterministically.
- **Delivered.** One mechanism, two thin site adapters over the single `select`:
  `select_swap_target` (candidates must be `Repr`s) and `select_packing` (must be `PackScheme`s) —
  a wrong-kind candidate at a site is an explicit `WrongSiteKind` refusal, never a coercion. The
  wiring test selects a target for an exact Dense F32 value, builds the `Node::Swap` with the
  policy's content hash (WF2), runs it through the reference interpreter + `CertifiedSwapEngine`,
  and asserts the result's `Meta.policy_used` **is** the `PolicyRef` — "which policy chose this?"
  answerable from the value alone (RFC-0005 §3). The override path forces the alternate target
  deterministically across repeated calls. The packing site consumes the same entry point and is
  wired for real by E2-7/M-250 (its adapter + cost figures are already in place).
- **Honesty.** The packing-site *consumption* is honestly deferred to E2-7 exactly as the issue
  scopes it; nothing here pre-claims M-250.

---

### 6.11 M-230 — Typed dim-tracked `Dense{dim,dtype}` ops · #58 · P1 · done 2026-06-11

- **Goal / acceptance (from issue).** Dense construction over `ScalarKind` (`F32`/`BF16`) with dim
  in the type; elementwise + similarity ops with `Exact`/`Bounded` tags per op; dim mismatch a
  typed error; property tests; float-op ε via E2-4.
- **Delivered.** New crate `mycelium-dense` (KC-3: stays out of the trusted kernel, like
  numerics/selection): `DenseSpace{dim, dtype}` binds both in the type; `value` checks every
  element finite and **exactly on the dtype grid** (an off-grid payload contradicts its `Repr` —
  refused, never re-rounded). `add`/`sub`/`scale` are **`Proven`** with a per-element relative ε
  (`Rel`) and a `ProvenThm` basis — Higham Thm 2.2 with side-conditions checked per element
  (F32: native single rounding, `u = 2⁻²⁴`; BF16: the two-rounding composition
  `≤ 2⁻⁸ + 2⁻²³`, derivation in the crate docs); `neg` is **`Exact`** (never rounds);
  `dot`/`similarity` are `f64` measurement helpers (mirroring `VsaModel::similarity`).
  Mismatches (dim/dtype), NaN, off-grid, overflow, subnormal results, and approximate sources are
  typed explicit errors. A 20k-pair sweep per dtype exercises the disclosed bound (SC-2).
- **Honesty.** `F16`/`F64` are explicitly unsupported (no exact `f64` reference for `F64` ops);
  composing an *approximate input's* bound with the op ε remains the open magnitude-aware rule
  recorded at M-204/M-211 — refused via `ApproximateSource`, never fabricated.

### 6.12 M-240 — VSA models MAP-B + BSC · #60 · P1 · done 2026-06-11

- **Goal / acceptance (from issue).** `MapB`/`Bsc` implement the trait; bind/unbind/permute
  `Exact` (property-tested round-trips); bundle carries the honest capacity tag per the §4 matrix.
- **Delivered.** `mycelium-vsa::{mapb,bsc}`: MAP-B (bipolar, sign-rounded bundle, tie → first
  operand, documented) and BSC (binary, XOR bind, majority bundle, **centered Hamming
  similarity** `1 − 2·d_H/d`). Exact ops have Value adapters with `Derived` provenance; alphabet
  violations (`±1` / `{0,1}`) are the explicit `NonAlphabetComponent`. The **intrinsic** bundle
  tag is `Proven` per the matrix (MAP-B membership-only, Clarkson Thm 16; BSC on-expectation,
  Heim / Yi & Achour) — but the corpus carries a checked-instantiation *formula* only for MAP-I
  (M-131), so the **Value-level** bundles carry an **`Empirical`** δ instead, from declared
  `EmpiricalProfile` constants (odd m ≤ 5, d ≥ 1024, δ = 1e-2) **exercised with exactly the
  declared 10⁴ trials** in `tests/empirical_profiles.rs` (M-I3 — the basis is honest because the
  suite runs it).
- **Honesty.** A `Proven` value tag without a corpus formula would violate M-I2/VR-5; the honest
  path is the trial-backed `Empirical` downgrade plus an explicit refusal outside the validated
  profile (`OutsideEmpiricalProfile`).

### 6.13 M-241 — VSA models HRR + FHRR (Empirical unbind) · #61 · P1 · done 2026-06-11

- **Goal / acceptance (from issue).** `Hrr`/`Fhrr` implement the trait; bind algebraic; unbind
  tagged `Empirical` with a trial-validated `EmpiricalFit` bound, routed through cleanup;
  bind→unbind→cleanup recovery trials within the stated δ.
- **Delivered.** `mycelium-vsa::{hrr,fhrr}`: HRR (naive-`O(d²)` circular convolution — the
  trusted reference; correlation unbind) and FHRR (phase-angle components in `(−π, π]`;
  phase-add bind; complex-sum bundle with the explicit `DegenerateBundleComponent` refusal when a
  phasor sum vanishes). `unbind` is the **residual `Empirical` weak link** (matrix-pinned in
  `tests/matrix.rs`; FHRR stays `Empirical` even though pure-pair recovery is near-exact — the
  matrix is normative, never upgraded). The Value-level `unbind_values` carries a
  trial-validated δ profile (d ≥ 256, codebook ≤ 16, δ = 1e-2; 2×10³ / 10⁴ trials) and is
  **provenance-gated to the validated single-factor regime** (the input must be a
  `vsa.{hrr,fhrr}.bind` product) — the structural witness of T1.2's "single-factor" scope.
  Recovery routes through `CleanupMemory` with inspectable confidence/margin (FR-S4/G2).
- **Honesty.** Unbinding bundles/unknown vectors stays algebra (no tag is issued); multi-factor
  recovery is the Phase-3 resonator (RFC-0003 §6), not silently approximated here.

### 6.14 M-242 — Sparse/SBC + the §4 matrix + RR-13 · #62 · P1 · done 2026-06-11

- **Goal / acceptance (from issue).** A sparse model with declared sparsity as a static
  refinement + observed sparsity as runtime metadata (T1.3); the §4 matrix as a single
  source-of-truth table asserted in tests; MAP-B deep nesting an explicit flagged condition.
- **Delivered.** `mycelium-vsa::sbc`: sparse block codes (one-hot per block; blockwise index-add
  bind with an exact algebraic unbind; counting-Bloom bundle; within-block permute). Values carry
  the **declared** `Sparse{max_active: blocks}` class in the `Repr` and the **observed**
  `SparsityObs` in `Meta` (T1.3, both halves). `mycelium-vsa::matrix` encodes RFC-0003 §4 as
  `RFC0003_MATRIX` (6 models × 4 ops, row bases documented); `tests/matrix.rs` asserts every
  implemented model×op intrinsic tag equals the table, pins the HRR/FHRR `Empirical` unbind, and
  checks totality/closure. **RR-13**: bundling a MAP-B bundle is the explicit
  `NestedBundleUnsupported` refusal (provenance-detected; landed with M-240, tested there).
- **Honesty.** SBC bind/unbind are tagged `Proven` *as the §4 row states* even though the algebra
  is exact — encoding the normative row verbatim (downgrades are always honest); a Value-level
  SBC bundle bound is recorded future work (no corpus Bloom formula), not approximated.

### 6.15 M-231 — Dense↔VSA swaps with δ bounds · #59 · P1 · done 2026-06-11

- **Goal / acceptance (from issue).** `dense_to_vsa`/`vsa_to_dense` emit `Bounded` certs with
  ε and/or δ, basis derived (`ProvenThm` where a cited capacity theorem's side-conditions check;
  else `EmpiricalFit`); certs validate through M-210; SC-2 for the new swaps.
- **Delivered.** `mycelium-cert::dense_vsa`: `dense_to_vsa` encodes a **bipolar** `Dense{n,F32}`
  vector as the MAP-I superposition of signed atoms from a **deterministic versioned codebook**
  (`swap.dense_vsa.enc.v1`) — a genuine n-item bipolar bundle, so the T0.2 capacity theorem
  applies; `vsa_to_dense` decodes by signed correlation, **provenance-gated to enc products**.
  The δ basis is derived, never asserted: `ProvenThm` iff `vsa_dim ≥ requiredDim(n, δ)` (reusing
  `mycelium_vsa::capacity` — the M-131 checked instantiation), `EmpiricalFit` iff the
  trial-validated profile covers the instance (n ≤ 16, dim ≥ 32n, δ = 0.05; 10⁴ round-trip
  trials in `tests/dense_vsa.rs`), and an explicit `InsufficientCapacity` type-error elsewhere.
  The **M-210 checker's δ-side lands** (the recorded `Incomplete` placeholder retired):
  `ProbabilityBound` certs discharge by tier-i union-bound claim-vs-certificate plus
  deterministic re-derivation equality (which re-checks side-conditions and rejects a basis
  upgrade — VR-5). `CertifiedSwapEngine` serves both directions at the documented
  `DENSE_VSA_DEFAULT_DELTA`; SC-3 global updated (new rows emit-and-validate; uncovered
  instances stay explicit).
- **Honesty.** Only the bipolar subclass encodes — the weighted-superposition bound is not in the
  corpus, so general-real components are refused (`NotBipolar`), not tagged; vanished decode
  correlations are `AmbiguousDecode`, never an arbitrary sign.

### 6.16 M-260 — Reconstruction manifest (ReconInfo) · #65 · P1 · done 2026-06-11

- **Goal / acceptance (from issue).** A `ReconInfo` serializing to
  `reconstruction-manifest.schema.json` (the ratified name — the issue's `recon-info.schema.json`
  reconciled, closing the §7 naming OQ) distinguishing indexed retrieval from compositional
  reconstruction, content-addressed codebooks, attached bound; the compositional path recovers a
  **novel combination**; round-trip + reconstruction tests.
- **Delivered.** `mycelium-core::recon` (the kernel carries the metadata *field* per RFC-0003 §2):
  `ReconInfo` with a validating constructor + re-validating `Deserialize` enforcing the schema
  invariants — compositional ⇒ recipe required, indexed ⇒ recipe absent, `Cleanup` ⇒ threshold in
  `[0,1]`, `Resonator` ⇒ factors + budget **and probabilistic-only** (FR-C2: a `ProvenThm` basis
  is refused), bound well-formed. `Meta` gains the ratified `reconstruction` field
  (`with_reconstruction`; wire-optional — `meta.schema.json` already specified it; the meta.rs
  deferral note retired). `mycelium-vsa::recon::reconstruct_role` executes the manifest: unbind
  by a recipe-named role, clean up against the codebook, threshold by the manifest's own
  `cleanup_threshold` (below-threshold / unknown-role / non-compositional are explicit refusals).
  Acceptance test: `bundle(color⊗red, shape⊗cube)` with both pairs **absent from every codebook**
  is recovered role-by-role through the manifest (the §6 exit criterion), carrying a `Proven`
  capacity bound from the checked instantiation, surviving the full value wire round-trip.
- **Honesty.** The indexed-vs-compositional distinction is *operational* (an indexed manifest
  refuses the compositional path); resonator factorization stays Phase-3 with the
  probabilistic-only ceiling enforced in the type.

### 6.17 M-250 — Schedule-staged packing selector · #63 · P1 · done 2026-06-12

- **Goal / acceptance (from issue).** A packing candidate set + cost function; **exhaustive**
  selection over the fixed bitnet.cpp set (`I2_S`/`TL1`/`TL2`) through the M-222 `select` API,
  emitting an EXPLAIN trace; the chosen layout recorded in `meta.physical` (lossless, M-I5);
  determinism + override tested; an E1 perf-harness stub recorded.
- **Delivered.** `mycelium-select` (the **one** mechanism, RFC-0005 §4 — a thin wrapper over the
  M-222 `select_packing` adapter, no parallel selector): `BITNET_PACKINGS = [I2S, TL1, TL2]`;
  `bitnet_packing_policy()` builds the validated policy — the three candidates, a single
  `Always → Cheapest` rule over the **bits/element** cost model (`I2_S`/`TL1` = 2.0, `TL2` = 1.67
  b/w; RFC-0004 §5 / DN-01). `select_layout(policy, inputs, forced)` evaluates the cost model
  **exhaustively** over all three (the set is small + fixed — T1.4, not an autoscheduler) and maps
  the chosen `PackScheme` to the recorded `PhysicalLayout::TritPacked{scheme}`, returning it with
  the **mandatory EXPLAIN** (every candidate costed, the matched rule, the override state).
  `record_packing_layout` attaches it to a `Meta` via the new **`Meta::with_physical`** — a
  **lossless** record builder (**M-I5**: it touches only `physical`, leaving the guarantee, bound,
  and value untouched, so M-I1…M-I4 are preserved by construction). The exhaustive cheapest is
  `TL2` deterministically; a first-class `forced` override picks `I2_S` (index 0, the lossless
  multiply-add default) or `TL1`; an out-of-range override is an explicit `SelectError`. Tests
  (`tests/packing.rs`): the fixed candidate set, the costed EXPLAIN ranking, determinism, override
  (incl. out-of-range refusal), M-I5 losslessness on `Meta`, and size-invariance of the winner.
  The **E1 stub** is `cargo xtask e1`: it times the substrate packing codec's pack/unpack
  round-trip per scheme (the build-phase confirmation that staging is cheap to materialize), with
  an explicit "E1 verdict: not established" — the calibrated kernel benchmark awaits the native
  libMLIR/LLVM path (ADR-009), never pre-written (VR-5).
- **Honesty.** Packing is a **schedule concern, not a type distinction** (DN-01 §2/§6): the layout
  is chosen at a lowering stage and recorded as the inspectable `meta.physical`, never in the type
  or the content hash. The cost is real storage **bits**, not "arbitrary internal units"
  (RFC-0005 §2). The packing-codec *bytes* live in `mycelium-mlir` (the AOT crate), **not** the
  trusted kernel — `mycelium-core` gains only the tiny `with_physical` record (KC-3).

### 6.18 M-251 — E3 wrong-layout soundness differential · #64 · P1 · done 2026-06-12

- **Goal / acceptance (from issue).** Extend the M-151 interp↔AOT differential so a deliberately
  **mislabeled** `physical` layout **fails** observational equivalence through the M-210 checker
  (NFR-7) and a **correct** one passes — proving the layout record is sound-by-checking.
- **Delivered.** `mycelium-mlir::pack` — the **substrate byte-layout codec**: each `PackScheme` is
  a bijective trit↔byte encoding (`I2_S`/`TL1`/`TwoBitPerTrit` are the three 2-bit-code rotations,
  4 trits/byte; `TL2`/`FiveTritPerByte` are base-3, 5 trits/byte), so the three bitnet schemes are
  **mutually distinct** and reading a buffer under the wrong scheme misreads it. Decoding is
  **total** (an out-of-range code/byte folds `mod 3`) — a misread, never a panic; round-trip under
  the *same* scheme is the identity (property-tested, incl. the degenerate all-`Zero` case which
  still diverges across schemes). `relayout_trits(trits, packed_as, read_as)` models the hazard:
  pack under the true scheme, read under the recorded tag. `aot::run_with_layout` extends the
  M-151 differential to the packing stage — for a ternary result it materializes the buffer
  (`packed_as`) and reads it back under the recorded tag (`read_as`), recording the layout on the
  result's `Meta` (M-I5). The E3 test (`tests/wrong_layout.rs`): a **correct** tag (packed-as ==
  tag) is the identity ⇒ observably equal to the layout-agnostic reference ⇒ the M-210
  `ObservationalEquiv` checker **validates**; a **mislabeled** tag (packed-as ≠ tag) misreads the
  buffer ⇒ a different payload ⇒ the **same** checker reports `NotValidated{ Diverged }` (the
  circuit-breaker; fall back to the reference — ADR-007). The verdict flips **solely** on the
  layout tag (so a passing E3 is meaningful, not vacuous), and the true scheme is the one the M-250
  selector actually chooses (tying the soundness check to the selector it guards).
- **Honesty.** This is the Mycelium analogue of the MLIR-`transpose` / Rust-`packed` "wrong layout
  misreads memory" bug DN-01 §4 cites — the layout record is trusted **only because a wrong one is
  caught** (NFR-7). The real `pack`/`unpack` *native* kernels are deferred (the codec here is the
  honest skeleton that stands in for them, sufficient to exercise E3); the calibrated E1 perf
  answer awaits the native backend (ADR-009).

## 7. Risks & open questions

| Id | Item | Disposition |
|---|---|---|
| **T0.1c** | ε and δ do **not** share one composition algebra (settled negative). | Accepted as inherent (ADR-010): two kernels, one certificate. The crate exposes them as separate monoids meeting at `{ε,δ,strength}`; `strength` composes by `meet`. |
| **RR-12** | Dual-path semantic divergence (interpreter vs AOT). | Carried from Phase 1; the M-210 shared checker **has folded the M-151 differential in** (every corpus pair validates through the `ObservationalEquiv` instance, done 2026-06-10). **E3 landed (2026-06-12, M-251):** `run_with_layout` extends the differential to the schedule-staged packing stage — a mislabeled `meta.physical` tag misreads the buffer and is caught as `Diverged`, a correct one passes (NFR-7). The dual-path divergence risk now covers layout, not just lowering/ordering. |
| **RR-13** | MAP-B accuracy degrades past a nesting depth. | **Enforced (2026-06-11, M-240/M-242):** a MAP-B bundle input that is itself a MAP-B bundle (detected via provenance) is the explicit `NestedBundleUnsupported` refusal — never a silent accuracy loss (G2). |
| **KC-3** | Integrative complexity → un-auditable kernel. | §5 decision: numerics + selection in separate crates; VSA stays behind ADR-008. Re-run KC-3 at the gate. |
| **KC-4** | Cert-check overhead unknown until the checker exists. | **Measured** by M-212 (2026-06-10, §6.7): same order as the swap itself — downgrade path not triggered on this evidence. Numeric budget ratification still pending (maintainer). |
| **OQ (naming)** | Issue E2-5 (#32) says `recon-info.schema.json`; the ratified file is `reconstruction-manifest.schema.json`. | **Resolved (2026-06-11, M-260):** built against the ratified name; §6.16 records the reconciliation. |

---

## 8. How this doc stays honest

- **Append-only with status transitions**, mirroring the ADR/RFC discipline: this file moves
  `Living draft → exit-gate met` only when the Phase-2 exit gate (§1) is met; task rows update in
  place as their issues progress, but gate verdicts (§5) never pre-record an upgrade. **Transitioned
  2026-06-12** — all five §1 build conditions met after Batch H; the §5 KC re-run was run *at* the
  gate, not pre-written (KC-2 stays honestly open and out of the gate's scope).
- **Every task row carries its issue number** (`idmap.tsv` is the join key) so the board and this doc
  cannot silently diverge.
- **Progress is reported back to the issues** — each task's substantive output links its artifact from
  the GitHub issue, and the issue is closed when its acceptance is met (or left open with an honest
  note if blocked).

---

## Meta — changelog & maintenance

- **2026-06-12 (Batch H lands — Phase-2 exit gate met):** M-250/M-251 done — epic **E2-7 complete
  at the task level**, and with it **all five §1 exit-gate build conditions**. The schedule-staged
  packing selector (`mycelium-select::{bitnet_packing_policy, select_layout, record_packing_layout}`
  reusing the one E2-6 mechanism; the chosen `PhysicalLayout` recorded on `meta.physical` via the
  new lossless `Meta::with_physical`, M-I5), the E3 wrong-layout soundness differential
  (`mycelium-mlir::pack` byte codec + `run_with_layout` extending M-151 — a mislabeled layout is
  caught as `Diverged` through the M-210 checker, a correct one passes; NFR-7), and the E1
  perf-harness stub (`cargo xtask e1`). §2 rows, §6.17–§6.18, §7 RR-12, and the §5 Phase-2-gate KC
  re-run (KC-1/KC-3/KC-4 re-confirmed; KC-2 open + out of gate scope) added; doc status →
  **exit-gate met**. Remaining (out of the gate): the RFC-0006 ratification (#67, maintainer) and
  the KC-2 *run* (M-002, #3, LLM API access).
- **2026-06-11 (Batch G lands):** M-230/M-240/M-241/M-242/M-231/M-260 done — epics **E2-1, E2-2,
  E2-5 complete at the task level**. The Dense operational surface (`mycelium-dense`), the full
  VSA model breadth (MAP-B/BSC/HRR/FHRR/SBC) with the §4 matrix as a checked table and the
  trial-validated `EmpiricalProfile` pattern, the Dense↔VSA δ-certified swaps + the M-210
  checker's δ-side, and the reconstruction manifest (`ReconInfo` + `Meta.reconstruction` +
  the compositional novel-combination recovery). §2 rows, §6.11–§6.16, §7 RR-13/OQ-naming
  dispositions updated. Remaining for the Phase-2 exit gate: **Batch H** (M-250 packing
  selector → M-251 E3 wrong-layout differential).
- **2026-06-10 (E2-6 lands):** M-220/M-221/M-222 done — the `mycelium-select` decision-table
  policy language (total/terminating by construction, content-addressed, explicit bits-based cost),
  the mandatory serializable EXPLAIN + the LSP fifth artifact kind (`analyze_with` +
  `policy-divergence` warning), and the swap-site wiring (`Meta.policy_used = PolicyRef` through
  the real interpreter; packing adapter ready for E2-7). §2 rows + §6.8–§6.10 added.
- **2026-06-10 (E2-3 lands):** M-210/M-211/M-212 done — the shared TV checker
  (`mycelium-cert::check`, with the M-120 cert and the M-151 differential folded in as instances),
  the first `Bounded` swap (Dense F32→BF16, proven `Rel 2^−8` rounding bound), and the KC-4
  measurement + SC-3 global test. §2 rows, §5 KC-4 verdict (measured, downgrade not triggered;
  budget ratification pending), §6.5–§6.7, and the §7 RR-12/KC-4 dispositions updated. M-211
  scope note: it needed only the Phase-1 `Dense{dim,dtype}` repr; M-230 (Dense *ops*) stays open.
- **2026-06-09 (initial draft):** decomposed Phase-2 epics #28–#34 into 18 `M-2xx` tasks
  (#48–#65), created as sub-issues of their epics and appended to `idmap.tsv`. Records the readiness
  table (§2), the batch/parallelization plan (§3), the critical path with the E2-4 numerics kernels
  as keystone (§4), the honest Phase-1→2 KC-1…KC-4 re-run (§5), and a per-task detail skeleton (§6)
  to fill as tasks land. KC-3 sequencing decision: numerics + selection as separate crates.
- Maintain append-only; supersede, don't rewrite. Re-run KC-1…KC-4 at the phase gate (Foundation
  Meta). Keep `Proven|Empirical|Declared` verdicts honest per VR-5.
</content>
