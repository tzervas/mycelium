//! M-740 Stage 5, increment 7 (M-1012; DN-26 §7.3 row 5 / §10) — the self-hosted `compiler.semcore`
//! port of elab.rs's PURE L0 lowering helpers: the LIVE-ORACLE differential gate for the frontend →
//! kernel L0 seam under **DN-26 §10 Option A** (the in-language mirror model).
//!
//! Helpers ported into `lib/compiler/semcore.myc` and gated here:
//!   * `scalar_kind` / `sparsity_class` (elab.rs) — the boundary-independent enum maps (land first).
//!   * `type_repr` (elab.rs) — surface `TypeRef` → kernel `Repr`.
//!   * `lit_value` (elab.rs) — a representation literal's L0 `Value` (Bin/Trit/Str + refusals;
//!     LBytes/LFloat DEFERRED — `.myc` FLAG-semcore-25, asserted to refuse never-silently below).
//!   * `field_spec` / `ty_to_repr` / `ty_to_field_ty_ref` (elab.rs) — checked `Ty` → build-time specs.
//!   * `policy_name_preimage` (elab.rs, extracted this wave) — the wild-free preimage of
//!     `policy_name_ref`; the BLAKE3 hashing step is DEFERRED (`.myc` FLAG-semcore-27).
//!
//! **Live-oracle posture (VR-5).** Every case calls the REAL Rust `elab::*` on a fixture; the oracle's
//! kernel `Value`/`Repr`/`FieldSpec` is encoded as a `.myc` MIRROR literal (Option A §2.2 — the
//! "harness-side marshalling" witness) and compared against the `.myc` port's independently-built
//! mirror by the `.myc` structural equalities (`value_eq`/`repr_eq`/`field_spec_eq`/… — FLAG-semcore-28,
//! the `ty_eq` posture). A mis-lowering diverges the two mirrors and fails the case. The two productions
//! are genuinely independent (the port never calls the kernel; the oracle never calls the port).
//!
//! M-981 applies: only the L1-eval leg is exercised (small synthetic fixtures, not a corpus program).
//! `scalar_kind`/`sparsity_class` are covered exhaustively (they twin the increment-4 tags).

use crate::ast::{BaseType, Literal, Path, Scalar, Sparsity, TypeRef, WidthRef};
use crate::checkty::{check_nodule, Ty, Width};
use crate::elab::{
    build_registry, field_spec, lit_value, policy_name_preimage, scalar_kind, sparsity_class,
    ty_to_field_ty_ref, ty_to_repr, type_repr,
};
use crate::eval::Evaluator;
use crate::mono::monomorphize;
use crate::parse;
use mycelium_core::{
    FieldSpec, FieldTyRef, FloatWidth, FnSig, Meta, Payload, Provenance, Repr, ScalarKind,
    SparsityClass, Trit, Value,
};

const SEMCORE_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../lib/compiler/semcore.myc"
));

fn program(driver: &str) -> String {
    format!("{SEMCORE_SRC}\n{driver}")
}

/// Extract a `Binary{N}` `CoreValue`'s bits as a `u32` (MSB-first) — the established convention.
fn core_bits_as_u32(v: &mycelium_core::CoreValue) -> u32 {
    let repr_val = v
        .as_repr()
        .unwrap_or_else(|| panic!("expected a Repr CoreValue, got {v:?}"));
    match repr_val.payload() {
        Payload::Bits(bits) => bits.iter().fold(0u32, |acc, &b| (acc << 1) | u32::from(b)),
        other => panic!("expected a Bits payload, got {other:?}"),
    }
}

/// L1-eval-only assertion (the M-981 convention): parse → check → monomorphize → build_registry →
/// eval `main` → compare the `Binary{32}` result to `expected_u32`.
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
        "{label}: L1-eval result {got} does not match expected {expected_u32}"
    );
}

/// Drive a `Bool`-valued `.myc` witness expression; assert it evaluates `True` (→ 1).
fn assert_true(label: &str, myc_bool_expr: &str) {
    let driver =
        format!("fn main() => Binary{{32}} =\n  match {myc_bool_expr} {{ True => one32(), False => zero32() }};\n");
    assert_l1_only_u32(label, &program(&driver), 1);
}

