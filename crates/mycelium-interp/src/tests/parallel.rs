//! M-862: the `parallel_eval == sequential_eval` differential over a corpus of pure fragments, plus
//! `is_pure` gate tests (the EXPLAIN-able selection). White-box access via `use crate::…::*`
//! (CLAUDE.md test-layout rule).
use crate::parallel::is_pure;
use crate::{EvalError, Interpreter};
use mycelium_core::{
    Alt, CtorSpec, DataRegistry, DeclSpec, FieldSpec, Meta, Node, Payload, Provenance, Repr, Value,
};
use std::collections::BTreeMap;

fn byte(bits: [bool; 8]) -> Node {
    Node::Const(
        Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(bits.to_vec()),
            Meta::exact(Provenance::Root),
        )
        .unwrap(),
    )
}

fn op(prim: &str, args: Vec<Node>) -> Node {
    Node::Op {
        prim: prim.to_owned(),
        args,
    }
}

/// `Pair(Binary{8}, Binary{8})` and `type Nat = Z | S(Nat)` — enough data shape for
/// `Construct`/`Match` fixtures.
fn registry() -> DataRegistry {
    let mut m = BTreeMap::new();
    m.insert(
        "Pair".to_owned(),
        DeclSpec {
            ctors: vec![CtorSpec {
                fields: vec![
                    FieldSpec::Repr(Repr::Binary { width: 8 }),
                    FieldSpec::Repr(Repr::Binary { width: 8 }),
                ],
            }],
        },
    );
    m.insert(
        "Nat".to_owned(),
        DeclSpec {
            ctors: vec![
                CtorSpec { fields: vec![] },
                CtorSpec {
                    fields: vec![FieldSpec::Data("Nat".to_owned())],
                },
            ],
        },
    );
    DataRegistry::build(&m).unwrap()
}

fn z(r: &DataRegistry) -> Node {
    Node::Construct {
        ctor: r.ctor_ref("Nat", 0).unwrap(),
        args: vec![],
    }
}
fn s(r: &DataRegistry, n: Node) -> Node {
    Node::Construct {
        ctor: r.ctor_ref("Nat", 1).unwrap(),
        args: vec![n],
    }
}

/// `drop_ = Fix(f, λn. match n { Z => Z, S(m) => f m })` — reused from `r4_tests`'s fixture shape.
fn drop_(r: &DataRegistry) -> Node {
    Node::Fix {
        name: "f".into(),
        body: Box::new(Node::Lam {
            param: "n".into(),
            body: Box::new(Node::Match {
                scrutinee: Box::new(Node::Var("n".into())),
                alts: vec![
                    Alt::Ctor {
                        ctor: r.ctor_ref("Nat", 0).unwrap(),
                        binders: vec![],
                        body: z(r),
                    },
                    Alt::Ctor {
                        ctor: r.ctor_ref("Nat", 1).unwrap(),
                        binders: vec!["m".into()],
                        body: Node::App {
                            func: Box::new(Node::Var("f".into())),
                            arg: Box::new(Node::Var("m".into())),
                        },
                    },
                ],
                default: None,
            }),
        }),
    }
}

/// A corpus of **pure** Core IR fragments spanning every node family the parallel evaluator
/// special-cases (`Op`/`Construct` fan-out, `App`/`Let`/`Match`/`Fix` ordering).
fn pure_corpus() -> Vec<Node> {
    let r = registry();
    vec![
        // A bare constant.
        byte([true; 8]),
        // A single Op.
        op("bit.not", vec![byte([false; 8])]),
        // Nested, independent Op args — the exact "independent Construct/Op element" shape M-862
        // targets: `and(not(a), or(b, c))`.
        op(
            "bit.and",
            vec![
                op(
                    "bit.not",
                    vec![byte([true, false, true, false, true, false, true, false])],
                ),
                op(
                    "bit.or",
                    vec![
                        byte([false; 8]),
                        byte([false, false, false, false, true, true, true, true]),
                    ],
                ),
            ],
        ),
        // A Construct whose two fields are themselves independent Op subterms.
        Node::Construct {
            ctor: r.ctor_ref("Pair", 0).unwrap(),
            args: vec![
                op("bit.not", vec![byte([true; 8])]),
                op("bit.and", vec![byte([true; 8]), byte([false; 8])]),
            ],
        },
        // A deeper Construct: S(S(Z)).
        s(&r, s(&r, z(&r))),
        // Let over a pure Op.
        Node::Let {
            id: "x".into(),
            bound: Box::new(op("bit.not", vec![byte([false; 8])])),
            body: Box::new(op("bit.and", vec![Node::Var("x".into()), byte([true; 8])])),
        },
        // Beta reduction: (λx. not(x)) applied to a value.
        Node::App {
            func: Box::new(Node::Lam {
                param: "x".into(),
                body: Box::new(op("bit.not", vec![Node::Var("x".into())])),
            }),
            arg: Box::new(byte([false, false, false, false, true, true, true, true])),
        },
        // Curried application (both App positions are independent closed subterms).
        Node::App {
            func: Box::new(Node::App {
                func: Box::new(Node::Lam {
                    param: "x".into(),
                    body: Box::new(Node::Lam {
                        param: "y".into(),
                        body: Box::new(op(
                            "bit.xor",
                            vec![Node::Var("x".into()), Node::Var("y".into())],
                        )),
                    }),
                }),
                arg: Box::new(byte([true, true, true, true, false, false, false, false])),
            }),
            arg: Box::new(byte([false, false, false, false, true, true, true, true])),
        },
        // Match selecting a constructor arm.
        Node::Match {
            scrutinee: Box::new(s(&r, z(&r))),
            alts: vec![
                Alt::Ctor {
                    ctor: r.ctor_ref("Nat", 0).unwrap(),
                    binders: vec![],
                    body: z(&r),
                },
                Alt::Ctor {
                    ctor: r.ctor_ref("Nat", 1).unwrap(),
                    binders: vec!["m".into()],
                    body: Node::Var("m".into()),
                },
            ],
            default: None,
        },
        // Fix-driven structural recursion (drop_ applied to S(S(S(Z)))).
        Node::App {
            func: Box::new(drop_(&r)),
            arg: Box::new(s(&r, s(&r, s(&r, z(&r))))),
        },
    ]
}

