//! `mycelium-std-sys` — audited FFI/syscall floor for the Mycelium standard library.
//!
//! # Purpose
//!
//! This crate is the **single, audited phylum** for all `wild`/FFI/OS-syscall contact in the
//! Mycelium standard library tree. By routing every low-level interface through `std-sys`
//! exclusively, the pure `std` crates (`std-math`, `std-rand`, `std-fs`, `std-time`, …) can
//! earn a **`wild`-free badge** as soon as they wire through this phylum — no `unsafe` or
//! FFI contact anywhere in their own code (RFC-0016 §9).
//!
//! **Wiring** the existing std crates through `std-sys` is deferred to a future wave (M-541
//! established the floor; the call-site plumbing is the next step). The pure std crates
//! currently use Rust's own `f64` / `std::*` wrappers as placeholders, tagged `Declared`,
//! with FLAGs pointing here.
//!
//! # Honesty
//!
//! All functions in this crate carry the **`Declared`** guarantee tag (RFC-0016 §4.1 C2 /
//! VR-5). No audited theorem backs the libm precision, OS entropy quality, FS semantics, or
//! clock resolution. Promotion requires:
//! - `Empirical`: documented test coverage with measured error bounds.
//! - `Proven`: a verified theorem whose side-conditions are checked.
//!
//! Neither is established in v0 of this crate.
//!
//! # LR-9 rationale — `std-sys` as a phylum boundary
//!
//! LR-9 mandates that `wild` (unsafe FFI) appear in **exactly one place** in the std tree —
//! this crate. Placing syscall contact here (rather than scattered across individual std crates)
//! means the `wild` audit surface is bounded, inspectable, and `EXPLAIN`-able (G2/G11/SC-3).
//! The rest of the std library stays pure Rust with no `unsafe` blocks, satisfying KC-3.
//!
//! # Modules
//!
//! - [`math`] — transcendental function floor (libm via Rust `f64` intrinsics).
//! - [`rand`] — platform entropy floor (`fill_bytes`).
//! - [`fs`] — filesystem syscall floor (thin `std::fs` wrappers).
//! - [`time`] — OS clock floor (wall + monotonic + sleep).

#![forbid(unsafe_code)]

pub mod fs;
pub mod math;
pub mod rand;
pub mod time;
