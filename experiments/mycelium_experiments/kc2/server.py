"""Auto-managed llama.cpp HTTP server for non-interactive KC-2 runs.

The CLI backend is unreliable on some builds (it enters an interactive REPL that
ignores `-no-cnv`). The server's `/completion` endpoint is a clean, one-shot path
with the model loaded ONCE — far faster for a multi-task/multi-seed sweep.

This module makes the server *scripted* instead of a manual `llama-server … &`:
- **reuse** a server that is already healthy at the requested URL (never double-bind);
- otherwise pick a **free port** (the manual default 8080 collides with a lingering
  server — observed on-device: "couldn't bind HTTP server socket … port: 8080");
- wait for `/health`, returning the URL + the process we launched;
- tear down **only** what we launched (no orphans).

Never-silent (G2): a missing binary, an early exit (port in use), or a server that
never becomes ready is an explicit error — never a silent fallback.
"""

from __future__ import annotations

import logging
import shutil
import socket
import subprocess
import time
import urllib.error
import urllib.request
from pathlib import Path


def find_llama_server(explicit: str | None = None) -> str | None:
    """Resolve the `llama-server` binary (explicit path/name → PATH)."""
    if explicit:
        if Path(explicit).is_file():
            return explicit
        return shutil.which(explicit)
    for name in ("llama-server", "llama-cpp-server"):
        found = shutil.which(name)
        if found:
            return found
    return None


def server_healthy(base_url: str, timeout: float = 2.0) -> bool:
    """True iff GET <base_url>/health returns 200 (the server is up and a model is loaded)."""
    url = base_url.rstrip("/") + "/health"
    try:
        with urllib.request.urlopen(url, timeout=timeout) as resp:  # noqa: S310 — local
            return int(getattr(resp, "status", resp.getcode())) == 200
    except (urllib.error.URLError, OSError, ValueError):
        return False


def _free_port(host: str = "127.0.0.1") -> int:
    """Ask the OS for an unused TCP port (bind to :0, read it back, release)."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind((host, 0))
        return int(s.getsockname()[1])


def ensure_server(
    *,
    model: str,
    ctx_size: int,
    log: logging.Logger,
    n_gpu_layers: int = 0,
    host: str = "127.0.0.1",
    port: int | None = None,
    binary: str | None = None,
    ready_timeout: int = 300,
    log_path: Path | None = None,
) -> tuple[str, subprocess.Popen[bytes] | None]:
    """Return (base_url, proc). proc is None when an existing server was REUSED.

    If ``port`` is given and a server is already healthy there, reuse it (proc=None).
    Otherwise launch `llama-server` on a free port (or the requested one), redirect its
    chatty logs to ``log_path``, and poll `/health` until ready. Raises on any failure.
    """
    if port is not None:
        url = f"http://{host}:{port}"
        if server_healthy(url):
            log.info("Reusing healthy llama-server at %s", url)
            return url, None

    binp = find_llama_server(binary)
    if not binp:
        msg = (
            "llama-server not found (Termux: `pkg install llama-cpp` ships it next to "
            "`llama`). Pass --server-binary PATH, or --server URL to use a running one."
        )
        raise RuntimeError(msg)

    use_port = port if port is not None else _free_port(host)
    url = f"http://{host}:{use_port}"
    cmd = [binp, "-m", model, "-c", str(ctx_size), "--host", host, "--port", str(use_port)]
    if n_gpu_layers > 0:
        cmd += ["--n-gpu-layers", str(n_gpu_layers)]
    log.info("Launching llama-server: %s", " ".join(cmd))

    sink = open(log_path, "ab") if log_path else subprocess.DEVNULL  # noqa: SIM115
    try:
        proc = subprocess.Popen(cmd, stdout=sink, stderr=sink, stdin=subprocess.DEVNULL)
    finally:
        if log_path is not None and sink is not subprocess.DEVNULL:
            sink.close()  # the child holds its own dup'd fd

    t0 = time.monotonic()
    while time.monotonic() - t0 < ready_timeout:
        if proc.poll() is not None:
            where = f" (server log: {log_path})" if log_path else ""
            msg = (
                f"llama-server exited early (code {proc.returncode}) before becoming "
                f"ready — port {use_port} already in use, or a bad model/flag?{where}"
            )
            raise RuntimeError(msg)
        if server_healthy(url, timeout=2.0):
            log.info("llama-server ready at %s (%.1fs)", url, time.monotonic() - t0)
            return url, proc
        time.sleep(1.0)

    stop_server(proc, log)
    msg = f"llama-server at {url} did not become ready within {ready_timeout}s"
    raise RuntimeError(msg)


def stop_server(proc: subprocess.Popen[bytes] | None, log: logging.Logger) -> None:
    """Terminate a server we launched (no-op for a reused/None proc). Best-effort, never raises."""
    if proc is None:
        return
    log.info("Stopping llama-server (pid %s)", proc.pid)
    try:
        proc.terminate()
        proc.wait(timeout=15)
    except (subprocess.TimeoutExpired, OSError):
        try:
            proc.kill()
        except OSError:
            pass
