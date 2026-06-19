//! M-302 — interp↔**native** differential testing (NFR-7; VR-4; RR-12; phase-3.md §2 Batch J).
//!
//! Extends the M-151 differential (`differential.rs`, interp vs the `aot::run` env-machine) to the
//! **genuinely compiled** path: each bit-subset program is run under the M-110 reference interpreter
//! *and* `mycelium_mlir::compile_and_run` (LLVM IR → `llc` → `clang` → native → read-back), and the
//! pair must be **observably equivalent** (`repr + payload + guarantee`) and **validate through the
//! single shared M-210 checker** (`ObservationalEquiv`). A deliberately divergent pair must be
//! caught — so a passing differential is meaningful, not vacuous.
//!
//! The compiled path needs `llc`/`clang`; where they are absent `compile_and_run` returns
//! `AotError::ToolchainMissing` and the test **skips** (the house "skip gracefully" idiom), exactly
//! as the proofs/api/supply-chain checks do — never a false failure.
//!
//! M-373 (Increment-1): extends coverage to the `Construct`/`Match` data fragment — non-recursive,
//! bounded, stack-alloca lowering (DN-15 §4.1 / RFC-0004 §11.2). Guarantee tag: `Declared`
//! (hand-written IR lowering; the differential is empirical evidence, not a proof — VR-5). The
//! `App`/`Lam`/`Fix`/`FixGroup` nodes must still return explicit `AotError::UnsupportedNode`.

use mycelium_cert::{check, CheckVerdict, Evidence, RefinementRelation};
use mycelium_core::{
    Alt, CtorSpec, DataRegistry, DeclSpec, FieldSpec, GuaranteeStrength, Meta, Node, Payload,
    Provenance, Repr, Trit, Value,
};
use mycelium_interp::{IdentitySwapEngine, Interpreter, PrimRegistry};
use mycelium_mlir::AotError;
use mycelium_numerics::Certificate;
use std::collections::BTreeMap;

