# Mycelium — Session Kickoff Prompt

Read `.claude/agent-context.md` for the compact orientation brief (CLAUDE.md wins on any conflict).

## Mission: drive **every** open issue & work item to done — Gap-Closure + Dogfooding

The **1.0.0 kernel/core gate is closed** (every ADR-021 row green; M-654 landed). This wave resolves
the **entire remaining backlog to done** — **except** two items the maintainer reserves:
**M-655** (cut the 1.0.0 tag / ADR-021 `Accepted → Enacted`) and **M-381 / M-646** (LLM local runs).
Once E7-1 lands, **also complete M-649** (self-host the first stdlib nodule). Roadmap: **`DN-11 §5`**
(Phase-6 plan) + **`DN-14 §3`** (self-hosting gate) + the **E7-1 / E7-2** epics in `issues.yaml`.

### Already done (do not redo)
1.0.0 gate **all green** (A1·A2·A3·A4·A5·B1·B2); stdlib 25/25 enacted (RFC-0016/0017/0021 Enacted);
DN-04/05/10/11/12 Resolved; M-502 done; M-651 done. The maintainer tags 1.0.0 on their own.

### Track 1 — E7-1: L1 Stage-1 Language Completeness (Phase 5, `crates/mycelium-l1`)
| ID | Title | depends_on |
|----|-------|-----------|
| **M-656** | spec(rfc-0007): ratify stage-1 generic type parameters | — |
| **M-657** | feat(l1): generics in `checkty.rs` + `elab.rs` | M-656 |
| **M-658** | spec(rfc-0007): stage-1 trait interfaces + `impl` | M-657 |
| **M-659** | feat(l1): trait checking + `impl` blocks | M-658 |
| **M-660** | spec+feat(l1): effect annotations on `fn` (RFC-0014 stage-1) | M-659 |
| **M-661** | feat(l1): `wild` block typecheck — auditable FFI surface | M-660 |
| **M-662** | feat(l1): `phylum` construct + cross-nodule import resolution | M-657, M-659 |
| **M-663** | feat(l1): enact RFC-0018 stage-1 static guarantee grading | M-657 |
| **M-664** | feat(l1): `consume`/`grow`/`impl` surface keywords | M-659 |

### Track 2 — E7-2: RFC-0008 Runtime Vocabulary (Phase 7, `crates/mycelium-l1` + runtime)
| ID | Title | depends_on |
|----|-------|-----------|
| **M-665** | feat(l1-lexer): reserve the 10 DN-03 runtime terms never-silent *(in flight)* | — |
| **M-666** | feat(l1+runtime): R1 `hypha` + `colony` surface constructs | M-665 |
| **M-667** | feat(l1+runtime): R1 `fuse` / `reclaim` / `tier` | M-666 |
| **M-668** | design: R2 planning — `xloc`/`mesh`/`cyst`/`graft`/`forage`/`backbone` | M-667 |

### Track 3 — Dogfooding (build Mycelium **phyla**, never "crates"; IDs minted from the RFCs)
| Item | Title | Gate |
|------|-------|------|
| **`mycelium-web`** *(planned RFC-0022 — in research, not yet minted)* | Web-tooling phylum (HTTP client/server/routing/JSON) | follow-up research → build |
| **`mycelium-adk`** *(planned RFC-0023 — in research, not yet minted)* | Google ADK port (Agent/Tool/Session/Runner/multi-agent) | follow-up research → build |
| **doc-site** | build the in-repo doc site + run the lang-ref autogen (`just docs-site` / `scripts/docsite.sh`) | unblocked |
| **LSP completions** | baseline scaffolding completions in `crates/mycelium-lsp` (grounded in the L1 grammar) | unblocked |

### Track 4 — Self-hosting (after E7-1)
| ID | Title | depends_on |
|----|-------|-----------|
| **M-649** | self-host the first stdlib nodule in Mycelium-lang | E7-1 (M-502 ✅) |

