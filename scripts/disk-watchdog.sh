#!/usr/bin/env bash
# disk-watchdog.sh — warn when the build filesystem crosses a usage threshold and
# list the largest reclaimable target/ dirs, so a human (or the sweeper) can act.
# Read-only: never deletes anything. Exit 0 = healthy, 1 = over WARN, 2 = over CRIT.
#
# Usage: disk-watchdog.sh [--warn 80] [--crit 90] [--path /] [--top 10]
set -euo pipefail
WARN=80; CRIT=90; FSPATH=/; TOP=10
while [ $# -gt 0 ]; do case "$1" in
  --warn) WARN="$2"; shift;; --crit) CRIT="$2"; shift;;
  --path) FSPATH="$2"; shift;; --top) TOP="$2"; shift;;
  -h|--help) sed -n '2,9p' "$0"; exit 0;; *) echo "unknown arg: $1" >&2; exit 2;;
esac; shift; done
if [ -t 1 ]; then R=$'\e[31m'; G=$'\e[32m'; Y=$'\e[33m'; Z=$'\e[0m'; else R='' G='' Y='' Z=''; fi

read -r _ _ used avail pct _ < <(df -P "$FSPATH" | tail -1)
usep=${pct%\%}
echo "disk-watchdog: $FSPATH  used=$used avail=$avail (${usep}%)  warn=${WARN}% crit=${CRIT}%"

status=0; label="${G}healthy${Z}"
[ "$usep" -ge "$WARN" ] && { status=1; label="${Y}WARN${Z}"; }
[ "$usep" -ge "$CRIT" ] && { status=2; label="${R}CRITICAL${Z}"; }
echo "status: $label"

if [ "$status" -ge 1 ]; then
  REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || echo .)"
  echo
  echo "Top ${TOP} worktree target/ dirs by size (candidates for scripts/worktree-target-sweep.sh):"
  # List every worktree's target with size + lock + merge state, largest first.
  git -C "$REPO_ROOT" worktree list --porcelain | awk '
    /^worktree /{wt=$2} /^branch /{br=$2} /^locked/{lk="LOCKED"} /^detached/{br="(detached)"}
    /^$/{print wt"\t"br"\t"(lk?lk:"-"); wt=br=lk=""} END{if(wt)print wt"\t"br"\t"(lk?lk:"-")}' |
  while IFS=$'\t' read -r wt br lk; do
    [ -d "$wt/target" ] || continue
    m="-"; git -C "$REPO_ROOT" merge-base --is-ancestor "$br" dev 2>/dev/null && m="merged"
    printf '%s\t%s\t%s\t%s\n' "$(du -sk "$wt/target" 2>/dev/null|cut -f1)" "$lk" "$m" "$wt/target"
  done | sort -rn | head -"$TOP" | while IFS=$'\t' read -r k lk m p; do
    printf '  %-8s %-7s %-7s %s\n' "$(numfmt --to=iec $((k*1024)))" "$lk" "$m" "$p"
  done
  echo
  echo "${Y}Action:${Z} run  scripts/worktree-target-sweep.sh  (dry-run) then  --apply."
fi
exit "$status"
