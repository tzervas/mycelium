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

# Wave-1 port-surface target list (E33-1 / M-1003) -- the semcore SCC as ONE batch-mode phylum
# target (DN-124 §3.2/M-1079: the 5 files mutually reference each other -- checkty/elab/eval/mono/
# fuse ARE the one l1-frontend phylum, so they are transpiled+vetted together, never as 5 isolated
# single-file invocations that could never credit a cross-nodule `use` between them) + the 12
# unported stdlib crate src/ dirs (per-crate, already each one real phylum boundary -- no change
# needed for them to gain phylum-mode partial verdicts, DN-124 §3.2). Format:
# "<output-subdir>|<input>|<kind>", where a `semcore`-kind row's <input> is a COMMA-separated file
# list (the batch's exact member set) and a `stdlib`-kind row's <input> is a single directory (one
# phylum already). Kept as an explicit, ordered list (not a glob over crates/) so the manifest's
# target set is exactly the maintainer-confirmed wave-1 breadth (.claude/kickoffs/trx2.md) --
# expanding it is M-1006's ladder, not this driver's job. Every target gets its OWN output
# subdirectory (never a shared flat dir), which is what avoids the batch-mode stem-collision hazard
# (12 stdlib crates all have `lib.rs`) -- and, for semcore, IS the DN-124 §6 Attack-1a boundary
# constraint: the phylum-vet dir holds EXACTLY these 5 files, never mycelium-l1/src/'s other ~40
# unrelated files (a "bag of unrelated files" would risk resolving a `use` a real build separates).
TARGETS=(
  "semcore|crates/mycelium-l1/src/checkty.rs,crates/mycelium-l1/src/elab.rs,crates/mycelium-l1/src/eval.rs,crates/mycelium-l1/src/mono.rs,crates/mycelium-l1/src/fuse.rs|semcore"
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

# Pre-DN-124 layout leftover: semcore used to write 5 NESTED per-file subdirs
# (semcore/checkty/, semcore/elab/, ...). The new batch target writes flat into semcore/ directly --
# clear the stale nested dirs so a re-run never leaves orphaned pre-M-1079 artifacts behind
# (never-silent staleness, mirrors the existing per-target stale-file sweep below).
for stale in checkty elab eval mono fuse; do
  [[ -d "gen/myc-drafts/semcore/$stale" ]] && rm -rf "gen/myc-drafts/semcore/$stale"
done

rc=0
for row in "${TARGETS[@]}"; do
  # shellcheck disable=SC2034  # kind is part of the row format; only manifest_gen.py needs it
  IFS='|' read -r subdir input kind <<<"$row"
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
    -o -name 'union.gap.json' -o -name 'vet.json' -o -name 'REMAP.md' \) -delete

  if [[ "$kind" == "semcore" ]]; then
    # Batch the mutually-referencing semcore files as ONE phylum (DN-124 §3.2), via the transpiler's
    # `--files <f1,f2,...>` explicit-file-set mode (M-1079): no directory discovery, no staging --
    # each file's REAL repo path is transpiled and recorded verbatim, so summary.json/vet.json stay
    # deterministic and portable (a staging/scratch path would embed a random tmp name into a
    # committed artifact every run -- a determinism hazard this driver's own "no churning" discipline
    # forbids). `transpile_batch` installs the cross-nodule symbol table across the set (gap-close-2
    # wave-2), and (per M-1079 Unit 2) `--files`-mode `--vet` ALSO runs phylum-mode vetting.
    IFS=',' read -ra files <<<"$input"
    missing=0
    for f in "${files[@]}"; do
      if [[ ! -e "$f" ]]; then
        fail "target not found: $f (subdir $subdir) -- recorded as transpile_failed in the manifest"
        missing=1
      fi
    done
    if (( missing )); then
      rc=1
      continue
    fi
    out="$("$TRANSPILE" --vet --files "$input" "$outdir" 2>&1)"
  else
    if [[ ! -e "$input" ]]; then
      fail "target not found: $input (subdir $subdir) -- recorded as transpile_failed in the manifest"
      rc=1
      continue
    fi
    out="$("$TRANSPILE" --vet "$input" "$outdir" 2>&1)"
  fi

  vet_line="$(printf '%s\n' "$out" | grep -- '--vet over' | head -1)"
  if [[ -z "$vet_line" ]]; then
    fail "$subdir: transpile --vet produced no vet summary (hard parse failure?) -- $out"
    rc=1
    continue
  fi
  ok "$subdir: ${vet_line#mycelium-transpile: --vet }"
  # M-A dual-report (DN-124 §4.3): surface the phylum-mode line too, when the CLI printed one.
  phylum_line="$(printf '%s\n' "$out" | grep -- '--vet --phylum' | head -1)"
  [[ -n "$phylum_line" ]] && ok "$subdir: ${phylum_line#mycelium-transpile: }"
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
