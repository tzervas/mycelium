# Design council — swaps · tags · honesty-poison containment · UX/DX (2026-07-17)

| Field | Value |
|-------|--------|
| **Status** | **In session** — research/design phase (not implement) |
| **Participants** | Maintainer · L0 · **design agents A–E** (research effort as recorded per agent) |
| **Scope** | Mycelium only — ergonomics of **swaps**, **required tagging** (Meta, honesty lattice, manual metadata, typing), **honesty-poison containment** (isolation so a downgraded grade cannot silently contaminate whole applications or datasets), and a **broader UX/DX scan** beyond pure swaps/tags |
| **Out** | Component-repo decomp · SemVer claim · product code in this phase |
| **After** | Capture design docs · update ranked workstream/waves · resume autonomous L0→L1→L2 (agents = composer-2.5-fast) |

## Goal

Identify **ergonomic improvements** and **deterministic machinery** that simplify UX/DX for four first-class pillars:

1. **Swaps** — use, management, typing of representation changes (never-silent, certificates, policies).
2. **Tagging surface** — honesty/transparency lattice (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`), Meta/provenance, any **manual** metadata or typing annotations the language currently forces on authors.
3. **Honesty-poison containment** — prevent whole-application / downstream **poisoning** from a downgraded honesty rating in a program or dataset: retain accuracy safely; prefer **deterministic** isolation (airlock / firewall / quarantine / meet-boundary); minimum **transparent EXPLAIN** on every isolation decision; do not kill quality; **no greenwashing** (no cast that makes weak data look Exact).
4. **Broader UX/DX** — improvements worth investigating beyond pure swaps/tags (diagnostics, tooling presentation, three-axis clarity, authoring paths, transpile-scale friction).

**Smoothing** across all four: defaults, inference, sugar, tooling, and **deterministic** (reproducible, EXPLAIN-able) machinery so power is not a cognitive tax and honesty is not a silent tax on the rest of the plant.

## Non-negotiables (bound, not negotiable away)

| # | Gate | Basis |
|---|------|-------|
| N1 | Transparency / never-silent | G2 |
| N2 | No guarantee upgrade without checked basis | VR-5 |
| N3 | Swaps never silent; policy identity recorded | RFC-0001/0002 S1/WF1–WF2 |
| N4 | Append-only decisions when ratifying later | house rule #3 |
| N5 | Prefer **deterministic machinery** over ad-hoc convention | KC-3 / G2 |
| N6 | **Isolation of contamination** — meet may weaken; it must not silently launder. Weak grades stop at explicit boundaries unless an airlock remints with basis | companion 02 · RFC-0018 G-Swap · VR-5 |
| N7 | **No quality kill** — containment must not force global `Declared` or disable Exact cores; separate paths and seals preserve high-assurance work | companion 02 cleanroom · FR accuracy goals |
| N8 | **No greenwashing** — success of isolation is never "looks Exact downstream" without predicate, Swap cert, or equivalent checked remint | VR-5 · G2 |

## Agent partition (five lenses)

| Agent | Lens | Primary corpus | Deliverable |
|-------|------|----------------|-------------|
| **A — Swaps ergonomics** | Authoring/managing/typing swaps; certificates; policies; surface syntax pain; **how failed checks / partial regimes must not present as Exact downstream** | RFC-0001 §Swap · RFC-0002 · Glossary §2.19 · DN-29 · `stdlib/swap.md` | [AGENT-A-SWAPS-ERGONOMICS-2026-07-17.md](./AGENT-A-SWAPS-ERGONOMICS-2026-07-17.md) |
| **B — Tagging / Meta / lattice UX** | Guarantee tags, Meta, provenance, manual annotations, mode (fast vs certified); **airlock / firewall / quarantine / meet-boundary as a core design slice** | ADR-032 · DN-29 · RFC-0018 · RFC-0005 · companion 02/04 · DN-126 | [DN-141](../../notes/DN-141-Tagging-Meta-Honesty-Lattice-UX.md) (Draft) |
| **C — Deterministic machinery + synthesis** | Cross-cutting defaults, inference, sugar, tooling; stress-test A/B/D/E; **ranked AX-stack including containment IDs** | All agent inputs · language surface (RFC-0006/0020) · emit/transpile friction as evidence | [AGENT-C-AX-STACK-SYNTHESIS-2026-07-17.md](./AGENT-C-AX-STACK-SYNTHESIS-2026-07-17.md) |
| **D — Honesty-poison containment** | Contamination threat model; deterministic isolation rules; EXPLAIN packages; quality retention without greenwash | companion 02 · RFC-0018 meet · RFC-0034 modes · Agent A cert fallibility · DN-141 isolation slice | [AGENT-D-HONESTY-POISON-CONTAINMENT-2026-07-17.md](./AGENT-D-HONESTY-POISON-CONTAINMENT-2026-07-17.md) |
| **E — Broader UX/DX backlog** | Improvements beyond pure swaps/tags worth investigating; ranked backlog for post-council re-rank | Diagnostics contracts · DN-126 axes · transpile DX · tutorial/tooling gaps | [AGENT-E-UX-DX-BACKLOG-2026-07-17.md](./AGENT-E-UX-DX-BACKLOG-2026-07-17.md) |

## Deliverable per agent (to L0 → council)

1. Pain inventory (concrete, cited).
2. 3–7 design options (ranked); tradeoffs.
3. **Recommended direction** (not ratified — Draft only).
4. Deterministic machinery candidates (rules/defaults that remove manual work).
5. Open questions for maintainer.
6. Suggested follow-on M-ids / wave slots (Declared).
7. **Cross-links** to other agents so ranks do not contradict without an explicit tension flag.

## Council agenda (with maintainer)

1. Agents report (**A → B → D → E → C**): swaps, tags, **poison containment**, broader UX, then synthesis.
2. L0 synthesizes tensions (especially isolation vs ceremony vs quality).
3. Maintainer steers / selects.
4. Capture: design note(s) + workstream re-rank → then resume autonomous implement waves.

## Cross-cutting product question (all agents answer)

> When a value, binding, or dataset is at grade *g* weaker than its consumer demand, what **deterministic** isolation options exist so the rest of the application retains accuracy and Exact cores stay Exact — with **EXPLAIN** for every dynamic boundary and **no** silent upgrade?

Agent D owns the threat model and isolation catalog; Agents A/B own swap-cert and lattice/airlock machinery; Agent C ranks the joint package; Agent E captures non-swap/tag UX that makes containment usable.

## Trunk baseline at council open

- `main` · `dev` · `integration` tracked at session open (same-content trunks under DN-97; re-fetch before implement resume).
- ONESHOT transpile prep mid-flight; this council **pauses** implement pace for design quality.

## Artifact map

| Artifact | Role |
|----------|------|
| This brief | Council charter + non-negotiables + partition |
| Agent A | Swap ergonomics package A1–A7 + poison × cert interaction |
| DN-141 (B) | Lattice/Meta UX stack with **isolation as core slice** (Draft — not Accepted) |
| Agent D | Poison-containment design (threat model, isolation rules, EXPLAIN package) |
| Agent E | Broader UX/DX investigation backlog |
| Agent C | Unified AX-stack ranks (X1… + containment AX-IDs) + stress questions |

**Status discipline:** no agent may claim RFC/DN/ADR **Accepted**. DN-141 stays **Draft** until maintainer ratification. All mechanisms remain `Declared` until differential-witnessed (VR-5).
