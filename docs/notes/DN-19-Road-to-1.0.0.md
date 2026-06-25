# Design Note DN-19 — Road to 1.0.0: Critical-Path Gap Analysis

| Field | Value |
|---|---|
| **Note** | DN-19 |
| **Status** | **Draft** (planning capture, 2026-06-21) |
| **Decides** | *Nothing normatively.* Advisory planning capture (DN-08/DN-11 posture): identifies the remaining gaps between the current state and a 1.0.0 kernel/core release, orders them by dependency and priority, and recommends next actions. Every gap cites its gate row or source document. The actual "ship 1.0.0" decision is made by the maintainer and recorded append-only in ADR-021 (Proposed → Accepted → Enacted). |
| **Feeds** | ADR-021 (1.0.0 Release-Readiness Gate, Proposed); `docs/reviews/2026-06-14-deep-review/05-remediation-roadmap.md` (WS1–WS9 status); `tools/github/issues.yaml` (open-item status). |
| **Date** | 2026-06-21 |

> **Posture (honesty rule / VR-5).** This note is an advisory **planning capture** — the same
> role DN-08 played for maturation and DN-11 played for the post-KC-2 wave. It reopens no
> ratified decision and grades nothing beyond the strength actually established. Statuses cited
> below are read directly from ADR-021 (2026-06-21) and the remediation roadmap (2026-06-15
> status updates); nothing is pre-written or speculatively upgraded. Append-only: supersede,
> do not rewrite.

---

## 1. The gate

**1.0.0 = ADR-021 Gate A + Gate B open rows.** ADR-021 (Proposed, 2026-06-21) is the spine.
It defines two gates:

> **Erratum (2026-06-25, post-audit).** ADR-021 is no longer "Proposed": it reached **Accepted
> (2026-06-21)** and was then **Superseded by ADR-022 (2026-06-23)**, which carries this Gate A/B
> forward as **track T1** (the core/kernel 1.0.0 sub-gate, further amended by ADR-024). The Gate A/B
> *criteria* below are unchanged; only their home document moved (see the §6 changelog spine-erratum).

**Gate A — resolvable now (honesty-integrity + durability):**

| Row | Criterion | Status (per ADR-021) |
|---|---|---|
| A1 | Zero open **High** findings (deep-review Waves 0–1) | ✅ met — WS1–WS7 landed 2026-06-15 |
| A2 | Every **Medium** resolved or explicitly deferred with a one-line rationale | ⏳ verify |
| A3 | Honesty surface **durable**: cargo-mutants green; proptest migration; fuzz targets in CI (WS8) | ⏳ open |
| A4 | `just check` green incl. gitleaks + `cargo deny`/`cargo audit` wired (WS7, WS9) | ⏳ partial |
| A5 | KC-4 numeric cert-overhead budget **threshold** ratified (maintainer decision) | ◻ decision-pending |

**Gate B — decision / external:**

| Row | Criterion | Status (per ADR-021) |
|---|---|---|
| B1 | RFC-0003 §4 reconciled (WS3) + RFC-0006/RFC-0007 `Draft → Accepted` | ⏳ verify |
| B2 | KC-2 (LLM-leverage) verdict recorded | ✅ met — DN-09 §10, 2026-06-21 |

The definition of "gate met" is all open rows resolved. Everything in §3 of this note is
explicitly **post-1.0** and must not be conflated with gate items.

---

## 2. The critical path

An ordered list of the gaps that actually gate 1.0.0. Order: dependencies first, then priority
within each dependency tier. Each entry carries: what it is, why it is needed (gate row), what
must precede it, whether it can run in parallel with other open items, and estimated size.

### GAP-1 — B1 (RFC status reconciliation): verify RFC-0003, RFC-0006, RFC-0007 statuses

**What it is.** ADR-021 Gate B1 asks: is RFC-0003 §4 reconciled (WS3 decision), and are
RFC-0006 and RFC-0007 advanced beyond Draft?

**Current status (read from RFC headers, 2026-06-21):**
- RFC-0003: **Accepted (r4)** — the WS3 §4.1 erratum landed (r3), resonator decode params
  added (r4). The §4 reconciliation required by WS3 is done. Source: RFC-0003 status header;
  remediation roadmap WS3 "decision settled 2026-06-15."
- RFC-0006: **Accepted (r5)** — KC-2 verdict (DN-09) discharged Q1; concrete L3 syntax
  committed. Source: RFC-0006 status header.
