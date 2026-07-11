//! The static-HTML renderer (spec §8.1 — static HTML path). One reviewed template (§5): a `<header>`
//! bar, a **persistent navigation `<nav>` sidebar** with client-side search, a level-graded `<main>`
//! within a readable measure, an **"on this page" table of contents**, and a provenance `<footer>` —
//! **semantic HTML by construction** (the §4.1 legibility/accessibility bar: heading order never
//! skips, code carries a `language-*` class, every nav is labelled). Every node element carries
//! `data-cid="blake3:…"`, its content address — the hook the dual-projection-parity lint checks
//! against the JSON view.
//!
//! The shared visual language (typography, light/dark palettes, tables, syntax colours) lives in one
//! self-contained, offline stylesheet ([`crate::theme::READING_CSS`]); Mycelium code examples are
//! coloured by the trusted L1 lexer ([`crate::highlight`]) with a never-silent plain-text fallback.

use crate::emit::{html_escape, Artifacts};
use crate::highlight;
use crate::inline::{self, Span};
use crate::ir::{DocModel, Node, Payload, SourceKind, XrefResolution};
use crate::theme;

/// Render parsed inline [`Span`]s to HTML: `<strong>`/`<em>`/`<code class="inl">`, and `<a class="x">`
/// for **external** links only (internal/relative links render as their text — their resolved
/// navigation is the `Xref` sibling node, so the inline path never emits a dead `.md` href). Every
/// text/code run is HTML-escaped (never raw markup injection).
fn render_inline_html(spans: &[Span<'_>]) -> String {
    let mut out = String::new();
    for span in spans {
        match span {
            Span::Text(t) => out.push_str(&html_escape(t)),
            Span::Code(c) => {
                out.push_str("<code class=\"inl\">");
                out.push_str(&html_escape(c));
                out.push_str("</code>");
            }
            Span::Strong(inner) => {
                out.push_str("<strong>");
                out.push_str(&render_inline_html(inner));
                out.push_str("</strong>");
            }
            Span::Em(inner) => {
                out.push_str("<em>");
                out.push_str(&render_inline_html(inner));
                out.push_str("</em>");
            }
            Span::Link { text, href } => {
                if inline::is_external(href) {
                    out.push_str(&format!("<a class=\"x\" href=\"{}\">", html_escape(href)));
                    out.push_str(&render_inline_html(text));
                    out.push_str("</a>");
                } else {
                    out.push_str(&render_inline_html(text));
                }
            }
        }
    }
    out
}

/// Render parsed inline [`Span`]s to **plain escaped text** (formatting and links stripped) — for a
/// `<title>`, the nav sidebar, and the "on this page" ToC, where inline HTML tags would be invalid
/// (a `<title>`) or produce nested `<a>` (a ToC/sidebar link). So a heading with `**bold**` still
/// reads cleanly as `bold` in those places, never as literal `**bold**`.
fn render_inline_text(spans: &[Span<'_>]) -> String {
    let mut out = String::new();
    for span in spans {
        match span {
            Span::Text(t) | Span::Code(t) => out.push_str(&html_escape(t)),
            Span::Strong(inner) | Span::Em(inner) => out.push_str(&render_inline_text(inner)),
            Span::Link { text, .. } => out.push_str(&render_inline_text(text)),
        }
    }
    out
}

/// Parse + render inline markdown in `text` to HTML (the common one-shot for prose/cells/headings).
fn inline_html(text: &str) -> String {
    render_inline_html(&inline::parse(text))
}

/// Parse + render inline markdown in `text` to plain escaped text (nav labels / `<title>`).
fn inline_text(text: &str) -> String {
    render_inline_text(&inline::parse(text))
}

/// The corpus-family groups, in navigation order (shared by the index and the sidebar tree).
const GROUPS: &[(SourceKind, &str)] = &[
    (SourceKind::Spec, "Specifications & contracts"),
    (SourceKind::Rfc, "RFCs"),
    (SourceKind::Adr, "Architecture decisions"),
    (SourceKind::Note, "Design notes"),
    (SourceKind::Api, "API reference"),
    (SourceKind::Devlog, "Devlog"),
    (SourceKind::Other, "Other"),
];

/// The pinned template content hash (provenance, §6) — the address of the shared template/style.
#[must_use]
pub fn template_hash() -> String {
    use crate::hash::DocHasher;
    let mut h = DocHasher::new();
    h.tag(200).str(theme::READING_CSS);
    h.finish().as_str().to_owned()
}

/// Render the whole model to an HTML site: `index.html` plus one `pages/<anchor>.html` per document.
#[must_use]
pub fn render(model: &DocModel) -> Artifacts {
    let mut arts = Artifacts::new();
    arts.put("index.html", render_index(model));
    for doc in &model.documents {
        arts.put(
            format!("pages/{}.html", doc.anchor),
            render_page(doc, model),
        );
    }
    arts
}

/// The concatenation of every page (for the parity/legibility lints, which scan the rendered output).
#[must_use]
pub fn render_concat(model: &DocModel) -> String {
    let mut s = render_index(model);
    for doc in &model.documents {
        s.push('\n');
        s.push_str(&render_page(doc, model));
    }
    s
}

fn doc_title(doc: &Node) -> &str {
    doc.title.as_deref().unwrap_or(&doc.anchor)
}

fn source_kind(doc: &Node) -> SourceKind {
    match &doc.payload {
        Payload::Document { source_kind } => *source_kind,
        _ => SourceKind::Other,
    }
}

/// The one reviewed page template (§5): head (self-contained theme + no-flash init), a sticky header
/// bar with the theme toggle, the sidebar/main/ToC layout, footer, and the end-of-body scripts.
///
/// `search_base` is `""` on the index and `"../"` on a `pages/` page (the relative prefix the search
/// JS prepends to reach `search-index.jsonl`). `toc` is `None` on the index (no per-page ToC).
fn page_shell(
    title: &str,
    search_base: &str,
    sidebar: &str,
    toc: Option<&str>,
    main: &str,
) -> String {
    let layout_class = if toc.is_some() {
        "layout"
    } else {
        "layout no-toc"
    };
    let toc_html = toc.unwrap_or("");
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n\
         <meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\n\
         <title>{title} — Mycelium</title>\n\
         <style>{css}</style>\n{head_init}\n\
         </head>\n<body>\n\
         {skip}\n\
         <header class=\"site-header\"><div class=\"bar\">\
         <p class=\"site-title\">Mycelium Documentation</p>\
         <p class=\"tagline\">A projection of the cited corpus — never a parallel truth (ADR-003/G11).</p>\
         {toggle}</div></header>\n\
         <div class=\"{layout_class}\">\n{sidebar}\n\
         <main>\n<div id=\"content\" tabindex=\"-1\">\n{main}\n</div>\n</main>\n\
         {toc_html}\n</div>\n\
         <footer>Generated from the Mycelium corpus · one template (hash <code>{th}</code>) · \
         every block is content-addressed (ADR-003). Undocumented items are flagged, never invented (G2).</footer>\n\
         <script>window.MYC_BASE={base_json};</script>\n\
         {toggle_js}\n{search_js}\n\
         </body>\n</html>\n",
        title = inline_text(title),
        css = theme::READING_CSS,
        head_init = theme::HEAD_THEME_INIT,
        skip = theme::SKIP_LINK,
        toggle = theme::THEME_TOGGLE_BUTTON,
        toggle_js = theme::THEME_TOGGLE_JS,
        search_js = theme::CORPUS_SEARCH_JS,
        base_json = json_str(search_base),
        th = html_escape(&short_hash(&template_hash())),
    )
}

/// A minimal JSON string literal (for the inlined `window.MYC_BASE`). Only `"`, `\`, and `<` need
/// escaping here; the base is always a short ASCII path like `""` or `"../"`.
fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '<' => out.push_str("\\u003c"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn short_hash(h: &str) -> String {
    // `blake3:<12 hex>…` — readable provenance without the full 64 hex.
    match h.split_once(':') {
        Some((algo, digest)) => format!("{algo}:{}…", &digest[..digest.len().min(12)]),
        None => h.to_owned(),
    }
}

/// The persistent navigation sidebar: a client-side search box over `search-index.jsonl`, then the
/// corpus grouped by family. `link_prefix` is `"pages/"` on the index and `""` on a page (both reach
/// the sibling `pages/<anchor>.html`). `current` highlights the page being read (`aria-current`).
fn render_sidebar(model: &DocModel, link_prefix: &str, current: Option<&str>) -> String {
    let mut nav = String::from("<nav class=\"sidebar\" aria-label=\"Documentation navigation\">\n");
    nav.push_str(
        "<input id=\"corpus-search-box\" class=\"nav-search\" type=\"search\" \
         placeholder=\"Search the docs…\" aria-label=\"Search the documentation\" autocomplete=\"off\">\n\
         <ul id=\"corpus-search-results\" class=\"search-results\" aria-live=\"polite\"></ul>\n",
    );
    for (kind, label) in GROUPS {
        let mut items = String::new();
        for doc in &model.documents {
            if source_kind(doc) == *kind {
                let current_attr = if current == Some(doc.anchor.as_str()) {
                    " aria-current=\"page\""
                } else {
                    ""
                };
                items.push_str(&format!(
                    "  <li><a href=\"{prefix}{a}.html\"{cur}>{t}</a></li>\n",
                    prefix = link_prefix,
                    a = html_escape(&doc.anchor),
                    cur = current_attr,
                    t = inline_text(doc_title(doc)),
                ));
            }
        }
        if !items.is_empty() {
            nav.push_str(&format!(
                "<p class=\"nav-group\">{}</p>\n<ul>\n{items}</ul>\n",
                html_escape(label)
            ));
        }
    }
    nav.push_str("</nav>");
    nav
}

/// The "on this page" table of contents, built from the document's headings (level-2 sections and
/// their immediate level-3 subsections — deeper levels are omitted to keep the ToC scannable).
/// `None` when the document has no headings (so no empty ToC rail is rendered).
fn render_toc(doc: &Node) -> Option<String> {
    let mut entries: Vec<(u8, &str, &str)> = Vec::new();
    collect_toc(&doc.children, 2, &mut entries);
    if entries.is_empty() {
        return None;
    }
    let mut nav = String::from(
        "<nav class=\"on-this-page\" aria-label=\"On this page\">\
         <p class=\"toc-title\">On this page</p>\n<ul>\n",
    );
    for (depth, anchor, label) in entries {
        nav.push_str(&format!(
            "  <li><a class=\"lvl-{depth}\" href=\"#{a}\">{t}</a></li>\n",
            a = html_escape(anchor),
            t = inline_text(label),
        ));
    }
    nav.push_str("</ul></nav>");
    Some(nav)
}

/// Collect `(depth, anchor, label)` for `Section`/`ApiItem` headings down to depth 3 (inclusive).
fn collect_toc<'a>(nodes: &'a [Node], depth: u8, out: &mut Vec<(u8, &'a str, &'a str)>) {
    if depth > 3 {
        return;
    }
    for node in nodes {
        match &node.payload {
            Payload::Section => {
                out.push((depth, &node.anchor, node.title.as_deref().unwrap_or("")));
                collect_toc(&node.children, depth + 1, out);
            }
            Payload::ApiItem { signature, .. } => {
                out.push((depth, &node.anchor, signature.as_deref().unwrap_or("")));
                collect_toc(&node.children, depth + 1, out);
            }
            _ => {}
        }
    }
}

/// The index→detail entry point (§4.1 #2): documents grouped by corpus family in the sidebar, plus a
/// short welcome. The sidebar's search box is the site-wide search UI (over `search-index.jsonl`).
fn render_index(model: &DocModel) -> String {
    let sidebar = render_sidebar(model, "pages/", None);
    let main = format!(
        "<h1>Mycelium Documentation</h1>\n\
         <p>The living projection of the cited corpus — RFCs, architecture decisions, design notes, \
         specifications, and the projected API reference. Every page is content-addressed \
         (ADR-003/G11); nothing here is a parallel truth.</p>\n\
         <h2>Browse the corpus</h2>\n\
         <p>Use the navigation sidebar (grouped by family) or the search box to find a document. \
         Each page offers graded depth (minimal \u{00b7} medium \u{00b7} detailed — RFC-0013 levels \
         reused) and an \u{201c}on this page\u{201d} outline.</p>\n\
         <p>{count} documents, {nodes} content-addressed blocks.</p>",
        count = model.documents.len(),
        nodes = model.all_nodes().len(),
    );
    page_shell("Index", "", &sidebar, None, &main)
}

fn render_page(doc: &Node, model: &DocModel) -> String {
    let mut main = String::new();
    main.push_str(&format!(
        "<article id=\"{id}\" data-cid=\"{cid}\"><h1>{t}</h1>\n",
        id = html_escape(&doc.anchor),
        cid = html_escape(doc.id.as_str()),
        t = html_escape(doc_title(doc)),
    ));
    for child in &doc.children {
        render_node(child, 2, &doc.anchor, &mut main);
    }
    main.push_str("</article>");
    let sidebar = render_sidebar(model, "", Some(&doc.anchor));
    let toc = render_toc(doc);
    page_shell(doc_title(doc), "../", &sidebar, toc.as_deref(), &main)
}

/// Render one node at heading `depth` (2..=6, clamped — heading order never skips, §4.1 #8).
/// `doc_anchor` is the enclosing document's anchor (so a cross-document xref gets the right page href).
fn render_node(node: &Node, depth: usize, doc_anchor: &str, buf: &mut String) {
    let cid = html_escape(node.id.as_str());
    match &node.payload {
        Payload::Section => {
            let h = depth.clamp(2, 6);
            let lvl = node
                .level
                .map(|l| format!(" <span class=\"level\">{}</span>", l.as_str()))
                .unwrap_or_default();
            buf.push_str(&format!(
                "<section data-cid=\"{cid}\" id=\"{id}\">\n<h{h}>{t}{lvl}</h{h}>\n",
                id = html_escape(&node.anchor),
                t = inline_html(node.title.as_deref().unwrap_or("")),
            ));
            for c in &node.children {
                render_node(c, depth + 1, doc_anchor, buf);
            }
            buf.push_str("</section>\n");
        }
        Payload::Prose { text } => {
            // A GitHub-style pipe table projects (as prose) into one newline-joined block — render it
            // as a real <table> (presentation only; the node id is unchanged, so dual-projection
            // parity and content-addressing are untouched — the JSON/Typst views render the same
            // node as verbatim text). Everything else stays a paragraph.
            if let Some(table) = render_table(text, &cid) {
                buf.push_str(&table);
            } else {
                buf.push_str(&format!(
                    "<p data-cid=\"{cid}\">{}</p>\n",
                    inline_html(text)
                ));
            }
        }
        Payload::Example {
            lang,
            source,
            checked,
        } => {
            let badge = if *checked {
                " <span class=\"checked\" title=\"type-checked in CI\">✓ checked</span>"
            } else {
                " <span class=\"level\" title=\"illustrative, not CI-checked\">illustrative</span>"
            };
            // Wire the trusted L1 lexer for `myc`/`myc-checked` fences (never-silent: any failure —
            // a lexer error, a non-myc language, a non-ASCII span — falls back to the plain escaped
            // source; highlighting is a lexical Empirical/Declared heuristic, never fabricated).
            let inner = highlight::highlight(lang, source).unwrap_or_else(|| html_escape(source));
            buf.push_str(&format!(
                "<figure data-cid=\"{cid}\">{badge}\n<pre><code class=\"language-{lang}\">{inner}</code></pre>\n</figure>\n",
                lang = html_escape(lang),
            ));
        }
        Payload::Xref { target } => {
            let (href, class) = match &target.resolution {
                XrefResolution::Internal { anchor } => {
                    // Same page → a bare fragment; cross-page → the sibling page + fragment.
                    let target_doc = anchor.split("--").next().unwrap_or(anchor);
                    let href = if target_doc == doc_anchor {
                        format!("#{}", html_escape(anchor))
                    } else {
                        format!("{}.html#{}", html_escape(target_doc), html_escape(anchor))
                    };
                    (href, "")
                }
                XrefResolution::ExternalUrl | XrefResolution::OutOfScope => {
                    (html_escape(&target.raw), "")
                }
                XrefResolution::Dead { .. } | XrefResolution::Unresolved => {
                    (html_escape(&target.raw), " class=\"unresolved\"")
                }
            };
            buf.push_str(&format!(
                "<a data-cid=\"{cid}\" href=\"{href}\"{class}>{t}</a>\n",
                t = html_escape(&target.raw),
            ));
        }
        Payload::ApiItem { signature, summary } => {
            let h = depth.clamp(2, 6);
            buf.push_str(&format!(
                "<section data-cid=\"{cid}\" id=\"{id}\">\n<h{h}><code>{sig}</code></h{h}>\n",
                id = html_escape(&node.anchor),
                sig = html_escape(signature.as_deref().unwrap_or("")),
            ));
            match summary {
                Some(s) => buf.push_str(&format!("<p>{}</p>\n", html_escape(s))),
                None => buf.push_str(
                    "<p class=\"undocumented\">undocumented — no summary projected from source (G2)</p>\n",
                ),
            }
            for c in &node.children {
                render_node(c, depth + 1, doc_anchor, buf);
            }
            buf.push_str("</section>\n");
        }
        Payload::Undocumented { what } => {
            buf.push_str(&format!(
                "<p data-cid=\"{cid}\" class=\"undocumented\">undocumented: {}</p>\n",
                html_escape(what),
            ));
        }
        Payload::Document { .. } | Payload::Index => {
            // Nested documents/index are not expected inside a page body; render children flatly.
            for c in &node.children {
                render_node(c, depth, doc_anchor, buf);
            }
        }
    }
}

/// Render a GitHub-style pipe table (`| a | b |` with a `|---|---|` separator row) as a scroll-safe
/// `<table>`, keeping the node's `data-cid`. Returns `None` when `text` is not a well-formed table
/// (then the caller renders a paragraph) — a robust check: the separator row must have the same cell
/// count as the header, so ordinary prose with a stray dash is never mistaken for a table.
///
/// `pub(crate)` (with [`split_row`]/[`is_separator_cell`]) for white-box unit testing in
/// `src/tests/html.rs`, not a downstream API.
pub(crate) fn render_table(text: &str, cid: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() < 2 {
        return None;
    }
    // The header must be genuinely pipe-delimited (GFM tables always carry `|`) — this rejects a
    // setext-style `Heading\n---` line, which is a header+dashes but not a table.
    if !lines[0].contains('|') {
        return None;
    }
    let header = split_row(lines[0]);
    if header.is_empty() {
        return None;
    }
    let sep = split_row(lines[1]);
    if sep.len() != header.len() || !sep.iter().all(|c| is_separator_cell(c)) {
        return None;
    }
    let mut out = format!("<div class=\"table-wrap\"><table data-cid=\"{cid}\">\n<thead><tr>");
    for cell in &header {
        out.push_str(&format!("<th>{}</th>", inline_html(cell)));
    }
    out.push_str("</tr></thead>\n<tbody>\n");
    for line in &lines[2..] {
        if line.trim().is_empty() {
            continue;
        }
        out.push_str("<tr>");
        for cell in split_row(line) {
            out.push_str(&format!("<td>{}</td>", inline_html(&cell)));
        }
        out.push_str("</tr>\n");
    }
    out.push_str("</tbody></table></div>\n");
    Some(out)
}

/// Split a pipe-table row into trimmed cells (a single optional leading/trailing `|` is stripped).
pub(crate) fn split_row(line: &str) -> Vec<String> {
    let t = line.trim();
    let t = t.strip_prefix('|').unwrap_or(t);
    let t = t.strip_suffix('|').unwrap_or(t);
    t.split('|').map(|c| c.trim().to_owned()).collect()
}

/// Whether a separator cell is well-formed (`-`, optionally with alignment colons: `:--`, `--:`, `:-:`).
pub(crate) fn is_separator_cell(cell: &str) -> bool {
    let c = cell.trim();
    !c.is_empty() && c.contains('-') && c.chars().all(|ch| ch == '-' || ch == ':')
}
