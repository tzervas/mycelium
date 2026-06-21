# RFC-0023 — Agent Development Kit Phylum (`adk`)

| Field | Value |
|---|---|
| **RFC** | 0023 |
| **Status** | **Accepted** (2026-06-21 — maintainer ratification: design agreed; **Enacted** still gated on the `mycelium-adk` build + E7-1/E7-2). *Ratification trail:* **Draft** (2026-06-21 — research-grounded design from `research/13-adk-phylum-RECORD.md`, fractured-methodology pass; four Opus sub-reasoners over one cross-context packet). **Not ratified.** The follow-up deep pass (**RP-9**) gates ratification: it must scaffold/implement `mycelium-adk`, discharge the E7-1/E7-2 dependencies the surface needs (§7), and honestly scope any ADK-shaped LLM-leverage claim (which inherits DN-09's open posture). *Honesty (VR-5):* this Draft asserts **no** LLM-capability result. · **RP-9 research gate SUBSTANTIALLY DISCHARGED (2026-06-21, `dfr` session)** — the §11 register verified against primary ADK source (pinned `adk-python` **v2.3.0** / "ADK 2.0") + landed in-repo substrate by four fractured Opus sub-reasoners (`research/13-adk-phylum-RECORD.md` §6); **design soundness confirmed, no soundness falsification** (concept-map rows sound; tool-dispatch taxonomy complete & never-silent; harness reuse single-path; honesty differentiator grounded+falsifiable, stays `Declared`; RT2 `Empirical`; drift item 1 + parser-not-lexer item 3 resolved). **One open completeness item:** the §3 concept-map is incomplete for ADK 2.0 — **graph/Workflow** dropped + agent-`mode`/code-Router/`RunConfig`-budget rows missing — `dfb` must add them + re-pin §3 (G2: named, not closed). Other residuals: deferred-to-build empirical-on-code; a `ToolError` budget-arm; the runtime-realization choice (recommend `mycelium-mlir::runtime`); LLM-leverage (item 6) scoped, **no verdict** (DN-09 INDETERMINATE). **Pending maintainer ratification** — ratification additionally requires the `mycelium-adk` build + E7-1/E7-2 (per this Status), naturally honored by "pending"; an agent stages the discharge, the maintainer ratifies. · **RATIFIED → Accepted (2026-06-21, maintainer).** The one open completeness item is now **CLOSED** — §3 repaired (new **§3.7**: ADK 2.0 graph **Workflow Runtime**, operating-`mode`, code-Router, `RunConfig`-budget; §3 pinned **v2.3.0**). Ratified decisions: `adk.runner` on **`mycelium-mlir::runtime`** (R23-Q1; migrate to the std.runtime facade later); **`ToolError`** budget routes to **`TaskOutcome::BudgetExhausted`** (branded names canonical; RP-9 `TypeMismatch/Exec/Budget` = aliases); **Session** snapshot-v0, concurrent merge deferred to **`fuse`** (R23-Q2; never silent-overwrite); **LLM-leverage** stays no-verdict (separate measured pass, DN-09). **Enacted** gated on the build + E7-1/E7-2. |
| **Type** | Application / phylum design (a library port; rests on RFC-0008/0016/0021 + the LLM harness; no L0 or L1 kernel change — KC-3) |
| **Date** | 2026-06-21 |
| **Depends on** | RFC-0008 (the runtime/concurrency model — `colony`/`hypha`, RT1–RT7; the §4.7 composition contract); RFC-0016 (§4.1 per-op contract C1–C6, §4.5 guarantee matrix — the phylum-authoring convention); RFC-0021 (semantic-level projections / `LlmCanonical` — the LLM-facing surface, leverage Declared/open); DN-09 (the honest LLM-leverage posture — KC-2 = proceed); the LLM harness (`tools/llm-harness/` + `crates/mycelium-bench/src/llm.rs` — the model layer's real-call substrate); ADR-003 (content-addressed identity — Session/State); ADR-013 (`spore` — an agent deploys as a spore); LR-8 (`substrate` — the `graft` capability handle); G2 (never-silent); VR-5 (honest tags); KC-2/KC-3 |
| **Coupled with** | **E7-1** (L1 stage-1 language completeness — generics M-657 / traits + `impl` M-659 / effects M-660 / `consume`-`grow`-`impl` M-664: the `Tool<In,Out>`/`Agent` surface dependency); **E7-2** (RFC-0008 runtime vocabulary lexing → construct activation — `colony { … }`/`hypha` M-666, `fuse`/`reclaim`/`tier` M-667, the RT3 delegation/racing vocabulary M-668: the runtime-mapping surface dependency); the prospective `mycelium-adk` Rust crate (greenfield — does not exist yet) |

---

## 1. Summary

Google's **Agent Development Kit (ADK)** is an open-source framework for building LLM agents:
an **Agent** (LLM-driven or workflow), **Tools** the agent calls, a **Session/State/Memory** layer
for context, a **Runner** event loop, multi-agent composition (delegation + sub-agent trees), and a
pluggable **model layer**. This RFC designs a faithful **Mycelium port of ADK as a phylum** — a set
of `nodule`s (`adk.agent`, `adk.tool`, `adk.session`, `adk.runner`, `adk.model`) under the RFC-0016
§4.1 per-op contract.

The central claim:

> ADK's abstractions map onto Mycelium's existing runtime and library models **without inventing a
> new kernel primitive** (KC-3) — an agent invocation is a `colony` of cooperating `hypha`
> (RFC-0008 §4.7), a tool is a typed `fn -> Result<Out, ToolError>` (an external-infra tool is a
> `graft` over an affine `Substrate`), Session/State is content-addressed value state (ADR-003),
> and the model layer wires to the **existing LLM harness**. And — the differentiator (§6) — the
> Mycelium port is **honest where Python ADK is silent**: an LLM output is type-forbidden from
> `Proven`/`Exact`, a tool failure is a non-silent `Result::Err`, and nondeterministic agent
> routing is a reified, `EXPLAIN`-able policy, not an opaque whim.

The RFC is **Draft** because it is a *design against partly-unbuilt foundations*: the `colony`/`hypha`
L1 surface (E7-2/M-666) and the generics/traits for `Tool<In,Out>`/`Agent` (E7-1/M-657/M-659) are
not yet landed, and no `mycelium-adk` crate exists. The honest path is **Rust-first** (§7): a
`mycelium-adk` crate over the already-landed runtime + harness, with the Mycelium-language surface
as a target that lands as E7-1/E7-2 complete. The **RP-9** deep pass (§10) gates ratification.

*Honesty up front (VR-5).* This RFC makes **no** claim that LLMs reliably author Mycelium-ADK code.
The measured floor (DN-09 §9) is *weak-but-recoverable* — grammar-in-context primer 100%, bare
surface 0%, retention ratio **INDETERMINATE** — and stays `Declared`/open (§6.4, §10). The
differentiator is **not** a claim about model quality; it is that the substrate never *lets* an
unreliable model output masquerade as reliable.

---

## 2. Motivation

### 2.1 Why port ADK

ADK is a well-designed, widely-used agent framework with a clean conceptual core (Agent / Tool /
Session / Runner / model). Porting it to Mycelium (a) **dogfoods** the runtime tier (RFC-0008) and
the standard library (RFC-0016) against a real, demanding application — multi-agent orchestration is
exactly the `colony`/`hypha` use case RFC-0008 §4.7 was designed for, and is the named **E7-2 gap**
the runtime vocabulary exists to fill; and (b) demonstrates the project's thesis — *honest,
never-silent guarantees* — on the one workload where silence is most dangerous: a machine that
**acts** on LLM output. An agent framework that silently launders a hallucination into a tool call
into an action is the failure mode Mycelium's honesty rule exists to prevent.

### 2.2 The honesty gap ADK leaves open

In Python ADK an LLM response is an untyped `str`/`dict` (no epistemic status), a tool may return
`None`/raise and the loop proceeds, and `transfer_to_agent` routing is an opaque model decision with
no reified record. None of these are ADK defects — they are the Python substrate. Mycelium's
substrate makes each of them *unrepresentable as silent* (§6). This is the port's reason to exist
beyond a straight translation.

### 2.3 What this RFC does and does not settle

It **settles** (as a Draft design): the ADK→Mycelium concept-map (§3, source-cited), the phylum
surface (§4, Mycelium syntax), the honesty model (§6), the harness wiring (§5), and the Rust-first
reality + the E7-1/E7-2 dependency chain (§7). It **does not settle**: an implemented crate, a
ratified map, or any measured LLM-leverage claim — all deferred to RP-9 (§10).

---

## 3. The ADK → Mycelium concept-map (researched, source-cited)

ADK's abstractions, captured from the official docs (`adk.dev`, formerly `google.github.io/adk-docs`)
and the `google/adk-python` source on `main`, then mapped to Mycelium. Every ADK row cites a public
source; every Mycelium row cites a corpus location. (Could-not-verify items: §11.)

**Pinned (RP-9 ratification, 2026-06-21).** The §3 map is verified against **`adk-python` v2.3.0**
(ADK 2.0, GA 2026-05-19); **§3.7** adds the ADK 2.0 graph **Workflow Runtime** + collaborative-
orchestration concepts the Phase-1 draft omitted. `dfb` re-checks the surface at build (R23-Q5).

### 3.1 Agent

| ADK concept (source) | Faithful description | Mycelium mapping | Grounding |
|---|---|---|---|
| **`BaseAgent`** (`adk-python/.../agents/base_agent.py`) | Foundational unit; defines the agent tree (`name`, `description`, `sub_agents`, auto `parent_agent`) + run entry points (`run_async`/`run_live`). | A **`nodule`** (static org. unit) for the definition; its run is a **`colony`**. | DN-06; `token.rs:25,213`; RFC-0008 §4.7 (`:257-259`) |
| **`LlmAgent`** (alias **`Agent`**; `agents/llm_agent.py`) | Self-contained unit; the LLM *dynamically decides* flow/tools/output. Fields: `model`, `instruction`, `description`, `tools`, `sub_agents`, `output_key`, `output_schema`, `planner`, callbacks. | An `adk.agent` value: a `nodule` of typed defs whose *running* invocation is a `colony` whose model-decisions are **RT3** (reified, EXPLAIN-able). | `RFC-0008:109-116` (RT3); §4 below |
| **`SequentialAgent`** (`workflow-agents/sequential-agents/`) | Runs sub-agents in fixed order, threading one `InvocationContext` (state via `output_key` → `{placeholder}`). Deterministic. | An **ordered `hypha` chain** (spawn-order sequential run). | RT2 (`RFC-0008:99-107`); `runtime.rs:144` |
| **`ParallelAgent`** (`workflow-agents/parallel-agents/`) | Runs sub-agents concurrently in **isolated branches** (no auto state-sharing); results via distinct `output_key`s. Deterministic. | A **`colony` fork/join** over the M-357 deterministic executor (RT2 fragment); branch isolation = RT1 share-nothing. | RT1/RT2 (`RFC-0008:91-107`); `runtime.rs:172`, `colony.rs:103` |
| **`LoopAgent`** (`workflow-agents/loop-agents/`) | Repeats sub-agents until `max_iterations` **or** a sub-agent sets `actions.escalate`. | **Bounded** `for`/structural recursion (Total). *Unbounded loops have no mapping.* | `for` Active (`token.rs:123`); lexicon `:160-163` |
| **Custom agent** (`custom-agents/`) | Subclass `BaseAgent`, implement `_run_async_impl` for bespoke flow. | A user `nodule` composing the `adk.runner` primitives directly. | §4.4 |

### 3.2 Tool

| ADK concept (source) | Faithful description | Mycelium mapping | Grounding |
|---|---|---|---|
| **Function tool** (`tools/function_tool.py`) | A plain Python fn → tool: a `FunctionDeclaration` is **built from the signature + type hints + docstring** (`build_function_declaration`); the return (a dict) flows back to the model; errors are wrapped in error dicts. | A typed `fn tool(args: In) -> Result<Out, ToolError>`; the **call-schema is *derived from the typed signature*** (type *is* the schema — cannot drift). | RFC-0016 §4.1 C1/C4 (`:82-94`) |
| **`FunctionTool`** (same) | Explicit wrapper; validates args, injects `ToolContext`, sync/async detect. | The `fn`'s checked signature + an injected `ToolContext` value. | §4.2 |
| **`LongRunningFunctionTool`** (`Event.long_running_tool_ids`) | Long-running/async-result tools; call-id tracked for later correlation. | A `hypha` whose result returns over the Event channel (RT4 explicit outcome). | `RFC-0008:118-126`; `supervise.rs:94-103` |
| **`AgentTool`** (agent-as-tool; `workflows/patterns/`) | Wraps an agent as a callable; **caller retains control**, gets the result back. | An ordinary typed tool call into a sub-`colony`; result returns (not a control handoff). | §4.2/§4.4 |
| **`ToolContext`** (`agents/context.py`) | Per-call context: `state`, `actions`, `function_call_id`, artifacts, memory, auth. | A value parameter carrying the (immutable-snapshot) state + an explicit actions record. | §4.2; ADR-003 |
| **External-infra tool** (network/FS/API) | A tool calling outside the process. | A **`graft`** — a capability contract over an affine `Substrate` handle, consumed once. | Glossary `:115-117`; `RFC-0008:223`; LR-8 |

### 3.3 Session / State / Memory

| ADK concept (source) | Faithful description | Mycelium mapping | Grounding |
|---|---|---|---|
| **`Session`** (`sessions/session.py`) | One ongoing interaction: `id`, `app_name`, `user_id`, `state` (dict), `events` (ordered `list[Event]`), `last_update_time`. | A content-addressed `Session` value: a `State` snapshot + an append-only `List<Event>`. | RFC-0016 §4.3 `content` (`:139`); ADR-003 |
| **`State`** (`sessions/state.py`) | Key-value scratchpad, prefix-scoped (`app:`/`user:`/`temp:`/session), transactional `_delta`. | A content-addressed value snapshot; "mutation" → a **new** snapshot that **moves** (RT1). **FLAG: immutability tension** (§4.3). | RT1 (`RFC-0008:91-97`); C4 (`:92-94`) |
| **`Event`** (`events/event.py`; inherits `LlmResponse`) | The unit yielded + the session record entry: `author`, `content`, `actions`, `partial`, `is_final_response()`. | A content-addressed log entry; values move over a typed SPSC channel. | RT1; `channel.rs` (M-357 follow-on) |
| **`EventActions`** (`events/event_actions.py`) | Control/state signals: `state_delta`, `transfer_to_agent`, `escalate`, `skip_summarization`. | An explicit reified record (state-delta = next snapshot; `transfer` = an RT3 routing decision). | RT3 (`RFC-0008:109-116`) |
| **`Memory` / `BaseMemoryService`** (`memory/base_memory_service.py`) | Long-term cross-session recall: `add_session_to_memory`, `search_memory`. | A **separate** content-addressed store (distinct from per-turn State). | RFC-0016 §4.3 (`:139`) |
| **`SessionService`** (`adk.dev/sessions/`) | CRUD + append-events/modify-state; in-memory + DB/Vertex variants. | A service `nodule` over content-addressed snapshots; persistence is a `graft`. | §4.3; LR-8 |

### 3.4 Runner

| ADK concept (source) | Faithful description | Mycelium mapping | Grounding |
|---|---|---|---|
| **`Runner`** (`runners.py`) | Execution engine: `run()`/`run_async(user_id, session_id, new_message, …)` **yields `Event`s**; events are **persisted before being yielded**; drives Agent ↔ Tool ↔ Model. | The **colony scheduler**: a structured-concurrency `Scope` join driving the agent's `hypha`, yielding values over the Event channel, persisting snapshots. | RT7 no-orphan join (`RFC-0008:148-156`); `runtime.rs:144,172`; §4.4 |
| **`InMemoryRunner`** (same) | Runner pre-wired with in-memory session/memory/artifact services. | A `Runner` configured with in-memory snapshot stores. | §4.4 |
| **Event loop** (`adk.dev/runtime/`) | Yield/pause/resume: run logic, yield at tool/model breakpoints, persist via services. | The `Scope`'s cooperative-stepping run (RT2); a breakpoint = a `hypha` awaiting a tool/model `Result`. | RT2 (`RFC-0008:99-107`) |

### 3.5 Multi-agent composition

| ADK concept (source) | Faithful description | Mycelium mapping | Grounding |
|---|---|---|---|
| **Hierarchy** (`base_agent.py`) | `sub_agents` + auto `parent_agent`; a tree with one `root_agent`. | A parent **`colony`** spawning child `hypha`/nested colonies (structured scope tree). | RT7 (`RFC-0008:148-156`); §4.7 C2 (`:272-275`) |
| **Coordinator/Dispatcher** (`workflows/patterns/`) | An `LlmAgent` whose LLM reads each sub-agent's `description` and routes to the best-fit specialist. | A coordinator `colony` whose routing is an **RT3** reified selection policy (the third RFC-0005 site). | RT3 (`RFC-0008:109-116`); `:224` (`forage`) |
| **LLM-driven delegation (AutoFlow)** — `transfer_to_agent(agent_name=…)` (`event_actions.py`) | The LLM emits a framework call; AutoFlow **hands full control** to the named agent. Gated by `disallow_transfer_to_parent/peers`. | An **RT3 construct**: the model's transfer choice is `Declared`/`Empirical`, **never `Proven`**; the policy is `EXPLAIN`-able. | RT3/RT5 (`RFC-0008:109-133`); VR-5 |

### 3.6 Model layer

| ADK concept (source) | Faithful description | Mycelium mapping | Grounding |
|---|---|---|---|
| **`BaseLlm`** (`models/base_llm.py`) | `model: str`; `generate_content_async(req, stream)` async-yields `LlmResponse`; `connect()` for live; `supported_models()` regexes. | An `adk.model` interface `fn` returning a tagged `Result<LlmOutcome, ModelError>` — wired to the harness. | §5; `llm.rs` |
| **`Gemini`** (`models/google_llm.py`) | Default Gemini impl; calls the GenAI client. | (Not the first backend — the harness's xAI/local arms are.) See §5 / §11. | `tools/llm-harness/README.md` |
| **`LiteLlm`** (`models/lite_llm.py`) | Wrapper over LiteLLM; `model="anthropic/claude-…"`/`"openai/gpt-…"`; provider from prefix; converts `LlmRequest` → provider payloads. | The OpenAI-compatible bridge maps onto the harness's `grok/` arm (xAI, OpenAI-compatible) + local `harness.py`. | §5; `README.md:94-101` |
| **`LlmRequest`/`LlmResponse`** (`models/llm_response.py`; `Event` ⊂ `LlmResponse`) | Model I/O envelope: `content`, `usage_metadata`, `partial`, `error_code`, `get_function_calls()`. | Reuse the harness `GrokLlmReport`/`GrokOutcome` result schema; the response value carries a guarantee tag. | `llm.rs:243-258,356-383` |
| **`LLMRegistry`** (`models/registry.py`) | String → `BaseLlm` subclass via `supported_models()` regex `fullmatch`. | A model-resolution `fn` (string → backend); a no-match is a never-silent `Err`. | C1 (`RFC-0016:82-84`) |

### 3.7 Graph Workflow Runtime + collaborative orchestration (ADK 2.0 — `adk-python` v2.3.0)

> Added at RP-9 ratification (2026-06-21). ADK 2.0 (GA 2026-05-19) replaced the hierarchical agent
> executor with a **graph-based Workflow Runtime** — *"a slider from dynamic, model-led reasoning to
> strict, deterministic workflows."* This is the explicit graph form of §3's load-bearing RT3↔RT2 line,
> so it maps node-for-node onto the `colony`/`hypha` model with **no invented machinery**. Sources:
> `adk.dev/2.0`; `adk-python` #4581 (GraphAgent — directed-graph workflow orchestration).

| ADK 2.0 concept (source) | Faithful description | Mycelium mapping | Grounding |
|---|---|---|---|
| **Workflow Runtime** (graph execution engine; `adk.dev/2.0`) | Graph-based engine composing deterministic execution flows; the "slider" from model-led to strictly deterministic. | A **`colony` DAG** where **each node carries its own honesty tag**: rule-driven nodes **RT2** (deterministic), model-led nodes **RT3** (reified, `EXPLAIN`-able). The slider *is* the per-node RT2/RT3 boundary. | RT2/RT3 (`RFC-0008:99-116`); §3 load-bearing line |
| **Routing** (Workflow Runtime / code-driven Agent-Router) | Directs flow to the next node(s) by rule or model decision. | A reified selection policy (the RFC-0005 selection site): rule-routing **RT2**, model-routing **RT3** — a `Declared`/`Empirical` choice, never `Proven`. | RT3; RFC-0005 §2; VR-5 |
| **Fan-out / fan-in** | Split to parallel branches, join their results. | A **`colony` fork/join**; branches are RT1 share-nothing; the join is a structured-scope barrier (RT7). | RT1/RT7 (`RFC-0008:91-97,148-156`); `crates/mycelium-mlir/src/runtime.rs` |
| **Loops / retry** | Repeat a node / re-attempt on failure, bounded. | **Bounded** structural recursion (Total) for loops; retry = an **RT5** declarative recovery policy (bounded effects). *Unbounded → no mapping.* | `for` Active (`token.rs:123`); RT5 (`RFC-0008:127-133`) |
| **Dynamic nodes** (graph mutated mid-run) | The graph adds/rewrites nodes at runtime (model-driven). | An **RT3** reified decision — the node-set change is a `Declared`/`Empirical` selection, materialized + `EXPLAIN`-able, never a silent rewrite. | RT3; VR-5; G2 |
| **Human-in-the-loop / interrupt → resume** | Pause for external (human) input; resume later. | A **`hypha` awaiting an external `Result`** (never-silent); a *durable* pause is a **`cyst`** checkpoint (dormancy → reclaim). | `cyst` (Glossary; DN-03); RT4 (`RFC-0008:118-126`) |
| **Nested workflows** | A node is itself a sub-workflow. | A **nested `colony`** (a structured-scope subtree). | RT7; §4.7 C2 |
| **State management** (per-graph state across nodes) | State threaded across the graph's nodes. | A content-addressed `State` **snapshot that moves** (RT1); concurrent-branch merge = an explicit **`fuse`** (deferred, E7-2 M-667), never a silent overwrite. | RT1; §4.3; R23-Q2 |
| **Collaborative orchestration — operating `mode`** (chat / task / single-turn; formerly the Task-based Agent Collaboration API) | A coordinator delegates with an explicit mode: **chat** (full user interaction), **task** (interaction for clarifications), **single-turn** (no interaction, parallel). | A reified **`mode` field** on the coordinator `colony`: chat/task ⇒ a `hypha` awaiting external input (human-in-the-loop, never-silent); single-turn ⇒ a `ParallelAgent`-style RT1 fork/join. The mode is inspectable, never implicit. | RT1/RT3/RT4; §3.5; G2 |
| **`RunConfig.max_llm_calls`** (model-call budget) | Caps total model calls per run. | A never-silent **effect budget** on the colony (M-353): overrun ⇒ explicit **`TaskOutcome::BudgetExhausted`**, never a silent truncation. | `crates/mycelium-interp/src/supervise.rs` (`TaskOutcome::BudgetExhausted`); M-353 |

**The load-bearing line.** ADK's **LLM-agent vs workflow-agent** split is exactly Mycelium's
**RT3-nondeterministic vs RT2-deterministic** boundary. Workflow agents (Sequential/Parallel/Loop)
are deterministic orchestration → RT2; LLM reasoning and `transfer_to_agent` routing are
nondeterministic → RT3 (reified, named, `EXPLAIN`-able). Every honesty property in §6 follows from
keeping that line bright.

---

## 4. The phylum surface (Mycelium syntax)

The `adk` phylum's `nodule`s, sketched in Mycelium surface syntax (per
`.claude/memory/lang-lexicon-syntax.md:117,295-323`). Each follows the RFC-0016 §4.1 per-op
contract (C1 never-silent · C2 honest tag · C3 EXPLAIN-able · C4 content-addressed value-semantics ·
C6 declared/bounded effects) and ships a §4.5 guarantee matrix.

> **These sketches are a TARGET, not today-compilable.** Generic signatures (`Tool<In,Out>`,
> `List<Event>`) need E7-1/M-657; `colony`/`hypha`/`graft`/`fuse` need E7-2/M-666-667 and the
> `consume`/`graft` surface (M-664). The Rust-first crate (§7) implements the *semantics* now; the
> surface lands as E7-1/E7-2 complete. Every dependency is FLAGGed inline.

### 4.1 `adk.tool` — typed tools with declared effects

```mycelium
// nodule: adk.tool
nodule adk.tool

// Every tool failure is an explicit, named error — never a silent None/empty (C1).
type ToolError = BadArgs(Text) | OutOfDomain(Text) | Refused(Text) | Upstream(Text)

// RATIFIED (2026-06-21, RP-9): budget is NOT a ToolError arm — a tool's budget overrun surfaces on the
//   task channel as RunError::BudgetExhausted / TaskOutcome::BudgetExhausted (the per-task effect
//   budget, M-353; supervise.rs). obl.3 map: TypeMismatch->BadArgs, Exec->Upstream,
//   Budget->TaskOutcome::BudgetExhausted. The branded names above are canonical (generic names = aliases).

// FLAG (E7-1 / M-657): `Tool<In, Out>` needs generics — not in the language yet
//   (issues.yaml:1879, 2066-2074). Until then, tools are concrete-typed.
// FLAG (E7-1 / M-664): `graft` / `consume` surface keywords land with the runtime vocab.

// A pure tool: explicit Result, declared-empty effects, honest tag (C1/C2/C6).
fn run_pure(args: In) -> Result<Out @ Declared, ToolError> = …

// An external-infra tool: a `graft` over an affine Substrate, consumed exactly once, io effect.
graft run_io(cap: Substrate{Net}, args: In) -> Result<Out @ Empirical, ToolError>
//     ^ affine handle consumed once (LR-8); io effect declared (C6 / M-660); RT4 partial-fail explicit

// The model-facing call-schema is DERIVED from In/Out — never hand-written (no drift, C4).
//   (Mirrors ADK build_function_declaration, but type-checked, not docstring-driven.)
fn schema_of() -> ToolSchema = derive_schema::<In, Out>()        // needs generics (M-657)
```

### 4.2 `adk.agent` — agent definitions

```mycelium
// nodule: adk.agent
nodule adk.agent

// An agent's static definition. FLAG (E7-1): the generic tool list + the `Agent`/`Tool` trait
//   surface need generics (M-657) + traits/`impl` (M-659).
type Instruction = Static(Text) | Dynamic(/* state -> Text, an InstructionProvider */)

// An LLM agent: the model decides flow (RT3). Its run is a colony (adk.runner).
type LlmAgent = LlmAgent(
  name:        Text,
  model:       ModelRef,                  // resolves via adk.model (§4.5)
  instruction: Instruction,
  description: Text,                       // used by a coordinator's routing policy (RT3)
  tools:       List<ToolRef>,              // needs List<_> (generics, M-657)
  sub_agents:  List<AgentRef>,
  output_key:  Option<Text>               // auto-store final text into State[output_key]
)

// Workflow agents: deterministic orchestration (RT2) — code decides flow, not the model.
type Workflow =
    Sequential(List<AgentRef>)            // ordered hypha chain
  | Parallel(List<AgentRef>)              // colony fork/join (RT2 fragment)
  | Loop(body: List<AgentRef>, max_iterations: Nat)   // BOUNDED only — Nat cap is mandatory
//                                         ^ FLAG: an unbounded ADK loop has NO mapping (lexicon:160-163)

type Agent = Llm(LlmAgent) | Flow(Workflow)
```

### 4.3 `adk.session` — content-addressed state

```mycelium
// nodule: adk.session
nodule adk.session

// State and Event are immutable, content-addressed values (RFC-0016:139; content/src/lib.rs:1-7).
type State   = State(/* content-addressed key->value snapshot; prefix scoping app|user|temp|session */)
type Event   = Event(/* one content-addressed log entry: author, content, actions */)
type Session = Session(state: State, events: List<Event>)        // List needs generics (M-657)

// A "mutation" returns a NEW snapshot value that MOVES (RT1, RFC-0008:91-97). No in-place write.
fn put(s: State, key: Text, v: Value) -> State = …               // value-semantic (C4)

// Append to the event log (append-only, content-addressed). Total, Exact, effect-free.
fn append(events: List<Event>, e: Event) -> List<Event> = …

// FLAG — immutability tension (RFC-0016:92-94 / RT1 vs ADK's mutable prefix-scoped State):
//   v0 = state-as-snapshots above (ADK's transactional _delta is itself snapshot-like, easing this).
//   Concurrent sub-agent merge = `fuse` (RT6, RFC-0008:135-145): payload joins up / guarantee meets
//   down; a non-lawful merge is an EXPLICIT conflict surfaced to a policy (RT3), never a silent
//   overwrite. `fuse` is Ratified-not-yet-lexed (lexicon:145) -> E7-2/M-667. Flag, do not paper over.
//
// RATIFIED (2026-06-21, RP-9 / R23-Q2): v1 = the snapshot model above; concurrent sub-agent State
//   merge is deferred to `fuse`. v1 never silently overwrites — it serializes commits or surfaces an
//   explicit conflict.
```

### 4.4 `adk.runner` — the colony scheduler

```mycelium
// nodule: adk.runner
nodule adk.runner

// The runner outcome stream: values move over the Event channel (RT1). A run yields Events and
// persists snapshots (ADK Runner persists-before-yield).
type RunError = AgentFailed(Text) | BudgetExhausted | Cancelled    // mirrors TaskOutcome (supervise.rs:94-103)

// FLAG (E7-2 / M-666): the `colony { … }` / `hypha <expr>` L1 constructs are not yet active
//   (lexed, but no parser production — token.rs:34,42 reserved-not-active). The Rust-first runner
//   calls mycelium_mlir::runtime / mycelium_std_runtime::colony directly (§7).
//
// RATIFIED (2026-06-21, RP-9 / R23-Q1): v1 builds on `mycelium_mlir::runtime` — the only landed runtime
//   with typed value-returning hyphae + never-silent TaskOutcome (incl. BudgetExhausted) + the RT2
//   differential; migrate to the std.runtime facade (ADR-020) when it grows value-returning tasks.

// Run an agent against a session. The agent's reason->act->observe loop is a `hypha`; the runner is
// the structured-concurrency scope (RT7 no-orphan join). Partial failure is explicit (RT4).
fn run(agent: Agent, session: Session, message: Content)
    -> Result<(Session, List<Event>), RunError> =
  colony {
    // ordered/forked hypha per the agent kind; each yields a typed outcome, never a silent variant
    hypha step(agent, session, message)
  }
//  ^ TARGET syntax (M-666). RT7: the colony cannot exit while a child hypha is in flight.
```

### 4.5 `adk.model` — wired to the LLM harness

```mycelium
// nodule: adk.model
nodule adk.model

// Reuses the harness GrokLlmReport/GrokOutcome schema (crates/mycelium-bench/src/llm.rs:243-258,356-383).
type ModelError = MissingKey | ModelUnavailable | SpendCapped(Text) | Decode(Text)

// model_allowed_tags = {Declared, Empirical} ONLY — never Proven/Exact (llm.rs:63-64, 570).
// The endpoint is external infra -> a `graft` over an affine Substrate; declares io + a USD Budget.
graft generate(
    cap:    Substrate{Xai},        // live arm: xAI/Grok, OpenAI-compatible (README.md:94-101)
    req:    LlmRequest,
    budget: UsdBudget              // never-silent spend gate (README.md:97-101) — best-effort => Declared
) -> Result<LlmOutcome @ Empirical, ModelError> = …
//          ^ tag verbatim-preserved from the harness, never upgraded (llm.rs:367-368, 507-508)

// Missing key / model => never-silent Err, NEVER a fabricated answer (README.md:48-49,476; C1).
// Synthetic / self-test runs stay flagged and are never presented as real quality (llm.rs:410-419).
fn is_synthetic(o: LlmOutcome) -> Bool = …

// Local arm: harness.py (llama.cpp, default qwen2.5-coder-1.5b) — same interface, a different cap.
graft generate_local(cap: Substrate{LlamaCpp}, req: LlmRequest)
    -> Result<LlmOutcome @ Empirical, ModelError> = …
```

---

## 5. Harness wiring (the model layer's real-call substrate)

The `adk.model` layer does **not** invent an LLM client — it wires to the **existing** harness, so
the port inherits the harness's honesty machinery for free.

- **Live arm — xAI/Grok.** `tools/llm-harness/grok/` is an OpenAI-compatible client against
  `https://api.x.ai/v1` (`README.md:94-101`) with a **never-silent USD spend gate** (`--max-usd`,
  default $10; a unit whose *estimated* cost would breach the cap is **refused before it is sent**;
  the run stops with a partial, honestly-flagged report — `README.md:97-101,119-123`). The gate is
  **best-effort, not a formal bound** (`README.md:99-101`), so any cost guarantee `adk.model`
  surfaces is **`Declared`**, never `Proven`. ADK's `LiteLlm` (OpenAI-compatible providers, incl.
  Anthropic/xAI) is the conceptual analogue this arm fills.
- **Local arm — llama.cpp.** `harness.py` runs a local GGUF (default `qwen2.5-coder-1.5b`,
  `README.md:153-156`), or a mock mode for offline/CI. Same `adk.model` interface, a different
  `Substrate` cap.
- **Result schema — reuse `GrokLlmReport`.** The model response reuses the harness ingestion schema
  (`crates/mycelium-bench/src/llm.rs:243-258`): `parse_any_llm_json` dispatches Grok-vs-bench on a
  root `"metadata"` key (`:445-456,485-551`); per-outcome `guarantee_tag` is **preserved verbatim,
  never upgraded** (`:367-368,507-508`); `model_allowed_tags=["Declared","Empirical"]` (`:63-64`);
  `is_synthetic` flags mock/self-test runs so they are never presented as real quality
  (`:146-152,410-419`); malformed input is a **loud parse error** (`deny_unknown_fields`,
  `:240-242`), never a silent drop (G2).
- **Never-silent absence.** A missing key/model is an explicit `SKIP`/`Err`, never a fabricated
  answer (`README.md:48-49,476`; C1 `RFC-0016:82-84`).

This is the concrete mechanism by which the honesty model (§6) is *enforced in code*, not merely
asserted in prose: the substrate the model layer stands on already refuses to upgrade a tag, drop
an error, or present a mock as real.

---

## 6. The honesty model — the differentiator

The decisive case for the port: Mycelium's substrate makes *silence unrepresentable* where Python
ADK (by virtue of its Python substrate) is epistemically silent. The differentiator is **not** a
claim that LLMs are reliable — it is that **the framework never lets an unreliable LLM output
masquerade as reliable**. Five parts; each Mycelium claim is cited.

### 6.1 LLM outputs are `Declared`/`Empirical`, never `Proven` (VR-5)

In Python ADK an LLM response is an untyped `str`/`dict`: a hallucinated citation and a checked
theorem are the *same type*. In Mycelium every model output value carries a guarantee tag, and the
substrate **type-forbids** tagging a model output `Proven`/`Exact` — `model_allowed_tags =
["Declared","Empirical"]` (`llm.rs:63-64`), and the tag is **preserved verbatim, never upgraded**
(`llm.rs:367-368,507-508`; CLAUDE.md honesty rule). `Proven`/`Exact` denote a *checked* basis, which
an LLM has none of by construction. **An agent framework on this substrate cannot silently launder a
hallucination into a fact.**

### 6.2 Tool calls are never-silent (`Result`)

Python ADK tools may return `None`, raise, or return garbage, and the loop proceeds. In Mycelium a
tool is `fn -> Result<Out, ToolError>`: a failed/out-of-range/refused call is an **explicit `Err`**
the loop must branch on (CLAUDE.md #2 — "out-of-range is an explicit `Option`/error"; RT4
`RFC-0008:118-126` — "partial failure is explicit … no construct may erase that"; the composition
contract C3 `:276-280` — "exactly one explicit `TaskOutcome` … no silent/dropped variant"). **The
model loop cannot pretend the tool succeeded.**

### 6.3 Non-determinism is reified + `EXPLAIN`-able

The two nondeterministic hinges — the model call and `transfer_to_agent` routing — are opaque in
Python ADK. In Mycelium they are **RT3** constructs: "every departure from RT2's fragment … is an
explicit construct whose decision procedure is a content-addressed RFC-0005 policy with **mandatory
EXPLAIN**" (`RFC-0008:109-116`), and where a guarantee is genuinely probabilistic it is **tagged on
the lattice** ("suspicion with confidence, never `Exact`", RT5 `:127-133`). "Why did the agent route
here / call this tool" is an **inspectable record**, not an opaque whim (CLAUDE.md #2 — no black
boxes).

