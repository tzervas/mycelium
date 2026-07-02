# Research Record 28 — Starting the Mesh/Xloc/Cyst Research Track (Plan Only, Not the Survey)

> **What this file is.** The **start** of a dedicated, long-lead research track for the three
> heaviest RFC-0008 R2 distribution constructs — `mesh`, `xloc`, `cyst` — as commissioned by
> M-913 (kickoff `enb`, epic E28-1, Phase-I H1). It is a **plan for a prior-art survey**, a
> **problem statement**, an **open-question ledger**, and a **skeleton for the future design
> note** the survey will back. It is explicitly **not** the survey itself (no primary sources
> fetched, no literature verdicts rendered), and it **implements nothing** — no code, no
> grammar, no active syntax. Posture: **Declared/Draft** throughout (VR-5); every claim here is
> a scoping/planning claim, not a researched result.
>
> **Non-gating (explicit).** This track is a **separate, parallel, long-lead lane**. M-914 (the
> H1 capstone integration demo) does **not** depend on this record or on any of `mesh`/`xloc`/
> `cyst` landing. Nothing in the `enb` kickoff's Definition of Done requires this track to
> progress past this starting note. See §6.

| Field | Value |
|---|---|
| **Record** | research/28 |
| **Backs** | a future design note (skeleton in §5; not yet numbered — see FLAG-O2) |
| **Task** | M-913 |
| **Epic / kickoff** | E28-1 / `enb` |
| **Date** | July 2, 2026 |
| **Status** | Started (this record); the survey itself is **not started** — see §3 |

---

## 1. Problem statement

RFC-0008 §4.5 reserves six R2 (distribution-tier) vocabulary names as **ratified, not yet
lexed**: `xloc`, `mesh`, `cyst`, `graft`, `forage`, `backbone`
(`.claude/memory/lang-lexicon-syntax.md` §Reserved-not-active). DN-63 (Draft, 2026-06-29)
decomposes all six into per-construct typing/elaboration sketches and a dependency ordering
(DN-63 §4). That ordering is the grounding fact this record starts from:

```
forage   (lightest -- pure RFC-0005 policy annotation, no new mechanism)
backbone (consumes forage; needs a mesh transport interface)
    -- mesh   (needs a clock model + a Byzantine-tolerance decision -- the longest lead time)
        -- xloc   (needs mesh as carrier, needs the graft capability decision)
            -- graft  (RFC-0028 section 7 follow-on)
            -- cyst   (needs xloc for mobility, needs the RFC-0027 OQ-3 reclamation decision)
```

`mesh`, `xloc`, and `cyst` are the **three constructs at the bottom of that chain** — DN-63
§8's FLAG-O3 names `mesh` specifically as carrying "the longest research lead time (DN-61 §B.1
clock/R8-Q3 and §B.2 Byzantine/R8-Q4 research passes)" and recommends starting that research
"as a parallel workstream immediately, not waiting for the `forage`/`backbone` RFCs to land
first." `xloc` and `cyst` are downstream of `mesh` in the same chain and each carries its own
unresolved research-grade question (DN-63 §3.1 FLAG-3 — the wire-format-swap story for `xloc`;
§3.3 FLAG-7/FLAG-9 — dormant-value reclamation and continuation representation for `cyst`).
`graft`, `forage`, and `backbone` are comparatively light: `forage` reuses the existing
RFC-0005 policy mechanism outright, `backbone` is a semantics-free transport-hint declaration
(DN-63 §3.6, FLAG-15 already resolved by M-825), and `graft` is scoped mainly by the RFC-0028
§7 follow-on rather than open literature. That is the basis (`Declared`, grounded in DN-63 §4/
§8) for **why this record scopes exactly `mesh`/`xloc`/`cyst`** — the "D-heavy" trio per the
`enb` kickoff's M-913 row (`.claude/kickoffs/enb.md`) — as one research track, separately from
the lighter three.

