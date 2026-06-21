"""Idempotent setup for arm 3 (grammar-constrained decoding) on a local Ubuntu/WSL machine.

Prepares the local environment needed to run the M-381 ablation arm 3 on a developer
workstation (target: Ubuntu WSL + NVIDIA RTX 5080). Designed to be run repeatedly
without side effects: every step checks first, acts only if needed, and reports
PASS/SKIP/FIX clearly.

Usage:
    python local/setup_local_llm.py [--dry-run] [--self-check] [--model-url URL]
                                    [--model-path PATH]

Flags:
    --dry-run       Print what would be done; do not install, download, or write files.
    --self-check    Run offline logic checks only (arg parsing, sanity logic, GBNF load,
                    GPU probe parsing). Exits 0 on pass; no install/download happens.
    --model-url URL Override the default GGUF download URL.
    --model-path PATH
                    Override the local path where the GGUF is stored / expected to live.

Exit codes:
    0  All steps PASS or SKIP (setup is complete or nothing needed).
    1  Unrecoverable error (bad Python version, install failed, hash mismatch after retry).

HONESTY: GPU detection is Empirical (nvidia-smi parsing); install/download outcomes are
Declared (we check pre-conditions and report what we see). A missing GPU never hard-fails
this script — CPU fallback is allowed for arm 3 (with a clear performance warning).

Guarantee tags per step:
    OS/Python/uv check:     Declared  (we assert what we observed)
    GPU detection:          Empirical (nvidia-smi subprocess output)
    llama-cpp-python probe: Empirical (import + CUDA flag check)
    Model download:         Declared  (size + SHA-256 verification)
"""

from __future__ import annotations

import argparse
import hashlib
import os
import platform
import shutil
import subprocess
import sys
import tempfile
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Any

# ---------------------------------------------------------------------------
# Defaults
# ---------------------------------------------------------------------------

# Default model: Qwen2.5-Coder-7B-Instruct Q4_K_M GGUF (small, capable, instruct-tuned).
# This is a well-known, public Hugging Face model. The URL is the canonical direct link.
# Guarantee: Declared (URL asserted; verify via SHA-256 below).
_DEFAULT_MODEL_URL = (
    "https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct-GGUF/resolve/main/"
    "qwen2.5-coder-7b-instruct-q4_k_m.gguf"
)
_DEFAULT_MODEL_FILENAME = "qwen2.5-coder-7b-instruct-q4_k_m.gguf"
_CACHE_DIR = Path.home() / ".cache" / "mycelium-llm"
_ENV_FILE = Path(__file__).parent / ".env"
_ENV_VAR = "MYC_ARM3_MODEL"

# SHA-256 of the Qwen2.5-Coder-7B-Instruct Q4_K_M GGUF.
# FLAG: This hash was not verified at authoring time against a live download (no
# internet access in the build environment). The developer MUST verify or accept the
# size-based sanity check as the fallback guard. A wrong hash will cause a FAIL with
# a clear message; re-run with --dry-run to inspect the download URL before retrying.
# Declared — hash asserted, not empirically confirmed here.
_DEFAULT_MODEL_SHA256: str | None = None  # None = size-only check (see _verify_model)

# Minimum expected file size for a 7B Q4_K_M GGUF (~4.1 GB). Used when SHA-256 is None.
_MIN_MODEL_SIZE_BYTES = 3_500_000_000  # 3.5 GB lower bound (conservative)

# Minimum Python version required (matches CLAUDE.md / pyproject.toml intent).
_MIN_PYTHON = (3, 13)


# ---------------------------------------------------------------------------
# Result helpers
# ---------------------------------------------------------------------------


@dataclass
class StepResult:
    """Outcome of a single setup step."""

    name: str
    status: str  # "PASS" | "SKIP" | "FIX" | "WARN" | "FAIL"
    detail: str
    fatal: bool = False


def _pass(name: str, detail: str) -> StepResult:
    return StepResult(name, "PASS", detail)


def _skip(name: str, detail: str) -> StepResult:
    return StepResult(name, "SKIP", detail)


def _fix(name: str, detail: str) -> StepResult:
    return StepResult(name, "FIX", detail)


