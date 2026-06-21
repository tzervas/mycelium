//! `adk.agent` — pure data: agent definitions, id newtypes, workflows (RFC-0023 §4.2).
//!
//! This module is **pure data** — no execution, no effects.  The agent tree is a
//! content-addressed value; running it is the runner's job (see `adk.runner`).
//!
//! ## Types
//! - [`ModelRef`] — opaque, printable model identifier
//! - [`ToolRef`] — opaque, printable tool identifier
//! - [`AgentRef`] — opaque, printable agent identifier
//! - [`Instruction`] — `Static(String)` | `Dynamic(placeholder)`
//! - [`LlmAgent`] — fields: `name`, `model`, `instruction`, `description`, `tools`,
//!   `sub_agents`, `output_key`
//! - [`Workflow`] — `Sequential` | `Parallel` | `Loop { body, max }`
//! - [`Agent`] — `Llm(LlmAgent)` | `Flow(Workflow)`
//!
//! ## FLAGs
//! - **FLAG (E7-1 / M-657):** `tools: Vec<ToolRef>` and `sub_agents: Vec<AgentRef>`
//!   use `Vec<_>` here; the Mycelium-language surface needs `List<_>` (generics, M-657).
//! - **FLAG (E7-1 / M-659):** The `Agent`/`Tool` *trait* abstractions (not the `Agent`
//!   *data type* here) need traits+`impl` (M-659).
//! - **FLAG (RFC-0023 §4.2):** `Instruction::Dynamic` is a placeholder — the
//!   `(state -> Text)` function-value surface needs the effects system (M-660) and
//!   closures.  The variant exists to make the shape explicit; a `Dynamic` instruction
//!   cannot be evaluated until that surface lands.
//!
//! ## Honesty (VR-5)
//! - No execution here.  All types are `Debug + Clone + PartialEq` (value-semantic).
//! - `Workflow::Loop` requires a `max: usize` cap — unbounded loops have no mapping
//!   (RFC-0023 §4.2: "Nat cap is mandatory").
//! - `AgentRef`/`ToolRef`/`ModelRef` are simple id newtypes — no resolution/lookup
//!   (that is the runner's job).
//!
//! ## Design spec
//! `docs/rfcs/RFC-0023-Agent-Development-Kit-Phylum.md` §4.2

use std::fmt;

// ── Id newtypes ───────────────────────────────────────────────────────────────

/// An opaque, printable model identifier (e.g. `"grok"`, `"claude-3-5-sonnet"`).
///
/// Resolution from string to a backend is the model layer's job (`adk.model`).
/// A no-match is never-silent `Err` (C1 / RFC-0023 §3.6).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelRef(pub String);

impl ModelRef {
    /// Construct a `ModelRef` from a string id.
    pub fn new(id: impl Into<String>) -> Self {
        ModelRef(id.into())
    }
}

impl fmt::Display for ModelRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "model:{}", self.0)
    }
}

/// An opaque, printable tool identifier (resolves to a `Tool` impl at runtime).
///
/// A no-match during resolution is never-silent `Err` (C1).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ToolRef(pub String);

impl ToolRef {
    /// Construct a `ToolRef` from a string id.
    pub fn new(id: impl Into<String>) -> Self {
        ToolRef(id.into())
    }
}

impl fmt::Display for ToolRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tool:{}", self.0)
    }
}

/// An opaque, printable agent identifier (resolves to an `Agent` value at runtime).
///
/// A no-match during resolution is never-silent `Err` (C1).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentRef(pub String);

impl AgentRef {
    /// Construct an `AgentRef` from a string id.
    pub fn new(id: impl Into<String>) -> Self {
        AgentRef(id.into())
    }
}

impl fmt::Display for AgentRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "agent:{}", self.0)
    }
}

// ── Instruction ───────────────────────────────────────────────────────────────

