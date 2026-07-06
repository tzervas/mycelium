//! M-740 Stage 5, increment 1 (DN-26 §7.3 row 5 / §9 flag-1) — the self-hosted `compiler.semcore`
//! PARTIAL first increment: the type vocabulary (`Ty`/`Width`/`DataInfo`/`CtorInfo`) plus the
//! Maranget usefulness/decision-tree pipeline, the affine use-once tracker, and the static
//! guarantee-grading pass. `lib/compiler/semcore.myc`'s own header documents in full what is IN
//! this increment and what is DEFERRED (fuse.rs / checkty.rs's checking logic / elab.rs / eval.rs
//! / mono.rs — feasibility-gated on M-986/M-987); this file is the unit differential gate for the
//! ported subset only.
//!
//! **A real, newly-surfaced finding this gate must document (semcore.myc's own FLAG-semcore-10):**
//! unlike every prior Stage (`ast.rs`/`token.rs`/`parse.rs`/`nodule.rs`, all `pub mod` with their
//! port-relevant items re-exported `pub`), the four Rust modules this increment ports —
//! `usefulness.rs`, `decision.rs`, `affine.rs`, `grade.rs` — are declared `pub(crate) mod` / plain
//! `mod` in `crates/mycelium-l1/src/lib.rs`. An external integration-test crate (this file) sees
//! only the crate's PUBLIC API surface, so it cannot call `usefulness::useful`,
//! `decision::compile`, affine's `Tracker`/`use_at`, or `grade::check_guarantees` directly — the
//! DN-26 §7.4 "checked against a live Rust oracle" contract every prior stage's differential relies
//! on does not hold here, and Rust sources are READ-ONLY in this leaf (no visibility change is this
//! leaf's call to make). So instead of a live-oracle comparison, every case below asserts the
//! `.myc` port's verdict against a HAND-DERIVED expected value, with the derivation reasoned in a
//! comment at each call site (never a fabricated/guessed constant) — grading this differential
//! `Empirical`, not the stronger "vs. a live oracle" posture (VR-5). FLAGGED UP for the
//! maintainer/DN-26 owner: lifting these four modules to `pub` (or a dedicated `#[cfg(test)]`-gated
//! shim) would let a future revision restore a live-oracle differential.
//!
//! M-981 applies as in every prior self-hosted-compiler-scale stage: only the L1-eval leg is
//! exercised (the L0-substitution interpreter / AOT three-way smoke is skipped for this partial
//! increment — every input here is a small synthetic fixture, not drawn from a corpus, so the
//! marginal value of a three-way leg is low relative to its eval-depth cost, M-987).

use mycelium_core::Payload;
use mycelium_l1::elab::build_registry;
use mycelium_l1::{check_nodule, monomorphize, parse, Evaluator};

/// Extract a `Binary{N}` `CoreValue`'s bits as a `u32` (MSB-first) — the established convention
/// from every prior stage's harness (`compiler_stage1.rs`'s own `core_bits_as_u32`).
fn core_bits_as_u32(v: &mycelium_core::CoreValue) -> u32 {
    let repr_val = v
        .as_repr()
        .unwrap_or_else(|| panic!("expected a Repr CoreValue, got {v:?}"));
    match repr_val.payload() {
        Payload::Bits(bits) => bits.iter().fold(0u32, |acc, &b| (acc << 1) | u32::from(b)),
        other => panic!("expected a Bits payload, got {other:?}"),
    }
}

const SEMCORE_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../lib/compiler/semcore.myc"
));

