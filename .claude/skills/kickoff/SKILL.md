---
name: kickoff
description: Load and execute a stowed wave kickoff by its short UID (e.g. e7l, dfr, dfb). Reads .claude/kickoffs/<UID>.md + the standing context, then drives that task set on its protected head branch per the Wave-N workflow. Use at the START of a parent session when the user types "/kickoff <uid>" / "run kickoff <uid>" / "begin <uid>" — it is the context-optimized instigator that fires a stowed kickoff in its own session.
---

# /kickoff &lt;UID&gt;

The context-optimized **instigator**: one line starts a parent session on a stowed task set.

1. **Resolve** `<UID>` → read `.claude/kickoffs/<UID>.md` (the task-set brief). If `<UID>` is missing
   or the file doesn't exist, `ls .claude/kickoffs/` and ask which.
2. **Orient (lean):** read `.claude/agent-context.md` (current state + post-compaction handoff); obey
   `CLAUDE.md` (house rules win); skim `.claude/kickoffs/README.md` for the multi-session +
   protected-head workflow. Do **not** re-read the whole corpus — pull targeted context as needed.
3. **Set up the head:** ensure your protected head branch (named in the kickoff) exists and is current
   with `main` (`git fetch origin main` → merge down, or `scripts/sync-heads.sh <head>`). Work
   sub-branches off the head; child branches merge **freely** (octopus/`--no-ff`, no PR); land a
   completed unit onto the head **via a `--no-ff` PR** (lineage preserved — heads never squash).
4. **Execute** the kickoff's task table in order, using the **scoped swarm method** it specifies
   (serial-on-`mycelium-l1` / parallel-leaf octopus / fractured Opus reasoners) and the honesty rules
   (VR-5/G2, append-only, a property test per bound, flag-don't-guess on architecturally-significant
   choices). Pass the resolved model explicitly to every spawned agent; ensure each **pushes before it
   completes** (compaction orphans unpushed work — branches are the durable artifact).
5. **Continuity:** update each issue's status + body as it lands; honor the cross-session gates the
   kickoff names **via the issues**, never by touching another session's files.
6. **Finish:** when the task set is green on the head, it is ready for the final integration to `main`
   (`/wave-land` head→`main` squash-PR; then `scripts/sync-heads.sh` propagates the new `main` down).
   Only `main` squashes.