/// Drive a `Bool`-valued `.myc` witness expression; assert it evaluates `False` (→ 0). The
/// non-vacuity guard — proves the comparison actually discriminates.
fn assert_false(label: &str, myc_bool_expr: &str) {
    let driver =
        format!("fn main() => Binary{{32}} =\n  match {myc_bool_expr} {{ True => one32(), False => zero32() }};\n");
    assert_l1_only_u32(label, &program(&driver), 0);
}

// ── Rust → `.myc` fixture encoders (surface input types) ──────────────────────────────────────────

fn encode_u32(n: u32) -> String {
    let mut s = String::from("0b");
    for (count, i) in (0..32).rev().enumerate() {
        if count != 0 && count % 4 == 0 {
            s.push('_');
        }
        s.push(if (n >> i) & 1 == 1 { '1' } else { '0' });
    }
    s
}

fn encode_bytes(s: &str) -> String {
    format!("{s:?}")
}

fn encode_scalar(s: Scalar) -> &'static str {
    match s {
        Scalar::F16 => "SF16",
        Scalar::Bf16 => "SBf16",
        Scalar::F32 => "SF32",
        Scalar::F64 => "SF64",
    }
}

fn encode_sparsity(sp: &Sparsity) -> String {
    match sp {
        Sparsity::Dense => "SpDense".to_owned(),
        Sparsity::Sparse(k) => format!("SpSparse({})", encode_u32(*k)),
    }
}

fn encode_width(w: &Width) -> String {
    match w {
        Width::Lit(n) => format!("WdLit({})", encode_u32(*n)),
        Width::Var(v) => format!("WdVar({})", encode_bytes(v)),
    }
}

fn encode_widthref(w: &WidthRef) -> String {
    match w {
        WidthRef::Lit(n) => format!("WLit({})", encode_u32(*n)),
        WidthRef::Name(v) => format!("WName({})", encode_bytes(v)),
    }
}

fn encode_ty(t: &Ty) -> String {
    match t {
        Ty::Binary(w) => format!("TyBinary({})", encode_width(w)),
        Ty::Ternary(w) => format!("TyTernary({})", encode_width(w)),
        Ty::Dense(d, s) => format!("TyDense({}, {})", encode_u32(*d), encode_scalar(*s)),
        Ty::Vsa {
            model,
            dim,
            sparsity,
        } => format!(
            "TyVsa({}, {}, {})",
            encode_bytes(model),
            encode_u32(*dim),
            encode_sparsity(sparsity)
        ),
        Ty::Data(n, args) => format!("TyData({}, {})", encode_bytes(n), encode_ty_list(args)),
        Ty::Substrate(t) => format!("TySubstrate({})", encode_bytes(t)),
        Ty::Seq(elem, n) => format!("TySeq({}, {})", encode_ty(elem), encode_u32(*n)),
        Ty::Bytes => "TyBytes".to_owned(),
        Ty::Float => "TyFloat".to_owned(),
        Ty::Var(v) => format!("TyVar({})", encode_bytes(v)),
        Ty::Fn(a, r) => format!("TyFn({}, {})", encode_ty(a), encode_ty(r)),
    }
}

fn encode_ty_list(ts: &[Ty]) -> String {
    let mut s = String::from("Nil");
    for t in ts.iter().rev() {
        s = format!("Cons({}, {})", encode_ty(t), s);
    }
    s
}

fn encode_typeref(t: &TypeRef) -> String {
    // The surface guarantee slot is never inspected by `type_repr`; always emit `None`.
    format!("TR({}, None)", encode_basetype(&t.base))
}

fn encode_typeref_list(ts: &[TypeRef]) -> String {
    let mut s = String::from("Nil");
    for t in ts.iter().rev() {
        s = format!("Cons({}, {})", encode_typeref(t), s);
    }
    s
}

