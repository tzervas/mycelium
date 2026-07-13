//! DN-138 §4.1 Alt A / §8 WU-2 — the `PRELUDE_INSTANCE_SEEDS` primitive-instance-seed spine.
//!
//! **The load-bearing test in this file is [`every_seed_sig_pins_to_its_real_lib_std_body`]** (DN-138
//! §5 obligation 1 — the whole soundness argument for Alt A rests on it): each seeded
//! [`crate::checkty::InstanceInfo`] is diffed, byte-for-byte (`for_ty`/`trait_args`/`methods`),
//! against what the REAL `lib/std/fmt.myc` (`Show`) / `lib/std/derive_prelude.myc` (`Init`/`Ord3`)
//! body actually checks to. A seed whose `for_ty` names a width/type the real body does not provide
//! fails here (a `None` lookup); a seed whose method-name set diverges from the real impl's checked
//! method set ALSO fails here (`InstanceInfo`'s `PartialEq` compares the whole struct). Because
//! every one of `Show`/`Init`/`Ord3` is single-parameter and param-only-sig (DN-122 §13.1's admitted
//! shape), pinning `for_ty` exactly determines the substituted method signature too (the trait's own
//! `TraitInfo` is a fixed Rust constant) — so this `InstanceInfo` diff is not merely a name check,
//! it is the full signature pin DN-138 §5 obligation 1 demands. Instance EXISTENCE is subsumed: no
//! real body ⇒ `real_env.instances.get(&key)` is `None` ⇒ the test fails (never a false pass).

use crate::checkty::*;
use crate::parse;

fn env(src: &str) -> Env {
    check_nodule(&parse(src).expect("parses")).expect("checks")
}

fn check_err(src: &str) -> CheckError {
    check_nodule(&parse(src).expect("parses")).expect_err("must fail to check")
}

/// `Show`'s real primitive instances (DN-127, already landed) — `lib/std/fmt.myc`.
const FMT_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../lib/std/fmt.myc"
));

/// `Init`/`Ord3`'s real primitive instances (DN-138 WU-1, this leaf) — `lib/std/derive_prelude.myc`.
const DERIVE_PRELUDE_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../lib/std/derive_prelude.myc"
));

fn fmt_env() -> Env {
    env(FMT_SRC)
}

fn derive_prelude_env() -> Env {
    env(DERIVE_PRELUDE_SRC)
}

/// **THE sig-pin differential (DN-138 §5 obligation 1).** Every entry of
/// [`crate::checkty::PRELUDE_INSTANCE_SEEDS`] is diffed against the real `lib/std` body it claims
/// to mirror. Non-vacuous: 9 entries, each independently looked up; a drift in ANY one of them
/// (wrong width, wrong method name, a body that stops existing) fails this test at the specific
/// failing entry, naming it.
#[test]
fn every_seed_sig_pins_to_its_real_lib_std_body() {
    let fmt = fmt_env();
    let prelude = derive_prelude_env();
    let mut checked = 0usize;
    for seed in PRELUDE_INSTANCE_SEEDS {
        let seeded = (seed.instance)();
        let head = type_head(&seeded.for_ty)
            .unwrap_or_else(|| panic!("seed for `{}` has no concrete head", seed.trait_name));
        let key = (seed.trait_name.to_owned(), head.clone());
        let real_env = if seed.trait_name == "Show" {
            &fmt
        } else {
            &prelude
        };
        let real = real_env.instances.get(&key).unwrap_or_else(|| {
            panic!(
                "no REAL `{}` instance at head `{head}` found in the lib/std oracle — the seed \
                 claims a resolution fact `lib/std` does not actually provide (DN-138 §5 obl. 1 \
                 sig-drift hazard). seeded={seeded:?}",
                seed.trait_name
            )
        });
        assert_eq!(
            real, &seeded,
            "seed/body divergence for `{}` at head `{head}` — the seeded fact and the real \
             `lib/std` instance must be byte-identical (for_ty/trait_args/methods); a mismatch \
             here is exactly the check-passes/eval-fails hazard DN-138 §5 obl. 1 exists to catch",
            seed.trait_name
        );
        checked += 1;
    }
    assert_eq!(
        checked, 9,
        "expected exactly the 9 DN-138 increment-1 seeds (Show/Init/Ord3 x Binary{{64}}/Bytes/Bool)"
    );
}

