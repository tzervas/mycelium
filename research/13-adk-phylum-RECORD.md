# Research Record 13 — ADK-as-Phylum: porting Google's Agent Development Kit to Mycelium (RFC-0023 / RP-9)

> **Status: Empirical/Declared** — source + RFC/ADR/DN are ground truth; this record is the
> research basis for **RFC-0023** (Draft). It captures the *fractured research methodology* pass
> (four focused sub-reasoners sharing one cross-context packet) that produced the ADK→Mycelium
> concept-map, the phylum-surface design, the honesty argument, and the harness wiring. It does
> **not** ratify anything: RFC-0023 stays **Draft** pending the follow-up deep pass (RP-9). Every
> ADK claim cites a public source; every Mycelium claim cites a corpus `file:line`.

---

## 1. Scope

**In scope (design-decidable here):** a faithful capture of Google ADK's abstractions
(Agent / Tool / Session-State-Memory / Runner / multi-agent / model layer) from public sources;
the mapping of each onto Mycelium's lexicon and runtime model (`nodule`/`colony`/`hypha`,
RFC-0008); the Tool / Session / Model-layer phylum surface following the RFC-0016 §4.1 per-op
contract; the model layer's wiring to the existing LLM harness (`tools/llm-harness/` +
`crates/mycelium-bench/src/llm.rs`); and the honesty argument (the differentiator).

**Out of scope (NOT discharged here — RP-9 follow-up):** an implemented `mycelium-adk` crate; a
ratified concept-map; the E7-1 generics/traits work (`Tool<In,Out>`/`Agent`) and the E7-2 runtime
constructs (`colony { … }`/`hypha`) the surface depends on; any *measured* LLM-leverage claim for
ADK-shaped generation (that inherits DN-09's open posture). RFC-0023 is therefore Draft.

**Method.** A clean *fractured* re-launch after a prior single-pass attempt stalled: the
orchestrator assembled one tight cross-context packet, then fanned out four Opus max-effort
sub-reasoners — (1) ADK concept research (web), (2) Agent↔runtime mapping, (3) Tool+model layer,
(4) honesty model — and synthesized their cited findings into RFC-0023 + this record.

---

## 2. Findings

### T12.1 — ADK's abstractions, faithfully captured (sub-reasoner 1, web-cited)

Google's ADK (open-source; `github.com/google/adk-python`, docs now `adk.dev`, formerly
`google.github.io/adk-docs`) is organized around six abstractions, captured in RFC-0023 §3's
concept-map with per-claim source URLs. The load-bearing distinction for the port is **LLM-agent
(`LlmAgent`, alias `Agent` — reasoning, the model decides flow) vs. workflow-agent
(`SequentialAgent`/`ParallelAgent`/`LoopAgent` — deterministic, code decides flow)**. ADK states
this explicitly: workflow agents orchestrate "without consulting an AI model for the orchestration"
(adk.dev/agents/workflow-agents/). This line is exactly Mycelium's RT2-deterministic vs
RT3-nondeterministic boundary (T12.2), which is why the mapping is clean. Key fields verified from
`adk-python` source (`Agent: TypeAlias = LlmAgent`; `model`/`instruction`/`tools`/`sub_agents`/
`output_key`; `BaseAgent.run_async`; `Runner.run_async` yields `Event`s persisted to the session;
`EventActions.transfer_to_agent`/`escalate`; `BaseLlm.generate_content_async`; `LiteLlm` for
non-Gemini providers). Full source URL list + could-not-verify items are in RFC-0023 §11 / §3.

### T12.2 — Agent / Runner / multi-agent → the Mycelium runtime model (sub-reasoner 2)

- **Agent definition → `nodule`** (static org. unit, DN-06; `nodule` is *Active* —
  `crates/mycelium-l1/src/token.rs:25,213`). A *running* agent invocation → a **`colony`**: the
  RFC-0008 §4.7 "structured scope of cooperating `hypha` under shared cancellation + supervision"
  (`docs/rfcs/RFC-0008-Runtime-and-Concurrency-Execution-Model.md:257-259`).
- **The agent's reason→act→observe loop → a `hypha`** (one concurrent execution unit, §4.5
  `RFC-0008:219`); **`Runner.run()` → the colony's structured-concurrency scheduler** (RT7
  no-orphan join, `RFC-0008:148-156`), realized by the M-357 executor. The ADK Event stream maps
  to **values moving over a typed SPSC channel** (RT1, `RFC-0008:91-97`; the M-357 follow-on
  channels).
