//! R7-Q4 (M-390) equivalence guard: the L1 surface prim table (`checkty::prim_sig` +
//! `prim_kernel_name`) must stay consistent with the content-addressed prim table `Π`
//! (`mycelium_core::PrimTable`). DN-10 §3.4: `Π_new(hash(p)) = Π_old(name(p))`.
//!
//! The surface table is L1 sugar: width-polymorphic surface names (`add`, `xor`, …) that elaborate
//! onto the trusted kernel prims (`trit.add`, `bit.xor`, …). This guard pins three properties so the
//! two hard-coded lists cannot drift: (1) every surface prim's kernel target is a *declared* kernel
//! prim in the table; (2) the table entry's arity matches what `prim_sig` accepts; (3) the operand
//! and result *paradigms* agree.

use mycelium_core::{PrimParadigm, PrimTable};
use mycelium_l1::checkty::{prim_kernel_name, prim_sig};
use mycelium_l1::{checkty::Width, Ty};

/// The paradigm a surface `Ty` operand presents to the kernel.
fn paradigm_of(t: &Ty) -> PrimParadigm {
    match t {
        Ty::Binary(_) => PrimParadigm::Binary,
        Ty::Ternary(_) => PrimParadigm::Ternary,
        other => panic!("prim operands are Binary/Ternary in v0, got {other:?}"),
    }
}

/// Each surface prim, with representative well-typed operands and the result `prim_sig` must yield.
fn surface_cases() -> Vec<(&'static str, Vec<Ty>, Ty)> {
    vec![
        (
            "not",
            vec![Ty::Binary(Width::Lit(8))],
            Ty::Binary(Width::Lit(8)),
        ),
        (
            "xor",
            vec![Ty::Binary(Width::Lit(8)), Ty::Binary(Width::Lit(8))],
            Ty::Binary(Width::Lit(8)),
        ),
        (
            "add",
            vec![Ty::Ternary(Width::Lit(4)), Ty::Ternary(Width::Lit(4))],
            Ty::Ternary(Width::Lit(4)),
        ),
        (
            "sub",
            vec![Ty::Ternary(Width::Lit(4)), Ty::Ternary(Width::Lit(4))],
            Ty::Ternary(Width::Lit(4)),
        ),
        (
            "mul",
            vec![Ty::Ternary(Width::Lit(4)), Ty::Ternary(Width::Lit(4))],
            Ty::Ternary(Width::Lit(4)),
        ),
        (
            "neg",
            vec![Ty::Ternary(Width::Lit(4))],
            Ty::Ternary(Width::Lit(4)),
        ),
        // RFC-0032 D2 (M-748): width-uniform binary logical + never-silent arithmetic.
        (
            "and",
            vec![Ty::Binary(Width::Lit(8)), Ty::Binary(Width::Lit(8))],
            Ty::Binary(Width::Lit(8)),
        ),
        (
            "or",
            vec![Ty::Binary(Width::Lit(8)), Ty::Binary(Width::Lit(8))],
            Ty::Binary(Width::Lit(8)),
        ),
        (
            "add_bin",
            vec![Ty::Binary(Width::Lit(8)), Ty::Binary(Width::Lit(8))],
            Ty::Binary(Width::Lit(8)),
        ),
        (
            "sub_bin",
            vec![Ty::Binary(Width::Lit(8)), Ty::Binary(Width::Lit(8))],
            Ty::Binary(Width::Lit(8)),
        ),
        // RFC-0033 §4.1.2/§4.1.3 (M-887, `enb` Gap B): never-silent two's-complement multiply.
        (
            "mul_bin",
            vec![Ty::Binary(Width::Lit(8)), Ty::Binary(Width::Lit(8))],
            Ty::Binary(Width::Lit(8)),
        ),
        // RFC-0033 §4.1.2/§4.1.3 (M-888, `enb` Gap B): never-silent unsigned division/remainder.
        (
            "div_bin",
            vec![Ty::Binary(Width::Lit(8)), Ty::Binary(Width::Lit(8))],
            Ty::Binary(Width::Lit(8)),
        ),
        (
            "rem_bin",
            vec![Ty::Binary(Width::Lit(8)), Ty::Binary(Width::Lit(8))],
            Ty::Binary(Width::Lit(8)),
        ),
        // RFC-0033 §4.1.2/§4.1.3 (M-889, `enb` Gap B): never-silent logical left/right shift.
        (
            "shl_bin",
            vec![Ty::Binary(Width::Lit(8)), Ty::Binary(Width::Lit(8))],
            Ty::Binary(Width::Lit(8)),
        ),
        (
            "shr_bin",
            vec![Ty::Binary(Width::Lit(8)), Ty::Binary(Width::Lit(8))],
            Ty::Binary(Width::Lit(8)),
        ),
        // RFC-0033 §4.1.2/§4.1.3 (M-766, `enb` Gap B): never-silent two's-complement add/sub/neg —
        // completes the shared set `mul_bin` started. `add_tc`/`sub_tc` (not `add_bin`/`sub_bin`,
        // already claimed by the unsigned `bit.add`/`bit.sub`); `neg_bin` has no such conflict.
        (
            "add_tc",
            vec![Ty::Binary(Width::Lit(8)), Ty::Binary(Width::Lit(8))],
            Ty::Binary(Width::Lit(8)),
        ),
        (
            "sub_tc",
            vec![Ty::Binary(Width::Lit(8)), Ty::Binary(Width::Lit(8))],
            Ty::Binary(Width::Lit(8)),
        ),
        (
            "neg_bin",
            vec![Ty::Binary(Width::Lit(8))],
            Ty::Binary(Width::Lit(8)),
        ),
    ]
}

