//! Values: a [`Repr`] + a representation-specific payload + [`Meta`] (RFC-0001 §4.2;
//! `value.schema.json`).

use crate::meta::Meta;
use crate::repr::Repr;
use crate::WfError;

/// A balanced trit in `{-1, 0, +1}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trit {
    /// −1.
    Neg,
    /// 0.
    Zero,
    /// +1.
    Pos,
}

/// Representation-specific payload. Detailed VSA storage (sparse index/value pairs) lands with the
/// VSA submodule (M-130); here a hypervector is a dense scalar vector.
#[derive(Debug, Clone, PartialEq)]
pub enum Payload {
    /// Bits of a `Binary` value (length == `width`).
    Bits(Vec<bool>),
    /// Trits of a `Ternary` value (length == `trits`).
    Trits(Vec<Trit>),
    /// Scalars of a `Dense` value (length == `dim`).
    Scalars(Vec<f64>),
    /// Components of a `Vsa` value (length == `dim`).
    Hypervector(Vec<f64>),
}

/// A Mycelium value. The only constructor, [`Value::new`], rejects a malformed `repr` and a
/// payload that does not match its `repr` (the wire-form well-formedness of `value.schema.json`).
#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    repr: Repr,
    payload: Payload,
    meta: Meta,
}

impl Value {
    /// Build a value, checking `repr.well_formed()` and that `payload` matches `repr`. (`meta` is
    /// already invariant-checked by [`Meta::new`].)
    pub fn new(repr: Repr, payload: Payload, meta: Meta) -> Result<Self, WfError> {
        if !repr.well_formed() {
            return Err(WfError::MalformedRepr);
        }
        if !payload_matches(&repr, &payload) {
            return Err(WfError::PayloadReprMismatch);
        }
        Ok(Value {
            repr,
            payload,
            meta,
        })
    }

    /// The representation descriptor.
    #[must_use]
    pub fn repr(&self) -> &Repr {
        &self.repr
    }
    /// The payload.
    #[must_use]
    pub fn payload(&self) -> &Payload {
        &self.payload
    }
    /// The metadata.
    #[must_use]
    pub fn meta(&self) -> &Meta {
        &self.meta
    }
}

fn payload_matches(repr: &Repr, payload: &Payload) -> bool {
    match (repr, payload) {
        (Repr::Binary { width }, Payload::Bits(b)) => b.len() == *width as usize,
        (Repr::Ternary { trits }, Payload::Trits(t)) => t.len() == *trits as usize,
        (Repr::Dense { dim, .. }, Payload::Scalars(s)) => s.len() == *dim as usize,
        (Repr::Vsa { dim, .. }, Payload::Hypervector(h)) => h.len() == *dim as usize,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::Provenance;

    #[test]
    fn well_matched_value_constructs() {
        let v = Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(vec![true, false, true, true, false, false, true, false]),
            Meta::exact(Provenance::Root),
        );
        assert!(v.is_ok());
    }

    #[test]
    fn payload_length_must_match_repr() {
        let v = Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(vec![true, false]), // wrong length
            Meta::exact(Provenance::Root),
        );
        assert_eq!(v.unwrap_err(), WfError::PayloadReprMismatch);
    }

    #[test]
    fn payload_paradigm_must_match_repr() {
        let v = Value::new(
            Repr::Binary { width: 1 },
            Payload::Trits(vec![Trit::Pos]), // wrong paradigm
            Meta::exact(Provenance::Root),
        );
        assert_eq!(v.unwrap_err(), WfError::PayloadReprMismatch);
    }

    #[test]
    fn malformed_repr_rejected() {
        let v = Value::new(
            Repr::Binary { width: 0 },
            Payload::Bits(vec![]),
            Meta::exact(Provenance::Root),
        );
        assert_eq!(v.unwrap_err(), WfError::MalformedRepr);
    }
}
