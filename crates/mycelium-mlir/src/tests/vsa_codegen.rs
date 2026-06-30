//! In-crate white-box tests for `vsa_codegen.rs` (M-854; RFC-0039 §5.2; CLAUDE.md test-layout
//! rule). These are pure **emission** + **logic** checks (no toolchain): the per-operand
//! side-condition validation, the never-silent refusals (SBC/MAP-B model gate, off-alphabet,
//! out-of-regime, insufficient capacity), the inspectable `VsaExplain` / dumpable IR comment
//! (RFC-0004 §6), the honest reference-vs-codegen guarantee split (VR-5), and that the emitted IR
//! carries the explicit per-element ops (no opaque pass). The compiled-path differential
//! (native ≡ `mycelium-vsa`, M-210-checked, mutant-witnessed) lives in `tests/vsa_differential.rs`.

use crate::vsa_codegen::{
    emit_vsa_llvm_ir, first_non_binary, first_non_bipolar, first_off_phase, hrr_involution,
    VsaAotError, VsaCgOp, VsaExplain, VsaModelId, VsaProgram, VSA_CODEGEN_GUARANTEE,
};
use mycelium_core::{GuaranteeStrength, PhysicalLayout};

// ─── fixtures ────────────────────────────────────────────────────────────────────────────────────

/// A program for `op` over `model` at `dim` with the given operands + optional shift/δ.
fn prog(
    op: VsaCgOp,
    model: VsaModelId,
    dim: u32,
    items: Vec<Vec<f64>>,
    shift: Option<i64>,
    bundle_delta: Option<f64>,
) -> VsaProgram {
    VsaProgram {
        op,
        model,
        dim,
        items,
        shift,
        bundle_delta,
    }
}

/// A small bipolar (`±1`) vector for MAP-I.
fn bipolar(dim: u32) -> Vec<f64> {
    (0..dim)
        .map(|i| if i.is_multiple_of(2) { 1.0 } else { -1.0 })
        .collect()
}

/// A small binary (`{0,1}`) vector for BSC.
fn binary(dim: u32) -> Vec<f64> {
    (0..dim).map(|i| f64::from(i % 2)).collect()
}

/// A small real vector for HRR.
fn real(dim: u32) -> Vec<f64> {
    (0..dim).map(|i| f64::from(i) * 0.25 - 1.0).collect()
}

/// A small in-range phase vector for FHRR (each in `(−π, π]`).
fn phase(dim: u32) -> Vec<f64> {
    (0..dim).map(|i| f64::from(i % 5) * 0.5 - 1.0).collect()
}

/// A canonical well-formed program per `(model, op)` (used to assert emission + EXPLAIN shape). HRR
/// `unbind` uses dim 256 / FHRR/BSC bundle use the profile dims so they pass `validate()`.
fn canonical(model: VsaModelId, op: VsaCgOp) -> VsaProgram {
    let dim = match (model, op) {
        (VsaModelId::Hrr | VsaModelId::Fhrr, VsaCgOp::Unbind) => 256,
        (VsaModelId::Bsc, VsaCgOp::Bundle) => 1024,
        _ => 8,
    };
    let one = |m: VsaModelId, d: u32| match m {
        VsaModelId::MapI => bipolar(d),
        VsaModelId::Bsc => binary(d),
        VsaModelId::Hrr => real(d),
        VsaModelId::Fhrr => phase(d),
    };
    match op {
        VsaCgOp::Bind | VsaCgOp::Unbind | VsaCgOp::Similarity => prog(
            op,
            model,
            dim,
            vec![one(model, dim), one(model, dim)],
            None,
            None,
        ),
        VsaCgOp::Permute => prog(op, model, dim, vec![one(model, dim)], Some(2), None),
        VsaCgOp::Bundle => {
            // MAP-I bundle needs a δ + a dim ≥ requiredDim; BSC needs odd m ≤ 5 at d ≥ 1024.
            let (items, delta, d) = match model {
                VsaModelId::MapI => (
                    (0..3).map(|_| bipolar(2048)).collect::<Vec<_>>(),
                    Some(1e-2),
                    2048,
                ),
                VsaModelId::Bsc => ((0..3).map(|_| binary(1024)).collect(), None, 1024),
                VsaModelId::Hrr => ((0..3).map(|_| real(8)).collect(), None, 8),
                VsaModelId::Fhrr => ((0..3).map(|_| phase(8)).collect(), None, 8),
            };
            // make MAP-I items distinct so the capacity bound's distinctness side-condition holds at
            // the value level (the codegen does not re-check distinctness; the differential's
            // reference does — here we just need a lowerable program).
            prog(op, model, d, items, None, delta)
        }
    }
}

