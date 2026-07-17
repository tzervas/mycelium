# Design Agent F — Diagnostics · tracing emitters · first-fault localization (2026-07-17)

| Field | Value |
|-------|--------|
| **Status** | **Draft** (council research — **not** Accepted; does not ratify) |
| **Agent** | F — Diagnostics / tracing emitters at hard-to-trace points |
| **Honesty** | Claims are **`Declared`** (corpus-grounded design) unless tagged otherwise |
| **Scope** | Mycelium only; no product code; no merge PRs |
| **Design pack home** | [DESIGN-03](./DESIGN-03-MACHINERY-DIAGNOSTICS-AND-UX.md) §3 (normative pack summary) · [01](./DESIGN-01-SWAPS-AND-POLICY.md) swap sites · [02](./DESIGN-02-TAGS-META-AND-CONTAINMENT.md) isolation instance |
| **Role** | **Annex** — deep site catalog + Localize-1 rationale for the three-doc design pack (not a fourth pack) |
| **Grounds** | RFC-0013 (Enacted) · DN-04 · RFC-0034 §7 gen≠consumption · RFC-0005 EXPLAIN · DN-22 (Draft) · G2 / G11 / VR-5 / KC-3 |

> **Posture (VR-5 / G2 / house rule #3).** Recommends a **first-fault localization bus**:
> deterministic structured events at compile-time and runtime chokepoints so authors learn
> **where / how / why** without digging the whole tree. Does **not** move RFC/DN/ADR status.
> Diagnostics remain **additive** over explicit errors (DN-04 / RFC-0013). Generation is cheap
> at key sites; **consumption** is tiered so noise is never a perf death (RFC-0034 §7).

---

## 0. Mandate (maintainer, council integrate)

1. **Emitters at hard-to-trace points** — compile-time and runtime issues **instantly
   localizable** (where · how · why) without tree archaeology.
2. Prefer **deterministic structured events** as truth; minimum **transparent EXPLAIN**
   (lean / normal / audit).
3. **Generation ≠ consumption** — always generate enough signal for first-fault; never force
   full audit traces into the hot path or default CLI noise floor.
4. **No third swap-policy system** — policy create/apply stays Agent A (catalog · default ·
   resolve-and-record · EXPLAIN); F only emits `policy_resolve` events for that path.
5. Cross-link isolation EXPLAIN, grade/meet/seal surfaces, and Agent E presentation under
   **one** envelope.

---

## 1. Pain inventory (localization DX)

| ID | Pain | Who | Tag |
|----|------|-----|-----|
| **F1** | Failure symptom far from cause (Declared pipeline, SwapError, meet refuse at consumer; first downgrade elsewhere) | authors, certified ops | `Declared` DX tax |
| **F2** | Multiple refuse classes, no shared first-fault schema | everyone | A P5/P9; D T3; RFC-0013 partial productization |
| **F3** | EXPLAIN packages designed separately (isolation, policy, grade, mode) without a common bus | tooling, LSP | `Declared` integration gap |
| **F4** | Compile vs runtime split hides half the story | certified authors | A dual-channel; RFC-0002 TV incompleteness |
| **F5** | Noise fear kills useful emitters (nothing vs firehose) | operators | RFC-0034 §7; DN-04; KC-3 |
| **F6** | Transpile/vet residual operator-heavy; first poison not a structured first-fault | porters, L0 | E E16; M-1043/1046 partial |

---

## 2. Principles

| # | Principle | Basis |
|---|-----------|-------|
| **FP1** | **First-fault wins** — originating refuse/downgrade is primary; symptoms cite `parent_event` | council N9 · G2 |
| **FP2** | Structured event is truth; prose/JSON/code are G11 projections | RFC-0013 · DN-04 · DN-22 |
| **FP3** | Additive, never substitutive over `Result`/`Option`/type errors | DN-04 I1 |
| **FP4** | Generate at sites; consume by tier | RFC-0034 §7 |
| **FP5** | No black-box policies — resolve always records identity + basis_ref | RFC-0005 · WF2 · VR-5 |
| **FP6** | Tooling layer (`mycelium-diag` / checker / runtime hooks), not kernel logging deps | KC-3 |

---

## 3. First-fault event envelope (minimum)

| Field | Meaning |
|-------|---------|
| `event_id` | Stable id for this fault instance |
| `phase` | `compile` · `check` · `runtime` · `transpile` · `packaging` |
| `site_kind` | §4 catalog |
| `where` | nodule path · span · IR node · hypha id |
| `how` | registry machine code (RFC-0013 / DN-22 projection) |
| `why` | structured reason enum + optional one-liner |
| `decision` | `ok` · `refuse` · `fallback` · `downgrade` · `remint` · `candidate` |
| `inputs` | grades, Reprs, policy refs, mode — as applicable |
| `basis_ref` | cert hash, predicate id, catalog policy hash, or empty |
| `parent_event` | first-fault link if symptom |
| `explain_tier_ready` | lean · normal · audit expandability |

**Isolation EXPLAIN** (DN-141 §3.5 / Agent D M10) is an **instance** of this envelope with
`site_kind ∈ {airlock, firewall, quarantine, meet_refuse, swap_check}`.

**Policy resolve** (Agent A A1/A2) is an instance with `site_kind: policy_resolve`.

---

## 4. Site catalog (must-emit)

### 4.1 Compile / check

| site_kind | Trigger | Instant localization |
|-----------|---------|----------------------|
| `legal_pair_refuse` | Illegal Repr pair / no bound | pair; regime; catalog hints |
| `policy_resolve` | Catalog path or `policy: default` → `PolicyRef` | hash + catalog id |
| `missing_conversion` | Cross-paradigm without `swap` | from/to; insert candidate (never auto) |
| `regime_type_lie` | Total type over partial regime | A-PC1; required Option/Result |
| `meet_boundary` | Meet across export / certified demand / Exact partition | grades; rule id |
| `grade_annotation` | Illegal strengthen | G-Weaken path |
| `mode_firewall` | fast floor vs certified Exact demand | mode × grade cell |
| `import_first_edge` | First bad import edge | path; grade of import |
| `transpile_gap` | First poison / residual | gap class; src breadcrumb; idiom |

### 4.2 Runtime

| site_kind | Trigger | Instant localization |
|-----------|---------|----------------------|
| `swap_exec` | Swap Ok/Err / out-of-range | pair, policy hash, regime |
| `swap_check` | Cert Validated / Refuted / NotValidated | cert id; fallback taken |
| `seal_remint` | Airlock pass/fail | predicate; remint grade |
| `isolation_dynamic` | Runtime quarantine / partition / firewall | isolation package fields |
| `grade_meet` | Dynamic meet | meet root + first weak leaf |
| `reconstruction` | Lossy Dense/VSA | reconstruction policy (D M8) |

### 4.3 Non-sites

- Every arithmetic / field access — **no**.
- Pure Exact success in `fast` — crumb optional only; no event flood.
- Intermediate meets inside an already-quarantined region — package at export boundary.

---

## 5. Options ranked

| Rank | Option | Verdict |
|------|--------|---------|
| **★ 1 Localize-1** | Shared FirstFault package + site catalog + tiered EXPLAIN + optional runtime breadcrumb ring | **Recommend** |
| **2** | Domain silo enrichment only (GapReason / cert strings) | Interim only; absorb into Localize-1 |
| **3** | Always-on full spans (OTel-style default) | **Reject** (G-perf / F5) |
| **0** | Docs-only | **Reject** as sufficient |

### F-core (Rank 1) package

1. Envelope schema (§3) shared by A/B/D/E.
2. Site catalog (§4) as checker + runtime hooks.
3. First-fault linking via `parent_event`.
4. Consumption: **lean** (site · where · decision · code) · **normal** (+ why · inputs · basis) · **audit** (full envelope + DAGs).
5. DN-22 codes project from the same record when productized.
6. Swap/policy fail path always emits; LSP/CLI jump to `where`.
7. Isolation package = subtype of envelope (Agent D).

---

## 6. Pack interaction matrix

| Peer | Interaction |
|------|-------------|
| **Pack 01** | Policy streamline **is** `policy_resolve` emitters. Failures emit `swap_exec` / `swap_check` / `regime_type_lie`. Fail localization is first-class on swap/check. |
| **Pack 02** | Grade/meet/seal diagnostic surfaces → `grade_meet` / `seal_remint` / `meet_boundary`. Isolation EXPLAIN ⊆ F envelope. |
| **Pack 03 §3** | Normative Localize-1 summary + AX X15; this annex holds the full catalog. |
| **Pack 03 §5 Rank-1 UX** | Presentation **consumes** F bus. Fold F into Rank-1 as diagnostics spine. No parallel schema. |
| **AX ranks** | X1 = policy streamline (explicit); **X15** = diagnostic bus / first-fault. |

---

## 7. Adversarial stress-test

| Attack | Response |
|--------|----------|
| "Just logging" | Structured, registry-backed, first-fault-linked, EXPLAIN-expandable; additive (DN-04) |
| "Always-on kills perf" | Lean stub; audit on query; no kernel logger |
| "Duplicates RFC-0013" | Extends productization to council site catalog + first-fault links |
| "Duplicates isolation EXPLAIN" | Isolation is a site family, not a second bus |
| "Third policy system" | Hard gate: only Agent A catalog + default + EXPLAIN |
| "`next` teaches laundry" | Candidates never suggest `as Exact`; seal/Swap with basis only |

**Verdict:** Localize-1 holds if fault-only, single policy catalog (A), I1 additive, lean never hides export Declared/refuse.

---

## 8. Open questions (maintainer)

1. **OQ-F1** Diag home: extend `mycelium-diag` / LSP diagnostics only, or also `std.diagnostics` (M-346)?
2. **OQ-F2** Runtime breadcrumb ring default on/off in `fast`?
3. **OQ-F3** `event_id` content-addressed vs session-scoped?
4. **OQ-F4** DN-22 compact codes pace vs FirstFault schema first?
5. **OQ-F5** How early may F schema land tooling-only during design pause?
6. **OQ-F6** Certified: audit materialize on every refuse, or pull-based?

---

## 9. Wave sketch (Declared; no M-ids minted here)

| Wave | Content |
|------|---------|
| **W0** | Transpile worklist + mode print (E Rank-1 partial; necessary not sufficient) |
| **W1** | FirstFault schema in diag crate |
| **W2** | Check attachments: import / meet / swap / policy |
| **W3** | Runtime swap + isolation + optional crumb ring |
| **W4** | `myc explain` / unified EXPLAIN panel (E) |
| **W5** | Sugar span-map / `reveal` |

Prefer **widen** existing M-104x / EXPLAIN issues after free-id verify; do not double-mint.

---

## 10. Definition of Done (this artifact)

- [x] Pain F1–F6; principles FP1–FP6; envelope; site catalog.
- [x] Localize-1 recommendation; gen≠consumption; no third policy system.
- [x] Isolation/policy as instances; cross-links A–E/C; stress-test; OQs; waves.
- [ ] Maintainer steers OQ-F1/F2/F4 → schema freeze + build issues.

---

## Meta

- **2026-07-17 — Draft (Agent F / design-reasoner + integrator).** First-fault localization bus
  annex for the three-doc design pack. No RFC/DN status change. No product code. Complements
  pack 03 Rank-1 presentation; does not replace RFC-0013. Pack 02 remains the Draft DN-141 body
  successor — this file is a planning annex, not DN-142.
