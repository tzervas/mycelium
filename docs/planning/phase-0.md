# Phase 0 — Confirm & Specify (working plan)

| Field | Value |
|---|---|
| **Status** | **Living draft** (initial cut, 2026-06-09) |
| **Owns** | the concrete, issue-coupled expansion of Foundation §6 "Phase 0 — Feasibility probes & foundational specs" |
| **Source of truth above this doc** | `docs/Mycelium_Project_Foundation.md` §6 (roadmap), `tools/github/issues.yaml` (task ids), RFC-0001…0005 + ADR-010 + DN-01 (design corpus, all Accepted/Resolved) |
| **Mirrors** | the GitHub board: every task here carries its issue number from `tools/github/idmap.tsv` |
| **Companion docs** | `phase-1.md` … `phase-3.md` (forthcoming; decomposed when each phase gate opens) |

> **Grounding discipline.** This is a planning artifact, not a normative one. It cites the
> corpus (`FR/NFR/VR/SC/KC`, `RFC-xxxx §`, `ADR-0xx`, `Tx.y`, `G#`) for every claim about *what*
> is required; it does not introduce new requirements. Where it records a *decision about
> sequencing or scope* it says so explicitly and routes anything normative back to an RFC/ADR.
> Honesty rule applies to the gate verdicts below (KC-1/KC-2): they stay `passed (literature)` /
> `open` until a *checked build* moves them — see §5.

---

## 1. What Phase 0 is for

Phase 0 ("Confirm & Specify") is the bridge between a finalized **design** corpus and the first
**build** (Phase 1, the minimal viable core). Its job is narrow and high-leverage:

1. **Confirm** the one design bet the research left as "passed on the literature, not yet built":
   the cited-theorem + checked-instantiation strategy for honest VSA bounds (**KC-1**, the
   Liquid-Haskell `bundle` probe — M-001).
2. **Specify** the machine-readable contracts the build will compile against: the JSON data
   contracts (M-010) and the consolidated `SPECIFICATION.md` skeleton (M-011) that ties §1–§9 of
   the spec to the RFCs and §10 TODOs to live issues.
3. **Scaffold** the toolchains the build needs — the Rust workspace (M-091), the Python
   experiments project (M-092), and docs CI (M-090) — so Phase 1 starts green.
4. **Resolve** the one existential question the research did *not* settle: the LLM-leverage
   verdict (**KC-2**, M-002), on the throwaway surface fragment (M-020).

Foundation §6 records Phase 0 as **"largely complete"** at the *design* level: both research
passes are done, KC-1 passed on the literature, and RFC-0001…0005 / ADR-010 / DN-01 are Accepted.
What remains is exactly the **confirm + specify + scaffold** residue enumerated above. This doc is
the dependency-ordered, issue-coupled tracking of that residue.

### Phase-0 exit gate (what "done" means)

Phase 0 closes when **all** of:

- **KC-1 = confirmed (build)** — M-001 LH probe type-checks and Z3 discharges ≥3 settings (§5).
- **KC-2 = a written verdict** — M-002 produces a SC-5b baseline number and a proceed /
  reweight-to-human / fall-back-to-embedded-DSL decision (§5). (A *verdict*, not a pass — the
  acceptable outcomes include "fall back".)
- **Data contracts ratified** — M-010: the canonical schema set validates in CI (valid + invalid
  examples), status `draft → ratified`.
- **Spec skeleton ratified** — M-011: `SPECIFICATION.md` §1–§9 reconciled to the RFCs, every §10
  TODO linked to a live issue; status `consolidating-draft → ratified-skeleton`.
- **Toolchains green** — M-090/M-091/M-092 each pass their own check on an empty/no-op change.

Maps to Foundation §6 Phase-0 "Success metrics": SC-1 (P0.4 → carried to M-012/M-120 in Phase 1),
SC-2 partial (P0.1 → M-001), an LLM baseline X for SC-5b (P0.2 → M-002), written KC-1/KC-2
verdicts, and an agreed execution-model/toolchain spec sketch (P0.6 → folded into M-011's
`SPECIFICATION.md`).

---

## 2. The Phase-0 task set (readiness at a glance)

All nine Phase-0 issues, with issue number (`idmap.tsv`), priority, dependency, and **build
readiness** — i.e. whether the artifact the issue acts on actually exists yet. This is the most
important column: see §3 for the gap it exposes.

