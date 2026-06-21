#!/usr/bin/env bash
# Change-scoping for the tiered test recipes (DN-20). Computes the set of workspace crates
# touched by the working diff vs the base ref, EXPANDS that set to include every reverse-dependent
# (so a change to a dependency re-runs the crates that depend on it), and emits `-p <crate>` args
# for `cargo test`/`cargo nextest`. The honest contract is **over-test, never under-test**:
#
#   - SAFETY (never silently under-test): when the change touches a shared/root file (root
#     Cargo.toml, Cargo.lock, .cargo/, justfile, scripts/, rust-toolchain.toml, .gitattributes,
#     deny.toml, a workspace-wide file), OR change-detection cannot run (no cargo / no jq / no git),
#     OR no base ref is available, OR no crate maps from the diff — we fall back to `--workspace`,
#     which tests everything. The fast tier may *over*-scope; it must never miss a crate.
#   - `just check-full` always runs the FULL workspace regardless, so a release catches anything a
#     fast-tier scoping ever left out (DN-20 honesty guardrail — coverage is focused, never removed).
#
# Output contract (read by scripts/checks/test.sh):
#   - prints the cargo package-selection args on STDOUT, space-separated, ONE of:
#       --workspace                      (conservative full run)
#       -p crateA -p crateB ...          (scoped run)
#       --no-changes                     (empty selection: no changed crate vs base — caller skips Rust tests this tier)
#   - prints all human/diagnostic narration on STDERR (so STDOUT is clean for `$(...)` capture).
#   - exits 0 on success or any graceful fallback; non-zero only on an internal contract violation.
#
# Offline + deterministic: uses only `git`, `cargo metadata` (local, --offline), and `jq`. No
# network, no clock, no global state. Honors a caller-supplied base ref via $MYC_BASE_REF, else
# tries origin/main, then main, then the merge-base — and falls back to --workspace if none exist.
#
# Honesty (house rule 1 / VR-5): the crate-from-path mapping and the reverse-dep closure are an
# EXACT graph computation over `cargo metadata` (the resolver's own data) — not a heuristic. The
# only Declared element is the shared-file trigger list below (a maintainer-chosen conservative set);
# it can only ever *widen* the run to --workspace, never narrow it, so it cannot cause an under-test.
set -uo pipefail

# Resolve repo root from this script's location (scripts/checks/ -> repo root is two up).
SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SELF_DIR/../.." && pwd)"
cd "$ROOT" || { echo "changed-crates: cannot cd to repo root" >&2; echo "--workspace"; exit 0; }

# Everything explanatory goes to stderr; only the selection args go to stdout.
log() { printf '  %s\n' "$*" >&2; }

emit_workspace() {
  # The conservative full run. $1 (optional) is the human reason.
  [[ -n "${1:-}" ]] && log "scope: --workspace ($1)"
  echo "--workspace"
  exit 0
}

have() { command -v "$1" >/dev/null 2>&1; }

# --- Preconditions: any missing tool ⇒ conservative full run (never under-test). ------------------
have git   || emit_workspace "git not found"
have cargo || emit_workspace "cargo not found — cannot map paths to crates"
have jq    || emit_workspace "jq not found — cannot parse cargo metadata"
[[ -f Cargo.toml ]] || emit_workspace "no workspace Cargo.toml"

# --- Resolve a base ref to diff against. ----------------------------------------------------------
# Priority: caller override ($MYC_BASE_REF) -> origin/main -> main -> empty (=> full run).
# We diff the working tree (committed + uncommitted) against the base's merge-base so a feature
# branch is scoped to its OWN changes, not everything since it forked.
base_ref=""
for cand in "${MYC_BASE_REF:-}" origin/main main; do
  [[ -z "$cand" ]] && continue
  if git rev-parse --verify --quiet "$cand^{commit}" >/dev/null 2>&1; then base_ref="$cand"; break; fi
done
[[ -z "$base_ref" ]] && emit_workspace "no base ref (origin/main / main absent) — cannot diff"

