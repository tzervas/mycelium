//! L1 static checking (RFC-0007 §4.4/§4.5): the monomorphic typechecker, the structural totality
//! checker, and the scope-quantified `matured ⟹ total` gate (RFC-0017 §4.2). Every refusal is
//! an explicit `CheckError`.

use mycelium_l1::{check_nodule, check_nodule_matured, parse, Totality};

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
fn wild_is_denied_outside_a_std_sys_nodule() {
    // M-661: a `wild` block in a non-`@std-sys` nodule is a HARD `CheckError` (the audited FFI floor
    // lives only in `std-sys` — RFC-0016 §8-Q6 / LR-9; never a silent escape — G2). Not a lint.
    let src = "nodule d\nfn f(x: Binary{8}) -> Binary{8} !{ffi} = wild { x }";
    let err = check(src).unwrap_err();
    assert!(
        err.message.contains("wild") && err.message.contains("std-sys"),
        "the refusal must point at the missing `@std-sys` context, got: {}",
        err.message
    );
}

// --- M-661: the `wild` block — the audited FFI floor (RFC-0016 §8-Q6; LR-9/S6; ADR-014) ----------
// Settled design: a `wild` block is legal ONLY inside a `@std-sys` nodule (else a hard refusal, not a
// lint — G2). Its body is the trusted/opaque FFI escape — NOT recursively type-checked (audited, not
// verified — VR-5). It needs an EXPECTED type (synthesis refuses with "ascribe"). `wild` is the `ffi`
// effect source: a fn containing it must declare `!{ffi}` (M-660 coverage). Execution is STAGED (the
// elaborator lowers it to an explicit `Residual` — no FFI host in v0). Guarantee on the gate:
// `Declared` (a structural + audited context gate, not a theorem).

#[test]
fn a_wild_block_in_a_std_sys_nodule_type_checks_with_an_opaque_body() {
    // The acceptance: a `@std-sys` nodule with `fn read_byte() -> Binary{8} !{ffi} = wild { … }`
    // type-checks. The body (`foreign_read()`) is opaque — `foreign_read` is NOT a declared fn, yet
    // the block still checks, because the body is the trusted FFI escape (not recursively checked).
    let env = check(
        "nodule std.sys.fs @std-sys\n\
         fn read_byte() -> Binary{8} !{ffi} = wild { foreign_read() }",
    )
    .expect("a wild block in a @std-sys nodule, with !{ffi} declared, type-checks (opaque body)");
    // The fn is recorded with its `ffi` effect (EXPLAIN / future wiring).
    assert_eq!(
        env.fn_decl("read_byte").expect("fn read_byte").sig.effects,
        vec!["ffi".to_owned()]
    );
}

#[test]
fn a_wild_body_is_not_recursively_type_checked() {
    // The body is the trusted/opaque escape (audited, not verified — VR-5/ADR-014): even a body that
    // would NOT type-check on its own (calls an unknown name, with a deliberately wrong shape) is
    // accepted, because the checker does not descend into it — it conforms to the expected type by
    // ascription. This is the load-bearing "opaque body" property.
    let env = check(
        "nodule std.sys.x @std-sys\n\
         fn f() -> Binary{8} !{ffi} = wild { totally_undefined_ffi(does, not, exist) }",
    )
    .expect("the wild body is opaque — not recursively checked, so an unknown callee is fine");
    assert!(env.fn_decl("f").is_some());
}

#[test]
fn a_wild_in_a_std_sys_nodule_without_declaring_ffi_is_a_coverage_refusal() {
    // M-661 × M-660: `wild` performs the `ffi` effect, so a fn containing it must declare `!{ffi}`.
    // Here the nodule IS `@std-sys` (so the context gate passes), but the fn omits `!{ffi}` — the
    // effect-coverage pass refuses it, naming `ffi` and framing it as an under-declaration (G2).
    let err = check(
        "nodule std.sys.fs @std-sys\n\
         fn read_byte() -> Binary{8} = wild { foreign_read() }",
    )
    .expect_err("a wild block whose enclosing fn does not declare !{ffi} must be refused");
    assert!(
        err.message.contains("ffi") && err.message.contains("does not declare"),
        "the refusal must name the undeclared `ffi` effect, got: {}",
        err.message
    );
}

#[test]
fn a_wild_in_synthesis_position_demands_an_ascription() {
    // The body takes its type from context (it is not synthesized). In a synthesis position — here a
    // `let` bound with no annotation, whose bound expr must self-synthesize — the checker refuses with
    // an explicit "ascribe" message, never a guessed type (G2).
    let err = check(
        "nodule std.sys.x @std-sys\n\
         fn f() -> Binary{8} !{ffi} = let v = wild { foreign() } in v",
    )
    .expect_err("a wild block in a synthesis position must demand an ascription");
    // The message says "Ascribe …" (capitalized at the sentence start) — match case-insensitively.
    let lower = err.message.to_lowercase();
    assert!(
        lower.contains("ascribe") && lower.contains("wild"),
        "the refusal must ask for an ascription of the wild block's type, got: {}",
        err.message
    );
}

