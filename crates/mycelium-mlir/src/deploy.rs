//! **The native-artifact descriptor a deployable Spore embeds** (M-620; ADR-013; ADR-003;
//! RFC-0004 §6; VR-4/VR-5/G2).
//!
//! ADR-013 makes a **Spore** the content-addressed deployable unit — a hash-identified DAG of code +
//! state + metadata. M-620 asks: produce a deployable Spore *from the native-compiled backend*. The
//! full Spore wire-format that embeds compiled native artifacts (and the workspace wiring that makes
//! `mycelium-spore` depend on this crate) is a **design-first decision** recorded in `DN-18`
//! (design-complete, impl-pending) — it is an ADR-level change, not a silent build detail. What this
//! module lands **now** is the buildable, crate-local primitive that the Spore layer will embed: an
//! inspectable, content-addressed [`NativeArtifact`] descriptor of one natively-compiled program,
//! carrying the **VR-4 no-opaque-lowering attestation** into the deployed unit.
//!
//! **Content-addressed identity (ADR-003).** A [`NativeArtifact`]'s identity **is the content hash of
//! the program it compiles** — supplied by the caller (the program's `ContentHash`), the same
//! Unison-style code identity ADR-003 fixes. Everything else the descriptor carries — the dumpable IR
//! text, the toolchain versions, the `EXPLAIN` — is **metadata, not identity** (two builds of the
//! *same* program on different LLVM patch versions are the *same* artifact identity; their IR text
//! may differ). [`NativeArtifact::id`] returns that canonical identity; [`NativeArtifact::same_identity_as`]
//! compares by it, ignoring metadata — exactly ADR-003's "metadata is not identity".
//!
//! **VR-4 carried into the deployment (the M-620↔M-630 seam).** The descriptor embeds the
//! [`crate::vr4`] cross-backend attestation (every backend's lowering is dumpable — no opaque pass)
//! **and** the program's own dumpable lowered IR, so the no-opaque-lowering guarantee (VR-4) travels
//! *with* the deployed unit and is inspectable at the deployment site, not just at build time
//! (RFC-0004 §6). [`NativeArtifact::explain`] renders the whole attestation.
//!
//! **Never-silent (G2).** A missing/ambiguous deploy input is an explicit [`DeployError`], never a
//! guessed default: a program the native backend cannot lower soundly is refused with the backend's
//! own `EXPLAIN`-able reason (routed to the proven path), never fragile codegen shipped to fill the
//! gap (G2/VR-5).
//!
//! **Honesty (VR-5).** The descriptor's guarantee is `Empirical` — the lowered IR is the real
//! artifact, its faithfulness evidenced by the differentials (M-302/M-602); never `Proven` (no
//! machine-checked end-to-end deployment-correctness theorem; G2/VR-5).

use mycelium_core::{ContentHash, GuaranteeStrength, Node};

use crate::llvm::{emit_llvm_ir, AotError};
use crate::vr4::{cross_backend_gate, CrossBackendGate};

/// Why producing a deployable native artifact failed — always explicit (G2), never a guessed default.
///
/// Note on "missing input" (G2, the stronger form): the deploy **identity** is a [`ContentHash`],
/// which is a *validated, non-empty* type — an empty/malformed identity is **unrepresentable**, so
/// there is no runtime "missing identity" branch to take. That is the strongest G2 posture: an
/// invalid input cannot be constructed, rather than being rejected after the fact (CLAUDE.md banked
/// guard 2). The remaining never-silent failure is a program the native backend cannot lower soundly.
#[derive(Debug, Clone, PartialEq)]
pub enum DeployError {
    /// The program is outside the fragment the native backend can lower soundly. Carries the
    /// backend's own `EXPLAIN`-able reason; the program runs on the proven (interpreter / richer)
    /// path — fragile codegen is **never** shipped to fill the gap (G2/VR-5).
    NotDeployable(String),
}

impl core::fmt::Display for DeployError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DeployError::NotDeployable(m) => {
                write!(
                    f,
                    "program is not natively deployable: {m} (runs on the proven path)"
                )
            }
        }
    }
}

