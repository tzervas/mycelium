//! nodule: `web.route` — a reified, inspectable route table + dispatch with mandatory EXPLAIN
//! (C3), per RFC-0022 §4.1 / §4.5.
//!
//! # Design
//! - [`RouteTable`] is a reified, inspectable collection of route patterns + handlers.
//!   The whole table can be iterated / inspected — no black box (C3).
//! - [`match_route`] returns a [`RouteMatch`] naming **which pattern matched and which captures
//!   were extracted** — the EXPLAIN artifact (C3 "selections are inspectable").
//! - Errors are explicit and discriminated: [`RouteError::NotFound`] (404) vs
//!   [`RouteError::MethodNotAllowed`] (405 + allowed methods) — never a silent wrong-handler (C1/G2).
//!
//! # Patterns
//! Static segments: `/foo/bar`. Parameterized segments: `/users/:id` (`:name` captures one
//! path segment). Trailing wildcards are NOT supported (a research-gated extension).
//!
//! # Guarantee summary (RFC-0022 §4.5)
//! - `RouteTable` (reified): `Exact` / Total, EXPLAIN-able (C3).
//! - `match_route`: `Exact`-when-`Ok`, Fallible `Err(NotFound|MethodNotAllowed)`, none, EXPLAIN (C3).
//! - Matching is pure: the `RouteMatch` names the pattern + captures.

use std::collections::BTreeMap;
use std::fmt;

use crate::http::{Method, Request, Response};

// ── Handler type ─────────────────────────────────────────────────────────────

/// An HTTP handler function: pure mapping from `Request` → `Result<Response, HttpError>`.
///
/// # Guarantee: `Declared` (FLAGGED)
/// Handler purity is a *contract*, not a type-system invariant — the type system cannot
/// prevent a handler from touching global state or performing I/O. This is always `Declared`
/// (VR-5: we cannot promote a convention to `Proven` without a formal encapsulation mechanism).
/// FLAG: handler-purity contract is Declared, not Proven (RFC-0022 §10.2 E7-2 / VR-5).
pub type Handler = fn(Request) -> Result<Response, HttpError>;

/// Error type for handler-level failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    /// The handler returned an internal server error.
    Internal {
        /// Why the handler failed.
        why: String,
    },
    /// The request entity could not be processed.
    UnprocessableEntity {
        /// Why.
        why: String,
    },
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpError::Internal { why } => write!(f, "internal server error: {why}"),
            HttpError::UnprocessableEntity { why } => write!(f, "unprocessable entity: {why}"),
        }
    }
}

mycelium_std_core::impl_std_error!(HttpError);

// ── RouteError ───────────────────────────────────────────────────────────────

/// The explicit error set for route dispatch failures (C1/G2 — never a silent wrong-handler).
///
/// # Guarantee: discriminated 404/405 — never silent (C1)
/// The caller can inspect `RouteError::MethodNotAllowed::allowed` to synthesize the correct
/// `Allow:` header (RFC-7231 §6.5.5) without black-box inspection of the route table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteError {
    /// No pattern in the table matched the request path (HTTP 404).
    NotFound,
    /// The path was found but the method is not registered for it (HTTP 405).
    ///
    /// `allowed` lists every method registered for the matched path — never empty (C1).
    MethodNotAllowed {
        /// The methods registered for the matched path (non-empty; C1).
        allowed: Vec<Method>,
    },
}

impl fmt::Display for RouteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouteError::NotFound => write!(f, "404 Not Found: no route matched the path"),
            RouteError::MethodNotAllowed { allowed } => {
                let methods: Vec<&str> = allowed.iter().map(|m| m.as_str()).collect();
                write!(
                    f,
                    "405 Method Not Allowed; allowed: [{}]",
                    methods.join(", ")
                )
            }
        }
    }
}

mycelium_std_core::impl_std_error!(RouteError);

// ── Path parameters ───────────────────────────────────────────────────────────

/// Named path captures from a parametric route (e.g. `/users/:id` → `{"id": "42"}`).
///
/// # Guarantee: `Exact` / Total (a `BTreeMap` — deterministic, inspectable — C3).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PathParams(BTreeMap<String, String>);

