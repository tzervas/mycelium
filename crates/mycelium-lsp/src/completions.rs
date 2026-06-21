//! **Lexical/scaffolding completion provider** (dogfooding wave — LSP completions track).
//!
//! Scope and honesty (Declared):
//! - This is a **lexical + scaffolding** provider only. It offers the ACTIVE keywords from the L1
//!   surface and a small set of high-value code scaffolding snippets grounded in the grammar
//!   (`docs/spec/grammar/mycelium.ebnf`). It has **no** semantic context: it does not know the
//!   type at the cursor, what names are in scope, or what imports are open. Clients should present
//!   these as simple keyword/scaffold suggestions — never as type-aware or inference-driven
//!   completions. Guarantee: `Declared` (asserted capabilities, always flagged).
//!
//! Active keywords are drawn from the `keyword()` function in `mycelium-l1::token` — the
//! authoritative source for which words lex as keywords today. Reserved-not-active words
//! (`phylum`, `colony`) and ratified-not-yet-lexed runtime words (`hypha`, `fuse`, …) are
//! intentionally excluded from keyword completions: offering them as if usable would violate the
//! honesty rule (VR-5 / G2).
//!
//! `matured` is offered as a keyword (it is reserved — using it at item position is an explicit
//! parse error with a teaching diagnostic, RFC-0017 §4.1) because the correct usage in a header
//! comment (`// @matured: true`) benefits from a header-scaffold snippet, not from a keyword
//! completion in code position. The keyword entry is provided with a detail note pointing at the
//! header attribute form. `thaw` is offered as a keyword because `thaw fn f(…)` is active syntax.
//!
//! LSP completion item kinds (integer codes from the LSP specification):
//! - `1` = Text
//! - `14` = Keyword
//! - `15` = Snippet

use serde_json::{json, Value};

/// LSP `CompletionItemKind` for a reserved keyword.
const KIND_KEYWORD: u8 = 14;
/// LSP `CompletionItemKind` for a code scaffold snippet.
const KIND_SNIPPET: u8 = 15;

/// LSP `insertTextFormat`: plain text (`1`) vs snippet grammar (`2`).
const FORMAT_PLAIN: u8 = 1;
const FORMAT_SNIPPET: u8 = 2;

/// A single LSP completion item (minimal fields: `label`, `kind`, `insertText`,
/// `insertTextFormat`, `detail`, `documentation`).
///
/// All fields serialise to the LSP `CompletionItem` shape required by the protocol.
#[derive(Debug, Clone, PartialEq)]
pub struct CompletionItem {
    /// The label shown in the completion list.
    pub label: &'static str,
    /// LSP `CompletionItemKind` integer code.
    pub kind: u8,
    /// The text inserted (plain or snippet grammar depending on `insert_text_format`).
    pub insert_text: &'static str,
    /// `1` = plain, `2` = snippet (tab stops `$1`, `${1:placeholder}`, `$0`).
    pub insert_text_format: u8,
    /// Short one-line detail shown inline in most editors.
    pub detail: &'static str,
    /// Longer documentation string (plain text; shown in a hover panel).
    pub documentation: &'static str,
}

impl CompletionItem {
    /// Serialize this item to an LSP `CompletionItem` JSON value.
    #[must_use]
    pub fn to_lsp_value(&self) -> Value {
        json!({
            "label": self.label,
            "kind": self.kind,
            "insertText": self.insert_text,
            "insertTextFormat": self.insert_text_format,
            "detail": self.detail,
            "documentation": self.documentation,
        })
    }
}

// ---------------------------------------------------------------------------
// Active keyword completions
// ---------------------------------------------------------------------------
//
// Source-of-truth: `crates/mycelium-l1/src/token.rs` `keyword()` function.
// Only ACTIVE keywords are listed here. Reserved-not-active (`phylum`, `colony`) are
// intentionally absent — they cannot open any construct yet (G2 / VR-5).

