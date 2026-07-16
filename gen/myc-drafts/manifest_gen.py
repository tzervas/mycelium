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


def phylum_dual_report(
    vet: dict, non_test_items: int, checked_fraction_pct: float
) -> dict | None:
    """Extract the DN-124 M-A dual-report block from a batch-mode `vet.json`'s additive `phylum`
    field (Unit 2, `mycelium-transpile::vet::VetReport::with_phylum`). Returns `None` when the field
    is absent (an older, pre-M-1079 `vet.json`, or a run where `--vet` never reached directory mode)
    -- never fabricates a phylum result that was not actually measured (G2/VR-5).

    `delta_basis_pct` is `checked_fraction_phylum_pct - checked_fraction_pct`, computed over the
    IDENTICAL denominator both fractions already share -- labeled a **basis correction**, never
    folded into `checked_fraction_pct` (which stays the oracle-mode number, untouched) or presented
    as lever progress (DN-124 §4.3).
    """
    phylum = vet.get("phylum")
    if phylum is None:
        return None
    checked_clean_phylum = vet.get("total_checked_clean_items_phylum", 0)
    checked_fraction_phylum_pct = round(pct(checked_clean_phylum, non_test_items), 2)
    return {
        "ran": phylum.get("ran", False),
        "ok": phylum.get("ok", False),
        "checked_clean_items_phylum": checked_clean_phylum,
        "checked_fraction_phylum_pct": checked_fraction_phylum_pct,
        "delta_basis_pct": round(checked_fraction_phylum_pct - checked_fraction_pct, 2),
        "diagnostic": phylum.get("diagnostic", ""),
        "nodule_count": len(phylum.get("nodules", [])),
    }


