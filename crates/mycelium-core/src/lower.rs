//! Inspectable lowering — `≥2` dumpable/diffable stages (M-112; RFC-0004 §5/§6; SC-4; WF5).
//!
//! The interpreter is the reference semantics (M-110); *lowering* is the backend-agnostic path
//! toward codegen, and its defining property here is **inspectability**: every stage has a
//! canonical textual [`dump`](Stage::text) (deterministic — structurally identical programs render
//! identically, SC-4), each pass **preserves `Meta`** (WF5 — guarantee tags survive, never silently
//! dropped), and the packing decision is an **explicit, recorded** schedule choice (RFC-0004 §5; no
//! hidden layout). [`stages`] returns the pipeline so adjacent stages can be diffed.
//!
//! The two stages shipped:
//! - **`core`** — the Core IR node tree (RFC-0001 §4.5), rendered canonically.
//! - **`substrate`** — an **A-normal form**: nested `Op`/`Swap`/`Let` flattened to a linear list of
//!   named bindings (the classic pre-codegen shape every backend consumes), with each binding whose
//!   result representation is *statically known* (a `Const` or a `Swap` target) annotated with its
//!   **scheduled [`PhysicalLayout`]** (the default schedule, RFC-0004 §5 / DN-01).
//!
//! Layout for `Op` results is intentionally left unannotated: the kernel has no operator-typing yet
//! (a later RFC), so inferring it would be a black box (G2). The omission is explicit, not silent.

use core::fmt::Write as _;

use crate::meta::PackScheme;
use crate::node::{Alt, Node};
use crate::repr::{Repr, ScalarKind, SparsityClass};
use crate::value::{Payload, Trit, Value};
use crate::{GuaranteeStrength, PhysicalLayout};

/// One lowering stage: a name and its canonical, diffable textual dump.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage {
    /// Stage name (`"core"`, `"substrate"`).
    pub name: &'static str,
    /// The canonical textual rendering (SC-4: deterministic; structurally identical → identical).
    pub text: String,
}

/// The default schedule-staged packing for a representation (RFC-0004 §5; DN-01). The fixed,
/// enumerable layout set keeps selection tractable (T1.4) — `I2_S` is the lossless ternary default.
#[must_use]
pub fn schedule(repr: &Repr) -> PhysicalLayout {
    match repr {
        Repr::Binary { .. } => PhysicalLayout::BinaryWords,
        Repr::Ternary { .. } => PhysicalLayout::TritPacked {
            scheme: PackScheme::I2S,
        },
        Repr::Dense { .. } => PhysicalLayout::DenseArray,
        Repr::Vsa { sparsity, .. } => PhysicalLayout::VsaStore {
            sparse: matches!(sparsity, SparsityClass::Sparse { .. }),
        },
    }
}

/// Run the lowering pipeline, returning every stage in order (currently `core` → `substrate`).
#[must_use]
pub fn stages(node: &Node) -> Vec<Stage> {
    vec![
        Stage {
            name: "core",
            text: dump_core(node),
        },
        Stage {
            name: "substrate",
            text: lower_to_anf(node).dump(),
        },
    ]
}

// --- rendering helpers (shared, canonical) -----------------------------------------------------

fn render_scalar_kind(k: ScalarKind) -> &'static str {
    match k {
        ScalarKind::F16 => "F16",
        ScalarKind::Bf16 => "BF16",
        ScalarKind::F32 => "F32",
        ScalarKind::F64 => "F64",
    }
}

fn render_repr(repr: &Repr) -> String {
    match repr {
        Repr::Binary { width } => format!("Binary{{{width}}}"),
        Repr::Ternary { trits } => format!("Ternary{{{trits}}}"),
        Repr::Dense { dim, dtype } => format!("Dense{{{dim}:{}}}", render_scalar_kind(*dtype)),
        Repr::Vsa {
            model,
            dim,
            sparsity,
        } => {
            let s = match sparsity {
                SparsityClass::Dense => "dense".to_owned(),
                SparsityClass::Sparse { max_active } => format!("sparse≤{max_active}"),
            };
            format!("VSA{{{model}:{dim} {s}}}")
        }
    }
}

