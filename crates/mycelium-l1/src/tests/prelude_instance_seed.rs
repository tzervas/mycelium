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
//!
//! **CRITICAL fix (strict-review mutation test, this leaf).** The claim in the previous paragraph
//! was FALSE as originally implemented: the sig-pin test used to build its oracle `Env`s via the
//! ORDINARY `check_nodule` pipeline, which unconditionally re-runs the
//! [`crate::checkty::PRELUDE_INSTANCE_SEEDS`] seeding step on every nodule it checks — so a seed
//! naming a head the real body does NOT provide self-inserts into its own empty `instances` slot
//! while the "real" oracle is being built, and the comparison then diffs the seed against ITSELF
//! (a trivial, silent pass). Proof: mutating `seed_init_bool()`'s `for_ty` to a nonexistent head
//! still passed all 9 entries. Drift on an ALREADY-EXISTING head is still caught correctly (the
//! real declaration registers first, so the seed's `entry().or_insert()` is a no-op and the
//! comparison is genuine) — only a NOVEL nonexistent head was silently masked. The fix: the
//! sig-pin test now builds its oracles via [`fmt_env_clean`]/[`derive_prelude_env_clean`], which
//! engage [`crate::checkty::SuppressInstanceSeedingForTest`] so the returned `Env` reflects ONLY
//! what the real source body itself declares — proven non-vacuous by
//! [`a_seed_naming_a_head_absent_from_the_real_body_is_caught_by_the_clean_oracle`] and
//! [`seed_instance_for_nodule_self_inserts_a_fact_at_an_otherwise_unoccupied_key`], below.

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

/// **THE sig-pin oracle builder (DN-138 §5 obligation 1, CRITICAL fix).** Builds `Show`'s real
/// `lib/std/fmt.myc` oracle with [`SuppressInstanceSeedingForTest`] engaged, so the returned `Env`
/// carries ONLY what `fmt.myc` itself actually declares — never a seed's own self-insertion. This
/// is what makes a seed naming a nonexistent head genuinely resolve to `None` below, instead of a
/// trivial self-comparison.
fn fmt_env_clean() -> Env {
    let _guard = SuppressInstanceSeedingForTest::engage();
    env(FMT_SRC)
}

/// **THE sig-pin oracle builder (DN-138 §5 obligation 1, CRITICAL fix)** — the `Init`/`Ord3`
/// analogue of [`fmt_env_clean`], over `lib/std/derive_prelude.myc`.
fn derive_prelude_env_clean() -> Env {
    let _guard = SuppressInstanceSeedingForTest::engage();
    env(DERIVE_PRELUDE_SRC)
}

/// The sig-pin core check, factored out so both the real (9-seed, all-real-heads) test and the
/// adversarial (fabricated-head) regression test below exercise the IDENTICAL comparison logic —
/// the only variable is which seed list and which oracle `Env`s are passed in. Panics on the first
/// divergence, exactly like the original inline loop; the adversarial test observes that with
/// `std::panic::catch_unwind`.
fn assert_every_seed_pins(seeds: &[crate::preseed::PreludeInstanceSeed], fmt: &Env, prelude: &Env) {
    for seed in seeds {
        let seeded = (seed.instance)();
        let head = type_head(&seeded.for_ty)
            .unwrap_or_else(|| panic!("seed for `{}` has no concrete head", seed.trait_name));
        let key = (seed.trait_name.to_owned(), head.clone());
        let real_env = if seed.trait_name == "Show" {
            fmt
        } else {
            prelude
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
    }
}

/// **THE sig-pin differential (DN-138 §5 obligation 1).** Every entry of
/// [`crate::checkty::PRELUDE_INSTANCE_SEEDS`] is diffed against the real `lib/std` body it claims
/// to mirror, built via the CLEAN (seeding-suppressed) oracle — see the module doc's CRITICAL fix.
/// Non-vacuous: 9 entries, each independently looked up; a drift in ANY one of them (wrong width,
/// wrong method name, a body that stops existing) fails this test at the specific failing entry,
/// naming it — and, per the sibling adversarial tests below, a head absent from the real body is
/// now genuinely caught, never silently masked by seed self-insertion.
#[test]
fn every_seed_sig_pins_to_its_real_lib_std_body() {
    let fmt = fmt_env_clean();
    let prelude = derive_prelude_env_clean();
    assert_every_seed_pins(&PRELUDE_INSTANCE_SEEDS, &fmt, &prelude);
    assert_eq!(
        PRELUDE_INSTANCE_SEEDS.len(),
        9,
        "expected exactly the 9 DN-138 increment-1 seeds (Show/Init/Ord3 x Binary{{64}}/Bytes/Bool)"
    );
}

/// A fabricated `InstanceInfo` at a head neither `lib/std/fmt.myc` nor
/// `lib/std/derive_prelude.myc` ever declares an instance of, and which `PRELUDE_INSTANCE_SEEDS`
/// itself never seeds — used only by the adversarial tests below to reproduce the strict-review
/// mutation-testing finding (mutating a real seed's `for_ty` to a nonexistent head) without
/// touching the real, private seed table.
fn bogus_absent_head_instance() -> InstanceInfo {
    InstanceInfo {
        trait_name: "Init".to_owned(),
        trait_args: vec![Ty::Data("AdversarialNotReal".to_owned(), vec![])],
        for_ty: Ty::Data("AdversarialNotReal".to_owned(), vec![]),
        methods: vec!["init".to_owned()],
    }
}

const BOGUS_SEED: crate::preseed::PreludeInstanceSeed = crate::preseed::PreludeInstanceSeed {
    trait_name: "Init",
    impl_hint: "impl Init[AdversarialNotReal] for AdversarialNotReal { … } (test-only, fabricated \
                — never a real prelude seed)",
    instance: bogus_absent_head_instance,
};

/// **The real adversarial proof of the CRITICAL fix (DN-138 §5 obl. 1).** A seed naming a head
/// ABSENT from both real oracle files must make [`assert_every_seed_pins`] genuinely fail against
/// the CLEAN (seeding-suppressed) oracle: the bogus seed's head is truly absent from what
/// `fmt.myc`/`derive_prelude.myc` themselves declare, so the lookup is `None` and the check panics.
/// This is the non-vacuous proof that DN-138 §5 obligation 1's guardrail now actually catches a
/// nonexistent-head seed, rather than being able to pass by re-diffing a self-inserted fact against
/// itself (the exact hazard the module doc's CRITICAL fix note describes).
#[test]
fn a_seed_naming_a_head_absent_from_the_real_body_is_caught_by_the_clean_oracle() {
    let clean_fmt = fmt_env_clean();
    let clean_prelude = derive_prelude_env_clean();
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        assert_every_seed_pins(&[BOGUS_SEED], &clean_fmt, &clean_prelude);
    }));
    assert!(
        caught.is_err(),
        "a seed naming a head absent from BOTH real oracle files must make the sig-pin check FAIL \
         against the clean (seeding-suppressed) oracle — non-vacuous proof the guardrail actually \
         catches a nonexistent-head seed, not merely a re-diff of a self-inserted fact"
    );
}

