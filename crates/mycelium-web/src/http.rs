//! nodule: `web.http` — the HTTP/1.1 value model + parsing, per RFC-0022 §4.1 / §4.5.
//!
//! # Guarantee summary
//! - Accessors (`method`, `path`, `header_get`): `Exact` / Total — no accuracy semantics.
//! - `Request::new`: `Exact` / Total.
//! - `with_header`: `Exact`-when-`Ok`, Fallible `Err(InvalidHeaderName|InvalidHeaderValue)`.
//! - `Status::from_u16`: `Exact`-when-`Ok`, Fallible `Err(OutOfRange{code})` — **never a clamp**.
//! - `parse_request` / `parse_response` / `Method::parse` / `Url::parse`: `Exact`-when-`Ok`,
//!   Fallible with **located** errors (byte offset `at` / field `why`).
//! - `serialize_request`: `Exact`, Fallible (header name/value validation).
//!
//! # Never-silent (C1/G2)
//! Every fallible operation returns an explicit `Result` with a [`HttpParseError`] carrying a
//! located locus (byte offset) — never a clamp, never a sentinel, never a partial result.
//!
//! # EXPLAIN-able (C3)
//! [`HttpParseError`] variants name the byte offset + why, so every parse failure is legible.

use std::collections::BTreeMap;
use std::fmt;

// ── Locus + error types ──────────────────────────────────────────────────────

/// A byte offset into the HTTP input (locus of a parse failure, C1/RFC-0013 I1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteOffset(pub u64);

impl fmt::Display for ByteOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "byte {}", self.0)
    }
}

/// The explicit error set for HTTP parsing / construction failures (C1/G2 — never-silent).
///
/// Every variant carries a locus (byte offset or description) so the caller can surface
/// *where* the parse failed — never a locationless "parse error" (G2/C3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpParseError {
    /// The input ended before a complete HTTP message was parsed.
    Truncated {
        /// Byte offset where input terminated.
        at: ByteOffset,
    },
    /// Bytes at `at` violate the HTTP/1.1 grammar (RFC-7230).
    Malformed {
        /// Byte offset of the malformed datum.
        at: ByteOffset,
        /// Human-readable description of the grammar rule violated (G11).
        why: String,
    },
    /// A `Status` code was outside the valid `100..=599` range (never clamped).
    OutOfRange {
        /// The rejected status code.
        code: u16,
    },
    /// A method token contained characters outside the RFC-7230 `token` production.
    InvalidMethod {
        /// The rejected token.
        token: String,
    },
    /// A URL could not be parsed per the RFC-3986/WHATWG subset.
    InvalidUrl {
        /// Byte offset of the parse failure.
        at: ByteOffset,
        /// Why the URL was rejected.
        why: String,
    },
    /// A header field name contained characters outside the RFC-7230 `token` production.
    InvalidHeaderName {
        /// The rejected name.
        name: String,
    },
    /// A header field value contained invalid octets (RFC-7230 field-value rules).
    InvalidHeaderValue {
        /// The rejected value.
        value: String,
    },
}

impl fmt::Display for HttpParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpParseError::Truncated { at } => write!(f, "truncated HTTP message at {at}"),
            HttpParseError::Malformed { at, why } => {
                write!(f, "malformed HTTP at {at}: {why}")
            }
            HttpParseError::OutOfRange { code } => {
                write!(
                    f,
                    "status code {code} is out of range 100..=599 (never a clamp — C1/G2)"
                )
            }
            HttpParseError::InvalidMethod { token } => {
                write!(f, "invalid HTTP method token {token:?}")
            }
            HttpParseError::InvalidUrl { at, why } => {
                write!(f, "invalid URL at {at}: {why}")
            }
            HttpParseError::InvalidHeaderName { name } => {
                write!(f, "invalid header field name {name:?}")
            }
            HttpParseError::InvalidHeaderValue { value } => {
                write!(f, "invalid header field value {value:?}")
            }
        }
    }
}

mycelium_std_core::impl_std_error!(HttpParseError);

// ── Method ────────────────────────────────────────────────────────────────────

/// HTTP request method (RFC-7231 §4 + RFC-5789 `PATCH`; extensible via `Extension`).
///
/// # Guarantee
/// - Constructors: `Exact` / Total (no accuracy semantics; the variant is the value).
/// - `Method::parse`: `Exact`-when-`Ok`, Fallible `Err(InvalidMethod)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
    /// An extension method token (RFC-7230 §3.2.6 `token`).
    Extension(String),
}

