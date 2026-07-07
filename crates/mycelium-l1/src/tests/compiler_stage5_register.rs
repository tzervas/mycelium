//! M-740 Stage 5 (M-1013 STEP 3, PR-1; DN-26 §7.3 / §10.2) — the self-hosted `compiler.semcore`
//! port of checkty.rs's leaf-most **register-family** unit: `resolve_ctors` (+ its `first_duplicate`
//! dup-name helper), the LIVE-ORACLE marshalling differential for the constructor-resolution seam.
//!
//! Helpers ported into `lib/compiler/semcore.myc` and gated here:
//!   * `first_duplicate` (checkty.rs) — the first value appearing more than once, left to right.
//!   * `resolve_ctors` (checkty.rs) — resolve every surface `Ctor`'s field `TypeRef`s (the decl's
//!     type params in scope) into checked `CtorInfo`s, refusing a duplicate constructor name.
//!
//! **Differential method — harness MARSHALLING (DN-26 §10.2).** Each case runs the REAL Rust
//! `checkty::{resolve_ctors, first_duplicate}` oracle on a fixture, producing a genuine
//! `Result<Vec<CtorInfo>, _>` / `Option<&String>`. It then evaluates the `.myc` port *directly* (the
//! driver's `main` returns the mirror value) and DECODES that `L1Value` mirror ADT back into the real
//! Rust type (`decode_ty`/`decode_ctor_info`/`decode_data_info` — the never-silent inverse of the
//! mirror constructors, built on the shared `marshal_support` primitives). The two independently-
//! produced values are compared with **Rust's own trusted derived `==`**. A mis-lowering diverges the
//! decoded value from the oracle; `marshal_discriminates` proves each new decoder arm reads every
//! dimension it claims to (the migrated non-vacuity discipline). `Err` messages differ across the two
//! productions, so `decode_result`/`want.map_err(|_| ())` normalize both to `()` (any `Err` == any
//! `Err`; only the `Ok` payload is a meaningful differential).

use crate::ast::{BaseType, Ctor, Scalar, Sparsity, TypeDecl, TypeRef, Vis, WidthRef};
use crate::checkty::{first_duplicate, resolve_ctors, CtorInfo, DataInfo, Ty, Width};
use crate::eval::L1Value;
use crate::tests::marshal_support::*;
use std::collections::BTreeMap;

// ── L1Value → checkty decoders (register-family output types; the marshalling inverse) ──────────────

fn decode_scalar(v: &L1Value) -> Scalar {
    match expect_data(v, "Scalar").0 {
        "SF16" => Scalar::F16,
        "SBf16" => Scalar::Bf16,
        "SF32" => Scalar::F32,
        "SF64" => Scalar::F64,
        c => panic!("marshal decode_scalar: unexpected ctor {c}"),
    }
}

fn decode_sparsity(v: &L1Value) -> Sparsity {
    let (ctor, fields) = expect_data(v, "Sparsity");
    match ctor {
        "SpDense" => Sparsity::Dense,
        "SpSparse" => Sparsity::Sparse(decode_u32(&fields[0])),
        c => panic!("marshal decode_sparsity: unexpected ctor {c}"),
    }
}

fn decode_width(v: &L1Value) -> Width {
    let (ctor, fields) = expect_data(v, "Width");
    match ctor {
        "WdLit" => Width::Lit(decode_u32(&fields[0])),
        "WdVar" => Width::Var(decode_string(&fields[0])),
        c => panic!("marshal decode_width: unexpected ctor {c}"),
    }
}

