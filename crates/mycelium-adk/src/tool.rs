//! nodule: `adk.tool` — typed tools with explicit error sets, per RFC-0023 §4 / §6.2.
//!
//! TODO(leaf ADK / M-671): `ToolError { BadArgs | OutOfDomain | Refused | Upstream }` and a pure
//! tool surface using **Rust generics** (not the gated Mycelium-language generics) — every call is
//! an explicit `Result<Out, ToolError>`, **never a silent `None`/default** (C1/G2). The effectful
//! `run_io` (graft, `io` effect) is FLAGGED-deferred.
