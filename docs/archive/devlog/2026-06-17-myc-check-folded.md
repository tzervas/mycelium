# Devlog — 2026-06-17 · Folding `myc-check` — growing a prototype up without breaking it

> **What this is** (see `docs/notes/Narrative-Capture-and-Authoring.md`): the *narrative* layer — the
> messy middle the RFCs smooth over. Append-only, informal, honest.

**Theme.** Third tool folded: `myc-check` (`crates/mycelium-check`), the correctness/type-check driver.
Unlike `mycfmt` and `spore` (new tools), this one is a *prototype growing up*: a single-file oracle bin
already lived in `mycelium-l1`, wired (in spirit) to the M-002 LLM-leverage harness. The job was to grow
it into a project-aware CI gate **without breaking the contract the harness depends on.**

---

## 1. The binary-name collision, and the honest resolution

Two crates can't both ship a `myc-check` bin (same output filename → cargo collision). So I couldn't just
add a new one alongside the old. I checked first: *nothing* in `scripts/`, `xtask/`, or any automation
references the old bin by path — only a prose doc-comment mentions it. So the honest move was to **move the
prototype up**: port its exact oracle behavior into the new driver bin (exit 2 parse / 3 check /
`--expect-main` / the machine-readable `ok` first line), then delete the old file. "Grow up the prototype"
taken literally — the prototype *becomes* the grown-up tool, with its contract preserved byte-for-byte.

## 2. The honesty call: don't invent a class you can't see

The contract talks about routing `NotValidated`/`TypeMismatch`/`UnresolvedName` through the M-362 baseline.
But the checker's `CheckError` is a **flat `{site, message}`** — it doesn't structurally distinguish a type
mismatch from an unresolved name; the distinction only lives in free-text. The tempting move is to regex
the message and claim a finer class. That's exactly the kind of fabrication VR-5 forbids. So the driver
routes every check refusal at the honest umbrella class `NotValidated` (a real registry builtin), and the
baseline gives it `Medium`/`stream`. Report what you know; never invent a label. If the checker later
grows typed error variants, the routing gets finer for free.

## 3. Aggregation, and refusing the silent empty pass

A CI gate that stops at the first error is annoying; one that silently passes an empty directory is
*dangerous*. So the driver checks **every** `.myc` in the project and reports all findings
deterministically (sorted by file), and a project with **no** sources is an explicit exit-5 resolution
error — because "checked nothing, all clear" is the silent lie G2 exists to prevent. The smoke test walks
it: two nodules, one bad → both files reported, exit 3; empty dir → exit 5.

## 4. The base stayed put

`check_nodule` (the M-210 trusted checker) is untouched. The driver is pure orchestration above it
(KC-3): discover → parse → check → route → aggregate. No new dependency — it composes `mycelium-l1`, the
`mycelium-lsp` baseline, and `mycelium-proj`.

## 5. What shipped

`crates/mycelium-check` (lib + `myc-check` bin), 4 lib tests (clean/parse/check-routed/aggregation) green;
the prototype bin removed (behavior ported); `cargo fmt`/`clippy -D warnings` clean. The contract moved
Accepted → enacted. **Three of five M-361 children are now code** (M-364 `mycfmt`, M-368 `spore`, M-365
`myc-check`). Remaining: M-367 (security) and M-366 (lint+fix).

**Refs:** `crates/mycelium-check/{src/lib.rs,src/bin/myc-check.rs}`; removed
`crates/mycelium-l1/src/bin/myc-check.rs`; `docs/spec/Myc-Check-Driver-Contract.md` (Accepted/enacted);
M-365 (#137).
