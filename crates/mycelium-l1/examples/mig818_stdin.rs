//! M-818 migration filter (batch, layout-preserving): read stdin containing one-or-more Mycelium
//! programs separated by the sentinel line `===MIG818===`, and for each, insert the now-mandatory
//! `;` component terminators (DN-57 §3) **without** reformatting — comments, blank lines, and
//! indentation are preserved. Each migrated program is written to stdout separated by the same
//! sentinel. A program that does not lenient-parse is echoed UNCHANGED behind a `===MIG818SKIP===`
//! marker (so the caller leaves it alone — never a silent corruption, G2).
//!
//! Minimal insertion: `parse_phylum_lenient_points` reports each `(line, col)` where a `;` was
//! missing (the position of the token that should have followed it). We map each to a byte offset,
//! walk back past whitespace and `//` line-comments, and insert `;` immediately after the last
//! non-whitespace/non-comment character — i.e. at the true end of the preceding component.

use std::io::Read;

const SEP: &str = "===MIG818===";
const SKIP: &str = "===MIG818SKIP===";

/// Byte offset of a 1-based (line, col) in `src`.
fn offset_of(src: &str, line: u32, col: u32) -> usize {
    let mut cur_line = 1u32;
    let mut idx = 0usize;
    for (i, ch) in src.char_indices() {
        if cur_line == line {
            // col is 1-based char column within the line.
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

/// Walk back from `at` over whitespace and trailing `//` line-comments; return the offset of the end
/// of the last real (non-whitespace, non-comment) content before `at`. `;` is inserted there.
fn end_of_preceding(src: &str, at: usize) -> usize {
    let bytes = src.as_bytes();
    let mut i = at;
    loop {
        // Skip whitespace immediately before i.
        while i > 0 && (bytes[i - 1] as char).is_whitespace() {
            i -= 1;
        }
        // If the line ending at i is a `// …` comment, skip the whole comment line and continue.
        // Find the start of the current line.
        let line_start = src[..i].rfind('\n').map_or(0, |p| p + 1);
        let line = &src[line_start..i];
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            // The whole line is a comment (after earlier content was consumed) — drop it and loop.
            i = line_start;
            if i == 0 {
                break;
            }
            continue;
        }
        // Check for a trailing `// …` after real content on this line: cut at the `//`.
        if let Some(pos) = line.find("//") {
            // Only treat as a comment if `//` is not inside content we care about; here the line has
            // real content before `//`, so the component ends before the comment.
            let real_end = line_start + pos;
            // Trim trailing whitespace before the comment.
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

fn migrate_one(src: &str) -> Result<String, ()> {
    let points = mycelium_l1::parse_phylum_lenient_points(src).map_err(|_| ())?;
    if points.is_empty() {
        return Ok(src.to_owned());
    }
    // Compute insertion byte offsets, dedup + sort descending so earlier inserts don't shift later.
    let mut offsets: Vec<usize> = points
        .iter()
        .map(|p| {
            let at = offset_of(src, p.line, p.col);
            end_of_preceding(src, at)
        })
        .collect();
    offsets.sort_unstable();
    offsets.dedup();
    let mut out = src.to_owned();
    for off in offsets.into_iter().rev() {
        // Avoid a double `;` if one is somehow already there.
        if out.as_bytes().get(off.wrapping_sub(1)) == Some(&b';') {
            continue;
        }
        out.insert(off, ';');
    }
    Ok(out)
}

fn main() {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .expect("read stdin");
    let chunks: Vec<&str> = input.split(SEP).collect();
    let mut outs: Vec<String> = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        match migrate_one(chunk) {
            Ok(out) => outs.push(out),
            Err(()) => outs.push(format!("{SKIP}\n{chunk}")),
        }
    }
    print!("{}", outs.join(SEP));
}
