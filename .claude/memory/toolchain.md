# Toolchain — Memory File

**Status: Empirical/Declared** — source + contract specs are ground truth; this is an orientation
aid. Not normative.

---

## What it is

The tool crates that sit **above the kernel** (KC-3): parse/typecheck driver, formatter,
linter, security auditor, doc pipeline, packaging, benchmarking, and LSP. Plus the `justfile`
check suite that is the single local↔CI source of truth.

---

## Where it lives

| Crate | Binary | Role | Key contract spec |
|---|---|---|---|
| `mycelium-check` | `myc-check` | Project-aware correctness driver (M-365) | `docs/spec/Myc-Check-Driver-Contract.md` |
| `mycelium-fmt` | `mycfmt` | Identity-preserving formatter (M-364) | `docs/spec/Mycfmt-Formatter-Contract.md` |
| `mycelium-lint` | `myc-lint` | Invariant linter + auto-fix (M-366) | `docs/spec/Lint-and-Autofix-Contract.md` |
| `mycelium-sec` | `myc-sec` | Security checks: `wild`-audit + secrets + supply-chain (M-367) | `docs/spec/Security-Checks-Contract.md` |
| `mycelium-doc` | `myc-doc` | Doc-IR build + §4.1 quality-bar lint (M-363) | `docs/spec/Narrative-Authoring-Pipeline.md` |
| `mycelium-spore` | `spore` | Content-addressed packaging (M-368; ADR-013) | `docs/spec/Spore-Build-and-Publish-Contract.md` |
| `mycelium-bench` | xtask + lib | Honest benchmarking + LLM-report ingestion (E-BENCH) | — |
| `mycelium-lsp` | — | LSP facade + baseline diagnostics + EXPLAIN channel (M-140/362) | RFC-0013 |
| `mycelium-build` | — | Stable-component gate + build certificates (M-311; RFC-0004 §4) | RFC-0004 §4 |
| `mycelium-proj` | — | Project manifest (`mycelium-proj.toml`) + header + inheritance (M-359) | `docs/spec/Nodule-Header-and-Project-Manifest.md` |

All tool crates depend on the kernel (`mycelium-core`/`-l1`/`-interp`) and on nothing the kernel
depends on (KC-3).

---

## `myc-check` — parse + typecheck oracle + CI gate

`crates/mycelium-check/src/lib.rs`. Project-aware driver above the trusted `mycelium_l1::check_nodule`
(the M-210 shared checker). Discovers `.myc` files under a project root, checks the whole phylum,
aggregates findings deterministically (sorted by file; check/src/lib.rs:177).

**Exit codes** (check/src/lib.rs:88–99):
- **0** clean · **2** any parse error · **3** any check error · **5** resolution failure

**`Finding` fields**: `file`, `kind` (Parse|Check), `site`, `message`, `level`, `route`
(check/src/lib.rs:32–47).

**Honesty**: a `CheckError` is a flat `{site, message}` — the driver does NOT fabricate a finer
class (`TypeMismatch` vs `UnresolvedName`) it cannot structurally distinguish (VR-5). Check
refusals route through the M-362 `DiagnosticPolicy` auto-baseline at the umbrella `NotValidated`
class (check/src/lib.rs:133–141). An empty `.myc` source set is an explicit `ResolveError`, never
a silent pass (G2; check/src/lib.rs:191–195).

**API**: `check_sources(&[(String, String)]) -> Report` · `check_project(dir) -> Result<Report,
ResolveError>` · `check_source_default(file, src, out)` (M-644 ergonomic convenience).

---

## `mycfmt` — identity-preserving formatter

`crates/mycelium-fmt/src/lib.rs`. Three invariants, all runtime-guarded + tested:

- **C1 identity-preservation** — `parse(out) == parse(src)`; a mismatch is `FmtError::OutOfScope`,
  never an emitted rewrite (fmt/src/lib.rs:11).
- **C2 idempotence** — `format(format(s)) == format(s)` byte-for-byte.
- **C3 header-preservation** — DN-06 `// nodule:` marker + M-359 `// @key:` header re-emitted
  canonically; malformed header is `FmtError::Header`, never a silent drop (G2/VR-5).

Body is formatted from the **raw parse** (not ambient-resolved twin), so `default paradigm` /
`with paradigm` are PRESERVED (formatting ≠ "expand ambient"; fmt/src/lib.rs:21).

`MYCFMT_VERSION = "mycfmt-0"` — a `[toolchain].format` pin mismatch is refused (fmt/src/lib.rs:33).

---

## `myc-lint` — invariant linter

`crates/mycelium-lint/src/lib.rs`. Surfaces M-141 invariant lints + M-358/M-359 header lints as
**actionable** findings. `FixTier` boundary (lint/src/lib.rs:28–40):

- `Suggest` — printed, reviewable edit; never auto-applied.
- `Apply` — behaviour-preserving edit; applied only on `--fix`.
- `Scaffold` — incomplete skeleton; **never** auto-applied (control-flow changes are the author's
  declared, opt-in choice — A2/I1/I5).

