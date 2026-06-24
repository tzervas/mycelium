//! White-box + property tests for [`crate::cert_scope`] — the M-790 DoD laws (RFC-0034 §6 / §3.1).
//!
//! The two property tests required by the DoD:
//! 1. **Resolution precedence** — a `nodule` declaration overrides a `phylum` one, which overrides a
//!    `global` one (most-specific-wins), for *any* combination of modes and *any* declaration order.
//! 2. **Cross-mode never-silent-upgrade** — composing a value produced under one mode into a
//!    computation under any other mode never upgrades the value's guarantee strength (VR-5); a `fast`
//!    value entering a `certified` computation is always an explicit, visible boundary event.

use crate::cert_scope::*;
use mycelium_core::cert_mode::CertMode;
use mycelium_core::guarantee::GuaranteeStrength;
use proptest::prelude::*;

/// Strategy over the three certification modes.
fn any_mode() -> impl Strategy<Value = CertMode> {
    prop::sample::select(CertMode::ALL.to_vec())
}

/// Strategy over the three scopes.
fn any_scope() -> impl Strategy<Value = CertScope> {
    prop::sample::select(CertScope::ALL.to_vec())
}

/// Strategy over the four guarantee strengths.
fn any_strength() -> impl Strategy<Value = GuaranteeStrength> {
    prop::sample::select(GuaranteeStrength::ALL.to_vec())
}

// --- deterministic unit checks (the named, load-bearing cases) ---

#[test]
fn no_declarations_resolves_to_the_default() {
    let r = resolve_mode(&[]);
    assert_eq!(r, ResolvedMode::defaulted());
    assert_eq!(r.mode, CertMode::Fast);
    assert_eq!(r.source, None);
}

#[test]
fn parse_and_word_round_trip() {
    for mode in CertMode::ALL {
        assert_eq!(parse_cert_mode(cert_mode_word(mode)).unwrap(), mode);
    }
}

#[test]
fn parse_rejects_an_unknown_mode_word() {
    // Never-silent (G2): an out-of-set word is an explicit error, not a guess.
    let e = parse_cert_mode("turbo").unwrap_err();
    assert!(e.contains("unknown @certification mode"), "{e}");
    // The serde-capitalized spelling is *not* the surface spelling (FLAG-A) — also rejected.
    assert!(parse_cert_mode("Fast").is_err());
}

#[test]
fn scope_specificity_is_global_lt_phylum_lt_nodule() {
    assert!(CertScope::Global.specificity() < CertScope::Phylum.specificity());
    assert!(CertScope::Phylum.specificity() < CertScope::Nodule.specificity());
}

#[test]
fn fast_into_certified_is_an_explicit_boundary_never_an_upgrade() {
    // The concrete DoD case: a `fast` value (Empirical-intended) entering a `certified` computation.
    let ev = compose(
        CertMode::Fast,
        CertMode::Certified,
        GuaranteeStrength::Empirical,
    );
    // It is a visible boundary (producer ran less certification than consumer)…
    assert!(ev.is_boundary());
    // …and the value did NOT inherit a stronger guarantee — `Fast` floors Empirical to Declared.
    assert_eq!(ev.effective, GuaranteeStrength::Declared);
    assert!(!ev.upgraded_strength());
}

#[test]
fn structural_exact_survives_the_boundary() {
    // A structural `Exact` (e.g. a bijective swap) is not floored even under `Fast` (it earned the
    // strength structurally) — composing it never *upgrades*, and it stays Exact.
    let ev = compose(
        CertMode::Fast,
        CertMode::Certified,
        GuaranteeStrength::Exact,
    );
    assert_eq!(ev.effective, GuaranteeStrength::Exact);
    assert!(!ev.upgraded_strength());
}

// --- DoD property tests ---

proptest! {
    /// DoD #1 — **resolution precedence**: with declarations present at a set of scopes, the resolved
    /// mode is exactly the one declared at the most-specific scope, regardless of declaration order.
    #[test]
    fn prop_most_specific_scope_wins(
        global in any_mode(),
        phylum in any_mode(),
        nodule in any_mode(),
        // Which scopes actually carry a declaration (at least one, else the law is "default").
        has_global in any::<bool>(),
        has_phylum in any::<bool>(),
        has_nodule in any::<bool>(),
    ) {
        let mut decls = Vec::new();
        if has_global { decls.push(CertDecl { scope: CertScope::Global, mode: global }); }
        if has_phylum { decls.push(CertDecl { scope: CertScope::Phylum, mode: phylum }); }
        if has_nodule { decls.push(CertDecl { scope: CertScope::Nodule, mode: nodule }); }

        let r = resolve_mode(&decls);

        // Expected winner: nodule > phylum > global; none ⇒ default.
        let (exp_mode, exp_src) = if has_nodule {
            (nodule, Some(CertScope::Nodule))
        } else if has_phylum {
            (phylum, Some(CertScope::Phylum))
        } else if has_global {
            (global, Some(CertScope::Global))
        } else {
            (CertMode::default(), None)
        };
        prop_assert_eq!(r.mode, exp_mode);
        prop_assert_eq!(r.source, exp_src);
    }

    /// DoD #1 (order-independence) — resolution picks by specificity, not by position: shuffling the
    /// declaration vector cannot change the result.
    #[test]
    fn prop_resolution_is_order_independent(
        a in any_scope(), ma in any_mode(),
        b in any_scope(), mb in any_mode(),
    ) {
        // Two declarations at (possibly distinct) scopes; if scopes collide, drop one (the parser
        // forbids two declarations at the same scope).
        let mut forward = vec![CertDecl { scope: a, mode: ma }];
        if b != a { forward.push(CertDecl { scope: b, mode: mb }); }
        let mut backward = forward.clone();
        backward.reverse();
        prop_assert_eq!(resolve_mode(&forward), resolve_mode(&backward));
    }

    /// DoD #2 — **cross-mode composition never silently upgrades**: for *any* producer/consumer modes
    /// and *any* incoming strength, the effective strength after crossing is never stronger than what
    /// the value came in with (VR-5), and an up-crossing is always a visible boundary event.
    #[test]
    fn prop_cross_mode_never_upgrades_strength(
        producer in any_mode(),
        consumer in any_mode(),
        incoming in any_strength(),
    ) {
        let ev = compose(producer, consumer, incoming);
        // Never an upgrade: the effective strength's rank is >= the incoming rank (weaker-or-equal).
        prop_assert!(!ev.upgraded_strength(),
            "compose({:?},{:?},{:?}) upgraded to {:?}", producer, consumer, incoming, ev.effective);
        prop_assert!(ev.effective.rank() >= incoming.rank());
        // The effective strength is floored by the PRODUCER's mode, not the consumer's — the value
        // keeps only the strength its own mode established.
        prop_assert_eq!(ev.effective, producer.gate_guarantee(incoming));
        // An up-crossing (producer weaker-certified than consumer) is flagged as a boundary.
        prop_assert_eq!(ev.is_boundary(), producer.depth() < consumer.depth());
    }
}