/// Adversarial witness for the sig-pin test's own non-vacuousness: if a seed's claimed `for_ty`
/// does NOT exist as a real instance in the cited oracle file, the lookup is `None` and the test
/// panics — instance existence is subsumed by the sig-pin differential, not a separate check. This
/// is asserted here directly (rather than only trusted from the passing test above) by probing a
/// deliberately-wrong head against the real environments.
#[test]
fn a_seed_whose_head_does_not_exist_in_the_real_body_is_never_silently_accepted() {
    let fmt = fmt_env();
    // `Show` has no `Binary{32}` instance in the real file (only `Binary{64}` — the width-erased
    // head is `"Binary"`, so this looks up the SAME slot `Binary{64}` occupies, and it IS present;
    // this probes a head that is not present at all instead).
    assert!(
        fmt.instances
            .get(&("Show".to_owned(), "Data:NotReal".to_owned()))
            .is_none(),
        "a fabricated head must never resolve in the real oracle"
    );
}

/// DN-138 §5 obligation 4 (conditional-on-need) / §2 fact 2 (the `mono` fast-path regression this
/// guards against): a program that uses NONE of `Show`/`Init`/`Ord3` gets NO seeded instance at
/// all — `env.instances` stays empty, preserving `crate::mono::is_already_monomorphic`'s
/// `env.instances.is_empty()` fast-path for every trait-free program.
#[test]
fn a_trait_free_program_seeds_no_primitive_instance() {
    let checked = env("nodule d;\nfn id(x: Binary{8}) => Binary{8} = x;");
    assert!(
        checked.instances.is_empty(),
        "a trait-free program must not seed any primitive instance, got {:?}",
        checked.instances.keys().collect::<Vec<_>>()
    );
}

/// The positive integration case: a nodule that declares its OWN `impl Show[Struct] for Struct`
/// whose body calls `render` on a `Binary{64}` field resolves the seeded `Show[Binary{64}]`
/// instance with no local declaration of it — this is the exact shape a transpiled
/// `derive(Debug)`-composed struct produces (DN-138 §1/§3).
#[test]
fn a_struct_impl_resolves_the_seeded_binary64_show_instance_with_no_local_declaration() {
    let checked = env(
        "nodule d;\n\
         type Pair = Pair(Binary{64}, Bytes);\n\
         impl Show[Pair] for Pair {\n\
           fn render(x: Pair) => Bytes =\n\
             match x { Pair(a, b) => bytes_concat(bytes_concat(\"Pair(\", render(a)), render(b)) };\n\
         };",
    );
    assert!(checked.traits.contains_key("Show"));
    assert!(checked
        .instances
        .contains_key(&("Show".to_owned(), "Binary".to_owned())));
    assert!(checked
        .instances
        .contains_key(&("Show".to_owned(), "Bytes".to_owned())));
    // The user's OWN instance is present too, distinct from the seeded primitive ones (DN-112
    // Rank 1: a nodule-qualified `Ty::Data` head, `"Data:<home>::Pair"` for `nodule d;`'s home `d`).
    assert!(checked
        .instances
        .contains_key(&("Show".to_owned(), "Data:d::Pair".to_owned())));
}

/// Same shape for `Init`/`Ord3` — a struct deriving `Default`/`PartialOrd`-equivalent composition
/// over a `Binary{64}` field resolves the seeded instances with no local declaration.
#[test]
fn a_struct_impl_resolves_the_seeded_binary64_init_and_ord3_instances() {
    let init_checked = env("nodule d;\n\
         type Wrap = Wrap(Binary{64});\n\
         impl Init[Wrap] for Wrap {\n\
           fn init() => Wrap =\n\
             Wrap(init());\n\
         };");
    assert!(init_checked
        .instances
        .contains_key(&("Init".to_owned(), "Binary".to_owned())));

    let ord_checked = env("nodule d;\n\
         type Wrap = Wrap(Binary{64});\n\
         impl Ord3[Wrap] for Wrap {\n\
           fn cmp(a: Wrap, b: Wrap) => Binary{8} =\n\
             match a { Wrap(p0) => match b { Wrap(q0) => cmp(p0, q0) } };\n\
         };");
    assert!(ord_checked
        .instances
        .contains_key(&("Ord3".to_owned(), "Binary".to_owned())));
}