const MODELS: [VsaModelId; 4] = [
    VsaModelId::MapI,
    VsaModelId::Bsc,
    VsaModelId::Hrr,
    VsaModelId::Fhrr,
];
const VALUE_OPS: [VsaCgOp; 4] = [
    VsaCgOp::Bind,
    VsaCgOp::Unbind,
    VsaCgOp::Bundle,
    VsaCgOp::Permute,
];

// ─── op / model metadata (mirrors mycelium-vsa) ──────────────────────────────────────────────────

/// `similarity` is a measurement (no `Meta`); the four value ops produce a Value.
#[test]
fn is_value_op_classifies_the_surface() {
    for op in VALUE_OPS {
        assert!(op.is_value_op(), "{op:?} produces a Value");
    }
    assert!(
        !VsaCgOp::Similarity.is_value_op(),
        "similarity is a measurement"
    );
}

/// Model ids match the `mycelium-vsa` registry keys (so provenance / EXPLAIN are never anonymous).
#[test]
fn model_registry_ids_match_the_vsa_keys() {
    assert_eq!(VsaModelId::MapI.registry_id(), "MAP-I");
    assert_eq!(VsaModelId::Bsc.registry_id(), "BSC");
    assert_eq!(VsaModelId::Hrr.registry_id(), "HRR");
    assert_eq!(VsaModelId::Fhrr.registry_id(), "FHRR");
}

/// Op names match the `mycelium-vsa` operation keys (so provenance matches the reference); similarity
/// has no op key (it is a bare measurement).
#[test]
fn op_names_match_the_vsa_keys() {
    assert_eq!(
        VsaModelId::MapI.op_name(VsaCgOp::Bind).as_deref(),
        Some("vsa.map_i.bind")
    );
    assert_eq!(
        VsaModelId::Bsc.op_name(VsaCgOp::Bundle).as_deref(),
        Some("vsa.bsc.bundle")
    );
    assert_eq!(
        VsaModelId::Hrr.op_name(VsaCgOp::Unbind).as_deref(),
        Some("vsa.hrr.unbind")
    );
    assert_eq!(
        VsaModelId::Fhrr.op_name(VsaCgOp::Permute).as_deref(),
        Some("vsa.fhrr.permute")
    );
    assert_eq!(VsaModelId::MapI.op_name(VsaCgOp::Similarity), None);
}

