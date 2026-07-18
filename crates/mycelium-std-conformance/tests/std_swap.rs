//! Differential tests for `std.swaps` (M-929, E29-1, kickoff `opp`, RFC-0031 D5) — the `.myc`
//! port of `crates/mycelium-std-swap/src/lib.rs`, the never-silent representation-change surface.
//!
//! # Scope (surface-check, D5 row 1 — see `lib/std/swap.myc`'s module doc for the full writeup)
//! Ported: (a) the RFC-0016 §4.5 `GUARANTEE_MATRIX` as data + the `assert_matrix_invariants`
//! contract as checkable structure; (b) the never-silent `CheckError`/`Fallback` model; (c) the
//! bijective binary<->ternary swap class through the language's own `swap(…, to: …, policy: …)`
//! construct at the RFC-0002 §5 legal pairs (8,6)/(4,3). FLAGged, not forced (VR-5/G2): the
//! kernel re-exports (`SwapCertificate`/`SwapError`/`Value`/… — RFC-0031 D1), `Swapped`/
//! `ExplainRecord` (kernel-typed fields), and the `f32_to_bf16`/`dense_to_vsa`/`vsa_to_dense`/
//! `check_swap`/`explain` ops (kernel-engine dispatch).
//!
//! # Harness design
//! Execution/comparison machinery lives in the shared [`harness`] fixture (M-925). This file
//! supplies the nodule's `include_str!`, per-case drivers, and — the row this port owns per the
//! harness doc (§4) — the **live Rust-oracle differential** against the retained
//! `mycelium-std-swap` crate (RFC-0031 D6; the crate is NOT retired): matrix expectations are
//! computed from `mycelium_std_swap::GUARANTEE_MATRIX` at test time, and the swap-value corpus is
//! evaluated on BOTH sides (the `.myc` surface through the L1 pipeline; the oracle through
//! `bin_to_tern`/`tern_to_bin`) and compared payload-for-payload — never hand-copied into only
//! one side.
//!
//! **Oracle honesty note (VR-5):** the L1/L0/AOT swap engine (`BinaryTernarySwapEngine`) and the
//! Rust oracle both dispatch to the *same* `mycelium-cert` kernel functions (RFC-0031 D1 — the
//! kernel stays Rust). This differential therefore validates the **translation surface**
//! (parse → check → elaborate → each execution path surfaces identical kernel semantics), not two
//! independent swap implementations.
//!
//! # Never-silent reject-case conformance (the M-929 DoD extra — G2)
//! No silent swap path may survive translation. The reject section at the bottom pins each layer:
//! - **parse:** a `swap` missing its `policy:` (or its `to:`) is a parse error (S1/WF2) —
//!   the fragment "a swap is never silent" is the parser's own diagnostic;
//! - **check:** an implicit cross-paradigm edge (no `swap` written) is an explicit
//!   `MissingConversion` refusal, never an inserted conversion;
//! - **runtime, all three paths:** an illegal pair and an out-of-range decode are explicit
//!   refusals from the L1 evaluator, the L0 interpreter, AND the AOT path — and the Rust oracle
//!   refuses the *same* instances (`IllegalPair`/`OutOfRange`), so no engine on either side has a
//!   silent path (clamp/sentinel/wrap).
//!
//! # Honesty tags (VR-5 — never upgraded in translation)
//! - **`Exact`/`Proven`/`Empirical`** claims live in the ported matrix DATA at the same strength
//!   as the Rust rows; the transcription itself is `Declared` (asserted data, structurally
//!   checked below against the live oracle).
//! - **`Empirical`** — the three-way differential agreement (L1-eval ≡ L0-interp ≡ AOT) AND the
//!   Rust-oracle differential, validated by trial on the corpus in this file; neither is a
//!   machine-checked proof.
//!
//! # Pre-port polish (D5 row 2)
//! Recorded clean — no Rust-side change: `mycelium-std-swap` is the DN-66 frozen baseline
//! (2026-07-01); polishing a frozen oracle would be churn (G2: stated, not silently skipped).

mod harness;

use mycelium_core::{
    binary::int_to_bits, operation_hash, ternary::int_to_trits, GuaranteeStrength, Meta, Payload,
    Provenance, Repr, Trit, Value,
};
use mycelium_l1::elab::build_registry;
use mycelium_l1::{check_nodule, elaborate, monomorphize, parse, Evaluator};
use mycelium_std_swap::{
    assert_matrix_invariants, bin_to_tern, legal_pair, tern_to_bin, PolicyRef, SwapCertificate,
    SwapError, GUARANTEE_MATRIX,
};

/// The std.swaps nodule source, loaded at compile time — the single source of truth.
const SWAP_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../lib/std/swap.myc"
));

