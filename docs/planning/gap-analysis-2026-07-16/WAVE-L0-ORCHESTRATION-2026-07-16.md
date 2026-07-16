# L0 orchestration — expressibility residual wave (2026-07-16)

| Field | Value |
|---|---|
| **Framework** | **`maint-guide.md`** (Phase 0 alignment → Phase 1 leaves → Phase 2 validation → Phase 3 security → PM close-out) |
| **Role** | L0 = project/program manager only (no self-implement) |
| **Pattern** | L0 → L1 epic → L2 leaves (worktree-isolated) |
| **Model floor** | L2 **and** L1 prefer `grok-composer-2.5-fast` (record actual if runtime differs) |
| **Durable handoff** | `PROGRAM-HANDOFF.md` (update every pause / before L1 spawn) |
| **Base trunks** | see `PROGRAM-HANDOFF.md` (live tips) |
| **Honesty** | Tracker rows `Declared`; pilot numbers `Empirical` when re-measured |
| **Release gate** | **No** cz SemVer / `main` squash until pilot path + remote CI green honestly |

## L0 standing duties (maint-guide mapping)

| maint-guide | L0 action |
|-------------|-----------|
| Phase 0 | Read corpus + issues; write/update `PROGRAM-HANDOFF.md`; decompose waves |
| Phase 1 | Spawn L1 only — never product code |
| Phase 2 | Watch GHA; require leaf gates; canary/promote policy |
| Phase 3 | Spawn security-reviewer at cycle end before release |
| PM workflow | Review PRs; merge via `gh pr merge --merge`; own CHANGELOG/issues close-out (or L1 docs leaf) |
| Acceptance | Handoff current; no over-claim; shared files reconciled |

## Wave map (see also PROGRAM-HANDOFF open queue)

### Epic A — ORACLE-R1 (**complete** on `dev`/`integration`)

| Leaf | PR | Status |
|------|-----|--------|
| A1 lit-zero | #1647 | merged |
| A2 Strength | #1648 | merged |
| A3 remeasure | #1649 | merged |
| A4 DEFAULT_FUEL | #1650 | merged |
| A5 Show WallInstant | #1651 | merged |
| I promote | #1652 | merged |

### Epic B — NETCLOSE-E1 (next code, serial transpile)

| Leaf | Owns | M-id |
|------|------|------|
| B1 Import net-close | `symtab.rs` + minimal emit | M-1084 |
| B2 Conversion residual | `emit`/`prim_map` | M-1037 |
| B3 Derive rows | derive emit | M-1086 |

### Epic C — design / long-pole

| Item | Status |
|------|--------|
| M-875 expand-first | needs-design |
| M-740 compiler `.myc` | separate epic |

### Epic R — release

Held until `PROGRAM-HANDOFF.md` release gate checklist is complete.

## L1 / L2 contracts

Inject **maint-guide.md** + **PROGRAM-HANDOFF.md** into every L1 prompt. L1 injects leaf ownership +
serial rules into every L2 prompt. L2 opens PR → `dev`; L1 reviews; L0 merges after review.

## See also

- `maint-guide.md`
- `PROGRAM-HANDOFF.md`
- `EXPRESS-ORACLE-BLOCKERS-2026-07-16.md`
- `WAVE-EXPRESSIBILITY-NEXT.md`
- `.claude/kickoffs/_doc-maintenance.md`
