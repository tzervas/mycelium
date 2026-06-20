#!/usr/bin/env python3
"""Doc-status currency gate (offline).

Enforces the ratified decision-status lattice across the corpus's decision docs and
keeps the navigational index READMEs from drifting out of agreement with the
authoritative per-doc `Status` headers. Three passes, each cheap, deterministic, and
never-silent (every check AND every failure is printed with the file:line):

1. **Lattice validation** — parse every decision doc's authoritative `Status` header
   (the numbered decision docs `docs/rfcs/RFC-*.md`, `docs/adr/ADR-*.md`,
   `docs/notes/DN-*.md`, plus `docs/spec/stdlib/*.md`) and
   FAIL if its leading status token is not in the allowed lattice
   {Draft, Proposed, Preliminary, Accepted, Enacted, Superseded, Resolved}. A bare
   legacy compound `Accepted — Enacted` is FAILed as normalization-needed (the canonical
   spelling is the standalone `Enacted`, #236). Parenthetical/qualifier suffixes
   (`(scoped)`, `(framework)`, `(r4)`, `(needs-design)`, `(Rust-first half)`, a trailing
   `— <clarification>`) are allowed *after* a valid leading token.

2. **Nav-doc <-> header cross-check** — for the index READMEs that list each doc with a
   claimed status (`docs/rfcs/README.md`, `docs/adr/README.md`), assert the claimed
   status token matches the doc's authoritative header. A mismatch FAILs with the doc id
   + claimed-vs-authoritative. This is exactly the drift that left 8 stale RFC rows.

3. **Declared stale-phrase invariants** — read the committed manifest
   `tools/doc-status-invariants.yaml` of maintainer-DECLARED rules (e.g. "once every
   stdlib spec except runtime/self-hosting-readiness is Accepted-or-later, no nav README
   may say 'pending ratification'") and FAIL on any violation.

HONESTY (house-rule 1 / VR-5): this is a **Declared** line/regex heuristic — *source is
ground truth*. It parses the leading token of a `Status` header and the status cell of an
index table; it does not understand prose. The pass-3 rules are maintainer-Declared
decisions in the manifest, never inferred by the script. Use it to catch the named drift,
not as an authority on a doc's meaning.

Exit 0 = all current; exit 1 = at least one violation (each printed with a fix hint).
Skip-graceful: if PyYAML is absent, pass 3 is skipped with a printed warning (passes 1-2
still run); the shell wrapper additionally skips the whole gate when python3 is absent.
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# The ratified status lattice (#236): Draft/Proposed/Preliminary -> Accepted -> Enacted
# -> Superseded; notes -> Resolved. These are the only valid LEADING status tokens.
LATTICE = {
    "Draft",
    "Proposed",
    "Preliminary",
    "Accepted",
    "Enacted",
    "Superseded",
    "Resolved",
}

# Forward rank for the `all_status_at_least` precondition in pass 3. Resolved is the
# notes-track terminal of Accepted; it ranks with Accepted for ">=" comparisons.
LATTICE_RANK = {
    "Draft": 0,
    "Proposed": 0,
    "Preliminary": 0,
    "Accepted": 1,
    "Resolved": 1,
    "Enacted": 2,
    "Superseded": 3,
}

# The decision-doc globs pass 1 validates. RFC/ADR/DN-numbered decision docs and the
# per-module stdlib specs carry an authoritative `Status` on the ratified lattice. The
# notes dir is scoped to numbered `DN-*.md` decision notes (the same convention
# scripts/doc_currency.py indexes); its un-numbered reference/living docs
# (Lexicon-Reference, *-Forward-Compat, research-prompts, …) use their own vocabulary
# ('Living note', no Status row) and are out of scope for the lattice gate. Likewise the
# other docs/spec/*.md contracts use Ratified / ratified-skeleton / Living and are out
# of scope. Source is ground truth.
DECISION_GLOBS = [
    "docs/rfcs/RFC-*.md",
    "docs/adr/ADR-*.md",
    "docs/notes/DN-*.md",
    "docs/spec/stdlib/*.md",
]

# Files in those globs that are not themselves a single decision doc (index READMEs,
# the conformance template) — skipped by pass 1's header scan.
NON_DECISION_NAMES = {"README.md", "_TEMPLATE.md"}

# The legacy compound spelling pass 1 flags as normalization-needed.
LEGACY_COMPOUND = re.compile(r"Accepted\s*[—–-]{1,2}\s*Enacted", re.IGNORECASE)

STATUS_ROW = re.compile(r"^\|\s*\*\*Status\*\*\s*\|\s*(.*?)\s*\|\s*$")


def rel(path: Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


# --------------------------------------------------------------------------- parsing


def find_status_line(text: str) -> tuple[int, str] | None:
    """Return (1-based line number, raw status-cell text) of the authoritative
    `| **Status** | ... |` header row, or None if the doc has no such header."""
    for lineno, line in enumerate(text.splitlines(), 1):
        m = STATUS_ROW.match(line)
        if m:
            return lineno, m.group(1)
    return None


def leading_token(cell: str) -> str | None:
    """Extract the leading status token from a status-cell string. Strips markdown
    bold/emphasis and a leading word; returns the bare token (e.g. 'Accepted',
    'Enacted', 'Draft') or None if no alphabetic leading word is present.

    Examples:
      '**Accepted** (r4) ...'           -> 'Accepted'
      '**Enacted (r3)** (2026-...)'     -> 'Enacted'
      '**Draft (needs-design)** (...)'  -> 'Draft'
      'Proposed'                        -> 'Proposed'
      '**Draft / Resolved-as-capture**' -> 'Draft'
      '**Draft — direction ratified**'  -> 'Draft'
    """
    # Strip leading markdown emphasis markers and whitespace.
    s = cell.lstrip()
    s = re.sub(r"^[*_`]+", "", s).lstrip()
    m = re.match(r"([A-Za-z][A-Za-z-]*)", s)
    if not m:
        return None
    # Collapse a hyphenated compound like 'ratified-skeleton' to its first word so the
    # lattice check sees the leading lattice token, not the qualifier.
    return m.group(1).split("-")[0]


@dataclass(frozen=True)
class DocStatus:
    path: Path
    lineno: int
    cell: str
    token: str | None  # leading lattice token, or None if it cannot be parsed


def collect_decision_statuses() -> list[DocStatus]:
    """Authoritative status of every decision doc, in sorted path order."""
    out: list[DocStatus] = []
    seen: set[Path] = set()
    for pattern in DECISION_GLOBS:
        for path in sorted(REPO_ROOT.glob(pattern)):
            if path.name in NON_DECISION_NAMES or path in seen:
                continue
            seen.add(path)
            text = path.read_text(encoding="utf-8")
            found = find_status_line(text)
            if found is None:
                out.append(DocStatus(path, 0, "", None))
                continue
            lineno, cell = found
            out.append(DocStatus(path, lineno, cell, leading_token(cell)))
    return out


# ------------------------------------------------------------------ pass 1: lattice


def check_lattice(statuses: list[DocStatus], errors: list[str]) -> None:
    print("  pass 1 — lattice validation (every decision-doc Status header)")
    for ds in statuses:
        loc = f"{rel(ds.path)}:{ds.lineno}"
        if ds.lineno == 0:
            print(f"    fail  {rel(ds.path)} — no `| **Status** |` header row found")
            errors.append(f"{rel(ds.path)}: decision doc has no Status header")
            continue
        if LEGACY_COMPOUND.search(ds.cell):
            print(
                f"    fail  {loc} — legacy compound 'Accepted — Enacted'; "
                "normalize to the standalone `Enacted` token (#236)"
            )
            errors.append(
                f"{loc}: legacy compound 'Accepted — Enacted' (normalize to `Enacted`)"
            )
            continue
        if ds.token is None:
            print(f"    fail  {loc} — Status cell has no leading token: {ds.cell!r}")
            errors.append(f"{loc}: Status cell has no parseable leading token")
            continue
        if ds.token not in LATTICE:
            print(
                f"    fail  {loc} — leading token {ds.token!r} not in the lattice "
                f"{{{', '.join(sorted(LATTICE))}}}"
            )
            errors.append(
                f"{loc}: status token {ds.token!r} not in the allowed lattice"
            )
            continue
        print(f"    ok    {loc} — {ds.token}")


# -------------------------------------------------------------- pass 2: cross-check

# An index row: `| 0013 | Title | **Enacted** (...) | refs |` (RFC README) or
# `| 013 | Title | Accepted | Location |` (ADR README). We key the doc by its numeric id
# and compare the claimed leading token to the authoritative header token.
RFC_ROW = re.compile(r"^\|\s*(\d{4})\s*\|[^|]*\|\s*(.*?)\s*\|")
ADR_ROW = re.compile(r"^\|\s*(\d{3})\s*\|[^|]*\|\s*(.*?)\s*\|")


def _status_by_id(statuses: list[DocStatus], prefix: str) -> dict[str, DocStatus]:
    """Map e.g. '0013' -> its DocStatus, for docs named '<prefix>-<id>-...md'."""
    pat = re.compile(rf"{re.escape(prefix)}-(\d+)")
    out: dict[str, DocStatus] = {}
    for ds in statuses:
        m = pat.match(ds.path.name)
        if m:
            out[m.group(1)] = ds
    return out


def _cross_check_readme(
    readme: Path,
    row_re: re.Pattern[str],
    by_id: dict[str, DocStatus],
    label: str,
    errors: list[str],
) -> None:
    if not readme.exists():
        print(f"    skip  {rel(readme)} — index README not found")
        return
    for lineno, line in enumerate(readme.read_text(encoding="utf-8").splitlines(), 1):
        m = row_re.match(line)
        if not m:
            continue
        doc_id, claimed_cell = m.group(1), m.group(2)
        if doc_id not in by_id:
            continue  # a row for a doc that lives elsewhere (e.g. Foundation §8 ADRs)
        ds = by_id[doc_id]
        loc = f"{rel(readme)}:{lineno}"
        if LEGACY_COMPOUND.search(claimed_cell):
            print(
                f"    fail  {loc} — {label}-{doc_id} row uses legacy compound "
                "'Accepted — Enacted'; normalize to `Enacted`"
            )
            errors.append(
                f"{loc}: {label}-{doc_id} index row uses legacy compound "
                "'Accepted — Enacted'"
            )
            continue
        claimed = leading_token(claimed_cell)
        if claimed is None:
            print(f"    fail  {loc} — {label}-{doc_id} row has no status token")
            errors.append(f"{loc}: {label}-{doc_id} index row has no status token")
            continue
        if claimed != ds.token:
            print(
                f"    fail  {loc} — {label}-{doc_id} row claims {claimed!r} but the "
                f"authoritative header ({rel(ds.path)}:{ds.lineno}) is {ds.token!r}"
            )
            errors.append(
                f"{loc}: {label}-{doc_id} index row claims {claimed!r}, "
                f"authoritative header is {ds.token!r}"
            )
            continue
        print(f"    ok    {loc} — {label}-{doc_id} {claimed} (matches header)")


def check_cross(statuses: list[DocStatus], errors: list[str]) -> None:
    print("  pass 2 — nav-doc <-> authoritative-header cross-check")
    _cross_check_readme(
        REPO_ROOT / "docs" / "rfcs" / "README.md",
        RFC_ROW,
        _status_by_id(statuses, "RFC"),
        "RFC",
        errors,
    )
    _cross_check_readme(
        REPO_ROOT / "docs" / "adr" / "README.md",
        ADR_ROW,
        _status_by_id(statuses, "ADR"),
        "ADR",
        errors,
    )


# ------------------------------------------------ pass 3: declared stale-phrase rules


def _status_token_for(path: Path) -> str | None:
    if not path.exists():
        return None
    found = find_status_line(path.read_text(encoding="utf-8"))
    if found is None:
        return None
    return leading_token(found[1])


def _precondition_met(when: dict, errors: list[str]) -> tuple[bool, str]:
    """Evaluate an optional `when:` precondition. Returns (met, human-reason).

    Supports `all_status_at_least` (every non-excepted doc in the glob has reached the
    floor) and `any_status_at_least` (at least one has). A malformed clause fails the
    gate with a controlled message rather than a `KeyError` traceback.
    """
    mode = next(
        (k for k in ("all_status_at_least", "any_status_at_least") if k in when), None
    )
    if mode is None:
        return True, "no precondition"
    spec = when.get(mode) or {}
    glob, floor = spec.get("glob"), spec.get("floor")
    if not glob or not floor:
        errors.append(f"manifest: `{mode}` needs both 'glob' and 'floor'")
        return False, f"malformed `{mode}` (needs glob + floor)"
    floor_rank = LATTICE_RANK.get(floor)
    if floor_rank is None:
        errors.append(f"manifest: unknown floor status {floor!r} in when-clause")
        return False, f"unknown floor {floor!r}"
    except_rel = set(spec.get("except", []))
    ranked = [
        (rel(p), _status_token_for(p))
        for p in sorted(REPO_ROOT.glob(glob))
        if rel(p) not in except_rel and p.name not in NON_DECISION_NAMES
    ]
    below = [
        f"{r} ({t})" for r, t in ranked if LATTICE_RANK.get(t or "", -1) < floor_rank
    ]
    at_floor = [r for r, t in ranked if LATTICE_RANK.get(t or "", -1) >= floor_rank]
    if mode == "all_status_at_least":
        if below:
            return False, f"{len(below)} doc(s) below {floor}: {', '.join(below)}"
        return True, f"all {glob} (except carve-outs) >= {floor}"
    # any_status_at_least
    if at_floor:
        return True, f"{len(at_floor)} doc(s) of {glob} >= {floor}"
    return False, f"no {glob} doc has reached {floor} yet"


def check_invariants(errors: list[str]) -> None:
    print(
        "  pass 3 — declared stale-phrase invariants (tools/doc-status-invariants.yaml)"
    )
    manifest = REPO_ROOT / "tools" / "doc-status-invariants.yaml"
    if not manifest.exists():
        print(f"    skip  manifest not found: {rel(manifest)}")
        return
    try:
        import yaml  # noqa: PLC0415 — optional dep; degrade gracefully if absent
    except ImportError:
        print("    skip  PyYAML not installed — pass 3 skipped (run `just setup`)")
        return

    data = yaml.safe_load(manifest.read_text(encoding="utf-8")) or {}
    rules = data.get("rules", [])
    if not rules:
        print("    skip  manifest has no rules")
        return

    for rule in rules:
        rid = rule.get("id", "<unnamed>")
        phrase = rule.get("phrase")
        in_files = rule.get("in_files", [])
        when = rule.get("when")
        if not phrase or not in_files:
            errors.append(f"manifest rule {rid!r}: missing 'phrase' or 'in_files'")
            print(f"    fail  rule {rid!r} — malformed (needs phrase + in_files)")
            continue
        if when:
            met, reason = _precondition_met(when, errors)
            if not met:
                print(f"    skip  rule {rid!r} — precondition not met ({reason})")
                continue
            print(f"    note  rule {rid!r} — precondition met ({reason})")
        needle = phrase.lower()
        for rel_path in in_files:
            path = REPO_ROOT / rel_path
            if not path.exists():
                print(f"    skip  rule {rid!r} — target absent: {rel_path}")
                continue
            hit = False
            for lineno, line in enumerate(
                path.read_text(encoding="utf-8").splitlines(), 1
            ):
                if needle in line.lower():
                    hit = True
                    print(
                        f"    fail  {rel_path}:{lineno} — forbidden phrase "
                        f"{phrase!r} (rule {rid!r})"
                    )
                    errors.append(
                        f"{rel_path}:{lineno}: forbidden phrase {phrase!r} "
                        f"(rule {rid!r})"
                    )
            if not hit:
                print(f"    ok    {rel_path} — clean of {phrase!r} (rule {rid!r})")


# --------------------------------------------------------------------------- driver


def main() -> int:
    print("doc-status: lattice + nav cross-check + declared stale-phrase invariants")
    errors: list[str] = []
    statuses = collect_decision_statuses()
    check_lattice(statuses, errors)
    check_cross(statuses, errors)
    check_invariants(errors)

    print()
    if errors:
        print(f"doc-status: {len(errors)} violation(s):")
        for e in errors:
            print(f"  - {e}")
        return 1
    print("doc-status: all decision-doc statuses current and on the ratified lattice.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