#[test]
fn an_ascribed_wild_in_a_let_bound_position_type_checks() {
    // Dual of the above: with the `let` binding ANNOTATED (`let v: Binary{8} = …`), the bound has a
    // known expected type and the opaque `wild` body conforms to it — the program checks. (The
    // annotation is on the binding, the surface form the bidirectional checker threads as `expected`.)
    let env = check(
        "nodule std.sys.x @std-sys\n\
         fn f() -> Binary{8} !{ffi} = let v: Binary{8} = wild { foreign() } in v",
    )
    .expect("an annotated let-binding supplies the wild bound's type and checks");
    assert!(env.fn_decl("f").is_some());
}

#[test]
fn over_declaring_ffi_without_a_wild_block_is_allowed() {
    // Symmetry with M-660 I5: declaring `!{ffi}` is a contract — a fn may reserve it without (yet)
    // containing a `wild` block. A pure-bodied `@std-sys` fn declaring `!{ffi}` checks (over-decl OK).
    let env = check(
        "nodule std.sys.x @std-sys\n\
         fn f() -> Binary{8} !{ffi} = 0b00000000",
    )
    .expect("over-declaring `ffi` without a wild block is allowed (a declaration is a contract)");
    assert_eq!(
        env.fn_decl("f").expect("fn f").sig.effects,
        vec!["ffi".to_owned()]
    );
}

#[test]
fn a_wild_inside_an_impl_method_is_gated_by_the_nodule_std_sys_context() {
    // The `@std-sys` gate flows into impl-method bodies too: in a NON-`@std-sys` nodule a `wild`
    // inside an impl method is the same hard refusal as in a top-level fn (the context gate is the
    // nodule's, not the item's). This pins the impl-method threading of `std_sys`.
    let err = check(
        "nodule d\n\
         trait Ffi<A> { fn raw(x: A) -> A !{ffi} }\n\
         impl Ffi<Binary{8}> for Binary{8} { fn raw(x: Binary{8}) -> Binary{8} !{ffi} = wild { host(x) } }",
    )
    .expect_err("a wild inside an impl method of a non-@std-sys nodule must be refused");
    assert!(
        err.message.contains("wild") && err.message.contains("std-sys"),
        "the impl-method wild refusal must cite the missing `@std-sys` context, got: {}",
        err.message
    );
}

#[test]
fn a_wild_in_a_std_sys_nodule_is_staged_at_elaboration() {
    // Execution is STAGED: a `@std-sys` `wild` fn type-checks, but elaborating it to L0 is an explicit
    // `Residual` (no FFI host in v0 — M-661), never a fabricated artifact (G2). Mirrors M-657/659/660.
    let nodule = parse(
        "nodule std.sys.fs @std-sys\nfn read_byte() -> Binary{8} !{ffi} = wild { foreign_read() }",
    )
    .expect("parses");
    let env = check_nodule(&nodule).expect("type-checks");
    let err = mycelium_l1::elaborate(&env, "read_byte")
        .expect_err("a wild block has no L0 form in v0 — staged as Residual");
    let msg = err.to_string();
    assert!(
        msg.contains("staged") && (msg.contains("FFI") || msg.contains("wild")),
        "the elaboration refusal must frame wild/FFI as staged, got: {msg}"
    );
}

#[test]
fn the_std_sys_marker_is_parsed_off_the_header() {
    // The `@std-sys` marker is a parsed header attribute (not a naming convention): a nodule named
    // `std.sys.fs` WITHOUT the marker is NOT std-sys, and any nodule WITH the marker is — the name is
    // irrelevant. This pins that the gate keys on the marker, never on the path.
    let with_marker = parse("nodule anything.at.all @std-sys\nfn f() -> Binary{1} = 0b0")
        .expect("parses with marker");
    assert!(with_marker.std_sys, "the @std-sys marker must set std_sys");
    let no_marker =
        parse("nodule std.sys.fs\nfn f() -> Binary{1} = 0b0").expect("parses without marker");
    assert!(
        !no_marker.std_sys,
        "a `std.sys.*`-named nodule WITHOUT the marker is not std-sys (attribute, not convention)"
    );
    // Consequently a `wild` in the marker-less `std.sys.*` nodule is still refused.
    let err = check("nodule std.sys.fs\nfn f() -> Binary{8} !{ffi} = wild { x }")
        .expect_err("no marker ⇒ wild refused even under a std.sys.* name");
    assert!(err.message.contains("std-sys"), "got: {}", err.message);
}

// --- stage-1 unbounded parametric generics (RFC-0007 §11; M-657) -----------------------------
// The §4.4 "generics are a deferred error" posture is discharged: a generic *type* declaration and a
// generic *function* now type-check, instantiate, and (for the unbounded core) check honestly. What
// stays an explicit refusal: wrong arity, a representation-specific op on a type parameter (the
// Repr-polymorphism restriction, RFC-0019 §4.6), and an undetermined type parameter — never a guess.

const LIST: &str = "nodule d\ntype List<A> = Nil | Cons(A, List<A>)\n";

#[test]
fn a_generic_data_type_and_a_total_generic_fn_check() {
    // `is_empty` is total (covers both constructors), generic over `A`, and representation-agnostic.
    let env = check(&format!(
        "{LIST}fn is_empty<A>(xs: List<A>) -> Binary{{1}} = match xs {{ Nil => 0b1, Cons(_, _) => 0b0 }}"
    ))
    .expect("a generic data type + total generic fn check");
    assert_eq!(env.totality["is_empty"], Totality::Total);
}

