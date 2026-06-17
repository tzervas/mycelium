"""Run the KC-2 LLM-leverage experiment (M-002) against a local llama.cpp model.

    # one-shot, Mycelium arm only (needs myc-check; baseline is opt-in, see below):
    python -m mycelium_experiments.kc2 --model PATH.gguf --out kc2-report.json

    # against a llama.cpp server (cleaner output than the CLI):
    python -m mycelium_experiments.kc2 --server http://localhost:8080 --out kc2-report.json

    # include the baseline arm (executes model-generated Python — see the warning):
    python -m mycelium_experiments.kc2 --model PATH.gguf \\
        --arms mycelium,baseline --allow-untrusted-baseline --out kc2-report.json

Honesty posture (house rules):
- NEVER-SILENT (G2): an unavailable checker SKIPs its arm with an explicit reason —
  never a fake 0%. A backend error aborts loudly, never a silent empty generation.
- VR-5: this writes measured RATES only. The KC-2 verdict (proceed / reweight-to-human
  / fall-back-to-embedded-DSL) is a maintainer-written analysis; it is NOT computed here.
- The baseline arm executes generated Python in-process; it is OFF by default and
  requires --allow-untrusted-baseline (run it inside a container/VM — see checkers.py).
"""

from __future__ import annotations

import argparse
import datetime
import json
import os
import sys
from pathlib import Path

from mycelium_experiments.kc2 import llm
from mycelium_experiments.kc2.checkers import (
    BaselineChecker,
    MyceliumChecker,
    ToolUnavailable,
)
from mycelium_experiments.kc2.harness import ArmReport, run_arm
from mycelium_experiments.kc2.summary import assess, render_summary
from mycelium_experiments.kc2.tasks import TASKS

# Mirror tools/llm-harness's cache layout so a model fetched there is reused here.
_DEFAULT_MODEL_FILENAME = "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf"

# KC-2 prompts (primer + task + feedback + generation) are larger than the harness's,
# so the workload wants more context — still capped to fit available RAM (auto).
_KC2_DESIRED_CTX = 2048

_VERDICT = (
    "not established — the KC-2 verdict (proceed / reweight-to-human / "
    "fall-back-to-embedded-DSL) requires a maintainer-written analysis of this run; "
    "this harness never pre-writes it (VR-5)"
)


def _default_model_dir() -> Path:
    env = os.environ.get("MYCELIUM_LLM_MODEL_DIR")
    if env:
        return Path(env).expanduser()
    xdg = os.environ.get("XDG_CACHE_HOME")
    base = Path(xdg).expanduser() if xdg else Path.home() / ".cache"
    return base / "mycelium-llm-harness" / "models"


def _find_model(explicit: str | None) -> str | None:
    if explicit:
        return explicit if Path(explicit).is_file() else None
    md = _default_model_dir()
    default = md / _DEFAULT_MODEL_FILENAME
    if default.is_file():
        return str(default)
    try:
        for p in sorted(md.glob("*.gguf")):
            if p.is_file():
                return str(p)
    except OSError:
        pass
    return None


def _read_primer(path: str | None) -> str | None:
    if not path:
        return None
    return Path(path).expanduser().read_text(encoding="utf-8")


def _arm_metrics(report: ArmReport) -> dict[str, object]:
    return {
        "ran": True,
        "syntactic_validity_rate": report.syntactic_validity_rate,
        "first_attempt_pass_rate": report.first_attempt_pass_rate,
        "eventual_pass_rate": report.eventual_pass_rate,
        "mean_iterations_to_pass": report.mean_iterations_to_pass,
        "outcomes": [
            {
                "task": o.task_id,
                "first_attempt_valid": o.first_attempt_valid,
                "first_attempt_passed": o.first_attempt_passed,
                "passed": o.passed,
                "iterations": o.iterations,
            }
            for o in report.outcomes
        ],
    }


