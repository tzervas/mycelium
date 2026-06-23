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
    """Return the **unique** existing src dir(s) for a crate name.

    Deduped by resolved path: when `normalised == crate_name` (no `_`/`-` difference)
    the two candidates collapse to one dir — searching it twice would make every
    symbol look doubly-defined (a false ambiguity). `xtask/src` is only a candidate
    for the `xtask` crate itself, not every crate (accuracy: a non-xtask symbol must
    not resolve into xtask).
    """
    # Normalise: cargo-public-api uses _ but directories use -.
    normalised = crate_name.replace("_", "-")
    candidates = [
        repo_root / "crates" / normalised / "src",
        repo_root / "crates" / crate_name / "src",
    ]
    if crate_name in ("xtask", "mycelium-xtask", "mycelium_xtask"):
        candidates.append(repo_root / "xtask" / "src")
    seen: set[Path] = set()
    out: list[Path] = []
    for p in candidates:
        if not p.is_dir():
            continue
        resolved = p.resolve()
        if resolved in seen:
            continue
        seen.add(resolved)
        out.append(p)
    return out


def _file_module_path(rs_file: Path, src_dir: Path) -> str:
    """The within-crate module path implied by a file's location under `src_dir`.

    Honors both Rust module-file conventions: `src/lib.rs` / `src/main.rs` and any
    `…/mod.rs` map to their *containing* module; a plain `foo.rs` adds `foo`. So
    `src/lib.rs` → ''; `src/llvm.rs` → 'llvm'; `src/dialect/native.rs` →
    'dialect::native'; `src/dialect/mod.rs` → 'dialect'. Matched against a symbol's
    (crate-prefix-stripped) module path to attribute it to the right file even when
    the same short name is defined in several modules.
    """
    try:
        rel = rs_file.relative_to(src_dir)
    except ValueError:
        return ""
    parts = list(rel.parts)
    if not parts:
        return ""
    stem = parts[-1]
    if stem in ("lib.rs", "main.rs", "mod.rs"):
        segs = list(parts[:-1])
    else:
        segs = list(parts[:-1]) + [stem[:-3] if stem.endswith(".rs") else stem]
    return "::".join(segs)


def _strip_crate_prefix(module: str, crate_name: str) -> str:
    """Drop the leading crate segment from a qualified module path — the qualified
    symbol path carries it (`mycelium_mlir::llvm`) but file paths don't (`llvm`)."""
    crate_us = crate_name.replace("-", "_")
    if crate_us and (module == crate_us or module.startswith(crate_us + "::")):
        return module[len(crate_us) :].lstrip(":")
    return module


def _module_for_file(sym_mod: str) -> str:
    """The *module* whose file holds a symbol's definition, for file matching.

    Drops trailing type/trait segments (Rust convention: modules are `snake_case`,
    types/traits are `PascalCase`; a method/assoc-fn lives in an `impl Type` block
    inside its module's file, not a file named after the type). `node::Node` → `node`
    (so `Node::content_hash` resolves to `node.rs`); `llvm` → `llvm`; `Foo` → ''.
    """
    segs = sym_mod.split("::") if sym_mod else []
    while segs and segs[-1][:1].isupper():
        segs.pop()
    return "::".join(segs)