/// The shared driver prelude: small `.myc` encoder helpers every scenario's `main()` calls, plus
/// two small shared fixtures (`opt2_types`, `optlist_types`). Kept separate from each scenario's
/// own `main()` so every test only differs in the one expression under test.
fn driver_prelude() -> String {
    r#"
// ---- shared fixtures -----------------------------------------------------------------------
// Opt2: a 2-nullary-ctor enum (like `Bool`) — the smallest possible finite signature.
fn opt2_types() => Vec[DataInfo] =
  Cons(DI("Opt2", Nil, Cons(CI("OA", Nil), Cons(CI("OB", Nil), Nil))), Nil);

// OptList: a recursive 1-field-ctor + nullary-ctor enum (like `Vec`) — exercises a non-nullary
// constructor's field-type expansion (`Vec[DataInfo]` order: OptList before Opt2, since ONil's
// field references Opt2 and lookup is order-independent — the checker's own shell-first
// registration note applies equally to this VALUE-level registry list, which is looked up by
// name, not position).
fn optlist_types() => Vec[DataInfo] =
  Cons(DI("OptList", Nil, Cons(CI("OCons", Cons(TyData("Opt2", Nil), Cons(TyData("OptList", Nil), Nil))), Cons(CI("ONil", Nil), Nil))),
  Cons(DI("Opt2", Nil, Cons(CI("OA", Nil), Cons(CI("OB", Nil), Nil))), Nil));

// ---- usefulness verdict encoders -----------------------------------------------------------
fn useful_verdict(res: Result[Option[Vec[Pat]], Bytes]) => Binary{32} =
  match res {
    Err(_) => 0b0000_0000_0000_0000_0000_0000_0000_0011,
    Ok(o) => match o {
      None => 0b0000_0000_0000_0000_0000_0000_0000_0000,
      Some(_) => 0b0000_0000_0000_0000_0000_0000_0000_0001
    }
  };

fn useful_witness_is(res: Result[Option[Vec[Pat]], Bytes], want: Bytes) => Binary{32} =
  match res {
    Err(_) => 0b0000_0000_0000_0000_0000_0000_0000_0000,
    Ok(o) => match o {
      None => 0b0000_0000_0000_0000_0000_0000_0000_0000,
      Some(w) => match bytes_eq(render_list(w), want) {
        0b1 => 0b0000_0000_0000_0000_0000_0000_0000_0001,
        _ => 0b0000_0000_0000_0000_0000_0000_0000_0000
      }
    }
  };

// ---- decision verdict encoders -------------------------------------------------------------
fn compile_ok(res: Result[Tree, Bytes]) => Tree =
  match res { Ok(t) => t, Err(_) => Fail };

fn bool_code(b: Bool) => Binary{32} =
  match b { True => 0b0000_0000_0000_0000_0000_0000_0000_0001, False => 0b0000_0000_0000_0000_0000_0000_0000_0000 };

// tree_arm_code: the routed arm index, or the sentinel `0xFFFF_FFFF` for a reached `Fail` (arm
// indices in every fixture below are 0/1, so the all-ones sentinel cannot collide).
fn tree_arm_code(o: Option[Binary{32}]) => Binary{32} =
  match o { None => 0b1111_1111_1111_1111_1111_1111_1111_1111, Some(a) => a };

// ---- affine verdict encoders ----------------------------------------------------------------
fn outcome_code(o: UseOutcome) => Binary{32} =
  match o {
    NotAffine => 0b0000_0000_0000_0000_0000_0000_0000_0000,
    FirstUse => 0b0000_0000_0000_0000_0000_0000_0000_0001,
    DoubleUse(_, _, _) => 0b0000_0000_0000_0000_0000_0000_0000_0010
  };

fn slot_code(s: Slot) => Binary{32} =
  match s {
    Skip => 0b0000_0000_0000_0000_0000_0000_0000_0000,
    Live(_) => 0b0000_0000_0000_0000_0000_0000_0000_0001,
    Moved(_, _) => 0b0000_0000_0000_0000_0000_0000_0000_0010
  };

fn slots_first(v: Vec[Slot]) => Slot =
  match v { Cons(h, _) => h, Nil => Skip };

// ---- grade verdict encoders ------------------------------------------------------------------
fn strength_code(s: Strength) => Binary{32} =
  match s {
    GDeclared => 0b0000_0000_0000_0000_0000_0000_0000_0000,
    GEmpirical => 0b0000_0000_0000_0000_0000_0000_0000_0001,
    GProven => 0b0000_0000_0000_0000_0000_0000_0000_0010,
    GExact => 0b0000_0000_0000_0000_0000_0000_0000_0011
  };

// grade_result_code: the sentinel `255` for `Err` (a real grading violation) — no valid
// `Strength` code below in this file's fixtures reaches that value.
fn grade_result_code(res: Result[Strength, CheckError]) => Binary{32} =
  match res {
    Err(_) => 0b0000_0000_0000_0000_0000_0000_1111_1111,
    Ok(s) => strength_code(s)
  };

fn bytes_ty() => TypeRef = TR(KwBytes, None);
fn bytes_ty_g(g: Strength) => TypeRef = TR(KwBytes, Some(g));
fn one_path(name: Bytes) => Path = Pth(Cons(name, Nil));
"#
    .to_owned()
}

