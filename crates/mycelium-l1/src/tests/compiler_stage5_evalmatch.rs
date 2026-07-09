//! M-740 Stage 5 (M-1013 eval PR-1; DN-26 §7.3 row 5 / §9 flag-1) — the self-hosted
//! `compiler.semcore` port of `eval.rs`'s FIRST fragment: the Data/Ctor/Ident/Tuple/Wildcard
//! fraction of `Evaluator::try_match` (eval.rs 2005-2079), ported as `lval_try_match` in
//! `lib/compiler/semcore.myc`.
//!
//! **Live-oracle posture (VR-5).** Every non-`Lit` case calls the REAL Rust
//! `Evaluator::try_match` (widened to `pub(crate)` this leaf, zero logic change — see eval.rs's
//! doc comment on the fn) on a hand-built `Env`/`L1Value` fixture, producing a genuine
//! `(bool, Vec<(String, L1Value)>)`. It then evaluates the `.myc` port's `lval_try_match` mirror
//! driver and DECODES the returned `L1Value` back into the same shape (`decode_match_result`,
//! built on `marshal_support`'s shared primitives plus this file's own `LVal`/`Bool` decoders).
//! The two independently-produced values are compared with Rust's own trusted derived `==`.
//!
//! **`Pattern::Lit` is OUT OF SCOPE (FLAG-semcore-35 in `semcore.myc`) — no case here claims
//! oracle parity for it.** The port's `PLit` arm is a blanket refusal (comparing a literal pattern
//! against a representation value needs the trusted kernel `Value`'s equality, not modeled here);
//! `lval_try_match_lit_refuses` below is a STANDALONE probe (no oracle comparison) that the port's
//! refusal is explicit (`Err`), never a panic or a silently-wrong `Ok` (G2).
//!
//! M-981 applies: only the L1-eval leg is exercised (small synthetic fixtures, not a corpus
//! program); `try_match` needs no `monomorphize` on the ORACLE side (it is not part of the CEK
//! machine), but the `.myc` DRIVER side still runs through the established `check` →
//! `monomorphize("main")` → `Evaluator::call` marshalling pipeline every other Stage-5 differential
//! uses (`marshal_support::assert_l1_marshal`'s own method), since the self-hosted evaluator only
//! runs a monomorphized `Env`.

use std::sync::Arc;

use crate::ast::{Literal, Pattern};
use crate::checkty::{check_nodule, CtorInfo, DataInfo};
use crate::eval::{Evaluator, L1Value};
use crate::mono::monomorphize;
use crate::parse;
use crate::tests::marshal_support::*;

// ── local `TestVal` — the shared fixture/decoder shape for this file's narrowed `LVal` mirror ───
// (`Data(ty, ctor, fields)` mirrors `LData`/`L1Value::Data` exactly; `Opaque` stands in for
// `L1Value::{Repr, Substrate, Fn}` collapsed per `semcore.myc`'s FLAG-semcore-35/36 — every arm
// this increment ports treats the three identically, so one placeholder suffices; see those FLAGs.)
#[derive(Debug, Clone, PartialEq)]
enum TestVal {
    Data(String, String, Vec<TestVal>),
    Opaque,
}

/// `TestVal` → a REAL `eval::L1Value` fixture for the oracle call. `Opaque` becomes
/// `L1Value::Fn("opaque")` — the cheapest real `L1Value` variant to construct (no kernel `Value`,
/// no live `Substrate` handle) that is, for every arm `try_match` implements besides `Lit`,
/// observationally identical to `Repr`/`Substrate` (see the FLAG-semcore-35 argument this mirrors).
fn testval_to_l1value(v: &TestVal) -> L1Value {
    match v {
        TestVal::Data(ty, ctor, fields) => L1Value::Data {
            ty: ty.clone(),
            ctor: ctor.clone(),
            fields: Arc::new(fields.iter().map(testval_to_l1value).collect()),
        },
        TestVal::Opaque => L1Value::Fn("opaque".to_owned()),
    }
}