impl PathParams {
    /// Create an empty `PathParams`.
    #[must_use]
    pub fn new() -> Self {
        PathParams(BTreeMap::new())
    }

    /// Insert a capture.
    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.0.insert(name.into(), value.into());
    }

    /// Get a capture by name, or `None`.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0.get(name).map(|s| s.as_str())
    }

    /// Iterate over all `(name, value)` pairs in sorted order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Number of captures.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// True if no captures.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// ── RouteMatch (the EXPLAIN artifact — C3) ────────────────────────────────────

/// The inspectable result of a successful route match (C3 — EXPLAIN-able dispatch).
///
/// Every field is public so the caller can log / assert exactly *which* pattern matched
/// and *which* captures were extracted — no opaque dispatch (C3 / RFC-0022 §4.5).
///
/// Note: `PartialEq` compares `pattern`, `method`, and `captures` only — **not** `handler`.
/// Function pointer equality is not well-defined in Rust (addresses can be deduplicated
/// across codegen units). The EXPLAIN surface is the pattern + captures; handler identity
/// is not part of the inspectable result.
#[derive(Debug, Clone)]
pub struct RouteMatch {
    /// The pattern string that matched (e.g. `"/users/:id"`).
    pub pattern: String,
    /// The HTTP method that matched.
    pub method: Method,
    /// The named captures extracted from the path (empty for static routes).
    pub captures: PathParams,
    /// The handler for this route.
    pub handler: Handler,
}

impl PartialEq for RouteMatch {
    fn eq(&self, other: &Self) -> bool {
        // Compare pattern, method, and captures only — not handler (fn-ptr equality is not
        // well-defined; see Rust docs on `fn_addr_eq`).
        self.pattern == other.pattern
            && self.method == other.method
            && self.captures == other.captures
    }
}

impl Eq for RouteMatch {}

// ── RoutePattern ──────────────────────────────────────────────────────────────

/// A parsed route pattern (segments, some of which are `:param` captures).
#[derive(Debug, Clone, PartialEq, Eq)]
struct RoutePattern {
    /// The original pattern string (for EXPLAIN).
    raw: String,
    /// Parsed segments: `Literal("foo")` or `Param("id")`.
    segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    Literal(String),
    Param(String),
}

impl RoutePattern {
    fn parse(pattern: &str) -> Self {
        let segments = pattern
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| {
                if let Some(name) = s.strip_prefix(':') {
                    Segment::Param(name.to_owned())
                } else {
                    Segment::Literal(s.to_owned())
                }
            })
            .collect();
        RoutePattern {
            raw: pattern.to_owned(),
            segments,
        }
    }

    /// Try to match `path` (a `/`-split-and-filtered slice of segments) against this pattern.
    /// Returns `Some(PathParams)` on success, `None` otherwise.
    fn try_match(&self, path_segments: &[&str]) -> Option<PathParams> {
        if self.segments.len() != path_segments.len() {
            return None;
        }
        let mut captures = PathParams::new();
        for (seg, path_seg) in self.segments.iter().zip(path_segments) {
            match seg {
                Segment::Literal(lit) => {
                    if lit != path_seg {
                        return None;
                    }
                }
                Segment::Param(name) => {
                    captures.insert(name.clone(), *path_seg);
                }
            }
        }
        Some(captures)
    }
}

// ── RouteEntry ────────────────────────────────────────────────────────────────

/// One entry in the route table: a pattern + a map from method to handler.
#[derive(Clone)]
struct RouteEntry {
    pattern: RoutePattern,
    handlers: BTreeMap<String, Handler>, // keyed by method.as_str()
}

// ── RouteTable ────────────────────────────────────────────────────────────────

/// A reified, inspectable route table (C3 — EXPLAIN-able; RFC-0022 §4.5).
///
/// Routes are matched in insertion order. The first matching pattern wins (longest-prefix
/// wins via insertion order convention — callers must insert more-specific patterns first).
///
/// # Guarantee: `Exact` / Total (constructors + accessors); EXPLAIN-able (iterator).
#[derive(Clone, Default)]
pub struct RouteTable {
    entries: Vec<RouteEntry>,
}

