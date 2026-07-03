# Grammar distribution runbook (M-697)

The ready-to-fire playbook for getting `.myc` syntax highlighting into editors and, eventually,
GitHub's own language bar. Companion to `tools/grammar/README.md` (source-of-truth + drift gate)
and RFC-0026 (scope-name ratification). **Nothing in this document ships silently** (G2): every
row states whether it is landed in this repo today or gated on a maintainer/external action, and
every external claim below is cited to the source checked.

## a. Status — done vs staged

| # | Channel | Status | Gate |
|---|---|---|---|
| 1 | VS Code / Cursor — local `.vsix` install | **DONE** (build recipe below; the extension itself ships from `editors/vscode/`, a sibling M-697 deliverable) | none — works today, no network/registry needed |
| 2 | Open VSX (open-vsx.org) | **STAGED** | maintainer signs the Eclipse Publisher Agreement once (§c step 2), then a token + two `npx ovsx` commands |
| 3 | Microsoft VS Code Marketplace | **STAGED, OPTIONAL** | Azure DevOps org + PAT + a Marketplace publisher — skip unless VS-Code-proper Marketplace search matters |
| 4 | GitHub Linguist (`.myc` in the language bar) | **STAGED, hard-gated** | repo must go **public** with real usage; a public grammar repo; Linguist's own adoption bar (§e — verbatim below, this is the long pole) |
| 5 | GitLab / Rouge | **STAGED** | `tools/grammar/rouge/` drafted in this PR (lexer + spec + README); submission PR to `rouge-ruby/rouge` is a separate, maintainer-gated step |
| 6 | Other editors (Neovim/Zed/Helix/Emacs) | **STAGED** | each consumes `tools/grammar/tree-sitter-mycelium/` directly once its structural grammar lands (RFC-0026 §3.4 follow-up); config snippets below work as soon as a `src/parser.c` exists |

**Honesty note on the Linguist row:** "Mycelium" appearing in GitHub's own language-bar UI comes
**only** from a merged Linguist PR — see `.gitattributes` (root) for why no `linguist-language=`
shortcut is used instead.

## b. VS Code / Cursor — local install (PRIMARY, Azure-free, works today)

No registry, no account, no network call beyond the initial `npm`/`npx` fetch of the packaging
tool. Works identically in VS Code and Cursor (Cursor consumes standard `.vsix` files).

Build the package:

```bash
cd editors/vscode && npx @vscode/vsce package
```

This produces `editors/vscode/mycelium-language-0.1.0.vsix` (name/version from
`editors/vscode/package.json`: `name: mycelium-language`, `version: 0.1.0`, `publisher: tzervas`).

Install it:

```bash
code --install-extension editors/vscode/mycelium-language-0.1.0.vsix
# Cursor uses the same flag:
cursor --install-extension editors/vscode/mycelium-language-0.1.0.vsix
```

Reload the editor window; `.myc` files should now highlight via `source.mycelium`.

## c. Open VSX — CHOSEN registry (Azure-free)

Open VSX (open-vsx.org, now Eclipse-Foundation-run) is the registry this project targets for
public distribution, because it needs no Azure DevOps account (unlike the MS Marketplace, §d).
**All of the following is deferred to the public-release phase** — do not run these against a
private/pre-release repo.

Steps, corrected against Open VSX's own publishing guide (checked 2026-07-02,
<https://github.com/eclipse/openvsx/wiki/Publishing-Extensions>) — note the guide's actual flow
signs in via an **Eclipse Foundation account** (not a bare GitHub login) before the Publisher
Agreement step:

1. **Create an eclipse.org account** with the same GitHub username you use to log in to
   open-vsx.org (register at <https://accounts.eclipse.org/user/register>, filling in the
   *GitHub Username* field).
2. **Sign the Eclipse Foundation Publisher Agreement** — a one-time manual gate. On
   open-vsx.org, go to your profile → *Log in with Eclipse* → *Show Publisher Agreement* → read
   → *Agree*.
3. **Generate an access token** at <https://open-vsx.org/user-settings/tokens>.
4. **Create the namespace** (the `publisher` field in `editors/vscode/package.json`, currently
   `tzervas`):

   ```bash
   npx ovsx create-namespace tzervas -p <token>
   ```

   If `tzervas` is ever unavailable, fall back to the namespace `mycelium-language` and update
   `editors/vscode/package.json`'s `publisher` field to match before publishing (the namespace
   **must** equal the `publisher` value — ovsx enforces this).
