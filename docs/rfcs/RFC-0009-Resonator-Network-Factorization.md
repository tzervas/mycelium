# RFC-0009 — Resonator-Network Factorization

| Field | Value |
|---|---|
| **RFC** | 0009 |
| **Status** | **Draft** (Phase-3 exploratory design; ratification is the maintainer's) |
| **Type** | Foundational / normative (once Accepted) — opt-in VSA submodule feature |
| **Date** | June 15, 2026 |
| **Depends on** | RFC-0003 (VSA submodule boundary, `VsaModel`, guarantee matrix, reconstruction manifest §6); RFC-0001 (`VSA` Repr, `Hypervector`, `CrosstalkBound`, lattice); ADR-008 (VSA optional submodule); ADR-010/011 (bound kernels, `BoundBasis`) |
| **Grounding** | **FR-C2**, **G4**, **G7**, **RR-5**; RFC-0003 §6; research **T0.2 / T1.2 / T2.2** |
| **Implements (task)** | **M-350** (Resonator-network factorization, opt-in, probabilistic) — this RFC is its *needs-design* deliverable: it fixes the convergence regime, the honest guarantee, and the never-silent contract **before** any factorization code is built (per the §6 risk note **RR-5/G4**). |

## 1. Scope

Recovering the **unknown factors** of a VSA binding product. Given a hypervector
`s = x₁ ⊛ x₂ ⊛ … ⊛ x_F` that is the bind of `F` factors, where each factor `xᵢ` is drawn from a
*known* finite codebook `Cᵢ` (an item memory), find the `(x₁, …, x_F)` that produced `s`. Single-factor
unbinding (RFC-0003 §3) needs **all-but-one** factor already known; this RFC is the case where *none*
are known and a brute-force search over `∏ᵢ |Cᵢ|` combinations is intractable.

This feature is **Phase-3 exploratory**, **opt-in**, **probabilistic-only**, and **never in the
kernel** (FR-C2; KC-3). It lives entirely in the VSA submodule (`mycelium-vsa`) behind the existing
`DecodeProcedure::Resonator` reconstruction mode (RFC-0003 §6), whose schema already **enforces** the
probabilistic-only ceiling in the type (`mycelium-core::recon`, A6 checks). Nothing here changes the
kernel, the swap machinery, or any existing guarantee.

**Out of scope.** Learning/initialising the codebooks (assumed given, content-addressed per the
manifest); factoring products whose factors are *not* in a codebook; any `Proven` convergence claim
(see §5 — convergence is not guaranteed, RR-5).

## 2. Problem & why a resonator

The clean inverse `unbind(s, xⱼ)` recovers `xᵢ` only when every *other* factor `xⱼ` is known and the
model's unbind is exact (MAP-I/BSC) — or approximately, with cleanup, when it is lossy (HRR/FHRR;
RFC-0003 §4 "Net"). With all factors unknown the estimates are mutually circular: estimating `x₁`
needs `x₂…x_F`, which need `x₁`. A **resonator network** (Frady, Kent, Olshausen, Sommer,
*Neural Computation* **32**(12), 2020) breaks the circularity by holding a *superposition* estimate of
each factor and updating them **in parallel** against the others' current estimates, letting the
codebook cleanup at each step pull the superpositions toward a self-consistent set of clean atoms —
a fixed point of the coupled update. It is iterative, bounded, and **best-effort**.

## 3. Algorithm (reference semantics)

Let `g = ⊛` be the model's `bind`, `g⁻¹` its `unbind` (RFC-0003 §3), `d` the dimension, and for factor
slot `i` let `Cᵢ = {cᵢ,₁ … cᵢ,ₖᵢ}` be its codebook. Stack `Cᵢ` as a `kᵢ × d` matrix.