- **Workflow agents:** `SequentialAgent` → ordered `hypha` chain (RT2); `ParallelAgent` → a
  `colony` fork/join over the M-357 deterministic executor (RT2 fragment, `RFC-0008:99-107`) — note
  ADK ParallelAgent runs branches *isolated* (no auto state-sharing), which aligns with RT1
  share-nothing; `LoopAgent` → **bounded** `for`/recursion **only** — an *unbounded* ADK loop has
  **no Mycelium mapping** (no `while`/`loop`; lexicon `lang-lexicon-syntax.md:160-163`). ADK
  `LoopAgent` has a `max_iterations` cap and an `escalate` exit, so the *bounded* form maps; the
  port must require the bound. **Hard porting constraint.**
- **LLM-driven delegation (`transfer_to_agent`) → an RT3 construct** with a named, reified,
  EXPLAIN-able selection policy (the third RFC-0005 site, like `forage`): the model's choice is
  `Declared`/`Empirical`, **never `Proven`** (RT3 `RFC-0008:109-116`; RT5 `:127-133`). ADK's
  `AgentTool` (agent-as-tool, caller retains control) maps to an ordinary typed tool call; only
  `transfer_to_agent` (control hands off, model-chosen) is the RT3 construct. **This is the
  determinism boundary — the load-bearing line of the whole mapping.**

### T12.3 — CORRECTION to the packet: the E7-2 gap is the *parser*, not the *lexer*

The cross-context packet (and `issues.yaml` M-665/M-666, and the memory file) described the 10
runtime terms + `hypha`/`colony` as "not yet in the lexer". **Direct source inspection refutes
this:** all 10 runtime terms **and** `hypha`/`colony` are present in both the `Tok` enum
(`token.rs:29-60`) and `keyword()` (`token.rs:216-229`) — **M-665 lexer reservation has landed.**
The remaining E7-2 gap is purely the **parser construct (M-666)**: no parser production consumes
these tokens, so `colony { … }` / `hypha <expr>` produce an explicit "reserved … not yet active"
parse error. RFC-0023 records this correction and FLAGs it; the memory file / issues.yaml should be
re-checked before citing the gap as "lexer". (Verified by the orchestrator directly,
`token.rs:29-60,216-229`.)

### T12.4 — Two landed runtime realizations (sub-reasoner 2, orchestrator-verified)

Two distinct, already-landed Rust runtimes can back the `mycelium-adk` colony scheduler:
`mycelium-std-runtime::colony::Scope::join_all` (FIFO, **Exact** sweep, ADR-020 §4 —
`crates/mycelium-std-runtime/src/colony.rs:103`) and `mycelium-mlir::runtime`
(`Scope::run_sequential`/`run_interleaved`/`run_dataflow`; `Colony` = a type alias —
`crates/mycelium-mlir/src/runtime.rs:106,144,172,214`). `TaskOutcome` has exactly the four
never-silent variants `Done`/`Failed`/`BudgetExhausted`/`Cancelled`
(`crates/mycelium-interp/src/supervise.rs:94-103`) — the RT4/C3 channel. The port must pick one
realization deliberately (FLAG; see RFC-0023 §7).

### T12.5 — Tool / Session / Model layer (sub-reasoner 3)

- **Tool → a typed `fn -> Result<Out, ToolError>`** carrying an honest guarantee tag + an explicit
  error set + declared effects (RFC-0016 §4.1 C1/C2/C6 —
  `docs/rfcs/RFC-0016-Core-Library-and-Standard-Library.md:82-100`). External-infra tools are a
  **`graft`** over an affine `Substrate` handle (Glossary `docs/Glossary.md:115-117`;
  `RFC-0008:223`). **The tool's call-schema is *derived from the typed signature*** — the type *is*
  the schema, so it cannot drift (honest-by-construction; C4). This is the exact dual of ADK's
  `build_function_declaration()` (which derives the model-facing `FunctionDeclaration` from the
  Python signature + docstring): both derive the schema from the signature, but Mycelium's is
  *type-checked*, not docstring-driven. **`Tool<In,Out>` needs generics → E7-1/M-657**
  (`tools/github/issues.yaml:2066-2074`). Generics are not yet in the language (`issues.yaml:1879`).
- **Session/State → content-addressed value state** (`std.content`, RFC-0016 §4.3 `:139`;
  ADR-003). State updates are **values that MOVE** (RT1, `RFC-0008:91-97`); the Event list is an
  append-only log of content-addressed values; Memory is a separate store. **FLAG — the
  immutability tension:** ADK's `State` is *mutable* with prefix scoping
  (`app:`/`user:`/`temp:`/session) and a transactional `_delta` (`adk-python` `sessions/state.py`);
  Mycelium is immutable-value (C4 `:92-94`). Honest v0 = state-as-snapshots; the principled
  concurrent-merge story is `fuse` (RT6 join-payload/meet-guarantee, `RFC-0008:135-145`), a
  non-lawful merge being an explicit RT3 conflict, never a silent overwrite. `fuse` is
  Ratified-not-yet-lexed (E7-2). ADK's delta-merge is itself snapshot-like, which eases the mapping.