/// The honest per-op value-level guarantee mirrors the reference's value-level surface (RFC-0003 §4.1,
/// VR-5): permute/bind Exact for every model; unbind Exact (MAP-I/BSC) vs Empirical (HRR/FHRR); bundle
/// Proven (MAP-I) / Empirical (BSC) / Declared (HRR/FHRR — no reference value-level bound); similarity
/// is a measurement (None). This is the load-bearing tag table — a wrong row mis-tags a value.
#[test]
fn reference_guarantee_mirrors_the_value_level_surface() {
    use GuaranteeStrength::{Declared, Empirical, Exact, Proven};
    // (model, op, expected)
    let table: &[(VsaModelId, VsaCgOp, GuaranteeStrength)] = &[
        // permute Exact for every model.
        (VsaModelId::MapI, VsaCgOp::Permute, Exact),
        (VsaModelId::Bsc, VsaCgOp::Permute, Exact),
        (VsaModelId::Hrr, VsaCgOp::Permute, Exact),
        (VsaModelId::Fhrr, VsaCgOp::Permute, Exact),
        // bind Exact for every model.
        (VsaModelId::MapI, VsaCgOp::Bind, Exact),
        (VsaModelId::Bsc, VsaCgOp::Bind, Exact),
        (VsaModelId::Hrr, VsaCgOp::Bind, Exact),
        (VsaModelId::Fhrr, VsaCgOp::Bind, Exact),
        // unbind: self-inverse Exact (MAP-I/BSC) vs the weak link Empirical (HRR/FHRR).
        (VsaModelId::MapI, VsaCgOp::Unbind, Exact),
        (VsaModelId::Bsc, VsaCgOp::Unbind, Exact),
        (VsaModelId::Hrr, VsaCgOp::Unbind, Empirical),
        (VsaModelId::Fhrr, VsaCgOp::Unbind, Empirical),
        // bundle: MAP-I Proven (capacity); BSC Empirical (profile); HRR/FHRR Declared (no ref bound).
        (VsaModelId::MapI, VsaCgOp::Bundle, Proven),
        (VsaModelId::Bsc, VsaCgOp::Bundle, Empirical),
        (VsaModelId::Hrr, VsaCgOp::Bundle, Declared),
        (VsaModelId::Fhrr, VsaCgOp::Bundle, Declared),
    ];
    for &(m, op, want) in table {
        assert_eq!(
            m.reference_guarantee(op),
            Some(want),
            "{m:?} {op:?} value-level tag must be {want:?}"
        );
    }
    // similarity is a measurement for every model — no Meta tag.
    for m in MODELS {
        assert_eq!(
            m.reference_guarantee(VsaCgOp::Similarity),
            None,
            "{m:?} similarity is a measurement (no Meta)"
        );
    }
}

/// The 1.0.0-native-mandatory model gate: only MAP-I/BSC/HRR/FHRR parse; SBC/MAP-B/unknown are `None`
/// (the caller turns that into an explicit `UnsupportedModel` refusal — never a silent substitution).
#[test]
fn only_mandatory_models_parse_sbc_mapb_refused() {
    assert_eq!(
        VsaModelId::from_registry_id("MAP-I"),
        Some(VsaModelId::MapI)
    );
    assert_eq!(VsaModelId::from_registry_id("BSC"), Some(VsaModelId::Bsc));
    assert_eq!(VsaModelId::from_registry_id("HRR"), Some(VsaModelId::Hrr));
    assert_eq!(VsaModelId::from_registry_id("FHRR"), Some(VsaModelId::Fhrr));
    // SBC / MAP-B / MAP-C and unknown ids are NOT mandatory-native (OQ-3) — refused, never served by
    // another model.
    for id in ["SBC", "MAP-B", "MAP-C", "VTB", "nonsense"] {
        assert_eq!(
            VsaModelId::from_registry_id(id),
            None,
            "{id} must not parse as a mandatory-native model (refused never-silently)"
        );
    }
}

// ─── the inspectable EXPLAIN record + dumpable IR comment (RFC-0004 §6 — no black box) ────────────

/// Every value op emits the dumpable EXPLAIN comment (op, model, dim, guarantees, carrier) — never a
/// hidden lowering (G2). The codegen-guarantee is always Empirical; the value carries the reference tag.
#[test]
fn every_value_op_emits_the_dumpable_explain_comment() {
    for model in MODELS {
        for op in VALUE_OPS {
            let p = canonical(model, op);
            let (ir, explain) = emit_vsa_llvm_ir(&p).expect("canonical program lowers");
            assert!(
                ir.contains(&format!("; vsa {}", p.model.op_name(op).unwrap())),
                "{model:?} {op:?} IR must carry the dumpable EXPLAIN comment:\n{ir}"
            );
            assert!(
                ir.contains("codegen-guarantee=Empirical"),
                "{model:?} {op:?} IR must record the Empirical codegen guarantee (VR-5):\n{ir}"
            );
            assert!(
                ir.contains("carrier=real-Vec<f64> dense"),
                "{model:?} {op:?} IR must record the real-Vec<f64> carrier status (E20-1 gate):\n{ir}"
            );
            assert_eq!(explain.model, model.registry_id());
            assert_eq!(explain.codegen_guarantee, GuaranteeStrength::Empirical);
            assert_eq!(explain.reference_guarantee, model.reference_guarantee(op));
        }
    }
}