### 6.4 The empirical honesty floor (no overclaim)

This part keeps the section honest about *itself*. The measured floor (DN-09 §9.2/§9.3): in the
M-381 ablation (8 tasks × 3 seeds), **arm1 bare novel surface = 0.0%**, **arm2 +grammar-in-context
primer = 100.0%** [Empirical] (`DN-09:311-312,326-329`); arm3 (constrained decoding) **blocked**;
arm4 (LlmCanonical) 0% is a **scoring artifact, not a model failure** (`DN-09:331-338`, Declared);
the **retention ratio is INDETERMINATE** and the leverage claim stays **`Declared`/open**
(`DN-09:439-442`; carved out → RP-1/M-381 per `RFC-0021`). Mycelium does **not** claim LLMs
reliably author Mycelium-ADK code. The differentiator and the guardrail are the same mechanism:
§6.1–§6.3 hold **regardless of model quality**, because they constrain the substrate, not the model.

### 6.5 Synthetic ≠ real (the harness gate)

A mocked tool/stubbed model in Python is indistinguishable from the real thing to the surrounding
code. The Mycelium substrate refuses to present a fixture as real-model evidence: a run is marked
`mode:"mock"`/`status:"mock-PASS"` and **never presented as evidence of real model quality**
(`llm.rs:5-9`); `is_synthetic()` is the primary honesty gate (`:146-152,410-419`); `provenance()`
stamps `SYNTHETIC (fixture; not real model quality)` (`:155-175`). **The port inherits: a
simulated/mocked agent run is never reported as a real one.**

