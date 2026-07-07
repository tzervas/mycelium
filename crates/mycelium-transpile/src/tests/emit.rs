//! Unit tests for the `.myc` emitter, over a small fixture corpus (data-driven — per CLAUDE.md
//! "Complex test logic lives in fixtures + parameterization, not in test bodies").

use crate::gap::Category;
use crate::transpile::transpile_source;

/// The expected outcome for one fixture.
enum Expect {
    /// The item is emitted, and the `.myc` text contains this substring.
    Emitted {
        item: &'static str,
        contains: &'static str,
    },
    /// The item is not emitted at all, and at least one gap of this category is recorded.
    Gapped { category: Category },
    /// The item is emitted (containing the substring) AND at least one sub-gap of the given
    /// category is also recorded for it (e.g. a dropped `#[derive(..)]`).
    EmittedAndGapped {
        item: &'static str,
        contains: &'static str,
        sub_gap_category: Category,
    },
}

struct Case {
    name: &'static str,
    rust: &'static str,
    expect: Expect,
}

/// The fixture corpus. Each row cites the grammar production it exercises.
fn cases() -> Vec<Case> {
    vec![
        // `type_item`: C-like enum -> a sum type (grammar §type_item/constructor).
        Case {
            name: "c_like_enum",
            rust: "enum Ordering { Less, Equal, Greater }",
            expect: Expect::Emitted {
                item: "Ordering",
                contains: "type Ordering = Less | Equal | Greater;",
            },
        },
        // `fn_item`: a single-expression body (grammar §fn_item).
        Case {
            name: "simple_fn",
            rust: "fn is_lt(o: bool) -> bool { o }",
            expect: Expect::Emitted {
                item: "is_lt",
                contains: "fn is_lt(o: Bool) => Bool = o;",
            },
        },
        // `match_expr` over bool literal patterns (grammar §match_expr/pattern).
        Case {
            name: "match_expr",
            rust: "fn pick(o: bool) -> bool { match o { true => false, false => true } }",
            expect: Expect::Emitted {
                item: "pick",
                contains: "match o { True => False, False => True }",
            },
        },
        // A `let`-chain + tail expr desugars to nested `let ... in ...` (still a single
        // `fn_item` body expression).
        Case {
            name: "let_chain_body",
            rust: "fn double(x: bool) -> bool { let y = x; y }",
            expect: Expect::Emitted {
                item: "double",
                contains: "let y = x in y",
            },
        },
        // Tuple-variant enum: positional fields map via `constructor`'s optional field list.
        Case {
            name: "tuple_variant_enum",
            rust: "enum Foo { A(u8), B }",
            expect: Expect::Emitted {
                item: "Foo",
                contains: "type Foo = A(Binary{8}) | B;",
            },
        },
        // A tuple struct maps to a single-constructor `type_item`.
        Case {
            name: "tuple_struct",
            rust: "struct Bf16Bits(u16);",
            expect: Expect::Emitted {
                item: "Bf16Bits",
                contains: "type Bf16Bits = Bf16Bits(Binary{16});",
            },
        },
        // KNOWN HARD GAP: `trait` — every realistic trait in the target crate gaps (default
        // bodies, supertraits, or an unresolvable `Self`); this fixture exercises the
        // unresolvable-`self` path specifically (no default body, no supertrait).
        Case {
            name: "trait_self_unresolvable",
            rust: "trait Foo { fn bar(&self) -> bool; }",
            expect: Expect::Gapped {
                category: Category::Trait,
            },
        },
        // KNOWN HARD GAP: `macro_rules!` definitions — no macro system in the grammar.
        Case {
            name: "macro_rules_gap",
            rust: "macro_rules! foo { () => {}; }",
            expect: Expect::Gapped {
                category: Category::MacroDef,
            },
        },
        // Item-position macro invocations are a distinct category from macro *definitions*.
        Case {
            name: "macro_invocation_gap",
            rust: "some_macro!(a, b, c);",
            expect: Expect::Gapped {
                category: Category::MacroInvocation,
            },
        },
        // KNOWN HARD GAP: a named-field ("record") struct — no record surface in `constructor`.
        Case {
            name: "struct_named_fields_gap",
            rust: "struct Foo { x: u8 }",
            expect: Expect::Gapped {
                category: Category::Struct,
            },
        },
        // M-873 follow-on (DN-41): a numeric-widening `impl Widen<..> for ..` whose body is a
        // qualified associated-function call (`u16::from(self)`, the real shape of Rust's
        // widening bodies in `mycelium-std-cmp`) must never be emitted with the *fabricated*
        // `from(self)` text (`from` is not a Mycelium builtin — no grammar production; only prose
        // mentions in `docs/spec/grammar/mycelium.ebnf`). Once both `Self`/target map to
        // `Binary{N}`/`Binary{M}` (unsigned widening), it is now instead emitted **faithfully**
        // via the real DN-41 `width_cast` prim — a strict improvement over the earlier "gap the
        // whole impl" behavior this case originally pinned (see
        // `widen_impls_never_fabricate_from_in_real_crate` in `src/tests/diff.rs` for the
        // real-crate-scale version of this guard).
        Case {
            name: "widen_binary_emits_width_cast",
            rust: "impl Widen<u16> for u8 { fn widen(self) -> u16 { u16::from(self) } }",
            expect: Expect::Emitted {
                item: "impl Widen[Binary{16}] for Binary{8}",
                contains: "width_cast(self, 0b0000_0000_0000_0000)",
            },
        },
        // Widen over a non-`Binary` `Self` (e.g. `bool`) has no `width_cast` witness path (`Self`
        // doesn't map to `Binary{N}` at all) — the qualified `u32::from(self)` call stays an
        // honest gap, unchanged from the pre-DN-41 behavior.
        Case {
            name: "widen_bool_from_call_still_gapped_not_fabricated",
            rust: "impl Widen<u32> for bool { fn widen(self) -> u32 { u32::from(self) } }",
            expect: Expect::Gapped {
                category: Category::Impl,
            },
        },
        // DN-41 §2: `Narrow::narrow` is fallible (`Result<To, NarrowError>`) — no `= expr
        // fn_item` body can express a Result-returning refuse, so it stays an explicit,
        // DN-41-cited gap rather than a forced/fabricated emission.
        Case {
            name: "narrow_gapped_cites_dn41",
            rust: "impl Narrow<u8> for u16 { fn narrow(self) -> Result<u8, NarrowError> { u8::try_from(self) } }",
            expect: Expect::Gapped {
                category: Category::Impl,
            },
        },
        // KNOWN HARD GAP: multi-statement fn body (an interior statement that is neither a
        // simple `let` nor the trailing expression).
        Case {
            name: "multi_stmt_body_gap",
            rust: "fn foo(x: bool) -> bool { let y = x; println!(\"{}\", 1); y }",
            expect: Expect::Gapped {
                category: Category::MultiStmtBody,
            },
        },
        // A string literal maps to a `StrLit` (grammar line 414/430; M-910/M-911) — reachable in
        // an emittable body as a call argument (its type is inferred, not named). The Rust `\n`
        // decodes to a raw newline which is re-escaped back to `\n` in the emitted StrLit.
        Case {
            name: "string_literal_arg_emits_strlit",
            rust: "fn f(x: u8) -> u8 { g(x, \"hi\\n\") }",
            expect: Expect::Emitted {
                item: "f",
                contains: "g(x, \"hi\\n\")",
            },
        },
        // A float literal maps to a `FloatLit` (grammar line 414/443; ADR-040/M-897) when its
        // digit string is a well-formed, finite FloatLit — reachable as a call argument.
        Case {
            name: "float_literal_arg_emits_floatlit",
            rust: "fn f(x: u8) -> u8 { g(x, 1.5) }",
            expect: Expect::Emitted {
                item: "f",
                contains: "g(x, 1.5)",
            },
        },
        // An exponent-form float likewise maps (`syn` normalizes `E`→`e`, drops the `+`).
        Case {
            name: "float_exponent_arg_emits_floatlit",
            rust: "fn f(x: u8) -> u8 { g(x, 2.5E+3) }",
            expect: Expect::Emitted {
                item: "f",
                contains: "g(x, 2.5e3)",
            },
        },
        // An explicit-element array maps to a `ListLit` (grammar line 415; RFC-0032 D3) —
        // reachable as a call argument.
        Case {
            name: "array_literal_arg_emits_listlit",
            rust: "fn f(x: u8) -> u8 { g(x, [x, x]) }",
            expect: Expect::Emitted {
                item: "f",
                contains: "g(x, [x, x])",
            },
        },
        // KNOWN HARD GAP: a string literal carrying a control char with no Mycelium escape
        // (`\x07` bell) — StrLit has no `\xNN` form, so it is never-silently gapped, never emitted
        // as a raw byte (G2/VR-5).
        Case {
            name: "string_control_char_gapped",
            rust: "fn f(x: u8) -> u8 { g(x, \"\\x07\") }",
            expect: Expect::Gapped {
                category: Category::Other,
            },
        },
        // KNOWN HARD GAP: a Rust-only float shape (trailing-dot `2.` → digit string "2.", empty
        // fraction) has no faithful Mycelium FloatLit spelling — gapped rather than reshaped (VR-5).
        Case {
            name: "float_trailing_dot_gapped",
            rust: "fn f(x: u8) -> u8 { g(x, 2.) }",
            expect: Expect::Gapped {
                category: Category::Other,
            },
        },
        // KNOWN HARD GAP: a well-shaped float whose value is not finite binary64 (`1e999` → +inf)
        // — a literal is a conversion boundary, so out-of-range is a never-silent refuse, never a
        // silent ±inf (ADR-040 §2.4).
        Case {
            name: "float_non_finite_gapped",
            rust: "fn f(x: u8) -> u8 { g(x, 1e999) }",
            expect: Expect::Gapped {
                category: Category::Other,
            },
        },
        // KNOWN HARD GAP: an array-repeat `[x; N]` — `ListLit` has no repeat form.
        Case {
            name: "array_repeat_gapped",
            rust: "fn f(x: u8) -> u8 { g(x, [x; 4]) }",
            expect: Expect::Gapped {
                category: Category::Other,
            },
        },
        // A bounded generic type parameter has no bare-identifier `type_params` mapping.
        Case {
            name: "generic_bound_gap",
            rust: "fn foo<T: Clone>(x: T) -> T { x }",
            expect: Expect::Gapped {
                category: Category::GenericBound,
            },
        },
        // A named-field enum variant is a payload-shape gap distinct from a whole-struct gap.
        Case {
            name: "payload_variant_named_fields_gap",
            rust: "enum Foo { A { x: u8 } }",
            expect: Expect::Gapped {
                category: Category::PayloadVariant,
            },
        },
        // `#[derive(..)]` (any non-doc attribute) is dropped but recorded — the item is still
        // emitted (structural mapping doesn't need the derive), with a DeriveAttr sub-gap.
        Case {
            name: "derive_attr_sub_gap",
            rust: "#[derive(Debug, Clone)]\nenum Foo { A, B }",
            expect: Expect::EmittedAndGapped {
                item: "Foo",
                contains: "type Foo = A | B;",
                sub_gap_category: Category::DeriveAttr,
            },
        },
        // M-1001: a `use` import is FLAGGED, not emitted — the transpiler has no cross-nodule symbol
        // table so it cannot confirm the path resolves (the vet loop confirms such imports fail
        // `myc check` name-resolution), and an emitted `use` poisons the whole draft's check.
        Case {
            name: "simple_use_gapped",
            rust: "use a::b::C;",
            expect: Expect::Gapped {
                category: Category::Import,
            },
        },
        // Grouped `use` is likewise an Import gap.
        Case {
            name: "grouped_use_gap",
            rust: "use a::{b, c};",
            expect: Expect::Gapped {
                category: Category::Import,
            },
        },
        // M-1001: a type whose name is a Mycelium reserved word (`Float`) can't be emitted verbatim
        // (it would lex as a keyword) — gapped ReservedWord, never renamed (VR-5/G2).
        Case {
            name: "reserved_type_name",
            rust: "enum Float { A, B }",
            expect: Expect::Gapped {
                category: Category::ReservedWord,
            },
        },
        // M-1001: a variant/constructor named a reserved word (`Exact`) — the collision that poisoned
        // `mycelium-l1/src/eval.rs`'s parse in the §8.7 baseline.
        Case {
            name: "reserved_variant",
            rust: "enum GuaranteeStrength { Exact, Loose }",
            expect: Expect::Gapped {
                category: Category::ReservedWord,
            },
        },
        // Shared-reference erasure (this leaf, ADR-003): a fn whose params are `&T` shared references
        // now maps — the references are erased so the signature becomes value params, exactly as the
        // hand-port renders it. This is the item-level effect that unblocks emission (the real-corpus
        // shape: `fn digest_eq(a: &ContentHash, b: &ContentHash) -> bool`).
        Case {
            name: "shared_ref_params_emit",
            rust: "fn digest_eq(a: &Ordering, b: &Ordering) -> bool { a == b }",
            expect: Expect::Emitted {
                item: "digest_eq",
                contains: "fn digest_eq(a: Ordering, b: Ordering) => Bool = a == b;",
            },
        },
        // NEVER-SILENT CASCADE: a fn taking `&mut T` stays gapped — a mutable reference has no
        // value-semantic correspondence (ADR-003), so it is NOT erased. The whole fn gaps (Other),
        // never a partial emission that silently drops the mutation.
        Case {
            name: "mut_ref_param_gapped",
            rust: "fn bump(x: &mut Ordering) -> bool { true }",
            expect: Expect::Gapped {
                category: Category::Other,
            },
        },
        // NEVER-SILENT CASCADE: a fn taking `&str` still gaps — the reference erases but the referent
        // `str` has no confirmed base_type arm, so the honest deeper blocker surfaces (Other), never
        // a fabricated emission.
        Case {
            name: "shared_ref_to_str_still_gapped",
            rust: "fn is_err(msg: &str) -> bool { true }",
            expect: Expect::Gapped {
                category: Category::Other,
            },
        },
    ]
}

