//! nodule: `web.client` — `get` / `post` / `request`, per RFC-0022 §4.1 / §4.5.
//!
//! Build the **pure / value** surface now: request construction and response parsing (reusing
//! [`crate::http`]). Define a `Transport` trait seam (mirror `std.io`'s `Substrate`) with an
//! in-memory / loopback implementation for tests.
//!
//! FLAGGED-gated (never faked): the **real socket transport** (U2 socket-floor, U8 `net`-effect
//! granularity) is deferred behind the seam — a production socket path returns an explicit,
//! never-silent `Err`, not a stub success. `get_json` (transfer + decode) is `Empirical`.
