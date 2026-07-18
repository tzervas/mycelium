# Design Note DN-141 — Tags, Meta & Containment: the Honesty-Poison Wall Model

| Field | Value |
|---|---|
| **Note** | DN-141 |
| **Status** | **Draft.** Body **ratified-in-content by maintainer steer 2026-07-17** (the P2-Q1..Q5 steer register, `docs/planning/design-steer-2026-07-17/PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.2), **awaiting formal ratification date**. Per house rule #3 / prime directive **H2** ("never claim `Accepted` without maintainer ratification date"), this note stays **Draft** until a maintainer records that date; it does **not** self-ratify. Per prime directive **H1**, the `DN-141`/`DN-142` slots were checked free in `docs/notes/` before this file was minted (both absent, verified 2026-07-18). |
| **Free-ID check** | `docs/notes/DN-141-*.md` and `docs/notes/DN-142-*.md`: **both absent** in the repository at capture time (glob-verified, 2026-07-18). No collision. |
| **Decides** | The containment/wall model for the guarantee lattice (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`): how meet-composition contamination stops at named boundaries instead of laundering silently or forcing a global `Declared` floor — ratifying pack 02's design (`docs/planning/gap-analysis-2026-07-16/DESIGN-02-TAGS-META-AND-CONTAINMENT.md`) as this note's normative body, with the five 2026-07-17 steers (§8 Q1–Q5 there) folded in. |
| **Grounds on** | `DESIGN-02` (source pack — kept unedited; this note distills it, see §1 below) · `PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.2 (P2-Q1..Q5) + §4 item 2 (Phase-1 capture instruction) · RFC-0018 §3.3 (Swap-as-endorsement), §4.3 (G-Swap), §4.5 (the ratified implicit-flows decision — see §6.4's honesty correction) · RFC-0034 §5 (the `fast`/`balanced`/`certified` modes), §6 (RFC-0012 scoped resolution, reused here), §7 (generation ≠ consumption) · ADR-013 §2 ("artifact metadata" as the spore-envelope extension point) · RFC-0013 (the first-fault diagnostic record pack 03 owns; DN-141 packages are instances of it, never a second bus) · house rules #1–#5 (VR-5, G2, append-only, grounding, KC-3). |
| **Depends on** | RFC-0018 (Enacted, stage 1a — the static grade lattice + G-Swap this note's remint rule composes with); RFC-0034 (the mode axis this note's firewall composes with); ADR-013 (the spore artifact this note's envelope extends); RFC-0012 (the scoped-precedence mechanism `nodule`/`phylum`/`global` this note's boundary tiers reuse — no new scoping machinery, per the steer's own convention for `policy`/`CertMode`/retention). |
| **Feeds** | Pack 03's first-fault bus (RFC-0013 amendment, Phase-1 item 3) — every dynamic boundary decision this note defines (`airlock`/`firewall`/`quarantine`/`meet_refuse`/`swap_check`) is packaged as a **first-fault-envelope instance** (§7), not a parallel diagnostic system (**N9**/G-9). |
| **Date** | 2026-07-18 |
| **Definition of Done** | §9. In one line: **Accepted** requires a maintainer ratification date recorded (H2) plus the DESIGN-02 §9 gate items carried forward — steers on the (now-resolved) open questions, the remint hinge specified so laundry is impossible by construction, the pack-03 stress probes passing, and no status claim beyond what's checked (VR-5). |

> **Posture (transparency rule / VR-5 / G2).** This note **distills** DESIGN-02 into a citable
> DN body per the steer's own instruction ("pack 02 ratifies as DN-141's body; one source of
> truth" — steer P2-Q5) and folds in the five 2026-07-17 steers as normative. It does **not**
> edit `DESIGN-02` itself (left as the historical design-pack record). Every normative claim below
> cites its basis (steer register row, RFC/ADR section, or DESIGN-02 section); rationale summaries
> carried over from the steer are tagged `Declared` (the steer document's own honesty field says
> the same of itself) — they are the maintainer's recorded reasoning, not independently
> re-verified primary-source claims by this note. **One correction is applied against the steer's
> own grounds text, not silently repeated** (VR-5 §6.4 below): RFC-0018 §4.5 is **Enacted/Ratified**
> (Design A, 2026-06-18), not "still-open," contrary to the P2-Q4 grounds column's phrasing —
> flagged, not parroted.

---

## 1. Why this document exists (distilled from DESIGN-02 §1)

Composition uses lattice **meet** (weakest-wins, RFC-0018 §4.1: `g₁ ∧ g₂` = the lesser grade in
trust order). That is *correct* integrity composition — and also the exact mechanism by which one
`Declared` leaf, one mixed-provenance dataset row, or one `fast`-floored claim can **poison** an
entire pipeline if there is no **named boundary** at which contamination is forced to declare
itself. Prime directive **N5** ("contamination stops at walls — no laundering, no global quality
kill," source: DESIGN-02) states the requirement this note answers: *how do tags stay honest
without drowning authors, and how does contamination stop at walls without killing `Exact` cores
or greenwashing weak data?*

No prior landed `docs/notes/DN-141-*.md` file exists in this repository (free-ID check above).
DESIGN-02's own header names a "former Draft DN-141" as one of its distilled sources — that was
pre-repository working material (an earlier agent pass, "Agent D"), not a decision-corpus entry
this note supersedes. This is therefore the **first landed DN-141**, minted directly from
DESIGN-02 + the steer, per house rule #3 (append-only decisions begin at `Draft`).

## 2. Three orthogonal axes — do not collapse (DESIGN-02 §2)

| Axis | Answers | Silent-collapse risk |
|---|---|---|
| **Grade** (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`, RFC-0018 §4.1) | How strong is this claim? | Treating `fast` as "all `Declared` forever" |
| **Cert mode** (`fast` · `balanced` · `certified`, RFC-0034 §5) | How much cert machinery ran? | Treating an unchecked cert as `certified` |
| **Typing strictness** (`loose` · `strict`) | How hard does the checker refuse? | Hiding a refusal as warning-only, without naming the mode |

