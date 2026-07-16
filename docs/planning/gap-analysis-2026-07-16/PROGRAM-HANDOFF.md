# Program handoff — ORACLE-R1 / expressibility residual (2026-07-16)

| Field | Value |
|-------|--------|
| **Framework** | Repo-root **`maint-guide.md`** (Phase 0–3 + L0→L1→L2 + PM close-out) |
| **L0** | Grok session (PM/orchestrator only — no product self-implement) |
| **Phase** | **Close-out / release-hold** — Phase 1 A1–A5 landed; Phase 2 remote CI **one success witnessed** (host); Phase 3 security not yet cycle-end; **Epic R SemVer held** |
| **main** | `aad96b7a` (v0.463.1) |
| **integration** | `41234a14` (promote #1654 after MAINT-CLOSE-1) |
| **dev** | `ba97eb94` (MAINT-CLOSE-1 #1653) |
| **Trees identical?** | **yes** — `dev` and `integration` share tree `40da1e637f6da600c308b94262db1af7bb751a25` |
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
| **#1653** | MAINT-CLOSE-1 | maint-guide + handoff + CHANGELOG/issues/Doc-Index on `dev` |
| **#1654** | promote close-out | `dev` → `integration` (shared tree) |

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

### Phase 2 remote CI — host-witness (Empirical)

| Field | Value |
|-------|--------|
| **Tag** | **Empirical** (host podman logs) — **not** API-bound |
| **Job** | self-hosted `checks` |
| **Result** | **Succeeded** |
| **When** | **2026-07-16 23:30:07Z** |
| **Runner** | `shared-podman-1` |
| **Host log** | `Job checks completed with result: Succeeded` |
| **Phase seen** | just ci → nextest `--workspace` |
| **GH Actions API** | **503 / degraded** at observation — **run id and exact head branch not bound via API** |
| **Do not invent** | No run URL, no fabricated `run_id` |
| **Tip SHA honesty** | Which tip SHA the GH job checked is **Declared** if unknown via API. Host context: runner busy **after** tips `dev=ba97eb94` / `integration=41234a14` (same tree `40da1e63…`) — plausible association only, not a checked binding |

**Release implication:** one remote success is **progressing / one success witnessed** — **not** enough to open Epic R or flip SemVer HOLD.

## Open queue (ranked — next L1 waves)

1. **Phase 2** — re-up ephemeral runner after job exit; re-bind tip `checks` when GH API recovers (confirm run id + head SHA). Runner: `shared-podman-1` via `gha-runner-ctl`.
2. **PM close-out (MAINT-CLOSE-1)** — **landed** #1653 / #1654; residual only if handoff/CI notes lag tips.
3. **Epic B (serial transpile):** M-1084 import net-close → M-1037 conversion residual → optional M-1086 derive.
4. **Optional post-A5 remeasure** — refresh M-1006 table with eval Clean + std-time 45.9%.
5. **M-875** expand-first — **needs-design** (no implement until Accepted).
6. **M-740** compiler `.myc` — separate epic.
7. **Epic R** — cz SemVer **only** when: pilot path honest, remote CI success **tip-bound**, L0 authorize, no one-shot over-claim. **Still HOLD.**

## FLAGs (orch-owned)

- [x] `CHANGELOG.md` — ORACLE-R1 A1–A5 net entry (MAINT-CLOSE-1 docs PR)
- [x] `tools/github/issues.yaml` — M-1006 doc_refs + A4/A5 body note; M-1090 stays `todo` (verify-first)
- [x] `docs/Doc-Index.md` — PROGRAM-HANDOFF + maint-guide pointer on gap-analysis row
- [x] Land `maint-guide.md` + this handoff + WAVE-L0 orchestration (MAINT-CLOSE-1)
- [ ] `docs/api-index/` — only if public API symbols changed (transpile internals: usually N/A)
- [ ] Model floor: agent catalog may list only `grok-4.5`; prefer `grok-composer-2.5-fast` and **record actual**

## Blockers / ops

- GitHub Actions API **503 / degraded** at CI-success observation — cannot bind run id / exact head branch via API
- Ephemeral self-hosted runner **exits after job** — `shared-podman-1` needs **re-up** before next remote check
- `wsl-cpu-1-tzervas-mycelium` offline (prior); host-witness used `shared-podman-1`
- L2 spawn sometimes unavailable in child sessions → L1 implemented L2-owned paths (process debt — prefer worktree L2)

## Next L1 brief (paste-ready)

```
You are L1 under L0. Framework: repo-root maint-guide.md (Phases 0–3).
Model floor: grok-composer-2.5-fast (record actual if different).
Base: origin/dev @ ba97eb94 (fetch first).

Phase 2 residual (ops + honesty):
- When GH Actions API recovers: bind the Succeeded checks run
  (run id + head SHA) for tips after ba97eb94 / 41234a14 — do not invent.
- Re-up shared-podman-1 if the ephemeral runner has exited.
- Do NOT open Epic R / SemVer; HOLD remains.

Optional product next (only if L0 says go):
- Wave B1 M-1084 import net-close (serial transpile)

Report: PR# if any, SHA, FLAGs, model used.
```

## Release gate

- [x] Residual oracle A1–A5 on `dev` + `integration`
- [x] Remote CI **progressing / one success witnessed** (Empirical host podman; 2026-07-16 23:30:07Z)
- [ ] Remote CI green **tip-bound** (API run id + head SHA; re-up runner as needed)
- [ ] Full post-A5 remeasure committed (optional but preferred)
- [x] issues.yaml + CHANGELOG close-out (MAINT-CLOSE-1; statuses honest — not one-shot)
- [x] No one-shot over-claim
- [ ] L0 authorize SemVer / release history squash

**Status:** **HOLD Epic R.** SemVer still **HOLD** — one host-witnessed success is not tip-bound release green.

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
| 2026-07-16 | Phase 2 host-witness: `checks` **Succeeded** @ 23:30:07Z on `shared-podman-1` (Empirical; API 503 — no run URL); tips `dev=ba97eb94` / `int=41234a14`; Epic R still HOLD |
