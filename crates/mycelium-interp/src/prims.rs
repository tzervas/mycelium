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

use std::cmp::Ordering;
use std::collections::BTreeMap;

use mycelium_core::{
    binary, operation_hash, ternary, Bound, GuaranteeStrength, Meta, Payload, Provenance, Repr,
    Trit, Value,
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
    /// `bit.not/and/or/xor`), fixed-width balanced-ternary arithmetic (`trit.neg/add/sub/mul`,
    /// M-111), the reduce-to-`Bool` comparison prims (`cmp.eq`/`cmp.lt` → `Binary{1}`, RFC-0032 D1,
    /// M-747), never-silent fixed-width binary arithmetic (`bit.add`/`bit.sub`, RFC-0032 D2,
    /// M-748), the never-silent `Binary` width-cast (`bit.width_cast` — zero-extend widen / checked
    /// narrow, DN-41, M-798), never-silent indexed-sequence access (`seq.len`/`seq.get`, RFC-0032 D3,
    /// M-749), never-silent byte-string access
    /// (`bytes.len`/`bytes.get`/`bytes.slice`/`bytes.concat`, RFC-0032 D4, M-750), and the
    /// never-silent two's-complement `Binary` multiply (`bin.mul`, RFC-0033 §4.1.2/§4.1.3, M-887 —
    /// the first `enb` Gap-B prim; sets the registry pattern the sibling Gap-B/C prims mirror).
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
        // RFC-0032 D1 (M-747): reduce-to-`Bool` comparison/equality over `Binary{N}`/`Ternary{N}`.
        r.register("cmp.eq", prim_cmp_eq);
        r.register("cmp.lt", prim_cmp_lt);
        // RFC-0032 D2 (M-748): never-silent fixed-width binary arithmetic over `Binary{N}`.
        r.register("bit.add", prim_bit_add);
        r.register("bit.sub", prim_bit_sub);
        // RFC-0033 §4.1.2/§4.1.3 (M-887, `enb` Gap B): never-silent two's-complement multiply.
        r.register("bin.mul", prim_bin_mul);
        // DN-41 (M-798): never-silent width-cast (zero-extend widen / checked narrow) over `Binary`.
        r.register("bit.width_cast", prim_width_cast);
        // RFC-0032 D3 (M-749): never-silent indexed-sequence access over `Repr::Seq`.
        r.register("seq.len", prim_seq_len);
        r.register("seq.get", prim_seq_get);
        // RFC-0032 D4 (M-750): never-silent byte-string access over `Repr::Bytes`.
        r.register("bytes.len", prim_bytes_len);
        r.register("bytes.get", prim_bytes_get);
        r.register("bytes.slice", prim_bytes_slice);
        r.register("bytes.concat", prim_bytes_concat);
        // DN-58 §A (M-817): the `Binary` `Fuse` semilattice meet (bitwise-AND). The user-`Data` fuse
        // does **not** register a prim — it elaborates to the resolved `Fuse::join` call (DN-58 §A.5);
        // the non-`Binary` reprs have no committed canonical meet in v0 (DN-58 §A.6 F-A3), so only the
        // `Binary` meet is a built-in here. (RFC-0008 RT6; RFC-0027 §10.6 provenance shape.)
        r.register("fuse_join:binary", prim_fuse_join_binary);
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

// --- RFC-0032 D1 (M-747): reduce-to-`Bool` comparison/equality ---------------------------------
//
// `eq`/`lt` are the two kernel comparison prims (RFC-0032 §5 D1). Each takes two equal-width
// operands of the **same paradigm** (`Binary{N}` or `Ternary{N}`) and reduces to a one-bit truth
// value. **Realization note (engineering call, RFC-0032 Q1):** a kernel prim returns a
// representation [`Value`], never a `.myc` data value, so the `Bool` of D1 bottoms out here as
// **`Binary{1}`** (`0b1` = true, `0b0` = false); the `.myc` `std.cmp` surface lifts that bit into
// the `Bool` ADT (`match eq(a, b) { 0b1 => True, _ => False }`) — a one-line bridge that lands with
// the E13-1 `std.cmp` port (M-718), demonstrated by the M-752 smoke ports. Guarantee **`Exact`**: a
// total decidable relation over a finite domain. Mismatched widths/paradigms are an explicit
// never-silent [`EvalError::PrimType`] — never a silent `false`/`0b0` (G2).

/// Compare two representation values for the comparison prims. Requires equal width and the same
/// paradigm; a mismatch is an explicit error (never a silent ordering). The orderings are the D1
/// total orders: **unsigned magnitude** for `Binary{N}` (MSB-first lexicographic over the bits) and
/// **balanced-integer value** for `Ternary{N}` (MSB-first lexicographic over the signed digits — for
/// fixed-width balanced ternary the most-significant differing digit dominates, so this equals the
/// integer-value order).
fn cmp_repr_operands(prim: &str, a: &Value, b: &Value) -> Result<Ordering, EvalError> {
    match (a.repr(), b.repr()) {
        (Repr::Binary { width: wa }, Repr::Binary { width: wb }) => {
            if wa != wb {
                return Err(EvalError::PrimType {
                    prim: prim.to_owned(),
                    why: format!("width mismatch: Binary{{{wa}}} vs Binary{{{wb}}}"),
                });
            }
            let xa = as_bits(prim, a)?;
            let xb = as_bits(prim, b)?;
            Ok(xa.iter().cmp(xb.iter()))
        }
        (Repr::Ternary { trits: ta }, Repr::Ternary { trits: tb }) => {
            if ta != tb {
                return Err(EvalError::PrimType {
                    prim: prim.to_owned(),
                    why: format!("width mismatch: Ternary{{{ta}}} vs Ternary{{{tb}}}"),
                });
            }
            let xa = as_trits(prim, a)?;
            let xb = as_trits(prim, b)?;
            Ok(xa
                .iter()
                .map(|t| ternary::digit(*t))
                .cmp(xb.iter().map(|t| ternary::digit(*t))))
        }
        _ => Err(EvalError::PrimType {
            prim: prim.to_owned(),
            why: "comparison requires two operands of the same paradigm (both Binary or both \
                  Ternary)"
                .to_owned(),
        }),
    }
}

/// Build the `Binary{1}` truth value for a comparison result (`0b1` = true). Threads provenance/
/// guarantee honestly via [`compose_result`]; comparison has no defined ε-propagation over an
/// approximate input, so an approximate operand is refused (`ApproxRule::Refuse`) rather than
/// fabricating a bound (G2/VR-5).
fn bool_result(prim: &str, inputs: &[&Value], truth: bool) -> Result<Value, EvalError> {
    compose_result(
        prim,
        inputs,
        Repr::Binary { width: 1 },
        Payload::Bits(vec![truth]),
        ApproxRule::Refuse,
    )
}

/// `cmp.eq : (T{N}, T{N}) → Binary{1}` — structural width-typed equality (RFC-0032 D1).
fn prim_cmp_eq(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    expect_arity(prim, args, 2)?;
    let truth = cmp_repr_operands(prim, args[0], args[1])? == Ordering::Equal;
    bool_result(prim, args, truth)
}

/// `cmp.lt : (T{N}, T{N}) → Binary{1}` — the D1 total order (`a < b`).
fn prim_cmp_lt(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    expect_arity(prim, args, 2)?;
    let truth = cmp_repr_operands(prim, args[0], args[1])? == Ordering::Less;
    bool_result(prim, args, truth)
}

// --- RFC-0032 D2 (M-748): never-silent fixed-width binary arithmetic ---------------------------
//
// `bit.add`/`bit.sub` are unsigned fixed-width ripple-carry add/subtract over `Binary{N}` (bits
// MSB-first), exactly mirroring the `trit.*` in-range contract: a result outside `[0, 2^N)` is an
// explicit [`EvalError::Overflow`], **never** a silent wrap (RFC-0032 §5 D2; G2). Guarantee
// **`Exact`** on the in-range result. (A wrapping/modular `add` is intentionally absent — it would
// be a separate, *declared* op, never this never-silent one.)

/// Shared kernel for the never-silent binary arithmetic prims. `subtract == false` is ripple-carry
/// addition (carry-out ⇒ overflow); `subtract == true` is ripple-borrow subtraction (borrow-out ⇒
/// underflow, i.e. a negative result with no unsigned representation). Operands must be equal-width
/// `Binary`; a width mismatch is an explicit [`EvalError::PrimType`].
fn bin_arith(prim: &str, args: &[&Value], subtract: bool) -> Result<Value, EvalError> {
    expect_arity(prim, args, 2)?;
    let a = as_bits(prim, args[0])?;
    let b = as_bits(prim, args[1])?;
    if a.len() != b.len() {
        return Err(EvalError::PrimType {
            prim: prim.to_owned(),
            why: format!("width mismatch: {} vs {} bits", a.len(), b.len()),
        });
    }
    let n = a.len();
    let mut out = vec![false; n];
    if subtract {
        let mut borrow = 0i8;
        for i in (0..n).rev() {
            let d = i8::from(a[i]) - i8::from(b[i]) - borrow;
            if d < 0 {
                out[i] = (d + 2) == 1;
                borrow = 1;
            } else {
                out[i] = d == 1;
                borrow = 0;
            }
        }
        if borrow != 0 {
            return Err(EvalError::Overflow {
                prim: prim.to_owned(),
            });
        }
    } else {
        let mut carry = 0u8;
        for i in (0..n).rev() {
            let s = u8::from(a[i]) + u8::from(b[i]) + carry;
            out[i] = (s & 1) == 1;
            carry = s >> 1;
        }
        if carry != 0 {
            return Err(EvalError::Overflow {
                prim: prim.to_owned(),
            });
        }
    }
    compose_result(
        prim,
        args,
        args[0].repr().clone(),
        Payload::Bits(out),
        ApproxRule::Refuse,
    )
}

/// `bit.add : (Binary{N}, Binary{N}) → Binary{N}` — never-silent unsigned addition (RFC-0032 D2).
fn prim_bit_add(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    bin_arith(prim, args, false)
}

/// `bit.sub : (Binary{N}, Binary{N}) → Binary{N}` — never-silent unsigned subtraction (RFC-0032 D2).
fn prim_bit_sub(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    bin_arith(prim, args, true)
}

// --- RFC-0033 §4.1.2/§4.1.3 (M-887, `enb` Gap B): never-silent two's-complement multiply --------
//
// `bin.mul` is the first Gap-B prim of the RFC-0033 shared two's-complement arithmetic set
// (ADR-028: signedness lives in the *op*, not the `Repr` — `add`/`sub`/`mul`/`neg` are bit-identical
// across the signed/unsigned reading of the operands; only division, comparison, shift, and
// overflow *detection* are signedness-split). This prim reads its equal-width `Binary{N}` operands
// under the two's-complement (**signed**) interpretation — "two's-complement multiply" per the
// M-887 task naming — distinct from `bit.add`/`bit.sub`'s existing **unsigned** overflow contract
// (RFC-0032 D2). The kernel codec lives in [`mycelium_core::binary`] (the M-120 two's-complement
// home, shared with the binary↔ternary swap), mirroring how `trit_binop` delegates to
// [`mycelium_core::ternary`]. Never-silent: an out-of-range product is an explicit
// [`EvalError::Overflow`], never a wrap (RFC-0033 §4.1.3; G2).
//
// **Registry pattern for the rest of Gap B/C (M-766/M-767/M-888/M-889/…):** kernel codec in
// `mycelium-core` (arithmetic + the never-silent bound, `Option<Vec<bool>>`) → a thin prim wrapper
// here that checks arity/width, calls the codec, and maps `None` to `EvalError::Overflow` →
// registered under a `bin.*`/`bit.*`/`trit.*`-namespaced kernel name → surfaced in
// `mycelium-l1/src/checkty.rs` (`prim_family`/`prim_sig`/`prim_kernel_name`) under a distinct
// surface name → pinned in `mycelium-core::PrimTable::builtins()` (the content-addressed `Π`,
// DN-10 §3.4 equivalence) and `mycelium-l1/tests/prim_table.rs`'s `surface_cases()`.
//
// **Width cap (current scope).** [`mycelium_core::binary::mul`] is exact for `n ≤
// `[`mycelium_core::binary::MUL_MAX_WIDTH`]` (an `i128` intermediate product — the same cap
// `bits_to_int`/`int_to_bits` already declare); a wider operand refuses with an explicit
// [`EvalError::PrimType`] naming the cap, never a silent truncation. Arbitrary-width `Binary`
// multiply (matching `bit.add`/`bit.sub`'s width-unbounded ripple-carry) is out of scope for
// M-887 — FLAGged for the Gap-B follow-ons that reconcile the full shared op set (M-766/M-767).

/// `bin.mul : (Binary{N}, Binary{N}) → Binary{N}` — never-silent two's-complement multiply
/// (RFC-0033 §4.1.2/§4.1.3, M-887). Equal-width operands, `N ≤
/// `[`mycelium_core::binary::MUL_MAX_WIDTH`]` (see the module note above); a width mismatch or an
/// over-cap width is an explicit [`EvalError::PrimType`], and an out-of-range product is an
/// explicit [`EvalError::Overflow`] — never a silent wrap (G2).
fn prim_bin_mul(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    expect_arity(prim, args, 2)?;
    let a = as_bits(prim, args[0])?;
    let b = as_bits(prim, args[1])?;
    if a.len() != b.len() {
        return Err(EvalError::PrimType {
            prim: prim.to_owned(),
            why: format!("width mismatch: {} vs {} bits", a.len(), b.len()),
        });
    }
    if a.len() > binary::MUL_MAX_WIDTH {
        return Err(EvalError::PrimType {
            prim: prim.to_owned(),
            why: format!(
                "width {} exceeds the {}-bit two's-complement multiply cap (M-887 scope)",
                a.len(),
                binary::MUL_MAX_WIDTH
            ),
        });
    }
    let out = binary::mul(a, b).ok_or_else(|| EvalError::Overflow {
        prim: prim.to_owned(),
    })?;
    compose_result(
        prim,
        args,
        args[0].repr().clone(),
        Payload::Bits(out),
        ApproxRule::Refuse,
    )
}

// --- DN-41 (M-798): never-silent `Binary` width-cast -------------------------------------------
//
// `bit.width_cast(value: Binary{N}, into: Binary{M}) -> Binary{M}` re-widths an unsigned `Binary`
// value (MSB-first). Because `Binary` is **sign-free** (ADR-028), a re-width is purely a matter of
// the unsigned magnitude:
//   - **Widen** (`M > N`): **zero-extension** — pad `M − N` zero bits on the MSB side. Exact, total,
//     lossless (the unsigned value is unchanged); the guarantee is `Exact`.
//   - **Identity** (`M == N`): a copy. Exact.
//   - **Narrow** (`M < N`): the value fits `Binary{M}` **iff** every dropped high bit (the top
//     `N − M`) is zero. A fitting narrow is exact and lossless; a value that does **not** fit is a
//     never-silent [`EvalError::Overflow`] — never a silent truncation (G2/VR-5), exactly mirroring
//     the `bit.add`/`bit.sub` out-of-range contract.
// **Width witness, not a value operand.** The target width `M` is carried by the *second* operand's
// **width** (`into.repr()` = `Binary{M}`); its *bits are unused*. This threads `M` to the kernel
// through the existing surface→kernel dispatch (`prim_kernel_name`) with no result-type plumbing —
// the motivating call `lt(width_cast(idx8, len32), len32)` reuses the very `Binary{32}` length it is
// about to compare against as the width witness (M-717: widen a `Binary{8}` byte index to compare it
// against a `Binary{32}` `bytes_len`). The result inherits the *first* operand's guarantee/bound by
// the standard `compose_result` threading (an approximate value is refused — width-cast has no
// defined ε-rule; G2). A non-`Binary` operand on either side is an explicit type refusal.

/// `bit.width_cast : (Binary{N}, Binary{M}) → Binary{M}` — never-silent unsigned width-cast (DN-41).
/// The second operand is a **width witness** (only its `Binary{M}` width is read; its bits are
/// ignored). Widening (`M > N`) zero-extends (Exact); narrowing (`M < N`) refuses with
/// [`EvalError::Overflow`] when the value does not fit `M` bits — never a silent truncation (G2).
fn prim_width_cast(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    expect_arity(prim, args, 2)?;
    let value = as_bits(prim, args[0])?;
    let witness = as_bits(prim, args[1])?;
    let n = value.len();
    let m = witness.len();
    let out: Vec<bool> = if m >= n {
        // Widen (or identity): zero-extend on the MSB side. `Binary` is sign-free (ADR-028), so the
        // pad bits are always zero; the unsigned magnitude is preserved exactly.
        let mut bits = vec![false; m - n];
        bits.extend_from_slice(value);
        bits
    } else {
        // Narrow: the value fits `Binary{M}` iff the dropped high `N − M` bits are all zero. A set
        // high bit means the magnitude exceeds `2^M − 1`, so the narrow would lose information — an
        // explicit never-silent refusal, never a silent truncation (G2/VR-5).
        let (dropped, kept) = value.split_at(n - m);
        if dropped.iter().any(|&b| b) {
            return Err(EvalError::Overflow {
                prim: prim.to_owned(),
            });
        }
        kept.to_vec()
    };
    // Thread the result off the **value** operand only (the witness contributes its width, not its
    // value/guarantee): compose over `[value]` so the result inherits the value's guarantee/bound
    // (an approximate value is refused — width-cast has no defined ε-propagation rule; G2). The
    // result `Repr` is the witness's own `Binary{M}` (cloned, never reconstructed from a `usize`
    // cast) — its width is `M` by construction, so the output width matches the produced bits.
    compose_result(
        prim,
        &args[..1],
        args[1].repr().clone(),
        Payload::Bits(out),
        ApproxRule::Refuse,
    )
}

// --- RFC-0032 D3 (M-749): indexed-sequence primitives ------------------------------------------
//
// `seq.get`/`seq.len` are the never-silent indexing surface over `Repr::Seq` (RFC-0032 D3). A kernel
// prim returns a representation [`Value`], not a `.myc` data value, so:
//   - `seq.len(s) -> Binary{32}` is the element count as an unsigned 32-bit value (the seq's `len`).
//   - `seq.get(s, i) -> elem` returns the `i`-th element, with `i` an unsigned `Binary{N}` index. An
//     **out-of-bounds index is an explicit [`EvalError::PrimType`]**, never a panic or a silent
//     default (G2). The `.myc` `Vec::get` surface lifts this into the `Option` the spec names
//     (`get(s, i) -> Option<elem>`); the never-silence is what makes that lift honest.
// Guarantee **`Exact`**: total/decidable over the in-range domain.

/// Interpret an unsigned `Binary{N}` value as a `usize` index (MSB-first bits). A non-Binary operand
/// is an explicit error; a width that cannot fit `usize` (`> 64` here, conservatively the pointer
/// width is ≥ 32) overflowing `usize` is also refused rather than silently truncated (G2).
fn as_index(prim: &str, v: &Value) -> Result<usize, EvalError> {
    let bits = as_bits(prim, v)?;
    if bits.len() > usize::BITS as usize {
        return Err(EvalError::PrimType {
            prim: prim.to_owned(),
            why: format!(
                "index width {} exceeds the {}-bit usize index space",
                bits.len(),
                usize::BITS
            ),
        });
    }
    // MSB-first accumulate; the width guard above keeps this within `usize`.
    let idx = bits
        .iter()
        .fold(0usize, |acc, &b| (acc << 1) | usize::from(b));
    Ok(idx)
}

/// Extract the elements of a `Repr::Seq` operand; a non-sequence is an explicit error (G2).
fn as_seq<'a>(prim: &str, v: &'a Value) -> Result<&'a [Value], EvalError> {
    v.seq_elems().ok_or_else(|| EvalError::PrimType {
        prim: prim.to_owned(),
        why: "expected a Seq operand".to_owned(),
    })
}

