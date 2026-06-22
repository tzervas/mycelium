# Kickoff `e7lb` — E7 Language Completeness, continuation (`mycelium-l1`)

> Stowed kickoff, UID **`e7lb`** — the **continuation** of `e7l` (whose first tranche M-656→M-660 LANDED).
> Read `.claude/agent-context.md` + `CLAUDE.md` first (house rules win); `e7l.md` has the full
> landed-tranche detail.

## ⚡ RESUME HERE (2026-06-22 — read first)

**Branch fresh off `main`.** `main` carries the full e7l first tranche through **M-660** + this context
checkpoint. On resume: `git fetch origin main` → branch a fresh working branch off `origin/main`. `main`
is PR-only; squash-PR per logical unit. (Lesson from e7l: a single working branch off `main`, squash-PR'd
per tranche, works cleanly — no separate protected head needed. The L1 collision files
`token`/`parse`/`checkty`/`elab` serialize: one editor at a time, mitigation #7.)

**▶ FIRST: review + land the in-flight M-661 leaf.** M-661 (`wild`/FFI floor) was delegated to an Opus
leaf just before this checkpoint. On resume: `git fetch origin`, find its pushed branch
(`git branch -r | grep -iE 'wild|661|worktree-agent'`), verify scope (`crates/mycelium-l1/**` +
`docs/spec/grammar/**` + RFC-0016/RFC-0006/DN-14 append-only notes — nothing else), then run the loop:
**honesty + SOUNDNESS review → gates (`cargo test -p mycelium-l1`) → reconcile orchestrator files
(CHANGELOG, `issues.yaml` M-661→done, DN-14 §3 row 9, regen `api-index`) → Copilot round → curated
squash-PR to `main`**. (Copilot has caught a real soundness bug on *every* kernel PR this wave — review
the M-661 leaf against the settled design below, especially the std-sys gate + the `wild`⇒`ffi` coverage
wiring.)

**M-661 settled design (maintainer-decided — review the leaf against this):**
- `std-sys` is an **explicit `@std-sys` nodule-header attribute** (NOT a naming convention); `Nodule`
  gains the marker field + parser support (`Tok::At` if needed).
- `wild { … }` type-checks **iff** the nodule is `@std-sys` — else a **hard `CheckError`** (LR-9 /
  RFC-0016 §8-Q6; the issue's "lint warning" was amended to a hard refusal, clean G2). Its body is the
  **trusted/opaque FFI escape** (NOT recursively checked — conforms to the expected type; audited, not
  verified — VR-5/ADR-014). A `wild` in synthesis position (no expected type) → an "ascribe" refusal.
- `wild` is the **`ffi` effect source** (binds to M-660): a fn with a `wild` block must declare `!{ffi}`
  (else M-660's coverage refuses).
- **Execution is STAGED** (`elab.rs` → `Residual`): no FFI host in v0, so `wild` type-checks + gates +
  audits now; running it is a future capability (the issue's "runs" is staged, like M-657/659/660).
  `myc-sec` `audit_wild` is unchanged (the SAFETY-comment audit; the checker is the std-sys context gate).

**Then (full lexicon, in order):** M-662 (`phylum` + cross-nodule import resolution — **also lands the
cross-nodule orphan-rule enforcement M-659 deferred**; `Tok::Phylum` is lexed but has no production yet)
→ M-663 (RFC-0018 static guarantee grading — **stays `Declared`** until a checked basis, VR-5) → M-664
(`consume`/`grow`/`impl` surface keywords; also fix the stale `lang-lexicon-syntax.md` legend [~l.100]
that still lists `impl` as reserved-not-lexed) → **E7-2** M-667/M-668 (`fuse`/`reclaim`/`tier` → R2
design). **THEN dogfooding:** M-673 (elaboration — monomorphization + trait dictionaries; makes
generics/traits actually RUN) → M-649 (self-host the first `.myc` stdlib nodule) → the example phylum.

**Done so far (LANDED to `main`, 2026-06-22 — see `e7l.md` for detail):** M-656/657/658 (generics
spec+checker [elab staged→M-673]; trait spec + `impl` reserved) · **M-659** (trait/impl checker +
coherence; dictionary lowering staged→M-673) · **M-660** (effect annotations `!{eff}` + coverage check;
checker-only, no L0 node) · track-a PM tooling · M-674 depth-safety. Follow-ups: **M-673** (elaboration),
**M-675** (idmap reconcile), **M-676** (multi-area field — secondary), **M-677** (effect→budget runtime wiring).

## Chain (dependency-ordered)
| # | Issue(s) | What | Status |
|---|---|---|---|
| — | M-656…M-660 | generics → traits → effects | ✅ landed |
| 1 | **M-661** | `wild`/FFI floor (audited; `@std-sys`) | **in-flight leaf — review + land FIRST** |
| 2 | M-662 | `phylum` + cross-nodule (+ cross-nodule orphan rule) | next |
| 3 | M-663 | RFC-0018 grading — stays `Declared` (VR-5) | |
| 4 | M-664 | `consume`/`grow`/`impl` surface keywords | |
| 5 | M-667 → M-668 | E7-2: `fuse`/`reclaim`/`tier` → R2 design | |
| 6 | M-673 → M-649 | dogfooding: elaboration (RUN) → self-host first nodule → example phylum | after the lexicon |

## Ownership / swarm method (same as e7l)
- **You own:** `crates/mycelium-l1/**`, `docs/spec/grammar/**`, the implemented RFC/DN append-only notes,
  and (M-649) one new `.myc` nodule. **Read-only / FLAG up** (orchestrator reconciles once per merge):
  `tools/github/issues.yaml`, `CHANGELOG.md`, `docs/Doc-Index.md`, `docs/api-index/`, workspace `Cargo.toml`.
- **Serialize the L1 collision files** (token/parse/checkty/elab — one editor at a time). Opus orchestrator
  + an Opus leaf per L1-touching impl task, landed one at a time in dependency order, each pulling `main`
  down first. Per-task loop: **Explore-map → settle architecturally-significant choices (FLAG to the
  maintainer, don't guess — cf. the M-660 effect syntax + the M-661 std-sys gate) → Opus leaf →
  honesty+soundness review → Copilot round → curated squash-PR to `main`**.

## Honesty / done
Every bound at its honest strength; specs → "implemented Rust-first, pending ratification" (never silently
`Accepted`); a property/soundness test per bound; never-silent `Result`/`Option`; flag-don't-guess on
architecturally-significant choices. **Done** = M-661→M-664 + E7-2 + the dogfooding chain
(M-673→M-649→example phylum) all landed on `main`, every issue body + status updated.