/// The system instruction for an LLM agent (RFC-0023 §4.2).
///
/// - `Static` — a fixed string given at construction time.
/// - `Dynamic` — a **placeholder** for a `(state -> Text)` provider.
///
/// ## FLAG (E7-1 / M-660 — effects)
/// `Dynamic` cannot be evaluated yet: the function-value / closure surface (M-660)
/// and the effects system are not yet active.  The variant is present to make the
/// *design shape* explicit and compilable; the runner must refuse to evaluate a
/// `Dynamic` instruction until M-660 lands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    /// A fixed instruction string known at agent-construction time.
    Static(String),

    /// A dynamic instruction whose text depends on the session state at invocation
    /// time.  The `String` field is a **documentation placeholder** (the name of the
    /// intended provider function); it cannot be called — FLAG-deferred (M-660).
    ///
    /// FLAG (E7-1 / M-660): actual function-value evaluation deferred.
    Dynamic(String),
}

// ── LlmAgent ─────────────────────────────────────────────────────────────────

/// A pure data description of an LLM-driven agent (RFC-0023 §4.2).
///
/// The model *dynamically decides* flow/tools/output at runtime (RT3 — nondeterministic,
/// reified, EXPLAIN-able).  This struct is *pure data* — no execution here.
///
/// ## Fields
/// - `name` — unique agent name (used in EXPLAIN records and routing decisions)
/// - `model` — resolved by `adk.model` at runtime; no-match is `Err`
/// - `instruction` — system prompt (static or dynamic-placeholder)
/// - `description` — used by a coordinator's routing policy (RT3) to pick this agent
/// - `tools` — tool refs available to this agent; resolved at runtime
/// - `sub_agents` — agent refs this agent may delegate to
/// - `output_key` — if `Some(k)`, the runner stores final text into `State[k]`
///
/// ## FLAG (E7-1 / M-657)
/// `tools` and `sub_agents` use `Vec<_>`; the Mycelium-language surface targets
/// `List<ToolRef>` / `List<AgentRef>` (needs generics M-657).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmAgent {
    /// Unique name identifying this agent in logs, routing, and EXPLAIN records.
    pub name: String,
    /// The model backend to use; resolved by `adk.model` at runtime.
    pub model: ModelRef,
    /// System instruction (static string or dynamic placeholder — FLAG M-660).
    pub instruction: Instruction,
    /// Human-readable description used by coordinator routing policies (RT3).
    pub description: String,
    /// Tool refs available to this agent; resolved at runtime.
    ///
    /// FLAG (E7-1 / M-657): Mycelium target surface is `List<ToolRef>`.
    pub tools: Vec<ToolRef>,
    /// Sub-agent refs this agent may delegate to (transfer_to_agent targets).
    ///
    /// FLAG (E7-1 / M-657): Mycelium target surface is `List<AgentRef>`.
    pub sub_agents: Vec<AgentRef>,
    /// If `Some(key)`, the runner stores the agent's final text into `State[key]`
    /// (ADK `output_key` / state threading pattern).
    pub output_key: Option<String>,
}

// ── Workflow ──────────────────────────────────────────────────────────────────

