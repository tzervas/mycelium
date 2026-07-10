# ADR-045 — Kernel & Lexicon Unfreeze for Early Gap-Closure (a bounded, re-freezable NOW-window; the ADR-042 end-state unchanged)

| Field | Value |
|---|---|
| **ADR** | 045 |
| **Status** | **Draft** (2026-07-10 — records the maintainer's 2026-07-10 direction to **temporarily + boundedly UNFREEZE** the kernel primitive/type set and the stdlib public-API surface ("lexicon") so the language can be **optimized while its kernel/lexicon expressibility gaps are closed early** — before the same closure is a hundred-module lift once the stdlib port + DN-88 decomposition scale up. **This session drafts; the maintainer ratifies.** `Draft → Accepted` is the maintainer's step — and is conditioned on resolving the three FLAGged scoping questions (§6 OQ-1/OQ-2/OQ-3). `Accepted → Enacted` happens **only** when the re-freeze conditions (§2.4) are met — the named gap-closure worklist is exhausted, the DN-56 gate is re-scored green by a DN-76-successor scorecard, and the DN-39-only diff policy is reinstated by a follow-up maintainer-ratified ADR (house rule #3 — must step through Accepted first, never skip to Enacted). **Enacts nothing; ratifies nothing; freezes/unfreezes nothing while Draft.** |
| **Decides** | *Proposes, for maintainer ratification (nothing here is self-ratified — VR-5/house rule #3):* (1) **The unfreeze.** For a **bounded window**, lift the **NOW-horizon** freezes on the **kernel** (`mycelium-core` / `mycelium-l1` — the L1 ten-node budget + the ratified Π primitive set + the value representations) and the **lexicon** (the stdlib public-API surface of the 26 `mycelium-std-*` crates), so both may be **optimized** and **grown to close expressibility gaps** now. (2) **What stays unchanged.** The **ADR-042 §2.1(b) END-STATE** — the entire first-party project, kernel included, rewritten to `.myc`; **zero foreign first-party languages** by the DN-88 decomposition gate — is **untouched**; only the NOW-horizon freeze is lifted. The **DN-39 kernel *boundary*** (the trusted-component *set*) is **not** widened by this ADR; the never-silent (G2) discipline is **retained** through the window (every kernel/lexicon change stays reviewed, `EXPLAIN`-able, and honestly tagged — VR-5). (3) **Re-freeze conditions (§2.4).** The window closes, and the freezes re-declare, only on the three checked conditions in §2.4. (4) **Coordination (§2.5).** This ADR **governs**; it does **not implement** — kernel/grammar/runtime implementation is the cloud CC session's **semcore self-hosting lane (M-1013)**. |
| **Amends / proposes to supersede** *(on ratification, not while Draft — house rule #3)* | **ADR-042 §2.1(a) Decision (1) — "Rust-base freeze (NOW)"**: the NOW-horizon freeze on new language-project surface is **lifted for the window**, scoped to kernel + lexicon gap-closure/optimization (ADR-042 §2.1(b) END-STATE **unchanged**). **DN-56 §9 — "THE KERNEL FREEZE IS DECLARED (M-969)"** + **§6 post-freeze diff policy (DN-39-only)**: the primary kernel-freeze declaration and its "DN-39-promotion-only / else `core 2.0.0`" diff policy are **suspended for the window** — generalizing DN-56's own **2026-07-06** addendum (pre-production, the freeze is a *discipline gate* with a per-change maintainer-authorization path) from a per-change authorization into a **standing, bounded window**. **DN-66 §2 — the stdlib stable-API freeze** of the 26 `mycelium-std-*` public APIs (the "lexicon"): unfrozen for the window (DN-66 §8's enactment/retirement gate — retirement waits on fully-operational `.myc` rewrites — is **unchanged**). |
| **Relates (pointer only — NOT edited by this ADR)** | **DN-76** (the four-condition kernel-freeze scorecard, the M-969 gate instrument) — a **DN-76-successor scorecard** is the §2.4 re-score instrument; DN-76 itself is the **cloud SCC session's lane** and is **flagged, not edited** here. **ADR-043** (Rust retirement / retire-when-proven) — **unaffected**: the retirement gate is unchanged; this ADR only lifts the *freeze* the gate assumes stable underneath it. **RFC-0041 §6** (within-freeze behavior-preserving hardening) and **DN-39** (the KC-3 promotion bar) remain available and are the *narrower* channels this window widens. |
| **Grounds** | Maintainer directive (2026-07-10, this session — recorded verbatim in intent; `Declared` until `Accepted`, then binds); **ADR-042 §2.1(a)/(b), §4** (the two-horizon freeze this amends at the NOW edge and preserves at the end-state); **DN-56 §6/§9 + the 2026-07-06 discipline-gate addendum** (the primary kernel-freeze + its already-admitted per-change authorization path this generalizes); **DN-66 §2/§8** (the stdlib stable-API freeze + its enactment gate); **kickoff `spw` §Mechanism + the D5 runbook step 1** (the *measured* enabler gap — transpiler union `checked_fraction` ≈ 0–8%, M-991 verdict / DN-34 §8.7–§8.9; the "STOP and FLAG to `enb`" below-grammar-enabler escalation the port loop already hits); **DN-88** (the component-repo decomposition horizon — the "hundred-module lift" the early window pre-empts); **RFC-0006 §3 layer model** (L0 Core IR → L1 kernel calculus → L2 surface → L3 projection — the layers OQ-1/OQ-2 partition); **M-1013 / DN-26** (the semcore self-hosting lane this ADR coordinates with, does not implement); house rules **#3** (append-only), **#5** (KISS/YAGNI), **G2**, **VR-5**, **KC-3**. |
| **Date** | 2026-07-10 |

> **Posture (transparency rule / VR-5 / house rule #4).** This is a **Draft governance record**, authored
> by a design-reasoner session that **recommends, never ratifies**. It **enacts nothing and changes no
> other document's status**: ADR-042, DN-56, and DN-66 keep their current statuses; their amendment lands
> only if/when the maintainer ratifies this ADR `Draft → Accepted`. The unfreeze's **necessity over the
> existing narrower channels is itself contestable and is argued against in §4** (the adversarial
> stress-test) — this session does **not** soften that disagreement to please the stated direction
> (house rule #4: follow the evidence, not the speaker). Three scoping questions the maintainer must
> decide are **FLAGged, not guessed** (§6): whether **L0 Core IR** is in scope, whether **surface grammar
> (L2/L3)** is in scope, and the **exact bound/owner** of the window. Every normative claim is grounded
> (§9) or marked open.

---

## 1. Context

Three ratified freezes currently fix the NOW edge of the language surface:

- **ADR-042 §2.1(a) — the Rust-base freeze (NOW).** No new Rust language-project surface accrues; new
  functionality is authored in `.myc`. The **§2.1(b) END-STATE** is separate and unaffected here: the whole
  first-party project, kernel included, becomes `.myc` — zero foreign first-party languages by the DN-88
  decomposition gate.
- **DN-56 §9 — the kernel freeze (M-969, Enacted 2026-07-02).** The trusted base (`mycelium-core` + the L1
  ten-node calculus + the ratified Π prim set + the value representations) is **immutable**; §6 fixes the
  post-freeze diff policy — the frozen kernel changes **only** via a DN-39 default-DENY promotion, else it
  is a `core 2.0.0` event. A **2026-07-06 addendum** already softened this pre-production: the freeze is a
  *discipline gate*, not a delivered-trusted-core immutability guarantee, and the maintainer authorizes
  sound kernel modifications **per change**, recorded, never silent.
- **DN-66 §2 — the stdlib stable-API freeze.** The public API of the 26 `mycelium-std-*` crates (the
  "lexicon") is a dated baseline a future change must not silently break; §8 gates *retirement* on the
  `.myc` rewrites being fully operational.

**The 2026-07-10 direction.** The maintainer directs a **temporary, bounded unfreeze** of the kernel
(primitive/type set) **and** the lexicon (stdlib public-API surface), so the language can be **optimized
while its kernel/lexicon gaps are closed early**. The driver is timing, and it is the maintainer's
**zero-hand-port north star**: today ports are **hand-ported** because the transpiler cannot yet express
the disjoint majority — kickoff `spw` records the union `checked_fraction` at **≈ 0–8%** (M-991 verdict,
DN-34 §8.7–§8.9), and its port runbook already **STOPs and FLAGs to `enb`** whenever a target needs a
"below-grammar enabler." The unfreeze exists so the **language** can close those expressibility gaps
(grammar / kernel / runtime), so that the **transpiler** — which holds the translate rules across
L0/L1/L2/L3 and their projections — can eventually **mechanically port** what is hand-ported today. Closing
the gaps **now**, while the surface is one monorepo of a modest size, avoids doing the same closure **once
the DN-88 decomposition has fanned the stdlib into per-component repos** (`std-io`, `std-fs`, `std-vsa`,
`std-numerics`, …) — where a single kernel/lexicon change becomes a coordinated many-repo lift.

## 2. Decision (proposed)

### 2.1 Scope of the unfreeze

For the bounded window (§2.4 fixes its end; §6 OQ-3 fixes its exact length/owner), the following are
**unfrozen** — free to be optimized and grown to close gaps, under the retained discipline of §2.2:

- **Kernel primitive/type set** — the `mycelium-core` trusted base and `mycelium-l1`: the L1 kernel
  calculus node budget, the ratified Π primitive set, and the value representations. New primitives, node
  refinements, and representation additions that close a **named** expressibility gap are in scope.
- **Lexicon — the stdlib public-API surface** of the 26 `mycelium-std-*` crates (DN-66 §2). Signature,
  guarantee-tag, and exported-op-set changes that close a gap or optimize an API are in scope; the DN-66 §8
  retirement/enactment gate is unchanged (no crate retires on a partial port during the window).

**Explicitly out of scope of the *decision*, IN scope of the *questions* (§6):** whether the **L0 Core IR**
(RFC-0001 — the deepest, trusted base of the layer stack) is itself unfrozen or stays the frozen floor
(OQ-1); and whether **surface grammar / projection expansion (L2/L3)** is covered (OQ-2). A Draft surfaces
these; it does not decide them.

### 2.2 What stays unchanged — the invariants the window does NOT relax

- **The ADR-042 §2.1(b) END-STATE is untouched.** Zero foreign first-party languages, kernel included, by
  the DN-88 gate remains the destination. This ADR lifts only the **NOW-horizon** freeze; it does not
  re-open, weaken, or re-time the end-state.
- **Never-silent, no black boxes (G2) is retained.** Every kernel/lexicon change in the window is reviewed,
  reified, `EXPLAIN`/`reveal`-able, and honestly tagged; a swap stays never-silent; out-of-range stays an
  explicit `Option`/error. The window relaxes *immutability*, not *transparency*.
- **The honesty lattice is retained (VR-5).** No guarantee tag is upgraded without a checked basis; the
  port differentials still earn `Empirical` and nothing is claimed `Proven` without a checked theorem.
- **The DN-39 boundary is not widened by this ADR.** Growing/optimizing the *implementation* of the kernel
  is distinct from *admitting a new component into the TCB*; the trusted-component **set** (DN-39 §7) is not
  enlarged by this decision. A genuine TCB-boundary change still routes through the DN-39 bar.
- **KISS/YAGNI (house rule #5).** The window licenses gap-closure and optimization; it is **not** a license
  to grow the kernel for ergonomics that lower to it — the DN-38 lowering law still says features lower to
  the kernel rather than enlarging it, save where a gap genuinely cannot be expressed above the kernel.

### 2.3 Rationale (grounded)

1. **Optimize while the surface is small (timing).** The kernel + lexicon are at their smallest and most
   coordinated *now* — one monorepo. Post-decomposition (DN-88) the same change is a many-repo lift.
2. **Close *measured* enabler gaps, not speculative ones.** The worklist is driven by the port itself: the
   `spw` runbook's "STOP and FLAG to `enb`" escalations and the transpiler's ≈ 0–8% `checked_fraction` are
   the empirical evidence of *which* gaps block mechanical porting. The window batches their closure.
3. **Serve the zero-hand-port north star.** Each closed grammar/kernel/runtime gap moves a hand-port into
   the transpiler's mechanical reach — the point of the exercise is fewer hand-ports later, not more kernel
   now.

### 2.4 Re-freeze conditions (the window's exit — all three, checked)

The window closes and the freezes re-declare **only** when **all** hold, each on a checked basis (VR-5):

1. **The named gap-closure worklist is exhausted.** A dated, enumerated worklist of the kernel/lexicon
   enabler gaps this window exists to close (seeded from the `spw`/`enb` FLAG stream and the transpiler
   gap-class profile) is complete — each item landed or explicitly re-scoped by a recorded decision.
2. **The DN-56 gate is re-scored green.** A **DN-76-successor scorecard** re-runs the DN-56 §5 five-condition
   freeze gate over the *new* kernel + lowering surface and returns green, assessed independently of the
   wave that closed the worklist (the DN-76 §5A independent-re-score discipline, guarding completion bias).
3. **The DN-39-only diff policy is reinstated by a follow-up maintainer-ratified ADR.** A successor ADR
   (or an amendment) re-declares the DN-56 §6 post-freeze diff policy in force, moving *this* ADR
   `Accepted → Enacted` (house rule #3). Only at that ratified step does the kernel/lexicon re-freeze.

Until all three hold, the window stays open — and honestly so; the kernel is not silently re-frozen, nor
silently held open forever (the §4 adversarial finding names indefinite-window creep as the primary risk;
§6 OQ-3 forces a bound at ratification).

### 2.5 Coordination — this ADR governs, it does not implement

Kernel/grammar/runtime **implementation** is the cloud CC session's **semcore self-hosting lane
(M-1013 / DN-26 Stage-5)** — `lib/compiler/semcore.myc` and the `mycelium-l1` frontend. This ADR is a
**governance record**: it sets the window, its scope, and its exit; it lands **no** kernel/lexicon code and
edits **no** `mycelium-core` / `mycelium-l1` / `lib/compiler/*` file. The **DN-76 scorecard** (the re-score
instrument's predecessor) is that lane's artifact and is **flagged, not edited** here. Cross-session
continuity rides `issues.yaml` + branches, never by editing the other lane's tree (CLAUDE.md Wave-N).

## 3. Consequences

- **The trusted base moves under an in-flight port.** The semcore port (M-1013) builds **on** the kernel;
  optimizing/growing the kernel during the port can shift the base beneath it. Mitigation: changes route
  through the semcore lane's discipline; the port is itself the gap-profiler that *drives* the worklist, so
  the two are co-designed rather than racing. This coupling is a coordination requirement, not a side note.
- **The "no black boxes by construction" guarantee reverts from structural to disciplinary — temporarily.**
  DN-56 §1's promise is *structural* once the kernel is frozen. During the window it holds by **discipline**
  (never-silent review + honest tags) rather than by **construction** (immutability). This is the real cost;
  it is bounded by §2.4 and is exactly the posture DN-56's own 2026-07-06 addendum already adopted
  pre-production (a discipline gate, per-change authorized).
- **DN-39 / RFC-0041 §6 / the 2026-07-06 per-change authorization remain available** and are the narrower,
  always-on channels; the window is the wider, time-boxed one that additionally licenses *new surface* for
  gap-closure at wave scale — which those narrower channels do not cleanly cover (§4).
- **A re-score instrument is owed.** §2.4 condition 2 requires a DN-76-successor scorecard; standing it up
  is follow-on work (owned by the semcore/freeze lane, not this ADR).
- **Cascade on acceptance (see §8).** On ratification, ADR-042/DN-56/DN-66 take their append-only status
  amendments, and a separate descriptive sweep + shared-memory update follows — none of it done while Draft.

## 4. Adversarial stress-test (VR-5 / house rule #4 — arguing against the recommendation)

A design-reasoner's job is to try to break its own recommendation. The recommendation (Option A, §5) is
that a **bounded standing unfreeze ADR** is the right formalization. The strongest disconfirming lines:

- **F1 — Is the unfreeze even *necessary*, or does Option B already suffice? (the central finding.)**
  DN-56 already admits, pre-production, a **per-change maintainer-authorization path** (2026-07-06) *and*
  the DN-39 promotion bar *and* the RFC-0041 §6 behavior-preserving channel. If the real enabler-gap count
  is **small**, a standing unfreeze is **YAGNI** (house rule #5) — each gap could land as a per-change
  authorization, keeping the freeze nominally intact. **Honest verdict:** the unfreeze's *marginal* value
  over Option B is real but **conditional** — it is worth it **only if** the gap worklist is wave-sized
  (many gaps, discovered continuously by the port) **and** the work includes **optimization that changes
  surface** and **new primitive/grammar/API surface**, which the per-change/DN-39/§6 channels cover only
  awkwardly at scale (DN-39 is one heavy dossier per *TCB admission*; §6 is behavior-preserving *only*;
  per-change auth is sound-*only*, one at a time). **This is a genuine disagreement surfaced, not
  smoothed:** if the maintainer's gap list is short, Option B is the KISS choice and this ADR is
  over-machinery. The maintainer should size the worklist (§6 OQ-3) before ratifying — the bound is what
  makes Option A defensible over Option B.
- **F2 — Indefinite-window creep is the primary failure mode.** "Temporary" freezes that lack a hard bound
  become permanent. Without §2.4's exhausted-worklist + re-score + re-freeze-ADR gate *and* an explicit
  duration/owner (OQ-3), the kernel simply stays open, and DN-56's structural guarantee never returns. The
  mitigation is only as strong as the maintainer's answer to OQ-3.
- **F3 — Moving the base under the semcore port could destabilize the very port it means to help (§3).**
  If kernel churn outruns the port's ability to track it, the differential thrashes. The co-design framing
  mitigates but does not eliminate this; it argues for a *narrow, worklist-driven* window over a broad one.
- **F4 — Append-only optics.** Unfreezing an `Enacted` freeze (DN-56 §9) can read as rewriting a landed
  decision. It is **not**, if done append-only: DN-56 is *superseded/amended forward*, never edited to say
  "unfrozen"; the freeze declaration stands in history and the window is a new, dated, forward decision that
  the re-freeze ADR later closes. The append-only discipline (house rule #3) is preserved **only** if the
  status transitions happen at ratification and via supersession — which is exactly why this session leaves
  ADR-042/DN-56/DN-66 statuses untouched while Draft.

**Stress-test verdict:** the unfreeze is **defensible but not unconditionally** — it is the right call
**iff** (a) the gap worklist is genuinely wave-sized (F1), and (b) the window is hard-bounded with a named
re-score owner (F2/OQ-3). If either fails, **Option B (per-change authorization, no new ADR) is the
honest, simpler choice.** This finding is put to the maintainer, not buried.

## 5. Alternatives considered — the ranked recommendation + objective table

**Objective function (criteria the formalization must satisfy):** append-only integrity (house rule #3);
never-silent / no-black-boxes preservation (G2); gap-closure enablement (the maintainer objective);
bound + reversibility; KISS/YAGNI (house rule #5); end-state fidelity (ADR-042 §2.1(b)).

| # | Option | Append-only | Never-silent (G2) | Gap-closure | Bounded/reversible | KISS/YAGNI | End-state fidelity | Verdict |
|---|---|---|---|---|---|---|---|---|
| **A** | **Bounded standing unfreeze ADR** (this ADR) — amend the three freezes for a named, time-boxed window; retain never-silent + DN-39-review; re-freeze via §2.4 | Yes (supersede fwd) | Retained | **Full** (batches wave-sized closure incl. new surface) | Yes (§2.4 + OQ-3) | OK **iff** worklist is wave-sized | Unchanged | **RECOMMENDED** |
| **B** | **No new ADR — per-change maintainer authorization** (DN-56 2026-07-06) + DN-39 promotions | Yes | Retained | Partial (per-change; sound-only; no batch surface) | Inherently (no window to close) | **Best** if gaps are few | Unchanged | **Runner-up — the honest choice if the gap list is short (§4/F1)** |
| **C** | **Permanent unfreeze / abandon the freeze** | No (discards a structural guarantee) | At risk | Full | **No** | — | At risk | Rejected — not the direction; discards DN-56 §1 by construction |
| **D** | **Narrow window — maintenance + behavior-preserving optimization only** (RFC-0041 §6 scope) | Yes | Retained | **Fails** (no new surface ⇒ no expressibility gap closed) | Yes | Simple | Unchanged | Rejected — misses the core objective |

**Recommendation: Option A, conditioned on OQ-1/OQ-2/OQ-3 (§6).** It is the only option that both enables
wave-scale gap-closure *and* preserves append-only integrity + never-silent discipline + a hard exit — **but
only if the window is genuinely bounded and the worklist genuinely wave-sized.** Where those conditions do
not hold, the ranked runner-up **Option B** is the correct, simpler choice, and this Draft says so plainly
(§4). The decision between A and B is the maintainer's, informed by the worklist size.

## 6. Open questions — FLAGged for maintainer ratification (a Draft surfaces; it does not decide)

- **OQ-1 — Is the L0 Core IR (RFC-0001) in scope for the unfreeze?** The layer stack is **L0 Core IR → L1
  kernel calculus (RFC-0007) → L2 surface → L3 projection** (RFC-0006 §3). §2.1 unfreezes the L1 prim/type
  set and the lexicon; it leaves **open** whether **L0 itself** is unfrozen or stays the **frozen trusted
  floor** while primitives/grammar *above* it grow. *Reasoner's non-binding lean (for the maintainer to
  accept or reject):* keep **L0 frozen** as the trusted base and unfreeze **L1-and-above** — L0 is the
  deepest content-addressing/metadata contract (RFC-0001 §4.5 well-formedness invariants), the most
  expensive to churn and the least likely to be the true site of a stdlib-port enabler gap. **Not decided
  here.** The maintainer fixes L0's status.
- **OQ-2 — Does the unfreeze explicitly cover surface grammar / projection expansion (L2/L3)?** Many
  expressibility gaps a port hits are **grammar** gaps (a construct the surface cannot yet spell), not
  kernel-primitive gaps. If L2/L3 grammar expansion is **in scope**, the window is the vehicle for it (and
  RFC-0037's grammar baseline / the DN-54 extension surface come into play). If **out of scope**, grammar
  gaps still route through the ordinary grammar-RFC path and only kernel/lexicon change under this window.
  **Not decided here.** The maintainer fixes the L2/L3 scope.
- **OQ-3 — The exact bound/duration of the window, and who re-scores the gate.** §2.4 fixes the *conditions*
  for re-freeze; it does **not** fix a **calendar/milestone bound** or name the **owner** who runs the
  DN-76-successor re-score and authors the re-freeze ADR. Per §4/F2 this is **load-bearing** — an unbounded
  "temporary" window is the primary failure mode. *Options to choose among:* (a) a milestone bound (e.g.
  "until the `spw` stdlib port wave + its `enb` enabler worklist complete"); (b) a scorecard bound (the
  window ends when the DN-76-successor re-scores green); (c) a calendar bound. **Not decided here.** The
  maintainer fixes the bound and names the re-score owner.

## 7. Definition of Done (house rule #6)

**For this ADR (the decision record):**

- [x] The maintainer's 2026-07-10 unfreeze direction recorded faithfully as **Draft**, with scope (§2.1),
  retained invariants (§2.2), rationale (§2.3), re-freeze conditions (§2.4), and coordination note (§2.5).
- [x] The exact clauses it proposes to amend cited with section numbers (header "Amends" row + §2.2/§8):
  ADR-042 §2.1(a) Decision (1); DN-56 §9 + §6; DN-66 §2 — with ADR-042 §2.1(b), DN-56 §8-equivalent DN-66
  §8, and the DN-39 boundary held unchanged.
- [x] The three scoping questions FLAGged, not decided (§6 OQ-1/OQ-2/OQ-3).
- [x] The adversarial stress-test run, including the case **against** the recommendation (§4), and a ranked
  recommendation with an explicit objective table (§5).
- [ ] **Maintainer ratification `Draft → Accepted`** — conditioned on the maintainer resolving OQ-1/OQ-2/OQ-3
  (L0 scope, L2/L3 scope, the window's bound + re-score owner). *This is the maintainer's step; this session
  does not take it.*
- [ ] On acceptance: the append-only cascade (§8) applied by the integrating parent.
- [ ] **`Accepted → Enacted` only when the §2.4 re-freeze conditions are met** (worklist exhausted +
  DN-76-successor scorecard green + a follow-up maintainer-ratified ADR reinstates the DN-39-only diff
  policy) — a checked basis, never ratification alone (house rule #3).

**For the policy it proposes (the standing gate, checked through the window):**

- [ ] Every kernel/lexicon change in the window is reviewed, `EXPLAIN`-able, and honestly tagged (G2/VR-5);
  the DN-39 boundary is not silently widened.
- [ ] The named gap-closure worklist is maintained (dated, enumerated) and each item lands or is re-scoped
  by a recorded decision — no silent scope drift.
- [ ] The window's bound (OQ-3) is respected and the re-freeze is executed on the §2.4 conditions.

## 8. Cascade on acceptance (post-ratification, coordinated — NOT done while Draft)

None of the below is performed by this Draft session (append-only discipline: statuses move at ratification,
via supersession, owned by the integrating parent / the relevant lane):

- **Bucket-A — append-only status amendments** to the superseded/amended decisions: ADR-042 (§2.1(a)
  Decision-1 amended-for-window), DN-56 (§6/§9 suspended-for-window), DN-66 (§2 unfrozen-for-window) —
  each a forward, dated changelog entry, **never a rewrite** of the decision body (house rule #3). *(A
  one-line append-only forward-reference footer pointing at this Draft has been added to ADR-042/DN-56/DN-66
  now — see §Grounding — as maintainer-authorized discoverability; the actual status amendment waits for
  ratification.)*
- **Bucket-B — descriptive sweep** (a separate coordinated PR): `docs/CURRENT-STATE.md`,
  `docs/Doc-Index.md`, `README.md`, and the ≈25 `mycelium-std-*` crate "DN-66 freeze" doc-comment headers —
  reworded to reflect the window. **Not edited now.**
- **Bucket-C — the DN-76-successor scorecard** stood up as the §2.4 re-score instrument (semcore/freeze
  lane; DN-76 itself flagged, not edited).
- **Bucket-D — shared memories** (`.claude/memory/*` freeze references) updated to point at the window.
  **Not edited now.**
- **Bucket-E — orchestrator-owned rows** (`CHANGELOG.md`, `docs/Doc-Index.md`, `tools/github/issues.yaml`
  ADR-045 registration + any minted worklist IDs): **FLAGged** to the integrating parent — not edited by
  this session.

## 9. Grounding / honesty

- Maintainer directive, 2026-07-10 (this session) — the unfreeze §2 records; `Declared` until `Accepted`.
- **ADR-042 §2.1(a) Decision (1)** (the NOW Rust-base freeze this amends at the NOW edge) and **§2.1(b)/§4**
  (the END-STATE + the enabler-classification tension, both held unchanged).
- **DN-56 §6** (post-freeze diff policy — DN-39-only / `core 2.0.0`), **§9** (the M-969 freeze declaration),
  and the **2026-07-06 discipline-gate addendum** (the per-change authorization path this generalizes).
- **DN-66 §2** (the stdlib stable-API "lexicon" freeze), **§8** (the enactment/retirement gate, unchanged).
- **DN-76** (the four-condition kernel-freeze scorecard — the predecessor of the §2.4 re-score instrument;
  **cloud SCC lane, flagged not edited**). **ADR-043** (retire-when-proven — unaffected).
- **kickoff `spw` §Mechanism + §runbook step 1** (transpiler `checked_fraction` ≈ 0–8%, M-991 / DN-34
  §8.7–§8.9; the "STOP and FLAG to `enb`" below-grammar-enabler escalation) — the *measured* enabler-gap
  basis for §2.3.
- **DN-88** (the component-repo decomposition horizon — the "hundred-module lift" §1/§2.3 pre-empts).
- **RFC-0006 §3** (the L0/L1/L2/L3 layer model behind OQ-1/OQ-2); **RFC-0001** (L0 Core IR — OQ-1's subject).
- **M-1013 / DN-26 Stage-5** (the semcore self-hosting lane §2.5 coordinates with; verified in `issues.yaml`
  read-only, 2026-07-10 — the STAGE-5 semcore increment in flight).
- House rules **#3** (append-only — supersede, never rewrite; Draft only), **#5** (KISS/YAGNI — the §4/F1
  necessity test), **G2** (never-silent, no black boxes — retained through the window), **VR-5** (no tag
  upgraded past its basis; the §4 disagreement not softened), **KC-3** (the small trusted base the DN-39
  boundary protects).

---

## Meta — changelog

- **2026-07-10 — Draft (design-reasoner session; branch `claude/adr-045-kernel-unfreeze`).** Records the
  maintainer's 2026-07-10 direction to **temporarily + boundedly UNFREEZE** the kernel primitive/type set
  (`mycelium-core`/`mycelium-l1`) and the stdlib public-API "lexicon" (the 26 `mycelium-std-*` crates) for
  an **early gap-closure/optimization window**, so the language closes its kernel/lexicon expressibility
  gaps now — before the DN-88 decomposition turns the same closure into a hundred-module lift — in service
  of the **zero-hand-port north star** (close grammar/kernel/runtime gaps so the transpiler can mechanically
  port what is hand-ported today). **Proposes to amend** ADR-042 §2.1(a) Decision (1) (NOW freeze lifted for
  the window; §2.1(b) END-STATE **unchanged**), DN-56 §9 + §6 (kernel-freeze + DN-39-only diff policy
  suspended for the window — generalizing the 2026-07-06 per-change discipline-gate authorization into a
  standing window), and DN-66 §2 (stdlib stable-API freeze unfrozen; §8 retirement gate unchanged). **Does
  not edit DN-76** (cloud SCC lane) or ADR-043 (retire-when-proven, unaffected). **Re-freeze conditions
  (§2.4):** named worklist exhausted + DN-56 gate re-scored green by a DN-76-successor scorecard + DN-39-only
  diff policy reinstated by a follow-up maintainer-ratified ADR (`Accepted → Enacted` gate). **FLAGs three
  scoping questions** for the maintainer (§6): L0 Core IR in scope? · L2/L3 surface grammar in scope? · the
  window's exact bound + re-score owner. **Adversarial stress-test (§4):** the unfreeze is defensible **iff**
  the gap worklist is wave-sized and the window is hard-bounded — else the simpler **per-change
  authorization (Option B)** is the honest choice; this disagreement is surfaced, not softened (house rule
  #4). **Status: Draft** — this session drafts; the maintainer ratifies. Enacts nothing; changes no other
  document's status. `docs/Doc-Index.md` / `CHANGELOG.md` / `issues.yaml` rows owned by the integrating
  parent (§8 Bucket-E). (VR-5 / G2 / house rules #3/#5.)
