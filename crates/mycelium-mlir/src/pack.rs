//! The **substrate byte-layout codec** for ternary values (RFC-0004 §5; DN-01) — the AOT/compiled
//! path's model of *how trits are physically packed into bytes* under a chosen [`PackScheme`].
//!
//! This is the substrate-level detail the Core IR deliberately omits (the type is packing-agnostic;
//! DN-01 §6) and the trusted kernel never carries (KC-3 — it lives here, in the AOT crate, not in
//! `mycelium-core`). Each scheme is a **bijective trit↔byte encoding**; decoding bytes under the
//! *wrong* scheme produces a different trit sequence — which is exactly the
//! MLIR-`transpose`/Rust-`packed` class of "a wrong layout tag misreads memory" bug DN-01 §4 cites,
//! and the soundness hazard the E3 differential (M-251) must catch (RFC-0004 §8; NFR-7).
//!
//! Schemes (RFC-0004 §5; the bitnet.cpp set + the two reference packings):
//! - `I2_S`, `TL1`, `TwoBitPerTrit` — **2 bits/trit**, 4 trits/byte, distinguished by their code
//!   LUT (the three rotations of `{0,1,2}`), so the same trits pack to *different* bytes.
//! - `TL2`, `FiveTritPerByte` — **base-3**, 5 trits/byte (`3⁵ = 243 ≤ 256`; ≈1.6–1.67 b/w),
//!   distinguished by digit order.
//! - `Unpacked` — 1 trit/byte.
//!
//! Decoding is **total** (never panics): an out-of-range code/byte folds `mod 3`, so reading a
//! buffer under a mismatched scheme yields *some* trit sequence deterministically — a misread, not
//! a crash. Round-trip under the *same* scheme is the identity ([`pack_trits`] ∘ [`unpack_trits`]).

use mycelium_core::{PackScheme, Trit};

/// Trits-per-byte for `scheme` (the packing density's structural form).
fn group_size(scheme: PackScheme) -> usize {
    match scheme {
        PackScheme::Unpacked => 1,
        PackScheme::TwoBitPerTrit | PackScheme::I2S | PackScheme::Tl1 => 4,
        PackScheme::FiveTritPerByte | PackScheme::Tl2 => 5,
    }
}

/// A trit as a base-3 digit `{0, 1, 2}` (`Neg→0, Zero→1, Pos→2`).
fn d01(t: Trit) -> u8 {
    match t {
        Trit::Neg => 0,
        Trit::Zero => 1,
        Trit::Pos => 2,
    }
}

/// The inverse of [`d01`], total via `mod 3` (so a cross-scheme misread decodes to *a* trit, never
/// a panic).
fn from_d01(d: u8) -> Trit {
    match d % 3 {
        0 => Trit::Neg,
        1 => Trit::Zero,
        _ => Trit::Pos,
    }
}

/// The per-scheme 2-bit code LUT — the three rotations of `{0,1,2}`, so `I2_S`/`TL1`/`TwoBitPerTrit`
/// pack identical trits to *different* bytes (the distinguishing detail E3 relies on).
fn two_bit_rot(scheme: PackScheme) -> u8 {
    match scheme {
        PackScheme::I2S => 0,
        PackScheme::Tl1 => 2,
        PackScheme::TwoBitPerTrit => 1,
        _ => 0,
    }
}

/// The base-3 digit order for the 5-trit-per-byte schemes — `TL2` keeps `d01`, `FiveTritPerByte`
/// reverses it, so they remain distinct encodings.
fn base3_reversed(scheme: PackScheme) -> bool {
    matches!(scheme, PackScheme::FiveTritPerByte)
}

/// Encode `trits` to bytes under `scheme` (bijective; the AOT path's physical buffer). The final
/// partial group is zero-padded; [`unpack_trits`] reads exactly the requested count back.
#[must_use]
pub fn pack_trits(trits: &[Trit], scheme: PackScheme) -> Vec<u8> {
    let g = group_size(scheme);
    let mut bytes = Vec::with_capacity(trits.len().div_ceil(g));
    for chunk in trits.chunks(g) {
        let byte = match scheme {
            PackScheme::Unpacked => d01(chunk[0]),
            PackScheme::TwoBitPerTrit | PackScheme::I2S | PackScheme::Tl1 => {
                let rot = two_bit_rot(scheme);
                let mut b: u8 = 0;
                for (i, &t) in chunk.iter().enumerate() {
                    let code = (d01(t) + rot) % 3; // ∈ {0,1,2}, fits 2 bits
                    b |= code << (2 * i);
                }
                b
            }
            PackScheme::FiveTritPerByte | PackScheme::Tl2 => {
                let rev = base3_reversed(scheme);
                let mut b: u16 = 0;
                let mut p: u16 = 1;
                for &t in chunk {
                    let digit = if rev { 2 - d01(t) } else { d01(t) };
                    b += u16::from(digit) * p;
                    p *= 3;
                }
                u8::try_from(b).expect("five base-3 digits fit in a byte (3^5 = 243 < 256)")
            }
        };
        bytes.push(byte);
    }
    bytes
}