/// Build a full test program by appending a typed driver to the nodule source.
fn program(driver: &str) -> String {
    harness::program(SWAP_SRC, driver)
}

/// Thin re-export of the shared [`harness::assert_three_way`] (same pattern as `std_diag.rs`).
fn assert_three_way(label: &str, src: &str, expected_src: &str) {
    harness::assert_three_way(label, src, expected_src);
}

// ══════════════════════════════════════════════════════════════════════════════════════════════
// Fixtures — the corpus is data; test bodies are asserts over cases (house test-layout rule).
// Oracle-side values are built from the SAME bit/trit patterns the `.myc` literals denote, so the
// two sides can never drift apart silently.
// ══════════════════════════════════════════════════════════════════════════════════════════════

/// The 7 ported rows: (`.myc` row-constructor name, oracle op name) — in `GUARANTEE_MATRIX` order.
const ROWS: &[(&str, &str)] = &[
    ("row_bin_to_tern", "bin_to_tern"),
    ("row_tern_to_bin", "tern_to_bin"),
    ("row_f32_to_bf16", "f32_to_bf16"),
    ("row_dense_to_vsa", "dense_to_vsa"),
    ("row_vsa_to_dense", "vsa_to_dense"),
    ("row_check_swap", "check_swap"),
    ("row_explain", "explain"),
];

/// The `Binary{8}` differential corpus: the sign boundary (two's-complement read), zero/one, the
/// crate tests' own 42/7 values, and both all-bits edges.
const BIN8_CORPUS: &[u8] = &[0x00, 0x01, 0x07, 0x2A, 0x7F, 0x80, 0xB2, 0xFE, 0xFF];

/// The in-range `Ternary{6}` decode corpus (Binary{8}'s two's-complement range is [-128, 127]).
const TERN6_CORPUS: &[i64] = &[-128, -78, -1, 0, 1, 13, 127];

/// A canonical policy hash for the oracle side (stands in for a real RFC-0005 policy; the payload
/// comparison is policy-independent — the policy only enters the value's `Meta`).
fn test_policy() -> PolicyRef {
    operation_hash("test.policy.std_swap_port.v1")
}

/// Build a `Binary{n}` oracle value from a signed integer (two's complement, MSB-first — the same
/// bit pattern the `.myc` `0b…` literal of that width denotes).
fn make_binary(value: i64, width: u32) -> Value {
    let bits = int_to_bits(value, width)
        .unwrap_or_else(|| panic!("value {value} does not fit in {width} bits"));
    let meta = Meta::new(
        Provenance::Root,
        GuaranteeStrength::Exact,
        None,
        None,
        None,
        None,
    )
    .expect("meta is well-formed");
    Value::new(Repr::Binary { width }, Payload::Bits(bits), meta)
        .expect("binary value is well-formed")
}

/// Build a `Ternary{m}` oracle value from a signed integer (balanced, MSB-first).
fn make_ternary(value: i64, trits: u32) -> Value {
    let tv = int_to_trits(i128::from(value), trits)
        .unwrap_or_else(|| panic!("value {value} does not fit in {trits} trits"));
    let meta = Meta::new(
        Provenance::Root,
        GuaranteeStrength::Exact,
        None,
        None,
        None,
        None,
    )
    .expect("meta is well-formed");
    Value::new(Repr::Ternary { trits }, Payload::Trits(tv), meta)
        .expect("ternary value is well-formed")
}

/// Render an unsigned bit pattern as the `.myc` `0b…` literal of exactly `width` digits.
fn bin_lit(v: u64, width: u32) -> String {
    format!("0b{v:0w$b}", w = width as usize)
}

/// Render a signed integer as the `.myc` `0t…` balanced-ternary literal of exactly `m` glyphs
/// (MSB-first; `-`/`0`/`+` — the wire glyphs of `binary-ternary.md` §1), via the same
/// `int_to_trits` codec the oracle values use.
fn tern_lit(v: i64, m: u32) -> String {
    let trits =
        int_to_trits(i128::from(v), m).unwrap_or_else(|| panic!("{v} does not fit in {m} trits"));
    let glyphs: String = trits
        .iter()
        .map(|t| match t {
            Trit::Neg => '-',
            Trit::Zero => '0',
            Trit::Pos => '+',
        })
        .collect();
    format!("0t{glyphs}")
}

/// Render a Rust `bool` as the `.myc` literal that denotes the same `Bool` value.
fn myc_bool(b: bool) -> &'static str {
    if b {
        "True"
    } else {
        "False"
    }
}

/// The `.myc` `Guarantee` constructor denoting an oracle [`GuaranteeStrength`] (the G-prefixed
/// keyword rename — FLAG-swap-0 in the nodule doc).
fn g_ctor(s: GuaranteeStrength) -> &'static str {
    match s {
        GuaranteeStrength::Exact => "GExact",
        GuaranteeStrength::Proven => "GProven",
        GuaranteeStrength::Empirical => "GEmpirical",
        GuaranteeStrength::Declared => "GDeclared",
    }
}