### 6.6 Contrast table

| Concern | Python ADK (silent) | Mycelium-ADK (honest) | Grounding |
|---|---|---|---|
| LLM output's epistemic status | Untyped `str`/`dict`; fact = hallucination | Tagged; model output type-forbidden from `Proven`/`Exact`; tag never upgraded | `llm.rs:63-64,367-368`; CLAUDE.md #1 |
| Tool-call failure | `None`/raise may be swallowed; loop proceeds | Explicit `Result`; `Err` must be handled, never read as success | CLAUDE.md #2; `RFC-0008:118-126,276-280` |
| "Why did it route / call this?" | Opaque model whim; no record | Reified RFC-0005 policy with mandatory `EXPLAIN` | `RFC-0008:109-116` |
| Probabilistic guarantees | Implicit / as certainty | Per-op claim, suspicion-with-confidence, never `Exact` | `RFC-0008:127-133` |
| Malformed data at a boundary | Often coerced/dropped | Loud parse error; nothing silently dropped (G2) | `llm.rs:177-180,240-242` |
| LLM code-gen reliability | Implied by demos | Stated: bare 0%, primer 100% [Empirical], retention INDETERMINATE; `Declared`/open | `DN-09:311-338,439-442` |
| Mocked vs real run | Indistinguishable | Marked SYNTHETIC end-to-end; never reported as real | `llm.rs:5-9,146-152,410-419` |