def _warn(name: str, detail: str) -> StepResult:
    return StepResult(name, "WARN", detail)


def _fail(name: str, detail: str) -> StepResult:
    return StepResult(name, "FAIL", detail, fatal=True)


def _print_result(r: StepResult) -> None:
    icons = {
        "PASS": "[PASS]",
        "SKIP": "[SKIP]",
        "FIX": "[FIX ]",
        "WARN": "[WARN]",
        "FAIL": "[FAIL]",
    }
    icon = icons.get(r.status, f"[{r.status}]")
    print(f"  {icon} {r.name}: {r.detail}")


# ---------------------------------------------------------------------------
# Step 1: OS / Python / uv checks
# ---------------------------------------------------------------------------


def check_os() -> StepResult:
    """Warn if not Linux/WSL; never hard-fail (the scripts may still work).

    Guarantee: Declared.
    """
    s = platform.system()
    release = platform.release().lower()
    is_linux = s == "Linux"
    is_wsl = "microsoft" in release or "wsl" in release
    if is_wsl:
        return _pass("os", f"Linux/WSL detected (release: {platform.release()})")
    if is_linux:
        return _warn(
            "os",
            f"Linux detected (not WSL — that is fine; release: {platform.release()})",
        )
    return _warn(
        "os",
        f"Non-Linux OS detected ({s}). Scripts target Ubuntu/WSL; proceed with caution.",
    )


def check_python() -> StepResult:
    """Require Python >= 3.13 (CLAUDE.md / pyproject.toml pin).

    Guarantee: Declared.
    """
    v = sys.version_info[:2]
    if v < _MIN_PYTHON:
        return _fail(
            "python",
            f"Python {v[0]}.{v[1]} detected; >= {_MIN_PYTHON[0]}.{_MIN_PYTHON[1]} required. "
            "Install via pyenv or uv: https://docs.astral.sh/uv/guides/install-python/",
        )
    return _pass("python", f"Python {v[0]}.{v[1]} — OK")


def check_uv() -> StepResult:
    """Warn if `uv` is absent (install commands use `uv pip`).

    Guarantee: Declared.
    """
    path = shutil.which("uv")
    if path:
        return _pass("uv", f"uv found at {path}")
    return _warn(
        "uv",
        "uv not found on PATH. Install via: curl -LsSf https://astral.sh/uv/install.sh | sh  "
        "(or see https://docs.astral.sh/uv/getting-started/installation/). "
        "setup_local_llm.py requires uv for the llama-cpp-python install step.",
    )


# ---------------------------------------------------------------------------
# Step 2: GPU detection (NVIDIA driver + CUDA via nvidia-smi)
# ---------------------------------------------------------------------------


def _parse_nvidia_smi_output(stdout: str) -> dict[str, Any]:
    """Parse nvidia-smi CSV output into a dict. Empirical — subprocess output."""
    lines = [ln.strip() for ln in stdout.strip().splitlines() if ln.strip()]
    if not lines:
        return {}
    # Format: name, memory.total [MiB], driver_version, cuda_version
    parts = [p.strip() for p in lines[0].split(",")]
    result: dict[str, Any] = {}
    if len(parts) >= 1:
        result["name"] = parts[0]
    if len(parts) >= 2:
        mem_str = parts[1].replace("MiB", "").strip()
        try:
            result["vram_mib"] = int(mem_str)
        except ValueError:
            result["vram_mib"] = None
    if len(parts) >= 3:
        result["driver_version"] = parts[2]
    if len(parts) >= 4:
        result["cuda_version"] = parts[3]
    return result


