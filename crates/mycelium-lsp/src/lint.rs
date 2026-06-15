//! The **invariant linter** (M-141; SC-3; G2; FR-M3; VR-5).
//!
//! Static, inspectable checks over a Core IR [`Node`] that surface the house honesty rules as
//! [`Diagnostic`]s for authoring tools (the LSP, M-140). The lints:
//!
//! - **`implicit-swap`** (error) — an `Op` whose `Const` operands span *more than one paradigm*
//!   (binary/ternary/dense/vsa). A representation change must be an explicit [`Node::Swap`], never
//!   implied by feeding mixed-paradigm operands to an op (FR-M3; SC-3 "no implicit conversion").
//! - **`unverified-bound`** (warning) — a value carrying a `Declared` guarantee. A user-asserted,
//!   unvalidated bound must *always* be surfaced; it is never silently trusted (M-I4; VR-5).
//! - **`placeholder-policy`** (error) — a [`Node::Swap`] whose `policy` is a stub (an all-zero
//!   digest, or `todo`/`tbd`/`none`/`placeholder`) rather than a real `PolicyRef` (G2: a swap's
//!   selection must be reified, not faked).
//! - **`free-variable`** (error) — a `Var` with no enclosing binder (an open term the interpreter
//!   cannot run).
//!
//! Note WF1 (only `Swap` changes a representation) and WF2 (every `Swap` carries a `PolicyRef`) are
//! enforced *by construction* in the `Node` grammar, so a literally repr-changing non-`Swap` node or
//! a policy-less swap is unrepresentable; these lints catch the *spirit* of those rules at the level
//! where authoring mistakes actually occur (mixed-paradigm ops, stub policies).

use mycelium_core::{GuaranteeStrength, Node, Repr, Value};

/// Severity of a [`Diagnostic`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// A house-rule violation that should block (honesty / never-silent).
    Error,
    /// An advisory the author must see (e.g. an unverified `Declared` value).
    Warning,
}

/// A single lint finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Stable lint code (e.g. `"implicit-swap"`).
    pub code: &'static str,
    /// Severity.
    pub severity: Severity,
    /// A breadcrumb path to the offending node (e.g. `"let a/swap/op f"`).
    pub at: String,
    /// Human-readable explanation.
    pub message: String,
}

impl Diagnostic {
    /// The breadcrumb [`Self::at`] as a structured, navigable path (split on `/`) — so a client can
    /// locate the offending node step-by-step rather than parsing the string (M-310). An empty
    /// breadcrumb (the program root) yields an empty path.
    #[must_use]
    pub fn path(&self) -> Vec<&str> {
        if self.at.is_empty() {
            Vec::new()
        } else {
            self.at.split('/').collect()
        }
    }
}

/// Lint a (closed) Core IR program, returning all findings in deterministic traversal order.
#[must_use]
pub fn lint(node: &Node) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut scope: Vec<&str> = Vec::new();
    walk(node, "", &mut scope, &mut out);
    out
}

/// Whether `lint` found at least one `Error`-severity diagnostic.
#[must_use]
pub fn has_errors(diags: &[Diagnostic]) -> bool {
    diags.iter().any(|d| d.severity == Severity::Error)
}

fn paradigm(repr: &Repr) -> &'static str {
    match repr {
        Repr::Binary { .. } => "binary",
        Repr::Ternary { .. } => "ternary",
        Repr::Dense { .. } => "dense",
        Repr::Vsa { .. } => "vsa",
    }
}

/// A policy reference that is a stub rather than a real reified policy.
fn is_placeholder_policy(policy: &mycelium_core::ContentHash) -> bool {
    let d = policy.digest();
    d.bytes().all(|b| b == b'0') || matches!(d, "todo" | "tbd" | "none" | "placeholder")
}

fn here(prefix: &str, step: &str) -> String {
    if prefix.is_empty() {
        step.to_owned()
    } else {
        format!("{prefix}/{step}")
    }
}

fn check_value(v: &Value, at: &str, out: &mut Vec<Diagnostic>) {
    if v.meta().guarantee() == GuaranteeStrength::Declared {
        out.push(Diagnostic {
            code: "unverified-bound",
            severity: Severity::Warning,
            at: at.to_owned(),
            message: "value carries a Declared (user-asserted, unvalidated) bound — surface it; \
                      never trust it silently (VR-5/M-I4)"
                .to_owned(),
        });
    }
}