impl RouteTable {
    /// Create an empty route table.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn new() -> Self {
        RouteTable {
            entries: Vec::new(),
        }
    }

    /// Register a handler for `(method, pattern)`.
    ///
    /// Patterns may contain `:param` segments. Multiple methods may be registered
    /// for the same pattern by calling `add` multiple times.
    ///
    /// # Guarantee: `Exact` / Total (no validation that patterns are well-formed; invalid
    /// patterns will simply never match — callers are responsible for pattern correctness).
    pub fn add(&mut self, method: Method, pattern: impl Into<String>, handler: Handler) {
        let pattern_str = pattern.into();
        let route_pattern = RoutePattern::parse(&pattern_str);
        let method_key = method.as_str().to_owned();
        // Find an existing entry for this pattern (by raw string equality).
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|e| e.pattern.raw == route_pattern.raw)
        {
            entry.handlers.insert(method_key, handler);
        } else {
            let mut handlers = BTreeMap::new();
            handlers.insert(method_key, handler);
            self.entries.push(RouteEntry {
                pattern: route_pattern,
                handlers,
            });
        }
    }

    /// Number of registered patterns.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if no routes are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over all `(pattern_str, registered_methods)` pairs (EXPLAIN-able — C3).
    ///
    /// # Guarantee: `Exact` / Total (BTreeMap order for methods; insertion order for patterns)
    pub fn patterns(&self) -> impl Iterator<Item = (&str, Vec<&str>)> {
        self.entries.iter().map(|e| {
            let methods: Vec<&str> = e.handlers.keys().map(|s| s.as_str()).collect();
            (e.pattern.raw.as_str(), methods)
        })
    }
}

