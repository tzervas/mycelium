"""Run the VSA compositional bounds experiment (M-832, OQ-F).

Maps where the MAP-I `Proven` capacity formula extends to multi-hop compositions
and where it degrades to `Empirical`-only.  Feeds maintainer insight for OQ-F.

Quick start (GPU, default sizes):
    cd experiments
    python -m mycelium_experiments.vsa_bounds --sweep single
    python -m mycelium_experiments.vsa_bounds --sweep multihop
    python -m mycelium_experiments.vsa_bounds --sweep both

CPU-only / quick profile:
    python -m mycelium_experiments.vsa_bounds --sweep both --quick

Demo profile (CPU-feasible, produces in-regime obligations for small-m cases):
    python -m mycelium_experiments.vsa_bounds --demo --numpy-only --no-plots

Proof-discovery mode (emits candidate bounds and checkable proof obligations):
    python -m mycelium_experiments.vsa_bounds --proof
    python -m mycelium_experiments.vsa_bounds --proof --quick
    python -m mycelium_experiments.vsa_bounds --proof --emit-obligations --results-dir results/

The --proof mode runs the multihop sweep, fits candidate closed-form bounds,
validates them empirically, and emits:
  - A PROOF-SUMMARY.md with comparative ranking per composition and emitted obligations.
  - SMT-LIB 2 (.smt2) files for Z3 to discharge (all models x compositions).
  - Liquid Haskell skeleton (.hs) files mirroring proofs/lh-bundle/.
  - Lean 4 skeleton (.lean) files for the OQ-A/M-827 mechanization path.

Results land in experiments/results/ (default) or --results-dir DIR.

VR-5: measured RATES only; candidate bounds are Declared; Proven is NEVER stamped here.
Never-silent (G2): backend printed at startup, unavailable GPU falls back loudly,
refuted candidates are reported, not hidden.
"""

from __future__ import annotations

import argparse
import sys
from datetime import datetime, timezone
from pathlib import Path


def _now_utc() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="python -m mycelium_experiments.vsa_bounds",
        description=(
            "VSA compositional bounds experiment (M-832, OQ-F). "
            "Sweeps MAP-I (and MAP-B) failure rate vs dimension at single-hop and multi-hop."
        ),
    )
    p.add_argument(
        "--sweep",
        choices=["single", "multihop", "both"],
        default="both",
        help="Which sweep to run (default: both).",
    )
    p.add_argument(
        "--model",
        choices=["mapi", "mapb", "hrr", "fhrr", "all"],
        default="mapi",
        help="VSA model(s) to include in the sweep (default: mapi; 'all' = mapi mapb hrr fhrr).",
    )
    p.add_argument(
        "--quick",
        action="store_true",
        help=(
            "Quick/CPU profile: small dimensions and trial counts for local testing "
            "(d<=1024, trials<=200, F<=2, k<=8, h<=2)."
        ),
    )
    p.add_argument(
        "--demo",
        action="store_true",
        help=(
            "Demo/CPU profile: raises d up to ~8192 with reduced trial counts (~50) "
            "to produce non-empty in-regime obligations without GPU.  "
            "Uses F=[2], k=[4], h=[1,2] — smallest-m cases where the candidate_dim "
            "is well below d=8192.  Implies --proof.  "
            "Suitable for generating committed EXAMPLE artifacts.  "
            "Guarantee: Empirical (trial-measured rates at these d values)."
        ),
    )
    p.add_argument(
        "--trials",
        type=int,
        default=None,
        help="Override Monte-Carlo trial count per point (default: 1000 single / 500 multihop).",
    )
    p.add_argument(
        "--delta",
        type=float,
        default=0.02,
        help="Target failure probability delta (default: 0.02, matching MAPI_RESONATOR_PROFILE).",
    )
    p.add_argument(
        "--results-dir",
        default=None,
        help="Output directory (default: experiments/results/ relative to CWD).",
    )
    p.add_argument(
        "--no-plots",
        action="store_true",
        help="Skip matplotlib plots (useful when matplotlib is not installed).",
    )
    p.add_argument(
        "--numpy-only",
        action="store_true",
        help="Force numpy-cpu backend even if torch/CUDA is available.",
    )
    # Proof-discovery mode.
    p.add_argument(
        "--proof",
        action="store_true",
        help=(
            "Proof-discovery mode: run the multihop sweep, fit candidate closed-form "
            "multi-hop bounds (Declared), validate them empirically, and emit checkable "
            "proof obligations (SMT-LIB + LH skeleton + Lean 4 skeleton).  "
            "Implies --sweep multihop.  "
            "VR-5: no Proven claims are made; the obligations are Declared stubs."
        ),
    )
    p.add_argument(
        "--emit-obligations",
        action="store_true",
        help=(
            "Emit SMT-LIB (.smt2), Liquid Haskell (.hs), and Lean 4 (.lean) proof "
            "obligation files alongside the PROOF-SUMMARY.md (implies --proof).  "
            "Emits all models x all compositions.  "
            "Also copies obligations to proofs/vsa-multihop-bound/ stubs if present."
        ),
    )
    p.add_argument(
        "--eff-m-model",
        choices=["A_exponential", "B_linear", "C_sqrt", "all"],
        default="all",
        help=(
            "Effective-m model(s) to test in proof-discovery mode "
            "(default: all three; see candidate_bound.py for descriptions)."
        ),
    )
    # GPU-tuning knobs
    p.add_argument(
        "--d-values",
        help="Comma-separated list of dimensions to sweep (overrides --quick / defaults).",
    )
    p.add_argument(
        "--m-values",
        help="Comma-separated bundle sizes for single sweep (default: 3,5,10,20,50).",
    )
    p.add_argument(
        "--F-values",
        help="Comma-separated factor slot counts for multihop (default: 2,3).",
    )
    p.add_argument(
        "--k-values",
        help="Comma-separated codebook sizes for multihop (default: 4,8,16).",
    )
    p.add_argument(
        "--h-values",
        help="Comma-separated hop depths for multihop (default: 1,2,3).",
    )
    return p


