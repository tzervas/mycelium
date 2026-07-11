#!/usr/bin/env python3
"""narrate.report — dual JSON + human reports and the committed-output writer.

Two projections of one :class:`~narrate.session.NarrateRun` (G11 — dual
projection):

  * :func:`emit_reports` — writes a structured JSON report AND a human-readable
    text report (the run's rounds, ``validated_fraction``, and the never-silent
    dropped-sentence list).
  * :func:`write_output` — writes the committed validated prose to
    ``narrate/out/`` with a DETERMINISTIC provenance header (unit, target,
    model-id, seed, ``validated_fraction``, guarantee tag, and the facts it was
    generated from).  No wall-clock in the prose file, so identical inputs ⇒
    byte-identical output (idempotence).

Pure Python standard library only.
"""

from __future__ import annotations

import json
from pathlib import Path

from narrate.session import NarrateRun


def provenance_header(run: NarrateRun) -> str:
    """A deterministic HTML-comment provenance block for the committed prose."""
    lines = [
        "<!--",
        "  narrate provenance (deterministic; no wall-clock — idempotent bytes)",
        f"  unit:               {run.unit}",
        f"  target:             {run.target}",
        f"  model_id:           {run.model_id}",
        f"  seed:               {run.seed}",
        f"  guarantee_tag:      {run.guarantee_tag}",
        f"  validated_fraction: {run.validated_fraction:.6f}",
        f"  status:             {run.status}",
        f"  source_index:       {run.source_index}",
        "  generated_from (facts):",
    ]
    for ref in run.fact_doc_refs:
        lines.append(f"    - {ref}")
    lines.append(
        "  NOTE: prose is Declared/Empirical (VR-5) — validated against the facts"
    )
    lines.append("  above; unvalidated sentences were dropped (see the report).")
    lines.append("-->")
    return "\n".join(lines)


def write_output(run: NarrateRun, out_dir: Path) -> Path:
    """Write validated prose + provenance header; return the output path."""
    out_dir = Path(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    path = out_dir / f"{run.unit}.{run.target}.md"
    body = provenance_header(run) + "\n\n" + run.committed_prose.rstrip() + "\n"
    path.write_text(body, encoding="utf-8")
    return path


def emit_reports(run: NarrateRun, reports_dir: Path, run_id: str) -> tuple[Path, Path]:
    """Write the JSON + human reports; return ``(json_path, txt_path)`` (G11)."""
    reports_dir = Path(reports_dir)
    reports_dir.mkdir(parents=True, exist_ok=True)
    stem = f"narrate-{run.unit}-{run.target}-{run_id}"
    json_path = reports_dir / f"{stem}.json"
    txt_path = reports_dir / f"{stem}.txt"

    json_path.write_text(
        json.dumps(run.to_dict(), indent=2, sort_keys=True), encoding="utf-8"
    )

    lines = [
        "=" * 72,
        "Mycelium narrate report — validated narrative generation",
        f"Unit    : {run.unit}",
        f"Target  : {run.target}",
        f"Model   : {run.model_id}  (guarantee={run.guarantee_tag}, seed={run.seed})",
        f"Status  : {run.status}",
        f"Validated fraction: {run.validated_fraction:.4f}  "
        f"({'committed' if run.committed_prose.strip() else 'nothing committed'})",
        "=" * 72,
        "",
        run.message,
        "",
    ]
    for rnd in run.rounds:
        res = rnd.result
        corr = " [correction]" if rnd.is_correction else ""
        lines.append(
            f"  Round {rnd.round_number}{corr}: "
            f"{res.validated_sentences}/{res.total_sentences} validated "
            f"(fraction={res.validated_fraction:.4f}, dropped={len(res.dropped)})"
        )
        for v in res.dropped:
            lines.append(
                f"    DROPPED [p{v.paragraph_index}s{v.sentence_index}]: {v.reason}"
            )
            lines.append(f"      text: {v.text[:100]}")
    lines += [
        "",
        "-" * 72,
        "Committed facts (grounding basis):",
    ]
    for ref in run.fact_doc_refs:
        lines.append(f"  {ref}")
    lines += [
        "-" * 72,
        f"Guarantee posture: {run.guarantee_tag} — narration validated against "
        "extracted facts (VR-5). Unvalidated sentences dropped, never silently "
        "(G2).",
        "=" * 72,
    ]
    txt_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return json_path, txt_path
