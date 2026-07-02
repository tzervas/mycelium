# Mycelium — Current State

**Start here.** This is a dense, link-heavy pointer index — not a narrative — so an agent or
maintainer can see "what's true now" in one short read instead of digging through the corpus.
**Tag:** `Empirical/Declared` (a maintained cross-check of `CHANGELOG.md`/`issues.yaml`/`Doc-Index.md`,
not itself a normative source). **Refreshed at integration-tier close-out** (see CLAUDE.md
§Concurrent-PR development); if it looks stale, `CHANGELOG.md` (top) wins.

## Project state, in one paragraph

Mycelium is in the **design + Rust-first implementation** phase. The **kernel/core** 1.0.0 sub-gate
(ADR-022 track T1) is **gate-met / tag-ready** — every criterion is closed, but the tag itself
(**M-703/M-655**) is **maintainer-reserved** and has not been cut. **Full-language 1.0.0** (ADR-022,
the broader criteria across all tracks) is **in progress**: T2–T7 are done, T8 (docs/stability) is
in-progress with its only blocker being the maintainer's own release act (**M-738**), and T9
(self-hosting) is explicitly **not tag-gating**.

## Gate state — ADR-022 tracks (verify against `issues.yaml` epic labels)

| Track | Epic | Status |
|---|---|---|
| T1 — core/kernel sub-gate | E10-1 | `in-progress` (label) — gate criteria met, **tag pending** (M-703, maintainer-reserved) |
| T2 — surface language | E11-1 | `done` |
| T3 — runtime | E12-1 | `done` |
| T4 — stdlib | E13-1 | `done` (on the ADR-035-narrowed bar) |
| T5 — FFI | E14-1 | `done` |
| T6 — AOT | E15-1 / E25-1 | `done` |
| T7 — toolchain | E16-1 | `done` |
| T8 — docs/stability | E17-1 | `in-progress` — 3/4 children done (M-735/M-736/M-737); **M-738 (release act) blocked only on the maintainer's tag-cut**, not engineering |
| T9 — self-hosting | E18-1 | `needs-design` — explicitly **not** tag-gating (ADR-036) |

**Kernel freeze (DN-56 §5): 1/5 conditions met.** Condition #1 (census/never-silent floor) is
satisfied (W5, 2026-06-27); condition #3 (primitive set closed) is contingent on **ADR-033 FLAG-1**;
conditions #2 (reject-ledger), #4 (lowering surface closed), #5 (KC-3 completeness review) remain
open. See `docs/notes/DN-56-Kernel-Completeness-And-Freeze-Criterion.md` §5/§7.

## Corpus status digest

Rough counts (verified against the tree, 2026-07-01): **27 ADRs**, **39 RFCs**, **72 design notes**
(`docs/adr/`, `docs/rfcs/`, `docs/notes/`, each minus its `README.md`). **One formal supersession:**
**ADR-021 → ADR-022** (2026-06-23) — ADR-021 is archived verbatim at
`docs/archive/adr/ADR-021-1.0.0-Release-Readiness-Gate.md`; its kernel Gate A/B criteria carry
forward as ADR-022 track T1. **`docs/Doc-Index.md` is the authoritative status resolver** — this
digest is a snapshot, that table is live.

## Implementation state

- **52 workspace crates** (`mycelium-*`, plus the `xtask` dev-tool — see root `Cargo.toml`
  `[workspace] members`).
- **L1 frontend complete:** generics/traits **RUN** (monomorphization + trait-dictionary elaboration,
  M-673, done); effects + runtime budgets (M-660/M-677); full HOF incl. capturing closures via
  Reynolds defunctionalization (M-704, done, KC-3 — no new L0 node); width-generics (DN-42,
  const-generic `Width{Lit,Var}`, M-753 done).
- The **interpreter is the trusted base**; **MLIR→LLVM AOT** has full-language coverage + parallel
  dispatch (E25-1, done).
- **25/26 stdlib specs Accepted/ratified** (`docs/spec/stdlib/*.md`; only `self-hosting-readiness.md`
  stays `Draft (needs-design)` — it is a gate doc, not a crate spec).
