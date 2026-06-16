//! Content-addressing: hash-of-AST definition identity, with names as separable metadata
//! (RFC-0001 §4.6; ADR-003). M-103.
//!
//! ```text
//! hash(def) = H( normalize(structure(def)) ‖ types_with_repr(def) ‖ static_contract(def) )
//! ```
//!
//! **Hashed (identity-bearing):** the normalized node structure (α-renamed so binder *names* don't
//! matter and bound variables are position-independent de Bruijn indices), the result/operand types
//! *including [`Repr`]*, the literal value of a [`Node::Const`], the operator name of a
//! [`Node::Op`], and the static contract of a [`Node::Swap`] (its target `Repr` and `policy`).
//!
//! **Not hashed (metadata):** human names (binder names, free-variable spellings are kept only
//! because they are part of the *contract* of an open term — see below), source spans, comments,
//! and **all dynamic value metadata** ([`crate::meta::Meta`]: provenance, measured sparsity,
//! realized bounds, `policy_used`). Names are stored separately in a [`Names`] map; renaming a
//! definition does not change its identity.
//!
//! Consequences (the M-103 acceptance, RFC-0001 §4.6): two definitions differing only in
//! representation paradigm get *different* hashes; a definition and any α-renaming/reformatting get
//! the *same* hash; identical definitions collide.
//!
//! The kernel hash is **BLAKE3**; the encoding fed to it is domain-separated and length-prefixed so
//! that distinct structures can never collide by concatenation ambiguity (an injective framing).

use std::collections::HashMap;

use crate::id::ContentHash;
use crate::node::{Alt, Node, VarId};
use crate::repr::{Repr, ScalarKind, SparsityClass};
use crate::value::{Payload, Trit, Value};

/// Domain-separation tags — one byte per syntactic form, so the framing is injective across kinds.
pub(crate) mod tag {
    pub const VAR_BOUND: u8 = 0x01;
    pub const VAR_FREE: u8 = 0x02;
    pub const CONST: u8 = 0x03;
    pub const LET: u8 = 0x04;
    pub const OP: u8 = 0x05;
    pub const SWAP: u8 = 0x06;

    pub const REPR_BINARY: u8 = 0x10;
    pub const REPR_TERNARY: u8 = 0x11;
    pub const REPR_DENSE: u8 = 0x12;
    pub const REPR_VSA: u8 = 0x13;

    pub const PAYLOAD_BITS: u8 = 0x20;
    pub const PAYLOAD_TRITS: u8 = 0x21;
    pub const PAYLOAD_SCALARS: u8 = 0x22;
    pub const PAYLOAD_HYPERVECTOR: u8 = 0x23;

    pub const SPARSITY_DENSE: u8 = 0x30;
    pub const SPARSITY_SPARSE: u8 = 0x31;

    // r3 (RFC-0001 §4.3/§4.5, RFC-0011): the data registry Σ + the Construct/Match nodes.
    // (0x07 is the standalone `operation_hash` PRIM tag — kept distinct here.)
    pub const CONSTRUCT: u8 = 0x08;
    pub const MATCH: u8 = 0x09;
    pub const ALT_CTOR: u8 = 0x40;
    pub const ALT_LIT: u8 = 0x41;
    pub const MATCH_DEFAULT: u8 = 0x42;
    pub const MATCH_NO_DEFAULT: u8 = 0x43;

    pub const DATADECL: u8 = 0x50;
    pub const CTOR_DECL: u8 = 0x51;
    pub const FIELD_REPR: u8 = 0x52;
    pub const FIELD_DATA: u8 = 0x53; // an out-of-cycle data field: continues with the decl hash
    pub const FIELD_CYCLE: u8 = 0x54; // an in-cycle data field: continues with a placeholder index
    pub const CTOR_REF: u8 = 0x55;
    pub const DATUM: u8 = 0x56;

    // r4 (RFC-0001 r4; RFC-0007 §4.1): the function/recursion nodes.
    pub const LAM: u8 = 0x0a;
    pub const APP: u8 = 0x0b;
    pub const FIX: u8 = 0x0c;
}

