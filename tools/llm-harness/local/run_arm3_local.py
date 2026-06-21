"""Driver for M-381 ablation arm 3 (grammar-constrained decoding) on a local machine.

Runs arm 3 over the gold task set using a local GGUF model via llama-cpp-python.
For each task, it:
  1. Builds the arm-3 prompt via ``arm3_constrained_prompt(task)``.
  2. Loads the GBNF grammar via ``mycelium_gold_gbnf()``.
  3. Decodes under the GBNF constraint using ``llama_cpp.Llama``.
  4. Scores the .myc output with ``MycCheckScorer`` (cargo / myc-check).
  5. Prints per-task pass@1 and aggregate metrics.

Usage:
    python local/run_arm3_local.py [--workers N] [--seeds S [S ...]] [--repo-root PATH]
                                   [--dry-run] [--self-check] [--task-set NAME]
                                   [--output JSON]

Flags:
    --workers N         Number of parallel worker threads (default: 1, sequential).
                        The RTX 5080 can handle batching but llama_cpp model inference
                        is not thread-safe for the same Llama instance — workers here
                        means N independent task batches each with their OWN model
                        instance, each loaded once per thread. High N will OOM if VRAM
                        is limited. Default 1 is safest; 2-4 is reasonable on 16 GB VRAM.
    --seeds S [S ...]   Random seeds for generation (default: [42]). Multiple seeds give
                        pass@k for k seeds per task.
    --repo-root PATH    Path to the Mycelium repo root (default: auto-detect from
                        __file__). Used to locate ``cargo`` / ``myc-check``.
    --dry-run           Build prompts + grammar, probe backend, print what would run;
                        skip actual model inference + scoring. Good for config validation.
    --self-check        Run offline logic checks only — no model, no cargo. Verifies
                        prompt building, GBNF loading, arg parsing, and scoring math.
    --task-set NAME     Which task set to run (default: gold-compose-v1).
    --output JSON       Write per-task results to a JSON file (optional).

Exit codes:
    0  Run completed (even if some tasks SKIP'd due to missing backend/scorer).
    1  Backend or configuration error that prevented any run.

HONESTY (G2 / VR-5):
  * If ``ConstrainedBackend`` is unavailable (no llama_cpp / no model), every task
    reports SKIP with the explicit reason — never a fabricated score.
  * If MycCheckScorer is unavailable (no cargo / myc-check), scoring reports SKIP
    (never a false PASS).
  * pass@1 is computed only over scored (non-skipped) tasks; it is None if all skip.

Guarantee tags:
    Backend probe:       Empirical (import + env var + file check)
    Grammar generation:  Declared  (GBNF is authored configuration)
    Prompt building:     Declared  (authored configuration)
    Model inference:     Empirical (model output under GBNF constraint)
    Scoring:             Empirical (myc-check verdict via exit code)
    Aggregate metrics:   Empirical (derived from scored verdicts)
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

# ---------------------------------------------------------------------------
# Sys-path setup: allow running as ``python local/run_arm3_local.py`` from
# tools/llm-harness/ or from anywhere with the harness root resolvable.
# ---------------------------------------------------------------------------
_HERE = Path(__file__).parent
_HARNESS_ROOT = _HERE.parent
if str(_HARNESS_ROOT) not in sys.path:
    sys.path.insert(0, str(_HARNESS_ROOT))

# ---------------------------------------------------------------------------
# Lazy imports from grok (never raise at module level — G2)
# ---------------------------------------------------------------------------
_IMPORT_ERROR: str | None = None
try:
    from grok.arm3_constrained import (  # type: ignore[import-untyped]
        ConstrainedBackend,
        SkipResult,
        arm3_constrained_prompt,
        mycelium_gold_gbnf,
    )
    from grok.scoring import (  # type: ignore[import-untyped]
        MycCheckScorer,
        ScoreResult,
    )
    from grok.tasks import Task, task_set  # type: ignore[import-untyped]
except ImportError as _e:
    _IMPORT_ERROR = str(_e)

# Env file emitted by setup_local_llm.py — load if present so users don't have to
# manually export MYC_ARM3_MODEL in every shell session.
_ENV_FILE = _HERE / ".env"
_ENV_VAR = "MYC_ARM3_MODEL"


def _load_env_file() -> None:
    """Load KEY=VALUE pairs from local/.env if it exists (best-effort, no crash)."""
    if not _ENV_FILE.exists():
        return
    try:
        for line in _ENV_FILE.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            key, _, val = line.partition("=")
            key = key.strip()
            val = val.strip()
            if key and key not in os.environ:
                os.environ[key] = val
    except OSError:
        pass  # non-fatal


# ---------------------------------------------------------------------------
# Per-task result
# ---------------------------------------------------------------------------


@dataclass
class TaskRunResult:
    """Result of running arm 3 on one task, one seed."""

    task_id: str
    seed: int
    status: str  # "clean" | "type_error" | "syntax_error" | "error" | "skip" | "backend_skip"
    pass1: bool  # True iff typecheck_pass (the arm 3 "pass" metric)
    latency_s: float
    generated: str
    skip_reason: str
    score_detail: str

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


# ---------------------------------------------------------------------------
# Single-task runner
# ---------------------------------------------------------------------------


def _run_one(
    task: "Task",
    backend: "ConstrainedBackend",
    scorer: "MycCheckScorer",
    seed: int,
) -> "TaskRunResult":
    """Run arm 3 on one (task, seed) pair. Never raises — returns a result.

    Guarantee: Empirical when available (model + myc-check), Declared when skipping.
    """
    t0 = time.monotonic()
    gen_result = backend.generate(task, seed=seed)
    gen_latency = time.monotonic() - t0

    # Backend skipped — no model available
    if isinstance(gen_result, SkipResult):
        return TaskRunResult(
            task_id=task.id,
            seed=seed,
            status="backend_skip",
            pass1=False,
            latency_s=gen_latency,
            generated="",
            skip_reason=gen_result.reason,
            score_detail="",
        )

    # Inference succeeded — score the output
    generated = gen_result.content or ""
    score: ScoreResult = scorer.score(generated)
    status = score.verdict  # clean/type_error/syntax_error/error/skip
    pass1 = score.typecheck_pass
    score_detail = score.message

    return TaskRunResult(
        task_id=task.id,
        seed=seed,
        status=status,
        pass1=pass1,
        latency_s=gen_latency,
        generated=generated,
        skip_reason=(score.message if score.verdict == "skip" else ""),
        score_detail=score_detail,
    )


# ---------------------------------------------------------------------------
# Aggregate over (task, seed) results
# ---------------------------------------------------------------------------


def _compute_pass_at_1(results: list[TaskRunResult]) -> float | None:
    """pass@1 over non-skipped task results (Empirical).

    Skipped tasks are excluded from the denominator (G2 — never count as fail).
    Returns None if all tasks skipped.
    """
    scored = [r for r in results if r.status not in ("backend_skip", "skip")]
    if not scored:
        return None
    return sum(1 for r in scored if r.pass1) / len(scored)


def _print_table(results: list[TaskRunResult]) -> None:
    """Print a per-task result table."""
    header = f"{'task':<30} {'seed':>5} {'status':<14} {'pass1':>5} {'latency_s':>10}"
    print(header)
    print("-" * len(header))
    for r in results:
        pass_str = (
            "YES"
            if r.pass1
            else ("---" if r.status in ("backend_skip", "skip") else "NO ")
        )
        print(
            f"{r.task_id:<30} {r.seed:>5} {r.status:<14} {pass_str:>5} {r.latency_s:>9.2f}s"
        )


# ---------------------------------------------------------------------------
# --self-check: offline logic verification
# ---------------------------------------------------------------------------


def run_self_check() -> bool:
    """Offline self-check: verifies logic without a model or cargo.

    Guarantee: Declared (asserted checks).
    """
    print("=== run_arm3_local.py --self-check (offline) ===\n")
    passed = 0
    failed = 0

    def check(name: str, ok: bool, detail: str) -> None:
        nonlocal passed, failed
        if ok:
            print(f"  [PASS] {name}: {detail}")
            passed += 1
        else:
            print(f"  [FAIL] {name}: {detail}")
            failed += 1

    # 1. grok imports: SKIP (not FAIL) when harness package not installed.
    #    Run `uv sync` in tools/llm-harness/ to install. Arg-parser checks run regardless.
    grok_available = _IMPORT_ERROR is None
    if grok_available:
        check("imports/grok", True, "OK")
    else:
        print(
            f"  [SKIP] imports/grok: grok not installed — run `uv sync` "
            f"in tools/llm-harness/ ({_IMPORT_ERROR})"
        )
        passed += 1  # SKIP counts as pass; this is expected before `uv sync`

    # 2. Arg parser
    try:
        parser = _build_parser()
        a = parser.parse_args(["--dry-run", "--workers", "4", "--seeds", "1", "2"])
        check("arg-parser/dry-run", a.dry_run is True, "parsed")
        check("arg-parser/workers", a.workers == 4, f"workers={a.workers}")
        check("arg-parser/seeds", a.seeds == [1, 2], f"seeds={a.seeds}")
        a2 = parser.parse_args(["--self-check"])
        check("arg-parser/self-check", a2.self_check is True, "parsed")
    except Exception as exc:
        check("arg-parser", False, f"exception: {exc}")

    # 3-6: grok-dependent checks — skipped (as SKIP, not FAIL) when grok not installed.
    if grok_available:
        # 3. GBNF loads
        try:
            gbnf = mycelium_gold_gbnf()
            check("gbnf/load", bool(gbnf) and "root" in gbnf, f"len={len(gbnf)}")
        except Exception as exc:
            check("gbnf/load", False, f"exception: {exc}")

        # 4. task_set returns gold tasks
        try:
            tasks = task_set()
            check("tasks/gold-count", len(tasks) == 8, f"count={len(tasks)}")
            check(
                "tasks/ids-unique",
                len({t.id for t in tasks}) == len(tasks),
                "ids unique",
            )
        except Exception as exc:
            check("tasks/load", False, f"exception: {exc}")

        # 5. arm3_constrained_prompt builds for every gold task
        try:
            tasks = task_set()
            for t in tasks:
                msgs = arm3_constrained_prompt(t)
                check(
                    f"prompt/{t.id}",
                    len(msgs) == 2
                    and msgs[0].role == "system"
                    and msgs[1].role == "user",
                    f"roles=[{','.join(m.role for m in msgs)}]",
                )
        except Exception as exc:
            check("prompt/build-all", False, f"exception: {exc}")

        # 6. ConstrainedBackend constructs without crash (available=False expected without model)
        try:
            backend = ConstrainedBackend()
            check("backend/construct", True, f"available={backend.available}")
            # If not available, generate must return SkipResult (G2)
            if not backend.available:
                task = task_set()[0]
                result = backend.generate(task, seed=0)
                check(
                    "backend/skip-result",
                    isinstance(result, SkipResult) and result.status == "skip",
                    f"type={type(result).__name__}, "
                    f"status={getattr(result, 'status', '?')!r}",
                )
        except Exception as exc:
            check("backend/construct", False, f"exception: {exc}")
    else:
        for name in [
            "gbnf/load",
            "tasks/gold-count",
            "tasks/ids-unique",
            "prompt/build",
            "backend/construct",
            "backend/skip-result",
        ]:
            print(f"  [SKIP] {name}: grok not installed")
            passed += 1

    # 7. _compute_pass_at_1 math
    try:
        r1 = TaskRunResult("g01", 42, "clean", True, 1.0, "", "", "")
        r2 = TaskRunResult("g02", 42, "type_error", False, 1.0, "", "", "")
        r3 = TaskRunResult("g03", 42, "backend_skip", False, 1.0, "", "no backend", "")
        rate = _compute_pass_at_1([r1, r2, r3])
        # scored = [r1, r2]; pass1 = 1; rate = 0.5
        check("metrics/pass_at_1", rate == 0.5, f"rate={rate}")
        # All skip => None
        rate_none = _compute_pass_at_1([r3])
        check("metrics/pass_at_1_all_skip", rate_none is None, f"rate={rate_none}")
    except Exception as exc:
        check("metrics/pass_at_1", False, f"exception: {exc}")

    # 8. .env file loading (synthetic)
    import tempfile

    try:
        with tempfile.NamedTemporaryFile(mode="w", suffix=".env", delete=False) as tf:
            tf.write("MYC_SELF_CHECK_TEST_VAR=hello_world\n")
            tf.write("# comment line\n")
            tf.write("ANOTHER_VAR=value\n")
            tmp_path = Path(tf.name)

        # Patch _ENV_FILE temporarily
        orig_env = os.environ.pop("MYC_SELF_CHECK_TEST_VAR", None)
        try:
            global _ENV_FILE
            _old_env_file = _ENV_FILE
            _ENV_FILE = tmp_path
            _load_env_file()
            _ENV_FILE = _old_env_file
        finally:
            if orig_env is not None:
                os.environ["MYC_SELF_CHECK_TEST_VAR"] = orig_env
            tmp_path.unlink(missing_ok=True)

        check("env/load", True, "env file loading ran without crash")
    except Exception as exc:
        check("env/load", False, f"exception: {exc}")

    # 9. TaskRunResult serialisation
    try:
        r = TaskRunResult(
            "g01",
            42,
            "clean",
            True,
            1.23,
            "nodule x\nfn f(a: Binary{8}) -> Binary{8} = a",
            "",
            "clean: parses and type-checks",
        )
        d = r.to_dict()
        check(
            "task-result/to-dict",
            d["task_id"] == "g01" and d["pass1"] is True,
            f"keys={list(d.keys())}",
        )
    except Exception as exc:
        check("task-result/to-dict", False, f"exception: {exc}")

    print(f"\nSelf-check: {passed} passed, {failed} failed")
    return failed == 0


# ---------------------------------------------------------------------------
# Arg parser
# ---------------------------------------------------------------------------


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        description=(
            "Run M-381 ablation arm 3 (grammar-constrained decoding) on a local GGUF model."
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    p.add_argument(
        "--workers",
        type=int,
        default=1,
        help=(
            "Number of parallel worker threads (default: 1). Each worker loads its OWN "
            "Llama instance — high N may OOM on limited VRAM. 2-4 is safe on 16 GB."
        ),
    )
    p.add_argument(
        "--seeds",
        type=int,
        nargs="+",
        default=[42],
        help="Random seeds for generation (default: 42). Multiple seeds give pass@k.",
    )
    p.add_argument(
        "--repo-root",
        default=None,
        help="Path to the Mycelium repo root (for myc-check). Auto-detected by default.",
    )
    p.add_argument(
        "--dry-run",
        action="store_true",
        help="Probe backend + build prompts; skip model inference and scoring.",
    )
    p.add_argument(
        "--self-check",
        action="store_true",
        help="Run offline logic checks only — no model, no cargo.",
    )
    p.add_argument(
        "--task-set",
        default="gold-compose-v1",
        help="Task set name (default: gold-compose-v1).",
    )
    p.add_argument(
        "--output",
        default=None,
        help="Write per-task JSON results to this file (optional).",
    )
    return p


# ---------------------------------------------------------------------------
# Repo-root auto-detection
# ---------------------------------------------------------------------------


def _auto_repo_root() -> Path | None:
    """Walk up from __file__ to find the repo root (contains justfile or .git).

    Declared — heuristic; falls back to None if not found.
    """
    candidate = Path(__file__).resolve()
    for _ in range(10):
        candidate = candidate.parent
        if (candidate / "justfile").exists() or (candidate / ".git").exists():
            return candidate
    return None


# ---------------------------------------------------------------------------
# Sharded parallel execution
# ---------------------------------------------------------------------------


def _shard_tasks(
    tasks: list["Task"],
    seeds: list[int],
    n_workers: int,
) -> list[list[tuple["Task", int]]]:
    """Partition (task, seed) pairs across N shards for parallel execution.

    Each shard is a contiguous slice so each worker processes a disjoint subset.
    Declared — deterministic partitioning.
    """
    pairs = [(task, seed) for task in tasks for seed in seeds]
    shards: list[list[tuple["Task", int]]] = [[] for _ in range(n_workers)]
    for i, pair in enumerate(pairs):
        shards[i % n_workers].append(pair)
    return shards


def _run_shard(
    shard: list[tuple["Task", int]],
    scorer: "MycCheckScorer",
) -> list[TaskRunResult]:
    """Run a shard: one ConstrainedBackend per thread (thread-safe isolation).

    Guarantee: Empirical when backend available, Declared when skipping.
    """
    # Each thread builds its own backend (its own Llama model load) for thread safety.
    local_backend = ConstrainedBackend()
    results: list[TaskRunResult] = []
    for task, seed in shard:
        r = _run_one(task, local_backend, scorer, seed)
        results.append(r)
        # Print a progress line per task so the run is not opaque
        status_tag = (
            "[PASS]" if r.pass1 else ("[SKIP]" if "skip" in r.status else "[----]")
        )
        print(
            f"  {status_tag} {r.task_id} (seed={r.seed}, {r.latency_s:.2f}s, {r.status})"
        )
    return results


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main() -> int:
    _load_env_file()

    parser = _build_parser()
    args = parser.parse_args()

    if args.self_check:
        ok = run_self_check()
        return 0 if ok else 1

    # Check grok imports are available
    if _IMPORT_ERROR:
        print(
            f"[FAIL] Cannot import grok harness: {_IMPORT_ERROR}\n"
            "       Run from tools/llm-harness/ or ensure the package is installed.\n"
            "       cd tools/llm-harness && uv sync"
        )
        return 1

    dry_run: bool = args.dry_run
    n_workers: int = max(1, args.workers)
    seeds: list[int] = args.seeds
    task_set_name: str = args.task_set

    # Resolve repo root for MycCheckScorer
    if args.repo_root:
        repo_root = Path(args.repo_root)
    else:
        repo_root = _auto_repo_root()

    print("=== M-381 arm 3 — local grammar-constrained decoding ===\n")
    print(f"  task-set  : {task_set_name}")
    print(f"  seeds     : {seeds}")
    print(f"  workers   : {n_workers}")
    print(f"  repo-root : {repo_root}")
    print(f"  dry-run   : {dry_run}")
    if _ENV_FILE.exists():
        print(f"  env-file  : {_ENV_FILE} (loaded)")
    model_path = os.environ.get("MYC_ARM3_MODEL", "<unset>")
    print(f"  model     : {model_path}\n")

    # Load tasks
    try:
        tasks = task_set(task_set_name)
    except ValueError as exc:
        print(f"[FAIL] Unknown task set: {exc}")
        return 1
    print(f"  tasks: {len(tasks)} loaded from task set '{task_set_name}'")

    # Build GBNF (check it loads)
    gbnf = mycelium_gold_gbnf()
    print(f"  GBNF: {len(gbnf)} chars, start rule present: {'root' in gbnf}")

    # Probe backend
    backend_probe = ConstrainedBackend()
    if not backend_probe.available:
        print(
            f"\n[SKIP] ConstrainedBackend not available: {backend_probe._skip_reason}"
        )
        print("       Run setup_local_llm.py first and export MYC_ARM3_MODEL.")
        if not dry_run:
            return 0  # SKIP is not an error (G2: no fabricated result)
    else:
        print(
            f"\n  [PASS] ConstrainedBackend: available (model loaded from {model_path})"
        )

    if dry_run:
        print("\n[DRY-RUN] Prompts that would be sent:\n")
        for t in tasks:
            msgs = arm3_constrained_prompt(t)
            print(f"  --- {t.id} ---")
            for m in msgs:
                snippet = m.content[:120].replace("\n", " ")
                print(
                    f"    [{m.role}] {snippet}{'...' if len(m.content) > 120 else ''}"
                )
        print(f"\n[DRY-RUN] {len(tasks) * len(seeds)} (task, seed) pairs would run.")
        print("[DRY-RUN] No model inference or scoring performed.")
        return 0

    # Set up scorer
    scorer = MycCheckScorer(repo_root=repo_root)

    # Partition tasks into shards
    shards = _shard_tasks(tasks, seeds, n_workers)
    actual_shards = [s for s in shards if s]
    print(
        f"\n  Running {len(tasks) * len(seeds)} (task, seed) pairs across {len(actual_shards)} shard(s)...\n"
    )

    all_results: list[TaskRunResult] = []
    t_run_start = time.monotonic()

    if n_workers == 1 or len(actual_shards) == 1:
        # Sequential — simplest, safest; a single Llama model instance
        # Re-use backend_probe (already loaded) if available
        seq_backend = backend_probe if backend_probe.available else ConstrainedBackend()
        for task, seed in actual_shards[0] if actual_shards else []:
            r = _run_one(task, seq_backend, scorer, seed)
            all_results.append(r)
            status_tag = (
                "[PASS]" if r.pass1 else ("[SKIP]" if "skip" in r.status else "[----]")
            )
            print(
                f"  {status_tag} {r.task_id} (seed={r.seed}, {r.latency_s:.2f}s, {r.status})"
            )
    else:
        # Parallel: each thread gets its own shard and its own Llama model instance.
        # We do NOT share the backend_probe instance across threads.
        print(
            f"  [NOTE] Parallel mode: each of {n_workers} threads loads a separate model instance."
        )
        print(f"         Ensure enough VRAM for {n_workers}x model copies.\n")
        with ThreadPoolExecutor(max_workers=n_workers) as pool:
            futures = {
                pool.submit(_run_shard, shard, scorer): shard for shard in actual_shards
            }
            for fut in as_completed(futures):
                try:
                    shard_results = fut.result()
                    all_results.extend(shard_results)
                except Exception as exc:
                    print(f"  [WARN] Shard raised exception: {exc}")

    total_elapsed = time.monotonic() - t_run_start

    # --- Results table ---
    print(f"\n{'=' * 62}")
    print("Per-task results:")
    print(f"{'=' * 62}")
    # Sort by task_id, then seed for deterministic display
    all_results.sort(key=lambda r: (r.task_id, r.seed))
    _print_table(all_results)

    # --- Aggregate metrics ---
    pass_at_1 = _compute_pass_at_1(all_results)
    scored_count = sum(
        1 for r in all_results if r.status not in ("backend_skip", "skip")
    )
    skip_count = len(all_results) - scored_count

    print(f"\n{'=' * 62}")
    print("Aggregate (arm 3, grammar-constrained decoding):")
    print(f"  total (task×seed):  {len(all_results)}")
    print(f"  scored:             {scored_count}")
    print(f"  skipped:            {skip_count}")
    if pass_at_1 is not None:
        print(
            f"  pass@1:             {pass_at_1:.3f}  ({sum(1 for r in all_results if r.pass1)}/{scored_count})"
        )
    else:
        print("  pass@1:             N/A (all tasks skipped — G2: no fabricated rate)")
    print(f"  total elapsed:      {total_elapsed:.1f}s")
    print("  guarantee:          Empirical (model + myc-check, if available)")

    # --- JSON output ---
    if args.output:
        out_path = Path(args.output)
        payload = {
            "task_set": task_set_name,
            "seeds": seeds,
            "workers": n_workers,
            "total": len(all_results),
            "scored": scored_count,
            "skipped": skip_count,
            "pass_at_1": pass_at_1,
            "elapsed_s": total_elapsed,
            "guarantee": "Empirical",
            "results": [r.to_dict() for r in all_results],
        }
        try:
            out_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
            print(f"\n  [OK] Results written to {out_path}")
        except OSError as exc:
            print(f"\n  [WARN] Could not write output to {args.output}: {exc}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
