//! M-818 one-shot corpus migrator: read a `.myc` (or extracted program) file with the lenient
//! optional-`;` parser, re-emit it with the mandatory-`;` canonical printer. Preserves the leading
//! `//` comment banner verbatim (the conformance fixtures carry an `// exercises:` header). Phylum
//! sources round-trip through `expand_phylum_to_source`; single-nodule through `expand_to_source`.
//!
//! Usage: `cargo run -p mycelium-l1 --example mig818 -- <file.myc> ...`
//! Exits non-zero (and prints to stderr) on any file that does not lenient-parse, so a fixture the
//! migrator cannot handle is never silently left un-migrated (G2).

use std::fs;

fn migrate_one(path: &str) -> Result<(), String> {
    let src = fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;

    // Split off the leading comment/blank banner so it survives the printer (which drops comments).
    let mut banner = String::new();
    let mut body_start = 0usize;
    for line in src.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.is_empty() {
            banner.push_str(line);
            body_start += line.len();
        } else {
            break;
        }
    }
    let body = &src[body_start..];

    // Lenient-parse the body, then re-emit terminated. Prefer the phylum entry (superset).
    let rendered = match mycelium_l1::parse_phylum_lenient(body) {
        Ok(ph) => mycelium_l1::expand_phylum_to_source(&ph),
        Err(e) => return Err(format!("{path}: lenient phylum parse failed: {e}")),
    };

    let mut out = banner;
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&rendered);
    fs::write(path, out).map_err(|e| format!("write {path}: {e}"))?;
    Ok(())
}

fn main() {
    let mut failures = 0;
    for path in std::env::args().skip(1) {
        match migrate_one(&path) {
            Ok(()) => println!("migrated {path}"),
            Err(e) => {
                eprintln!("FAILED {e}");
                failures += 1;
            }
        }
    }
    if failures > 0 {
        std::process::exit(1);
    }
}
