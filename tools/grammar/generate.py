#!/usr/bin/env python3
"""Editor-grammar generator (M-731; RFC-0026).

Single source of truth: the canonical L1 lexer `keyword()` table in
`crates/mycelium-l1/src/token.rs`. This generator *extracts* the keyword set from that function
and emits the editor-grammar artifacts so the lexer can never silently diverge from the grammars
that colour editors (G2). A `--check` mode (the `just drift-check` gate) regenerates in memory and
fails on any divergence from the committed files.

HONESTY / status (VR-5, G2):
  * The keyword *set* and its *class buckets* are derived mechanically from `token.rs` (the RHS
    `=> Tok::…` of each arm), never hand-maintained — so adding a keyword to the lexer without
    regenerating fails the drift gate.
  * The *scope names* are the **ratified RFC-0026 §3.2 (Accepted)** table: standard names per layer —
    TextMate scopes carry a `.mycelium` suffix (`TM_SCOPES`), while the tree-sitter captures
    (`TS_CAPTURES`) are the standard *unsuffixed* names (`@keyword`, `@type.builtin`, …). The
    tree-sitter `highlights.scm` covers the four word buckets only (a reserved-word scaffold);
    comment/number/operator/identifier captures arrive with the full structural grammar (RFC-0026
    §3.4). A change to the table supersedes RFC-0026 (append-only); the generator is then re-run.

Usage:
  generate.py                 # (re)write the committed artifacts under tools/grammar/
  generate.py --check         # drift gate: fail (exit 2) if committed artifacts are stale
  generate.py --self-test     # extraction sanity check (offline; exit 3 on failure)
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

# Repo-root-relative locations (this file lives at tools/grammar/generate.py).
HERE = Path(__file__).resolve().parent
REPO_ROOT = HERE.parent.parent
TOKEN_RS = REPO_ROOT / "crates" / "mycelium-l1" / "src" / "token.rs"

# Ratified RFC-0026 §3.2 scope names — standard TextMate / tree-sitter / LSP names with a `.mycelium`
# suffix (chosen for maximal theme compatibility; see RFC-0026 §3.2 / §5 Q2). The keyword/type/scalar/
# strength buckets are lexer-derived (§3.3); these tables fix the NAME each bucket renders to.
TM_SCOPES = {
    "keyword": "keyword.control.mycelium",
    "type": "storage.type.mycelium",
    "scalar": "support.type.builtin.mycelium",
    "strength": "storage.modifier.guarantee.mycelium",
}
TM_COMMENT = "comment.line.double-slash.mycelium"
TM_NUMERIC = "constant.numeric.mycelium"
# tree-sitter highlight captures (RFC-0026 §3.2).
TS_CAPTURES = {
    "keyword": "keyword",
    "type": "type",
    "scalar": "type.builtin",
    "strength": "attribute",
}

# The substrate/representation type keywords — identified by their bare `Tok::<Variant>` RHS.
TYPE_VARIANTS = {"Binary", "Ternary", "Dense", "Vsa", "Substrate", "Sparse"}


def extract_keywords(token_rs: str) -> dict[str, list[str]]:
    """Extract the keyword set from the `keyword()` function body, bucketed by lexer class.

    The buckets are derived from each arm's right-hand side (`=> Tok::…`):
      * `Tok::Strength(...)`  -> `strength`  (the guarantee-strength lattice)
      * `Tok::Scalar(...)`    -> `scalar`    (F16/BF16/F32/F64)
      * `Tok::{Binary,...}`   -> `type`      (substrate/representation types)
      * everything else       -> `keyword`   (declaration/control/runtime-vocabulary words)

    Never-silent: a `keyword()` body that cannot be located is a hard error, not an empty result.
    """
    m = re.search(
        r"pub fn keyword\(word: &str\) -> Option<Tok> \{(.*?)\n\}", token_rs, re.DOTALL
    )
    if not m:
        raise SystemExit(
            "drift: could not locate `pub fn keyword(...)` in token.rs (G2)"
        )
    body = m.group(1)

    buckets: dict[str, list[str]] = {
        "keyword": [],
        "type": [],
        "scalar": [],
        "strength": [],
    }
    # Each arm: "<word>" => Tok::<RHS>,   (the RHS may be `Strength(StrengthTok::Exact)` etc.)
    for word, rhs in re.findall(r'"([^"]+)"\s*=>\s*(Tok::[^,]+),', body):
        if rhs.startswith("Tok::Strength"):
            buckets["strength"].append(word)
        elif rhs.startswith("Tok::Scalar"):
            buckets["scalar"].append(word)
        elif rhs[len("Tok::") :] in TYPE_VARIANTS:
            buckets["type"].append(word)
        else:
            buckets["keyword"].append(word)

    for k in buckets:
        buckets[k] = sorted(set(buckets[k]))
    if not any(buckets.values()):
        raise SystemExit("drift: extracted an empty keyword set from token.rs (G2)")
    return buckets


def render_keywords_json(buckets: dict[str, list[str]]) -> str:
    """The canonical extracted snapshot — the artifact the drift gate diffs and the grammars build on."""
    doc = {
        "_generated_by": "tools/grammar/generate.py",
        "_source_of_truth": "crates/mycelium-l1/src/token.rs::keyword()",
        "_honesty": "keyword set + class buckets are lexer-derived (G2); scope names are the ratified RFC-0026 §3.2 table.",
        "classes": buckets,
    }
    return json.dumps(doc, indent=2) + "\n"


def _regex_alt(words: list[str]) -> str:
    """A `\\b`-anchored alternation matching exactly these whole words (longest-first, escaped)."""
    alts = "|".join(re.escape(w) for w in sorted(words, key=len, reverse=True))
    return rf"\b({alts})\b"


def render_tmlanguage(buckets: dict[str, list[str]]) -> str:
    """A minimal TextMate grammar using the ratified RFC-0026 §3.2 scope names."""
    patterns = []
    for cls in ("keyword", "type", "scalar", "strength"):
        words = buckets[cls]
        if not words:
            continue
        patterns.append(
            {
                "name": TM_SCOPES[cls],
                "match": _regex_alt(words),
            }
        )
    # Comments and binary/ternary literals are syntactic (not in keyword()). The binary literal allows
    # `_` digit separators, matching the lexer (lex_binary accepts '0'/'1'/'_').
    patterns.append({"name": TM_COMMENT, "match": r"//.*$"})
    patterns.append(
        {
            "name": TM_NUMERIC,
            "match": r"0b[01_]+|<[+\-0]+>",
        }
    )

    grammar = {
        "$schema": "https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json",
        "name": "Mycelium",
        "scopeName": "source.mycelium",
        "_generated_by": "tools/grammar/generate.py — DO NOT EDIT BY HAND",
        "_scope_names_status": "RATIFIED — RFC-0026 §3.2 (Accepted): standard TextMate names, lexer-derived buckets.",
        "patterns": patterns,
    }
    return json.dumps(grammar, indent=2) + "\n"


def render_tree_sitter(buckets: dict[str, list[str]]) -> str:
    """A tree-sitter `grammar.js` scaffold listing the lexer keywords. Full structure is follow-up."""
    all_words = sorted({w for words in buckets.values() for w in words})
    kw_rules = ",\n".join(f"      '{w}'" for w in all_words)
    return f"""// tree-sitter grammar for Mycelium — GENERATED by tools/grammar/generate.py. DO NOT EDIT.
