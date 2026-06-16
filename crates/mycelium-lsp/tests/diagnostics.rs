//! M-345 / RFC-0013 §5 acceptance — structured diagnostics & reified error policy.
//!
//! The central test is the **never-silent invariant** (I1/I2/I4): applying *any* policy leaves the
//! explicit error still propagating. The rest cover the round-trip projection (I3), registry lookup /
//! no-`eval` (X1), the allowlisted detailed tier (X2), and the representation-crossing audit view
//! (I5/VR-5).

use mycelium_core::{GuaranteeStrength, Meta, Node, Payload, Provenance, Repr, Value};
use mycelium_lsp::diagnostics::{
    present, AuditView, ClassRegistry, DiagnosticPolicy, DiagnosticRecord, Level, ReasonedError,
    Rule,
};

fn registry() -> ClassRegistry {
    ClassRegistry::with_builtins()
}

fn an_error(reg: &ClassRegistry) -> ReasonedError {
    ReasonedError::new(
        reg.resolve("SwapOutOfRange").unwrap(),
        "value left the certified range here",
        "let a/swap",
    )
    .with_reason("ternary 0t21 decodes to 7, above the Binary{3} max 7+1")
    .with_context("from_repr", "Ternary{6}")
    .with_context("to_repr", "Binary{3}")
}

// --- the central never-silent invariant (I1/I2/I4) ---

#[test]
fn a_policy_never_suppresses_the_error() {
    let reg = registry();
    let err = an_error(&reg);

    // A battery of policies, including ones that try hardest to "make the error go away": a minimal
    // level, a pure route, an empty rule, a message override.
    let mut routed = DiagnosticPolicy::new();
    routed
        .on(
            &reg,
            "SwapOutOfRange",
            Rule::new()
                .route("diagnostics_channel")
                .level(Level::Minimal),
        )
        .unwrap();
    let mut overridden = DiagnosticPolicy::new();
    overridden
        .on(
            &reg,
            "SwapOutOfRange",
            Rule::new().message("(quietly noted)"),
        )
        .unwrap();
    let mut empty_rule = DiagnosticPolicy::new();
    empty_rule.on(&reg, "SwapOutOfRange", Rule::new()).unwrap();
    let unrelated = {
        // A policy whose only rule is for a *different* class — must not touch this error at all.
        let mut p = DiagnosticPolicy::new();
        p.on(&reg, "TypeMismatch", Rule::new().route("void"))
            .unwrap();
        p
    };

    let policies: Vec<Option<&DiagnosticPolicy>> = vec![
        None,
        Some(&routed),
        Some(&overridden),
        Some(&empty_rule),
        Some(&unrelated),
    ];

    for policy in policies {
        let p = present(err.clone(), policy);
        // I1: the explicit error is returned UNCHANGED — it still propagates. A mutant renderer that
        // dropped or softened the error is caught here.
        assert_eq!(
            p.error, err,
            "a policy must never alter or suppress the propagating error (I1)"
        );
        // I2: even at the minimal level the refusal is named (class + site present in the human view).
        let human = p.diagnostic.to_human();
        assert!(
            human.contains("SwapOutOfRange") && human.contains("let a/swap"),
            "the refusal must always be named, regardless of policy (I2): {human}"
        );
        // I4: a route never gates propagation — the error is still there even when routed away.
        if p.diagnostic.route.is_some() {
            assert_eq!(p.error, err, "routing must not affect propagation (I1/I4)");
        }
    }
}

#[test]
fn lowering_the_level_shows_less_never_hides_the_error() {
    let reg = registry();
    let err = an_error(&reg);
    let mut minimal = DiagnosticPolicy::new();
    minimal
        .on(&reg, "SwapOutOfRange", Rule::new().level(Level::Minimal))
        .unwrap();
    let mut detailed = DiagnosticPolicy::new();
    detailed
        .on(&reg, "SwapOutOfRange", Rule::new().level(Level::Detailed))
        .unwrap();

    let min = present(err.clone(), Some(&minimal)).diagnostic.to_human();
    let det = present(err.clone(), Some(&detailed)).diagnostic.to_human();

    // Minimal still names the refusal (I2); detailed adds more — never less.
    assert!(min.contains("SwapOutOfRange"));
    assert!(det.contains("SwapOutOfRange"));
    assert!(
        det.len() > min.len(),
        "detailed shows MORE of the same truth"
    );
    // Medium-and-up reveal the reason; minimal does not (verbosity, not existence).
    assert!(!min.contains("decodes to 7"));
    assert!(det.contains("decodes to 7"));
}

// --- round-trip projection (I3) ---

#[test]
fn human_and_json_are_two_views_of_one_content_addressed_truth() {
    let reg = registry();
    let mut policy = DiagnosticPolicy::new();
    policy
        .on(
            &reg,
            "SwapOutOfRange",
            Rule::new().level(Level::Detailed).tag("swap").tag("review"),
        )
        .unwrap();
    let record = present(an_error(&reg), Some(&policy)).diagnostic;

    let id = record.content_id();

    // JSON round-trips to an equal record with the same content id (I3).
    let json = record.to_json();
    let back = DiagnosticRecord::from_json(&json).expect("json round-trips");
    assert_eq!(back, record);
    assert_eq!(back.content_id(), id);
    // The JSON projection embeds the id.
    assert!(json.contains(id.as_str()));

    // The human projection carries the SAME content id — two views, one truth.
    let human = record.to_human();
    assert!(
        human.contains(id.as_str()),
        "the human view must carry the diagnostic's content identity (I3)"
    );
}