/// The complete set of active keyword completions.
///
/// Order: structural keywords first (most common at top level), then expression-level, then
/// type/qualifier/strength keywords. Alphabetical within each group for stability.
pub const KEYWORD_COMPLETIONS: &[CompletionItem] = &[
    // --- structural keywords ---
    CompletionItem {
        label: "default",
        kind: KIND_KEYWORD,
        insert_text: "default",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — ambient paradigm declaration",
        documentation: "Opens a nodule-scope ambient declaration: `default paradigm P`. \
                        RFC-0012 §4.2.",
    },
    CompletionItem {
        label: "fn",
        kind: KIND_KEYWORD,
        insert_text: "fn",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — function definition",
        documentation: "Declares a function: `fn name(params) -> ReturnType = body`. \
                        L1 grammar; RFC-0007.",
    },
    CompletionItem {
        label: "matured",
        kind: KIND_KEYWORD,
        insert_text: "matured",
        insert_text_format: FORMAT_PLAIN,
        detail: "reserved keyword — scope-level AOT promotion (item position -> parse error)",
        documentation: "RESERVED: using `matured` at item position is a parse error with a \
                        teaching diagnostic (RFC-0017 §4.1). Declare maturation in the file \
                        header as `// @matured: true` or in `mycelium-proj.toml`. \
                        Use `thaw fn` to keep one definition interpreted inside a matured scope.",
    },
    CompletionItem {
        label: "nodule",
        kind: KIND_KEYWORD,
        insert_text: "nodule",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — nodule declaration (basic static unit)",
        documentation: "Declares a nodule (the basic static organizational unit, approx. module). \
                        Every source file starts with `// nodule: path.name` then `nodule path.name`. \
                        DN-06; RFC-0006.",
    },
    CompletionItem {
        label: "paradigm",
        kind: KIND_KEYWORD,
        insert_text: "paradigm",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — ambient granularity selector",
        documentation: "Used with `default` or `with`: `default paradigm P` / `with paradigm P { ... }`. \
                        RFC-0012 §4.2/§4.4.",
    },
    CompletionItem {
        label: "spore",
        kind: KIND_KEYWORD,
        insert_text: "spore",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — reconstruction-manifest construction",
        documentation: "Constructs a reconstruction manifest / deployable artifact. \
                        DN-02 §2/§7; RFC-0003 §6; ADR-013.",
    },
    CompletionItem {
        label: "thaw",
        kind: KIND_KEYWORD,
        insert_text: "thaw",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — de-maturation: keep one fn interpreted",
        documentation: "Keeps one function definition interpreted inside an otherwise-matured scope: \
                        `thaw fn f(...) -> T = ...`. RFC-0017 §4.3.",
    },
    CompletionItem {
        label: "trait",
        kind: KIND_KEYWORD,
        insert_text: "trait",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — typeclass / behavior set",
        documentation: "Declares a trait (typeclass / behavior set; `guild` was declined). \
                        DN-02 §7.",
    },
    CompletionItem {
        label: "type",
        kind: KIND_KEYWORD,
        insert_text: "type",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — data-type (sum) declaration",
        documentation: "Declares a sum type (ADT): `type Name = Ctor(T) | Ctor2(T1, T2)`. \
                        DN-02 §7.",
    },
    CompletionItem {
        label: "use",
        kind: KIND_KEYWORD,
        insert_text: "use",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — import",
        documentation: "Imports a name from another nodule: `use path.to.name`. \
                        DN-02 §3.",
    },
    CompletionItem {
        label: "wild",
        kind: KIND_KEYWORD,
        insert_text: "wild",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — denied-by-default unsafe block",
        documentation: "The only FFI/raw-memory site: `wild { expr }`. \
                        Denied by default; requires an explicit capability grant. DN-02 §5/§7.",
    },
    // --- expression keywords ---
    CompletionItem {
        label: "else",
        kind: KIND_KEYWORD,
        insert_text: "else",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — conditional else branch",
        documentation: "The else branch of `if cond then e1 else e2`. DN-02 §3.",
    },
    CompletionItem {
        label: "for",
        kind: KIND_KEYWORD,
        insert_text: "for",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — bounded iteration (structural recursion sugar)",
        documentation: "Bounded iteration over a structural recursion: \
                        `for x in coll, acc = init => body`. Total by construction. \
                        RFC-0007 §4.8 r2; DN-03 §2.",
    },
    CompletionItem {
        label: "if",
        kind: KIND_KEYWORD,
        insert_text: "if",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — conditional expression",
        documentation: "Conditional: `if cond then e1 else e2`. DN-02 §3.",
    },
    CompletionItem {
        label: "in",
        kind: KIND_KEYWORD,
        insert_text: "in",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — binding body separator",
        documentation: "Separates the binding from its body in `let x = e in body`. grammar.",
    },
    CompletionItem {
        label: "let",
        kind: KIND_KEYWORD,
        insert_text: "let",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — local binding",
        documentation: "Local binding: `let x = expr in body`. DN-02 §3.",
    },
    CompletionItem {
        label: "match",
        kind: KIND_KEYWORD,
        insert_text: "match",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — pattern match (exhaustiveness checked)",
        documentation: "Pattern match: `match expr { Ctor(x) => body, ... }`. \
                        Coverage is checked by the Maranget algorithm -- exhaustiveness and \
                        redundancy, never assumed. DN-02 §3.",
    },
    CompletionItem {
        label: "then",
        kind: KIND_KEYWORD,
        insert_text: "then",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — conditional then branch",
        documentation: "The then branch of `if cond then e1 else e2`. DN-02 §3.",
    },
    CompletionItem {
        label: "with",
        kind: KIND_KEYWORD,
        insert_text: "with",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — block-scope ambient override",
        documentation: "Opens a block-scope ambient override: `with paradigm P { ... }`. \
                        RFC-0012 §4.4.",
    },
    // --- swap keywords ---
    CompletionItem {
        label: "policy",
        kind: KIND_KEYWORD,
        insert_text: "policy",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — swap policy label (inside swap(...))",
        documentation: "Labels the policy argument of a swap expression: \
                        `swap(x, to: TargetType, policy: policy_name)`. \
                        Both `to:` and `policy:` are mandatory (S1/WF2). grammar.",
    },
    CompletionItem {
        label: "swap",
        kind: KIND_KEYWORD,
        insert_text: "swap",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — never-silent representation change",
        documentation: "The never-silent representation change: \
                        `swap(expr, to: TargetType, policy: policy_name)`. \
                        Both `to:` and `policy:` are always mandatory -- a swap is never implicit \
                        (S1/WF2). RFC-0001 §4.5; RFC-0002.",
    },
    CompletionItem {
        label: "to",
        kind: KIND_KEYWORD,
        insert_text: "to",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — swap target label (inside swap(...))",
        documentation: "Labels the target-type argument of a swap expression: \
                        `swap(x, to: TargetType, policy: policy_name)`. grammar.",
    },
    // --- type keywords ---
    CompletionItem {
        label: "Binary",
        kind: KIND_KEYWORD,
        insert_text: "Binary",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — N-bit binary representation type",
        documentation: "N-bit binary representation: `Binary{N}` (e.g. `Binary{8}`). \
                        RFC-0001; grammar.",
    },
    CompletionItem {
        label: "Dense",
        kind: KIND_KEYWORD,
        insert_text: "Dense",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — dense embedding type",
        documentation: "Dense floating-point embedding: `Dense{N, ScalarKind}` \
                        (e.g. `Dense{1024, F32}`). RFC-0001; grammar.",
    },
    CompletionItem {
        label: "Sparse",
        kind: KIND_KEYWORD,
        insert_text: "Sparse",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — sparsity qualifier (for VSA)",
        documentation: "Sparsity qualifier for a VSA type: `Sparse{N}`. grammar.",
    },
    CompletionItem {
        label: "Substrate",
        kind: KIND_KEYWORD,
        insert_text: "Substrate",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — affine external-resource kind (consumed exactly once)",
        documentation: "Affine external resource: `Substrate{Name}`. Consumed exactly once -- \
                        linear type semantics. DN-02 §2; LR-8.",
    },
    CompletionItem {
        label: "Ternary",
        kind: KIND_KEYWORD,
        insert_text: "Ternary",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — N-trit balanced-ternary type",
        documentation: "N-trit balanced-ternary representation: `Ternary{N}` (e.g. `Ternary{6}`). \
                        RFC-0001; grammar.",
    },
    CompletionItem {
        label: "VSA",
        kind: KIND_KEYWORD,
        insert_text: "VSA",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — hypervector type",
        documentation: "Hypervector type: `VSA{model, dim, sparsity}`. RFC-0001; grammar.",
    },
    // --- scalar-kind keywords ---
    CompletionItem {
        label: "BF16",
        kind: KIND_KEYWORD,
        insert_text: "BF16",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — BFloat16 scalar kind (for Dense)",
        documentation: "BFloat16 scalar kind used in `Dense{N, BF16}`. grammar.",
    },
    CompletionItem {
        label: "F16",
        kind: KIND_KEYWORD,
        insert_text: "F16",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — Float16 scalar kind (for Dense)",
        documentation: "Float16 scalar kind used in `Dense{N, F16}`. grammar.",
    },
    CompletionItem {
        label: "F32",
        kind: KIND_KEYWORD,
        insert_text: "F32",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — Float32 scalar kind (for Dense)",
        documentation: "Float32 scalar kind used in `Dense{N, F32}`. grammar.",
    },
    CompletionItem {
        label: "F64",
        kind: KIND_KEYWORD,
        insert_text: "F64",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — Float64 scalar kind (for Dense)",
        documentation: "Float64 scalar kind used in `Dense{N, F64}`. grammar.",
    },
    // --- guarantee-strength keywords ---
    CompletionItem {
        label: "Declared",
        kind: KIND_KEYWORD,
        insert_text: "Declared",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — guarantee strength: asserted, always flagged",
        documentation: "Guarantee-strength tag: `T @ Declared` -- asserted without a theorem or \
                        empirical trials. Always flagged in the toolchain. Lattice: \
                        Exact > Proven > Empirical > Declared. RFC-0001; DN-02 §7.",
    },
    CompletionItem {
        label: "Empirical",
        kind: KIND_KEYWORD,
        insert_text: "Empirical",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — guarantee strength: supported by trials",
        documentation: "Guarantee-strength tag: `T @ Empirical` -- backed by measured trials. \
                        Lattice: Exact > Proven > Empirical > Declared. RFC-0001; DN-02 §7.",
    },
    CompletionItem {
        label: "Exact",
        kind: KIND_KEYWORD,
        insert_text: "Exact",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — guarantee strength: lossless / bit-exact",
        documentation: "Guarantee-strength tag: `T @ Exact` -- lossless / bit-exact. \
                        Strongest on the lattice: Exact > Proven > Empirical > Declared. \
                        RFC-0001; DN-02 §7.",
    },
    CompletionItem {
        label: "Proven",
        kind: KIND_KEYWORD,
        insert_text: "Proven",
        insert_text_format: FORMAT_PLAIN,
        detail: "keyword — guarantee strength: theorem-backed (checked side-conditions)",
        documentation: "Guarantee-strength tag: `T @ Proven` -- backed by a theorem whose \
                        side-conditions are checked. Only allowed with a checked theorem; \
                        downgrade to `Empirical`/`Declared` otherwise (VR-5). \
                        Lattice: Exact > Proven > Empirical > Declared. RFC-0001; DN-02 §7.",
    },
];

