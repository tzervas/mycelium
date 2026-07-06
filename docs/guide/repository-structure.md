# Repository structure

One-line purpose: a directory-by-directory map of the repo, and where ADR-001…009 physically live.

```text
mycelium/
├── README.md                 ← you are here
├── LICENSE                   ← MIT
├── CONTRIBUTING.md           ← decision process, transparency rule, dev env, workflow
├── CLAUDE.md                 ← operating guide for Claude Code / agents (the house rules)
├── CHANGELOG.md              ← Keep-a-Changelog; design baseline + implementation edits
├── Cargo.toml                ← Rust workspace (50 crates + xtask; MSRV 1.96.1, ADR-007/ADR-041)
├── rust-toolchain.toml       ← pinned MSRV
├── justfile                  ← one source of truth for local↔CI checks (`just check`)
├── deny.toml                 ← cargo-deny supply-chain policy
├── crates/                   ← the Rust kernel + reference interpreter + stdlib (see the workspace map)
├── docs/
│   ├── Mycelium_Project_Foundation.md   ← charter, requirements, ADR-001…009, roadmap, risks
│   ├── Doc-Index.md                     ← map of the corpus + status + dependency DAG
│   ├── Glossary.md                      ← the fungal lexicon + transparency/architecture terms
│   ├── guide/       ← this directory — topic docs the root README links out to
│   ├── rfcs/        ← RFC-0001…0039 (normative designs) + index
│   ├── adr/         ← ADR-010…034 as files (ADR-001…009 live in the Foundation §8) + index
│   ├── notes/       ← DN-01 onward design notes + reference material (lexicon, examples, research
│   │                  prompts) — see `docs/Doc-Index.md` for the current highest number
│   ├── spec/        ← per-module + per-tool specs (stdlib/, api/ baselines, swaps/, grammar/)
│   ├── planning/    ← phase-by-phase build plans (phase-0 … phase-8)
│   ├── api-index/   ← the committed, generated agent code index (INDEX.md / index.json)
│   ├── wiki/        ← source for the published GitHub wiki
│   └── devlog/      ← append-only development log
├── research/                 ← the evidence base (records 01 … 27)
├── examples/                 ← worked `.myc` programs (hello-phylum, repr-tour)
├── lib/                      ← self-hosted Mycelium-lang stdlib (`.myc`; std.result — M-649/RFC-0024)
├── gen/                      ← generated Declared draft material (myc-drafts/ transpiler staging — E33-1; never imported by lib/)
├── experiments/               ← uv-managed Python experiments (the KC-2 LLM-leverage harness)
├── fuzz/                      ← cargo-fuzz durability targets (standalone nightly workspace; WS8/M-654)
├── proofs/                    ← Z3/SMT2 + Liquid-Haskell proof artifacts
├── scripts/                   ← the check tooling (scripts/checks/* behind `just check`)
├── editors/                   ← editor integrations (VS Code/Cursor extension — `mycelium-language`; M-924/lang-tooling)
├── tools/                     ← GitHub issue bootstrap, LLM harness, Termux setup, grammar artifacts
└── xtask/                     ← cargo-xtask repo-automation entrypoint
```

> **Note on ADRs.** ADR-001 through ADR-009 live inside `docs/Mycelium_Project_Foundation.md` §8
> (the decision log); ADR-010 onward are broken out as their own files in `docs/adr/`.
> All are append-only with status transitions. The authoritative, always-current map of the whole
> corpus (every RFC/ADR/DN with status) is [`docs/Doc-Index.md`](../Doc-Index.md).

## Documentation, generated and hand-written

- **Wiki** — the browsable project wiki is generated from
  [`docs/wiki/`](../wiki/) (Home · Architecture · Crate Index · Memory Model · Tunable
  Certification · Getting Started · API Reference) and published to the GitHub wiki by the
  manual-dispatch [`publish-docs`](../../.github/workflows/publish-docs.yml) workflow.
- **API docs** — rustdoc for the workspace: `just docs` (or `cargo doc --no-deps --workspace`);
  published to GitHub Pages by the same workflow. The committed grep-friendly agent index is
  [`docs/api-index/INDEX.md`](../api-index/INDEX.md).
- **Per-crate READMEs** — every crate has its own `README.md` (linked in the [workspace map](workspace-map.md)).
- **Design corpus** — `docs/` (`rfcs/`, `adr/`, `notes/`, `spec/`); status in
  [`docs/Doc-Index.md`](../Doc-Index.md).

---

**See also:** [Workspace map](workspace-map.md) · [Decisions & reading order](decisions-and-reading-order.md) ·
[Status & roadmap](status-and-roadmap.md)

[← Back to README](../../README.md)