```text
state:   x̂ᵢ(0) for i ∈ 1..F            -- initial factor estimates (§9 Q1: uniform codebook superposition)
update:  for each i, against the *snapshot* x̂(t) (parallel / Jacobi, not in-place — §8.1 P6):
           rᵢ      = g⁻¹( s, g_{j≠i} x̂ⱼ(t) )     -- "explain away" the other factors
           aᵢ      = cleanup_i( rᵢ )              -- project onto codebook Cᵢ (§9 Q2)
           x̂ᵢ(t+1) = aᵢ
decode:  ι(t) = ( argmaxⱼ sim(x̂ᵢ(t), cᵢ,ⱼ) )_{i∈1..F}   -- the per-slot top-atom *index tuple*
stop:    converged   iff the decoded index tuple ι is unchanged for one full sweep AND every slot's
                          top-similarity ≥ τ_lock (a discrete fixed point — NOT real-valued
                          vector-stability, §8.1 P3); OR
         exhausted   iff t reaches the manifest iteration_budget (RFC-0003 §6: ≥ 1); OR
         oscillating iff a previously-seen *index tuple* ι recurs within a bounded history window
                          (discrete cycle detection — the real-valued state rarely recurs bit-exactly
                          under softmax cleanup, so cycles must be detected on ι, §8.1 P3/§9 Q3)
```

`cleanup_i(r)` scores `r` against every atom of `Cᵢ` by the model's `similarity` and returns a
codebook-constrained estimate — either the **softmax-weighted superposition** `Σⱼ softmax(β·simⱼ)·cᵢ,ⱼ`
(the standard resonator, which keeps the network differentiable/contractive) or the **hard arg-max**
atom (a winner-take-all variant). The choice and the inverse-temperature `β` are design parameters
(§6 Q2), recorded in the manifest so the run is reproducible.

`g_{j≠i} x̂ⱼ(t)` is the bind of all *other* current estimates. For a **self-inverse** model
(`VsaModel::self_inverse()` — MAP-I/BSC) `g⁻¹ = g`, so the update is a single bind of `s` with all the
other estimates. For a non-self-inverse model the model's approximate `unbind` is used, and the result
is at best `Empirical` (RFC-0003 §4 matrix).

On stop, the decoded factors are the per-slot top-similarity atoms `argmaxⱼ sim(x̂ᵢ, cᵢ,ⱼ)`, each with
its similarity as a per-factor confidence.

## 4. Reification & EXPLAIN (no black box — G2)

A resonator decode is reified end to end, like every selection/conversion in Mycelium:

- **Manifest (static, content-addressed).** `DecodeProcedure::Resonator` already records the per-factor
  codebook references (`factors`) and the `iteration_budget` (RFC-0003 §6; `mycelium-core::recon`).
  This RFC adds the *parameters* `{ cleanup: Softmax{β} | ArgMax, init: …, τ_lock }` to the same
  manifest, so two runs of the same manifest are identical.
- **Run trace (dynamic, `EXPLAIN`-able).** A factorization yields an inspectable record: the
  per-iteration per-factor top-atom and its similarity (the *similarity trajectory*), the **stop
  reason** (`Converged | BudgetExhausted | Oscillating`), the iteration count, and the final
  per-factor confidences. `EXPLAIN` renders this trace; there is no opaque pass.

## 5. Honest guarantee (the crux — FR-C2 / G4 / VR-5)

Resonator factorization is **probabilistic-only**. The guarantee contract:

1. **Never `Proven`.** Convergence to the correct factors is **not guaranteed** — it almost always
   succeeds *within an operational-capacity regime* and fails (to a wrong fixed point, a limit cycle,
   or budget exhaustion) outside it (Frady et al. 2020 §4; RR-5). The bound basis is capped at
   **`Empirical`** for a model with an exact bind, and **`Declared`** otherwise; the
   `mycelium-core::recon` schema already **rejects** a `Resonator` decode whose basis exceeds
   `EmpiricalFit` (A6, FR-C2). This RFC does not relax that ceiling and **must not** be read as a basis
   to upgrade it (VR-5).
2. **Operational regime is a side-condition, checked.** The empirical success probability `1 − δ`
   holds only over a validated regime — the factor count `F`, the codebook sizes `kᵢ`, and the
   dimension `d` (the *operational capacity* `∏ᵢ kᵢ` relative to `d`). The prototype records this as a
   trial-validated profile **following the `EmpiricalProfile` pattern** (`mycelium-vsa`) but over the
   resonator's `{F, ∏ᵢ kᵢ, d}` axes — *not* the existing bundle `EmpiricalProfile` (whose
   side-conditions are `{max_items, min_dim}`; §8.1 P4); a `ResonatorProfile::check` refusal is an
   explicit `OutsideEmpiricalProfile`, never a stretched tag. The reported certificate is
   `{ ε, δ, strength = Empirical }` with the regime attached.