/// The `.myc` `CertKind` constructor denoting an oracle `cert_kind` field. An unknown kind is a
/// loud failure — the port must be extended, never silently bucketed (G2).
fn kind_ctor(k: Option<&str>) -> &'static str {
    match k {
        Some("Bijective") => "KBijective",
        Some("Bounded") => "KBounded",
        None => "KNone",
        Some(other) => panic!("oracle cert_kind {other:?} unknown to the port — extend CertKind"),
    }
}

/// The reference-program preamble redeclaring the ported ADTs an expected `main` returns
/// (the `std_ternary.rs` precedent: the ref program declares the same type shape locally).
const REF_TYPES: &str = "type Guarantee = GExact | GProven | GEmpirical | GDeclared;\n\
                         type CertKind = KBijective | KBounded | KNone;";

/// Render the `n`-deep nested `add_u(0b1, …)` expression `matrix_len`'s recursive spine-walk
/// expands to (the `std_diag.rs` precedent: the reference recomputes via the SAME primitive-op
/// composition, so it carries the matching `Derived` provenance while remaining an independent
/// check of the row count). `Binary{64}`-width literals (W-1 length/count canon, 2026-07-18 —
/// `matrix_len` itself moved `Binary{8}` -> `Binary{64}`).
fn myc_len_chain(n: u8) -> String {
    let mut expr =
        "0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000"
            .to_owned();
    for _ in 0..n {
        expr = format!(
            "add_u(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001, {expr})"
        );
    }
    expr
}

/// Run `driver`'s `main` through the L1 pipeline and return the result's kernel [`Payload`] —
/// the Rust-oracle bridge (same monomorphize/eval path as [`harness::assert_three_way`]; the
/// three-way obligation is carried by the paired `assert_three_way` case).
fn eval_payload(label: &str, driver: &str) -> Payload {
    let src = program(driver);
    let env = check_nodule(&parse(&src).unwrap_or_else(|e| panic!("{label}: parse failed: {e}")))
        .unwrap_or_else(|e| panic!("{label}: check failed: {e}"));
    let mono =
        monomorphize(&env, "main").unwrap_or_else(|e| panic!("{label}: monomorphize failed: {e}"));
    let registry =
        build_registry(&mono).unwrap_or_else(|e| panic!("{label}: build_registry failed: {e}"));
    let val = Evaluator::new(&mono)
        .call("main", vec![])
        .unwrap_or_else(|e| panic!("{label}: L1-eval failed: {e}"));
    let core = val
        .to_core(&mono, &registry)
        .unwrap_or_else(|| panic!("{label}: result is outside the r3 data fragment"));
    core.as_repr()
        .unwrap_or_else(|| panic!("{label}: expected a repr value"))
        .payload()
        .clone()
}

/// Assert that `driver` parses + checks, and then **every** execution path — the L1 fuel-guarded
/// evaluator, the elaborate→L0 reference interpreter, and the AOT env-machine — refuses at
/// runtime with an explicit error containing `fragment` (the intended-reason discipline of the
/// conformance corpus, A4). This is the "no silent swap path survives translation" gate (G2):
/// a wrong VALUE from any path would fail here just as loudly as a silent success.
fn assert_all_paths_refuse(label: &str, driver: &str, fragment: &str) {
    use mycelium_cert::BinaryTernarySwapEngine;
    use mycelium_interp::{Interpreter, PrimRegistry};

    let src = program(driver);
    let env = check_nodule(&parse(&src).unwrap_or_else(|e| panic!("{label}: parse failed: {e}")))
        .unwrap_or_else(|e| panic!("{label}: check failed: {e}"));
    let mono =
        monomorphize(&env, "main").unwrap_or_else(|e| panic!("{label}: monomorphize failed: {e}"));

    // Path 1: L1 fuel-guarded evaluator — must refuse, for the intended reason.
    let l1_err = Evaluator::new(&mono)
        .call("main", vec![])
        .expect_err(&format!(
            "{label}: L1-eval must refuse — a silent swap path (G2)"
        ));
    assert!(
        l1_err.to_string().contains(fragment),
        "{label}: L1-eval refused for an unexpected reason:\n  expected fragment: {fragment:?}\n  \
         actual: {l1_err}"
    );

    // Path 2: elaborate→L0 reference interpreter — must refuse, same reason.
    let node = elaborate(&env, "main").unwrap_or_else(|e| panic!("{label}: elaborate failed: {e}"));
    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(BinaryTernarySwapEngine),
    );
    let l0_err = interp.eval_core(&node).expect_err(&format!(
        "{label}: L0-interp must refuse — a silent swap path (G2)"
    ));
    assert!(
        l0_err.to_string().contains(fragment),
        "{label}: L0-interp refused for an unexpected reason:\n  expected fragment: {fragment:?}\n  \
         actual: {l0_err}"
    );

    // Path 3: AOT env-machine — must refuse, same reason.
    let prims = PrimRegistry::with_builtins();
    let aot_err = mycelium_mlir::run_core(&node, &prims, &BinaryTernarySwapEngine).expect_err(
        &format!("{label}: AOT run_core must refuse — a silent swap path (G2)"),
    );
    assert!(
        aot_err.to_string().contains(fragment),
        "{label}: AOT refused for an unexpected reason:\n  expected fragment: {fragment:?}\n  \
         actual: {aot_err}"
    );
}