fn render_payload(p: &Payload) -> String {
    match p {
        Payload::Bits(b) => {
            let s: String = b.iter().map(|&x| if x { '1' } else { '0' }).collect();
            format!("bits={s}")
        }
        Payload::Trits(t) => {
            let s: String = t
                .iter()
                .map(|&x| match x {
                    Trit::Neg => '-',
                    Trit::Zero => '0',
                    Trit::Pos => '+',
                })
                .collect();
            format!("trits={s}")
        }
        Payload::Scalars(xs) => format!("scalars={xs:?}"),
        Payload::Hypervector(xs) => format!("hv={xs:?}"),
    }
}

fn render_guarantee(g: GuaranteeStrength) -> &'static str {
    match g {
        GuaranteeStrength::Exact => ":exact",
        GuaranteeStrength::Proven => ":proven",
        GuaranteeStrength::Empirical => ":empirical",
        GuaranteeStrength::Declared => ":declared",
    }
}

/// Render a `Const` value head: `const <repr> <payload> <guarantee>` (Meta-preserving — the
/// guarantee tag is always shown, WF5).
fn render_const(v: &Value) -> String {
    format!(
        "const {} {} {}",
        render_repr(v.repr()),
        render_payload(v.payload()),
        render_guarantee(v.meta().guarantee())
    )
}

fn short_hash(h: &crate::ContentHash) -> String {
    let d = h.digest();
    let head: String = d.chars().take(8).collect();
    format!("{}:{head}", h.algo())
}

// --- Stage 0: canonical Core IR dump -----------------------------------------------------------

/// The canonical, deterministic textual rendering of a Core IR node (the `core` stage). A
/// projection: it does not affect content identity (RFC-0001 §4.6/§4.8), and structurally identical
/// nodes render identically (SC-4). Reused as the basis of the formatter (M-142).
#[must_use]
pub fn dump_node(node: &Node) -> String {
    dump_core(node)
}

fn dump_core(node: &Node) -> String {
    let mut s = String::new();
    write_core(node, 0, &mut s);
    s
}

/// The **canonical formatter** (M-142; RFC-0001 §4.8; ADR-003). Like [`dump_node`] but with binder
/// names **α-normalized** to `v0, v1, …` in binding order, so that definitions differing only in
/// names (a "reformatting") render to *identical* canonical text — and, since names are not part of
/// content identity (RFC-0001 §4.6), that shared canonical form carries one shared
/// [`Node::content_hash`]. Formatting is a projection: it never changes identity.
#[must_use]
pub fn format(node: &Node) -> String {
    let mut s = String::new();
    let mut scope: Vec<(String, String)> = Vec::new();
    let mut counter = 0usize;
    write_canon(node, 0, &mut scope, &mut counter, &mut s);
    s
}

fn write_canon(
    node: &Node,
    depth: usize,
    scope: &mut Vec<(String, String)>,
    counter: &mut usize,
    s: &mut String,
) {
    indent(depth, s);
    match node {
        Node::Const(v) => {
            let _ = writeln!(s, "{}", render_const(v));
        }
        Node::Var(x) => {
            // Innermost-first; a bound var renders as its canonical name, a free var keeps its own.
            match scope.iter().rev().find(|(orig, _)| orig == x) {
                Some((_, canon)) => {
                    let _ = writeln!(s, "var {canon}");
                }
                None => {
                    let _ = writeln!(s, "free {x}");
                }
            }
        }
        Node::Let { id, bound, body } => {
            let canon = format!("v{counter}");
            *counter += 1;
            let _ = writeln!(s, "let {canon} =");
            write_canon(bound, depth + 1, scope, counter, s);
            indent(depth, s);
            let _ = writeln!(s, "in");
            scope.push((id.clone(), canon));
            write_canon(body, depth + 1, scope, counter, s);
            scope.pop();
        }
        Node::Op { prim, args } => {
            let _ = writeln!(s, "op {prim}");
            for a in args {
                write_canon(a, depth + 1, scope, counter, s);
            }
        }
        Node::Swap {
            src,
            target,
            policy,
        } => {
            let _ = writeln!(s, "swap -> {} @{}", render_repr(target), short_hash(policy));
            write_canon(src, depth + 1, scope, counter, s);
        }
        Node::Construct { ctor, args } => {
            let _ = writeln!(s, "construct {ctor}");
            for a in args {
                write_canon(a, depth + 1, scope, counter, s);
            }
        }
        Node::Match {
            scrutinee,
            alts,
            default,
        } => {
            let _ = writeln!(s, "match");
            write_canon(scrutinee, depth + 1, scope, counter, s);
            for alt in alts {
                indent(depth, s);
                match alt {
                    Alt::Ctor {
                        ctor,
                        binders,
                        body,
                    } => {
                        // α-normalize the binder names to v0, v1, … in binding order (the canonical
                        // dump never leaks source names — §4.8).
                        let canon: Vec<String> = (0..binders.len())
                            .map(|_| {
                                let c = format!("v{counter}");
                                *counter += 1;
                                c
                            })
                            .collect();
                        let _ = writeln!(s, "alt {ctor} ({})", canon.join(" "));
                        let mark = scope.len();
                        for (orig, c) in binders.iter().zip(&canon) {
                            scope.push((orig.clone(), c.clone()));
                        }
                        write_canon(body, depth + 1, scope, counter, s);
                        scope.truncate(mark);
                    }
                    Alt::Lit { value, body } => {
                        let _ = writeln!(s, "alt-lit {}", render_const(value));
                        write_canon(body, depth + 1, scope, counter, s);
                    }
                }
            }
            indent(depth, s);
            match default {
                Some(d) => {
                    let _ = writeln!(s, "default");
                    write_canon(d, depth + 1, scope, counter, s);
                }
                None => {
                    let _ = writeln!(s, "no-default");
                }
            }
        }
    }
}

