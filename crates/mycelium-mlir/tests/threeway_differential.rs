//! M-602 / M-725 — the **three-way** native differential (NFR-7; VR-4; RR-12; RFC-0029 §7; phase-6).
//!
//! Extends the M-302 interp↔native differential (`native_differential.rs`) to a **third** compiled
//! path: the **real MLIR-dialect** lowering (`dialect::native`, feature `mlir-dialect`; M-601, widened
//! by M-725). For the in-fragment calculus corpus the programs run under
//!
//! 1. the M-110 **reference interpreter** (the trusted base),
//! 2. the **direct-LLVM** backend (`mycelium_mlir::compile_and_run`; `llvm.rs`), and
//! 3. the **MLIR-dialect** backend (`mycelium_mlir::mlir_compile_and_run`; emits `arith`/`func`/`cf`
//!    MLIR, runs `mlir-opt | mlir-translate → clang → native`),
//!
//! and all three must be **observably equivalent** (`repr + payload + guarantee`), each pair
//! **validated through the single shared M-210 checker** (`ObservationalEquiv`). A deliberately
//! divergent lowering on *any* path is caught — so a passing three-way differential is meaningful,
//! not vacuous.
//!
//! **Honest fragment boundary (VR-5/G2).** The MLIR-dialect path covers the element-wise fragment
//! (`core.id`, `bit.not/and/or/xor`, `trit.neg`) **plus** (M-725) the balanced-ternary additive carry
//! chain `trit.add`/`trit.sub` — including its never-silent overflow read-back. The **new boundary**
//! is `trit.mul` (the shifted-accumulate fragment), the data fragment, closures and recursion: each an
//! **explicit refusal** there (`DialectError::Unsupported`) that routes to the direct-LLVM/interp path.
//! This test asserts BOTH: the in-fragment corpus (element-wise + trit add/sub, in-range *and*
//! overflowing) is three-way equivalent — on the result *and* the overflow refusal — AND the
//! out-of-fragment corpus (`trit.mul`, …) is explicitly refused by the MLIR path while still
//! interp ≡ direct-LLVM (so coverage is honest, never silently claimed).
//!
//! **Toolchain skip.** Both compiled paths need their tools (`llc`/`clang` for direct-LLVM;
//! `mlir-opt`/`mlir-translate`/`clang` for the dialect path). Where a tool is absent the path returns
//! a `ToolchainMissing` and the test **skips** that path (the house "skip gracefully" idiom) — never
//! a false failure.
//!
//! **Guarantee:** `Empirical` — the differential is empirical evidence the MLIR lowering agrees with
//! the trusted interpreter over the corpus; never upgraded to `Proven` without a checked proof (VR-5).

#![cfg(feature = "mlir-dialect")]

use mycelium_cert::{check, CheckVerdict, Evidence, RefinementRelation};
use mycelium_core::{GuaranteeStrength, Meta, Node, Payload, Provenance, Repr, Trit, Value};
use mycelium_interp::{IdentitySwapEngine, Interpreter, PrimRegistry};
use mycelium_mlir::{AotError, DialectError};
use mycelium_numerics::Certificate;

// ─── shared helpers (local; the `common` module's helpers are a superset we don't fully need) ──

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

type Observable<'a> = (&'a Repr, &'a Payload, GuaranteeStrength);
fn observable(v: &Value) -> Observable<'_> {
    (v.repr(), v.payload(), v.meta().guarantee())
}

fn interp_eval(node: &Node) -> Value {
    Interpreter::new(PrimRegistry::with_builtins(), Box::new(IdentitySwapEngine))
        .eval(node)
        .expect("interpreter must evaluate the element-wise corpus")
}

/// Mean ns/call of the fastest batch of `iters` calls (after one warmup batch). House timing style —
/// no benchmarking dependency (mirrors `xtask::e1::bench`). Used only by the `#[ignore]` perf test.
#[allow(dead_code)]
fn bench(iters: u32, mut f: impl FnMut()) -> f64 {
    for _ in 0..iters {
        f();
    }
    let mut best = f64::INFINITY;
    for _ in 0..5 {
        let t = std::time::Instant::now();
        for _ in 0..iters {
            f();
        }
        #[allow(clippy::cast_precision_loss)]
        let per = t.elapsed().as_nanos() as f64 / f64::from(iters);
        best = best.min(per);
    }
    best
}

