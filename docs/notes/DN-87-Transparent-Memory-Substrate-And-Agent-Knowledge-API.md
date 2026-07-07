# DN-87 — mycelium-tero: The Transparent Memory Substrate & Agent Knowledge API

| Field | Value |
|---|---|
| **Note** | DN-87 |
| **Status** | **Proposed** (2026-07-07) — maintainer vision captured; four design forks orchestrator-resolved (`Declared`, maintainer-overridable); the build wave fires when the maintainer names the project. |
| **Task** | epic **E39-1** + M-1015…M-1019 (minted with this note); kickoff **`mem`** (stowed) |
| **Related** | the current agent-memory system this *supplements* (CLAUDE.md · `.claude/agent-context.md` · `docs/CURRENT-STATE.md` · `docs/api-index/` · `docs/lib-index/` · the skills); DN-85/DN-86 (the multi-language program this may eventually serve cross-platform); `mycelium-vsa` (the semantic layer's substrate); RFC-0003/RFC-0009/RFC-0010 (VSA + resonator + decode); ADR-022 §7 / CONTRIBUTING §Licensing (MIT-only) |
| **Guarantee** | `Declared` throughout (a captured vision + resolved direction); every future retrieval-quality claim is gated `Empirical` per §6 before it is made |
| **Naming** | **NAMED `mycelium-tero`** (maintainer, 2026-07-07) — a quiet homage to **Atsushi Tero**, for his contribution to science and engineering. The name is sugar; the system is code (no biology beyond terminology). *(Prior: reserved/name-agnostic until this date.)* |

> **The maintainer's vision (2026-07-07, captured near-verbatim).** Automate converting the
> project's information — the development methodologies, the intents, the language being developed,
> all of it — into an **optimized set**: the human-friendly format **supplemented** (never replaced)
> by a **generated, transparent encoding** with search/access **akin to, but improved upon, RAG** —
> efficient, fast, transparent, performant. The result is an **optimized memory extension** that
> informs an agent *better than the current system does* — extending its effective experience,
> context, and memory of this project. **Platform-agnostic**: intuitively accessed by an optimized
> set of skills over a **secure API** usable by Claude Code, Grok, and any other platform — maybe a
> Mycelium-lang package of it, if the language is truly developed enough. **MIT licensed** — so
> Anthropic (or anyone) can extend their agents with it if it truly helps; the maintainer prefers
> compensation but explicitly prioritizes the ideas being helpful and widely utilized —
> a contribution to society writ large.

## 1. Why (the problem in the current system)

Today an agent's project memory is the hand-maintained pointer stack (CLAUDE.md → agent-context →
CURRENT-STATE → Doc-Index → api-index/lib-index → grep). It works — it is the externalized-cognition
substrate this repo deliberately built — but it is **read-optimized for humans and only
grep-optimized for agents**: retrieval is manual navigation, cross-cutting questions ("what did we
decide about X and why, across which DNs/issues/commits?") cost many file reads, and none of it is
consumable by *other* platforms. The goal is a generated, queryable, provenance-carrying layer over
the same ground truth, so any agent loads *answers with citations* instead of re-reading the corpus.

## 2. The four design forks — resolved (`Declared`; maintainer may override any)

Resolved by the orchestrator on the maintainer's stated constraints after the interactive confirm
failed (the session's standing pattern; same basis discipline as the trx2 launch record):

1. **Substrate = hybrid, two honest layers.**
   - **Layer 1 — deterministic structured index** (the floor): the proven in-repo pattern
     (`api-index`/`lib-index`: deterministic generation, grep-friendly + machine-readable, committed,
     drift-gated) **generalized to the whole corpus** — docs/RFC/ADR/DN, `issues.yaml`, CHANGELOG,
     research records, skills, source symbols. Fast, transparent by construction, boring on purpose.
   - **Layer 2 — VSA semantic memory** (the ceiling, and the genuine improved-on-RAG bet): encode
     facts/relations as hypervector structures using **Mycelium's own substrate** (`mycelium-vsa`
     bind/bundle; retrieval via resonator/cleanup decode) — structured, compositional retrieval with
     `EXPLAIN`-able decode traces, not opaque nearest-neighbor. Dogfoods the language's actual value
     proposition as the memory system's engine.
   - Every answer carries **provenance** (source file/line/id + the guarantee tag of what it cites);
     the two layers are separately tagged and a Layer-2 answer always names its Layer-1 evidence.
2. **v0 languages = Rust core + Python ingestion.** The engine is a `mycelium-<NAME>` Rust crate
   (directly reusing `mycelium-vsa`/`mycelium-dense`; automatically a future transpiler-ladder
   target), ingestion/glue is Python (the `code_index.py` precedent). **The Mycelium-lang package is
   a phase-gated milestone** (M-1019), not v0 — honest about toolchain maturity (boot10/M-993).
3. **API = MCP server + plain HTTP/JSON**, one core behind two thin fronts, token-scoped auth.
   MCP gives native tool ergonomics to every MCP-speaking platform; HTTP is the universal floor
   (Grok, curl, anything). The "optimized set of skills" rides on these fronts.
4. **Sequencing = capture + mint now; the build wave fires on naming.** This note + epic E39-1 +
   M-1015…M-1019 + the stowed kickoff (`mem`) land now; the parallel build wave (the maintainer's
   requested shape — §5) launches the moment the maintainer delivers the name, since the crate
   cannot scaffold nameless.

## 3. What it is NOT (boundaries, stated up front)

- **Not a replacement for the human corpus.** The generated encoding *supplements* the
  human-friendly format; source remains ground truth (the api-index posture, inherited verbatim).
- **Not a chat-history memory.** It encodes the *corpus* (methodologies, decisions, intents,
  language docs, tracker state) — durable project knowledge, not conversation logs.
- **Not a better-than-RAG claim until measured.** §6 gates that claim behind an `Empirical` eval.
- **Not platform-locked.** MIT-only (house rule; CONTRIBUTING §Licensing), no vendor coupling in
  the core; the MCP front is one adapter among N.

## 4. Minted shape (slots verified free; mitigation #1)

- **E39-1 (epic)** — the program. (E35–E38 deliberately skipped: DN-86 §6 *proposes* them for the
  transpiler front-ends; minting over that reserved range would collide two programs' numbering.)
- **M-1015** — corpus ingestion + the deterministic structured index (Layer 1) over the whole corpus.
- **M-1016** — query engine + provenance: every answer carries citations + tags; `EXPLAIN`-able.
- **M-1017** — the API fronts (MCP + HTTP, token-scoped) + the cross-platform access-skill set.
- **M-1018** — the VSA semantic layer (Layer 2) + the eval harness (§6) — the improved-on-RAG bet.
- **M-1019** — the Mycelium-lang package (phase-gated on boot10/M-993 + the DN-85/M-1006 ladder).

## 5. The build pattern (the maintainer's requested shape)

Verbatim intent: *"parallel maxx it up a bit — orchestrate some tightly scoped epic orchestrators
who themselves orchestrate issue/change-scoped leaf sonnet agents, all working on disjoint by
design; each common-touch or conflicting file gets sorted out by their lowest single common parent,
working that way up the tree in change-scoped PRs through integration into main."*

This **is** the repo's ratified pattern — CLAUDE.md §Fractal Swarm (nested orchestrators, ownership
rises to the nearest shared parent) + §Concurrent-PR development (change-scoped leaf PRs, per-PR
agent review via `/pr-land`, tier promotion `dev → integration → main`) — so the kickoff (`mem`)
simply *instantiates* it: one orchestrator, one epic orchestrator per M-issue lane, Sonnet leaves in
isolated worktrees, disjoint dirs by construction, shared files integrator-owned.

## 6. Honesty posture (binding on the program)

1. **The improved-on-RAG claim is `Empirical`-gated:** M-1018 ships an eval harness (a question set
   drawn from real agent tasks over this corpus; graded on answer correctness, provenance fidelity,
   and latency) comparing Layer 2 against the Layer-1 baseline (and optionally a conventional
   embedding baseline). Until Layer 2 measurably beats/complements the baseline, the claim stays
   aspiration, and the system honestly serves Layer-1 answers (G2/VR-5).
2. **Provenance is mandatory, not best-effort:** an answer without a resolvable citation is a
   refusal, not an answer (the never-silent rule applied to retrieval).
3. **Freshness is drift-gated** like the indices it generalizes: the encoding regenerates
   deterministically from the corpus; a stale encoding fails a gate, never silently serves.
4. **Security floor for the API:** token-scoped auth, read-only by default, no secret material in
   the encoded corpus (the gitleaks gate runs over generated artifacts too).
5. **MIT-only** (ADR-022 §7 / CONTRIBUTING) — explicitly intended for anyone, Anthropic included,
   to extend their agents with; the maintainer's stated preference (compensation preferred, wide
   usefulness prioritized) is recorded here as intent, not a license term.

