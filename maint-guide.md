# Comprehensive Automated Maintenance Review

> **Mycelium binding:** this is the standing **program/maintenance operating system** for agent
> swarms. It is the framework named in CHANGELOG (“maint-guide framework (orch plans; leaves
> execute)”). Pair with `CLAUDE.md` (house rules, DN-97 trunks, fractal ownership),
> `.claude/kickoffs/_doc-maintenance.md` (post-land doc DoD), and per-wave **program handoffs**
> under `docs/planning/**` (e.g. `gap-analysis-2026-07-16/PROGRAM-HANDOFF.md`).

## Objective

Perform a comprehensive maintenance review of the project to continuously improve implementation
quality while ensuring the repository remains aligned with its documented vision, roadmap,
architecture, and project management artifacts.

The maintenance process must continuously reconcile the implementation against the project's
documented goals, ensuring that the codebase, documentation, roadmap, and issue tracker remain
synchronized.

The maintenance process must never intentionally regress functionality, APIs, compatibility, user
experience, stability, security, or performance. Every accepted change must continue to satisfy all
project requirements, deliverables, acceptance criteria, and success metrics.

### Role lattice (L0 → L1 → L2) — mandatory for agent work

| Role | Who | Does | Does **not** |
|------|-----|------|----------------|
| **L0** | Program / project manager (session orchestrator) | Phase-0 alignment; wave plan; spawn L1; PR review gate; GHA/runner hygiene; promote PRs; issue/changelog **close-out ownership**; durable **handoff docs**; release **authorization** only | Product implementation; silent SemVer; force-push; direct commits to `main`/`integration`/`dev` |
| **L1** | Epic agent | Re-decompose; spawn L2; self-review L2 PRs; write epic handoff notes; FLAG shared files | Prefer not implement product code (tiny pins OK after L2 stall); protected-branch writes |
| **L2** | Leaf implementer (`isolation: worktree`) | Own disjoint paths; tests + emit/docs; open PR → parent tier; report SHA/PR/FLAGs | Shared files (`CHANGELOG`, `issues.yaml`, `Doc-Index`, `api-index`); merge to protected trunks |

**Model floor (this program):** L1 and L2 prefer **`grok-composer-2.5-fast`**. If the runtime only
offers another model, **record the actual model in the handoff** (never-silent).