/// `seq.len : Seq<T, N> → Binary{32}` — the element count as an unsigned 32-bit value (RFC-0032 D3).
fn prim_seq_len(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    expect_arity(prim, args, 1)?;
    let elems = as_seq(prim, args[0])?;
    // The seq's `len` is a `u32` field (well-formedness caps it at MAX_DIM ≤ 2^30 ≤ 2^32), so 32
    // bits hold it exactly. Use the same *checked* conversion as `bytes.len` rather than a silent
    // `as u32` truncation — defensive parity in the trusted base, so a future path that ever yields
    // an over-2^32 sequence refuses (G2) instead of wrapping.
    let n = u32::try_from(elems.len()).map_err(|_| EvalError::PrimType {
        prim: prim.to_owned(),
        why: format!(
            "sequence length {} exceeds the 32-bit length encoding",
            elems.len()
        ),
    })?;
    u32_as_binary32(prim, args, n)
}

/// `seq.get : (Seq<T, N>, Binary{W}) → T` — never-silent indexed access (RFC-0032 D3). An
/// out-of-bounds index is an explicit [`EvalError::PrimType`] (the `.myc` surface lifts to `Option`),
/// never a panic or a silent default (G2).
fn prim_seq_get(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    expect_arity(prim, args, 2)?;
    let elems = as_seq(prim, args[0])?;
    let i = as_index(prim, args[1])?;
    match elems.get(i) {
        Some(e) => {
            // Return the element faithfully at **its own** established basis (VR-5): the result's
            // guarantee is the element's, never upgraded — an `Empirical`/`Declared` element must
            // not re-stamp as `Exact` just because the container/index were `Exact`. It is then
            // `meet`-downgraded by the container and index strengths (you cannot trust an element
            // retrieved from a less-certain container more than that container). Indexing is exact
            // and introduces no error, so the element's `bound` carries through unchanged; if that
            // (guarantee, bound) pairing is internally inconsistent for some exotic container meta,
            // `Meta::new` refuses (`EvalError::Wf`) rather than fabricating one (G2). Provenance is
            // re-stamped `Derived` from the access inputs (lineage), as for any other prim result.
            let guarantee = GuaranteeStrength::propagate(
                e.meta().guarantee(),
                args.iter().map(|v| v.meta().guarantee()),
            );
            let bound = e.meta().bound().cloned();
            let provenance = Provenance::Derived {
                op: operation_hash(prim),
                inputs: args.iter().map(|v| v.content_hash()).collect(),
            };
            let meta =
                Meta::new(provenance, guarantee, bound, None, None, None).map_err(EvalError::Wf)?;
            Value::new(e.repr().clone(), e.payload().clone(), meta).map_err(EvalError::Wf)
        }
        None => Err(EvalError::PrimType {
            prim: prim.to_owned(),
            why: format!(
                "index {i} out of bounds for a sequence of length {}",
                elems.len()
            ),
        }),
    }
}

