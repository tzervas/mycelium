//! Build orchestration: walk the corpus + schemas + example project roots, project each into the
//! doc-IR, **resolve cross-references against the assembled model**, and hand back a finalized
//! [`DocModel`]. This is the drift-proof projection toolchain's entry point — everything downstream
//! (emit, §4.1 lint) is a pure function of the model it returns.
//!
//! Honest scope (spec §8): HTML + Typst(→PDF) + machine JSON are built; **EPUB is a deferral** — the
//! Typst path can fan out to EPUB later, but shipping a half-EPUB would violate "never a half-build"
//! (§4.1), so v0 emits a deferral note instead of a broken artifact.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::corpus::{ingest, AnchorAlloc};
use crate::ir::{DocModel, Node, Payload, SourceKind, XrefResolution, XrefTarget};
use crate::{apiref, emit};

/// EPUB is an honest deferral (spec §8 / §4.1 "never a half-build"). The build records this rather
/// than emitting a broken e-book.
pub const EPUB_DEFERRAL: &str =
    "EPUB is deferred (spec §8.5): the Typst path is the PDF fan-out for v0; an EPUB renderer is a \
     later, separate artifact — emitting a half-EPUB would violate the §4.1 'never a half-build' bar.";

/// What to ingest.
#[derive(Debug, Clone)]
pub struct BuildInput {
    /// The repo root (all source paths are recorded repo-relative to it — stable provenance).
    pub repo_root: PathBuf,
    /// The markdown corpus root (e.g. `docs`), if any.
    pub corpus_root: Option<PathBuf>,
    /// The JSON-schema root (e.g. `docs/spec/schemas`), if any.
    pub schemas_root: Option<PathBuf>,
    /// Example/project roots to project `.myc` nodules from (e.g. `examples`).
    pub example_roots: Vec<PathBuf>,
}

impl BuildInput {
    /// The conventional layout rooted at `repo_root`: `docs/`, `docs/spec/schemas/`, `examples/`,
    /// and `lib/std/` (the self-hosted standard library — M-736). Every `.myc` nodule under
    /// `lib/std/` is projected into the API reference (per-module `fn` signatures + `@summary`),
    /// so the generated stdlib API docs grow as the E13-1 self-hosting migration lands modules.
    /// Today only `lib/std/result.myc` self-hosts; the remaining stdlib modules are Rust-first and
    /// appear here as they are ported (the gap is honest, not silent — G2).
    #[must_use]
    pub fn conventional(repo_root: impl Into<PathBuf>) -> Self {
        let repo_root = repo_root.into();
        BuildInput {
            corpus_root: Some(repo_root.join("docs")),
            schemas_root: Some(repo_root.join("docs/spec/schemas")),
            example_roots: vec![repo_root.join("examples"), repo_root.join("lib/std")],
            repo_root,
        }
    }
}

