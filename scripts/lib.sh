#!/usr/bin/env bash
# Shared helpers for scripts/checks/*. Source this; do not execute directly.
# Convention: each check script exits 0 on success OR graceful skip, non-zero on a real failure.
set -euo pipefail

# Colorize when stdout is a terminal, OR when a parent explicitly forces it (so a check whose
# output is piped — e.g. through `tee` in all.sh's capture — still renders color on the real TTY).
if [[ -t 1 || -n "${MYC_FORCE_COLOR:-}" ]]; then
  C_RED=$'\033[31m'; C_GRN=$'\033[32m'; C_YEL=$'\033[33m'; C_DIM=$'\033[2m'; C_RST=$'\033[0m'
else
  C_RED=''; C_GRN=''; C_YEL=''; C_DIM=''; C_RST=''
fi

# Repo root = parent of the dir holding this lib.
# shellcheck disable=SC2034  # consumed by the scripts that source this lib
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

have()    { command -v "$1" >/dev/null 2>&1; }
section() { printf '%s── %s ──%s\n' "$C_DIM" "$*" "$C_RST"; }
ok()      { printf '  %sok%s    %s\n'   "$C_GRN" "$C_RST" "$*"; }
skip()    { printf '  %sskip%s  %s\n'   "$C_YEL" "$C_RST" "$*"; }
fail()    { printf '  %sFAIL%s  %s\n'   "$C_RED" "$C_RST" "$*"; }

# List tracked files matching the given git pathspecs (NUL-safe), into array $TRACKED.
tracked() {
  # shellcheck disable=SC2034  # TRACKED is read by callers after they invoke `tracked ...`
  mapfile -d '' -t TRACKED < <(git ls-files -z -- "$@")
}

# The Mycelium project roots the M-361 toolchain gates over: tracked `mycelium-proj.toml`
# dirs, EXCLUDING any path under tests/fixtures/. Populates the array $MYC_ROOTS (sorted,
# de-duplicated). Those fixtures (incl. bad-header.myc + the reject/ corpus) are
# intentionally-bad must-fail inputs — running the tools over them would erroneously turn
# the gate red (Phase-9 Wave-A locked decision #3). Scope is real project roots only.
myc_roots() {
  # shellcheck disable=SC2034  # MYC_ROOTS is read by callers after they invoke `myc_roots`
  MYC_ROOTS=()
  local f
  while IFS= read -r -d '' f; do
    case "$f" in */tests/fixtures/*|tests/fixtures/*) continue ;; esac
    MYC_ROOTS+=("$(dirname "$f")")
  done < <(git ls-files -z -- '*mycelium-proj.toml')
  if [[ ${#MYC_ROOTS[@]} -gt 0 ]]; then
    mapfile -t MYC_ROOTS < <(printf '%s\n' "${MYC_ROOTS[@]}" | sort -u)
  fi
}
