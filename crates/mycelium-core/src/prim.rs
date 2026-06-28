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

/// How a prim's operand and result *widths* relate. Most of the builtin set is width-preserving —
/// every operand and the result share one width (`bit.xor : Binary{n} × Binary{n} → Binary{n}`,
/// `trit.add : Ternary{m} × Ternary{m} → Ternary{m}`, the unary cases trivially: [`WidthRel::Uniform`]).
/// The reduce-to-`Bool` comparison prims (`cmp.eq`/`cmp.lt`, RFC-0032 D1) are the exception — they
/// **collapse** to a fixed `Binary{1}` independent of the operand width ([`WidthRel::Collapse`]). New
/// rules (e.g. a width-changing pack) are added as variants, never silently assumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidthRel {
    /// All operands and the result share one width.
    Uniform,
    /// The result width is **fixed and independent** of the operands' shared width — the
    /// width-collapsing rule of the reduce-to-`Bool` comparison prims (`cmp.eq`/`cmp.lt`, RFC-0032
    /// D1): two equal-width operands reduce to a one-bit `Binary{1}` truth value. (Operand widths
    /// must still agree; only the result is decoupled.)
    Collapse,
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

    /// The default table: the closed v0 kernel-prim set — the identity, the elementwise binary logic
    /// (`bit.*`), the fixed-width balanced-ternary arithmetic (`trit.*`, M-111), the reduce-to-`Bool`
    /// comparison prims (`cmp.eq`/`cmp.lt`, RFC-0032 D1), and the never-silent binary arithmetic
    /// (`bit.add`/`bit.sub`, RFC-0032 D2). Every entry is `intrinsic = Exact`; all are width-`Uniform`
    /// **except** `cmp.eq`/`cmp.lt`, which are width-`Collapse` (operand width → `Binary{1}`). This is
    /// the single source of truth the `mycelium-interp` intrinsic and the `mycelium-l1` surface table
    /// are checked against.
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
        // RFC-0032 D1 (M-747): the reduce-to-`Bool` comparison prims are width-*collapsing* — two
        // equal-width operands of either paradigm reduce to a `Binary{1}` truth value. The operands
        // are typed `Any` because each may be Binary OR Ternary; this does NOT permit a *cross*-
        // paradigm comparison — the same-paradigm + equal-width constraint is enforced (never-silent,
        // G2) by the interpreter prim (`prims.rs::cmp_repr_operands`) and the L1 checker branch
        // (`checkty.rs`), since the per-operand `Any` paradigm model cannot express "both agree".
        let cmp = || PrimDecl {
            sig: PrimSig {
                operands: vec![Any, Any],
                result: Binary,
                width: WidthRel::Collapse,
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
        // RFC-0032 D1 (M-747): reduce-to-`Bool` comparison/equality (width-collapsing → Binary{1}).
        t.insert("cmp.eq", cmp());
        t.insert("cmp.lt", cmp());
        // RFC-0032 D2 (M-748): never-silent fixed-width binary arithmetic (width-uniform).
        t.insert("bit.add", exact(vec![Binary, Binary], Binary));
        t.insert("bit.sub", exact(vec![Binary, Binary], Binary));
        // DN-41 (M-798): never-silent `Binary` width-cast (zero-extend widen / checked narrow).
        // `intrinsic = Exact` (the widen/identity/in-range-narrow result equals the unsigned value
        // exactly; a lossy narrow is a never-silent *runtime* refusal, not a non-Exact intrinsic).
        // **Width-model note (FLAG):** the Π `WidthRel` model is `Uniform`/`Collapse` only — it has
        // **no first-class width-*change* relation**, so this width-cast prim cannot express "result
        // width = the *second* (witness) operand's width" in the coarse table. It is recorded
        // `Uniform` here as the nearest tag; the real never-silent typing — both operands `Binary`,
        // result width = witness width `M`, the narrowing-fit refusal — is enforced by the interpreter
        // prim (`prims.rs::prim_width_cast`) and the L1 checker (`checkty.rs`), exactly as the seq/
        // bytes prims' real typing lives in their interpreter prims (same paradigm-model escape hatch).
        // A first-class width-change `WidthRel` is a deliberate, RFC-unpinned extension left for later.
        t.insert("bit.width_cast", exact(vec![Binary, Binary], Binary));
        // RFC-0032 D3 (M-749): never-silent indexed-sequence access. Both are `intrinsic = Exact`
        // (total/decidable over the in-range domain). **Paradigm-model note (FLAG):** the Π paradigm
        // model is `Binary`/`Ternary`/`Any` only — it has no first-class `Seq` paradigm, and a
        // sequence-element result type cannot be expressed in it. So the seq operand and the
        // `seq.get` element result are typed `Any` here (the table's existing paradigm-polymorphic
        // escape hatch, as for `core.id`); the real never-silent typing — "operand must be a `Seq`",
        // out-of-bounds refusal, the element repr of the result — is enforced by the interpreter prim
        // (`prims.rs::{as_seq,as_index,prim_seq_get}`), not encoded in this coarse signature. A
        // first-class `Seq` paradigm in `PrimParadigm` is a deliberate, RFC-unpinned extension left
        // for the surface-typing work (it ripples into the checker + content-addressing).
        t.insert("seq.len", exact(vec![Any], Binary));
        t.insert("seq.get", exact(vec![Any, Binary], Any));
        // RFC-0032 D4 (M-750): never-silent byte-string access. All `intrinsic = Exact`. Same
        // paradigm-model FLAG as the seq prims: the Π model has no first-class `Bytes` paradigm, so
        // the bytes operand/result are typed `Any` (the real "operand must be `Bytes`" + out-of-range
        // refusals are enforced by the interpreter prims `prims.rs::{as_bytes_payload,prim_bytes_*}`).
        // `bytes.len`/`bytes.get` produce a `Binary` (length / a `Binary{8}` byte); `bytes.slice`/
        // `bytes.concat` produce `Bytes` (typed `Any` here).
        t.insert("bytes.len", exact(vec![Any], Binary));
        t.insert("bytes.get", exact(vec![Any, Binary], Binary));
        t.insert("bytes.slice", exact(vec![Any, Binary, Binary], Any));
        t.insert("bytes.concat", exact(vec![Any, Any], Any));
        // DN-58 §A (M-817): the `Binary` `Fuse` semilattice meet (bitwise-AND). `intrinsic = Exact`
        // (a total greatest-lower-bound). The user-`Data` fuse registers no prim — it elaborates to the
        // resolved `Fuse::join` call (DN-58 §A.5) — and the non-`Binary` reprs have no committed meet
        // (DN-58 §A.6 F-A3), so this is the only `fuse_join:*` kernel prim.
        t.insert("fuse_join:binary", exact(vec![Binary, Binary], Binary));
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
