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
use mycelium_l1::Ty;

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
        ("not", vec![Ty::Binary(8)], Ty::Binary(8)),
        ("xor", vec![Ty::Binary(8), Ty::Binary(8)], Ty::Binary(8)),
        ("add", vec![Ty::Ternary(4), Ty::Ternary(4)], Ty::Ternary(4)),
        ("sub", vec![Ty::Ternary(4), Ty::Ternary(4)], Ty::Ternary(4)),
        ("mul", vec![Ty::Ternary(4), Ty::Ternary(4)], Ty::Ternary(4)),
        ("neg", vec![Ty::Ternary(4)], Ty::Ternary(4)),
    ]
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