5. **Publish**:

   ```bash
   cd editors/vscode && npx ovsx publish -p <token>
   ```

   (`ovsx publish` packages via `vsce` itself if no `.vsix` is given, then uploads — no separate
   package step is required for this path, unlike §b/§d.)

Source for the exact command forms: `ovsx`'s own README (checked 2026-07-02,
<https://raw.githubusercontent.com/eclipse/openvsx/master/cli/README.md>) — `ovsx create-namespace
<name> -p <token>` and `ovsx publish -p <token>`.

## d. MS VS Code Marketplace — OPTIONAL / skippable (Azure)

**Skip unless you want VS-Code-proper Marketplace search discoverability** — this path requires
an Azure DevOps organization, which Open VSX (§c) exists specifically to avoid.

Checklist (Declared — not exercised in this task; standard `vsce`/Azure DevOps flow):

1. Create (or reuse) an Azure DevOps organization at <https://dev.azure.com>.
2. Create a Personal Access Token (PAT) scoped to **"All accessible organizations"** and
   **Marketplace → Manage**.
3. Create a Marketplace publisher (one-time) at
   <https://marketplace.visualstudio.com/manage>, matching `editors/vscode/package.json`'s
   `publisher: tzervas`.
4. Publish:

   ```bash
   cd editors/vscode && npx @vscode/vsce publish -p <PAT>
   ```

## e. GitHub Linguist — STAGED (fires at the public flip)

Linguist only recognizes languages/extensions with **real usage on public GitHub repos** — this
repo must be public first, and even then the adoption bar below applies. This is the long pole of
the whole runbook; read the caveat before filing anything.

### e.0 — Collision check (done, this PR)

Checked 2026-07-02 against
<https://raw.githubusercontent.com/github-linguist/linguist/main/lib/linguist/languages.yml>
(9452 lines fetched; `grep -i 'myc'` — zero matches). **Result: `.myc` is FREE** — no existing
Linguist entry claims it, under any language name. (The `github/linguist` org path
redirects to `github-linguist/linguist`, Linguist's post-2024 home; both resolve to the same
content.)

### e.1 — The adoption-bar caveat (verbatim, from Linguist's own CONTRIBUTING.md)

Quoting Linguist's contribution guide directly (checked 2026-07-02,
<https://raw.githubusercontent.com/github-linguist/linguist/main/CONTRIBUTING.md>), because this
is the honest constraint that governs whether this PR can be filed, not just how:

> We will only add new extensions once they have sufficient usage on GitHub... This means we do
> not accept PRs for very new or hobby languages, and will close any such PRs that attempt to add
> them.

and the numeric bar:

> at least 2000 files per extension... indexed in the last year, excluding forks, for extensions
> or filenames expected to occur more than once per repo... the results should show a reasonable
> distribution across unique `:user/:repo` combinations.

**Honest assessment (Declared):** at the time of writing this repo has 93 `.myc` files, in one
repository. That is nowhere near the 2000-files/multiple-repos bar Linguist states above. Filing
the PR before genuine third-party adoption exists is very likely to be closed per that stated
policy — this is not a paperwork gate, it's a real evidentiary one. The steps below are the
**ready-to-fire mechanics** for whenever that adoption threshold is plausibly met; they are not a
claim that filing today would succeed.

### e.2 — The `languages.yml` entry (ready to file, once e.1's bar is met)

Alphabetical position confirmed 2026-07-02: `Mycelium` sorts between `Mustache` (line 4971 of the
fetched file) and `Myghty` (line 4981) — insert immediately before `Myghty:`.

```yaml
Mycelium:
  type: programming
  color: "#7B68EE"
  extensions:
  - ".myc"
  tm_scope: source.mycelium
  ace_mode: text
  # language_id: assigned by `script/update-ids` at PR time — never hand-invent one (below).
```

- **`color`** — `#7B68EE` (MediumSlateBlue), picked because it is not already used by any entry in
  the fetched `languages.yml` (checked 2026-07-02, both `#7B68EE` and the alternative `#8A2BE2`
  came back with zero hits) and reads as "fungal/violet". **Declared and freely changeable** —
  Linguist's own docs say a color change needs community consensus, not unilateral pick (same
  CONTRIBUTING.md, "Changing the color associated with a language").