**v0 reality**: every lint fix maps to `Suggest` or `Scaffold`; `--fix` applies nothing in v0
(header canonicalization delegated to `mycfmt`; lint/src/lib.rs:13–16). `myc-lint` v0 cannot
rewrite your code.

**§4.1 doc-quality lint** (M-363 §6, 8 checks) now ACTIVE (Phase 9 Wave B): runs in `mycelium-doc`
over the doc-IR; has its own gate (`myc-doc`); does NOT block the M-366 lint gate.

Key lints from M-141: implicit-swap detection, untagged-bound advisory, header field
canonicalization. All findings are explicit (G2), never silent.

---

## `myc-sec` — security checks

`crates/mycelium-sec/src/lib.rs`. Three families:

1. **`wild`-block audit** — inventories every `wild` block (LR-9/S6; DN-02 §5 denied-by-default
   unsafe); requires each to carry `// SAFETY:` justification (ADR-014). Unjustified `wild` →
   `Severity::Medium` finding. Honestly scoped to *inventory + justification-presence*; does not
   adjudicate soundness (VR-5).
2. **Secrets scan** — orchestrates `scripts/checks/secrets.sh` (gitleaks).
3. **Supply-chain** — orchestrates `scripts/checks/deny.sh` (cargo-deny + cargo-audit).

**`skip ≠ pass`** (the crux): a missing scanner is *reduced coverage*, reported as such, never
folded into a clean bill (sec/src/lib.rs:13).

`Severity` values (sec/src/lib.rs:18–30): `Info | Low | Medium | High | Critical`. Fixed, declared
map — never heuristically scored (VR-5).

---

## `myc-doc` — doc pipeline + quality-bar lint

`crates/mycelium-doc/src/lib.rs`. Architecture: **one content-addressed doc-IR, many renderers**
(spec §3). Corpus (RFCs/ADRs/notes/specs) + code + M-359 nodule-header metadata + JSON schemas →
`ir::DocModel` → HTML, Typst (→ PDF), machine JSON (doc/src/lib.rs:7).

**Generation is projection, not authorship** (G2 analogue): an item that cannot be grounded is
flagged "undocumented," never invented.

**`doc_lint`** (§4.1 quality-bar, 8 checks; `doc/src/doc_lint.rs`): single-template conformance,
navigability, progressive disclosure, checked examples, no-dead-xref, dual-projection parity,
no-hallucinated-prose, legibility/accessibility. These are the checks that activate the dormant
lint named in `mycelium_lint`.

`CHECK_NAMES` — the single source of truth for the 8 check names (doc/src/lib.rs:29).

---

## `spore` — content-addressed packaging

`crates/mycelium-spore/src/lib.rs`. Builds a content-addressed deployable unit from
`mycelium-proj.toml` (ADR-013; DN-06/Glossary). Identity = the content-addressed DAG (source by
hash + resolved dependency edges + germination surface); **metadata is not identity** (ADR-003).
Two builds of identical code+deps → same spore hash regardless of version label.

A missing phylum surface, no sources, or a dep without `hash` → explicit error (G2; spore/src/lib.rs:9).

v0 scope: single project, hash-pinned deps; on-disk encoding is **named-provisional** (M-368 §9.1),
to be superseded when RFC-0008 R2 wire-schema lands. Source hashing: raw-byte BLAKE3.

---

## `mycelium-bench` — honest benchmarking

`crates/mycelium-bench/src/lib.rs`. Measures existing backends against the interpreter (never
changes them). Backends: interpreter (trusted base) · AOT env-machine · JIT · direct-LLVM ·
MLIR-dialect (feature-gated). Verdict taxonomy (bench/src/lib.rs:17–34): `WIN | LOSS | neutral |
correctness-LOSS | capability-LOSS | runtime-error | skip`.

**Honesty**: all numbers are `Empirical` (trial mean + count + spread); capability/skip/error are
`Declared`; NO pre-written perf target (VR-5); a differential divergence from the trusted
interpreter is a recorded correctness LOSS. A **debug build is refused** for perf numbers.

Also ingests LLM-harness report (`llm` module) so language-leverage data (KC-2/SC-5b) sits
alongside execution data.

---

## `mycelium-lsp` — LSP + baseline diagnostics + EXPLAIN

`crates/mycelium-lsp/src/lib.rs`. The minimal toolchain surface (FR-S5): invariant linter (M-141),
canonical formatter (M-142), LSP feedback facade (M-140), auto-baseline diagnostics (M-362,
RFC-0015), the selection EXPLAIN channel (RFC-0005 §4).

Key re-exports (lsp/src/lib.rs:22–50): `derive_baseline`, `DiagnosticPolicy`, `ClassRegistry`,
`present`, `Level`, `ReasonedError` — all consumed by `mycelium-check`.

`ClassRegistry::with_builtins()` — the builtin class set including `NotValidated` (the umbrella
static-check class). `derive_baseline(&registry)` → `DiagnosticPolicy` (the auto-derived baseline;
`explain_baseline` makes it EXPLAIN-able).