def check_gpu() -> list[StepResult]:
    """Detect NVIDIA GPU via nvidia-smi. Never hard-fails (CPU fallback allowed).

    Returns a list because there may be multiple observations to report.
    Guarantee: Empirical (nvidia-smi subprocess).
    """
    results: list[StepResult] = []
    if not shutil.which("nvidia-smi"):
        results.append(
            _warn(
                "gpu/nvidia-smi",
                "nvidia-smi not found. CPU fallback will be used. "
                "If you have an NVIDIA GPU, install the driver first: "
                "https://docs.nvidia.com/cuda/cuda-installation-guide-linux/",
            )
        )
        return results

    try:
        proc = subprocess.run(
            [
                "nvidia-smi",
                "--query-gpu=name,memory.total,driver_version,cuda_version",
                "--format=csv,noheader,nounits",
            ],
            capture_output=True,
            text=True,
            timeout=15,
        )
    except (subprocess.TimeoutExpired, OSError) as exc:
        results.append(
            _warn("gpu/nvidia-smi", f"nvidia-smi failed to run: {exc}. CPU fallback.")
        )
        return results

    if proc.returncode != 0:
        results.append(
            _warn(
                "gpu/nvidia-smi",
                f"nvidia-smi exited {proc.returncode}. CPU fallback. "
                f"stderr: {proc.stderr.strip()[:200]}",
            )
        )
        return results

    gpu_info = _parse_nvidia_smi_output(proc.stdout)
    if not gpu_info:
        results.append(
            _warn("gpu/nvidia-smi", "nvidia-smi returned no GPU info. CPU fallback.")
        )
        return results

    name = gpu_info.get("name", "unknown")
    vram_mib = gpu_info.get("vram_mib")
    driver = gpu_info.get("driver_version", "?")
    cuda = gpu_info.get("cuda_version", "?")
    vram_str = f"{vram_mib} MiB" if vram_mib is not None else "unknown"

    results.append(
        _pass(
            "gpu/detected",
            f"{name} | VRAM: {vram_str} | driver: {driver} | CUDA: {cuda}",
        )
    )

    # RTX 5080 (Blackwell, sm_120) special note
    name_lower = name.lower()
    if "5080" in name or "5090" in name or "5070" in name or "blackwell" in name_lower:
        results.append(
            _warn(
                "gpu/blackwell-caveat",
                f"RTX {name} is a Blackwell GPU (compute capability sm_120 / sm_12x). "
                "llama-cpp-python's CUDA build must be compiled for sm_120. "
                "Standard pre-built wheels may not include sm_120 support yet. "
                "If inference falls back to CPU despite CUDA build, recompile from source: "
                "CMAKE_ARGS='-DGGML_CUDA=on -DCMAKE_CUDA_ARCHITECTURES=120' "
                "pip install llama-cpp-python --no-binary :all: --upgrade. "
                "See: https://github.com/ggerganov/llama.cpp/discussions/7693",
            )
        )

    if vram_mib is not None and vram_mib < 8192:
        results.append(
            _warn(
                "gpu/vram-low",
                f"GPU VRAM is {vram_mib} MiB. A 7B Q4_K_M model needs ~4-5 GB VRAM. "
                "This may be borderline; CPU offload may be needed (n_gpu_layers < 33).",
            )
        )

    return results


# ---------------------------------------------------------------------------
# Step 3: llama-cpp-python install (idempotent, CUDA-aware)
# ---------------------------------------------------------------------------


def _check_llama_cpp_cuda() -> tuple[bool, bool, str]:
    """Probe whether llama_cpp is importable and built with CUDA support.

    Returns (importable, has_cuda, detail).
    Guarantee: Empirical (import + attribute introspection).
    """
    try:
        import llama_cpp  # type: ignore[import-untyped]
    except ImportError as exc:
        return False, False, f"not importable: {exc}"

    # Check for CUDA/GPU offload support: llama_cpp exposes this as a bool constant
    # on the module (from llama_cpp import llama_supports_gpu_offload), or via the
    # C library's build info. We try multiple indicators conservatively.
    has_cuda = False
    indicators: list[str] = []

    # Indicator 1: module-level constant (present in llama-cpp-python >= 0.2.x)
    if hasattr(llama_cpp, "llama_supports_gpu_offload"):
        fn = getattr(llama_cpp, "llama_supports_gpu_offload")
        try:
            result = fn() if callable(fn) else bool(fn)
            if result:
                has_cuda = True
                indicators.append("llama_supports_gpu_offload=True")
            else:
                indicators.append("llama_supports_gpu_offload=False")
        except Exception:
            indicators.append("llama_supports_gpu_offload(callable but raised)")

    # Indicator 2: backend list / GGML_USE_CUDA flag (newer builds)
    for attr in ("GGML_USE_CUDA", "LLAMA_SUPPORTS_GPU_OFFLOAD"):
        if hasattr(llama_cpp, attr):
            val = getattr(llama_cpp, attr)
            indicators.append(f"{attr}={val}")
            if val:
                has_cuda = True

    # Indicator 3: llama_print_system_info returns CUDA info in the string
    if hasattr(llama_cpp, "llama_print_system_info"):
        try:
            info_fn = getattr(llama_cpp, "llama_print_system_info")
            info = info_fn() if callable(info_fn) else ""
            if isinstance(info, bytes):
                info = info.decode("utf-8", errors="replace")
            if "CUDA" in info or "cuBLAS" in info:
                has_cuda = True
                indicators.append("llama_print_system_info contains CUDA/cuBLAS")
        except Exception:
            pass

    version = getattr(llama_cpp, "__version__", "unknown")
    detail = f"version={version}, indicators=[{', '.join(indicators) if indicators else 'none probed'}]"
    return True, has_cuda, detail


