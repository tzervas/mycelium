# Kickoff `frz` — Phase-I H2: the Rust-reference closeout remainder + kernel-freeze (the closing act)

> **UID:** `frz` · **Basis:** **ADR-038** (Accepted, 2026-07-01) + the umbrella roadmap
> `docs/planning/road-to-1.0.0-and-mycelium-rewrite.md` **§4 (H2 — Rust-reference closeout
> remainder)** · **DN-56** (`Kernel-Completeness-And-Freeze-Criterion` — the four freeze conditions;
> 0/4 met today, default-DENY holds — DN-39) · **RFC-0038** (Accepted — inject-mode security axis,
> **design ratified, mechanism unbuilt**; enactment moves its claims `Declared → Enacted`) · **DN-63**
> + **M-828** (R2 runtime vocabulary; the remainder beyond `enb`'s D-lite subset) · **RFC-0027/DN-32**
> (memory model — the runtime-lane sequencing FLAG) · consumes **`grm`'s** H2a ratifications
> (RFC-0037 migration · DN-54 extension-checker · tuple · ADR-033 FLAG-1) and **`enb`'s** B/C/A/E
> prim lane (both are freeze preconditions).
> **Planned by:** Fable (ADR-038 §2.7); **implemented by:** Opus/Sonnet/Haiku per the PM table.
> **References the doc-maintenance contract** (`_doc-maintenance.md`) in its DoD.
> **Sequencing (roadmap §4/§8): Phase-I closeout.** Runs **after `enb`** (prims) and **alongside
> `grm`** (grammar); the **kernel-freeze declaration (M-969) is STRICTLY LAST** — a maintainer act,
> the sibling of `flp`'s usability ratification, taken only once its four conditions are checked
> (`enb` closes "primitive set closed", `grm` closes "lowering surface closed"). The non-freeze lanes
> (inject-mode, R2-buildable, l1-tail) run in parallel after `enb`. **Freeze is NOT a `flp` (public-
> flip) prerequisite** — spore identity is source-hash, so a moving kernel never blocks stdlib
> publishing; the freeze gives the public a *stable* baseline, not a *gating* one (roadmap §2b).

## Goal

Close the H2 "Rust-reference closeout remainder" — the tag-track items that are neither H0
foundations (`acy`), H1 usability enablers (`enb`), nor the H2a grammar-stability set (`grm`) — so
that Phase I ends on a **frozen, complete, honestly-tagged** kernel + surface. Four lanes:

1. **Kernel freeze (DN-56)** — the closing act: complete the reject-ledger, confirm the primitive
   set and lowering surface closed (against `enb`/`grm`), pass the KC-3 completeness review, then the
   **maintainer declares the freeze**. Runs last.
2. **Inject-mode enactment (RFC-0038)** — build the Accepted-but-unbuilt hot-inject security-axis
   mechanism (scope-confirmed with the maintainer), moving its claims `Declared → Enacted`.
3. **Runtime/concurrency maturity closeout** — the R2 vocabulary remainder beyond `enb`'s D-lite
   (M-828 tail), plus an honest RT2/determinism tagging pass (VR-5). The **heavy, needs-design
   runtime items are a research spike now / Phase-II build** — non-gating (see the split below).
4. **l1-semantics closeout tail** — the language items `grm` does not own: guard clauses (M-833,
   design-first), per-instantiation guarantee-tag context through monomorphization (M-844), the
   `Fuse` prelude + semilattice-law checker, and `via` delegation trait-registry ordering.

## ⚠ Maintainer decisions this kickoff carries (FLAG, never guess — G2/VR-5)

The dossier tasks *prepare* these; nothing here decides them.

| Decision | Prepared by | Gates | Ref |
|---|---|---|---|
| **RFC-0038 build-scope** — which of `whole`/`module`/`call` enforcement + the §8.4–§8.8 surface to *build now* vs defer as R&D (M-836…M-842/M-849) | M-960 | M-961 (enactment) | RFC-0038 §8 · DN-64 §7 |
| **RFC-0027/DN-32 memory-model ratification** — the runtime-lane sequencing precondition (confirm, don't guess) | M-962 | M-963 (R2 remainder) · the runtime-maturity split | RFC-0027 · DN-32 · roadmap §8 |
| **Guard-clause ratification** (`when <cond>` + guarantee propagation to the arm) | M-968 | its own implementation (post-ratification) | M-833 · roadmap §4 l1 lane |
| **R2 buildable-vs-research split** — which R2 constructs are Phase-I-buildable (directed) vs research-spike/Phase-II (the mesh gossip/Byzantine long pole) | M-962 | M-963 · the Phase-II hand-off | DN-63 · M-828/M-831 |
| **Kernel-freeze declaration** (DN-56, after its four conditions are checked) — the closing act | M-958/M-959 + `enb`/`grm` | M-969 (the act) · `flp` inherits a frozen baseline | DN-56 · DN-39 |

## Scope

**In:** the four lanes above + the freeze close-out (condition audit → reject-ledger → KC-3 review →
declaration). **Out:** the H2a grammar items (`grm` owns RFC-0037/DN-54/tuple/ADR-033 FLAG-1 —
`frz`'s freeze *consumes* them, never re-lands them); the B/C/A/E prims (`enb` owns them — freeze
preconditions); the mass `.myc` port (Phase II / `rwr`); the public flip (`flp`); any *new* surface
idea (a mid-kickoff addition is a FLAG, not a task). **Heavy runtime-maturity items — M-869
(AOT/interp async parity), M-868 (scheduler leapfrogging), M-831 (substrate/hypha reclamation) — are
`needs-design`/`research`: their design spike may start now, but they build in Phase II and DO NOT
gate the flip** (recorded here so the omission is explicit, not silent — G2).

## Swarm method + model tiering (ADR-038 §2.7)

**Small Hybrid-tiered swarm, serial through the l1 lane and the freeze gate.** The l1-tail tasks
(M-965/M-966/M-967) touch the `mycelium-l1` frontend — the repo's one serial lane — so they run one
at a time and **serialize with `grm`/`enb`'s l1 work** (land + pull down before the next). The
inject-mode lane (`mycelium-interp`/`mycelium-sec`) and the R2/runtime lane
(`mycelium-std-runtime`) are disjoint and run in parallel with each other and the l1 lane. The four
dossiers (M-958/M-960/M-962/M-968) are docs-only and run in parallel. `issues.yaml`, `CHANGELOG`,
`Doc-Index`, and `docs/api-index/` are **orchestrator-owned** — leaves FLAG, never edit. One
isolated worktree per leaf (mitigation #11); commit/push split (#12); scoped PRs to `dev` via
`/pr-land`. **M-969 (the freeze declaration) is a single supervised maintainer session** — never
fanned out, never automated past its checklist.

## PM decomposition — bite-sized tasks

Proposed M-ids **M-958…M-969** — next-free after `rwr`'s block (highest minted today is M-876;
`acy`…`opp` propose M-877…M-935, `flp` M-936…M-946, `rwr` M-947…M-957); **re-verify each slot at
minting** (mitigation #1). None minted by this doc. Verification-first: every task **verifies the
landed state before building** — the runtime pool (M-864/M-865) and the serial-closeout l1 work
(M-822…M-826) already landed, so nothing is re-landed and no condition is assumed closed.

### Lane A — Kernel freeze (DN-56; the anchor, closes last)

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-958 | **DN-56 four-condition audit dossier** — record the *current* state of each condition (reject-ledger completeness · primitive set closed · lowering surface closed · KC-3 completeness) with an honest `Empirical` verdict per condition; enumerate exactly what each still needs and which kickoff closes it (`enb` → primitives; `grm` → lowering surface) | As the maintainer, I want a checked, per-condition freeze scorecard, so that the freeze declaration is an evidenced act, not a vibe | All four conditions scored (`Empirical`, evidence cited); the remaining work per condition enumerated with its owning kickoff; DN-39 default-DENY re-affirmed as holding throughout | Haiku | — (docs; reads `enb`/`grm` status) |
| M-959 | **Reject-ledger completion** (DN-56 condition 1) — the exhaustive, never-silent ledger of every construct the kernel *rejects* (with the reason + the surface alternative), so a rejection is inspectable, not a silent gap (G2) | As a language user, I want every kernel rejection reified and explained, so that "the kernel won't do X" is a documented decision I can `EXPLAIN`, not a surprise | Ledger complete against the current reject set; each entry `{construct, reason, alternative}`; a regression guard fails if a reject path is added without a ledger row | Sonnet | M-958 |
| M-969 | **THE KERNEL-FREEZE DECLARATION (strictly last; one maintainer act):** with all four DN-56 conditions checked (M-958 scorecard green: `enb` primitives closed · `grm` lowering surface closed · M-959 reject-ledger complete · KC-3 review passed), the maintainer declares the kernel frozen; DN-56 → the freeze recorded append-only; DN-39 promotions (if any) are the *only* permitted kernel diff thereafter | As the maintainer, I want the freeze to be one deliberate, condition-gated act, so that "the kernel is stable" is a checked declaration a public release can stand on | Executed ONLY after the four-condition scorecard is green (recorded); DN-56 status advanced append-only; the post-freeze diff policy (DN-39-only) stated; CHANGELOG logs the act | Opus | M-958, M-959, **`enb`** (primitives), **`grm`** (lowering surface), + maintainer gate |

### Lane B — Inject-mode enactment (RFC-0038 → Enacted)

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-960 | **Inject-mode build-scope dossier** — from RFC-0038 §8's Accepted design, recommend the *Phase-I-buildable* subset (the `whole`-app compile/load-time signature default + `InjectCert`/`TrustRoot` verify path) vs the deferred R&D (§K.2/§L/§M key-management, replay/expiry, controller-mode topology — M-836…M-842/M-849); every deferral flagged, none dropped | As the maintainer, I want the inject-mode build scoped to what a public release actually needs, so that I enact a coherent slice, not the whole R&D surface at once | Buildable subset vs deferred R&D enumerated with a recommendation (⟐); each deferral mapped to its R&D issue; no silent scope drop (G2) | Sonnet | — (docs; RFC-0038 Accepted) |
| M-961 | **Enact the confirmed inject-mode subset** — build `loose`/`inoculated` mode gating, `InjectCert` verify against a colony `TrustRoot`, the never-silent `InjectError::UnsignedCode`/`BadSignature` refusals, and the default-plus-deviations manifest; three-way differential where a path is executable; RFC-0038's claims move `Declared → Enacted` **only** for what is actually built (VR-5 — the rest stays `Declared`) | As a deployer, I want production code to require a valid inject signature and refuse unsigned injection never-silently, so that the security axis is real, not declared | The confirmed subset built + tested; refusals never-silent; only the built claims flip `Declared → Enacted` (the deferred surface stays `Declared`, flagged); RFC-0038 status note appended | Sonnet | M-960 |

### Lane C — Runtime/concurrency maturity closeout

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-962 | **R2 remainder + memory-model sequencing dossier** — scope the R2 vocabulary beyond `enb`'s D-lite (M-828 tail): which constructs are Phase-I-buildable/directed vs research-spike (the mesh gossip/Byzantine long pole, M-831); confirm the RFC-0027/DN-32 memory-model ratification the runtime lane sequences on | As the runtime effort, I want a clean line between build-now R2 and research-later R2, so that Phase I closes a coherent runtime surface without blocking on the long pole | The buildable-vs-research split recorded (⟐); the RFC-0027/DN-32 precondition confirmed or FLAGged; the research items (M-831) marked Phase-II, non-gating | Sonnet | — (docs; DN-63/M-828) |
| M-963 | **Activate the directed R2 remainder** (M-828 tail) — the buildable constructs the dossier confirms, per DN-63; mechanized `SelectionPolicy` capture/setting where directed; never-silent refusal for the unbuilt ones | As a runtime programmer, I want the confirmed R2 vocabulary usable, so that `std.runtime`'s surface is complete for what Phase I ships | The confirmed subset activated + tested (three-way where executable); the unbuilt constructs refuse never-silently; honest tags | Sonnet | M-962 |
| M-964 | **RT2/determinism honesty pass** (VR-5) — audit every runtime/concurrency guarantee tag; a determinism claim stays `Empirical` unless a machine-checked side-condition upgrades it; no `Proven` without a checked basis | As a reviewer, I want the runtime's guarantees tagged at their real strength, so that "deterministic" means checked, not hoped | Every runtime tag audited; upgrades justified by a checked side-condition or reverted to `Empirical`; the audit recorded | Haiku | — (audit) |

### Lane D — l1-semantics closeout tail (serial with `grm`/`enb`)

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-965 | **`Fuse` prelude + semilattice-law checker** (F-A1/F-A2) — the prelude surface + a checker that the declared join/meet obey idempotence/commutativity/associativity; a law violation is a never-silent refusal, not a silent accept | As a concurrency author, I want my `Fuse` lattice's laws checked, so that a mis-declared join is caught at definition, not in production | Prelude landed; the three laws checked; violations refuse never-silently; property test per law | Sonnet | **`grm`** (grammar-stable) |
| M-966 | **`via` delegation trait-registry ordering** — a deterministic, `EXPLAIN`-able resolution order for `via`-delegated trait methods; ambiguity is a never-silent refusal | As a library author, I want `via` delegation to resolve deterministically and inspectably, so that method dispatch is never a silent surprise | Ordering deterministic + `EXPLAIN`-able; ambiguity refuses never-silently; tests over the ordering cases | Sonnet | M-965 (l1 serial) |
| M-967 | **M-844 — per-instantiation guarantee-tag context through monomorphization** — thread the guarantee-tag context per instantiation so a monomorphized copy carries its call-site's tag, not a lost/merged one (VR-5 — tags must not silently upgrade across mono) | As the transparency system, I want each monomorphized instance to keep its own guarantee tag, so that provenance survives specialization | Tag context threaded per instantiation; a differential shows no tag is silently upgraded/lost through mono; tests over mixed-tag instantiations | Sonnet | M-966 (l1 serial) |
| M-968 | **Guard-clause dossier** (M-833, design-first) — prepare `when <cond>` guards + guarantee propagation to the guarded arm for maintainer ratification; enumerate the exhaustiveness/propagation semantics; nothing built until ratified | As the maintainer, I want guard clauses specified before they are built, so that the ratification is over a concrete semantics, not a sketch | Semantics enumerated (exhaustiveness + guarantee propagation); ratification-ready; FLAGged as decision-gated (no implementation in this task) | Sonnet | — (docs; M-833) |

## Definition of Done (kickoff)

+ **Kernel freeze:** DN-56's four conditions scored (`Empirical`, M-958), reject-ledger complete
  (M-959), and — once `enb`/`grm` close their conditions — the maintainer's freeze declaration
  executed (M-969) with the DN-39-only post-freeze diff policy stated; DN-56 advanced append-only.
+ **Inject-mode:** the maintainer-confirmed subset built + tested (M-961); RFC-0038's claims moved
  `Declared → Enacted` **only** for what is built (the deferred R&D stays `Declared`, flagged).
+ **Runtime closeout:** the directed R2 remainder activated (M-963); RT2 tags honest (M-964); the
  heavy needs-design items (M-869/M-868/M-831) recorded as Phase-II/non-gating, none silently
  dropped.
+ **l1 tail:** `Fuse` law-checker, `via` ordering, and per-instantiation tag context landed; guard
  clauses ratification-ready (M-968, decision-gated).
+ Every op honestly tagged (VR-5); every refusal never-silent (G2); property test per bound;
  doc-maintenance per `_doc-maintenance.md`; every FLAG raised, none guessed. Land as scoped PRs to
  `dev` via `/pr-land`.

## Prerequisites

1. **`enb` landed** (the B/C/A/E prim lane) — the freeze's "primitive set closed" condition and the
   `consume`-execution the l1 tail assumes.
2. **`grm` landed** (the H2a grammar-stability set) — the freeze's "lowering surface closed"
   condition and the grammar the l1-tail serial work writes against.
3. **Maintainer gates:** RFC-0038 build-scope (M-960), RFC-0027/DN-32 memory-model ratification
   (M-962), guard-clause ratification (M-968→M-833), and the **kernel-freeze declaration** (M-969) —
   the freeze never front-runs its four checked conditions.
