#!/usr/bin/env python3
"""Sugar-index generator (v0) — DN-38's Layered-Lowering Atlas realized as a generated artifact.

Renders the committed `docs/sugar-index/` (INDEX.md + index.json) from the HAND-AUTHORED
`tools/grammar/sugar.yaml` registry, and — the drift-guard mechanic (G2) — cross-checks that
registry against the SAME lexer keyword() extraction `tools/grammar/generate.py` already uses for
the editor grammars, so the sugar catalog can never silently diverge from the lexer:

  1. every keyword `crates/mycelium-l1/src/token.rs::keyword()` knows (the "keyword" bucket —
     declaration/control/runtime-vocabulary words; the type/scalar/strength buckets are base
     type/literal vocabulary, out of v0 scope — see `sugar.yaml`'s header) has EXACTLY one row in
     `sugar.yaml`'s `sugars:` list — a keyword with no row, or a row naming an unknown keyword,
     fails loudly (never a silent gap in either direction);
  2. each row's `status` (active/reserved) agrees with `generate.py`'s `STRUCTURAL_KEYWORDS`
     classification (the same empirically-verified-against-the-conformance-corpus set the editor
     grammars already trust) — a keyword that moves from reserved to active (or vice versa)
     without an updated `sugar.yaml` row fails loudly;
  3. each row's `token.rs:LINE` citation (embedded in its `grammar_rule` field) resolves — the
     cited line, read fresh from token.rs, must actually map that keyword to a `Tok::` variant.
     A line-number drift (token.rs edited, sugar.yaml not updated) fails loudly, not silently.

`glyph_sugars` (glyph-lexed sugars like DN-102's `?`, lexed outside `keyword()`) and
`not_lexed_sugars` (documented-but-not-yet-lexed names like `reveal`) are rendered but NOT part of
the keyword()-driven cross-check (see `sugar.yaml`'s scope note) — they have no `token.rs:LINE`
citation to validate against.

Every row also renders its `native_strategy` (M-1058 follow-up): the ratified DN-111
native-equivalence taxonomy value (`NativeEquivalent`/`IdiomaticRemapping`/`Approximation`/
`InteropBridge`, or `unclassified` where `sugar.yaml` honestly declines to guess — VR-5). This
generator only RENDERS the field verbatim from `sugar.yaml`; it does not classify or validate it
(classification lives in `sugar.yaml` itself, via `.claude/skills/native-translate/SKILL.md`).

HONESTY (VR-5, G2): this generated index is `Empirical`/`Declared` — a projection of the
hand-authored `sugar.yaml` registry (itself a curated projection of the source DNs + token.rs).
`sugar.yaml` + the DNs it cites + token.rs are ground truth; use this index to find where to
`Read`, not as an authoritative reference (mirrors `docs/api-index/`'s disclaimer).

Usage:
    python3 tools/grammar/sugar_index.py                 # (re)write docs/sugar-index/
    python3 tools/grammar/sugar_index.py --check          # drift gate: fail if stale (exit 2/3/4)
    python3 tools/grammar/sugar_index.py --self-test       # offline sanity; exit 3 on failure
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

try:
    import yaml
except (
    ImportError
):  # pragma: no cover — skip-graceful, mirrors the doc-index.sh tool-absent path
    yaml = None  # type: ignore[assignment]

# Repo-root-relative locations (this file lives at tools/grammar/sugar_index.py).
HERE = Path(__file__).resolve().parent
REPO_ROOT = HERE.parent.parent
SUGAR_YAML = HERE / "sugar.yaml"
TOKEN_RS = REPO_ROOT / "crates" / "mycelium-l1" / "src" / "token.rs"

# tools/grammar/ is already a package-less directory; import the sibling module by path so this
# script has no dependency ordering surprise (`python3 tools/grammar/sugar_index.py` run from
# anywhere still finds it — DRY: reuse generate.py's extraction rather than re-deriving it).
sys.path.insert(0, str(HERE))
import generate as grammar_gen  # noqa: E402  (after sys.path mutation, by necessity)

HONESTY_TAG = (
    "Empirical/Declared — a curated projection of tools/grammar/sugar.yaml (itself a projection "
    "of the source DNs + token.rs). sugar.yaml + the cited DNs + token.rs are ground truth; use "
    "this index to find where to Read, not as an authoritative reference."
)

TOKEN_RS_CITE_RE = re.compile(r"token\.rs:(\d+)")
# Matches a keyword() arm: `"word" => Tok::...,` — used to confirm a cited line really maps the
# expected word (mirrors generate.py's own extraction regex).
KEYWORD_ARM_RE = re.compile(r'"([^"]+)"\s*=>\s*Tok::')


class DriftError(SystemExit):
    """A never-silent (G2) drift failure — carries a specific exit code (see CLI docstring)."""


def load_registry(path: Path = SUGAR_YAML) -> dict:
    if yaml is None:
        raise DriftError(
            "pyyaml not installed — skip this gate (see scripts/checks/sugar-index.sh)"
        )
    if not path.exists():
        raise DriftError(f"drift: {path} is missing (G2)")
    doc = yaml.safe_load(path.read_text(encoding="utf-8"))
    for key in ("sugars", "glyph_sugars", "not_lexed_sugars"):
        if key not in doc or not isinstance(doc[key], list):
            raise DriftError(
                f"drift: sugar.yaml missing/malformed top-level list `{key}` (G2)"
            )
    return doc


def cross_check(registry: dict, token_rs_text: str) -> list[str]:
    """Return a list of human-readable drift failures (empty ⇒ clean). Never raises — the CLI
    layer decides how to fail (so --self-test can exercise this against synthetic fixtures)."""
    failures: list[str] = []
    buckets = grammar_gen.extract_keywords(token_rs_text)
    lexer_keywords = set(buckets["keyword"])
    active_keywords = lexer_keywords & grammar_gen.STRUCTURAL_KEYWORDS

    rows = registry["sugars"]
    row_by_kw: dict[str, dict] = {}
    for row in rows:
        kw = row.get("keyword")
        if kw in row_by_kw:
            failures.append(f"duplicate sugar.yaml row for keyword `{kw}`")
        row_by_kw[kw] = row

    # (1) coverage both directions — no lexer keyword missing a row, no row naming an unknown word.
    missing = sorted(lexer_keywords - row_by_kw.keys())
    for kw in missing:
        failures.append(
            f"lexer keyword `{kw}` (token.rs::keyword()) has no sugar.yaml row"
        )
    unknown = sorted(row_by_kw.keys() - lexer_keywords)
    for kw in unknown:
        failures.append(
            f"sugar.yaml row `{kw}` does not match any token.rs::keyword() entry"
        )

    # (2) status agreement (reuses generate.py's STRUCTURAL_KEYWORDS classification verbatim).
    for kw, row in row_by_kw.items():
        if kw not in lexer_keywords:
            continue  # already reported as `unknown` above
        expected = "active" if kw in active_keywords else "reserved"
        got = row.get("status")
        if got != expected:
            failures.append(
                f"`{kw}`: sugar.yaml status is `{got}` but token.rs/generate.py classifies it "
                f"`{expected}` (STRUCTURAL_KEYWORDS drift)"
            )

    # (3) token.rs:LINE citation resolves.
    token_lines = token_rs_text.splitlines()
    for kw, row in row_by_kw.items():
        grammar_rule = row.get("grammar_rule") or ""
        m = TOKEN_RS_CITE_RE.search(grammar_rule)
        if not m:
            failures.append(f"`{kw}`: grammar_rule has no `token.rs:LINE` citation")
            continue
        line_no = int(m.group(1))
        if not (1 <= line_no <= len(token_lines)):
            failures.append(f"`{kw}`: cited token.rs:{line_no} is out of range")
            continue
        cited_line = token_lines[line_no - 1]
        arm = KEYWORD_ARM_RE.search(cited_line)
        if not arm or arm.group(1) != kw:
            failures.append(
                f'`{kw}`: token.rs:{line_no} does not map "{kw}" => Tok::… '
                f"(got: {cited_line.strip()!r}) — citation is stale, re-cite the current line"
            )

    return failures


def _row_sort_key(row: dict) -> str:
    return str(row.get("keyword") or row.get("glyph") or row.get("name") or "")


def render_json(registry: dict) -> str:
    payload = {
        "generated": HONESTY_TAG,
        "source": "tools/grammar/sugar.yaml",
        "sugars": sorted(registry["sugars"], key=_row_sort_key),
        "glyph_sugars": sorted(registry["glyph_sugars"], key=_row_sort_key),
        "not_lexed_sugars": sorted(registry["not_lexed_sugars"], key=_row_sort_key),
    }
    return json.dumps(payload, indent=2, ensure_ascii=False) + "\n"


def _table(rows: list[dict], id_field: str, id_header: str) -> list[str]:
    lines = [
        f"| {id_header} | Status | Grammar rule | Lowering target | Defining doc | Build status | Native strategy |",
        "|---|---|---|---|---|---|---|",
    ]
    for row in sorted(rows, key=_row_sort_key):
        rid = f"`{row.get(id_field)}`"
        status = row.get("status", "—")
        grammar_rule = (
            str(row.get("grammar_rule") or "—").replace("|", "\\|").replace("\n", " ")
        )
        lowering = (
            str(row.get("lowering_target") or "—")
            .replace("|", "\\|")
            .replace("\n", " ")
        )
        doc = str(row.get("defining_doc") or "—").replace("|", "\\|")
        build = row.get("build_status", "—")
        native_strategy = str(row.get("native_strategy") or "—").replace("|", "\\|")
        lines.append(
            f"| {rid} | {status} | {grammar_rule} | {lowering} | {doc} | {build} | {native_strategy} |"
        )
    lines.append("")
    return lines


def render_markdown(registry: dict) -> str:
    lines: list[str] = []
    lines.append("# Mycelium Sugar Index — the surface-sugar / lowering catalog (v0)")
    lines.append("")
    lines.append(f"> **Honesty:** `{HONESTY_TAG}`")
    lines.append(
        "> Realizes DN-38 §6's per-feature Lowering Map as a generated artifact. Source of "
        "truth: `tools/grammar/sugar.yaml` (hand-authored) + `crates/mycelium-l1/src/token.rs` "
        "(mechanically cross-checked)."
    )
    lines.append(
        "> **Native strategy** (M-1058 follow-up, DN-111): the ratified DN-111 taxonomy — "
        "`NativeEquivalent` (alias Adaptation) · `IdiomaticRemapping` (alias Solution) · "
        "`Approximation` · `InteropBridge` (alias Bridge), or `unclassified` where no row exists "
        "to classify yet (a `Gap`/superseded keyword) — never a fabricated guess (VR-5). See "
        "`.claude/skills/native-translate/SKILL.md` for the classification procedure."
    )
    lines.append("")
    lines.append(
        "## Sugars (identifier keywords — `token.rs::keyword()`, cross-checked against "
        "`generate.py`'s `STRUCTURAL_KEYWORDS`)"
    )
    lines.append("")
    lines += _table(registry["sugars"], "keyword", "Keyword")
    lines.append(
        "## Glyph sugars (single-token, lexed outside `keyword()` — not part of the mechanical "
        "cross-check)"
    )
    lines.append("")
    lines += _table(registry["glyph_sugars"], "glyph", "Glyph")
    lines.append(
        "## Not-yet-lexed sugars (documented, no token.rs entry — not part of the mechanical "
        "cross-check)"
    )
    lines.append("")
    lines += _table(registry["not_lexed_sugars"], "name", "Name")
    return "\n".join(lines).rstrip() + "\n"


ARTIFACTS = {
    "INDEX.md": render_markdown,
    "index.json": render_json,
}


def generate(registry: dict) -> dict[str, str]:
    return {rel: render(registry) for rel, render in ARTIFACTS.items()}


def write(out_dir: Path, rendered: dict[str, str]) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    for rel, content in rendered.items():
        (out_dir / rel).write_text(content, encoding="utf-8")


def check(out_dir: Path, rendered: dict[str, str]) -> int:
    stale = []
    for rel, content in rendered.items():
        path = out_dir / rel
        if not path.exists() or path.read_text(encoding="utf-8") != content:
            stale.append(f"docs/sugar-index/{rel}")
    if stale:
        print("drift: the committed sugar index is stale vs a fresh regeneration:")
        for rel in stale:
            print(f"  - {rel}")
        print(
            "fix: run `python3 tools/grammar/sugar_index.py` and commit the result (G2)."
        )
        return 2
    print("docs/sugar-index/ is current with tools/grammar/sugar.yaml")
    return 0


def self_test() -> int:
    """Offline sanity: determinism + a synthetic drift-guard demo (never touches committed state).

    Never-silent (G2): failure mode text is printed for every case, including the deliberately
    synthetic ones, so a reader can see the drift-guard actually fires rather than trusting it by
    assertion alone.
    """
    failures: list[str] = []

    if yaml is None:
        print("self-test SKIP: pyyaml not installed")
        return 0

    registry = load_registry()
    token_rs_text = TOKEN_RS.read_text(encoding="utf-8")

    # --- Real cross-check must be clean today (the committed sugar.yaml is expected current) ---
    real_failures = cross_check(registry, token_rs_text)
    if real_failures:
        failures.append(
            "the COMMITTED sugar.yaml fails its own cross-check (regenerate/fix it): "
            + "; ".join(real_failures)
        )

    # --- Determinism: render twice, compare byte-for-byte ---
    if generate(registry) != generate(registry):
        failures.append("generation is not deterministic")

    # --- Synthetic drift-guard demo (1): a lexer keyword with no sugar.yaml row must fail loudly.
    synthetic_token_rs = token_rs_text.replace(
        '"nodule" => Tok::Nodule,',
        '"nodule" => Tok::Nodule,\n        "totally_new_kw" => Tok::Nodule,',
        1,
    )
    demo_missing = cross_check(registry, synthetic_token_rs)
    if not any(
        "totally_new_kw" in f and "no sugar.yaml row" in f for f in demo_missing
    ):
        failures.append(
            "drift-guard demo (missing row) did NOT fire — a lexer keyword with no registry "
            "row must fail loudly"
        )
    else:
        print(f"drift-guard demo (missing row) fired as expected: {demo_missing}")

    # --- Synthetic drift-guard demo (2): a stale token.rs:LINE citation must fail loudly.
    # Rewrite whatever `token.rs:N` the object row currently cites (not a hardcoded line number —
    # those drift whenever token.rs::keyword() is edited and sugar.yaml is re-cited; the demo must
    # stay valid after a re-cite pass, not pin a historical line).
    import copy
    import re as _re

    tampered = copy.deepcopy(registry)
    object_row = None
    for row in tampered["sugars"]:
        if row["keyword"] == "object":
            object_row = row
            break
    if object_row is None:
        failures.append(
            "drift-guard demo (stale citation) setup failed — no `object` sugar.yaml row"
        )
    else:
        new_rule, n_sub = _re.subn(
            r"token\.rs:\d+", "token.rs:1", object_row["grammar_rule"], count=1
        )
        if n_sub != 1:
            failures.append(
                "drift-guard demo (stale citation) setup failed — object grammar_rule has no "
                f"`token.rs:LINE` citation to tamper (got: {object_row['grammar_rule']!r})"
            )
        else:
            object_row["grammar_rule"] = new_rule
            demo_stale = cross_check(tampered, token_rs_text)
            if not any(
                "object" in f and ("stale" in f or "does not map" in f)
                for f in demo_stale
            ):
                failures.append(
                    "drift-guard demo (stale citation) did NOT fire — a stale token.rs:LINE "
                    "citation must fail loudly"
                )
            else:
                print(
                    f"drift-guard demo (stale citation) fired as expected: {demo_stale}"
                )

    if failures:
        for f in failures:
            print(f"self-test FAIL: {f}")
        return 3
    print(
        "sugar_index self-test passed (real registry clean · determinism · drift-guard demos)"
    )
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument(
        "--check",
        action="store_true",
        help="drift gate: fail if committed index is stale",
    )
    ap.add_argument(
        "--self-test",
        action="store_true",
        help="offline sanity (never touches committed state)",
    )
    ap.add_argument(
        "--output-dir",
        default=str(REPO_ROOT / "docs" / "sugar-index"),
        help="where the artifacts live (default: docs/sugar-index/)",
    )
    args = ap.parse_args()

    if yaml is None:
        print(
            "sugar-index: pyyaml not installed — skipping (see scripts/checks/sugar-index.sh)"
        )
        return 0

    if args.self_test:
        return self_test()

    registry = load_registry()
    token_rs_text = TOKEN_RS.read_text(encoding="utf-8")
    failures = cross_check(registry, token_rs_text)
    if failures:
        print(
            "drift: tools/grammar/sugar.yaml fails its cross-check against token.rs (G2):"
        )
        for f in failures:
            print(f"  - {f}")
        print(
            "fix: update tools/grammar/sugar.yaml to match token.rs::keyword(), then re-run."
        )
        return 4

    rendered = generate(registry)
    out_dir = Path(args.output_dir)
    if args.check:
        return check(out_dir, rendered)
    write(out_dir, rendered)
    print(
        f"wrote {len(rendered)} sugar-index artifact(s) from {len(registry['sugars'])} rows -> {out_dir}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