/// The checked `Ty` mirror (all 11 variants) → `checkty::Ty`. Recursive on `Data`/`Seq`/`Fn`.
fn decode_ty(v: &L1Value) -> Ty {
    let (ctor, fields) = expect_data(v, "Ty");
    match ctor {
        "TyBinary" => Ty::Binary(decode_width(&fields[0])),
        "TyTernary" => Ty::Ternary(decode_width(&fields[0])),
        "TyDense" => Ty::Dense(decode_u32(&fields[0]), decode_scalar(&fields[1])),
        "TyVsa" => Ty::Vsa {
            model: decode_string(&fields[0]),
            dim: decode_u32(&fields[1]),
            sparsity: decode_sparsity(&fields[2]),
        },
        "TyData" => Ty::Data(decode_string(&fields[0]), decode_vec(&fields[1], decode_ty)),
        "TySubstrate" => Ty::Substrate(decode_string(&fields[0])),
        "TySeq" => Ty::Seq(Box::new(decode_ty(&fields[0])), decode_u32(&fields[1])),
        "TyBytes" => Ty::Bytes,
        "TyFloat" => Ty::Float,
        "TyVar" => Ty::Var(decode_string(&fields[0])),
        "TyFn" => Ty::Fn(
            Box::new(decode_ty(&fields[0])),
            Box::new(decode_ty(&fields[1])),
        ),
        c => panic!("marshal decode_ty: unexpected ctor {c}"),
    }
}

/// `CI(name, fields)` → `checkty::CtorInfo`.
fn decode_ctor_info(v: &L1Value) -> CtorInfo {
    let (ctor, fields) = expect_data(v, "CtorInfo");
    match ctor {
        "CI" => CtorInfo {
            name: decode_string(&fields[0]),
            fields: decode_vec(&fields[1], decode_ty),
        },
        c => panic!("marshal decode_ctor_info: unexpected ctor {c}"),
    }
}

/// `DI(name, params, ctors)` → `checkty::DataInfo`. (`resolve_ctors` returns `Vec<CtorInfo>`; this
/// decoder is exercised by `marshal_discriminates` and pairs with `encode_data_info` on the input side
/// — it is the register-family's data-type mirror, ready for the later `register_types` increment.)
fn decode_data_info(v: &L1Value) -> DataInfo {
    let (ctor, fields) = expect_data(v, "DataInfo");
    match ctor {
        "DI" => DataInfo {
            name: decode_string(&fields[0]),
            params: decode_vec(&fields[1], decode_string),
            ctors: decode_vec(&fields[2], decode_ctor_info),
        },
        c => panic!("marshal decode_data_info: unexpected ctor {c}"),
    }
}

// ── Rust → `.myc` fixture encoders (register-family INPUT types; built on shared primitives) ─────────

fn encode_vis(v: Vis) -> &'static str {
    match v {
        Vis::Private => "Private",
        Vis::Pub => "Pub",
    }
}

fn encode_names(names: &[String]) -> String {
    let mut s = String::from("Nil");
    for n in names.iter().rev() {
        s = format!("Cons({}, {})", encode_bytes(n), s);
    }
    s
}

fn encode_ctor(c: &Ctor) -> String {
    format!(
        "Ct({}, {})",
        encode_bytes(&c.name),
        encode_typeref_list(&c.fields)
    )
}

fn encode_ctor_list(cs: &[Ctor]) -> String {
    let mut s = String::from("Nil");
    for c in cs.iter().rev() {
        s = format!("Cons({}, {})", encode_ctor(c), s);
    }
    s
}

fn encode_type_decl(td: &TypeDecl) -> String {
    format!(
        "TD({}, {}, {}, {})",
        encode_vis(td.vis),
        encode_bytes(&td.name),
        encode_names(&td.params),
        encode_ctor_list(&td.ctors)
    )
}

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

// ── small fixture constructors (test bodies stay `assert over a case`) ──────────────────────────────

fn named(name: &str, args: Vec<TypeRef>) -> BaseType {
    BaseType::Named(name.to_owned(), args)
}

fn ctor(name: &str, fields: Vec<TypeRef>) -> Ctor {
    Ctor {
        name: name.to_owned(),
        fields,
    }
}

fn type_decl(name: &str, params: &[&str], ctors: Vec<Ctor>) -> TypeDecl {
    TypeDecl {
        vis: Vis::Private,
        name: name.to_owned(),
        params: params.iter().map(|s| (*s).to_owned()).collect(),
        ctors,
    }
}

