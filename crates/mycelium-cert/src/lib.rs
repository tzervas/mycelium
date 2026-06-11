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
//! the [`check`] module (M-210, E2-3): `check(A, B, R, claimed, evidence)` validates bijective
//! certificates by re-derivation equality, bounded certificates through the `mycelium-numerics`
//! tier-i checker (E2-4), and interp↔AOT observational equivalence (the M-151 differential) — one
//! checker, every instance, never a silent pass. The serialized certificate form is exactly
//! `docs/spec/schemas/swap-certificate.schema.json`.

pub mod check;
pub mod dense;
pub mod dense_vsa;

use serde::{Deserialize, Serialize};

use mycelium_core::{
    binary, operation_hash, ternary, Bound, ContentHash, GuaranteeStrength, Meta, Payload,
    Provenance, Repr, Value, WfError,
};
use mycelium_interp::{EvalError, SwapEngine};

pub use check::{check, CheckVerdict, Evidence, Fallback, NotValidatedReason, RefinementRelation};
pub use dense::{dense_f32_to_bf16, BF16_MIN_NORMAL, BF16_REL_EPS};
pub use dense_vsa::{dense_to_vsa, vsa_to_dense, DENSE_VSA_EMP_DELTA, DENSE_VSA_MODEL};

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
    /// A Dense source element is NaN/±Inf — rounding it has no defined error bound; explicit,
    /// never silent (M-211 acceptance; RFC-0002 §5).
    NonFinite {
        /// Index of the offending element.
        index: usize,
    },
    /// A Dense source element is not exactly an `f32` value although the repr declares
    /// `dtype: F32` — the payload contradicts its own representation; refused, never re-rounded.
    NotAnF32 {
        /// Index of the offending element.
        index: usize,
    },
    /// A Dense source element is subnormal — outside the checked side-conditions of the proven
    /// relative rounding bound (M-211 v1 scope); refused rather than tagged with a bound the
    /// theorem does not cover (VR-5/G2).
    SubnormalUnsupported {
        /// Index of the offending element.
        index: usize,
    },
    /// Rounding overflowed the target's finite range — explicit, never a silent ±Inf.
    RoundOverflow {
        /// Index of the offending element.
        index: usize,
    },
    /// The source value is itself approximate; composing its bound with the swap's ε is not yet a
    /// defined rule (E2-1 Dense numerics) — refused, never fabricated.
    ApproximateSource,
    /// A Dense↔VSA instance no basis covers: the proven capacity side-condition
    /// `vsa_dim ≥ requiredDim(components, δ)` fails *and* the trial-validated empirical profile
    /// does not reach it — a type error, not a `Declared` gamble (M-231; RFC-0002 §5).
    InsufficientCapacity {
        /// Dense components being encoded/decoded.
        components: u32,
        /// The hypervector dimension supplied.
        dim: u32,
        /// The dimension the cited theorem requires at the requested δ.
        required: u64,
    },
    /// A Dense component is not `±1` — the cited capacity theorem covers bundles of bipolar
    /// atoms only; a weighted-superposition bound is not in the corpus (M-231 v1 scope).
    NotBipolar {
        /// Index of the offending component.
        index: usize,
    },
    /// `vsa_to_dense` of a value that is not a `swap.dense_vsa.enc.v1` product — the δ describes
    /// retrieval from that encoding and nothing else (VR-5; provenance-gated).
    NotDenseVsaEncoding,
    /// A decode correlation vanished — the component's sign is undefined; explicit, never an
    /// arbitrary pick.
    AmbiguousDecode {
        /// Index of the undecodable component.
        index: usize,
    },
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
            SwapError::NonFinite { index } => {
                write!(f, "element {index} is NaN/Inf — no defined rounding bound")
            }
            SwapError::NotAnF32 { index } => {
                write!(
                    f,
                    "element {index} is not exactly an f32 value (payload contradicts dtype F32)"
                )
            }
            SwapError::SubnormalUnsupported { index } => {
                write!(
                    f,
                    "element {index} is subnormal — outside the proven relative-bound range (v1 scope)"
                )
            }
            SwapError::RoundOverflow { index } => {
                write!(
                    f,
                    "element {index} overflows the target's finite range on rounding"
                )
            }
            SwapError::ApproximateSource => {
                write!(
                    f,
                    "source is approximate; composing its bound with the swap ε is not a defined rule yet (E2-1)"
                )
            }
            SwapError::InsufficientCapacity {
                components,
                dim,
                required,
            } => write!(
                f,
                "no basis covers this Dense↔VSA instance: {components} components into dim {dim} \
                 (the theorem requires ≥ {required}; the empirical profile does not reach it)"
            ),
            SwapError::NotBipolar { index } => write!(
                f,
                "component {index} is not ±1 — the capacity theorem covers bipolar bundles only"
            ),
            SwapError::NotDenseVsaEncoding => write!(
                f,
                "source is not a swap.dense_vsa.enc.v1 product — its δ would describe nothing"
            ),
            SwapError::AmbiguousDecode { index } => write!(
                f,
                "component {index}'s decode correlation vanished — its sign is undefined"
            ),
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

/// The δ the engine requests for a Dense↔VSA swap when no policy channel supplies one — the same
/// target the M-131 capacity validation uses. A future selection-policy extension (RFC-0005) may
/// make it choosable; until then it is one documented constant, never an implicit per-call guess.
pub const DENSE_VSA_DEFAULT_DELTA: f64 = 1e-2;

/// A [`SwapEngine`] over the **complete certified swap surface** (SC-3 global, M-212): the
/// bijective binary↔ternary class (M-120), the bounded Dense `F32→BF16` class (M-211), the
/// bounded-probabilistic Dense↔VSA class (M-231, at [`DENSE_VSA_DEFAULT_DELTA`]), and
/// same-`Repr` identity. Every implemented legal-pair-table row goes through a
/// certificate-emitting function; everything else is an explicit error — never silent
/// (RFC-0002 §5: a pair with no statable bound is a type error).
#[derive(Debug, Clone, Copy, Default)]
pub struct CertifiedSwapEngine;

impl SwapEngine for CertifiedSwapEngine {
    fn swap(&self, src: &Value, target: &Repr, policy: &ContentHash) -> Result<Value, EvalError> {
        match (src.repr(), target) {
            (
                Repr::Dense {
                    dim: src_dim,
                    dtype: mycelium_core::ScalarKind::F32,
                },
                Repr::Dense {
                    dim: target_dim,
                    dtype: mycelium_core::ScalarKind::Bf16,
                },
            ) if src_dim == target_dim => Ok(dense::dense_f32_to_bf16(src, policy)?.0),
            (
                Repr::Dense {
                    dtype: mycelium_core::ScalarKind::F32,
                    ..
                },
                Repr::Vsa {
                    model,
                    dim,
                    sparsity: mycelium_core::SparsityClass::Dense,
                },
            ) if model == dense_vsa::DENSE_VSA_MODEL => {
                Ok(dense_vsa::dense_to_vsa(src, *dim, DENSE_VSA_DEFAULT_DELTA, policy)?.0)
            }
            (
                Repr::Vsa { model, .. },
                Repr::Dense {
                    dim,
                    dtype: mycelium_core::ScalarKind::F32,
                },
            ) if model == dense_vsa::DENSE_VSA_MODEL => {
                Ok(dense_vsa::vsa_to_dense(src, *dim, DENSE_VSA_DEFAULT_DELTA, policy)?.0)
            }
            _ => BinaryTernarySwapEngine.swap(src, target, policy),
        }
    }
}