impl Method {
    /// Parse a method from a byte string.
    ///
    /// # Guarantee: `Exact`-when-`Ok`, Fallible `Err(InvalidMethod)`
    /// Standard tokens parse to typed variants; non-standard valid tokens produce
    /// `Extension(token)`. Any token containing characters outside the RFC-7230 `token`
    /// production (i.e. not in `! # $ % & ' * + - . ^ _ ` | ~ 0-9 A-Z a-z`) is
    /// refused with `Err(InvalidMethod)` — never silent, never clamped (C1/G2).
    pub fn parse(bytes: &[u8]) -> Result<Self, HttpParseError> {
        let token = std::str::from_utf8(bytes).map_err(|_| HttpParseError::InvalidMethod {
            token: format!("<invalid UTF-8 bytes: {} bytes>", bytes.len()),
        })?;
        if !is_valid_token(token) {
            return Err(HttpParseError::InvalidMethod {
                token: token.to_owned(),
            });
        }
        Ok(match token {
            "GET" => Method::Get,
            "HEAD" => Method::Head,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "DELETE" => Method::Delete,
            "CONNECT" => Method::Connect,
            "OPTIONS" => Method::Options,
            "TRACE" => Method::Trace,
            "PATCH" => Method::Patch,
            other => Method::Extension(other.to_owned()),
        })
    }

    /// The canonical ASCII string representation of the method.
    ///
    /// # Guarantee: `Exact` / Total
    pub fn as_str(&self) -> &str {
        match self {
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Connect => "CONNECT",
            Method::Options => "OPTIONS",
            Method::Trace => "TRACE",
            Method::Patch => "PATCH",
            Method::Extension(s) => s.as_str(),
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// RFC-7230 §3.2.6 `token` production.
///
/// A token character is any VCHAR except `( ) < > @ , ; : \ " / [ ] ? = { } SP HTAB`.
fn is_valid_token(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    const SEPARATORS: &[char] = &[
        '(', ')', '<', '>', '@', ',', ';', ':', '\\', '"', '/', '[', ']', '?', '=', '{', '}', ' ',
        '\t',
    ];
    s.chars()
        .all(|c| c.is_ascii() && c > '\x1f' && c != '\x7f' && !SEPARATORS.contains(&c))
}

// ── Status ────────────────────────────────────────────────────────────────────

/// A validated HTTP status code in `100..=599` (RFC-7231 §6).
///
/// # Guarantee
/// - `Status::from_u16`: `Exact`-when-`Ok`, Fallible `Err(OutOfRange{code})` — **never a clamp**.
///   A code outside `100..=599` is returned as-is in the error, not adjusted silently (C1/G2).
/// - `Status::as_u16` / `Status::class`: `Exact` / Total.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Status(u16);

impl Status {
    /// Parse a status code from a `u16`.
    ///
    /// # Guarantee: `Exact`-when-`Ok`, Fallible `Err(HttpParseError::OutOfRange{code})`
    /// A code outside `100..=599` is **refused** — never clamped, never wrapped around (C1/G2).
    pub fn from_u16(code: u16) -> Result<Self, HttpParseError> {
        if (100..=599).contains(&code) {
            Ok(Status(code))
        } else {
            Err(HttpParseError::OutOfRange { code })
        }
    }

    /// The numeric status code.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn as_u16(self) -> u16 {
        self.0
    }

    /// The status class (1xx–5xx).
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn class(self) -> StatusClass {
        match self.0 / 100 {
            1 => StatusClass::Informational,
            2 => StatusClass::Success,
            3 => StatusClass::Redirection,
            4 => StatusClass::ClientError,
            _ => StatusClass::ServerError,
        }
    }

    // Commonly-used constants (never-clamped; validated at compile-time via the `const`
    // `WELL_FORMED_STATUS_CODES_MUST_BE_100_599` assertion in tests).
    /// `200 OK`
    pub const OK: Status = Status(200);
    /// `404 Not Found`
    pub const NOT_FOUND: Status = Status(404);
    /// `405 Method Not Allowed`
    pub const METHOD_NOT_ALLOWED: Status = Status(405);
    /// `400 Bad Request`
    pub const BAD_REQUEST: Status = Status(400);
    /// `500 Internal Server Error`
    pub const INTERNAL_SERVER_ERROR: Status = Status(500);
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// HTTP status class (1xx–5xx).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClass {
    Informational,
    Success,
    Redirection,
    ClientError,
    ServerError,
}

// ── Headers ───────────────────────────────────────────────────────────────────

/// HTTP header field collection — value-semantic multimap (RFC-7230 §3.2).
///
/// # Semantics
/// Field names are stored case-insensitively (lowercased on insert per RFC-7230 §3.2).
/// A field may appear multiple times; `header_get` returns all values for a name.
///
/// Internally `BTreeMap<String, Vec<String>>` — inspectable, no hidden hash state (C3).
///
/// # Guarantee: `Exact` / Total (accessors); `Exact`-when-`Ok` / Fallible (mutating ops)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Headers {
    // BTreeMap for deterministic ordering (inspectable / EXPLAIN-able — C3).
    inner: BTreeMap<String, Vec<String>>,
}

impl Headers {
    /// Create an empty `Headers`.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn new() -> Self {
        Headers {
            inner: BTreeMap::new(),
        }
    }