These three dials are independent by design (RFC-0034 §11's transparency argument: collapsing grade
into mode, or mode into typing, is exactly the kind of silent conflation the transparency rule
forbids). Nothing in this note merges them into one slider.

## 3. Mental model — plant vs. cleanroom (DESIGN-02 §3, carried forward verbatim in structure)

Two coexisting worlds: an **Exact-core cleanroom**, where dual-path Exact leaves compose directly
into an Exact pipeline; and a **dust zone** (exploration/port/ML), where `Declared`/`Empirical`
material composes freely **inside a quarantine bag** (meet is free there — R4 below). Crossing from
the bag into the cleanroom, or into a `pub` export, or into a `certified` consumer, requires a
**seal**: `std.airlock` validates a **total predicate** or a **Swap certificate** (RFC-0018 §3.3).
A successful seal yields `Some` and the value enters the core; a failed seal yields `None`/`Result`
plus an `EXPLAIN` package (§7) — **never** a silent `as Exact` cast. The published result carries
its written grade (`pub export @ grade`), and — per RFC-0034 §5/ADR-013 — a deployed spore's
envelope should carry the minimum grade and cert mode composed into it.

**Honesty note on the spore-envelope claim.** ADR-013 §2 names "artifact metadata — provenance,
guarantee/bound certificates, signatures" as one of the four components of a spore, but does
**not itself** specify a `min_grade`/`mode` pair as fixed envelope fields — that shape is a
DESIGN-02 elaboration on top of ADR-013's "artifact metadata" extension point and RFC-0034 §5's
mode axis, not a field ADR-013 already ships. Implementing P2-Q3 (§6.3) therefore requires
extending the spore metadata schema at that named extension point; this is flagged as an open
implementation item, not assumed pre-existing (VR-5).

**Invariant:** meet may **weaken** inside a bag. Crossing into an Exact core, a `pub` export, or a
certified consumer requires a seal (or an explicit weak export accepted by a consumer that opts
in). Never `as Exact`.

## 4. Deterministic rules (DESIGN-02 §5, carried forward)