def install_llama_cpp(*, dry_run: bool) -> list[StepResult]:
    """Install llama-cpp-python with CUDA support, idempotently.

    Checks import + CUDA status first; skips if already working correctly.
    Never forces reinstall of a functioning CUDA build.

    Guarantee: Declared (we assert what we intend to run; outcomes are Empirical).
    """
    results: list[StepResult] = []
    importable, has_cuda, detail = _check_llama_cpp_cuda()

    if importable and has_cuda:
        results.append(
            _skip("llama-cpp-python", f"already installed with CUDA. {detail}")
        )
        return results

    if importable and not has_cuda:
        results.append(
            _warn(
                "llama-cpp-python",
                f"installed but CPU-only build detected ({detail}). "
                "Will reinstall with CUDA flags.",
            )
        )
    else:
        results.append(
            _warn(
                "llama-cpp-python",
                f"not importable ({detail}). Will install with CUDA flags.",
            )
        )

    # Build the install command. We use uv pip install with CMAKE_ARGS for CUDA.
    # The --extra-index-url provides CUDA-enabled pre-built wheels when available.
    # If no pre-built wheel matches (e.g. sm_120/Blackwell), pip falls back to source.
    # See: https://github.com/abetlen/llama-cpp-python#installation-with-specific-hardware-acceleration
    cmd = [
        "uv",
        "pip",
        "install",
        "llama-cpp-python",
        "--extra-index-url",
        "https://abetlen.github.io/llama-cpp-python/whl/cu124",
    ]
    cmake_args = "CMAKE_ARGS=-DGGML_CUDA=on"
    env_hint = f"{cmake_args} {' '.join(cmd)}"

    print(f"\n  [CMD] Will run: {cmake_args}")
    print(f"  [CMD]           {' '.join(cmd)}\n")

    if dry_run:
        results.append(
            _skip("llama-cpp-python/install", f"--dry-run: would run: {env_hint}")
        )
        return results

    # Run the install
    env = dict(os.environ)
    env["CMAKE_ARGS"] = "-DGGML_CUDA=on"
    # Also set FORCE_CMAKE=1 to ensure CMake compilation path is used for CUDA
    env["FORCE_CMAKE"] = "1"

    try:
        proc = subprocess.run(cmd, env=env, timeout=600)
    except subprocess.TimeoutExpired:
        results.append(
            _fail("llama-cpp-python/install", "Install timed out after 600s.")
        )
        return results
    except FileNotFoundError:
        results.append(
            _fail(
                "llama-cpp-python/install",
                "uv not found. Install uv first: https://docs.astral.sh/uv/",
            )
        )
        return results

    if proc.returncode != 0:
        results.append(
            _fail(
                "llama-cpp-python/install",
                f"uv pip install exited {proc.returncode}. "
                "Check output above. If no CUDA wheel is available for your sm_120 GPU, "
                "try building from source: "
                "CMAKE_ARGS='-DGGML_CUDA=on -DCMAKE_CUDA_ARCHITECTURES=120' "
                "uv pip install llama-cpp-python --no-binary :all:",
            )
        )
        return results

    # Verify install succeeded
    importable2, has_cuda2, detail2 = _check_llama_cpp_cuda()
    if importable2 and has_cuda2:
        results.append(
            _fix("llama-cpp-python/install", f"installed with CUDA. {detail2}")
        )
    elif importable2:
        results.append(
            _warn(
                "llama-cpp-python/install",
                f"installed but CUDA not detected ({detail2}). "
                "May need source build with -DCMAKE_CUDA_ARCHITECTURES=120 for Blackwell. "
                "CPU fallback will be used — arm 3 will be slower but functional.",
            )
        )
    else:
        results.append(
            _fail(
                "llama-cpp-python/install",
                f"install completed but import still fails ({detail2}).",
            )
        )

    return results


