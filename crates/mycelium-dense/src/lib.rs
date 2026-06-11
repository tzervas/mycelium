//! `mycelium-dense` — the **Dense paradigm operational surface** (M-230; RFC-0001 §4.1;
//! RFC-0002 §5): typed, dimension-tracked `Dense{dim, dtype}` values and elementwise/embedding
//! ops — the Dense analogue of the VSA `VsaModel` surface.
//!
//! Dimension and dtype are part of the type ([`DenseSpace`] binds both); a mismatch is a typed
//! error, never a silent broadcast or coercion (G2). Per the honesty rule, every op carries an
//! honest per-op tag ([`DenseSpace::op_guarantee`]):
//!
//! - **`neg`** is **`Exact`** — negation never rounds (the dtype grids are symmetric).
//! - **`add`/`sub`/`scale`** are **`Proven`**, carrying a per-element relative ε
//!   ([`Bound`] `Error{eps, Rel}`) with a `ProvenThm` basis: the standard round-to-nearest
//!   relative-error theorem (Higham 2002, Thm 2.2 — the same basis as the M-211 `F32→BF16`
//!   swap), with its side-conditions **checked per element** (exact on-grid inputs; finite,
//!   zero-or-normal, non-overflowing results). A violated side-condition is an explicit
//!   [`DenseError`], never a bound the theorem does not cover (VR-5).
//! - **`dot`/`similarity`** are *measurement helpers* returning bare `f64` (no `Meta` to tag),
//!   mirroring `VsaModel::similarity`.
//!
//! **Honest scope (v1, same as M-211).** Sources must be `Exact`: composing an approximate
//! input's own bound with the op's rounding ε needs the magnitude-aware Dense composition rule
//! that is still open (recorded at M-204/M-211) — refused explicitly via
//! [`DenseError::ApproximateSource`], never fabricated. `F16`/`F64` dtypes are explicitly
//! unsupported (`F64` ops cannot be re-derived against an exact reference in `f64`; `F16` lands
//! with a use case), and subnormal results are refused (outside the cited theorem's
//! side-conditions).

use mycelium_core::{
    operation_hash, Bound, BoundBasis, BoundKind, ContentHash, GuaranteeStrength, Meta, NormKind,
    Payload, Provenance, Repr, ScalarKind, Value, WfError,
};

/// Single-rounding relative bound for native `f32` ops: the unit roundoff `u = β^(1−p)/2 = 2^−24`
/// for IEEE binary32 (`p = 24`) under round-to-nearest (Higham 2002, Thm 2.2).
pub const F32_OP_REL_EPS: f64 = 5.960_464_477_539_063e-8; // 2⁻²⁴, exact in f64

/// Two-rounding relative bound for BF16 ops: the op is computed as a native `f32` op
/// (`u₁ = 2^−24`) and rounded to the bfloat16 grid (`u₂ = 2^−8`); the composition
/// `(1+δ₁)(1+δ₂) − 1 ≤ u₁ + u₂ + u₁u₂ ≤ 2^−8 + 2^−23` (the slack absorbs the cross term).
pub const BF16_OP_REL_EPS: f64 = 0.003_906_25 + 1.192_092_895_507_812_5e-7; // 2⁻⁸ + 2⁻²³

/// Smallest positive *normal* magnitude on both the `f32` and bfloat16 grids (`2^−126` — bf16
/// keeps f32's exponent range). Below it the relative-error theorem's side-condition fails.
pub const DENSE_MIN_NORMAL: f64 = f32::MIN_POSITIVE as f64;

const F32_OP_CITATION: &str = "round-to-nearest relative error ≤ u = β^(1−p)/2 = 2^−24 for IEEE \
     binary32 (β=2, p=24) — Higham, Accuracy and Stability of Numerical Algorithms (2002), Thm 2.2; \
     native f32 op (single rounding); side-conditions checked per element: exact on-grid inputs, \
     finite zero-or-normal result, no overflow";

const BF16_OP_CITATION: &str = "two-rounding composition (1+δ₁)(1+δ₂)−1 ≤ 2^−8 + 2^−23: native f32 \
     op (u₁ = 2^−24) then bfloat16 round-to-nearest (u₂ = 2^−8) — Higham (2002), Thm 2.2 applied at \
     each rounding; side-conditions checked per element at both steps: exact on-grid inputs, finite \
     zero-or-normal results, no overflow";