// --- registry lookup / no eval (X1) ---

#[test]
fn unknown_error_class_is_an_explicit_error_never_evaluated() {
    let reg = registry();
    // A bare lookup of an unknown class is an explicit error (never silently coerced).
    assert!(reg.resolve("os.system('rm -rf /')").is_err());
    assert!(reg.resolve("TotallyMadeUp").is_err());

    // A policy cannot name an unknown class — `on` rejects it explicitly (no eval path; X1).
    let mut policy = DiagnosticPolicy::new();
    let err = policy
        .on(&reg, "NotARealClass", Rule::new())
        .expect_err("an unknown class must be refused");
    assert_eq!(err.name, "NotARealClass");

    // A known class resolves.
    assert!(reg.resolve("SwapOutOfRange").is_ok());
}

#[test]
fn a_policy_file_with_an_unknown_class_is_rejected_whole() {
    let reg = registry();
    let json =
        r#"{ "on": { "SwapOutOfRange": { "level": "medium" }, "Bogus": { "route": "x" } } }"#;
    let file: mycelium_lsp::diagnostics::PolicyFile = serde_json::from_str(json).unwrap();
    // The file is a projection of the canonical declaration (§4.7); ingesting validates every class
    // through the registry — an unknown one rejects the whole file, never partially/silently applies.
    let err = DiagnosticPolicy::from_file(&reg, &file).expect_err("unknown class rejects the file");
    assert_eq!(err.name, "Bogus");
}

// --- allowlisted detailed tier (X2) ---

#[test]
fn the_detailed_tier_is_allowlisted_never_a_wholesale_dump() {
    let reg = registry();
    // An error carrying a secret-bearing context field that is NOT on the allowlist.
    let err = ReasonedError::new(
        reg.resolve("SwapOutOfRange").unwrap(),
        "out of range",
        "swap",
    )
    .with_context("from_repr", "Ternary{6}") // allowlisted
    .with_context("AWS_SECRET_ACCESS_KEY", "hunter2") // NOT allowlisted
    .with_context("env", "<entire os.environ>"); // NOT allowlisted

    let mut detailed = DiagnosticPolicy::new();
    detailed
        .on(&reg, "SwapOutOfRange", Rule::new().level(Level::Detailed))
        .unwrap();
    let record = present(err, Some(&detailed)).diagnostic;

    // The allowlisted field survives; the secret-bearing fields were never gathered (X2).
    assert!(record.context.contains_key("from_repr"));
    assert!(!record.context.contains_key("AWS_SECRET_ACCESS_KEY"));
    assert!(!record.context.contains_key("env"));
    // Not even via the rendered detailed view.
    let human = record.to_human();
    assert!(!human.contains("hunter2"));
    assert!(!human.contains("os.environ"));
}

// --- representation-crossing audit view (I5 / VR-5) ---

fn byte() -> Value {
    Value::new(
        Repr::Binary { width: 8 },
        Payload::Bits(vec![true, false, true, true, false, false, true, false]),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

fn swap_to(target: Repr) -> Node {
    Node::Swap {
        src: Box::new(Node::Const(byte())),
        target,
        policy: mycelium_core::ContentHash::parse("blake3:po1icy_Ref00").unwrap(),
    }
}

#[test]
fn audit_view_enumerates_every_crossing_wherever_it_sits() {
    // One crossing at top level, one buried inside a `let` body and an `op` — placement-independent.
    let program = Node::Let {
        id: "a".into(),
        bound: Box::new(swap_to(Repr::Ternary { trits: 6 })),
        body: Box::new(Node::Op {
            prim: "id".into(),
            args: vec![swap_to(Repr::Ternary { trits: 6 })],
        }),
    };
    let view = AuditView::of(&program);
    assert_eq!(
        view.crossings.len(),
        2,
        "every swap is enumerated regardless of where it sits (I5)"
    );
    // It is a read-only projection with a dual human/JSON form.
    assert!(view.to_json().contains("crossings"));
    assert!(view.to_human().contains("→"));
}

#[test]
fn audit_view_reads_honesty_off_each_crossing_never_upgrades_it() {
    // A legal Binary{8} → Ternary{6} crossing is bijective (Exact) — read off the certificate.
    let exact = AuditView::of(&swap_to(Repr::Ternary { trits: 6 }));
    assert_eq!(exact.crossings.len(), 1);
    assert_eq!(
        exact.crossings[0].honesty,
        Some(GuaranteeStrength::Exact),
        "a bijective crossing's honesty is read as Exact"
    );

    // An illegal Binary{8} → Ternary{2} pair has no derivable certificate here. The honesty is
    // honestly `None` (unknown) — NEVER silently upgraded to Exact (VR-5). This is the mutant-witness
    // for an "assume Exact" bug.
    let unknown = AuditView::of(&swap_to(Repr::Ternary { trits: 2 }));
    assert_eq!(unknown.crossings.len(), 1);
    assert_eq!(
        unknown.crossings[0].honesty, None,
        "an underivable crossing stays unknown, never upgraded to Exact (VR-5)"
    );
    // The human view says `unknown`, not a fabricated bound.
    assert!(unknown.to_human().contains("unknown"));
}

#[test]
fn audit_view_reports_from_to_and_policy_per_crossing() {
    let view = AuditView::of(&swap_to(Repr::Ternary { trits: 6 }));
    let c = &view.crossings[0];
    assert_eq!(c.from, Some(Repr::Binary { width: 8 }));
    assert_eq!(c.to, Repr::Ternary { trits: 6 });
    assert_eq!(c.policy, "blake3:po1icy_Ref00");
}