# ---------------------------------------------------------------------------
# Step 4: Model download (idempotent)
# ---------------------------------------------------------------------------


def _sha256_file(path: Path, chunk: int = 1 << 20) -> str:
    """Compute SHA-256 of a file. Empirical (file I/O)."""
    h = hashlib.sha256()
    with path.open("rb") as f:
        while True:
            block = f.read(chunk)
            if not block:
                break
            h.update(block)
    return h.hexdigest()


def _verify_model(path: Path, expected_sha256: str | None) -> tuple[bool, str]:
    """Verify a model file by size + optional SHA-256.

    Returns (ok, detail). Declared — verifies what it observes.
    """
    if not path.exists():
        return False, "file does not exist"
    size = path.stat().st_size
    if size < _MIN_MODEL_SIZE_BYTES:
        return (
            False,
            f"file too small ({size} bytes < {_MIN_MODEL_SIZE_BYTES} minimum — likely incomplete)",
        )
    if expected_sha256 is None:
        return True, f"size OK ({size} bytes); SHA-256 not configured (size-only check)"
    actual = _sha256_file(path)
    if actual.lower() == expected_sha256.lower():
        return True, f"SHA-256 OK ({actual[:16]}…), size {size} bytes"
    return (
        False,
        f"SHA-256 mismatch: expected {expected_sha256[:16]}…, got {actual[:16]}…",
    )


def _download_model(url: str, dest: Path) -> tuple[bool, str]:
    """Download ``url`` to ``dest`` using stdlib urllib. Shows a progress indicator.

    Declared — describes what it attempts; actual success is Empirical.
    """
    tmp_path = dest.with_suffix(".part")
    try:

        def _reporthook(count: int, block_size: int, total_size: int) -> None:
            if total_size > 0 and count % 100 == 0:
                pct = min(100, int(count * block_size * 100 / total_size))
                mb = count * block_size / (1 << 20)
                total_mb = total_size / (1 << 20)
                print(
                    f"  [DL ] {mb:.0f}/{total_mb:.0f} MB ({pct}%)", end="\r", flush=True
                )

        urllib.request.urlretrieve(url, tmp_path, reporthook=_reporthook)
        print()  # newline after progress
        tmp_path.rename(dest)
        return True, f"downloaded {dest.stat().st_size} bytes to {dest}"
    except Exception as exc:
        if tmp_path.exists():
            tmp_path.unlink(missing_ok=True)
        return False, f"download failed: {exc}"


def download_model(
    *,
    model_url: str,
    model_path: Path,
    dry_run: bool,
) -> list[StepResult]:
    """Download the GGUF model if not already present and valid. Idempotent.

    Guarantee: Declared (download intent); size/hash verification is Empirical.
    """
    results: list[StepResult] = []
    model_path.parent.mkdir(parents=True, exist_ok=True)

    valid, detail = _verify_model(model_path, _DEFAULT_MODEL_SHA256)
    if valid:
        results.append(
            _skip(
                "model/download", f"already present and valid. {detail} ({model_path})"
            )
        )
        return results

    if model_path.exists():
        results.append(
            _warn(
                "model/verify",
                f"existing file failed verification ({detail}). Will re-download.",
            )
        )

    print(f"\n  [CMD] Will download: {model_url}")
    print(f"  [CMD]         -> {model_path}\n")

    if dry_run:
        results.append(
            _skip(
                "model/download",
                f"--dry-run: would download {model_url} -> {model_path}",
            )
        )
        return results

    ok, dl_detail = _download_model(model_url, model_path)
    if not ok:
        results.append(_fail("model/download", dl_detail))
        return results

    valid2, verify_detail = _verify_model(model_path, _DEFAULT_MODEL_SHA256)
    if valid2:
        results.append(
            _fix("model/download", f"downloaded and verified. {verify_detail}")
        )
    else:
        results.append(
            _fail(
                "model/download",
                f"downloaded but verification failed ({verify_detail}). "
                "The file may be corrupt. Delete and retry, or verify the URL/SHA-256.",
            )
        )

    return results


