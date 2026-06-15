//! The **data registry `Σ`** and constructor references (`#T#i`) — RFC-0001 §4.3 (r3); RFC-0007
//! §4.2; ADR-003 (Unison identity). M-320.
//!
//! r3 folds algebraic data into the Core IR (RFC-0011). A data **value** is built by a
//! [`Node::Construct`](crate::Node) and scrutinised by a [`Node::Match`](crate::Node); the data
//! **declarations** those nodes reference live **here, in the registry — not in the term grammar**
//! (the GHC-Core/Lean/Coq/Unison convergence, RFC-0007 §4.2). Keeping declarations out of the term
//! language is what preserves WF4: a `Construct`/`Match` node hashes over its structure plus the
//! [`CtorRef`] hashes it mentions, and the term grammar does not grow with every data type.
//!
//! # Identity (RFC-0007 §4.2; ADR-003)
//! A declaration `type T<a…> = C₁(τ…) | … | Cₙ(τ…)` is content-addressed over its **α-normalised
//! structure** — constructor order is significant, field types (incl. their `Repr`) are
//! significant, **names are not identity**. A constructor reference is `#T#i` ([`CtorRef`]): the
//! declaration hash ‖ the constructor index. A **self-recursive** declaration hashes its own
//! occurrences as a cycle **placeholder** (the Unison scheme): `Nat = Z | S(Nat)`'s `S` field is a
//! back-reference, so it is encoded as a placeholder, never the (circular) final hash.
//!
//! # Scope (honesty)
//! **Self-recursion is fully realised and tested** (`Nat`, `Bytes`, `List`-shaped types — the r3
//! reachable fragment). **Mutual recursion** (a multi-member cycle) is **R7-Q3, deferred to
//! RFC-0001 r4** (the L1 prototype accepts only self-recursion). The general cycle machinery below
//! *handles* a multi-member group structurally, but its canonical member ordering is provisional
//! (insertion-order tie-break) until r4 fixes the full Unison hash-ordering — flagged, never
//! silently assumed.

use std::collections::BTreeMap;

use crate::content::Canon;
use crate::id::ContentHash;
use crate::repr::Repr;

/// A constructor reference `#T#i` (RFC-0007 §4.2): the content hash of a data declaration and the
/// constructor's index within it. Two constructors are the *same* constructor iff their declaration
/// hash and index agree — names play no part (ADR-003).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CtorRef {
    decl: ContentHash,
    index: u32,
}

impl CtorRef {
    /// Build a constructor reference from a declaration hash and a constructor index.
    #[must_use]
    pub fn new(decl: ContentHash, index: u32) -> Self {
        CtorRef { decl, index }
    }

    /// The referenced data declaration's content hash (`#T`).
    #[must_use]
    pub fn decl(&self) -> &ContentHash {
        &self.decl
    }

    /// The constructor's index within its declaration (`#i`).
    #[must_use]
    pub fn index(&self) -> u32 {
        self.index
    }
}

impl core::fmt::Display for CtorRef {
    /// The Unison spelling `#<declhash>#<i>` (RFC-0007 §4.2).
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#{}#{}", self.decl.as_str(), self.index)
    }
}

/// A field type within a resolved declaration: a representation type, or a (possibly cyclic) data
/// type reference. This is the *identity-bearing* field shape (RFC-0007 §4.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldTy {
    /// A representation-typed field (`Binary{n}` | `Ternary{m}` | `Dense{…}` | `VSA{…}`).
    Repr(Repr),
    /// A data-typed field, referencing another (or the same, recursively) declaration by hash.
    Data(ContentHash),
}

/// One constructor of a resolved declaration: its field types, in declaration order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtorDecl {
    /// Field types, in declaration order (the index is the field's `#i` position).
    pub fields: Vec<FieldTy>,
}

/// A resolved, content-addressed data declaration: its constructors in declaration order (the index
/// is the `#i` of [`CtorRef`]). Names are stored separately (they are not identity — ADR-003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDecl {
    /// Constructors, in declaration order.
    pub ctors: Vec<CtorDecl>,
}