#[test]
fn a_generic_fn_instantiates_at_a_concrete_type() {
    // `first_or<A>` returns `A`; the call site infers `A = Binary{8}` from the arguments and the
    // result type is `Binary{8}` — the never-guess instantiation of RFC-0007 §11.3.
    let env = check(&format!(
        "{LIST}fn first_or<A>(xs: List<A>, d: A) -> A = match xs {{ Nil => d, Cons(x, _) => x }}\n\
         fn main() -> Binary{{8}} = first_or(Cons(0b0000_0001, Nil), 0b0000_0000)"
    ))
    .expect("a generic fn instantiates at Binary{8}");
    assert_eq!(env.totality["main"], Totality::Total);
}

#[test]
fn the_wrong_type_argument_arity_is_explicit_never_a_guess() {
    // `Pair` takes two type arguments; applying it to one is a clean error (RFC-0007 §11.3), not a
    // silently-defaulted second argument.
    let src = "nodule d\ntype Pair<A, B> = MkPair(A, B)\n\
               fn f(p: Pair<Binary{8}>) -> Binary{8} = match p { MkPair(a, b) => a }";
    let err = check(src).unwrap_err();
    assert!(
        err.message.contains("type argument") && err.message.contains("got 1"),
        "expected an arity error, got: {}",
        err.message
    );
}

#[test]
fn a_representation_specific_op_on_a_type_parameter_is_refused() {
    // RFC-0019 §4.6 (unbounded case): a value of abstract type `A` is representation-opaque, so a
    // paradigm-specific prim (`and`, a `Binary` op) may not apply to it — refused, never a silent
    // coercion/swap (S1). The restriction falls out of the abstract-variable discipline.
    let src = format!("{LIST}fn bad<A>(x: A) -> A = and(x, x)");
    let err = check(&src).unwrap_err();
    assert!(
        err.message.contains("accept") || err.message.contains("and"),
        "expected a representation-op refusal, got: {}",
        err.message
    );
}

#[test]
fn an_undetermined_type_parameter_is_explicit_not_a_guess() {
    // `g<A>()` mentions `A` nowhere in its value parameters, so a call cannot determine it — an
    // explicit "does not determine it" error, never a guessed default (G2/VR-5).
    let src = "nodule d\nfn g<A>() -> Binary{1} = 0b1\nfn main() -> Binary{1} = g()";
    let err = check(src).unwrap_err();
    assert!(
        err.message.contains("does not determine"),
        "expected an undetermined-type-parameter error, got: {}",
        err.message
    );
}

