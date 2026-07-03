//! Fuzz target 5: depth-structured M-210 checker fuzzing (RFC-0041 / RR-29 §4 — the deep-nesting
//! regression net, W0 "safety net" wave).
//!
//! Mirrors `fuzz_m210_checker.rs`'s strategy (feed source text to `check_sources`, which exercises
//! parse -> type-check -> diagnostic-routing, and — for inputs that parse — `mycelium_l1::check_nodule`
//! transitively), but *deliberately synthesizes* a deeply-nested expression from the fuzz input's
//! derived depth (see `depth_common::nested_source`) instead of hoping arbitrary bytes happen to
//! nest. This stresses the checker/totality/elaboration passes that run **after** a successful
//! parse — the parser's own `MAX_EXPR_DEPTH` guard bounds the *parsed* AST depth (so most derived
//! depths here will be rejected at parse time with an explicit `ParseError`), but this target keeps
//! the checker's own recursive passes (`checkty.rs`, `totality.rs`, `usefulness.rs`, `elab.rs`) in
//! the depth-fuzzing net in case any of them is reachable on a path the parser guard doesn't fully
//! cover (RR-29 §4's stated worry — a guard fixed at one layer, not audited at every layer).
//!
//! Invariant under test: neither the parser nor the checker may ever panic/abort. A malformed or
//! over-deep input must yield an explicit `FindingKind::Parse`/`FindingKind::Check` finding — never
//! a crash (G2/VR-5).
//!
//! Guarantee tag: Empirical (coverage-guided fuzzer over a depth-structured input space; not
//! exhaustive).
#![no_main]

use libfuzzer_sys::fuzz_target;
use mycelium_check::check_sources;

#[path = "depth_common.rs"]
mod depth_common;
use depth_common::{derive_depth, nested_source};

/// Same rationale as `fuzz_depth_parse`'s `MAX_DEPTH`: comfortably past `MAX_EXPR_DEPTH` (256),
/// bounded to keep a single fuzz iteration cheap.
const MAX_DEPTH: usize = 8192;

fuzz_target!(|data: &[u8]| {
    let (depth, rest) = derive_depth(data, MAX_DEPTH);
    let use_brackets = rest.first().is_some_and(|b| b % 2 == 0);
    let src = nested_source(depth, use_brackets);

    // check_sources must not panic/abort on any input, at any synthesized depth. Findings (parse or
    // check errors) are the explicit, expected failure mode.
    let _report = check_sources(&[("fuzz_depth_input.myc".to_owned(), src)]);
});
