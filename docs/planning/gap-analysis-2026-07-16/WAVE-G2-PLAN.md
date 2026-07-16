# Wave G2 plan — remaining PARTITION crates (2026-07-16)

**Orch mode:** plan only. **All execution:** `grok-composer-2.5-fast` (exact ID).
**Base:** `origin/dev` @ `708bbc14` (post PR #1629 G1 merge).
**Orch branch:** `claude/orch/gap-g2-2026-07-16`
**Status channel:** tg-agent-relay `@mycelium` (do not block on TUI).

## Goal

Exact residual for each remaining crate toward (1) full Rust implementation
completion and (2) transpile-to-Mycelium readiness. G1 covered 12 priority crates;
G2 is the PARTITION remainder (**44 crates**).

## Leaf contract (mandatory — G1 failure lessons)

1. Model: **`grok-composer-2.5-fast` only** (never grok-4.5 / sonnet / haiku / aliases).
2. `isolation: worktree`; branch from **pushed** orch tip `origin/claude/orch/gap-g2-2026-07-16`.
3. Branch name: `claude/leaf/gap-g2-<crate-short>` (e.g. `gap-g2-cert`, `gap-g2-check`).
4. **First action:** create the leaf branch (never commit on orch branch).
5. **Only write:** `docs/planning/gap-analysis-2026-07-16/leaves/<crate>.md`
6. Template: `LEAF-TEMPLATE.md`
7. Prior assessments: G1 SYNTHESIS, language-completeness-gap-inventory, DN-136 phase2
   worklist, DN-99, zero-hand-port-delta-ledger, DN-34, CURRENT-STATE.
8. Commit + push separately; report **branch + SHA + FLAGs**.
9. No `issues.yaml` / `CHANGELOG` / `Doc-Index` / other crates / index regen.

## Crate list (44) — batches of ~10–12

### Batch A — kernel residual (6)
- mycelium-cert
- mycelium-dense
- mycelium-numerics
- mycelium-select
- mycelium-vsa
- mycelium-vsa-decode

### Batch B — runtime residual (4)
- mycelium-rt-abi
- mycelium-sched
- mycelium-stack
- mycelium-workstack

### Batch C — aot (2)
- mycelium-mir-passes
- mycelium-mlir

### Batch D — stdlib residual (19)
- mycelium-std-conformance
- mycelium-std-content
- mycelium-std-dense
- mycelium-std-diag
- mycelium-std-fs
- mycelium-std-math
- mycelium-std-numerics
- mycelium-std-rand
- mycelium-std-recover
- mycelium-std-runtime
- mycelium-std-select
- mycelium-std-spore
- mycelium-std-swap
- mycelium-std-sys
- mycelium-std-sys-host
- mycelium-std-ternary
- mycelium-std-testing
- mycelium-std-time
- mycelium-std-vsa

### Batch E — toolchain (12)
- mycelium-build
- mycelium-check
- mycelium-cli
- mycelium-cli-common
- mycelium-diag
- mycelium-doc
- mycelium-fmt
- mycelium-lint
- mycelium-lsp
- mycelium-proj
- mycelium-sec
- mycelium-spore

### Batch F — bench (1)
- mycelium-bench

## Conductor procedure

1. Push this orch branch first (worktree base = upstream).
2. Fan out Batch A…F sequentially or 2 batches parallel max (resource bound).
3. Each batch: spawn N worktree leaves → wait → verify each leaf file present
   (`git ls-tree` / path count; mitigation #7).
4. Merge leaf refs the **child reports** (`--no-ff`), not assumed names.
5. After all 44: author `SYNTHESIS-G2.md` (rank residual; delta vs G1 SYNTHESIS;
   update work-wave recs).
6. PR orch branch → `dev` (lineage-preserving merge).
7. TG status via `/root/.claude/telegram-bridge/tg-send.sh` at batch boundaries.

## Out of scope for G2

- Code/impl (that is Wave G3 / M-1090 parallel track)
- Shared-file close-out (`CHANGELOG`, issues flip) — integration tier