fn encode_basetype(b: &BaseType) -> String {
    match b {
        BaseType::Binary(w) => format!("KwBinary({})", encode_widthref(w)),
        BaseType::Ternary(w) => format!("KwTernary({})", encode_widthref(w)),
        BaseType::Dense(d, s) => format!("KwDense({}, {})", encode_u32(*d), encode_scalar(*s)),
        BaseType::Vsa {
            model,
            dim,
            sparsity,
        } => format!(
            "Vsa({}, {}, {})",
            encode_bytes(model),
            encode_u32(*dim),
            encode_sparsity(sparsity)
        ),
        BaseType::Substrate(t) => format!("KwSubstrate({})", encode_bytes(t)),
        BaseType::Seq { elem, len } => format!("KwSeq({}, {})", encode_typeref(elem), encode_u32(*len)),
        BaseType::Bytes => "KwBytes".to_owned(),
        BaseType::Float => "KwFloat".to_owned(),
        BaseType::Named(name, args) => {
            format!("Named({}, {})", encode_bytes(name), encode_typeref_list(args))
        }
        BaseType::Fn(a, r) => format!("FnArrow({}, {})", encode_typeref(a), encode_typeref(r)),
        BaseType::Tuple(elems) => format!("Tuple({})", encode_typeref_list(elems)),
        BaseType::Ambient(_) => {
            panic!("Ambient BaseType is not exercised by the increment-7 differential")
        }
    }
}

fn encode_literal(l: &Literal) -> String {
    match l {
        Literal::Bin(s) => format!("Bin({})", encode_bytes(s)),
        Literal::Trit(s) => format!("Trit({})", encode_bytes(s)),
        Literal::Str(s) => format!("Str({})", encode_bytes(s)),
        Literal::Int(_) => "Int(0b0000000000000000000000000000000000000000000000000000000000000000)"
            .to_owned(),
        Literal::List(_) => "List(Nil)".to_owned(),
        other => panic!("literal {other:?} is not exercised by the increment-7 differential"),
    }
}

fn encode_path(p: &Path) -> String {
    let mut s = String::from("Nil");
    for seg in p.0.iter().rev() {
        s = format!("Cons({}, {})", encode_bytes(seg), s);
    }
    format!("Pth({s})")
}

// ── Rust → `.myc` MIRROR-literal encoders (oracle kernel outputs — Option A §2.2) ─────────────────

fn encode_core_scalar_kind(s: ScalarKind) -> &'static str {
    match s {
        ScalarKind::F16 => "SkF16",
        ScalarKind::Bf16 => "SkBf16",
        ScalarKind::F32 => "SkF32",
        ScalarKind::F64 => "SkF64",
    }
}

fn encode_core_sparsity(sp: &SparsityClass) -> String {
    match sp {
        SparsityClass::Dense => "ScDense".to_owned(),
        SparsityClass::Sparse { max_active } => format!("ScSparse({})", encode_u32(*max_active)),
    }
}

fn encode_core_float_width(w: FloatWidth) -> &'static str {
    match w {
        FloatWidth::F64 => "FwF64",
    }
}

fn encode_core_repr(r: &Repr) -> String {
    match r {
        Repr::Binary { width } => format!("RBinary({})", encode_u32(*width)),
        Repr::Ternary { trits } => format!("RTernary({})", encode_u32(*trits)),
        Repr::Dense { dim, dtype } => {
            format!("RDense({}, {})", encode_u32(*dim), encode_core_scalar_kind(*dtype))
        }
        Repr::Vsa {
            model,
            dim,
            sparsity,
        } => format!(
            "RVsa({}, {}, {})",
            encode_bytes(model),
            encode_u32(*dim),
            encode_core_sparsity(sparsity)
        ),
        Repr::Seq { elem, len } => format!("RSeq({}, {})", encode_core_repr(elem), encode_u32(*len)),
        Repr::Float { width } => format!("RFloat({})", encode_core_float_width(*width)),
        Repr::Bytes => "RBytes".to_owned(),
    }
}