/// The inverse direction, over the REAL oracle's output — used to compare the oracle's
/// `Vec<(String, L1Value)>` binds against the port's decoded `Vec<(String, TestVal)>` binds on
/// common ground. Never reached on a `Repr`/`Substrate` value in this file's corpus (only `Data`
/// and the `Fn("opaque")` placeholder ever appear — the differential never constructs a real
/// `Repr`/`Substrate`, per the `Lit`-arm narrowing), so those two arms are a documented, unreached
/// `panic!` rather than a silent misdecode (G2).
fn l1value_to_testval(v: &L1Value) -> TestVal {
    match v {
        L1Value::Data { ty, ctor, fields } => TestVal::Data(
            ty.clone(),
            ctor.clone(),
            fields.iter().map(l1value_to_testval).collect(),
        ),
        L1Value::Fn(_) => TestVal::Opaque,
        L1Value::Repr(_) | L1Value::Substrate(_) => panic!(
            "l1value_to_testval: a Repr/Substrate value reached the oracle-side decoder — this \
             file's corpus never constructs one (the Lit-arm narrowing keeps them out of scope)"
        ),
    }
}

// ── `.myc` LVal encoder (fixture INPUT side) ──────────────────────────────────────────────────────

fn encode_lval_list(vs: &[TestVal]) -> String {
    let mut s = String::from("Nil");
    for v in vs.iter().rev() {
        s = format!("Cons({}, {})", encode_lval(v), s);
    }
    s
}

fn encode_lval(v: &TestVal) -> String {
    match v {
        TestVal::Data(ty, ctor, fields) => format!(
            "LData({}, {}, {})",
            encode_bytes(ty),
            encode_bytes(ctor),
            encode_lval_list(fields)
        ),
        TestVal::Opaque => "LOpaque".to_owned(),
    }
}

// ── `.myc` LVal / Bool / Pair / Result decoders (output side) ────────────────────────────────────

fn decode_lval(v: &L1Value) -> TestVal {
    let (ctor, fields) = expect_data(v, "LVal");
    match ctor {
        "LData" => TestVal::Data(
            decode_string(&fields[0]),
            decode_string(&fields[1]),
            decode_vec(&fields[2], decode_lval),
        ),
        "LOpaque" => TestVal::Opaque,
        c => panic!("marshal decode_lval: unexpected ctor {c}"),
    }
}

fn decode_bool(v: &L1Value) -> bool {
    match expect_data(v, "Bool").0 {
        "True" => true,
        "False" => false,
        c => panic!("marshal decode_bool: unexpected ctor {c}"),
    }
}

fn decode_bind_pair(v: &L1Value) -> (String, TestVal) {
    let (ctor, fields) = expect_data(v, "Pair");
    match ctor {
        "Pr" => (decode_string(&fields[0]), decode_lval(&fields[1])),
        c => panic!("marshal decode_bind_pair: unexpected ctor {c}"),
    }
}

/// The full `Result[Pair[Bool, Vec[Pair[Bytes, LVal]]], Bytes]` → `Result<(bool, binds), ()>`.
fn decode_match_result(v: &L1Value) -> Result<(bool, Vec<(String, TestVal)>), ()> {
    decode_result(v, |ok_v| {
        let (ctor, fields) = expect_data(ok_v, "Pair");
        match ctor {
            "Pr" => (
                decode_bool(&fields[0]),
                decode_vec(&fields[1], decode_bind_pair),
            ),
            c => panic!("marshal decode_match_result: unexpected ctor {c}"),
        }
    })
}

// ── `.myc` DataInfo/CtorInfo/Pattern/Literal encoders (module-local — mirrors
// compiler_stage5_register.rs's private `encode_data_info`/`encode_ctor_info`, not shared) ──────────

fn encode_ctor_info(ci: &CtorInfo) -> String {
    format!(
        "CI({}, {})",
        encode_bytes(&ci.name),
        encode_ty_list(&ci.fields)
    )
}