/// The **in-fragment** corpus the MLIR-dialect path covers: the element-wise ops (`core.id`,
/// `bit.not/and/or/xor`, `trit.neg`) **plus** (M-725) the additive carry chain `trit.add`/`trit.sub`
/// over `Binary{w}`/`Ternary{m}`, straight-line (through `let`s). All the trit-additive cases here
/// stay **in range** (no overflow) so every path produces a value; the overflow refusal is covered
/// separately by [`overflow_corpus`]. A small deterministic set, not a statistical sample.
fn element_wise_corpus() -> Vec<Node> {
    let cst = |bits: [bool; 8]| Node::Const(byte(bits));
    vec![
        // bare constant
        cst(A),
        // core.id passthrough
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
            args: vec![cst(A), cst(ONES)],
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
        // balanced-ternary negation (a Ternary lane end-to-end)
        Node::Op {
            prim: "trit.neg".into(),
            args: vec![Node::Const(tern(vec![
                Trit::Pos,
                Trit::Zero,
                Trit::Neg,
                Trit::Pos,
            ]))],
        },
        // trit.neg through a let / core.id passthrough on a ternary value
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
        // ── M-725: the additive carry chain, all in-range (no overflow) ──
        // trit.add: 1 + 1 = 2 (= [0,+,-]) in 3 trits (max magnitude 13).
        Node::Op {
            prim: "trit.add".into(),
            args: vec![
                Node::Const(tern(vec![Trit::Zero, Trit::Zero, Trit::Pos])),
                Node::Const(tern(vec![Trit::Zero, Trit::Zero, Trit::Pos])),
            ],
        },
        // trit.add with a multi-trit carry ripple: 7 + (−7) = 0, over 4 trits.
        Node::Op {
            prim: "trit.add".into(),
            args: vec![
                Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Neg, Trit::Pos])),
                Node::Const(tern(vec![Trit::Zero, Trit::Neg, Trit::Pos, Trit::Neg])),
            ],
        },
        // trit.sub: 3 − 1 = 2, over 3 trits.
        Node::Op {
            prim: "trit.sub".into(),
            args: vec![
                Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Zero])),
                Node::Const(tern(vec![Trit::Zero, Trit::Zero, Trit::Pos])),
            ],
        },
        // nested: (a + b) through a let, then negate — a Ternary lane end-to-end with carry.
        Node::Let {
            id: "s".into(),
            bound: Box::new(Node::Op {
                prim: "trit.add".into(),
                args: vec![
                    Node::Const(tern(vec![Trit::Pos, Trit::Zero, Trit::Neg])),
                    Node::Const(tern(vec![Trit::Neg, Trit::Pos, Trit::Pos])),
                ],
            }),
            body: Box::new(Node::Op {
                prim: "trit.neg".into(),
                args: vec![Node::Var("s".into())],
            }),
        },
    ]
}

/// The three-way differential over the element-wise corpus: interp ≡ direct-LLVM ≡ MLIR-dialect,
/// each pair validated through the shared M-210 checker. Skips a path whose toolchain is absent.
#[test]
fn interp_directllvm_mlirdialect_are_three_way_equivalent() {
    let mut ran_mlir = false;
    for (i, node) in element_wise_corpus().iter().enumerate() {
        let interp = interp_eval(node);

        // Path 2: direct-LLVM (skip if llc/clang absent).
        let direct = match mycelium_mlir::compile_and_run(node) {
            Ok(v) => Some(v),
            Err(AotError::ToolchainMissing(_)) => None,
            Err(e) => panic!("program #{i}: direct-LLVM path errored: {e}"),
        };

        // Path 3: MLIR-dialect (skip if mlir-opt/mlir-translate/clang absent).
        let mlir = match mycelium_mlir::mlir_compile_and_run(node) {
            Ok(v) => Some(v),
            Err(DialectError::ToolchainMissing(_)) => None,
            Err(e) => panic!("program #{i}: MLIR-dialect path errored: {e}"),
        };

        if let Some(d) = &direct {
            assert_eq!(
                observable(&interp),
                observable(d),
                "program #{i}: interp vs direct-LLVM diverged"
            );
        }
        if let Some(m) = &mlir {
            ran_mlir = true;
            // Mutant-witness: a wrong arith op (e.g. arith.ori for bit.and) would diverge here.
            assert_eq!(
                observable(&interp),
                observable(m),
                "program #{i}: interp vs MLIR-dialect diverged ({:?} vs {:?})",
                interp.payload(),
                m.payload()
            );
            // M-210: the interp↔MLIR pair validates through the single shared TV checker.
            assert_eq!(
                check(
                    &interp,
                    m,
                    RefinementRelation::ObservationalEquiv,
                    Certificate::exact(),
                    &Evidence::Observational,
                ),
                CheckVerdict::Validated {
                    strength: GuaranteeStrength::Exact
                },
                "program #{i}: the shared checker must validate the interp↔MLIR pair"
            );
        }
        // When BOTH compiled paths ran, the two compiled artifacts must also agree with each other
        // (the third edge of the triangle) — validated through the same checker.
        if let (Some(d), Some(m)) = (&direct, &mlir) {
            assert_eq!(
                observable(d),
                observable(m),
                "program #{i}: direct-LLVM vs MLIR-dialect diverged"
            );
            assert_eq!(
                check(
                    d,
                    m,
                    RefinementRelation::ObservationalEquiv,
                    Certificate::exact(),
                    &Evidence::Observational,
                ),
                CheckVerdict::Validated {
                    strength: GuaranteeStrength::Exact
                },
                "program #{i}: the shared checker must validate the direct-LLVM↔MLIR pair"
            );
        }
    }
    // If the MLIR toolchain was present, we must actually have exercised it on at least one program
    // (guard against a vacuous pass where every program silently skipped).
    if mycelium_mlir::MlirTools::is_available() {
        assert!(
            ran_mlir,
            "MLIR toolchain is available but no program exercised the dialect path — vacuous"
        );
    }
}

