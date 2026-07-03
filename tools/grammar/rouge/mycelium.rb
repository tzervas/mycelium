# frozen_string_literal: true

# Mycelium lexer for Rouge (GitLab's syntax highlighter).
#
# STATUS: draft, UNSUBMITTED (M-697). This file is not part of `tools/grammar/generate.py`'s
# drift-gated output (`just drift-check` does not cover it) — it is a hand-written derivation,
# kept honest by citing its sources below rather than by an automated regeneration gate. If the
# lexer keyword/token layer (`crates/mycelium-l1/src/token.rs`) changes, this file must be
# updated by hand; it will silently drift otherwise (flag this to a human reviewer, G2).
#
# Keyword-class source: `tools/grammar/keywords.json` (as committed alongside this draft),
# itself generated from `crates/mycelium-l1/src/token.rs::keyword()`. The four buckets below
# (`KEYWORDS`, `TYPE_KEYWORDS`, `SCALAR_KEYWORDS`, `STRENGTH_KEYWORDS`) are a verbatim copy of
# that file's `classes` map at the time this draft was written (2026-07-02) — **the generator
# owns the canonical set**; re-copy from `tools/grammar/keywords.json` after any lexer keyword
# change rather than hand-editing these arrays independently.
#
# Literal/operator/escape grammar grounded directly in `crates/mycelium-l1/src/{token,lexer}.rs`
# (not the TextMate/tree-sitter artifacts, which may lag a landed lexer change until their own
# `just grammar-gen` run): binary `0b[01_]+`, balanced-ternary `0t[+0-]+` (RFC-0037 D4 — the
# lexer's `TritLit`, MSB-first, retired the earlier `<+0-…>` angle form), bytes `0x[hex_]+`
# (RFC-0032 D4/M-750 `BytesLit` — an even-hex-digit byte string, *not* a hex integer; there is no
# separate hex-integer literal in the surface grammar), decimal float/int, the RFC-0025 §4.1
# operator-glyph table (`<<`/`>>`/`=>`/`==`/`!=`/`&&`/`||`/`&`/`|`/`+`/`-`/`*`/`/`/`%`/`^`/`<`/`>`/
# `=`/`!`/`@`), the retired `->` return arrow (RFC-0037 D4 — still lexed, but as an explicit
# teaching-reject, never a silent accept), and the minimal string escape set `\n \t \\ \" \0 \r`
# (an unrecognized escape is a lexer-level `ParseError` — here rendered as `Error`).

module Rouge
  module Lexers
    class Mycelium < RegexLexer
      title "Mycelium"
      desc "The Mycelium multi-paradigm value-semantics language (github.com/tzervas/mycelium)"
      tag 'mycelium'
      filenames '*.myc'
      mimetypes 'text/x-mycelium'

      # --- keyword classes (source: tools/grammar/keywords.json, snapshot 2026-07-02) ---

      def self.keywords
        @keywords ||= Set.new(%w(
          Float backbone colony consume cyst default derive else fn for forage
          fuse graft grow hypha if impl in lambda let lower match matured mesh
          nodule object paradigm phylum policy pub reclaim spore swap thaw then
          tier to trait type use via wild with xloc
        ))
      end

      def self.type_keywords
        @type_keywords ||= Set.new(%w(Binary Bytes Dense Seq Sparse Substrate Ternary VSA))
      end

      def self.scalar_keywords
        @scalar_keywords ||= Set.new(%w(BF16 F16 F32 F64))
      end

      def self.strength_keywords
        @strength_keywords ||= Set.new(%w(Declared Empirical Exact Proven))
      end

      # `@tier(...)`/`@forage(...)` are attribute forms on `@` + an ordinary keyword identifier
      # (DN-58 §C / DN-70 D1); `@std-sys` is the one atomic nodule-header marker token
      # (M-661) — all three render as Name::Decorator here.
      def self.decorator_words
        @decorator_words ||= Set.new(%w(tier forage))
      end

      id_regex = /[A-Za-z_][A-Za-z0-9_]*/

      state :root do
        rule %r/\s+/m, Text::Whitespace

        rule %r(//.*$), Comment::Single

        # --- literals (order matters: prefix-specific forms before the bare-decimal fallback) ---
        rule %r/0b[01_]+/, Num::Bin
        rule %r/0t[+0\-]+/, Num # balanced ternary — no dedicated Rouge token subtype (Declared)
        rule %r/0x[0-9a-fA-F_]+/, Num::Hex # a bytes literal (RFC-0032 D4), not a hex integer
        rule %r/\d[\d_]*\.\d[\d_]*([eE][+-]?\d+)?/, Num::Float
        rule %r/\d[\d_]*[eE][+-]?\d+/, Num::Float
        rule %r/\d[\d_]*/, Num::Integer

        rule %r/"/, Str::Double, :string

        # --- decorators (must precede the bare `@` operator rule) ---
        rule %r/@std-sys\b/, Name::Decorator
        rule %r/@(?:#{Regexp.union(Mycelium.decorator_words.to_a)})\b/, Name::Decorator

        # --- the retired `->` return arrow: an explicit reject token, never silently accepted ---
        rule %r/->/, Error

        # --- declaration names: `fn NAME` / `type|trait|object NAME` ---
        rule %r/\b(fn)(\s+)(#{id_regex})/ do
          groups Keyword, Text::Whitespace, Name::Function
        end
        rule %r/\b(type|trait|object)(\s+)(#{id_regex})/ do
          groups Keyword, Text::Whitespace, Name::Class
        end

        # --- word buckets (lexer-derived; see the file header) ---
        rule %r/\b#{id_regex}\b/ do |m|
          word = m[0]
          if self.class.keywords.include?(word)
            token Keyword
          elsif self.class.type_keywords.include?(word)
            token Keyword::Type
          elsif self.class.scalar_keywords.include?(word)
            token Name::Builtin
          elsif self.class.strength_keywords.include?(word)
            token Keyword::Constant
          elsif word =~ /\A[A-Z]/
            # capitalized, not a reserved word: a nullary/data constructor or type name
            # (`Some`, `None`, `True`, `False`, a user `type` name, …).
            token Name::Constant
          else
            token Name
          end
        end

        # `@` on its own is the bare guarantee-annotation glyph (`T @ Exact`), not a decorator.
        rule %r/@/, Operator

        # --- operators (two-char forms first so alternation doesn't stop at a one-char prefix) ---
        rule %r/(?:<<|>>|=>|==|!=|&&|\|\|)/, Operator
        rule %r/[&|+\-*\/%\^<>=!]/, Operator

        # --- punctuation ---
        rule %r/[(){}\[\]:,;.]/, Punctuation
      end

      state :string do
        rule %r/"/, Str::Double, :pop!
        rule %r/\\n/, Str::Escape
        rule %r/\\t/, Str::Escape
        rule %r/\\\\/, Str::Escape
        rule %r/\\"/, Str::Escape
        rule %r/\\0/, Str::Escape
        rule %r/\\r/, Str::Escape
        # any other `\x` is an unrecognized escape — the lexer raises an explicit ParseError
        # (never-silent, G2); rendered here as an Error token rather than accepted as text.
        rule %r/\\./, Error
        rule %r/[^\\"]+/, Str::Double
      end
    end
  end
end