#[test]
fn generic_instantiation_type_checks_across_widths_a_property_bound() {
    // Property (per instantiation bound, RFC-0007 §11.3): instantiating a generic at `Binary{n}` for
    // a range of widths `n` type-checks and yields a `Binary{n}` result — uniformly, never a guess.
    for n in [1u32, 2, 4, 8, 16, 32, 64, 128] {
        let src = format!(
            "{LIST}fn first_or<A>(xs: List<A>, d: A) -> A = match xs {{ Nil => d, Cons(x, _) => x }}\n\
             fn main() -> Binary{{{n}}} = first_or(Cons(0b{ones}, Nil), 0b{zeros})",
            ones = "1".repeat(n as usize),
            zeros = "0".repeat(n as usize),
        );
        let env = check(&src).unwrap_or_else(|e| panic!("n={n} should check: {e}"));
        assert_eq!(env.totality["main"], Totality::Total, "n={n}");
    }
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

// --- stage-1 traits + impls with coherence (RFC-0019 §4.1/§4.4/§4.5; RFC-0007 §12; M-659) --------
// The trait checker: trait/impl declarations type-check with coherence (global uniqueness + orphan
// rule); a bounded generic call requires a resolvable instance; dictionary-passing is *typed* in the
// checker but its L0 lowering is STAGED to a `Residual` (M-673). Every coherence / orphan /
// missing-method / undetermined / no-instance case is an explicit `CheckError` — never silent (G2).
// Guarantee tags on the checker entry points are `Declared` (a structural registry check, not a
// theorem; RFC-0019's coherence result is Declared-with-argument — VR-5).

/// `Cmp<A>` — a one-method, single-parameter trait used across the M-659 tests.
const CMP: &str = "nodule d\ntrait Cmp<A> { fn cmp(a: A, b: A) -> Binary{2} }\n";
/// `Cmp<A>` plus the canonical `impl Cmp<Binary{8}> for Binary{8}` (the M-659 acceptance instance).
const CMP_I8: &str = "nodule d\ntrait Cmp<A> { fn cmp(a: A, b: A) -> Binary{2} }\n\
     impl Cmp<Binary{8}> for Binary{8} { fn cmp(a: Binary{8}, b: Binary{8}) -> Binary{2} = 0b00 }\n";

#[test]
fn a_trait_and_impl_check() {
    // The M-659 acceptance: a single-parameter trait + a concrete instance with a total, simple
    // method body. The instance is registered and coherent.
    let env = check(CMP_I8).expect("a trait + impl check");
    assert!(env.trait_info("Cmp").is_some(), "trait registered");
    // Keyed by (trait, type-head) — `Binary{8}`'s head is "Binary".
    assert!(
        env.instance("Cmp", "Binary").is_some(),
        "instance registered under (Cmp, Binary)"
    );
}

#[test]
fn a_trait_method_sig_must_resolve() {
    // A trait method referencing an unknown type is an explicit refusal (the trait pass resolves
    // each method sig with the trait params in scope).
    let err = check("nodule d\ntrait T<A> { fn f(x: Nope) -> A }").unwrap_err();
    assert!(err.message.contains("unknown type"), "got: {}", err.message);
}

#[test]
fn a_bounded_generic_fn_checks_with_an_instance() {
    // `use_cmp<T: Cmp>` calls the trait method `cmp` through its bound (the dictionary is staged to
    // elaboration — M-673), and the call site resolves the instance `(Cmp, Binary)`.
    let env = check(&format!(
        "{CMP_I8}fn use_cmp<T: Cmp>(a: T, b: T) -> Binary{{2}} = cmp(a, b)\n\
         fn main() -> Binary{{2}} = use_cmp(0b0000_0001, 0b0000_0010)"
    ))
    .expect("a bounded generic fn checks with an instance");
    // The bounded fn is registered as generic.
    assert_eq!(env.fn_decl("use_cmp").unwrap().sig.params.len(), 1);
}

#[test]
fn a_bounded_generic_fn_without_an_instance_is_an_explicit_error() {
    // Same `use_cmp<T: Cmp>`, but no instance for the call's `Binary{8}` — the bound is unsatisfiable
    // at the call site, an explicit "no instance" refusal (never assumed — G2/VR-5).
    let err = check(&format!(
        "{CMP}fn use_cmp<T: Cmp>(a: T, b: T) -> Binary{{2}} = cmp(a, b)\n\
         fn main() -> Binary{{2}} = use_cmp(0b0000_0001, 0b0000_0010)"
    ))
    .unwrap_err();
    assert!(
        err.message.contains("no instance") && err.message.contains("Cmp"),
        "got: {}",
        err.message
    );
}

#[test]
fn a_duplicate_instance_on_the_same_head_is_a_coherence_error() {
    // Two instances on the same `(trait, type-head)` — even at different widths (`Binary{8}` and
    // `Binary{4}` share the head "Binary") — is a global-uniqueness / overlapping-instance violation
    // (RFC-0019 §4.5; the documented stage-1 head-granular over-rejection).
    let err = check(&format!(
        "{CMP_I8}impl Cmp<Binary{{4}}> for Binary{{4}} \
         {{ fn cmp(a: Binary{{4}}, b: Binary{{4}}) -> Binary{{2}} = 0b00 }}"
    ))
    .unwrap_err();
    assert!(
        err.message.contains("coherence") || err.message.contains("overlapping"),
        "got: {}",
        err.message
    );
}

#[test]
fn an_impl_missing_a_method_is_an_explicit_error() {
    let err = check(
        "nodule d\ntrait Two<A> { fn f(x: A) -> A\n fn g(x: A) -> A }\n\
         impl Two<Binary{8}> for Binary{8} { fn f(x: Binary{8}) -> Binary{8} = x }",
    )
    .unwrap_err();
    assert!(
        err.message.contains("missing method") && err.message.contains("g"),
        "got: {}",
        err.message
    );
}

#[test]
fn an_impl_with_an_extra_method_is_an_explicit_error() {
    let err = check(
        "nodule d\ntrait One<A> { fn f(x: A) -> A }\n\
         impl One<Binary{8}> for Binary{8} \
         { fn f(x: Binary{8}) -> Binary{8} = x\n fn h(x: Binary{8}) -> Binary{8} = x }",
    )
    .unwrap_err();
    assert!(
        err.message.contains("not in trait") && err.message.contains("h"),
        "got: {}",
        err.message
    );
}

#[test]
fn an_impl_method_with_the_wrong_signature_is_an_explicit_error() {
    // The method body's declared return (`Binary{4}`) disagrees with the trait's required return
    // (`Binary{2}` after substituting the impl's trait arg) — an explicit edge mismatch.
    let err = check(
        "nodule d\ntrait Cmp<A> { fn cmp(a: A, b: A) -> Binary{2} }\n\
         impl Cmp<Binary{8}> for Binary{8} \
         { fn cmp(a: Binary{8}, b: Binary{8}) -> Binary{4} = 0b0000 }",
    )
    .unwrap_err();
    assert!(
        err.message.contains("return") && err.message.contains("Binary{2}"),
        "got: {}",
        err.message
    );
}

#[test]
fn an_impl_for_an_unknown_trait_is_an_explicit_error() {
    let err = check(
        "nodule d\nimpl Nope<Binary{8}> for Binary{8} { fn f(x: Binary{8}) -> Binary{8} = x }",
    )
    .unwrap_err();
    assert!(
        err.message.contains("unknown trait"),
        "got: {}",
        err.message
    );
}

#[test]
fn an_impl_with_the_wrong_trait_arg_arity_is_an_explicit_error() {
    // `Cmp` takes one type argument; supplying two is a clean arity error (never a guess).
    let err = check(
        "nodule d\ntrait Cmp<A> { fn cmp(a: A, b: A) -> Binary{2} }\n\
         impl Cmp<Binary{8}, Binary{8}> for Binary{8} \
         { fn cmp(a: Binary{8}, b: Binary{8}) -> Binary{2} = 0b00 }",
    )
    .unwrap_err();
    assert!(
        err.message.contains("type argument"),
        "got: {}",
        err.message
    );
}

#[test]
fn a_concrete_trait_method_call_resolves_via_an_instance() {
    // An unqualified trait-method call at a concrete type (no bounded fn) types via the concrete
    // instance `(Cmp, Binary)`.
    let env = check(&format!(
        "{CMP_I8}fn direct() -> Binary{{2}} = cmp(0b0000_0001, 0b0000_0010)"
    ))
    .expect("a concrete trait-method call resolves via the instance");
    assert_eq!(env.totality["direct"], Totality::Total);
}

#[test]
fn a_concrete_trait_method_call_without_an_instance_is_an_explicit_error() {
    // `cmp` at `Ternary{2}` — no instance for that head — is an explicit refusal (never a guess).
    let err = check(&format!(
        "{CMP_I8}fn direct() -> Binary{{2}} = cmp(<00>, <00>)"
    ))
    .unwrap_err();
    assert!(err.message.contains("no instance"), "got: {}", err.message);
}

#[test]
fn an_ambiguous_trait_method_call_is_an_explicit_error_never_a_guess() {
    // The method name `m` is declared by two traits — with no qualified-call syntax in stage-1 this
    // is ambiguous, an explicit refusal, never a silent pick (RFC-0019 §4.4; G2/VR-5).
    let err = check(
        "nodule d\ntrait A1<X> { fn m(x: X) -> X }\ntrait A2<X> { fn m(x: X) -> X }\n\
         fn f() -> Binary{8} = m(0b0000_0001)",
    )
    .unwrap_err();
    assert!(err.message.contains("ambiguous"), "got: {}", err.message);
}

#[test]
fn an_undetermined_trait_method_call_is_an_explicit_error() {
    // A trait whose method does not mention its type parameter in a position the args determine:
    // `mk` returns `A` but takes no `A` argument, so a bare call cannot determine the receiver — an
    // explicit "does not determine" refusal, never a guessed instance.
    let err =
        check("nodule d\ntrait Mk<A> { fn mk() -> A }\nfn f() -> Binary{8} = mk()").unwrap_err();
    assert!(
        err.message.contains("does not determine") || err.message.contains("no instance"),
        "got: {}",
        err.message
    );
}

#[test]
fn a_duplicate_trait_declaration_is_an_explicit_error() {
    let err = check("nodule d\ntrait T<A> { fn f(x: A) -> A }\ntrait T<A> { fn g(x: A) -> A }")
        .unwrap_err();
    assert!(
        err.message.contains("duplicate trait"),
        "got: {}",
        err.message
    );
}

#[test]
fn a_duplicate_method_in_a_trait_is_an_explicit_error() {
    let err = check("nodule d\ntrait T<A> { fn f(x: A) -> A\n fn f(x: A) -> A }").unwrap_err();
    assert!(
        err.message.contains("duplicate method"),
        "got: {}",
        err.message
    );
}

#[test]
fn a_bound_on_an_unknown_trait_is_an_explicit_error() {
    let err = check("nodule d\nfn f<T: Nope>(x: T) -> T = x").unwrap_err();
    assert!(
        err.message.contains("unknown trait"),
        "got: {}",
        err.message
    );
}

#[test]
fn a_representation_specific_op_on_a_bounded_type_parameter_is_still_refused() {
    // RFC-0019 §4.6: a bound does NOT grant representation-specific ops. A `Binary` prim (`not`) on a
    // bounded `T: Cmp` value is refused exactly as in the unbounded case — never a silent coercion.
    let err = check(&format!("{CMP}fn bad<T: Cmp>(x: T) -> T = not(x)")).unwrap_err();
    assert!(
        err.message.contains("accept") || err.message.contains("not"),
        "got: {}",
        err.message
    );
}

#[test]
fn bounds_on_type_or_trait_parameters_are_a_parse_refusal() {
    // Stage-1: bounds live only on function type-params (the dictionary site). A bound on a `type`
    // (or `trait`) parameter is an explicit parse refusal — never silently dropped (G2).
    let err = parse("nodule d\ntype T<A: Cmp> = C(A)").unwrap_err();
    assert!(err.message.contains("deferred"), "got: {}", err.message);
}

#[test]
fn an_orphan_instance_for_a_foreign_type_and_trait_is_refused() {
    // The orphan rule (RFC-0019 §4.5) is checked on a `Data` head. Single-nodule stage-1 treats a
    // *local* trait OR a *local* data type as ownership, and primitive reprs as local. To exercise
    // the orphan refusal we make BOTH foreign: a trait declared, but an `impl` of it for a data type
    // declared in *neither* the trait's nodule nor the type's — but single-nodule everything is local.
    // So instead we confirm the locality logic positively: an instance for a primitive repr is legal
    // even when the trait is local (primitives count as owned), and a `Data` instance for a locally
    // declared type is legal. The cross-nodule orphan refusal is staged with M-662 (phylum work);
    // here we pin that a `Data` instance whose type is undeclared is rejected (unknown type), the
    // single-nodule stand-in for "you don't own this type".
    let err = check(
        "nodule d\ntrait T<A> { fn f(x: A) -> A }\n\
         impl T<Foreign> for Foreign { fn f(x: Foreign) -> Foreign = x }",
    )
    .unwrap_err();
    assert!(err.message.contains("unknown type"), "got: {}", err.message);
    // And a locally declared data type *is* a legal instance head (orphan rule satisfied via the
    // type's locality).
    let ok = check(
        "nodule d\ntype Pt = P(Binary{8})\ntrait T<A> { fn f(x: A) -> A }\n\
         impl T<Pt> for Pt { fn f(x: Pt) -> Pt = x }",
    );
    assert!(
        ok.is_ok(),
        "a local data type is a legal instance head: {ok:?}"
    );
}

#[test]
fn coherence_is_a_property_across_a_sweep_of_types_and_widths() {
    // Property (RFC-0019 §4.5 global uniqueness, per (trait, type-head)): across a sweep of repr
    // types and widths, a SINGLE instance per head always *checks*, and a SECOND instance on the
    // same head always *fails* with a coherence error — uniformly, never a guess. (Deterministic
    // sweep in the established `check.rs` property style — the existing generics property test, l.372,
    // uses the same loop form; no new proptest dependency.)
    // Each case: (trait-arg+for type written form, a *different* width on the SAME head).
    let cases: &[(&str, &str)] = &[
        ("Binary{8}", "Binary{16}"),
        ("Binary{1}", "Binary{32}"),
        ("Ternary{3}", "Ternary{9}"),
        ("Ternary{6}", "Ternary{12}"),
        ("Dense{4, F32}", "Dense{8, F32}"),
    ];
    for (ty, other) in cases {
        // A unique instance on the head always checks.
        let unique = format!(
            "nodule d\ntrait Tr<A> {{ fn f(x: A) -> A }}\n\
             impl Tr<{ty}> for {ty} {{ fn f(x: {ty}) -> {ty} = x }}"
        );
        let unique_res = check(&unique);
        assert!(
            unique_res.is_ok(),
            "a unique instance on {ty} must check: {:?}",
            unique_res
        );
        // A second instance on the SAME head (different width) always fails coherence.
        let dup = format!(
            "nodule d\ntrait Tr<A> {{ fn f(x: A) -> A }}\n\
             impl Tr<{ty}> for {ty} {{ fn f(x: {ty}) -> {ty} = x }}\n\
             impl Tr<{other}> for {other} {{ fn f(x: {other}) -> {other} = x }}"
        );
        let e = check(&dup).expect_err("a second instance on the head must be rejected");
        assert!(
            e.message.contains("coherence") || e.message.contains("overlapping"),
            "second instance on {ty}'s head must be a coherence error, got: {}",
            e.message
        );
    }
}

#[test]
fn an_instance_on_the_same_head_but_a_different_width_does_not_satisfy_a_call() {
    // Coherence keys per type-head, but RESOLUTION must match the FULL concrete type: a `Binary{8}`
    // instance must NOT satisfy a trait-method call whose receiver is `Binary{4}` (same head). This
    // is over-rejection-for-duplicates / never-over-acceptance-for-missing (RFC-0019 §4.5; G2).
    let src = "nodule d\n\
        trait Tr<A> { fn f(x: A) -> A }\n\
        impl Tr<Binary{8}> for Binary{8} { fn f(x: Binary{8}) -> Binary{8} = x }\n\
        fn g(x: Binary{4}) -> Binary{4} = f(x)";
    let e = check(src).expect_err("a Binary{4} call must not reuse the Binary{8} instance");
    assert!(
        e.message.contains("no instance") && e.message.contains("declared for"),
        "expected an explicit 'no instance … declared for' refusal, got: {}",
        e.message
    );
}

// --- stage-1 effect annotations + coverage (RFC-0014 §3.4/§4.5 I3/I5; M-660) ---------------------
// The surface `!{eff1, eff2}` after a fn's return type declares its effect set (empty = pure). The
// effect-coverage check requires a fn's DECLARED effects ⊇ the effects it PERFORMS, where performing
// = the union of its top-level callees' declared effects. Under-declaration is an explicit
// `CheckError` (G2/RFC-0014 I3); over-declaration is allowed (a declaration is a contract — I5).
// Guarantee on the pass: `Declared` (a structural coverage check, not a theorem). Effect names are
// plain identifiers (kernel kinds `retry|alloc|io|cascade|time` + user `Named`), NOT reserved words.

#[test]
fn an_effect_annotated_fn_parses_and_checks() {
    // `a` over-declares `io` with a pure (literal) body — allowed (the declaration is a contract,
    // RFC-0014 I5; a fn may reserve an effect its body does not yet perform). It must check, and the
    // declared set must be recorded on the fn's signature for EXPLAIN / future wiring.
    let env = check("nodule d\nfn a() -> Binary{8} !{io} = 0b00000000").expect("checks");
    assert_eq!(
        env.fn_decl("a").expect("fn a").sig.effects,
        vec!["io".to_owned()],
        "the declared effect set is recorded on the signature"
    );
}

#[test]
fn an_unannotated_caller_of_an_effectful_fn_is_a_check_error() {
    // THE M-660 acceptance: `b` calls the `io`-effectful `a` but declares no effects (unannotated ⇒
    // pure, RFC-0014 I5), so it PERFORMS `io` without DECLARING it — an explicit under-declaration
    // refusal naming the effect and the callee (RFC-0014 I3; never silent — G2).
    let err = check(
        "nodule d\nfn a() -> Binary{8} !{io} = 0b00000000\n\
         fn b() -> Binary{8} = a()",
    )
    .expect_err("an unannotated caller of an effectful fn must be refused");
    assert!(
        err.message.contains("io") && err.message.contains('a'),
        "the refusal must name the missing effect `io` and the callee `a`, got: {}",
        err.message
    );
    assert!(
        err.message.contains("does not declare") || err.message.contains("undeclared"),
        "the refusal must frame it as an under-declaration, got: {}",
        err.message
    );
}

#[test]
fn a_caller_that_declares_the_callees_effect_checks() {
    // `c` declares `io`, the effect its callee `a` performs — coverage holds (declared ⊇ performed),
    // so it checks. The compositional-check line of RFC-0014 §8 (manual-declare + compositional-check).
    let env = check(
        "nodule d\nfn a() -> Binary{8} !{io} = 0b00000000\n\
         fn c() -> Binary{8} !{io} = a()",
    )
    .expect("a caller that declares the callee's effect checks");
    assert_eq!(
        env.fn_decl("c").expect("fn c").sig.effects,
        vec!["io".to_owned()]
    );
}

#[test]
fn over_declaration_is_allowed() {
    // `d` declares `!{io, time}` but only calls `a` (which performs `io`) — declaring the unused
    // `time` is fine (a contract, never an error/lint — RFC-0014 I5). Coverage is a SUPERSET check.
    let env = check(
        "nodule d\nfn a() -> Binary{8} !{io} = 0b00000000\n\
         fn d() -> Binary{8} !{io, time} = a()",
    )
    .expect("over-declaration is allowed");
    assert_eq!(
        env.fn_decl("d").expect("fn d").sig.effects,
        vec!["io".to_owned(), "time".to_owned()]
    );
}

#[test]
fn an_empty_written_effect_set_is_pure_and_equals_unannotated() {
    // `!{}` is an explicit "declares no effects" — identical in meaning to an unannotated (pure) fn.
    // A pure-bodied fn with `!{}` checks; both record the empty effect set.
    let env = check(
        "nodule d\nfn p() -> Binary{8} !{} = 0b00000000\n\
         fn q() -> Binary{8} = 0b00000000",
    )
    .expect("an explicit empty effect set is pure");
    assert!(env.fn_decl("p").expect("fn p").sig.effects.is_empty());
    assert!(env.fn_decl("q").expect("fn q").sig.effects.is_empty());
}

#[test]
fn a_duplicate_effect_name_in_one_annotation_is_a_parse_refusal() {
    // A repeated effect in one annotation is a written redundancy — an explicit parse refusal (never
    // a silent dedup — G2/RFC-0014 §4.5).
    let err = parse("nodule d\nfn a() -> Binary{8} !{io, io} = 0b00000000")
        .expect_err("a duplicate effect name must be rejected");
    assert!(
        err.message.contains("duplicate effect"),
        "got: {}",
        err.message
    );
}

#[test]
fn effect_coverage_is_monotone_over_a_callee_sweep_a_property_bound() {
    // Property (RFC-0014 I3, the compositional-check bound): a fn calling a set of effectful fns must
    // declare ⊇ the union of their declared effects. Across a sweep of subsets of a fixed effect
    // pool, declaring EXACTLY the performed union always checks, and OMITTING any one performed
    // effect always fails naming that effect — uniformly, never a guess. (Deterministic sweep in the
    // established `check.rs` property style — the generics/coherence property tests use the same loop
    // form; no new proptest dependency.)
    let pool = ["io", "time", "alloc", "cascade"];
    // One effectful leaf fn per pool effect: `e_io`, `e_time`, … each declaring its single effect.
    let leaves: String = pool
        .iter()
        .map(|eff| format!("fn e_{eff}() -> Binary{{8}} !{{{eff}}} = 0b00000000\n"))
        .collect();
    // Sweep every non-empty subset of the pool (bitmask 1..2^N).
    for mask in 1u32..(1 << pool.len()) {
        let chosen: Vec<&str> = pool
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, e)| *e)
            .collect();
        // The caller calls each chosen leaf exactly once; its performed set is precisely `chosen`.
        let calls: String = chosen
            .iter()
            .map(|eff| format!("let _{eff} = e_{eff}() in "))
            .collect();
        let body = format!("{calls}0b00000000");

        // (a) Declaring EXACTLY the performed union checks.
        let declared = chosen.join(", ");
        let ok_src =
            format!("nodule d\n{leaves}fn caller() -> Binary{{8}} !{{{declared}}} = {body}");
        let ok = check(&ok_src);
        assert!(
            ok.is_ok(),
            "declaring exactly the performed union {chosen:?} must check: {:?}",
            ok.err()
        );

        // (b) OMITTING any single performed effect fails, naming that effect (under-declaration).
        for omit in &chosen {
            let kept: Vec<&str> = chosen.iter().copied().filter(|e| e != omit).collect();
            let kept_decl = kept.join(", ");
            let bad_src =
                format!("nodule d\n{leaves}fn caller() -> Binary{{8}} !{{{kept_decl}}} = {body}");
            let err = match check(&bad_src) {
                Ok(_) => {
                    panic!("omitting performed effect `{omit}` from {chosen:?} must fail to check")
                }
                Err(e) => e,
            };
            // The omitted effect cannot be the kept set; the error must name it.
            assert!(
                err.message.contains(omit),
                "omitting `{omit}` (chosen={chosen:?}) must name it in the refusal, got: {}",
                err.message
            );
        }
    }
}

