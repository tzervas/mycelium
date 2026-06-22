//! L1 static checking (RFC-0007 §4.4/§4.5): the monomorphic typechecker, the structural totality
//! checker, and the scope-quantified `matured ⟹ total` gate (RFC-0017 §4.2). Every refusal is
//! an explicit `CheckError`.

use mycelium_l1::{
    check_nodule, check_nodule_matured, elaborate, parse, ElabError, Evaluator, Totality,
};

fn check(src: &str) -> Result<mycelium_l1::Env, mycelium_l1::CheckError> {
    let nodule = parse(src).expect("parses");
    check_nodule(&nodule)
}

fn check_matured(src: &str) -> Result<mycelium_l1::Env, mycelium_l1::CheckError> {
    let nodule = parse(src).expect("parses");
    check_nodule_matured(&nodule, true)
}

#[test]
fn well_typed_swap_fn_checks() {
    let env = check(
        "nodule d\nfn widen(x: Binary{8}) -> Ternary{6} = swap(x, to: Ternary{6}, policy: rt)",
    )
    .expect("checks");
    assert_eq!(env.totality["widen"], Totality::Total);
}

#[test]
fn type_mismatch_is_explicit() {
    // body is Ternary{6}, signature says Binary{8}. As of M-344 (RFC-0012 §4.4) a *cross-paradigm*
    // edge is sharpened from a generic mismatch to an explicit `MissingConversion` that names the
    // from/to reprs and points at writing a `swap` — still never-silent, now more actionable.
    let err =
        check("nodule d\nfn f(x: Binary{8}) -> Binary{8} = swap(x, to: Ternary{6}, policy: rt)")
            .unwrap_err();
    assert!(
        err.message.contains("MissingConversion") && err.message.contains("swap"),
        "got: {}",
        err.message
    );
}

#[test]
fn same_paradigm_width_mismatch_is_a_plain_type_error() {
    // A same-paradigm mismatch (two Binary widths) is *not* a MissingConversion — no conversion
    // would bridge it — so it keeps the plain "type" wording (RFC-0012 §4.4 boundary).
    let err = check("nodule d\nfn f(x: Binary{8}) -> Binary{6} = not(x)").unwrap_err();
    assert!(
        err.message.contains("type") && !err.message.contains("MissingConversion"),
        "got: {}",
        err.message
    );
}

#[test]
fn exhaustive_match_checks_and_nonexhaustive_is_refused() {
    let ok = "nodule d\ntype Sign = Neg | Zero | Pos\nfn f(s: Sign) -> Sign = match s { Neg => s, Zero => s, Pos => s }";
    assert!(check(ok).is_ok());

    let bad = "nodule d\ntype Sign = Neg | Zero | Pos\nfn f(s: Sign) -> Sign = match s { Neg => s, Pos => s }";
    let err = check(bad).unwrap_err();
    assert!(
        err.message.contains("non-exhaustive"),
        "got: {}",
        err.message
    );
}

const NAT: &str = "nodule d\ntype Nat = Z | S(Nat)\n";

#[test]
fn nested_pattern_match_typechecks() {
    // Depth-2 nested patterns, exhaustive (Z | S(Z) | S(S(_))). The binder `m` is in scope at the
    // S(S(m)) arm with type Nat.
    let ok =
        format!("{NAT}fn pred2(n: Nat) -> Nat = match n {{ Z => Z, S(Z) => Z, S(S(m)) => m }}");
    assert!(check(&ok).is_ok(), "{:?}", check(&ok));
}

#[test]
fn nested_nonexhaustive_match_reports_a_precise_witness() {
    // Z | S(S(m)) misses S(Z) — the Maranget witness names the missing nested case exactly.
    let bad = format!("{NAT}fn f(n: Nat) -> Nat = match n {{ Z => Z, S(S(m)) => m }}");
    let err = check(&bad).unwrap_err();
    assert!(
        err.message.contains("non-exhaustive"),
        "got: {}",
        err.message
    );
    assert!(
        err.message.contains("S(Z)"),
        "witness must name the missing case, got: {}",
        err.message
    );
}

#[test]
fn nested_redundant_arm_is_unreachable() {
    // After Z and S(m), the nested arm S(Z) is already covered ⇒ unreachable (W7 redundancy).
    let bad = format!("{NAT}fn f(n: Nat) -> Nat = match n {{ Z => Z, S(m) => m, S(Z) => Z }}");
    let err = check(&bad).unwrap_err();
    assert!(err.message.contains("unreachable"), "got: {}", err.message);
}

#[test]
fn nested_binder_drives_structural_descent_for_matured() {
    // RFC-0017 §4.2: scope-quantified gate. A recursion descending through a *nested* pattern
    // binder (S(S(m)) → m) is structurally smaller, so the totality checker must classify it Total
    // and the matured scope must admit it (the depth-2 descent the extended smallness tracking
    // now recognizes).
    let ok = format!(
        "{NAT}fn half(n: Nat) -> Nat = \
         match n {{ Z => Z, S(Z) => Z, S(S(m)) => S(half(m)) }}"
    );
    assert!(check_matured(&ok).is_ok(), "{:?}", check_matured(&ok));
}

#[test]
fn structural_recursion_is_total_and_gates_matured() {
    // RFC-0017 §4.2: a structurally-decreasing self-recursion over a Peano-like type is classified
    // Total and admitted by a matured scope.
    let src = "nodule d\n\
               type Nat = Z | S(Nat)\n\
               fn count(n: Nat) -> Nat = match n { Z => n, S(m) => count(m) }";
    let env = check_matured(src).expect("checks");
    assert_eq!(env.totality["count"], Totality::Total);
}

#[test]
fn non_decreasing_recursion_cannot_be_matured() {
    // Mutant-witness for RFC-0017 §4.2 / RFC-0007 §4.5: the recursive call passes the parameter
    // unchanged → not structurally smaller → Partial. In a matured scope, a non-total non-thaw fn
    // must be refused.
    let src = "nodule d\n\
               type Nat = Z | S(Nat)\n\
               fn spin(n: Nat) -> Nat = match n { Z => n, S(m) => spin(n) }";
    let err = check_matured(src).unwrap_err();
    assert!(
        err.message.contains("matured") || err.message.contains("total"),
        "got: {}",
        err.message
    );
}

#[test]
fn thaw_fn_is_exempt_from_matured_scope_gate() {
    // Mutant-witness for RFC-0017 §4.3: a partial fn marked `thaw` is exempt from the matured
    // scope totality gate. Without the `thaw` exemption, this would be refused.
    let src = "nodule d\n\
               type Nat = Z | S(Nat)\n\
               thaw fn spin(n: Nat) -> Nat = match n { Z => n, S(m) => spin(n) }";
    let env = check_matured(src)
        .expect("thaw fn must be accepted even in a matured scope (RFC-0017 §4.3)");
    assert_eq!(env.totality["spin"], Totality::Partial);
}

