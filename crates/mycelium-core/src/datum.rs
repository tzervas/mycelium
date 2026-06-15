//! Algebraic **data values** and the runtime value sum (RFC-0001 §4.2 r3; RFC-0011 §4.6).
//!
//! A [`Construct`](crate::Node) node evaluates to a [`Datum`] — a saturated constructor application
//! (constructor tag + field values). A datum is **not** one of the four paradigm [`Repr`](crate::Repr)
//! kinds (those stay closed — RFC-0011 §4.6); it is a different category of value, so the runtime
//! value the interpreter yields is the sum [`CoreValue`] = a representation [`Value`] **or** a
//! [`Datum`]. This mirrors `crates/mycelium-l1::eval::L1Value` (`Repr(Value) | Data{…}`) so the L1
//! evaluator and the L0 interpreter agree on the data fragment (NFR-7).
//!
//! # Honesty (the meet-summary; maintainer-confirmed)
//! A datum carries **one** honesty field: a [`GuaranteeStrength`] **summary** — the `meet` of its
//! fields' guarantees. It carries **no [`Bound`](crate::Bound)**: a datum is not itself an
//! approximation; the quantitative bounds that justify a non-`Exact` summary live on the **leaf
//! representation values** it contains (drillable via provenance / EXPLAIN). M-I1
//! (`guarantee≠Exact ⟺ bound`) is an invariant of *representation* values — where the bound
//! quantifies *that value's* error — not of structural composites. The datum summary is a derived
//! disclosure (an addendum to RFC-0001 §4.7's propagation), and like every guarantee it is
//! monotone-downward (`Construct`/`Match` only ever `meet`, never upgrade — VR-5).

use crate::content::Canon;
use crate::data::CtorRef;
use crate::guarantee::GuaranteeStrength;
use crate::id::ContentHash;
use crate::value::Value;

/// A constructed data value: a saturated constructor application (RFC-0011 §4.1, W6) with a
/// meet-summary guarantee.
#[derive(Debug, Clone, PartialEq)]
pub struct Datum {
    ctor: CtorRef,
    fields: Vec<CoreValue>,
    guarantee: GuaranteeStrength,
}

impl Datum {
    /// Construct a datum from a constructor reference and its field values, computing the
    /// meet-summary guarantee = `meet` of the fields' guarantees with the intrinsic `Exact`
    /// (construction adds no approximation — RFC-0011 §4.6). With all-`Exact` fields the summary is
    /// `Exact`, consistent with M-I1 (no bound).
    #[must_use]
    pub fn new(ctor: CtorRef, fields: Vec<CoreValue>) -> Self {
        let guarantee = GuaranteeStrength::meet_all(fields.iter().map(CoreValue::guarantee));
        Datum {
            ctor,
            fields,
            guarantee,
        }
    }

    /// The constructor reference (`#T#i`).
    #[must_use]
    pub fn ctor(&self) -> &CtorRef {
        &self.ctor
    }

    /// The field values, in declaration order.
    #[must_use]
    pub fn fields(&self) -> &[CoreValue] {
        &self.fields
    }

    /// The meet-summary guarantee.
    #[must_use]
    pub fn guarantee(&self) -> GuaranteeStrength {
        self.guarantee
    }

    /// This datum with its summary guarantee met against `g` (weakest-wins). Used by `Match` to
    /// fold the scrutinee's guarantee into a data result (RFC-0011 §4.6); never upgrades (VR-5).
    #[must_use]
    pub fn meet_guarantee(mut self, g: GuaranteeStrength) -> Self {
        self.guarantee = self.guarantee.meet(g);
        self
    }

    /// The identity-bearing content hash of this datum: its constructor reference and its fields'
    /// content (the guarantee summary is dynamic metadata — excluded, like `Meta` on a [`Value`];
    /// RFC-0001 §4.6).
    #[must_use]
    pub fn content_hash(&self) -> ContentHash {
        let mut c = Canon::new();
        c.datum(self);
        c.finish()
    }
}

/// A runtime value: a representation [`Value`] (one of the four paradigm kinds) or an algebraic
/// [`Datum`]. The interpreter's normal forms (RFC-0011 §4.6).
#[derive(Debug, Clone, PartialEq)]
pub enum CoreValue {
    /// A representation value (`repr + payload + Meta`).
    Repr(Value),
    /// An algebraic data value.
    Data(Datum),
}

impl CoreValue {
    /// The underlying representation value, if this is a [`CoreValue::Repr`].
    #[must_use]
    pub fn as_repr(&self) -> Option<&Value> {
        match self {
            CoreValue::Repr(v) => Some(v),
            CoreValue::Data(_) => None,
        }
    }

    /// The underlying datum, if this is a [`CoreValue::Data`].
    #[must_use]
    pub fn as_data(&self) -> Option<&Datum> {
        match self {
            CoreValue::Data(d) => Some(d),
            CoreValue::Repr(_) => None,
        }
    }

    /// This value's guarantee: a representation value's own `Meta.guarantee`, or a datum's
    /// meet-summary. The single honesty accessor the `Construct`/`Match` meet rules fold over.
    #[must_use]
    pub fn guarantee(&self) -> GuaranteeStrength {
        match self {
            CoreValue::Repr(v) => v.meta().guarantee(),
            CoreValue::Data(d) => d.guarantee(),
        }
    }

