# Mycelium — local checks. One source of truth: pre-commit and CI route through these
# same recipes (which call scripts/checks/*), so local and remote results match exactly.
# Quickstart:  just setup   then   just check
set shell := ["bash", "-uc"]

# List recipes.
default:
    @just --list

# Best-effort install of the check tools (uv tool / npx / pip). Safe to re-run.
setup:
    @bash scripts/install-tools.sh

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
proofs:
    @bash scripts/checks/proofs.sh
api:
    @bash scripts/checks/api.sh
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

# --- pre-commit (optional, easy DX) ---
# Install the git hooks so `just check`-equivalent runs on every commit.
hooks:
    @pre-commit install --install-hooks
# Run all pre-commit hooks across the repo now.
pre-commit:
    @pre-commit run --all-files