// ══════════════════════════════════════════════════════════════════════════════════════════════
// Rust-oracle grounding (row 4) — the retained crate's own contracts hold, and the port covers
// its matrix one-to-one (a new oracle row fails HERE, never a silent port gap — G2).
// ══════════════════════════════════════════════════════════════════════════════════════════════

/// The oracle's own guarantee-matrix invariants hold (its `assert_matrix_invariants`), and the
/// test pairs (8,6)/(4,3) are legal while (8,4) is not — the corpus preconditions.
#[test]
fn oracle_invariants_and_legal_pairs() {
    assert_matrix_invariants();
    assert!(legal_pair(8, 6), "corpus pair (8,6) must be legal");
    assert!(legal_pair(4, 3), "corpus pair (4,3) must be legal");
    assert!(!legal_pair(8, 4), "reject-case pair (8,4) must be illegal");
}

/// Every oracle matrix row has exactly one ported `.myc` row constructor, in the same order.
/// Mutation witness: add/remove/reorder a `GUARANTEE_MATRIX` row → this fails before any silent
/// port gap can form.
#[test]
fn oracle_matrix_rows_match_port_rows_one_to_one() {
    let ported: Vec<&str> = ROWS.iter().map(|(_, op)| *op).collect();
    let oracle: Vec<&str> = GUARANTEE_MATRIX.iter().map(|r| r.op).collect();
    assert_eq!(
        oracle, ported,
        "the oracle matrix and the ported rows must correspond one-to-one, in order"
    );
}

// ══════════════════════════════════════════════════════════════════════════════════════════════
// Guarantee matrix — three-way cases driven against the LIVE oracle rows (std_diag.rs precedent:
// every expected value is computed from `GUARANTEE_MATRIX` at test time, not hardcoded).
// ══════════════════════════════════════════════════════════════════════════════════════════════

/// `matrix_len(matrix())` equals the live oracle's row count.
///
/// **W-1 length/count canon (2026-07-18):** `matrix_len` moved `Binary{8}` -> `Binary{64}`
/// (`docs/spec/swaps/binary-ternary.md` §A.4; `docs/spec/stdlib/swap.md` §B.3) — this test's
/// driver/expected widths track that change.
///
/// Guarantee: Declared (the transcription); Empirical (differential).
#[test]
fn matrix_len_matches_oracle_row_count() {
    let expected_count = u8::try_from(GUARANTEE_MATRIX.len()).expect("row count fits u8");
    let driver = "fn main() => Binary{64} = matrix_len(matrix());";
    let expected = format!(
        "nodule ref;\nfn main() => Binary{{64}} = {};",
        myc_len_chain(expected_count)
    );
    assert_three_way(
        "matrix_len == oracle GUARANTEE_MATRIX.len()",
        &program(driver),
        &expected,
    );
}

/// Every ported row's `guarantee`, `fallible`, `cert_carrying`, and `cert_kind` fields equal the
/// live oracle row's — field-for-field, all 7 rows (data-driven; VR-5: the tag is READ OFF the
/// oracle, never re-asserted here).
#[test]
fn every_row_field_matches_oracle() {
    for (ctor, op) in ROWS {
        let row = GUARANTEE_MATRIX
            .iter()
            .find(|r| r.op == *op)
            .unwrap_or_else(|| panic!("oracle row {op} exists (guarded by the one-to-one test)"));

        let driver = format!("fn main() => Guarantee = row_guarantee({ctor}());");
        let expected = format!(
            "nodule ref;\n{REF_TYPES}\nfn main() => Guarantee = {};",
            g_ctor(row.guarantee)
        );
        assert_three_way(
            &format!("{op}: guarantee tag"),
            &program(&driver),
            &expected,
        );

        let driver = format!("fn main() => Bool = row_fallible({ctor}());");
        let expected = format!(
            "nodule ref;\nfn main() => Bool = {};",
            myc_bool(row.fallible)
        );
        assert_three_way(&format!("{op}: fallible"), &program(&driver), &expected);

        let driver = format!("fn main() => Bool = row_cert_carrying({ctor}());");
        let expected = format!(
            "nodule ref;\nfn main() => Bool = {};",
            myc_bool(row.cert_carrying)
        );
        assert_three_way(
            &format!("{op}: cert_carrying"),
            &program(&driver),
            &expected,
        );

        let driver = format!("fn main() => CertKind = row_cert_kind({ctor}());");
        let expected = format!(
            "nodule ref;\n{REF_TYPES}\nfn main() => CertKind = {};",
            kind_ctor(row.cert_kind)
        );
        assert_three_way(&format!("{op}: cert_kind"), &program(&driver), &expected);
    }
}