- **Model layer → wired to `tools/llm-harness`**, reusing the `GrokLlmReport`/`LlmReport` schema
  (`crates/mycelium-bench/src/llm.rs`). ADK's `BaseLlm`/`LiteLlm` (OpenAI-compatible providers incl.
  Anthropic/xAI) maps onto the harness's live arm via `grok/` (xAI/Grok, OpenAI-compatible) and the
  local arm via `harness.py` (llama.cpp). Honesty mechanisms, cited to `llm.rs`:
  `model_allowed_tags=["Declared","Empirical"]` (`:63-64`), tag preserved verbatim never upgraded
  (`:367-368,507-508`), `is_synthetic` gate (`:146-152,410-419`), never-silent parse errors /
  `deny_unknown_fields` (`:177-180,240-242`), the never-silent USD spend gate (best-effort, so a
  budget claim is `Declared` not `Proven` — `tools/llm-harness/README.md:97-101`). A missing
  key/model is a never-silent `SKIP`/`Err`, never a fabricated answer (`README.md:48-49,476`).

### T12.6 — The honesty model is the differentiator (sub-reasoner 4)

The decisive case for the port: Mycelium's substrate makes *silence unrepresentable* where Python
ADK is epistemically silent. Five parts, each contrasting Python-ADK-silent vs Mycelium-honest:

1. **LLM outputs are `Declared`/`Empirical`, never `Proven` (VR-5)** — a *type-level* allow-list
   (`llm.rs:63-64`; CLAUDE.md honesty rule). A hallucination cannot be laundered into a fact.
2. **Tool calls are never-silent (`Result`)** — a failed call is an explicit `Err` the loop must
   handle (CLAUDE.md #2; RT4 `RFC-0008:118-126`; C3 `:276-280`). (ADK wraps tool errors in dicts
   the model *may* ignore; Mycelium forces the branch.)
3. **Non-determinism is reified + EXPLAIN-able** — agent routing is a named RFC-0005 policy with
   mandatory EXPLAIN, not an opaque whim (RT3 `RFC-0008:110-116`; RT5 `:127-133`).
4. **The empirical honesty floor (no overclaim)** — DN-09 §9.2/§9.3: bare novel surface **0.0%**,
   grammar-in-context primer **100.0%** [Empirical], retention ratio **INDETERMINATE**; the
   leverage claim stays `Declared`/open. Mycelium does not claim LLMs are reliable; it claims the
   framework never *lets* an unreliable output masquerade as reliable.
5. **Synthetic ≠ real** — the harness refuses to present a fixture/mock as real-model evidence
   (`llm.rs:5-9,146-152`); the port inherits this.

### T12.7 — Rust-first reality + the E7-1/E7-2 dependency chain

No `mycelium-adk` crate exists (greenfield, orchestrator-verified). Self-hosting is blocked
(`issues.yaml`), so the port is **Rust-first**: a `mycelium-adk` Rust crate calling the
already-landed runtime (`mycelium-mlir::runtime` / `mycelium-std-runtime::colony` /
`mycelium-interp::supervise`) and harness (`mycelium-bench::llm` + `tools/llm-harness`), following
the same per-op-guarantee-matrix-as-data + `#![forbid(unsafe_code)]` convention every landed
`mycelium-std-*` crate uses (e.g. `crates/mycelium-std-content/src/lib.rs:1-30`). The *Mycelium-lang*
surface (`adk.agent`/`adk.tool`/`adk.session`/`adk.runner`/`adk.model`) is a **target**, gated on:

- **E7-1** (`issues.yaml:2041-2053`): generics (M-657) for `Tool<In,Out>`/`List<Event>`; traits +
  `impl` (M-659) for the `Agent`/`Tool` abstractions; effect annotations (M-660); `consume`/`grow`/`impl`
  surface (M-664). **`Tool<In,Out>`/`Agent` are the headline E7-1 dependency.**
- **E7-2** (`issues.yaml:2146-2158`): the `colony { … }`/`hypha` L1 constructs (M-666), then
  `fuse`/`reclaim`/`tier` (M-667), then the RT3 vocabulary for delegation/racing (M-668). **The
  runtime mapping's surface, and the RT3 delegation construct, are the E7-2 dependency.**

---

## 3. Decisions this record supports

- **RFC-0023 may be authored as Draft** on this basis: the concept-map (T12.1), the runtime
  mapping (T12.2), the Tool/Session/Model surface (T12.5), the honesty argument (T12.6), and the
  Rust-first/E7-x reality (T12.7) are all grounded. A maintainer **could** read RFC-0023 and decide
  the concept-map is faithful and the honesty case sound.