#[test]
fn effect_coverage_accounts_for_trait_method_calls_and_impl_method_bodies() {
    // The coverage check must see effects performed through a TRAIT-METHOD call (not only direct fn
    // calls) and inside an IMPL-METHOD body — otherwise an effect could be hidden from a caller,
    // breaking the RFC-0014 invariant "an effect a function performs is visible in its signature".
    const LOG: &str = "nodule d\n\
        trait Log<A> { fn log(x: A) -> A !{io} }\n\
        impl Log<Binary{8}> for Binary{8} { fn log(x: Binary{8}) -> Binary{8} !{io} = x }\n";
    // (1) A fn calling the effectful trait method `log` performs `io` and must declare it.
    let bad = format!("{LOG}fn f(x: Binary{{8}}) -> Binary{{8}} = log(x)");
    let e = check(&bad)
        .expect_err("an unannotated caller of an effectful trait method must be refused");
    assert!(
        e.message.contains("io") && e.message.contains("does not declare"),
        "got: {}",
        e.message
    );
    // Declaring it checks.
    let ok = format!("{LOG}fn f(x: Binary{{8}}) -> Binary{{8}} !{{io}} = log(x)");
    assert!(
        check(&ok).is_ok(),
        "declaring `io` checks: {:?}",
        check(&ok)
    );

    // (2) An IMPL-METHOD body that performs an effect its declared set (== the trait method's) does
    // not cover is refused — here `m` declares `time` (matching the trait) but its body performs `io`
    // via the top-level `ioop`, so the effect would be hidden if impl bodies were not checked.
    let bad_impl = "nodule d\n\
        fn ioop() -> Binary{8} !{io} = 0b00000000\n\
        trait T<A> { fn m(x: A) -> Binary{8} !{time} }\n\
        impl T<Binary{8}> for Binary{8} { fn m(x: Binary{8}) -> Binary{8} !{time} = let _y = ioop() in x }";
    let e2 = check(bad_impl)
        .expect_err("an impl method performing an effect it does not declare must be refused");
    assert!(
        e2.message.contains("io") && e2.message.contains("does not declare"),
        "got: {}",
        e2.message
    );
}

