# Design Note DN-115 — M-1054 Stage 2 (OQ-H1): Def-Site Resolution of Free RHS Identifiers, Single-Nodule Fragment

| Field | Value |
|---|---|
| **Note** | DN-115 — a design-reasoner scoping note (plan → review → improve → ratify → implement) for **Stage 2** of the M-1054 native-metaprogramming facility: lifting DN-114's **Stage-2 gate** (`checkty.rs::rhs_first_free_id`) for the **single-nodule** case so a sugar rule's free RHS identifier binds its **definition-site** meaning (referential transparency, OQ-H1). Filed leaf-scoped (not folded into DN-110/DN-110-8.2, which stay the design-level source) so the semcore serial lane can iterate without editing a shared/held design doc. |
| **Status** | **Draft** — a design-reasoner recommendation for the maintainer/orchestrator to ratify. This note **recommends, it does not ratify** (house rule #3; the author never moves status to `Accepted`). Moves `Draft → Accepted` only via maintainer/orchestrator review of the PR this note ships with; the code it plans stays `Declared` until landed + tested (VR-5). |
| **Owner-scope** | Owns only this file. Treats `crates/mycelium-l1/**` (semcore serial lane — this scoping is **READ-ONLY** on it), `docs/Doc-Index.md`, `CHANGELOG.md`, `tools/github/issues.yaml`, `docs/api-index/`, and every other note/RFC/ADR as **read-only** — FLAGGED for the integrating parent, not edited here. |
| **Decides (proposes, for ratification)** | *All `Declared` unless a landed/checked basis is cited.* (1) The **primary, code-grounded finding**: single-nodule def-site resolution / referential transparency is **already achieved by construction** in Stage 1b — so Stage 2 is **not** "add def-site resolution" but "**prove** the present guarantee end-to-end (`Declared → Empirical`) on the real elaborator, close two narrow gate-correctness gaps, and make the single-nodule invariant explicit so Stage 4 is a deliberate change." (2) The **resolved reference** for single-nodule is the **already-content-addressed inlined L0 body** (status quo) — **no new L0 node / reference kind** (KC-3, YAGNI); the explicit cross-nodule *symbol reference* is Stage 4 / DN-113 work, resolved by **M-1024's linker** (`resolve_imports`/`PhylumEnv::link`), not by raw hashing. (3) Resolution stays where it is — **Pass-1 elab against the def-site env** — with a documented invariant that use-env == def-env **only** because single-nodule; the Stage-2 gate is **relaxed** only for the two over-/under-refusal gaps, and **still refuses** every genuinely cross-nodule/phylum free id (→ Stage 4). |
| **Feeds / builds on** | **M-1054** (the epic) / **DN-114** §3.1 (the Stage-2 gate this note revisits) / **DN-110-8.2-hygiene-deepdive** §4(C), §6, §7 E2, §10 OQ-H1 (the design-level source) / **DN-113** (cross-phylum resolution — Stage 4, the deferred half) / **M-1024** (`checkty.rs::resolve_imports`/`PhylumEnv::link`, the same-phylum linker) / **ADR-003** (content-addressed L0 identity) / **KC-3** (no kernel growth). |
| **Grounding basis** | `Empirical` where read against the codebase at `dev`/worktree tip **`3cd7dbc2`** (the Stage-1b landing; commands + `file:line` cited inline). The proposed Stage-2 build is **`Declared`** until landed + tested. Where undetermined, flagged as an open question, never guessed (G2/VR-5). |

---

## §0 The question, in one line

Macro hygiene half (B) — *referential transparency*: a free identifier in a sugar rule's RHS must
resolve to **what it meant at the rule's definition site**, regardless of what that name means at the
use/call site. Stage 2 is: relax DN-114's Stage-2 gate so single-nodule free RHS ids are
**accepted-with-def-site-resolution** instead of refused, while still refusing the cross-nodule/phylum
case (→ Stage 4).

## §1 The finding that reframes the task (read this first — it is adversarial to the brief, VR-5)

The task brief assumes Stage 2 must **add** def-site resolution — "Stage 1b's Stage-2 gate currently
refuses any RHS free id … Stage 2 should instead RESOLVE such a free id at def-site." **Read against
the code, the single-nodule half of that resolution already exists, by construction.** I flag this
plainly rather than author a plan that re-implements a landed guarantee (mitigation #14: the codebase
is ground truth; the brief is `Declared`).

Two facts establish it:

1. **The gate already *accepts* the same-nodule fragment.** `rhs_first_free_id`
   (`crates/mycelium-l1/src/checkty.rs:5778`) returns `None` (accept) for a bare `Path` whose name is a
   value param, an RHS-local binder, **or** `self.fns.contains_key` / `self.lower_rules.contains_key` /
   `self.ctor(name).is_some()` / `prim_family(name).is_some()` (`checkty.rs:5807-5814`). Same-nodule
   top-level fns, lower-rules, constructors, and core prims are **already in-fragment.** The only ids it
   refuses are those in **none** of those sets — which, under the single-nodule scope, are exactly the
   cross-nodule/unresolvable names (Stage 4), plus two edge gaps (§4).

2. **Elaboration already *resolves* the accepted fragment at the def-site — before the use site is
   touched.** `elaborate_value_parametric_rule_inner` (`elab.rs:799`) runs **Pass 1**
   (`el.expr`, `elab.rs:885`) over the RHS against the **def-site global env** (`env2`, built from the
   rule's own nodule; the value-param `scope` is the *only* thing seeded — `elab.rs:874-883`), **then**
   Pass 2 (`sugar_expand`, `elab.rs:921`) does only (A) `%`-freshening + (B) value-param substitution.
   So every accepted free id is resolved in Pass 1:
   - a fn **call** `helper(v)` → `Elab::app` inlines the def-site body (`elab.rs:2434` dispatch region /
     the ordinary fn-inlining branch);
   - a fn used as a **value** → monomorphize + defunctionalize to a closed term (`elab.rs:855`, `1740`);
   - a **nullary ctor** → `Node::Construct` (`elab.rs:1716-1722`);
   - a **prim** → `Node::Op`.
   By the time Pass 2 runs, **no bare `Var("helper")` for a free id survives** — Pass 2's `Node::Var`
   arm only ever sees value-param placeholders (→ (B)) and RHS-local binders (→ (A)) (`elab.rs:941-957`).

**Consequence (the crux, §3):** a use-site local of the same spelling **cannot** capture a def-site
free id, because the def-site free id was resolved (inlined / lowered) in Pass 1 against the def-site
env, and the use-site's local scope is *never consulted* for RHS-free-id resolution — it only supplies
the **argument** terms spliced by (B). The E2 property (`hygiene_defsite_resolution.rs`) therefore
already holds on the real elaborator for single-nodule — by a **different mechanism** (Pass-1 inlining)
than the E2 *prototype*'s `DefEnv` snapshot lookup. **It is simply untested end-to-end on the real
path.** That gap — not a missing resolution mechanism — is the real Stage-2 work.

## §2 What must resolve, and to what (task Q1)

### §2.1 In Stage-2 scope vs. deferred to Stage 4

| Free RHS id class | Gate today | Resolution today | Stage |
|---|---|---|---|
| Value parameter | accept | (B) splice use-site arg | done (Stage 1) |
| RHS-local binder (`let`/`lambda`/`for`/match-arm) | accept | (A) `%`-freshen | done (Stage 1) |
| Same-nodule top-level `fn` (call or value position) | accept | Pass-1 inline / defunctionalize (def-site) | **already resolved** |
| Same-nodule constructor / core `prim_family` | accept | Pass-1 `Construct` / `Op` (def-site) | **already resolved** |
| Same-nodule nullary `lower`-rule, **call** position | accept | Pass-1 `Elab::app` dispatch (nested expand) | **already resolved** |
| Same-nodule nullary `lower`-rule, **value** position | accept | **residual "unresolved name"** at elab | **gap G2 (§4.2)** |
| Same-nodule **VSA / float** prim | **over-refuse** | (would be `Op`) | **gap G1 (§4.1)** |
| Cross-nodule (same phylum) `use`-imported symbol | refuse | — (needs def-site import ctx) | **Stage 4 / DN-113** |
| Cross-phylum symbol | refuse | — (not linkable at all yet) | **Stage 4 / DN-113** |

### §2.2 The resolved reference — three options, ranked

OQ-H1 / E2 phrase the resolved reference as "a content-addressed L0 ref (ADR-003)." Evaluated against
the code:

- **Option A — the already-content-addressed inlined L0 body (status quo).** Pass-1 resolution produces
  a frozen L0 term whose identity **is** content-addressed (ADR-003 rides under every `Node`; two
  same-intent expansions dedup by hash — deepdive §6 "true half"). For single-nodule this **already**
  satisfies E2's observable property ("no bare `Var("helper")`; binds the def-site value") *and*
  OQ-H1's soundness goal. **No new node, no new reference kind (KC-3, YAGNI).**
- **Option B — a new explicit content-addressed *reference* node.** Reify `Ref(hash(A::helper))` as an
  L0 node. Matches E2's literal wording — but deepdive **§6 explicitly CORRECTED** the "content-address
  gives hygiene / is the load-bearing reference" hypothesis: content-addressing does **not** collapse
  alpha-equivalence, and the binder-namespace partition (not a hash reference) is what buys hygiene.
  A new node is a **KC-3 kernel-growth cost** with no single-nodule benefit.
- **Option C — a resolved, nodule-qualified symbol id (M-1024).** Resolve to an absolute
  `nodule::helper` binding via `resolve_imports`/`PhylumEnv::link` (`checkty.rs:1712`/`1086`), threading
  a def-site symbol id into the expander. This is the **natural mechanism for Stage 4** (cross-nodule),
  where the resolution env genuinely differs from the use site — but it is **premature for
  single-nodule** (use-env == def-env makes the qualified id redundant with inlining).

**Recommendation (ranked): A ≫ C ≫ B for Stage 2.** Keep the status-quo inlined-L0-body resolution;
add **no** reference node. Record the design *direction* that Stage 4's cross-nodule reference is
**Option C** (M-1024 qualified-name resolution), with content-addressing (ADR-003) as the L0 body's
identity underneath — **not** Option B (raw hashing is not the resolution mechanism; deepdive §6). See
§7 for the argument *against* this ranking.

## §3 Where resolution happens, and the use-site-shadowing hazard (task Q2 + Q3)

**Q2 — phase.** Resolution stays in **Pass-1 elaboration** (`el.expr` against the def-site env), which
composes with DN-114's ratified two-phase split (L1 check/type/gate; L0 expand) with **zero change**:
the checker's Stage-2 gate stays a pure *admissibility* test (does every free id resolve in-fragment at
the def-site?), and the L0 phase does the actual resolution as it already does. The resolved ref is
**not** threaded from `check_sugar_call` into the expander — it is resolved **independently at expand
time** against the same (def-site == use-site, single-nodule) env. **No conflict with FLAG-A**: FLAG-A
is exactly "L1 checks/types + resolves-admissibility; L0 expands," and this keeps both halves in place.

**The one honest caveat — a *coincidence* to make explicit (§6 DoD item).** Pass-1 resolves the free id
against `self.env` at the **use site** (the env `Elab::app` holds when it dispatches to the expander).
This yields the def-site answer **only because single-nodule ⇒ use-env == def-env.** Today that
coincidence is *undocumented*; Stage 4's cross-nodule change (where the two envs diverge) would
silently resolve against the **wrong** (use-site) env if this dependency is not called out. Stage 2
must add an **explicit, documented invariant** (a doc comment + a debug-assert or a checker precondition
that the rule is same-nodule) so Stage 4's def-site-env threading is a **deliberate** change, never a
silent correctness regression (G2).

**Q3 — the shadowing hazard, and why it cannot bite (single-nodule).** Consider the E2 shape: rule
`bump(v) = helper(v)` in nodule A (`fn helper(x) = x + 100`); use site
`let helper = (λx. x - 100) in bump(5)`. Mechanism, step by step:

1. `check_sugar_call` runs the Stage-2 gate against `self.fns` (top-level, def==use nodule) — `helper`
   is a top-level fn ⇒ accept. The use-site **local** `let helper` is **not** in `self.fns`; it plays
   no part in the gate.
2. Pass-1 `el.expr` elaborates `helper(v)` against the def-site global env ⇒ **inlines** A's `helper`
   body (`x + 100`) with the value-param placeholder `v` inside. No `Var("helper")` remains.
3. Pass-2 `sugar_expand` (A)-freshens RHS binders into the `%sugar#bump@<site>%tmp` namespace and
   (B)-splices the use-site argument `5` for `v`. `%`-names are surface-illegal (`elab.rs:807-814`), so
   they are disjoint from the use-site's `helper` by construction.
4. The expanded L0 spliced under `let helper = (λx. x-100) in …` evaluates to `5 + 100 = 105`
   (def-site), **not** `5 - 100 = -95` (use-site capture).

So `%`-freshening handles capture of *RHS binders*; **Pass-1 def-site resolution** handles referential
transparency of *free ids*. The two are orthogonal, and **together** close both hygiene halves — for
single-nodule — **by construction, not by runtime search.** Content-addressing plays no *causal* role
here (deepdive §6); it only makes the resolved body dedup cleanly.

## §4 The two narrow gate-correctness gaps Stage 2 should close (the real, small work)

Both are single-nodule, both flagged in DN-114 §3.1 / the gate's own doc comment — neither is a
referential-transparency hole.

### §4.1 G1 — VSA/float-prim over-refusal (accept path too narrow)

`rhs_first_free_id` recognizes `prim_family(name)` but **not** the VSA-prim
(`try_check_vsa_prim`) or float-prim (`try_check_float_prim`) dispatch sets — those are
argument-shape-matching functions, not name predicates (`checkty.rs:5797-5806`, the gate's own
"Known residual (flagged, not silent)"). Effect: an RHS calling e.g. `vsa_bind` or a float prim is
**over-refused** as if it were a genuinely-free id. This is soundness-safe (a false *refusal*, never a
false accept) but wrongly narrows the accepted fragment. **Fix:** add a **name-predicate** view of the
VSA/float prim sets and consult it in the gate (a `prim_name_is_recognized(name)` helper, or expose the
recognized-name set from the two dispatchers). Small, local, testable.

### §4.2 G2 — bare nullary-`lower`-rule value-position over-acceptance (accept/elab mismatch)

The gate accepts `self.lower_rules.contains_key(name)` **position-agnostically** (`checkty.rs:5809`),
but `Elab::app`'s §5.2 dispatch resolves a lower-rule only in **call** position (`elab.rs:2434`); a
**bare** nullary-lower-rule reference in value position falls to the `Expr::Path` arm
(`elab.rs:1704-1754`), which does **not** consult `lower_rules` ⇒ `residual("unresolved name")`. So a
program the checker **accepts** would **red at elab** — precisely the "green at check, red at eval"
failure DN-114 §1 exists to prevent. Not exercised by any fixture (latent). **Fix (KISS, preferred):**
**narrow the gate** — accept a nullary-lower-rule id only in call-head position (mirror the elab
dispatch's own position sensitivity), refusing the bare value-position form with a never-silent,
Stage-2-labeled message. (Alternative — extend the `Expr::Path` arm to resolve a bare nullary lower
rule — is larger and touches the elaborator; deferred unless a real use case appears. FLAG for the
maintainer.)

## §5 Interaction with the gates and later stages (task Q4)

- **Stage-3 (affine, OQ-H4) gate is orthogonal and untouched.** It runs *before* the Stage-2 gate in
  `check_sugar_call` (`ty_structurally_contains_substrate` at `checkty.rs:5605-5622`;
  `rhs_first_affine_binding` at `:5643-5650`; the Stage-2 gate at `:5654`). Stage 2 edits only
  `rhs_first_free_id` (+ a prim-name helper); it does not touch either affine check. Confirmed
  orthogonal by construction.
- **Cross-nodule/phylum stays refused.** The gate does **not** consult `self.imports` (DN-114 §3.1);
  Stage 2 keeps it that way. A cross-nodule (even same-phylum) free id remains a never-silent,
  Stage-4-directed refusal. This is *required for correctness*, not just scope-limiting: resolving a
  cross-nodule id against the **use-site** env (which is what Pass-1 does today) would bind the wrong
  meaning once def-env ≠ use-env — the exact hazard Stage 4 / DN-113 must design the def-site-import-context
  threading for.
- **Honest tag boundary.** Single-nodule def-site resolution / referential transparency moves
  `Declared → Empirical` **once §6's real-elaborator test lands** (currently only the E2 *prototype* is
  `Empirical`; the real path is *unattested*). VSA/float-prim acceptance and nullary-lower-rule
  position handling become `Empirical` on their fixtures. **Cross-nodule / cross-phylum stays
  `Declared`/open (Stage 4).** No tag upgraded past its checked basis (VR-5).

## §6 Non-vacuous test plan (task Q5) — the E1/E2 lesson: the real elaborator is *more* hygienic than the prototype

New in-crate white-box module `crates/mycelium-l1/src/tests/defsite_resolution_stage2.rs` (sibling to
`reachability_stage1b.rs`; white-box per `LowerDecl::value_params`' no-surface-grammar note). **Reuse
E2's fixture shape** (`hygiene_defsite_resolution.rs`: the `bump(v)=helper(v)` rule, def-site
`helper=x+100`, use-site shadow `let helper=λx.x-100`, resolved 105 vs captured -95) but drive it
through the **real** `elaborate(&env, "main")` entry — not the throwaway `DefEnv`/`Expander` — exactly
as `reachability_stage1b.rs::full_chain_step2` does.

1. **`stage2_defsite_resolution_real_elaborator`** — build a nodule with top-level `fn helper`, rule
   `bump`, and `main = let helper = <different> in bump(5)`; `check` ⇒ `Ok`; `elaborate` ⇒ an L0 term
   with (a) **no bare `Var("helper")`** (reuse E2's `contains_var`), and (b) `Interpreter::eval` == 105
   (def-site), asserted by both `alpha_eq` against a disjointly-spelled oracle **and** the eval
   differential (dual oracle, independent of `alpha_eq`).
2. **`stage2_control_use_site_shadow_would_capture`** — the **non-vacuity control** (the E2 lesson made
   load-bearing). Because the real elaborator resolves at Pass 1 (there is no "disable def-site
   resolution" flag — resolution is inlining, not a toggle), the control must **construct** the broken
   variant: hand-build the captured L0 (free `helper` left as a bare `Var`, E2's `captured` node)
   spliced under the same use-site `let helper` shadow, and assert `eval == -95 ≠ 105`. This proves the
   harness can *observe* a real referential-transparency break — so the passing test is discriminating,
   not vacuously green. (Mirror E2's `expected_captured != expected_resolved` + `contains_var` sanity
   assertions.)
3. **`stage2_vsa_or_float_prim_rhs_now_accepted`** (G1) — a rule whose RHS calls a VSA/float prim;
   `check` ⇒ `Ok` (was over-refused); `elaborate` ⇒ the expected `Node::Op`; plus a
   **`control_pre_fix_over_refused`** note verifying (during development, by reverting the helper) the
   old gate refused it — non-vacuity for the widening.
4. **`stage2_bare_nullary_lower_rule_value_position_refused`** (G2, if the narrow-the-gate fix is
   chosen) — asserts the never-silent, Stage-2-labeled refusal fires at **check** (not a residual at
   elab), closing the accept/elab mismatch honestly.
5. **Regression:** the existing `control_free_non_param_id_hits_stage2_residual`
   (`reachability_stage1b.rs`) must still pass for a **genuinely cross-nodule** free id — Stage 2 must
   **not** start accepting those. Add/keep a cross-nodule-shaped fixture that stays refused → Stage 4.

Every control is verified to **genuinely fail when its mechanism is disabled** (the DN-114 §5
non-vacuity law), hand-checked during development and restored before commit.

## §7 The argument *against* my own recommendation (VR-5 — no sycophancy, even to my own plan)

The strongest case *against* §2.2's "Option A, add no reference node":

> **OQ-H1 and E2 explicitly call for a content-addressed def-site *reference*, and inlining is not a
> reference.** A future `reveal` (M-1051, DN-106) that shows an *inlined* helper body — rather than a
> named `A::helper` reference — is arguably *less* transparent: the reader sees the expansion, not the
> provenance. And front-loading the explicit resolved-reference representation now (Option C) would let
> Stage 4 reuse it instead of re-touching the expander.

**Weighing it honestly.** The objection has real force on the `reveal`/provenance axis, and I do not
dismiss it. But three grounded reasons keep Option A ranked first *for Stage 2*: (1) deepdive **§6**
already ruled that content-addressing is **not** the load-bearing hygiene mechanism — so a
content-addressed *reference* buys inspectability, **not** soundness, and inspectability is **DN-106 /
M-1051 / OQ-H3's** design axis, not Stage 2's; (2) KC-3 forbids kernel growth without a discharged
need, and single-nodule has none (YAGNI); (3) the cross-nodule reference Stage 4 needs is a **qualified
symbol id via M-1024** (Option C), **not** the raw hash (Option B) OQ-H1's wording literally suggests —
so building Option B now would be building the *wrong* thing early. **Disposition:** Option A for Stage
2; **FLAG the `reveal`-provenance question to OQ-H3 / M-1051** (§8) so it is decided where it belongs,
not silently foreclosed here. If the maintainer values reveal-provenance over KISS, the ranking flips
to **C ≫ A** — I present both and defer the call (this is a maintainer decision, house rule #3).

## §8 Definition of Done (what maintainer/orchestrator ratification of this note requires)

Ratifying **this note** (`Draft → Accepted`) means the maintainer/orchestrator agrees, on the merits:

- [ ] **The §1 finding is accepted** — single-nodule def-site resolution already holds by construction;
      Stage 2 is *prove + close two gaps + document the invariant*, not *add resolution*. (If the
      maintainer disputes it, the counter-evidence is a single-nodule fixture where a use-site local
      *does* capture a def-site free id on the real elaborator — I could not construct one; §3.)
- [ ] **Q1 resolved-reference ranking (A ≫ C ≫ B) is accepted**, or flipped to **C ≫ A** if
      reveal-provenance is prioritized (§7) — an explicit maintainer call.
- [ ] **G2 fix chosen** — narrow the gate (KISS, recommended) vs. extend the `Expr::Path` arm (§4.2).
- [ ] **The §6 test plan is accepted** as the `Declared → Empirical` bar for single-nodule referential
      transparency on the real elaborator, with the non-vacuity control mandatory.
- [ ] **The Stage-4 boundary is confirmed** — cross-nodule/phylum free-id resolution stays refused and
      is DN-113 / M-1060's job; the single-nodule invariant (§3) is documented so Stage 4 is deliberate.

**Then** the implementation PR (a *separate* leaf, tracked by a new M-id — maintainer mints it) carries
its **own** DoD: gate edits + prim-name helper + `defsite_resolution_stage2.rs` land; change-scoped
gates green (`cargo fmt`/`clippy -D warnings`/`test -p mycelium-l1`); the honest tags move exactly as
§5 bounds; this note moves `Draft → Accepted` on that review. **Enacted** is *not* reached by this note
(house rule #3: no implementation lands here).

## §9 Scope / size (task Q6) — the clean cut

A tightly-scoped, well-under-budget reviewable unit (DN-97 ≈1–2k-LOC soft / 4,000 hard cap — this is a
few hundred LOC):

| File | Change | Est. |
|---|---|---|
| `crates/mycelium-l1/src/checkty.rs` | `rhs_first_free_id`: consult a VSA/float prim **name** predicate (G1); narrow nullary-`lower`-rule accept to call position (G2); doc-comment the single-nodule invariant (§3) | ~40–80 LOC |
| `crates/mycelium-l1/src/checkty.rs` or the prim module | a `prim_name_is_recognized`/`vsa_prim_names()`/`float_prim_names()` name view feeding G1 | ~20–40 LOC |
| `crates/mycelium-l1/src/elab.rs` | (only if Option C or the G2-extend alt is chosen — **not** in the recommended path) a documented same-nodule-invariant assert near the expander entry | ~0–15 LOC |
| `crates/mycelium-l1/src/tests/defsite_resolution_stage2.rs` | **new** — the §6 corpus + non-vacuity controls, reusing E2 fixtures | ~200–300 LOC |
| `crates/mycelium-l1/src/tests/mod.rs` | register the new test module | ~1 LOC |

**Explicitly deferred to Stage 4 / DN-113 / M-1060:** any consultation of `self.imports`; any
cross-nodule/phylum resolution; the explicit qualified-symbol / content-addressed **reference**
representation (Option C); def-site-import-context threading. All refused, never-silent, Stage-4-labeled.

## §10 FLAGs for the integrating parent / maintainer (append-only, dated — not edited from this note)

- **FLAG-Doc-Index** (2026-07-11): add a `docs/Doc-Index.md` row for DN-115.
- **FLAG-CHANGELOG** (2026-07-11): append a design-phase entry — "DN-115 (Draft): M-1054 Stage 2
  scoping — single-nodule def-site resolution already holds by construction; Stage 2 = prove +
  close two gate gaps + document the invariant."
- **FLAG-issues.yaml** (2026-07-11): M-1054's body should record (a) the §1 finding, (b) a **new
  Stage-2 implementation M-id** (maintainer mints) `depends_on` DN-115, (c) the two gate gaps G1/G2 as
  the concrete deliverables, (d) the §7 reveal-provenance question routed to OQ-H3 / M-1051.
- **FLAG-maintainer decision** (2026-07-11): the Q1 ranking (A ≫ C ≫ B vs. flip to C ≫ A for
  reveal-provenance) and the G2 fix choice (narrow-gate vs. extend-Path) are the two open calls in §8.
- **No `crates/mycelium-l1/` edit was made** — this note is READ-ONLY scoping on the semcore serial
  lane, per its owner-scope.

---

*Author: design-reasoner (Opus). Recommends; does not ratify (house rule #3). Every code citation is
`Empirical` at worktree tip `3cd7dbc2`; every design proposal is `Declared` until landed + tested
(VR-5).*