fn indent(depth: usize, s: &mut String) {
    for _ in 0..depth {
        s.push_str("  ");
    }
}

fn write_core(node: &Node, depth: usize, s: &mut String) {
    indent(depth, s);
    match node {
        Node::Const(v) => {
            let _ = writeln!(s, "{}", render_const(v));
        }
        Node::Var(x) => {
            let _ = writeln!(s, "var {x}");
        }
        Node::Let { id, bound, body } => {
            let _ = writeln!(s, "let {id} =");
            write_core(bound, depth + 1, s);
            indent(depth, s);
            let _ = writeln!(s, "in");
            write_core(body, depth + 1, s);
        }
        Node::Op { prim, args } => {
            let _ = writeln!(s, "op {prim}");
            for a in args {
                write_core(a, depth + 1, s);
            }
        }
        Node::Swap {
            src,
            target,
            policy,
        } => {
            let _ = writeln!(s, "swap -> {} @{}", render_repr(target), short_hash(policy));
            write_core(src, depth + 1, s);
        }
        Node::Construct { ctor, args } => {
            let _ = writeln!(s, "construct {ctor}");
            for a in args {
                write_core(a, depth + 1, s);
            }
        }
        Node::Match {
            scrutinee,
            alts,
            default,
        } => {
            let _ = writeln!(s, "match");
            write_core(scrutinee, depth + 1, s);
            for alt in alts {
                indent(depth, s);
                match alt {
                    Alt::Ctor {
                        ctor,
                        binders,
                        body,
                    } => {
                        let _ = writeln!(s, "alt {ctor} ({})", binders.join(" "));
                        write_core(body, depth + 1, s);
                    }
                    Alt::Lit { value, body } => {
                        let _ = writeln!(s, "alt-lit {}", render_const(value));
                        write_core(body, depth + 1, s);
                    }
                }
            }
            indent(depth, s);
            match default {
                Some(d) => {
                    let _ = writeln!(s, "default");
                    write_core(d, depth + 1, s);
                }
                None => {
                    let _ = writeln!(s, "no-default");
                }
            }
        }
    }
}

// --- Stage 1: A-normal-form "substrate" --------------------------------------------------------

/// An operand of a lowered binding: a reference to a named/temp binding. (Public so backends — the
/// MLIR emitter / AOT path, M-150 — can consume the lowered IR.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Atom {
    /// A source `let`-bound name.
    Named(String),
    /// An introduced temporary, `%k`.
    Temp(usize),
}

impl Atom {
    /// The canonical textual rendering of this operand (`name` or `%k`).
    #[must_use]
    pub fn render(&self) -> String {
        match self {
            Atom::Named(x) => x.clone(),
            Atom::Temp(k) => format!("%{k}"),
        }
    }
}

/// The right-hand side of a lowered binding.
#[derive(Debug, Clone, PartialEq)]
pub enum Rhs {
    /// A constant value (carries its `Meta`, WF5).
    Const(Value),
    /// An alias to another binding (from a source `let`).
    Alias(Atom),
    /// A primitive application over atoms.
    Op {
        /// The primitive name.
        prim: String,
        /// Operand atoms.
        args: Vec<Atom>,
    },
    /// The representation-changing swap (carries its target and policy, WF1/WF2).
    Swap {
        /// The value being converted.
        src: Atom,
        /// The target representation.
        target: Repr,
        /// The selection policy reference (RFC-0005).
        policy: crate::ContentHash,
    },
}

