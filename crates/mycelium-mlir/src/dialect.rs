//! Textual ternary-dialect emitter (M-150; RFC-0004 §6).
//!
//! Renders the lowered A-normal form (`mycelium-core::lower`) as an MLIR-*style* module — one
//! dialect op per binding, all attributes inline so nothing is opaque. This is a **textual**
//! artifact (no libMLIR binding); it stands in for the real `ternary` → `arith`/`vector` → LLVM
//! lowering, which is deferred.

use core::fmt::Write as _;

use mycelium_core::lower::{self, Anf, AnfAlt, Rhs};
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
///
/// The data + recursion fragment (`Construct`/`App`/`Lam`/`Fix`/`Match`, M-342) renders as dialect
/// ops too, with body-bearing ops (closures, match arms) carrying their nested block as a textual
/// **region** — a faithful skeleton, not a silent flatten. This is the dumpable shape of the eventual
/// MLIR path; the executable path for these nodes is the `aot::run` env-machine.
#[must_use]
pub fn emit(node: &Node) -> String {
    let anf = lower::lower_to_anf(node);
    let mut s = String::from("module {\n  func.func @kernel() -> !myc.value {\n");
    emit_block(&anf, 2, &mut s);
    s.push_str("  }\n}");
    s
}

/// Emit one ANF block's ops + terminator at indent `depth` (in 2-space units). Recursive: nested
/// regions (closure/recursion bodies, match arms) emit at a deeper indent.
fn emit_block(anf: &Anf, depth: usize, s: &mut String) {
    let pad = "  ".repeat(depth);
    for b in anf.bindings() {
        let name = b.name.render();
        let layout = b
            .layout
            .map(|l| format!("  // layout = {l:?}"))
            .unwrap_or_default();
        let _ = write!(s, "{pad}{name} = ");
        emit_op(&b.rhs, depth, s);
        let _ = writeln!(s, "{layout}");
    }
    let _ = writeln!(
        s,
        "{pad}\"func.return\"({}) : (!myc.value) -> ()",
        anf.result().render()
    );
}

/// Emit one lowered RHS as a dialect op (no trailing newline). Body-bearing ops embed nested regions.
fn emit_op(rhs: &Rhs, depth: usize, s: &mut String) {
    let pad = "  ".repeat(depth);
    match rhs {
        Rhs::Const(v) => {
            let _ = write!(
                s,
                "\"ternary.const\"() {{repr = \"{}\", value = \"{}\", guarantee = \"{:?}\"}} : () -> !myc.value",
                repr_attr(v.repr()),
                payload_attr(v.payload()),
                v.meta().guarantee()
            );
        }
        Rhs::Alias(a) => {
            let _ = write!(
                s,
                "\"ternary.alias\"({}) : (!myc.value) -> !myc.value",
                a.render()
            );
        }
        Rhs::Op { prim, args } => {
            let operands: Vec<String> = args
                .iter()
                .map(mycelium_core::lower::Atom::render)
                .collect();
            let _ = write!(
                s,
                "\"ternary.op\"({}) {{prim = \"{prim}\"}} : ({}) -> !myc.value",
                operands.join(", "),
                vec!["!myc.value"; operands.len()].join(", ")
            );
        }
        Rhs::Swap {
            src,
            target,
            policy,
        } => {
            let _ = write!(
                s,
                "\"ternary.swap\"({}) {{target = \"{}\", policy = \"{}\"}} : (!myc.value) -> !myc.value",
                src.render(),
                repr_attr(target),
                policy.as_str()
            );
        }
        Rhs::Construct { ctor, args } => {
            let operands: Vec<String> = args
                .iter()
                .map(mycelium_core::lower::Atom::render)
                .collect();
            let _ = write!(
                s,
                "\"myc.construct\"({}) {{ctor = \"{ctor}\"}} : ({}) -> !myc.value",
                operands.join(", "),
                vec!["!myc.value"; operands.len()].join(", ")
            );
        }
        Rhs::App { func, arg } => {
            let _ = write!(
                s,
                "\"myc.app\"({}, {}) : (!myc.value, !myc.value) -> !myc.value",
                func.render(),
                arg.render()
            );
        }
        Rhs::Lam { param, body } => {
            let _ = writeln!(s, "\"myc.lam\"() ({{  // param = \"{param}\"");
            emit_block(body, depth + 1, s);
            let _ = write!(s, "{pad}}}) : () -> !myc.value");
        }
        Rhs::Fix { name, body } => {
            let _ = writeln!(s, "\"myc.fix\"() ({{  // self = \"{name}\"");
            emit_block(body, depth + 1, s);
            let _ = write!(s, "{pad}}}) : () -> !myc.value");
        }
        Rhs::Match {
            scrutinee,
            alts,
            default,
        } => {
            let _ = writeln!(s, "\"myc.match\"({}) (", scrutinee.render());
            for alt in alts {
                match alt {
                    AnfAlt::Ctor {
                        ctor,
                        binders,
                        body,
                    } => {
                        let _ = writeln!(s, "{pad}  {{  // alt {ctor} ({})", binders.join(" "));
                        emit_block(body, depth + 2, s);
                        let _ = writeln!(s, "{pad}  }},");
                    }
                    AnfAlt::Lit { value, body } => {
                        let _ =
                            writeln!(s, "{pad}  {{  // alt-lit {}", payload_attr(value.payload()));
                        emit_block(body, depth + 2, s);
                        let _ = writeln!(s, "{pad}  }},");
                    }
                }
            }
            match default {
                Some(d) => {
                    let _ = writeln!(s, "{pad}  {{  // default");
                    emit_block(d, depth + 2, s);
                    let _ = writeln!(s, "{pad}  }}");
                }
                None => {
                    let _ = writeln!(s, "{pad}  // no-default");
                }
            }
            let _ = write!(s, "{pad}) : (!myc.value) -> !myc.value");
        }
    }
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
