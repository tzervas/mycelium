//! In-crate white-box test modules (test layout rule: no tests in logic files; one submodule per
//! source module under test). Extracted as-touched from the logic files (M-797); these modules use
//! `use crate::<mod>::*` for white-box access to the logic module's `pub(crate)` items.

mod ambient;
mod ast;
mod checkty;
mod elab;
mod eval;
mod lexer;
mod lib_root;
mod mono;
mod parse;
