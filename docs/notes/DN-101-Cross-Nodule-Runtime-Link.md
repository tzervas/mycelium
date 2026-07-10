# Design Note DN-101 — Cross-Nodule Runtime Link (the ENB-1 runtime-execution close)

| Field | Value |
|---|---|
| **Note** | DN-101 |
| **Status** | **Accepted** (2026-07-11, maintainer ratification — see the dated "Ratification / Maintainer decision" note below; the v0 flat-namespace policy of §5 option (A) is ratified as the shipped baseline, with the long-term choice between §5 options (B) and (C) left explicitly open for a dedicated planning pass). Originally **Draft** (2026-07-10), authored alongside the **first landable increment** of M-1024 (ENB-1). It records the design of the cross-nodule **runtime-execution** close, recommended a name-collision policy **for ratification**, and at Draft time **enacted nothing** and **moved no other doc's status** (house rule #3, append-only). Tags are `Empirical` where read against the code / witnessed by a running differential, `Declared` for any design not yet ratified (VR-5). |
| **Decides** | *Proposes, for ratification:* (1) that the runtime dual of the check-time import resolution (`resolve_imports`, M-662) is a **phylum-wide runtime link** producing one linked `Env` the existing elab/mono/eval pipeline consumes unchanged; (2) the **v0 flat-namespace** semantics (one declaration per simple name across the phylum) with a **never-silent refusal** on a cross-nodule name collision; (3) the **residual** to defer (AOT parity, qualified per-nodule scoping for collision *disambiguation*, the `.myc` runtime mirror). It does **not** edit `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (the integrating session owns those). |
| **Feeds** | DN-99 §A0 / register row #41 (ENB-1); M-1024; M-982 (the deferred collision-disambiguation follow-up); DN-26 (SCC self-hosting); DN-34 §4/§8.2 (the cross-nodule gap). |
| **Grounds on** | KC-3 (small kernel, no new L0 node), DRY (reuse the check-time registry), G2 (never-silent), VR-5 (no tag upgraded past its basis). |
| **Date** | July 10, 2026 |
| **Task** | M-1024 (ENB-1) — cross-nodule symbol resolution + runtime execution: the runtime half. |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This note records a design and a running
> increment; it does **not** take a decision (house rule #3 — the maintainer ratifies). Empirical claims
> are witnessed by `crates/mycelium-l1/tests/phylum_exec.rs` (a running differential against an inlined
> single-nodule oracle) read against the dev tip `8cd0a796`. The collision-policy **recommendation**
> below is `Declared` until ratified. **No sycophancy:** §3 flags where the tracker/plan lagged the code
> (direct cross-nodule execution *already worked*, undocumented — mitigation #14), and §6 states the
> residual plainly rather than claiming M-1024 whole.

---

## §1 Purpose

Close the **runtime-execution** half of cross-nodule symbols (DN-99 register row #41, ENB-1). The
**check-time** half is landed: `checkty::resolve_imports` (M-662) resolves every `use` against the
phylum-wide `pub` export table, refusing unknown/private/ambiguous imports. This note gives the
**evaluator** a phylum-wide view so a `use`d symbol actually *evaluates* across nodules — retiring the
local-mirror sidestep for execution (DN-99 §A0 rank 1; ADR-045 unfrozen posture).

## §2 What the code already did (verified, was undocumented)

Reading the checked-`Env` assembly (`checkty.rs::check_nodule_with`): a nodule's checking registry is
seeded from its resolved imports (`fns = imports.fns.clone(); fns.extend(own)`), and the returned
`Env.fns` **retains** the imported `pub` decls (only *own* fns are re-checked and overwritten). So a
consumer nodule's `Env` already carries its **directly** imported `pub` decls **with their bodies**.

**Consequence (witnessed):** a `use`d `pub` fn with a **self-contained body** already executes when the
consumer nodule's `Env` is run — `direct_cross_nodule_pub_fn_executes_equals_inlined_oracle` passes on
both the L1-eval and L0-elaborate legs. The prior `check_phylum` doc comment ("eval keeps its per-nodule
reach; a cross-nodule call lowers to a never-silent `Unsupported`/`Residual`") was **stale**; this note
corrects it. (The gap the plan named was real but *narrower* than "no cross-nodule execution.")

## §3 The real gap — transitive resolution

The consumer's `Env` has the imported `pub` fn, but **not** that fn's home-nodule **private** callees.
A `pub outer` whose body calls a private `inner` in its home nodule is `Stuck "unknown function inner"`
when run from the consumer's per-nodule `Env` (`transitive_case_without_link_is_a_never_silent_stuck` —
explicit, never a panic, never a wrong value; G2). Closing this needs the **whole phylum** visible.

## §4 The close — `PhylumEnv::link()` (the runtime dual of `resolve_imports`)

`link()` folds every nodule's **checked** declarations into **one** `Env`, keyed by **simple name**, that
the existing pipeline (`elab` / `mono` / `eval`, each already `&Env`-shaped) consumes unchanged. KC-3: no
new L0 node, no new evaluator mode. DRY: the check pass already validated every `use`; `link` merges each
name **from its home nodule's checked (ambient-resolved) `Env`** — the authoritative decl — and **never
merges an imported copy** (a less-resolved clone). So the linked `Env` is *strictly more correct* than
running a consumer's per-nodule `Env` directly, which is why a phylum — including a phylum-of-one — should
be run through `link` (`phylum_of_one_link_runs_identically_to_check_nodule`).

With the link, the transitive case resolves: `transitive_cross_nodule_private_helper_executes_via_link`
agrees with the inlined single-nodule oracle on both the L1 and L0 legs.

## §5 The decision to ratify — name-collision policy (v0 = flat namespace, refuse)

A flat merge cannot represent two nodules declaring the **same simple name** (two private `helper`s; or a
name that *shadows* an import — legal at check time, where own-shadows-`use`). The alternatives:

| Option | Semantics | Cost | Verdict |
|---|---|---|---|
| **(A) Flat namespace, refuse collision** *(implemented v0)* | One decl per simple name phylum-wide; a collision is an explicit `CheckError`, never a silent winner. | Small; no eval/mono/elab change. Rejects some check-legal programs (own-shadows-`use`, same-name privates). | **Recommended for v0** — runs the stdlib-porter shape (shared-type/helper nodule + distinct-named consumers) and is honest (G2) about what it cannot yet represent. |
| **(B) Qualified names + per-frame home-nodule scoping** | Each fn resolves simple names against its **home** nodule's scope; the evaluator threads a "current nodule" per frame. | Deep: touches eval's CEK frames + mono + elab. Fully general (handles collisions + shadowing). | **The end-state** (M-982). Too large for this increment. |
| **(C) Name-mangling to qualified keys** | Rewrite each nodule's private names to `nodule$name`, rewrite body references, flat-merge collision-free. | Medium: an AST/elab rewrite pass; subtle. | A viable middle path; still needs a rewrite pass + differential. Deferred. |

**Recommendation (`Declared`, for ratification):** ship **(A)** now; take **(B)** as the M-982 end-state
(it *disambiguates* a collision rather than refusing it). The refusal in (A) is the never-silent boundary
(G2) that keeps the increment honest until (B)/(C) is ratified — a collision is deferred *explicitly*, not
guessed. **This is the open decision the maintainer should ratify** before (A) is promoted past Draft.

## §6 Residual (flagged, not hidden — VR-5 / G2)

1. **AOT parity** — the witness exercises the interpreter + L0-elaborate legs only; the third (AOT/MLIR)
   leg is desktop-held and not run here. The DoD's "AOT" half of the two-witness claim is **open** (M-1024
   follow-up). *Do not read this increment as AOT-complete.*
2. **`.myc` runtime mirror — N/A, not skipped.** The self-hosted `lib/compiler/*.myc` frontend has **no
   evaluator** (semcore.myc header: the CEK machine, elab, and check_phylum's checking logic are unported,
   feasibility-gated on M-986/M-987 — "it verifies the compiler; it does not run programs"). The check-time
   dual (`resolve_imports`) is already ported (M-1013 STEP 4); the **runtime** link has no `.myc`
   counterpart to mirror until the `.myc` evaluator exists. DN-26 SCC parity is satisfied at the check
   tier; the runtime tier's `.myc` parity is **deferred behind the evaluator port**, not owed here.
3. **Collision disambiguation** — Option (B)/(C) above (M-982).
4. **Multiple-entry semantics** — a flat link exposes one `main` (or refuses two); which nodule's entry a
   multi-entry phylum runs is out of scope (the v0 phylum has one entry).
5. **Ambient-staleness of the *direct* per-nodule path** — a consumer's imported copy carries the
   *pre-resolution* body; identical for ambient-free fns (so §2 is sound), but a would-be-ambient imported
   fn run via the per-nodule `Env` (not `link`) could differ. `link` sidesteps this by using the home
   decl; the per-nodule direct path is superseded by `link` and should not be the run entry.

## §7 Definition of Done (this note + the increment)

- **DN DoD:** the design recorded; the collision policy put up for ratification with its alternatives; the
  residual enumerated. *(This note.)*
- **Increment DoD (the landable slice of M-1024):** a two-nodule phylum executes a `use`d symbol under the
  **interpreter** (direct **and** transitive), witnessed against an inlined oracle on the L1 + L0 legs; the
  runtime link **reuses** the check-time resolution (no duplicate resolution); an unresolved/colliding
  symbol is **never-silent** (explicit `Stuck` / `CheckError`, not a panic); change-scoped
  `cargo test -p mycelium-l1` green for the new suite; `fmt`/`clippy -D warnings` clean. **AOT parity and
  collision disambiguation are explicitly deferred** (§6) — M-1024 stays `todo`, not `done`.

---

## Ratification / Maintainer decision (2026-07-11)

> **Ratifies §5.** Maintainer: *"I'd rather determine if option C or B is best long term, hard work is
> fine, plan it out first, nail down what the api should look like, and go from there
> progressively/iteratively and keep the docs updated and aligned. Ratify."*

**Recorded decision (append-only — this note's original §5 text above is unchanged; this section adds
the ratification, per house rule #3):**

1. **DN-101 moves Draft → Accepted.** The §5 table's option **(A) flat namespace, refuse collision**
   is ratified as the **shipped v0 baseline** — it is what M-1024's first landable increment actually
   implements, and it stays the running semantics.
2. **The long-term choice is left open, not resolved by this ratification.** The maintainer's "option C
   or B" refers to the two deferred alternatives in the §5 table: **option (B) qualified names +
   per-frame home-nodule scoping** (the fully general end-state, tracked as M-982) and **option (C)
   name-mangling to qualified keys** (the medium-cost AST/elab rewrite-pass middle path). Which of
   these is best **long term** is explicitly **not decided here** — it is deferred to a **dedicated
   planning pass**: design the API shape first (what a cross-nodule reference/call site should look
   like under each option), then implement **progressively/iteratively**, keeping the docs updated and
   aligned as the design firms up.
3. **Follow-up filed:** **M-1048** — "DN-101 long-term cross-nodule-link API — determine option B vs C,
   design API, iterate" (`status:todo`, `doc_refs: corpus:DN-101`, `tools/github/issues.yaml`).

## Changelog

- **2026-07-11** — **Ratified (maintainer, house rule #3).** Status **Draft → Accepted**: the v0
  flat-namespace collision policy (§5 option A) is confirmed as the shipped baseline; the long-term
  option-B-vs-option-C choice is left open for a dedicated planning pass (API design first, then
  progressive/iterative implementation, docs kept aligned throughout). Follow-up filed as **M-1048**.
  Append-only — the original §5 design record above is unchanged; this is an added ratification note.
- **2026-07-10** — DN-101 created (**Draft**): records the cross-nodule runtime-link close (ENB-1 /
  M-1024), the `PhylumEnv::link` design, the v0 flat-namespace collision policy (recommended, not
  ratified), and the deferred residual (AOT parity, qualified scoping = M-982, `.myc` runtime mirror =
  M-986/M-987). Witnessed by `crates/mycelium-l1/tests/phylum_exec.rs`.
- **2026-07-10** — renumbered DN-100 → **DN-101** at integration (never-silent, G2): the number DN-100
  was concurrently taken by the M-1032 (ENB-9) macro-expand transpiler DN, which landed on `dev` first.
  Content unchanged; only the note number + its internal references were updated to avoid the collision.