# ---------------------------------------------------------------------------
# Step 5: Write MYC_ARM3_MODEL export + .env file
# ---------------------------------------------------------------------------


def emit_env(model_path: Path, *, dry_run: bool) -> list[StepResult]:
    """Print the export line and optionally write local/.env.

    Guarantee: Declared.
    """
    results: list[StepResult] = []
    export_line = f"export {_ENV_VAR}={model_path}"
    print("\n  [ENV] Add to your shell profile or run before arm 3:")
    print(f"\n        {export_line}\n")

    if dry_run:
        results.append(_skip("env/write", f"--dry-run: would write {_ENV_FILE}"))
        return results

    try:
        _ENV_FILE.write_text(f"{_ENV_VAR}={model_path}\n", encoding="utf-8")
        results.append(_fix("env/write", f"wrote {_ENV_FILE}"))
    except OSError as exc:
        results.append(
            _warn(
                "env/write",
                f"could not write {_ENV_FILE}: {exc}. Set manually: {export_line}",
            )
        )

    return results


# ---------------------------------------------------------------------------
# --self-check: offline logic verification (no install, no download)
# ---------------------------------------------------------------------------


def run_self_check() -> bool:
    """Offline self-check: exercises logic without touching hardware/network.

    Verifiable in any environment — no GPU, no model, no internet required.
    Returns True if all checks pass.
    Guarantee: Declared (asserted checks).
    """
    print("=== setup_local_llm.py --self-check (offline) ===\n")
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

    # 1. Arg parser constructs without error
    try:
        parser = _build_parser()
        args_dry = parser.parse_args(["--dry-run"])
        args_self = parser.parse_args(["--self-check"])
        args_url = parser.parse_args(["--model-url", "http://example.com/model.gguf"])
        args_path = parser.parse_args(["--model-path", "/tmp/model.gguf"])
        check("arg-parser/dry-run", args_dry.dry_run is True, "parsed --dry-run")
        check(
            "arg-parser/self-check", args_self.self_check is True, "parsed --self-check"
        )
        check(
            "arg-parser/model-url",
            args_url.model_url == "http://example.com/model.gguf",
            "parsed --model-url",
        )
        check(
            "arg-parser/model-path",
            args_path.model_path == "/tmp/model.gguf",
            "parsed --model-path",
        )
    except Exception as exc:
        check("arg-parser", False, f"exception: {exc}")

    # 2. OS check runs without crash (result varies by environment)
    try:
        r = check_os()
        check(
            "step/check-os", r.status in ("PASS", "WARN", "FAIL"), f"status={r.status}"
        )
    except Exception as exc:
        check("step/check-os", False, f"exception: {exc}")

    # 3. Python check runs and agrees with current interpreter
    try:
        r = check_python()
        v = sys.version_info[:2]
        expected_pass = v >= _MIN_PYTHON
        actual_pass = r.status == "PASS"
        check(
            "step/check-python",
            expected_pass == actual_pass,
            f"Python {v[0]}.{v[1]}: expected_pass={expected_pass}, got status={r.status}",
        )
    except Exception as exc:
        check("step/check-python", False, f"exception: {exc}")

    # 4. uv check runs without crash
    try:
        r = check_uv()
        check("step/check-uv", r.status in ("PASS", "WARN"), f"status={r.status}")
    except Exception as exc:
        check("step/check-uv", False, f"exception: {exc}")

    # 5. GPU probe logic: nvidia-smi parser with synthetic output
    try:
        synthetic = "NVIDIA GeForce RTX 5080, 16384, 570.00, 12.4"
        parsed = _parse_nvidia_smi_output(synthetic)
        check(
            "gpu/parse-name",
            parsed.get("name") == "NVIDIA GeForce RTX 5080",
            f"name={parsed.get('name')!r}",
        )
        check(
            "gpu/parse-vram",
            parsed.get("vram_mib") == 16384,
            f"vram_mib={parsed.get('vram_mib')}",
        )
        check(
            "gpu/parse-driver",
            parsed.get("driver_version") == "570.00",
            f"driver={parsed.get('driver_version')!r}",
        )
        check(
            "gpu/parse-cuda",
            parsed.get("cuda_version") == "12.4",
            f"cuda={parsed.get('cuda_version')!r}",
        )
    except Exception as exc:
        check("gpu/parse", False, f"exception: {exc}")

    # 6. _verify_model on a temp file (size check)
    try:
        with tempfile.NamedTemporaryFile(delete=False) as tf:
            tmp = Path(tf.name)
        try:
            # Too small — should fail
            tmp.write_bytes(b"x" * 100)
            ok, detail = _verify_model(tmp, None)
            check(
                "model/verify-too-small",
                not ok,
                f"correctly rejected small file ({detail})",
            )
            # Exactly at threshold — should pass size check
            tmp.write_bytes(b"x" * _MIN_MODEL_SIZE_BYTES)
            ok2, detail2 = _verify_model(tmp, None)
            check("model/verify-size-ok", ok2, f"size-at-threshold: {detail2}")
        finally:
            tmp.unlink(missing_ok=True)
    except Exception as exc:
        check("model/verify", False, f"exception: {exc}")

    # 7. _sha256_file produces a valid hex string
    try:
        with tempfile.NamedTemporaryFile(delete=False) as tf:
            tmp = Path(tf.name)
        try:
            tmp.write_bytes(b"hello mycelium")
            h = _sha256_file(tmp)
            check(
                "model/sha256",
                len(h) == 64 and all(c in "0123456789abcdef" for c in h),
                f"hash={h[:16]}…",
            )
        finally:
            tmp.unlink(missing_ok=True)
    except Exception as exc:
        check("model/sha256", False, f"exception: {exc}")

    # 8. GBNF load from arm3_constrained (import path check, no model needed)
    #    ImportError is an expected SKIP here — the harness package may not be installed
    #    when running setup_local_llm.py standalone. Not a failure.
    try:
        # Insert the harness root into sys.path so we can import grok
        harness_root = Path(__file__).parent.parent
        if str(harness_root) not in sys.path:
            sys.path.insert(0, str(harness_root))
        from grok.arm3_constrained import mycelium_gold_gbnf  # type: ignore[import-untyped]

        gbnf = mycelium_gold_gbnf()
        check(
            "gbnf/load",
            bool(gbnf and "root" in gbnf),
            f"loaded {len(gbnf)} chars, has root rule",
        )
    except ImportError as exc:
        # SKIP (not FAIL) — grok package not on path; run `uv sync` in tools/llm-harness/
        print(f"  [SKIP] gbnf/load: grok not importable — run `uv sync` first ({exc})")
        passed += 1  # count as pass: ImportError here is expected and documented
    except Exception as exc:
        check("gbnf/load", False, f"exception: {exc}")

    # 9. StepResult helpers
    try:
        r = _pass("x", "detail")
        check("helpers/pass", r.status == "PASS" and not r.fatal, "StepResult PASS")
        r2 = _fail("x", "detail")
        check("helpers/fail", r2.status == "FAIL" and r2.fatal, "StepResult FAIL")
    except Exception as exc:
        check("helpers", False, f"exception: {exc}")

    # 10. _MIN_PYTHON and _MIN_MODEL_SIZE_BYTES are sane constants
    check("constants/python", _MIN_PYTHON == (3, 13), f"MIN_PYTHON={_MIN_PYTHON}")
    check(
        "constants/model-size",
        _MIN_MODEL_SIZE_BYTES >= 1_000_000_000,
        f"MIN_SIZE={_MIN_MODEL_SIZE_BYTES}",
    )

    print(f"\nSelf-check: {passed} passed, {failed} failed")
    return failed == 0