fn walk<'a>(node: &'a Node, prefix: &str, scope: &mut Vec<&'a str>, out: &mut Vec<Diagnostic>) {
    match node {
        Node::Const(v) => check_value(v, &here(prefix, "const"), out),
        Node::Var(x) => {
            if !scope.iter().rev().any(|b| b == x) {
                out.push(Diagnostic {
                    code: "free-variable",
                    severity: Severity::Error,
                    at: here(prefix, &format!("var {x}")),
                    message: format!("`{x}` is not bound by any enclosing `let` (open term)"),
                });
            }
        }
        Node::Let { id, bound, body } => {
            let at = here(prefix, &format!("let {id}"));
            walk(bound, &at, scope, out);
            scope.push(id);
            walk(body, &at, scope, out);
            scope.pop();
        }
        Node::Op { prim, args } => {
            let at = here(prefix, &format!("op {prim}"));
            // implicit-swap: mixed-paradigm Const operands imply a conversion the author must make
            // explicit with a Swap.
            let mut paradigms: Vec<&str> = args
                .iter()
                .filter_map(|a| match a {
                    Node::Const(v) => Some(paradigm(v.repr())),
                    _ => None,
                })
                .collect();
            paradigms.sort_unstable();
            paradigms.dedup();
            if paradigms.len() > 1 {
                out.push(Diagnostic {
                    code: "implicit-swap",
                    severity: Severity::Error,
                    at: at.clone(),
                    message: format!(
                        "op `{prim}` mixes paradigms [{}] — insert an explicit `swap` (no implicit conversion; FR-M3/SC-3)",
                        paradigms.join(", ")
                    ),
                });
            }
            for a in args {
                walk(a, &at, scope, out);
            }
        }
        Node::Swap {
            src,
            target,
            policy,
        } => {
            let at = here(prefix, &format!("swap -> {}", paradigm(target)));
            if is_placeholder_policy(policy) {
                out.push(Diagnostic {
                    code: "placeholder-policy",
                    severity: Severity::Error,
                    at: at.clone(),
                    message: format!(
                        "swap references a placeholder policy `{}` — a swap must cite a real reified PolicyRef (G2)",
                        policy.as_str()
                    ),
                });
            }
            walk(src, &at, scope, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_core::{ContentHash, Meta, Payload, Provenance, ScalarKind, Trit};

    fn binary8() -> Value {
        Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(vec![true; 8]),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    fn ternary6() -> Value {
        Value::new(
            Repr::Ternary { trits: 6 },
            Payload::Trits(vec![Trit::Zero; 6]),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    fn declared() -> Value {
        Value::new(
            Repr::Dense {
                dim: 1,
                dtype: ScalarKind::F32,
            },
            Payload::Scalars(vec![1.0]),
            Meta::new(
                Provenance::Root,
                GuaranteeStrength::Declared,
                Some(mycelium_core::Bound {
                    kind: mycelium_core::BoundKind::Probability { delta: 0.1 },
                    basis: mycelium_core::BoundBasis::UserDeclared,
                }),
                None,
                None,
                None,
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn codes(diags: &[Diagnostic]) -> Vec<&str> {
        diags.iter().map(|d| d.code).collect()
    }

    // --- implicit-swap: positive + negative ---

    #[test]
    fn implicit_swap_fires_on_mixed_paradigms() {
        let node = Node::Op {
            prim: "f".into(),
            args: vec![Node::Const(binary8()), Node::Const(ternary6())],
        };
        let d = lint(&node);
        assert!(codes(&d).contains(&"implicit-swap"));
        assert!(has_errors(&d));
    }

    #[test]
    fn implicit_swap_clean_on_same_paradigm() {
        let node = Node::Op {
            prim: "f".into(),
            args: vec![Node::Const(binary8()), Node::Const(binary8())],
        };
        assert!(!codes(&lint(&node)).contains(&"implicit-swap"));
    }

    // --- unverified-bound: positive + negative ---

    #[test]
    fn unverified_bound_fires_on_declared() {
        let d = lint(&Node::Const(declared()));
        assert_eq!(codes(&d), vec!["unverified-bound"]);
        assert_eq!(d[0].severity, Severity::Warning);
    }

    #[test]
    fn unverified_bound_clean_on_exact() {
        assert!(lint(&Node::Const(binary8())).is_empty());
    }

    // --- placeholder-policy: positive + negative ---

    fn swap_with(policy: &str) -> Node {
        Node::Swap {
            src: Box::new(Node::Const(binary8())),
            target: Repr::Ternary { trits: 6 },
            policy: ContentHash::parse(policy).unwrap(),
        }
    }

    #[test]
    fn placeholder_policy_fires_on_stub() {
        assert!(codes(&lint(&swap_with("blake3:00000000"))).contains(&"placeholder-policy"));
        assert!(codes(&lint(&swap_with("policy:todo"))).contains(&"placeholder-policy"));
    }

    #[test]
    fn placeholder_policy_clean_on_real_ref() {
        assert!(!codes(&lint(&swap_with("blake3:Hh3kQ_x-1A"))).contains(&"placeholder-policy"));
    }

    // --- free-variable: positive + negative ---

    #[test]
    fn free_variable_fires_when_unbound() {
        let d = lint(&Node::Var("x".into()));
        assert_eq!(codes(&d), vec!["free-variable"]);
    }

    #[test]
    fn free_variable_clean_when_bound() {
        let node = Node::Let {
            id: "x".into(),
            bound: Box::new(Node::Const(binary8())),
            body: Box::new(Node::Var("x".into())),
        };
        assert!(lint(&node).is_empty());
    }

    #[test]
    fn lint_is_deterministic() {
        let node = Node::Op {
            prim: "f".into(),
            args: vec![Node::Const(binary8()), Node::Const(ternary6())],
        };
        assert_eq!(lint(&node), lint(&node));
    }

    #[test]
    fn scoping_respected_in_nested_lets() {
        // `y` is free in the body even though `x` is bound.
        let node = Node::Let {
            id: "x".into(),
            bound: Box::new(Node::Const(binary8())),
            body: Box::new(Node::Var("y".into())),
        };
        assert_eq!(codes(&lint(&node)), vec!["free-variable"]);
    }

    #[test]
    fn diagnostic_path_is_the_navigable_breadcrumb() {
        // M-310: the `at` breadcrumb splits into a structured, navigable path.
        let d = Diagnostic {
            code: "x",
            severity: Severity::Warning,
            at: "let a/swap/op f".to_owned(),
            message: String::new(),
        };
        assert_eq!(d.path(), vec!["let a", "swap", "op f"]);
        // An empty breadcrumb (the program root) yields an empty path, not `[""]`.
        let root = Diagnostic {
            at: String::new(),
            ..d.clone()
        };
        assert!(root.path().is_empty());
    }
}