/// Build the resolved doc model from the input.
///
/// # Errors
/// Propagates the first filesystem error (with its path) — never a silent skip of a present source.
pub fn build(input: &BuildInput) -> std::io::Result<DocModel> {
    let mut alloc = AnchorAlloc::new();
    let mut docs: Vec<Node> = Vec::new();
    // file_index: repo-relative normalized path → document anchor (for xref resolution).
    let mut file_index: BTreeMap<String, String> = BTreeMap::new();

    // 1) Markdown corpus.
    if let Some(root) = &input.corpus_root {
        let mut md = collect_files(root, "md")?;
        md.sort();
        for path in md {
            if is_excluded(&path) {
                continue;
            }
            let rel = repo_rel(&input.repo_root, &path);
            let src = std::fs::read_to_string(&path)?;
            let kind = classify(&rel);
            let node = ingest(&rel, &src, kind, &mut alloc);
            file_index.insert(rel.clone(), node.anchor.clone());
            docs.push(node);
        }
    }

    // 2) JSON schemas (api reference).
    if let Some(root) = &input.schemas_root {
        let mut schemas = collect_files(root, "json")?;
        schemas.sort();
        for path in schemas {
            if is_excluded(&path) || !ends_with(&path, ".schema.json") {
                continue;
            }
            let rel = repo_rel(&input.repo_root, &path);
            let src = std::fs::read_to_string(&path)?;
            if let Some(node) = apiref::project_schema(&rel, &src, &mut alloc) {
                file_index.insert(rel.clone(), node.anchor.clone());
                docs.push(node);
            }
        }
    }

    // 3) Example/project `.myc` nodules (api reference + checked examples).
    for root in &input.example_roots {
        if !root.exists() {
            continue;
        }
        let mut myc = collect_files(root, "myc")?;
        myc.sort();
        for path in myc {
            if is_excluded(&path) {
                continue;
            }
            let rel = repo_rel(&input.repo_root, &path);
            let src = std::fs::read_to_string(&path)?;
            let node = apiref::project_nodule(&rel, &src, &mut alloc);
            file_index.insert(rel.clone(), node.anchor.clone());
            docs.push(node);
        }
    }

    // Preliminary model → the anchor universe the resolver checks against.
    let prelim = DocModel::new(docs);
    let anchors: std::collections::BTreeSet<String> = prelim.anchors.keys().cloned().collect();
    let corpus_rel = input
        .corpus_root
        .as_ref()
        .map(|r| repo_rel(&input.repo_root, r));

    let ctx = ResolveCtx {
        anchors,
        file_index,
        corpus_rel,
    };
    let resolved: Vec<Node> = prelim
        .documents
        .iter()
        .map(|d| resolve_node(d, &ctx))
        .collect();
    Ok(DocModel::new(resolved))
}

/// Emit every artifact (HTML site · Typst source · machine JSON · the EPUB deferral note).
#[must_use]
pub fn emit_all(model: &DocModel) -> emit::Artifacts {
    let mut arts = emit::Artifacts::new();
    for (k, v) in emit::html::render(model).files {
        arts.put(k, v);
    }
    for (k, v) in emit::json::render(model).files {
        arts.put(k, v);
    }
    arts.put("doc.typ", emit::typst::render(model));
    arts.put("EPUB-DEFERRED.txt", EPUB_DEFERRAL);
    arts
}

// ── xref resolution ─────────────────────────────────────────────────────────────────────────────

struct ResolveCtx {
    anchors: std::collections::BTreeSet<String>,
    file_index: BTreeMap<String, String>,
    /// The corpus root, repo-relative (e.g. `docs`) — internal links under it must resolve.
    corpus_rel: Option<String>,
}

/// Rebuild a node with its cross-references resolved (hashes repropagate from the leaves up).
fn resolve_node(node: &Node, ctx: &ResolveCtx) -> Node {
    let children: Vec<Node> = node.children.iter().map(|c| resolve_node(c, ctx)).collect();
    let payload = match &node.payload {
        Payload::Xref { target } => Payload::Xref {
            target: resolve_target(&target.raw, &node.anchor, &node.provenance.source, ctx),
        },
        other => other.clone(),
    };
    Node::new(
        node.anchor.clone(),
        node.title.clone(),
        node.level,
        node.provenance.clone(),
        payload,
        children,
    )
}

/// The document anchor a sub-anchor belongs to (the prefix before the first `--`).
fn doc_of(anchor: &str) -> &str {
    anchor.split("--").next().unwrap_or(anchor)
}

fn resolve_target(raw: &str, here_anchor: &str, source: &str, ctx: &ResolveCtx) -> XrefTarget {
    let res = classify_target(raw, here_anchor, source, ctx);
    XrefTarget {
        raw: raw.to_owned(),
        resolution: res,
    }
}