# Prefer the merge-base so we diff only this branch's net change; fall back to the ref itself.
merge_base="$(git merge-base "$base_ref" HEAD 2>/dev/null || true)"
diff_from="${merge_base:-$base_ref}"

# --- Collect changed paths: committed (base..HEAD) UNION working-tree (staged+unstaged+untracked).
# `--no-renames` keeps both old+new paths visible; untracked files are added explicitly so a brand
# new crate is detected before its first commit.
changed_paths="$(
  {
    git diff --name-only --no-renames "$diff_from" -- 2>/dev/null
    git diff --name-only --no-renames HEAD -- 2>/dev/null            # uncommitted vs HEAD
    git ls-files --others --exclude-standard 2>/dev/null             # untracked
  } | sort -u
)"

if [[ -z "$changed_paths" ]]; then
  # No diff at all (clean tree at the base tip): nothing to test in the fast tier. Emit an empty
  # selection sentinel so the caller can short-circuit. We use a distinct token, not --workspace.
  log "scope: no changed files vs $diff_from — empty selection"
  echo "--no-changes"
  exit 0
fi

log "base: $base_ref (diff from ${merge_base:+merge-base }$diff_from); $(printf '%s\n' "$changed_paths" | grep -c .) changed path(s)"

# --- SAFETY: shared/root files ⇒ full run. --------------------------------------------------------
# These either change the build for every crate or are the wave's collision surface; a touch here
# means "the whole workspace could be affected" — fall back to --workspace (Declared, widen-only).
shared_re='^(Cargo\.toml|Cargo\.lock|rust-toolchain\.toml|deny\.toml|\.gitattributes|justfile)$|^\.cargo/|^scripts/|^\.github/'
while IFS= read -r p; do
  [[ -z "$p" ]] && continue
  if [[ "$p" =~ $shared_re ]]; then
    emit_workspace "shared/root file changed: $p"
  fi
done <<< "$changed_paths"

# --- Map changed paths -> workspace crates via cargo metadata. ------------------------------------
# `cargo metadata --no-deps` lists workspace members with their manifest paths; the crate's source
# root is that manifest's directory. A changed path under <dir>/ belongs to that crate. Offline.
# Strictly --offline (the header's contract): a network-hitting fallback could hang in a
# restricted env. An offline-metadata failure is itself a reason to widen to --workspace
# (over-test, never under-test) rather than risk the network.
meta="$(cargo metadata --no-deps --format-version 1 --offline 2>/dev/null)" \
  || emit_workspace "cargo metadata (offline) failed — cannot map paths to crates"

# name<TAB>relative-crate-dir for every workspace member (dir relative to repo root).
crate_dirs="$(
  printf '%s' "$meta" | jq -r --arg root "$ROOT/" '
    .packages[] | [ .name, (.manifest_path | rtrimstr("/Cargo.toml")) ] | @tsv
  ' | while IFS=$'\t' read -r name path; do
        rel="${path#"$ROOT"/}"
        printf '%s\t%s\n' "$name" "$rel"
      done
)"

