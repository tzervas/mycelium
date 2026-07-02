# Kickoff `rwr` — Phase II: the progressive Mycelium rewrite (post-public; `0.x → 1.0.0`)

> **UID:** `rwr` · **Basis:** **ADR-038** (Proposed) §2.3/§2.5/§2.8 + the umbrella roadmap
> `docs/planning/road-to-1.0.0-and-mycelium-rewrite.md` **§7** · RFC-0031 §5 **D4–D7** (order ·
> stability bar · oracle retention · one spore per phylum) · **RFC-0033 §7** + its V-wave
> (**M-760…M-784** — RFC-0033-named slots, unminted today) · ADR-030/031 (the deferred one-way
> doors) · ADR-036 §2.3 (differential + replace-on-satisfaction, unchanged) · DN-66/ADR-035/M-867
> (retirement scope) · `docs/planning/self-hosting-port-ledger.md` (+ the `opp`-measured data).
> **Planned by:** Fable (ADR-038 §2.7); **implemented by:** Opus/Sonnet/Haiku per the PM table.
> **References the doc-maintenance contract** (`_doc-maintenance.md`) in its DoD.
> **Posture — higher-altitude than the Phase-I kickoffs, and honestly so:** Phase II is
> *progressive, per-crate/module, decision-gated* (ADR-038 §2.3). The rows below are **wave-level**
> tasks; per-crate port issues are minted **per wave**, from measured data, not enumerated here.
> **Sequencing: post-public** — after `flp`'s M-945; the porting lanes are additionally **gated on
> `grm`'s grammar-stable baseline** and on transpiler maturity per rung.

## Goal

Progressively rewrite the remaining corpus in Mycelium, **in the open**: each module built beside
its Rust reference, **Rust≡Mycelium differential-validated**, replacing its original **only on the
maintainer's satisfaction** (ADR-036 §2.3) — while the **public semver climbs `0.x → 1.0.0`**
(ADR-038 §2.8), where **`1.0.0` = fully rewritten into Mycelium where appropriate + 100%
operational** (subject to FLAG-V2). The version *is* the honest progress measure; nothing here has
a calendar deadline.

## Scope

**In:** the four lanes — mass stdlib `.myc` porting (RFC-0031 D5 per-op bar + differential + D6
`#[deprecated]` marking, oracle retained), toolchain porting (Rust toolchain as oracle), the
**V-wave value-model remainder** (M-760…M-784 incl. the single M-780 rehash at its tripwire), and
the semver/terminal bookkeeping through the `1.0.0` act. **Out:** compiler self-hosting as a task
lane (**it is an aspiration** — see below); any rehash before the value-persistence tripwire
(RFC-0033 §7); un-gated mass porting (waits on `grm`); minting per-crate port issues from this doc.

## Compiler self-hosting — an aspiration, NOT a task lane (say so honestly)

