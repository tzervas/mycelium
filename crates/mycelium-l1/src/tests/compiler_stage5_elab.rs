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
//! **Differential method — harness MARSHALLING (M-1013 STEP 2; DN-26 §10.2).** Every case runs the
//! REAL Rust `elab::*` oracle on a fixture, producing a genuine `mycelium_core::{Value,Repr,FieldSpec,
//! …}`. It then evaluates the `.myc` port helper *directly* (the driver's `main` returns the mirror
//! value, not a `Bool`), and DECODES that `L1Value` mirror ADT back into the real `mycelium_core` type
//! (the `decode_*` family below — the never-silent inverse of the mirror constructors). The two
//! independently-produced kernel values are compared with **Rust's own trusted derived `==`** — no
//! hand-written `.myc`-side `_eq` comparator on the trust path. A mis-lowering diverges the decoded
//! value from the oracle and fails the `assert_eq!`. The two productions are genuinely independent (the
//! port never calls the kernel; the oracle never calls the port). This SUPERSEDES the increment-7
//! landing's `.myc`-side structural-equality comparators (the retired FLAG-semcore-28 `_eq` family);
//! the decoder is now the trust surface, guarded by `marshal_discriminates` (its non-vacuity twin).
//!
//! M-981 applies: only the L1-eval leg is exercised (small synthetic fixtures, not a corpus program).
//! `scalar_kind`/`sparsity_class` are covered exhaustively (they twin the increment-4 tags).

use crate::ast::{BaseType, Literal, Path, Scalar, Sparsity, TypeRef, WidthRef};
use crate::checkty::{check_nodule, Ty, Width};
use crate::elab::{
    field_spec, lit_value, policy_name_preimage, scalar_kind, sparsity_class, ty_to_field_ty_ref,
    ty_to_repr, type_repr,
};
use crate::eval::{Evaluator, L1Value};
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

// ── L1Value → mycelium_core decoders (M-1013 STEP 2; DN-26 §10.2 — the marshalling inverse) ────────
//
// Each decoder is the never-silent inverse of a mirror constructor: it walks the port's `L1Value`
// output and rebuilds the REAL `mycelium_core` type, panicking (the harness's established failure
// mode) on an unexpected constructor rather than silently mis-decoding (G2). A mirror ADT node comes
// back as `L1Value::Data { ty, ctor, fields }`; a `Binary{N}`/`Bytes` leaf comes back as
// `L1Value::Repr(Value)` (read via `as_repr()`).

/// The base constructor name, with `monomorphize`'s injective mangle suffix stripped
/// (`mono.rs` §4: names are joined with `$`, with a `#` kind-tag on nullary data). A generic ctor
/// specializes to `Cons$Binary1`/`Some$Repr`/`Ok$Repr$Bytes`/…; the monomorphic mirror ctors
/// (`RBinary`, `PlBits`, `Val`, …) carry no separator and pass through unchanged.
fn base_ctor(ctor: &str) -> &str {
    let end = ctor.find(['$', '#']).unwrap_or(ctor.len());
    &ctor[..end]
}

/// Every mirror ADT node is an `L1Value::Data`; return its (unmangled) constructor name + fields.
fn expect_data<'a>(v: &'a L1Value, what: &str) -> (&'a str, &'a [L1Value]) {
    match v {
        L1Value::Data { ctor, fields, .. } => (base_ctor(ctor), fields.as_slice()),
        other => panic!("marshal {what}: expected a Data node, got {other:?}"),
    }
}

/// A `Binary{32}` mirror int → `u32`, MSB-first — the exact convention `core_bits_as_u32` used.
fn decode_u32(v: &L1Value) -> u32 {
    match v.as_repr().map(Value::payload) {
        Some(Payload::Bits(bits)) => bits.iter().fold(0u32, |acc, &b| (acc << 1) | u32::from(b)),
        other => panic!("marshal decode_u32: expected a Repr(Binary) Bits leaf, got {other:?}"),
    }
}

