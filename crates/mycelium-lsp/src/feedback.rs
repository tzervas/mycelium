//! The **LSP feedback facade** (M-140; FR-S5; Foundation §5.8; SC-5 channel).
//!
//! One call, one surface, the **four** semantic-feedback artifact kinds the dual-intelligibility
//! goal delivers (the same surface serves human IDEs and AI co-authors):
//!
//! 1. **typecheck/invariant diagnostics** — from the linter ([`crate::lint`]);
//! 2. **swap certificates** — the inspectable `SwapCertificate` for each statically-resolvable swap
//!    site (`mycelium-cert`);
//! 3. **bound/guarantee annotations** — the per-value honesty tag + bound (RFC-0001 §4.3/§4.7);
//! 4. **lowering-stage dumps** — the dumpable/diffable stages (`mycelium-core::lower`, M-112).
//!
//! This is the **skeleton**: a programmatic in-process surface (a scripted client drives
//! [`analyze`] and reads the four channels). Wrapping it in the LSP wire protocol over stdio is a
//! later, mechanical step.

use mycelium_cert::{binary_to_ternary, ternary_to_binary, SwapCertificate};
use mycelium_core::lower::{self, Stage};
use mycelium_core::{Bound, GuaranteeStrength, Node, Repr};

use crate::lint::{self, Diagnostic};

/// A per-value honesty annotation: where it is, its guarantee tag, and its bound (if approximate).
#[derive(Debug, Clone, PartialEq)]
pub struct GuaranteeAnnotation {
    /// Breadcrumb to the value.
    pub at: String,
    /// The disclosed guarantee strength.
    pub guarantee: GuaranteeStrength,
    /// The bound, if the value is approximate.
    pub bound: Option<Bound>,
}

/// A swap site and the certificate it emits (when statically resolvable).
#[derive(Debug, Clone, PartialEq)]
pub struct SwapSite {
    /// Breadcrumb to the swap.
    pub at: String,
    /// The target representation.
    pub target: Repr,
    /// The emitted certificate, or `None` when the source is not a statically-known value or the
    /// pair is not supported (the reason is surfaced as a diagnostic, never silent).
    pub certificate: Option<SwapCertificate>,
}

/// The aggregated feedback surface (SC-5 channel) for one Core IR program.
#[derive(Debug, Clone, PartialEq)]
pub struct Feedback {
    /// (1) Typecheck/invariant diagnostics.
    pub diagnostics: Vec<Diagnostic>,
    /// (3) Per-value bound/guarantee annotations.
    pub guarantees: Vec<GuaranteeAnnotation>,
    /// (2) Swap certificates, one entry per swap site.
    pub swaps: Vec<SwapSite>,
    /// (4) Lowering-stage dumps.
    pub stages: Vec<Stage>,
}

/// Analyze a Core IR program and return all four feedback artifact kinds over one surface.
#[must_use]
pub fn analyze(node: &Node) -> Feedback {
    let mut diagnostics = lint::lint(node);
    let mut guarantees = Vec::new();
    let mut swaps = Vec::new();
    collect(node, "", &mut guarantees, &mut swaps, &mut diagnostics);
    Feedback {
        diagnostics,
        guarantees,
        swaps,
        stages: lower::stages(node),
    }
}

fn here(prefix: &str, step: &str) -> String {
    if prefix.is_empty() {
        step.to_owned()
    } else {
        format!("{prefix}/{step}")
    }
}

fn collect(
    node: &Node,
    prefix: &str,
    g: &mut Vec<GuaranteeAnnotation>,
    sw: &mut Vec<SwapSite>,
    diags: &mut Vec<Diagnostic>,
) {
    match node {
        Node::Const(v) => {
            g.push(GuaranteeAnnotation {
                at: here(prefix, "const"),
                guarantee: v.meta().guarantee(),
                bound: v.meta().bound().cloned(),
            });
        }
        Node::Var(_) => {}
        Node::Let { id, bound, body } => {
            let at = here(prefix, &format!("let {id}"));
            collect(bound, &at, g, sw, diags);
            collect(body, &at, g, sw, diags);
        }
        Node::Op { prim, args } => {
            let at = here(prefix, &format!("op {prim}"));
            for a in args {
                collect(a, &at, g, sw, diags);
            }
        }
        Node::Swap {
            src,
            target,
            policy,
        } => {
            let at = here(prefix, "swap");
            // Resolve a certificate when the source is a statically-known constant value.
            let certificate = match src.as_ref() {
                Node::Const(v) => {
                    let result = match (v.repr(), target) {
                        (Repr::Binary { .. }, Repr::Ternary { trits }) => {
                            Some(binary_to_ternary(v, *trits, policy))
                        }
                        (Repr::Ternary { .. }, Repr::Binary { width }) => {
                            Some(ternary_to_binary(v, *width, policy))
                        }
                        _ => None,
                    };
                    match result {
                        Some(Ok((_, cert))) => Some(cert),
                        Some(Err(e)) => {
                            // Never silent: a failed/illegal swap surfaces as a diagnostic.
                            diags.push(Diagnostic {
                                code: "swap-error",
                                severity: crate::lint::Severity::Error,
                                at: at.clone(),
                                message: e.to_string(),
                            });
                            None
                        }
                        None => None,
                    }
                }
                _ => None,
            };
            sw.push(SwapSite {
                at: at.clone(),
                target: target.clone(),
                certificate,
            });
            collect(src, &at, g, sw, diags);
        }
    }
}
