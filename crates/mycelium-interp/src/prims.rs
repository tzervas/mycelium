//! Primitive operators for the interpreter (M-110).
//!
//! A [`Node::Op`](mycelium_core::Node::Op) names a primitive; the registry maps that name to an
//! implementation. The built-in set here is deliberately the **exact, elementwise** fragment — the
//! identity and per-element logical ops — so the reference interpreter is testable end-to-end
//! without prejudging **M-111** (balanced-ternary *arithmetic* — add/mul with carries, property-
//! tested against an integer oracle), which extends this registry.
//!
//! Every built-in is `guarantee = Exact` and type-checks its operands; a wrong arity, paradigm, or
//! width is an explicit [`EvalError::PrimType`], never a silent coercion (SC-3; G2). Result metadata
//! is threaded honestly: provenance becomes `Derived{ op: hash(prim), inputs: [hash(arg)…] }`
//! (RFC-0001 §4.6) and the guarantee is the `meet` of the inputs and the prim's intrinsic strength
//! (RFC-0001 §4.7). Exact inputs through an exact built-in stay `Exact` / `bound = None`.
//!
//! **Composing an approximate input (M-204; ADR-010).** With the verified-numerics kernels landed
//! (`mycelium-numerics`, E2-4), a built-in over an *approximate* input no longer refuses outright: it
//! composes the input's `Error` bound through the affine ε-kernel and meets the strength to the
//! weakest input ([`mycelium_numerics::compose_error_bound`]). Each prim declares its
//! `ApproxRule`: the additive ternary arithmetic (`trit.add/sub/neg`) carries a sound affine
//! composition; `core.id` passes the bound through unchanged; the logical `bit.*` ops and `trit.mul`
//! have **no defined ε-propagation rule over approximate inputs** and so still refuse
//! (`EvalError::ApproxCompositionUnsupported`) — refusing remains the honest choice over fabricating a
//! bound (G2/VR-5).

use std::collections::BTreeMap;

use mycelium_core::{
    operation_hash, ternary, Bound, GuaranteeStrength, Meta, Payload, Provenance, Repr, Trit, Value,
};
use mycelium_numerics::{compose_error_bound, ErrorOp};

use crate::EvalError;

/// How a built-in composes an *approximate* input's bound (M-204). Exact inputs never reach this —
/// they short-circuit to an `Exact`/`bound = None` result.
#[derive(Debug, Clone, Copy)]
enum ApproxRule {
    /// No defined ε-propagation over an approximate input — refuse (honest; `bit.*`, `trit.mul`).
    Refuse,
    /// Unary identity (`core.id`): pass the single input's bound and strength through unchanged.
    Passthrough,
    /// Compose the inputs' `Error` bounds through the affine ε-kernel under this op (the additive
    /// ternary arithmetic — sound 1-Lipschitz propagation).
    Error(ErrorOp),
}

/// A primitive implementation: a pure function from argument values to a result value (or an error).
pub type PrimFn = fn(prim: &str, args: &[&Value]) -> Result<Value, EvalError>;

/// The name→implementation table the interpreter dispatches `Op` nodes through. Extensible: M-111
/// (arithmetic) and later passes register additional prims here.
#[derive(Clone, Default)]
pub struct PrimRegistry {
    table: BTreeMap<String, PrimFn>,
}

impl PrimRegistry {
    /// An empty registry.
    #[must_use]
    pub fn empty() -> Self {
        PrimRegistry {
            table: BTreeMap::new(),
        }
    }

    /// The default registry: the exact built-ins — elementwise logical (`core.id`,
    /// `bit.not/and/or/xor`) and fixed-width balanced-ternary arithmetic (`trit.neg/add/sub/mul`,
    /// M-111).
    #[must_use]
    pub fn with_builtins() -> Self {
        let mut r = PrimRegistry::empty();
        r.register("core.id", prim_id);
        r.register("bit.not", prim_bit_not);
        r.register("bit.and", prim_bit_and);
        r.register("bit.or", prim_bit_or);
        r.register("bit.xor", prim_bit_xor);
        r.register("trit.neg", prim_trit_neg);
        r.register("trit.add", prim_trit_add);
        r.register("trit.sub", prim_trit_sub);
        r.register("trit.mul", prim_trit_mul);
        r
    }

    /// Register (or replace) a primitive.
    pub fn register(&mut self, name: &str, f: PrimFn) {
        self.table.insert(name.to_owned(), f);
    }

    /// Look up a primitive by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<PrimFn> {
        self.table.get(name).copied()
    }

    /// The registered primitive names (sorted).
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.table.keys().map(String::as_str).collect()
    }
}

