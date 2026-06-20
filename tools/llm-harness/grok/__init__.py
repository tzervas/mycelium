"""Grok / xAI co-authoring + ablation harness for Mycelium (M-330, M-331, M-381).

This package adds xAI/Grok backends to the local-llama harness next door
(``coauthor.py``/``harness.py``) without replacing it. It is **pure Python +
optional ``xai_sdk``**, runnable from a WSL host with nothing but ``uv``.

HONESTY (house rule 1; VR-5): no API key exists in the build/test environment,
so live KC-2/SC-5b/retention numbers cannot be produced here. The package ships
a deterministic, network-free ``--self-test`` (the green gate) that exercises
model ordering, RPM/TPM pacing math, batch-vs-live cost accounting, scoring and
report emission against a *mocked* client. Plumbing is **Empirical** (self-test
verified); the leverage/quality *verdict* stays **open / Declared (pending run)**
until a human runs the live experiment.

Modules:
    models      — load ``models.toml``; cheapest-first ordering; USD cost math.
    ratelimit   — per-model RPM+TPM pacer with exponential backoff on 429.
    client      — pluggable chat clients (OpenAI-compatible REST, xai_sdk batch,
                  deterministic Mock).
    scoring     — score generated Mycelium source via ``myc-check`` (syntactic vs
                  type-check) and the LSP diagnostics shim.
    coauthor_loop — the M-330 generate -> feedback -> fix loop on a client+scorer.
    ablation    — the M-381 / research/11 T11.7 retention-ratio ablation runner.
    report      — per-model JSON + cross-model markdown comparison emission.
    selftest    — the offline deterministic self-test (green gate).
    cli         — the command-line driver (``--mode live|batch``, ``--ablation``,
                  ``--self-test``).
"""

from __future__ import annotations

__all__ = ["__version__"]

__version__ = "0.1.0"