/// A `Binary{1}` mirror bit → `bool`.
fn decode_bit(v: &L1Value) -> bool {
    match v.as_repr().map(Value::payload) {
        Some(Payload::Bits(bits)) if bits.len() == 1 => bits[0],
        other => panic!("marshal decode_bit: expected a 1-bit Bits leaf, got {other:?}"),
    }
}

/// A `Bytes` leaf → raw bytes.
fn decode_bytes(v: &L1Value) -> Vec<u8> {
    match v.as_repr().map(Value::payload) {
        Some(Payload::Bytes(b)) => b.clone(),
        other => panic!("marshal decode_bytes: expected a Bytes leaf, got {other:?}"),
    }
}

/// A `Bytes` leaf → `String` (the increment-7 fixtures are ASCII).
fn decode_string(v: &L1Value) -> String {
    String::from_utf8(decode_bytes(v)).expect("marshal decode_string: non-UTF8 bytes")
}

/// A `.myc` `Vec[A]` (`Nil | Cons(A, Vec[A])`) → `Vec<T>`, decoding each element with `elem`.
fn decode_vec<T>(v: &L1Value, elem: impl Fn(&L1Value) -> T) -> Vec<T> {
    let mut out = Vec::new();
    let mut cur = v;
    loop {
        let (ctor, fields) = expect_data(cur, "Vec");
        match ctor {
            "Nil" => return out,
            "Cons" => {
                out.push(elem(&fields[0]));
                cur = &fields[1];
            }
            other => panic!("marshal decode_vec: unexpected ctor {other}"),
        }
    }
}

fn decode_scalar_kind(v: &L1Value) -> ScalarKind {
    match expect_data(v, "ScalarK").0 {
        "SkF16" => ScalarKind::F16,
        "SkBf16" => ScalarKind::Bf16,
        "SkF32" => ScalarKind::F32,
        "SkF64" => ScalarKind::F64,
        c => panic!("marshal decode_scalar_kind: unexpected ctor {c}"),
    }
}

fn decode_sparsity_class(v: &L1Value) -> SparsityClass {
    let (ctor, fields) = expect_data(v, "SparsityC");
    match ctor {
        "ScDense" => SparsityClass::Dense,
        "ScSparse" => SparsityClass::Sparse {
            max_active: decode_u32(&fields[0]),
        },
        c => panic!("marshal decode_sparsity_class: unexpected ctor {c}"),
    }
}

fn decode_float_width(v: &L1Value) -> FloatWidth {
    match expect_data(v, "FloatW").0 {
        "FwF64" => FloatWidth::F64,
        c => panic!("marshal decode_float_width: unexpected ctor {c}"),
    }
}

fn decode_trit(v: &L1Value) -> Trit {
    match expect_data(v, "TritK").0 {
        "TkNeg" => Trit::Neg,
        "TkZero" => Trit::Zero,
        "TkPos" => Trit::Pos,
        c => panic!("marshal decode_trit: unexpected ctor {c}"),
    }
}

fn decode_repr(v: &L1Value) -> Repr {
    let (ctor, fields) = expect_data(v, "Repr");
    match ctor {
        "RBinary" => Repr::Binary {
            width: decode_u32(&fields[0]),
        },
        "RTernary" => Repr::Ternary {
            trits: decode_u32(&fields[0]),
        },
        "RDense" => Repr::Dense {
            dim: decode_u32(&fields[0]),
            dtype: decode_scalar_kind(&fields[1]),
        },
        "RVsa" => Repr::Vsa {
            model: decode_string(&fields[0]),
            dim: decode_u32(&fields[1]),
            sparsity: decode_sparsity_class(&fields[2]),
        },
        "RSeq" => Repr::Seq {
            elem: Box::new(decode_repr(&fields[0])),
            len: decode_u32(&fields[1]),
        },
        "RFloat" => Repr::Float {
            width: decode_float_width(&fields[0]),
        },
        "RBytes" => Repr::Bytes,
        c => panic!("marshal decode_repr: unexpected ctor {c}"),
    }
}