/// **Verify-first correction (mitigation #14 / VR-5), pinned as its own regression:** an IDENTICAL
/// self-provision (a nodule that both triggers the `Show` seed and ALSO hand-declares the exact
/// SAME `Show[Binary{64}]` instance the seed provides) is NOT refused — it is exactly the
/// `lib/std/fmt.myc`/`lib/std/derive_prelude.myc` shape (the sig-pin test's own oracle files), and
/// must check clean. The real-hand-written instance simply wins; nothing is seeded on top of it.
#[test]
fn an_identical_self_provided_primitive_instance_is_not_a_redeclare_conflict() {
    let checked = env("nodule d;\n\
         type Pair = Pair(Binary{64});\n\
         impl Show[Pair] for Pair {\n\
           fn render(x: Pair) => Bytes = match x { Pair(a) => render(a) };\n\
         };\n\
         impl Show[Binary{64}] for Binary{64} {\n\
           fn render(x: Binary{64}) => Bytes = \"x\";\n\
         };");
    assert!(checked
        .instances
        .contains_key(&("Show".to_owned(), "Binary".to_owned())));
}

/// DN-138 §5 obligation 5 (never-silent redeclare-refusal) — the GENUINE conflict case: a nodule
/// that triggers the `Show` seed and ALSO hand-declares a DIFFERENT concrete type at the SAME
/// width-erased `"Binary"` head the seed occupies (`Binary{32}` vs the seed's `Binary{64}`) is
/// refused, never silently letting either side win.
#[test]
fn redeclaring_a_seeded_primitive_instance_at_a_conflicting_width_is_refused() {
    let err = check_err(
        "nodule d;\n\
         type Pair = Pair(Binary{64});\n\
         impl Show[Pair] for Pair {\n\
           fn render(x: Pair) => Bytes = match x { Pair(a) => render(a) };\n\
         };\n\
         impl Show[Binary{32}] for Binary{32} {\n\
           fn render(x: Binary{32}) => Bytes = \"x\";\n\
         };",
    );
    assert!(
        err.message.contains("Show") && err.message.contains("Binary"),
        "expected a redeclare/coherence-conflict refusal naming `Show`/the `Binary` head, got: {}",
        err.message
    );
}

/// DN-138 §2 fact 1 (width-erased coherence) / §5(b) (the "honest width-mismatch gap" the
/// adversarial stress-test names): a struct whose field is a NARROW `Binary{8}` (not the seeded
/// `Binary{64}`) still refuses to resolve `render` for it — the seed only covers the one width
/// increment 1 targets; a narrower width is an explicit, never-silent `myc check` refusal, never a
/// silently-reused mismatched instance (`require_instance`'s own `info.for_ty == *concrete` guard).
#[test]
fn a_narrow_width_scalar_field_does_not_silently_reuse_the_binary64_show_instance() {
    let err = check_err(
        "nodule d;\n\
         type Pair = Pair(Binary{8});\n\
         impl Show[Pair] for Pair {\n\
           fn render(x: Pair) => Bytes = match x { Pair(a) => render(a) };\n\
         };",
    );
    assert!(
        err.message.contains("Show") && err.message.contains("Binary{8}"),
        "expected an explicit no-instance-for-Binary{{8}} refusal, got: {}",
        err.message
    );
}

/// `Float` is never seeded (DN-138 §5 obligation 3 / ADR-040): no `(Show|Init|Ord3, "Float")` key
/// ever appears among the 9 increment-1 seeds.
#[test]
fn float_is_never_among_the_seeded_heads() {
    for seed in PRELUDE_INSTANCE_SEEDS {
        let info = (seed.instance)();
        assert_ne!(
            type_head(&info.for_ty),
            Some("Float".to_owned()),
            "`{}` must never seed a `Float` instance (ADR-040)",
            seed.trait_name
        );
    }
}
