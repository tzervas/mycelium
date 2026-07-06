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

**Kernel FROZEN (declared 2026-07-02) — DN-56 → Enacted.** The freeze was declared on the **DN-76
green scorecard** (4/4 conditions plus the KC-3 completeness review; M-969, PR #1051); post-freeze
kernel diffs are **DN-39-only**. **RFC-0041 → Enacted (2026-07-05):** recursion-depth safety landed
via the promoted W0–W7 wave (PR #1155) — work-stack budgets everywhere, the `myc run` SIGABRT
closed; **DN-84 → Resolved**. See
`docs/notes/DN-56-Kernel-Completeness-And-Freeze-Criterion.md` and DN-76. *(Refreshed 2026-07-05 —
this paragraph previously said "1/5 conditions met", a pre-freeze snapshot superseded by the
2026-07-02 declaration.)*

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

- **The function-first umbrella roadmap** — `docs/planning/road-to-1.0.0-and-mycelium-rewrite.md`
  (**revised 2026-07-01** from the Rust-reference-completion + acyclic-deps plan, kickoff UID `rcp`;
  a pointer stub remains at the old path, which open **PR #913** carried in its prior form) is
  **Advisory/Proposed**, governed by **ADR-038** (*Pragmatic Dogfooding: the Function-First Release
  Strategy* — **Accepted**, 2026-07-01, maintainer-ratified; **FLAG-V1/V2 resolved** — public release
  sub-`1.0.0`/`0.x` at usability, `1.0.0` = fully-rewritten-where-appropriate): Phase I → `lang 1.0.0` + public
  release gated on **functional usability** (H0 acyclic-deps enforcement → H1 below-grammar
  enablers → H2 closeout lanes, H2a grammar-stability gate before mass porting); Phase II →
  post-public **progressive** Mycelium rewrite (compiler self-hosting deferred/conditional).
  ADR-038 refines ADR-036 §2.4 (release gate: usability, not Rust-replacement) and records that
  ADR-036 superseded RFC-0031 §5 D1's compiler-forever-Rust permanence (append-only notes on both).
- **Stdlib surface-sufficiency finding** — `docs/spec/stdlib/self-hosting-readiness.md` **§0 is
  live on this branch** (the 2026-07-01 currency update): the current surface is verified
  sufficient for **~19/26** stdlib crates; the **~5 real blockers are below-grammar** (float ops,
  binary mul/div + signed, Dense/VSA-to-L1 prims, R2 vocab, `Substrate` execution) — these are
  exactly the roadmap's **H1** enabler set.
- **Phase-I kickoffs authored (2026-07-01, planning tier per ADR-038 §2.7)** —
  `.claude/kickoffs/{acy,enb,grm,opp,frz}.md`: `acy` (H0 acyclic-deps + hygiene, **lands first**) →
  `enb` (H1 enabler closure, the usability critical path) · `grm` (H2a grammar-stability gate,
  ratification-gated) · `opp` (opportunistic `.myc` ports, non-gating, parallel) · `frz` (H2
  Rust-reference closeout — kernel freeze (DN-56) + inject-mode (RFC-0038) + R2 remainder (M-828) +
  l1-semantics tail; its **kernel-freeze declaration is the last Phase-I act**, gated on `enb`+`grm`,
  with the heavy runtime items M-869/M-868/M-831 marked Phase-II/non-gating). Every task
  carries a user story + DoD; M-ids **M-877…M-935 + `frz`'s M-958…M-969** (plus the RFC-0033-named
  M-766/M-767) are **proposed only** — minted at each kickoff after slot re-verification (mitigation #1).
- **Phase-II kickoffs authored (2026-07-01, same planning tier)** — `.claude/kickoffs/{flp,rwr}.md`:
  `flp` (**the public flip, two staged** — **Stage 1:** flip the **monorepo** public at a `0.x` in
  one gated act, strictly last in Phase I; **Stage 2 (later, post-public):** author the owed DN-27 §4
  binding decomposition ADR (ADR-039 proposed) and **push it to the remote as the maintainer's
  decomposition decision point**, then decompose into per-phylum-group repos, lock-then-archive the
  monorepo last — all maintainer-gated on the usability ratification + the `0.x` + FLAG-V1, resolved
  2026-07-01: public release is sub-`1.0.0`, no label collision) and
  `rwr` (**the post-public progressive rewrite** — mass porting gated on `grm`, transpiler ladder,
  toolchain ports, the V-wave remainder with the single M-780 rehash at its value-persistence
  tripwire, `1.0.0` terminal dossier with FLAG-V2, resolved 2026-07-01: `1.0.0` = fully dogfooded/
  self-hosted/rewritten-where-appropriate; compiler self-hosting stays an aspiration, not
  a lane). M-ids **M-936…M-957 proposed only**; `rwr` is deliberately higher-altitude (per-wave
  minting — ADR-038 §2.3).

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