/// Sanity: the MLIR-dialect path actually discriminates — two different programs are NOT observably
/// equal and the shared checker reports the divergence (never a vacuous pass). So the equivalence
/// above is meaningful.
#[test]
fn mlir_dialect_distinguishes_different_programs() {
    let not_a = Node::Op {
        prim: "bit.not".into(),
        args: vec![Node::Const(byte(A))],
    };
    let id_a = Node::Op {
        prim: "core.id".into(),
        args: vec![Node::Const(byte(A))],
    };
    let (x, y) = match (
        mycelium_mlir::mlir_compile_and_run(&not_a),
        mycelium_mlir::mlir_compile_and_run(&id_a),
    ) {
        (Ok(x), Ok(y)) => (x, y),
        (Err(DialectError::ToolchainMissing(_)), _)
        | (_, Err(DialectError::ToolchainMissing(_))) => return,
        (x, y) => panic!("MLIR-dialect path errored: {x:?} / {y:?}"),
    };
    assert_ne!(observable(&x), observable(&y), "not(A) != id(A)");
    let verdict = check(
        &x,
        &y,
        RefinementRelation::ObservationalEquiv,
        Certificate::exact(),
        &Evidence::Observational,
    );
    assert!(
        matches!(verdict, CheckVerdict::NotValidated { .. }),
        "the checker must reject the divergent MLIR pair, got {verdict:?}"
    );
}

/// The out-of-fragment corpus: nodes the MLIR-dialect path must **explicitly refuse** (routing to
/// the direct-LLVM/interp path), while interp ≡ direct-LLVM still holds. This proves coverage is
/// honest — the dialect path never silently mis-lowers a node it doesn't support (G2/VR-5).
///
/// **M-725 moved the boundary:** `trit.add`/`trit.sub` are now IN-fragment (they appear in
/// [`element_wise_corpus`] / [`overflow_corpus`]); the new boundary is `trit.mul`, the
/// shifted-accumulate fragment. So these cases exercise the *new* refusal, not the old one.
fn out_of_fragment_corpus() -> Vec<Node> {
    vec![
        // trit *multiply* — the new boundary: refused by the dialect path, lowered by direct-LLVM.
        Node::Op {
            prim: "trit.mul".into(),
            args: vec![
                Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Neg])),
                Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Zero])),
            ],
        },
        Node::Op {
            prim: "trit.mul".into(),
            args: vec![
                Node::Const(tern(vec![Trit::Zero, Trit::Zero, Trit::Pos])),
                Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Pos])),
            ],
        },
        // trit.mul nested behind an in-fragment trit.add — the whole program is refused (the MLIR
        // path refuses the *program*, not just the op) and routed to direct-LLVM/interp.
        Node::Op {
            prim: "trit.add".into(),
            args: vec![
                Node::Op {
                    prim: "trit.mul".into(),
                    args: vec![
                        Node::Const(tern(vec![Trit::Zero, Trit::Zero, Trit::Pos])),
                        Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Zero])),
                    ],
                },
                Node::Const(tern(vec![Trit::Zero, Trit::Zero, Trit::Pos])),
            ],
        },
    ]
}