/// A canonical, injective, metadata-free byte encoder feeding a [`blake3::Hasher`]. Every write is
/// either a fixed-width integer or a length-prefixed blob, so no two distinct structures share an
/// encoding.
pub(crate) struct Canon {
    h: blake3::Hasher,
}

impl Canon {
    pub(crate) fn new() -> Self {
        Canon {
            h: blake3::Hasher::new(),
        }
    }

    pub(crate) fn tag(&mut self, t: u8) {
        self.h.update(&[t]);
    }

    pub(crate) fn u32(&mut self, n: u32) {
        self.h.update(&n.to_le_bytes());
    }

    pub(crate) fn u64(&mut self, n: u64) {
        self.h.update(&n.to_le_bytes());
    }

    /// A length-prefixed byte blob (the prefix makes the framing injective).
    pub(crate) fn blob(&mut self, bytes: &[u8]) {
        self.u64(bytes.len() as u64);
        self.h.update(bytes);
    }

    pub(crate) fn str(&mut self, s: &str) {
        self.blob(s.as_bytes());
    }

    /// A finite-precision scalar by its exact bit pattern — deterministic and bit-faithful (so e.g.
    /// `+0.0` and `-0.0` are distinct identities, as they are distinct literals).
    fn f64(&mut self, x: f64) {
        self.h.update(&x.to_bits().to_le_bytes());
    }

    pub(crate) fn finish(self) -> ContentHash {
        let hex = self.h.finalize().to_hex();
        // BLAKE3 hex is 64 lowercase [0-9a-f] chars — always a well-formed digest.
        ContentHash::from_parts("blake3", hex.as_str()).expect("blake3 hex is a valid digest")
    }

    /// Absorb a [`ContentHash`] (e.g. a referenced data-declaration hash) as a length-prefixed blob.
    pub(crate) fn hash(&mut self, h: &ContentHash) {
        self.str(h.as_str());
    }

    /// Absorb a [`CtorRef`](crate::data::CtorRef): its declaration hash and constructor index. The
    /// constructor *name* is not identity-bearing (ADR-003) — only the `#T#i` pair is.
    pub(crate) fn ctor_ref(&mut self, c: &crate::data::CtorRef) {
        self.tag(tag::CTOR_REF);
        self.hash(c.decl());
        self.u32(c.index());
    }
}

impl Canon {
    fn scalar_kind(&mut self, k: ScalarKind) {
        // The scalar precision is semantically significant (it bounds embedding error) — part of
        // the type, hence identity-bearing (RFC-0001 §4.1).
        self.h.update(&[k.tag()]);
    }

    pub(crate) fn repr(&mut self, r: &Repr) {
        match r {
            Repr::Binary { width } => {
                self.tag(tag::REPR_BINARY);
                self.u32(*width);
            }
            Repr::Ternary { trits } => {
                self.tag(tag::REPR_TERNARY);
                self.u32(*trits);
            }
            Repr::Dense { dim, dtype } => {
                self.tag(tag::REPR_DENSE);
                self.u32(*dim);
                self.scalar_kind(*dtype);
            }
            Repr::Vsa {
                model,
                dim,
                sparsity,
            } => {
                self.tag(tag::REPR_VSA);
                self.str(model);
                self.u32(*dim);
                match sparsity {
                    SparsityClass::Dense => self.tag(tag::SPARSITY_DENSE),
                    SparsityClass::Sparse { max_active } => {
                        self.tag(tag::SPARSITY_SPARSE);
                        self.u32(*max_active);
                    }
                }
            }
        }
    }

    fn payload(&mut self, p: &Payload) {
        match p {
            Payload::Bits(bits) => {
                self.tag(tag::PAYLOAD_BITS);
                self.u64(bits.len() as u64);
                for &b in bits {
                    self.h.update(&[u8::from(b)]);
                }
            }
            Payload::Trits(trits) => {
                self.tag(tag::PAYLOAD_TRITS);
                self.u64(trits.len() as u64);
                for &t in trits {
                    let code: u8 = match t {
                        Trit::Neg => 0,
                        Trit::Zero => 1,
                        Trit::Pos => 2,
                    };
                    self.h.update(&[code]);
                }
            }
            Payload::Scalars(xs) => {
                self.tag(tag::PAYLOAD_SCALARS);
                self.u64(xs.len() as u64);
                for &x in xs {
                    self.f64(x);
                }
            }
            Payload::Hypervector(xs) => {
                self.tag(tag::PAYLOAD_HYPERVECTOR);
                self.u64(xs.len() as u64);
                for &x in xs {
                    self.f64(x);
                }
            }
        }
    }

