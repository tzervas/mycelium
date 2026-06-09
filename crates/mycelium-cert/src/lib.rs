//! `mycelium-cert` — swap certificates and the **binary↔ternary certified swap** (M-120;
//! RFC-0002 §3/§4; `docs/spec/swaps/binary-ternary.md`).
//!
//! A swap is **never silent** (SC-3): it yields a value in the target paradigm *and* an inspectable
//! [`SwapCertificate`] describing what the conversion cost. The binary↔ternary swap over a *legal*
//! `(n, m)` pair is the one genuinely **bijective/exact** class (`LosslessWithinRange`): it emits a
//! [`SwapCertificate::Bijective`] that references the once-per-`(n,m)` round-trip lemma (M-121,
//! `lemma_ref`) and binds it with concrete `params` — no per-value proof. The inverse `dec` is
//! **partial**: a ternary value outside the binary range is an explicit [`SwapError::OutOfRange`]
//! (P4), never a coerced wrap.
//!
//! The single, unified translation-validation certificate *checker* (shared with RFC-0004 §3) is
//! still **E2-3/E2-4** (Phase 2); this crate provides the certificate vocabulary and the first swap
//! that emits one. The serialized form is exactly `docs/spec/schemas/swap-certificate.schema.json`.

use serde::{Deserialize, Serialize};

use mycelium_core::{
    binary, operation_hash, ternary, Bound, ContentHash, GuaranteeStrength, Meta, Payload,
    Provenance, Repr, Value, WfError,
};
use mycelium_interp::{EvalError, SwapEngine};

/// Concrete parameters binding a bijection lemma to one use — `{ width, trits }` for binary↔ternary
/// (lets the certificate be cached by content hash; RFC-0002 §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinTernParams {
    /// Binary width `n`.
    pub width: u32,
    /// Ternary width `m`.
    pub trits: u32,
}

/// The inspectable certificate every swap produces (RFC-0002 §3/§5; `swap-certificate.schema.json`).
/// Tagged on `kind`; `src`/`target`/`policy_used` are common to both forms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SwapCertificate {
    /// Exact-within-range: references a once-per-swap-kind bijection lemma plus binding params.
    Bijective {
        /// Source representation.
        src: Repr,
        /// Target representation.
        target: Repr,
        /// The policy that selected/justified the swap.
        policy_used: ContentHash,
        /// Content hash of the round-trip/injectivity lemma (M-121).
        lemma_ref: ContentHash,
        /// Concrete parameters binding the lemma to this use.
        params: BinTernParams,
    },
    /// Lossy/bounded: carries a [`Bound`] (with its basis) and the policy used.
    Bounded {
        /// Source representation.
        src: Repr,
        /// Target representation.
        target: Repr,
        /// The policy that selected/justified the swap.
        policy_used: ContentHash,
        /// The error/probability bound and how it was obtained.
        bound: Bound,
    },
}

/// Why a swap could not be performed — always explicit (SC-3; G2), never a silent coercion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwapError {
    /// The source value is not in the expected source paradigm for this swap.
    WrongSource {
        /// What the engine expected (e.g. "Binary").
        expected: &'static str,
    },
    /// The `(width, trits)` pair is not legal for a lossless swap — `B_n ⊄ T_m` (RFC-0002 §5). A
    /// pair with no statable bound is a **type error**, not a `Declared` gamble.
    IllegalPair {
        /// Binary width.
        width: u32,
        /// Ternary width.
        trits: u32,
    },
    /// `dec` of a ternary value that lies outside the binary range — the partial-inverse path (P4).
    OutOfRange,
    /// A constructed result violated a Core IR invariant.
    Wf(WfError),
}

impl core::fmt::Display for SwapError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SwapError::WrongSource { expected } => write!(f, "expected a {expected} source value"),
            SwapError::IllegalPair { width, trits } => write!(
                f,
                "illegal pair: Binary{{{width}}} ⊄ Ternary{{{trits}}} (2^(n-1) > (3^m−1)/2)"
            ),
            SwapError::OutOfRange => {
                write!(
                    f,
                    "ternary value is outside the target binary range (dec = None)"
                )
            }
            SwapError::Wf(e) => write!(f, "well-formedness violation: {e}"),
        }
    }
}

impl std::error::Error for SwapError {}

impl From<SwapError> for EvalError {
    fn from(e: SwapError) -> Self {
        EvalError::Swap(e.to_string())
    }
}

/// Whether `(n, m)` admits a lossless binary→ternary swap: `B_n ⊆ T_m ⇔ 2^(n-1) ≤ (3^m − 1)/2`
/// (`binary-ternary.md` §2). Uses `i128` so the binary side never overflows the comparison.
#[must_use]
pub fn legal_pair(width: u32, trits: u32) -> bool {
    let Some(tern_max) = ternary::max_magnitude(trits) else {
        return false; // ternary side overflows i64 — far beyond any legal small pair
    };
    // 2^(n-1): the magnitude of the most-negative n-bit value, the binding constraint.
    let bin_max_neg_mag: i128 = 1i128 << width.saturating_sub(1);
    bin_max_neg_mag <= i128::from(tern_max)
}

