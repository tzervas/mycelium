# RFC-0010 — Decode-Methodology Selection

| Field | Value |
|---|---|
| **RFC** | 0010 |
| **Status** | **Accepted** |
| **Type** | Foundational / normative |
| **Date** | June 15, 2026 |
| **Depends on** | RFC-0005 (the `SelectionPolicy` decision-table mechanism + mandatory `EXPLAIN`); RFC-0009 (the resonator decode + the `ResonatorProfile` regime gate + the §10.3 measured matrix); RFC-0003 (the VSA guarantee matrix + `DecodeProcedure`); RFC-0001 (`Meta`, guarantee lattice, content-addressing); **G2**, **G4**, **VR-5**; cross-cutting tension **D** |
| **Coupled with** | RFC-0002 (swap-target selection) and RFC-0004 (packing-schedule selection) — the **same** RFC-0005 mechanism, a third site |

## 1. Summary

A value can be decoded back to its constituents (factors of a bind product; roles/fillers of a
record) by **more than one methodology**, and those methodologies carry **different honest guarantee
tags**: a brute-force enumeration of `∏ᵢ kᵢ` codebook combinations recovers the factors **`Exact`ly**
(it *is* the differential oracle of RFC-0009 §5.3), while the iterative resonator (RFC-0009 §3)
recovers them only **`Empirical`ly** — and only inside a validated `{F, ∏kᵢ, d}` regime. Choosing
between them is therefore a real, tag-changing decision that today is made by hand.

This RFC designs that choice as a **third site of the one RFC-0005 selection mechanism** (alongside
swap targets and packing schedules — "one mechanism, two sites" becomes three; DRY/SoC). A
**`DecodeMethodPolicy`** is an ordered decision table over **exact metadata** (`F`, the operational
capacity `∏ᵢ kᵢ`, `d`, the model, and `ResonatorProfile` membership) returning a choice among a
**finite** candidate set `{ BruteForceExact, Resonator(Hebbian), Refuse }`, with a cost model, a
mandatory default arm, and a mandatory `EXPLAIN` trace. The **guarantee tag flows from the chosen
arm** — the selector never upgrades a tag (VR-5), and an out-of-regime input is a **`Refuse`**, never
a silent best-effort guess (G2). No code lands with this RFC; it is the *needs-design* artifact the
decision matrix plugs into (per RR-5/G4, design the regime + the honest contract before building).

## 2. Motivation

Two findings from the RFC-0009 §10.3 cleanup ablation set the scope:

1. **The cleanup-variant axis collapses to a single winner inside the validated envelope, so it does
   *not* need a selector yet (YAGNI).** Measured exact-tuple recovery (failure rate, lower is better)
   over the wall corners (F=3) shows the **Hebbian bipolar** cleanup `sign(Σⱼ simⱼ·cⱼ)` is
   *Pareto-dominant* — best-or-tied at every corner:

   | corner (F=3) | Softmax β=6 | SoftmaxSign β=6 | ArgMax | **Hebbian** |
   |---|---|---|---|---|
   | k=16, d=4096 (∏k=4096) | 1.000 | 1.000 | 0.053 | **0.000** |
   | k=16, d≥8192 | 1.000 | 1.000 | 0.000 | **0.000** |
   | k=32, d=8192 (∏k=32768) | 1.000 | 1.000 | 0.575 | **0.085** |
   | k=32, d=16384 | 1.000 | 1.000 | 0.105 | **0.005** |

   With one dominant option there is nothing to *select*; `Cleanup::Hebbian` is simply the validated
   default knob (RFC-0009 §10.3). A cleanup-variant selector would be premature machinery for a
   one-row table — it re-opens only when a *second* winner appears (another model — BSC next; or
   `k≥32`, where Hebbian and ArgMax separate but neither yet holds δ tight). This RFC therefore
   **defers** cleanup-variant selection (§8) and selects over **methodology**, where the choice is
   real *now*.

2. **The methodology axis is a real, tag-changing choice.** Brute force is tractable and **`Exact`**
   precisely when `∏ᵢ kᵢ` is small enough to enumerate; past that crossover it is intractable and the
   resonator is the only option — but the resonator is **`Empirical`** and only inside its
   `ResonatorProfile`. So the same `reconstruct_factors` request maps to *different guarantee tags*
   depending on exact, inspectable metadata. Making that mapping a reified, explainable policy (rather
   than caller folklore) is exactly the G2/ADR-006 "no black box" posture the swap- and
   packing-selection sites already enjoy — and it keeps the honesty rule mechanical: the tag is read
   off the chosen arm, never asserted by the caller.