- RFC-0007: **Accepted (r4)** — v0 kernel calculus §4.1–4.8 ratified 2026-06-15. Source:
  RFC-0007 status header.

**Conclusion.** GAP-1 appears **already closed** — all three RFCs are Accepted. The ADR-021
§7 open question asks the maintainer to *confirm* this reading rather than assert it. The
concrete-syntax ratification of RFC-0006/0007 remains KC-2-gated (a 1.x item per ADR-021 §5),
but the Accepted statuses themselves satisfy Gate B1. The maintainer should confirm this
explicitly and record it append-only in ADR-021 (closing the B1 ⏳).

**Depends on:** nothing. **Parallelizable:** yes — this is a verification pass, not a build
task. **Est. size:** XS (1 reading pass + ADR-021 annotation, ≤1 hour).

---

### GAP-2 — A2 (Medium-findings ledger): close or explicitly defer every open Medium

**What it is.** Gate A2 requires that every Medium finding from the 2026-06-14 deep review be
either resolved or explicitly deferred with a one-line rationale. Source: ADR-021 A2; remediation
roadmap §1 (status line: "Every Medium resolved or explicitly deferred").

**Remaining open Mediums (read from remediation roadmap WS status notes, 2026-06-15):**
- **WS2 remaining:** A6-03 (schema-example pinning), A6-06 (recon schema reconciliation),
  A6-08, A6-09, A1-04, A1-05.
- **WS3 remaining:** A3-04 (MAP-I/MAP-B bind alphabet checks), A3-05 (Bundle.hs header),
  A3-06/C1-04 (on-expectation qualifier), A3-07 (EmptyCodebook variant), A3-08/09/10
  (test gaps).
- **WS4 remaining:** A4-03 (per-frame eval depth), A4-04 (Wf-path test), reject-corpus
  assertion strength.
- **WS5 remaining:** A5-02 (non-ternary layout refusal), A5-03 (unpack\_trits Result),
  A5-05 (vacuous sweep assert), A5-06 (false comment), A5-07 (pin eps constants), A5-08 (nit).
- **WS6 remaining:** A6-05 (LSP unsupported-swap-pair diagnostic), A6-10/B2-04 (exec
  allow\_untrusted guard), A6-11 (xtask kc4 precheck).

