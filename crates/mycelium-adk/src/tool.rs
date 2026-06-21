//! `adk.tool` — typed tools with declared effects (RFC-0023 §4.1).
//!
//! Every tool failure is an explicit, named error — never a silent `None` or a
//! default value (C1 / G2 / RFC-0016 §4.1).
//!
//! ## Honesty contract (VR-5)
//! - A pure tool returns `Result<Out, ToolError>`.  `Ok` is success; `Err` is an
//!   explicit, named failure.  No variant silently swallows a failure.
//! - An effectful `run_io` (graft/io) is **FLAGGED-deferred** (E7-2/M-664 —
//!   the `graft`/`consume` surface keywords are not yet active; E7-1/M-657 —
//!   generics not yet in the language).  See [`ToolError::Upstream`] for the
//!   placeholder shape.
//!
//! ## Guarantee tag: `Exact` for pure tool calls (no accuracy semantics)
//! A tool's output has no approximation of its own; the honesty tag of its *content*
//! comes from the upstream source (an LLM-produced argument carries `Declared`/`Empirical`,
//! which flows through the `meet` — but the tool dispatch itself is `Exact`).
//!
//! ## FLAG (E7-1 / M-657) — generics
//! The `Tool` trait uses Rust generics (`type In; type Out`).  The Mycelium-language
//! surface (`Tool<In, Out>`, `fn schema_of() -> ToolSchema = derive_schema::<In,Out>()`)
//! needs E7-1/M-657 generics.  The Rust trait below implements the *semantics* now.
//!
//! ## FLAG (E7-1 / M-664 / E7-2 / M-666) — effectful `run_io`
//! The `graft run_io(cap: Substrate{Net}, args: In) -> Result<Out @ Empirical, ToolError>`
//! surface from RFC-0023 §4.1 is deferred: `graft`/`consume` keywords not yet lexed
//! and the effect system (M-660) is not yet active.  This file provides the pure
//! surface only; `run_io` is documented as a flag, not implemented.
//!
//! ## Design spec
//! `docs/rfcs/RFC-0023-Agent-Development-Kit-Phylum.md` §4.1

use std::fmt;

// ── ToolError ─────────────────────────────────────────────────────────────────

/// The explicit error set for a tool invocation (C1 / RFC-0023 §4.1).
///
/// Every variant is a named, inspectable failure — never a sentinel, a `None`,
/// or a default that silently discards the error (G2 / RFC-0016 §4.1 C1).
///
/// # Honesty contract
/// No variant means "proceed as if the tool succeeded."  A failed tool call is an
/// explicit `Err(ToolError)` the caller must branch on (RFC-0023 §6.2 — "The model
/// loop cannot pretend the tool succeeded").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    /// The arguments did not satisfy the tool's schema / type constraints.
    ///
    /// `0` is a human-readable explanation of the constraint that was violated
    /// (G11 dual projection — machine-discriminable tag, human-legible message).
    BadArgs(String),

    /// The requested operation is outside the tool's defined domain.
    ///
    /// For example: a weather tool asked for a city it has no data for, a math
    /// tool asked to divide by zero.  `0` names the out-of-domain input.
    OutOfDomain(String),

    /// The tool refused to execute (e.g. a safety policy, an auth failure, a
    /// capability not granted by the current `ToolContext`).
    ///
    /// A refusal is **not** a success — the caller must handle it explicitly.
    /// `0` names the refusal reason.
    Refused(String),

    /// An upstream dependency (network, external API, file system) failed.
    ///
    /// For external-infra tools (`graft`, M-664) this is the concrete failure
    /// from the substrate.  `0` carries the upstream error description.
    ///
    /// FLAG (E7-2 / M-664): the `graft run_io` form that *produces* this variant
    /// is deferred — the `graft`/`consume` keyword surface is not yet active.
    /// This variant is present so the pure-tool error set is complete and the
    /// guarantee matrix can reference it.
    Upstream(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolError::BadArgs(msg) => write!(f, "bad tool arguments: {msg}"),
            ToolError::OutOfDomain(msg) => write!(f, "tool call out of domain: {msg}"),
            ToolError::Refused(msg) => write!(f, "tool call refused: {msg}"),
            ToolError::Upstream(msg) => write!(f, "upstream error: {msg}"),
        }
    }
}

