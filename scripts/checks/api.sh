#!/usr/bin/env bash
# Public-API surface gate (KC-3 small auditable kernel; supports A2-05 private-fields work).
# Diffs each crate's public API against a committed snapshot under docs/spec/api/<crate>.txt and
# FAILS on an unreviewed change — so an accidental `pub` (or a widened surface) is caught in review.
# Skips gracefully when cargo-public-api is absent or a crate has no baseline yet; bootstrap or
# update the snapshots with `just api-baseline`. (cargo-public-api drives a nightly rustdoc; that
# nightly is used only to introspect the surface, it does not change the MSRV-pinned build.)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "public-api surface"
rc=0

if ! { [[ -f Cargo.toml ]] && have cargo; }; then skip "no Cargo.toml or cargo"; exit 0; fi
if ! cargo public-api --help >/dev/null 2>&1; then
  skip "cargo-public-api not installed (run \`just setup\`) — surface gate skipped"
  exit 0
fi

# cargo-public-api reads the surface from rustdoc-JSON — a NIGHTLY-only rustdoc feature — so the
# surface is introspected with a nightly toolchain (MYC_API_TOOLCHAIN, default `nightly`), while the
# MSRV-pinned 1.96.1 still builds every real artifact (ADR-041): this gate reads the surface, it
# produces no build output. Pin to a date (MYC_API_TOOLCHAIN=nightly-YYYY-MM-DD) for baselines that
# stay reproducible across image rebuilds. When that toolchain is absent, SKIP (never fail) — a
# missing introspection toolchain leaves the surface *unverified*, exactly like a missing
# cargo-public-api above; it is not a surface regression (G2: honest skip, not a spurious red).
api_toolchain="${MYC_API_TOOLCHAIN:-nightly}"
if ! { have rustup && rustup run "$api_toolchain" rustdoc --version >/dev/null 2>&1; }; then
  skip "no '$api_toolchain' rustdoc (cargo-public-api needs nightly rustdoc-JSON; \`rustup toolchain install $api_toolchain --profile minimal --component rustdoc\` or run scripts/install-tools.sh) — surface gate skipped"
  exit 0
fi

baseline_dir="docs/spec/api"
checked=0
for d in crates/*/ xtask/; do
  [[ -f "${d}Cargo.toml" ]] || continue
  pkg="$(basename "$d")"
  # cargo-public-api introspects a *library* rustdoc; a bin-only crate (no src/lib.rs) has no
  # Rust public-API surface to gate (its surface is its CLI), so skip it — never fail on it.
  if [[ ! -f "${d}src/lib.rs" ]]; then
    skip "$pkg: bin-only (no library target) — public-API gate N/A"
    continue
  fi
  base="$baseline_dir/$pkg.txt"
  if [[ ! -f "$base" ]]; then
    skip "$pkg: no baseline ($base) — run \`just api-baseline\`"
    continue
  fi
  if ! cur="$(cargo public-api --toolchain "$api_toolchain" --package "$pkg" --simplified 2>/dev/null)"; then
    fail "$pkg: cargo public-api failed to build the surface"
    rc=1
    continue
  fi
  if diff -u "$base" <(printf '%s\n' "$cur") >/dev/null 2>&1; then
    ok "$pkg: public API unchanged"
    checked=$((checked + 1))
  else
    fail "$pkg: public API changed vs $base — review and, if intended, \`just api-baseline\`"
    diff -u "$base" <(printf '%s\n' "$cur") || true
    rc=1
  fi
done

if [[ $checked -eq 0 && $rc -eq 0 ]]; then
  skip "no API baselines present yet — run \`just api-baseline\` to establish them"
fi
exit $rc