fn run(case: &Case) {
    let (myc, report) = transpile_source(case.rust, "fixture.rs", "fixture")
        .unwrap_or_else(|e| panic!("case `{}` failed to parse/transpile: {e}", case.name));
    match &case.expect {
        Expect::Emitted { item, contains } => {
            assert!(
                report.emitted_items.iter().any(|n| n == item),
                "case `{}`: expected `{item}` in emitted_items, got {:?}",
                case.name,
                report.emitted_items
            );
            assert!(
                myc.contains(contains),
                "case `{}`: expected .myc to contain `{contains}`, got:\n{myc}",
                case.name
            );
        }
        Expect::Gapped { category } => {
            assert!(
                report.emitted_items.is_empty(),
                "case `{}`: expected no emitted items, got {:?}",
                case.name,
                report.emitted_items
            );
            assert!(
                report.gaps.iter().any(|g| g.category == *category),
                "case `{}`: expected a gap of category {:?}, got {:?}",
                case.name,
                category.as_str(),
                report
                    .gaps
                    .iter()
                    .map(|g| g.category.as_str())
                    .collect::<Vec<_>>()
            );
        }
        Expect::EmittedAndGapped {
            item,
            contains,
            sub_gap_category,
        } => {
            assert!(
                report.emitted_items.iter().any(|n| n == item),
                "case `{}`: expected `{item}` in emitted_items, got {:?}",
                case.name,
                report.emitted_items
            );
            assert!(
                myc.contains(contains),
                "case `{}`: expected .myc to contain `{contains}`, got:\n{myc}",
                case.name
            );
            assert!(
                report.gaps.iter().any(|g| g.category == *sub_gap_category),
                "case `{}`: expected a sub-gap of category {:?}, got {:?}",
                case.name,
                sub_gap_category.as_str(),
                report
                    .gaps
                    .iter()
                    .map(|g| g.category.as_str())
                    .collect::<Vec<_>>()
            );
        }
    }
}

