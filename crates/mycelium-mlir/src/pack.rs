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
//! **A5-08 stand-in note.** This base-3 `TL2` is a placeholder: at 5 trits/byte it realizes 1.6
//! b/w, whereas the published bitnet.cpp TL2 figure (and the selector's cost model in
//! `mycelium-select`) is **1.67 b/w**. The discrepancy is inert for *selection* (both are < 2.0, so
//! TL2 still wins the exhaustive cheapest). The M-360 native TL2 **dot kernel** (`bitnet`) decodes
//! *this* placeholder codec — so the kernel landing does **not** change the b/w; it inherits the
//! 1.6 stand-in. Aligning to bitnet.cpp's true 1.67-b/w TL2 bit-packing is therefore tied to the
//! M-360 **real-layout / SIMD** increment (not the scalar kernel), and stays flagged here and on
//! `packing_bits_per_element` in `mycelium-select/src/lib.rs` until then.
//!
//! Decoding is **total** (never panics): an out-of-range code/byte folds `mod 3`, so reading a
//! buffer under a mismatched scheme yields *some* trit sequence deterministically — a misread, not
//! a crash. Round-trip under the *same* scheme is the identity ([`pack_trits`] ∘ [`unpack_trits`]).

use mycelium_core::{PackScheme, Trit};

/// A packing-codec error. A short buffer is **explicit** (A5-03): `unpack_trits` never silently
/// truncates to fewer trits than requested — a buffer that cannot hold `count` trits under the
/// scheme's density is the diagnostic [`PackError::BufferTooShort`], not a quiet partial decode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackError {
    /// The byte buffer is too short to decode `count` trits under `scheme`: `count` needs at least
    /// `needed` bytes (`count.div_ceil(trits_per_byte)`) but only `got` were supplied.
    BufferTooShort {
        /// The trit count requested.
        count: usize,
        /// The minimum bytes the scheme requires for `count` trits.
        needed: usize,
        /// The bytes actually supplied.
        got: usize,
    },
}

impl core::fmt::Display for PackError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PackError::BufferTooShort { count, needed, got } => write!(
                f,
                "buffer too short to decode {count} trits: need {needed} bytes, got {got}"
            ),
        }
    }
}

impl std::error::Error for PackError {}

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

/// Decode `count` trits from `bytes` under `scheme`. A code/byte outside the scheme's valid range
/// folds `mod 3`, so reading a buffer packed under a *different* scheme yields a deterministic
/// (wrong) trit sequence — the misread, never a panic.
///
/// A buffer too short for `count` trits is the explicit [`PackError::BufferTooShort`] (A5-03):
/// the codec never silently returns fewer trits than requested. When the buffer is long enough,
/// decoding cannot fail.
pub fn unpack_trits(
    bytes: &[u8],
    scheme: PackScheme,
    count: usize,
) -> Result<Vec<Trit>, PackError> {
    let g = group_size(scheme);
    let needed = count.div_ceil(g);
    if bytes.len() < needed {
        return Err(PackError::BufferTooShort {
            count,
            needed,
            got: bytes.len(),
        });
    }
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
    Ok(out)
}

/// Re-materialize trits through a pack-then-read round-trip where the buffer is **packed as**
/// `packed_as` but **read back as** `read_as` (the recorded `Meta.physical` tag). When the tag is
/// correct (`packed_as == read_as`) this is the identity; a wrong tag *misreads* the buffer — the
/// soundness hazard the E3 differential catches (RFC-0004 §8; NFR-7).
#[must_use]
pub fn relayout_trits(trits: &[Trit], packed_as: PackScheme, read_as: PackScheme) -> Vec<Trit> {
    let mut bytes = pack_trits(trits, packed_as);
    // A denser `packed_as` (5 trits/byte) emits fewer bytes than a sparser `read_as` (4 trits/byte)
    // needs for the same count; zero-pad to the bytes `read_as` requires so the read is the modeled
    // misread (a wrong layout tag over the *same* buffer, zero-extended) — never an explicit short.
    let needed = trits.len().div_ceil(group_size(read_as));
    if bytes.len() < needed {
        bytes.resize(needed, 0);
    }
    unpack_trits(&bytes, read_as, trits.len())
        .expect("buffer zero-padded to read_as's required length, so the read cannot be short")
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
            let back = unpack_trits(&pack_trits(&t, s), s, t.len()).unwrap();
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
        // Reading arbitrary bytes (e.g. a TL2 buffer under a 2-bit scheme) never panics, as long as
        // the buffer is long enough for the requested count. The sparsest scheme is `Unpacked` at
        // 1 trit/byte, so 5 bytes supply at least 5 trits under *every* scheme; request 5 so the
        // length precondition holds across the board (A5-03 makes a short buffer an explicit error,
        // not a panic or a silent truncation — exercised separately below).
        let bytes = [0xFF, 0x00, 0xAB, 0x7C, 242];
        for s in ALL_SCHEMES {
            let _ = unpack_trits(&bytes, s, 5).unwrap();
        }
    }

    #[test]
    fn a_short_buffer_is_an_explicit_error_not_a_silent_truncation() {
        // A5-03 mutant-witness: before the fix `unpack_trits` silently returned fewer trits than
        // requested when `bytes` was too short. Now it is an explicit `BufferTooShort`.
        // I2S packs 4 trits/byte: 1 byte holds at most 4 trits, so asking for 5 must refuse.
        assert_eq!(
            unpack_trits(&[0u8], PackScheme::I2S, 5),
            Err(PackError::BufferTooShort {
                count: 5,
                needed: 2,
                got: 1,
            })
        );
        // An empty buffer cannot supply even one trit.
        assert_eq!(
            unpack_trits(&[], PackScheme::Tl2, 1),
            Err(PackError::BufferTooShort {
                count: 1,
                needed: 1,
                got: 0,
            })
        );
        // The exact-fit boundary succeeds (no off-by-one refusal): 2 bytes hold 8 trits under I2S.
        assert!(unpack_trits(&[0u8, 0u8], PackScheme::I2S, 8).is_ok());
    }
}