    /// The identity-bearing part of a value: its `Repr` (type, incl. paradigm) and its literal
    /// payload. `Meta` is dynamic metadata and is deliberately excluded (RFC-0001 §4.6).
    pub(crate) fn value(&mut self, v: &Value) {
        self.repr(v.repr());
        self.payload(v.payload());
    }

    /// Encode a node under a binder scope (innermost binder last), α-renaming bound variables to
    /// de Bruijn indices so binder *names* never reach the hash.
    fn node(&mut self, n: &Node, scope: &mut Vec<VarId>) {
        match n {
            Node::Var(name) => {
                // Innermost-first search; a bound var becomes its de Bruijn index, a free var keeps
                // its name (a free name is part of an open term's contract, not a local detail).
                if let Some(pos) = scope.iter().rposition(|b| b == name) {
                    let de_bruijn = (scope.len() - 1 - pos) as u32;
                    self.tag(tag::VAR_BOUND);
                    self.u32(de_bruijn);
                } else {
                    self.tag(tag::VAR_FREE);
                    self.str(name);
                }
            }
            Node::Const(v) => {
                self.tag(tag::CONST);
                self.value(v);
            }
            Node::Let { id, bound, body } => {
                self.tag(tag::LET);
                // The bound expression is in the *outer* scope; the binder name itself is NOT hashed.
                self.node(bound, scope);
                scope.push(id.clone());
                self.node(body, scope);
                scope.pop();
            }
            Node::Op { prim, args } => {
                self.tag(tag::OP);
                self.str(prim); // the operator IS identity-bearing
                self.u64(args.len() as u64);
                for a in args {
                    self.node(a, scope);
                }
            }
            Node::Swap {
                src,
                target,
                policy,
            } => {
                self.tag(tag::SWAP);
                self.node(src, scope);
                self.repr(target); // the target type is part of the static contract
                self.str(policy.as_str()); // and so is the policy reference (RFC-0005)
            }
            Node::Construct { ctor, args } => {
                self.tag(tag::CONSTRUCT);
                self.ctor_ref(ctor); // the constructor identity (#T#i) is identity-bearing
                self.u64(args.len() as u64);
                for a in args {
                    self.node(a, scope);
                }
            }
            Node::Match {
                scrutinee,
                alts,
                default,
            } => {
                self.tag(tag::MATCH);
                self.node(scrutinee, scope);
                self.u64(alts.len() as u64);
                for alt in alts {
                    match alt {
                        Alt::Ctor {
                            ctor,
                            binders,
                            body,
                        } => {
                            self.tag(tag::ALT_CTOR);
                            self.ctor_ref(ctor);
                            // Binder *names* are not hashed (α-normalised); their count + positions
                            // are, via the de Bruijn scope the body is hashed under.
                            self.u64(binders.len() as u64);
                            let mark = scope.len();
                            scope.extend(binders.iter().cloned());
                            self.node(body, scope);
                            scope.truncate(mark);
                        }
                        Alt::Lit { value, body } => {
                            self.tag(tag::ALT_LIT);
                            self.value(value); // the literal is identity-bearing (repr + payload)
                            self.node(body, scope);
                        }
                    }
                }
                match default {
                    Some(d) => {
                        self.tag(tag::MATCH_DEFAULT);
                        self.node(d, scope);
                    }
                    None => self.tag(tag::MATCH_NO_DEFAULT),
                }
            }
            Node::Lam { param, body } => {
                // The param name is α-normalised (de Bruijn); identity is over the body structure.
                self.tag(tag::LAM);
                scope.push(param.clone());
                self.node(body, scope);
                scope.pop();
            }
            Node::App { func, arg } => {
                self.tag(tag::APP);
                self.node(func, scope);
                self.node(arg, scope);
            }
            Node::Fix { name, body } => {
                // The self-reference name is α-normalised (de Bruijn); the body sees it in scope.
                self.tag(tag::FIX);
                scope.push(name.clone());
                self.node(body, scope);
                scope.pop();
            }
        }
    }
}

