#!/usr/bin/env bash
# Run the full local suite. This is the single entrypoint shared by `just check`,
# `just ci`, and the GitHub Actions workflow — guaranteeing local↔CI parity.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1

# Force color into the per-check subprocesses when we're on a real terminal: their stdout is
# piped through `tee` below (so it's no longer a TTY), but the combined stream still lands on
# the user's terminal, so colors should survive.
[[ -t 1 ]] && export MYC_FORCE_COLOR=1

checks=(structured shell markdown links doc-currency doc-status schema grammar spell secrets format lint safety unsafe-per-use deps-acyclic test myc-fmt myc-check myc-sec myc-lint myc-doc myc-spore myc-dogfood proofs api doc-index lib-index tero-index deny drift license-first-party licenses)

# --- Specific exit codes: which component failed, and why -------------------------------------
# A process exit status is a single byte (0-255), so we pack it: the high 5 bits are a stable
# COMPONENT id (1-31), the low 3 bits a REASON sub-code (0-7) — `exit = (component<<3) | reason`.
# So `echo $?` after `just check` decodes uniquely: `component = code>>3`, `reason = code & 7`.
# On multiple failures the FIRST failed gate sets the process exit byte (the digest lists them all,
# each with its own code). The same values are also printed as a compact base-36 string `F<c><r>`
# (e.g. `Fn2`) — `0-9a-z` per position — so a human reads the gate + reason at a glance.
#
# Component ids are EXPLICIT (not the array index) so reordering `checks` never renumbers them.
declare -A COMPONENT_ID=(
  [structured]=1 [shell]=2 [markdown]=3 [links]=4 [doc-currency]=5 [doc-status]=6 [schema]=7
  [grammar]=8 [spell]=9 [secrets]=10 [format]=11 [lint]=12 [safety]=13 [test]=14 [myc-fmt]=15
  [myc-check]=16 [myc-sec]=17 [myc-lint]=18 [myc-doc]=19 [myc-spore]=20 [proofs]=21 [api]=22
  [doc-index]=23 [deny]=24 [drift]=25 [unsafe-per-use]=26 [license-first-party]=27 [licenses]=28
  [deps-acyclic]=29 [myc-dogfood]=30 [lib-index]=31
  # The 5-bit component-id space (1-31) is exhausted, so the tero-index drift gate SHARES doc-index's
  # id 23 — both are index-drift gates, so an exit-byte collision is semantically benign (the failure
  # digest still names the actual gate that failed by its own script name). E39-1/DN-87.
  [tero-index]=23
)
# REASON sub-codes (0-7): 1 = generic failure (a gate that just `exit 1`s). A gate MAY exit 2-6 to
# name a *specific* failure mode (documented in that gate's script); 7 = an unexpected/other exit
# (the gate exited with a code > 6, e.g. a tool crash). 0 is success (never recorded as a failure).
B36=0123456789abcdefghijklmnopqrstuvwxyz
b36() { printf '%s' "${B36:$1:1}"; } # encode a 0-35 value as one base-36 digit

failed=(); fcodes=()
logdir="$(mktemp -d "${TMPDIR:-/tmp}/myc-checks.XXXXXX")"
trap 'rm -rf "$logdir"' EXIT

for c in "${checks[@]}"; do
  # Capture each gate's output while streaming it live, so a failure can be replayed in the
  # end-of-run digest instead of being lost in the scrollback. `PIPESTATUS[0]` recovers the gate's
  # OWN exit status through the `tee` pipe (which is what carries the reason sub-code).
  set +e
  bash "$REPO_ROOT/scripts/checks/$c.sh" 2>&1 | tee "$logdir/$c.log"
  rc=${PIPESTATUS[0]}
  set -e
  if (( rc != 0 )); then
    reason=$(( rc >= 1 && rc <= 6 ? rc : 7 ))
    failed+=("$c")
    fcodes+=("$(( (COMPONENT_ID[$c] << 3) | reason ))")
  fi
  echo
done

printf '%s========================================%s\n' "$C_DIM" "$C_RST"
if [[ ${#failed[@]} -eq 0 ]]; then
  printf '%sALL CHECKS PASSED%s (skips are non-fatal)\n' "$C_GRN" "$C_RST"
  exit 0
fi

# Failure digest — name every failed gate, its packed exit byte + base-36 code, how to reproduce it
# in isolation, and the tail of its output, so "what failed (and why)" is never a mystery.
printf '%sFAILED (%d of %d): %s%s\n' \
  "$C_RED" "${#failed[@]}" "${#checks[@]}" "${failed[*]}" "$C_RST"
printf '%s(code F<component><reason> base-36; process exit = component<<3 | reason)%s\n' \
  "$C_DIM" "$C_RST"
digest_lines=40
for i in "${!failed[@]}"; do
  c=${failed[i]}; code=${fcodes[i]}
  comp=$(( code >> 3 )); reason=$(( code & 7 ))
  printf '\n%s── failed gate: %s   code F%s%s   exit %d ──%s\n' \
    "$C_RED" "$c" "$(b36 "$comp")" "$(b36 "$reason")" "$code" "$C_RST"
  printf '   %sreproduce:%s bash scripts/checks/%s.sh\n' "$C_DIM" "$C_RST" "$c"
  printf '   %s(last %d lines of its output — run the reproduce command for the full log)%s\n' \
    "$C_DIM" "$digest_lines" "$C_RST"
  tail -n "$digest_lines" "$logdir/$c.log" | sed 's/^/   │ /'
done

# The process exit byte summarizes the FIRST failure (component + reason).
exit "${fcodes[0]}"
