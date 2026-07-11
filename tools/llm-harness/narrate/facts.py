#!/usr/bin/env python3
"""narrate.facts — the grounded fact-extraction input model.

A *context chunk* is the set of grounded FACTS for ONE unit (a nodule / module or
a book section): the extracted function signatures, `@summary` / preceding
doc-block prose, and source locations. We do NOT re-implement extraction — we
CONSUME the already-committed, deterministic index JSON the Rust/`.myc` side
produces:

  * ``docs/lib-index/index.json``  — the `.myc` stdlib surface (per-nodule items)
  * ``docs/api-index/index.json``  — the Rust-crate surface

Both are ``{"generated": <honesty note>, "items": [ ... ]}``.  The two item
schemas differ slightly; :func:`load_facts` auto-detects which and normalises to
one :class:`Fact` record.

Honesty posture (VR-5 / G2):
  * Facts carry the index's own guarantee tag verbatim (``Empirical/Declared``),
    never upgraded.  The index is a heuristic; *source is ground truth* — a Fact
    is a pointer to where to Read, plus the verbatim text the index captured.
  * A missing / absent summary is represented as an EXPLICIT ``documented=False``
    fact ("undocumented"), never invented away and never silently dropped (G2).

Pure Python standard library only (Termux-portable).
"""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

# An identifier "run": a maximal code-identifier token.  Used to derive the
# grounding vocabulary a Fact licenses (see :meth:`Fact.vocabulary`).
_IDENT_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def identifier_runs(text: str) -> set[str]:
    """Every code-identifier token in ``text`` (generics/punctuation stripped)."""
    if not text:
        return set()
    return set(_IDENT_RE.findall(text))


@dataclass(frozen=True)
class Fact:
    """One grounded fact about a single code item — verbatim, with provenance.

    Never carries invented content: ``text`` / ``signature`` / ``summary`` are
    copied verbatim from the committed index; ``documented`` records whether a
    prose summary was present (``False`` ⇒ an explicit "undocumented" fact, G2).
    """

    id: str  # canonical symbol id, e.g. "std.result::map"
    kind: str  # nodule | type | ctor | fn | struct | enum | ...
    unit: str  # owning unit (nodule/module), e.g. "std.result"
    source_path: str  # file path relative to repo root
    line: int  # 1-based line
    signature: str  # verbatim signature (may be "")
    summary: str | None  # verbatim prose summary, or None if absent
    guarantee_tag: str  # verbatim from the index (e.g. "Empirical/Declared")
    documented: bool  # True iff a prose summary was present

    @property
    def name(self) -> str:
        """The short symbol name (last ``::`` segment), generics stripped."""
        tail = self.id.split("::")[-1]
        return tail.split("[")[0].split("<")[0].strip()

    @property
    def doc_ref(self) -> str:
        """The canonical ``src:`` doc_refs token pointing at this fact."""
        return f"src:{self.source_path}:{self.line}"

    @property
    def text(self) -> str:
        """Verbatim narratable text: signature plus summary when present."""
        parts = [p for p in (self.signature, self.summary) if p]
        return " — ".join(parts) if parts else "(undocumented)"

    def vocabulary(self) -> set[str]:
        """Identifier tokens this fact LICENSES for grounding.

        The union of: the short name, every identifier run in the signature, and
        every identifier run in the summary.  A generated sentence is grounded
        only if its code tokens are drawn from this vocabulary (across the unit's
        whole fact set) — the anti-hallucination basis.
        """
        vocab = {self.name}
        vocab |= identifier_runs(self.signature)
        if self.summary:
            vocab |= identifier_runs(self.summary)
        # dotted unit segments (std.result -> std, result, std.result)
        vocab.add(self.unit)
        vocab |= set(self.unit.split("."))
        return {v for v in vocab if v}

    def to_dict(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "kind": self.kind,
            "unit": self.unit,
            "source_path": self.source_path,
            "line": self.line,
            "signature": self.signature,
            "summary": self.summary,
            "guarantee_tag": self.guarantee_tag,
            "documented": self.documented,
        }