- **`ace_mode: text`** — Ace has no Mycelium mode; `text` is Linguist's documented fallback for a
  language without one (used by many entries with a real grammar but no matching Ace mode).
- **`tm_scope: source.mycelium`** — matches this repo's ratified RFC-0026 §3.2 scope name exactly;
  no translation needed.
- **`language_id`** — **do not invent one.** Linguist assigns it via `script/update-ids`
  (CONTRIBUTING.md step "Generate a unique ID for your language by running `script/update-ids`") —
  run at PR time against the live registry, never hand-picked.

### e.3 — The grammar submission (Linguist vendors a TextMate-format grammar via a public repo)

Linguist highlights via a **vendored TextMate grammar pulled from a public repo** you name with
`script/add-grammar <url>` — it does not read this monorepo's `tools/grammar/` directly. Two
options, evaluated:

| Option | What | Recommendation |
|---|---|---|
| A | Publish `tools/grammar/tree-sitter-mycelium` (or a small wrapper) as its own public repo, *and* have it also carry/re-export `mycelium.tmLanguage.json` | Extra repo-layout work now, but it becomes the **one** public grammar repo every downstream tool (Linguist, Zed, Helix's monorepo-`subpath` support aside) can point at |
| B | Mirror just the TextMate grammar to a dedicated `tzervas/mycelium-tmlanguage` repo | Simpler, smaller surface, but yet another repo to keep in drift-lockstep with the generator |

**Recommended: Option A** — one public grammar repo (structural + TextMate) beats two repos that
can silently drift from each other and from the generator (the same drift concern `just
drift-check` exists to prevent in-repo). Whichever is chosen, it must stay generated/drift-checked
the same way the in-repo copies are (`tools/grammar/generate.py`), not hand-maintained a second
time.

### e.4 — Sample files (`samples/Mycelium/`)

Linguist requires "real-world code showing common usage," explicitly rejecting hello-world
tutorials (CONTRIBUTING.md, "Adding a language" step 3). Recommend copying real, non-trivial
in-repo files, not writing new ones:

- `lib/std/option.myc` — a real stdlib nodule (generics, `match`, guarantee-tagged doc comments).
- `lib/std/result.myc` — sibling stdlib nodule, same shape.
- `docs/examples/https-downloader-layered.myc` — a larger, realistic multi-construct program
  (imports, swaps, effects) rather than another small stdlib file.

State clearly in the PR (per CONTRIBUTING.md) that these are project source under this repo's MIT
license, with a link back to the original path in `tzervas/mycelium`.

### e.5 — Exact command sequence (once e.1's bar is met; maintainer's authed `gh`)

```bash
gh repo fork github/linguist --clone
cd linguist
git checkout -b add-mycelium-language
script/bootstrap                      # install Linguist's own deps (Ruby/bundler)

# 1. languages.yml — insert the e.2 block before `Myghty:`, omitting language_id for now.

# 2. Vendor the grammar (Option A/B from e.3):
script/add-grammar https://github.com/tzervas/<chosen-grammar-repo>

# 3. Samples (e.4): copy the 2-3 files into samples/Mycelium/

# 4. Assign the real ID:
script/update-ids

# 5. Test, per Linguist's own CONTRIBUTING.md:
bundle exec rake test
bundle exec script/cross-validation --test

git add -A
git commit -m "Add Mycelium language"
git push -u origin add-mycelium-language
gh pr create -R github/linguist \
  --title "Add Mycelium language" \
  --body "… link to a GitHub search showing in-the-wild .myc usage per CONTRIBUTING.md …"
```

`gh repo fork` targets `github/linguist`, which today redirects to the project's actual home at
`github-linguist/linguist` (verified via the CONTRIBUTING.md fetch above resolving under that
org) — `gh` follows the redirect transparently.

## f. GitLab / Rouge

GitLab's highlighter is Rouge (Ruby), separate from GitHub's Linguist/TextMate stack. This PR
drafts the lexer at `tools/grammar/rouge/mycelium.rb` (see that directory's own README for the
tested-vs-untested status of the draft). Status: **staged** — submission to `rouge-ruby/rouge` is
a separate, maintainer-gated PR, not opened by this task.

Submission steps (per Rouge's own README.md "Contributing" section, checked 2026-07-02,
<https://raw.githubusercontent.com/rouge-ruby/rouge/main/README.md>):

```bash
gh repo fork rouge-ruby/rouge --clone
cd rouge
git checkout -b add-mycelium-lexer
# copy tools/grammar/rouge/mycelium.rb      -> lib/rouge/lexers/mycelium.rb
# copy tools/grammar/rouge/mycelium_spec.rb -> spec/lexers/mycelium_spec.rb
# copy a demo sample (see tools/grammar/rouge/README.md)
#   -> spec/visual/samples/mycelium.myc (Rouge's own visual-sample convention)

# Rouge's own documented single-file test invocation (rake-driven, no bundle exec needed):
TEST=spec/lexers/mycelium_spec.rb rake

# A direct rspec invocation is Declared-only here (unverified in this task, time-boxed):
# bundle exec rspec spec/lexers/mycelium_spec.rb

# Visual smoke:
bundle exec rougify highlight -l mycelium <file>.myc

git add -A && git commit -m "Add Mycelium lexer"
git push -u origin add-mycelium-lexer
gh pr create -R rouge-ruby/rouge --title "Add Mycelium lexer" --body "…"
```

Rouge does **not** state a Linguist-style adoption bar in its own CONTRIBUTING/README — its
posture is explicitly "we want to make it as easy as we can... to contribute a lexer," including
an explicit "submit unfinished, we'll help" allowance. This makes Rouge a materially lower-friction
target than Linguist (§e) for early submission, though a maintainer still decides timing.

## g. Other editors — consume `tools/grammar/tree-sitter-mycelium/` directly

Every snippet below points at this monorepo + subpath and needs no external registry. They become
fully functional once the structural grammar (`src/parser.c`, generated by the `tree-sitter`
CLI from `grammar.js`) exists — today's `grammar.js` is the RFC-0026 §3.4 keyword-only scaffold
(see `tools/grammar/README.md`), so these configs are correct but currently point at a partial
grammar.

### Neovim (`nvim-treesitter`)

```lua
local parser_config = require("nvim-treesitter.parsers").get_parser_configs()
parser_config.mycelium = {
  install_info = {
    url = "https://github.com/tzervas/mycelium",
    files = { "tools/grammar/tree-sitter-mycelium/src/parser.c" },
    location = "tools/grammar/tree-sitter-mycelium", -- monorepo subdir
  },
  filetype = "mycelium",
}
vim.filetype.add({ extension = { myc = "mycelium" } })
```

### Zed

Zed's extension model expects a grammar repo at its own root (no documented monorepo-subpath
field, unlike Helix below) — so Zed consumption is cleaner once §e.3's Option A public grammar
repo exists; point `extension.toml` at that repo, or at this monorepo as a stopgap if Zed's
`grammars` loader tolerates a non-root path (unverified — Declared):

```toml
[grammars.mycelium]
repository = "https://github.com/tzervas/mycelium"
rev = "<commit-sha>"

[language_servers.mycelium]
# LSP wiring (crates/mycelium-lsp) is a separate, already-tracked deliverable.
```

### Helix

Helix's grammar source explicitly supports a `subpath`, which fits this monorepo directly
(the same mechanism Helix itself uses for `tree-sitter-typescript`'s `typescript`/`tsx` subdirs):

```toml
[[language]]
name = "mycelium"
scope = "source.mycelium"
file-types = ["myc"]
comment-token = "//"

[[grammar]]
name = "mycelium"
source = { git = "https://github.com/tzervas/mycelium", rev = "<commit-sha>", subpath = "tools/grammar/tree-sitter-mycelium" }
```

### Emacs 29+ (`treesit`)

```elisp
(setq treesit-language-source-alist
  '((mycelium "https://github.com/tzervas/mycelium" nil "tools/grammar/tree-sitter-mycelium")))
(treesit-install-language-grammar 'mycelium)

(define-derived-mode mycelium-ts-mode prog-mode "Mycelium"
  "Major mode for Mycelium (.myc) using tree-sitter."
  (when (treesit-ready-p 'mycelium)
    (treesit-parser-create 'mycelium)
    (treesit-major-mode-setup)))
(add-to-list 'auto-mode-alist '("\\.myc\\'" . mycelium-ts-mode))
```

## Changelog

- 2026-07-02 — initial runbook (M-697): status table, VS Code/Cursor local install, Open VSX
  steps, MS Marketplace (optional) checklist, Linguist collision check + adoption-bar caveat +
  submission sequence, Rouge pointer, editor snippets (Neovim/Zed/Helix/Emacs).