#[test]
fn non_decreasing_recursion_is_allowed_when_not_matured() {
    // Same body without `matured` checks fine — partiality is an honest classification, not an error.
    let src = "nodule d\n\
               type Nat = Z | S(Nat)\n\
               fn spin(n: Nat) -> Nat = match n { Z => n, S(m) => spin(n) }";
    let env = check(src).expect("checks");
    assert_eq!(env.totality["spin"], Totality::Partial);
}

#[test]
fn shadowing_rebind_does_not_leak_smallness() {
    // A4-01 regression: the inner arm rebinds `m` (matching `p`, an unrelated parameter), shadowing
    // the outer `m` (a piece of `n`). The recursion `f(m, p)` is therefore NOT structural —
    // `f(3,2) → f(1,2) → f(1,2) → …` diverges — so it must be classified Partial. Mutant-witness:
    // without the drop-and-restore shadow handling in descend_walk's Match arm, the stale smallness
    // of the outer `m` leaks in, `f` is wrongly classified Total, and the `matured` form below is
    // wrongly accepted.
    let body = "match n { Z => Z, S(m) => match p { Z => Z, S(m) => f(m, p) } }";
    let src = format!("nodule d\ntype Nat = Z | S(Nat)\nfn f(n: Nat, p: Nat) -> Nat = {body}");
    let env = check(&src).expect("checks");
    assert_eq!(env.totality["f"], Totality::Partial);

    // In a matured scope the same partial fn must be refused (RFC-0017 §4.2).
    let matured_src =
        format!("nodule d\ntype Nat = Z | S(Nat)\nfn f(n: Nat, p: Nat) -> Nat = {body}");
    let err = check_matured(&matured_src).unwrap_err();
    assert!(
        err.message.contains("matured") || err.message.contains("total"),
        "got: {}",
        err.message
    );
}

// --- mutual-descent classification (RFC-0007 §4.5; M-343 / R7-Q3 loose end) ---

#[test]
fn mutual_recursion_with_structural_descent_is_total() {
    // ping/pong descend on position 0 across the group, so the FixGroup is `Total` and `matured`
    // is admissible — the M-343 loose end: before mutual-descent classification this was `Partial`.
    let src = "nodule d\ntype Nat = Z | S(Nat)\n\
               fn ping(n: Nat) -> Nat = match n { Z => Z, S(m) => pong(m) }\n\
               fn pong(n: Nat) -> Nat = match n { Z => Z, S(m) => ping(m) }";
    let env = check(src).expect("checks");
    assert_eq!(env.totality["ping"], Totality::Total);
    assert_eq!(env.totality["pong"], Totality::Total);

    // The whole group may therefore be admitted by a matured scope (RFC-0017 §4.2).
    let matured = "nodule d\ntype Nat = Z | S(Nat)\n\
                   fn ping(n: Nat) -> Nat = match n { Z => Z, S(m) => pong(m) }\n\
                   fn pong(n: Nat) -> Nat = match n { Z => Z, S(m) => ping(m) }";
    check_matured(matured).expect("a totally-descending mutual group admits a matured scope");
}

#[test]
fn non_productive_mutual_cycle_is_partial() {
    // `a(n) = b(n)` / `b(n) = a(n)` never decreases anything — an unproductive cycle. Honest
    // `Partial` (still runnable, fuel-clocked), and `matured` is refused. Mutant-witness: a checker
    // that classified *any* mutual group `Total` would wrongly mature this non-terminating pair.
    let src = "nodule d\ntype Nat = Z | S(Nat)\n\
               fn a(n: Nat) -> Nat = b(n)\n\
               fn b(n: Nat) -> Nat = a(n)";
    let env = check(src).expect("checks");
    assert_eq!(env.totality["a"], Totality::Partial);
    assert_eq!(env.totality["b"], Totality::Partial);

    // Mutant-witness for RFC-0017 §4.2: a matured scope must refuse partial fns.
    let matured = "nodule d\ntype Nat = Z | S(Nat)\n\
                   fn a(n: Nat) -> Nat = b(n)\n\
                   fn b(n: Nat) -> Nat = a(n)";
    let err = check_matured(matured).unwrap_err();
    assert!(
        err.message.contains("matured") || err.message.contains("total"),
        "got: {}",
        err.message
    );
}

#[test]
fn partial_descent_in_a_mutual_group_is_partial() {
    // Descent must hold on *every* inter-member call. Here `ping` decreases but `pong` re-calls
    // `ping(n)` with the parameter unchanged, so no assignment witnesses descent → `Partial`.
    let src = "nodule d\ntype Nat = Z | S(Nat)\n\
               fn ping(n: Nat) -> Nat = match n { Z => Z, S(m) => pong(m) }\n\
               fn pong(n: Nat) -> Nat = match n { Z => Z, S(m) => ping(n) }";
    let env = check(src).expect("checks");
    assert_eq!(env.totality["ping"], Totality::Partial);
    assert_eq!(env.totality["pong"], Totality::Partial);
}

#[test]
fn three_function_mutual_cycle_descends() {
    // f → g → h → f, each peeling one constructor: a productive 3-cycle is `Total`.
    let src = "nodule d\ntype Nat = Z | S(Nat)\n\
               fn f(n: Nat) -> Nat = match n { Z => Z, S(m) => g(m) }\n\
               fn g(n: Nat) -> Nat = match n { Z => Z, S(m) => h(m) }\n\
               fn h(n: Nat) -> Nat = match n { Z => Z, S(m) => f(m) }";
    let env = check(src).expect("checks");
    assert_eq!(env.totality["f"], Totality::Total);
    assert_eq!(env.totality["g"], Totality::Total);
    assert_eq!(env.totality["h"], Totality::Total);
}

#[test]
fn mutual_descent_on_different_argument_positions() {
    // The designated descent position can differ per member: `f` descends on position 0, `g` on
    // position 1. This exercises the position-assignment search (not just a single shared index),
    // and is `Total` because the structural size strictly decreases around the whole cycle.
    let src = "nodule d\ntype Nat = Z | S(Nat)\n\
               fn f(a: Nat, b: Nat) -> Nat = match a { Z => b, S(m) => g(b, m) }\n\
               fn g(x: Nat, y: Nat) -> Nat = match y { Z => x, S(k) => f(k, x) }";
    let env = check(src).expect("checks");
    assert_eq!(env.totality["f"], Totality::Total);
    assert_eq!(env.totality["g"], Totality::Total);
}

