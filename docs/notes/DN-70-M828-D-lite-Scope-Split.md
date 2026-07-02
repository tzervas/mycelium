# Design Note DN-70 — M-828 D-lite Scope Split: the R2 Usability Subset (H1) vs the Maturity Remainder (H2)

| Field | Value |
|---|---|
| **Note** | DN-70 |
| **Status** | **Accepted** (2026-07-02 — **maintainer sign-off recorded**: the D-lite split is approved. Drafted 2026-07-02 as **Draft**, split proposed pending sign-off; that history stands unchanged — this line records the forward transition, append-only, house rule #3. FLAG-D1 is cleared — see the dated sign-off note below and §Definition of Done. M-906/M-907 unblock) |
| **Decides** | *Proposes, does not ratify:* the scope split of **M-828** into the **D-lite usability subset** (H1 — `forage` activation + `backbone` verification) and the **H2-deferred remainder** (the other four R2 constructs + the mechanized `SelectionPolicy` capture-&-set surface). Each row is grounded in DN-63; the maintainer's signature is the decision (kickoff `enb`, M-905 gate) |
| **Feeds** | M-828 (its body carries the split once the integrator applies FLAG-D5's text); M-906 (`forage` activation — scope fixed here); M-907 (`backbone` verification — scope fixed here); DN-63 (per-construct decomposition — unchanged, cited row-by-row); RFC-0008 §4.5 (the activation status rule — see FLAG-D2); ADR-038 Phase I / roadmap §3 (the H1 D-lite row) |
| **Task** | M-905 (epic E28-1, kickoff `enb`) |
| **Date** | July 2, 2026 |

> **Posture (transparency rule / VR-5 / G2).** This is a **scope-split memo** — a planning
> decision proposal, not a design ratification. Every claim is tagged: the in-repo state
> assertions are `Empirical` (verified against the working tree at `origin/dev`
> `de11922`, 2026-07-02, file:line cited); the split itself and all forward-looking scope
> statements are `Declared` (a proposed decision, decided only by the maintainer's sign-off).
> No DN-63 open question is resolved here by a plausible-sounding answer; where the subset
> *narrows* a FLAG, that narrowing is stated explicitly and the full FLAG stays open for the
> H2 vehicle. **Append-only:** the sign-off (or its refusal/amendment) lands as a dated
> section below, not as a rewrite.
>
> **Maintainer sign-off (append-only, 2026-07-02).** FLAG-D1 is cleared: the maintainer signs the
> D-lite split — §3's in-scope rows (D1 `forage` single-node activation, D2 `backbone`
> verification) as M-906/M-907's scope, and §5's deferred rows R-1…R-6 as the M-828 H2 remainder.
> The gate that held M-906/M-907 at `status:blocked` (§1 "The gate") is now cleared; both move to
> `status:todo` (tools/github/issues.yaml). FLAG-D2 (the RFC vehicle question), FLAG-D3 (wording
> correction), FLAG-D4 (DN-slot verification), and FLAG-D5 (M-828 body text) are unaffected by this
> sign-off and remain for the integrator per their original disposition.

---

## 1. Context and gate

**M-828** (DN-64 OQ-B disposition) is the *full* R2 runtime-vocabulary maturity wave:
`forage` and `backbone` "must be made active", **plus** a mechanized `SelectionPolicy`
capture-and-set surface (record/replay, a setter surface) "to improve ergonomics while
retaining transparency, provenance and explainability" (M-828 body,
`tools/github/issues.yaml`; research/27-dn64-ergonomics-rnd-RECORD.md).

**ADR-038 Phase I / roadmap §3** (`docs/planning/road-to-1.0.0-and-mycelium-rewrite.md`,
H1 table, row **D-lite**) bounds what lands in H1: *"The subset of
`forage`/`backbone`/`xloc`/`mesh`/`cyst`/`graft` (DN-63) that `std.runtime`'s **usable
surface** needs — activated per DN-63; the full R2 maturity wave stays H2/Phase II"*, with
the explicit instruction *"scope the split at kickoff — FLAG"*. This memo is that split
(kickoff `enb`, task M-905).

**The gate:** M-906 (`forage` activation) and M-907 (`backbone` verification) are
`status:blocked` on the maintainer signing this split (issues.yaml M-906/M-907 bodies;
kickoff `enb` §Prerequisites item 3). The sign-off is FLAG-D1 below.

**The R1 gate is satisfied** (`Empirical`): DN-63 §7 gates all R2 activation on M-667;
M-667 is `status:done` (issues.yaml). The structural precondition for activating *any*
R2 construct is met — which is exactly why the split must now say *which ones*.

---

## 2. What "`std.runtime`'s usable surface" means here — grounded

The H1 Definition of Done (roadmap §3) requires *"a Mycelium program exercising … the
R2-lite runtime surface"* to **run** via `myc run` — i.e. the usable surface is the set of
R2 constructs a runtime-vocabulary program can *execute rather than refuse* in Phase I's
single-node interpreter reality (M-906 user story, issues.yaml).

**Current in-repo state (`Empirical`, verified 2026-07-02 at `de11922`):**

- `forage` and `backbone` **lex** as keywords (`crates/mycelium-l1/src/token.rs:369–370`)
  and are **rejected at parse** with a teaching diagnostic — "reserved for the runtime
  model (RFC-0008), not yet active" — at both item and expression position
  (`crates/mycelium-l1/src/parse.rs:517–518, 1735`). Reserved-not-active, never-silent (G2).
- No executing `forage` or `backbone` construct exists anywhere in the tree: the only
  occurrences of `Backbone` outside tests/diagnostics are the token variant and its
  spelling-table entry (`token.rs:59`, `parse.rs:2337–2338`); `mycelium-std-runtime`
  documents the whole §4.5 six as "reserved, not yet activated"
  (`crates/mycelium-std-runtime/src/lib.rs:39`), and its guarantee-matrix test *forbids*
  the reserved vocabulary in v0 public op names
  (`crates/mycelium-std-runtime/src/guarantee_matrix.rs:205`).
- The **RFC-0005 mechanism `forage` needs already exists**: `mycelium-select` ships the
  content-addressed, EXPLAIN-mandatory `SelectionPolicy` decision table
  (`crates/mycelium-select/src/lib.rs:424`); DN-63 §3.5 defines `forage` as *"not a new
  mechanism, but the third application of the existing one"*.
- The single-node scheduler already **mirrors the `forage` posture** without depending on
  it: `mycelium-sched`'s `StealPolicy` is a total, deterministic, EXPLAIN-able placement
  decision procedure, documented as "mirroring the reserved `forage` construct's EXPLAIN
  posture" (`crates/mycelium-sched/src/scheduler.rs:178–180`) — the RT3 pattern the D-lite
  `forage` binds to surface syntax.
- `backbone`'s **decision** landed with M-825 (2026-06-29): DN-63 FLAG-15 resolved,
  `backbone` = **runtime-dynamic promoted**, recorded append-only in RFC-0008 §4.5 and
  DN-63 §3.6. What landed is the *decision record* — see §4/FLAG-D3 for the honest
  inventory framing M-907 must verify against.

**Therefore** (`Declared`, grounded in the DN-63 §4 dependency table): `forage` and
`backbone` are the **only two** R2 constructs whose prerequisites are met inside H1 —
`forage` has *"none"* in DN-63 §4's blocker column (R1 done; RFC-0005 exists), and
`backbone`'s framing decision is already made (M-825). The other four each carry an
unmet research or RFC prerequisite (§5 below) that cannot be satisfied inside H1 without
guessing (G2/VR-5). The roadmap's naming of exactly `forage`/`backbone` for D-lite is
consistent with DN-63's own launch order (§4: *"`forage` RFC first … `backbone` in
parallel or immediately after"*).

---

## 3. In-scope rows — the D-lite subset (H1)

All scope statements `Declared` (proposed; decided by sign-off). Each row cites its DN-63
basis.

### D1 — `forage`: single-node activation (task M-906)

| Aspect | D-lite scope | DN-63 basis |
|---|---|---|
| Surface | `@forage(policy)` annotation on `hypha`/`colony`; lex (already done) → parse → elab → interp | §3.5 typing strategy (`hypha @forage(policy: PlacementPolicy) { … }`) |
| Mechanism | The existing RFC-0005 `SelectionPolicy` (`mycelium-select`) — **no new mechanism** (KC-3); the policy is content-addressed, total, deterministic, mandatory-EXPLAIN | §3.5: "the third application of the existing one"; RFC-0005 §2/§3 |
| Candidate set | The **single-node** worker/execution set (degenerate but real — the same shape `mycelium-sched::StealPolicy` already decides over). Multi-node placement is an **explicit residual** (H2), tracked, never silent | §3.5 elaboration strategy (scheduler consults the active policy); dependency note: the multi-node "available node set comes from the mesh overlay" — `mesh` is H2 |
| FLAG-13 (Meta signals) | **Narrowed for the subset, not resolved**: D-lite fixes a minimal signal set (worker occupancy / local scheduler signals, mirroring `StealPolicy`'s inputs). The full node-level signal inventory stays FLAG-13, owned by the H2 `forage` maturity work | §3.5 FLAG-13 (`Declared` open — stays open) |
| FLAG-14 (no candidates) | **Adopted for the subset as DN-63 already specifies**: empty/failing candidate set ⇒ `ForageError::NoCandidates`, a typed explicit error, never a silent hang (RT4) | §3.5 FLAG-14 (the required shape is stated there; D-lite implements exactly it) |
| Conformance obligation | The RT2 placement differential: two placements of the same deterministic computation ⇒ same `TaskOutcome`/transcripts. Tag `Declared` until the differential test exists, then `Empirical` — never `Proven` without a mechanized theorem (VR-5) | §3.5 semantics-free paragraph; §5 tag table (`forage` semantics-free row) |
| Never-silent | Deferred parts (multi-node, full signal set) refuse or record an explicit residual + tracker entry — no silent accept (G2) | M-906 DoD (issues.yaml); RFC-0008 RT3/RT4 |

### D2 — `backbone`: verify the landed state, close residual gaps only (task M-907)

| Aspect | D-lite scope | DN-63 basis |
|---|---|---|
| Primary deliverable | A **verified inventory** (`Empirical`) of what M-825 actually landed vs this subset — *verification row, not an activation row*. Don't re-land | M-907 DoD (issues.yaml); §3.6 FLAG-15 resolution note |
| Baseline expectation for that inventory | Per §2 above (`Empirical`, 2026-07-02): the landed state is the **decision** (runtime-dynamic promoted; RFC-0008 §4.5 append + DN-63 FLAG-15 resolved) + the reserved keyword + teaching diagnostics — **no executing construct**. M-907 re-verifies this with evidence at its own date and records deltas | §3.6; M-825 `landed_basis` (issues.yaml) |
| Residual-gap closing (bounded) | At most: the minimal `BackboneRef` input surface that D1's *single-node* policy consumes — and on a single node a declared transport path has no observable distinct from the default, so the expected D-lite outcome is **"recorded residual, nothing to build"** unless M-907's inventory finds otherwise. Fallback semantics, when built, must be `BackboneError::Unavailable` — explicit, never a silent hang (RT4) | §3.6 (backbone "is an input to `forage` policies"; fallback shape); §4 (backbone's blockers include `mesh` — H2) |
| FLAG-16 (`BackboneRef` affinity) | **Stays open** — not needed for the verification row; owned by the H2 backbone implementation RFC | §3.6 FLAG-16 (`Declared` open) |

### Why this subset keeps H1 bounded and honest

- **Bounded:** D-lite adds exactly one activation (D1, riding an existing, tested mechanism)
  plus one evidence-gathering verification (D2). No new kernel mechanism, no research
  dependency, no distribution machinery. The serial-lane cost is one L1 surgery (M-906),
  already sequenced by the kickoff's collision law.
- **Honest:** every deferred piece is *named* in §5 with its unmet prerequisite — nothing is
  silently dropped (G2), and nothing lands whose prerequisites would have to be guessed
  (VR-5). The subset never upgrades a DN-63 `Declared` tag: the placement differential
  earns `Empirical` only when written; FLAG-13/FLAG-16 remain open where DN-63 left them.

---

## 4. The M-825 "landed construct" wording — corrected honestly (`Empirical`)

The kickoff `enb` M-907 row (and the minted M-907 title) say M-825 *"landed a backbone
construct 2026-06-29"*. Verified against the tree and the issue record (§2): **M-825 landed
the backbone *decision*** (FLAG-15 → runtime-dynamic promoted; append-only notes in
RFC-0008 §4.5 and DN-63 §3.6; M-825 `landed_basis`: "Maintainer decision recorded … The
future backbone implementation RFC proceeds on this basis"), **not an executing construct**.
The M-907 *task framing* ("verify, don't re-land") is exactly right; only the word
"construct" overstates. This is FLAG-D3 — a wording correction for the integrator, and the
honest baseline M-907's inventory starts from.

---

## 5. Deferred rows — the H2 remainder (stays on M-828)

Each row `Declared`, grounded in DN-63's own prerequisite analysis. None of these can land
in H1 without either guessing past an open research question or bypassing a stated gate.

| # | Deferred item | Why deferred — the unmet prerequisite | DN-63 basis |
|---|---|---|---|
| R-1 | `mesh` activation | Needs the DN-61 B.1 (clock) + B.2 (Byzantine) research passes — the longest-lead R2 items; also the v0 gossip-protocol choice (FLAG-4) and the `ProbabilityBound` commitment (§5 special note). The `enb` kickoff starts this research as a separate, explicitly non-gating lane (M-913) | §3.2; §4 (launch order 3); §5 |
| R-2 | `graft` activation | Needs the RFC-0028 §7 capability follow-on (FLAG-10/11/12 open) | §3.4; §4 (launch order 4) |
| R-3 | `xloc` activation | Needs `mesh` (carrier) + `graft` (capability check) at least Accepted; wire-format swap story open (FLAG-1/2/3) | §3.1; §4 (launch order 5) |
| R-4 | `cyst` activation | Needs `xloc` (mobility) + the RFC-0027 OQ-3 reclamation-in-dormancy resolution (FLAG-7/8/9) | §3.3; §4 (launch order 6) |
| R-5 | Mechanized `SelectionPolicy` **capture & setting** (record/replay + setter surface) — the second half of M-828's DoD | Ergonomics, not usability: no H1 DoD row needs it (roadmap §3), and its design record (research/27-dn64-ergonomics-rnd-RECORD.md) is an R&D track, not a ready spec. Deferring it is precisely what keeps D-lite "lite" | M-828 body (DN-64 OQ-B); roadmap §3 H1 DoD |
| R-6 | `forage`/`backbone` **maturity**: multi-node candidate sets + full FLAG-13 signal inventory; real transport paths, the backbone promotion mechanism + `BackboneRef` lifecycle, FLAG-16 affinity | The multi-node reality depends on `mesh` (R-1); the promotion mechanism is explicitly assigned to the backbone implementation RFC by the M-825 resolution note | §3.5 dependency note; §3.6 FLAG-15 resolution + FLAG-16 |

**The H2 remainder stays tracked on M-828** (which stays open); the D-lite subset is
tracked on M-906/M-907 under epic E28-1. FLAG-D5 gives the integrator the exact
append-only body text recording this on M-828.

---

## 6. Honest-tag summary for the D-lite rows (VR-5)

| Claim | Tag | Basis / gap |
|---|---|---|
| Reserved-not-active current state of `forage`/`backbone`; no executing construct in-tree | `Empirical` | Verified 2026-07-02 at `de11922`; file:line in §2 |
| M-667 (R1) done ⇒ DN-63 §7 gate satisfied | `Empirical` | issues.yaml M-667 `status:done` |
| `forage`/`backbone` are the only two R2 constructs with met prerequisites in H1 | `Declared` | DN-63 §4 blocker table (itself `Declared`); no formal dependency proof |
| The split itself (in-scope §3 vs deferred §5) | `Declared` → decided on maintainer sign-off | FLAG-D1; roadmap §3's "scope the split at kickoff — FLAG" |
| D-lite `forage` semantics-free placement | `Declared`; `Empirical` once the RT2 placement differential test exists | DN-63 §5 tag table; never `Proven` without a mechanized theorem |
| D-lite `backbone` expected outcome ("recorded residual, nothing to build" on one node) | `Declared` — a prediction M-907's inventory checks, not a pre-judgment of it | §3 D2 row |

---

## 7. FLAGS

| FLAG | What it is | Who decides / applies |
|---|---|---|
| **FLAG-D1** | **Maintainer sign-off on the split** (§3 in-scope vs §5 deferred) — the M-905 gate. M-906/M-907 stay `status:blocked` until signed. Sign-off (or amendment) is recorded as a dated append-only section here | **Maintainer** — **CLEARED 2026-07-02**, see the sign-off note above. |
| **FLAG-D2** | **RFC vehicle for the D-lite `forage` activation.** RFC-0008 §4.5's status rule is *normative*: activation "requires an implementation RFC committing each construct's typing and elaboration per RFC-0006 §4.3"; DN-63 §6 recommends a per-construct `RFC-00XX-forage`. Options: (a) M-906 ships a scoped `forage` implementation RFC (subset-sized — the D-lite scope in §3 D1 is most of its content) alongside the code — **recommended**, keeps §4.5 intact; (b) the maintainer explicitly decides the subset activation may land with the RFC following. Not guessed here (G2) | **Maintainer** (with FLAG-D1) |
| **FLAG-D3** | Kickoff/M-907 wording: "M-825 landed a backbone construct" → should read "M-825 landed the backbone **decision** (runtime-dynamic promoted)". §4 above carries the verified basis. Integrator corrects the M-907 body wording at integration tier (issues.yaml is orchestrator-owned) | Integrator |
| **FLAG-D4** | DN number: **DN-70** minted here (DN-69 was taken by the scalar-float dossier landed 2026-07-02; verified free at `de11922`). Integrator re-verifies the slot at merge (mitigation #1 analogue) and registers the Doc-Index row | Integrator |
| **FLAG-D5** | **M-828 body update** (orchestrator-owned — NOT applied from this branch). Recommended append-only addition to the M-828 `body:` in `tools/github/issues.yaml`: see the block below | Integrator |

**FLAG-D5 recommended M-828 body-update text** (append to the existing body, verbatim):

```text
D-lite scope split (2026-07-02, DN-70; maintainer sign-off tracked as M-905/FLAG-D1):
H1 lands only the usability subset, under epic E28-1 —
- forage activated single-node (M-906): @forage policy annotation via the existing
  RFC-0005 SelectionPolicy machinery (mycelium-select); explicit
  ForageError::NoCandidates (DN-63 FLAG-14 shape); mandatory EXPLAIN trail; RT2
  placement differential (Empirical when written). Multi-node placement is an
  explicit residual (H2).
- backbone verified against the landed M-825 decision (M-907): inventory recorded
  Empirical; residual gaps closed or FLAGged, no re-land; DN-63 FLAG-16 stays open.
This issue keeps tracking the H2 remainder: mesh/graft/xloc/cyst activation (DN-63
S4 order, each gated on its stated research/RFC prerequisite) and the mechanized
SelectionPolicy capture-and-set surface (DN-64 OQ-B ergonomics half). The remainder
is NOT implied by the D-lite landing; it advances only through its own vehicles.
```

**Also FLAGged to the integrator** (files not touched from this branch, per ownership):
the `CHANGELOG.md` entry, the `docs/Doc-Index.md` row for DN-70, and (if adopted) a
`doc_refs: corpus:DN-70` addition on M-905/M-906/M-907.

---

## Definition of Done

This DN is **Accepted** when:

1. The maintainer signs the split (FLAG-D1) — confirming §3 as M-906/M-907's scope and §5
   as the M-828 remainder — and disposes FLAG-D2 (the RFC vehicle question).
2. The integrator applies FLAG-D3/D4/D5 (wording fix, index registration, M-828 body).

**Item 1 is satisfied 2026-07-02** (the maintainer sign-off note above) for the split
confirmation; **FLAG-D2 is not separately disposed** in this sign-off and stays open for the
integrator/maintainer to resolve before M-906 ships its `forage` implementation RFC. Item 2
remains the integrator's, per ownership (append-only — not applied from this branch).

This DN is **Resolved** when:

- M-906 and M-907 are done per their DoDs, and the M-828 body carries the split (so the H2
  remainder is unambiguously tracked).

---

## Meta — changelog

- **2026-07-02 — Draft created (M-905, kickoff `enb`, epic E28-1).** The M-828 D-lite scope
  split: in-scope rows D1 (`forage` single-node activation, M-906) + D2 (`backbone`
  verification, M-907), each grounded in DN-63 §3.5/§3.6/§4; deferred rows R-1…R-6 (H2,
  staying on M-828), each with its unmet prerequisite named. Current-state inventory
  `Empirical` (verified at `de11922`); split `Declared`, gated on maintainer sign-off
  (FLAG-D1). FLAGs D1–D5 raised (sign-off; RFC vehicle; M-825 wording correction; DN-slot
  verification; M-828 body text). Advisory; append-only. No shared file touched.
- **2026-07-02 — Accepted (maintainer sign-off recorded).** FLAG-D1 cleared: the D-lite split
  (§3 in-scope vs §5 deferred) is approved as drafted. Unblocks M-906/M-907
  (tools/github/issues.yaml status:blocked → status:todo). FLAG-D2 (RFC vehicle) not separately
  disposed — stays open; FLAG-D3/D4/D5 unaffected, remain integrator-owned. Append-only; VR-5; G2.
