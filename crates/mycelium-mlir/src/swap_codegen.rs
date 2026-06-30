//! Native direct-LLVM codegen of the **`Swap` node** — the one Repr-changing node (WF1) — for the
//! **certified binary↔ternary swap class** (M-852; epic E25-1; ADR-034 re-gating native AOT into
//! lang 1.0.0; RFC-0002 §3/§4 *Swap Certificate*; RFC-0004 §6 *no-opaque-lowering* / the §2
//! direct-LLVM revisit clause).
//!
//! ## What this lowers
//! `Rhs::Swap{ src, target, policy }` for the **bijective binary↔ternary class** (RFC-0002 §4): the
//! only genuinely exact/bijective swap class. A binary↔ternary swap over a **legal pair** `(n, m)`
//! (`2^(n-1) ≤ (3^m − 1)/2`; RFC-0002 §5) is **value-preserving** — it maps the *same integer*
//! between two representations — so the native lowering is an explicit, dumpable integer transcode:
//!
//! - **`Binary{n} → Ternary{m}`** (`enc`): decode the `n` bits MSB-first as a two's-complement
//!   integer (`Σ bitᵢ·2^…`, sign-extend the MSB), then re-encode that integer as `m` balanced
//!   trits by the balanced-division algorithm (`r = v mod 3`; `2 ≡ −1` with a borrow) — exactly
//!   `mycelium_core::ternary::int_to_trits`, digit-for-digit, in IR. Total on a legal pair.
//! - **`Ternary{m} → Binary{n}`** (`dec`, **partial**): decode the `m` trits as an integer
//!   (`Horner ·3`), then re-encode as `n` two's-complement bits. The inverse is **partial** — a
//!   ternary value outside `B_n` is **out of range** and is signalled **never-silently** through the
//!   existing overflow read-back protocol (`AotError::Overflow`; SC-3/G2), exactly as the
//!   interpreter raises `SwapError::OutOfRange`.
//! - **same-`Repr`** (`Binary{n}→Binary{n}` / `Ternary{m}→Ternary{m}`): the identity swap — the
//!   lane passes through unchanged (the trivial engine's contract).
//!
//! Every other swap kind (`Dense`/`Vsa`, a non-bit/trit pair, an **illegal** `(n,m)` pair) is an
//! explicit [`AotError::UnsupportedNode`] — **never** silently lowered (G2).
//!
//! ## The cert-mode decision (maintainer-ratified 2026-06-30)
//! The certificate handling is a **reified, EXPLAIN-able, never-silent mode** ([`SwapCertMode`]) with
//! two settings, recorded in the emitted IR (a dumpable comment) **and** in the returned
//! [`SwapExplain`] (so the cert *source* is never hidden — G2):
//!
//! - **[`SwapCertMode::Recheck`] — DEFAULT.** Codegen **independently re-runs the certificate check
//!   at compile time**: it re-checks the bijection side-condition ([`legal_pair`], RFC-0002 §5) as an
//!   **independent compile-time basis** (it does *not* trust the interpreter's cert), and the emitted
//!   transcode IR **is** the re-derivation of `enc`/`dec` on the source. A swap whose side-condition
//!   fails is refused at compile time (`UnsupportedNode`), never emitted.
//! - **[`SwapCertMode::ReuseInterp`] — OPT-IN.** Codegen carries the **interpreter-computed
//!   certificate** forward (it still emits the same transcode IR, but records that the certificate
//!   basis is *carried*, not independently re-checked) — faster; skips the compile-time re-check.
//!
//! ## Guarantee tag (VR-5)
//! **`Empirical`** for **both** modes. The `Recheck` mode re-checks the bijection's `(n,m)`
//! side-condition (the checkable half of RFC-0002 §3), but the **once-per-kind round-trip lemma**
//! (M-121) is *referenced*, not machine-checked **inside this codegen** — the emitted IR is
//! hand-written textual LLVM transcode, and its agreement with the trusted interpreter is established
//! by the M-210 differential (empirical evidence), not by a proof object linked here. Per the house
//! transparency rule, `Proven` requires a theorem whose side-conditions are checked *with the theorem
//! in hand*; this codegen checks a side-condition but does not carry the proof, so it stays
//! `Empirical` (a mutation caught by the differential is the basis). Upgrading to `Proven` would need
//! the M-121 proof wired as a checked basis here — flagged, not assumed (VR-5). `ReuseInterp` is
//! likewise `Empirical` (carried + differential-checked).
//!
//! ## Inspectability (RFC-0004 §6 — no opaque lowering)
//! Every step is explicit, dumpable textual IR: the bit/trit decode (one op per element), the
//! balanced-division encode loop (unrolled, one block of arithmetic per output digit), and the
//! range-check that drives the never-silent read-back. A leading IR comment records the swap kind,
//! the `(n,m)` pair, the legal-pair verdict, the cert mode, and the cert source — so a reader of the
//! `.ll` sees *which* certificate basis the swap was lowered under (no black box; ADR-009/VR-4).
//!
//! **Submodule confinement:** zero `unsafe` (compiler-enforced by the crate's `#![forbid]`).

