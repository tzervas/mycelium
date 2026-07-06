#!/usr/bin/env bash
# gen/myc-drafts/regenerate.sh -- the M-1002 batch driver (kickoff trx2 E-B / epic E33-1).
#
# Runs the transpile -> `myc check` vet loop (M-1000) over the wave-1 port-surface target list
# (M-1003: the 5 mycelium-l1 semcore SCC files + the 12 unported stdlib crates) into per-target
# subdirectories under gen/myc-drafts/, then assembles the deterministic MANIFEST.md/manifest.json
# from the artifacts the transpiler itself already wrote.
#
# Pure orchestration: this script drives the existing `mycelium-transpile --vet` CLI
# (crates/mycelium-transpile) and the real `myc-check` oracle (crates/mycelium-check), then folds
# their already-deterministic per-target JSON into one manifest via manifest_gen.py. No transpiler
# or gap/vet logic lives here (DRY -- the Rust crate is the single source of truth for what a gap
# or a vet verdict IS; this script only counts/aggregates what it already wrote).
#
# Build discipline mirrors scripts/checks/transpile-vet.sh: both binaries are built ONCE up front
# and the pre-built myc-check binary is handed to the transpiler via MYC_CHECK_CMD, so `--vet`
# spawns it directly rather than nesting `cargo run` per file (avoids build-lock contention across
# 17 target invocations).
#
# Determinism: no wall-clock timestamps are written to any diffed artifact. The manifest's one
# provenance field is `generated_from_commit` (git SHA of HEAD at generation time) -- see
# manifest_gen.py's docstring. Re-running this script at the same commit with no source changes
# reproduces manifest.json/MANIFEST.md byte-for-byte.
#
# Usage: bash gen/myc-drafts/regenerate.sh   (run from anywhere; resolves the repo root itself)
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/../../scripts/lib.sh"
cd "$REPO_ROOT" || exit 1
section "gen/myc-drafts regenerate (M-1002 batch driver -- transpile -> myc check vet loop)"

if ! have cargo; then skip "no cargo -- myc-drafts regenerate skipped"; exit 0; fi
if ! have python3; then fail "no python3 -- cannot assemble the manifest"; exit 1; fi

OUT_ROOT="$SCRIPT_DIR"

# Build both binaries once (avoids nested-cargo build-lock contention across N target runs --
# mirrors scripts/checks/transpile-vet.sh).
if ! cargo build -q -p mycelium-check --bin myc-check 2>/dev/null; then
  fail "could not build myc-check"
  exit 1
fi
if ! cargo build -q -p mycelium-transpile --bin mycelium-transpile 2>/dev/null; then
  fail "could not build mycelium-transpile"
  exit 1
fi
MYC_CHECK="$REPO_ROOT/target/debug/myc-check"
TRANSPILE="$REPO_ROOT/target/debug/mycelium-transpile"
export MYC_CHECK_CMD="$MYC_CHECK"

# Wave-1 port-surface target list (E33-1 / M-1003) -- semcore SCC (per-file) + the 12 unported
# stdlib crate src/ dirs (per-crate). Format: "<output-subdir>|<input>|<kind>". Kept as an explicit,
# ordered list (not a glob over crates/) so the manifest's target set is exactly the
# maintainer-confirmed wave-1 breadth (.claude/kickoffs/trx2.md) -- expanding it is M-1006's ladder,
# not this driver's job. Every target gets its OWN output subdirectory (never a shared flat dir),
# which is what avoids the batch-mode stem-collision hazard (12 stdlib crates all have `lib.rs`).
TARGETS=(
  "semcore/checkty|crates/mycelium-l1/src/checkty.rs|semcore"
  "semcore/elab|crates/mycelium-l1/src/elab.rs|semcore"
  "semcore/eval|crates/mycelium-l1/src/eval.rs|semcore"
  "semcore/mono|crates/mycelium-l1/src/mono.rs|semcore"
  "semcore/fuse|crates/mycelium-l1/src/fuse.rs|semcore"
  "stdlib/std-conformance|crates/mycelium-std-conformance/src|stdlib"
  "stdlib/std-content|crates/mycelium-std-content/src|stdlib"
  "stdlib/std-dense|crates/mycelium-std-dense/src|stdlib"
  "stdlib/std-fs|crates/mycelium-std-fs/src|stdlib"
  "stdlib/std-io|crates/mycelium-std-io/src|stdlib"
  "stdlib/std-numerics|crates/mycelium-std-numerics/src|stdlib"
  "stdlib/std-rand|crates/mycelium-std-rand/src|stdlib"
  "stdlib/std-runtime|crates/mycelium-std-runtime/src|stdlib"
  "stdlib/std-sys|crates/mycelium-std-sys/src|stdlib"
  "stdlib/std-sys-host|crates/mycelium-std-sys-host/src|stdlib"
  "stdlib/std-time|crates/mycelium-std-time/src|stdlib"
  "stdlib/std-vsa|crates/mycelium-std-vsa/src|stdlib"
)

rc=0
for row in "${TARGETS[@]}"; do
  # shellcheck disable=SC2034  # kind is part of the row format; only manifest_gen.py needs it
  IFS='|' read -r subdir input kind <<<"$row"
  if [[ ! -e "$input" ]]; then
    fail "target not found: $input (subdir $subdir) -- recorded as transpile_failed in the manifest"
    rc=1
    continue
  fi
  # Repo-root-relative (never absolute): the transpiler bakes the out-dir argument verbatim into
  # vet.json's `myc_file` field, so an absolute path here would embed this checkout's filesystem
  # location into a committed, diffed artifact -- non-portable and a determinism hazard of the
  # same class the "no churning timestamps" rule guards against. We already `cd`ed to $REPO_ROOT
  # above, so a repo-relative out-dir resolves identically for this invocation.
  outdir="gen/myc-drafts/$subdir"
  mkdir -p "$outdir"
  # Clear stale artifacts from a previous run so a shrinking target set (or a stem that no longer
  # emits) never leaves an orphaned file behind (never-silent staleness).
  find "$outdir" -maxdepth 1 -type f \( -name '*.myc' -o -name '*.gap.json' -o -name 'summary.json' \
    -o -name 'union.gap.json' -o -name 'vet.json' \) -delete
  out="$("$TRANSPILE" --vet "$input" "$outdir" 2>&1)"
  vet_line="$(printf '%s\n' "$out" | grep -- '--vet over' | head -1)"
  if [[ -z "$vet_line" ]]; then
    fail "$subdir: transpile --vet produced no vet summary (hard parse failure?) -- $out"
    rc=1
    continue
  fi
  ok "$subdir: ${vet_line#mycelium-transpile: --vet }"
done

section "assembling MANIFEST.md / manifest.json"
SOURCE_COMMIT="$(git -C "$REPO_ROOT" rev-parse HEAD)"
if ! python3 "$SCRIPT_DIR/manifest_gen.py" --root "$OUT_ROOT" --repo-root "$REPO_ROOT" \
    --source-commit "$SOURCE_COMMIT" --targets "${TARGETS[@]}"; then
  fail "manifest_gen.py reported one or more transpile_failed targets (see manifest.json/MANIFEST.md) -- manifest was still written"
  rc=1
fi

if (( rc != 0 )); then
  fail "myc-drafts regenerate: completed with failures recorded (never silent -- see manifest)"
else
  ok "myc-drafts regenerate complete -- $OUT_ROOT/MANIFEST.md + manifest.json"
fi
exit "$rc"