/// Deterministic workflow orchestration (RT2 — code decides flow; RFC-0023 §4.2).
///
/// Workflow agents are deterministic (RT2) — the code decides flow, not the model.
/// This is the ADK `SequentialAgent`/`ParallelAgent`/`LoopAgent` equivalent.
///
/// ## Variants
/// - `Sequential` — ordered hypha chain; each step runs after the prior completes.
/// - `Parallel` — colony fork/join (RT2 fragment); runs concurrently in isolated branches.
/// - `Loop` — bounded structural loop; **`max` cap is mandatory** (unbounded loops have
///   no deterministic mapping; RFC-0023 §4.2).
///
/// ## Honesty (VR-5)
/// The determinism of RT2 sequentialization is `Empirical` (the M-357 changelog is the
/// evidence), **not** `Proven` (no mechanized proof in-repo; RFC-0023 §11).
///
/// ## FLAG (E7-1 / M-657)
/// `Sequential` and `Parallel` bodies use `Vec<AgentRef>`; the Mycelium target surface
/// is `List<AgentRef>` (needs generics M-657).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Workflow {
    /// Run sub-agents in a fixed order, threading state via `output_key`.
    ///
    /// ADK `SequentialAgent` mapping: ordered hypha chain (RFC-0023 §3.1/§4.2, RT2).
    /// FLAG (E7-1 / M-657): target surface uses `List<AgentRef>`.
    Sequential(Vec<AgentRef>),

    /// Run sub-agents concurrently in isolated branches; results via distinct `output_key`s.
    ///
    /// ADK `ParallelAgent` mapping: colony fork/join (RFC-0023 §3.1/§4.2, RT1/RT2).
    /// Results are deterministic because branches are isolated (RT1 share-nothing).
    /// FLAG (E7-1 / M-657): target surface uses `List<AgentRef>`.
    Parallel(Vec<AgentRef>),

    /// Repeat `body` until `max` iterations are reached.
    ///
    /// ADK `LoopAgent` mapping (RFC-0023 §3.1/§4.2).  **`max` is mandatory** — an
    /// unbounded loop has no mapping in the Mycelium value model (structural recursion
    /// requires a termination witness; RFC-0023 §4.2 "FLAG: an unbounded ADK loop has
    /// NO mapping (lexicon:160-163)").
    /// FLAG (E7-1 / M-657): target surface uses `List<AgentRef>` for `body`.
    Loop {
        /// Sub-agents to repeat.
        body: Vec<AgentRef>,
        /// Maximum number of iterations.  This cap is **mandatory**; callers must
        /// provide a finite `max > 0`.  A runner may refuse `max == 0` with `Err`.
        max: usize,
    },
}

// ── Agent ─────────────────────────────────────────────────────────────────────

