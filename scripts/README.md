# scripts/ — local, no-AI check tooling

The deterministic quality gate for Mycelium. **One implementation, three entrypoints** — so
what you run locally is exactly what CI runs:

```
just check            ──┐
pre-commit (just hooks) ─┼──►  scripts/checks/*.sh  ──►  same result everywhere
.github/workflows (CI) ──┘
```

## Use it

```sh
just setup     # install the check tools (uv tool / npx / pip); safe to re-run
just check     # run the full suite
just fmt       # auto-format (rust + python)
just hooks     # install pre-commit so checks run on every commit
just <name>    # run one check (e.g. md, links, doc-currency, doc-status, schema, spell, lint); `just --list` shows them all
```

No `just`? The scripts are plain bash: `bash scripts/checks/all.sh`.

## Cloud sessions (Claude Code on the web)

`scripts/install-tools.sh` is also the canonical toolchain installer for cloud sessions. Wire it as
the environment **Setup script** (Cloud environment settings → *Setup script* field):

```sh
bash scripts/install-tools.sh
```

The setup script runs the **first** time a session starts in an environment; Anthropic then
**snapshots the filesystem** and reuses that snapshot for later sessions — the compiled tools
(`cargo` binaries in `~/.cargo/bin`, `uv` tools in `~/.local/bin`) persist, and the setup step is
**skipped** on subsequent sessions. The toolchain is compiled **once**, not per session. The
snapshot rebuilds only when the setup script or network allowlist changes, or after ~7 days.

- **Don't** put this in a `SessionStart` hook — those run on every session and are **not** cached,
  so they would recompile the toolchain each time.
- Setup scripts have a **~5-minute** cache-build budget. The `cargo` introspection tools for
  `just map` / `just api` (cargo-modules/depgraph/public-api) are the slow tail and are **not** part
  of `just check`; export `MYCELIUM_SKIP_OPTIONAL_CARGO=1` to skip them and stay under budget. The
  security gates (cargo-deny/cargo-audit) and the rest still install.
- Cloud network access must allow crates.io / PyPI / npm (the default **Trusted** policy does).

