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
state:   x̂ᵢ(0) for i ∈ 1..F            -- initial factor estimates (see §6 Q1)
update:  for each i in parallel:
           rᵢ      = g⁻¹( s, g_{j≠i} x̂ⱼ(t) )     -- "explain away" the other factors
           aᵢ      = cleanup_i( rᵢ )              -- project onto codebook Cᵢ (see §6 Q2)
           x̂ᵢ(t+1) = aᵢ
stop:    converged   iff every x̂ᵢ is stable (each slot's top-similarity atom unchanged AND
                          its similarity ≥ τ_lock) for one full sweep; OR
         exhausted   iff t reaches the manifest iteration_budget (RFC-0003 §6: ≥ 1); OR
         oscillating iff a previously-seen state vector recurs (cycle detection)
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
   dimension `d` (the *operational capacity* `∏ᵢ kᵢ` relative to `d`). The prototype records this as an
   `EmpiricalProfile` (the existing trial-validated-profile pattern, `mycelium-vsa`); a decode requested
   **outside** the validated regime is an explicit refusal (`OutsideEmpiricalProfile`), never a
   stretched tag. The reported certificate is `{ ε, δ, strength = Empirical }` with the regime attached.
3. **Lossy-bounded, best-effort.** Even within regime the reconstruction is lossy (cleanup is a
   projection); the per-factor confidence is reported, and a confidence below the manifest threshold is
   an explicit `CleanupConfidenceBelowThreshold`-style refusal (the cleanup contract already exists,
   `mycelium-vsa`), never a silent low-confidence guess.

## 6. Never-silent & termination (G2)

- **Bounded.** The iteration budget (`≥ 1`, schema-checked) caps work — a non-converging run **cannot
  hang**; it stops with `BudgetExhausted`.
- **Explicit non-convergence.** `BudgetExhausted` and `Oscillating` are **error verdicts**, not a
  returned "answer". A resonator decode returns `Result<Factors, ResonatorError>` (or an `Option`),
  never a wrapped/garbage factor set. This mirrors the swap "never silent" rule (S1/G2): an out-of-regime
  or non-converged factorization is surfaced, not papered over.
- **Determinism.** Given the manifest parameters and a seeded initialisation, a run is reproducible
  (the trace is a function of the inputs), so the differential/property tests are stable.

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

## 9. Open design questions (settle before/while building)

- **Q1 — Initialisation.** Superposition of *all* codebook atoms per slot (the standard "uniform"
  start), random, or a single seeded guess. Affects the convergence basin and the `δ` regime.
- **Q2 — Cleanup shape.** Softmax-weighted superposition (with `β`) vs hard arg-max; the former is the
  literature default (smoother dynamics) but needs `β` tuning, the latter is simpler but more prone to
  early lock-in. Both are recorded in the manifest.
- **Q3 — Convergence & oscillation detection.** The `τ_lock` threshold and the cycle-detection window
  (resonators are known to limit-cycle rather than converge in some regimes — that must be *detected*,
  per §6, not run to budget silently).
- **Q4 — Deriving `δ`.** How the trial harness estimates the per-regime success probability and the
  granularity of the `{F, kᵢ, d}` grid the `EmpiricalProfile` covers.
- **Q5 — Multiplicity.** When several factorisations are (near-)consistent with `s`, report the
  top-confidence one with its confidence, or refuse as ambiguous. (Default: report + confidence; an
  ambiguity below a margin is an explicit refusal.)
- **Q6 — Per-model scope.** Which `VsaModel`s the first prototype targets — MAP-I/BSC (exact bind ⇒
  `Empirical` ceiling) first; HRR/FHRR (approximate ⇒ `Declared`) deferred.

## 10. Phasing

1. **Design (this RFC).** Fix the contract above. *Ratification is the maintainer's.*
2. **Prototype** (next M-350 increment, post-Accept): the `mycelium-vsa::resonator` loop over MAP-I,
   the run trace + `EXPLAIN`, and a **trial-validated `EmpiricalProfile`** establishing one `{F,k,d}`
   regime — tag `Empirical`, never upgraded; out-of-regime an explicit refusal. Differential-checked
   against brute-force factorisation on small instances (the oracle), like the other VSA ops.
3. **Maybe.** Wider model/regime coverage, the softmax/argmax ablation, HRR/FHRR (`Declared`).

## 11. Acceptance criteria (what "done" means for the prototype, once this is Accepted)

- A `Resonator` decode runs the §3 loop, returns the §4 trace, and **never hangs** (budget) and **never
  silently returns a non-converged result** (§6).
- The reported guarantee is `Empirical` with an attached `{F, kᵢ, d}` regime; a request outside it is an
  explicit refusal; the basis is **never** `Proven` (schema-enforced).
- Correctness on small instances is **differential-checked against brute force**; the empirical `δ` is
  established by the declared trial count (the `EmpiricalProfile` pattern), not asserted.

## Changelog

- **2026-06-15 — Draft.** Initial design from the RFC-0003 §6 resonator note and the FR-C2/G4/RR-5
  risk position: the iterative update (Frady et al. 2020), the never-`Proven` / `Empirical`-ceiling
  honesty contract (already schema-enforced in `mycelium-core::recon`), the never-silent termination
  verdicts, reuse of `VsaModel` + `CleanupMemory` + `EmpiricalProfile`, and the open questions to
  settle before building. Prior art (`embeddenator-retrieval`/`-vsa`) flagged to mine, not copy.
  Satisfies the M-350 *needs-design* gate (document the convergence regime + bounds before building —
  RR-5/G4). No code; nothing in the kernel.