//
// Status (RFC-0026, Accepted): the keyword set below is derived from the canonical lexer
// (crates/mycelium-l1/src/token.rs::keyword()) and kept in lockstep by `just drift-check`. This is a
// keyword-accurate SCAFFOLD; the full structural grammar (productions beyond the reserved-word set)
// is the community follow-up named in RFC-0026 §3.4. Highlight captures are in queries/highlights.scm.
module.exports = grammar({{
  name: 'mycelium',
  // The reserved-word set, kept in lockstep with the lexer by `just drift-check`.
  word: $ => $.identifier,
  rules: {{
    // Scaffold source rule — the full structural productions are the RFC-0026 §3.4 follow-up.
    source_file: $ => repeat(choice($.keyword, $.identifier)),
    identifier: $ => /[A-Za-z_][A-Za-z0-9_]*/,
    keyword: $ => choice(
{kw_rules}
    ),
  }},
}});
"""


def render_highlights_scm(buckets: dict[str, list[str]]) -> str:
    """tree-sitter highlight queries using the ratified RFC-0026 §3.2 capture names."""
    lines = [
        "; tree-sitter highlight queries for Mycelium — GENERATED by tools/grammar/generate.py.",
        "; Capture names are the ratified RFC-0026 §3.2 (Accepted) standard tree-sitter captures.",
        "",
    ]
    for cls in ("keyword", "type", "scalar", "strength"):
        words = buckets[cls]
        if not words:
            continue
        alt = " ".join(f'"{w}"' for w in words)
        lines.append(f"[{alt}] @{TS_CAPTURES[cls]}")
    return "\n".join(lines) + "\n"


# (relative path under tools/grammar) -> renderer
ARTIFACTS = {
    "keywords.json": render_keywords_json,
    "mycelium.tmLanguage.json": render_tmlanguage,
    "tree-sitter-mycelium/grammar.js": render_tree_sitter,
    "tree-sitter-mycelium/queries/highlights.scm": render_highlights_scm,
}


def generate(buckets: dict[str, list[str]]) -> dict[str, str]:
    """Render every artifact to its string content (the in-memory truth the gate diffs against)."""
    return {rel: render(buckets) for rel, render in ARTIFACTS.items()}


def write(out_dir: Path, rendered: dict[str, str]) -> None:
    for rel, content in rendered.items():
        path = out_dir / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")


def check(out_dir: Path, rendered: dict[str, str]) -> int:
    """Drift gate: committed artifacts must match a fresh regeneration. Exit 2 on any drift (G2)."""
    stale: list[str] = []
    for rel, content in rendered.items():
        path = out_dir / rel
        if not path.exists() or path.read_text(encoding="utf-8") != content:
            stale.append(rel)
    if stale:
        print(
            "drift: the committed editor grammars are stale vs the lexer keyword() table:"
        )
        for rel in stale:
            print(f"  - tools/grammar/{rel}")
        print(
            "fix: run `python3 tools/grammar/generate.py` and commit the result (G2)."
        )
        return 2
    print("grammar artifacts are current with the lexer keyword() table")
    return 0


def self_test(buckets: dict[str, list[str]]) -> int:
    """Offline sanity: known keywords land in the right buckets; generation is deterministic."""
    failures = []
    if "nodule" not in buckets["keyword"]:
        failures.append("`nodule` should be a keyword")
    if "Binary" not in buckets["type"]:
        failures.append("`Binary` should be a type")
    if "Exact" not in buckets["strength"]:
        failures.append("`Exact` should be a strength")
    if "F32" not in buckets["scalar"]:
        failures.append("`F32` should be a scalar")
    if generate(buckets) != generate(buckets):
        failures.append("generation is not deterministic")
    if failures:
        for f in failures:
            print(f"self-test FAIL: {f}")
        return 3
    print("grammar generator self-test passed (extraction + determinism)")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--check",
        action="store_true",
        help="drift gate: fail if committed artifacts are stale",
    )
    ap.add_argument(
        "--self-test",
        action="store_true",
        help="offline extraction/determinism sanity check",
    )
    ap.add_argument(
        "--output-dir",
        default=str(HERE),
        help="where artifacts live (default: tools/grammar/)",
    )
    args = ap.parse_args()

    buckets = extract_keywords(TOKEN_RS.read_text(encoding="utf-8"))
    out_dir = Path(args.output_dir)

    if args.self_test:
        return self_test(buckets)
    rendered = generate(buckets)
    if args.check:
        return check(out_dir, rendered)
    write(out_dir, rendered)
    total = sum(len(v) for v in buckets.values())
    print(
        f"wrote {len(rendered)} grammar artifact(s) from {total} lexer keywords -> {out_dir}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
