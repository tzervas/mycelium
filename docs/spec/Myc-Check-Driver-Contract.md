# Spec (Proposed) — `myc-check` project-aware driver contract

| Field | Value |
|---|---|
| **Status** | **Accepted** (2026-06-16 — design; **enacted 2026-06-17** by `crates/mycelium-check` — the `myc-check` lib + CLI; §8.1 warning policy ratified). The contract is now code; project resolution, baseline-routed aggregation, and CI exit codes are tested. |
| **Scope** | The contract for growing the `myc-check` prototype into a **project-aware** correctness driver: project resolution (the `mycelium-proj.toml` surface + dependencies), whole-`phylum`/program checking, diagnostic aggregation routed via the M-362 baseline, honest per-op guarantee tags, and CI-usable exit semantics |
| **Depends on** | The `myc-check` prototype (`crates/mycelium-l1/src/bin/myc-check.rs` — `parse`/`check_nodule`, exit codes); the M-210 shared TV checker (`mycelium_l1::{check_nodule, check_and_resolve, CheckError}`); M-359 (`mycelium_proj::{parse_manifest, parse_header, resolve}` — the surface + inheritance); RFC-0013 (structured diagnostics, levels); M-362 / RFC-0015 (`mycelium_lsp::baseline` — the auto-derived `DiagnosticPolicy`, `present`, the class registry); VR-5 (honest tags, never upgraded); G2 (every refusal explicit); KC-3 (above the kernel) |
| **Feeds** | M-361 (the full-fat toolchain — the correctness gate); CI (`scripts/checks/`); M-366 (shares the diagnostic surface) |
| **Grounds on** | `myc-check.rs`; `crates/mycelium-lsp/src/baseline.rs` (the total `class → (level, route)` derivation); `crates/mycelium-lsp/src/lint.rs` (the diagnostic codes); the M-359 resolver (`resolve.rs`) |

## 1. Summary