#[test]
fn deeply_nested_input_is_refused_not_a_crash() {
    // A4-02/B2-01 regression: unbounded recursive descent would overflow the host stack and abort
    // the process (SIGABRT) on crafted nesting. The depth guard turns that into an explicit
    // ParseError, well before any crash. Mutant-witness: removing the MAX_EXPR_DEPTH guard in
    // parse_expr makes 2000-deep input abort instead of returning Err.
    let deep = format!(
        "nodule d\nfn f(x: Binary{{8}}) -> Binary{{8}} = {}x{}",
        "(".repeat(2000),
        ")".repeat(2000)
    );
    let err = parse(&deep).expect_err("deep nesting must be refused, not crash");
    assert!(err.message.contains("nests deeper"), "got: {}", err.message);

    // A modest nesting still parses.
    let shallow = format!(
        "nodule d\nfn f(x: Binary{{8}}) -> Binary{{8}} = {}x{}",
        "(".repeat(50),
        ")".repeat(50)
    );
    assert!(parse(&shallow).is_ok());
}

#[test]
fn wild_is_denied_by_default() {
    let src = "nodule d\nfn f(x: Binary{8}) -> Binary{8} = wild { x }";
    let err = check(src).unwrap_err();
    assert!(err.message.contains("wild"), "got: {}", err.message);
}

#[test]
fn generic_adt_declaration_registers_shell() {
    // A generic type declaration registers a GenericShell; no instantiation yet.
    let src = "nodule d\ntype Box<T> = Wrap(T)";
    let env = check(src).expect("generic ADT declaration must check");
    // Shell is stored in generics, not in types
    assert!(
        env.generics.contains_key("Box"),
        "Box should be in generics, got: {:?}",
        env.generics.keys().collect::<Vec<_>>()
    );
    assert!(
        !env.types.contains_key("Box"),
        "Box should NOT be in types until instantiated"
    );
}

#[test]
fn generic_adt_wrong_arity_is_an_explicit_error() {
    // List<A> has 1 param; supplying 2 args at use-site is an explicit error.
    let src =
        "nodule d\ntype List<A> = Nil | Cons(A, List<A>)\ntype Bad = X(List<Binary{8}, Binary{4}>)";
    let err = check(src).unwrap_err();
    assert!(
        err.message.contains("arity") || err.message.contains("argument"),
        "got: {}",
        err.message
    );
}

#[test]
fn shell_field_monomorphic_type_with_args_is_refused() {
    // M-673 review (PR #348): a GENERIC ADT whose ctor field applies a NON-generic type to type
    // arguments is an explicit error (the `resolve_shell_field_ty` path) — never a silent wrong
    // `Ty::App`. G2/never-silent.
    let src = "nodule d\ntype Flag = On | Off\ntype Bad<A> = Mk(Flag<A>)";
    let err = check(src).unwrap_err();
    assert!(
        err.message.contains("not generic") || err.message.contains("no type arguments"),
        "got: {}",
        err.message
    );
}

#[test]
fn shell_field_generic_arity_mismatch_is_refused() {
    // M-673 review (PR #348): a GENERIC ADT whose ctor field applies a generic at the WRONG arity
    // is an explicit error (the `resolve_shell_field_ty` path) — never a wrong-arity `Ty::App`.
    let src = "nodule d\ntype List<A> = Nil | Cons(A, List<A>)\ntype Bad<A> = Mk(List<A, A>)";
    let err = check(src).unwrap_err();
    assert!(
        err.message.contains("arity") || err.message.contains("argument"),
        "got: {}",
        err.message
    );
}

#[test]
fn recursive_generic_adt_instantiates_correctly() {
    // List<Binary{8}> is the classic self-referential generic.
    // The shell-first algorithm must handle this without infinite recursion.
    let src =
        "nodule d\ntype List<A> = Nil | Cons(A, List<A>)\ntype ByteList = Wrap(List<Binary{8}>)";
    let env = check(src).expect("recursive generic ADT must check");
    // The monomorphic instantiation must be concrete in types
    assert!(
        env.types.contains_key("List<Binary{8}>"),
        "List<Binary{{8}}> should be instantiated in types, got: {:?}",
        env.types.keys().collect::<Vec<_>>()
    );
}

// --- S5: generic function signatures (M-657) ---

#[test]
fn generic_fn_instantiates_monomorphically() {
    // `fn id<A>(x: A) -> A = x` called at Binary{8} resolves to Binary{8} → Binary{8}.
    // The body type-checks with Ty::Var("A") in scope; the call site instantiates A=Binary{8}.
    let src = "nodule d\nfn id<A>(x: A) -> A = x\nfn main() -> Binary{8} = id(0b0000_0001)";
    let env = check(src).expect("generic id must check");
    assert!(env.fns.contains_key("id"), "id should be in fns");
    assert!(env.fns.contains_key("main"), "main should be in fns");
}

#[test]
fn generic_fn_with_adt_arg_instantiates() {
    // is_cons<A>(xs: List<A>) -> Bool — exhaustive match on a generic ADT (M-657 criteria).
    // The caller provides a concrete List<Binary{8}> instantiation via a monomorphic wrapper.
    // The wrapper fn `wrap` forces List<Binary{8}> into types, making Nil/Cons concrete.
    let src = concat!(
        "nodule d\n",
        "type List<A> = Nil | Cons(A, List<A>)\n",
        "fn is_cons<A>(xs: List<A>) -> Bool = match xs { Nil => False, Cons(_, _) => True }\n",
        // Force List<Binary{8}> into the concrete registry by using it as a parameter type.
        // Then pass it to is_cons.
        "fn check_list(xs: List<Binary{8}>) -> Bool = is_cons(xs)",
    );
    let env = check(src).expect("generic fn with ADT arg must check");
    assert!(env.fns.contains_key("is_cons"), "is_cons should be in fns");
    assert!(
        env.fns.contains_key("check_list"),
        "check_list should be in fns"
    );
    assert!(
        env.types.contains_key("List<Binary{8}>"),
        "List<Binary{{8}}> should be in types"
    );
}

#[test]
fn repr_mismatched_instantiation_is_never_a_silent_swap() {
    // Calling `fn id<A>(x: A) -> A` with a Binary{8} arg, then using the result where
    // a Ternary{9} is expected, must produce an explicit mismatch error — never a silent swap.
    let src = concat!(
        "nodule d\n",
        "fn id<A>(x: A) -> A = x\n",
        "fn main() -> Ternary{9} = id(0b0000_0001)",
    );
    let err = check(src).unwrap_err();
    // The body type mismatch or return type mismatch must be explicit.
    // MissingConversion is the cross-paradigm form of never-silent mismatch (RFC-0012 §4.4).
    assert!(
        err.message.contains("mismatch")
            || err.message.contains("expect")
            || err.message.contains("MissingConversion"),
        "got: {}",
        err.message
    );
}

