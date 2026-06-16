# Changelog

All notable changes to this project are recorded here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Dates are ISO-8601.

This project is in the **design phase**; "changes" here are to the documentation
corpus, not released software. Versioning will begin when the kernel does.

## [Unreleased]

### Added (2026-06-16: narrative & automated-authoring pipeline — design, M-363)
- **`docs/spec/Narrative-Authoring-Pipeline.md`** (**Proposed**) — the M-363 pipeline design (design-first;
  **ratify before building**): a **one content-addressed doc IR → many renderers** architecture
  (HTML/PDF/EPUB + machine JSON, so all formats share identity — ADR-003/G11, no drift); four projection
  generators (apiref/manual/book/blog) with their corpus sources; one reviewed template (the human gate
  for the fully-automated outputs); and the **§4.1 quality bar as a checkable 8-point lint** — single
  template, navigability, progressive disclosure (RFC-0013 levels), **checked examples** (a stale example
  fails the build — never-silent for docs, G2), no dead xrefs, dual-projection parity, **no hallucinated
  prose / undocumented-is-flagged**, legibility/accessibility. Placed in the M-361 toolchain (KC-3). The
  build stack + format/versioning choices are **flagged for ratification (§8); no pipeline code lands until
  ratified.**
- **`research/07-narrative-authoring-pipeline-RECORD.md`** — prior art (rustdoc/docs.rs, mdBook, Sphinx/MyST,
  Antora, literate programming, Pandoc/Typst, spec-generated manuals) traced as **T7.1–T7.7**, grounding the
  design (the no-drift, checked-examples, one-IR-many-renderers decisions).

### Added (2026-06-16: RFC-0015 automatic baseline diagnostics & recovery, M-362)
- **RFC-0015 ratified `Draft → Accepted`** and enacted. Prior art (DynEL, Rust `tracing`/`log`, Erlang/OTP,
  Python `logging`, structured-logging) traced into **`research/06-automatic-baseline-diagnostics-RECORD.md`**
  (findings **T6.1–T6.5**, discharging the §7 grounding obligation); the four §8 questions **resolved**.
- **`crates/mycelium-lsp/src/baseline.rs`** — the automation layer *over* RFC-0013 (presentation) +
  RFC-0014 (recovery), honest by construction (the §4.1 boundary A1–A4):
  - **`derive_baseline` / `derive_baseline_for`** — auto-derive a zero-config baseline `DiagnosticPolicy`
    from the error-class registry via a **total, inspectable closed `class → (level, route)` table**
    (`baseline_for_class`), optionally scoped per-definition by its **declared effect** classes. The result
    is presentation-only — structurally incapable of changing control flow (A1/I1) — content-addressed,
    and tagged `baseline`.
  - **`explain_baseline`** — the `EXPLAIN`: every class with its derived level/route + **rationale** (A3;
    "what baseline applied here, and why?").
  - **`recovery_profile`** + **`RecoveryProfile`** (`strict` / `resilient`) — the **closed, opt-in,
    bounded** recovery set (A2): `strict` propagates everything; `resilient` applies bounded `retry(≤3)`
    (`RESILIENT_MAX_ATTEMPTS`) to the **explicitly-supplied** classes only (RFC-0014 I4/I5). Recovery is
    **never** auto-applied — it is produced only on explicit request.
- **Honesty boundary, as tests:** A1 (a baseline can never suppress an error — `present` returns it
  unchanged), A2 (recovery bounded + opt-in), A3 (content-addressed + EXPLAIN-able), A4 (derivation is a
  total, deterministic function of the registry — every class covered). No new error mechanism; no kernel
  change (KC-3). `scripts/checks/all.sh` green.

### Added (2026-06-16: structured nodule header + project manifest, M-359)
- **`crates/mycelium-proj`** — the project-metadata layer (KC-3, above the kernel) enacting the
  *Nodule-Header-and-Project-Manifest* spec (**Accepted** 2026-06-16; the three §7 format choices ratified
  by the maintainer: header sigil `// @key: value`; the v0 key set extended with `repository`/`keywords`/
  `deprecated`; `@updated` author-maintained):
  - **`header`** — the structured nodule header parser: the `// @key: value` lines (closed 9-key v0 set)
    over the `// nodule:` marker (reuses M-358's `parse_nodule_header`). An **unknown** key, a
    **duplicate** key, or a **malformed** value (non-SPDX `@license`, non-ISO `@since`/`@updated`,
    ill-formed `@version`, non-URL `@repository`) is an **explicit** error, never silently ignored or
    guessed (G2 / VR-5 — checked, never fabricated).
  - **`manifest`** — `mycelium-proj.toml`, read by a **minimal, no-new-dependency TOML-subset** reader
    (the workspace keeps its deps few/vetted; adding a full TOML crate would be an ADR). It is honestly a
    subset — strings/arrays/inline-tables/booleans, single-line values — and an out-of-subset construct is
    an explicit error (G2). The closed `[project]` table is typed + validated; optional tables are accepted
    but not yet interpreted (M-361).
  - **`resolve`** — top-down inheritance (`in-file > manifest`) with **per-field provenance** and an
    **`EXPLAIN`**, so a field's effective value *and source* are never ambient (G2). A local value
    overrides the manifest (an allowed override, not a conflict; spec §4).
- **`mycelium-lsp::lint_structured_header`** (M-141) surfaces a malformed header as a `Diagnostic`.
- **Schemas** `docs/spec/schemas/{nodule-header,mycelium-proj}.schema.json` + valid/invalid examples
  (the SPDX-membership and calendar-date-range checks live in code, recorded in each schema's
  `x-mycelium.$comment` per the schemas-README rule). End-to-end conformance fixtures in
  `crates/mycelium-proj/tests/`.
- **Honesty/identity:** metadata is **not** identity — nothing here perturbs a content hash (ADR-003).
  No kernel change (KC-3). `scripts/checks/all.sh` green (incl. the JSON-schema gate).

### Changed (2026-06-16: DN-06 lexicon migration — static keyword `colony` → `nodule`, M-358)
- **The L1 surface keyword `colony` is now `nodule`** (DN-06, Resolved 2026-06-16) — a pure, mechanical
  rename across the lexer/token/parser/AST/checker/elaborator (`crates/mycelium-l1`), the LSP toolchain
  surface (`crates/mycelium-lsp`), the normative grammar oracle (`docs/spec/grammar/mycelium.ebnf` +
  README), and the **full accept/reject conformance corpus** (the `01-minimal-*`/`01-no-*-header` fixtures
  renamed accordingly). **No semantic change**: content-addressed identity is computed over elaborated L0,
  never the surface keyword or a Rust type name (ADR-003), so every definition's content hash is unchanged.
- **`phylum` and `colony` are now reserved-not-active keywords.** `phylum` (the library-scale grouping
  above nodules) and `colony` (reassigned to the RFC-0008 §4.7 **dynamic** runtime grouping of `hypha`)
  lex as keywords — so they can never be silent identifiers — but no L1 construct consumes them yet, so
  neither opens a program (new `conformance/reject/10-reserved-not-active.myc`; G2).
- **The `// nodule:` header marker (DN-06 §6) is wired in.** New `mycelium_l1::parse_nodule_header`
  recognises the first-non-blank-line marker (`// nodule: <dotted.name>` or bare `// nodule`); a near-miss
  *named* marker (empty/ill-formed name) is an **explicit** error, never silently dropped (G2). The M-141
  linter surfaces a malformed marker (`lint_nodule_header`) and the M-142 surface formatter preserves a
  valid one across a canonical re-print. The structured `// @key:` header + `mycelium-proj.toml` manifest
  layer on top of this (M-359).
- **Honesty/grounding:** DN-02 §2's `colony = module` line stays superseded by DN-06 (append-only); the
  Glossary, Lexicon-Reference, grammar README, and DN-06 changelog are updated to record execution.
  `scripts/checks/all.sh` green (incl. the conformance gate).

### Added (2026-06-16: typed SPSC channels — the RT2 communicating fragment, M-357 follow-on)
- **`crates/mycelium-mlir/src/channel.rs`** — the Kahn-deterministic *communicating* half of the RFC-0008
  RT2 fragment (§4.3), extending the landed fork/join runtime. **Typed single-producer/single-consumer
  channels**: `Network::channel` returns an affine `Sender`/`Receiver` pair (neither `Clone` — SPSC by
  construction, RT1) over a buffer of **explicit, finite** capacity (`NonZeroUsize` — no unbounded silent
  buffer, RT7's spirit on queues). **Demand-signalled backpressure**: `try_send` on a full buffer returns
  `Full(v)` handing the value back (never dropped); the producer yields and is re-polled as the consumer
  drains. **Explicit close**: dropping the `Sender` lets the `Receiver` drain then see `Closed`
  (end-of-stream, never a hang); a send to a hung-up receiver is `Disconnected(v)` (G2, never a silent
  drop). A new **`Scope::run_dataflow(order, progress)`** (in `runtime.rs`) schedules communicating tasks
  and surfaces a stalled network as an explicit **`Deadlock { parked }`** — never a silent hang (the
  cooperative scheduler cannot block). Determinism is verified by a **Kahn-determinism differential**: the
  same network under two distinct fair schedules (`SweepOrder::Ascending`/`Descending`) yields identical
  outcomes + transcripts (T4.1) — tagged **`Empirical`** (the differential is the evidence) with Kahn T4.1
  cited, **not** `Proven` (no mechanized proof in-repo; VR-5). Deferred (honest boundary): multi-source
  `select`/`merge` (RT3), session/protocol typing beyond the §4.3 hook, zero-capacity rendezvous,
  `xloc`/`mesh` (R2). No kernel change (KC-3); no `unsafe`. RFC-0008 §4.6 staging note + Meta-changelog
  updated (append-only). `just check` green.

### Fixed (2026-06-16: PM manifest drift — labels.json out of sync with issues.yaml)
- **`tools/github/labels.json`** was missing three labels that `issues.yaml` already uses —
  **`type:design`** (12 issues), **`priority:P3`** (11 issues), and **`area:language`** (1). Because
  `gh issue create --label <name>` errors on a label the bootstrap never created, this silently stalled
  issue creation: the five staged Phase-7/8 issues (**M-358/359/361/362/363**) were not created on the
  prior run. Added the three labels (matching the existing color/description style) so a sync run creates
  them first, then the issues that reference them.

### Added (2026-06-16: one-command PM gap-closer + manifest preflight)
- **`tools/github/gh-sync-all.sh`** — a single **idempotent** command that reconciles the repo with the
  manifests in one pass: a preflight, then `gh-bootstrap-local.sh` (labels + milestones), then
  `gh-issues-sync.py` (create absent issues + assign milestones + append `idmap.tsv`). Safe to rerun any
  time `issues.yaml`/`labels.json`/`milestones.json` gains entries; nothing is duplicated. Supports
  `--dry-run` (preview issue creation, no repo writes).
- **`tools/github/manifest-check.py`** — the preflight: every label/milestone `issues.yaml` references
  must be **defined** in `labels.json`/`milestones.json`, else an explicit fail-fast error (the
  never-silent rule, G2 — a missing label can no longer silently leave issues uncreated). Reverse drift
  (a defined-but-unused manifest entry) is an advisory note only.
- Docs updated to make `gh-sync-all.sh` the canonical re-sync entrypoint: `MILESTONES.md`,
  `mcp-bootstrap.md`, `termux-bootstrap.md`. The two component scripts stay single-purpose.

### Added (2026-06-16: mobile/Termux GitHub bootstrap — phone-autonomous PM)
- **`tools/github/termux-setup.sh`** + **`tools/github/gh-issues-sync.py`** + **`termux-bootstrap.md`**.
  A single, ordered, **idempotent** path to run the *whole* GitHub project-management bootstrap from an
  Android phone (Termux) with nothing pre-configured: installs packages from the package manager (no
  `curl | bash`), sets the git identity, generates a passphrase-protected **GPG signing key** and uploads
  only the **public** key, authenticates `gh` (browser/device OAuth or a supplied token, held by `gh` —
  never committed), then chains `gh-bootstrap-local.sh` (labels + milestones) into the new
  `gh-issues-sync.py`. The Python helper is the **gh-driven local analogue of `mcp-bootstrap.md` Steps
  1–2** — it closes the one gap that previously needed a model+MCP session (issue *creation*): snapshot
  issues by title, create only the absent ones with labels, assign milestones by title, and **append**
  (never rewrite) new `task_id → number → db_id` rows to `idmap.tsv`. Honesty-aligned: never-silent (every
  step announced; conflicts/missing milestones are explicit), no black boxes, no secrets in the repo
  (private GPG key stays on-device; token in `gh` config; credential helper, not token-in-URL). Scope
  boundary matches `gh-bootstrap-local.sh`: dependency/sub-issue linking (Step 4) still needs an
  MCP/GraphQL pass. `shellcheck`/`ruff` clean.

### Added (2026-06-16: narrative capture + automated-authoring intent, initial capture)
- **`docs/notes/Narrative-Capture-and-Authoring.md` (Living)** + the seeded **`docs/devlog/`** append-only
  narrative layer. Captures the maintainer's intent to record enough development narrative — decisions,
  **struggles, problems solved, the how and why** — to enable **partially-to-fully automated** authoring
  of project **blog** posts, a **language book**, and a **reference manual**, distributed **free** in
  digital formats. Notes that the honesty rule already makes the corpus a grounded, cited, append-only
  narrative (~80% of the raw material); the one gap (the struggle / problem-solving *how*) is filled by a
  lightweight `docs/devlog/` (first entry: `2026-06-16-rfc0008-integration-wave.md`, a worked example).
  All three outputs are **synthesis from the cited corpus** under the same discipline as the language —
  grounded/cited (no hallucination), projection-not-parallel-truth (no drift — ADR-003),
  human-in-the-loop, append-only provenance. Full pipeline design + tooling is a fresh session, tracked
  **M-363** (Phase 8). Registered in `Doc-Index.md`.
- **Added (future-planning):** a fourth output — **fully-automated documentation + API reference** (the
  most automatable: pure projection from code + schemas + the M-359 nodule-header metadata; rustdoc-first,
  Mycelium-lang doc-comments later; shipped free + served live/LSP-hover) — and a **format quality bar**
  (note §4.1): "clean · presentable · legible · intelligible · digestible" made a **checkable** contract
  (one consistent template; index→detail navigation; progressive-disclosure graded depth reusing
  RFC-0013's levels; checked inline examples; dual human/machine projection — G11; legibility/accessibility
  by construction; **undocumented is flagged, never invented** — the doc analogue of never-silent G2).

### Added (RFC-0015 — 2026-06-16: Automatic Baseline Diagnostics & Recovery, Draft)
- **RFC-0015 (Draft, Proposed)** captures the DynEL **automated-baseline** design point the maintainer
  added to the roadmap: an **automation layer over RFC-0013/0014** that auto-derives a zero-config
  **baseline** diagnostic/logging policy from the language's structured mapping (registry + routes +
  declared effects), **auto-applies** it (wrapping for logging/QoL), and offers a ladder of *light*
  overrides → *fully manual*. The load-bearing **honesty boundary** is fixed up front: automatic =
  **additive presentation/logging only** (safe because RFC-0013 never changes control flow — I1);
  **automatic recovery is opt-in, declared, bounded** (no implicit control-flow change — RFC-0014
  I3/I4/I5); the baseline is a **reified, `EXPLAIN`-able** policy (no black box — SC-3); the derivation is
  a **total, inspectable** function of the mapping, not learned (VR-5/RFC-0005). Tooling-layer; no kernel
  change (KC-3). Forward-pointed from RFC-0013 §9 + RFC-0014 §9; registered in `Doc-Index.md`; tracked
  **M-362**. **No code** — design point only.

### Changed (DN-06 — 2026-06-16: static-organization & dynamic-grouping lexicon — `phylum` / `nodule` / `colony`)
- **DN-06 ratified** (maintainer-directed), introducing on-brand terms for static organization and
  deconflicting a real collision: **`phylum`** (content-addressed **library-scale** unit) and
  **`nodule`** (the **basic** static unit, replacing the generic "module") for static organization, and
  **`colony`** reassigned to the **dynamic** runtime grouping of active `hypha` (RFC-0008 §4.7). The
  reassignment **supersedes DN-02 §2's `colony` = module** line (append-only — DN-02's changelog records
  it; `phylum`/`nodule` had no prior use, so only `colony` collided). Justified by the DN-02 three-test
  gate: `colony` on a *living, supervised grouping of tasks* is a higher-fidelity T-map than on a static
  file, and `nodule` beats the generic "module" for the static unit.
- **Supplement (DN-06 §6 resolved):** a `nodule` is declared by a **header comment**
  (`// nodule: <name>`, or bare `// nodule`) on the first non-blank line — **not** in the filename/path
  (paths stay conventional; no `nodule` bloat). RFCs/docs use `nodule` for "module" going forward. A
  **dedicated `docs/Glossary.md`** is created — a summarized **Index** over a detailed **Glossary**
  (the fungal lexicon + honesty/architecture concepts), each entry citing its normative source, maintained
  separately from the RFCs (registered in `Doc-Index.md`). The header-comment convention folds into M-358.
- **Proposed — structured nodule header + `mycelium-proj.toml` manifest (`docs/spec/Nodule-Header-and-Project-Manifest.md`).**
  At the maintainer's preference for a *structured* header carrying useful metadata (license, authors,
  first/last dates, version) on a nodule/phylum **root**, with **subnodules inheriting** top-down: a
  closed-key in-file header (`// @key: value`), a `mycelium-proj.toml` manifest (the pyproject/Cargo analogue,
  scoped for Mycelium), and explicit `EXPLAIN`-able inheritance (in-file → nodule-root → `mycelium-proj.toml`).
  Honesty-aligned: **metadata is not identity** (the content hash stays canonical — ADR-003), no ambient
  metadata (unknown keys/conflicts are explicit errors — G2), declared-only license/version (VR-5),
  tooling-layer (KC-3). **Proposed** — the format choices (§7) are flagged for sign-off; no code lands
  until ratified. Records the long-term **full-fat toolchain** as the new anticipated **Phase 8** (epic
  **M-361**); the schema's enactment is **M-359**.
- **Adopted going forward:** the RFC-0008 §4.7 structured scope is realized as `mycelium-mlir::runtime`'s
  **`Colony`** (alias of the structured `Scope`). The **surface keyword migration** `colony` → `nodule`
  (the L1 lexer/parser/AST/checker — ~226 refs — plus the grammar EBNF + LR(1) oracle + the 23-file
  conformance corpus) is a pure rename + two reserved additions (`phylum`/`colony`), tracked as **M-358**
  and staged (the grammar contract moves in one auditable change). Until executed, `colony` is the
  deprecated spelling of `nodule`. RFC-0006 + RFC-0008 carry append-only forward-references; `phylum`
  and `colony` are reserved-not-active until their constructs land.

### Changed (RFC-0008 — 2026-06-16: Runtime & Concurrency Execution Model ratified `Draft → Accepted`)
- **RFC-0008 ratified `Draft → Accepted`** (maintainer): the seven runtime invariants **RT1–RT7** and
  the §4 model are now **normative** (the Runtime-tier grounding ADR-012 §7.3 required). Ratification
  opens the runtime track in staged slices: the **budget-unification slice** (RFC-0014 §4.8 — M-353,
  below) and the **route → observability-sink** binding (RFC-0013 §8 — M-354) needed no RT1–RT7
  commitment and proceed first; the **concurrency/supervision** track (RFC-0014 single-task boundary
  lifted — per-task budgets, cancellation, cross-task propagation, `reclaim` bounded cascades; RT4/RT7 —
  M-355/M-356) is the §4.7 revision, presented frozen-spec before folding. The §4.5 runtime vocabulary
  stays **reserved, not active syntax** until the implementation RFCs land.

### Added (RFC-0008 R1 — 2026-06-16: M-357 v0 / deterministic fork/join executor + RT2 differential)
- **M-357 (v0 slice) — the RT2 deterministic fork/join runtime over the §4.7 primitives.** The
  maintainer-chosen minimal scope (fork/join + the differential; typed channels deferred to the next
  slice): `crates/mycelium-mlir/src/runtime.rs` — a structured-concurrency `Scope` (RT7: every child is
  **joined**, none orphaned) over cooperative `Task`s, each carrying its **own** `Budgets` ledger and the
  shared `CancelToken` (M-356 C1/C2). Two strategies — `run_sequential` (the reference) and a
  deterministic `run_interleaved` round-robin — that the RT2 guarantee makes observationally equal over
  **pure** tasks (RT1). The scheduler lives **outside** the kernel (RT2; the trusted evaluator stays
  sequential — KC-3).
