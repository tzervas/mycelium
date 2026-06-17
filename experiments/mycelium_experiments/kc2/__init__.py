"""KC-2 LLM-leverage experiment harness (M-002; Foundation §6 P0.2; SC-5b; G10).

The fixed benchmark: tasks generated in the **minimal Mycelium surface fragment** versus a
**Python-embedded DSL baseline**, measuring (1) syntactic validity, (2) type-check pass rate,
and (3) edit-to-fix iterations.

This package is the *harness* — the structural deliverable. **Running** the experiment needs an
LLM generator. The ``llm`` module + the ``python -m mycelium_experiments.kc2`` entry point now
supply one backed by a **local llama.cpp model** (the same model ``tools/llm-harness`` fetches),
so a run no longer requires a hosted API. A real run still does not *establish* the KC-2 verdict:
that is a maintainer-written analysis of the measured rates, never pre-written here (VR-5).

Layout:
- ``tasks``    — the fixed benchmark task set, each with reference solutions for both arms
  (the references prove the benchmark is well-posed; they are *not* used to score generators).
- ``baseline`` — the Python-embedded DSL (the comparison arm of R6/G10).
- ``checkers`` — the pass/fail oracles: ``myc-check`` (parse + typecheck + task signature) for
  the Mycelium arm; AST-parse + DSL execution for the baseline arm.
- ``harness``  — the generator protocol, the edit-to-fix loop, metrics, and the report.
- ``llm``      — a local-llama.cpp-backed generator + primers (generator configuration).
- ``__main__`` — the runnable entry point (``python -m mycelium_experiments.kc2``).
"""

from mycelium_experiments.kc2.baseline import Bin, Tern, bnot, swap, tadd, xor
from mycelium_experiments.kc2.checkers import (
    BaselineChecker,
    CheckResult,
    MyceliumChecker,
    ToolUnavailable,
)
from mycelium_experiments.kc2.harness import (
    ArmReport,
    StaticGenerator,
    TaskOutcome,
    run_arm,
    run_experiment,
)
from mycelium_experiments.kc2.llm import (
    LlamaGenerator,
    auto_ctx_size,
    auto_gpu_layers,
    build_prompt,
    cli_backend,
    detect_gpu,
    detect_memory,
    extract_source,
    primer_for,
    reclaim_memory,
    server_backend,
)
from mycelium_experiments.kc2.runner import RunConfig, run_one, run_suite
from mycelium_experiments.kc2.server import (
    ensure_server,
    find_server_pids,
    server_healthy,
    stop_external_servers,
    stop_server,
)
from mycelium_experiments.kc2.summary import assess, render_summary
from mycelium_experiments.kc2.tasks import TASKS, Task

__all__ = [
    "TASKS",
    "ArmReport",
    "BaselineChecker",
    "Bin",
    "CheckResult",
    "LlamaGenerator",
    "MyceliumChecker",
    "RunConfig",
    "StaticGenerator",
    "Task",
    "TaskOutcome",
    "Tern",
    "ToolUnavailable",
    "assess",
    "ensure_server",
    "auto_ctx_size",
    "auto_gpu_layers",
    "bnot",
    "build_prompt",
    "cli_backend",
    "detect_gpu",
    "detect_memory",
    "extract_source",
    "find_server_pids",
    "primer_for",
    "reclaim_memory",
    "render_summary",
    "run_arm",
    "run_experiment",
    "run_one",
    "run_suite",
    "server_backend",
    "server_healthy",
    "stop_external_servers",
    "stop_server",
    "swap",
    "tadd",
    "xor",
]
