# Kickoff `lex` — L1 surface completion (`mycelium-l1`)

> Continues the E7-1/E7-2 L1-surface chain (`e7l`/`e7lb`/`e7lc` landed **M-656 → M-662**: generics ·
> traits · effects · `wild`/FFI · phylum + cross-nodule). Read `.claude/agent-context.md` +
> `CLAUDE.md` (house rules win) + `.claude/kickoffs/README.md` (the tiered `dev → integration → main`
> workflow) first.

## ⚡ RESUME HERE

**Branch off `dev`.** All work lands in `dev` first (messy OK); promote `dev → integration → main`
per the tiered workflow. The L1 collision files (`token`/`parse`/`checkty`/`elab`/`ambient`)
**serialize** — one editor at a time (mitigation #7) — so this is a **serial-on-L1 Sonnet swarm**
(Opus orchestrator + an Opus leaf per L1-touching task, landed one at a time in dependency order).

**▶ FIRST: M-663 — RFC-0018 static guarantee grading** in `checkty.rs` / `elab.rs`. RFC-0018 is
Accepted (2026-06-18) but not enacted: `elab.rs` returns an explicit `Residual` for any `@ g`
guarantee annotation. Implement the **Design-A** graded-typing pass — the `@ g` index becomes a
statically-checked constraint; the checker enforces the lattice `Exact ⊐ Proven ⊐ Empirical ⊐
Declared` and propagates guarantee **meets** at call sites. **Honesty (VR-5): the noninterference
argument stays `Declared-with-argument` — do NOT upgrade.** Acceptance: `fn f(x: Binary{8} @ Exact)
-> Binary{8} @ Exact` type-checks; passing a `@ Empirical` arg is a `CheckError`; `elab.rs` no longer
`Residual`s `@ g`; **DN-14 §3 row 11 → present**; RFC-0018 → Enacted (after the M-648 audit criteria).

## Chain (dependency-ordered)
| # | Issue(s) | What | Status |
|---|---|---|---|
| — | M-656…M-662 | generics → traits → effects → `wild`/FFI → phylum/cross-nodule | ✅ landed |
| 1 | **M-663** | RFC-0018 static guarantee grading — **stays `Declared`** (VR-5) | **active — ▶ first** |
| 2 | M-664 | `consume`/`grow`/`impl` surface keywords (+ fix the stale `.claude/memory/lang-lexicon-syntax.md` legend [~l.100] that still lists `impl` reserved-not-lexed) | next |
| 3 | M-667 → M-668 | E7-2: `fuse`/`reclaim`/`tier` reservation → activation → R2 design | |
| 4 | M-673 → M-649 | dogfooding: elaboration (monomorphization + trait dictionaries → makes generics/traits **RUN**) → self-host the first `.myc` stdlib nodule | after the lexicon (can split into its own kickoff once unblocked) |

## Ownership / method
- **Owns:** `crates/mycelium-l1/**`, `docs/spec/grammar/**`, the implemented RFC/DN append-only
  notes, + (M-649) one new `.myc` nodule. **Read-only / FLAG up** (the integrating parent reconciles):
  `tools/github/issues.yaml`, `CHANGELOG.md`, `docs/Doc-Index.md`, `docs/api-index/`, workspace
  `Cargo.toml`.
- **Serial-on-L1** (one editor of the collision files at a time). Per-task loop: **design-map → FLAG
  the architecturally-significant choices (flag-don't-guess — cf. M-660 effect syntax, M-661
  `@std-sys`, M-662 Q1–Q5) → Opus leaf → honesty + soundness review → Copilot round → land** (feature
  → `dev` → `integration` → `main`). **Copilot has caught a real bug on every kernel PR this wave** —
  review hard, especially soundness + never-silent G2.
- **Honesty / done:** every bound at its honest strength; specs → "implemented Rust-first, pending
  ratification" (never silently `Accepted`); a property/soundness test per bound; never-silent
  `Result`/`Option`. **Done** = M-663 → M-664 + E7-2 (M-667/M-668) + the dogfooding chain (M-673 →
  M-649) landed on `main`, every issue body + status updated.

## Open follow-ups (filed)
**M-675** (idmap reconcile) · **M-676** (multi-area project field — secondary) · **M-677**
(effect → `mycelium-interp::budget` runtime wiring + per-effect budget syntax `retry(<=3)`).
