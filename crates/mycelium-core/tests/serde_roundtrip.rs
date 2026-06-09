//! M-104 — Core IR (de)serialization round-trips and schema-shape pinning.
//!
//! Two guarantees are checked here:
//!
//! 1. **Faithful round-trip** (RFC-0001 §4.8): `from_json(to_json(v)) == v`, including `Meta`, over
//!    a corpus spanning all four paradigms, every guarantee level, every bound kind/basis, every
//!    physical layout, and both provenance variants.
//! 2. **Schema agreement**: the serializer's output for representative values is *exactly* the
//!    committed `docs/spec/schemas/examples/value/valid/*.json` instances — and those files are what
//!    CI validates against `value.schema.json` (`scripts/checks/schema.sh`). So pinning emitted
//!    output to those files is what ties "the code emits schema-valid JSON" to a checked artifact.

use mycelium_core::bound::{Bound, BoundBasis, BoundKind, NormKind};
use mycelium_core::guarantee::GuaranteeStrength;
use mycelium_core::id::ContentHash;
use mycelium_core::meta::{Meta, PackScheme, PhysicalLayout, Provenance, SparsityObs};
use mycelium_core::repr::{Repr, ScalarKind, SparsityClass};
use mycelium_core::value::{Payload, Trit, Value};

fn hash(s: &str) -> ContentHash {
    ContentHash::parse(s).expect("valid content hash")
}

/// A spread of `(guarantee, bound)` pairs that satisfy M-I1…M-I4 — one per basis kind, plus Exact.
fn meta_variants() -> Vec<Meta> {
    let derived = Provenance::Derived {
        op: hash("blake3:op01"),
        inputs: vec![hash("blake3:in_a"), hash("blake3:in_b")],
    };
    vec![
        // Exact, no bound (M-I1).
        Meta::exact(Provenance::Root),
        // Proven + ProvenThm capacity bound (M-I2), with rich optional fields.
        Meta::new(
            derived.clone(),
            GuaranteeStrength::Proven,
            Some(Bound {
                kind: BoundKind::Capacity {
                    items: 3,
                    dim: 10_000,
                },
                basis: BoundBasis::ProvenThm {
                    citation: "Clarkson-Ubaru-Yang 2023".into(),
                },
            }),
            Some(SparsityObs {
                active: 97,
                density: 0.0097,
            }),
            Some(PhysicalLayout::VsaStore { sparse: true }),
            Some(hash("blake3:policy_x9")),
        )
        .unwrap(),
        // Empirical + EmpiricalFit error bound (M-I3), with a packed physical layout.
        Meta::new(
            Provenance::Root,
            GuaranteeStrength::Empirical,
            Some(Bound {
                kind: BoundKind::Error {
                    eps: 0.004,
                    norm: NormKind::L2,
                },
                basis: BoundBasis::EmpiricalFit {
                    trials: 10_000,
                    method: "Frady-Sommer Gaussian".into(),
                },
            }),
            None,
            Some(PhysicalLayout::TritPacked {
                scheme: PackScheme::Tl2,
            }),
            None,
        )
        .unwrap(),
        // Declared + UserDeclared probability bound (M-I4).
        Meta::new(
            Provenance::Root,
            GuaranteeStrength::Declared,
            Some(Bound {
                kind: BoundKind::Probability { delta: 0.01 },
                basis: BoundBasis::UserDeclared,
            }),
            None,
            None,
            None,
        )
        .unwrap(),
        // Declared crosstalk WITH a tail (exercises the optional field round-trip).
        Meta::new(
            Provenance::Root,
            GuaranteeStrength::Declared,
            Some(Bound {
                kind: BoundKind::Crosstalk {
                    expected: 0.02,
                    tail: Some(0.1),
                },
                basis: BoundBasis::UserDeclared,
            }),
            None,
            None,
            None,
        )
        .unwrap(),
        // Declared crosstalk WITHOUT a tail (the omitted-field path).
        Meta::new(
            Provenance::Root,
            GuaranteeStrength::Declared,
            Some(Bound {
                kind: BoundKind::Crosstalk {
                    expected: 0.02,
                    tail: None,
                },
                basis: BoundBasis::UserDeclared,
            }),
            None,
            Some(PhysicalLayout::DenseArray),
            None,
        )
        .unwrap(),
    ]
}

