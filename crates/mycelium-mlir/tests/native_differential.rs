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
use mycelium_core::{GuaranteeStrength, Meta, Node, Payload, Provenance, Repr, Value};
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

const A: [bool; 8] = [true, false, true, true, false, false, true, false];
const B: [bool; 8] = [false, false, true, false, true, false, true, true];
const ONES: [bool; 8] = [true; 8];

/// The **bit-subset** corpus: straight-line bit logic the direct-LLVM backend lowers (no swaps, no
/// trits — those are out of subset and tested for refusal in the unit tests). A small *deterministic*
/// set of programs, not a statistical sample.
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