/// The content hash of the once-per-swap-kind binary↔ternary round-trip lemma (P1/P2,
/// `binary-ternary.md` §4) — the `lemma_ref` every bijective certificate references. The M-121
/// machine-checked proof is published under this identity (`proofs/binary-ternary-roundtrip/`).
#[must_use]
pub fn roundtrip_lemma_ref() -> ContentHash {
    operation_hash("lemma.binary_ternary.roundtrip.v1")
}

fn swap_meta(src: &Value, policy: &ContentHash) -> Result<Meta, SwapError> {
    // Within range the swap is Exact / bound = None (P3, M-I1); it records the policy used (ADR-006)
    // and a Derived provenance over the source value (RFC-0001 §4.6).
    Meta::new(
        Provenance::Derived {
            op: operation_hash("swap.binary_ternary"),
            inputs: vec![src.content_hash()],
        },
        GuaranteeStrength::Exact,
        None,
        None,
        None,
        Some(policy.clone()),
    )
    .map_err(SwapError::Wf)
}

/// `enc`: encode an `n`-bit two's-complement [`Value`] into `m` balanced trits over a legal pair.
/// Total on `B_n` (RFC-0002 §4); returns the converted value and a `Bijective` certificate.
pub fn binary_to_ternary(
    src: &Value,
    trits_width: u32,
    policy: &ContentHash,
) -> Result<(Value, SwapCertificate), SwapError> {
    let Repr::Binary { width } = *src.repr() else {
        return Err(SwapError::WrongSource { expected: "Binary" });
    };
    let Payload::Bits(bits) = src.payload() else {
        return Err(SwapError::WrongSource { expected: "Binary" });
    };
    if !legal_pair(width, trits_width) {
        return Err(SwapError::IllegalPair {
            width,
            trits: trits_width,
        });
    }
    let value = binary::bits_to_int(bits);
    // Legal pair ⇒ B_n ⊆ T_m ⇒ encoding is total.
    let trits = ternary::int_to_trits(value, trits_width)
        .expect("legal pair guarantees the value fits in m trits");
    let target = Repr::Ternary { trits: trits_width };
    let out = Value::new(
        target.clone(),
        Payload::Trits(trits),
        swap_meta(src, policy)?,
    )
    .map_err(SwapError::Wf)?;
    let cert = SwapCertificate::Bijective {
        src: Repr::Binary { width },
        target,
        policy_used: policy.clone(),
        lemma_ref: roundtrip_lemma_ref(),
        params: BinTernParams {
            width,
            trits: trits_width,
        },
    };
    Ok((out, cert))
}

/// `dec`: decode `m` balanced trits back into an `n`-bit two's-complement [`Value`]. **Partial** —
/// a value outside `B_n` is [`SwapError::OutOfRange`] (P4, never silent). Returns the value and a
/// `Bijective` certificate on success.
pub fn ternary_to_binary(
    src: &Value,
    binary_width: u32,
    policy: &ContentHash,
) -> Result<(Value, SwapCertificate), SwapError> {
    let Repr::Ternary { trits } = *src.repr() else {
        return Err(SwapError::WrongSource {
            expected: "Ternary",
        });
    };
    let Payload::Trits(digits) = src.payload() else {
        return Err(SwapError::WrongSource {
            expected: "Ternary",
        });
    };
    if !legal_pair(binary_width, trits) {
        return Err(SwapError::IllegalPair {
            width: binary_width,
            trits,
        });
    }
    let value = ternary::trits_to_int(digits);
    let bits = binary::int_to_bits(value, binary_width).ok_or(SwapError::OutOfRange)?;
    let target = Repr::Binary {
        width: binary_width,
    };
    let out = Value::new(target.clone(), Payload::Bits(bits), swap_meta(src, policy)?)
        .map_err(SwapError::Wf)?;
    let cert = SwapCertificate::Bijective {
        src: Repr::Ternary { trits },
        target,
        policy_used: policy.clone(),
        lemma_ref: roundtrip_lemma_ref(),
        params: BinTernParams {
            width: binary_width,
            trits,
        },
    };
    Ok((out, cert))
}

/// A [`SwapEngine`](mycelium_interp::SwapEngine) for the reference interpreter that performs the
/// certified binary↔ternary swap (and same-`Repr` identity), refusing anything else explicitly. The
/// emitted certificate is available from the standalone [`binary_to_ternary`]/[`ternary_to_binary`]
/// functions; the interpreter result carries the honest `Meta` (Exact, `policy_used`, provenance).
#[derive(Debug, Clone, Copy, Default)]
pub struct BinaryTernarySwapEngine;

impl SwapEngine for BinaryTernarySwapEngine {
    fn swap(&self, src: &Value, target: &Repr, policy: &ContentHash) -> Result<Value, EvalError> {
        match (src.repr(), target) {
            (Repr::Binary { .. }, Repr::Ternary { trits }) => {
                Ok(binary_to_ternary(src, *trits, policy)?.0)
            }
            (Repr::Ternary { .. }, Repr::Binary { width }) => {
                Ok(ternary_to_binary(src, *width, policy)?.0)
            }
            (a, b) if a == b => {
                // Same representation → identity (the trivial engine's contract).
                mycelium_interp::IdentitySwapEngine.swap(src, target, policy)
            }
            (a, b) => Err(EvalError::UnsupportedSwap {
                from: a.clone(),
                to: b.clone(),
            }),
        }
    }
}