impl Value {
    /// The content hash of this value's *identity-bearing* content: its [`Repr`] and payload, with
    /// all dynamic [`crate::meta::Meta`] excluded (RFC-0001 §4.6). Two values with identical
    /// repr+payload but different provenance/bounds collide; differing paradigm or literal does not.
    #[must_use]
    pub fn content_hash(&self) -> ContentHash {
        let mut c = Canon::new();
        c.value(self);
        c.finish()
    }
}

impl Node {
    /// The content hash of this definition (RFC-0001 §4.6; ADR-003). Identity is over the
    /// α-normalized structure, types-with-`Repr`, constant literals, operator names, and swap
    /// contracts — never over binder names or dynamic value metadata. Hence: trivial renames do not
    /// change the hash; identical definitions collide; a paradigm change does not.
    #[must_use]
    pub fn content_hash(&self) -> ContentHash {
        let mut c = Canon::new();
        let mut scope = Vec::new();
        c.node(self, &mut scope);
        c.finish()
    }
}

/// The content address of a *primitive operation* identified by its name — for the `op` field of a
/// [`crate::meta::Provenance::Derived`] record produced by the interpreter (M-110). Domain-separated
/// from node/value hashes so a prim name can never collide with a structural hash.
#[must_use]
pub fn operation_hash(prim: &str) -> ContentHash {
    let mut c = Canon::new();
    c.tag(0x07); // PRIM domain tag (distinct from the node/repr/payload tags above)
    c.str(prim);
    c.finish()
}

/// The separable `hash ↔ name` side-table (RFC-0001 §4.6, "names-as-metadata"). Names live *here*,
/// not in identity, so they can be attached, changed, or dropped without affecting a definition's
/// [`ContentHash`]. This is the kernel-side model of Unison's name store (ADR-003).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Names {
    map: HashMap<ContentHash, String>,
}

impl Names {
    /// An empty name table.
    #[must_use]
    pub fn new() -> Self {
        Names {
            map: HashMap::new(),
        }
    }

    /// Bind a human name to a content hash, returning any previous name for that hash. Re-binding a
    /// different name is allowed and changes nothing about identity (that is the whole point).
    pub fn bind(&mut self, hash: ContentHash, name: impl Into<String>) -> Option<String> {
        self.map.insert(hash, name.into())
    }

    /// The name bound to `hash`, if any.
    #[must_use]
    pub fn name_of(&self, hash: &ContentHash) -> Option<&str> {
        self.map.get(hash).map(String::as_str)
    }

    /// Number of bound names.
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Whether the table is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::Names;
    use crate::id::ContentHash;
    use crate::meta::{Meta, Provenance};
    use crate::node::Node;
    use crate::repr::{Repr, ScalarKind};
    use crate::value::{Payload, Value};