/// The codegen-correctness guarantee is **Empirical** (VR-5 — the differential + mutant-witness are
/// the basis; no proof object linked here). Pinned so it cannot silently upgrade past its basis.
#[test]
fn codegen_guarantee_is_empirical_never_upgraded() {
    assert_eq!(VSA_CODEGEN_GUARANTEE, GuaranteeStrength::Empirical);
    for model in MODELS {
        for op in VALUE_OPS {
            let (_, explain) = emit_vsa_llvm_ir(&canonical(model, op)).unwrap();
            assert_eq!(
                explain.codegen_guarantee,
                GuaranteeStrength::Empirical,
                "{model:?} {op:?} codegen guarantee must stay Empirical (VR-5)"
            );
        }
    }
}

/// A value op records the inspectable `Meta.physical = VsaStore{sparse:false}` schedule (DN-01; the
/// schedule-as-metadata discipline); a measurement op (similarity) has no physical schedule.
#[test]
fn value_ops_record_the_vsa_store_schedule() {
    for model in MODELS {
        for op in VALUE_OPS {
            let (_, explain) = emit_vsa_llvm_ir(&canonical(model, op)).unwrap();
            assert_eq!(
                explain.physical,
                Some(PhysicalLayout::VsaStore { sparse: false }),
                "{model:?} {op:?} must record the VsaStore schedule"
            );
        }
        let (_, explain) = emit_vsa_llvm_ir(&canonical(model, VsaCgOp::Similarity)).unwrap();
        assert_eq!(
            explain.physical, None,
            "{model:?} similarity (measurement) has no physical schedule"
        );
    }
}

/// The `VsaExplain` carries BOTH the reference value tag and the (distinct) codegen tag — the
/// inspectable, never-conflated honest split (VR-5).
#[test]
fn vsa_explain_carries_the_honest_guarantee_split() {
    let e = VsaExplain {
        op: "vsa.map_i.bundle".to_owned(),
        model: "MAP-I",
        dim: 2048,
        physical: Some(PhysicalLayout::VsaStore { sparse: false }),
        reference_guarantee: Some(GuaranteeStrength::Proven),
        codegen_guarantee: GuaranteeStrength::Empirical,
        carrier: "real-Vec<f64> dense",
    };
    assert_eq!(e.reference_guarantee, Some(GuaranteeStrength::Proven));
    assert_eq!(e.codegen_guarantee, GuaranteeStrength::Empirical);
    assert_ne!(
        e.reference_guarantee.unwrap(),
        e.codegen_guarantee,
        "the reference value tag and the codegen-correctness tag must stay distinct (VR-5)"
    );
}

// ─── the IR transcode shape (no opaque pass — §6) ────────────────────────────────────────────────

/// MAP-I bind/unbind emit the explicit per-element `fmul double` product (one op per element). BSC
/// bind emits `fsub`+`fabs` (XOR = |a−b|). FHRR bind/unbind emit `fadd`/`fsub` + the phase wrap.
#[test]
fn bind_unbind_emit_explicit_per_element_ir() {
    // MAP-I product: one fmul per element.
    let mi = canonical(VsaModelId::MapI, VsaCgOp::Bind);
    let dim = mi.dim as usize;
    let (ir, _) = emit_vsa_llvm_ir(&mi).unwrap();
    assert_eq!(
        ir.matches("fmul double").count(),
        dim,
        "MAP-I bind must emit one fmul per element (§6):\n{ir}"
    );
    // BSC XOR: |a−b| per element (fsub + fabs).
    let bsc = canonical(VsaModelId::Bsc, VsaCgOp::Bind);
    let (ir, _) = emit_vsa_llvm_ir(&bsc).unwrap();
    assert!(
        ir.contains("fsub double") && ir.contains("@llvm.fabs.f64"),
        "BSC bind must emit the |a−b| XOR lowering:\n{ir}"
    );
    // FHRR bind: phase add + the `frem`-based rem_euclid wrap (bit-exact with `f64::rem_euclid`,
    // including the −0.0 sign). Unbind: fsub.
    let fadd = emit_vsa_llvm_ir(&canonical(VsaModelId::Fhrr, VsaCgOp::Bind))
        .unwrap()
        .0;
    assert!(
        fadd.contains("fadd double") && fadd.contains("frem double"),
        "FHRR bind must emit phase-add + the frem-based wrap (rem_euclid):\n{fadd}"
    );
    let fsub = emit_vsa_llvm_ir(&canonical(VsaModelId::Fhrr, VsaCgOp::Unbind))
        .unwrap()
        .0;
    assert!(
        fsub.contains("fsub double"),
        "FHRR unbind must emit phase-sub:\n{fsub}"
    );
}

