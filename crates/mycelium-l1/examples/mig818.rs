//! M-818 one-shot `.myc` corpus migrator (layout-preserving): insert the now-mandatory `;`
//! component terminators (DN-57 §3) into each file in place, **without** reformatting (comments,
//! blank lines, indentation preserved). Uses `parse_phylum_lenient_points` to locate each missing
//! `;` and inserts it at the end of the preceding component.
//!
//! Usage: `cargo run -p mycelium-l1 --example mig818 -- <file.myc> ...`
//! Exits non-zero (and reports to stderr) on any file that does not lenient-parse, so a fixture the
//! migrator cannot handle is never silently left un-migrated (G2).

use std::fs;

fn offset_of(src: &str, line: u32, col: u32) -> usize {
    let mut cur_line = 1u32;
    let mut idx = 0usize;
    for (i, ch) in src.char_indices() {
        if cur_line == line {
            let mut c = 1u32;
            let mut j = i;
            for (k, _) in src[i..].char_indices() {
                if c == col {
                    return i + k;
                }
                c += 1;
                j = i + k;
            }
            return j + 1;
        }
        if ch == '\n' {
            cur_line += 1;
            idx = i + 1;
        }
    }
    idx
}

fn end_of_preceding(src: &str, at: usize) -> usize {
    let bytes = src.as_bytes();
    let mut i = at;
    loop {
        while i > 0 && (bytes[i - 1] as char).is_whitespace() {
            i -= 1;
        }
        let line_start = src[..i].rfind('\n').map_or(0, |p| p + 1);
        let line = &src[line_start..i];
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            i = line_start;
            if i == 0 {
                break;
            }
            continue;
        }
        if let Some(pos) = line.find("//") {
            let real_end = line_start + pos;
            let mut e = real_end;
            while e > line_start && (bytes[e - 1] as char).is_whitespace() {
                e -= 1;
            }
            return e;
        }
        break;
    }
    i
}

fn migrate_one(path: &str) -> Result<(), String> {
    let src = fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    let points = mycelium_l1::parse_phylum_lenient_points(&src)
        .map_err(|e| format!("{path}: lenient parse failed: {e}"))?;
    if points.is_empty() {
        return Ok(());
    }
    let mut offsets: Vec<usize> = points
        .iter()
        .map(|p| end_of_preceding(&src, offset_of(&src, p.line, p.col)))
        .collect();
    offsets.sort_unstable();
    offsets.dedup();
    let mut out = src.clone();
    for off in offsets.into_iter().rev() {
        if out.as_bytes().get(off.wrapping_sub(1)) == Some(&b';') {
            continue;
        }
        out.insert(off, ';');
    }
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
