//! In-crate white-box tests for `mycelium-diag` (CLAUDE.md test layout: no tests in logic files).
//! `mycelium-diag`'s only source module is `lib.rs` itself (a single-file crate), so this is one
//! flat file rather than a `tests/` directory with one submodule per source module — the same shape
//! as the `mycelium-std-recover/src/tests.rs` precedent CLAUDE.md names. `use crate::*;` gives
//! white-box access to private items (`render_where`).
//!
//! Extracted as-touched (M-797) from the previous inline `#[cfg(test)] mod tests { … }` block while
//! landing the RFC-0013 Amendment A1 first-fault envelope (W-A); the pre-existing tests are carried
//! over verbatim (only de-indented and `use super::*;` → `use crate::*;`), and the new envelope /
//! `first_fault_line` / backward-compatibility tests are appended below.

use crate::*;

// ── Builder contract ────────────────────────────────────────────────────────────────────────

#[test]
fn builders_are_total_and_locus_absence_is_explicit() {
    let d = Diag::error(Code::OutOfRange).message("payload len ≠ width");
    assert_eq!(d.severity(), Severity::Error);
    assert_eq!(d.code(), &Code::OutOfRange);
    // A missing locus is explicit None, never a fabricated zero (G2).
    assert!(d.locus.is_none());
}

#[test]
fn at_records_an_explicit_locus() {
    let d = Diag::warn(Code::Budget).at(Locus {
        source: Some("x.myc".into()),
        line: Some(3),
        column: None,
    });
    let l = d.locus.expect("locus set");
    assert_eq!(l.line, Some(3));
    // An absent column stays None — not a fabricated 0 (G2).
    assert!(l.column.is_none());
}

// ── Severity ordering (typed distinction, never stringly-typed) ─────────────────────────────

/// `Severity` is a typed distinction with a defined order (RFC-0013 §4.1).
/// Mutation witness: removing `PartialOrd`/`Ord` derives makes this fail.
#[test]
fn severity_is_a_typed_distinction_with_ordering() {
    assert!(Severity::Debug < Severity::Info);
    assert!(Severity::Info < Severity::Warn);
    assert!(Severity::Warn < Severity::Error);
    // Exhaustively verify all pairs are ordered consistently.
    for (i, a) in Severity::ALL.iter().enumerate() {
        for (j, b) in Severity::ALL.iter().enumerate() {
            match i.cmp(&j) {
                std::cmp::Ordering::Less => assert!(*a < *b, "{a:?} < {b:?}"),
                std::cmp::Ordering::Greater => assert!(*a > *b, "{a:?} > {b:?}"),
                std::cmp::Ordering::Equal => assert_eq!(*a, *b, "{a:?} == {b:?}"),
            }
        }
    }
}

/// `Severity::as_str` round-trips through the serde rename (non-stringly typed).
/// Mutation witness: renaming a variant without updating `as_str` breaks this test.
#[test]
fn severity_as_str_matches_serde_rename() {
    for s in Severity::ALL {
        let json = serde_json::to_string(&s).expect("Severity serializes");
        // serde rename_all = "lowercase" wraps the string in quotes.
        let expected = format!("\"{}\"", s.as_str());
        assert_eq!(
            json, expected,
            "Severity::{s:?}.as_str() must match serde rename"
        );
    }
}

// ── Content hash (ADR-003 / RFC-0013 I3) ───────────────────────────────────────────────────

/// The content hash is deterministic: the same `Diag` always produces the same hash.
/// Mutation witness: changing the domain tag `"mycelium.diag.v1"` in `content_hash` changes all
/// hashes and breaks this test.
#[test]
fn content_hash_is_deterministic() {
    let d = Diag::error(Code::OutOfRange)
        .message("test msg")
        .note("some note");
    let h1 = d.content_hash();
    let h2 = d.content_hash();
    assert_eq!(h1, h2, "content_hash must be deterministic");
}

/// `content_hash()` is presentation-invariant: producing human()/machine() views does not change
/// the hash.
/// Mutation witness: having `human()` or `machine()` mutate state (impossible with &self, but
/// guard it anyway) would break this test.
#[test]
fn content_hash_is_presentation_invariant() {
    let d = Diag::error(Code::OutOfRange).message("msg");
    let h = d.content_hash();
    let _ = d.human();
    let _ = d.machine();
    assert_eq!(
        d.content_hash(),
        h,
        "human()/machine() must not change identity"
    );
}