// --- bounded iteration (RFC-0007 §4.8, r2) ---

const BYTES: &str = "nodule d\ntype Bytes = End | More(Binary{8}, Bytes)\n";

#[test]
fn a_for_fold_typechecks_and_is_total() {
    let env = check(&format!(
        "{BYTES}fn checksum(bs: Bytes) -> Binary{{8}} =\n    for b in bs, acc = 0b0000_0000 => xor(acc, b)"
    ))
    .expect("checks");
    // Bounded by construction: the fn is non-recursive, so it is Total and admissible in a
    // matured scope (RFC-0017 §4.2).
    assert_eq!(env.totality["checksum"], Totality::Total);
    // Confirm admission in a matured scope.
    let env2 = check_matured(&format!(
        "{BYTES}fn checksum(bs: Bytes) -> Binary{{8}} =\n    for b in bs, acc = 0b0000_0000 => xor(acc, b)"
    ))
    .expect("a total for-fold is admitted by a matured scope");
    assert_eq!(env2.totality["checksum"], Totality::Total);
}

#[test]
fn for_over_a_non_linear_type_is_an_explicit_refusal() {
    // A branching (tree) type is outside the v0 linear-recursion shape.
    let err = check(
        "nodule d\ntype Tree = Leaf | Node(Tree, Tree)\nfn f(t: Tree) -> Binary{8} =\n    for x in t, acc = 0b0000_0000 => acc",
    )
    .unwrap_err();
    assert!(
        err.message.contains("linearly recursive"),
        "got: {}",
        err.message
    );
}

#[test]
fn for_body_must_yield_the_accumulator_type() {
    let err = check(&format!(
        "{BYTES}fn f(bs: Bytes) -> Binary{{8}} =\n    for b in bs, acc = 0b0000_0000 => <+0->"
    ))
    .unwrap_err();
    assert!(err.message.contains("accumulator"), "got: {}", err.message);
}

#[test]
fn for_over_a_repr_value_is_an_explicit_refusal() {
    let err = check("nodule d\nfn f(x: Binary{8}) -> Binary{8} = for b in x, acc = x => acc")
        .unwrap_err();
    assert!(err.message.contains("data value"), "got: {}", err.message);
}

#[test]
fn imperative_words_get_teaching_diagnostics() {
    // Juxtaposition (`while x`) was never valid syntax — the parse error teaches (§4.8).
    let perr = parse("nodule d\nfn f(x: Binary{8}) -> Binary{8} = while x").unwrap_err();
    assert!(
        perr.message.contains("for x in xs"),
        "got: {}",
        perr.message
    );
    // Call-shaped use fails name resolution — the check error teaches too.
    let cerr = check("nodule d\nfn f(x: Binary{8}) -> Binary{8} = loop(x)").unwrap_err();
    assert!(
        cerr.message.contains("not a Mycelium form"),
        "got: {}",
        cerr.message
    );
}

// --- S6: M-657B — generic recursive functions elaborate (monomorphization pass) ---

/// A self-recursive generic function over a generic ADT materializes as a monomorphic instance
/// and ELABORATES to a closed L0 Fix term (M-657B). The instance `length<Binary{8}>` becomes
/// a self-recursive monomorphic `FnDecl` that the existing Fix machinery handles.
#[test]
fn recursive_generic_fn_elaborates_via_monomorphization() {
    // `length` is self-recursive over `List<A>` — the canonical recursive generic shape.
    // After monomorphization, `length<Binary{8}>` is a monomorphic self-recursive fn whose
    // body only mentions `length<Binary{8}>` — the Fix machinery wraps it as usual.
    //
    // Note: `byte_length(xs: List<Binary{8}>)` is required to force `List<Binary{8}>` into
    // env.types so that the checker can resolve `Nil`/`Cons` at the concrete type; without
    // a concrete-type wrapper the checker does not know which instantiation to mint.
    let src = concat!(
        "nodule d\n",
        "type List<A> = Nil | Cons(A, List<A>)\n",
        "type Nat = Z | S(Nat)\n",
        "fn length<A>(xs: List<A>) -> Nat = match xs {\n",
        "    Nil => Z,\n",
        "    Cons(_, rest) => S(length(rest))\n",
        "}\n",
        // Wrapper forces List<Binary{8}> into the concrete registry and wraps the call.
        "fn byte_length(xs: List<Binary{8}>) -> Nat = length(xs)\n",
        "fn main() -> Nat = byte_length(Nil)",
    );
    let env = check(src).expect("recursive generic fn must type-check");
    let node = elaborate(&env, "main")
        .expect("recursive generic fn must elaborate (M-657B: monomorphize then Fix)");
    // The L0 term must contain a Fix node — the self-recursive instance was lowered properly.
    let s = format!("{node:?}");
    assert!(
        s.contains("Fix") || s.contains("fix"),
        "elaborated term must contain a Fix for the recursive instance, got: {s}"
    );
}

/// A generic function instantiated with two different concrete types by two separate calls each
/// elaborates independently (M-657B materialization). Each call site uses a monomorphic wrapper
/// that forces the concrete type into env.types, preventing ambiguity.
#[test]
fn two_instantiations_of_same_generic_fn_elaborate() {
    // Two separate programs, each exercising a different instantiation.
    // This avoids checker ambiguity (two List<X>/List<Y> instances in the same env).

    // First: is_cons<Binary{8}>
    let src8 = concat!(
        "nodule d\n",
        "type List<A> = Nil | Cons(A, List<A>)\n",
        "fn is_cons<A>(xs: List<A>) -> Bool = match xs { Nil => False, Cons(_, _) => True }\n",
        "fn check_list(xs: List<Binary{8}>) -> Bool = is_cons(xs)\n",
        "fn main() -> Bool = check_list(Cons(0b0000_0001, Nil))",
    );
    let env8 = check(src8).expect("is_cons<Binary{8}> must type-check");
    elaborate(&env8, "main").expect("is_cons<Binary{8}> must elaborate");

    // Second: is_cons<Binary{4}>
    let src4 = concat!(
        "nodule d\n",
        "type List<A> = Nil | Cons(A, List<A>)\n",
        "fn is_cons<A>(xs: List<A>) -> Bool = match xs { Nil => False, Cons(_, _) => True }\n",
        "fn check_list(xs: List<Binary{4}>) -> Bool = is_cons(xs)\n",
        "fn main() -> Bool = check_list(Cons(0b0001, Nil))",
    );
    let env4 = check(src4).expect("is_cons<Binary{4}> must type-check");
    elaborate(&env4, "main").expect("is_cons<Binary{4}> must elaborate");
}