mycelium_std_core::impl_std_error!(ToolError);

// ── Tool trait ────────────────────────────────────────────────────────────────

/// A pure, typed tool surface (RFC-0023 §4.1).
///
/// Every implementation must:
/// - Return an explicit `Result<Self::Out, ToolError>` — never `Option`, never
///   a silent default (C1 / G2).
/// - Have no hidden effects in `run`; effectful tools use `run_io` (deferred,
///   E7-2/M-664).
/// - Carry a `NAME` identifying the tool in logs, schemas, and EXPLAIN records.
///
/// ## Guarantee tag (per-op)
/// `run` is `Exact` when the tool has no accuracy semantics of its own and its
/// input is well-formed.  A tool whose output depends on approximate upstream
/// data (e.g. an LLM-generated argument) inherits the input's tag via the
/// `GuaranteeStrength::meet` — that is the *caller's* obligation, not this trait.
///
/// ## FLAG (E7-1 / M-657) — Mycelium-language surface
/// The Mycelium target syntax is:
/// ```mycelium
/// fn run_pure(args: In) -> Result<Out @ Declared, ToolError> = …
/// fn schema_of() -> ToolSchema = derive_schema::<In, Out>()
/// ```
/// The `derive_schema` derivation needs generics (M-657); the Rust trait below
/// implements the *semantics* without that surface.
pub trait Tool {
    /// The input type this tool accepts.
    type In;
    /// The output type this tool produces on success.
    type Out;

    /// A human- and machine-readable name for this tool (used in EXPLAIN records,
    /// error messages, and the model-facing schema).
    const NAME: &'static str;

    /// Run the tool with the given arguments.  Returns an explicit `Result` —
    /// `Ok(Self::Out)` on success, `Err(ToolError)` on any failure (C1 / G2).
    ///
    /// # Honesty (VR-5)
    /// The caller must handle `Err`.  There is no variant of this API that
    /// silently swallows a failure and returns a default (RFC-0023 §6.2).
    ///
    /// # Effects
    /// `run` is **pure** — no I/O, no side effects.  Effectful tools (network,
    /// FS) are expressed via `run_io` (FLAG-deferred, E7-2/M-664).
    fn run(&self, args: Self::In) -> Result<Self::Out, ToolError>;
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{Tool, ToolError};
    use proptest::prelude::*;

    // ── A minimal concrete tool for testing ───────────────────────────────────

    /// A pure doubling tool: doubles a `u32`.  Used to verify the trait surface
    /// and proptest the "a failed call is always Err" bound.
    struct DoubleTool;

    impl Tool for DoubleTool {
        type In = u32;
        type Out = u32;
        const NAME: &'static str = "double";

        fn run(&self, args: u32) -> Result<u32, ToolError> {
            args.checked_mul(2)
                .ok_or_else(|| ToolError::OutOfDomain(format!("{args} overflows u32 on doubling")))
        }
    }

    /// A pure refusing tool: always refuses.
    struct AlwaysRefuse;

    impl Tool for AlwaysRefuse {
        type In = ();
        type Out = u32;
        const NAME: &'static str = "refuse";

        fn run(&self, _: ()) -> Result<u32, ToolError> {
            Err(ToolError::Refused("this tool always refuses".to_owned()))
        }
    }

    // ── Unit tests ────────────────────────────────────────────────────────────

    /// A successful tool call is `Ok`.
    #[test]
    fn pure_tool_succeeds_on_valid_input() {
        let t = DoubleTool;
        assert_eq!(t.run(21), Ok(42));
    }