fn decode_payload(v: &L1Value) -> Payload {
    let (ctor, fields) = expect_data(v, "Payload");
    match ctor {
        "PlBits" => Payload::Bits(decode_vec(&fields[0], decode_bit)),
        "PlTrits" => Payload::Trits(decode_vec(&fields[0], decode_trit)),
        "PlBytes" => Payload::Bytes(decode_bytes(&fields[0])),
        c => panic!("marshal decode_payload: unexpected ctor {c}"),
    }
}

fn decode_meta(v: &L1Value) -> Meta {
    match expect_data(v, "Meta").0 {
        "MtExactRoot" => Meta::exact(Provenance::Root),
        c => panic!("marshal decode_meta: unexpected ctor {c}"),
    }
}

/// Rebuild a real `Value` through its ONLY constructor (`Value::new` runs `check_well_formed` +
/// payload/repr matching + canonical-NaN normalization — value.rs). A `Value::new` rejection here is a
/// real port divergence (the port built a malformed mirror), never swallowed.
fn decode_value(v: &L1Value) -> Value {
    let (ctor, fields) = expect_data(v, "Value");
    match ctor {
        "Val" => Value::new(
            decode_repr(&fields[0]),
            decode_payload(&fields[1]),
            decode_meta(&fields[2]),
        )
        .unwrap_or_else(|e| {
            panic!("marshal decode_value: Value::new rejected a decoded mirror (port divergence): {e:?}")
        }),
        c => panic!("marshal decode_value: unexpected ctor {c}"),
    }
}

fn decode_fn_sig(v: &L1Value) -> FnSig {
    let (ctor, fields) = expect_data(v, "KFnSig");
    match ctor {
        "KFS" => FnSig {
            arity: decode_u32(&fields[0]),
            params: decode_vec(&fields[1], decode_field_ty_ref),
            ret: Box::new(decode_field_ty_ref(&fields[2])),
        },
        c => panic!("marshal decode_fn_sig: unexpected ctor {c}"),
    }
}

fn decode_field_ty_ref(v: &L1Value) -> FieldTyRef {
    let (ctor, fields) = expect_data(v, "FieldTyRef");
    match ctor {
        "FtRepr" => FieldTyRef::Repr(decode_repr(&fields[0])),
        "FtData" => FieldTyRef::Data(decode_string(&fields[0])),
        "FtFn" => FieldTyRef::Fn(Box::new(decode_fn_sig(&fields[0]))),
        c => panic!("marshal decode_field_ty_ref: unexpected ctor {c}"),
    }
}

fn decode_field_spec(v: &L1Value) -> FieldSpec {
    let (ctor, fields) = expect_data(v, "FieldSpec");
    match ctor {
        "FsRepr" => FieldSpec::Repr(decode_repr(&fields[0])),
        "FsData" => FieldSpec::Data(decode_string(&fields[0])),
        "FsFn" => FieldSpec::Fn {
            arity: decode_u32(&fields[0]),
            sig: decode_fn_sig(&fields[1]),
        },
        c => panic!("marshal decode_field_spec: unexpected ctor {c}"),
    }
}

fn decode_option<T>(v: &L1Value, elem: impl Fn(&L1Value) -> T) -> Option<T> {
    let (ctor, fields) = expect_data(v, "Option");
    match ctor {
        "None" => None,
        "Some" => Some(elem(&fields[0])),
        c => panic!("marshal decode_option: unexpected ctor {c}"),
    }
}