impl std::error::Error for DeployError {}

/// The inspectable, content-addressed descriptor of one natively-compiled program — the unit a
/// deployable Spore embeds (M-620; ADR-013/ADR-003). Identity is the program's content hash; the
/// dumpable IR + VR-4 attestation are carried metadata (not identity, ADR-003).
#[derive(Debug, Clone, PartialEq)]
pub struct NativeArtifact {
    /// The **canonical identity**: the content hash of the program this artifact compiles (ADR-003
    /// code identity). Two builds of the same program are the same artifact, whatever the toolchain.
    identity: ContentHash,
    /// The program's **dumpable lowered LLVM IR** — the no-opaque-lowering evidence carried into the
    /// deployed unit (VR-4; RFC-0004 §6). Metadata, not identity (a different LLVM patch may render
    /// it differently for the same program).
    lowered_ir: String,
    /// The VR-4 **cross-backend attestation**: every backend's lowering is dumpable (no opaque pass).
    /// Carried so the guarantee travels with the deployment. Metadata, not identity.
    vr4: CrossBackendGate,
    /// The honest strength of "this artifact faithfully runs the program" — `Empirical` (the
    /// differentials), never `Proven` (VR-5).
    faithfulness: GuaranteeStrength,
}

impl NativeArtifact {
    /// Build the native-artifact descriptor for `node` under the content identity `identity`
    /// (the program's `ContentHash`, ADR-003 — supplied by the caller, *not* recomputed from the IR,
    /// because identity is the code's hash, not the lowering's text).
    ///
    /// Lowers `node` to dumpable LLVM IR via the direct-LLVM backend and records the VR-4
    /// cross-backend attestation. A program the backend cannot lower soundly is an explicit
    /// [`DeployError::NotDeployable`] (the backend's own refusal reason) — never fragile codegen
    /// (G2/VR-5). The `identity` is a validated, non-empty [`ContentHash`], so a "missing identity"
    /// is unrepresentable (the strongest G2 form — no defaulting possible). Returns
    /// [`DeployError::NotDeployable`] wrapping a `ToolchainMissing` when the compiler is absent so a
    /// caller can skip (the house idiom).
    pub fn build(node: &Node, identity: ContentHash) -> Result<Self, DeployError> {
        // The dumpable lowering is the artifact's no-opaque evidence; an out-of-fragment node (or a
        // missing toolchain) is an explicit refusal routed to the proven path — never fragile output.
        let lowered_ir = match emit_llvm_ir(node) {
            Ok(ir) => ir,
            Err(AotError::ToolchainMissing(t)) => {
                return Err(DeployError::NotDeployable(format!(
                    "native toolchain absent ({t}) — cannot produce a deployable native artifact here"
                )));
            }
            Err(e) => {
                return Err(DeployError::NotDeployable(e.to_string()));
            }
        };
        let vr4 = cross_backend_gate(node);
        Ok(NativeArtifact {
            identity,
            lowered_ir,
            vr4,
            // The artifact is a real compiled lowering; its faithfulness is evidenced by the
            // interp↔native differentials (M-302/M-602), never proven end-to-end (VR-5).
            faithfulness: GuaranteeStrength::Empirical,
        })
    }

    /// The canonical content-addressed identity (the program's hash; ADR-003). **This** is the
    /// artifact's identity — metadata is not.
    #[must_use]
    pub fn id(&self) -> &ContentHash {
        &self.identity
    }

    /// The dumpable lowered LLVM IR carried into the deployment (VR-4 evidence). Metadata.
    #[must_use]
    pub fn lowered_ir(&self) -> &str {
        &self.lowered_ir
    }

    /// The VR-4 cross-backend attestation travelling with the deployed unit (no opaque pass anywhere).
    #[must_use]
    pub fn vr4(&self) -> &CrossBackendGate {
        &self.vr4
    }

