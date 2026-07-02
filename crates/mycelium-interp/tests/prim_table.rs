//! R7-Q4 (M-390) equivalence guard: the interpreter's intrinsic-guarantee assumption must agree
//! with the content-addressed prim table `Π` (`mycelium_core::PrimTable`). DN-10 §3.4:
//! `Π_new(hash(p)) = Π_old(name(p))` — here at the *intrinsic guarantee* level.
//!
//! The interpreter threads a hard-coded `intrinsic = Exact` through `compose_result`
//! (`crates/mycelium-interp/src/prims.rs`) for every **scalar** built-in. The **tensor-valued**
//! `dense.*` group (M-890) does *not* route through `compose_result`: the kernel
//! (`mycelium-dense`) constructs the result `Value` with its own per-op tag
//! (`DenseSpace::op_guarantee` — `dense.neg` `Exact`, the rest `Proven`), and the wrapper carries
//! it unchanged (VR-5). These tests pin both halves to the registry so none can drift: a scalar
//! prim declaring a non-`Exact` intrinsic in the table would make `compose_result`'s hard-coded
//! `Exact` dishonest (the signal to rewire it to read the table — the flagged follow-up), and a
//! `dense.*` table entry disagreeing with the kernel's `op_guarantee` would break the
//! carried-not-upgraded contract. The **model-dispatched** `vsa.*` bind group (M-892) likewise
//! bypasses `compose_result` — the `mycelium-vsa` kernel constructs the result with its
//! per-*model* tag — so its Π entry is pinned to the **meet over the dispatch set** (MAP-I/FHRR/
//! BSC), recomputed from the kernel (VR-5: one table slot must not over-claim for any model).

use mycelium_core::{GuaranteeStrength, PrimTable};
use mycelium_dense::{DenseOp, DenseSpace};
use mycelium_interp::PrimRegistry;
use mycelium_vsa::{Bsc, Fhrr, MapI, VsaModel, VsaOp};

#[test]
fn interp_builtin_names_match_the_prim_table() {
    let interp_reg = PrimRegistry::with_builtins();
    let table_reg = PrimTable::builtins();
    let mut interp = interp_reg.names();
    let mut table = table_reg.names();
    interp.sort_unstable();
    table.sort_unstable();
    assert_eq!(
        interp, table,
        "the interpreter dispatch set and the content-addressed prim table must list the same \
         kernel prims (no drift between the executable table and the declared Π)"
    );
}

#[test]
fn every_interp_builtin_intrinsic_matches_its_composition_path() {
    // Two composition paths, one source of truth (the table):
    //   - scalar prims route through `compose_result`, whose intrinsic is hard-coded `Exact` —
    //     so their table entry must say `Exact` (else the hard-coding is dishonest);
    //   - the tensor-valued `dense.*` prims (M-890) carry the KERNEL's tag — so their table entry
    //     must equal `DenseSpace::op_guarantee` verbatim (VR-5: carried, never upgraded).
    let table = PrimTable::builtins();
    let dense_kernel_tag = |name: &str| -> Option<GuaranteeStrength> {
        let op = match name {
            "dense.add" => DenseOp::Add,
            "dense.sub" => DenseOp::Sub,
            "dense.neg" => DenseOp::Neg,
            "dense.scale" => DenseOp::Scale,
            // M-891: the measurement pair — Proven (the binary64 accumulation bound).
            "dense.dot" => DenseOp::Dot,
            "dense.similarity" => DenseOp::Similarity,
            _ => return None,
        };
        Some(DenseSpace::op_guarantee(op))
    };
    // M-892: the VSA bind group's kernel ops (bundle/cleanup ride M-893/M-894).
    let vsa_op = |name: &str| -> Option<VsaOp> {
        match name {
            "vsa.bind" => Some(VsaOp::Bind),
            "vsa.unbind" => Some(VsaOp::Unbind),
            "vsa.permute" => Some(VsaOp::Permute),
            _ => None,
        }
    };
    for name in PrimRegistry::with_builtins().names() {
        if let Some(kernel_tag) = dense_kernel_tag(name) {
            assert_eq!(
                table.intrinsic(name),
                Some(kernel_tag),
                "prim `{name}`: the table's intrinsic must be carried verbatim from the \
                 mycelium-dense kernel's op_guarantee (VR-5)",
            );
        } else if let Some(op) = vsa_op(name) {
            // M-892: the model-dispatched VSA bind group — the table's single intrinsic is the
            // **meet over the dispatch set** (MAP-I/FHRR/BSC) of the kernel's per-model tags
            // (VR-5: a single Π slot must not over-claim for the weakest model; the runtime
            // value carries the dispatched model's own tag). Widening the dispatch set must
            // keep this meet honest — this guard recomputes it from the kernel itself.
            let models: [&dyn VsaModel; 3] = [&MapI::new(4), &Fhrr::new(4), &Bsc::new(4)];
            let meet =
                GuaranteeStrength::meet_all(models.iter().map(|m| m.intrinsic_guarantee(op)));
            assert_eq!(
                table.intrinsic(name),
                Some(meet),
                "prim `{name}`: the table's intrinsic must be the meet over the MAP-I/FHRR/BSC \
                 dispatch set of the mycelium-vsa kernel's per-model tags (VR-5)",
            );
        } else if name.starts_with("flt.") {
            // ADR-040 §2.6 (M-898): the scalar-float group routes through `flt_result`, whose
            // per-op tag is the ratified `Empirical` host-conformance posture (the value-side
            // twin — tag + zero-deviation bound — is guarded in `src/tests/prims.rs`).
            assert_eq!(
                table.intrinsic(name),
                Some(GuaranteeStrength::Empirical),
                "prim `{name}`: the table's intrinsic must match flt_result's ADR-040 §2.6 \
                 Empirical (VR-5)",
            );
        } else {
            assert_eq!(
                table.intrinsic(name),
                Some(GuaranteeStrength::Exact),
                "prim `{name}`: the table's intrinsic must match compose_result's hard-coded Exact",
            );
        }
    }
}
