#!/usr/bin/env python3
"""Editor-grammar generator (M-731; RFC-0026) — v2 (full-surface grammars).

Single source of truth for the KEYWORD SET: the canonical L1 lexer `keyword()` table in
`crates/mycelium-l1/src/token.rs`. This generator *extracts* the keyword set from that function
and emits the editor-grammar artifacts so the lexer can never silently diverge from the grammars
that colour editors (G2). A `--check` mode (the `just drift-check` gate) regenerates in memory and
fails on any divergence from the committed files.

HONESTY / status (VR-5, G2):
  * The keyword *set* and its *class buckets* are derived mechanically from `token.rs` (the RHS
    `=> Tok::…` of each arm), never hand-maintained — so adding a keyword to the lexer without
    regenerating fails the drift gate. The STRUCTURAL productions below (v2) additionally consume
    specific keywords by name; the `--self-test` asserts every extracted keyword appears in every
    rendered artifact, so a lexer keyword rename/addition that the templates miss fails LOUDLY
    (exit 3), never silently drops a word from highlighting.
  * The *scope names* are the **ratified RFC-0026 §3.2 (Accepted)** table for the four keyword
    buckets — TextMate scopes carry a `.mycelium` suffix (`TM_SCOPES`), tree-sitter captures
    (`TS_CAPTURES`) are the standard *unsuffixed* names. v2 adds the comment/string/number/
    operator/declaration scopes RFC-0026 §3.4 anticipated ("arrive with the full structural
    grammar"), using the standard TextMate / tree-sitter names for each layer.
  * The v2 STRUCTURAL grammar (tree-sitter) and the tmLanguage patterns are derived by hand from
    the normative oracle `docs/spec/grammar/mycelium.ebnf` + the L1 lexer's literal lexers
    (strings M-910, floats ADR-040/M-897, `0b`/`0t`/`0x` literals, RFC-0025/M-745 operators) —
    guarantee `Empirical`: verified by parsing the conformance accept corpus + `lib/std/*.myc`
    (zero ERROR nodes), not proven equivalent to the EBNF. Two DOCUMENTED permissive deviations
    (Declared): (1) reserved-not-active keywords parse as an explicit `reserved_keyword` atom
    (the L1 parser rejects them; a highlighting grammar wants them coloured in snippets/docs);
    (2) repr-type brace arities are checked by the checker, not this grammar. The EBNF stays the
    accept/reject oracle; these grammars are for identification + highlighting.

Usage:
  generate.py                 # (re)write the committed artifacts under tools/grammar/
  generate.py --check         # drift gate: fail (exit 2) if committed artifacts are stale
  generate.py --self-test     # extraction + template-coverage sanity (offline; exit 3 on failure)
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

# Repo-root-relative locations (this file lives at tools/grammar/generate.py).
HERE = Path(__file__).resolve().parent
REPO_ROOT = HERE.parent.parent
TOKEN_RS = REPO_ROOT / "crates" / "mycelium-l1" / "src" / "token.rs"

# Ratified RFC-0026 §3.2 scope names — standard TextMate / tree-sitter / LSP names with a
# `.mycelium` suffix (chosen for maximal theme compatibility; see RFC-0026 §3.2 / §5 Q2). The
# keyword/type/scalar/strength buckets are lexer-derived (§3.3); these tables fix the NAME each
# bucket renders to.
TM_SCOPES = {
    "keyword": "keyword.control.mycelium",
    "type": "storage.type.mycelium",
    "scalar": "support.type.builtin.mycelium",
    "strength": "storage.modifier.guarantee.mycelium",
}
# tree-sitter highlight captures (RFC-0026 §3.2).
TS_CAPTURES = {
    "keyword": "keyword",
    "type": "type",
    "scalar": "type.builtin",
    "strength": "attribute",
}

# The substrate/representation type keywords — identified by their bare `Tok::<Variant>` RHS.
# `Float` is the ADR-040 (M-897) nullary repr-type keyword; `BinShort`/`TernShort`/`EmbShort`/
# `HvecShort` are the RFC-0037 D2-b short repr aliases (M-915) — token.rs reserves them as "the
# same class as `Binary` itself", so they bucket as types here (a v2 bucket CORRECTION: v1
# misfiled `Float` under `keyword` because this set predated ADR-040).
TYPE_VARIANTS = {
    "Binary",
    "Ternary",
    "Dense",
    "Vsa",
    "Substrate",
    "Sparse",
    "Seq",
    "Bytes",
    "Float",
    "BinShort",
    "TernShort",
    "EmbShort",
    "HvecShort",
}

# Keywords the v2 structural tree-sitter grammar consumes by name (every entry below MUST appear
# quoted in the grammar template — asserted by --self-test). The keyword-bucket words NOT in this
# set are the reserved-not-active vocabulary (DN-03 §4 / EBNF header): they lex as keywords but no
# production consumes them; the grammar parses them as an explicit `reserved_keyword` atom
# (documented permissive deviation — see module docstring).
STRUCTURAL_KEYWORDS = {
    "colony",
    "consume",
    "default",
    "derive",
    "else",
    "fn",
    "for",
    "forage",
    "fuse",
    "hypha",
    "if",
    "impl",
    "in",
    "lambda",
    "let",
    "lower",
    "match",
    "nodule",
    "object",
    "paradigm",
    "phylum",
    "policy",
    "priv",
    "pub",
    "reclaim",
    "spore",
    "swap",
    "thaw",
    "then",
    "tier",
    "to",
    "trait",
    "type",
    "use",
    "via",
    "wild",
    "with",
}


def extract_keywords(token_rs: str) -> dict[str, list[str]]:
    """Extract the keyword set from the `keyword()` function body, bucketed by lexer class.

    The buckets are derived from each arm's right-hand side (`=> Tok::…`):
      * `Tok::Strength(...)`  -> `strength`  (the guarantee-strength lattice)
      * `Tok::Scalar(...)`    -> `scalar`    (F16/BF16/F32/F64)
      * `Tok::{Binary,...}`   -> `type`      (substrate/representation types + short aliases)
      * everything else       -> `keyword`   (declaration/control/runtime-vocabulary words)

    Never-silent: a `keyword()` body that cannot be located is a hard error, not an empty result.
    """
    m = re.search(
        r"pub fn keyword\(word: &str\) -> Option<Tok> \{(.*?)\n\}", token_rs, re.DOTALL
    )
    if not m:
        raise SystemExit(
            "drift: could not locate `pub fn keyword(...)` in token.rs (G2)"
        )
    body = m.group(1)

    buckets: dict[str, list[str]] = {
        "keyword": [],
        "type": [],
        "scalar": [],
        "strength": [],
    }
    # Each arm: "<word>" => Tok::<RHS>,   (the RHS may be `Strength(StrengthTok::Exact)` etc.)
    for word, rhs in re.findall(r'"([^"]+)"\s*=>\s*(Tok::[^,]+),', body):
        if rhs.startswith("Tok::Strength"):
            buckets["strength"].append(word)
        elif rhs.startswith("Tok::Scalar"):
            buckets["scalar"].append(word)
        elif rhs[len("Tok::") :] in TYPE_VARIANTS:
            buckets["type"].append(word)
        else:
            buckets["keyword"].append(word)

    for k in buckets:
        buckets[k] = sorted(set(buckets[k]))
    if not any(buckets.values()):
        raise SystemExit("drift: extracted an empty keyword set from token.rs (G2)")
    return buckets


def reserved_inactive(buckets: dict[str, list[str]]) -> list[str]:
    """The keyword-bucket words no structural production consumes (DN-03 §4 reserved set)."""
    return sorted(set(buckets["keyword"]) - STRUCTURAL_KEYWORDS)


def render_keywords_json(buckets: dict[str, list[str]]) -> str:
    """The canonical extracted snapshot — the artifact the drift gate diffs and the grammars build on."""
    doc = {
        "_generated_by": "tools/grammar/generate.py",
        "_source_of_truth": "crates/mycelium-l1/src/token.rs::keyword()",
        "_honesty": "keyword set + class buckets are lexer-derived (G2); scope names are the ratified RFC-0026 §3.2 table.",
        "classes": buckets,
    }
    return json.dumps(doc, indent=2) + "\n"


def _regex_alt(words: list[str]) -> str:
    """A `\\b`-anchored alternation matching exactly these whole words (longest-first, escaped)."""
    alts = "|".join(re.escape(w) for w in sorted(words, key=len, reverse=True))
    return rf"\b({alts})\b"


def render_tmlanguage(buckets: dict[str, list[str]]) -> str:
    """The full-surface TextMate grammar (v2).

    Pattern order is significant (first match wins): comments and strings first (so nothing
    re-matches inside them), then literals (grounded in the L1 lexers: `lex_binary`/`lex_trit`/
    `lex_hex_bytes`/`lex_int`/`lex_string`), the retired `->` (lexed only for a teaching reject —
    RFC-0037 D4 — so it renders as deprecated), declaration-name captures, the four ratified
    keyword buckets, attributes, capitalized type/constructor names, and operators last.
    """
    patterns: list[dict] = [
        # `//` line comment (the only comment form — mycelium.ebnf terminals note). The
        # `// nodule:`-header doc tags (`@version:` …, Nodule-Header spec) highlight inside it.
        {
            "name": "comment.line.double-slash.mycelium",
            "begin": "//",
            "end": "$",
            "patterns": [
                {
                    "name": "storage.type.annotation.header-tag.mycelium",
                    "match": "@[A-Za-z][A-Za-z0-9-]*:",
                }
            ],
        },
        # StrLit (M-910/M-911): minimal escape set `\n \t \\ \" \0 \r`; any other escape is a
        # lex error — rendered `invalid.illegal` (never silently styled as a good escape, G2).
        {
            "name": "string.quoted.double.mycelium",
            "begin": '"',
            "end": '"|$',
            "patterns": [
                {
                    "name": "constant.character.escape.mycelium",
                    "match": '\\\\[nt\\\\"0r]',
                },
                {"name": "invalid.illegal.escape.mycelium", "match": "\\\\."},
            ],
        },
        # Literals (BinLit / TritLit / BytesLit / FloatLit / Int — mycelium.ebnf terminals).
        {"name": "constant.numeric.binary.mycelium", "match": "\\b0b[01_]+\\b"},
        {"name": "constant.numeric.ternary.mycelium", "match": "\\b0t[+0-]+"},
        {"name": "constant.numeric.bytes.mycelium", "match": "\\b0x[0-9A-Fa-f_]+\\b"},
        {
            "name": "constant.numeric.float.mycelium",
            "match": "\\b[0-9]+\\.[0-9]+(?:[eE][+-]?[0-9]+)?\\b|\\b[0-9]+[eE][+-]?[0-9]+\\b",
        },
        {"name": "constant.numeric.integer.mycelium", "match": "\\b[0-9]+\\b"},
        # `->` is RETIRED as the return arrow (RFC-0037 D4 → `=>`); still lexed solely for a
        # teaching reject, so it renders as deprecated, never as a healthy operator.
        {"name": "invalid.deprecated.return-arrow.mycelium", "match": "->"},
        # Declaration-name captures (before the keyword buckets so the two-token match wins).
        {
            "match": "\\b(nodule|phylum|use)\\s+([A-Za-z_][A-Za-z0-9_.]*)",
            "captures": {
                "1": {"name": TM_SCOPES["keyword"]},
                "2": {"name": "entity.name.namespace.mycelium"},
            },
        },
        {
            "match": "\\b(fn)\\s+([A-Za-z_][A-Za-z0-9_]*)",
            "captures": {
                "1": {"name": TM_SCOPES["keyword"]},
                "2": {"name": "entity.name.function.mycelium"},
            },
        },
        {
            "match": "\\b(type|trait|object)\\s+([A-Za-z_][A-Za-z0-9_]*)",
            "captures": {
                "1": {"name": TM_SCOPES["keyword"]},
                "2": {"name": "entity.name.type.mycelium"},
            },
        },
        {
            "match": "\\b(lower|derive)\\s+([A-Za-z_][A-Za-z0-9_]*)",
            "captures": {
                "1": {"name": TM_SCOPES["keyword"]},
                "2": {"name": "entity.name.function.lowering.mycelium"},
            },
        },
        # `@`-attributes: exactly the landed forms — `@tier(mode)` (DN-58 §C), `@forage(policy)`
        # (DN-70 D1 / M-906), and the `@std-sys` header marker (M-661; lexed as ONE token). Bare
        # `@` (the guarantee annotation `T @ Strength`) falls through to the operator pattern.
        {
            "name": "entity.other.attribute-name.mycelium",
            "match": "@(?:tier|forage|std-sys)\\b",
        },
    ]
    # The four ratified keyword buckets (lexer-derived).
    for cls in ("keyword", "type", "scalar", "strength"):
        words = buckets[cls]
        if not words:
            continue
        patterns.append({"name": TM_SCOPES[cls], "match": _regex_alt(words)})
    patterns += [
        # Capitalized names: constructors + named types/type variables (after the buckets, so
        # `Binary`/`F32`/`Exact` keep their ratified bucket scopes).
        {
            "name": "entity.name.type.mycelium",
            "match": "\\b[A-Z][A-Za-z0-9_]*\\b",
        },
        # Operators (RFC-0025 / M-705 / M-745): `<=`/`>=` are RETIRED glyphs (RFC-0037 D1 —
        # `lte`/`gte` are word-form calls), so no two-char comparison alternates here.
        {
            "name": "keyword.operator.mycelium",
            "match": "<<|>>|=>|==|!=|&&|\\|\\||[-+*/%^&|<>=!@]",
        },
        {"name": "punctuation.terminator.mycelium", "match": ";"},
        {"name": "punctuation.separator.mycelium", "match": "[,.:]"},
    ]

    grammar = {
        "$schema": "https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json",
        "name": "Mycelium",
        "scopeName": "source.mycelium",
        "fileTypes": ["myc"],
        "_generated_by": "tools/grammar/generate.py — DO NOT EDIT BY HAND",
        "_scope_names_status": "RATIFIED — RFC-0026 §3.2 (Accepted) for the four keyword buckets; the v2 comment/string/literal/operator/declaration scopes are the standard TextMate names (RFC-0026 §3.4 follow-up, Empirical vs the conformance corpus).",
        "patterns": patterns,
    }
    return json.dumps(grammar, indent=2) + "\n"


def _js_array(words: list[str]) -> str:
    """A JS array literal of single-quoted words."""
    return "[" + ", ".join(f"'{w}'" for w in words) + "]"


# The v2 structural tree-sitter grammar template. `__KW_*__` placeholders are injected from the
# lexer-derived buckets; everything else is derived by hand from docs/spec/grammar/mycelium.ebnf
# (see module docstring for the honesty tags + the two documented permissive deviations).
GRAMMAR_JS_TEMPLATE = """// tree-sitter grammar for Mycelium — GENERATED by tools/grammar/generate.py. DO NOT EDIT.
//
// Status (RFC-0026 §3.4 follow-up; v2): STRUCTURAL grammar derived from the normative oracle
// docs/spec/grammar/mycelium.ebnf + the L1 lexer (crates/mycelium-l1/src/{lexer,token}.rs). The
// keyword sets below are lexer-derived and kept in lockstep by `just drift-check`; the structural
// productions are Empirical — verified by parsing the conformance accept corpus and lib/std/*.myc
// with zero ERROR nodes, not proven equivalent to the EBNF (which stays the accept/reject oracle).
// Two DOCUMENTED permissive deviations (Declared, G2): (1) reserved-not-active keywords parse as
// an explicit `reserved_keyword` atom (the L1 parser rejects them; a highlighting grammar renders
// them in snippets/docs); (2) repr-type brace arities are checker territory, not enforced here.
// Highlight captures are in queries/highlights.scm.