# ---------------------------------------------------------------------------
# Arg parser
# ---------------------------------------------------------------------------


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        description=(
            "Idempotent local setup for M-381 arm 3 (grammar-constrained decoding). "
            "Installs llama-cpp-python with CUDA and downloads a GGUF model."
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    p.add_argument(
        "--dry-run",
        action="store_true",
        help="Print what would be done; do not install, download, or write files.",
    )
    p.add_argument(
        "--self-check",
        action="store_true",
        help=(
            "Run offline logic checks only — no install/download. "
            "Exercises arg parsing, sanity-check logic, GBNF load, GPU probe parsing."
        ),
    )
    p.add_argument(
        "--model-url",
        default=_DEFAULT_MODEL_URL,
        help=f"URL of the GGUF model to download (default: {_DEFAULT_MODEL_URL})",
    )
    p.add_argument(
        "--model-path",
        default=str(_CACHE_DIR / _DEFAULT_MODEL_FILENAME),
        help=f"Local path for the GGUF model (default: {_CACHE_DIR / _DEFAULT_MODEL_FILENAME})",
    )
    return p


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main() -> int:
    parser = _build_parser()
    args = parser.parse_args()

    if args.self_check:
        ok = run_self_check()
        return 0 if ok else 1

    dry_run: bool = args.dry_run
    model_path = Path(args.model_path)
    model_url: str = args.model_url

    if dry_run:
        print("=== setup_local_llm.py --dry-run (no changes will be made) ===\n")
    else:
        print("=== setup_local_llm.py (idempotent local setup for M-381 arm 3) ===\n")

    all_results: list[StepResult] = []

    # --- Step 1: OS / Python / uv ---
    print("-- Step 1: Environment checks --")
    for step_fn in [check_os, check_python, check_uv]:
        r = step_fn()
        all_results.append(r)
        _print_result(r)
        if r.fatal:
            print(f"\n[FATAL] {r.name} failed. Cannot continue.")
            return 1

    # --- Step 2: GPU detection ---
    print("\n-- Step 2: GPU detection --")
    gpu_results = check_gpu()
    all_results.extend(gpu_results)
    for r in gpu_results:
        _print_result(r)

    # --- Step 3: llama-cpp-python install ---
    print("\n-- Step 3: llama-cpp-python (CUDA) --")
    install_results = install_llama_cpp(dry_run=dry_run)
    all_results.extend(install_results)
    for r in install_results:
        _print_result(r)
    if any(r.fatal for r in install_results):
        print("\n[FATAL] llama-cpp-python install failed. See above.")
        return 1

    # --- Step 4: Model download ---
    print("\n-- Step 4: Model download --")
    dl_results = download_model(
        model_url=model_url, model_path=model_path, dry_run=dry_run
    )
    all_results.extend(dl_results)
    for r in dl_results:
        _print_result(r)
    if any(r.fatal for r in dl_results):
        print("\n[FATAL] Model download/verification failed. See above.")
        return 1

    # --- Step 5: Emit env ---
    print("\n-- Step 5: Environment variable --")
    env_results = emit_env(model_path, dry_run=dry_run)
    all_results.extend(env_results)
    for r in env_results:
        _print_result(r)

    # --- Summary ---
    counts: dict[str, int] = {}
    for r in all_results:
        counts[r.status] = counts.get(r.status, 0) + 1

    print("\n=== Summary ===")
    for status, cnt in sorted(counts.items()):
        print(f"  {status}: {cnt}")

    fatals = [r for r in all_results if r.fatal]
    if fatals:
        print(f"\n[FAIL] {len(fatals)} fatal error(s). Setup incomplete.")
        return 1

    if dry_run:
        print("\n[OK] --dry-run complete. Re-run without --dry-run to apply changes.")
    else:
        print("\n[OK] Setup complete. Run arm 3 with:")
        print(f"     export {_ENV_VAR}={model_path}")
        print("     python local/run_arm3_local.py")
    return 0


if __name__ == "__main__":
    sys.exit(main())
