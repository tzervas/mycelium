# Companion guide — relating Mycelium's ideas (maintained supplement)

> **Status:** living companion · **not** a substitute for the normative corpus
> (RFC / ADR / DN / `Doc-Index.md`). **Honesty:** synthesis is `Declared` unless a
> claim cites code/tests (`Empirical`) or a discharged proof (`Proven`).
>
> **Why this exists:** the append-only corpus is the permanent architecture record.
> Readers still need a *historian* — thematic maps, airlock patterns, and visual
> relations — so they can navigate capability space without replaying meeting order.
> Grounded in the 2026-07-16 external analysis pack + audio critique
> (`_sources/`); see `_sources/README.md`.

## Start here

| Path | For |
|---|---|
| [00 — How to read](00-how-to-read.md) | Resolution chain: CHANGELOG → CURRENT-STATE → Doc-Index → companion themes |
| [01 — Thesis & tower](01-thesis-and-tower.md) | Why Mycelium exists; L0→L3; swap-as-first-class |
| [02 — Guarantee airlocks](02-guarantee-airlocks.md) | Weakest-wins lattice **and** how to quarantine `Declared` contamination |
| [03 — Memory as one lifecycle](03-memory-as-lifecycle.md) | L1→L2→L3 as unified fallout of acyclicity + structured concurrency |
| [04 — Three trust axes](04-three-trust-axes.md) | Guarantee grade · certification depth · typing strictness (DN-126) |
| [05 — Thematic decision map](05-thematic-decision-map.md) | ADR-045 window grouped by *what it completes*, not DN number |
| [06 — Expressibility & transpile](06-expressibility-and-transpile.md) | L3 native answers · M-991 profiler · path to one-shot readiness |
| [Diagrams](diagrams.md) | Mermaid maps (lattice, memory, decision clusters, crate strata) |

## What this is / is not

| This supplement **is** | This supplement **is not** |
|---|---|
| Thematic curator over the corpus | A rewrite of Accepted/Enacted RFCs/ADRs |
| Developer-facing "how ideas interlock" | A second source of truth for statuses |
| Safe place for narrative + diagrams | A license to renumber historical docs |

**Append-only rule still holds in `docs/rfcs/`, `docs/adr/`, `docs/notes/`.** The
companion *points* and *groups*; it does not edit decision bodies.

## Maintenance

- After a wave that lands a design cluster: update § in `05-thematic-decision-map.md`
  and any diagram edges (do not invent status — read `Doc-Index.md`).
- After a guarantee/cert surface change: re-check airlock examples in `02-`.
- Index entry: `docs/Doc-Index.md` § companion; regenerate tero-index when linked.
