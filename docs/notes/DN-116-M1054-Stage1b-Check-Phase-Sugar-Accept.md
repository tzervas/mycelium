# Design Note DN-116 — M-1054 Stage 1b: Check-Phase Sugar-Call Accept (Option B, the Def-Time RHS Type-Scheme)

| Field | Value |
|---|---|
| **Note** | DN-116 — a leaf-scoped implementation note for the M-1054 native-metaprogramming facility's Stage 1b: making the verified Stage-1 hygienic expander ([`elaborate_lower_rule_with_args`](../../crates/mycelium-l1/src/elab.rs), `crates/mycelium-l1/src/elab.rs`) reachable end-to-end from a value-parametric sugar-rule call site — the checker's `Cx::check_sugar_call` accept path plus `Elab::app`'s new §5.2 dispatch. Filed as a leaf-scoped note (not folded into DN-110/DN-110-8.2-hygiene-deepdive, which stay the design-level source) so the semcore serial lane can iterate without editing a shared/held design doc. |
| **Status** | **Draft** — records a leaf's implementation choice (Option B) and its two honesty gates for the orchestrator's review; not itself a ratification of DN-110/DN-110-8.2's design (those stay `Accepted`, not `Enacted` — see DN-110's own status line). Moves toward `Accepted` only via the orchestrator's/maintainer's review of the PR this note ships with. |
| **Owner-scope** | Owns only this file. Treats `docs/Doc-Index.md`, `CHANGELOG.md`, `tools/github/issues.yaml`, `docs/api-index/`, and every other note/RFC/ADR as **read-only** — flagged, not edited, for the integrating parent to reconcile at `dev → integration`. |
| **Decides** | *For this leaf's scope only* (all `Declared` unless a landed/checked basis is cited): (1) **Option B** — a monomorphic value-parametric `lower` rule's RHS result type is fixed at **definition** (no coercion exists in this checker — `want == got` everywhere), so the accept path types the call **once**, from the RHS inferred with its value params bound to their *declared* types, never from the call site's own argument nodes; (2) two **honest, never-silent gates** bound the accepted fragment to what Stage 1b's elaboration side can actually expand soundly — **Stage 2 (OQ-H1)**: refuse a genuinely free RHS identifier (not a value param, not an RHS-local binder, not a same-nodule fn/ctor/prim/`lower`-rule); **Stage 3 (OQ-H4)**: refuse an affine (`Substrate`) value parameter, and (defensively) a structurally affine RHS-local binding; (3) `Elab::app` dispatches a checker-accepted sugar call to the existing Stage 1 expansion machinery — no new elaboration logic, no double-expansion. |
| **Feeds / builds on** | **M-1054** (the implementation epic) / **DN-110** §5-A Rank-1 (the facility) / **DN-110-8.2-hygiene-deepdive** §4 (A)+(B) hygiene model, §10 OQ-H1/OQ-H4 (the two residuals this note gates on) / **M-1055** (E1's PASS verdict — the hygiene mechanism this note reaches) / the **landed Stage 0** (`crates/mycelium-l1/src/checkty.rs::Cx::check_sugar_call`, recognition + unconditional refusal) and **Stage 1** (`crates/mycelium-l1/src/elab.rs::elaborate_lower_rule_with_args`, the (A)+(B) hygienic expansion, reachable only by direct/white-box call before this note). |
| **Guarantee** | Code citations are `Empirical` (read at the tree this leaf branched from, `dev` `7f6b4c88`). The Stage 1b mechanism itself — end-to-end reachability over the white-box, affine-free, single-nodule, monomorphic fragment — is `Empirical` per `crates/mycelium-l1/src/tests/reachability_stage1b.rs`'s dual-oracle corpus (below). Surface-source reachability, cross-nodule resolution (Stage 2), affine soundness over the expanded surface (Stage 3), and generic value-parametric sugar stay `Declared`/out of scope (VR-5 — no tag upgraded past its checked basis). |

---

## §1 The gap (FLAG-B, verified by a design-reasoner scoping)

Stage 1 (landed, on `dev`/`main`) built the L0 elab-phase hygienic expander
(`elaborate_value_parametric_rule`/`sugar_expand`, `crates/mycelium-l1/src/elab.rs`) —
empirically capture-safe over its own fixture corpus (`src/tests/facility_stage1_hygiene.rs`,
`Declared -> Empirical` for (A)+(B) on the real elaborator path). But it was reachable only by
white-box/direct call. Reachability from a real call site needs **two** changes, both gated by the
checker:

1. **Check-phase.** `Cx::check_sugar_call` (`checkty.rs`) matched arity/types then unconditionally
   refused on every path (Stage 0's own documented, structural guard — "this function can never
   return `Ok`"). It had to start **accepting** a well-formed call.
2. **Elab-phase.** `Elab::app` (`elab.rs`) had no dispatch branch for a recognized
   value-parametric-rule call; even an accepted checker call would still fail at elaboration,
   falling through to the generic `"unknown function/constructor/prim"` residual.

A checker-only fix would go green at check and red at eval — this note's scope is doing **both**.

## §2 Option B — the def-time RHS type-scheme

Three options were on the table for typing an accepted sugar call:

- **Option A (rejected)** — reuse the L0 expander at check time. Rejected: double-expands (the
  checker would run the same hygienic walk the elaborator later re-runs) and drags `%`-namespace
  freshening — an elaboration-phase concern — into the checker, breaking the ratified two-phase
  split (L1 check-phase for typing/def-site/affine; L0 elab-phase for the hygienic expansion).
- **Option A′ (rejected)** — substitute-then-check (elaborate the call site's literal RHS with
  arguments spliced in, then check the result). Rejected: reintroduces capture *at the checker*
  (the same hazard Stage 1's whole (A)+(B) design exists to avoid) — that mechanism belongs to the
  Stage 3 affine re-check over the *expanded* surface, not to this accept path.
- **Option B (adopted)** — the checker has **no coercion**: every existing accept path in
  `checkty.rs` is an exact `want == got` match (see the ordinary user-fn accept branch,
  `checkty.rs` `check_app`, and the sugar-call's own per-argument check, both unchanged by this
  note). So a **monomorphic** value-parametric rule's RHS result type is fixed the moment the rule
  is *defined* — inferring it once, with the declared value parameters bound to their *declared*
  types (never the call site's own argument nodes), gives the one answer every call site shares.
  `Cx::check_sugar_call`'s per-argument loop (unchanged in shape from Stage 0) still separately
  re-checks each *argument's* type against its declared parameter type at every call site; Option B
  only fixes the **result**.

### §2.1 The shared inference pass

`checkty.rs::check_lower_rule_rhs_type` (a rule's **definition-time** validator, run once per rule
at registration — Pass 3e-late) previously seeded its type-checking `Cx` with an **empty** initial
value scope, since v0 had no value-parametric rules reachable through it. This note extracts the
shared inference logic into `infer_expr_rule_rhs_type(ld, rhs, …) -> Result<Ty, CheckError>`, which
now **seeds the scope from `ld.value_params`** (bound to their declared, resolved types) before
calling `Cx::infer`. `check_lower_rule_rhs_type` calls it and discards the `Ty` (its own contract is
just "does this rule's RHS type-check", unchanged); `Cx::check_sugar_call`'s accept path calls the
**same** function to get the call site's `ret_ty`. One inference pass, two callers, DRY by
construction — a nullary rule's empty `value_params` makes this byte-identical to the pre-Stage-1b
empty-scope walk (no behavioral change for that case).

The shared function also seeds the **affine tracker** (`Tracker::seeded`, M-919) from the same
value-param scope — mirroring `check_fn_body`'s own `Tracker::seeded(&scope)` call — so a
double-consume of a value parameter *or* an RHS-local `Substrate` binding is refused at definition
time exactly as it would be inside an ordinary function body. This is the M-919 **upper-bound**
(duplication) check only; it does not by itself make a Substrate-typed value parameter safe to
*accept at a call site* (see §3 below — expansion-time substitution is a different hazard than an
ordinary function call's CBV `Let`-binding).

## §3 The two honest gates (never-silent refusals, not silent accepts)

A type-only accept is sound **only** if it is scoped to the fragment Stage 1's elaboration can
actually expand without reintroducing a hygiene or affine hazard. Two gates, each a distinct,
message-carrying refusal citing its stage and open question — never a fabricated or
capture-unsafe/affine-unsafe accept (G2/VR-5):

### §3.1 Stage 2 (OQ-H1) — free-identifier gate

`elaborate_lower_rule_with_args`'s own doc comment (Stage 1, unchanged by this leaf) is explicit: a
genuinely free RHS identifier — neither a value parameter nor an RHS-local binder — is left as a
bare, **unresolved** `Var` by the expansion; def-site resolution (to a content-addressed reference,
ADR-003) is Stage 2's own job (OQ-H1), not attempted by the Stage 1 machinery this note wires up.
Accepting a call whose RHS contains such an identifier would either (a) fail at elaboration with a
confusing, non-Stage-2-labeled error, or (b) — worse, if the identifier happens to coincidentally
resolve some other way — silently rely on def-site information the expansion does not actually
thread through.

`Cx::check_sugar_call` gates this explicitly: a depth-first, left-to-right walk
(`Self::rhs_first_free_id`, `checkty.rs`) over the RHS, treating as **bound** the value parameters
plus every RHS-local binder introduced by an ancestor `let`/`lambda`/`for`/match-arm pattern, and as
**within the fragment** a same-nodule, unambiguous top-level `fn`, `lower`-rule, constructor, or
`prim_family`-recognized kernel primitive (`add_s` et al.) reference. The first identifier that is
neither refuses, citing "Stage 2" and "OQ-H1" by name. **Known residual (flagged, not silent):** the
VSA-prim and float-prim dispatch sets (`try_check_vsa_prim`/`try_check_float_prim`) are
argument-shape-matching functions, not name predicates, so they are not checked here — an RHS
calling a VSA/float prim would be (over-)refused by this gate as if it were a free identifier. Not
exercised by any Stage 1b fixture; a residual for whoever extends this gate's coverage.

Cross-nodule / import-ambiguous resolution — the actual substance of OQ-H1 — is out of Stage 1b's
single-nodule scope entirely; `imports` is not consulted by this gate.

### §3.2 Stage 3 (OQ-H4) — affine gate

Two parts, both citing "Stage 3" and "OQ-H4":

1. **Any value parameter whose type *is, or structurally contains*, `Substrate` is refused
   outright.** `Elab::app`'s ordinary (non-sugar) fn-call inlining `Let`-binds each argument
   **exactly once** (`bindings.push((param.name.clone(), self.fresh(&param.name), karg, pty))`, then
   a single `Node::Let` per parameter) — even if the parameter is referenced multiple times in the
   body, the body references the *let-bound fresh variable*, never the raw argument node twice.
   Stage 1's (B) substitution is different: `sugar_expand` splices the **raw argument `Node`
   verbatim** at **every** RHS occurrence of its value param (no let-binding, by design — value
   params are substituted, not bound). An ordinary (non-affine) argument may safely be spliced at
   more than one occurrence (re-evaluation, not a soundness hazard, for a pure v0 value); an
   *affine* argument may not — splicing it into two occurrences would double-consume it. This is a
   **real, structural** hazard (not a defensive guess): refusing every `Substrate`-**containing**
   value parameter removes it entirely.
   **Adversarial-verify finding (2026-07-11, HIGH — fixed).** The original check
   (`matches!(pty, Ty::Substrate(_))`) matched only a **top-level** `Substrate` type, missing a
   *composite* value param — a `Data` type whose constructor structurally wraps a `Substrate` field
   (e.g. `Data("Handle")` with ctor `Wrap(Substrate{gpu})`). A rule `Dup(h: Handle)` referencing `h`
   twice, each occurrence pattern-matched (`match h { Wrap(s) => … }`) to extract the affine field,
   was **false-accepted** by part 1 and missed by part 2 (part 2 only inspects `Let`-bound affine
   producers, never match-arm patterns) — the expander then splices the caller's single `h` argument
   node at both occurrences, double-consuming the extracted `Substrate` field on `eval`. **Fixed**
   by `Self::ty_structurally_contains_substrate` (`checkty.rs`): a recursive structural walk over
   `pty` that resolves into a registered `Data` type's constructor field types (substituting the
   type's own arguments for its abstract params via `subst_ty`) and refuses if `Substrate` is
   reachable anywhere in that structure, not just at the top level. Termination is by an on-path
   `visiting: BTreeSet<String>` of type names (cycle-cut, not a depth counter — the finite type
   registry bounds recursion depth without one); a plain non-affine `Data` type, including a
   recursive one with no affine field, still accepts. See `ty_structurally_contains_substrate`'s own
   doc comment for the full termination argument and its honest scope note (an unresolved abstract
   `Ty::Var` field is conservatively treated as non-`Substrate` — Stage 1b's single-nodule,
   concrete-value-param scope means this isn't reachable in practice; flagged, not silently
   widened). Regression: `checkty.rs`'s `stage3_composite_substrate_field_value_param_refused_*`
   fixtures (the `Handle`/`Wrap(Substrate)`/`Dup` shape above), verified non-vacuous by temporarily
   reverting the recursive walk to the old top-level-only `matches!` and confirming the fixture
   false-accepts, then restoring.
2. **A structurally affine RHS-local binding is refused defensively.** A `let`-bound call to a `fn`
   whose *declared* return type is `Substrate`, or a bare `consume`, is refused
   (`Self::rhs_first_affine_binding` + `Self::expr_is_structurally_affine`, `checkty.rs`) — the exact
   "helper-fn-acquired `Substrate`" shape `infer_expr_rule_rhs_type`'s own doc comment names. This is
   a **structural** (declared-signature-based) detector, not a second full affine-inference pass —
   narrower than the eventual Stage 3 substituted-Expr affine re-check (M-919/OQ-H4 proper), but
   sufficient to refuse the realistic shape rather than silently accept it.

**Design-reasoner note — superseded by the 2026-07-11 adversarial-verify finding above (house rule 3,
no self-ratification; this record stays append-only, so the original reasoning is kept, not
deleted, with the correction appended).** The original text here argued part 2 may be *redundant*
with part 1, reasoning that Stage 1's per-expansion-site freshening (OQ-H5) plus
`infer_expr_rule_rhs_type`'s seeded Tracker keep an RHS-local `Substrate` binding's own use-count
correct at every expansion site. **That argument did not hold**, but not for a part-2 reason: part 1
itself was under-strength (top-level-only), which the composite-type finding above closed by making
part 1 structurally recursive. With part 1 now closing the composite/nested case too, **part 2 is
still not redundant with part 1** — part 1 gates the *value parameter's declared type*; part 2 gates
a *different* hazard entirely, an RHS-**local** `let`-bound affine acquisition (a helper `fn`
returning `Substrate`, or a bare `consume`) that never appears in any parameter's type at all. The
two parts close two distinct hazard classes (parameter-typed vs. locally-acquired affine values);
neither subsumes the other. This resolves the design-reasoner's open question: **not redundant**,
confirmed by tracing a concrete case each part alone would miss (a `let`-bound `Substrate`-returning
helper used twice in the RHS has no affine-typed *value parameter* at all, so part 1 alone would
miss it; the composite value-parameter case above has no RHS-local `let` binding at all, so part 2
alone would miss it).

**Superseded-in-part by Stage 3 / DN-117 (2026-07-11, append-only — house rule #3, the text above is
kept, not rewritten).** M-1054 Stage 3 (OQ-H4) has since landed
(`docs/notes/DN-117-M1054-Stage3-Affine-Over-Substituted-Expr.md`, `Accepted`): §3.2's two-part
wholesale-refusal gate described above is **replaced** by a real linear-use check over a
check-time-only substituted `Expr` (the argument spliced at every RHS occurrence of its value
param, walked by the real M-919 `Tracker`) — **accept-linear-or-dropped, refuse-duplicated**,
rather than refusing every affine-typed value parameter or RHS-local binding outright. Concretely:
part 1 (`ty_structurally_contains_substrate`) is demoted from the accept/refuse decision to a cheap
*trigger* only; part 2 (`rhs_first_affine_binding`/`expr_is_structurally_affine`) is **deleted** —
its job turned out to be fully subsumed by `infer_expr_rule_rhs_type`'s own pre-existing
active-tracker walk over the unsubstituted RHS (DN-117 §3.3), which needed no new machinery at all.
The composite (`Handle`/`Wrap(Substrate)`) case this section's adversarial-verify finding closed is
now handled precisely by the real walk (DN-117 R2/A2), not by the structural over-approximation
described above. The corrected drop verdict (an unused affine value parameter, or an unused
RHS-local affine binding, is **accepted**, not refused — the static pass enforces only
use-at-most-once, never a must-consume lower bound) is DN-117 §4.3's own grounded correction of this
note's original (defensive, over-refusing) posture. See DN-117 for the full mechanism, its own
adversarial findings, and the non-vacuous test corpus
(`crates/mycelium-l1/src/tests/affine_stage3.rs`).

## §4 `Elab::app`'s §5.2 dispatch

A single new branch in `Elab::app` (`elab.rs`), tried **last**, mirroring `check_sugar_call`'s own
identical last-resort placement in `Cx::check_app` (so a name resolving any other way — HOF, `fn`,
recursion, constructor, prim, trait-method — is completely unaffected, regression-safe by
construction): if `self.env.lower_rules.contains_key(name)`, elaborate each argument left-to-right
(CBV, matching every other call-site branch) and hand the elaborated `Node`s to
`elaborate_lower_rule_with_args` — the **existing**, unmodified Stage 1 machinery. Item-shaped-RHS
refusal and arity re-validation are **not** duplicated here; `elaborate_lower_rule_with_args`'s own
`match_value_params` performs both, never-silently — a program reaching elaboration has already
passed `check_sugar_call`'s identical gates, so in practice this call sees only already-checked,
expr-shaped, arity-matched invocations, with the callee's own checks as defense in depth (DRY,
KC-3 — no new elaboration logic).

## §5 Reachability tests + non-vacuity (THE NON-VACUITY LAW)

`crates/mycelium-l1/src/tests/reachability_stage1b.rs` (new, in-crate, white-box per
`LowerDecl::value_params`'s own doc comment — no surface grammar exists yet). Because the real
elaborator is *more* hygienic than the E1 prototype (Pass 1 already namespaces same-invocation
binders via `Elab::fresh`), the "colliding" call-site argument is **elicited** — not hand-picked —
via the identical `fresh_kernel_name_via_real_elaboration` technique
`facility_stage1_hygiene.rs` uses, eliciting the real elaborator's own first-fresh-name choice for a
`let t = … in t` shape, reproducing the actual OQ-H5 cross-invocation-collision hazard (two
*independent* top-level elaborations — `main`'s own, and `Swap2`'s RHS expansion invoked mid-way by
the new dispatch — each reset `Elab::fresh`'s counter to `0`).

- **Full chain**: `full_chain_step1_check_accepts` (`Cx::check`/`infer_type` → `Ok`, correctly
  typed) and `full_chain_step2_elaborate_dispatches_and_is_capture_safe` (the **ordinary**
  `elaborate(&env, "main")` entry — not a direct expander call — expands, verified capture-safe by
  two independent oracles: `alpha_eq` against a disjointly-spelled hand oracle, and an
  `Interpreter::eval` observational differential).
- **Non-vacuity controls, verified to genuinely fail when the mechanism under test is disabled**
  (each hand-checked during this leaf's own development, by literally disabling the code path and
  re-running — restored before commit):
  - (a) `full_chain_control_disable_freshening_breaks_both_oracles` — the `#[cfg(test)]`-only
    `elaborate_value_parametric_rule_disable_freshening_for_test` entry point, applied to this
    module's own fixture, reproduces the hand-derived *wrong* (captured) value and fails
    `alpha_eq` against the hygienic oracle.
  - (b) `control_arity_mismatch_still_refuses` / `control_argument_type_mismatch_still_refuses` /
    `control_item_shaped_rule_still_has_no_expression_form` — the pre-existing Stage 0 refusal
    diagnostics still fire; Stage 1b's accept path did not swallow them.
  - (c) `control_affine_value_param_hits_stage3_residual` / `control_free_non_param_id_hits_stage2_residual`
    / `control_affine_rhs_local_binding_hits_stage3_residual` — each gate's own dedicated
    residual fires, naming its Stage + OQ.
  - Directly verified during development (not committed as a mechanism, since the harness has no
    reachable "disable" flag for the two checker gates or the `Elab::app` dispatch itself — only
    for Stage 1's own (A) freshening): temporarily disabling the `Elab::app` dispatch made
    `full_chain_step2` fail with the pre-Stage-1b `"unknown function/constructor/prim"` residual;
    temporarily disabling `check_sugar_call`'s accept made `full_chain_step1` fail; temporarily
    disabling the Stage-2 gate's `if let` made `control_free_non_param_id_…` fail (falling through
    to a *different*, non-"Stage 2 (OQ-H1)"-labeled `cx.infer` error); temporarily disabling the
    Stage-3 part-1 gate's `matches!` made `control_affine_value_param_…` fail by flipping to `Ok`
    entirely.

## §6 Definition of Done

- [x] `Cx::check_sugar_call` accepts a well-formed, gate-clearing value-parametric sugar call,
      typed via the shared, once-per-call-site `infer_expr_rule_rhs_type` (Option B).
- [x] `Elab::app` dispatches an accepted sugar call to the existing Stage 1 expansion machinery —
      no new elaboration logic, no double-expansion, last-resort placement mirroring the checker.
- [x] Stage 2 (OQ-H1) and Stage 3 (OQ-H4) gates both land, each a dedicated, message-carrying,
      stage/OQ-citing refusal — never a silent accept past the affine-free, single-nodule,
      params-and-globally-unambiguous-fns/ctors/prims-only fragment.
- [x] `crates/mycelium-l1/src/tests/reachability_stage1b.rs` lands the full-chain (check → elab →
      eval) reachability corpus with dual-oracle capture-safety and the required non-vacuity
      controls, each independently verified (during development) to genuinely fail when its
      mechanism is disabled.
- [x] Pre-existing Stage 0/Stage 1 test suites updated for the new, honest contract (no test left
      asserting a claim Stage 1b has superseded) — `checkty.rs`'s former
      `stage0_sugar_call_is_recognized_and_dispatched` (asserted unconditional refusal) is now
      `stage1b_sugar_call_recognized_and_accepted` (asserts the accept); `facility_stage1_hygiene.rs`'s
      module doc comment's stale "check_sugar_call... unchanged... still refuses every recognized
      call" claim is corrected.
- [x] Change-scoped gates green: `cargo fmt -p mycelium-l1`, `cargo clippy -p mycelium-l1
      --all-targets -- -D warnings`, `cargo test -p mycelium-l1` (535 passed, 1 ignored, 0 failed).
- [ ] Orchestrator review + integration close-out (this note's own status moves `Draft -> Accepted`
      only via that review — house rule #3, no self-ratification).

## §7 FLAGs for the integrating parent

- **Shared files** (not touched by this leaf, per its scope — flagged for the integration-tier
  reconciliation): `CHANGELOG.md` entry; `docs/Doc-Index.md` row for this note; `docs/api-index/`
  regeneration (`Cx::check_sugar_call`'s signature is unchanged, but its behavior and doc comment
  changed materially, as did `Elab::app`'s); `tools/github/issues.yaml` — M-1054's own body could
  record this leaf's residual scope (single-nodule/affine-free/monomorphic; the §3.2 open question
  on whether Stage-3-part-2 is redundant with part-1) as a design note for whoever picks up Stage 2
  or Stage 3 proper.
- **Scope question for the orchestrator/maintainer** (§3.2 above): is the structurally-affine
  RHS-local-binding gate (Stage 3 part 2) load-bearing, or is it a defensible-but-redundant
  belt-and-braces addition given part 1 (the value-parameter gate) plus Stage 1's own freshening
  design? This leaf implements both per the task's explicit ask, but flags the open reasoning
  rather than asserting the redundancy claim as settled (VR-5 — don't upgrade an unchecked
  argument to a decided fact).
- **No real bug found in the Stage-1 hygiene logic itself** — `elaborate_lower_rule_with_args`/
  `sugar_expand`/`elaborate_value_parametric_rule_inner` are unmodified by this leaf; every
  reachability-test finding was a genuine, expected coverage gap (the old refusal), not a
  correctness defect in the landed Stage 1 mechanism.
- **DN-116 renumber (integration close-out, 2026-07-11) — residual, deliberately deferred.** This
  note was filed as **DN-114** and collided with the unrelated `docs/notes/DN-114-Validated-Narrative-Generation.md`
  (E40-1, kept as DN-114 per maintainer directive). Renumbered to **DN-116** at this close-out (file
  moved; self-refs and DN-115's citations of it updated in the same pass). **`crates/mycelium-l1/`
  source comments and test-module doc comments citing "DN-114"**
  (`elab.rs`, `checkty.rs`, `src/tests/checkty.rs`, `src/tests/defsite_resolution_stage2.rs`,
  `src/tests/reachability_stage1b.rs`, `src/tests/facility_stage1_hygiene.rs`) **were NOT repointed
  in this pass** — a concurrent Stage-3 agent is actively editing `mycelium-l1`, and touching those
  files here would race it. Flagged, not silent (G2): those in-source citations still read "DN-114"
  and mean this note (now DN-116) until a follow-up touches that crate — grep `DN-114` in
  `crates/mycelium-l1/` to find the residual set.
