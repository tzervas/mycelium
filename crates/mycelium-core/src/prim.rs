//! The **prim table `Π`** as content-addressed declarations — RFC-0007 §4.4 (T-Op); RFC-0007 §8
//! R7-Q4; DN-10 §3; ADR-003 (Unison identity); RFC-0001 §4.7 (the intrinsic guarantee `g_f`). M-390.
//!
//! RFC-0007 §4.4 gives every primitive `p` a signature `Π(p) = (τ₁…τₙ) → τ` and (RFC-0001 §4.7) an
//! *intrinsic guarantee* `g_f` it contributes to the result's guarantee meet. Historically `Π` was a
//! fixed builtin table hard-coded in the elaborator/typechecker and an `Exact` constant in the
//! interpreter. Here it becomes **declarations with their own content addresses** — exactly the model
//! the data registry `Σ` ([`crate::data::DataRegistry`]) already uses for constructors (RFC-0001
//! §4.3 r3): each prim is keyed by the content hash of its *signature + intrinsic guarantee*, with
//! its (kernel) name kept separately as metadata (ADR-003 — names are not identity). A prim is then
//! an inspectable, EXPLAIN-able registry entry (G2/SC-3), not a black box.
//!
//! # Scope (honesty)
//! Every v0 builtin is `intrinsic = Exact` (the exact, elementwise/arithmetic fragment). The table
//! stores that intrinsic *as data* so a future non-`Exact` prim (e.g. a VSA `bundle`, RFC-0003 §5)
//! is a registry entry carrying its own honest tag — but *how* a non-`Exact` prim's bound-basis is
//! stored with the declaration (a cited theorem vs an empirical fit, with its [`crate::BoundBasis`])
//! is the **RP-7** spike (DN-10 §3.6), deliberately *not* settled here. v0's all-`Exact` table is
//! sound (the prim set is closed and small), so this migration is a *uniformity/inspectability* gain
//! (VR-5: not a correctness fix, not dishonest meanwhile).
//!
//! The migration preserves `Π`-lookup semantics exactly: for every prim `p`,
//! `Π_new(hash(p)) = Π_old(name(p))` (DN-10 §3.4) — guarded by the `Π_new == Π_old` equivalence
//! tests in `mycelium-l1` (the surface table) and `mycelium-interp` (the intrinsic).

use std::collections::BTreeMap;

use crate::content::Canon;
use crate::guarantee::GuaranteeStrength;
use crate::id::ContentHash;

/// The representation paradigm of a prim operand or result (the `τ`'s paradigm in `Π(p)`). `Any` is
/// the paradigm-polymorphic identity (`core.id : a → a`); the concrete paradigms pin a prim to
/// `Binary{·}` or `Ternary{·}` (RFC-0007 §4.4). Width is governed separately by [`WidthRel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimParadigm {
    /// Paradigm-polymorphic (the identity prim): any single paradigm, passed through.
    Any,
    /// A `Binary{n}` operand/result.
    Binary,
    /// A `Ternary{m}` operand/result.
    Ternary,
}

/// How a prim's operand and result *widths* relate. The whole v0 builtin set is width-preserving:
/// every operand and the result share one width (`bit.xor : Binary{n} × Binary{n} → Binary{n}`,
/// `trit.add : Ternary{m} × Ternary{m} → Ternary{m}`, the unary cases trivially). This is the single
/// v0 rule; new rules (e.g. a width-changing pack) are added as variants, never silently assumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidthRel {
    /// All operands and the result share one width.
    Uniform,
}

/// A prim's signature `Π(p) = (τ₁…τₙ) → τ` (RFC-0007 §4.4): the per-operand paradigms (arity is their
/// count), the result paradigm, and the width relation. Identity-bearing; names are excluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimSig {
    /// Operand paradigms, in order (the length is the prim's arity).
    pub operands: Vec<PrimParadigm>,
    /// The result paradigm.
    pub result: PrimParadigm,
    /// How operand/result widths relate.
    pub width: WidthRel,
}

impl PrimSig {
    /// The prim's arity (operand count).
    #[must_use]
    pub fn arity(&self) -> usize {
        self.operands.len()
    }
}

/// A resolved, content-addressed prim declaration: its signature and the *intrinsic guarantee* `g_f`
/// it contributes to a result's guarantee meet (RFC-0001 §4.7). The (kernel) name is stored
/// separately in the [`PrimTable`] (it is not identity — ADR-003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimDecl {
    /// The signature `(τ₁…τₙ) → τ`.
    pub sig: PrimSig,
    /// The intrinsic guarantee `g_f` (RFC-0001 §4.7). `Exact` for every v0 builtin.
    pub intrinsic: GuaranteeStrength,
}

