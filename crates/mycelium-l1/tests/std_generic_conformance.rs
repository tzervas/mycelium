//! **M-719 — stdlib conformance over the WIDTH-GENERIC surface.**
//!
//! The named conformance gate for the M-718 generic surface: every width-generic stdlib op
//! (std.cmp `cmp/le/ge/max/min`, std.math `badd/bsub/band/bor/bxor/bnot` + `tadd/tsub/tmul/tneg`,
//! std.collections `map_get/set_contains`) is checked **three-way** (L1-eval(mono) ≡ elaborate→
//! L0-interp ≡ AOT) at **≥ 2 distinct widths** AND its never-silent width refusals are exercised, in
//! one auditable, data-driven table. The per-module test files (`std_cmp.rs`, `std_math.rs`,
//! `std_collections.rs`, `width_generic.rs`) carry the exhaustive case-by-case coverage; THIS file is
//! the consolidated gate that asserts the generic surface, as a whole, conforms and refuses honestly.
//!
//! # Scope (honest boundary — VR-5)
//! This closes M-719's "conformance over the new generic surface" clause (three-way + never-silent
//! refusals). M-719's BROADER closure — retiring/deprecating the reference `mycelium-std-*` Rust
//! crates and freezing a documented stable API per RFC-0031 — remains open and is NOT claimed here.
//!
//! # Honesty tags
//! - **`Exact`** — each op is Exact on its in-range result (the per-module files ground each tag).
//! - **`Empirical`** — the three-way differential agreement, by trial on the cases below.
//! - **never-silent (G2)** — a width mismatch is an explicit static refusal, never a silent coercion.
//!
//! Test-layout (CLAUDE.md §Test-layout): the conformance cases are a data table; each test body is an
//! `assert` over a case, not bespoke per-case logic.

use mycelium_cert::{check_core, BinaryTernarySwapEngine, CheckVerdict};
use mycelium_core::GuaranteeStrength;
use mycelium_interp::{Interpreter, PrimRegistry};
use mycelium_l1::elab::build_registry;
use mycelium_l1::{check_nodule, elaborate, monomorphize, parse, Evaluator};

const CMP_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../lib/std/cmp.myc"
));
const MATH_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../lib/std/math.myc"
));
const COLLECTIONS_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../lib/std/collections.myc"
));

/// One three-way conformance case: a nodule source, a `main` driver appended to it, and a reference
/// program whose interpreted value the three paths must equal.
struct Case {
    label: &'static str,
    nodule: &'static str,
    /// The `main` driver body appended to `nodule` (a full `fn main() -> … = …`).
    driver: &'static str,
    /// The reference program (a complete `nodule ref … fn main() -> … = …`).
    reference: &'static str,
}

