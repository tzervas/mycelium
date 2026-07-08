# DN-88 — Component-Repo Decomposition & the Managerial Re-Export Topology (Post-Dogfood, GHCR Spores)

| Field | Value |
|---|---|
| **Note** | DN-88 |
| **Status** | **Proposed** (2026-07-07; planning capture, DN-17 posture — advisory, decides nothing normatively) |
| **Task** | proposed tracking issue **M-1020** under a new epic **E40-1** (ids **proposed, not minted** — mitigation #1; the orchestrator must verify both slots are still free before minting). No `docs/planning/phase-8.md` exists yet at authoring time — this note's own task is provisionally "Phase 8" per the brief that commissioned it; **FLAGGED**, not assumed (§8.8) |
| **Related** | **DN-27** (the original 2026-06-23 post-1.0.0 decomposition capture — this note **refines**, does not supersede, its trigger and elaborates its "phylum re-export repos" line into a concretely worked pattern; DN-27 stays **Draft**, untouched, append-only); **DN-28** (registry architecture / content-hash-DAG + fetch-and-verify model this topology publishes into); **ADR-013** (`spore` = the deployable unit — what each component repo publishes); **ADR-037** (**Enacted** — GHCR/OCI is already the binding v0 registry backend; this note does not redecide it); **Spore-Build-and-Publish-Contract** (`mycelium-proj.toml` → spore, incl. §10 the GHCR remote backend); **DN-18** (native-deploy spore schema — a sibling artifact-embedding precedent); **RFC-0016** (stdlib scope/taxonomy — the worked exemplar, §4); **RFC-0031** (self-hosted stdlib composition — D1 ring/layer boundary, D2 phylum layout, D7 spore packaging, all reused below); **ADR-036/ADR-038** (dogfooding + release-strategy — the precondition's grounding, §3); **DN-68** (the Rust-workspace acyclic-deps invariant — the precedent this note's dependency-layering criterion generalizes, §5) |
| **Grounding** | `lib/std/mycelium-proj.toml` (today's single-phylum stdlib — the concrete "before" state, read verbatim below); `crates/mycelium-proj/src/manifest.rs` (`[surface].exports` typed as own-nodules-only, v0); `docs/spec/grammar/mycelium.ebnf` + `.claude/memory/lang-lexicon-syntax.md` (`phylum` reserved-not-active; `use`/`pub` scoped to one project) |
| **Guarantee** | **`Declared`** throughout — a forward plan; nothing here is implemented, enacted, or checked. Every claim is tagged inline where it draws on checked source vs. the maintainer's stated intent. |

> **The maintainer's vision (2026-07-07, captured near-verbatim).** Once the entire Rust codebase is
> dogfooded into Mycelium-lang and reaches a **production-ready** state, the monorepo's Mycelium
> nodules and phyla get decomposed into **component repos + executable packages (spores)**, one per
> component, each **stowed in GHCR** — with **re-exporting managerial phyla/nodules** where
> appropriate: a group's top-level phylum re-exports its member component phyla to present a
> **clean, somewhat unified interface** for grouped packages. Example: the **stdlib** becomes a set
> of component repos (`std-io`, `std-fs`, `std-vsa`, `std-numerics`, …), and the **stdlib phylum** is
> the single point of entry that re-exports them. The **same pattern** applies elsewhere — compiler,
> runtime, capability groups, wherever a group benefits from one managerial entry point.
>
> This note records the **topology, criteria, and workflow** so that intent is durable and ready when
> the maintainer kicks it off. The actual **decomposition guide** and the **component→repo mapping**
> are produced **later**, gated on production-ready (§3) — this note does not enumerate them.

---

## 1. Status / posture

**Proposed / `Declared`**, advisory (DN-17 posture: a planning capture, not a ratified decision).
This note **enacts nothing** — no repo is created, no CI is wired, no `mycelium-proj.toml` is split,
no code changes as a result of it. It **supersedes no existing decision** (append-only, house
rule #3): it **builds on** ADR-013 (spore = deployable unit), ADR-037 (GHCR/OCI is the binding v0 registry
backend), DN-28 (the content-hash-DAG + fetch-and-verify distribution model), and the
Spore-Build-and-Publish-Contract (`mycelium-proj.toml` → spore) — none of which this note redecides —
and it **refines** DN-27 (the original decomposition capture), which stays `Draft` and unedited.

The refinement, stated honestly: DN-27 (2026-06-23) gated the decomposition on "post-`lang 1.0.0`",
which at the time meant the ADR-022 Rust-reference tag. ADR-038 (Accepted, 2026-07-01) has since
**split** that meaning — the public-release flip now happens far earlier, at **Phase I functional
usability** (a `0.x`), while the **public semver's `1.0.0`** was redefined to mean "fully rewritten
into Mycelium (where appropriate) and 100% operational" (ADR-038 §2.8) — a **later**, terminal state.
DN-27's decomposition intent tracks that terminal state, not the Phase-I flip; this note pins the
precondition to it explicitly (§3) and adds the two things DN-27 named as future work: a **worked**
managerial re-export pattern (§4, with the phylum-re-export-mechanism finding) and the **mapping
criteria + method** (§5, not the mapping itself — DN-27 §5 explicitly deferred that to "a future
ADR"/"the maintainer draws the actual per-component mapping"). ADR-038's own changelog already notes
DN-27's "mechanics ADR stays future work at Phase-II kickoff" — this note is exactly that pre-staging,
still short of the binding ADR.

## 2. Goal

Once the monorepo's Mycelium nodules/phyla are production-ready (§3), decompose them into **per-
component repos**, each publishing an independently-versioned **spore** (ADR-013) to **GHCR**
(ADR-037 — already the binding registry backend; this note publishes *into* that model, it does not
redesign it). Where a group of components is naturally consumed together, a **managerial re-export
phylum** presents one clean, unified entry point over its member component phyla — so a consumer who
wants "the stdlib" still imports one thing, even though the stdlib is now N repos underneath.

This note sits **on top of** four already-decided pieces, cited rather than restated:

- **ADR-013** fixes *what* is published — a spore, a content-addressed DAG of code + values +
  reconstruction manifest + metadata.
- **ADR-037** fixes *where* it is published — GHCR, as an OCI 1.1 artifact, per the DN-28 dense-map
  (object blobs + config blob + tag).
- **DN-28** fixes the *distribution model* — a content-hash DAG (the map) plus fetch-and-verify
  against a content store, not the registry storing full bytes.
- **Spore-Build-and-Publish-Contract** fixes *how a project becomes a spore* — the
  `mycelium-proj.toml` → build pipeline → identity-vs-metadata split (ADR-003) → `EXPLAIN`.

**What this note adds:** the multi-repo split **topology** — how a single monorepo's phyla become
many component repos plus managerial re-export phyla, on what criteria, in what order. None of the
above four are re-litigated.

## 3. Precondition — the two-stage production-ready gate

**Decomposition does not begin until the dogfood drive reaches the maintainer's stated bar:** every
ported nodule/phylum must be, **in order**:

1. **Interpreter-runnable, with a 100% checkout of the interpreted variant** — the Mycelium-lang
   reimplementation runs correctly under the trusted reference interpreter, with full (not partial)
   coverage of its Rust-reference behavior; and only then
2. **AOT-compiled `.myc`** — the same nodule/phylum is also compiled through the native/AOT path.

Transpiling (the existing DN-34/M-1006-ladder machinery) knocks out the mechanical bulk of a port;
hand-patching brings each ported unit the rest of the way to production readiness. This is the
maintainer's forward-looking bar, recorded here **`Declared`** — it is a new, more granular
elaboration than anything currently checked-and-gating in the corpus, so it is stated as intent, not
claimed as an already-operational metric (VR-5).

**Grounding, and one honest gap:**

- **ADR-038 §2.8** already fixes the *terminal* state this precondition operationalizes: the public
  semver climbs `0.x → 1.0.0` tracking the Mycelium rewrite, and **`1.0.0` ≡ "fully rewritten into
  Mycelium (where appropriate) and 100% operational."** The two-stage bar above is this note's
  concrete, per-unit reading of that terminal state — checkable per nodule/phylum rather than only as
  a whole-project label.
- **ADR-036 decision (3)** already establishes the *validation mechanism* a "production-ready" claim
  would lean on: each Mycelium reimplementation is differential-validated against its Rust reference,
  extending the existing interp≡AOT≡JIT discipline (RFC-0029 §7.5, the shared M-210 checker) to a
  **Rust≡Mycelium** axis. Stage (1) above ("100% checkout, interpreted") and stage (2) ("AOT-compiled")
  read naturally as two checkpoints along that same differential ladder — but this note does **not**
  claim that mapping is already wired to a specific metric; it is a plausible reading, flagged as such
  (§8.5).
- **DN-34** (the transpiler strategy) and the **M-1006 ladder** / `myc-drafts` skill are the concrete
  machinery behind "transpiling knocks out the bulk": the transpiler is a **gap-profiling instrument**
  (the M-991 verdict, DN-34 §8.7–§8.8) whose `checked_fraction` (myc-check-clean) vs
  `expressible_fraction` (text emitted) split is exactly the "mechanical bulk vs. hand-patch"
  distinction this precondition describes — draft `.myc` graduates into `lib/` only via hand-vetted,
  differential-witnessed porting (M-993).
- **Do not conflate this with the ADR-038 Phase-I public-release flip.** Phase I (functional
  usability, a `0.x` semver) is an **earlier, separate** gate — the repo can go public long before
  every nodule/phylum clears this note's two-stage bar. Decomposition is gated on the **later**,
  terminal state. This distinction is easy to blur (both involve "going public"/GitHub) and is
  FLAGGED explicitly in §8.6 so a future reader does not shortcut it.

## 4. The managerial re-export pattern

**The pattern:** a group of related, independently-useful components each ship as their own
**component phylum** (one repo, one `mycelium-proj.toml`, one GHCR spore). A **managerial phylum**
sits above them, depending on each by content hash (the existing hash-authoritative
`[dependencies]` model — Spore-Build-and-Publish-Contract §4) and **re-exporting** their public
surfaces as its own — so a consumer who imports only the managerial phylum sees one clean surface,
never the N-way split underneath, exactly as if it were still the single monorepo phylum it used to
be.

### 4.1 Worked exemplar — the stdlib

The stdlib is the clearest exemplar because its internal taxonomy is **already ratified** (RFC-0016
§4.2–§4.4): a three-ring layering (Ring 0 kernel-adjacent re-exports, Ring 1 capability surfaces /
Tier A, Ring 2 general library / Tier B) over a 23-module taxonomy, and — grounded directly in the
current tree — the stdlib is **today one single phylum**:

```toml
# lib/std/mycelium-proj.toml (verbatim, as of this note)
[project]
name = "std"
kind = "phylum"
...
[surface]
exports = ["std.result", "std.option", "std.cmp", "std.iter", "std.collections", "std.math",
           "std.text", "std.fmt", "std.core", "std.diag", "std.error", "std.recover",
           "std.select", "std.spores", "std.swaps", "std.ternary", "std.testing"]
```

One manifest, one `[surface].exports` list, one spore. **After** decomposition (illustrative only —
the actual split is future work per §5, not committed here), the pattern would read: each module (or
a cohesive cluster of modules) becomes its own component phylum/repo — `std-io`, `std-fs`, `std-vsa`,
`std-numerics`, `std-collections`, … — each with its own `mycelium-proj.toml`, its own GHCR spore
(`ghcr.io/<owner>/std-numerics:<version>`), independently versioned. A top-level **`stdlib`**
managerial phylum depends on the set it groups and re-exports their surfaces, so
`use stdlib.numerics.*` (or the equivalent) still works exactly as `use std.numerics.*` did before
the split — the **unified interface** the maintainer's vision names.

### 4.2 Does the re-export mechanism exist today? — FLAG, checked

**No — it needs a new mechanism.** Grounded findings:

- **`phylum` is reserved-not-active.** It lexes as a keyword (so it can never silently become an
  identifier — G2) but **no construct consumes it yet**
  (`.claude/memory/lang-lexicon-syntax.md` §"Reserved-not-active"; verified against
  `crates/mycelium-l1/src/lib.rs`'s `phylum_and_colony_are_reserved_not_active` test). There is
  today no `.myc`-level statement that names another phylum and re-exports part of it.
- **`[surface].exports` is own-nodules-only, v0.** `crates/mycelium-proj/src/manifest.rs` types it as
  "the public nodule names a phylum germinates from" — a flat list of the *project's own* dotted
  nodule names (confirmed against `lib/std/mycelium-proj.toml` above: every entry is `std.<module>`,
  never `<other-phylum>.<module>`). There is no field today letting a manifest re-export a
  **dependency's** exported surface as its own.
- **`use`/`pub` are scoped to one project.** The grammar's `use_item` imports a dotted path and `pub`
  governs cross-*nodule* visibility (`docs/spec/grammar/mycelium.ebnf`); neither is documented as
  spanning a `[dependencies]` edge into another phylum's surface.

**Conclusion:** the managerial re-export phylum pattern needs **one of two new mechanisms**, neither
decided here: (a) a `[surface].exports` grammar/schema extension letting an entry reference a
dependency phylum's exported nodule, or (b) activating `phylum` as a live construct with a `.myc`
-level cross-phylum re-export statement (a `pub use`-equivalent that crosses a `[dependencies]` edge).
This is **FLAGGED as a likely follow-on RFC** (§8.1) — the decomposition cannot execute the "clean
unified interface" half of the vision until one of these lands; the plain multi-repo split (component
repos alone, no managerial re-export) is executable today without it.

### 4.3 Generalizing beyond stdlib

The maintainer's vision states the same pattern applies "wherever a group benefits from one
managerial entry point." Named as **candidates only** (not committed — the full group list is a
maintainer decision at kickoff, §8.2):

- **The compiler/toolchain**, once ADR-038 Phase II's progressive self-hosting proceeds
  (lexer/parser/checker/elaborator/monomorphizer/codegen as candidate components under one
  managerial `compiler`/`toolchain` phylum).
- **The runtime**, once the RFC-0008 `hypha`/`colony`/scheduler/supervision constructs activate
  (currently mostly reserved-not-active vocabulary) — candidate components under one managerial
  `runtime` phylum.
- **Capability groups** — e.g. the numerics/swap/vsa/dense cluster (RFC-0016 Tier A, "differentiator"
  modules already sharing a ring) could be one managerial group distinct from the Tier-B "common"
  cluster, if usage patterns favor that split over a flat stdlib split.

## 5. Component→repo mapping criteria + method

This section fixes **how** to decide which nodules/phyla belong in which component repo — not the
mapping itself (the maintainer draws that at decomposition time, per group, per §6).

**Criteria:**

1. **Disjoint-ownership seams.** A component-repo boundary should already be a directory/crate
   boundary with no other component reaching into it — the same rule the swarm pattern already uses
   for parallel-agent file ownership (CLAUDE.md "partition by file ownership, not just by task").
   Boundaries that already exist as separate `mycelium-std-*` crates or `lib/<x>` directories are the
   natural first candidates.
2. **Cohesion, not a line-count target.** Mirrors the ≈1–2k-LOC-delta PR guideline's spirit (DN-65):
   a component repo should hold one cohesive, independently-versionable unit; components that are
   always co-consumed should stay merged rather than being split for the sake of a smaller repo
   (KISS/YAGNI) — cohesion wins over granularity.
3. **Dependency layering stays acyclic.** A component repo may not import "above" its layer: RFC-0016's
   ring layering (Ring 0 kernel-adjacent < Ring 1 capability surfaces < Ring 2 general library) and
   RFC-0031 D1's irreducible-Rust boundary already order the stdlib this way; carrying that ordering
   into the repo graph makes it acyclic by construction, the same discipline DN-68 already documents
   and enforces for the Rust workspace (`xtask/src/deps/` + `deps-strata.toml`) — the multi-repo
   version of the identical invariant.
4. **Content-addressed identity is repo-location-agnostic.** A component's spore identity (ADR-003)
   is computed from its `mycelium-proj.toml` root's own code/deps/surface (Spore-Build-and-Publish-
   Contract §3) — nothing in that pipeline depends on the project living inside the monorepo. Splitting
   a phylum into its own repo must **not** change its spore identity for unchanged code; this is a
   checkable acceptance criterion at split time (§6.3), not a design constraint on the mapping itself.
5. **The re-export boundary is the "always co-consumed" test.** RFC-0031 D7 kept the stdlib as one
   spore because "the stdlib is consumed as a whole, so per-nodule spores would multiply the
   dependency surface with no isolation benefit at this maturity" — and named a future split as
   "deferred to when a consumer needs the smaller surface." That is exactly this note's trigger
   signal: once dogfood-complete usage shows consumers regularly depending on a narrow slice (e.g.
   only `std.io`), that slice is a split candidate — mirroring DN-27's original motivating story ("I
   want to depend on a small component repo … so my dependency surface is minimal").

**Method (not the mapping):**

1. Start from the group's **already-documented internal taxonomy** — for stdlib, RFC-0016 §4.3/§4.4's
   Tier-A/Tier-B table is very likely already the seam; an un-dogfooded group (compiler, runtime)
   would need its own analogous internal taxonomy drafted first.
2. **Verify acyclicity** of each candidate split against the ring/layer graph (criterion 3) — reuse
   the DN-68/`xtask deps` style of check, generalized from crates to repos.
3. **Verify independent buildability** of each candidate against the Spore-Build-and-Publish-Contract
   §3–§4 pipeline — does the candidate have a self-contained `[surface].exports` and fully
   hash-pinned `[dependencies]`? A candidate that cannot build standalone is not yet a valid
   component-repo boundary.
4. **Size/cohesion-check** (criterion 2) — merge candidates that are always co-consumed rather than
   forcing a split for its own sake.
5. **Draft the managerial phylum's re-export list** as the union of the chosen components' surfaces —
   executable only once §4.2's mechanism lands.

## 6. Workflow plan

Run **per group** (stdlib is the most likely first group, given its taxonomy is already ratified),
**minted per-wave**, not as one big-bang split of the whole monorepo:

1. **Confirm §3's precondition** holds for every nodule/phylum in the group — never decompose a group
   still short of the interpreter-100%/AOT bar.
2. **Apply §5's method** to the group's existing taxonomy to produce the candidate component list and
   the managerial phylum's member list (the maintainer's per-component mapping pass — out of scope
   here, produced by the future decomposition guide).
