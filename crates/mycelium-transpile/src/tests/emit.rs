//! Unit tests for the `.myc` emitter, over a small fixture corpus (data-driven — per CLAUDE.md
//! "Complex test logic lives in fixtures + parameterization, not in test bodies").

use crate::emit::{emit_expr, TypeEnv};
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
        // M-1006 (E33-1): a named-field ("record") struct whose fields all resolve in-file now emits
        // POSITIONALLY (field names dropped + recorded via a `NamedFieldDrop` sub-gap) — the
        // grammar-grounded mapping the `lib/std/*.myc` hand-ports use (`type GuaranteeRow = Row(..)`).
        Case {
            name: "struct_named_fields_emits_positionally",
            rust: "struct Foo { x: u8, y: bool }",
            expect: Expect::EmittedAndGapped {
                item: "Foo",
                contains: "type Foo = Foo(Binary{8}, Bool)",
                sub_gap_category: Category::NamedFieldDrop,
            },
        },
        // M-1006 §8.14: a named-field struct with a `String` field now EMITS — `String` maps to
        // `Bytes` (RFC-0033 §3.2), so the record is fully mappable and emits positionally.
        Case {
            name: "struct_named_field_string_maps_to_bytes",
            rust: "struct WithText { s: String, n: u32 }",
            expect: Expect::EmittedAndGapped {
                item: "WithText",
                contains: "type WithText = WithText(Bytes, Binary{32})",
                sub_gap_category: Category::NamedFieldDrop,
            },
        },
        // M-1006: a named-field struct with an UNMAPPABLE field type (`f32`) still gaps — the
        // field's own precise repr reason wins (mapped *before* the resolvability gate), so the gap
        // profile keeps "unmappable field" distinct from "out-of-file reference". (P4/P5, DN-99 §8
        // ENB-6: `char` itself now maps to `Binary{32}`, so this fixture moved to `f32` — still
        // genuinely unmapped, `Float` being binary64-only — to keep exercising the still-gapped
        // case.)
        Case {
            name: "struct_named_field_unmappable_type_still_gaps",
            rust: "struct Bad { c: f32 }",
            expect: Expect::Gapped {
                category: Category::Struct,
            },
        },
        // M-1006 resolvability gate: a named-field struct whose fields all MAP but reference a type
        // not declared in this file (`Elsewhere`) is gated — emitting it would introduce an
        // unresolved reference that poisons the file's `myc check`. Left an honest `Struct` gap.
        Case {
            name: "struct_named_field_out_of_file_ref_is_gated",
            rust: "struct Ref { h: Elsewhere }",
            expect: Expect::Gapped {
                category: Category::Struct,
            },
        },
        // M-1006 greatest-fixpoint: mutually-recursive named-field structs (`A` <-> `B`) resolve as a
        // group and emit — a *least* fixpoint would wrongly gate both (each waits on the other). Both
        // are declared in-file and reference only each other + builtins, so the cycle is resolvable.
        Case {
            name: "mutually_recursive_named_structs_resolve",
            rust: "struct A { b: B, x: u8 }\nstruct B { a: A }",
            expect: Expect::EmittedAndGapped {
                item: "A",
                contains: "type A = A(B, Binary{8})",
                sub_gap_category: Category::NamedFieldDrop,
            },
        },
        // M-1006 Lever 1: a `self.<field>` projection in an impl body desugars to a `match` on the
        // struct's single (positional) constructor — the faithful equivalent (no projection surface in
        // the grammar). `Perm` is resolvable (its ctor emits), so the projection is gated ON.
        Case {
            name: "field_projection_desugars_to_match",
            rust: "struct Perm { mode: u8 }\nimpl Perm { fn get(self) -> u8 { self.mode } }",
            expect: Expect::Emitted {
                item: "impl Perm",
                contains: "match self { Perm(p0) => p0 }",
            },
        },
        // M-1006 Lever 1: struct-literal construction `Foo { mode: a }` -> the positional ctor call
        // `Foo(a)` (fields ordered by declaration). `Self { .. }` resolves the same way in impl context.
        Case {
            name: "struct_literal_construction_emits_positional_ctor",
            rust: "struct Foo { mode: u8 }\nfn mk(a: u8) -> Foo { Foo { mode: a } }",
            expect: Expect::Emitted {
                item: "mk",
                contains: "Foo(a)",
            },
        },
        // M-1006 Lever 1 gate: a field access on a NON-`self` base gaps — the transpiler tracks no
        // local types, so it cannot resolve the projection to a constructor position (never a guess).
        // (No struct is declared here, so the sole item is the gapping `peek`.)
        Case {
            name: "field_access_on_non_self_base_gaps",
            rust: "fn peek(f: u8) -> u8 { f.mode }",
            expect: Expect::Gapped {
                category: Category::Other,
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
        // M-1006 (E33-1): a named-field enum variant whose fields resolve now emits POSITIONALLY
        // (`A { x: u8 }` -> `A(Binary{8})`), names dropped + recorded via a `NamedFieldDrop` sub-gap.
        Case {
            name: "payload_variant_named_fields_emits_positionally",
            rust: "enum Foo { A { x: u8 }, B }",
            expect: Expect::EmittedAndGapped {
                item: "Foo",
                contains: "type Foo = A(Binary{8}) | B",
                sub_gap_category: Category::NamedFieldDrop,
            },
        },
        // M-1006 §8.14: a named-field variant with a `String` field now EMITS — `String` maps to
        // `Bytes` (RFC-0033 §3.2), names dropped + recorded via a `NamedFieldDrop` sub-gap.
        Case {
            name: "payload_variant_string_field_maps_to_bytes",
            rust: "enum Msg { Text { s: String }, Empty }",
            expect: Expect::EmittedAndGapped {
                item: "Msg",
                contains: "type Msg = Text(Bytes) | Empty",
                sub_gap_category: Category::NamedFieldDrop,
            },
        },
        // M-1006: a named-field variant with an UNMAPPABLE field type (`char`) still gaps — the
        // variant's own precise reason wins (mapped before the resolvability gate). (P4/P5,
        // DN-99 §8 ENB-6: `char` itself now maps to `Binary{32}`, so this fixture moved to `f32`
        // — still genuinely unmapped, `Float` being binary64-only — to keep exercising the
        // still-gapped case.)
        Case {
            name: "payload_variant_unmappable_field_still_gaps",
            rust: "enum Bad { A { c: f32 } }",
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
        // DN-125 (M-1081): a fn taking a top-level `&mut T` parameter now value-threads (Alt A,
        // Rank 1) instead of hard-gapping — `x` erases to a by-value `Ordering` param, and the
        // return type widens to carry `x` back out alongside the genuine `bool` return value
        // (this fixture's body never actually reassigns `x`, so the threaded slot is just `x`
        // itself, unchanged — a vacuously-correct rebind, not a special case; `map_signature`'s
        // `FnArg::Typed` `&mut T` arm does not require the body to mutate). Was
        // `mut_ref_param_gapped` pre-DN-125; kept the same fixture Rust source so the two
        // behaviors (gap -> emit) are directly comparable in history.
        Case {
            name: "mut_ref_param_value_threads",
            rust: "fn bump(x: &mut Ordering) -> bool { true }",
            expect: Expect::Emitted {
                item: "bump",
                contains: "fn bump(x: Ordering) => (Ordering, Bool) = (x, True);",
            },
        },
        // M-1006 §8.14: a fn taking `&str` now emits — the reference erases to `str`, which maps to
        // `Bytes` (RFC-0033 §3.2). The real-corpus shape `fn message(&self) -> &str` (a String/`str`
        // accessor) is the class this unblocks.
        Case {
            name: "shared_ref_to_str_emits_bytes",
            rust: "fn tag(msg: &str) -> bool { true }",
            expect: Expect::Emitted {
                item: "tag",
                contains: "fn tag(msg: Bytes) => Bool = True;",
            },
        },
        // NEVER-SILENT CASCADE: a fn taking `&f32` still gaps — the reference erases but the referent
        // `f32` has no confirmed base_type arm, so the honest deeper blocker surfaces (Other), never
        // a fabricated emission. (P4/P5, DN-99 §8 ENB-6: `char` itself now maps to `Binary{32}`, so
        // this fixture moved to `f32` — still genuinely unmapped — to keep exercising the cascade.)
        Case {
            name: "shared_ref_to_unmappable_referent_still_gapped",
            rust: "fn is_err(c: &f32) -> bool { true }",
            expect: Expect::Gapped {
                category: Category::Other,
            },
        },
        // ── trx2 Lane C Deliverable 1: operand-type-gated operator emission ─────────────────────
        // (verify-first, mitigation #14 — every surface name below was confirmed against the real
        // built `target/debug/myc`/`target/debug/myc-check` toolchain; see this module's
        // `binop_operand_gated_forms_check_clean` live-oracle test for the `myc check`-clean
        // proof, and `emit.rs`'s `Expr::Binary` arm doc for the full citation trail.)
        //
        // Both operands are known `Binary{16}` params (from `MappedSig::params` via `sig_type_env`)
        // -> `&`/`|` rewrite to the bare-call prim forms `and`/`or` (the glyph desugar target
        // `band`/`bor` is NOT a prim — `myc check`-confirmed to fail with no import).
        Case {
            name: "bitand_known_binary_emits_and_call",
            rust: "fn f(a: u16, b: u16) -> u16 { a & b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "and(a, b)",
            },
        },
        Case {
            name: "bitor_known_binary_emits_or_call",
            rust: "fn f(a: u16, b: u16) -> u16 { a | b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "or(a, b)",
            },
        },
        // `^` is already the correct prim name after the parser's glyph desugar (`Tok::Caret` ->
        // word `"xor"`, which IS a bare-call prim) — left as the unchanged glyph; no rewrite.
        Case {
            name: "bitxor_known_binary_stays_glyph",
            rust: "fn f(a: u16, b: u16) -> u16 { a ^ b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "a ^ b",
            },
        },
        // `!=`/`>` desugar to `ne`/`gt`, which are non-`pub` `lib/std/cmp.myc` functions, not
        // prims — a bare `ne(a,b)`/`gt(a,b)` call fails identically to the glyph (both parse to the
        // same `Expr::App`). The verified fix composes them from the `eq`/`lt` prims directly
        // (exactly `cmp.myc`'s own `ne{N}`/`gt{N}` derivation), which DOES check clean with no
        // import.
        Case {
            name: "ne_known_binary_composes_from_eq",
            rust: "fn f(a: u16, b: u16) -> bool { a != b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "(match eq(a, b) { 0b1 => False, _ => True })",
            },
        },
        Case {
            name: "gt_known_binary_composes_from_eq_and_lt",
            rust: "fn f(a: u16, b: u16) -> bool { a > b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "(match eq(a, b) { 0b1 => False, _ => match lt(a, b) { 0b1 => False, \
                            _ => True } })",
            },
        },
        // `==`/`<` are RFC-0032 D1's ratified glyphs — unchanged by this deliverable even though
        // both operands here are known `Binary{16}` (the operand-gate only fires for the
        // `& | != >` arms).
        Case {
            name: "eq_lt_known_binary_stay_glyphs",
            rust: "fn f(a: u16, b: u16) -> bool { a == b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "a == b",
            },
        },
        // Non-`Binary{N}` operand (a `bool` param, mapped to `Bool` — never a `Binary{N}` text per
        // `map_type`) keeps the CURRENT (pre-deliverable) emission unchanged: still the bare glyph,
        // not a call. Proves the gate is genuinely operand-typed, not unconditional.
        Case {
            name: "bitand_non_binary_operand_keeps_glyph",
            rust: "fn f(a: bool, b: bool) -> bool { a & b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "a & b",
            },
        },
        Case {
            name: "gt_non_binary_operand_keeps_glyph",
            rust: "fn f(a: bool, b: bool) -> bool { a > b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "a > b",
            },
        },
        // One operand unknown (a call result, not a bare in-scope identifier) — the gate requires
        // BOTH operands resolved, so this also keeps the glyph (never a half-composed emission).
        Case {
            name: "ne_one_operand_unresolved_keeps_glyph",
            rust: "fn f(a: u16, b: u16) -> bool { a != g(b) }",
            expect: Expect::Emitted {
                item: "f",
                contains: "a != g(b)",
            },
        },
        // ── P4/P5 (DN-99 §8 ENB-6 / M-1029 / ADR-028): signed-operand-gated op emission ─────────
        // (verify-first, mitigation #14 — see this module's `signed_numeric_idiom_check_clean`
        // live-oracle test for the `myc check`-clean proof over the real toolchain, and
        // `emit.rs`'s `Expr::Binary`/`Expr::Unary` arm docs for the full citation trail.) Both
        // operands are known source-signed `i32` params -> the `_s`-suffixed op family.
        Case {
            name: "signed_add_emits_add_s",
            rust: "fn f(a: i32, b: i32) -> i32 { a + b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "add_s(a, b)",
            },
        },
        Case {
            name: "signed_sub_emits_sub_s",
            rust: "fn f(a: i32, b: i32) -> i32 { a - b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "sub_s(a, b)",
            },
        },
        Case {
            name: "signed_mul_emits_mul_s",
            rust: "fn f(a: i32, b: i32) -> i32 { a * b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "mul_s(a, b)",
            },
        },
        Case {
            name: "signed_neg_emits_neg_s",
            rust: "fn f(a: i32) -> i32 { -a }",
            expect: Expect::Emitted {
                item: "f",
                contains: "neg_s(a)",
            },
        },
        Case {
            name: "signed_lt_composes_bridged_lt_s",
            rust: "fn f(a: i32, b: i32) -> bool { a < b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "(match lt_s(a, b) { 0b1 => True, _ => False })",
            },
        },
        Case {
            name: "signed_gt_composes_from_eq_and_lt_s",
            rust: "fn f(a: i32, b: i32) -> bool { a > b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "(match eq(a, b) { 0b1 => False, _ => match lt_s(a, b) { 0b1 => False, \
                            _ => True } })",
            },
        },
        // D3 arithmetic-operator-emission residual (this leaf): the UNSIGNED counterpart to the
        // `_s`-suffixed arms above. Prior to this leaf the unsigned `Add`/`Sub`/`Mul` operand-gate
        // fell through to the plain glyph (pinned by this same case's now-superseded
        // `unsigned_add_keeps_glyph_unchanged_by_this_leaf` name/comment), which did NOT
        // `myc check`-clean for a `Binary{N}` operand pair — `add` is the *ternary*-only
        // `prim_family` member (checkty.rs:9975), so `a + b` on two `Binary{N}` values failed with
        // `` `add` does not accept argument types [Binary(..), Binary(..)] `` (T-Op; RFC-0007
        // §4.4). Confirmed the exact repro `fn add2(a: u64, b: u64) -> u64 { a + b }` before this
        // fix. Now composes to the already-registered `add_u`/`sub_u`/`mul_u` prims (width-
        // preserving `Binary{N}` arithmetic, RFC-0032 D2/M-748 + RFC-0033 §4.1.2 CU-1) — proven
        // `myc check`-clean with no import (`binop_operand_gated_forms_check_clean` below).
        Case {
            name: "unsigned_add_emits_add_u",
            rust: "fn f(a: u32, b: u32) -> u32 { a + b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "add_u(a, b)",
            },
        },
        Case {
            name: "unsigned_sub_emits_sub_u",
            rust: "fn f(a: u32, b: u32) -> u32 { a - b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "sub_u(a, b)",
            },
        },
        Case {
            name: "unsigned_mul_emits_mul_u",
            rust: "fn f(a: u32, b: u32) -> u32 { a * b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "mul_u(a, b)",
            },
        },
        // Non-`Binary{N}` operand keeps the plain glyph (the gate is genuinely operand-typed, not
        // unconditional) — the twin of `bitand_non_binary_operand_keeps_glyph` for `+`.
        Case {
            name: "add_non_binary_operand_keeps_glyph",
            rust: "fn f(a: bool, b: bool) -> bool { a + b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "a + b",
            },
        },
        // A `let`-aliased local of a known `Binary{N}` param is itself recognized as known (the
        // `Stmt::Local` env-extension case (a): "RHS is a bare param already in the env").
        Case {
            name: "let_alias_of_known_binary_extends_env",
            rust: "fn f(a: u16, b: u16) -> bool { let c = a; c > b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "match eq(c, b) { 0b1 => False, _ => match lt(c, b) { 0b1 => False, \
                            _ => True } }",
            },
        },
        // An impl method's `self` parameter is threaded into the env too (via `sig_type_env`
        // already covering the `Receiver` arm's `("self", ty)` entry from `map_signature`) — a
        // `Binary{N}`-mapped `Self` type (here `u16` -> `Binary{16}`) participates in the same
        // operand gate. Uses a non-`Widen` trait name so `try_width_cast_widen_body`'s DN-41
        // special-case (which bypasses this body-emission path entirely) never intercepts it.
        Case {
            name: "impl_method_self_known_binary_participates_in_gate",
            rust: "impl Foo for u16 { fn m(self, b: u16) -> u16 { self & b } }",
            expect: Expect::Emitted {
                item: "impl Foo for Binary{16}",
                contains: "and(self, b)",
            },
        },
        // trx2 A1 (DN-34 §8.18): an `as` cast that WIDENS one unsigned `Binary` to a wider one
        // (`u16 as u32`, `Binary{16}` -> `Binary{32}`, `M >= N`) emits the faithful DN-41
        // `width_cast` — end-to-end through a fn body whose param type seeds the operand's env
        // entry. `width_cast` zero-extends (unsigned), matching Rust's unsigned widening exactly.
        // (The float-crossing / unknown-operand fidelity cases are pinned at the reason-string
        // level in `expr_cast_fidelity` below, which this table's `Expect` cannot express — it
        // asserts category, not the FLAG reason.)
        Case {
            name: "cast_widen_binary_emits_width_cast",
            rust: "fn f(x: u16) -> u32 { x as u32 }",
            expect: Expect::Emitted {
                item: "f",
                contains: "width_cast(x, 0b0000_0000_0000_0000_0000_0000_0000_0000)",
            },
        },
        // DN-51 §2 D3/§6 (maintainer-authorized DN-39 post-freeze promotion): an `as` cast that
        // NARROWS one unsigned `Binary` to a smaller one (`u32 as u16`, `Binary{32}` -> `Binary{16}`,
        // `M < N`) now emits the faithful DN-51 `truncate` — end-to-end through a fn body whose
        // param type seeds the operand's env entry. `truncate` unconditionally keeps the low `M`
        // bits, matching Rust's wrapping narrow exactly (where `width_cast`'s checked narrow would
        // refuse — see `expr_cast_fidelity`'s `narrow_u32_as_u16_emits_truncate` for the direct
        // gap-reason-level pin of the prior FLAGged state, now an emission).
        Case {
            name: "cast_narrow_binary_emits_truncate",
            rust: "fn f(x: u32) -> u16 { x as u16 }",
            expect: Expect::Emitted {
                item: "f",
                contains: "truncate(x, 0b0000_0000_0000_0000)",
            },
        },
        // ── D3 operand-type-inference depth (DN-34 §8.16 residual, trx2 follow-on) ───────────────
        // A literal operand (suffixed or not) is STILL left unresolved — never guessed. A suffixed
        // literal's *type* is decidable, but composing it into a prim call does not `myc check`-clean
        // (verify-first finding: the real toolchain refuses a bare decimal `Int` operand — "no
        // representation family" — and fixing that needs the width-correct `BinLit` spelling DN-34
        // §8.13/§8.14 already flagged as an undecided "typed-literal form" design decision; see
        // `expr_env_type`'s doc). So the gate still does not fire here, and the prior glyph emission
        // is unchanged — this pins that non-result.
        Case {
            name: "bitand_known_binary_with_suffixed_literal_keeps_glyph",
            rust: "fn f(a: u16) -> u16 { a & 5u16 }",
            expect: Expect::Emitted {
                item: "f",
                contains: "a & 5",
            },
        },
        Case {
            name: "bitand_known_binary_with_unsuffixed_literal_keeps_glyph",
            rust: "fn f(a: u16) -> u16 { a & 5 }",
            expect: Expect::Emitted {
                item: "f",
                contains: "a & 5",
            },
        },
        // `(e)`/`&e` ARE structurally transparent to the operand-type gate (this module's own
        // `Expr::Paren`/`Expr::Reference` emission arms treat them identically to `e` itself).
        Case {
            name: "bitand_known_binary_through_paren_emits_and_call",
            rust: "fn f(a: u16, b: u16) -> u16 { (a) & b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "and((a), b)",
            },
        },
        Case {
            name: "bitand_known_binary_through_reference_emits_and_call",
            rust: "fn f(a: u16, b: u16) -> u16 { &a & b }",
            expect: Expect::Emitted {
                item: "f",
                contains: "and(a, b)",
            },
        },
        // ── DN-99 #72 string-literal pattern — ENABLER LANDED (M-1035/ENB-12), now EMITS ─────────
        // A string-literal match arm `"yes" => …` is grammatically valid Mycelium surface. It was
        // gapped while the L1 checker rejected a `match` on a `Bytes` scrutinee; M-1035/ENB-12
        // landed that enabler (`check_match` admits `Ty::Bytes` with a required wildcard/default
        // arm for the open domain). So the faithful surface now EMITS and `myc check`-cleans —
        // verified against the real oracle. `&str` → `Bytes`, `true`/`false` → `True`/`False`, so
        // this lowers to `match s { "yes" => True, _ => False }` (the first enabler-driven trx win).
        // See `string_literal_pattern_emits_with_l1_enabler`.
        Case {
            name: "string_literal_pattern_emits_with_l1_enabler",
            rust: "fn classify(s: &str) -> bool { match s { \"yes\" => true, _ => false } }",
            expect: Expect::Emitted {
                item: "classify",
                contains: "match s { \"yes\" => True, _ => False }",
            },
        },
        // ── DN-118 Phase 1 — the closure-EMIT pass (`lambda_expr`) ────────────────────────────────
        // A move/`Copy`-capture closure (every free var read-only, no mutation signal): Mechanical
        // per the DN-109 D5/D7 ratchet, auto-emitted as `lambda(params) => body`, captures left as
        // ordinary in-scope references (mono's whole-program defunctionalization, RFC-0024 §4A,
        // resolves the capture set — this transpiler never synthesizes an env record). Verified
        // `myc check`-clean end-to-end (this exact shape) in DN-118 Phase 0's verify-first probe —
        // the `apply$Fn` synthetic-`Env` gap the facility hit is a *different*, unrelated mechanism
        // (`elaborate_lower_rule`'s ad-hoc single-function `Env`, `lower`-rule RHS elaboration
        // only), not a general `myc check`/whole-program limitation.
        Case {
            name: "closure_move_copy_capture_emits_lambda",
            rust: "fn make_masker(n: u16) -> u16 { let f = |x: u16| x & n; f(n) }",
            expect: Expect::Emitted {
                item: "make_masker",
                contains: "let f = lambda(x: Binary{16}) => and(x, n) in f(n)",
            },
        },
        // An untyped closure parameter has no `lambda_expr`'s `Ident ':' type_ref` correspondence
        // — this transpiler has no type-inference pass to recover an omitted type (VR-5: absence,
        // never a guess).
        Case {
            name: "closure_untyped_param_gapped",
            rust: "fn f(n: u16) -> u16 { let g = |x| x; g(n) }",
            expect: Expect::Gapped {
                category: Category::Closure,
            },
        },
        // VERIFY-FIRST FINDING (mitigation #14): a multi-parameter closure PARSES to a `lambda`
        // declaration but the L1 checker curries it (RFC-0024 §4A.8/M-822), so an ordinary
        // multi-arg call site (`f(a, b)`, this transpiler's existing `Expr::Call` emission)
        // fails `myc check` — confirmed empirically against the real oracle, NOT emitted as a
        // plausible-but-failing form (G2/VR-5); deferred as a separate, larger call-site-aware
        // unit of work.
        Case {
            name: "closure_multi_param_gapped",
            rust: "fn combine(a: u16, b: u16) -> u16 { let f = |x: u16, y: u16| and(x, y); \
                   f(a, b) }",
            expect: Expect::Gapped {
                category: Category::Closure,
            },
        },
        // A zero-parameter closure has no v0 `lambda` form (the grammar note on `lambda_expr`).
        Case {
            name: "closure_zero_param_gapped",
            rust: "fn f(n: u16) -> u16 { let g = || n; g() }",
            expect: Expect::Gapped {
                category: Category::Closure,
            },
        },
        // The DN-109 D7 safety gate: a captured binding mutated via compound assignment
        // (`total += x`, the syntactic shape of an `FnMut`-style accumulator capture) is FLAGGED,
        // never auto-emitted — `syn` carries no borrowck facts, so this cannot be proven
        // value-safe (mono would otherwise silently snapshot `total`'s value at closure
        // construction, diverging from the Rust closure's per-call-mutated semantics).
        Case {
            name: "closure_fnmut_compound_assign_capture_gapped",
            rust: "fn f(n: u16) -> u16 { let mut total = 0; let mut g = |x: u16| total += x; \
                   g(n); total }",
            expect: Expect::Gapped {
                category: Category::Closure,
            },
        },
        // The same D7 gate for an explicit `&mut` on a captured binding.
        Case {
            name: "closure_fnmut_explicit_mut_ref_capture_gapped",
            rust: "fn f(n: u16) -> u16 { let mut total = 0; let g = |x: u16| { let r = &mut \
                   total; x }; g(n) }",
            expect: Expect::Gapped {
                category: Category::Closure,
            },
        },
        // The same D7 gate for a captured binding used as a method-call RECEIVER — `syn` cannot
        // decide `&self` vs `&mut self` from syntax alone, so this is conservatively flagged too
        // (never auto-emitted on the hope the method happens to be read-only).
        Case {
            name: "closure_captured_method_receiver_gapped",
            rust: "fn f(n: u16) -> u16 { let v = n; let g = |x: u16| v.wrapping_add(x); g(n) }",
            expect: Expect::Gapped {
                category: Category::Closure,
            },
        },
        // NEGATIVE control: a closure that mutates a PURELY INTERNAL local (never escapes, never a
        // capture — bound and mutated entirely within the closure's own body) must NOT be
        // misclassified as a captured-mutation `Closure` gap. It still gaps (Mycelium's body
        // grammar has no assignment-statement production at all, `MultiStmtBody`'s pre-existing
        // semicolon-terminated-statement refusal), but via the ordinary generic path — pinning
        // that the DN-109 D7 scan does not false-positive on a shadowed/local name.
        Case {
            name: "closure_purely_local_mutation_not_misclassified_as_closure_gap",
            rust: "fn f(n: u16) -> u16 { let g = |x: u16| { let mut acc = 0; acc += x; acc }; \
                   g(n) }",
            expect: Expect::Gapped {
                category: Category::MultiStmtBody,
            },
        },
        // T-A1 (DN-122 §13.2 WU-A; positive control): a single-param, param-only-sig foreign-trait
        // impl of the registered `Ord3` prelude trait (`mvp_prelude_trait_shape`) — receiverless
        // methods, every value-param `Self`, a primitive return — synthesizes the `[<SelfTy>]`
        // Mycelium trait-argument the Rust source itself never spells (see `emit_impl`'s MVP block).
        // `binop_operand_gated_forms_check_clean`-style live-oracle coverage of the SAME shape
        // (that it actually `myc check`s clean) is below, `mvp_cmp_emit_check_agreement`.
        Case {
            name: "mvp_cmp_eligible_synthesizes_trait_arg",
            rust: "impl Ord3 for u8 { fn cmp(a: Self, b: Self) -> u8 { a } }",
            expect: Expect::Emitted {
                item: "impl Ord3[Binary{8}] for Binary{8}",
                contains: "impl Ord3[Binary{8}] for Binary{8} {\n  fn cmp(a: Binary{8}, b: Binary{8}) => Binary{8} = a;\n};",
            },
        },
        // T-A2 (negative/honest-gap control): `Widen` is two-type/`Self`-receiver-needing (DN-122
        // §13.1's own adversarial narrowing) — it is emitted EXACTLY as before WU-A (no bracket
        // synthesis, no fabricated trait/`Self` body), still an honest `myc check`-time residual
        // (M-876/M-1076), never silently "fixed" by the MVP recognizer. Mirrors the pre-existing
        // `widen_binary_emits_width_cast_not_fabricated_from` assertion, pinned here specifically
        // against MVP-bracket leakage.
        Case {
            name: "mvp_widen_unaffected_by_mvp_recognizer",
            rust: "impl Widen<u16> for u8 { fn widen(self) -> u16 { u16::from(self) } }",
            expect: Expect::Emitted {
                item: "impl Widen[Binary{16}] for Binary{8}",
                contains: "impl Widen[Binary{16}] for Binary{8} {",
            },
        },
        // T-A3 half 1 (emit<->check agreement, the transpile-time half): a `Ord3`-named impl whose
        // `cmp` method has a `self` RECEIVER (the exact shape `Widen`/`MycEq`/etc. all use) is
        // correctly recognized as INELIGIBLE (`has_self_receiver` excludes it) — emitted unchanged,
        // no `[<SelfTy>]` bracket. The live-oracle half (that the real checker ALSO refuses this
        // shape, confirming the exclusion was not overcautious) is `mvp_cmp_emit_check_agreement`.
        Case {
            name: "mvp_cmp_self_receiver_excluded_no_bracket",
            rust: "impl Ord3 for u8 { fn cmp(self, other: Self) -> u8 { self } }",
            expect: Expect::Emitted {
                item: "impl Ord3 for Binary{8}",
                contains: "impl Ord3 for Binary{8} {",
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

/// DN-99 #72 enabler-landed pin (M-1035/ENB-12, the first enabler-driven trx win): once the L1
/// checker admits a `Bytes` scrutinee in `match` (with the required wildcard/default arm for the
/// open domain), a string-literal match pattern EMITS and `myc check`-cleans. This pins the
/// faithful lowering (`&str`→`Bytes`, `"yes"` verbatim, `true`/`false`→`True`/`False`, `_` default)
/// and — its VR-5/G2 twin — that a string-literal match WITHOUT a default stays gapped never-
/// silently (a non-exhaustive `Bytes` match is check-failing surface we must not emit).
#[test]
fn string_literal_pattern_emits_with_l1_enabler() {
    // With the default arm: emits the faithful, check-clean surface.
    let rust = "fn classify(s: &str) -> bool { match s { \"yes\" => true, _ => false } }";
    let (myc, report) = transpile_source(rust, "fixture.rs", "fixture")
        .unwrap_or_else(|e| panic!("failed to parse/transpile: {e}"));
    assert!(
        report.emitted_items.iter().any(|n| n == "classify"),
        "the string-pattern fn must now emit (enabler landed), got emitted={:?} gaps={:?}",
        report.emitted_items,
        report.gaps.iter().map(|g| &g.reason).collect::<Vec<_>>()
    );
    assert!(
        myc.contains("match s { \"yes\" => True, _ => False }"),
        "the faithful check-clean string-pattern surface must be emitted, got:\n{myc}"
    );

    // Without a default arm: `Bytes` is an open domain, so an emission would be non-exhaustive and
    // check-FAIL — it must stay gapped never-silently with a reason naming the open-domain default
    // requirement (VR-5/G2), never fake-emitted.
    let no_default = "fn c(s: &str) -> bool { match s { \"yes\" => true, \"no\" => false } }";
    let (myc2, report2) = transpile_source(no_default, "fixture.rs", "fixture")
        .unwrap_or_else(|e| panic!("failed to parse/transpile: {e}"));
    assert!(
        !report2.emitted_items.iter().any(|n| n == "c"),
        "a defaultless string-literal match must stay gapped (would be non-exhaustive), got {:?}",
        report2.emitted_items
    );
    assert!(
        !myc2.contains("match s"),
        "the non-exhaustive (check-failing) surface must NEVER be emitted, got:\n{myc2}"
    );
    assert!(
        report2
            .gaps
            .iter()
            .any(|g| g.reason.contains("without a wildcard/default arm")
                && g.reason.contains("open value domain")),
        "the defaultless-match gap must cite the open-`Bytes` default requirement, got {:?}",
        report2.gaps.iter().map(|g| &g.reason).collect::<Vec<_>>()
    );
}

/// DN-132 P1 (M-1089): a named-field **struct pattern** on an in-file struct desugars to the
/// grammar's positional `Ctor` surface -- declaration-order placement (OQ-5) regardless of the
/// *pattern's* field order, a wildcard `_` at every field the pattern does not name (OQ-4,
/// regardless of whether the pattern spells `..` -- SS5.2 point 4), and the sub-pattern of a
/// renamed/nested field binding recursively mapped.
#[test]
fn struct_pattern_desugars_to_positional_ctor() {
    let cases = [
        // All fields named, in declaration order, no rest.
        (
            "struct Foo { x: u8, y: u8 } fn f(v: Foo) -> u8 { match v { Foo { x, y } => x, } }",
            "Foo(x, y)",
        ),
        // `..` rest: an unmentioned field is a wildcard.
        (
            "struct Foo { x: u8, y: u8 } fn f(v: Foo) -> u8 { match v { Foo { x, .. } => x, } }",
            "Foo(x, _)",
        ),
        // No `..` but still one field unmentioned -- SS5.2 point 4: the transpiler accepts either
        // spelling and emits the identical positional wildcard (only syntactically-valid Rust,
        // where `rustc` already enforces `..` for a genuinely partial pattern, ever reaches here).
        (
            "struct Foo { x: u8, y: u8 } fn f(v: Foo) -> u8 { match v { Foo { x } => x, } }",
            "Foo(x, _)",
        ),
        // Field-order canonicalization (OQ-5): pattern spells `y, x`, out of declaration order.
        (
            "struct Foo { x: u8, y: u8 } fn f(v: Foo) -> u8 { match v { Foo { y, x } => x, } }",
            "Foo(x, y)",
        ),
        // Field rename (`field: binding`) -- the sub-pattern binds a different local name.
        (
            "struct Foo { x: u8, y: u8 } fn f(v: Foo) -> u8 { match v { Foo { x: a, y: b } => a, } }",
            "Foo(a, b)",
        ),
        // A nested/literal sub-pattern at a named field recurses through `map_pattern`.
        (
            "struct Foo { x: u8, y: u8 } fn f(v: Foo) -> u8 { match v { Foo { x: 0, y } => y, _ => 0 } }",
            "Foo(0, y)",
        ),
        // `Self::Ctor { .. }` inside an `impl` -- the ctor-name resolution takes only the path's
        // last segment (the identical convention `Pat::Path`/`Pat::TupleStruct` already use), so
        // the `Self::` qualifier is transparent.
        (
            "struct Foo { x: u8, y: u8 } impl Foo { fn f(self) -> u8 { match self { Self { x, .. } => x, } } }",
            "Foo(x, _)",
        ),
    ];
    for (rust, needle) in cases {
        let (myc, report) = transpile_source(rust, "fixture.rs", "fixture")
            .unwrap_or_else(|e| panic!("case `{rust}` failed to parse/transpile: {e}"));
        assert!(
            myc.contains(needle),
            "case `{rust}`: expected .myc to contain `{needle}`, got:\n{myc}\ngaps={:?}",
            report.gaps.iter().map(|g| &g.reason).collect::<Vec<_>>()
        );
    }
}

/// DN-132 P1 (M-1089): the never-silent gap paths (VR-5/G2) -- a struct pattern is only ever
/// desugared when its constructor + every named field is *confirmed* resolvable; anything short of
/// that refuses rather than emitting a guessed/partial-arity `Ctor`.
#[test]
fn struct_pattern_never_silently_gaps() {
    let cases = [
        // No confirmed in-file layout at all (an undeclared/foreign constructor name).
        (
            "struct Foo { x: u8, y: u8 } fn f(v: Foo) -> u8 { match v { Bar { x, .. } => x, _ => 0 } }",
            "no confirmed in-file layout",
        ),
        // The DN-132 SS5.1 component-seam boundary this leaf documents (verify-first, mitigation
        // #14): an **enum struct-variant** pattern still gaps today, because `struct_layouts`
        // (`transpile.rs`, a sibling leaf's scope) does not yet walk `Item::Enum` variants -- this
        // arm is written generically over `struct_layout`, so it composes automatically the moment
        // that population change lands, with no further edit here.
        (
            "enum E { A { x: u8, y: u8 } } fn f(v: E) -> u8 { match v { E::A { x, .. } => x, _ => 0 } }",
            "no confirmed in-file layout",
        ),
        // A field name absent from the resolved layout -- never a silent wildcard/drop.
        (
            "struct Foo { x: u8, y: u8 } fn f(v: Foo) -> u8 { match v { Foo { z, .. } => 0, _ => 1 } }",
            "not a declared field",
        ),
        // A duplicate field name within one pattern (defensive: `syn` parses this even though
        // `rustc` itself would reject it -- DN-132 OQ-4c).
        (
            "struct Foo { x: u8, y: u8 } fn f(v: Foo) -> u8 { match v { Foo { x, x, .. } => 0, _ => 1 } }",
            "more than once",
        ),
        // A positional field-index member (`0: a`) on a struct-pattern -- out of DN-132 P1 scope.
        (
            "struct Foo(u8, u8); fn f(v: Foo) -> u8 { match v { Foo { 0: a, 1: _b } => a, } }",
            "positional field-index member",
        ),
    ];
    for (rust, needle) in cases {
        let (myc, report) = transpile_source(rust, "fixture.rs", "fixture")
            .unwrap_or_else(|e| panic!("case `{rust}` failed to parse/transpile: {e}"));
        assert!(
            !report.emitted_items.iter().any(|n| n == "f"),
            "case `{rust}`: `f` must stay gapped, got emitted={:?} myc:\n{myc}",
            report.emitted_items
        );
        assert!(
            report.gaps.iter().any(|g| g.reason.contains(needle)),
            "case `{rust}`: expected a gap whose reason contains `{needle}`, got {:?}",
            report.gaps.iter().map(|g| &g.reason).collect::<Vec<_>>()
        );
    }
}

/// **The verify-first proof** (mitigation #14) for DN-132 P1 (M-1089): every struct-pattern shape
/// [`struct_pattern_desugars_to_positional_ctor`] proves the *text* of is run through the REAL
/// `myc-check` oracle here, proving the emitted positional `Ctor` pattern actually **type-checks**
/// (the property the whole DN-132 P1 deliverable is for -- it reuses the Maranget usefulness pass
/// unchanged, so a real `myc check` pass is the honest confirmation of that claim, not just a
/// substring match). Skips gracefully (never fails) when `myc-check` is not built, exactly like
/// `binop_operand_gated_forms_check_clean` above.
#[test]
fn struct_pattern_forms_check_clean_against_real_toolchain() {
    let Some(bin) = super::vet::find_myc_check() else {
        eprintln!(
            "emit: live oracle test skipped — no runnable myc-check (set MYC_CHECK_CMD or build \
             `cargo build -p mycelium-check --bin myc-check`). The fixture-corpus text assertions \
             above still cover the emitted shape."
        );
        return;
    };

    let dir = std::env::temp_dir().join(format!(
        "mycelium-transpile-emit-struct-pattern-oracle-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");

    let rust_snippets = [
        // All fields named, declaration order.
        (
            "struct Foo { x: u8, y: u8 } fn f(v: Foo) -> u8 { match v { Foo { x, y } => x, } }",
            "f",
        ),
        // `..` rest -- an unmentioned field wildcards.
        (
            "struct Foo { x: u8, y: u8 } fn f(v: Foo) -> u8 { match v { Foo { x, .. } => x, } }",
            "f",
        ),
        // Field-order canonicalization (OQ-5): pattern spells `y, x`.
        (
            "struct Foo { x: u8, y: u8 } fn f(v: Foo) -> u8 { match v { Foo { y, x } => x, } }",
            "f",
        ),
        // Field rename (`field: binding`).
        (
            "struct Foo { x: u8, y: u8 } fn f(v: Foo) -> u8 { match v { Foo { x: a, y: b } => a, } }",
            "f",
        ),
        // Bare `Self { .. }` inside an `impl` -- the pattern-side `Self` resolution. Impl blocks
        // are recorded under `impl <Type>`, not the bare method name (see
        // `inherent_impl_no_self_name_collision_is_mangled_and_checks_clean`'s precedent).
        (
            "struct Foo { x: u8, y: u8 } impl Foo { fn f(self) -> u8 { match self { Self { x, .. } => x, } } }",
            "impl Foo",
        ),
        // A three-field struct, only one field bound, `..` for the rest.
        (
            "struct P3 { a: u8, b: u8, c: u8 } fn f(v: P3) -> u8 { match v { P3 { b, .. } => b, } }",
            "f",
        ),
    ];
    for (i, (rust, item)) in rust_snippets.iter().enumerate() {
        let (myc, report) = transpile_source(rust, "fixture.rs", "oracle")
            .unwrap_or_else(|e| panic!("failed to parse/transpile `{rust}`: {e}"));
        assert!(
            report.emitted_items.iter().any(|n| n == item),
            "case {i} (`{rust}`) failed to emit `{item}`: gaps={:?}",
            report.gaps.iter().map(|g| &g.reason).collect::<Vec<_>>()
        );
        let path = dir.join(format!("case_{i}.myc"));
        std::fs::write(&path, &myc).expect("write case .myc");

        let checker = crate::vet::MycChecker {
            command: vec![bin.display().to_string()],
            cwd: None,
        };
        let rec = checker.vet_file(&path, "fixture.rs", 1, 1);
        assert_eq!(
            rec.class,
            crate::vet::VetClass::Clean,
            "case {i} (`{rust}`) must check CLEAN with the real myc-check oracle — emitted:\n{myc}\n\
             diagnostic={:?}",
            rec.diagnostic
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

/// The #72 co-poison fix: a Rust ownership/identity-conversion no-op method (`.to_owned()`,
/// `.clone()`, `.to_string()`, `.into()`, …) has no Mycelium free-function/prim referent, so
/// desugaring it to a bare `to_owned(recv)` would FABRICATE an unknown prim (`myc check`:
/// `unknown function/constructor/prim to_owned`). It must be gapped, never fake-emitted (G2/VR-5).
/// Verified against the real oracle in the vet loop: without this gap, emitting the string-literal
/// `match` (M-1035) in `checkty::vsa_kernel_model_id` — whose arms are `"MAP-I".to_owned()` — poisons
/// the whole file's file-gated `checked_fraction`; with it, that fn gaps cleanly (no regression) and
/// gapping the fabricated conversions un-poisons real files (a measured `checked_fraction` rise).
#[test]
fn conversion_noop_method_gaps_never_fabricates_unknown_prim() {
    // A bare `.to_owned()` on a `&str` must gap, not emit a fabricated `to_owned(...)` call.
    let rust = "fn f(s: &str) -> String { s.to_owned() }";
    let (myc, report) = transpile_source(rust, "fixture.rs", "fixture")
        .unwrap_or_else(|e| panic!("failed to parse/transpile: {e}"));
    assert!(
        !report.emitted_items.iter().any(|n| n == "f"),
        "a `.to_owned()`-bodied fn must gap (no fabricated bare-call emission), got {:?}",
        report.emitted_items
    );
    assert!(
        !myc.contains("to_owned("),
        "the fabricated `to_owned(...)` bare call must NEVER be emitted, got:\n{myc}"
    );
    assert!(
        report.gaps.iter().any(|g| g
            .reason
            .contains("ownership/identity-conversion no-op method")),
        "the conversion gap must name the no-op-conversion class, got {:?}",
        report.gaps.iter().map(|g| &g.reason).collect::<Vec<_>>()
    );

    // The real `checkty::vsa_kernel_model_id` shape: a string-literal `match` (now emittable per
    // M-1035) whose arm bodies are `.to_owned()` — the whole fn must gap cleanly (the enabler flip
    // does NOT drag a fabricated `to_owned` into an emission), so no check-failing surface lands.
    let real =
        "fn m(s: &str) -> String { match s { \"A\" => \"a\".to_owned(), _ => s.to_owned() } }";
    let (myc2, report2) = transpile_source(real, "fixture.rs", "fixture")
        .unwrap_or_else(|e| panic!("failed to parse/transpile: {e}"));
    assert!(
        !report2.emitted_items.iter().any(|n| n == "m"),
        "the to_owned-bodied string-match fn must gap (no fabricated emission), got {:?}",
        report2.emitted_items
    );
    assert!(
        !myc2.contains("to_owned("),
        "no fabricated `to_owned(...)` may leak even inside a now-emittable string-match, got:\n{myc2}"
    );

    // An explicit `.deref()` call is the same fabrication class (the docstring claims `Deref`
    // coverage): it must gap, never emit a fabricated `deref(recv)` bare call (PR #1372 review fix).
    let deref = "fn g(s: &str) -> &str { s.deref() }";
    let (myc3, report3) = transpile_source(deref, "fixture.rs", "fixture")
        .unwrap_or_else(|e| panic!("failed to parse/transpile: {e}"));
    assert!(
        !report3.emitted_items.iter().any(|n| n == "g"),
        "a `.deref()`-bodied fn must gap (no fabricated bare-call emission), got {:?}",
        report3.emitted_items
    );
    assert!(
        !myc3.contains("deref("),
        "the fabricated `deref(...)` bare call must NEVER be emitted, got:\n{myc3}"
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

use super::vet::find_myc_check;

/// **The verify-first proof** (mitigation #14) for trx2 Lane C Deliverable 1: every operand-gated
/// rewrite in `Expr::Binary` (`and`/`or` for `&`/`|`, the `eq`/`lt`-composed forms for `!=`/`>`) is
/// run through the REAL `myc-check` oracle here, not just asserted as a substring match (the
/// `emit_fixture_corpus` cases above prove the *text*; this proves the text actually **type-checks**
/// with zero imports — the property the whole deliverable is for). Skips gracefully (never fails)
/// when `myc-check` is not built, exactly like `src/tests/vet.rs`'s `live_myc_check_classifies_clean_and_broken`.
#[test]
fn binop_operand_gated_forms_check_clean() {
    let Some(bin) = find_myc_check() else {
        eprintln!(
            "emit: live oracle test skipped — no runnable myc-check (set MYC_CHECK_CMD or build \
             `cargo build -p mycelium-check --bin myc-check`). The fixture-corpus text assertions \
             above still cover the emitted shape."
        );
        return;
    };

    let dir = std::env::temp_dir().join(format!(
        "mycelium-transpile-emit-binop-oracle-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");

    // Every rewrite this deliverable makes, in ONE nodule (mirrors the real driver: one file, no
    // cross-nodule imports) — `and`/`or`/`eq`/`lt` must all resolve as bare-call prims with no
    // `use`, and the composed `!=`/`>` match expressions must type as `Bool`.
    let rust_snippets = [
        "fn f_and(a: u16, b: u16) -> u16 { a & b }",
        "fn f_or(a: u16, b: u16) -> u16 { a | b }",
        "fn f_ne(a: u16, b: u16) -> bool { a != b }",
        "fn f_gt(a: u16, b: u16) -> bool { a > b }",
        // `^` (unchanged glyph) rides along as a negative control — it must ALSO check clean
        // (it already did before this deliverable; this pins that it still does).
        "fn f_xor(a: u16, b: u16) -> u16 { a ^ b }",
        // D3 operand-type-inference depth (DN-34 §8.16 residual): a `&`-reference-wrapped operand
        // must ALSO check clean, proving the extended `expr_env_type` gate composes into a real,
        // myc-check-clean body, not just matching test-fixture text.
        "fn f_and_ref(a: u16, b: u16) -> u16 { &a & b }",
        // D3 arithmetic-operator-emission residual (this leaf, the Add-glyph unblock): the
        // `add_u`/`sub_u`/`mul_u`-composed unsigned arithmetic ops must resolve as bare-call
        // prims with no `use` and type-check the fn's declared return width — the exact repro
        // this leaf closes (`fn add2(a: u64, b: u64) -> u64 { a + b }` failed `myc check` with
        // `` `add` does not accept argument types [Binary(..), Binary(..)] `` before this fix).
        "fn f_add_u(a: u16, b: u16) -> u16 { a + b }",
        "fn f_sub_u(a: u16, b: u16) -> u16 { a - b }",
        "fn f_mul_u(a: u16, b: u16) -> u16 { a * b }",
    ];
    for (i, rust) in rust_snippets.iter().enumerate() {
        let (myc, report) = transpile_source(rust, "fixture.rs", "oracle")
            .unwrap_or_else(|e| panic!("failed to parse/transpile `{rust}`: {e}"));
        assert!(
            !report.emitted_items.is_empty(),
            "case {i} (`{rust}`) failed to emit at all: gaps={:?}",
            report.gaps
        );
        let path = dir.join(format!("case_{i}.myc"));
        std::fs::write(&path, &myc).expect("write case .myc");

        let checker = crate::vet::MycChecker {
            command: vec![bin.display().to_string()],
            cwd: None,
        };
        let rec = checker.vet_file(&path, "fixture.rs", 1, 1);
        assert_eq!(
            rec.class,
            crate::vet::VetClass::Clean,
            "case {i} (`{rust}`) must check CLEAN with the real myc-check oracle — emitted:\n{myc}\n\
             diagnostic={:?}",
            rec.diagnostic
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

/// **The DN-118 Phase 1 verify-first live-oracle proof** (mitigation #14): the closure-EMIT pass's
/// `lambda` output — for a move/`Copy`-capture closure, the `closure_move_copy_capture_emits_lambda`
/// fixture above — is run through the REAL `myc-check` oracle, WHOLE-NODULE (one file, mirroring the
/// real driver), not just asserted as a substring match. This is the property the whole Phase 1
/// closure-EMIT pass exists to prove: the `apply$Fn` synthetic-`Env` gap the facility hit
/// (`elaborate_lower_rule`'s ad-hoc single-function `Env`, a `lower`-rule-only mechanism, DN-118's
/// header) is CLOSED here because mono's whole-program defunctionalization (RFC-0024 §4A, M-704)
/// resolves the generated `apply$Fn$Binary16$Binary16` dispatcher itself when the whole nodule is
/// checked — exactly as DN-118 Phase 0's standalone verify-first probe (`myc check` + `myc run`
/// against a hand-written `.myc`) already confirmed. Skips gracefully (never fails) when
/// `myc-check` is not built.
#[test]
fn closure_move_copy_capture_checks_clean() {
    let Some(bin) = find_myc_check() else {
        eprintln!(
            "emit: live oracle test skipped — no runnable myc-check (set MYC_CHECK_CMD or build \
             `cargo build -p mycelium-check --bin myc-check`). The fixture-corpus text assertion \
             (`closure_move_copy_capture_emits_lambda`) still covers the emitted shape."
        );
        return;
    };

    let dir = std::env::temp_dir().join(format!(
        "mycelium-transpile-emit-closure-oracle-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");

    // The move/Copy-capture closure case (`closure_move_copy_capture_emits_lambda`'s Rust source) —
    // the shape whose transpiled `.myc` must resolve `apply$Fn$Binary16$Binary16` whole-program.
    let rust = "fn make_masker(n: u16) -> u16 { let f = |x: u16| x & n; f(n) }";
    let (myc, report) = transpile_source(rust, "fixture.rs", "oracle")
        .unwrap_or_else(|e| panic!("failed to parse/transpile `{rust}`: {e}"));
    assert!(
        !report.emitted_items.is_empty(),
        "closure case failed to emit at all: gaps={:?}",
        report.gaps
    );
    assert!(
        report.gaps.is_empty(),
        "closure case must have zero gaps (the `apply$Fn` gap must be fully closed): {:?}",
        report.gaps
    );
    let path = dir.join("closure_case.myc");
    std::fs::write(&path, &myc).expect("write closure case .myc");

    let checker = crate::vet::MycChecker {
        command: vec![bin.display().to_string()],
        cwd: None,
    };
    let rec = checker.vet_file(&path, "fixture.rs", 1, 1);
    assert_eq!(
        rec.class,
        crate::vet::VetClass::Clean,
        "closure case must check CLEAN with the real myc-check oracle (the apply$Fn dispatcher \
         must resolve whole-program) — emitted:\n{myc}\ndiagnostic={:?}",
        rec.diagnostic
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// **The verify-first live-oracle proof** (mitigation #14) for DN-51 §2 D3/§6's transpiler flip:
/// the narrow-cast `truncate` emission (`cast_narrow_binary_emits_truncate` above) is run through
/// the REAL `myc-check` oracle, not just asserted as a substring match — the property that matters
/// is that the emitted `truncate(x, <M-bit zero witness>)` call genuinely type-checks, mirroring
/// `binop_operand_gated_forms_check_clean`'s pattern. Skips gracefully (never fails) when
/// `myc-check` is not built.
#[test]
fn cast_narrow_truncate_emission_checks_clean() {
    let Some(bin) = find_myc_check() else {
        eprintln!(
            "emit: live oracle test skipped — no runnable myc-check (set MYC_CHECK_CMD or build \
             `cargo build -p mycelium-check --bin myc-check`). The fixture-corpus text assertion \
             (`cast_narrow_binary_emits_truncate`) still covers the emitted shape."
        );
        return;
    };

    let dir = std::env::temp_dir().join(format!(
        "mycelium-transpile-emit-truncate-oracle-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");

    // The narrow-cast case (`u32 as u16`, the FLAG-truncate-not-emittable arm this task flips),
    // plus the widen/identity siblings alongside it in one nodule — pinning that `truncate` and
    // `width_cast` coexist cleanly in the same file (no cross-nodule imports, matching the real
    // driver's one-file-per-input shape).
    let rust_snippets = [
        "fn f_narrow(x: u32) -> u16 { x as u16 }",
        "fn f_widen(x: u16) -> u32 { x as u32 }",
        "fn f_identity(x: u32) -> u32 { x as u32 }",
        // A narrow all the way down to a single bit — the boundary `M = 1` case.
        "fn f_narrow_to_bit(x: u32) -> u8 { x as u8 }",
    ];
    for (i, rust) in rust_snippets.iter().enumerate() {
        let (myc, report) = transpile_source(rust, "fixture.rs", "oracle")
            .unwrap_or_else(|e| panic!("failed to parse/transpile `{rust}`: {e}"));
        assert!(
            !report.emitted_items.is_empty(),
            "case {i} (`{rust}`) failed to emit at all: gaps={:?}",
            report.gaps
        );
        let path = dir.join(format!("case_{i}.myc"));
        std::fs::write(&path, &myc).expect("write case .myc");

        let checker = crate::vet::MycChecker {
            command: vec![bin.display().to_string()],
            cwd: None,
        };
        let rec = checker.vet_file(&path, "fixture.rs", 1, 1);
        assert_eq!(
            rec.class,
            crate::vet::VetClass::Clean,
            "case {i} (`{rust}`) must check CLEAN with the real myc-check oracle — emitted:\n{myc}\n\
             diagnostic={:?}",
            rec.diagnostic
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

/// **The verify-first live-oracle proof** (mitigation #14) for P4/P5 (DN-99 §8 ENB-6 / M-1029 /
/// ADR-028): every signed-int / usize / isize / char numeric-type-idiom emission this leaf added
/// runs through the REAL `myc-check` oracle, mirroring `binop_operand_gated_forms_check_clean`'s
/// pattern. **Non-vacuity (this leaf's verify-first finding):** every one of these Rust snippets
/// was a hard GAP before this leaf (`i8..i128`/`isize`/`usize`/`char` all refused in `map_type`,
/// so the whole containing fn never emitted at all) — this test both proves the new emission is
/// `myc check`-clean AND (via `report.emitted_items`) that it is a *real* emission, not a
/// coincidental no-op. Skips gracefully (never fails) when `myc-check` is not built.
#[test]
fn signed_numeric_idiom_check_clean() {
    let Some(bin) = find_myc_check() else {
        eprintln!(
            "emit: live oracle test skipped — no runnable myc-check (set MYC_CHECK_CMD or build \
             `cargo build -p mycelium-check --bin myc-check`). The fixture-corpus text assertions \
             elsewhere still cover the emitted shape."
        );
        return;
    };

    let dir = std::env::temp_dir().join(format!(
        "mycelium-transpile-emit-p4p5-numeric-idiom-oracle-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");

    let rust_snippets = [
        // i32 arithmetic — routes to the landed `add_s`/`sub_s`/`mul_s` (ADR-028: overflow-checked
        // two's-complement, distinct from the unsigned `_u` family).
        "fn f_add(a: i32, b: i32) -> i32 { a + b }",
        "fn f_sub(a: i32, b: i32) -> i32 { a - b }",
        "fn f_mul(a: i32, b: i32) -> i32 { a * b }",
        // i64 arithmetic — a second width, pinning the width-parametric emission is not an i32-only
        // special case.
        "fn f_add64(a: i64, b: i64) -> i64 { a + b }",
        // i32 comparison — the signed order `lt_s`, bridged `Binary{1}` -> `Bool` (mirrors the
        // unsigned `Gt` composition already proven by `binop_operand_gated_forms_check_clean`).
        "fn f_lt(a: i32, b: i32) -> bool { a < b }",
        "fn f_gt(a: i32, b: i32) -> bool { a > b }",
        "fn f_ne(a: i32, b: i32) -> bool { a != b }",
        // Unary negation — the landed `neg_s`.
        "fn f_neg(a: i32) -> i32 { -a }",
        // `isize` — same `Binary{64}` mapping as `i64`, but sourced from the DISTINCT `isize` Rust
        // type (pins that `type_is_signed_int` recognizes `isize`, not just the fixed-width `iN`s).
        "fn f_neg_isize(a: isize) -> isize { -a }",
        // `usize` — a plain identity fn (the realistic index/count-parameter shape); proves the
        // UNSIGNED `Binary{64}` mapping alone (no `_s` routing — `usize` is never marked signed).
        "fn f_usize_identity(i: usize) -> usize { i }",
        // `char` — a plain identity fn; proves the `Binary{32}` codepoint mapping.
        "fn f_char_identity(c: char) -> char { c }",
    ];
    for (i, rust) in rust_snippets.iter().enumerate() {
        let (myc, report) = transpile_source(rust, "fixture.rs", "oracle")
            .unwrap_or_else(|e| panic!("failed to parse/transpile `{rust}`: {e}"));
        assert!(
            !report.emitted_items.is_empty(),
            "case {i} (`{rust}`) failed to emit at all: gaps={:?}",
            report.gaps
        );
        let path = dir.join(format!("case_{i}.myc"));
        std::fs::write(&path, &myc).expect("write case .myc");

        let checker = crate::vet::MycChecker {
            command: vec![bin.display().to_string()],
            cwd: None,
        };
        let rec = checker.vet_file(&path, "fixture.rs", 1, 1);
        assert_eq!(
            rec.class,
            crate::vet::VetClass::Clean,
            "case {i} (`{rust}`) must check CLEAN with the real myc-check oracle — emitted:\n{myc}\n\
             diagnostic={:?}",
            rec.diagnostic
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

/// Regression guard (HIGH finding, PR #1299 review, fix 1a) for the `Stmt::Local` shadow-
/// invalidation bug: a `let` that **shadows** an existing name with an RHS of *unknown* type left
/// the shadowed name's *stale* prior type in `local_env`, so `Expr::Binary`'s operand-type gate
/// could keep firing using a type that no longer applies to the (now-shadowed) name. Repro: `let x
/// = a;` (RHS is the known `Binary{16}` param `a`, so `x` is recorded as `Binary{16}`), then `let x
/// = true;` shadows `x` with a bool-literal RHS (unknown type to this module — never a `Binary{N}`
/// guess). The tail `x != b` must fall back to the plain `!=` glyph (the shadowed `x`'s type is
/// invalidated), never the `eq`/`lt`-composed form the gate would wrongly emit using the *old*
/// binding.
#[test]
fn let_shadow_with_unknown_type_invalidates_stale_binary_env_entry() {
    let rust = "fn f(a: u16, b: u16) -> bool { let x = a; let x = true; x != b }";
    let (myc, report) = transpile_source(rust, "fixture.rs", "fixture")
        .unwrap_or_else(|e| panic!("failed to parse/transpile: {e}"));
    assert!(
        report.emitted_items.iter().any(|n| n == "f"),
        "expected `f` to emit, got emitted_items={:?}",
        report.emitted_items
    );
    assert!(
        myc.contains("x != b"),
        "expected the shadowed `x != b` tail to fall back to the plain glyph (the shadow \
         invalidates x's known-Binary{{16}} type from the OLD `let x = a;` binding), got:\n{myc}"
    );
    assert!(
        !myc.contains("match eq(x, b)"),
        "the operand-type gate must NOT fire on the shadowed `x` using the stale OLD binding's \
         type — `let x = true;` shadows it with an unknown-type RHS, got:\n{myc}"
    );
}

/// Regression guard (HIGH finding, PR #1299 review, fix 1b) for the match-arm pattern-binding gap:
/// a name a match arm's pattern **binds** (here `Wrap::A(x)`'s `u32` payload `x`) must never
/// inherit an outer local's type through `env` — the outer `x: u16` (`Binary{16}`) parameter must
/// not leak onto the pattern-bound `x`, which is a *different* binding (the enum payload,
/// `Binary{32}`). Before the fix this mis-fired `and(x, b)` using the outer `Binary{16}` — a real
/// `myc check` width-mismatch failure once the pattern-bound `x` (actually `Binary{32}`) is
/// resolved against `b: Binary{16}`. The arm must fall back to the plain `&` glyph.
#[test]
fn match_arm_pattern_bound_name_invalidates_outer_binary_env_entry() {
    let rust = "enum Wrap { A(u32), B } fn f(x: u16, b: u16, w: Wrap) -> u16 { match w { \
                Wrap::A(x) => x & b, Wrap::B => b } }";
    let (myc, report) = transpile_source(rust, "fixture.rs", "fixture")
        .unwrap_or_else(|e| panic!("failed to parse/transpile: {e}"));
    assert!(
        report.emitted_items.iter().any(|n| n == "f"),
        "expected `f` to emit, got emitted_items={:?}",
        report.emitted_items
    );
    assert!(
        myc.contains("x & b"),
        "expected the `Wrap::A(x) => x & b` arm to fall back to the plain glyph (the \
         pattern-bound `x` is a distinct Binary{{32}} payload, not the outer u16 param), \
         got:\n{myc}"
    );
    assert!(
        !myc.contains("and(x, b)"),
        "the operand-type gate must NOT fire using the outer `x: u16` param's type for the \
         pattern-bound `x` (a real Binary{{32}} payload vs Binary{{16}} `b` — a genuine \
         width-mismatch myc-check failure if emitted), got:\n{myc}"
    );
}

/// **The verify-first live-oracle proof** (mitigation #14) for both PR #1299 review fixes above:
/// runs the two repros' emitted `.myc` through the REAL `myc-check` oracle. Honest finding
/// (never a silently-skipped false-green, G2): neither repro's *fixed* (fallen-back-to-glyph)
/// emission is actually `myc check`-clean — but for a completely different, PRE-EXISTING and
/// separately-tracked reason than the bug being fixed here. `!=`/`&` in the un-gated (operand-type
/// unknown) fallback path desugar to the bare word calls `ne`/`band`, which are not resolvable
/// prims with no import (exactly the failure mode this module's `Expr::Binary` doc already
/// documents for every other un-gated `!=`/`&` case, e.g. `bitand_non_binary_operand_keeps_glyph`
/// above) — this is orthogonal to, and unaffected by, the type-env shadow/pattern-binding fixes.
/// What this test proves is the *negative* the fixes exist for: the diagnostic is the KNOWN
/// `ne`/`band` gap, never a mismatched-width `and`/`eq` prim-call failure the pre-fix bug would
/// have risked (or, worse, a coincidentally-succeeding wrong-type `Clean` result). Skips
/// gracefully (never fails) when `myc-check` is not built.
#[test]
fn shadow_and_pattern_bound_fixes_fall_back_to_known_gap_not_wrong_prim_call() {
    let Some(bin) = find_myc_check() else {
        eprintln!(
            "emit: live oracle test skipped — no runnable myc-check (set MYC_CHECK_CMD or build \
             `cargo build -p mycelium-check --bin myc-check`). The fixture-corpus text \
             assertions above still cover the emitted shape."
        );
        return;
    };

    let dir = std::env::temp_dir().join(format!(
        "mycelium-transpile-emit-shadow-pattern-oracle-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");

    // (case name, rust source, the un-gated glyph word this fallback desugars to — the honest,
    // pre-existing gap the diagnostic must name; NOT the mismatched-width prim the pre-fix bug
    // would have wrongly emitted).
    let cases = [
        (
            "let_shadow",
            "fn f(a: u16, b: u16) -> bool { let x = a; let x = true; x != b }",
            "`ne`",
        ),
        (
            "match_arm_pattern_bound",
            "enum Wrap { A(u32), B } fn f(x: u16, b: u16, w: Wrap) -> u16 { match w { \
             Wrap::A(x) => x & b, Wrap::B => b } }",
            "`band`",
        ),
    ];
    for (name, rust, expected_gap_word) in cases {
        let (myc, report) = transpile_source(rust, "fixture.rs", "oracle")
            .unwrap_or_else(|e| panic!("case `{name}` (`{rust}`) failed to parse/transpile: {e}"));
        assert!(
            !report.emitted_items.is_empty(),
            "case `{name}` (`{rust}`) failed to emit at all: gaps={:?}",
            report.gaps
        );
        // Never the wrong-type prim call the pre-fix bug would have risked.
        assert!(
            !myc.contains("eq(x, b)") && !myc.contains("and(x, b)"),
            "case `{name}`: must never emit the mismatched-type prim-call form the shadow/\
             pattern-binding bug would have produced, got:\n{myc}"
        );

        let path = dir.join(format!("{name}.myc"));
        std::fs::write(&path, &myc).expect("write case .myc");
        let checker = crate::vet::MycChecker {
            command: vec![bin.display().to_string()],
            cwd: None,
        };
        let rec = checker.vet_file(&path, "fixture.rs", 1, 1);
        assert_eq!(
            rec.class,
            crate::vet::VetClass::CheckError,
            "case `{name}` (`{rust}`) was expected to hit the KNOWN pre-existing {expected_gap_word} \
             gap (never silently `Clean` on a wrong-type basis) — emitted:\n{myc}\ndiagnostic={:?}",
            rec.diagnostic
        );
        assert!(
            rec.diagnostic.contains(expected_gap_word),
            "case `{name}`: expected the diagnostic to name the known pre-existing \
             {expected_gap_word} gap, got: {}",
            rec.diagnostic
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

/// **DN-34 §8.13/8.14 "D4" — the live-oracle proof for inherent-impl no-`self` associated-fn
/// mangling.** Two different tuple-struct types each declare a same-named, receiver-less
/// constructor (`Foo::new`/`Bar::new`) in ONE nodule — exactly the shape that regressed the
/// gap-close-2 Phase-0 re-measure (`Duration::from_nanos`/`MonoInstant::from_nanos`,
/// `Task::new`/`TaskCtx::new`/`Deadlock::new`). Before the fix both emit a bare `fn new(...)`,
/// which `mycelium-l1`'s M-664 inherent-impl desugar lifts to the SAME flat top-level name —
/// `myc check` real-oracle-verified `duplicate function`. After the fix each is mangled
/// `{Type}__new`, so the combined nodule is myc-check **Clean**. Skips gracefully (never fails)
/// when `myc-check` is not built.
#[test]
fn inherent_impl_no_self_name_collision_is_mangled_and_checks_clean() {
    let Some(bin) = find_myc_check() else {
        eprintln!(
            "emit: live oracle test skipped — no runnable myc-check (set MYC_CHECK_CMD or build \
             `cargo build -p mycelium-check --bin myc-check`). The fixture-corpus text \
             assertions above still cover the emitted shape."
        );
        return;
    };

    // A parameterized constructor (never a bare literal — a bare integer literal has no
    // representation family in v0 with no `default paradigm` in scope, orthogonal to what this
    // test is proving).
    let rust = "pub struct Foo(u32);\n\
                impl Foo {\n\
                    pub fn new(x: u32) -> Self { Foo(x) }\n\
                }\n\
                pub struct Bar(u32);\n\
                impl Bar {\n\
                    pub fn new(x: u32) -> Self { Bar(x) }\n\
                }\n";
    let (myc, report) = transpile_source(rust, "fixture.rs", "oracle")
        .unwrap_or_else(|e| panic!("failed to parse/transpile the two-`new` fixture: {e}"));
    assert!(
        report.emitted_items.iter().any(|n| n == "impl Foo")
            && report.emitted_items.iter().any(|n| n == "impl Bar"),
        "both impl blocks must emit (mangling must not turn either into a gap): {:?} (gaps={:?})",
        report.emitted_items,
        report.gaps
    );
    assert!(
        myc.contains("Foo__new"),
        "expected the mangled name `Foo__new` in the emitted text, got:\n{myc}"
    );
    assert!(
        myc.contains("Bar__new"),
        "expected the mangled name `Bar__new` in the emitted text, got:\n{myc}"
    );
    assert!(
        !myc.contains("fn new("),
        "the bare, colliding name `fn new(` must never be emitted once mangling applies, got:\n{myc}"
    );

    let dir = std::env::temp_dir().join(format!(
        "mycelium-transpile-emit-inherent-mangle-oracle-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("case.myc");
    std::fs::write(&path, &myc).expect("write case .myc");
    let checker = crate::vet::MycChecker {
        command: vec![bin.display().to_string()],
        cwd: None,
    };
    let rec = checker.vet_file(&path, "fixture.rs", 1, 1);
    assert_eq!(
        rec.class,
        crate::vet::VetClass::Clean,
        "the mangled two-`new` nodule must check CLEAN with the real myc-check oracle (a \
         `duplicate function` here would mean the mangling regressed) — emitted:\n{myc}\n\
         diagnostic={:?}",
        rec.diagnostic
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// A `self`-receiving inherent-impl method is deliberately **not** mangled (see
/// `emit::mangled_inherent_fn_name`'s doc for the full scope rationale — mangling those would also
/// require rewriting every `visit_method_call` call site to the identical mangled name, a larger,
/// separately-scoped change). Pins that boundary: `as_ref`-shaped same-named `self`-methods across
/// two types stay bare (so a caller's un-qualified `.method()` desugar still resolves) — the
/// residual flat-namespace collision risk for that case is real and undocumented-away, not silently
/// "fixed" by this narrower change (VR-5: no overclaiming).
#[test]
fn self_receiving_inherent_method_is_left_unmangled() {
    let rust = "pub struct Foo(u32);\n\
                impl Foo {\n\
                    pub fn get(&self) -> u32 { self.0 }\n\
                }\n";
    let (myc, report) = transpile_source(rust, "fixture.rs", "oracle")
        .unwrap_or_else(|e| panic!("failed to parse/transpile the self-method fixture: {e}"));
    assert!(
        report.emitted_items.iter().any(|n| n == "impl Foo"),
        "impl Foo must emit: {:?} (gaps={:?})",
        report.emitted_items,
        report.gaps
    );
    assert!(
        myc.contains("fn get(") && !myc.contains("Foo__get"),
        "a `self`-receiving method must keep its bare name (never mangled), got:\n{myc}"
    );
}

// --- trx2 A1: `Expr::Cast` fidelity matrix (DN-34 §8.18) ---------------------------------------
//
// Rust `as` is lossy/wrapping/saturating/rounding by design; Mycelium's conversion prims are
// checked/refusing by design. This data-driven table pins that fidelity boundary at the
// gap-reason level (which the `cases()` table's `Expect` cannot express — it asserts a `Category`,
// not the FLAG reason). Drives `emit_expr` directly so a case can seed the operand's `TypeEnv`
// type precisely (a bare in-scope identifier is the only shape whose type the emitter resolves
// without guessing — see `expr_env_type`).

/// The expected outcome for one `Expr::Cast` fidelity case.
enum CastExpect {
    /// Emits this exact `.myc` text (faithful, `myc check`-clean).
    Emits(&'static str),
    /// Gaps with a reason containing this substring (the never-silent, honest refusal — G2/VR-5).
    GapReasonContains(&'static str),
}

/// One cast case: the operand-name -> mapped-type-text env seed, the Rust cast source, the outcome.
struct CastCase {
    name: &'static str,
    env: &'static [(&'static str, &'static str)],
    src: &'static str,
    expect: CastExpect,
}

fn cast_cases() -> Vec<CastCase> {
    use CastExpect::{Emits, GapReasonContains};
    vec![
        // WIDEN (`u16 as u32`, Binary{16} -> Binary{32}, M >= N): the one decidable-faithful slice —
        // `width_cast` zero-extends (unsigned), matching Rust's unsigned widening exactly (DN-41 §3).
        CastCase {
            name: "widen_u16_as_u32_emits_width_cast",
            env: &[("x", "Binary{16}")],
            src: "x as u32",
            expect: Emits("width_cast(x, 0b0000_0000_0000_0000_0000_0000_0000_0000)"),
        },
        // IDENTITY (`x as u32` where x is already Binary{32}, M == N): width_cast is identity here —
        // still faithful, still emitted.
        CastCase {
            name: "identity_u32_as_u32_emits_width_cast",
            env: &[("x", "Binary{32}")],
            src: "x as u32",
            expect: Emits("width_cast(x, 0b0000_0000_0000_0000_0000_0000_0000_0000)"),
        },
        // NARROW (`u32 as u16`, Binary{32} -> Binary{16}, M < N): Rust WRAPS (low 16 bits);
        // `width_cast` would REFUSE on overflow (not faithful), but `truncate` (DN-51 §2 D3, now
        // landed — maintainer-authorized DN-39 post-freeze promotion) unconditionally keeps the low
        // `M` bits — an exact match, so this now emits rather than FLAGging.
        CastCase {
            name: "narrow_u32_as_u16_emits_truncate",
            env: &[("x", "Binary{32}")],
            src: "x as u16",
            expect: Emits("truncate(x, 0b0000_0000_0000_0000)"),
        },
        // FLOAT->INT (`f64 as i32`): operand is `Float`, so this is CU-3 territory regardless of the
        // (signed) target — Rust saturates, `flt.to_bin` refuses; no faithful prim, gap CU-3.
        CastCase {
            name: "float_to_int_f64_as_i32_gaps_cu3",
            env: &[("x", "Float")],
            src: "x as i32",
            expect: GapReasonContains("PENDING-DESIGN(CU-3-fidelity)"),
        },
        // INT->FLOAT (`i64 as f64`): the target is a float, so this routes to CU-3 regardless of the
        // operand. (`i64` does not map to any `Binary{N}` — signed magnitude, `map_type` gaps — so it
        // is absent from the env; the target-float route gives the CU-3 gap, not the unknown-operand
        // one.) Rust rounds; `bin.to_flt` errs |n| > 2^53; no faithful prim, gap CU-3.
        CastCase {
            name: "int_to_float_i64_as_f64_gaps_cu3",
            env: &[],
            src: "x as f64",
            expect: GapReasonContains("PENDING-DESIGN(CU-3-fidelity)"),
        },
        // UNKNOWN OPERAND (`foo() as u32`): the operand is a call, not a bare in-scope identifier, so
        // its type is unknown — refuse rather than guess it (VR-5), and no float is involved.
        CastCase {
            name: "unknown_operand_call_gaps_never_guesses",
            env: &[],
            src: "foo() as u32",
            expect: GapReasonContains("operand type unknown"),
        },
    ]
}

#[test]
fn expr_cast_fidelity() {
    for c in cast_cases() {
        let expr: syn::Expr = syn::parse_str(c.src)
            .unwrap_or_else(|e| panic!("case `{}`: failed to parse `{}`: {e}", c.name, c.src));
        let mut env = TypeEnv::new();
        for (k, v) in c.env {
            env.insert((*k).to_string(), (*v).to_string());
        }
        match (c.expect, emit_expr(&expr, None, &env)) {
            (CastExpect::Emits(want), Ok(text)) => {
                assert_eq!(text, want, "case `{}`: emitted text mismatch", c.name)
            }
            (CastExpect::Emits(want), Err(g)) => panic!(
                "case `{}`: expected emit `{want}`, got gap: {}",
                c.name, g.reason
            ),
            (CastExpect::GapReasonContains(sub), Err(g)) => assert!(
                g.reason.contains(sub),
                "case `{}`: gap reason did not contain `{sub}`; got: {}",
                c.name,
                g.reason
            ),
            (CastExpect::GapReasonContains(sub), Ok(text)) => panic!(
                "case `{}`: expected a gap containing `{sub}`, got emit `{text}`",
                c.name
            ),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// DN-122 §13 (M-1080; WU-A) — the MVP foreign-trait-impl live-oracle proof (T-A1/T-A2/T-A3).
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// **T-A1 (positive control) + T-A3 (emit<->check agreement), against the REAL toolchain.** The
/// fixture-corpus cases above (`mvp_cmp_eligible_synthesizes_trait_arg`,
/// `mvp_widen_unaffected_by_mvp_recognizer`, `mvp_cmp_self_receiver_excluded_no_bracket`) prove the
/// emitted *text*; this proves the emitter's eligibility judgment agrees with what `myc check`
/// actually accepts — never a `[<SelfTy>]` bracket for a shape the checker would refuse, and never
/// a missed bracket for a shape that would otherwise check clean. Skips gracefully (never fails)
/// when `myc-check` is not built, exactly like `src/tests/vet.rs`'s live-oracle tests.
#[test]
fn mvp_cmp_emit_check_agreement() {
    let Some(bin) = find_myc_check() else {
        eprintln!(
            "emit: DN-122/M-1080 MVP live oracle test skipped — no runnable myc-check (set \
             MYC_CHECK_CMD or build `cargo build -p mycelium-check --bin myc-check`). The \
             fixture-corpus text assertions above still cover the emitted shape."
        );
        return;
    };

    struct AgreementCase {
        name: &'static str,
        rust: &'static str,
        /// Whether the emitted `.myc` carries the MVP-synthesized `[<SelfTy>]` bracket for `Ord3`.
        expect_bracket: bool,
        /// Whether the real `myc-check` oracle accepts the emitted file clean.
        expect_clean: bool,
    }
    let cases = [
        // T-A1: single-param, param-only-sig, receiverless — MVP-eligible, checks clean.
        AgreementCase {
            name: "eligible_cmp",
            rust: "impl Ord3 for u8 { fn cmp(a: Self, b: Self) -> u8 { a } }",
            expect_bracket: true,
            expect_clean: true,
        },
        // T-A2: `Widen` (two-type/`Self`-receiver-needing) — unaffected by the MVP recognizer,
        // stays an honest `myc check`-time residual (M-876/M-1076), exactly as before WU-A.
        AgreementCase {
            name: "widen_stays_a_residual",
            rust: "impl Widen<u16> for u8 { fn widen(self) -> u16 { u16::from(self) } }",
            expect_bracket: false,
            expect_clean: false,
        },
        // T-A3: `self`-receiver `Ord3` impl — correctly excluded (no bracket); the checker refuses
        // it too (`cmp_used` still seeds the prelude trait since the impl NAMES `Ord3`, so the
        // checker's own arity/shape enforcement — not an "unknown trait" gap — is what fires here;
        // either way, never a silent accept).
        AgreementCase {
            name: "self_receiver_excluded_and_checker_agrees",
            rust: "impl Ord3 for u8 { fn cmp(self, other: Self) -> u8 { self } }",
            expect_bracket: false,
            expect_clean: false,
        },
        // T-A3: wrong arity (`Ord3::cmp` takes exactly 2 params in the prelude) — excluded, and the
        // checker's `register_instances` arity guard refuses it too.
        AgreementCase {
            name: "wrong_arity_excluded_and_checker_agrees",
            rust: "impl Ord3 for u8 { fn cmp(a: Self) -> u8 { a } }",
            expect_bracket: false,
            expect_clean: false,
        },
    ];

    let dir = std::env::temp_dir().join(format!(
        "mycelium-transpile-emit-mvp-cmp-oracle-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");

    for (i, case) in cases.iter().enumerate() {
        let (myc, report) = transpile_source(case.rust, "fixture.rs", "oracle")
            .unwrap_or_else(|e| panic!("case `{}` failed to parse/transpile: {e}", case.name));
        assert!(
            !report.emitted_items.is_empty(),
            "case `{}` failed to emit at all: gaps={:?}",
            case.name,
            report.gaps
        );
        let has_bracket = myc.contains("Ord3[Binary{8}] for Binary{8}");
        assert_eq!(
            has_bracket, case.expect_bracket,
            "case `{}`: MVP-bracket presence mismatch; emitted:\n{myc}",
            case.name
        );

        let path = dir.join(format!("case_{i}.myc"));
        std::fs::write(&path, &myc).expect("write case .myc");
        let checker = crate::vet::MycChecker {
            command: vec![bin.display().to_string()],
            cwd: None,
        };
        let rec = checker.vet_file(&path, "fixture.rs", 1, 1);
        let is_clean = rec.class == crate::vet::VetClass::Clean;
        assert_eq!(
            is_clean, case.expect_clean,
            "case `{}`: emit<->check agreement violated — emitter's bracket judgment ({}) must \
             agree with the real checker's verdict; diagnostic={:?}\nemitted:\n{myc}",
            case.name, case.expect_bracket, rec.diagnostic
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}
