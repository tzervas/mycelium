# Rouge lexer draft — Mycelium (M-697)

GitLab highlights code with [Rouge](https://github.com/rouge-ruby/rouge) (Ruby), a separate
stack from GitHub's Linguist/TextMate pipeline (see `tools/grammar/DISTRIBUTION.md` §e/§f for the
Linguist side). This directory is a **draft, unsubmitted** Rouge lexer for `.myc` — no PR has been
opened against `rouge-ruby/rouge`; that decision is the orchestrator's/maintainer's (see
`tools/grammar/DISTRIBUTION.md` §f for the submission steps, once approved).

## Files

| File | Purpose |
|---|---|
| `mycelium.rb` | The `Rouge::RegexLexer` subclass — target path `lib/rouge/lexers/mycelium.rb` in a Rouge checkout. |
| `mycelium_spec.rb` | A spec in Rouge's own format (`describe`/`it` + `Support::Lexing`/`Support::Guessing`) — target path `spec/lexers/mycelium_spec.rb`. |
| `README.md` | This file. |

Grounding for both files: `crates/mycelium-l1/src/token.rs` + `crates/mycelium-l1/src/lexer.rs`
(the canonical lexer — literal forms, the escape set, the operator-glyph table) and
`tools/grammar/keywords.json` (the four word-class buckets — copy that file's `classes` map by
hand here after any lexer keyword change; this draft is **not** wired into `just drift-check`, so
it can silently drift if not kept in sync manually — flagged in `mycelium.rb`'s header comment).

## Status: TESTED (not just Declared)

Unlike a purely-Declared draft, this lexer was actually exercised against the real `rouge` Ruby
gem in this task's environment. Exactly what ran:

1. **Environment**: no Ruby was preinstalled; `apt-get install -y ruby-full` (Ubuntu, Ruby 3.2.3)
   succeeded within the time-box. `gem install rouge --user-install` installed the real
   `rouge` 5.0.0 gem (not a mock/stub).
2. **Load + smoke script** (`ruby` one-off, not committed): required `rouge`, then this
   directory's `mycelium.rb`, then lexed:
   - Every stdlib nodule in `lib/std/*.myc` (17 files), every accept-corpus fixture in
     `docs/spec/grammar/conformance/accept/*.myc` (25 files), and every file under `examples/**`
     (6 files) — **48 real, valid `.myc` files, 0 `Error` tokens** across all of them.
   - Guard cases for every literal form (`0b…`, `0t…`, `0x…`, float/int), the full escape set
     (`\n \t \\ \" \0 \r`), all three decorator forms (`@std-sys`, `@tier(...)`, `@forage(...)`),
     the bare `@` guarantee glyph, the shift operators, `=>`, declaration-name tagging — all
     **0 `Error` tokens**.
   - Two intentional-failure guards: the retired `->` return arrow and an unrecognized `\q`
     escape — both correctly produced an **`Error`** token (never a silent accept, G2).
3. **The actual spec file, run for real**: `mycelium_spec.rb` (this directory) was run against
   Rouge's own test harness — not a from-scratch reimplementation. `spec/spec_helper.rb`,
   `spec/support/lexing.rb`, and `spec/support/guessing.rb` were fetched directly from
   `rouge-ruby/rouge`'s `main` branch (the exact files a real checkout's `rake` run would use),
   plus `minitest`/`minitest-power_assert`/`power_assert` (the gems Rouge's own `Gemfile`
   declares for its spec DSL). Result:

   ```
   21 runs, 56 assertions, 0 failures, 0 errors, 0 skips
   ```

   (Two spec-authoring mistakes were caught and fixed in this process — the whitespace token's
   real `qualname` is `Text.Whitespace`, not `Text` — itself a small piece of evidence the run was
   real and not rubber-stamped.)