// --- Build-time specs (names are build keys, never hashed) ------------------------------------

/// A build-time field spec: a representation field, or a data field referencing another declaration
/// **by name** (the name is a build key for resolving references — it is *not* hashed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldSpec {
    /// A representation-typed field.
    Repr(Repr),
    /// A data-typed field, by the referenced declaration's build-time name.
    Data(String),
}

/// A build-time constructor spec: its fields, in declaration order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtorSpec {
    /// The fields, in declaration order.
    pub fields: Vec<FieldSpec>,
}

/// A build-time declaration spec: its constructors, in declaration order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclSpec {
    /// The constructors, in declaration order.
    pub ctors: Vec<CtorSpec>,
}

/// Why building a [`DataRegistry`] from specs failed — always explicit (never a silent drop).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// A field references a data declaration name that is not in the spec set.
    UnknownTypeRef {
        /// The declaration whose field has the dangling reference.
        in_decl: String,
        /// The unresolved referenced name.
        missing: String,
    },
}

impl core::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RegistryError::UnknownTypeRef { in_decl, missing } => write!(
                f,
                "data declaration `{in_decl}` references unknown type `{missing}`"
            ),
        }
    }
}

impl std::error::Error for RegistryError {}

/// The content-addressed data registry `Σ` (RFC-0001 §4.3 r3): the resolved declarations keyed by
/// their content hash, plus the build-time `name → hash` resolution used to form [`CtorRef`]s.
///
/// Built once from a set of [`DeclSpec`]s ([`DataRegistry::build`]); the elaborator and the
/// interpreter share *one* registry so that a constructor's identity (`#T#i`) is the same on every
/// execution path (the NFR-7 differential is about *one* `CtorRef` set, never two).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataRegistry {
    /// Resolved declarations, keyed by content hash.
    decls: BTreeMap<ContentHash, DataDecl>,
    /// Build-time name → content hash (names are metadata, kept only for reference resolution).
    by_name: BTreeMap<String, ContentHash>,
}

impl DataRegistry {
    /// Build the registry from a set of named declaration specs, computing every declaration's
    /// content hash (cycle-aware: self-references hash as a placeholder, RFC-0007 §4.2). Returns an
    /// explicit [`RegistryError`] for any dangling reference — never a partial registry.
    pub fn build(specs: &BTreeMap<String, DeclSpec>) -> Result<Self, RegistryError> {
        // Validate references first (never proceed on a dangling ref).
        for (name, decl) in specs {
            for ctor in &decl.ctors {
                for field in &ctor.fields {
                    if let FieldSpec::Data(r) = field {
                        if !specs.contains_key(r) {
                            return Err(RegistryError::UnknownTypeRef {
                                in_decl: name.clone(),
                                missing: r.clone(),
                            });
                        }
                    }
                }
            }
        }

        let sccs = strongly_connected_components(specs);
        let mut by_name: BTreeMap<String, ContentHash> = BTreeMap::new();
        let mut decls: BTreeMap<ContentHash, DataDecl> = BTreeMap::new();

        // Process SCCs dependencies-first (the SCC list is already in reverse-topological order),
        // so every out-of-cycle data reference already has a hash when we encode a member.
        for scc in &sccs {
            let in_cycle: BTreeMap<&str, usize> = scc
                .iter()
                .enumerate()
                .map(|(i, n)| (n.as_str(), i))
                .collect();

            // The group hash: encode each member's structure, in canonical (here: spec) order, with
            // in-cycle references as their placeholder index and out-of-cycle references as their
            // (already computed) hash. A singleton self-loop is the common r3 case; multi-member
            // ordering is provisional (R7-Q3, r4).
            let group_hash = {
                let mut c = Canon::new();
                c.tag(crate::content::tag::DATADECL);
                c.u64(scc.len() as u64);
                for name in scc {
                    let decl = &specs[name];
                    encode_decl(&mut c, decl, &in_cycle, &by_name);
                }
                c.finish()
            };

            // Each member's final hash = H(group ‖ member index) — distinguishing members of the
            // same cycle while sharing the cycle's structural identity (the Unison recipe).
            for (i, name) in scc.iter().enumerate() {
                let mut c = Canon::new();
                c.tag(crate::content::tag::DATADECL);
                c.hash(&group_hash);
                c.u32(i as u32);
                let member_hash = c.finish();
                by_name.insert(name.clone(), member_hash.clone());
            }

            // Now that every member has a hash, build the resolved `DataDecl`s (cycle references can
            // now be filled in with the just-computed member hashes).
            for name in scc {
                let decl = &specs[name];
                let resolved = resolve_decl(decl, &by_name);
                let hash = by_name[name].clone();
                decls.insert(hash, resolved);
            }
        }

        Ok(DataRegistry { decls, by_name })
    }

