# Mycelium — local checks. One source of truth: pre-commit and CI route through these
# same recipes (which call scripts/checks/*), so local and remote results match exactly.
# Quickstart:  just setup   then   just check
set shell := ["bash", "-uc"]

# List recipes.
default:
    @just --list

# One-command, idempotent, parameterized install of the dev environment + toolchains
# (rust · python/uv · check tools · pre-commit hooks). `bash scripts/install.sh --help` lists
# components; `--mlir` adds libMLIR (ADR-019, opt-in). Safe to re-run.
setup:
    @bash scripts/install.sh

# Provision the OFF-by-default `mlir-dialect` feature's libMLIR toolchain (nala/apt; may use sudo).
# Deliberately kept OUT of `just setup` so the default never apt-installs or sudo-prompts for an
# optional feature most contributors don't build (ADR-019); run this only if you want that feature.
# Idempotent — safe to re-run; a second run on a provisioned box is an all-present no-op.
setup-mlir:
    @bash scripts/setup-mlir.sh

# Install gitleaks (the secret-scan tool) via apt — KEPT OUT of `just setup` (same ADR-019 principle
# as setup-mlir: the default never apt-installs or sudo-prompts). Use in environments where the
# GitHub-release install is blocked — e.g. a repo-scoped Claude-Code-on-the-web session, where the
# proxy 403s the release download — so the secret-scan gate gets FULL gitleaks coverage instead of
# `scripts/checks/secrets.sh`'s minimal fallback. Best-effort + idempotent (no-op if already present).
setup-secrets:
    @command -v gitleaks >/dev/null 2>&1 && echo "gitleaks already installed ($(gitleaks version 2>/dev/null))" || \
      { (command -v sudo >/dev/null 2>&1 && sudo apt-get install -y gitleaks) || apt-get install -y gitleaks; } \
      || echo "gitleaks install skipped (no apt/permission) — secret-scan uses the minimal fallback"

# Full-repo secret scan — gitleaks in --redact mode (allowlist/scope in .gitleaks.toml).
secrets-scan:
    @gitleaks detect --redact --no-banner -c .gitleaks.toml --source .
alias gitleaks := secrets-scan

# ── Tiered testing (DN-20) ────────────────────────────────────────────────────────────────────
# Three change-scoped, heavy-gated tiers so the everyday loop is fast while release stays durable.
# Honesty (VR-5): no property/bound test is ever dropped — only its CASE COUNT is tiered (low every
# commit, full on release). Coverage is focused, never removed. See docs/notes/DN-20.

# Tier 0 — pre-commit / fastest. ONLY the change-scoped crates' unit + regression/witness tests
# (no integration, no proptest, no doctests, no mutation/fuzz). Ultra-fast feedback.
test-fast:
    @MYC_TEST_TIER=fast bash scripts/checks/test.sh

# Run the FULL local non-test gate suite + Tier-1 tests. Identical to what CI runs (`just ci`).
# Tier 1 (local↔CI parity): change-scoped crates (+ reverse-deps) — unit + regression/witness +
# integration + proptest at LOW cases — PLUS every always-on non-test gate (fmt, clippy, markdown,
# doc-status, doc_refs, deny/audit, …). Skips mutation + fuzz (those are `just check-full`).
check:
    @MYC_TEST_TIER=check PROPTEST_CASES=${PROPTEST_CASES:-8} bash scripts/checks/all.sh

# CI entrypoint — same as `check` (explicit alias used by .github/workflows/checks.yml).
ci: check

