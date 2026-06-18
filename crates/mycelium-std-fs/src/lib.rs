//! `std.fs` — std.fs — filesystem over substrates (explicit failures).
//!
//! **Scaffold (Batch P5-B).** This crate is the registered, buildable skeleton for the
//! `std.fs` standard-library module (M-528). The implementation lands in this
//! branch's wave per its design spec (`docs/spec/stdlib/fs.md`) and the RFC-0016 §4.1
//! contract (C1–C6), shipping a checked guarantee matrix (§4.5). It is a KC-3 consumer of
//! the landed kernel/capability crates — it adds **no trusted code** and carries no
//! `unsafe`.
#![forbid(unsafe_code)]