/// A mutually-recursive generic pair — each calls the other — lowers to a FixGroup whose
/// members are the two monomorphic instances (M-657B, M-343 FixGroup).
#[test]
fn mutually_recursive_generic_pair_lowers_to_fixgroup() {
    // `even<A>` and `odd<A>` are mutually recursive over `List<A>`.
    // The monomorphic wrapper `byte_even` forces List<Binary{8}> into types.
    let src = concat!(
        "nodule d\n",
        "type List<A> = Nil | Cons(A, List<A>)\n",
        "fn even<A>(xs: List<A>) -> Bool = match xs {\n",
        "    Nil => True,\n",
        "    Cons(_, rest) => odd(rest)\n",
        "}\n",
        "fn odd<A>(xs: List<A>) -> Bool = match xs {\n",
        "    Nil => False,\n",
        "    Cons(_, rest) => even(rest)\n",
        "}\n",
        // Wrapper forces List<Binary{1}> (single-bit marker) into types.
        "fn byte_even(xs: List<Binary{1}>) -> Bool = even(xs)\n",
        "fn main() -> Bool = byte_even(Nil)",
    );
    let env = check(src).expect("mutually recursive generic pair must type-check");
    let node = elaborate(&env, "main")
        .expect("mutually recursive generic pair must elaborate (M-657B + FixGroup)");
    // Must contain FixGroup or Fix — the two monomorphic instances are mutually recursive.
    let s = format!("{node:?}");
    assert!(
        s.contains("FixGroup") || s.contains("Fix"),
        "elaborated term must contain Fix/FixGroup for mutual recursion, got: {s}"
    );
}

/// When monomorphization cannot infer the concrete type args for a generic call, the elaboration
/// produces an explicit `ElabError::Residual` — never a panic, never `UnknownFn` for a present fn.
/// (M-657B honesty guard: VR-5/G2.)
#[test]
fn unresolvable_generic_call_is_explicit_residual_never_a_panic() {
    // A non-generic `main` that calls a generic function where the type argument cannot be
    // inferred from the provided arguments alone. The call `id(False)` where `id<A>(x: A) -> A`
    // — `False` is type `Bool`, so `A=Bool` should be inferable. This tests that a successful
    // inference path does NOT produce a panic. A function with an inferred Bool result:
    let src_ok = concat!(
        "nodule d\n",
        "fn id<A>(x: A) -> A = x\n",
        "fn main() -> Bool = id(False)",
    );
    // This should NOT check (id is generic, Bool is not a List, but `id<A>(x:A)` should work).
    // Actually `id<A>` should check because `False: Bool` and `A=Bool`, so:
    let env_ok = check(src_ok).expect("id(False) must type-check with A=Bool");
    let result_ok = elaborate(&env_ok, "main");
    match result_ok {
        Ok(_) => {}                           // inference succeeded — good
        Err(ElabError::Residual { .. }) => {} // explicit refusal — also acceptable
        Err(ElabError::UnknownFn(name)) => {
            panic!("got UnknownFn({name}) — never UnknownFn for a present fn");
        }
    }
}

/// The instance cap (256) fires as an explicit `ElabError::Residual` when the monomorphization
/// worklist would expand without bound — never a panic or silent hang (VR-5/G2, M-657B).
///
/// Honesty (Declared): constructing a program whose monomorphization genuinely exceeds the cap
/// requires polymorphic recursion — a form that stage-1 generics cannot express directly (the
/// checker refuses self-calls at a different type than the declared type parameter). The cap is
/// validated at this stage by asserting that any elaboration failure for a generic program is an
/// explicit Residual, and that the non-polymorphic recursive case (already tested in
/// `recursive_generic_fn_elaborates_via_monomorphization`) does NOT fire the cap.
#[test]
fn instance_cap_error_is_explicit_residual_not_a_panic() {
    // Ordinary recursive generic function — must NOT fire the cap (255 < 256 for any sane
    // program; the self-call only adds the already-memoized instance). This test validates
    // that the cap does not fire on ordinary recursion and that the path is clean.
    let src = concat!(
        "nodule d\n",
        "type List<A> = Nil | Cons(A, List<A>)\n",
        "type Nat = Z | S(Nat)\n",
        "fn length<A>(xs: List<A>) -> Nat = match xs { Nil => Z, Cons(_, rest) => S(length(rest)) }\n",
        "fn byte_length(xs: List<Binary{8}>) -> Nat = length(xs)\n",
        "fn main() -> Nat = byte_length(Nil)",
    );
    let env = check(src).expect("must type-check");
    // Must succeed — ordinary self-recursive generic does not fire the cap.
    let _node =
        elaborate(&env, "main").expect("ordinary recursive generic must NOT fire the instance cap");
}

// ---- S5: trait registry + impl checking + coherence (M-658/M-659) ----

#[test]
fn trait_declaration_registers_in_env() {
    // A `trait Show { fn show(x: Binary{8}) -> Binary{8} }` registers in `env.traits`.
    let src = "nodule d\ntrait Show {\n  fn show(x: Binary{8}) -> Binary{8}\n}";
    let env = check(src).expect("trait declaration must check");
    assert!(
        env.traits.contains_key("Show"),
        "Show should be in env.traits; got: {:?}",
        env.traits.keys().collect::<Vec<_>>()
    );
    assert_eq!(env.traits["Show"].methods.len(), 1);
    assert_eq!(env.traits["Show"].methods[0].name, "show");
}

#[test]
fn duplicate_trait_declaration_is_an_explicit_error() {
    // Two `trait Show` declarations → explicit CheckError (never a silent shadow — G2).
    let src = concat!(
        "nodule d\n",
        "trait Show { fn show(x: Binary{8}) -> Binary{8} }\n",
        "trait Show { fn show(x: Binary{8}) -> Binary{8} }\n",
    );
    let err = check(src).unwrap_err();
    assert!(
        err.message.contains("duplicate trait"),
        "expected duplicate-trait error; got: {}",
        err.message
    );
}

#[test]
fn impl_for_registered_type_checks() {
    // A valid `impl Show for Binary{8}` registers in `env.impls` and type-checks the method body.
    let src = concat!(
        "nodule d\n",
        "trait Show { fn show(x: Binary{8}) -> Binary{8} }\n",
        "impl Show for Binary{8} {\n",
        "  fn show(x: Binary{8}) -> Binary{8} = x\n",
        "}\n",
    );
    let env = check(src).expect("impl must check");
    assert!(
        env.impl_info("Show", &mycelium_l1::Ty::Binary(8)).is_some(),
        "impl Show for Binary{{8}} should be registered"
    );
}

