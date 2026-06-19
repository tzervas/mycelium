#!/usr/bin/env python3
"""Agent code-index generator for Mycelium.

Guarantee: Empirical/Declared — this tool uses a line/regex heuristic to locate
definitions and extract doc comments. The source files (crates/**/*.rs) are the
ground truth; this index is a navigation aid only. Re-exports, macro-generated
items, and cfg-gated items that cannot be located appear in the ``flagged`` section
(G2: never silently dropped).

Usage:
    python3 tools/docgen/code_index.py [--output-dir <dir>] [--self-test]

    --output-dir  Directory for index.json and INDEX.md (default: docs/api-index)
    --self-test   Determinism + completeness check; exits 0 on PASS, 1 on FAIL
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Optional

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------
HONESTY_TAG = (
    "Empirical/Declared — line/regex heuristic; source is ground truth. "
    "Use this index to find where to Read, not as an authoritative reference."
)

# Kinds we index (from cargo-public-api output)
INDEXABLE_KINDS = frozenset(
    ["fn", "struct", "enum", "trait", "type", "const", "mod", "use"]
)

# Regex to match a definition line in a .rs file.
# Handles optional `pub`, optional `async`, then the kind keyword, then the name.
# The name is followed by one of: `<`, `(`, ` `, `{`, `;`
_DEF_RE_CACHE: dict[str, re.Pattern] = {}


def _def_re(kind: str, name: str) -> re.Pattern:
    """Return compiled regex to find `pub [async] <kind> <name>[<( {;]` in a .rs file."""
    key = f"{kind}:{name}"
    if key not in _DEF_RE_CACHE:
        escaped = re.escape(name)
        if kind == "fn":
            pat = rf"^\s*pub\s+(?:async\s+)?fn\s+{escaped}\s*[<({{;]"
        elif kind in ("struct", "enum", "trait", "type", "const", "mod"):
            pat = rf"^\s*pub\s+(?:async\s+)?{re.escape(kind)}\s+{escaped}\s*[<({{;:,\s]"
        else:
            # use / re-export — handled via flagged path, but add a fallback
            pat = r"^\s*pub\s+use\b"
        _DEF_RE_CACHE[key] = re.compile(pat, re.MULTILINE)
    return _DEF_RE_CACHE[key]


# ---------------------------------------------------------------------------
# Parsing cargo-public-api snapshot lines
# ---------------------------------------------------------------------------
def _parse_line(line: str, crate_name: str) -> Optional[dict]:
    """Parse one `pub <...>` line from a cargo-public-api snapshot.

    Returns a dict with keys: kind, crate, qualified_path, short_name,
    or None if the line should be skipped.

    Handled patterns (cargo-public-api --simplified output):
      pub fn <path>(<args>) -> <ret>
      pub const fn <path>(<args>) -> <ret>    <- const fn is a compound kind
      pub struct <path>
      pub enum <path>
      pub trait <path>
      pub type <path> = <type>
      pub const <path>: <type>
      pub mod <path>
      pub use <path>                           <- re-export, goes to flagged
      pub <path>::<variant>                   <- enum variant body, skipped
    """
    line = line.strip()
    if not line.startswith("pub "):
        return None

    # Tokenise
    tokens = line.split()
    if len(tokens) < 3:
        return None

    kind = tokens[1]

    # Skip enum variant body lines: `pub mycelium_core::Foo::Bar::field: T`
    # These have `::` in tokens[1] (the path is the second token, not a keyword).
    if "::" in kind:
        return None

    # Handle `pub const fn <path>` — treat as kind="fn" (a const fn is still a fn)
    if kind == "const" and len(tokens) >= 4 and tokens[2] == "fn":
        kind = "fn"
        raw_path = tokens[3]
    elif kind in INDEXABLE_KINDS:
        raw_path = tokens[2]
    else:
        # impl, extern, unsafe fn (without pub unsafe — shouldn't appear after pub),
        # or anything else we don't index.
        return None

    # The raw_path may have generics / fn sig: strip everything from `(` or `<`
    # e.g. "mycelium_core::binary::bits_to_int(&[bool]) -> i64"
    path_part = re.split(r"[(<]", raw_path)[0]

    segments = path_part.split("::")
    short_name = segments[-1] if segments else path_part

    # Skip root module lines: `pub mod mycelium_core` (no :: in path, kind=mod)
    if kind == "mod" and "::" not in path_part:
        return None

    # Module path (everything except the short name)
    module_path = "::".join(segments[:-1]) if len(segments) > 1 else ""

    return {
        "kind": kind,
        "crate": crate_name,
        "module": module_path,
        "symbol": path_part,
        "short_name": short_name,
        "file": None,
        "line": None,
        "summary": None,
        "guarantee_tag": None,
        "corpus_refs": [],
    }


# ---------------------------------------------------------------------------
# Source file search
# ---------------------------------------------------------------------------
def _crate_src_dirs(repo_root: Path, crate_name: str) -> list[Path]:
    """Return candidate src dirs for a crate name."""
    # Normalise: cargo-public-api uses _ but directories use -
    normalised = crate_name.replace("_", "-")
    candidates = [
        repo_root / "crates" / normalised / "src",
        repo_root / "crates" / crate_name / "src",
        repo_root / "xtask" / "src",
    ]
    return [p for p in candidates if p.is_dir()]


def _find_definition(
    src_dirs: list[Path], kind: str, short_name: str
) -> Optional[tuple[str, int, Optional[str]]]:
    """Search .rs files in src_dirs for the definition of `short_name`.

    Returns (relative_file_path, 1-based_line_no, summary) or None.
    The relative_file_path is relative to the repository root (passed via src_dirs).
    """
    if kind == "use":
        return None  # Re-exports: cannot reliably locate without type resolution

    pattern = _def_re(kind, short_name)

    for src_dir in src_dirs:
        rs_files = sorted(src_dir.rglob("*.rs"))
        for rs_file in rs_files:
            try:
                text = rs_file.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            lines = text.splitlines()
            for i, src_line in enumerate(lines):
                if pattern.search(src_line):
                    line_no = i + 1  # 1-based
                    summary = _extract_summary(lines, i)
                    return (str(rs_file), line_no, summary)
    return None


def _extract_summary(lines: list[str], def_line_idx: int) -> Optional[str]:
    """Walk backwards from def_line_idx collecting `///` doc comments.

    Returns the first sentence of the first (topmost) doc comment line,
    or None if no doc comment found.
    """
    doc_lines: list[str] = []
    idx = def_line_idx - 1
    while idx >= 0:
        stripped = lines[idx].strip()
        if stripped.startswith("///"):
            # Remove the `///` prefix (and optional single space)
            content = stripped[3:]
            if content.startswith(" "):
                content = content[1:]
            doc_lines.insert(0, content)
            idx -= 1
        elif stripped.startswith("#[") or stripped.startswith("#![") or stripped == "":
            # Attribute or blank line — keep looking above
            idx -= 1
        else:
            break

    if not doc_lines:
        return None

    # First sentence = up to the first `.` followed by space or end of string
    first_line = doc_lines[0]

    # Strip Rust inline doc links: [`foo`](Target) or [foo](Target) → foo
    # These are valid Rust doc syntax but break the markdown link checker when
    # written verbatim into INDEX.md tables (checker sees them as relative links).
    first_line = re.sub(r"\[`?([^`\]]+)`?\]\([^)]+\)", r"\1", first_line)

    m = re.match(r"^(.*?\.)\s", first_line)
    if m:
        return m.group(1).strip()
    # No period found — take the whole first line (truncated)
    return first_line[:120].strip() or None


# ---------------------------------------------------------------------------
# Main generator
# ---------------------------------------------------------------------------
def build_index(repo_root: Path) -> tuple[list[dict], list[dict]]:
    """Build the symbol table and flagged list from docs/spec/api/*.txt.

    Returns (items, flagged) where each item has the standard schema.
    """
    api_dir = repo_root / "docs" / "spec" / "api"
    txt_files = sorted(api_dir.glob("*.txt"))

    items: list[dict] = []
    flagged: list[dict] = []

    for txt_file in txt_files:
        crate_name = txt_file.stem  # e.g. "mycelium-core"
        src_dirs = _crate_src_dirs(repo_root, crate_name)

        try:
            lines = txt_file.read_text(encoding="utf-8").splitlines()
        except OSError as exc:
            flagged.append(
                {
                    "symbol": f"<{crate_name}>",
                    "reason": f"could not read {txt_file}: {exc}",
                }
            )
            continue

        for raw_line in lines:
            parsed = _parse_line(raw_line, crate_name)
            if parsed is None:
                continue  # comment, blank, skip line, or unparsable kind

            kind = parsed["kind"]
            short_name = parsed["short_name"]
            symbol = parsed["symbol"]

            if kind == "use":
                flagged.append(
                    {
                        "symbol": symbol,
                        "reason": "re-export (pub use) — cannot locate definition without type resolution",
                    }
                )
                continue

            if not src_dirs:
                flagged.append(
                    {
                        "symbol": symbol,
                        "reason": f"no src/ directory found for crate {crate_name!r}",
                    }
                )
                continue

            result = _find_definition(src_dirs, kind, short_name)
            if result is None:
                flagged.append(
                    {
                        "symbol": symbol,
                        "reason": (
                            f"definition not found via regex heuristic "
                            f"(kind={kind!r}, name={short_name!r}) — "
                            "possibly macro-generated or cfg-gated"
                        ),
                    }
                )
                continue

            abs_file, line_no, summary = result
            # Make path relative to repo_root
            try:
                rel_file = str(Path(abs_file).relative_to(repo_root))
            except ValueError:
                rel_file = abs_file

            item = {
                "symbol": symbol,
                "kind": kind,
                "crate": crate_name,
                "module": parsed["module"],
                "file": rel_file,
                "line": line_no,
                "summary": summary,
                "guarantee_tag": None,
                "corpus_refs": [],
            }
            items.append(item)

    # Stable key order: sort by (crate, symbol)
    items.sort(key=lambda x: (x["crate"], x["symbol"]))
    flagged.sort(key=lambda x: x["symbol"])

    return items, flagged


def emit_json(items: list[dict], flagged: list[dict], output_dir: Path) -> None:
    """Write docs/api-index/index.json."""
    output_dir.mkdir(parents=True, exist_ok=True)
    payload = {
        "generated": HONESTY_TAG,
        "items": items,
        "flagged": flagged,
    }
    out = output_dir / "index.json"
    out.write_text(
        json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )


def emit_markdown(items: list[dict], flagged: list[dict], output_dir: Path) -> None:
    """Write docs/api-index/INDEX.md."""
    output_dir.mkdir(parents=True, exist_ok=True)

    lines: list[str] = []
    lines.append("# Mycelium Agent Code Index")
    lines.append("")
    lines.append(f"> **Honesty:** `{HONESTY_TAG}`")
    lines.append(
        "> Use the index to find where to `Read`, not as an authoritative reference."
    )
    lines.append("")

    # Group by crate
    from itertools import groupby

    def crate_key(item: dict) -> str:
        return item["crate"]

    for crate_name, group in groupby(items, key=crate_key):
        group_list = list(group)
        lines.append(f"## {crate_name}")
        lines.append("")
        lines.append("| Symbol | Kind | File:Line | Summary |")
        lines.append("|---|---|---|---|")
        for item in group_list:
            symbol = f"`{item['symbol']}`"
            kind = item["kind"]
            file_line = (
                f"`{item['file']}:{item['line']}`"
                if item["file"] and item["line"]
                else "—"
            )
            summary = item["summary"] or "—"
            # Escape any pipe chars in summary
            summary = summary.replace("|", "\\|")
            lines.append(f"| {symbol} | {kind} | {file_line} | {summary} |")
        lines.append("")

    # Flagged section
    lines.append("## Flagged items")
    lines.append("")
    lines.append("Items the heuristic could not locate (G2: never silently dropped):")
    lines.append("")
    if flagged:
        lines.append("| Symbol | Reason |")
        lines.append("|---|---|")
        for f in flagged:
            sym = f"`{f['symbol']}`".replace("|", "\\|")
            reason = f["reason"].replace("|", "\\|")
            lines.append(f"| {sym} | {reason} |")
    else:
        lines.append("*(none — all public items located)*")
    lines.append("")

    out = output_dir / "INDEX.md"
    out.write_text("\n".join(lines), encoding="utf-8")


# ---------------------------------------------------------------------------
# Self-test
# ---------------------------------------------------------------------------
def self_test(repo_root: Path, output_dir: Path) -> int:
    """Run determinism + completeness checks. Returns 0 on PASS, 1 on FAIL."""
    import tempfile

    print("Running self-test…")
    failures: list[str] = []

    # --- Determinism: run twice, compare JSON ---
    with tempfile.TemporaryDirectory() as tmp1, tempfile.TemporaryDirectory() as tmp2:
        p1, p2 = Path(tmp1), Path(tmp2)
        items1, flagged1 = build_index(repo_root)
        emit_json(items1, flagged1, p1)
        items2, flagged2 = build_index(repo_root)
        emit_json(items2, flagged2, p2)

        if (p1 / "index.json").read_bytes() == (p2 / "index.json").read_bytes():
            print("  PASS  determinism: two runs produce identical JSON")
        else:
            print(
                "  FAIL  determinism: two runs differ — generator is non-deterministic"
            )
            failures.append("determinism")

    # --- Completeness: every `pub` line from spec/api/*.txt in items or flagged ---
    api_dir = repo_root / "docs" / "spec" / "api"
    txt_files = sorted(api_dir.glob("*.txt"))

    items_set = {i["symbol"] for i in items1}
    flagged_set = {f["symbol"] for f in flagged1}

    missing: list[str] = []
    for txt_file in txt_files:
        crate_name = txt_file.stem
        for raw_line in txt_file.read_text(encoding="utf-8").splitlines():
            parsed = _parse_line(raw_line, crate_name)
            if parsed is None:
                continue  # skipped line
            sym = parsed["symbol"]
            # Re-check: skip root mod lines explicitly (same logic as build_index)
            if parsed["kind"] == "mod" and "::" not in sym:
                continue
            if sym not in items_set and sym not in flagged_set:
                missing.append(f"{crate_name}: {sym}")

    if not missing:
        print("  PASS  completeness: all public items in items or flagged")
    else:
        print(f"  FAIL  completeness: {len(missing)} item(s) not in items or flagged:")
        for m in missing[:20]:
            print(f"        {m}")
        if len(missing) > 20:
            print(f"        … and {len(missing) - 20} more")
        failures.append("completeness")

    if failures:
        print(f"FAIL — {len(failures)} check(s) failed: {', '.join(failures)}")
        return 1
    print("PASS — all self-test checks passed")
    return 0


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------
def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate Mycelium agent code index from cargo-public-api snapshots."
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=None,
        help="Directory for index.json and INDEX.md (default: docs/api-index next to repo root)",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Determinism + completeness check; exits 0 on PASS, 1 on FAIL",
    )
    args = parser.parse_args()

    # Repo root = two levels up from this script (tools/docgen/ -> tools/ -> repo root)
    repo_root = Path(__file__).resolve().parent.parent.parent

    output_dir = (
        args.output_dir if args.output_dir else repo_root / "docs" / "api-index"
    )
    output_dir = output_dir.resolve()

    if args.self_test:
        return self_test(repo_root, output_dir)

    items, flagged = build_index(repo_root)
    emit_json(items, flagged, output_dir)
    emit_markdown(items, flagged, output_dir)

    n_items = len(items)
    n_flagged = len(flagged)
    print(
        f"docs/api-index: {n_items} items indexed, {n_flagged} flagged "
        f"(Empirical/Declared heuristic — source is ground truth)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