fn classify_target(raw: &str, here_anchor: &str, source: &str, ctx: &ResolveCtx) -> XrefResolution {
    if raw.starts_with("http://") || raw.starts_with("https://") {
        return XrefResolution::ExternalUrl;
    }
    if raw.starts_with("mailto:") || raw.contains("://") {
        return XrefResolution::OutOfScope;
    }
    let (path_part, frag) = match raw.split_once('#') {
        Some((p, f)) => (p, Some(f)),
        None => (raw, None),
    };

    // Pure same-document fragment.
    if path_part.is_empty() {
        let here_doc = doc_of(here_anchor);
        return resolve_fragment(here_doc, frag, ctx);
    }

    // A relative link to another file.
    let normalized = normalize_join(parent_dir(source), path_part);
    if let Some(doc_anchor) = ctx.file_index.get(&normalized) {
        return resolve_fragment(doc_anchor, frag, ctx);
    }
    // Not ingested: is it an *internal* corpus doc we should have (dead) or genuinely external?
    let is_md = ends_with_str(&normalized, ".md") || ends_with_str(&normalized, ".markdown");
    if is_md {
        if let Some(corpus) = &ctx.corpus_rel {
            if normalized.starts_with(&format!("{corpus}/")) {
                return XrefResolution::Dead {
                    reason: format!(
                        "internal corpus link does not resolve to an ingested document: {raw}"
                    ),
                };
            }
        }
    }
    // A link outside the generated site (README, research/, tooling, an image, …) — links.sh owns it.
    XrefResolution::OutOfScope
}

/// Resolve a fragment against a target document: the section anchor if it exists, else the document
/// top (fragment-level anchoring is best-effort in v0; whole-document xrefs are enforced).
fn resolve_fragment(doc_anchor: &str, frag: Option<&str>, ctx: &ResolveCtx) -> XrefResolution {
    match frag {
        None => XrefResolution::Internal {
            anchor: doc_anchor.to_owned(),
        },
        Some(f) => {
            let candidate = format!("{doc_anchor}--{}", crate::corpus::slugify(f));
            if ctx.anchors.contains(&candidate) {
                XrefResolution::Internal { anchor: candidate }
            } else {
                XrefResolution::Internal {
                    anchor: doc_anchor.to_owned(),
                }
            }
        }
    }
}

// ── filesystem helpers ──────────────────────────────────────────────────────────────────────────

fn classify(rel: &str) -> SourceKind {
    if rel.contains("/rfcs/") {
        SourceKind::Rfc
    } else if rel.contains("/adr/") {
        SourceKind::Adr
    } else if rel.contains("/notes/") {
        SourceKind::Note
    } else if rel.contains("/devlog/") {
        SourceKind::Devlog
    } else if rel.contains("/spec/") {
        SourceKind::Spec
    } else {
        SourceKind::Other
    }
}

/// Intentionally-bad must-fail fixtures and the reject corpus are out of scope (gate discipline,
/// Wave-A locked decision #3) — projecting them would wrongly redden the build.
fn is_excluded(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.contains("/tests/fixtures/")
        || s.contains("/fixtures/")
        || s.contains("/reject/")
        || s.contains("/target/")
}

fn ends_with(path: &Path, suffix: &str) -> bool {
    path.to_string_lossy().ends_with(suffix)
}

fn ends_with_str(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

/// Recursively collect files with the given extension under `root`.
fn collect_files(root: &Path, ext: &str) -> std::io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
                out.push(path);
            }
        }
    }
    Ok(out)
}