// --- RFC-0032 D4 (M-750): byte-string primitives -----------------------------------------------
//
// `bytes.len`/`bytes.get`/`bytes.slice`/`bytes.concat` are the never-silent byte surface over
// `Repr::Bytes` (RFC-0032 D4). UTF-8 decode is written in `.myc` over these prims, never in the
// kernel. Out-of-range access is an explicit refusal (the `.myc` surface lifts to `Option`); a
// non-bytes operand is an explicit type refusal (G2). Guarantee **`Exact`**.

/// Extract the bytes of a `Repr::Bytes` operand; a non-bytes value is an explicit error (G2).
fn as_bytes_payload<'a>(prim: &str, v: &'a Value) -> Result<&'a [u8], EvalError> {
    v.bytes().ok_or_else(|| EvalError::PrimType {
        prim: prim.to_owned(),
        why: "expected a Bytes operand".to_owned(),
    })
}

/// Build a `Binary{32}` value from a `u32`, MSB-first (the never-silent length/index encoding).
fn u32_as_binary32(prim: &str, inputs: &[&Value], n: u32) -> Result<Value, EvalError> {
    let out: Vec<bool> = (0..32).rev().map(|k| (n >> k) & 1 == 1).collect();
    compose_result(
        prim,
        inputs,
        Repr::Binary { width: 32 },
        Payload::Bits(out),
        ApproxRule::Refuse,
    )
}

