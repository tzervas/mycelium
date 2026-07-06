#!/usr/bin/env python3
"""Assemble gen/myc-drafts/{manifest.json,MANIFEST.md} from already-written per-target artifacts.

M-1002 (kickoff trx2 E-B). Pure aggregation -- this script re-derives NOTHING about gaps/vetting;
it only reads the artifacts `mycelium-transpile --vet` already wrote (per-file `<stem>.gap.json`,
per-target `vet.json`, and for batch/directory targets `summary.json`) and folds their already-
Declared/Empirical numbers into one manifest. See gen/myc-drafts/README.md for the honesty
contract this manifest documents.

Determinism: no wall-clock timestamps are written anywhere (CLAUDE.md's markdown/diff-churn
guidance carries over to this JSON/MD pair) -- the one provenance field is `generated_from_commit`,
the git SHA of the Rust sources the drafts were regenerated against, which is stable across reruns
at the same commit. All dict output is key-sorted (`sort_keys=True`); target order follows the
caller-supplied `--targets` list, not directory iteration, so two runs at the same commit produce
byte-identical output.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import Counter
from pathlib import Path

# Per-target notes tied to the destination-convention call-outs the epic brief asks for
# (lib/compiler/README.md's semcore-single-nodule + FLAG-ast-5/FLAG-parse-2 discipline). Kept as
# a static table here (not derived from the transpile output, which has no notion of "destination
# nodule") -- this is contextual annotation, not a measurement.
SEMCORE_NOTE = (
    "Destination convention (lib/compiler/README.md): this file is one of the nine "
    "mycelium-l1 semantic-core modules that hand-port into a SINGLE Mycelium nodule "
    "(compiler.semcore) for nodule-wide mutual recursion (DN-26 SCC). This transpiler emits "
    "one .myc per Rust file and does NOT perform that merge. Any emitted/gapped constructor "
    "whose name would collide with another semcore type's constructor (or a reserved word) "
    "once merged is subject to the FLAG-ast-5/FLAG-parse-2 per-type constructor-prefixing "
    "convention at hand-port time -- gapped ReservedWord items in this draft are exactly the "
    "collisions the transpiler refuses to auto-rename (VR-5/G2)."
)
STDLIB_NOTE = (
    "Standalone phylum draft -- no cross-file nodule-merge convention applies (unlike the "
    "semcore SCC). Each emitted .myc file stands as its own draft nodule candidate."
)


def sha256_of(path: Path) -> str:
    h = hashlib.sha256()
    h.update(path.read_bytes())
    return h.hexdigest()


def load_json(path: Path):
    if not path.exists():
        return None
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def pct(numerator: int, denom: int) -> float:
    if denom == 0:
        return 0.0
    return numerator / denom * 100.0


def gap_category_counts_from_gaps(gaps: list) -> dict:
    c: Counter = Counter()
    for g in gaps:
        c[g["category"]] += 1
    return dict(c)


def build_target_entry(
    repo_root: Path, out_root: Path, subdir: str, input_rel: str, kind: str
) -> dict:
    outdir = out_root / subdir
    entry: dict = {
        "target": subdir,
        "kind": kind,
        "output_dir": f"gen/myc-drafts/{subdir}",
        "note": SEMCORE_NOTE if kind == "semcore" else STDLIB_NOTE,
    }

    if kind == "semcore":
        stem = Path(input_rel).stem
        gap_path = outdir / f"{stem}.gap.json"
        vet_path = outdir / "vet.json"
        gap = load_json(gap_path)
        vet = load_json(vet_path)
        if gap is None or vet is None:
            entry["status"] = "transpile_failed"
            src_path = repo_root / input_rel
            entry["error"] = (
                f"expected artifacts missing under {subdir} "
                f"(gap.json present={gap is not None}, vet.json present={vet is not None}, "
                f"source present={src_path.exists()})"
            )
            # Best-effort: still record the source hash when the file exists (a transpile crash
            # with the input present is a different, more actionable failure than a missing
            # input) -- but never crash the manifest run over a target whose input itself is
            # absent (the "target not found" case regenerate.sh already reported loudly).
            entry["rust_sources"] = (
                [{"path": input_rel, "sha256": sha256_of(src_path)}]
                if src_path.exists()
                else []
            )
            return entry

        src_path = repo_root / input_rel
        entry["rust_sources"] = [{"path": input_rel, "sha256": sha256_of(src_path)}]
        total_items = gap["total_top_level_items"]
        gaps_list = gap["gaps"]
        test_items = sum(1 for g in gaps_list if g["category"] == "TestItem")
        non_test_items = total_items - test_items
        emitted = len(gap["emitted_items"])
        gap_count = len(gaps_list)
        vrec = vet["records"][0] if vet["records"] else None
        checked_clean = vet["total_checked_clean_items"]
        entry["status"] = "ok"
        entry["stats"] = {
            "total_top_level_items": total_items,
            "non_test_items": non_test_items,
            "emitted_items": emitted,
            "checked_clean_items": checked_clean,
            "gap_count": gap_count,
            "expressible_fraction_pct": round(pct(emitted, non_test_items), 2),
            "checked_fraction_pct": round(pct(checked_clean, non_test_items), 2),
        }
        entry["gap_category_counts"] = gap_category_counts_from_gaps(gaps_list)
        entry["vet_class_counts"] = vet["class_counts"]
        entry["vet_diagnostic"] = vrec["diagnostic"] if vrec else ""
        return entry

    # kind == "stdlib": batch/directory mode.
    summary_path = outdir / "summary.json"
    vet_path = outdir / "vet.json"
    summary = load_json(summary_path)
    vet = load_json(vet_path)
    if summary is None or vet is None:
        entry["status"] = "transpile_failed"
        entry["error"] = (
            f"expected artifacts missing under {subdir} "
            f"(summary.json present={summary is not None}, vet.json present={vet is not None})"
        )
        entry["rust_sources"] = []
        return entry

    rust_sources = []
    for f in summary["files"]:
        rel = f["file"]
        rust_sources.append({"path": rel, "sha256": sha256_of(repo_root / rel)})
    entry["rust_sources"] = rust_sources

    totals = summary["totals"]
    non_test_items = totals["non_test_items"]
    emitted = totals["emitted"]
    checked_clean = vet["total_checked_clean_items"]
    entry["status"] = "ok"
    entry["stats"] = {
        "total_top_level_items": totals["total_items"],
        "non_test_items": non_test_items,
        "emitted_items": emitted,
        "checked_clean_items": checked_clean,
        "gap_count": totals["gaps"],
        "expressible_fraction_pct": round(pct(emitted, non_test_items), 2),
        "checked_fraction_pct": round(pct(checked_clean, non_test_items), 2),
    }
    entry["gap_category_counts"] = totals["category_counts"]
    entry["vet_class_counts"] = vet["class_counts"]
    entry["file_count"] = len(summary["files"])
    return entry


def render_markdown(manifest: dict) -> str:
    lines = []
    lines.append("# `gen/myc-drafts/` manifest (M-1002/M-1003, kickoff `trx2` E-B)")
    lines.append("")
    lines.append(
        "> Auto-generated by `gen/myc-drafts/regenerate.sh` + `manifest_gen.py` -- do not hand-edit."
        " Regenerate with `bash gen/myc-drafts/regenerate.sh`. Every number here is `Declared`"
        " (transpile emission) or `Empirical` (the `myc check` vet verdict) -- see README.md's"
        " honesty contract."
    )
    lines.append("")
    lines.append(f"- **Generated from commit:** `{manifest['generated_from_commit']}`")
    lines.append(f"- **Schema version:** {manifest['schema_version']}")
    lines.append(
        f"- **Kickoff / epic:** `{manifest['kickoff']}` / `{manifest['epic']}` (wave {manifest['wave']})"
    )
    lines.append("")

    targets = manifest["targets"]
    ok_targets = [t for t in targets if t.get("status") == "ok"]
    total_non_test = sum(t["stats"]["non_test_items"] for t in ok_targets)
    total_emitted = sum(t["stats"]["emitted_items"] for t in ok_targets)
    total_checked = sum(t["stats"]["checked_clean_items"] for t in ok_targets)
    lines.append(
        f"**Union across all {len(targets)} wave-1 targets:** {total_non_test} non-test items, "
        f"{total_emitted} emitted ({pct(total_emitted, total_non_test):.1f}% expressible), "
        f"{total_checked} myc-check-clean ({pct(total_checked, total_non_test):.1f}% checked, "
        "file-gated lower bound -- see DN-34 §8.7 for the metric's stated denominator/numerator)."
    )
    lines.append("")

    lines.append(
        "| Target | Kind | non-test items | emitted | expressible % | checked % | vet classes | status |"
    )
    lines.append("|---|---|---:|---:|---:|---:|---|---|")
    for t in targets:
        if t.get("status") != "ok":
            lines.append(
                f"| `{t['target']}` | {t['kind']} | — | — | — | — | — | **{t['status']}**: {t.get('error', '')} |"
            )
            continue
        s = t["stats"]
        classes = ", ".join(
            f"{k}={v}" for k, v in sorted(t["vet_class_counts"].items())
        )
        lines.append(
            f"| `{t['target']}` | {t['kind']} | {s['non_test_items']} | {s['emitted_items']} | "
            f"{s['expressible_fraction_pct']:.1f} | {s['checked_fraction_pct']:.1f} | {classes} | ok |"
        )
    lines.append("")

    lines.append("## Per-target gap category counts")
    lines.append("")
    for t in targets:
        if t.get("status") != "ok":
            continue
        cats = ", ".join(
            f"{k}={v}" for k, v in sorted(t["gap_category_counts"].items())
        )
        lines.append(
            f"- **`{t['target']}`** ({t['stats']['gap_count']} gaps): {cats if cats else '(none)'}"
        )
    lines.append("")

    lines.append("## Notes")
    lines.append("")
    seen_notes = set()
    for t in targets:
        note = t.get("note")
        if note and note not in seen_notes:
            seen_notes.add(note)
            lines.append(f"- **{t['kind']}:** {note}")

    # Exactly one trailing newline (avoid MD012 "multiple consecutive blank lines" at EOF --
    # `lines` must not itself end with a blank "" entry, since join already terminates the last
    # element without a following separator and this return adds the file's one trailing `\n`).
    return "\n".join(lines) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--root", required=True, help="gen/myc-drafts/ absolute path")
    ap.add_argument("--repo-root", required=True, help="repo root absolute path")
    ap.add_argument(
        "--source-commit", required=True, help="git SHA the drafts were generated at"
    )
    ap.add_argument(
        "--targets",
        required=True,
        nargs="+",
        help="pipe-delimited rows: <output-subdir>|<input-path>|<kind>",
    )
    args = ap.parse_args()

    out_root = Path(args.root)
    repo_root = Path(args.repo_root)

    targets_out = []
    for row in args.targets:
        subdir, input_rel, kind = row.split("|")
        targets_out.append(
            build_target_entry(repo_root, out_root, subdir, input_rel, kind)
        )

    manifest = {
        "schema_version": 1,
        "kickoff": "trx2",
        "epic": "E33-1",
        "wave": 1,
        "generated_from_commit": args.source_commit,
        "targets": targets_out,
    }

    manifest_json_path = out_root / "manifest.json"
    manifest_json_path.write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )

    manifest_md_path = out_root / "MANIFEST.md"
    manifest_md_path.write_text(render_markdown(manifest), encoding="utf-8")

    any_failed = any(t.get("status") != "ok" for t in targets_out)
    if any_failed:
        print(
            "manifest_gen.py: one or more targets are transpile_failed -- see manifest for details",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