fn byte(bits: [bool; 8]) -> Value {
    Value::new(
        Repr::Binary { width: 8 },
        Payload::Bits(bits.to_vec()),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

fn tern(trits: Vec<Trit>) -> Value {
    let m = trits.len() as u32;
    Value::new(
        Repr::Ternary { trits: m },
        Payload::Trits(trits),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

const A: [bool; 8] = [true, false, true, true, false, false, true, false];
const B: [bool; 8] = [false, false, true, false, true, false, true, true];
const ONES: [bool; 8] = [true; 8];

/// The bit/trit-subset corpus: straight-line bit logic + balanced-ternary `neg` the direct-LLVM
/// backend lowers (no swaps, no trit *arithmetic* yet — those are out of subset and tested for
/// refusal in the unit tests). A small *deterministic* set of programs, not a statistical sample.
fn bit_corpus() -> Vec<Node> {
    let cst = |bits: [bool; 8]| Node::Const(byte(bits));
    vec![
        // bare constant
        cst(A),
        // core.id (passthrough)
        Node::Op {
            prim: "core.id".into(),
            args: vec![cst(A)],
        },
        // let / var alias
        Node::Let {
            id: "a".into(),
            bound: Box::new(cst(A)),
            body: Box::new(Node::Var("a".into())),
        },
        // each bit op
        Node::Op {
            prim: "bit.not".into(),
            args: vec![cst(A)],
        },
        Node::Op {
            prim: "bit.and".into(),
            args: vec![cst(A), cst(B)],
        },
        Node::Op {
            prim: "bit.or".into(),
            args: vec![cst(A), cst(B)],
        },
        Node::Op {
            prim: "bit.xor".into(),
            args: vec![cst(A), cst(ONES)], // xor with all-ones == complement
        },
        // nested: not(a xor b) through a let
        Node::Let {
            id: "x".into(),
            bound: Box::new(Node::Op {
                prim: "bit.xor".into(),
                args: vec![cst(A), cst(B)],
            }),
            body: Box::new(Node::Op {
                prim: "bit.not".into(),
                args: vec![Node::Var("x".into())],
            }),
        },
        // M-301 trit slice: balanced-ternary negation (a Ternary lane, end-to-end).
        Node::Op {
            prim: "trit.neg".into(),
            args: vec![Node::Const(tern(vec![
                Trit::Pos,
                Trit::Zero,
                Trit::Neg,
                Trit::Pos,
            ]))],
        },
        // trit.neg through a let / core.id passthrough on a ternary value.
        Node::Let {
            id: "t".into(),
            bound: Box::new(Node::Const(tern(vec![Trit::Neg, Trit::Neg, Trit::Pos]))),
            body: Box::new(Node::Op {
                prim: "core.id".into(),
                args: vec![Node::Op {
                    prim: "trit.neg".into(),
                    args: vec![Node::Var("t".into())],
                }],
            }),
        },
        // M-301 trit *carry* arithmetic (in range, so no overflow): add 5+4=9, sub 9-4=5, mul 2*3=6
        // over 3 trits. These exercise the ripple-carry / shifted-accumulate codegen end-to-end.
        Node::Op {
            prim: "trit.add".into(),
            args: vec![
                Node::Const(tern(vec![Trit::Pos, Trit::Neg, Trit::Neg])),
                Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Pos])),
            ],
        },
        Node::Op {
            prim: "trit.sub".into(),
            args: vec![
                Node::Const(tern(vec![Trit::Pos, Trit::Zero, Trit::Zero])),
                Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Pos])),
            ],
        },
        Node::Op {
            prim: "trit.mul".into(),
            args: vec![
                Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Neg])),
                Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Zero])),
            ],
        },
        // nested arithmetic through a let: (5 + 4) - 4 = 5, mixing the adder and subtractor.
        Node::Let {
            id: "s".into(),
            bound: Box::new(Node::Op {
                prim: "trit.add".into(),
                args: vec![
                    Node::Const(tern(vec![Trit::Pos, Trit::Neg, Trit::Neg])),
                    Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Pos])),
                ],
            }),
            body: Box::new(Node::Op {
                prim: "trit.sub".into(),
                args: vec![
                    Node::Var("s".into()),
                    Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Pos])),
                ],
            }),
        },
    ]
}

type Observable<'a> = (&'a Repr, &'a Payload, GuaranteeStrength);

fn observable(v: &Value) -> Observable<'_> {
    (v.repr(), v.payload(), v.meta().guarantee())
}

/// Run a program through the interpreter; the bit subset uses no swaps, so an identity swap engine
/// suffices on the reference side.
fn interp_eval(node: &Node) -> Value {
    Interpreter::new(PrimRegistry::with_builtins(), Box::new(IdentitySwapEngine))
        .eval(node)
        .expect("interpreter must evaluate the bit-subset corpus")
}

#[test]
fn interp_and_native_are_observably_equivalent_on_the_bit_corpus() {
    for (i, node) in bit_corpus().iter().enumerate() {
        let native = match mycelium_mlir::compile_and_run(node) {
            Ok(v) => v,
            // Environment skip: no native toolchain → cannot run the compiled path here.
            Err(AotError::ToolchainMissing(_)) => return,
            Err(e) => panic!("program #{i}: native path errored: {e}"),
        };
        let interp = interp_eval(node);
        // Mutant-witness: if emit_op used the wrong LLVM instruction (e.g. `or` for `bit.and`), the
        // native payload would diverge from the interpreter's and this assertion would fail.
        assert_eq!(
            observable(&interp),
            observable(&native),
            "program #{i} diverged: interp {:?} vs native {:?}",
            interp.payload(),
            native.payload()
        );
        // M-210: the same pair validates through the single shared TV checker.
        assert_eq!(
            check(
                &interp,
                &native,
                RefinementRelation::ObservationalEquiv,
                Certificate::exact(),
                &Evidence::Observational,
            ),
            CheckVerdict::Validated {
                strength: GuaranteeStrength::Exact
            },
            "program #{i}: the shared checker must validate the interp↔native pair"
        );
    }
}