## Two-phase research discipline (every research task)
Each research task runs **two phases**, and the **follow-up gates ratification/build**:
1. **Initial** — produce the design as a **Draft** RFC + a `research/NN-…-RECORD.md` (T-labelled
   findings + a **Honest-Uncertainty Register §** per the Record-09/11 pattern: what's design-decidable
   vs irreducibly empirical) + mint a follow-up **`RP-n`** in `docs/notes/research-prompts.md` (question,
   falsification threshold, "Feeds:", status Open).
2. **Follow-up** — a deep, multi-source verification pass (use **`/deep-research`**) that discharges the
   open gates and resolves the `RP-n`. **The RFC stays `Draft` and its code is NOT built/landed until the
   follow-up discharges its gates** (RFC-0021's carve-out is the precedent). Design may proceed in
   parallel; **ratification and landing wait on the follow-up.** Append-only; never pre-write a verdict.

## Sequencing — **max parallel** (every unblocked track at once)
Run Tracks 1–4 **concurrently**. The only serialization is the **shared collision surface**: E7-1 and
E7-2 both edit `crates/mycelium-l1` (`token.rs`/`parse.rs`/`checkty.rs`/`elab.rs`). The **orchestrator
owns those files** and lands the L1 changes **in dependency order** — never two leaves editing the same
L1 file in parallel (mitigation #7: merge the ref each child *reports*, then *count* landed files).
Dogfooding research, doc-site, and LSP completions are disjoint and fully parallel now; the web/ADK
**builds** start once their follow-up research discharges.

## Swarm model — Opus + Sonnet (as needed)
Opus **orchestrator** + **Opus** epic agents + **Sonnet** leaves. Pass the resolved model **explicitly**
to every spawn (never substitute silently). Fractal merge flow (leaf → epic → orch → squash PR → main);
**push the parent tip before spawning worktree children**; **pull squashed `main` down before each
merge-up**. Use the deep-research skill for research follow-ups.

## Prompting policy — autonomous; ask only on real ambiguity
**Proceed automatically until all requirements are satisfied.** A leaf **FLAGs up**, never guesses
(G2/VR-5). The orchestrator decides from sensible defaults and uses **`AskUserQuestion`** **only** for:
(1) genuinely ambiguous requirements, (2) architecturally-significant choices, (3) honesty/guarantee-tag
tradeoffs, or (4) scope changes. Otherwise, drive on.

## Key invariants for this wave
- **Honesty (VR-5)**: every bound/guarantee at its honestly-supportable strength; `Proven` only with a
  *checked* basis. **LLM/agent outputs stay `Declared`/`Empirical`, never `Proven`** (the ADK port's
  differentiator). A spec moves to "implemented (Rust-first), pending ratification", never silently `Accepted`.
- **Never-silent (G2)**: every fallible path returns `Option`/`Result`/explicit error; reserved-not-active
  keywords lex as keywords (never silent identifiers).
- **Append-only**: status flips add a resolution record; supersede, don't rewrite.
- **Grounded**: every claim cites the corpus (`G/A/R/T` labels, RFC/ADR/DN §). Research follow-ups cite sources.
- **Tests**: a property test for every bound; a regression/witness test for every behavior; `just check` green.
- **IDs**: pre-check every new `M-xxx`/`E-xxx` slot (`grep "id: M-669" issues.yaml`); after any change,
  `python3 tools/github/gh-issues-sync.py --validate` warning-clean.

## Branch & PR flow

```
Branch from main → develop → just check green → pull squashed main down → curated squash PR → main
```

`main` is **never touched directly** — the only write is the PR's squash-merge. Handle every Copilot/CI
review comment **first**, then land. Keep `issues.yaml` status + `CHANGELOG` append-only and current.
Use `/land` for the final squash.

## Stop conditions (the maintainer's, not the swarm's)
- **M-655** — cut the 1.0.0 tag + move ADR-021 `Accepted → Enacted` (prepped, but the maintainer's step).
- **M-381 / M-646** — the LLM local runs (scripts are polished: `cd tools/llm-harness && ./run.sh --all`).
