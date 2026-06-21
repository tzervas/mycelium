//! nodule: `web.server` — the server as a colony of request-handling hyphae
//! (RFC-0008 R1), per RFC-0022 §4.1 / §4.5.
//!
//! # Design
//! - Dispatch is structured on [`mycelium_std_runtime::colony::Scope`]: each request is a
//!   spawned `Task` in a `Scope<(), ()>`. Per-request join is **`Empirical`** via RT2
//!   (matching `Scope::join_all`'s honest tag).
//! - **In-memory dispatch** is fully implemented and testable: `dispatch_request` routes
//!   a parsed [`Request`] through a [`RouteTable`] to its handler.
//! - **Real socket bind/accept-loop** is **FLAGGED-gated** (U2 socket-floor, U8 net-effect):
//!   `serve` always returns explicit `Err(ServeError::Refused { why })` — never a stub
//!   success (C1/G2 — never-silent).
//! - Handler-purity contract is **`Declared`** (FLAGGED): the type system cannot enforce that
//!   a handler is pure; the contract is a convention, not a proof.
//!
//! # Guarantee summary (RFC-0022 §4.5)
//! - `dispatch_request`: `Exact`-when-`Ok`, Fallible `Err(NotFound|MethodNotAllowed)`, none, yes.
//! - per-request join (`Scope::join_all`): `Empirical`, Fallible `Err(ServeError|TaskPanicked|EffectBudget)`, **io**.
//! - handler-purity contract: `Declared` (FLAGGED — convention, not proven).
//! - `serve` (real-socket bind): `Exact`-when-`Ok`, Fallible `Err(ServeError::Refused { why })`, **io**, yes
//!   (FLAGGED-gated).

use std::fmt;

use mycelium_std_runtime::{colony::Scope, task::Task};

use crate::http::{Request, Response};
use crate::route::{match_route, HttpError, RouteError, RouteMatch, RouteTable};

// ── ServeError ────────────────────────────────────────────────────────────────

/// Error type for server operations (C1 — never-silent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServeError {
    /// A route dispatch failure (wraps [`RouteError`]).
    Route(RouteError),
    /// A handler returned an error (wraps [`HttpError`]).
    Handler(HttpError),
    /// The server's task join detected a panicked request task (never-silent — G2).
    TaskPanicked,
    /// A declared effect budget was exceeded (RFC-0014 §4.5 / C6).
    EffectBudget {
        /// The budget kind exceeded.
        kind: String,
    },
    /// The real socket bind/accept-loop is FLAGGED-gated (U2/U8 gate not discharged — C1/G2).
    ///
    /// # FLAGGED: U2 (socket bind / accept-loop), U8 (net-effect OS permission).
    /// These gates are NOT discharged. `serve` returns this variant rather than attempting
    /// a real bind — never a stub success (C1/G2 — never-silent). The `why` message names
    /// the open research gate.
    Refused {
        /// Why the operation was refused.
        why: String,
    },
}

impl fmt::Display for ServeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServeError::Route(e) => write!(f, "route error: {e}"),
            ServeError::Handler(e) => write!(f, "handler error: {e}"),
            ServeError::TaskPanicked => write!(f, "a request task panicked (never-silent — G2)"),
            ServeError::EffectBudget { kind } => write!(f, "effect budget exceeded ({kind})"),
            ServeError::Refused { why } => write!(f, "server refused: {why}"),
        }
    }
}

mycelium_std_core::impl_std_error!(
    ServeError,
    source = |this| {
        match this {
            ServeError::Route(e) => Some(e),
            ServeError::Handler(e) => Some(e),
            _ => None,
        }
    }
);

impl From<RouteError> for ServeError {
    fn from(e: RouteError) -> Self {
        ServeError::Route(e)
    }
}

impl From<HttpError> for ServeError {
    fn from(e: HttpError) -> Self {
        ServeError::Handler(e)
    }
}

// ── Listener abstraction (FLAGGED-gated) ──────────────────────────────────────

/// A placeholder for a real TCP listener (FLAGGED-gated U2/U8).
///
/// # FLAGGED: U2 (socket bind), U8 (net-effect).
/// This type carries the insertion point for a future OS-backed listener. Until the gate is
/// discharged, `serve` always returns `Err(ServeError::Refused { why })`.
#[derive(Debug)]
pub struct Listener {
    /// The bind address (for diagnostic purposes only — not connected).
    address: String,
}

impl Listener {
    /// Create a (FLAGGED) listener placeholder.
    ///
    /// # FLAGGED: constructing a `Listener` does NOT bind a socket. The `serve` function
    /// will refuse with `Err(ServeError::Refused { why })`. This type is the insertion point for a
    /// future OS-backed listener once RP-10 + ADR-014 are ratified.
    #[must_use]
    pub fn new(address: impl Into<String>) -> Self {
        Listener {
            address: address.into(),
        }
    }

    /// The bind address.
    #[must_use]
    pub fn address(&self) -> &str {
        &self.address
    }
}

// ── In-memory dispatch ────────────────────────────────────────────────────────

