# VSA Compositional Bounds Experiment (M-832 / OQ-F)

Maps where the MAP-I single-hop `Proven` capacity formula extends to multi-hop VSA
compositions (bind-chains, bundle-of-binds, nested unbind) and where it degrades to
`Empirical`-only — feeding the maintainer's GPU analysis of OQ-F tonight.

**Guarantee: Empirical** — all reported rates are trial-measured.
The verdict on `Proven` subsets is maintainer analysis, not computed here (VR-5 / G2).

---

## Prerequisites

```bash
# From the repo root — check GPU access first (WSL2 / native Linux):
bash experiments/docker/gpu-setup.sh

# Install Python deps including torch + matplotlib:
cd experiments
uv sync --group gpu
```

If `uv sync --group gpu` fails (torch not yet in the lock), install manually:

```bash
uv run pip install torch matplotlib
```

The numpy path always works without torch — CI and tests use it.

---

## Run commands (paste tonight)

### Full GPU sweep (both single-hop and multihop, all default sizes)

```bash
cd experiments
python -m mycelium_experiments.vsa_bounds --sweep both
```

Expected runtime on a mid-range NVIDIA GPU (RTX 3080 / 4080 class):

- `single` sweep: ~2-5 min (m in {3,5,10,20,50}, d in {512..16384}, 1000 trials each)
- `multihop` sweep: ~20-60 min depending on GPU speed
  (models=mapi+mapb, F in {2,3}, k in {4,8,16}, h in {1,2,3}, d in {512..16384},
  500 trials each — 6 compositions × 2 models × 2×3×3×6 = 648 points)

Total: ~1-2 hours at full size.  Use `--quick` for a 5-minute sanity check first.

### Quick CPU sanity check (no GPU needed)

```bash
cd experiments
python -m mycelium_experiments.vsa_bounds --sweep both --quick --numpy-only
```

Expected runtime: ~2-5 min on a modern CPU.

### Single-hop parity check only (fastest; confirms Proven formula matches measured rates)

```bash
cd experiments
python -m mycelium_experiments.vsa_bounds --sweep single
```

### Multihop only (the OQ-F research question)

```bash
cd experiments
python -m mycelium_experiments.vsa_bounds --sweep multihop
```

### With all four VSA models (MAP-I, MAP-B, HRR, FHRR)

```bash
cd experiments
python -m mycelium_experiments.vsa_bounds --sweep both --model all
```

HRR and FHRR bind is O(d log d) via FFT; at d=16384 this is measurably slower but
tractable on GPU.

### Custom sizes (tune to your GPU)

```bash
cd experiments
python -m mycelium_experiments.vsa_bounds \
    --sweep both \
    --d-values 2048,4096,8192,16384 \
    --m-values 5,10,20,50 \
    --F-values 2,3 \
    --k-values 8,16,32 \
    --h-values 1,2,3 \
    --trials 2000
```

---

## Output

Results land in `experiments/results/` (override with `--results-dir DIR`):

```
<utc>-vsa-bounds-single.json        # per-point measurements (single sweep)
<utc>-vsa-bounds-single.csv         # same as CSV
<utc>-vsa-bounds-multihop.json      # per-point measurements (multihop sweep)
<utc>-vsa-bounds-multihop.csv       # same as CSV
<utc>-vsa-bounds-SUMMARY.md         # candidate regimes + diverging regimes (Empirical data)
<utc>-vsa-bounds-single-failure-rate-vs-d.png        # failure rate vs d curves
<utc>-vsa-bounds-bound-vs-measured.png               # Proven formula vs measured scatter
<utc>-vsa-bounds-multihop-mapi-bind_chain.png        # per-composition failure rate plots
... (one plot per model x composition)
```

Key columns in the JSON/CSV:

- `measured_rate`: trial-measured failure rate (`Empirical`)
- `bound_holds`: True iff `d >= required_dim(m, delta)` (Proven side-condition)
- `bound_respected`: True iff `measured_rate <= delta`
- `naive_extrapolated_m`: effective m for naive formula extrapolation (`Declared`)
- `naive_bound_holds`: True iff `d >= required_dim(naive_m, delta)`
- `bound_diverges`: True iff naive formula predicts OK but measured rate > delta
  (evidence against Proven extension)

---

## What the SUMMARY.md tells you

- **Candidate Proven subset**: points where `naive_bound_holds AND rate <= delta`.
  These are `Empirical` evidence that the closed-form formula tracks multi-hop
  reality.  A genuine `Proven` result requires a new theorem or reduction.
- **Diverging regimes**: points where the formula predicts OK but rate exceeds delta.
  These bound what cannot be covered by the naive extrapolation.

The maintainer's task: inspect the candidate regimes and decide whether any composition
type admits a derivation reducing to Clarkson/Thomas.

---

## Backend

Detected automatically at startup; printed to stderr:

```
[vsa_bounds] backend: torch-cuda:NVIDIA GeForce RTX 4080
```

If torch is not installed:

```
[vsa_bounds] backend: numpy-cpu  (torch not installed; run `uv sync --group gpu` ...)
```

Force numpy path: `--numpy-only`.

GPU preflight reference: `experiments/docker/gpu-setup.sh` (NVIDIA Container Toolkit,
WSL2 / native Linux, CUDA 12.8).

---

## Relation to the Rust resonator profile

`crates/mycelium-vsa/tests/resonator_profile.rs` validates the MAP-I resonator at
`F<=3, k<=16, prod_k<=4096, d>=4096, delta=0.02` (`MAPI_RESONATOR_PROFILE`).
This experiment is the scaled-up Python analogue — it sweeps the same regime and
extends it to multi-hop compositions the Rust test does not cover.

The `required_dim` Python formula in `capacity.py` is cross-validated against the
Rust `capacity.rs` via `experiments/tests/test_vsa_bounds.py::test_required_dim_parity`
(hardcoded values from the M-001 LH probe table).
