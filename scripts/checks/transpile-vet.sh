#!/usr/bin/env bash
# transpile-vet — the transpile → `myc check` VET LOOP wrapper (M-1000).
#
# Runs the Rust→Mycelium transpiler (crates/mycelium-transpile) over a set of representative
# targets with `--vet`, so each emitted `.myc` is validated by the REAL `myc check` oracle and the
# transpiler's accuracy is measured as **checked_fraction** (myc-check-clean) — not only the
# emission-only expressible_fraction (which over-counts: it never runs the toolchain). A draft is
# then `myc-check-clean` or `gap/vet-flagged`, never silently broken (G2).
#
# Build discipline (avoids nested-`cargo` build-lock contention): `myc-check` is built ONCE up front
# and its binary path handed to the transpiler via `MYC_CHECK_CMD`, so the transpiler's `--vet`
# spawns the pre-built binary directly rather than a nested `cargo run` (see
# crates/mycelium-transpile/src/vet.rs::MycChecker). Mirrors scripts/checks/myc-dogfood.sh's
# per-file oracle invocation.
#
# Targets: pass one or more `<crate-src-dir | .rs file>` args, OR none for the default
# representative set (a semcore probe + two unported stdlib crates + the std-cmp pilot). Output
# artifacts (per-target `.myc`/`.gap.json` + `summary.json`/`union.gap.json`/`vet.json`) land under
# a scratch dir (TMPDIR); the per-target headline `checked_fraction` vs `expressible_fraction` line
# is printed here.
#
# NON-GATING / ADVISORY by design (like myc-dogfood): this MEASURES the transpiler; it prints
# (never silent) but does not turn `just check` red. Skips gracefully when cargo is absent. Exit 0
# on success/skip; non-zero only on a hard driver error (a target that fails to transpile-parse).
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "transpile-vet (transpile → myc check vet loop — M-1000; advisory)"

if ! have cargo; then skip "no cargo — transpile-vet skipped"; exit 0; fi

# Default representative target set (M-1001 profile: ≥1 semcore module + unported stdlib crates +
# the std-cmp pilot). Overridable by CLI args.
if [[ $# -gt 0 ]]; then
  TARGETS=("$@")
else
  TARGETS=(
    "crates/mycelium-l1/src/eval.rs"    # semcore probe (checkty type vocabulary + guarantee grading)
    "crates/mycelium-l1/src/fuse.rs"    # semcore probe (the DN-26 tractable-sub-core witness)
    "crates/mycelium-std-time/src"      # unported stdlib crate
    "crates/mycelium-std-rand/src"      # unported stdlib crate
    "crates/mycelium-std-cmp/src"       # the std-cmp pilot (DN-34 §8.2)
  )
fi

# Build both binaries once (first myc-check build can be slow — expected).
if ! cargo build -q -p mycelium-check --bin myc-check 2>/dev/null; then
  fail "could not build myc-check — transpile-vet cannot run the oracle"
  exit 1
fi
if ! cargo build -q -p mycelium-transpile --bin mycelium-transpile 2>/dev/null; then
  fail "could not build mycelium-transpile"
  exit 1
fi
MYC_CHECK="$REPO_ROOT/target/debug/myc-check"
TRANSPILE="$REPO_ROOT/target/debug/mycelium-transpile"
export MYC_CHECK_CMD="$MYC_CHECK"

OUT_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/transpile-vet.XXXXXX")"
trap 'rm -rf "$OUT_ROOT"' EXIT

rc=0
for tgt in "${TARGETS[@]}"; do
  if [[ ! -e "$tgt" ]]; then
    skip "target not found: $tgt"
    continue
  fi
  out="$OUT_ROOT/$(echo "$tgt" | tr '/.' '__')"
  mkdir -p "$out"
  # `--vet` prints the checked_fraction/expressible_fraction headline; capture it for a tidy line.
  line="$("$TRANSPILE" --vet "$tgt" "$out" 2>/dev/null | grep -- '--vet over' | head -1)"
  if [[ -z "$line" ]]; then
    fail "$tgt: transpile --vet produced no vet summary (hard parse failure?)"
    rc=1
    continue
  fi
  # Strip the scratch path from the printed line for readability.
  ok "$tgt: ${line#mycelium-transpile: --vet }"
done

if (( rc != 0 )); then
  fail "transpile-vet: one or more targets failed to transpile"
else
  ok "transpile-vet: vet loop complete over ${#TARGETS[@]} target(s) (checked_fraction is the port-accuracy metric — DN-34 §8.7)"
fi
exit "$rc"