/// The Dense operations this surface supplies (RFC-0001 §4.1 — the Dense analogue of
/// [`VsaOp`](https://docs.rs/mycelium-vsa)'s closed op set).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenseOp {
    /// Elementwise addition.
    Add,
    /// Elementwise subtraction.
    Sub,
    /// Elementwise negation.
    Neg,
    /// Scalar multiplication.
    Scale,
}

/// Why a Dense operation could not be performed — always explicit, never a silent coercion (G2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DenseError {
    /// Operand dimensionality disagrees with the space's `dim` — a type error (M-230 acceptance).
    DimMismatch {
        /// Expected dimensionality.
        expected: u32,
        /// Actual dimensionality.
        got: u32,
    },
    /// Operand dtype disagrees with the space's `dtype` — a type error, never re-rounded.
    DtypeMismatch {
        /// The dtype the space expected.
        expected: ScalarKind,
    },
    /// The dtype has no supported op set in v1 (`F16`/`F64` — honest scope, see crate docs).
    UnsupportedDtype {
        /// The unsupported dtype.
        dtype: ScalarKind,
    },
    /// The value handed in is not a `Dense` value at all.
    NotDense,
    /// An element is NaN/±Inf — no rounding bound is defined for it.
    NonFinite {
        /// Index of the offending element.
        index: usize,
    },
    /// An element is not exactly representable on the declared dtype grid — the payload
    /// contradicts its own representation; refused, never silently re-rounded.
    NotOnGrid {
        /// Index of the offending element.
        index: usize,
    },
    /// The scale factor is non-finite or off the dtype grid (same contract as the elements).
    ScalarOffGrid,
    /// A result element is subnormal — outside the cited theorem's side-conditions (v1 scope,
    /// same honest refusal as M-211).
    SubnormalUnsupported {
        /// Index of the offending element.
        index: usize,
    },
    /// A result element overflows the dtype's finite range — explicit, never a silent ±Inf.
    Overflow {
        /// Index of the offending element.
        index: usize,
    },
    /// The source value is itself approximate; composing its bound with the op's rounding ε is
    /// not a defined rule yet (recorded at M-204/M-211) — refused, never fabricated.
    ApproximateSource,
    /// A constructed result violated a Core IR invariant.
    Wf(WfError),
}

impl core::fmt::Display for DenseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DenseError::DimMismatch { expected, got } => {
                write!(f, "dimension mismatch: expected {expected}, got {got}")
            }
            DenseError::DtypeMismatch { expected } => {
                write!(f, "dtype mismatch: expected {expected:?}")
            }
            DenseError::UnsupportedDtype { dtype } => {
                write!(
                    f,
                    "dtype {dtype:?} has no supported Dense op set (M-230 v1 scope)"
                )
            }
            DenseError::NotDense => write!(f, "expected a Dense value"),
            DenseError::NonFinite { index } => {
                write!(f, "element {index} is NaN/Inf — no defined rounding bound")
            }
            DenseError::NotOnGrid { index } => {
                write!(f, "element {index} is not on the declared dtype grid")
            }
            DenseError::ScalarOffGrid => {
                write!(f, "scale factor is non-finite or off the dtype grid")
            }
            DenseError::SubnormalUnsupported { index } => write!(
                f,
                "result element {index} is subnormal — outside the proven relative-bound range"
            ),
            DenseError::Overflow { index } => {
                write!(
                    f,
                    "result element {index} overflows the dtype's finite range"
                )
            }
            DenseError::ApproximateSource => write!(
                f,
                "source is approximate; composing its bound with the op ε is not a defined rule yet"
            ),
            DenseError::Wf(e) => write!(f, "well-formedness violation: {e}"),
        }
    }
}

impl std::error::Error for DenseError {}

/// Round an `f32` to the nearest bfloat16 (ties to even), widened back to `f32` bit-exactly —
/// the same grid the M-211 swap targets. Caller has excluded NaN/Inf.
fn round_f32_to_bf16(x: f32) -> f32 {
    let bits = x.to_bits();
    let lsb = (bits >> 16) & 1;
    f32::from_bits(((bits + 0x7FFF + lsb) >> 16) << 16)
}

