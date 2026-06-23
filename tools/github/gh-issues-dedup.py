#!/usr/bin/env python3
"""gh-issues-dedup.py — detect and reconcile DUPLICATE GitHub issues (safety net).

``gh-issues-sync.py`` matches by idmap-number-then-title and is *designed* to never
duplicate (RECONCILE.md). This is the safety net for when that contract slips — e.g. a
sync re-run before ``idmap.tsv`` was committed created twins (historically #126-129
duplicated the canonical M-302/M-330/M-342/M-348 — see the idmap.tsv note).

It groups issues by the **task-id encoded in the title** (``M-xxx`` / ``Exx-x``) and by
**normalized title**, picks the lowest-numbered (earliest) as the **canonical**, and
reports the rest as duplicates. **DRY-RUN by default**; ``--apply`` closes the
non-canonical *open task-id* twins (never the canonical), and ``--fix-idmap`` re-anchors
``idmap.tsv`` rows to the canonical number.

Honesty / safety (G2): every detection, action, and skip is printed; an unparseable or
ambiguous case (title-only match, or a CLOSED canonical) is **reported, never auto-acted
on**; nothing is guessed. ``--self-test`` exercises the pure logic offline (no network).

Usage::

    python tools/github/gh-issues-dedup.py                  # dry-run report
    python tools/github/gh-issues-dedup.py --apply          # close duplicate twins
    python tools/github/gh-issues-dedup.py --apply --fix-idmap
    python tools/github/gh-issues-dedup.py --self-test      # offline logic test
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

REPO = "tzervas/mycelium"
IDMAP = Path(__file__).resolve().parent / "idmap.tsv"
TASK_ID_RE = re.compile(r"^\s*(E\d+-\d+|M-\d+)\b")


def parse_task_id(title: str) -> str | None:
    """The leading ``M-xxx`` / ``Exx-x`` task-id of an issue title, or ``None``."""
    m = TASK_ID_RE.match(title or "")
    return m.group(1) if m else None


def norm_title(title: str) -> str:
    """Whitespace-collapsed, lower-cased title for same-title matching."""
    return " ".join((title or "").split()).lower()


def find_duplicate_sets(issues: list[dict]) -> tuple[list, list]:
    """PURE. Group ``issues`` (dicts with ``number``/``title``/``state``).

    Returns ``(id_dups, title_dups)``; each is a list of ``(key, canonical, [dup, ...])``
    where ``canonical`` is the lowest-numbered issue. Only sets with more than one member
    are returned. ``title_dups`` excludes sets already covered by a task-id set, so a
    duplicate is reported once. No I/O — directly unit-testable.
    """
    by_id: dict[str, list] = defaultdict(list)
    by_title: dict[str, list] = defaultdict(list)
    for it in issues:
        tid = parse_task_id(it["title"])
        if tid:
            by_id[tid].append(it)
        by_title[norm_title(it["title"])].append(it)

    def split(group: dict[str, list]) -> list:
        out = []
        for key, items in sorted(group.items()):
            if len(items) < 2:
                continue
            ordered = sorted(items, key=lambda i: i["number"])
            out.append((key, ordered[0], ordered[1:]))
        return out

    id_dups = split(by_id)
    id_keys = {k for k, _, _ in id_dups}
    title_dups = [
        (k, c, d)
        for (k, c, d) in split(by_title)
        if parse_task_id(c["title"]) not in id_keys
    ]
    return id_dups, title_dups


def load_idmap(path: Path) -> dict[str, int]:
    """``task_id -> issue_number`` from ``idmap.tsv`` (comments/blank lines skipped)."""
    mapping: dict[str, int] = {}
    if not path.exists():
        return mapping
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) >= 2 and parts[1].isdigit():
            mapping[parts[0]] = int(parts[1])
    return mapping


def gh_json(args: list[str]) -> list:
    """Run ``gh <args>`` and parse stdout as JSON (raises on a non-zero exit)."""
    out = subprocess.run(["gh", *args], capture_output=True, text=True, check=True)
    return json.loads(out.stdout or "[]")


def list_issues(repo: str) -> list[dict]:
    """All issues (open + closed) via ``gh issue list`` — excludes PRs by construction."""
    return gh_json(
        [
            "issue",
            "list",
            "--repo",
            repo,
            "--state",
            "all",
            "--limit",
            "2000",
            "--json",
            "number,title,state,createdAt",
        ]
    )


def close_issue(repo: str, number: int, comment: str) -> None:
    """Close ``#number`` as not-planned with an explanatory ``comment``."""
    subprocess.run(
        [
            "gh",
            "issue",
            "close",
            str(number),
            "--repo",
            repo,
            "--reason",
            "not planned",
            "--comment",
            comment,
        ],
        check=True,
    )


def _reanchor_idmap(path: Path, reanchor: dict[str, int]) -> None:
    """Rewrite ``idmap.tsv`` rows so each ``task_id`` in ``reanchor`` maps to its number."""
    out = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip() and not line.lstrip().startswith("#"):
            parts = line.split("\t")
            if len(parts) >= 2 and parts[0] in reanchor:
                parts[1] = str(reanchor[parts[0]])
                line = "\t".join(parts)
        out.append(line)
    path.write_text("\n".join(out) + "\n", encoding="utf-8")