/// Build a result value with honest provenance/guarantee threading (RFC-0001 §4.6/§4.7). The
/// intrinsic strength of every built-in is `Exact`, so the result strength is the `meet` over the
/// inputs. Exact inputs → an `Exact`/`bound = None` result; an approximate input is composed per the
/// prim's [`ApproxRule`] (M-204) — or explicitly refused when no rule applies (never a fabricated
/// bound; G2).
fn compose_result(
    prim: &str,
    inputs: &[&Value],
    repr: Repr,
    payload: Payload,
    rule: ApproxRule,
) -> Result<Value, EvalError> {
    let strength = GuaranteeStrength::propagate(
        GuaranteeStrength::Exact,
        inputs.iter().map(|v| v.meta().guarantee()),
    );
    let provenance = Provenance::Derived {
        op: operation_hash(prim),
        inputs: inputs.iter().map(|v| v.content_hash()).collect(),
    };
    let (guarantee, bound) = if strength == GuaranteeStrength::Exact {
        // All inputs exact ⇒ exact result, no bound (M-I1).
        (GuaranteeStrength::Exact, None)
    } else {
        compose_approx(prim, inputs, rule)?
    };
    // The `EvalError::Wf` arms below are *defensive*, not reachable from the public API with the
    // built-in prims (A4-04). `compose_approx` only ever pairs a non-`Exact` strength with a
    // present `Bound` (and `Exact` with `None`), so `Meta::new`'s M-I1 coupling check passes; and
    // every built-in produces a `payload` whose length matches the cloned input `repr`, so
    // `Value::new`'s payload↔repr check passes. They stay as explicit errors so that a *future*
    // prim (or a custom `PrimRegistry` registered via `Interpreter::new`) whose output is
    // internally inconsistent refuses honestly rather than panicking (G2 — never silent, never a
    // crash on constructed input).
    let meta = Meta::new(provenance, guarantee, bound, None, None, None).map_err(EvalError::Wf)?;
    Value::new(repr, payload, meta).map_err(EvalError::Wf)
}

/// Compose the bound + strength for a result over at least one *approximate* input (M-204; ADR-010).
/// The honest upgrade over the Phase-1 blanket refusal: a defined rule composes a *checked* bound; an
/// undefined one still refuses rather than guessing.
fn compose_approx(
    prim: &str,
    inputs: &[&Value],
    rule: ApproxRule,
) -> Result<(GuaranteeStrength, Option<Bound>), EvalError> {
    let refuse = || EvalError::ApproxCompositionUnsupported {
        prim: prim.to_owned(),
    };
    match rule {
        ApproxRule::Refuse => Err(refuse()),
        ApproxRule::Passthrough => {
            // Identity preserves the bound exactly (citation included) — clone it through.
            let v = inputs.first().ok_or_else(refuse)?;
            Ok((v.meta().guarantee(), v.meta().bound().cloned()))
        }
        ApproxRule::Error(op) => {
            // The non-exact inputs carry the Error bounds; exact inputs contribute the ε/strength
            // identity, so collecting only the present bounds is exactly the composition input set.
            let bounds: Vec<&Bound> = inputs.iter().filter_map(|v| v.meta().bound()).collect();
            let composed = compose_error_bound(&bounds, op).ok_or_else(refuse)?;
            Ok((composed.strength, Some(composed.bound)))
        }
    }
}

fn expect_arity(prim: &str, args: &[&Value], n: usize) -> Result<(), EvalError> {
    if args.len() == n {
        Ok(())
    } else {
        Err(EvalError::PrimType {
            prim: prim.to_owned(),
            why: format!("expected {n} argument(s), got {}", args.len()),
        })
    }
}

fn as_bits<'a>(prim: &str, v: &'a Value) -> Result<&'a [bool], EvalError> {
    match (v.repr(), v.payload()) {
        (Repr::Binary { .. }, Payload::Bits(b)) => Ok(b),
        _ => Err(EvalError::PrimType {
            prim: prim.to_owned(),
            why: "expected a Binary operand".to_owned(),
        }),
    }
}

// --- built-ins ---------------------------------------------------------------------------------

/// `core.id : a → a`. Identity (re-stamps provenance); useful as a no-op and a test fixture.
fn prim_id(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    expect_arity(prim, args, 1)?;
    let v = args[0];
    compose_result(
        prim,
        args,
        v.repr().clone(),
        v.payload().clone(),
        ApproxRule::Passthrough,
    )
}

/// `bit.not : Binary{n} → Binary{n}` — elementwise complement.
fn prim_bit_not(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    expect_arity(prim, args, 1)?;
    let bits = as_bits(prim, args[0])?;
    let out: Vec<bool> = bits.iter().map(|&b| !b).collect();
    compose_result(
        prim,
        args,
        args[0].repr().clone(),
        Payload::Bits(out),
        ApproxRule::Refuse,
    )
}