- **RFC-0023 stays Draft.** Ratification is gated on the RP-9 deep pass: implementing (or
  scaffolding) `mycelium-adk`, discharging the E7-1/E7-2 dependencies the surface needs, and an
  honest scoping of any ADK-shaped LLM-leverage claim (which inherits DN-09 / RP-1's open posture).
- **No LLM-capability claim is asserted.** Per VR-5 and DN-09, the record asserts *no* leverage
  result for ADK-shaped generation; the honest floor (T12.6 part 4) is the only measured statement.

---

## 4. Key sources

**ADK (public — full URL list in RFC-0023 §11 / sub-reasoner 1 source list):**
`adk.dev` (agents/workflow-agents/sessions/runtime/workflows-patterns) and
`raw.githubusercontent.com/google/adk-python/main/src/google/adk/...` (agents, tools, sessions,
events, models, memory).

**Mycelium corpus (file:line cited inline above):**

- `CLAUDE.md` — house rules (honesty lattice; G2 never-silent; no black boxes).
- `.claude/memory/lang-lexicon-syntax.md` — lexicon + reserved-word status + surface syntax.
- `docs/rfcs/RFC-0008-Runtime-and-Concurrency-Execution-Model.md` — RT1–RT7; §4.5 vocabulary; §4.7
  colony/composition.
- `docs/rfcs/RFC-0016-Core-Library-and-Standard-Library.md` — §4.1 per-op contract C1–C6; §4.5
  guarantee matrix; Tier-A modules.
- `docs/rfcs/RFC-0021-Semantic-Level-Projections.md` — projections as total inspectable views; the
  LLM-leverage carve-out (Declared/open).
- `docs/notes/DN-09-KC-2-Verdict.md` — the honest LLM-leverage posture; §9.2/§9.3 ablation.
- `crates/mycelium-bench/src/llm.rs` — `GrokLlmReport`; the honesty mechanisms in code.
- `tools/llm-harness/README.md` — the harness (live xAI + local llama arm; spend gate; never-silent
  SKIP).
- `crates/mycelium-l1/src/token.rs` — the lexer (the T12.3 correction).
- `crates/mycelium-{mlir,std-runtime,interp}/src/{runtime,colony,supervise}.rs` — the two landed
  runtimes + `TaskOutcome`.
- `crates/mycelium-std-content/src/lib.rs` — the per-op-matrix + never-silent convention the port
  follows.
- `tools/github/issues.yaml` — E7-1 (2041-2053), E7-2 (2146-2158) and their M-65x/M-66x children.

---

## 5. Honest-uncertainty register

- **The concept-map's fidelity is `Empirical`/`Declared`, not `Proven`.** ADK abstractions were
  captured from public docs/source at one point in time; ADK evolves (the docs site now reads
  "ADK 2.0" and adds graph/dynamic-workflow + router concepts that may post-date a pinned
  `adk-python` target — see RFC-0023 §11). Re-verify the API surface at RP-9.
- **The whole phylum surface is a *target*, not compilable today.** Every generic signature
  (`Tool<In,Out>`, `List<Event>`) needs E7-1/M-657; every runtime construct (`colony`/`hypha`/`fuse`)
  needs E7-2/M-666-667; the RT3 delegation construct needs M-668. The Rust-first semantics are
  available now; the surface sugar is future.
- **The State immutability tension is genuinely unresolved at the language level.** Snapshot-model
  is the honest v0; `fuse`-merge is the principled concurrent story but depends on RFC-0008 Phase-7
  construct activation. RFC-0023 flags it rather than papering over it.
- **No ADK-shaped LLM-leverage is measured.** It inherits DN-09 §9's INDETERMINATE retention ratio;
  RP-9 carves the measured question explicitly and never pre-writes a verdict (VR-5).
- **The packet contained a stale claim (T12.3) that direct source refuted.** That a load-bearing
  packet fact was wrong is itself a flag: RP-9 must re-verify the E7-1/E7-2 status against source,
  not against the planning artifacts.

---

## Meta — changelog

- **2026-06-21 — Created (RFC-0023 research pass; fractured methodology).** Four Opus sub-reasoners
  sharing one cross-context packet produced: the ADK concept-map (web-cited), the Agent↔runtime
  mapping (RFC-0008-grounded), the Tool/Session/Model surface (RFC-0016 + `llm.rs`-grounded), and
  the honesty-as-differentiator argument. Records the T12.3 correction (E7-2 gap is the parser, not
  the lexer — direct source check). States the LLM-leverage claim stays Declared/open (DN-09 / RP-9),
  the surface is a target gated on E7-1/E7-2, and the port is Rust-first. Supports RFC-0023 (Draft);
  ratification gated on the RP-9 deep pass. Honest-uncertainty register §5.
