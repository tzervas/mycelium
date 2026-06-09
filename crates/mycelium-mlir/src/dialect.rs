//! Textual ternary-dialect emitter (M-150; RFC-0004 §6).
//!
//! Renders the lowered A-normal form (`mycelium-core::lower`) as an MLIR-*style* module — one
//! dialect op per binding, all attributes inline so nothing is opaque. This is a **textual**
//! artifact (no libMLIR binding); it stands in for the real `ternary` → `arith`/`vector` → LLVM
//! lowering, which is deferred.

use core::fmt::Write as _;

use mycelium_core::lower::{self, Rhs};
use mycelium_core::{Node, Payload, Repr, Trit};

fn repr_attr(repr: &Repr) -> String {
    match repr {
        Repr::Binary { width } => format!("binary<{width}>"),
        Repr::Ternary { trits } => format!("ternary<{trits}>"),
        Repr::Dense { dim, .. } => format!("dense<{dim}>"),
        Repr::Vsa { model, dim, .. } => format!("vsa<{model},{dim}>"),
    }
}

fn payload_attr(p: &Payload) -> String {
    match p {
        Payload::Bits(b) => b.iter().map(|&x| if x { '1' } else { '0' }).collect(),
        Payload::Trits(t) => t
            .iter()
            .map(|&x| match x {
                Trit::Neg => '-',
                Trit::Zero => '0',
                Trit::Pos => '+',
            })
            .collect(),
        Payload::Scalars(xs) => format!("{xs:?}"),
        Payload::Hypervector(xs) => format!("{xs:?}"),
    }
}

/// Emit the textual `ternary`-dialect module for `node` (one op per lowered binding). Deterministic.
#[must_use]
pub fn emit(node: &Node) -> String {
    let anf = lower::lower_to_anf(node);
    let mut s = String::from("module {\n  func.func @kernel() -> !myc.value {\n");
    for b in anf.bindings() {
        let name = b.name.render();
        let line = match &b.rhs {
            Rhs::Const(v) => format!(
                "\"ternary.const\"() {{repr = \"{}\", value = \"{}\", guarantee = \"{:?}\"}} : () -> !myc.value",
                repr_attr(v.repr()),
                payload_attr(v.payload()),
                v.meta().guarantee()
            ),
            Rhs::Alias(a) => {
                format!("\"ternary.alias\"({}) : (!myc.value) -> !myc.value", a.render())
            }
            Rhs::Op { prim, args } => {
                let operands: Vec<String> = args.iter().map(mycelium_core::lower::Atom::render).collect();
                format!(
                    "\"ternary.op\"({}) {{prim = \"{prim}\"}} : ({}) -> !myc.value",
                    operands.join(", "),
                    vec!["!myc.value"; operands.len()].join(", ")
                )
            }
            Rhs::Swap {
                src,
                target,
                policy,
            } => format!(
                "\"ternary.swap\"({}) {{target = \"{}\", policy = \"{}\"}} : (!myc.value) -> !myc.value",
                src.render(),
                repr_attr(target),
                policy.as_str()
            ),
        };
        let layout = b
            .layout
            .map(|l| format!("  // layout = {l:?}"))
            .unwrap_or_default();
        let _ = writeln!(s, "    {name} = {line}{layout}");
    }
    let _ = writeln!(
        s,
        "    \"func.return\"({}) : (!myc.value) -> ()",
        anf.result().render()
    );
    s.push_str("  }\n}");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_core::{ContentHash, Meta, Provenance, Value};

    fn byte() -> Value {
        Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(vec![true, false, true, true, false, false, true, false]),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    fn program() -> Node {
        Node::Let {
            id: "a".into(),
            bound: Box::new(Node::Const(byte())),
            body: Box::new(Node::Swap {
                src: Box::new(Node::Var("a".into())),
                target: Repr::Ternary { trits: 6 },
                policy: ContentHash::parse("blake3:round_trip_safe").unwrap(),
            }),
        }
    }

    #[test]
    fn emits_a_module_with_one_op_per_binding() {
        let m = emit(&program());
        assert!(m.starts_with("module {"));
        assert!(m.contains("func.func @kernel"));
        assert!(m.contains("\"ternary.const\""));
        assert!(m.contains("\"ternary.swap\""));
        assert!(m.contains("func.return"));
        // target + policy attributes are present (no opaque pass).
        assert!(m.contains("target = \"ternary<6>\""));
        assert!(m.contains("policy = \"blake3:round_trip_safe\""));
    }

    #[test]
    fn emission_is_deterministic() {
        assert_eq!(emit(&program()), emit(&program()));
    }
}