/// Every ported row's `op` name byte-length and first byte match the live oracle's (full Bytes
/// equality is FLAGged — no `bytes_eq` prim; RFC-0032 D4 — so length + first byte is the honest
/// transcription check available today, the `std_diag.rs` precedent).
#[test]
fn every_row_op_name_matches_oracle_length_and_first_byte() {
    for (ctor, op) in ROWS {
        let row = GUARANTEE_MATRIX
            .iter()
            .find(|r| r.op == *op)
            .unwrap_or_else(|| panic!("oracle row {op} exists"));

        let driver = format!("fn main() => Binary{{32}} = bytes_len(row_op({ctor}()));");
        // Same-prim reference on a fresh literal of the ORACLE's own op string (Derived provenance,
        // oracle-supplied content).
        let expected = format!(
            "nodule ref;\nfn main() => Binary{{32}} = bytes_len(\"{}\");",
            row.op
        );
        assert_three_way(
            &format!("{op}: op byte-length"),
            &program(&driver),
            &expected,
        );

        let driver =
            format!("fn main() => Binary{{8}} = bytes_get(row_op({ctor}()), 0b0000_0000);");
        let expected = format!(
            "nodule ref;\nfn main() => Binary{{8}} = bytes_get(\"{}\", 0b0000_0000);",
            row.op
        );
        assert_three_way(
            &format!("{op}: op first byte"),
            &program(&driver),
            &expected,
        );
    }
}

/// The four ported invariant folds and their conjunction agree with the same invariants computed
/// over the LIVE oracle matrix (the `.myc` port of `assert_matrix_invariants`, value-domain form).
#[test]
fn matrix_invariants_match_oracle() {
    let ops_nonempty = GUARANTEE_MATRIX.iter().all(|r| !r.op.is_empty());
    let bij_exact = GUARANTEE_MATRIX
        .iter()
        .filter(|r| r.cert_kind == Some("Bijective"))
        .all(|r| r.guarantee == GuaranteeStrength::Exact);
    let bounded_not_exact = GUARANTEE_MATRIX
        .iter()
        .filter(|r| r.cert_kind == Some("Bounded"))
        .all(|r| r.guarantee != GuaranteeStrength::Exact);
    let nonfallible_none = GUARANTEE_MATRIX
        .iter()
        .filter(|r| !r.fallible)
        .all(|r| r.cert_kind.is_none());

    for (fn_name, oracle_value) in [
        ("all_ops_nonempty(matrix())", ops_nonempty),
        ("bijective_implies_exact(matrix())", bij_exact),
        ("bounded_never_exact(matrix())", bounded_not_exact),
        ("nonfallible_no_cert_kind(matrix())", nonfallible_none),
        (
            "matrix_invariants_hold()",
            ops_nonempty && bij_exact && bounded_not_exact && nonfallible_none,
        ),
    ] {
        let driver = format!("fn main() => Bool = {fn_name};");
        let expected = format!(
            "nodule ref;\nfn main() => Bool = {};",
            myc_bool(oracle_value)
        );
        assert_three_way(fn_name, &program(&driver), &expected);
    }
}

// ══════════════════════════════════════════════════════════════════════════════════════════════
// CheckError / Fallback — the never-silent check-error model (structural; hand-computed
// expectations — Declared, per the nodule doc; RFC-0002 §2).
// ══════════════════════════════════════════════════════════════════════════════════════════════

/// Arm dispatch and the explicit-fallback contract: `Refuted` is refuted (and carries no fallback);
/// `NotValidated` always carries the explicit `UseReference` route the caller must take — never a
/// silent pass.
#[test]
fn check_error_arms_and_explicit_fallback() {
    for (label, driver, expected_bool) in [
        (
            "refuted is refuted",
            "fn main() => Bool = is_refuted(refuted(\"counterexample\"));",
            true,
        ),
        (
            "not_validated is not refuted",
            "fn main() => Bool = is_refuted(not_validated(\"tv incompleteness\"));",
            false,
        ),
        (
            "not_validated is not_validated",
            "fn main() => Bool = is_not_validated(not_validated(\"tv incompleteness\"));",
            true,
        ),
        (
            "not_validated carries an explicit fallback",
            "fn main() => Bool = has_explicit_fallback(not_validated(\"tv incompleteness\"));",
            true,
        ),
        (
            "refuted carries no fallback route",
            "fn main() => Bool = has_explicit_fallback(refuted(\"counterexample\"));",
            false,
        ),
        (
            "the fallback is always UseReference",
            "fn main() => Bool = fallback_is_use_reference(not_validated(\"tv incompleteness\"));",
            true,
        ),
    ] {
        let expected = format!(
            "nodule ref;\nfn main() => Bool = {};",
            myc_bool(expected_bool)
        );
        assert_three_way(label, &program(driver), &expected);
    }
}

