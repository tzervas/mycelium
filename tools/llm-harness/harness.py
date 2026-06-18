#!/usr/bin/env python3
"""
Mycelium LLM-validation harness — portable, Termux-friendly (ARM/aarch64).

Design posture (non-negotiable house rules):
- NEVER-SILENT (G2, RFC-0013 I1): every failure is explicit, logged, structured.
  A missing tool/model ⇒ SKIP, never a false PASS.
- HONESTY / guarantee lattice (VR-5): model-derived claims are tagged
  Empirical or Declared — NEVER Proven or Exact. The harness enforces this.
- DUAL PROJECTION (G11): every run writes BOTH a structured JSON report AND a
  human-readable text report, as two renderers of one result object.
- Pure Python standard library only: no pip dependencies. Termux-portable.

Usage:
    python3 harness.py --mock              # dry-run: no model needed (CI/cloud)
    python3 harness.py --list-models       # show the model registry + cache dir
    python3 harness.py --ensure-model      # fetch the default model if absent, then run
    python3 harness.py --ensure-model --model-id qwen2.5-coder-14b   # desktop tier
    python3 harness.py --model PATH.gguf   # real mode via a local model file
    python3 harness.py --llama-cli PATH --model PATH.gguf
    python3 harness.py --server URL        # real mode via llama.cpp HTTP server

Exit codes:
    0   all results are PASS or SKIP (no FAILures)
    1   one or more FAILures
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import logging
import os
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Any

# ---------------------------------------------------------------------------
# Guarantee lattice (VR-5)
# ---------------------------------------------------------------------------

LATTICE_ORDERED = ["Exact", "Proven", "Empirical", "Declared"]
# Model-derived claims may only carry these two tags.
MODEL_ALLOWED_TAGS = {"Empirical", "Declared"}


def assert_model_tag(tag: str, claim_id: str) -> None:
    """Raise ValueError if tag is stronger than allowed for model-derived claims."""
    if tag not in LATTICE_ORDERED:
        raise ValueError(f"Unknown guarantee tag '{tag}' on claim '{claim_id}'")
    if tag not in MODEL_ALLOWED_TAGS:
        raise ValueError(
            f"[VR-5 VIOLATION] Claim '{claim_id}' carries tag '{tag}', "
            f"which is FORBIDDEN for model-derived claims. "
            f"Allowed: {sorted(MODEL_ALLOWED_TAGS)}. "
            "A model output may never be Proven or Exact without a checked basis."
        )


# ---------------------------------------------------------------------------
# Validation result
# ---------------------------------------------------------------------------

STATUS_PASS = "PASS"
STATUS_SKIP = "SKIP"
STATUS_FAIL = "FAIL"
# Special variant: mock-PASS means the validation ran against a fixture, not a
# real model. It is NEVER treated as a real PASS for model quality purposes.
STATUS_MOCK_PASS = "mock-PASS"


@dataclass
class ValidationResult:
    """One validation's outcome — the single canonical result type."""

    id: str
    status: str  # PASS | SKIP | FAIL | mock-PASS
    guarantee_tag: str | None  # Empirical | Declared | None (for pure-logic checks)
    message: str
    detail: dict[str, Any] = field(default_factory=dict)

    def is_failure(self) -> bool:
        return self.status == STATUS_FAIL

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


# ---------------------------------------------------------------------------
# Validation registry
# ---------------------------------------------------------------------------

ValidationFn = Any  # Callable[[RunContext], ValidationResult]

_REGISTRY: list[
    tuple[str, str, bool, ValidationFn]
] = []  # (id, desc, requires_model, fn)


def validation(vid: str, description: str, *, requires_model: bool = True):
    """Decorator: register a validation function.

    requires_model=True (default) means the check needs a real model/server; with
    none available it becomes an explicit SKIP (never a false PASS). Set False for
    pure-logic checks (e.g. the VR-5 tag gate) that run in every mode. Carrying this
    as metadata — instead of matching on a specific id — keeps NEVER-SILENT correct
    as validations are added.
    """

    def decorator(fn: ValidationFn) -> ValidationFn:
        _REGISTRY.append((vid, description, requires_model, fn))
        return fn

    return decorator


# ---------------------------------------------------------------------------
# Run context — carries all resolved config for the run
# ---------------------------------------------------------------------------


@dataclass
class RunContext:
    mock: bool
    llama_cli: str | None  # resolved path to llama-cli binary
    model_path: str | None  # resolved path to .gguf model
    server_url: str | None  # llama.cpp HTTP server base URL
    reports_dir: Path
    run_id: str  # ISO timestamp
    log: logging.Logger
    # Memory governor: cap the llama.cpp context so it does not allocate a KV cache
    # for the model's full trained window (Qwen2.5 ⇒ 32k) on a phone — that blows
    # past the Android low-memory killer (SIGKILL/9) during load. Our prompts are
    # tiny, so a small context is correct, not just safe. Tunable via --ctx-size.
    ctx_size: int = 2048
    # GPU offload: number of model layers to push to the GPU (llama.cpp -ngl). 0 = CPU
    # only (the default on a phone, where no GPU is detected). On desktop the auto-sizer
    # sets this from detected VRAM unless --cpu-only / --n-gpu-layers says otherwise.
    n_gpu_layers: int = 0
    # Per-generation subprocess timeout (s). CPU phones run ~1-2 tok/s, so the 120s
    # default is too tight once a real prompt + completion is involved; tunable.
    llama_timeout: int = 300
    extra_llama_args: list[str] = field(default_factory=list)


# ---------------------------------------------------------------------------
# Memory enumeration + auto context sizing (the OOM governor, self-tuning)
# ---------------------------------------------------------------------------
#
# The real-mode SIGKILL on a phone is a KV-cache blowout: with no `-c`, llama.cpp
# sizes the KV cache for the model's *full trained window* (Qwen2.5 = 32k). We pick
# a context that (a) is as large as the workload actually needs and (b) fits the
# device's *available* RAM with headroom — never silently, always logged (EXPLAIN).
# Swap is detected and reported but NOT counted toward the budget: KV/compute are
# active allocations that thrash (and still trip the low-memory killer) if paged.

# The harness's prompts are tiny (tens of tokens, n_predict ≤ 128), so it never needs
# a large window — auto-sizing mostly clamps DOWNWARD on small devices.
_HARNESS_DESIRED_CTX = 1024
# Conservative upper bound on KV bytes/token for a phone-class q4 model with GQA
# (≈ 2·layers·kv_dim·2B). Over-estimating here makes us under-allocate context — safe.
_KV_MB_PER_TOKEN = 0.032