# Tier 2 — release / nightly / durability. The FULL workspace at HIGH proptest cases, PLUS the
# heavy durability gates: cargo-mutants (`just mutants`) + a cargo-fuzz smoke (scripts/checks/
# fuzz-smoke.sh, which wraps `just fuzz` skip-gracefully — a missing nightly/cargo-fuzz skips, never
# hard-fails). This is the M-654 WS8 durability gate. SLOW by design — run deliberately (release/nightly).
check-full:
    @echo "── check-full (Tier 2: full workspace · high proptest cases · mutants · fuzz) ──"
    @MYC_TEST_TIER=full PROPTEST_CASES=${PROPTEST_CASES:-256} bash scripts/checks/all.sh
    @echo "── durability: cargo-mutants (trusted base) ──"
    @just mutants
    @echo "── durability: cargo-fuzz smoke (first target, 60s; needs nightly + cargo-fuzz) ──"
    @bash scripts/checks/fuzz-smoke.sh

# ADVISORY supplementary scanners (opt-in — NOT part of `just check`): extra supply-chain +
# code-quality coverage runnable fully in-env with no CI runners. osv-scanner (OSV.dev supply-chain
# — works where cargo-audit's RustSec git-fetch 403s), cargo-geiger (unsafe audit, ADR-014),
# cargo-hack (feature-powerset on mycelium-mlir). Each skips gracefully when absent. Install: `just setup-scan`.
scan:
    @bash scripts/checks/scan.sh

# Best-effort install of the `just scan` advisory tools (osv-scanner via Go; cargo-hack/geiger/machete via cargo).
setup-scan:
    @go install github.com/google/osv-scanner/v2/cmd/osv-scanner@latest 2>/dev/null && echo "  ok    osv-scanner" || echo "  skip  osv-scanner (needs Go)"
    @cargo install --locked cargo-hack 2>/dev/null && echo "  ok    cargo-hack" || echo "  skip  cargo-hack"
    @cargo install --locked cargo-geiger 2>/dev/null && echo "  ok    cargo-geiger" || echo "  skip  cargo-geiger"
    @cargo install --locked cargo-machete 2>/dev/null && echo "  ok    cargo-machete" || echo "  skip  cargo-machete"

# Auto-format code (rust + python). Writes changes.
fmt:
    @bash scripts/checks/format.sh --fix

# --- individual checks (all called by `just check`) ---
fmt-check:
    @bash scripts/checks/format.sh
lint:
    @bash scripts/checks/lint.sh
md:
    @bash scripts/checks/markdown.sh
links:
    @bash scripts/checks/links.sh
doc-currency:
    @bash scripts/checks/doc-currency.sh
doc-status:
    @bash scripts/checks/doc-status.sh
schema:
    @bash scripts/checks/schema.sh
grammar:
    @bash scripts/checks/grammar.sh
spell:
    @bash scripts/checks/spell.sh
shell:
    @bash scripts/checks/shell.sh
structured:
    @bash scripts/checks/structured.sh
secrets:
    @bash scripts/checks/secrets.sh
# `// SAFETY:`-adjacency gate (ADR-014 §8.1 / DN-21 §5 F-3 / M-681): every Rust `unsafe` under
# crates/ must carry an adjacent `// SAFETY:` justification. Pure git-grep — no toolchain, never skips.
safety-check:
    @bash scripts/checks/safety.sh
alias safety := safety-check
# Branch-protection guard (rsm): refuse work on a protected branch (main/integration/dev/claude/head/*)
# and keep commits on the working branch. Idempotent + parameterized; also wired as a git
# pre-commit/pre-push hook (.pre-commit-config.yaml) + a Claude PreToolUse hook (.claude/settings.json).
branch-guard:
    @bash scripts/checks/branch-guard.sh
alias bg := branch-guard
# worktree-guard: assert the worktree-isolation discipline — one isolated worktree per concurrent
# agent; the orchestrator's main tree a clean pointer. Idempotent + parameterized (--leaf /
# --orchestrator / --quiet); the worktree analogue of branch-guard (CLAUDE.md mitigation #11).
worktree-guard *ARGS:
    @bash scripts/checks/worktree-guard.sh {{ARGS}}