/// Decode `count` trits from `bytes` under `scheme`. **Total**: a code/byte outside the scheme's
/// valid range folds `mod 3`, so reading a buffer packed under a *different* scheme yields a
/// deterministic (wrong) trit sequence — the misread, never a panic.
#[must_use]
pub fn unpack_trits(bytes: &[u8], scheme: PackScheme, count: usize) -> Vec<Trit> {
    let g = group_size(scheme);
    let mut out = Vec::with_capacity(count);
    'outer: for (bi, &byte) in bytes.iter().enumerate() {
        for i in 0..g {
            if bi * g + i >= count {
                break 'outer;
            }
            let trit = match scheme {
                PackScheme::Unpacked => from_d01(byte),
                PackScheme::TwoBitPerTrit | PackScheme::I2S | PackScheme::Tl1 => {
                    let rot = two_bit_rot(scheme);
                    let code = (byte >> (2 * i)) & 0b11;
                    // invert the rotation: d01 = code - rot (mod 3); +3 keeps it non-negative.
                    from_d01((code + 3 - rot) % 3)
                }
                PackScheme::FiveTritPerByte | PackScheme::Tl2 => {
                    let rev = base3_reversed(scheme);
                    let digit = (byte / 3u8.pow(u32::try_from(i).expect("i < 5"))) % 3;
                    from_d01(if rev { 2 - digit } else { digit })
                }
            };
            out.push(trit);
        }
    }
    out
}

/// Re-materialize trits through a pack-then-read round-trip where the buffer is **packed as**
/// `packed_as` but **read back as** `read_as` (the recorded `Meta.physical` tag). When the tag is
/// correct (`packed_as == read_as`) this is the identity; a wrong tag *misreads* the buffer — the
/// soundness hazard the E3 differential catches (RFC-0004 §8; NFR-7).
#[must_use]
pub fn relayout_trits(trits: &[Trit], packed_as: PackScheme, read_as: PackScheme) -> Vec<Trit> {
    let bytes = pack_trits(trits, packed_as);
    unpack_trits(&bytes, read_as, trits.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_SCHEMES: [PackScheme; 6] = [
        PackScheme::Unpacked,
        PackScheme::TwoBitPerTrit,
        PackScheme::FiveTritPerByte,
        PackScheme::I2S,
        PackScheme::Tl1,
        PackScheme::Tl2,
    ];

    fn sample() -> Vec<Trit> {
        // 11 trits: spans a partial 4-group and a partial 5-group, mixed values.
        vec![
            Trit::Neg,
            Trit::Pos,
            Trit::Zero,
            Trit::Pos,
            Trit::Neg,
            Trit::Zero,
            Trit::Pos,
            Trit::Pos,
            Trit::Neg,
            Trit::Zero,
            Trit::Neg,
        ]
    }

    #[test]
    fn round_trip_is_identity_under_the_same_scheme() {
        for s in ALL_SCHEMES {
            let t = sample();
            let back = unpack_trits(&pack_trits(&t, s), s, t.len());
            assert_eq!(back, t, "scheme {s:?} must round-trip losslessly");
            // relayout with a matching tag is the identity.
            assert_eq!(relayout_trits(&t, s, s), t);
        }
    }

    #[test]
    fn the_three_bitnet_schemes_are_mutually_distinct_encodings() {
        // The E3 precondition: a buffer packed under one bitnet scheme, read under another, misreads.
        let bitnet = [PackScheme::I2S, PackScheme::Tl1, PackScheme::Tl2];
        let t = sample();
        for &a in &bitnet {
            for &b in &bitnet {
                if a != b {
                    assert_ne!(
                        relayout_trits(&t, a, b),
                        t,
                        "packing as {a:?} then reading as {b:?} must diverge"
                    );
                }
            }
        }
    }

    #[test]
    fn an_all_zero_buffer_still_misreads_across_schemes() {
        // Even the degenerate all-Zero value diverges (the LUTs map Zero differently), so E3 does
        // not rely on lucky test data.
        let t = vec![Trit::Zero; 5];
        assert_ne!(relayout_trits(&t, PackScheme::I2S, PackScheme::Tl1), t);
        assert_ne!(relayout_trits(&t, PackScheme::I2S, PackScheme::Tl2), t);
    }

    #[test]
    fn decoding_is_total_on_arbitrary_bytes() {
        // Reading arbitrary bytes (e.g. a TL2 buffer under a 2-bit scheme) never panics.
        let bytes = [0xFF, 0x00, 0xAB, 0x7C, 242];
        for s in ALL_SCHEMES {
            let _ = unpack_trits(&bytes, s, 9);
        }
    }
}