// ══════════════════════════════════════════════════════════════════════════════════════════════
// The bijective swap class — three-way + LIVE oracle payload differential (row 4).
// ══════════════════════════════════════════════════════════════════════════════════════════════

/// `bin8_to_tern6` over the corpus: the ported fn ≡ the direct native `swap` (three-way, matching
/// Derived provenance), AND its payload equals the Rust oracle's `bin_to_tern` output trits for
/// the same bit pattern — with a Bijective certificate on the oracle side.
#[test]
fn bin8_to_tern6_matches_oracle_over_corpus() {
    let policy = test_policy();
    for &v in BIN8_CORPUS {
        let lit = bin_lit(u64::from(v), 8);
        let driver = format!("fn main() => Ternary{{6}} = bin8_to_tern6({lit});");
        let expected = format!(
            "nodule ref;\nfn main() => Ternary{{6}} = swap({lit}, to: Ternary{{6}}, policy: rt);"
        );
        assert_three_way(
            &format!("bin8_to_tern6({lit}) three-way"),
            &program(&driver),
            &expected,
        );

        // Oracle side: the same 8-bit pattern, read as two's complement (the kernel codec).
        let signed = i64::from(v as i8);
        let swapped = bin_to_tern(&make_binary(signed, 8), 6, &policy)
            .unwrap_or_else(|e| panic!("oracle bin_to_tern({signed}) failed: {e}"));
        assert!(
            matches!(swapped.cert, SwapCertificate::Bijective { .. }),
            "oracle cert for {signed} must be Bijective"
        );
        let myc_payload = eval_payload(&format!("bin8_to_tern6({lit}) payload"), &driver);
        assert_eq!(
            &myc_payload,
            swapped.value.payload(),
            "payload divergence vs the Rust oracle for {lit} (= {signed})"
        );
    }
}

/// `tern6_to_bin8` over the in-range corpus: three-way + oracle payload agreement.
#[test]
fn tern6_to_bin8_matches_oracle_over_corpus() {
    let policy = test_policy();
    for &v in TERN6_CORPUS {
        let lit = tern_lit(v, 6);
        let driver = format!("fn main() => Binary{{8}} = tern6_to_bin8({lit});");
        let expected = format!(
            "nodule ref;\nfn main() => Binary{{8}} = swap({lit}, to: Binary{{8}}, policy: rt);"
        );
        assert_three_way(
            &format!("tern6_to_bin8({lit}) three-way"),
            &program(&driver),
            &expected,
        );

        let swapped = tern_to_bin(&make_ternary(v, 6), 8, &policy)
            .unwrap_or_else(|e| panic!("oracle tern_to_bin({v}) failed: {e}"));
        let myc_payload = eval_payload(&format!("tern6_to_bin8({lit}) payload"), &driver);
        assert_eq!(
            &myc_payload,
            swapped.value.payload(),
            "payload divergence vs the Rust oracle for {lit} (= {v})"
        );
    }
}

/// `roundtrip8` is the identity on the corpus (the `LosslessWithinRange` property, RFC-0002 §4):
/// three-way against the direct nested-swap composition, payload identity with the input bits,
/// and the Rust oracle round-trips the same values to the same payloads.
#[test]
fn roundtrip8_is_identity_and_matches_oracle() {
    let policy = test_policy();
    for &v in BIN8_CORPUS {
        let lit = bin_lit(u64::from(v), 8);
        let driver = format!("fn main() => Binary{{8}} = roundtrip8({lit});");
        let expected = format!(
            "nodule ref;\nfn main() => Binary{{8}} = \
             swap(swap({lit}, to: Ternary{{6}}, policy: rt), to: Binary{{8}}, policy: rt);"
        );
        assert_three_way(
            &format!("roundtrip8({lit}) three-way"),
            &program(&driver),
            &expected,
        );

        let signed = i64::from(v as i8);
        let src = make_binary(signed, 8);
        let enc = bin_to_tern(&src, 6, &policy)
            .unwrap_or_else(|e| panic!("oracle enc({signed}) failed: {e}"));
        let dec = tern_to_bin(&enc.value, 8, &policy)
            .unwrap_or_else(|e| panic!("oracle dec(enc({signed})) failed: {e}"));
        assert_eq!(
            dec.value.payload(),
            src.payload(),
            "oracle round-trip must be the identity for {signed}"
        );
        let myc_payload = eval_payload(&format!("roundtrip8({lit}) payload"), &driver);
        assert_eq!(
            &myc_payload,
            src.payload(),
            "ported round-trip must return the original bits for {lit}"
        );
    }
}