**Why needed.** The gate requires a complete ledger — either the Medium is fixed, or there is
an explicit one-line rationale for why it is deferred (e.g., "deferred post-1.0: cosmetic nit;
no honesty impact"). Unresolved Mediums with no rationale leave the honesty-integrity claim
ungrounded. Source: ADR-021 A2; remediation roadmap §5 gate criterion 2.

**Depends on:** GAP-1 (to know which WS3 items are truly open vs. already swept). In practice
the Medium sweep can begin independently since it touches different code areas. **Parallelizable:**
yes — WS2/WS3/WS4/WS5/WS6 remaining items are disjoint by crate. **Est. size:** M (one to three
days of engineering, depending on how many are fixed vs. deferred).

---

### GAP-3 — A4 (cargo deny / cargo audit wired): close the `just check` partial skip

**What it is.** Gate A4 is partially met: `just check` is green locally, but `cargo deny` and
`cargo audit` currently **skip** (not installed). Source: ADR-021 A4 note "must be wired +
green for the gate"; remediation roadmap WS7 ("wire `cargo audit` + `cargo deny` into
`scripts/checks/`").

**Why needed.** A `just check` that silently skips supply-chain checks is not the same as one
that passes them. The gate requires these to be wired and green, not silently skipped.
`cargo deny` verifies license policy and duplicate-dependency hygiene; `cargo audit` surfaces
known CVEs. These are the minimum supply-chain bar for a 1.0.0 (source: ADR-021 A4;
remediation roadmap WS7; CLAUDE.md "no black boxes").

**Depends on:** nothing (tool installation is independent of code changes). **Parallelizable:**
yes — install and wire the tools as a standalone PR independent of all other gaps.
**Est. size:** S (tool install + justfile wiring + CI update, a few hours; cargo deny policy
file may need initial tuning).

---

### GAP-4 — A3 (WS8 durability): cargo-mutants, proptest migration, fuzz targets

**What it is.** Gate A3 is the largest remaining engineering gap. WS8 ("Durable test
infrastructure") requires: (a) `cargo-mutants` on `mycelium-core`/`-cert`/`-numerics`/`-vsa`
as an opt-in `just mutants` recipe, green on the trusted base; (b) proptest migration of the
hand-rolled fixed-seed LCG property tests in `-numerics`/`-vsa` (shrinking, `PROPTEST_CASES`,
CI seed rotation); (c) `cargo-fuzz` targets for the L1 lexer+parser, the M-210 checker, and
the schema/manifest deserializers. Source: ADR-021 A3; remediation roadmap WS8.

**Why needed.** WS1–WS7 fixed every High and most Mediums. WS8 turns those one-time fixes into
durable guarantees: a honesty-surface fix that has no mutant witness can regress silently. The
remediation roadmap (§3 WS8) put it directly: "This is what turns 'we fixed it once' into
'it stays fixed.'" Source: ADR-021 A3; remediation roadmap WS8.

**Sub-tasks (independent within WS8):**
1. `just mutants` recipe + cargo-mutants on the four trusted-base crates.
2. proptest migration of LCG tests in `-numerics` and `-vsa`.
3. `cargo-fuzz` targets (lexer/parser, checker, deserializers).
4. `cargo-llvm-cov` as a mutation-testing map (not a coverage gate).

**Depends on:** GAP-2 (the Medium sweep closes the remaining test gaps; some WS8 targets
overlap the same code areas). In practice (a) and (b) can start immediately on the already-fixed
code; (c) is largely independent. **Parallelizable:** yes — the four WS8 sub-tasks are
parallelizable across agents (disjoint crates/tools). **Est. size:** L (the largest gap; cargo-
mutants alone may require triage of survivors; full WS8 is likely 1–2 weeks of focused work).

**Open question (ADR-021 §7 Q3).** The maintainer must decide: is WS8 a hard 1.0.0 blocker,
or is "cargo-mutants opt-in + fuzz targets" a "1.0.1 hardening" follow-up? This note records
both as valid framings. If WS8 is a hard blocker, GAP-4 gates the release. If the maintainer
accepts a "durability-deferred" 1.0.0, GAP-4 becomes a tracked 1.0.1 item and must be recorded
explicitly (not silently skipped).

---

### GAP-5 — A5 (KC-4 cert-overhead threshold): maintainer decision on the numeric budget

**What it is.** Gate A5 is decision-pending: the KC-4 cert-overhead *measurement* is done
(M-212, `cargo xtask kc4` passes), but the *threshold* — the numeric budget the maintainer
accepts as "within KC-4 budget" — has not been ratified. Source: ADR-021 A5 "threshold is a
maintainer call"; remediation roadmap §5 gate criterion 5.

**Why needed.** KC-4 ("small, auditable, measurable overhead") is a non-functional requirement.
Its fulfillment claim ("cert overhead within KC-4 budget") is currently `Declared` because no
threshold has been stated. To grade it `Empirical` (measured against a ratified bar), the
maintainer must set the bar. Without this, the KC-4 claim is "budget = whatever the current
measurement happens to be" — not a verifiable assertion. Source: ADR-021 A5; foundation §2
(KC-4); VR-5 (never upgrade without checked basis).

**Depends on:** the measurement exists (M-212 done). This is a pure maintainer decision with
no engineering prerequisite. **Parallelizable:** yes — this is an independent decision, not
a build task. **Est. size:** XS (the maintainer reads the `xtask kc4` output, decides the
threshold, and records it in ADR-021 or a short ADR-021 annotation).

---

### GAP-6 — ADR-021 ratification: Proposed → Accepted

**What it is.** ADR-021 itself is currently **Proposed**, not Accepted. The maintainer must
ratify the *criteria* (agreeing that "these are the 1.0.0 release-readiness criteria") before
the gate can be called met.

> **Erratum (2026-06-25):** this GAP-6 premise is **closed/superseded** — ADR-021 *was* ratified to
> **Accepted (2026-06-21)** and is now **Superseded by ADR-022 (2026-06-23)**; the criteria-ratification
> this GAP asked for happened, and the gate moved into ADR-022 track T1 (amended by ADR-024). See §6. The "ship 1.0.0" act (Accepted → Enacted at the tagged release)
is separate. Source: ADR-021 §6; CLAUDE.md house rule #3 (append-only decisions; Proposed →
Accepted → Enacted; never skip straight to Enacted).

**Why needed.** Without ratification, the gate criteria are advisory (a planning document, like
this DN). The maintainer's sign-off on the *criteria* is what makes the gate binding. Source:
ADR-021 §1 and §6; CLAUDE.md house rule #3.

**Depends on:** GAP-5 (the maintainer reviews A5 as part of the ratification walk-through
described in ADR-021 §7). Also benefits from GAP-1 and GAP-2 being confirmed. **Parallelizable:**
partially — the maintainer can review ADR-021 in parallel with engineering on GAP-3/GAP-4.
**Est. size:** XS (a reading pass + status flip + changelog entry; pure maintainer time).

---

### Dependency / ordering summary

```
GAP-1 (B1 verify)   ─────────────────────────────────────────────────────────┐
GAP-2 (A2 Mediums)  ──────────────────────────────────┐                      │
GAP-3 (A4 cargo deny)  ──── (independent) ────────────┤                      │
GAP-4 (A3 WS8 durability)  ─ (starts after GAP-2) ───►│                      │
GAP-5 (A5 threshold)  ───── (maintainer, independent) ─┤                      │
                                                        │                      │
GAP-6 (ADR-021 Accepted) ◄──────────────────────────────────────────────────┘
          │
          ▼
     Tag 1.0.0
```

Items that can run immediately in parallel: GAP-1 (verification), GAP-3 (tooling), GAP-5
(maintainer decision). GAP-2 can begin in parallel with GAP-1 (the reading pass is fast);
GAP-4 can begin after GAP-2's Medium sweep identifies which test gaps remain open.

---

## 3. Explicitly out of scope for 1.0.0

These are Phase-3+ maturation items, confirmed out-of-scope by ADR-021 §5 and the remediation
roadmap §5. They are tracked as 1.x or post-1.0 but must **not** be treated as release blockers.

| Item | Why out of scope | Source |
|---|---|---|
| Native libMLIR/LLVM codegen | Phase-3 maturation; interpreter is the trusted base for 1.0.0 | ADR-021 §5; remediation roadmap §5 |
| JIT for dynamic VSA workloads | Downstream of native path (E3-4); Phase-3 | ADR-021 §5; phase-3.md §1 |
| Semantic-level projections (FR-C1) | Phase-3 exploratory; KC-2 cleared but not a kernel gate | ADR-021 §5; phase-3.md epic E3-1 |
| Resonator factorization (FR-C2) | Probabilistic-only, explicitly post-1.0 | ADR-021 §5; phase-3.md epic E3-5 |
| BitNet packed-ternary acceleration (FR-C3) | Phase-3 performance maturation | ADR-021 §5; phase-3.md epic E3-6 |
| Native ternary-hardware path | Forward-compat stub; Phase-3 (R7) | ADR-021 §5; phase-3.md epic E3-7 |
| Concrete surface-language ratification (KC-2-gated) | Scoped to 1.x; the kernel/core is the 1.0.0 product | ADR-021 §2; RFC-0006 status line |
| Self-hosting (Phase 5, M-502) | Phase-5; the Rust kernel is the 1.0.0 delivery | ADR-021 §5; phase-5.md |
| M-381 arms 3/5 ablation (grammar-constrained decoding, embedded-DSL) | Non-blocking research follow-up; KC-2 verdict already recorded | ADR-021 B2 (met); issues.yaml M-381 STATUS 2026-06-21 |
| AI co-authoring loop (M-330, E3-2) | Phase-3 tooling; operational arm needs LLM API | ADR-021 §5; phase-3.md epic E3-2 |

---

## 4. Parallelization plan

The six gaps split naturally into three parallel tracks once the initial verification pass
(GAP-1) is done:

**Track I — Verification / decision (no engineering, maintainer-driven):**
- GAP-1: RFC status confirmation (XS, can start now).
- GAP-5: KC-4 threshold decision (XS, can start now; reads `cargo xtask kc4` output).
- GAP-6: ADR-021 Proposed → Accepted (XS, after GAP-5 + GAP-1 confirmed).

**Track II — Tooling wiring (independent of Track III code changes):**
- GAP-3: `cargo deny`/`cargo audit` install + justfile wiring + CI update (S, can start now;
  no shared files with Track III).

**Track III — Code / test engineering (two sequential sub-waves):**
- GAP-2 (wave A): Medium sweep — WS2/WS3/WS4/WS5/WS6 remaining items. Disjoint by crate;
  can run as a swarm (one agent per WS, disjoint directories). Orchestrator owns CHANGELOG +
  Doc-Index.
- GAP-4 (wave B): WS8 durability — cargo-mutants + proptest + fuzz. Starts after the Medium
  sweep closes remaining test gaps; otherwise mutant-survivors may include already-identified
  gaps. WS8 sub-tasks (mutants, proptest, fuzz, cov-map) are disjoint and can run as a swarm.

**Constraint.** GAP-4's mutant baseline should be measured *after* GAP-2's fixes land, so
surviving mutants reflect genuine test gaps rather than known open Mediums. This is why
Track III is wave-A-then-wave-B rather than fully parallel.

**Swarm eligibility:** Track III wave-A is a strong swarm candidate (five independent WS
crates, one agent each). Track II (GAP-3) can run as a leaf agent alongside wave-A.

---

## 5. Recommended next 3 actions

**Action 1 (do now, maintainer + any agent): Confirm Gate B1 and A2 ledger (GAP-1 + start
GAP-2).**

Read RFC-0003 (Accepted r4), RFC-0006 (Accepted r5), RFC-0007 (Accepted r4) status headers
and record in ADR-021 §4 that B1 is met. Simultaneously, produce the explicit Medium ledger
from the WS2–WS6 remaining items listed in the remediation roadmap (2026-06-15 status notes)
and for each item: (a) fix it, or (b) write a one-line deferral rationale. This closes Gate A2
and unblocks the rest of Track III. Source: ADR-021 A2, B1; remediation roadmap WS2–WS6
status notes.

**Action 2 (do in parallel with Action 1, any agent): Wire `cargo deny` and `cargo audit` into
`just check` (GAP-3).**

Install `cargo-deny` and `cargo-audit` (or add them to `just setup`), create a minimal
`deny.toml` (licenses + advisories), update `scripts/checks/all.sh` with the skip-if-missing
pattern, and confirm `just check` green with them active. This is a standalone PR with no
dependency on the Medium sweep. Source: ADR-021 A4; remediation roadmap WS7.

**Action 3 (maintainer, do before or in parallel with Actions 1–2): Ratify the KC-4 threshold
and move ADR-021 to Accepted (GAP-5 + GAP-6).**

Run `cargo xtask kc4`, read the cert-overhead output, and decide on a numeric threshold (e.g.,
"cert path adds ≤ N µs per swap" or "cert overhead ≤ P% of total op time"). Append the
threshold to ADR-021 (closing A5). Then, with B1 confirmed (Action 1) and the A4/A2 work
underway, move ADR-021 from Proposed to Accepted — recording that the maintainer agrees with
these release-readiness criteria. This does **not** ship 1.0.0; it ratifies the gate the
subsequent engineering closes. Source: ADR-021 §6, §7 Q1–Q2; CLAUDE.md house rule #3.

*(After Actions 1–3: launch the WS8 durability swarm (GAP-4) and, once `just check` is green
with all gaps closed, the maintainer cuts the tagged 1.0.0 release and moves ADR-021 Accepted →
Enacted.)*

---

## 6. Changelog

- **2026-06-25 — Decisions + spine erratum (post corpus-alignment audit; advisory note, no status move).** Three append-only records, none rewriting the §1–§5 prose:
  - **Spine re-framed under ADR-022 (GAP-6 hygiene).** The note's spine treats **ADR-021** as "Proposed, needs ratification". That is **superseded**: ADR-021 reached **Accepted (2026-06-21)** and was then **Superseded by ADR-022 (2026-06-23)** — its kernel **Gate A/B is carried forward, preserved not discarded, as ADR-022 track T1** (the core/kernel 1.0.0 sub-gate; further amended by **ADR-024** to add E19-1 to T1's Definition of Done). Read every "ADR-021 (Proposed)" / GAP-6 reference below as **ADR-021 (Accepted → Superseded-by-ADR-022); the gate now lives in ADR-022 §4/§5 track T1**. The §2 GAP table and §1 Gate-A/B rows remain the correct *criteria* — only their *home document* moved.
  - **D4 — DEFER the full Gate A2/A3/A4/A5 pass-criteria definition to post-T1 (advisory for now).** The maintainer's decision (2026-06-25): the precise *pass thresholds* for A2 (Medium-ledger completeness), A3 (durability — mutants/proptest/fuzz), A4 (`cargo deny`/`cargo audit` wiring) and A5 (KC-4 cert-overhead numeric budget) are **deferred to post-T1**; they remain **advisory** until then. The GAP-2 ledger subsection below stands as the (draft) A2 evidence; nothing here ratifies a gate (ADR-022 §5 + the maintainer sign-off do).
  - **D8 — DEFER Theme-A verification to post-T1.** The VSA/numerics confirmatory probes — the in-repo **performance benchmark** (no Mycelium-native benchmark yet; perf figures stay `Declared` until measured, VR-5), the single Liquid-Haskell **`bundle` capacity-refinement** probe (KC-1), and the **≥100-vector measured resonator corpus** — are **deferred post-T1**. They are confirmatory (they gate only *upgrading* a `Declared`/axiomatized-citation tag to `Empirical`/`Proven`), not correctness blockers; recorded here so the deferral is explicit, never silent (G2).
