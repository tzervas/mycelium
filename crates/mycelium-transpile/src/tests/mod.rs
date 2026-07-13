//! Test entry point (house rule: no inline tests in logic files — every `#[cfg(test)]` unit test
//! lives in this dedicated in-crate module, per CLAUDE.md "Test layout").

mod batch;
mod corpus;
mod diff;
mod emit;
mod invariant;
mod map;
mod mut_thread;
mod prim_map;
mod remap;
mod reserved;
mod symtab;
mod taxonomy;
mod transpile;
mod vet;