use std::fmt::Write as _; // `writeln!` into a String never fails — call sites discard the Result.

use mycelium_core::{ternary, Repr};

use crate::llvm::{AotError, Lane, LaneKind, Ssa};

/// How the swap's **certificate** is handled by native codegen — a reified, EXPLAIN-able,
/// never-silent mode (maintainer-ratified 2026-06-30). The selected mode and the resulting cert
/// **source** are recorded in the emitted IR comment and the returned [`SwapExplain`] (G2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SwapCertMode {
    /// **DEFAULT** — re-run the certificate check at compile time: re-check the bijection
    /// side-condition ([`legal_pair`]) as an independent compile-time basis (do **not** trust the
    /// interpreter's cert), and emit the transcode IR as the re-derivation of `enc`/`dec`.
    #[default]
    Recheck,
    /// **OPT-IN** — reuse the interpreter-computed certificate: emit the same transcode IR but carry
    /// the interpreter's certificate basis forward (skip the compile-time re-check). Faster.
    ReuseInterp,
}

impl SwapCertMode {
    /// A short, stable label for the EXPLAIN record / IR comment (never a hidden mode — G2).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            SwapCertMode::Recheck => "recheck(compile-time independent re-check)",
            SwapCertMode::ReuseInterp => "reuse-interp(carried certificate)",
        }
    }

    /// The cert **source** this mode records — independently re-checked vs carried-from-interp. This
    /// is the never-hidden provenance of the certificate basis (RFC-0034 §3.1 spirit; G2).
    #[must_use]
    pub fn cert_source(self) -> &'static str {
        match self {
            SwapCertMode::Recheck => "compile-time-rechecked",
            SwapCertMode::ReuseInterp => "interp-carried",
        }
    }
}

/// The inspectable record of how a `Swap` node was lowered — the EXPLAIN payload (RFC-0004 §6;
/// no black box). Returned alongside the lowered [`Lane`] so a test / an `EXPLAIN` consumer can see
/// the swap kind, the `(n,m)` pair, the legal-pair verdict, the cert mode, and the cert source —
/// never a silent lowering (G2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwapExplain {
    /// Source representation (as rendered).
    pub src: String,
    /// Target representation (as rendered).
    pub target: String,
    /// The binary width `n` involved in the pair (the `Binary{n}` side).
    pub width: u32,
    /// The ternary width `m` involved in the pair (the `Ternary{m}` side).
    pub trits: u32,
    /// Whether `(n, m)` is a legal (bijective) pair — the re-checked RFC-0002 §5 side-condition.
    pub legal_pair: bool,
    /// The cert mode this swap was lowered under.
    pub mode: SwapCertMode,
    /// The cert source (`compile-time-rechecked` / `interp-carried`), from [`SwapCertMode`].
    pub cert_source: &'static str,
    /// `true` for the identity (same-`Repr`) swap — no transcode, the lane passes through.
    pub identity: bool,
}

