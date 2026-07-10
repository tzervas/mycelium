#!/usr/bin/env bash
# (Re)generate the committed public-API snapshots that scripts/checks/api.sh gates against.
# Writes docs/spec/api/<crate>.txt for every workspace crate. Run after an *intended* API change
# (the diff is then a reviewed part of the commit). Requires cargo-public-api (+ a nightly rustdoc).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib.sh"
cd "$REPO_ROOT" || exit 1
section "regenerate public-api baselines"

if ! { [[ -f Cargo.toml ]] && have cargo; }; then skip "no Cargo.toml or cargo"; exit 0; fi
if ! cargo public-api --help >/dev/null 2>&1; then
  skip "cargo-public-api not installed (run \`just setup\`) — cannot generate baselines"
  exit 0
fi

# Introspect the surface with a NIGHTLY toolchain (MYC_API_TOOLCHAIN, default `nightly`) — rustdoc-JSON
# is nightly-only; the MSRV-pinned 1.96.1 still builds every real artifact (ADR-041). The SAME knob
# drives scripts/checks/api.sh, so the baselines are generated with the toolchain the gate reads them
# with. Skip-graceful when the nightly toolchain is absent (matches the gate).
api_toolchain="${MYC_API_TOOLCHAIN:-nightly}"
if ! { have rustup && rustup run "$api_toolchain" rustdoc --version >/dev/null 2>&1; }; then
  skip "no '$api_toolchain' rustdoc (\`rustup toolchain install $api_toolchain --profile minimal --component rustdoc\` or run scripts/install-tools.sh) — cannot generate baselines"
  exit 0
fi

baseline_dir="docs/spec/api"
mkdir -p "$baseline_dir"
for d in crates/*/ xtask/; do
  [[ -f "${d}Cargo.toml" ]] || continue
  pkg="$(basename "$d")"
  # Skip bin-only crates (no src/lib.rs): cargo-public-api introspects a library rustdoc and a
  # binary has no Rust public-API surface to baseline (mirrors the skip in scripts/checks/api.sh).
  if [[ ! -f "${d}src/lib.rs" ]]; then
    skip "$pkg: bin-only (no library target) — no public-API baseline"
    continue
  fi
  if cargo +"$api_toolchain" public-api --package "$pkg" --simplified >"$baseline_dir/$pkg.txt" 2>/dev/null; then
    ok "$pkg → $baseline_dir/$pkg.txt"
  else
    fail "$pkg: cargo public-api failed"
  fi
done
echo
ok "baselines written under $baseline_dir/ — review the diff before committing"
