//! Unit tests for `map_type`'s concrete generic type-application mapping (E33-1 M-1006 phase-1) —
//! `Result<Duration, TimeErr>` -> `Result[Duration, TimeErr]` via the grammar's
//! `base_type ::= Ident type_args?` + `type_args ::= '[' type_ref (',' type_ref)* ']'`
//! (docs/spec/grammar/mycelium.ebnf lines 258 + 265, RFC-0037 D1 — square brackets, not `<>`).
//!
//! **Guarantee: `Declared`** — these assert the grammar-text mapping this module documents, not that
//! the emitted surface parses/typechecks in a real Mycelium toolchain (that is the vet loop's job).
//! Data-driven (per CLAUDE.md "Complex test logic lives in fixtures + parameterization"): a test body
//! is `assert over a case`, the cases live in a table.

use crate::gap::Category;
use crate::map::map_type;

/// Parse a Rust type from text (white-box fixture builder — `syn` is a dev+runtime dep here).
fn ty(text: &str) -> syn::Type {
    syn::parse_str::<syn::Type>(text)
        .unwrap_or_else(|e| panic!("fixture `{text}` is not a parseable Rust type: {e}"))
}

/// The expected outcome for one mapped-type case.
enum Expect {
    /// `map_type` returns this exact surface text.
    Ok(&'static str),
    /// `map_type` gaps with this category (the whole application refused — never a partial emission).
    Gap(Category),
}

struct Case {
    rust: &'static str,
    expect: Expect,
}

/// The mapped-generic-application corpus. Each row cites the behaviour it pins.
fn cases() -> Vec<Case> {
    use Category::*;
    use Expect::*;
    vec![
        // A 2-arg application whose args are ordinary named types (the exact real-corpus shape from
        // `gen/myc-drafts/stdlib/std-time` — `Result<Duration, TimeErr>`).
        Case {
            rust: "Result<Duration, TimeErr>",
            expect: Ok("Result[Duration, TimeErr]"),
        },
        // A mapped-builtin argument: `u8` -> `Binary{8}` inside the application.
        Case {
            rust: "Vec<u8>",
            expect: Ok("Vec[Binary{8}]"),
        },
        // A single-arg application (min `type_args` arity == 1).
        Case {
            rust: "Option<u32>",
            expect: Ok("Option[Binary{32}]"),
        },
        // A *nested* application recurses through the public `map_type` (budget re-arms per level).
        Case {
            rust: "Result<Option<u32>, E>",
            expect: Ok("Result[Option[Binary{32}], E]"),
        },
        // Deeper nesting + a mapped builtin at the leaf.
        Case {
            rust: "Box<Vec<u16>>",
            expect: Ok("Box[Vec[Binary{16}]]"),
        },
        // An unmappable *type* argument gaps the whole application — `String` has no confirmed
        // base_type arm, and its precise inner `GapReason` (category `Other`) propagates unchanged
        // (never a partial `Option[..]` emission). This is the tuple-arm propagation precedent.
        Case {
            rust: "Option<String>",
            expect: Gap(Other),
        },
        // A signed-integer argument likewise propagates its own (Other) gap — Binary{N} is unsigned.
        Case {
            rust: "Vec<i32>",
            expect: Gap(Other),
        },
        // A lifetime argument has no `type_ref` surface -> GenericBound gap for the whole path.
        Case {
            rust: "Ref<'a, T>",
            expect: Gap(GenericBound),
        },
        // A const-generic argument likewise -> GenericBound gap (not a type_ref).
        Case {
            rust: "Arr<T, 4>",
            expect: Gap(GenericBound),
        },
        // A reserved-word HEAD still gaps (ReservedWord) — never emit un-lexable text, even for a
        // fully-mappable argument list. `Exact` is a guarantee-strength keyword (crate::reserved).
        Case {
            rust: "Exact<u8>",
            expect: Gap(Category::ReservedWord),
        },
        // A reserved *repr* keyword head (`Seq`) also gaps rather than emitting `Seq[..]` (the
        // transpiler has no sanctioned rename — VR-5/G2).
        Case {
            rust: "Seq<u8>",
            expect: Gap(Category::ReservedWord),
        },
    ]
}

fn run(case: &Case) {
    let mapped = map_type(&ty(case.rust), None);
    match &case.expect {
        Expect::Ok(surface) => {
            let got = mapped.unwrap_or_else(|e| {
                panic!(
                    "case `{}`: expected Ok(`{surface}`), got gap [{}] {}",
                    case.rust,
                    e.category.as_str(),
                    e.reason
                )
            });
            assert_eq!(
                &got, surface,
                "case `{}`: mapped surface mismatch",
                case.rust
            );
        }
        Expect::Gap(category) => {
            let err = mapped.expect_err(&format!(
                "case `{}`: expected a gap of category {:?}, got Ok",
                case.rust,
                category.as_str()
            ));
            assert_eq!(
                err.category, *category,
                "case `{}`: gap category mismatch — reason was: {}",
                case.rust, err.reason
            );
        }
    }
}

#[test]
fn generic_application_mapping_corpus() {
    for case in cases() {
        run(&case);
    }
}

/// Regression guard (G2 / never-silent): a gapped generic application must return **no** surface
/// text at all — a lifetime-arg refusal must not leak a partial `Ref[..]` emission, and the reason
/// must name the offending construct (here, the lifetime).
#[test]
fn lifetime_arg_gap_is_total_and_named() {
    let err = map_type(&ty("Ref<'a, T>"), None)
        .expect_err("a lifetime type-argument must gap the whole application");
    assert_eq!(err.category, Category::GenericBound);
    assert!(
        err.reason.contains("not a type"),
        "the gap reason must explain the refusal (never silent): {}",
        err.reason
    );
}

/// The bare (zero-argument) named-type pass-through is unchanged by the generic-application arm — a
/// plain `Duration` still maps to itself, so the new arm did not regress the existing row.
#[test]
fn bare_named_type_still_passes_through() {
    assert_eq!(map_type(&ty("Duration"), None).unwrap(), "Duration");
}

/// A qualified multi-segment generic path stays gapped (the `segments.len() > 1` arm owns it — the
/// new single-segment arm must not weaken it): `std::result::Result<T, E>` is still an `Other` gap.
#[test]
fn qualified_generic_path_still_gapped() {
    let err = map_type(&ty("std::result::Result<u8, E>"), None)
        .expect_err("a qualified multi-segment generic path must stay gapped");
    assert_eq!(err.category, Category::Other);
}