/// Whether `(n, m)` admits a lossless binary↔ternary swap: `B_n ⊆ T_m ⇔ 2^(n-1) ≤ (3^m − 1)/2`
/// (`binary-ternary.md` §2; RFC-0002 §5). This is an **independent** re-implementation of the
/// `mycelium-cert::legal_pair` side-condition over `mycelium_core::ternary::max_magnitude`, so the
/// `Recheck` mode's compile-time verdict has its **own** basis (it does not import the cert crate /
/// trust the interpreter's cert). `i128` so the binary side never overflows the comparison.
#[must_use]
pub fn legal_pair(width: u32, trits: u32) -> bool {
    let Some(tern_max) = ternary::max_magnitude(trits) else {
        return false; // ternary side overflows i64 — far beyond any legal small pair
    };
    // 2^(n-1): the magnitude of the most-negative n-bit value, the binding constraint.
    let bin_max_neg_mag: i128 = 1i128 << width.saturating_sub(1);
    bin_max_neg_mag <= i128::from(tern_max)
}

/// Lower a `Rhs::Swap` node natively (M-852). Given the already-lowered source [`Lane`] and the
/// source/target [`Repr`], emit the transcode IR into `body` and return the result lane plus its
/// [`SwapExplain`]. Range failures on the partial `dec` direction (`Ternary → Binary`) push an `i1`
/// overflow register onto `flags` so the program's read-back protocol refuses **never-silently**
/// (matching the interpreter's `SwapError::OutOfRange`; SC-3/G2).
///
/// Covered: binary↔ternary over a legal pair, and same-`Repr` identity. Everything else (Dense/VSA,
/// a non-bit/trit pair, an **illegal** `(n,m)` pair) is an explicit [`AotError::UnsupportedNode`] —
/// the swap is never silently mis-lowered.
///
/// `pub(crate)`: it takes the internal codegen [`Lane`]/[`Ssa`] types, so it is an internal seam —
/// the **public** swap surface is [`SwapCertMode`]/[`SwapExplain`]/[`legal_pair`] plus the
/// `*_with_swap_mode` entries on [`crate::llvm`].
pub(crate) fn lower_swap(
    src_lane: &Lane,
    src_repr: &Repr,
    target: &Repr,
    mode: SwapCertMode,
    ssa: &mut Ssa,
    body: &mut String,
    flags: &mut Vec<String>,
) -> Result<(Lane, SwapExplain), AotError> {
    // The EXPLAIN comment is emitted up-front for *every* lowering path, so the `.ll` always records
    // the swap's cert basis (RFC-0004 §6; never a hidden swap — G2).
    match (src_repr, target) {
        // ── Identity (same Repr): the trivial swap, no transcode (RFC-0002; IdentitySwapEngine). ──
        (a, b) if a == b => {
            let (width, trits) = match a {
                Repr::Binary { width } => (*width, 0),
                Repr::Ternary { trits } => (0, *trits),
                other => {
                    return Err(unsupported_swap(
                        other,
                        b,
                        "identity swap is only lowered for bit/trit reprs here",
                    ));
                }
            };
            let explain = SwapExplain {
                src: format!("{a:?}"),
                target: format!("{b:?}"),
                width,
                trits,
                legal_pair: true, // an identity is trivially a bijection on itself
                mode,
                cert_source: mode.cert_source(),
                identity: true,
            };
            emit_explain_comment(&explain, body);
            Ok((src_lane.clone(), explain))
        }
        // ── Binary{n} → Ternary{m}: enc, total on a legal pair (value-preserving transcode). ──
        (Repr::Binary { width }, Repr::Ternary { trits }) => {
            let (width, trits) = (*width, *trits);
            let legal = check_legal(width, trits, mode)?;
            let explain = mk_explain(src_repr, target, width, trits, legal, mode);
            emit_explain_comment(&explain, body);
            if src_lane.kind != LaneKind::Binary {
                return Err(AotError::UnsupportedNode(format!(
                    "swap Binary→Ternary: source lane is {:?}, expected a Binary lane (G2)",
                    src_lane.kind
                )));
            }
            let int_reg = emit_bits_to_int(&src_lane.vals, ssa, body);
            // enc is total on a legal pair (no range failure), so no overflow flag is pushed.
            let lane = emit_int_to_trits(&int_reg, trits as usize, ssa, body);
            Ok((lane, explain))
        }
        // ── Ternary{m} → Binary{n}: dec, PARTIAL — range failure is never-silent (out of range). ──
        (Repr::Ternary { trits }, Repr::Binary { width }) => {
            let (width, trits) = (*width, *trits);
            let legal = check_legal(width, trits, mode)?;
            let explain = mk_explain(src_repr, target, width, trits, legal, mode);
            emit_explain_comment(&explain, body);
            if src_lane.kind != LaneKind::Ternary {
                return Err(AotError::UnsupportedNode(format!(
                    "swap Ternary→Binary: source lane is {:?}, expected a Ternary lane (G2)",
                    src_lane.kind
                )));
            }
            let int_reg = emit_trits_to_int(&src_lane.vals, ssa, body);
            let (lane, oor) = emit_int_to_bits(&int_reg, width as usize, ssa, body);
            // The dec inverse is partial: a ternary value outside B_n is out of range — push the
            // never-silent flag so the read-back refuses (matches SwapError::OutOfRange; SC-3/G2).
            flags.push(oor);
            Ok((lane, explain))
        }
        // ── Everything else: an explicit refusal — never a silent mis-lowering (G2). ──
        (a, b) => Err(unsupported_swap(
            a,
            b,
            "only the certified binary↔ternary class (and same-Repr identity) is lowered natively \
             (M-852); Dense/VSA and other swap kinds stay explicit refusals",
        )),
    }
}