alias wg := worktree-guard
# Per-use unsafe escape gate (M-793; RFC-0034 §9; sharpens ADR-014): (A) trusted-kernel crates
# (`mycelium-core`, `-cert`, `-numerics`, `-vsa`) must retain `#![forbid(unsafe_code)]`; (B) every
# non-kernel `unsafe` site must carry a per-use `#[allow(unsafe_code)]` (or `cfg_attr` form) within
# 12 lines above — no crate-global `#![allow(unsafe_code)]` accepted. Pure git-grep, never skips.
unsafe-per-use-check:
    @bash scripts/checks/unsafe-per-use.sh
alias unsafe-per-use := unsafe-per-use-check
test:
    @bash scripts/checks/test.sh
# Mycelium toolchain gates (M-361): canonical format, type-check, security audit, lint over
# the real project roots (mycelium-proj.toml dirs, excluding tests/fixtures/). Skip if cargo absent.
myc-fmt:
    @bash scripts/checks/myc-fmt.sh
myc-check:
    @bash scripts/checks/myc-check.sh
myc-sec:
    @bash scripts/checks/myc-sec.sh
myc-lint:
    @bash scripts/checks/myc-lint.sh
# Non-gating packaging smoke (M-368): `spore build` over each root; always exits 0.
myc-spore:
    @bash scripts/checks/myc-spore.sh
proofs:
    @bash scripts/checks/proofs.sh
api:
    @bash scripts/checks/api.sh
# Drift gate: committed docs/api-index/ must match a fresh regeneration. Skip if python3 absent.
doc-index:
    @bash scripts/checks/doc-index.sh

# (Re)generate THIRD-PARTY-LICENSES.md from Cargo.lock via cargo-about (about.toml + about.hbs).
# Run after any dependency bump/add/remove; commit the result. Needs cargo-about:
# `cargo install cargo-about --locked --features cli` (or `just setup`).
licenses:
    @cargo about generate --workspace --fail about.hbs -o THIRD-PARTY-LICENSES.md \
      && printf '%s\n' "$(cat THIRD-PARTY-LICENSES.md)" > THIRD-PARTY-LICENSES.md \
      && echo "  ok    THIRD-PARTY-LICENSES.md regenerated — review the diff and commit" \
      || echo "  FAIL  cargo-about not installed or generation failed — cargo install cargo-about --locked --features cli"
alias third-party-licenses := licenses
# Drift gate: committed THIRD-PARTY-LICENSES.md must match a fresh `just licenses` regeneration.
# Skip-graceful if cargo-about is absent (same pattern as `deny`/`doc-index`).
licenses-check:
    @bash scripts/checks/licenses.sh
# Supply-chain gate: cargo-deny (deny.toml) + cargo-audit. Skips if the tools are absent.
deny:
    @bash scripts/checks/deny.sh
# PERSISTENT, eyes-open fix for the in-env git-proxy hijack of cargo-deny/cargo-audit (web/remote
# execution env only). The session-injected `insteadOf` rewrite over-broadly routes the PUBLIC
# RustSec advisory-db git fetch through the scoped git proxy, which 403s ⇒ a FALSE `deny` red (NOT a
# finding — see scripts/checks/deny.sh header + .claude/memory/toolchain.md). This installs a SCOPED
# longest-prefix override for `https://github.com/RustSec/` ONLY, so just that one public repo uses
# the ALLOWED general-HTTPS path (proven reachable; TLS + HTTPS_PROXY untouched — never a blanket
# github.com un-rewrite). Idempotent. Session-scoped (the env re-injects git config on each fresh
# container — re-run then). After this, `cargo deny` / `just deny` / `just check` run reliably.
deny-net-fix:
    @git config --global url."https://github.com/RustSec/".insteadOf "https://github.com/RustSec/" \
      && echo "  ok    scoped RustSec/advisory-db fetch enabled via allowed HTTPS path — \`just deny\` now runs reliably" \
      || echo "  FAIL  could not apply the scoped git-config override"

