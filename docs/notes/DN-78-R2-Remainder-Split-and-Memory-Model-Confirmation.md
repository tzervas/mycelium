# Design Note DN-78 — frz Lane C: the M-828 R2 Remainder Buildable-vs-Research Split, and the RFC-0027/DN-32 Memory-Model Confirmation

| Field | Value |
|---|---|
| **Note** | DN-78 |
| **Status** | **Accepted** (2026-07-02 — decided under the maintainer's **delegation** of the R2-split and memory-model decisions to the orchestrator, recorded 2026-07-02 in the frz Lane C brief; the split below is the decision, recorded append-only per that delegation. The delegation itself is `Declared` as relayed — see Posture) |
| **Decides** | Under delegated authority: the split of the **M-828 tail** (the R2 remainder beyond DN-70's D-lite) into the **Phase-I-buildable/directed subset** (§3 — the mechanized `SelectionPolicy` capture-and-set surface, runtime-side, plus the never-silent residual ledger) and the **research-spike/Phase-II remainder** (§4 — `mesh`/`graft`/`xloc`/`cyst` activation, multi-node maturity, and the heavy runtime-maturity items M-869/M-868/M-831). Confirms (does not decide) the RFC-0027/DN-32 memory-model sequencing precondition (§2) |
| **Feeds** | M-963 (activate the directed remainder — scope fixed here); M-964 (RT2/determinism honesty audit — recorded as the dated appendix below); M-828 (its body carries the tail split once the integrator applies FLAG-F3); DN-63 (per-construct decomposition — unchanged, cited row-by-row); DN-70 (the D-lite split this note builds past); `.claude/kickoffs/frz.md` Lane C |
| **Task** | M-962 (epic E31-1, kickoff `frz`, Lane C) |
| **Date** | July 2, 2026 |

> **Posture (transparency rule / VR-5 / G2).** This is a scope-split dossier in the DN-70
> lineage. Every claim is tagged: in-repo state assertions are `Empirical` (verified against
> the working tree at `origin/dev` `629aa12`, 2026-07-02, file:line cited); the split itself
> is a **decision under delegation** — the maintainer delegated the R2-split and memory-model
> decisions to the orchestrator (2026-07-02, frz Lane C brief), and this note exercises that
> delegation rather than waiting on a per-item ratification. The delegation is `Declared` as
> relayed (this note cannot independently verify the maintainer's statement; the citation is
> the kickoff brief). No DN-63 open question is resolved here by a plausible-sounding answer;
> where the buildable subset narrows a FLAG, the narrowing is explicit and the full FLAG stays
> open for its Phase-II vehicle. **Append-only:** amendments land as dated sections below.

---

## 1. Context and gates

**M-828** (DN-64 OQ-B disposition) is the full R2 runtime-vocabulary maturity wave:
`forage`/`backbone` activation **plus** a mechanized `SelectionPolicy` capture-and-set
surface (M-828 body, `tools/github/issues.yaml`; research/27-dn64-ergonomics-rnd-RECORD.md §2).

**DN-70** (Accepted 2026-07-02, maintainer-signed) already carved out the **D-lite usability
subset** for kickoff `enb`: D1 (`forage` single-node activation, M-906) and D2 (`backbone`
verification, M-907), deferring rows R-1…R-6 as the H2/M-828 remainder. At this note's base
(`629aa12`) M-906/M-907 are `status:todo` — enb's parallel lane, **not** touched here
(`Empirical`: issues.yaml).

**This note is the next slice of that remainder.** Kickoff `frz` Lane C (M-962→M-963→M-964)
directs: scope which of the R-1…R-6 rows are Phase-I-buildable/**directed** vs
research-spike/Phase-II, confirm the RFC-0027/DN-32 memory-model precondition, then build the
directed subset in `mycelium-std-runtime` (M-963) and run the RT2/determinism honesty audit
(M-964).

**The R1 gate is satisfied** (`Empirical`): DN-63 §7 gates all R2 work on M-667;
M-667 is `status:done` (issues.yaml; already verified by DN-70 §1).

**The delegation (the decision basis, `Declared` as relayed):** the maintainer delegated the
R2-split and memory-model decisions to the orchestrator on 2026-07-02; the frz Lane C brief
passes that delegation to this task with the instruction to decide per this dossier's
recommendation and proceed to build, recording the decision append-only. This note is that
record. A maintainer amendment, if any, lands as a dated section below (append-only).

---

## 2. The RFC-0027/DN-32 memory-model precondition — CONFIRMED (`Empirical`)

The frz decision table names "RFC-0027/DN-32 memory-model ratification" as the runtime-lane
sequencing precondition, with the instruction *confirm, don't guess*. Confirmed, with
evidence:

- **RFC-0027 is `Accepted` (2026-06-25), maintainer-ratified** — its status header records
  "*Proposed → Accepted, ratified by the maintainer 2026-06-25*"; OQ-1 (the banner blocker:
  sibling sweep/reclamation coupling) and OQ-4 are resolved by DN-32; OQ-2/5/6 are deferred
  non-blockers (`docs/rfcs/RFC-0027-Memory-Management-and-Reclamation.md`, header + banner).
- **DN-32 is `Accepted` (2026-06-25), maintainer-ratified** — the three-layer hybrid memory
  architecture (affine-first / explicit RC / region-in-scope; sibling reclamation
  concurrent-by-default) (`docs/notes/DN-32-Three-Layer-Hybrid-Memory-Architecture.md`, header).
- **The runtime lane already builds on it, in this crate:** MEM-1 (reclamation EXPLAIN/audit
  record, relocated to `mycelium-rt-abi`, re-exported), MEM-2 (`rc` — non-atomic intra-hypha
  RC cell, DN-32 §2.2), MEM-3 (`region` — batched scope-exit reclamation, DN-32 §2.3 /
  RFC-0027 §10.3), plus the live wiring (`scope_region`, `network` ChannelClose) are landed
  (`crates/mycelium-std-runtime/src/lib.rs` module docs; `rc.rs`/`region.rs`/`scope_region.rs`).

**Verdict: the precondition holds — no FLAG needed.** Two honest residuals, recorded so the
confirmation is not overstated (G2):

1. RFC-0027 is `Accepted`, **not `Enacted`** — the Layer-2 static uniqueness analysis and the
   cross-hypha sharing sub-question (RFC-0027 §7 follow-on / DN-32 §7) remain open. They do
   not gate this lane: nothing in §3's buildable subset takes a new memory-model decision.
2. The sibling-reclamation safety tag is **`Proven`-modulo the LR-9 side-condition** (RFC-0027
   §8 wording; mirrored at `region.rs:48`) — a qualified claim, not a bare `Proven`; there is
   no in-repo mechanized proof. The M-964 audit (appendix) revisits this tag.

---

## 3. The buildable/directed subset (Phase I — built by M-963, in `mycelium-std-runtime`)

All rows decided under the §1 delegation; each cites its basis. The subset deliberately adds
**no new kernel mechanism and no distribution machinery** (KC-3), riding the existing,
Accepted RFC-0005 `SelectionPolicy` machinery (`mycelium-select`,
`crates/mycelium-select/src/lib.rs` — content-addressed policies, mandatory EXPLAIN,
deterministic `select`, `PolicyRegistry`).

### B-1 — Mechanized `SelectionPolicy` **capture** (record/replay)

The first half of M-828's capture-and-set DoD (research record §2.2/§2.4 item 3):

- **Capture-to-value:** materialize the policy that decided a recorded `Explanation` back
  into a nameable, diffable, inspectable `SelectionPolicy` value via the `PolicyRegistry` —
  never an opaque handle (ADR-006). An unresolvable `policy_ref` is an explicit typed error,
  never a silent reconstruction (G2).
- **Replay:** re-run the recorded inputs (honoring the recorded override state) against the
  captured policy and compare the decision to the record; divergence is an explicit typed
  error, never a silent pass. Replay-reaches-the-recorded-decision is the executable
  differential for this surface (the record-vs-replay two-way; no L1/AOT path exists for it
  yet, so a three-way differential is not executable here — stated, not skipped).

**Why buildable now:** RFC-0005 is Accepted and `mycelium-select` is landed and tested; the
capture surface is pure single-node library work over existing values. No open research
question is guessed: the capture surfaces exactly the RFC-0005-conformant policy value
(research record §2.2 transparency constraints).

### B-2 — Mechanized `SelectionPolicy` **setting** (reified setter surface)

The second half of M-828's capture-and-set DoD:

- A runtime **policy slot** per RFC-0005 site (swap-target / packing / placement) holding the
  active `SelectionPolicy`, with a `set` operation that is itself reified: every set appends
  a transition record (site, previous policy ref, new policy ref, sequence number) to an
  inspectable, append-only log. A mechanized set is never a silent override — the transition
  is recorded and EXPLAIN stays answerable afterward (research record §2.2: "the setter
  surface must itself be inspectable and record the transition (G2)").
- Selection **through** the slot records the mandatory `Explanation` into a trace the
  developer can extract for capture/diffing — the "runtime records which policy it applied"
  half of mechanized capture.

**Why buildable now:** same basis as B-1. The slot is runtime-side machinery, exactly the
posture `lib.rs` already documents ("the runtime *machinery* they will dispatch to … is what
this crate now provides").

### B-3 — The never-silent **R2 residual ledger + refusal surface**

G2 mechanized: a typed, tested ledger in `mycelium-std-runtime` with one row per deferred
item in §4 — construct, why deferred (the unmet prerequisite), and the tracker — plus a
`require`-style refusal entry point returning an explicit typed error for any runtime path
that would need a deferred construct. The deferral itself becomes inspectable and
regression-guarded (a deferred construct without a ledger row fails a test), not prose.

*Naming note (ADR-020 §5):* the reserved vocabulary stays out of public **operation names**
(the guarantee-matrix test enforces this); the ledger *names the constructs it refuses* in
its data — a refusal must name what it refuses to be never-silent. This is the same posture
as the parse-time teaching diagnostics (`mycelium-l1/src/parse.rs`).

### Explicit residuals **within** the buildable subset (not built; ledgered)

- **The L1 surface binding** of capture/set (surface syntax for capturing/setting a policy
  from Mycelium code) — needs the serial l1 lane and an RFC vehicle (RFC-0008 §4.5 status
  rule; DN-70 FLAG-D2's vehicle question applies to it too). Stays on M-828. Ledger row.
- **Placement-site live consumers:** the placement slot exists and is settable/EXPLAIN-able,
  but its only in-tree consumer shape today is the single-node scheduler posture
  (`mycelium-sched::StealPolicy`); multi-node placement is §4 R-6. Ledger row.

---

## 4. The research-spike / Phase-II remainder (deferred, never-silent)

Each row `Declared`, grounded in DN-63's prerequisite analysis (unchanged from DN-70 §5 where
overlapping); none can land in Phase I without guessing past an open research question
(G2/VR-5). All rows get B-3 ledger entries.

| # | Deferred item | Unmet prerequisite | Tracker | Basis |
|---|---|---|---|---|
| R-1 | `mesh` activation | DN-61 B.1 (clock) + B.2 (Byzantine) research passes — the longest-lead R2 items (the gossip/Byzantine long pole); v0 protocol choice (DN-63 FLAG-4) + `ProbabilityBound` commitment | M-913 (research lane, non-gating); remainder on M-828 | DN-63 §3.2/§4/§5 |
| R-2 | `graft` activation | RFC-0028 §7 capability follow-on (FLAG-10/11/12 open) | M-828 | DN-63 §3.4 |
| R-3 | `xloc` activation | `mesh` (carrier) + `graft` (capability check) at least Accepted; wire-format swap story (FLAG-1/2/3) | M-828 | DN-63 §3.1/§4 |
| R-4 | `cyst` activation | `xloc` (mobility) + RFC-0027 OQ-3 reclamation-in-dormancy (FLAG-7/8/9; OQ-3 is *mitigated*, not resolved — RFC-0027 banner) | M-828 | DN-63 §3.3 |
| R-5ʹ | Capture/set **L1 surface** (the residual of R-5 after B-1/B-2) | l1 serial lane + RFC vehicle (RFC-0008 §4.5 status rule) | M-828 | §3 above; DN-70 R-5 |
| R-6 | `forage`/`backbone` maturity: multi-node candidate sets, full FLAG-13 signal inventory, real transport paths, backbone promotion mechanism + FLAG-16 affinity | `mesh` (R-1); the backbone implementation RFC (M-825 resolution note) | M-828 | DN-63 §3.5/§3.6; DN-70 R-6 |
| P-1 | M-869 — AOT/interp async parity | needs-design; heavy runtime maturity | M-869 | frz §Scope (explicitly Phase II, non-gating) |
| P-2 | M-868 — scheduler leapfrogging | needs-design; heavy runtime maturity | M-868 | frz §Scope (Phase II, non-gating) |
| P-3 | M-831 — substrate/hypha reclamation interaction (LR-8 × RT7 × RFC-0014 recovery) | type:research; needs the DN-59/`graft` direction | M-831 | frz §Scope (Phase II, non-gating); M-831 body |

*Wording note (minor FLAG-F1):* the frz Lane C brief glosses the "mesh gossip/Byzantine long
pole" with the id M-831; per issues.yaml, M-831 is the substrate/hypha **reclamation**
research and the mesh research lane is **M-913**. Both are Phase-II; the table above cites
each accurately.

**Parallel lane, not this note's scope:** D-lite (M-906 `forage` single-node activation,
M-907 `backbone` verification) is kickoff `enb`'s, per DN-70 — `status:todo` at `629aa12`,
disjoint from Lane C (this lane owns `crates/mycelium-std-runtime/**` only).

---

## 5. Honest-tag summary (VR-5)

| Claim | Tag | Basis / gap |
|---|---|---|
| RFC-0027/DN-32 both `Accepted`, maintainer-ratified 2026-06-25 | `Empirical` | Status headers, quoted §2; verified at `629aa12` |
| MEM-1/2/3 + live wiring landed in `mycelium-std-runtime` | `Empirical` | Module inventory, file paths in §2 |
| M-667 (R1) done ⇒ DN-63 §7 gate satisfied | `Empirical` | issues.yaml; DN-70 §1 |
| The delegation (maintainer → orchestrator → this task) | `Declared` | Relayed via the frz Lane C brief 2026-07-02; not independently verifiable from the tree |
| The split itself (§3 buildable vs §4 deferred) | Decided under delegation; the *grounding* of each row is `Declared` (DN-63 §4 blocker analysis — no formal dependency proof) | §1; DN-63 §4 |
| B-1/B-2 prerequisites met (RFC-0005 Accepted; `mycelium-select` landed) | `Empirical` | `crates/mycelium-select/src/lib.rs` (SelectionPolicy/select/explain/PolicyRegistry) |
| Replay-reaches-recorded-decision | `Empirical` once the property test lands with M-963; never `Proven` without a mechanized theorem | §3 B-1; the M-964 appendix audits the landed tag |
| Deferred rows' unbuildability in Phase I | `Declared` | DN-63 prerequisite analysis; DN-70 §5 |

---

## 6. FLAGs

| FLAG | What it is | Who |
|---|---|---|
| **FLAG-F1** | frz-brief wording: the mesh long pole is tracked by M-913 (research) + the M-828 remainder, not M-831 (which is the substrate/hypha reclamation research). §4 wording note | Orchestrator (kickoff doc wording; no action needed in-tree) |
| **FLAG-F2** | **Spec amendment for the additive API** (DN-66 freeze): M-963 adds public modules to `mycelium-std-runtime` beyond the frozen v0 R1 baseline documented in `docs/spec/stdlib/runtime.md`. Additive, not breaking — but per the DN-66 freeze note in `lib.rs`, the spec + changelog must record it, and `docs/spec/**` is not this lane's to edit | Integrator |
| **FLAG-F3** | **M-828 body update** (orchestrator-owned issues.yaml): append the Lane C tail split — B-1/B-2 landed runtime-side (M-963), R-5ʹ (L1 surface) + R-1…R-4/R-6 remain on M-828 per §4 | Integrator |
| **FLAG-F4** | **DN slot:** DN-78 minted per the orchestrator's assignment (highest landed DN at `629aa12` is DN-72; parallel frz/flp lanes may mint DN-73…77). Integrator re-verifies the slot at merge (mitigation #1) and registers the Doc-Index row | Integrator |
| **FLAG-F5** | `CHANGELOG.md`, `docs/Doc-Index.md`, `docs/api-index/` regeneration, and M-962/M-963/M-964 issue close-out are orchestrator/integration-tier owned — not touched from this branch | Integrator |

---

## Definition of Done

This DN is **Accepted** at creation (the §1 delegation is the acceptance basis; recorded
append-only). It is **Resolved** when:

1. M-963 lands the §3 subset (B-1/B-2/B-3) with green change-scoped checks, and
2. M-964's audit is recorded as the dated appendix below, and
3. the integrator applies FLAG-F2…F5 (spec amendment note, M-828 body, index rows).

A maintainer amendment to the split, if any, supersedes by dated section — never a rewrite.

---

## Appendix — M-964 RT2/determinism honesty audit (2026-07-02, append-only)

**Scope** (`Empirical` — audited at the M-963 tip of this branch): every runtime/concurrency
guarantee tag in `mycelium-std-runtime` — the 29 `guarantee_matrix::MATRIX` rows, the 7
in-crate `*_STRENGTH` constants (`task`/`dataflow`/`network`/`colony`), the 5 re-exported
constants (owned by `mycelium-sched` / `mycelium-rt-abi`; observed values recorded), and the
prose tag tables in `rc.rs` / `region.rs` / `scope_region.rs` / `dataflow.rs` / `task.rs` /
`colony.rs` / `network.rs`.

**Audit rule applied** (the M-964 DoD): a determinism claim stays `Empirical` unless a
machine-checked side-condition upgrades it; no `Proven` without a checked basis.

| # | Finding | Verdict |
|---|---|---|
| 1 | **No `Proven` anywhere in the machine-readable surface** — zero MATRIX rows and zero strength constants at `Proven` (there is no in-repo mechanized concurrency theorem) | Holds; now **regression-guarded** by `determinism_claims_stay_honest` (`src/tests/guarantee_matrix.rs`), which fails on any future `Proven` row |
| 2 | **Cross-thread / schedule-dependent determinism claims are all `Empirical`**, each with a property/differential test: Kahn-determinism (Scope/Colony/Network + `KAHN_DETERMINISM_STRENGTH`, `SCOPE_JOIN_STRENGTH`, `COLONY_KAHN_STRENGTH`), RT2 sequentialization (OS threads + work-stealing, `SCHEDULER_RT2_STRENGTH`), liveness ×2 (`SCHEDULER_LIVENESS_STRENGTH`), deadlock detection/freedom (`DEADLOCK_DETECTION_STRENGTH`), supervision propagation (`SUPERVISION_PROPAGATION_STRENGTH`), and the new M-963 replay differential | Retained at `Empirical` — no upgrade (no checked side-condition exists) |
| 3 | **Two determinism claims are `Exact`:** `SweepOrder determinism` (a pure function of the queue state; the single-threaded cooperative sweep — `task.rs:82`, `SWEEP_DETERMINISM_STRENGTH` at `dataflow.rs:40`, whose `Exact` is explicitly scoped to the cooperative path with the OS-pool path separately `Empirical`) and `Steal-victim-selection policy determinism` (`StealPolicy::select_victim` is a total pure function of its inputs — `STEAL_POLICY_STRENGTH`; the cross-thread *execution* claims are the separate `Empirical` rows) | **Audited as by-construction** (pure single-threaded functions, no ambient input) and whitelisted in the regression test — any whitelist addition requires re-running this audit, not just editing the list |
| 4 | **The prose `Proven`-modulo-LR-9 tag** (sibling-reclamation safety, `region.rs:48`/`:211`) mirrors RFC-0027 §8's Accepted, maintainer-ratified wording — a *qualified* claim that names its unchecked side-condition and states "no in-repo mechanized proof" | **Retained as qualified prose** — not a bare `Proven`, and rewriting it here would desync the crate from the Accepted RFC. Guard: it must not be promoted into any machine-readable tag above `Empirical` until an in-repo mechanized proof (LR-9 side-condition checked) lands — the no-`Proven` regression test enforces this for the MATRIX. *Observation (non-blocking FLAG):* a stricter VR-5 reading renders it `Declared`-with-argument; that reclassification belongs to an RFC-0027 amendment, not this leaf |
| 5 | **`Declared` tags** — task purity (`TASK_PURITY_STRENGTH`; type system cannot enforce), the rc-probe / `rc==1`-reuse / region-batching **perf** claims (DN-32 §6a: expected, unmeasured) | Correctly `Declared`; retained |
| 6 | **M-963's own rows** land per this rule: 4 × `Exact` (fail-closed / by-construction: setter record, unset-slot refusal, capture resolution, deferred-construct refusal) + 1 × `Empirical` (replay-reaches-recorded-decision — property-tested, not `Proven`) | Consistent; pinned by `m963_rows_present_at_expected_strength` |
| 7 | Inline-test inventory: `colony.rs` + `task.rs` still carry inline `#[cfg(test)]` modules (M-797 lazy sweep) — untouched by this lane, so the as-touched extraction rule was not triggered for them (`guarantee_matrix.rs`, which this lane *did* touch, had its tests extracted) | Recorded, not silently skipped (G2) |

**Downgrades required: none.** No tag was found above its supportable strength.
**Upgrades performed: none** (nothing acquired a machine-checked side-condition).

---

## Meta — changelog

- **2026-07-02 — Created + Accepted under delegation (M-962, kickoff `frz` Lane C, epic E31-1).**
  The M-828 tail split: buildable/directed B-1 (capture/replay), B-2 (reified setter),
  B-3 (residual ledger/refusal) vs research/Phase-II R-1…R-4, R-5ʹ, R-6, P-1…P-3, each with
  its unmet prerequisite and tracker named. RFC-0027/DN-32 memory-model precondition
  **confirmed** (`Empirical`, §2) with two honest residuals recorded. FLAGs F1–F5 raised.
  Grounded in DN-63, DN-70, RFC-0027, DN-32, RFC-0005, research/27 §2, frz Lane C brief.
  Append-only. No shared file touched from this branch.
- **2026-07-02 — M-963 landed + M-964 audit appended (same branch, dated append).** The §3
  subset is built in `mycelium-std-runtime` (`policy_mech` B-1/B-2, `r2_residual` B-3, five
  guarantee-matrix rows, 94 tests green incl. the record-vs-replay proptest differential).
  The M-964 RT2/determinism honesty audit is recorded as the appendix above: no `Proven`
  anywhere in the machine-readable surface (now regression-guarded), all schedule-dependent
  determinism claims `Empirical`, two by-construction `Exact` determinism claims whitelisted,
  the `region.rs` `Proven`-modulo-LR-9 prose retained as qualified (with a non-blocking
  observation FLAG), zero downgrades/upgrades. DoD items 1–2 satisfied; item 3 (FLAG-F2…F5)
  remains the integrator's.