| Rule | Deterministic behavior |
|---|---|
| **R1 Modular bottom** | Unannotated code demands/advertises **Declared** (matches RFC-0018 §4.7's R18-Q5 disposition: an unannotated parameter demands `Declared`, an unannotated return advertises `Declared` — grading only ever *bites* where a grade is written). |
| **R2 Weaken-only annotation** | `e @ g` may only weaken the inferred grade (RFC-0018 §3.2/G-Weaken — the same VR-5 rule, restated as a typing rule). |
| **R3 Matrix mint** | Library ops get grades from committed tables, not ad hoc annotation. |
| **R4 Meet free inside the containment unit** | Composition inside a quarantine bag is unrestricted — **scoped to the nodule per P2-Q4** (§6.4). |
| **R5 Meet refuse at export / Exact demand / certified consumer** | Without a seal, refuse — table-driven allow/refuse (the meet-boundary table, §8's B3). |
| **R6 Remint only if** a total Exact-decidable predicate **or** a Swap certificate **or** a basis-carrying strengthen | Else a type error — **narrowed to v0 = {Swap cert, total Exact-decidable predicate} per P2-Q1** (§6.1); trial-basis (`Empirical`) strengthening is explicitly deferred, not included in "basis-carrying strengthen" for v0. |
| **R7 Mode floor** | `fast` cannot display an unearned `Empirical`/`Proven` grade as checked (RFC-0034 §7's honest floor: `fast` does not claim `Empirical` because it did not run the trials). |
| **R8 Isolation EXPLAIN** | Every dynamic boundary decision generates a package that is an **instance of the pack-03 first-fault envelope** (RFC-0013), never a second bus (§7). |

When pure dual-path composition is impossible (e.g. statistical models), the consumer must
**accept** the weak grade **and** receive the `EXPLAIN` package; a consumer that demands `@ Exact`
against a genuinely weaker input gets an **error**, never theater.

## 5. Pain this model addresses (DESIGN-02 §4, retained for traceability)

| ID | Pain | Containment angle |
|---|---|---|
| T1 | One weak intermediate degrades a whole expression | Dual-path / meet-boundary (R4/R5) |
| T2 | Dataset `meet_all` zeros a whole batch's `Exact` | Partition by grade (B6, §8) |
| T3 | Unannotated fns stay `Declared` forever on return | Catalog inference; write `@ g` only on exports (R1/R3) |
| T4 | `fast` meets into a `certified` Exact claim | Mode firewall + seal (§6.3) |
| T5 | VSA/resonator dust | Seal-to-codebook or `≤ Empirical` explicit (R6) |
| T6 | Transpile `Declared` flood | Draft-phylum quarantine; no tag fabrication |
| T7 | Laundry seal (fake remint) | Checker remint hinge (R6) or no remint sugar — closed by construction, §6.1 |
| U1 | Manual `@ g` ceremony everywhere | Modular bottom = `Declared`; write only to strengthen/export (R1) |
| U2 | "Why `Declared`?" is second-class | Grade `EXPLAIN` tiers (§7) |

## 6. The five 2026-07-17 steers — normative body

This section folds `PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.2 (P2-Q1..Q5) into DN-141's
normative text, resolving DESIGN-02 §8's five open questions.

### 6.1 P2-Q1 — remint bases v0 (resolves DESIGN-02 §8 Q1)

**Decision (binding steer, P2-Q1).** Remint bases in v0 are exactly two: (a) a **Swap
certificate** — the existing RFC-0018 §3.3 endorsement point, where the certificate's attested
grade is what the checker trusts (RFC-0018 §4.3 G-Swap: `g_out` may exceed `g_in` only when
`cert.valid` holds); and (b) a **total, Exact-decidable predicate** — a checker-verifiable
function that decides membership exactly, with no partial/undefined cases. **Trial-basis
(`Empirical`) remint is deferred** to a later, separately-audited, separately-`EXPLAIN`ed channel
— it is *not* part of v0's `std.airlock`, narrowing R6's "basis-carrying strengthen" clause for
this release.

**Grounds.** This is the minimal extension of RFC-0018's only-raise rule (G-Swap already
establishes that a grade may rise only through a certified endorsement); every release path out of
a quarantine bag is attack surface, so widening it to include an unaudited trial-basis channel in
v0 is declined. The rationale mirrors sanitizer-gated remint patterns and "parse, don't validate"
(cited in the steer's own companion citation register; not independently re-verified here — see
this note's posture statement).

**Normative rule.** `std.airlock` v0's predicate argument **must be total and Exact-decidable**;
a predicate with an empty/undecidable basis is **refused by construction**, never silently
accepted as a laundered `Exact`. The refusal is the `seal_remint` first-fault site (§7): laundry
(an attempted remint with an empty basis) is exactly T7's pain, and this rule is what closes it —
`basis_ref` is empty **only** on refuse, never on a `pass_remint` decision.

### 6.2 P2-Q2 — export-only seal first; no `Quarantined[T]` carrier in v0 (resolves DESIGN-02 §8 Q2)

**Decision (binding steer, P2-Q2).** v0 ships **export-only seal enforcement** — the boundary is
checked at the crossing point (export / Exact demand / certified consumer), not carried as a
first-class type. **No `Quarantined[T]` type carrier lands in v0.** A companion, timeboxed design
spike (**S1**, Appendix A) is commissioned in parallel as prep for a probable future need — the
spike produces a design record only, no production code.

**Grounds.** A coarse boundary check (checked at the wall, not threaded through every type) delivers
the containment property at the lowest ceremony cost — the pattern this steer's rationale draws on
is the same "check at the boundary, not everywhere" tradeoff documented for capability/taint
propagation systems with high per-value ceremony (JIF/LIO-class systems, per the steer's citation
register). The maintainer judges a type-level carrier a **probable** future need (not certain), which
is exactly the disposition that warrants a spike now and an implementation deferral — YAGNI applied
to the *carrier*, not to the *containment property* itself (R4/R5 ship in v0 regardless).

**Normative rule.** Export-only seal enforcement is R5's mechanism in v0: the meet-boundary table
(B3, §8) is consulted at export / Exact-demand / certified-consumer crossings; there is no `T`-level
`Quarantined` wrapper to track a value's containment status across a function signature. See
Appendix S1 for the carrier design prep.

### 6.3 P2-Q3 — certified-colony admission via explicit airlock (resolves DESIGN-02 §8 Q3)

**Decision (binding steer, P2-Q3).** **Yes** — in `certified`-mode colonies, `fast`-mode spores are
admitted **only via an explicit airlock**. This upgrades RFC-0034 §6's cross-mode visibility
requirement (combining a `fast` value into a `certified` computation surfaces the mode boundary)
from a *visibility* guarantee into a *governance* guarantee: the boundary is not just observable, it
is a checked admission gate.

**Grounds.** RFC-0034 §6 already requires that cross-mode composition "surfaces the mode boundary;
the result cannot silently inherit `certified` strength it did not earn" — this steer makes that
requirement an enforced admission check rather than a passive surfacing. The analogy drawn in the
steer's rationale is a clearance/classification-boundary check (cited in the steer's companion
register). Per §3's honesty note, the spore envelope's min-grade + mode fields are the DESIGN-02
elaboration this check reads — an extension of ADR-013's "artifact metadata," not an existing field.

**Normative rule.** A certified colony's admission path reads the incoming spore's envelope
(min-grade + mode, once that schema extension lands per §3); a `fast`-floored spore without an
explicit airlock pass is **refused**, and the refusal emits a `mode_firewall` first-fault (§7) —
never a silent no-op admission and never a silent grade rewrite.

### 6.4 P2-Q4 — meet free inside the nodule only (resolves DESIGN-02 §8 Q4)

**Decision (binding steer, P2-Q4).** **Meet is free inside the nodule only.** Crossing a `phylum`
boundary is a **boundary decision plus `EXPLAIN`** — not a free composition. This fixes R4 (§4) to
the *finest* tier of the RFC-0012/RFC-0034 §6 scoping precedence (`global ⊐ phylum ⊐ nodule`,
most-specific-wins), i.e. the smallest defensible blast radius: a quarantine bag's free-meet zone is
exactly one nodule, never a whole phylum. A companion, timeboxed spike (**S2**, Appendix B) is
commissioned on phylum-wide free meet as a probable future widening.

**Grounds.** Matching the finest scoping tier keeps the free-meet zone as small as the corpus's own
scoping mechanism allows without inventing new machinery (RFC-0012's precedence chain is reused
verbatim, as `policy`/`CertMode`/retention resolution already do — no fourth bespoke scoping
mechanism).

**Honesty correction on the steer's own grounds text (VR-5, not silently repeated).** The steer's
P2-Q4 grounds column additionally states this choice "keeps RFC-0018 §4.5 (implicit flows — still-
open maintainer decision) local." **This note flags that framing as inaccurate against the current
corpus, rather than restating it.** RFC-0018 §4.5 is **Enacted/Ratified**, not open: the maintainer
recorded **R18-Q1 = Design A** (data-lineage-only, no `pc`-taint) on 2026-06-18, and stage 1a
(§4.7) landed it in `crates/mycelium-l1` via M-663 (2026-06-22) — RFC-0018's own Meta-changelog
states this explicitly ("ACCEPTED... R18-Q2/R7-Q2 closed"; "ENACTED (stage 1a; M-663)"). Repeating
"still-open" here would itself be an ungrounded claim (house rule #4 binds agreement with a steer's
rationale exactly as it binds any other claim — VR-5 applied to assent).

What **does** remain genuinely open and load-bearing for S2's scope is a **narrower, correctly-cited**
coupling: RFC-0018 §4.5's own ratified note records a **precondition** (research/09, T9.6) that
"Design A's sufficiency rests on the calculus being pure; when observable effects land, they must
become graded outputs (RFC-0014, route i) or carry a local `pc` (route ii)." Widening the meet-free
zone from nodule to phylum enlarges exactly the region over which that purity precondition would
need to keep holding — a wider free-meet zone is a wider region where an effectful, `Declared`-
influenced computation could compose without a boundary decision, which is precisely the class of
case T9.6 flags as needing a `pc`/graded-output treatment once effects land. **S2 is scoped to that
precondition-interaction question**, not to re-litigating the already-ratified Design A/B choice
(Appendix B).

**Normative rule.** R4 (§4) reads: composition inside a **nodule**'s quarantine bag is meet-free.
A `phylum`-crossing composition is **not** automatically free; it is a boundary decision, table-driven
per B3 (§8), and every such crossing emits an `EXPLAIN` package (§7) — an instance of the pack-03
first-fault envelope, never silent.

### 6.5 P2-Q5 — this note is pack 02's sole ratification vehicle; the DN-142 number is reserved elsewhere (resolves DESIGN-02 §8 Q5)

**Decision (binding steer, P2-Q5).** Pack 02 ratifies **as this note's body** — "one source of
truth" (the steer's own wording). DESIGN-02 §8 Q5's alternative ("new DN-142 'containment
topology'") is **declined**: no second, phylum/topology-specific note is minted for pack 02's
subject matter.

**Numbering disambiguation (flagged to avoid a collision-looking coincidence).** The `DN-142` slot
*is* being minted in this same wave — but for an **unrelated subject**: the "Swap Ergonomics DN"
(`PROGRAM-HANDOFF` §4 item 1, pack 01 — `policy: ambient` resolution law, the cert-handle
architecture, `to:` elision gates), which is the *next free DN slot* found by that note's own
free-ID check, independently of pack 02. **This is not the "DN-142 containment topology" idea
DESIGN-02 §8 Q5 floated and this steer declined** — that hypothetical note remains unminted, by
this steer's own decision, full stop. Anyone cross-referencing a future `DN-142` file should read
it as the Swap Ergonomics DN, not as a pack-02 companion.

## 7. Diagnostic surfaces — grade, meet, seal (DESIGN-02 §5.1–§5.2, first-fault instances)

Every dynamic boundary decision this note defines generates a package that is an **instance of the
pack-03 first-fault envelope** (RFC-0013, amended per `PROGRAM-HANDOFF` §1.3 P3-Q1) — never a
second, parallel diagnostic system (**N9**/G-9).

### 7.1 Isolation `EXPLAIN` package (minimum fields)

| Field | Meaning |
|---|---|
| `boundary_kind` | `airlock` · `firewall` · `quarantine` · `meet_refuse` · `swap_check` |
| `input_grade` / `demand_grade` | lattice points at the boundary |
| `decision` | `pass_remint` · `pass_weak` · `refuse` · `fallback` |
| `basis_ref` | predicate id, Swap-cert hash, or **empty on refuse** (never empty on `pass_remint` — R6/§6.1) |
| `policy_used` / `cert_mode` | `Meta` fields (RFC-0034 §5) |
| `where` / `event_id` | span + stable fault id (the pack-03 envelope) |
| `never_silent` | always true for `refuse`/`fallback` (G2) |

Symptoms downstream set `parent_event`/`child_cause` back to this first refuse — first-fault
localization (**N6**/**N9**), never a tree-dig.

### 7.2 Surface-to-`site_kind` mapping

| Surface | When it fires | `site_kind` | Instant localization |
|---|---|---|---|
| Grade provenance | `EXPLAIN`/hover "why this grade?" | grade provenance (not always refuse) | meet-tree root + first weak leaf |
| Meet decide | Meet of two-plus grades | `grade_meet` · `meet_boundary` | operands; rule id; allow/refuse |
| Boundary refuse | Export / certified demand / Exact partition | `meet_boundary` | demand vs. input |
| Seal attempt | Airlock remint pass/fail | `seal_remint` | predicate; remint grade; empty basis on laundry refuse |
| Annotation error | Illegal strengthen / cast-upgrade | `grade_annotation` | written token; required Swap/seal/basis |
| Mode × grade firewall | Certified demand over a `fast`-floored claim | `mode_firewall` | mode cell + grades (never a grade rewrite) |

**Generation ≠ consumption** (RFC-0034 §7): lean stubs fire on refuse/downgrade always; audit-tier
DAGs are pull/opt-in. This note does not require emission on every successful Exact structural op
in `fast` — that would violate the always-lean-generation, tunable-consumption split.

## 8. Recommended package (Draft, carried from DESIGN-02 §6)

| Slice | What | Role |
|---|---|---|
| B1 | Structural grade catalog + CI overclaim guard | Everyday honesty without hand tags |
| B2 | Grade/isolation `EXPLAIN` — lean · normal · audit | "Why this grade / boundary?" |
| B3 | Meet-boundary table (export / certified / Exact partition; nodule-vs-phylum per §6.4) | Deterministic walls |
| B4 | `std.airlock` seal/recertify + laundry CI (v0 basis = §6.1) | Named remint only |
| B5 | Certified firewall (mode × grade, per §6.3) | Cross-mode refuse without seal |
| B6 | Spore/dataset partitions | Packaging-level containment |
| B7 | Optional basis syntax `@ Empirical(…)` / `@ Proven(…)` | When dogfood needs WRITE+EARN |

**Author ceremony budget (unchanged from DESIGN-02):** zero tags on exploratory code; annotate
public APIs and seals; Exact cores stay on dual-path inputs.

## 9. Definition of Done

Before this note may be ratified `Accepted` (H2 — a maintainer must record the date):

- [x] Steers recorded on DESIGN-02 §8's five open questions (this note, §6.1–§6.5).
- [x] The remint hinge (§6.1/R6) specified so laundry is impossible by construction (empty-basis
      refusal, `seal_remint` first-fault).
- [ ] The pack-03 stress probes (DESIGN-03 §7) pass against this note's `site_kind` mapping (§7.2)
      — pending pack-03's RFC-0013 amendment landing (Phase-1 item 3; tracked separately).
- [x] No status claim beyond `Draft` — this note does not claim `Accepted` (VR-5/H2).
- [ ] The spore-envelope schema extension (§3's honesty note) is minted as an implementation item
      before P2-Q3 (§6.3) can move past design.
- [ ] S1 and S2 spikes (Appendices A/B) run and their trigger criteria are evaluated against
      first-fault telemetry once the bus (W-A) is live.

## Appendix A — S1 spike: `Quarantined[T]` carrier design prep (Declared, timeboxed, no production code)

Per §6.2 (P2-Q2), commissioned as design prep for a **probable** future need, not an implementation.
Every claim below is `Declared` (asserted design sketch, not built or checked) and out of scope for
this note's ratification (§9).

**Carrier type sketch.** `Quarantined[T]` as a **zero-cost newtype** over the existing graded type
`T @ g`: a nominal wrapper `Quarantined[T] ≜ { inner: T @ g }` that forbids implicit unwrap — no
`Deref`/coercion path exists from `Quarantined[T]` to `T`. Construction happens implicitly on entry
to a quarantine bag (mirrors R4's existing "meet free inside the nodule" semantics — the wrapper
adds no new composition rule, only a type-level marker of "still inside the bag"). Extraction is the
one privileged operation, requiring an `std.airlock` call exactly as export-only seals do today.

**Interaction with meet rules R4/R5.** Meet composition **inside** the wrapper is unchanged — two
`Quarantined[T]` values compose meet-free per R4, and the result is itself `Quarantined[T]` (the
wrapper is closed under meet, matching the existing bag semantics). R5's boundary refusal is
unchanged in *when* it fires (export / Exact demand / certified consumer); what changes is *where*
the type system can now **statically** flag an attempted crossing — a function whose parameter type
is `T @ g` (not `Quarantined[T]`) called with a `Quarantined[T]` argument becomes a **type error at
the call site**, catching what is today only a *dynamic* R5 refusal. This is a strict precision
gain over export-only enforcement, not a semantic change to R4/R5 themselves.

**Ceremony cost estimate (`Declared` — no implementation exists to measure).** Every quarantine-bag
boundary (today: zero explicit syntax, R4 is ambient within a nodule) would gain an explicit
wrap/unwrap pair at each function signature that crosses into or out of quarantined territory. The
steer's own P2-Q2 grounds cites this exact ceremony cost (JIF/LIO-class burden evidence) as the
reason export-only ships first; this spike does not dispute that cost, it estimates its shape:
linear in the number of quarantine-crossing function signatures in a nodule, concentrated at
library/API boundaries rather than internal composition (internal composition stays wrapper-closed,
per the previous paragraph).

**Migration path from export-only seals.** Every existing `std.airlock` call site (an export-only
seal crossing) becomes a `Quarantined[T]` **unwrap** call under the carrier design — a mechanical,
call-site-local rewrite, not a redesign, because export-only was always the semantically-simpler
subset of what a type-level carrier expresses (a carrier can additionally track containment through
a signature; export-only cannot). No existing `std.airlock` semantics change.

**Adoption trigger criteria.** Any of: (a) first-fault telemetry (once the pack-03 bus, W-A, is
live) shows `meet_boundary`/`seal_remint` refusal volume at nodule-exit crossings above a threshold
the maintainer judges ceremony-worth-paying-for; (b) a concrete library use case needs to accept "a
still-quarantined value" as a **parameter type** (not just a call-site check) — something export-only
enforcement structurally cannot express; (c) maintainer judgment, per the steer's own framing of this
as a "probable" (not certain) future need. No rule changes as a result of this spike alone (§6.2) —
only a future, separately-ratified decision could adopt the carrier.

## Appendix B — S2 spike: phylum-wide free meet (Declared, timeboxed, no production code)

Per §6.4 (P2-Q4), commissioned as design prep for a **probable** future widening. `Declared`,
out of scope for this note's ratification.

**Blast-radius analysis.** Widening R4's free-meet zone from `nodule` to `phylum` removes the
boundary decision + `EXPLAIN` requirement (§4/§6.4) for every nodule-to-nodule composition that
stays inside one phylum. A phylum (RFC-0034 §6's middle scoping tier) can contain many nodules
authored/composed by many contributors or agents over time — so the blast radius of a single
`Declared` leaf silently propagating (in the "no boundary decision fires" sense — grades still meet
correctly, but the walk-to-a-wall property that lets an author *notice* a crossing is what's lost)
grows from "one nodule's internal composition" to "everything anyone ever placed in the same
phylum." This is the concrete cost the steer's own P2-Q4 disposition ("smallest defensible blast
radius") is declining to accept in v0.

**Coupling to RFC-0018 §4.5 (correctly scoped per §6.4's honesty correction).** RFC-0018 §4.5 is
ratified (Design A, data-lineage-only) — this spike does **not** reopen that choice. The genuine
coupling is RFC-0018 §4.5's own recorded precondition (`research/09` T9.6): Design A's
noninterference sufficiency assumes the calculus stays **pure**; once observable effects land, they
must become graded outputs (RFC-0014 route i) or carry a local `pc` (route ii). A wider meet-free
zone is a wider region over which that purity precondition must keep holding for Design A's
sufficiency argument to remain valid — S2 must therefore assess, before any phylum-wide widening: if
effects land inside a phylum-wide meet-free zone, does the T9.6 precondition still hold at
phylum-exit, or does the widened zone need its own `pc`-like discipline that nodule-scoped R4 avoided
needing? This is the load-bearing open question S2 is scoped to, not a re-litigation of R18-Q1.

**Boundary-table delta (S2's required output, not assumed here).** Today's meet-boundary table (B3,
§8) has one crossing tier under R4/R5: nodule-exit. A phylum-wide widening would either (a) collapse
that to phylum-exit only (removing the nodule-internal wall entirely), or (b) add an intermediate,
softer tier (nodule-exit becomes advisory/free, phylum-exit stays the hard wall). S2 must produce
the actual table delta as a design-note output — this appendix does not presuppose which shape is
correct.

**Trigger criteria.** Measured boundary-ceremony cost inside cohesive, single-authored phyla: once
first-fault telemetry (W-A) is live and counting `meet_boundary` refusal events per phylum, if
refusals cluster overwhelmingly at intra-phylum, inter-nodule crossings within phyla that show no
independent evidence of a real trust boundary (a proxy the spike must define precisely, not assume),
that is empirical evidence toward widening. Until such telemetry exists, this stays `Declared` and
undecided — no widening happens on this spike's say-so alone.

## Appendix C — W-C implementation note (X2–X5, 2026-07-18; `Declared`, disclosed judgment calls)

Per this note's own posture (§0/no-status-change discipline — this appendix does **not** move
`DN-141`'s `Status` past `Draft`, H1/H2), the W-C leaf (steer §5 row: "structural grade catalog + CI
overclaim guard; regime→result enforcement (`regime_type_lie`); meet-boundary table; isolation
EXPLAIN as envelope instances") records what actually landed in `crates/mycelium-l1/`, and the
disclosed residuals it chose **not** to force, rather than silently claiming completion (VR-5).

**X2 (structural grade catalog + overclaim guard) — landed.**
`crates/mycelium-l1/src/grade_catalog.rs` commits the R3 "matrix mint" as data: one row per
RFC-0018 §4.3 structural rule `crate::grade::Gx::grade` already implements (G-Const…G-Wrapping),
each naming its RFC citation — replacing scattered doc-comment prose with a single, queryable,
completeness-tested table. R1 (modular bottom) and R2 (weaken-only annotation) were **already**
fully implemented and tested (`crates/mycelium-l1/tests/check.rs`'s guarantee-grading suite,
`M-663`) — cited, not duplicated (exit-criterion instruction). The **overclaim guard** is an
exhaustive (not sampled — `Strength` is a 4-variant closed enum) property-test suite over
`Strength::meet`/`Strength::satisfies` (`src/tests/grade_catalog.rs`) proving the lattice arithmetic
every rule composes on can never let a composed grade outrank either of its inputs — the algebraic
form of "no op's displayed/exported grade exceeds its catalog/basis." R7 (the `fast`-mode floor) is
**already** implemented and tested at the value/kernel layer (`mycelium_core::CertMode::gate_result`,
`crates/mycelium-core/src/tests/{cert_mode,mode_tests,mode_harness}.rs`) — cited, not duplicated; the
static checker (`grade.rs`) runs mode-independently, so there is no analogous static-layer floor to
add. **Out of scope, by RFC-0018's own disposition:** R18-Q3's per-prim signature table (§8: "a
separate tracked deliverable... the conservative G-Op default is sound meanwhile") — this leaf does
not invent per-prim precision RFC-0018 itself declines to require yet (VR-5, no unbounded upgrade).

**X3 (regime→result enforcement) — classification + `Diag` builder landed; hard refusal deferred,
disclosed.** `crates/mycelium-l1/src/regime.rs` implements RFC-0002 §4's own **direction-aware**
distinction (`enc : Bin_n -> Tern_m`, no `Option` in its signature = **Total**; `dec : Tern_m ->
Option Bin_n` = **Partial**) plus a `regime_type_lie` `Diag` builder (`SiteKind::RegimeTypeLie`).
**Deliberately not wired as a hard checker refusal into the pre-existing, explicit `to:` swap
spelling**: doing so would retroactively break already-shipped, already-tested behavior — concretely,
the `dec`-direction round-trip chain `swap(swap(b, to: Ternary{6}, policy: rt), to: Binary{8}, policy:
rt)` that `crates/mycelium-l1/tests/differential.rs`, `tests/runnable_gate.rs`, and
`crates/mycelium-bench/src/corpus.rs` all assert type-checks (verified 2026-07-18: still present,
still green). DN-142 §5 gate 3 itself binds "regime typing from the resolved pair" to the **`to:`-
elision feature (X9)**, which is held per the steer (§5, "AX-sugar... after walls") and does not
exist anywhere in `parse.rs` today (grep-verified, zero hits) — X9's own elision-resolution path is
the natural, non-breaking call site for a hard `regime_type_lie` refusal once it lands. Retrofitting
the *existing* spelling instead would be an unauthorized, breaking redesign outside a single leaf's
authority (mitigation #14). Flagged, not silently decided either way.

**X4 (meet-boundary table) — table + `Diag` builder landed and tested-equivalent to the real
enforcement; not yet live-wired into `grade.rs`, disclosed.** `crates/mycelium-l1/src/meet_boundary.rs`
commits `BoundaryKind::{Export, ExactDemand, CertifiedConsumer}` (the last with **no row** — X6/X7,
held to wave W-E, per P2-Q2/P2-Q3) and a table-driven `check_boundary`/`meet_boundary_refuse_diag`
pair, proven (exhaustively, `src/tests/meet_boundary.rs`) equivalent to the arithmetic
`grade.rs::Gx::require`/`check_fn_grades` already enforce. R4 ("meet is free inside") is confirmed
holding in `grade.rs`'s actual source today by a grep-level regression guard (7 `.meet(` internal-
composition sites vs. 3 `self.require(` boundary-demand sites, pinned). **Not wired live into
`grade.rs`**: `require`'s three call sites are not all the same DN-141/DESIGN-03 site_kind (a `let`/
value ascription is `grade_annotation`; only the `grade_app` argument-demand call site is genuinely
`meet_boundary`), and `grade.rs` is the file `crates/mycelium-std-conformance/tests/reject_ledger.rs`
pins an exact `CheckError`-construction count for (DN-80) — correctly threading a `BoundaryKind`
through three semantically-distinct call sites is a small, focused follow-on in its own right, not
folded into this already-large leaf (KC-3 smallest-auditable-step discipline).

**X5 (isolation EXPLAIN as envelope instances) — the `mycelium-l1` → `mycelium-diag` edge added;
two refusal sites live-wired; two more built and tested but not (yet) live-wired, disclosed.**
Added `mycelium-diag` as a **direct** dependency of `mycelium-l1` (`Cargo.toml`) — judged
architecturally clean: both crates are `core`-tier (`xtask/deps-strata.toml`), the edge is strictly
downward (`mycelium-l1` stratum 4 → `mycelium-diag` stratum 1), and `mycelium-diag` was already
pulled in *transitively* via `mycelium-cert` (W-A's `swap_check` emitter, `mode.rs`), so no new node
enters the workspace dependency graph. **Live-wired** (`crates/mycelium-l1/src/checkty.rs::check_swap`):
the `legal_pair_refuse` and `policy_resolve` (refuse form) first-fault packages now back the
checker's own `illegal swap pair`/`no ambient policy declared` `CheckError`s — the `Diag`'s rendered
text is the error message (one source of truth, never a second diagnostic system — G-9), and the
DN-80-pinned `checkty.rs` `CheckError::new`/`::at` (115) and `self.err(` (135) call-site counts are
**unchanged** (an argument-only change to two pre-existing call sites, verified by re-grep — no DN-80
reconcile row needed). A `policy_resolve` **success**-crumb builder also landed
(`ambient_policy::policy_resolve_diag`, reusing `explain_origin`'s rendering, DRY) but is not wired to
a live sink — a successful resolution is an optional RFC-0013 §4.6 "non-site" crumb, and `checkty::Cx`
has no diagnostics-collection channel to push it through without a broader, separately-scoped API
change. `regime_type_lie` (X3) and `meet_boundary` (X4) are correct and tested but likewise not
live-wired, for the reasons stated in their own sections above.

**Residual/deferred items, summarized (never silently dropped — G2):**
1. R18-Q3 per-prim grade table — RFC-0018's own separately-tracked deliverable, out of scope.
2. `regime_type_lie` hard refusal — deferred to the X9 `to:`-elision landing (or an explicit
   maintainer decision to accept breaking the cited pre-existing tests).
3. `meet_boundary`/`grade_annotation` live wiring into `grade.rs::require` — a small, focused
   follow-on (three call sites, two distinct site_kinds) deliberately left unfolded into this leaf.
4. A `checkty::Cx` diagnostics-collection channel (for `policy_resolve`'s success crumb and any
   future non-refusing first-fault emission) — a broader API addition, not attempted here.
5. `CertifiedConsumer` boundary crossing (mode × grade firewall, P2-Q3/X7) — held to wave W-E per the
   steer; `meet_boundary.rs`'s enum names the kind with no row, not fabricated.

## 10. FLAGs for the integrating parent (this note does not edit these)

- **`CHANGELOG.md`** — add a `docs(notes): DN-141 …` entry under the design-phase section
  (append-only). *Owned by the integrating parent.*
- **`docs/Doc-Index.md`** — register `DN-141` in the notes index with its one-line summary and
  cross-refs to DESIGN-02 / RFC-0018 / RFC-0034 / ADR-013. *Owned by the integrating parent.*
- **`tools/github/issues.yaml`** — this note mints no build issues (design-only, no code); the
  §9 residual DoD items (spore-envelope schema extension, S1/S2 spike execution, pack-03 stress
  probes) are candidates for issue minting once a build wave is scoped. *Owned by the integrating
  parent — `issues.yaml` is read-only to this note.*

## 11. See also

- Pack [01](../planning/gap-analysis-2026-07-16/DESIGN-01-SWAPS-AND-POLICY.md) — Swap-cert
  failures feed the same isolation story (§6.1's remint basis).
- Pack [03](../planning/gap-analysis-2026-07-16/DESIGN-03-MACHINERY-DIAGNOSTICS-AND-UX.md) —
  first-fault emitters, the shared envelope + site catalog this note's §7 instances.
- `PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.2/§4 item 2 — the steer register this note
  ratifies as its body.

## Changelog (this note)

| When | Note |
|---|---|
| 2026-07-18 | **Draft** minted. Distills `DESIGN-02-TAGS-META-AND-CONTAINMENT.md` into DN-141's body per steer P2-Q5 ("pack 02 ratifies as DN-141's body; one source of truth"); folds in the five 2026-07-17 binding steers (P2-Q1..Q5, §6) as normative; adds S1 (`Quarantined[T]` carrier) and S2 (phylum-wide free meet) spike appendices per the steer's own commissioning; flags one correction against the steer's own P2-Q4 grounds text (RFC-0018 §4.5 is Enacted/Ratified, not "still-open" — §6.4) rather than silently repeating it, per VR-5/house rule #4. Free-ID check: `DN-141`/`DN-142` both verified absent before minting (H1); `DN-142` is noted as reserved elsewhere in this wave for the unrelated Swap Ergonomics DN (§6.5), not for pack 02. Status stays **Draft**; not self-ratifying (H2). `CHANGELOG.md`/`docs/Doc-Index.md`/`tools/github/issues.yaml` rows FLAGGED for the integrating parent (§10), not edited here. |
| 2026-07-18 | **Appendix C added** (W-C leaf, steer wave X2–X5): records what actually landed in `crates/mycelium-l1/` (`grade_catalog.rs`/`regime.rs`/`meet_boundary.rs` + the direct `mycelium-diag` dependency edge + two live-wired first-fault refusal sites in `check_swap`) and the disclosed residuals (the `regime_type_lie` hard refusal deferred to the held X9 `to:`-elision feature; `meet_boundary`/`grade_annotation` live-wiring into `grade.rs::require` left as a focused follow-on; R18-Q3's per-prim table out of scope). Status stays **Draft**; this appendix does not self-ratify (H1/H2). `CHANGELOG.md`/`docs/Doc-Index.md`/`docs/api-index/`/`tools/github/issues.yaml` rows FLAGGED for the integrating parent, not edited here. |