/// One lowered binding: a name, its right-hand side, and (where statically known) its scheduled
/// physical layout.
#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    /// The binding's name.
    pub name: Atom,
    /// Its right-hand side.
    pub rhs: Rhs,
    /// The scheduled packing, when the result repr is statically known (RFC-0004 §5).
    pub layout: Option<PhysicalLayout>,
}

/// A flattened (A-normal-form) lowering of a Core IR node.
#[derive(Debug, Clone)]
pub struct Anf {
    bindings: Vec<Binding>,
    result: Atom,
}

/// Lower a Core IR node into A-normal form (flatten nested `Op`/`Swap`/`Let` to a linear binding
/// list). Pure and deterministic; `Meta` rides along on `Const` bindings (WF5).
///
/// **Repr-only (r3, RFC-0011 §4.4 Q5).** The ANF substrate / AOT path covers the representation
/// fragment; the r3 data-and-matching nodes (`Construct`/`Match`) are **interpreter-first** and have
/// no ANF lowering yet. They never reach here: the M-210 differential runs the data fragment as
/// *L1-eval ≡ L0-interp* (AOT "where reachable"), and [`crate::Node::is_aot_lowerable`] lets a
/// caller filter them out explicitly before lowering. Reaching the data arms below is therefore an
/// upstream-contract violation (a loud panic, never a silent mis-lowering).
#[must_use]
pub fn lower_to_anf(node: &Node) -> Anf {
    let mut b = Vec::new();
    let mut next = 0usize;
    let result = flatten(node, &mut b, &mut next);
    Anf {
        bindings: b,
        result,
    }
}

fn fresh(next: &mut usize) -> usize {
    let k = *next;
    *next += 1;
    k
}

fn flatten(node: &Node, out: &mut Vec<Binding>, next: &mut usize) -> Atom {
    match node {
        Node::Var(x) => Atom::Named(x.clone()),
        Node::Const(v) => {
            let name = Atom::Temp(fresh(next));
            out.push(Binding {
                name: name.clone(),
                rhs: Rhs::Const(v.clone()),
                layout: Some(schedule(v.repr())),
            });
            name
        }
        Node::Let { id, bound, body } => {
            let ba = flatten(bound, out, next);
            out.push(Binding {
                name: Atom::Named(id.clone()),
                rhs: Rhs::Alias(ba),
                layout: None,
            });
            flatten(body, out, next)
        }
        Node::Op { prim, args } => {
            let atoms: Vec<Atom> = args.iter().map(|a| flatten(a, out, next)).collect();
            let name = Atom::Temp(fresh(next));
            out.push(Binding {
                name: name.clone(),
                rhs: Rhs::Op {
                    prim: prim.clone(),
                    args: atoms,
                },
                layout: None, // Op result repr is not statically known (no operator typing yet).
            });
            name
        }
        Node::Swap {
            src,
            target,
            policy,
        } => {
            let sa = flatten(src, out, next);
            let name = Atom::Temp(fresh(next));
            out.push(Binding {
                name: name.clone(),
                rhs: Rhs::Swap {
                    src: sa,
                    target: target.clone(),
                    policy: policy.clone(),
                },
                layout: Some(schedule(target)),
            });
            name
        }
        // r3 (RFC-0011 §4.4 Q5): the data nodes are interpreter-first and have no ANF lowering. A
        // caller must filter them out (Node::is_aot_lowerable) before lowering; reaching here is an
        // upstream-contract break, surfaced loudly rather than mis-lowered silently (never-silent).
        Node::Construct { .. } | Node::Match { .. } => unreachable!(
            "Construct/Match have no AOT/ANF lowering in r3 (RFC-0011 Q5); they run on the \
             reference interpreter — filter with Node::is_aot_lowerable before lowering"
        ),
    }
}

fn render_layout(l: PhysicalLayout) -> String {
    match l {
        PhysicalLayout::BinaryWords => "BinaryWords".to_owned(),
        PhysicalLayout::TritPacked { scheme } => format!("TritPacked({scheme:?})"),
        PhysicalLayout::DenseArray => "DenseArray".to_owned(),
        PhysicalLayout::VsaStore { sparse } => format!("VsaStore(sparse={sparse})"),
    }
}