/// `bytes.len : Bytes → Binary{32}` — the byte count (RFC-0032 D4).
fn prim_bytes_len(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    expect_arity(prim, args, 1)?;
    let bytes = as_bytes_payload(prim, args[0])?;
    let n = u32::try_from(bytes.len()).map_err(|_| EvalError::PrimType {
        prim: prim.to_owned(),
        why: format!(
            "byte length {} exceeds the 32-bit length encoding",
            bytes.len()
        ),
    })?;
    u32_as_binary32(prim, args, n)
}

/// `bytes.get : (Bytes, Binary{W}) → Binary{8}` — never-silent indexed byte access (RFC-0032 D4). An
/// out-of-bounds index is an explicit refusal (the `.myc` surface lifts to `Option`), never a silent
/// default (G2). The returned byte is a `Binary{8}` value (MSB-first).
fn prim_bytes_get(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    expect_arity(prim, args, 2)?;
    let bytes = as_bytes_payload(prim, args[0])?;
    let i = as_index(prim, args[1])?;
    match bytes.get(i) {
        Some(&byte) => {
            let out: Vec<bool> = (0..8).rev().map(|k| (byte >> k) & 1 == 1).collect();
            compose_result(
                prim,
                args,
                Repr::Binary { width: 8 },
                Payload::Bits(out),
                ApproxRule::Refuse,
            )
        }
        None => Err(EvalError::PrimType {
            prim: prim.to_owned(),
            why: format!(
                "byte index {i} out of bounds for a byte string of length {}",
                bytes.len()
            ),
        }),
    }
}

