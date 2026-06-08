#!/usr/bin/env bash
# Shared helpers for scripts/checks/*. Source this; do not execute directly.
# Convention: each check script exits 0 on success OR graceful skip, non-zero on a real failure.
set -euo pipefail

if [[ -t 1 ]]; then
  C_RED=$'\033[31m'; C_GRN=$'\033[32m'; C_YEL=$'\033[33m'; C_DIM=$'\033[2m'; C_RST=$'\033[0m'
else
  C_RED=''; C_GRN=''; C_YEL=''; C_DIM=''; C_RST=''
fi

# Repo root = parent of the dir holding this lib.
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

have()    { command -v "$1" >/dev/null 2>&1; }
section() { printf '%s── %s ──%s\n' "$C_DIM" "$*" "$C_RST"; }
ok()      { printf '  %sok%s    %s\n'   "$C_GRN" "$C_RST" "$*"; }
skip()    { printf '  %sskip%s  %s\n'   "$C_YEL" "$C_RST" "$*"; }
fail()    { printf '  %sFAIL%s  %s\n'   "$C_RED" "$C_RST" "$*"; }

# List tracked files matching the given git pathspecs (NUL-safe), into array $TRACKED.
tracked() { mapfile -d '' -t TRACKED < <(git ls-files -z -- "$@"); }
