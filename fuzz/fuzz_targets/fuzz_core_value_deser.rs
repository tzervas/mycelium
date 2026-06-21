//! Fuzz target 3: `mycelium-core` serde `Value` and `Meta` JSON deserializers.
//!
//! Invariant: `serde_json::from_slice::<Value>` and `serde_json::from_slice::<Meta>`
//! MUST NEVER PANIC on arbitrary byte input. Malformed/mismatched JSON must return
//! `Err` — never a panic, never a silently-accepted bad value (RFC-0001 §4.8; G2).
//!
//! The `Value::deserialize` impl re-runs `Value::new` (repr well-formedness +
//! payload↔repr agreement) and `Meta::deserialize` re-runs M-I1…M-I4 on the way
//! in — so malformed wire data is rejected, never silently trusted. This target
//! confirms those paths survive adversarial bytes.
//!
//! Guarantee tag: Empirical (coverage-guided fuzzer; not exhaustive).
#![no_main]

use libfuzzer_sys::fuzz_target;
use mycelium_core::{Meta, Value};

fuzz_target!(|data: &[u8]| {
    // Both deserializers must not panic. `Err` is the only acceptable outcome for
    // invalid bytes — never a panic.
    let _ = serde_json::from_slice::<Value>(data);
    let _ = serde_json::from_slice::<Meta>(data);
});
