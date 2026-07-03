# -*- coding: utf-8 -*- #
# frozen_string_literal: true

# Draft spec for the Mycelium Rouge lexer (M-697), written in Rouge's own spec format
# (`describe`/`it` + `Support::Lexing#assert_tokens_equal` / `#assert_no_errors`,
# `Support::Guessing#assert_guess`) so it can be dropped into `rouge-ruby/rouge`'s
# `spec/lexers/mycelium_spec.rb` with only the `require` path adjusted (Rouge's own
# `spec/spec_helper.rb` is loaded automatically by its `.rspec`/Rakefile in that repo; this file
# assumes that harness, matching `spec/lexers/ruby_spec.rb`'s shape). TESTED status: see this
# directory's README.md for exactly what was run and how, in this task, without a full Rouge
# checkout.

describe Rouge::Lexers::Mycelium do
  let(:subject) { Rouge::Lexers::Mycelium.new }

  describe 'guessing' do
    include Support::Guessing

    it 'guesses by filename' do
      assert_guess :filename => 'foo.myc'
    end

    it 'guesses by mimetype' do
      assert_guess :mimetype => 'text/x-mycelium'
    end
  end

  describe 'lexing' do
    include Support::Lexing

    describe 'comments' do
      it 'lexes a line comment' do
        assert_tokens_equal '// a comment',
          ['Comment.Single', '// a comment']
      end
    end

    describe 'numeric literals' do
      it 'lexes a binary literal' do
        assert_tokens_equal '0b1011_0010',
          ['Literal.Number.Bin', '0b1011_0010']
      end

      it 'lexes a balanced-ternary literal' do
        assert_no_errors('0t+0--0')
      end

      it 'lexes a bytes literal (0x…, not a hex integer — RFC-0032 D4)' do
        assert_tokens_equal '0xDE_AD_BE_EF',
          ['Literal.Number.Hex', '0xDE_AD_BE_EF']
      end

      it 'distinguishes Float from Integer' do
        assert_tokens_equal '1.5',
          ['Literal.Number.Float', '1.5']
        assert_tokens_equal '42',
          ['Literal.Number.Integer', '42']
      end

      it 'lexes a float with an exponent' do
        assert_no_errors('1.5e-3')
        assert_no_errors('2e10')
      end
    end

    describe 'strings' do
      it 'lexes the minimal escape set with no Error token' do
        assert_no_errors('"a\\nb\\tc\\\\d\\"e\\0f\\rg"')
      end

      it 'flags an unrecognized escape as Error (never-silent, G2)' do
        assert_has_token('Error', '"bad\\qescape"')
      end
    end

    describe 'decorators' do
      it 'lexes @std-sys as one atomic decorator token' do
        assert_tokens_equal '@std-sys',
          ['Name.Decorator', '@std-sys']
      end

      it 'lexes @tier(...) and @forage(...) as decorators' do
        assert_has_token('Name.Decorator', '@tier(compiled)')
        assert_has_token('Name.Decorator', '@forage(policy: p)')
      end

      it 'lexes the bare @ guarantee-annotation glyph as Operator, not a decorator' do
        assert_tokens_equal 'Ternary @ Exact',
          ['Keyword.Type', 'Ternary'],
          ['Text.Whitespace', ' '],
          ['Operator', '@'],
          ['Text.Whitespace', ' '],
          ['Keyword.Constant', 'Exact']
      end
    end

    describe 'operators' do
      it 'lexes the shift operators as two-char tokens' do
        assert_tokens_equal 'a << b >> c',
          ['Name', 'a'],
          ['Text.Whitespace', ' '],
          ['Operator', '<<'],
          ['Text.Whitespace', ' '],
          ['Name', 'b'],
          ['Text.Whitespace', ' '],
          ['Operator', '>>'],
          ['Text.Whitespace', ' '],
          ['Name', 'c']
      end

      it 'flags the retired -> return arrow as Error (RFC-0037 D4), never a silent accept' do
        assert_has_token('Error', 'fn f(x) -> x')
      end

      it 'lexes the => return arrow as an Operator, not an Error' do
        deny_has_token('Error', 'fn f(x) => x')
      end
    end

    describe 'declaration names' do
      it 'tags an fn declaration name as Name.Function' do
        assert_tokens_includes 'fn my_func(x) => x',
          ['Name.Function', 'my_func']
      end

      it 'tags a type declaration name as Name.Class' do
        assert_tokens_includes 'type MyType = Foo | Bar',
          ['Name.Class', 'MyType']
      end
    end

    describe 'word buckets (source: tools/grammar/keywords.json)' do
      it 'lexes the four keyword-json classes distinctly' do
        assert_tokens_equal 'let',    ['Keyword', 'let']
        assert_tokens_equal 'Binary', ['Keyword.Type', 'Binary']
        assert_tokens_equal 'F64',    ['Name.Builtin', 'F64']
        assert_tokens_equal 'Exact',  ['Keyword.Constant', 'Exact']
      end

      it 'tags a capitalized non-reserved identifier as Name.Constant' do
        assert_tokens_equal 'True', ['Name.Constant', 'True']
      end
    end

    describe 'a real corpus file' do
      # NOTE for the eventual rouge-ruby/rouge submission: this reads a file from the
      # `tzervas/mycelium` monorepo by relative path, which only resolves inside that repo's own
      # worktree (as run for this task's TESTED check — see this directory's README). Before
      # filing upstream, replace this with an inline string literal or a small fixture copied
      # into Rouge's own `spec/` tree — do not carry a cross-repo path into the submitted spec.
      it 'lexes lib/std/option.myc with no Error tokens' do
        src = File.read(File.expand_path('../../../lib/std/option.myc', __dir__))
        deny_has_token('Error', src)
      end
    end
  end
end
