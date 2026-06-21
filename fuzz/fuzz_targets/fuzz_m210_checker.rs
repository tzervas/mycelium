//! Fuzz target 2: M-210 shared checker entrypoint.
//!
//! Strategy: feed arbitrary bytes as source text to `mycelium_check::check_sources`.
//! This exercises the full parse → type-check → diagnostic-routing pipeline on
//! arbitrary input. For inputs that parse successfully, it also exercises
//! `mycelium_l1::check_nodule` (the trusted M-210 checker kernel).
//!
//! Invariant: neither the parser nor the checker must EVER panic. A malformed
//! input must yield explicit `FindingKind::Parse` or `FindingKind::Check`
//! findings — never a panic (G2/VR-5).
//!
//! Guarantee tag: Empirical (coverage-guided fuzzer; not exhaustive).
#![no_main]

use libfuzzer_sys::fuzz_target;
use mycelium_check::check_sources;

fuzz_target!(|data: &[u8]| {
    let src = match std::str::from_utf8(data) {
        Ok(s) => s.to_owned(),
        Err(_) => String::from_utf8_lossy(data).into_owned(),
    };

    // check_sources must not panic on any input. Findings (parse or check errors)
    // are the explicit, expected failure mode.
    let _report = check_sources(&[("fuzz_input.myc".to_owned(), src)]);
});