### 6.7 What we do NOT claim

We do **not** claim LLMs reliably author Mycelium-ADK code, nor that the port makes models smarter,
nor that non-determinism is *removed* (model calls + delegation remain nondeterministic —
`RFC-0008:109-116`). The measured leverage is *weak-but-recoverable* and stays `Declared`/open
(DN-09 §9; RP-1/RP-9). The single, honest claim is structural: the substrate **never lets an
unreliable model output masquerade as reliable** — outputs cannot be tagged `Proven`/`Exact`, tool
failures cannot be swallowed, selections cannot be opaque, a mock cannot be reported as real. That
holds whatever the model's competence — which is exactly why it is the case for the port existing.

---

## 7. Rust-first reality + the E7-1 / E7-2 dependencies

**No `mycelium-adk` crate exists** (greenfield, verified). Self-hosting is blocked, so the port is
**Rust-first**: a `mycelium-adk` Rust crate over the already-landed runtime + harness, following the
same convention every landed `mycelium-std-*` crate uses — a per-op **guarantee-matrix-as-data**
asserted in tests (RFC-0016 §4.5) and `#![forbid(unsafe_code)]` (e.g.
`crates/mycelium-std-content/src/lib.rs:1-30`). The Mycelium-language surface (§4) is a **target**
that lands as the dependencies below complete.

