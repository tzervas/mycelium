# VSA desktop heavy-check results — M-832 / OQ-F

This directory collects the outputs of the **VSA heavy checks** that are held out of the
cloud-session `just check` gate and run on the maintainer's (GPU) desktop instead
(CLAUDE.md §Local checks → "Heavy checks run on the desktop"). It is the push-back target for
`scripts/vsa-desktop-checks.sh`.

## How to produce the contents

On the desktop, from the repo root:

```bash
bash scripts/vsa-desktop-checks.sh          # full bundle (crate durability + GPU experiment + proofs + mutants)
# then push the results back:
git add experiments/results/vsa-m832 && git commit -m "vsa: desktop heavy-check results" && git push

# FOLLOW-UP: run ONLY the #[ignore] heavy instruments (resonator_capacity_sweep, resonator_cleanup_ablation,
# …) that the default run skips, into a separate log, then push to supplement — stages 2-4 are skipped:
VSA_IGNORED_ONLY=1 bash scripts/vsa-desktop-checks.sh
git add experiments/results/vsa-m832 && git commit -m "vsa: ignored heavy instruments" && git push
```

Prerequisites (each stage skips gracefully if absent): `cargo`, `uv` + a CUDA `torch`
(`cd experiments && uv sync --group gpu`; `bash experiments/docker/gpu-setup.sh` to check GPU),
`z3`, `cabal` (LiquidHaskell), `lean`/`lake`, `cargo-mutants`.

## What lands here

| File / dir | Stage | Guarantee |
|---|---|---|
| `vsa-crate-tests.log` | `mycelium-vsa` + `mycelium-std-vsa` full-tier tests (HIGH proptest, **excludes** `#[ignore]`) | Empirical |
| `vsa-crate-tests-ignored.log` | the `#[ignore]` heavy instruments only (`VSA_IGNORED_ONLY=1`, `--ignored --nocapture`) — the supplemental follow-up run | Empirical |
| `m832-sweep-gpu.log` / `m832-sweep-cpu.log` | the M-832 multi-hop sweep (GPU, or CPU numpy fallback) | Empirical |
| `m832-proof-emit.log`, `obligations/` | emitted `PROOF-SUMMARY.md` + `.smt2`/`.hs`/`.lean` proof obligations | **Declared** (candidate) |
| `proof-z3.log` | z3 discharge of the concrete arithmetic in the `.smt2` obligations | Empirical (per-obligation solver verdict) |
| `proof-lh.log`, `proof-lean.log` | LiquidHaskell / Lean skeleton builds | Declared until discharged |
| `vsa-mutants.log` | cargo-mutants on the VSA crates | Empirical |

## Honesty contract (VR-5 / G2)

The experiment's rates are **Empirical** (trial-measured). The emitted proof obligations are
**Declared** — candidate theorems, not proofs; the `Proven` tag requires a proof assistant to
discharge the obligations **and** the underlying theorem to be formally established or cited
(ADR-032 / ADR-010). This directory **collects evidence; it does not grade it** — the verdict on
which multi-hop subsets are `Proven` vs `Empirical` is the maintainer's analysis (RFC-0003 §5,
the OQ-F open question), recorded separately in `proofs/vsa-multihop-bound/` and DN-34.