/// `roundtrip4` over the FULL 16-value `Binary{4}` corpus (the small-width exhaustive form of the
/// crate's own full-n8-corpus property test), with per-value oracle payload agreement on the
/// encode leg.
#[test]
fn roundtrip4_full_corpus_identity_and_oracle_agreement() {
    let policy = test_policy();
    for v in 0u64..16 {
        let lit = bin_lit(v, 4);
        #[allow(clippy::cast_possible_wrap)]
        let signed = if v >= 8 { v as i64 - 16 } else { v as i64 };

        let driver = format!("fn main() => Binary{{4}} = roundtrip4({lit});");
        let myc_payload = eval_payload(&format!("roundtrip4({lit})"), &driver);
        let src = make_binary(signed, 4);
        assert_eq!(
            &myc_payload,
            src.payload(),
            "roundtrip4 must return the original bits for {lit} (= {signed})"
        );

        let driver = format!("fn main() => Ternary{{3}} = bin4_to_tern3({lit});");
        let myc_enc = eval_payload(&format!("bin4_to_tern3({lit})"), &driver);
        let oracle_enc = bin_to_tern(&src, 3, &policy)
            .unwrap_or_else(|e| panic!("oracle bin_to_tern({signed}, 3) failed: {e}"));
        assert_eq!(
            &myc_enc,
            oracle_enc.value.payload(),
            "encode payload divergence vs the Rust oracle for {lit} (= {signed})"
        );
    }
}

/// **W-1 canonical width (2026-07-18): `Binary{64} <-> Ternary{41}`** — the E-W1/M-1119 enablement
/// item unblocked this pair for the conversion-utility fast path
/// (`docs/spec/swaps/binary-ternary.md` §A.3/§A.5; `docs/spec/stdlib/swap.md` §B.2/§B.4). A curated
/// edge corpus (`i64::MIN`/`MAX` and values whose magnitude needs the 41st trit — i.e. exceeds
/// `max_magnitude(40)` but not `max_magnitude(41)`, so 40 trits alone could not represent them,
/// exactly the values M-1119 exists for) — NOT exhaustive (`2^64` is infeasible; mirrors the
/// curated-corpus style of `TERN6_CORPUS` above, scaled to the new canonical width). The exhaustive
/// kernel-level witness lives in
/// `mycelium-core::tests::ternary::m41_round_trip_covers_the_full_binary64_range`; this test's own
/// job is confirming the `.myc` `swap(…, to: …, policy: …)` surface — not just the underlying Rust
/// kernel fns — round-trips correctly at the new width, end to end (L1-eval ≡ L0-interp ≡ AOT, plus
/// the live oracle).
#[test]
fn w1_bin64_tern41_roundtrip_over_curated_corpus() {
    let policy = test_policy();
    let corpus: &[i64] = &[
        i64::MIN,
        i64::MIN + 1,
        -1,
        0,
        1,
        i64::MAX - 1,
        i64::MAX,
        7_000_000_000_000_000_000,
        -7_000_000_000_000_000_000,
    ];
    for &signed in corpus {
        let bits = signed as u64; // two's-complement bit pattern, reinterpreted unsigned
        let lit = bin_lit(bits, 64);

        // roundtrip64 is the identity — three-way + oracle agreement.
        let driver = format!("fn main() => Binary{{64}} = roundtrip64({lit});");
        let expected = format!(
            "nodule ref;\nfn main() => Binary{{64}} = \
             swap(swap({lit}, to: Ternary{{41}}, policy: rt), to: Binary{{64}}, policy: rt);"
        );
        assert_three_way(
            &format!("roundtrip64({lit}) three-way"),
            &program(&driver),
            &expected,
        );

        let src = make_binary(signed, 64);
        let enc = bin_to_tern(&src, 41, &policy)
            .unwrap_or_else(|e| panic!("oracle enc({signed}) failed: {e}"));
        assert!(
            matches!(enc.cert, SwapCertificate::Bijective { .. }),
            "oracle cert for {signed} must be Bijective"
        );
        let dec = tern_to_bin(&enc.value, 64, &policy)
            .unwrap_or_else(|e| panic!("oracle dec(enc({signed})) failed: {e}"));
        assert_eq!(
            dec.value.payload(),
            src.payload(),
            "oracle round-trip must be the identity for {signed}"
        );
        let myc_payload = eval_payload(&format!("roundtrip64({lit}) payload"), &driver);
        assert_eq!(
            &myc_payload,
            src.payload(),
            "ported round-trip must return the original bits for {lit} (= {signed})"
        );

        // bin64_to_tern41 payload matches the oracle encode directly.
        let driver = format!("fn main() => Ternary{{41}} = bin64_to_tern41({lit});");
        let myc_enc = eval_payload(&format!("bin64_to_tern41({lit})"), &driver);
        assert_eq!(
            &myc_enc,
            enc.value.payload(),
            "encode payload divergence vs the Rust oracle for {lit} (= {signed})"
        );
    }
}