**Handoff rule:** every L0↔L1 and L1↔L2 transition leaves a **durable doc** on disc (this guide’s
[Handoff packet](#handoff-packet-template) or the wave’s `PROGRAM-HANDOFF.md`). Branches hold code;
handoffs hold **intent** (CLAUDE.md mitigation #8).

---

# Phase 0 — Repository, Documentation & Project Alignment

Before modifying the implementation, perform a comprehensive review of the repository to understand
the project's intended direction.

Review, correlate, and reconcile all available project artifacts, including but not limited to:

* README
* Architecture documentation
* Design documents
* ADRs (Architecture Decision Records)
* Technical specifications
* Development roadmap (`docs/planning/phase-*.md`, Foundation roadmap)
* Milestones
* GitHub Projects / Project boards
* GitHub Issues **and** `tools/github/issues.yaml` (repo SoT; verify-first before implement)
* Feature requests / enhancement proposals / bug reports
* TODOs and FIXMEs
* Release notes / Changelogs (`CHANGELOG.md`)
* Developer / user / API documentation (`docs/api-index/`, `docs/Doc-Index.md`)
* CI/CD workflows (self-hosted `checks`; advisory remote CI policy)
* Repository configuration / coding standards / contribution guidelines (`CONTRIBUTING.md`)
* **Wave handoffs** under `docs/planning/**` and `.claude/kickoffs/`

Determine:

* Current project goals.
* Intended architecture.
* Planned features.
* Deferred work.
* Outstanding bugs.
* Technical debt.
* Documentation gaps.
* Missing implementations.
* Areas where implementation has diverged from documented intent.

Identify opportunities where existing plans, accepted issues, roadmap items, or documented intentions
can now be completed without introducing regressions.

Where practical, implement those items as part of the maintenance cycle (**via L2 leaves**, not L0
self-implement).

When documentation is outdated relative to the implementation, update the documentation (**L1/L0
close-out or dedicated docs leaf**).

When implementation is incomplete relative to accepted plans or specifications, complete the
implementation whenever it can be done safely within the existing architecture.

The implementation should continuously converge toward the documented goals of the project.

### Phase 0 exit criteria (Mycelium)

- [ ] Tip SHAs recorded for `main` / `integration` / `dev`
- [ ] Active M-ids verified against code (mitigation #14)
- [ ] Wave goal + honesty tags (`Empirical` / `Declared`) stated
- [ ] `PROGRAM-HANDOFF.md` (or dated successor) current
- [ ] Collision map (serial vs parallel crates) fixed before fan-out

---

# Phase 1 — Performance, Efficiency & Code Quality

Perform a comprehensive review of the implementation to identify opportunities to improve:

* Execution performance
* Memory utilization
* Resource consumption
* Startup and runtime latency
* Algorithmic efficiency
* Logical simplicity
* Code readability
* Maintainability
* Modularity
* Reliability
* Robustness
* Error handling
* Dependency health
* Technical debt
* **Transpile honesty** (`checked_fraction` vs `expressible_fraction`; never fabricate prims)

Where improvements can be implemented without altering intended behavior, apply them using the
smallest practical set of changes (**L2**, change-scoped).

Do not remove features, alter public APIs, change expected outputs, or introduce breaking changes
solely for optimization.

**House rules bind Phase 1:** VR-5 (no guarantee upgrades without basis), G2 (never silent),
append-only ADR/RFC/DN, grounding.

---

# Phase 2 — Validation & Regression Prevention

After all implementation changes have been completed:

* Execute the complete validation pipeline appropriate to the **tier**:
  * Leaf: change-scoped `cargo fmt` / `clippy -D warnings` / `test -p <crate>` (+ transpile-vet when
    measuring)
  * Promote leaf→`dev` / canary: `just check-canary` when feasible
  * `dev`→`integration` / `integration`→`main`: tier policy in CLAUDE.md (canary vs full `just check`)
* Run linting / formatting / static analysis as above
* Execute integration and end-to-end tests when available
* Validate benchmarks where applicable
* **Remote self-hosted GHA** advisory `checks` — L0 watches; runner via `gha-runner-ctl`

Confirm that:

* No functionality has been lost.
* No regressions have been introduced.
* Performance has not decreased.
* Stability has not decreased.
* Compatibility remains intact.
* Existing APIs continue to behave identically unless explicitly versioned.
* Honesty metrics are not over-claimed (pilot `checked_fraction` still `Empirical`).

If any proposed improvement introduces regressions or fails validation, discard or revise it before
proceeding.

---

# Phase 3 — Final Security Audit (Always Last)

After all implementation and documentation changes are complete, perform a comprehensive security
review of the resulting project (use `/security-review` skill or L0-spawned `security-reviewer`).

Audit for, including but not limited to:

* Known vulnerabilities
* Unsafe coding patterns
* Injection vulnerabilities
* Authentication / authorization / privilege escalation
* Secrets exposure
* Dependency / supply-chain risks
* Insecure defaults / misconfigurations
* Race conditions / memory safety / input validation
* Cryptographic misuse / data leakage / sensitive logging
* File system / network security concerns

Apply all security improvements that can be implemented while preserving existing functionality,
UX, public APIs, compatibility, stability, performance, and project requirements.

The security review is intentionally performed last so it evaluates the fully updated project.

---

# Security Exception Workflow

If a security issue cannot be remediated without introducing regressions or removing required
functionality:

* Do not merge the change.
* Create a dedicated branch using conventional naming.
* Implement the proposed remediation (L2).
* Open a **Draft** Pull Request.
* Document: vulnerability, severity, evidence, remediation, functional/compatibility impact,
  tradeoffs, alternatives, recommendation.
* Clearly indicate the remediation is intentionally withheld pending human review.

---

# Automated Integration & Project Management Workflow

When every validation step succeeds:

1. Create a feature branch using conventional naming (`claude/leaf/<EPIC>-<LEAF>-<kebab>`).
2. Commit changes using Conventional Commits (issue/task id when known).
3. Open a Pull Request (base = working tier, usually `dev`).
4. Execute automated PR review against a strict review rubric (`/pr-review` / L0-spawned reviewer).
5. Resolve review findings where possible (L2 patcher).
6. Re-run the complete validation pipeline (change-scoped or canary).
7. Merge **via PR only** after quality gates succeed:
   * leaf→`dev`: lineage `--merge` (`gh pr merge --merge`)
   * `dev`→`integration`: lineage `--merge`
   * `integration`→`main`: **squash only** (`/land`) — never day-to-day for residual waves
8. Generate the appropriate maintenance or patch release **only when L0 authorizes Epic R**
   (cz SemVer / release notes) — **not** automatic on every wave.
9. Produce release notes summarizing completed roadmap items, closed issues, performance /
   security / reliability / docs / dependency updates, and validation results.

Following a successful merge (close-out — **L0 owns shared surface**, may spawn L1 docs leaf):

* Close or update completed issues in **`tools/github/issues.yaml`** (SoT) then sync GH when able.
* Update project boards / milestones where appropriate.
* Update roadmap progress and planning handoffs.
* Remove completed TODOs and FIXMEs when safe.
* Update documentation to reflect the current implementation (`_doc-maintenance.md` DoD).
* Open follow-up issues for work intentionally deferred.
* Ensure repository metadata accurately reflects the project's current state.
* Append `CHANGELOG.md` + Doc-Index as needed.
* **Never force-push.** Pull-down before merge-up (DN-97 / CLAUDE.md).

### Trunk policy (DN-97)

| Trunk | Role | Content vs rigor |
|-------|------|------------------|
| `dev` | working | full check suite expected over time; messy OK |
| `integration` | staging | polished; promote PR from `dev` |
| `main` | release history | squash-only landings; lightweight export via `just package-release` |

Same-content trunks after promote: tree SHAs should match after `dev`→`integration` `--no-ff`.

---

# Handoff packet template

Copy into `docs/planning/<wave>/PROGRAM-HANDOFF.md` (or date-stamped successor). **Update before
every L0 pause, compaction risk, or L1 spawn.**

```markdown
# Program handoff — <WAVE-ID> (<ISO-DATE>)

| Field | Value |
|-------|--------|
| **L0** | <session / agent id> |
| **Phase** | 0 / 1 / 2 / 3 / close-out / release-hold |
| **main** | `<sha>` |
| **integration** | `<sha>` |
| **dev** | `<sha>` |
| **Trees identical?** | yes/no (dev↔int) |
| **Honesty** | Declared / Empirical notes |

## Goal (one paragraph)
…

## Done this cycle (PRs + SHAs)
| PR | Leaf | Result |
|----|------|--------|
| #… | … | … |

## Empirical measures (if any)
| Target | checked% | expressible% | File |
|--------|----------|--------------|------|
| … | … | … | … |

## Open queue (ranked)
1. …
2. …

## FLAGs (orch-owned)
- CHANGELOG / issues.yaml / Doc-Index / api-index: …

## Blockers
- GHA / runners / model floor / design gates: …

## Next L1 brief (paste-ready)
```
(contract: model floor, base tip, leaves, serial rules, do not merge, report shape)
```

## Release gate
- [ ] Pilot path honest
- [ ] Remote CI green on tip
- [ ] No one-shot over-claim
- [ ] L0 authorize SemVer
```

---

# Acceptance Criteria

Every accepted maintenance cycle must satisfy all of the following:

* Repository implementation aligns with documented project goals.
* Documentation accurately reflects the implementation.
* Completed roadmap items are implemented and tracked.
* Completed issues are closed or status-synced in `issues.yaml`.
* Project boards accurately reflect current progress (when used).
* No loss of functionality.
* No unintended behavior changes.
* No API regressions.
* No compatibility regressions.
* No stability regressions.
* No security regressions.
* No performance regressions.
* All automated tests required for the tier pass.
* All quality gates required for the tier pass.
* All project requirements remain satisfied.
* All deliverables remain complete.
* All acceptance criteria remain satisfied.
* All success metrics remain achieved **or** honestly deferred with FLAGs (VR-5).
* **Handoff packet updated** before the session ends or context is at risk.

If any condition cannot be satisfied automatically, the change must not be merged and must instead
follow the Security Exception Workflow or an equivalent draft-review workflow for human evaluation.

---

# See also

* `CLAUDE.md` — house rules, swarm, PR workflow, check tiers
* `.claude/kickoffs/_doc-maintenance.md` — docs DoD on land
* `.claude/kickoffs/README.md` — multi-session heads
* `docs/planning/gap-analysis-2026-07-16/` — active expressibility program
* `CONTRIBUTING.md` — human contributor path
