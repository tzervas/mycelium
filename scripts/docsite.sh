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
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib.sh"
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
if have python3 && [[ -f "$REPO_ROOT/tools/docgen/code_index.py" ]]; then
  if python3 "$REPO_ROOT/tools/docgen/code_index.py" 2>&1; then
    ok "api-index regenerated"
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

# ── 3.5. Language-reference landing page ─────────────────────────────────────────────────────────
# Generates lang-ref/index.html: a single browsable index linking the grammar, lexicon, and
# per-module stdlib specs from the corpus. Links are relative to corpus/pages/ so they work
# both from the filesystem and from the http.server. Emitted only when the corpus built (the
# source pages must exist). Gracefully skipped (warn only) when corpus is absent.
section "language reference landing page (lang-ref/index.html)"

HAS_LANG_REF=0
LANG_REF_OUT="$OUT/lang-ref"

if [[ $HAS_CORPUS -eq 1 ]]; then
  mkdir -p "$LANG_REF_OUT"
  LANG_REF_DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

  # ── stdlib module entries ──────────────────────────────────────────────────────────────────────
  # Each entry: "page-slug|Label|Ring|Grounding"
  # Slugs must match the corpus/pages/<slug>.html files that myc-doc emits.
  declare -a STDLIB_ENTRIES=(
    "core|core / prelude|Ring 0 — kernel-adjacent|RFC-0001; value model + guarantee lattice"
    "swap|swap|Ring 1 — Tier A|RFC-0002; the never-silent swap certificate"
    "ternary|ternary|Ring 1 — Tier A|RFC-0001 §4; balanced-ternary"
    "dense|dense|Ring 1 — Tier A|RFC-0001 §4.1; dense embedding"
    "select|select / explain|Ring 1 — Tier A|RFC-0005 / ADR-006; selection policy + EXPLAIN"
    "content|content / hash|Ring 1 — Tier A|ADR-003; content-addressed identity"
    "numerics|numerics|Ring 1 — Tier A|ADR-010/011; verified numerics"
    "vsa|vsa / hdc|Ring 1 — Tier A|RFC-0003/0009; hypervector HDC"
    "diag|diag|Ring 1 — Tier A|RFC-0013; structured diagnostics"
    "recover|recover|Ring 1 — Tier A|RFC-0014; declarative error recovery"
    "runtime|runtime / colony|Ring 1 — Tier A (Phase-7 gated)|RFC-0008; concurrency model"
    "spore|spore|Ring 1 — Tier A|ADR-013; deployable artifact"
    "sys|sys (OS/FFI floor)|Ring 1 — Tier A|RFC-0016 §8-Q6; ADR-014"
    "iter|iter|Ring 2 — Tier B|RFC-0007 §4.8; total bounded iteration"
    "math|math|Ring 2 — Tier B|ADR-010; approximations carry their tag"
    "error|error / option / result|Ring 2 — Tier B|RFC-0013; propagation floor"
    "cmp|cmp / convert|Ring 2 — Tier B|RFC-0001; lossy convert is explicit"
    "fmt|fmt|Ring 2 — Tier B|G11; dual human/machine projection"
    "collections|collections|Ring 2 — Tier B|RFC-0001; value-semantic, no silent reorder"
    "text|text / string|Ring 2 — Tier B|parse -> Result; lossy encoding explicit"
    "io|io + serialize|Ring 2 — Tier B|LR-8; substrate single-consumption; canonical JSON"
    "fs|fs|Ring 2 — Tier B|ADR-014; every path/permission failure explicit"
    "time|time|Ring 2 — Tier B|monotonic vs wall is a typed distinction"
    "rand|rand|Ring 2 — Tier B|RT3; nondeterminism reified/named"
    "testing|testing|Ring 2 — Tier B|a skipped check is reported, never a silent pass"
  )

  STDLIB_ROWS=""
  for entry in "${STDLIB_ENTRIES[@]}"; do
    IFS='|' read -r slug label ring grounding <<< "$entry"
    page="corpus/pages/${slug}.html"
    if [[ -f "$OUT/$page" ]]; then
      STDLIB_ROWS="${STDLIB_ROWS}<tr><td><a href=\"../${page}\"><code>${label}</code></a></td><td>${ring}</td><td>${grounding}</td></tr>"
    else
      STDLIB_ROWS="${STDLIB_ROWS}<tr><td><code>${label}</code> <em>(page not in corpus)</em></td><td>${ring}</td><td>${grounding}</td></tr>"
    fi
  done

  cat > "$LANG_REF_OUT/index.html" <<'LANGREF_CSS'
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Mycelium &mdash; Language Reference</title>
<style>
:root{--fg:#1a1a2e;--bg:#fdfdfd;--accent:#3a5;--dim:#667;--code:#f4f4f8}
*{box-sizing:border-box}
body{margin:0;font:16px/1.6 system-ui,-apple-system,Segoe UI,Roboto,sans-serif;color:var(--fg);background:var(--bg)}
header,main,footer{max-width:70rem;margin:0 auto;padding:1rem 1.25rem}
header{border-bottom:2px solid var(--accent)}
a{color:var(--accent)}
h1,h2,h3,h4,h5,h6{line-height:1.25}
ul{padding-left:1.5rem}
li{margin:.4rem 0}
table{border-collapse:collapse;width:100%;font-size:.9em;margin:1rem 0}
th,td{border:1px solid #ddd;padding:.4rem .7rem;text-align:left;vertical-align:top}
th{background:#f0f4f0;font-weight:600}
code{font:0.9em ui-monospace,SFMono-Regular,Menlo,monospace;background:var(--code);padding:.1em .3em;border-radius:3px}
pre{background:var(--code);padding:.75rem 1rem;border-radius:6px;overflow:auto;font-size:.9em}
.note{background:#f0f8f0;border:1px solid var(--accent);border-radius:6px;padding:.75rem 1rem;margin:1.5rem 0;font-size:.9em}
.warn{background:#fff8f0;border:1px solid #c93;border-radius:6px;padding:.75rem 1rem;margin:.75rem 0;font-size:.9em}
footer{color:var(--dim);font-size:.85rem;border-top:1px solid #ddd;margin-top:2rem}
</style>
</head>
LANGREF_CSS
  # Append the body with shell-expanded variables (separate heredoc, no single-quoting)
  cat >> "$LANG_REF_OUT/index.html" <<LANGREFBODY
<body>
<header>
  <h1>Mycelium &mdash; Language Reference</h1>
  <p>Grammar, reserved-word lexicon, and per-module standard-library specifications.
     All links point to the corpus HTML projection &mdash; the <strong>source files are ground truth</strong>
     (Empirical/Declared; this index is an orientation aid, not a parallel spec). Generated ${LANG_REF_DATE}.</p>
  <p><a href="../index.html">&larr; Back to docsite landing</a></p>
</header>
<main>

<section>
<h2>1. Surface grammar</h2>
<div class="note">Normative oracle: <code>docs/spec/grammar/mycelium.ebnf</code> (W3C EBNF, v0, L1-facing fragment).
The conformance corpus under <code>docs/spec/grammar/conformance/</code> is the machine-verified gate &mdash;
every <code>accept/</code> program must parse; every <code>reject/</code> program must fail with an explicit diagnostic.
<code>scripts/checks/grammar.sh</code> and <code>crates/mycelium-l1</code> <code>tests/conformance.rs</code> are the checks.</div>
<ul>
  <li><a href="../corpus/index.html">Grammar artifacts README</a> (find it in the corpus index &mdash; deep page anchors are unstable) &mdash; conformance corpus layout + checking</li>
  <li><a href="../corpus/pages/rfc-0006-surface-language-and-term-layering.html">RFC-0006 &mdash; Surface Language and Term Layering</a> &mdash; L0&rarr;L1&rarr;L2/L3 layer cake; invariants S1&ndash;S6 (Accepted)</li>
  <li><a href="../corpus/pages/rfc-0007-l1-kernel-calculus.html">RFC-0007 &mdash; L1 Kernel Calculus</a> &mdash; ten-node budget, typing, totality (Accepted)</li>
  <li><a href="../corpus/pages/rfc-0020-l2-surface-term-language.html">RFC-0020 &mdash; L2 Surface Term Language</a> &mdash; L2 elaboration surface (verify current status in doc)</li>
  <li><a href="../corpus/pages/specification.html">Specification</a> &mdash; the full Mycelium specification (corpus view)</li>
</ul>

<h3>Key grammar productions (curated excerpt &mdash; <code>docs/spec/grammar/mycelium.ebnf</code> is the normative source)</h3>
<pre>program        ::= nodule_header item*
nodule_header  ::= 'nodule' path
item           ::= use_item | default_item | type_item | trait_item | fn_item
fn_item        ::= 'thaw'? 'fn' Ident type_params? '(' params? ')' '-&gt;' type_ref '=' expr
type_ref       ::= base_type ('@' strength)?
swap_expr      ::= 'swap' '(' expr ',' 'to' ':' type_ref ',' 'policy' ':' path ')'
wild_expr      ::= 'wild' '{' expr '}'
for_expr       ::= 'for' Ident 'in' app_expr ',' Ident '=' app_expr '=&gt;' expr</pre>
<p>Literals: <code>0b1011_0010</code> (binary) &middot; <code>&lt;+0--0&gt;</code> (balanced ternary, MSB-first) &middot;
bare decimal <code>42</code> (universal-until-elaboration) &middot; <code>[1.5, -2.25]</code> (list).
No defaulting across representation families (Q6).</p>
</section>

<section>
<h2>2. Reserved-word lexicon</h2>
<div class="note">Sources of truth: <code>crates/mycelium-l1/src/token.rs</code> (the actual <code>keyword()</code> function)
and <code>docs/spec/grammar/mycelium.ebnf</code>. The tables below are an orientation aid &mdash; re-verify against
<code>token.rs</code> after any lexer change (ground truth rule: VR-5 / G2).</div>
<ul>
  <li><a href="../corpus/pages/lexicon-reference.html">Lexicon Reference</a> &mdash; terse catalog with mnemonics and tier table</li>
  <li><a href="../corpus/pages/dn-02-fungal-lexicon-and-reserved-words.html">DN-02 &mdash; Fungal Lexicon and Reserved Words</a> &mdash; naming LAW + three-test gate (Resolved 2026-06-10)</li>
  <li><a href="../corpus/pages/dn-03-lexicon-amendment-surface-and-runtime-forms.html">DN-03 &mdash; Lexicon Amendment: Surface and Runtime Forms</a> &mdash; runtime names; one-name-per-term rule (Resolved 2026-06-10)</li>
  <li><a href="../corpus/pages/dn-06-static-organization-and-dynamic-grouping-lexicon.html">DN-06 &mdash; Static Organization and Dynamic Grouping Lexicon</a> &mdash; <code>phylum</code> / <code>nodule</code> / <code>colony</code> (Resolved 2026-06-16)</li>
  <li><a href="../corpus/pages/glossary.html">Glossary</a> &mdash; per-term definitions with normative citations</li>
</ul>

<h3>Active keywords (in lexer <code>keyword()</code> and consumed by a construct)</h3>
<table>
<thead><tr><th>Keyword</th><th>Concept</th><th>Themed?</th><th>Normative source</th></tr></thead>
<tbody>
<tr><td><code>nodule</code></td><td>The basic static organizational unit; opens a program</td><td>themed</td><td>DN-06; RFC-0006</td></tr>
<tr><td><code>use</code></td><td>Import</td><td>conventional</td><td>DN-02 §3</td></tr>
<tr><td><code>type</code></td><td>Data-type (sum) declaration</td><td>conventional</td><td>DN-02 §7</td></tr>
<tr><td><code>trait</code></td><td>Typeclass / behavior set</td><td>conventional</td><td>DN-02 §7</td></tr>
<tr><td><code>fn</code></td><td>Function definition</td><td>conventional</td><td>DN-02 §3</td></tr>
<tr><td><code>thaw</code></td><td>De-maturation: keeps one <code>fn</code> interpreted inside a matured scope</td><td>conventional-clearest</td><td>RFC-0017 §4.3/§5; DN-03</td></tr>
<tr><td><code>let</code> &hellip; <code>in</code></td><td>Local binding</td><td>conventional</td><td>DN-02 §3</td></tr>
<tr><td><code>if</code> / <code>then</code> / <code>else</code></td><td>Conditional</td><td>conventional</td><td>DN-02 §3</td></tr>
<tr><td><code>match</code></td><td>Pattern match</td><td>conventional</td><td>DN-02 §3</td></tr>
<tr><td><code>for</code></td><td>Bounded iteration sugar (total by construction)</td><td>conventional</td><td>RFC-0007 §4.8; DN-03</td></tr>
<tr><td><code>swap</code></td><td>The never-silent representation change</td><td>native corpus term</td><td>RFC-0001 §4.5; RFC-0002</td></tr>
<tr><td><code>default</code> / <code>paradigm</code> / <code>with</code></td><td>Ambient representation (nodule-scope and block-scope)</td><td>conventional</td><td>RFC-0012 §4.2/§4.4</td></tr>
<tr><td><code>wild</code></td><td>Denied-by-default unsafe block</td><td>themed</td><td>DN-02 §5/§7</td></tr>
<tr><td><code>spore</code></td><td>Reconstruction manifest; deployable artifact form</td><td>themed</td><td>DN-02 §2/§7; ADR-013</td></tr>
<tr><td><code>to</code> / <code>policy</code></td><td>Swap target and policy labels</td><td>conventional</td><td>grammar</td></tr>
<tr><td><code>Binary</code> / <code>Ternary</code> / <code>Dense</code> / <code>VSA</code></td><td>Representation types</td><td>themed/conventional</td><td>RFC-0001; grammar</td></tr>
<tr><td><code>Substrate</code></td><td>Affine external resource kind (consumed exactly once)</td><td>themed</td><td>DN-02 §2; LR-8</td></tr>
<tr><td><code>Exact</code> / <code>Proven</code> / <code>Empirical</code> / <code>Declared</code></td><td>Guarantee strength tags (the honesty lattice)</td><td>formal</td><td>RFC-0001; DN-02 §7</td></tr>
<tr><td><code>F16</code> / <code>BF16</code> / <code>F32</code> / <code>F64</code></td><td>Scalar kind keywords for Dense</td><td>conventional</td><td>grammar</td></tr>
<tr><td><code>Sparse</code></td><td>Sparsity qualifier for VSA</td><td>conventional</td><td>grammar</td></tr>
</tbody>
</table>

<h3>Reserved-not-active (lex as keywords; no construct consumes them yet)</h3>
<table>
<thead><tr><th>Keyword</th><th>Future meaning</th><th>Normative source</th></tr></thead>
<tbody>
<tr><td><code>phylum</code></td><td>Library-scale grouping above nodules; activates when its construct lands (RFC-0006)</td><td>DN-06; RFC-0006</td></tr>
<tr><td><code>matured</code></td><td>RFC-0017 header/manifest key for scope-level AOT promotion; a teaching ParseError at item position &mdash; no L1 item/expression construct consumes it (not a fn modifier)</td><td>RFC-0017; DN-02 §7</td></tr>
<tr><td><code>colony</code></td><td>Dynamic runtime grouping of active <code>hypha</code>; reassigned from former static meaning (DN-06)</td><td>DN-06; RFC-0008 §4.7</td></tr>
<tr><td><code>hypha</code></td><td>Reserved keyword &mdash; produces a ParseError; not yet active as a construct (RFC-0008 §4.5)</td><td>DN-03 §4; RFC-0008 §4.5; M-665</td></tr>
<tr><td><code>fuse</code></td><td>Reserved keyword &mdash; produces a ParseError; not yet active as a construct (RFC-0008 §4.5)</td><td>DN-03 §4; RFC-0008 §4.5/RT6; M-665</td></tr>
<tr><td><code>mesh</code></td><td>Reserved keyword &mdash; produces a ParseError; not yet active as a construct (RFC-0008 §4.5)</td><td>DN-03 §4; RFC-0008 §4.5/RT5; M-665</td></tr>
<tr><td><code>graft</code></td><td>Reserved keyword &mdash; produces a ParseError; not yet active as a construct (RFC-0008 §4.5)</td><td>DN-03 §4; RFC-0008 §4.5/RT4; M-665</td></tr>
<tr><td><code>cyst</code></td><td>Reserved keyword &mdash; produces a ParseError; not yet active as a construct (RFC-0008 §4.5)</td><td>DN-03 §4; RFC-0008 §4.5/RT2; M-665</td></tr>
<tr><td><code>xloc</code></td><td>Reserved keyword &mdash; produces a ParseError; not yet active as a construct (RFC-0008 §4.5)</td><td>DN-03 §4; RFC-0008 §4.5/RT1; M-665</td></tr>
<tr><td><code>forage</code></td><td>Reserved keyword &mdash; produces a ParseError; not yet active as a construct (RFC-0008 §4.5)</td><td>DN-03 §4; RFC-0008 §4.5/RT3; M-665</td></tr>
<tr><td><code>backbone</code></td><td>Reserved keyword &mdash; produces a ParseError; not yet active as a construct (RFC-0008 §4.5)</td><td>DN-03 §4; RFC-0008 §4.5/RT3; M-665</td></tr>
<tr><td><code>tier</code></td><td>Reserved keyword &mdash; produces a ParseError; not yet active as a construct (RFC-0008 §4.5)</td><td>DN-03 §4; RFC-0008 §4.5; M-665</td></tr>
<tr><td><code>reclaim</code></td><td>Reserved keyword &mdash; produces a ParseError; not yet active as a construct (RFC-0008 §4.5)</td><td>DN-03 §4; RFC-0008 §4.5/RT7; M-665</td></tr>
</tbody>
</table>
<div class="warn">Using any of the above as identifiers is a parse error (they lex as keywords &mdash; G2: never silent). See <code>conformance/reject/10-reserved-not-active.myc</code> and <code>conformance/reject/12-runtime-vocab-reserved-not-active.myc</code>.</div>

<h3>Ratified, not yet lexed (lex as ordinary identifiers today)</h3>
<table>
<thead><tr><th>Keyword</th><th>Future meaning</th><th>Normative source</th></tr></thead>
<tbody>
<tr><td><code>impl</code></td><td>Inherent methods on a type</td><td>DN-03 §1</td></tr>
<tr><td><code>consume</code></td><td>Acquire exclusive ownership of an affine <code>substrate</code></td><td>DN-03 §1</td></tr>
<tr><td><code>grow</code></td><td>Derive-like generative capability extension</td><td>DN-03 §1</td></tr>
</tbody>
</table>

<h3>Not reserved (DN-02 §6)</h3>
<p><code>while</code> &middot; <code>loop</code> &middot; <code>break</code> &middot; <code>continue</code> &middot; <code>return</code> &mdash;
unbounded iteration undermines the divergence bit; the toolchain emits teaching diagnostics pointing at
recursion or <code>for</code> when they appear.</p>
</section>

<section>
<h2>3. Standard-library module specifications</h2>
<div class="note">25/25 module specs <strong>Accepted</strong> as of 2026-06-21 (DN-07; DN-16 re-audit).
The Mycelium-lang migration half waits on the KC-2-gated L3 authoring surface (M-502). Each spec fixes
scope, exported-op surface, and the RFC-0016 §4.5 guarantee matrix &mdash; the per-op contract, asserted in tests.</div>
<ul>
  <li><a href="../corpus/pages/rfc-0016-core-library-and-standard-library.html">RFC-0016 &mdash; Core Library and Standard Library</a> &mdash; the contract + taxonomy keystone (Accepted)</li>
  <li><a href="../corpus/index.html">stdlib spec index README</a> (find it in the corpus index &mdash; deep page anchors are unstable) &mdash; module index, ring layering, cross-module reconciliation</li>
</ul>

<table>
<thead><tr><th>Module</th><th>Ring / tier</th><th>Grounding crux</th></tr></thead>
<tbody>
${STDLIB_ROWS}
</tbody>
</table>
</section>

<section>
<h2>4. Language-layer design documents</h2>
<ul>
  <li><a href="../corpus/pages/rfc-0001-core-ir-and-metadata-schema.html">RFC-0001 &mdash; Core IR and Metadata Schema</a> &mdash; L0 frozen trusted base; value model; guarantee lattice</li>
  <li><a href="../corpus/pages/rfc-0007-l1-kernel-calculus.html">RFC-0007 &mdash; L1 Kernel Calculus</a> &mdash; ten-node budget (Lam/App/Construct/Match/Fix); totality</li>
  <li><a href="../corpus/pages/rfc-0012-ambient-representation-and-scoped-overrides.html">RFC-0012 &mdash; Ambient Representation and Scoped Overrides</a> &mdash; <code>default paradigm</code> / <code>with paradigm</code></li>
  <li><a href="../corpus/pages/rfc-0017-maturation-scope-and-de-maturation.html">RFC-0017 &mdash; Maturation Scope and De-maturation</a> &mdash; <code>matured</code> scope + <code>thaw fn</code></li>
  <li><a href="../corpus/pages/rfc-0019-traits-and-parametric-polymorphism.html">RFC-0019 &mdash; Traits and Parametric Polymorphism</a></li>
  <li><a href="../corpus/pages/nodule-header-and-project-manifest.html">Nodule Header and Project Manifest spec</a> &mdash; <code>// @key: value</code> schema + <code>mycelium-proj.toml</code></li>
  <li><a href="../corpus/pages/example-programs-reference.html">Example Programs Reference</a> &mdash; annotated <code>.myc</code> examples (note §Grounding: some pre-ratification)</li>
</ul>
</section>

</main>
<footer>
Generated by <code>just docs-site</code> (<code>scripts/docsite.sh</code>). Advisory &mdash; not committed, not gated.
Normative ground truth: <code>docs/spec/grammar/mycelium.ebnf</code>, <code>crates/mycelium-l1/src/token.rs</code>,
<code>docs/spec/stdlib/</code> (the EBNF + lexer + specs decide; <code>.claude/memory/lang-lexicon-syntax.md</code> is a non-normative maintenance note). This page is a hand-curated orientation snapshot (Empirical/Declared);
verify normative claims in the corpus source files. ADR-003/G11: a projection of the corpus, never a parallel truth.
</footer>
</body>
</html>
LANGREFBODY

  ok "language reference landing page → $LANG_REF_OUT/index.html"
  HAS_LANG_REF=1
else
  skip "corpus not built — lang-ref landing page skipped (run just docs-site with cargo to enable)"
fi

# ── 4. Landing page ───────────────────────────────────────────────────────────────────────────────
section "landing page"

CORPUS_LINK=""
LANG_REF_LINK=""
RUSTDOC_LINK=""
API_INDEX_LINK=""
MISSING_SECTIONS=""

if [[ $HAS_CORPUS -eq 1 ]]; then
  CORPUS_LINK='<li><a href="corpus/index.html"><strong>Corpus</strong> — Design docs: RFCs, ADRs, design notes, specs (myc-doc HTML view)</a></li>'
else
  MISSING_SECTIONS="${MISSING_SECTIONS}<li>Corpus (myc-doc build skipped — cargo or myc-doc unavailable)</li>"
fi

if [[ $HAS_LANG_REF -eq 1 ]]; then
  LANG_REF_LINK='<li><a href="lang-ref/index.html"><strong>Language reference</strong> — Grammar, reserved-word lexicon, and stdlib module specs (browsable index)</a></li>'
else
  MISSING_SECTIONS="${MISSING_SECTIONS}<li>Language reference (requires corpus; run just docs-site with cargo)</li>"
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
      ${LANG_REF_LINK}
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
