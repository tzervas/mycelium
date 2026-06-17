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

import json
import shutil
import subprocess
from collections.abc import Callable, Sequence
from dataclasses import dataclass, field

from mycelium_experiments.kc2.tasks import Task

# A backend is anything that turns a full prompt string into the model's raw text.
Backend = Callable[[str], str]


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
    extra_args: Sequence[str] | None = None,
    timeout: int = 180,
) -> Backend:
    """A backend that shells out to `llama` / `llama-cli`.

    NOTE (verify on the target build): recent llama.cpp defaults to interactive
    *conversation* mode and may echo the prompt back, both of which corrupt one-shot
    code generation. If you see the prompt echoed into the source, pass the build's
    flags via ``extra_args`` (commonly ``-no-cnv`` and/or ``--no-display-prompt``)
    after confirming them in ``llama --help`` — or prefer ``server_backend`` (clean
    /completion output). We do not hard-code those flags (availability varies).
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
            "--log-disable",
            "-e",
            *(extra_args or []),
        ]
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
        if proc.returncode != 0:
            msg = f"llama exited {proc.returncode}: {proc.stderr.strip()[:800]}"
            raise RuntimeError(msg)
        return proc.stdout

    return complete


def server_backend(
    base_url: str,
    *,
    seed: int = 42,
    n_predict: int = 256,
    timeout: int = 180,
) -> Backend:
    """A backend that POSTs to a llama.cpp HTTP server's /completion endpoint."""
    import urllib.request

    url = base_url.rstrip("/") + "/completion"

    def complete(prompt: str) -> str:
        payload = json.dumps(
            {"prompt": prompt, "seed": seed, "n_predict": n_predict, "stream": False}
        ).encode()
        req = urllib.request.Request(
            url, data=payload, headers={"Content-Type": "application/json"}, method="POST"
        )
        with urllib.request.urlopen(req, timeout=timeout) as resp:  # noqa: S310 — local server
            body = resp.read().decode()
        return str(json.loads(body).get("content", ""))

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
        return extract_source(self.backend(prompt), arm)


def resolve_llama_cli(explicit: str | None = None) -> str | None:
    """Best-effort `llama` / `llama-cli` resolution (PATH only — the doctor heals PATH)."""
    if explicit:
        return explicit
    for name in ("llama-cli", "llama"):
        found = shutil.which(name)
        if found:
            return found
    return None
