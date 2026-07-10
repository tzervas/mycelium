//! Differential tests for `std.numerics` (M-1020, kickoff `spw`, RFC-0031 D5) — the `.myc` port of
//! the honesty-crux surface of `crates/mycelium-std-numerics/src/lib.rs` (the guarantee lattice,
//! its `meet`, the `BoundBasis`→strength derivation, the `Approx` strength carrier, and the
//! `NumErr`/`CheckErr` refusal/verdict variant sets).
//!
//! # Scope (surface-check, D5 row 1 — see `lib/std/numerics.myc`'s module doc for the full writeup)
//! `mycelium-std-numerics` is dominated by the float-valued ε/δ *magnitude* surface (`error_bound`,
//! `prob_bound`, `union_delta`, `accuracy_to_probability`, `check_error`/`check_union`, and
//! `combine`'s ε-composition). The `.myc` runtime has NO scalar-float VALUE form yet (`enb` Gap A,
//! M-895/M-896), and that algebra is the `mycelium-numerics` kernel (RFC-0031 D1 boundary, stays
//! Rust). The FR-N3 sealed `ProvenThm` witness and the `&mut Formatter` `Display` impls have no
//! value-semantic `.myc` form either. All of that is FLAGged in the nodule, NOT ported (VR-5/G2 — a
//! mostly-FLAGged honest result is the success case). This file tests ONLY the ported strength
//! surface — the "never-upgrading" core the crate exists to be.
//!
//! # Harness design + row-4 Rust oracle
//! Execution/comparison machinery lives in the shared [`harness`] fixture (M-925); this file supplies
//! the nodule's `include_str!`, the per-case drivers, and the row-4 Rust-oracle wiring. Every
//! `expected` value is computed LIVE from the retained Rust oracles — `mycelium_core::{GuaranteeStrength,
//! BoundBasis}` (the lattice `meet`/`rank`/`basis`→strength ground truth) and
//! `mycelium_std_numerics::{NumErr, CheckErr}` (the refusal/verdict VARIANT set) — never a hardcoded
//! literal. A real divergence between the Rust source and the `.myc` transcription flips the computed
//! oracle value and fails the case; adding/removing a `NumErr`/`CheckErr` variant breaks the
//! exhaustive Rust match at COMPILE time (never a silent drift).
//!
//! # Honesty tags (VR-5 — carried at the same strength as `mycelium-std-numerics`, never upgraded)
//! - **`Exact`** — `rank`/`meet`/`meet_all`/`basis_strength`/`is_stronger_eq`, the `Approx`
//!   projections, and the `NumErr`/`CheckErr` discriminators: total, match-defined, finite-domain ops
//!   with no accuracy semantics of their own (RFC-0016 §4.1 C2).
//! - **`Declared`** — the lattice DATA (rank values, meet table, discriminant codes) as transcribed,
//!   mirroring the Rust source's own structural `#[cfg(test)]` assertions.
//! - **`Empirical`** — the three-way differential agreement (L1-eval ≡ L0-interp ≡ AOT) below,
//!   validated by trial on the cases exercised, against the live oracles.

mod harness;

use mycelium_core::{BoundBasis, GuaranteeStrength};
use mycelium_std_numerics::{CheckErr, NumErr};

/// The std.numerics nodule source, loaded at compile time — the single source of truth.
const NUMERICS_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../lib/std/numerics.myc"
));

/// Build a full test program by appending a driver to the nodule source.
fn program(driver: &str) -> String {
    harness::program(NUMERICS_SRC, driver)
}

/// Thin re-export of the shared [`harness::assert_three_way`].
fn assert_three_way(label: &str, src: &str, expected_src: &str) {
    harness::assert_three_way(label, src, expected_src);
}

/// Render a `u8` as the grouped 8-bit `.myc` binary literal that denotes the same `Binary{8}`
/// (matches the `0b0000_0011` grouping the landed `.myc` files use).
fn myc_u8_bits(n: u8) -> String {
    format!("0b{:04b}_{:04b}", n >> 4, n & 0x0F)
}

/// Render a Rust `bool` as the `.myc` literal that denotes the same `Bool` value.
fn myc_bool(b: bool) -> &'static str {
    if b {
        "True"
    } else {
        "False"
    }
}

/// Map a live `mycelium_core::GuaranteeStrength` to the `.myc` mirror constructor
/// (`lib/std/numerics.myc::Guarantee`). Exhaustive — a new strength variant breaks this at compile
/// time (never a silent gap).
fn myc_guarantee(g: GuaranteeStrength) -> &'static str {
    match g {
        GuaranteeStrength::Exact => "GExact",
        GuaranteeStrength::Proven => "GProven",
        GuaranteeStrength::Empirical => "GEmpirical",
        GuaranteeStrength::Declared => "GDeclared",
    }
}