/// Dispatch a single request through the route table, invoking the matched handler.
///
/// # Guarantee: `Exact`-when-`Ok`
/// The routing decision and handler invocation are both inspectable: a `RouteMatch` is
/// produced internally (the EXPLAIN artifact — C3), and the handler receives the full
/// `Request`. Any dispatch failure (404/405) or handler error is an explicit `Err` (C1/G2).
///
/// # Fallibility: `Err(ServeError::Route(NotFound|MethodNotAllowed) | ServeError::Handler(…))`
/// Never silently dispatches to the wrong handler or silently swallows a handler error (C1/G2).
///
/// # Effects: none (pure over the in-memory `RouteTable` and `Request`).
/// Handler I/O is the handler's own declared effect — not this function's.
///
/// # EXPLAIN-able: yes (C3)
/// The `RouteMatch` (pattern + captures) is produced and available to the caller via the
/// returned response's provenance (in a production system; here it is used internally and
/// surfaced on error).
pub fn dispatch_request(table: &RouteTable, request: Request) -> Result<Response, ServeError> {
    let method = request.method().clone();
    let path = request.path().to_owned();

    let route_match: RouteMatch = match_route(table, &method, &path).map_err(ServeError::from)?;

    let handler = route_match.handler;
    handler(request).map_err(ServeError::from)
}

/// Dispatch a batch of requests through the route table, structured as a colony of hyphae.
///
/// Each request is spawned as a `Task` in a `Scope<(), ServeError>` (the Mycelium structured
/// concurrency primitive). Responses are collected in spawn order (FIFO — `Exact` sweep order).
///
/// # Guarantee (per-request join): `Empirical` via RT2 (matching `Scope::join_all`)
/// The join-all property is `Empirical` — not `Proven` (VR-5). See `mycelium-std-runtime`.
///
/// # Fallibility: `Err(ServeError::TaskPanicked)` on any panicking task (G2 — never-silent).
///
/// # Effects: **none** (in-memory only; real socket accept-loop is FLAGGED-gated).
///
/// # FLAGGED: handler-purity contract is `Declared` (convention, not proven — VR-5).
/// Tasks may call handlers that touch global state; we cannot prevent this at the type level.
/// The `Declared` tag is the honest maximum here.
pub fn dispatch_batch(
    table: &RouteTable,
    requests: Vec<Request>,
) -> Result<Vec<Result<Response, ServeError>>, ServeError> {
    // Use a shared-state approach: each task processes a request and stores its result.
    // (v0: Scope<(),()> executes tasks sequentially — the Empirical guarantee matches RT2.)
    let results: std::sync::Arc<std::sync::Mutex<Vec<Result<Response, ServeError>>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::with_capacity(requests.len())));

    let mut scope: Scope<(), ()> = Scope::new();

    for req in requests {
        let table_clone = table.clone();
        let results_clone = results.clone();
        scope.spawn(Task::new(move || {
            let result = dispatch_request(&table_clone, req);
            results_clone.lock().unwrap().push(result);
        }));
    }

    scope.join_all().map_err(|_| ServeError::TaskPanicked)?;

    let guard = results.lock().unwrap();
    Ok(guard.clone())
}

// ── serve (FLAGGED-gated real socket) ────────────────────────────────────────