impl Anf {
    /// The canonical, diffable dump of the substrate stage (SC-4).
    #[must_use]
    pub fn dump(&self) -> String {
        let mut s = String::from("substrate {\n");
        for b in &self.bindings {
            let rhs = match &b.rhs {
                Rhs::Const(v) => render_const(v),
                Rhs::Alias(a) => a.render(),
                Rhs::Op { prim, args } => {
                    let a: Vec<String> = args.iter().map(Atom::render).collect();
                    format!("op {prim} {}", a.join(" "))
                }
                Rhs::Swap {
                    src,
                    target,
                    policy,
                } => {
                    format!(
                        "swap {} -> {} @{}",
                        src.render(),
                        render_repr(target),
                        short_hash(policy)
                    )
                }
            };
            let _ = write!(s, "  {} = {rhs}", b.name.render());
            if let Some(l) = b.layout {
                let _ = write!(s, "    ; layout={}", render_layout(l));
            }
            s.push('\n');
        }
        let _ = writeln!(s, "  result {}", self.result.render());
        s.push('}');
        s
    }

    /// Number of bindings (for tests/tooling).
    #[must_use]
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Whether there are no bindings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// The ordered bindings (for backends consuming the lowered IR — M-150).
    #[must_use]
    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    /// The result operand.
    #[must_use]
    pub fn result(&self) -> &Atom {
        &self.result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::{Meta, Provenance};
    use crate::value::Payload;
    use crate::ContentHash;

    fn byte() -> Value {
        Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(vec![true, false, true, true, false, false, true, false]),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    /// `let a = byte in swap(a -> Ternary{6})` — exercises Let + Swap + Var.
    fn sample() -> Node {
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
    fn pipeline_has_at_least_two_named_stages() {
        let st = stages(&sample());
        assert!(st.len() >= 2);
        assert_eq!(st[0].name, "core");
        assert_eq!(st[1].name, "substrate");
        // Diffable: the two stages render differently.
        assert_ne!(st[0].text, st[1].text);
    }

    #[test]
    fn dump_is_deterministic_and_structural() {
        // SC-4: structurally identical nodes render identically at every stage.
        let a = stages(&sample());
        let b = stages(&sample());
        assert_eq!(a, b);
    }

    #[test]
    fn substrate_is_flat_and_schedules_known_layouts() {
        let anf = lower_to_anf(&sample());
        let dump = anf.dump();
        // The const byte is scheduled BinaryWords; the swap target Ternary is scheduled TritPacked.
        assert!(dump.contains("layout=BinaryWords"), "{dump}");
        assert!(dump.contains("TritPacked(I2S)"), "{dump}");
        assert!(dump.contains("result"));
        // Flattened: a swap binding references an atom, not a nested tree.
        assert!(
            dump.contains("swap a -> Ternary{6}") || dump.contains("swap %"),
            "{dump}"
        );
    }

    #[test]
    fn meta_guarantee_survives_lowering() {
        // WF5: a non-Exact const keeps its tag in both stages.
        let proven = Value::new(
            Repr::Vsa {
                model: "MAP-I".into(),
                dim: 4,
                sparsity: SparsityClass::Dense,
            },
            Payload::Hypervector(vec![1.0, 0.0, 0.0, -1.0]),
            Meta::new(
                Provenance::Root,
                GuaranteeStrength::Proven,
                Some(crate::Bound {
                    kind: crate::BoundKind::Capacity { items: 2, dim: 4 },
                    basis: crate::BoundBasis::ProvenThm {
                        citation: "x".into(),
                    },
                }),
                None,
                None,
                None,
            )
            .unwrap(),
        )
        .unwrap();
        let node = Node::Const(proven);
        let st = stages(&node);
        assert!(st[0].text.contains(":proven"));
        assert!(st[1].text.contains(":proven"));
    }

    #[test]
    fn nested_ops_flatten_to_temporaries() {
        // op f (op g c) c  →  %0 = c ; %1 = op g %0 ; %2 = op f %1 %0 ; result %2  (positional temps)
        let c = Node::Const(byte());
        let node = Node::Op {
            prim: "f".into(),
            args: vec![
                Node::Op {
                    prim: "g".into(),
                    args: vec![c.clone()],
                },
                c,
            ],
        };
        let dump = lower_to_anf(&node).dump();
        assert!(dump.contains("op g"));
        assert!(dump.contains("op f"));
        assert!(dump.contains("%0"));
    }
}