fn encode_core_fn_sig(sig: &FnSig) -> String {
    let mut params = String::from("Nil");
    for p in sig.params.iter().rev() {
        params = format!("Cons({}, {})", encode_core_field_ty_ref(p), params);
    }
    format!(
        "KFS({}, {}, {})",
        encode_u32(sig.arity),
        params,
        encode_core_field_ty_ref(&sig.ret)
    )
}

fn encode_core_field_ty_ref(f: &FieldTyRef) -> String {
    match f {
        FieldTyRef::Repr(r) => format!("FtRepr({})", encode_core_repr(r)),
        FieldTyRef::Data(n) => format!("FtData({})", encode_bytes(n)),
        FieldTyRef::Fn(sig) => format!("FtFn({})", encode_core_fn_sig(sig)),
    }
}

fn encode_core_field_spec(fs: &FieldSpec) -> String {
    match fs {
        FieldSpec::Repr(r) => format!("FsRepr({})", encode_core_repr(r)),
        FieldSpec::Data(n) => format!("FsData({})", encode_bytes(n)),
        FieldSpec::Fn { arity, sig } => {
            format!("FsFn({}, {})", encode_u32(*arity), encode_core_fn_sig(sig))
        }
    }
}

fn encode_core_payload(p: &Payload) -> String {
    match p {
        Payload::Bits(bits) => {
            let mut s = String::from("Nil");
            for b in bits.iter().rev() {
                s = format!("Cons({}, {})", if *b { "0b1" } else { "0b0" }, s);
            }
            format!("PlBits({s})")
        }
        Payload::Trits(trits) => {
            let mut s = String::from("Nil");
            for t in trits.iter().rev() {
                let tag = match t {
                    Trit::Neg => "TkNeg",
                    Trit::Zero => "TkZero",
                    Trit::Pos => "TkPos",
                };
                s = format!("Cons({tag}, {s})");
            }
            format!("PlTrits({s})")
        }
        Payload::Bytes(bytes) => {
            let text =
                String::from_utf8(bytes.clone()).expect("increment-7 Str payload fixtures are ASCII");
            format!("PlBytes({})", encode_bytes(&text))
        }
        other => panic!("payload {other:?} is outside the increment-7 wild-free subset (FLAG-semcore-23)"),
    }
}

fn encode_core_value(v: &Value) -> String {
    // `lit_value` always builds `Meta::exact(Provenance::Root)`; the mirror models exactly that
    // (FLAG-semcore-24). Assert the invariant so a future Meta-varying helper cannot slip through.
    assert_eq!(
        v.meta(),
        &Meta::exact(Provenance::Root),
        "lit_value must produce exact(Root) meta (FLAG-semcore-24)"
    );
    format!(
        "Val({}, {}, MtExactRoot)",
        encode_core_repr(v.repr()),
        encode_core_payload(v.payload())
    )
}

fn encode_core_repr_result<E>(r: &Result<Repr, E>) -> String {
    match r {
        Ok(repr) => format!("Ok({})", encode_core_repr(repr)),
        Err(_) => "Err(\"oracle-refused\")".to_owned(),
    }
}

fn encode_core_value_result<E>(r: &Result<Value, E>) -> String {
    match r {
        Ok(v) => format!("Ok({})", encode_core_value(v)),
        Err(_) => "Err(\"oracle-refused\")".to_owned(),
    }
}

fn encode_core_opt_repr(o: &Option<Repr>) -> String {
    match o {
        Some(r) => format!("Some({})", encode_core_repr(r)),
        None => "None".to_owned(),
    }
}

fn encode_core_opt_field_spec(o: &Option<FieldSpec>) -> String {
    match o {
        Some(fs) => format!("Some({})", encode_core_field_spec(fs)),
        None => "None".to_owned(),
    }
}

fn encode_core_opt_field_ty_ref(o: &Option<FieldTyRef>) -> String {
    match o {
        Some(f) => format!("Some({})", encode_core_field_ty_ref(f)),
        None => "None".to_owned(),
    }
}