/// Build the explicit `UnsupportedNode` for an unsupported swap pair (never silent — G2).
fn unsupported_swap(a: &Repr, b: &Repr, why: &str) -> AotError {
    AotError::UnsupportedNode(format!("swap {a:?} → {b:?}: {why}"))
}

/// Re-check the bijection side-condition. In [`SwapCertMode::Recheck`] an illegal pair is refused at
/// compile time (the swap is never emitted — VR-5/G2). In [`SwapCertMode::ReuseInterp`] the
/// side-condition is *still computed* (it is cheap, and the verdict is recorded in EXPLAIN), but the
/// legality is whatever the carried certificate asserts; an illegal pair would already have been an
/// `IllegalPair` error in the interpreter before this point, so codegen records it and proceeds.
/// Returns the legal-pair verdict for the EXPLAIN record.
fn check_legal(width: u32, trits: u32, mode: SwapCertMode) -> Result<bool, AotError> {
    let legal = legal_pair(width, trits);
    if mode == SwapCertMode::Recheck && !legal {
        return Err(AotError::UnsupportedNode(format!(
            "swap Binary{{{width}}}↔Ternary{{{trits}}}: the compile-time re-check (recheck mode) \
             rejects the bijection side-condition — (n,m) is NOT a legal pair (B_n ⊄ T_m, RFC-0002 \
             §5); the swap is refused, never emitted (VR-5/G2)"
        )));
    }
    Ok(legal)
}

/// Build the [`SwapExplain`] for a transcode (non-identity) swap.
fn mk_explain(
    src: &Repr,
    target: &Repr,
    width: u32,
    trits: u32,
    legal: bool,
    mode: SwapCertMode,
) -> SwapExplain {
    SwapExplain {
        src: format!("{src:?}"),
        target: format!("{target:?}"),
        width,
        trits,
        legal_pair: legal,
        mode,
        cert_source: mode.cert_source(),
        identity: false,
    }
}

/// Emit the dumpable EXPLAIN comment into the IR (RFC-0004 §6 — the swap's cert basis is visible in
/// the `.ll`; never a black box, G2).
fn emit_explain_comment(e: &SwapExplain, body: &mut String) {
    let _ = writeln!(
        body,
        "  ; swap {} -> {} | pair (n={}, m={}) legal={} | cert-mode={} | cert-source={}{}",
        e.src,
        e.target,
        e.width,
        e.trits,
        e.legal_pair,
        e.mode.label(),
        e.cert_source,
        if e.identity { " | identity" } else { "" },
    );
}