    /// The honest faithfulness strength — `Empirical` (the differentials), never `Proven` (VR-5).
    #[must_use]
    pub fn faithfulness(&self) -> GuaranteeStrength {
        self.faithfulness
    }

    /// Whether two artifacts have the **same content-addressed identity** (ADR-003) — i.e. compile the
    /// same program — **ignoring metadata** (the IR text, the attestation, the tool versions). This is
    /// the "metadata is not identity" comparison: two builds of the same program are equal here even
    /// if their carried IR differs byte-for-byte.
    #[must_use]
    pub fn same_identity_as(&self, other: &NativeArtifact) -> bool {
        self.identity == other.identity
    }

    /// A human-readable `EXPLAIN` of the deployable artifact: its content identity, the carried-IR
    /// size, the faithfulness tag, and the embedded VR-4 attestation — so the deployed unit's
    /// no-opaque-lowering guarantee is auditable at the deployment site (no black box; RFC-0004 §6).
    #[must_use]
    pub fn explain(&self) -> String {
        format!(
            "NativeArtifact (M-620 deployable; ADR-013/ADR-003):\n  identity: {} (content-addressed \
             code identity — metadata is NOT identity)\n  lowered LLVM IR: {} bytes (dumpable — VR-4 \
             evidence carried into the deployment)\n  faithfulness: {:?} (the differentials; never \
             Proven — VR-5)\n{}",
            self.identity.as_str(),
            self.lowered_ir.len(),
            self.faithfulness,
            self.vr4.explain(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_core::{Meta, Payload, Provenance, Repr, Trit, Value};

    fn byte(bits: [bool; 8]) -> Value {
        Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(bits.to_vec()),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    fn ident(tag: &str) -> ContentHash {
        // A well-formed blake3 identity placeholder (the program hash the caller supplies).
        ContentHash::parse(&format!("blake3:{tag}")).expect("valid id")
    }

    fn in_fragment() -> Node {
        Node::Op {
            prim: "bit.not".into(),
            args: vec![Node::Op {
                prim: "bit.xor".into(),
                args: vec![
                    Node::Const(byte([true, false, true, true, false, false, true, false])),
                    Node::Const(byte([false, false, true, false, true, false, true, true])),
                ],
            }],
        }
    }

    #[test]
    fn builds_a_deployable_artifact_carrying_the_vr4_attestation() {
        // M-620: a native artifact built from the in-fragment program carries its content identity,
        // dumpable IR (VR-4 evidence), and the cross-backend no-opaque attestation.
        let art = match NativeArtifact::build(&in_fragment(), ident("deadbeef00")) {
            Ok(a) => a,
            // The direct-LLVM emitter does not need a toolchain to *emit* IR, so this should succeed;
            // if a future change made it toolchain-gated, a skip would be the honest outcome.
            Err(DeployError::NotDeployable(m)) if m.contains("toolchain absent") => return,
            Err(e) => panic!("unexpected deploy error: {e}"),
        };
        assert_eq!(art.id().as_str(), "blake3:deadbeef00");
        assert!(
            !art.lowered_ir().is_empty(),
            "dumpable IR is carried (VR-4)"
        );
        // The VR-4 attestation covers all backends (no opaque pass) — the deployed guarantee.
        assert!(art.vr4().covered() >= 5);
        // Honest tag: Empirical, never Proven (VR-5).
        assert_eq!(art.faithfulness(), GuaranteeStrength::Empirical);
        // The EXPLAIN names the identity and the VR-4 obligation.
        let ex = art.explain();
        assert!(ex.contains("blake3:deadbeef00"));
        assert!(ex.contains("VR-4 no-opaque-lowering"));
    }

    #[test]
    fn identity_is_the_program_hash_and_metadata_is_not_identity() {
        // ADR-003: two artifacts with the SAME program identity are the same unit even if their
        // carried metadata (here, a different identity tag would change identity — so we hold the
        // identity fixed and confirm same_identity_as keys on it, not on the IR bytes).
        let a = NativeArtifact::build(&in_fragment(), ident("aaaa0000")).unwrap();
        let b = NativeArtifact::build(&in_fragment(), ident("aaaa0000")).unwrap();
        let c = NativeArtifact::build(&in_fragment(), ident("bbbb1111")).unwrap();
        assert!(
            a.same_identity_as(&b),
            "same program hash ⇒ same identity (ADR-003)"
        );
        assert!(
            !a.same_identity_as(&c),
            "different program hash ⇒ different identity"
        );
        // The carried IR is byte-identical for the same program here, but identity does NOT depend on
        // it — it is the supplied content hash (metadata is not identity).
        assert_eq!(a.id(), b.id());
    }

    #[test]
    fn a_missing_identity_is_unrepresentable_and_a_swap_is_an_explicit_refusal() {
        // G2 (the strongest form): the deploy identity is a validated, non-empty ContentHash, so an
        // empty/malformed identity cannot even be constructed — `ContentHash::parse` rejects it. We
        // pin that: an empty digest does not parse, so a "missing identity" is unrepresentable, not a
        // runtime branch (CLAUDE.md banked guard 2).
        assert!(
            ContentHash::parse("blake3:").is_none(),
            "an empty digest is unrepresentable"
        );
        // The remaining never-silent path: a Swap is outside the direct-LLVM bit subset ⇒ explicit
        // NotDeployable (routed to the proven path), never fragile codegen (G2/VR-5).
        let swap = Node::Swap {
            src: Box::new(Node::Const(byte([true; 8]))),
            target: Repr::Ternary { trits: 6 },
            policy: ContentHash::parse("blake3:round_trip_safe").unwrap(),
        };
        match NativeArtifact::build(&swap, ident("cccc2222")) {
            Err(DeployError::NotDeployable(m)) => {
                assert!(!m.is_empty(), "the refusal carries an EXPLAIN-able reason");
            }
            Ok(_) => panic!("a Swap must not be natively deployable (out of fragment)"),
        }
    }

    #[test]
    fn a_trit_carry_op_deploys_on_the_direct_llvm_native_path() {
        // The direct-LLVM backend (M-301) DOES natively compile trit carry arithmetic
        // (`trit.add/sub/mul` over Ternary{m}) — so it is genuinely deployable, and the artifact
        // carries its dumpable IR + the VR-4 attestation. (The MLIR-dialect fragment refuses trit
        // carry and routes it here — that boundary lives in vr4.rs, not in the deploy artifact, which
        // uses the richer direct-LLVM backend.) This pins that the native deploy path covers more than
        // the element-wise fragment, honestly.
        let add = Node::Op {
            prim: "trit.add".into(),
            args: vec![
                Node::Const(
                    Value::new(
                        Repr::Ternary { trits: 3 },
                        Payload::Trits(vec![Trit::Pos, Trit::Neg, Trit::Zero]),
                        Meta::exact(Provenance::Root),
                    )
                    .unwrap(),
                ),
                Node::Const(
                    Value::new(
                        Repr::Ternary { trits: 3 },
                        Payload::Trits(vec![Trit::Zero, Trit::Pos, Trit::Pos]),
                        Meta::exact(Provenance::Root),
                    )
                    .unwrap(),
                ),
            ],
        };
        let art = NativeArtifact::build(&add, ident("dddd3333")).expect("trit.add is deployable");
        assert_eq!(art.id().as_str(), "blake3:dddd3333");
        assert!(!art.lowered_ir().is_empty(), "carries dumpable IR (VR-4)");
        assert_eq!(art.faithfulness(), GuaranteeStrength::Empirical);
    }

    #[test]
    fn the_artifact_explain_is_deterministic_for_deployment() {
        // The deployed attestation must be byte-deterministic so its identity is stable across runs /
        // machines (ADR-003 content-addressing).
        let a = NativeArtifact::build(&in_fragment(), ident("eeee4444")).unwrap();
        let b = NativeArtifact::build(&in_fragment(), ident("eeee4444")).unwrap();
        assert_eq!(a.explain(), b.explain());
    }
}
