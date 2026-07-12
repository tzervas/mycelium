//! The gap-close-2 **cross-nodule symbol table** (the `Import` gap-class lever — Phase-0 re-measure,
//! `docs/notes/DN-34-Rust-to-Mycelium-Transpiler-Strategy.md` §8.19/§8.20): a **batch-scoped** map
//! from a Rust crate-root-relative module path to the Mycelium nodule this transpiler derived for
//! it, plus the set of item names that sibling's own pass actually **emitted** — never a name that
//! merely exists in the Rust source but itself gapped. See `batch.rs::transpile_batch`'s two-pass
//! driver for how this is built and installed, and `transpile::dispatch_use` for the consumer.
//!
//! **Scope (deliberately narrow — VR-5/G2, "flag, don't guess"):** this resolves ONLY a `use`
//! whose head is `crate::<mod>...` (the crate-root-absolute form) or, when the head segment itself
//! names a batch sibling module (the crate-root file's own bare `use <mod>::Item;`/`pub use
//! <mod>::Item;` form — real Rust name resolution: a bare first segment resolves in the *current
//! file's own scope* before the extern prelude, so this can only ever match a genuine sibling, never
//! misfire on an external crate coincidentally sharing a name — see [`use_candidates`]'s doc). A
//! `self::`/`super::`-headed path is **not** attempted (relative-to-current-module resolution is out
//! of this increment's scope) and falls through to the unchanged, pre-existing unresolved-import gap.
//! Every miss — an out-of-batch head, an in-batch sibling that itself gapped the requested name, a
//! `self`-module-binding leaf, a rename, or a cross-nodule glob — is still recorded as a
//! [`crate::gap::Category::Import`] [`crate::gap::GapReason`] naming exactly what could not resolve
//! and why (never silently dropped).
//!
//! **No bare-name collapse (the M-1060 lesson):** a resolved leaf is always emitted against the
//! sibling's *derived, home-qualified* nodule path (`use <nodule_path>.<Item>;`), never a bare
//! `<Item>` — the identical discipline `crates/mycelium-l1/src/checkty.rs`'s DN-113
//! `qualify_cross_phylum`/`merge_phyla_exports` use for the kernel's own cross-phylum resolution.

use std::collections::{HashMap, HashSet};
use syn::UseTree;

/// The kind of one flattened `use`-tree leaf (see [`use_candidates`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CandidateKind {
    /// A plain imported name (`Item` in `use crate::mod::Item;`, or one member of a
    /// `use crate::mod::{A, B};` group) — the only kind [`SymbolTable::resolve`] can ever resolve.
    Name(String),
    /// `self` inside a group (`use crate::mod::{self, Item}`) — binds the **module itself** as a
    /// local name, not an item. There is no "import a nodule as a name" construct in this grammar,
    /// so this leaf is unresolvable by construction (distinct from a plain lookup miss).
    SelfModule,
    /// `use crate::mod::Item as Alias;` — a rename. Scoped OUT of this increment: emitting a
    /// renamed cross-nodule reference would need the alias threaded through every downstream
    /// reference to `Alias` in this file's body, which this transpiler does not do — flagged, not
    /// guessed (VR-5/G2).
    Rename { from: String, to: String },
    /// `use crate::mod::*;` — a cross-nodule glob. Scoped OUT, mirroring DN-113 v1's own deferral of
    /// a cross-phylum glob to M-982 rather than guessing a disambiguation.
    Glob,
}

/// One flattened leaf of a (possibly grouped/nested) `use` tree, with the crate-root-relative Rust
/// module path it was found under (e.g. `["checkty"]`, `["foo", "bar"]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UseCandidate {
    pub module_segs: Vec<String>,
    pub kind: CandidateKind,
}