The opacity trap RFC-0005 §2 warns about (statistics-driven cardinality estimates) **does not arise
here**: every predicate input is *exact* kernel metadata — `F` and the `kᵢ` are codebook sizes, `d` is
the dimension, regime membership is a total `ResonatorProfile::check`. There are no sampled estimates.

## 3. Guide-level explanation

Think of it as the resonator's front door. A caller asks "recover the factors of this value against
these codebooks." The policy looks **only at exact metadata** and routes:

```text
inputs:  model, F = #codebooks, k = (k₁…k_F), ∏k = Πᵢ kᵢ, d = dim
table (first match wins, RFC-0005 §2):
  1. ∏k ≤ enum_budget  AND identifiable           → BruteForceExact     (tag: Exact)
  2. ResonatorProfile.check(F, k, d) == Ok        → Resonator{Hebbian}  (tag: Empirical, δ attached)
  3. otherwise                                    → Refuse              (OutsideEmpiricalProfile)
```

- **Arm 1 — `BruteForceExact`.** When `∏k` is within an enumeration budget the brute-force oracle
  (RFC-0009 §5.3) *is* the decode: it returns the global arg-max tuple with an **`Exact`** tag (it
  checks every combination — no probabilistic gap). It also answers *identifiability* for free (is the
  true tuple the unique arg-max?), which the resonator cannot.
- **Arm 2 — `Resonator{Hebbian}`.** When enumeration is intractable but the request sits inside the
  validated `{F, ∏k, d}` regime, run the RFC-0009 §3 loop with the validated Hebbian default. Tag is
  **`Empirical`** with the `MAPI_RESONATOR_PROFILE` δ attached; the §5.4 confidence/margin gate and
  the never-silent verdicts still apply on top (a converged-but-ambiguous run still refuses).
- **Arm 3 — `Refuse`.** Too big to enumerate *and* outside the resonator's validated regime → an
  explicit `OutsideEmpiricalProfile`, never a silent best-effort guess. This is the honest answer the
  matrix must be able to give (G2).

Every selection emits the RFC-0005 `Explanation` — the inputs considered, each candidate's cost, the
matched rule, the chosen arm, and the resulting guarantee tag — so "why was this decoded by brute
force / by the resonator / refused?" is always answerable (SC-5). The policy is content-addressed; its
`PolicyRef` is recorded on the decode's `Meta` exactly as swap/packing decisions record theirs.

## 4. Reference-level design (normative)

### 4.1 The candidate set (finite, RFC-0005 §2.1)

`DecodeMethod ∈ { BruteForceExact, Resonator{cleanup}, Refuse }`. `cleanup` defaults to the validated
`Cleanup::Hebbian` (RFC-0009 §10.3) and is a **recorded knob, not a selected axis** in this RFC (§8).
The set is closed; adding a model (BSC) or a methodology extends it by an append-only revision with its
own measured evidence (VR-5) — never by a learned/open-ended search.

### 4.2 Predicates over exact metadata (RFC-0005 §2.5 — no sampled statistics)

