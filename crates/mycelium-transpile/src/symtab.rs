//! The gap-close-2 **cross-nodule symbol table** (the `Import` gap-class lever — Phase-0 re-measure,
//! `docs/notes/DN-34-Rust-to-Mycelium-Transpiler-Strategy.md` §8.19/§8.20): a **batch-scoped** map
//! from a Rust crate-root-relative module path to the Mycelium nodule this transpiler derived for
//! it, plus the set of item names that sibling's own pass actually **emitted** — never a name that
//! merely exists in the Rust source but itself gapped. See `batch.rs::transpile_batch`'s two-pass
//! driver for how this is built and installed, and `transpile::dispatch_use` for the consumer.
//!
//! **M-1084 (Import net-close): `self::`/`super::` relative resolution + cross-PHYLUM resolution.**
//! The original gap-close-2 lever resolved only `crate::<mod>...` (crate-root-absolute) and the
//! bare crate-root form (`use <mod>::Item;` — real Rust "uniform paths": an unqualified `use` head
//! is *always* tried crate-root-relative first, in every file, not just the crate-root one). This
//! increment closes the two residuals that lever's own doc named as scoped out:
//! 1. **`self::`/`super::`** now resolve relative to the CURRENT file's own module path
//!    ([`use_candidates`]'s `current_module` parameter) — [`HeadKind::SameCrate`], always looked up
//!    in the current file's own crate, never tried as an extern-crate name. Multi-level `super::`
//!    chains (`super::super::X`, two directories up) are peeled one level at a time. A `super::`
//!    chain that would walk past the crate root is a genuine structural miss — real Rust itself
//!    rejects this — never a guess (VR-5/G2).
//! 2. **Cross-phylum**: a *bare* head (neither `crate`/`self`/`super`) is ambiguous in real Rust
//!    between "this crate's own crate-root submodule" and "an extern crate's own name" —
//!    [`HeadKind::Bare`] — and [`SymbolTable::candidate_lookup_keys`] tries both, in Rust's own
//!    precedence order (same-crate first, then the head read literally as the named sibling
//!    PHYLUM's own extern-crate identifier — `transpile::derive_crate_ident`, the standard Cargo
//!    package-name -> crate-identifier mapping). A hit requires the exact crate identifier AND the
//!    exact emitted name to both be real entries in this batch's table (never a guess — VR-5); this
//!    only ever fires when the referenced phylum's own files are *in the same batch* (e.g. a
//!    multi-crate `--files` invocation) — a phylum transpiled in a wholly separate run is, honestly,
//!    still an out-of-batch miss (G2). This mirrors `crates/mycelium-l1/src/checkty.rs`'s DN-113
//!    `merge_phyla_exports`: "one added qualifier dimension" lets the SAME resolver handle both the
//!    intra-crate and cross-phylum case, no second resolver (DRY; DN-113 §9.6).
//!
//! Still scoped OUT (unchanged): a rename (the alias would need threading through every downstream
//! reference in the file body — out of this transpiler's rewrite surface) and a glob (no
//! disambiguation strategy yet — mirrors DN-113 v1's own deferral to M-982). Every miss — an
//! out-of-batch head, an in-batch sibling that itself gapped the requested name, a `self`-module-
//! binding leaf, a rename, a glob, or a `super::` with no parent to go up to — is still recorded as
//! a [`crate::gap::Category::Import`] [`crate::gap::GapReason`] naming exactly what could not
//! resolve and why (never silently dropped).
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

/// Which resolution namespace a candidate leaf's HEAD names (M-1084) — see the module docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeadKind {
    /// `crate::`/`self::`/`super::` — unambiguously relative to the CURRENT file's own crate; never
    /// tried as an extern-crate (cross-phylum) name.
    SameCrate,
    /// A literal head that is neither `crate`/`self`/`super` — ambiguous in real Rust between "this
    /// crate's own crate-root submodule" (tried FIRST, Rust's own precedence: a local crate-root
    /// item shadows a same-named extern crate) and "an extern crate's own name" (the cross-phylum
    /// interpretation, tried only on a same-crate miss).
    Bare,
}