fn encode_ctor_info_list(cis: &[CtorInfo]) -> String {
    let mut s = String::from("Nil");
    for ci in cis.iter().rev() {
        s = format!("Cons({}, {})", encode_ctor_info(ci), s);
    }
    s
}

fn encode_data_info(d: &DataInfo) -> String {
    format!(
        "DI({}, {}, {})",
        encode_bytes(&d.name),
        encode_names(&d.params),
        encode_ctor_info_list(&d.ctors)
    )
}

fn encode_data_info_list(ds: &[DataInfo]) -> String {
    let mut s = String::from("Nil");
    for d in ds.iter().rev() {
        s = format!("Cons({}, {})", encode_data_info(d), s);
    }
    s
}

fn encode_names(ns: &[String]) -> String {
    let mut s = String::from("Nil");
    for n in ns.iter().rev() {
        s = format!("Cons({}, {})", encode_bytes(n), s);
    }
    s
}

fn encode_pattern_list(ps: &[Pattern]) -> String {
    let mut s = String::from("Nil");
    for p in ps.iter().rev() {
        s = format!("Cons({}, {})", encode_pattern(p), s);
    }
    s
}

fn encode_pattern(p: &Pattern) -> String {
    match p {
        Pattern::Wildcard => "PWildcard".to_owned(),
        Pattern::Lit(l) => format!("PLit({})", encode_literal(l)),
        Pattern::Ctor(n, subs) => {
            format!("PCtor({}, {})", encode_bytes(n), encode_pattern_list(subs))
        }
        Pattern::Ident(n) => format!("PIdent({})", encode_bytes(n)),
        Pattern::Tuple(subs) => format!("PTuple({})", encode_pattern_list(subs)),
        Pattern::Or(subs) => format!("POr({})", encode_pattern_list(subs)),
    }
}

/// A minimal `Literal` encoder — only `Bytes` is needed (the standalone `Lit`-refuses probe).
fn encode_literal(l: &Literal) -> String {
    match l {
        Literal::Bytes(hex) => format!("LBytes({})", encode_bytes(hex)),
        other => panic!("encode_literal: {other:?} not needed by this file's fixtures"),
    }
}

// ── fixture registry: one shared `Shape` type (SPoint | SCircle(Binary{32}) | SSquare(_, _)) ─────

const FIXTURE_SRC: &str = "nodule test.evalmatch_fixture;\n\
     type Shape = SPoint | SCircle(Binary{32}) | SSquare(Binary{32}, Binary{32});\n\
     fn shape_probe() => Binary{1} = 0b1;\n";

/// The REAL checked `Env` (for the oracle) + the extracted `Shape` `DataInfo` (for both sides —
/// the `.myc` `types` argument is `encode_data_info_list`'d from the SAME `DataInfo` the oracle's
/// `env.types` holds, so a marshalling bug can never hide behind a hand-typed mismatch).
fn fixture_env_and_shape() -> (crate::checkty::Env, DataInfo) {
    let env = check_nodule(&parse(FIXTURE_SRC).unwrap_or_else(|e| panic!("fixture parse: {e}")))
        .unwrap_or_else(|e| panic!("fixture check: {e}"));
    let shape = env
        .types
        .get("Shape")
        .cloned()
        .unwrap_or_else(|| panic!("fixture: `Shape` missing from env.types"));
    (env, shape)
}

fn opaque(n: &str) -> TestVal {
    let _ = n; // the placeholder carries no identity this fragment inspects (FLAG-semcore-35/36)
    TestVal::Opaque
}

fn data(ty: &str, ctor: &str, fields: Vec<TestVal>) -> TestVal {
    TestVal::Data(ty.to_owned(), ctor.to_owned(), fields)
}

// ── the marshalling runner: real oracle `try_match` vs. the `.myc` port ──────────────────────────