The table's predicates read only: `model_id`; `F = |codebooks|`; the per-slot sizes `kᵢ`; the
operational capacity `∏ᵢ kᵢ` (saturating, as `ResonatorProfile::check` already computes); `d`; and the
total `ResonatorProfile::check(F, k, d)` verdict. All are exact and total; evaluation is structural and
terminating (RFC-0005's expressiveness ceiling is preserved — the policy is not Turing-complete).

### 4.3 Cost model (RFC-0005 §2.1 explicit cost)

The cost function ranks tractable candidates by a transparent, metadata-only estimate:
`cost(BruteForceExact) = ∏ᵢ kᵢ` (combinations enumerated, each an `F`-fold bind + similarity);
`cost(Resonator) = iteration_budget · F · (Σᵢ kᵢ)` (sweeps × per-slot cleanup work). `enum_budget` (the
arm-1 threshold on `∏k`) is a declared policy parameter, **fit by measurement** (the brute-force
wall-clock crossover at a given `d`), not asserted — recorded in the policy like `β`/`τ_lock` are
recorded in the resonator profile. Ties break by the RFC-0005 fixed rule (lowest candidate index ⇒
prefer the `Exact` arm).

### 4.4 Guarantee-tag flow (VR-5 — the honesty crux)

The decode's guarantee tag is **read off the chosen arm**, never chosen independently:
`BruteForceExact → Exact`; `Resonator → Empirical` with the `MAPI_RESONATOR_PROFILE.bound()`
(`EmpiricalFit`) attached; `Refuse → no value` (an `OutsideEmpiricalProfile` error). The selector
**cannot upgrade** a tag and **cannot** turn a `Refuse` into a guess (the `mycelium-core::recon` schema
still caps the resonator basis at `EmpiricalFit`; this RFC does not touch that ceiling). A
`BruteForceExact` arm that finds the instance **non-identifiable** (true tuple not the unique arg-max)
is itself a `Refuse`, not a coin-flip — identifiability is a precondition of the `Exact` claim.

### 4.5 Reification, determinism, override, EXPLAIN (RFC-0005 §§2–4, reused verbatim)

- **One mechanism.** `DecodeMethodPolicy` is a `SelectionPolicy` instance; selection goes through the
  single RFC-0005 `select`/`explain`. No parallel selector is introduced (DRY/SoC).
- **Determinism.** Same exact inputs ⇒ same arm; the resonator arm remains seed-deterministic
  (RFC-0009 §6). The whole decode is reproducible and content-addressable.
- **Override.** A forced arm is first-class (RFC-0005 §2.4): a caller may pin `Resonator` even when
  `∏k ≤ enum_budget` (e.g. to measure the resonator), recorded in the trace — but a forced arm
  **cannot** escape the honesty floor (forcing `Resonator` out of regime still `Refuse`s; forcing
  `BruteForceExact` on a non-identifiable instance still `Refuse`s).
- **EXPLAIN (mandatory).** Every selection emits `{inputs, per-candidate cost, matched rule, chosen
  arm, resulting guarantee tag, override state}`; the LSP surfaces it (SC-5). No decode-method
  selection happens without one (G2).

### 4.6 Non-goals (this RFC)

No cleanup-variant selection (§8); no new kernel surface (the policy is `mycelium-select` +
`mycelium-vsa` glue, like the existing decode path — KC-3); no change to the resonator loop, the
profile δ, or the recon schema; no learned/statistics-driven costing (RFC-0005 §2.5).

## 5. Drawbacks

- **A table with three arms may look like over-engineering** for what a caller could `if`-branch. The
  payoff is the reified `PolicyRef` + `EXPLAIN` (auditability, content-addressing) and a *single* place
  the exact-vs-empirical/refuse honesty is enforced rather than re-implemented per call site.
- **`enum_budget` is a real number that must be measured**, and it is hardware/`d`-dependent; setting it
  by guess would re-introduce the cost-opacity RFC-0005 avoids. It is therefore a recorded, measured
  policy parameter, with the honest default being conservative (prefer `Refuse`/resonator over an
  enumeration that might be too slow).
- **Scope discipline.** The candidate set must stay closed and append-only; the temptation to add a
  "best-effort anyway" arm is exactly the §8.1 P7 fabrication failure mode RFC-0009 exists to prevent.

## 6. Rationale & alternatives

- **Why a decision table, not a learned/auto-tuned selector?** Same reason as RFC-0005 §2: an
  unanalyzable policy *is* the black box ADR-006 forbids; the inputs here are exact, so a total table is
  both sufficient and honest.
- **Why not just always run the resonator?** It throws away a free **`Exact`** answer (and free
  identifiability) whenever `∏k` is enumerable, and it silently degrades the tag from `Exact` to
  `Empirical`. The whole point of Mycelium is to *keep* the stronger honest guarantee when it's
  available (the lattice `Exact ⊐ Empirical`).
- **Why not select the cleanup variant too?** No second winner exists in the measured envelope (§2.1);
  a selector over a one-row table is YAGNI. Deferred to §8 with a concrete re-open trigger.
- **Alternative — fold this into RFC-0009.** Rejected: RFC-0009 is the *resonator*; methodology choice
  spans resonator **and** brute force **and** refusal, and it reuses the RFC-0005 mechanism — it is a
  selection-site design, so it belongs with the other selection sites (RFC-0005's "sites").

## 7. Prior art

- **RFC-0005** (database cost-based optimizers): the decision-table + mandatory-`EXPLAIN` shape and the
  exact-metadata-not-cardinality-estimates discipline, adopted wholesale.
- **RFC-0009 §5.3 / §8.1 P5** (the brute-force differential oracle; "converged ≠ correct"): the
  `Exact` arm *is* the oracle, and the never-silent verdicts are the floor the selector cannot breach.
- **RFC-0009 §10.3** (the cleanup ablation): the measured matrix that grounds both the deferral of
  cleanup-selection and the existence of the methodology choice.
- **Frady et al. 2020**: the resonator whose tractable-capacity ceiling is *why* a brute-force fallback
  matters for small `∏k`.

## 8. Unresolved questions

- **`enum_budget` crossover — *measured* (2026-06-15).** The wall-clock sweep
  (`tests/decode_select.rs::decode_method_enum_budget_crossover`) puts the **cost-parity crossover at
  `∏k ≈ 100–128`**, d-independent (both methods scale with `d`): brute force is cheaper only for
  `∏k ≲ 64`; at the validated edge `∏k=4096` it costs ≈ **19×** the resonator (≈76 ms vs ≈4 ms at
  d=4096) for the `Exact`-over-`Empirical` upgrade. This turns the open question into a **policy** one:
  `DEFAULT_ENUM_BUDGET` is currently *guarantee-maximal* (= `max_capacity` = 4096 ⇒ always `Exact`
  in-regime, bounded ≤ ≈157 ms at d=8192); a *cost-optimal* default would be ≈128. The knob is exposed
  per call and the choice is the maintainer's; the EXPLAIN cost lines surface the trade.
- **Identifiability precheck cost.** Arm 1 gets identifiability for free; should arm 2 (resonator) also
  run a cheap identifiability precheck so a `Refuse` distinguishes "ambiguous instance" from "resonator
  miss"? (Leans yes for `EXPLAIN` quality; costs `∏k` — only affordable in the arm-1 regime.)
- **When does the cleanup axis re-open?** Concretely: a second model (BSC — `Cleanup::Hebbian` is
  MAP-I-validated; BSC's XOR self-inverse may favour a different projection) or a measured `k≥32`
  corner where two cleanups both hold δ tight. Until then, cleanup stays a recorded default, not a
  selected axis (§4.1).

## 9. Future possibilities

- **Per-model cleanup selection** once a second winner is measured (BSC, then sparse/HRR/FHRR — the
  latter `Declared`, not `Empirical`, per RFC-0009 §9 Q6), folded in as additional table rows with
  their own evidence.
- **Dimension as a selectable knob** (choose the smallest `d` that holds the target δ for a given
  `{F, ∏k}`), reusing the same cost/`EXPLAIN` machinery.
- **A unified `reconstruct(...)` front door** that routes role-decode (RFC-0003 cleanup) and
  factor-decode (RFC-0009 resonator) through the one policy, so every reconstruction records *which*
  methodology and *what tag* it earned.

## Meta — changelog

- **2026-06-15 — Draft.** Initial design, prompted by the RFC-0009 §10.3 ablation outcome. Frames
  decode-methodology choice as a **third site of the one RFC-0005 selection mechanism** (no parallel
  selector — DRY/SoC): a content-addressed, `EXPLAIN`-mandatory decision table over **exact** metadata
  (`F`, `∏kᵢ`, `d`, model, `ResonatorProfile` membership) choosing among
  `{ BruteForceExact (Exact), Resonator{Hebbian} (Empirical), Refuse }`, with the **guarantee tag read
  off the chosen arm** (VR-5) and out-of-regime/non-identifiable inputs an explicit refusal (never
  silent — G2). Records the §10.3 finding that the **cleanup-variant axis collapses to one winner
  (Hebbian) inside the validated envelope**, so cleanup-selection is **deferred** (YAGNI) with a
  concrete re-open trigger (a second model/regime). **No code; nothing in the kernel** (the policy is
  `mycelium-select` + `mycelium-vsa` glue; the recon schema's `≤Empirical` ceiling is untouched).
  Satisfies the *needs-design* gate: the regime + honest contract are fixed before any selector is
  built (RR-5/G4). Status **Draft** — recommendations pending maintainer ratification.
- **2026-06-15 — Accepted (ratified).** Maintainer ratifies the design above (Draft → Accepted,
  append-only). Authorises the prototype: extend the **one** RFC-0005 selection mechanism additively
  to a **third site** — an abstract `DecodeMethod` candidate `{ BruteForceExact, Resonator, Refuse }`,
  decode queryable facts (`F`, `∏kᵢ`, `d`, `in_regime`) on `SelectionInputs`, the predicates
  (`CapacityAtMost`, `FactorsAtMost`, `InResonatorRegime`), and a `select_decode_method` site adapter
  (`mycelium-select`, core-only — the facts are generic integers/booleans, no VSA types leak in) — plus
  the `mycelium-vsa` executor that computes the facts (`∏kᵢ`; `MAPI_RESONATOR_PROFILE::check` for
  `in_regime`), routes through `select`, and runs the chosen arm with the **tag read off the arm**
  (brute-force `Exact` + identifiability, resonator `Empirical`, else `Refuse`). The honesty floor is
  enforced in the executor: a forced `BruteForceExact` beyond the enumeration budget, or on a
  non-identifiable instance, still `Refuse`s; a forced `Resonator` out of regime still `Refuse`s. No
  kernel change (the recon `≤Empirical` ceiling is untouched); cleanup-variant selection stays deferred
  (§8). Honesty contract unchanged.
- **2026-06-15 — Prototype landed (M-350; informative).** Built `mycelium-vsa::decode_select`
  (`reconstruct_factors_auto` / `explain_decode_method` / `decode_method_policy` /
  `DecodeSelection`) over the additively-extended one mechanism (`mycelium-select`: `DecodeMethod`,
  `DecodeFacts`, the `CapacityAtMost`/`FactorsAtMost`/`InResonatorRegime` predicates, the
  `select_decode_method` adapter — all core-only). The brute-force arm is identifiability-checked
  (`VsaError::NonIdentifiable`) and tagged `Exact`; the resonator arm is tagged `Empirical`; the honesty
  floor (§4.5) is enforced in the executor and covered by tests (forced overrides cannot escape it).
  **Measured design consequence (honest):** with the default `enum_budget` = the resonator's validated
  `max_capacity` (4096), every in-regime request is also enumerable, so the **brute-force `Exact` arm
  dominates the entire current validated regime** — the resonator arm is reachable only at a tighter
  (latency-driven) budget or once the validated capacity grows past the enumeration budget. This
  sharpens §8's `enum_budget` question (a wall-clock crossover, still to be measured) and underlines why
  pushing the resonator's validated capacity well beyond what is cheaply enumerable is what makes the
  `Empirical` arm load-bearing. No kernel change; cleanup-variant selection still deferred (§8). RFC
  stays **Accepted**; honesty contract unchanged.
- **2026-06-15 — `enum_budget` crossover measured + Value-level wiring (M-350; informative).** Two
  follow-ups. **(A) §8 crossover, now measured.** The wall-clock instrument
  (`tests/decode_select.rs::decode_method_enum_budget_crossover`) times brute force vs the resonator
  per decode across `{F, k, d}`: the **cost-parity crossover is `∏k ≈ 100–128`** (d-independent), brute
  force cheaper only for `∏k ≲ 64`, and **≈19× more expensive at the regime edge `∏k=4096`** (≈76 ms vs
  ≈4 ms, d=4096) — so the default `enum_budget = max_capacity` is *guarantee-maximal* (always `Exact`
  in-regime), not *latency-minimal* (≈128). The number is recorded; the default value stays the
  maintainer's policy call (the §8 question is now a guarantee-vs-latency trade, not an unknown).
  **(C) Value-level wiring.** `mycelium-vsa::reconstruct_factors_selected` routes a `Resonator` manifest
  through the selector instead of always running the resonator — a small `∏k` is upgraded to brute-force
  `Exact`, an in-regime request runs the `Empirical` resonator, else an explicit refusal. It does **not**
  pre-gate on the resonator profile, so a brute-forceable instance *outside* the resonator regime (e.g.
  `F=4, k=8`, ∏=4096 — which the plain `reconstruct_factors` refuses) is recovered **exactly** (RFC-0010
  §4.4: brute force is `Exact` for any factor count). Tag still read off the arm; recon `≤Empirical`
  ceiling untouched. RFC stays **Accepted**; honesty contract unchanged.