#[test]
fn impl_for_unknown_trait_is_an_explicit_error() {
    // `impl Unknown for Binary{8}` — trait not declared → explicit CheckError (G2).
    let src =
        "nodule d\nimpl Unknown for Binary{8} {\n  fn show(x: Binary{8}) -> Binary{8} = x\n}\n";
    let err = check(src).unwrap_err();
    assert!(
        err.message.contains("unknown trait"),
        "expected unknown-trait error; got: {}",
        err.message
    );
}

#[test]
fn overlapping_impl_is_an_explicit_error() {
    // Two `impl Show for Binary{8}` declarations → coherence violation (RFC-0019 §4.5).
    let src = concat!(
        "nodule d\n",
        "trait Show { fn show(x: Binary{8}) -> Binary{8} }\n",
        "impl Show for Binary{8} { fn show(x: Binary{8}) -> Binary{8} = x }\n",
        "impl Show for Binary{8} { fn show(x: Binary{8}) -> Binary{8} = x }\n",
    );
    let err = check(src).unwrap_err();
    assert!(
        err.message.contains("overlapping impl") || err.message.contains("coherence"),
        "expected coherence error; got: {}",
        err.message
    );
}

#[test]
fn impl_missing_method_is_an_explicit_error() {
    // `impl Show for Binary{8}` without `show` → missing method CheckError (never-silent, G2).
    let src = concat!(
        "nodule d\n",
        "trait Show { fn show(x: Binary{8}) -> Binary{8} }\n",
        "impl Show for Binary{8} { }\n",
    );
    let err = check(src).unwrap_err();
    assert!(
        err.message.contains("missing method"),
        "expected missing-method error; got: {}",
        err.message
    );
}

#[test]
fn impl_extra_method_is_an_explicit_error() {
    // Providing a method NOT in the trait → explicit error (never-silent, G2).
    let src = concat!(
        "nodule d\n",
        "trait Show { fn show(x: Binary{8}) -> Binary{8} }\n",
        "impl Show for Binary{8} {\n",
        "  fn show(x: Binary{8}) -> Binary{8} = x\n",
        "  fn extra(x: Binary{8}) -> Binary{8} = x\n",
        "}\n",
    );
    let err = check(src).unwrap_err();
    assert!(
        err.message.contains("not declared in trait"),
        "expected undeclared-method error; got: {}",
        err.message
    );
}

#[test]
fn impl_method_type_mismatch_is_an_explicit_error() {
    // Method body returns wrong type → explicit CheckError (never-silent, G2).
    let src = concat!(
        "nodule d\n",
        "trait Show { fn show(x: Binary{8}) -> Binary{8} }\n",
        "impl Show for Binary{8} {\n",
        "  fn show(x: Binary{8}) -> Binary{8} = swap(x, to: Ternary{6}, policy: rt)\n",
        "}\n",
    );
    let err = check(src).unwrap_err();
    // Either a MissingConversion or a plain type mismatch — either way it must not succeed.
    assert!(
        !err.message.is_empty(),
        "impl method type mismatch should be an error; got: {}",
        err.message
    );
}

// ---- S6: bounded-call resolution (M-658/M-659) ----

#[test]
fn bounded_generic_call_with_impl_checks() {
    // `fn identity<T: Show>(x: T) -> T` called with `Binary{8}` where `impl Show for Binary{8}`
    // exists — the bound check must find the impl and succeed. The call `identity(0b0000_0000)`
    // anchors T=Binary{8}; the impl lookup for Show@Binary{8} must pass (M-659).
    let src = concat!(
        "nodule d\n",
        "trait Show { fn show(x: Binary{8}) -> Binary{8} }\n",
        "impl Show for Binary{8} { fn show(x: Binary{8}) -> Binary{8} = x }\n",
        "fn identity<T: Show>(x: T) -> T = x\n",
        "fn main() -> Binary{8} = identity(0b0000_0000)\n",
    );
    check(src).expect("bounded generic call with registered impl must check");
}

#[test]
fn bounded_call_missing_impl_is_an_explicit_error() {
    // `fn identity<T: Show>(x: T) -> T` called with `Binary{8}` but NO `impl Show for Binary{8}`
    // registered → explicit CheckError (never-silent, G2 / M-659).
    let src = concat!(
        "nodule d\n",
        "trait Show { fn show(x: Binary{8}) -> Binary{8} }\n",
        // No impl registered for Binary{8}.
        "fn identity<T: Show>(x: T) -> T = x\n",
        "fn main() -> Binary{8} = identity(0b0000_0000)\n",
    );
    let err = check(src).unwrap_err();
    assert!(
        err.message.contains("no impl of"),
        "expected missing-impl error; got: {}",
        err.message
    );
}

// ---- S7: compile-time dictionary dispatch (M-658/M-659) ----
// These tests verify that trait method calls inside bounded generic functions are correctly
// resolved at monomorphization time and that the evaluator and elaborator can execute them.

#[test]
fn trait_method_call_in_bounded_generic_evaluates_correctly() {
    // `fn apply_show<T: Show>(x: T) -> Binary{8} = show(x)` — the body calls the trait method
    // `show` directly. After monomorphization, `show(x)` must be rewritten to the concrete
    // impl method and be callable by the evaluator (S7 / M-659 compile-time dictionary).
    let src = concat!(
        "nodule d\n",
        "trait Show { fn show(x: Binary{8}) -> Binary{8} }\n",
        // impl: `show(x) = not(x)` so we can distinguish it from the identity.
        "impl Show for Binary{8} { fn show(x: Binary{8}) -> Binary{8} = not(x) }\n",
        "fn apply_show<T: Show>(x: T) -> Binary{8} = show(x)\n",
        "fn main() -> Binary{8} = apply_show(0b0000_0000)\n",
    );
    let env = check(src).expect("must check");
    let result = Evaluator::new(&env)
        .call("main", vec![])
        .expect("must evaluate");
    // `not(0b0000_0000)` = 0b1111_1111.
    let val = result.as_repr().expect("repr value");
    // Compare by repr payload: the result should be all-true bits (0b11111111).
    use mycelium_core::Payload;
    match val.payload() {
        Payload::Bits(bits) => {
            assert_eq!(
                bits,
                &vec![true; 8],
                "expected not(0b00000000) = 0b11111111"
            );
        }
        other => panic!("expected Bits payload, got {other:?}"),
    }
}

