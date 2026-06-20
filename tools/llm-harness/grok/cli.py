"""Command-line driver for the Grok/xAI co-authoring + ablation harness.

Entry points (all also reachable from ``harness.py --grok …``):

    # offline green gate — no key, no network (the CI/dev check):
    uv run python -m grok.cli --self-test

    # live sweep over the rubric (cheapest-first), key from env:
    export XAI_API_KEY=...   # or GROK_API_KEY
    uv run python -m grok.cli --mode live --models-file models.toml

    # batch sweep (independent first-pass generations) via xai_sdk:
    uv run python -m grok.cli --mode batch --models-file models.toml

    # M-381 retention-ratio ablation (live), explicit seeds:
    uv run python -m grok.cli --mode live --ablation --seeds 11,23,42

Never-silent (G2): a missing key, a bad rubric, or an unknown model is an explicit
error with a non-zero exit — never a quiet no-op.
"""

from __future__ import annotations

import argparse
import logging
import sys
from pathlib import Path

from .models import ModelConfigError, default_models_path, load_models, order_models
from .tasks import TASK_SET_ID, task_set


def _split_csv(value: str | None) -> list[str] | None:
    if not value:
        return None
    return [v.strip() for v in value.split(",") if v.strip()]


def _find_repo_root(start: Path) -> Path | None:
    cur = start.resolve()
    for parent in [cur, *cur.parents]:
        if (parent / ".git").exists():
            return parent
    return None


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="grok-harness",
        description="Mycelium Grok/xAI co-authoring + retention-ratio ablation harness "
        "(M-330/M-331/M-381). Pure Python + uv; optional xai_sdk for batch.",
    )
    mode = p.add_argument_group("run mode")
    mode.add_argument(
        "--self-test",
        action="store_true",
        help="run the deterministic OFFLINE self-test (no key/network) and exit. "
        "This is the green gate.",
    )
    mode.add_argument(
        "--emit-sample",
        action="store_true",
        help="with --self-test: also (re)write the committed SYNTHETIC sample report.",
    )
    mode.add_argument(
        "--mode",
        choices=("live", "batch"),
        default="live",
        help="live = sequential OpenAI-compatible REST (default); "
        "batch = xai_sdk batch for independent generations.",
    )
    mode.add_argument(
        "--ablation",
        action="store_true",
        help="run the M-381 retention-ratio ablation (research/11 §T11.7) per model.",
    )
    mode.add_argument(
        "--list-models",
        action="store_true",
        help="print the resolved (ordered) model list and exit.",
    )

    cfg = p.add_argument_group("configuration")
    cfg.add_argument(
        "--models-file",
        type=Path,
        default=None,
        help="path to the model rubric TOML (default: bundled models.toml).",
    )
    cfg.add_argument(
        "--models",
        default=None,
        help="comma-separated subset of model ids to run (cheapest-first within it).",
    )
    cfg.add_argument(
        "--order",
        default=None,
        help="comma-separated EXPLICIT model order (overrides cheapest-first).",
    )
    cfg.add_argument(
        "--base-url",
        default="https://api.x.ai/v1",
        help="OpenAI-compatible base URL for live mode (default: xAI).",
    )
    cfg.add_argument("--seed", type=int, default=42, help="RNG seed (default 42).")
    cfg.add_argument(
        "--seeds",
        default=None,
        help="comma-separated seeds for the ablation (default: 11,23,42).",
    )
    cfg.add_argument(
        "--max-rounds",
        type=int,
        default=3,
        help="max generate->fix rounds per task in live mode (default 3).",
    )
    cfg.add_argument(
        "--max-retries",
        type=int,
        default=5,
        help="max throttle/backoff retries per request in live mode (default 5).",
    )
    cfg.add_argument(
        "--task-set",
        default=TASK_SET_ID,
        help=f"task set id (default {TASK_SET_ID}).",
    )
    cfg.add_argument(
        "--reports-dir",
        type=Path,
        default=None,
        help="where to write reports (default: tools/llm-harness/reports/).",
    )
    cfg.add_argument(
        "-v", "--verbose", action="store_true", help="DEBUG-level logging."
    )
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    logging.basicConfig(
        level=logging.DEBUG if args.verbose else logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s: %(message)s",
    )
    log = logging.getLogger("grok.cli")

    here = Path(__file__).resolve()
    pkg_reports = here.parent.parent / "reports"
    reports_dir = args.reports_dir or pkg_reports

    # --- offline self-test (green gate) -------------------------------------
    if args.self_test:
        from .selftest import run_self_test

        return run_self_test(
            emit_sample=args.emit_sample, reports_dir=reports_dir, verbose=args.verbose
        )

    # --- resolve the model rubric (never-silent on a bad file) --------------
    models_file = args.models_file or default_models_path()
    try:
        specs = load_models(models_file)
        ordered = order_models(
            specs, select=_split_csv(args.models), order=_split_csv(args.order)
        )
    except ModelConfigError as exc:
        log.error("model rubric error: %s", exc)
        return 2

    if args.list_models:
        print(f"resolved model order ({len(ordered)}) from {models_file}:")
        for i, m in enumerate(ordered, 1):
            print(
                f"  {i}. {m.id}  ctx={m.context}  rpm={m.rpm} tpm={m.tpm}  "
                f"sync=${m.in_price}/${m.out_price}  "
                f"batch=${m.batch_in_price}/${m.batch_out_price} per Mtok"
            )
        return 0

    # --- a real run needs the late imports (which may pull optional deps) ---
    from .runner import RunConfig, run

    tasks = task_set(args.task_set)
    seeds = _split_csv(args.seeds)
    cfg = RunConfig(
        mode=args.mode,
        models=ordered,
        tasks=tasks,
        task_set_id=args.task_set,
        reports_dir=reports_dir,
        seed=args.seed,
        max_rounds=args.max_rounds,
        max_retries=args.max_retries,
        run_ablation=args.ablation,
        ablation_seeds=[int(s) for s in seeds] if seeds else None,
        repo_root=_find_repo_root(here),
        base_url=args.base_url,
    )

    log.info(
        "running %d model(s) in %s mode%s (task set %s, seed %d)",
        len(ordered),
        args.mode,
        " + ablation" if args.ablation else "",
        args.task_set,
        args.seed,
    )
    try:
        md_path, json_paths = run(cfg, log=log)
    except Exception as exc:  # never-silent: surface the failure + non-zero exit
        log.error("run failed: %s", exc)
        return 1
    print(f"comparison report: {md_path}")
    for jp in json_paths:
        print(f"per-model report:  {jp}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
