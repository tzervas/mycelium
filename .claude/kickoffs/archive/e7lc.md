# Kickoff `e7lc` — E7 Language Completeness, continuation 2 (`mycelium-l1`)

> Stowed kickoff, UID **`e7lc`** — the **second continuation** of `e7l`/`e7lb` (whose tranches
> M-656→M-661 LANDED). Read `.claude/agent-context.md` + `CLAUDE.md` first (house rules win);
> `e7lb.md` / `e7l.md` carry the landed-tranche detail.

## ⚡ RESUME HERE (2026-06-22 — read first)

**Branch fresh off `main`.** `main` tip is **`e583ff2`** — carries the full e7l/e7lb tranche through
**M-661** (`wild`/FFI floor) **+ DN-21** (unsafe-hardening survey + the M-678 epic). On resume:
`git fetch origin main` → branch a fresh working branch off `origin/main`. `main` is PR-only;
squash-PR per logical unit. The L1 collision files (`token`/`parse`/`checkty`/`elab`/`ambient`)
**serialize** — one editor at a time (mitigation #7).

**▶ FIRST: M-662 (`phylum` + cross-nodule) — design-map → FLAG decisions → implement.** This is an
**architecturally-significant** language feature (it defines the phylum construct + the cross-nodule
model), so **settle the choices with the maintainer FIRST (flag, don't guess** — cf. the M-660 effect
syntax + the M-661 `@std-sys` gate). The loop:

1. **Design-map** (the surface is already scoped below — re-verify + read the spec §s):
   - `phylum` is **lexed but has NO production** (`token.rs` `Tok::Phylum`; "reserved, not yet active").
   - `use path` parses to `Item::Use(Path)`, is carried through `ambient.rs`, but is **ignored by the
     checker** (`checkty.rs:587` — `Item::Use(_) => {}`); v0 is **single-nodule** (`checkty.rs:66`;
     `checkty.rs:1443` "dotted path does not resolve in v0 (single-nodule)").
   - The **cross-nodule orphan rule is STAGED** "with the phylum work" — `checkty.rs:932-934` +
     `checkty.rs:1007-1021` (M-659 deferred it here). **Landing M-662 lands this enforcement.**
   - Specs to read: **DN-06** (nodule/phylum static org), **RFC-0006** (phylum surface syntax?),
     **RFC-0019 §4.5** (cross-nodule orphan rule — normative), **RFC-0007** (nodule kernel semantics),
     **Glossary** (`phylum` = versioned content-addressed library/package), `docs/spec/grammar/mycelium.ebnf`.
2. **FLAG to the maintainer** (`AskUserQuestion`) the 5 architecturally-significant choices below. The
   prior-session design-mapping already scoped them with **spec-grounded recommendations** — present each
   recommendation as the default and let the maintainer confirm/override. All 5 recs are mutually
   compatible (single-file · header · specific-imports · phylum-registry · registration-time orphan check):
   - **Q1 — Source model:** one source file with multiple nodules **vs** multi-file (filesystem-mapped).
     Spec: DN-06 §6 naming is **header-based** (not filename); RFC-0006 elaborates phylum→L1. **Rec:
     single-file v0** — a `parse_phylum()` → `Phylum { path, nodules: Vec<Nodule> }`; multi-file deferred
     until the `mycelium-proj.toml` manifest (DN-06 §6, still *Proposed*) is Accepted.
   - **Q2 — Phylum surface syntax:** a **header** (`// phylum: std`, parallel to `// nodule:`) **vs** a
     construct (`phylum std { … }`). Spec: DN-06 §6 header design; RFC-0006 grammar discipline. **Rec:
     header-based** — phylum is a *metadata grouping*, not a syntax container (keeps the grammar light;
     identity stays per-nodule per ADR-003).
   - **Q3 — `use` resolution:** specific imports (`use std.collections.List`) **vs** wildcard; visibility
     (all-public **vs** a `pub` marker). Spec: S1/S5 never-silent; RFC-0019 §4.5 phylum visibility. **Rec:
     specific imports only; all top-level names public-to-phylum in v0** (no wildcard / no `pub` yet — defer
     to v2); a `use` of a non-existent name is a never-silent `CheckError` (G2).
   - **Q4 — Type registry:** global-per-phylum **qualified** keys (`"std.collections.List"`) **vs**
     per-nodule flattened-on-`use`. Spec: RFC-0007 declarations-as-registry; ADR-003 content-addressed.
     **Rec: global-per-phylum, qualified keys** — `use` binds a name locally → the qualified registry key;
     `check_path` (today `checkty.rs:1443` rejects dotted paths) resolves a single segment via local scope +
     imports (multi-segment qualified syntax deferred).
   - **Q5 — Cross-nodule orphan rule (RFC-0019 §4.5):** centralized 2nd pass **vs** check-at-registration
     with phylum-wide visibility. **Rec: registration-time, phylum-wide visibility** — each nodule's `impl`s
     checked against the phylum registry (trait OR type-head declared in *some* nodule of the phylum ⇒ legal;
     else orphan error). Visibility for the orphan check is **automatic** (no `use` required — it's about
     enforcement *authority*, not namespace). Stays **`Declared`** (VR-5 — RFC-0019 coherence is
     Declared-with-argument; do NOT upgrade).
   - **Staging:** **no new L1 node** (KC-3) — phylum/cross-nodule is an *elaboration artifact*; multi-nodule
     elaborates to the existing per-nodule L1 (linked by `use`→local name bindings). Stage execution if
     needed, à la M-657/659/660/661.
3. **Then Opus leaf → honesty+soundness review → Copilot round → curated squash-PR to `main`.** Reconcile
   the orchestrator files (CHANGELOG, `issues.yaml` M-662→done + the orphan-rule note, **DN-14 §3 row 10**,
   regen `docs/api-index/`) + this kickoff (M-662 → landed). Grammar artifact (`mycelium.ebnf`) + the
   accept/reject conformance corpus **must** gain the phylum/`use` rules (RFC-0006 §4.3, committed L3 surface).

> The design-mapping (prior session, captured above as Q1–Q5) is **done** — the next session goes straight
> to flagging Q1–Q5, then implementing. Implementation path once confirmed: (1) phylum header parse +
> multi-nodule AST, (2) the phylum-wide registry + `use` resolution, (3) generalize the orphan-rule check
> (`checkty.rs:932-1026`), (4) grammar + conformance corpus, (5) a two-nodule cross-`use` accept test.

## Chain (dependency-ordered)
| # | Issue(s) | What | Status |
|---|---|---|---|
| — | M-656…M-661 | generics → traits → effects → wild/FFI | ✅ landed |
| 1 | **M-662** | `phylum` + cross-nodule (+ the cross-nodule orphan rule M-659 deferred) | ✅ **landed** (2026-06-22; single-file phylum · `pub` + glob `use` · phylum-wide orphan; all `Declared`) |
| 2 | M-663 | RFC-0018 static guarantee grading — **stays `Declared`** (VR-5) | **active — next** |
| 3 | M-664 | `consume`/`grow`/`impl` surface keywords (+ fix the stale `lang-lexicon-syntax.md` legend [~l.100] that still lists `impl` reserved-not-lexed) | |
| 4 | M-667 → M-668 | E7-2: `fuse`/`reclaim`/`tier` → R2 design | |
| 5 | M-673 → M-649 | dogfooding: elaboration (RUN) → self-host the first `.myc` nodule → the example phylum | after the lexicon |

## Parallel track — the M-678 unsafe-hardening epic (DN-21; disjoint from L1)
DN-21 (`docs/notes/DN-21-Unsafe-Code-Hardening-Survey.md`) filed **M-678** (epic) → **M-679** (strengthen
the 3 thin JIT-FFI `// SAFETY:` comments + `debug_assert!(!ptr.is_null())`) · **M-680** (forbid-pin the
trusted base + the 11 zero-unsafe `mycelium-mlir` submodules) · **M-681** (`just safety-check` SAFETY-
adjacency gate) · **M-682** (in-house **`Sym<'lib,T>`** lifetime-binding newtype — **maintainer-chosen**
over `libloading`, zero new dep) · **M-683** (document the `audit_wild` `.myc` vs `clippy` `.rs` scope
split). All live in `crates/mycelium-mlir/**` + check tooling — **DISJOINT from the L1 lexicon**, so they
run as a **parallel leaf** alongside the serial L1 chain (e7l lesson: a disjoint file runs parallel). All
behaviour-preserving, zero-new-dependency. See DN-21 §6 (priority/risk table) + §7 (the irreducible unsafe
floor — calling JIT'd fn-ptrs / `dlopen` ctor / the ABI claim; **do NOT** rewrite the SIMD in Rust intrinsics).

## Ownership / swarm method (same as e7lb)
- **You own:** `crates/mycelium-l1/**`, `docs/spec/grammar/**`, the implemented RFC/DN append-only notes, +
  (M-649) one new `.myc` nodule. (The M-678 track adds `crates/mycelium-mlir/**`, `scripts/checks/**`,
  and ADR-014 / Security-Checks-Contract append-only notes.) **Read-only / FLAG up** (orchestrator
  reconciles once per merge): `tools/github/issues.yaml`, `CHANGELOG.md`, `docs/Doc-Index.md`,
  `docs/api-index/`, workspace `Cargo.toml`.
- **Serialize the L1 collision files** (token/parse/checkty/elab/ambient — one editor at a time). Opus
  orchestrator + an Opus leaf per L1-touching task, landed one at a time in dependency order, each pulling
  `main` down first. Per-task loop: **design-map → FLAG arch-significant choices → Opus leaf →
  honesty+soundness review → Copilot round → curated squash-PR**. (Copilot has caught a real bug on every
  kernel PR this wave — review the leaf hard, esp. soundness + never-silent G2.)

## Open follow-ups (filed)
**M-673** (elaboration — monomorphization + trait dictionaries; makes generics/traits RUN) · **M-675**
(idmap full reconcile) · **M-676** (multi-area project field — secondary) · **M-677** (effect →
`mycelium-interp::budget` runtime wiring). The **M-678** epic (above) is the unsafe-hardening track.

## Honesty / done
Every bound at its honest strength; specs → **"implemented Rust-first, pending ratification"** (never
silently `Accepted`); a property/soundness test per bound; never-silent `Result`/`Option`; flag-don't-guess
on arch-significant choices. **Done** = M-662→M-664 + E7-2 + the dogfooding chain (M-673→M-649→example
phylum) landed on `main`, every issue body + status updated. (The M-678 unsafe track is independently done
when its 5 children land.)