/// `bytes.slice : (Bytes, Binary{W}, Binary{W}) → Bytes` — never-silent sub-slice `[start, end)`
/// (RFC-0032 D4). An out-of-range or inverted range is an explicit refusal, never a silently-clamped
/// range (G2).
fn prim_bytes_slice(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    expect_arity(prim, args, 3)?;
    let bytes = as_bytes_payload(prim, args[0])?;
    let start = as_index(prim, args[1])?;
    let end = as_index(prim, args[2])?;
    if start > end || end > bytes.len() {
        return Err(EvalError::PrimType {
            prim: prim.to_owned(),
            why: format!(
                "slice range [{start}, {end}) is out of bounds or inverted for a byte string of \
                 length {}",
                bytes.len()
            ),
        });
    }
    compose_result(
        prim,
        args,
        Repr::Bytes,
        Payload::Bytes(bytes[start..end].to_vec()),
        ApproxRule::Refuse,
    )
}

/// `bytes.concat : (Bytes, Bytes) → Bytes` — byte concatenation (RFC-0032 D4). Total/`Exact`.
fn prim_bytes_concat(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    expect_arity(prim, args, 2)?;
    let a = as_bytes_payload(prim, args[0])?;
    let b = as_bytes_payload(prim, args[1])?;
    let mut out = Vec::with_capacity(a.len() + b.len());
    out.extend_from_slice(a);
    out.extend_from_slice(b);
    compose_result(
        prim,
        args,
        Repr::Bytes,
        Payload::Bytes(out),
        ApproxRule::Refuse,
    )
}

