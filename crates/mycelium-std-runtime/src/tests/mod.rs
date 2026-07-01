//! In-crate test modules for `mycelium-std-runtime` (M-797 test layout).
//!
//! One submodule per source module, each doing `use crate::…::*` for white-box access.
//! Logic files carry no test code — tests live here.
//!
//! `scheduler` has no test submodule here: the Scheduler itself (and its tests) relocated to
//! `mycelium-sched` (E25/M-862 dependency-cycle fix); `crate::scheduler` is now a thin re-export
//! (see `src/scheduler.rs`), so its behavior is exercised by `mycelium-sched`'s own test suite.

pub mod composition;
pub mod network;
pub mod rc;
pub mod reclamation;
pub mod region;
pub mod scope_region;