/// The **overflow** corpus (M-725): in-fragment `trit.add`/`trit.sub` programs whose fixed-width
/// result leaves the `m`-trit range. All three paths must **refuse** non-silently — the interpreter
/// errors (`EvalError::Overflow`), the direct-LLVM path returns `AotError::Overflow`, and the
/// MLIR-dialect path returns `DialectError::Overflow` (the shared sentinel read-back). This is the
/// overflow half of the honest carry boundary — a value is never silently wrapped (SC-3/G2).
fn overflow_corpus() -> Vec<Node> {
    vec![
        // max(2 trits) + max(2 trits) = 4 + 4 = 8, out of the 2-trit range [−4, 4]. ([+,+] = 4.)
        Node::Op {
            prim: "trit.add".into(),
            args: vec![
                Node::Const(tern(vec![Trit::Pos, Trit::Pos])),
                Node::Const(tern(vec![Trit::Pos, Trit::Pos])),
            ],
        },
        // 4 − (−4) = 8, out of the 2-trit range.
        Node::Op {
            prim: "trit.sub".into(),
            args: vec![
                Node::Const(tern(vec![Trit::Pos, Trit::Pos])),
                Node::Const(tern(vec![Trit::Neg, Trit::Neg])),
            ],
        },
    ]
}

#[test]
fn out_of_fragment_nodes_are_refused_by_mlir_but_run_on_direct_llvm() {
    for (i, node) in out_of_fragment_corpus().iter().enumerate() {
        // The MLIR-dialect path refuses explicitly (never silently mis-lowers).
        match mycelium_mlir::mlir_compile_and_run(node) {
            Err(DialectError::Unsupported(_)) => { /* expected explicit refusal */ }
            Err(DialectError::ToolchainMissing(_)) => { /* env skip — still no silent success */ }
            Ok(v) => panic!(
                "out-of-fragment program #{i} must be refused by the MLIR path, got {:?}",
                v.payload()
            ),
            Err(e) => panic!("out-of-fragment program #{i}: unexpected MLIR error: {e}"),
        }
        // …and the direct-LLVM path still agrees with the interpreter (coverage is preserved there).
        let interp = interp_eval(node);
        match mycelium_mlir::compile_and_run(node) {
            Ok(d) => assert_eq!(
                observable(&interp),
                observable(&d),
                "out-of-fragment program #{i}: interp vs direct-LLVM diverged"
            ),
            Err(AotError::ToolchainMissing(_)) => { /* env skip */ }
            Err(e) => panic!("out-of-fragment program #{i}: direct-LLVM errored: {e}"),
        }
    }
}

/// M-725: the **overflow** three-way refusal parity. An in-fragment `trit.add`/`trit.sub` whose
/// result leaves the `m`-trit range must be refused **non-silently by all three paths** — the
/// interpreter errors, and both compiled paths return an explicit `Overflow`. Never a silent wrap on
/// any path (SC-3/G2), so the carry boundary is honest on overflow as well as on value.
#[test]
fn overflowing_trit_arithmetic_is_refused_non_silently_three_ways() {
    for (i, node) in overflow_corpus().iter().enumerate() {
        // Path 1: the reference interpreter must error (not return a wrapped value).
        let interp = Interpreter::new(PrimRegistry::with_builtins(), Box::new(IdentitySwapEngine))
            .eval(node);
        assert!(
            interp.is_err(),
            "overflow program #{i}: interpreter must refuse (error), got {:?}",
            interp.ok().map(|v| v.payload().clone())
        );

        // Path 2: direct-LLVM must return an explicit Overflow (or skip if the toolchain is absent).
        match mycelium_mlir::compile_and_run(node) {
            Err(AotError::Overflow(_)) => { /* expected explicit refusal */ }
            Err(AotError::ToolchainMissing(_)) => { /* env skip — still no silent success */ }
            Ok(v) => panic!(
                "overflow program #{i}: direct-LLVM must refuse, got {:?}",
                v.payload()
            ),
            Err(e) => panic!("overflow program #{i}: unexpected direct-LLVM error: {e}"),
        }

        // Path 3: MLIR-dialect must return an explicit Overflow (the shared sentinel read-back), or
        // skip if its toolchain is absent. NOT Unsupported (these ops are in-fragment now), NOT Ok.
        match mycelium_mlir::mlir_compile_and_run(node) {
            Err(DialectError::Overflow(_)) => { /* expected explicit refusal */ }
            Err(DialectError::ToolchainMissing(_)) => { /* env skip */ }
            Ok(v) => panic!(
                "overflow program #{i}: MLIR-dialect must refuse (Overflow), got {:?}",
                v.payload()
            ),
            Err(e) => panic!("overflow program #{i}: unexpected MLIR error: {e}"),
        }
    }
}

// ─── M-602 E1 speedup: MLIR-dialect native vs interpreter (MEASURED, no pre-written target) ────