- **2026-06-21 — Draft.** Planning capture of the remaining gaps to a 1.0.0 kernel/core release,
  grounded in ADR-021 (Proposed, 2026-06-21) and the deep-review remediation roadmap
  (WS1–WS9 status, 2026-06-15). Identifies six gaps (GAP-1 through GAP-6), orders them by
  dependency and priority, partitions into three parallel tracks, and recommends three immediate
  actions. Records that Gate A1 and Gate B2 are met; B1 is likely met pending maintainer
  confirmation. Decides nothing normatively.

---

## GAP-2 Medium-Findings Ledger (draft, 2026-06-24)

> **Posture (honesty rule / VR-5).** This is a **DRAFT verification ledger** for ADR-021 **Gate A2**
> (GAP-2, §2 above), produced by an agent re-grounding the prior disposition record
> (`docs/reviews/2026-06-14-deep-review/06-medium-findings-ledger.md`, M-653, 2026-06-21) against the
> **live tree** (`origin/main` tip `db4a6be`). It **does not ratify the gate** — Gate A2 requires the
> maintainer's sign-off (ADR-021 §6). Append-only: this records a new verification pass; it rewrites
> no prior prose and re-grades nothing upward. Full per-finding evidence (greps, `file:line`, test
> runs) is archived in `docs/handoffs/gap-2-ledger-context.md`.

