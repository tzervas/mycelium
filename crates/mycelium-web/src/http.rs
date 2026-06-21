//! nodule: `web.http` — the HTTP/1.1 value model (`Method` / `Status` / `Headers` / `Url` /
//! `Body` / `Request` / `Response`) and never-silent parsing, per RFC-0022 §4.1 / §4.5.
//!
//! TODO(leaf WEB / M-670): build the non-gated core — `Status::from_u16` (validated `100..=599`,
//! **never a clamp**), `Method::parse`, `Url::parse` (RFC-3986 / WHATWG subset),
//! `parse_request` / `parse_response` (explicit located errors carrying a byte offset),
//! accessors, and `serialize_request`. Copy the located-error pattern from
//! `crates/mycelium-std-io/src/error.rs` (`SerError` with `ByteOffset` / `FieldPath`).
