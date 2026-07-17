# Design council — swaps · tagging · UX/DX ergonomics (2026-07-17)

| Field | Value |
|-------|--------|
| **Status** | **In session** — research/design phase (not implement) |
| **Participants** | Maintainer · L0 (grok-4.5) · **3× design agents (grok-4.5 high)** |
| **Scope** | Mycelium only — ergonomics of **swaps** + **required tagging** (Meta, honesty/transparency lattice, manual metadata, typing) |
| **Out** | Component-repo decomp · SemVer claim · product code in this phase |
| **After** | Capture design docs · update ranked workstream/waves · resume autonomous L0→L1→L2 (agents = composer-2.5-fast) |

## Goal

Identify **ergonomic improvements** and **deterministic machinery** that simplify UX/DX for:

1. **Swaps** — use, management, typing of representation changes (never-silent, certificates, policies).
2. **Tagging surface** — honesty/transparency lattice (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`), Meta/provenance, any **manual** metadata or typing annotations the language currently forces on authors.
3. **Smoothing** — defaults, inference, sugar, tooling, and **deterministic** (reproducible, EXPLAIN-able) machinery so the power of swaps/tags is not a cognitive tax.

## Non-negotiables (bound, not negotiable away)

- Transparency / never-silent (G2); VR-5 (no guarantee upgrade without basis).
- Swaps never silent (RFC-0001/0002).
- Append-only decisions when ratifying later.
- Prefer **deterministic machinery** over ad-hoc convention.

## Agent partition (three lenses)

| Agent | Lens | Primary corpus |
|-------|------|----------------|
| **A — Swaps ergonomics** | Authoring/managing/typing swaps; certificates; policies; surface syntax pain | RFC-0001 §Swap · RFC-0002 · Glossary §2.19 · companion airlocks · DN-29 modes |
| **B — Tagging / Meta / lattice UX** | Guarantee tags, Meta, provenance, manual annotations, mode (fast vs certified) | ADR-032 · DN-29 · Glossary §2.4–2.5 · RFC-0005 EXPLAIN · companion 02-guarantee-airlocks |
| **C — Deterministic machinery + synthesis** | Cross-cutting defaults, inference, sugar, tooling; stress-test A/B proposals; ranked options | All of above + language surface (RFC-0006/0020) · existing emit/transpile tag patterns as *evidence of friction* |

## Deliverable per agent (to L0 → council)

1. Pain inventory (concrete, cited).
2. 3–7 design options (ranked); tradeoffs.
3. **Recommended direction** (not ratified — Draft only).
4. Deterministic machinery candidates (rules/defaults that remove manual work).
5. Open questions for maintainer.
6. Suggested follow-on M-ids / wave slots (Declared).

## Council agenda (with maintainer)

1. Agents report (A → B → C).
2. L0 synthesizes tensions.
3. Maintainer steers / selects.
4. Capture: design note(s) + workstream re-rank → then resume autonomous implement waves.

## Trunk baseline at council open

- `main` `aad96b7a` · `dev` `d71c9b02` · `integration` `3f60a0d2` (same tree)
- ONESHOT transpile prep mid-flight; this council **pauses** implement pace for design quality.