/// A reference `.myc` program whose `main` is a single `Binary{8}` literal `n`.
fn ref_u8(n: u8) -> String {
    format!(
        "nodule ref;\nfn main() => Binary{{8}} = {};",
        myc_u8_bits(n)
    )
}

/// A reference `.myc` program whose `main` is a single `Bool` literal.
fn ref_bool(b: bool) -> String {
    format!("nodule ref;\nfn main() => Bool = {};", myc_bool(b))
}

/// The four lattice strengths, in declaration (strongest→weakest) order — the exhaustive domain the
/// `.myc` `Guarantee` mirror must reproduce (`GuaranteeStrength::ALL` is the same list, verified by
/// the loop bodies below).
const ALL_STRENGTHS: [GuaranteeStrength; 4] = [
    GuaranteeStrength::Exact,
    GuaranteeStrength::Proven,
    GuaranteeStrength::Empirical,
    GuaranteeStrength::Declared,
];

// ── rank — mirror of GuaranteeStrength::rank (driven against the live oracle, not hardcoded) ──────

/// `rank(Gx)` in `.myc` equals `GuaranteeStrength::rank` for every strength.
/// Guarantee: Exact (total, match-defined); Empirical (differential).
#[test]
fn rank_matches_core_oracle() {
    for &g in &ALL_STRENGTHS {
        let driver = format!("fn main() => Binary{{8}} = rank({});", myc_guarantee(g));
        let src = program(&driver);
        assert_three_way(&format!("rank({g:?})"), &src, &ref_u8(g.rank()));
    }
}

// ── meet — mirror of GuaranteeStrength::meet, EXHAUSTIVE over all 16 pairs (the FR-N2/VR-5 crux) ──

/// `rank(meet(Ga, Gb))` in `.myc` equals `a.meet(b).rank()` for ALL 16 strength pairs — the
/// weakest-wins composition rule that keeps a composed strength from ever exceeding its weakest
/// input. Driven against the live `GuaranteeStrength::meet` oracle.
///
/// Mutation witness: any `.myc` meet-table arm that returned a STRONGER result than the oracle
/// (e.g. `meet(GProven, GDeclared) => GProven`) would flip the computed rank and fail here — the
/// exact VR-5 violation the port guards.
#[test]
fn meet_is_weakest_wins_matches_core_oracle_all_16_pairs() {
    for &a in &ALL_STRENGTHS {
        for &b in &ALL_STRENGTHS {
            let driver = format!(
                "fn main() => Binary{{8}} = rank(meet({}, {}));",
                myc_guarantee(a),
                myc_guarantee(b)
            );
            let src = program(&driver);
            assert_three_way(
                &format!("meet({a:?},{b:?})"),
                &src,
                &ref_u8(a.meet(b).rank()),
            );
        }
    }
}

// ── is_stronger_eq — the lattice order predicate, EXHAUSTIVE over all 16 pairs ─────────────────────

/// `is_stronger_eq(Ga, Gb)` in `.myc` equals `a.rank() <= b.rank()` (a is at least as strong as b)
/// for ALL 16 pairs, driven against the live oracle's rank order.
#[test]
fn is_stronger_eq_matches_core_rank_order_all_16_pairs() {
    for &a in &ALL_STRENGTHS {
        for &b in &ALL_STRENGTHS {
            let expected = a.rank() <= b.rank();
            let driver = format!(
                "fn main() => Bool = is_stronger_eq({}, {});",
                myc_guarantee(a),
                myc_guarantee(b)
            );
            let src = program(&driver);
            assert_three_way(
                &format!("is_stronger_eq({a:?},{b:?})"),
                &src,
                &ref_bool(expected),
            );
        }
    }
}

// ── meet_all — mirror of GuaranteeStrength::meet_all (the empty sequence is the Exact identity) ────

