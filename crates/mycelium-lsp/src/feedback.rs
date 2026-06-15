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
//! Since **M-221** the facade also surfaces the fifth kind, **selection EXPLAIN traces**
//! (RFC-0005 §2.2/§4; SC-5): [`analyze_with`] takes a [`PolicyRegistry`] and, at every swap site
//! whose `PolicyRef` resolves and whose source is statically known, re-derives the deterministic
//! [`Explanation`] — answering *"why was this representation chosen?"* in-editor. When the policy's
//! own choice disagrees with the node's recorded target, a `policy-divergence` warning surfaces it
//! (an override or a stale policy — visible either way, never silent).
//!
//! This is the in-process surface: a scripted client drives [`analyze`]/[`analyze_with`] and reads
//! the channels. The LSP **wire protocol** over stdio (JSON-RPC framing + LSP-shaped diagnostics +
//! the lifecycle handshake) lives in [`crate::wire`] (M-310); the remaining gap to a full server is
//! document sync, which needs a text → `Node` path (the L1 surface, M-320).

use mycelium_cert::{binary_to_ternary, ternary_to_binary, SwapCertificate};
use mycelium_core::lower::{self, Stage};
use mycelium_core::{Bound, GuaranteeStrength, Node, Repr};
use mycelium_select::{explain, Candidate, Explanation, PolicyRegistry, SelectionInputs};

use crate::lint::{self, Diagnostic, Severity};

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
    /// The emitted certificate, or `None` when the source is not a statically-known value (so no
    /// certificate *can* be derived here), or when the swap is statically known but failed or has
    /// no implemented certifier. In the latter two cases the reason is surfaced as a diagnostic
    /// (`swap-error` / `unsupported-swap-pair`) — `None` is never silent for a known source.
    pub certificate: Option<SwapCertificate>,
}

/// A surfaced selection EXPLAIN (M-221; RFC-0005 §4): the swap site and the re-derived trace.
#[derive(Debug, Clone, PartialEq)]
pub struct ExplainSite {
    /// Breadcrumb to the swap whose selection this explains.
    pub at: String,
    /// The deterministic EXPLAIN record (same `Meta` in → same trace out).
    pub explanation: Explanation,
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
    /// (5) Selection EXPLAIN traces (M-221) — one per swap site whose `PolicyRef` resolves in the
    /// registry handed to [`analyze_with`]; empty under plain [`analyze`].
    pub explanations: Vec<ExplainSite>,
}

/// A structured, at-a-glance rollup of a [`Feedback`] (M-310): per-artifact-kind counts and the
/// diagnostic severity breakdown, plus the worst severity present. This is the machine-navigable
/// health signal an AI co-author's feedback loop (SC-5b / E3-2) or an IDE status line consumes
/// without re-walking the channels — the "rich diagnostics" maturation of the M-140 facade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackSummary {
    /// Count of `Error`-severity diagnostics.
    pub errors: usize,
    /// Count of `Warning`-severity diagnostics.
    pub warnings: usize,
    /// Count of per-value guarantee annotations (kind 3).
    pub guarantees: usize,
    /// Count of swap sites (kind 2).
    pub swaps: usize,
    /// Count of lowering-stage dumps (kind 4).
    pub stages: usize,
    /// Count of selection EXPLAIN traces (kind 5).
    pub explanations: usize,
    /// The worst diagnostic severity present, if any (`Error` outranks `Warning`).
    pub worst: Option<Severity>,
}

impl FeedbackSummary {
    /// Clean = no `Error`-severity diagnostics — the gate [`crate::lint::has_errors`] checks, lifted
    /// to the whole feedback surface.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.errors == 0
    }
}

impl Feedback {
    /// Summarize this feedback into a [`FeedbackSummary`] (M-310). Deterministic.
    #[must_use]
    pub fn summary(&self) -> FeedbackSummary {
        let errors = self
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        let warnings = self
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count();
        // `Error` outranks `Warning`; `None` when there are no diagnostics.
        let worst = if errors > 0 {
            Some(Severity::Error)
        } else if warnings > 0 {
            Some(Severity::Warning)
        } else {
            None
        };
        FeedbackSummary {
            errors,
            warnings,
            guarantees: self.guarantees.len(),
            swaps: self.swaps.len(),
            stages: self.stages.len(),
            explanations: self.explanations.len(),
            worst,
        }
    }
}

/// Analyze a Core IR program and return the feedback artifact kinds over one surface. EXPLAIN
/// traces need a policy registry — use [`analyze_with`] to surface them.
#[must_use]
pub fn analyze(node: &Node) -> Feedback {
    analyze_with(node, &PolicyRegistry::new())
}

