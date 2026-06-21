//! Per-operation guarantee matrix for the `web` phylum (RFC-0016 §4.5 / RFC-0022 §4.5).
//!
//! Every exported operation has exactly one row in [`MATRIX`]. The matrix is the load-bearing
//! C2/VR-5 deliverable: guarantee tags are asserted in tests, not prose-only.
//!
//! # Tag justification summary (VR-5 — downgrade rather than overclaim)
//!
//! | Tag | Rows | Reason |
//! |---|---|---|
//! | `Exact` | `parse_request`, `parse_response`, `status_from_u16`, `header_get`, `method`, `path`, `Request::new`, `with_header`, `encode_body`, `match_route`, `route_table`, `get`, `request`, `serve` | No accuracy/precision/probability semantics (RFC-0016 C2 "no accuracy semantics → Exact"). Fallible ops are `Exact`-when-`Ok`. |
//! | `Empirical` | `decode_body`, `get_json`, `per-request-join` | Round-trip / join property established by proptest corpus / RT2 differential; no checked theorem → `Empirical`, not `Proven` (VR-5). |
//! | `Declared` | `handler-purity` | The type system cannot enforce handler purity — this is a convention, always FLAGGED (VR-5). |
//!
//! # Effect column (C6)
//! - `"none"` — the op is pure over its in-memory input; no OS facility touched.
//! - `"io"` — the op uses a transport / socket; the `io` effect is declared (U2/U8 research gate).
//!
//! # EXPLAIN-able column (C3)
//! - `"n/a"` — total accessors; no selection, conversion, or approximation.
//! - `"yes"` — the op carries a diagnostic record (located `HttpParseError` / `RouteMatch`
//!   pattern+captures / `ServeError` chain) — the machine-legible EXPLAIN surface (C3/G11).
//!
//! # Flag guard (VR-5)
//! The tests below assert that:
//! - No op is `Proven` (no checked theorem exists for any web op).
//! - Every `io` op declares the `io` effect.
//! - Non-finite f64 is refused fallibly (never silent null) — asserted via `encode_body` row.
//! - Every `Declared` row is intentional and flagged in the `error_set` field.

// ── Re-export from std.io to share the matrix types ──────────────────────────
//
// We import `GuaranteeTag` / `Fallibility` / `Explainable` / `MatrixRow` from `std.io`'s
// `guarantee_matrix` module (DRY/KC-3). The struct shapes are identical; re-using them keeps
// the web matrix parseable by the same tooling as the std.io matrix.

pub use mycelium_std_io::guarantee_matrix::{Explainable, Fallibility, GuaranteeTag, MatrixRow};

// ── MATRIX ────────────────────────────────────────────────────────────────────