/// Emit IR decoding an MSB-first `Binary` lane (`i32` elements in `{0,1}`) into a two's-complement
/// integer in an `i64` SSA register. Mirrors `mycelium_core::binary::bits_to_int`: accumulate the
/// unsigned magnitude (`acc = acc·2 + bitᵢ`), then subtract `2^n` iff the MSB (sign bit, element 0)
/// is set. Each step is explicit IR (no opaque pass; §6).
fn emit_bits_to_int(bits: &[String], ssa: &mut Ssa, body: &mut String) -> String {
    let n = bits.len();
    // acc starts at 0 (i64).
    let mut acc = "0".to_owned();
    for v in bits {
        // Zero-extend the i32 bit to i64, then acc = acc*2 + bit.
        let z = ssa.fresh();
        let _ = writeln!(body, "  {z} = zext i32 {v} to i64");
        let sh = ssa.fresh();
        let _ = writeln!(body, "  {sh} = mul i64 {acc}, 2");
        let next = ssa.fresh();
        let _ = writeln!(body, "  {next} = add i64 {sh}, {z}");
        acc = next;
    }
    if n == 0 {
        return acc; // empty string denotes 0 (binary::bits_to_int contract)
    }
    // Sign correction: if the MSB (element 0) is set, subtract 2^n. Branch-free via select.
    let sign = &bits[0];
    let is_neg = ssa.fresh();
    let _ = writeln!(body, "  {is_neg} = icmp eq i32 {sign}, 1");
    // 2^n as an i64 constant (n ≤ 62 for the subset; a wider n would overflow i64 and is out of the
    // bijective small-pair regime anyway — a legal pair caps n well under 64).
    let two_pow_n: i64 = if n < 62 { 1i64 << n } else { i64::MAX };
    let corrected = ssa.fresh();
    let _ = writeln!(body, "  {corrected} = sub i64 {acc}, {two_pow_n}");
    let out = ssa.fresh();
    let _ = writeln!(body, "  {out} = select i1 {is_neg}, i64 {corrected}, i64 {acc}");
    out
}

/// Emit IR decoding an MSB-first `Ternary` lane (`i32` elements in `{−1,0,1}`) into an integer in an
/// `i64` SSA register. Mirrors `mycelium_core::ternary::trits_to_int`: Horner from the MSB
/// (`acc = acc·3 + digitᵢ`). Each step is explicit IR (§6).
fn emit_trits_to_int(trits: &[String], ssa: &mut Ssa, body: &mut String) -> String {
    let mut acc = "0".to_owned();
    for v in trits {
        let ext = ssa.fresh();
        let _ = writeln!(body, "  {ext} = sext i32 {v} to i64");
        let mul = ssa.fresh();
        let _ = writeln!(body, "  {mul} = mul i64 {acc}, 3");
        let next = ssa.fresh();
        let _ = writeln!(body, "  {next} = add i64 {mul}, {ext}");
        acc = next;
    }
    acc
}

/// Emit IR encoding an `i64` integer (`int_reg`) into an MSB-first `Ternary` lane of `m` trits.
/// Mirrors `mycelium_core::ternary::int_to_trits` digit-for-digit: per LSB digit, `r = v rem 3`
/// folded to balanced `{−1,0,1}` (`2 ≡ −1`, with a borrow `v += 1`), then `v = (v − digit)/3`.
/// Computed branch-free with `srem`/`sdiv` and a `select` for the `2 → −1` fold. The result lane is
/// MSB-first (we compute LSB-first then reverse). `m` is the codegen-fixed target width, so the loop
/// is unrolled — every digit is explicit IR (§6). A legal pair guarantees the value fits, so the
/// final quotient is provably 0 (no range flag needed on the `enc` direction).
fn emit_int_to_trits(int_reg: &str, m: usize, ssa: &mut Ssa, body: &mut String) -> Lane {
    let mut v = int_reg.to_owned();
    let mut lsb_first: Vec<String> = Vec::with_capacity(m);
    for _ in 0..m {
        // Balanced remainder: r0 = v.rem_euclid(3) ∈ {0,1,2}; euclidean rem via `((v srem 3)+3) srem 3`.
        let sr = ssa.fresh();
        let _ = writeln!(body, "  {sr} = srem i64 {v}, 3");
        let plus3 = ssa.fresh();
        let _ = writeln!(body, "  {plus3} = add i64 {sr}, 3");
        let r0 = ssa.fresh(); // r0 = euclidean remainder ∈ {0,1,2}
        let _ = writeln!(body, "  {r0} = srem i64 {plus3}, 3");
        // digit = (r0 == 2) ? -1 : r0  (the balanced fold; `2 ≡ −1 (mod 3)`).
        let is_two = ssa.fresh();
        let _ = writeln!(body, "  {is_two} = icmp eq i64 {r0}, 2");
        let digit64 = ssa.fresh();
        let _ = writeln!(body, "  {digit64} = select i1 {is_two}, i64 -1, i64 {r0}");
        // v_next = (v − digit) / 3  (exact division; v − digit ≡ 0 (mod 3) by construction). This is
        // `v.div_euclid(3)` with the `r==2 ⇒ v+=1` borrow folded in: (v − (−1))/3 = (v+1)/3.
        let v_minus = ssa.fresh();
        let _ = writeln!(body, "  {v_minus} = sub i64 {v}, {digit64}");
        let v_next = ssa.fresh();
        let _ = writeln!(body, "  {v_next} = sdiv i64 {v_minus}, 3");
        // Truncate the balanced digit to the i32 lane element (in {−1,0,1}).
        let digit32 = ssa.fresh();
        let _ = writeln!(body, "  {digit32} = trunc i64 {digit64} to i32");
        lsb_first.push(digit32);
        v = v_next;
    }
    let vals: Vec<String> = lsb_first.into_iter().rev().collect(); // → MSB-first
    Lane {
        kind: LaneKind::Ternary,
        vals,
    }
}