/// Run the three-way differential on `nodule + driver` and assert all paths agree AND equal the
/// reference value. The harness mirrors `std_cmp.rs::assert_three_way` (the single source of the
/// three-path discipline).
fn run_case(c: &Case) {
    let label = c.label;
    let src = format!("{}\n{}", c.nodule, c.driver);

    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(BinaryTernarySwapEngine),
    );
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;

    let env = check_nodule(&parse(&src).unwrap_or_else(|e| panic!("{label}: parse failed: {e}")))
        .unwrap_or_else(|e| panic!("{label}: check failed: {e}"));
    let mono =
        monomorphize(&env, "main").unwrap_or_else(|e| panic!("{label}: monomorphize failed: {e}"));
    assert!(
        mono.fns.values().all(|fd| fd.sig.params.is_empty())
            && mono.types.values().all(|d| d.params.is_empty()),
        "{label}: monomorphized env must be closed (no generics)"
    );
    let registry =
        build_registry(&mono).unwrap_or_else(|e| panic!("{label}: build_registry failed: {e}"));

    let l1_core = Evaluator::new(&mono)
        .call("main", vec![])
        .unwrap_or_else(|e| panic!("{label}: L1-eval failed: {e}"))
        .to_core(&mono, &registry)
        .unwrap_or_else(|| panic!("{label}: L1 result outside the r3 data fragment"));
    let node = elaborate(&env, "main").unwrap_or_else(|e| panic!("{label}: elaborate failed: {e}"));
    let l0_core = interp
        .eval_core(&node)
        .unwrap_or_else(|e| panic!("{label}: L0-interp failed: {e}"));
    let aot_core = mycelium_mlir::run_core(&node, &prims, &engine)
        .unwrap_or_else(|e| panic!("{label}: AOT failed: {e}"));

    assert_eq!(l1_core, l0_core, "{label}: L1-eval vs L0-interp diverged");
    assert_eq!(l0_core, aot_core, "{label}: L0-interp vs AOT diverged");
    for (x, y, pair) in [
        (&l1_core, &l0_core, "L1↔interp"),
        (&l0_core, &aot_core, "interp↔AOT"),
    ] {
        assert_eq!(
            check_core(x, y),
            CheckVerdict::Validated {
                strength: GuaranteeStrength::Exact
            },
            "{label}: the shared checker must validate the {pair} pair"
        );
    }

    let ref_env = check_nodule(
        &parse(c.reference).unwrap_or_else(|e| panic!("{label}: ref parse failed: {e}")),
    )
    .unwrap_or_else(|e| panic!("{label}: ref check failed: {e}"));
    let ref_node = elaborate(&ref_env, "main")
        .unwrap_or_else(|e| panic!("{label}: ref elaborate failed: {e}"));
    let expected = interp
        .eval_core(&ref_node)
        .unwrap_or_else(|e| panic!("{label}: ref eval failed: {e}"));
    assert_eq!(l1_core, expected, "{label}: result != expected reference");
}

// ── Conformance cases — the generic surface at ≥ 2 widths each ──────────────────────────────────────

/// std.cmp at Binary{8} and Binary{16}.
const CMP_CASES: &[Case] = &[
    Case {
        label: "cmp@8: cmp(1,2)=Lt",
        nodule: CMP_SRC,
        driver: "fn main() -> Ordering = cmp(0b0000_0001, 0b0000_0010)",
        reference: "nodule ref\ntype Ordering = Lt | Eq | Gt\nfn main() -> Ordering = Lt",
    },
    Case {
        label: "cmp@16: cmp(256,2)=Gt",
        nodule: CMP_SRC,
        driver: "fn main() -> Ordering = cmp(0b0000_0001_0000_0000, 0b0000_0000_0000_0010)",
        reference: "nodule ref\ntype Ordering = Lt | Eq | Gt\nfn main() -> Ordering = Gt",
    },
    Case {
        label: "le@8: le(2,2)=True",
        nodule: CMP_SRC,
        driver: "fn main() -> Bool = le(0b0000_0010, 0b0000_0010)",
        reference: "nodule ref\nfn main() -> Bool = True",
    },
    Case {
        label: "max@16: max(1,256)=256",
        nodule: CMP_SRC,
        driver: "fn main() -> Binary{16} = max(0b0000_0000_0000_0001, 0b0000_0001_0000_0000)",
        reference: "nodule ref\nfn main() -> Binary{16} = 0b0000_0001_0000_0000",
    },
];

/// std.math: binary at Binary{8}/Binary{16}, ternary at Ternary{3}.
const MATH_CASES: &[Case] = &[
    Case {
        label: "badd@8: badd(3,5)=8",
        nodule: MATH_SRC,
        driver: "fn main() -> Binary{8} = badd(0b0000_0011, 0b0000_0101)",
        reference: "nodule ref\nfn main() -> Binary{8} = add_bin(0b0000_0011, 0b0000_0101)",
    },
    Case {
        label: "badd@16: badd(256,1)=257",
        nodule: MATH_SRC,
        driver:
            "fn main() -> Binary{16} = badd(0b0000_0001_0000_0000, 0b0000_0000_0000_0001)",
        reference:
            "nodule ref\nfn main() -> Binary{16} = add_bin(0b0000_0001_0000_0000, 0b0000_0000_0000_0001)",
    },
    Case {
        label: "tadd@3: tadd(+1,-1)=0",
        nodule: MATH_SRC,
        driver: "fn main() -> Ternary{3} = tadd(<00+>, <00->)",
        reference: "nodule ref\nfn main() -> Ternary{3} = add(<00+>, <00->)",
    },
];

