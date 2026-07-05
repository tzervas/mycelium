//! Fuzz target 4: depth-structured L1 parser fuzzing (RFC-0041 / RR-29 §4 — the deep-nesting
//! regression net, W0 "safety net" wave).
//!
//! Unlike `fuzz_l1_parse` (arbitrary bytes as source text, coverage-guided), this target
//! *deliberately synthesizes* a deeply-nested expression — `((((…0b0…))))` or `[[[[…0b0…]]]]` —
//! from the fuzz input's derived depth, so the durability tier reliably exercises the
//! recursive-descent parser's nesting path rather than hoping a random byte soup happens to nest.
//!
//! Invariant under test: `mycelium_l1::parse` must **never panic or abort**, at any nesting depth.
//! The parser's `MAX_EXPR_DEPTH` guard (`crates/mycelium-l1/src/parse.rs`, A4-02/DN-40) is meant to
//! turn deep nesting into an explicit `ParseError` well before the host stack would overflow, and
//! `parse`/`parse_phylum` already run on `mycelium_stack::with_deep_stack`'s managed stack as a
//! second line of defense. This target is the fuzz-side regression net confirming that guard holds
//! under adversarial depths — an `Ok` or `Err` is a pass; a SIGABRT is the bug (tracked by the
//! RFC-0041 fix waves, not by this harness).
//!
//! Guarantee tag: Empirical (coverage-guided fuzzer over a depth-structured input space; not
//! exhaustive — it does not prove the guard correct for every depth, only that observed depths
//! don't crash).
#![no_main]

use libfuzzer_sys::fuzz_target;

#[path = "depth_common.rs"]
mod depth_common;
use depth_common::{derive_depth, nested_source};

/// Well above `MAX_EXPR_DEPTH` (256 at time of writing) so the guard is exercised deep past its
/// boundary, but bounded so a single fuzz iteration stays cheap (source-string construction here is
/// O(depth), separate from whatever the parser itself does past the guard).
const MAX_DEPTH: usize = 8192;

fuzz_target!(|data: &[u8]| {
    let (depth, rest) = derive_depth(data, MAX_DEPTH);
    let use_brackets = rest.first().is_some_and(|b| b % 2 == 0);
    let src = nested_source(depth, use_brackets);

    // INVARIANT: must never panic/abort. `Ok(Nodule)` or `Err(ParseError)` are the only acceptable
    // outcomes — including for depths well past MAX_EXPR_DEPTH.
    let _ = mycelium_l1::parse(&src);
});
