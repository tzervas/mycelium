//! `web` — a Rust-first web-tooling phylum: HTTP/1.1 value model + parsing, JSON, routing,
//! client, and server, per **RFC-0022** (M-670).
//!
//! # Status — implemented Rust-first, pending ratification
//! RFC-0022 is **`Draft`**. The `dfr` deep-research pass (**RP-10**) gates its §10
//! Honest-Uncertainty Register; until it discharges, this crate builds only the
//! *design-decidable-now* core (RFC-0022 §10.2 **D1–D10**). Every research-gated surface is
//! **FLAGGED** and fails with an explicit, never-silent error — it is never silently faked:
//! - real socket transport / accept-loop (U2 socket-floor, U8 `net`-effect granularity),
//! - HTTP/2 + HTTP/3 + TLS (U3–U5),
//! - the `.myc` typed-`Json<T>` / `colony { hypha … }` surfaces (E7-1 / E7-2).
//!
//! # Honesty crux (RFC-0016 §4.1 C1–C6)
//! - **Never-silent (C1/G2):** every parse is an explicit `Result` with a *located* error
//!   (byte offset / field); a `Status` outside `100..=599` is an `Err`, never a clamp.
//! - **No new codec (DRY):** [`json`] delegates to `std.io`'s one canonical JSON projection.
//! - **EXPLAIN-able routing (C3):** [`route`] reifies its table; a match yields an inspectable
//!   record (which pattern + captures), never an opaque dispatch.
//! - **Honest tags (C2/VR-5):** see [`guarantee_matrix`] — `Exact`-when-`Ok` for parsing,
//!   `Empirical` for JSON round-trip + server join, `Declared` for the handler-purity contract.
//!   No op is `Proven` (no checked theorem exists).
//!
//! # Nodules (RFC-0022 §4.1)
//! [`http`] · [`json`] · [`route`] · [`client`] · [`server`].
#![forbid(unsafe_code)]

pub mod client;
pub mod guarantee_matrix;
pub mod http;
pub mod json;
pub mod route;
pub mod server;
