//! Fuzz target 1: L1 lexer + recursive-descent parser (H8 class).
//!
//! Invariant: `mycelium_l1::parse` MUST NEVER PANIC on arbitrary `&str` input.
//! Malformed inputs must return an explicit `Err(ParseError)` — never a panic,
//! never a silent accept (S5/G2; stated in the `mycelium-l1` crate doc).
//!
//! Guarantee tag: Empirical (no formal proof; coverage-guided fuzzer confirms
//! no-panic on the observed corpus, not on all possible inputs).
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Accept arbitrary bytes and interpret as UTF-8 if valid; otherwise supply
    // the lossily-decoded string.  The parser must not panic on *any* input —
    // valid UTF-8 or not — so we exercise both paths.
    let s = match std::str::from_utf8(data) {
        Ok(s) => s.to_owned(),
        Err(_) => String::from_utf8_lossy(data).into_owned(),
    };

    // INVARIANT: must never panic. `Err` is the only acceptable failure mode.
    let _ = mycelium_l1::parse(&s);
});