**The problem, stated plainly:** the `mesh`/`xloc`/`cyst` implementation RFCs cannot be
responsibly drafted from the corpus alone. DN-63 §3.2–§3.3 already sketch a `Declared` typing
and elaboration strategy for each, but every strategy is qualified by an open question that
needs literature grounding before it can be answered honestly (VR-5 forbids asserting a
`Proven`/`Empirical` bound from an unchecked sketch). RFC-0008 R2 is gated on R1 completion
(M-667) regardless (DN-63 §7: "No R2 implementation RFC should advance to Accepted before
M-667 is done"), and R1 is **not** done at the time of this record. So the research track has
zero implementation urgency today — but it has non-zero **lead-time** urgency: gossip-protocol
selection (mesh), clock-model selection (mesh), and durable-execution/continuation-format
precedent (cyst) are each multi-source literature questions that take real calendar time to do
honestly, independent of when R1 finishes. Starting the survey *plan* now — without touching
the R1 critical path — is the way to bank that lead time (M-913's user story, `enb` kickoff:
"As the Phase-II runtime effort, I want the distributed-vocabulary research started now, so
that its long lead time overlaps Phase I instead of following it").

---

## 2. Scope

**In scope for this track:** `mesh` (RT5 gossip/pub-sub overlay), `xloc` (RT1/RT4 fallible
cross-node value movement), `cyst` (RT2/§4.4 content-addressed dormable-computation
checkpoint).

**Out of scope for this track** (tracked by DN-63 but not part of M-913): `graft`, `forage`,
`backbone`. Nothing here should be read as blocking or reordering those three; DN-63 §4's
"natural launch order" already sequences `forage`/`backbone` ahead of `mesh` for the
*implementation-RFC* track, which is unaffected by this *research* track running in parallel.

**Out of scope for this track (harder boundary):** anything that would constitute *doing* the
research (fetching a primary source, rendering a literature verdict), drafting grammar, writing
an elaboration strategy beyond what DN-63 §3.2/§3.3 already sketches, or touching R1
(`hypha`/`colony`/`fuse`/`reclaim`/`tier`) implementation work. Those stay untouched by this
record.

---

## 3. Prior-art survey plan (the plan, not the survey)

This section names **what each future survey pass must cover and why**, following the method
precedent already established in this repo's `research/` corpus (`research/17`, `19`, `22`,
`24`: primary sources fetched and text-extracted, verdicts checked against the actual
theorem/mechanism rather than a secondary summary, tagged on the `Exact ⊐ Proven ⊐ Empirical ⊐
Declared` lattice — never a `Proven` claim without the side-conditions checked). **No source
below has been fetched or verified for this record.** Each bullet is a planned search
direction, not a finding.

### 3.1 `mesh` — the longest-lead pass (DN-63 §8 FLAG-O3)

Two sub-questions, each independently research-grade (DN-61 Part B.1/B.2):

- **Gossip/pub-sub protocol choice (DN-63 §3.2 FLAG-4).** Planned literature: Demers et al.
  (epidemic broadcast — already cited at T4.2 per DN-63 §5); HyParView (partial-view
  membership) and Plumtree (broadcast tree over HyParView); libp2p gossipsub (the
  production-deployed variant closest to a real mesh v0 target). The survey pass must extract
  each protocol's actual delivery-probability theorem (not a blog-post restatement) and its
  stated assumptions (network model, churn rate, message loss rate) so DN-63 §5's honest-tag
  requirement ("a generic 'probabilistic' tag with no committed protocol is `Declared` at
  best") can be discharged for a specifically named protocol.
- **Clock model (DN-61 §B.1 / R8-Q3).** Planned literature: Lamport logical clocks (the
  baseline already used at R1 per M-356 C4); Hybrid Logical Clocks (Kulkarni et al.) for
  causal ordering with a physical-time bound; Google TrueTime/Spanner (for the
  synchronized-uncertainty-interval design pattern, as a comparison point — not necessarily
  adoptable given Mycelium has no dedicated time-sync infrastructure). The survey pass must
  determine what precision guarantee is honestly claimable *without* deployment-specific NTP
  measurement (DN-61 §B.1 already flags physical clocks as `Empirical`, not `Exact`).
- **Byzantine tolerance (DN-61 §B.2 / R8-Q4).** Planned literature: the BFT-CRDT trade-off
  literature DN-61 names but does not cite by title; PBFT and successors for the
  consensus-cost baseline RFC-0008 §6 already positions semilattice merge *against*. The
  survey pass must answer DN-61's stated question — "is the v0 mesh abstraction a viable
  target for BFT or does it require a separate mesh variant" — with named sources, not
  restated as an open question a second time.

### 3.2 `xloc` — cross-node value movement

- **Capability-based distributed references.** Planned literature: the E language's
  distributed object-capability model (promise pipelining, no ambient authority — directly
  relevant to DN-63 §3.4's `graft`/`xloc` capability-composition question, FLAG-1/FLAG-11);
  Cap'n Proto RPC's capability-passing wire protocol (a concrete, implemented precedent for
  "capability travels with the reference, checked at the remote end").
- **Explicit-failure RPC/mobility models.** Planned literature: Emerald (mobile objects with
  explicit location and explicit failure — an early, directly on-point precedent for RT4's
  "local and remote are different types"); Erlang's `rpc`/distributed-Erlang failure model
  (explicit `{badrpc, Reason}` — a working never-silent precedent to compare against DN-63
  §3.1's `XlocError` variant sketch).
- **Wire-format/serialization-with-provenance.** Planned literature: any published treatment
  of "serialize with the type's canonical wire form, explicit conversion otherwise" — the
  survey pass should determine whether existing systems (e.g. Cap'n Proto's canonical
  encoding, protobuf's deterministic-serialization mode) offer a precedent for DN-63 §3.1
  FLAG-3 (does every repr have a canonical wire form, or does `xloc` need an explicit `Swap`
  first).

### 3.3 `cyst` — content-addressed dormable-computation checkpoint

- **Durable execution.** Planned literature: Temporal and Azure Durable Functions (already
  named at T4.4 per DN-63 §3.3 — the survey pass must go to primary documentation/papers, not
  the DN-63 paraphrase, and extract the actual determinism-of-replay requirement each system
  enforces and how it's checked, since that is the direct analogue of Mycelium's RT2
  dormability gate).
- **Checkpoint/restart and distributed snapshots.** Planned literature: CRIU (process
  checkpoint/restart — a concrete continuation-serialization precedent, relevant to DN-63 §3.3
  FLAG-9's L0-bytecode-vs-higher-level-IR question); the Chandy–Lamport distributed snapshot
  algorithm (for the *distributed* case — a `cyst` that spans a computation touching more than
  one node, not yet addressed by DN-63 §3.3 at all, which is scoped to a single dormable
  computation).
- **Content-addressed continuations.** Planned literature: this repo's own `research/16`/`17`
  (env-machine reclamation, RC + reuse under content-addressing) is itself a relevant internal
  precedent for how a continuation's value heap interacts with reclamation while dormant — the
  survey pass should treat `research/16`/`17` as a starting citation, not re-derive it, and
  extend outward only for the genuinely new questions (distributed reclamation of a dormant
  cyst — DN-61 §B.3 already scopes this as its own research item, cross-referenced not
  duplicated here).

### 3.4 Cross-cutting: distributed reclamation (DN-61 §B.3)

Not a fourth construct, but a cross-cutting concern touching all three: DN-61 §B.3 already
names the planned literature (weighted reference counting — Bevan 1987, Watson & Watson 1987;
CRDT-tombstone GC — Shapiro et al.) for the case where a value's reclaiming scope moves off-node
via `xloc`, or a `cyst`'s value heap must be reclaimed while dormant. This record does not
duplicate DN-61 §B.3's plan; it flags that the `mesh`/`xloc`/`cyst` survey passes (§3.1–§3.3
above) must consume DN-61 §B.3's outcome rather than re-derive a reclamation story
independently.

### 3.5 Method for the actual survey passes (not run here)

Each future survey pass (one per construct, or one per sub-question where a construct has more
than one — `mesh` in particular has three: protocol, clock, Byzantine) should follow the
already-established `research/` method: fetch primary sources, text-extract, quote/paraphrase
the actual theorem or mechanism (not a secondary summary), adversarially check what is proven
vs. asserted vs. left as future work by the source itself, and tag every claim on the
`Exact ⊐ Proven ⊐ Empirical ⊐ Declared` lattice with the checked basis named. `research/17`
§Method is the concrete precedent to follow.

---

## 4. Open questions (research-scoping FLAGs)

These are scoping questions **about the research track itself** — distinct from DN-63's
construct-level FLAGs (FLAG-1 through FLAG-16) and DN-61's Part-B agenda items, which already
exist and are not restated here beyond the cross-references above.

- **RQ-1 — One integrated pass or three independent passes?** `mesh` sits upstream of `xloc`
  and `cyst` in DN-63 §4's dependency chain (mesh → xloc → cyst). Should the survey run
  `mesh` to completion before starting `xloc`/`cyst`, or can `xloc`/`cyst` prior-art research
  proceed in parallel since the *literature* for each is largely independent even though the
  *design* is sequential? `Declared` open; the orchestrator/maintainer should decide when
  scheduling the actual survey work.
- **RQ-2 — Who/what model tier does the survey work?** The `research/` corpus precedent
  (`research/17`, `19`, `22`, `24`) is deep, adversarial, multi-source work — closer to the
  Opus-reasoner tier named in the Fractal Swarm System's "fractured Opus reasoners for
  research" swarm-pattern note than a Sonnet/Haiku leaf task. `Declared` open; a future
  kickoff commissioning the actual survey should decide model tier per the swarm-mode table.
- **RQ-3 — Does the non-gating status need a standing marker beyond this record?** This
  record states non-gating status in its header and §6, but nothing in `issues.yaml` or the
  `enb` kickoff enforces it mechanically (there is no dependency edge to check, because M-913
  has none by design). `Declared` open — flagged to the integrator (FLAG-O3 below); a
  `depends_on: []` on M-913 and any survey-continuation issues it spawns is the existing
  mechanism and should stay that way (never add a spurious dependency edge that would make
  H1's DoD depend on this track).
- **RQ-4 — Should the Byzantine-tolerance sub-question (§3.1) be split into its own record
  given DN-61 §B.2's note that it "may require a mesh *variant* … rather than a flag on the
  existing `mesh` construct" (DN-61 FLAG-2)?** `Declared` open; deferred to whoever scopes the
  first actual `mesh` survey pass.

---

## 5. Skeleton for the future design note (not filled in)

The eventual design note this track produces should follow the DN-35/DN-36/DN-37 pattern
(research-backed Draft advisory notes, later Accepted as *direction*, never as ratified
grammar) rather than DN-63's decomposition-note pattern, because its job is to turn survey
findings into a `mesh`/`xloc`/`cyst`-specific design direction, not to re-decompose what DN-63
already decomposed. Headers only — every section below is empty pending the actual survey
passes in §3:

```markdown
# Design Note DN-XX -- Mesh/Xloc/Cyst Distributed Runtime: Research-Backed Design Direction

| Field | Value |
|---|---|
| Note | DN-XX (number TBD -- FLAG-O2) |
| Status | Draft |
| Feeds | RFC-0008 4.5/4.6 (R2 activation); DN-63 (per-construct decomposition this
          note refines); DN-61 Part B.1/B.2/B.3 (the research agenda this note closes) |
| Task | (the future M-id that commissions the actual survey passes) |

## 1. Scope and relationship to DN-63
(How this note's findings refine, not replace, DN-63 3.2/3.3's typing/elaboration sketches.)

## 2. `mesh` -- protocol, clock, and Byzantine findings
### 2.1 Gossip/pub-sub protocol verdict
### 2.2 Clock model verdict
### 2.3 Byzantine-tolerance verdict
(each backed by its own research/NN-...-RECORD.md, cited not restated)

## 3. `xloc` -- capability composition and wire-format findings

## 4. `cyst` -- dormability, continuation representation, and distributed-reclamation findings

## 5. Revised dependency ordering and honest guarantee-tag inventory
(DN-63 4/5 updated in light of research findings -- append-only relative to DN-63, or a
superseding note per house rule #3, whichever the maintainer directs)

## 6. Remaining open questions for the implementation RFCs

## Definition of Done
```

---

## 6. Non-gating relationship to Phase I (explicit statement)

- **M-913 has `depends_on: []`** in `tools/github/issues.yaml` — it blocks nothing and is
  blocked by nothing in the `enb` kickoff's Gap A–E lanes, `myc run`, string literals, or
  `hash.*`.
- **M-914 (the H1 capstone) does not depend on M-913.** The `enb` kickoff's ordering table
  lists M-913 in the "(early, parallel)" row specifically because it runs alongside the serial
  prim lane without gating it.
- **RFC-0008 R2 stays gated on R1 completion (M-667)** regardless of this track's progress
  (DN-63 §7). Nothing in this record changes, advances, or bypasses that gate. This record
  produces **no implementation RFC**, **no grammar change**, **no active syntax** — `mesh`,
  `xloc`, `cyst` remain reserved-not-active per the lexicon note.
- **This record's own Definition of Done (§below) is satisfied by the *existence* of the plan**,
  not by any research finding — the finding work is future, separately-commissioned work.

---

## 7. FLAGs

- **FLAG-O1 (orchestrator-owned files).** This record does not touch `CHANGELOG.md`,
  `docs/Doc-Index.md`, or `tools/github/issues.yaml` — those are orchestrator-owned per
  CLAUDE.md swarm discipline. The integrating parent should register this record (a
  `Doc-Index.md` row analogous to the `research/26`/`27` rows) and close M-913 with a
  `landed_basis` note pointing here.
- **FLAG-O2 (DN number).** §5's skeleton names the future design note `DN-XX` — no number is
  reserved here. The integrator/orchestrator must mint the next free `DN-` number at the time
  the actual survey work is commissioned (not now — minting early risks a collision with
  unrelated work landing in between, per swarm mitigation #1).
- **FLAG-O3 (non-gating enforcement).** RQ-3 (§4) notes there is no mechanical enforcement of
  non-gating status beyond the absence of a dependency edge. The integrator should confirm
  M-913's `depends_on: []` stays that way when closing this task, and should not let any
  future survey-continuation issue acquire a dependency edge from H1-critical-path work.
- **FLAG-O4 (survey commissioning).** RQ-1/RQ-2 (§4) are scheduling/staffing questions for
  whoever commissions the actual `research/29`+ survey pass(es). Not resolved here — flagged
  forward.

---

## Definition of Done

This record (research/28) is **done** when:

1. The problem statement (§1) and scope (§2) are grounded in DN-63/DN-61/the lexicon note
   (done — see citations throughout).
2. A prior-art survey **plan** (§3) exists per construct, naming candidate primary sources and
   the method, without executing the survey (done — no source fetched or verified here).
3. Open questions (§4) and a design-note skeleton (§5) exist for the future work to pick up.
4. Non-gating status for Phase I is stated explicitly (§6) and FLAGs are surfaced for the
   integrator (§7).

The **track** (as opposed to this starting record) is done only when a future design note
following the §5 skeleton reaches at least Draft, backed by actual `research/29`+ survey
passes — out of scope for M-913.

---

## Meta — changelog

| Date | Change |
|---|---|
| 2026-07-02 | Created (M-913, kickoff `enb`, epic E28-1). Starts the mesh/xloc/cyst research track: problem statement (§1, grounded in DN-63 §4/§8 FLAG-O3 and the dependency chain), scope (§2, `mesh`/`xloc`/`cyst` in vs. `graft`/`forage`/`backbone` out), a per-construct prior-art survey **plan** (§3 — sources and method named, no survey executed), four scoping open questions (§4, RQ-1..RQ-4), a headers-only skeleton for the future research-backed design note (§5), an explicit non-gating statement for Phase I (§6), and four FLAGs for the integrating parent (§7, FLAG-O1..FLAG-O4). No shared file touched (CHANGELOG/Doc-Index/issues.yaml — orchestrator-owned, FLAG-O1). Implements nothing; posture `Declared`/Draft throughout. |
