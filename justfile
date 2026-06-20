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

# Provision the OFF-by-default `mlir-dialect` feature's libMLIR toolchain (apt; may use sudo).
# Deliberately kept OUT of `just setup` so the default never apt-installs or sudo-prompts for an
# optional feature most contributors don't build (ADR-019); run this only if you want that feature.
setup-mlir:
    @bash scripts/setup-mlir.sh

# Full-repo secret scan — gitleaks in --redact mode (allowlist/scope in .gitleaks.toml).
secrets-scan:
    @gitleaks detect --redact --no-banner -c .gitleaks.toml --source .
alias gitleaks := secrets-scan

# Run the FULL local suite. Identical to what CI runs (`just ci`).
check:
    @bash scripts/checks/all.sh

# CI entrypoint — same as `check` (explicit alias used by .github/workflows/checks.yml).
ci: check

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
# Supply-chain gate: cargo-deny (deny.toml) + cargo-audit. Skips if the tools are absent.
deny:
    @bash scripts/checks/deny.sh

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

# --- pre-commit (optional, easy DX) ---
# Install the git hooks so `just check`-equivalent runs on every commit.
hooks:
    @pre-commit install --install-hooks
# Run all pre-commit hooks across the repo now.
pre-commit:
    @pre-commit run --all-files
