# Kickoff `tool10` — E16-1 Toolchain, IDE & package distribution

> Stowed kickoff, UID **`tool10`**. Read `CLAUDE.md` + `.claude/kickoffs/README.md`
> (the tiered `dev → integration → main` workflow) first. This file adds the specifics.

## Head branch (your locked base)

**`claude/head/tool10`** — persistent, protected base; all work merges into it; `main`
is PR-only; the head → `main` PR is the final squash step.

## Status

| Field | Value |
|---|---|
| **UID** | `tool10` |
| **Head branch** | `claude/head/tool10` |
| **Status** | ready |
| **Swarm mode** | Sonnet |
| **Depends on** | E9-1 (editor highlighting & grammar — M-693/M-694/M-695/M-696/M-697 must land before M-731 finalizes) |

## Scope

Drive **E16-1** (Toolchain, IDE & package distribution) to done — the full-language 1.0.0
developer experience: LSP completeness, editor syntax highlighting delivery, package
manager, toolchain UX, and reproducible distribution.

| # | Issue | What | Dir / crate |
|---|---|---|---|
| 1 | **M-730** | Full LSP: completions / hover / semantic-tokens / go-to-def beyond the skeleton | `crates/mycelium-lsp/**` |
| 2 | **M-731** | Editor syntax highlighting delivery (RFC-0026): ship TextMate + tree-sitter + LSP semantic tokens | `tools/grammar/**` (new) · `crates/mycelium-lsp/src/` |
| 3 | **M-732** | Package manager: spore publish / resolve / registry | `crates/mycelium-spore/**` |
| 4 | **M-733** | Toolchain UX: one-command init/build/test/run + error-message quality bar | `crates/mycelium-cli-common/**` · `justfile` |
| 5 | **M-734** | Reproducible toolchain distribution: pinned, content-addressed install | `scripts/dist/**` (new) · `Cargo.lock` |

## Epic & issue IDs

E16-1 (epic), M-730, M-731, M-732, M-733, M-734.

## Grounding

- `crates/mycelium-lsp/src/`: has completions, diagnostics, fmt — **no** hover, go-to-def,
  or semantic-token provider today (`crates/mycelium-lsp/src/lib.rs`).
- `crates/mycelium-spore/src/lib.rs`: the build artifact; **no** publish/resolve/registry today.
- RFC-0026 (Draft): the binding grammar artifact + scope-name decision (M-693); M-731 depends on it.
- DN-24 (Draft): the editor highlighting design space note; RFC-0026 is the binding decision.
- DN-25 (forthcoming — from another roadmap agent): the 1.0.0 full-language north-star definition.
- ADR-021 (`Accepted`): the kernel 1.0.0 gate; this wave is the full-language layer on top.
- All guarantee claims: `Declared` until checked basis exists (VR-5).

## Swarm / parallelization pattern

**Parallel-leaf Sonnet swarm with serialization on shared LSP files:**

- M-730 + M-731 both touch `crates/mycelium-lsp/src/` → **serialize these two**
  (M-730 lands first; M-731 extends semantic tokens without collision).
- M-732, M-733, M-734 own disjoint crates/dirs → **fan out in parallel**.
- Each leaf follows `/dev-workflow`: small auditable steps, honest per-op guarantee tags,
  property test per bound, never-silent `Result`/`Option`, `EXPLAIN`-able selections.
  Run `cargo fmt` / `clippy -D warnings` / `cargo test -p <crate>` green per leaf.

**Collision surface owned by this kickoff head (never agent-edited):**

- workspace `Cargo.toml` (new crate entries)
- `CHANGELOG.md`
- `docs/Doc-Index.md`
- `tools/github/issues.yaml` + `idmap.tsv`
- `docs/api-index/`

## Sequencing & dependencies

```
E9-1 (M-693 RFC-0026 decision → scope names fixed)
  └─ M-731 (grammar delivery: TextMate + tree-sitter + LSP semantic tokens)
       └─ depends on M-730 (LSP completeness: adds semantic-token provider base)

M-732, M-733, M-734 — independent; fan out after E16-1 head branch is pushed
```

Do **not** finalize M-731's scope-name table until RFC-0026 is Accepted (M-693 Done).
If M-693 is not yet done, M-731 may scaffold the generator and drift-check but must
leave scope names as `TODO` stubs — never guess (G2 / VR-5).

## Definition of Done

- [ ] M-730: `mycelium-lsp` provides hover, go-to-def, and semantic tokens; all existing
  completions/diagnostics/fmt tests still green.
- [ ] M-731: TextMate grammar + tree-sitter grammar + LSP semantic-token provider ship;
  the generator derives from `token.rs` `keyword()`; `just drift-check` is green and
  wired into CI; no silent divergence from the real lexer (G2).
- [ ] M-732: `mycelium-spore` can publish + resolve + query a registry endpoint; the
  content-addressed identity (ADR-003) is preserved; fallibility is never-silent.
- [ ] M-733: `myc init`, `myc build`, `myc test`, `myc run` work end-to-end; error
  messages meet the DN-22 diagnostic quality bar (structured, actionable, not opaque).
- [ ] M-734: a reproducible install script pins the toolchain to a content-addressed
  artifact; re-installs produce a byte-identical result.
- [ ] All guarantee tags are at their honest strength (never upgraded without a checked
  basis — VR-5); property test per bound; `just check` green.
- [ ] Every issue body + status updated; E16-1 epic status updated.
- [ ] Head → `main` PR passes `/pr-review` + Copilot round; squash commit is curated.

## Landing

`/wave-land` or `/land` once the head branch carries all five tasks green and
`just check` passes. The squash commit subject must reflect the net change
(`feat(toolchain): E16-1 — full LSP, editor grammars, spore publish, toolchain UX, reproducible dist`).
Orchestrator reconciles `CHANGELOG.md`, `Doc-Index.md`, `issues.yaml`, `api-index/` once
before the final squash-PR to `main`.