/// The E1 perf verdict half (M-602; M-303; NFR-4): a **measured** MLIR-dialect-native-vs-interpreter
/// ratio on the element-wise fragment. Reported as-measured — **no pre-written target** (VR-5); the
/// number is whatever the box produces.
///
/// `#[ignore]` by default (it spawns processes + times — not part of the fast unit gate) and
/// **refuses a debug build** (an unoptimized timing is meaningless, exactly as `xtask e1` refuses).
/// Run with: `cargo test -p mycelium-mlir --features mlir-dialect --release -- --ignored
/// e1_mlir_dialect_speedup_is_measured --nocapture`.
///
/// **Honest caption (printed):** the MLIR-native per-invocation figure is **process-spawn-bound** for
/// this trivial kernel (one `putchar` loop), so the ratio reflects spawn + run vs in-process eval —
/// captioned as such, never sold as raw compute throughput. This is the *AOT-path* E1 number; the
/// in-process compute-throughput E1 numbers (BitNet kernels) live in `xtask e1` §3–§5.
#[test]
#[ignore = "perf measurement: run with --release --ignored --nocapture"]
fn e1_mlir_dialect_speedup_is_measured() {
    // Refuse a debug build — an unoptimized timing is meaningless (parity with `xtask e1`'s
    // debug-build refusal). Print + return rather than assert (a `cfg!` assert is a constant).
    if cfg!(debug_assertions) {
        eprintln!(
            "E1(MLIR) refusing to measure a debug build — re-run with --release \
             (`cargo test -p mycelium-mlir --features mlir-dialect --release -- --ignored \
             e1_mlir_dialect_speedup_is_measured --nocapture`)."
        );
        return;
    }

    // Representative element-wise program: not(A xor B) over 8 bits.
    let prog = Node::Op {
        prim: "bit.not".into(),
        args: vec![Node::Op {
            prim: "bit.xor".into(),
            args: vec![Node::Const(byte(A)), Node::Const(byte(B))],
        }],
    };

    // Compile once through the MLIR pipeline (skip if the toolchain is absent).
    let t0 = std::time::Instant::now();
    let artifact = match mycelium_mlir::mlir_compile(&prog) {
        Ok(a) => a,
        Err(DialectError::ToolchainMissing(tool)) => {
            eprintln!("E1(MLIR) skip: MLIR toolchain absent ({tool}) — run scripts/setup-mlir.sh.");
            return;
        }
        Err(e) => panic!("MLIR compile failed: {e}"),
    };
    #[allow(clippy::cast_precision_loss)]
    let compile_ns = t0.elapsed().as_nanos() as f64;

    // Correctness gate before timing: the MLIR artifact must agree with the interpreter (refusing to
    // time a wrong kernel — the `xtask e1` discipline).
    let interp = Interpreter::new(PrimRegistry::with_builtins(), Box::new(IdentitySwapEngine));
    let interp_val = interp.eval(&prog).expect("interp eval");
    let native_val = artifact.run().expect("MLIR artifact run");
    assert_eq!(
        observable(&interp_val),
        observable(&native_val),
        "E1(MLIR): native disagrees with interpreter — refusing to time a wrong kernel"
    );

    // Warm timing: minimum mean over a few batches (house style — no bench dependency).
    let native_ns = bench(40u32, || {
        std::hint::black_box(artifact.run().expect("run"));
    });
    let interp_ns = bench(20_000u32, || {
        std::hint::black_box(interp.eval(std::hint::black_box(&prog)).expect("eval"));
    });

    let ratio = if native_ns > 0.0 {
        native_ns / interp_ns
    } else {
        0.0
    };
    println!(
        "== E1 (M-602): MLIR-dialect native vs interpreter (element-wise, LLVM {}) ==",
        artifact.llvm_major()
    );
    println!("  MLIR AOT compile (emit MLIR + mlir-opt + mlir-translate + clang), one-time : {compile_ns:>14.0} ns");
    println!("  MLIR native per-invocation (spawn + run, warm)                            : {native_ns:>14.0} ns  [process-spawn-bound]");
    println!("  interpreter per-eval (in-process)                                         : {interp_ns:>14.0} ns");
    println!(
        "  ratio native/interp = {ratio:>6.1}x  (>1 ⇒ spawn dominates for this trivial kernel)"
    );
    println!(
        "  note: the per-invocation figure is process-spawn-bound for this trivial kernel, not \
         kernel compute. This is the AOT-path E1 number, measured — no pre-written target (VR-5). \
         In-process compute-throughput E1 numbers are in `xtask e1` §3–§5."
    );
}