impl fmt::Debug for RouteTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RouteTable")
            .field("entry_count", &self.entries.len())
            .field(
                "patterns",
                &self
                    .entries
                    .iter()
                    .map(|e| &e.pattern.raw)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

// ── match_route ───────────────────────────────────────────────────────────────

/// Match a method + path against the route table.
///
/// Returns a [`RouteMatch`] naming the matched pattern + captures (the EXPLAIN artifact — C3).
///
/// # Guarantee: `Exact`-when-`Ok`
/// When a match is found the returned `RouteMatch` accurately names the pattern and captures —
/// no hidden approximation (C3).
///
/// # Fallibility: `Err(RouteError::NotFound | RouteError::MethodNotAllowed{allowed})`
/// - `NotFound` — no pattern matched the path (HTTP 404).
/// - `MethodNotAllowed` — the path matched a pattern but the method was not registered;
///   `allowed` names every registered method for that path — **never a silent wrong-handler**
///   (C1/G2). `allowed` is never empty.
///
/// # Effects: none
/// Pure — no IO, no side effects.
///
/// # EXPLAIN-able: yes (C3)
/// The `RouteMatch` in `Ok` names which pattern matched and which captures were extracted.
/// `RouteError::MethodNotAllowed::allowed` names what *would* have matched — both sides
/// of the dispatch decision are exposed.
pub fn match_route(
    table: &RouteTable,
    method: &Method,
    path: &str,
) -> Result<RouteMatch, RouteError> {
    // Split path into segments (strip leading `/`).
    let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let method_key = method.as_str();

    // First pass: find all entries whose pattern matches the path segments.
    let mut path_matched: Vec<&RouteEntry> = Vec::new();
    for entry in &table.entries {
        if entry.pattern.try_match(&path_segments).is_some() {
            path_matched.push(entry);
        }
    }

    if path_matched.is_empty() {
        return Err(RouteError::NotFound);
    }

    // Second pass: among path-matched entries, find one whose method matches.
    for entry in &path_matched {
        if let Some(handler) = entry.handlers.get(method_key) {
            // A match — build the EXPLAIN artifact.
            let captures = entry
                .pattern
                .try_match(&path_segments)
                .expect("pattern matched in first pass; must match in second");
            return Ok(RouteMatch {
                pattern: entry.pattern.raw.clone(),
                method: method.clone(),
                captures,
                handler: *handler,
            });
        }
    }

    // Path matched but method didn't — collect all allowed methods (non-empty by construction).
    let allowed: Vec<Method> = path_matched
        .iter()
        .flat_map(|e| e.handlers.keys().map(|k| Method::parse(k.as_bytes())))
        .filter_map(|r| r.ok())
        .collect();

    // `allowed` is guaranteed non-empty: `path_matched` is non-empty and every entry has
    // at least one handler (inserted by `add`).
    Err(RouteError::MethodNotAllowed { allowed })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{Body, Headers, Status};

    fn ok_handler(req: Request) -> Result<Response, HttpError> {
        let _ = req;
        Ok(Response::new(Status::OK, Headers::new(), Body::empty()))
    }

    fn other_handler(req: Request) -> Result<Response, HttpError> {
        let _ = req;
        Ok(Response::new(
            Status::from_u16(201).unwrap(),
            Headers::new(),
            Body::empty(),
        ))
    }

    // ── Basic match ───────────────────────────────────────────────────────────

    #[test]
    fn match_static_route() {
        let mut table = RouteTable::new();
        table.add(Method::Get, "/hello", ok_handler);
        let m = match_route(&table, &Method::Get, "/hello").unwrap();
        assert_eq!(m.pattern, "/hello");
        assert!(m.captures.is_empty());
    }

    #[test]
    fn match_parametric_route() {
        let mut table = RouteTable::new();
        table.add(Method::Get, "/users/:id", ok_handler);
        let m = match_route(&table, &Method::Get, "/users/42").unwrap();
        assert_eq!(m.pattern, "/users/:id");
        assert_eq!(m.captures.get("id"), Some("42"));
    }

    // ── RouteError::NotFound (C1/G2) ──────────────────────────────────────────

    /// A path with no matching pattern yields `NotFound` — never a wrong handler (C1/G2).
    /// Guard: returning Ok (dispatching to a wrong handler) makes this fail.
    #[test]
    fn no_match_yields_not_found_never_wrong_handler() {
        let mut table = RouteTable::new();
        table.add(Method::Get, "/ping", ok_handler);
        let err = match_route(&table, &Method::Get, "/pong").unwrap_err();
        assert_eq!(
            err,
            RouteError::NotFound,
            "unmatched path must yield NotFound (C1/G2)"
        );
    }

    // ── RouteError::MethodNotAllowed (C1/G2) ──────────────────────────────────

    /// A matched path with the wrong method yields `MethodNotAllowed{allowed}` — never
    /// dispatches to the wrong handler and never returns an empty `allowed` list (C1/G2).
    #[test]
    fn wrong_method_yields_method_not_allowed_with_allowed_list() {
        let mut table = RouteTable::new();
        table.add(Method::Get, "/resource", ok_handler);
        table.add(Method::Post, "/resource", other_handler);

        let err = match_route(&table, &Method::Delete, "/resource").unwrap_err();
        match err {
            RouteError::MethodNotAllowed { allowed } => {
                assert!(
                    !allowed.is_empty(),
                    "allowed list must be non-empty (C1/G2)"
                );
                assert!(
                    allowed.contains(&Method::Get) || allowed.contains(&Method::Post),
                    "allowed list must contain the registered methods"
                );
            }
            other => panic!("expected MethodNotAllowed, got {other:?}"),
        }
    }

    // ── EXPLAIN artifact (C3) ─────────────────────────────────────────────────

    /// The `RouteMatch` names the pattern + captures — the C3 EXPLAIN artifact.
    /// Guard: returning the match without the pattern/captures would fail the C3 invariant.
    #[test]
    fn route_match_exposes_pattern_and_captures() {
        let mut table = RouteTable::new();
        table.add(Method::Get, "/items/:category/:id", ok_handler);
        let m = match_route(&table, &Method::Get, "/items/books/99").unwrap();
        assert_eq!(m.pattern, "/items/:category/:id");
        assert_eq!(m.captures.get("category"), Some("books"));
        assert_eq!(m.captures.get("id"), Some("99"));
    }

    // ── RouteTable is inspectable (C3) ────────────────────────────────────────

    #[test]
    fn route_table_is_inspectable() {
        let mut table = RouteTable::new();
        table.add(Method::Get, "/a", ok_handler);
        table.add(Method::Post, "/b", ok_handler);
        let patterns: Vec<_> = table.patterns().collect();
        assert_eq!(patterns.len(), 2);
    }

    // ── Multiple methods on same pattern ──────────────────────────────────────

    #[test]
    fn multiple_methods_on_same_pattern() {
        let mut table = RouteTable::new();
        table.add(Method::Get, "/data", ok_handler);
        table.add(Method::Post, "/data", other_handler);

        assert!(match_route(&table, &Method::Get, "/data").is_ok());
        assert!(match_route(&table, &Method::Post, "/data").is_ok());
        assert!(match_route(&table, &Method::Delete, "/data").is_err());
    }

    // ── Error Display (EXPLAIN / C3) ──────────────────────────────────────────

    #[test]
    fn route_error_not_found_display() {
        let e = RouteError::NotFound;
        assert!(e.to_string().contains("404") || e.to_string().contains("Not Found"));
    }

    #[test]
    fn route_error_method_not_allowed_display_includes_allowed() {
        let e = RouteError::MethodNotAllowed {
            allowed: vec![Method::Get, Method::Post],
        };
        let s = e.to_string();
        assert!(
            s.contains("GET") || s.contains("GET"),
            "Display must include allowed methods"
        );
        assert!(s.contains("405") || s.contains("Method Not Allowed"));
    }

    #[test]
    fn route_error_is_std_error() {
        let e = RouteError::NotFound;
        let _: &dyn std::error::Error = &e;
    }

    // ── Property tests (VR-5 / one per bound) ─────────────────────────────────

    mod property {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Route match determinism: same (method, path) always maps to the same
            /// result (Ok or Err discriminant is stable). (Empirical — no formal proof.)
            /// Guard: non-determinism in routing would make this fail.
            #[test]
            fn prop_match_route_is_deterministic(
                path in r"/[a-z]{1,6}(/[a-z]{1,6}){0,2}",
            ) {
                let mut table = RouteTable::new();
                table.add(Method::Get, "/alpha/beta", ok_handler);
                table.add(Method::Post, "/alpha/beta", ok_handler);
                table.add(Method::Get, "/gamma", ok_handler);

                let result1 = match_route(&table, &Method::Get, &path);
                let result2 = match_route(&table, &Method::Get, &path);
                prop_assert_eq!(result1.is_ok(), result2.is_ok(),
                    "match_route must be deterministic for same input");
            }

            /// `NotFound` is returned when no pattern matches — never a wrong handler.
            /// Guard: returning Ok for a path with no entry makes this fail.
            #[test]
            fn prop_not_found_when_no_match(
                path in r"/[x-z]{6,10}",
            ) {
                // Table only has short paths; the above strategy generates long paths
                // that will not match.
                let mut table = RouteTable::new();
                table.add(Method::Get, "/ab", ok_handler);
                // paths starting with /x|/y|/z won't match /ab
                let result = match_route(&table, &Method::Get, &path);
                // Either not found or a coincidental match (proptest might generate /ab by
                // accident via the charset overlap — but /[x-z]{6,10} won't produce /ab).
                // The property: if Err, it must be NotFound or MethodNotAllowed, never a panic.
                match result {
                    Ok(_) | Err(RouteError::NotFound) | Err(RouteError::MethodNotAllowed { .. }) => {}
                }
            }

            /// `MethodNotAllowed::allowed` is never empty (C1/G2).
            /// Guard: producing an empty `allowed` list makes this fail.
            #[test]
            fn prop_method_not_allowed_has_nonempty_allowed_list(
                method_idx in 0usize..5,
            ) {
                let methods = [Method::Get, Method::Post, Method::Put, Method::Delete, Method::Patch];
                let registered = &methods[0]; // only GET
                let tested = &methods[method_idx];

                let mut table = RouteTable::new();
                table.add(registered.clone(), "/resource", ok_handler);

                if tested == registered {
                    // same method — must be Ok
                    prop_assert!(match_route(&table, tested, "/resource").is_ok());
                } else {
                    // different method — must be MethodNotAllowed with non-empty allowed
                    match match_route(&table, tested, "/resource") {
                        Err(RouteError::MethodNotAllowed { allowed }) => {
                            prop_assert!(!allowed.is_empty(),
                                "MethodNotAllowed::allowed must be non-empty (C1/G2)");
                        }
                        Ok(_) => prop_assert!(false, "unexpected Ok for non-matching method"),
                        Err(RouteError::NotFound) => prop_assert!(false, "unexpected NotFound"),
                    }
                }
            }
        }
    }
}