/// A `.myc` `Result[A, Bytes]` → `Result<T, ()>`. The `Err` arm's message differs across the two
/// independent productions (Rust `ElabError` vs the `.myc` `Bytes` string), so it is normalized to
/// `()` — the exact posture of the retired `res_*_eq` (any `Err` == any `Err`; only the `Ok` payload
/// is a meaningful differential).
fn decode_result<T>(v: &L1Value, elem: impl Fn(&L1Value) -> T) -> Result<T, ()> {
    let (ctor, fields) = expect_data(v, "Result");
    match ctor {
        "Ok" => Ok(elem(&fields[0])),
        "Err" => Err(()),
        c => panic!("marshal decode_result: unexpected ctor {c}"),
    }
}

// ── the marshalling assertion helper ───────────────────────────────────────────────────────────────

/// Parse → check → monomorphize → eval `main` → DECODE the mirror `L1Value` → `assert_eq!` against the
/// trusted Rust oracle value. The comparator is Rust's own derived `==` (no `.myc`-side `_eq`). No
/// `build_registry`/`to_core` on this path — the decoder walks the `L1Value` (and its embedded
/// `core::Value` leaves) directly.
fn assert_l1_marshal<T: PartialEq + std::fmt::Debug>(
    label: &str,
    driver: &str,
    decode: impl Fn(&L1Value) -> T,
    want: T,
) {
    let src = program(driver);
    let env = check_nodule(&parse(&src).unwrap_or_else(|e| panic!("{label}: parse failed: {e}")))
        .unwrap_or_else(|e| panic!("{label}: check failed: {e}"));
    let mono =
        monomorphize(&env, "main").unwrap_or_else(|e| panic!("{label}: monomorphize failed: {e}"));
    let l1_val = Evaluator::new(&mono)
        .call("main", vec![])
        .unwrap_or_else(|e| panic!("{label}: L1-eval failed: {e}"));
    let got = decode(&l1_val);
    assert_eq!(
        got, want,
        "{label}: decoded marshal {got:?} does not match oracle {want:?}"
    );
}

/// Eval a bare mirror-literal driver and decode it — the `marshal_discriminates` primitive (no oracle;
/// used only to prove the decoder distinguishes single-dimension-distinct mirror values).
fn decode_driver<T>(ret_ty: &str, expr: &str, decode: impl Fn(&L1Value) -> T) -> T {
    let driver = format!("fn main() => {ret_ty} = {expr};\n");
    let src = program(&driver);
    let env = check_nodule(&parse(&src).unwrap_or_else(|e| panic!("decode_driver parse: {e}")))
        .unwrap_or_else(|e| panic!("decode_driver check: {e}"));
    let mono = monomorphize(&env, "main").unwrap_or_else(|e| panic!("decode_driver mono: {e}"));
    let l1_val = Evaluator::new(&mono)
        .call("main", vec![])
        .unwrap_or_else(|e| panic!("decode_driver eval: {e}"));
    decode(&l1_val)
}

