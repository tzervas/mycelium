#!/usr/bin/env bash
# Per-repo preflight for the decomposition alignment assessment (read-only).
# Emits TSV: repo, head, head==origin/main, head==lock_pin, commits, dirty,
#            author, date, msg_sha, msg_sha==archive, readme_sha_match, live_tip_status
set -u
LOCK=/home/user/mycelium-lang/components.lock
ARCHIVE=aad96b7a425710db5e91094d4fc2ca21a129e41a
BASE=/home/user

declare -A PIN
while IFS='=' read -r k v; do
  [[ "$k" =~ ^#.*$ || -z "$k" ]] && continue
  PIN["$k"]="$v"
done < "$LOCK"

printf 'repo\thead\thead_eq_origin_main\thead_eq_pin\tcommits\tdirty\tauthor\tdate\tmsg_sha_eq_archive\treadme_sha_eq_archive\tlive_main_eq_pin\n'

for d in "$BASE"/mycelium-*/; do
  repo=$(basename "$d")
  cd "$d" || continue
  head=$(git rev-parse HEAD 2>/dev/null)
  om=$(git rev-parse origin/main 2>/dev/null || echo MISSING)
  eq_om=$([[ "$head" == "$om" ]] && echo yes || echo NO)
  pin=${PIN[$repo]:-NOPIN}
  eq_pin=$([[ "$head" == "$pin" ]] && echo yes || echo "NO($pin)")
  n=$(git rev-list --count HEAD 2>/dev/null)
  dirty=$([[ -z $(git status --porcelain 2>/dev/null) ]] && echo clean || echo DIRTY)
  author=$(git log -1 --format='%an <%ae>' 2>/dev/null)
  date=$(git log -1 --format='%aI' 2>/dev/null)
  msg=$(git log -1 --format='%s' 2>/dev/null)
  msg_sha=$(grep -oE '[0-9a-f]{40}' <<<"$msg" | head -1 || true)
  eq_msg=$([[ "$msg_sha" == "$ARCHIVE" ]] && echo yes || echo "NO(${msg_sha:-none})")
  readme_sha=$(grep -oE '[0-9a-f]{40}' README.md 2>/dev/null | head -1 || true)
  eq_readme=$([[ "$readme_sha" == "$ARCHIVE" ]] && echo yes || echo "NO(${readme_sha:-none})")
  live=$(git ls-remote origin refs/heads/main 2>/dev/null | awk '{print $1}')
  if [[ -z "$live" ]]; then live_st="UNRESOLVABLE"
  elif [[ "$live" == "$pin" ]]; then live_st="yes"
  else live_st="DRIFT($live)"; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$repo" "${head:0:12}" "$eq_om" "$eq_pin" "$n" "$dirty" "$author" "$date" "$eq_msg" "$eq_readme" "$live_st"
done

# --- lock accounting ---
echo "---LOCK-ACCOUNTING---"
total=0; rust=0; myc=0
for k in "${!PIN[@]}"; do
  total=$((total+1))
  if [[ "$k" == *-myc ]]; then myc=$((myc+1)); else rust=$((rust+1)); fi
done
echo "pins_total=$total pins_rust=$rust pins_myc=$myc"
echo "pinned repos with no local clone:"
for k in "${!PIN[@]}"; do [[ -d "$BASE/$k" ]] || echo "  $k=${PIN[$k]}"; done | sort
echo "local clones with no pin:"
for d in "$BASE"/mycelium-*/; do r=$(basename "$d"); [[ "$r" == mycelium ]] && continue; [[ -n "${PIN[$r]:-}" ]] || echo "  $r"; done