def _disambiguate_by_module(
    matches: list[tuple[str, int, Optional[str], str]], sym_mod: str
) -> Optional[tuple[str, int, Optional[str]]]:
    """Pick the definition whose file-module matches the symbol's module.

    Accurate + robust (no brittle brace parsing): prefer an **exact** file-module
    match (the file-per-module case); else the **longest strict-ancestor** file-module
    (the inline-`mod` case, where the remaining segments are inline modules in that
    file). Returns the chosen (file, line, summary) only when it is *unambiguous* —
    a tie (or no module signal) returns None so the caller flags it (G2: never a
    silent mis-attribution; source stays ground truth).
    """
    exact = [m for m in matches if m[3] == sym_mod]
    if len(exact) == 1:
        return exact[0][:3]
    if exact:
        return None  # several defs in the same module → genuinely ambiguous

    # Inline-module fallback: a non-root file-module that is a strict ancestor of the
    # symbol's module (its remaining segments are inline `mod`s inside that file).
    ancestors = sorted(
        (
            m
            for m in matches
            if m[3] and (sym_mod == m[3] or sym_mod.startswith(m[3] + "::"))
        ),
        key=lambda m: len(m[3]),
        reverse=True,
    )
    if ancestors and (
        len(ancestors) == 1 or len(ancestors[0][3]) > len(ancestors[1][3])
    ):
        return ancestors[0][:3]
    return None