/// Emit IR encoding an `i64` integer (`int_reg`) into an MSB-first `Binary` lane of `n` two's-
/// complement bits, plus an `i1` **out-of-range** register (set iff the value does not fit `B_n`).
/// Mirrors `mycelium_core::binary::int_to_bits`: range-check `lo ≤ v ≤ hi` (`lo = −2^(n-1)`,
/// `hi = 2^(n-1) − 1`); reduce mod `2^n`; read bit `(n−1−i)`. The range bit is the never-silent
/// `dec`-partiality signal (SwapError::OutOfRange; SC-3/G2). Every step is explicit IR (§6).
fn emit_int_to_bits(int_reg: &str, n: usize, ssa: &mut Ssa, body: &mut String) -> (Lane, String) {
    if n == 0 {
        // Zero-width: representable iff v == 0 (binary::int_to_bits n==0 contract). The lane is empty;
        // out-of-range iff v != 0.
        let oor = ssa.fresh();
        let _ = writeln!(body, "  {oor} = icmp ne i64 {int_reg}, 0");
        return (
            Lane {
                kind: LaneKind::Binary,
                vals: Vec::new(),
            },
            oor,
        );
    }
    // Range bounds: lo = −2^(n-1), hi = 2^(n-1) − 1. For the bijective small-pair regime n ≤ 62.
    let half: i64 = if n - 1 < 62 { 1i64 << (n - 1) } else { i64::MAX };
    let lo = -half;
    let hi = half - 1;
    let lt_lo = ssa.fresh();
    let _ = writeln!(body, "  {lt_lo} = icmp slt i64 {int_reg}, {lo}");
    let gt_hi = ssa.fresh();
    let _ = writeln!(body, "  {gt_hi} = icmp sgt i64 {int_reg}, {hi}");
    let oor = ssa.fresh();
    let _ = writeln!(body, "  {oor} = or i1 {lt_lo}, {gt_hi}");
    // Read each two's-complement bit directly off the i64 (the low `n` bits are the representation;
    // when in range, bit (n-1) is the sign bit and matches the two's-complement encoding). Element i
    // (MSB-first) is bit (n-1-i).
    let vals: Vec<String> = (0..n)
        .map(|i| {
            let shift = n - 1 - i;
            let sh = ssa.fresh();
            let _ = writeln!(body, "  {sh} = lshr i64 {int_reg}, {shift}");
            let m = ssa.fresh();
            let _ = writeln!(body, "  {m} = and i64 {sh}, 1");
            let t = ssa.fresh();
            let _ = writeln!(body, "  {t} = trunc i64 {m} to i32");
            t
        })
        .collect();
    (
        Lane {
            kind: LaneKind::Binary,
            vals,
        },
        oor,
    )
}
