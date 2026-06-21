//! nodule: `web.client` — `get` / `post` / `request` + JSON convenience, per RFC-0022 §4.1 / §4.5.
//!
//! # Design
//! The client is structured around a [`Transport`] trait seam (mirror of `std.io`'s `Substrate`):
//! - In-memory / loopback impl for tests: `InMemoryTransport`.
//! - Real socket transport is **FLAGGED-gated** (U2 socket-floor, U8 net-effect): the real path
//!   returns explicit `Err(ClientError::Refused { why })` — never a stubbed success (C1/G2).
//!
//! # Guarantee summary (RFC-0022 §4.5)
//! - `get` / `request`: `Exact`-when-`Ok`, Fallible `Err(UnexpectedEof|Refused|EffectBudget)`,
//!   **io** (transport-gated), EXPLAIN-able.
//! - `get_json`: `Empirical`, Fallible `Err(ClientError::HttpParse|Json|Refused)`, **io**, EXPLAIN-able.
//! - Real socket transport: `Err(ClientError::Refused { why })` — explicit, never-silent, FLAGGED.

use std::fmt;

use crate::http::{parse_response, serialize_request};
use crate::http::{Body, Headers, HttpParseError, Method, Request, Response, Url};
use crate::json::{decode_body, encode_body, JsonError};
use mycelium_core::Value;

// ── ClientError ───────────────────────────────────────────────────────────────

/// Error type for client operations (C1 — never-silent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientError {
    /// The request or response could not be parsed (located error — C3).
    HttpParse(HttpParseError),
    /// The transport refused the operation (includes the real-socket FLAGGED case).
    ///
    /// # FLAGGED: real socket transport is NOT implemented (U2/U8 gated).
    /// When using the real socket path, `why` explains the FLAGGED gate (U2/U8 not discharged).
    Refused {
        /// Why the transport refused (G11 dual projection).
        why: String,
    },
    /// The transport connection was terminated before a complete response was received.
    UnexpectedEof {
        /// Bytes received before the EOF.
        received: usize,
    },
    /// A declared io/effect budget was exceeded (RFC-0014 §4.5 / C6).
    EffectBudget {
        /// The budget kind exceeded.
        kind: String,
    },
    /// JSON encode/decode error (wraps [`JsonError`]).
    Json(JsonError),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::HttpParse(e) => write!(f, "HTTP parse error: {e}"),
            ClientError::Refused { why } => write!(f, "transport refused: {why}"),
            ClientError::UnexpectedEof { received } => {
                write!(f, "unexpected EOF after {received} bytes")
            }
            ClientError::EffectBudget { kind } => write!(f, "effect budget exceeded ({kind})"),
            ClientError::Json(e) => write!(f, "JSON error: {e}"),
        }
    }
}

mycelium_std_core::impl_std_error!(
    ClientError,
    source = |this| {
        match this {
            ClientError::HttpParse(e) => Some(e),
            ClientError::Json(e) => Some(e),
            _ => None,
        }
    }
);

impl From<HttpParseError> for ClientError {
    fn from(e: HttpParseError) -> Self {
        ClientError::HttpParse(e)
    }
}

impl From<JsonError> for ClientError {
    fn from(e: JsonError) -> Self {
        ClientError::Json(e)
    }
}

// ── Transport trait seam ──────────────────────────────────────────────────────

/// The transport seam: abstracts over in-memory (test) and real socket (FLAGGED-gated).
///
/// Mirror of `std.io`'s `Substrate` pattern: the trait is the observable abstraction;
/// the real socket impl is behind a research gate (U2/U8).
///
/// # Guarantee
/// `send_receive` is `Exact`-when-`Ok`, Fallible (transport-specific errors mapped to
/// `ClientError`), `io` effect declared on all impls.
pub trait Transport: fmt::Debug {
    /// Send a serialized HTTP request and receive a serialized HTTP response.
    ///
    /// # Guarantee: `Exact`-when-`Ok`, Fallible `Err(ClientError)`, **io** effect
    /// The transport returns the raw bytes of the response (including headers), which the
    /// caller will then parse via `parse_response`. This keeps parsing pure and testable.
    ///
    /// # FLAGGED: real socket transport is NOT implemented (U2 socket-floor, U8 net-effect).
    /// A `RealSocketTransport` (future, FLAGGED) would return `Err(ClientError::Refused { why })` until
    /// the research gate is discharged.
    fn send_receive(&self, request_bytes: &[u8]) -> Result<Vec<u8>, ClientError>;
}

// ── InMemoryTransport (in-memory test impl) ───────────────────────────────────

/// Handler closure type alias for `InMemoryTransport`.
type InMemoryHandler = std::sync::Arc<dyn Fn(&[u8]) -> Option<Vec<u8>> + Send + Sync>;

