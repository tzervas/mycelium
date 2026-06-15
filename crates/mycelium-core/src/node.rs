//! The Core IR node grammar (RFC-0001 §4.5).
//!
//! This commits the **core subset** of `SPECIFICATION.md` §10.2; the full term language
//! (abstraction, application, recursion, modules) is layered above this and is a later RFC.
//!
//! ```ebnf
//! Node ::= Const { value: Value }
//!        | Var   { id: VarId }
//!        | Let   { id: VarId, bound: Node, body: Node }
//!        | Op    { prim: Prim, args: [Node] }          (* paradigm-specific primitive *)
//!        | Swap  { src: Node, target: Repr, policy: PolicyRef }  (* the ONLY Repr-changing node *)
//!        | Construct { ctor: CtorRef, args: [Node] }            (* NEW (r3): saturated, W6 *)
//!        | Match { scrutinee: Node, alts: [Alt], default: Option<Node> } (* NEW (r3): flat, W7 *)
//! ```
//!
//! Well-formedness (RFC-0001 §4.5): **WF1** every change of a value's `Repr` is a [`Node::Swap`];
//! **WF2** every [`Node::Swap`] carries a [`PolicyRef`] — enforced *by construction* here, since
//! the `policy` field is mandatory. The r3 nodes carry **WF6** (`Construct` saturation), **WF7**
//! (flat, checked-exhaustive `Match`), and **WF8** (no `Swap` introduced through a `Match`/`Construct`
//! elaboration); WF6/WF7 coverage is *checked* above the kernel (the M-320 usefulness analysis +
//! the data registry [`crate::data::DataRegistry`]), never assumed here (RFC-0011 §4.3).

use crate::data::CtorRef;
use crate::id::ContentHash;
use crate::repr::Repr;
use crate::value::Value;

/// A variable identifier (a name; not part of content identity — RFC-0001 §4.6).
pub type VarId = String;
/// A primitive operator name; each declares its operand/result paradigms (RFC-0001 §4.5).
pub type Prim = String;
/// A reference to the selection policy a swap used (RFC-0005), as a content hash.
pub type PolicyRef = ContentHash;

/// A Core IR node.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// A constant value.
    Const(Value),
    /// A variable reference.
    Var(VarId),
    /// A let binding.
    Let {
        /// Bound name.
        id: VarId,
        /// The bound expression.
        bound: Box<Node>,
        /// The body in which `id` is in scope.
        body: Box<Node>,
    },
    /// A paradigm-specific primitive application.
    Op {
        /// The primitive.
        prim: Prim,
        /// Operands.
        args: Vec<Node>,
    },
    /// The only node that changes a value's representation; always carries a policy (WF1/WF2).
    Swap {
        /// The value being converted.
        src: Box<Node>,
        /// The target representation.
        target: Repr,
        /// The policy that chose/justified the swap.
        policy: PolicyRef,
    },
    /// A saturated constructor application (RFC-0011 §4.1, r3): builds a data value. SC-3-transparent
    /// (Repr-transparent — no `Swap`). `args.len()` must equal the constructor's field count (WF6);
    /// saturation is *checked* against the data registry above the kernel.
    Construct {
        /// The constructor (`#T#i`).
        ctor: CtorRef,
        /// The field expressions, in declaration order (saturated, WF6).
        args: Vec<Node>,
    },
    /// A **flat** pattern match (RFC-0011 §4.1, r3): one scrutinee, single-level constructor/literal
    /// alternatives, at most one default. Coverage is *checked* (WF7), never assumed; the Maranget
    /// decision tree that lowers nested surface patterns to this flat form stays an untrusted
    /// artifact above the kernel (RFC-0011 §4.4).
    Match {
        /// The value being scrutinised.
        scrutinee: Box<Node>,
        /// The alternatives, tried first-match, left-to-right.
        alts: Vec<Alt>,
        /// The catch-all branch, taken when no alternative matches.
        default: Option<Box<Node>>,
    },
}

/// One alternative of a flat [`Node::Match`] (RFC-0011 §4.1): a constructor arm (binding exactly the
/// constructor's arity) or a literal arm (over the non-enumerated `Binary{n}`/`Ternary{m}` domain).
#[derive(Debug, Clone, PartialEq)]
pub enum Alt {
    /// A constructor arm: matches a data value of constructor `ctor`, binding its fields to `binders`
    /// (exactly the constructor's arity — WF7), left-to-right.
    Ctor {
        /// The constructor matched (`#T#i`).
        ctor: CtorRef,
        /// The field binders, in declaration order (length == the constructor's arity).
        binders: Vec<VarId>,
        /// The arm body, in scope of `binders`.
        body: Node,
    },
    /// A literal arm: matches a representation value equal (repr + payload) to `value`. Because the
    /// `Binary{n}`/`Ternary{m}` domain is not enumerated, a `Match` carrying literal arms must also
    /// carry a `default` (WF7) — checked above the kernel.
    Lit {
        /// The literal value to match (a `Binary{n}`/`Ternary{m}` constant).
        value: Value,
        /// The arm body.
        body: Node,
    },
}

impl Node {
    /// Whether this node is the (only) representation-changing node, [`Node::Swap`] (WF1).
    #[must_use]
    pub fn is_repr_changing(&self) -> bool {
        matches!(self, Node::Swap { .. })
    }

    /// Whether this whole node is in the **AOT-lowerable** (representation-only) fragment — i.e. it
    /// contains no r3 data node (`Construct`/`Match`). The AOT/ANF path is repr-only in r3
    /// (RFC-0011 §4.4 Q5); a caller filters with this before [`crate::lower::lower_to_anf`], and the
    /// data fragment runs on the reference interpreter instead (the M-210 differential is L1-eval ≡
    /// L0-interp there). Honest: a `false` here is *why* a program is interpreter-only, not a gap.
    #[must_use]
    pub fn is_aot_lowerable(&self) -> bool {
        match self {
            Node::Const(_) | Node::Var(_) => true,
            Node::Let { bound, body, .. } => bound.is_aot_lowerable() && body.is_aot_lowerable(),
            Node::Op { args, .. } => args.iter().all(Node::is_aot_lowerable),
            Node::Swap { src, .. } => src.is_aot_lowerable(),
            Node::Construct { .. } | Node::Match { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::{Meta, Provenance};
    use crate::value::{Payload, Value};

    fn byte() -> Value {
        Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(vec![true, false, true, true, false, false, true, false]),
            Meta::exact(Provenance::Root),
        )
        .expect("well-formed byte")
    }

    #[test]
    fn builds_a_let_with_a_swap() {
        // let a = 0b1011_0010 in swap(a, to: Ternary{6}, policy: ...)
        let policy = ContentHash::parse("policy:round_trip_safe").expect("hash");
        let node = Node::Let {
            id: "a".to_owned(),
            bound: Box::new(Node::Const(byte())),
            body: Box::new(Node::Swap {
                src: Box::new(Node::Var("a".to_owned())),
                target: Repr::Ternary { trits: 6 },
                policy,
            }),
        };
        match node {
            Node::Let { body, .. } => assert!(body.is_repr_changing()),
            _ => panic!("expected a Let"),
        }
    }

    #[test]
    fn only_swap_changes_repr() {
        assert!(!Node::Var("x".to_owned()).is_repr_changing());
        assert!(!Node::Op {
            prim: "add_binary".to_owned(),
            args: vec![],
        }
        .is_repr_changing());
    }
}