fn assert_try_match(label: &str, types: &[DataInfo], pat: &Pattern, val: &TestVal) {
    let (env, _) = fixture_env_and_shape();
    let l1_val = testval_to_l1value(val);
    let mut binds = Vec::new();
    let want: Result<(bool, Vec<(String, TestVal)>), ()> = Evaluator::new(&env)
        .try_match("evalmatch-test", pat, &l1_val, &mut binds)
        .map(|matched| {
            (
                matched,
                binds
                    .iter()
                    .map(|(n, v)| (n.clone(), l1value_to_testval(v)))
                    .collect(),
            )
        })
        .map_err(|_| ());

    let driver = format!(
        "fn main() => Result[Pair[Bool, Vec[Pair[Bytes, LVal]]], Bytes] = \
         lval_try_match({}, {}, {}, Nil);\n",
        encode_data_info_list(types),
        encode_pattern(pat),
        encode_lval(val)
    );
    let src = program(&driver);
    let check_env =
        check_nodule(&parse(&src).unwrap_or_else(|e| panic!("{label}: parse failed: {e}")))
            .unwrap_or_else(|e| panic!("{label}: check failed: {e}"));
    let mono = monomorphize(&check_env, "main")
        .unwrap_or_else(|e| panic!("{label}: monomorphize failed: {e}"));
    let l1_result = Evaluator::new(&mono)
        .call("main", vec![])
        .unwrap_or_else(|e| panic!("{label}: L1-eval failed: {e}"));
    let got = decode_match_result(&l1_result);

    assert_eq!(
        got, want,
        "{label}: decoded marshal {got:?} does not match oracle {want:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// Structural gate: `semcore.myc` (with the eval PR-1 additions) parses and type-checks green.
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn semcore_evalmatch_parses_and_checks() {
    let nodule = parse(SEMCORE_SRC).unwrap_or_else(|e| panic!("semcore.myc: parse failed: {e}"));
    check_nodule(&nodule).unwrap_or_else(|e| panic!("semcore.myc: check failed: {e}"));
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// Wildcard: always matches, binds nothing.
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn try_match_wildcard() {
    let (_, shape) = fixture_env_and_shape();
    let cases: Vec<TestVal> = vec![
        data("Shape", "SPoint", vec![]),
        data("Shape", "SCircle", vec![opaque("r")]),
        opaque("fn-value"),
    ];
    for v in cases {
        assert_try_match(
            &format!("wildcard vs {v:?}"),
            std::slice::from_ref(&shape),
            &Pattern::Wildcard,
            &v,
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// Ident: a bare name that IS one of the value's data type's ctor names checks equality; otherwise
// it binds the whole value (both the "matches a ctor name" and "shadows/is a plain binder" legs).
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn try_match_ident_as_nullary_ctor() {
    let (_, shape) = fixture_env_and_shape();
    // "SPoint" IS a ctor of Shape and the value's own ctor -- matches, no bind.
    assert_try_match(
        "Ident(SPoint) vs SPoint (same nullary ctor)",
        std::slice::from_ref(&shape),
        &Pattern::Ident("SPoint".to_owned()),
        &data("Shape", "SPoint", vec![]),
    );
    // "SPoint" IS a ctor of Shape, but the value's ctor is "SCircle" -- refuses (false), no bind.
    assert_try_match(
        "Ident(SPoint) vs SCircle (different nullary alt)",
        std::slice::from_ref(&shape),
        &Pattern::Ident("SPoint".to_owned()),
        &data("Shape", "SCircle", vec![opaque("r")]),
    );
}

#[test]
fn try_match_ident_as_binder() {
    let (_, shape) = fixture_env_and_shape();
    // "x" is NOT a ctor of Shape -- binds the whole value regardless of its shape.
    let cases: Vec<TestVal> = vec![
        data("Shape", "SPoint", vec![]),
        data("Shape", "SCircle", vec![opaque("r")]),
        opaque("fn-value"),
    ];
    for v in cases {
        assert_try_match(
            &format!("Ident(x) vs {v:?} (plain binder)"),
            std::slice::from_ref(&shape),
            &Pattern::Ident("x".to_owned()),
            &v,
        );
    }
    // An empty `types` registry: the Ident guard's lookup misses entirely -- also a plain binder
    // (mirrors `self.env.types.get(ty)` returning `None`).
    assert_try_match(
        "Ident(x) vs SPoint with an EMPTY types registry",
        &[],
        &Pattern::Ident("x".to_owned()),
        &data("Shape", "SPoint", vec![]),
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// Ctor: matches on ctor-name equality, then recurses into sub-patterns/fields pairwise (zip
// semantics); a name mismatch or a non-`Data` value short-circuits `false`.
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn try_match_ctor_cases() {
    let (_, shape) = fixture_env_and_shape();
    let cases: Vec<(&str, Pattern, TestVal)> = vec![
        (
            "SCircle(_) vs SCircle(r) -- matches, wildcard sub",
            Pattern::Ctor("SCircle".to_owned(), vec![Pattern::Wildcard]),
            data("Shape", "SCircle", vec![opaque("r")]),
        ),
        (
            "SCircle(w) vs SCircle(r) -- matches, binds w",
            Pattern::Ctor("SCircle".to_owned(), vec![Pattern::Ident("w".to_owned())]),
            data("Shape", "SCircle", vec![opaque("r")]),
        ),
        (
            "SSquare(w, h) vs SSquare(a, b) -- matches, binds BOTH in order",
            Pattern::Ctor(
                "SSquare".to_owned(),
                vec![Pattern::Ident("w".to_owned()), Pattern::Ident("h".to_owned())],
            ),
            data("Shape", "SSquare", vec![opaque("a"), opaque("b")]),
        ),
        (
            "SCircle(_) vs SSquare(..) -- ctor name mismatch, false",
            Pattern::Ctor("SCircle".to_owned(), vec![Pattern::Wildcard]),
            data("Shape", "SSquare", vec![opaque("a"), opaque("b")]),
        ),
        (
            "SPoint vs the opaque placeholder -- non-Data value, false",
            Pattern::Ctor("SPoint".to_owned(), vec![]),
            opaque("fn-value"),
        ),
        (
            "nested: SSquare(SCircle(_), y) vs SSquare(SCircle(r), b) -- deep recursion + partial binds",
            Pattern::Ctor(
                "SSquare".to_owned(),
                vec![
                    Pattern::Ctor("SCircle".to_owned(), vec![Pattern::Wildcard]),
                    Pattern::Ident("y".to_owned()),
                ],
            ),
            data(
                "Shape",
                "SSquare",
                vec![data("Shape", "SCircle", vec![opaque("r")]), opaque("b")],
            ),
        ),
        (
            "nested mismatch: SSquare(SPoint, y) vs SSquare(SCircle(r), b) -- inner sub fails first",
            Pattern::Ctor(
                "SSquare".to_owned(),
                vec![
                    Pattern::Ctor("SPoint".to_owned(), vec![]),
                    Pattern::Ident("y".to_owned()),
                ],
            ),
            data(
                "Shape",
                "SSquare",
                vec![data("Shape", "SCircle", vec![opaque("r")]), opaque("b")],
            ),
        ),
    ];
    for (label, pat, val) in cases {
        assert_try_match(label, std::slice::from_ref(&shape), &pat, &val);
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// Tuple: desugars to `Ctor(tuple_ctor_name(n), subs)` -- exercised against a value already shaped
// as the synthetic tuple ctor (no `Shape` registration needed; the `Ctor` arm never consults
// `types` for a name-equality check).
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn try_match_tuple_desugars() {
    let matching = data("Tuple$2", "MkTuple$2", vec![opaque("a"), opaque("b")]);
    let mismatched = data("Shape", "SPoint", vec![]);
    assert_try_match(
        "Tuple(x, y) vs MkTuple$2(a, b) -- desugars + binds both",
        &[],
        &Pattern::Tuple(vec![
            Pattern::Ident("x".to_owned()),
            Pattern::Ident("y".to_owned()),
        ]),
        &matching,
    );
    assert_try_match(
        "Tuple(x, y) vs SPoint -- desugared Ctor name mismatch, false",
        &[],
        &Pattern::Tuple(vec![
            Pattern::Ident("x".to_owned()),
            Pattern::Ident("y".to_owned()),
        ]),
        &mismatched,
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// Or: an internal invariant violation if it ever reaches the evaluator (must be desugared by the
// checker first) -- both the oracle and the port refuse (collapsed to `Err(())` by
// `decode_result`, so this DOES compare against the oracle, unlike the `Lit` narrowing below).
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn try_match_or_refuses() {
    let (_, shape) = fixture_env_and_shape();
    assert_try_match(
        "Or([SPoint, SCircle(_)]) vs SPoint -- internal-invariant refusal",
        &[shape],
        &Pattern::Or(vec![
            Pattern::Ident("SPoint".to_owned()),
            Pattern::Ctor("SCircle".to_owned(), vec![Pattern::Wildcard]),
        ]),
        &data("Shape", "SPoint", vec![]),
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// Lit: OUT OF SCOPE (FLAG-semcore-35) -- a standalone "refuses cleanly" probe, NOT an
// oracle-parity case (the real oracle's `Lit`-vs-`Data`/`Substrate`/`Fn` legs are `Ok(false)`,
// which the port's blanket refusal does not reproduce -- documented, never silently claimed).
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn lval_try_match_lit_refuses() {
    let driver = "fn main() => Result[Pair[Bool, Vec[Pair[Bytes, LVal]]], Bytes] = \
                   lval_try_match(Nil, PLit(LBytes(\"ab\")), LOpaque, Nil);\n";
    let src = program(driver);
    let env = check_nodule(&parse(&src).unwrap_or_else(|e| panic!("parse: {e}")))
        .unwrap_or_else(|e| panic!("check: {e}"));
    let mono = monomorphize(&env, "main").unwrap_or_else(|e| panic!("mono: {e}"));
    let l1_val = Evaluator::new(&mono)
        .call("main", vec![])
        .unwrap_or_else(|e| panic!("eval: {e}"));
    let got = decode_match_result(&l1_val);
    assert_eq!(
        got,
        Err(()),
        "lval_try_match's Lit arm must refuse explicitly (Err), never silently succeed -- got {got:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// Non-vacuity probe: a `.myc` literal whose SHAPE differs from the oracle's must NOT decode equal —
// proves the decoder actually reads the ctor name it claims to (the established non-vacuity twin).
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn evalmatch_marshal_discriminates() {
    let (_, shape) = fixture_env_and_shape();
    let want_ok = (true, vec![("x".to_owned(), TestVal::Opaque)]);
    // A DIFFERENT bind name ("y" instead of "x") must not decode equal.
    let wrong_driver = format!(
        "fn main() => Result[Pair[Bool, Vec[Pair[Bytes, LVal]]], Bytes] = \
         lval_try_match({}, PIdent(\"y\"), LData(\"Shape\", \"SPoint\", Nil), Nil);\n",
        encode_data_info_list(&[shape])
    );
    let src = program(&wrong_driver);
    let env = check_nodule(&parse(&src).unwrap_or_else(|e| panic!("parse: {e}")))
        .unwrap_or_else(|e| panic!("check: {e}"));
    let mono = monomorphize(&env, "main").unwrap_or_else(|e| panic!("mono: {e}"));
    let l1_val = Evaluator::new(&mono)
        .call("main", vec![])
        .unwrap_or_else(|e| panic!("eval: {e}"));
    let got = decode_match_result(&l1_val);
    assert_ne!(
        got,
        Ok(want_ok),
        "evalmatch_marshal_discriminates: a bind name of \"y\" decoded equal to the \"x\" oracle \
         value -- the decoder is not reading the bind name"
    );
}
