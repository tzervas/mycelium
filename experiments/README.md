# Mycelium experiments — run order & runbook

UV-managed Python (ADR-007; Python ≥3.13). Home of the **KC-2 LLM-leverage experiment**
(M-002), the VSA compositional-bounds sweep, and other measurement code. Not the kernel
(that's Rust).

## Contents

- [Current status: KC-2 already has a verdict](#current-status-kc-2-already-has-a-verdict)
- [Layout](#layout)
- [Run order](#run-order)
- Detail doc: [`KC2-RUNBOOK.md`](KC2-RUNBOOK.md) — steps 1–4 in full (harness validation, unit
  tests, the real KC-2 run + all its knobs, capturing results)
- Related: [`docker/README.md`](docker/README.md) (desktop-GPU containerised run) ·
  [`mycelium_experiments/vsa_bounds/README.md`](mycelium_experiments/vsa_bounds/README.md)
  (M-832 VSA compositional-bounds sweep) ·
  [`surface-fragment/README.md`](surface-fragment/README.md) (the throwaway grammar KC-2 generates)

## Current status: KC-2 already has a verdict

**KC-2 is a satisfied kill-criterion experiment, not an ongoing gate.** It already returned a
decisive verdict — **proceed** ([`DN-09-KC-2-Verdict.md`](../docs/notes/DN-09-KC-2-Verdict.md),
2026-06-18): the measured
LLM leverage on the Mycelium surface is weak but recoverable, not the irrecoverable collapse the
criterion guards against. That closed the standing KC-2 gate and selected the L3 strategy
(committed text syntax + a co-equal structured-projection layer).

Consequences for this directory, honestly stated (VR-5 — don't overstate what's still open):

- `experiments/tests/test_kc2.py` is **skipped by default** (`pytest.mark.skipif`) — it needs a
  local llama.cpp backend that is not a 1.0.0 build dependency. Opt in explicitly with
  `MYC_RUN_KC2=1` if you want to re-exercise it.
- The runbook below (and [`KC2-RUNBOOK.md`](KC2-RUNBOOK.md)) still works end-to-end and remains
  useful for **reproducing** the measurement or extending it (new tasks, new models, the
  retention-ratio ablation in `tools/llm-harness/GROK-HARNESS.md`) — it is not required for the
  1.0.0 release path.
- The **retention-ratio ablation** (M-381, `tools/llm-harness`'s `grok/` package) is a distinct,
  still-open follow-up track — see `tools/llm-harness/GROK-HARNESS.md`.

## Layout

| Path | What it is |
|---|---|
| `mycelium_experiments/kc2/` | the KC-2 harness (generator, arms, scoring, reporting) |
| `mycelium_experiments/vsa_bounds/` | the M-832 VSA compositional-bounds sweep — see its own [README](mycelium_experiments/vsa_bounds/README.md) |
| `surface-fragment/` | the throwaway grammar (M-020) KC-2 generates against — see its own [README](surface-fragment/README.md) |
| `docker/` | containerised desktop-GPU run of the KC-2 matrix — see its own [README](docker/README.md) |
| `primers/` | the per-arm generator primers (`mycelium-minimal.txt`, `mycelium-examples.txt`) |
| `results/` | run outputs (JSON/summary/log); gitignored except committed reference reports |
| `tests/` | pytest unit tests (`test_kc2.py`, `test_vsa_bounds.py`, `test_smoke.py`) |
| `run-kc2-matrix.sh` | unattended {model}×{primer} matrix runner |

## Run order

This is the **optimal order** to run the local-LLM tests + experiments on a device
(Termux/Android included), capture results, and commit them. Each step is honest:
a missing tool/model is an explicit **SKIP**, never a false pass (G2/VR-5).

### 0. Prerequisites — heal the environment once

The `tools/llm-harness` doctor installs/links what's needed and prints a bottom-line
**READY / NOT READY** verdict:

```sh
python tools/llm-harness/harness.py --doctor          # install + fix PATH (prompts; --yes to skip)
python tools/llm-harness/harness.py --doctor --check-only   # read-only report (safe on a phone)
```

You want it to end with `✓ READY — llama.cpp CLI + a local model are both present.`
If not, the verdict lists exactly what's missing and the one command to fix it.

### 1–4. Validate, test, run, capture

See [`KC2-RUNBOOK.md`](KC2-RUNBOOK.md) for the full commands:

1. **Validate the model + honesty gates** (`tools/llm-harness`) — determinism, JSON projection,
   the VR-5 guarantee-tag gate, latency.
2. **Experiment unit tests** (no model needed) — the KC-2 harness logic against fixtures.
3. **The KC-2 experiment, for real** (M-002) — two arms, syntactic validity + pass rate +
   edit-to-fix iterations, plus the primer/model matrix and the desktop-GPU containerised path.
4. **Capture results** — commit the JSON report + summary + harness reports to your branch.

## Third-party license note: the optional `gpu` dependency-group

**Disclosure (never-silent, G2) — not part of the Rust `THIRD-PARTY-LICENSES.md` scope.** The
Rust kernel's dependency tree is 100% permissive (MIT/Apache-2.0/BSD/ISC/Unicode/Unlicense/CC0/
MPL-file-level — `cargo deny check licenses` is green, see `THIRD-PARTY-LICENSES.md` at the repo
root). This directory's optional Python `gpu` dependency-group (`[dependency-groups].gpu` in
`experiments/pyproject.toml` — `torch` and `matplotlib`, opted into via `uv sync --group gpu`) is a
different case: on Linux/Windows, `torch`'s CUDA build transitively pulls a set of
**NVIDIA-proprietary** CUDA runtime packages (`nvidia-cublas`, `nvidia-cuda-runtime`,
`nvidia-cuda-cupti`, `nvidia-cuda-nvrtc`, `nvidia-cufft`, `nvidia-cusolver`, `nvidia-cusparse`,
`nvidia-nvjitlink`, and related `nvidia-*` wheels — see the pinned versions in
`experiments/uv.lock`), distributed under **NVIDIA's own EULA** — **not** an OSI-approved
open-source license.

Scope of the exposure, stated plainly:

- **Opt-in only.** The base `experiments` install (`uv sync`, `uv sync --group dev`) never touches
  these packages — they land only if you explicitly run `uv sync --group gpu`.
- **Experiments-only.** This dependency-group exists solely for the `vsa_bounds` GPU-accelerated
  sweep (M-832) and similar local measurement harnesses under `experiments/`. It is not imported
  by, linked against, or shipped with any Mycelium kernel/spore artifact — the `mycelium-*` Rust
  crates and the `.myc` stdlib have no dependency on it.
- **Not redistributed.** Nothing under this dependency-group is bundled into a Mycelium release
  artifact; a contributor who opts in is installing NVIDIA's own binaries locally, under NVIDIA's
  own terms, for their own experiment run.

If you opt into `--group gpu`, you accept NVIDIA's EULA for those packages directly (`uv`/`pip`
print the package source at install time; NVIDIA publishes its CUDA EULA alongside each package).
This note exists so that opt-in is an *informed* one, not a silent one.

---
[Back to `tools/llm-harness/README.md`](../tools/llm-harness/README.md)