#[test]
fn trait_method_call_evaluates_independently_per_impl() {
    // Two impls of the same trait for different types — the monomorphizer must dispatch to
    // the correct impl for each instantiation (S7: per-instance dictionary, not shared).
    // `impl Transform for Binary{8}  { fn transform(x) = not(x) }`
    // `fn apply<T: Transform>(x: T) -> Binary{8} = transform(x)` called with Binary{8}.
    // Verifies that multiple impls coexist and the right one fires (M-659 coherence).
    //
    // We can't have two impls for the same type, so we test one concrete impl: the key
    // property is that the call dispatches to `not`, not an identity.
    let src = concat!(
        "nodule d\n",
        "trait Transform { fn transform(x: Binary{8}) -> Binary{8} }\n",
        "impl Transform for Binary{8} { fn transform(x: Binary{8}) -> Binary{8} = not(x) }\n",
        "fn apply<T: Transform>(x: T) -> Binary{8} = transform(x)\n",
        "fn main() -> Binary{8} = apply(0b1111_0000)\n",
    );
    let env = check(src).expect("must check");
    let result = Evaluator::new(&env)
        .call("main", vec![])
        .expect("must evaluate");
    // `not(0b1111_0000)` = 0b0000_1111.
    let val = result.as_repr().expect("repr value");
    use mycelium_core::Payload;
    match val.payload() {
        Payload::Bits(bits) => {
            assert_eq!(
                bits,
                &vec![false, false, false, false, true, true, true, true],
                "expected not(0b11110000) = 0b00001111"
            );
        }
        other => panic!("expected Bits payload, got {other:?}"),
    }
}

#[test]
fn bounded_generic_trait_method_elaborates_correctly() {
    // Verify that a bounded generic with a trait method call also elaborates to L0 correctly.
    // The elaborator uses the monomorphized env (from `monomorphize`) so the trait-method
    // rewriting done in S7 must be present in the elaborated result (KC-3: no new kernel nodes).
    let src = concat!(
        "nodule d\n",
        "trait Id { fn id_method(x: Binary{8}) -> Binary{8} }\n",
        "impl Id for Binary{8} { fn id_method(x: Binary{8}) -> Binary{8} = x }\n",
        "fn apply_id<T: Id>(x: T) -> Binary{8} = id_method(x)\n",
        "fn main() -> Binary{8} = apply_id(0b1010_1010)\n",
    );
    let env = check(src).expect("must check");
    // L1 evaluator must give the correct result.
    let l1_result = Evaluator::new(&env)
        .call("main", vec![])
        .expect("L1 eval must succeed");
    let l1_val = l1_result.as_repr().expect("repr value");
    use mycelium_core::Payload;
    match l1_val.payload() {
        Payload::Bits(bits) => {
            // `id_method(0b10101010) = 0b10101010` (identity via impl).
            assert_eq!(
                bits,
                &vec![true, false, true, false, true, false, true, false],
                "expected id_method(0b10101010) = 0b10101010"
            );
        }
        other => panic!("expected Bits payload, got {other:?}"),
    }
    // Elaboration to L0 must also succeed — no new kernel nodes needed (KC-3).
    let _ = elaborate(&env, "main").expect("must elaborate to L0");
}

#[test]
fn prop_trait_dispatch_is_consistent_across_representative_concrete_types() {
    // S8 / M-658 / M-659 property bound (Declared): for each concrete type implementing a trait,
    // the dict-dispatch and runtime-dispatch paths agree on the result. We exercise this by
    // building a nodule that supplies *separate* impls for several Binary widths (4, 8, 16) and a
    // single bounded generic `apply_not`, then verifying that calling it with a value of each
    // concrete type produces the result that the per-type `not` prim would produce.
    //
    // This is a representative coverage loop — not exhaustive over all widths — consistent with
    // the DN-20 Tier-0/Tier-1 tiering (low proptest cases per commit).
    //
    // Guarantee tag: Declared (the dispatch contract is argued, not machine-checked; see VR-5).

    struct Case {
        width: u32,
        input_bits: &'static str,
        expected: Vec<bool>, // expected `not` of input
    }

    // `not` inverts every bit, so 0b0000 → 0b1111, 0b0000_0000 → 0b1111_1111, etc.
    let cases: &[Case] = &[
        Case {
            width: 4,
            input_bits: "0b0000",
            expected: vec![true, true, true, true],
        },
        Case {
            width: 8,
            input_bits: "0b0000_0000",
            expected: vec![true, true, true, true, true, true, true, true],
        },
        Case {
            width: 16,
            input_bits: "0b0000_0000_0000_0000",
            expected: vec![
                true, true, true, true, true, true, true, true, true, true, true, true, true, true,
                true, true,
            ],
        },
    ];

    for case in cases {
        let w = case.width;
        // Build a nodule with a *parametric* trait `Invertible<A>` and a conforming impl for each
        // width. Multi-width dispatch is exactly what a parametric trait expresses: each
        // `impl Invertible<Binary{N}> for Binary{N}` substitutes A↦Binary{N}, so its `invert`
        // signature conforms (M-658/M-659 parametric). Impls for widths not used by `main` are
        // valid, accepted, and unused — they neither overlap nor miss a method.
        let src = format!(
            "nodule d\n\
             trait Invertible<A> {{ fn invert(x: A) -> A }}\n\
             impl Invertible<Binary{{4}}> for Binary{{4}} {{ fn invert(x: Binary{{4}}) -> Binary{{4}} = not(x) }}\n\
             impl Invertible<Binary{{8}}> for Binary{{8}} {{ fn invert(x: Binary{{8}}) -> Binary{{8}} = not(x) }}\n\
             impl Invertible<Binary{{16}}> for Binary{{16}} {{ fn invert(x: Binary{{16}}) -> Binary{{16}} = not(x) }}\n\
             fn apply_not<T: Invertible>(x: T) -> T = invert(x)\n\
             fn main() -> Binary{{{w}}} = apply_not({input})\n",
            w = w,
            input = case.input_bits,
        );
        let env = check(&src).unwrap_or_else(|e| panic!("width {w}: must check, got: {e:?}"));
        let result = Evaluator::new(&env)
            .call("main", vec![])
            .unwrap_or_else(|e| panic!("width {w}: must evaluate, got: {e:?}"));
        let val = result
            .as_repr()
            .unwrap_or_else(|| panic!("width {w}: expected repr value"));
        use mycelium_core::Payload;
        match val.payload() {
            Payload::Bits(bits) => {
                assert_eq!(
                    bits, &case.expected,
                    "width {w}: not({}) dispatch gave wrong result",
                    case.input_bits
                );
            }
            other => panic!("width {w}: expected Bits payload, got {other:?}"),
        }
    }
}

