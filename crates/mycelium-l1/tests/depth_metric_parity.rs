//! RFC-0041 W0 — the **§4.0 depth-metric** property test and the **§5.1 error-parity** differential
//! gate (recursion-depth safety, "safety-net" wave). This file adds *tests only* — no behaviour
//! change, no edit to any logic file.
//!
//! ## The §4.0 metric (`source_call_depth`) — Part 1, PASSES today
//! §4.0 fixes **one machine-independent depth metric**: the budget is charged **one unit per
//! source-level call/β boundary** — a user function application (`App`) or a `Fix` unfold — **not**
//! per internal IR node. Consequences pinned here:
//!   * an n-ary application `f(a, b, c)` is depth **1**, not 3 (arity does not multiply depth);
//!   * nested user calls increment by 1 each (`not(not(not(x)))` is depth 3);
//!   * a data-spine literal — a `Cons`/list literal `[e₀, …, e_{N-1}]` — has data-spine depth **N**,
//!     charged by element uniformly (RFC-0040 desugars it to an N-long `Cons` chain);
//!   * a flat, non-recursive body is depth 0/1.
//!
//! [`source_call_depth`] is a **pure** syntactic function of the *surface AST* (parse-only, no
//! type-checking, no cross-function inlining): it measures the maximum call-chain nesting within a
//! function body. It is therefore a **static** approximation of the runtime charge — cross-function
//! recursion depth is a runtime quantity, out of scope for the pure metric (§4.0: tail iterations do
//! not charge either). Honesty (VR-5): the metric and its property test are **`Empirical`/`Declared`**
//! — a heuristic over fixtures, **never** `Proven`. Source is ground truth.
//!
//! ## The §5.1 error-parity gate (`error_parity_at_the_canonical_threshold`) — Part 2, `#[ignore]`
//! §5.1 requires **one canonical over-budget error variant + width** shared across all three paths,
//! refused at the **same §4.0 metric threshold** for an input past the floor. It **fails today** and
//! is a W0 precondition to reconcile, not an afterthought (RFC-0041 §5.1). Today the three paths
//! diverge:
//!   * **L1-eval** refuses with `L1Error::DepthExceeded { limit: u32 }` at its default depth **64**
//!     (`eval.rs` `DEFAULT_DEPTH`), charging per-`Expr`-node (not the §4.0 metric yet);
//!   * **L0-interp** (`Interpreter::eval`) has **no** depth budget — it is recursive and **SIGABRTs**
//!     on deep input (the very hole RFC-0041 §1 targets); it constructs no `DepthLimit` until W4;
//!   * **AOT** (`mycelium_mlir::run` / `run_core`, the env-machine) refuses with
//!     `EvalError::DepthLimit { limit: usize }` at a **dynamic** ceiling in `[10 000, 2 000 000]`
//!     (DN-05 memory-derived), not the deterministic floor.
//!
//! So no single input makes all three refuse at one threshold today — hence `#[ignore = "W5"]`.
//!
//! **Canonical over-budget variant (orchestrator decision — encoded as this gate's target):**
//! `DepthExceeded { limit: u32 }` on the **§4.0 metric**, defaulting to the **4096** depth floor
//! (§4.2). This becomes `mycelium-workstack`'s `RecursionBudget` error in **W1**. The interp
//! `EvalError::DepthLimit { limit: usize }` and the AOT `EvalError::DepthLimit { limit: usize }`
//! reconcile to it in **W4** (interp constructs `DepthLimit`, `myc run` routed through it) and
//! **W3½** (AOT env-machine onto the shared guard); **W5** aligns L1-eval to the canonical metric +
//! variant and raises eval 64 → 4096. When those land, this ignored gate goes green.
//!
//! The ignored gate still **compiles** (it references only real public APIs), so it is a live,
//! type-checked specification of the target, not a comment.

use mycelium_cert::BinaryTernarySwapEngine;
use mycelium_interp::{Interpreter, PrimRegistry};
use mycelium_l1::ast::{Expr, Item, Literal};
use mycelium_l1::{check_nodule, elaborate, parse, Evaluator};

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// Part 1 — the §4.0 machine-independent depth metric (a pure function + property test).
// ─────────────────────────────────────────────────────────────────────────────────────────────────

