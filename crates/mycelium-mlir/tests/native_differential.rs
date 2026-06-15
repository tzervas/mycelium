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

use mycelium_cert::{check, CheckVerdict, Evidence, RefinementRelation};
use mycelium_core::{GuaranteeStrength, Meta, Node, Payload, Provenance, Repr, Trit, Value};
use mycelium_interp::{IdentitySwapEngine, Interpreter, PrimRegistry};
use mycelium_mlir::AotError;
use mycelium_numerics::Certificate;

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