/// M-301 overflow parity: a fixed-width balanced-ternary result out of range must be **refused** by
/// *both* the interpreter (`EvalError::Overflow`) and the native path (`AotError::Overflow`) — never
/// a silent wrap on either side (SC-3/G2). 4 + 4 = 8 overflows the 2-trit range (max magnitude 4).
#[test]
fn interp_and_native_agree_on_overflow_refusal() {
    let overflow = Node::Op {
        prim: "trit.add".into(),
        args: vec![
            Node::Const(tern(vec![Trit::Pos, Trit::Pos])),
            Node::Const(tern(vec![Trit::Pos, Trit::Pos])),
        ],
    };
    // The interpreter refuses with an explicit overflow.
    let interp_err = Interpreter::new(PrimRegistry::with_builtins(), Box::new(IdentitySwapEngine))
        .eval(&overflow);
    assert!(
        matches!(interp_err, Err(mycelium_interp::EvalError::Overflow { .. })),
        "interpreter must refuse the out-of-range sum, got {interp_err:?}"
    );
    // The native path computes the overflow at runtime and refuses through the read-back protocol.
    match mycelium_mlir::compile_and_run(&overflow) {
        Ok(v) => panic!(
            "native path silently wrapped the overflow: {:?}",
            v.payload()
        ),
        Err(AotError::Overflow(_)) => { /* expected — parity with the interpreter */ }
        Err(AotError::ToolchainMissing(_)) => { /* environment skip */ }
        Err(e) => panic!("native path errored unexpectedly: {e}"),
    }
}

/// Sanity: the compiled path actually discriminates — two different bit programs are NOT observably
/// equal and the shared checker reports the divergence explicitly (never a silent pass). So a passing
/// differential above is meaningful, not vacuous.
#[test]
fn native_differential_distinguishes_different_programs() {
    let not_a = Node::Op {
        prim: "bit.not".into(),
        args: vec![Node::Const(byte(A))],
    };
    let id_a = Node::Op {
        prim: "core.id".into(),
        args: vec![Node::Const(byte(A))],
    };
    let (x, y) = match (
        mycelium_mlir::compile_and_run(&not_a),
        mycelium_mlir::compile_and_run(&id_a),
    ) {
        (Ok(x), Ok(y)) => (x, y),
        (Err(AotError::ToolchainMissing(_)), _) | (_, Err(AotError::ToolchainMissing(_))) => return,
        (x, y) => panic!("native path errored: {x:?} / {y:?}"),
    };
    // not(A) != id(A) for a non-self-complementary A.
    assert_ne!(observable(&x), observable(&y));
    let verdict = check(
        &x,
        &y,
        RefinementRelation::ObservationalEquiv,
        Certificate::exact(),
        &Evidence::Observational,
    );
    assert!(
        matches!(verdict, CheckVerdict::NotValidated { .. }),
        "the checker must reject the genuinely divergent native pair, got {verdict:?}"
    );
}

// ─── M-373: Construct / Match data-fragment differential (Increment-1) ────────────────────────

/// Build the shared `DataRegistry` for the data corpus: `type Box = Mk(Binary{8})` (a single
/// constructor wrapping one 8-bit field). Non-recursive — no `FieldSpec::Data` back-reference —
/// so this is firmly within the DN-15 §4.1 Increment-1 subset.
fn box_registry() -> DataRegistry {
    let mut specs = BTreeMap::new();
    specs.insert(
        "Box".to_owned(),
        DeclSpec {
            ctors: vec![CtorSpec {
                fields: vec![FieldSpec::Repr(Repr::Binary { width: 8 })],
            }],
        },
    );
    DataRegistry::build(&specs).expect("Box registry must build")
}