/// Whether `x` is exactly representable on the `dtype` grid (finite values only).
fn on_grid(dtype: ScalarKind, x: f64) -> bool {
    #[allow(clippy::cast_possible_truncation)] // representability is exactly what we check
    let xf = x as f32;
    if f64::from(xf) != x {
        return false;
    }
    match dtype {
        ScalarKind::F32 => true,
        ScalarKind::Bf16 => round_f32_to_bf16(xf) == xf,
        // Unreachable behind DenseSpace::new's dtype gate; conservatively off-grid.
        ScalarKind::F16 | ScalarKind::F64 => false,
    }
}

/// A typed Dense space: every value it constructs or operates on has exactly this `dim` and
/// `dtype` (dim-in-the-type, M-230 acceptance).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DenseSpace {
    /// Dimensionality.
    pub dim: u32,
    /// Element dtype (`F32` or `BF16` in v1).
    pub dtype: ScalarKind,
}

impl DenseSpace {
    /// A Dense space of `dim`-vectors over `dtype`. `F16`/`F64` are an explicit
    /// [`DenseError::UnsupportedDtype`] (v1 scope — see crate docs).
    pub fn new(dim: u32, dtype: ScalarKind) -> Result<Self, DenseError> {
        match dtype {
            ScalarKind::F32 | ScalarKind::Bf16 => Ok(DenseSpace { dim, dtype }),
            ScalarKind::F16 | ScalarKind::F64 => Err(DenseError::UnsupportedDtype { dtype }),
        }
    }

    /// The `Repr` of this space's values.
    #[must_use]
    pub fn repr(&self) -> Repr {
        Repr::Dense {
            dim: self.dim,
            dtype: self.dtype,
        }
    }

    /// The honest intrinsic guarantee per op: `neg` never rounds (`Exact`); `add`/`sub`/`scale`
    /// round once (or twice for BF16) under the cited theorem (`Proven`).
    #[must_use]
    pub fn op_guarantee(op: DenseOp) -> GuaranteeStrength {
        match op {
            DenseOp::Neg => GuaranteeStrength::Exact,
            DenseOp::Add | DenseOp::Sub | DenseOp::Scale => GuaranteeStrength::Proven,
        }
    }

    /// The per-element relative ε this space's rounding ops carry.
    #[must_use]
    pub fn op_rel_eps(&self) -> f64 {
        match self.dtype {
            ScalarKind::Bf16 => BF16_OP_REL_EPS,
            // `new` admits only F32 | Bf16; F32 is the remaining case.
            _ => F32_OP_REL_EPS,
        }
    }

