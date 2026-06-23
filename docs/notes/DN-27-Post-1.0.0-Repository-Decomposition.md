# Design Note DN-27 — Post-1.0.0 Repository Decomposition & Public MIT Release

| Field | Value |
|---|---|
| **Note** | DN-27 |
| **Status** | **Draft** (2026-06-23; planning capture, DN-17 posture) |
| **Feeds** | ADR-022 §10 (long-term vision); ADR-013 (`spore` deployable); ADR-018 (versioning); ADR-003 (content-addressed identity) |
| **Date** | June 23, 2026 |
| **Decides** | *Nothing normatively* — advisory. Records the **post-`lang 1.0.0`** plan (maintainer, 2026-06-23): decompose the monorepo into component repos + phylum re-export repos and flip the project to a public, MIT-licensed corpus. Captured now so the 1.0.0 work is shaped with this end-state in mind; the binding decision is a future ADR drafted when `lang 1.0.0` nears. |
| **Task** | post-1.0.0 (gated on the ADR-022 gate closing) |

> **Posture (honesty rule / VR-5).** Advisory, forward-looking. Enacts nothing — the repo today is a
> single private monorepo. This note only records the intended end-state so present decisions
> (phylum boundaries, re-export surfaces, licensing) don't paint it into a corner.

---

## 1. Goal

The maintainer's end-state (ADR-022 §10): **once `lang 1.0.0` is reached, completed, and satisfied**,
the project is restructured for public consumption — decomposed into focused **component repos** and
**phylum re-export repos** that present a friendly interface over the phyla/nodules, and the whole set
is published as **public, MIT-licensed repositories** so developers, users, and models have a useful,
composable corpus to leverage.

## 2. User stories / motivating use cases

- As a **downstream app developer**, I want to depend on a small **component repo** (one phylum's
  implementation) without cloning the whole project, so my dependency surface is minimal and versioned.
- As a **library/phylum author**, I want a **phylum re-export repo** that groups the component repos
  and presents one coherent interface, so consumers import a phylum, not a pile of components.
- As a **model / AI co-author**, I want the public MIT corpus split into navigable repos with clear
  re-export surfaces, so I can retrieve and reason over a phylum without ingesting the monorepo.
- As the **maintainer**, I want the decomposition to fall out of the existing phylum/nodule boundaries
  (content-addressed identity, ADR-003), so splitting is mechanical, not a re-architecture.

## 3. Scope & decision space (post-1.0.0)

In scope (when triggered): the repo topology (component repos ↔ phylum re-export repos), the mapping
from phyla/nodules to repos, the public-release + MIT-licensing flip (private monorepo → public repo
set), version/tag propagation across repos (ADR-018 per-axis SemVer), and how `spore` artifacts
(ADR-013) publish from the split repos. Out of scope: changing any language semantics; this is purely
distribution/topology.

Decisions deferred to the future ADR: per-repo vs workspace versioning, the re-export repo's interface
form (does it re-export source, `spore` artifacts, or both?), CI/release wiring across N repos, and the
migration order (likely mirrors the DN-26 zero-Rust port order — stabilize a phylum, then split it).

## 4. Definition of Done (for this note's *eventual* realization)

- A binding ADR (drafted when `lang 1.0.0` nears) fixes the component/re-export topology + the
  phylum→repo mapping + the public-release + MIT flip mechanics.
- Every split repo is **MIT-licensed** (first-party rule, ADR-022 §7) and traces to its monorepo origin
  (content-addressed identity preserved — ADR-003).
- The public repo set is navigable: each phylum re-export repo presents a documented interface over its
  component repos; no consumer needs the monorepo.
- The flip is **gated on `lang 1.0.0`** — never executed earlier (the monorepo stays the single source
  of truth until the gate closes).

## 5. Open questions

- **Trigger granularity:** all-at-once at 1.0.0, or phylum-by-phylum as each stabilizes (tracking the
  DN-26 port)?
- **Re-export form:** source re-export vs `spore`-artifact registry vs both.
- **History:** carry full git history into each component repo, or start clean from the 1.0.0 tag?
- **Tooling:** does `gh-issues-sync.py`'s reconcile contract extend across N public repos, or one
  orchestrating repo?

## 6. Grounding & honesty

The project is a single private monorepo today; this note enacts none of the above. It is grounded in
the maintainer's 2026-06-23 directive (ADR-022 §10 Q3 resolution) and the existing phylum/nodule model
(DN-06) + content-addressed identity (ADR-003) that make a mechanical split feasible. No tag is
upgraded; no release is declared.