### 7.1 What already exists (the Rust-first foundation)

- **Two runtime realizations** (the colony scheduler): `mycelium-std-runtime::colony::Scope::join_all`
  (FIFO, **Exact** sweep, ADR-020 §4 — `colony.rs:103`) and `mycelium-mlir::runtime`
  (`Scope::run_sequential`/`run_interleaved`/`run_dataflow`; `Colony` = type alias — `runtime.rs:106`).
  **The port must pick one deliberately** (FLAG — they are distinct; RP-9 decides).
- **`TaskOutcome`** — the never-silent C3 channel, exactly `Done`/`Failed`/`BudgetExhausted`/`Cancelled`
  (`supervise.rs:94-103`); plus `Supervisor`/`RestartIntensity` (M-356) for `reclaim`-style
  supervision.
- **Typed SPSC channels** (the Event stream) — `mycelium-mlir::channel` (finite capacity, explicit
  backpressure, explicit close — G2). **Multi-source `select`/`merge` is deferred (RT3)** — so an
  ADK "first-tool-to-respond-wins" race has **no deterministic mapping** and must also be an RT3
  reified policy.
- **The LLM harness** — the model layer's substrate (§5).

### 7.2 E7-1 — the `Tool<In,Out>` / `Agent` surface dependency

`tools/github/issues.yaml:2041-2053`. The typed agent/tool surface needs: **generics** (M-657 —
`Tool<In,Out>`, `List<Event>`, schema-derivation over `In`), **traits + `impl`** (M-659 — the
`Agent`/`Tool` abstractions), **effect annotations** (M-660 — the declared-effect column on a `fn`),
and the **`consume`/`graft`/`impl` surface** (M-664). Generics are **not yet in the language**
(`issues.yaml:1879`). **Every generic signature in §4 is a target, not compilable today.**