/// Extract every candidate leaf from a `use` item's tree, or `None` when the head is `self`/`super`
/// (relative-to-current-module forms this increment does not attempt — see module docs) or when the
/// tree has no module-path segment at all (a bare `use Item;` naming nothing to resolve against).
///
/// A `crate::...` head is peeled — the rest becomes the crate-root-relative module path. Any other
/// head is tried **as-is** as a crate-root-relative candidate too (the bare-sibling-module form, e.g.
/// `use error::FsErr;` written directly in a crate's `lib.rs`, whose own `mod error;` puts `error` in
/// `lib.rs`'s local scope without needing `crate::`). This never mis-fires on a genuine external
/// crate (`use serde::Serialize;`): [`SymbolTable::resolve`] only ever returns a hit when the head
/// concretely matches a batch sibling's own module key, so an unrelated crate sharing a name is
/// simply never a key in the table (a miss, not a guess — VR-5).
pub(crate) fn use_candidates(tree: &UseTree) -> Option<Vec<UseCandidate>> {
    let UseTree::Path(p) = tree else {
        return None;
    };
    let head = p.ident.to_string();
    if head == "self" || head == "super" {
        return None;
    }
    let mut prefix = if head == "crate" {
        Vec::new()
    } else {
        vec![head]
    };
    let mut out = Vec::new();
    flatten(&p.tree, &mut prefix, &mut out);
    Some(out)
}

fn flatten(tree: &UseTree, prefix: &mut Vec<String>, out: &mut Vec<UseCandidate>) {
    match tree {
        UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            flatten(&p.tree, prefix, out);
            prefix.pop();
        }
        UseTree::Name(n) => {
            let kind = if n.ident == "self" {
                CandidateKind::SelfModule
            } else {
                CandidateKind::Name(n.ident.to_string())
            };
            out.push(UseCandidate {
                module_segs: prefix.clone(),
                kind,
            });
        }
        UseTree::Rename(r) => out.push(UseCandidate {
            module_segs: prefix.clone(),
            kind: CandidateKind::Rename {
                from: r.ident.to_string(),
                to: r.rename.to_string(),
            },
        }),
        UseTree::Glob(_) => out.push(UseCandidate {
            module_segs: prefix.clone(),
            kind: CandidateKind::Glob,
        }),
        UseTree::Group(g) => {
            for t in &g.items {
                flatten(t, prefix, out);
            }
        }
    }
}

/// One sibling nodule's cross-nodule-visible surface, as seen by this batch's own baseline pass.
#[derive(Debug, Clone)]
pub(crate) struct NoduleSymbols {
    /// The Mycelium nodule path this file transpiles to (`transpile::derive_nodule_path`'s output)
    /// — the qualifier a resolved `use` is emitted against (`use <nodule_path>.<Item>;`).
    pub nodule_path: String,
    /// Every top-level item name this batch's baseline pass actually **emitted** for this file
    /// (`GapReport::emitted_items`) — the only names a cross-nodule `use` may ever resolve to.
    pub emitted: HashSet<String>,
}

/// The batch-wide cross-nodule symbol table: Rust crate-root-relative module path (dot-joined, e.g.
/// `"checkty"`, `"foo.bar"`) -> that sibling's [`NoduleSymbols`]. Built once per batch (see
/// `batch.rs::build_symbol_table`) from every file's baseline-pass [`crate::gap::GapReport`].
#[derive(Debug, Clone, Default)]
pub(crate) struct SymbolTable {
    modules: HashMap<String, NoduleSymbols>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, module_key: String, nodule_path: String, emitted: HashSet<String>) {
        self.modules.insert(
            module_key,
            NoduleSymbols {
                nodule_path,
                emitted,
            },
        );
    }

    /// Resolve `name` in `module_key` (dot-joined Rust module-path segments). Returns the
    /// **home-qualified** Mycelium nodule path to emit against — never the bare `module_key`, never
    /// a guessed rename (VR-5; mirrors DN-113 `qualify_cross_phylum`'s never-bare discipline).
    pub fn resolve(&self, module_key: &str, name: &str) -> Option<&str> {
        self.modules
            .get(module_key)
            .filter(|m| m.emitted.contains(name))
            .map(|m| m.nodule_path.as_str())
    }

    /// Is `module_key` a batch sibling at all (regardless of whether a particular name resolves in
    /// it)? Used to word an honest, distinct reason for "this head isn't even a batch sibling" vs
    /// "it is a sibling, but it gapped this particular name".
    pub fn has_module(&self, module_key: &str) -> bool {
        self.modules.contains_key(module_key)
    }

    /// Every `(module_key, name)` pair this table can resolve, for a candidate leaf's module path
    /// segments joined with `.`. A thin convenience over [`Self::resolve`] used by
    /// `batch.rs`'s pub-needed scan.
    pub fn module_key(module_segs: &[String]) -> String {
        module_segs.join(".")
    }
}