/// Start serving requests from `listener` using `table` — **FLAGGED-gated (U2/U8)**.
///
/// # FLAGGED: real socket bind/accept-loop is NOT implemented (U2, U8 gate not discharged).
/// This function always returns `Err(ServeError::Refused { why })` — never a stub success
/// (C1/G2 — never-silent). Use [`dispatch_request`] or [`dispatch_batch`] for in-memory
/// dispatch (fully implemented and testable).
///
/// A real implementation will be added in a future task once RP-10 + ADR-014 (the `wild`
/// OS-facility seam for network I/O) are ratified and the socket floor is discharged.
///
/// # Guarantee: `Exact`-when-`Ok`, Fallible `Err(ServeError::Refused { why })`, **io** declared,
/// EXPLAIN-able (the `why` message names the open gate).
pub fn serve(_table: RouteTable, listener: Listener) -> Result<(), ServeError> {
    Err(ServeError::Refused {
        why: format!(
            "real socket bind/accept-loop on {:?} is FLAGGED-gated (U2 socket-floor, \
             U8 net-effect — RP-10 pending); refusing explicitly — never a stub success \
             (C1/G2 — never-silent). Use dispatch_request / dispatch_batch for in-memory testing.",
            listener.address()
        ),
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{Body, Headers, Method, Status, Url};
    use crate::route::RouteTable;

    fn make_table() -> RouteTable {
        let mut table = RouteTable::new();
        table.add(Method::Get, "/hello", |_req| {
            Ok(Response::new(
                Status::OK,
                Headers::new(),
                Body::from_string("hello"),
            ))
        });
        table.add(Method::Post, "/echo", |req| {
            let body = req.body().clone();
            Ok(Response::new(Status::OK, Headers::new(), body))
        });
        table
    }

    fn get_request(path: &str) -> Request {
        let url = Url::parse_str(path).expect("valid path");
        Request::new(Method::Get, url, Headers::new(), Body::empty())
    }

    // ── dispatch_request ──────────────────────────────────────────────────────

    #[test]
    fn dispatch_matched_route_succeeds() {
        let table = make_table();
        let req = get_request("/hello");
        let resp = dispatch_request(&table, req).unwrap();
        assert_eq!(resp.status().as_u16(), 200);
        assert_eq!(resp.body().as_bytes(), b"hello");
    }

    #[test]
    fn dispatch_not_found_yields_route_error_never_wrong_handler() {
        let table = make_table();
        let req = get_request("/nonexistent");
        let err = dispatch_request(&table, req).unwrap_err();
        assert!(
            matches!(err, ServeError::Route(RouteError::NotFound)),
            "unmatched path must yield Route(NotFound) — never dispatches wrong handler (C1/G2)"
        );
    }

    #[test]
    fn dispatch_method_not_allowed_yields_explicit_error() {
        let table = make_table();
        // /hello is only registered for GET; try POST.
        let url = Url::parse_str("/hello").unwrap();
        let req = Request::new(Method::Post, url, Headers::new(), Body::empty());
        let err = dispatch_request(&table, req).unwrap_err();
        assert!(
            matches!(err, ServeError::Route(RouteError::MethodNotAllowed { .. })),
            "wrong method must yield Route(MethodNotAllowed) — never dispatches wrong handler (C1/G2)"
        );
    }

    // ── dispatch_batch (colony of hyphae, Empirical) ──────────────────────────

    #[test]
    fn dispatch_batch_empty_succeeds() {
        let table = make_table();
        let results = dispatch_batch(&table, vec![]).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn dispatch_batch_multiple_requests() {
        let table = make_table();
        let reqs = vec![get_request("/hello"), get_request("/hello")];
        let results = dispatch_batch(&table, reqs).unwrap();
        assert_eq!(results.len(), 2);
        for r in &results {
            assert!(r.is_ok());
        }
    }

    #[test]
    fn dispatch_batch_mixes_ok_and_err() {
        let table = make_table();
        let reqs = vec![get_request("/hello"), get_request("/nonexistent")];
        let results = dispatch_batch(&table, reqs).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
    }

    // ── serve (FLAGGED — always Err, never stub success) ──────────────────────

    /// `serve` with a real listener always returns `Err(Refused)` — the gate is not discharged.
    /// Guard: returning Ok from `serve` would mean we faked a socket bind — never silent (C1/G2).
    #[test]
    fn serve_real_listener_always_returns_err_refused_never_stub_success() {
        let table = make_table();
        let listener = Listener::new("127.0.0.1:0");
        let err = serve(table, listener).unwrap_err();
        assert!(
            matches!(err, ServeError::Refused { .. }),
            "serve must always return Refused (FLAGGED gate not discharged — C1/G2)"
        );
        if let ServeError::Refused { why } = err {
            assert!(
                why.contains("FLAGGED") || why.contains("socket"),
                "Refused::why must explain the gated status"
            );
        }
    }

    // ── Error Display (EXPLAIN — C3/G11) ──────────────────────────────────────

    #[test]
    fn serve_error_task_panicked_display() {
        let e = ServeError::TaskPanicked;
        assert!(
            e.to_string().contains("panic") || e.to_string().contains("never"),
            "Display must mention panic (G2 — never-silent)"
        );
    }

    #[test]
    fn serve_error_refused_display_includes_why() {
        let e = ServeError::Refused {
            why: "socket gate".to_owned(),
        };
        assert!(e.to_string().contains("socket gate"));
    }

    #[test]
    fn serve_error_is_std_error() {
        let e = ServeError::TaskPanicked;
        let _: &dyn std::error::Error = &e;
    }

    // ── Property tests (VR-5 / one per bound) ─────────────────────────────────

    mod property {
        use super::*;
        use proptest::prelude::*;

        fn make_prop_table() -> RouteTable {
            let mut t = RouteTable::new();
            t.add(Method::Get, "/alpha", |_| {
                Ok(Response::new(Status::OK, Headers::new(), Body::empty()))
            });
            t
        }

        proptest! {
            /// dispatch_request is deterministic: same (method, path) always maps to the same
            /// Ok/Err discriminant. (Empirical — no formal proof of the route function.)
            /// Guard: non-determinism in dispatch_request makes this fail.
            #[test]
            fn prop_dispatch_is_deterministic(
                path in r"/[a-z]{1,8}",
            ) {
                let table = make_prop_table();
                let req1 = get_request(&path);
                let req2 = get_request(&path);
                let r1 = dispatch_request(&table, req1).is_ok();
                let r2 = dispatch_request(&table, req2).is_ok();
                prop_assert_eq!(r1, r2, "dispatch_request must be deterministic");
            }

            /// serve always returns Err for any listener address (FLAGGED — never stub success).
            #[test]
            fn prop_serve_always_err(addr in r"127\.0\.0\.1:[0-9]{4}") {
                let table = make_prop_table();
                let listener = Listener::new(addr);
                let result = serve(table, listener);
                prop_assert!(result.is_err(),
                    "serve must always Err (FLAGGED gate not discharged — C1/G2)");
            }
        }
    }
}
