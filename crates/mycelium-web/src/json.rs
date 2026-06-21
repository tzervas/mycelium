//! nodule: `web.json` — a thin convenience over `std.io`'s one canonical JSON codec
//! (no new codec — DRY/KC-3), per RFC-0022 §4.1 / §4.5.
//!
//! TODO(leaf WEB / M-670): `encode_body(&Value) -> Result<Body, JsonError>` delegating to
//! `mycelium_std_io::to_json` (non-finite f64 refused, **never a silent `null`**; `Exact`-when-`Ok`),
//! and `decode_body(&Body) -> Result<Value, JsonError>` delegating to `mycelium_std_io::from_json`
//! (`Empirical` — round-trip verified by proptest, not `Proven`). Carry the `std.io` locus.