- **Runtime R1 vocabulary active** (`hypha`/`colony`/`fuse`/`reclaim`/`tier` — ADR-020 Enacted, v0
  R1 surface).
- **`fast`/`certified` cert modes** (ADR-032/RFC-0034).
- **Remote GHCR/OCI spore registry** (ADR-037, Enacted same-day 2026-07-01; live-dogfooded against
  GHCR).
- **`mycelium-transpile` PoC** (M-873, epic E18-1, `status:done`; DN-34 §8) — a `syn`-based
  Rust→Mycelium spike with a never-silent gap report; measured ≈12.4% grand-union surface coverage
  across 6 core-lib crates (`Empirical`, DN-34 §8.5) — feeds the surface-feature backlog, not a
  self-hosting mechanism itself.

## Active / next work

- **The Rust-reference-completion + acyclic-deps plan** — `docs/planning/rust-reference-completion-and-acyclic-deps.md`
  (kickoff UID `rcp`) is **PROPOSED**, under maintainer review in **open PR #913** (branch
  `worktree-agent-a89ab8d467a2b2b46`) — **not yet merged to `dev`**, so the plan doc and the `rcp.md`
  kickoff do not exist on this branch yet. It reframes the remaining work as a bounded closeout
  (Workstream A: make the acyclic-deps invariant structural; B–G: language/value-model/runtime/
  toolchain/security/kernel-freeze closeout) that gates **E18-1 self-hosting**.
- **Stdlib surface-sufficiency finding** (carried in the same open PR #913, destined for
  `docs/spec/stdlib/self-hosting-readiness.md` §0): the current surface is verified sufficient for
  **~19/26** stdlib crates; the **~5 real blockers are below-grammar** (float ops, binary mul/div +
  signed, Dense/VSA-to-L1 prims, R2 vocab, `Substrate` execution) — **not yet reflected** in the
  ratified spec on `dev` as of this writing (FLAG: verify PR #913's merge state before citing its
  file paths as live).

## Open maintainer decisions

See `docs/planning/Blocked-Decisions-Ratification-Map.md` for the full grouped ratification map
(14 groups + 14 ungrouped singletons, by 1.0-critical-path priority). Standing items surfaced across
recent landings:

- **Tuple-type** (prerequisite for multi-arg arrows / full partial application — RFC-0024 §4A.8; map
  group G2).
- **ADR-033 `FieldSpec::Fn` FLAG-1** (dynamic-dispatch soundness; shared prerequisite for kernel-freeze
  condition #3 — map group G4/G5).
- **RFC-0037 + DN-54** — RFC-0037 itself is **Enacted** (2026-06-27, the grammar epic); the open
  piece is **DN-54's derive-site consumption model**, underdetermined and needs maintainer
  ratification (DN-54 stays `Accepted`, not `Enacted`; see E11-1's `landed_basis`).
- **RFC-0038 scope** (Inject-Mode Security Axis) — `Accepted` (2026-06-29, design ratified) but
  **not yet built**; every mechanism claim stays `Declared` until the Implementation DoD (§13).
- **RFC-0035** (Security Scanning Toolkit) — `Proposed` (2026-06-24), awaiting ratification.
- **RFC-0027/DN-32 memory model** — the reclamation/ownership follow-on questions (OQ-1..OQ-6) not
  already covered by the runtime-vocab track (map group G3).

## Where truth lives

| What | Where |
|---|---|
| Grammar | `docs/spec/grammar/` |
| Decisions / status (ADR/RFC/DN) | `docs/Doc-Index.md` → `docs/adr/` \| `docs/rfcs/` \| `docs/notes/` |
| Tasks / epics | `tools/github/issues.yaml` |
| Recent changes | `CHANGELOG.md` (top = most recent) |
| History / superseded | `docs/archive/` (+ its `README.md` ledger) |
| Distilled component memory | `.claude/memory/` |
| Operating rules | `CLAUDE.md` |