/// `rank(meet_all([..]))` in `.myc` equals `GuaranteeStrength::meet_all(..).rank()` over a spread of
/// sequences, INCLUDING the empty sequence (→ Exact, the TOP identity — a fabricated non-Exact here
/// would be a silent upgrade). Driven against the live oracle.
#[test]
fn meet_all_matches_core_oracle() {
    let sequences: &[&[GuaranteeStrength]] = &[
        &[],
        &[GuaranteeStrength::Proven],
        &[GuaranteeStrength::Proven, GuaranteeStrength::Empirical],
        &[
            GuaranteeStrength::Exact,
            GuaranteeStrength::Proven,
            GuaranteeStrength::Exact,
        ],
        &[
            GuaranteeStrength::Proven,
            GuaranteeStrength::Declared,
            GuaranteeStrength::Empirical,
        ],
    ];
    for seq in sequences {
        // The empty list is `Nil` (unambiguous element type); a non-empty list is the `[..]`
        // literal that desugars to the local `Vec` cons-list (the `std.diag::matrix()` convention).
        let list = if seq.is_empty() {
            "Nil".to_owned()
        } else {
            let items: Vec<&str> = seq.iter().map(|g| myc_guarantee(*g)).collect();
            format!("[{}]", items.join(", "))
        };
        let expected = GuaranteeStrength::meet_all(seq.iter().copied());
        let driver = format!("fn main() => Binary{{8}} = rank(meet_all({list}));");
        let src = program(&driver);
        assert_three_way(
            &format!("meet_all({seq:?})"),
            &src,
            &ref_u8(expected.rank()),
        );
    }
}

// ── basis_strength — mirror of BoundBasis::strength (the honest tag = exactly what the basis supports) ──

/// One `(Rust BoundBasis, .myc constructor)` differential pair. The `.myc` mirror is Float-free (the
/// magnitude lives in `BoundKind`, FLAGged), so only the strength-determining shape is carried.
struct BasisCase {
    /// The live Rust oracle basis (its `.strength()` is the ground truth).
    rust: BoundBasis,
    /// The `.myc` `BoundBasis` mirror constructor expression.
    myc: &'static str,
}

/// The three `BoundBasis` shapes, each paired with its `.myc` mirror. Built once so `basis_strength`
/// and `attach` share the exact same oracle cases.
fn basis_cases() -> Vec<BasisCase> {
    vec![
        BasisCase {
            rust: BoundBasis::ProvenThm {
                citation: "ADR-010 §1 affine-arithmetic ε-composition".to_owned(),
            },
            myc: "BProvenThm(\"ADR-010 §1 affine-arithmetic ε-composition\")",
        },
        BasisCase {
            rust: BoundBasis::EmpiricalFit {
                trials: 100,
                method: "monte-carlo".to_owned(),
            },
            myc: "BEmpiricalFit(0b0110_0100, \"monte-carlo\")",
        },
        BasisCase {
            rust: BoundBasis::UserDeclared,
            myc: "BUserDeclared",
        },
    ]
}

/// `rank(basis_strength(Bx))` in `.myc` equals `BoundBasis::strength().rank()` for each basis shape,
/// driven against the live oracle. This is the core VR-5 rule — the tag is exactly what the basis
/// supports (ProvenThm→Proven, EmpiricalFit→Empirical, UserDeclared→Declared), never asserted.
///
/// Mutation witness: mapping `BUserDeclared` to `GProven` (an upgrade with no checked basis) flips
/// the computed rank and fails here.
#[test]
fn basis_strength_matches_core_oracle() {
    for case in basis_cases() {
        let expected = case.rust.strength().rank();
        let driver = format!(
            "fn main() => Binary{{8}} = rank(basis_strength({}));",
            case.myc
        );
        let src = program(&driver);
        assert_three_way(
            &format!("basis_strength({:?})", case.rust),
            &src,
            &ref_u8(expected),
        );
    }
}

// ── Approx carrier — attach derives strength from basis (FR-N1/VR-5); value/basis projections ──────

/// `rank(approx_strength(attach(True, Bx)))` in `.myc` equals the basis-derived strength rank for
/// each basis — the `Approx` carrier stores and projects the DERIVED strength (never a caller-set
/// field). Mirrors `mycelium_std_numerics::Approx::attach` + `strength`. Driven against the live
/// `BoundBasis::strength` oracle (the strength is derived identically).
#[test]
fn attach_derives_strength_from_basis_matches_core_oracle() {
    for case in basis_cases() {
        let expected = case.rust.strength().rank();
        let driver = format!(
            "fn main() => Binary{{8}} = rank(approx_strength(attach(True, {})));",
            case.myc
        );
        let src = program(&driver);
        assert_three_way(
            &format!("attach/approx_strength({:?})", case.rust),
            &src,
            &ref_u8(expected),
        );
    }
}

/// `approx_value(attach(v, basis))` projects the carried value unchanged (total projection, Exact).
#[test]
fn approx_value_projects_carried_value() {
    let driver = "fn main() => Bool = approx_value(attach(True, BUserDeclared));";
    let src = program(driver);
    assert_three_way("approx_value projection", &src, &ref_bool(true));
}

// ── combine_strength — the STRENGTH half of FR-N2 combine, EXHAUSTIVE over the 9 reachable pairs ───