/// One flattened leaf of a (possibly grouped/nested) `use` tree, with the crate-root-relative Rust
/// module path it was found under (e.g. `["checkty"]`, `["foo", "bar"]`) and the [`HeadKind`] that
/// determines how [`SymbolTable::candidate_lookup_keys`] resolves it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UseCandidate {
    pub module_segs: Vec<String>,
    pub kind: CandidateKind,
    pub head_kind: HeadKind,
}

/// Extract every candidate leaf from a `use` item's tree, given the CURRENT file's own
/// crate-root-relative module path (`current_module` — e.g. `[]` for a crate-root `lib.rs`/`mod.rs`,
/// `["foo", "bar"]` for `foo/bar.rs`; see `transpile::derive_module_segments`). Returns `None` when
/// the tree has no module-path segment at all (a bare `use Item;` naming nothing to resolve
/// against), or when the head is `super` and `current_module` is already empty (no parent to go up
/// to at the crate root — a genuine structural miss, not attempted; VR-5/G2).
///
/// - A `crate::...` head is peeled — the rest becomes the crate-root-relative module path
///   ([`HeadKind::SameCrate`]).
/// - A `self::...` head resolves relative to `current_module` itself ([`HeadKind::SameCrate`]; M-1084).
/// - A `super::...` head resolves relative to `current_module`'s PARENT ([`HeadKind::SameCrate`];
///   M-1084).
/// - Any other head is tried **as-is** as a crate-root-relative candidate too (the bare-sibling-
///   module form, e.g. `use error::FsErr;` — real Rust "uniform paths": an unqualified `use` head is
///   *always* crate-root-relative-or-extern-prelude, in every file, not just the crate-root one) —
///   [`HeadKind::Bare`]. This never mis-fires on a genuine external crate (`use serde::Serialize;`):
///   [`SymbolTable::resolve`] only ever returns a hit when the head concretely matches a batch
///   sibling's own module key (same-crate) or a batch sibling PHYLUM's own extern-crate identifier
///   (cross-phylum), so an unrelated crate sharing neither is simply never a key in the table (a
///   miss, not a guess — VR-5).
pub(crate) fn use_candidates(
    tree: &UseTree,
    current_module: &[String],
) -> Option<Vec<UseCandidate>> {
    let UseTree::Path(p) = tree else {
        return None;
    };
    let head = p.ident.to_string();
    let (mut prefix, head_kind, mut rest): (Vec<String>, HeadKind, &UseTree) = match head.as_str() {
        "crate" => (Vec::new(), HeadKind::SameCrate, &p.tree),
        "self" => (current_module.to_vec(), HeadKind::SameCrate, &p.tree),
        "super" => {
            if current_module.is_empty() {
                return None;
            }
            (
                current_module[..current_module.len() - 1].to_vec(),
                HeadKind::SameCrate,
                &p.tree,
            )
        }
        _ => (vec![head], HeadKind::Bare, &p.tree),
    };
    // Peel any further CONSECUTIVE leading `super::` segments (multi-level, e.g.
    // `super::super::X` — two directories up) — each one more level up from the current parent. A
    // chain that would walk past the crate root is a genuine structural miss, never a guess. Only
    // meaningful after a `crate`/`self`/`super` head (`HeadKind::SameCrate`) — a `Bare` head never
    // enters this loop, so a literal `foo::super::bar` (nonsensical Rust, never real code) is left
    // untouched rather than mis-peeled.
    if head_kind == HeadKind::SameCrate {
        while let UseTree::Path(next) = rest {
            if next.ident != "super" {
                break;
            }
            if prefix.is_empty() {
                return None;
            }
            prefix.pop();
            rest = &next.tree;
        }
    }
    let mut out = Vec::new();
    flatten(rest, &mut prefix, head_kind, &mut out);
    Some(out)
}

