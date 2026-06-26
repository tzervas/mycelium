# RFC-0005 — Selection-Policy Language

| Field | Value |
|---|---|
| **RFC** | 0005 |
| **Status** | **Accepted** (solidified from the research pass) |
| **Type** | Foundational / normative |
| **Date** | June 08, 2026 |
| **Depends on** | RFC-0001 (`SelectionPolicy`, `PolicyRef`, `Meta.policy_used`, content-addressing); ADR-006 (reified policies); **G2**, cross-cutting **D**; Research Findings **T2.3** |
| **Coupled with** | RFC-0002 (swap-target selection), RFC-0004 (packing-schedule selection — same mechanism) |

## 1. Scope
The `SelectionPolicy` behind `PolicyRef`: how automatic, metadata-driven representation selection (swap targets; packing schedules) is expressed, reified, recorded, and explained — analyzable by construction, never opaque (ADR-006, G2).

## 2. Decision — a total, non-learned, cost-based policy with mandatory EXPLAIN (T2.3)
Database cost-based optimizers are the richest precedent and teach both how to make selection auditable **and exactly how it becomes a black box**:
- *Auditable via `EXPLAIN`/`EXPLAIN ANALYZE`:* chosen plan, per-candidate estimated cost, estimated-vs-actual.
- *Black-box failure modes (documented):* (1) costs in arbitrary internal units detached from hardware; (2) **cardinality estimates** (sampling error, staleness, broken independence) — the dominant error source; (3) operators that *can't* be estimated → silently wrong costs. **The opacity lives in the statistics-driven estimates, not the rules.**

**Mycelium's design:**
1. **Form:** a **decision-table / total predicate** function from inspectable inputs (queryable `Meta`: bounds, `dtype`, sparsity class) to a choice among a **finite** candidate set, with an explicit cost function. **Not Turing-complete, terminating, total** — an unanalyzable policy *is* the black box ADR-006 forbids; the expressiveness ceiling is the feature.
2. **Mandatory EXPLAIN:** every automatic selection emits an inspectable record `{inputs considered, cost of each candidate, chosen option, deterministic override hook}`.
3. **Determinism:** same `Meta` in → same choice out (reproducible, content-addressable).
4. **Override:** user hints / forced choices are first-class (cf. optimizer plan hints), for deterministic control.
5. **Mycelium avoids the cardinality-estimation trap:** its "statistics" are **exact metadata** (proven/declared bounds, `dtype`, sparsity class), **not sampled estimates** — so the principal source of optimizer opacity does not arise here. This is a structural advantage worth stating explicitly.

## 3. Reification & recording
A policy is a first-class, **content-addressed** value (inspectable, diffable). Every swap/packing decision records the `PolicyRef` (the policy's content hash) it used (RFC-0001 `Meta.policy_used`). Guarantee (the operational form of G2): one can always answer *"which policy chose this, and what does that policy do?"*

## 4. Scope of application & explanation
- **One mechanism, two sites:** swap-target selection (RFC-0002) and packing-schedule selection (RFC-0004). No parallel mechanisms (DRY/SoC).
- **Explainability:** an `explain(policy, meta) → trace` capability is part of the contract; the LSP surfaces it (SC-5) to answer "why was this representation/packing chosen?".
- **Composition/conflict:** multiple applicable policies compose deterministically; conflicts resolve by a fixed, declared precedence (no nondeterminism).

## 5. Interfaces
Provides `SelectionPolicy`/`PolicyRef` for RFC-0001's `Swap` and `Meta.policy_used`; same mechanism serves **RFC-0004** packing schedules; policies content-addressed per RFC-0001 §4.6.

> **Footnote — tunable certification (RFC-0034 / ADR-032, 2026-06-24; append-only).** Selection/cert machinery is gated by the active certification mode per **RFC-0034**; **`EXPLAIN` of the active mode itself stays mandatory in every mode** (G2 — never silent). The selection mechanism is **unchanged**. See **ADR-032**, which supersedes the *unconditional* reading.