**Scope.** The 25 open Medium finding-ids listed in §2 GAP-2 (WS2–WS6), matching the M-653 ledger.
Each was re-located in the current tree by its cited test/variant/marker name; a representative
subset (one per workstream with a Rust target) was **executed green** (not merely confirmed present).

**Verdicts (each FIXED; grounding cited per-row in the handoff):**

| WS | Findings | Verdict | Note |
|---|---|---|---|
| WS2 | A1-04, A1-05, A6-03, A6-06, A6-08, A6-09 | FIXED ×6 | A6-03 is the M-653-landed wire-spelling pin (executed: pass). |
| WS3 | A3-04, A3-05, A3-06/C1-04, A3-07, A3-08, A3-09, A3-10 | FIXED ×7 | A3-05 is an honest comment-reconciliation (Haskell/z3 absent locally → `Declared` here, README is the checkable artifact). A3-08/A3-10 are documented scope notes, not new behavioral tests. A3-06 is *qualified, not upgraded*. |
| WS4 | A4-03, A4-04, reject-corpus | FIXED ×3 | reject-corpus integrity is the M-653-landed bidirectional test. |
| WS5 | A5-02, A5-03, A5-05, A5-06, A5-07, A5-08 | FIXED ×6 | **A5-08 citation FLAG:** the M-653 ledger cites `mycelium-dense/lib.rs::bits_per_element(Tl2)`, but the fix actually lives in `mycelium-select::packing_bits_per_element` + `mycelium-mlir/pack.rs`. Fix is present; the *citation* needs correcting. |
| WS6 | A6-05, A6-10/B2-04, A6-11 | FIXED ×3 | A6-11 is structural (xtask runner, no unit surface). |

**Tally: 25 finding-ids · 25 FIXED · 0 DEFERRED · 0 N-A · 0 verdict-flags.** One non-verdict
**citation flag** (A5-08, above) for the maintainer to correct in the record. Subject to maintainer
ratification, Gate A2 is supportable on verified evidence.

### Changelog (this subsection)

- **2026-06-24 — Draft ledger appended.** GAP-2 Gate-A2 verification pass: re-grounded all 25 open
  Mediums against the live tree, executed a representative subset green, recorded one citation flag
  (A5-08). Draft — pending maintainer sign-off. Evidence: `docs/handoffs/gap-2-ledger-context.md`.