    /// Insert a header field.
    ///
    /// The field name is lowercased (RFC-7230 case-insensitive comparison via canonical form).
    ///
    /// # Guarantee: `Exact`-when-`Ok`, Fallible `Err(InvalidHeaderName|InvalidHeaderValue)`
    pub fn insert(
        &mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), HttpParseError> {
        let name = name.into();
        let value = value.into();
        validate_header_name(&name)?;
        validate_header_value(&value)?;
        let key = name.to_ascii_lowercase();
        self.inner.entry(key).or_default().push(value);
        Ok(())
    }

    /// Retrieve all values for a header field name (case-insensitive).
    ///
    /// Returns an empty slice if the field is absent — never `None` vs `Some([])` ambiguity (C1).
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn get(&self, name: &str) -> &[String] {
        let key = name.to_ascii_lowercase();
        match self.inner.get(&key) {
            Some(values) => values.as_slice(),
            None => &[],
        }
    }

    /// Get the first value for a header field name (case-insensitive), or `None`.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn get_first(&self, name: &str) -> Option<&str> {
        self.get(name).first().map(|s| s.as_str())
    }

    /// Iterate over all `(name, value)` pairs in sorted field-name order.
    ///
    /// # Guarantee: `Exact` / Total (EXPLAIN-able: BTreeMap order is deterministic)
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner.iter().flat_map(|(name, values)| {
            values
                .iter()
                .map(move |value| (name.as_str(), value.as_str()))
        })
    }

    /// Number of distinct field names (not total entries).
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// True if no fields are present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// RFC-7230 §3.2.6: header field names must be valid `token` chars.
fn validate_header_name(name: &str) -> Result<(), HttpParseError> {
    if !is_valid_token(name) {
        return Err(HttpParseError::InvalidHeaderName {
            name: name.to_owned(),
        });
    }
    Ok(())
}

/// RFC-7230 §3.2.6: header field values must be visible ASCII or SP/HTAB, no control chars.
fn validate_header_value(value: &str) -> Result<(), HttpParseError> {
    // RFC-7230 field-value: any VCHAR (0x21..=0x7E), SP (0x20), HTAB (0x09).
    // Leading/trailing OWS is allowed; internal NUL or other control chars are not.
    for b in value.bytes() {
        match b {
            0x09 | 0x20 | 0x21..=0x7E => {} // HTAB | SP | VCHAR
            _ => {
                return Err(HttpParseError::InvalidHeaderValue {
                    value: value.to_owned(),
                });
            }
        }
    }
    Ok(())
}

// ── Url ──────────────────────────────────────────────────────────────────────

/// A URL parsed per the RFC-3986/WHATWG subset used by HTTP/1.1.
///
/// Stores the raw string plus extracted components. The parser accepts absolute HTTP/HTTPS URLs
/// and origin-form paths (e.g. `/foo/bar?baz=1`). Fragments are ignored (RFC-7230 §5.1).
///
/// # Guarantee
/// - `Url::parse`: `Exact`-when-`Ok`, Fallible `Err(HttpParseError::InvalidUrl{at, why})`.
/// - Accessors (`scheme`, `host`, `path`, `query`): `Exact` / Total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url {
    raw: String,
    scheme: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    path: String,
    query: Option<String>,
}

impl Url {
    /// Parse a URL from bytes (RFC-3986/WHATWG subset for HTTP/1.1).
    ///
    /// # Guarantee: `Exact`-when-`Ok`, Fallible `Err(HttpParseError::InvalidUrl{at, why})`
    /// Any URL that cannot be parsed is refused with a located error — never silently accepted
    /// with a truncated representation (C1/G2).
    pub fn parse(bytes: &[u8]) -> Result<Self, HttpParseError> {
        let raw = std::str::from_utf8(bytes).map_err(|e| HttpParseError::InvalidUrl {
            at: ByteOffset(e.valid_up_to() as u64),
            why: "URL contains invalid UTF-8".to_owned(),
        })?;
        Self::parse_str(raw)
    }

