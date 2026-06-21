#!/usr/bin/env bash
# scripts/docsite.sh — Turnkey local docsite generator (advisory; NOT part of `just check`).
#
# Assembles a single browsable static site under target/docsite/ from three sources:
#   1. Corpus HTML  — `myc-doc build` projects the design corpus (RFCs/ADRs/DNs/specs) into
#                      a content-addressed HTML+JSON view (M-363).
#   2. Agent index  — docs/api-index/ (INDEX.md + index.json); regenerated via `just docs-index`
#                      if python3 is available, otherwise uses the committed snapshot.
#   3. Rustdoc      — `cargo doc --no-deps --workspace` public API; folded in as a sub-tree.
#
# Each section skips gracefully when its tool is absent; the landing page reflects what
# actually built so links are never dead.
#
# Usage:
#   just docs-site          # via the justfile recipe (preferred)
#   bash scripts/docsite.sh # direct invocation
#
# Output: target/docsite/index.html (landing page) + sub-trees: corpus/ rustdoc/ api-index/
#
# WSL browsing: after the script prints the output path, run from that directory:
#   cd target/docsite && python3 -m http.server 8080
# Then open http://localhost:8080 in your Windows browser.
#
# Design rules (house rules honoured here):
#   - Idempotent: safe to re-run; always starts from scratch (rm -rf + rebuild).
#   - Skip-gracefully: missing tool → warn + skip section; site still emits from what's available.
#   - No curl|bash.  No silent failures (set -euo pipefail; every skip is printed).
#   - Output is gitignored (target/); never committed.

set -euo pipefail
source "${BASH_SOURCE%/*}/lib.sh"
cd "$REPO_ROOT" || exit 1

section "docsite (local browsable site — advisory, not a gate)"

OUT="$REPO_ROOT/target/docsite"
rm -rf "$OUT"
mkdir -p "$OUT"

# Track which sections actually built (for the landing page).
HAS_CORPUS=0
HAS_RUSTDOC=0
HAS_API_INDEX=0

# ── 1. Corpus HTML via myc-doc build ──────────────────────────────────────────────────────────────
section "corpus (myc-doc build)"
if ! have cargo; then
  skip "no cargo — corpus section skipped (install Rust to enable)"
else
  CORPUS_OUT="$OUT/corpus"
  mkdir -p "$CORPUS_OUT"
  if cargo run -q -p mycelium-doc --bin myc-doc -- build \
      --repo-root "$REPO_ROOT" \
      --out "$CORPUS_OUT" 2>&1; then
    ok "corpus HTML → $CORPUS_OUT/index.html"
    HAS_CORPUS=1
  else
    skip "myc-doc build failed — corpus section skipped (check above for details)"
    rm -rf "$CORPUS_OUT"
  fi
fi

# ── 2. Agent API index ─────────────────────────────────────────────────────────────────────────────
section "agent API index (docs/api-index/)"
API_INDEX_OUT="$OUT/api-index"
mkdir -p "$API_INDEX_OUT"

# Try to regenerate from source; fall back to the committed snapshot.
REGENERATED=0
if have python3 && [[ -f "$REPO_ROOT/tools/docgen/code_index.py" ]]; then
  if python3 "$REPO_ROOT/tools/docgen/code_index.py" 2>&1; then
    ok "api-index regenerated"
    REGENERATED=1
  else
    skip "docs-index regeneration failed — using committed snapshot"
  fi
else
  skip "python3 or tools/docgen/code_index.py absent — using committed snapshot"
fi

if [[ -f "$REPO_ROOT/docs/api-index/INDEX.md" ]]; then
  cp "$REPO_ROOT/docs/api-index/INDEX.md" "$API_INDEX_OUT/INDEX.md"
  HAS_API_INDEX=1
fi
if [[ -f "$REPO_ROOT/docs/api-index/index.json" ]]; then
  cp "$REPO_ROOT/docs/api-index/index.json" "$API_INDEX_OUT/index.json"
  HAS_API_INDEX=1
fi

# Wrap INDEX.md in a minimal HTML page for in-browser viewing.
if [[ $HAS_API_INDEX -eq 1 ]] && [[ -f "$API_INDEX_OUT/INDEX.md" ]]; then
  python3 - "$API_INDEX_OUT/INDEX.md" "$API_INDEX_OUT/index.html" <<'PYEOF' 2>/dev/null || true
import sys, re, html, pathlib

src = pathlib.Path(sys.argv[1]).read_text()
out_path = pathlib.Path(sys.argv[2])

