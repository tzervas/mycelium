# Wave plan — Gap analysis G1 (transpile-ready path)

**Orch mode:** plan only. **All execution:** `grok-composer-2.5-fast`.
**Base:** `origin/claude/orch/gap-analysis-2026-07-16` @ f0091082 (on main tip aad96b7a / v0.463.1)

## Preconditions (conductor)
1. Merge open sync-down PR #1627 (main→integration) if still open.
2. Open+merge sync-down integration→dev (plain --no-ff PR).
3. Confirm gap-analysis orch branch still tip; no dirty accidental deletes.

## Wave G1 — 12 leaves (one crate each, disjoint file ownership)

Each leaf:
- Model: **`grok-composer-2.5-fast`** (exact ID; never grok-4.5)
- `isolation: worktree`
- Branch: `claude/leaf/gap-<crate-short>` from `origin/claude/orch/gap-analysis-2026-07-16`
- **First action:** create leaf branch (never commit on orch branch)
- **Only write:** `docs/planning/gap-analysis-2026-07-16/leaves/<crate>.md`
- Template: `LEAF-TEMPLATE.md`
- Prior assessments: language-completeness-gap-inventory, DN-136-phase2 worklist, DN-99, zero-hand-port-delta-ledger, DN-34, CURRENT-STATE
- Commit + push separately; report branch+SHA+FLAGs
- No issues.yaml / CHANGELOG / other crates

### Leaves
| ID | Crate | Group | Focus |
|---|---|---|---|
| L1 | mycelium-transpile | transpile | DN-136/140 residual; M-1090/1084/1037; checked_fraction |
| L2 | mycelium-l1 | frontend | L1 completeness; self-host M-740; generics/effects residual |
| L3 | mycelium-core | kernel | Core IR / T1 residual; Value/Repr |
| L4 | mycelium-interp | runtime | trusted base completeness; three-way witness |
| L5 | mycelium-std-core | stdlib | port readiness |
| L6 | mycelium-std-fmt | stdlib | M-1090 WU-3 write!/format! |
| L7 | mycelium-std-error | stdlib | Error/Result surface |
| L8 | mycelium-std-io | stdlib | Import/host-effect residual |
| L9 | mycelium-std-collections | stdlib | Vec/map residual |
| L10 | mycelium-std-text | stdlib | Bytes/string residual |
| L11 | mycelium-std-cmp | stdlib | Ord3/derive linkage |
| L12 | mycelium-std-iter | stdlib | HOF/iterator residual |

## Wave G2+ (after G1 integrate)
Remaining crates per PARTITION.md (kernel rest, aot, toolchain, remaining stdlib) — same leaf contract.

## Integration (conductor after G1)
1. Octopus-merge or sequential merge of 12 leaf branches into orch branch (docs-only → conflict-free if one file each).
2. Author `SYNTHESIS.md`: ranked residual to (A) Rust completion (B) transpile readiness; delta vs prior assessments; work-wave recommendations.
3. PR orch branch → `dev`.

## Failure lessons (from failed G1 attempt)
- Leaves must **not** stay on orch branch
- Restore accidental deletes; never commit them
- If spawn fails, retry single leaf with same contract