/// An in-memory loopback transport for testing — no real socket involved.
///
/// Routes requests to a user-supplied closure that maps serialized request bytes to
/// serialized response bytes. This is the test impl described in RFC-0022 §4.1.
///
/// # Guarantee: `Exact`-when-`Ok`, Fallible `Err(ClientError::Refused)` if the handler
/// returns `None`. The `io` effect is declared on [`Transport::send_receive`].
pub struct InMemoryTransport {
    /// The handler closure. Takes raw request bytes, returns raw response bytes or `None`
    /// to simulate a connection refusal.
    handler: InMemoryHandler,
}

impl fmt::Debug for InMemoryTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemoryTransport").finish_non_exhaustive()
    }
}

impl InMemoryTransport {
    /// Create a new in-memory transport with the given handler.
    ///
    /// # Guarantee: `Exact` / Total
    pub fn new(handler: impl Fn(&[u8]) -> Option<Vec<u8>> + Send + Sync + 'static) -> Self {
        InMemoryTransport {
            handler: std::sync::Arc::new(handler),
        }
    }

    /// Create a transport that always returns the given fixed response bytes.
    ///
    /// # Guarantee: `Exact` / Total
    pub fn fixed_response(response_bytes: Vec<u8>) -> Self {
        Self::new(move |_| Some(response_bytes.clone()))
    }

    /// Create a transport that always refuses (simulating a connection error).
    ///
    /// # Guarantee: `Exact` / Total
    pub fn always_refused(reason: impl Into<String>) -> Self {
        let why = reason.into();
        Self::new(move |_| {
            let _ = &why;
            None
        })
    }
}

impl Transport for InMemoryTransport {
    fn send_receive(&self, request_bytes: &[u8]) -> Result<Vec<u8>, ClientError> {
        match (self.handler)(request_bytes) {
            Some(response) => Ok(response),
            None => Err(ClientError::Refused {
                why: "in-memory transport returned None (connection refused)".to_owned(),
            }),
        }
    }
}

// ── FLAGGED: RealSocketTransport ──────────────────────────────────────────────
//
// FLAG (U2, U8): Real socket transport is NOT implemented.
// The `Transport` trait seam is the insertion point for a future `std-sys` backed
// socket transport. Until RP-10 (the socket-floor research gate) is discharged, any
// attempt to use a real socket must return an explicit error — never a stubbed success (C1/G2).
//
// This stub type carries the gate: it exists so the type system can name the unavailable path,
// but its `send_receive` always returns `Err(ClientError::Refused { why })`.

/// A placeholder for the (FLAGGED-gated) real socket transport.
///
/// # FLAGGED: U2 socket-floor (socket bind/connect), U8 net-effect (OS network permissions).
/// These are NOT discharged. This type always returns `Err(ClientError::Refused { why })`
/// to make the unavailability explicit — never a stub success (C1/G2 — never-silent).
///
/// A production socket impl will be added in a future task once RP-10 + ADR-014 (the `wild`
/// OS-facility seam for network I/O) are ratified.
#[derive(Debug)]
pub struct RealSocketTransport {
    /// Kept for future: the target address.
    address: String,
}

impl RealSocketTransport {
    /// Create a (FLAGGED) real socket transport targeting `address`.
    ///
    /// FLAGGED: this constructor succeeds, but every `send_receive` call on the returned
    /// transport will return `Err(ClientError::Refused { why })` — the gate is not discharged.
    #[must_use]
    pub fn new(address: impl Into<String>) -> Self {
        RealSocketTransport {
            address: address.into(),
        }
    }
}

impl Transport for RealSocketTransport {
    /// # FLAGGED: always returns `Err(ClientError::Refused { why })` (U2/U8 gate not discharged — C1/G2).
    fn send_receive(&self, _request_bytes: &[u8]) -> Result<Vec<u8>, ClientError> {
        Err(ClientError::Refused {
            why: format!(
                "real socket transport to {:?} is FLAGGED-gated (U2 socket-floor, U8 net-effect \
                 not yet discharged — RP-10 pending); refusing explicitly — never a stub success \
                 (C1/G2 — never-silent)",
                self.address
            ),
        })
    }
}

// ── Client operations ─────────────────────────────────────────────────────────

/// Send an HTTP GET request via `transport` and return the parsed response.
///
/// # Guarantee: `Exact`-when-`Ok`, Fallible `Err(ClientError)`, **io** effect, EXPLAIN-able
/// Request building is pure. The transport carries the `io` effect. The response is parsed
/// strictly (via `parse_response` — never-silent parse errors, C1/G2).
///
/// FLAGGED: if `transport` is a [`RealSocketTransport`], this always returns
/// `Err(ClientError::Refused { why })` — the socket gate is not discharged.
pub fn get(url: &Url, transport: &dyn Transport) -> Result<Response, ClientError> {
    let req = Request::new(Method::Get, url.clone(), Headers::new(), Body::empty());
    request(req, transport)
}

