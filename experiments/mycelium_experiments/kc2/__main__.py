"""Run the KC-2 LLM-leverage experiment (M-002) against a local llama.cpp model.

    # auto-managed server (recommended): loads the model once, no interactive REPL,
    # picks a free port and tears the server down afterwards:
    python -m mycelium_experiments.kc2 --serve --max-iters 1

    # a sequence of seeds, unattended; one report each + an index.json:
    python -m mycelium_experiments.kc2 --serve --seeds 42,123,7

    # against an already-running server, or the CLI backend:
    python -m mycelium_experiments.kc2 --server http://localhost:8080
    python -m mycelium_experiments.kc2 --model PATH.gguf        # CLI (EOF-guarded)

Reports land in --results-dir (default experiments/results/): per run a
<utc>-<name>.json + .summary.txt, plus index.json and a suite .log.

Honesty (house rules): NEVER-SILENT (G2) — an unavailable checker SKIPs its arm with a
reason; a backend/server error aborts loudly. VR-5 — measured RATES only; the KC-2
verdict is a maintainer-written analysis, never computed here. The baseline arm executes
generated Python in-process and is OFF unless --allow-untrusted-baseline (sandbox only).
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path

from mycelium_experiments.kc2 import llm, server
from mycelium_experiments.kc2.runner import RunConfig, make_logger, now_utc, run_suite
from mycelium_experiments.kc2.tasks import TASKS

# Mirror tools/llm-harness's cache layout so a model fetched there is reused here.
# Preference order when --model is not given: fastest cached coder model first. The
# 0.5B decodes ~2-3x quicker than the 1.5B on a phone CPU (generation time dominates a
# sweep); the 1.5B is the stronger fallback. Fetch either via the llm-harness, e.g.
#   python tools/llm-harness/harness.py --ensure-model --model-id qwen2.5-coder-0.5b
_PREFERRED_MODELS = (
    "qwen2.5-coder-0.5b-instruct-q4_k_m.gguf",
    "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf",
)
# KC-2 prompts (primer + task + feedback + generation) want more context than the
# validation harness — still capped to fit available RAM (auto).
_KC2_DESIRED_CTX = 2048


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
    for name in _PREFERRED_MODELS:  # fastest cached coder model first
        cand = md / name
        if cand.is_file():
            return str(cand)
    try:
        for p in sorted(md.glob("*.gguf")):
            if p.is_file():
                return str(p)
    except OSError:
        pass
    return None


def _read_primer(path: str | None) -> str | None:
    return Path(path).expanduser().read_text(encoding="utf-8") if path else None


def _require_model(explicit: str | None) -> str:
    model = _find_model(explicit)
    if not model:
        sys.exit(
            "ERROR: no model. Pass --model PATH.gguf, or fetch the fast 0.5B coder with "
            "`python tools/llm-harness/harness.py --ensure-model --model-id "
            f"qwen2.5-coder-0.5b` (caches into {_default_model_dir()})."
        )
    return model


def _resolve_ctx(args: argparse.Namespace, model: str) -> int:
    if args.ctx_size is not None:
        return int(args.ctx_size)
    ctx, reason = llm.auto_ctx_size(
        _KC2_DESIRED_CTX, model, llm.detect_memory(), swap_fraction=0.5 if args.use_swap else 0.0
    )
    print(f"Auto-ctx: {reason} (override with --ctx-size)", file=sys.stderr)
    return ctx


def _resolve_ngl(args: argparse.Namespace, model: str) -> int:
    if args.cpu_only:
        return 0
    if args.n_gpu_layers is not None:
        return int(args.n_gpu_layers)
    ngl, reason = llm.auto_gpu_layers(llm.detect_gpu(), model)
    print(f"Auto-GPU: {reason}", file=sys.stderr)
    return ngl


def _parse_seeds(args: argparse.Namespace) -> list[int]:
    if args.seeds:
        return [int(s.strip()) for s in args.seeds.split(",") if s.strip()]
    return [int(args.seed)]


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="python -m mycelium_experiments.kc2",
        description="Run the KC-2 LLM-leverage experiment against a local llama.cpp model.",
    )
    src = p.add_mutually_exclusive_group()
    src.add_argument(
        "--serve", action="store_true", help="Auto-launch + manage a llama-server (recommended)."
    )
    src.add_argument("--server", metavar="URL", help="Use an already-running llama.cpp server.")
    src.add_argument("--llama-cli", metavar="PATH", help="Use the `llama`/`llama-cli` backend.")
    p.add_argument("--model", metavar="PATH", help="Path to a .gguf model (server/cli backends).")
    p.add_argument("--server-binary", metavar="PATH", help="Path to `llama-server` (for --serve).")
    p.add_argument("--host", default="127.0.0.1", help="Host for --serve (default 127.0.0.1).")
    p.add_argument("--port", type=int, default=None, help="Port for --serve (default: a free one).")
    p.add_argument(
        "--keep-server",
        action="store_true",
        help="With --serve, leave the managed server running afterwards (default: tear it down).",
    )
    p.add_argument(
        "--stop-server",
        action="store_true",
        help="Reap running llama-server processes (orphans from a manual launch) and exit. "
        "Use --port N to target one.",
    )
    p.add_argument(
        "--arms", default="mycelium", help="Comma-separated: mycelium,baseline (default mycelium)."
    )
    p.add_argument(
        "--allow-untrusted-baseline",
        action="store_true",
        help="Run the baseline arm, which EXECUTES generated Python in-process. Sandbox only.",
    )
    p.add_argument(
        "--max-iters",
        type=int,
        default=2,
        help="Attempts per task before moving on (default 2: first try + one edit-to-fix).",
    )
    p.add_argument(
        "--limit", type=int, default=None, metavar="N", help="Run only the first N tasks (lighter)."
    )
    p.add_argument("--seed", type=int, default=42, help="Generation seed (default 42).")
    p.add_argument("--seeds", metavar="A,B,C", help="Run a SEQUENCE of seeds, one report each.")
    p.add_argument(
        "--n-predict",
        type=int,
        default=128,
        help="Max new tokens per generation (default 128). The task solutions are short; "
        "lower = faster on a slow CPU, but too low truncates a valid program.",
    )
    p.add_argument(
        "--timeout",
        type=int,
        default=600,
        help="PER-GENERATION timeout in seconds (default 600). It refreshes every attempt — "
        "there is no cumulative suite timeout — so raise it on a glacial phone (~0.5 tok/s) "
        "rather than letting a slow but valid generation get cut off.",
    )
    p.add_argument(
        "--ctx-size",
        type=int,
        default=None,
        help="Context window -c. DEFAULT auto (sized from free RAM; avoids the 32k OOM).",
    )
    p.add_argument(
        "--use-swap",
        action="store_true",
        help="Count ~half of free swap toward the auto ctx budget.",
    )
    p.add_argument("--cpu-only", action="store_true", help="Force CPU only (no GPU offload).")
    p.add_argument(
        "--n-gpu-layers", type=int, default=None, help="GPU layers -ngl (default auto from VRAM)."
    )
    p.add_argument("--primer-mycelium", metavar="FILE", help="Override the Mycelium-arm primer.")
    p.add_argument("--primer-baseline", metavar="FILE", help="Override the baseline-arm primer.")
    p.add_argument(
        "--llama-extra-arg",
        action="append",
        default=[],
        metavar="ARG",
        help="Extra arg for the llama CLI (repeatable).",
    )
    p.add_argument(
        "--no-reclaim",
        action="store_true",
        help="Skip the gentle pre-run RAM reclaim (gc/malloc_trim/sync, + drop_caches if root).",
    )
    p.add_argument(
        "--results-dir", metavar="DIR", help="Where reports land (default experiments/results/)."
    )
    p.add_argument("--out", metavar="PATH", help="Also copy the (single-seed) JSON report here.")
    return p


def main() -> int:
    args = _build_parser().parse_args()

    # Standalone teardown: reap llama-server processes (e.g. an orphan from a manual
    # `llama-server … &`) and exit. No run, no model needed.
    if args.stop_server:
        rdir = Path(args.results_dir or "results").expanduser()
        rdir.mkdir(parents=True, exist_ok=True)
        log = make_logger(rdir / f"{now_utc()}-stop.log")
        killed = server.stop_external_servers(log, port=args.port)
        print(f"Stopped {len(killed)} llama-server process(es): {killed}", file=sys.stderr)
        return 0

    arms = tuple(a.strip() for a in args.arms.split(",") if a.strip())
    if unknown := [a for a in arms if a not in ("mycelium", "baseline")]:
        sys.exit(f"unknown arm(s): {unknown} (expected mycelium and/or baseline)")
    seeds = _parse_seeds(args)

    primers: dict[str, str] = {}
    if (pm := _read_primer(args.primer_mycelium)) is not None:
        primers["mycelium"] = pm
    if (pb := _read_primer(args.primer_baseline)) is not None:
        primers["baseline"] = pb

    results_dir = Path(args.results_dir).expanduser() if args.results_dir else Path("results")
    results_dir.mkdir(parents=True, exist_ok=True)
    suite_id = now_utc()
    log = make_logger(results_dir / f"{suite_id}-suite.log")

    # Gently free RAM BEFORE sizing the context, so the freed memory is available to the
    # model + KV cache (auto_ctx_size reads memory fresh just below). Non-destructive.
    if not args.no_reclaim:
        llm.reclaim_memory(log)

    proc = None
    n_predict = args.n_predict
    try:
        # --- choose / build the backend factory (seed -> Backend) ---
        if args.server:
            if not server.server_healthy(args.server):
                log.warning("No healthy server at %s (/health). Trying anyway.", args.server)
            model_label, backend_label = args.server, "server"

            def make_backend(seed: int) -> llm.Backend:
                return llm.server_backend(
                    args.server, seed=seed, n_predict=n_predict, timeout=args.timeout
                )
        elif args.llama_cli is not None or (not args.serve):
            # CLI backend (explicit --llama-cli, or the default when neither --serve/--server)
            cli = llm.resolve_llama_cli(args.llama_cli)
            if not cli:
                sys.exit(
                    "ERROR: no `llama`/`llama-cli`. Use --serve, --server URL, --llama-cli PATH, "
                    "or run `python tools/llm-harness/harness.py --doctor`."
                )
            model = _require_model(args.model)
            ctx_size, ngl = _resolve_ctx(args, model), _resolve_ngl(args, model)
            model_label, backend_label = model, "llama-cli"

            def make_backend(seed: int) -> llm.Backend:
                return llm.cli_backend(
                    cli,
                    model,
                    seed=seed,
                    n_predict=n_predict,
                    ctx_size=ctx_size,
                    n_gpu_layers=ngl,
                    timeout=args.timeout,
                    extra_args=args.llama_extra_arg,
                )
        else:  # --serve : auto-manage a server
            model = _require_model(args.model)
            ctx_size, ngl = _resolve_ctx(args, model), _resolve_ngl(args, model)
            base_url, proc = server.ensure_server(
                model=model,
                ctx_size=ctx_size,
                n_gpu_layers=ngl,
                host=args.host,
                port=args.port,
                binary=args.server_binary,
                log=log,
                log_path=results_dir / f"{suite_id}-llama-server.log",
            )
            model_label, backend_label = model, "server(managed)"

            def make_backend(seed: int) -> llm.Backend:
                return llm.server_backend(
                    base_url, seed=seed, n_predict=n_predict, timeout=args.timeout
                )

        # --- run the sequence ---
        tasks = TASKS[: args.limit] if args.limit else TASKS
        if args.limit:
            log.info("--limit %d: running %d of %d tasks", args.limit, len(tasks), len(TASKS))
        configs = [
            RunConfig(
                name=f"seed{seed}",
                arms=arms,
                seed=seed,
                max_iters=args.max_iters,
                allow_untrusted_baseline=args.allow_untrusted_baseline,
            )
            for seed in seeds
        ]
        index = run_suite(
            configs,
            backend_factory=make_backend,
            primers=primers,
            model_label=model_label,
            backend_label=backend_label,
            results_dir=results_dir,
            tasks=tasks,
            log=log,
        )
    finally:
        # Auto-teardown after all reports + logs are written. --keep-server leaves the
        # managed server up (its URL is logged) so a follow-up run can reuse it.
        if proc is not None and args.keep_server:
            log.info("--keep-server: leaving managed llama-server running at %s", model_label)
            log.info("Stop it later: python -m mycelium_experiments.kc2 --stop-server")
        else:
            server.stop_server(proc, log)

    # Back-compat: copy the single run's JSON to --out when one seed was requested.
    if args.out and len(seeds) == 1 and index["runs"]:
        src = results_dir / index["runs"][0]["report"]  # type: ignore[index]
        dst = Path(args.out).expanduser()
        dst.write_text(src.read_text(encoding="utf-8"), encoding="utf-8")
        log.info("copied %s -> %s", src.name, dst)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