// ---------------------------------------------------------------------------
// Scaffolding snippets
// ---------------------------------------------------------------------------
//
// Snippet grammar: `$1`, `$2`, ... are tab stops; `${1:placeholder}` has a default text;
// `$0` is the final cursor position. Grounded in `docs/spec/grammar/mycelium.ebnf`.

/// The set of high-value scaffolding snippets.
pub const SNIPPET_COMPLETIONS: &[CompletionItem] = &[
    CompletionItem {
        label: "nodule-header",
        kind: KIND_SNIPPET,
        insert_text: "// nodule: ${1:path.name}\nnodule ${1:path.name}\n$0",
        insert_text_format: FORMAT_SNIPPET,
        detail: "snippet — nodule header (comment marker + declaration)",
        documentation: "Scaffolds the required header for a Mycelium source file: \
                        the `// nodule: path` comment (recognised by parse_nodule_header) \
                        followed by the `nodule` declaration. DN-06; RFC-0006.",
    },
    CompletionItem {
        label: "fn-def",
        kind: KIND_SNIPPET,
        insert_text: "fn ${1:name}(${2:x}: ${3:Binary{8}}) -> ${4:Binary{8}} =\n  ${0:expr}",
        insert_text_format: FORMAT_SNIPPET,
        detail: "snippet — function definition",
        documentation:
            "Scaffolds a function definition: `fn name(param: Type) -> ReturnType = body`. \
                        Grammar: `fn Ident type_params? '(' params? ')' '->' type_ref '=' expr`. \
                        RFC-0007.",
    },
    CompletionItem {
        label: "type-adt",
        kind: KIND_SNIPPET,
        insert_text: "type ${1:Name} = ${2:Ctor}(${3:Binary{8}}) | ${4:Ctor2}(${5:Binary{8}})\n$0",
        insert_text_format: FORMAT_SNIPPET,
        detail: "snippet — sum type (ADT) declaration",
        documentation: "Scaffolds a sum-type (ADT) declaration: \
                        `type Name = Ctor(T) | Ctor2(T1, T2)`. \
                        DN-02 §7; grammar `type_item`.",
    },
    CompletionItem {
        label: "match-expr",
        kind: KIND_SNIPPET,
        insert_text: "match ${1:expr} {\n  ${2:Ctor}(${3:x}) => ${4:x},\n  ${5:_} => ${0:todo},\n}",
        insert_text_format: FORMAT_SNIPPET,
        detail: "snippet — match expression (exhaustiveness checked)",
        documentation: "Scaffolds a match expression. Coverage is checked by the Maranget \
                        usefulness algorithm -- exhaustiveness and redundancy; never assumed. \
                        DN-02 §3; grammar `match_expr`.",
    },
    CompletionItem {
        label: "swap-expr",
        kind: KIND_SNIPPET,
        insert_text: "swap(${1:expr}, to: ${2:TargetType}, policy: ${3:policy_name})$0",
        insert_text_format: FORMAT_SNIPPET,
        detail: "snippet — never-silent representation change",
        documentation: "Scaffolds a swap expression. Both `to:` and `policy:` are always \
                        mandatory -- omitting either is a parse error (S1/WF2). The swap is \
                        never implicit; every representation change is lexically visible. \
                        RFC-0001 §4.5; RFC-0002.",
    },
];