# Editor-grammar drift gate (M-731; RFC-0026): committed tools/grammar/ must match a fresh
# regeneration from the lexer keyword() table (G2 — never a silent divergence). Skip if python3 absent.
drift-check:
    @bash scripts/checks/drift.sh
alias drift := drift-check
# (Re)generate the committed editor grammars (TextMate + tree-sitter) from the lexer keyword()
# table; commit the result. Run after any change to crates/mycelium-l1/src/token.rs::keyword().
grammar-gen:
    @python3 tools/grammar/generate.py
# Reproducible-distribution self-test (M-734): proves the pin/verify/install mechanism is
# byte-identical on re-install and never-silent on a tampered/missing artifact. Deliberately NOT in
# `just check` (it needs a hasher and is a release-engineering gate); run it before cutting a dist.
dist-verify:
    @bash scripts/dist/verify.sh

# --- durability / WS8 (M-654; opt-in, deliberately NOT part of `just check`) ---
# Mutation testing on the trusted base. SLOW (re-runs the suite per mutant) — run deliberately.
# Every surviving mutant is a missing/weak test: kill it with a regression test or justify it.
# `just mutants` = the four trusted-base crates; override the args to scope, e.g.
# `just mutants -p mycelium-cert`. Needs cargo-mutants (`cargo install --locked cargo-mutants`).
mutants *ARGS="-p mycelium-core -p mycelium-cert -p mycelium-interp -p mycelium-numerics":
    @cargo mutants {{ARGS}}
# cargo-fuzz targets (libFuzzer). Needs NIGHTLY: `rustup toolchain install nightly` +
# `cargo install --locked cargo-fuzz`. Targets live in fuzz/fuzz_targets/. `just fuzz <target> [secs]`
# smoke-runs one; `just fuzz-list` lists them. The pinned stable default (rust-toolchain.toml) is
# untouched — fuzzing uses `+nightly` explicitly.
fuzz target secs="60":
    @cargo +nightly fuzz run {{target}} -- -max_total_time={{secs}}
fuzz-list:
    @cargo +nightly fuzz list

# --- code map / observability (advisory; not gating) ---
# Generate code maps (crate deps, module structure, rustdoc incl. private) under target/map/.
map:
    @bash scripts/map.sh
# (Re)generate the committed public-API snapshots under docs/spec/api/ after an intended change.
api-baseline:
    @bash scripts/api-baseline.sh
# Build rustdoc HTML locally (NOT committed — output in target/doc/).
docs:
    cargo doc --workspace --no-deps
# Regenerate committed agent index (docs/api-index/); commit the result after any public-API change.
docs-index:
    python3 tools/docgen/code_index.py
# Assemble a browsable local docsite under target/docsite/ — corpus (myc-doc HTML), agent API
# index, and rustdoc. Advisory, NOT part of `just check`. Skip-graceful: missing tools warn only.
# WSL: cd target/docsite && python3 -m http.server 8080, then open http://localhost:8080.
docs-site:
    @bash scripts/docsite.sh
# Build the curated, chaptered myc-doc BOOK (M-363 output (b)) — one linear reading order over the
# corpus with prev/next nav + a client-side search index, driven by docs/book-manifest.json.
# Advisory, NOT part of `just check`. Output under target/doc-book/book/ (gitignored).
docs-book:
    cargo run -q -p mycelium-doc --bin myc-doc -- book --repo-root . --out target/doc-book
# Build the local docs Podman/Docker container (corpus + book + rustdoc + agent index, served via
# python3 -m http.server). Advisory. Prefers podman, falls back to docker, errors clearly if neither.
docs-container-build:
    @bash scripts/docs-container.sh build
# Run the built docs container, serving on http://localhost:8080.
docs-container-run:
    @bash scripts/docs-container.sh run

# --- pre-commit (optional, easy DX) ---
# Install the git hooks so `just check`-equivalent runs on every commit.
hooks:
    @pre-commit install --install-hooks
# Run all pre-commit hooks across the repo now.
pre-commit:
    @pre-commit run --all-files