    /// Parse a URL from a string.
    ///
    /// # Guarantee: `Exact`-when-`Ok`, Fallible `Err(HttpParseError::InvalidUrl{at, why})`
    pub fn parse_str(raw: &str) -> Result<Self, HttpParseError> {
        if raw.is_empty() {
            return Err(HttpParseError::InvalidUrl {
                at: ByteOffset(0),
                why: "empty URL".to_owned(),
            });
        }
        // Origin-form: starts with '/'.
        if raw.starts_with('/') {
            let (path, query) = split_path_query(raw);
            return Ok(Url {
                raw: raw.to_owned(),
                scheme: None,
                host: None,
                port: None,
                path: path.to_owned(),
                query: query.map(|s| s.to_owned()),
            });
        }
        // Absolute-form: must start with http:// or https://
        let (scheme, rest) = if let Some(r) = raw.strip_prefix("https://") {
            ("https", r)
        } else if let Some(r) = raw.strip_prefix("http://") {
            ("http", r)
        } else {
            return Err(HttpParseError::InvalidUrl {
                at: ByteOffset(0),
                why: format!(
                    "URL must be origin-form ('/…') or absolute-form ('http[s]://…'), got {raw:?}"
                ),
            });
        };
        let (authority, path_and_query) = if let Some(slash) = rest.find('/') {
            (&rest[..slash], &rest[slash..])
        } else {
            (rest, "/")
        };
        let (host_str, port) =
            parse_authority(authority).map_err(|why| HttpParseError::InvalidUrl {
                at: ByteOffset(scheme.len() as u64 + 3),
                why,
            })?;
        let (path, query) = split_path_query(path_and_query);
        Ok(Url {
            raw: raw.to_owned(),
            scheme: Some(scheme.to_owned()),
            host: Some(host_str.to_owned()),
            port,
            path: path.to_owned(),
            query: query.map(|s| s.to_owned()),
        })
    }

    /// The raw URL string.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// The scheme (`"http"` / `"https"`), or `None` for origin-form.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn scheme(&self) -> Option<&str> {
        self.scheme.as_deref()
    }

    /// The host component, or `None` for origin-form.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    /// The port, or `None` when not specified.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// The path component (always starts with `/`).
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The query string (without the leading `?`), or `None`.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

/// Split `path_and_query` at the first `?`.
fn split_path_query(s: &str) -> (&str, Option<&str>) {
    if let Some(q) = s.find('?') {
        (&s[..q], Some(&s[q + 1..]))
    } else {
        (s, None)
    }
}

/// Parse `host[:port]` from an authority component.
fn parse_authority(authority: &str) -> Result<(String, Option<u16>), String> {
    if let Some(colon) = authority.rfind(':') {
        let host = &authority[..colon];
        let port_str = &authority[colon + 1..];
        let port: u16 = port_str
            .parse()
            .map_err(|_| format!("invalid port {port_str:?}"))?;
        Ok((host.to_owned(), Some(port)))
    } else {
        Ok((authority.to_owned(), None))
    }
}

// ── Body ─────────────────────────────────────────────────────────────────────

/// An HTTP message body — in-memory bytes (streaming `Source` is research-gated: FLAG U2/U8).
///
/// # Guarantee: `Exact` / Total (value type; no accuracy semantics).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Body {
    bytes: Vec<u8>,
}

impl Body {
    /// Create an empty body.
    #[must_use]
    pub fn empty() -> Self {
        Body { bytes: Vec::new() }
    }

    /// Create a body from bytes.
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Body { bytes }
    }

    /// Create a body from a UTF-8 string.
    #[must_use]
    pub fn from_string(s: impl Into<String>) -> Self {
        Body {
            bytes: s.into().into_bytes(),
        }
    }

    /// The raw bytes.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Length in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// True if the body is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Consume the body into its raw bytes.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

// ── Request ───────────────────────────────────────────────────────────────────

/// An HTTP/1.1 request (value type).
///
/// # Guarantee
/// - `Request::new`: `Exact` / Total.
/// - `with_header`: `Exact`-when-`Ok`, Fallible `Err(InvalidHeaderName|InvalidHeaderValue)`.
/// - Accessors (`method`, `url`, `path`, `headers`): `Exact` / Total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    method: Method,
    url: Url,
    headers: Headers,
    body: Body,
}

impl Request {
    /// Create a new request.
    ///
    /// # Guarantee: `Exact` / Total
    /// Returns `Self` unconditionally — header validation was done at `Headers` construction
    /// time (`Headers::insert`); this constructor accepts already-validated components (C1/G2).
    pub fn new(method: Method, url: Url, headers: Headers, body: Body) -> Self {
        Request {
            method,
            url,
            headers,
            body,
        }
    }

    /// Add a header field, returning the modified request (builder pattern).
    ///
    /// # Guarantee: `Exact`-when-`Ok`, Fallible `Err(InvalidHeaderName|InvalidHeaderValue)`
    pub fn with_header(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, HttpParseError> {
        self.headers.insert(name, value)?;
        Ok(self)
    }

    /// The HTTP method.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// The URL.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// The URL path.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn path(&self) -> &str {
        self.url.path()
    }

    /// Get a header field value (case-insensitive). Returns the first value or `None`.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn header_get(&self, name: &str) -> Option<&str> {
        self.headers.get_first(name)
    }

    /// All headers.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// The request body.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn body(&self) -> &Body {
        &self.body
    }
}

// ── Response ──────────────────────────────────────────────────────────────────

/// An HTTP/1.1 response (value type).
///
/// # Guarantee: `Exact` / Total for all accessors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    status: Status,
    headers: Headers,
    body: Body,
}

impl Response {
    /// Create a new response.
    #[must_use]
    pub fn new(status: Status, headers: Headers, body: Body) -> Self {
        Response {
            status,
            headers,
            body,
        }
    }

