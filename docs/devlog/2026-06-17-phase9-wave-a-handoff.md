# Phase 9 kickoff — Wave A: close the M-361 toolchain gate

**Purpose.** Dense, tightly-scoped context for the *fresh session* that folds Wave A. Do a wide
scan of only the files named here, then targeted work — design is already done and the decisions
below are **locked**.

## Status at handoff (2026-06-17)

- Branch `claude/upbeat-archimedes-63igb4` holds this session's work; **merge to `main` first**, then
  branch fresh (`claude/mycelium-phase9-<slug>`).
- The M-361 "full-fat toolchain" is **code** — `mycfmt`/`myc-check`/`myc-sec`/`myc-lint`/`spore`
  folded (PR #141, merged). The five children + M-358/359/362 are reconciled to `status:done`
  (kept **open**; the M-361 epic **#132** closes when this gate lands).
- This session also upgraded `gh-issues-sync.py` (idempotent create **and** update, cross-platform /
  PowerShell, `--self-test`) + added `gh-sync-all.ps1`. That is PM tooling, separate from Wave A.
- **Wave A is designed and decided but NOT folded.** That is the fresh session's job.

## Mission

Wire the four folded tools into the CI-parity gate so `just check` actually runs them. **This closes
the M-361 epic (#132).** Go straight to folding; the design-first step is complete.

## Locked decisions (do not re-litigate)

1. **Minimal canonical phylum in A.** Create `examples/<name>/` = a `mycelium-proj.toml` plus 1–2
   canonical `.myc` nodules, authored to PASS all four gates, so the suite is green-and-real now
   (not all-skips). Wave D later expands it into the full end-to-end conformance fixture.
2. **Fail-the-suite posture.** A real finding fails `just check` (like `lint`/`test`); skip when
   cargo/binary/project is absent. Remote `checks.yml` stays manual-dispatch + advisory.
3. **CRITICAL — never gate the intentionally-bad fixtures.** The tools must NEVER run over
   `docs/spec/grammar/conformance/reject/` or `crates/*/tests/fixtures/` (incl. `bad-header.myc`):
   those are must-fail test inputs, and running the tools over them would erroneously turn
   `just check` red. Scope = real project roots (dirs containing `mycelium-proj.toml`) EXCLUDING any
   path under `tests/fixtures/`. With only the new `examples/<name>/` phylum present, that is the
   sole gated root. (Verified this session: `mycfmt --check` flags 11/12 `accept/` fixtures as
   "would reformat", and the `reject/` corpus is must-fail — so the loose corpus is unusable as a gate.)

## Read these — tight scope (wide scan here, nothing else)

- **Conventions to mirror:** `scripts/lib.sh` (`have`/`skip`/`ok`/`fail`/`tracked`; each check exits 0
  on success-or-skip, non-zero on a real failure); `scripts/checks/deny.sh` and
  `scripts/checks/lint.sh` (the `have cargo` skip-gracefully pattern); `scripts/checks/all.sh`
  (append the new checks to the `checks=(...)` array, after `test`).
- **Wiring points:** `justfile` (one recipe per check — add `myc-fmt`/`myc-check`/`myc-sec`/`myc-lint`);
  `.pre-commit-config.yaml` (local hooks delegate to the scripts; use `pass_filenames: false` and a
  `files:` regex).
- **Tool CLIs (verified this session — exit codes matter):**
  - `crates/mycelium-fmt/src/bin/mycfmt.rs`: `mycfmt --check [--explain] [--config <toml>] <file|->...`
    → 0 ok / 1 would-reformat / 2 parse / 3 header / 4 out-of-scope (pin) / 64 usage / 66 io.
    `--check` writes nothing.
  - `crates/mycelium-check/src/bin/myc-check.rs`: `myc-check --project <dir> [--explain]`
    → 0 / 2 parse / 3 check / 5 project-resolution / 64 / 66. Checks every `.myc` under `<dir>`
    (parse + L1 type), baseline-routed.
  - `crates/mycelium-sec/src/bin/myc-sec.rs`:
    `myc-sec --project <dir> [--strict] [--explain] [--no-secrets] [--no-supply-chain]`
    → `wild`-block audit + a FULL/REDUCED coverage receipt (skip != pass).
  - `crates/mycelium-lint/src/bin/myc-lint.rs`: `myc-lint [--project <dir>] [--fix] [--explain] <file|->...`
    → suggest-only; `--fix` applies nothing in v0 (honest).
- **Contracts (only if a detail is unclear):** `docs/spec/Mycfmt-Formatter-Contract.md`,
  `Myc-Check-Driver-Contract.md`, `Security-Checks-Contract.md`, `Lint-and-Autofix-Contract.md`.

## Build order (targeted)

1. Author `examples/<name>/`: the manifest + canonical nodules. Tip: write a nodule, run
   `target/debug/mycfmt --write` to canonicalize it, then confirm `mycfmt --check` is 0. Then confirm
   `myc-check --project examples/<name>` = 0, `myc-sec` clean, `myc-lint` clean.
2. Add `scripts/checks/{myc-fmt,myc-check,myc-sec,myc-lint}.sh`: source `lib.sh`; `have cargo` else
   skip; discover roots = `mycelium-proj.toml` dirs MINUS any `tests/fixtures/` path; invoke
   `cargo run -q -p <crate> --bin <bin> -- ...`; map exit codes to `ok`/`fail`; skip when no root;
   print one honest coverage line each.
3. Wire: append the four to the `all.sh` `checks=(...)` array; add `justfile` recipes; add
   `.pre-commit-config.yaml` local hooks (`files: \.myc$|mycelium-proj\.toml$`, `pass_filenames: false`).
4. Honesty notes per gate: `myc-lint --fix` applies nothing (v0); `myc-check` stops at name-visibility
   (cross-phylum depth is the M-365 deferral); the §4.1 doc-lint stays dormant until M-363 build.
5. `git add -A && bash scripts/checks/all.sh`, then confirm the literal `ALL CHECKS PASSED` line **in
   the log** (a trailing `echo` can mask the real exit code). Commit; update `CHANGELOG.md`; then close
   M-361 **#132** (status:done; the epic is done) and flip it in `tools/github/issues.yaml`.

## Then (subsequent waves — not Wave A)

- **D** — expand `examples/` into the full end-to-end conformance fixture (dogfood the whole suite).
- **B** — M-363 BUILD pipeline (custom doc-IR + Typst + the §4.1 doc-quality lint, which activates
  M-366's dormant `DOC_QUALITY_CHECKS`).
- **C** — named deferrals: L1 effect declarations (unblocks the RFC-0015 §9 lint), M-365 cross-phylum
  depth, mycfmt §10.1/§10.2 (paren + comment-preserving), spore §9.2/§9.3 (richer include + signing).
- Runtime track (alternative): typed-channel follow-ons on `mycelium-mlir::channel`.

## House rules (non-negotiable; see CLAUDE.md)

Honesty lattice Exact ⊐ Proven ⊐ Empirical ⊐ Declared (never upgrade without a checked basis, VR-5);
never-silent G2 (selections/conversions/fixes reified + EXPLAIN-able; out-of-range is an explicit
error); append-only ADR/RFC/DN (supersede, don't rewrite); KC-3 small auditable kernel — tooling lives
ABOVE it, **no kernel change**; `git add -A` before `all.sh`; green before every commit; **no new
dependency** (serde/serde_json/blake3 are pinned — a new one is an ADR, not a build detail); the env
may lack shellcheck/codespell/check-jsonschema/cargo-deny/cargo-audit (the suite skips them gracefully);
commit trailers `Co-Authored-By` + `Claude-Session`.
