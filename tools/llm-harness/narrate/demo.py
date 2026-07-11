#!/usr/bin/env python3
"""narrate.demo — the end-to-end D3 demo (one real ``lib/std`` unit, Mock backends).

Runs the whole pipeline offline on ONE real unit's committed facts:

  1. load the facts for ``std.result`` from ``docs/lib-index/index.json``;
  2. generate a reference-manual entry (deterministic MockGenerator, cached);
  3. validate it (MockChecker — the faithfulness oracle);
  4. write the validated prose + provenance header to ``narrate/out/``;
  5. emit dual JSON + human reports to ``narrate/reports/``;
  6. print the ``validated_fraction``.

Run it either way:

    python3 -m narrate.demo          # from tools/llm-harness/
    python3 narrate/demo.py          # from tools/llm-harness/ (script form)

Exit code 0 on success (validated_fraction reported), non-zero on setup failure.
"""

from __future__ import annotations

import sys
from pathlib import Path

# Bootstrap: make ``narrate`` importable whether run as a module or a script.
try:
    from narrate.facts import find_repo_root, lib_index_path, load_facts
except ModuleNotFoundError:  # pragma: no cover - script-form invocation
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    from narrate.facts import find_repo_root, lib_index_path, load_facts

from narrate.checker import MockChecker
from narrate.generator import CachingGenerator, MockGenerator
from narrate.prompts import load_template
from narrate.report import emit_reports, write_output
from narrate.session import narrate_unit

DEMO_UNIT = "std.result"
DEMO_TARGET = "ref-manual-entry"


def run_demo(unit: str = DEMO_UNIT, target: str = DEMO_TARGET) -> int:
    """Execute the demo; return an exit code."""
    here = Path(__file__).resolve().parent
    repo_root = find_repo_root(here)
    index = lib_index_path(repo_root)

    if not index.is_file():
        print(f"SKIP: committed index not found at {index} (nothing to narrate).")
        return 0

    print(f"narrate demo: loading facts for {unit!r} from {index} …")
    facts = load_facts(index, unit)
    print(
        f"  loaded {len(facts.facts)} facts "
        f"({len(facts.undocumented())} undocumented) — content_hash="
        f"{facts.content_hash()}"
    )

    template = load_template(target)
    cache_dir = here / ".cache"
    generator = CachingGenerator(base=MockGenerator(), cache_dir=cache_dir)
    checker = MockChecker()

    run = narrate_unit(facts, template, generator, checker, max_rounds=3)

    out_path = write_output(run, here / "out")
    json_path, txt_path = emit_reports(run, here / "reports", run_id="demo")

    print("")
    print(f"  status            : {run.status}")
    print(f"  cache hit         : {generator.last_was_cache_hit}")
    print(f"  committed output  : {out_path}")
    print(f"  json report       : {json_path}")
    print(f"  text report       : {txt_path}")
    print("")
    print(f"  validated_fraction = {run.validated_fraction:.4f}")
    print("")
    print("--- committed prose (first 600 chars) ---")
    print(run.committed_prose[:600])
    return 0


def main() -> int:
    return run_demo()


if __name__ == "__main__":
    sys.exit(main())