/// A registered-type **shell** (empty ctors) — exactly what `register_types` inserts into `types`
/// before `resolve_ctors` runs, so a recursive field reference resolves.
fn shell(name: &str, params: &[&str]) -> DataInfo {
    DataInfo {
        name: name.to_owned(),
        params: params.iter().map(|s| (*s).to_owned()).collect(),
        ctors: vec![],
    }
}

fn types_map(types: &[DataInfo]) -> BTreeMap<String, DataInfo> {
    types.iter().map(|d| (d.name.clone(), d.clone())).collect()
}

// `decode_driver` shorthands (bare mirror-literal round-trips for the non-vacuity gate).
fn dd_ty(expr: &str) -> Ty {
    decode_driver("Ty", expr, decode_ty)
}
fn dd_ci(expr: &str) -> CtorInfo {
    decode_driver("CtorInfo", expr, decode_ctor_info)
}
fn dd_di(expr: &str) -> DataInfo {
    decode_driver("DataInfo", expr, decode_data_info)
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// Decoder non-vacuity: each new decoder arm must DISCRIMINATE on every dimension it reads (M-1013
// STEP 2 convention — decode two mirror literals differing in exactly one dimension, assert `!=`, so a
// decoder that dropped a dimension is caught rather than silently collapsing distinct values).
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn marshal_discriminates() {
    // decode_width (via TyBinary): variant tag, the WdLit u32, the WdVar string.
    assert_ne!(
        dd_ty(&format!("TyBinary(WdLit({}))", encode_u32(8))),
        dd_ty(&format!("TyBinary(WdVar({}))", encode_bytes("N")))
    );
    assert_ne!(
        dd_ty(&format!("TyBinary(WdLit({}))", encode_u32(8))),
        dd_ty(&format!("TyBinary(WdLit({}))", encode_u32(16)))
    );
    assert_ne!(
        dd_ty(&format!("TyBinary(WdVar({}))", encode_bytes("N"))),
        dd_ty(&format!("TyBinary(WdVar({}))", encode_bytes("M")))
    );

    // decode_ty variant tags.
    assert_ne!(
        dd_ty(&format!("TyBinary(WdLit({}))", encode_u32(8))),
        dd_ty(&format!("TyTernary(WdLit({}))", encode_u32(8)))
    );
    assert_ne!(dd_ty("TyBytes"), dd_ty("TyFloat"));
    assert_ne!(
        dd_ty(&format!("TyData({}, Nil)", encode_bytes("A"))),
        dd_ty(&format!("TyVar({})", encode_bytes("A")))
    );
    assert_ne!(
        dd_ty(&format!("TySubstrate({})", encode_bytes("a"))),
        dd_ty(&format!("TyVar({})", encode_bytes("a")))
    );

    // decode_scalar (via TyDense dtype): all four kinds distinct; plus the dim u32.
    assert_ne!(
        dd_ty(&format!("TyDense({}, SF16)", encode_u32(4))),
        dd_ty(&format!("TyDense({}, SBf16)", encode_u32(4)))
    );
    assert_ne!(
        dd_ty(&format!("TyDense({}, SBf16)", encode_u32(4))),
        dd_ty(&format!("TyDense({}, SF32)", encode_u32(4)))
    );
    assert_ne!(
        dd_ty(&format!("TyDense({}, SF32)", encode_u32(4))),
        dd_ty(&format!("TyDense({}, SF64)", encode_u32(4)))
    );
    assert_ne!(
        dd_ty(&format!("TyDense({}, SF16)", encode_u32(4))),
        dd_ty(&format!("TyDense({}, SF16)", encode_u32(8)))
    );

    // decode_sparsity + TyVsa fields (model, dim, sparsity).
    assert_ne!(
        dd_ty(&format!(
            "TyVsa({}, {}, SpDense)",
            encode_bytes("A"),
            encode_u32(4)
        )),
        dd_ty(&format!(
            "TyVsa({}, {}, SpSparse({}))",
            encode_bytes("A"),
            encode_u32(4),
            encode_u32(8)
        ))
    );
    assert_ne!(
        dd_ty(&format!(
            "TyVsa({}, {}, SpSparse({}))",
            encode_bytes("A"),
            encode_u32(4),
            encode_u32(8)
        )),
        dd_ty(&format!(
            "TyVsa({}, {}, SpSparse({}))",
            encode_bytes("A"),
            encode_u32(4),
            encode_u32(16)
        ))
    );
    assert_ne!(
        dd_ty(&format!(
            "TyVsa({}, {}, SpDense)",
            encode_bytes("A"),
            encode_u32(4)
        )),
        dd_ty(&format!(
            "TyVsa({}, {}, SpDense)",
            encode_bytes("B"),
            encode_u32(4)
        ))
    );
    assert_ne!(
        dd_ty(&format!(
            "TyVsa({}, {}, SpDense)",
            encode_bytes("A"),
            encode_u32(4)
        )),
        dd_ty(&format!(
            "TyVsa({}, {}, SpDense)",
            encode_bytes("A"),
            encode_u32(8)
        ))
    );

    // decode_ty TyData name + fields; TySeq elem + len; TyVar/TySubstrate string; TyFn param + ret.
    assert_ne!(
        dd_ty(&format!("TyData({}, Nil)", encode_bytes("A"))),
        dd_ty(&format!("TyData({}, Nil)", encode_bytes("B")))
    );
    assert_ne!(
        dd_ty(&format!("TyData({}, Nil)", encode_bytes("A"))),
        dd_ty(&format!(
            "TyData({}, Cons(TyBytes, Nil))",
            encode_bytes("A")
        ))
    );
    assert_ne!(
        dd_ty(&format!(
            "TyData({}, Cons(TyBytes, Nil))",
            encode_bytes("A")
        )),
        dd_ty(&format!(
            "TyData({}, Cons(TyFloat, Nil))",
            encode_bytes("A")
        ))
    );
    assert_ne!(
        dd_ty(&format!("TySeq(TyBytes, {})", encode_u32(2))),
        dd_ty(&format!("TySeq(TyFloat, {})", encode_u32(2)))
    );
    assert_ne!(
        dd_ty(&format!("TySeq(TyBytes, {})", encode_u32(2))),
        dd_ty(&format!("TySeq(TyBytes, {})", encode_u32(3)))
    );
    assert_ne!(
        dd_ty(&format!("TyVar({})", encode_bytes("A"))),
        dd_ty(&format!("TyVar({})", encode_bytes("B")))
    );
    assert_ne!(
        dd_ty(&format!("TySubstrate({})", encode_bytes("a"))),
        dd_ty(&format!("TySubstrate({})", encode_bytes("b")))
    );
    assert_ne!(
        dd_ty("TyFn(TyBytes, TyFloat)"),
        dd_ty("TyFn(TyFloat, TyFloat)")
    );
    assert_ne!(
        dd_ty("TyFn(TyBytes, TyFloat)"),
        dd_ty("TyFn(TyBytes, TyBytes)")
    );

    // decode_ctor_info (CI): name + fields.
    assert_ne!(
        dd_ci(&format!("CI({}, Nil)", encode_bytes("A"))),
        dd_ci(&format!("CI({}, Nil)", encode_bytes("B")))
    );
    assert_ne!(
        dd_ci(&format!("CI({}, Nil)", encode_bytes("A"))),
        dd_ci(&format!("CI({}, Cons(TyBytes, Nil))", encode_bytes("A")))
    );

    // decode_data_info (DI): name + params + ctors.
    assert_ne!(
        dd_di(&format!("DI({}, Nil, Nil)", encode_bytes("A"))),
        dd_di(&format!("DI({}, Nil, Nil)", encode_bytes("B")))
    );
    assert_ne!(
        dd_di(&format!("DI({}, Nil, Nil)", encode_bytes("A"))),
        dd_di(&format!(
            "DI({}, Cons({}, Nil), Nil)",
            encode_bytes("A"),
            encode_bytes("P")
        ))
    );
    assert_ne!(
        dd_di(&format!("DI({}, Nil, Nil)", encode_bytes("A"))),
        dd_di(&format!(
            "DI({}, Nil, Cons(CI({}, Nil), Nil))",
            encode_bytes("A"),
            encode_bytes("C")
        ))
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// first_duplicate (LIVE — checkty::first_duplicate): None + the first-repeat cases (left to right).
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn first_duplicate_cases() {
    let cases: Vec<Vec<&str>> = vec![
        vec![],
        vec!["a"],
        vec!["a", "b", "c"],
        vec!["a", "b", "a"],      // → Some("a")
        vec!["x", "x"],           // → Some("x")
        vec!["a", "b", "b", "a"], // → Some("b") (first repeat)
    ];
    for (i, xs) in cases.iter().enumerate() {
        let owned: Vec<String> = xs.iter().map(|s| (*s).to_owned()).collect();
        let want: Option<String> = first_duplicate(&owned).cloned();
        assert_l1_marshal(
            &format!("first_duplicate_{i}"),
            &format!(
                "fn main() => Option[Bytes] = first_duplicate({});\n",
                encode_names(&owned)
            ),
            |v| decode_option(v, decode_string),
            want,
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// resolve_ctors (LIVE — checkty::resolve_ctors): monomorphic enum, generic recursive type, repr-typed
// fields, and the two refusals (unknown field type, duplicate ctor). Compared to the live oracle by
// Rust's derived `==` (Err normalized to `()`).
// ─────────────────────────────────────────────────────────────────────────────────────────────────
#[test]
fn resolve_ctors_cases() {
    let cases: Vec<(&str, Vec<DataInfo>, TypeDecl)> = vec![
        // Monomorphic enum: Color = Red | Green | Blue.
        (
            "mono_enum",
            vec![],
            type_decl(
                "Color",
                &[],
                vec![
                    ctor("Red", vec![]),
                    ctor("Green", vec![]),
                    ctor("Blue", vec![]),
                ],
            ),
        ),
        // Generic recursive: List[A] = Nil | Cons(A, List[A]). The `List` shell (empty ctors) is in
        // `types`, exactly as `register_types` inserts it before calling `resolve_ctors`.
        (
            "generic_recursive",
            vec![shell("List", &["A"])],
            type_decl(
                "List",
                &["A"],
                vec![
                    ctor("Nil", vec![]),
                    ctor(
                        "Cons",
                        vec![
                            tref(named("A", vec![])),
                            tref(named("List", vec![tref(named("A", vec![]))])),
                        ],
                    ),
                ],
            ),
        ),
        // Repr-typed fields: Rec = Mk(Binary{8}, Bytes, Seq{Binary{8}, 4}).
        (
            "repr_fields",
            vec![],
            type_decl(
                "Rec",
                &[],
                vec![ctor(
                    "Mk",
                    vec![
                        tref(BaseType::Binary(WidthRef::Lit(8))),
                        tref(BaseType::Bytes),
                        tref(BaseType::Seq {
                            elem: Box::new(tref(BaseType::Binary(WidthRef::Lit(8)))),
                            len: 4,
                        }),
                    ],
                )],
            ),
        ),
        // Unknown type name in a field → Err (both sides).
        (
            "unknown_field",
            vec![],
            type_decl(
                "Bad",
                &[],
                vec![ctor("Mk", vec![tref(named("Nope", vec![]))])],
            ),
        ),
        // Duplicate constructor → Err (both sides).
        (
            "duplicate_ctor",
            vec![],
            type_decl("Dup", &[], vec![ctor("A", vec![]), ctor("A", vec![])]),
        ),
    ];
    for (label, types, td) in &cases {
        let map = types_map(types);
        let want = resolve_ctors(&map, td).map_err(|_| ());
        assert_l1_marshal(
            &format!("resolve_ctors_{label}"),
            &format!(
                "fn main() => Result[Vec[CtorInfo], Bytes] = resolve_ctors({}, {});\n",
                encode_data_info_list(types),
                encode_type_decl(td)
            ),
            |v| decode_result(v, |v| decode_vec(v, decode_ctor_info)),
            want,
        );
    }
}
