#!/usr/bin/env bash
# Per-use unsafe escape gate (M-793; RFC-0034 §9; sharpens ADR-014).
#
# RFC-0034 §9 sharpens ADR-014 from a global lint toggle to a *per-use, source-visible* opt-in:
# every `unsafe` block in the workspace must be a conscious, auditable choice, independent of
# certification mode — even `fast` is memory-safe. This gate enforces two properties:
#
#   A. **Trusted-kernel crates stay unsafe-free** (compiler-enforced via `#![forbid(unsafe_code)]`):
#      `mycelium-core`, `mycelium-cert`, `mycelium-numerics`, `mycelium-vsa`.  If any of these loses
#      its `#![forbid(unsafe_code)]` line, the gate fails loudly (G2 / never-silent).
#
#   B. **Every non-kernel `unsafe` site carries a per-use `#[allow(unsafe_code)]`** (or its
#      `cfg_attr` variant) within `WINDOW` lines *above* the site — in addition to the existing
#      `// SAFETY:` adjacency requirement (Audit-1 of `safety.sh`, ADR-014 §8.1).  A crate-global
#      `#![allow(unsafe_code)]` is NOT accepted; only a site-local attribute that makes the escape
#      explicit and grep-auditable.
#
# Honesty (VR-5 / `Empirical`): both audits are `Empirical`/`Declared` regex heuristics — the Rust
# compiler and `#![forbid(unsafe_code)]` are the ground truth for audit A; the source is ground
# truth for audit B.  The regex approach is intentional: pure shell + `git grep`, no toolchain
# dependency, so the gate always runs and never silently skips.
#
# This script is COMPLEMENTARY to `safety.sh` (which audits `// SAFETY:` adjacency); together
# they enforce the full ADR-014 + RFC-0034 §9 per-use policy.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1

status=0

# ─────────────────────────────────────────────────────────────────────────────────────────────────
# Audit A — trusted-kernel crates must retain `#![forbid(unsafe_code)]` (RFC-0034 §9 / ADR-014).
# ─────────────────────────────────────────────────────────────────────────────────────────────────
section "Trusted-kernel crates: #![forbid(unsafe_code)] still present (RFC-0034 §9 / ADR-014)"

# The four crates whose zero-unsafe guarantee is compiler-enforced (named by DN-21 §5 F-1 /
# ADR-014 §Consequences).  If any other crate earns this status in future, add it here.
#
# `mycelium-stack` (RFC-0041 §5 / RR-29 §4, W0): the L1 host-stack adapter crate — kept OUTSIDE the
# trusted `mycelium-l1` kernel *by design* (ADR-014, KC-3) so that when W2 wires the grow-on-demand
# `stacker`/`psm` hybrid, the ONLY `unsafe` in the whole recursion-depth-safety surface is contained
# here. Today it is `#![forbid(unsafe_code)]` (std-only `with_deep_stack`, empty deps); RR-29 §4
# flagged that audit-A previously EXCLUDED this crate, so ADR-014 containment rested on an
# *unprotected* forbid line. Adding it here means an accidental removal of the forbid attribute fails
# loudly instead of silently. When W2 lands the optional `grow-on-demand` feature (`stacker::maybe_grow`,
# a safe API whose internal unsafe stays inside the `stacker` leaf crate), this crate's own source
# stays unsafe-free, so this KERNEL_CRATES entry remains valid — audit-A watches this crate's own
# `forbid` line, not its dependency graph. Audit B (per-use #[allow(unsafe_code)]) is unaffected
# either way since `stacker`'s unsafe lives upstream, outside `crates/`.
KERNEL_CRATES=(
  crates/mycelium-core/src/lib.rs
  crates/mycelium-cert/src/lib.rs
  crates/mycelium-numerics/src/lib.rs
  crates/mycelium-vsa/src/lib.rs
  crates/mycelium-stack/src/lib.rs
  crates/mycelium-workstack/src/lib.rs
)

kernel_ok=1
for kf in "${KERNEL_CRATES[@]}"; do
  if [[ ! -f "$kf" ]]; then
    fail "trusted-kernel file missing: $kf — cannot verify #![forbid(unsafe_code)]"
    kernel_ok=0; status=1; continue
  fi
  # grep -q exits 0 on match, 1 on no match; both are expected.  Only >=2 is an error.
  if ! grep -qE '^[[:space:]]*#!\[forbid\(unsafe_code\)\]' "$kf" 2>/dev/null; then
    fail "$kf lost its #![forbid(unsafe_code)] — trusted-kernel zero-unsafe guarantee broken"
    kernel_ok=0; status=1
  fi
done
if (( kernel_ok )); then
  ok "all ${#KERNEL_CRATES[@]} trusted-kernel crates retain #![forbid(unsafe_code)]"
fi

