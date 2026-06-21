//! nodule: `web.server` — the server as a `colony` of request-handling `hyphae`
//! (RFC-0008 R1), per RFC-0022 §4.1 / §4.5.
//!
//! Build the **structural** surface now: route a parsed request to its handler and dispatch on
//! the runtime [`mycelium_std_runtime::colony::Scope`] (per-request join is `Empirical`-via-RT2,
//! **not `Proven`** — matches `Scope::join_all`'s honest tag). The handler-purity contract is
//! `Declared` (the type system cannot enforce it; always FLAGGED).
//!
//! FLAGGED-gated (never faked): the **real socket bind / accept-loop** (U2 socket-floor) is
//! deferred — `serve` against a real listener returns an explicit, never-silent `Err`.