---

## `just check` — the single source of truth

`justfile` (lines 27–87). `just check` = `bash scripts/checks/all.sh` = CI (`just ci`). **Local and
CI use identical recipes** — if it's green locally, it's green in CI (CLAUDE.md house rule).

**Skip-graceful**: checks skip (never fail) when a tool or language isn't present yet (`just check`
on a repo with no Rust code still passes).

Recipe map (justfile:38–88):

```text
fmt-check    lint    md       links      doc-currency   doc-status
schema       grammar spell    shell      structured     secrets
test         myc-fmt myc-check myc-sec   myc-lint       myc-spore
proofs       api     doc-index deny
```

**Myc-tool gates** (lines 67–75): `myc-fmt`, `myc-check`, `myc-sec`, `myc-lint` run over real
`.myc` project roots (`mycelium-proj.toml` dirs, excluding `tests/fixtures/`). They skip if
`cargo` is absent.

`just fmt` — auto-format (Rust + Python), writes changes (`scripts/checks/format.sh --fix`).
`just docs-site` — assemble browsable local docsite → `target/docsite/` (justfile:105); advisory,
NOT part of `just check`. WSL: `cd target/docsite && python3 -m http.server 8080`.
`just docs-index` — regenerate committed `docs/api-index/` after any public-API change (justfile:100).

---

## Remote CI policy

`.github/workflows/checks.yml` — **manual-dispatch only** (`workflow_dispatch`), **advisory**
(non-blocking). CLAUDE.md: no `on: push` / `on: pull_request` auto-triggers without an explicit
decision. Remote CI runs the same `just ci`. Do NOT add auto-triggers without an ADR.

---

## Key invariants (honesty)

- **`skip ≠ pass`** — a missing tool/scanner is *reduced coverage*, always reported (G2).
- **VR-5 on tags** — `myc-check` never re-labels a `Declared`/`Empirical` guarantee; the driver
  reports the checker's verdict, never invents a finer class.
- **Never-silent fix** (`myc-lint`) — every fix is reified + opt-in; `Scaffold` fixes are NEVER
  auto-applied; control-flow stays the author's choice.
- **Identity-preservation** (`mycfmt`) — C1 is a runtime guard; a rewrite that changes the AST is
  a `FmtError::OutOfScope`, never emitted.
- **Content-addressed packaging** (`spore`) — identity = source DAG hash, not the version label;
  missing inputs are explicit errors.

---

## Read more

- `crates/mycelium-check/src/lib.rs` — driver, exit codes, Finding/Report types
- `crates/mycelium-fmt/src/lib.rs` — C1/C2/C3, MYCFMT_VERSION
- `crates/mycelium-lint/src/lib.rs` — FixTier, finding structure
- `crates/mycelium-sec/src/lib.rs` — Severity, `wild`-audit, skip-≠-pass
- `crates/mycelium-doc/src/lib.rs` — DocModel, doc_lint, CHECK_NAMES
- `crates/mycelium-spore/src/lib.rs` — SourceFile, spore-hash invariant
- `crates/mycelium-bench/src/lib.rs` — Verdict taxonomy, honesty rules
- `crates/mycelium-lsp/src/lib.rs` — ClassRegistry, DiagnosticPolicy, EXPLAIN channel
- `justfile` — all recipes, tool gates, docs-site
- `docs/spec/Myc-Check-Driver-Contract.md` — project resolution, baseline, exit codes
- `docs/spec/Mycfmt-Formatter-Contract.md` — C1/C2/C3, version pin
- `docs/spec/Lint-and-Autofix-Contract.md` — FixTier discipline
- `docs/spec/Security-Checks-Contract.md` — `wild`-audit scope, severity map
- `docs/spec/Spore-Build-and-Publish-Contract.md` — content-addressed packaging

---

## Gotchas

- `just check` skips gracefully but NEVER silently passes — a skip is reported.
- `myc-lint --fix` applies **nothing** in v0 (all current lints are `Suggest`/`Scaffold`); header
  canonicalization goes through `mycfmt`, not `myc-lint`.
- `myc-doc` has its own CI gate separate from `myc-lint`; the §4.1 doc-quality checks do NOT
  block the M-366 lint gate.
- `mycfmt` formats from the **raw parse** — `default paradigm` / `with paradigm` blocks are
  preserved in output, not expanded (formatting ≠ resolution).
- `spore` v0 uses raw-byte BLAKE3 for source hashing; canonicalized (mycfmt) hashing is a future
  refinement.
- Tool PATH: prefer `just fmt` / `just check`; raw `ruff` may not be on `$PATH` if installed via
  `uv tool install`. Probe: `command -v ruff || ~/.local/bin/ruff` (CLAUDE.md failure-mode §3).
- Do NOT add `on: push` / `on: pull_request` CI triggers — remote CI is manual-dispatch only
  (CLAUDE.md remote CI policy).