#[test]
fn a_trait_method_with_effects_an_impl_with_different_effects_is_refused() {
    // Effect conformance (RFC-0014 §4.5 I3; M-660): an impl method's declared effects must EQUAL the
    // trait method's. Here the trait declares `cmp` with `!{io}` but the impl method declares `!{}`
    // (pure) — an explicit refusal (never a silent widen/narrow — G2).
    let err = check(
        "nodule d\ntrait Cmp<A> { fn cmp(a: A, b: A) -> Binary{2} !{io} }\n\
         impl Cmp<Binary{8}> for Binary{8} { fn cmp(a: Binary{8}, b: Binary{8}) -> Binary{2} = 0b00 }",
    )
    .expect_err("an impl method whose effects differ from the trait's must be refused");
    assert!(
        err.message.contains("effect") && err.message.contains("match"),
        "the refusal must frame an effect-annotation mismatch, got: {}",
        err.message
    );
}

#[test]
fn a_trait_method_with_matching_effects_in_the_impl_checks() {
    // The dual of the refusal: an impl method declaring the SAME effects as the trait method checks
    // (exact-match conformance — RFC-0014 §4.5). The trait and impl both declare `!{io}`.
    let env = check(
        "nodule d\ntrait Cmp<A> { fn cmp(a: A, b: A) -> Binary{2} !{io} }\n\
         impl Cmp<Binary{8}> for Binary{8} \
         { fn cmp(a: Binary{8}, b: Binary{8}) -> Binary{2} !{io} = 0b00 }",
    )
    .expect("an impl method whose effects match the trait's checks");
    assert!(env.trait_info("Cmp").is_some());
}