# Minimal Markdown-to-HTML: headings, tables, code spans, bold, paragraphs.
def md2html(text):
    lines = text.split('\n')
    out = []
    in_table = False
    for line in lines:
        # Headings
        m = re.match(r'^(#{1,6})\s+(.*)', line)
        if m:
            if in_table:
                out.append('</tbody></table>'); in_table = False
            lvl = len(m.group(1))
            out.append(f'<h{lvl}>{inline(m.group(2))}</h{lvl}>')
            continue
        # Table rows
        if line.startswith('|'):
            cells = [c.strip() for c in line.strip('|').split('|')]
            is_sep = all(re.match(r'^[-:]+$', c) for c in cells if c)
            if is_sep:
                out.append('<tbody>'); in_table = True; continue
            row_html = ''.join(f'<td>{inline(c)}</td>' for c in cells)
            if not in_table:
                out.append('<table><thead><tr>' + ''.join(f'<th>{inline(c)}</th>' for c in cells) + '</tr></thead>')
            else:
                out.append(f'<tr>{row_html}</tr>')
            continue
        if in_table and not line.startswith('|'):
            out.append('</tbody></table>'); in_table = False
        # Blank line
        if not line.strip():
            out.append('<br>')
            continue
        # Blockquote
        if line.startswith('>'):
            out.append(f'<blockquote>{inline(line[1:].strip())}</blockquote>')
            continue
        out.append(f'<p>{inline(line)}</p>')
    if in_table:
        out.append('</tbody></table>')
    return '\n'.join(out)

def inline(text):
    # Code spans
    text = re.sub(r'`([^`]+)`', lambda m: f'<code>{html.escape(m.group(1))}</code>', text)
    # Bold
    text = re.sub(r'\*\*([^*]+)\*\*', r'<strong>\1</strong>', text)
    # Links
    text = re.sub(r'\[([^\]]+)\]\(([^)]+)\)', r'<a href="\2">\1</a>', text)
    return text