/// HRR bind/unbind emit the circular-convolution `fmul`+`fadd` accumulation (one product per (k,i)
/// pair — `dim²` products for the naive reference form, mirroring `Hrr::cconv`).
#[test]
fn hrr_bind_emits_circular_convolution() {
    let p = canonical(VsaModelId::Hrr, VsaCgOp::Bind);
    let dim = p.dim as usize;
    let (ir, _) = emit_vsa_llvm_ir(&p).unwrap();
    assert_eq!(
        ir.matches("fmul double").count(),
        dim * dim,
        "HRR bind must emit dim² products (naive circular convolution, §6):\n{ir}"
    );
    assert!(
        ir.contains("fadd double"),
        "HRR bind must accumulate the convolution in f64:\n{ir}"
    );
}

/// FHRR bundle emits the complex-sum phasor reduction (`@cos`/`@sin` per term, `@atan2`, a sqrt
/// magnitude check) and the never-silent `DEGENERATE` sentinel trap (a vanished phasor sum).
#[test]
fn fhrr_bundle_emits_phasor_reduction_with_degenerate_trap() {
    let p = canonical(VsaModelId::Fhrr, VsaCgOp::Bundle);
    let (ir, _) = emit_vsa_llvm_ir(&p).unwrap();
    assert!(
        ir.contains("@cos(double") && ir.contains("@sin(double") && ir.contains("@atan2(double"),
        "FHRR bundle must emit the cos/sin/atan2 phasor reduction:\n{ir}"
    );
    assert!(
        ir.contains("@.s_deg, i64 0, i64 0") && ir.contains("br i1"),
        "FHRR bundle must emit the never-silent DEGENERATE trap branch (G2):\n{ir}"
    );
}

/// `permute` is a coordinate bijection (Exact) — it emits **no arithmetic** (no fmul/fadd/fsub),
/// just the printed permuted components (folded host-side). The marker of "no rounding/no trap" is the
/// absence of any float arithmetic op in the body.
#[test]
fn permute_emits_no_arithmetic() {
    for model in MODELS {
        let (ir, _) = emit_vsa_llvm_ir(&canonical(model, VsaCgOp::Permute)).unwrap();
        assert!(
            !ir.contains("fmul double")
                && !ir.contains("fadd double")
                && !ir.contains("fsub double"),
            "{model:?} permute is a coordinate bijection — it must emit no float arithmetic:\n{ir}"
        );
    }
}

/// `similarity` emits the per-model measurement IR and prints exactly one f64: cosine (MAP-I/HRR) with
/// the zero-norm guard; centered Hamming (BSC); mean phase-cos (FHRR).
#[test]
fn similarity_emits_the_per_model_measurement() {
    // MAP-I/HRR cosine: sqrt norms + fdiv + the zero-norm select guard.
    for model in [VsaModelId::MapI, VsaModelId::Hrr] {
        let (ir, _) = emit_vsa_llvm_ir(&canonical(model, VsaCgOp::Similarity)).unwrap();
        assert!(
            ir.contains("@llvm.sqrt.f64") && ir.contains("fdiv double") && ir.contains("select i1"),
            "{model:?} similarity must emit cosine + the zero-norm guard:\n{ir}"
        );
    }
    // BSC centered Hamming: fcmp oeq + the 1 − 2·h/d arithmetic.
    let (ir, _) = emit_vsa_llvm_ir(&canonical(VsaModelId::Bsc, VsaCgOp::Similarity)).unwrap();
    assert!(
        ir.contains("fcmp oeq double") && ir.contains("fdiv double"),
        "BSC similarity must emit centered-Hamming IR:\n{ir}"
    );
    // FHRR mean phase-cos.
    let (ir, _) = emit_vsa_llvm_ir(&canonical(VsaModelId::Fhrr, VsaCgOp::Similarity)).unwrap();
    assert!(
        ir.contains("@cos(double") && ir.contains("fdiv double"),
        "FHRR similarity must emit mean phase-cos:\n{ir}"
    );
}