/// `path` made repo-relative (with `/` separators), or its lossy form if not under `repo_root`.
fn repo_rel(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

/// The parent directory of a repo-relative path (`docs/rfcs/x.md` → `docs/rfcs`).
fn parent_dir(rel: &str) -> &str {
    rel.rsplit_once('/').map_or("", |(d, _)| d)
}

/// Join `base` (a dir) and a relative `link`, resolving `.`/`..`, returning a clean repo-relative path.
fn normalize_join(base: &str, link: &str) -> String {
    let mut parts: Vec<&str> = if base.is_empty() {
        Vec::new()
    } else {
        base.split('/').collect()
    };
    for seg in link.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_join_resolves_dot_dot() {
        assert_eq!(
            normalize_join("docs/rfcs", "../adr/ADR-003.md"),
            "docs/adr/ADR-003.md"
        );
        assert_eq!(normalize_join("docs", "./Glossary.md"), "docs/Glossary.md");
        assert_eq!(normalize_join("docs/spec", "x.md"), "docs/spec/x.md");
    }

    #[test]
    fn classify_maps_paths_to_families() {
        assert_eq!(classify("docs/rfcs/RFC-0001.md"), SourceKind::Rfc);
        assert_eq!(classify("docs/adr/ADR-010.md"), SourceKind::Adr);
        assert_eq!(classify("docs/notes/DN-06.md"), SourceKind::Note);
        assert_eq!(classify("docs/devlog/x.md"), SourceKind::Devlog);
        assert_eq!(classify("docs/spec/SPEC.md"), SourceKind::Spec);
        assert_eq!(classify("docs/Glossary.md"), SourceKind::Other);
    }

    fn ctx_with(files: &[(&str, &str)], anchors: &[&str], corpus: &str) -> ResolveCtx {
        ResolveCtx {
            anchors: anchors.iter().map(|s| (*s).to_owned()).collect(),
            file_index: files
                .iter()
                .map(|(p, a)| ((*p).to_owned(), (*a).to_owned()))
                .collect(),
            corpus_rel: Some(corpus.to_owned()),
        }
    }

    #[test]
    fn an_external_url_is_out_of_scope_not_dead() {
        let ctx = ctx_with(&[], &[], "docs");
        assert_eq!(
            classify_target("https://example.com", "d--x", "docs/a.md", &ctx),
            XrefResolution::ExternalUrl
        );
    }

    #[test]
    fn a_resolving_internal_md_link_is_internal() {
        let ctx = ctx_with(
            &[("docs/rfcs/RFC-0013.md", "rfc-0013")],
            &["rfc-0013", "rfc-0013--levels"],
            "docs",
        );
        // file-level
        assert_eq!(
            classify_target("../rfcs/RFC-0013.md", "spec--x", "docs/spec/a.md", &ctx),
            XrefResolution::Internal {
                anchor: "rfc-0013".to_owned()
            }
        );
        // fragment-level
        assert_eq!(
            classify_target(
                "../rfcs/RFC-0013.md#levels",
                "spec--x",
                "docs/spec/a.md",
                &ctx
            ),
            XrefResolution::Internal {
                anchor: "rfc-0013--levels".to_owned()
            }
        );
    }

    #[test]
    fn a_broken_internal_corpus_link_is_dead() {
        let ctx = ctx_with(&[], &[], "docs");
        match classify_target("../rfcs/RFC-9999.md", "spec--x", "docs/spec/a.md", &ctx) {
            XrefResolution::Dead { .. } => {}
            other => panic!("expected Dead, got {other:?}"),
        }
    }

    #[test]
    fn a_link_outside_the_corpus_is_out_of_scope() {
        let ctx = ctx_with(&[], &[], "docs");
        // README at repo root — links.sh owns it, not the doc-IR.
        assert_eq!(
            classify_target("../../README.md", "spec--x", "docs/spec/a.md", &ctx),
            XrefResolution::OutOfScope
        );
        // a non-markdown target
        assert_eq!(
            classify_target("../../scripts/lib.sh", "spec--x", "docs/spec/a.md", &ctx),
            XrefResolution::OutOfScope
        );
    }

    #[test]
    fn a_missing_fragment_falls_back_to_the_document_top() {
        let ctx = ctx_with(
            &[("docs/x.md", "x")],
            &["x"], // no x--nope anchor
            "docs",
        );
        assert_eq!(
            classify_target("x.md#nope", "y--a", "docs/y.md", &ctx),
            XrefResolution::Internal {
                anchor: "x".to_owned()
            }
        );
    }
}
