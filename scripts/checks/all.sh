#!/usr/bin/env bash
# Run the full local suite. This is the single entrypoint shared by `just check`,
# `just ci`, and the GitHub Actions workflow — guaranteeing local↔CI parity.
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1

# Force color into the per-check subprocesses when we're on a real terminal: their stdout is
# piped through `tee` below (so it's no longer a TTY), but the combined stream still lands on
# the user's terminal, so colors should survive.
[[ -t 1 ]] && export MYC_FORCE_COLOR=1

checks=(structured shell markdown links doc-currency doc-status schema grammar spell secrets format lint safety test myc-fmt myc-check myc-sec myc-lint myc-doc myc-spore proofs api doc-index deny)
failed=()

# Capture each gate's output while still streaming it live, so a failure can be replayed in the
# end-of-run digest instead of being lost in the scrollback (the "what actually failed?" mystery).
logdir="$(mktemp -d "${TMPDIR:-/tmp}/myc-checks.XXXXXX")"
trap 'rm -rf "$logdir"' EXIT

for c in "${checks[@]}"; do
  # `pipefail` (set in lib.sh) makes the check's own exit status win over tee's, so a real
  # failure is still detected even though the output is piped.
  if ! bash "$REPO_ROOT/scripts/checks/$c.sh" 2>&1 | tee "$logdir/$c.log"; then
    failed+=("$c")
  fi
  echo
done

printf '%s========================================%s\n' "$C_DIM" "$C_RST"
if [[ ${#failed[@]} -eq 0 ]]; then
  printf '%sALL CHECKS PASSED%s (skips are non-fatal)\n' "$C_GRN" "$C_RST"
  exit 0
fi

# Failure digest — name every failed gate, show how to reproduce it in isolation, and replay the
# tail of its output, so "what failed" is never a mystery: it sits at the very bottom of the run.
printf '%sFAILED (%d of %d): %s%s\n' \
  "$C_RED" "${#failed[@]}" "${#checks[@]}" "${failed[*]}" "$C_RST"
digest_lines=40
for c in "${failed[@]}"; do
  printf '\n%s── failed gate: %s ──%s\n' "$C_RED" "$c" "$C_RST"
  printf '   %sreproduce:%s bash scripts/checks/%s.sh\n' "$C_DIM" "$C_RST" "$c"
  printf '   %s(last %d lines of its output — run the reproduce command for the full log)%s\n' \
    "$C_DIM" "$digest_lines" "$C_RST"
  tail -n "$digest_lines" "$logdir/$c.log" | sed 's/^/   │ /'
done
exit 1