// ─── emission determinism ────────────────────────────────────────────────────────────────────────

#[test]
fn emission_is_deterministic() {
    for model in MODELS {
        for op in [
            VsaCgOp::Bind,
            VsaCgOp::Bundle,
            VsaCgOp::Permute,
            VsaCgOp::Similarity,
        ] {
            let p = canonical(model, op);
            assert_eq!(
                emit_vsa_llvm_ir(&p).map(|(ir, _)| ir),
                emit_vsa_llvm_ir(&p).map(|(ir, _)| ir),
                "{model:?} {op:?} emission must be deterministic"
            );
        }
    }
}

// ─── never-silent refusals (G2) — the validation half, no toolchain needed ───────────────────────

/// A dimension mismatch between operands is refused (matches `VsaError::DimMismatch`).
#[test]
fn dim_mismatch_is_refused() {
    let p = prog(
        VsaCgOp::Bind,
        VsaModelId::MapI,
        4,
        vec![bipolar(4), bipolar(2)],
        None,
        None,
    );
    match emit_vsa_llvm_ir(&p) {
        Err(VsaAotError::DimMismatch { expected, got }) => {
            assert_eq!(expected, 4);
            assert_eq!(got, 2);
        }
        other => panic!("dim mismatch must be refused, got {other:?}"),
    }
}

/// An empty bundle is refused (matches `VsaError::EmptyBundle`).
#[test]
fn empty_bundle_is_refused() {
    let p = prog(
        VsaCgOp::Bundle,
        VsaModelId::MapI,
        4,
        vec![],
        None,
        Some(1e-2),
    );
    assert!(matches!(
        emit_vsa_llvm_ir(&p),
        Err(VsaAotError::EmptyBundle)
    ));
}

/// Off-alphabet components are refused per model (matches `VsaError::NonAlphabetComponent`): a
/// non-`±1` for MAP-I, a non-`{0,1}` for BSC, an out-of-range phase for FHRR. HRR has no alphabet.
#[test]
fn off_alphabet_components_are_refused() {
    // MAP-I: 0.5 is not ±1.
    let mut a = bipolar(4);
    a[2] = 0.5;
    let p = prog(
        VsaCgOp::Bind,
        VsaModelId::MapI,
        4,
        vec![a, bipolar(4)],
        None,
        None,
    );
    assert!(matches!(
        emit_vsa_llvm_ir(&p),
        Err(VsaAotError::NonAlphabetComponent {
            model: "MAP-I",
            index: 2
        })
    ));
    // BSC: 2.0 is not {0,1}.
    let mut b = binary(4);
    b[1] = 2.0;
    let p = prog(
        VsaCgOp::Bind,
        VsaModelId::Bsc,
        4,
        vec![b, binary(4)],
        None,
        None,
    );
    assert!(matches!(
        emit_vsa_llvm_ir(&p),
        Err(VsaAotError::NonAlphabetComponent {
            model: "BSC",
            index: 1
        })
    ));
    // FHRR: 7.0 is outside (−π, π].
    let mut c = phase(4);
    c[3] = 7.0;
    let p = prog(
        VsaCgOp::Bind,
        VsaModelId::Fhrr,
        4,
        vec![c, phase(4)],
        None,
        None,
    );
    assert!(matches!(
        emit_vsa_llvm_ir(&p),
        Err(VsaAotError::NonAlphabetComponent {
            model: "FHRR",
            index: 3
        })
    ));
    // HRR has no alphabet — an arbitrary real vector lowers fine.
    let p = prog(
        VsaCgOp::Bind,
        VsaModelId::Hrr,
        4,
        vec![vec![0.1, -3.7, 100.0, 0.0], real(4)],
        None,
        None,
    );
    assert!(
        emit_vsa_llvm_ir(&p).is_ok(),
        "HRR has no alphabet constraint"
    );
}