    /// The content hash of the declaration registered under build-time name `name`, if any.
    #[must_use]
    pub fn decl_hash(&self, name: &str) -> Option<&ContentHash> {
        self.by_name.get(name)
    }

    /// A [`CtorRef`] for constructor `index` of the declaration named `name`, if the declaration is
    /// registered and the index is in range.
    #[must_use]
    pub fn ctor_ref(&self, name: &str, index: u32) -> Option<CtorRef> {
        let hash = self.by_name.get(name)?;
        let decl = self.decls.get(hash)?;
        if (index as usize) < decl.ctors.len() {
            Some(CtorRef::new(hash.clone(), index))
        } else {
            None
        }
    }

    /// The resolved declaration at content hash `hash`, if registered.
    #[must_use]
    pub fn decl(&self, hash: &ContentHash) -> Option<&DataDecl> {
        self.decls.get(hash)
    }

    /// The constructor declaration a [`CtorRef`] points at, if registered and in range.
    #[must_use]
    pub fn ctor(&self, ctor: &CtorRef) -> Option<&CtorDecl> {
        self.decls
            .get(ctor.decl())
            .and_then(|d| d.ctors.get(ctor.index() as usize))
    }

    /// The number of fields the referenced constructor takes (its saturation arity, WF6).
    #[must_use]
    pub fn field_count(&self, ctor: &CtorRef) -> Option<usize> {
        self.ctor(ctor).map(|c| c.fields.len())
    }

    /// The number of constructors of the data type the [`CtorRef`] belongs to (for WF7 coverage).
    #[must_use]
    pub fn ctor_count(&self, ctor: &CtorRef) -> Option<usize> {
        self.decls.get(ctor.decl()).map(|d| d.ctors.len())
    }
}

/// Encode a declaration's identity-bearing structure into `c`: each constructor (order significant)
/// and its fields (order significant), with names excluded. In-cycle data references become a
/// placeholder index; out-of-cycle data references become the referenced hash.
fn encode_decl(
    c: &mut Canon,
    decl: &DeclSpec,
    in_cycle: &BTreeMap<&str, usize>,
    by_name: &BTreeMap<String, ContentHash>,
) {
    c.tag(crate::content::tag::CTOR_DECL);
    c.u64(decl.ctors.len() as u64);
    for ctor in &decl.ctors {
        c.u64(ctor.fields.len() as u64);
        for field in &ctor.fields {
            match field {
                FieldSpec::Repr(r) => {
                    c.tag(crate::content::tag::FIELD_REPR);
                    c.repr(r);
                }
                FieldSpec::Data(name) => {
                    if let Some(&idx) = in_cycle.get(name.as_str()) {
                        c.tag(crate::content::tag::FIELD_CYCLE);
                        c.u32(idx as u32);
                    } else {
                        c.tag(crate::content::tag::FIELD_DATA);
                        // Resolved earlier (dependencies-first); a reference inside the validated
                        // spec set with a non-cycle target always has a hash by now.
                        c.hash(&by_name[name]);
                    }
                }
            }
        }
    }
}

/// Build the resolved [`DataDecl`] for `decl`, with each data field carrying the referenced
/// declaration's (now-computed) hash.
fn resolve_decl(decl: &DeclSpec, by_name: &BTreeMap<String, ContentHash>) -> DataDecl {
    DataDecl {
        ctors: decl
            .ctors
            .iter()
            .map(|ctor| CtorDecl {
                fields: ctor
                    .fields
                    .iter()
                    .map(|f| match f {
                        FieldSpec::Repr(r) => FieldTy::Repr(r.clone()),
                        FieldSpec::Data(name) => FieldTy::Data(by_name[name].clone()),
                    })
                    .collect(),
            })
            .collect(),
    }
}