/// `rank(combine_strength(attach(True, Ba), attach(True, Bb)))` equals `a.strength().meet(b.strength()).rank()`
/// for ALL 9 basis pairs — the analogue of the Rust
/// `combine_strength_is_meet_for_all_strength_pairs_exhaustive` test, restricted to the strength half
/// (the ε-composition half is float-valued, FLAGged). Driven against the live `meet` oracle over the
/// live `BoundBasis::strength` values.
///
/// Mutation witness: replacing the `.myc` `meet` with any other combinator (e.g. strongest-wins)
/// breaks at least one pair here — the same guard the Rust exhaustive test provides.
#[test]
fn combine_strength_is_meet_matches_core_oracle_all_9_pairs() {
    let cases = basis_cases();
    for a in &cases {
        for b in &cases {
            let expected = a.rust.strength().meet(b.rust.strength()).rank();
            let driver = format!(
                "fn main() => Binary{{8}} = rank(combine_strength(attach(True, {}), attach(True, {})));",
                a.myc, b.myc
            );
            let src = program(&driver);
            assert_three_way(
                &format!("combine_strength({:?},{:?})", a.rust, b.rust),
                &src,
                &ref_u8(expected),
            );
        }
    }
}

// ── NumErr — discriminant codes tied to the LIVE mycelium_std_numerics::NumErr variant set ─────────

/// One `NumErr` differential case: the live Rust variant, its `.myc` mirror constructor, and the
/// stable discriminant code the port assigns. The match below is EXHAUSTIVE (no wildcard), so adding
/// or removing a `NumErr` variant in the Rust oracle breaks compilation here — a compile-time
/// differential, never a silent drift.
fn num_err_case(e: &NumErr) -> (&'static str, u8) {
    match e {
        NumErr::BadEps => ("BadEps", 0),
        NumErr::BadDelta => ("BadDelta", 1),
        NumErr::NoRule => ("NoRule", 2),
        NumErr::NormMismatch => ("NormMismatch", 3),
        NumErr::Overflow => ("Overflow", 4),
    }
}

/// `num_err_code(Variant)` in `.myc` equals the code paired with each live `NumErr` variant. The
/// explicit `NumErr` value list plus the exhaustive `num_err_case` match together assert the port's
/// refusal-record variant set matches the Rust oracle's, variant-for-variant.
#[test]
fn num_err_code_matches_oracle_variant_set() {
    let variants = [
        NumErr::BadEps,
        NumErr::BadDelta,
        NumErr::NoRule,
        NumErr::NormMismatch,
        NumErr::Overflow,
    ];
    for v in &variants {
        let (ctor, code) = num_err_case(v);
        let driver = format!("fn main() => Binary{{8}} = num_err_code({ctor});");
        let src = program(&driver);
        assert_three_way(&format!("num_err_code({ctor})"), &src, &ref_u8(code));
    }
}

// ── CheckErr — is_malformed tied to the LIVE mycelium_std_numerics::CheckErr variant set ───────────

/// The exhaustive `.myc`-portable discriminator over the live `CheckErr` oracle: `Malformed` → true,
/// `Rejected{..}` → false. Exhaustive match (no wildcard) — a variant change breaks compilation.
///
/// NOTE (FLAG-num-1): only the `Malformed` arm is EXERCISABLE in `.myc` today — the `Rejected(Float,
/// Float)` arm needs a scalar-float VALUE, which the `.myc` runtime does not have yet (`enb` Gap A).
/// So this differential drives the `Malformed` case against the oracle; the `Rejected`-arm value is
/// asserted on the Rust side only (documenting the honest scope limit, never faking a float).
fn check_err_is_malformed(c: &CheckErr) -> bool {
    match c {
        CheckErr::Malformed => true,
        CheckErr::Rejected { .. } => false,
    }
}

/// `is_malformed(Malformed)` in `.myc` equals the oracle's `Malformed`-arm verdict (true). The
/// `Rejected` arm's oracle verdict (false) is confirmed Rust-side; its `.myc` evaluation is blocked
/// on the Float-value enabler (FLAG-num-1) and is exercised once floats land.
#[test]
fn is_malformed_matches_oracle_variant_set() {
    // Oracle: Malformed is malformed; a Rejected verdict is not (float args are placeholders — the
    // oracle discriminates on the constructor, not the magnitudes).
    assert!(check_err_is_malformed(&CheckErr::Malformed));
    assert!(!check_err_is_malformed(&CheckErr::Rejected {
        recomputed: 0.3,
        claimed: 0.1,
    }));

    let driver = "fn main() => Bool = is_malformed(Malformed);";
    let src = program(driver);
    assert_three_way(
        "is_malformed(Malformed)",
        &src,
        &ref_bool(check_err_is_malformed(&CheckErr::Malformed)),
    );
}