/// Shared elementwise binary-logical kernel for `bit.and/or/xor`.
fn bit_binop(prim: &str, args: &[&Value], op: fn(bool, bool) -> bool) -> Result<Value, EvalError> {
    expect_arity(prim, args, 2)?;
    let a = as_bits(prim, args[0])?;
    let b = as_bits(prim, args[1])?;
    if a.len() != b.len() {
        return Err(EvalError::PrimType {
            prim: prim.to_owned(),
            why: format!("width mismatch: {} vs {}", a.len(), b.len()),
        });
    }
    let out: Vec<bool> = a.iter().zip(b).map(|(&x, &y)| op(x, y)).collect();
    compose_result(
        prim,
        args,
        args[0].repr().clone(),
        Payload::Bits(out),
        ApproxRule::Refuse,
    )
}

fn prim_bit_and(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    bit_binop(prim, args, |x, y| x & y)
}
fn prim_bit_or(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    bit_binop(prim, args, |x, y| x | y)
}
fn prim_bit_xor(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    bit_binop(prim, args, |x, y| x ^ y)
}

fn as_trits<'a>(prim: &str, v: &'a Value) -> Result<&'a [Trit], EvalError> {
    match (v.repr(), v.payload()) {
        (Repr::Ternary { .. }, Payload::Trits(t)) => Ok(t),
        _ => Err(EvalError::PrimType {
            prim: prim.to_owned(),
            why: "expected a Ternary operand".to_owned(),
        }),
    }
}

/// `trit.neg : Ternary{m} → Ternary{m}` — digit-wise sign flip. Exactly `value(−t) = −value(t)`
/// (balanced ternary has no sign asymmetry; `docs/spec/swaps/binary-ternary.md` §1). Always in range.
fn prim_trit_neg(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    expect_arity(prim, args, 1)?;
    let out = ternary::neg(as_trits(prim, args[0])?);
    compose_result(
        prim,
        args,
        args[0].repr().clone(),
        Payload::Trits(out),
        ApproxRule::Error(ErrorOp::Neg),
    )
}

/// Shared kernel for the fixed-width balanced-ternary binary arithmetic prims (`trit.add/sub/mul`).
/// Operands must be equal-width `Ternary`; an out-of-range result is an explicit
/// [`EvalError::Overflow`], never a silent wrap (M-111; `binary-ternary.md` §1).
fn trit_binop(
    prim: &str,
    args: &[&Value],
    op: fn(&[Trit], &[Trit]) -> Option<Vec<Trit>>,
    rule: ApproxRule,
) -> Result<Value, EvalError> {
    expect_arity(prim, args, 2)?;
    let a = as_trits(prim, args[0])?;
    let b = as_trits(prim, args[1])?;
    if a.len() != b.len() {
        return Err(EvalError::PrimType {
            prim: prim.to_owned(),
            why: format!("width mismatch: {} vs {} trits", a.len(), b.len()),
        });
    }
    let out = op(a, b).ok_or_else(|| EvalError::Overflow {
        prim: prim.to_owned(),
    })?;
    compose_result(
        prim,
        args,
        args[0].repr().clone(),
        Payload::Trits(out),
        rule,
    )
}

/// `trit.add`: balanced-ternary addition is exact on the values, so an approximate input's ε
/// propagates additively (1-Lipschitz; affine `Add`) — sound (M-204).
fn prim_trit_add(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    trit_binop(prim, args, ternary::add, ApproxRule::Error(ErrorOp::Add))
}
/// `trit.sub`: same additive ε propagation as `trit.add` (affine `Sub`).
fn prim_trit_sub(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    trit_binop(prim, args, ternary::sub, ApproxRule::Error(ErrorOp::Sub))
}
/// `trit.mul`: multiplicative ε propagation needs the central operand magnitudes (affine `Mul`); that
/// plumbing lands with the Dense numerics (E2-1), so an approximate input is refused for now — honest,
/// not a fabricated bound (G2).
fn prim_trit_mul(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    trit_binop(prim, args, ternary::mul, ApproxRule::Refuse)
}

#[cfg(test)]
mod mutant_witness_tests {
    //! Mutant-witness tests for prims.rs survivors (M-654 Gate A3).
    use super::*;
    use mycelium_core::{Meta, Payload, Provenance, Repr, Value};

