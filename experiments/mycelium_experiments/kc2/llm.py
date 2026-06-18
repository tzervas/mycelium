"""Local-LLM generators for the KC-2 experiment (M-002).

The KC-2 harness (``harness.py``) is generator-agnostic: it measures whatever
:class:`~mycelium_experiments.kc2.harness.Generator` it is handed. Historically the
only documented blocker on *running* M-002 was "needs LLM API access". This module
removes that blocker for the **local** case: a generator backed by a llama.cpp model
on the device (the same model the ``tools/llm-harness`` doctor installs/fetches), via
either the ``llama`` / ``llama-cli`` binary or a llama.cpp HTTP server.

Honesty posture (house rules):
- **NEVER-SILENT (G2):** a backend failure raises — it is never a silently empty
  generation that the checker would score as a (failing) attempt. A missing
  binary/model is an explicit error at construction, not a fake result.
- **VR-5:** this module only *produces candidate source*. It establishes **no**
  KC-2 verdict; the verdict is a maintainer-written analysis of a real run.
- **Generator configuration, not task content:** the language/DSL **primer** below
  is configuration (per ``tasks.py``). It is a *generic* syntax cheatsheet for each
  arm — it deliberately does **not** contain any task's answer — and it is the chief
  knob a maintainer tunes. Both arms get a primer of comparable generosity so the
  comparison measures the *language*, not primer effort. Override either with a file
  (``--primer-mycelium`` / ``--primer-baseline``) and record which primer a run used.

Pure stdlib (the experiments project has no runtime dependencies).
"""

from __future__ import annotations

import ctypes
import gc
import json
import logging
import os
import shutil
import subprocess
import sys
from collections.abc import Callable, Sequence
from dataclasses import dataclass, field
from pathlib import Path

from mycelium_experiments.kc2.tasks import Task

# A backend is anything that turns a full prompt string into the model's raw text.
Backend = Callable[[str], str]

# Conservative upper bound on KV bytes/token for a phone-class q4 model (≈32 KB);
# over-estimating makes the auto-sizer under-allocate context, which is the safe error.
_KV_MB_PER_TOKEN = 0.032


