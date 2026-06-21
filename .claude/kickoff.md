# Mycelium — Session Kickoff Prompt

Read `.claude/agent-context.md` for the compact orientation brief (CLAUDE.md wins on any conflict).

## Mission: close the remaining 1.0.0 gate rows → cut 1.0.0 (kernel/core)

**ADR-021 (the 1.0.0 release-readiness gate) is Accepted.** 1.0.0 = closing its remaining open
Gate-A rows; everything else (surface language, self-hosting, native codegen, JIT, projections,
arms 3/5) is **post-1.0 / 1.x** (ADR-021 §5). The full prioritized roadmap is **`docs/notes/DN-19-Road-to-1.0.0.md`** — read it first.

### Already closed (this is the *remaining* gap list)
A1 (zero open High findings) ✅ · A5 (KC-4 cert-overhead budget ratified: ≤5 µs + ≤2× guardrail) ✅ ·
B1 (RFC-0003/0006/0007 Accepted) ✅ · B2 (KC-2 verdict recorded — determinate retention ratio) ✅.

### Current wave: the 1.0.0 critical path (DN-19 §2)

| ID | Gate | Title | Priority | Parallel? |
|----|------|-------|----------|-----------|
| **M-652** | A4 | Make the `cargo deny`/`cargo audit` gate non-skip — already wired (`deny.toml` + `scripts/checks/deny.sh` + in `all.sh`); provision the tools so it runs green, not skip-passes | P1 | ✅ standalone |
| **M-653** | A2 | Medium-findings ledger: close or explicitly defer every open deep-review Medium (one-line rationale each) | P1 | ✅ swarm (by finding/crate) |
| **M-654** | A3 | WS8 durability: cargo-mutants green on the trusted base + LCG property tests → proptest (seed rotation) + cargo-fuzz targets in CI | P0 | partial (after M-653) |
| **M-655** | ADR-021 | Cut 1.0.0: once A2/A3/A4 close, move ADR-021 `Accepted → Enacted` at the tagged release (per-crate SemVer, ADR-018) | P1 | after M-652–654 |

### Remaining open corpus/impl items (parallel, not 1.0.0-gating)

| ID | Title | Priority |
|----|-------|----------|
| **M-647** | RFC-0020 L2 surface: scoped ratification (§4.2/§4.5 carve-out) | P1 |
| **M-648** | Editorial sweep: landed RFCs → Enacted; Draft DN-04/05/10/12/14/15/17/18 → Resolved where the decision landed | P2 |
| **M-651** | Harness→bench schema bridge: Grok report (`metadata/quality/performance/outcomes`) ingestion in `mycelium-bench` | P2 |
| **M-649** | Self-hosting Stage-2: first stdlib module in `.myc` L1 syntax — **post-1.0** (ADR-021 §5; M-502 gate) | P0 (post-1.0) |

### Backlog (post-1.0, non-blocking)
- **RP-8** — performance optimization + tool-extraction spike (profile → optimize cert-check toward
  the nanosecond target → extract perf kernels; never weaken a guarantee). `docs/notes/research-prompts.md`.
- **arms 3/5 live runs** — M-381 ablation arms (arm 3 needs a local GBNF model; `tools/llm-harness/local/`).

## Swarm guidance

Default mode: **Sonnet Swarm** (all spawned agents Sonnet; CLAUDE.md §Fractal Swarm). Opus orchestrator.

Parallelizable now (disjoint dirs):
- **M-652** (A4) — the gate is already wired (`deny.toml` + `scripts/checks/deny.sh` + in `all.sh`); the work is provisioning `cargo-deny`/`cargo-audit` so it runs green instead of skip-passing (+ policy review); standalone PR.
- **M-653** (A2) — the Medium-findings sweep fans out by finding/crate (each leaf owns its fix + regression test).
- **M-647 / M-648 / M-651** — docs + `mycelium-bench`; independent of the gate rows.
- **M-654** (A3) is the heavy one — start cargo-mutants on the trusted base + the proptest migration after M-653's fixes land (so mutants run against the cleaned tree).

Shared files (orchestrator-owned, read-only for leaves): `CHANGELOG.md`, `docs/Doc-Index.md`,
`tools/github/issues.yaml`, `docs/api-index/`, `justfile`, `docs/planning/phase-*.md`.

## Key invariants for this wave

- **Honesty rule (VR-5)**: every new bound/guarantee tagged at its honestly-supportable strength;
  `Proven` only with a checked basis. Durability work (M-654) must not paper over a real failure.
- **Never-silent (G2)**: every fallible Rust/Python path returns `Option`/`Result`/explicit error.
- **Append-only**: status flips add a resolution record; no rewriting Accepted/Enacted text.
- **Each fix ships a regression test** that fails without it (cite the finding ID) — the WS8 discipline.
- **A2 honesty**: a deferred Medium gets a one-line rationale, never a silent drop.

## Branch and PR flow

```
Branch from main → develop → just check (already runs the cargo-deny/audit gate; M-652 makes it non-skip)
  → pull squashed main down into the branch → PR → squash to main
```

`main` is **never touched directly** — the only write to it is the PR's squash-merge (CLAUDE.md
§Commits & PRs / §Autonomous PR workflow). **Before PR-ing, pull the latest squashed `main` into the
branch** (`git fetch origin main` → merge/rebase → re-`just check`) so the diff is clean and the
squash-merge is conflict-free; in a swarm, propagate the squashed `main` down through every level
after each landing (pull-down flows down, squash-merge flows up). Use `/land` for the final squash.
New IDs: verify the slot is free first (`grep "id: M-655" tools/github/issues.yaml`). After adding
issues, run `python3 tools/github/gh-issues-sync.py --validate` (must be warning-clean).