def _find_definition(
    src_dirs: list[Path],
    kind: str,
    short_name: str,
    module: str = "",
    crate_name: str = "",
) -> Optional[tuple[str, int, Optional[str], bool]]:
    """Search .rs files in src_dirs for the definition of `short_name`.

    Returns (file_path, 1-based_line_no, summary, ambiguous) or None. When the short
    name is defined in more than one file, the symbol's `module` disambiguates (prefer
    the file whose path matches the module, after stripping the leading crate segment —
    `mycelium_mlir::llvm` → `llvm.rs`). If it still cannot be disambiguated, the first
    match is returned with `ambiguous=True` so the caller flags it (never a silent
    mis-attribution — G2; source stays ground truth).
    """
    if kind == "use":
        return None  # Re-exports: cannot reliably locate without type resolution

    pattern = _def_re(kind, short_name)

    # All definitions of `short_name`, each tagged with its file's module path.
    matches: list[tuple[str, int, Optional[str], str]] = []
    for src_dir in src_dirs:
        for rs_file in sorted(src_dir.rglob("*.rs")):
            try:
                text = rs_file.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            lines = text.splitlines()
            for i, src_line in enumerate(lines):
                if pattern.search(src_line):
                    matches.append(
                        (
                            str(rs_file),
                            i + 1,
                            _extract_summary(lines, i),
                            _file_module_path(rs_file, src_dir),
                        )
                    )
                    break  # first definition per file

    if not matches:
        return None
    if len(matches) == 1:
        f, ln, summ, _ = matches[0]
        return (f, ln, summ, False)

    # Several files define `short_name` → attribute by module (accurate + robust).
    sym_mod = _module_for_file(_strip_crate_prefix(module, crate_name))
    chosen = _disambiguate_by_module(matches, sym_mod)
    if chosen is not None:
        f, ln, summ = chosen
        return (f, ln, summ, False)

    # Genuinely ambiguous → best-effort first match, flagged (never silently wrong, G2).
    f, ln, summ, _ = matches[0]
    return (f, ln, summ, True)


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

            result = _find_definition(
                src_dirs, kind, short_name, parsed["module"], crate_name
            )
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

            abs_file, line_no, summary, ambiguous = result
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
            if ambiguous:
                # Located, but the short name is defined in several modules and the
                # symbol's module path didn't disambiguate — never silently trust it (G2).
                flagged.append(
                    {
                        "symbol": symbol,
                        "reason": (
                            f"ambiguous: short name {short_name!r} is defined in multiple "
                            f"modules; attributed to {rel_file} by heuristic — verify "
                            "against source (ground truth)"
                        ),
                    }
                )

    # De-duplicate rows that resolve to the same (file, line) with the same short symbol
    # name — a re-export alias and its canonical definition are the same item. Keep one
    # canonical row: prefer the entry with the shorter qualified symbol path (the
    # re-exported alias path is shorter than the internal module path for crate-level
    # re-exports, and the canonical module path is shorter for assoc-fn duplicates);
    # break ties lexically for determinism. Genuinely distinct symbols (different
    # file:line, or different short name) are never dropped (G2: never silent).
    # Dropped alias symbols are tracked in `deduped_aliases` (used by the self-test
    # completeness check to confirm they are covered by their canonical row).
    from collections import OrderedDict

    canonical: dict[tuple, dict] = (
        OrderedDict()
    )  # (file, line, short_name) -> best item
    for item in items:
        short = item["symbol"].split("::")[-1]
        key = (item["file"], item["line"], short)
        if key not in canonical:
            canonical[key] = item
        else:
            # Prefer the shorter qualified path (canonical over alias); break ties lexically.
            prev_sym = canonical[key]["symbol"]
            cur_sym = item["symbol"]
            if len(cur_sym) < len(prev_sym) or (
                len(cur_sym) == len(prev_sym) and cur_sym < prev_sym
            ):
                canonical[key] = item
    # Record the alias symbols that were dropped (covered by a canonical row at the same
    # file:line). They go into flagged as "dedup-alias" entries so the completeness
    # check (self-test) can confirm no item was silently lost (G2: never silent).
    for item in items:
        short = item["symbol"].split("::")[-1]
        key = (item["file"], item["line"], short)
        kept_sym = canonical[key]["symbol"]
        if item["symbol"] != kept_sym:
            flagged.append(
                {
                    "symbol": item["symbol"],
                    "reason": (
                        f"dedup-alias: same definition as `{kept_sym}` "
                        f"at {item['file']}:{item['line']} — one canonical row kept"
                    ),
                }
            )
    items = list(canonical.values())

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
    # Deduped aliases appear in flagged with reason "dedup-alias: …" so they are still
    # "covered" here — the completeness invariant is: no item is silently lost (G2).
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

    # --- Module-aware attribution: a short name defined in several modules resolves
    #     by the symbol's module; only genuinely-undecidable cases are ambiguous (G2). ---
    def _case(name: str, got: object, want: object) -> None:
        if got != want:
            print(f"  FAIL  attribution {name}: got {got!r}, want {want!r}")
            failures.append(f"attribution:{name}")

    sd = Path("/c/src")
    _case("file_mod/lib", _file_module_path(sd / "lib.rs", sd), "")
    _case("file_mod/leaf", _file_module_path(sd / "llvm.rs", sd), "llvm")
    _case(
        "file_mod/nested",
        _file_module_path(sd / "dialect" / "native.rs", sd),
        "dialect::native",
    )
    _case(
        "file_mod/mod.rs", _file_module_path(sd / "dialect" / "mod.rs", sd), "dialect"
    )
    _case(
        "strip_crate",
        _strip_crate_prefix("mycelium_mlir::llvm", "mycelium-mlir"),
        "llvm",
    )
    _case("mod_for_file/type", _module_for_file("node::Node"), "node")
    _case("mod_for_file/fn", _module_for_file("llvm"), "llvm")
    _case("mod_for_file/bare_type", _module_for_file("Foo"), "")
    # `compile` defined in both llvm and dialect::native → module decides; root is ambiguous.
    twin = [
        ("/c/src/dialect/native.rs", 694, None, "dialect::native"),
        ("/c/src/llvm.rs", 2450, None, "llvm"),
    ]
    _case(
        "disambig/llvm",
        _disambiguate_by_module(twin, "llvm"),
        ("/c/src/llvm.rs", 2450, None),
    )
    _case(
        "disambig/native",
        _disambiguate_by_module(twin, "dialect::native"),
        ("/c/src/dialect/native.rs", 694, None),
    )
    _case("disambig/root_ambiguous", _disambiguate_by_module(twin, ""), None)
    _case(
        "disambig/inline_ancestor",
        _disambiguate_by_module([("/c/src/llvm.rs", 10, None, "llvm")], "llvm::inner"),
        ("/c/src/llvm.rs", 10, None),
    )
    if not any(f.startswith("attribution:") for f in failures):
        print("  PASS  attribution: module-aware symbol resolution (12 cases)")

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
