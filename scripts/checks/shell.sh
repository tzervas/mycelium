#!/usr/bin/env bash
# Lint shell scripts with shellcheck (incl. the shellcheck-py wheel).
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "shellcheck"

tracked '*.sh'
if [[ ${#TRACKED[@]} -eq 0 ]]; then skip "no shell scripts"; exit 0; fi
if ! have shellcheck; then skip "shellcheck not found — run \`just setup\`"; exit 0; fi

# SC1091: can't follow non-constant `source` (lib.sh path); accepted by design.
if shellcheck -x -e SC1091 "${TRACKED[@]}"; then
  ok "${#TRACKED[@]} script(s) clean"
else
  fail "shellcheck findings"; exit 1
fi
