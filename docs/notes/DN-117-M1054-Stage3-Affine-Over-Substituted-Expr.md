# Design Note DN-117 — M-1054 Stage 3 (OQ-H4): Affine Soundness over the Substituted Expr — Lift the Stage-3 Gate from Refuse-All-Affine to Accept-Linear / Refuse-Duplicated

| Field | Value |
|---|---|
| **Status** | **Accepted** (2026-07-11, delegated ratification — see the dated "Ratification (maintainer-delegated, orchestrator-selected on the merits, 2026-07-11)" section below, mirroring the M-1054 Stage 2 (DN-115) ratification precedent). **Accepted ratifies the §1–§7 design decisions (Rank 1/FLAG-A, the gate→trigger demotion, the drop-accept correction, the §5 test contract) and records the implementation leaf's own adversarial findings while landing it, NOT `Enacted`** (house rule #3: `Enacted` requires stepping through `Accepted` first and means *fully implemented/landed, outside ongoing maintenance* — a call for the integrating parent, not this leaf). Originally **Draft** (2026-07-11) — a design-reasoner build-plan scoping note; the author never moved status to `Accepted` (house rule #3). It does **not** move DN-110/DN-110-8.2 off `Accepted`, nor M-1054 off `in-progress`. |
| **Kind** | Design note (leaf-scoped, forward-to-recommendation). No code lands with this note — it scopes the Stage-3 implementation for a subsequent `/forward` leaf. |
| **Decides (proposes, for ratification)** | *For M-1054 Stage 3's scope only; all `Declared` unless a landed/checked basis is cited:* (1) **Q1 mechanism** — run the M-919 affine `Tracker` over the **substituted `Expr`** (each type-checked argument `Expr` spliced at every RHS occurrence of its value param), at **check time**, inside `Cx::check_sugar_call` — the L1-surface-AST layer the affine tracker already operates on (FLAG-A), **not** a post-elab L0 `Node` pass. Reconciles with Option B (DN-116): check still returns the call node for `Elab::app` to expand; the substituted `Expr` is a **check-time-only** artifact for the affine verdict. (2) **Q2 accounting** — reuse the existing `Tracker` (`use_at` + `snapshot`/`restore`/`merge_alt`) for linear-use counting, which already handles match arms / lets / lambdas by scope-index; **replace** the three conservative structural approximations (`ty_structurally_contains_substrate` part 1, `rhs_first_affine_binding`/`expr_is_structurally_affine` part 2) as the *accept/refuse decision*, retaining the cheap `ty_structurally_contains_substrate` predicate only as the *trigger* that decides whether the substituted-Expr walk is needed. (3) **Q3 orthogonality** — Stage 3 is orthogonal to Stage 2 (def-site resolution) and Stage 1 (`%`-freshening); the guarantee-tag ratchet moves the double-consume (**upper-bound**) property `Declared → Empirical` for the real substituted-Expr, **stays `Empirical`** (no `Proven` without a checked theorem), and the **drop lower-bound stays a runtime (M-904) concern, not a static refusal** (grounded correction of the task brief — see §4.3). (4) **Q5 scope** — a ≤~1–2k-LOC reviewable unit: the substituted-Expr affine re-check + gate relaxation + tests; cross-nodule affine stays **Stage 4** (DN-113/M-1060). |
| **Grounds in** | DN-110 §8.2 (D)/OQ-H4 (`Accepted` basis); DN-110-8.2-hygiene-deepdive §4(D), §7 E5, §10 OQ-H4; DN-116 (Stage-1b, Option B + §3.2 Stage-3 gate); DN-115 (Stage 2, orthogonality); M-919/DN-71 Model S §4.2 (the landed affine tracker); the E5 experiment `crates/mycelium-l1/src/tests/hygiene_affine_expanded.rs` (landed, `Empirical` for the upper bound). Read against `dev @ 3c7e85d7`. |
| **Guarantee posture** | `Empirical` where read against the codebase at `dev @ 3c7e85d7` (the cited functions, the E5 module, the landed tracker semantics); `Declared` for the proposed (unbuilt) Stage-3 mechanism and its soundness argument. No `Proven` claim anywhere. |

> **Note on DN-116 references in this file (renumber RESOLVED, 2026-07-11 integration close-out).**
> "DN-116" below means `docs/notes/DN-116-M1054-Stage1b-Check-Phase-Sugar-Accept.md` (the M-1054
> Stage-1b note) — originally filed as DN-114, which collided with the unrelated
> `docs/notes/DN-114-Validated-Narrative-Generation.md` (kept as DN-114 per maintainer directive). The
> renumber to **DN-116** landed at this close-out (file moved; docs-only inbound refs, including this
> note's, updated in the same pass — see M-1054's issue body and DN-115 §10). This note was authored
> as **DN-117** because DN-116 was reserved for that renumber (verified free at the time: no `DN-117`
> in `docs/notes/`, `docs/Doc-Index.md`, `tools/github/issues.yaml`, or `CHANGELOG.md` at
> `dev @ 3c7e85d7`). `crates/mycelium-l1/` source/test comments citing the old "DN-114" spelling for
> this Stage-1b note were repointed to DN-116 in the same Stage-3 landing pass (the residual the
> renumber note below once deferred).

---

## §1 Verify-first — premises confirmed against the codebase (mitigation #1 + #14)

Before scoping, each premise of the design question was checked against `dev @ 3c7e85d7`; the tracker
(`issues.yaml`) is `Declared`, the code is ground truth (VR-5).