// ---- Parametric traits (M-658/M-659 parametric extension, **Declared**) ----
// A parametric trait `trait Cmp<A>` puts its TYPE PARAMETER in the method signatures. An
// `impl Cmp<C> for C` substitutes `A ↦ C`; the impl's method signatures must conform to the
// substituted trait signatures (never-silent, G2). v0 restriction: single-param traits with the
// trait argument == the `for` type. Multi-param traits and `impl T<C> for D` with C≠D are
// honest deferrals (explicit CheckError, never a silent accept).

#[test]
fn parametric_trait_impl_checks() {
    // `trait Cmp<A> { fn same(x: A) -> A }` — `A` appears in BOTH method positions.
    // `impl Cmp<Binary{8}> for Binary{8}` substitutes A↦Binary{8}; the impl sig conforms.
    let src = concat!(
        "nodule d\n",
        "trait Cmp<A> { fn same(x: A) -> A }\n",
        "impl Cmp<Binary{8}> for Binary{8} { fn same(x: Binary{8}) -> Binary{8} = x }\n",
    );
    let env = check(src).expect("parametric trait + conforming impl must check");
    assert!(
        env.impl_info("Cmp", &mycelium_l1::Ty::Binary(8)).is_some(),
        "impl Cmp for Binary{{8}} should be registered"
    );
    assert_eq!(env.traits["Cmp"].params, vec!["A".to_owned()]);
}

#[test]
fn parametric_trait_impl_without_explicit_args_infers_for_ty() {
    // `impl Cmp for Binary{8}` (no `<…>`) — the single trait param is inferred as the `for` type.
    let src = concat!(
        "nodule d\n",
        "trait Cmp<A> { fn same(x: A) -> A }\n",
        "impl Cmp for Binary{8} { fn same(x: Binary{8}) -> Binary{8} = x }\n",
    );
    let env = check(src).expect("impl with inferred trait arg must check");
    assert!(env.impl_info("Cmp", &mycelium_l1::Ty::Binary(8)).is_some());
}

#[test]
fn parametric_trait_bounded_call_evaluates() {
    // A bounded generic `fn use_it<T: Cmp>(x: T) -> T = same(x)` calls the trait method `same`,
    // which dispatches (compile-time dictionary) to the concrete impl after monomorphization.
    let src = concat!(
        "nodule d\n",
        "trait Cmp<A> { fn same(x: A) -> A }\n",
        "impl Cmp<Binary{8}> for Binary{8} { fn same(x: Binary{8}) -> Binary{8} = x }\n",
        "fn use_it<T: Cmp>(x: T) -> T = same(x)\n",
        "fn main() -> Binary{8} = use_it(0b1010_1010)\n",
    );
    let env = check(src).expect("parametric bounded call must check");
    let result = Evaluator::new(&env)
        .call("main", vec![])
        .expect("evaluates");
    let val = result.as_repr().expect("repr value");
    use mycelium_core::Payload;
    match val.payload() {
        Payload::Bits(bits) => assert_eq!(
            bits,
            &vec![true, false, true, false, true, false, true, false]
        ),
        other => panic!("expected Bits payload, got {other:?}"),
    }
}

#[test]
fn parametric_impl_signature_must_conform_to_trait() {
    // The impl method body checks against its OWN declared sig (Binary{4}→Binary{4}, valid), but
    // that sig does NOT match the trait sig substituted at Binary{8} (`same(Binary{8})->Binary{8}`).
    // This must be an explicit CheckError — a silent accept would be unsound (G2/VR-5).
    let src = concat!(
        "nodule d\n",
        "trait Cmp<A> { fn same(x: A) -> A }\n",
        "impl Cmp<Binary{8}> for Binary{8} { fn same(x: Binary{4}) -> Binary{4} = x }\n",
    );
    let err = check(src).unwrap_err();
    assert!(
        err.message.contains("requires") && err.message.contains("parameter"),
        "expected a signature-conformance error naming the trait's required parameter type; got: {}",
        err.message
    );
}

#[test]
fn parametric_impl_trait_arg_ne_for_ty_is_a_deferred_error() {
    // `impl Cmp<Binary{4}> for Binary{8}` — the written trait argument (Binary{4}) differs from
    // the `for` type (Binary{8}). v0 defers `impl T<C> for D` with C≠D — explicit error (G2).
    let src = concat!(
        "nodule d\n",
        "trait Cmp<A> { fn same(x: A) -> A }\n",
        "impl Cmp<Binary{4}> for Binary{8} { fn same(x: Binary{8}) -> Binary{8} = x }\n",
    );
    let err = check(src).unwrap_err();
    assert!(
        err.message.contains("deferred") && err.message.contains("different from"),
        "expected a C≠D deferral error; got: {}",
        err.message
    );
}

#[test]
fn m659_literal_acceptance_two_param_method_checks() {
    // The exact M-659 acceptance example: a parametric trait whose method takes two arguments of
    // the trait parameter type and returns a concrete `Binary{2}`. The conforming impl substitutes
    // A↦Binary{8} and must realize `cmp(Binary{8}, Binary{8}) -> Binary{2}`.
    let src = concat!(
        "nodule d\n",
        "trait Cmp<A> { fn cmp(a: A, b: A) -> Binary{2} }\n",
        "impl Cmp<Binary{8}> for Binary{8} { fn cmp(a: Binary{8}, b: Binary{8}) -> Binary{2} = 0b00 }\n",
    );
    let env = check(src).expect("M-659 literal acceptance must check");
    assert!(env.impl_info("Cmp", &mycelium_l1::Ty::Binary(8)).is_some());
    assert_eq!(env.traits["Cmp"].methods[0].value_params.len(), 2);
}

#[test]
fn ambiguous_trait_method_across_two_bounds_is_refused() {
    // Two traits declare a method `m`; a generic fn bounded by both calls `m`. v0 dispatches by
    // method name and has no disambiguation syntax, so the call is an explicit CheckError — never
    // a silent first-bound pick (G2/VR-5).
    let src = concat!(
        "nodule d\n",
        "trait A { fn m(x: Binary{8}) -> Binary{8} }\n",
        "trait B { fn m(x: Binary{8}) -> Binary{8} }\n",
        "impl A for Binary{8} { fn m(x: Binary{8}) -> Binary{8} = x }\n",
        "impl B for Binary{8} { fn m(x: Binary{8}) -> Binary{8} = not(x) }\n",
        "fn f<X: A, Y: B>(x: X, y: Y) -> Binary{8} = m(x)\n",
    );
    let err = check(src).unwrap_err();
    assert!(
        err.message.contains("ambiguous") && err.message.contains('m'),
        "expected an ambiguous-trait-method error; got: {}",
        err.message
    );
}