    fn op_citation(&self) -> &'static str {
        match self.dtype {
            ScalarKind::Bf16 => BF16_OP_CITATION,
            _ => F32_OP_CITATION,
        }
    }

    /// Construct an **`Exact`** Dense value, checking every element is finite and exactly on the
    /// dtype grid — an off-grid payload would contradict its own `Repr` (refused, never
    /// re-rounded).
    pub fn value(&self, xs: Vec<f64>) -> Result<Value, DenseError> {
        self.check_elements(&xs)?;
        Value::new(
            self.repr(),
            Payload::Scalars(xs),
            Meta::exact(Provenance::Root),
        )
        .map_err(DenseError::Wf)
    }

    fn check_elements(&self, xs: &[f64]) -> Result<(), DenseError> {
        if xs.len() != self.dim as usize {
            return Err(DenseError::DimMismatch {
                expected: self.dim,
                got: u32::try_from(xs.len()).unwrap_or(u32::MAX),
            });
        }
        for (index, &x) in xs.iter().enumerate() {
            if !x.is_finite() {
                return Err(DenseError::NonFinite { index });
            }
            if !on_grid(self.dtype, x) {
                return Err(DenseError::NotOnGrid { index });
            }
        }
        Ok(())
    }

    /// Extract the scalar payload of a value belonging to this space, re-checking the contract
    /// the ops rely on (right space, `Exact` source, on-grid elements).
    fn scalars_of<'a>(&self, v: &'a Value) -> Result<&'a [f64], DenseError> {
        let Repr::Dense { dim, dtype } = *v.repr() else {
            return Err(DenseError::NotDense);
        };
        if dtype != self.dtype {
            return Err(DenseError::DtypeMismatch {
                expected: self.dtype,
            });
        }
        if dim != self.dim {
            return Err(DenseError::DimMismatch {
                expected: self.dim,
                got: dim,
            });
        }
        if v.meta().guarantee() != GuaranteeStrength::Exact {
            return Err(DenseError::ApproximateSource);
        }
        let Payload::Scalars(xs) = v.payload() else {
            return Err(DenseError::NotDense);
        };
        self.check_elements(xs)?;
        Ok(xs)
    }

    /// One elementwise result under the theorem's checked side-conditions: compute as a native
    /// `f32` op, round to the dtype grid, and refuse (explicitly) any element the cited bound
    /// does not cover.
    fn round_result(&self, y32: f32, index: usize) -> Result<f64, DenseError> {
        if !y32.is_finite() {
            return Err(DenseError::Overflow { index });
        }
        if y32 != 0.0 && f64::from(y32.abs()) < DENSE_MIN_NORMAL {
            return Err(DenseError::SubnormalUnsupported { index });
        }
        let rounded = match self.dtype {
            ScalarKind::Bf16 => {
                let r = round_f32_to_bf16(y32);
                if !r.is_finite() {
                    return Err(DenseError::Overflow { index });
                }
                if r != 0.0 && f64::from(r.abs()) < DENSE_MIN_NORMAL {
                    return Err(DenseError::SubnormalUnsupported { index });
                }
                r
            }
            _ => y32,
        };
        Ok(f64::from(rounded))
    }

    /// Wrap a rounded result with its honest `Proven` rounding bound (M-I2: the basis is the
    /// checked theorem instantiation, never an assertion).
    fn wrap_proven(
        &self,
        data: Vec<f64>,
        op: &str,
        inputs: Vec<ContentHash>,
    ) -> Result<Value, DenseError> {
        let bound = Bound {
            kind: BoundKind::Error {
                eps: self.op_rel_eps(),
                norm: NormKind::Rel,
            },
            basis: BoundBasis::ProvenThm {
                citation: self.op_citation().to_owned(),
            },
        };
        let meta = Meta::new(
            Provenance::Derived {
                op: operation_hash(op),
                inputs,
            },
            GuaranteeStrength::Proven,
            Some(bound),
            None,
            None,
            None,
        )
        .map_err(DenseError::Wf)?;
        Value::new(self.repr(), Payload::Scalars(data), meta).map_err(DenseError::Wf)
    }

    /// Elementwise `a + b` (**`Proven`**, per-element relative ε — see crate docs).
    pub fn add_values(&self, a: &Value, b: &Value) -> Result<Value, DenseError> {
        self.elementwise(a, b, "dense.add", |x, y| x + y)
    }

    /// Elementwise `a − b` (**`Proven`**, same bound as `add`).
    pub fn sub_values(&self, a: &Value, b: &Value) -> Result<Value, DenseError> {
        self.elementwise(a, b, "dense.sub", |x, y| x - y)
    }

    fn elementwise(
        &self,
        a: &Value,
        b: &Value,
        op: &str,
        f: impl Fn(f32, f32) -> f32,
    ) -> Result<Value, DenseError> {
        let xs = self.scalars_of(a)?;
        let ys = self.scalars_of(b)?;
        let mut out = Vec::with_capacity(xs.len());
        for (index, (&x, &y)) in xs.iter().zip(ys).enumerate() {
            // On-grid checked above, so the f32 narrowing is exact.
            #[allow(clippy::cast_possible_truncation)]
            let y32 = f(x as f32, y as f32);
            out.push(self.round_result(y32, index)?);
        }
        self.wrap_proven(out, op, vec![a.content_hash(), b.content_hash()])
    }

    /// Elementwise negation (**`Exact`** — the grids are symmetric, so no element ever rounds).
    pub fn neg_value(&self, a: &Value) -> Result<Value, DenseError> {
        let xs = self.scalars_of(a)?;
        let out: Vec<f64> = xs.iter().map(|&x| -x).collect();
        let meta = Meta::new(
            Provenance::Derived {
                op: operation_hash("dense.neg"),
                inputs: vec![a.content_hash()],
            },
            GuaranteeStrength::Exact,
            None,
            None,
            None,
            None,
        )
        .map_err(DenseError::Wf)?;
        Value::new(self.repr(), Payload::Scalars(out), meta).map_err(DenseError::Wf)
    }

    /// Scalar multiplication `c · a` (**`Proven`**). `c` must be finite and on the dtype grid —
    /// the same contract as the elements (else [`DenseError::ScalarOffGrid`]).
    pub fn scale_value(&self, a: &Value, c: f64) -> Result<Value, DenseError> {
        if !c.is_finite() || !on_grid(self.dtype, c) {
            return Err(DenseError::ScalarOffGrid);
        }
        let xs = self.scalars_of(a)?;
        let mut out = Vec::with_capacity(xs.len());
        for (index, &x) in xs.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)] // on-grid checked: narrowing is exact
            let y32 = (c as f32) * (x as f32);
            out.push(self.round_result(y32, index)?);
        }
        self.wrap_proven(out, "dense.scale", vec![a.content_hash()])
    }

    /// Dot product in `f64` — a *measurement* helper (no `Meta` to tag), mirroring
    /// `VsaModel::similarity`. Typed errors for space mismatches.
    pub fn dot(&self, a: &Value, b: &Value) -> Result<f64, DenseError> {
        let xs = self.scalars_of(a)?;
        let ys = self.scalars_of(b)?;
        Ok(xs.iter().zip(ys).map(|(x, y)| x * y).sum())
    }

    /// Cosine similarity in `[-1, 1]` (`0` if either operand has zero norm) — a measurement
    /// helper in `f64`.
    pub fn similarity(&self, a: &Value, b: &Value) -> Result<f64, DenseError> {
        let xs = self.scalars_of(a)?;
        let ys = self.scalars_of(b)?;
        let dot: f64 = xs.iter().zip(ys.iter()).map(|(x, y)| x * y).sum();
        let na: f64 = xs.iter().map(|x| x * x).sum::<f64>().sqrt();
        let nb: f64 = ys.iter().map(|x| x * x).sum::<f64>().sqrt();
        if na == 0.0 || nb == 0.0 {
            Ok(0.0)
        } else {
            Ok(dot / (na * nb))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_dtypes_are_explicit() {
        assert_eq!(
            DenseSpace::new(4, ScalarKind::F64),
            Err(DenseError::UnsupportedDtype {
                dtype: ScalarKind::F64
            })
        );
        assert_eq!(
            DenseSpace::new(4, ScalarKind::F16),
            Err(DenseError::UnsupportedDtype {
                dtype: ScalarKind::F16
            })
        );
    }

    #[test]
    fn construction_checks_the_grid() {
        let s = DenseSpace::new(2, ScalarKind::F32).unwrap();
        assert!(s.value(vec![1.5, -0.625]).is_ok());
        // 0.1 is not exactly an f32.
        assert_eq!(
            s.value(vec![1.0, 0.1]),
            Err(DenseError::NotOnGrid { index: 1 })
        );
        assert_eq!(
            s.value(vec![f64::NAN, 0.0]),
            Err(DenseError::NonFinite { index: 0 })
        );
        let b = DenseSpace::new(2, ScalarKind::Bf16).unwrap();
        // 1.5 is on the bf16 grid; 1.501953125 (1.5 + 2^-9) is f32-exact but off the bf16 grid.
        assert!(b.value(vec![1.5, -2.0]).is_ok());
        assert_eq!(
            b.value(vec![1.5, 1.501_953_125]),
            Err(DenseError::NotOnGrid { index: 1 })
        );
    }

    #[test]
    fn neg_is_exact() {
        let s = DenseSpace::new(3, ScalarKind::F32).unwrap();
        let a = s.value(vec![1.5, -0.625, 0.0]).unwrap();
        let n = s.neg_value(&a).unwrap();
        assert_eq!(n.meta().guarantee(), GuaranteeStrength::Exact);
        assert_eq!(n.payload(), &Payload::Scalars(vec![-1.5, 0.625, 0.0]));
        assert_eq!(
            DenseSpace::op_guarantee(DenseOp::Neg),
            GuaranteeStrength::Exact
        );
    }

    #[test]
    fn add_carries_the_proven_rounding_bound() {
        let s = DenseSpace::new(2, ScalarKind::F32).unwrap();
        let a = s.value(vec![1.5, 2.5]).unwrap();
        let b = s.value(vec![0.25, -1.0]).unwrap();
        let y = s.add_values(&a, &b).unwrap();
        assert_eq!(y.meta().guarantee(), GuaranteeStrength::Proven);
        match y.meta().bound() {
            Some(Bound {
                kind: BoundKind::Error { eps, norm },
                basis: BoundBasis::ProvenThm { .. },
            }) => {
                assert_eq!(*eps, F32_OP_REL_EPS);
                assert_eq!(*norm, NormKind::Rel);
            }
            other => panic!("expected a ProvenThm Error bound, got {other:?}"),
        }
        match y.meta().provenance() {
            Provenance::Derived { op, inputs } => {
                assert_eq!(op, &operation_hash("dense.add"));
                assert_eq!(inputs, &vec![a.content_hash(), b.content_hash()]);
            }
            other => panic!("expected Derived, got {other:?}"),
        }
    }

    #[test]
    fn mismatches_and_approximate_sources_are_typed_errors() {
        let s = DenseSpace::new(2, ScalarKind::F32).unwrap();
        let a = s.value(vec![1.0, 2.0]).unwrap();
        let wrong_dim = DenseSpace::new(3, ScalarKind::F32)
            .unwrap()
            .value(vec![1.0, 2.0, 3.0])
            .unwrap();
        assert_eq!(
            s.add_values(&a, &wrong_dim),
            Err(DenseError::DimMismatch {
                expected: 2,
                got: 3
            })
        );
        let wrong_dtype = DenseSpace::new(2, ScalarKind::Bf16)
            .unwrap()
            .value(vec![1.0, 2.0])
            .unwrap();
        assert_eq!(
            s.add_values(&a, &wrong_dtype),
            Err(DenseError::DtypeMismatch {
                expected: ScalarKind::F32
            })
        );
        // An approximate source is refused (no composition rule yet) — built via the M-204-style
        // derived bound to simulate one.
        let approx = Value::new(
            s.repr(),
            Payload::Scalars(vec![1.0, 2.0]),
            Meta::new(
                Provenance::Root,
                GuaranteeStrength::Declared,
                Some(Bound {
                    kind: BoundKind::Error {
                        eps: 0.1,
                        norm: NormKind::Rel,
                    },
                    basis: BoundBasis::UserDeclared,
                }),
                None,
                None,
                None,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            s.add_values(&a, &approx),
            Err(DenseError::ApproximateSource)
        );
    }

    #[test]
    fn overflow_and_subnormal_results_are_explicit() {
        let s = DenseSpace::new(1, ScalarKind::F32).unwrap();
        let max = s.value(vec![f64::from(f32::MAX)]).unwrap();
        assert_eq!(
            s.add_values(&max, &max),
            Err(DenseError::Overflow { index: 0 })
        );
        // 1.5·2⁻¹²⁶ − 1.25·2⁻¹²⁶ = 0.25·2⁻¹²⁶: subnormal, refused.
        let a = s.value(vec![1.5 * DENSE_MIN_NORMAL]).unwrap();
        let b = s.value(vec![1.25 * DENSE_MIN_NORMAL]).unwrap();
        assert_eq!(
            s.sub_values(&a, &b),
            Err(DenseError::SubnormalUnsupported { index: 0 })
        );
    }

    #[test]
    fn scale_checks_the_factor() {
        let s = DenseSpace::new(2, ScalarKind::Bf16).unwrap();
        let a = s.value(vec![1.5, -2.0]).unwrap();
        assert_eq!(s.scale_value(&a, 0.1), Err(DenseError::ScalarOffGrid));
        let y = s.scale_value(&a, 2.0).unwrap();
        assert_eq!(y.payload(), &Payload::Scalars(vec![3.0, -4.0]));
        assert_eq!(y.meta().guarantee(), GuaranteeStrength::Proven);
    }

    #[test]
    fn similarity_is_a_measurement_helper() {
        let s = DenseSpace::new(2, ScalarKind::F32).unwrap();
        let a = s.value(vec![1.0, 0.0]).unwrap();
        let b = s.value(vec![0.0, 1.0]).unwrap();
        assert!((s.similarity(&a, &b).unwrap()).abs() < 1e-12);
        assert!((s.similarity(&a, &a).unwrap() - 1.0).abs() < 1e-12);
        let z = s.value(vec![0.0, 0.0]).unwrap();
        assert_eq!(s.similarity(&a, &z).unwrap(), 0.0);
        assert_eq!(s.dot(&a, &b).unwrap(), 0.0);
    }
}