### 7.3 E7-2 — the `colony`/`hypha` runtime-surface dependency

`tools/github/issues.yaml:2146-2158`. **Correction to the common assumption (verified against
source):** the lexer reservation has **already landed** — all 10 runtime terms **and** `hypha`/`colony`
are in `keyword()`/`Tok` (`token.rs:29-60,216-229`), so **M-665 is done**. The remaining gap is the
**parser construct (M-666)**: no production consumes these tokens, so `colony { … }`/`hypha <expr>`
yield an explicit "reserved … not yet active" parse error. After M-666: `fuse`/`reclaim`/`tier`
(M-667), then the **RT3 vocabulary for delegation/racing** (M-668 — which the coordinator routing and
tool-racing mappings need). **The §4.4 `colony`/`hypha` surface and the RT3 delegation construct are
the E7-2 dependency.**

### 7.4 Honest status statement

This RFC is a **design against an Accepted-but-not-fully-Enacted runtime** (RFC-0008 is Accepted; its
R1 surface lands with M-666/M-667; RT3 forms are not in R1 scope). The honest sequencing: ship the
Rust-first `mycelium-adk` crate over the landed foundation (§7.1) with honest tags + a guarantee
matrix; adopt the Mycelium-language surface (§4) per-construct as E7-1/E7-2 land; keep any ADK-shaped
LLM-leverage claim `Declared`/open (DN-09). The spec moves **"implemented (Rust-first), pending
ratification"**, never silently to Accepted.