/// The **§4.0 depth metric** of a source program: the maximum source-level call/β nesting over every
/// top-level function body, charging **one unit per user-`App` boundary** and **N for an N-element
/// data-spine (`Cons`/list) literal** — never per internal IR node. Pure and parse-only (no
/// type-checking); a heuristic (`Empirical`/`Declared`, not `Proven`). Panics never-silently (G2) on
/// a source that does not parse — callers pass known-good fixtures.
fn source_call_depth(src: &str) -> usize {
    let nodule = parse(src).expect("the §4.0 metric fixture must parse");
    nodule
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(decl) => Some(expr_depth(&decl.body)),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

/// The §4.0 nesting depth of a single surface expression. **`App` charges +1** (one source call/β
/// boundary) over the deepest of its head and arguments — so an n-ary `f(a, b, c)` with flat
/// arguments is depth 1, not 3, and nested calls increment by 1. A **list literal** (the `Cons`
/// data-spine, RFC-0040) charges its **element count** plus the deepest element (so `[a, b, c]` of
/// flat elements is depth 3). Every other construct is transparent — it recurses into its children
/// and takes the max, charging nothing itself (a `let`/`match`/`swap`/ascription is not a call).
fn expr_depth(e: &Expr) -> usize {
    match e {
        // A user function / constructor application: one source call/β boundary. n-ary args are
        // siblings (max, not sum), so `f(a, b, c)` is depth 1; nesting accumulates by 1 each.
        Expr::App { head, args } => {
            1 + expr_depth(head).max(args.iter().map(expr_depth).max().unwrap_or(0))
        }
        // A data-spine (`Cons`) literal of N elements is depth N (charged by element uniformly),
        // plus the deepest element's own nesting.
        Expr::Lit(Literal::List(elems)) => {
            elems.len() + elems.iter().map(expr_depth).max().unwrap_or(0)
        }
        // Bare literals and path/variable references are leaves.
        Expr::Lit(_) | Expr::Path(_) => 0,
        // Binders / control that are NOT call boundaries: transparent (max over children).
        Expr::Let { bound, body, .. } => expr_depth(bound).max(expr_depth(body)),
        Expr::If { cond, conseq, alt } => expr_depth(cond)
            .max(expr_depth(conseq))
            .max(expr_depth(alt)),
        Expr::Match { scrutinee, arms } => {
            expr_depth(scrutinee).max(arms.iter().map(|a| expr_depth(&a.body)).max().unwrap_or(0))
        }
        Expr::For { xs, init, body, .. } => {
            expr_depth(xs).max(expr_depth(init)).max(expr_depth(body))
        }
        Expr::Swap { value, .. } => expr_depth(value),
        Expr::WithParadigm { body, .. } => expr_depth(body),
        Expr::Wild(inner) | Expr::Spore(inner) | Expr::Consume(inner) => expr_depth(inner),
        Expr::Colony(hyphae) => hyphae
            .iter()
            .map(|h| expr_depth(&h.body))
            .max()
            .unwrap_or(0),
        Expr::Lambda { body, .. } => expr_depth(body),
        Expr::Fuse { left, right } => expr_depth(left).max(expr_depth(right)),
        Expr::Reclaim { policy, body } => expr_depth(policy).max(expr_depth(body)),
        Expr::Ascribe(inner, _) => expr_depth(inner),
        // A tuple is a single flat constructor (arity does not add spine depth): max over elements.
        Expr::TupleLit(elems) => elems.iter().map(expr_depth).max().unwrap_or(0),
    }
}

/// **§4.0 property (`Empirical`): the metric matches the known source-level call depth of each
/// fixture.** Each fixture pins one clause of §4.0 — a flat body is 0, a single call is 1, an n-ary
/// call is 1 (not its arity), nested calls increment by 1, a nested constructor spine counts its
/// depth, and an N-element list literal is N. This test is **not** ignored and MUST PASS.
#[test]
fn source_call_depth_matches_the_known_metric_of_each_fixture() {
    // (source, expected §4.0 depth, pinned §4.0 clause).
    let fixtures: &[(&str, usize, &str)] = &[
        // flat, non-recursive body → 0 (a bare literal is not a call).
        (
            "nodule d;\nfn main() => Binary{8} = 0b1011_0010;",
            0,
            "flat literal is depth 0",
        ),
        // a single user call → 1.
        (
            "nodule d;\nfn main() => Binary{8} = not(0b1011_0010);",
            1,
            "one call is depth 1",
        ),
        // an n-ary (2-arg) application is depth 1, NOT its arity.
        (
            "nodule d;\nfn main() => Binary{8} = xor(0b1011_0010, 0b1111_1111);",
            1,
            "binary application is depth 1 (arity does not multiply)",
        ),
        // an n-ary (3-arg) application is STILL depth 1 — the canonical `f(a,b,c)` case (§4.0).
        (
            "nodule d;\nfn main() => Binary{8} = f(0b1, 0b1, 0b1);",
            1,
            "ternary application f(a,b,c) is depth 1, not 3",
        ),
        // nested user calls increment by 1 each: not(not(not(x))) → 3.
        (
            "nodule d;\nfn main() => Binary{8} = not(not(not(0b1010_1010)));",
            3,
            "three nested calls are depth 3",
        ),
        // helper + caller: the metric is the max over all fn bodies (main's flip(flip(..)) = 2).
        (
            "nodule d;\nfn flip(x: Binary{8}) => Binary{8} = not(x);\nfn main() => Binary{8} = flip(flip(0b1010_1010));",
            2,
            "max over bodies: nested flip(flip(..)) is depth 2",
        ),
        // a nested constructor data-spine S(S(Z)) → 2 (each S is one boundary; Z is a leaf).
        (
            "nodule d;\ntype Nat = Z | S(Nat);\nfn main() => Nat = S(S(Z));",
            2,
            "nested constructor spine S(S(Z)) is depth 2",
        ),
        // a list (Cons) literal of N=3 elements → 3, charged by element uniformly.
        (
            "nodule d;\nfn main() => Seq{Binary{8}, 3} = [0b1111_0000, 0b0000_1111, 0b1010_1010];",
            3,
            "a Cons literal of N elements is depth N",
        ),
    ];

    for (src, expected, clause) in fixtures {
        assert_eq!(
            source_call_depth(src),
            *expected,
            "§4.0 metric mismatch — {clause}:\n{src}"
        );
    }
}

/// **Non-vacuity / mutant witness: the metric genuinely DISTINGUISHES different-depth sources**
/// (mirrors `differential.rs`'s `the_data_differential_distinguishes_divergent_elaborations`). A
/// metric that returned a constant — or that summed arity instead of taking the max — would still
/// pass a lone equality; these assertions rule that out, so a green Part-1 test is meaningful.
#[test]
fn the_metric_distinguishes_different_depth_sources() {
    let one_call = "nodule d;\nfn main() => Binary{8} = not(0b1010_1010);";
    let three_nested = "nodule d;\nfn main() => Binary{8} = not(not(not(0b1010_1010)));";
    // Depth is not constant: a deeper nest measures strictly deeper.
    assert!(
        source_call_depth(three_nested) > source_call_depth(one_call),
        "the metric must rank a 3-deep nest above a single call (not a constant)"
    );

    // Arity is NOT nesting: a 3-ARG flat call (depth 1) is strictly shallower than a 3-DEEP nest
    // (depth 3) — the §4.0 distinction a per-node or per-arg charge would collapse.
    let three_arg = "nodule d;\nfn main() => Binary{8} = f(0b1, 0b1, 0b1);";
    assert_ne!(
        source_call_depth(three_arg),
        source_call_depth(three_nested),
        "a 3-arg flat call and a 3-deep nest must NOT measure the same depth"
    );
    assert!(
        source_call_depth(three_arg) < source_call_depth(three_nested),
        "an n-ary call must be shallower than an equally-wide nesting (arity ≠ depth)"
    );

    // A longer data-spine literal measures strictly deeper — the element-count charge is real.
    let cons1 = "nodule d;\nfn main() => Seq{Binary{8}, 1} = [0b1111_0000];";
    let cons3 =
        "nodule d;\nfn main() => Seq{Binary{8}, 3} = [0b1111_0000, 0b0000_1111, 0b1010_1010];";
    assert!(
        source_call_depth(cons3) > source_call_depth(cons1),
        "a longer Cons literal must measure a deeper data-spine"
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// Part 2 — the §5.1 cross-path error-parity + threshold differential (defined, `#[ignore = "W5"]`).
// ─────────────────────────────────────────────────────────────────────────────────────────────────

/// The **canonical over-budget threshold** (§4.2 default floor): depth **4096** on the §4.0 metric.
/// The canonical *variant* is `L1Error::DepthExceeded { limit: u32 }` (→ `mycelium-workstack`'s
/// `RecursionBudget` error, W1); interp/AOT `EvalError::DepthLimit { limit: usize }` reconcile to it
/// (W4 / W3½), and L1-eval is re-charged onto this metric + raised 64 → 4096 in W5.
const CANONICAL_DEPTH_FLOOR: u32 = 4096;

/// Build a program whose `main` constructs a nested `Nat` data-spine `S(S(…S(Z)…))` of depth `n` —
/// so its **§4.0 metric depth is exactly `n`**. Used to drive an input past the canonical floor.
fn deep_nat_program(n: usize) -> String {
    let mut body = String::from("Z");
    for _ in 0..n {
        body = format!("S({body})");
    }
    format!("nodule d;\ntype Nat = Z | S(Nat);\nfn main() => Nat = {body};")
}

/// **§5.1 error-parity + threshold gate (W0-defined; `#[ignore = "W5"]` because it FAILS today).**
/// For an input past the floor, all three execution paths must **refuse with the canonical
/// over-budget error at the same §4.0 metric threshold** (`CANONICAL_DEPTH_FLOOR`):
///   * L1-eval → `L1Error::DepthExceeded { limit }`;
///   * L0-interp → `EvalError::DepthLimit { limit }` (constructed in W4; today the path has no
///     budget and SIGABRTs, which is *why* this whole test is ignored — running it would abort the
///     test binary);
///   * AOT env-machine → `EvalError::DepthLimit { limit }`.
///
/// Each `limit` must equal `CANONICAL_DEPTH_FLOOR`. This goes green once **W4** constructs the interp
/// `DepthLimit`, **W3½** pins AOT to the deterministic floor, and **W5** aligns L1-eval to the §4.0
/// metric + variant. The test is kept COMPILING (real public APIs) so it is a checked specification.
#[test]
#[ignore = "W5: fails today — L1 refuses at 64, interp has no budget (SIGABRTs), AOT floor is dynamic [10k,2M]; goes green when W4 constructs interp DepthLimit and W5 aligns eval to the canonical metric+variant"]
fn error_parity_at_the_canonical_threshold() {
    use mycelium_interp::EvalError;
    use mycelium_l1::L1Error;

    // An input whose §4.0 metric depth (== the nesting count) is safely past the 4096 floor.
    let depth = (CANONICAL_DEPTH_FLOOR as usize) + 1000;
    let src = deep_nat_program(depth);
    // The generated input's §4.0 metric depth is exactly `depth` — past the floor by construction.
    assert_eq!(
        source_call_depth(&src),
        depth,
        "the deep input's §4.0 metric depth must equal its constructed nesting"
    );

    let env = check_nodule(&parse(&src).expect("the deep input parses")).expect("checks");
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;
    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(BinaryTernarySwapEngine),
    );

    // Path 1 — L1-eval: the canonical variant `DepthExceeded { limit }` at the canonical floor.
    let l1_err = Evaluator::new(&env)
        .call("main", vec![])
        .expect_err("L1-eval must refuse the over-floor input, never a host-stack abort");
    assert!(
        matches!(l1_err, L1Error::DepthExceeded { limit } if limit == CANONICAL_DEPTH_FLOOR),
        "L1-eval must refuse with the canonical DepthExceeded {{ limit: {CANONICAL_DEPTH_FLOOR} }}; got: {l1_err:?}"
    );

    // Path 2 — L0-interp: `EvalError::DepthLimit { limit }` at the same threshold (W4 constructs it).
    let node = elaborate(&env, "main").expect("the deep input elaborates");
    let interp_err = interp
        .eval(&node)
        .expect_err("L0-interp must refuse the over-floor input, never SIGABRT (W4)");
    assert!(
        matches!(interp_err, EvalError::DepthLimit { limit } if limit == CANONICAL_DEPTH_FLOOR as usize),
        "L0-interp must refuse with DepthLimit {{ limit: {CANONICAL_DEPTH_FLOOR} }} (reconciles to the canonical variant, W4); got: {interp_err:?}"
    );

    // Path 3 — AOT env-machine: `EvalError::DepthLimit { limit }` at the same deterministic floor.
    let aot_err = mycelium_mlir::run(&node, &prims, &engine)
        .expect_err("AOT must refuse the over-floor input at the deterministic floor (W3½)");
    assert!(
        matches!(aot_err, EvalError::DepthLimit { limit } if limit == CANONICAL_DEPTH_FLOOR as usize),
        "AOT must refuse with DepthLimit {{ limit: {CANONICAL_DEPTH_FLOOR} }} pinned to the floor (W3½); got: {aot_err:?}"
    );
}