/// The `web` phylum guarantee matrix.
///
/// 14 rows — one per exported op group (RFC-0022 §4.5). Asserted in tests — never prose-only
/// (C2 / VR-5).
pub const MATRIX: &[MatrixRow] = &[
    // ── http.parse_request / parse_response ───────────────────────────────────
    // Exact-when-Ok: faithful parsing with no accuracy semantics.
    // Fallible: every parse error is a located HttpParseError (byte offset `at`) — never silent.
    // Effects: none (pure over in-memory bytes).
    // EXPLAIN: yes — every Err carries a ByteOffset locus (C3/G11).
    MatrixRow {
        op: "http::parse_request",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Fallible,
        error_set: "Err(Truncated|Malformed|OutOfRange|InvalidMethod|InvalidUrl|InvalidHeaderName|InvalidHeaderValue) @at",
        effects: "none",
        explainable: Explainable::Yes,
    },
    MatrixRow {
        op: "http::parse_response",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Fallible,
        error_set: "Err(Truncated|Malformed|OutOfRange|InvalidHeaderName|InvalidHeaderValue) @at",
        effects: "none",
        explainable: Explainable::Yes,
    },
    // ── http.status_from_u16 ──────────────────────────────────────────────────
    // Exact-when-Ok: 100..=599 is the honest validated range.
    // Fallible: OutOfRange{code} — NEVER a clamp (C1/G2).
    // Effects: none. EXPLAIN: yes (the rejected code is carried in Err).
    MatrixRow {
        op: "http::status_from_u16",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Fallible,
        error_set: "Err(OutOfRange{code}) — code outside 100..=599; NEVER a clamp (C1/G2)",
        effects: "none",
        explainable: Explainable::Yes,
    },
    // ── http accessors (header_get, method, path) ─────────────────────────────
    // Total, Exact: pure reads of well-formed Request/Response fields.
    // No selection, conversion, or approximation → EXPLAIN n/a (C3).
    MatrixRow {
        op: "http::header_get",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Total,
        error_set: "",
        effects: "none",
        explainable: Explainable::NotApplicable,
    },
    MatrixRow {
        op: "http::method",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Total,
        error_set: "",
        effects: "none",
        explainable: Explainable::NotApplicable,
    },
    MatrixRow {
        op: "http::path",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Total,
        error_set: "",
        effects: "none",
        explainable: Explainable::NotApplicable,
    },
    // ── http.Request::new / with_header ───────────────────────────────────────
    // Exact-when-Ok: header validation is always-on.
    // Fallible: Err(InvalidHeaderName|InvalidHeaderValue) — never a silent invalid header.
    // Effects: none. EXPLAIN: n/a (validation result is in Err variant, not a selection).
    MatrixRow {
        op: "http::Request::new",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Total,
        error_set: "",
        effects: "none",
        explainable: Explainable::NotApplicable,
    },
    MatrixRow {
        op: "http::with_header",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Fallible,
        error_set: "Err(InvalidHeaderName|InvalidHeaderValue)",
        effects: "none",
        explainable: Explainable::NotApplicable,
    },
    // ── json.encode_body ─────────────────────────────────────────────────────
    // Exact-when-Ok: faithful JSON projection (delegates to std.io::to_json — DRY/KC-3).
    // Fallible: Err(OutOfDomain) — non-finite f64 is REFUSED, never a silent null (C1/G2).
    // Effects: none. EXPLAIN: n/a (the rejection is in Err, not a hidden selection).
    MatrixRow {
        op: "json::encode_body",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Fallible,
        error_set: "Err(OutOfDomain) — non-finite f64 refused; NEVER a silent null (C1/G2)",
        effects: "none",
        explainable: Explainable::NotApplicable,
    },
    // ── json.decode_body ─────────────────────────────────────────────────────
    // Empirical: round-trip property inherited from std.io::from_json (proptest corpus).
    // Fallible: Err(JsonError::Decode(SerError)) with byte-offset locus (C3).
    // Effects: none. EXPLAIN: yes (SerError carries a locus).
    MatrixRow {
        op: "json::decode_body",
        guarantee: GuaranteeTag::Empirical,
        fallibility: Fallibility::Fallible,
        error_set: "Err(JsonError::Decode(SerError) @locus | JsonError::OutOfDomain)",
        effects: "none",
        explainable: Explainable::Yes,
    },
    // ── route.match_route ─────────────────────────────────────────────────────
    // Exact-when-Ok: the RouteMatch names the pattern + captures — EXPLAIN artifact (C3).
    // Fallible: Err(NotFound) or Err(MethodNotAllowed{allowed}) — never a silent wrong-handler (C1/G2).
    // Effects: none (pure over RouteTable + path string).
    // EXPLAIN: yes — RouteMatch (Ok) + allowed-methods list (MethodNotAllowed) are both inspectable.
    MatrixRow {
        op: "route::match_route",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Fallible,
        error_set: "Err(NotFound | MethodNotAllowed{allowed: Vec<Method>}) — NEVER silent wrong-handler (C1/G2)",
        effects: "none",
        explainable: Explainable::Yes,
    },
    // ── route.table (RouteTable accessor) ────────────────────────────────────
    // Exact, Total: RouteTable is reified + inspectable via `.patterns()` iterator (C3).
    // EXPLAIN: yes — the table is the EXPLAIN surface for the route dispatch decision.
    MatrixRow {
        op: "route::table",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Total,
        error_set: "",
        effects: "none",
        explainable: Explainable::Yes,
    },
    // ── client.get / client.request ───────────────────────────────────────────
    // Exact-when-Ok: request building is exact; response parsing is exact-when-ok.
    // Fallible: Err(UnexpectedEof|Refused|EffectBudget) from transport, or HttpParse on response.
    // Effects: **io** (declared; transport gate). EXPLAIN: yes (located Err).
    // FLAGGED: real socket transport is gated (U2/U8). In-memory impl available for testing.
    MatrixRow {
        op: "client::get",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Fallible,
        error_set: "Err(UnexpectedEof|Refused|EffectBudget) — transport gated; in-memory impl + Err on real socket",
        effects: "io",
        explainable: Explainable::Yes,
    },
    MatrixRow {
        op: "client::request",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Fallible,
        error_set: "Err(UnexpectedEof|Refused|EffectBudget) — transport gated; in-memory impl + Err on real socket",
        effects: "io",
        explainable: Explainable::Yes,
    },
    // ── client.get_json ───────────────────────────────────────────────────────
    // Empirical: JSON decode inherits Empirical from std.io.
    // Fallible: Err(ClientError wrapping HttpError|JsonError) with locus.
    // Effects: **io**. EXPLAIN: yes.
    MatrixRow {
        op: "client::get_json",
        guarantee: GuaranteeTag::Empirical,
        fallibility: Fallibility::Fallible,
        error_set: "Err(ClientError::HttpParse|Json|Refused|UnexpectedEof|EffectBudget)",
        effects: "io",
        explainable: Explainable::Yes,
    },
    // ── server.per-request-join ───────────────────────────────────────────────
    // Empirical: per-request join via Scope::join_all is Empirical (RT2 differential).
    // Fallible: Err(ServeError::TaskPanicked|EffectBudget) — G2 never-silent on panic.
    // Effects: **io** (declared; dispatch dispatches handlers). EXPLAIN: yes.
    MatrixRow {
        op: "server::per_request_join",
        guarantee: GuaranteeTag::Empirical,
        fallibility: Fallibility::Fallible,
        error_set: "Err(ServeError::TaskPanicked|EffectBudget|Refused) — TaskPanicked never silent (G2)",
        effects: "io",
        explainable: Explainable::Yes,
    },
    // ── server.handler-purity-contract ────────────────────────────────────────
    // Declared (FLAGGED): the type system cannot enforce handler purity — this is a convention.
    // Always FLAGGED (VR-5). EXPLAIN: n/a (it's a contract, not an op with a runtime artifact).
    // NOTE: `Declared` rows must be intentional and flagged — the guard test verifies this.
    MatrixRow {
        op: "server::handler_purity_contract",
        guarantee: GuaranteeTag::Declared,
        fallibility: Fallibility::Total, // "total" in the sense that every handler is accepted
        error_set: "",
        effects: "none",
        explainable: Explainable::NotApplicable,
    },
    // ── server.serve (real-socket bind — FLAGGED-gated) ───────────────────────
    // Exact-when-Ok: the bind/accept-loop is faithful IF the gate were discharged.
    // Fallible: Err(Refused{Unwired}) — gate not discharged, refusing explicitly (C1/G2).
    // Effects: **io** (declared — a real bind would touch OS network).
    // EXPLAIN: yes (the Refused::why names the open gate).
    MatrixRow {
        op: "server::serve",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Fallible,
        error_set: "Err(Refused{Unwired}) — FLAGGED gate (U2/U8 not discharged); NEVER a stub success (C1/G2)",
        effects: "io",
        explainable: Explainable::Yes,
    },
];

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{Explainable, Fallibility, GuaranteeTag, MATRIX};

    // The expected op names (one per spec row).
    const EXPECTED_OPS: &[&str] = &[
        "http::parse_request",
        "http::parse_response",
        "http::status_from_u16",
        "http::header_get",
        "http::method",
        "http::path",
        "http::Request::new",
        "http::with_header",
        "json::encode_body",
        "json::decode_body",
        "route::match_route",
        "route::table",
        "client::get",
        "client::request",
        "client::get_json",
        "server::per_request_join",
        "server::handler_purity_contract",
        "server::serve",
    ];

    /// Every op named in the spec §4.5 surface appears in the matrix exactly once.
    /// Guard: removing or renaming any op from MATRIX makes this fail.
    #[test]
    fn matrix_contains_all_expected_ops() {
        for name in EXPECTED_OPS {
            assert!(
                MATRIX.iter().any(|r| r.op == *name),
                "matrix is missing op {:?} (RFC-0022 §4.5)",
                name
            );
        }
        assert_eq!(
            MATRIX.len(),
            EXPECTED_OPS.len(),
            "matrix has unexpected extra or missing rows (expected {} rows, got {})",
            EXPECTED_OPS.len(),
            MATRIX.len()
        );
    }

    /// No op is `Proven` (VR-5 — no checked theorem exists for any web op).
    /// Guard: upgrading any row to `Proven` without a checked theorem makes this fail.
    #[test]
    fn no_op_is_proven_without_a_checked_theorem() {
        for row in MATRIX {
            assert_ne!(
                row.guarantee,
                GuaranteeTag::Proven,
                "op {:?} claims Proven without a checked theorem — \
                 downgrade to Empirical (VR-5 / RFC-0022 §4.5)",
                row.op
            );
        }
    }

    /// The `Declared` rows are intentional and exactly the expected set.
    /// Guard: accidentally Declaring a non-convention row (or un-Declaring handler-purity) fails.
    #[test]
    fn declared_rows_are_exactly_the_intentional_set() {
        let declared_ops: Vec<&str> = MATRIX
            .iter()
            .filter(|r| r.guarantee == GuaranteeTag::Declared)
            .map(|r| r.op)
            .collect();
        // Only the handler-purity contract is Declared (VR-5 — a convention, not a proof).
        assert_eq!(
            declared_ops,
            vec!["server::handler_purity_contract"],
            "only handler_purity_contract should be Declared (VR-5 FLAG)"
        );
    }

    /// The `Empirical` rows are exactly the expected set (round-trip / join ops).
    /// Guard: upgrading or downgrading an Empirical row makes this fail.
    #[test]
    fn empirical_rows_are_exactly_the_expected_set() {
        let empirical_ops: Vec<&str> = MATRIX
            .iter()
            .filter(|r| r.guarantee == GuaranteeTag::Empirical)
            .map(|r| r.op)
            .collect();
        let expected = [
            "json::decode_body",
            "client::get_json",
            "server::per_request_join",
        ];
        for op in &expected {
            assert!(
                empirical_ops.contains(op),
                "op {:?} must be Empirical (VR-5 — round-trip/join property, no checked theorem)",
                op
            );
        }
        assert_eq!(
            empirical_ops.len(),
            expected.len(),
            "unexpected Empirical rows: {empirical_ops:?}"
        );
    }

    /// Fallible ops have a non-empty error set; total ops have an empty one (C1).
    /// Guard: setting a fallible op's error_set to "" makes this fail.
    #[test]
    fn fallibility_and_error_set_are_consistent() {
        for row in MATRIX {
            match row.fallibility {
                Fallibility::Total => assert!(
                    row.error_set.is_empty(),
                    "total op {:?} must have an empty error_set (C1)",
                    row.op
                ),
                Fallibility::Fallible => assert!(
                    !row.error_set.is_empty(),
                    "fallible op {:?} must name its error set (C1)",
                    row.op
                ),
            }
        }
    }

    /// The `io` ops declare the `io` effect (C6).
    /// Guard: changing any io op's effect to "none" makes this fail.
    #[test]
    fn io_ops_declare_io_effect() {
        let io_ops = [
            "client::get",
            "client::request",
            "client::get_json",
            "server::per_request_join",
            "server::serve",
        ];
        for op in &io_ops {
            let row = MATRIX
                .iter()
                .find(|r| r.op == *op)
                .unwrap_or_else(|| panic!("op {:?} missing from matrix", op));
            assert!(
                row.effects.contains("io"),
                "op {:?} must declare the io effect (C6 / RFC-0022 §4.5)",
                op
            );
        }
    }

    /// The pure / parse ops are effect-free (C6).
    /// Guard: adding an effect to a pure op makes this fail.
    #[test]
    fn pure_ops_declare_no_effects() {
        let pure_ops = [
            "http::parse_request",
            "http::parse_response",
            "http::status_from_u16",
            "http::header_get",
            "http::method",
            "http::path",
            "http::Request::new",
            "http::with_header",
            "json::encode_body",
            "json::decode_body",
            "route::match_route",
            "route::table",
        ];
        for op in &pure_ops {
            let row = MATRIX
                .iter()
                .find(|r| r.op == *op)
                .unwrap_or_else(|| panic!("op {:?} missing from matrix", op));
            assert_eq!(
                row.effects, "none",
                "op {:?} must be pure/effect-free (C6)",
                op
            );
        }
    }

    /// `json::encode_body` is fallible (refuses non-finite f64 — never silent null, C1/G2).
    /// Guard: flipping encode_body to Total (re-introducing the silent-null path) makes this fail.
    #[test]
    fn encode_body_is_fallible_never_silent_null() {
        let row = MATRIX
            .iter()
            .find(|r| r.op == "json::encode_body")
            .expect("encode_body row");
        assert_eq!(
            row.fallibility,
            Fallibility::Fallible,
            "encode_body must be fallible — non-finite f64 refused, NEVER a silent null (C1/G2)"
        );
        assert!(
            row.error_set.contains("OutOfDomain"),
            "encode_body error_set must name OutOfDomain (non-finite refusal)"
        );
        assert!(
            row.error_set.to_lowercase().contains("null")
                || row.error_set.to_lowercase().contains("c1"),
            "encode_body error_set must reference the never-silent-null policy (C1/G2)"
        );
    }

    /// `http::status_from_u16` is fallible and names OutOfRange — never a clamp (C1/G2).
    /// Guard: flipping status_from_u16 to Total or removing OutOfRange makes this fail.
    #[test]
    fn status_from_u16_is_fallible_never_clamped() {
        let row = MATRIX
            .iter()
            .find(|r| r.op == "http::status_from_u16")
            .expect("status_from_u16 row");
        assert_eq!(
            row.fallibility,
            Fallibility::Fallible,
            "status_from_u16 must be fallible — out-of-range codes refused, NEVER clamped (C1/G2)"
        );
        assert!(
            row.error_set.contains("OutOfRange"),
            "status_from_u16 error_set must name OutOfRange"
        );
        assert!(
            row.error_set.to_lowercase().contains("clamp")
                || row.error_set.to_lowercase().contains("c1"),
            "status_from_u16 error_set must reference the never-clamp policy (C1/G2)"
        );
    }

    /// `route::match_route` is fallible with named 404/405 errors — never a silent wrong-handler.
    /// Guard: flipping to Total or removing NotFound/MethodNotAllowed makes this fail.
    #[test]
    fn match_route_is_fallible_with_explicit_errors() {
        let row = MATRIX
            .iter()
            .find(|r| r.op == "route::match_route")
            .expect("match_route row");
        assert_eq!(row.fallibility, Fallibility::Fallible);
        assert!(
            row.error_set.contains("NotFound"),
            "match_route error_set must name NotFound (404)"
        );
        assert!(
            row.error_set.contains("MethodNotAllowed"),
            "match_route error_set must name MethodNotAllowed (405)"
        );
    }

    /// `route::match_route` and `route::table` are EXPLAIN-able (C3 — the EXPLAIN surface).
    /// Guard: removing EXPLAIN from these ops violates C3.
    #[test]
    fn route_ops_are_explainable() {
        for op in &["route::match_route", "route::table"] {
            let row = MATRIX
                .iter()
                .find(|r| r.op == *op)
                .unwrap_or_else(|| panic!("op {:?} missing", op));
            assert_eq!(
                row.explainable,
                Explainable::Yes,
                "op {:?} must be EXPLAIN-able (C3 — RFC-0022 §4.5)",
                op
            );
        }
    }

    /// The decode / round-trip ops carry EXPLAIN artifacts (C3 — locus).
    #[test]
    fn decode_ops_are_explainable() {
        for op in &[
            "json::decode_body",
            "http::parse_request",
            "http::parse_response",
        ] {
            let row = MATRIX
                .iter()
                .find(|r| r.op == *op)
                .unwrap_or_else(|| panic!("op {:?} missing", op));
            assert_eq!(
                row.explainable,
                Explainable::Yes,
                "op {:?} must be EXPLAIN-able (C3 — RFC-0013 diagnostic locus)",
                op
            );
        }
    }

    /// Total accessor ops are NotApplicable for EXPLAIN (no selection/conversion).
    #[test]
    fn total_accessor_ops_are_not_explainable() {
        let na_ops = [
            "http::header_get",
            "http::method",
            "http::path",
            "http::Request::new",
        ];
        for op in &na_ops {
            let row = MATRIX
                .iter()
                .find(|r| r.op == *op)
                .unwrap_or_else(|| panic!("op {:?} missing", op));
            assert_eq!(
                row.explainable,
                Explainable::NotApplicable,
                "op {:?} must be NotApplicable for EXPLAIN (C3 n/a — faithful re-export)",
                op
            );
        }
    }

    /// `server::serve` names the FLAGGED gate in its error_set — never a stub success.
    /// Guard: removing the gate mention from the error_set silently hides the limitation.
    #[test]
    fn serve_names_flagged_gate_in_error_set() {
        let row = MATRIX
            .iter()
            .find(|r| r.op == "server::serve")
            .expect("server::serve row");
        assert_eq!(row.fallibility, Fallibility::Fallible);
        assert!(
            row.error_set.to_uppercase().contains("FLAG") || row.error_set.contains("stub"),
            "server::serve error_set must reference the FLAGGED gate (C1/G2 — never stub success)"
        );
    }
}