/// One value per paradigm, each paired with each `Meta` variant — the full round-trip corpus.
fn value_corpus() -> Vec<Value> {
    let reprs_payloads: Vec<(Repr, Payload)> = vec![
        (
            Repr::Binary { width: 8 },
            Payload::Bits(vec![true, false, true, true, false, false, true, false]),
        ),
        (
            Repr::Ternary { trits: 6 },
            Payload::Trits(vec![
                Trit::Zero,
                Trit::Neg,
                Trit::Zero,
                Trit::Zero,
                Trit::Pos,
                Trit::Zero,
            ]),
        ),
        (
            Repr::Dense {
                dim: 3,
                dtype: ScalarKind::Bf16,
            },
            Payload::Scalars(vec![0.5, -1.25, 2.0]),
        ),
        (
            Repr::Vsa {
                model: "MAP-I".into(),
                dim: 4,
                sparsity: SparsityClass::Sparse { max_active: 2 },
            },
            Payload::Hypervector(vec![1.0, 0.0, 0.0, -1.0]),
        ),
    ];
    let mut out = Vec::new();
    for (repr, payload) in reprs_payloads {
        for meta in meta_variants() {
            out.push(Value::new(repr.clone(), payload.clone(), meta).unwrap());
        }
    }
    out
}

#[test]
fn json_round_trip_is_faithful() {
    for v in value_corpus() {
        let json = serde_json::to_string(&v).expect("serialize");
        let back: Value = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(
            v, back,
            "round-trip must preserve the value incl. Meta\n{json}"
        );
        // Stable: re-serializing the round-tripped value yields byte-identical JSON.
        let json2 = serde_json::to_string(&back).expect("re-serialize");
        assert_eq!(json, json2, "serialization must be deterministic");
    }
}

#[test]
fn bf16_renders_as_bf16() {
    // The Rust spelling is `Bf16`; the wire/schema spelling is `BF16`.
    let v = Value::new(
        Repr::Dense {
            dim: 1,
            dtype: ScalarKind::Bf16,
        },
        Payload::Scalars(vec![1.0]),
        Meta::exact(Provenance::Root),
    )
    .unwrap();
    let j: serde_json::Value = serde_json::to_value(&v).unwrap();
    assert_eq!(j["repr"]["dtype"], "BF16");
}

/// Pin the serializer's output to the committed, CI-schema-validated example files. Comparing as
/// parsed `serde_json::Value` makes this insensitive to whitespace/key-order formatting.
fn assert_matches_example(value: &Value, example_path: &str) {
    let file = std::fs::read_to_string(example_path)
        .unwrap_or_else(|e| panic!("read {example_path}: {e}"));
    let from_file: serde_json::Value = serde_json::from_str(&file).expect("example parses");
    let emitted: serde_json::Value = serde_json::to_value(value).expect("serialize");
    assert_eq!(
        emitted, from_file,
        "serializer output must equal {example_path} (the schema-validated artifact)"
    );
    // And the committed file must deserialize back into the same value.
    let from_file_val: Value = serde_json::from_str(&file).expect("example deserializes to Value");
    assert_eq!(&from_file_val, value);
}

const EX: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/spec/schemas/examples/value/valid"
);

