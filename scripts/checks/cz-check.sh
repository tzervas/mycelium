#!/usr/bin/env bash
# Commitizen message lint (M-1107 / Wave 0-CZ).
# Skip-graceful if `cz` is absent (exit 0). Non-zero only on a real invalid message.
#
# Usage:
#   scripts/checks/cz-check.sh                 # validate HEAD commit message
#   scripts/checks/cz-check.sh <msg-file>      # validate a message file
#   scripts/checks/cz-check.sh -m "feat: …"    # validate an inline message
#   just cz-check [args…]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=../lib.sh
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "commitizen (cz check)"

if ! have cz; then
  skip "cz (commitizen) not found — install via \`uv tool install commitizen\` or \`just setup\`"
  exit 0
fi

# Prefer project .cz.toml when present (allowed_prefixes, major_version_zero, …).
if [[ $# -eq 0 ]]; then
  # Validate the most recent commit on this branch.
  if cz check --allow-abort --rev-range HEAD~1..HEAD; then
    ok "HEAD commit message is conventional (or allowed_prefix)"
  else
    fail "HEAD commit message failed cz check"
    exit 1
  fi
elif [[ "${1:-}" == "-m" || "${1:-}" == "--message" ]]; then
  shift
  msg="${1:-}"
  if [[ -z "$msg" ]]; then
    fail "usage: cz-check.sh -m \"<message>\""
    exit 64
  fi
  if cz check --allow-abort --message "$msg"; then
    ok "message ok: $msg"
  else
    fail "message failed cz check"
    exit 1
  fi
else
  # Treat first arg as a commit-msg file path (git-hook shape).
  msg_file="$1"
  if [[ ! -f "$msg_file" ]]; then
    fail "message file not found: $msg_file"
    exit 64
  fi
  if cz check --allow-abort --commit-msg-file "$msg_file"; then
    ok "message file ok: $msg_file"
  else
    fail "message file failed cz check: $msg_file"
    exit 1
  fi
fi