/// The M-890 (`enb` Gap C) dense elementwise prims are **tensor-valued** — their operands/results
/// are `Ty::Dense(dim, scalar)`, typed by a dedicated checker branch (`try_check_dense_prim`), and
/// their Π entries use the documented `Any`/`Uniform` paradigm-model escape hatch (no first-class
/// `Dense` paradigm yet — the same FLAG as the seq/bytes prims). So they fit neither
/// `surface_cases` (which asserts Binary/Ternary operand paradigms) nor the comparison guard. This
/// guard pins their surface→Π consistency directly: each surface name maps to a declared kernel
/// prim with the right arity, `Any` operands/result, and — the M-890 core contract — the intrinsic
/// tag **carried from the kernel** (`dense.neg` `Exact`; `dense.add`/`dense.sub`/`dense.scale`
/// `Proven`; VR-5 — the kernel-side twin lives in `mycelium-interp/tests/prim_table.rs`, which can
/// see `DenseSpace::op_guarantee` itself).
#[test]
fn dense_prims_resolve_to_declared_tensor_valued_kernel_prims() {
    use mycelium_core::GuaranteeStrength;
    let table = PrimTable::builtins();
    for (surface, kernel_expected, arity, intrinsic) in [
        ("dense_add", "dense.add", 2, GuaranteeStrength::Proven),
        ("dense_sub", "dense.sub", 2, GuaranteeStrength::Proven),
        ("dense_neg", "dense.neg", 1, GuaranteeStrength::Exact),
        ("dense_scale", "dense.scale", 2, GuaranteeStrength::Proven),
    ] {
        let kernel = prim_kernel_name(surface)
            .unwrap_or_else(|| panic!("dense prim `{surface}` must map to a kernel name"));
        assert_eq!(kernel, kernel_expected, "surface→kernel mapping drifted");
        assert!(
            table.contains(kernel),
            "surface `{surface}` → kernel `{kernel}`, but `{kernel}` is not declared in Π",
        );
        let decl = table.get(kernel).expect("declared prim");
        assert_eq!(decl.sig.arity(), arity, "`{kernel}` arity drifted");
        assert!(
            decl.sig.operands.iter().all(|p| *p == PrimParadigm::Any),
            "`{kernel}` operands use the documented `Any` escape hatch (no Dense paradigm yet)",
        );
        assert_eq!(decl.sig.result, PrimParadigm::Any);
        assert_eq!(
            decl.intrinsic, intrinsic,
            "`{kernel}` intrinsic must be carried from the kernel's op_guarantee (VR-5)",
        );
    }
}

/// The RFC-0032 D1 (M-747) comparison prims are **width-collapsing** and paradigm-flexible
/// (`Any, Any → Binary`, `WidthRel::Collapse`), so they do not fit the width-uniform `surface_cases`
/// shape (they bypass `prim_sig` via a dedicated checker branch). This guard pins their surface→Π
/// consistency directly: each maps to a declared collapsing kernel prim with arity 2.
#[test]
fn comparison_prims_resolve_to_declared_collapsing_kernel_prims() {
    use mycelium_core::WidthRel;
    let table = PrimTable::builtins();
    for surface in ["eq", "lt"] {
        let kernel = prim_kernel_name(surface)
            .unwrap_or_else(|| panic!("comparison prim `{surface}` must map to a kernel name"));
        assert!(
            table.contains(kernel),
            "surface `{surface}` → kernel `{kernel}`, but `{kernel}` is not declared in Π",
        );
        let decl = table.get(kernel).expect("declared prim");
        assert_eq!(decl.sig.arity(), 2, "`{kernel}` is binary (two operands)");
        assert_eq!(
            decl.sig.result,
            PrimParadigm::Binary,
            "`{kernel}` reduces to a Binary truth value",
        );
        assert_eq!(
            decl.sig.width,
            WidthRel::Collapse,
            "`{kernel}` is width-collapsing (operands' width → Binary{{1}})",
        );
    }
}

#[test]
fn surface_prims_resolve_to_declared_kernel_prims() {
    let table = PrimTable::builtins();
    for (surface, args, _ret) in surface_cases() {
        let kernel = prim_kernel_name(surface)
            .unwrap_or_else(|| panic!("surface prim `{surface}` must map to a kernel name"));
        assert!(
            table.contains(kernel),
            "surface `{surface}` → kernel `{kernel}`, but `{kernel}` is not a declared prim in Π",
        );
        let _ = args;
    }
}

#[test]
fn surface_signature_matches_the_kernel_declaration() {
    let table = PrimTable::builtins();
    for (surface, args, ret) in surface_cases() {
        // `prim_sig` accepts the representative operands and yields the expected result type.
        assert_eq!(
            prim_sig(surface, &args),
            Some(ret.clone()),
            "surface `{surface}` signature changed unexpectedly",
        );

        let kernel = prim_kernel_name(surface).expect("kernel name");
        let decl = table.get(kernel).expect("declared prim");

        // Arity agrees.
        assert_eq!(
            decl.sig.arity(),
            args.len(),
            "surface `{surface}` arity disagrees with kernel `{kernel}` declaration",
        );
        // Per-operand paradigm agrees.
        for (i, arg) in args.iter().enumerate() {
            assert_eq!(
                decl.sig.operands[i],
                paradigm_of(arg),
                "surface `{surface}` operand {i} paradigm disagrees with kernel `{kernel}`",
            );
        }
        // Result paradigm agrees.
        assert_eq!(
            decl.sig.result,
            paradigm_of(&ret),
            "surface `{surface}` result paradigm disagrees with kernel `{kernel}`",
        );
    }
}
