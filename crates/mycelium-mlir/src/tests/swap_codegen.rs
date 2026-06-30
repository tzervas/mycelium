//! In-crate white-box tests for `swap_codegen.rs` (M-852; CLAUDE.md test-layout rule). These are
//! pure **emission** + **logic** checks (no toolchain): the `legal_pair` side-condition, the
//! `SwapCertMode` EXPLAIN record/comment, the never-silent refusals, and that the emitted IR carries
//! the dumpable cert-basis comment (RFC-0004 §6). The compiled-path differential (interp ≡ native,
//! both cert modes, M-210-checked) lives in `tests/swap_differential.rs`.

use crate::llvm::{emit_llvm_ir, emit_llvm_ir_with_swap_mode};
use crate::swap_codegen::{legal_pair, SwapCertMode, SwapExplain};
use mycelium_core::{ContentHash, Meta, Node, Payload, Provenance, Repr, Trit, Value};

fn binary(bits: Vec<bool>) -> Value {
    let width = bits.len() as u32;
    Value::new(
        Repr::Binary { width },
        Payload::Bits(bits),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

fn ternary(trits: Vec<Trit>) -> Value {
    let m = trits.len() as u32;
    Value::new(
        Repr::Ternary { trits: m },
        Payload::Trits(trits),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

fn policy() -> ContentHash {
    ContentHash::parse("blake3:round_trip_safe").unwrap()
}

fn swap_b_to_t(bits: Vec<bool>, m: u32) -> Node {
    Node::Swap {
        src: Box::new(Node::Const(binary(bits))),
        target: Repr::Ternary { trits: m },
        policy: policy(),
    }
}

// ─── legal_pair (the re-checked RFC-0002 §5 side-condition) ──────────────────────────────────────

/// `legal_pair` matches the published worked pairs: `(8,6)` legal (128 ≤ 364), `(8,4)` illegal
/// (128 > 40), `(4,4)` legal (8 ≤ 40), `(4,3)` legal (8 ≤ 13), `(8,3)` illegal (128 > 13).
#[test]
fn legal_pair_matches_the_side_condition() {
    assert!(legal_pair(8, 6), "(8,6) is legal: 2^7=128 ≤ (3^6−1)/2=364");
    assert!(!legal_pair(8, 4), "(8,4) is illegal: 128 > (3^4−1)/2=40");
    assert!(legal_pair(4, 4), "(4,4) is legal: 2^3=8 ≤ 40");
    assert!(legal_pair(4, 3), "(4,3) is legal: 8 ≤ (3^3−1)/2=13");
    assert!(!legal_pair(8, 3), "(8,3) is illegal: 128 > 13");
}

/// `legal_pair` is the **independent** re-implementation of `mycelium-cert::legal_pair` — it agrees
/// with the cert crate's verdict over a small grid (so the `Recheck` basis is the same side-condition,
/// just computed in this crate without importing cert).
#[test]
fn legal_pair_agrees_with_the_cert_crate() {
    for n in 1u32..=12 {
        for m in 1u32..=10 {
            assert_eq!(
                legal_pair(n, m),
                mycelium_cert::legal_pair(n, m),
                "legal_pair({n},{m}) disagrees with mycelium-cert"
            );
        }
    }
}

// ─── EXPLAIN record + the dumpable IR comment (RFC-0004 §6, no black box) ─────────────────────────

/// The emitted IR carries the dumpable swap cert-basis comment for **both** modes, naming the cert
/// source (never hidden — G2). The two modes record distinct cert sources.
#[test]
fn emitted_ir_records_the_cert_mode_and_source() {
    let prog = swap_b_to_t(vec![true, false, true, true, false, false, true, false], 6);
    let recheck = emit_llvm_ir_with_swap_mode(&prog, SwapCertMode::Recheck).unwrap();
    let reuse = emit_llvm_ir_with_swap_mode(&prog, SwapCertMode::ReuseInterp).unwrap();

    // The dumpable comment names the swap, the (n,m) pair, legal=true, the mode, and the source.
    assert!(
        recheck.contains("; swap") && recheck.contains("legal=true"),
        "recheck IR must carry the dumpable swap comment:\n{recheck}"
    );
    assert!(
        recheck.contains("cert-source=compile-time-rechecked"),
        "recheck IR must record the compile-time-rechecked cert source:\n{recheck}"
    );
    assert!(
        reuse.contains("cert-source=interp-carried"),
        "reuse IR must record the interp-carried cert source:\n{reuse}"
    );
    // The transcode itself (the value-preserving enc) is the same in both modes — the mode only
    // changes the recorded basis, not the emitted arithmetic.
    let strip = |s: &str| {
        s.lines()
            .filter(|l| !l.trim_start().starts_with("; swap"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(
        strip(&recheck),
        strip(&reuse),
        "the two cert modes must emit identical transcode IR (only the comment differs)"
    );
}

/// The default `emit_llvm_ir` uses the `Recheck` mode (the project default — compile-time re-check).
#[test]
fn default_emit_uses_recheck_mode() {
    let prog = swap_b_to_t(vec![false; 8], 6);
    let default = emit_llvm_ir(&prog).unwrap();
    let recheck = emit_llvm_ir_with_swap_mode(&prog, SwapCertMode::Recheck).unwrap();
    assert_eq!(default, recheck, "default emit must be the Recheck mode");
    assert!(default.contains("cert-source=compile-time-rechecked"));
}

/// `SwapCertMode` labels/sources are stable, distinct, and never empty (the EXPLAIN strings — G2).
#[test]
fn swap_cert_mode_labels_are_distinct_and_nonempty() {
    assert_ne!(
        SwapCertMode::Recheck.label(),
        SwapCertMode::ReuseInterp.label()
    );
    assert_ne!(
        SwapCertMode::Recheck.cert_source(),
        SwapCertMode::ReuseInterp.cert_source()
    );
    assert!(!SwapCertMode::Recheck.label().is_empty());
    assert!(!SwapCertMode::ReuseInterp.cert_source().is_empty());
    assert_eq!(SwapCertMode::default(), SwapCertMode::Recheck);
}

/// The `SwapExplain` is constructible and its fields round-trip the mode/source pairing (the
/// inspectable record an EXPLAIN consumer reads).
#[test]
fn swap_explain_pairs_mode_and_source() {
    let e = SwapExplain {
        src: "Binary { width: 8 }".into(),
        target: "Ternary { trits: 6 }".into(),
        width: 8,
        trits: 6,
        legal_pair: true,
        mode: SwapCertMode::ReuseInterp,
        cert_source: SwapCertMode::ReuseInterp.cert_source(),
        identity: false,
    };
    assert_eq!(e.cert_source, "interp-carried");
    assert!(!e.identity);
}

// ─── the IR transcode shape (no opaque pass — §6) ────────────────────────────────────────────────

/// The Binary→Ternary transcode emits the explicit decode (`zext`/`mul`/`add` for `bits_to_int`) and
/// the balanced-division encode (`srem`/`sdiv`/`select` for `int_to_trits`) — every step visible IR.
#[test]
fn b_to_t_emits_explicit_transcode_ir() {
    let prog = swap_b_to_t(vec![true, false, true, true, false, false, true, false], 6);
    let ir = emit_llvm_ir(&prog).unwrap();
    assert!(ir.contains("zext i32"), "bits_to_int zext missing:\n{ir}");
    assert!(ir.contains("srem i64"), "int_to_trits srem missing:\n{ir}");
    assert!(ir.contains("sdiv i64"), "int_to_trits sdiv missing:\n{ir}");
    assert!(
        ir.contains("select i1"),
        "balanced fold select missing:\n{ir}"
    );
    // The ternary output read-back uses the '-'(45)/'0'(48)/'+'(43) select chain.
    assert!(ir.contains("i32 45") && ir.contains("i32 43"));
}

/// The Ternary→Binary transcode emits the `dec` decode (`sext`/`mul`/`add` for `trits_to_int`) and
/// the range-check (`icmp slt`/`icmp sgt`/`or i1`) that drives the never-silent out-of-range read-back.
#[test]
fn t_to_b_emits_range_checked_transcode_ir() {
    let prog = Node::Swap {
        src: Box::new(Node::Const(ternary(vec![Trit::Zero, Trit::Pos, Trit::Neg]))),
        target: Repr::Binary { width: 4 },
        policy: policy(),
    };
    let ir = emit_llvm_ir(&prog).unwrap();
    assert!(ir.contains("sext i32"), "trits_to_int sext missing:\n{ir}");
    // The never-silent range check + the overflow read-back branch.
    assert!(
        ir.contains("icmp slt i64") && ir.contains("icmp sgt i64"),
        "range check missing:\n{ir}"
    );
    assert!(
        ir.contains("br i1") && ir.contains("ovf:"),
        "the never-silent out-of-range read-back branch must be emitted:\n{ir}"
    );
}

// ─── emission determinism ────────────────────────────────────────────────────────────────────────

#[test]
fn swap_emission_is_deterministic() {
    let prog = swap_b_to_t(vec![true, false, true, true, false, false, true, false], 6);
    assert_eq!(emit_llvm_ir(&prog), emit_llvm_ir(&prog));
}

// ─── never-silent refusals (G2) ──────────────────────────────────────────────────────────────────

/// An **illegal pair** is refused at compile time in `Recheck` mode (`UnsupportedNode`) — never
/// emitted (VR-5/G2). The `emit_llvm_ir` default (Recheck) surfaces the refusal during lowering.
#[test]
fn illegal_pair_is_refused_in_recheck_emit() {
    let prog = swap_b_to_t(vec![true, false, true, true, false, false, true, false], 4); // (8,4) illegal
    match emit_llvm_ir(&prog) {
        Err(crate::llvm::AotError::UnsupportedNode(msg)) => {
            assert!(
                msg.contains("legal pair") || msg.contains("recheck"),
                "refusal must name the side-condition; got: {msg}"
            );
        }
        other => panic!("Recheck emit must refuse the illegal pair (8,4), got {other:?}"),
    }
}

/// An identity swap (same `Repr`) lowers to a pass-through (no transcode), recorded as `identity` in
/// the dumpable comment — still never silent (the comment is present).
#[test]
fn identity_swap_passes_through_with_an_explain_comment() {
    let prog = Node::Swap {
        src: Box::new(Node::Const(binary(vec![true, false, true, false]))),
        target: Repr::Binary { width: 4 },
        policy: policy(),
    };
    let ir = emit_llvm_ir(&prog).unwrap();
    assert!(
        ir.contains("identity"),
        "the identity swap must record the identity comment:\n{ir}"
    );
    // No balanced-division transcode is emitted for an identity (the lane passes through).
    assert!(
        !ir.contains("srem i64"),
        "an identity swap must not emit a transcode:\n{ir}"
    );
}
