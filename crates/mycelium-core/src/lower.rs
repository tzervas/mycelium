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

use crate::data::CtorRef;
use crate::meta::PackScheme;
use crate::node::{Alt, Node, VarId};
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
        Node::Lam { param, body } => {
            let canon = format!("v{counter}");
            *counter += 1;
            let _ = writeln!(s, "lam {canon} =>");
            scope.push((param.clone(), canon));
            write_canon(body, depth + 1, scope, counter, s);
            scope.pop();
        }
        Node::App { func, arg } => {
            let _ = writeln!(s, "app");
            write_canon(func, depth + 1, scope, counter, s);
            write_canon(arg, depth + 1, scope, counter, s);
        }
        Node::Fix { name, body } => {
            let canon = format!("v{counter}");
            *counter += 1;
            let _ = writeln!(s, "fix {canon} =>");
            scope.push((name.clone(), canon));
            write_canon(body, depth + 1, scope, counter, s);
            scope.pop();
        }
        Node::FixGroup { defs, body } => {
            let _ = writeln!(s, "fixgroup");
            // α-normalise every member name first — the group binds them all mutually, so each is in
            // scope for every definition and the continuation (the canonical dump never leaks names).
            let mark = scope.len();
            for (name, _) in defs {
                let canon = format!("v{counter}");
                *counter += 1;
                scope.push((name.clone(), canon));
            }
            for (i, (_, def)) in defs.iter().enumerate() {
                let canon = scope[mark + i].1.clone();
                indent(depth + 1, s);
                let _ = writeln!(s, "def {canon} =>");
                write_canon(def, depth + 2, scope, counter, s);
            }
            indent(depth + 1, s);
            let _ = writeln!(s, "in");
            write_canon(body, depth + 1, scope, counter, s);
            scope.truncate(mark);
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
        Node::Lam { param, body } => {
            let _ = writeln!(s, "lam {param} =>");
            write_core(body, depth + 1, s);
        }
        Node::App { func, arg } => {
            let _ = writeln!(s, "app");
            write_core(func, depth + 1, s);
            write_core(arg, depth + 1, s);
        }
        Node::Fix { name, body } => {
            let _ = writeln!(s, "fix {name} =>");
            write_core(body, depth + 1, s);
        }
        Node::FixGroup { defs, body } => {
            let _ = writeln!(s, "fixgroup");
            for (name, def) in defs {
                indent(depth + 1, s);
                let _ = writeln!(s, "def {name} =>");
                write_core(def, depth + 2, s);
            }
            indent(depth + 1, s);
            let _ = writeln!(s, "in");
            write_core(body, depth + 1, s);
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
    /// A saturated constructor application (RFC-0011 §4.1): builds a data value from field atoms.
    Construct {
        /// The constructor (`#T#i`).
        ctor: CtorRef,
        /// The field operands, in declaration order (saturated, WF6).
        args: Vec<Atom>,
    },
    /// Application of a function atom to an argument atom (RFC-0001 r4; call-by-value).
    App {
        /// The function operand (resolves to a closure).
        func: Atom,
        /// The argument operand.
        arg: Atom,
    },
    /// A lambda abstraction (RFC-0001 r4) — a **closure** value. Its body is a **nested** ANF block
    /// evaluated only on application (lazily), so the linear binding list stays acyclic.
    Lam {
        /// The bound parameter (a `Named` atom inside `body`).
        param: VarId,
        /// The body, lowered to a nested block (shares the program-wide temp counter, so its temps
        /// never collide with the enclosing scope).
        body: Anf,
    },
    /// General recursion (RFC-0001 r4) — its body (typically a [`Rhs::Lam`]) is a nested ANF block;
    /// the env-machine unfolds it under a fuel clock on application.
    Fix {
        /// The self-reference name bound in `body`.
        name: VarId,
        /// The recursive body, lowered to a nested block.
        body: Anf,
    },
    /// One member of a **mutual-recursion group** (RFC-0001 r5; [`Node::FixGroup`]). Lowering emits
    /// one such binding per member, each carrying the whole group's lowered definitions (`defs`) plus
    /// `which` member it is; the env-machine binds it to a suspension that, on application, re-binds
    /// every member name to its own focus suspension (so siblings can call each other) and enters
    /// `which`'s body — the env analogue of the interpreter's focus unfold, under the fuel clock.
    FixGroup {
        /// All members of the group `(name, lowered definition)` — shared by every member binding so
        /// each can resolve its siblings on unfold.
        defs: Vec<(VarId, Anf)>,
        /// Which member name this binding resolves to.
        which: VarId,
    },
    /// A flat pattern match (RFC-0011 §4.1): a scrutinee atom, single-level alternatives whose bodies
    /// are **nested** ANF blocks (evaluated only when selected), and at most one default block.
    Match {
        /// The scrutinised operand.
        scrutinee: Atom,
        /// The alternatives, tried first-match, left-to-right.
        alts: Vec<AnfAlt>,
        /// The catch-all block, taken when no alternative matches.
        default: Option<Anf>,
    },
}

/// One alternative of a lowered [`Rhs::Match`] — the ANF analogue of [`crate::node::Alt`], with the
/// arm body lowered to a nested block.
#[derive(Debug, Clone, PartialEq)]
pub enum AnfAlt {
    /// A constructor arm: matches a data value of `ctor`, binding its fields to `binders`
    /// (left-to-right, exactly the constructor's arity — WF7).
    Ctor {
        /// The constructor matched (`#T#i`).
        ctor: CtorRef,
        /// The field binders (`Named` atoms inside `body`).
        binders: Vec<VarId>,
        /// The arm body, lowered to a nested block (in scope of `binders`).
        body: Anf,
    },
    /// A literal arm: matches a representation value equal (repr + payload) to `value`.
    Lit {
        /// The literal value to match.
        value: Value,
        /// The arm body, lowered to a nested block.
        body: Anf,
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
#[derive(Debug, Clone, PartialEq)]
pub struct Anf {
    bindings: Vec<Binding>,
    result: Atom,
}

/// Lower a Core IR node into A-normal form (flatten nested nodes to a linear binding list). Pure and
/// deterministic; `Meta` rides along on `Const` bindings (WF5).
///
/// **Full v0 calculus (RFC-0011 §4.4 Q5 closed; M-342).** The ANF substrate / AOT env-machine path
/// covers the whole v0 calculus: `Const/Var/Let/Op/Swap` plus the r3/r4 data + recursion nodes
/// (`Construct`/`Match`/`Lam`/`App`/`Fix`). Body-bearing nodes (`Lam`/`Fix` bodies, `Match` arm/default
/// bodies) lower to **nested** ANF blocks evaluated lazily by the env-machine (so the binding list
/// stays acyclic and arms/closures are not eagerly run); a single program-wide temp counter keeps
/// every `Temp` globally unique, so a nested scope can never shadow an enclosing temp.
///
/// The native LLVM backend (`mycelium-mlir::llvm`) remains the **bit/trit subset** and refuses
/// data/closure nodes with an explicit `UnsupportedNode` (VR-5); this ANF + the `aot::run` env-machine
/// are the path the three-way differential exercises on the full calculus.
#[must_use]
pub fn lower_to_anf(node: &Node) -> Anf {
    let mut next = 0usize;
    lower_block(node, &mut next)
}

/// Lower a (sub-)expression to its own ANF block, **sharing** the program-wide temp counter `next`
/// so temps stay globally unique across nested blocks (closure/arm bodies).
fn lower_block(node: &Node, next: &mut usize) -> Anf {
    let mut b = Vec::new();
    let result = flatten(node, &mut b, next);
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
        Node::Construct { ctor, args } => {
            let atoms: Vec<Atom> = args.iter().map(|a| flatten(a, out, next)).collect();
            let name = Atom::Temp(fresh(next));
            out.push(Binding {
                name: name.clone(),
                rhs: Rhs::Construct {
                    ctor: ctor.clone(),
                    args: atoms,
                },
                layout: None, // a datum is not a representation value — no physical layout.
            });
            name
        }
        Node::App { func, arg } => {
            let f = flatten(func, out, next);
            let a = flatten(arg, out, next);
            let name = Atom::Temp(fresh(next));
            out.push(Binding {
                name: name.clone(),
                rhs: Rhs::App { func: f, arg: a },
                layout: None,
            });
            name
        }
        Node::Lam { param, body } => {
            // The body is a nested block, not flattened into the current one: a closure body runs
            // only on application (lazy). The shared `next` keeps its temps globally unique.
            let body = lower_block(body, next);
            let name = Atom::Temp(fresh(next));
            out.push(Binding {
                name: name.clone(),
                rhs: Rhs::Lam {
                    param: param.clone(),
                    body,
                },
                layout: None,
            });
            name
        }
        Node::FixGroup { defs, body } => {
            // Lower every member definition to a nested block, then emit one `Rhs::FixGroup` binding
            // per member (each carrying the whole group). The member names are `Named` atoms, so the
            // continuation — and each sibling body — resolves them directly from the environment.
            let lowered: Vec<(VarId, Anf)> = defs
                .iter()
                .map(|(name, def)| (name.clone(), lower_block(def, next)))
                .collect();
            for (name, _) in defs {
                out.push(Binding {
                    name: Atom::Named(name.clone()),
                    rhs: Rhs::FixGroup {
                        defs: lowered.clone(),
                        which: name.clone(),
                    },
                    layout: None,
                });
            }
            flatten(body, out, next)
        }
        Node::Fix { name: fname, body } => {
            let body = lower_block(body, next);
            let name = Atom::Temp(fresh(next));
            out.push(Binding {
                name: name.clone(),
                rhs: Rhs::Fix {
                    name: fname.clone(),
                    body,
                },
                layout: None,
            });
            name
        }
        Node::Match {
            scrutinee,
            alts,
            default,
        } => {
            let s = flatten(scrutinee, out, next);
            // Each arm/default body is a nested block (evaluated only when selected, never eagerly).
            let alts: Vec<AnfAlt> = alts
                .iter()
                .map(|alt| match alt {
                    Alt::Ctor {
                        ctor,
                        binders,
                        body,
                    } => AnfAlt::Ctor {
                        ctor: ctor.clone(),
                        binders: binders.clone(),
                        body: lower_block(body, next),
                    },
                    Alt::Lit { value, body } => AnfAlt::Lit {
                        value: value.clone(),
                        body: lower_block(body, next),
                    },
                })
                .collect();
            let default = default.as_ref().map(|d| lower_block(d, next));
            let name = Atom::Temp(fresh(next));
            out.push(Binding {
                name: name.clone(),
                rhs: Rhs::Match {
                    scrutinee: s,
                    alts,
                    default,
                },
                layout: None,
            });
            name
        }
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

/// Render one lowered RHS into `s`, leaving the cursor at the end of its text (no trailing newline).
/// Flat RHSs render inline; body-bearing RHSs render a header then their nested block(s) indented.
fn write_rhs(rhs: &Rhs, depth: usize, s: &mut String) {
    match rhs {
        Rhs::Const(v) => {
            let _ = write!(s, "{}", render_const(v));
        }
        Rhs::Alias(a) => {
            let _ = write!(s, "{}", a.render());
        }
        Rhs::Op { prim, args } => {
            let a: Vec<String> = args.iter().map(Atom::render).collect();
            let _ = write!(s, "op {prim} {}", a.join(" "));
        }
        Rhs::Swap {
            src,
            target,
            policy,
        } => {
            let _ = write!(
                s,
                "swap {} -> {} @{}",
                src.render(),
                render_repr(target),
                short_hash(policy)
            );
        }
        Rhs::Construct { ctor, args } => {
            let a: Vec<String> = args.iter().map(Atom::render).collect();
            let _ = write!(s, "construct {ctor} {}", a.join(" "));
        }
        Rhs::App { func, arg } => {
            let _ = write!(s, "app {} {}", func.render(), arg.render());
        }
        Rhs::Lam { param, body } => {
            let _ = writeln!(s, "lam {param} =>");
            body.write_block(depth + 1, s);
        }
        Rhs::Fix { name, body } => {
            let _ = writeln!(s, "fix {name} =>");
            body.write_block(depth + 1, s);
        }
        Rhs::FixGroup { defs, which } => {
            let names: Vec<&str> = defs.iter().map(|(n, _)| n.as_str()).collect();
            let _ = writeln!(s, "fixgroup-member {which} of ({})", names.join(", "));
            for (name, body) in defs {
                let _ = writeln!(s, "{}def {name} =>", "  ".repeat(depth + 1));
                body.write_block(depth + 2, s);
            }
        }
        Rhs::Match {
            scrutinee,
            alts,
            default,
        } => {
            let _ = writeln!(s, "match {}", scrutinee.render());
            let pad = "  ".repeat(depth + 1);
            for alt in alts {
                match alt {
                    AnfAlt::Ctor {
                        ctor,
                        binders,
                        body,
                    } => {
                        let _ = writeln!(s, "{pad}alt {ctor} ({}) =>", binders.join(" "));
                        body.write_block(depth + 2, s);
                    }
                    AnfAlt::Lit { value, body } => {
                        let _ = writeln!(s, "{pad}alt-lit {} =>", render_const(value));
                        body.write_block(depth + 2, s);
                    }
                }
                s.push('\n');
            }
            match default {
                Some(d) => {
                    let _ = writeln!(s, "{pad}default =>");
                    d.write_block(depth + 2, s);
                }
                None => {
                    let _ = write!(s, "{pad}no-default");
                }
            }
        }
    }
}

impl Anf {
    /// The canonical, diffable dump of the substrate stage (SC-4). Nested blocks (closure/recursion
    /// bodies, match arms) render indented; the flat-fragment output is unchanged.
    #[must_use]
    pub fn dump(&self) -> String {
        let mut s = String::new();
        self.write_block(0, &mut s);
        s
    }

    fn write_block(&self, depth: usize, s: &mut String) {
        let pad = "  ".repeat(depth);
        let inner = "  ".repeat(depth + 1);
        let _ = writeln!(s, "{pad}substrate {{");
        for b in &self.bindings {
            let _ = write!(s, "{inner}{} = ", b.name.render());
            write_rhs(&b.rhs, depth + 1, s);
            if let Some(l) = b.layout {
                let _ = write!(s, "    ; layout={}", render_layout(l));
            }
            s.push('\n');
        }
        let _ = writeln!(s, "{inner}result {}", self.result.render());
        let _ = write!(s, "{pad}}}");
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

    // ===== Mutant-witnesses for render_scalar_kind (lower.rs:72:5) =====
    // Replaced with "" or "xyzzy" — must emit the actual scalar kind name.
    // Tests by checking dump_node output for Dense reprs carries the specific dtype string.
    #[test]
    fn render_scalar_kind_emits_the_kind_name() {
        // The dump of a Dense const must contain the dtype string: "F32", "F16", "BF16", "F64".
        for (dtype, expected) in [
            (ScalarKind::F32, "F32"),
            (ScalarKind::F16, "F16"),
            (ScalarKind::Bf16, "BF16"),
            (ScalarKind::F64, "F64"),
        ] {
            let v = Value::new(
                Repr::Dense { dim: 4, dtype },
                Payload::Scalars(vec![0.0, 0.0, 0.0, 0.0]),
                Meta::exact(Provenance::Root),
            )
            .unwrap();
            let text = dump_node(&Node::Const(v));
            assert!(
                text.contains(expected),
                "dump of Dense{{{dtype:?}}} must contain '{expected}': got {text:?}"
            );
            // The empty-string and "xyzzy" replacements would fail this check.
            assert!(
                !text.contains("xyzzy"),
                "dump must not contain sentinel 'xyzzy': got {text:?}"
            );
        }
    }

    // ===== Mutant-witnesses for render_payload (lower.rs:100:5) =====
    // Replaced with String::new() or "xyzzy".into() — must emit non-empty payload text.
    #[test]
    fn render_payload_emits_non_empty_payload_text() {
        // Bits payload: should contain "bits=..."
        let v_bits = Value::new(
            Repr::Binary { width: 4 },
            Payload::Bits(vec![true, false, true, false]),
            Meta::exact(Provenance::Root),
        )
        .unwrap();
        let bits_text = dump_node(&Node::Const(v_bits));
        assert!(
            bits_text.contains("bits=1010"),
            "dump of Binary Const must contain 'bits=1010': got {bits_text:?}"
        );
        // Trits payload: should contain "trits=..."
        use crate::value::Trit;
        let v_trits = Value::new(
            Repr::Ternary { trits: 3 },
            Payload::Trits(vec![Trit::Pos, Trit::Zero, Trit::Neg]),
            Meta::exact(Provenance::Root),
        )
        .unwrap();
        let trits_text = dump_node(&Node::Const(v_trits));
        assert!(
            trits_text.contains("trits=+0-"),
            "dump of Ternary Const must contain 'trits=+0-': got {trits_text:?}"
        );
    }

    // ===== Mutant-witnesses for short_hash (lower.rs:142:5) =====
    // Replaced with String::new() or "xyzzy".into() — must emit a non-empty algo:prefix string.
    #[test]
    fn short_hash_emits_algo_and_prefix() {
        let h = ContentHash::parse("blake3:round_trip_safe").unwrap();
        let v = Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(vec![false; 8]),
            Meta::exact(Provenance::Root),
        )
        .unwrap();
        let swap_node = Node::Swap {
            src: Box::new(Node::Const(v)),
            target: Repr::Ternary { trits: 6 },
            policy: h,
        };
        let text = dump_node(&swap_node);
        // short_hash renders as "algo:first8chars" — the @prefix should appear in the dump.
        assert!(
            text.contains("@blake3:"),
            "dump of Swap must contain '@blake3:' from short_hash: got {text:?}"
        );
        // The empty-string and "xyzzy" replacements would omit this.
        assert!(!text.is_empty(), "dump must not be empty");
    }

    // ===== Mutant-witnesses for dump_node (lower.rs:154:5) =====
    // Replaced with String::new() or "xyzzy".into() — must emit the actual node representation.
    #[test]
    fn dump_node_emits_non_empty_and_meaningful_text() {
        let text = dump_node(&Node::Var("hello".into()));
        assert!(!text.is_empty(), "dump_node must return non-empty string");
        assert!(text.contains("var hello"), "dump_node Var must contain 'var hello': {text:?}");
        assert!(!text.contains("xyzzy"), "dump_node must not return sentinel");

        let text2 = dump_node(&Node::Const(byte()));
        assert!(text2.contains("const Binary{8}"), "dump_node Const must contain type: {text2:?}");
    }

    // ===== Mutant-witnesses for format (lower.rs:170:5 and write_canon with ()) =====
    // format() replaced with String::new() or "xyzzy".into().
    // write_canon replaced with () — makes format() always return "".
    // Tests check that format() returns non-empty, meaningful α-normalized output.
    #[test]
    fn format_returns_alpha_normalized_output() {
        // A Var node should render with a canonical name.
        let text = format(&Node::Var("my_var".into()));
        assert!(!text.is_empty(), "format must return non-empty string");
        // A Const should render with the value details.
        let text2 = format(&Node::Const(byte()));
        assert!(text2.contains("const"), "format Const must contain 'const': {text2:?}");
        assert!(!text2.contains("xyzzy"), "format must not return sentinel");
    }

    // ===== Mutant-witnesses for write_canon counter arithmetic (lower.rs:202:22 += → -=/*=) =====
    // *counter += 1 → *counter -= 1 or *counter *= 1 makes consecutive let/lam binders get the
    // same or decrementing names. Tests check that nested let nodes generate sequential names.
    #[test]
    fn format_alpha_normalizes_nested_lets_with_sequential_names() {
        // let x = (let y = c in y) in x — two nested lets should get v0 and v1.
        let c = Node::Const(byte());
        let inner = Node::Let {
            id: "y".into(),
            bound: Box::new(c),
            body: Box::new(Node::Var("y".into())),
        };
        let outer = Node::Let {
            id: "x".into(),
            bound: Box::new(inner),
            body: Box::new(Node::Var("x".into())),
        };
        let text = format(&outer);
        // Must contain both v0 and v1 (sequential counter).
        assert!(text.contains("v0"), "format must use v0 for first let: {text:?}");
        assert!(text.contains("v1"), "format must use v1 for second let: {text:?}");
        // The two names must be DIFFERENT (counter must increment, not stay at 0).
        let v0_count = text.matches("v0").count();
        let v1_count = text.matches("v1").count();
        // Both appear — different binders are given different names.
        assert!(v0_count > 0 && v1_count > 0,
            "format must produce distinct sequential names v0, v1: {text:?}");
    }

    // ===== Mutant-witnesses for write_canon indentation arithmetic (depth + 1 → depth - 1/*1) =====
    // write_canon(bound, depth + 1, ...) → wrong depth. Tests check indentation levels are correct.
    #[test]
    fn format_indents_nested_nodes_more_than_parent() {
        // A Let at depth 0 renders "let v0 =" then the bound at depth 1 (2 more spaces).
        let let_node = Node::Let {
            id: "x".into(),
            bound: Box::new(Node::Const(byte())),
            body: Box::new(Node::Var("x".into())),
        };
        let text = format(&let_node);
        // The "let" keyword should appear at depth 0 (no leading spaces).
        let let_line = text.lines().find(|l| l.contains("let v0")).expect("must have let line");
        assert!(!let_line.starts_with("  "),
            "let at depth 0 must not be indented: {let_line:?}");
        // The bound (const) should be at depth 1 (2 leading spaces).
        let const_line = text.lines().find(|l| l.contains("const")).expect("must have const line");
        assert!(const_line.starts_with("  "),
            "const at depth 1 must start with 2 spaces: {const_line:?}");
        assert!(!const_line.starts_with("    "),
            "const at depth 1 must not start with 4 spaces (depth+1 must be 1, not 2): {const_line:?}");
    }

    // ===== Mutant-witnesses for write_core indentation arithmetic =====
    // Similar to write_canon but for dump_node (which uses write_core).
    #[test]
    fn dump_node_indents_nested_nodes_more_than_parent() {
        let let_node = Node::Let {
            id: "x".into(),
            bound: Box::new(Node::Const(byte())),
            body: Box::new(Node::Var("x".into())),
        };
        let text = dump_node(&let_node);
        // "let x =" at depth 0 — no leading spaces.
        let let_line = text.lines().find(|l| l.contains("let x")).expect("must have let line");
        assert!(!let_line.starts_with("  "),
            "let at depth 0 must not be indented in dump_node: {let_line:?}");
        // "const ..." at depth 1 — 2 leading spaces.
        let const_line = text.lines().find(|l| l.contains("const")).expect("must have const line");
        assert!(const_line.starts_with("  "),
            "const at depth 1 must start with 2 spaces in dump_node: {const_line:?}");
        assert!(!const_line.starts_with("    "),
            "const at depth 1 must not start with 4 spaces: {const_line:?}");
    }

    // ===== Mutant-witnesses for fresh() counter (lower.rs:603:5 → 0, lower.rs:604:11 += → *=) =====
    // fresh() replaced with constant 0: every temp is %0.
    // *next += 1 → *next *= 1: counter never advances, all temps are %0.
    // Tests check that multiple Consts/Ops produce DISTINCT temp names.
    #[test]
    fn lowering_assigns_distinct_sequential_temps_to_consts() {
        // Two constants in an Op: each should get a distinct temp name.
        let c = Node::Const(byte());
        let node = Node::Op {
            prim: "bit.xor".into(),
            args: vec![c.clone(), c.clone()],
        };
        let anf = lower_to_anf(&node);
        // With 2 Const args + 1 Op result, there must be at least 2 distinct temps.
        let dump = anf.dump();
        assert!(dump.contains("%0"), "must have temp %0: {dump:?}");
        assert!(dump.contains("%1"), "must have temp %1: {dump:?}");
        // If fresh() always returns 0, %1 would never appear (everything is %0).
        // If counter never advances (*= 1 mutant), same issue.
    }

    // ===== Mutant-witnesses for Anf::len, is_empty, bindings =====
    // lower.rs:898:9 Anf::len replaced with 0 or 1.
    // lower.rs:904:9 Anf::is_empty replaced with true or false.
    // lower.rs:910:9 Anf::bindings replaced with Vec::leak(Vec::new()).
    #[test]
    fn anf_len_is_empty_and_bindings_reflect_actual_content() {
        // A single Const node produces exactly 1 binding.
        let anf_const = lower_to_anf(&Node::Const(byte()));
        assert_eq!(anf_const.len(), 1, "single Const must produce 1 binding");
        assert!(!anf_const.is_empty(), "single Const ANF must not be empty");
        assert_eq!(anf_const.bindings().len(), 1, "bindings() must have 1 entry");

        // A Var produces 0 bindings (it's just a named atom — no temp allocation).
        let anf_var = lower_to_anf(&Node::Var("x".into()));
        assert_eq!(anf_var.len(), 0, "Var must produce 0 bindings");
        assert!(anf_var.is_empty(), "Var ANF must be empty");
        assert_eq!(anf_var.bindings().len(), 0, "bindings() must have 0 entries for Var");

        // Two nested Consts produce 2 bindings.
        let node = Node::Op {
            prim: "bit.not".into(),
            args: vec![Node::Const(byte())],
        };
        let anf_op = lower_to_anf(&node);
        // 1 binding for the Const arg + 1 for the Op result = 2.
        assert_eq!(anf_op.len(), 2, "Op(Const) must produce 2 bindings: {}", anf_op.dump());
        assert!(!anf_op.is_empty(), "Op(Const) ANF must not be empty");
        assert_eq!(anf_op.bindings().len(), 2, "bindings() must have 2 entries");
        // The bindings() slice must contain the actual bindings, not an empty Vec.
        let names: Vec<String> = anf_op.bindings().iter().map(|b| b.name.render()).collect();
        assert!(names.len() == 2, "bindings() must have 2 named entries: {names:?}");
    }

    // ===== Mutant-witnesses for write_rhs indentation arithmetic (lower.rs:818–859) =====
    // depth + 1 → depth - 1 or depth * 1 in write_rhs's recursive write_block calls.
    // Tests check that nested lambda/fix bodies in the substrate dump are indented correctly.
    #[test]
    fn substrate_dump_indents_nested_blocks() {
        // A Lam node lowers to an Rhs::Lam which calls write_block at depth+1.
        let lam = Node::Lam {
            param: "x".into(),
            body: Box::new(Node::Const(byte())),
        };
        let anf = lower_to_anf(&lam);
        let dump = anf.dump();
        // The substrate block header at depth 0: "substrate {"
        assert!(dump.contains("substrate {"), "must have substrate header: {dump:?}");
        // There must be content inside the substrate block (inner indent > outer).
        // The inner block is indented by 2 additional spaces vs the enclosing "substrate {".
        let lines: Vec<&str> = dump.lines().collect();
        // Find the "substrate {" line and check that the line after it is indented.
        let sub_idx = lines.iter().position(|l| l.contains("substrate {")).unwrap();
        if sub_idx + 1 < lines.len() {
            // At minimum, something follows at greater indentation.
            let next_meaningful: Option<&str> = lines[(sub_idx + 1)..].iter()
                .find(|l| !l.trim().is_empty())
                .copied();
            if let Some(inner_line) = next_meaningful {
                // Must have at least 2 leading spaces (depth 1 = "  ").
                assert!(inner_line.starts_with("  "),
                    "inner substrate content must be indented: {inner_line:?}\nfull dump:\n{dump}");
            }
        }
    }

    // ===== Mutant-witnesses for write_block indentation (lower.rs:881, 885) =====
    // "  ".repeat(depth + 1) → "  ".repeat(depth * 1). Tests check that nested substrate blocks
    // (inner Anf inside a Lam/Fix Rhs) indent the inner substrate header.
    #[test]
    fn nested_substrate_block_is_indented_relative_to_outer() {
        // A Fix over a body of Fix: produces a nested substrate block.
        let fix = Node::Fix {
            name: "f".into(),
            body: Box::new(Node::Var("f".into())),
        };
        let anf = lower_to_anf(&fix);
        let dump = anf.dump();
        // Find the Rhs::Fix rendering — the outer substrate {} header is at depth 0.
        // The nested body's "substrate {" should be at depth 2 (4 spaces).
        // At minimum, there should be a nested block.
        assert!(dump.contains("fix f =>"), "must have 'fix f =>' in dump: {dump:?}");
        // Find any "substrate {" after the first one — that's the nested block.
        let substrate_count = dump.matches("substrate {").count();
        assert!(substrate_count >= 2,
            "nested Fix must have >= 2 substrate blocks in dump: {dump:?}");
        // The inner substrate block must be more indented than the outer one.
        let lines: Vec<&str> = dump.lines().collect();
        let sub_lines: Vec<(usize, &&str)> = lines.iter().enumerate()
            .filter(|(_, l)| l.contains("substrate {"))
            .collect();
        if sub_lines.len() >= 2 {
            let outer_indent = sub_lines[0].1.len() - sub_lines[0].1.trim_start().len();
            let inner_indent = sub_lines[1].1.len() - sub_lines[1].1.trim_start().len();
            assert!(inner_indent > outer_indent,
                "inner substrate block must be more indented than outer:\nouter={outer_indent}, inner={inner_indent}\n{dump}");
        }
    }
}