    /// Add a header field, returning the modified response.
    ///
    /// # Guarantee: `Exact`-when-`Ok`, Fallible
    pub fn with_header(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, HttpParseError> {
        self.headers.insert(name, value)?;
        Ok(self)
    }

    /// The status code.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn status(&self) -> Status {
        self.status
    }

    /// Get a header field value (case-insensitive). Returns the first value or `None`.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn header_get(&self, name: &str) -> Option<&str> {
        self.headers.get_first(name)
    }

    /// All headers.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// The response body.
    ///
    /// # Guarantee: `Exact` / Total
    #[must_use]
    pub fn body(&self) -> &Body {
        &self.body
    }
}

// ── HTTP/1.1 parsers ──────────────────────────────────────────────────────────
//
// Minimal hand-written HTTP/1.1 request/response parsers following RFC-7230.
// These handle the HTTP/1.1 case only; HTTP/2+TLS is FLAGGED-gated (U3–U5).

/// Parse an HTTP/1.1 request from bytes.
///
/// # Guarantee: `Exact`-when-`Ok`, Fallible `Err(HttpParseError)` with located errors (C1/G2)
/// The parser follows RFC-7230 §3 (request-line + CRLF + headers + CRLF + body). Every
/// parse failure returns a located `HttpParseError` — never a partial/silently-truncated result.
///
/// # FLAGGED: HTTP/2 + HTTP/3 + TLS (U3–U5) are NOT implemented here; this parser handles
/// HTTP/1.1 only. A non-HTTP/1.1 framing is detected and refused with `Err(Malformed)`.
pub fn parse_request(bytes: &[u8]) -> Result<Request, HttpParseError> {
    let mut cursor = 0usize;

    // Request-line: `METHOD SP Request-URI SP HTTP/1.1 CRLF`
    let (method, url, after_line) = parse_request_line(bytes, cursor)?;
    cursor = after_line;

    // Headers + blank line
    let (headers, after_headers) = parse_headers(bytes, cursor)?;
    cursor = after_headers;

    // Body: everything remaining
    let body = Body::from_bytes(bytes[cursor..].to_vec());

    Ok(Request::new(method, url, headers, body))
}

/// Parse an HTTP/1.1 response from bytes.
///
/// # Guarantee: `Exact`-when-`Ok`, Fallible `Err(HttpParseError)` with located errors (C1/G2)
pub fn parse_response(bytes: &[u8]) -> Result<Response, HttpParseError> {
    let mut cursor = 0usize;

    // Status-line: `HTTP/1.1 SP Status-Code SP Reason-Phrase CRLF`
    let (status, after_line) = parse_status_line(bytes, cursor)?;
    cursor = after_line;

    // Headers + blank line
    let (headers, after_headers) = parse_headers(bytes, cursor)?;
    cursor = after_headers;

    // Body
    let body = Body::from_bytes(bytes[cursor..].to_vec());

    Ok(Response::new(status, headers, body))
}

/// Serialize an HTTP/1.1 request to bytes.
///
/// # Guarantee: `Exact` / Total (for well-constructed `Request` values — header validation
/// was done at construction time). No new validation here; serialize is a faithful projection.
///
/// # FLAGGED: streaming body is not supported (U2/U8 gated); body must fit in memory.
pub fn serialize_request(req: &Request) -> Vec<u8> {
    let mut out = Vec::new();
    // Request-line
    out.extend_from_slice(req.method().as_str().as_bytes());
    out.push(b' ');
    out.extend_from_slice(req.url().as_str().as_bytes());
    out.push(b' ');
    out.extend_from_slice(b"HTTP/1.1\r\n");
    // Headers
    for (name, value) in req.headers().iter() {
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(b": ");
        out.extend_from_slice(value.as_bytes());
        out.extend_from_slice(b"\r\n");
    }
    // Blank line
    out.extend_from_slice(b"\r\n");
    // Body
    out.extend_from_slice(req.body().as_bytes());
    out
}

// ── Internal parse helpers ────────────────────────────────────────────────────

/// Find the next CRLF in `bytes[from..]`, return its start offset and the offset after.
fn find_crlf(bytes: &[u8], from: usize) -> Option<(usize, usize)> {
    let window = &bytes[from..];
    window
        .windows(2)
        .position(|w| w == b"\r\n")
        .map(|pos| (from + pos, from + pos + 2))
}