#[test]
fn emit_fixture_corpus() {
    for case in cases() {
        run(&case);
    }
}

/// Regression guard (High finding, G2/DN-34 §4, extended by DN-41/M-873 follow-on): the
/// never-silent gap mechanism means a *gapped* item's `.myc` text is never emitted at all — pin
/// that down directly for the bool-`Self` widen shape (which still has no `width_cast` witness
/// path — `Self` doesn't map to `Binary{N}`) so a future change that started emitting a
/// partial/fallback body for this case would fail loudly here, not just leave `emitted_items`
/// empty while still leaking fabricated text into the `.myc` output.
#[test]
fn widen_bool_from_call_produces_no_fabricated_myc_text() {
    let rust = "impl Widen<u32> for bool { fn widen(self) -> u32 { u32::from(self) } }";
    let (myc, report) = transpile_source(rust, "fixture.rs", "fixture")
        .unwrap_or_else(|e| panic!("failed to parse/transpile: {e}"));
    assert!(
        report.emitted_items.is_empty(),
        "expected the bool Widen impl to be fully gapped, got emitted_items={:?}",
        report.emitted_items
    );
    assert!(
        !myc.contains("from("),
        "emitted .myc text must never contain a fabricated `from(...)` call (from is not a \
         Mycelium builtin — G2/DN-34 §4), got:\n{myc}"
    );
}