def _build_backend(args: argparse.Namespace) -> llm.Backend:
    """Resolve a backend or exit with an explicit, actionable message (never-silent)."""
    if args.server:
        return llm.server_backend(args.server, seed=args.seed, n_predict=args.n_predict)
    cli = llm.resolve_llama_cli(args.llama_cli)
    if not cli:
        sys.exit(
            "ERROR: no `llama`/`llama-cli` found. Pass --llama-cli PATH or --server URL, "
            "or run `python tools/llm-harness/harness.py --doctor` to install/heal it."
        )
    model = _find_model(args.model)
    if not model:
        sys.exit(
            "ERROR: no model. Pass --model PATH.gguf, or fetch the default with "
            "`python tools/llm-harness/harness.py --ensure-model` (it caches into "
            f"{_default_model_dir()})."
        )
    if args.ctx_size is not None:
        ctx_size = args.ctx_size
    else:
        mem = llm.detect_memory()
        ctx_size, reason = llm.auto_ctx_size(
            _KC2_DESIRED_CTX, model, mem, swap_fraction=0.5 if args.use_swap else 0.0
        )
        print(f"Auto-ctx: {reason} (override with --ctx-size)", file=sys.stderr)

    if args.cpu_only:
        n_gpu_layers = 0
    elif args.n_gpu_layers is not None:
        n_gpu_layers = args.n_gpu_layers
    else:
        n_gpu_layers, gpu_reason = llm.auto_gpu_layers(llm.detect_gpu(), model)
        print(f"Auto-GPU: {gpu_reason}", file=sys.stderr)

    return llm.cli_backend(
        cli,
        model,
        seed=args.seed,
        n_predict=args.n_predict,
        ctx_size=ctx_size,
        n_gpu_layers=n_gpu_layers,
        timeout=args.timeout,
        extra_args=args.llama_extra_arg,
    )