/// Build the shared `DataRegistry` for a two-constructor type: `type Color = Red | Blue`.
/// Both constructors carry no fields — the tag alone is the payload. Used to exercise the
/// `switch i64` dispatch with two arms, one of which produces `A` and the other `B`.
fn color_registry() -> DataRegistry {
    let mut specs = BTreeMap::new();
    specs.insert(
        "Color".to_owned(),
        DeclSpec {
            ctors: vec![
                CtorSpec { fields: vec![] }, // Red  (tag 0)
                CtorSpec { fields: vec![] }, // Blue (tag 1)
            ],
        },
    );
    DataRegistry::build(&specs).expect("Color registry must build")
}

/// The data-fragment corpus (M-373 Increment-1): non-recursive `Construct`/`Match` programs whose
/// final result is a repr lane (bit vector). Each program is valid under both the interpreter
/// (`Interpreter::eval`) and the LLVM data-fragment path (`compile_and_run`).
///
/// Guarantee: `Declared` — hand-written IR lowering, empirically validated by the differential
/// (VR-5: never upgraded to `Proven` without a checked proof).
fn data_corpus() -> Vec<Node> {
    let reg = box_registry();
    let col = color_registry();
    let mk_box = |bits: [bool; 8]| Node::Construct {
        ctor: reg.ctor_ref("Box", 0).unwrap(),
        args: vec![Node::Const(byte(bits))],
    };
    let red = || Node::Construct {
        ctor: col.ctor_ref("Color", 0).unwrap(),
        args: vec![],
    };
    let blue = || Node::Construct {
        ctor: col.ctor_ref("Color", 1).unwrap(),
        args: vec![],
    };

    vec![
        // 1. Construct Box(A), match to extract the inner field b → return b unchanged.
        //    Tests: stack alloca for a 1-field type; tag-load + switch dispatch; field-load + phi.
        Node::Match {
            scrutinee: Box::new(mk_box(A)),
            alts: vec![Alt::Ctor {
                ctor: reg.ctor_ref("Box", 0).unwrap(),
                binders: vec!["b".to_owned()],
                body: Node::Var("b".to_owned()),
            }],
            default: None,
        },
        // 2. Construct Box(A), match and apply bit.not to the extracted field.
        //    Tests: bit op in a match arm body; arm body uses a binder (not just a constant).
        Node::Match {
            scrutinee: Box::new(mk_box(A)),
            alts: vec![Alt::Ctor {
                ctor: reg.ctor_ref("Box", 0).unwrap(),
                binders: vec!["b".to_owned()],
                body: Node::Op {
                    prim: "bit.not".into(),
                    args: vec![Node::Var("b".to_owned())],
                },
            }],
            default: None,
        },
        // 3. Let-bound Construct, then match. Tests that a Construct result in the env (Datum)
        //    can be looked up as the scrutinee of a later Match — the full env-lookup path.
        Node::Let {
            id: "box_a".into(),
            bound: Box::new(mk_box(A)),
            body: Box::new(Node::Match {
                scrutinee: Box::new(Node::Var("box_a".into())),
                alts: vec![Alt::Ctor {
                    ctor: reg.ctor_ref("Box", 0).unwrap(),
                    binders: vec!["b".to_owned()],
                    body: Node::Op {
                        prim: "bit.and".into(),
                        args: vec![Node::Var("b".to_owned()), Node::Const(byte(B))],
                    },
                }],
                default: None,
            }),
        },
        // 4. Two-constructor Color type: match Red → return A; match Blue → return B.
        //    Tests the switch with two real arms (the phi merge collects two (label, Lane) pairs).
        Node::Match {
            scrutinee: Box::new(red()),
            alts: vec![
                Alt::Ctor {
                    ctor: col.ctor_ref("Color", 0).unwrap(), // Red
                    binders: vec![],
                    body: Node::Const(byte(A)),
                },
                Alt::Ctor {
                    ctor: col.ctor_ref("Color", 1).unwrap(), // Blue
                    binders: vec![],
                    body: Node::Const(byte(B)),
                },
            ],
            default: None,
        },
        // 5. Same two-constructor Color type but select Blue → return B (mutant-witness that the
        //    switch dispatches on the correct tag, not always on arm 0).
        Node::Match {
            scrutinee: Box::new(blue()),
            alts: vec![
                Alt::Ctor {
                    ctor: col.ctor_ref("Color", 0).unwrap(), // Red
                    binders: vec![],
                    body: Node::Const(byte(A)),
                },
                Alt::Ctor {
                    ctor: col.ctor_ref("Color", 1).unwrap(), // Blue
                    binders: vec![],
                    body: Node::Const(byte(B)),
                },
            ],
            default: None,
        },
    ]
}