def build_target_entry(
    repo_root: Path, out_root: Path, subdir: str, input_rel: str, kind: str
) -> dict:
    """Every wave-1 target is now batch/directory-mode (DN-124 §3.2/M-1079: semcore's 5
    mutually-referencing files are transpiled+vetted as ONE phylum, exactly like a stdlib crate's
    `src/` already was) -- so `semcore` and `stdlib` read the identical `summary.json`+`vet.json`
    artifact shape; only the annotation `note` differs by kind. `input_rel` is a single directory
    path for `stdlib`, or a COMMA-separated list of the batch's member files for `semcore` (the
    provenance list `rust_sources` records every member individually either way).
    """
    outdir = out_root / subdir
    entry: dict = {
        "target": subdir,
        "kind": kind,
        "output_dir": f"gen/myc-drafts/{subdir}",
        "note": SEMCORE_NOTE if kind == "semcore" else STDLIB_NOTE,
    }

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
        # Best-effort provenance even on failure (a transpile crash with the input present is a
        # different, more actionable failure than a missing input) -- never crash the manifest run
        # over one failed target (the "target not found" case regenerate.sh already reported loudly).
        member_paths = input_rel.split(",") if kind == "semcore" else []
        entry["rust_sources"] = [
            {"path": p, "sha256": sha256_of(repo_root / p)}
            for p in member_paths
            if (repo_root / p).exists()
        ]
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
    checked_fraction_pct = round(pct(checked_clean, non_test_items), 2)
    entry["status"] = "ok"
    entry["stats"] = {
        "total_top_level_items": totals["total_items"],
        "non_test_items": non_test_items,
        "emitted_items": emitted,
        "checked_clean_items": checked_clean,
        "gap_count": totals["gaps"],
        "expressible_fraction_pct": round(pct(emitted, non_test_items), 2),
        "checked_fraction_pct": checked_fraction_pct,
    }
    entry["gap_category_counts"] = totals["category_counts"]
    entry["vet_class_counts"] = vet["class_counts"]
    entry["file_count"] = len(summary["files"])
    # DN-124 M-A dual-report: a SEPARATE block, never merged into `stats.checked_fraction_pct`
    # (which stays the oracle-mode, per-file basis this metric has always been) -- `None` when this
    # run's vet.json carries no phylum result to report (VR-5: never fabricated).
    entry["phylum"] = phylum_dual_report(vet, non_test_items, checked_fraction_pct)
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
        "file-gated lower bound -- see DN-34 §8.7 for the metric's stated denominator/numerator). "
        "**This `checked` figure is the oracle-mode (per-file, phylum-blind) basis** -- see the "
        "phylum-mode dual-report section below (DN-124/M-1079)."
    )
    lines.append("")

    lines.append(
        "| Target | Kind | non-test items | emitted | expressible % | checked % (oracle) | vet classes | status |"
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

    # DN-124 M-A dual-report: a SEPARATE section, never merged into the oracle-mode table above.
    # `checked % (phylum)` is the basis-corrected metric (matches the REAL phylum boundary a build
    # checks); `Δ_basis (pp)` is `phylum - oracle`, explicitly labeled a ONE-TIME BASIS CORRECTION
    # (recovered false-fails), never presented as lever/transpiler progress (VR-5).
    phylum_targets = [t for t in ok_targets if t.get("phylum") is not None]
    lines.append(
        "## Phylum-mode dual-report (DN-124/M-1079 -- transition-cycle basis correction)"
    )
    lines.append("")
    lines.append(
        "> `checked_fraction_phylum` measures myc-check-clean coverage against the REAL phylum a "
        "nodule belongs to (the kernel's cross-nodule resolver), never a phylum-of-one counterfactual "
        "-- oracle mode false-FAILs a correctly-emitted cross-nodule `use` (DN-124 §1). The `Δ_basis` "
        "jump this run shows is a **basis correction** (recovering previously-false-failed items), "
        "**NOT** lever/transpiler progress -- a real lever gain would show up phylum-to-phylum across "
        "runs, never folded into this one-time delta (DN-124 §4.3, VR-5)."
    )
    lines.append("")
    if phylum_targets:
        total_checked_phylum = sum(
            t["phylum"]["checked_clean_items_phylum"] for t in phylum_targets
        )
        phylum_denom = sum(t["stats"]["non_test_items"] for t in phylum_targets)
        oracle_pct_over_phylum_denom = pct(
            sum(t["stats"]["checked_clean_items"] for t in phylum_targets), phylum_denom
        )
        phylum_pct = pct(total_checked_phylum, phylum_denom)
        lines.append(
            f"**Union across the {len(phylum_targets)} target(s) with a phylum result:** "
            f"{total_checked_phylum} myc-check-clean under phylum mode "
            f"({phylum_pct:.1f}% checked_fraction_phylum) vs {oracle_pct_over_phylum_denom:.1f}% "
            f"checked_fraction (oracle), over the SAME {phylum_denom} non-test-item denominator -- "
            f"**Δ_basis = {phylum_pct - oracle_pct_over_phylum_denom:+.1f}pp** (basis correction)."
        )
        lines.append("")
        lines.append(
            "| Target | checked % (oracle) | checked % (phylum) | Δ_basis (pp) | phylum ok | phylum ran |"
        )
        lines.append("|---|---:|---:|---:|---|---|")
        for t in phylum_targets:
            p = t["phylum"]
            lines.append(
                f"| `{t['target']}` | {t['stats']['checked_fraction_pct']:.1f} | "
                f"{p['checked_fraction_phylum_pct']:.1f} | {p['delta_basis_pct']:+.1f} | "
                f"{p['ok']} | {p['ran']} |"
            )
        lines.append("")
        not_ran = [t for t in phylum_targets if not t["phylum"]["ran"]]
        if not_ran:
            lines.append(
                "**Not run this cycle (myc-check --phylum could not execute -- never counted "
                "clean, G2):** "
                + ", ".join(
                    f"`{t['target']}` ({t['phylum']['diagnostic']})" for t in not_ran
                )
            )
            lines.append("")
    else:
        lines.append(
            "_(No target carries a phylum-mode result this run -- every `vet.json` predates the "
            "M-1079 dual-report wiring, or every target failed to transpile. Re-run "
            "`bash gen/myc-drafts/regenerate.sh` to populate this section.)_"
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
