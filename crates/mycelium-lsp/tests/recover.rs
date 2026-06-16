//! M-352 / RFC-0014 §5 acceptance — declarative error recovery & bounded effects.
//!
//! The central test is the **never-silent recovery invariant** (I1): for a corpus of errors and *any*
//! action, the error is either explicitly recovered or re-propagated — never dropped. The rest cover
//! the bounded-overrun-is-explicit test (I4), the no-undeclared-effect test (I3), the honest-guarantee
//! test (I2/VR-5), the opt-in default-scope test (I5), and the shared-registry / no-`eval` discipline
//! (X1) recovery inherits from RFC-0013.

use mycelium_core::{GuaranteeStrength, Meta, Payload, Provenance, Repr, Value};
use mycelium_lsp::recover::{
    check_effects, handle, Action, EffectBudget, EffectBudgets, EffectKind, EffectSet, Outcome,
    RecoveryPolicy, Resolution, StructuredError,
};
use mycelium_lsp::ClassRegistry;

fn registry() -> ClassRegistry {
    ClassRegistry::with_builtins()
}

fn byte() -> Value {
    Value::new(
        Repr::Binary { width: 8 },
        Payload::Bits(vec![true, false, true, true, false, false, true, false]),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

fn an_error(reg: &ClassRegistry) -> StructuredError {
    StructuredError::new(
        reg.resolve("SwapOutOfRange").unwrap(),
        "value left the certified range",
        "let a/swap",
    )
}

// --- the central never-silent recovery invariant (I1) ---

#[test]
fn a_handler_never_drops_an_error() {
    let reg = registry();
    let err = an_error(&reg);

    // Every recovery action, including the ones nearest to "make it disappear".
    let actions = vec![
        None, // no rule at all
        Some(Action::Fallback {
            value: Box::new(byte()),
        }),
        Some(Action::Retry { max_attempts: 2 }),
        Some(Action::Escalate {
            to: reg.resolve("NotValidated").unwrap(),
        }),
        Some(Action::CleanupThenPropagate {
            effect: EffectKind::Io,
        }),
    ];

    for action in actions {
        let mut policy = RecoveryPolicy::new();
        if let Some(a) = action {
            policy.on(&reg, "SwapOutOfRange", a).unwrap();
        }
        let mut budgets = EffectBudgets::new();
        // An attempt thunk that always fails (so retry exhausts → must still propagate).
        let resolution = handle(Outcome::Err(err.clone()), &policy, &mut budgets, || {
            Outcome::Err(an_error(&reg))
        });
        // I1: the result is ALWAYS Recovered or Propagated — never a drop (no third variant exists).
        match resolution {
            Resolution::Recovered { .. } => { /* an explicit recovered value */ }
            Resolution::Propagated { error, .. } => {
                // A propagated error is still an explicit, surfacing error (possibly escalated).
                assert!(
                    !error.message.is_empty(),
                    "a propagated error must still carry its reason"
                );
            }
        }
    }
}

#[test]
fn no_policy_propagates_the_error_unchanged() {
    let reg = registry();
    let err = an_error(&reg);
    let policy = RecoveryPolicy::new(); // empty
    let mut budgets = EffectBudgets::new();
    let r = handle(Outcome::Err(err.clone()), &policy, &mut budgets, || {
        Outcome::Ok(byte())
    });
    match r {
        Resolution::Propagated { error, policy } => {
            assert_eq!(error, err, "an unmatched error propagates UNCHANGED (I1)");
            assert!(policy.is_none(), "no policy acted");
        }
        Resolution::Recovered { .. } => panic!("no policy must not recover"),
    }
}

#[test]
fn ok_passes_through_carrying_its_own_guarantee() {
    let reg = registry();
    let policy = RecoveryPolicy::new();
    let mut budgets = EffectBudgets::new();
    let r = handle(Outcome::Ok(byte()), &policy, &mut budgets, || {
        Outcome::Err(an_error(&reg))
    });
    match r {
        Resolution::Recovered {
            guarantee, policy, ..
        } => {
            assert_eq!(
                guarantee,
                GuaranteeStrength::Exact,
                "Ok keeps the value's own guarantee"
            );
            assert!(policy.is_none(), "nothing was recovered — no policy acted");
        }
        Resolution::Propagated { .. } => panic!("Ok must pass through"),
    }
}

// --- honest guarantee (I2 / VR-5) ---

#[test]
fn a_fallback_is_honestly_declared_never_upgraded() {
    let reg = registry();
    // The fallback VALUE is itself Exact-tagged, but a *substituted* fallback has no checked basis, so
    // recovery must clamp it to `Declared` — never upgraded (I2/VR-5). Mutant-witness: a renderer that
    // propagated the value's Exact tag would be caught here.
    let mut policy = RecoveryPolicy::new();
    policy
        .on(
            &reg,
            "SwapOutOfRange",
            Action::Fallback {
                value: Box::new(byte()),
            },
        )
        .unwrap();
    let mut budgets = EffectBudgets::new();
    let r = handle(Outcome::Err(an_error(&reg)), &policy, &mut budgets, || {
        Outcome::Err(an_error(&reg))
    });
    match r {
        Resolution::Recovered {
            guarantee, policy, ..
        } => {
            assert_eq!(
                guarantee,
                GuaranteeStrength::Declared,
                "a substituted fallback is honestly Declared, never upgraded (I2/VR-5)"
            );
            assert!(
                policy.is_some(),
                "the recovering policy is recorded (PolicyRef)"
            );
        }
        Resolution::Propagated { .. } => panic!("fallback must recover"),
    }
}

// --- retry: bounded, additive on exhaustion (I1/I4) ---

#[test]
fn retry_recovers_within_attempts_else_propagates() {
    let reg = registry();
    let mut policy = RecoveryPolicy::new();
    policy
        .on(&reg, "SwapOutOfRange", Action::Retry { max_attempts: 3 })
        .unwrap();

    // Fails twice, then succeeds → recovered within the 3-attempt bound.
    let mut budgets = EffectBudgets::new();
    let mut calls = 0;
    let r = handle(Outcome::Err(an_error(&reg)), &policy, &mut budgets, || {
        calls += 1;
        if calls < 3 {
            Outcome::Err(an_error(&reg))
        } else {
            Outcome::Ok(byte())
        }
    });
    assert!(
        matches!(r, Resolution::Recovered { .. }),
        "retry recovers within the bound"
    );

    // Always fails → retries exhausted → the original error PROPAGATES (additive — I1), never a hang.
    let mut budgets = EffectBudgets::new();
    let err = an_error(&reg);
    let r = handle(Outcome::Err(err.clone()), &policy, &mut budgets, || {
        Outcome::Err(an_error(&reg))
    });
    match r {
        Resolution::Propagated { error, .. } => assert_eq!(error.class, err.class),
        Resolution::Recovered { .. } => panic!("exhausted retries must propagate, not fabricate"),
    }
}

#[test]
fn escalate_repropagates_a_transformed_but_explicit_error() {
    let reg = registry();
    let mut policy = RecoveryPolicy::new();
    policy
        .on(
            &reg,
            "SwapOutOfRange",
            Action::Escalate {
                to: reg.resolve("NotValidated").unwrap(),
            },
        )
        .unwrap();
    let mut budgets = EffectBudgets::new();
    let r = handle(Outcome::Err(an_error(&reg)), &policy, &mut budgets, || {
        Outcome::Ok(byte())
    });
    match r {
        Resolution::Propagated { error, policy } => {
            assert_eq!(
                error.class.as_str(),
                "NotValidated",
                "escalated to the new class"
            );
            assert!(
                error.message.contains("escalated from SwapOutOfRange"),
                "transform is explicit"
            );
            assert!(policy.is_some());
        }
        Resolution::Recovered { .. } => panic!("escalate re-propagates, never recovers"),
    }
}

// --- bounded effects: overrun is explicit + graceful (I4) ---

#[test]
fn a_budget_overrun_is_an_explicit_graceful_error() {
    // A cascade bounded to depth 2: two consumes succeed, the third is an explicit EffectBudgetExhausted
    // (never a hang/stack overflow/OOM — I4). The analogue of FuelExhausted/DepthLimit.
    let mut budgets = EffectBudgets::new().with(EffectBudget::Depth(2));
    assert!(budgets.consume(EffectKind::Cascade, 1).is_ok());
    assert!(budgets.consume(EffectKind::Cascade, 1).is_ok());
    let err = budgets
        .consume(EffectKind::Cascade, 1)
        .expect_err("the third cascade overruns the depth-2 budget");
    assert_eq!(err.kind, EffectKind::Cascade);
    assert_eq!(err.remaining, 0);
}

#[test]
fn an_undeclared_budget_means_the_effect_cannot_run_by_default() {
    // I5: the default scope is the narrowest — an effect with NO declared budget cannot consume; you opt
    // into a broader effect by explicitly declaring its budget.
    let mut tight = EffectBudgets::new();
    assert!(
        tight.consume(EffectKind::Alloc, 1).is_err(),
        "an undeclared effect cannot run by default (I5)"
    );
    let mut opted_in = EffectBudgets::new().with(EffectBudget::Bytes(64));
    assert!(
        opted_in.consume(EffectKind::Alloc, 64).is_ok(),
        "declaring the budget opts the effect in"
    );
    assert!(
        opted_in.consume(EffectKind::Alloc, 1).is_err(),
        "but only up to the declared bound"
    );
}

// --- declared effects: no undeclared effect (I3) ---

#[test]
fn performing_an_undeclared_effect_is_an_explicit_error() {
    let declared: EffectSet = [EffectKind::Retry, EffectKind::Alloc].into_iter().collect();

    // A subset of the declared effects checks fine.
    let ok: EffectSet = [EffectKind::Retry].into_iter().collect();
    assert!(check_effects(&declared, &ok).is_ok());

    // Performing `Io` — not declared — is an explicit error (no unknown side effects; I3).
    let bad: EffectSet = [EffectKind::Retry, EffectKind::Io].into_iter().collect();
    let err = check_effects(&declared, &bad).expect_err("an undeclared effect must be refused");
    assert_eq!(err.effect, EffectKind::Io);
}

// --- shared registry / no eval (X1, inherited from RFC-0013) ---

#[test]
fn recovery_policy_resolves_classes_through_the_shared_registry() {
    let reg = registry();
    let mut policy = RecoveryPolicy::new();
    // A known class resolves.
    assert!(policy
        .on(&reg, "SwapOutOfRange", Action::Retry { max_attempts: 1 })
        .is_ok());
    // An unknown class is an explicit error — never an eval path (X1 applies equally to recovery, §4.4).
    let err = policy
        .on(&reg, "rm -rf /", Action::Retry { max_attempts: 1 })
        .expect_err("an unknown class must be refused");
    assert_eq!(err.name, "rm -rf /");
}

#[test]
fn the_same_policy_has_a_stable_content_id() {
    let reg = registry();
    let build = || {
        let mut p = RecoveryPolicy::new();
        p.on(&reg, "SwapOutOfRange", Action::Retry { max_attempts: 3 })
            .unwrap();
        p.on(
            &reg,
            "NotValidated",
            Action::Fallback {
                value: Box::new(byte()),
            },
        )
        .unwrap();
        p
    };
    assert_eq!(
        build().content_id(),
        build().content_id(),
        "a reified recovery policy is content-addressed and identity-stable (PolicyRef)"
    );
}