    fn byte(bits: [bool; 8]) -> Value {
        Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(bits.to_vec()),
            Meta::exact(Provenance::Root),
        )
        .expect("well-formed byte")
    }

    const B: [bool; 8] = [true, false, true, true, false, false, true, false];

    fn swap_def(binder: &str) -> Node {
        let policy = ContentHash::parse("blake3:round_trip_safe").expect("hash");
        Node::Let {
            id: binder.to_owned(),
            bound: Box::new(Node::Const(byte(B))),
            body: Box::new(Node::Swap {
                src: Box::new(Node::Var(binder.to_owned())),
                target: Repr::Ternary { trits: 6 },
                policy,
            }),
        }
    }

    #[test]
    fn hash_is_well_shaped_blake3() {
        let h = swap_def("a").content_hash();
        assert_eq!(h.algo(), "blake3");
        assert_eq!(h.digest().len(), 64); // BLAKE3 → 32 bytes → 64 hex chars
        assert!(ContentHash::parse(h.as_str()).is_some());
    }

    #[test]
    fn identical_defs_collide() {
        assert_eq!(swap_def("a").content_hash(), swap_def("a").content_hash());
    }

    #[test]
    fn trivial_renames_do_not_change_identity() {
        // Same structure, different binder name (and matching bound-var use) → α-equivalent.
        assert_eq!(
            swap_def("a").content_hash(),
            swap_def("longer_name").content_hash(),
            "α-renaming a binder must not change identity (RFC-0001 §4.6)"
        );
    }

    #[test]
    fn dynamic_metadata_is_not_hashed() {
        // Two constants with identical repr+payload but different provenance must collide.
        let exact = byte(B);
        let derived = Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(B.to_vec()),
            Meta::new(
                Provenance::Derived {
                    op: ContentHash::parse("blake3:some_op").unwrap(),
                    inputs: vec![],
                },
                crate::guarantee::GuaranteeStrength::Exact,
                None,
                None,
                None,
                None,
            )
            .expect("exact meta"),
        )
        .expect("well-formed");
        assert_eq!(exact.content_hash(), derived.content_hash());
        assert_eq!(
            Node::Const(exact).content_hash(),
            Node::Const(derived).content_hash()
        );
    }

    #[test]
    fn paradigm_change_changes_identity() {
        // A definition differing only in representation paradigm gets a different hash (§4.6).
        let bin = Node::Const(byte(B));
        let tern = Node::Const(
            Value::new(
                Repr::Ternary { trits: 6 },
                Payload::Trits(vec![crate::value::Trit::Zero; 6]),
                Meta::exact(Provenance::Root),
            )
            .expect("well-formed"),
        );
        assert_ne!(bin.content_hash(), tern.content_hash());
    }

    #[test]
    fn distinct_literals_differ() {
        let mut flipped = B;
        flipped[0] = !flipped[0];
        assert_ne!(
            Node::Const(byte(B)).content_hash(),
            Node::Const(byte(flipped)).content_hash()
        );
    }

    #[test]
    fn scalar_precision_is_identity_bearing() {
        // Dense{dim, F32} and Dense{dim, F64} are distinct types (precision bounds error).
        let f32v = Node::Const(
            Value::new(
                Repr::Dense {
                    dim: 2,
                    dtype: ScalarKind::F32,
                },
                Payload::Scalars(vec![1.0, 2.0]),
                Meta::exact(Provenance::Root),
            )
            .unwrap(),
        );
        let f64v = Node::Const(
            Value::new(
                Repr::Dense {
                    dim: 2,
                    dtype: ScalarKind::F64,
                },
                Payload::Scalars(vec![1.0, 2.0]),
                Meta::exact(Provenance::Root),
            )
            .unwrap(),
        );
        assert_ne!(f32v.content_hash(), f64v.content_hash());
    }

    #[test]
    fn op_operator_name_is_identity_bearing() {
        let add = Node::Op {
            prim: "add_binary".to_owned(),
            args: vec![Node::Const(byte(B))],
        };
        let sub = Node::Op {
            prim: "sub_binary".to_owned(),
            args: vec![Node::Const(byte(B))],
        };
        assert_ne!(add.content_hash(), sub.content_hash());
    }

    #[test]
    fn free_variables_keep_their_names() {
        // Distinct free names are distinct contracts → distinct identity (not α-renamable).
        assert_ne!(
            Node::Var("x".to_owned()).content_hash(),
            Node::Var("y".to_owned()).content_hash()
        );
    }

    #[test]
    fn names_are_metadata_outside_identity() {
        // The same definition can carry different human names; identity is unchanged.
        let h = swap_def("a").content_hash();
        let mut names = Names::new();
        assert!(names.is_empty());
        assert_eq!(names.bind(h.clone(), "to_ternary"), None);
        assert_eq!(names.name_of(&h), Some("to_ternary"));
        // Re-binding a new name does not (and cannot) change the hash.
        assert_eq!(
            names.bind(h.clone(), "as_balanced"),
            Some("to_ternary".into())
        );
        assert_eq!(names.name_of(&h), Some("as_balanced"));
        assert_eq!(swap_def("renamed_binder").content_hash(), h);
        assert_eq!(names.len(), 1);
    }
}