    /// The identity-bearing content hash (RFC-0001 §4.6): a representation value's repr+payload, or
    /// a datum's constructor+fields.
    #[must_use]
    pub fn content_hash(&self) -> ContentHash {
        match self {
            CoreValue::Repr(v) => v.content_hash(),
            CoreValue::Data(d) => d.content_hash(),
        }
    }
}

impl From<Value> for CoreValue {
    fn from(v: Value) -> Self {
        CoreValue::Repr(v)
    }
}

impl From<Datum> for CoreValue {
    fn from(d: Datum) -> Self {
        CoreValue::Data(d)
    }
}

impl Canon {
    /// Encode a [`CoreValue`]'s identity-bearing content (a representation value's repr+payload, or
    /// a datum's constructor+fields). `Meta` / the datum summary are dynamic and excluded.
    pub(crate) fn core_value(&mut self, v: &CoreValue) {
        match v {
            CoreValue::Repr(rv) => self.value(rv),
            CoreValue::Data(d) => self.datum(d),
        }
    }

    /// Encode a [`Datum`]: its constructor reference then each field (order significant).
    pub(crate) fn datum(&mut self, d: &Datum) {
        self.tag(crate::content::tag::DATUM);
        self.ctor_ref(d.ctor());
        self.u64(d.fields().len() as u64);
        for f in d.fields() {
            self.core_value(f);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{CtorSpec, DataRegistry, DeclSpec, FieldSpec};
    use crate::meta::{Meta, Provenance};
    use crate::repr::Repr;
    use crate::value::Payload;
    use std::collections::BTreeMap;

    fn nat_registry() -> DataRegistry {
        let mut m = BTreeMap::new();
        m.insert(
            "Nat".to_owned(),
            DeclSpec {
                ctors: vec![
                    CtorSpec { fields: vec![] },
                    CtorSpec {
                        fields: vec![FieldSpec::Data("Nat".to_owned())],
                    },
                ],
            },
        );
        DataRegistry::build(&m).unwrap()
    }

    fn byte(g: GuaranteeStrength) -> Value {
        let meta = match g {
            GuaranteeStrength::Exact => Meta::exact(Provenance::Root),
            other => Meta::new(
                Provenance::Root,
                other,
                Some(crate::bound::Bound {
                    kind: crate::bound::BoundKind::Error {
                        eps: 0.1,
                        norm: crate::bound::NormKind::Linf,
                    },
                    basis: match other {
                        GuaranteeStrength::Proven => crate::bound::BoundBasis::ProvenThm {
                            citation: "t".into(),
                        },
                        GuaranteeStrength::Empirical => crate::bound::BoundBasis::EmpiricalFit {
                            trials: 1,
                            method: "m".into(),
                        },
                        _ => crate::bound::BoundBasis::UserDeclared,
                    },
                }),
                None,
                None,
                None,
            )
            .unwrap(),
        };
        Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(vec![false; 8]),
            meta,
        )
        .unwrap()
    }

    #[test]
    fn nullary_datum_is_exact() {
        let reg = nat_registry();
        let z = Datum::new(reg.ctor_ref("Nat", 0).unwrap(), vec![]);
        assert_eq!(z.guarantee(), GuaranteeStrength::Exact);
    }

    #[test]
    fn construct_summary_is_the_meet_of_fields() {
        let reg = nat_registry();
        // A datum over an Empirical leaf summarises as Empirical (honesty degrades — RFC-0011 §4.6).
        let s = Datum::new(
            reg.ctor_ref("Nat", 1).unwrap(),
            vec![CoreValue::Repr(byte(GuaranteeStrength::Empirical))],
        );
        assert_eq!(s.guarantee(), GuaranteeStrength::Empirical);
        // Exact leaf → Exact summary.
        let s2 = Datum::new(
            reg.ctor_ref("Nat", 1).unwrap(),
            vec![CoreValue::Repr(byte(GuaranteeStrength::Exact))],
        );
        assert_eq!(s2.guarantee(), GuaranteeStrength::Exact);
    }

    #[test]
    fn meet_guarantee_only_degrades() {
        let reg = nat_registry();
        let z = Datum::new(reg.ctor_ref("Nat", 0).unwrap(), vec![]);
        assert_eq!(
            z.clone()
                .meet_guarantee(GuaranteeStrength::Exact)
                .guarantee(),
            GuaranteeStrength::Exact
        );
        assert_eq!(
            z.meet_guarantee(GuaranteeStrength::Declared).guarantee(),
            GuaranteeStrength::Declared
        );
    }

    #[test]
    fn content_hash_excludes_the_summary_but_not_the_fields() {
        let reg = nat_registry();
        // Two S(Z) data with differently-tagged leaves: the hash is over repr+payload only, so the
        // guarantee summary does not change identity, but a different payload does.
        let z = || Datum::new(reg.ctor_ref("Nat", 0).unwrap(), vec![]);
        let a = Datum::new(reg.ctor_ref("Nat", 1).unwrap(), vec![CoreValue::Data(z())]);
        let b = Datum::new(reg.ctor_ref("Nat", 1).unwrap(), vec![CoreValue::Data(z())]);
        assert_eq!(a.content_hash(), b.content_hash());
        // Z vs S(Z) differ.
        assert_ne!(z().content_hash(), a.content_hash());
    }
}
