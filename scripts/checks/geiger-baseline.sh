#!/usr/bin/env bash
# cargo-geiger unsafe-DEPENDENCY baseline gate (RFC-0041 §5 / RR-29 §4, W0 supply-chain scope).
#
# `just scan` already runs cargo-geiger as a one-shot ADVISORY report (tail of the current unsafe
# count, never fails). This script is the companion DRIFT gate: it records which dependencies
# (workspace + third-party) cargo-geiger reports as containing nonzero `unsafe` usage, and fails
# loudly if a run reports a crate that ISN'T in the committed baseline — so a new unsafe-carrying
# dependency (the prime example: W2 wiring `stacker`/`psm` into `mycelium-stack`, RFC-0041 §4.3) can
# never land silently (G2). An intentional, reviewed addition is a one-line `--update` + a
# THIRD-PARTY-LICENSES.md / about.toml note, not a surprise.
#
# Honesty (VR-5/G2): this is an `Empirical` regex-over-cargo-geiger-Ascii-output heuristic (cargo-geiger
# has no stable machine-readable output this script depends on) — cargo-geiger's own report is ground
# truth, this baseline is a delta-tracker over it. Skip-graceful: cargo-geiger is not required for
# `just check`; if it's absent, this prints a clear skip (never a silent pass) and exits 0.
#
# Usage:
#   scripts/checks/geiger-baseline.sh            # compare the current report to the baseline
#   scripts/checks/geiger-baseline.sh --update   # regenerate the baseline from the current report
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1

BASELINE="$SCRIPT_DIR/geiger-baseline.txt"
section "cargo-geiger unsafe-dependency baseline (RFC-0041 §5 / RR-29 §4)"

if ! command -v cargo-geiger >/dev/null 2>&1; then
  skip "cargo-geiger not installed (\`cargo install cargo-geiger\` or \`just setup-scan\`) — baseline drift check skipped."
  skip "NOTE (G2): the committed baseline ($BASELINE) is a W0 PLACEHOLDER — never generated against a"
  skip "real cargo-geiger run (the tool was unavailable in the W0 authoring sandbox). Flagged for"
  skip "follow-up: install cargo-geiger and run \`$0 --update\` for a real baseline before this gate"
  skip "is relied on to catch a future unsafe dependency (e.g. W2's stacker/psm)."
  exit 0
fi

raw="$(mktemp)"
current="$(mktemp)"
trap 'rm -f "$raw" "$current"' EXIT

if ! cargo geiger --output-format Ascii --all-features >"$raw" 2>/dev/null; then
  fail "cargo geiger failed to run — cannot compare against baseline"
  exit 1
fi

# Extract the set of "crate version" entries cargo-geiger marks with a nonzero-unsafe symbol
# (☢ = unsafe usage found; ❓ = unsafe usage found but locally #![allow]-suppressed / unaudited).
# The dependency name+version is the trailing "name version" pair on each such table row.
grep -E '☢|❓' "$raw" \
  | sed -E 's/^.*[☢❓][[:space:]]+([A-Za-z0-9_-]+[[:space:]]+[0-9][0-9A-Za-z.+-]*).*$/\1/' \
  | sort -u > "$current" || true

if [[ "${1:-}" == "--update" ]]; then
  {
    echo "# cargo-geiger unsafe-dependency baseline (RFC-0041 §5 / RR-29 §4)."
    echo "# One 'crate version' per line: every dependency cargo-geiger reports with nonzero unsafe"
    echo "# usage (☢/❓) as of the last --update run. Regenerate: scripts/checks/geiger-baseline.sh --update."
    echo "# A new line appearing here across a diff IS a decision (new unsafe dep) — review it, note it"
    echo "# in THIRD-PARTY-LICENSES.md/about.toml (pin exact version), THEN update the baseline."
    cat "$current"
  } > "$BASELINE"
  n=$(grep -vc '^#' "$BASELINE" 2>/dev/null || true)
  n=${n:-0}
  ok "baseline regenerated: ${n} unsafe-using dependenc(y/ies) recorded to $BASELINE"
  exit 0
fi

if [[ ! -f "$BASELINE" ]]; then
  fail "no baseline at $BASELINE — run \`$0 --update\` once to establish it"
  exit 1
fi

baseline_entries="$(grep -v '^#' "$BASELINE" 2>/dev/null | grep -v '^[[:space:]]*$' || true)"
new_entries="$(comm -13 <(printf '%s\n' "$baseline_entries" | sort -u) "$current" || true)"

if [[ -n "$new_entries" ]]; then
  fail "NEW unsafe-using dependenc(y/ies) not in the baseline (never-silent, G2 — a new unsafe dep must be a conscious, reviewed decision):"
  printf '        %s\n' "$new_entries"
  echo "        If intentional and reviewed (e.g. W2's stacker/psm), record it in THIRD-PARTY-LICENSES.md"
  echo "        / about.toml (exact-version pin) THEN: $0 --update"
  exit 1
fi

ok "no new unsafe-using dependencies vs. the baseline ($BASELINE)"
