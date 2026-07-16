# Program handoff — ORACLE-R1 / expressibility residual (2026-07-16)

| Field | Value |
|-------|--------|
| **Framework** | Repo-root **`maint-guide.md`** (Phase 0–3 + L0→L1→L2 + PM close-out) |
| **L0** | Grok session (PM/orchestrator only — no product self-implement) |
| **Phase** | **Close-out / release-hold** — Phase 1 A1–A5 landed; Phase 2 remote CI still watched; Phase 3 security not yet cycle-end; **Epic R SemVer held** |
| **main** | `aad96b7a` (v0.463.1) |
| **integration** | `060326db` (promote #1652) |
| **dev** | `f783d4ce` |
| **Trees identical?** | **yes** — `dev` and `integration` share tree `8918798766100044d95ea109cf1d1a639ad7499a` |
| **Honesty** | Pilot numbers **Empirical**; tracker rows **Declared** until flipped with basis |

## Goal

Close residual **oracle poisons** that zeroed pilot `checked_fraction`, remeasure honestly, promote
to staging, and **stop short** of one-shot transpile claims and cz SemVer until CI + readiness.

## Done this cycle (PRs + SHAs)

| PR | Leaf | Result |
|----|------|--------|
| #1645 | prior oracle poisons | cmp 12.6% / rand 17.6% Clean (baseline) |
| #1646 | promote oracle → int | staging caught #1645 era |
| **#1647** | **A1** lit-zero / signed field compare | bare-`0` on `is_negative`/`is_zero` closed |
| **#1648** | **A2** Strength lattice co-emit | `unknown type Strength` closed |
| **#1649** | **A3** M-1006 remeasure | Empirical post-A1/A2 table |
| **#1650** | **A4** DEFAULT_FUEL/DEPTH | **eval myc-check Clean**; expr ~21.4% |
| **#1651** | **A5** wide Show + call-arg BinLit | **std-time checked 0% → 45.9% Clean** |
| **#1652** | Epic I promote | `dev` → `integration` lineage merge |

## Empirical measures

### Post-A1+A2 (A3 artifact — `M1006-remeasure-post-A1A2-2026-07-16.md`)

| Target | checked% | expressible% | File / poison |
|--------|----------|--------------|---------------|
| std-cmp | 12.6% | 12.6% | Clean |
| std-rand | 17.6% | 17.6% | Clean |
| std-time | 0.0% | 45.9% | Show WallInstant (then fixed by A5) |
| eval.rs | 0.0% | 16.7% | DEFAULT_FUEL (then fixed by A4) |
| union (5) | 8.5% | 18.6% | cmp+rand drive numerator |

### Post-A5 (L1 + L0 review re-ran transpile-vet std-time)

| Target | checked% | expressible% | File |
|--------|----------|--------------|------|
| std-time | **45.9%** (17/37) | 45.9% | **Clean** |
| eval | myc-check **ok** | ~21.4% | **Clean** |

Wide Show is **Declared** opaque `"<Binary{N}>"` — not Exact Debug (VR-5).

## Open queue (ranked — next L1 waves)

1. **Phase 2** — tip `checks` on `dev`/`integration` green (runner: `shared-podman-1`; `gha-runner-ctl`).
2. **PM close-out leaf (MAINT-CLOSE-1)** — in flight / this PR: `CHANGELOG` + `issues.yaml` (M-1006 doc_refs / partial status; M-1090 remeasure note) + Doc-Index + land `maint-guide.md`.
3. **Epic B (serial transpile):** M-1084 import net-close → M-1037 conversion residual → optional M-1086 derive.
4. **Optional post-A5 remeasure** — refresh M-1006 table with eval Clean + std-time 45.9%.
5. **M-875** expand-first — **needs-design** (no implement until Accepted).
6. **M-740** compiler `.myc` — separate epic.
7. **Epic R** — cz SemVer **only** when: pilot path honest, remote CI success, L0 authorize, no one-shot over-claim.

## FLAGs (orch-owned)

- [x] `CHANGELOG.md` — ORACLE-R1 A1–A5 net entry (MAINT-CLOSE-1 docs PR)
- [x] `tools/github/issues.yaml` — M-1006 doc_refs + A4/A5 body note; M-1090 stays `todo` (verify-first)
- [x] `docs/Doc-Index.md` — PROGRAM-HANDOFF + maint-guide pointer on gap-analysis row
- [x] Land `maint-guide.md` + this handoff + WAVE-L0 orchestration (MAINT-CLOSE-1)
- [ ] `docs/api-index/` — only if public API symbols changed (transpile internals: usually N/A)
- [ ] Model floor: agent catalog may list only `grok-4.5`; prefer `grok-composer-2.5-fast` and **record actual**

## Blockers / ops

- GitHub Actions API intermittent **503**
- `wsl-cpu-1-tzervas-mycelium` offline; `shared-podman-1` online after prepare+up
- L2 spawn sometimes unavailable in child sessions → L1 implemented L2-owned paths (process debt — prefer worktree L2)

## Next L1 brief (paste-ready)

```
You are L1 under L0. Framework: repo-root maint-guide.md (Phases 0–3).
Model floor: grok-composer-2.5-fast (record actual if different).
Base: origin/dev @ f783d4ce (fetch first).

Wave MAINT-CLOSE-1 (docs/PM only, parallel-OK with pure docs):
- Land maint-guide.md if not on tip
- Land PROGRAM-HANDOFF.md + update gap-analysis README
- CHANGELOG entry for ORACLE-R1 A1–A5 (append-only)
- issues.yaml: M-1006 doc_refs → M1006-remeasure-post-A1A2 + note A4/A5 residual movement;
  do not mark one-shot done; verify-first status flips only
- FLAG Doc-Index if needed
- PR → dev; do not merge
- Spawn L2 for any single-file-owned slice if tool works; docs-only may be L1

Then optional Wave B1 M-1084 only if L0 says go (serial transpile).

Report: PR#, SHA, FLAGs, model used.
```

## Release gate

- [x] Residual oracle A1–A5 on `dev` + `integration`
- [ ] Remote CI green on tip
- [ ] Full post-A5 remeasure committed (optional but preferred)
- [x] issues.yaml + CHANGELOG close-out (MAINT-CLOSE-1; statuses honest — not one-shot)
- [x] No one-shot over-claim
- [ ] L0 authorize SemVer / release history squash

**Status:** **HOLD Epic R.**

## Artifact map

| Doc | Role |
|-----|------|
| `maint-guide.md` | Standing OS (this program’s law) |
| `WAVE-L0-ORCHESTRATION-2026-07-16.md` | L0 wave map A/B/I/R |
| `EXPRESS-ORACLE-BLOCKERS-2026-07-16.md` | Technical residual notes |
| `M1006-remeasure-post-A1A2-2026-07-16.md` | Empirical A3 |
| `WAVE-EXPRESSIBILITY-NEXT.md` | Longer expressibility backlog |
| `.claude/kickoffs/_doc-maintenance.md` | Docs DoD on land |

## Changelog (handoff)

| When | Note |
|------|------|
| 2026-07-16 | Initial program handoff after #1647–#1652; maint-guide adopted as durable OS |
| 2026-07-16 | MAINT-CLOSE-1: land maint-guide + handoff + CHANGELOG/issues/Doc-Index close-out PR → working tier |