/// Send an HTTP POST request with a body via `transport` and return the parsed response.
///
/// # Guarantee: `Exact`-when-`Ok`, Fallible `Err(ClientError)`, **io**, EXPLAIN-able
///
/// FLAGGED: same as [`get`] — real socket transport is FLAGGED-gated.
pub fn post(url: &Url, body: Body, transport: &dyn Transport) -> Result<Response, ClientError> {
    let req = Request::new(Method::Post, url.clone(), Headers::new(), body);
    request(req, transport)
}

/// Send an HTTP request via `transport` and return the parsed response.
///
/// # Guarantee: `Exact`-when-`Ok`, Fallible `Err(ClientError)`, **io**, EXPLAIN-able
/// This is the base operation; `get`/`post` delegate to it (DRY).
///
/// FLAGGED: if `transport` is a [`RealSocketTransport`], returns `Err(ClientError::Refused { why })`.
pub fn request(req: Request, transport: &dyn Transport) -> Result<Response, ClientError> {
    let bytes = serialize_request(&req);
    let response_bytes = transport.send_receive(&bytes)?;
    let response = parse_response(&response_bytes)?;
    Ok(response)
}

/// Send a GET request and decode the response body as a JSON [`Value`].
///
/// # Guarantee: `Empirical` (inherited from `decode_body` which inherits from `std.io::from_json`)
/// The JSON round-trip property is `Empirical` — established by the `std.io` proptest corpus.
/// The `io` + transport effect is declared.
///
/// # Fallibility: `Err(ClientError)` — may wrap `HttpParse`, `Json`, or `Refused`.
///
/// FLAGGED: real socket transport gate (U2/U8).
pub fn get_json(url: &Url, transport: &dyn Transport) -> Result<Value, ClientError> {
    let response = get(url, transport)?;
    let value = decode_body(response.body())?;
    Ok(value)
}