#[test]
fn pure_corpus_fragments_are_all_marked_pure() {
    for (i, node) in pure_corpus().iter().enumerate() {
        assert!(is_pure(node), "corpus[{i}] expected pure: {node:?}");
    }
}

/// The M-862 headline claim: `eval_core_parallel == eval_core` over the pure corpus (Empirical,
/// differential-checked — never `Proven`, VR-5).
#[test]
fn parallel_eval_matches_sequential_eval_over_the_pure_corpus() {
    let interp = Interpreter::default();
    for (i, node) in pure_corpus().iter().enumerate() {
        let seq = interp.eval_core(node);
        let par = interp.eval_core_parallel(node);
        assert_eq!(seq, par, "corpus[{i}] diverged: {node:?}");
    }
}

/// Repeating the differential many times catches any nondeterminism a data race would introduce
/// (rayon's scheduling order varies run to run; the *result* must not).
#[test]
fn parallel_eval_is_deterministic_across_repeated_runs() {
    let interp = Interpreter::default();
    for node in pure_corpus() {
        let first = interp.eval_core_parallel(&node);
        for _ in 0..25 {
            assert_eq!(
                first,
                interp.eval_core_parallel(&node),
                "nondeterministic: {node:?}"
            );
        }
    }
}

#[test]
fn wild_prim_is_never_pure_even_deeply_nested() {
    // A `wild:` op anywhere in the tree makes the WHOLE fragment ineligible (all-or-nothing gate).
    let wild_leaf = op("wild:foreign", vec![byte([true; 8])]);
    assert!(!is_pure(&wild_leaf));

    let nested = op("bit.and", vec![byte([true; 8]), wild_leaf]);
    assert!(
        !is_pure(&nested),
        "a nested wild: op must taint the whole fragment"
    );
}

#[test]
fn an_impure_fragment_falls_back_to_the_sequential_reference_and_still_matches() {
    // `eval_core_parallel` on an impure fragment must equal plain `eval_core` (the fallback path),
    // not merely "run without crashing".
    let interp = Interpreter::default();
    let wild = op("wild:foreign", vec![byte([true; 8])]);
    assert_eq!(interp.eval_core(&wild), interp.eval_core_parallel(&wild));
    assert!(matches!(
        interp.eval_core_parallel(&wild).unwrap_err(),
        EvalError::UnknownPrim(p) if p == "wild:foreign"
    ));
}

#[test]
fn swap_is_conservatively_never_pure() {
    // `SwapEngine` is an opaque `Box<dyn>` — purity can't be verified structurally, so every `Swap`
    // is a parallelism boundary even though the shipped `IdentitySwapEngine` happens to be pure.
    let swap = Node::Swap {
        src: Box::new(byte([true; 8])),
        target: Repr::Binary { width: 8 },
        policy: mycelium_core::operation_hash("policy"),
    };
    assert!(!is_pure(&swap));

    let interp = Interpreter::default();
    assert_eq!(interp.eval_core(&swap), interp.eval_core_parallel(&swap));
}

#[test]
fn fuel_exhaustion_agrees_between_sequential_and_parallel() {
    // Fix(f, f) loops; both paths must refuse identically under a tight fuel budget — never a hang,
    // never a silent divergence between the two evaluators.
    let spin = Node::Fix {
        name: "f".into(),
        body: Box::new(Node::Var("f".into())),
    };
    assert!(is_pure(&spin));
    let interp = Interpreter::default().with_fuel(64);
    assert_eq!(
        interp.eval_core(&spin).unwrap_err(),
        EvalError::FuelExhausted
    );
    assert_eq!(
        interp.eval_core_parallel(&spin).unwrap_err(),
        EvalError::FuelExhausted
    );
}

#[test]
fn eval_parallel_repr_entry_point_agrees_with_eval() {
    let interp = Interpreter::default();
    let node = op("bit.not", vec![byte([false; 8])]);
    assert_eq!(
        interp.eval(&node).unwrap(),
        interp.eval_parallel(&node).unwrap()
    );
}

#[test]
fn eval_parallel_on_a_data_result_is_an_explicit_refusal_like_eval() {
    let r = registry();
    let interp = Interpreter::default();
    assert_eq!(
        interp.eval_parallel(&z(&r)).unwrap_err(),
        EvalError::DataResult
    );
}
