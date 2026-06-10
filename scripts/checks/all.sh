#!/usr/bin/env bash
# Run the full local suite. This is the single entrypoint shared by `just check`,
# `just ci`, and the GitHub Actions workflow — guaranteeing local↔CI parity.
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1

checks=(structured shell markdown links schema grammar spell secrets format lint test proofs)
failed=()
for c in "${checks[@]}"; do
  if ! bash "$REPO_ROOT/scripts/checks/$c.sh"; then failed+=("$c"); fi
  echo
done

printf '%s========================================%s\n' "$C_DIM" "$C_RST"
if [[ ${#failed[@]} -eq 0 ]]; then
  printf '%sALL CHECKS PASSED%s (skips are non-fatal)\n' "$C_GRN" "$C_RST"
  exit 0
else
  printf '%sFAILED: %s%s\n' "$C_RED" "${failed[*]}" "$C_RST"
  exit 1
fi
