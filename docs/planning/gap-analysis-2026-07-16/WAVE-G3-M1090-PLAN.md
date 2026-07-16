# Wave G3 plan — M-1090 WU-3 `write!`/`format!` lowering (2026-07-16)

**Orch mode:** plan only. **Execution:** `grok-composer-2.5-fast`.
**Base:** `origin/dev` @ `708bbc14`.
**Branch:** `claude/leaf/M1090-wu3-write-format-lowering`
**Status channel:** tg-agent-relay `@mycelium`.

## Scope (transpile-only)

Implement **M-1090 WU-3** only (DN-127 Accepted; DN-136 B4):

| WU | Status | Location |
|---|---|---|
| WU-1 std `to_dec`/`digit_byte`/`impl Show` | **landed** | `lib/std/fmt.myc` |
| WU-2 `Show` prelude seed | **landed** | (seed_prelude_trait path) |
| WU-3 transpiler `write!`/`format!` lowering | **this leaf** | `crates/mycelium-transpile/` |

**Depends_on M-1081:** `status:done` (unblocked).

**Not in this leaf:** M-1086 (tracker already `done` — re-vet residual only if
touched incidentally; FLAG, do not re-implement). No `io.myc`. No issues.yaml
close-out until DoD green + PR.

## Mechanism (DN-127 §2/§8)

- Pure `render: T → Bytes` (no `&mut Formatter` sink).
- `write!`/`format!` → `bytes_concat` of literal-`Bytes` fragments + `render(argᵢ)`
  (Show dispatch).
- **Alt C first:** pure-literal + already-`Bytes` args emit + check clean;
  then Show dispatch for interpolations.
- Missing Show / unrenderable → **honest gap** (`MacroInvocation` / explicit residual),
  never fabricated text (G2/VR-5).
- Float rendering remains OQ-1 residual (refused never-silently).
- Reuse existing emit surface: `emit/derives/show.rs` already emits `impl Show` forms;
  new work is **macro-invocation recognizer** feeding `bytes_concat` fold.

## Definition of Done (issue + DN-127)

1. Format-string parser + `bytes_concat`-fragment emitter for `write!`/`format!`.
2. Property tests:
   - **T-1** pure-literal `write!` emits + `myc check`s clean
   - **T-2** single Show-able interpolation (e.g. int via Show) emit↔check
   - **T-3** missing Show / unsupported → explicit gap (never silent success)
3. `cargo fmt` / `clippy -D warnings` / `cargo test -p mycelium-transpile` green.
4. Guarantee tags honest (`Declared` until live-oracle witnessed; `Empirical` after).
5. PR → `dev` with Tracking footer for M-1090 (do **not** flip issues.yaml `done`
   until full DoD including re-measure is honest — partial land keeps `todo` or notes residual).

## Procedure

1. Branch from pushed `origin/dev` (or G2 orch if only after merge — prefer **dev tip**).
2. Read DN-127 §2/§6/§8, DN-136 B4, `emit.rs` macro sites, `emit/derives/show.rs`,
   existing `Category::MacroInvocation` gap paths.
3. Tests-first where practical; implement lowering; never-silent gaps.
4. Change-scoped checks only (`-p mycelium-transpile`).
5. Commit small batches; push; open PR → `dev`.
6. TG status at start / after green tests / after PR open.

## FLAGs for orch

- Shared CHANGELOG / issues status / Doc-Index / M-1006 re-measure
- Any need to touch `lib/std/fmt.myc` (should not — WU-1 landed)