/// Distinct canonical fields produce distinct hashes (collision resistance for common cases).
/// Mutation witness: removing field-specific hashing in `content_hash` makes two distinct `Diag`s
/// collide.
#[test]
fn distinct_fields_produce_distinct_hashes() {
    let a = Diag::error(Code::OutOfRange).message("a");
    let b = Diag::error(Code::OutOfRange).message("b");
    assert_ne!(
        a.content_hash(),
        b.content_hash(),
        "different message → different hash"
    );

    let c = Diag::warn(Code::OutOfRange).message("a");
    assert_ne!(
        a.content_hash(),
        c.content_hash(),
        "different severity → different hash"
    );

    let d = Diag::error(Code::Budget).message("a");
    assert_ne!(
        a.content_hash(),
        d.content_hash(),
        "different code → different hash"
    );

    let e = Diag::error(Code::OutOfRange).message("a").note("extra");
    assert_ne!(
        a.content_hash(),
        e.content_hash(),
        "extra note → different hash"
    );
}

/// A `Diag` with a locus vs. without produces distinct hashes (explicit absence, G2).
/// Mutation witness: commenting out the locus branch in `content_hash` makes this collide.
#[test]
fn locus_absence_is_explicit_in_hash() {
    let without = Diag::error(Code::OutOfRange).message("m");
    let with_locus = Diag::error(Code::OutOfRange).message("m").at(Locus {
        source: Some("f.myc".into()),
        line: None,
        column: None,
    });
    assert_ne!(
        without.content_hash(),
        with_locus.content_hash(),
        "locus changes identity (G2 — absence is distinct from presence)"
    );
}

/// `None` locus and an all-`None`-field `Some(Locus::default())` produce distinct hashes (G2).
/// Mutation witness: changing the locus presence tag from 1 to 0 collapses these two cases.
#[test]
fn locus_none_differs_from_default_locus() {
    let no_locus = Diag::error(Code::OutOfRange).message("m");
    let default_locus = Diag::error(Code::OutOfRange)
        .message("m")
        .at(Locus::default()); // all-None fields
    assert_ne!(
        no_locus.content_hash(),
        default_locus.content_hash(),
        "None locus ≠ Some(Locus::default()) — explicit absence (G2)"
    );
}

/// A `Diag` with a non-empty trace produces a distinct hash from one without (G2).
/// Mutation witness: commenting out the trace encoding in `content_hash` collapses these.
#[test]
fn trace_is_identity_bearing() {
    let no_trace = Diag::error(Code::OutOfRange).message("m");
    let with_trace = Diag::error(Code::OutOfRange)
        .message("m")
        .trace(Trace::empty().with_frame("outer"));
    assert_ne!(
        no_trace.content_hash(),
        with_trace.content_hash(),
        "non-empty trace changes identity"
    );
}

/// A `Diag` survives clone/re-use with identity unchanged (value-semantic).
/// Mutation witness: making `note()` mutate in-place rather than return a new value would cause
/// the original's hash to change.
#[test]
fn diag_identity_unchanged_through_clone() {
    let base = Diag::error(Code::OutOfRange).message("base");
    let base_hash = base.content_hash();
    // Value-semantic builder: `base` is unchanged; the extended record is a new value.
    let extended = base.clone().note("extra detail");
    assert_eq!(
        base.content_hash(),
        base_hash,
        "base record identity must not change"
    );
    assert_ne!(
        base.content_hash(),
        extended.content_hash(),
        "adding a note changes identity"
    );
}

// ── Dual projection (G11 / RFC-0013 I3) ────────────────────────────────────────────────────

/// `human()` is total for any well-formed `Diag` (including empty message, no locus, no notes).
/// Mutation witness: making `human()` return an `Option` or panic on empty message breaks this.
#[test]
fn human_is_total() {
    let d = Diag::error(Code::OutOfRange);
    let h = d.human();
    assert!(h.contains("[ERROR]"), "human() must name the severity");
    assert!(h.contains("OutOfRange"), "human() must name the code");
    assert!(h.contains("id:"), "human() must embed the content id (I3)");
}

/// `machine()` is total and embeds the content `id` field.
/// Mutation witness: removing the `id` injection from `machine()` makes this fail.
#[test]
fn machine_is_total_and_embeds_id() {
    let d = Diag::error(Code::Budget).message("budget exceeded");
    let json = d.machine();
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("machine() must produce valid JSON");
    assert!(
        parsed.get("id").is_some(),
        "machine() must embed the content id (I3)"
    );
    let id_field = parsed["id"].as_str().expect("id is a string");
    assert_eq!(
        id_field,
        d.content_hash().as_str(),
        "embedded id must match content_hash()"
    );
}