/// Parse the request-line, return `(Method, Url, offset_after_crlf)`.
fn parse_request_line(bytes: &[u8], start: usize) -> Result<(Method, Url, usize), HttpParseError> {
    let (crlf, after) = find_crlf(bytes, start).ok_or(HttpParseError::Truncated {
        at: ByteOffset(bytes.len() as u64),
    })?;
    let line = &bytes[start..crlf];

    // Split on spaces: METHOD SP URI SP HTTP/1.x
    let mut parts = line.splitn(3, |&b| b == b' ');
    let method_bytes = parts.next().ok_or(HttpParseError::Malformed {
        at: ByteOffset(start as u64),
        why: "missing method in request-line".to_owned(),
    })?;
    let uri_bytes = parts.next().ok_or(HttpParseError::Malformed {
        at: ByteOffset(start as u64 + method_bytes.len() as u64 + 1),
        why: "missing URI in request-line".to_owned(),
    })?;
    let version = parts.next().unwrap_or(b"");
    if version != b"HTTP/1.1" {
        return Err(HttpParseError::Malformed {
            at: ByteOffset(crlf as u64 - version.len() as u64),
            why: format!(
                "expected HTTP/1.1 version, got {:?}",
                String::from_utf8_lossy(version)
            ),
        });
    }

    let method = Method::parse(method_bytes)?;
    let url = Url::parse(uri_bytes)?;
    Ok((method, url, after))
}

/// Parse the status-line, return `(Status, offset_after_crlf)`.
fn parse_status_line(bytes: &[u8], start: usize) -> Result<(Status, usize), HttpParseError> {
    let (crlf, after) = find_crlf(bytes, start).ok_or(HttpParseError::Truncated {
        at: ByteOffset(bytes.len() as u64),
    })?;
    let line = &bytes[start..crlf];

    // `HTTP/1.x SP NNN SP Reason`
    let mut parts = line.splitn(3, |&b| b == b' ');
    let version = parts.next().unwrap_or(b"");
    if version != b"HTTP/1.1" {
        return Err(HttpParseError::Malformed {
            at: ByteOffset(start as u64),
            why: format!(
                "expected HTTP/1.1 version in status-line, got {:?}",
                String::from_utf8_lossy(version)
            ),
        });
    }
    let code_bytes = parts.next().ok_or(HttpParseError::Malformed {
        at: ByteOffset(start as u64 + version.len() as u64 + 1),
        why: "missing status code in status-line".to_owned(),
    })?;
    let code_str = std::str::from_utf8(code_bytes).map_err(|_| HttpParseError::Malformed {
        at: ByteOffset(start as u64 + version.len() as u64 + 1),
        why: "status code is not valid UTF-8".to_owned(),
    })?;
    let code: u16 = code_str
        .trim()
        .parse()
        .map_err(|_| HttpParseError::Malformed {
            at: ByteOffset(start as u64 + version.len() as u64 + 1),
            why: format!("status code {code_str:?} is not a valid integer"),
        })?;
    let status = Status::from_u16(code)?;
    Ok((status, after))
}