4. **`rougify` CLI** (the gem's own bundled binary, `-r <file>` to load an out-of-tree lexer):
   - `rougify -r mycelium.rb list` → lists `mycelium: The Mycelium multi-paradigm value-semantics
     language (github.com/tzervas/mycelium)`.
   - `rougify -r mycelium.rb guess lib/std/option.myc` → correctly resolves to the `mycelium` tag
     by filename.
   - `rougify -r mycelium.rb highlight -l mycelium <file>` → produced real ANSI-colorized output
     (see the demo below, rendered via the `html` formatter instead, since ANSI escapes don't
     survive in Markdown) for real corpus files.

**What is still UNTESTED / Declared:**
- **Rouge's own `rake`/`TEST=…` invocation was not run** — this task's harness reconstructs the
  relevant pieces of `spec/spec_helper.rb` standalone (fetched from upstream `main`, not run
  inside a full Rouge git checkout via Bundler), because a full checkout + `bundle install` was
  out of scope for this draft. The reconstruction loads the *same* support modules and gems Rouge
  itself uses, so this is strong evidence, but it is not identical to "cloned `rouge-ruby/rouge`,
  ran `TEST=spec/lexers/mycelium_spec.rb rake`" — that exact invocation is Declared until someone
  runs it inside a real checkout (the mechanics are in `tools/grammar/DISTRIBUTION.md` §f).
- **`bundle exec rougify highlight`** (bundler-wrapped) was not exercised — only the gem-direct
  `rougify` (installed via `--user-install`, no Gemfile/Bundler in this task's environment). The
  command shape is identical either way; only the invocation wrapper differs.
- The `mycelium_spec.rb` corpus test reads a file by a monorepo-relative path (see the `NOTE` in
  that file) — that specific test does **not** port as-is to a `rouge-ruby/rouge` PR; it needs an
  inline string or a copied-in fixture first.
- Coverage is lexical only — no claim is made about the full structural surface (the tree-sitter
  grammar itself is still a keyword-only scaffold per `tools/grammar/README.md`), only that this
  regex lexer's token classification matches the landed lexer contract on the corpus tested.

## Demo sample

Source (`docs/spec/grammar/conformance/accept/05-guarantee-annotation.myc`, a real, small
conformance fixture — guarantee annotation + `swap`):

```mycelium
// exercises: guarantee-annotated return type (the LR-6 honesty index)
nodule numerics;

fn certified(x: Dense{768, F32}) => Dense{768, BF16} @ Proven =
    swap(x, to: Dense{768, BF16}, policy: bf16_round);
```

`rougify -r mycelium.rb highlight -f html -l mycelium <file>` output (Pygments/Rouge-standard CSS
classes — `c1` comment, `k` keyword, `n` name, `p` punctuation, `nf` Name.Function, `kt`
Keyword.Type, `mi` Literal.Number.Integer, `nb` Name.Builtin, `o` operator, `kc`
Keyword.Constant), confirming the token split matches the RFC-0026 §3.2 bucket intent:

```html
<span class="c1">// exercises: guarantee-annotated return type (the LR-6 honesty index)</span><span class="w">
</span><span class="k">nodule</span><span class="w"> </span><span class="n">numerics</span><span class="p">;</span><span class="w">

</span><span class="k">fn</span><span class="w"> </span><span class="nf">certified</span><span class="p">(</span><span class="n">x</span><span class="p">:</span><span class="w"> </span><span class="kt">Dense</span><span class="p">{</span><span class="mi">768</span><span class="p">,</span><span class="w"> </span><span class="nb">F32</span><span class="p">})</span><span class="w"> </span><span class="o">=&gt;</span><span class="w"> </span><span class="kt">Dense</span><span class="p">{</span><span class="mi">768</span><span class="p">,</span><span class="w"> </span><span class="nb">BF16</span><span class="p">}</span><span class="w"> </span><span class="o">@</span><span class="w"> </span><span class="kc">Proven</span><span class="w"> </span><span class="o">=</span><span class="w">
    </span><span class="k">swap</span><span class="p">(</span><span class="n">x</span><span class="p">,</span><span class="w"> </span><span class="k">to</span><span class="p">:</span><span class="w"> </span><span class="kt">Dense</span><span class="p">{</span><span class="mi">768</span><span class="p">,</span><span class="w"> </span><span class="nb">BF16</span><span class="p">},</span><span class="w"> </span><span class="k">policy</span><span class="p">:</span><span class="w"> </span><span class="n">bf16_round</span><span class="p">);</span><span class="w">
</span>
```

## Submission (not performed by this task; see `tools/grammar/DISTRIBUTION.md` §f)

1. `gh repo fork rouge-ruby/rouge --clone`
2. Copy `mycelium.rb` → `lib/rouge/lexers/mycelium.rb` and `mycelium_spec.rb` →
   `spec/lexers/mycelium_spec.rb` (adjusting the corpus-file test per the NOTE in that file).
3. Add a demo sample per Rouge's visual-test convention (see its README's "Testing Rouge"
   section) — e.g. the fixture shown above.
4. `TEST=spec/lexers/mycelium_spec.rb rake` (Rouge's own documented single-lexer test command).
5. `bundle exec rougify highlight -l mycelium <file>` for a final visual check.
6. `gh pr create -R rouge-ruby/rouge …`

Rouge's own contribution posture (its README's "Developing Lexers" section) is markedly lower
friction than GitHub Linguist's adoption-bar (`tools/grammar/DISTRIBUTION.md` §e.1) — it
explicitly invites work-in-progress submissions ("submit a pull request with what you have and
make it clear... the lexer isn't finished yet"). This draft is deliberately more complete than
that minimum bar.
