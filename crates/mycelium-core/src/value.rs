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
    /// Elements of a [`Repr::Seq`] value (length == the seq's `len`; every element's `repr` matches
    /// the seq's `elem`). RFC-0032 D3 (M-749).
    Seq(Vec<Value>),
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
    /// A sequence payload renders as a JSON array of self-describing element [`Value`]s — each
    /// element round-trips through its own `Value` (de)serialization, so the seq is checked
    /// element-wise on the way in. RFC-0032 D3 (M-749).
    #[serde(rename = "seq")]
    Seq(Vec<Value>),
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
            Payload::Seq(elems) => PayloadWire::Seq(elems.clone()),
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
            PayloadWire::Seq(elems) => Payload::Seq(elems),
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
    /// Build a value, checking [`Repr::check_well_formed`] (positivity, non-empty model, and the
    /// [`crate::repr::MAX_DIM`] over-allocation cap) and that `payload` matches `repr`. (`meta` is
    /// already invariant-checked by [`Meta::new`].)
    pub fn new(repr: Repr, payload: Payload, meta: Meta) -> Result<Self, WfError> {
        // Never-silent well-formedness: rejects a non-positive dimension *and* (DN-40 §3) a declared
        // dimension above `repr::MAX_DIM` before the payload is examined or any value materialized,
        // with an error naming the offending field/value/cap (over-allocation / DoS guard).
        repr.check_well_formed()?;
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

    /// The element count of a [`Repr::Seq`] value, or `None` for any other representation
    /// (never-silent — a non-sequence has no length here, G2). `Exact`: a total decidable query.
    #[must_use]
    pub fn seq_len(&self) -> Option<usize> {
        match self.payload() {
            Payload::Seq(elems) => Some(elems.len()),
            _ => None,
        }
    }

    /// Never-silent indexed access into a [`Repr::Seq`] value (RFC-0032 D3): the `i`-th element, or
    /// `None` when `i` is out of bounds **or** the value is not a sequence — **never** a panic or a
    /// silent default (G2). `Exact`: total over its domain. The `.myc` `Vec::get` surface bottoms
    /// out on this.
    #[must_use]
    pub fn seq_get(&self, i: usize) -> Option<&Value> {
        match self.payload() {
            Payload::Seq(elems) => elems.get(i),
            _ => None,
        }
    }

    /// The elements of a [`Repr::Seq`] value as a slice, or `None` for any other representation
    /// (the fold/iterate basis — RFC-0032 D3). Never-silent: a non-sequence yields `None`, not an
    /// empty slice.
    #[must_use]
    pub fn seq_elems(&self) -> Option<&[Value]> {
        match self.payload() {
            Payload::Seq(elems) => Some(elems),
            _ => None,
        }
    }
}

fn payload_matches(repr: &Repr, payload: &Payload) -> bool {
    match (repr, payload) {
        (Repr::Binary { width }, Payload::Bits(b)) => b.len() == *width as usize,
        (Repr::Ternary { trits }, Payload::Trits(t)) => t.len() == *trits as usize,
        (Repr::Dense { dim, .. }, Payload::Scalars(s)) => s.len() == *dim as usize,
        (Repr::Vsa { dim, .. }, Payload::Hypervector(h)) => h.len() == *dim as usize,
        // A sequence payload matches iff it has exactly `len` elements **and** every element's own
        // `repr` equals the declared element repr `elem` (homogeneity — RFC-0032 D3). Each element
        // is itself a `Value`, so its payload↔repr agreement was already enforced by its own
        // `Value::new`; here we only re-check the count and the element-type homogeneity.
        (Repr::Seq { elem, len }, Payload::Seq(elems)) => {
            elems.len() == *len as usize && elems.iter().all(|e| e.repr() == elem.as_ref())
        }
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