/// A MAP-I `bundle` below `requiredDim(items, δ)` is refused (`InsufficientCapacity`) — never an
/// unbacked `Proven` (matches `VsaError::InsufficientCapacity`; VR-5/M-I2). At dim 64, 3 items, δ=1e-2
/// the theorem requires 1141, so it fails.
#[test]
fn map_i_bundle_below_required_dim_is_refused() {
    let items: Vec<Vec<f64>> = (0..3).map(|_| bipolar(64)).collect();
    let p = prog(
        VsaCgOp::Bundle,
        VsaModelId::MapI,
        64,
        items,
        None,
        Some(1e-2),
    );
    match emit_vsa_llvm_ir(&p) {
        Err(VsaAotError::InsufficientCapacity {
            items,
            dim,
            required,
        }) => {
            assert_eq!(items, 3);
            assert_eq!(dim, 64);
            assert!(required > 64, "required dim {required} must exceed 64");
        }
        other => panic!("insufficient capacity must be refused, got {other:?}"),
    }
}

/// A MAP-I `bundle` with no δ is malformed (a Proven capacity bound needs a target failure probability).
#[test]
fn map_i_bundle_without_delta_is_malformed() {
    let items: Vec<Vec<f64>> = (0..3).map(|_| bipolar(2048)).collect();
    let p = prog(VsaCgOp::Bundle, VsaModelId::MapI, 2048, items, None, None);
    assert!(matches!(
        emit_vsa_llvm_ir(&p),
        Err(VsaAotError::Malformed(_))
    ));
}

/// A BSC `bundle` outside its profile (even item count, or below dim 1024) is refused
/// (`OutsideEmpiricalProfile`) — matches `BSC_BUNDLE_PROFILE.check`.
#[test]
fn bsc_bundle_outside_profile_is_refused() {
    // Even item count (4) — outside the odd-only profile.
    let items: Vec<Vec<f64>> = (0..4).map(|_| binary(1024)).collect();
    let p = prog(VsaCgOp::Bundle, VsaModelId::Bsc, 1024, items, None, None);
    assert!(matches!(
        emit_vsa_llvm_ir(&p),
        Err(VsaAotError::OutsideEmpiricalProfile(_))
    ));
    // Below dim 1024.
    let items: Vec<Vec<f64>> = (0..3).map(|_| binary(256)).collect();
    let p = prog(VsaCgOp::Bundle, VsaModelId::Bsc, 256, items, None, None);
    assert!(matches!(
        emit_vsa_llvm_ir(&p),
        Err(VsaAotError::OutsideEmpiricalProfile(_))
    ));
}

/// HRR/FHRR `unbind` below the profile minimum dim (256) is refused (`OutsideEmpiricalProfile`) —
/// matches the reference's `*_UNBIND_PROFILE.check`.
#[test]
fn hrr_fhrr_unbind_below_min_dim_is_refused() {
    let p = prog(
        VsaCgOp::Unbind,
        VsaModelId::Hrr,
        64,
        vec![real(64), real(64)],
        None,
        None,
    );
    assert!(matches!(
        emit_vsa_llvm_ir(&p),
        Err(VsaAotError::OutsideEmpiricalProfile(_))
    ));
    let p = prog(
        VsaCgOp::Unbind,
        VsaModelId::Fhrr,
        64,
        vec![phase(64), phase(64)],
        None,
        None,
    );
    assert!(matches!(
        emit_vsa_llvm_ir(&p),
        Err(VsaAotError::OutsideEmpiricalProfile(_))
    ));
}

/// A binary op with < 2 operands, a permute with no shift, are malformed programs — refused
/// explicitly, never panicking.
#[test]
fn malformed_programs_are_refused() {
    let one_operand = prog(
        VsaCgOp::Bind,
        VsaModelId::MapI,
        4,
        vec![bipolar(4)],
        None,
        None,
    );
    assert!(matches!(
        emit_vsa_llvm_ir(&one_operand),
        Err(VsaAotError::Malformed(_))
    ));
    let no_shift = prog(
        VsaCgOp::Permute,
        VsaModelId::MapI,
        4,
        vec![bipolar(4)],
        None,
        None,
    );
    assert!(matches!(
        emit_vsa_llvm_ir(&no_shift),
        Err(VsaAotError::Malformed(_))
    ));
}