impl PrimDecl {
    /// The content hash of this declaration's identity-bearing content (signature + intrinsic
    /// guarantee), with the name excluded (ADR-003). Two prims are the *same* prim iff their
    /// signature and intrinsic agree — domain-separated from node/data hashes so a prim can never
    /// collide with a structural node hash.
    #[must_use]
    pub fn content_hash(&self) -> ContentHash {
        let mut c = Canon::new();
        c.prim_decl(&self.sig, self.intrinsic);
        c.finish()
    }
}

/// A prim reference `#p` (the prim analogue of [`CtorRef`](crate::CtorRef) `#T#i`): the content hash
/// of a [`PrimDecl`]. A term referring to a prim by *identity* refers to it by this hash, not its
/// name (ADR-003).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrimRef(ContentHash);

impl PrimRef {
    /// Build a prim reference from a declaration hash.
    #[must_use]
    pub fn new(decl: ContentHash) -> Self {
        PrimRef(decl)
    }

    /// The referenced declaration's content hash.
    #[must_use]
    pub fn decl(&self) -> &ContentHash {
        &self.0
    }
}

impl core::fmt::Display for PrimRef {
    /// The Unison-style spelling `#<declhash>` (a prim has no constructor index, unlike `#T#i`).
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#{}", self.0.as_str())
    }
}

/// The content-addressed **prim table `Π`** (RFC-0007 §4.4; R7-Q4): resolved declarations keyed by
/// their content hash, plus the build-time `name → hash` resolution used to form [`PrimRef`]s — the
/// same two-map shape as [`DataRegistry`](crate::DataRegistry), so a prim's identity (`#p`) is the
/// same on every path (the NFR-7 differential is over *one* prim set, never two).
///
/// Prims have no inter-references (unlike data, which can be mutually recursive), so building is a
/// flat hash-and-insert — no SCC/cycle handling is needed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrimTable {
    /// Resolved declarations, keyed by content hash.
    decls: BTreeMap<ContentHash, PrimDecl>,
    /// Build-time (kernel) name → content hash (names are metadata — ADR-003).
    by_name: BTreeMap<String, ContentHash>,
}

impl PrimTable {
    /// An empty table.
    #[must_use]
    pub fn new() -> Self {
        PrimTable::default()
    }

    /// Register (or replace) a prim declaration under build-time kernel name `name`, returning its
    /// [`PrimRef`]. Re-registering a name re-points it; identity is the decl hash, not the name.
    pub fn insert(&mut self, name: impl Into<String>, decl: PrimDecl) -> PrimRef {
        let hash = decl.content_hash();
        self.by_name.insert(name.into(), hash.clone());
        self.decls.insert(hash.clone(), decl);
        PrimRef::new(hash)
    }

    /// The default table: the closed v0 kernel-prim set — the identity, the elementwise binary
    /// logic (`bit.*`), and the fixed-width balanced-ternary arithmetic (`trit.*`, M-111). Every
    /// entry is `intrinsic = Exact` and width-`Uniform`. This is the single source of truth the
    /// `mycelium-interp` intrinsic and the `mycelium-l1` surface table are checked against.
    #[must_use]
    pub fn builtins() -> Self {
        use PrimParadigm::{Any, Binary, Ternary};
        let mut t = PrimTable::new();
        let exact = |operands: Vec<PrimParadigm>, result: PrimParadigm| PrimDecl {
            sig: PrimSig {
                operands,
                result,
                width: WidthRel::Uniform,
            },
            intrinsic: GuaranteeStrength::Exact,
        };
        // Identity (paradigm-polymorphic passthrough).
        t.insert("core.id", exact(vec![Any], Any));
        // Elementwise binary logic.
        t.insert("bit.not", exact(vec![Binary], Binary));
        t.insert("bit.and", exact(vec![Binary, Binary], Binary));
        t.insert("bit.or", exact(vec![Binary, Binary], Binary));
        t.insert("bit.xor", exact(vec![Binary, Binary], Binary));
        // Fixed-width balanced-ternary arithmetic (M-111).
        t.insert("trit.neg", exact(vec![Ternary], Ternary));
        t.insert("trit.add", exact(vec![Ternary, Ternary], Ternary));
        t.insert("trit.sub", exact(vec![Ternary, Ternary], Ternary));
        t.insert("trit.mul", exact(vec![Ternary, Ternary], Ternary));
        t
    }

    /// The content hash of the prim registered under kernel name `name`, if any.
    #[must_use]
    pub fn decl_hash(&self, name: &str) -> Option<&ContentHash> {
        self.by_name.get(name)
    }

