//! In-crate white-box test modules (test layout rule, CLAUDE.md: no tests in logic files; one
//! submodule per source module under test). This crate's inline `#[cfg(test)] mod tests` blocks
//! elsewhere are pre-existing (M-797 lazy-retrofit debt, not yet swept); these modules are
//! extracted **as-touched** — `build.rs` (extended with `extra_md_files`, §`crate::book`), the
//! brand-new `book.rs`, and `ir.rs` (extracted when `Node::walk` was guarded, RFC-0041 W1/§4.7),
//! which start clean here rather than compounding the debt.

mod book;
mod build;
mod ir;