---

## 8. Example programs

Illustrative `.myc` (TARGET surface — see §4/§7 FLAGs). They show the *honest shape*, not
today-compilable code.

### 8.1 A single weather agent with one tool

```mycelium
// nodule: examples.weather
nodule examples.weather
use adk.agent
use adk.tool

// A tool that calls a weather API: a `graft` over an affine Net substrate, honest Empirical tag,
// explicit error set (C1/C2). The model-facing schema is derived from (Text -> Report).
type Report = Report(tempC: Int, summary: Text)
graft get_weather(cap: Substrate{Net}, city: Text) -> Result<Report @ Empirical, ToolError> = …

fn weather_agent() -> Agent =
  Llm(LlmAgent(
    name:        "weather",
    model:       model_ref("grok"),         // resolves via adk.model -> the harness live arm
    instruction: Static("Answer weather questions. Call get_weather; never guess."),
    description: "Answers weather questions for a city.",
    tools:       [tool_ref(get_weather)],   // List<_> needs M-657
    sub_agents:  [],
    output_key:  Some("weather_answer")
  ))
// Honesty: the model's answer carries a Declared/Empirical tag (never Proven); a failed get_weather
// is an explicit Err the runner must handle (never a silent gap the model papers over).
```

### 8.2 A sequential pipeline (deterministic, RT2)

```mycelium
// nodule: examples.pipeline
nodule examples.pipeline
use adk.agent

// SequentialAgent -> an ordered hypha chain (RT2 deterministic). Each step reads the prior step's
// output_key from the threaded State snapshot.
fn research_pipeline() -> Agent =
  Flow(Sequential([
    agent_ref(gather_sources),     // writes State["sources"]
    agent_ref(draft_summary),      // reads {sources}, writes State["draft"]
    agent_ref(fact_check)          // reads {draft}, writes State["checked"]
  ]))
// Determinism is Empirical (the RT2 sequentialization differential is the evidence), NEVER Proven
// (no mechanized proof in-repo; VR-5 — RFC-0008 M-357 changelog).
```

### 8.3 A coordinator with LLM-driven delegation (nondeterministic, RT3)

```mycelium
// nodule: examples.support
nodule examples.support
use adk.agent

// A coordinator routes to a specialist. The ROUTING DECISION is the model's -> an RT3 construct with
// a named, EXPLAIN-able policy. The choice is Declared/Empirical, NEVER Proven (VR-5).
fn support_coordinator() -> Agent =
  Llm(LlmAgent(
    name:        "support",
    model:       model_ref("grok"),
    instruction: Static("Route the user to the right specialist by their description."),
    description: "Front-desk router.",
    tools:       [],
    sub_agents:  [agent_ref(billing_agent), agent_ref(tech_agent)],  // transfer_to_agent targets
    output_key:  None
  ))
// vs an AgentTool (agent-as-tool): the coordinator would CALL a specialist and RETAIN control — an
// ordinary typed tool call, not the RT3 transfer above. The two control-transfer models stay distinct.
// EXPLAIN("why route to billing?") yields the reified routing record, not an opaque model whim.
```

---

## 9. Drawbacks

- **It is a design against unbuilt foundations.** The Mycelium-language surface needs E7-1 (generics/
  traits) and E7-2 (the `colony`/`hypha` constructs + the RT3 vocabulary). Until then the port is
  Rust-first; the §4 sketches are aspirational. (Mitigation: ship Rust-first; adopt the surface
  per-construct as the dependencies land.)
- **The State immutability tension is real.** ADK's mutable prefix-scoped scratchpad has no 1:1
  immutable analogue; the snapshot model is the honest v0, with `fuse` as the principled concurrent
  story (future). This is friction the Python original does not have.
- **The honesty contract raises per-tool cost.** Every tool carries a tag, an explicit error set,
  declared effects, and a matrix row (RFC-0016 §5 notes this cost). The payoff is §6.
- **No measured LLM-leverage for ADK-shaped generation.** The leverage claim is `Declared`/open
  (DN-09); the port cannot promise an LLM will author its agents well. (This is honest, not a defect
  — §6.4/§6.7.)

## 10. Unresolved questions (gate ratification — RP-9)

The follow-up deep pass (**RP-9**, §11 / `docs/notes/research-prompts.md`) must address, before this
moves to Accepted:

