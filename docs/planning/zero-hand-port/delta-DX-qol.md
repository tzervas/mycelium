# Appendix — DX / QoL Track (Transpiler + Toolchain + Port Workflow)

> Supporting inventory for `docs/planning/zero-hand-port-delta-ledger.md` §4.5. Draft, 2026-07-10.
> Grounded evidence backing the ledger's DX-closure ranking; not independently ratified.

**Scope:** the *developer-experience / quality-of-life* gaps that make the
transpile-port-verify-graduate loop slow, opaque, error-prone, or repetitive. **Complements** the
correctness/expressibility ledger (`delta-L3-transpiler.md` plus `delta-L4L5-idiom-structural-DRAFT.md`
plus `delta-L2-engines.md`), which owns *translation-rule* gaps (Import symbol table, macro-expand,
type/record/generic surface). This track deliberately does **not** re-inventory those, a construct
that cannot be *expressed* is a correctness gap; a construct whose *tooling around it* is rough is a
DX gap.

**Grounding (VR-5):** every row cites the source file/line read at analysis time. No fresh
transpiler/build run (a batch was building concurrently). Emission is `Declared`; vet numbers
`Empirical`. **Highest tracked issue id at analysis time = M-1022**; the reconciled ledger and
DN-109 file the new items as **M-1041..M-1047** (the agent's original M-1023.. proposal collided
with ids landed by a concurrent batch; mitigation #1).

**Confirmed-GOOD (not gaps, grounded so we don't re-open them):**

- Output **is deterministic**, `discover_rs_files` sorts (`batch.rs:47`); counts use
  `BTreeMap`; `regenerate.sh` documents byte-for-byte reproducibility at a fixed commit.
- Never-silent **is** satisfied everywhere, every unmapped construct routes to a `GapReason`/
  `Category` (`gap.rs`), stem collisions WARN (`bin:222-229`), parse failures are reported and
  fail (`bin:201-203`), `ToolUnavailable` is never counted clean (`vet.rs:70`).
- The `// nodule:` header plus `@summary` provenance banner **are** emitted (`transpile.rs:449-465`).
- `checked_fraction` vs `expressible_fraction` split **is** surfaced on stdout plus `vet.json`
  (`bin:92-114`, `vet.rs`).
- The per-crate output-subdir workaround **already** contains the flat-emit data-loss hazard at
  the workflow layer (`regenerate.sh:56-57`, each target gets its own out-dir).
- The loop is **operationalized as skills** (`/transpile-vet`, `/myc-drafts`) and the CLI binaries
  are built-once/`MYC_CHECK_CMD` (no per-file `cargo run`, `regenerate.sh:14-17`).

---

## Register

| # | Gap | Area | Severity | Effort | Layer | Tracked? | Closure recommendation |
|---|-----|------|----------|--------|-------|----------|------------------------|
| D1 | **Visitor-DRY meta-gap**, `emit.rs`+`map.rs` are ~15 free `emit_*`/`map_*` fns (`emit_expr_inner` alone = 19 `Expr::` arms; `emit.rs:734`), each a hand-`match` over `syn` node kinds; **no `trait ExprVisitor`/fold**. A new construct = edit every mirror match by hand. Force-multiplier: the M-1006 ladder *folds lessons back into `emit.rs` every phase*, so this tax is paid on the critical path repeatedly. | contributor | **Slows (force-mult)** | L | transpiler | filed as **M-1041** (unifies with the L2 visitor-DRY meta-gap) | Introduce a `fold`/visitor trait (or a `dispatch!` macro) so one new arm = one edit; keep the never-silent catch-all. |
| D2 | **Flat emit / no source-tree mirror**, batch mode writes `<stem>.myc` into ONE flat out-dir, discarding directory structure (`bin:205-238`). | output | Slows | M | transpiler | filed as **M-1042** | Mirror the source tree under out-dir (`<rel/path>.myc`); the CLI, not just `regenerate.sh`, should be collision-safe. |
| D3 | **Stem-collision last-writer-wins (data loss)**, two files sharing a stem (two `mod.rs`) means later overwrites earlier, only a WARNING (`bin:222-229`). Never-silent but still lossy. | output | Slows | S | transpiler | folded into **M-1042** | Path-qualify the emitted filename (part of D2); no overwrite. |
| D3b | **No per-item source breadcrumb in emitted `.myc`**, header exists but no `// src: <file>:<line>` per emitted item; a hand-porter must cross-ref `gap.json` to locate the Rust source. | output | Slows | S | transpiler | filed as **M-1043** | Emit a `// src: file:line` comment above each item (data already in `Gap`/span). Biggest single hand-edit accelerator. |
| D4 | **Emitted `.myc` is not run through `mycfmt`**, output is hand-rolled `chunks.join("\n\n")` (`transpile.rs:462`); spacing/indent is whatever `emit.rs` hardcodes. | output/toolchain | Polish | S | transpiler | folded into **M-1047** | Optional `--fmt` post-pass piping emission through `mycfmt` (M-364) for canonical, readable drafts. |
| D5 | **No dry-run / summary-only mode**, CLI always writes artifacts; can't profile a crate's gap distribution without writing files. | output/workflow | Polish | S | transpiler | folded into **M-1047** | `--summary`/`--dry-run` flag: compute plus print the gap/category/fraction table, write nothing. |
| D6 | **Gap reasons descriptive but not always actionable**, reasons say what/why (`gap.rs`, good on `NamedFieldDrop`) but rarely give the exact hand-idiom to apply. Never-silent OK, but not maximally *helpful*. | diagnostics | Slows | M | transpiler | filed as **M-1045** | Add an optional `suggested_idiom` field to `Gap`; populate for the common classes (record to positional ctor, `use` to FLAG, reserved-word to rename). |
| D7 | **No EXPLAIN for *successful* idiom/selection choices**, `gap.json` records failures only; when the emitter picks String-to-Bytes / positional-ctor there's no per-item EXPLAIN of the choice (house rule #2). | diagnostics | Polish | M | transpiler | subsumed by the **remap manifest** `idiom_choices` field (DN-109 §5.2) | Emit a per-item selection note (why this idiom) as part of the manifest's `idiom_choices`, rather than a separate `explain.json`. |
| D8 | **No "closest-to-clean" investment ranking**, vet is file-gated all-or-nothing (`vet.rs:143`); one poison item zeros a file's credit and the report never says *which* item to fix first, nor ranks files by items-from-clean. | workflow/diagnostics | Slows | S-M | workflow | filed as **M-1046** | Rank vetted files by (emitted minus checked) delta plus surface the first `check-error:` diagnostic per file (already captured, `VetRecord::diagnostic`) as a ranked worklist in the manifest. |
| D9 | **LSP missing providers**, has hover/definition/diagnostics/semantic-tokens/fmt/completions, but **no references, rename, code_action, signature_help**; completions are lexical/scaffolding only (no scope/type awareness, `completions.rs:1-9`). | toolchain | Slows (`.myc` hand-edit) | L | toolchain | **PARTIAL**, E2-5 "Full LSP" epic (`issues.yaml:391`), not decomposed | Decompose the Full-LSP epic into per-provider issues (references, rename, code-action first); not re-filed here (already tracked). |
| D10 | **tree-sitter grammar drift, no visibility rule**, `grammar.js` has **no `pub`/`priv`/visibility node** (grep empty) while the surface has `pub` (M-662; EBNF `mycelium.ebnf:149`). Editor highlighting drifts from the real grammar (the "priv structural gap"). | toolchain | Polish | S | toolchain | **already landed** (M-1039, PR #1385, CHANGELOG 2026-07-10) | No new issue; the grammar.js `priv` gap this row flagged was independently found and closed same-day by the terminal-review fast-follow batch (verified against CHANGELOG.md before filing, mitigation #14). |
| D11 | **Category-enum DRY tax**, adding a `Category` means editing the enum plus `as_str()` match plus keeping the `gap.json` string field in sync (`gap.rs:17-99`); `prim_map.rs` table similarly hand-maintained. | contributor | Polish | S | transpiler | folded into **M-1041** | Derive `as_str` (strum-style) or a single macro; minor, bundle with the visitor-DRY fix. |
| D12 | **No incremental/watch mode; hand-rolled arg parsing (no `--help`)**, re-transpiles everything each run; `env::args` loop, no clap (`bin:33-42`). Corpus is small so low urgency; compounds as flags (D2/D4/D5) accrue. | output/infra | Polish | M | transpiler | folded into **M-1047** | Once >=3 flags land, adopt a minimal arg helper plus `--help`; watch/incremental deferred (YAGNI until corpus grows). |

---

## Counts by severity (12 gaps plus 2 sub-rows folded)

- **Blocks the zero-hand-port program:** 0 (correctness ledger owns the blockers; DX gaps slow,
  not block).
- **Slows (incl. the D1 force-multiplier):** 6, D1, D2, D3(to D2), D3b, D6, D8, D9.
- **Polish:** 6, D4, D5, D7, D10, D11(to D1), D12.

## Tracked vs newly-surfaced

- **Newly-surfaced, filed at integration:** D1/D11 (M-1041), D2/D3 (M-1042), D3b (M-1043), D6
  (M-1045), D8 (M-1046), D4/D5/D12 (M-1047, combined "transpiler DX polish"). D7 subsumed into the
  remap-manifest `idiom_choices` field rather than a standalone issue.
- **Already tracked, no new issue:** D9 (E2-5 Full-LSP epic, undecomposed); D10 (landed same-day
  as M-1039, verified against CHANGELOG before filing, so not re-opened, mitigation #14).
- **Adjacent-already-tracked (correctness ledger, do NOT double-count here):** cross-nodule symbol
  table / project-mode vet (Import, 117 gaps, `delta-L3-transpiler.md` §5.1, adjacent to M-1001)
  is *also* a DX enabler (unblocks multi-file drafts) but is booked as a correctness lever, not
  here.

## Flags / un-groundable

- D10's exact "priv" framing at analysis time: `grammar.js` had **no visibility node at all** and
  the surface uses `pub` (not a `priv` keyword). The "priv structural gap" was real as grammar
  drift; it was independently found and fixed by the terminal-review fast-follow batch (M-1039)
  before this ledger reconciled, so it is recorded here as closed rather than re-filed.
- The visitor-DRY "13 edits across 8 walkers" figure: grounded as ~15 free `emit_*`/`map_*` fns
  and a 19-arm `emit_expr_inner`, no visitor trait, directionally consistent with the L2 appendix's
  independent count; the exact 13/8 figure was not reproduced digit-for-digit across both
  analyses.