3. **`δ` is measured against ground truth, not self-reported convergence.** A resonator can *converge*
   (reach a stable index tuple) to a **wrong** fixed point; "converged" therefore does **not** mean
   "correct" (§8.1 P5 — both reference implementations conflate the two). The validated `δ` is the
   **correct-factor recovery rate measured against a brute-force oracle** on small instances (§10/§11),
   counting a wrong-fixed-point *or* a `BudgetExhausted`/`Oscillating` verdict as a failure. The
   convergence rate the reference implementations report is **not** the certificate quantity.
4. **Lossy-bounded, best-effort, ambiguity surfaced.** Even within regime the reconstruction is lossy
   (cleanup is a projection); the per-factor confidence **and margin** are reported (the existing
   `CleanupMemory::cleanup` already returns both — `Match { confidence, margin }`). A confidence below
   the manifest threshold is an explicit `BelowCleanupThreshold` refusal (the cleanup contract already
   exists, `mycelium-vsa`), and a **small margin** (an ambiguous slot whose top two atoms are
   near-tied) is an explicit ambiguity refusal (§9 Q5), never a silent low-confidence or coin-flip
   guess. The reference implementations report candidate lists but **never refuse** on either — the
   gap this item closes (§8.1 P5).

## 6. Never-silent & termination (G2)

- **Bounded.** The iteration budget (`≥ 1`, schema-checked) caps work — a non-converging run **cannot
  hang**; it stops with `BudgetExhausted`. (Both reference implementations are bounded by a
  `max_iterations` too — that part they get right; §8.1.)
- **Explicit non-convergence; only a clean `Converged` yields factors.** `BudgetExhausted` and
  `Oscillating` are **error verdicts**, not a returned "answer", and so is a `Converged` run that fails
  the §5.4 confidence/margin gate. A resonator decode returns `Result<Factors, ResonatorError>` (or an
  `Option`); the factor set is produced **only** on a `Converged` verdict whose every slot clears
  `τ_lock`, the confidence threshold, and the margin threshold. This mirrors the swap "never silent"
  rule (S1/G2). It is the **central correction over the prior art** (§8.1 P5): both reference
  implementations return a fully-populated factor struct *regardless* of convergence — `converged` is
  at most an ignorable boolean (and `infer_semantics` ignores it entirely) — so a wrong fixed point is
  handed back as an answer.
- **Convergence is a discrete fixed point, not vector-stability.** Convergence is tested on the decoded
  per-slot **atom-index tuple** (`ι` in §3), not on cosine-to-previous-iterate as both reference
  implementations do (§8.1 P3): under softmax cleanup the real-valued superposition keeps drifting even
  once the decoded atoms are stable, so a cosine-to-previous test either never trips or trips on a
  non-codebook point.
- **Determinism.** Given the manifest parameters and a **seeded** initialisation, a run is reproducible
  (the trace is a function of the inputs), so the differential/property tests are stable. The reference
  implementations seed initial estimates from an **unseeded** thread RNG (`SparseVec::random()` over
  `rand::rng()`), so their runs are *not* reproducible (§8.1 P1) — Mycelium fixes the seed in the
  manifest.

## 7. Integration (builds on what exists)

- **`VsaModel`** (RFC-0003 §3): the update uses `bind`/`unbind`/`similarity` already on the trait; no
  new trait method is required for the core loop. (`cleanup_i` is the existing `CleanupMemory` scored
  over a per-slot codebook; the softmax variant is a thin wrapper over `similarity`.)
- **`DecodeProcedure::Resonator`** (RFC-0003 §6 / `mycelium-core::recon`): the manifest hook, factor
  codebooks, iteration budget, and the probabilistic-only ceiling are *already in the kernel schema*.
  This RFC fills in the decode-side parameters and semantics.
- **`EmpiricalProfile`** (`mycelium-vsa`): the validated-regime gate and the explicit out-of-profile
  refusal already exist for bundle/unbind; the resonator reuses the same honesty machinery for its
  `{F, kᵢ, d}` regime.

So the buildable surface is small: a `resonator` module in `mycelium-vsa` implementing the §3 loop over
any `VsaModel`, returning the §4 trace and the §5/§6 verdict — gated by the §5 regime profile, with the
manifest plumbing already present.

## 8. Prior art (to mine during prototyping)