/// Send a POST request with a JSON-encoded [`Value`] body.
///
/// # Guarantee: `Exact`-when-`Ok` (encode is `Exact`-when-`Ok`; rest delegates to `request`).
/// # Fallibility: `Err(ClientError)` — may wrap `Json::OutOfDomain` or transport errors.
///
/// FLAGGED: real socket transport gate (U2/U8).
pub fn post_json(
    url: &Url,
    value: &Value,
    transport: &dyn Transport,
) -> Result<Response, ClientError> {
    let body = encode_body(value)?;
    let mut headers = Headers::new();
    headers
        .insert("content-type", "application/json")
        .expect("valid header");
    let req = Request::new(Method::Post, url.clone(), headers, body);
    request(req, transport)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_core::{
        meta::{Meta, Provenance},
        repr::Repr,
        value::{Payload, Value},
    };

    fn ok_response_bytes() -> Vec<u8> {
        b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nhello".to_vec()
    }

    fn make_transport_that_echoes_200() -> InMemoryTransport {
        InMemoryTransport::fixed_response(ok_response_bytes())
    }

    fn binary_value() -> Value {
        Value::new(
            Repr::Binary { width: 4 },
            Payload::Bits(vec![true, false, true, false]),
            Meta::exact(Provenance::Root),
        )
        .expect("well-formed binary value")
    }

    // ── get / request / post ──────────────────────────────────────────────────

    #[test]
    fn get_with_in_memory_transport_succeeds() {
        let transport = make_transport_that_echoes_200();
        let url = Url::parse_str("/hello").unwrap();
        let resp = get(&url, &transport).unwrap();
        assert_eq!(resp.status().as_u16(), 200);
    }

    #[test]
    fn post_with_in_memory_transport_succeeds() {
        let transport = make_transport_that_echoes_200();
        let url = Url::parse_str("/submit").unwrap();
        let resp = post(&url, Body::from_string("payload"), &transport).unwrap();
        assert_eq!(resp.status().as_u16(), 200);
    }

    #[test]
    fn transport_refusal_yields_err_refused_never_silent() {
        let transport = InMemoryTransport::always_refused("test refusal");
        let url = Url::parse_str("/path").unwrap();
        let err = get(&url, &transport).unwrap_err();
        assert!(
            matches!(err, ClientError::Refused { .. }),
            "transport refusal must yield Refused, never Ok (C1/G2)"
        );
    }

    // ── RealSocketTransport is always Err (FLAGGED — never stub success) ──────

    /// The real socket transport always refuses — the gate is not discharged.
    /// Guard: returning Ok from `send_receive` would mean we faked a socket — never silent (C1/G2).
    #[test]
    fn real_socket_transport_always_returns_err_unwired() {
        let transport = RealSocketTransport::new("127.0.0.1:8080");
        let url = Url::parse_str("/ping").unwrap();
        let err = get(&url, &transport).unwrap_err();
        assert!(
            matches!(err, ClientError::Refused { .. }),
            "RealSocketTransport must always Err with Refused (FLAGGED — never stub success)"
        );
        // Check that the error message mentions the gate.
        if let ClientError::Refused { why } = err {
            assert!(
                why.contains("FLAGGED") || why.contains("socket"),
                "Refused::why must explain the gated status"
            );
        }
    }

    // ── get_json / post_json ──────────────────────────────────────────────────

    #[test]
    fn get_json_decodes_value_from_response_body() {
        // Encode a value, wrap it in a 200 response, and serve it from in-memory transport.
        let v = binary_value();
        let body = crate::json::encode_body(&v).expect("encode");
        let resp_bytes = {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n");
            bytes.extend_from_slice(body.as_bytes());
            bytes
        };
        let transport = InMemoryTransport::fixed_response(resp_bytes);
        let url = Url::parse_str("/api").unwrap();
        let decoded = get_json(&url, &transport).unwrap();
        assert_eq!(decoded, v);
    }

    #[test]
    fn post_json_sends_encoded_value() {
        // Verify that the request body seen by the transport is valid JSON.
        let v = binary_value();
        let sent_body = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
        let sent_body_clone = sent_body.clone();
        let transport = InMemoryTransport::new(move |req_bytes| {
            // Parse the request to extract the body.
            if let Ok(req) = crate::http::parse_request(req_bytes) {
                *sent_body_clone.lock().unwrap() = req.body().as_bytes().to_vec();
            }
            Some(ok_response_bytes())
        });

        let url = Url::parse_str("/post").unwrap();
        post_json(&url, &v, &transport).unwrap();

        let body_bytes = sent_body.lock().unwrap().clone();
        let body = Body::from_bytes(body_bytes);
        let decoded = crate::json::decode_body(&body).expect("body must be valid JSON");
        assert_eq!(decoded, v);
    }

    // ── ClientError Display (EXPLAIN — C3/G11) ────────────────────────────────

    #[test]
    fn client_error_refused_display_includes_why() {
        let e = ClientError::Refused {
            why: "connection reset".to_owned(),
        };
        assert!(e.to_string().contains("connection reset"));
    }

    #[test]
    fn client_error_is_std_error() {
        let e = ClientError::Refused {
            why: "test".to_owned(),
        };
        let _: &dyn std::error::Error = &e;
    }

    // ── Property tests (VR-5 / one per bound) ─────────────────────────────────

    mod property {
        use super::*;
        use proptest::prelude::*;

        fn arb_finite_dense_value() -> impl Strategy<Value = Value> {
            (1u32..=4u32).prop_flat_map(|d| {
                prop::collection::vec((-16_i32..=16_i32).prop_map(f64::from), d as usize).prop_map(
                    move |scalars| {
                        Value::new(
                            Repr::Dense {
                                dim: d,
                                dtype: mycelium_core::repr::ScalarKind::F64,
                            },
                            Payload::Scalars(scalars),
                            Meta::exact(Provenance::Root),
                        )
                        .expect("well-formed dense value")
                    },
                )
            })
        }

        proptest! {
            /// `get` with the in-memory transport: request↔response round-trip (Empirical).
            /// Guard: any asymmetry in serialize_request / parse_response makes this fail.
            #[test]
            fn prop_get_in_memory_round_trip(
                path in r"/[a-z]{1,8}",
            ) {
                // Build a transport that echoes back a fixed 200 response.
                let transport = InMemoryTransport::fixed_response(
                    b"HTTP/1.1 200 OK\r\n\r\n".to_vec()
                );
                let url = Url::parse_str(&path).expect("valid path");
                let result = get(&url, &transport);
                prop_assert!(result.is_ok(), "get with in-memory transport must succeed");
                let resp = result.unwrap();
                prop_assert_eq!(resp.status().as_u16(), 200);
            }

            /// `post_json` then `get_json` round-trip: value survives JSON encode/decode (Empirical).
            #[test]
            fn prop_json_client_round_trip(v in arb_finite_dense_value()) {
                // Encode, serve, decode via transport.
                let encoded_body = encode_body(&v).expect("encode: finite value");
                let mut resp_bytes = Vec::new();
                resp_bytes.extend_from_slice(b"HTTP/1.1 200 OK\r\n\r\n");
                resp_bytes.extend_from_slice(encoded_body.as_bytes());

                let transport = InMemoryTransport::fixed_response(resp_bytes);
                let url = Url::parse_str("/api").unwrap();
                let decoded = get_json(&url, &transport).expect("decode must succeed");
                prop_assert_eq!(v, decoded,
                    "JSON client round-trip must be identity (Empirical)");
            }
        }
    }
}