3. **Carve each component into its own repo** — a fresh `mycelium-proj.toml` phylum root per
   component. History-carry (full git history vs. a clean start at the split point) is an **open
   question** (§8.3, inherited unresolved from DN-27). Verify content-addressed identity survives the
   split unchanged (criterion 4) — a pre/post-split differential over the spore id and its conformance
   suite is the acceptance gate, the repo-split analogue of the swarm's "verify the merge landed the
   whole set" discipline (mitigation #7).
4. **Wire the managerial re-export phylum** — once §4.2's mechanism is ratified (the follow-on RFC),
   its `[dependencies]` names each component phylum by content hash + version (the existing
   hash-authoritative model), and its re-export surface presents the group's unified interface.
5. **Publish** each component spore and the managerial spore to GHCR (`spore publish --registry
   ghcr://<owner>` — the already-Enacted ADR-037 path), each independently versioned/tagged.
6. **Verify the unified interface** — a consumer depending on only the managerial phylum must see the
   same public surface as depending on the pre-split monorepo phylum: a differential/conformance
   check, the repo-split analogue of RFC-0031 D5's per-op stability bar.
7. **Per-repo CI** — each component repo gets its own scoped check pipeline, mirroring the
   tier-scoped-testing discipline CLAUDE.md's Concurrent-PR development section already establishes
   for PRs, applied here to repos. The concrete wiring is decided at decomposition time, not designed
   here.