// --- DN-58 §A (M-817): the `Binary` `Fuse` semilattice meet ------------------------------------
//
// `fuse(a, b)` is a lawful binary merge over a declared commutative/associative/idempotent meet
// (RFC-0008 RT6). For the `Binary` paradigm the meet is **bitwise-AND** — the greatest-lower-bound of
// the boolean lattice, idempotent (`a ∧ a = a`), commutative, and associative. This is the executable
// **repr** case; the user-`Data` case elaborates instead to the resolved `Fuse::join` call (DN-58
// §A.5), and the non-`Binary` reprs have no committed canonical meet in v0 (DN-58 §A.6 F-A3).
//
// **Provenance shape (DN-58 §A.5 / RFC-0027 §10.6).** A `fuse` result's provenance is the canonical
// `Derived{op:"fuse_join", inputs:[hash(a), hash(b)]}` — the merge node the δ-CRDT Merkle anti-entropy
// story reads downstream — **not** the per-paradigm prim name, so every fusible paradigm shares one
// merge-op identity. The guarantee is the `meet` of the inputs' guarantees (RFC-0001 §4.7); the meet
// op is intrinsically `Exact` (a total greatest-lower-bound). The semilattice laws are **`Empirical`**
// (property-tested over bit-vectors, not mechanized-`Proven` here — VR-5).

