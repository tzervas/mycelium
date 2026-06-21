//! nodule: `adk.agent` — agent definitions as pure data, per RFC-0023 §4.
//!
//! TODO(leaf ADK / M-671): `LlmAgent` (name, model, instruction, description, tools, sub_agents,
//! output_key), `Workflow { Sequential | Parallel | Loop(max) }`, `Agent { Llm | Flow }`, and
//! `Instruction { Static | Dynamic }` — all pure, value-semantic data (no execution here; the
//! drive loop lives in [`crate::runner`], which is gated).