/// The top-level agent sum type (RFC-0023 §4.2).
///
/// An `Agent` is either:
/// - `Llm` — model-driven, nondeterministic (RT3); the model decides flow.
/// - `Flow` — deterministic workflow (RT2); the code decides flow.
///
/// This is **pure data** — the runner resolves refs and drives execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Agent {
    /// An LLM-driven agent.  Flow/tool/output are decided by the model (RT3 — reified,
    /// EXPLAIN-able, never opaque).
    Llm(LlmAgent),
    /// A deterministic workflow agent.  Flow is decided by code (RT2).
    Flow(Workflow),
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{Agent, AgentRef, Instruction, LlmAgent, ModelRef, ToolRef, Workflow};
    use proptest::prelude::*;

    fn simple_llm_agent() -> LlmAgent {
        LlmAgent {
            name: "test-agent".to_owned(),
            model: ModelRef::new("grok"),
            instruction: Instruction::Static("You are helpful.".to_owned()),
            description: "A test agent.".to_owned(),
            tools: vec![ToolRef::new("get_weather")],
            sub_agents: vec![],
            output_key: Some("answer".to_owned()),
        }
    }

    // ── Unit tests ─────────────────────────────────────────────────────────────

    /// `LlmAgent` is pure data — all fields are accessible after construction.
    #[test]
    fn llm_agent_is_pure_data() {
        let a = simple_llm_agent();
        assert_eq!(a.name, "test-agent");
        assert_eq!(a.model, ModelRef::new("grok"));
        assert_eq!(a.tools.len(), 1);
        assert_eq!(a.output_key, Some("answer".to_owned()));
    }

    /// `Agent::Llm` wraps an `LlmAgent`; `Agent::Flow` wraps a `Workflow`.
    #[test]
    fn agent_sum_type_is_correct() {
        let llm_agent = Agent::Llm(simple_llm_agent());
        let flow_agent = Agent::Flow(Workflow::Sequential(vec![AgentRef::new("step1")]));
        assert!(matches!(llm_agent, Agent::Llm(_)));
        assert!(matches!(flow_agent, Agent::Flow(_)));
    }

    /// `Workflow::Loop` requires an explicit `max` cap (no unbounded loops).
    #[test]
    fn workflow_loop_requires_max_cap() {
        let w = Workflow::Loop {
            body: vec![AgentRef::new("step")],
            max: 10,
        };
        match w {
            Workflow::Loop { max, .. } => assert_eq!(max, 10),
            _ => panic!("expected Loop variant"),
        }
    }

    /// `Instruction::Dynamic` is a placeholder — the string field documents the intent.
    #[test]
    fn instruction_dynamic_is_placeholder() {
        let instr = Instruction::Dynamic("state_to_prompt".to_owned());
        assert!(matches!(instr, Instruction::Dynamic(_)));
        // Note: Dynamic cannot be *evaluated* — it is a design-shape placeholder (FLAG M-660).
    }

    /// Id newtypes Display includes the prefix (for EXPLAIN records).
    #[test]
    fn id_newtypes_display_includes_prefix() {
        assert!(ModelRef::new("grok").to_string().contains("model:"));
        assert!(ToolRef::new("weather").to_string().contains("tool:"));
        assert!(AgentRef::new("billing").to_string().contains("agent:"));
    }

    /// `Agent` is `Clone` + `PartialEq` (value-semantic).
    #[test]
    fn agent_is_value_semantic_clone_and_eq() {
        let a = Agent::Llm(simple_llm_agent());
        let b = a.clone();
        assert_eq!(a, b);
    }

    /// `Workflow::Sequential` and `Parallel` hold the agent refs correctly.
    #[test]
    fn workflow_variants_hold_refs() {
        let refs = vec![AgentRef::new("a"), AgentRef::new("b")];
        let seq = Workflow::Sequential(refs.clone());
        let par = Workflow::Parallel(refs.clone());
        assert!(matches!(seq, Workflow::Sequential(v) if v.len() == 2));
        assert!(matches!(par, Workflow::Parallel(v) if v.len() == 2));
    }

    // ── Property tests ────────────────────────────────────────────────────────

    proptest! {
        /// BOUND: `ModelRef`/`ToolRef`/`AgentRef` preserve their id string (no truncation
        /// or transformation).  Guard: any truncation or modification breaks this.
        #[test]
        fn prop_id_newtypes_preserve_id(id in "[a-z][a-z0-9-]{0,30}") {
            let m = ModelRef::new(id.clone());
            let t = ToolRef::new(id.clone());
            let a = AgentRef::new(id.clone());
            prop_assert_eq!(&m.0, &id);
            prop_assert_eq!(&t.0, &id);
            prop_assert_eq!(&a.0, &id);
        }

        /// BOUND: `Workflow::Loop` always carries the provided `max` cap unchanged.
        #[test]
        fn prop_loop_max_is_preserved(max in 1usize..=1000) {
            let w = Workflow::Loop {
                body: vec![AgentRef::new("step")],
                max,
            };
            match w {
                Workflow::Loop { max: stored, .. } => prop_assert_eq!(stored, max),
                _ => prop_assert!(false, "expected Loop"),
            }
        }

        /// BOUND: `Agent::Llm` always wraps the provided `LlmAgent` by value (no field loss).
        #[test]
        fn prop_agent_llm_preserves_name(name in "[a-z]{1,20}") {
            let a = Agent::Llm(LlmAgent {
                name: name.clone(),
                model: ModelRef::new("m"),
                instruction: Instruction::Static("instr".to_owned()),
                description: "d".to_owned(),
                tools: vec![],
                sub_agents: vec![],
                output_key: None,
            });
            match a {
                Agent::Llm(llm) => prop_assert_eq!(llm.name, name),
                _ => prop_assert!(false, "expected Llm"),
            }
        }
    }
}