/// [`analyze`], plus the **EXPLAIN channel** (M-221; SC-5): every swap site whose `PolicyRef`
/// resolves in `policies` and whose source is statically known gets its selection re-derived and
/// surfaced; a disagreement between the policy's choice and the node's recorded target raises a
/// `policy-divergence` warning (override or stale policy — surfaced, never silent).
#[must_use]
pub fn analyze_with(node: &Node, policies: &PolicyRegistry) -> Feedback {
    let mut diagnostics = lint::lint(node);
    let mut guarantees = Vec::new();
    let mut swaps = Vec::new();
    let mut explanations = Vec::new();
    let mut cx = Collect {
        policies,
        g: &mut guarantees,
        sw: &mut swaps,
        ex: &mut explanations,
        diags: &mut diagnostics,
    };
    collect(node, "", &mut cx);
    Feedback {
        diagnostics,
        guarantees,
        swaps,
        stages: lower::stages(node),
        explanations,
    }
}

fn here(prefix: &str, step: &str) -> String {
    if prefix.is_empty() {
        step.to_owned()
    } else {
        format!("{prefix}/{step}")
    }
}

/// The traversal state — bundled so the walk stays one recursive function.
struct Collect<'a> {
    policies: &'a PolicyRegistry,
    g: &'a mut Vec<GuaranteeAnnotation>,
    sw: &'a mut Vec<SwapSite>,
    ex: &'a mut Vec<ExplainSite>,
    diags: &'a mut Vec<Diagnostic>,
}

fn collect(node: &Node, prefix: &str, cx: &mut Collect<'_>) {
    match node {
        Node::Const(v) => {
            cx.g.push(GuaranteeAnnotation {
                at: here(prefix, "const"),
                guarantee: v.meta().guarantee(),
                bound: v.meta().bound().cloned(),
            });
        }
        Node::Var(_) => {}
        Node::Let { id, bound, body } => {
            let at = here(prefix, &format!("let {id}"));
            collect(bound, &at, cx);
            collect(body, &at, cx);
        }
        Node::Op { prim, args } => {
            let at = here(prefix, &format!("op {prim}"));
            for a in args {
                collect(a, &at, cx);
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
                            cx.diags.push(Diagnostic {
                                code: "swap-error",
                                severity: crate::lint::Severity::Error,
                                at: at.clone(),
                                message: e.to_string(),
                            });
                            None
                        }
                        // The source is a statically-known value, but this swap pair has no
                        // implemented certifier yet (e.g. Binary→Dense). That is *not* the same as
                        // "source not statically known" — silently returning `None` would hide a
                        // missing-coverage gap. Surface it explicitly (never silent).
                        None => {
                            cx.diags.push(Diagnostic {
                                code: "unsupported-swap-pair",
                                severity: crate::lint::Severity::Error,
                                at: at.clone(),
                                message: format!(
                                    "no certified swap is implemented for {:?} → {target:?}; the \
                                     certificate channel is empty for this site (not silent)",
                                    v.repr()
                                ),
                            });
                            None
                        }
                    }
                }
                _ => None,
            };
            // The EXPLAIN channel (M-221): re-derive the selection when the policy resolves and
            // the source value is statically known (deterministic — same Meta, same trace).
            if let (Some(p), Node::Const(v)) = (cx.policies.get(policy), src.as_ref()) {
                let explanation = explain(p, &SelectionInputs::of_value(v));
                if !matches!(&explanation.chosen, Candidate::Repr(r) if r == target) {
                    cx.diags.push(Diagnostic {
                        code: "policy-divergence",
                        severity: crate::lint::Severity::Warning,
                        at: at.clone(),
                        message: format!(
                            "the recorded policy would choose {:?}, but the node's target is \
                             {target:?} (an override or a stale policy — verify which)",
                            explanation.chosen
                        ),
                    });
                }
                cx.ex.push(ExplainSite {
                    at: at.clone(),
                    explanation,
                });
            }
            cx.sw.push(SwapSite {
                at: at.clone(),
                target: target.clone(),
                certificate,
            });
            collect(src, &at, cx);
        }
        // r3 (RFC-0011): the data nodes are Repr-transparent (no swap, WF8) — the feedback walk
        // simply recurses into their children so guarantee/swap/EXPLAIN sites beneath them surface.
        Node::Construct { ctor, args } => {
            let at = here(prefix, &format!("construct {ctor}"));
            for a in args {
                collect(a, &at, cx);
            }
        }
        Node::Match {
            scrutinee,
            alts,
            default,
        } => {
            let at = here(prefix, "match");
            collect(scrutinee, &at, cx);
            for alt in alts {
                match alt {
                    mycelium_core::Alt::Ctor { ctor, body, .. } => {
                        collect(body, &here(&at, &format!("alt {ctor}")), cx);
                    }
                    mycelium_core::Alt::Lit { body, .. } => {
                        collect(body, &here(&at, "alt-lit"), cx);
                    }
                }
            }
            if let Some(d) = default {
                collect(d, &here(&at, "default"), cx);
            }
        }
    }
}