# ─────────────────────────────────────────────────────────────────────────────────────────────────
# Audit B — every non-kernel `unsafe` site carries a per-use `#[allow(unsafe_code)]` (RFC-0034 §9).
#
# Policy (RFC-0034 §9 + ADR-014 §Decision ¶3):
#   - Accepted per-use forms (either is compliant):
#       #[allow(unsafe_code)]                                     — unconditional; silent everywhere
#       #[cfg_attr(not(debug_assertions), allow(unsafe_code))]   — warn in dev/test, silent release
#   - NOT accepted: a crate-wide `#![allow(unsafe_code)]` (inner attribute, prefixed with `!`).
#     That suppresses the warning globally and defeats the per-use explicitness requirement.
# ─────────────────────────────────────────────────────────────────────────────────────────────────
section "Per-use #[allow(unsafe_code)] at every unsafe site (RFC-0034 §9 / ADR-014)"

# Lines scanned upward from an `unsafe` site for its per-use attribute.  Matches the window in
# `safety.sh` (WINDOW=12) so both gates use consistent context.
WINDOW=12

# Candidate unsafe sites under crates/ (same pattern as safety.sh audit 1).
# Distinguish grep exit codes: 0 = matches found, 1 = no matches, >=2 = real error.
raw=$(git grep -nE '(^|[^[:alnum:]_])unsafe[[:space:]]+(\{|fn|impl|trait)' \
  -- ':(glob)crates/**/*.rs') || grep_rc=$?
grep_rc=${grep_rc:-0}
if (( grep_rc >= 2 )); then
  fail "git grep failed (exit ${grep_rc}) scanning for \`unsafe\` sites — cannot audit"
  exit 1
fi

# Filter out line/doc comments (a `// unsafe {` is prose, not code).
hits=()
if [[ -n "$raw" ]]; then
  mapfile -t hits < <(printf '%s\n' "$raw" | grep -vE ':[0-9]+:[[:space:]]*//' || true)
fi

if [[ ${#hits[@]} -eq 0 ]]; then
  ok "no Rust \`unsafe\` sites under crates/ — per-use escape gate trivially satisfied"
else
  missing_allow=()
  global_allow=()
  for h in "${hits[@]}"; do
    file="${h%%:*}"
    rest="${h#*:}"
    line="${rest%%:*}"

    start=$(( line > WINDOW ? line - WINDOW : 1 ))
    # Scan the window above for a per-use (outer) allow attribute in either accepted form.
    # The accepted pattern matches:
    #   #[allow(unsafe_code)]
    #   #[cfg_attr(not(debug_assertions), allow(unsafe_code))]
    # (and any reasonable whitespace variants thereof — the compiler normalises them).
    # We deliberately do NOT match `#![allow(unsafe_code)]` (inner attribute — the `!` after `#`)
    # because that form is crate-global and defeats the per-use requirement.
    window_text=""
    if (( line > 1 )); then
      window_text=$(sed -n "${start},$((line - 1))p" "$file" 2>/dev/null || true)
    fi

    # Pattern matches per-use OUTER attributes (no `!` after `#`):
    #   #[allow(unsafe_code)]
    #   #[cfg_attr(not(debug_assertions), allow(unsafe_code))]
    # The `!` absence is structural: `#[` is an outer attribute (per-site); `#![` is inner (crate-global).
    if printf '%s\n' "$window_text" | grep -qE '#\[(cfg_attr\(.+,\s*)?allow\(unsafe_code\)'; then
      : # compliant — per-use outer attribute found
    else
      missing_allow+=("$file:$line")
    fi
  done

  # Separate check: flag any crate that has a crate-global `#![allow(unsafe_code)]` (inner attr).
  # These are NOT compliant; every site must be annotated individually.
  global_raw=$(git grep -lE '^[[:space:]]*#!\[allow\(unsafe_code\)\]' \
    -- ':(glob)crates/**/*.rs') || global_rc=$?
  global_rc=${global_rc:-0}
  if (( global_rc >= 2 )); then
    fail "git grep failed (exit ${global_rc}) scanning for crate-global #![allow(unsafe_code)]"
    exit 1
  fi
  if [[ -n "$global_raw" ]]; then
    while IFS= read -r gf; do global_allow+=("$gf"); done <<< "$global_raw"
  fi

  peruse_ok=1
  if [[ ${#missing_allow[@]} -gt 0 ]]; then
    fail "${#missing_allow[@]} \`unsafe\` site(s) lack a per-use #[allow(unsafe_code)] within ${WINDOW} lines above (RFC-0034 §9):"
    printf '        %s\n' "${missing_allow[@]}"
    peruse_ok=0; status=1
  fi
  if [[ ${#global_allow[@]} -gt 0 ]]; then
    fail "${#global_allow[@]} file(s) use crate-global #![allow(unsafe_code)] — use per-site #[allow(unsafe_code)] instead (RFC-0034 §9):"
    printf '        %s\n' "${global_allow[@]}"
    peruse_ok=0; status=1
  fi
  if (( peruse_ok )); then
    ok "${#hits[@]} \`unsafe\` site(s): each has a per-use #[allow(unsafe_code)] — no crate-global allows (RFC-0034 §9)"
  fi
fi

exit "$status"