/// `from_json(machine(d))` recovers a record equal to `d` (the round-trip property, I3).
/// Mutation witness: injecting the `id` field into JSON without ignoring it on deserialization
/// would cause `from_json` to fail with an unknown-field error.
#[test]
fn machine_to_from_json_round_trips() {
    let d = Diag::error(Code::OutOfRange)
        .message("range violation")
        .at(Locus {
            source: Some("src.myc".into()),
            line: Some(12),
            column: Some(5),
        })
        .trace(
            Trace::empty()
                .with_frame("check_range")
                .with_frame("validate"),
        )
        .note("expected 0..256")
        .note("got 300");
    let json = d.machine();
    let recovered = Diag::from_json(&json).expect("machine() JSON must be valid");
    assert_eq!(d, recovered, "from_json(machine(d)) must equal d (I3)");
    assert_eq!(
        d.content_hash(),
        recovered.content_hash(),
        "round-trip preserves content identity (I3)"
    );
}

/// `from_json` returns an explicit `Err` on malformed input (C1 — never a partial/sentinel record).
/// Mutation witness: removing the `?` / error handling from `from_json` makes malformed input
/// silently succeed.
#[test]
fn from_json_returns_explicit_err_on_malformed_input() {
    // Completely invalid JSON.
    assert!(Diag::from_json("not json at all").is_err());
    // Unknown severity variant.
    assert!(Diag::from_json(r#"{"severity":"unknown_level","code":{"kind":"OutOfRange"},"message":"","locus":null,"trace":{"frames":[]},"notes":[]}"#).is_err());
}

/// The human and machine projections share the same content id (I3).
/// Mutation witness: using a different hash in `human()` vs. `content_hash()` would make the
/// embedded ids diverge.
#[test]
fn human_and_machine_share_content_id() {
    let d = Diag::warn(Code::HashMismatch).message("mismatch detected");
    let h = d.human();
    let m = d.machine();
    let id = d.content_hash().as_str().to_owned();
    assert!(h.contains(&id), "human() must embed the content id (I3)");
    assert!(m.contains(&id), "machine() must embed the content id (I3)");
}

/// The `Code::Other` variant round-trips through serde correctly.
/// Mutation witness: removing the `Other` variant or changing the serde tag breaks this.
#[test]
fn code_other_round_trips() {
    let d = Diag::error(Code::Other("MyCustomCode".into())).message("custom");
    let json = d.machine();
    let recovered = Diag::from_json(&json).expect("round-trip");
    assert_eq!(d.code(), recovered.code());
    assert_eq!(d.code().as_str(), "MyCustomCode");
}

// ─── First-fault envelope (RFC-0013 Amendment A1 §10) — the W-A additions ──────────────────────

/// A minimal, valid envelope for tests (fixture; complex per-case data lives in the parameterized
/// tests below, not in ad hoc bespoke logic per test body — CLAUDE.md test-layout rule).
fn swap_check_envelope() -> FirstFaultEnvelope {
    FirstFaultEnvelope::new(
        EventId::new("evt-1"),
        Phase::Runtime,
        SiteKind::SwapCheck,
        Decision::NotValidated,
        "swap_check.v0",
        CertMode::Certified,
    )
}

// ── Backward compatibility (I1 — additive, never substitutive) ────────────────────────────────

/// **Content-hash stability golden.** An envelope-less `Diag`'s `content_hash()` is byte-identical
/// to the hash this exact construction produced *before* Amendment A1 landed (pinned literally —
/// computed against the pre-amendment code and hardcoded here, not merely "equal to itself").
/// Mutation witness: any change to `content_hash`'s pre-envelope byte stream (including adding an
/// unconditional presence-tag byte for `envelope`) breaks this golden.
#[test]
fn content_hash_is_stable_for_envelope_less_diags_across_the_amendment() {
    let d = Diag::error(Code::OutOfRange).message("golden test");
    assert!(d.envelope().is_none());
    assert_eq!(
        d.content_hash().as_str(),
        "blake3:05afb8d0402f2e35101d695579edf64bd3dba82ff8bab6d092b7da0ec49291a3",
        "an envelope-less Diag's content_hash must be byte-identical to the pre-amendment hash (I1)"
    );
}

/// `human()` output for an envelope-less `Diag` is unaffected by the amendment: attaching no
/// envelope produces the exact same rendering as before (I1). A companion enveloped `Diag` still
/// renders identically via `human()` — the envelope is not consumed by `human()` at all in this
/// wave (the dedicated renderer is `first_fault_line()`, not `human()` — a judgment call, flagged in
/// the wave report; not decided whether a future revision folds envelope detail into `human()`'s
/// detailed tier, §10.5).
#[test]
fn human_output_is_unchanged_for_envelope_less_diags() {
    let without = Diag::error(Code::OutOfRange).message("m");
    let with_envelope = without.clone().with_envelope(swap_check_envelope());
    // Strip the trailing `id: <hash>` line — the only part that can differ, since attaching an
    // envelope changes content identity (see `attached_envelope_changes_content_identity`) — and
    // compare the rest verbatim.
    let strip_id = |s: &str| {
        s.rsplit_once("\n  id: ")
            .map_or_else(|| s.to_owned(), |(rest, _)| rest.to_owned())
    };
    assert_eq!(
        strip_id(&without.human()),
        strip_id(&with_envelope.human()),
        "human() text (minus the embedded content id) must be identical whether or not an \
         envelope is attached (I1) — human() does not consume the envelope in this wave"
    );
}

// ── Envelope participates in content_hash + machine(), never in a way that upgrades a grade ────

/// The envelope IS identity-bearing once attached: two Diags differing only in `site_kind` produce
/// different hashes (I1 backward-compat is about the None case only — see the golden above).
/// Mutation witness: dropping the envelope branch in `content_hash` collapses these.
#[test]
fn attached_envelope_changes_content_identity() {
    let base = Diag::error(Code::OutOfRange).message("m");
    let a = base.clone().with_envelope(swap_check_envelope());
    let b = base.with_envelope(FirstFaultEnvelope::new(
        EventId::new("evt-1"),
        Phase::Runtime,
        SiteKind::PolicyResolve, // differs only here
        Decision::NotValidated,
        "swap_check.v0",
        CertMode::Certified,
    ));
    assert_ne!(
        a.content_hash(),
        b.content_hash(),
        "different site_kind must change identity once an envelope is attached"
    );
}

/// `machine()` embeds the envelope object (not merely `null`) when attached, and the round-trip
/// (`from_json(machine(d)) == d`) still holds with the envelope present (I3 extended to the new
/// field).
#[test]
fn machine_embeds_the_envelope_and_round_trips() {
    let d = Diag::error(Code::Other("SwapCheckNotValidated".to_owned()))
        .message("swap certificate did not validate")
        .with_envelope(swap_check_envelope());
    let json = d.machine();
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(
        parsed.get("envelope").is_some_and(|v| !v.is_null()),
        "machine() must embed a non-null envelope object when one is attached"
    );
    assert_eq!(parsed["envelope"]["site_kind"]["kind"], "SwapCheck");
    let recovered = Diag::from_json(&json).expect("round-trip");
    assert_eq!(
        d, recovered,
        "from_json(machine(d)) must equal d with an envelope attached (I3)"
    );
    assert_eq!(d.content_hash(), recovered.content_hash());
}

/// An envelope-less `Diag`'s `machine()` projection carries an explicit `"envelope":null` (never
/// silently omitted — G2) and still round-trips against a **legacy**, pre-amendment JSON string that
/// has no `envelope` key at all (serde's built-in `Option` default) — proving old wire data stays
/// readable (I1).
#[test]
fn legacy_json_without_an_envelope_key_still_deserializes() {
    let d = Diag::error(Code::OutOfRange).message("m");
    let json = d.machine();
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(
        parsed["envelope"].is_null(),
        "no envelope attached ⇒ explicit null (G2)"
    );

    // A pre-amendment record with literally no `envelope` key.
    let legacy = r#"{"severity":"error","code":{"kind":"OutOfRange"},"message":"m","locus":null,"trace":{"frames":[]},"notes":[]}"#;
    let recovered =
        Diag::from_json(legacy).expect("legacy JSON without `envelope` must still parse");
    assert!(recovered.envelope().is_none());
    assert_eq!(
        recovered, d,
        "a legacy record must recover to the same Diag"
    );
}

// ── Grades never upgrade (VR-5) — the record is a reporting field, not a decision site ─────────

/// `grades` on the envelope is a plain reporting field: attaching one does not touch `severity` (the
/// only place a "strength" concept lives on the base `Diag`) — the envelope cannot upgrade anything
/// (VR-5, §10.4 rule 3). Exercised by round-tripping a `Validated` verdict's grades.
#[test]
fn grades_round_trip_without_upgrading_anything() {
    let env = FirstFaultEnvelope::new(
        EventId::new("evt-2"),
        Phase::Runtime,
        SiteKind::SwapCheck,
        Decision::Resolved,
        "swap_check.v0",
        CertMode::Certified,
    )
    .with_grades(Grades {
        input: vec![GuaranteeStrength::Exact],
        output: Some(GuaranteeStrength::Exact),
    });
    let d = Diag::info(Code::Other("SwapCheckValidated".to_owned())).with_envelope(env);
    assert_eq!(
        d.severity(),
        Severity::Info,
        "attaching grades never changes severity"
    );
    let recovered = Diag::from_json(&d.machine()).expect("round-trip");
    let renv = recovered.envelope().expect("envelope present");
    assert_eq!(renv.grades.input, vec![GuaranteeStrength::Exact]);
    assert_eq!(renv.grades.output, Some(GuaranteeStrength::Exact));
}

// ── site_kind / decision escape hatches (Other) round-trip, mirroring Code::Other ──────────────

#[test]
fn site_kind_other_round_trips() {
    let sk = SiteKind::Other("future_site".to_owned());
    assert_eq!(sk.as_str(), "future_site");
    let json = serde_json::to_string(&sk).expect("serializes");
    let back: SiteKind = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(sk, back);
}

#[test]
fn decision_other_round_trips() {
    let d = Decision::Other("future_decision".to_owned());
    assert_eq!(d.as_str(), "future_decision");
    let json = serde_json::to_string(&d).expect("serializes");
    let back: Decision = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(d, back);
}

/// The 13-entry catalog's canonical names match the RFC-0013 Amendment A1 §10.3 table exactly —
/// a single parameterized test over the whole closed set (CLAUDE.md: data-driven, not per-case
/// bespoke bodies).
#[test]
fn site_kind_catalog_names_match_the_amendment_table() {
    let cases: [(SiteKind, &str); 13] = [
        (SiteKind::PolicyResolve, "policy_resolve"),
        (SiteKind::LegalPairRefuse, "legal_pair_refuse"),
        (SiteKind::MissingConversion, "missing_conversion"),
        (SiteKind::RegimeTypeLie, "regime_type_lie"),
        (SiteKind::SwapExec, "swap_exec"),
        (SiteKind::SwapCheck, "swap_check"),
        (SiteKind::MeetBoundary, "meet_boundary"),
        (SiteKind::GradeMeet, "grade_meet"),
        (SiteKind::SealRemint, "seal_remint"),
        (SiteKind::ModeFirewall, "mode_firewall"),
        (SiteKind::GradeAnnotation, "grade_annotation"),
        (SiteKind::ImportFirstEdge, "import_first_edge"),
        (SiteKind::TranspileGap, "transpile_gap"),
    ];
    for (kind, expected) in cases {
        assert_eq!(kind.as_str(), expected);
    }
}

// ── EventId ──────────────────────────────────────────────────────────────────────────────────

#[test]
fn event_id_from_content_hash_uses_the_hash_string() {
    let d = Diag::error(Code::OutOfRange).message("m");
    let eid = EventId::from_content_hash(&d.content_hash());
    assert_eq!(eid.as_str(), d.content_hash().as_str());
}

// ── first_fault_line (RFC-0013 Amendment A1 §10 / DESIGN-03 §3.2 — the W-A exit criterion) ─────

/// An envelope-less `Diag` has no first-fault line (never fabricated — G2).
#[test]
fn first_fault_line_is_none_without_an_envelope() {
    let d = Diag::error(Code::OutOfRange).message("m");
    assert_eq!(d.first_fault_line(), None);
}

/// The exact `where · site_kind · decision` shape, with a locus attached — the W-A exit criterion.
#[test]
fn first_fault_line_renders_where_site_kind_decision_with_a_locus() {
    let d = Diag::error(Code::Other("SwapCheckNotValidated".to_owned()))
        .message("swap certificate did not validate")
        .at(Locus {
            source: Some("swap.myc".into()),
            line: Some(12),
            column: Some(5),
        })
        .with_envelope(swap_check_envelope());
    assert_eq!(
        d.first_fault_line().as_deref(),
        Some("swap.myc:12:5 · swap_check · not_validated")
    );
}

/// Without a locus, `where` renders the explicit unknown marker `"?"` — never a fabricated position.
#[test]
fn first_fault_line_renders_unknown_where_without_a_locus() {
    let d = Diag::error(Code::Other("SwapCheckNotValidated".to_owned()))
        .with_envelope(swap_check_envelope());
    assert_eq!(
        d.first_fault_line().as_deref(),
        Some("? · swap_check · not_validated")
    );
}

/// A partial locus (source only, no line/column) renders just the known part.
#[test]
fn first_fault_line_renders_a_partial_locus() {
    let d = Diag::error(Code::Other("SwapCheckNotValidated".to_owned()))
        .at(Locus {
            source: Some("swap.myc".into()),
            line: None,
            column: None,
        })
        .with_envelope(swap_check_envelope());
    assert_eq!(
        d.first_fault_line().as_deref(),
        Some("swap.myc · swap_check · not_validated")
    );
}