/// Evaluate a `data_corpus` program through the reference interpreter, returning a repr `Value`.
/// Programs in the corpus always reduce to a repr value (never a datum), so `eval` suffices.
fn interp_eval_core(node: &Node) -> Value {
    Interpreter::new(PrimRegistry::with_builtins(), Box::new(IdentitySwapEngine))
        .eval(node)
        .expect("interpreter must evaluate every data_corpus program to a repr value")
}

/// M-373 Increment-1: interp ↔ native are observably equivalent on the `data_corpus`.
///
/// Guarantee: `Declared` — the differential is empirical evidence (VR-5). The design rationale
/// (stack `alloca`, `switch i64`, `@abort()` default) is in `llvm.rs` §M-373 / DN-15 §4.1.
#[test]
fn interp_and_native_are_observably_equivalent_on_the_data_corpus() {
    for (i, node) in data_corpus().iter().enumerate() {
        let native = match mycelium_mlir::compile_and_run(node) {
            Ok(v) => v,
            // Environment skip: no native toolchain → cannot run the compiled path here.
            Err(AotError::ToolchainMissing(_)) => return,
            Err(e) => panic!("data program #{i}: native path errored: {e}"),
        };
        let interp = interp_eval_core(node);
        // Mutant-witness: if the Construct stored the wrong tag, or Match loaded from the wrong
        // slot, or the phi merge picked the wrong arm, the payloads would diverge here.
        assert_eq!(
            observable(&interp),
            observable(&native),
            "data program #{i} diverged: interp {:?} vs native {:?}",
            interp.payload(),
            native.payload()
        );
        // M-210: the same pair validates through the single shared TV checker.
        assert_eq!(
            check(
                &interp,
                &native,
                RefinementRelation::ObservationalEquiv,
                Certificate::exact(),
                &Evidence::Observational,
            ),
            CheckVerdict::Validated {
                strength: GuaranteeStrength::Exact
            },
            "data program #{i}: the shared checker must validate the interp↔native pair"
        );
    }
}

/// M-373 refusal parity: `App`/`Lam`/`Fix`/`FixGroup` nodes must still return an explicit
/// `AotError::UnsupportedNode` on the native path — never a panic, never silent UB (G2).
/// Verifies that the refusal split (Construct/Match pulled out of the catch-all) did not
/// accidentally silence the remaining unsupported nodes.
#[test]
fn lam_and_fix_are_still_explicitly_refused_by_the_native_path() {
    // A bare Lam — not callable without App, but sufficient to test the refusal path.
    let lam = Node::Lam {
        param: "x".to_owned(),
        body: Box::new(Node::Var("x".to_owned())),
    };
    // A Fix node (self-referential recursion — not in the bounded non-recursive subset).
    let fix = Node::Fix {
        name: "f".to_owned(),
        body: Box::new(Node::Var("f".to_owned())),
    };
    for (label, node) in [("Lam", &lam), ("Fix", &fix)] {
        match mycelium_mlir::compile_and_run(node) {
            Err(AotError::UnsupportedNode(_)) => { /* expected explicit refusal */ }
            Err(AotError::ToolchainMissing(_)) => { /* environment skip */ }
            Ok(v) => panic!(
                "{label} must be refused; native path returned {:?}",
                v.payload()
            ),
            Err(e) => panic!("{label} errored with an unexpected variant: {e}"),
        }
    }
}