`myc-check` today is a single-file oracle: `parse` → `check_nodule`, exit `0/2/3`. M-365 grows it into the
**project-aware correctness driver** of the full-fat suite: it resolves a `mycelium-proj.toml` project (its
public `[surface]` and its `[dependencies]`), checks the **whole** `phylum`/program, reports **every**
refusal as an RFC-0013 structured diagnostic **routed through the M-362 auto-baseline**, and exits
**non-zero on any error** so CI can gate on it. It changes nothing about *what* the checker decides — the
M-210 shared checker (`check_nodule`/`check_and_resolve`) remains the trusted base (KC-3); M-365 is the
*driver* that aggregates and presents. The honesty lattice is preserved end-to-end: a value's per-op
guarantee tag (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`) is reported as the checker computed it, **never
upgraded** by the driver (VR-5).

Presented design-first; no driver code lands until acknowledged (the M-365 gate).

## 2. Project resolution contract

Given a starting path, the driver resolves the project deterministically and inspectably:

1. **Find the manifest** — discover `mycelium-proj.toml` upward from the target (or take `--config`); a
   project with no manifest is checked as a bare set of files (single-file mode, the current behaviour),
   reported as such — never silently assumed.
2. **Determine the check set** — for a `phylum`, the nodules reachable from `[surface].exports` plus the
   files named on the CLI; for a `program`/`script`, the entry plus its reachable nodules. The set is
   **printed under `--explain`** (no ambient "which files did it check?").
3. **Resolve dependencies** — `[dependencies]` give the external `phylum` surfaces available to name
   resolution (by hash, ADR-003; the M-368 resolver is reused once it lands). In v0, dependencies are
   resolved for **name visibility** only; a dep that cannot be resolved is an **explicit error**, never a
   silent "unknown name elsewhere".
4. **Header inheritance** — each file's effective header is resolved via the M-359 `resolve` (`in-file >
   manifest`), so a `@deprecated`/`@license` check has the *effective* value with its provenance.

A **cycle** in the nodule/dependency graph is an explicit error (the program is a DAG). Resolution order is
deterministic; the same project yields the same check set and the same diagnostics.

## 3. Diagnostic aggregation (RFC-0013 via the M-362 baseline)

Each refusal — a `ParseError`, a `CheckError` (`NotValidated`/`TypeMismatch`/`UnresolvedName`/…), or a
header error — becomes an RFC-0013 structured diagnostic. The driver:

- **Routes via the auto-baseline.** It derives the M-362 baseline `DiagnosticPolicy` (`derive_baseline`
  over the class registry) and `present`s each refusal through it, so the level/route are the language's
  own honest defaults (additive only — A1; the baseline can never suppress a refusal). The applied baseline
  is `EXPLAIN`-able (`explain_baseline`).
- **Aggregates across the whole check set** — collects every file's diagnostics into one deterministic,
  source-ordered report (not first-error-and-stop): a `phylum` with three bad nodules reports all three.
- **Preserves honest tags (VR-5).** A `Declared`/`Empirical` guarantee on a value is surfaced as the
  checker computed it; the driver never re-labels it. An `unverified-bound` advisory (a `Declared` value)
  is reported, never silently dropped.
- **Never-silent (G2).** Every `NotValidated`/`TypeMismatch`/`UnresolvedName` is an explicit diagnostic
  with a source position and a class; nothing is swallowed or downgraded to a pass.

## 4. Exit semantics (CI-usable)

| Exit | Meaning |
|---|---|
| `0` | every file in the check set parses, checks, and resolves — clean |
| `2` | one or more **parse** errors (syntactic) |
| `3` | one or more **check** errors (type/totality/name/validation) |
| `5` | a **project-resolution** error (no/ambiguous manifest input, unresolved dep, cycle) |
| `64` / `66` | usage / I/O |

A run with *any* error exits non-zero (the CI gate). Warnings (e.g. `unverified-bound`) do **not** fail the
build by default but are always printed; `--deny-warnings` promotes them (opt-in, never silent).

## 5. CLI surface & EXPLAIN

```
myc-check [--config <toml>] [--explain] [--deny-warnings] [--format human|json] <path|file.myc|->...
```

`--explain` prints the resolved check set, the dependency resolution, and the applied baseline policy
(`explain_baseline`) — so "what did it check, against what, and how were diagnostics routed?" is always
answerable (no black box). `--format json` emits the RFC-0013 dual human/machine projection (G11).
Hand-rolled arg parsing — **no new dependency** (reuses `mycelium-l1` + `mycelium-proj` + `mycelium-lsp`'s
diagnostic surface).

## 6. Scope (honest)

v0 drives the **already-checkable** L1 fragment the M-210 checker accepts; it adds **project resolution +
aggregation + baseline routing**, not new checking power. Cross-`phylum` dependency *checking* (verifying a
dep's surface types) rides on M-368's resolver and is v0-limited to **name visibility** (a dep provides
names; deep cross-phylum type-checking is named as a follow-on). The driver lives entirely above the kernel
(KC-3): it calls the trusted checker, never reimplements it.

## 7. Test plan (acceptance gate)

1. **Resolution** — manifest discovery; check-set determination from `[surface]`; single-file fallback
   reported as such; an unresolved dep / a cycle → exit 5.
2. **Aggregation** — a project with multiple bad nodules reports *all* diagnostics, deterministically
   source-ordered; a clean project → exit 0.
3. **Baseline routing** — each class is presented at its `baseline_for_class` level/route; the baseline
   never suppresses a refusal (A1); `--explain` shows the policy.
4. **Honest tags (VR-5)** — a `Declared` value yields the `unverified-bound` advisory; the driver never
   upgrades a tag.
5. **Exit codes** — parse→2, check→3, resolution→5, clean→0; `--deny-warnings` promotes warnings to
   non-zero.
6. **JSON** — `--format json` round-trips the RFC-0013 structured form (G11).

## 8. Open questions (flagged, not decided)

1. **Warning policy default** — **Ratified (2026-06-17): warnings print but do not fail by default;
   `--deny-warnings` is the opt-in CI gate.** A release gate that wants warnings to fail opts in explicitly;
   the default never silently passes (warnings are always printed) and never silently fails (the gate is
   opt-in, not ambient).
2. **Cross-phylum depth** — v0 resolves dep *names*; deep cross-phylum type-checking is deferred to a
   follow-on once M-368's resolver lands. Confirm v0 may stop at name visibility.
3. **JSON schema** — reuse the RFC-0013 diagnostic JSON shape; confirm no new schema is needed here.

## Meta — changelog

- **2026-06-16 — Proposed (M-365 design).** The project-aware correctness driver contract, design-first.
  Grows `myc-check` from a single-file oracle into the suite's correctness gate: deterministic **project
  resolution** (manifest surface + dependencies + M-359 header inheritance; missing/ambiguous input,
  unresolved dep, or cycle → explicit exit 5), **whole-`phylum` diagnostic aggregation** routed through the
  **M-362 auto-baseline** (`derive_baseline`/`present`/`explain_baseline` — additive-only A1, EXPLAIN-able),
  **honest per-op tags preserved** (VR-5 — never upgraded), and **CI exit semantics** (non-zero on any
  error; opt-in `--deny-warnings`). The trusted M-210 checker is unchanged — this is the driver above it
  (KC-3); **no new dependency**. No code lands until acknowledged. Append-only.
- **2026-06-17 — Open question §8.1 ratified.** Warnings **print but do not fail** the build by default;
  `--deny-warnings` remains the opt-in CI gate. §8.2 (cross-phylum depth) and §8.3 (JSON schema reuse)
  remain deferred to the first implementation pass. Append-only.
- **2026-06-17 — Accepted (enacted by `crates/mycelium-check`, M-365).** The contract is now code: the
  driver lib (`check_project`/`check_sources`/`check_source`/`Report`) + the `myc-check` CLI — **no new
  dependency** (reuses `mycelium-l1` + the `mycelium-lsp` M-362 baseline + `mycelium-proj`; KC-3). The
  prototype **grew up in place**: the single-file **oracle** mode (the M-002/KC-2 harness contract — exit
  2/3, `--expect-main`, `ok`/`parse-error:`/`check-error:`) is preserved verbatim (the old
  `mycelium-l1` bin is removed; its behavior ported), and a **`--project`/`--config` mode** added that
  walks the whole project, **aggregates** every refusal deterministically (all files, not first-error),
  routes **check** refusals through the **M-362 baseline** at the umbrella `NotValidated` class
  (`Medium`/`stream`; additive-only — never suppressed, A1), and exits **2 parse / 3 check / 5
  resolution / 0 clean** (CI-usable). Honest: the flat `CheckError` is **not** split into a finer class it
  cannot structurally distinguish (VR-5); a project with no `.myc` sources is an explicit exit-5
  resolution error, never a silent empty pass (G2). v0 scope: name-visibility dependency resolution and
  the warning default (§8.1) as ratified; deep cross-phylum checking + JSON output deferred (§8.2/§8.3).
  Append-only.
