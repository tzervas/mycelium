//! nodule: `adk.session` — content-addressed state + event log, per RFC-0023 §4.
//!
//! TODO(leaf ADK / M-671): `State`, `Event`, `Session { state, events }` as **immutable,
//! value-semantic snapshots** — a "mutation" (`put(state, key, v) -> State`) returns a new
//! snapshot (content-addressed identity, ADR-003), it never mutates in place. Pure.