def detect_memory() -> dict[str, int | None]:
    """Available/total/swap RAM in MB from /proc/meminfo (stdlib; None if unknown)."""
    out: dict[str, int | None] = {
        "mem_total_mb": None,
        "mem_available_mb": None,
        "swap_free_mb": None,
    }
    try:
        text = Path("/proc/meminfo").read_text(encoding="utf-8")
    except OSError:
        return out
    kv: dict[str, int] = {}
    for line in text.splitlines():
        name, _, rest = line.partition(":")
        fields = rest.strip().split()
        if fields and fields[0].isdigit():
            kv[name.strip()] = int(fields[0])  # kB
    if "MemTotal" in kv:
        out["mem_total_mb"] = kv["MemTotal"] // 1024
    avail = kv.get("MemAvailable")
    if avail is None and "MemFree" in kv:
        avail = kv.get("MemFree", 0) + kv.get("Buffers", 0) + kv.get("Cached", 0)
    out["mem_available_mb"] = (avail // 1024) if avail is not None else None
    if "SwapFree" in kv:
        out["swap_free_mb"] = kv["SwapFree"] // 1024
    return out


def reclaim_memory(log: logging.Logger) -> dict[str, int | None]:
    """Gently free RAM before a run so more is available for the model + KV cache.

    Non-destructive only — it never kills processes (reaping orphan llama-servers, the
    bigger lever on a phone, stays the explicit `--stop-server`):
      - ``gc.collect()`` — release this process's own garbage;
      - ``malloc_trim(0)`` — return free()'d heap to the OS (glibc; bionic may lack it);
      - ``sync`` — flush dirty pages so the kernel can reclaim them;
      - drop reclaimable page cache — **root only**; on an unrooted phone this is not
        permitted and is skipped (logged, never-silent). The kernel already counts
        reclaimable cache in MemAvailable, so the unrooted gain here is modest.

    Returns {before_mb, after_mb, freed_mb}. Run before context sizing so the freed
    memory is reflected in `auto_ctx_size`.
    """
    before = detect_memory().get("mem_available_mb")
    gc.collect()
    # malloc_trim is glibc-only; CDLL(None) means "the main program" on POSIX but raises
    # on Windows. Skip it off POSIX rather than rely on catching a platform-specific error.
    if os.name == "posix":
        try:
            libc = ctypes.CDLL(None)
            if hasattr(libc, "malloc_trim"):
                libc.malloc_trim(0)
        except (OSError, AttributeError, ValueError):
            pass
    try:
        subprocess.run(["sync"], timeout=30, check=False)  # noqa: S603,S607 — flush dirty pages
    except (OSError, subprocess.SubprocessError):
        pass
    try:
        with open("/proc/sys/vm/drop_caches", "w", encoding="ascii") as f:
            f.write("1\n")
        log.info("reclaim: dropped reclaimable page cache")
    except OSError:
        log.debug("reclaim: drop_caches not permitted (unrooted) — skipped")
    after = detect_memory().get("mem_available_mb")
    freed = (after - before) if isinstance(before, int) and isinstance(after, int) else None
    log.info(
        "reclaim: available RAM %s → %s MB%s",
        before,
        after,
        f"  (+{freed} MB)" if isinstance(freed, int) and freed > 0 else "",
    )
    return {"before_mb": before, "after_mb": after, "freed_mb": freed}


def auto_ctx_size(
    want: int,
    model: str | None,
    mem: dict[str, int | None],
    *,
    swap_fraction: float = 0.0,
) -> tuple[int, str]:
    """Pick ctx = min(workload need, what available memory safely holds). Returns (ctx, reason).

    By default only RAM is budgeted; ``swap_fraction > 0`` (the --use-swap flag) adds that
    fraction of free swap (slower — KV thrashes if it pages out). Conservative when memory
    is unknown. Always overridable with --ctx-size.
    """
    avail = mem.get("mem_available_mb")
    swap_free = mem.get("swap_free_mb")
    model_mb = 0
    try:
        if model and Path(model).is_file():
            model_mb = Path(model).stat().st_size // (1024 * 1024)
    except OSError:
        pass
    if not isinstance(avail, int) or avail <= 0:
        chosen = min(want, 1024)
        return chosen, f"available RAM unknown — conservative ctx={chosen}"
    reserve = max(768, model_mb)  # OS + worst-case resident weights
    swap_budget = (
        int(swap_free * swap_fraction)
        if (swap_fraction > 0 and isinstance(swap_free, int) and swap_free > 0)
        else 0
    )
    usable = (avail - reserve) + swap_budget
    cap = int((usable * 0.40) / _KV_MB_PER_TOKEN) if usable > 0 else 256
    cap = max(256, (cap // 256) * 256)
    chosen = max(256, min(want, cap))
    swap_note = f" +{swap_budget}MB swap" if swap_budget else ""
    reason = (
        f"avail={avail}MB{swap_note} model={model_mb}MB reserve={reserve}MB "
        f"usable={max(0, usable)}MB → memory allows ~{cap}, want {want} ⇒ ctx={chosen}"
    )
    return chosen, reason


def detect_gpu() -> list[dict[str, object]]:
    """Best-effort GPU enumeration (NVIDIA/ROCm/Metal). Empty on a phone; never raises."""
    gpus: list[dict[str, object]] = []
    smi = shutil.which("nvidia-smi")
    if smi:
        try:
            res = subprocess.run(
                [smi, "--query-gpu=name,memory.free", "--format=csv,noheader,nounits"],
                capture_output=True,
                text=True,
                timeout=15,
            )
            if res.returncode == 0:
                for line in res.stdout.strip().splitlines():
                    parts = [p.strip() for p in line.split(",")]
                    if len(parts) >= 2:
                        gpus.append(
                            {
                                "name": parts[0],
                                "backend": "cuda",
                                "vram_free_mb": int(parts[1]) if parts[1].isdigit() else None,
                            }
                        )
        except (OSError, subprocess.SubprocessError):
            pass
    if not gpus and shutil.which("rocm-smi"):
        gpus.append({"name": "AMD ROCm GPU", "backend": "rocm", "vram_free_mb": None})
    if not gpus and sys.platform == "darwin":
        gpus.append({"name": "Apple GPU (Metal)", "backend": "metal", "vram_free_mb": None})
    return gpus


def auto_gpu_layers(gpus: list[dict[str, object]], model: str | None) -> tuple[int, str]:
    """Choose -ngl: full offload when VRAM holds the model (or is unknown), else CPU.

    Returns (ngl, reason). A phone build has no GPUs ⇒ (0, ...), a no-op there.
    """
    if not gpus:
        return 0, "no GPU detected — CPU only"
    g = gpus[0]
    model_mb = 0
    try:
        if model and Path(model).is_file():
            model_mb = Path(model).stat().st_size // (1024 * 1024)
    except OSError:
        pass
    free = g.get("vram_free_mb")
    if not isinstance(free, int) or model_mb == 0 or free >= int(model_mb * 1.15):
        return 999, f"{g['name']} ({g['backend']}) — offloading all layers (-ngl 999)"
    return 0, f"{g['name']} — {free}MB free < model {model_mb}MB; CPU (set --n-gpu-layers N)"


# ---------------------------------------------------------------------------
# Default primers (generator configuration — tune, then record which was used)
# ---------------------------------------------------------------------------
# Generic syntax cheatsheets only. NO task answers appear here (that would bias
# the measurement). Keep the two arms comparably generous.

PRIMER_MYCELIUM = """\
You write programs in the Mycelium surface fragment. Reply with ONLY the program
source — no prose, no markdown fences, no explanation.

Syntax (generic examples, NOT answers):
  nodule bench                         # every program starts with a nodule line
  fn name(x: Binary{8}) -> Binary{8} = not(x)
  fn main() -> Ternary{4} = add(<00+->, <0+0->)
- Types: Binary{N} (N bits), Ternary{N} (N balanced trits).
- Binary literal: 0b1010_1010 (underscores allowed). Ternary literal: <+0-> (MSB first).
- Built-in ops: not(x), xor(a, b), add(a, b), swap(x, to: Ternary{6}, policy: name).
- swap ALWAYS needs a `to:` type and a `policy:` name; there is no implicit conversion.
- Sum types + match:
    type Sign = Neg | Zero | Pos
    fn label(s: Sign) -> Ternary{1} = match s { Neg => <->, Zero => <0>, _ => <+> }
- Recursive list type + recursion:
    type Bytes = End | More(Binary{8}, Bytes)
- Bounded iteration (fold):  for b in bs, acc = 0b0000_0000 => xor(acc, b)
- let-binding:  let bs = More(0b1111_0000, End) in for b in bs, acc = ... => ...
- `matured fn main() -> ...` marks a promoted stable component.
"""

PRIMER_BASELINE = """\
You write programs in a small Python-embedded DSL. Reply with ONLY the program
source — no prose, no markdown fences, no explanation. Define a function `main()`.

The DSL (already imported for you — do NOT re-import):
  Bin("1010_1010")   # width-checked two's-complement binary word, MSB first
  Tern("+0-")        # width-checked balanced-ternary word, MSB first
  bnot(x)            # bitwise complement of a Bin
  xor(a, b)          # bitwise xor of two equal-width Bins
  tadd(a, b)         # balanced-ternary add of two equal-width Terns
  swap(value, to=("bin", 8), policy="name")   # to=("bin"|"tern", width); policy is mandatory

Generic examples (NOT answers):
  def main():
      return bnot(Bin("0000_1111"))
  def flip(x):
      return bnot(x)
  def main():
      return swap(Bin("1011_0010"), to=("tern", 6), policy="rt")
"""


def primer_for(arm: str) -> str:
    """The default primer for an arm ("mycelium" | "baseline")."""
    if arm == "mycelium":
        return PRIMER_MYCELIUM
    if arm == "baseline":
        return PRIMER_BASELINE
    msg = f"unknown arm {arm!r} (expected 'mycelium' or 'baseline')"
    raise ValueError(msg)


# ---------------------------------------------------------------------------
# Prompt assembly + source extraction
# ---------------------------------------------------------------------------


def build_prompt(task: Task, arm: str, feedback: Sequence[str], primer: str) -> str:
    """Assemble the full generation prompt: primer + task + any edit-to-fix feedback."""
    parts = [primer.rstrip(), "", "TASK:", task.prompt.strip()]
    if feedback:
        parts += ["", "Your previous attempt failed its checker with this diagnostic:"]
        parts += [f"  {fb.strip()}" for fb in feedback]
        parts += ["Fix it. Reply with ONLY the corrected program source."]
    else:
        parts += ["", "Reply with ONLY the program source."]
    return "\n".join(parts) + "\n"


def extract_source(raw: str, arm: str) -> str:
    """Pull the program source out of a model's raw text.

    Models often wrap code in markdown fences or add prose despite instructions; we
    take the first fenced block when present, else the stripped text. This is a
    best-effort *projection*, not a guarantee — a checker still judges the result, so
    a bad extraction surfaces as an honest failed attempt, never a false pass.
    """
    text = raw.strip()
    if "```" in text:
        segments = text.split("```")
        if len(segments) >= 2:
            block = segments[1]
            block_lines = block.splitlines()
            # Drop a leading language tag line (```mycelium / ```python / ```).
            if block_lines and block_lines[0].strip().lower() in {
                "mycelium",
                "myc",
                "python",
                "py",
                "",
            }:
                block_lines = block_lines[1:]
            extracted = "\n".join(block_lines).strip()
            if extracted:
                return extracted + "\n"
    return text + "\n"


# ---------------------------------------------------------------------------
# Backends (CLI subprocess / HTTP server) — minimal, self-contained
# ---------------------------------------------------------------------------


def cli_backend(
    llama_cli: str,
    model: str,
    *,
    seed: int = 42,
    n_predict: int = 256,
    ctx_size: int = 2048,
    n_gpu_layers: int = 0,
    extra_args: Sequence[str] | None = None,
    timeout: int = 300,
) -> Backend:
    """A backend that shells out to `llama` / `llama-cli`.

    ``ctx_size`` caps the context window (``-c``): keep it small so llama.cpp does not
    allocate a KV cache for the model's full trained window (Qwen2.5 = 32k), which
    OOM-kills the process (SIGKILL/9) on a phone.

    One-shot generation is enforced with ``-no-cnv`` + ``--no-display-prompt`` (verified
    on-device): without them recent llama-cli enters its interactive conversation REPL —
    it generates, then waits at a `>` prompt until the subprocess TIMES OUT, and echoes
    the prompt into stdout. Remove via ``extra_args`` only if a build rejects them, or
    use ``server_backend`` (clean /completion output).
    """

    def complete(prompt: str) -> str:
        cmd = [
            llama_cli,
            "--model",
            model,
            "--prompt",
            prompt,
            "--seed",
            str(seed),
            "--n-predict",
            str(n_predict),
            "--ctx-size",
            str(ctx_size),
            "--log-disable",
            "-e",
            "-no-cnv",  # one-shot: don't enter the interactive chat REPL (it never exits)
            "--no-display-prompt",  # stdout = completion only, not the echoed prompt
            *(["--n-gpu-layers", str(n_gpu_layers)] if n_gpu_layers > 0 else []),
            *(extra_args or []),
        ]
        # stdin=DEVNULL is the build-agnostic safety net: some llama-cli builds ignore
        # -no-cnv and still enter the interactive REPL. Feeding EOF makes that REPL exit
        # after the first response instead of hanging on the terminal (no Ctrl+C needed).
        proc = subprocess.run(
            cmd, capture_output=True, text=True, timeout=timeout, stdin=subprocess.DEVNULL
        )
        if proc.returncode != 0:
            msg = f"llama exited {proc.returncode}: {proc.stderr.strip()[:800]}"
            raise RuntimeError(msg)
        return proc.stdout

    return complete


def server_backend(
    base_url: str,
    *,
    seed: int = 42,
    n_predict: int = 192,
    timeout: int = 600,
    stop: Sequence[str] | None = ("\nTASK:",),
) -> Backend:
    """A backend that POSTs to a llama.cpp HTTP server's /completion endpoint.

    ``timeout`` is the client read budget for ONE generation. A phone CPU decodes a 1.5B
    model at ~0.3–0.7 tok/s, so ``n_predict`` tokens can take minutes — keep the timeout
    generous (default 600 s) or generations die mid-stream and abort the run. ``stop``
    halts the model early on an obvious boundary (a fabricated next ``TASK:``), and
    ``cache_prompt`` reuses the shared-primer KV across calls (the big common prefix).
    """
    import urllib.error
    import urllib.request

    url = base_url.rstrip("/") + "/completion"

    def complete(prompt: str) -> str:
        body = {
            "prompt": prompt,
            "seed": seed,
            "n_predict": n_predict,
            "stream": False,
            "cache_prompt": True,
        }
        if stop:
            body["stop"] = list(stop)
        payload = json.dumps(body).encode()
        req = urllib.request.Request(
            url, data=payload, headers={"Content-Type": "application/json"}, method="POST"
        )
        try:
            with urllib.request.urlopen(req, timeout=timeout) as resp:  # noqa: S310 — local
                raw = resp.read().decode()
        except (TimeoutError, urllib.error.URLError, OSError) as exc:
            # Abort loudly (G2): never a silent empty generation. The runner checkpoints
            # the attempts collected so far before this propagates, so no data is lost.
            msg = (
                f"server /completion failed after {timeout}s ({type(exc).__name__}: {exc}). "
                f"On a slow phone raise --timeout or lower --n-predict (now {n_predict})."
            )
            raise RuntimeError(msg) from exc
        return str(json.loads(raw).get("content", ""))

    return complete


# ---------------------------------------------------------------------------
# The generator (implements the harness Generator protocol)
# ---------------------------------------------------------------------------


@dataclass
class LlamaGenerator:
    """A KC-2 generator backed by a local llama.cpp model.

    ``backend`` turns a prompt into raw text; ``primers`` maps an arm to its primer
    (defaults to the module primers). Implements ``__call__(task, arm, feedback)``.
    """

    backend: Backend
    primers: dict[str, str] = field(default_factory=dict)
    calls: list[tuple[str, str, int]] = field(default_factory=list)

    def __call__(self, task: Task, arm: str, feedback: Sequence[str]) -> str:
        primer = self.primers.get(arm) or primer_for(arm)
        prompt = build_prompt(task, arm, feedback, primer)
        self.calls.append((task.id, arm, len(feedback)))
        raw = self.backend(prompt)
        # Some builds ignore --no-display-prompt and echo the prompt back into stdout;
        # if the verbatim prompt is present, keep only what follows it (the completion).
        if prompt and prompt in raw:
            raw = raw.split(prompt, 1)[1]
        return extract_source(raw, arm)


def resolve_llama_cli(explicit: str | None = None) -> str | None:
    """Best-effort `llama` / `llama-cli` resolution (PATH only — the doctor heals PATH)."""
    if explicit:
        return explicit
    for name in ("llama-cli", "llama"):
        found = shutil.which(name)
        if found:
            return found
    return None