| Task | Issue | Pri | Depends on | Maps to | Artifact exists? | Readiness |
|---|---|---|---|---|---|---|
| **M-010** Ratify core data-contract schemas | [#5](https://github.com/tzervas/mycelium/issues/5) | P0 | — | P0.6 / RFC-0001–0003 | **No** — `docs/spec/schemas/` is empty | **Author-then-ratify** (§3, §6.1) |
| **M-011** Ratify `SPECIFICATION.md` | [#6](https://github.com/tzervas/mycelium/issues/6) | P1 | M-010 | P0.6 | **No** — `docs/spec/SPECIFICATION.md` absent | **Author-then-ratify**, gated on M-010 (§6.3) |
| **M-001** LH `bundle` capacity probe | [#2](https://github.com/tzervas/mycelium/issues/2) | P0 | M-010 | P0.1 / KC-1 | n/a (new build) | **Ready** — narrow dep on `CapacityBound` shape (§6.2) |
| **M-012** Binary↔ternary encoding spec | [#7](https://github.com/tzervas/mycelium/issues/7) | P1 | — | P0.4 / RFC-0002 §4 | n/a (new spec) | **Ready** — pure spec authoring |
| **M-020** Minimal surface fragment | [#4](https://github.com/tzervas/mycelium/issues/4) | P0 | — | P0.2 / SPEC §10.1 | n/a (throwaway) | **Ready**, but value-gated on M-002 design |
| **M-002** KC-2 LLM-leverage experiment | [#3](https://github.com/tzervas/mycelium/issues/3) | P0 | M-020, M-010 | P0.2 / KC-2 | n/a (experiment) | **Blocked** on M-020 + M-010 |
| **M-090** Docs CI | [#8](https://github.com/tzervas/mycelium/issues/8) | P1 | — | infra | partial — `checks.yml` exists, advisory | **Mostly done** (§6.5) |
| **M-091** Rust workspace skeleton | [#9](https://github.com/tzervas/mycelium/issues/9) | P0 | — | infra / ADR-007 | **No** — no `Cargo.toml` yet | **Ready** — pure scaffolding (§6.4) |
| **M-092** Python tooling skeleton | [#10](https://github.com/tzervas/mycelium/issues/10) | P1 | — | infra / ADR-007 | **No** — no `experiments/` yet | **Ready** — pure scaffolding |

Legend — **Ready**: can start now from the corpus. **Author-then-ratify**: the issue is framed as
"ratify X" but X must be *authored* first (see §3). **Blocked**: a hard dependency is open.

---

## 3. The reframing every planner needs to see first

> **Finding (sequencing decision, 2026-06-09).** Three Phase-0 issues — M-010, M-011, and the soft
> dep of M-001/M-002 on M-010 — are written as *ratify the existing artifact*. **The artifacts do
> not exist yet.** `docs/spec/` is absent entirely: no `schemas/`, no `SPECIFICATION.md`. The
> `issues.yaml` header even notes the `docs/planning/` set (this file) was "forthcoming in-repo,"
> and the design corpus deliberately kept the data model *in the RFCs* (RFC-0001 §4.1–§4.8) rather
> than as standalone JSON.

This is not a defect in the corpus — it is the expected boundary between design and build. But it
changes the *shape* of M-010 and M-011 from a one-pass review into **author → self-review →
ratify**. The plan accounts for this explicitly:

- The check tooling is **already wired and waiting**: `scripts/checks/schema.sh` documents the
  exact convention (`docs/spec/schemas/<name>.schema.json` + `examples/<name>/{valid,invalid}/`)
  and skips gracefully until the directory appears. So authoring the schemas immediately lights up
  CI with no tooling work — M-010's "examples validate in CI" acceptance is satisfiable the moment
  the files land. (Grounding: `scripts/checks/schema.sh`; `justfile` `schema` recipe.)
- The schemas are a **faithful JSON projection of RFC-0001 §4.1–§4.8 + RFC-0002 §3 + RFC-0003 §6**,
  not new design. Anything that would require a *design* choice not already in the corpus is a
  flag, not a free decision — it routes back to a targeted RFC amendment (per the operating
  principle: minimal, targeted updates over rewrites).

### Proposed canonical schema set (the "10 schemas" of M-010)

M-010 references "the 10 JSON Schemas" as if enumerated; they are not, so here is the proposed
canonical set derived 1:1 from the corpus. Each is a draft-2020-12 schema under
`docs/spec/schemas/`. **This list is itself a ratification artifact** — confirming it *is* part of
M-010.

| # | `<name>.schema.json` | Source (normative) | Notes |
|---|---|---|---|
| 1 | `repr` | RFC-0001 §4.1 | `Binary/Ternary/Dense/VSA`; closed kinds, open `ScalarKind`/`ModelId`/`SparsityClass` registries |
| 2 | `value` | RFC-0001 §4.2, §4.8 | top-level `{repr, payload, meta}`; the self-describing wire form |
| 3 | `meta` | RFC-0001 §4.3 | the 7 `Meta` fields; carries invariants M-I1…M-I5 (expressed as far as JSON Schema allows; full check in M-104) |
| 4 | `guarantee` | RFC-0001 §3.4, §4.7 | `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` enum + the meet contract reference |
| 5 | `bound` | RFC-0001 §4.3, ADR-010 | `ErrorBound/ProbabilityBound/CrosstalkBound/CapacityBound` + `BoundBasis` |
| 6 | `provenance` | RFC-0001 §4.6 | `Root \| Derived{op, inputs}` content-hash DAG |
| 7 | `physical-layout` | RFC-0001 §4.1, §4.3; DN-01; RFC-0004 §5 | `PhysicalLayout` + `PackScheme` registry; the schedule *record* |
| 8 | `swap-certificate` | RFC-0002 §3, §5 | `Bijective(proof) \| Bounded{ε,δ,basis}` + legal-pair reference |
| 9 | `policy` | RFC-0005 | `PolicyRef` + reified `SelectionPolicy` surface (EXPLAIN-able) |
| 10 | `reconstruction-manifest` | RFC-0003 §6 | model+dim, content-addressed codebooks, recipe, decode params, bound cert |

If review collapses or splits any of these (e.g. `guarantee` folded into `meta`, or `bound` split
by kind), that adjustment is recorded against M-010 and the count is restated — the number "10" is
a target, not a constraint. **Open question OQ-1 (see §7):** confirm the set with the issue author
before authoring, since "10" implies a prior intent that may differ from this derivation.

---

## 4. Critical path & sequencing

```
            ┌──────────────────────────────────────────────────────────┐
            │ M-091 Rust skeleton ─┐                                     │
 (parallel, │ M-092 Py skeleton  ──┤  no deps — start immediately       │
  no deps)  │ M-090 Docs CI ───────┤  (M-090 ~ already partly done)      │
            │ M-012 bin↔tern spec ─┘                                     │
            └──────────────────────────────────────────────────────────┘

 CRITICAL PATH (everything downstream of the data contracts):

   M-010 author+ratify schemas ──► M-011 ratify SPECIFICATION.md
        │                                  (links §10 TODOs → issues)
        │
        ├──► M-001 LH bundle probe  (needs `bound`/`CapacityBound` shape)  ──► KC-1 confirmed
        │
        └──► M-020 surface fragment ──► M-002 LLM experiment ──► KC-2 verdict
```

**Why M-010 is the keystone (highest leverage, do first).** It unblocks the most: M-011 consumes
it directly; M-001 needs the `CapacityBound`/`BoundBasis` shape pinned (so the LH refinement's
post-condition matches what the Rust side will actually carry); M-104 (Phase 1) (de)serializes Core
IR *to these schemas*; M-101's Rust `Meta`/`Repr` types mirror them. Pinning the contracts early
means Phase-1 build tasks plug into stable slots — the same discipline RFC-0001 §2 used for the
RFC ordering.

**Recommended execution order** (priority- and dependency-ordered):

1. **M-010** schemas (P0, keystone) — and concurrently the no-dep scaffolds **M-091 / M-092 /
   M-090** (independent, keep CI green from day one).
2. **M-011** `SPECIFICATION.md` skeleton (consumes M-010) + **M-012** bin↔ternary spec (independent
   P1, low-risk, feeds Phase-1 M-120/M-121).
3. **M-001** LH `bundle` probe (P0, the KC-1 confirming build) — the highest-*signal* item; start
   its scaffolding in parallel with M-010 since it only needs the `CapacityBound` shape, not the
   whole schema set.
4. **M-020** surface fragment → **M-002** LLM experiment (P0, the KC-2 verdict).

---

## 5. Gate verdicts — honest status (KC-1, KC-2)

Per the honesty rule and VR-5, gate status is tracked at the strength it has actually been
*established*, not at its hoped-for level.

| Gate | Question | Current status | What moves it | Issue |
|---|---|---|---|---|
| **KC-1** | Does a core VSA op admit a usefully tight, honestly-statable bound? | **passed (literature)** — Clarkson-Ubaru-Yang 2023, Thomas-Dasgupta-Rosing 2021 (T0.2); MAP-I/sparse `bundle` tagged `Proven` | **confirmed (build)** when the LH module type-checks **and** Z3 discharges ≥3 concrete `(d,k,s,m,δ)` settings | M-001 (#2) |
| **KC-2** | Does LLM code-gen/reasoning survive on the Mycelium surface? | **open** — research did not settle it (RR-3) | a *written verdict* (proceed / reweight-to-human / fall-back-to-embedded-DSL) + SC-5b baseline X | M-002 (#3) |

**KC-1 honesty note.** The literature pass justifies the `Proven` *tag* (a non-asymptotic theorem
exists). M-001 confirms the *strategy* — that we can axiomatize the theorem statement and have the
checker discharge only the arithmetic instantiation (RFC-0003 §5; ADR-010). Until that build runs,
the status string stays `passed (literature)`; M-001's acceptance criterion is precisely the
upgrade to `confirmed (build)`. We do not pre-write the upgrade.

---

## 6. Per-task detail (the leverage items)

### 6.1 M-010 — Ratify core data-contract schemas  ·  #5 · P0 · keystone

- **Goal.** A ratified, CI-validated JSON-Schema rendering of the corpus data contracts (§3 set).
- **Acceptance (from issue).** ≥1 valid + ≥1 invalid example per schema; CI runs
  check-jsonschema/ajv over examples; schema status `draft → ratified`.
- **Plan.**
  1. Confirm the canonical set (§3 table) with the author — OQ-1.
  2. Author each `<name>.schema.json` as a faithful projection of its cited RFC section. Encode in
     JSON Schema what JSON Schema can express (enums, required fields, the M-I1 `Exact ⟺ bound
     absent` conditional, M-I2/3/4 basis↔strength coupling via `if/then`); record invariants it
     *cannot* express (content-hash purity WF4, lossless-packing M-I5) as `$comment` + a pointer to
     the Phase-1 code check (M-101/M-104) that enforces them — never silently drop them.
  3. Author `examples/<name>/valid/*.json` and `examples/<name>/invalid/*.json` — the invalid ones
     chosen to exercise each schema's *honesty-load-bearing* constraint (e.g. a `Declared` value
     with a `ProvenThm` basis must be **invalid**).
  4. `just schema` (already wired) goes green; flip each schema's `status` field to `ratified`.
- **Risk.** JSON Schema cannot express the cross-field invariants fully → mitigated by the explicit
  `$comment` + Phase-1 code-check pointer, so the contract is honest about what CI does and does
  not check. No silent gaps.

### 6.2 M-001 — Liquid-Haskell `bundle` capacity-refinement probe  ·  #2 · P0 · highest signal

- **Goal.** The one confirming build for KC-1 / ADR-010 / the cited-theorem strategy.
- **Acceptance (from issue).** LH module type-checks; Z3 discharges ≥3 concrete `(d,k,s,m,δ)`
  settings; short writeup confirms the axiomatized-theorem + checked-instantiation strategy; KC-1
  `passed (literature) → confirmed (build)`.
- **Plan.** Encode MAP-I `bundle` per RFC-0003 §5 as the refinement
  `{v | activeCount v ≤ s} → {d | d ≥ ⌈(2/μ²)·ln(m/δ)⌉} → {r | failProb r ≤ δ}`, with the T0.2
  theorem statement as an **axiom** (`assume`/measure) and Z3 discharging only the arithmetic
  instantiation. Pick ≥3 `(d,k,s,m,δ)` points spanning a realistic regime. Land it under the
  Haskell-probe area of the repo (a `proofs/` or `experiments/lh-bundle/` location — confirm
  placement against M-091/M-092 layout, OQ-2).
- **Dependency on M-010.** Narrow: only the `CapacityBound`/`BoundBasis` shape (schema #5) must be
  pinned so the refinement's post-condition matches the bound the Rust side will carry. Can start
  scaffolding before the full schema set ratifies.
- **Honesty.** This is `verification` work; its output upgrades a *tag's basis*, so the writeup
  must state exactly what is axiomatized (theorem soundness) vs. checked (arithmetic) — that
  distinction *is* the deliverable (RFC-0002 §7; ADR-010).

### 6.3 M-011 — Ratify `SPECIFICATION.md`  ·  #6 · P1 · gated on M-010

- **Goal.** A consolidated `docs/spec/SPECIFICATION.md` whose §1–§9 are reconciled to the RFCs and
  whose §10 (open items) each link to a live issue id.
- **Acceptance (from issue).** Status `consolidating-draft → ratified-skeleton`.
- **Plan.** Author the skeleton as a *thin consolidation index over the RFCs* (not a re-derivation
  — DRY; the RFCs stay normative). §1–§9 mirror: value model (RFC-0001), swaps (RFC-0002), VSA
  (RFC-0003), execution (RFC-0004), policy (RFC-0005), bounds (ADR-010). §10 enumerates the open
  build items — and **every** §10.x TODO must point at an issue: §10.1 surface fragment → M-020
  (#4); §10.2 Core IR node grammar → M-101 (#11); §10.3 reference-interpreter small-step rules →
  M-110 (#15); §10.5 lowering stages → M-112 (#17). No floating TODOs (mirrors RFC-0001 §8's
  "resolved with pointers" discipline).

### 6.4 M-091 — Rust workspace skeleton  ·  #9 · P0 · ready now

- **Acceptance (from issue).** `cargo fmt --check`, `clippy -D warnings`, `test` all green on the
  empty skeleton in CI.
- **Plan.** Cargo workspace with crates `mycelium-core`, `mycelium-interp`, `mycelium-vsa`,
  `mycelium-mlir` (stub), `mycelium-cert` (stub), `xtask`; **MSRV pinned 1.92** via
  `rust-version = "1.92"` + a `rust-toolchain.toml` (ADR-007). Do **not** bump the pin even if the
  container's toolchain is newer — that is an ADR-level decision, not a build detail (CLAUDE.md
  toolchain rule). Wire the Rust checks into `scripts/checks/lint.sh`/`format.sh` so `just check`
  covers them (they currently skip-when-absent).

### 6.5 M-090 — Docs CI  ·  #8 · P1 · mostly done

- **State.** `.github/workflows/checks.yml` already runs the full `just ci` suite (markdown-lint,
  link-check, schema, spell, structured, secrets) — **manual-dispatch, advisory** per the Remote
  CI policy (CLAUDE.md; do **not** add `on: push`/`pull_request` without an ADR).
- **Remaining for acceptance ("green on a no-op PR").** Confirm a PR template is wired
  (`.github/pull_request_template.md`), and verify the suite is genuinely green on a no-op — the
  schema/link checks skip gracefully today, so the gap is mostly the PR template + a green run
  record. Keep the advisory/manual posture; this issue is **not** a license to auto-trigger CI.

---

## 7. Risks & open questions

| Id | Item | Disposition |
|---|---|---|
| **OQ-1** | Is the §3 canonical 10-schema set the intended "10 schemas" of M-010? | **Ask the author before authoring** — "10" implies prior intent; the set is derived, not given. |
| **OQ-2** | Where do the LH probe (M-001) and a `proofs/` tree live relative to M-091/M-092? | Decide alongside M-091 layout; record in `SPECIFICATION.md` §10 / phase-1.md. |
| **RR-3** | LLM leverage collapses on novel syntax (KC-2) | The one existential risk research didn't settle; M-002 is its circuit-breaker. Acceptable outcomes include fall-back-to-embedded-DSL. (Foundation RR-3.) |
| **RR-2/KC-3** | Schema + spec surface grows past single-expert auditability | Keep schemas a thin 1:1 projection of the RFCs; no new design in the spec layer. |
| **Seq-1** | M-010/M-011 framed as "ratify" but artifacts absent (§3) | Reframed here as author-then-ratify; tooling already supports it (`schema.sh`). Surface to the author so issue bodies can be amended if desired. |

---

## 8. How this doc stays honest

- **Append-only with status transitions**, mirroring the ADR/RFC discipline (Foundation Meta): this
  file moves `Living draft → ratified` only when the Phase-0 exit gate (§1) is met; individual task
  rows update in place as their issues progress, but gate verdicts (§5) never pre-record an upgrade.
- **Every task row carries its issue number** so the board and this doc cannot silently diverge;
  `idmap.tsv` is the join key.
- **Progress is reported back to the issues**, not only here — each task's substantive output links
  its artifact from the GitHub issue (per the operating principle: close the loop back to issues).

---

## Meta — changelog & maintenance

- **2026-06-09 (initial draft):** first issue-coupled expansion of Foundation §6 Phase 0. Records
  the readiness table (§2), the author-then-ratify reframing for M-010/M-011 (§3), the proposed
  canonical 10-schema set, the critical path (§4), honest KC-1/KC-2 gate status (§5), and per-task
  plans for the leverage items (§6). Open questions OQ-1/OQ-2 raised for the author.
- Maintain append-only; supersede, don't rewrite. Re-run KC-1…KC-4 at the phase gate (Foundation
  Meta). Keep `Proven|Empirical|Declared` verdicts honest per VR-5.
