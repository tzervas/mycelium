//! Live-executor scope/region wiring — SCAFFOLD (filled by the MEM live-wiring leaf).
//!
//! This module ties a [`crate::region::Region`]'s lifecycle to a single-hypha
//! structured-concurrency scope so that scope-exit reclamation (`ScopeExit`) fires from the
//! running executor. Implementation lands in the live-wiring leaf.
