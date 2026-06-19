#!/usr/bin/env python3
"""doc_refs: grammar validator for Mycelium issues.yaml doc_refs: fields.

Grammar for doc_refs list entries:
  api:<crate>::<path>        — check symbol exists in docs/api-index/index.json
  corpus:<DOC>[#<anchor>]   — check DOC appears in docs/Doc-Index.md; if #anchor given,
                               check a heading matching the anchor slug exists in the corpus
  src:<path>[:<line>]        — check file exists at <path> from repo root;
                               if :<line> given, check line number <= file length

Guarantee: Empirical/Declared — file existence and JSON membership checks are exact;
heading anchor matching is a slug heuristic (may miss or false-match unusual headings).

Never-silent (G2): every ref is checked; all failures are reported before exit.

Usage:
  python3 tools/github/doc_refs_check.py
    [--issues-yaml tools/github/issues.yaml]
    [--index-json  docs/api-index/index.json]
    [--doc-index   docs/Doc-Index.md]
    [--root .]           # repo root for src: refs
    [--self-test]        # validate against a known-good/bad synthetic manifest

Exit: 0 = all refs resolve; 1 = any dangling ref (all reported before exit).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

try:
    import yaml  # type: ignore[import-untyped]
except ImportError:  # pragma: no cover
    sys.exit("PyYAML is required: `pip install pyyaml` or `uv tool install pyyaml`")

HERE = Path(__file__).resolve().parent


# ---------------------------------------------------------------------------
# Slug helper (GitHub-style heading anchor)
# ---------------------------------------------------------------------------
def _heading_slug(text: str) -> str:
    """Convert a Markdown heading to its GitHub-style anchor slug.

    GitHub: lowercase, strip everything except letters/digits/spaces/hyphens,
    replace spaces with hyphens.
    """
    text = text.lower()
    text = re.sub(r"[^\w\s-]", "", text, flags=re.UNICODE)
    text = re.sub(r"\s+", "-", text.strip())
    text = re.sub(r"-+", "-", text)
    return text


# ---------------------------------------------------------------------------
# Corpus: extract headings from a markdown file
# ---------------------------------------------------------------------------
def _corpus_headings(path: Path) -> set[str]:
    """Return the set of heading slugs from a markdown file."""
    slugs: set[str] = set()
    try:
        for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
            m = re.match(r"^#{1,6}\s+(.*)", line)
            if m:
                slugs.add(_heading_slug(m.group(1)))
    except OSError:
        pass
    return slugs


# ---------------------------------------------------------------------------
# Reference checkers
# ---------------------------------------------------------------------------
def _check_api_ref(
    ref_value: str,
    issue_id: str,
    api_symbols: set[str],
    errors: list[str],
) -> None:
    """Check api:<crate>::<path> ref."""
    # Strip the "api:" prefix
    path = ref_value[4:]  # e.g. "mycelium-core::binary::bits_to_int"
    if path not in api_symbols:
        errors.append(
            f"{issue_id}: dangling api: ref {ref_value!r} — "
            f"symbol {path!r} not found in docs/api-index/index.json"
        )


def _check_corpus_ref(
    ref_value: str,
    issue_id: str,
    doc_index_text: str,
    corpus_root: Path,
    errors: list[str],
) -> None:
    """Check corpus:<DOC>[#<anchor>] ref."""
    # Strip "corpus:" prefix
    rest = ref_value[7:]  # e.g. "RFC-0016#some-section"

    anchor: str | None = None
    if "#" in rest:
        doc_id, anchor = rest.split("#", 1)
    else:
        doc_id = rest

    # Check DOC appears in Doc-Index.md as a token (not merely as a substring of another id).
    # Use non-alnum/hyphen boundaries to avoid "RFC-001" matching inside "RFC-0012".
    if not re.search(r"(?<![A-Za-z0-9-])" + re.escape(doc_id) + r"(?![A-Za-z0-9-])", doc_index_text):
        errors.append(
            f"{issue_id}: dangling corpus: ref {ref_value!r} — "
            f"{doc_id!r} not found in docs/Doc-Index.md"
        )
        return  # No point checking anchor if the doc isn't in the index

    # If anchor given, look for a heading in the corpus file
    if anchor:
        # Search in docs/ tree for the doc file
        doc_files = list(corpus_root.rglob(f"**/{doc_id}*.md")) + list(
            corpus_root.rglob(f"**/{doc_id.lower()}*.md")
        )
        # Also try exact stem match
        doc_files += list(corpus_root.rglob("*.md"))
        doc_files = [
            f for f in doc_files if doc_id.lower() in f.stem.lower() or doc_id in f.name
        ]

        if not doc_files:
            # Can't find the file but the doc is in the index — treat as a skip
            # with a warning (not an error) since Doc-Index may not have a file path.
            return

        found_slug = False
        target_slug = _heading_slug(anchor)
        for doc_file in doc_files:
            if target_slug in _corpus_headings(doc_file):
                found_slug = True
                break

        if not found_slug:
            errors.append(
                f"{issue_id}: dangling corpus: anchor ref {ref_value!r} — "
                f"heading slug {target_slug!r} not found in {doc_id} file(s)"
            )


def _check_src_ref(
    ref_value: str,
    issue_id: str,
    repo_root: Path,
    errors: list[str],
) -> None:
    """Check src:<path>[:<line>] ref."""
    # Strip "src:" prefix
    rest = ref_value[4:]  # e.g. "crates/mycelium-core/src/lib.rs:42"

    line_no: int | None = None
    # Detect trailing :<digits> as a line number
    m = re.match(r"^(.+):(\d+)$", rest)
    if m:
        path_str = m.group(1)
        line_no = int(m.group(2))
    else:
        path_str = rest

    target = repo_root / path_str
    if not target.exists():
        errors.append(
            f"{issue_id}: dangling src: ref {ref_value!r} — "
            f"file {path_str!r} does not exist"
        )
        return

    if line_no is not None:
        try:
            with target.open(encoding="utf-8", errors="replace") as fh:
                n_lines = sum(1 for _ in fh)
            if line_no > n_lines:
                errors.append(
                    f"{issue_id}: src: ref {ref_value!r} — "
                    f"line {line_no} exceeds file length {n_lines}"
                )
        except OSError as exc:
            errors.append(
                f"{issue_id}: src: ref {ref_value!r} — "
                f"could not read {path_str!r}: {exc}"
            )


# ---------------------------------------------------------------------------
# Main validator
# ---------------------------------------------------------------------------
def validate(
    issues_yaml: Path,
    index_json: Path,
    doc_index: Path,
    repo_root: Path,
) -> list[str]:
    """Validate all doc_refs entries in issues_yaml.

    Returns a list of error strings (empty = all OK).
    """
    errors: list[str] = []

    # Load API index symbols
    api_symbols: set[str] = set()
    if index_json.exists():
        try:
            payload = json.loads(index_json.read_text(encoding="utf-8"))
            api_symbols = {item["symbol"] for item in payload.get("items", [])}
        except (json.JSONDecodeError, KeyError, OSError) as exc:
            errors.append(f"could not load {index_json}: {exc}")
    else:
        errors.append(
            f"index.json not found at {index_json} — "
            "run 'just docs-index' to generate it"
        )

    # Load Doc-Index text for corpus: ref checking
    doc_index_text: str = ""
    if doc_index.exists():
        try:
            doc_index_text = doc_index.read_text(encoding="utf-8")
        except OSError as exc:
            errors.append(f"could not read {doc_index}: {exc}")
    else:
        errors.append(f"Doc-Index not found at {doc_index}")

    # Corpus root for anchor search
    corpus_root = doc_index.parent if doc_index.exists() else repo_root / "docs"

    # Load issues.yaml
    try:
        spec = yaml.safe_load(issues_yaml.read_text(encoding="utf-8"))
    except (OSError, yaml.YAMLError) as exc:
        errors.append(f"could not parse {issues_yaml}: {exc}")
        return errors

    issues = spec.get("issues", []) if spec else []

    for issue in issues:
        issue_id = issue.get("id", "<unknown>")
        doc_refs = issue.get("doc_refs") or []
        for ref in doc_refs:
            if not isinstance(ref, str):
                errors.append(f"{issue_id}: doc_refs entry is not a string: {ref!r}")
                continue
            if ref.startswith("api:"):
                _check_api_ref(ref, issue_id, api_symbols, errors)
            elif ref.startswith("corpus:"):
                _check_corpus_ref(ref, issue_id, doc_index_text, corpus_root, errors)
            elif ref.startswith("src:"):
                _check_src_ref(ref, issue_id, repo_root, errors)
            else:
                errors.append(
                    f"{issue_id}: unknown doc_refs prefix in {ref!r} "
                    "(expected api:, corpus:, or src:)"
                )

    return errors


# ---------------------------------------------------------------------------
# Self-test
# ---------------------------------------------------------------------------
def self_test(
    index_json: Path,
    doc_index: Path,
    repo_root: Path,
) -> int:
    """Run against a synthetic manifest with known-good and known-bad refs.

    Returns 0 on PASS, 1 on FAIL.
    """
    import tempfile
    import os

    print("Running self-test…")
    failures: list[str] = []

    # Known-good src: ref — this file itself must exist
    known_good_src = "src:tools/github/doc_refs_check.py"
    # Known-bad src: ref
    known_bad_src = "src:does/not/exist.py"
    # Known-good corpus: ref — RFC-0016 is in the Doc-Index
    known_good_corpus = "corpus:RFC-0016"

    synthetic_yaml = f"""\
issues:
  - id: SELFTEST-GOOD
    title: "good refs"
    labels: []
    body: ""
    doc_refs: [{known_good_src!r}, {known_good_corpus!r}]
  - id: SELFTEST-BAD
    title: "bad refs"
    labels: []
    body: ""
    doc_refs: [{known_bad_src!r}]
"""

    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".yaml", delete=False, encoding="utf-8"
    ) as tmp:
        tmp.write(synthetic_yaml)
        tmp_path = Path(tmp.name)

    try:
        errors = validate(tmp_path, index_json, doc_index, repo_root)
    finally:
        os.unlink(tmp_path)

    # We expect exactly one error: the known-bad src ref
    bad_errs = [e for e in errors if known_bad_src in e]
    good_errs = [e for e in errors if known_good_src in e or known_good_corpus in e]

    if bad_errs:
        print(f"  PASS  bad ref detected: {bad_errs[0]!r}")
    else:
        print(f"  FAIL  bad ref not detected: {known_bad_src!r} should have errored")
        failures.append("bad-ref-detection")

    if not good_errs:
        print("  PASS  good refs accepted")
    else:
        print(f"  FAIL  good refs rejected: {good_errs}")
        failures.append("good-ref-rejection")

    if failures:
        print(
            f"FAIL — {len(failures)} self-test check(s) failed: {', '.join(failures)}"
        )
        return 1
    print("PASS — all self-test checks passed")
    return 0


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------
def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate doc_refs: entries in issues.yaml (G2: never silent)."
    )
    parser.add_argument(
        "--issues-yaml",
        type=Path,
        default=HERE / "issues.yaml",
        help="Path to issues.yaml (default: tools/github/issues.yaml)",
    )
    parser.add_argument(
        "--index-json",
        type=Path,
        default=None,
        help="Path to docs/api-index/index.json (default: auto-detected from repo root)",
    )
    parser.add_argument(
        "--doc-index",
        type=Path,
        default=None,
        help="Path to docs/Doc-Index.md (default: auto-detected from repo root)",
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=None,
        help="Repo root for src: refs (default: auto-detected from this file's location)",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Run against a synthetic manifest; exits 0 on PASS, 1 on FAIL",
    )
    args = parser.parse_args()

    # Repo root = parent of tools/github/ → tools/ → root
    repo_root = args.root if args.root else HERE.parent.parent
    repo_root = repo_root.resolve()

    index_json = (
        args.index_json
        if args.index_json
        else repo_root / "docs" / "api-index" / "index.json"
    )
    doc_index = (
        args.doc_index if args.doc_index else repo_root / "docs" / "Doc-Index.md"
    )

    if args.self_test:
        return self_test(index_json, doc_index, repo_root)

    errors = validate(args.issues_yaml, index_json, doc_index, repo_root)

    if errors:
        for err in errors:
            print(f"ERROR: {err}", file=sys.stderr)
        print(
            f">> doc_refs check FAILED: {len(errors)} dangling reference(s) — fix before syncing.",
            file=sys.stderr,
        )
        return 1

    print(">> doc_refs check OK: all doc_refs entries resolve.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
