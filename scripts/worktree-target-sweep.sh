#!/usr/bin/env bash
# worktree-target-sweep.sh — reclaim build-cache bloat from git-worktree target/ dirs.
#
# Dry-run BY DEFAULT (prints what it *would* reclaim, never deletes without --apply).
# Never-silent: every skip prints its reason. Never touches a live build, a locked
# worktree, the main checkout (unless --include-main), or any dist/ tree.
#
# A worktree target/ is RECLAIMABLE only when ALL hold:
#   (1) the worktree is NOT locked           (locked == an active agent owns it)
#   (2) its branch is merged into the mainline  OR  the worktree dir is gone (prunable)
#   (3) target/ has not been modified for >= --stale-min minutes  (liveness grace)
#   (4) no rustc/cargo process is currently building under that worktree path
# Criterion (3) is the load-bearing guard: a merged branch whose target was touched
# seconds ago is a *live rebuild* in an in-flight wave — never reclaim it.
#
# Usage:
#   worktree-target-sweep.sh                 # dry-run, default policy
#   worktree-target-sweep.sh --apply         # actually delete (after reviewing dry-run)
#   worktree-target-sweep.sh --stale-min 60  # tighten/loosen the liveness grace (default 240)
#   worktree-target-sweep.sh --mainline main # merge target other than 'dev'
#   worktree-target-sweep.sh --include-main  # ALSO consider the main checkout's target (opt-in)
#   worktree-target-sweep.sh --incremental-only  # only delete stale debug/incremental/ cruft
set -euo pipefail

REPO_ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel 2>/dev/null || git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

APPLY=0; STALE_MIN=240; MAINLINE=dev; INCLUDE_MAIN=0; INCR_ONLY=0
while [ $# -gt 0 ]; do case "$1" in
  --apply) APPLY=1;;
  --stale-min) STALE_MIN="$2"; shift;;
  --mainline) MAINLINE="$2"; shift;;
  --include-main) INCLUDE_MAIN=1;;
  --incremental-only) INCR_ONLY=1;;
  -h|--help) sed -n '2,30p' "$0"; exit 0;;
  *) echo "unknown arg: $1" >&2; exit 2;;
esac; shift; done

if [ -t 1 ]; then R=$'\e[31m'; G=$'\e[32m'; Y=$'\e[33m'; D=$'\e[2m'; Z=$'\e[0m'; else R='' G='' Y='' D='' Z=''; fi
mode="${Y}DRY-RUN${Z} (no deletions; pass --apply to reclaim)"; [ "$APPLY" = 1 ] && mode="${R}APPLY${Z} (deleting)"
echo "worktree-target-sweep — $mode  mainline=$MAINLINE  stale-min=$STALE_MIN"
echo

MAIN_WT="$REPO_ROOT"
now=$(date +%s)
total_kb=0

# Is any cargo/rustc actively building under $1 ? (never reclaim a live build)
build_active() { pgrep -af 'rustc|cargo' 2>/dev/null | grep -Fq -- "$1" && return 0 || return 1; }
# minutes since target/ last modified
age_min() { echo $(( (now - $(stat -c %Y "$1")) / 60 )); }
sz_kb()   { du -sk "$1" 2>/dev/null | cut -f1; }
human()   { du -sh "$1" 2>/dev/null | cut -f1; }

reclaim() {  # $1=path  $2=reason
  local p="$1" why="$2" k; k=$(sz_kb "$p"); total_kb=$((total_kb + k))
  if [ "$APPLY" = 1 ]; then
    rm -rf "$p" && printf '  %sRECLAIMED%s %-8s %s  (%s)\n' "$G" "$Z" "$(numfmt --to=iec $((k*1024)))" "$p" "$why"
  else
    printf '  %swould reclaim%s %-8s %s  (%s)\n' "$Y" "$Z" "$(numfmt --to=iec $((k*1024)))" "$p" "$why"
  fi
}

# Parse `git worktree list --porcelain` into: path \t branch \t locked.
# Process-substitution (not a pipe) so the loop runs in THIS shell and total_kb survives.
while IFS=$'\t' read -r wt br lk; do
  [ -z "$wt" ] && continue
  tgt="$wt/target"
  if [ "$wt" = "$MAIN_WT" ] && [ "$INCLUDE_MAIN" != 1 ]; then
    [ -d "$tgt" ] && echo "  ${D}skip${Z} $tgt  (main checkout — use --include-main to consider it; $(human "$tgt"))"
    continue
  fi
  [ -d "$tgt" ] || continue

  # --- incremental-only mode: reap stale debug/incremental/ even in kept worktrees ---
  if [ "$INCR_ONLY" = 1 ]; then
    for inc in "$tgt"/debug/incremental "$tgt"/release/incremental; do
      [ -d "$inc" ] || continue
      if [ "$(age_min "$inc")" -lt "$STALE_MIN" ]; then echo "  ${D}skip${Z} $inc  (modified <${STALE_MIN}m ago)"; continue; fi
      build_active "$wt" && { echo "  ${D}skip${Z} $inc  (live build under $wt)"; continue; }
      reclaim "$inc" "stale incremental cruft (incremental=false)"
    done
    continue
  fi

  # --- full target reclaim path ---
  if [ "$lk" = "LOCKED" ]; then echo "  ${D}skip${Z} $tgt  (worktree LOCKED — active agent)"; continue; fi
  if build_active "$wt"; then echo "  ${D}skip${Z} $tgt  (live rustc/cargo under $wt)"; continue; fi
  a=$(age_min "$tgt")
  if [ "$a" -lt "$STALE_MIN" ]; then echo "  ${D}skip${Z} $tgt  (modified ${a}m ago < ${STALE_MIN}m grace — may be live)"; continue; fi

  merged=0
  if [ "$br" != "(detached)" ] && [ -n "$br" ]; then
    git merge-base --is-ancestor "$br" "$MAINLINE" 2>/dev/null && merged=1
  fi
  if [ "$merged" = 1 ]; then reclaim "$tgt" "branch ${br#refs/heads/} merged into $MAINLINE, idle ${a}m"
  else echo "  ${D}skip${Z} $tgt  (branch ${br#refs/heads/} NOT merged into $MAINLINE; idle ${a}m)"; fi
done < <(git worktree list --porcelain | awk '
  /^worktree /{wt=$2} /^branch /{br=$2} /^detached/{br="(detached)"} /^locked/{lk="LOCKED"}
  /^$/{print wt"\t"br"\t"(lk?lk:"unlocked"); wt=br=lk=""}
  END{if(wt)print wt"\t"br"\t"(lk?lk:"unlocked")}')

# Prune worktree refs whose directory is already gone (reclaims their orphaned targets too).
echo
echo "${D}git worktree prune (gone dirs):${Z}"
if [ "$APPLY" = 1 ]; then git worktree prune -v || true; else git worktree prune -v --dry-run || true; fi

echo
printf '%sTotal %s: %s%s\n' "$G" "$([ "$APPLY" = 1 ] && echo reclaimed || echo reclaimable)" "$(numfmt --to=iec $((total_kb*1024)))" "$Z"
[ "$APPLY" = 1 ] || echo "${D}(dry-run — re-run with --apply to delete the above)${Z}"
