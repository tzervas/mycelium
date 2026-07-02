//! In-crate test modules (CLAUDE.md test-layout rule): one submodule per source module,
//! white-box via `use crate::…` — logic files carry no test code.

mod inject_gate;
mod wild_audit;