def report_and_act(repo: str, apply: bool, fix_idmap: bool) -> int:
    """List issues, report duplicate sets, and (with ``apply``) close the open twins."""
    issues = list_issues(repo)
    idmap = load_idmap(IDMAP)
    id_dups, title_dups = find_duplicate_sets(issues)
    open_n = sum(1 for i in issues if i["state"].upper() == "OPEN")
    print(
        f">> {len(issues)} issue(s) ({open_n} open) on {repo}; "
        f"{len(id_dups)} task-id duplicate set(s), {len(title_dups)} title-only set(s)."
    )

    actions: list[tuple[str, int, int]] = []
    for key, canon, dups in id_dups:
        canon_open = canon["state"].upper() == "OPEN"
        mapped = idmap.get(key)
        anchor = f" [idmap → #{mapped}]" if mapped else " [idmap: unmapped]"
        print(f"\n  task-id {key}: canonical #{canon['number']}{anchor}")
        for d in dups:
            state = d["state"].upper()
            print(f"    duplicate #{d['number']} ({state}): {d['title']}")
            if state == "OPEN" and canon_open:
                actions.append((key, canon["number"], d["number"]))
            elif state == "OPEN":
                print(
                    "    canonical is CLOSED — not auto-closing; review manually (G2)"
                )
        if mapped and mapped != canon["number"]:
            print(
                f"    NOTE idmap anchors {key} to #{mapped}, not canonical "
                f"#{canon['number']} — re-anchor with --fix-idmap"
            )

    for _key, canon, dups in title_dups:
        print(
            f"\n  same-title (no task-id): canonical #{canon['number']}: {canon['title']}"
        )
        for d in dups:
            print(f"    duplicate #{d['number']} ({d['state'].upper()})")
        print("    (title-only match — NOT auto-closed; review manually, G2)")

    if not id_dups and not title_dups:
        print("   no duplicates detected — clean.")
        return 0

    if not apply:
        print(
            f"\n>> DRY-RUN: {len(actions)} open task-id duplicate(s) would be closed. "
            "Re-run with --apply to act."
        )
        return 0

    print(f"\n>> --apply: closing {len(actions)} open duplicate(s)…")
    for key, canon_n, dup_n in actions:
        comment = (
            f"Duplicate of #{canon_n} (task `{key}`). Closed by gh-issues-dedup.py — "
            f"the canonical issue is #{canon_n}."
        )
        close_issue(repo, dup_n, comment)
        print(f"   closed #{dup_n} (dup of #{canon_n}, {key})")

    if fix_idmap:
        reanchor = {
            k: c["number"]
            for k, c, _ in id_dups
            if idmap.get(k) and idmap[k] != c["number"] and c["state"].upper() == "OPEN"
        }
        if reanchor:
            _reanchor_idmap(IDMAP, reanchor)
            for k, n in reanchor.items():
                print(f"   idmap: re-anchored {k} → #{n}")
    return 0


def _self_test() -> int:
    """Offline check of the pure grouping + canonical-selection logic."""
    sample = [
        {"number": 87, "title": "M-302 — foo", "state": "OPEN"},
        {"number": 126, "title": "M-302 — foo", "state": "OPEN"},
        {"number": 50, "title": "E13-1 (epic) — bar", "state": "OPEN"},
        {"number": 9, "title": "Random note", "state": "OPEN"},
        {"number": 12, "title": "random   note", "state": "CLOSED"},
    ]
    id_dups, title_dups = find_duplicate_sets(sample)
    assert len(id_dups) == 1, id_dups
    key, canon, dups = id_dups[0]
    assert key == "M-302" and canon["number"] == 87 and dups[0]["number"] == 126
    assert parse_task_id("M-700 — x") == "M-700"
    assert parse_task_id("E9-1 (epic) — y") == "E9-1"
    assert parse_task_id("no id here") is None
    assert any(k == "random note" for k, _, _ in title_dups), title_dups
    print("self-test OK: grouping · canonical selection · title-only detection")
    return 0


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(
        description="Detect/reconcile duplicate GitHub issues (dry-run by default)."
    )
    p.add_argument(
        "--apply",
        action="store_true",
        help="close non-canonical OPEN task-id duplicates (default: dry-run report)",
    )
    p.add_argument(
        "--fix-idmap",
        action="store_true",
        help="with --apply, re-anchor idmap.tsv rows to the canonical number",
    )
    p.add_argument("--repo", default=REPO, help=f"owner/name (default {REPO})")
    p.add_argument(
        "--self-test",
        action="store_true",
        help="run the offline logic test; no network",
    )
    a = p.parse_args(argv)
    if a.self_test:
        return _self_test()
    return report_and_act(a.repo, a.apply, a.fix_idmap)


if __name__ == "__main__":
    sys.exit(main())