// Small fixture constructors keeping test bodies to `assert over a case`.
fn bin(n: u32) -> Ty {
    Ty::Binary(Width::Lit(n))
}
fn data(n: &str, args: Vec<Ty>) -> Ty {
    Ty::Data(n.to_owned(), args)
}
fn tref(base: BaseType) -> TypeRef {
    TypeRef {
        base,
        guarantee: None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// Structural gate: `semcore.myc` (with the increment-7 additions) parses and type-checks green.
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn semcore_elab_parses_and_checks() {
    let nodule = parse(SEMCORE_SRC).unwrap_or_else(|e| panic!("semcore.myc: parse failed: {e}"));
    check_nodule(&nodule).unwrap_or_else(|e| panic!("semcore.myc: check failed: {e}"));
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// Non-vacuity probes: WRONG comparisons must discriminate (yield False → 0).
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn elab_witness_discriminates() {
    // scalar_kind(F16) = SkF16; comparing against the wrong SkF32 must be False.
    assert_false(
        "scalar_kind_wrong",
        "scalark_eq(scalar_kind(SF16), SkF32)",
    );
    // type_repr(Binary{8}) = Ok(RBinary(8)); comparing against Ok(RBinary(16)) must be False.
    assert_false(
        "type_repr_wrong",
        "res_repr_eq(type_repr(TR(KwBinary(WLit(0b0000_0000_0000_0000_0000_0000_0000_1000)), None)), Ok(RBinary(0b0000_0000_0000_0000_0000_0000_0001_0000)))",
    );
    // field_spec(Binary{8}) = Some(FsRepr(RBinary(8))); comparing against None must be False.
    assert_false(
        "field_spec_wrong",
        &format!("opt_fs_eq(field_spec({}), None)", encode_ty(&bin(8))),
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// scalar_kind (LIVE — elab::scalar_kind): exhaustive over the 4 scalar kinds.
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn scalar_kind_cases() {
    for s in [Scalar::F16, Scalar::Bf16, Scalar::F32, Scalar::F64] {
        let want = scalar_kind(s);
        assert_true(
            &format!("scalar_kind_{s:?}"),
            &format!(
                "scalark_eq(scalar_kind({}), {})",
                encode_scalar(s),
                encode_core_scalar_kind(want)
            ),
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// sparsity_class (LIVE — elab::sparsity_class): Dense + Sparse(k) (the max_active passthrough).
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn sparsity_class_cases() {
    let cases = [
        Sparsity::Dense,
        Sparsity::Sparse(1),
        Sparsity::Sparse(8),
        Sparsity::Sparse(4096),
    ];
    for (i, sp) in cases.iter().enumerate() {
        let want = sparsity_class(sp);
        assert_true(
            &format!("sparsity_class_{i}"),
            &format!(
                "sparsityc_eq(sparsity_class({}), {})",
                encode_sparsity(sp),
                encode_core_sparsity(&want)
            ),
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// type_repr (LIVE — elab::type_repr): every arm, incl. width-var refusals, the VSA model
// canonicalization (`MAP_I`→`MAP-I`), nested Seq, and the named/Substrate/Fn/Tuple refusals.
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn type_repr_cases() {
    let cases: Vec<BaseType> = vec![
        BaseType::Binary(WidthRef::Lit(8)),
        BaseType::Binary(WidthRef::Name("N".to_owned())), // width-var → Err
        BaseType::Ternary(WidthRef::Lit(6)),
        BaseType::Ternary(WidthRef::Name("M".to_owned())), // width-var → Err
        BaseType::Dense(1024, Scalar::F32),
        BaseType::Dense(16, Scalar::Bf16),
        // Surface model id `MAP_I` canonicalizes to `MAP-I` (both sides via vsa_kernel_model_id).
        BaseType::Vsa {
            model: "MAP_I".to_owned(),
            dim: 256,
            sparsity: Sparsity::Dense,
        },
        BaseType::Vsa {
            model: "FHRR".to_owned(),
            dim: 512,
            sparsity: Sparsity::Sparse(8),
        },
        BaseType::Seq {
            elem: Box::new(tref(BaseType::Binary(WidthRef::Lit(8)))),
            len: 4,
        },
        // Nested Seq of Bytes.
        BaseType::Seq {
            elem: Box::new(tref(BaseType::Seq {
                elem: Box::new(tref(BaseType::Bytes)),
                len: 2,
            })),
            len: 3,
        },
        BaseType::Bytes,
        BaseType::Float,
        BaseType::Substrate("file".to_owned()), // → Err
        BaseType::Named("Bool".to_owned(), vec![]), // → Err
        BaseType::Fn(
            Box::new(tref(BaseType::Binary(WidthRef::Lit(8)))),
            Box::new(tref(BaseType::Bytes)),
        ), // → Err
        BaseType::Tuple(vec![
            tref(BaseType::Binary(WidthRef::Lit(8))),
            tref(BaseType::Bytes),
        ]), // → Err
    ];
    for (i, base) in cases.iter().enumerate() {
        let t = tref(base.clone());
        let want = type_repr("t", &t);
        assert_true(
            &format!("type_repr_{i}"),
            &format!(
                "res_repr_eq(type_repr({}), {})",
                encode_typeref(&t),
                encode_core_repr_result(&want)
            ),
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// lit_value (LIVE — elab::lit_value): the wild-free arms (Bin/Trit/Str) + the refusals (Int/List).
// The DEFERRED arms (LBytes/LFloat) are covered separately (they refuse; not compared to the oracle).
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn lit_value_cases() {
    let cases: Vec<Literal> = vec![
        Literal::Bin("1010".to_owned()),
        Literal::Bin("1010_1100".to_owned()), // separators filtered
        Literal::Bin("1".to_owned()),
        Literal::Trit("+0-".to_owned()),
        Literal::Trit("0".to_owned()),
        Literal::Str("hello".to_owned()),
        Literal::Str("".to_owned()), // empty → Repr::Bytes, empty payload (well-formed)
        Literal::Int(0),             // → Err (no representation family)
        Literal::List(vec![]),       // → Err (lowers through expr_inner)
    ];
    for (i, l) in cases.iter().enumerate() {
        let want = lit_value("t", l);
        assert_true(
            &format!("lit_value_{i}"),
            &format!(
                "res_value_eq(lit_value({}), {})",
                encode_literal(l),
                encode_core_value_result(&want)
            ),
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// lit_value DEFERRED arms (FLAG-semcore-25): the `.myc` port refuses `0x..`/float literals
// never-silently (G2) rather than faking a value. A `.myc`-only assertion (no oracle agreement).
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn lit_value_deferred_arms_refuse() {
    assert_true(
        "lit_value_lbytes_refuses",
        "match lit_value(LBytes(\"deadbeef\")) { Err(_) => True, Ok(_) => False }",
    );
    assert_true(
        "lit_value_lfloat_refuses",
        "match lit_value(LFloat(\"1.5\")) { Err(_) => True, Ok(_) => False }",
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// ty_to_repr (LIVE — elab::ty_to_repr): repr types resolve; Data/Var/Substrate/Fn → None.
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn ty_to_repr_cases() {
    let cases: Vec<Ty> = vec![
        bin(8),
        Ty::Binary(Width::Var("N".to_owned())), // → None
        Ty::Ternary(Width::Lit(6)),
        Ty::Dense(32, Scalar::F64),
        Ty::Vsa {
            model: "MAP-I".to_owned(), // already-canonical (checked Ty)
            dim: 128,
            sparsity: Sparsity::Sparse(4),
        },
        Ty::Seq(Box::new(bin(8)), 4),
        Ty::Seq(Box::new(data("List", vec![bin(8)])), 2), // elem has no repr → None
        Ty::Bytes,
        Ty::Float,
        data("Bool", vec![]), // → None
        Ty::Var("A".to_owned()), // → None
        Ty::Substrate("file".to_owned()), // → None
        Ty::Fn(Box::new(bin(8)), Box::new(Ty::Bytes)), // → None
    ];
    for (i, t) in cases.iter().enumerate() {
        let want = ty_to_repr(t);
        assert_true(
            &format!("ty_to_repr_{i}"),
            &format!(
                "opt_repr_eq(ty_to_repr({}), {})",
                encode_ty(t),
                encode_core_opt_repr(&want)
            ),
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// ty_to_field_ty_ref (LIVE — elab::ty_to_field_ty_ref): Data(∅)→FtData, Fn→FtFn, repr→FtRepr, None else.
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn ty_to_field_ty_ref_cases() {
    let cases: Vec<Ty> = vec![
        data("Bool", vec![]),                    // → FtData
        data("List", vec![bin(8)]),              // generic Data → None
        Ty::Var("A".to_owned()),                 // → None
        Ty::Substrate("file".to_owned()),        // → None
        bin(8),                                  // → FtRepr(RBinary(8))
        Ty::Bytes,                               // → FtRepr(RBytes)
        Ty::Fn(Box::new(bin(8)), Box::new(Ty::Bytes)), // → FtFn(sig)
        // Nested (curried) arrow: A => (B => C).
        Ty::Fn(
            Box::new(bin(8)),
            Box::new(Ty::Fn(Box::new(Ty::Bytes), Box::new(Ty::Float))),
        ),
        // A Fn with a non-monomorphic leaf → None.
        Ty::Fn(Box::new(Ty::Var("A".to_owned())), Box::new(Ty::Bytes)),
    ];
    for (i, t) in cases.iter().enumerate() {
        let want = ty_to_field_ty_ref(t);
        assert_true(
            &format!("ty_to_field_ty_ref_{i}"),
            &format!(
                "opt_ftr_eq(ty_to_field_ty_ref({}), {})",
                encode_ty(t),
                encode_core_opt_field_ty_ref(&want)
            ),
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// field_spec (LIVE — elab::field_spec): every arm, incl. Data(∅)→FsData, generic Data→None, Fn→FsFn.
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn field_spec_cases() {
    let cases: Vec<Ty> = vec![
        bin(8),
        Ty::Binary(Width::Var("N".to_owned())), // → None
        Ty::Ternary(Width::Lit(6)),
        Ty::Dense(1024, Scalar::F32),
        Ty::Vsa {
            model: "MAP-I".to_owned(),
            dim: 256,
            sparsity: Sparsity::Dense,
        },
        Ty::Seq(Box::new(bin(8)), 4),
        Ty::Seq(Box::new(Ty::Var("A".to_owned())), 2), // elem no repr → None
        Ty::Bytes,
        Ty::Float,
        data("Bool", vec![]),   // → FsData
        data("List", vec![bin(8)]), // generic → None
        Ty::Var("A".to_owned()),    // → None
        Ty::Substrate("file".to_owned()), // → None
        Ty::Fn(Box::new(bin(8)), Box::new(Ty::Bytes)), // → FsFn
        // Fn with an unresolvable leaf → None.
        Ty::Fn(Box::new(bin(8)), Box::new(Ty::Var("R".to_owned()))),
    ];
    for (i, t) in cases.iter().enumerate() {
        let want = field_spec(t);
        assert_true(
            &format!("field_spec_{i}"),
            &format!(
                "opt_fs_eq(field_spec({}), {})",
                encode_ty(t),
                encode_core_opt_field_spec(&want)
            ),
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// policy_name_preimage (LIVE — elab::policy_name_preimage): the domain-separated preimage
// (`policy-name.v0:<dotted>`). The BLAKE3 hashing step is DEFERRED (FLAG-semcore-27).
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn policy_name_preimage_cases() {
    let cases: Vec<Path> = vec![
        Path(vec!["roundtrip".to_owned()]),
        Path(vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]),
        Path(vec![]), // empty → "policy-name.v0:"
    ];
    for (i, p) in cases.iter().enumerate() {
        let want = policy_name_preimage(p);
        assert_true(
            &format!("policy_name_preimage_{i}"),
            &format!(
                "match bytes_eq(policy_name_preimage({}), {}) {{ 0b1 => True, _ => False }}",
                encode_path(p),
                encode_bytes(&want)
            ),
        );
    }
}
