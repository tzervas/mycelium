---
name: myc-drafts
description: >-
  Work the committed `.myc` draft corpus (`gen/myc-drafts/`, M-1002/M-1003): regenerate it
  deterministically, triage targets from the manifest, graduate a draft into `lib/` the hand-vetted
  way (M-993), and run an M-1006 ladder phase (bounded target set → rip → patch → record → feed
  lessons back into the transpiler). The corpus honesty contract binds throughout: everything under
  `gen/myc-drafts/` is `Declared` draft material — never imported by `lib/`, never dogfood-gated;
  drafts graduate only via hand-vetted port work with a differential witness.
when_to_use: >-
  Use when starting port work on a semcore module or an unported stdlib crate (triage its draft +
  gap profile first — never port cold), when refreshing the corpus after transpiler changes, or when
  running a phase of the M-1006 whole-corpus ladder. For ad-hoc profiling of arbitrary Rust (no
  committed corpus), use /transpile-vet instead.
allowed-tools: Bash(just myc-drafts-regen:*), Bash(gen/myc-drafts/regenerate.sh:*), Bash(cargo run:*), Bash(cargo build:*), Bash(python3:*), Bash(jq:*)
---

# myc-drafts

The operational form of **E33-1** (kickoff trx2 E-B): the staging tree that turns "hand-port 15k
lines cold" into "start from a vetted draft + a ranked gap profile".

## The corpus

```
gen/myc-drafts/
  README.md            the honesty contract + regeneration command (read it first)
  regenerate.sh        the driver: transpile → vet all targets, per-target subdirs
  manifest_gen.py      pure aggregation over the transpiler's own artifacts
  MANIFEST.md          human triage table (per draft: source + hash, stats, gap counts, vet status)
  manifest.json        the same, machine-readable; pins the source git SHA
  semcore/… stdlib/…   per-target drafts + gap/vet reports
```

Regenerate: `just myc-drafts-regen` (deterministic — byte-identical manifests and full-tree sha256
across runs at the same commit; commit the delta). The manifest pins its **source SHA**: numbers are
true as of that commit, and a transpiler change (e.g. the PR #1207 guard tightening) makes a refresh
worth running, not the old record wrong.

## Triage a port target (before ANY porting)

1. Open its `MANIFEST.md` row: vet status, gap counts by category, `checked_fraction`.
2. Read its `union.gap.json` for the concrete blockers (which items, which categories, why).
3. Decide the split: myc-check-clean draft items are fix-up work; gapped items are **design work**
   (type-coverage/traits/structs — see the DN-34 §8.9 ranked worklist). Scope the port issue to the
   residual, per mitigation #14 (verify against the codebase, not the tracker).

## Graduate a draft into `lib/` (the M-993 path)

A draft NEVER moves by copy alone. The checklist:
1. Copy into the destination nodule (`lib/compiler/semcore.myc` for the semcore SCC — single-nodule
   rule) and adapt to the port conventions (`lib/compiler/README.md`): per-type ctor prefixes
   (FLAG-ast-5/FLAG-parse-2), nodule header, `//` doc conventions.
2. `myc check` clean (per-file; `/myc-dogfood` runs both toolchain witnesses).
3. A **differential vs the Rust oracle** (the `compiler_stage*.rs` harness pattern) — only this
   upgrades the work from `Declared` to `Empirical` (VR-5). New flags are reported up, never
   silently worked around.
4. The draft's manifest row is then out of date on purpose — the next regen reflects reality.
   Never edit `gen/` by hand to "match".

## Run an M-1006 ladder phase

Per the maintainer's two-stage breadth decision (2026-07-06): port surface first (done, wave 1),
then the rest of the Rust corpus **in controlled phases**, each polishing the transpiler:
1. **Mint the phase** (per-wave, mitigation #1): a bounded target list + an M-issue under E33-1.
2. Extend the driver's target list (or a phase-local list) → rip → vet.
3. **Patch**: close the phase's top gap classes in `crates/mycelium-transpile` (per-op honesty
   rules — /transpile-vet §Honesty), and/or patch drafts where the fix is target-local.
4. **Record**: phase manifest (targets, before/after `expressible`/`checked`, fixes landed,
   lessons) + a DN-34 §8.x append-only section. Lessons feed the next phase's priorities.
5. Reconcile with `rwr` at Phase-II: trx2 phase manifests are **inputs** to its port-wave
   manifests (M-947…M-957) — coordinate via `issues.yaml`, never duplicate waves.

## Guard-rails

- **Zero edits under `lib/`** from any corpus run (the drafts stay out of the dogfood gate).
- Committed artifacts carry **no absolute paths** and no churning timestamps (both were real
  wave-1 bugs — keep them fixed).
- A target that fails to transpile entirely gets an explicit `transpile_failed` manifest row —
  never a silent hole (G2).