## 7. User stories + Definition of Done

**User stories — this note:**

- As the **maintainer**, I want the decomposition's topology, mapping criteria, and workflow recorded
  now, ahead of the dogfood drive completing, so present phylum/nodule boundaries (already being drawn
  under RFC-0016/RFC-0031) are shaped toward a clean future split instead of being painted into a
  corner (restates DN-27 §1's identical story, updated against the now-Enacted GHCR backend and the
  now-ratified two-stage precondition).
- As a **future decomposition-guide author**, I want mapping **criteria** — not a pre-drawn mapping —
  so I am not re-litigating "what makes a good component repo" from scratch when the gate closes.
- As a **downstream consumer** (post-decomposition), I want a managerial phylum's re-export surface to
  be behaviorally indistinguishable from today's single monorepo phylum, so the multi-repo split is
  never a breaking change on its own.
- As a **language/tooling maintainer**, I want the phylum-re-export mechanism gap named explicitly
  (§4.2) so it is scheduled as a real RFC rather than discovered as a blocker mid-decomposition.

**Definition of Done — this note (the "ready to execute" gate):** recorded (a) the precondition (the
two-stage bar, grounded against ADR-038/ADR-036/DN-34, §3); (b) the managerial re-export pattern
worked concretely on stdlib, with the phylum-re-export-mechanism-exists-or-not finding (§4); (c)
mapping criteria + method, not the mapping (§5); (d) an ordered per-group workflow (§6); (e) user
stories + DoD for both this note and the eventual guide (§7); (f) every open question flagged (§8).
Nothing is implemented; nothing is enacted; DN-27 is untouched (append-only, house rule #3).

**Definition of Done — the eventual guide (future work, out of scope here):** a binding ADR/RFC that
(1) ratifies the phylum re-export mechanism (§4.2's FLAG resolved), (2) draws the actual
per-component mapping by applying §5's method per named group, and (3) specifies the CI/versioning/
history mechanics §6 leaves open — gated on §3's precondition closing, drafted at the Phase-II kickoff
ADR-038 already points DN-27's mechanics work toward.

## 8. Open questions / FLAGs for the maintainer

1. **Phylum re-export mechanism (§4.2).** Needs a new RFC — either a `[surface].exports`/
   `[dependencies]` schema extension or an activated `phylum` construct with a cross-phylum `pub
   use`-equivalent. Not decided here; the plain multi-repo split (no managerial re-export) is
   executable without it, but the "clean unified interface" half of the vision is not.
2. **The group list.** Stdlib is the only worked exemplar (§4.1); compiler/runtime/capability groups
   are named as candidates only (§4.3). The actual full group list is the maintainer's call at
   kickoff — deliberately not committed here (mitigation #1: don't pre-mint the whole ladder).
3. **History carry.** Full git history per component repo, or a clean start at the split point?
   Inherited, unresolved from DN-27 §5 — this note does not resolve it either.
4. **Re-export form.** Source re-export vs. spore-artifact-only re-export vs. both — inherited,
   unresolved from DN-27 §5; tied to §4.2's mechanism choice (a `.myc`-level construct would make
   source re-export natural; a manifest-only mechanism would likely stay artifact/spore-level only).
5. **How is the §3 precondition operationally checked, per nodule/phylum?** This note reads it as two
   checkpoints on the existing Rust≡Mycelium differential ladder (ADR-036 decision 3) but does not
   specify a concrete metric (an analogue of the transpiler's `checked_fraction`). FLAG for whoever
   drafts the binding ADR.
6. **Do not conflate with the ADR-038 Phase-I public-release flip.** The public repo can go live (a
   `0.x`, functional usability) well before this note's terminal decomposition gate closes — these are
   two different milestones that happen to both involve "going public." Stated explicitly in §3 to
   prevent a future shortcut.
7. **Tooling across N repos.** Does the issue-tracker/changelog reconciliation extend across N public
   repos, or stay one orchestrating repo? Inherited, unresolved from DN-27 §5 open question 4.
8. **Ids proposed, not minted.** This note proposes tracking issue **M-1020** under a new epic
   **E40-1**, provisionally under "Phase 8" (no `docs/planning/phase-8.md` exists yet at authoring
   time). The orchestrator applying this note's proposed issue entry must re-verify both slots are
   still free (mitigation #1) and decide whether a `phase-8.md` planning doc is warranted or whether
   this task belongs under an existing phase's tail.

## Meta — changelog

- **2026-07-07 — Created (`Proposed`).** Captures the maintainer's forward-looking decomposition
  vision (component repos + GHCR spores + managerial re-export phyla) near-verbatim, building on the
  already-decided artifact/registry model (ADR-013, ADR-037, DN-28, the Spore-Build-and-Publish-
  Contract) without redeciding any of it. Pins the precondition to ADR-038 §2.8's terminal
  "1.0.0 ≡ fully rewritten + 100% operational" semver definition, stated as the maintainer's concrete
  two-stage per-unit bar (interpreter 100% checkout, then AOT-compiled). Works the managerial
  re-export pattern on the stdlib exemplar (grounded against the live `lib/std/mycelium-proj.toml`)
  and finds, checked against source, that **phylum re-export does not exist today** — `phylum` is
  reserved-not-active and `[surface].exports` is own-nodules-only in v0 — flagging it as a likely
  follow-on RFC. Records mapping criteria + method (not the mapping) and an ordered, per-group
  workflow. Refines, does not supersede, DN-27 (append-only; DN-27 unedited). Ids proposed
  (**E40-1**/**M-1020**), not minted (mitigation #1). Decides nothing normatively; enacts nothing.