/// Parse CRLF-terminated header fields until a blank line. Returns `(Headers, offset_after_blank)`.
fn parse_headers(bytes: &[u8], start: usize) -> Result<(Headers, usize), HttpParseError> {
    let mut headers = Headers::new();
    let mut cursor = start;

    loop {
        // Check for blank line (end of headers)
        if bytes.get(cursor..cursor + 2) == Some(b"\r\n") {
            cursor += 2;
            break;
        }
        // Check for truncated input
        if cursor >= bytes.len() {
            return Err(HttpParseError::Truncated {
                at: ByteOffset(cursor as u64),
            });
        }

        let (crlf, after) = find_crlf(bytes, cursor).ok_or(HttpParseError::Truncated {
            at: ByteOffset(bytes.len() as u64),
        })?;
        let line = &bytes[cursor..crlf];

        // Split on first `:`
        let colon =
            line.iter()
                .position(|&b| b == b':')
                .ok_or_else(|| HttpParseError::Malformed {
                    at: ByteOffset(cursor as u64),
                    why: "header field has no colon separator".to_owned(),
                })?;
        let name = std::str::from_utf8(&line[..colon]).map_err(|_| HttpParseError::Malformed {
            at: ByteOffset(cursor as u64),
            why: "header name is not valid UTF-8".to_owned(),
        })?;
        let value_raw =
            std::str::from_utf8(&line[colon + 1..]).map_err(|_| HttpParseError::Malformed {
                at: ByteOffset(cursor as u64 + colon as u64 + 1),
                why: "header value is not valid UTF-8".to_owned(),
            })?;
        let value = value_raw.trim(); // OWS stripping (RFC-7230 §3.2.3)

        headers
            .insert(name, value)
            .map_err(|e| HttpParseError::Malformed {
                at: ByteOffset(cursor as u64),
                why: e.to_string(),
            })?;

        cursor = after;
    }

    Ok((headers, cursor))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Status::from_u16: never-clamp guard (C1/G2) ───────────────────────────

    /// Status codes in range parse successfully.
    #[test]
    fn status_valid_range() {
        assert!(Status::from_u16(100).is_ok());
        assert!(Status::from_u16(200).is_ok());
        assert!(Status::from_u16(404).is_ok());
        assert!(Status::from_u16(599).is_ok());
    }

    /// Status codes outside 100..=599 are refused — **never clamped** (C1/G2).
    /// Guard: returning Ok (clamping or wrapping) for any rejected code makes this fail.
    #[test]
    fn status_out_of_range_is_refused_never_clamped() {
        for bad in [0u16, 99, 600, 1000, u16::MAX] {
            let result = Status::from_u16(bad);
            assert!(
                matches!(result, Err(HttpParseError::OutOfRange { code }) if code == bad),
                "status code {bad} must produce OutOfRange, not Ok or a clamped value (C1/G2)"
            );
        }
    }

    // ── Method::parse ─────────────────────────────────────────────────────────

    #[test]
    fn method_parse_standard() {
        assert_eq!(Method::parse(b"GET").unwrap(), Method::Get);
        assert_eq!(Method::parse(b"POST").unwrap(), Method::Post);
        assert_eq!(Method::parse(b"DELETE").unwrap(), Method::Delete);
        assert_eq!(Method::parse(b"PATCH").unwrap(), Method::Patch);
    }

    #[test]
    fn method_parse_extension() {
        let m = Method::parse(b"PURGE").unwrap();
        assert_eq!(m, Method::Extension("PURGE".to_owned()));
    }

    #[test]
    fn method_parse_invalid_rejected() {
        assert!(Method::parse(b"").is_err());
        assert!(Method::parse(b"GET POST").is_err()); // contains space
        assert!(Method::parse(&[0xFF]).is_err()); // invalid UTF-8
    }

    // ── Url::parse ────────────────────────────────────────────────────────────

    #[test]
    fn url_parse_origin_form() {
        let url = Url::parse(b"/foo/bar?baz=1").unwrap();
        assert_eq!(url.path(), "/foo/bar");
        assert_eq!(url.query(), Some("baz=1"));
        assert_eq!(url.scheme(), None);
    }

    #[test]
    fn url_parse_absolute_form() {
        let url = Url::parse(b"https://example.com/path?q=1").unwrap();
        assert_eq!(url.scheme(), Some("https"));
        assert_eq!(url.host(), Some("example.com"));
        assert_eq!(url.path(), "/path");
        assert_eq!(url.query(), Some("q=1"));
    }

    #[test]
    fn url_parse_empty_is_err() {
        assert!(Url::parse(b"").is_err());
    }

    #[test]
    fn url_parse_invalid_scheme_is_err() {
        assert!(Url::parse(b"ftp://example.com/").is_err());
    }

    // ── Headers ───────────────────────────────────────────────────────────────

    #[test]
    fn headers_insert_and_get() {
        let mut h = Headers::new();
        h.insert("Content-Type", "application/json").unwrap();
        assert_eq!(h.get_first("content-type"), Some("application/json"));
        assert_eq!(h.get_first("Content-Type"), Some("application/json")); // case-insensitive
    }

    #[test]
    fn headers_invalid_name_refused() {
        let mut h = Headers::new();
        assert!(h.insert("Content Type", "x").is_err()); // space in name
        assert!(h.insert("", "x").is_err());
    }

    #[test]
    fn headers_invalid_value_refused() {
        let mut h = Headers::new();
        assert!(h.insert("X-Foo", "bar\x00baz").is_err()); // NUL in value
    }

    // ── parse_request / serialize_request round-trip ──────────────────────────

    #[test]
    fn parse_request_simple_get() {
        let raw = b"GET /hello HTTP/1.1\r\nHost: example.com\r\nContent-Length: 0\r\n\r\n";
        let req = parse_request(raw).unwrap();
        assert_eq!(req.method(), &Method::Get);
        assert_eq!(req.path(), "/hello");
        assert_eq!(req.header_get("host"), Some("example.com"));
    }

    #[test]
    fn parse_request_with_body() {
        let raw = b"POST /submit HTTP/1.1\r\nContent-Type: text/plain\r\n\r\nhello world";
        let req = parse_request(raw).unwrap();
        assert_eq!(req.method(), &Method::Post);
        assert_eq!(req.body().as_bytes(), b"hello world");
    }

    #[test]
    fn parse_request_malformed_missing_crlf() {
        // No CRLF at all → Truncated
        let raw = b"GET /foo HTTP/1.1";
        assert!(parse_request(raw).is_err());
    }

    /// HTTP/2.0 in request line must be rejected — never-silent (C1/G2).
    /// Guard: returning Ok for a non-HTTP/1.1 request would silently accept a wrong protocol.
    #[test]
    fn parse_request_http2_version_is_err_never_silent() {
        let raw = b"GET / HTTP/2.0\r\n\r\n";
        assert!(
            parse_request(raw).is_err(),
            "HTTP/2.0 request must be Err — only HTTP/1.1 is accepted (C1/G2)"
        );
    }

    /// HTTP/2.0 in response status line must be rejected — never-silent (C1/G2).
    /// Guard: returning Ok for a non-HTTP/1.1 response would silently accept a wrong protocol.
    #[test]
    fn parse_response_http2_version_is_err_never_silent() {
        let raw = b"HTTP/2.0 200 OK\r\n\r\n";
        assert!(
            parse_response(raw).is_err(),
            "HTTP/2.0 response must be Err — only HTTP/1.1 is accepted (C1/G2)"
        );
    }

    #[test]
    fn serialize_request_round_trip() {
        let url = Url::parse_str("/ping").unwrap();
        let mut headers = Headers::new();
        headers.insert("host", "example.com").unwrap();
        let req = Request::new(Method::Get, url, headers, Body::empty());
        let serialized = serialize_request(&req);
        let recovered = parse_request(&serialized).unwrap();
        assert_eq!(recovered.method(), &Method::Get);
        assert_eq!(recovered.path(), "/ping");
        assert_eq!(recovered.header_get("host"), Some("example.com"));
    }

    #[test]
    fn parse_response_simple_ok() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello";
        let resp = parse_response(raw).unwrap();
        assert_eq!(resp.status().as_u16(), 200);
        assert_eq!(resp.body().as_bytes(), b"hello");
    }

    #[test]
    fn parse_response_invalid_status_code_refused() {
        // Status 600 is outside range — must be Err
        let raw = b"HTTP/1.1 600 Bogus\r\n\r\n";
        let err = parse_response(raw).unwrap_err();
        assert!(
            matches!(err, HttpParseError::OutOfRange { code: 600 }),
            "600 must produce OutOfRange, got {err:?}"
        );
    }

    // ── Error Display (locus / EXPLAIN — C3/G11) ──────────────────────────────

    #[test]
    fn error_display_includes_locus() {
        let e = HttpParseError::Malformed {
            at: ByteOffset(42),
            why: "bad grammar".to_owned(),
        };
        let s = e.to_string();
        assert!(
            s.contains("42"),
            "Display must include byte offset (C3/G11)"
        );
        assert!(s.contains("bad grammar"), "Display must include why");
    }

    #[test]
    fn error_display_out_of_range_includes_code() {
        let e = HttpParseError::OutOfRange { code: 999 };
        assert!(e.to_string().contains("999"));
    }

    #[test]
    fn http_parse_error_is_std_error() {
        let e = HttpParseError::Truncated { at: ByteOffset(0) };
        let _: &dyn std::error::Error = &e;
    }

    // ── Property tests (VR-5 / one per bound) ─────────────────────────────────

    mod property {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Status round-trip: valid codes parse successfully and round-trip through as_u16.
            /// Guard: a clamp that changes the code would fail `code == parsed.as_u16()`.
            #[test]
            fn prop_status_round_trip(code in 100u16..=599u16) {
                let status = Status::from_u16(code).expect("in-range code must succeed");
                prop_assert_eq!(status.as_u16(), code,
                    "Status::as_u16 must round-trip (never clamped)");
            }

            /// Out-of-range status codes are refused — never clamped (C1/G2).
            /// Guard: returning Ok for any out-of-range code makes this fail.
            #[test]
            fn prop_status_out_of_range_is_always_err(code in prop_oneof![0u16..99u16, 600u16..u16::MAX]) {
                prop_assert!(
                    Status::from_u16(code).is_err(),
                    "code {code} is out of range and must be Err (never clamped — C1/G2)"
                );
            }

            /// Request parse↔serialize round-trip: a serialized request re-parses to the same
            /// method + path + selected headers (Empirical).
            #[test]
            fn prop_request_serialize_parse_round_trip(
                method in prop_oneof![
                    Just(Method::Get), Just(Method::Post), Just(Method::Put),
                    Just(Method::Delete), Just(Method::Head),
                ],
                path in r"/[a-z]{1,8}(/[a-z]{1,8}){0,3}",
            ) {
                let url = Url::parse_str(&path).expect("valid path");
                let req = Request::new(method.clone(), url, Headers::new(), Body::empty());
                let bytes = serialize_request(&req);
                let recovered = parse_request(&bytes).expect("serialized request must parse back");
                prop_assert_eq!(recovered.method(), &method,
                    "round-trip must preserve method");
                prop_assert_eq!(recovered.path(), path.as_str(),
                    "round-trip must preserve path");
            }

            /// Route match: same method + path always maps to the same result (determinism).
            /// (Route-level determinism is also tested in route.rs proptest; this tests http level.)
            #[test]
            fn prop_method_parse_standard_tokens_are_stable(
                token in prop_oneof![
                    Just("GET"), Just("POST"), Just("PUT"), Just("DELETE"),
                    Just("HEAD"), Just("PATCH"), Just("OPTIONS"),
                ],
            ) {
                let parsed = Method::parse(token.as_bytes()).expect("standard token must parse");
                let parsed2 = Method::parse(token.as_bytes()).expect("must parse again");
                prop_assert_eq!(&parsed, &parsed2,
                    "method parse must be deterministic");
            }
        }
    }
}