def detect_memory() -> dict[str, Any]:
    """Best-effort RAM/swap enumeration in MB. Pure stdlib; never raises.

    Reads /proc/meminfo (Linux/Termux/Android) and falls back to POSIX sysconf.
    Any field we cannot determine is left None (never guessed) — honest input to the
    auto-sizer, which degrades to a conservative default when memory is unknown.
    """
    info: dict[str, Any] = {
        "mem_total_mb": None,
        "mem_available_mb": None,
        "swap_total_mb": None,
        "swap_free_mb": None,
        "source": "unknown",
    }
    try:
        meminfo = Path("/proc/meminfo")
        if meminfo.is_file():
            kv: dict[str, int] = {}
            for line in meminfo.read_text(encoding="utf-8").splitlines():
                name, _, rest = line.partition(":")
                fields = rest.strip().split()
                if fields and fields[0].isdigit():
                    kv[name.strip()] = int(fields[0])  # kB
            mb = lambda k: (kv[k] // 1024) if k in kv else None  # noqa: E731
            info["mem_total_mb"] = mb("MemTotal")
            avail = mb("MemAvailable")
            if avail is None and "MemFree" in kv:
                avail = (
                    kv.get("MemFree", 0) + kv.get("Buffers", 0) + kv.get("Cached", 0)
                ) // 1024
            info["mem_available_mb"] = avail
            info["swap_total_mb"] = mb("SwapTotal")
            info["swap_free_mb"] = mb("SwapFree")
            info["source"] = "/proc/meminfo"
            return info
    except OSError:
        pass
    try:
        page = os.sysconf("SC_PAGE_SIZE")
        names = getattr(os, "sysconf_names", {})
        if "SC_PHYS_PAGES" in names:
            info["mem_total_mb"] = os.sysconf("SC_PHYS_PAGES") * page // (1024 * 1024)
        if "SC_AVPHYS_PAGES" in names:
            info["mem_available_mb"] = (
                os.sysconf("SC_AVPHYS_PAGES") * page // (1024 * 1024)
            )
        if info["mem_total_mb"] is not None:
            info["source"] = "sysconf"
    except (OSError, ValueError):
        pass
    return info


def auto_ctx_size(
    want: int,
    model_path: str | None,
    mem: dict[str, Any],
    log: logging.Logger,
    *,
    swap_fraction: float = 0.0,
) -> int:
    """Choose a context size = min(workload need, what available memory safely holds).

    By default only physical RAM is budgeted. With ``swap_fraction > 0`` (the --use-swap
    flag), that fraction of free swap is added to the budget — letting the context grow
    when RAM is tight, at the cost of speed (KV in swap thrashes per token) and some OOM
    risk (the killer still targets RSS). Logged in full (EXPLAIN, never silent);
    conservative when memory is unknown; always overridable with --ctx-size.
    """
    avail = mem.get("mem_available_mb")
    swap_free = mem.get("swap_free_mb")
    model_mb = 0
    try:
        if model_path and os.path.isfile(model_path):
            model_mb = os.path.getsize(model_path) // (1024 * 1024)
    except OSError:
        pass

    if not isinstance(avail, int) or avail <= 0:
        chosen = min(want, 1024)
        log.warning(
            "Auto-ctx: available memory unknown (source=%s) — using a conservative "
            "ctx-size=%d. Override with --ctx-size.",
            mem.get("source"),
            chosen,
        )
        return chosen

    # Reserve headroom for the OS + the model's working set. With mmap the weights are
    # page-cache backed (reclaimable), but reserve their size anyway as a worst case.
    reserve = max(768, model_mb)
    swap_budget = (
        int(swap_free * swap_fraction)
        if (swap_fraction > 0 and isinstance(swap_free, int) and swap_free > 0)
        else 0
    )
    usable = (avail - reserve) + swap_budget
    cap = int((usable * 0.40) / _KV_MB_PER_TOKEN) if usable > 0 else 256
    cap = max(256, (cap // 256) * 256)  # floor + round to a 256 multiple
    chosen = max(256, min(want, cap))
    log.info(
        "Auto-ctx: avail=%dMB%s model=%dMB → reserve=%dMB usable=%dMB; "
        "memory allows ~%d, workload wants %d ⇒ ctx-size=%d (override with --ctx-size).",
        avail,
        f" +{swap_budget}MB swap (of {swap_free} free)" if swap_budget else "",
        model_mb,
        reserve,
        max(0, usable),
        cap,
        want,
        chosen,
    )
    if swap_budget:
        log.info(
            "Auto-ctx: counting %dMB of swap toward the budget (--use-swap) — expect "
            "slower generation if the KV cache pages out.",
            swap_budget,
        )
    if usable <= 0:
        log.warning(
            "Auto-ctx: low memory headroom — the model (%dMB) is large vs available "
            "RAM (%dMB). Try --use-swap, a smaller tier "
            "(--ensure-model --model-id qwen2.5-0.5b-instruct), or move the model to a "
            "roomier volume with --model-dir.",
            model_mb,
            avail,
        )
    return chosen


# ---------------------------------------------------------------------------
# GPU + external-storage enumeration (desktop GPU offload; SD-card overflow)
# ---------------------------------------------------------------------------


def detect_gpu() -> list[dict[str, Any]]:
    """Best-effort GPU enumeration. Returns [{name, backend, vram_total_mb, vram_free_mb}].

    Empty on a machine with no discrete-GPU tooling (e.g. a phone) — never raises. Covers
    NVIDIA (nvidia-smi), AMD (rocm-smi, presence only), and Apple Metal (macOS).
    """
    gpus: list[dict[str, Any]] = []
    smi = shutil.which("nvidia-smi")
    if smi:
        try:
            res = subprocess.run(
                [
                    smi,
                    "--query-gpu=name,memory.total,memory.free",
                    "--format=csv,noheader,nounits",
                ],
                capture_output=True,
                text=True,
                timeout=15,
            )
            if res.returncode == 0:
                for line in res.stdout.strip().splitlines():
                    parts = [p.strip() for p in line.split(",")]
                    if len(parts) >= 3 and parts[1].isdigit():
                        gpus.append(
                            {
                                "name": parts[0],
                                "backend": "cuda",
                                "vram_total_mb": int(parts[1]),
                                "vram_free_mb": int(parts[2])
                                if parts[2].isdigit()
                                else None,
                            }
                        )
        except (OSError, subprocess.SubprocessError):
            pass
    if not gpus and shutil.which("rocm-smi"):
        try:
            res = subprocess.run(
                ["rocm-smi", "--showproductname"],
                capture_output=True,
                text=True,
                timeout=15,
            )
            if res.returncode == 0 and res.stdout.strip():
                gpus.append(
                    {
                        "name": "AMD ROCm GPU",
                        "backend": "rocm",
                        "vram_total_mb": None,
                        "vram_free_mb": None,
                    }
                )
        except (OSError, subprocess.SubprocessError):
            pass
    if not gpus and sys.platform == "darwin":
        gpus.append(
            {
                "name": "Apple GPU (Metal)",
                "backend": "metal",
                "vram_total_mb": None,
                "vram_free_mb": None,
            }
        )
    return gpus


def auto_gpu_layers(
    gpus: list[dict[str, Any]], model_path: str | None, log: logging.Logger
) -> int:
    """Choose -ngl (layers to offload). Full offload when VRAM holds the model; else 0
    (CPU) with a note. Metal/ROCm with unknown VRAM ⇒ full offload (the runtime manages).

    A phone build (no GPU tooling) ⇒ no GPUs ⇒ 0, so this is a no-op there.
    """
    if not gpus:
        return 0
    g = gpus[0]
    model_mb = 0
    try:
        if model_path and os.path.isfile(model_path):
            model_mb = os.path.getsize(model_path) // (1024 * 1024)
    except OSError:
        pass
    free = g.get("vram_free_mb") or g.get("vram_total_mb")
    # Offload all when VRAM is unknown, the model size is unknown, or it comfortably
    # fits; only refuse when we KNOW the VRAM and KNOW the model won't fit.
    if not isinstance(free, int) or model_mb == 0 or free >= int(model_mb * 1.15):
        log.info(
            "Auto-GPU: %s (%s) — offloading all layers (-ngl 999). "
            "Use --cpu-only to disable, or --n-gpu-layers N to set it.",
            g["name"],
            g["backend"],
        )
        return 999
    log.warning(
        "Auto-GPU: %s — %dMB free VRAM < model %dMB (+headroom); staying on CPU "
        "(-ngl 0). Set --n-gpu-layers N for partial offload.",
        g["name"],
        free,
        model_mb,
    )
    return 0


def detect_external_storage() -> list[dict[str, Any]]:
    """Best-effort large external/SD volumes with free space (MB). Never raises.

    Termux exposes shared/SD storage under ~/storage and /storage/<UUID>; we report
    free space so a roomy SD card can host the model cache (--model-dir) or a swapfile
    (root). Informational — we never auto-mount or auto-swapon.
    """
    home = Path.home()
    paths = [
        Path("/storage/emulated/0"),
        home / "storage" / "external-1",
        home / "storage" / "shared",
    ]
    try:
        paths.extend(p for p in Path("/storage").glob("*-*"))
    except OSError:
        pass
    seen: set[str] = set()
    out: list[dict[str, Any]] = []
    for p in paths:
        try:
            if not p.exists():
                continue
            rp = os.path.realpath(p)
            if rp in seen:
                continue
            seen.add(rp)
            du = shutil.disk_usage(str(p))
            out.append(
                {
                    "path": str(p),
                    "free_mb": du.free // (1024 * 1024),
                    "total_mb": du.total // (1024 * 1024),
                }
            )
        except (OSError, ValueError):
            continue
    return out


# ---------------------------------------------------------------------------
# llama.cpp invocation helpers
# ---------------------------------------------------------------------------


def _call_llama_cli(
    ctx: RunContext,
    prompt: str,
    seed: int = 42,
    n_predict: int = 128,
    extra_args: list[str] | None = None,
) -> dict[str, Any]:
    """
    Shell out to llama-cli. Returns dict with keys:
        stdout, stderr, returncode, wall_seconds, token_counts (best-effort)
    Raises RuntimeError on non-zero exit.
    """
    cmd = [
        ctx.llama_cli,
        "--model",
        ctx.model_path,
        "--prompt",
        prompt,
        "--seed",
        str(seed),
        "--n-predict",
        str(n_predict),
        "--ctx-size",
        str(ctx.ctx_size),  # cap the KV cache — see RunContext.ctx_size (OOM governor)
        "--log-disable",  # suppress llama.cpp's internal log spam to stderr
        "-e",  # escape newlines in prompt
        # One-shot completion, verified on-device (Termux b-unknown): without these,
        # recent llama-cli enters its interactive *conversation* REPL — it generates,
        # then waits at a `>` prompt forever (the subprocess times out), and echoes the
        # prompt into stdout (polluting the V-01/V-02 parse). `-no-cnv` disables the chat
        # loop (generate, then exit at EOS); `--no-display-prompt` keeps stdout to just
        # the completion. Remove via --llama-arg only if a build rejects them.
        "-no-cnv",
        "--no-display-prompt",
    ]
    if ctx.n_gpu_layers > 0:  # offload to GPU only when one was detected/requested
        cmd += ["--n-gpu-layers", str(ctx.n_gpu_layers)]
    cmd.extend(ctx.extra_llama_args)
    if extra_args:
        cmd.extend(extra_args)

    ctx.log.debug("llama-cli cmd: %s", cmd)
    t0 = time.monotonic()
    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        timeout=ctx.llama_timeout,
        # EOF on stdin: if a build ignores -no-cnv and enters the interactive REPL, it
        # exits after the first response instead of hanging on the terminal.
        stdin=subprocess.DEVNULL,
    )
    wall = time.monotonic() - t0

    # Best-effort token count from llama.cpp stderr summary lines
    prompt_tokens = None
    gen_tokens = None
    for line in result.stderr.splitlines():
        if "prompt eval time" in line.lower() and "tokens" in line.lower():
            parts = line.split()
            for i, p in enumerate(parts):
                if p in ("tokens", "token") and i > 0:
                    try:
                        prompt_tokens = int(parts[i - 1])
                    except (ValueError, IndexError):
                        pass
        if "eval time" in line.lower() and "runs" in line.lower():
            parts = line.split()
            for i, p in enumerate(parts):
                if p in ("runs",) and i > 0:
                    try:
                        gen_tokens = int(parts[i - 1])
                    except (ValueError, IndexError):
                        pass

    if result.returncode != 0:
        raise RuntimeError(
            f"llama-cli exited {result.returncode}.\nstderr: {result.stderr[:2000]}"
        )

    return {
        "stdout": result.stdout,
        "stderr": result.stderr,
        "returncode": result.returncode,
        "wall_seconds": round(wall, 3),
        "token_counts": {
            "prompt": prompt_tokens,
            "generated": gen_tokens,
            "note": "Declared — parsed from llama.cpp stderr summary; may be None if format changed",
        },
    }


def _call_server(
    ctx: RunContext,
    prompt: str,
    seed: int = 42,
    n_predict: int = 128,
) -> dict[str, Any]:
    """
    POST to llama.cpp /completion endpoint. Pure stdlib (urllib).
    Returns same shape as _call_llama_cli.
    """
    import urllib.request

    payload = json.dumps(
        {
            "prompt": prompt,
            "seed": seed,
            "n_predict": n_predict,
            "stream": False,
        }
    ).encode()

    url = ctx.server_url.rstrip("/") + "/completion"
    req = urllib.request.Request(
        url,
        data=payload,
        headers={"Content-Type": "application/json"},
        method="POST",
    )

    ctx.log.debug("POST %s", url)
    t0 = time.monotonic()
    with urllib.request.urlopen(req, timeout=120) as resp:
        body = resp.read().decode()
    wall = time.monotonic() - t0

    data = json.loads(body)
    text = data.get("content", "")
    tokens_predicted = data.get("tokens_predicted")
    tokens_evaluated = data.get("tokens_evaluated")

    return {
        "stdout": text,
        "stderr": "",
        "returncode": 0,
        "wall_seconds": round(wall, 3),
        "token_counts": {
            "prompt": tokens_evaluated,
            "generated": tokens_predicted,
            "note": "Empirical — reported by llama.cpp server /completion response",
        },
    }


def _generate(
    ctx: RunContext,
    prompt: str,
    seed: int = 42,
    n_predict: int = 128,
    extra_args: list[str] | None = None,
) -> dict[str, Any]:
    """Dispatch to the right backend (llama-cli or server)."""
    if ctx.server_url:
        return _call_server(ctx, prompt, seed=seed, n_predict=n_predict)
    return _call_llama_cli(
        ctx, prompt, seed=seed, n_predict=n_predict, extra_args=extra_args
    )


# ---------------------------------------------------------------------------
# Validation 1: model-load + deterministic-seed round-trip
# ---------------------------------------------------------------------------

_DETERMINISM_PROMPT = "Complete this sequence with exactly the next three integers and nothing else: 1, 1, 2, 3, 5,"
_DETERMINISM_FIXTURE = "8, 13, 21"


@validation(
    "V-01-determinism",
    "Fixed-seed round-trip: same prompt ⇒ same output across two runs",
)
def v01_determinism(ctx: RunContext) -> ValidationResult:
    vid = "V-01-determinism"

    if ctx.mock:
        # Mock mode: return a synthetic fixture that demonstrates the check logic.
        out_a = _DETERMINISM_FIXTURE
        out_b = _DETERMINISM_FIXTURE
        matched = out_a.strip() == out_b.strip()
        return ValidationResult(
            id=vid,
            status=STATUS_MOCK_PASS,
            guarantee_tag="Declared",
            message=(
                "[MOCK] Determinism check simulated with fixture — "
                "not a real model run. "
                f"Fixture outputs matched: {matched}"
            ),
            detail={
                "mode": "mock",
                "fixture": _DETERMINISM_FIXTURE,
                "run_a": out_a,
                "run_b": out_b,
                "matched": matched,
                "guarantee_note": (
                    "Declared — synthetic fixture, not model-derived. "
                    "Real runs would be tagged Empirical."
                ),
            },
        )

    # Real mode
    try:
        ctx.log.info("[V-01] Run A: generating with seed=42")
        res_a = _generate(ctx, _DETERMINISM_PROMPT, seed=42, n_predict=32)
        ctx.log.info("[V-01] Run B: generating with seed=42 (repeat)")
        res_b = _generate(ctx, _DETERMINISM_PROMPT, seed=42, n_predict=32)
    except Exception as exc:
        ctx.log.exception("[V-01] Generation failed")
        return ValidationResult(
            id=vid,
            status=STATUS_FAIL,
            guarantee_tag=None,
            message=f"Generation error: {exc}",
            detail={"error": str(exc)},
        )

    out_a = res_a["stdout"].strip()
    out_b = res_b["stdout"].strip()
    matched = out_a == out_b

    return ValidationResult(
        id=vid,
        status=STATUS_PASS if matched else STATUS_FAIL,
        guarantee_tag="Empirical",
        message=(
            f"Determinism check {'PASSED' if matched else 'FAILED'} "
            f"(seed=42, two runs, outputs {'matched' if matched else 'DIFFERED'})."
        ),
        detail={
            "mode": "real",
            "seed": 42,
            "run_a_output": out_a,
            "run_b_output": out_b,
            "matched": matched,
            "run_a_wall_seconds": res_a["wall_seconds"],
            "run_b_wall_seconds": res_b["wall_seconds"],
            "guarantee_note": (
                "Empirical — verified by two independent generations. "
                "Determinism is hardware/quantisation-dependent; "
                "a mismatch here is a FAIL but does not imply a broken model, "
                "only that fixed-seed reproducibility is not guaranteed on this platform."
            ),
        },
    )


# ---------------------------------------------------------------------------
# Validation 2: structured-output / JSON-projection conformance (G11)
# ---------------------------------------------------------------------------

_JSON_SCHEMA_PROMPT = (
    "Respond with ONLY a JSON object, no other text. "
    "The object must have exactly these keys: "
    '"value" (an integer), "label" (a short string), "confidence" (a float between 0 and 1). '
    'Example: {"value": 42, "label": "example", "confidence": 0.95}'
)

_JSON_FIXTURE = '{"value": 42, "label": "fixture", "confidence": 0.95}'

_EXPECTED_KEYS = {"value", "label", "confidence"}
_EXPECTED_TYPES: dict[str, type | tuple[type, ...]] = {
    "value": int,
    "label": str,
    "confidence": float,
}


def _validate_json_shape(raw: str) -> tuple[bool, str, dict[str, Any]]:
    """
    Parse raw string as JSON and validate it matches the expected schema.
    Returns (ok, error_message, parsed_object).
    """
    # Strip markdown code fences if present
    stripped = raw.strip()
    if stripped.startswith("```"):
        lines = stripped.splitlines()
        stripped = "\n".join(
            line for line in lines if not line.startswith("```")
        ).strip()

    try:
        obj = json.loads(stripped)
    except json.JSONDecodeError as exc:
        return False, f"JSON parse error: {exc}", {}

    if not isinstance(obj, dict):
        return False, f"Expected a JSON object, got {type(obj).__name__}", {}

    missing = _EXPECTED_KEYS - obj.keys()
    if missing:
        return False, f"Missing keys: {sorted(missing)}", obj

    for key, expected_type in _EXPECTED_TYPES.items():
        val = obj[key]
        # JSON numbers: Python parses integers as int, floats as float.
        # Accept int where float is expected (e.g., 1 instead of 1.0).
        if key == "confidence" and isinstance(val, int):
            val = float(val)
            obj[key] = val
        if not isinstance(val, expected_type):
            return (
                False,
                f"Key '{key}': expected {expected_type.__name__}, got {type(val).__name__}",
                obj,
            )

    if not (0.0 <= obj["confidence"] <= 1.0):
        return (
            False,
            f"'confidence' must be in [0, 1], got {obj['confidence']}",
            obj,
        )

    return True, "", obj


@validation(
    "V-02-json-projection",
    "Structured-output / JSON-projection conformance (G11)",
)
def v02_json_projection(ctx: RunContext) -> ValidationResult:
    vid = "V-02-json-projection"

    if ctx.mock:
        ok, err, parsed = _validate_json_shape(_JSON_FIXTURE)
        return ValidationResult(
            id=vid,
            status=STATUS_MOCK_PASS if ok else STATUS_FAIL,
            guarantee_tag="Declared",
            message=(
                f"[MOCK] JSON-projection check against fixture — not a real model run. "
                f"Fixture {'parsed and validated OK' if ok else 'FAILED: ' + err}"
            ),
            detail={
                "mode": "mock",
                "fixture_input": _JSON_FIXTURE,
                "parse_ok": ok,
                "parse_error": err or None,
                "parsed": parsed,
                "expected_keys": sorted(_EXPECTED_KEYS),
                "guarantee_note": (
                    "Declared — fixture response, not model-derived. "
                    "Real runs would be tagged Empirical."
                ),
            },
        )

    # Real mode
    try:
        ctx.log.info("[V-02] Prompting for structured JSON output")
        res = _generate(ctx, _JSON_SCHEMA_PROMPT, seed=42, n_predict=128)
    except Exception as exc:
        ctx.log.exception("[V-02] Generation failed")
        return ValidationResult(
            id=vid,
            status=STATUS_FAIL,
            guarantee_tag=None,
            message=f"Generation error: {exc}",
            detail={"error": str(exc)},
        )

    raw = res["stdout"].strip()
    ok, err, parsed = _validate_json_shape(raw)

    return ValidationResult(
        id=vid,
        status=STATUS_PASS if ok else STATUS_FAIL,
        guarantee_tag="Empirical",
        message=(
            f"JSON-projection {'PASSED' if ok else 'FAILED'}: "
            f"{'schema matched' if ok else err}"
        ),
        detail={
            "mode": "real",
            "raw_output": raw,
            "parse_ok": ok,
            "parse_error": err or None,
            "parsed": parsed,
            "expected_keys": sorted(_EXPECTED_KEYS),
            "wall_seconds": res["wall_seconds"],
            "guarantee_note": (
                "Empirical — validated by parsing actual model output against schema. "
                "JSON conformance is prompt/model-dependent; "
                "failures here indicate the model did not follow the structured-output instruction."
            ),
        },
    )


# ---------------------------------------------------------------------------
# Validation 3: guarantee-tag honesty check (VR-5) — runs in BOTH modes
# ---------------------------------------------------------------------------

# Test fixtures: one compliant claim, one violating claim
_TAG_FIXTURES = [
    {
        "claim_id": "test-claim-empirical",
        "tag": "Empirical",
        "content": "The model produced output X in 7 out of 10 trials.",
        "expected": "ok",
    },
    {
        "claim_id": "test-claim-declared",
        "tag": "Declared",
        "content": "User asserts this is correct (unverified).",
        "expected": "ok",
    },
    {
        "claim_id": "test-claim-proven-FORBIDDEN",
        "tag": "Proven",
        "content": "The model has been formally proven correct.",
        "expected": "FAIL",
    },
    {
        "claim_id": "test-claim-exact-FORBIDDEN",
        "tag": "Exact",
        "content": "The model output is exactly bit-perfect.",
        "expected": "FAIL",
    },
]


@validation(
    "V-03-tag-honesty",
    "Guarantee-tag honesty gate (VR-5): model-derived claims must be Empirical or Declared",
    requires_model=False,  # pure logic — runs in every mode, never SKIPs
)
def v03_tag_honesty(ctx: RunContext) -> ValidationResult:
    vid = "V-03-tag-honesty"
    # This validation runs identically in both modes — it is pure logic.

    detected_violations: list[dict[str, str]] = []
    detected_compliant: list[dict[str, str]] = []

    for fixture in _TAG_FIXTURES:
        claim_id = fixture["claim_id"]
        tag = fixture["tag"]
        expected = fixture["expected"]
        try:
            assert_model_tag(tag, claim_id)
            # No exception ⇒ tag is allowed
        except ValueError as exc:
            if expected == "FAIL":
                # This is the correct outcome — we detected the violation
                detected_violations.append(
                    {
                        "claim_id": claim_id,
                        "tag": tag,
                        "violation_message": str(exc),
                        "outcome": "correctly-rejected",
                    }
                )
                ctx.log.info(
                    "[V-03] Correctly rejected forbidden tag '%s' on claim '%s'",
                    tag,
                    claim_id,
                )
            else:
                # Unexpected — the check is broken
                detected_violations.append(
                    {
                        "claim_id": claim_id,
                        "tag": tag,
                        "violation_message": str(exc),
                        "outcome": "unexpected-rejection",
                    }
                )
            continue

        if expected == "ok":
            detected_compliant.append(
                {
                    "claim_id": claim_id,
                    "tag": tag,
                    "outcome": "correctly-allowed",
                }
            )
            ctx.log.info(
                "[V-03] Correctly allowed tag '%s' on claim '%s'", tag, claim_id
            )
        else:
            # We expected a violation but didn't get one — honesty gate failed
            detected_violations.append(
                {
                    "claim_id": claim_id,
                    "tag": tag,
                    "violation_message": f"Tag '{tag}' should have been rejected but was allowed",
                    "outcome": "missed-violation",
                }
            )
            ctx.log.error(
                "[V-03] MISSED VIOLATION: tag '%s' on claim '%s' should have been rejected",
                tag,
                claim_id,
            )

    # The gate passes only if all expected violations were correctly rejected
    # and all compliant claims were correctly allowed.
    missed = [v for v in detected_violations if v["outcome"] == "missed-violation"]
    unexpected_rejections = [
        v for v in detected_violations if v["outcome"] == "unexpected-rejection"
    ]
    correctly_rejected = [
        v for v in detected_violations if v["outcome"] == "correctly-rejected"
    ]

    all_ok = len(missed) == 0 and len(unexpected_rejections) == 0

    return ValidationResult(
        id=vid,
        status=STATUS_PASS if all_ok else STATUS_FAIL,
        guarantee_tag=None,  # pure logic, no model claim involved
        message=(
            f"Tag-honesty gate {'PASSED' if all_ok else 'FAILED'}. "
            f"Correctly rejected {len(correctly_rejected)} forbidden tag(s), "
            f"correctly allowed {len(detected_compliant)} compliant tag(s). "
            + (f"MISSED {len(missed)} violation(s)! " if missed else "")
            + (
                f"UNEXPECTED rejections: {len(unexpected_rejections)}. "
                if unexpected_rejections
                else ""
            )
        ),
        detail={
            "mode": "mock" if ctx.mock else "real",
            "note": (
                "This validation runs identically in mock and real mode — "
                "it is pure logic (no model involved)."
            ),
            "fixtures_tested": len(_TAG_FIXTURES),
            "correctly_rejected": correctly_rejected,
            "correctly_allowed": detected_compliant,
            "missed_violations": missed,
            "unexpected_rejections": unexpected_rejections,
            "vr5_rule": (
                "Model-derived claims may only carry tags in {Empirical, Declared}. "
                "Proven or Exact requires a checked theorem — "
                "a model output can never be Proven or Exact by definition."
            ),
        },
    )


# ---------------------------------------------------------------------------
# Validation 4: latency/token report
# ---------------------------------------------------------------------------

_LATENCY_PROMPT = "What is 2 + 2? Answer with just the number."
_LATENCY_FIXTURE_WALL = 0.0  # clearly 0 in mock mode as a sentinel


@validation(
    "V-04-latency-tokens",
    "Latency and token-count report per generation",
)
def v04_latency_tokens(ctx: RunContext) -> ValidationResult:
    vid = "V-04-latency-tokens"

    if ctx.mock:
        return ValidationResult(
            id=vid,
            status=STATUS_MOCK_PASS,
            guarantee_tag="Declared",
            message=(
                "[MOCK] Latency/token report — synthetic numbers. "
                "wall_seconds=0.0 is a sentinel meaning 'not measured (mock mode)'. "
                "Prompt tokens and generated tokens are fixture values."
            ),
            detail={
                "mode": "mock",
                "wall_seconds": _LATENCY_FIXTURE_WALL,
                "token_counts": {
                    "prompt": 12,
                    "generated": 3,
                    "note": "Declared — synthetic fixture values, not measured",
                },
                "output": "4",
                "guarantee_note": (
                    "Declared — all numbers are synthetic fixtures. "
                    "Real runs would be tagged Empirical."
                ),
            },
        )

    # Real mode
    try:
        ctx.log.info("[V-04] Generating latency probe")
        res = _generate(ctx, _LATENCY_PROMPT, seed=1, n_predict=16)
    except Exception as exc:
        ctx.log.exception("[V-04] Generation failed")
        return ValidationResult(
            id=vid,
            status=STATUS_FAIL,
            guarantee_tag=None,
            message=f"Generation error: {exc}",
            detail={"error": str(exc)},
        )

    return ValidationResult(
        id=vid,
        status=STATUS_PASS,
        guarantee_tag="Empirical",
        message=(
            f"Latency probe completed in {res['wall_seconds']:.3f}s. "
            f"Prompt tokens: {res['token_counts']['prompt']}, "
            f"Generated tokens: {res['token_counts']['generated']}."
        ),
        detail={
            "mode": "real",
            "wall_seconds": res["wall_seconds"],
            "token_counts": res["token_counts"],
            "output": res["stdout"].strip(),
            "guarantee_note": (
                "Empirical — wall-clock time measured by harness (monotonic clock). "
                "Token counts parsed from llama.cpp output; "
                "may be None if llama.cpp changed its log format."
            ),
        },
    )


# ---------------------------------------------------------------------------
# Report writing
# ---------------------------------------------------------------------------


def _now_iso() -> str:
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def _write_json_report(
    path: Path,
    run_id: str,
    mode: str,
    results: list[ValidationResult],
    summary: dict[str, Any],
) -> None:
    doc = {
        "harness": "mycelium-llm-validation",
        "version": "0.1.0",
        "run_id": run_id,
        "mode": mode,
        "timestamp_utc": run_id,
        "honesty_posture": {
            "never_silent": True,
            "guarantee_lattice": LATTICE_ORDERED,
            "model_allowed_tags": sorted(MODEL_ALLOWED_TAGS),
            "vr5_rule": (
                "Model-derived claims are Empirical or Declared — NEVER Proven or Exact. "
                "A SKIP is never a PASS. mock-PASS is explicitly labelled and "
                "never treated as evidence of real model quality."
            ),
        },
        "summary": summary,
        "results": [r.to_dict() for r in results],
    }
    path.write_text(
        json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )


def _write_text_report(
    path: Path,
    run_id: str,
    mode: str,
    results: list[ValidationResult],
    summary: dict[str, Any],
) -> None:
    lines: list[str] = []
    lines.append("=" * 72)
    lines.append("  Mycelium LLM-validation harness  — human-readable projection (G11)")
    lines.append("=" * 72)
    lines.append(f"  Run ID  : {run_id}")
    lines.append(f"  Mode    : {mode}")
    lines.append("  Posture : NEVER-SILENT (G2) · VR-5 honesty · DUAL PROJECTION (G11)")
    lines.append("")
    lines.append("  Honesty posture:")
    lines.append("    · A missing tool/model ⇒ SKIP, never a false PASS")
    lines.append(
        "    · Model-derived claims: Empirical or Declared — NEVER Proven/Exact"
    )
    lines.append("    · mock-PASS = fixture run, not a real model quality signal")
    lines.append("")
    lines.append("-" * 72)
    lines.append("  Validation results")
    lines.append("-" * 72)

    for r in results:
        tag_str = f" [{r.guarantee_tag}]" if r.guarantee_tag else ""
        lines.append(f"  [{r.status:10s}]{tag_str}  {r.id}")
        lines.append(f"             {r.message}")
        lines.append("")

    lines.append("-" * 72)
    lines.append("  Summary")
    lines.append("-" * 72)
    for k, v in summary.items():
        lines.append(f"  {k:20s}: {v}")
    lines.append("")
    lines.append(
        "  NOTE: This is the human-readable projection. "
        "See companion .json for the machine projection."
    )
    lines.append("=" * 72)

    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


# ---------------------------------------------------------------------------
# Main run loop
# ---------------------------------------------------------------------------


# ---------------------------------------------------------------------------
# Binary discovery + PATH self-healing (shared by llama.cpp and the hf CLI)
# ---------------------------------------------------------------------------
#
# The #1 real-world failure on a phone (Termux) is NOT "tool missing" but
# "tool installed in a dir that isn't on PATH" — `pip --user` → ~/.local/bin,
# a hand-built llama.cpp → ~/llama.cpp/build/bin. `shutil.which` only sees
# PATH, so it reports a false "missing". We therefore (a) search the dirs
# installers/builds actually use, (b) auto-prepend a found dir to THIS
# process's PATH so the rest of the run works, and (c) optionally persist the
# fix to the user's shell rc (--fix-path). Every step is logged (G2).


def _is_termux() -> bool:
    """True on Termux/Android, where $PREFIX points into com.termux."""
    return "com.termux" in (os.environ.get("PREFIX", "") or "")


def _path_entries() -> list[str]:
    return [p for p in os.environ.get("PATH", "").split(os.pathsep) if p]


def _on_path(directory: Path) -> bool:
    rd = os.path.realpath(str(directory))
    return any(os.path.realpath(p) == rd for p in _path_entries())


def _prepend_process_path(directory: Path, log: logging.Logger | None = None) -> None:
    """Add `directory` to THIS process's PATH (in-memory) so child processes see it."""
    if not _on_path(directory):
        os.environ["PATH"] = str(directory) + os.pathsep + os.environ.get("PATH", "")
        if log is not None:
            log.debug("Prepended %s to PATH for this run.", directory)


def _shell_rc_files() -> list[Path]:
    """Likely shell rc files to persist a PATH fix into (shell-implied + common)."""
    home = Path.home()
    out: list[Path] = []
    if os.environ.get("SHELL", "").endswith("zsh"):
        out.append(home / ".zshrc")
    out.append(home / ".bashrc")  # Termux's default login shell
    out.append(home / ".profile")
    seen: set[str] = set()
    uniq: list[Path] = []
    for p in out:
        if str(p) not in seen:
            seen.add(str(p))
            uniq.append(p)
    return uniq


def persist_path_fix(directory: Path, log: logging.Logger, *, assume_yes: bool) -> bool:
    """Append `export PATH="dir:$PATH"` to a shell rc, idempotently. Returns True if applied.

    Opt-in (caller gates on --fix-path) + consent unless --yes. We only APPEND a
    clearly-marked line and skip if the dir is already referenced — never rewrite
    an existing rc.
    """
    line = f'export PATH="{directory}:$PATH"'
    marker = "# added by mycelium-llm-harness (--fix-path)"
    for rc in _shell_rc_files():
        try:
            if rc.is_file() and str(directory) in rc.read_text(encoding="utf-8"):
                log.info("PATH fix already present in %s — nothing to do.", rc)
                return True
        except OSError:
            pass
    target = _shell_rc_files()[0]
    if not _confirm(
        f"Persist PATH fix for {directory} by appending to {target}?",
        assume_yes=assume_yes,
        log=log,
    ):
        log.warning("Not persisting PATH. Add it yourself:\n    %s", line)
        return False
    try:
        with open(target, "a", encoding="utf-8") as f:
            f.write(f"\n{marker}\n{line}\n")
    except OSError as exc:
        log.error("Could not write %s: %s. Add it yourself:\n    %s", target, exc, line)
        return False
    log.info(
        "Wrote PATH fix to %s. Run `source %s` or restart your shell to pick it up.",
        target,
        target,
    )
    return True


def _note_off_path(
    found: Path,
    log: logging.Logger | None,
    *,
    fix_path: bool = False,
    assume_yes: bool = False,
) -> None:
    """A binary was found off-PATH: heal this run, advise, optionally persist."""
    d = found.parent
    if _on_path(d):
        return
    _prepend_process_path(d, log)
    if log is not None:
        log.warning(
            "%s is not on PATH. Using it for this run (PATH healed in-process). "
            'Make it permanent:\n    export PATH="%s:$PATH"   '
            "(or re-run with --fix-path to append it to your shell rc).",
            found,
            d,
        )
    if fix_path and log is not None:
        persist_path_fix(d, log, assume_yes=assume_yes)


def _search_dirs_for(names: tuple[str, ...], dirs: list[Path]) -> Path | None:
    for d in dirs:
        for name in names:
            cand = d / name
            if cand.is_file() and os.access(cand, os.X_OK):
                return cand
    return None


def _dedup_existing(dirs: list[Path]) -> list[Path]:
    seen: set[str] = set()
    out: list[Path] = []
    for d in dirs:
        k = str(d)
        if k not in seen:
            seen.add(k)
            out.append(d)
    return out


def _llama_search_dirs() -> list[Path]:
    """Dirs a hand-built or packaged llama.cpp commonly lands in (besides PATH)."""
    home = Path.home()
    dirs: list[Path] = []
    roots = [home, Path.cwd()]
    prefix = os.environ.get("PREFIX")
    if prefix:
        roots.append(Path(prefix))
        dirs.append(Path(prefix) / "bin")  # Termux `pkg install` target
    opt = os.environ.get("MYCELIUM_LLAMA_DIR")
    if opt:
        p = Path(opt).expanduser()
        dirs += [p, p / "bin", p / "build" / "bin"]
    for root in roots:
        for sub in ("llama.cpp", "llama-cpp", "llamacpp", "llama"):
            base = root / sub
            dirs += [base / "build" / "bin", base / "build", base / "bin", base]
    return _dedup_existing(dirs)


_LLAMA_BIN_NAMES = ("llama-cli", "llama", "main", "llama.cpp")


def _resolve_llama_cli(
    explicit: str | None,
    log: logging.Logger | None = None,
    *,
    fix_path: bool = False,
    assume_yes: bool = False,
) -> str | None:
    """Resolve llama-cli: explicit → PATH → known build/install dirs → shallow globs."""
    if explicit:
        if os.path.isfile(explicit) and os.access(explicit, os.X_OK):
            return explicit
        if log is not None:
            log.warning("--llama-cli is not an executable file: %s", explicit)
        return None
    for name in _LLAMA_BIN_NAMES:
        found = shutil.which(name)
        if found:
            return found
    # Off PATH: search common build/install dirs, then shallow globs.
    hit = _search_dirs_for(_LLAMA_BIN_NAMES, _llama_search_dirs())
    if hit is None:
        # Both the modern (`llama-cli`) and the packaged/renamed (`llama`) names:
        # the Termux `llama-cpp` package installs the CLI as plain `llama`, so a
        # glob that only matched `llama-cli` would miss a hand-built `llama` too.
        for root in (Path.home(), Path.cwd()):
            for pat in (
                "*/build/bin/llama-cli",
                "*/build/bin/llama",
                "llama*/build/bin/llama-cli",
                "llama*/build/bin/llama",
                "*/llama-cli",
                "*/llama",
            ):
                for cand in sorted(root.glob(pat)):
                    if cand.is_file() and os.access(cand, os.X_OK):
                        hit = cand
                        break
                if hit:
                    break
            if hit:
                break
    if hit is not None:
        _note_off_path(hit, log, fix_path=fix_path, assume_yes=assume_yes)
        return str(hit)
    return None


# ---------------------------------------------------------------------------
# Claude Code CLI discovery (same off-PATH problem: npm global bin / nvm)
# ---------------------------------------------------------------------------
#
# Not used by the validations — but the user hit the identical Termux trap
# (installed, not linked onto PATH), so --doctor / --fix-path cover it too.

_CLAUDE_BIN_NAMES = ("claude",)


def _npm_query(arg: str) -> str | None:
    """Best-effort `npm <arg>` one-liner (e.g. `config get prefix`, `root -g`)."""
    npm = shutil.which("npm")
    if not npm:
        return None
    try:
        res = subprocess.run(
            [npm, *arg.split()], capture_output=True, text=True, timeout=20
        )
    except Exception:  # noqa: BLE001 — best-effort
        return None
    val = res.stdout.strip()
    return val if (res.returncode == 0 and val) else None


def _npm_global_bin() -> Path | None:
    """Best-effort npm global bin dir via `npm config get prefix`."""
    prefix = _npm_query("config get prefix")
    if not prefix:
        return None
    cand = Path(prefix) / "bin"  # unix layout; Termux $PREFIX/bin (on PATH)
    return cand if cand.is_dir() else Path(prefix)


def _claude_package_cli() -> Path | None:
    """The Claude Code package entry (cli.js) if installed but possibly UNLINKED.

    The exact Termux failure: `npm i -g @anthropic-ai/claude-code` unpacks into
    <root>/@anthropic-ai/claude-code but the `claude` bin symlink never lands on
    PATH. We locate cli.js so --doctor can say "installed, not linked" + the fix.
    """
    roots: list[Path] = []
    r = _npm_query("root -g")
    if r:
        roots.append(Path(r))
    prefix = os.environ.get("PREFIX")
    if prefix:
        roots.append(Path(prefix) / "lib" / "node_modules")
    roots.append(Path.home() / ".npm-global" / "lib" / "node_modules")
    for root in _dedup_existing(roots):
        cli = root / "@anthropic-ai" / "claude-code" / "cli.js"
        if cli.is_file():
            return cli
    return None


def _claude_search_dirs() -> list[Path]:
    """Dirs the Claude Code CLI (npm/bun/pnpm/volta/nvm) commonly lands in."""
    home = Path.home()
    dirs: list[Path] = []
    g = _npm_global_bin()
    if g:
        dirs.append(g)
    dirs += [
        home / ".local" / "bin",
        home / ".npm-global" / "bin",
        home / ".bun" / "bin",
        home / ".volta" / "bin",
        home / ".claude" / "local",  # native/local installer
    ]
    pnpm = os.environ.get("PNPM_HOME")
    if pnpm:
        dirs.append(Path(pnpm))
    prefix = os.environ.get("PREFIX")
    if prefix:
        dirs.append(Path(prefix) / "bin")
    nvm = home / ".nvm" / "versions" / "node"
    if nvm.is_dir():
        dirs += sorted(nvm.glob("*/bin"))
    return _dedup_existing(dirs)


def _resolve_claude_cli(
    explicit: str | None = None,
    log: logging.Logger | None = None,
    *,
    fix_path: bool = False,
    assume_yes: bool = False,
) -> str | None:
    """Resolve the `claude` binary: explicit → PATH → npm/nvm/Termux dirs → globs."""
    if explicit:
        if os.path.isfile(explicit) and os.access(explicit, os.X_OK):
            return explicit
        if log is not None:
            log.warning("--claude-cli is not an executable file: %s", explicit)
        return None
    found = shutil.which("claude")
    if found:
        return found
    hit = _search_dirs_for(_CLAUDE_BIN_NAMES, _claude_search_dirs())
    if hit is None:
        for pat in (".nvm/versions/node/*/bin/claude", ".npm-global/bin/claude"):
            for cand in sorted(Path.home().glob(pat)):
                if cand.is_file() and os.access(cand, os.X_OK):
                    hit = cand
                    break
            if hit:
                break
    if hit is not None:
        _note_off_path(hit, log, fix_path=fix_path, assume_yes=assume_yes)
        return str(hit)
    return None


def _writable_path_bin_dir(
    log: logging.Logger, *, assume_yes: bool = False, fix_path: bool = False
) -> Path | None:
    """A bin dir we can symlink into that is (or will be made) on PATH.

    Prefers a dir already on PATH and writable (on Termux that's `$PREFIX/bin`),
    so a new link is immediately usable. Otherwise falls back to `~/.local/bin`,
    creating it and healing PATH (in-process now; persisted with --fix-path).
    """
    candidates: list[Path] = []
    prefix = os.environ.get("PREFIX")
    if prefix:
        candidates.append(Path(prefix) / "bin")  # Termux: already on PATH
    candidates.append(Path.home() / ".local" / "bin")
    for d in candidates:
        if d.is_dir() and _on_path(d) and os.access(d, os.W_OK):
            return d
    fallback = Path.home() / ".local" / "bin"
    try:
        fallback.mkdir(parents=True, exist_ok=True)
    except OSError as exc:
        log.error("Could not create %s for a launcher: %s", fallback, exc)
        return None
    if not _on_path(fallback):
        _prepend_process_path(fallback, log)
        if fix_path:
            persist_path_fix(fallback, log, assume_yes=assume_yes)
    return fallback


def link_claude_cli(
    pkg: Path, log: logging.Logger, *, assume_yes: bool = False, fix_path: bool = False
) -> str | None:
    """Link `claude` → the installed npm package's cli.js into a PATH bin dir.

    The Termux trap: the package unpacked but the `claude` bin symlink never
    landed on PATH. cli.js carries a `#!/usr/bin/env node` shebang, so a symlink
    to it is directly runnable. No curl|bash, no global npm mutation. Prompts
    unless --yes. Returns the new launcher path or None.
    """
    if not shutil.which("node"):
        log.warning(
            "Claude Code package is at %s but `node` is not on PATH — install "
            "Node first (Termux: pkg install nodejs), then re-run --doctor.",
            pkg,
        )
        return None
    bin_dir = _writable_path_bin_dir(log, assume_yes=assume_yes, fix_path=fix_path)
    if bin_dir is None:
        return None
    target = bin_dir / "claude"
    if not _confirm(
        f"Link `claude` → {pkg} in {bin_dir}?", assume_yes=assume_yes, log=log
    ):
        log.warning('Not linking. Do it yourself:\n    ln -s "%s" "%s"', pkg, target)
        return None
    try:
        # cli.js must be executable for the shebang launcher to work.
        mode = pkg.stat().st_mode
        pkg.chmod(mode | 0o111)
        if target.exists() or target.is_symlink():
            target.unlink()
        target.symlink_to(pkg)
    except OSError as exc:
        log.error(
            'Could not link claude: %s. Do it yourself:\n    ln -s "%s" "%s"',
            exc,
            pkg,
            target,
        )
        return None
    log.info("Linked claude: %s → %s", target, pkg)
    return str(target)


CLAUDE_NPM_PACKAGE = "@anthropic-ai/claude-code"


def install_claude_cli(
    log: logging.Logger, *, assume_yes: bool = False, fix_path: bool = False
) -> str | None:
    """Best-effort `npm install -g @anthropic-ai/claude-code`, then resolve/link.

    Honesty / supply-chain (CONTRIBUTING.md): installs the published npm package
    — never curl|bash. On Termux we first point npm's global prefix at `$PREFIX`
    so the `claude` link lands on the existing PATH. Prompts unless --yes.
    """
    npm = shutil.which("npm")
    if not npm:
        log.warning(
            "Cannot auto-install the Claude Code CLI: `npm` not found. Install "
            "Node/npm first (Termux: pkg install nodejs), then re-run --doctor."
        )
        return None
    if not _confirm(
        f"Claude Code CLI not found. Install '{CLAUDE_NPM_PACKAGE}' globally via npm now?",
        assume_yes=assume_yes,
        log=log,
    ):
        log.warning(
            "Skipping Claude Code install. Do it yourself:\n    npm install -g %s",
            CLAUDE_NPM_PACKAGE,
        )
        return None
    prefix = os.environ.get("PREFIX")
    if _is_termux() and prefix:
        # Make global bin links land on the existing PATH ($PREFIX/bin).
        try:
            subprocess.run(
                [npm, "config", "set", "prefix", prefix],
                capture_output=True,
                text=True,
                timeout=60,
            )
        except Exception as exc:  # noqa: BLE001 — best-effort, surfaced below
            log.warning("Could not set npm prefix to $PREFIX: %s", exc)
    cmd = [npm, "install", "-g", CLAUDE_NPM_PACKAGE]
    log.info("Installing Claude Code CLI: %s", " ".join(cmd))
    try:
        res = subprocess.run(cmd, capture_output=True, text=True, timeout=900)
    except Exception as exc:  # noqa: BLE001 — never-silent
        log.error("npm failed to start: %s", exc)
        return None
    if res.returncode != 0:
        log.error(
            "npm install exited %d: %s",
            res.returncode,
            (res.stderr or res.stdout or "").strip()[:400],
        )
        return None
    found = _resolve_claude_cli(None, log, fix_path=fix_path, assume_yes=assume_yes)
    if found:
        log.info("Claude Code CLI installed: %s", found)
        return found
    # Installed but the bin link missed PATH again — link it ourselves.
    pkg = _claude_package_cli()
    if pkg:
        return link_claude_cli(pkg, log, assume_yes=assume_yes, fix_path=fix_path)
    log.warning(
        "npm reported success but `claude` was not found on PATH or in the usual "
        "dirs. Restart your shell, or add npm's global bin to PATH, then re-run."
    )
    return None


# ---------------------------------------------------------------------------
# Hugging Face CLI: detect → (opt-in) install → auth check/prompt
# ---------------------------------------------------------------------------
#
# The hf CLI gives a robust, resumable, auth-aware download path (gated repos,
# CDN retries) that the stdlib urllib fallback can't match. The harness:
#   1. detects `hf` (new) or `huggingface-cli` (legacy) on PATH;
#   2. on --install-hf-cli, installs the published `huggingface_hub[cli]`
#      package — NOT via `curl … | bash` (CONTRIBUTING.md supply-chain rule:
#      no piping a remote script into a shell). The upstream one-liner is
#      printed as a reviewed manual fallback only;
#   3. checks auth (`hf auth whoami`) and, if unauthenticated, prompts to log
#      in. This is NON-FATAL: the default registry is public/ungated, so a
#      token-less run still downloads. Auth only matters for gated repos.
# Every step is explicit and logged (G2 never-silent); a missing/declined CLI
# simply falls back to the built-in stdlib downloader.

HF_INSTALL_SCRIPT_URL = "https://hf.co/cli/install.sh"
HF_PIP_PACKAGE = "huggingface_hub[cli]"


def _confirm(prompt: str, *, assume_yes: bool, log: logging.Logger) -> bool:
    """Yes/no gate. --yes auto-confirms; a non-TTY without --yes declines (safe default)."""
    if assume_yes:
        log.info("%s  [--yes: auto-confirmed]", prompt)
        return True
    if not sys.stdin.isatty():
        log.warning(
            "%s  — no TTY and --yes not set; declining (non-interactive).", prompt
        )
        return False
    try:
        ans = input(f"{prompt} [y/N] ").strip().lower()
    except EOFError:
        return False
    return ans in ("y", "yes")


# ---------------------------------------------------------------------------
# System package managers + vetted release downloads + checksum verification
# ---------------------------------------------------------------------------
#
# Supply-chain posture (CONTRIBUTING.md): we install runtime tools from the OS
# package manager (Termux `pkg` ≡ apt; Debian/Ubuntu `apt-get`; Fedora `dnf`;
# Arch `pacman`; openSUSE `zypper`; macOS `brew`) or from an official release
# artifact whose URL is pinned and whose SHA-256 we VERIFY before use — NEVER by
# building a fragile language-package from source (the hf-xet/Rust build that
# fails on Termux/aarch64) and NEVER by piping a remote script into a shell
# (`curl … | bash`). The OS package manager verifies its own repo signatures;
# for a direct release download we add an explicit checksum gate. Every step is
# logged (G2 never-silent); a failure is surfaced, never silently accepted.


def _detect_system_pkg() -> tuple[str, list[str]] | None:
    """Detect a system package manager. Returns (label, install_argv_prefix) or None.

    install_argv_prefix is the command up to (but excluding) the package name(s),
    already carrying a non-interactive 'assume yes'. Termux's `pkg` is a thin apt
    wrapper, so installs land in `$PREFIX/bin` (already on PATH). Off Termux we
    prepend `sudo` for managers that need root (apt/dnf/pacman/zypper) only when
    we are not already root and sudo exists; `brew` is never run under sudo.
    """
    if _is_termux() and shutil.which("pkg"):
        return ("pkg", ["pkg", "install", "-y"])
    is_root = getattr(os, "geteuid", lambda: 1)() == 0
    sudo = [] if is_root else (["sudo"] if shutil.which("sudo") else [])
    for mgr, args in (
        ("apt-get", ["apt-get", "install", "-y"]),
        ("dnf", ["dnf", "install", "-y"]),
        ("pacman", ["pacman", "-S", "--noconfirm"]),
        ("zypper", ["zypper", "install", "-y"]),
    ):
        if shutil.which(mgr):
            return (mgr, sudo + args)
    if shutil.which("brew"):  # macOS/Linuxbrew — never under sudo
        return ("brew", ["brew", "install"])
    return None


def install_system_package(
    name_by_mgr: dict[str, str],
    log: logging.Logger,
    *,
    assume_yes: bool = False,
    purpose: str = "",
) -> bool:
    """Install a package via the detected system manager. Returns True on success.

    name_by_mgr maps a manager label ('pkg', 'apt-get', 'dnf', 'pacman',
    'zypper', 'brew') to the package name there. A manager with no entry ⇒ we have
    no vetted package name for it and decline rather than guess. The OS package
    manager verifies its own signatures/checksums (no curl|bash, no unpinned fetch
    from us). Prompts unless --yes; never-silent on failure.
    """
    detected = _detect_system_pkg()
    if detected is None:
        log.warning(
            "No system package manager found (looked for pkg/apt-get/dnf/pacman/"
            "zypper/brew)%s.",
            f" — needed for {purpose}" if purpose else "",
        )
        return False
    label, prefix = detected
    pkg = name_by_mgr.get(label)
    if not pkg:
        log.warning(
            "No known %s package%s — install it another way (see the notes above).",
            label,
            f" for {purpose}" if purpose else "",
        )
        return False
    cmd = [*prefix, pkg]
    if not _confirm(
        f"Install '{pkg}' via {label}?" + (f"  ({purpose})" if purpose else ""),
        assume_yes=assume_yes,
        log=log,
    ):
        log.warning("Skipping %s install. Do it yourself:\n    %s", pkg, " ".join(cmd))
        return False
    log.info("Installing via %s: %s", label, " ".join(cmd))
    try:
        res = subprocess.run(cmd, text=True, timeout=1800)  # inherit stdio: progress
    except Exception as exc:  # noqa: BLE001 — never-silent
        log.error("%s failed to start: %s", label, exc)
        return False
    if res.returncode != 0:
        log.error("%s exited %d (see output above).", label, res.returncode)
        return False
    log.info("Installed '%s' via %s.", pkg, label)
    return True


def sha256_file(path: Path) -> str:
    """Streaming SHA-256 of a file (constant memory; large GGUF-safe)."""
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def verify_sha256(path: Path, expected: str, log: logging.Logger) -> bool:
    """True iff path's SHA-256 matches `expected`. Logs the outcome (never silent).

    A pinned checksum is the supply-chain gate for a direct release/model
    download (CONTRIBUTING.md): a mismatch is a loud, explicit failure — never a
    silent accept. Accepts a bare hex digest or a `sha256:<hex>` form.
    """
    want = expected.strip().lower()
    if want.startswith("sha256:"):
        want = want.split(":", 1)[1]
    actual = sha256_file(path)
    if actual == want:
        log.info("SHA-256 verified: %s", path.name)
        return True
    log.error(
        "SHA-256 MISMATCH for %s\n    expected: %s\n    actual:   %s",
        path,
        want,
        actual,
    )
    return False


# ---------------------------------------------------------------------------
# llama.cpp install (OS package manager → vetted from-source fallback)
# ---------------------------------------------------------------------------


def install_llama_cpp(
    log: logging.Logger, *, assume_yes: bool = False, fix_path: bool = False
) -> str | None:
    """Install llama.cpp from the OS package manager, then resolve `llama-cli`.

    Supply-chain (CONTRIBUTING.md): we install the OS-packaged, repo-signed build
    (Termux `pkg install llama-cpp`; some distros ship `llama.cpp`) rather than
    building a fragile source tree blind or piping a script. Where no package
    exists (most non-Termux aarch64), we DON'T guess — we print the vetted
    from-source / pinned-release steps and return None, so the run SKIPs honestly
    instead of faking an install (G2 never-silent).
    """
    ok = install_system_package(
        # Termux: `llama-cpp` (binaries → $PREFIX/bin, on PATH). A few distros
        # package it too; omit a manager here rather than guess a wrong name.
        {"pkg": "llama-cpp", "brew": "llama.cpp"},
        log,
        assume_yes=assume_yes,
        purpose="llama.cpp (llama-cli / llama-server)",
    )
    if ok:
        found = _resolve_llama_cli(None, log, fix_path=fix_path, assume_yes=assume_yes)
        if found:
            log.info("llama.cpp installed: %s", found)
            return found
        log.warning(
            "Package installed but no llama.cpp CLI (llama-cli / llama) was found "
            "on PATH — open a new shell and re-run, or pass --llama-cli PATH."
        )
        return None
    log.warning(
        "Could not install llama.cpp from a package manager. Build it from the "
        "official source (vet the repo, pin a tag), then re-run with --llama-cli:\n"
        "    git clone https://github.com/ggml-org/llama.cpp\n"
        "    cd llama.cpp && cmake -B build -DCMAKE_BUILD_TYPE=Release -DGGML_NATIVE=OFF\n"
        "    cmake --build build -j --target llama-cli\n"
        "  Or use a checksummed prebuilt release artifact (x86-64 / macOS / Windows):\n"
        "    https://github.com/ggml-org/llama.cpp/releases"
    )
    return None


def _candidate_bin_dirs() -> list[Path]:
    """Bin dirs where a pip/uv/pipx-installed `hf` lands but may NOT be on PATH.

    Termux (and `pip install --user` generally) drop console scripts in
    ~/.local/bin / $PREFIX/bin / pipx-and-uv venvs without adding them to PATH,
    so `which hf` misses a perfectly-installed CLI. We search them explicitly.
    """
    dirs: list[Path] = []
    try:
        import site
        import sysconfig

        scripts = sysconfig.get_path("scripts")
        if scripts:
            dirs.append(Path(scripts))  # this interpreter's scripts dir
        try:
            dirs.append(Path(sysconfig.get_path("scripts", scheme="posix_user")))
        except Exception:  # noqa: BLE001 — scheme may not exist
            pass
        dirs.append(Path(site.getuserbase()) / "bin")  # pip --user → ~/.local/bin
    except Exception:  # noqa: BLE001 — best-effort discovery
        pass
    home = Path.home()
    dirs.append(home / ".local" / "bin")  # pipx / uv tool symlinks
    xdg = Path(os.environ.get("XDG_DATA_HOME") or (home / ".local" / "share"))
    dirs.append(xdg / "pipx" / "venvs" / "huggingface_hub" / "bin")
    for tool in ("huggingface-hub", "huggingface_hub"):
        dirs.append(xdg / "uv" / "tools" / tool / "bin")
    pipx_home = os.environ.get("PIPX_HOME")
    if pipx_home:
        dirs.append(Path(pipx_home) / "venvs" / "huggingface_hub" / "bin")
    prefix = os.environ.get("PREFIX")  # Termux: /data/data/com.termux/files/usr
    if prefix:
        dirs.append(Path(prefix) / "bin")
    return _dedup_existing(dirs)


def _hf_module_cmd(log: logging.Logger | None = None) -> list[str] | None:
    """If huggingface_hub's CLI is importable by some Python, return a `-m` invocation.

    Last-resort fallback when no console script exists/links anywhere: the
    package itself still drives downloads via the legacy CLI module.
    """
    module = "huggingface_hub.commands.huggingface_cli"
    pythons = [sys.executable]
    for name in ("python3", "python"):
        w = shutil.which(name)
        if w and w not in pythons:
            pythons.append(w)
    for py in pythons:
        try:
            res = subprocess.run(
                [py, "-c", f"import {module}"], capture_output=True, timeout=20
            )
        except Exception:  # noqa: BLE001 — try the next interpreter
            continue
        if res.returncode == 0:
            return [py, "-m", module]
    return None


def _find_hf_off_path(
    log: logging.Logger | None = None,
    *,
    fix_path: bool = False,
    assume_yes: bool = False,
) -> tuple[list[str] | None, str | None]:
    """Search known install bin dirs for an `hf`/`huggingface-cli` not on PATH."""
    for d in _candidate_bin_dirs():
        for name, style in (("hf", "hf"), ("huggingface-cli", "legacy")):
            cand = d / name
            if cand.is_file() and os.access(cand, os.X_OK):
                _note_off_path(cand, log, fix_path=fix_path, assume_yes=assume_yes)
                return [str(cand)], style
    return None, None


def _resolve_hf_cli(
    explicit: str | None = None,
    log: logging.Logger | None = None,
    *,
    fix_path: bool = False,
    assume_yes: bool = False,
) -> tuple[list[str] | None, str | None]:
    """Return (cmd, style) for the Hugging Face CLI, or (None, None).

    cmd is an argv PREFIX — usually [path], or [python, -m, module] for the
    importable-package fallback. style is "hf" (new `hf auth …`) or "legacy"
    (`huggingface-cli …`). Searches PATH → known install dirs → `-m` fallback.
    """
    if explicit:
        if os.path.isfile(explicit) and os.access(explicit, os.X_OK):
            style = (
                "legacy" if "huggingface-cli" in os.path.basename(explicit) else "hf"
            )
            return [explicit], style
        if log is not None:
            log.warning("--hf-cli is not an executable file: %s", explicit)
        return None, None
    found = shutil.which("hf")
    if found:
        return [found], "hf"
    found = shutil.which("huggingface-cli")
    if found:
        return [found], "legacy"
    cmd, style = _find_hf_off_path(log, fix_path=fix_path, assume_yes=assume_yes)
    if cmd:
        return cmd, style
    mod = _hf_module_cmd(log)
    if mod:
        if log is not None:
            log.warning(
                "No `hf`/`huggingface-cli` executable found, but huggingface_hub is "
                "importable — driving it via `%s -m huggingface_hub…`. Install the "
                "[cli] extra (or fix PATH) to get the real `hf` command.",
                Path(mod[0]).name,
            )
        return mod, "legacy"
    return None, None


def install_hf_cli(
    log: logging.Logger, *, assume_yes: bool = False, fix_path: bool = False
) -> tuple[list[str] | None, str | None]:
    """Best-effort install of the hf CLI. Returns (cmd, style) or (None, None).

    OPT-IN ONLY (--install-hf-cli / --setup-hf) — never run by --doctor. The hf
    CLI is a Python package whose recent versions pull the native `hf-xet` build,
    which has no aarch64 wheel and FAILS to compile under Termux. The harness does
    not need it: the stdlib downloader fetches the public registry, and a gated
    repo works via `$HF_TOKEN`. We keep this path for users who explicitly want
    the CLI, but we warn first. Honesty / supply-chain (CONTRIBUTING.md): we do
    NOT `curl … | bash`; we install the published `huggingface_hub[cli]` package
    via uv / pipx / pip, in that order.
    """
    log.warning(
        "Installing the hf CLI builds the native `hf-xet` dependency, which often "
        "FAILS on Termux/aarch64 (no prebuilt wheel). If it does, that's fine — "
        "the built-in stdlib downloader still fetches models, and `$HF_TOKEN` "
        "unlocks gated repos without this CLI."
    )
    methods: list[tuple[list[str], str]] = []
    if shutil.which("uv"):
        methods.append((["uv", "tool", "install", HF_PIP_PACKAGE], "uv tool"))
    if shutil.which("pipx"):
        methods.append((["pipx", "install", HF_PIP_PACKAGE], "pipx"))
    # On Termux, plain `pip install` lands in $PREFIX/bin (already on PATH);
    # `--user` (~/.local/bin) is the off-PATH trap. Off-Termux, prefer --user.
    pip_cmd = [sys.executable, "-m", "pip", "install"]
    if not _is_termux():
        pip_cmd.append("--user")
    pip_cmd.append(HF_PIP_PACKAGE)
    methods.append((pip_cmd, "pip" if _is_termux() else "pip --user"))

    # On Termux, pip/pipx/uv come from `pkg` (≡ `apt`); surface that so a phone
    # with no installer yet has a clear first step.
    termux_hint = (
        "\n  On Termux (pkg ≡ apt): pkg install python  # provides pip\n"
        "    pkg install pipx   # or:  pkg install uv   (then the lines above)"
        if _is_termux()
        else ""
    )
    manual = (
        "Install it yourself, then re-run:\n"
        f"    uv tool install '{HF_PIP_PACKAGE}'\n"
        f"    pipx install '{HF_PIP_PACKAGE}'\n"
        f"    pip install --user '{HF_PIP_PACKAGE}'"
        f"{termux_hint}\n"
        f"  (or the upstream script, after reviewing it: "
        f"curl -LsSf {HF_INSTALL_SCRIPT_URL} | bash)"
    )

    if not _confirm(
        f"Hugging Face CLI not found. Install '{HF_PIP_PACKAGE}' now?",
        assume_yes=assume_yes,
        log=log,
    ):
        log.warning("Skipping hf-CLI install. %s", manual)
        return None, None

    for cmd, label in methods:
        log.info("Installing hf CLI via %s: %s", label, " ".join(cmd))
        try:
            res = subprocess.run(cmd, capture_output=True, text=True, timeout=600)
        except Exception as exc:  # noqa: BLE001 — never-silent: surface + try next
            log.warning("  %s failed to start: %s", label, exc)
            continue
        if res.returncode != 0:
            log.warning(
                "  %s exited %d: %s",
                label,
                res.returncode,
                (res.stderr or res.stdout or "").strip()[:400],
            )
            continue
        cli, style = _resolve_hf_cli(log=log, fix_path=fix_path, assume_yes=assume_yes)
        if cli:
            log.info("hf CLI installed via %s: %s", label, " ".join(cli))
            return cli, style
        log.warning(
            "  %s reported success but 'hf'/'huggingface-cli' was not found, even in "
            "the usual install dirs (%s). Restart your shell, or add the install bin "
            "dir to PATH, then re-run.",
            label,
            ", ".join(str(d) for d in _candidate_bin_dirs()),
        )
    log.error("Could not install the hf CLI automatically. %s", manual)
    return None, None


def hf_auth_status(
    cmd: list[str], style: str, log: logging.Logger
) -> tuple[bool, str | None]:
    """Return (authenticated, who). Best-effort; a query failure ⇒ (False, None)."""
    sub = ["auth", "whoami"] if style == "hf" else ["whoami"]
    try:
        res = subprocess.run([*cmd, *sub], capture_output=True, text=True, timeout=30)
    except Exception as exc:  # noqa: BLE001 — never-silent
        log.warning("Could not query Hugging Face auth status: %s", exc)
        return False, None
    out = (res.stdout + "\n" + res.stderr).strip()
    if res.returncode == 0 and out and "not logged in" not in out.lower():
        return True, out.splitlines()[0].strip()
    return False, None


def hf_login(
    cmd: list[str],
    style: str,
    log: logging.Logger,
    *,
    assume_yes: bool = False,
    token: str | None = None,
) -> bool:
    """Authenticate to Hugging Face. Token (flag/env) is non-interactive; else prompt.

    Returns True on a successful login. NON-FATAL on failure/decline — public
    models still download; only gated repos need auth.
    """
    token = (
        token or os.environ.get("HF_TOKEN") or os.environ.get("HUGGING_FACE_HUB_TOKEN")
    )
    base = [*cmd, "auth", "login"] if style == "hf" else [*cmd, "login"]

    if token:
        log.info("Logging in to Hugging Face with a token from --hf-token/$HF_TOKEN…")
        try:
            res = subprocess.run(
                [*base, "--token", token], capture_output=True, text=True, timeout=60
            )
        except Exception as exc:  # noqa: BLE001 — never-silent
            log.error("Hugging Face login failed to run: %s", exc)
            return False
        if res.returncode == 0:
            log.info("Hugging Face login OK (token).")
            return True
        log.error(
            "Hugging Face login failed: %s",
            (res.stderr or res.stdout or "").strip()[:400],
        )
        return False

    instructions = (
        "Not logged in to Hugging Face. Public models (the default registry) "
        "still download without auth — this only blocks GATED repos. To "
        "authenticate:\n"
        f"    {' '.join(base)}            # interactive\n"
        "    export HF_TOKEN=hf_xxxxxxxx        # or set a token, then re-run "
        "(or pass --hf-token)"
    )

    if sys.stdin.isatty() and _confirm(
        "Log in to Hugging Face now (interactive prompt)?",
        assume_yes=assume_yes,
        log=log,
    ):
        log.info("Launching `%s` (interactive)…", " ".join(base))
        try:
            res = subprocess.run(base)  # inherit stdio so the prompt works
        except Exception as exc:  # noqa: BLE001 — never-silent
            log.error("Hugging Face login failed: %s", exc)
            return False
        return res.returncode == 0

    log.warning(instructions)
    return False


def ensure_hf_ready(
    explicit_cli: str | None,
    log: logging.Logger,
    *,
    allow_install: bool = False,
    assume_yes: bool = False,
    want_auth: bool = True,
    token: str | None = None,
    fix_path: bool = False,
) -> tuple[list[str] | None, str | None]:
    """Detect → (opt-in) install → auth-check the hf CLI. Returns (cmd, style).

    (None, None) means no usable CLI — the caller falls back to the stdlib
    downloader. Auth is checked/prompted but never fatal (public registry).
    """
    cli, style = _resolve_hf_cli(
        explicit_cli, log, fix_path=fix_path, assume_yes=assume_yes
    )
    if not cli:
        log.info(
            "Hugging Face CLI not found%s.",
            f" (looked for: {explicit_cli})"
            if explicit_cli
            else " on PATH, known install dirs, or as an importable module",
        )
        if allow_install:
            cli, style = install_hf_cli(log, assume_yes=assume_yes, fix_path=fix_path)
        else:
            log.info(
                "Pass --install-hf-cli to install '%s' automatically, or install it "
                "yourself (uv tool / pipx / pip). Falling back to the built-in "
                "stdlib downloader for now.",
                HF_PIP_PACKAGE,
            )
        if not cli:
            return None, None
    else:
        log.info(
            "Hugging Face CLI: %s (%s)",
            " ".join(cli),
            "hf" if style == "hf" else "legacy huggingface-cli",
        )

    if want_auth and style is not None:
        authed, who = hf_auth_status(cli, style, log)
        if authed:
            log.info("Hugging Face: authenticated as %s.", who)
        else:
            log.info("Hugging Face: not authenticated.")
            hf_login(cli, style, log, assume_yes=assume_yes, token=token)

    return cli, style


def _download_via_hf_cli(
    cli: list[str],
    repo: str,
    filename: str,
    model_dir: Path,
    log: logging.Logger,
) -> Path | None:
    """Fetch a single GGUF file via `hf download`. Returns the verified path or None.

    Downloads only the named file (not the whole repo), into model_dir/filename,
    then verifies the GGUF magic before promoting — same honesty bar as the
    stdlib path: a missing/truncated/non-GGUF result is an explicit error, never
    a false "present".
    """
    dest = model_dir / filename
    cmd = [*cli, "download", repo, filename, "--local-dir", str(model_dir)]
    log.info("Downloading via hf CLI: %s", " ".join(cmd))
    try:
        res = subprocess.run(
            cmd, text=True, timeout=3600
        )  # inherit stdio: live progress
    except Exception as exc:  # noqa: BLE001 — never-silent
        log.error("hf download failed to run: %s", exc)
        return None
    if res.returncode != 0:
        log.error(
            "hf download exited %d (auth/network/repo error). "
            "Falling back to the built-in downloader.",
            res.returncode,
        )
        return None
    if is_valid_gguf(dest):
        log.info(
            "Model ready (hf CLI): %s (%s)", dest, _human_bytes(dest.stat().st_size)
        )
        return dest
    log.error(
        "hf download completed but %s failed GGUF validation "
        "(missing/truncated/not a GGUF). Not promoting.",
        dest,
    )
    return None


# ---------------------------------------------------------------------------
# Model acquisition (idempotent, stdlib-only, Termux-friendly)
# ---------------------------------------------------------------------------
#
# A small registry of GGUF models sized for THIS harness's use case — seeded
# generation, structured/JSON output (G11), and instruction-following for the
# guarantee-tag honesty gate. The defaults are mobile-sized and UNGATED
# (Apache-2.0 Qwen2.5 family), so a phone can `--ensure-model` and walk away;
# bigger desktop tiers are registered for later runs on a real GPU.
#
# Honesty (G2/VR-5): the URLs/filenames below are BEST-EFFORT and may change
# upstream — a download is verified by the GGUF magic header + a clean,
# complete transfer, never assumed. A failed/partial/HTML-error fetch is an
# explicit error (the .part file is kept for resume), never a false "present".
# Override any entry with --model-url, or bypass the registry with --model PATH.

HF_RESOLVE = "https://huggingface.co/{repo}/resolve/main/{filename}"

# Each entry: repo + filename (→ HF resolve URL), tier, rough size, license,
# and the use-case note that justifies it for this harness.
# tier: "mobile" (phone / Termux / CPU) | "desktop" (discrete GPU)
MODELS: dict[str, dict[str, Any]] = {
    "qwen2.5-0.5b-instruct": {
        "repo": "Qwen/Qwen2.5-0.5B-Instruct-GGUF",
        "filename": "qwen2.5-0.5b-instruct-q4_k_m.gguf",
        "tier": "mobile",
        "approx_gb": 0.4,
        "license": "Apache-2.0",
        "use_case": "smallest/fastest smoke-test tier; weakest reasoning",
    },
    "qwen2.5-1.5b-instruct": {
        "repo": "Qwen/Qwen2.5-1.5B-Instruct-GGUF",
        "filename": "qwen2.5-1.5b-instruct-q4_k_m.gguf",
        "tier": "mobile",
        "approx_gb": 1.0,
        "license": "Apache-2.0",
        "use_case": "general instruct + JSON/structured output on a phone",
    },
    "qwen2.5-coder-0.5b": {
        "repo": "Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF",
        "filename": "qwen2.5-coder-0.5b-instruct-q4_k_m.gguf",
        "tier": "mobile",
        "approx_gb": 0.4,
        "license": "Apache-2.0",
        "use_case": (
            "fastest code tier — ~2-3x quicker decode than the 1.5B on a phone CPU; "
            "preferred for unattended KC-2 sweeps where generation time dominates"
        ),
    },
    "qwen2.5-coder-1.5b": {
        "repo": "Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF",
        "filename": "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf",
        "tier": "mobile",
        "approx_gb": 1.0,
        "license": "Apache-2.0",
        "use_case": (
            "code + structured output; stronger reasoning than the 0.5B but slower "
            "(Mycelium surface generation + JSON projection conformance)"
        ),
    },
    "qwen2.5-3b-instruct": {
        "repo": "Qwen/Qwen2.5-3B-Instruct-GGUF",
        "filename": "qwen2.5-3b-instruct-q4_k_m.gguf",
        "tier": "mobile",
        "approx_gb": 2.0,
        "license": "Qwen-Research (non-commercial — check before use)",
        "use_case": "stronger reasoning, still phone-feasible (slow)",
    },
    "qwen2.5-coder-3b": {
        "repo": "Qwen/Qwen2.5-Coder-3B-Instruct-GGUF",
        "filename": "qwen2.5-coder-3b-instruct-q4_k_m.gguf",
        "tier": "mobile",
        "approx_gb": 2.0,
        "license": "Qwen-Research (non-commercial — check before use)",
        "use_case": "stronger code/structured output; phone-feasible (slow)",
    },
    "qwen2.5-coder-7b": {
        "repo": "Qwen/Qwen2.5-Coder-7B-Instruct-GGUF",
        "filename": "qwen2.5-coder-7b-instruct-q4_k_m.gguf",
        "tier": "desktop",
        "approx_gb": 4.7,
        "license": "Apache-2.0",
        "use_case": "strong code model; fits 8GB+ VRAM (RTX 3090Ti / 5080)",
    },
    "qwen2.5-coder-14b": {
        "repo": "Qwen/Qwen2.5-Coder-14B-Instruct-GGUF",
        "filename": "qwen2.5-coder-14b-instruct-q4_k_m.gguf",
        "tier": "desktop",
        "approx_gb": 9.0,
        "license": "Apache-2.0",
        "use_case": "high-quality code/structured; fits 16GB (5080) / 24GB (3090Ti)",
    },
    "qwen2.5-coder-32b": {
        "repo": "Qwen/Qwen2.5-Coder-32B-Instruct-GGUF",
        "filename": "qwen2.5-coder-32b-instruct-q4_k_m.gguf",
        "tier": "desktop",
        "approx_gb": 19.8,
        "license": "Apache-2.0",
        "use_case": (
            "best open code model; ~24GB at q4_k_m (3090Ti tight / 5080 needs "
            "offload). FLAG: some repos split this into parts — verify on HF or "
            "fetch with huggingface-cli, then pass --model PATH."
        ),
    },
}

DEFAULT_MODEL_ID = "qwen2.5-coder-1.5b"

GGUF_MAGIC = b"GGUF"
# A real q4_k_m model is hundreds of MB; this floor catches an HTML 404/gated
# page saved under a .gguf name (never let that masquerade as a model).
_MIN_GGUF_BYTES = 1_000_000


def _human_bytes(n: int | None) -> str:
    if n is None:
        return "?"
    f = float(n)
    for unit in ("B", "KB", "MB", "GB", "TB"):
        if f < 1024 or unit == "TB":
            return f"{f:.1f}{unit}" if unit != "B" else f"{int(f)}B"
        f /= 1024
    return f"{f:.1f}TB"


def _model_url(spec: dict[str, Any]) -> str:
    if spec.get("url"):
        return str(spec["url"])
    return HF_RESOLVE.format(repo=spec["repo"], filename=spec["filename"])


def default_model_dir() -> Path:
    """Resolve the model cache dir (outside the repo, so models are never committed)."""
    env = os.environ.get("MYCELIUM_LLM_MODEL_DIR")
    if env:
        return Path(env).expanduser()
    xdg = os.environ.get("XDG_CACHE_HOME")
    base = Path(xdg).expanduser() if xdg else Path.home() / ".cache"
    return base / "mycelium-llm-harness" / "models"


def is_valid_gguf(path: Path) -> bool:
    """True iff path looks like a real, complete-enough GGUF (magic + size floor)."""
    try:
        if not path.is_file() or path.stat().st_size < _MIN_GGUF_BYTES:
            return False
        with open(path, "rb") as f:
            return f.read(4) == GGUF_MAGIC
    except OSError:
        return False


def expected_model_path(model_id: str, model_dir: Path) -> Path:
    """Where a given model id would live on disk (registry filename, else <id>.gguf)."""
    spec = MODELS.get(model_id)
    filename = str(spec["filename"]) if spec else f"{model_id}.gguf"
    return model_dir / filename


def find_cached_model(model_dir: Path) -> str | None:
    """Return a valid GGUF already in model_dir — the default first, else any.

    Lets real mode reuse an already-downloaded model with no --ensure-model and
    no hf round-trip (the walk-away property, post-download).
    """
    default = expected_model_path(DEFAULT_MODEL_ID, model_dir)
    if is_valid_gguf(default):
        return str(default)
    try:
        for p in sorted(model_dir.glob("*.gguf")):
            if is_valid_gguf(p):
                return str(p)
    except OSError:
        pass
    return None


def list_models_text() -> str:
    lines = [
        "Registered GGUF models (--model-id ID). Default: " + DEFAULT_MODEL_ID,
        "Cache dir: "
        + str(default_model_dir())
        + "  (override: --model-dir / $MYCELIUM_LLM_MODEL_DIR)",
        "",
    ]
    for tier in ("mobile", "desktop"):
        lines.append(f"  [{tier}]")
        for mid, spec in MODELS.items():
            if spec["tier"] != tier:
                continue
            star = " *" if mid == DEFAULT_MODEL_ID else "  "
            lines.append(f"  {star}{mid:<24} ~{spec['approx_gb']}GB  {spec['license']}")
            lines.append(f"        {spec['use_case']}")
        lines.append("")
    lines.append(
        "Honesty: URLs are best-effort; a download is verified by the GGUF magic "
        "header. Override with --model-url, or use a self-verified file via --model PATH."
    )
    return "\n".join(lines)


def _download_with_resume(
    url: str, dest_part: Path, log: logging.Logger
) -> tuple[int, int | None]:
    """Stream url → dest_part with HTTP Range resume.

    Returns (bytes_on_disk, expected_total). expected_total is the full file size
    when the server advertises it (Content-Length / Content-Range), else None.
    The caller verifies bytes_on_disk == expected_total before accepting the file,
    so a silently-truncated transfer is never promoted. Raises on a hard transfer
    failure (the .part file is left in place to resume).
    """
    import urllib.error
    import urllib.request

    headers = {"User-Agent": "mycelium-llm-harness/0 (+python-stdlib-urllib)"}
    # Gated Hugging Face repos: send a bearer token when one is configured, so the
    # stdlib downloader can reach gated models WITHOUT the hf CLI (the whole point
    # of dropping the fragile Python-package install). Scoped to HF hosts only so
    # the token never leaks to a --model-url pointing elsewhere.
    token = os.environ.get("HF_TOKEN") or os.environ.get("HUGGING_FACE_HUB_TOKEN")
    if token and "huggingface.co" in url:
        headers["Authorization"] = f"Bearer {token}"
    existing = dest_part.stat().st_size if dest_part.exists() else 0
    if existing:
        headers["Range"] = f"bytes={existing}-"
        log.info("Resuming at %s", _human_bytes(existing))

    req = urllib.request.Request(url, headers=headers)
    try:
        resp = urllib.request.urlopen(req, timeout=60)  # follows redirects (HF CDN)
    except urllib.error.HTTPError as exc:
        if exc.code == 416 and existing:
            # Range not satisfiable ⇒ the .part is already the full file.
            return existing, existing
        raise

    status = getattr(resp, "status", resp.getcode())
    # Determine total expected size for progress (best-effort).
    total: int | None = None
    crange = resp.headers.get("Content-Range")
    clen = resp.headers.get("Content-Length")
    if crange and "/" in crange:
        try:
            total = int(crange.rsplit("/", 1)[1])
        except ValueError:
            total = None
    elif clen is not None:
        try:
            total = (existing + int(clen)) if status == 206 else int(clen)
        except ValueError:
            total = None

    # If the server ignored our Range (status 200), restart from scratch.
    append = existing > 0 and status == 206
    mode = "ab" if append else "wb"
    downloaded = existing if append else 0

    last_log = time.monotonic()
    with resp, open(dest_part, mode) as f:
        while True:
            chunk = resp.read(256 * 1024)
            if not chunk:
                break
            f.write(chunk)
            downloaded += len(chunk)
            now = time.monotonic()
            if now - last_log >= 5.0:
                pct = f" ({downloaded * 100 // total}%)" if total else ""
                log.info(
                    "  …%s%s%s",
                    _human_bytes(downloaded),
                    pct,
                    f" of {_human_bytes(total)}" if total else "",
                )
                last_log = now
    return downloaded, total


def ensure_model(
    model_id: str,
    model_dir: Path,
    log: logging.Logger,
    *,
    allow_download: bool = True,
    url_override: str | None = None,
    hf_cmd: list[str] | None = None,
    prefer_hf: bool = True,
    sha256: str | None = None,
) -> Path | None:
    """Idempotent: return a local GGUF path, downloading ONLY if absent/invalid.

    Returns the Path on success, or None on failure (explicit + logged — never a
    false path). Re-running is a cheap presence check (the walk-away property).

    Integrity: when a SHA-256 is known (the `sha256` arg or a pinned `spec`
    value), a freshly-downloaded file MUST match it before promotion — a mismatch
    is rejected loudly (supply-chain gate, CONTRIBUTING.md). With no pinned
    checksum we fall back to the GGUF magic + complete-transfer check and say so.
    """
    spec = MODELS.get(model_id)
    if spec is None and url_override is None:
        log.error(
            "Unknown model id '%s'. Run --list-models, or pass --model-url "
            "with any --model-id name to fetch an arbitrary GGUF.",
            model_id,
        )
        return None
    if spec is None:
        spec = {"filename": f"{model_id}.gguf", "url": url_override}

    expected_sha = sha256 or spec.get("sha256")

    model_dir.mkdir(parents=True, exist_ok=True)
    dest = model_dir / spec["filename"]

    if is_valid_gguf(dest):
        log.info(
            "Model already present (idempotent skip): %s (%s)",
            dest,
            _human_bytes(dest.stat().st_size),
        )
        return dest
    if dest.exists():
        log.warning("Existing file failed GGUF validation; will re-fetch: %s", dest)
    if not allow_download:
        log.warning("Model absent and --no-download set: %s. SKIP.", dest)
        return None

    log.info(
        "Fetching model '%s' (~%sGB, %s)",
        model_id,
        spec.get("approx_gb", "?"),
        spec.get("license", "?"),
    )
    log.info("  into %s", dest)

    # Preferred path: the hf CLI (robust, resumable, auth-aware). Only when we
    # have a registry repo and no explicit URL override (the CLI is repo-based).
    if prefer_hf and hf_cmd and spec.get("repo") and url_override is None:
        ensured = _download_via_hf_cli(
            hf_cmd, str(spec["repo"]), str(spec["filename"]), model_dir, log
        )
        if ensured:
            if expected_sha and not verify_sha256(ensured, expected_sha, log):
                log.error(
                    "Pinned SHA-256 did not match the hf-CLI download — not "
                    "promoting %s. (Check --model-sha256 / the pinned value.)",
                    ensured,
                )
                return None
            return ensured
        log.warning(
            "hf CLI download did not succeed — falling back to the built-in "
            "stdlib downloader."
        )

    url = url_override or _model_url(spec)
    log.info("  from %s", url)

    dest_part = dest.with_suffix(dest.suffix + ".part")
    # Auto-resume across transient failures: a flaky/slow link (a phone on cellular, or a
    # 468MB model over a stalling connection) drops mid-stream, but _download_with_resume
    # appends to the .part and re-requests with HTTP Range, so each retry picks up where the
    # last left off. One invocation now survives several drops instead of needing a manual
    # re-run each time (the .part is still kept if we ultimately give up).
    max_attempts = 6
    downloaded, total = 0, None
    for attempt in range(1, max_attempts + 1):
        try:
            downloaded, total = _download_with_resume(url, dest_part, log)
        except Exception as exc:  # noqa: BLE001 — never-silent: surface, keep .part, retry
            if attempt >= max_attempts:
                log.error(
                    "Download failed after %d attempts: %s. Re-run to resume (kept %s).",
                    max_attempts, exc, dest_part,
                )
                return None
            wait = min(30, 2**attempt)
            log.warning(
                "Download error (attempt %d/%d): %s — retrying in %ds (resumes from %s).",
                attempt, max_attempts, exc, wait, dest_part,
            )
            time.sleep(wait)
            continue
        if total is None or downloaded == total:
            break  # complete (or size unknown — the GGUF/size checks below decide)
        if attempt >= max_attempts:
            log.error(
                "Incomplete after %d attempts: got %s of %s. Re-run to resume: %s",
                max_attempts, _human_bytes(downloaded), _human_bytes(total), dest_part,
            )
            return None
        wait = min(30, 2**attempt)
        log.warning(
            "Incomplete (%s of %s) — resuming (attempt %d/%d) in %ds.",
            _human_bytes(downloaded), _human_bytes(total), attempt, max_attempts, wait,
        )
        time.sleep(wait)

    # Completeness: if the server advertised a size, the transfer must match it.
    # A truncated body that still starts with the GGUF magic must NOT be promoted.
    if total is not None and downloaded != total:
        log.error(
            "Incomplete download: got %s of %s. Not promoting (re-run to resume): %s",
            _human_bytes(downloaded),
            _human_bytes(total),
            dest_part,
        )
        return None

    if not is_valid_gguf(dest_part):
        log.error(
            "Downloaded file failed GGUF validation — not a GGUF (truncated, or an "
            "HTML 404/gated page): %s. NOT promoting; removing it. Check the "
            "URL/filename or pass --model-url.",
            dest_part,
        )
        try:
            dest_part.unlink()
        except OSError:
            pass
        return None

    # Supply-chain gate: a pinned checksum must match before promotion. A
    # mismatch keeps the .part for inspection rather than promoting bad bytes.
    if expected_sha:
        if not verify_sha256(dest_part, expected_sha, log):
            log.error(
                "Pinned SHA-256 did not match — NOT promoting %s. "
                "The .part is kept for inspection; verify the URL/checksum.",
                dest_part,
            )
            return None
    else:
        log.info(
            "No pinned SHA-256 for this model — integrity rests on the GGUF magic "
            "+ complete transfer. Pass --model-sha256 to enforce one."
        )

    os.replace(dest_part, dest)
    log.info("Model ready: %s (%s)", dest, _human_bytes(dest.stat().st_size))
    return dest


def run_harness(args: argparse.Namespace) -> int:
    """Execute all validations. Returns exit code (0 = no FAIL)."""
    run_id = _now_iso()
    reports_dir = Path(__file__).parent / "reports"
    reports_dir.mkdir(parents=True, exist_ok=True)

    # Set up logging: both file and stderr
    log_path = reports_dir / f"{run_id}.log"
    log = logging.getLogger("mycelium.llm-harness")
    log.setLevel(logging.DEBUG)
    fmt = logging.Formatter(
        fmt="%(asctime)s  %(levelname)-8s  %(message)s",
        datefmt="%Y-%m-%dT%H:%M:%SZ",
    )
    # File handler (full trace)
    fh = logging.FileHandler(log_path, encoding="utf-8")
    fh.setLevel(logging.DEBUG)
    fh.setFormatter(fmt)
    log.addHandler(fh)
    # Stderr handler (INFO and above)
    sh = logging.StreamHandler(sys.stderr)
    sh.setLevel(logging.INFO)
    sh.setFormatter(fmt)
    log.addHandler(sh)

    log.info("=== Mycelium LLM-validation harness ===")
    log.info("Run ID: %s", run_id)

    # Determine mode
    mock_mode = bool(getattr(args, "mock", False))
    server_url: str | None = getattr(args, "server", None)
    model_path: str | None = getattr(args, "model", None)
    llama_cli_arg: str | None = getattr(args, "llama_cli", None)

    if mock_mode:
        log.info("Mode: MOCK (dry-run) — no model, fixture responses, CI-safe")
        llama_cli = None
    elif server_url:
        log.info("Mode: REAL (server) — %s", server_url)
        llama_cli = None
        if model_path:
            log.warning(
                "--model is ignored in server mode (model is loaded server-side)"
            )
    else:
        fix_path = bool(getattr(args, "fix_path", False))
        assume_yes = bool(getattr(args, "assume_yes", False))
        llama_cli = _resolve_llama_cli(
            llama_cli_arg, log, fix_path=fix_path, assume_yes=assume_yes
        )
        if llama_cli:
            log.info(
                "Mode: REAL (llama.cpp CLI) — binary: %s (alias: %s)",
                llama_cli,
                os.path.basename(llama_cli),
            )
        else:
            log.warning(
                "llama.cpp CLI (llama-cli / llama) not found%s (searched PATH, "
                "~/llama.cpp/build/bin, $PREFIX/bin, $MYCELIUM_LLAMA_DIR, and shallow "
                "globs). SKIPPING all model-dependent validations (skip-gracefully, "
                "G2). To run real mode: provide --llama-cli PATH or build llama.cpp. "
                "To suppress this and run CI-safe: use --mock. Run --doctor to diagnose.",
                f" (looked for: {llama_cli_arg})" if llama_cli_arg else "",
            )

        # Idempotent model acquisition. Triggered by --ensure-model or by
        # naming a --model-id; downloads ONLY if absent/invalid. Runs even when
        # llama-cli is missing, so a phone can prefetch the model first and
        # build llama.cpp later (the cached model is reused — walk-away property).
        want_ensure = bool(getattr(args, "ensure_model", False)) or bool(
            getattr(args, "model_id", None)
        )
        if model_path is None and want_ensure:
            model_id = getattr(args, "model_id", None) or DEFAULT_MODEL_ID
            md_arg = getattr(args, "model_dir", None)
            model_dir = Path(md_arg).expanduser() if md_arg else default_model_dir()
            url_override = getattr(args, "model_url", None)

            # If the model is already on disk, there is nothing to download — so
            # do NOT spin up / install the hf CLI just to no-op. (The user's phone
            # has the model already; this avoids nagging for a tool it won't use.)
            already_present = url_override is None and is_valid_gguf(
                expected_model_path(model_id, model_dir)
            )

            prefer_hf = not bool(getattr(args, "no_hf_cli", False))
            hf_cmd = None
            if (
                prefer_hf
                and not bool(getattr(args, "no_download", False))
                and not already_present
            ):
                # Hugging Face CLI: detect → (opt-in) install → auth check/prompt.
                # Falls back to the built-in stdlib downloader when unavailable.
                hf_cmd, _ = ensure_hf_ready(
                    getattr(args, "hf_cli", None),
                    log,
                    allow_install=bool(getattr(args, "install_hf_cli", False)),
                    assume_yes=assume_yes,
                    want_auth=True,
                    token=getattr(args, "hf_token", None),
                    fix_path=fix_path,
                )
            elif already_present:
                log.info(
                    "Model already present — skipping hf-CLI setup (no download needed)."
                )

            log.info("Ensuring model '%s' in %s (idempotent)…", model_id, model_dir)
            ensured = ensure_model(
                model_id,
                model_dir,
                log,
                allow_download=not bool(getattr(args, "no_download", False)),
                url_override=url_override,
                hf_cmd=hf_cmd,
                prefer_hf=prefer_hf,
                sha256=getattr(args, "model_sha256", None),
            )
            if ensured:
                model_path = str(ensured)
            else:
                log.warning(
                    "Model could not be ensured — model-dependent validations "
                    "will SKIP (skip-gracefully, never a false pass)."
                )
        elif model_path is None and not want_ensure:
            # No --model / --ensure-model, but a model may already be cached from a
            # previous run — reuse it so real mode "just works" once llama.cpp exists.
            md_arg = getattr(args, "model_dir", None)
            model_dir = Path(md_arg).expanduser() if md_arg else default_model_dir()
            cached = find_cached_model(model_dir)
            if cached:
                model_path = cached
                log.info("Using cached model (no --ensure-model needed): %s", cached)

        if not llama_cli and model_path and not server_url:
            log.warning(
                "Model path provided (%s) but no llama-cli found — "
                "model-dependent validations will SKIP.",
                model_path,
            )

    # Validate model path if in real (llama-cli) mode
    if not mock_mode and llama_cli and not server_url:
        if not model_path:
            log.warning(
                "No --model provided. "
                "Model-dependent validations will SKIP (skip-gracefully)."
            )
            llama_cli = None  # treat as unavailable
        elif not os.path.isfile(model_path):
            log.warning(
                "Model file not found: %s. Model-dependent validations will SKIP.",
                model_path,
            )
            llama_cli = None

    # Build context — if we have neither server nor cli, run as effective-skip-mode
    effective_mock = mock_mode or (not server_url and not llama_cli)
    if effective_mock and not mock_mode:
        log.info(
            "Falling back to SKIP mode for model-dependent validations "
            "(no model/binary available — use --mock for fixture mode)."
        )

    # Resource sizing (only meaningful for the llama-cli path — server/mock skip it).
    # Context: explicit --ctx-size wins; else auto-size from RAM (+ optional swap).
    # GPU: --cpu-only forces 0; else explicit --n-gpu-layers wins; else auto from VRAM.
    cli_path = bool(llama_cli) and not server_url
    ctx_size_arg = getattr(args, "ctx_size", None)
    use_swap = bool(getattr(args, "use_swap", False))
    if ctx_size_arg is not None:
        ctx_size = int(ctx_size_arg)
    elif cli_path:
        ctx_size = auto_ctx_size(
            _HARNESS_DESIRED_CTX,
            model_path,
            detect_memory(),
            log,
            swap_fraction=0.5 if use_swap else 0.0,
        )
    else:
        ctx_size = _HARNESS_DESIRED_CTX

    ngl_arg = getattr(args, "n_gpu_layers", None)
    if bool(getattr(args, "cpu_only", False)):
        n_gpu_layers = 0
    elif ngl_arg is not None:
        n_gpu_layers = int(ngl_arg)
    elif cli_path:
        n_gpu_layers = auto_gpu_layers(detect_gpu(), model_path, log)
    else:
        n_gpu_layers = 0

    ctx = RunContext(
        mock=mock_mode,
        llama_cli=llama_cli,
        model_path=model_path,
        server_url=server_url,
        reports_dir=reports_dir,
        run_id=run_id,
        log=log,
        ctx_size=ctx_size,
        n_gpu_layers=n_gpu_layers,
        llama_timeout=int(getattr(args, "timeout", 300) or 300),
        extra_llama_args=list(getattr(args, "llama_arg", None) or []),
    )

    results: list[ValidationResult] = []

    # Run all registered validations
    for vid, description, requires_model, fn in _REGISTRY:
        log.info("--- Running %s: %s ---", vid, description)
        try:
            # If we have no model and are not in explicit mock mode,
            # model-dependent validations become SKIPs. Pure-logic checks
            # (requires_model=False, e.g. V-03) always run.
            if not mock_mode and not server_url and not llama_cli and requires_model:
                result = ValidationResult(
                    id=vid,
                    status=STATUS_SKIP,
                    guarantee_tag=None,
                    message=(
                        "SKIP — no model or llama-cli available. "
                        "This is not a PASS. "
                        "Provide --model + (--llama-cli or --server) for real mode, "
                        "or use --mock for fixture mode."
                    ),
                    detail={"skip_reason": "no_model_available"},
                )
            else:
                result = fn(ctx)
        except Exception as exc:
            log.exception(
                "Unhandled exception in %s — converting to FAIL (RFC-0013 I1)", vid
            )
            result = ValidationResult(
                id=vid,
                status=STATUS_FAIL,
                guarantee_tag=None,
                message=f"Unhandled exception (never-silent rule): {exc}",
                detail={"exception": str(exc), "type": type(exc).__name__},
            )

        results.append(result)
        status_str = result.status
        log.info(
            "[%s] %s — %s",
            status_str,
            vid,
            result.message[:120] + ("..." if len(result.message) > 120 else ""),
        )

    # Compute summary
    counts: dict[str, int] = {
        STATUS_PASS: 0,
        STATUS_SKIP: 0,
        STATUS_FAIL: 0,
        STATUS_MOCK_PASS: 0,
    }
    for r in results:
        counts[r.status] = counts.get(r.status, 0) + 1

    any_fail = counts[STATUS_FAIL] > 0
    exit_code = 1 if any_fail else 0
    # Honest overall (SKIP/mock is never a real PASS). Exit code stays 0 unless a
    # real FAIL: MOCK and INCONCLUSIVE are "no failures, but nothing was actually
    # validated against a real model" — not a green light, not an error.
    if any_fail:
        overall = "FAIL"
    elif mock_mode:
        overall = "MOCK"  # fixtures only — never conflated with real PASS
    elif counts[STATUS_SKIP] > 0 or counts[STATUS_PASS] == 0:
        overall = "INCONCLUSIVE"  # something skipped, or nothing passed
    else:
        overall = "PASS"

    summary = {
        "overall": overall,
        "total": len(results),
        "pass": counts[STATUS_PASS],
        "mock_pass": counts[STATUS_MOCK_PASS],
        "skip": counts[STATUS_SKIP],
        "fail": counts[STATUS_FAIL],
        "exit_code": exit_code,
        "mode": "mock"
        if mock_mode
        else ("server" if server_url else ("real" if llama_cli else "skip")),
        "model": model_path or server_url or None,
    }

    mode_label = summary["mode"]
    log.info(
        "=== DONE: overall=%s | pass=%d mock-pass=%d skip=%d fail=%d ===",
        overall,
        counts[STATUS_PASS],
        counts[STATUS_MOCK_PASS],
        counts[STATUS_SKIP],
        counts[STATUS_FAIL],
    )

    # Write dual projection reports (G11)
    json_path = reports_dir / f"{run_id}-report.json"
    txt_path = reports_dir / f"{run_id}-report.txt"

    _write_json_report(json_path, run_id, mode_label, results, summary)
    _write_text_report(txt_path, run_id, mode_label, results, summary)

    log.info("JSON report  : %s", json_path)
    log.info("Text report  : %s", txt_path)
    log.info("Run log      : %s", log_path)

    # Also print the text report to stdout for visibility
    print(txt_path.read_text(encoding="utf-8"))

    return exit_code


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        description=(
            "Mycelium LLM-validation harness. "
            "Validates llama.cpp model behaviour against honesty/correctness properties. "
            "NEVER-SILENT: missing tool/model ⇒ SKIP, never false PASS."
        )
    )
    mode = p.add_mutually_exclusive_group()
    mode.add_argument(
        "--mock",
        action="store_true",
        help=(
            "Dry-run: no model needed. Exercises full plumbing with fixture responses. "
            "All model-dependent validations are labelled mock-PASS. CI/cloud-safe."
        ),
    )
    mode.add_argument(
        "--server",
        metavar="URL",
        help=(
            "llama.cpp HTTP server base URL (e.g. http://localhost:8080). "
            "Uses /completion endpoint."
        ),
    )
    p.add_argument(
        "--model",
        metavar="PATH",
        help="Path to a local .gguf model file (bypasses the model registry).",
    )
    p.add_argument(
        "--llama-cli",
        metavar="PATH",
        dest="llama_cli",
        help=(
            "Path to the llama-cli binary. If omitted, searches PATH, then common "
            "build/install dirs (~/llama.cpp/build/bin, $PREFIX/bin, $MYCELIUM_LLAMA_DIR) "
            "and shallow globs."
        ),
    )
    p.add_argument(
        "--ctx-size",
        type=int,
        default=None,
        dest="ctx_size",
        metavar="N",
        help=(
            "llama.cpp context window (-c). DEFAULT: auto — the harness enumerates "
            "available RAM (/proc/meminfo) and picks a context that fits with headroom, "
            "capped by what the tiny prompts need. This stops llama.cpp sizing the KV "
            "cache for the model's full 32k window, which OOM-kills (SIGKILL/9) a phone. "
            "Pass an explicit N to override the auto value."
        ),
    )
    p.add_argument(
        "--use-swap",
        action="store_true",
        dest="use_swap",
        help=(
            "Let auto context sizing count ~half of free swap toward the memory budget "
            "(bigger context when RAM is tight). Trades speed for capacity — the KV "
            "cache thrashes if it pages out. Off by default."
        ),
    )
    p.add_argument(
        "--cpu-only",
        action="store_true",
        dest="cpu_only",
        help="Force CPU only — never offload to a GPU even if one is detected.",
    )
    p.add_argument(
        "--timeout",
        type=int,
        default=300,
        dest="timeout",
        metavar="SEC",
        help=(
            "Per-generation llama-cli timeout in seconds (default 300). Raise it on a "
            "slow CPU phone (~1-2 tok/s) if a validation times out."
        ),
    )
    p.add_argument(
        "--n-gpu-layers",
        type=int,
        default=None,
        dest="n_gpu_layers",
        metavar="N",
        help=(
            "Model layers to offload to the GPU (llama.cpp -ngl). DEFAULT: auto — full "
            "offload when detected VRAM holds the model, else CPU. 0 = CPU; 999 = all. "
            "Ignored on a CPU-only llama.cpp build (e.g. a phone)."
        ),
    )
    p.add_argument(
        "--llama-arg",
        action="append",
        default=[],
        dest="llama_arg",
        metavar="ARG",
        help=(
            "Extra arg passed through to llama-cli (repeatable), e.g. "
            "--llama-arg=-no-cnv --llama-arg=--no-display-prompt if a real run shows "
            "conversation-mode chatter or the prompt echoed into the output."
        ),
    )
    # --- idempotent model acquisition ---
    p.add_argument(
        "--ensure-model",
        action="store_true",
        dest="ensure_model",
        help=(
            "Check for the selected --model-id and download it ONLY if absent "
            "(idempotent; resumable). Safe to re-run; great for a phone to "
            "prefetch in the background."
        ),
    )
    p.add_argument(
        "--model-id",
        metavar="ID",
        dest="model_id",
        help=(
            f"Registered model to use/fetch (default: {DEFAULT_MODEL_ID}). "
            "Naming one implies --ensure-model. See --list-models."
        ),
    )
    p.add_argument(
        "--model-dir",
        metavar="DIR",
        dest="model_dir",
        help=(
            "Where to cache models (default: $MYCELIUM_LLM_MODEL_DIR or "
            "~/.cache/mycelium-llm-harness/models — outside the repo)."
        ),
    )
    p.add_argument(
        "--model-url",
        metavar="URL",
        dest="model_url",
        help="Override the download URL for --model-id (fetch an arbitrary GGUF).",
    )
    p.add_argument(
        "--model-sha256",
        metavar="HEX",
        dest="model_sha256",
        help=(
            "Expected SHA-256 of the downloaded model. When set (or pinned in the "
            "registry), the download must match it before being accepted — a "
            "mismatch is rejected (supply-chain integrity gate)."
        ),
    )
    p.add_argument(
        "--no-download",
        action="store_true",
        dest="no_download",
        help="With --ensure-model: only check presence; never download.",
    )
    # --- Hugging Face CLI integration ---
    p.add_argument(
        "--hf-cli",
        metavar="PATH",
        dest="hf_cli",
        help=(
            "Path to the `hf` (or legacy `huggingface-cli`) binary. If omitted, "
            "searches PATH then known install dirs (~/.local/bin, $PREFIX/bin)."
        ),
    )
    p.add_argument(
        "--install-hf-cli",
        action="store_true",
        dest="install_hf_cli",
        help=(
            "OPTIONAL: install 'huggingface_hub[cli]' (uv/pipx/pip — never "
            "curl|bash) if the hf CLI is missing. Not needed for downloads (the "
            "stdlib path + $HF_TOKEN cover them) and may fail to build hf-xet on "
            "aarch64/Termux. Prompts unless --yes."
        ),
    )
    p.add_argument(
        "--no-hf-cli",
        action="store_true",
        dest="no_hf_cli",
        help="Disable the hf-CLI path; use the built-in stdlib downloader only.",
    )
    p.add_argument(
        "--hf-token",
        metavar="TOKEN",
        dest="hf_token",
        help=(
            "Hugging Face token for non-interactive login (or set $HF_TOKEN). "
            "Only needed for gated repos; the default registry is public."
        ),
    )
    p.add_argument(
        "--setup-hf",
        action="store_true",
        dest="setup_hf",
        help=(
            "Set up the hf CLI (detect → install if missing → check/prompt auth), "
            "then exit. Implies --install-hf-cli."
        ),
    )
    p.add_argument(
        "-y",
        "--yes",
        action="store_true",
        dest="assume_yes",
        help="Assume yes to prompts (install/login) — for non-interactive runs.",
    )
    p.add_argument(
        "--list-models",
        action="store_true",
        dest="list_models",
        help="Print the registered models and the cache dir, then exit.",
    )
    # --- diagnostics + PATH self-healing ---
    p.add_argument(
        "--doctor",
        action="store_true",
        dest="doctor",
        help=(
            "Diagnose AND heal the environment (llama.cpp, hf CLI, Claude Code "
            "CLI, model cache): install missing tools from the OS package manager "
            "/ npm (pkg/apt/brew, npm — never curl|bash), link an unlinked CLI, and "
            "fix PATH, then exit. The hf CLI is optional (stdlib downloader + "
            "$HF_TOKEN cover downloads) and is not auto-installed. Mutations prompt "
            "unless --yes. Add --check-only for a read-only report."
        ),
    )
    p.add_argument(
        "--check-only",
        action="store_true",
        dest="check_only",
        help=(
            "With --doctor: read-only report — diagnose and print the fix for each "
            "miss, but install nothing and write no PATH changes. Safe to run on a "
            "phone and paste back."
        ),
    )
    p.add_argument(
        "--fix-path",
        action="store_true",
        dest="fix_path",
        help=(
            "When a binary is found off-PATH, append an `export PATH=…` line to "
            "your shell rc (idempotent; prompts unless --yes). PATH is always "
            "healed in-process for the current run regardless of this flag. "
            "(--doctor implies this unless --check-only.)"
        ),
    )
    p.add_argument(
        "--claude-cli",
        metavar="PATH",
        dest="claude_cli",
        help="Explicit path to the `claude` binary (for --doctor/--fix-path).",
    )
    return p


def _console_logger(name: str = "mycelium.llm-harness.setup") -> logging.Logger:
    """A minimal stderr logger for commands that run before run_harness()."""
    log = logging.getLogger(name)
    if not log.handlers:
        log.setLevel(logging.DEBUG)
        sh = logging.StreamHandler(sys.stderr)
        sh.setLevel(logging.INFO)
        sh.setFormatter(
            logging.Formatter(
                "%(asctime)s  %(levelname)-8s  %(message)s", "%Y-%m-%dT%H:%M:%SZ"
            )
        )
        log.addHandler(sh)
    return log


def run_doctor(args: argparse.Namespace) -> int:
    """Diagnose AND heal tool/PATH/auth state. Always exits 0.

    By default doctor is self-healing: if a required tool is missing it installs
    it from the OS package manager / official release — llama.cpp via
    `pkg`/`apt`/`brew` (repo-signed, no source build), Claude Code via npm — never
    `curl|bash` and never a fragile language-package build. It links an
    installed-but-unlinked CLI and fixes PATH (in-process now, and — since healing
    implies --fix-path — persisted to your shell rc). The hf CLI is treated as
    OPTIONAL and is NOT auto-installed (the stdlib downloader + $HF_TOKEN cover
    the model fetch). Every mutation prompts for consent unless --yes;
    non-interactive runs without --yes decline safely (never-silent, G2). Pass
    --check-only for a pure, read-only report ("run it on a phone, paste it back").
    """
    log = _console_logger()
    check_only = bool(getattr(args, "check_only", False))
    heal = not check_only
    # Healing implies persisting PATH fixes too ("fix the paths automatically").
    fix_path = heal or bool(getattr(args, "fix_path", False))
    assume_yes = bool(getattr(args, "assume_yes", False))
    no_hf = bool(getattr(args, "no_hf_cli", False))
    actions: list[str] = []  # what healing actually did, summarised at the end
    lines: list[str] = []

    def out(s: str = "") -> None:
        lines.append(s)

    out("=" * 72)
    out("  Mycelium LLM-harness — environment doctor")
    out("=" * 72)
    out(
        "  Mode     : "
        + (
            "CHECK-ONLY (read-only report; no installs, no PATH writes)"
            if check_only
            else "HEAL (auto-install missing packages + fix PATH; --yes to skip prompts)"
        )
    )
    out(f"  Platform : {sys.platform}{'  (Termux/Android)' if _is_termux() else ''}")
    out(f"  Python   : {sys.version.split()[0]}  ({sys.executable})")
    out(f"  PREFIX   : {os.environ.get('PREFIX', '(unset)')}")
    out(f"  HOME     : {Path.home()}")
    out("  PATH     :")
    for p in _path_entries():
        out(f"      {p}")
    out("")

    out("-" * 72)
    out("  Installers / build tools")
    out("-" * 72)
    for tool in ("uv", "pipx", "npm", "node", "git", "cmake", "make", "clang"):
        w = shutil.which(tool)
        out(f"      {tool:8s}: {'OK  ' + w if w else '–   (not found)'}")
    out(f"      {'pip':8s}: OK  {sys.executable} -m pip")
    out("")

    out("-" * 72)
    # The CLI ships under more than one name: the modern build calls it
    # `llama-cli`, while the Termux `llama-cpp` package installs it as plain
    # `llama`. We accept either (see _LLAMA_BIN_NAMES), so name both here.
    out("  llama.cpp (llama-cli / llama)")
    out("-" * 72)
    llama = _resolve_llama_cli(
        getattr(args, "llama_cli", None), log, fix_path=fix_path, assume_yes=assume_yes
    )
    if not llama and heal:
        out("      NOT FOUND — installing from the system package manager…")
        installed = install_llama_cpp(log, assume_yes=assume_yes, fix_path=fix_path)
        if installed:
            llama = installed
            actions.append("Installed llama.cpp from the system package manager.")
    if llama:
        out(f"      FOUND: {llama}  (alias: {os.path.basename(llama)})")
        try:
            v = subprocess.run(
                [llama, "--version"], capture_output=True, text=True, timeout=20
            )
            blob = (v.stderr or v.stdout).strip()
            # Package builds often omit git metadata, so the binary self-reports
            # `version: 0 (unknown)`. Strip a leading `version:` it already
            # printed so we don't render a confusing `version: version: …`.
            line = blob.splitlines()[0].strip() if blob else "(no output)"
            if line.lower().startswith("version:"):
                line = line.split(":", 1)[1].strip()
            out(f"      version: {line}")
        except Exception as exc:  # noqa: BLE001
            out(f"      version: (could not run --version: {exc})")
    else:
        out("      NOT FOUND. Searched PATH +:")
        for d in _llama_search_dirs():
            out(f"          {d}")
        out("      Fix: install the OS package (Termux: pkg install llama-cpp), or")
        out(
            "           build from source (README Termux steps) + pass --llama-cli PATH,"
        )
        out("           or set $MYCELIUM_LLAMA_DIR to its directory.")
    out("")

    out("-" * 72)
    out("  Hugging Face CLI")
    out("-" * 72)
    cmd, style = _resolve_hf_cli(
        getattr(args, "hf_cli", None), log, fix_path=fix_path, assume_yes=assume_yes
    )
    if cmd:
        out(f"      FOUND: {' '.join(cmd)}  (style: {style})")
        authed, who = hf_auth_status(cmd, style, log)
        out(
            f"      auth : {'authenticated as ' + str(who) if authed else 'NOT authenticated (fine for public models)'}"
        )
    else:
        # OPTIONAL by design. The hf CLI is a Python package that drags in the
        # native `hf-xet` build, which fails on Termux/aarch64 — so doctor does
        # NOT auto-install it. The built-in stdlib downloader handles the public
        # registry, and a gated repo works by exporting $HF_TOKEN (sent as a
        # bearer header by the downloader). Install the CLI yourself only if you
        # specifically want it.
        out("      NOT FOUND — this is OPTIONAL, not a failure.")
        out("      The built-in stdlib downloader fetches the public registry; for")
        out("      a GATED repo, `export HF_TOKEN=hf_…` and the downloader uses it.")
        if not no_hf:
            out("      To install the CLI anyway (may need a native hf-xet build on")
            out(f"      aarch64): --install-hf-cli  (installs '{HF_PIP_PACKAGE}').")
    out("")

    out("-" * 72)
    out("  Claude Code CLI (claude)")
    out("-" * 72)
    claude = _resolve_claude_cli(
        getattr(args, "claude_cli", None),
        log,
        fix_path=fix_path,
        assume_yes=assume_yes,
    )
    if not claude and heal:
        pkg = _claude_package_cli()
        if pkg:
            out(f"      INSTALLED BUT NOT LINKED ({pkg}) — auto-linking…")
            linked = link_claude_cli(pkg, log, assume_yes=assume_yes, fix_path=fix_path)
            if linked:
                claude = linked
                actions.append("Linked the installed Claude Code CLI onto PATH.")
        else:
            out("      NOT FOUND — auto-installing via npm…")
            installed = install_claude_cli(
                log, assume_yes=assume_yes, fix_path=fix_path
            )
            if installed:
                claude = installed
                actions.append("Installed the Claude Code CLI via npm.")
    if claude:
        out(f"      FOUND: {claude}")
        try:
            v = subprocess.run(
                [claude, "--version"], capture_output=True, text=True, timeout=20
            )
            blob = (v.stdout or v.stderr).strip()
            out(f"      version: {blob.splitlines()[0] if blob else '(no output)'}")
        except OSError as exc:
            # e.g. "Exec format error" — a wrong-arch/corrupt binary, NOT a PATH
            # miss. Auto-install can't fix arch; surface the precise reinstall.
            out(f"      version: (could not run --version: {exc})")
            out(
                "      This looks like a wrong-arch or corrupt binary, not a PATH issue."
            )
            out("      Fix: reinstall for this CPU —")
            out(
                f"          npm uninstall -g {CLAUDE_NPM_PACKAGE}; npm install -g {CLAUDE_NPM_PACKAGE}"
            )
            out(
                "          (On Termux: pkg reinstall nodejs first if node itself errors.)"
            )
        except Exception as exc:  # noqa: BLE001 — never-silent
            out(f"      version: (could not run --version: {exc})")
    else:
        pkg = _claude_package_cli()
        if pkg:
            node = shutil.which("node") or "node"
            out(f"      INSTALLED BUT NOT LINKED: package at {pkg}")
            out(
                "      The npm package is there; the `claude` symlink never landed on PATH."
            )
            out("      Fix (one of):")
            out(f'          ln -s "{pkg}" "$PREFIX/bin/claude" && chmod +x "{pkg}"')
            out(
                '          npm config set prefix "$PREFIX" && npm install -g @anthropic-ai/claude-code'
            )
            out(f"          # or just run it directly:  {node} {pkg}")
        else:
            out("      NOT FOUND. Searched PATH +:")
            for d in _claude_search_dirs():
                out(f"          {d}")
            out(
                "      Fix: npm install -g @anthropic-ai/claude-code, and put npm's global"
            )
            out(
                '           bin on PATH. On Termux: npm config set prefix "$PREFIX" first,'
            )
            out("           so links land on the existing PATH.")
    out("")

    out("-" * 72)
    out("  Model cache")
    out("-" * 72)
    md_arg = getattr(args, "model_dir", None)
    md = Path(md_arg).expanduser() if md_arg else default_model_dir()
    spec = MODELS[DEFAULT_MODEL_ID]
    dest = md / str(spec["filename"])
    out(f"      dir: {md}")
    model_ready = is_valid_gguf(dest)
    if model_ready:
        out(
            f"      default model present: {dest.name} ({_human_bytes(dest.stat().st_size)})"
        )
    elif heal and _confirm(
        f"Default model absent. Download {dest.name} (~GB) now?",
        assume_yes=assume_yes,
        log=log,
    ):
        got = ensure_model(
            DEFAULT_MODEL_ID,
            md,
            log,
            allow_download=True,
            hf_cmd=(cmd if not no_hf else None),
            prefer_hf=not no_hf,
        )
        if got and is_valid_gguf(got):
            out(
                f"      default model fetched: {got.name} ({_human_bytes(got.stat().st_size)})"
            )
            model_ready = True
            actions.append(f"Downloaded the default model ({got.name}).")
        else:
            out(
                f"      default model still absent: {dest.name}  (download failed — see log)"
            )
    else:
        out(f"      default model absent : {dest.name}  (fetch with --ensure-model)")
    out("")

    out("-" * 72)
    out("  Memory + auto context size")
    out("-" * 72)
    mem = detect_memory()
    out(f"      source   : {mem.get('source')}")

    def _mb(v: Any) -> str:
        return f"{v} MB" if isinstance(v, int) else "(unknown)"

    out(
        f"      RAM      : {_mb(mem.get('mem_available_mb'))} available "
        f"of {_mb(mem.get('mem_total_mb'))} total"
    )
    swap_free = mem.get("swap_free_mb")
    out(
        f"      swap     : {_mb(swap_free)} free of {_mb(mem.get('swap_total_mb'))} "
        "(off by default; add --use-swap to count ~half toward the budget)"
    )
    # Show the context a real run WOULD auto-pick for the default model (the auto-sizer
    # logs its own reasoning; here we just surface the number alongside the inputs).
    model_for_size = str(dest) if model_ready else None
    chosen = auto_ctx_size(_HARNESS_DESIRED_CTX, model_for_size, mem, log)
    out(f"      auto ctx : {chosen}  (override with --ctx-size N)")
    if isinstance(swap_free, int) and swap_free > 0:
        with_swap = auto_ctx_size(
            _HARNESS_DESIRED_CTX, model_for_size, mem, log, swap_fraction=0.5
        )
        if with_swap != chosen:
            out(
                f"      with --use-swap : {with_swap}  (slower if the KV cache pages out)"
            )
    out("")

    out("-" * 72)
    out("  GPU (desktop offload)")
    out("-" * 72)
    gpus = detect_gpu()
    if gpus:
        for g in gpus:
            vram = (
                f"{g['vram_free_mb']} MB free of {g['vram_total_mb']} MB"
                if isinstance(g.get("vram_total_mb"), int)
                else "VRAM unknown"
            )
            out(f"      {g['backend']:5s}: {g['name']}  ({vram})")
        ngl = auto_gpu_layers(gpus, model_for_size, log)
        out(
            f"      auto -ngl: {ngl}  "
            + (
                "(full offload)"
                if ngl >= 999
                else "(CPU — VRAM too small)"
                if ngl == 0
                else ""
            )
        )
        out("      Override: --n-gpu-layers N, or force CPU with --cpu-only.")
    else:
        out("      none detected (CPU only). On a phone this is expected; on desktop,")
        out(
            "      install a CUDA/ROCm/Metal llama.cpp build + driver tooling (nvidia-smi)."
        )
    out("")

    out("-" * 72)
    out("  External storage (overflow)")
    out("-" * 72)
    vols = detect_external_storage()
    if vols:
        for v in vols:
            out(f"      {v['path']}: {v['free_mb']} MB free of {v['total_mb']} MB")
        out("      A roomy volume can host the model cache (--model-dir DIR) to keep")
        out(
            "      internal storage free, or back a swapfile for --use-swap (needs root:"
        )
        out("      mkswap + swapon on a file there).")
    else:
        out(
            "      none detected (run `termux-setup-storage` to expose shared/SD storage)."
        )
    out("")

    # Bottom-line readiness verdict — the one thing to read in this dense report.
    # Real-mode validations need BOTH a llama.cpp CLI and a local model; the cached
    # model is reused with no flags (the walk-away property). A SKIP is never a PASS,
    # so we say plainly whether the next command will actually run or skip (G2).
    out("-" * 72)
    out("  Readiness (real-mode validations)")
    out("-" * 72)
    if llama and model_ready:
        out("      ✓ READY — llama.cpp CLI + a local model are both present.")
        out("      Next: run the validations (auto-finds the CLI + cached model):")
        out("          python tools/llm-harness/harness.py")
    else:
        out("      ✗ NOT READY — real-mode validations would SKIP (not fail), because:")
        if not llama:
            out(
                "          · no llama.cpp CLI (llama-cli / llama) — see the section above"
            )
        if not model_ready:
            out("          · no local model — fetch it with:")
            out("                python tools/llm-harness/harness.py --ensure-model")
        out("      (Use --mock for a CI-safe fixture run that needs neither.)")
    out("")

    out("-" * 72)
    if check_only:
        out("  Result: check-only — no changes made.")
        out(
            "  Re-run without --check-only to auto-install missing packages + fix PATH,"
        )
        out("  or with --fix-path to only persist an off-PATH binary to your shell rc.")
    elif actions:
        out("  Healed:")
        for a in actions:
            out(f"      ✓ {a}")
        out("  Open a new shell (or `source` your rc) so persisted PATH fixes apply.")
    else:
        out("  Nothing to heal — everything required was already present, or fixes")
        out("  were declined/unavailable (see the per-tool notes above).")
    out("=" * 72)
    print("\n".join(lines))
    return 0


def _dispatch(args: argparse.Namespace) -> int:
    if getattr(args, "list_models", False):
        print(list_models_text())
        return 0
    if getattr(args, "doctor", False):
        return run_doctor(args)
    if getattr(args, "setup_hf", False):
        # Standalone: detect → install (opt-in implied) → auth check/prompt, then exit.
        log = _console_logger()
        cli, _ = ensure_hf_ready(
            getattr(args, "hf_cli", None),
            log,
            allow_install=True,  # --setup-hf is an explicit request to install
            assume_yes=bool(getattr(args, "assume_yes", False)),
            want_auth=True,
            token=getattr(args, "hf_token", None),
            fix_path=bool(getattr(args, "fix_path", False)),
        )
        return 0 if cli else 1
    return run_harness(args)


def main() -> None:
    args = _build_parser().parse_args()
    try:
        code = _dispatch(args)
    except BrokenPipeError:
        code = 0  # downstream pager/pipe closed — not our error
    # Flush explicitly: on some platforms (e.g. Termux/Android) an unflushed
    # buffer at interpreter teardown can surface as a spurious "Aborted".
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.flush()
        except (BrokenPipeError, ValueError, OSError):
            pass
    sys.exit(code)


if __name__ == "__main__":
    main()
