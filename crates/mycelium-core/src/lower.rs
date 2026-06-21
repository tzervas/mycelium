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
        assert!(
            text.contains("var hello"),
            "dump_node Var must contain 'var hello': {text:?}"
        );
        assert!(
            !text.contains("xyzzy"),
            "dump_node must not return sentinel"
        );

        let text2 = dump_node(&Node::Const(byte()));
        assert!(
            text2.contains("const Binary{8}"),
            "dump_node Const must contain type: {text2:?}"
        );
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
        assert!(
            text2.contains("const"),
            "format Const must contain 'const': {text2:?}"
        );
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
        assert!(
            text.contains("v0"),
            "format must use v0 for first let: {text:?}"
        );
        assert!(
            text.contains("v1"),
            "format must use v1 for second let: {text:?}"
        );
        // The two names must be DIFFERENT (counter must increment, not stay at 0).
        let v0_count = text.matches("v0").count();
        let v1_count = text.matches("v1").count();
        // Both appear — different binders are given different names.
        assert!(
            v0_count > 0 && v1_count > 0,
            "format must produce distinct sequential names v0, v1: {text:?}"
        );
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
        let let_line = text
            .lines()
            .find(|l| l.contains("let v0"))
            .expect("must have let line");
        assert!(
            !let_line.starts_with("  "),
            "let at depth 0 must not be indented: {let_line:?}"
        );
        // The bound (const) should be at depth 1 (2 leading spaces).
        let const_line = text
            .lines()
            .find(|l| l.contains("const"))
            .expect("must have const line");
        assert!(
            const_line.starts_with("  "),
            "const at depth 1 must start with 2 spaces: {const_line:?}"
        );
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
        let let_line = text
            .lines()
            .find(|l| l.contains("let x"))
            .expect("must have let line");
        assert!(
            !let_line.starts_with("  "),
            "let at depth 0 must not be indented in dump_node: {let_line:?}"
        );
        // "const ..." at depth 1 — 2 leading spaces.
        let const_line = text
            .lines()
            .find(|l| l.contains("const"))
            .expect("must have const line");
        assert!(
            const_line.starts_with("  "),
            "const at depth 1 must start with 2 spaces in dump_node: {const_line:?}"
        );
        assert!(
            !const_line.starts_with("    "),
            "const at depth 1 must not start with 4 spaces: {const_line:?}"
        );
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
        assert_eq!(
            anf_const.bindings().len(),
            1,
            "bindings() must have 1 entry"
        );

        // A Var produces 0 bindings (it's just a named atom — no temp allocation).
        let anf_var = lower_to_anf(&Node::Var("x".into()));
        assert_eq!(anf_var.len(), 0, "Var must produce 0 bindings");
        assert!(anf_var.is_empty(), "Var ANF must be empty");
        assert_eq!(
            anf_var.bindings().len(),
            0,
            "bindings() must have 0 entries for Var"
        );

        // Two nested Consts produce 2 bindings.
        let node = Node::Op {
            prim: "bit.not".into(),
            args: vec![Node::Const(byte())],
        };
        let anf_op = lower_to_anf(&node);
        // 1 binding for the Const arg + 1 for the Op result = 2.
        assert_eq!(
            anf_op.len(),
            2,
            "Op(Const) must produce 2 bindings: {}",
            anf_op.dump()
        );
        assert!(!anf_op.is_empty(), "Op(Const) ANF must not be empty");
        assert_eq!(anf_op.bindings().len(), 2, "bindings() must have 2 entries");
        // The bindings() slice must contain the actual bindings, not an empty Vec.
        let names: Vec<String> = anf_op.bindings().iter().map(|b| b.name.render()).collect();
        assert!(
            names.len() == 2,
            "bindings() must have 2 named entries: {names:?}"
        );
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
        assert!(
            dump.contains("substrate {"),
            "must have substrate header: {dump:?}"
        );
        // There must be content inside the substrate block (inner indent > outer).
        // The inner block is indented by 2 additional spaces vs the enclosing "substrate {".
        let lines: Vec<&str> = dump.lines().collect();
        // Find the "substrate {" line and check that the line after it is indented.
        let sub_idx = lines
            .iter()
            .position(|l| l.contains("substrate {"))
            .unwrap();
        if sub_idx + 1 < lines.len() {
            // At minimum, something follows at greater indentation.
            let next_meaningful: Option<&str> = lines[(sub_idx + 1)..]
                .iter()
                .find(|l| !l.trim().is_empty())
                .copied();
            if let Some(inner_line) = next_meaningful {
                // Must have at least 2 leading spaces (depth 1 = "  ").
                assert!(
                    inner_line.starts_with("  "),
                    "inner substrate content must be indented: {inner_line:?}\nfull dump:\n{dump}"
                );
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
        assert!(
            dump.contains("fix f =>"),
            "must have 'fix f =>' in dump: {dump:?}"
        );
        // Find any "substrate {" after the first one — that's the nested block.
        let substrate_count = dump.matches("substrate {").count();
        assert!(
            substrate_count >= 2,
            "nested Fix must have >= 2 substrate blocks in dump: {dump:?}"
        );
        // The inner substrate block must be more indented than the outer one.
        let lines: Vec<&str> = dump.lines().collect();
        let sub_lines: Vec<(usize, &&str)> = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.contains("substrate {"))
            .collect();
        if sub_lines.len() >= 2 {
            let outer_indent = sub_lines[0].1.len() - sub_lines[0].1.trim_start().len();
            let inner_indent = sub_lines[1].1.len() - sub_lines[1].1.trim_start().len();
            assert!(inner_indent > outer_indent,
                "inner substrate block must be more indented than outer:\nouter={outer_indent}, inner={inner_indent}\n{dump}");
        }
    }

    // ===== Mutant-witnesses for FixGroup indentation in write_canon (lower.rs:313:30, 315:40,
    // 317:26, 319:37) and the indent() function itself (lower.rs:326:5) =====
    //
    // write_canon FixGroup branch:
    //   line 313: indent(depth + 1, s)  — "def vX =>" lines must be at depth+1 (2 leading spaces)
    //   line 315: write_canon(def, depth + 2, ...) — def body must be at depth+2 (4 leading spaces)
    //   line 317: indent(depth + 1, s)  — "in" line must be at depth+1 (2 leading spaces)
    //   line 319: write_canon(body, depth + 1, ...) — body must be at depth+1 (2 leading spaces)
    //
    // indent() itself (line 326): replaced with () → all lines lose indentation entirely.
    //
    // A FixGroup at depth 0 should produce:
    //   fixgroup              ← depth 0, no indent
    //     def v0 =>           ← depth+1 = 1, "  " (2 spaces)
    //       <def_body>        ← depth+2 = 2, "    " (4 spaces)
    //     in                  ← depth+1 = 1, "  " (2 spaces)
    //       <body>            ← depth+1 = 1, "  " (2 spaces)
    #[test]
    fn format_fixgroup_indents_def_and_body_correctly() {
        // FixGroup { defs: [("f", Const), ("g", Const)], body: Var("f") }
        // α-normalize: f→v0, g→v1
        let c = Node::Const(byte());
        let fix_node = Node::FixGroup {
            defs: vec![
                ("f".into(), Box::new(c.clone())),
                ("g".into(), Box::new(c.clone())),
            ],
            body: Box::new(Node::Var("f".into())),
        };
        let text = format(&fix_node);

        // The "fixgroup" header must appear at depth 0 (no leading spaces).
        let fixgroup_line = text
            .lines()
            .find(|l| l.trim_start() == "fixgroup")
            .expect("format of FixGroup must contain 'fixgroup' header");
        assert!(
            !fixgroup_line.starts_with("  "),
            "fixgroup header must not be indented at depth 0: {fixgroup_line:?}"
        );

        // Each "def vX =>" line must appear at depth+1 = "  " (exactly 2 leading spaces).
        // Kills: indent(depth + 1, s) → () at line 313 (no indent → "def" at column 0)
        // Kills: indent function → () at line 326 (all indentation lost everywhere)
        for def_line in text.lines().filter(|l| l.contains("def v")) {
            assert!(
                def_line.starts_with("  "),
                "'def vX' must start with 2 spaces at depth+1: {def_line:?}"
            );
            assert!(!def_line.starts_with("    "),
                "'def vX' must not start with 4 spaces (must be depth+1, not depth+2): {def_line:?}");
        }

        // The "in" continuation must also appear at depth+1 = "  " (exactly 2 leading spaces).
        // Kills: indent(depth + 1, s) → () at line 317 (no indent → "in" at column 0)
        let in_line = text
            .lines()
            .find(|l| l.trim() == "in")
            .expect("format of FixGroup must contain 'in' continuation line");
        assert!(
            in_line.starts_with("  "),
            "'in' must start with 2 spaces at depth+1: {in_line:?}"
        );
        assert!(
            !in_line.starts_with("    "),
            "'in' must not start with 4 spaces (must be depth+1): {in_line:?}"
        );

        // The def body (a Const at depth+2) must appear at depth+2 = "    " (4 leading spaces).
        // Kills: write_canon(def, depth + 2, ...) → +2 replaced with *1 or -1 (wrong depth)
        let const_lines: Vec<&str> = text.lines().filter(|l| l.contains("const")).collect();
        assert!(
            !const_lines.is_empty(),
            "FixGroup must produce const lines from def bodies"
        );
        for const_line in &const_lines {
            assert!(
                const_line.starts_with("    "),
                "const in FixGroup def body must start with 4 spaces at depth+2: {const_line:?}"
            );
        }

        // The continuation body (Var "f" → rendered as "var v0" since f is the first def name)
        // must appear at depth+1 = "  " (2 leading spaces).
        // Kills: write_canon(body, depth + 1, ...) → +1 replaced with *1=0 (body unindented)
        let var_lines: Vec<&str> = text
            .lines()
            .filter(|l| l.trim_start().starts_with("var "))
            .collect();
        assert!(
            !var_lines.is_empty(),
            "FixGroup must produce a 'var' line for the body: {text:?}"
        );
        // The body var line is the LAST var line (after the "in" keyword); all var lines should
        // be at depth+1 = "  " (body is at depth+1; no vars appear inside def bodies here).
        for var_line in &var_lines {
            assert!(
                var_line.starts_with("  "),
                "body 'var' at depth+1 must start with 2 spaces: {var_line:?}"
            );
            // If depth+1 → depth*1 (=0 at root), no leading spaces → this assertion fails.
            // Mutation at line 319 produces 0 spaces.
        }
    }

    // ===== Mutant-witnesses for FixGroup indentation in write_core (dump_node path) =====
    // write_core FixGroup branch at:
    //   line 418: indent(depth + 1, s) — "def name =>" lines must be at depth+1
    //   line 420: write_core(def, depth + 2, s) — def body at depth+2
    //   line 422: indent(depth + 1, s) — "in" line at depth+1
    //   line 424: write_core(body, depth + 1, s) — body at depth+1
    //
    // Same structural invariant as write_canon, but via dump_node (uses original names).
    #[test]
    fn dump_node_fixgroup_indents_def_and_body_correctly() {
        let c = Node::Const(byte());
        let fix_node = Node::FixGroup {
            defs: vec![("f".into(), Box::new(c))],
            body: Box::new(Node::Var("f".into())),
        };
        let text = dump_node(&fix_node);

        // "fixgroup" at depth 0, no leading spaces.
        let fixgroup_line = text
            .lines()
            .find(|l| l.trim_start() == "fixgroup")
            .expect("dump_node FixGroup must contain 'fixgroup' header");
        assert!(
            !fixgroup_line.starts_with("  "),
            "fixgroup header must not be indented in dump_node: {fixgroup_line:?}"
        );

        // "def f =>" at depth+1 = 2 spaces.
        let def_line = text
            .lines()
            .find(|l| l.contains("def f"))
            .expect("dump_node FixGroup must contain 'def f =>' line");
        assert!(
            def_line.starts_with("  "),
            "'def f' must start with 2 spaces at depth+1 in dump_node: {def_line:?}"
        );
        assert!(
            !def_line.starts_with("    "),
            "'def f' must not start with 4 spaces in dump_node: {def_line:?}"
        );

        // "in" at depth+1 = 2 spaces.
        let in_line = text
            .lines()
            .find(|l| l.trim() == "in")
            .expect("dump_node FixGroup must contain 'in' line");
        assert!(
            in_line.starts_with("  "),
            "'in' must start with 2 spaces at depth+1 in dump_node: {in_line:?}"
        );
        assert!(
            !in_line.starts_with("    "),
            "'in' must not start with 4 spaces in dump_node: {in_line:?}"
        );

        // const body at depth+2 = 4 spaces.
        let const_line = text
            .lines()
            .find(|l| l.contains("const"))
            .expect("dump_node FixGroup must have const line in def body");
        assert!(const_line.starts_with("    "),
            "const in FixGroup def must start with 4 spaces at depth+2 in dump_node: {const_line:?}");
    }

    // ===== Mutant-witnesses for write_core Let bound indentation (lower.rs:342:37) =====
    // write_core(bound, depth + 1, s) → +1 replaced with *1 or -1.
    // If depth * 1 = depth (= 0 at root), the bound renders with 0 spaces instead of 2.
    // This test is separate from dump_node_indents_nested_nodes_more_than_parent because
    // that test may not fail on depth-0 mutations (0*1 = 0 = 0+1 is NOT equal when depth=0).
    #[test]
    fn dump_node_let_bound_is_indented_at_depth_plus_one() {
        // Nested Let: outer at depth 0, bound (inner Let) at depth 1, const body at depth 2.
        // Kills: write_core(bound, depth + 1, s) → depth*1 (=0 at root → no indent for bound).
        let inner = Node::Let {
            id: "y".into(),
            bound: Box::new(Node::Const(byte())),
            body: Box::new(Node::Var("y".into())),
        };
        let outer = Node::Let {
            id: "x".into(),
            bound: Box::new(inner),
            body: Box::new(Node::Var("x".into())),
        };
        let text = dump_node(&outer);

        // The outer "let x =" is at depth 0 (no leading spaces).
        let outer_let = text
            .lines()
            .find(|l| l.contains("let x"))
            .expect("dump_node nested Let must have 'let x' line");
        assert!(
            !outer_let.starts_with("  "),
            "outer let at depth 0 must not be indented: {outer_let:?}"
        );

        // The inner "let y =" is the bound of outer — must be at depth 1 = "  " (2 spaces).
        // Kills: write_core(bound, depth + 1, s) → depth * 1 (0 spaces when root depth=0)
        let inner_let = text
            .lines()
            .find(|l| l.contains("let y"))
            .expect("dump_node nested Let must have 'let y' line for bound");
        assert!(
            inner_let.starts_with("  "),
            "inner let (bound at depth+1) must start with 2 spaces: {inner_let:?}"
        );
        assert!(
            !inner_let.starts_with("    "),
            "inner let (bound at depth+1) must not start with 4 spaces: {inner_let:?}"
        );

        // The const body of the inner let is at depth 2 = "    " (4 spaces).
        let const_line = text
            .lines()
            .find(|l| l.contains("const"))
            .expect("dump_node nested Let must have const line");
        assert!(
            const_line.starts_with("    "),
            "const at depth+2 must start with 4 spaces: {const_line:?}"
        );
    }

    // ===== Mutant-witnesses: write_canon Op/Swap/Lam/App/Fix child indentation =====
    // Covers survivors at lower.rs lines 214 (Op args), 223 (Swap src), 228 (Construct args),
    // 285 (Lam body), 290/291 (App func/arg), 298 (Fix body).
    // depth+1 → depth*1 (=0 at root) or depth-1 (underflow) produces 0-space child lines.
    // At root depth=0: depth+1=1 → 2 leading spaces; depth*1=0 → 0 spaces.
    #[test]
    fn format_all_node_types_indent_children_at_depth_one() {
        // Op: args must be at depth 1 = "  " (2 spaces).
        let op_node = Node::Op {
            prim: "bit.xor".into(),
            args: vec![Node::Const(byte())],
        };
        let op_text = format(&op_node);
        let op_line = op_text
            .lines()
            .find(|l| l.trim_start().starts_with("const"))
            .expect("Op format must have a const arg line");
        assert!(
            op_line.starts_with("  "),
            "Op arg at depth 1 must start with 2 spaces: {op_line:?}\nfull:\n{op_text}"
        );
        assert!(
            !op_line.starts_with("    "),
            "Op arg must not have 4 spaces (depth must be 1 not 2): {op_line:?}"
        );

        // Swap: src must be at depth 1 = "  " (2 spaces).
        let swap_node = Node::Swap {
            src: Box::new(Node::Const(byte())),
            target: Repr::Ternary { trits: 6 },
            policy: crate::ContentHash::parse("blake3:round_trip_safe").unwrap(),
        };
        let swap_text = format(&swap_node);
        let swap_src_line = swap_text
            .lines()
            .find(|l| l.trim_start().starts_with("const"))
            .expect("Swap format must have a const src line");
        assert!(
            swap_src_line.starts_with("  "),
            "Swap src at depth 1 must start with 2 spaces: {swap_src_line:?}\nfull:\n{swap_text}"
        );
        assert!(
            !swap_src_line.starts_with("    "),
            "Swap src must not have 4 spaces: {swap_src_line:?}"
        );

        // Lam: body must be at depth 1 = "  " (2 spaces).
        let lam_node = Node::Lam {
            param: "x".into(),
            body: Box::new(Node::Const(byte())),
        };
        let lam_text = format(&lam_node);
        let lam_body_line = lam_text
            .lines()
            .find(|l| l.trim_start().starts_with("const"))
            .expect("Lam format must have a const body line");
        assert!(
            lam_body_line.starts_with("  "),
            "Lam body at depth 1 must start with 2 spaces: {lam_body_line:?}\nfull:\n{lam_text}"
        );
        assert!(
            !lam_body_line.starts_with("    "),
            "Lam body must not have 4 spaces: {lam_body_line:?}"
        );

        // App: both func and arg must be at depth 1 = "  " (2 spaces).
        let app_node = Node::App {
            func: Box::new(Node::Lam {
                param: "y".into(),
                body: Box::new(Node::Var("y".into())),
            }),
            arg: Box::new(Node::Const(byte())),
        };
        let app_text = format(&app_node);
        // The "lam" line is the func — must be indented at depth 1.
        let app_func_line = app_text
            .lines()
            .find(|l| l.trim_start().starts_with("lam "))
            .expect("App format must have a 'lam' func line");
        assert!(
            app_func_line.starts_with("  "),
            "App func at depth 1 must start with 2 spaces: {app_func_line:?}\nfull:\n{app_text}"
        );
        assert!(
            !app_func_line.starts_with("    "),
            "App func must not have 4 spaces: {app_func_line:?}"
        );
        // The "const" line is the arg — must also be indented at depth 1.
        let app_arg_line = app_text
            .lines()
            .find(|l| l.trim_start().starts_with("const"))
            .expect("App format must have a 'const' arg line");
        assert!(
            app_arg_line.starts_with("  "),
            "App arg at depth 1 must start with 2 spaces: {app_arg_line:?}\nfull:\n{app_text}"
        );
        assert!(
            !app_arg_line.starts_with("    "),
            "App arg must not have 4 spaces: {app_arg_line:?}"
        );

        // Fix: body must be at depth 1 = "  " (2 spaces).
        let fix_node = Node::Fix {
            name: "f".into(),
            body: Box::new(Node::Const(byte())),
        };
        let fix_text = format(&fix_node);
        let fix_body_line = fix_text
            .lines()
            .find(|l| l.trim_start().starts_with("const"))
            .expect("Fix format must have a const body line");
        assert!(
            fix_body_line.starts_with("  "),
            "Fix body at depth 1 must start with 2 spaces: {fix_body_line:?}\nfull:\n{fix_text}"
        );
        assert!(
            !fix_body_line.starts_with("    "),
            "Fix body must not have 4 spaces: {fix_body_line:?}"
        );
    }

    // ===== Mutant-witnesses: write_canon Match scrutinee/alt body/default indentation =====
    // Covers survivors at lower.rs lines 237 (scrutinee), 260 (ctor alt body), 265 (lit alt body),
    // 273 (default body).
    // depth+1 → depth*1 (=0 at root) produces 0-space lines for scrutinee, alt bodies, default.
    #[test]
    fn format_match_and_alt_indent_at_correct_depths() {
        // Match with a Lit alt and a default. Scrutinee, alt body, and default body must be at
        // depth 1 = "  " (2 leading spaces) when Match is at depth 0.
        let match_node = Node::Match {
            scrutinee: Box::new(Node::Const(byte())),
            alts: vec![crate::node::Alt::Lit {
                value: byte(),
                body: Node::Var("z".into()),
            }],
            default: Some(Box::new(Node::Const(byte()))),
        };
        let text = format(&match_node);

        // The scrutinee (a const) is at depth 1 = "  ".
        // Kills: write_canon(scrutinee, depth + 1, ...) → +1 with *1 (→ depth=0, no indent).
        let scrutinee_line = text
            .lines()
            .find(|l| l.trim_start().starts_with("const"))
            .expect("Match format must have a const scrutinee line");
        assert!(
            scrutinee_line.starts_with("  "),
            "Match scrutinee at depth 1 must start with 2 spaces: {scrutinee_line:?}\nfull:\n{text}"
        );
        assert!(
            !scrutinee_line.starts_with("    "),
            "Match scrutinee must not have 4 spaces: {scrutinee_line:?}"
        );

        // The Lit alt body (a free var "z") is at depth 1 = "  ".
        // Kills: write_canon(body, depth + 1, ...) at Alt::Lit line 265 → *1 (0 spaces).
        let lit_body_line = text
            .lines()
            .find(|l| l.trim_start().starts_with("free z"))
            .expect("Match format must have a 'free z' lit-alt body line");
        assert!(
            lit_body_line.starts_with("  "),
            "Lit alt body at depth 1 must start with 2 spaces: {lit_body_line:?}\nfull:\n{text}"
        );
        assert!(
            !lit_body_line.starts_with("    "),
            "Lit alt body must not have 4 spaces: {lit_body_line:?}"
        );

        // The default body (a const) appears after the "default" keyword, at depth 1 = "  ".
        // Kills: write_canon(d, depth + 1, ...) at line 273 → *1 (0 spaces).
        // We check the const after the "default" line (the scrutinee const appears first).
        let lines: Vec<&str> = text.lines().collect();
        let default_idx = lines
            .iter()
            .position(|l| l.trim() == "default")
            .expect("Match format must have a 'default' line");
        let default_body = lines[default_idx + 1];
        assert!(
            default_body.starts_with("  "),
            "default body at depth 1 must start with 2 spaces: {default_body:?}\nfull:\n{text}"
        );
        assert!(
            !default_body.starts_with("    "),
            "default body must not have 4 spaces: {default_body:?}"
        );
    }

    // ===== Mutant-witnesses: write_canon Var lookup (line 191 `==` → `!=`) =====
    // If `==` is replaced with `!=`: a bound var (in scope) would render as "free X" and a
    // free var (not in scope) would render as "var canon". The test checks that a bound var in a
    // Lam body renders as "var vN" (not "free x") and a genuinely free var renders as "free y".
    #[test]
    fn format_var_lookup_distinguishes_bound_and_free() {
        // Lam { param: "x", body: Var("x") } — "x" is bound, should render as "var v0".
        let lam_bound = Node::Lam {
            param: "x".into(),
            body: Box::new(Node::Var("x".into())),
        };
        let text = format(&lam_bound);
        // Must contain "var v0" (the bound param's canonical name).
        assert!(
            text.contains("var v0"),
            "bound Var in Lam body must render as 'var v0', got: {text:?}"
        );
        // Must NOT render "x" as a free var.
        assert!(
            !text.contains("free x"),
            "bound Var 'x' must not render as 'free x', got: {text:?}"
        );

        // A free variable (not bound by any enclosing Lam/Let/Fix) renders as "free y".
        let free_var = Node::Var("y".into());
        let text2 = format(&free_var);
        assert!(
            text2.contains("free y"),
            "free Var 'y' must render as 'free y', got: {text2:?}"
        );
        assert!(
            !text2.contains("var y"),
            "free Var 'y' must not render as 'var y' (no canon name), got: {text2:?}"
        );

        // A shadowed variable: outer Let binds "x", inner Lam rebinds "x". The Lam param
        // takes precedence (innermost-first). The body var "x" inside the Lam should see the
        // innermost binding.
        // let x = const in (lam x => x body)
        let shadowed = Node::Let {
            id: "x".into(),
            bound: Box::new(Node::Const(byte())),
            body: Box::new(Node::Lam {
                param: "x".into(),
                body: Box::new(Node::Var("x".into())),
            }),
        };
        let text3 = format(&shadowed);
        // x bound by Let → v0; x rebound by Lam → v1 (counter increments).
        // The var "x" inside lam sees the lam binder (v1), not the let binder (v0).
        // Killed if == → !=: the inner "x" would see the let binder instead of the lam binder.
        assert!(
            text3.contains("var v1"),
            "Var 'x' inside Lam must resolve to the Lam binder v1 (innermost), got: {text3:?}"
        );
    }

    // ===== Mutant-witnesses: write_canon Ctor alt counter arithmetic (lines 251/252) =====
    // *counter += 1 → *counter -= 1 or *counter *= 1 (never advances).
    // If counter never advances, all binders in the same alt get the same name (e.g. "v0 v0").
    // Test: a Ctor alt with 2 binders must produce 2 DISTINCT canonical names.
    #[test]
    fn format_ctor_alt_binders_get_distinct_sequential_names() {
        use crate::data::{CtorSpec, DataRegistry, DeclSpec};
        use std::collections::BTreeMap;

        // Build a simple 2-field constructor "Pair(a, b)".
        let mut m = BTreeMap::new();
        m.insert(
            "Pair".to_owned(),
            DeclSpec {
                ctors: vec![CtorSpec { fields: vec![] }],
            },
        );
        // We need a CtorRef to build a Ctor alt. Use 0 fields to avoid FieldTy complexity but
        // create a Match with two binders manually by repeating the var list trick.
        // Instead, use a 1-field ctor and a 2-binder alt via explicit binders list.
        // Actually, Node::Match Alt::Ctor has `binders: Vec<VarId>` — we set 2 manually.
        let reg = DataRegistry::build(&m).unwrap();
        let cref = reg.ctor_ref("Pair", 0).unwrap();

        let match_node = Node::Match {
            scrutinee: Box::new(Node::Const(byte())),
            alts: vec![crate::node::Alt::Ctor {
                ctor: cref,
                binders: vec!["a".into(), "b".into()], // two binders
                body: Node::Var("a".into()),           // body uses first binder
            }],
            default: None,
        };
        let text = format(&match_node);

        // The "alt #hash#0 (v0 v1)" line must have two DISTINCT canonical names.
        // CtorRef renders as "#<decl_hash>#<index>", not by name. So we search for "alt #".
        // Kills: *counter += 1 → *counter -= 1 or *= 1 (binders would all be "v0 v0").
        let alt_line = text
            .lines()
            .find(|l| l.trim_start().starts_with("alt #"))
            .expect("Ctor alt must produce an 'alt #hash#i (...)' line");
        // Both v0 and v1 must appear (distinct sequential names).
        assert!(
            alt_line.contains("v0"),
            "first binder must be 'v0': {alt_line:?}\nfull:\n{text}"
        );
        assert!(
            alt_line.contains("v1"),
            "second binder must be 'v1' (distinct from v0): {alt_line:?}\nfull:\n{text}"
        );
        // Ensure they are not the same name: "v0 v0" would indicate the counter never advanced.
        assert!(
            !alt_line.contains("v0 v0"),
            "binders must be distinct, not both 'v0': {alt_line:?}"
        );
    }

    // ===== Mutant-witnesses: write_core Op/Swap/Lam/App/Fix/Match child indentation =====
    // Covers survivors at lower.rs lines 345/350 (Let body/bound in write_core already tested),
    // 359 (Swap src), 364 (Construct args), 373 (Match scrutinee), 383/387 (alt bodies),
    // 395 (default body), 404 (Lam body), 408/409 (App func/arg), 413 (Fix body).
    #[test]
    fn dump_node_all_node_types_indent_children_at_depth_one() {
        // Op: args at depth 1 = "  " (2 spaces).
        let op_node = Node::Op {
            prim: "bit.not".into(),
            args: vec![Node::Const(byte())],
        };
        let op_text = dump_node(&op_node);
        let op_arg_line = op_text
            .lines()
            .find(|l| l.trim_start().starts_with("const"))
            .expect("dump_node Op must have a const arg line");
        assert!(
            op_arg_line.starts_with("  "),
            "dump_node Op arg at depth 1 must start with 2 spaces: {op_arg_line:?}"
        );
        assert!(
            !op_arg_line.starts_with("    "),
            "dump_node Op arg must not have 4 spaces: {op_arg_line:?}"
        );

        // Swap src: at depth 1 = "  ".
        let swap_node = Node::Swap {
            src: Box::new(Node::Const(byte())),
            target: Repr::Ternary { trits: 6 },
            policy: crate::ContentHash::parse("blake3:round_trip_safe").unwrap(),
        };
        let swap_text = dump_node(&swap_node);
        let swap_src_line = swap_text
            .lines()
            .find(|l| l.trim_start().starts_with("const"))
            .expect("dump_node Swap must have a const src line");
        assert!(
            swap_src_line.starts_with("  "),
            "dump_node Swap src at depth 1 must start with 2 spaces: {swap_src_line:?}"
        );
        assert!(
            !swap_src_line.starts_with("    "),
            "dump_node Swap src must not have 4 spaces: {swap_src_line:?}"
        );

        // Lam body: at depth 1 = "  ".
        let lam_node = Node::Lam {
            param: "x".into(),
            body: Box::new(Node::Const(byte())),
        };
        let lam_text = dump_node(&lam_node);
        let lam_body_line = lam_text
            .lines()
            .find(|l| l.trim_start().starts_with("const"))
            .expect("dump_node Lam must have a const body line");
        assert!(
            lam_body_line.starts_with("  "),
            "dump_node Lam body at depth 1 must start with 2 spaces: {lam_body_line:?}"
        );
        assert!(
            !lam_body_line.starts_with("    "),
            "dump_node Lam body must not have 4 spaces: {lam_body_line:?}"
        );

        // App: func and arg each at depth 1 = "  ".
        // func is a Lam (produces "lam x =>"), arg is a Const.
        let app_node = Node::App {
            func: Box::new(Node::Lam {
                param: "p".into(),
                body: Box::new(Node::Var("p".into())),
            }),
            arg: Box::new(Node::Const(byte())),
        };
        let app_text = dump_node(&app_node);
        let app_func_line = app_text
            .lines()
            .find(|l| l.trim_start().starts_with("lam p"))
            .expect("dump_node App must have a 'lam p' func line");
        assert!(
            app_func_line.starts_with("  "),
            "dump_node App func at depth 1 must start with 2 spaces: {app_func_line:?}"
        );
        assert!(
            !app_func_line.starts_with("    "),
            "dump_node App func must not have 4 spaces: {app_func_line:?}"
        );
        // The Const arg line.
        let app_arg_line = app_text
            .lines()
            .find(|l| l.trim_start().starts_with("const"))
            .expect("dump_node App must have a 'const' arg line");
        assert!(
            app_arg_line.starts_with("  "),
            "dump_node App arg at depth 1 must start with 2 spaces: {app_arg_line:?}"
        );
        assert!(
            !app_arg_line.starts_with("    "),
            "dump_node App arg must not have 4 spaces: {app_arg_line:?}"
        );

        // Fix body: at depth 1 = "  ".
        let fix_node = Node::Fix {
            name: "f".into(),
            body: Box::new(Node::Const(byte())),
        };
        let fix_text = dump_node(&fix_node);
        let fix_body_line = fix_text
            .lines()
            .find(|l| l.trim_start().starts_with("const"))
            .expect("dump_node Fix must have a const body line");
        assert!(
            fix_body_line.starts_with("  "),
            "dump_node Fix body at depth 1 must start with 2 spaces: {fix_body_line:?}"
        );
        assert!(
            !fix_body_line.starts_with("    "),
            "dump_node Fix body must not have 4 spaces: {fix_body_line:?}"
        );
    }

    // ===== Mutant-witnesses: write_core Match scrutinee/alt bodies/default indentation =====
    // Covers survivors at lower.rs lines 373 (scrutinee), 383 (Ctor alt body),
    // 387 (Lit alt body), 395 (default body).
    #[test]
    fn dump_node_match_indents_correctly() {
        // Match at depth 0: scrutinee, alt body, and default body must all be at depth 1 = "  ".
        let match_node = Node::Match {
            scrutinee: Box::new(Node::Const(byte())),
            alts: vec![crate::node::Alt::Lit {
                value: byte(),
                body: Node::Var("q".into()),
            }],
            default: Some(Box::new(Node::Const(byte()))),
        };
        let text = dump_node(&match_node);

        // Scrutinee (const) at depth 1 = "  ".
        let scrutinee_line = text
            .lines()
            .find(|l| l.trim_start().starts_with("const"))
            .expect("dump_node Match must have a const scrutinee line");
        assert!(
            scrutinee_line.starts_with("  "),
            "dump_node Match scrutinee at depth 1 must start with 2 spaces: {scrutinee_line:?}\nfull:\n{text}"
        );
        assert!(
            !scrutinee_line.starts_with("    "),
            "dump_node Match scrutinee must not have 4 spaces: {scrutinee_line:?}"
        );

        // Lit alt body (var "q") at depth 1 = "  ".
        let lit_body_line = text
            .lines()
            .find(|l| l.trim_start().starts_with("var q"))
            .expect("dump_node Match must have a 'var q' lit-alt body line");
        assert!(
            lit_body_line.starts_with("  "),
            "dump_node Lit alt body at depth 1 must start with 2 spaces: {lit_body_line:?}\nfull:\n{text}"
        );
        assert!(
            !lit_body_line.starts_with("    "),
            "dump_node Lit alt body must not have 4 spaces: {lit_body_line:?}"
        );

        // Default body at depth 1 = "  " — appears after the "default" keyword line.
        let lines: Vec<&str> = text.lines().collect();
        let default_idx = lines
            .iter()
            .position(|l| l.trim() == "default")
            .expect("dump_node Match must have a 'default' line");
        let default_body = lines[default_idx + 1];
        assert!(
            default_body.starts_with("  "),
            "dump_node default body at depth 1 must start with 2 spaces: {default_body:?}\nfull:\n{text}"
        );
        assert!(
            !default_body.starts_with("    "),
            "dump_node default body must not have 4 spaces: {default_body:?}"
        );
    }

    // ===== Mutant-witnesses: write_rhs Lam/Fix/FixGroup/Match indentation (lines 818–859) =====
    // write_rhs calls write_block at depth+1 for nested Anf bodies. If depth+1 → depth*1 (=0
    // at root), the inner "substrate {" appears at the same indent level as the outer, making
    // the nesting invisible. If depth+1 → depth-1 (0-1 underflows to 0), same symptom.
    //
    // write_rhs is invoked from Anf::write_block at depth+1 (line 885). So the outer substrate
    // is at depth 0, write_rhs at depth 1. A Rhs::Lam calls write_block at depth+1 = depth 2.
    // The inner substrate header "  ".repeat(2) = 4 spaces.
    //
    // Also covers line 885: write_rhs(&b.rhs, depth + 1, s) → depth*1 (=0 at root) shifts every
    // Rhs to depth 0, so the outer "substrate {}" at depth 0 and the rhs content also at depth 0
    // are indistinguishable.
    #[test]
    fn substrate_dump_all_rhs_types_indent_nested_blocks() {
        // --- Lam: outer substrate{} at depth 0, Rhs::Lam calls write_block at depth+1=2 ---
        let lam_node = Node::Lam {
            param: "x".into(),
            body: Box::new(Node::Const(byte())),
        };
        let lam_dump = lower_to_anf(&lam_node).dump();
        // The outer substrate header is at depth 0 (no leading spaces).
        let outer_sub = lam_dump
            .lines()
            .find(|l| l.trim() == "substrate {")
            .expect("Lam substrate dump must have an outer 'substrate {' line");
        assert!(
            !outer_sub.starts_with("  "),
            "outer substrate header must not be indented: {outer_sub:?}\nfull:\n{lam_dump}"
        );
        // The inner substrate header (Lam body) must be at depth 2 = "    " (4 leading spaces).
        // Kills: write_block(depth + 1, ...) → +1 replaced with *1 (=1 at depth 1 → only 2 spaces
        // instead of 4) or the outer write_rhs at depth+1 → *1 collapsing everything to depth 0.
        let inner_sub = lam_dump
            .lines()
            .filter(|l| l.trim() == "substrate {")
            .nth(1)
            .expect("Lam substrate dump must have an inner 'substrate {' for Lam body");
        assert!(
            inner_sub.starts_with("    "),
            "Lam body substrate at depth 2 must start with 4 spaces: {inner_sub:?}\nfull:\n{lam_dump}"
        );

        // --- Fix: same depth structure as Lam ---
        let fix_node = Node::Fix {
            name: "f".into(),
            body: Box::new(Node::Const(byte())),
        };
        let fix_dump = lower_to_anf(&fix_node).dump();
        let fix_inner_sub = fix_dump
            .lines()
            .filter(|l| l.trim() == "substrate {")
            .nth(1)
            .expect("Fix substrate dump must have an inner 'substrate {' for Fix body");
        assert!(
            fix_inner_sub.starts_with("    "),
            "Fix body substrate at depth 2 must start with 4 spaces: {fix_inner_sub:?}\nfull:\n{fix_dump}"
        );

        // --- FixGroup: the def bodies each get a nested substrate block ---
        let fg_node = Node::FixGroup {
            defs: vec![("g".into(), Box::new(Node::Const(byte())))],
            body: Box::new(Node::Var("g".into())),
        };
        let fg_dump = lower_to_anf(&fg_node).dump();
        // There must be a nested substrate block for the def body.
        let substrate_count = fg_dump.matches("substrate {").count();
        assert!(
            substrate_count >= 2,
            "FixGroup substrate dump must have at least 2 'substrate {{' blocks: {fg_dump:?}"
        );
        // The inner "substrate {" for the def body must be more indented than the outer.
        let fg_lines: Vec<&str> = fg_dump.lines().collect();
        let sub_positions: Vec<usize> = fg_lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.trim() == "substrate {")
            .map(|(i, _)| i)
            .collect();
        if sub_positions.len() >= 2 {
            let outer_indent =
                fg_lines[sub_positions[0]].len() - fg_lines[sub_positions[0]].trim_start().len();
            let inner_indent =
                fg_lines[sub_positions[1]].len() - fg_lines[sub_positions[1]].trim_start().len();
            assert!(
                inner_indent > outer_indent,
                "FixGroup inner substrate must be more indented than outer: outer={outer_indent}, inner={inner_indent}\n{fg_dump}"
            );
        }
    }

    // ===== Mutant-witnesses: write_rhs Match arm indentation (lines 847, 851, 859) =====
    // Match arms and default in write_rhs: the pad is "  ".repeat(depth + 1).
    // If depth+1 → depth*1 (=0 at root) or depth-1, the "alt" and "default" lines lose
    // their relative indent. Here depth=1 (from write_block), so depth+1=2 → 4 leading spaces
    // for alt lines vs 2 leading spaces for the outer "match" line.
    #[test]
    fn substrate_dump_match_rhs_arms_indented_relative_to_match() {
        // A Match with a Lit alt will lower to an Rhs::Match in the substrate.
        // The Match node itself is the result (scrutinee is a Const that becomes a temp).
        // Build: match (const byte) { alt-lit const_byte => Const(byte) | default => Var("_") }
        // This might not lower to an Rhs::Match cleanly; let's use a full Match node at root.
        let match_node = Node::Match {
            scrutinee: Box::new(Node::Const(byte())),
            alts: vec![crate::node::Alt::Lit {
                value: byte(),
                body: Node::Const(byte()),
            }],
            default: Some(Box::new(Node::Const(byte()))),
        };
        let dump = lower_to_anf(&match_node).dump();

        // The dump must contain the "match" keyword (from write_rhs for Rhs::Match).
        assert!(
            dump.contains("match "),
            "substrate dump of Match node must contain 'match': {dump:?}"
        );

        // The "alt-lit" line appears at depth+1 relative to the "match" line.
        // In the substrate, the outer substrate{} is at depth 0, write_rhs at depth 1 for
        // bindings, and the pad for alt lines is "  ".repeat(depth+1) = "  ".repeat(2) = 4 spaces.
        if let Some(alt_line) = dump.lines().find(|l| l.contains("alt-lit")) {
            let alt_indent = alt_line.len() - alt_line.trim_start().len();
            // Alt lines must have more than 2 leading spaces (they're nested inside the match).
            // At depth 1: pad = "  ".repeat(1+1) = "    " → 4 spaces.
            // If depth+1 → *1 (=1): pad = "  ".repeat(1) = "  " → 2 spaces (same as outer match line).
            assert!(
                alt_indent >= 4,
                "alt-lit line in substrate must have at least 4 leading spaces (depth+1=2): got {alt_indent} in {alt_line:?}\nfull:\n{dump}"
            );
        }

        // The default line similarly must have ≥ 4 leading spaces.
        if let Some(default_line) = dump.lines().find(|l| l.trim_start().starts_with("default")) {
            let default_indent = default_line.len() - default_line.trim_start().len();
            assert!(
                default_indent >= 4,
                "default line in substrate must have at least 4 leading spaces: got {default_indent} in {default_line:?}\nfull:\n{dump}"
            );
        }
    }

    // ===== Mutant-witness: Anf::write_block inner/result line at depth+1 (line 881, 891) =====
    // "  ".repeat(depth + 1) → "  ".repeat(depth * 1). At depth 1 (a nested write_block call),
    // depth+1=2 → 4 leading spaces for "result" and bindings; depth*1=1 → 2 leading spaces.
    // This test checks a nested Lam's inner substrate block has "result" at 6 spaces (depth=2+1=3
    // in the inner-inner sense), and more practically verifies that inside the outer substrate
    // block the binding line is indented relative to "substrate {".
    #[test]
    fn anf_write_block_result_line_indented_relative_to_substrate_header() {
        // A simple Const at the root: the outer write_block is at depth 0.
        // The "result %0" line must be at depth 1 = "  " (2 spaces).
        // "  ".repeat(0 + 1) = "  " → 2 spaces; "  ".repeat(0 * 1) = "" → 0 spaces (mutant).
        let simple = Node::Const(byte());
        let dump = lower_to_anf(&simple).dump();
        // Find the "result" line.
        let result_line = dump
            .lines()
            .find(|l| l.trim_start().starts_with("result "))
            .expect("substrate dump must have a 'result' line");
        assert!(
            result_line.starts_with("  "),
            "result line at depth 1 must start with 2 spaces: {result_line:?}\nfull:\n{dump}"
        );
        assert!(
            !result_line.starts_with("    "),
            "result line must not start with 4 spaces: {result_line:?}"
        );

        // For a Lam, there is a nested substrate block at depth 2.
        // write_block is called at depth=2 for the Lam body, so:
        //   inner = "  ".repeat(2+1) = "      " → 6 spaces for bindings and result inside the body.
        // The mutant "  ".repeat(depth*1) = "  ".repeat(2) = "    " → only 4 spaces.
        // We want the FIRST "result" line (inside the nested substrate), not the outer one.
        let lam = Node::Lam {
            param: "x".into(),
            body: Box::new(Node::Const(byte())),
        };
        let lam_dump = lower_to_anf(&lam).dump();
        // The actual layout (from the code):
        // substrate {                          ← depth=0, pad=""
        //   %1 = lam x =>                     ← depth=0, inner="  "
        //     substrate {                      ← depth=2, pad="    "
        //       %0 = const ...                 ← depth=2, inner="      "
        //       result %0                      ← depth=2, inner="      " (6 spaces) <-- FIRST result
        //   result %1                          ← depth=0, inner="  " (2 spaces)     <-- LAST result
        // }
        let nested_result = lam_dump
            .lines()
            .find(|l| l.trim_start().starts_with("result "))
            .expect("Lam substrate dump must have a nested 'result' line");
        // The first result is the nested one — inside the inner substrate block at depth 2.
        // It must have 6 leading spaces (depth=2, inner = "  ".repeat(3)).
        // Kills: "  ".repeat(depth+1) → "  ".repeat(depth*1): at depth 2 gives 4 not 6 spaces.
        assert!(
            nested_result.starts_with("      "),
            "nested result at inner depth (6 spaces) must come before outer result (2 spaces): {nested_result:?}\nfull:\n{lam_dump}"
        );
    }
}