/// The DN-41 companion of the guard above: a `Binary{N}`->`Binary{M}` widen must emit a **real**
/// `width_cast(self, ..)` call — never a fabricated `from(...)` call, and never left gapped now
/// that the faithful mapping exists.
#[test]
fn widen_binary_emits_width_cast_not_fabricated_from() {
    let rust = "impl Widen<u16> for u8 { fn widen(self) -> u16 { u16::from(self) } }";
    let (myc, report) = transpile_source(rust, "fixture.rs", "fixture")
        .unwrap_or_else(|e| panic!("failed to parse/transpile: {e}"));
    assert!(
        report
            .emitted_items
            .iter()
            .any(|n| n == "impl Widen[Binary{16}] for Binary{8}"),
        "expected the Binary widen impl to be emitted via width_cast, got emitted_items={:?}",
        report.emitted_items
    );
    assert!(
        !myc.contains("from("),
        "emitted .myc text must never contain a fabricated `from(...)` call (from is not a \
         Mycelium builtin — G2/DN-34 §4), got:\n{myc}"
    );
    assert!(
        myc.contains("width_cast(self, 0b0000_0000_0000_0000)"),
        "expected a real `width_cast(self, ..)` call with a 16-bit zero witness, got:\n{myc}"
    );
}

/// DN-41 companion: `Narrow::narrow` is fallible and has no `= expr` surface, so it must stay an
/// honest gap whose reason cites DN-41 — never a fabricated `try_from`/`?`-shaped emission.
#[test]
fn narrow_gap_cites_dn41_and_produces_no_fabricated_myc_text() {
    let rust = "impl Narrow<u8> for u16 { fn narrow(self) -> Result<u8, NarrowError> { u8::try_from(self) } }";
    let (myc, report) = transpile_source(rust, "fixture.rs", "fixture")
        .unwrap_or_else(|e| panic!("failed to parse/transpile: {e}"));
    assert!(
        report.emitted_items.is_empty(),
        "expected the Narrow impl to be fully gapped, got emitted_items={:?}",
        report.emitted_items
    );
    assert!(
        !myc.contains("try_from") && !myc.contains("width_cast"),
        "narrow bodies must never be fabricated (no try_from-shaped or width_cast emission), \
         got:\n{myc}"
    );
    assert!(
        report.gaps.iter().any(|g| g.reason.contains("DN-41")),
        "expected the narrow gap's reason to cite DN-41, got {:?}",
        report.gaps.iter().map(|g| &g.reason).collect::<Vec<_>>()
    );
}