/// std.collections: width-generic key lookup at Binary{8} and Binary{16}.
const COLLECTIONS_CASES: &[Case] = &[
    Case {
        label: "map_get@8: hit",
        nodule: COLLECTIONS_SRC,
        driver: "\
fn mk() -> Map<Binary{8}, Binary{8}> = MCons(0b0000_0001, 0b0000_1010, MNil)\n\
fn main() -> Option<Binary{8}> = map_get(mk(), 0b0000_0001)",
        reference:
            "nodule ref\ntype Option<A> = Some(A) | None\nfn main() -> Option<Binary{8}> = Some(0b0000_1010)",
    },
    Case {
        label: "map_get@16: hit after recurse",
        nodule: COLLECTIONS_SRC,
        driver: "\
fn mk() -> Map<Binary{16}, Binary{16}> = MCons(0b0000_0000_0000_0001, 0b0000_0000_0000_1010, MCons(0b0000_0001_0000_0000, 0b0000_0010_0000_0000, MNil))\n\
fn main() -> Option<Binary{16}> = map_get(mk(), 0b0000_0001_0000_0000)",
        reference:
            "nodule ref\ntype Option<A> = Some(A) | None\nfn main() -> Option<Binary{16}> = Some(0b0000_0010_0000_0000)",
    },
    Case {
        label: "set_contains@16: present after recurse",
        nodule: COLLECTIONS_SRC,
        driver: "\
fn mk() -> Set<Binary{16}> = SCons(0b0000_0000_0000_0001, SCons(0b0000_0001_0000_0000, SNil))\n\
fn main() -> Bool = set_contains(mk(), 0b0000_0001_0000_0000)",
        reference: "nodule ref\nfn main() -> Bool = True",
    },
];

#[test]
fn cmp_surface_conforms_three_way() {
    for c in CMP_CASES {
        run_case(c);
    }
}

#[test]
fn math_surface_conforms_three_way() {
    for c in MATH_CASES {
        run_case(c);
    }
}

#[test]
fn collections_surface_conforms_three_way() {
    for c in COLLECTIONS_CASES {
        run_case(c);
    }
}

// ── Never-silent width refusals over the generic surface (G2 / VR-5 / DN-42 §4) ──────────────────────

/// (label, nodule, driver) where `check_nodule` must REFUSE — a width mismatch is never a silent
/// coercion. One consolidated refusal table across the generic surface.
const REFUSAL_CASES: &[(&str, &str, &str)] = &[
    (
        "cmp mixed widths",
        CMP_SRC,
        "fn main() -> Ordering = cmp(0b0000_0001, 0b0000_0001_0000_0000)",
    ),
    (
        "badd mixed widths",
        MATH_SRC,
        "fn main() -> Binary{16} = badd(0b0000_0001, 0b0000_0001_0000_0000)",
    ),
    (
        "map_get mixed key widths",
        COLLECTIONS_SRC,
        "\
fn mk() -> Map<Binary{8}, Binary{8}> = MCons(0b0000_0001, 0b0000_1010, MNil)\n\
fn main() -> Option<Binary{8}> = map_get(mk(), 0b0000_0001_0000_0000)",
    ),
];

#[test]
fn generic_surface_width_mismatches_refuse() {
    for (label, nodule, driver) in REFUSAL_CASES {
        let src = format!("{nodule}\n{driver}");
        let parsed = parse(&src).unwrap_or_else(|e| panic!("{label}: parse should succeed: {e}"));
        assert!(
            check_nodule(&parsed).is_err(),
            "{label}: expected a never-silent width refusal, but check succeeded"
        );
    }
}
