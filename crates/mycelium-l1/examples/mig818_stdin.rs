//! M-818 migration filter (batch): read stdin containing one-or-more Mycelium programs separated by
//! the sentinel line `===MIG818===`, lenient-parse each (optional `;`), and write each program's
//! mandatory-`;` canonical re-emission to stdout, separated by the same sentinel. A program that
//! does not lenient-parse is echoed UNCHANGED with a `===MIG818SKIP===` marker line prepended in
//! its slot, so the Python rewriter skips it (never a silent corruption, G2).
//!
//! Used by the scratchpad Python rewriter that migrates inline program strings in Rust test files;
//! one cargo invocation handles all literals in a file.

use std::io::Read;

const SEP: &str = "===MIG818===";
const SKIP: &str = "===MIG818SKIP===";

fn migrate_one(src: &str) -> Result<String, ()> {
    mycelium_l1::parse_phylum_lenient(src)
        .map(|ph| mycelium_l1::expand_phylum_to_source(&ph))
        .or_else(|_| mycelium_l1::parse_lenient(src).map(|n| mycelium_l1::expand_to_source(&n)))
        .map_err(|_| ())
}

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).expect("read stdin");

    let chunks: Vec<&str> = input.split(SEP).collect();
    let mut outs: Vec<String> = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        match migrate_one(chunk) {
            Ok(out) => outs.push(out),
            // Echo unchanged behind the SKIP marker so the caller leaves this literal alone.
            Err(()) => outs.push(format!("{SKIP}\n{chunk}")),
        }
    }
    print!("{}", outs.join(SEP));
}
