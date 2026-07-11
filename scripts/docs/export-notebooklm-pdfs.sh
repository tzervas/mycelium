#!/usr/bin/env bash
# Export the language-focused corpus as per-cluster PDFs for Google NotebookLM.
#
# Ratified path: myc-doc (Typst renderer, docs/spec/Narrative-Authoring-Pipeline.md §8)
#   build a per-cluster doc.typ via `myc-doc build --manifest <cluster>` then
#   `typst compile` it to PDF. Each cluster (tools/docgen/notebooklm/*.json) stays
#   under NotebookLM's 500k-word/source cap; the whole set is well under 50 sources.
#
# Never-silent fallback (G2): if `typst` cannot be obtained OR myc-doc lacks
#   --manifest, emit a concatenated Markdown bundle per cluster instead (NotebookLM
#   ingests .md too). The export still ships; the mechanism is logged, not hidden.
#
# Honesty: the rendered bundles are Declared projections of the corpus (source is
#   ground truth). Deterministic given the corpus + manifests.
#
# Usage: scripts/docs/export-notebooklm-pdfs.sh [out_dir]
#   out_dir defaults to dist/notebooklm/
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"
OUT="${1:-dist/notebooklm}"
MANIFEST_DIR="tools/docgen/notebooklm"
TYPST_DIR="$REPO_ROOT/.tools"
mkdir -p "$OUT"

log() { printf '  %s\n' "$*"; }
say() { printf '\n== %s ==\n' "$*"; }

# ---- 1. build myc-doc -------------------------------------------------------
say "build myc-doc"
if command -v cargo >/dev/null 2>&1; then
  cargo build -q -p mycelium-doc --bin myc-doc
  MYC_DOC="$(cargo metadata --no-deps --format-version 1 2>/dev/null \
    | python3 -c 'import json,sys;print(json.load(sys.stdin)["target_directory"])')/debug/myc-doc"
  [ -x "$MYC_DOC" ] || MYC_DOC="target/debug/myc-doc"
  log "myc-doc: $MYC_DOC"
else
  log "cargo absent — will use the Markdown fallback"
  MYC_DOC=""
fi

# ---- 2. ensure typst (best-effort, skip-graceful) ---------------------------
say "ensure typst"
TYPST=""
if command -v typst >/dev/null 2>&1; then
  TYPST="$(command -v typst)"
elif [ -x "$TYPST_DIR/typst" ]; then
  TYPST="$TYPST_DIR/typst"
else
  mkdir -p "$TYPST_DIR"
  ARCH="$(uname -m)"; TRIPLE=""
  case "$ARCH" in
    x86_64|amd64) TRIPLE="x86_64-unknown-linux-musl" ;;
    aarch64|arm64) TRIPLE="aarch64-unknown-linux-musl" ;;
  esac
  if [ -n "$TRIPLE" ] && command -v curl >/dev/null 2>&1; then
    # PINNED release (never 'latest') so the fetched-and-executed binary is reproducible; verify after.
    TYPST_VERSION="${TYPST_VERSION:-0.15.0}"
    URL="https://github.com/typst/typst/releases/download/v${TYPST_VERSION}/typst-${TRIPLE}.tar.xz"
    log "fetching $URL"
    if curl -fsSL "$URL" -o "$TYPST_DIR/typst.tar.xz" 2>/dev/null \
       && tar -xJf "$TYPST_DIR/typst.tar.xz" -C "$TYPST_DIR" 2>/dev/null; then
      found="$(find "$TYPST_DIR" -name typst -type f | head -1 || true)"
      [ -n "$found" ] && { mv "$found" "$TYPST_DIR/typst"; chmod +x "$TYPST_DIR/typst"; TYPST="$TYPST_DIR/typst"; }
      # Integrity sanity-check: confirm the pinned version is what we fetched (never-silent on mismatch).
      [ -n "$TYPST" ] && { got="$("$TYPST" --version 2>/dev/null)"; case "$got" in *"$TYPST_VERSION"*) : ;; *) log "warn: typst version mismatch (wanted $TYPST_VERSION, got '${got:-none}')";; esac; }
    fi
  fi
  if [ -z "$TYPST" ] && command -v cargo >/dev/null 2>&1; then
    log "prebuilt fetch failed — trying 'cargo install typst-cli' (slow)"
    cargo install typst-cli --root "$TYPST_DIR" >/dev/null 2>&1 && TYPST="$TYPST_DIR/bin/typst" || true
  fi
fi
if [ -n "$TYPST" ]; then
  log "typst: $TYPST"
else
  log "typst unavailable — Markdown fallback for PDF steps"
fi

# ---- 3. per-cluster render --------------------------------------------------
emit_markdown_bundle() {  # $1 manifest, $2 out.md
  local manifest="$1" outfile="$2" rel
  : > "$outfile"
  while IFS= read -r rel; do
    { printf '\n\n<!-- source: %s -->\n\n' "$rel"; cat "$rel"; } >> "$outfile"
  done < <(python3 scripts/docs/notebooklm_resolve.py "$manifest" "$REPO_ROOT")
}

rendered=(); mode_used=()
for manifest in "$MANIFEST_DIR"/*.json; do
  name="$(basename "$manifest" .json)"
  say "cluster: $name"
  pdf="$OUT/mycelium-$name.pdf"
  workdir="target/notebooklm/$name"
  ok=""; build_err=""
  if [ -n "$MYC_DOC" ] && [ -n "$TYPST" ]; then
    rm -rf "$workdir"; mkdir -p "$workdir"
    # Capture the primary build's stderr (do NOT swallow it), so a real bug is distinguishable from
    # an absent-typst fallback (G2: the cause is logged, not hidden).
    if build_err="$("$MYC_DOC" build --repo-root . --manifest "$manifest" --out "$workdir" 2>&1 >/dev/null)"; then
      typ="$(find "$workdir" -name '*.typ' | head -1 || true)"
      if [ -n "$typ" ] && "$TYPST" compile "$typ" "$pdf" 2>"$workdir/typst.err" >/dev/null; then
        ok="pdf"
      else
        build_err="typst compile failed: $(tail -1 "$workdir/typst.err" 2>/dev/null)"
      fi
    fi
  fi
  if [ -z "$ok" ]; then
    [ -n "$build_err" ] && log "  cause (a bug, not absent-typst): $build_err"
    md="$OUT/mycelium-$name.md"
    emit_markdown_bundle "$manifest" "$md"
    rendered+=("$md"); mode_used+=("markdown")
    log "→ $md (fallback)"
  else
    rendered+=("$pdf"); mode_used+=("pdf")
    log "→ $pdf"
  fi
done

# ---- 4. summary (never-silent) ---------------------------------------------
say "summary"
printf '  %-34s %-8s %-10s %s\n' "artifact" "mode" "size" "~words"
for i in "${!rendered[@]}"; do
  f="${rendered[$i]}"; m="${mode_used[$i]}"
  size="$(du -h "$f" 2>/dev/null | cut -f1)"
  words="$(wc -w < "$f" 2>/dev/null || echo '?')"
  printf '  %-34s %-8s %-10s %s\n' "$(basename "$f")" "$m" "$size" "$words"
done
log "wrote ${#rendered[@]} bundle(s) to $OUT/ — upload as NotebookLM sources."
log "note: NotebookLM caps a source at 500k words / 200MB; keep clusters split."