Per ADR-038 §2.3 it is **doubly conditioned**: only if it demonstrably improves stability and/or
performance over the Rust frontend (evidence the maintainer accepts — a benchmark/defect case, not
an aesthetic), **and** only after the transpiler is **100% polished** (§2.5's ladder complete).
Absent that evidence the Rust frontend stays — that *is* the North Star applied to the compiler.
**No M-id is minted for it here.** If the double condition ever triggers, that event gets its own
kickoff + bootstrap-protocol ratification (RFC-0031 D3's no-circularity staging holds until then).

## ⚠ Maintainer decisions this kickoff carries (FLAG, never guess)

| Decision | Where it bites | Ref |
|---|---|---|
| Per-module **replace-on-satisfaction** act (every port; never self-declared) | M-949/M-952 rows, each wave | ADR-036 §2.3 |
| Toolchain-port module sign-offs (which components port, in what order) | M-951 → M-952 | ADR-036 §2.2/§2.3 |
| The **value-persistence tripwire** — the single M-780 rehash fires immediately BEFORE the first value-persistence feature, never speculatively | M-954 | RFC-0033 §7 · ADR-030/031 |
| **FLAG-V2** — whether `1.0.0` requires compiler self-hosting (proposed reading: no — compiler counts as appropriate-Rust unless §2.3's condition triggers) | M-957 | ADR-038 §2.8 |
| The semver scheme (set at the flip) + **every version bump and the `1.0.0` cut are maintainer release acts** | M-956/M-957 | ADR-038 §2.8 |

## Swarm method + model tiering (ADR-038 §2.7)

**Per-wave swarms, not one mega-swarm.** Each port wave is its own `/wave` run: disjoint crate
ownership (the `opp` pattern — one leaf per crate, D5 rows, orchestrator owns the ledger/indices),
sized from the manifest (M-947), with per-crate issues minted at that wave's kickoff. The V-wave
lane is kernel work — small, serial, Opus-led. Toolchain ports are one module at a time. One
isolated worktree per leaf (mitigation #11); commit/push split (#12); scoped PRs via `/pr-land`.
Public-repo reality post-flip: waves land per-repo (the `flp` topology), with cross-repo
coordination riding issues, never shared files.

## PM decomposition — wave-level tasks

Proposed M-ids **M-947…M-957** — next-free after `flp`'s block (highest minted today is M-876;
`acy`…`opp` propose M-877…M-935, `flp` proposes M-936…M-946); **re-verify each slot at minting**
(mitigation #1). The V-wave rows reference the **RFC-0033-named M-760…M-784 slots** — mint under
those names where free to keep the corpus cross-refs true (the `enb` M-766/M-767 precedent), FLAG
if taken. None minted by this doc.

### Lane 1 — mass stdlib `.myc` porting (gated: `grm` baseline + transpiler rung)

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-947 | **Port-wave manifest v1** — the ordered queue of all remaining un-ported crates, built from `opp`'s measured ledger (assist %, defect rates) + readiness §0; ROI-scored per ADR-038 §2.5 (**manifest-driven transcoding only where ROI-positive** after correction overhead); fixes the per-wave issue-minting protocol | As the Phase-II effort, I want the port order grounded in measured Phase-I data, so that waves are scheduled by evidence, not enthusiasm | Manifest committed (`Empirical` inputs cited per row); ROI basis explicit; wave-minting protocol stated; refreshed after every wave | Sonnet | `grm` complete (grammar-stable baseline + window) · `opp` M-935 (the ledger) |
| M-948 | **Transpiler hardening ladder** — small→large per the E18-1 demand data (unsupported types #1 at 36%, macros #2, bounded generics #3); **get it right at each size before moving up** (ADR-038 §2.5); coverage re-measured per rung | As a port leaf, I want the transpiler trustworthy at my crate's size class, so that assist output needs review, not archaeology | Per-rung coverage measured + recorded (`Empirical`, DN-34 §8 baseline); no rung claimed above its measured basis (VR-5); demand backlog re-ranked per rung | Opus | — (starts at Phase-II open; feeds every wave) |
| M-949 | **Port waves 1…N execution** (rolling) — per crate: pre-port polish → transpile-assist → hand-finish → **D5 per-op bar + Rust≡Mycelium differential** → **D6 `#[deprecated]` marking** (Rust oracle retained, never deleted); per-crate issues minted at each wave's kickoff, NOT here | As a language user, I want each replacement provably equivalent before the Rust original steps back, so that the rewrite never trades correctness for purity | Per wave: all D5 rows green per crate; differential vs the oracle green; D6 marking applied only on the maintainer's replace-on-satisfaction act; ledger updated | Sonnet (leads) / Haiku (mechanical) | M-947 (+ M-948's rung for the size class) |
| M-950 | **D6 retirement ledger** (rolling) — per cleared port: deprecation recorded, oracle-retention verified, M-867 fed; the ledger stays the honest rewrite-progress record the semver reads from | As the maintainer, I want one place that says exactly what is rewritten and what still rests on Rust, so that the version number never overstates progress (G2) | Ledger row per cleared port; zero unrecorded replacements; M-867 scope current | Haiku | M-949 (per wave) |

### Lane 2 — toolchain porting (Rust toolchain as oracle)

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-951 | **Toolchain-port scoping dossier** — which components (fmt/lint/doc/check/…) are ROI-positive Mycelium ports vs appropriate-Rust keepers; ordered plan; the Rust toolchain retained as the differential oracle throughout | As the maintainer, I want the toolchain's port/keep split argued with evidence per component, so that "Mycelium everywhere else" never becomes dogma (§2.1) | Dossier with per-component evidence + recommendation; maintainer sign-off requested per module; keepers recorded as decisions, not leftovers | Sonnet | M-947 (pattern data from Lane 1) |
| M-952 | **First toolchain port** (pattern-setter — the smallest ROI-positive component from M-951) with the Rust-as-oracle differential; calibrates toolchain-port cost the way `opp`'s `diag` calibrated stdlib cost | As a contributor, I want the toolchain-port pattern proven on the cheapest real component, so that later toolchain waves inherit a measured method | D5-equivalent bar + differential green; measured cost recorded (`Empirical`); replace only on the maintainer's act | Sonnet | M-951 + **maintainer sign-off** |

### Lane 3 — the V-wave value-model remainder (RFC-0033 V1–V5; kernel, serial)

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-953 | **V-wave resumption audit** — reconcile the RFC-0033-named M-760…M-784 slots against what H1 pulled forward (`enb`'s M-766/M-767 + the float-`Repr` work incl. the route-ii float ADR); publish the residual queue (incl. M-758/M-759 perf sequencing); mint under the RFC names where free | As the value-model effort, I want the deferred wave resumed from a verified inventory, so that nothing pulled forward is re-landed and nothing deferred is dropped | Audit table (landed via H1 / residual / N-A) recorded (`Empirical`); residual queue published; slots verified or FLAGged | Sonnet | `enb` complete (the pull-forwards landed) |
| M-954 | **The ADR-030/031 one-way doors + the SINGLE M-780 rehash — TRIPWIRE-armed, not scheduled:** executes immediately **before the first value-persistence feature**, never speculatively; the Dense quant-descriptor + VSA element-space doors and any float-`Repr` identity impact land as **one** identity-set change (RFC-0033 §7) | As a certified-mode user, I want content-address identity to change exactly once, at the moment persistence makes it load-bearing, so that no persisted value is ever silently orphaned | Zero rehashes before the tripwire (checked at every wave close); at the tripwire: doors + rehash land together, migration verified, the one-rehash claim recorded (`Empirical`); maintainer confirms the tripwire | Opus | M-953 + **the tripwire condition** (maintainer-confirmed) |
| M-955 | **V-wave remainder execution** — swap/guarantee reconciliation, M-758/M-759 perf, the rest of V1–V5 per the audited queue (door-dependent rows wait on M-954) | As a language user, I want the ratified value-model surface fully implemented, so that RFC-0033's design stops being partly on paper | Queue rows landed per RFC-0033 §3–§6 with honest tags + property tests per bound; door-dependent rows sequenced after M-954; statuses current | Sonnet | M-953 (M-954 for door-dependent rows) |

### Lane 4 — semver + the terminal

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-956 | **Public semver climb bookkeeping** (rolling) — apply the flip-time scheme: each maintainer-satisfied replacement advances the `0.x` per the scheme; the version stays exactly as honest as the M-950 ledger | As a public user, I want the version to state real rewrite progress, so that `0.x` reads as the honest measure ADR-038 §2.8 promises | Every bump maps to ledgered replacements; every bump is a maintainer release act (recorded); no bump above the ledger's basis (VR-5) | Haiku (+ maintainer act per bump) | `flp` M-945 (the scheme exists); rolling with M-950 |
| M-957 | **The `1.0.0` readiness dossier + terminal act** — evidence that everything-appropriate is rewritten + 100% operational; FLAG-V2 disposed (proposed reading: compiler = appropriate Rust unless §2.3 triggered); the maintainer cuts `1.0.0` | As the maintainer, I want `1.0.0` cut on a checked dossier, so that the terminal claim ("fully rewritten + 100% operational") carries its evidence | Dossier complete (per-module disposition: rewritten / appropriate-Rust-kept, each grounded); FLAG-V2 resolved by the maintainer; the act is the maintainer's, never the agent's | Opus (dossier) + **maintainer act** | all lanes + FLAG-V2 |

## Definition of Done (kickoff)

- Honest altitude: this kickoff closes when **`1.0.0` is cut** (M-957) — but its *checkable* DoD is
  per-wave: the manifest current after every wave; every port D5-green + differential-green with
  D6 marking only on the maintainer's act; the ledger + semver record never ahead of the evidence.
- **Single-rehash discipline intact end-to-end:** zero identity rehashes before the tripwire,
  exactly one at it (verified at every wave close — RFC-0033 §7).
- Compiler self-hosting either untriggered (Rust frontend standing, recorded as appropriate) or
  spun out into its own ratified kickoff — never started implicitly.
- Doc-maintenance per `_doc-maintenance.md` at every wave; per-crate issues minted per wave with
  slots re-verified (mitigation #1); every FLAG raised, none guessed (G2/VR-5).

## Prerequisites

1. **`flp` M-945 executed** (the project is public; the semver scheme exists). Planning inputs
   (`opp`'s ledger, `grm`'s baseline, the E18-1 demand data) accrue during Phase I.
2. **`grm` complete** for Lane 1 (no mass porting against a moving grammar — roadmap §5's rule);
   **`enb` complete** for Lane 3's audit.
3. **ADR-038 Accepted** (inherited); the maintainer-decision table above governs every gate inside.