fn program(driver: &str) -> String {
    format!("{SEMCORE_SRC}\n{}\n{driver}", driver_prelude())
}

/// L1-eval-only assertion (the M-981 convention every self-hosted-compiler-scale stage uses).
fn assert_l1_only_u32(label: &str, src: &str, expected_u32: u32) {
    let env = check_nodule(&parse(src).unwrap_or_else(|e| panic!("{label}: parse failed: {e}")))
        .unwrap_or_else(|e| panic!("{label}: check failed: {e}"));
    let mono =
        monomorphize(&env, "main").unwrap_or_else(|e| panic!("{label}: monomorphize failed: {e}"));
    let registry =
        build_registry(&mono).unwrap_or_else(|e| panic!("{label}: build_registry failed: {e}"));
    let l1_val = Evaluator::new(&mono)
        .call("main", vec![])
        .unwrap_or_else(|e| panic!("{label}: L1-eval failed: {e}"));
    let l1_core = l1_val
        .to_core(&mono, &registry)
        .unwrap_or_else(|| panic!("{label}: L1 result is outside the r3 data fragment"));
    let got = core_bits_as_u32(&l1_core);
    assert_eq!(
        got, expected_u32,
        "{label}: L1-eval result {got} does not match the hand-derived expected value {expected_u32}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// The structural gate: `semcore.myc` parses and type-checks green (no driver needed).
// ─────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn semcore_myc_parses_and_checks() {
    let nodule = parse(SEMCORE_SRC).unwrap_or_else(|e| panic!("semcore.myc: parse failed: {e}"));
    check_nodule(&nodule).unwrap_or_else(|e| panic!("semcore.myc: check failed: {e}"));
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// usefulness.rs: `useful` — a couple of exhaustive (not-useful) cases and a couple of
// non-exhaustive (useful, with a checked witness) cases.
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// Exhaustive: the matrix already covers both `Opt2` constructors (`OA`, `OB`), so `_` is NOT
/// useful w.r.t. it — hand-derived: a 2-row matrix `[[OA]], [[OB]]]` against a 2-ctor signature is
/// complete by definition, so `U(P, [_])` must be `None` (Maranget's own completeness rule).
#[test]
fn useful_exhaustive_two_ctors() {
    let driver = r#"
fn main() => Binary{32} =
  useful_verdict(useful(opt2_types(),
    Cons(Cons(MpCtor("OA", Nil), Nil), Cons(Cons(MpCtor("OB", Nil), Nil), Nil)),
    Cons(MpWild, Nil),
    Cons(TyData("Opt2", Nil), Nil)));
"#;
    assert_l1_only_u32("useful: OA+OB matrix, _ is not useful", &program(driver), 0);
}

/// Exhaustive (redundancy-flavored): a PRIOR wildcard row already subsumes everything, so a later
/// arm's own concrete pattern is NOT useful w.r.t. it — hand-derived: `U([[_]], [OA])` is `None`
/// because the wildcard row matches every value including any `OA`-headed one (Maranget's own
/// subsumption rule for a `Wild` row).
#[test]
fn useful_exhaustive_prior_wildcard_subsumes() {
    let driver = r#"
fn main() => Binary{32} =
  useful_verdict(useful(opt2_types(),
    Cons(Cons(MpWild, Nil), Nil),
    Cons(MpCtor("OA", Nil), Nil),
    Cons(TyData("Opt2", Nil), Nil)));
"#;
    assert_l1_only_u32(
        "useful: prior wildcard row subsumes OA, not useful",
        &program(driver),
        0,
    );
}

/// Non-exhaustive: the matrix covers only `OA`, so `_` IS useful w.r.t. it, with witness `OB` —
/// hand-derived: `Opt2`'s signature is `{OA, OB}`; only `OA` is present, so the missing
/// constructor `OB` (nullary — a bare witness head, no sub-witnesses) is exactly Maranget's
/// "incomplete signature" witness.
#[test]
fn useful_non_exhaustive_missing_ob() {
    let driver = r#"
fn main() => Binary{32} =
  useful_verdict(useful(opt2_types(),
    Cons(Cons(MpCtor("OA", Nil), Nil), Nil),
    Cons(MpWild, Nil),
    Cons(TyData("Opt2", Nil), Nil)));
"#;
    assert_l1_only_u32("useful: OA-only matrix, _ is useful", &program(driver), 1);

    let driver_witness = r#"
fn main() => Binary{32} =
  useful_witness_is(useful(opt2_types(),
    Cons(Cons(MpCtor("OA", Nil), Nil), Nil),
    Cons(MpWild, Nil),
    Cons(TyData("Opt2", Nil), Nil)), "OB");
"#;
    assert_l1_only_u32(
        "useful: OA-only matrix witness renders as OB",
        &program(driver_witness),
        1,
    );
}

/// Non-exhaustive over a 2-column matrix (exercising the recursive column-2 case + the
/// non-nullary-constructor field expansion via `OptList::OCons`) — hand-derived: column 0 is
/// `OptList` with only `ONil` present, so `OCons` is missing (witness head `OCons` with TWO
/// wildcard sub-fields, its arity). The single row is headed by `ONil` (not a wildcard), so it is
/// DROPPED entirely by `default_matrix` when recursing into column 1 — that recursive call sees
/// ZERO rows, so column 1's OWN missing-ctor search (over its `Opt2` signature, with nothing
/// present) independently reports Opt2's FIRST ctor `OA` — not a generic `_` — exactly like column
/// 0's own witness search does when nothing is present. (Verified by tracing the ported algorithm
/// step-by-step, matching usefulness.rs's Rust structure: an empty sub-matrix's "missing ctor" is
/// always the signature's first ctor, since `present` is empty for every finite column reached
/// with no constraining rows left.)
#[test]
fn useful_non_exhaustive_two_columns() {
    let driver = r#"
fn main() => Binary{32} =
  useful_witness_is(useful(optlist_types(),
    Cons(Cons(MpCtor("ONil", Nil), Cons(MpWild, Nil)), Nil),
    Cons(MpWild, Cons(MpWild, Nil)),
    Cons(TyData("OptList", Nil), Cons(TyData("Opt2", Nil), Nil))), "OCons(_, _), OA");
"#;
    assert_l1_only_u32(
        "useful: OptList ONil-only 2-column matrix witness is OCons(_, _), OA",
        &program(driver),
        1,
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// decision.rs: `compile` + `has_reachable_fail` + `tree_eval` — an exhaustive 2-arm match and a
// deliberately-incomplete 1-arm match fed directly to `compile` (bypassing surface exhaustiveness
// checking, which this partial increment does not port).
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// Exhaustive: `compile` on `[[OA]] -> arm0, [[OB]] -> arm1` produces a tree with no reachable
/// `Fail` — hand-derived: both `Opt2` constructors are covered, so `complete = True` and no
/// `default` branch is built (decision.rs's own `complete` short-circuit).
#[test]
fn compile_exhaustive_no_reachable_fail() {
    let driver = r#"
fn main() => Binary{32} =
  bool_code(has_reachable_fail(compile_ok(compile(opt2_types(),
    Cons(Cons(MpCtor("OA", Nil), Nil), Cons(Cons(MpCtor("OB", Nil), Nil), Nil)),
    Cons(0b0000_0000_0000_0000_0000_0000_0000_0000, Cons(0b0000_0000_0000_0000_0000_0000_0000_0001, Nil)),
    Cons(Nil, Nil),
    Cons(TyData("Opt2", Nil), Nil)))));
"#;
    assert_l1_only_u32(
        "compile: exhaustive OA/OB match has no reachable Fail",
        &program(driver),
        0,
    );
}

/// Both concrete inputs route to their hand-known arm via `tree_eval` on the same exhaustive tree.
#[test]
fn compile_exhaustive_routes_to_expected_arms() {
    let driver_oa = r#"
fn main() => Binary{32} =
  tree_arm_code(tree_eval(compile_ok(compile(opt2_types(),
    Cons(Cons(MpCtor("OA", Nil), Nil), Cons(Cons(MpCtor("OB", Nil), Nil), Nil)),
    Cons(0b0000_0000_0000_0000_0000_0000_0000_0000, Cons(0b0000_0000_0000_0000_0000_0000_0000_0001, Nil)),
    Cons(Nil, Nil),
    Cons(TyData("Opt2", Nil), Nil))), MpCtor("OA", Nil)));
"#;
    assert_l1_only_u32("compile: OA routes to arm 0", &program(driver_oa), 0);

    let driver_ob = r#"
fn main() => Binary{32} =
  tree_arm_code(tree_eval(compile_ok(compile(opt2_types(),
    Cons(Cons(MpCtor("OA", Nil), Nil), Cons(Cons(MpCtor("OB", Nil), Nil), Nil)),
    Cons(0b0000_0000_0000_0000_0000_0000_0000_0000, Cons(0b0000_0000_0000_0000_0000_0000_0000_0001, Nil)),
    Cons(Nil, Nil),
    Cons(TyData("Opt2", Nil), Nil))), MpCtor("OB", Nil)));
"#;
    assert_l1_only_u32("compile: OB routes to arm 1", &program(driver_ob), 1);
}

/// Incomplete: `compile` on a 1-arm matrix covering only `OA` produces a tree WITH a reachable
/// `Fail` (the `default` branch) — hand-derived: `OB` is not covered, so `complete = False` and
/// `compile_rows` on the (empty) default rows returns `Fail` directly.
#[test]
fn compile_incomplete_has_reachable_fail() {
    let driver = r#"
fn main() => Binary{32} =
  bool_code(has_reachable_fail(compile_ok(compile(opt2_types(),
    Cons(Cons(MpCtor("OA", Nil), Nil), Nil),
    Cons(0b0000_0000_0000_0000_0000_0000_0000_0000, Nil),
    Cons(Nil, Nil),
    Cons(TyData("Opt2", Nil), Nil)))));
"#;
    assert_l1_only_u32(
        "compile: OA-only match has a reachable Fail",
        &program(driver),
        1,
    );

    // The uncovered `OB` input routes to that reachable `Fail` (tree_eval -> None -> sentinel).
    let driver_fail = r#"
fn main() => Binary{32} =
  tree_arm_code(tree_eval(compile_ok(compile(opt2_types(),
    Cons(Cons(MpCtor("OA", Nil), Nil), Nil),
    Cons(0b0000_0000_0000_0000_0000_0000_0000_0000, Nil),
    Cons(Nil, Nil),
    Cons(TyData("Opt2", Nil), Nil))), MpCtor("OB", Nil)));
"#;
    assert_l1_only_u32(
        "compile: OB (uncovered) routes to the reachable Fail",
        &program(driver_fail),
        0xFFFF_FFFF,
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// affine.rs: `slots_use_at` (first use, then a double-consume) + `union_merge_into` (conservative
// branch merge).
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// First use of a live `Substrate` binding: hand-derived `FirstUse` (mirrors
/// `affine.rs::Tracker::use_at`'s `Live -> Moved` transition).
#[test]
fn affine_first_use_is_first_use() {
    let driver = r#"
fn main() => Binary{32} =
  match slots_use_at(slots_seeded(Cons(TySubstrate("H"), Nil)), 0b0000_0000_0000_0000_0000_0000_0000_0000, 0b0000_0000_0000_0000_0000_0000_0000_0000) {
    Pr(_, outcome) => outcome_code(outcome)
  };
"#;
    assert_l1_only_u32(
        "affine: first use of a live Substrate is FirstUse",
        &program(driver),
        1,
    );
}

/// A second use of the SAME binding (feeding the updated slots back in) is a double-consume —
/// hand-derived `DoubleUse` (mirrors `affine.rs::Tracker::use_at`'s `Moved -> DoubleUse` outcome).
#[test]
fn affine_second_use_is_double_use() {
    let driver = r#"
fn main() => Binary{32} =
  match slots_use_at(slots_seeded(Cons(TySubstrate("H"), Nil)), 0b0000_0000_0000_0000_0000_0000_0000_0000, 0b0000_0000_0000_0000_0000_0000_0000_0000) {
    Pr(slots2, _) => match slots_use_at(slots2, 0b0000_0000_0000_0000_0000_0000_0000_0000, 0b0000_0000_0000_0000_0000_0000_0000_0001) {
      Pr(_, outcome2) => outcome_code(outcome2)
    }
  };
"#;
    assert_l1_only_u32(
        "affine: second use of the same binding is DoubleUse",
        &program(driver),
        2,
    );
}

/// `union_merge_into` — a slot `Live` in `acc` but `Moved` in `other` becomes `Moved` (the
/// conservative "moved in either branch is moved afterward" rule) — hand-derived: `merge([Live],
/// [Moved]) = [Moved]`.
#[test]
fn affine_union_merge_moved_wins_over_live() {
    let driver = r#"
fn main() => Binary{32} =
  slot_code(slots_first(union_merge_into(
    Cons(Live("H"), Nil),
    Cons(Moved("H", 0b0000_0000_0000_0000_0000_0000_0000_0000), Nil))));
"#;
    assert_l1_only_u32(
        "affine: union_merge_into -- Moved wins over Live",
        &program(driver),
        2,
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// grade.rs: `grade_fn_body` — Let/If/Ascribe/App(call)/Wild, both an accepted and a demand-
// violation ("Err") outcome, mirroring RFC-0018 §4.3's documented rules exactly.
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// `let x = "s" in x` with no ascription: bound grade `Exact` (a literal), bound at `Exact`, body
/// (the path `x`) looks it up as `Exact`; the let's grade is `meet(Exact, Exact) = Exact` —
/// hand-derived per G-Let (no annotation to weaken against).
#[test]
fn grade_let_no_ascription_is_exact() {
    let driver = r#"
fn main_fn() => FnDecl =
  FD(Private, False, None,
     FS("main_fn", Nil, Nil, bytes_ty(), Nil, Nil),
     Let("x", None, Lit(Str("s")), Path(one_path("x"))));
fn main() => Binary{32} =
  grade_result_code(grade_fn_body(Nil, main_fn()));
"#;
    assert_l1_only_u32("grade: unascribed let is Exact", &program(driver), 3);
}

/// `"s" @ Empirical`: body grade `Exact` satisfies the `Empirical` demand (rank 3 >= rank 1), so
/// the ascription succeeds and the result carries the WEAKENED `Empirical` grade — hand-derived
/// per G-Weaken (the ascription may only weaken, never upgrade — VR-5).
#[test]
fn grade_ascribe_weakens_to_empirical() {
    let driver = r#"
fn main_fn() => FnDecl =
  FD(Private, False, None,
     FS("main_fn", Nil, Nil, bytes_ty(), Nil, Nil),
     Ascribe(Lit(Str("s")), bytes_ty_g(GEmpirical)));
fn main() => Binary{32} =
  grade_result_code(grade_fn_body(Nil, main_fn()));
"#;
    assert_l1_only_u32(
        "grade: Exact literal ascribed @ Empirical weakens to Empirical",
        &program(driver),
        1,
    );
}

/// `wild(...) @ Exact`: `wild` is graded `Declared` (the FFI floor) regardless of its body, which
/// does NOT satisfy the `Exact` demand (rank 0 < rank 3) — hand-derived per G-Weaken's failure
/// case: an `Err` result (encoded as the `255` sentinel).
#[test]
fn grade_ascribe_wild_violates_exact_demand() {
    let driver = r#"
fn main_fn() => FnDecl =
  FD(Private, False, None,
     FS("main_fn", Nil, Nil, bytes_ty(), Nil, Nil),
     Ascribe(Wild(Lit(Str("s"))), bytes_ty_g(GExact)));
fn main() => Binary{32} =
  grade_result_code(grade_fn_body(Nil, main_fn()));
"#;
    assert_l1_only_u32(
        "grade: wild @ Exact is a demand violation (Err)",
        &program(driver),
        0xFF,
    );
}

/// Design A: `if "c" then ("t" @ Proven) else wild("f")` — the condition's grade does not degrade
/// the result; the consequent grades `Proven` (Exact satisfies the Proven demand), the alternate
/// grades `Declared` (wild); the if's result is `meet(Proven, Declared) = Declared` — hand-derived
/// per RFC-0018 §4.5 G-Match/A (restated for `if`).
#[test]
fn grade_if_meets_branch_bodies_not_condition() {
    let driver = r#"
fn main_fn() => FnDecl =
  FD(Private, False, None,
     FS("main_fn", Nil, Nil, bytes_ty(), Nil, Nil),
     If(Lit(Str("c")), Ascribe(Lit(Str("t")), bytes_ty_g(GProven)), Wild(Lit(Str("f")))));
fn main() => Binary{32} =
  grade_result_code(grade_fn_body(Nil, main_fn()));
"#;
    assert_l1_only_u32(
        "grade: if meets Proven and Declared branches down to Declared",
        &program(driver),
        0,
    );
}

/// G-App: a callee `idfn(x: Bytes @ Empirical) => Bytes @ Empirical = x` called with an `Exact`
/// literal argument. The argument's `Exact` grade satisfies the `Empirical` demand (rank 3 >= 1);
/// the call's result is the callee's DECLARED return grade `Empirical` (not the argument's own
/// `Exact` — G-App returns the advertised signature grade, never the caller's actual value grade)
/// — hand-derived directly from grade.rs's own G-App rule.
#[test]
fn grade_app_known_callee_satisfied_demand() {
    let driver = r#"
fn idfn_decl() => FnDecl =
  FD(Private, False, None,
     FS("idfn", Nil, Cons(Prm("x", bytes_ty_g(GEmpirical)), Nil), bytes_ty_g(GEmpirical), Nil, Nil),
     Path(one_path("x")));
fn main_fn() => FnDecl =
  FD(Private, False, None,
     FS("main_fn", Nil, Nil, bytes_ty(), Nil, Nil),
     App(Path(one_path("idfn")), Cons(Lit(Str("s")), Nil)));
fn main() => Binary{32} =
  grade_result_code(grade_fn_body(Cons(Pr("idfn", idfn_decl()), Nil), main_fn()));
"#;
    assert_l1_only_u32(
        "grade: App to a known callee returns its declared Empirical return grade",
        &program(driver),
        1,
    );
}

/// G-App failure: a callee demanding `Exact` on its parameter, called with a `wild`-graded
/// (`Declared`) argument — the argument does not satisfy the demand (rank 0 < rank 3) — hand-
/// derived: an `Err` result (the `255` sentinel).
#[test]
fn grade_app_known_callee_violates_demand() {
    let driver = r#"
fn strictfn_decl() => FnDecl =
  FD(Private, False, None,
     FS("strictfn", Nil, Cons(Prm("x", bytes_ty_g(GExact)), Nil), bytes_ty_g(GExact), Nil, Nil),
     Path(one_path("x")));
fn main_fn() => FnDecl =
  FD(Private, False, None,
     FS("main_fn", Nil, Nil, bytes_ty(), Nil, Nil),
     App(Path(one_path("strictfn")), Cons(Wild(Lit(Str("s"))), Nil)));
fn main() => Binary{32} =
  grade_result_code(grade_fn_body(Cons(Pr("strictfn", strictfn_decl()), Nil), main_fn()));
"#;
    assert_l1_only_u32(
        "grade: App to a known callee with a Declared-graded arg violates an Exact demand",
        &program(driver),
        0xFF,
    );
}