    /// A [`PrimRef`] for the prim named `name`, if registered.
    #[must_use]
    pub fn prim_ref(&self, name: &str) -> Option<PrimRef> {
        self.by_name.get(name).cloned().map(PrimRef::new)
    }

    /// The resolved declaration at content hash `hash`, if registered.
    #[must_use]
    pub fn decl(&self, hash: &ContentHash) -> Option<&PrimDecl> {
        self.decls.get(hash)
    }

    /// The declaration a [`PrimRef`] points at, if registered.
    #[must_use]
    pub fn resolve(&self, prim: &PrimRef) -> Option<&PrimDecl> {
        self.decls.get(prim.decl())
    }

    /// The declaration registered under kernel name `name`, if any.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&PrimDecl> {
        let hash = self.by_name.get(name)?;
        self.decls.get(hash)
    }

    /// The intrinsic guarantee `g_f` of the prim named `name` (RFC-0001 §4.7), if registered.
    #[must_use]
    pub fn intrinsic(&self, name: &str) -> Option<GuaranteeStrength> {
        self.get(name).map(|d| d.intrinsic)
    }

    /// Whether a prim named `name` is registered.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }

    /// The registered kernel names, sorted.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.by_name.keys().map(String::as_str).collect()
    }

    /// Every entry as `(name, #p, decl)`, in name order — the inspectable surface for EXPLAIN over
    /// prims (DN-10 §3.2 step 4; G2/SC-3): a prim call can report which content-addressed
    /// declaration it resolves to, its signature, and its intrinsic guarantee.
    #[must_use]
    pub fn entries(&self) -> Vec<(&str, PrimRef, &PrimDecl)> {
        self.by_name
            .iter()
            .filter_map(|(name, hash)| {
                self.decls
                    .get(hash)
                    .map(|d| (name.as_str(), PrimRef::new(hash.clone()), d))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{PrimDecl, PrimParadigm, PrimSig, PrimTable, WidthRel};
    use crate::guarantee::GuaranteeStrength;

    fn xor() -> PrimDecl {
        PrimDecl {
            sig: PrimSig {
                operands: vec![PrimParadigm::Binary, PrimParadigm::Binary],
                result: PrimParadigm::Binary,
                width: WidthRel::Uniform,
            },
            intrinsic: GuaranteeStrength::Exact,
        }
    }

    #[test]
    fn hash_is_well_shaped_blake3_and_name_independent() {
        let h = xor().content_hash();
        assert_eq!(h.algo(), "blake3");
        assert_eq!(h.digest().len(), 64);
        // The same declaration under two different kernel names has the same identity (ADR-003).
        let mut t = PrimTable::new();
        let a = t.insert("bit.xor", xor());
        let b = t.insert("bit.xor_alias", xor());
        assert_eq!(a, b, "identity is the signature+intrinsic, not the name");
    }

    #[test]
    fn distinct_signatures_get_distinct_identities() {
        let not = PrimDecl {
            sig: PrimSig {
                operands: vec![PrimParadigm::Binary],
                result: PrimParadigm::Binary,
                width: WidthRel::Uniform,
            },
            intrinsic: GuaranteeStrength::Exact,
        };
        assert_ne!(
            xor().content_hash(),
            not.content_hash(),
            "different arity/paradigm ⇒ different identity"
        );
    }

    #[test]
    fn intrinsic_is_identity_bearing() {
        // A prim whose only difference is the intrinsic guarantee is a *different* declaration —
        // the honesty tag is part of identity (so an Exact prim can never alias an Empirical one).
        let mut declared = xor();
        declared.intrinsic = GuaranteeStrength::Declared;
        assert_ne!(xor().content_hash(), declared.content_hash());
    }

    #[test]
    fn builtins_are_present_and_resolvable() {
        let t = PrimTable::builtins();
        for name in [
            "core.id", "bit.not", "bit.and", "bit.or", "bit.xor", "trit.neg", "trit.add",
            "trit.sub", "trit.mul",
        ] {
            let r = t.prim_ref(name).expect("builtin registered");
            let d = t.resolve(&r).expect("ref resolves");
            assert_eq!(d.intrinsic, GuaranteeStrength::Exact);
            assert_eq!(t.intrinsic(name), Some(GuaranteeStrength::Exact));
        }
        // `entries()` is the EXPLAIN surface: one inspectable entry per builtin.
        assert_eq!(t.entries().len(), 9);
    }

    #[test]
    fn build_is_deterministic() {
        // Two independent builds produce the same hashes (content-addressing is a pure function).
        assert_eq!(PrimTable::builtins(), PrimTable::builtins());
    }
}