def _parse_int_list(s: str | None, default: list[int]) -> list[int]:
    if s is None:
        return default
    return [int(x.strip()) for x in s.split(",") if x.strip()]


def main() -> int:
    from . import backend as _be
    from .output import (
        plot_bound_vs_measured,
        plot_multihop_overview,
        plot_single,
        write_csv,
        write_json,
        write_summary,
    )
    from .sweeps import run_multihop_sweep, run_single_sweep

    args = _build_parser().parse_args()

    # --demo implies --proof (and --emit-obligations for convenience).
    demo_mode = args.demo
    # --proof and --emit-obligations both imply the proof-discovery path.
    proof_mode = args.proof or args.emit_obligations or demo_mode

    # Backend selection — never-silent (G2).
    be = _be.select(force_numpy=args.numpy_only)

    # Results directory.
    if args.results_dir:
        results_dir = Path(args.results_dir).expanduser()
    else:
        results_dir = Path("results")
    results_dir.mkdir(parents=True, exist_ok=True)

    run_id = _now_utc()
    prefix = f"{run_id}-vsa-bounds-"

    # Model selection.
    if args.model == "all":
        models = ["mapi", "mapb", "hrr", "fhrr"]
    else:
        models = [args.model]

    # Dimension values.
    # --demo: CPU-feasible profile designed to produce in-regime points.
    # The key insight: for Model A/B/C at h=1, F=2, k=4:
    #   m_eff = F*k^1 = 2*4 = 8 → required_dim(8, 0.02) ≈ 1382
    #   At d=8192 >> 1382 → in-regime. (Empirical)
    # For h=2, F=2, k=4 (Model B):
    #   m_eff = F*k*h = 2*4*2 = 16 → required_dim(16, 0.02) ≈ 1659
    #   Also well within d=8192. (Empirical)
    if demo_mode:
        default_d = [1024, 2048, 4096, 8192]
        default_m = [3, 5, 10]
        default_F = [2]
        default_k = [4]
        default_h = [1, 2]
        default_trials_single = 50
        default_trials_multi = 50
        print(
            "[vsa_bounds] --demo profile: d=[1024,2048,4096,8192] F=[2] k=[4] h=[1,2] "
            "trials=50 (CPU-feasible, designed to produce in-regime obligations).",
            file=sys.stderr,
        )
    elif args.quick:
        default_d = [256, 512, 1024]
        default_m = [3, 5, 10]
        default_F = [2]
        default_k = [4, 8]
        default_h = [1, 2]
        default_trials_single = 200
        default_trials_multi = 200
    else:
        # GPU-appropriate sizes (d up to 16384, 1000+ trials)
        default_d = [512, 1024, 2048, 4096, 8192, 16384]
        default_m = [3, 5, 10, 20, 50]
        default_F = [2, 3]
        default_k = [4, 8, 16]
        default_h = [1, 2, 3]
        default_trials_single = 1000
        default_trials_multi = 500

    d_values = _parse_int_list(args.d_values, default_d)
    m_values = _parse_int_list(args.m_values, default_m)
    F_values = _parse_int_list(args.F_values, default_F)
    k_values = _parse_int_list(args.k_values, default_k)
    h_values = _parse_int_list(args.h_values, default_h)

    trials_single = args.trials if args.trials is not None else default_trials_single
    trials_multi = args.trials if args.trials is not None else default_trials_multi

    # Proof mode implies multihop sweep.
    if proof_mode:
        do_single = False
        do_multi = True
    else:
        do_single = args.sweep in ("single", "both")
        do_multi = args.sweep in ("multihop", "both")

    single_results = []
    multihop_results = []

    if do_single:
        print(
            f"[vsa_bounds] single-hop sweep: m={m_values} d={d_values} "
            f"delta={args.delta} trials={trials_single}",
            file=sys.stderr,
        )
        # Run per model
        for model in models:
            sr = run_single_sweep(
                m_values=m_values,
                d_values=d_values,
                delta=args.delta,
                trials_per_point=trials_single,
                model=model,
                salt=0xDEAD_BEEF ^ hash(model),
            )
            single_results.extend(sr)

        write_json(single_results, results_dir / f"{prefix}single.json")
        write_csv(single_results, results_dir / f"{prefix}single.csv")

        if not args.no_plots:
            plot_single(single_results, results_dir, prefix=prefix)
            plot_bound_vs_measured(single_results, results_dir, prefix=prefix)

    if do_multi:
        print(
            f"[vsa_bounds] multihop sweep: models={models} F={F_values} k={k_values} "
            f"h={h_values} d={d_values} delta={args.delta} trials={trials_multi}",
            file=sys.stderr,
        )
        multihop_results = run_multihop_sweep(
            models=models,
            F_values=F_values,
            k_values=k_values,
            d_values=d_values,
            h_values=h_values,
            delta=args.delta,
            trials_per_point=trials_multi,
            progress=True,
        )
        write_json(multihop_results, results_dir / f"{prefix}multihop.json")
        write_csv(multihop_results, results_dir / f"{prefix}multihop.csv")

        if not args.no_plots:
            plot_multihop_overview(multihop_results, results_dir, prefix=prefix)

    write_summary(single_results, multihop_results, results_dir, prefix=prefix, backend=be)

    # ---------------------------------------------------------------------------
    # Proof-discovery mode: fit candidate bounds and emit obligations.
    # ---------------------------------------------------------------------------
    if proof_mode and multihop_results:
        from .candidate_bound import EffMModel, fit_and_validate, summarize_candidates
        from .proof_obligation import emit_obligations

        print(
            "\n[vsa_bounds] proof-discovery mode: fitting candidate multi-hop bounds ...",
            file=sys.stderr,
        )

        # Resolve effective-m models to test.
        if args.eff_m_model == "all":
            eff_models: list[EffMModel] = ["A_exponential", "B_linear", "C_sqrt"]
        else:
            eff_models = [args.eff_m_model]  # type: ignore[list-item]

        candidates = fit_and_validate(multihop_results, eff_m_models=eff_models)

        if not candidates:
            print(
                "[vsa_bounds] proof-discovery: no candidates — multihop sweep produced no data.",
                file=sys.stderr,
            )
        else:
            # Print candidate summary.
            summary_text = summarize_candidates(candidates)
            print(summary_text, file=sys.stderr)

            # Emit obligations (all models x all compositions x SMT2 + LH + Lean).
            proof_dir = results_dir
            obligations = emit_obligations(
                candidates,
                out_dir=proof_dir,
                run_id=run_id,
                backend=be,
                delta=args.delta,
            )

            proof_summary_path = obligations.get("PROOF-SUMMARY")
            if proof_summary_path:
                print(
                    f"[vsa_bounds] PROOF-SUMMARY: {proof_summary_path.resolve()}",
                    file=sys.stderr,
                )

            # Report emitted files.
            for key, path in sorted(obligations.items()):
                if key != "PROOF-SUMMARY":
                    print(
                        f"[vsa_bounds] obligation emitted: {path.name}",
                        file=sys.stderr,
                    )

            # Copy obligation stubs into proofs/vsa-multihop-bound/ if the directory exists.
            # (The orchestrator owns it; we populate run-outputs only.)
            # SMT2 and HS go to root; Lean goes to lean/ subdir.
            proofs_dir = Path("..") / "proofs" / "vsa-multihop-bound"
            if proofs_dir.exists():
                import shutil  # noqa: PLC0415

                lean_dir = proofs_dir / "lean"
                for key, path in obligations.items():
                    if key == "PROOF-SUMMARY":
                        dest = proofs_dir / path.name
                    elif path.suffix == ".lean":
                        lean_dir.mkdir(parents=True, exist_ok=True)
                        dest = lean_dir / path.name
                    else:
                        dest = proofs_dir / path.name
                    shutil.copy2(path, dest)
                    print(
                        f"[vsa_bounds] copied to proofs/vsa-multihop-bound/: {dest.name}",
                        file=sys.stderr,
                    )

    print(
        f"\n[vsa_bounds] done.  Results in: {results_dir.resolve()}",
        file=sys.stderr,
    )
    print(
        f"  SUMMARY: {results_dir.resolve()}/{prefix}SUMMARY.md",
        file=sys.stderr,
    )
    if proof_mode:
        print(
            "  Run with --proof to discover candidate bounds and emit proof obligations.",
            file=sys.stderr,
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