@dataclass
class FactSet:
    """The grounded facts for ONE unit — the bounded working set of a narration.

    This is the DN-96 context-window: one unit's facts loaded, narrated,
    validated, persisted, then dropped before the next unit.
    """

    unit: str
    facts: list[Fact]
    source_index: str  # path of the index the facts came from (provenance)

    def __post_init__(self) -> None:
        # Deterministic order: by (line, id) so a re-load is byte-identical.
        self.facts.sort(key=lambda f: (f.line, f.id))

    @property
    def header(self) -> Fact | None:
        """The unit-level fact (the nodule/module item), if present."""
        for f in self.facts:
            if f.kind in ("nodule", "module") or f.id == self.unit:
                return f
        return None

    @property
    def members(self) -> list[Fact]:
        """The non-header facts (types, ctors, fns) in deterministic order."""
        hdr = self.header
        return [f for f in self.facts if f is not hdr]

    def vocabulary(self) -> set[str]:
        """Union of every fact's licensed vocabulary (the grounding basis)."""
        vocab: set[str] = set(self.unit.split("."))
        vocab.add(self.unit)
        for f in self.facts:
            vocab |= f.vocabulary()
        return {v for v in vocab if v}

    def doc_refs(self) -> set[str]:
        """The set of resolvable ``src:`` doc_refs tokens these facts license."""
        return {f.doc_ref for f in self.facts}

    def undocumented(self) -> list[Fact]:
        """Facts with NO prose summary — surfaced explicitly, never hidden (G2)."""
        return [f for f in self.facts if not f.documented]

    def canonical_bytes(self) -> bytes:
        """Deterministic canonical serialisation (the cache-key input)."""
        payload = {
            "unit": self.unit,
            "facts": [f.to_dict() for f in self.facts],
        }
        return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()

    def content_hash(self) -> str:
        """blake2b digest of the canonical facts — stable across re-loads."""
        return hashlib.blake2b(self.canonical_bytes(), digest_size=16).hexdigest()


# ---------------------------------------------------------------------------
# Loaders — consume the committed index JSON (never re-extract)
# ---------------------------------------------------------------------------


def _item_unit(item: dict[str, Any]) -> str:
    """Resolve the owning-unit key for an index item, across both schemas."""
    # lib-index: {"nodule": "std.result", ...}
    if item.get("nodule"):
        return str(item["nodule"])
    # api-index: {"module": "mycelium_bench", "crate": "mycelium-bench", ...}
    if item.get("module"):
        return str(item["module"])
    if item.get("crate"):
        return str(item["crate"])
    # fall back to the symbol's own path prefix
    sym = str(item.get("symbol", ""))
    return sym.split("::")[0] if "::" in sym else sym


def _item_to_fact(item: dict[str, Any], unit: str) -> Fact:
    """Normalise one index item (either schema) to a :class:`Fact`."""
    summary = item.get("summary")
    if isinstance(summary, str) and not summary.strip():
        summary = None
    # tag key differs across schemas
    tag = item.get("tag") or item.get("guarantee_tag") or "Declared"
    return Fact(
        id=str(item.get("symbol", "")),
        kind=str(item.get("kind", "item")),
        unit=unit,
        source_path=str(item.get("file", "")),
        line=int(item.get("line", 0) or 0),
        signature=str(item.get("signature", "") or ""),
        summary=summary,
        guarantee_tag=str(tag),
        documented=summary is not None,
    )


def load_index(index_path: str | Path) -> list[dict[str, Any]]:
    """Load the raw ``items`` array from a committed index JSON file."""
    p = Path(index_path)
    payload = json.loads(p.read_text(encoding="utf-8"))
    items = payload.get("items", [])
    if not isinstance(items, list):
        raise ValueError(f"{p}: 'items' is not a list")
    return items


def load_facts(index_path: str | Path, unit: str) -> FactSet:
    """Load the :class:`FactSet` for one ``unit`` from an index JSON file.

    ``unit`` is matched against each item's owning-unit key (``nodule`` for the
    lib-index, ``module``/``crate`` for the api-index).  Raises ``KeyError`` (an
    explicit, never-silent failure) if the unit is absent from the index.
    """
    items = load_index(index_path)
    facts = [_item_to_fact(it, unit) for it in items if _item_unit(it) == unit]
    if not facts:
        available = sorted({_item_unit(it) for it in items})
        raise KeyError(
            f"unit {unit!r} not found in {index_path}. "
            f"{len(available)} units available "
            f"(e.g. {', '.join(available[:8])}…)"
        )
    return FactSet(unit=unit, facts=facts, source_index=str(index_path))


def list_units(index_path: str | Path) -> list[str]:
    """Every distinct owning-unit present in an index (deterministic order)."""
    return sorted({_item_unit(it) for it in load_index(index_path)})


# ---------------------------------------------------------------------------
# Repo-root resolution (so loaders work regardless of cwd)
# ---------------------------------------------------------------------------


def find_repo_root(start: Path | None = None) -> Path:
    """Walk up until a workspace ``Cargo.toml`` (or ``docs/lib-index``) is found."""
    p = (start or Path(__file__)).resolve()
    for _ in range(24):
        if (p / "docs" / "lib-index" / "index.json").is_file():
            return p
        cargo = p / "Cargo.toml"
        if cargo.is_file() and "[workspace]" in cargo.read_text(encoding="utf-8"):
            return p
        if p.parent == p:
            break
        p = p.parent
    # Fallback: the ancestor four levels up (tools/llm-harness/narrate/facts.py)
    return Path(__file__).resolve().parents[3]


# Convenience: default index locations relative to a repo root.
def lib_index_path(repo_root: Path | None = None) -> Path:
    root = repo_root or find_repo_root()
    return root / "docs" / "lib-index" / "index.json"


def api_index_path(repo_root: Path | None = None) -> Path:
    root = repo_root or find_repo_root()
    return root / "docs" / "api-index" / "index.json"
