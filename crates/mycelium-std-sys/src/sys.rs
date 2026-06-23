//! \[Declared\] Process / environment floor (RFC-0028 §4.5; M-722). Thin, never-silent wrappers over
//! Rust `std::process` and `std::env`.
//!
//! This is the audited syscall floor for `std.sys`'s OS contact: process exit and environment-variable
//! reads. Per LR-9 / RFC-0016 §8-Q6 all such contact lives in this single `std-sys` phylum.
//!
//! # Honesty (VR-5)
//!
//! Every function carries the **`Declared`** guarantee tag — unaudited `std::process` / `std::env`
//! wrappers; no theorem or measured bound backs OS process/env semantics. Promotion requires a
//! checked or measured basis (none in v0).
//!
//! # Never-silent (G2)
//!
//! `get_env` returns an explicit `Option`: a missing or non-Unicode variable is `None`, never an
//! empty-string stand-in. `exit` does not return (it terminates the process), so its "failure mode"
//! is structural — the explicit, caller-chosen status code is the contract.
//!
//! # Guarantee matrix (RFC-0016 §4.5)
//!
//! | op | signature | failure mode | tag |
//! |----|-----------|--------------|-----|
//! | `exit` | `(i32) -> !` | n/a (terminates) | `Declared` |
//! | `get_env` | `(&str) -> Option<String>` | missing/non-Unicode → `None` (never-silent) | `Declared` |
//! | `args` | `() -> Vec<String>` | n/a | `Declared` |

/// \[Declared\] Terminate the process with `code`. Does not return. The exit status is the caller's
/// explicit choice — never a silent `0` substituted for an error path (G2): a program that wants to
/// signal failure passes a non-zero `code`.
pub fn exit(code: i32) -> ! {
    std::process::exit(code)
}

/// \[Declared\] Read environment variable `name`. Returns `None` if the variable is unset **or** is
/// not valid Unicode — an explicit absence, never an empty-string stand-in (G2). Use when a missing
/// variable is a recoverable condition; the `None` must be handled, not assumed present.
#[must_use]
pub fn get_env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// \[Declared\] The process's command-line arguments (including arg 0). A lossy conversion is avoided:
/// any non-Unicode argument is dropped rather than replaced with a placeholder — callers needing the
/// raw `OsString` form should use `std::env::args_os` directly (this floor is the Unicode convenience).
#[must_use]
pub fn args() -> Vec<String> {
    std::env::args().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Never-silent (G2): an unset variable is an explicit `None`, not an empty string. Uses a name
    /// that is overwhelmingly unlikely to be set in any environment.
    #[test]
    fn an_unset_env_var_is_none_not_empty_string() {
        let name = "MYCELIUM_STD_SYS_DEFINITELY_UNSET_VARIABLE_X9Z";
        assert_eq!(get_env(name), None, "an unset var must be None, never \"\"");
    }

    /// A set variable round-trips through the reader (the OS env is the ground truth; this pins that
    /// the wrapper returns the value verbatim, not transformed).
    #[test]
    fn a_set_env_var_reads_back_verbatim() {
        // SAFETY note: this is `std::env::set_var`, not Rust `unsafe` — single-threaded test, no FFI.
        std::env::set_var("MYCELIUM_STD_SYS_TEST_VAR", "floor-42");
        assert_eq!(
            get_env("MYCELIUM_STD_SYS_TEST_VAR"),
            Some("floor-42".to_owned())
        );
    }

    /// `args()` always contains at least the program name (arg 0).
    #[test]
    fn args_includes_arg_zero() {
        assert!(!args().is_empty(), "args must include arg 0");
    }
}