    fn byte(bits: [bool; 8]) -> Value {
        Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(bits.to_vec()),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    // ---- prims.rs:61 — PrimRegistry::empty → Default::default() ----
    // JUSTIFIED: PrimRegistry derives Default (BTreeMap::new()), and `empty()` also constructs
    // BTreeMap::new(). The two are semantically identical — both produce an empty registry with no
    // registered prims. This mutant is genuinely equivalent and is excluded via mutants.toml.

    // ---- prims.rs:169 — expect_arity → Ok(()) ----
    // Mutant: expect_arity always succeeds, even with wrong arity — arity errors are never raised.
    // Kill: invoking a prim with wrong arity must return a PrimType error, not succeed silently.
    #[test]
    fn expect_arity_rejects_wrong_arity() {
        // Mutant-witness: prims.rs:169 replace expect_arity → Ok(()).
        // bit.not requires exactly 1 arg; providing 0 or 2 must be a PrimType error.
        // Test via the PrimRegistry public API.
        let reg = PrimRegistry::with_builtins();
        let f = reg.get("bit.not").expect("bit.not registered");
        let b = byte([true; 8]);
        // Zero args → PrimType.
        assert!(
            matches!(f("bit.not", &[]), Err(EvalError::PrimType { .. })),
            "bit.not with 0 args must be PrimType"
        );
        // Two args → PrimType.
        assert!(
            matches!(f("bit.not", &[&b, &b]), Err(EvalError::PrimType { .. })),
            "bit.not with 2 args must be PrimType"
        );
        // One arg → Ok (correct arity).
        assert!(
            f("bit.not", &[&b]).is_ok(),
            "bit.not with 1 arg must succeed"
        );
    }

    // ---- prims.rs:240 — prim_bit_and: & → | or ^ ----
    // Mutant A (& → |): AND is replaced by OR — (1&0)=0 but (1|0)=1.
    // Mutant B (& → ^): AND is replaced by XOR — (1&1)=1 but (1^1)=0.
    // Kill: test a case where AND, OR, and XOR all differ (e.g. a=1,b=0 and a=1,b=1).
    #[test]
    fn bit_and_is_conjunction_not_disjunction_or_xor() {
        // Mutant-witness: prims.rs:240 & → | or ^.
        let reg = PrimRegistry::with_builtins();
        let f = reg.get("bit.and").expect("bit.and registered");

        // Operands: a = [true; 8], b = [false; 8].
        // AND: all false. OR: all true. XOR: all true. AND ≠ OR,XOR.
        let a = byte([true; 8]);
        let b_zeros = byte([false; 8]);
        let result = f("bit.and", &[&a, &b_zeros]).expect("bit.and evaluates");
        assert_eq!(
            result.payload(),
            &Payload::Bits(vec![false; 8]),
            "bit.and([1;8], [0;8]) must be [0;8] (AND), not [1;8] (OR/XOR)"
        );

        // Operands: a = [true; 8], b = [true; 8].
        // AND: all true. OR: all true. XOR: all false. AND ≠ XOR here.
        let b_ones = byte([true; 8]);
        let result2 = f("bit.and", &[&a, &b_ones]).expect("bit.and evaluates");
        assert_eq!(
            result2.payload(),
            &Payload::Bits(vec![true; 8]),
            "bit.and([1;8], [1;8]) must be [1;8] (AND/OR), distinguishing from XOR ([0;8])"
        );
    }

    // ---- prims.rs:243 — prim_bit_or: | → & or ^ ----
    // Mutant A (| → &): OR is replaced by AND — (1|0)=1 but (1&0)=0.
    // Mutant B (| → ^): OR is replaced by XOR — (1|1)=1 but (1^1)=0.
    // Kill: test case where OR, AND, XOR all differ.
    #[test]
    fn bit_or_is_disjunction_not_conjunction_or_xor() {
        // Mutant-witness: prims.rs:243 | → & or ^.
        let reg = PrimRegistry::with_builtins();
        let f = reg.get("bit.or").expect("bit.or registered");

        // Operands: a = [true; 8], b = [false; 8].
        // OR: all true. AND: all false. XOR: all true. OR ≠ AND.
        let a = byte([true; 8]);
        let b_zeros = byte([false; 8]);
        let result = f("bit.or", &[&a, &b_zeros]).expect("bit.or evaluates");
        assert_eq!(
            result.payload(),
            &Payload::Bits(vec![true; 8]),
            "bit.or([1;8], [0;8]) must be [1;8] (OR), not [0;8] (AND)"
        );

        // Operands: a = [true; 8], b = [true; 8].
        // OR: all true. AND: all true. XOR: all false. OR ≠ XOR here.
        let b_ones = byte([true; 8]);
        let result2 = f("bit.or", &[&a, &b_ones]).expect("bit.or evaluates");
        assert_eq!(
            result2.payload(),
            &Payload::Bits(vec![true; 8]),
            "bit.or([1;8], [1;8]) must be [1;8] (OR/AND), distinguishing from XOR ([0;8])"
        );

        // Mixed: a=[T,F,T,F,T,F,T,F], b=[F,F,F,F,F,F,F,F].
        // OR=[T,F,T,F,T,F,T,F], AND=[F;8], XOR=[T,F,T,F,T,F,T,F] — OR and XOR agree here.
        // But the two tests above already distinguish OR from both AND and XOR.
    }
}