#[test]
fn emitted_value_matches_committed_examples() {
    let binary = Value::new(
        Repr::Binary { width: 8 },
        Payload::Bits(vec![true, false, true, true, false, false, true, false]),
        Meta::exact(Provenance::Root),
    )
    .unwrap();
    assert_matches_example(&binary, &format!("{EX}/binary-const.json"));

    let ternary = Value::new(
        Repr::Ternary { trits: 6 },
        Payload::Trits(vec![
            Trit::Zero,
            Trit::Neg,
            Trit::Zero,
            Trit::Zero,
            Trit::Pos,
            Trit::Zero,
        ]),
        Meta::exact(Provenance::Root),
    )
    .unwrap();
    assert_matches_example(&ternary, &format!("{EX}/ternary-const.json"));

    let dense = Value::new(
        Repr::Dense {
            dim: 3,
            dtype: ScalarKind::F32,
        },
        Payload::Scalars(vec![0.5, -1.25, 2.0]),
        Meta::exact(Provenance::Root),
    )
    .unwrap();
    assert_matches_example(&dense, &format!("{EX}/dense-const.json"));

    let vsa = Value::new(
        Repr::Vsa {
            model: "MAP-I".into(),
            dim: 4,
            sparsity: SparsityClass::Sparse { max_active: 2 },
        },
        Payload::Hypervector(vec![1.0, 0.0, 0.0, -1.0]),
        Meta::new(
            Provenance::Derived {
                op: hash("blake3:bundle_Op01"),
                inputs: vec![hash("blake3:hv_a01"), hash("blake3:hv_b02")],
            },
            GuaranteeStrength::Proven,
            Some(Bound {
                kind: BoundKind::Capacity {
                    items: 3,
                    dim: 10_000,
                },
                basis: BoundBasis::ProvenThm {
                    citation: "Clarkson-Ubaru-Yang 2023".into(),
                },
            }),
            Some(SparsityObs {
                active: 97,
                density: 0.0097,
            }),
            Some(PhysicalLayout::VsaStore { sparse: true }),
            Some(hash("blake3:policy_x9")),
        )
        .unwrap(),
    )
    .unwrap();
    assert_matches_example(&vsa, &format!("{EX}/vsa-proven-capacity.json"));
}

// --- Deserialization is never silently lenient: malformed wire forms are rejected. -------------

#[test]
fn rejects_payload_repr_mismatch() {
    // Binary{8} with a 2-bit payload — caught by Value::new on the way in.
    let json = r#"{ "repr": { "kind": "Binary", "width": 8 }, "payload": { "bits": "10" },
                   "meta": { "provenance": { "kind": "Root" }, "guarantee": "Exact" } }"#;
    assert!(serde_json::from_str::<Value>(json).is_err());
}

#[test]
fn rejects_exact_with_bound() {
    // M-I1 violation re-checked on deserialize.
    let json = r#"{ "repr": { "kind": "Binary", "width": 1 }, "payload": { "bits": "0" },
                   "meta": { "provenance": { "kind": "Root" }, "guarantee": "Exact",
                             "bound": { "kind": "ProbabilityBound", "delta": 0.1,
                                        "basis": { "kind": "UserDeclared" } } } }"#;
    assert!(serde_json::from_str::<Value>(json).is_err());
}

#[test]
fn rejects_declared_claiming_proven_basis() {
    // M-I4 violation: Declared cannot carry a ProvenThm basis.
    let json = r#"{ "repr": { "kind": "Binary", "width": 1 }, "payload": { "bits": "0" },
                   "meta": { "provenance": { "kind": "Root" }, "guarantee": "Declared",
                             "bound": { "kind": "CapacityBound", "items": 3, "dim": 9,
                                        "basis": { "kind": "ProvenThm", "citation": "x" } } } }"#;
    assert!(serde_json::from_str::<Value>(json).is_err());
}

#[test]
fn rejects_malformed_content_hash() {
    let json = r#"{ "repr": { "kind": "Binary", "width": 1 }, "payload": { "bits": "0" },
                   "meta": { "provenance": { "kind": "Derived", "op": "NOT A HASH", "inputs": [] },
                             "guarantee": "Exact" } }"#;
    assert!(serde_json::from_str::<Value>(json).is_err());
}

#[test]
fn rejects_bad_trit_glyph() {
    let json = r#"{ "repr": { "kind": "Ternary", "trits": 2 }, "payload": { "trits": "0x" },
                   "meta": { "provenance": { "kind": "Root" }, "guarantee": "Exact" } }"#;
    assert!(serde_json::from_str::<Value>(json).is_err());
}