/// Compose a `fuse_join` result: the `meet` of the input guarantees + the canonical
/// `Derived{op:"fuse_join", …}` provenance (DN-58 §A.5). The meet op introduces no error, so an
/// **exact** pair yields an exact result with no bound; an **approximate** input has no defined
/// ε-propagation rule for the meet (as for `bit.and`), so it is refused — never a fabricated bound
/// (G2/VR-5).
fn fuse_join_result(
    prim: &str,
    inputs: &[&Value],
    repr: Repr,
    payload: Payload,
) -> Result<Value, EvalError> {
    let strength = GuaranteeStrength::propagate(
        GuaranteeStrength::Exact,
        inputs.iter().map(|v| v.meta().guarantee()),
    );
    if strength != GuaranteeStrength::Exact {
        // No committed ε-rule for the meet over an approximate input — refuse honestly (G2/VR-5),
        // exactly as the underlying `bit.and` does.
        return Err(EvalError::ApproxCompositionUnsupported {
            prim: prim.to_owned(),
        });
    }
    let provenance = Provenance::Derived {
        op: operation_hash("fuse_join"),
        inputs: inputs.iter().map(|v| v.content_hash()).collect(),
    };
    let meta = Meta::new(provenance, GuaranteeStrength::Exact, None, None, None, None)
        .map_err(EvalError::Wf)?;
    Value::new(repr, payload, meta).map_err(EvalError::Wf)
}

/// `fuse_join:binary : (Binary{N}, Binary{N}) → Binary{N}` — the `Binary` `Fuse` meet (bitwise-AND;
/// DN-58 §A). Commutative/associative/idempotent (`Empirical`). A width/paradigm mismatch is an
/// explicit [`EvalError::PrimType`], never a silent coercion (G2). The result carries the canonical
/// `fuse_join` provenance (DN-58 §A.5).
fn prim_fuse_join_binary(prim: &str, args: &[&Value]) -> Result<Value, EvalError> {
    expect_arity(prim, args, 2)?;
    let a = as_bits(prim, args[0])?;
    let b = as_bits(prim, args[1])?;
    if a.len() != b.len() {
        return Err(EvalError::PrimType {
            prim: prim.to_owned(),
            why: format!("width mismatch: {} vs {} bits", a.len(), b.len()),
        });
    }
    let out: Vec<bool> = a.iter().zip(b).map(|(&x, &y)| x & y).collect();
    fuse_join_result(prim, args, args[0].repr().clone(), Payload::Bits(out))
}