#[test]
fn an_effect_carrying_call_through_a_transitive_chain_must_be_declared() {
    // Coverage composes one hop at a time (the v0 compositional check, RFC-0014 §8): `mid` calls the
    // `io`-effectful `leaf`, so `mid` must declare `io`; `top` calls `mid` (which declares `io`), so
    // `top` must declare `io` too. With every link declaring `io`, the chain checks.
    let env = check(
        "nodule d\nfn leaf() -> Binary{8} !{io} = 0b00000000\n\
         fn mid() -> Binary{8} !{io} = leaf()\n\
         fn top() -> Binary{8} !{io} = mid()",
    )
    .expect("a fully-declared effect chain checks");
    assert_eq!(
        env.fn_decl("top").expect("fn top").sig.effects,
        vec!["io".to_owned()]
    );

    // Break the middle link: `mid` performs `io` (via `leaf`) but does not declare it → refusal.
    let err = check(
        "nodule d\nfn leaf() -> Binary{8} !{io} = 0b00000000\n\
         fn mid() -> Binary{8} = leaf()\n\
         fn top() -> Binary{8} !{io} = mid()",
    )
    .expect_err("an undeclared middle link must be refused");
    assert!(
        err.message.contains("io"),
        "the refusal must name the undeclared effect, got: {}",
        err.message
    );
}