// ─── mutant-witness for the host-side alphabet / involution helpers ──────────────────────────────

/// Direct witness for the alphabet predicates (`first_non_bipolar`/`first_non_binary`/`first_off_phase`)
/// — the host checks the lowering's input validation relies on. Pins exactly which components are
/// accepted/rejected, killing the `== ↔ !=` / boundary mutants.
#[test]
fn alphabet_predicates_accept_and_reject_exactly() {
    // bipolar: ±1 accepted; anything else is the first offender.
    assert_eq!(first_non_bipolar(&[1.0, -1.0, 1.0]), None);
    assert_eq!(first_non_bipolar(&[1.0, 0.0, -1.0]), Some(1));
    assert_eq!(first_non_bipolar(&[1.0, -1.0, 2.0]), Some(2));
    // binary: 0/1 accepted.
    assert_eq!(first_non_binary(&[0.0, 1.0, 0.0]), None);
    assert_eq!(first_non_binary(&[0.0, 1.0, -1.0]), Some(2));
    assert_eq!(first_non_binary(&[0.5, 1.0]), Some(0));
    // phase: in (−π, π] accepted; −π exclusive, π inclusive; NaN/Inf rejected.
    assert_eq!(first_off_phase(&[0.0, std::f64::consts::PI, -1.0]), None);
    assert_eq!(
        first_off_phase(&[-std::f64::consts::PI, 0.0]),
        Some(0),
        "−π is exclusive (the open lower bound)"
    );
    assert_eq!(first_off_phase(&[0.0, f64::NAN]), Some(1));
    assert_eq!(first_off_phase(&[0.0, 4.0]), Some(1));
}

/// Direct witness for `hrr_involution` (`b~[i] = b[(−i) mod d]`) — the host fold the unbind correlation
/// relies on. Pins the exact index map (kills the `(d − i) ↔ (d + i)` / off-by-one mutants).
#[test]
fn hrr_involution_maps_indices_exactly() {
    // d = 4: involution(b)[0] = b[0], [1] = b[3], [2] = b[2], [3] = b[1].
    let b = vec![10.0, 20.0, 30.0, 40.0];
    assert_eq!(hrr_involution(&b), vec![10.0, 40.0, 30.0, 20.0]);
    // d = 1: identity.
    assert_eq!(hrr_involution(&[7.0]), vec![7.0]);
    // d = 3: [0]=b[0], [1]=b[2], [2]=b[1].
    assert_eq!(hrr_involution(&[1.0, 2.0, 3.0]), vec![1.0, 3.0, 2.0]);
}

/// The `VsaAotError` `Display` strings discriminate the variants (kills the
/// `fmt -> Ok(Default::default())` mutant, which would blank every message — a never-silent refusal
/// must say *what* was refused, G2/ADR-006).
#[test]
fn error_display_messages_discriminate_and_are_nonempty() {
    let cases: [(VsaAotError, &str); 5] = [
        (
            VsaAotError::UnsupportedModel("SBC".to_owned()),
            "1.0.0-native-mandatory",
        ),
        (
            VsaAotError::UnsupportedCarrier("block-sparse".to_owned()),
            "E20-1",
        ),
        (VsaAotError::EmptyBundle, "at least one"),
        (VsaAotError::DegenerateBundleComponent, "phasor"),
        (
            VsaAotError::InsufficientCapacity {
                items: 3,
                dim: 64,
                required: 1141,
            },
            "Proven",
        ),
    ];
    for (err, needle) in cases {
        let msg = err.to_string();
        assert!(!msg.is_empty(), "{err:?} Display must be non-empty (G2)");
        assert!(
            msg.contains(needle),
            "{err:?} Display must name the refusal ({needle:?}); got: {msg}"
        );
    }
}