1. **The Stage-3 gate refuses all affine, wholesale.** `Cx::check_sugar_call`
   (`crates/mycelium-l1/src/checkty.rs:5576`) enforces two Stage-3 (OQ-H4) refusals:
   - **Part 1** (`checkty.rs:5605`): for each value param, `ty_structurally_contains_substrate`
     (`checkty.rs:5740`) — a recursive walk into registered `Data` ctor fields (with `subst_ty` over
     the type's own args, on-path cycle-cut via a `visiting: BTreeSet<String>`) — refuses any value
     param whose type **is or structurally contains** `Substrate`.
   - **Part 2** (`checkty.rs:5643`): `rhs_first_affine_binding` (`checkty.rs:6012`) +
     `expr_is_structurally_affine` (`checkty.rs:6134`) refuse an RHS-local `let`-bound affine
     acquisition (a `fn` whose *declared* return type resolves to `Substrate`, or a bare `consume`).

   Both are conservative **structural over-approximations**: they refuse the *possibility* of a
   double-consume without ever counting actual uses over the substituted expansion.

2. **The reason: the affine tracker never sees the substituted expansion.** `check_sugar_call`
   returns `app_node(head, rebuilt)` — a **call node** (Option B, DN-116 §2) — and the actual
   splicing of the argument node at each RHS occurrence happens later, in
   `elab.rs::sugar_expand` (`crates/mycelium-l1/src/elab.rs:921`) / `elaborate_value_parametric_rule_inner`
   (`elab.rs:799`), at **elaboration**, which runs **after** check. So at check time there is no
   substituted term to affine-check; the gate refuses affine wholesale rather than "silently accept a
   splice it cannot verify" (G2). This is stated verbatim in `infer_expr_rule_rhs_type`'s own doc
   comment (`checkty.rs:2727`–`2734`): the seeded tracker there "is the M-919 *upper-bound* check
   only — it does not by itself make a *call site* with a `Substrate`-containing value parameter safe
   to accept … see `Cx::check_sugar_call`'s own Stage-3 (OQ-H4) gate."

3. **The affine tracker is an L1-`Expr` pass, keyed on `Ty` — it cannot run at L0.** `Tracker`
   (`crates/mycelium-l1/src/affine.rs:130`) is index-lockstep with the checker's `scope`, seeded via
   `Tracker::seeded` (`affine.rs:151`), and its `Slot::for_ty` (`affine.rs:82`) keys `Live`/`Skip` on
   `Ty::Substrate`. L0 `mycelium_core::Node` is **untyped** (`mycelium-core/src/node.rs:101`, cited by
   the E5 module) — there is no type at L0 to tell a `Substrate{Sock}` binder from any other, so
   **there is no affine pass at L0 and cannot be one without re-deriving types**. This is the load-bearing
   fact that decides Q1.

4. **E5 is landed and is a preview of the production wiring.**
   `crates/mycelium-l1/src/tests/hygiene_affine_expanded.rs` (declared in `src/tests/mod.rs:27`)
   realizes "affine soundness on the expanded L0" as a **fully-expanded SOURCE program** — the RHS
   with the formal parameter's occurrences substituted by the argument's surface spelling — fed
   through the **real** `check_nodule`/`check_fn_body`/`Tracker` pipeline. Its module doc (lines
   32–38) states this "likely **previews how the real M-1054 facility will have to be wired**:
   `check_lower_rule_rhs_type` already type/affine-checks a rule's RHS as an L1 surface expression, and
   any future value-parameter substitution M-1054 adds would most naturally **re-run that same
   L1-level pass on the substituted RHS**, exactly the shape this module exercises — not a parallel
   Node-level pass that does not otherwise exist in the codebase." **This is the mechanism this note
   recommends.** (Correcting the task brief's "E5 was Empirical … unbuilt" framing: E5 *is* built and
   green, `Empirical` for the upper bound only — see §4.3.)

**Premise confirmed:** the design question is well-posed and not a re-decision of landed work. Stage 3
is genuinely unbuilt; the two structural gates are exactly the seam to relax.

---

## §2 Q1 — What gets affine-checked, and where (ranked recommendation)

**Objective function.** Minimize new machinery (KISS/DRY, KC-3); operate at the layer where the
landed M-919 tracker already works (no new pass); preserve Option B (no double-expansion); catch
genuine duplication precisely (accept linear, refuse duplicated); stay never-silent (G2); keep the
honesty tag exactly at its checked basis (VR-5).

| Criterion (weight) | Rank 1 — substitute-then-affine-check at check time over the substituted `Expr` (FLAG-A) | Rank 2 — affine-check the expanded L0 `Node` post-elab | Rank 3 — refine the def-time structural over-approximation (no real tracker) |
|---|---|---|---|
| New machinery (KISS/KC-3) | **Low** — reuse `Tracker`, `Cx::check` walk, `subst`-style splice | High — a new L0-level affine pass that does not exist | Low, but re-implements linear counting badly |
| Correct layer (§1.3) | **Yes** — tracker is L1-`Expr`, keyed on `Ty` | **No** — L0 `Node` is untyped; needs type re-derivation | Yes (L1) but no real counting |
| Preserves Option B | **Yes** — returns call node; substituted `Expr` is check-time-only | Requires check to consume elab output (layering inversion: check runs before elab) | Yes |
| Precision (accept-linear / refuse-duplicated) | **Full** — real use-count over the substituted term | Full (if built) | **None** — still refuses whole classes conservatively |
| Honesty / never-silent | **Yes** — double-consume diagnostic reused verbatim | Yes | Weaker — over-refuses, "refuse the possibility" |
| E5 precedent | **Directly previewed** (§1.4) | Contradicted by E5's own "not a parallel Node pass" finding | N/A |

### Rank 1 (RECOMMENDED) — substitute-then-affine-check the substituted `Expr` at check time

Inside `Cx::check_sugar_call`, after arity + per-argument type checks produce the `rebuilt` argument
`Expr`s (`checkty.rs:5628`–`5640`):

1. **Build the substituted RHS `Expr`** — splice each `rebuilt[i]` `Expr` at every occurrence of value
   param `rule.value_params[i].name` in `rhs`, the **Expr-level analogue** of `elab.rs::sugar_expand`'s
   (B) substitution (which operates on L0 `Node`s). This is a pure `Expr → Expr` walk that respects
   shadowing (an occurrence bound by an inner `let`/`match`/`lambda` binder of the same spelling is
   **not** substituted — reuse `sugar_expand`'s own unshadowed-occurrence logic, `elab.rs:949`).
2. **Affine-check the substituted `Expr`** with an **active `Tracker`** seeded from the value-param
   scope (as `infer_expr_rule_rhs_type` already does at `checkty.rs:2773`), walking it via the existing
   `Cx::check` recursion so every `use_at`/`snapshot`/`restore`/`merge_alt` fires exactly as in an
   ordinary function body. Accept iff no `UseOutcome::DoubleUse` is produced; on a double-use, refuse
   with the **existing** DN-71 both-sites `double-consume` diagnostic.
3. **Return the call node unchanged** (`app_node(head, rebuilt)`, `checkty.rs:5703`) — Option B is
   untouched; `Elab::app` still expands via the unmodified Stage-1 machinery. The substituted `Expr` is
   a **check-time-only** artifact, discarded after the affine verdict; elaboration re-derives the
   expansion independently (no double-expansion, DRY).

**Why this is correct for both the top-level and the composite case** (the DN-116 §3.2 review
finding): the affine identity lives in the **argument expression**, not the parameter's declared type.
For a top-level `p: Substrate` used twice, checking the *unsubstituted* RHS would already catch it (`p`
is a `Live` slot referenced twice); but for a composite `h: Handle` (a `Data` wrapping `Substrate`)
used twice with per-occurrence extraction, `h`'s slot is `Skip` (`Handle` is not `Substrate`-typed at
the binding level), so the unsubstituted walk misses it — the double-consume only manifests when the
caller's single argument `Wrap(consume s)` is spliced at both occurrences, yielding `consume s` twice.
And for the **affine-hiding-non-affine-type** case (`p: Int`, argument `let _ = consume s in 0`, used
twice), the parameter type does not structurally contain `Substrate` at all, so *no type-based gate*
can catch it — only the substituted-term walk sees the duplicated `consume s`. The substituted-`Expr`
walk is the single mechanism that is correct for **all three** classes; the structural gates are all
partial approximations of exactly it.

**Reconciliation with DN-110-8.2 (D)'s "expanded L0" language and FLAG-A.** DN-110 §8.2 (D) and E5's
title say "checked on the **expanded L0**." That is loose: L0 `Node` is untyped (§1.3), so the affine
tracker cannot run there. **FLAG-A is the accurate refinement** — the real production layer is the L1
check-phase tracker over the **substituted `Expr`**, semantically equivalent for affine purposes (same
use-count structure as the expanded L0) but at the only layer the tracker can operate. E5's own module
doc reaches the identical conclusion (§1.4). This note adopts FLAG-A as the mechanism and records the
"expanded L0" phrasing in DN-110-8.2 (D) as an accepted-but-imprecise antecedent (no supersede needed:
DN-110-8.2 stays `Accepted` — the *design intent* "checked on the expanded term, never laundered by the
sugar" is unchanged; only the *layer* is pinned by this note).

### Rank 2 (rejected) — affine-check the expanded L0 `Node`

Matches DN-110-8.2 (D)'s literal phrasing but requires either a brand-new L0-level affine pass (which
does not exist and would duplicate M-919 — DRY/KISS violation) or feeding elab's output back into
check (a layering inversion: check runs before elab). Contradicted by E5's explicit "not a parallel
Node-level pass" finding. Rejected.

### Rank 3 (rejected) — refine the structural over-approximation

Keep a def-time syntactic check but make it "smarter" (e.g. count syntactic occurrences without a real
tracker). This is what the current gate does; it cannot achieve accept-linear/refuse-duplicated
precision (it has no notion of the argument's affinity, no branch-merge for match arms) and would
re-implement linear counting worse than the landed `Tracker`. Rejected — it is the very thing Stage 3
replaces.

---

## §3 Q2 — Linear-use accounting, and the disposition of the structural gate

### 3.1 The accounting *is* the existing `Tracker` walk

The M-919 `Tracker` already implements linear-use accounting over arbitrary `Expr` shapes, and it
already composes with the composite/pattern cases the Stage-1b review flagged:

- **Sequential use** — `use_at` transitions `Live → Moved`; a second use on a reachable path yields
  `UseOutcome::DoubleUse { first_ordinal, this_ordinal }` (`affine.rs:206`), refused with the
  both-sites diagnostic. Exactly-once ⇒ `FirstUse` ⇒ clean.
- **`match` arms** — `snapshot`/`restore` fork the pre-arm state; `merge_alt` union-merges (`Moved` in
  *either* branch ⇒ `Moved` after, `affine.rs:112`/`199`). So an affine value consumed in **one** arm
  is fine; consumed in the **scrutinee and an arm**, or **twice in one arm**, is a double-use;
  consumed in **two mutually-exclusive arms** is **not** a false double-use (correct linear semantics).
  This is precisely the composite/pattern case (`match h { Wrap(s) => consume s }` at two occurrences)
  that part 1's structural gate could only refuse wholesale.
- **`let` / `lambda` / `for` binders** — pushed/popped in lockstep with `scope` (`Tracker::push`/`pop`/
  `truncate`), so an RHS-local `%`-fresh `Substrate` binder is tracked **independently** of any
  use-site binder (**by scope index, not by name** — `affine.rs` module docs; the E5 module's
  `fixture_independent_rhs_fresh_binder` proves the no-false-positive independence).

Nothing new is needed for the counting itself — Stage 3 feeds the **substituted** `Expr` to the same
walk, where Stage 1b fed the **unsubstituted** RHS.

### 3.2 `ty_structurally_contains_substrate` — repurposed as a trigger, retired as the decision

The recursive structural gate is **replaced as the accept/refuse decision** by the substituted-`Expr`
linear count (strictly more precise: it accepts a composite-typed value param whose actual argument is
not affine-duplicating, and refuses only genuine duplication). It is **retained only as a cheap
trigger**: if the rule has *any* affine surface (any value param for which
`ty_structurally_contains_substrate` holds, **or** any RHS-local affine binding), run the
substituted-`Expr` walk; otherwise the affine-free fast path is unchanged and the substituted walk is
skipped (the current cheap path stays cheap — no regression for the affine-free fragment).

**Do not use an occurrence-count-only fast path as the *soundness* criterion.** A tempting optimization
— "if every value param occurs ≤1 time, skip" — is **unsound** for (a) a composite affine argument
consumed inside a `for` loop with a single syntactic occurrence, and (b) cross-argument aliasing (the
same `s` passed to two params each used once). The safe rule is: **when any affine surface is present,
always build and walk the substituted `Expr`.** The occurrence-count filter may later be added *only*
for the provably-safe "≤1 occurrence of every value param **and** no RHS-local affine binding" case, as
a pure performance refinement — flagged as a follow-up, not part of the Stage-3 correctness path.

### 3.3 Net simplification

`rhs_first_affine_binding` + `expr_is_structurally_affine` (part 2) become **subsumed** by the
substituted-`Expr` walk: an RHS-local `let`-bound affine acquisition used twice is caught by the real
tracker directly (it was already caught for the *unsubstituted* RHS by `infer_expr_rule_rhs_type`'s
seeded tracker; the substituted walk is a superset). So Stage 3 is a **net DRY win** — three structural
approximations (part 1's decision + part 2's two helpers) collapse into one real affine pass, with the
part-1 predicate demoted to a trigger. (Whether to physically delete the part-2 helpers or keep them as
an internal assertion is an implementation choice for the `/forward` leaf — this note recommends
deletion for KISS, with the E5 corpus + new tests as the safety net.)

---

## §4 Q3 — Interaction, orthogonality, and the guarantee-tag ratchet

### 4.1 Orthogonal to Stage 1 (`%`-freshening)

Stage 1's per-expansion `%`-namespace freshening (`Elab::fresh`) makes rule binders disjoint from
use-site argument free variables. The affine `Tracker` tracks **by scope index, not by name**
(`affine.rs` module docs; `checkty.rs:3883` field doc), so a `%`-fresh RHS `Substrate` binder and a
use-site `Substrate` argument occupy **distinct slots** and are counted independently *regardless of
spelling* — no double-count, no false positive. E5's `fixture_independent_rhs_fresh_binder` (accept)
vs `_duplicated` (reject) is the landed non-vacuous witness. Orthogonal: freshening guarantees name
disjointness; the affine walk then counts correctly over the substituted term.

### 4.2 Orthogonal to Stage 2 (def-site resolution)

Stage 2 (DN-115, single-nodule) already refuses genuinely free RHS identifiers before this point
(`rhs_first_free_id`, `checkty.rs:5654`), so the substituted `Expr` contains only value-param
arguments, resolved def-site references, and RHS-local binders — a resolution-clean term. The affine
walk operates over that; it neither resolves names nor re-decides Stage 2. Cross-nodule/phylum affine
is **Stage 4** (DN-113/M-1060), out of scope here — the substituted-`Expr` walk stays within the
single-nodule fragment Stage 1b/Stage 2 established.

### 4.3 Guarantee-tag ratchet (VR-5)

- **E5 today:** `Empirical` for the **upper-bound** (duplication) property over the surface-substitution
  model, via the real M-919 static pass; **not** validated as a static claim for the **drop lower
  bound**.
- **Stage 3 production:** the double-consume rejection over the real `Expr`-level substituted term
  moves `Declared → Empirical` once the §5 tests are green (property + differential against a
  hand-oracle at DN-20 LOW/HIGH). It **stays `Empirical`** — there is **no checked theorem** of linear-
  type soundness for the sugar expansion (DN-110-8.2 §8's soundness argument is explicitly `Declared`),
  so **no `Proven` claim is admissible** without a future Lean/checked discharge (out of scope; a
  standing proof obligation, flagged not built).
- **The drop case is ACCEPT, not REJECT — a grounded correction of the task brief (VR-5, mitigation
  #14).** The task brief (and E5's original commissioning brief) assumed a dropped/unused affine value
  param is a *rejected* affine violation. **It is not**: the landed M-919 static pass enforces only the
  **use-at-most-once upper bound**; the **must-consume lower bound is closed at runtime** (M-904
  `release_if_abandoned`/`SubstrateHandle::release`; DN-71 §8 FLAG-4), grounded directly in
  `crates/mycelium-l1/src/tests/affine.rs:304`–`312`
  (`a_never_consumed_substrate_binding_checks_the_static_pass_does_not_reject_leaks`) and re-derived
  through E5's `e5_drop_case_grounding_matches_the_landed_static_posture`. So Stage 3 must **accept** a
  linearly-used-*or-dropped* affine value param and **refuse only duplication** — matching hand-written
  code's own static posture exactly. A Stage-3 gate that *rejected* drops would be **stricter than the
  language itself**, a false-positive regression. This is the sharpest honesty correction in this note.
- **Dynamic multiplicity (loops) stays runtime, unchanged.** The static tracker walks a `for` body
  **once** (`checkty.rs:6505`–`6506` push/pop around a single body walk), so `consume p` inside a loop
  is a single *static* use; runtime multiplicity is the M-904 backstop's job, identically for
  hand-written and sugar-expanded code. Stage 3 inherits this posture faithfully — it is **not** a new
  gap.

---

## §5 Q4 — Non-vacuous test plan (the E5 lesson: every verdict is an independent hand-oracle)

Home: extend `crates/mycelium-l1/src/tests/reachability_stage1b.rs` (the Stage-3 controls
`control_affine_value_param_hits_stage3_residual` / `control_affine_rhs_local_binding_hits_stage3_residual`
live there and must be **superseded** — see below) and/or a new sibling `stage3_affine_substituted.rs`,
white-box in-crate per house-rule test layout. Reuse the E5 helpers (`is_double_consume`, the
`build_source`/`check` shape) for DRY.

**Acceptance fixtures (the lift — previously all refused):**

- **A1 — linear top-level accept.** `use1(p: Substrate) = consume p`, invoked with a use-site
  `Substrate` argument → **ACCEPT**, and the value is consumed exactly once (assert the accepted call
  node types and that no double-consume fires). *This is the behavior change: previously part 1 refused
  it wholesale.*
- **A2 — linear composite accept.** `once(h: Handle) = match h { Wrap(s) => consume s }` where
  `Handle` wraps `Substrate`, invoked with a `Wrap(consume s)`-shaped argument used **once** →
  **ACCEPT**. *Previously refused by the recursive part-1 gate.*
- **A3 — drop accept (the corrected verdict).** `drop0(p: Substrate) = 0` (param unused) → **ACCEPT**
  (grounded in §4.3; the static pass does not reject drops). Assert ACCEPT, not REJECT — a control that
  the gate is not stricter than the language.

**Refusal fixtures (genuine duplication still caught):**

- **R1 — top-level double-consume.** `dup(p: Substrate) = (consume p, consume p)` → **REFUSE** with the
  DN-71 both-sites `double-consume` diagnostic (`is_double_consume`).
- **R2 — composite double-consume via two pattern matches.** `dup2(h: Handle) = (match h { Wrap(s) =>
  consume s }, match h { Wrap(s) => consume s })` invoked with a single affine `Handle` argument →
  **REFUSE** (the exact DN-116 §3.2 review case; the substituted term consumes the extracted field
  twice). *This is the case the structural gate could only refuse by over-approximating; here it is
  refused for the right reason and A2 shows a sibling composite is accepted.*
- **R3 — affine-hiding non-affine type.** `dupI(p: Int) = (p, p)` invoked with `let _ = consume s in 0`
  → **REFUSE** (double-consume of `s`), proving the mechanism is not type-gated (no structural gate
  could catch this).
- **R4 — cross-argument aliasing.** a two-param rule `pair(a, b) = …a…b…` invoked as `pair(consume s,
  consume s)` → **REFUSE**. (Note: this may already be caught at the call site by the *outer* tracker
  walking the argument list — assert *which* layer refuses it, and keep it as a regression either way.)
- **R5 — match-arm independence is NOT a false positive.** `pick(h: Handle) = match cond { True =>
  match h { Wrap(s) => consume s }, False => 0 }` → **ACCEPT** (consumed in at most one arm), guarding
  against a naive occurrence-counter that would false-reject. This is the branch-merge correctness
  control.

**Non-vacuity controls (the E5 discipline — a broken linear count must genuinely leak):**

1. **Mutation flips the verdict, both directions:** A1→R1 (argument used once → twice) flips
   ACCEPT→REFUSE; a fixture pair where only an RHS-own `%`-fresh binder's use-count changes also flips
   (mirroring E5's `e5_mutation_flips_verdict_on_rhs_own_binder_duplication`). A checker that always
   accepted or always rejected fails at least one direction.
2. **Sabotage control:** temporarily disable the substitution (feed the *unsubstituted* RHS to the
   walk, or seed an **inert** tracker) and assert R2/R3 then **false-accept** — proving the substituted
   walk is load-bearing, then restore. (This is the exact non-vacuity method DN-116 §3.2 used for the
   composite gate: revert-to-old, confirm false-accept, restore.)
3. **Every REFUSE asserts the *specific* `double-consume` diagnostic** (`is_double_consume`), never
   "failed for some reason" — so a refusal for an unrelated cause cannot masquerade as a pass.

**Superseded controls (append-only, in tests too).** `control_affine_value_param_hits_stage3_residual`
and `control_affine_rhs_local_binding_hits_stage3_residual` (`reachability_stage1b.rs:502`/`555`)
currently assert the *wholesale-refusal* behavior and its "Stage 3"/"OQ-H4" residual diagnostic. Under
Stage 3 these must be **rewritten to the new contract** (linear→accept, duplicated→refuse) — the
diagnostic changes from "Stage 1b accepts only the affine-free fragment / not built yet" to the
`double-consume` refusal. Document the change in the test as a Stage-3 supersede, not a silent flip.

---

## §6 Q5 — Scope, size, and the clean cut

**In scope (one ≤~1–2k-LOC reviewable unit; DN-65/DN-97 soft target, 4,000-LOC hard cap):**

- `checkty.rs`: the substituted-`Expr` builder (Expr-level analogue of `sugar_expand`'s (B),
  shadowing-aware); the affine re-check over it inside `check_sugar_call`; the relaxation of the
  part-1 gate from refusal to trigger; the removal/demotion of part-2 helpers; the diagnostic wiring
  (reuse the DN-71 both-sites message).
- Tests: the §5 acceptance/refusal/non-vacuity corpus; the two superseded controls rewritten.
- Docs (FLAGged to the integrating parent, not edited from this leaf): DN-116 §3.2 gains an
  append-only "superseded by Stage 3 (DN-117)" pointer; DN-110-8.2 OQ-H4 marked resolved (use-time
  precise, per its own recommendation); `CHANGELOG.md` + `Doc-Index.md` + `issues.yaml` M-1054 status
  rows.

**Out of scope (explicitly, never-silent):**

- **Cross-nodule / cross-phylum affine → Stage 4** (DN-113 / M-1060). The substituted-`Expr` walk stays
  within the single-nodule fragment; a value param whose affine identity crosses a nodule boundary is
  not reachable here (Stage 2 already refuses cross-nodule free identifiers).
- **Dynamic (loop/recursion) multiplicity → M-904 runtime backstop** (unchanged; §4.3).
- **The drop lower bound → M-904 runtime** (unchanged; §4.3).
- **Lambda-captured-and-called-twice affine** — if the tracker's current handling of an affine value
  captured by a lambda invoked multiple times is not already sound for hand-written code, Stage 3
  inherits that exact posture (no better, no worse). **FLAG** to confirm during implementation; do not
  claim Stage 3 closes it.
- **Occurrence-count performance fast path** — deferred (§3.2), sound only for the ≤1-occurrence +
  no-RHS-affine-binding case.
- **A `Proven` linear-soundness theorem** — a standing proof obligation, not this unit.

---

## §7 Adversarial stress-test (VR-5 / house rule #4 — argue against the recommendation)

1. **"Just check the unsubstituted RHS — the tracker already seeds value params."** Breaks on the
   **composite** case (§2 Rank 1): `h: Handle`'s slot is `Skip`, so referencing `h` twice is invisible
   to the tracker; only substitution surfaces the duplicated `consume s`. Also breaks on the
   affine-hiding-non-affine-type case (R3). The unsubstituted walk is *necessary but not sufficient* —
   substitution is load-bearing. **Recommendation survives.**
2. **"The substituted `Expr` duplicates argument *evaluation* — is that a new hazard?"** No: value
   params are substituted (not `let`-bound) **by design** (DN-116 §3.2 line 119); duplicating a *pure*
   value's evaluation is re-evaluation, not a soundness hazard in v0, and duplicating an *affine*
   argument's evaluation is exactly the double-consume the tracker now catches. The affine subset is
   the only unsound subset, and it is precisely the subset the walk refuses. **Survives.**
3. **"Rejecting drops would be safer."** This is the trap the task brief fell into. Rejecting drops
   would make the Stage-3 gate **stricter than the language's own static contract** (§4.3), a
   false-positive regression against `affine.rs:304`–`312`. The must-consume bound is M-904's runtime
   job. Following the evidence over the brief: **accept drops.** (Sycophancy would have been to encode
   the brief's assumed REJECT; the codebase says ACCEPT.)
4. **"Occurrence-count fast path is a cheap win."** Unsound for loops-with-single-occurrence and
   cross-argument aliasing (§3.2). Rejected as a correctness mechanism; admissible only as a
   narrowly-guarded perf refinement later. **Survives — and the note is stricter than the tempting
   shortcut.**
5. **"Why not follow DN-110-8.2's literal 'expanded L0'?"** Because L0 is untyped (§1.3) and the
   tracker is keyed on `Ty`; the literal reading is unimplementable without a second type derivation.
   FLAG-A and E5's own finding both land on the L1 substituted-`Expr` layer. **Survives** — and this
   note pins the layer DN-110-8.2 left imprecise.

**Verdict:** the recommendation holds under every sequence tried, and is *stricter* than the brief in
two places (accept-drops, no occurrence-count shortcut) where the evidence cuts against the sketched
direction.

---

## §8 Definition of Done — what "Accepted" requires of the maintainer (house rule #6)

This note is `Draft`. To ratify it to `Accepted` (design ratification only; **not** `Enacted` — no
code lands with the note), the maintainer/orchestrator confirms:

1. **Q1 mechanism** — the L1 substituted-`Expr` affine check at check time (Rank 1 / FLAG-A) is the
   chosen mechanism, and Rank 2 (L0 `Node` pass) is recorded rejected. (Or the maintainer names a
   variant.)
2. **Q2 disposition** — `ty_structurally_contains_substrate` demoted to a trigger; part-2 helpers
   subsumed; **no** occurrence-count fast path on the correctness path.
3. **Q3 drop verdict** — the drop case is **ACCEPT** (runtime-backstopped), explicitly ratifying the
   correction of the task brief; the guarantee tag stays `Empirical` (no `Proven`).
4. **Q4 test contract** — the §5 acceptance/refusal/non-vacuity corpus (incl. the sabotage control and
   the two superseded controls rewritten) is the acceptance bar for the implementation leaf.
5. **Q5 boundary** — cross-nodule affine is Stage 4; the lambda-capture and loop-multiplicity items are
   FLAGged out, not silently included.
6. **Enactment gate (separate, later):** Stage 3 is `Enacted` only when the implementation leaf lands
   green (`just check`), the tag has moved to `Empirical` on the real substituted-`Expr`, DN-116 §3.2
   carries the append-only supersede pointer, and DN-110-8.2 OQ-H4 is marked resolved.

---

## §9 FLAGs (for the integrating parent — append-only, dated; not edited from this leaf)

- **FLAG-DocIndex (2026-07-11):** add a `docs/Doc-Index.md` row for **DN-117** (this note) so
  `corpus:DN-117` `doc_refs` resolve. Orchestrator-owned; not edited here.
- **FLAG-CHANGELOG (2026-07-11):** add a `CHANGELOG.md` entry recording DN-117 (Draft) under the
  design-phase framing. Orchestrator-owned; not edited here.
- **FLAG-issues (2026-07-11):** M-1054's issue body / a new Stage-3 sub-issue should reference DN-117
  as the Stage-3 build-plan basis, and (when the implementation leaf runs) `depends_on` M-919 (landed) +
  the Stage-2 M-1069 close-out. Orchestrator-owned; not edited here.
- **FLAG-DN114-supersede (2026-07-11):** DN-116 §3.2 (the wholesale Stage-3 gate) needs an
  append-only "superseded-in-part by Stage 3 / DN-117 (accept-linear, refuse-duplicated)" pointer when
  Stage 3 lands — house rule #3, supersede-don't-rewrite.
- **FLAG-DN110-8.2 (2026-07-11):** DN-110-8.2 §10 OQ-H4 should be marked **resolved** (use-time
  precise, per its own §10 recommendation) at Stage-3 enactment; and (D)'s "expanded L0" phrasing gets
  an append-only pointer to FLAG-A / DN-117 §2 pinning the layer as the L1 substituted-`Expr`.
- **FLAG-renumber (2026-07-11, RESOLVED same day):** this note's "DN-116" refs are to the M-1054
  Stage-1b note, renumbered from DN-114 to **DN-116** at the 2026-07-11 integration close-out; this
  note's inbound refs were folded into that same renumber pass. `crates/mycelium-l1/` source/test
  comments citing the old "DN-114" spelling were repointed to DN-116 in this Stage-3 landing's own
  pull-down merge (no longer deferred).
- **FLAG-lambda-affine (2026-07-11, open question):** confirm during implementation whether an affine
  value captured by a lambda invoked multiple times is soundly tracked for hand-written code; Stage 3
  inherits that posture and must not over-claim (§6).

---

## Ratification (maintainer-delegated, orchestrator-selected on the merits, 2026-07-11)

**Recorded decision (append-only — this note's original §0–§9 text above is unchanged; this section
adds the ratification, per house rule #3, mirroring the M-1054 Stage 2 (DN-115) ratification
precedent).** The recommended design (Rank 1 / FLAG-A) was selected on the merits stated in §1–§7;
this section records that selection **plus** the implementation leaf's own adversarial findings
while landing it (VR-5 — grounded, not merely asserted).

1. **Q1 mechanism (Rank 1 / FLAG-A) — ACCEPTED and LANDED as specified.** The check-time-only
   substituted `Expr` (shadowing-aware (B) splice) is built inside `Cx::check_sugar_call` and walked
   by the real M-919 `Tracker`, seeded/scoped as DN-117 directed; the call node returned to `Elab::app`
   is unchanged (Option B intact); `elab.rs::sugar_expand` is byte-unmodified. Rank 2 (an L0 `Node`
   pass) stays rejected, for the reasons §2 already gives.
2. **Q2 disposition — ACCEPTED and LANDED, with the trigger widened beyond this note's own §3.2
   literal text (a grounded refinement, not a guess — see finding 3 below).**
   `ty_structurally_contains_substrate` is demoted to a trigger input (never the decision); the
   part-2 helpers (`rhs_first_affine_binding`/`expr_is_structurally_affine`) are **deleted**, per
   §3.3's own KISS recommendation — their job is subsumed by `infer_expr_rule_rhs_type`'s existing,
   unconditional, unsubstituted-RHS walk (already active there since M-919), which the R3 finding
   below confirms is unaffected. No occurrence-count fast path was added to the correctness path.
3. **ADVERSARIAL FINDING (HIGH, fixed in the same leaf) — §3.2's literal trigger text
   under-specifies R3; the implementation widens it, not narrows it.** §3.2 names exactly two trigger
   conditions ("any value param for which `ty_structurally_contains_substrate` holds, **or** any
   RHS-local affine binding"), both keyed on the **rule's own static shape**. R3 (§5, "affine-hiding
   non-affine type": an `Int`-typed param whose *argument* carries a side-effecting `consume`)
   defeats a purely rule-shape-keyed trigger by construction — no static fact about the *rule*
   distinguishes it from an ordinary, safe `Int` param. **Fix (landed):** the trigger is the OR of
   (a) §3.2's own type-based condition and (b) a new, *exact* (not approximate) per-argument signal —
   did checking this argument, as actually typed, touch the affine tracker at all (any slot's
   Live/Moved state changed)? This is a strict superset of §3.2's own text (never narrows what it
   already covered) and is a natural, near-zero-cost byproduct of the per-argument check the code
   already runs (KISS preserved — no new machinery, just a widened OR). Verified non-vacuously: the
   `#[cfg(test)]`-only sabotage hook (`Cx::stage3_sabotage_skip_substitution`,
   `infer_type_with_active_affine_sabotaged`) feeds R2 and R3 the *unsubstituted* RHS, and both
   wrongly ACCEPT under that sabotage — proving the real splice (not the trigger alone) does the
   work, and that R3 specifically needs the widened trigger to ever reach it.
4. **ADVERSARIAL FINDING (MEDIUM, fixed in the same leaf) — a capture hazard in the check-time-only
   substitution, beyond §2 step 1's literal (B)-only text.** §2 step 1 only specifies the shadow-skip
   half of substitution (don't splice an occurrence shadowed by an inner binder of the *param's*
   spelling). Verification found a second, distinct hazard: a spliced argument's own free variable
   (e.g. a caller's `s` inside `consume s`) can be *captured* by an RHS-local binder that
   coincidentally shares its spelling (`let s = 5 in p`), producing a spurious refusal of an
   otherwise-linear ACCEPT case. **Fix (landed):** the substitution walk also fresh-renames every
   RHS-local binder it introduces (`Let`/`Lambda`/`For`/most match-arm-pattern idents) to a
   `%`-prefixed synthetic name, mirroring `elab.rs::sugar_expand`'s own (A) step at this layer — an
   established, already-landed technique re-applied, not an invented one. **One narrow residual is
   honestly left open, not silently closed:** a match-arm `Pattern::Ident` that is genuinely
   ambiguous between "binder" and "nullary constructor reference" cannot be disambiguated without a
   scrutinee type, which this pure-`Expr` walk does not have; a conservative,
   type-independent approximation (`Cx::is_any_nullary_ctor` — never rename a name that spells *any*
   registered nullary ctor anywhere) is sound (never corrupts a real ctor reference) but leaves a
   compound, narrow coincidence unclosed (FLAG-pattern-ctor-collision, `Cx::stage3_substitute_pattern`'s
   own doc comment). Flagged, not guessed past.
5. **ADVERSARIAL FINDING (Empirical, pre-existing — not a Stage-3 regression) — aliasing a
   *pre-existing* composite-affine local across two independent destructurings is accepted, and
   this is inherited, not introduced.** Verified directly: an equivalent **ordinary, non-sugar** `fn`
   with the identical shape (`match h {Wrap(s1)=>s1}, match h {Wrap(s2)=>s2}` over the same
   pre-existing `h: Handle`) is *also* accepted by the landed M-919 tracker today (each pattern-match
   field-capture creates its own independent slot, with no memory that both destructure the same
   underlying value) — see `crates/mycelium-l1/src/tests/checkty.rs`'s
   `stage3_prior_handle_alias_destructured_twice_is_a_known_pre_existing_gap`. Stage 3 faithfully
   inherits this pre-existing M-919 posture (DN-117 §4.3's own design goal — match hand-written
   code's posture exactly, never stricter, never laxer) rather than closing or worsening it; recorded
   here as an open, honestly-flagged item for a *future* M-919 hardening (composite-value-identity
   tracking), not this leaf's scope.
6. **Q3 drop verdict — ACCEPTED and LANDED exactly as specified (§4.3).** A dropped affine value
   param, and a dropped RHS-local affine binding, are both **ACCEPT** — verified by `A3`
   (`crates/mycelium-l1/src/tests/affine_stage3.rs`) and the two superseded `reachability_stage1b.rs`
   controls (which asserted the old wholesale-refusal verdict and now assert ACCEPT, with the
   supersede documented in place — see house rule #3 applied to tests too, DN-117 §5's own
   instruction). No `Proven` claim is made; guarantee stays `Empirical`.
7. **Q4 test contract — ACCEPTED as the bar, and LANDED, exceeding it in one respect.** The §5
   corpus (A1–A3 accept; R1–R5 refuse) is fully implemented in
   `crates/mycelium-l1/src/tests/affine_stage3.rs` (12 tests), plus the two superseded
   `reachability_stage1b.rs` controls rewritten in place (not merely deleted — house rule #3). The
   mandatory non-vacuity items are both present: (1) the `A1→R1` mutation flips ACCEPT→REFUSE
   (`mutation_a1_to_r1_flips_accept_to_refuse`); (2) the sabotage control
   (`r2_sabotage_without_substitution_wrongly_accepts`,
   `r3_sabotage_without_substitution_wrongly_accepts`) flips R2 and R3 to false-accept. **Honesty
   note, not silently omitted:** `R4` (cross-argument aliasing) and `R1` (top-level duplication) do
   **not** flip under the sabotage control — documented per-fixture in the test module and confirmed
   by a dedicated non-flip assertion (`r4_sabotage_does_not_change_the_verdict_different_layer_refuses_it`):
   both are already caught by mechanisms other than the substituted-`Expr` splice (the ordinary
   per-argument affine bookkeeping for R4; the defensive param-scope seeding for R1's simple
   top-level case), so sabotaging *only* the splice cannot un-catch them. Reporting only the fixtures
   that flip, and being explicit about the two that don't (and why), is the honest form of this
   control (VR-5) — not a failure of the design.
8. **Q5 boundary — CONFIRMED.** Cross-nodule affine stays Stage 4 (untouched); the lambda-capture
   open question (FLAG-lambda-affine, §9) is **not** newly closed by this leaf — Stage 3 inherits
   whatever posture the landed tracker already has for a lambda-captured affine value, unchanged and
   unexamined here, exactly as flagged.
9. **CRITICAL — Accepted ratifies the design + implementation choices above; Enacted is NOT reached
   by this note (house rule #3 / VR-5).** Implementation landed in this **same** leaf's PR
   (`crates/mycelium-l1/src/checkty.rs` — the substituted-`Expr` builder + walk, the gate→trigger
   demotion, the part-2 helper deletion, the `#[cfg(test)]`-only active-tracker and sabotage entry
   points; `crates/mycelium-l1/src/tests/affine_stage3.rs` — new, 12 tests;
   `crates/mycelium-l1/src/tests/checkty.rs` + `reachability_stage1b.rs` — 4 pre-existing tests
   updated/superseded in place), change-scoped gates green (`cargo fmt -p mycelium-l1`, `cargo
   clippy -p mycelium-l1 --all-targets -- -D warnings`, `cargo test -p mycelium-l1` — unit +
   every integration target, 0 failures). Landing code in the same PR that ratifies the design does
   not itself confer `Enacted` — that status means "complete and stable, outside ongoing maintenance
   and future-dev integration," a call for the integrating parent to make on its own review, not this
   leaf. No guarantee tag here is upgraded past its checked basis: the double-consume (upper-bound)
   property over the real substituted `Expr` moves `Declared → Empirical`, checked by the §5/§7
   corpus above; no `Proven` claim is made anywhere (no checked theorem exists); the drop lower bound,
   cross-nodule affine, and the lambda-capture/prior-alias residuals stay exactly as scoped (`Declared`
   or explicitly open).
10. **FLAG dispositions.** FLAG-DN114-supersede and FLAG-DN110-8.2 (§9) remain standing for the
    integrating parent (this leaf does not edit those notes — orchestrator-owned per its own scope
    note); FLAG-DocIndex/FLAG-CHANGELOG/FLAG-issues remain standing (shared files, not edited from
    this leaf); FLAG-renumber is unaffected; FLAG-lambda-affine stays open (point 8, above). One new
    flag is added: **FLAG-pattern-ctor-collision** (point 4, above) — the narrow match-arm-pattern
    residual in the check-time-only substitution's capture-avoidance.
11. **ERRATA (2026-07-11, append-only — text above kept, not rewritten, house rule #3) — CRITICAL,
    fixed post-ratification: point 4's "sound (never corrupts a real ctor reference)" claim was
    WRONG.** A separate leaf's adversarial review of facility Stage 3 found the ambiguous-`Ident`
    handling point 4 describes was **not** sound — it was a confirmed, reproducible **false ACCEPT**
    of a genuine double-consume, on `claude/leaf/m1054-stage3-affine`. Point 4's own reasoning
    ("never corrupts a real ctor reference") only checked the *ctor-reference* reading; it did not
    check the *binder* reading, and that is exactly where the hazard lives: when the ambiguous
    identifier is genuinely a binder (not a ctor reference) for *this* pattern, leaving it
    **unrenamed** — the choice point 4 ratified — skips this same walk's own (A)-style
    capture-avoidance discipline (the paragraph immediately preceding point 4's own citation, in
    `Cx::stage3_substitute_expr`'s doc comment). A spliced argument's free variable of the same
    spelling as the unrenamed binder (a caller-outer local, coincidentally named the same as the
    colliding nullary ctor — a pure spelling accident, unrelated to the binder's own scrutinee type)
    is then **captured** by the pattern binder instead of resolving to the caller's value, hiding a
    real double-consume from the tracker. **Confirmed reproducible** (verified both ways: the exact
    call is falsely `Ok`-accepted under the pre-fix code and correctly `Err`-refused under the
    post-fix code, by `git stash`-reverting and restoring just the fix and re-running the same
    fixture — `stage3_pattern_ctor_collision_false_accept_is_now_refused`,
    `crates/mycelium-l1/src/tests/affine_stage3.rs`): `Sentinel = s` (a nullary ctor spelled `s`);
    `Pick3(h: Handle, q: Substrate) = match h { Wrap(s) => consume q }`; called as
    `(Pick3(Wrap(consume h_backing), s), consume s)` with a caller-outer local also named `s` — the
    call was wrongly accepted, silently double-consuming the caller's outer `s`.
    **Fix (landed, same errata date):** `Cx::stage3_substitute_pattern` now **refuses the whole
    sugar call** in the ambiguous case, with a never-silent diagnostic (G2), rather than guessing
    which reading is meant — threaded as a `Result<Pattern, CheckError>` through
    `stage3_substitute_pattern`/`stage3_substitute_arm` (previously infallible). **A conservative
    false-REFUSE here is sound; the false-ACCEPT it replaces was not.** The full close (real
    scrutinee-type-directed disambiguation, mirroring `Self::resolve_pattern`'s own logic) remains
    out of proportion to this fix's scope and stays open — FLAG-pattern-ctor-collision (point 10)
    is **not** closed by this errata, only its unsound resolution is. Two new tests added
    (non-vacuity control included — a same-shape call with a *non*-colliding binder spelling still
    ACCEPTS, confirming this fix does not over-refuse the ordinary case); the crate's full
    `cargo test -p mycelium-l1` (unit + every integration target) stays green, 0 failures.
    Guarantee posture: the corrected claim is `Empirical` (checked by the new corpus above), not
    `Proven` — same posture as the rest of this note's `Declared`/`Empirical` claims; no tag is
    upgraded past its checked basis (VR-5).

---

## Changelog (this note)

- **2026-07-11 — Draft.** Authored as a design-reasoner build-plan scoping note for M-1054 Stage 3
  (OQ-H4). Ranked recommendation: L1 substituted-`Expr` affine check at check time (FLAG-A / Rank 1).
  Records the grounded correction of the task brief's drop-case verdict (ACCEPT, not REJECT; §4.3).
  Read against `dev @ 3c7e85d7`. Not self-ratified (house rule #3).
- **2026-07-11 — Accepted (ratification + implementation, same leaf).** Design ratified per the
  Ratification section above; Stage 3 implemented in `crates/mycelium-l1/src/checkty.rs` +
  `crates/mycelium-l1/src/tests/affine_stage3.rs` (12 new tests) with two adversarial findings fixed
  in-flight (the R3 trigger-widening, the check-time-only capture hazard) and two honestly recorded
  (the pattern-ctor-collision residual; the pre-existing prior-handle-alias gap, not a regression).
  `Empirical` for the double-consume upper bound over the real substituted `Expr`, checked by the
  landed corpus; no `Proven` claim. Not `Enacted` (house rule #3) — that is the integrating parent's
  call.
- **2026-07-11 — Errata (CRITICAL fix, same day, separate leaf).** Ratification point 4's
  "sound (never corrupts a real ctor reference)" claim for the pattern-ctor-collision residual was
  found WRONG by adversarial review — a confirmed reproducible false ACCEPT of a genuine
  double-consume, not merely a narrow unclosed residual. Fixed:
  `Cx::stage3_substitute_pattern` now refuses the whole sugar call in the ambiguous case (a
  never-silent diagnostic, G2) instead of leaving the binder unrenamed. See Ratification point 11
  above for the full grounded correction; still `Empirical`, no `Proven` claim; FLAG-pattern-ctor-
  collision (the disambiguation gap itself) stays open. Branch
  `claude/leaf/m1054-stage3-affine`, held for re-verify (not merged).