    /// An out-of-domain call returns `Err(OutOfDomain)`, never a silent default.
    #[test]
    fn pure_tool_returns_err_on_overflow_never_silent() {
        let t = DoubleTool;
        let result = t.run(u32::MAX);
        assert!(
            matches!(result, Err(ToolError::OutOfDomain(_))),
            "overflow must be Err(OutOfDomain), not Ok or a truncated value; got {result:?}"
        );
    }

    /// A refusing tool returns `Err(Refused)` — the caller must handle it.
    #[test]
    fn refusing_tool_returns_err_refused() {
        let t = AlwaysRefuse;
        assert!(matches!(t.run(()), Err(ToolError::Refused(_))));
    }

    /// `ToolError` implements `std::error::Error` (compile-time check).
    #[test]
    fn tool_error_is_std_error() {
        let e = ToolError::BadArgs("missing field".to_owned());
        let _: &dyn std::error::Error = &e;
    }

    /// `ToolError` Display is non-empty for every variant (G11 — human-legible EXPLAIN).
    #[test]
    fn tool_error_display_is_non_empty_for_every_variant() {
        let variants = [
            ToolError::BadArgs("x".to_owned()),
            ToolError::OutOfDomain("y".to_owned()),
            ToolError::Refused("z".to_owned()),
            ToolError::Upstream("w".to_owned()),
        ];
        for v in &variants {
            let s = v.to_string();
            assert!(
                !s.is_empty(),
                "ToolError::{v:?} must have non-empty Display (G11)"
            );
        }
    }

    /// `ToolError::BadArgs` Display includes the message payload.
    #[test]
    fn tool_error_bad_args_display_includes_message() {
        let e = ToolError::BadArgs("missing 'city' field".to_owned());
        assert!(e.to_string().contains("missing 'city' field"));
    }

    /// `ToolError::Refused` Display includes the refusal reason.
    #[test]
    fn tool_error_refused_display_includes_reason() {
        let e = ToolError::Refused("no net capability".to_owned());
        assert!(e.to_string().contains("no net capability"));
    }

    // ── Property tests ────────────────────────────────────────────────────────

    proptest! {
        /// BOUND: `DoubleTool::run` is never silent — it always returns an explicit
        /// `Result`; for every input, the result is either `Ok(2*x)` or `Err(OutOfDomain)`.
        /// Guard: returning `Ok` with a truncated/wrong value, or panicking, fails this.
        #[test]
        fn prop_double_tool_never_silent(x in 0u32..=u32::MAX) {
            let t = DoubleTool;
            match t.run(x) {
                Ok(v) => prop_assert_eq!(v, 2 * x, "Ok case must return exactly 2*x"),
                Err(ToolError::OutOfDomain(_)) => {
                    // Correct: overflow is explicit OutOfDomain.
                    // Verify it is indeed an overflow case.
                    prop_assert!(x.checked_mul(2).is_none(), "OutOfDomain must only fire on overflow");
                }
                Err(e) => {
                    prop_assert!(false, "unexpected error variant: {e:?}");
                }
            }
        }

        /// BOUND: A tool failure is always `Err` — the caller cannot read a failed
        /// tool call as a success.  Guard: any implementation that returns `Ok` for
        /// an out-of-domain input breaks this bound.
        #[test]
        fn prop_always_refuse_tool_is_always_err(_x in 0u8..=255u8) {
            let t = AlwaysRefuse;
            prop_assert!(t.run(()).is_err(), "AlwaysRefuse must always return Err (C1 / G2)");
        }

        /// BOUND: `ToolError` variants carry their payload in Display (no dropped fields).
        #[test]
        fn prop_tool_error_display_includes_payload(msg in "[a-z]{1,20}") {
            for err in [
                ToolError::BadArgs(msg.clone()),
                ToolError::OutOfDomain(msg.clone()),
                ToolError::Refused(msg.clone()),
                ToolError::Upstream(msg.clone()),
            ] {
                let display = err.to_string();
                prop_assert!(
                    display.contains(&msg),
                    "ToolError {err:?} Display must include the payload {msg:?}; got {display:?}"
                );
            }
        }
    }
}
