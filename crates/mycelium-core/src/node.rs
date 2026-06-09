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
//! ```
//!
//! Well-formedness (RFC-0001 §4.5): **WF1** every change of a value's `Repr` is a [`Node::Swap`];
//! **WF2** every [`Node::Swap`] carries a [`PolicyRef`] — enforced *by construction* here, since
//! the `policy` field is mandatory.

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
}

impl Node {
    /// Whether this node is the (only) representation-changing node, [`Node::Swap`] (WF1).
    #[must_use]
    pub fn is_repr_changing(&self) -> bool {
        matches!(self, Node::Swap { .. })
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
