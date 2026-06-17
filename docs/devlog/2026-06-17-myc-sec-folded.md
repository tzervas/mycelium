# Devlog — 2026-06-17 · Folding `myc-sec` — the check whose job is to never lie green

> **What this is** (see `docs/notes/Narrative-Capture-and-Authoring.md`): the *narrative* layer — the
> messy middle the RFCs smooth over. Append-only, informal, honest.

**Theme.** Fourth tool folded: `myc-sec` (`crates/mycelium-sec`), security checks as tooling. A security
tool's most dangerous failure is a **false green** — saying "clean" when it just didn't look. So the whole
design orbits one rule: **skip ≠ pass.**

---

## 1. The check no scanner gives Mycelium: the `wild`-audit

`wild` is the language's only unsafe escape hatch (LR-9/S6) — denied by default, lexically marked. Off-the-
shelf scanners don't know what a `wild` block is, so this is the genuinely new code: a lexical recogniser
(same family as the M-141 header lints) that **inventories every `wild` block** and requires each to carry
an adjacent **ADR-014 `// SAFETY:`** justification. An unjustified `wild` is a `medium` finding. The honest
scope: it surfaces the author's SAFETY *claim*; it does **not** adjudicate soundness (VR-5 — report the
claim, never fabricate a verdict). I leaned strict on adjacency — a SAFETY comment separated from the
`wild` by a blank line does *not* justify it, because a stale, detached justification is worse than none.
Tests pin the corners: prose `wild`, `wildcard`/`rewild` identifiers, trailing vs preceding SAFETY, the
blank-line break.

## 2. skip ≠ pass, made executable

The existing `scripts/checks/{secrets,deny}.sh` already exit 0 on *success or graceful skip* — which means
exit code alone can't tell "clean" from "the scanner isn't installed." That's exactly the ambiguity a
false green hides in. So `myc-sec` classifies each orchestrated family three ways — **ok / REDUCED /
FAIL** — by reading the scripts' own `ok`/`skip`/`FAIL` markers, and prints a **coverage receipt**
(`FULL`/`REDUCED`) at the end. An overall "no failing findings" with `coverage: REDUCED` is explicitly
*not* a clean bill, and says so. That one line is the whole point of the tool.

## 3. What I deliberately did NOT do

I didn't reimplement gitleaks or cargo-deny in Rust (wrong — duplicates the existing gates), and I didn't
add a new scanner (that's an ADR, per the contract). v0 orchestrates what exists and adds the one check
that's ours. Adding a SAST engine later is a decision, not a build detail.

## 4. What shipped

`crates/mycelium-sec` (lib + `myc-sec` bin), 4 lib tests green; `cargo fmt`/`clippy -D warnings` clean;
the CLI smoke-tested (medium non-strict → exit 0; `--strict` → exit 1). The contract moved Accepted →
enacted. **Four of five M-361 children are now code** (M-364 `mycfmt`, M-368 `spore`, M-365 `myc-check`,
M-367 `myc-sec`). Remaining: **M-366** (lint + auto-fix) — the largest, with the suggest/apply/scaffold
fix model and the dormant §4.1 doc lint.

**Refs:** `crates/mycelium-sec/{src/lib.rs,src/bin/myc-sec.rs}`; `docs/spec/Security-Checks-Contract.md`
(Accepted/enacted); `scripts/checks/{secrets,deny}.sh`; M-367 (#139).