Docs: [code.claude.com/docs/en/claude-code-on-the-web](https://code.claude.com/docs/en/claude-code-on-the-web)
§ *Setup scripts* / *Environment caching*.

## What runs
| Check | Tool | Notes |
|---|---|---|
| `structured` | python | every tracked `.json/.yaml/.toml` parses |
| `shell` | shellcheck | `*.sh` (via `shellcheck-py` if no system binary) |
| `markdown` | markdownlint-cli2 | config `.markdownlint.jsonc`; run via `npx` |
| `links` | `lint_links.py` | **offline** relative-link / cross-ref / `@import` checker |
| `doc-currency` | `doc_currency.py` | README structure tree · Doc-Index coverage · cited counts |
| `doc-status` | `doc_status_check.py` | decision-status **lattice** · nav-README↔header cross-check · Declared stale-phrase invariants |
| `schema` | check-jsonschema | draft 2020-12 metaschema + example instances (per M-010) |
| `spell` | codespell | config `.codespellrc` |
| `secrets` | gitleaks | respects `.gitleaks.toml`; narrow fallback if gitleaks absent |
| `format` | cargo fmt / ruff | check-only; `--fix` to write |
| `lint` | clippy / ruff | `clippy -D warnings` per CONTRIBUTING |

## Design rules
- **Graceful skip:** a check whose tool or language isn't present prints `skip` and exits 0 —
  it never fails the suite. (Most code doesn't exist yet; checks light up as it lands.)
- **Tracked files only:** checks operate on `git ls-files` output (no `node_modules`, `target`).
- **Parity:** add new logic to `scripts/checks/*`, then expose it as a `just` recipe and a
  pre-commit hook — never reimplement a check in the workflow or the justfile.
- **Exit convention:** `0` = pass or skip; non-zero = real failure.

## The `doc-status` gate (status currency) — a Declared heuristic

`doc_status_check.py` keeps the corpus's decision statuses honest and the index READMEs
in agreement with the authoritative per-doc `Status` headers. Three never-silent passes:

1. **Lattice** — every decision doc (`docs/rfcs/RFC-*`, `docs/adr/ADR-*`,
   `docs/notes/DN-*`, `docs/spec/stdlib/*`) carries a leading status token on the ratified
   lattice `Draft/Proposed/Preliminary → Accepted → Enacted → Superseded` (notes →
   `Resolved`, #236). A bare legacy compound `Accepted — Enacted` fails as
   normalization-needed (canonical = standalone `Enacted`).
2. **Cross-check** — each `docs/rfcs/README.md` / `docs/adr/README.md` index row's claimed
   status must match the doc's authoritative header (the drift that left 8 stale RFC rows).
3. **Declared stale-phrase invariants** — the maintainer-authored rules in
   `tools/doc-status-invariants.yaml` (e.g. "once every stdlib spec except
   runtime/self-hosting-readiness is Accepted-or-later, no nav README may still say
   'pending ratification'").

**Honesty posture (house-rule 1 / VR-5):** this is a **Declared** line/regex heuristic —
*source is ground truth*. It reads the leading token of a `Status` row and a status cell,
not prose; the pass-3 rules are **Declared** maintainer decisions in the manifest, never
inferred by the script. Adding/relaxing an invariant is itself a decision — edit the
manifest deliberately (and note it in the changelog), don't let the gate guess one.

## Local docsite

`scripts/docsite.sh` (run via `just docs-site`) assembles a single browsable static site under
`target/docsite/` from four sources:

| Section | Tool | What it builds |
|---|---|---|
| **Corpus** | `myc-doc build` | HTML view of RFCs/ADRs/DNs/specs (M-363 doc-IR) |
| **Language reference** | `scripts/docsite.sh` (autogen) | Grammar + lexicon + stdlib spec index (`lang-ref/index.html`) |
| **Agent index** | `tools/docgen/code_index.py` | Symbol table with crate/file:line/guarantee-tag |
| **Rustdoc** | `cargo doc --no-deps --workspace` | Public Rust API reference |

Each section **skips gracefully** when its tool is absent (warns, continues). The output is
gitignored (`target/docsite/`); it is never committed.

**Usage:**

```sh
just docs-site              # build the site (all four sections)
```

**Browsing on WSL:** serve the output dir with Python's built-in HTTP server, then open the URL in
your Windows browser:

```sh
cd target/docsite
python3 -m http.server 8080
# open http://localhost:8080 in your Windows browser
```

The landing page (`target/docsite/index.html`) links all four sections and lists any that were
skipped. Most sections work when opened as a local file; rustdoc requires a server (absolute-path
links).

### Language-reference autogeneration

The **Language reference** (`lang-ref/index.html`) is generated deterministically from the corpus
in a single pass — no separate tool required. It is emitted only when the corpus builds (it links
into `corpus/pages/`). The page covers:

- **Surface grammar** — links to the grammar README, RFC-0006, RFC-0007, RFC-0020, and a
  **curated excerpt** of key EBNF productions (a hand-maintained snapshot, not auto-extracted;
  `docs/spec/grammar/mycelium.ebnf` is the normative source).
- **Reserved-word lexicon** — three tables (active keywords, reserved-not-active words, and
  ratified-not-yet-lexed names) as a **hand-curated orientation snapshot**, not auto-extracted from
  the lexer. The normative ground truth is `crates/mycelium-l1/src/token.rs` +
  `docs/spec/grammar/mycelium.ebnf` (`.claude/memory/lang-lexicon-syntax.md` is a non-normative
  maintenance note). Re-verify against `token.rs` after any lexer change (Empirical/Declared — G2/VR-5).
- **Standard-library module specs** — per-module links for all 25 modules (Ring 0/1/2), with
  ring, task, and grounding crux; traces to RFC-0016 §4.5 guarantee matrices.
- **Language-layer design documents** — links to RFC-0001, RFC-0012, RFC-0017, RFC-0019,
  the Nodule-Header spec, and the Example Programs Reference.

To regenerate after a corpus change, just re-run `just docs-site` — the lang-ref page is rebuilt
from scratch on every run (idempotent, deterministic, offline).

## Remote CI
`.github/workflows/checks.yml` is **manual-dispatch only** (`workflow_dispatch`) and
**advisory** (non-blocking) — it runs `just ci`, i.e. this same suite. See the repo CI policy
in `CLAUDE.md`.