/// **Mutation-witnessed proof of the bug this leaf fixes** (pinned so the contamination hole can
/// never silently return). Directly exercises the mechanism the strict review's mutation test
/// found: [`crate::preseed::PreludeInstanceSeed::seed_instance_for_nodule`] — the exact function
/// `check_nodule`'s per-nodule pass loops over `PRELUDE_INSTANCE_SEEDS` to call — self-inserts its
/// OWN fabricated fact into an otherwise-empty `instances` map for any nodule that merely triggers
/// the seed's trait, with no regard for whether a real declaration exists anywhere. This is why
/// building the sig-pin oracle via the ORDINARY (seeded) `check_nodule` pipeline could never
/// distinguish "a real declaration already filled this slot" from "nothing did, and the seed just
/// fabricated one" — the reason the sig-pin oracle above must be built with seeding suppressed.
#[test]
fn seed_instance_for_nodule_self_inserts_a_fact_at_an_otherwise_unoccupied_key() {
    use std::collections::BTreeMap;
    // A nodule that triggers `Init` (any `impl Init[...] for ...`) but declares NOTHING at the
    // bogus seed's own head — the real-world shape of "a seed whose head the real body doesn't
    // provide".
    let nodule = crate::parse(
        "nodule d;\n\
         type Wrap = Wrap(Bytes);\n\
         impl Init[Wrap] for Wrap {\n\
           fn init() => Wrap = Wrap(init());\n\
         };",
    )
    .expect("parses");
    let mut instances: BTreeMap<(String, String), InstanceInfo> = BTreeMap::new();
    BOGUS_SEED.seed_instance_for_nodule(&mut instances, &nodule);
    assert_eq!(
        instances.get(&("Init".to_owned(), "Data:AdversarialNotReal".to_owned())),
        Some(&bogus_absent_head_instance()),
        "seed_instance_for_nodule must have self-inserted its own fabricated fact at its own \
         empty key — this is the exact self-insertion mechanism the CRITICAL fix works around by \
         building the sig-pin oracle with seeding suppressed instead of via the ordinary pipeline"
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

/// **Verify-first correction (mitigation #14 / VR-5), the SECOND independent disconfirmation of
/// DN-138 §5 obligation 5's literal wording:** a nodule that triggers the `Show` seed and ALSO
/// hand-declares a DIFFERENT concrete type at the SAME width-erased `"Binary"` head the seed
/// occupies (`Binary{32}` vs the seed's `Binary{64}`) is a real, already-shipped shape — the
/// pre-existing DN-122/M-1080 MVP foreign-trait-impl test hand-declares exactly this for `Ord3`
/// (`impl Ord3[Binary{8}] for Binary{8}` in complete isolation), and it must keep checking clean.
/// The corrected semantics: the nodule's OWN instance wins (registered exactly as declared,
/// `Binary{32}`, never silently swapped for the seed's `Binary{64}`), and the seed simply declines
/// to add anything on top — proven here by asserting the actually-registered `for_ty`.
#[test]
fn a_nodule_own_different_width_instance_at_the_seeded_head_wins_over_the_seed() {
    let checked = env("nodule d;\n\
         type Pair = Pair(Bool);\n\
         impl Show[Pair] for Pair {\n\
           fn render(x: Pair) => Bytes = match x { Pair(a) => render(a) };\n\
         };\n\
         impl Show[Binary{32}] for Binary{32} {\n\
           fn render(x: Binary{32}) => Bytes = \"x\";\n\
         };");
    let registered = checked
        .instances
        .get(&("Show".to_owned(), "Binary".to_owned()))
        .expect("some Show/Binary instance must be registered");
    assert_eq!(
        registered.for_ty,
        Ty::Binary(Width::Lit(32)),
        "the nodule's OWN Binary{{32}} instance must win over the seed's Binary{{64}} fact, got {registered:?}"
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