// ---------------------------------------------------------------------------
// Public provider function
// ---------------------------------------------------------------------------

/// Return the full list of completion items (keywords + snippets) as an LSP
/// `CompletionList` JSON value (`{ isIncomplete: false, items: [...] }`).
///
/// This is a **lexical/scaffolding** provider -- `Declared` scope. It does not perform
/// semantic analysis, type lookup, or scope resolution. Clients should surface these as
/// generic keyword/scaffold suggestions rather than context-aware type-driven completions.
#[must_use]
pub fn completion_list() -> Value {
    let items: Vec<Value> = KEYWORD_COMPLETIONS
        .iter()
        .chain(SNIPPET_COMPLETIONS.iter())
        .map(CompletionItem::to_lsp_value)
        .collect();
    json!({
        "isIncomplete": false,
        "items": items,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ----- keyword presence -----

    #[test]
    fn all_active_structural_keywords_are_offered() {
        let labels: Vec<&str> = KEYWORD_COMPLETIONS.iter().map(|c| c.label).collect();
        // These are all the active structural keywords (token.rs `keyword()` -- active set).
        for kw in [
            "nodule", "use", "type", "trait", "fn", "thaw", "let", "in", "if", "then", "else",
            "match", "for", "swap", "default", "paradigm", "with", "wild", "spore", "to", "policy",
            "matured",
        ] {
            assert!(
                labels.contains(&kw),
                "active keyword `{kw}` missing from KEYWORD_COMPLETIONS"
            );
        }
    }

    #[test]
    fn all_active_type_keywords_are_offered() {
        let labels: Vec<&str> = KEYWORD_COMPLETIONS.iter().map(|c| c.label).collect();
        for kw in [
            "Binary",
            "Ternary",
            "Dense",
            "VSA",
            "Substrate",
            "Sparse",
            "F16",
            "BF16",
            "F32",
            "F64",
            "Exact",
            "Proven",
            "Empirical",
            "Declared",
        ] {
            assert!(
                labels.contains(&kw),
                "active type/scalar/strength keyword `{kw}` missing from KEYWORD_COMPLETIONS"
            );
        }
    }

    #[test]
    fn reserved_not_active_words_are_not_offered() {
        // `phylum` and `colony` are reserved-not-active: they lex as keywords but no construct
        // consumes them yet -- offering them as usable would violate the honesty rule (G2 / VR-5).
        let labels: Vec<&str> = KEYWORD_COMPLETIONS
            .iter()
            .chain(SNIPPET_COMPLETIONS.iter())
            .map(|c| c.label)
            .collect();
        for banned in ["phylum", "colony"] {
            assert!(
                !labels.contains(&banned),
                "reserved-not-active word `{banned}` must NOT appear in completions"
            );
        }
    }

    #[test]
    fn runtime_not_yet_lexed_words_are_not_offered() {
        // These are ratified but not yet in keyword() -- offering them as active syntax would
        // be dishonest (they currently lex as plain identifiers).
        let labels: Vec<&str> = KEYWORD_COMPLETIONS
            .iter()
            .chain(SNIPPET_COMPLETIONS.iter())
            .map(|c| c.label)
            .collect();
        for unlexed in [
            "hypha", "fuse", "mesh", "graft", "cyst", "xloc", "forage", "backbone", "tier",
            "reclaim", "impl", "consume", "grow",
        ] {
            assert!(
                !labels.contains(&unlexed),
                "ratified-not-yet-lexed word `{unlexed}` must NOT appear in completions"
            );
        }
    }

    // ----- snippet well-formedness -----

    #[test]
    fn all_snippets_have_snippet_format_and_contain_tab_stops() {
        for snippet in SNIPPET_COMPLETIONS {
            assert_eq!(
                snippet.insert_text_format, FORMAT_SNIPPET,
                "snippet `{}` must use FORMAT_SNIPPET (2)",
                snippet.label
            );
            assert!(
                snippet.insert_text.contains('$'),
                "snippet `{}` has no tab stops (`$`)",
                snippet.label
            );
        }
    }

    #[test]
    fn nodule_header_snippet_contains_nodule_and_comment_marker() {
        let nodule = SNIPPET_COMPLETIONS
            .iter()
            .find(|s| s.label == "nodule-header")
            .expect("nodule-header snippet must exist");
        assert!(nodule.insert_text.contains("// nodule:"));
        assert!(nodule.insert_text.contains("nodule "));
    }

    #[test]
    fn swap_snippet_contains_both_to_and_policy() {
        // S1/WF2: both `to:` and `policy:` must always be present in a swap.
        let swap = SNIPPET_COMPLETIONS
            .iter()
            .find(|s| s.label == "swap-expr")
            .expect("swap-expr snippet must exist");
        assert!(
            swap.insert_text.contains("to:"),
            "swap snippet must contain `to:` (S1)"
        );
        assert!(
            swap.insert_text.contains("policy:"),
            "swap snippet must contain `policy:` (S1/WF2)"
        );
    }

    #[test]
    fn fn_def_snippet_has_arrow_and_equals() {
        let fn_def = SNIPPET_COMPLETIONS
            .iter()
            .find(|s| s.label == "fn-def")
            .expect("fn-def snippet must exist");
        assert!(fn_def.insert_text.contains("->"), "fn-def must have `->`");
        assert!(fn_def.insert_text.contains('='), "fn-def must have `=`");
    }

    // ----- completion_list() shape -----

    #[test]
    fn completion_list_has_lsp_shape() {
        let list = completion_list();
        assert_eq!(
            list["isIncomplete"], false,
            "isIncomplete must be false for a static list"
        );
        let items = list["items"].as_array().expect("items must be an array");
        assert!(
            !items.is_empty(),
            "completion list must have at least one item"
        );
        // Every item must have the required LSP CompletionItem fields.
        for item in items {
            assert!(item["label"].is_string(), "each item must have a `label`");
            assert!(item["kind"].is_number(), "each item must have a `kind`");
            assert!(
                item["insertText"].is_string(),
                "each item must have `insertText`"
            );
        }
    }

    #[test]
    fn completion_list_total_count_matches_constants() {
        let list = completion_list();
        let items = list["items"].as_array().unwrap();
        assert_eq!(
            items.len(),
            KEYWORD_COMPLETIONS.len() + SNIPPET_COMPLETIONS.len(),
            "completion_list() must include every keyword and every snippet"
        );
    }

    #[test]
    fn keyword_completions_use_plain_format_and_kind_14() {
        for kw in KEYWORD_COMPLETIONS {
            assert_eq!(
                kw.kind, KIND_KEYWORD,
                "keyword `{}` must have kind=14 (Keyword)",
                kw.label
            );
            assert_eq!(
                kw.insert_text_format, FORMAT_PLAIN,
                "keyword `{}` must use plain insert-text format",
                kw.label
            );
        }
    }
}