An internal reference implementation exists — **`tzervas/embeddenator-retrieval`** ("signature-based
retrieval and resonator for holographic engrams") and the sparse-ternary VSA ops in
**`tzervas/embeddenator-vsa`**. These are explicitly flagged by the author as *rough* and are to be
**mined for pitfalls** (init strategy, oscillation in practice, regime where it actually converges),
**not** copied — anything adopted must re-derive its guarantee under §5 and earn its tag (VR-5). They
are outside this RFC's verified basis (not read at authoring time); the prototype task (below) reads
them.

### 8.1 Findings from the reference implementations (M-350 mining — informative)

The two implementations were read (`embeddenator-vsa::resonator` — the fuller Frady-style network with
named per-slot codebooks, soft/hard cleanup, and gradient codebook *training*; and
`embeddenator-retrieval::core::resonator` — a thinner variant). Both confirm the §5/§6 contract is
necessary by getting these things wrong. Concrete pitfalls (P-numbers referenced above):

- **P1 — Non-deterministic init.** Initial estimates are `SparseVec::random()` over an *unseeded*
  thread RNG (`rand::rng()`), so a run is not reproducible — directly at odds with §6 determinism and
  the differential-test stability §11 needs. *Lesson:* seed the init in the manifest (§9 Q1).
- **P2 — Unbacked "self-inverse" on a lossy algebra.** Both unbind by *re-binding* (`unbound.bind(other)`),
  commented "self-inverse property for sparse ternary". But the sparse-ternary multiplicative bind is a
  merge-join over the **intersection** of supports — non-overlapping support is dropped, so it is
  **lossy and not exactly self-inverse** (`x.bind(y).bind(y) ≠ x` in general). Treating it as `Exact`
  self-inverse like MAP-I bipolar is exactly the kind of unbacked exactness the honesty rule forbids
  (VR-5). *Lesson:* the first target must be a model whose bind is *genuinely* exact-self-inverse
  (MAP-I/BSC); sparse/block codes are `Declared`, not `Empirical` (§9 Q6).
- **P3 — No oscillation detection; wrong convergence test.** Neither detects limit cycles at all — a
  non-converging run burns the full `max_iterations` and returns its last estimate. Convergence is
  tested by cosine-to-*previous-iterate* (`≥ 0.99`, resp. `delta < 0.001`), not by decoded-atom
  stability, so under soft cleanup a stable decode can read as "not converged" (wasting budget) and a
  2-cycle is never recognised *as* a cycle. *Lesson:* detect cycles on the discrete index tuple `ι`;
  converge on `ι`-stability + `τ_lock` (§3, §6, §9 Q3).
- **P4 — No regime gate, no `δ`.** Neither estimates a success probability or checks an operational
  regime; `embeddenator-vsa` tracks a `converged_count/total` *convergence rate* but no `{F, kᵢ, d}`
  gate and no oracle. *Lesson:* a `{F, ∏kᵢ, d}` `ResonatorProfile` with an explicit out-of-regime
  refusal (§5.2, §9 Q4).
- **P5 — Wrong fixed point returned as an answer; convergence conflated with correctness.** `factorize`
  always returns a fully-populated result; non-convergence is at most a `converged: false` flag
  (ignored by `infer_semantics`), and there is **no test asserting the recovered factors are the true
  factors** — the one "convergence" test even factorizes a *bundle* (superposition), not a *bind*
  product, and asserts only `factors.len() == 2` and `delta < 1.0`. *Lesson:* only a clean `Converged`
  verdict yields factors; everything else is an error verdict; `δ` is oracle-measured correctness, not
  self-reported convergence (§5.3, §6).
- **P6 — In-place (Gauss-Seidel) update, not parallel.** `embeddenator-vsa` mutates each slot's estimate
  *in place* within a sweep, so later slots see already-updated earlier slots — not the parallel
  (Jacobi) update Frady analyses. This changes the dynamics and the basin. *Lesson:* update against a
  snapshot of the previous sweep (§3).
- **P7 — Scope creep / silent fabrication in the neighbourhood.** `embeddenator-retrieval` folds
  data-recovery heuristics into the same type — byte-averaging interpolation and a **zero-fill last
  resort** — i.e. it silently fabricates data when it has nothing. The antithesis of never-silent;
  out of scope here, but a sharp reminder of the failure mode §6 exists to prevent.

What they get **right** and is worth keeping: bounded iteration; the soft-cleanup as a softmax-weighted
superposition with a temperature (= 1/β) and an optional top-k truncation (`embeddenator-vsa`); a
per-slot candidate list with similarities (the raw material for the confidence/margin contract); and
the basic §3 loop shape. None of it carries a guarantee tag, so all of it must re-earn its tag under §5
(VR-5).

## 9. Open design questions — proposed resolutions (pending ratification)

Each question is now answered from the §8.1 prior-art mining; the **proposed resolution** is the
design the prototype should build. The RFC remains **Draft** — these are recommendations the maintainer
ratifies, not ratified decisions. Two genuinely-empirical knobs (`β`, the regime-grid granularity) are
deliberately left to be *fit by trials* rather than asserted (VR-5), and are flagged as such.

- **Q1 — Initialisation. → Resolved: uniform codebook superposition, seeded.** Initialise each slot to
  the **equal-weight superposition of all its codebook atoms** (the Frady "uniform" start), which puts
  the network in the centre of the basin; the (small) tie-breaking randomness is drawn from a
  **manifest-recorded seed** so runs are reproducible (§6 determinism). This rejects the prior art's
  unseeded random init (§8.1 P1). A single-seeded-guess init stays available for the §10.3 ablation.
- **Q2 — Cleanup shape. → Resolved: softmax-weighted superposition default; arg-max recorded variant;
  `β` is trial-fit.** Default to the softmax-weighted superposition `Σⱼ softmax(β·simⱼ)·cᵢ,ⱼ` (smoother
  dynamics, the literature default and the `embeddenator-vsa` default), with hard arg-max as a
  manifest-recorded variant. The convention is fixed as **`β = 1/temperature`** (the prior art's
  `temperature`); the *value* of `β` is a regime-dependent knob **fit by the §10 trials, not asserted**,
  and recorded per-profile. Full superposition over the codebook is the reference; any top-k truncation
  (as in `embeddenator-vsa`, top-8) is an explicit, recorded approximation, not the default.
- **Q3 — Convergence & oscillation detection. → Resolved: discrete index-tuple, bounded window.**
  Convergence = the decoded index tuple `ι` (§3) unchanged for one full sweep **and** every slot's
  top-similarity ≥ `τ_lock`. Oscillation = recurrence of a previously-seen `ι` within a bounded history
  window (default window = the iteration budget, i.e. remember every tuple seen this run; a smaller ring
  buffer is a recorded approximation). Detection is on the **discrete tuple**, never the real-valued
  state (§8.1 P3). `τ_lock` is a manifest threshold; its default is trial-fit alongside `β`.
- **Q4 — Deriving `δ`. → Resolved: oracle-measured, over a `{F, ∏kᵢ, d}` grid.** The trial harness
  draws random factor tuples from the codebooks, binds them, factorizes, and scores **exact recovery of
  the true tuple against the brute-force oracle** (§11) — *not* self-reported convergence (§8.1 P5). `δ`
  is the measured failure rate at the declared trial count; `1 − δ` holds only inside the validated
  grid. **First grid (recommendation, to be confirmed by trials):** `F ∈ {2, 3}`, equal `kᵢ ∈ {8, 16,
  32, 64}`, `d ∈ {1024, 4096, 8192}`, sweeping the operational-capacity ratio `∏ᵢ kᵢ / d`; the grid
  granularity itself is reported, not asserted. Encoded as a `ResonatorProfile` (§5.2), distinct from
  the bundle `EmpiricalProfile` (§8.1 P4).
- **Q5 — Multiplicity. → Resolved: report top-confidence with confidence + margin; refuse below
  margin.** Reuse the existing `CleanupMemory::cleanup` `Match { confidence, margin }`: report the
  top-confidence factor per slot with both numbers; a slot whose **margin** (top minus runner-up) is
  below the manifest's ambiguity threshold is an explicit refusal, as is a confidence below threshold
  (§5.4). No silent coin-flip between near-tied atoms (§8.1 P5). The ambiguity-margin default is
  trial-fit.
- **Q6 — Per-model scope. → Resolved: MAP-I first (then BSC); sparse/HRR/FHRR deferred and *not*
  `Empirical`.** First prototype targets **MAP-I** — its bipolar elementwise bind is *genuinely*
  exact-self-inverse (full support preserved), so the loop's unbind is exact and the ceiling is honestly
  `Empirical`. **BSC** (XOR self-inverse) follows. The prior art's **sparse-ternary** model is
  explicitly **not** the first target: its multiplicative bind is lossy / not exactly self-inverse
  (§8.1 P2), so resonator factorization over it is `Declared`, not `Empirical`. HRR/FHRR (approximate
  unbind ⇒ `Declared`) remain deferred (§10.3).

## 10. Phasing

1. **Design (this RFC).** Fix the contract above; §9 carries the proposed resolutions. *Ratification is
   the maintainer's.*
2. **Prototype** (next M-350 increment, post-Accept): the `mycelium-vsa::resonator` loop over **MAP-I**
   (§9 Q6), parallel/Jacobi update (§3, §8.1 P6), softmax cleanup with manifest `β`/`τ_lock` (§9 Q2/Q3),
   the run trace + `EXPLAIN`, and a **trial-validated `ResonatorProfile`** establishing one `{F, ∏kᵢ, d}`
   regime (§9 Q4) — tag `Empirical`, never upgraded; out-of-regime an explicit refusal.
   Differential-checked against brute-force factorisation on small instances (the oracle), like the
   other VSA ops.
3. **Maybe (§10.3).** Wider model/regime coverage, the softmax/arg-max and `β` ablation, BSC, then
   HRR/FHRR and sparse/block codes (all `Declared`, not `Empirical` — §9 Q6).

## 11. Acceptance criteria (what "done" means for the prototype, once this is Accepted)

- A `Resonator` decode runs the §3 loop, returns the §4 trace, and **never hangs** (budget) and **never
  silently returns a non-converged or wrong-fixed-point result** — only a clean `Converged` verdict that
  clears `τ_lock` + confidence + margin yields factors (§6, §8.1 P5).
- The reported guarantee is `Empirical` with an attached `{F, ∏kᵢ, d}` `ResonatorProfile`; a request
  outside it is an explicit refusal; the basis is **never** `Proven` (schema-enforced,
  `mycelium-core::recon`).
- Correctness on small instances is **differential-checked against brute force**, scoring *exact factor
  recovery* (not self-reported convergence — §8.1 P5); the empirical `δ` is established by the declared
  trial count (the `EmpiricalProfile`/`ResonatorProfile` pattern), not asserted.

## Changelog

- **2026-06-15 — Draft.** Initial design from the RFC-0003 §6 resonator note and the FR-C2/G4/RR-5
  risk position: the iterative update (Frady et al. 2020), the never-`Proven` / `Empirical`-ceiling
  honesty contract (already schema-enforced in `mycelium-core::recon`), the never-silent termination
  verdicts, reuse of `VsaModel` + `CleanupMemory` + `EmpiricalProfile`, and the open questions to
  settle before building. Prior art (`embeddenator-retrieval`/`-vsa`) flagged to mine, not copy.
  Satisfies the M-350 *needs-design* gate (document the convergence regime + bounds before building —
  RR-5/G4). No code; nothing in the kernel.
- **2026-06-15 — Draft revision (prior-art mining, M-350).** Read the reference implementations
  (`embeddenator-vsa::resonator`, `embeddenator-retrieval::core::resonator`) and folded the findings
  into the contract. Added **§8.1** (seven concrete pitfalls P1–P7: unseeded init; an unbacked
  "self-inverse" on the *lossy* sparse-ternary bind; no oscillation detection + a wrong
  cosine-to-previous convergence test; no regime/`δ`; a wrong fixed point returned as an answer with no
  correctness test; in-place Gauss-Seidel rather than parallel update; and silent zero-fill
  fabrication in the neighbouring code). **Resolved the §9 open questions** as recommendations (pending
  ratification): uniform seeded init (Q1); softmax default with `β = 1/temperature` trial-fit (Q2);
  discrete index-tuple convergence + bounded-window cycle detection (Q3); oracle-measured `δ` over a
  `{F, ∏kᵢ, d}` grid via a `ResonatorProfile` distinct from the bundle `EmpiricalProfile` (Q4);
  confidence **and margin** refusal reusing `CleanupMemory::cleanup` (Q5); MAP-I-first with
  sparse/HRR/FHRR `Declared` not `Empirical` (Q6). Tightened **§3** (parallel/Jacobi snapshot update;
  discrete-tuple decode + cycle detection), **§5** (regime as `{F, ∏kᵢ, d}`; `δ` = oracle-measured
  correctness, "converged ≠ correct"; margin-based ambiguity refusal), and **§6** (only a clean
  `Converged` verdict yields factors; seeded determinism). Status stays **Draft**; honesty contract
  unchanged (Empirical ceiling, never `Proven`, never silent); no code; nothing in the kernel.
