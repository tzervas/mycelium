#!/usr/bin/env bash
# scripts/docs-assets.sh — docs asset automation: capture -> optimize -> replace-in-place ->
# prune. Regenerates the canonical, committed docs/assets/ screenshot set from a freshly built
# local docsite (scripts/docsite.sh), in BOTH light and dark themes, via Playwright.
#
# Idempotent + deterministic: filenames are STABLE and descriptive (docsite-home-light.png, …) —
# never content-hashed — so a re-run OVERWRITES the same files in place and the working tree never
# accumulates duplicates.
#
# Advisory (like docs-site/docs-book), deliberately NOT part of `just check` — it needs a real
# browser and, on a first run in a fresh environment, network access to fetch one (`npm install` +
# a Chromium download; see scripts/docs-assets/capture.mjs). The companion GATE that IS part of
# `just check` is scripts/checks/docs-assets.sh — a lightweight, browser-free reference-integrity
# check (referenced-but-missing / present-but-orphaned), mirroring the docs/*-index/ drift-gate
# pattern without needing the heavy capture step on every commit.
#
# Honesty (VR-5/G2): every screenshot is a Declared projection of the docsite at capture time; the
# dark-theme captures use a capture-time-only style override (disclosed in capture.mjs's header and
# docs/guide/docsite-preview.md) since the docsite itself does not yet ship a dark stylesheet. If
# Playwright/Chromium can't launch, this script reports exactly why and leaves docs/assets/
# untouched — it never fabricates a screenshot.
#
# Usage: just docs-assets   (preferred)  |  bash scripts/docs-assets.sh
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib.sh"
cd "$REPO_ROOT" || exit 1

section "docs-assets (capture -> optimize -> replace-in-place -> prune)"

ASSETS_DIR="$REPO_ROOT/docs/assets"
mkdir -p "$ASSETS_DIR"

server_pid=""
cleanup() {
  [[ -n "$server_pid" ]] && kill "$server_pid" >/dev/null 2>&1 || true
  rm -f "/tmp/docs-assets-server.$$.log" "/tmp/docs-assets-docsite.$$.log"
}
trap cleanup EXIT

# ── 1/4 — build the site the screenshots are taken FROM ───────────────────────────────────────
section "1/4 build docsite"
if ! bash "$SCRIPT_DIR/docsite.sh" >"/tmp/docs-assets-docsite.$$.log" 2>&1; then
  cat "/tmp/docs-assets-docsite.$$.log"
  fail "docsite build failed — see above; asset capture skipped, docs/assets/ left as-is"
  exit 1
fi
ok "target/docsite/ built"

# ── 2/4 — serve it locally + capture (light + dark) via Playwright ────────────────────────────
section "2/4 capture (Playwright, light + dark)"
CAPTURE_DIR="$SCRIPT_DIR/docs-assets"
if [[ ! -d "$CAPTURE_DIR/node_modules/playwright-core" ]]; then
  if ! have npm; then
    skip "npm not found — cannot install the Playwright capture tooling (scripts/docs-assets/); docs/assets/ left as-is"
    exit 0
  fi
  echo "  installing capture tooling (scripts/docs-assets/) — first run only"
  if ! (cd "$CAPTURE_DIR" && npm install --no-audit --no-fund >/dev/null 2>&1); then
    skip "npm install failed (offline?) — cannot capture; docs/assets/ left as-is"
    exit 0
  fi
fi

PORT="${MYC_DOCS_ASSETS_PORT:-8931}"
python3 -m http.server "$PORT" --directory "$REPO_ROOT/target/docsite" >"/tmp/docs-assets-server.$$.log" 2>&1 &
server_pid=$!
# Wait for the server instead of a fixed sleep (never-silent: loud timeout, not a hang).
up=0
for _ in $(seq 1 30); do
  if curl -sf "http://127.0.0.1:$PORT/index.html" >/dev/null 2>&1; then up=1; break; fi
  sleep 0.2
done
if [[ $up -ne 1 ]]; then
  fail "local docsite server on :$PORT never came up — see /tmp/docs-assets-server.$$.log"
  exit 1
fi

capture_rc=0
node "$CAPTURE_DIR/capture.mjs" --base-url "http://127.0.0.1:$PORT" --site-dir "$REPO_ROOT/target/docsite" --out "$ASSETS_DIR" || capture_rc=$?
kill "$server_pid" >/dev/null 2>&1 || true
server_pid=""

if [[ $capture_rc -ne 0 ]]; then
  fail "capture.mjs exited $capture_rc (see above — no screenshot was fabricated); docs/assets/ left as previously committed"
  exit "$capture_rc"
fi

# ── 3/4 — optimize PNGs (skip-graceful) ────────────────────────────────────────────────────────
section "3/4 optimize"
if have oxipng; then
  if oxipng -o max --strip safe -q "$ASSETS_DIR"/*.png; then
    ok "optimized with oxipng"
  else
    skip "oxipng run failed — PNGs left unoptimized"
  fi
elif have pngquant; then
  opt_ok=1
  for f in "$ASSETS_DIR"/*.png; do
    pngquant --force --skip-if-larger --strip --output "$f" -- "$f" 2>/dev/null || opt_ok=0
  done
  if [[ $opt_ok -eq 1 ]]; then
    ok "optimized with pngquant"
  else
    skip "pngquant had failures on some files — check above"
  fi
else
  skip "neither oxipng nor pngquant installed — PNGs committed unoptimized (install either for smaller assets)"
fi

# ── 4/4 — prune orphans: delete any docs/assets/* not referenced by a committed doc ───────────
section "4/4 prune orphans"
pruned=0
shopt -s nullglob
for f in "$ASSETS_DIR"/*; do
  [[ -f "$f" ]] || continue
  name="$(basename "$f")"
  if ! grep -rlq --include='*.md' -F "$name" docs/ 2>/dev/null; then
    echo "  prune  $name (not referenced by any committed doc)"
    rm -f "$f"
    pruned=$((pruned + 1))
  fi
done
shopt -u nullglob
if [[ $pruned -eq 0 ]]; then ok "no orphans"; else ok "pruned $pruned orphan(s)"; fi

echo
ls -la "$ASSETS_DIR"
ok "docs-assets complete — review \`git status\`/\`git diff --stat\` before committing"
