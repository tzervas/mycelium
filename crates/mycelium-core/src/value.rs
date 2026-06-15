//! Values: a [`Repr`] + a representation-specific payload + [`Meta`] (RFC-0001 §4.2;
//! `value.schema.json`).
//!
//! The `serde` wire form is the self-describing `[Repr] ‖ [Meta] ‖ [payload]` of RFC-0001 §4.8,
//! faithfully round-trippable (`deserialize(serialize(v)) == v`, M-104). The `payload` is rendered
//! per paradigm: `Bits`/`Trits` as compact most-significant-first strings (`"10110010"`,
//! `"0-00+0"` over the alphabet `{+,0,-}`, matching `docs/spec/swaps/binary-ternary.md`), and
//! `Scalars`/`Hypervector` as JSON number arrays. [`Value`]'s `Deserialize` routes through
//! [`Value::new`], so a wire value that mismatches its `repr` is rejected, never silently accepted.

use serde::{Deserialize, Serialize};

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

impl Trit {
    /// The most-significant-first wire glyph for this trit: `-` / `0` / `+`
    /// (`docs/spec/swaps/binary-ternary.md` §1).
    #[must_use]
    fn to_wire_char(self) -> char {
        match self {
            Trit::Neg => '-',
            Trit::Zero => '0',
            Trit::Pos => '+',
        }
    }

    /// Parse a wire glyph back into a trit; `None` for any other character.
    #[must_use]
    fn from_wire_char(c: char) -> Option<Trit> {
        match c {
            '-' => Some(Trit::Neg),
            '0' => Some(Trit::Zero),
            '+' => Some(Trit::Pos),
            _ => None,
        }
    }
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

/// The externally-tagged wire projection of [`Payload`] — `{"bits": "…"}`, `{"trits": "…"}`,
/// `{"scalars": […]}`, `{"hypervector": […]}` — the paradigm-specific `payload` of
/// `value.schema.json`.
#[derive(Serialize, Deserialize)]
enum PayloadWire {
    #[serde(rename = "bits")]
    Bits(String),
    #[serde(rename = "trits")]
    Trits(String),
    #[serde(rename = "scalars")]
    Scalars(Vec<f64>),
    #[serde(rename = "hypervector")]
    Hypervector(Vec<f64>),
}

impl Serialize for Payload {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let wire = match self {
            Payload::Bits(bits) => {
                PayloadWire::Bits(bits.iter().map(|&b| if b { '1' } else { '0' }).collect())
            }
            Payload::Trits(trits) => {
                PayloadWire::Trits(trits.iter().map(|&t| t.to_wire_char()).collect())
            }
            Payload::Scalars(xs) => PayloadWire::Scalars(xs.clone()),
            Payload::Hypervector(xs) => PayloadWire::Hypervector(xs.clone()),
        };
        wire.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Payload {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        Ok(match PayloadWire::deserialize(deserializer)? {
            PayloadWire::Bits(s) => {
                let bits = s
                    .chars()
                    .map(|c| match c {
                        '1' => Ok(true),
                        '0' => Ok(false),
                        other => Err(Error::custom(format!(
                            "bit string has non-0/1 char {other:?}"
                        ))),
                    })
                    .collect::<Result<Vec<bool>, _>>()?;
                Payload::Bits(bits)
            }
            PayloadWire::Trits(s) => {
                let trits = s
                    .chars()
                    .map(|c| {
                        Trit::from_wire_char(c).ok_or_else(|| {
                            Error::custom(format!("trit string has non-+0- char {c:?}"))
                        })
                    })
                    .collect::<Result<Vec<Trit>, _>>()?;
                Payload::Trits(trits)
            }
            PayloadWire::Scalars(xs) => Payload::Scalars(xs),
            PayloadWire::Hypervector(xs) => Payload::Hypervector(xs),
        })
    }
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

/// The wire projection of [`Value`] (`value.schema.json`): `[Repr] ‖ [Meta] ‖ [payload]`.
/// `deny_unknown_fields` enforces the schema's `additionalProperties: false` (A6-02).
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValueWire {
    repr: Repr,
    payload: Payload,
    meta: Meta,
}

impl Serialize for Value {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ValueWire {
            repr: self.repr.clone(),
            payload: self.payload.clone(),
            meta: self.meta.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let w = ValueWire::deserialize(deserializer)?;
        // Re-check repr well-formedness and payload↔repr agreement: never silently accept (§4.8).
        Value::new(w.repr, w.payload, w.meta).map_err(serde::de::Error::custom)
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
