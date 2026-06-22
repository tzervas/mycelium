# Wave-2 kickoffs — multi-session, head-branch, lossless-merge workflow

The post-1.0 wave is split into **independent parent sessions**, each with a stowed kickoff (UID) and
its own **protected, persistent head branch**. Goal: maximum smooth, conflict-free, non-breaking,
robust parallel development. `main` is **PR-only** (direct push/merge blocked); the head branches are
**protected bases** that persist even when stale working branches are pruned.

## The sessions

| UID | Kickoff | Head branch | Owns (isolated) | Depends on |
|---|---|---|---|---|
| **`e7l`** | `e7l.md` | `claude/head/e7-language` | `crates/mycelium-l1/` · `docs/spec/grammar/` · 1 `.myc` nodule | M-666 on `main` (pull down) |
| **`e7lb`** | `e7lb.md` | *(single branch off `main`; squash-PR per tranche — e7l lesson)* | `crates/mycelium-l1/` · `docs/spec/grammar/` · 1 `.myc` nodule | **continuation of `e7l`** — tranche M-656→**M-661 LANDED**; continued by `e7lc` |
| **`e7lc`** | `e7lc.md` | *(single branch off `main`; squash-PR per task — e7l lesson)* | `crates/mycelium-l1/` · `docs/spec/grammar/` · 1 `.myc` nodule · (M-678 track: `crates/mycelium-mlir/`) | **continuation of `e7lb`** — M-656→M-661 + DN-21 on `main` (tip `e583ff2`); **▶ M-662 next** (design-mapped — Q1–Q5 ready to flag) |
| **`dfr`** | `dfr.md` | `claude/head/dogfood-research` | RFC-0022/0023 Status · `research/12,13` · `research-prompts.md` | — (gates `dfb`) |
| **`dfb`** | `dfb.md` | `claude/head/dogfood-build` | `crates/mycelium-web/` · `crates/mycelium-adk/` (NEW) · root `Cargo.toml` | `dfr` discharge (RP-10/RP-9) |

`e7l` and `dfb` are **fully disjoint** (L1 vs new crates); `dfr` is docs-only. Cross-work continuity is
carried by the **issues** (`depends_on` + body updates), not by touching each other's files.

## Branch / merge pattern (per head)
- Each session works sub-branches off **its head**, merges them **up into the head** (octopus/`--no-ff`,
  pull-down first), and self-integrates there. Swarm method is **scoped per task set** (see each kickoff:
  `e7l` serial-on-L1, `dfr` fractured Opus reasoners, `dfb` parallel-leaf octopus).
- A head **never** PRs to `main` mid-work. When a head is complete + green, that is the only thing it
  PRs.

## Final integration (a later, separate step — the orchestrator)
After the heads complete: octopus-merge the three heads onto an integration branch, **reconcile the
shared files once** (`issues.yaml` dedup, `CHANGELOG`, `Doc-Index`, workspace `Cargo.toml`), run the
full `just check`, then **squash-PR to `main`** (or land each head sequentially with pull-down). This is
"complete + integrate on the heads first, *then* pull down from `main` and merge back up" — safe,
lossless, deconflicted.

## Reserved (excluded from every session)
**M-655** (cut the 1.0.0 tag) and **M-381 / M-646** (LLM local runs) — maintainer-only.