- **Verified** (module tests): the **RT2 sequentialization differential** — `run_interleaved` ≡
  `run_sequential` over a counter corpus (with an interleave trace proving the schedules genuinely
  differ) **and over the real env-machine** (tasks running `run_core_with_effects` on `bit.not` L0
  programs; each scheduled outcome equals the standalone `run_core` evaluation — no new meaning,
  NFR-7/KC-3); **RT7** scope-cancellation (cancelling the scope → every pending child resolves to an
  explicit additive `Cancelled`, all joined, none leaked); and **C1** per-task budget isolation (one
  task overrunning its `alloc` budget never exhausts a sibling's). `just check` green. The next R1 slice
  is typed SPSC **channels** (the Kahn-deterministic communicating half). **M-357 (#122)**.

### Added (RFC-0008 §4.7 — 2026-06-16: M-356 / concurrency composition primitives, single-task boundary lifted)
- **M-356 — RFC-0014's single-task boundary lifted onto RFC-0008 (§4.7 added; §8 concurrency deferral
  resolved).** A **frozen-spec** (presented before folding): RFC-0008 **§4.7** specifies four
  compositions, each additive over the explicit error (I1) and declared + bounded (I3/I4) — **(C1)**
  per-task budgets (each task instances its own M-353 ledger; an overrun is an *in-that-task*
  `EvalError::EffectBudget`, never global); **(C2)** cooperative, **additive** cancellation observed at
  budget-check points (an explicit `Cancelled`, never preemptive; scope-tree propagation, RT7);
  **(C3)** cross-task failure propagation via an explicit `TaskOutcome` with **no silent/dropped
  variant** (I1 across the task boundary, RT4); **(C4)** `reclaim` **bounded-cascade** supervision
  bounded on **both** a total `cascade` effect budget (M-353) **and** a windowed max-restart-intensity
  over a **logical clock** (Erlang/OTP, Research Record 05 T5.3; wall-clock deferred to R8-Q3) —
  exceeding either an explicit escalation, never a storm.
- **Enacted** as **scheduler-independent** primitives in `mycelium_interp::supervise`
  (`CancelToken` / `TaskOutcome` / `RestartIntensity` / `Supervisor` / `Escalation`) — **no L0 node**,
  the trusted base stays sequential (RT2; KC-3) — verified there and composed with the recovery driver
  in `crates/mycelium-lsp/tests/recover.rs` (cancellation is explicit + additive; a task failure
  propagates explicitly; a supervised restart storm is bounded on both axes; a per-task budget overrun
  is an in-that-task refusal). The actual **task scheduler/executor and the RT2 sequentialization
  differential are explicitly *not* here** — they are RFC-0008 R1 (**M-357**), built on these
  primitives. `just check` green. Advances G2, VR-5, SC-3. RFC-0014 §8 concurrency deferral **resolved**.
  **M-356 (#121)**.

### Added (Phase 4 — 2026-06-16: M-354 / RFC-0013 §8 diagnostic routes ↔ RFC-0008 observability sinks)
- **M-354 — the diagnostic `route` set closed and bound to RFC-0008 sinks (RFC-0013 §8 resolved).** A
  **closed v0 route vocabulary** — `stream` / `audit` / `log` / `null` / `mesh` — in
  `crates/mycelium-lsp/src/diagnostics/sink.rs`, each bound to an `rfc0008.*` observability sink with an
  **honest delivery guarantee** on the lattice (RT5): `stream` (in-process synchronous), `audit`
  (durable), and `log` (best-effort) are `Declared`; **`null` honestly reports *not delivered*** (never
  a "fire and forget" claimed reliable); **`mesh` is probabilistic**, carrying a declared
  `ProbabilityBound` δ (upgraded to Empirical/Proven only with a checked convergence basis — VR-5/T4.2).
  Route resolution is **checked** against the closed set (the §4.5 X1 "looked up, never evaluated"
  discipline applied to routes — an out-of-set route is an explicit `UnknownRoute`, never a silent
  misroute) and lives **outside** `present` (`DiagnosticRecord::sink` is the dispatch point), so routing
  — or a failed resolution — **never gates propagation** (I1). A typed `Rule::route_to(Route)` setter is
  the checked path; the free-form `route(String)` remains the on-the-wire projection. Tooling layer only;
  **no kernel logging dependency** (KC-3).
- **Verified** by `crates/mycelium-lsp/tests/diagnostics.rs`: **never-silent across every closed route**
  (I1 re-run per route — the error still propagates, each route resolves to its sink), **honest sink
  guarantees** (no sink over-claims `Declared`; the null sink does not deliver; the mesh sink carries a
  well-formed δ — RT5/VR-5), and an **explicit unknown route** (an out-of-set route string surfaces
  `UnknownRoute` without gating propagation). `just check` green. Completes the RFC-0013 §8
  route-targets/observability deferral; advances NFR-2/SC-5b. **M-354 (#119)**.

### Added (Phase 4 — 2026-06-16: M-353 / RFC-0014 §4.8 effect-budget unification, enacted)
- **M-353 — effect budgets unified with the runtime's fuel/depth clocks (RFC-0014 §4.8 completed).** The
  recovery `Budgets` ledger — previously a tooling-only reified mechanism — is **lifted into
  `mycelium-interp`** (`mycelium_interp::budget`: `EffectKind`/`EffectBudget`/`EffectBudgetExhausted`/
  `Budgets`), the **shared budget-resolution surface** both the AOT env-machine (`mycelium-mlir`) and the
  recovery driver (`mycelium-lsp`) depend on — placed to avoid a crate cycle and to sit where the fuel
  clock already lives (**no kernel change** — KC-3, **no** new L0 node, **no** kernel hook). An effect
  overrun now routes through **`mycelium_interp::EvalError::EffectBudget`** — the effect sibling of
  `FuelExhausted` (time) / `DepthLimit` (space) on the **one runtime refusal channel** (the ratified §8
  disposition: *separate named budgets, one enforcement mechanism*): a budgeted effect overruns
  **gracefully at runtime exactly as a runaway recursion does**, never a hang/OOM (I4). The env-machine
  threads the same ledger (`run_core_with_effects`) and charges a declared **`alloc`** budget per
  control-stack frame — the **opt-in** sibling of the DN-05 depth ceiling (same per-frame-bytes basis);
  an absent budget (the default) leaves behaviour identical (I5). `recover::effect` re-exports the moved
  types (RFC-0014's enacted API is unchanged) and keeps the *checker* half (`check_effects`/
  `UndeclaredEffect` — I3) in the tooling layer.
- **Verified:** the **bounded-overrun-is-explicit test extended to the runtime path** (`mycelium-mlir`:
  `a_declared_alloc_effect_budget_overruns_gracefully_at_runtime` → `EvalError::EffectBudget`, and
  `an_absent_alloc_budget_leaves_runtime_behaviour_unchanged`), plus a **meaning-preserving three-way
  differential** where it touches L0 (`mycelium-l1`:
  `the_effect_ledger_is_meaning_preserving_on_the_recovery_match` — threading an ample ledger is
  observable-transparent on the recovery `Match`; NFR-7). `just check` green. Completes the RFC-0014 §4.8
  deferral; advances G2, VR-5, SC-3. **M-353 (#118)**.

### Added (Phase 4 — 2026-06-16: M-352 / RFC-0014 declarative recovery & bounded effects, accepted + enacted)
- **RFC-0014 ratified `Draft → Accepted`** (maintainer; all §8 dispositions normative) and **M-352
  enacted** as a **separable, tooling-layer** subsystem in `crates/mycelium-lsp/src/recover` (**no kernel
  change** — KC-3, zero new L0 nodes; no Python, ADR-007). Three pillars: **errors-as-propagating-values**
  (`Outcome` over a `StructuredError` whose class is registry-resolved — shares RFC-0013's registry, X1);
  **explicit declarative recovery** — the never-silent `handle` applies a reified
  `on <ErrorClass> => <action>` policy (RFC-0005 pattern; content-addressed `PolicyRef`; closed action set
  `fallback`/`retry`/`escalate`/`cleanup_then_propagate`) and yields a `Resolution` that is **always**
  *recovered* or *re-propagated* — there is no "dropped" variant (I1 enforced by the type); and
  **declared, bounded effects** (`EffectKind` set, per-kind `EffectBudget`, the `Budgets` ledger whose
  overrun is a graceful `EffectBudgetExhausted` — I4, and a compositional `check_effects` no-undeclared
  -effect check — I3). A substituted fallback is honestly `Declared`, never upgraded (I2/VR-5).
- **Verified** by `crates/mycelium-lsp/tests/recover.rs` (RFC-0014 §5): the central **never-silent
  recovery invariant** (every action leaves the error recovered or propagated, never dropped — I1), the
  **bounded-overrun-is-explicit** test (`EffectBudgetExhausted`, never a hang/OOM — I4), the **opt-in
  default-scope** test (an undeclared effect can't run — I5), the **no-undeclared-effect** test (I3), the
  **honest-guarantee** test (I2/VR-5), and the shared-registry / no-`eval` discipline (X1). The
  **L0-`Match`-over-error-sums lowering target** — "recovery adds no new kernel node" — is differentially
  verified in `mycelium-l1` (`recovery_match_over_a_result_sum_agrees_three_ways`: L1-eval ≡ L0-interp ≡
  AOT; NFR-7). **Out of v0 scope (honest boundary):** wiring the `Budgets` ledger into the AOT
  env-machine's runtime budget resolver is the RFC-0008 integration (§4.8). `just check` green. Advances
  SC-3, G2, VR-5, NFR-2/SC-5b. RFC-0014 status → **Accepted — Enacted**; **M-352 (#116)** closed.

### Added (Phase 4 — 2026-06-16: M-345 / RFC-0013 structured diagnostics, enacted)
- **M-345 — RFC-0013 structured diagnostics & reified error policy: enacted** in
  `crates/mycelium-lsp/src/diagnostics` (tooling layer; **no kernel change**, KC-3; no Python, ADR-007).
  Four parts: the **error-class registry** (names looked up, **never `eval`-ed** — §4.5 X1; v0 classes
  from the existing lint codes + `SwapError` family + `NotValidated`); the **content-addressed
  diagnostic record** with a BLAKE3 `content_id` and a **dual human + JSON projection** that round-trips
  (G11, §4.3), graded `minimal`/`medium`/`detailed` **levels** with an **allowlisted** detailed tier
  (§4.5 X2), and the never-silent **`present`** renderer that returns the explicit error **unchanged**
  alongside the presentation (§4.1 I1); the reified **`on <ErrorClass> => {message, tags, level,
  route}` policy** (RFC-0005 pattern; content-addressed `PolicyRef`; presentation/routing only — I4),
  with a `PolicyFile` projection that re-validates classes through the registry (file-as-projection,
  §4.7); and the **representation-crossing audit view** (§4.6; routed from RFC-0012 R12-Q2) — every
  `swap` + from/to repr + honesty bound **read off the certificate and never upgraded** (VR-5),
  location-independent (I5).
- **Verified** by `crates/mycelium-lsp/tests/diagnostics.rs` (RFC-0013 §5): the central **never-silent
  invariant** (a battery of policies — routed / message-override / minimal-level / unrelated — all leave
  the error propagating; I1/I2/I4), round-trip projection (I3), registry / no-`eval` (X1, incl.
  whole-file rejection on an unknown class), the detailed-tier allowlist (X2, a secret-bearing field
  never reaches the record or its rendering), and the audit view (I5/VR-5, incl. an underivable crossing
  reporting `unknown`, never `Exact`). `just check` green. Advances NFR-2 / SC-5b and the M-330 AI
  co-author loop. RFC-0013 status → **Accepted — Enacted**.

### Changed (Phase 4 — 2026-06-16: ratifications, RFC-0014 design decisions, M-343 totality completion)
- **RFC-0013 — Structured Diagnostics & Reified Error Policy: `Draft (Proposed) → Accepted`** (maintainer
  sign-off). No design content changed on acceptance; the §4 invariants I1–I5 and the §4.5 exclusions
  X1–X3 are now normative. Unblocks the **M-345** Rust tooling-layer build (`mycelium-lsp`/`xtask`; no
  kernel change). Verified by the central never-silent invariant test (I1/I2/I4) + round-trip / registry /
  allowlist / audit-view tests.
- **RFC-0014 — remaining §8 questions given proposed v0 dispositions** (maintainer sign-off pending; RFC
  stays Draft, no code yet): effect inference = *manual-declare + compositional-check* (caller must
  declare a superset of callee effects — `UndeclaredEffect` otherwise — but the checker never infers an
  undeclared effect); recovery-action set = the *closed* v0 set
  `fallback`/`retry`/`escalate`/`cleanup_then_propagate` (each never-silent + bounded; user actions a §9
  future inheriting I1/I3/I4); concurrency = *deferred to RFC-0008* with a single-task v0 boundary fixed
  now (per-evaluation budgets, no cross-task cascade — deferral is safe); handler composition = *lexical
  innermost-first* (unmatched re-propagates, never drops), handler effects declared + budgeted like any
  code, cascades bounded by `cascade(max_depth)`. With the §7 prior-art tracing done, RFC-0014 is **ready
  for a Draft→Accepted decision**.
- **RFC-0014 — three §8 design questions resolved** (maintainer; RFC stays Draft): effect mechanism =
  **declared annotations, coarse set** (capabilities/effect-rows additive futures only); **no
  kernel-visible hook** — effect-budget enforcement is entirely runtime/checker, **zero new L0 nodes**
  (KC-3); **separate named budgets over one enforcement mechanism** — each effect kind keeps its own
  `EXPLAIN`-able budget, all resolved/enforced by the existing DN-05 plumbing that already clocks `Fix`/
  `FixGroup` fuel and the M-347 depth ceiling (composed alongside, not collapsed). No code until Accepted.
- **RFC-0014 prior art traced into `research/`** — new **Research Record 05** (T5.1–T5.6) grounds
  Result/`?`, algebraic effects (Koka/Eff/OCaml 5), **Erlang/OTP bounded supervision** (verified:
  max-restart-intensity, defaults 1/5s), structured-concurrency cancellation, capabilities, and Mycelium's
  own fuel/depth/DN-05 budget idiom — discharging the §7/§8 grounding obligation (honest deltas + novelty
  flags recorded). RFC-0014 §7/§8 + status line updated to reflect the resolutions and the tracing.
- **M-343 — mutual-descent totality classification (R7-Q3 loose end closed).** The `FixGroup` elaboration +
  three-way differential had landed, but the structural totality checker still classified *every* mutual
  group `Partial`. Extends `crates/mycelium-l1::totality` from self-descent to **mutual structural descent**
  over a call-graph SCC: a group is `Total` iff a per-member designated argument position descends on every
  inter-member call (one well-founded measure; bounded position-assignment search). Sound — only adds
  justified `Total` verdicts; gates `matured`, never meaning (G2; runtime stays fuel-clocked). RFC-0007 §4.5
  revised (append-only); ping/pong now `Total`, a non-productive cycle stays `Partial`.

### Added (Phase 4 — RFC-0014: declarative error recovery & bounded effects, drafted)
- **RFC-0014 — Declarative Error Recovery & Bounded Effects (Draft (Proposed)).** Designs the isolated
  recovery subsystem RFC-0013 §8/§9 deferred (the DN04-Q1 recovery half) — a way for errors to **bubble**
  and **trigger functionality** (fallback, retry, cleanup, escalation), as a **separable** subsystem with
  a bounded blast radius. Three pillars: **errors-as-propagating-values** (the RFC-0001 substrate, G2);
  **explicit declarative recovery** (an explicit handling site that elaborates to L0 `Match` — **KC-3, no
  new kernel node** — plus a reified RFC-0005-pattern `on <ErrorClass> => <action>` recovery policy); and
  **declared, bounded effects** (effects named on signatures so there are no unknown side effects; every
  unbounded effect carries an explicit budget and overruns *gracefully* as `EffectBudgetExhausted` — the
  direct generalisation of the `Fix`/`FixGroup` fuel clock, the M-347 depth ceiling, and DN-05 budgets).
- **Records the maintainer's governing discipline:** effects and even cascades are allowed **when
  explicitly declared and implemented** so they stay *known and bounded* — the enemy is
  *unintended/unknown/unbounded* effects (memory explosion, runaway cascade, spooky action), not effects
  per se; default tightly scoped, broader opt-in by explicit declaration; recovery is **additive over**
  the explicit error (never silent — G2; never fabricates or upgrades a guarantee — VR-5). Isolation:
  budget enforcement lives with RFC-0004/0008/DN-05, **not** the kernel; clean **RFC-0013 split**
  (presentation vs. recovery; shared registry/pattern; RFC-0014 does not weaken RFC-0013's I1).
- Prior art (Result/`?`, algebraic effects, **Erlang/OTP bounded supervision**, structured-concurrency
  cancellation, capabilities, Mycelium's own budget idiom) recorded as **design inspiration not yet traced
  to `research/`** (a pre-ratification task). Many design choices (effect mechanism, budget vocabulary, any
  kernel hook) are **explicit open questions** — no code lands with the draft; ratification + a tracking
  milestone are the maintainer's. RFC index + RFC-0013 §8/§9 cross-refs updated. Advances SC-3, G2, VR-5,
  NFR-2/SC-5b.

### Added (Phase 4 — M-345: RFC-0013 structured diagnostics & reified error policy, drafted from DN-04)
- **RFC-0013 — Structured Diagnostics & Reified Error-Handling Policy (Draft (Proposed)).** Turns the
  DynEL-inspired DN-04 direction into a ratifiable, **tooling-layer** design with **no kernel change**
  (KC-3) and **no Python** (ADR-007 Rust-first; DynEL is reference-only). Imports three contracts —
  **graded context levels** (verbosity over EXPLAIN / `FeedbackSummary` / `NotValidatedReason`), **dual
  human + JSON projection** of one content-addressed diagnostic (G11), and a **reified per-definition
  error-handling policy** `on <ErrorClass> => {message, tags, level, route}` in the RFC-0005/ADR-006
  pattern — and **normatively excludes** three anti-patterns (config-string `eval` → registry lookup;
  wholesale env/locals dump → an allowlisted detailed tier; `logger.catch` swallowing → additive over a
  still-propagating error). Governing invariant: **a diagnostic is additive presentation over an
  explicit error, never a substitute** (G2 never-silent).
- **DN04-Q1 resolved → presentation/routing only for v0.** A policy shapes message/tags/level/route; the
  explicit error/`Option`/refusal **still propagates** unchanged. **Declarative recovery is deferred** to
  a separate future RFC, with the maintainer's constraints recorded (RFC-0013 §8/§9): an **isolated,
  separable** subsystem (SoC, bounded blast radius) with **explicit, declared, bounded** effect
  semantics (errors-as-values / reified effect handlers — errors propagate/bubble and can *trigger*
  functionality; effects and cascades are allowed *when explicitly declared/implemented* so they stay
  known and bounded — the enemy is *unintended/unknown/unbounded* effects, not effects per se), always
  **additive over** the explicit error. DN04-Q2 = free-form
  string tags (v0); DN04-Q3 = file is a projection of the canonical declaration; DN04-Q5 = standalone RFC
  now (stdlib graduation, M-346, a future option).
- **Carries the representation-crossing audit view** routed here from RFC-0012 R12-Q2 / M-351: a
  location-independent view enumerating every `swap` with its honesty bound (Exact/Proven/Empirical/
  Declared, never upgraded — VR-5) and selection policy. Advances NFR-2 / SC-5b (semantic feedback) +
  the AI co-author loop (M-330). DN-04 status updated (now feeds RFC-0013); RFC index updated. No code
  lands with the draft — ratification is the maintainer's append-only decision.

### Added (Phase 4 — M-343: mutual recursion in the L0 calculus; RFC-0001 r5, R7-Q3 resolved)
- **`FixGroup` — one new L0 node for mutual recursion** (RFC-0001 r5; the n-way generalisation of
  `Fix`). `FixGroup{defs, body}` binds a strongly-connected call group simultaneously (each definition
  and the continuation see all the group's names), so two functions can call each other. The
  elaborator (`mycelium-l1::elab`) now decomposes the reachable call graph into SCCs (Tarjan,
  callee-first) and lowers a self-recursive singleton to `Fix` and a group of ≥2 to a `FixGroup`;
  **mutual recursion is no longer an `ElabError::Residual`** — a structurally v0 program no longer
  residualises on recursion at all (only a dynamic `@ guarantee` index does). The node carries **no
  captured environment** and unfolds by substitution under the **same fuel clock** as `Fix` (a *focus*
  member-name unfold + a *continuation* unfold; the group binds all member names so substitution
  shadows them) — a non-productive group is an explicit budget exhaustion, never a hang.
- **Enacted across the trusted base and the AOT path, in lockstep:** `mycelium-core` (node +
  `is_aot_lowerable` + content-addressing + the canonical/core/ANF formatters + `Rhs::FixGroup`),
  `mycelium-interp` (the two-case unfold + capture-avoiding `subst`), `mycelium-mlir::aot` (the
  env-machine `FixGroup` suspension + unfold; the native-LLVM subset refuses it with `UnsupportedNode`
  like the rest of the data/recursion fragment, VR-5), and the dialect/LSP walkers.
- **Verified by the three-way M-210 differential** (L1-eval ≡ elaborate→L0-interp ≡ AOT) extended with
  mutually-recursive programs — ping/pong, even/odd over a Bool result, a constructive group that
  builds data on the way back, and a three-function cycle — plus a `FixGroup`-lowering witness. Resolves
  **R7-Q3** (the cycle *identity* was fixed in RFC-0001 r4; the matching *node* lands now). Full
  `cargo test` green. (NFR-7, VR-5, SC-3, LR-1; KC-3 — the kernel grows by exactly one deliberate,
  ratified node.)

### Decided (Phase 4 — M-351: RFC-0012 R12-Q1 & R12-Q2 resolved; no new ambient code)
- **R12-Q1 (per-use size) → no new sugar.** A paradigm-less **ascription** `e : {N}` already states an
  explicit size at the use site with the paradigm from the central `default` (now tested:
  `mycelium-l1/tests/ambient.rs::a_paradigm_less_ascription_states_the_per_use_size`), so a context-free
  bare decimal is sizable without a surrounding annotation and elaborates identically to longhand (I2).
  **Sizes stay explicit** (no ambient default width); a `u8`/`f64` literal suffix was **rejected**
  (imports signed/dtype affordances the kernel does not provide — v0 `Binary` is unsigned, no `iN`,
  `f64` is a Dense dtype not a width — a false-affordance footgun that also fails to generalize across
  the four paradigms). A paradigm-agnostic `:N` shorthand stays a possible future sugar iff terseness
  earns it (KISS/YAGNI).
- **R12-Q2 (paradigm-boundary swaps) → crossings stay at swap sites.** No default swap policy. **Swap
  sites** vs **`with paradigm` block edges** were weighed against the language's intention (fluid,
  paradigm-agnostic traversal): swap sites win — a `swap` is a free, first-class *anywhere* crossing and
  `with paradigm` stays pure tag-scoping (SoC), so safety stays total (explicit `swap`/G2,
  `MissingConversion`, ADR-016) while traversal stays maximally easy. Block edges would add only
  *auditability*, and only by constraining where crossings may live (forbidding mid-body swaps) — so the
  *boundary-audit* idea is **routed to observability tooling (M-345 → DN-04 / RFC-0008)** as a
  location-independent "every representation crossing + its honesty bound" view, where lossy conversions
  live. The enforced block-edge boundary is recorded as an optional future discipline (RFC-0012 §9, not
  adopted); the RFC-0005 decision-table form stays gated on RFC-0005 policy-objects in `mycelium-l1`.
  **M-351 (#114) closes with no new ambient surface.**

### Added (Phase 4 — M-344: enact RFC-0012 ambient representation; surface-only, never a black box)
- **`mycelium-l1::ambient` — the ambient resolution pass (RFC-0012 §4.3/§4.4 enacted).** A *declared,
  scoped, paradigm-only* default (`default paradigm P`) plus block-scope overrides
  (`with paradigm P { … }`) and a paradigm-less repr `{N}` / `{N, scalar}` / `{model, dim, sparsity}`,
  to offset honesty's verbosity (tension **A**) **without** a black box. Realized as a **surface→surface
  "expand to longhand" pass**: `resolve(Colony) → Colony` fills omitted paradigm tags, strips
  `with paradigm` blocks, and tags bare decimals, then the **unchanged** `check → elaborate` pipeline
  runs — so the two normative invariants hold *by construction*: **(I1)** the ambient inserts no `Swap`
  (it only fills tags/encodings — conversions stay author-written), and **(I2)** resolution is
  observationally the identity (`elaborate(p) = elaborate(resolve(p))`, identical content hash;
  RFC-0001 §4.6). The feature is **opt-in**: a program with no ambient resolves to itself unchanged.
- **Bare-decimal width-from-context (RFC-0012 §4.3; the maintainer-chosen v0 scope).** The checker is
  now **bidirectional**: a bare decimal under an ambient adopts the paradigm's encoding and takes its
  **width from the checked context** (an ascription, a parameter/return/field type, or a concrete
  sibling operand of a width-preserving prim). Where the width is **not** determined, it is an explicit
  **`UnresolvedWidth`** refusal — *never a built-in default width*. `Binary` unsigned and `Ternary`
  balanced encodings are range-checked (an overflow is an explicit refusal, never a silent wrap).
- **Three never-silent refusals (no black box; G2).** `UnresolvedAmbient` (a `{…}` with no enclosing
  ambient — no implicit global fallback), `ParadigmShapeMismatch` (a shape that does not fit the ambient
  paradigm — never coerced), and `MissingConversion` (a cross-paradigm value edge — the checker’s
  cross-paradigm mismatch is sharpened to name from/to + point at writing an explicit `swap`).
  Bare decimals under `Dense`/`VSA` (no bare-decimal encoding) and a duplicate colony `default` are
  refused too.
- **"Expand ambient" projection (M-142/LSP; RFC-0012 §5).** `mycelium-l1::expand_to_source` +
  `mycelium-lsp::expand_ambient` render a document's fully-resolved **longhand twin** on demand (the
  elided default is never *hidden*, only *elided*); a parse/check failure is reported, never a partial
  render. Provenance for "where did this paradigm come from?" is recorded at the **surface/resolution
  layer** (`ResolutionNote` via `resolve_report`) rather than as a new core `Provenance` variant — that
  would change a frozen data-contract schema for metadata that is not hashed (KC-3; see the RFC-0012
  changelog).
- **The RFC-0012 §4.6 meaning-preservation differential (NFR-7; `tests/ambient.rs`).** A corpus of
  `(ambient program, explicit longhand twin)` pairs asserts **identical elaborated content hash** (I2)
  and identical observed value where runnable; the never-silent refusals are each tested as explicit
  errors. **Grammar + conformance**: `mycelium.ebnf` gains `default paradigm` / `with paradigm` / the
  paradigm-less repr, with a new accept fixture (`12-ambient-representation.myc`) and reject fixture
  (`09-default-missing-paradigm.myc`). **Kernel untouched** (KC-3 — L0's frozen node set is unchanged;
  this is RFC-0006 surface sugar that elaborates away). RFC-0012 (Accepted) → **Enacted**; R12-Q3/Q4
  resolved, R12-Q1/Q2 partially (v0 enacted, extensions deferred).

### Added (Phase 4 — M-349: dynamic depth budget for the AOT env-machine; DN-05 §2.4 / DN05-Q5 enacted)
- **`mycelium-mlir::budget` — a `DepthBudget` trait that resolves the env-machine's control-stack
  ceiling *dynamically*, with an `EXPLAIN`-able basis (DN05-Q5 resolved).** With the M-347 trampoline
  the control stack is on the **heap**, so the ceiling is honestly a policy over **memory**: the default
  `AutoDepthBudget` reads detected headroom — `MemAvailable` (`/proc/meminfo`) capped by a finite
  `RLIMIT_AS` (`/proc/self/limits`) — via **pure-`std` `/proc`** (Linux), spends 70 % ÷ a conservative
  1 KiB/frame estimate, and clamps to `[10 000, 2 000 000]`. **Zero `unsafe`** (no FFI, no SP-reading —
  ADR-014's "minimal unsafe" satisfied with *none*); non-Linux or any read/parse failure falls back to
  the conservative static default (the prior `200 000`), **never a guess**. The resolved ceiling **and
  its derivation** are an inspectable `DepthResolution`/`DepthBasis` (`Display`; `aot::default_depth_budget`;
  printed by `xtask recursion-probe`) — no black box (G2); the limit itself stays an explicit
  `EvalError::DepthLimit` (never an abort/hang). `run`/`run_core`/`run_core_with_fuel` now resolve it
  dynamically; `run_core_with_budget` keeps the explicit override. **Measured** on this host: `MemAvailable`
  ≈ 15.99 GB → raw ≈ 10.9M, clamped to the 2 000 000 ceiling (vs the old fixed 200 000); a constrained
  host *tightens* below the fallback (unit-tested: 256 MiB ⇒ ≈ 183k). A **property test** bounds the
  derivation (`[floor, ceil]`, monotone in headroom) for all inputs incl. saturation. Per-frame cost is a
  `Declared` over-count (VR-5), not `Proven`. Trusted interpreter unchanged; three-way differential holds
  (NFR-7). DN-05 §2.4 / **DN05-Q5 → Resolved**; native-path *stack* detection (DN05-Q4 / M-348) reuses the
  trait.

### Changed (Phase 4 — M-347: AOT env-machine made stack-robust via a trampoline; DN-05 #2 enacted)
- **`mycelium-mlir::aot` rewritten as a trampoline over an explicit heap control stack
  (`eval_machine`).** Object-level recursion now lives on the **heap**, so the env-machine uses **O(1)
  host stack** — matching the reference interpreter. Deep recursion is bounded by **two explicit,
  graceful budgets**: `fuel` (Fix unfolds; time → `EvalError::FuelExhausted`) and a **control-stack
  depth ceiling** (space → new `EvalError::DepthLimit { limit }`) — **never a host-stack abort, never a
  hang** (G2). `run_core_with_budget(fuel, max_depth)` exposes both; `run`/`run_core`/`run_core_with_fuel`
  unchanged. **Empirically confirmed** (`xtask recursion-probe`, re-run): the env-machine is graceful at
  every fuel to 5 000 000 (`FuelExhausted` ≤200k, `DepthLimit{200000}` ≥250k) — the pre-fix ~600-unfold
  abort (DN-05 §1.1) is **gone**. The three-way differential (L1≡L0≡AOT) is unchanged (NFR-7 holds).
  **RFC-0004 §2** now banks the matching **normative requirement** for the native MLIR→LLVM path
  (stack-robustness designed in, not retrofitted — DN-05 #1; libMLIR provisioning is M-348). The
  *dynamic* depth budget (derive `max_depth` from headroom) stays the deferred policy (DN-05 §2.4 /
  DN05-Q5); the fixed 200 000 default is conservative + configurable.

### Added (Phase 4 — DN-05 + recursion-probe: AOT recursion stack-robustness strategy, M-347/M-348)
- **DN-05 (Draft) — AOT recursion execution strategy, empirically grounded.** Investigates making the
  M-342 env-machine recursion stack-robust *without bloat*. New `xtask recursion-probe` **measures**
  (not presumes) the limitation: the AOT env-machine aborts (host-stack overflow) at **~600**
  `Fix`-unfolds, while the reference interpreter is graceful at fuel 5 000 000 in **O(1)** host stack
  (a tiny-AST `spin`, abort depth found by binary-searching fuel in subprocesses; re-runnable). Records
  the maintainer-set priority: **(1)** bank native MLIR→LLVM stack-robustness as a design requirement
  (libMLIR-gated; provisioning is near-term via desktop/WSL — **M-348** #110); **(2)** an explicit
  control stack / **trampoline** in the env-machine (near-term buildable; turns the abort into an
  explicit budget/limit — makes never-silent **total** for the AOT path); **(3)** **tail-call
  detection** — cautious, optional, on top of #2, only if it earns its keep (KC-3/KISS/YAGNI). Plus
  **§2.4: the limit must be *dynamic*** — detect stack/heap headroom + per-frame cost at runtime and
  derive the safe depth (the ~14 KB/unfold cost varies by build/platform, so a static constant is the
  wrong knob), behind a small `DepthBudget` trait with a conservative static fallback, `EXPLAIN`-able
  basis, and an explicit error (never an abort/hang/black box). The trusted interpreter stays the base
  for deep recursion until #2 lands; the M-210 differential must still hold (NFR-7). Tracked M-347
  (#109, P1) + M-348 (#110). Design-first — no fix lands with the note.

### Added (Phase 4 — M-342: AOT path extended to the data + recursion fragment; RFC-0011 §4.4 Q5 closed)
- **The AOT `aot::run` env-machine now covers the full v0 calculus (M-342).** `mycelium-core::lower`
  gains ANF for the r3/r4 nodes — `Construct`/`App` (flat) and `Lam`/`Fix`/`Match` (with **nested ANF
  blocks** evaluated lazily, a single program-wide temp counter keeping temps globally unique) — and
  `mycelium-mlir::aot` becomes a big-step **environment machine** with closures (capturing their env),
  call-by-value `App`, fuel-clocked `Fix` unfolding, `Construct`→`Datum`, and arm-selecting `Match`.
  `run_core` returns a `CoreValue` (repr **or** datum); `run` keeps the repr-`Value` signature.
- **The three-way differential now spans the full calculus.** `mycelium-l1`'s data/recursion corpus
  (data, nested matches, self-recursion, `for`-folds) is checked **L1-eval ≡ L0-interp ≡ AOT** on the
  L0 `CoreValue`, with the shared **M-210** checker validating each repr-result pair (NFR-7). Closes
  RFC-0011 §4.4 **Q5**; `Node::is_aot_lowerable` is now total over the v0 node set.
- **Honest scope (VR-5).** The *native* direct-LLVM backend stays the **bit/trit subset** — the data +
  recursion nodes are an explicit `UnsupportedNode` refusal there (data/closure native codegen is the
  deferred MLIR→LLVM work). The env-machine uses the **host call stack** for object recursion (the
  fuel clock bounds *productive work* — a non-productive recursion is an explicit `FuelExhausted`,
  never a hang — but depth beyond the host stack aborts); the trusted base for deep recursion remains
  the O(1)-stack interpreter. A follow-on, **M-347** (#109), tracks making the env-machine recursion
  stack-robust / more efficient.

### Changed (Phase 4 — RFC-0012 RATIFIED: Draft → Accepted)
- **RFC-0012 ratified (Draft → Accepted, 2026-06-16; append-only).** The ambient-representation
  design (§4) is now the normative surface contract: the two invariants (I1 the ambient emits no
  `Swap`; I2 resolution is observationally the identity) and the never-silent override /
  `MissingConversion` rule are in force. The kernel is unaffected (KC-3 — RFC-0001's frozen node set
  is untouched). **No code lands with acceptance** — the elaborator/checker wiring is the gated
  follow-on **M-344** (#106): the resolution pass, the never-silent refusals, M-142/LSP "expand
  ambient" rendering, and the §4.6 meaning-preservation differential. RFC README + Doc-Index updated.

### Added (Phase 4 — roadmap: Mycelium core library / stdlib, M-346)
- **M-346 (#108) — core-library / stdlib roadmap anchor.** Records the maintainer's goal of a solid
  core-library feature set for usability, to be decomposed once the surface language is self-hosting
  (dogfooding; free of other *languages*). Inherits the non-negotiable principles (never-silent G2,
  honest per-op guarantee tags, no black boxes/EXPLAIN, small kernel KC-3 — stdlib lives *above* it,
  content-addressed ADR-003; Rust-first ADR-007 now → Mycelium-lang eventually). Seeds: `diagnostics`
  (DN-04), collections, numerics helpers, VSA/encoding utils, I/O + wire-form serialization. No code;
  draft a Core Library RFC near self-hosting and present before folding.

### Added (Phase 4 — DN-04 Draft: optional structured diagnostics, DynEL-inspired, M-345)
- **DN-04 (Draft) — evaluate DynEL's (`gitlab:albedo_black/DynEL`) feature set as *opt-in* structured
  diagnostics** (`docs/notes/DN-04-…`). Source read (maintainer-supplied zip). **Governing
  constraint:** diagnostics are *additive presentation* over Mycelium's explicit, reasoned errors —
  **never a substitute** for a never-silent error/`Option`/`CheckVerdict::NotValidated` (G2). Imports
  the *contracts* — graded context levels (minimal/medium/detailed), human + machine-readable (JSON)
  output as two **projections** of one content-addressed diagnostic (G11/M-380), and a **reified
  per-definition error-handling policy** `{exceptions, custom_message, tags}` (the RFC-0005 pattern;
  ADR-006) — and explicitly **excludes** DynEL's three anti-patterns: `eval`-on-config (code
  execution), full `os.environ` dump at the detailed level (secret leakage), and `logger.catch`
  exception-swallowing (a never-silent violation). **Rust-first (ADR-007): no Python added** — DynEL
  is reference-only; the feature is a Rust tooling-layer renderer (kernel untouched, KC-3), **eventually
  self-hosted in Mycelium-lang** (dogfooding; free of other *languages*). Tracked as M-345 (#107);
  Doc-Index + `idmap.tsv` / `issues.yaml` updated.

### Added (Phase 4 — RFC-0012 Draft: ambient representation & scoped overrides, M-344)
- **RFC-0012 (Draft) — a surface-only, declared, scoped, *paradigm-only* representation default +
  scoped override/conversion blocks** (`docs/rfcs/RFC-0012-…`), to offset honesty's verbosity (tension
  A) while refusing black boxes. The honest core is two **normative invariants**: **(I1)** the ambient
  emits no `Swap` (it fills an *omitted paradigm* + bare-literal encoding only — conversions stay
  author-written, WF1/WF2); **(I2)** resolution is observationally the identity — a program with the
  ambient and its longhand twin elaborate to *identical* L0 ⟹ identical content hash (RFC-0001 §4.6),
  defended by a meaning-preservation differential (NFR-7/M-210). Forbids the two black-box failure modes
  (repr-inference-from-usage; silent conversion insertion); cross-paradigm edges stay explicit `swap`s
  and a missing one is an explicit `MissingConversion` refusal (G2). The **trusted kernel is untouched**
  (KC-3) — L0's frozen node set does not change; this is RFC-0006 surface/term-layer sugar that
  elaborates away. Cross-module: exported signatures are concrete L0 reprs (ADR-016 boundary), so the
  ambient never leaks across modules. Per maintainer direction (2026-06-16): **paradigm-only**
  granularity, **full v0 scope** (defaults + overrides). **No code, no RFC-0001 change** — Draft is the
  present-before-fold step; ratification + wiring are the maintainer's append-only decision. RFC README +
  Doc-Index updated; issue M-344 (#106) added to `idmap.tsv` / `issues.yaml`.

### Changed (Phase 4 — ADR-016 + ADR-017 RATIFIED: Proposed → Accepted)
- **ADR-016 + ADR-017 ratified (Proposed → Accepted, 2026-06-16; append-only).** Maintainer gate
  cleared — no change to either decision. ADR-016 fixes the interpreted↔compiled ABI (dispatch by
  content hash; the RFC-0001 §4.8 wire form as the canonical value boundary); ADR-017 fixes
  hot-inject (hash-keyed dispatch + content-addressed dynamic linking, immutable-by-construction).
  ADR README + Doc-Index status updated to Accepted; the RFC-0004 §10 OQ-1/OQ-2 pointers stand.

### Added (Phase 4 — M-341: the in-process hot-inject prototype on the M-340 JIT)
- **`mycelium-mlir` gains the `inject` module — ADR-017's named first build step (ADR-016 call ABI).**
  An `Image` holds a `ContentHash → entry` dispatch table over the M-340 `dlopen` JIT:
  - **a call resolves to a compiled entry if present, else interprets** the registered definition
    (the RFC-0004 §9.1 continuum); a hash with neither is an explicit `InjectError::DispatchMiss`,
    never a silent guess (G2/SC-3) — and `resolve` makes the dispatch decision `EXPLAIN`-able;
  - **`inject` loads a content-addressed unit and registers a new `hash → entry`**, never mutating a
    live entry (publish-once; an edit is a new hash under a new entry — the atomicity hazard
    dissolves, ADR-017 decision 4);
  - **`recompile_closure`** computes the changed dependency-closure by hash reachability over the
    dependency graph — the recompile set, with no AST/file diff (decision 3).
  **Verified (NFR-7):** the injected-compiled path is observationally equivalent to the reference
  interpreter through the shared **M-210** TV checker (`ObservationalEquiv`); the safety argument is
  exercised under test — an in-flight call to the old hash finishes on old code while a new caller
  dispatches to the new hash (`tests/inject_hotswap.rs`). **Honest scope (VR-5):** in-process proof
  only; a unit is a *closed* bit/trit-subset program and the call boundary is the call ABI restricted
  to nullary units — the args-carrying value ABI (RFC-0001 §4.8 wire form) and cross-process / native
  units (RFC-0004 §2 / §10 OQ-3) stay deferred. New issues M-341 (#103), M-342 (#104, AOT-fragment
  extension), M-343 (#105, mutual-recursion elaboration) created + added to `idmap.tsv` / `issues.yaml`.

### Added (Phase 4 — ADR-016 + ADR-017 Proposed: the interpreted↔compiled ABI + hot-inject)
- **ADR-016 (Proposed) — the interpreted↔compiled ABI (RFC-0004 §10 OQ-1).** Dispatch a compiled
  stable component by its **content hash** (versioning is free, staleness structurally impossible —
  ADR-003: a change is a new hash, so an old compiled entry can never be applied to a changed
  definition); cross `CoreValue`s in the **self-describing wire form** (RFC-0001 §4.8) as the canonical
  value ABI, with a zero-copy fast-path as a *later, validated* optimization (robust/portable first).
  Honesty crosses the boundary (`Meta`/guarantee travel with the value — WF5). The boundary is
  toolchain, not kernel (KC-3); codegen deferred (MLIR→LLVM, RFC-0004 §2).
- **ADR-017 (Proposed) — hot-inject recompiled definitions (RFC-0004 §10 OQ-2).** A hash-keyed
  dispatch table (ADR-016) + content-addressed dynamic linking (the M-340 `dlopen` JIT is the seed):
  inject = load a content-addressed unit + register `hash → entry`, **never** mutate running code. The
  classic atomicity hazard **dissolves** because definitions are immutable — a change is a *new hash
  under a new entry*, so in-flight calls finish on old code and new callers dispatch to new code; the
  recompile set is **exactly the changed dependency-closure** by hash reachability (no AST diff). A
  working in-process prototype on M-340 is the recommended first build step once ratified; native
  codegen deferred. RFC-0004 §10 OQ-1/OQ-2 now point at the ADRs; ADR README + Doc-Index updated.

### Added (Phase 4 — RFC-0004 §9.2/§9.3 reference impl: build-target profiles in mycelium-build)
- **`mycelium-build` gains the `target` module — the build-target profiles (RFC-0004 r2 §9.2/§9.3),
  orthogonal to the §4 stable-component gate.** `BuildProfile` = `Interpret` (no targets, dev default)
  / `Slim(Target)` (one) / `Selective(set)` (a chosen subset) / `Fat` (all supported) — fat is
  first-class but optional; `targets()` resolves each to a concrete `(os, arch)` set. Slim/selective/fat
  share **one** artifact shape, a content-addressed per-target `VariantTable` (§9.3), with **never-silent
  runtime dispatch** (`select(host)` → the host's variant or an explicit `DispatchMiss` the caller
  resolves by interpreter fallback or refusal — never a wrong-target variant, G2/SC-3). **Honest scope
  (VR-5):** `realizable_targets` admits only the **host** today — a non-host `--slim`/`--target`/`--fat`
  is an explicit `BuildError::CrossTargetDeferred` (cross-target codegen awaits the MLIR→LLVM backend,
  RFC-0004 §2), never a host-only build mislabeled as fat. This is the build-orchestration layer that is
  *ready* for that backend, not the backend. (RFC-0004 §9; 15 build-crate tests)

### Added (Phase 3/4 — M-310 real LSP document sync, on the now-complete text→Node→L0 pipeline)
- **`mycelium-lsp` gains real document sync (`sync` module + `serve` wiring).** With the surface→L0
  pipeline complete (RFC-0011 r3 / RFC-0001 r4), the LSP server now handles
  `textDocument/didOpen`/`didChange`/`didClose` (full sync — `TextDocumentSyncKind.Full`, advertised
  in `initialize`), re-analyzing the whole document through **parse → check** on each edit and pushing
  `textDocument/publishDiagnostics` (cleared on a clean edit / close). **Honest spans (VR-5):** a
  *parse* diagnostic carries a **real** `line:col` range (the lexer's `Pos`); a *check* diagnostic is
  located at its `fn <name>` declaration with the function name in `data.breadcrumb` (the checker
  tracks the failing function, not yet the failing sub-expression span — flagged, never fabricated).
  `mycelium-lsp` now depends on `mycelium-l1` for the text→`Node` path (no cycle). Closes the M-310
  residual that the RFC-0011 enactment unblocked; phase-3 M-310 row → Done. 515 workspace tests pass.

### Changed (Phase 4 — RFC-0001 r4 ENACTED: Lam/App/Fix in L0; full L1-in-Core-IR)
- **Functions + general recursion are folded into the trusted Core IR (RFC-0001 r4), completing
  L1-in-Core-IR and retiring RFC-0007 §4.6's `Residual` for self-recursion entirely.** A
  self-recursive, data-building, matching program now elaborates to a closed L0 term and runs on the
  trusted reference interpreter + the M-210 differential.
  - **RFC-0001 r3 → r4** (append-only; **supersedes the r3 §4.5 grammar**): §4.5 gains `Lam` + `App` +
    `Fix` (RFC-0007 §4.1; **R7-Q1 resolved — a `Fix` node**); §4.2 gains the **function value model**
    (maintainer-confirmed: the v0 surface is first-order, so `Lam`/`App`/`Fix` are **closed** —
    application is capture-free substitution, **no environment-capturing closure value**, honoring
    §4.7; capturing closures + partial application are a named later revision); §4.6's **cycle-ordering
    is finished** (**R7-Q3 for identity** — a mutually-recursive declaration group now content-addresses
    canonically + name-independently). RFC-0007 §4.6 `Residual` retired except mutual recursion +
    dynamic guarantee indices; the `matured` totality gate (RFC-0007 §4.5) restated unchanged (the
    interpreter clocks every `Fix` — a mis-classification gates packaging, never meaning).
  - **Code:** `mycelium-core` (the three nodes + content-addressing + the canonical
    `canonical_cycle_order`); `mycelium-interp` (small-step β-reduction CBV; `Fix` unfolds by
    substitution under the fuel clock → non-productive recursion is an explicit `FuelExhausted`, never
    a hang; applying a non-function / a bare-function result are explicit refusals);
    `mycelium-l1::elab` (each reachable self-recursive function → `let f = Fix(f, λparams. body)`,
    calls → curried `App`, non-recursive calls still inline; `for` → a synthesized self-recursive
    `Fix` fold; **mutual recursion** → explicit `Residual`, deferred R7-Q3); `mycelium-lsp` walks.
  - **Verified (NFR-7):** the M-210 differential extends to the recursive + `for` fragment (L1-eval ≡
    elaborate→L0-interp on the `CoreValue` observable), with a mutual-recursion-refuses witness. 509
    workspace tests pass; clippy clean; `cargo fmt` applied. (RFC-0001 r4 / RFC-0007 §4.6/§8 Meta)

### Changed (Phase 3 — exit gate RE-ASSERTED MET; both residuals closed)
- **`docs/planning/phase-3.md` moves `Living draft → exit-gate met`.** With residuals **R1** (M-310
  text→`Node` path) and **R2** (RFC-0006/0007 ratified) both closed by the RFC-0011 r3 enactment, the §6
  gate's three conditions are satisfied: native execution path (met+measured), matured toolchain (the
  parser→checker→elaborate→L0 pipeline exists; the `didOpen`/`didChange` wiring is an ordinary M-310
  task, not gate-blocking), and L1 surface (RFC-0011 r3 enacted, RFC-0001 → r3). Claimed at the strength
  the checked runs establish (VR-5): 497 workspace tests + the M-210 data-fragment differential. Phase-3
  build tasks (M-310 sync, M-350/M-360 locals) continue past the gate; the standing core-language
  continuation is **RFC-0001 r4** (`Lam/App/Fix` into L0). Append-only (supersedes the "no exit gate
  claimed" line). (phase-3.md §6.1)

### Changed (Phase 3 — RFC-0004 r2: interpreted↔compiled continuum + build-target profiles; additive)
- **RFC-0004 gains §9 (the interpreted↔compiled continuum + build-target profiles) and §10 (open
  questions) — additive, changing no r1 decision (append-only).** Records the maintainer's execution
  direction (2026-06-15): **interpret freely during development (zero build step, the reference
  interpreter is the meaning), compile what is ready, never be forced into a heavyweight build, never
  recompile what has not changed.** §9 makes explicit that execution is a *per-definition continuum*
  (not interpreted-vs-compiled), that mixed interpreted + compiled stable components coexist in one run
  (same L0 `CoreValue` semantics, §3 checker guarantees agreement), and that **incremental compilation
  is "for free" from content-addressing** (ADR-003 — a definition's hash is its identity, so a compiled
  artifact is never stale; M-311/M-312 already realize the cache). The **build-target profiles** are
  normative and flexible: `interpret` (default), `build --slim <os>-<arch>` (one target), `build
  --target <list>` (a chosen subset), `build --fat` (all supported targets, universal) — **fat
  multi-target is first-class but optional, supported from the start**, the slim/selective/fat artifacts
  share one format (a content-addressed per-`(os,arch,cpu-features)` variant table), and runtime variant
  dispatch is **never-silent** (an unmatched host falls back to the interpreter or refuses explicitly,
  never runs a wrong-target variant — the M-360 SIMD feature-dispatch generalized). Cross-target rides
  §2's MLIR→LLVM path and stays **host-only until that backend lands** (honest deferral). §10 flags the
  genuinely-new, undesigned items: the interpreted↔compiled **ABI** (OQ-1), **hot-inject** of recompiled
  definitions into a running image (OQ-2; the M-340 `dlopen` JIT is the seed), the **fat-artifact
  packaging format** (OQ-3), and target-set-as-RFC-0005-policy (OQ-4). (RFC-0004 r2 Meta)

### Changed (Phase 3 — RFC-0011 r3 ENACTED: data + flat `Match` in L0; RFC-0001 → r3; M-320/M-310)
- **The L1 data-and-matching core is now folded into the frozen Core IR and implemented in lockstep
  (RFC-0011 r3, enacting the named RFC-0001 revision).** `Construct` + the flat `Match` are L0 Core IR
  nodes, so a non-recursive program that builds/matches data reaches the trusted reference interpreter
  and the M-210 differential — closing the text→`Node` gap that blocked **M-310** document sync
  (gate residual **R1 closed**) and dead-ended **M-320**'s decision-tree compiler.
  - **RFC-0001 r2 → r3** (append-only; **supersedes the r2 §4.5 grammar**): §4.5 gains `Construct` +
    flat `Match` + `Alt` and **WF6/WF7/WF8**; §4.6 gains the content-addressed **data registry Σ**
    (`CtorRef = #T#i`, Unison self-recursive placeholder hashing; mutual recursion implemented but
    deferred to r4 per R7-Q3); §4.2 gains the **data value `Datum`** + the runtime sum **`CoreValue`**;
    §4.7 gains the **datum guarantee-summary** addendum. RFC-0011 → **Accepted, r3 ENACTED**; RFC-0007
    §4.6's `Residual` is **narrowed** (retired for data/matching; `App`/`Fix`/`for` stay `Residual`, r4).
  - **The one genuinely-open value-model choice (maintainer-confirmed):** `Datum` is a **sibling** type —
    `Value<R>` is unchanged, *not* refactored into a `Repr | Data` sum — and carries a **meet-summary
    guarantee with no `Bound`** (bounds stay on the leaf representation values; an addendum to §4.7). The
    smaller, isolated change honors KC-3/KISS/YAGNI (data values arise only as `Construct`/`Match`
    results, never as `Const` literals in r3).
  - **Code:** `mycelium-core` (the registry, `Datum`/`CoreValue`, the nodes, content-addressing +
    canonical dump; AOT stays repr-only via `Node::is_aot_lowerable`, RFC-0011 §4.4 Q5);
    `mycelium-interp` (small-step `Construct`/`Match` + `eval_core`; `Construct` = `meet(fields)`;
    `Match` meet is identity for `Exact` scrutinees and an **explicit refusal** for a non-`Exact` data
    scrutinee — never a fabricated bound); `mycelium-l1::elab` (the M-320 Maranget tree lowers nested
    patterns to nested flat L0 `Match`, binding all constructor fields; `if` → `Bool` match).
  - **Verified (NFR-7):** the M-210 differential extends to the data fragment — **L1-eval ≡
    elaborate→L0-interp** on the `CoreValue` observable (`L1Value::to_core` bridges name-keyed →
    `#T#i`), with a mutant-witness; the M-310/M-320 phase-3 rows and §6.1 exit-gate verdict updated
    (R1 + R2 closed). 497 workspace tests pass; clippy clean; `cargo fmt` applied.
  - **Honesty/scope (VR-5):** `Lam/App/Fix` remain the named **r4** revision (full L1-in-Core-IR,
    R7-Q1/Q3); the AOT path and mutual-recursion cycle-ordering are explicit, flagged deferrals — not
    silent gaps. (RFC-0001 r3 / RFC-0011 / RFC-0007 §4.6 Meta)

### Changed (Phase 3 — RFC-0006 & RFC-0007 ratified, Draft → Accepted r4; maintainer sign-off)
- **RFC-0006 (surface/term-layering) and RFC-0007 (L1 kernel calculus) are now Accepted (r4), with a
  scoped §10 carve-out.** A completion-review found **no missing normative content** in the
  KC-2-independent scope — both are mature, and the v0 L1 calculus is prototype-realized in
  `crates/mycelium-l1` and exercised by the M-320 usefulness + decision-tree work — and the maintainer
  signed off on the carve-out. **Ratified:** RFC-0006 §3 layering / §4.1 invariants S1–S6 / §4.2
  capability targets LR-1…LR-9 / §4.3 grammar discipline / §8 positions Q2·Q4·Q5·Q7 (now realized by
  RFC-0007 §4.1–4.7 and the ratified **RFC-0011** staged-r3 `Match`-into-L0 decision), and RFC-0007
  §4.1–4.8 (the v0 calculus, stage-0 dynamic guarantee check). **Stays gated/deferred (NOT ratified):**
  concrete L3 surface syntax (KC-2/M-002-external), stage-1 static grading (RFC-0006 Q3 implicit-flows
  decision / R7-Q2), R7-Q1·Q3 → RFC-0001 r4, R7-Q4, and traits/LR-2. No design content changed on
  acceptance; each RFC's status line + §10 carry the carve-out so "Accepted" is never read as ratifying
  the gated parts (VR-5). RFC README index + Doc-Index status updated. This unblocks the core-language
  step (the RFC-0011 r3 enactment + M-320 L0 wiring). (RFC-0006 r4 / RFC-0007 r4 Meta)

### Changed (Phase 3 — true bitnet.cpp 1.67-b/w TL2 layout closes A5-08, M-360; E3-6; RFC-0004 §5)
- **`mycelium-mlir::pack` now realizes `TL2` as the true bitnet.cpp layout (1.67 b/w).** The prior
  `TL2` was a placeholder that packed identically to the `FiveTritPerByte` base-3 reference (5
  trits/byte ⇒ 1.6 b/w), while the selector cost model priced TL2 at the published **1.67 b/w** — the
  A5-08 discrepancy. `TL2` is now the real layout: **3 trits → a 5-bit LUT-index** (`c = d₀+3·d₁+9·d₂
  ∈ [0,27)`), bit-packed as a contiguous 5-bit-field stream ⇒ `5/3 ≈ 1.67` b/w — *less* dense than the
  1.6-b/w base-3 reference on purpose (the 5-bit index is directly LUT-addressable, bitnet's fast-decode
  trade). The two schemes are now genuinely distinct densities; a new shared `needed_bytes(scheme,
  count)` bound model (`⌈5·⌈count/3⌉/8⌉` for TL2) replaces the per-byte assumption. The native TL2
  **dot kernel** (`mycelium-mlir::bitnet`) decodes the bitstream inline (`digit = (code / 3ᵖ) mod 3`)
  with a **branch-free bounds-clamped 2-byte window** — the second byte index is clamped to the last
  valid byte (computed from `n`), so the final group's read never goes out of bounds even when its
  5-bit field fits in one byte (spilled bits masked off by `& 31`). Oracle-checked across widths
  (`jit_dot_matches_reference_all_schemes`); the bound is a refusal test; new `pack` property tests pin
  the 1.67 b/w density and the TL2≠`FiveTritPerByte` distinctness. The selector cost model now **matches**
  the codec — **A5-08 resolved** (the notes in `pack.rs` and `mycelium-select` updated from "stand-in /
  inert discrepancy" to "resolved"). `cargo xtask e1` §3 times the true TL2 kernel (≈1.25× vs scalar —
  honestly *slower per-element* than I2_S, the bitstream decode being more work; as-measured).
  **Honesty/scope (VR-5):** realizes the bitnet.cpp TL2 *density + 5-bit-LUT-index semantics*; the exact
  upstream byte/bit ordering is not claimed byte-identical (needs the source to verify) — the codec is
  self-consistent (round-trip identity) and oracle-checked. (phase-3.md §2 / §9.8 / Meta)

### Added (Phase 3 — BitNet hand-vectorized SIMD kernel, M-360; E3-6; FR-C3 / G3; RFC-0004 §5/§8)
- **`mycelium-mlir::simd` — a hand-vectorized (8-wide) I2_S packed-ternary dot kernel.** The scalar
  BitNet kernels decode one trit per loop step; this emits `i64 @myc_bitnet_dot_simd(ptr %w, ptr %x,
  i64 %n)` that unpacks + multiply-accumulates **8 trits per iteration** with LLVM vector types:
  broadcast the two packed bytes across 8 lanes (`shufflevector` mask `<0,0,0,0,1,1,1,1>`), bring each
  lane's 2-bit code to bit 0 (`lshr` by the constant vector `<0,2,4,6,0,2,4,6>`), `& 3` → code, `− 1`
  → signed weight, `mul <8 x i32>` with the contiguous activations, widen + accumulate into an
  `<8 x i64>` phi, then horizontally reduce (`@llvm.vector.reduce.add.v8i64`) with a **scalar epilogue**
  for the `n mod 8` tail. Every vector op is visible in the emitted IR (no opaque pass — FR-C3 /
  RFC-0004 §6); the vector loads carry explicit `align 1`/`align 4`. It reuses `BitnetDotKernel`'s
  bounds-checked `call` (a `pub(crate) from_loaded` ctor — DRY; same C signature + I2_S density model),
  so a short buffer is still an explicit refusal, never an OOB read. **The vector unpack is
  correctness-critical, so it is differential-checked against the scalar kernel as the oracle** —
  `tests/simd_differential.rs` runs a corpus bracketing the 8-lane width and the tail
  (n ∈ {0,1,7,8,9,15,16,17,31,33,64,255,256,257,1000}) and validates each scalar↔SIMD pair **through
  the single shared M-210 checker** (`ObservationalEquiv`/`Exact`), with a mismatched-buffer
  discrimination test (guard 7) so a green pass is not vacuous. `cargo xtask e1` **§5** times SIMD vs
  scalar over the same runtime buffer (indicative ≈1.2× — honest: clang already auto-vectorizes the
  scalar `-O2` loop, so the hand-vectorized gain is real-but-modest; as-measured, no target
  pre-written). **Scope/honesty (VR-5/G3):** **I2_S only** this increment (TL1/TL2 vectorized unpacks,
  plus the true 1.67-b/w bitnet.cpp **TL2 layout** that closes A5-08, are next); no parity with bitnet.cpp's
  AVX2/AVX512 LUT kernels is claimed; same exact dot product, no guarantee upgraded; the scalar kernels
  stay the oracle. (phase-3.md §2 / §9.8 / Meta)

### Added (Phase 3 — RFC-0011 the keystone: L0 `Match` / L1-in-Core-IR, ratified-decision; M-320/M-310)
- **`docs/rfcs/RFC-0011-L0-Match-and-L1-in-Core-IR.md` (Accepted — decision; enactment sequenced) — the named RFC-0001 revision.**
  The L0 Core IR is frozen at five nodes (`Const/Var/Let/Op/Swap`); RFC-0007 designed five L1 nodes but
  stopped short of putting them *into* L0 (its §4.6 elaboration covers only the evaluation-complete
  fragment, the rest is an explicit `Residual`). RFC-0006 §4.4 step 2 and RFC-0007 §9 name the missing
  step — "add the L1 node set to the Core IR" — and **this is that proposal.** It is the keystone for two
  stalled half-tasks: **M-320** (emit Maranget decision-tree leaves as real L0 nodes — blocked because L0
  has no matching node) and **M-310** (document sync — blocked because there is no text→`Node` path for
  matching/data). The RFC recommends a **staged** revision — **RFC-0001 r3** = the data-and-matching core
  (`Construct` + flat `Match` + a content-addressed data registry, with new kernel WF6/WF7/WF8 lifting
  RFC-0007's W6/W7/W8), staged ahead of an **r4** that adds `Lam/App/Fix` — so the five-node kernel grows
  in two auditable steps (KC-3). It recommends the **flat `Match`** as the kernel node (the M-320 Maranget
  tree stays the *untrusted, inspectable* compilation artifact above the kernel, per RFC-0007 §6), and
  records the two alternatives a maintainer might prefer (a low-level `Switch`/`Leaf` kernel form; the
  one-shot five-node fold). **Ratified 2026-06-15 (decision only; enactment sequenced).** The maintainer
  chose the staged path; RFC-0011 is **Accepted as the decision**, but because it depends on RFC-0007 and
  the maintainer directed that **RFC-0006 + RFC-0007 be completed and ratified first**, the §4.7 enactment
  — the RFC-0001 r2 → r3 text-fold, the RFC-0007 §4.6 narrowing, and the M-320 elaborator wiring — is
  **deferred** to land together as the core-lang step, in order: *exit-gate assembly → M-360 SIMD →
  ratify RFC-0006/0007 → enact r3 + wire*. **Frozen-L0 not flipped (VR-5):** RFC-0001 stays r2/frozen and
  the prototype keeps returning `Residual` until that step. Registered in the RFC README index and the
  Doc-Index. (phase-3.md §9.9 keystone)

### Added (Phase 3 — JIT runtime specialization, M-340; E3-4; ADR-009/ADR-014; RFC-0004 §5/§8)
- **`mycelium-mlir::specialize` — a weight-specialized ternary dot kernel (the classic JIT win).**
  The generic BitNet dot kernel (M-360) reads its weight buffer as a runtime pointer and re-unpacks it
  every call. In the inference setting the **weights are fixed at runtime** and only the activations
  vary, so `emit_specialized_dot_ir(weights)` bakes the (runtime-known) weight vector into the kernel
  `i64 @myc_bitnet_dot_spec(ptr %x)` as constants. The optimiser then **drops the unpack entirely**
  (no packed-byte load / shift / mask / `code−1`), **elides every zero-weight lane** (a `0` weight's
  activation load + multiply vanish from the emitted IR — the model's sparsity becomes inspectable,
  FR-C3), and **strength-reduces ±1 to a single `add`/`sub`**. The only runtime argument is the
  activation pointer; weights and length are compiled in. `compile_specialized_dot` JIT-compiles it
  (`clang -shared -O2`) via the M-340 dynamic loader; `SpecializedDotKernel::call` takes **no weight
  argument** (running it against weights it was not built for is unrepresentable — never a silent
  stale-weights run) and **bounds-checks** the activation buffer (a short buffer is an explicit
  `AotError`, never an OOB read). `nonzero()` exposes the surviving-lane count for EXPLAIN/inspection.
  **Validated (NFR-7):** `tests/specialize_differential.rs` runs the specialized and generic kernels
  over the same activations and validates them as observationally equivalent **through the single
  shared M-210 checker** (`ObservationalEquiv`, `Certificate::exact()` ⇒ `Validated{Exact}`), plus a
  negated-weights discrimination test that the checker must reject (guard 7, so a pass is meaningful).
  **Honest speedup (E1 §4 / VR-5):** `cargo xtask e1` §4 times specialized-vs-generic over the same
  runtime activation buffer (both runtime pointers, no constant folding) after an oracle cross-check;
  indicative single run (n=4096, ~66 % dense) ≈ **10.7× as measured** — reported as-measured, no
  target pre-written, sparsity/machine-dependent. **Honesty/scope:** same exact dot product, no
  guarantee upgraded (both `Exact`); the weights are runtime data baked at JIT time, activations stay
  runtime pointers, so the compute is real. (phase-3.md §2 / §9.10 / Meta)

### Added (Phase 3 — L1 Maranget decision-tree compiler, M-320; E3-3; RFC-0007 §3/§4.4)
- **`mycelium-l1::decision` — the codegen half of the Maranget pipeline.** Compiles a checked
  nested-pattern `match` into a flat decision `Tree` of `switch`/`leaf` nodes over **occurrences**
  (paths into the scrutinee) — Maranget 2008's "good decision trees": a left-to-right column heuristic
  (rotate the first non-wildcard column to the front), constructor/literal specialization, and a
  `default` branch **exactly** when a column's signature is incomplete (a data type missing
  constructors) or its domain is open (`Binary`/`Ternary`, never enumerated). This is RFC-0007 §3's
  "patterns compiled away by the elaborator", as the analysis-level IR. **Verified, not asserted:** a
  test-only tree evaluator (`eval_tree` over concrete `Pat` values) is checked to agree with a
  reference matcher on every `Nat` value up to a depth (a wrong column choice / specialization would
  diverge), plus first-match-on-overlap and the literal-needs-a-default shape. **Wired into the
  checker:** `checkty::infer_match`, after exhaustiveness passes, compiles the match and confirms the
  tree is `has_reachable_fail`-free — an exhaustive match must compile to total coverage, so the
  usefulness analysis (Maranget 2007) and the tree compiler must agree (defense in depth; an internal
  disagreement is an explicit error, never silent). **Honesty/scope (VR-5):** the tree's leaves are
  **not yet emitted as L0 Core IR** — L0 has no `Match` node, and adding one is the planned RFC-0001
  revision (RFC-0007 §4.6); the compilation algorithm is real and checked, and the L0 emission is the
  remaining step. No guarantee is touched; RFC-0006/0007 ratification stays the maintainer's
  append-only decision. (phase-3.md §2 / §9.9 / Meta)

### Added (Phase 3 — LSP wire protocol, M-310; E3-3; FR-S5 / SC-5)
- **`mycelium-lsp::wire` wraps the feedback facade in the LSP transport.** The byte-level JSON-RPC 2.0
  codec — `read_message`/`write_message` with `Content-Length` header framing (a clean inter-message
  EOF returns `None`; a truncated body / missing or invalid `Content-Length` / non-JSON body is an
  explicit `io::Error`, never a silent partial read) — plus the `Diagnostic` → LSP-`Diagnostic` mapping
  (spec `DiagnosticSeverity`: Error→1, Warning→2), the `textDocument/publishDiagnostics` notification
  builder, and a minimal `serve` lifecycle loop (`initialize` → capabilities + `serverInfo`,
  `shutdown` → null result, `exit` → stop; any other **request** → JSON-RPC `MethodNotFound` -32601,
  never silence; unknown notifications ignored). New dependency: the workspace-pinned `serde_json`.
  **Honesty/scope (VR-5):** not a document-syncing server — the facade analyzes Core IR `Node`s, not
  source text, so the server advertises `TextDocumentSyncKind.None` and the diagnostic `range` is a
  **zero placeholder** with the navigable location carried in `data.breadcrumb`; real source spans and
  `didOpen`/`didChange` sync arrive with the L1 surface (M-320), and the wire layer carries them
  without a protocol change. Seven tests (framing round-trip incl. back-to-back, clean-EOF,
  truncated-body refusal, severity mapping, `publishDiagnostics` shape, the scripted-client lifecycle,
  the unknown-request refusal). (phase-3.md §2 / §9.7 / Meta)

### Added (Phase 3 — BitNet TL1/TL2 packed-ternary kernels, M-360; E3-6; RFC-0004 §5/§8)
- **`mycelium-mlir::bitnet` now covers all three bitnet packings.** The I2_S-only dot kernel
  generalised to `emit_bitnet_dot_ir_for(scheme)`: **TL1** inverts the rot=2 code LUT
  (`d01 = (code+1) mod 3`, signed weight `d01−1`) and **TL2** decodes the base-3 5-trits/byte packing
  (`digit = (byte / 3ᵖ) mod 3` with the `3ᵖ ∈ {1,3,9,27,81}` divisor chosen by an inline select-chain),
  each a scalar loop with the scheme-specific unpack inlined and **inspectable** in the emitted LLVM IR
  (no opaque pass — RFC-0004 §6 / FR-C3). `BitnetDotKernel` carries its `PackScheme`, so the
  weight-buffer bounds check tracks the packing density (`n.div_ceil(4)` for I2_S/TL1, `/5` for TL2) —
  a short buffer stays an explicit `AotError`, never an OOB read. A non-bitnet `PackScheme` (Unpacked /
  TwoBitPerTrit / FiveTritPerByte) is the new explicit `AotError::UnsupportedScheme` refusal, never a
  silent misdecode. Each kernel is **differential-checked** against the packing-independent oracle
  `ternary_dot_ref` over the same `pack_trits` packing (`jit_dot_matches_reference_all_schemes`, n up to
  1000; the JIT actually compiled+ran here, matching all three). The **E1 §3** harness
  (`cargo xtask e1`) now times **all three** packings in-process over runtime data, each against a
  hand-written scalar baseline doing the identical per-scheme unpack (measured here: JIT beats scalar
  1.69× I2_S / 1.31× TL1 / 1.15× TL2 — whatever was measured, no pre-written claim, VR-5). The
  **A5-08** cross-reference notes (`mycelium-mlir::pack`, `mycelium-select`) are refined: the scalar
  TL2 kernel decodes the **1.6-b/w placeholder codec**, so it does *not* resolve the published
  1.67-b/w TL2 discrepancy (still inert for selection) — aligning to bitnet.cpp's true TL2 layout is
  now explicitly tied to the **real-layout / SIMD** increment, not the scalar kernel. **Honesty/scope:**
  scalar loops only — no parity with bitnet.cpp's hand-tuned **SIMD** is claimed (the next M-360
  increment); no guarantee is upgraded (VR-5/G3). (phase-3.md §2 / §9.8 / Meta)

### Added (Phase 3 — board sync: Phase-2 issues closed, Phase-3 M-3xx bootstrapped)
- **Tracker hygiene only.** Closed the completed Phase-2 epics (E2-1…E2-7, #28–34) and tasks
  (M-230…M-260, #58–65) as *completed* with grounding comments (CHANGELOG Batch G/H; Phase-2 exit gate
  met 2026-06-12). Created the Phase-3 M-3xx build tasks (#86–#98) from `tools/github/issues.yaml`,
  linked as sub-issues under E3-1…E3-7, closed the six shipped ones (M-301/302/303/311/312/370). Updated
  `tools/github/idmap.tsv` (M-301→#86 … M-380→#98) and `docs/planning/phase-3.md` §2/§8/Meta. No code or
  corpus-normative change.

### Added (Phase 3 — decode `enum_budget` default ratified, M-350; ADR-015; RFC-0010 §8)
- **`docs/adr/ADR-015-decode-enum-budget-default.md`** (Accepted): ratifies the RFC-0010 decode-selector
  default **`DEFAULT_ENUM_BUDGET = 4096`** (= `MAPI_RESONATOR_PROFILE.max_capacity`), the
  *guarantee-maximal* arm — every in-regime request is also enumerable, so the brute-force `Exact` arm
  dominates the whole validated envelope (never take `Empirical` when `Exact` is cheaply available) —
  over the *cost-optimal* ≈128. Grounded in the already-measured `∏k ≈ 100–128` cost-parity crossover
  (`d`-independent; ≈ 19× / ≤ ≈ 157 ms latency tax at the regime edge `∏k=4096`; cited from the
  `decode_method_enum_budget_crossover` instrument, **not re-run**). Tagged a `Declared` policy stance;
  neither value upgrades any guarantee (VR-5) — the budget moves only *which arm runs*, never *what tag
  it earns*. The cheap resonator-arm identifiability precheck (RFC-0010 §8) is recorded as the deferred
  re-open trigger (YAGNI). Standalone decision record — **no code, kernel, or test change**. Registered
  in `docs/Doc-Index.md` and the ADR index; RFC-0010 §8's `enum_budget` open question marked **resolved**
  (append-only footer).

### Fixed (Phase 3 — resonator premature-abort, M-350; RFC-0009 §3/§6)
- **Resonator no longer aborts a still-converging tuple as an oscillation.** The §3 loop decided
  oscillation on *any* recurrence of the decoded index tuple `ι`, so a tuple that had gone **stationary
  on `ι` while its per-slot confidence was still climbing** toward `τ_lock` (e.g. F=3,k=16, Hebbian,
  d=4096: the correct tuple at iter 2 with slot similarities `[1.0, 0.998, 0.72↗]`) recurred in the
  history at distance 1 and was mislabelled `Oscillating{period:1}` — a recoverable instance refused.
  The fix splits the two cases the discrete `ι` alone conflated: a **genuine limit cycle** (a *distinct*
  earlier tuple recurs ⇒ `period ≥ 2`) still refuses as `Oscillating`; a **stationary tuple** keeps
  iterating while the lock bottleneck (min per-slot similarity) is still rising and only refuses, with
  the new explicit `StopReason::Stalled` / `VsaError::ResonatorStalled` verdict, once that climb
  plateaus below `τ_lock` for `STALL_PATIENCE` sweeps (genuine stuck fixed point — **never-silent
  preserved**). Net effect: F=3,k=16 went **1/300 → 0/300** on the seed that exhibited the abort; the
  canonical 1000-trial gate stays **0/1000 ⇒ δ=0.02** (the gate's worst corner was already 0/1000, so
  the conservative ceiling is **unchanged** — no unmotivated tightening, VR-5). Tag stays **`Empirical`,
  MAP-I only, never `Proven`**; only a clean `Converged` clearing `τ_lock` + confidence + margin yields
  factors. The prior `stall_below_lock_*` unit test was updated (not deleted) to assert the new `Stalled`
  verdict; a regression test pins the exact previously-aborting instance to `Converged`. (phase-3.md §2 / Meta)

### Added (Phase 3 — resonator-network factorization prototype, M-350; RFC-0009 §10.2)
- **`mycelium-vsa::resonator`** — the RFC-0009 §3 factorization loop over any `VsaModel`
  (MAP-I-first), recovering the unknown factors of a bind product `s = x₁ ⊛ … ⊛ x_F`. Parallel /
  Jacobi **snapshot** update (§8.1 P6); softmax-superposition or arg-max cleanup (§9 Q2); uniform /
  seeded init (§9 Q1); convergence **and** oscillation decided on the **discrete top-atom index tuple
  `ι`** (§8.1 P3), bounded by the iteration budget. Deterministic via an in-crate LCG (no `rand`).
- **Never-silent honesty made structural (RFC-0009 §5.4/§6).** `factorize` returns a `Factorization`
  **only** on a clean `Converged` verdict that clears `τ_lock` + per-slot confidence + margin;
  `BudgetExhausted`, `Oscillating`, below-confidence, and below-margin are explicit `VsaError`s
  carrying the inspectable `ResonatorTrace` ("converged ≠ correct").
- **`ResonatorProfile` + `MAPI_RESONATOR_PROFILE`** — the `{F, ∏kᵢ, d}` regime gate
  (`check` → `OutsideEmpiricalProfile`; `bound` → `EmpiricalFit`), distinct from the bundle
  `EmpiricalProfile` (§5.2/§9 Q4). First regime `F≤2, k≤8, ∏k≤64, d≥4096`.
- **Trial-validated δ, oracle-measured.** `tests/resonator_oracle.rs` asserts **exact-tuple recovery**
  against a brute-force oracle (+ an exhaustive-argmax identifiability check);
  `tests/resonator_profile.rs` runs exactly `trials` (1000) at the worst point, scoring exact recovery
  (not self-reported convergence — §8.1 P5): **measured 0/1000 ⇒ δ=0.01** conservative ceiling, the
  test that *earns* the `Empirical` tag (VR-5).
- **Value-level decode.** `mycelium-vsa::reconstruct_factors` mirrors `reconstruct_role`: reads the r4
  `Resonator` manifest params, gates on the profile, runs the loop. Tag is **`Empirical`, MAP-I only,
  never `Proven`** (schema-enforced); sparse/HRR/FHRR deferred (§9 Q6). Additive `CleanupMemory`
  `atoms()`/`dim()` accessors; four resonator `VsaError` variants. **Nothing new in the kernel** beyond
  the r4 additive manifest metadata fields. (phase-3.md §2 / Meta)

### Added (Phase 3 — RFC-0010 follow-ups: enum_budget crossover + Value-level wiring, M-350)
- **`enum_budget` crossover measured (RFC-0010 §8).** A wall-clock instrument
  (`tests/decode_select.rs::decode_method_enum_budget_crossover`, `#[ignore]`d) times brute force vs the
  resonator per decode across `{F, k, d}`: the **cost-parity crossover is `∏k ≈ 100–128`** (d-independent
  — both scale with `d`); brute force is cheaper only for `∏k ≲ 64` and costs **≈19×** the resonator at
  the regime edge `∏k=4096` (≈76 ms vs ≈4 ms, d=4096). So `DEFAULT_ENUM_BUDGET = max_capacity` (4096) is
  **guarantee-maximal** (always `Exact` in-regime, bounded ≤ ≈157 ms at d=8192), *not* latency-minimal
  (≈128) — recorded as-measured (VR-5); the default value is a guarantee-vs-latency policy call, exposed
  per call and surfaced in the EXPLAIN cost lines. `DEFAULT_ENUM_BUDGET`'s doc carries the trade.
- **Value-level auto-selected decode** — `mycelium-vsa::reconstruct_factors_selected` routes a
  `Resonator` manifest through the RFC-0010 selector (instead of always running the resonator),
  returning a `DecodeSelection` with the **tag read off the chosen arm**. Unlike `reconstruct_factors`,
  it does **not** pre-gate on the resonator profile — a brute-forceable instance *outside* the resonator
  regime (e.g. `F=4, k=8`, ∏=4096, which the plain decode refuses) is recovered **exactly** by brute
  force (RFC-0010 §4.4). Shared manifest→`ResonatorParams` reading refactored into a helper (DRY). Four
  new `recon` tests (brute-Exact, resonator-Empirical, the F=4 capability gain, non-resonator rejection).
  (phase-3.md §2 / Meta)

### Added (Phase 3 — RFC-0010 decode-methodology selector prototype, M-350)
- **`mycelium-vsa::decode_select`** — the RFC-0010 decode-methodology selector, reusing the **one**
  RFC-0005 selection mechanism as a **third site** (no parallel selector). `reconstruct_factors_auto`
  routes a factorization request among `{ BruteForceExact, Resonator, Refuse }` by an ordered decision
  table over **exact** facts (`F`, `∏kᵢ`, `d`, `ResonatorProfile` membership), runs the chosen arm, and
  returns the recovered factors with the **guarantee tag read off the arm** — brute-force enumeration is
  **`Exact`** (identifiability-checked against ties), the resonator is **`Empirical`**, else an explicit
  `VsaError::DecodeRefused`. Every selection emits the mandatory EXPLAIN (`explain_decode_method` is the
  pure, no-execution form). `DecodeMethodPolicy` is content-addressed (`enum_budget` is part of its
  identity).
- **Honesty floor enforced (RFC-0010 §4.5).** A forced `BruteForceExact` beyond `enum_budget`, a forced
  `BruteForceExact` on a non-identifiable instance (`VsaError::NonIdentifiable`), and a forced
  `Resonator` out of regime all still **refuse** — a first-class override cannot escape the floor or
  upgrade a tag (VR-5). The `mycelium-core::recon` `≤Empirical` ceiling is untouched.
- **Mechanism extended additively** (`mycelium-select`, core-only): an abstract `DecodeMethod`
  candidate, the `DecodeFacts` queryable facts, the `CapacityAtMost`/`FactorsAtMost`/`InResonatorRegime`
  predicates, and the `select_decode_method` adapter. `mycelium-vsa` now depends on `mycelium-select`
  (acyclic — `mycelium-select` is `mycelium-core`-only).
- **Honest finding recorded.** With `DEFAULT_ENUM_BUDGET = MAPI_RESONATOR_PROFILE.max_capacity` (4096),
  *every* in-regime request is also enumerable, so the brute-force `Exact` arm dominates the **entire**
  validated regime (never take `Empirical` when `Exact` is cheaply available) — the resonator arm
  becomes load-bearing only at a tighter budget (latency) or once the validated capacity grows beyond
  the enumeration budget. The `enum_budget` wall-clock crossover stays the RFC-0010 §8 open question.
  (phase-3.md §2 / Meta)

### Added (Phase 3 — RFC-0010 decode-methodology selection design, M-350 needs-design)
- **`docs/rfcs/RFC-0010-Decode-Methodology-Selection.md`** (Draft): the design artifact for choosing a
  **decode methodology** as a **third site of the one RFC-0005 selection mechanism** (no parallel
  selector — DRY/SoC). A content-addressed, `EXPLAIN`-mandatory decision table over **exact** metadata
  (`F`, `∏kᵢ`, `d`, model, `ResonatorProfile` membership) routes among
  `{ BruteForceExact (Exact), Resonator{Hebbian} (Empirical), Refuse }`, with the **guarantee tag read
  off the chosen arm** (VR-5) and out-of-regime / non-identifiable inputs an explicit refusal
  (never-silent — G2). Records the §10.3 finding that the **cleanup-variant axis collapses to one
  winner (Hebbian)** inside the validated envelope, so cleanup-selection is **deferred** (YAGNI) with a
  concrete re-open trigger. **No code; nothing in the kernel.** Registered in the Doc-Index + RFC index;
  design gated on ratification. (phase-3.md §2 / Meta)

### Changed (Phase 3 — resonator operational-capacity wall breached, §10.3 cleanup ablation, M-350)
- **`MAPI_RESONATOR_PROFILE` widened `F≤3, k≤8, ∏k≤512` → `F≤3, k≤16, ∏k≤4096, d≥4096`** by fixing the
  cleanup dynamics, **not** by loosening the honesty contract. The original softmax cleanup fed the
  *real-valued* superposition straight into the next bind, so crosstalk compounded through the
  elementwise product of `F−1` noisy real vectors — the prototype collapsed as `∏k → d`. The §10.3
  ablation (`tests/resonator_profile.rs::resonator_cleanup_ablation`, `#[ignore]`d) measured four
  cleanups at the wall; the **Hebbian bipolar** projection `sign(Σⱼ simⱼ·cⱼ)` (Frady et al. 2020) keeps
  the explain-away on the `±1` alphabet, so the MAP-I unbind stays *exact*. Measured at F=3,k=16
  (∏=4096): **softmax 300/300 fail → Hebbian 0/300** at d=4096; the canonical 1000-trial gate now
  validates the F=3/k=16/d=4096 worst corner at **0/1000 ⇒ δ=0.02** conservative ceiling. New
  `Cleanup::Hebbian` (the validated default) + `Cleanup::SoftmaxSign`; `ResonatorParams::mapi_default`
  and the unspecified-manifest decode path adopt Hebbian (the kernel `CleanupShape` is unchanged —
  Hebbian lives only in `mycelium-vsa`).
- **Honest boundary recorded.** `SoftmaxSign` does **not** breach the wall (sign of a sharp softmax ≈ a
  noisy arg-max); `ArgMax` only partially (brittle at the tight d=4096 corner). F=3,k=32 (∏=32768) is
  left **outside** the validated envelope: 0.085 at d=8192 (not tight), 0.005 only at d≥16384 — recorded
  as boundary data, not claimed. F=3,k=16 added to the brute-force oracle. Tag stays **`Empirical`,
  MAP-I only, never `Proven`**. (phase-3.md §2 / Meta)

### Changed (Phase 3 — resonator validated regime widened + operational-capacity map, M-350)
- **`MAPI_RESONATOR_PROFILE` widened `F≤2, ∏k≤64` → `F≤3, k≤8, ∏k≤512, d≥4096`** with a **measured**
  δ. A staged capacity sweep (`tests/resonator_profile.rs::resonator_capacity_sweep`, `#[ignore]`d)
  mapped the operational edge: F=2/k=8 = **0/300**; F=3/k=8 (∏=512) = **6/1000 = 0.006** at d=4096
  (→ **0.001** at d=8192) ⇒ **δ=0.02** conservative ceiling at the worst corner (gate re-measured
  4/1000 on a fresh seed). The canonical gate now validates the F=3/k=8/d=4096 worst point.
- **Operational-capacity wall recorded (honest boundary data).** The prototype's softmax resonator
  (β=6, budget 50) collapses as `∏k → d`: **F=3/k=16 (∏=4096) ≈ 100% failure even at d=8192/β=10**,
  and k=32 is hopeless. So `k≤8` is the validated edge for F=3 at these knobs — a far smaller
  operational capacity than the literature's tuned resonators, reported as-measured not as-hoped
  (VR-5). Tightening (β, d) helps the in-regime k=8 corner but does **not** breach the wall; that is
  left to a future increment (better cleanup/normalisation). F=3 added to the brute-force oracle.
  Tag stays **`Empirical`, MAP-I only, never `Proven`**. (phase-3.md §2 / Meta)

### Added (Phase 3 — RFC-0009 resonator-network factorization design, M-350 needs-design)
- **`docs/rfcs/RFC-0009-Resonator-Network-Factorization.md`** (Draft): the *needs-design* deliverable
  for M-350 — fixes the convergence regime and the honest guarantee **before** any factorization code
  is built (RR-5/G4). Specifies the iterative resonator update over the existing `VsaModel`
  bind/unbind/cleanup (Frady et al. 2020); a **probabilistic-only** contract (basis capped at
  `Empirical`/`Declared`, **never** `Proven`; the `mycelium-core::recon` `Resonator` schema already
  enforces this ceiling, FR-C2), with the operational regime `{F, kᵢ, d}` as a checked
  `EmpiricalProfile` side-condition; never-silent termination (bounded budget;
  `BudgetExhausted`/`Oscillating` are explicit verdicts, never a wrapped result); full
  reification/`EXPLAIN`; and the open design questions. Prior art (`embeddenator-retrieval`/`-vsa`)
  flagged to mine, not copy. **No code; nothing in the kernel.** Registered in the Doc-Index;
  prototype gated on ratification. (phase-3.md §2 / Meta)
- **RFC-0009 Draft revision — prior-art mining (M-350).** Read the reference implementations
  (`embeddenator-vsa::resonator`, `embeddenator-retrieval::core::resonator`) and folded the findings
  back into the contract while keeping status **Draft** and the honesty contract intact. New **§8.1**
  documents seven concrete pitfalls (unseeded init; an unbacked "self-inverse" on the *lossy*
  sparse-ternary bind; no oscillation detection + a wrong cosine-to-previous convergence test; no
  regime/`δ`; a wrong fixed point returned as an answer with no correctness test; in-place Gauss-Seidel
  rather than parallel update; silent zero-fill fabrication). **§9 open questions resolved as
  recommendations** (uniform seeded init; softmax default, `β = 1/temperature` trial-fit; discrete
  index-tuple convergence + bounded-window cycle detection; oracle-measured `δ` over a `{F, ∏kᵢ, d}`
  `ResonatorProfile`; confidence **+ margin** refusal via `CleanupMemory`; MAP-I-first, sparse/HRR/FHRR
  `Declared` not `Empirical`). Tightened §3/§5/§6 accordingly ("converged ≠ correct"; only a clean
  `Converged` verdict yields factors). Records the maintainer caveat that `embeddenator` is
  acknowledged-experimental / not-yet-working — mined for problem-discovery only, with no evidential
  weight for any guarantee or convergence regime (VR-5). Still **no code; nothing in the kernel.**
  (phase-3.md §2 / Meta)
- **RFC-0009 ratified — Draft → Accepted (M-350).** Maintainer ratifies the contract; status
  `Accepted` (append-only). Authorises the §10.2 prototype (next: the `mycelium-vsa::resonator` MAP-I
  loop + `ResonatorProfile` + brute-force oracle + Value-level `reconstruct_factors()` decode). The
  decode-side manifest params (`cleanup`/`init`/`τ_lock`/`β`/`seed`) land as additive `DecodeSpec`
  metadata fields via the append-only **RFC-0003 r4** revision — additive metadata only, no kernel
  logic/guarantee change, ≤`Empirical` ceiling preserved (RFC-0003 §2; KC-3). (phase-3.md §2 / Meta)

### Added (Phase 3 — L1 nested patterns + Maranget usefulness, M-320)
- **`mycelium-l1::usefulness`** — Maranget's usefulness algorithm `U(P, q)` over a typed pattern
  matrix (Maranget 2007), witness-returning. L1 `match` now supports **nested** constructor/literal
  patterns, with coverage *checked* (W7): **exhaustiveness** (a `_` must not be useful — the witness
  names a concrete missing case, e.g. `S(Z)`, reported verbatim) and **redundancy** (an arm covered by
  the earlier rows is unreachable, subsuming the M-320 duplicate-literal check).
- **Checker + evaluator + totality** lifted from flat to nested: a recursive, type-directed
  `check_pattern` (binders typed by field type, linearity enforced); a unified `infer_match` (data +
  `Binary`/`Ternary`); a recursive `try_match` in the evaluator; and structural-descent smallness
  seeded from **nested** sub-binders (so `S(S(m)) → m` descends and admits `matured`).
- **Scope/honesty:** RFC-0007 is **Draft** and the prototype non-normative; this is the analysis half.
  The Maranget *decision-tree compilation to the flat kernel `Match`* (Maranget 2008; RFC-0007 §3) is
  the elaborator/L0 path and lands with full L1-in-Core-IR. Coverage stays checked, no guarantee
  touched. (phase-3.md §2 / §9.9 / Meta)

### Added (Phase 3 — BitNet packed-ternary acceleration, M-360 first increment; closes the open E1 compute-throughput item)
- **`mycelium-mlir::bitnet`** — the canonical BitNet **ternary multiply-accumulate**
  (`y = Σ digit(wᵢ)·xᵢ`, ternary weights · integer activations) emitted as **inspectable** LLVM IR
  (`i64 @myc_bitnet_dot(ptr %w, ptr %x, i64 %n)`: load the packed I2_S byte, extract the 2-bit code,
  signed weight `code−1`, multiply-add — one transparent op per loop-body step, FR-C3 "metadata, not
  hidden lowering"). JIT-compiled (`clang -shared -O2`) and called **in-process over runtime-pointer
  buffers** via the M-340 dynamic loader (refactored into a reusable `dlopen_path`/`Lib::sym`).
  Differential-checked against the Rust oracle (`ternary_dot_ref`) over several widths; bounds-checked
  so a short buffer is an explicit `AotError`, never an out-of-bounds read.
- **`cargo xtask e1` §3 now measures genuine packed-ternary compute throughput.** Because the kernel's
  weight/activation buffers are runtime arguments (not baked-in constants), the optimiser cannot fold
  the computation — so §3 times real unpack-compute over `n = 4096` elements against a hand-written
  Rust scalar baseline doing the identical I2_S work. This resolves §2's constant-fold/spawn caveat
  that had blocked the compute-throughput verdict. **Scope/honesty:** I2_S + scalar only — no
  bitnet.cpp SIMD parity claimed, TL1/TL2 are the next increments; the E1 number is measured, not
  pre-written (VR-5 / G3). (phase-3.md §2 / §9.8 / Meta)

### Added (Phase 3 — native trit carry arithmetic `add/sub/mul`, M-301 done)
- **`mycelium-mlir` now lowers balanced-ternary carry arithmetic over `Ternary{m}`.** `trit.add` is a
  fixed-width **ripple-carry** (LSB→MSB; balanced digit `x srem 3 − 1` and carry `x sdiv 3 − 1` with
  `x = aᵢ+bᵢ+carry+4 ≥ 1`, so the LLVM `srem`/`sdiv` are euclidean), `trit.sub = add(a, neg b)`, and
  `trit.mul` is **shifted accumulation** in a 2m-trit buffer (each `b` digit scales `a` via `i32 mul`,
  the digit being ±1/0). Each mirrors `mycelium-core::ternary` digit-for-digit.
- **Fixed-width overflow is detected at runtime and never wraps silently (SC-3/G2).** A non-zero final
  carry (add/sub) or non-zero product high trit (mul) sets an `i1` flag carried through an extended
  **read-back protocol**: the AOT artifact prints a `'!'` sentinel line and the JIT kernel — now
  `i32 @myc_kernel(ptr)` — returns a non-zero status, both surfaced as an explicit `AotError::Overflow`
  matching the interpreter's `EvalError::Overflow`. The M-302 (native) and M-340 (JIT) differential
  corpora gain in-range add/sub/mul + a nested `(5+4)−4`, plus an overflow-parity test. **Completes
  M-301** (last open slice). (phase-3.md §2 / §9.1 / Meta)

### Added (Phase 3 — native-ternary forward-compat map, M-370)
- **`docs/notes/Native-Ternary-Forward-Compat.md`** (Living note): documents the **ternary
  value-semantics contract** and the forward map from today's emulated-on-binary packing to a future
  3-state hardware backend, with the `ternary` dialect (`mycelium-mlir::dialect`) as the **stub
  target** and the R7 portability guarantee (what a native backend must keep invariant — values, the
  selection mechanism, the honesty rule, interpreter-as-reference). Documentation + stub only; **no
  3-state backend built** (ADR-005 / VR-5). Registered in the Doc-Index. Completes E3-7 at the
  documentation level.

### Added (Phase 3 — in-process JIT, M-340; first intentional unsafe under ADR-014)
- **`mycelium-mlir::jit`** — an in-process JIT: emits the kernel as `void @myc_kernel(ptr)`, compiles
  it to a shared object (`clang -shared`), and calls it **in-process** via `dlopen`/`dlsym` (the
  first intentional `unsafe` FFI under ADR-014 — justified `// SAFETY:` comments +
  `#[cfg_attr(not(debug_assertions), allow(unsafe_code))]`, **no new dependency**). Reuses the same
  `lower_program` + element encode/decode as the AOT path, so it agrees with the interpreter through
  the shared M-210 `ObservationalEquiv` checker (`tests/jit_differential.rs`, NFR-7). Removes the
  process-spawn overhead of the M-303 AOT path; skips gracefully when `clang` is absent. **Honest
  E1:** the closed kernel constant-folds, so a calibrated compute-throughput verdict still needs
  runtime-input kernels (M-360) — not pre-written (VR-5). (phase-3.md §2 / Meta)

### Added (Phase 3 — native AOT trit slice `trit.neg`, M-301)
- **`mycelium-mlir::llvm` is now kind-aware** (a `Lane` carries `Binary{w}` *or* `Ternary{m}`): the
  direct-LLVM backend lowers **`trit.neg`** over `Ternary{m}` end-to-end (digit-wise `0 - x` — exact,
  no carry), printing ternary output as `'-'`/`'0'`/`'+'` via a branch-free `select` chain (still one
  op per element) and reading it back into a `Ternary{m}` value. The parse shape is derived from the
  actual lowering (`lower_program` is the single source of truth for `emit_llvm_ir` + `result_shape`).
  The M-302 differential corpus gains two trit-`neg` programs (compiled + checked). `trit.add/sub/mul`
  (balanced-ternary carry arithmetic) and `bit.*`/`trit.*` on the wrong lane kind are explicit
  refusals (G2). (phase-3.md §2 / Meta)

### Changed (decision — ADR-014: `unsafe` policy relaxed from `forbid` to permitted-but-warned)
- **`unsafe_code` is now `"warn"` workspace-wide (was `"forbid"`).** `unsafe` is permitted when
  explicit and justified: it **warns** in `cargo build`/`cargo test` (the caution incentive) and
  still compiles/runs, the `just check` lint gate exempts only this lint (`scripts/checks/lint.sh`
  now runs `clippy -- -D warnings -A unsafe_code`, every *other* warning still a hard error), and a
  site silences the dev warning **for production release** with
  `#[cfg_attr(not(debug_assertions), allow(unsafe_code))]` + a mandatory `// SAFETY:` comment.
  Recorded as **ADR-014** (append-only; amends the M-091 lint policy). Enables in-process JIT/FFI
  (M-340) via raw `extern "C"` `dlopen`/`dlsym` with no new dependency. The trusted-base crates stay
  unsafe-free. CONTRIBUTING + the ADR index updated.

### Added (Phase 3 — LSP maturation: structured feedback summary, M-310)
- **`mycelium-lsp::FeedbackSummary`** (`Feedback::summary()`): a structured roll-up of an analysis —
  per-artifact-kind counts, the Error/Warning breakdown, the worst severity, and `is_clean()` — the
  at-a-glance health signal an AI co-author's feedback loop (SC-5b/E3-2) or an IDE status line
  consumes without re-walking the channels. Adds `Diagnostic::path()` (the `at` breadcrumb as a
  navigable `Vec<&str>`). Two tests incl. a worst-severity mutant-witness. (phase-3.md §9.7)

### Added (Phase 3 — content-addressed build cache, M-312)
- **`mycelium-build::cache`** — `BuildCache` caches `BuildCertificate`s by **build-request** content
  address: the key folds the component's identity hash with every decision input (spec ratification,
  the three obligations, the `promote` flag), so an unchanged request is a `Hit` reusing the prior
  certificate and any change in verification state is a `Miss` that re-decides — never a stale hit
  (G2). Three tests incl. the weakened-obligation `Aot → Interpreted` miss (mutant-witnessed).
  (phase-3.md §9.6)

### Added (Phase 3 — build-system stable-component gate, M-311)
- **`mycelium-build`** (new crate, outside the trusted kernel — KC-3): makes the RFC-0004 §4
  stable/experimental gate executable. `check_eligibility` runs the automatic §4 checks (spec
  ratified + obligations discharged) with specific blocking reasons; `decide(component, promote)`
  routes to **AOT only for an eligible, explicitly promoted** component (promotion is deliberate,
  §4) and refuses promotion of an ineligible one (never a silent AOT). Emits a content-addressed
  `BuildCertificate` (`cert_ref`, BLAKE3) with private fields and a re-validating `Deserialize`
  (`deny_unknown_fields`) so a forged `Aot` certificate is rejected on deserialize. Seven tests incl.
  forged-AOT + unknown-field rejection. (phase-3.md §9.5)

### Added (Phase 3 — L1 literal-pattern `match`, M-320)
- **`mycelium-l1`**: `match` now covers `Binary{n}`/`Ternary{m}` scrutinees with **literal patterns**,
  not just data types (the explicitly-deferred v0 gap). `checkty::infer_literal_match` enforces
  repr+width-matching literal arms, rejects duplicate literals, and **requires** a `_`/binder default
  (the 2ⁿ/3ᵐ domain is never enumerated — W7 coverage is never assumed); `eval::eval_literal_match`
  fires an arm on `repr + payload` equality. Elaboration is unchanged (the `Match` family already
  lowers to `Residual`). Five tests incl. three mutant-witnessed refusals. RFC-0007 ratification is
  presented, not flipped — that stays the maintainer's append-only decision (concrete syntax remains
  KC-2-gated). (phase-3.md §9.4)

### Added (Phase 3 — E1 native-path measurement, M-303)
- **`cargo xtask e1` §2** now measures the native AOT path against the interpreter (M-303): one-time
  AOT compile cost, warm native per-invocation (process spawn + run), and interpreter per-eval, for a
  bit-subset program. The E1 verdict moves from "no native path (stub)" to **native path established
  and measured** — the *compute-throughput* verdict ("reaches hand-packed perf") stays honestly NOT
  established, now with a precise reason: the standalone tiny-kernel artifact is process-spawn-bound
  and constant-folds, so it needs in-process execution (JIT/FFI — M-340 / deferred libMLIR). Adds the
  `compile` / `CompiledArtifact::run` compile-once/run-many split to `mycelium-mlir::llvm` (with
  `compile_and_run` as the wrapper). **Batch J (M-301→M-302→M-303) complete at the task level.**
  (phase-3.md §9.3)

### Added (Phase 3 — interp↔native differential, M-302)
- **`mycelium-mlir/tests/native_differential.rs`** — extends the M-151 differential to the *compiled*
  path: a bit-subset corpus runs under the reference interpreter and `compile_and_run`, asserting
  observable `(repr, payload, guarantee)` equality **and** validation through the single shared M-210
  `ObservationalEquiv` checker (NFR-7/VR-4/RR-12). A discrimination test confirms the differential is
  non-vacuous (two different programs → `NotValidated`). Skips gracefully when `llc`/`clang` are
  absent. (phase-3.md §9.2)

### Added (Phase 3 — native execution path, M-301 bit-subset slice)
- **`mycelium-mlir::llvm`** — a **direct-LLVM-IR AOT backend** that genuinely compiles the kernel
  **bit subset** (`core.id`, `bit.not/and/or/xor` over `Binary{w}`) to native code. `emit_llvm_ir`
  renders textual LLVM IR (one SSA op per output bit — no opaque pass, RFC-0004 §6); `compile_and_run`
  drives `llc` + `clang` to a real executable, runs it, and reads the result back as an `Exact`
  `Binary{w}` value. This is the first *compiled* execution path (RFC-0004 §2's direct-LLVM fallback;
  libMLIR absent, LLVM 18 present — the MLIR dialect lowering stays deferred, RR-N1). Everything
  outside the subset is an explicit `AotError` refusal (never silent); `llc`/`clang` absence is a
  skippable `ToolchainMissing`. Tests cover emit shape/determinism, four mutant-witnessed refusals, a
  width-mismatch refusal, and a toolchain-gated native↔interpreter roundtrip. (phase-3.md §9.1)

### Added (Phase-3 planning — scoping cut)
- **`docs/planning/phase-3.md`** (Living draft): scopes the Phase-3 epics #35–#41 (`E3-1…E3-7`) into
  `M-3xx` build tasks. Records the batch/parallelization plan with the **native execution path as the
  keystone** (it unblocks E1 + JIT/BitNet/native-ternary), the Phase-2→3 KC-1…KC-4 re-run, a
  **proposed** exit gate scoped to the buildable/local deliverables (exploratory + KC-2-gated epics
  tracked as honest out-of-gate stretch), and the risk register. **No exit gate claimed.** New risk
  **RR-N1**: the env has LLVM 18 but **no libMLIR**, so the realized first native step is a
  **direct-LLVM-IR AOT backend** (the RFC-0004 §2 fallback) with the MLIR dialect path deferred — a
  sequencing decision flagged for maintainer ratification, not silently adopted. KC-2 (LLM API) and
  the MLIR path (libMLIR) are named as the two external blockers.
- **`tools/github/issues.yaml`**: the Phase-3 epics decomposed into `M-301…M-380` child tasks
  (issue numbers pending bootstrap). Companion-doc references in `phase-0/1/2.md` updated
  (`phase-3.md` is no longer "forthcoming").

### Fixed (deep-review remediation — Medium/Low/Nit tail; all findings now closed)
- The remaining **Medium/Low/Nit** findings across every workstream are resolved (one commit per
  area), completing the review's Gate-A list — **0 findings now open**:
  - **core/cert (WS2):** recon manifest schema↔Rust reconciled (A6-06), `swap-certificate` requires
    bijective `params` (A6-09), `MalformedSparsity` variant (A6-08), basis-rank rule (A1-04),
    SC-3 helper asserts strength (A1-05), kernel `unreachable!`→`debug_assert` (C1-05).
  - **vsa (WS3):** MAP-I/MAP-B bind/unbind enforce the ±1 alphabet (A3-04), `EmptyCodebook` variant
    (A3-07), BSC on-expectation `Proven` documented (A3-06), tie-break/HRR/SC-2 notes (A3-08/09/10),
    `Bundle.hs` header reconciled (A3-05).
  - **l1/interp (WS4):** reject corpus pins per-file expected error reasons (A4), `Wf`-path +
    fuel-at-depth tests (A4-04), documented depth ceiling (A4-03).
  - **select/mlir/dense (WS5):** non-ternary layout refusal (A5-02), `unpack_trits` returns a
    `Result` not a silent truncation (A5-03), non-vacuous dense sweep (A5-05), pinned op-eps
    constants (A5-07), comment fixes (A5-06/A5-08).
  - **lsp/kc2/xtask (WS6):** never-silent unsupported-swap-pair diagnostic (A6-05), `exec` gated
    behind `allow_untrusted` (A6-10/B2-04), kc4 bijective-dec precheck (A6-11).
  - **numerics (WS1 deferred):** kernel-type fields are `pub(crate)` with accessors + a validating
    `Certificate` deserialize, making the outward-rounding/range invariants structural (A2-05);
    composed `Proven` basis preserves the input theorems' provenance (A2-09).

### Added (developer tooling — supply-chain gate)
- **`just deny`** (`scripts/checks/deny.sh`, in `just check`): runs `cargo deny check` + `cargo
  audit` when present (skip-if-missing), with a root `deny.toml` (advisories/licenses/sources).
  `.github/dependabot.yml` added (github-actions + cargo + pip, weekly; PRs only — no auto-CI).
  `[profile.release] overflow-checks = true`. gitleaks/cargo-deny/cargo-audit added to
  `install-tools.sh`. `npx markdownlint-cli2` pinned. Editorial: docs say "ruff format
  (Black-compatible)"; codespell + markdownlint clean repo-wide.

### Fixed (deep-review remediation — Wave 1)
- **WS3 — VSA certified-capacity side-conditions (finding A3-03/C1-02 H6 — the last Wave-1 High;
  advances M-I2/VR-5, SC-2).** `MapI::bundle_values_certified` issued a `Proven` `CapacityBound`
  after checking only the dimension instantiation (`dim ≥ requiredDim`), but the cited
  Clarkson/Thomas theorem also assumes **bipolar (±1) atoms** and **distinct items** — so a `Proven`
  tag could be obtained for a bundle of duplicates or non-bipolar vectors. The certified path now
  checks both before issuing the bound (`check_bipolar` → `NonAlphabetComponent`; `first_duplicate`
  → new `VsaError::DuplicateBundleItems`), and the margin `μ` plus the checked side-condition are
  **recorded in the bound's basis citation** so EXPLAIN/serialization expose exactly what the
  `Proven` tag rests on. Regression test refuses non-bipolar and duplicate inputs and still certifies
  distinct bipolar ones (mutant-witness A3-03); an existing capacity test that built identical
  undersized atoms was corrected to use per-item seeds so it still isolates the dimension condition.
- **WS6 — KC-2 baseline oracle fidelity (findings A6-01 H10, A6-04; M-002 well-posedness).** The
  Python baseline DSL read `Bin` as **unsigned** while the kernel/spec use **two's-complement**, so
  the benchmark's two arms computed different answers for the same prompt — e.g. `kc2-05`
  `swap(0b1011_0010 → 6-trit)` gave the baseline `+178` vs the kernel/spec `−78` (`0-00+0`),
  invisible because the oracle checked only result *shape*. `baseline.Bin.to_int` and the `Tern→Bin`
  swap are now two's-complement (`B_n = [−2^(n−1), 2^(n−1)−1]`), matching `binary.rs` — the worked
  example now yields `−78` in both arms. Added an `expect_value` field to `Task` (the independently
  computed integer) and a well-posedness test asserting each reference baseline's `to_int()` matches
  it, so a value-wrong reference or a future convention drift is caught (A6-04). Scoring stays
  shape-only (SC-5b symmetry). Remaining WS6 (tracked): A6-05 (LSP unsupported-swap diagnostic),
  A6-10/B2-04 (`exec` `allow_untrusted` guard), A6-11 (xtask kc4 precheck).
- **WS5 — `mycelium-select` content-addressing integrity (finding A5-01/B2-02 H9; advances
  RFC-0005 §3).** `SelectionPolicy::new` (and, via it, `Deserialize`) now rejects a rule predicate
  carrying a **non-finite `f64` literal** (`Predicate::literals_finite`, recursing through
  `All`/`Any`/`Not`), with a new `PolicyError::BadPredicateLiteral`. `NaN` and `±∞` both serialize
  to JSON `null`, so two materially different policies (`eps ≤ NaN`, never-matches, vs `eps ≤ ∞`,
  always-matches) would otherwise hash to the **same** `policy_ref` — collapsing the audit anchor
  recorded in `Meta.policy_used`. Regression test asserts all three non-finite forms are refused
  (and nesting is checked), citing A5-01 as its mutant-witness.
- **WS4 — `mycelium-l1` soundness + parser hardening (findings A4-01 H7, A4-02 H8; advances S5/G2,
  RFC-0007 §4.5).**
  - **Totality soundness (H7):** the structural totality checker classified a non-terminating
    function as `Total` and admitted it as `matured`, because a `Match` arm binder reusing an outer
    "smaller-than" variable's name was never dropped — stale smallness leaked into the arm body and a
    non-decreasing recursive call looked structural (`f(n,p)=match n{Z=>Z,S(m)=>match p{Z=>Z,S(m)=>
    f(m,p)}}` diverges yet was accepted). `descend_walk` now drops every binder a pattern introduces
    (recursively) for the arm body and restores it after, re-adding only the genuinely-smaller
    constructor sub-binders — mirroring the existing `Let`/`For` discipline.
  - **Parser DoS (H8):** the recursive-descent parser had no depth guard, so crafted deeply-nested
    input overflowed the host stack and aborted `myc-check` (the M-002 oracle) instead of returning
    an error. `parse_expr` is now depth-guarded (`MAX_EXPR_DEPTH = 256`), returning an explicit
    `ParseError`; bounding the parser bounds the AST depth, protecting the downstream
    typechecker/totality/elaborator passes transitively.
  - Regression tests for both (the divergent witness is `Partial` + `matured` refused; 2000-deep
    input returns `Err`, not a crash), each citing its finding ID as a mutant-witness.
  - Remaining WS4 (tracked): A4-03 (charge eval depth per call-frame), A4-04 (`Wf`-error-path test),
    and switching the reject corpus from `is_err()` to per-file expected-error-substring assertions.
- **WS2 — `mycelium-core` contract integrity (findings A6-02, B2-03, A1-01, A1-02, A1-03;
  advances M-I1…M-I4, the schema contract).** The JSON schema is now enforced on the Rust side too,
  closing the tampered-manifest vector:
  - `#[serde(deny_unknown_fields)]` on `ValueWire`/`MetaWire`/`ReconWire`, so an unknown wire field
    is **rejected**, not silently dropped — `additionalProperties: false` is now a real contract on
    both sides (A6-02). (`Bound` uses `#[serde(flatten)]`, which serde cannot combine with
    `deny_unknown_fields`; its integrity is enforced by `well_formed` below instead.)
  - `Bound::well_formed` now also checks **finiteness** (an infinite ε/crosstalk is a vacuous bound,
    A1-02) and the **basis constraints** — an `EmpiricalFit` must rest on `trials ≥ 1` with a named
    method, a `ProvenThm` must name its citation — so an evidence-free `Empirical` tag (`trials: 0`)
    is refused on deserialize (A6-02/B2-03). Fixed the stale `MetaWire` doc claiming `reconstruction`
    is "not carried" (A1-01/A6-07).
  - New unit tests (`bound.rs`) and wire-tamper regression tests (`serde_roundtrip.rs`), each citing
    its finding ID as a mutant-witness (A1-03).
  - Remaining WS2 (tracked, not yet done): A6-03 (broaden the emit-then-validate schema pinning to
    one example per enum/basis/layout), A6-06 (recon schema↔Rust conditional reconciliation), A6-08
    (sparsity `WfError` variant), A6-09 (cert `params` schema drift), A1-04/A1-05 (nits).
- **WS1 — `mycelium-numerics` honesty hardening (findings A2-01, A2-02, A2-03, A2-04, A2-06,
  A2-07, A2-08; advances VR-3/VR-5, SC-2).** A `Proven`/`Empirical` ε or δ that travels in a
  `Bound` is now a *true* upper bound under floating point, closing the headline honesty hole where
  `compose_error_bound` emitted `ProvenThm` on round-to-nearest f64 that could fall below the real
  bound:
  - New private `round` module: directed (outward) rounding (`add_up`/`mul_up`) via the Knuth/Møller
    two-sum and an FMA, rounding a bound-increasing result up **only when IEEE actually rounded
    down** — so an exact composition (e.g. `Exact ⊕ Exact`) stays exactly `0` and is not silently
    inflated to "approximate".
  - Every ε/δ composition rounds outward: `ErrorBound::{add,scale,mul}`, `AffineForm::radius`, the
    `mul` second-order remainder, `ProbBound::union`, and `ApRhlJudgment::seq`. Each `AffineForm`
    op also folds the magnitude of its own center/coefficient round-off into a reserved
    `ROUNDOFF_SYM`, so `radius` is a sound enclosure under f64 (A2-01).
  - The tier-i checker's tolerance is now **relative** (a few ULPs of the re-derivation) instead of
    an absolute `1e-12` that was vacuous for tiny bounds — a claim of `eps = 0` against a re-derived
    `~5e-13` is now correctly **rejected** (A2-02).
  - `AffineForm::uncertain` returns `Option`, refusing a non-finite center / non-finite or negative
    radius instead of silently collapsing infinite uncertainty to an exact form (A2-03, house rule
    2); `compose_error_bound` re-validates the composed magnitude and refuses an overflow to
    non-finite rather than emitting a fabricated `inf` bound (A2-04); `AffineForm::mul`
    `debug_assert`s its fresh-symbol precondition (A2-06).
  - Property tests strengthened to assert with **zero slack** over both deviation signs (A2-07) and
    new regression/refusal tests added, each citing the finding ID as its mutant-witness (A2-08).
  - Deferred within WS1 (tracked, not yet done): A2-05 (make the kernel-type fields private — a
    cross-crate API change, kept separate from this rounding fix) and A2-09 (composed-`Proven`
    citation provenance, Nit). The outward-rounding guarantee holds for all current call paths,
    which construct these types via `new`/`exact`/the composition methods.

### Changed (deep-review remediation — Wave 1)
- **Dev tooling — banked review lessons into the skills.** `dev-workflow/SKILL.md` gains a "Banked
  guards" section and `_shared/review-rubric.md` a "Recurring defect patterns (grep-first)" list, so
  the honesty-rule seams the review exposed (outward-rounded f64 bounds, fail-closed bound
  constructors, `deny_unknown_fields` + schema re-validation, depth-guarded recursive descent,
  ambiguous-encoding hashing, shadowing-aware analyses, mutant-witness tests) are caught while
  authoring and during review, not only in audit. Each guard cites the finding that motivated it.
- **RFC-0003 → Accepted (r3): §4.1 erratum** reconciling the §4 guarantee-tag table with its own
  "Net" line, resolving review findings **A3-01 / A3-02 (H4/H5)**. On a checked algebraic basis:
  `permute` is `Exact` for every model (the table's "Proven" conflated the permutation *operation* —
  an exactly-invertible coordinate shift — with sequence-decoding error growth, which belongs to the
  `bundle`/`unbind` path), and the HRR/FHRR bind/unbind cell splits into bind `Exact` (exact algebraic
  convolution / complex product) and unbind `Empirical` (the lossy approximate inverse — the residual
  weak link, unchanged). Append-only: the r2 table cells are preserved, §4.1 is authoritative. **No
  code tag changes** — `mycelium-vsa::matrix.rs` / `tests/matrix.rs` already followed the Net line;
  the non-citable "issue #61" rationale in the code comment is replaced by the §4.1 citation.

### Added (developer tooling — code enumeration / mapping)
- **`just map`** (advisory; `scripts/map.sh`): generates a crate-to-crate dependency graph
  (`cargo depgraph` → Graphviz, `cargo tree` fallback), per-crate module/item structure
  (`cargo modules`), and rustdoc including private items, under `target/map/` + `target/doc/`. Not
  part of `just check`. Function-level call graphs in Rust are partial (trait dispatch / generics) —
  use rust-analyzer's call hierarchy or `cargo-call-stack` for those.
- **`just api` / `just api-baseline`** (`scripts/checks/api.sh`, `scripts/api-baseline.sh`): a
  public-API **surface gate** wired into `just check`. It diffs each crate's surface against a
  committed snapshot (`docs/spec/api/<crate>.txt`) and fails on an unreviewed change — a guardrail
  for KC-3 and the A2-05 private-fields work. All tools are optional and **skip gracefully** when
  absent (installer adds them best-effort); snapshots are bootstrapped with `just api-baseline`.

### Added (advisory review artifact)
- **Deep review (2026-06):** `docs/reviews/2026-06-14-deep-review/` — a four-stage advisory
  review (correctness + test-quality, security audit, quality/style vs the house rules, and a
  QC/PE improvement roadmap) of the Phase-1/Phase-2 code at HEAD `e2d627e`. Report-only, gates
  nothing, changed no code. Verdict: strong, honesty-disciplined codebase (0 Critical); 11
  distinct High findings clustered at the honesty-tag/contract seams (numerics `Proven`-on-
  unrounded-f64, VSA matrix/capacity over-tagging vs RFC-0003 §4, a totality-checker soundness
  hole, an unbounded-recursion parser crash, a selection `PolicyRef` collision, and
  schema↔Rust contract leaks). Not registered in `docs/Doc-Index.md` (advisory, non-normative).

### Added (Phase-2 Batch H — schedule-staged packing selector + E3 wrong-layout differential)
- **M-250 (`mycelium-select` + `mycelium-core::Meta::with_physical`):** the **schedule-staged
  packing selector** (RFC-0004 §5; DN-01 Resolved; RFC-0005 §4). `bitnet_packing_policy` builds the
  fixed bitnet.cpp candidate set (`I2_S`/`TL1`/`TL2`) with an `Always → Cheapest` rule over the
  bits/element cost model; `select_layout`/`record_packing_layout` reuse the **one** E2-6 selection
  mechanism (`select_packing`) — adding only the `PackScheme → PhysicalLayout::TritPacked` record
  mapping — and emit the mandatory EXPLAIN. The exhaustive cheapest is `TL2` (1.67 b/w)
  deterministically; a first-class override forces `I2_S`/`TL1`; out-of-range overrides are explicit
  errors. The chosen layout is recorded on `Meta.physical` via the new `Meta::with_physical`, a
  **lossless** record builder (**M-I5**: touches only `physical`, leaving guarantee/bound/value
  untouched). Determinism + override + M-I5 losslessness are tested (`tests/packing.rs`).
- **M-251 (`mycelium-mlir::pack` + `run_with_layout` + `tests/wrong_layout.rs`):** the **E3
  wrong-layout soundness differential** (RFC-0004 §8; NFR-7; RR-12). A substrate byte-layout codec
  (`pack_trits`/`unpack_trits`/`relayout_trits`) gives each scheme a bijective trit↔byte encoding —
  the three bitnet schemes are mutually distinct, so reading a buffer under the wrong scheme
  misreads it (decoding is total, never a panic). `run_with_layout` extends the M-151 interp↔AOT
  differential to the packing stage: a **correctly-labeled** layout (packed-as == tag) is the
  identity and **validates** through the M-210 `ObservationalEquiv` checker; a **mislabeled** layout
  (packed-as ≠ tag) misreads the buffer and the same checker reports an explicit
  `NotValidated{ Diverged }` — the circuit-breaker fires (the layout record the M-250 selector chose
  is trusted *only because a wrong one is caught*). The true scheme used is the one M-250 actually
  selects, tying the soundness check to the selector it guards.
- **E1 perf-harness stub (`cargo xtask e1`):** times the substrate packing codec's pack/unpack
  round-trip per scheme — the build-phase confirmation that staging is cheap to materialize (the
  calibrated kernel benchmark awaits the native libMLIR/LLVM path; ADR-009). Honest framing: it
  reports numbers, the E1 verdict stays **not established** (VR-5; deferred to the native path).
- Phase-2 status: epic **E2-7 complete at the task level** → **all five Phase-2 exit-gate build
  conditions met** (numerics, full swap + shared checker, selection + EXPLAIN, Dense + VSA breadth,
  packing + reconstruction). KC-1…KC-4 re-run at the gate (phase-2.md §5): KC-1 confirmed (build,
  no regression), KC-3 holds (the packing codec landed in `mycelium-mlir`, not the trusted kernel;
  core gained only the tiny `with_physical` record), KC-4 unchanged (the layout check is the
  existing ~10 ns observational instance). KC-2 (LLM-survives-the-surface) and the RFC-0006
  ratification remain open but are **out of the Phase-2 exit-gate scope** (external/maintainer).

### Added (Phase-2 Batch G — Dense surface, VSA breadth, Dense↔VSA swaps, reconstruction manifest)
- **M-230 (`mycelium-dense`, new crate):** the typed dim-tracked `Dense{dim, dtype}` operational
  surface (RFC-0001 §4.1) — `DenseSpace` binds dim+dtype in the type; `add`/`sub`/`scale` are
  `Proven` with per-element relative ε (Higham Thm 2.2, side-conditions checked per element;
  BF16 carries the two-rounding composition `2⁻⁸ + 2⁻²³`); `neg` is `Exact`; `dot`/`similarity`
  are `f64` measurement helpers. Off-grid payloads, overflow, subnormal results, and approximate
  sources are typed explicit errors; a 20k-pair sweep per dtype exercises the bound (SC-2).
- **M-240/M-241/M-242 (`mycelium-vsa`):** the **full RFC-0003 §4 model breadth** — MAP-B
  (sign-rounded bundle), BSC (XOR bind, majority bundle, centered Hamming similarity), HRR
  (circular convolution; correlation unbind), FHRR (phasor phase algebra; explicit
  degenerate-bundle refusal), and SBC (one-hot-per-block sparse codes with the T1.3 placement:
  declared `Sparse{max_active}` class in the `Repr`, observed `SparsityObs` in `Meta`). The §4
  guarantee matrix is encoded as the single source-of-truth table (`RFC0003_MATRIX`) asserted
  model-by-model in tests; **HRR/FHRR unbind stays the pinned `Empirical` weak link** (T1.2).
  New honesty pattern: a declared **`EmpiricalProfile`** (regime + δ + trial count) backs every
  `Empirical` Value-level op and is exercised by exactly its declared trials in
  `tests/empirical_profiles.rs`; outside-profile calls are explicit refusals. **RR-13 enforced:**
  MAP-B bundle nesting beyond depth 1 is the explicit `NestedBundleUnsupported` error.
- **M-231 (`mycelium-cert::dense_vsa`):** Dense↔VSA swaps (RFC-0002 §5) — bipolar `Dense{n,F32}`
  vectors encode as MAP-I superpositions over a deterministic versioned codebook (a genuine
  bipolar bundle, so the T0.2 capacity theorem applies); decode is provenance-gated signed
  correlation. The δ certificate's basis is derived, never asserted: `ProvenThm` iff
  `vsa_dim ≥ requiredDim(n, δ)` (the M-131 checked instantiation), `EmpiricalFit` iff the
  10⁴-trial profile covers the instance, an explicit `InsufficientCapacity` type error elsewhere.
  The **M-210 checker's δ-side lands** (the recorded `Incomplete` placeholder retired):
  `ProbabilityBound` certificates discharge by tier-i union-bound claim-vs-certificate plus
  deterministic re-derivation equality. `CertifiedSwapEngine` + the SC-3 global test cover the
  new rows (SC-2 satisfied for the new swaps).
- **M-260 (`mycelium-core::recon` + `mycelium-vsa::recon`):** the **reconstruction manifest**
  (RFC-0003 §6; `reconstruction-manifest.schema.json`, the ratified name) — `ReconInfo` with a
  validating constructor/deserializer (compositional ⇒ recipe; resonator ⇒ probabilistic-only,
  FR-C2), carried in the ratified `Meta.reconstruction` field (`with_reconstruction`); the
  submodule-side `reconstruct_role` executes the manifest with the threshold made explicit.
  Acceptance: the compositional path **recovers a novel combination** never stored in any
  codebook (the §6 exit criterion), wire-round-tripped end to end.
- Phase-2 status: epics **E2-1, E2-2, E2-5 complete at the task level**; the Phase-2 exit gate
  now waits only on Batch H (M-250 packing selector → M-251 E3 wrong-layout differential).

### Changed (RFC-0007 r3 — `for` spelling adopted)
- **RFC-0007 §4.8 → r3**: the bounded-iteration spelling `for x in xs, acc = init => body`
  moves from *provisional* to **adopted** (maintainer decision, 2026-06-10) — committed now
  rather than held pending a KC-2 ablation run. The kc2-09/kc2-10 benchmark tasks remain as
  measurements of the choice, not its gate; like all v0 surface syntax it stays under RFC-0006
  §1's global KC-2 gate, and revisiting it later is an explicit recorded decision (append-only).
  Wording updated in DN-03 §2, Lexicon Reference, Example-Programs note, `mycelium.ebnf`, the
  prototype doc-comments, and the KC-2 tasks docstring.

### Added (DN-03 — lexicon amendment; resolves ADR-012 §7.5/§7.6)
- **DN-03** (Resolved): amends DN-02 (append-only) through the three-test gate — **adopt**
  `consume` and `grow` (Surface), **decline** `embody` (inherent methods keep the conventional
  `impl`), **reserve** `for` (the RFC-0007 §4.8 bounded-iteration keyword). Ratifies the
  **one name per term** (flat) — **rejecting ADR-012 §7.6's canonical+alias scheme** as needless
  surface area (the "content-addressing makes a second spelling free" benefit is speculative; two
  labels per concept to keep in sync is a real cost now). Ratifies the single Runtime names
  against RFC-0008 §4.5's grounded meanings: `hypha`, **`fuse`** (RT6 is genuine merge —
  `anastomose`/`weave` dropped), `xloc`, **`cyst`** (encystment = the dormant resumable form;
  `cyst(…)` constructor-style like `spore`), **`graft`** (resolves the `myco` collision with the
  language family name), **`mesh`**, `forage`, **`backbone`** (was `rhizomorph`), **`tier`** (was
  `dimorph` — the canonical behavior is interpreted↔native tiering), `reclaim`. `reclaim` scope
  clarified (runtime units, never memory). Runtime vocabulary stays reserved-not-active. Lexicon
  Reference, Example-Programs note, and RFC-0008 §3/§4.2/§4.4/§4.5 updated to the single names;
  Doc-Index gains the DN-03 row.

### Added (ADR-013 — `spore` is the deployable unit; resolves ADR-012 §7.4)
- **ADR-013** (Accepted, maintainer deliberation 2026-06-10): `spore` = the
  **content-addressed deployable unit** — a hash-identified DAG of code (ADR-003 definitions,
  ship-by-hash per T4.3), values (with `Meta` intact), the RFC-0003 §6 **reconstruction
  manifest** as one digest-referenced component, and artifact metadata. The narrow ratified
  sense is the **degenerate case**: `spore(v)` constructs the single-value spore (the manifest
  for `v`); the schema name `reconstruction-manifest` is unchanged. Grounded in T4.3/T4.4
  (Nix/OCI/Wasm/Unison convergence on content-addressed artifact DAGs).
- **RFC-0003 → Accepted (r2)**: §6 scope note only — manifest contents, schema, and guarantees
  unchanged. **RFC-0008 R8-Q5** resolved at the scope level (schema/signing/germination contract
  remain the R2 implementation stage's obligation). Lexicon-Reference `spore` flag resolved;
  ADR index gains 012/013 rows.

### Changed (RFC-0007 r2 — bounded iteration; resolves ADR-012 §7.2)
- **RFC-0007 §4.8 (new, r2)**: bounded iteration as **elaboration-defined sugar** over
  structural recursion — no new kernel node. Normative content = the desugaring to a synthesized
  self-recursive helper over *linearly recursive* (nil/cons-shaped) data, classified `Total` by
  the existing §4.5 checker with zero extension (bounded **by construction**: values are finite
  and acyclic). Provisional spelling A — `for x in xs, acc = init => body` — ships in the
  non-normative prototype grammar (`for` reserved, recorded in DN-03); named-args `fold` is the
  planned L2 library form; the ratified spelling is **KC-2-evidence-gated** (T3.6).
  `while`/`loop`/`break`/`continue`/`return` stay excluded and **unreserved**, with *teaching
  diagnostics* where they already error (parse-level juxtaposition + check-level unknown name).
- **Prototype** (`crates/mycelium-l1`): `for` through the whole pipeline — lexer/parser
  (+ teaching diagnostics), T-For with explicit linear-shape refusals, totality (a `for` adds no
  recursion), an **iterative** spine-walk evaluator (long folds cost fuel, never host stack),
  elaboration `Residual` (Fix is outside the evaluation-complete fragment); EBNF + conformance
  corpus (`accept/11`, `reject/08`). **KC-2**: tasks kc2-09 (`for`) / kc2-10 (explicit
  recursion) form the runnable iteration-spelling ablation pair. 44 crate tests green.

### Added (RFC-0008 + Research Pass 4 — the Runtime tier, grounded)
- **Research Record 04** (`research/04-runtime-concurrency-RECORD.md`; findings **T4.1–T4.6**):
  the fourth research pass, grounding the Runtime tier ADR-012 §7.3 flagged as aspirational —
  concurrency units & structured lifetimes (Erlang isolation, nurseries, Kahn/LVars determinism,
  CakeML clocked-semantics extension), state merge & meshes (CRDT convergence, session types,
  epidemic protocols), mobility & placement (Unison ship-by-hash, the Legion
  placement-is-never-semantics separation, Reactive-Streams backpressure, work-stealing bounds
  with side-conditions), durability (CRIU's exception catalogue vs durable-execution's
  determinism requirement; Nix/OCI/Wasm content-addressed artifacts), failure & supervision
  (OTP, FLP, φ-accrual, Waldo et al.), and mode switching (verified deoptimization, CoreJIT).
  Primary-source verified with per-target uncertainty registers; three explicit novelty flags
  (no found precedent: determinism-gated checkpointability; learned-placement-as-inspectable-
  policy; per-value guarantee tags across a distribution boundary).
- **RFC-0008 — Runtime & Concurrency Execution Model** (Draft): the runtime model the Runtime
  vocabulary presupposed, built on Pass 4. **RT1–RT7 runtime invariants** extend S1–S6 to
  concurrency/distribution: values move & state is never shared (RT1); the deterministic
  fragment is the default with *sequential reference semantics* — NFR-7 extends to concurrency
  via the M-210 checker (RT2); nondeterminism is reified as RFC-0005 policies — placement
  becomes the **third site** of the one selection mechanism (RT3); partial failure is explicit,
  distribution transparency forbidden (RT4); runtime guarantees (delivery/convergence/failure
  suspicion) are tagged on the same lattice with `ProbabilityBound`s (RT5); fusion is lawful
  semilattice merge — payload joins, guarantee meets (RT6); runtime lifetimes are structured —
  *a leaked task is not expressible*, extending LR-9 (RT7). RFC-0004's per-node model is
  extended, not changed; the Runtime vocabulary is grounded (§4.5 operational-meaning table)
  but stays **reserved, not active syntax**, pending DN-03 + implementation RFCs. The `spore`
  scope reconciliation (ADR-012 §7.4) and name ratification are deliberately left to the
  RFC-0003 revision and DN-03 respectively. Indexes updated (`docs/rfcs/README.md`,
  `docs/Doc-Index.md`, Lexicon-Reference status notes).

### Added (L1 execution: evaluator, elaboration, three-way differential)
- **L1 fuel-guarded evaluator** (`crates/mycelium-l1/src/eval.rs`; RFC-0007 §4.6): a big-step
  environment machine mirroring M-110's contract — CakeML-style clocked semantics (explicit
  `FuelExhausted`, never a hang; T3.4), dispatching through the *same* trusted prim registry and
  certified binary↔ternary swap engine as the L0 paths (NFR-7). Runs the full checked surface
  (data values, flat `match`, recursion); the stage-0 **dynamic guarantee-index check**
  (RFC-0007 §4.3): asserting `@ g` stronger than a value's tag is an explicit
  `GuaranteeTooWeak` — an annotation may only weaken, never upgrade (VR-5). A separate explicit
  recursion-**depth guard** (`DepthExceeded`) keeps deep recursion an error, never a host stack
  overflow. Checker-unreachable states are explicit `Stuck` errors, never panics (S5/G2).
- **Elaboration to L0 on the evaluation-complete fragment** (`crates/mycelium-l1/src/elab.rs`;
  RFC-0007 §4.6): acyclic calls inline (CBV order preserved via `Let` bindings); bodies must
  reduce to `Const/Var/Let/Op/Swap` residue; recursion (`Fix`), `match`/`if`, data construction,
  and dynamic guarantee indices are explicit **`Residual` refusals — never a partial artifact**.
  Includes the shared surface→kernel bridge (literals, repr resolution) and the documented v0
  **policy-name reference** stand-in (deterministic, domain-separated; honest about deferring
  RFC-0005 name→policy-object binding) shared by both execution paths.
- **The RFC-0007 §4.6 differential** (`crates/mycelium-l1/tests/differential.rs`; NFR-7): on a
  10-program fragment corpus, **L1-eval ↔ elaborate→L0-interp ↔ AOT** agree on the observable
  (`repr + payload + guarantee`), with every agreeing pair validated through the **M-210 shared
  TV checker** (`ObservationalEquiv`) and a control asserting the checker rejects a genuinely
  divergent pair. Outside-the-fragment behavior is pinned too: elaboration refuses (`Residual`)
  while L1-eval runs — including a `Total`-classified structural recursion that terminates and a
  `Partial` one that exhausts fuel explicitly. 31 crate tests; `just check` green.

### Added (KC-2 harness)
- **KC-2 LLM-leverage harness** (M-002 structural deliverable; Foundation §6 P0.2; SC-5b; G10):
  `experiments/mycelium_experiments/kc2/` — the **fixed 8-task benchmark** (minimal Mycelium
  surface fragment vs a **Python-embedded DSL baseline**, both arms carrying checked reference
  solutions that prove the benchmark well-posed), the `myc-check` CLI oracle
  (`crates/mycelium-l1/src/bin/myc-check.rs`: parse / typecheck / task-signature conformance with
  distinct exit codes — no AI in the judging loop, S6), and the generate→check→feedback harness
  measuring **syntactic validity**, **first-attempt type-check pass rate** (the SC-5b number),
  and **edit-to-fix iterations**. *Running* the experiment remains blocked on LLM API access
  (the documented M-002 external blocker); the report hard-codes
  `verdict: not established` — never pre-written (VR-5). Baseline-arm execution is in-process
  `exec` and documented as requiring a disposable sandbox for untrusted model output. 8 pytest
  tests; `just check` green.

### Added (L1 static analysis + lexicon integration)
- **L1 typechecker + structural totality checker** (`crates/mycelium-l1`, RFC-0007 §4.4/§4.5):
  the v0 monomorphic typechecker over the data registry (declarations-as-registry), exhaustiveness
  checked (W7, never assumed), representation-typed literals, generics/`spore`/`wild` as explicit
  refusals; a Foetus-style structural-descent totality classifier whose verdict gates `matured`
  (mutual recursion stays Partial — R7-Q3). 8 tests; clippy clean.
- **Lexicon integration & architect review** (ADR-012 §7; `Lexicon-Reference.md`,
  `Example-Programs-Reference.md`, `Doc-Index.md`): verified the maintainer's three new lexicon
  documents against the corpus and integrated them. **Applied:** de-conflicted the lexicon
  "L1/L2/L3" tier labels (which collided with RFC-0006's language layers L0–L3) → renamed
  **Surface / Runtime / Formal**; fixed example bracket typos; added grounding notes. **Flagged for
  the maintainer (ADR-012 §7):** the Runtime tier (`hyph`/`anas`/`xloc`/…) is an *aspirational,
  ungrounded* concurrency/distribution model needing a Runtime RFC (RFC-0008) + research Pass-4 and
  reconciliation with RFC-0004; imperative `loop`/`while` contradicts the functional core
  (RFC-0007 §6); `spore` scope drifted from RFC-0003's reconstruction manifest; new Surface terms
  (`consume`/`embody`/`grow`) need a DN-02 amendment through the three-test gate (`embody` weakest);
  several short forms (`sclrt`/`cmn`/`anas`/`myco`) recommended for refinement; example
  bound-kind/partiality corrections. No contradictions found with ADR-010/011, the guarantee
  lattice, or content-addressing.

### Changed (RFC-0006 language-layer requirements)
- **RFC-0006 → r3 (Draft): two foundational language requirements** (maintainer direction;
  grounded in T3.5). **S6 self-sufficiency / AI-independence** — Mycelium is a complete software-
  engineering language whose parser/checker/elaborator/interpreter/AOT path are ordinary
  deterministic software runnable with **no AI/LLM in the loop**; models are an optional
  co-authoring convenience, never a runtime/compile-time/semantic dependency (remove every model
  and the language still builds, checks, runs, and reproduces bit-for-bit). This bounds KC-2: it
  can only choose the L3 surface, never make the language *need* a model. **LR-9 memory safety by
  construction** — Rust-grade safety *outcomes* without the borrow checker: value semantics
  removes use-after-free/data-races/double-free from the model, the language exposes no manual
  alloc/free (automatic deterministic reclamation — Perceus + region inference), the sole leak
  vector (external resources) is closed by the affine `Resource` kind, and any unsafe op is
  denied-by-default + lexically marked — *in safe Mycelium a memory leak is not expressible*. New
  open question Q8 (reclamation mechanism, cycle handling, `unsafe` spelling).

### Added
- **L1 grammar infrastructure + parser prototype** (`docs/spec/grammar/`, `scripts/checks/grammar.sh`,
  `crates/mycelium-l1`; RFC-0006 §4.3; **non-normative until RFC-0006 ratifies**): the WebAssembly-spec
  pattern (T3.1-B) made real. **`docs/spec/grammar/mycelium.ebnf`** — the normative v0 surface grammar
  in W3C notation (not ISO 14977), over the ratified DN-02 vocabulary (`colony`, `use`, `type`,
  `trait`, `fn`, `matured`, `let`/`in`, `if`, `match`, `swap`, `wild`, `spore`, `Substrate{…}`, the
  `T @ Strength` honesty index, representation-typed literals). **A conformance corpus** of 10
  `accept/` + 7 `reject/` `.myc` programs, each with an explanatory header — the corpus is the ground
  truth, not any single parser. **`grammar.sh`** (wired into `just check`/CI) structurally validates
  the artifacts; **`mycelium-l1`** is the real parser gate — a hand-written, dependency-free lexer +
  recursive-descent parser producing an inspectable AST, with `tests/conformance.rs` asserting every
  `accept/` parses and every `reject/` fails with an **explicit `ParseError` (never a panic, never a
  silent accept** — S5/G2). The lexer disambiguates the one tricky token (`<` opening a ternary
  literal vs a type-arg list) by lookahead; a malformed ternary literal is an explicit error. First
  increment of the L1 track (RFC-0006 §3) — typechecker, Maranget match compiler, structural totality
  checker, and L0 elaboration land next.
- **DN-02 (Resolved) — Fungal Lexicon & Reserved-Word Set** (`docs/notes/DN-02-Fungal-Lexicon-and-Reserved-Words.md`;
  feeds RFC-0006 §4.3): the surface vocabulary of Mycelium-the-language, drafted then **ratified by
  the maintainer** the same day. Codifies the **naming law** as a three-test gate (T-map fidelity /
  T-illuminate teaching-value / T-learn dual-readability) — *theme where the fungal metaphor is
  accurate and illuminating; keep conventional where a borrowed term is clearer to learn and read*.
  Ratified themed set: `colony` = module, `network` = the content-addressed dependency web,
  `substrate` = the affine external-resource kind, `spore` = reconstruction manifest (schema stays
  `reconstruction-manifest`), `matured` = promoted stable/AOT component, `wild` = the
  denied-by-default unsafe block. Ratified conventional: `let`, `fn`, `type`, `trait`, `match`,
  `if`, `swap` (a native corpus term), `use`, the guarantee tags; guarantee annotation `T @ Exact`.
  Literals universal-until-elaboration (no cross-family defaulting). Language name = **Mycelium**
  (shared). Status **Resolved** — the set is now frozen into the grammar artifacts.
- **Research Pass 3 — language-layer targets T3.1–T3.6** (`research/03-language-layer-RECORD.md`;
  grounds RFC-0006 Q1–Q6): four parallel primary-source deep-dives. Headlines: every surveyed
  kernel (GHC Core, Lean, Coq, Unison) keeps ~10–16 expression nodes with **data declarations in
  a registry/environment layer** and Unison gives the cycle-hashing recipe (T3.1); the guarantee
  lattice is formally an **integrity lattice** — silent upgrade = IFC's *endorsement*, gated here
  by a checked certificate — and graded coeffects (Granule-style) subsume flat labels, with
  refinements reserved for certificate side-conditions (T3.2); GHC levity polymorphism's two
  restrictions + monomorphization give the LR-5 restriction set (T3.3); divergence-only effect
  tracking (Koka's `div`, degenerate) + Lean's `partial`-opaque split + CakeML clocked semantics
  settle Q4/LR-4 (T3.4); ownership/borrowing confirmed **not applicable** to value semantics
  (Hylo/Swift), linearity deferred to a reserved affine `Resource` hook (T3.5); and the measured
  LLM evidence (MultiPL-E/T, MTOB, SynCode, grammar-aligned-decoding distortion) yields a
  five-condition KC-2 design with an explicit falsification threshold (T3.6). Honest-uncertainty
  register included; two pieces flagged **novel with no found precedent** (grading + runtime
  certificates; totality gating AOT promotion). **RFC-0006 revised to r2 (still Draft)**: §8
  positions per question, new Q7; §4.2 postures updated.
- **RFC-0006 (Draft) — Surface Language, Grammar & Term-Language Layering**
  (`docs/rfcs/RFC-0006-Surface-Language-and-Term-Layering.md`; SPEC §10.2's deferred "later RFC"):
  the deliberation artifact that nails down the language architecture *before* implementation
  accretes a de-facto one. Fixes now: the **L0–L3 layering** (Core IR → kernel calculus → surface
  term language → KC-2-gated projection layer; only L0/L1 trusted — KC-3), the **syntactic honesty
  invariants S1–S5** (never-silent swap stays lexically visible through every layer; guarantee
  tags are part of every binding's observable interface; content-addressed identity; inspectable
  elaboration; explicit partiality), the **capability targets LR-1…LR-8** ("Rust-class and beyond"
  made checkable: ADTs, coherent traits, content-addressed modules, totality-postured recursion,
  plus the beyond-Rust core — Repr polymorphism and guarantee-indexed types; ownership/borrowing
  flagged as likely-not-applicable to a value-semantics substrate), and the **grammar/spec
  discipline** (EBNF + machine-readable grammar artifacts + conformance corpus, mirroring the
  schema pattern). Defers exactly one thing, deliberately: the concrete L3 syntax, which the
  corpus already gates on the KC-2 experiment (M-002; RR-3). Status **Draft** — ratification is a
  maintainer decision. Indexed in `docs/rfcs/README.md`, `docs/Doc-Index.md`, SPEC §10.2.
- **Selection-policy language + mandatory EXPLAIN + site wiring** (`mycelium-select` — a new
  crate — plus the `mycelium-lsp` EXPLAIN channel, **M-220/M-221/M-222**, Phase 2; RFC-0005;
  ADR-006; SC-5): realizes RFC-0005 §2's decision verbatim. **M-220:** `SelectionPolicy` — an
  ordered decision table (`Predicate` over queryable `Meta`: dtype, guarantee, ε bounds, sparsity —
  *exact* metadata, never sampled estimates) over a finite `Candidate` set (`Repr` | `PackScheme`),
  with an explicit `CostModel` (cost = weight × storage **bits**, a real declared unit) and a
  mandatory default arm — total and terminating *by construction* (validated constructor; wire
  forms re-validated on deserialize); deterministic (first-match precedence; `Cheapest` ties break
  to lowest index); **content-addressed** (`policy_ref()` = hash of the canonical serialization —
  RFC-0005 §3); first-class deterministic overrides. **M-221:** every selection emits a
  serializable `Explanation` `{policy ref, inputs considered, cost of every candidate, matched
  rule, chosen, override state}`; `explain(policy, inputs)` is total and deterministic; the
  `mycelium-lsp` facade surfaces it as the fifth artifact kind (`analyze_with(node, &PolicyRegistry)`
  re-derives the trace at each resolvable swap site and raises a `policy-divergence` warning when
  the node's target disagrees with the policy's choice — surfaced, never silent). **M-222:** one
  mechanism, two sites — `select_swap_target`/`select_packing` are thin adapters over the single
  `select` (a wrong-kind candidate at a site is an explicit refusal); the wiring test drives an
  auto-selected target through the real interpreter + `CertifiedSwapEngine` and the result records
  `Meta.policy_used = PolicyRef` (the packing site is consumed by E2-7/M-250). 15 new tests across
  policy semantics, EXPLAIN, LSP surfacing, and the swap-site wiring.
- **KC-4 cert-overhead measurement + SC-3 global exit** (`xtask kc4` +
  `mycelium-cert/tests/sc3.rs`, **M-212**, Phase 2; Foundation KC-4; SC-3; RFC-0002 §2):
  `cargo run --release -p xtask -- kc4` times every implemented swap kind and its M-210
  certificate check (no bench dependency; refuses debug builds — their numbers would be dishonest
  to record). **Measured 2026-06-10** (containerized runner, indicative): bijective check ≈1.6–1.7 µs
  (~1.3× its ~1.3 µs swap — it re-derives the swap), bounded `Dense{768}` check ≈2.0 µs (~0.13× its
  ~16 µs swap), observational pair ≈10 ns. Honest verdict: per-swap checking costs the same order
  as the swap itself — the KC-4 downgrade path is **not triggered on this evidence**; a *ratified*
  numeric budget remains a pending maintainer decision (recorded in `phase-2.md` §6.7, not
  pre-written as "within budget"). The SC-3 global test pins the whole surface: every implemented
  legal-pair row emits a certificate that validates through the one checker, and every
  rejected/unimplemented row is an explicit error — never silent, anywhere.
- **First Bounded/lossy swap — Dense `F32 → BF16`** (`mycelium-cert::dense`, **M-211**, Phase 2;
  RFC-0002 §3/§5; ADR-010 §1): establishes the split regime (ADR-002) alongside the bijective
  binary↔ternary class. `dense_f32_to_bf16` rounds to-nearest-even and emits a
  `SwapCertificate::Bounded` carrying the proven per-element relative rounding bound
  `{Rel, u = 2^−8}` with a `ProvenThm` basis — the strength is *derived from how the bound was
  obtained, never asserted* (RFC-0002 §3), and the theorem's side-conditions are **checked per
  element**: finite, exactly an `f32`, zero-or-normal, no overflow on rounding; each violation is
  a typed explicit `SwapError` (`NonFinite`/`NotAnF32`/`SubnormalUnsupported`/`RoundOverflow`),
  never a silent coercion. Approximate sources are refused (`ApproximateSource`) until the E2-1
  composition rule exists — refusal, never fabrication. The certificate **validates through the
  M-210 shared checker**, a tampered conversion is caught (tier-i rejection), and a new
  `CertifiedSwapEngine` serves the complete certified surface (bijective + bounded + identity),
  explicit `UnsupportedSwap` for everything else. 11 tests incl. a 20k-sweep soundness property
  for the `2^−8` bound and ties-to-even spot checks.
- **Single shared translation-validation certificate checker** (`mycelium-cert::check`, **M-210**,
  Phase 2; RFC-0002 §2; RFC-0004 §3; T1.1): one `check(A, B, R, claimed, evidence)` answering "does
  artifact B refine reference A under relation R within the claimed `{ε,δ,strength}`?" — build once,
  use twice. Three `RefinementRelation` instances: **Bijection** (the M-120 binary↔ternary cert —
  lemma reference + `legal_pair` side-condition checked, then structural *re-derivation equality*
  against B), **BoundedSimilarity** (lossy swaps — the measured A↔B deviation and the claim are both
  re-validated through the E2-4 `mycelium-numerics` tier-i checker; a claim tighter than its
  certificate, a certificate tighter than the measured instance, or a strength upgrade past the
  basis (VR-5) is rejected), and **ObservationalEquiv** (interp↔AOT over the NFR-7 observable —
  the **M-151 differential is folded in** as an instance and now validates every corpus pair
  through this checker). TV incompleteness is an explicit `NotValidated{reason, fallback}` with the
  `UseReference` fallback path — **never a silent pass** (RFC-0002 §2). `mycelium-numerics` now
  exports `basis_strength` (the M-I2…M-I4 basis→strength mapping) for certificate consumers.
  16 checker tests cover all three instances and every refusal path.
- **Interpreter composes approximate inputs honestly** (`mycelium-interp::prims`, **M-204**, Phase 2;
  RFC-0001 §4.7; ADR-010): retires the Phase-1 blanket `ApproxCompositionUnsupported` refusal for
  composable inputs. `exact_result` → `compose_result`: exact-over-exact stays `Exact`/`bound=None`
  (M-I1); over an approximate input it composes per a per-prim `ApproxRule` — `core.id` passes the
  bound through verbatim (citation preserved), `trit.add`/`sub`/`neg` carry the sound affine ε
  composition via `mycelium_numerics::compose_error_bound` (strength `meet`s to the weakest input,
  basis re-derived so M-I2…M-I4 hold), and `bit.*` / `trit.mul` still refuse (no defined ε rule —
  honest, never a fabricated bound). Five new golden tests cover additive ε composition (Proven⊕Proven
  → Proven, ε sums), negation (ε preserved), `core.id` passthrough, meet-down to Declared, and the
  explicit `trit.mul` refusal; the Phase-1 `bit.not` refusal test still holds. **Closes the documented
  Phase-1 honesty gap** (the interpreter previously could not compose approximate inputs).
- **Verified-numerics foundation — two bound kernels + shared certificate + tier-i checker**
  (`mycelium-numerics`, **M-201/M-202/M-203**, Phase 2; ADR-010; RFC-0001 §4.7; SPEC §10.7): a new
  crate realizing ADR-010's two-kernels-one-certificate decision, deliberately *outside*
  `mycelium-core` (KC-3/SoC — the trusted kernel stays small; numerics is a certificate consumer).
  **`error`** composes ε through **affine arithmetic** — `AffineForm` (`x₀ + Σxᵢ·εᵢ`) with *exact*
  linear ops (correlated noise symbols cancel) and a sound `mul` (second-order remainder onto a fresh
  symbol), and the scalar `ErrorBound{eps,norm}` projection (`add`/`sub`/`neg`/`scale`/`mul`).
  **`prob`** composes δ through the **union bound** (`min(1,Σδ)`) and the apRHL `[SEQ]` rule
  (`ApRhlJudgment` — ε adds as the `e^ε` factors multiply, δ adds, both saturating). They meet at the
  shared **`Certificate{eps,delta,strength}`** (`strength` by `meet`), with a **tier-i Rust checker**
  (`check_error_claim`/`check_union_claim`) that re-derives a composition and **rejects any claim
  tighter than the re-derivation** — never a silent pass (RFC-0002 §2) — and the one sanctioned
  cross-kernel rule `accuracy_to_probability` (ADR-010 §4). The three normative properties
  (**Soundness, Monotonicity, Determinism**; RFC-0001 §4.7) are property-tested over 20k-trial inline
  loops (Phase-1 house style — no `proptest`/`rand` dep); 17 tests green, clippy `-D warnings` clean.
- **Phase-2 plan + epic decomposition** (`docs/planning/phase-2.md`; **Phase 2**; Foundation §6;
  SPEC §10.7–§10.10): decomposed the seven Phase-2 epics (#28–#34) into 18 issue-coupled `M-2xx`
  build tasks (#48–#65), created as sub-issues of their epics and joined into `tools/github/idmap.tsv`.
  The plan mirrors `phase-1.md`: readiness table, batch/parallelization structure, the critical path
  (the ADR-010 ε/δ numerics kernels as keystone — they gate every honest approximation downstream),
  and an honest Phase-1→2 re-run of the kill criteria (KC-1 confirmed/no-regression; KC-2
  open/blocked on external LLM access; KC-3 holding — numerics + selection land as their own crates
  to keep the kernel auditable; KC-4 first-measurable when the shared checker lands). Planning
  artifact only — cites the corpus, introduces no requirements.
- **MLIR→LLVM AOT path — ternary-dialect skeleton + runnable AOT artifact** (`mycelium-mlir`,
  **M-150**, Phase 1; RFC-0004 §2/§6; ADR-007; T1.5): `dialect::emit` renders the lowered A-normal
  form as a textual `ternary`-dialect MLIR-style module (one op per binding, all attributes inline —
  the no-opaque-pass anchor), and `aot::run` is the **runnable artifact for the subset** — an
  independent big-step env-machine that executes the lowered ANF directly. Native libMLIR/LLVM
  codegen is **deferred** (Phase 3 matures it; honestly scoped as a textual skeleton + execution
  model, not a compiler).
- **Interp↔AOT differential** (`mycelium-mlir` tests, **M-151**, Phase 1; NFR-7; VR-4; RR-12): a
  harness runs a kernel corpus under both the M-110 reference interpreter (small-step substitution)
  and the M-150 AOT artifact (big-step env-machine over the lowered ANF) and asserts **observable
  equivalence** (repr + payload + guarantee); divergence fails CI. The two paths differ in IR shape
  and evaluation strategy, sharing only the trusted primitive/swap semantics — so the differential
  catches lowering/scheduling/ordering divergence (the cheap baseline preceding per-artifact
  translation validation in Phase 2). A control test confirms the harness discriminates.
- **LSP feedback facade** (`mycelium-lsp::feedback`, **M-140**, Phase 1; FR-S5; Foundation §5.8;
  SC-5): `analyze(node)` exposes the **four** semantic-feedback artifact kinds over one surface —
  (1) typecheck/invariant **diagnostics** (linter), (2) **swap certificates** for statically-
  resolvable swap sites, (3) per-value **bound/guarantee annotations**, (4) **lowering-stage dumps**.
  A failed/unsupported swap is surfaced on the diagnostics channel, never silent. Verified by a
  **scripted-client** integration test driving all four channels (incl. a Proven bound, an
  out-of-range swap, and invariant violations).
- **Canonical formatter** (`mycelium-core::lower::format` + `mycelium-lsp::fmt`, **M-142**, Phase 1;
  RFC-0001 §4.8; ADR-003): a canonical textual normal form that **α-normalizes binder names**
  (`v0, v1, …`), so definitions differing only in names render to identical text and share one
  `content_hash` — reformatting is a projection that never changes content-addressed identity (tested:
  renamed defs format identically and hash equally; formatting leaves identity untouched; free
  variables keep their names).
- **Invariant linter** (`mycelium-lsp::lint`, **M-141**, Phase 1; SC-3; G2; FR-M3; VR-5): static,
  inspectable lints over a Core IR program, emitted as `Diagnostic`s for authoring tools — `implicit-swap`
  (an `Op` mixing paradigms implies a conversion that must be an explicit `Swap`), `unverified-bound`
  (a `Declared` value must always be surfaced, never silently trusted), `placeholder-policy` (a swap
  citing a stub rather than a real `PolicyRef`), and `free-variable` (an open term). Each lint has a
  positive and a negative test. Introduces the toolchain crate `mycelium-lsp` (FR-S5), kept out of
  the auditable kernel (KC-3 — depends on core/interp/cert, nothing depends on it).
- **Inspectable lowering — ≥2 dumpable/diffable stages** (`mycelium-core::lower`, **M-112**, Phase 1;
  RFC-0004 §5/§6; SC-4; WF5): a backend-agnostic lowering pipeline. `stages(node)` returns **`core`**
  (the canonical Core IR tree dump) → **`substrate`** (an A-normal form flattening nested
  `Op`/`Swap`/`Let` to a linear binding list — the pre-codegen shape backends consume), each binding
  whose result repr is statically known (`Const`, `Swap` target) annotated with its **scheduled
  `PhysicalLayout`** (the default schedule, `I2_S` for ternary; RFC-0004 §5 / DN-01). Dumps are
  canonical (deterministic — structurally identical programs render identically, SC-4) and `Meta`
  guarantee tags survive lowering (WF5). `Op`-result layout is left explicitly unannotated (no
  operator typing yet — the omission is honest, not silent; G2).
- **Cleanup / item memory** (`mycelium-vsa::cleanup`, **M-132**, Phase 1; FR-S4; RFC-0003 §3): a
  labelled associative memory (`CleanupMemory`) that snaps a noisy query — an *approximate* `unbind`
  result or a `bundle` decode — to the nearest stored atom by similarity, returning a `Match { label,
  index, confidence, margin }`. The confidence (match cosine) and margin (gap to the runner-up) make
  approximate unbind *usable* and *inspectable* (the retrieval decision is reported, never a hidden
  nearest-neighbour pick; G2). Tested incl. the role⊗filler record-decode use case (bundle two bound
  pairs, unbind by a role, clean up to the right filler).
- **MAP-I bundle capacity bound — `Proven` via checked instantiation** (`mycelium-vsa::capacity`,
  **M-131**, Phase 1; RFC-0003 §5; ADR-010; SC-2; KC-1): `required_dim(m, δ) = ⌈(2/μ²)·ln(m/δ)⌉`
  (μ=0.1) and `proven_capacity_bound` / `MapI::bundle_values_certified`, which attach a **`Proven`**
  `CapacityBound` (basis `ProvenThm`, citing Clarkson-Ubaru-Yang 2023 / Thomas-Dasgupta-Rosing 2021)
  **iff** the checked side-condition `dim ≥ required_dim` holds — exactly the M-001 axiomatized-
  theorem + checked-instantiation pattern (the formula is cited, not re-proven). An undersized
  dimension returns an explicit `InsufficientCapacity` error rather than an unbacked `Proven` tag
  (M-I2/VR-5). `required_dim` reproduces the four M-001 probe settings (1141/1843/2164/2764).
  **Acceptance — ≥10⁴-trial empirical validation (SC-2):** over 10,000 independent trials at
  `dim ≥ required_dim(3, 1e-2)`, the measured nearest-neighbour retrieval-failure rate stays `≤ δ`.
- **VSA submodule — `VsaModel` trait + MAP-I** (`mycelium-vsa`, **M-130**, Phase 1; RFC-0003 §3–§4;
  ADR-008; T2.6): a composition-style `VsaModel` trait (`bind`/`unbind` + self-inverse flag,
  `bundle`, `permute`/`unpermute`, `similarity`, and the honest per-op intrinsic guarantee) and its
  first model **MAP-I** — `bind`/`unbind` are self-inverse and **`Exact`** (elementwise product),
  `permute` is **`Exact`** (cyclic shift), `bundle` is elementwise superposition. Value-level
  adapters for the Exact ops carry honest `Derived` provenance. **Dependency-gated** (ADR-008): the
  crate depends on `mycelium-core` but the kernel does not depend on it — VSA values stay
  type-checkable in the kernel without pulling in this algebra (KC-3). Tests: bind/unbind round-trip
  exactly, permute is invertible/cyclic, a bundle is far more similar to its members than to a
  stranger, dim-mismatch/empty-bundle are explicit errors. The `bundle` **`Proven`** capacity bound
  (M-I2: a *value*-level Proven bound needs a checked basis) is deferred to **M-131** — not stamped
  here (VR-5).
- **Binary↔ternary certified swap** (`mycelium-cert` + `mycelium-core::binary`, **M-120**, Phase 1;
  RFC-0002 §3/§4): `enc`/`dec` per `docs/spec/swaps/binary-ternary.md` over a legal `(n, m)` pair,
  emitting a `SwapCertificate::Bijective` (`LosslessWithinRange`) that references the once-per-pair
  round-trip lemma (`lemma_ref`) bound by concrete `params`. `enc` is total on `B_n`; `dec` is the
  **partial** inverse — a value outside the binary range is an explicit `SwapError::OutOfRange`
  (P4), an illegal pair is a **type error** (`IllegalPair`, RFC-0002 §5), never a `Declared` gamble.
  Within range the result is `Exact`/`bound = None` (P3, M-I1) and records `policy_used` + `Derived`
  provenance. A `BinaryTernarySwapEngine` plugs the swap into the M-110 interpreter. **Acceptance —
  `dec(enc x) = Some x` exhaustively over all 256 bytes** (8↔6, SC-1); serializer output pinned to a
  committed `swap-certificate` example validated against the schema in CI (SC-3). Adds a
  two's-complement codec `mycelium-core::binary` (exhaustively round-trip-tested).
- **Binary↔ternary round-trip proof** (`proofs/binary-ternary-roundtrip/`, **M-121**, Phase 1;
  VR-1/SC-1): the SMT-LIB2 injectivity obligation for the 8↔6 pair — **discharged by Z3 4.16.0
  (`unsat`)**: no two distinct 6-trit vectors collide ⟹ the value map is a bijection onto
  `[−364, 364] ⊇ B_8` ⟹ `dec(enc b) = b` (P1/P2). Wired into `scripts/checks/proofs.sh`
  (skip-graceful without z3); the lemma identity matches `mycelium_cert::roundtrip_lemma_ref()`. P3/P4
  are additionally decided by the M-120 exhaustive Rust corpus. (The fixed `8↔6` instance; a
  width-generic proof is future work — each legal pair gets its own discharged lemma.)
- **Balanced-ternary arithmetic** (`mycelium-core::ternary` + `mycelium-interp`, **M-111**, Phase 1;
  FR-M2): the single home for the balanced-ternary integer codec (`int ↔ trits`, MSB-first, the
  §3.1 digit-extraction algorithm) and fixed-width digit-wise arithmetic — `neg` (digit-wise sign
  flip = value negation), ripple-carry `add`/`sub`, and shifted-add `mul`. Out-of-range results are
  an explicit `None`/`EvalError::Overflow`, **never** a silent wrap (SC-3). The interpreter gains
  `trit.neg/add/sub/mul` primitives over it. **Acceptance — property-tested vs an `i64` oracle by
  exhaustion** over all operand pairs at widths `m ≤ 4` (and the codec round-trip/neg at `m ≤ 5`):
  in range the digit-wise result equals the encoded integer result, out of range it overflows.
  Grounded in `docs/spec/swaps/binary-ternary.md` §1/§3.1; reused by the M-120 swap.
- **Reference interpreter** (`mycelium-interp`, **M-110**, Phase 1): the trusted, executable
  **small-step operational semantics** for the Core IR, closing SPEC §10.3 (RFC-0004 §2; ADR-009;
  NFR-7). Call-by-value substitution over closed `Node`s with the rules E-Let-Bind/Step,
  E-Op-Arg/Apply, E-Swap-Arg/Apply (documented in the crate). An extensible **primitive registry**
  (`PrimRegistry`) ships the exact elementwise built-ins (`core.id`, `bit.not/and/or/xor`,
  `trit.neg`); a **`SwapEngine`** hook ships the trivial same-`Repr` `IdentitySwapEngine`. Results
  thread metadata honestly — guarantee by `meet` (RFC-0001 §4.7), provenance `Derived{op, inputs}`
  over content hashes (§4.6), `policy_used` on swaps. **Never silent**: free variables, unknown/
  ill-typed prims, unsupported cross-paradigm swaps, approximate-input composition (no bound kernel
  yet — ADR-010/E2-4), and fuel exhaustion are all explicit `EvalError`s. 20-case golden corpus.
  Adds `mycelium_core::operation_hash` (provenance op identity for prims). Scope boundary:
  balanced-ternary arithmetic + oracle property tests are **M-111**; the certified binary↔ternary
  swap + proof are **M-120/M-121**.
- **Guarantee `meet`-composition** (`mycelium-core::guarantee`, **M-102**, Phase 1):
  `GuaranteeStrength::meet` (the weakest-wins greatest-lower-bound) plus `propagate`/`meet_all` for
  the RFC-0001 §4.7 rule `guarantee(result) = meet(inputs…, g_f)`, and `TOP`/`ALL` constants. The
  meet-semilattice laws — commutativity, associativity, idempotence, identity `Exact`, `Declared`
  absorbing — are verified by **exhaustion** over all 4×4(×4) tuples (complete for the finite
  lattice, not sampled). Honesty can only degrade, never spuriously upgrade (VR-3/VR-5).
- **Content-addressing** (`mycelium-core::content`, **M-103**, Phase 1): `Node::content_hash` /
  `Value::content_hash` — a BLAKE3 hash over an injective, domain-separated, length-prefixed
  encoding of the *identity-bearing* content: the α-normalized structure (bound vars as de Bruijn
  indices, binder names dropped), types-with-`Repr`, constant literals, operator names, and swap
  target+policy. Dynamic `Meta` (provenance, bounds, sparsity, `policy_used`) is excluded. Adds a
  separable `hash ↔ name` table (`Names`) for names-as-metadata, `ScalarKind::tag`, and
  `ContentHash::from_parts`/`algo`/`digest`. Acceptance met: identical defs collide; trivial (α)
  renames don't change identity; a paradigm/precision/literal/operator change does (RFC-0001 §4.6;
  ADR-003).
- **Core IR (de)serialization** (`mycelium-core`, **M-104**, Phase 1): `serde`
  `Serialize`/`Deserialize` for `Value`/`Meta`/`Repr`/`Bound`/`Provenance`/… emitting *exactly* the
  ratified JSON data contracts (`kind`/`class`/`layout` tags; `VSA`/`BF16`/`TL1`/`TL2` renames;
  `payload` as `{bits|trits|scalars|hypervector}` with MSB-first bit/trit strings; `bound` modelled
  by presence; flat `kind`+`basis` `Bound`). `Deserialize` routes `Value`/`Meta` through their
  checked constructors, so M-I1…M-I4 and payload↔repr mismatches are rejected on the wire — never
  silently accepted. Faithful round-trip (`deserialize(serialize(v)) == v` incl. `Meta`) is tested
  over a corpus spanning all four paradigms × every guarantee/bound/basis/layout; serializer output
  is pinned to three new committed `value` examples (ternary/dense/vsa) that `scripts/checks/schema.sh`
  validates against `value.schema.json` in CI (RFC-0001 §4.8).
- **Core IR data structures** (`mycelium-core`, **M-101**, Phase 1): Rust types mirroring the
  ratified schemas — `Repr`/`ScalarKind`/`SparsityClass`, the `GuaranteeStrength` lattice,
  `Bound`/`BoundBasis`/`BoundKind`/`NormKind` (ADR-011: `basis` universal), `Meta` (with
  `Provenance`, `SparsityObs`, `PhysicalLayout`/`PackScheme`), `Value`/`Payload`, `ContentHash`,
  and the `Node` grammar (closes the core of SPEC §10.2; RFC-0001 §4.5). The honesty invariants
  **M-I1…M-I4** and payload↔repr/repr well-formedness are enforced **by construction**
  (`Meta::new`, `Value::new` → `WfError`). 17 unit tests; `fmt`/`clippy -D warnings`/`test` green on
  MSRV 1.92.
- **Minimal surface-syntax fragment** (`experiments/surface-fragment/`, **M-020**): a throwaway,
  experiment-only concrete syntax (EBNF + desugaring to the Core IR nodes + 3 reference programs:
  swap round-trip, VSA `bundle`, and a no-implicit-conversion type-error) to feed the KC-2
  experiment. **Not** a committed surface — gated on KC-2 (hence under `experiments/`, not
  `docs/spec/`). Linked from `SPECIFICATION.md` §10.1.
- **Binary↔ternary encoding spec** (`docs/spec/swaps/binary-ternary.md`, **M-012**): precise
  `enc`/`dec` for the canonical `8↔6` width — balanced-ternary digit semantics, the legality
  condition `B_n ⊆ T_m`, `LosslessWithinRange` with an `Option`-typed (never-silent) inverse, the
  four M-121 correctness obligations, and a worked round-trip + out-of-range example (RFC-0002
  §4/§5; T2.1). Linked from `SPECIFICATION.md` §6/§10.4.
- **Python tooling skeleton** (`experiments/`, **M-092**): a UV-managed project targeting
  **Python 3.13** (ADR-007) with a `dev` group (pytest, pytest-cov, ruff, black), a trivial
  importable module + passing smoke test, and a committed `uv.lock`. `scripts/checks/test.sh` runs
  it via `uv run --frozen pytest` under the pinned interpreter, so it joins the `just check`/CI
  suite (skip-graceful when uv is absent).
- **Rust workspace skeleton** (**M-091**): a 6-crate Cargo workspace (`mycelium-core`,
  `mycelium-interp`, `mycelium-vsa`, `mycelium-mlir` stub, `mycelium-cert` stub, `xtask`) with
  **MSRV pinned to 1.92** via `rust-toolchain.toml` + `rust-version` (ADR-007), workspace lints
  (`unsafe_code = forbid`, clippy warn), and a smoke test per crate. `cargo fmt --check`,
  `clippy -D warnings`, and `cargo test` are all green on 1.92. Adds `scripts/checks/test.sh` +
  `just test`, wired into the `just check`/CI suite (skip-graceful when a toolchain is absent), so
  test parity now holds local↔CI. Fixes a malformed `Cargo.lock` line in `.gitignore`.
- **M-001 probe scaffold** (`proofs/lh-bundle/`): the Liquid-Haskell MAP-I `bundle`
  capacity-refinement module + cabal project + writeup, encoding the axiomatized-theorem +
  checked-instantiation strategy with ≥3 concrete `(d,m,δ)` settings (RFC-0003 §5; T0.2). **Not yet
  discharged** — no GHC/LH/Z3 in this environment — so KC-1 stays `passed (literature)`; the
  derivation table is the independently-checkable artifact. Establishes `proofs/<name>/` as the
  home for machine-checkable proofs (resolves OQ-2).
- **`SPECIFICATION.md` skeleton** (`docs/spec/SPECIFICATION.md`, **M-011**): the consolidation index
  over the corpus — §1–§9 reconciled to RFC-0001 (r2)/RFC-0002…0005/ADR-010/011/DN-01 and pointed at
  the ratified `docs/spec/schemas/` contracts; §10 enumerates the open build items, each linked to a
  live issue (no floating TODOs). Status `consolidating-draft → ratified-skeleton`.
- **ADR-011 — `BoundBasis` is a property of every `Bound`** (`docs/adr/ADR-011-...md`, Accepted):
  formally supersedes the implicit RFC-0001 r1 §4.3 decision that scoped `basis` to `CapacityBound`
  only, so every approximate value (ε, δ, crosstalk, capacity) honestly records how its bound was
  obtained (VR-5, G5). Resolves OQ-3.
- **Core data-contract schemas** (`docs/spec/schemas/`, **M-010**): the 10 ratified JSON Schemas
  (draft 2020-12) — `repr`, `value`, `meta`, `guarantee`, `bound`, `provenance`,
  `physical-layout`, `swap-certificate`, `policy`, `reconstruction-manifest` — each a faithful
  projection of its source RFC/ADR section, plus ≥1 valid and ≥1 invalid example per schema (the
  invalids exercise the honesty-load-bearing invariants M-I1/M-I4). `just schema` validates the
  set in CI. The OQ-3/OQ-4/OQ-5 clarifications surfaced here are now resolved (see below /
  `docs/spec/schemas/README.md`).
- **Phase-0 working plan** (`docs/planning/phase-0.md`): the first issue-coupled expansion of
  Foundation §6, mapping the nine Phase-0 tasks (M-001/002/010/011/012/020/090/091/092) to their
  GitHub issues, the critical path, honest KC-1/KC-2 gate status, the proposed canonical
  data-contract schema set, and the author-then-ratify reframing for M-010/M-011 (the
  `docs/spec/` artifacts they ratify do not exist yet).
- Initial **design baseline**: project charter (`docs/Mycelium_Project_Foundation.md`, r3),
  document index (`docs/Doc-Index.md`), five RFCs (RFC-0001…0005, all Accepted),
  ADR-010 (Accepted), design note DN-01 (Resolved), and two research records
  (`research/01`, `research/02`).
- Repository scaffolding: `README.md`, `LICENSE` (MIT), `CONTRIBUTING.md`,
  `.gitignore` (Rust + Python), and index/process READMEs for `docs/adr/` and `docs/rfcs/`.
- **GitHub PM bootstrap** (`tools/github/`): `issues.yaml` / `labels.json` / `milestones.json`,
  the `mcp-bootstrap.md` runner + `gh-bootstrap-local.sh`, the `project-v2-spec.md` board spec,
  and the `idmap.tsv` task→issue map.
- **Agent tooling**: `CLAUDE.md` and `.claude/skills/` (`pr-review`, `security-review`,
  `dev-workflow`, `docs-review`, `changelog`) operationalizing the `CONTRIBUTING.md` house rules.
- **Local check tooling** with local↔CI parity: `justfile` + `scripts/checks/*` (markdownlint,
  offline link/cross-reference, json-schema, codespell, shellcheck, secret scan, fmt/lint),
  `.pre-commit-config.yaml`, and a manual-dispatch **advisory** GitHub Actions workflow.

### Changed
- **Proofs wired into the check suite** (`scripts/checks/proofs.sh` + `just proofs`): runs the
  LiquidHaskell `bundle` probe (`LC_ALL=C.UTF-8 cabal build`, a green build ⟺ LH `SAFE`),
  skip-graceful when GHC/cabal/z3 are absent. Added to `just check`/`just ci`; the manual-dispatch CI
  workflow now sets up GHC 9.8.2 + cabal + z3 (with a cabal/dist-newstyle cache) so the proof
  verifies on a manual run. (Whole suite remains `workflow_dispatch`-only.)
- **KC-1 confirmed (build)** (**M-001**): the Liquid-Haskell MAP-I `bundle` capacity refinement
  (`proofs/lh-bundle/`) type-checks **`SAFE` (16 constraints)** and Z3 discharged all four `(d,m,δ)`
  instantiations (GHC 9.8.2 · LiquidHaskell 0.9.8.2 · Z3 4.8.12), ratifying the axiomatized-theorem +
  checked-instantiation strategy (RFC-0003 §5; ADR-010). KC-1 moves `passed (literature) → confirmed
  (build)` in the Foundation §2.4 and Doc-Index §3/§4. (The Clarkson/Thomas theorem remains cited,
  not re-proven — by design.) Haskell build output (`dist-newstyle/`, `.liquid/`) gitignored;
  codespell skips them.
- **Docs/parity CI hardened** (`.github/workflows/checks.yml`, **M-090**): the manual-dispatch
  advisory workflow now sets up **uv** (so the `experiments/` Python 3.13 tests actually run) and
  **Rust** (pinned via `rust-toolchain.toml`, so fmt/clippy/test run), and adds an advisory
  **Codecov** upload of the experiments coverage. Markdown-lint + offline link-check + schema
  validation already covered `docs/**` and the schemas via `just ci`; the PR template was already
  wired. Posture unchanged: `workflow_dispatch` only, non-blocking (no auto-triggers — CLAUDE.md).
- **RFC-0001 → r2** (status stays Accepted): §4.3 `Bound` grammar revised per **ADR-011** —
  `BoundBasis` factored out to a required companion of *every* `Bound` (was: `CapacityBound` only),
  and `NormKind` enumerated `L1|L2|Linf|Rel` as an extensible registry (resolves OQ-4). The r1 §4.3
  grammar is formally superseded; indexes (`Doc-Index.md`, `docs/rfcs/README.md`,
  `docs/adr/README.md`) and the `bound` schema updated to match.

### Changed (baseline-review consistency pass)
- ADR-001 promoted to firmly **Accepted**; the "no statistical approximation vs
  fully-disclosed approximation" definitional question recorded as **settled**
  (fully-disclosed), consistent with the KC-1 pass and the guarantee lattice.
- Foundation §5.2 core-model sketch marked **superseded by RFC-0001** (packing is
  now schedule-staged, not in the type; guarantee lattice is the four-point form).
- Foundation §5.6 updated: **MLIR→LLVM** recorded as the committed AOT path
  (ADR-007 / RFC-0004), not a candidate.
- Foundation §6 Phase 0 annotated with post-research status (largely complete;
  remaining: the Liquid-Haskell `bundle` probe and the KC-2 LLM-leverage experiment).
- `README.md` decisions table: fixed a placeholder reference for the
  "no implicit conversion" rule (grounded in RFC-0001 §3.3 / FR-M3).
- `docs/Doc-Index.md`: the two research rows now point to the in-repo records.

### Fixed
- Markdown hygiene surfaced by the new check tooling: normalized emphasis to the corpus
  asterisk style and added a missing trailing newline (`README.md`,
  `research/01-prior-art-survey-RECORD.md`, `docs/notes/DN-01-Packing-Placement-Tradeoffs.md`).
- Copilot PR-review findings (PR #1, #42) addressed: corrected the binary↔ternary swap's partial
  right-inverse in RFC-0002 §4 (`dec y = Some x ⟹ enc x = y`; the prior `enc y = …` was a type
  error since `enc : Bin_n → Tern_m`); resolved a P0.3 status contradiction in the Foundation
  Meta section (P0.3 is already resolved per §6); corrected stale references in `tools/github/`
  (`gh-bootstrap.sh`, `docs/planning/*`, `project-v2-spec.md`); `gh-bootstrap-local.sh` now
  honors each milestone's `state` instead of hardcoding `open`.
- Tooling self-lint: `scripts/*` made shellcheck/ruff/markdownlint-clean (cd-failure guards,
  if/then/else over `A && B || C`, split imports, fenced-block spacing).

### Security
- `.gitleaks.toml`: removed an allowlist **regex** (`AKIA[0-9A-Z]{16}`) that exempted the AWS
  access-key-ID *pattern* from scanning — it would have suppressed detection of a real leaked
  key. The path allowlist is retained; pattern-level allowlisting is documented as forbidden.

### Open
- One confirming build: the Liquid-Haskell `bundle` capacity-refinement probe (RFC-0003 §5).
- One existential question: **KC-2 / LLM leverage** (the E4 experiment) — not yet settled.
- Decomposed task/issue set and phase planning documents — *forthcoming* (`docs/planning/`).