body = md2html(src)
page = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Agent Code Index — Mycelium</title>
<style>
:root{{--fg:#1a1a2e;--bg:#fdfdfd;--accent:#3a5;--dim:#667;--code:#f4f4f8}}
*{{box-sizing:border-box}}
body{{margin:0;font:16px/1.6 system-ui,-apple-system,Segoe UI,Roboto,sans-serif;color:var(--fg);background:var(--bg)}}
header,main,footer{{max-width:70rem;margin:0 auto;padding:1rem 1.25rem}}
header{{border-bottom:2px solid var(--accent)}}
a{{color:var(--accent)}}
h1,h2,h3,h4,h5,h6{{line-height:1.25}}
table{{border-collapse:collapse;width:100%;font-size:.9em}}
th,td{{border:1px solid #ddd;padding:.4rem .6rem;text-align:left}}
th{{background:#f0f4f0}}
code{{font:0.9em ui-monospace,SFMono-Regular,Menlo,monospace;background:var(--code);padding:.1em .3em;border-radius:3px}}
blockquote{{border-left:3px solid var(--accent);margin:1rem 0;padding:.5rem 1rem;color:var(--dim)}}
footer{{color:var(--dim);font-size:.85rem;border-top:1px solid #ddd;margin-top:2rem}}
</style>
</head>
<body>
<header><h1>Mycelium — Agent Code Index</h1>
<p><a href="../index.html">← Back to docsite landing</a></p></header>
<main>{body}</main>
<footer>Empirical/Declared — line/regex heuristic; source is ground truth. Use this index to find where to Read, not as an authoritative reference.</footer>
</body></html>"""
out_path.write_text(page)
PYEOF
  if [[ -f "$API_INDEX_OUT/index.html" ]]; then
    ok "api-index HTML wrapper → $API_INDEX_OUT/index.html"
  else
    # Fallback: bare link to INDEX.md
    HAS_API_INDEX=1
    ok "api-index (raw files only — python3 unavailable for HTML wrapper)"
  fi
fi

if [[ $HAS_API_INDEX -eq 1 ]]; then
  ok "agent index → $API_INDEX_OUT/"
else
  skip "no api-index files found — api-index section skipped"
fi

# ── 3. Rustdoc ────────────────────────────────────────────────────────────────────────────────────
section "rustdoc (cargo doc --no-deps --workspace)"
if ! have cargo; then
  skip "no cargo — rustdoc section skipped"
else
  RUSTDOC_SRC="$REPO_ROOT/target/doc"
  # Run cargo doc; pipe output; exit code matters for skip decision only.
  if cargo doc --no-deps --workspace --quiet 2>&1; then
    if [[ -d "$RUSTDOC_SRC" ]]; then
      RUSTDOC_OUT="$OUT/rustdoc"
      # Symlink target/doc into the site (avoids copying GBs; both are gitignored).
      ln -sfn "$RUSTDOC_SRC" "$RUSTDOC_OUT"
      ok "rustdoc → $RUSTDOC_OUT/ (symlink → $RUSTDOC_SRC)"
      HAS_RUSTDOC=1
    else
      skip "cargo doc succeeded but target/doc not found — rustdoc section skipped"
    fi
  else
    skip "cargo doc failed — rustdoc section skipped (check above for details)"
  fi
fi

# ── 4. Landing page ───────────────────────────────────────────────────────────────────────────────
section "landing page"

CORPUS_LINK=""
RUSTDOC_LINK=""
API_INDEX_LINK=""
MISSING_SECTIONS=""

if [[ $HAS_CORPUS -eq 1 ]]; then
  CORPUS_LINK='<li><a href="corpus/index.html"><strong>Corpus</strong> — Design docs: RFCs, ADRs, design notes, specs (myc-doc HTML view)</a></li>'
else
  MISSING_SECTIONS="${MISSING_SECTIONS}<li>Corpus (myc-doc build skipped — cargo or myc-doc unavailable)</li>"
fi

if [[ $HAS_API_INDEX -eq 1 ]]; then
  if [[ -f "$OUT/api-index/index.html" ]]; then
    API_INDEX_LINK='<li><a href="api-index/index.html"><strong>Agent code index</strong> — Symbol table: crate / file:line / guarantee tag (grep-friendly)</a></li>'
  else
    API_INDEX_LINK='<li><a href="api-index/INDEX.md"><strong>Agent code index</strong> — Symbol table (raw Markdown; open in a Markdown viewer)</a></li>'
  fi
else
  MISSING_SECTIONS="${MISSING_SECTIONS}<li>Agent code index (docs/api-index/ not found)</li>"
fi

if [[ $HAS_RUSTDOC -eq 1 ]]; then
  # Try to find the workspace-level index; fall back to top-level.
  RUSTDOC_INDEX="rustdoc/index.html"
  RUSTDOC_LINK="<li><a href=\"${RUSTDOC_INDEX}\"><strong>Rustdoc</strong> — Public Rust API reference (cargo doc)</a></li>"
else
  MISSING_SECTIONS="${MISSING_SECTIONS}<li>Rustdoc (cargo doc skipped)</li>"
fi

MISSING_HTML=""
if [[ -n "$MISSING_SECTIONS" ]]; then
  MISSING_HTML="<section class=\"missing\"><h2>Skipped sections</h2><ul>${MISSING_SECTIONS}</ul>
<p>Run <code>just setup</code> then <code>just docs-site</code> again to enable skipped sections.</p></section>"
fi

DATE_NOW="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

cat > "$OUT/index.html" <<HTMLEOF
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Mycelium — Local Docsite</title>
<style>
:root{--fg:#1a1a2e;--bg:#fdfdfd;--accent:#3a5;--dim:#667;--code:#f4f4f8}
*{box-sizing:border-box}
body{margin:0;font:16px/1.6 system-ui,-apple-system,Segoe UI,Roboto,sans-serif;color:var(--fg);background:var(--bg)}
header,main,footer{max-width:54rem;margin:0 auto;padding:1rem 1.25rem}
header{border-bottom:2px solid var(--accent)}
a{color:var(--accent)}
h1,h2,h3,h4,h5,h6{line-height:1.25}
ul{padding-left:1.5rem}
li{margin:.5rem 0}
.browse{background:#f0f8f0;border:1px solid var(--accent);border-radius:6px;padding:1rem 1.25rem;margin:1.5rem 0}
.missing{background:#fff8f0;border:1px solid #c93;border-radius:6px;padding:1rem 1.25rem;margin:1.5rem 0}
code{font:0.9em ui-monospace,SFMono-Regular,Menlo,monospace;background:var(--code);padding:.15em .4em;border-radius:3px}
pre{background:var(--code);padding:.75rem 1rem;border-radius:6px;overflow:auto}
footer{color:var(--dim);font-size:.85rem;border-top:1px solid #ddd;margin-top:2rem}
</style>
</head>
<body>
<header>
  <h1>Mycelium — Local Docsite</h1>
  <p>A unified, locally-browsable view of the Mycelium corpus and codebase. Generated ${DATE_NOW}.</p>
</header>
<main>
  <section>
    <h2>Documentation</h2>
    <ul>
      ${CORPUS_LINK}
      ${API_INDEX_LINK}
      ${RUSTDOC_LINK}
    </ul>
  </section>

  ${MISSING_HTML}

  <div class="browse">
    <h2>Browsing on WSL</h2>
    <p>Serve the site with Python's built-in HTTP server from the output directory, then open the URL in your Windows browser:</p>
    <pre>cd target/docsite
python3 -m http.server 8080</pre>
    <p>Then open <a href="http://localhost:8080">http://localhost:8080</a> in your browser.</p>
    <p>Or navigate directly to <code>target/docsite/index.html</code> — most sections work from the filesystem too (except rustdoc, which uses absolute links).</p>
  </div>
</main>
<footer>
  Generated by <code>just docs-site</code> (<code>scripts/docsite.sh</code>).
  Advisory output — not committed, not gated. Output is in <code>target/docsite/</code> (gitignored).
  Corpus projection is honest: a projection of the cited corpus, never a parallel truth (ADR-003/G11).
</footer>
</body>
</html>
HTMLEOF

ok "landing page → $OUT/index.html"

# ── Done ──────────────────────────────────────────────────────────────────────────────────────────
echo
printf '%s========================================%s\n' "$C_DIM" "$C_RST"
ok "docsite ready — $OUT/index.html"
echo
printf '  Browse on WSL:\n'
printf '    cd %s\n' "$OUT"
printf '    python3 -m http.server 8080\n'
printf '  Then open http://localhost:8080 in your Windows browser.\n'