// ══════════════════════════════════════════════════════════════════════════════════════════════
// Never-silent reject-case conformance (the M-929 DoD extra — G2): no silent swap path survives
// translation, at ANY layer — parse, check, or any of the three execution paths — and the Rust
// oracle refuses the same instances.
// ══════════════════════════════════════════════════════════════════════════════════════════════

/// A `swap` missing its mandatory `policy:` is a PARSE error (S1/WF2) — the never-silent swap is
/// grammar, not convention (the `02-swap-missing-policy.myc` conformance fixture's fragment).
#[test]
fn reject_swap_missing_policy_is_a_parse_error() {
    let driver = "fn main() => Ternary{6} = swap(0b1011_0010, to: Ternary{6});";
    let err = parse(&program(driver)).expect_err("a policy-less swap must not parse (S1/WF2)");
    assert!(
        err.to_string().contains("a swap is never silent"),
        "rejected for an unexpected reason: {err}"
    );
}

/// A `swap` missing its mandatory `to:` target is a PARSE error (S1/WF2).
#[test]
fn reject_swap_missing_target_is_a_parse_error() {
    let driver = "fn main() => Ternary{6} = swap(0b1011_0010, policy: rt);";
    let err = parse(&program(driver)).expect_err("a target-less swap must not parse (S1/WF2)");
    assert!(
        err.to_string().contains("the `to:` target label"),
        "rejected for an unexpected reason: {err}"
    );
}

/// An implicit cross-paradigm edge — a `Binary{8}` flowing into a `Ternary{6}` position with no
/// `swap` written — is an explicit `MissingConversion` CHECK refusal pointing at writing a `swap`;
/// the checker never inserts one (RFC-0012 §4.4; S1).
#[test]
fn reject_implicit_cross_paradigm_edge_is_a_check_error() {
    let driver = "fn silent_edge(x: Binary{8}) => Ternary{6} = x;\n\
                  fn main() => Ternary{6} = silent_edge(0b1011_0010);";
    let src = program(driver);
    let ast = parse(&src).expect("the program parses — the refusal is the checker's");
    let err = check_nodule(&ast)
        .expect_err("an implicit cross-paradigm edge must not check — a silent swap path (G2)");
    assert!(
        err.to_string().contains("never silently converted"),
        "rejected for an unexpected reason: {err}"
    );
}

/// An illegal pair — `Binary{8}` into `Ternary{4}` (2^7 > (3^4−1)/2; RFC-0002 §5) — is an explicit
/// runtime refusal from ALL THREE execution paths, and the Rust oracle refuses the same instance
/// with `IllegalPair` — never a clamp or a truncation on either side.
#[test]
fn reject_illegal_pair_on_all_paths_and_oracle() {
    let driver = "fn main() => Ternary{4} = swap(0b1011_0010, to: Ternary{4}, policy: rt);";
    assert_all_paths_refuse("illegal pair (8,4)", driver, "illegal pair");

    let policy = test_policy();
    let result = bin_to_tern(&make_binary(-78, 8), 4, &policy);
    assert!(
        matches!(result, Err(SwapError::IllegalPair { width: 8, trits: 4 })),
        "oracle must refuse the same instance with IllegalPair{{8,4}}, got {result:?}"
    );
}

/// An out-of-range decode — ternary 10 into `Binary{4}` (max signed 7), THROUGH the ported
/// `tern3_to_bin4` — is an explicit runtime refusal from ALL THREE execution paths, and the Rust
/// oracle refuses the same instance with `OutOfRange` — never a wrap or a sentinel (C1).
#[test]
fn reject_out_of_range_decode_on_all_paths_and_oracle() {
    let lit = tern_lit(10, 3);
    let driver = format!("fn main() => Binary{{4}} = tern3_to_bin4({lit});");
    assert_all_paths_refuse(
        "out-of-range decode (10 -> Binary{4})",
        &driver,
        "outside the target binary range",
    );

    let policy = test_policy();
    let result = tern_to_bin(&make_ternary(10, 3), 4, &policy);
    assert!(
        matches!(result, Err(SwapError::OutOfRange)),
        "oracle must refuse the same instance with OutOfRange, got {result:?}"
    );
}