# For each changed path, find the longest crate-dir prefix that contains it (longest-match so a
# nested crate wins over an ancestor). Collect the touched crate names.
declare -A touched=()
while IFS= read -r p; do
  [[ -z "$p" ]] && continue
  best_name=""; best_len=-1
  while IFS=$'\t' read -r name dir; do
    [[ -z "$dir" ]] && continue
    if [[ "$p" == "$dir/"* ]]; then
      len=${#dir}
      if (( len > best_len )); then best_len=$len; best_name="$name"; fi
    fi
  done <<< "$crate_dirs"
  [[ -n "$best_name" ]] && touched["$best_name"]=1
done <<< "$changed_paths"

if [[ ${#touched[@]} -eq 0 ]]; then
  # Changed files exist but none map to a crate (e.g. only docs/ changed). For the test tiers that
  # means there is nothing to compile-test from this diff. Emit the empty sentinel; the caller
  # short-circuits (the non-test doc gates still run in `just check`).
  log "scope: changed files map to no workspace crate (docs/other only) — empty selection"
  echo "--no-changes"
  exit 0
fi

# --- Expand to reverse-dependents (the transitive closure of "depends on a touched crate"). -------
# Build the intra-workspace reverse-dependency edges from a FULL `cargo metadata` resolve graph,
# then BFS outward from the touched set. We restrict to workspace members (external crates are
# irrelevant — they don't have tests here). Offline.
# Strictly --offline, and a failure here widens to --workspace: an empty resolve graph would
# leave `rev` empty, collapsing the reverse-dep closure to just the touched crates and
# UNDER-testing their dependents — a safety-contract violation. Widen instead (over-test).
full_meta="$(cargo metadata --format-version 1 --offline 2>/dev/null)" \
  || emit_workspace "cargo metadata (full resolve, offline) failed — cannot compute reverse-deps"

declare -A rev=()   # rev[dep_name] = "userA userB ..."  (who depends on dep_name)
if [[ -n "$full_meta" ]]; then
  # Workspace member names (the set we keep). NOTE: cargo's `resolve.nodes[].deps[].name` are the
  # *normalized* crate names (hyphens -> underscores, e.g. `mycelium_core`), whereas
  # `packages[].name` is the canonical manifest name (`mycelium-core`). We therefore key the
  # workspace set by the NORMALIZED name and map each dep back to its canonical name before
  # recording an edge — otherwise the join silently misses every intra-workspace edge (which would
  # collapse the reverse-dep closure to the empty set and under-test, violating the safety contract).
  declare -A is_ws=()       # normalized-name -> 1   (membership test for dep names)
  declare -A canon=()       # normalized-name -> canonical package name
  while IFS= read -r n; do
    [[ -z "$n" ]] && continue
    norm="${n//-/_}"
    is_ws["$norm"]=1
    canon["$norm"]="$n"
  done < <(printf '%s' "$full_meta" | jq -r '.workspace_members as $wm
    | .packages[] | select(.id as $id | $wm | index($id)) | .name')

  # Edges "pkg depends on dep" for workspace pkgs; record reverse edge dep -> pkg when both are ws.
  # $pkg is the canonical (hyphen) name from packages[]; $dep is the normalized (underscore) dep
  # name from the resolve graph — we map $dep back through `canon` to the canonical package name.
  while IFS=$'\t' read -r pkg dep_norm; do
    [[ -z "$pkg" || -z "$dep_norm" ]] && continue
    [[ -n "${is_ws[$dep_norm]:-}" ]] || continue            # dep is a workspace member?
    dep_canon="${canon[$dep_norm]}"
    rev["$dep_canon"]="${rev[$dep_canon]:-} $pkg"
  done < <(
    printf '%s' "$full_meta" | jq -r '
      .resolve.nodes[] as $n
      | ($n.id) as $id
      | (.packages[] | select(.id == $id) | .name) as $pname
      | $n.deps[]? | [ $pname, .name ] | @tsv
    '
  )
fi

# BFS the reverse-dep closure starting from the touched crates.
declare -A selected=()
queue=()
for c in "${!touched[@]}"; do selected["$c"]=1; queue+=("$c"); done
while (( ${#queue[@]} > 0 )); do
  cur="${queue[0]}"; queue=("${queue[@]:1}")
  for dependent in ${rev[$cur]:-}; do
    if [[ -z "${selected[$dependent]:-}" ]]; then
      selected["$dependent"]=1
      queue+=("$dependent")
    fi
  done
done

# --- Emit -p args (sorted for determinism). -------------------------------------------------------
mapfile -t sorted < <(printf '%s\n' "${!selected[@]}" | sort)
log "scope: ${#touched[@]} changed crate(s) -> ${#sorted[@]} with reverse-deps: ${sorted[*]}"
args=()
for c in "${sorted[@]}"; do args+=("-p" "$c"); done
echo "${args[*]}"
exit 0