fn flatten(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    head_kind: HeadKind,
    out: &mut Vec<UseCandidate>,
) {
    match tree {
        UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            flatten(&p.tree, prefix, head_kind, out);
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
                head_kind,
            });
        }
        UseTree::Rename(r) => out.push(UseCandidate {
            module_segs: prefix.clone(),
            kind: CandidateKind::Rename {
                from: r.ident.to_string(),
                to: r.rename.to_string(),
            },
            head_kind,
        }),
        UseTree::Glob(_) => out.push(UseCandidate {
            module_segs: prefix.clone(),
            kind: CandidateKind::Glob,
            head_kind,
        }),
        UseTree::Group(g) => {
            for t in &g.items {
                flatten(t, prefix, head_kind, out);
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

/// The batch-wide cross-nodule symbol table: a lookup key -> that sibling's [`NoduleSymbols`]. The
/// key is EITHER a bare, dot-joined Rust crate-root-relative module path (`"checkty"`, `"foo.bar"`
/// — when the inserting file has no derivable crate identity, e.g. a `src`-ancestor-less test
/// fixture: byte-identical to pre-M-1084 behavior) OR that same module path qualified under the
/// file's own crate identity via [`Self::qualify_key`] (`"mycelium_std_rand"`,
/// `"mycelium_std_rand.rng"` — M-1084's cross-phylum extension). Built once per batch (see
/// `batch.rs::build_symbol_table`) from every file's baseline-pass [`crate::gap::GapReport`]; never
/// hand-merged from two sources with a colliding key (each real file has exactly one derived key).
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

    /// Resolve `name` in `module_key` (dot-joined Rust module-path segments, bare OR
    /// crate-qualified — see struct docs). Returns the **home-qualified** Mycelium nodule path to
    /// emit against — never the bare `module_key`, never a guessed rename (VR-5; mirrors DN-113
    /// `qualify_cross_phylum`'s never-bare discipline).
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

    /// Qualify `module_key` under `crate_ident`'s own namespace — `crate_ident` alone when
    /// `module_key` is empty (the crate-root case), else `"{crate_ident}.{module_key}"` — never a
    /// bare, unqualified collapse across crates (M-1084: mirrors DN-113 `qualify_cross_phylum`'s own
    /// never-bare discipline, one dot-joined dimension instead of `::`, matching this table's own
    /// `module_key` convention).
    pub fn qualify_key(crate_ident: &str, module_key: &str) -> String {
        if module_key.is_empty() {
            crate_ident.to_string()
        } else {
            format!("{crate_ident}.{module_key}")
        }
    }

    /// The lookup key(s) to try, IN PRECEDENCE ORDER, for one candidate leaf — the single policy
    /// both `transpile::dispatch_use` (via `emit::cross_nodule_resolve`, one key at a time — the
    /// `EmitCtx` thread-local mediates `emit.rs` access) and `batch.rs::scan_pub_needed` (direct
    /// `&SymbolTable` access) consult, so the "which key(s), in what order" policy lives in exactly
    /// one place (DRY).
    ///
    /// [`HeadKind::SameCrate`] (`crate::`/`self::`/`super::`) yields exactly one key, qualified
    /// under `current_crate` when derivable, else the bare `module_key` (no real crate context —
    /// e.g. a `src`-ancestor-less test fixture; byte-identical to pre-M-1084 behavior).
    /// [`HeadKind::Bare`] yields up to two: the same-crate interpretation first (Rust's own
    /// precedence — a local crate-root item shadows a same-named extern crate), then the
    /// cross-phylum interpretation (the head segment itself read AS the named phylum's own
    /// extern-crate identifier).
    pub fn candidate_lookup_keys(
        current_crate: Option<&str>,
        candidate: &UseCandidate,
    ) -> Vec<String> {
        let module_key = Self::module_key(&candidate.module_segs);
        let same_crate_key = match current_crate {
            Some(c) => Self::qualify_key(c, &module_key),
            None => module_key,
        };
        let mut keys = vec![same_crate_key];
        if candidate.head_kind == HeadKind::Bare {
            if let Some((head, rest)) = candidate.module_segs.split_first() {
                keys.push(Self::qualify_key(head, &Self::module_key(rest)));
            }
        }
        keys
    }
}