- **R23-Q1 — Scaffold/implement `mycelium-adk` (Rust-first).** Stand up the crate over the landed
  runtime (§7.1) + harness (§5) with honest tags + a guarantee matrix; pick **one** runtime
  realization (§7.1 FLAG) and justify it.
- **R23-Q2 — The State immutability resolution.** Confirm the snapshot model as v0 and specify the
  `fuse`-merge concurrent story precisely (RT6 laws; the explicit-conflict-not-overwrite rule).
- **R23-Q3 — The RT3 delegation/racing constructs.** Specify the reified routing/racing policy
  (the `transfer_to_agent` and tool-race mappings) against M-668; confirm the `EXPLAIN` record shape.
- **R23-Q4 — ADK-shaped LLM-leverage (never pre-written).** Scope (do not assert) a measured
  question for LLM generation of `adk`-phylum programs, inheriting DN-09/RP-1's protocol and open
  posture (VR-5). The verdict arrives by a later supersession, not by gating this RFC.
- **R23-Q5 — ADK API drift.** Re-verify the §3 concept-map against a **pinned** `adk-python` release
  (the docs site now reads "ADK 2.0" with newer graph/router concepts — §11); pin the target version.

## 11. Honest-Uncertainty Register

- **The concept-map's fidelity is `Empirical`/`Declared`, not `Proven`.** §3 was captured from public
  ADK docs/source on `main` at one point in time. **Could-not-verify / version-drift items** (from
  the research pass): the docs site `google.github.io/adk-docs` now 301-redirects to `adk.dev`
  ("ADK 2.0"), which adds **graph/dynamic workflows, an experimental Agent-Routing/router, and an
  expanded `Context`** that may post-date a pinned `adk-python` target; **`ToolContext` lineage moved**
  (now `= Context` in `agents/context.py`; earlier it extended `CallbackContext`) though its
  capabilities are stable; the **built-in/third-party tool catalog** (Google Search, code-exec,
  OpenAPI/LangChain wrappers) was not fetched from a primary page; **`LongRunningFunctionTool`'s exact
  signature**, the **`MemoryService` concrete classes**, **`LlmRequest`'s fields**, and
  **`RunConfig`'s fields** were not source-verified. RP-9/R23-Q5 must re-verify against a pinned
  release.
- **The whole §4 surface is a TARGET, not compilable today.** Generics (E7-1/M-657), traits/`impl`
  (M-659), effects (M-660), and the `colony`/`hypha`/`fuse`/`graft` constructs (E7-2/M-666-667, M-664)
  are not yet in the language. The Rust-first semantics are available now; the surface sugar is future.
- **The packet contained a stale claim that direct source refuted (§7.3).** The E7-2 gap is the
  *parser* (M-666), **not** the *lexer* — M-665 (the lexer reservation of all 10 runtime terms +
  `hypha`/`colony`) has landed (`token.rs:29-60,216-229`). RP-9 must re-verify the E7-1/E7-2 status
  against **source**, not the planning artifacts.
- **Two runtime realizations exist; the port must choose (§7.1).** `mycelium-std-runtime::colony`
  vs `mycelium-mlir::runtime` are distinct; this RFC does not pick — RP-9/R23-Q1 does.
- **The State immutability tension is genuinely unresolved at the language level.** Flagged in §4.3/§9;
  the snapshot model is the honest v0.
- **No ADK-shaped LLM-leverage is measured.** It inherits DN-09 §9's INDETERMINATE retention ratio;
  R23-Q4 scopes the question without pre-writing a verdict (VR-5).
- **Determinism of the RT2 executor is `Empirical`, not `Proven`** (the sequentialization differential
  is the evidence — RFC-0008 M-357 changelog). A `ParallelAgent`'s "deterministic merge" claim must
  cite the differential, never `Proven`.

---

## Meta — changelog

- **2026-06-21 — RATIFIED → Accepted (maintainer).** The one open completeness item is **closed**: §3
  repaired with new **§3.7** mapping ADK 2.0's graph **Workflow Runtime** (a `colony` DAG with per-node
  RT2/RT3 tags), routing/fan-out-fan-in/loops/retry/nested, dynamic nodes, human-in-the-loop →
  `cyst` checkpoint, the collaborative operating-`mode` (chat/task/single-turn), and
  `RunConfig.max_llm_calls` → `TaskOutcome::BudgetExhausted`; §3 pinned to `adk-python` **v2.3.0**.
  Ratified design decisions (recorded inline): `adk.runner` on **`mycelium-mlir::runtime`** (R23-Q1,
  migrate to the std.runtime facade later); **`ToolError`** keeps `BadArgs|OutOfDomain|Refused|Upstream`
  with budget on **`TaskOutcome::BudgetExhausted`** (branded names canonical); **Session** snapshot-v0,
  concurrent merge deferred to **`fuse`** (R23-Q2, never silent-overwrite); **LLM-leverage** stays
  **no-verdict** (separate measured pass). **Accepted = design agreed; Enacted gated on the
  `mycelium-adk` build + E7-1/E7-2.** Append-only; trail preserved in the Status cell. Empirical/Declared,
  never `Proven` (VR-5).
- **2026-06-21 — RP-9 research gate substantially discharged (Phase-2 deep-research follow-up; `dfr`
  session).** Four fractured Opus sub-reasoners (A1 concept-map · A2 honesty-differentiator · A3
  tool-dispatch never-silent · A4 session/runner + harness reuse) verified §11 against primary ADK
  source (pinned `adk-python` v2.3.0) + landed in-repo substrate (`research/13-adk-phylum-RECORD.md`
  §6). **Design soundness confirmed, no soundness falsification**; register drift item 1 +
  parser-not-lexer item 3 resolved; item 7 (RT2 `Empirical`) confirmed; items 2/5 deferred; item 6
  (LLM-leverage) scoped with **no verdict** (DN-09 INDETERMINATE). **One open completeness item** — the
  ADK-2.0 concept-map repair (graph/Workflow + 3 rows) — plus a `ToolError` budget-arm constraint and
  the runtime-choice decision, all carried forward, none silently closed (G2). Status appended:
  **"RP-9 research gate substantially discharged; concept-map completeness open; pending maintainer
  ratification"** (ratification still requires the build + E7-1/E7-2). Findings Empirical/Declared,
  never `Proven` (VR-5). Append-only; no design content rewritten.
- **2026-06-21 — Created (Draft).** Research-grounded design from `research/13-adk-phylum-RECORD.md`
  (fractured-methodology pass; four Opus sub-reasoners over one cross-context packet). Captures the
  ADK→Mycelium concept-map (§3, source-cited), the `adk` phylum surface (§4, Mycelium syntax), the
  harness wiring (§5), the honesty-as-differentiator model (§6), the Rust-first reality + the
  E7-1/E7-2 dependency chain (§7), three example programs (§8), and the Honest-Uncertainty Register
  (§11). Records the §7.3 correction (E7-2 gap = parser, not lexer; M-665 landed). Asserts **no**
  LLM-capability result (VR-5); the LLM-leverage claim stays `Declared`/open (DN-09). Stays **Draft**
  — the RP-9 deep pass (§10) gates ratification. No kernel change (KC-3).