/// Tarjan's strongly-connected components over the declaration dependency graph (an edge `A → B`
/// when `A` has a data field of type `B`). Returns the SCCs in **reverse-topological order**
/// (dependencies before dependents), which is exactly the order [`DataRegistry::build`] needs so
/// every out-of-cycle reference is hashed first. Member order within an SCC is the specs' iteration
/// (name) order — a provisional tie-break for multi-member cycles (R7-Q3, deferred to r4).
fn strongly_connected_components(specs: &BTreeMap<String, DeclSpec>) -> Vec<Vec<String>> {
    // Successors: the distinct data-typed field references of each declaration.
    let succ = |name: &str| -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for ctor in &specs[name].ctors {
            for field in &ctor.fields {
                if let FieldSpec::Data(r) = field {
                    if !out.contains(r) {
                        out.push(r.clone());
                    }
                }
            }
        }
        out
    };

    struct Tarjan {
        index: BTreeMap<String, usize>,
        low: BTreeMap<String, usize>,
        on_stack: BTreeMap<String, bool>,
        stack: Vec<String>,
        next: usize,
        out: Vec<Vec<String>>,
    }

    impl Tarjan {
        fn run(&mut self, succ: &dyn Fn(&str) -> Vec<String>, v: &str) {
            self.index.insert(v.to_owned(), self.next);
            self.low.insert(v.to_owned(), self.next);
            self.next += 1;
            self.stack.push(v.to_owned());
            self.on_stack.insert(v.to_owned(), true);

            for w in succ(v) {
                if !self.index.contains_key(&w) {
                    self.run(succ, &w);
                    let lw = self.low[&w];
                    let lv = self.low[v];
                    self.low.insert(v.to_owned(), lv.min(lw));
                } else if *self.on_stack.get(&w).unwrap_or(&false) {
                    let iw = self.index[&w];
                    let lv = self.low[v];
                    self.low.insert(v.to_owned(), lv.min(iw));
                }
            }

            if self.low[v] == self.index[v] {
                let mut scc = Vec::new();
                loop {
                    let w = self.stack.pop().expect("non-empty while popping an SCC");
                    self.on_stack.insert(w.clone(), false);
                    scc.push(w.clone());
                    if w == v {
                        break;
                    }
                }
                // Tarjan emits SCCs in reverse-topological order already. Sort members by name for a
                // deterministic, provisional intra-cycle order (singletons are unaffected).
                scc.sort();
                self.out.push(scc);
            }
        }
    }

    let mut t = Tarjan {
        index: BTreeMap::new(),
        low: BTreeMap::new(),
        on_stack: BTreeMap::new(),
        stack: Vec::new(),
        next: 0,
        out: Vec::new(),
    };
    for name in specs.keys() {
        if !t.index.contains_key(name) {
            t.run(&succ, name);
        }
    }
    t.out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nat_spec() -> BTreeMap<String, DeclSpec> {
        // type Nat = Z | S(Nat)
        let mut m = BTreeMap::new();
        m.insert(
            "Nat".to_owned(),
            DeclSpec {
                ctors: vec![
                    CtorSpec { fields: vec![] },
                    CtorSpec {
                        fields: vec![FieldSpec::Data("Nat".to_owned())],
                    },
                ],
            },
        );
        m
    }

    #[test]
    fn self_recursive_decl_hashes_without_looping() {
        let reg = DataRegistry::build(&nat_spec()).expect("builds");
        let z = reg.ctor_ref("Nat", 0).expect("Z");
        let s = reg.ctor_ref("Nat", 1).expect("S");
        assert_eq!(z.decl(), s.decl(), "same declaration");
        assert_eq!(z.index(), 0);
        assert_eq!(s.index(), 1);
        assert_eq!(reg.field_count(&z), Some(0));
        assert_eq!(reg.field_count(&s), Some(1));
        assert_eq!(reg.ctor_count(&s), Some(2));
    }

    #[test]
    fn identity_is_structural_not_nominal() {
        // The same structure under a different *name* gets the same declaration hash (names are not
        // identity — ADR-003).
        let mut renamed = BTreeMap::new();
        renamed.insert(
            "Peano".to_owned(),
            DeclSpec {
                ctors: vec![
                    CtorSpec { fields: vec![] },
                    CtorSpec {
                        fields: vec![FieldSpec::Data("Peano".to_owned())],
                    },
                ],
            },
        );
        let nat = DataRegistry::build(&nat_spec()).unwrap();
        let peano = DataRegistry::build(&renamed).unwrap();
        assert_eq!(
            nat.decl_hash("Nat"),
            peano.decl_hash("Peano"),
            "α-equivalent declarations collide regardless of name"
        );
    }

    #[test]
    fn constructor_order_is_identity_bearing() {
        // Z | S(Nat)  vs  S(Nat) | Z  are different declarations (order significant).
        let mut swapped = BTreeMap::new();
        swapped.insert(
            "Nat".to_owned(),
            DeclSpec {
                ctors: vec![
                    CtorSpec {
                        fields: vec![FieldSpec::Data("Nat".to_owned())],
                    },
                    CtorSpec { fields: vec![] },
                ],
            },
        );
        let a = DataRegistry::build(&nat_spec()).unwrap();
        let b = DataRegistry::build(&swapped).unwrap();
        assert_ne!(a.decl_hash("Nat"), b.decl_hash("Nat"));
    }

    #[test]
    fn field_repr_is_identity_bearing() {
        // type B8 = Wrap(Binary{8})  vs  type B8 = Wrap(Binary{4}) differ.
        let mk = |w| {
            let mut m = BTreeMap::new();
            m.insert(
                "W".to_owned(),
                DeclSpec {
                    ctors: vec![CtorSpec {
                        fields: vec![FieldSpec::Repr(Repr::Binary { width: w })],
                    }],
                },
            );
            DataRegistry::build(&m).unwrap()
        };
        assert_ne!(mk(8).decl_hash("W"), mk(4).decl_hash("W"));
    }

    #[test]
    fn a_dangling_reference_is_an_explicit_error() {
        let mut m = BTreeMap::new();
        m.insert(
            "Tree".to_owned(),
            DeclSpec {
                ctors: vec![CtorSpec {
                    fields: vec![FieldSpec::Data("Forest".to_owned())], // not declared
                }],
            },
        );
        assert_eq!(
            DataRegistry::build(&m).unwrap_err(),
            RegistryError::UnknownTypeRef {
                in_decl: "Tree".to_owned(),
                missing: "Forest".to_owned(),
            }
        );
    }

    #[test]
    fn out_of_cycle_reference_resolves_dependencies_first() {
        // type Byte = MkByte(Binary{8});  type Pair = MkPair(Byte, Byte) — no cycle, Byte first.
        let mut m = BTreeMap::new();
        m.insert(
            "Byte".to_owned(),
            DeclSpec {
                ctors: vec![CtorSpec {
                    fields: vec![FieldSpec::Repr(Repr::Binary { width: 8 })],
                }],
            },
        );
        m.insert(
            "Pair".to_owned(),
            DeclSpec {
                ctors: vec![CtorSpec {
                    fields: vec![
                        FieldSpec::Data("Byte".to_owned()),
                        FieldSpec::Data("Byte".to_owned()),
                    ],
                }],
            },
        );
        let reg = DataRegistry::build(&m).expect("builds");
        let pair = reg.ctor_ref("Pair", 0).expect("MkPair");
        let byte_decl = reg.decl_hash("Byte").unwrap().clone();
        // The Pair constructor's two fields both resolve to the Byte declaration hash.
        let decl = reg.ctor(&pair).unwrap();
        assert_eq!(
            decl.fields,
            vec![FieldTy::Data(byte_decl.clone()), FieldTy::Data(byte_decl)]
        );
    }
}