### 6.1 — Eval-gate verdict record (append-only, M-1018)

The improved-on-RAG gate (§6 item 1). Each M-1018 `tero-eval` run appends a verdict; history is
never rewritten (the in-crate mirror is `crates/mycelium-tero/eval/VERDICT.md`).

- **Run 1 (2026-07-07, PR #1254) — CLOSED (serving Layer-1).** Over the PR-#1254 **dev** codebook of
  5141 rows — *frozen, not re-run for this tier promotion; the promoted integration index is 5119
  rows (the isolated tree carries fewer corpus items)* — (seed
  `0x7E7010185EEDC0DE`, 16 questions, k=5, host x86_64-linux): correctness@1 Layer-1 **10/16
  (0.625)** vs Layer-2 6/16 (0.375); correctness@5 Layer-1 **16/16** vs Layer-2 8/16; provenance
  fidelity 1.000 both; latency Layer-1 **3.25M** ns/query vs Layer-2 85.9M (~26×). Layer-2 does not
  beat Layer-1 on correctness (beyond the 20% band) or latency, so the gate stays **Closed**,
  `layer2_enabled` remains `false`, and the system serves Layer-1 — the improved-on-RAG claim stays
  aspiration (G2/VR-5). Nothing was tuned to force a pass; the decode thresholds are `Declared` and
  the eval measures raw recall without them.

## 7. Definition of Done (for this note)

DN-87 is **Resolved** when the maintainer (a) confirms/overrides the four §2 resolutions, (b)
delivers the name, and (c) the `mem` kickoff fires its first wave. The program's own DoD lives in
E39-1 and its children.

## Changelog

- 2026-07-07 — Created, **Proposed**. Captures the maintainer's memory-substrate vision
  (near-verbatim), the four orchestrator-resolved design forks (`Declared`), the minted shape
  (E39-1, M-1015…M-1019, kickoff `mem`), the build pattern mapping onto the ratified swarm/PR
  discipline, and the binding honesty posture (Empirical-gated retrieval claims, mandatory
  provenance, drift-gated freshness, MIT).
- 2026-07-07 — **Named `mycelium-tero`** (maintainer; quiet homage to Atsushi Tero, for his
  contribution to science and engineering). Kickoff `mem` fired: the workspace-registered
  `crates/mycelium-tero` scaffold landed with the naming PR; E39-1 → `in-progress`; wave 1 =
  the M-1015 lane. DoD items (b)+(c) met; (a) — the §2 fork confirmations — remains with the
  maintainer before this note moves to Resolved.
- 2026-07-07 — **M-1016/M-1017/M-1018 landed** (PR #1241, PR #1254). The query engine + mandatory
  provenance (M-1016); the API fronts (MCP + HTTP, token-scoped) + the 4 cross-platform skills
  (M-1017; ADR-044 — the workspace's first async runtime); and the VSA Layer-2 + Empirical eval gate
  (M-1018). **Gate verdict Run 1 = CLOSED** (§6.1): Layer-2 does not beat Layer-1, so the system
  serves Layer-1 and the improved-on-RAG claim stays aspiration. Minted **M-1020** (native HTTPS/TLS
  for the HTTP front, maintainer follow-on). The §2 fork confirmations remain the last gate before
  this note → Resolved.
