#!/usr/bin/env bash
# base-guard — assert a swarm leaf/working branch is built on the intended base tip, so a
# stale-base spawn (branched off the release tip, which lags the working tier by the whole in-flight
# wave) is caught BEFORE its work is merged, never silently.
#
# The footgun this closes (CLAUDE.md mitigation #13): an isolated-worktree agent whose base ref
# defaulted to the default branch (`origin/main`) instead of the working tier (`dev`) produces a
# branch that, diffed against the working tier, *reverts* every in-flight wave commit. Octopus-merging
# it silently backs out landed work. `worktree.baseRef=head` prevents it at the source; this guard is
# the defense-in-depth twin (verify the base the same way mitigation #7 verifies the merge landed).
#
# Usage:
#   scripts/checks/base-guard.sh --ref <leaf-ref> --base <expected-base-tip> [--quiet]
#   scripts/checks/base-guard.sh --ref <leaf-ref>            # --base defaults to origin/dev
#
#   --ref   the leaf/working ref to check (branch name or SHA; must exist locally — fetch first).
#   --base  the tip the ref MUST contain as an ancestor (branch name or SHA). Default: origin/dev.
#   --quiet suppress the ok line.
#
# Idempotent, pure reads. Exit 0 = ref contains base (safe to merge); exit 1 = STALE BASE (never
# merge); exit 2 = misuse / unknown ref.
set -euo pipefail

ref=""
base="origin/dev"
quiet=0
while [ $# -gt 0 ]; do
  case "$1" in
    --ref)   ref="${2:-}"; shift 2 ;;
    --base)  base="${2:-}"; shift 2 ;;
    --quiet) quiet=1; shift ;;
    -h|--help) sed -n '2,22p' "$0"; exit 0 ;;
    *) echo "base-guard: unknown arg '$1' (use --ref <r> --base <b> [--quiet])" >&2; exit 2 ;;
  esac
done

[ -n "$ref" ] || { echo "base-guard: --ref is required" >&2; exit 2; }
git rev-parse --git-dir >/dev/null 2>&1 || { echo "base-guard: not inside a git repo" >&2; exit 2; }

ref_sha="$(git rev-parse --verify --quiet "$ref^{commit}" || true)"
[ -n "$ref_sha" ] || { echo "base-guard: ref '$ref' not found (fetch first?)" >&2; exit 2; }
base_sha="$(git rev-parse --verify --quiet "$base^{commit}" || true)"
[ -n "$base_sha" ] || { echo "base-guard: base '$base' not found (fetch first?)" >&2; exit 2; }

if git merge-base --is-ancestor "$base_sha" "$ref_sha"; then
  [ "$quiet" -eq 1 ] || echo "ok  base-guard: '$ref' contains '$base' ($(git rev-parse --short "$base_sha")) — safe to merge"
  exit 0
fi

mb="$(git merge-base "$base_sha" "$ref_sha" 2>/dev/null || echo '?')"
cat >&2 <<EOF
✗ base-guard: STALE BASE — do NOT merge '$ref'.
  '$ref' ($(git rev-parse --short "$ref_sha")) does NOT contain the intended base '$base' ($(git rev-parse --short "$base_sha")).
  Their common ancestor is $(git rev-parse --short "$mb" 2>/dev/null || echo '?') — the ref was branched off a stale tip
  (likely the default branch instead of the working tier), so diffing it against the base would
  REVERT the in-flight work between them.
  Fix: re-base the leaf's real changes onto '$base' (branch fresh from it and re-apply, or merge
  '$base' in and resolve) — then re-run. See CLAUDE.md mitigation #13 and worktree.baseRef=head.
EOF
exit 1