def main() -> int:
    p = argparse.ArgumentParser(
        prog="python -m mycelium_experiments.kc2",
        description="Run the KC-2 LLM-leverage experiment against a local llama.cpp model.",
    )
    src = p.add_mutually_exclusive_group()
    src.add_argument("--server", metavar="URL", help="llama.cpp HTTP server base URL.")
    src.add_argument("--llama-cli", metavar="PATH", help="Path to `llama`/`llama-cli`.")
    p.add_argument("--model", metavar="PATH", help="Path to a .gguf model (CLI backend).")
    p.add_argument(
        "--arms",
        default="mycelium",
        help="Comma-separated arms to run: mycelium,baseline (default: mycelium).",
    )
    p.add_argument(
        "--allow-untrusted-baseline",
        action="store_true",
        help="Run the baseline arm, which EXECUTES model-generated Python in-process. "
        "Only with a sandbox (container/VM). Off by default (the arm SKIPs without it).",
    )
    p.add_argument("--max-iters", type=int, default=3, help="Edit-to-fix budget (default 3).")
    p.add_argument("--seed", type=int, default=42, help="Generation seed (default 42).")
    p.add_argument("--n-predict", type=int, default=256, help="Max new tokens (default 256).")
    p.add_argument(
        "--timeout",
        type=int,
        default=300,
        help="Per-generation llama-cli timeout in seconds (default 300). Raise on a slow "
        "CPU phone (~1-2 tok/s) if generation times out.",
    )
    p.add_argument(
        "--ctx-size",
        type=int,
        default=None,
        help="llama.cpp context window -c. DEFAULT: auto — sized from available RAM "
        "(/proc/meminfo) so a phone doesn't OOM-kill (SIGKILL/9) on the model's full 32k "
        "window. Pass an explicit N to override.",
    )
    p.add_argument(
        "--use-swap",
        action="store_true",
        help="Count ~half of free swap toward the auto context budget (slower if the KV "
        "cache pages out). Off by default.",
    )
    p.add_argument(
        "--cpu-only",
        action="store_true",
        help="Force CPU only — never offload to a GPU even if one is detected.",
    )
    p.add_argument(
        "--n-gpu-layers",
        type=int,
        default=None,
        help="GPU layers to offload (llama.cpp -ngl). DEFAULT: auto from detected VRAM "
        "(0 on a phone). 0 = CPU, 999 = all.",
    )
    p.add_argument("--primer-mycelium", metavar="FILE", help="Override the Mycelium-arm primer.")
    p.add_argument("--primer-baseline", metavar="FILE", help="Override the baseline-arm primer.")
    p.add_argument(
        "--llama-extra-arg",
        action="append",
        default=[],
        metavar="ARG",
        help="Extra arg passed to the llama CLI (repeatable), e.g. --llama-extra-arg=-no-cnv.",
    )
    p.add_argument("--out", metavar="PATH", help="Write the JSON report here (else stdout only).")
    args = p.parse_args()

    requested = [a.strip() for a in args.arms.split(",") if a.strip()]
    unknown = [a for a in requested if a not in ("mycelium", "baseline")]
    if unknown:
        p.error(f"unknown arm(s): {unknown} (expected mycelium and/or baseline)")

    backend = _build_backend(args)
    primers: dict[str, str] = {}
    if (pm := _read_primer(args.primer_mycelium)) is not None:
        primers["mycelium"] = pm
    if (pb := _read_primer(args.primer_baseline)) is not None:
        primers["baseline"] = pb
    generator = llm.LlamaGenerator(backend=backend, primers=primers)

    arms_report: dict[str, object] = {}
    for arm in requested:
        if arm == "mycelium":
            try:
                checker = MyceliumChecker()
            except ToolUnavailable as exc:
                arms_report["mycelium"] = {"ran": False, "skipped": str(exc)}
                print(f"SKIP mycelium arm: {exc}", file=sys.stderr)
                continue
            report = run_arm(generator, checker, "mycelium", TASKS, args.max_iters)
            arms_report["mycelium"] = _arm_metrics(report)
        else:  # baseline
            if not args.allow_untrusted_baseline:
                reason = (
                    "baseline arm executes model-generated Python in-process; "
                    "re-run with --allow-untrusted-baseline inside a sandbox (container/VM)"
                )
                arms_report["baseline"] = {"ran": False, "skipped": reason}
                print(f"SKIP baseline arm: {reason}", file=sys.stderr)
                continue
            print(
                "WARNING: --allow-untrusted-baseline executes generated code in-process. "
                "Ensure you are inside a disposable sandbox.",
                file=sys.stderr,
            )
            checker = BaselineChecker(allow_untrusted=True)
            report = run_arm(generator, checker, "baseline", TASKS, args.max_iters)
            arms_report["baseline"] = _arm_metrics(report)

    mycelium = arms_report.get("mycelium", {})
    sc5b = mycelium.get("first_attempt_pass_rate") if isinstance(mycelium, dict) else None
    document: dict[str, object] = {
        "experiment": "KC-2 LLM-leverage (M-002; Foundation §6 P0.2)",
        "run_utc": datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ"),
        "backend": "server" if args.server else "llama-cli",
        "model": (args.server or _find_model(args.model)),
        "task_count": len(TASKS),
        "edit_to_fix_budget": args.max_iters,
        "seed": args.seed,
        "arms": arms_report,
        "sc5b": sc5b,
        "verdict": _VERDICT,
    }
    # Descriptive assessment of the run (analysis, not a verdict — VR-5). One object,
    # two projections (G11): structured in the JSON, human-readable to the console.
    assessment = assess(document)
    document["assessment"] = assessment
    summary_text = render_summary(document, assessment)

    blob = json.dumps(document, indent=2, sort_keys=True)
    if args.out:
        out_path = Path(args.out).expanduser()
        out_path.write_text(blob + "\n", encoding="utf-8")
        out_path.with_suffix(out_path.suffix + ".summary.txt").write_text(
            summary_text, encoding="utf-8"
        )
        print(f"Wrote report: {out_path}  (+ {out_path.name}.summary.txt)", file=sys.stderr)
    print(blob)
    print(summary_text, file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