/// Never-silent guard (G2/VR-5): a string literal that cannot be faithfully re-escaped (a control
/// char with no Mycelium `\xNN`/`\u{..}` form) is gapped, and its raw byte NEVER leaks into the
/// emitted `.myc` text — a future change that started emitting the raw control byte (or a fabricated
/// `\x07` escape Mycelium's lexer would reject) would fail loudly here.
#[test]
fn string_control_char_never_leaks_raw_byte() {
    let rust = "fn f(x: u8) -> u8 { g(x, \"\\x07\") }";
    let (myc, report) = transpile_source(rust, "fixture.rs", "fixture")
        .unwrap_or_else(|e| panic!("failed to parse/transpile: {e}"));
    assert!(
        report.emitted_items.is_empty(),
        "expected the control-char string body to be fully gapped, got emitted_items={:?}",
        report.emitted_items
    );
    assert!(
        !myc.contains('\u{7}') && !myc.contains("\\x07"),
        "gapped control-char string must never leak a raw byte or a fabricated `\\x07` escape \
         (StrLit has no `\\xNN` form), got:\n{myc}"
    );
}

/// The sharpened `MultiStmtBody` reason (this leaf, E33-1 M-1006 phase-1) names the *kind* of the
/// offending interior statement — a nested item (local `static`/`const`/`fn`), a macro invocation,
/// or a semicolon-terminated statement expression — so the gap report is precise, not generic
/// (G2). Each is a genuinely design-blocked form (no local-item / no macro / value-discard has no
/// grammar surface); this pins the diagnostic text, not any emission.
#[test]
fn multi_stmt_body_reason_names_the_statement_kind() {
    let cases = [
        // A local `static` item statement (the real `mono_nanos` shape).
        (
            "fn f(x: u8) -> u8 { static Z: u8 = 0; x }",
            "nested item declaration",
        ),
        // A macro-invocation statement (the real `rejection_sample_u64` `debug_assert!` shape).
        (
            "fn f(x: u8) -> u8 { debug_assert!(x > 0); x }",
            "macro-invocation statement",
        ),
        // A semicolon-terminated (value-discarding) statement expression.
        (
            "fn f(x: u8) -> u8 { g(x); x }",
            "semicolon-terminated (value-discarding) statement expression",
        ),
    ];
    for (rust, needle) in cases {
        let (_, report) = transpile_source(rust, "fixture.rs", "fixture")
            .unwrap_or_else(|e| panic!("failed to parse/transpile `{rust}`: {e}"));
        assert!(
            report
                .gaps
                .iter()
                .any(|g| g.category == Category::MultiStmtBody && g.reason.contains(needle)),
            "case `{rust}`: expected a MultiStmtBody gap whose reason mentions `{needle}`, got {:?}",
            report
                .gaps
                .iter()
                .map(|g| (g.category.as_str(), g.reason.as_str()))
                .collect::<Vec<_>>()
        );
    }
}