// ── Rust → `.myc` fixture encoders (surface INPUT types — unchanged; build the driver's argument) ──

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
        BaseType::Seq { elem, len } => {
            format!("KwSeq({}, {})", encode_typeref(elem), encode_u32(*len))
        }
        BaseType::Bytes => "KwBytes".to_owned(),
        BaseType::Float => "KwFloat".to_owned(),
        BaseType::Named(name, args) => {
            format!(
                "Named({}, {})",
                encode_bytes(name),
                encode_typeref_list(args)
            )
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
        Literal::Int(_) => {
            "Int(0b0000000000000000000000000000000000000000000000000000000000000000)".to_owned()
        }
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
// Decoder non-vacuity: the marshalling decoder must DISCRIMINATE on every dimension it reads.
//
// CONVENTION (M-1013 STEP 2 — the marshalling twin of M-1012's `elab_witness_discriminates`, binding
// on every future self-hosting increment). With the differential now comparing decoded values by
// Rust's trusted derived `==`, a WRONG port output fails `assert_eq!` by construction — the old
// "comparator isn't vacuously True" obligation dissolves. The NEW trust surface is the `decode_*`
// family, whose failure mode is *dropping a dimension* (mapping distinct mirror values onto the same
// Rust value → a silent false pass). This test closes exactly that hole: for each decoder arm, decode
// two mirror literals differing in EXACTLY ONE dimension and assert the decoded Rust values are `!=` —
// proving the decoder actually reads that dimension. `decode_meta`/`Meta` is the one documented
// exception (single-inhabitant `MtExactRoot`, FLAG-semcore-24: no two DIFFERING `Meta` are
// constructible — becomes an addable case the moment `Meta` gains a second constructor).
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn marshal_discriminates() {
    // decode_scalar_kind — the variant it selects.
    assert_ne!(
        decode_driver("ScalarK", "SkF16", decode_scalar_kind),
        decode_driver("ScalarK", "SkF32", decode_scalar_kind)
    );
    // decode_float_width — single-inhabitant today (FwF64); no distinguishing pair (documented, like
    // decode_meta) — becomes an addable case when FloatW gains a second constructor.

    // decode_u32 (via RBinary width) — the integer dimension it folds.
    assert_ne!(
        decode_driver("Repr", &format!("RBinary({})", encode_u32(8)), decode_repr),
        decode_driver("Repr", &format!("RBinary({})", encode_u32(16)), decode_repr)
    );
    // decode_repr — the variant tag (RBinary vs RTernary at equal width).
    assert_ne!(
        decode_driver("Repr", &format!("RBinary({})", encode_u32(8)), decode_repr),
        decode_driver("Repr", &format!("RTernary({})", encode_u32(8)), decode_repr)
    );
    // decode_scalar_kind inside RDense — the dtype field.
    assert_ne!(
        decode_driver(
            "Repr",
            &format!("RDense({}, SkF16)", encode_u32(4)),
            decode_repr
        ),
        decode_driver(
            "Repr",
            &format!("RDense({}, SkF32)", encode_u32(4)),
            decode_repr
        )
    );
    // decode_sparsity_class — ScDense vs ScSparse, and the max_active field of ScSparse.
    assert_ne!(
        decode_driver("SparsityC", "ScDense", decode_sparsity_class),
        decode_driver(
            "SparsityC",
            &format!("ScSparse({})", encode_u32(8)),
            decode_sparsity_class
        )
    );
    assert_ne!(
        decode_driver(
            "SparsityC",
            &format!("ScSparse({})", encode_u32(8)),
            decode_sparsity_class
        ),
        decode_driver(
            "SparsityC",
            &format!("ScSparse({})", encode_u32(16)),
            decode_sparsity_class
        )
    );
    // decode_string inside RVsa — the model field (and dim, and sparsity, all read by decode_repr).
    assert_ne!(
        decode_driver(
            "Repr",
            &format!("RVsa({}, {}, ScDense)", encode_bytes("A"), encode_u32(4)),
            decode_repr
        ),
        decode_driver(
            "Repr",
            &format!("RVsa({}, {}, ScDense)", encode_bytes("B"), encode_u32(4)),
            decode_repr
        )
    );
    // decode_repr RSeq — the elem field and the len field, each varied once.
    assert_ne!(
        decode_driver(
            "Repr",
            &format!("RSeq(RBytes, {})", encode_u32(2)),
            decode_repr
        ),
        decode_driver(
            "Repr",
            &format!("RSeq(RBinary({}), {})", encode_u32(8), encode_u32(2)),
            decode_repr
        )
    );
    assert_ne!(
        decode_driver(
            "Repr",
            &format!("RSeq(RBytes, {})", encode_u32(2)),
            decode_repr
        ),
        decode_driver(
            "Repr",
            &format!("RSeq(RBytes, {})", encode_u32(3)),
            decode_repr
        )
    );
    // decode_bit inside PlBits — a single bit position (also exercises decode_vec length below).
    assert_ne!(
        decode_driver(
            "Payload",
            "PlBits(Cons(0b1, Cons(0b0, Nil)))",
            decode_payload
        ),
        decode_driver(
            "Payload",
            "PlBits(Cons(0b1, Cons(0b1, Nil)))",
            decode_payload
        )
    );
    // decode_vec length — PlBits of different lengths.
    assert_ne!(
        decode_driver("Payload", "PlBits(Cons(0b1, Nil))", decode_payload),
        decode_driver(
            "Payload",
            "PlBits(Cons(0b1, Cons(0b1, Nil)))",
            decode_payload
        )
    );
    // decode_trit inside PlTrits — the trit variant.
    assert_ne!(
        decode_driver("Payload", "PlTrits(Cons(TkPos, Nil))", decode_payload),
        decode_driver("Payload", "PlTrits(Cons(TkNeg, Nil))", decode_payload)
    );
    // decode_payload — the payload variant tag (PlBits vs PlBytes).
    assert_ne!(
        decode_driver("Payload", "PlBytes(\"x\")", decode_payload),
        decode_driver("Payload", "PlBytes(\"y\")", decode_payload)
    );
    // decode_value — two `Val` differing ONLY in payload (isolates the payload read specifically: the
    // repr and meta agree, so a decoder that dropped the payload would collapse them — the marshalling
    // migration of the old `lit_value_payload_wrong` probe).
    assert_ne!(
        decode_driver(
            "Value",
            &format!(
                "Val(RBinary({}), PlBits(Cons(0b1, Nil)), MtExactRoot)",
                encode_u32(1)
            ),
            decode_value
        ),
        decode_driver(
            "Value",
            &format!(
                "Val(RBinary({}), PlBits(Cons(0b0, Nil)), MtExactRoot)",
                encode_u32(1)
            ),
            decode_value
        )
    );
    // decode_field_ty_ref — the variant tag (FtRepr vs FtData).
    assert_ne!(
        decode_driver("FieldTyRef", "FtRepr(RBytes)", decode_field_ty_ref),
        decode_driver("FieldTyRef", "FtData(\"D\")", decode_field_ty_ref)
    );
    // decode_fn_sig — arity, params, and ret, each varied once (via FtFn wrapping a KFnSig).
    assert_ne!(
        decode_driver(
            "FieldTyRef",
            &format!("FtFn(KFS({}, Nil, FtRepr(RBytes)))", encode_u32(1)),
            decode_field_ty_ref
        ),
        decode_driver(
            "FieldTyRef",
            &format!("FtFn(KFS({}, Nil, FtRepr(RBytes)))", encode_u32(2)),
            decode_field_ty_ref
        )
    );
    assert_ne!(
        decode_driver(
            "FieldTyRef",
            &format!("FtFn(KFS({}, Nil, FtRepr(RBytes)))", encode_u32(1)),
            decode_field_ty_ref
        ),
        decode_driver(
            "FieldTyRef",
            &format!(
                "FtFn(KFS({}, Cons(FtRepr(RBytes), Nil), FtRepr(RBytes)))",
                encode_u32(1)
            ),
            decode_field_ty_ref
        )
    );
    assert_ne!(
        decode_driver(
            "FieldTyRef",
            &format!("FtFn(KFS({}, Nil, FtRepr(RBytes)))", encode_u32(1)),
            decode_field_ty_ref
        ),
        decode_driver(
            "FieldTyRef",
            &format!("FtFn(KFS({}, Nil, FtData(\"R\")))", encode_u32(1)),
            decode_field_ty_ref
        )
    );
    // decode_field_spec — the variant tag (FsRepr vs FsData), and the arity of FsFn.
    assert_ne!(
        decode_driver("FieldSpec", "FsRepr(RBytes)", decode_field_spec),
        decode_driver("FieldSpec", "FsData(\"D\")", decode_field_spec)
    );
    assert_ne!(
        decode_driver(
            "FieldSpec",
            &format!(
                "FsFn({}, KFS({}, Nil, FtRepr(RBytes)))",
                encode_u32(1),
                encode_u32(1)
            ),
            decode_field_spec
        ),
        decode_driver(
            "FieldSpec",
            &format!(
                "FsFn({}, KFS({}, Nil, FtRepr(RBytes)))",
                encode_u32(2),
                encode_u32(1)
            ),
            decode_field_spec
        )
    );
    // decode_option — Some(x) vs None.
    assert_ne!(
        decode_driver("Option[Repr]", "Some(RBytes)", |v| decode_option(
            v,
            decode_repr
        )),
        decode_driver("Option[Repr]", "None", |v| decode_option(v, decode_repr))
    );
    // decode_result — Ok(x) vs Err (the only two arms; Err normalizes to `()`).
    assert_ne!(
        decode_driver("Result[Repr, Bytes]", "Ok(RBytes)", |v| decode_result(
            v,
            decode_repr
        )),
        decode_driver("Result[Repr, Bytes]", "Err(\"e\")", |v| decode_result(
            v,
            decode_repr
        ))
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// scalar_kind (LIVE — elab::scalar_kind): exhaustive over the 4 scalar kinds.
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn scalar_kind_cases() {
    for s in [Scalar::F16, Scalar::Bf16, Scalar::F32, Scalar::F64] {
        assert_l1_marshal(
            &format!("scalar_kind_{s:?}"),
            &format!(
                "fn main() => ScalarK = scalar_kind({});\n",
                encode_scalar(s)
            ),
            decode_scalar_kind,
            scalar_kind(s),
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
        assert_l1_marshal(
            &format!("sparsity_class_{i}"),
            &format!(
                "fn main() => SparsityC = sparsity_class({});\n",
                encode_sparsity(sp)
            ),
            decode_sparsity_class,
            sparsity_class(sp),
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
        BaseType::Substrate("file".to_owned()),     // → Err
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
        assert_l1_marshal(
            &format!("type_repr_{i}"),
            &format!(
                "fn main() => Result[Repr, Bytes] = type_repr({});\n",
                encode_typeref(&t)
            ),
            |v| decode_result(v, decode_repr),
            want.map_err(|_| ()),
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// lit_value (LIVE — elab::lit_value): the wild-free arms (Bin/Trit/Str) + the refusals (Int/List).
// The DEFERRED arms (LBytes/LFloat) are covered separately (they refuse; not compared to the oracle).
// The `width == 0` LOWER-bound refusal (Bin/Trit) and `trit_of`'s Err arm are also exercised here —
// the untested refusal branches FLAG-semcore-29 calls out (the `.myc` port replicates the LOWER
// bound but not the `MAX_DIM` UPPER bound; see FLAG-semcore-29 in `semcore.myc` for that gap).
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn lit_value_cases() {
    let cases: Vec<Literal> = vec![
        Literal::Bin("1010".to_owned()),
        Literal::Bin("1010_1100".to_owned()), // separators filtered
        Literal::Bin("1".to_owned()),
        Literal::Bin("".to_owned()), // empty -> width==0 refusal (both sides Err)
        Literal::Trit("+0-".to_owned()),
        Literal::Trit("0".to_owned()),
        Literal::Trit("".to_owned()), // empty -> width==0 refusal (both sides Err)
        Literal::Trit("x".to_owned()), // invalid trit char -> `trit_of`'s Err arm (both sides Err)
        Literal::Str("hello".to_owned()),
        Literal::Str("".to_owned()), // empty → Repr::Bytes, empty payload (well-formed)
        Literal::Int(0),             // → Err (no representation family)
        Literal::List(vec![]),       // → Err (lowers through expr_inner)
    ];
    for (i, l) in cases.iter().enumerate() {
        let want = lit_value("t", l);
        assert_l1_marshal(
            &format!("lit_value_{i}"),
            &format!(
                "fn main() => Result[Value, Bytes] = lit_value({});\n",
                encode_literal(l)
            ),
            |v| decode_result(v, decode_value),
            want.map_err(|_| ()),
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// lit_value DEFERRED arms (FLAG-semcore-25): the `.myc` port refuses `0x..`/float literals
// never-silently (G2) rather than faking a value. No oracle agreement — the port must return `Err`.
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn lit_value_deferred_arms_refuse() {
    assert_l1_marshal(
        "lit_value_lbytes_refuses",
        "fn main() => Result[Value, Bytes] = lit_value(LBytes(\"deadbeef\"));\n",
        |v| decode_result(v, decode_value),
        Err(()),
    );
    assert_l1_marshal(
        "lit_value_lfloat_refuses",
        "fn main() => Result[Value, Bytes] = lit_value(LFloat(\"1.5\"));\n",
        |v| decode_result(v, decode_value),
        Err(()),
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
        data("Bool", vec![]),                          // → None
        Ty::Var("A".to_owned()),                       // → None
        Ty::Substrate("file".to_owned()),              // → None
        Ty::Fn(Box::new(bin(8)), Box::new(Ty::Bytes)), // → None
    ];
    for (i, t) in cases.iter().enumerate() {
        let want = ty_to_repr(t);
        assert_l1_marshal(
            &format!("ty_to_repr_{i}"),
            &format!(
                "fn main() => Option[Repr] = ty_to_repr({});\n",
                encode_ty(t)
            ),
            |v| decode_option(v, decode_repr),
            want,
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// ty_to_field_ty_ref (LIVE — elab::ty_to_field_ty_ref): Data(∅)→FtData, Fn→FtFn, repr→FtRepr, None else.
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn ty_to_field_ty_ref_cases() {
    let cases: Vec<Ty> = vec![
        data("Bool", vec![]),                          // → FtData
        data("List", vec![bin(8)]),                    // generic Data → None
        Ty::Var("A".to_owned()),                       // → None
        Ty::Substrate("file".to_owned()),              // → None
        bin(8),                                        // → FtRepr(RBinary(8))
        Ty::Bytes,                                     // → FtRepr(RBytes)
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
        assert_l1_marshal(
            &format!("ty_to_field_ty_ref_{i}"),
            &format!(
                "fn main() => Option[FieldTyRef] = ty_to_field_ty_ref({});\n",
                encode_ty(t)
            ),
            |v| decode_option(v, decode_field_ty_ref),
            want,
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
        data("Bool", vec![]),                          // → FsData
        data("List", vec![bin(8)]),                    // generic → None
        Ty::Var("A".to_owned()),                       // → None
        Ty::Substrate("file".to_owned()),              // → None
        Ty::Fn(Box::new(bin(8)), Box::new(Ty::Bytes)), // → FsFn
        // Fn with an unresolvable leaf → None.
        Ty::Fn(Box::new(bin(8)), Box::new(Ty::Var("R".to_owned()))),
    ];
    for (i, t) in cases.iter().enumerate() {
        let want = field_spec(t);
        assert_l1_marshal(
            &format!("field_spec_{i}"),
            &format!(
                "fn main() => Option[FieldSpec] = field_spec({});\n",
                encode_ty(t)
            ),
            |v| decode_option(v, decode_field_spec),
            want,
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// policy_name_preimage (LIVE — elab::policy_name_preimage): the domain-separated preimage
// (`policy-name.v0:<dotted>`). The BLAKE3 hashing step is DEFERRED (FLAG-semcore-27). The oracle
// returns a `String`; the port returns `Bytes` — decoded to a `String` and compared.
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
        assert_l1_marshal(
            &format!("policy_name_preimage_{i}"),
            &format!(
                "fn main() => Bytes = policy_name_preimage({});\n",
                encode_path(p)
            ),
            decode_string,
            want,
        );
    }
}