// GENERATED keyword sets (lexer-derived, G2 — the drift gate keeps these current).
const KW_KEYWORD = __KW_KEYWORD__;
const KW_TYPE = __KW_TYPE__;
const KW_SCALAR = __KW_SCALAR__;
const KW_STRENGTH = __KW_STRENGTH__;
// Reserved-not-active (DN-03 §4; EBNF header): lexed as keywords, consumed by no production.
const KW_RESERVED_INACTIVE = __KW_RESERVED__;

// RFC-0025 §4.1 tiers (M-705/M-745), written loosest→tightest; every binary operator is
// left-associative, prefix operators bind tighter than every binary operator.
const PREC = {
  OR: 2, AND: 3, EQ: 4, CMP: 5, BOR: 6, XOR: 7, BAND: 8, SHIFT: 9, ADD: 10, MUL: 11,
  UNARY: 12, ASCRIPTION: 13, CALL: 14,
};

function sep1(rule, sep) {
  return seq(rule, repeat(seq(sep, rule)));
}

module.exports = grammar({
  name: 'mycelium',
  word: $ => $.identifier,
  extras: $ => [/\\s/, $.comment],

  rules: {
    // program ::= phylum_header? nodule_block+ — flattened to a component list: the DN-57 §3
    // mandatory `;` terminator makes every component self-delimiting, so the flat form parses
    // the same programs (nodule_header opens a block; items follow until the next header).
    source_file: $ => seq(
      optional($.phylum_header),
      repeat(choice($.nodule_header, seq($._item, ';'))),
    ),

    // ── headers ────────────────────────────────────────────────────────────────────────────
    phylum_header: $ => seq('phylum', field('name', $._path)),
    // The `@std-sys` marker (M-661) is ONE atomic token (the `-` could not lex as `@` + ident).
    nodule_header: $ => seq('nodule', field('name', $._path), optional($.std_sys_marker), ';'),
    std_sys_marker: $ => token('@std-sys'),

    // ── items ──────────────────────────────────────────────────────────────────────────────
    _item: $ => choice(
      $.use_item, $.default_item, $.type_item, $.trait_item, $.impl_item,
      $.fn_item, $.object_item, $.lower_item, $.derive_item,
    ),

    // the glob tail `.*` lexes as ONE token, so `use a.b.*` never conflicts with a longer path.
    use_item: $ => seq('use', field('path', $._path), optional(token('.*'))),

    default_item: $ => seq('default', 'paradigm', $.paradigm),
    paradigm: $ => choice('Binary', 'Ternary', 'Dense', 'VSA'),

    type_item: $ => seq(
      optional('pub'), 'type', field('name', $.identifier), optional($.type_params),
      '=', sep1($.constructor, '|'),
    ),
    constructor: $ => seq(
      optional('priv'),
      field('name', $.identifier),
      optional(seq('(', sep1($.type_ref, ','), ')')),
    ),

    trait_item: $ => seq(
      optional('pub'), 'trait', field('name', $.identifier), optional($.type_params),
      '{', repeat(seq($.fn_sig, ';')), '}',
    ),
    fn_sig: $ => seq(
      'fn', field('name', $.identifier), optional($.type_params), optional($.const_params),
      '(', optional($.params), ')', '=>', field('return_type', $.type_ref), optional($.effects),
    ),

    // impl: trait-instance `impl Trait[args]? for T { … }` / inherent `impl T { … }` (M-659/
    // M-664) — unified head (a trait head `Ident type_args?` is a named type_ref).
    impl_item: $ => seq(
      'impl', field('trait_or_type', $.type_ref),
      optional(seq('for', field('type', $.type_ref))),
      '{', repeat(seq($.fn_item, ';')), '}',
    ),

    fn_item: $ => seq(
      optional('pub'), optional($.tier_attribute), optional('thaw'),
      'fn', field('name', $.identifier), optional($.type_params), optional($.const_params),
      '(', optional($.params), ')', '=>', field('return_type', $.type_ref), optional($.effects),
      '=', field('body', $._expr),
    ),
    // `@tier(compiled|interpreted)` (DN-58 §C; M-667) — the mode names are plain identifiers in
    // the lexer (RFC-0004 surface spellings, validated by the checker, not the grammar).
    tier_attribute: $ => seq('@', 'tier', '(', field('mode', $.identifier), ')'),

    // object composition (DN-53 / M-811): one constructor clause, then via/impl/fn members.
    object_item: $ => seq(
      optional('pub'), 'object', field('name', $.identifier), optional($.type_params),
      '{', $.constructor, ';', repeat(seq($._object_member, ';')), '}',
    ),
    _object_member: $ => choice($.via_clause, $.impl_item, $.fn_item),
    via_clause: $ => seq('via', field('index', $.int_literal), ':', field('trait', $.identifier), optional($.type_args)),

    // user-extensible generative lowering (DN-54 / M-812).
    lower_item: $ => seq('lower', field('name', $.identifier), optional($.type_params), '=', field('body', $._expr)),
    derive_item: $ => seq('derive', field('name', $.identifier), 'for', field('type', $.type_ref)),

    params: $ => sep1($.param, ','),
    param: $ => seq(field('name', $.identifier), ':', field('type', $.type_ref)),

    // effect annotation `!{ … }` (RFC-0014 §3.4; M-660): names are plain identifiers.
    effects: $ => seq('!', '{', optional(sep1($.identifier, ',')), '}'),

    // ── types ──────────────────────────────────────────────────────────────────────────────
    // type_ref ::= base_type ('@' strength)? ('=>' type_ref)? — the guarantee tag as a
    // type-level index (LR-6), plus the FUNCTION TYPE arrow (RFC-0024 §3 / RFC-0037 D4:
    // right-associative `A => B => C`; `@` binds tighter than `=>` — parse.rs
    // parse_type_ref_guarded; NOTE: implemented in the L1 parser, not yet in mycelium.ebnf).
    type_ref: $ => prec.right(seq(
      $._base_type,
      optional(seq('@', field('strength', $.strength))),
      optional(seq('=>', field('return', $.type_ref))),
    )),
    strength: $ => choice(...KW_STRENGTH),
    scalar_type: $ => choice(...KW_SCALAR),

    _base_type: $ => choice($.repr_type, $.tuple_type, $.ambient_type, $.named_type),
    // Repr types + the RFC-0037 D2-b short aliases (M-915). Brace ARITIES are checker territory
    // (documented deviation #2): each takes a comma list of width/scalar/sparsity/type args.
    repr_type: $ => choice(
      seq(
        // `Sparse` is NOT a repr head — it appears only inside the `sparsity` spec (EBNF).
        field('paradigm', choice(
          'Binary', 'Ternary', 'Dense', 'VSA', 'bin', 'tern', 'emb', 'hvec',
          'Substrate', 'Seq',
        )),
        '{', sep1($._repr_arg, ','), '}',
      ),
      'Bytes',
      'Float',
    ),
    _repr_arg: $ => choice($.int_literal, $.scalar_type, $.sparsity, $.type_ref),
    sparsity: $ => choice('Dense', seq('Sparse', '{', choice($.int_literal, $.identifier), '}')),
    tuple_type: $ => seq('(', $.type_ref, ',', sep1($.type_ref, ','), ')'),
    // paradigm-less repr `{…}` (RFC-0012 §4.2): the ambient supplies the paradigm.
    ambient_type: $ => seq('{', sep1(choice($.int_literal, $.identifier, $.scalar_type, $.sparsity), ','), '}'),
    named_type: $ => prec.right(seq(field('name', $.identifier), optional($.type_args))),
    type_args: $ => seq('[', sep1($.type_ref, ','), ']'),
    type_params: $ => seq('[', sep1($.identifier, ','), ']'),
    const_params: $ => seq('{', sep1($.identifier, ','), '}'),

    // ── expressions ────────────────────────────────────────────────────────────────────────
    _expr: $ => choice(
      $.let_expr, $.if_expr, $.match_expr, $.for_expr, $.swap_expr, $.with_expr,
      $.wild_expr, $.spore_expr, $.consume_expr, $.colony_expr, $.fuse_expr,
      $.reclaim_expr, $.lambda_expr, $._op_expr,
    ),

    let_expr: $ => prec.right(seq(
      'let', field('name', $.identifier), optional(seq(':', field('type', $.type_ref))),
      '=', field('value', $._expr), 'in', field('body', $._expr),
    )),
    if_expr: $ => prec.right(seq(
      'if', field('condition', $._expr), 'then', field('consequence', $._expr),
      'else', field('alternative', $._expr),
    )),
    match_expr: $ => seq(
      'match', field('value', $._expr), '{', sep1($.match_arm, ','), optional(','), '}',
    ),
    // or-patterns `A | B => e` (M-823 / R20-Q3).
    match_arm: $ => seq(sep1($.pattern, '|'), '=>', field('value', $._expr)),
    pattern: $ => choice(
      '_',
      $._literal,
      $.tuple_pattern,
      seq(field('name', $.identifier), optional(seq('(', sep1($.pattern, ','), ')'))),
    ),
    tuple_pattern: $ => seq('(', $.pattern, ',', sep1($.pattern, ','), ')'),

    // for: bounded structural fold (RFC-0007 §4.8).
    for_expr: $ => seq(
      'for', field('binder', $.identifier), 'in', field('iterable', $._app_expr),
      ',', field('accumulator', $.identifier), '=', field('init', $._app_expr),
      '=>', field('body', $._expr),
    ),

    // swap: the never-silent representation change — target + policy ALWAYS lexical (S1/WF2).
    swap_expr: $ => seq(
      'swap', '(', field('value', $._expr), ',', 'to', ':', field('to', $.type_ref),
      ',', 'policy', ':', field('policy', $._path), ')',
    ),
    with_expr: $ => seq('with', 'paradigm', $.paradigm, '{', field('body', $._expr), '}'),
    wild_expr: $ => seq('wild', '{', field('body', $._expr), '}'),
    spore_expr: $ => seq('spore', '(', field('value', $._expr), ')'),
    consume_expr: $ => prec(PREC.UNARY, seq('consume', field('value', $._app_expr))),

    // colony/hypha (RFC-0008 §4.7; M-666) with optional `@forage(policy)` (DN-70 D1; M-906).
    colony_expr: $ => seq('colony', '{', sep1($.hypha, ','), optional(','), '}'),
    hypha: $ => seq(optional($.forage_attribute), 'hypha', field('value', $._app_expr)),
    forage_attribute: $ => seq('@', 'forage', '(', field('policy', $._expr), ')'),

    fuse_expr: $ => seq('fuse', '(', field('left', $._expr), ',', field('right', $._expr), ')'),
    reclaim_expr: $ => seq(
      'reclaim', '(', field('policy', $._expr), ')', '{', field('body', $._expr), '}',
    ),
    lambda_expr: $ => prec.right(seq(
      'lambda', '(', optional($.params), ')', '=>', field('body', $._expr),
    )),

    // operator layer (RFC-0025 / M-705 / M-745): `<=`/`>=` glyphs are RETIRED (RFC-0037 D1) —
    // `lte`/`gte` are ordinary word-form calls, so they appear as call_expr, not operators.
    _op_expr: $ => choice($.binary_expr, $.unary_expr, $._app_expr),
    binary_expr: $ => choice(
      prec.left(PREC.OR, seq(field('left', $._op_expr), field('operator', '||'), field('right', $._op_expr))),
      prec.left(PREC.AND, seq(field('left', $._op_expr), field('operator', '&&'), field('right', $._op_expr))),
      prec.left(PREC.EQ, seq(field('left', $._op_expr), field('operator', choice('==', '!=')), field('right', $._op_expr))),
      prec.left(PREC.CMP, seq(field('left', $._op_expr), field('operator', choice('<', '>')), field('right', $._op_expr))),
      prec.left(PREC.BOR, seq(field('left', $._op_expr), field('operator', '|'), field('right', $._op_expr))),
      prec.left(PREC.XOR, seq(field('left', $._op_expr), field('operator', '^'), field('right', $._op_expr))),
      prec.left(PREC.BAND, seq(field('left', $._op_expr), field('operator', '&'), field('right', $._op_expr))),
      prec.left(PREC.SHIFT, seq(field('left', $._op_expr), field('operator', choice('<<', '>>')), field('right', $._op_expr))),
      prec.left(PREC.ADD, seq(field('left', $._op_expr), field('operator', choice('+', '-')), field('right', $._op_expr))),
      prec.left(PREC.MUL, seq(field('left', $._op_expr), field('operator', choice('*', '/', '%')), field('right', $._op_expr))),
    ),
    unary_expr: $ => prec.right(PREC.UNARY, seq(field('operator', choice('-', '!')), field('operand', $._op_expr))),

    // app_expr ::= primary calls* (':' type_ref)? — ascription binds looser than a call,
    // tighter than every operator.
    _app_expr: $ => choice($.call_expr, $.ascription_expr, $._primary),
    call_expr: $ => prec.left(PREC.CALL, seq(
      field('function', $._app_expr), '(', optional(sep1($._expr, ',')), ')',
    )),
    ascription_expr: $ => prec.left(PREC.ASCRIPTION, seq(
      field('value', $._app_expr), ':', field('type', $.type_ref),
    )),

    _primary: $ => choice(
      $._literal, $.identifier, $.path, $.tuple_expr, $.paren_expr, $.reserved_keyword,
    ),
    tuple_expr: $ => seq('(', $._expr, ',', sep1($._expr, ','), ')'),
    paren_expr: $ => seq('(', $._expr, ')'),
    // a dotted path has >= 2 segments; `_path` admits the single-segment (bare-ident) form.
    _path: $ => choice($.identifier, $.path),
    path: $ => prec.left(seq($.identifier, repeat1(seq('.', $.identifier)))),

    // Documented permissive deviation #1: the reserved-not-active vocabulary parses as an
    // explicit atom (the L1 parser REJECTS these — conformance/reject/12; a highlighting
    // grammar renders them in snippets/docs rather than erroring).
    reserved_keyword: $ => choice(...KW_RESERVED_INACTIVE),

    // ── literals (lexer terminals; mycelium.ebnf §Terminals) ───────────────────────────────
    _literal: $ => choice(
      $.bin_literal, $.trit_literal, $.bytes_literal, $.string_literal,
      $.float_literal, $.int_literal, $.list_literal,
    ),
    bin_literal: $ => token(/0b[01_]+/),
    trit_literal: $ => token(/0t[+0-]+/),
    bytes_literal: $ => token(/0x[0-9A-Fa-f_]+/),
    // FloatLit (ADR-040; M-897): digits `.` digits exponent? | digits exponent. `1.` stays
    // Int + `.` (the path glyph) — no trailing-dot float, no leading-dot `.5`.
    float_literal: $ => token(choice(
      /[0-9]+\\.[0-9]+([eE][+-]?[0-9]+)?/,
      /[0-9]+[eE][+-]?[0-9]+/,
    )),
    int_literal: $ => token(/[0-9]+/),
    // StrLit (M-910/M-911): minimal escape set; no raw newline/CR inside the literal.
    string_literal: $ => seq(
      '"',
      repeat(choice($.escape_sequence, token.immediate(prec(1, /[^"\\\\\\n\\r]+/)))),
      '"',
    ),
    escape_sequence: $ => token.immediate(/\\\\[nt\\\\"0r]/),
    list_literal: $ => seq('[', optional(sep1($._expr, ',')), ']'),

    comment: $ => token(seq('//', /[^\\n]*/)),
    identifier: $ => /[A-Za-z_][A-Za-z0-9_]*/,
  },
});
"""


def render_tree_sitter(buckets: dict[str, list[str]]) -> str:
    """Inject the lexer-derived keyword sets into the structural grammar template."""
    out = GRAMMAR_JS_TEMPLATE
    out = out.replace("__KW_KEYWORD__", _js_array(buckets["keyword"]))
    out = out.replace("__KW_TYPE__", _js_array(buckets["type"]))
    out = out.replace("__KW_SCALAR__", _js_array(buckets["scalar"]))
    out = out.replace("__KW_STRENGTH__", _js_array(buckets["strength"]))
    out = out.replace("__KW_RESERVED__", _js_array(reserved_inactive(buckets)))
    return out


def render_highlights_scm(buckets: dict[str, list[str]]) -> str:
    """tree-sitter highlight queries: the ratified RFC-0026 §3.2 bucket captures (generated) plus
    the v2 structural captures (comments/strings/literals/declarations/operators)."""
    lines = [
        "; tree-sitter highlight queries for Mycelium — GENERATED by tools/grammar/generate.py.",
        "; Bucket captures are the ratified RFC-0026 §3.2 (Accepted) names; the structural",
        "; captures are the standard tree-sitter names (RFC-0026 §3.4 follow-up).",
        "",
        "; comments + strings + literals",
        "(comment) @comment",
        "(string_literal) @string",
        "(escape_sequence) @string.escape",
        "(bin_literal) @number",
        "(trit_literal) @number",
        "(bytes_literal) @number",
        "(float_literal) @number",
        "(int_literal) @number",
        "",
        "; declarations",
        "(fn_item name: (identifier) @function)",
        "(fn_sig name: (identifier) @function)",
        "(type_item name: (identifier) @type)",
        "(trait_item name: (identifier) @type)",
        "(object_item name: (identifier) @type)",
        "(constructor name: (identifier) @constructor)",
        "(lower_item name: (identifier) @function)",
        "(derive_item name: (identifier) @function)",
        "(named_type name: (identifier) @type)",
        "(param name: (identifier) @variable.parameter)",
        "(nodule_header name: [(path) (identifier)] @module)",
        "(phylum_header name: [(path) (identifier)] @module)",
        "(use_item path: [(path) (identifier)] @module)",
        "",
        "; calls (the last path segment is the callee)",
        "(call_expr function: (identifier) @function.call)",
        "(call_expr function: (path (identifier) @function.call .))",
        "",
        "; attributes + markers",
        "(std_sys_marker) @attribute",
        "(tier_attribute) @attribute",
        '(forage_attribute "@" @attribute "forage" @attribute)',
        "",
    ]
    lines.append("; keyword buckets (lexer-derived, G2)")
    for cls in ("keyword", "type", "scalar", "strength"):
        words = buckets[cls]
        if not words:
            continue
        alt = " ".join(f'"{w}"' for w in words)
        lines.append(f"[{alt}] @{TS_CAPTURES[cls]}")
    lines += [
        "",
        "; operators (RFC-0025; `<=`/`>=` glyphs retired — RFC-0037 D1) + punctuation",
        '["||" "&&" "==" "!=" "<" ">" "<<" ">>" "|" "^" "&" "+" "-" "*" "/" "%" "=" "=>" "!" "@"] @operator',
        '["(" ")" "{" "}" "[" "]"] @punctuation.bracket',
        '[";" "," "." ":"] @punctuation.delimiter',
        "",
        "; wildcards + reserved vocabulary",
        '"_" @variable.builtin',
        "(reserved_keyword) @keyword",
    ]
    return "\n".join(lines) + "\n"


# (relative path under tools/grammar) -> renderer
ARTIFACTS = {
    "keywords.json": render_keywords_json,
    "mycelium.tmLanguage.json": render_tmlanguage,
    "tree-sitter-mycelium/grammar.js": render_tree_sitter,
    "tree-sitter-mycelium/queries/highlights.scm": render_highlights_scm,
}

# DOWNSTREAM COPIES of a canonical artifact, keyed by REPO_ROOT-relative path -> renderer. These
# are byte-identical build-time copies of a `tools/grammar/` artifact that must live inside another
# consumer's tree. Emitting + drift-checking them HERE (rather than via a consumer-side npm script)
# makes the copy impossible to silently stale vs the lexer-derived canonical grammar (G2): the same
# `just drift-check` gate that guards `tools/grammar/` now also guards the copy. Only processed on
# an in-place run (default --output-dir); a custom --output-dir renders the four canonical artifacts
# only (used by the offline determinism test), so these fixed repo-paths are never written elsewhere.
EXTRA_COPIES = {
    # The VS Code / Cursor extension ships the tmLanguage inside its own package (editors/vscode);
    # this copy is what `contributes.grammars` points at. `npm run sync-grammar` remains as a manual
    # convenience, but THIS gate is the authority — the copy cannot drift from the canonical grammar.
    "editors/vscode/syntaxes/mycelium.tmLanguage.json": render_tmlanguage,
}


def generate(buckets: dict[str, list[str]]) -> dict[str, str]:
    """Render every canonical artifact to its string content (the in-memory truth the gate diffs)."""
    return {rel: render(buckets) for rel, render in ARTIFACTS.items()}


def generate_extra(buckets: dict[str, list[str]]) -> dict[str, str]:
    """Render the downstream copies, keyed by REPO_ROOT-relative path."""
    return {rel: render(buckets) for rel, render in EXTRA_COPIES.items()}


def write(
    out_dir: Path, rendered: dict[str, str], extra: dict[str, str] | None = None
) -> None:
    for rel, content in rendered.items():
        path = out_dir / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
    for rel, content in (extra or {}).items():
        path = REPO_ROOT / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")


def check(
    out_dir: Path, rendered: dict[str, str], extra: dict[str, str] | None = None
) -> int:
    """Drift gate: committed artifacts (+ downstream copies) must match a fresh regeneration.

    Exit 2 on any drift (G2). A downstream copy that is MISSING is drift too (never-silent): once
    the consumer exists in-tree, the gate requires its copy present and current.
    """
    stale: list[str] = []
    for rel, content in rendered.items():
        path = out_dir / rel
        if not path.exists() or path.read_text(encoding="utf-8") != content:
            stale.append(f"tools/grammar/{rel}")
    for rel, content in (extra or {}).items():
        path = REPO_ROOT / rel
        if not path.exists() or path.read_text(encoding="utf-8") != content:
            stale.append(rel)
    if stale:
        print(
            "drift: the committed editor grammars are stale vs the lexer keyword() table:"
        )
        for rel in stale:
            print(f"  - {rel}")
        print(
            "fix: run `python3 tools/grammar/generate.py` and commit the result (G2)."
        )
        return 2
    print("grammar artifacts are current with the lexer keyword() table")
    return 0


def self_test(buckets: dict[str, list[str]]) -> int:
    """Offline sanity: bucket placement, template keyword coverage, and determinism.

    The coverage assertions are the v2 drift LINKAGE for the structural templates: every keyword
    the lexer knows must appear in every rendered artifact, and every STRUCTURAL_KEYWORDS entry
    must still exist in the lexer — so a lexer keyword addition/rename that the templates miss
    fails loudly here (G2), never silently drops a word from highlighting.
    """
    failures = []
    if "nodule" not in buckets["keyword"]:
        failures.append("`nodule` should be a keyword")
    if "Binary" not in buckets["type"]:
        failures.append("`Binary` should be a type")
    if "Float" not in buckets["type"]:
        failures.append("`Float` should be a type (ADR-040 repr-type keyword)")
    for short in ("bin", "tern", "emb", "hvec"):
        if short not in buckets["type"]:
            failures.append(
                f"`{short}` should be a type (RFC-0037 D2-b short repr alias, M-915)"
            )
    if "Exact" not in buckets["strength"]:
        failures.append("`Exact` should be a strength")
    if "F32" not in buckets["scalar"]:
        failures.append("`F32` should be a scalar")

    all_words = {w for words in buckets.values() for w in words}
    # STRUCTURAL_KEYWORDS ⊆ the lexer's keyword bucket (a renamed/removed keyword fails here).
    for kw in sorted(STRUCTURAL_KEYWORDS - set(buckets["keyword"])):
        failures.append(
            f"STRUCTURAL_KEYWORDS entry `{kw}` is not in the lexer keyword bucket"
        )
    # Every lexer keyword appears in every rendered artifact (nothing silently un-highlighted).
    rendered = generate(buckets)
    for rel, content in rendered.items():
        missing = sorted(
            w
            for w in all_words
            if f"'{w}'" not in content
            and f'"{w}"' not in content
            and f"|{w}" not in content
            and f"({w}" not in content
            and f"{w}|" not in content
            and f"{w})" not in content
        )
        if missing:
            failures.append(f"{rel}: lexer keywords missing from artifact: {missing}")
    if generate(buckets) != generate(buckets):
        failures.append("generation is not deterministic")
    if failures:
        for f in failures:
            print(f"self-test FAIL: {f}")
        return 3
    print("grammar generator self-test passed (extraction + coverage + determinism)")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--check",
        action="store_true",
        help="drift gate: fail if committed artifacts are stale",
    )
    ap.add_argument(
        "--self-test",
        action="store_true",
        help="offline extraction/coverage/determinism sanity check",
    )
    ap.add_argument(
        "--output-dir",
        default=str(HERE),
        help="where artifacts live (default: tools/grammar/)",
    )
    args = ap.parse_args()

    buckets = extract_keywords(TOKEN_RS.read_text(encoding="utf-8"))
    out_dir = Path(args.output_dir)

    if args.self_test:
        return self_test(buckets)
    rendered = generate(buckets)
    # Downstream copies are fixed REPO_ROOT paths — only processed on an in-place run (the drift
    # gate + a plain regenerate). A custom --output-dir renders the four canonical artifacts only.
    in_place = out_dir.resolve() == HERE.resolve()
    extra = generate_extra(buckets) if in_place else None
    if args.check:
        return check(out_dir, rendered, extra)
    write(out_dir, rendered, extra)
    n = len(rendered) + (len(extra) if extra else 0)
    total = sum(len(v) for v in buckets.values())
    print(f"wrote {n} grammar artifact(s) from {total} lexer keywords -> {out_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
