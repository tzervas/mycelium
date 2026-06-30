"""Output helpers — JSON, CSV, matplotlib plots, and SUMMARY.md (M-832, OQ-F).

VR-5: reports measured RATES only.  The analysis of which subsets admit `Proven` bounds
is maintainer work.  SUMMARY.md flags candidate regimes (where the naive formula
upper-bounds measured rates) but does NOT compute or assert a `Proven` verdict.

matplotlib import is lazy (only when plotting), so the core sweep runs without it.
"""

from __future__ import annotations

import csv
import dataclasses
import json
import sys
from pathlib import Path

from .capacity import CAPACITY_CITATION, MARGIN_MU
from .sweeps import MultihopResult, SingleResult


# ---------------------------------------------------------------------------
# Serialization helpers
# ---------------------------------------------------------------------------


def _to_dict(obj) -> dict:
    return dataclasses.asdict(obj)


def write_json(results: list, path: Path) -> None:
    """Write results as JSON (one list of dicts)."""
    path.write_text(json.dumps([_to_dict(r) for r in results], indent=2), encoding="utf-8")
    print(f"[vsa_bounds] wrote {path}", file=sys.stderr)


def write_csv(results: list, path: Path) -> None:
    """Write results as CSV."""
    if not results:
        return
    rows = [_to_dict(r) for r in results]
    with path.open("w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)
    print(f"[vsa_bounds] wrote {path}", file=sys.stderr)


# ---------------------------------------------------------------------------
# Plotting (lazy matplotlib import)
# ---------------------------------------------------------------------------


def _require_mpl():
    try:
        import matplotlib  # noqa: PLC0415

        matplotlib.use("Agg")
        import matplotlib.pyplot as plt  # noqa: PLC0415

        return plt
    except ImportError:
        print(
            "[vsa_bounds] matplotlib not installed — skipping plots "
            "(install with `uv sync --group gpu`)",
            file=sys.stderr,
        )
        return None


def plot_single(results: list[SingleResult], out_dir: Path, prefix: str = "") -> None:
    """Failure-rate vs d curves for each bundle size m (single sweep)."""
    plt = _require_mpl()
    if plt is None:
        return

    import numpy as np  # noqa: PLC0415

    # Group by m
    m_values = sorted({r.m for r in results})
    fig, ax = plt.subplots(figsize=(10, 6))
    cmap = plt.cm.viridis(np.linspace(0, 1, len(m_values)))

    for i, m in enumerate(m_values):
        rows = sorted([r for r in results if r.m == m], key=lambda r: r.d)
        ds = [r.d for r in rows]
        rates = [r.measured_rate for r in rows]
        req = rows[0].required_dim_proven if rows else None
        ax.plot(ds, rates, "o-", color=cmap[i], label=f"m={m}")
        if req is not None and min(ds) <= req <= max(ds):
            ax.axvline(req, color=cmap[i], linestyle="--", alpha=0.5)

    delta = results[0].delta if results else 0.02
    ax.axhline(delta, color="red", linestyle=":", label=f"delta={delta}")
    ax.set_xlabel("Dimension d")
    ax.set_ylabel("Failure rate (Empirical)")
    ax.set_title(
        f"MAP-I Bundle Capacity: failure rate vs d\n"
        f"(dashed verticals = Proven required_dim; red = delta={delta})"
    )
    ax.legend(bbox_to_anchor=(1.05, 1), loc="upper left")
    ax.set_ylim(-0.02, min(1.05, max(r.measured_rate for r in results) + 0.1))
    fig.tight_layout()
    fname = out_dir / f"{prefix}single-failure-rate-vs-d.png"
    fig.savefig(fname, dpi=150)
    plt.close(fig)
    print(f"[vsa_bounds] plot: {fname}", file=sys.stderr)


def plot_multihop_overview(results: list[MultihopResult], out_dir: Path, prefix: str = "") -> None:
    """Failure rate vs dimension for each composition type and hop depth."""
    plt = _require_mpl()
    if plt is None:
        return

    import numpy as np  # noqa: PLC0415

    comps = sorted({r.composition for r in results})
    models = sorted({r.model for r in results})
    h_vals = sorted({r.h for r in results})
    delta = results[0].delta if results else 0.02

    for model in models:
        for comp in comps:
            fig, axes = plt.subplots(1, len(h_vals), figsize=(5 * len(h_vals), 5), sharey=True)
            if len(h_vals) == 1:
                axes = [axes]

            for ax, h in zip(axes, h_vals):
                rows = [
                    r for r in results if r.model == model and r.composition == comp and r.h == h
                ]
                if not rows:
                    ax.set_visible(False)
                    continue
                F_vals = sorted({r.F for r in rows})
                k_vals = sorted({r.k for r in rows})
                cmap = plt.cm.plasma(np.linspace(0, 1, len(F_vals) * len(k_vals)))
                ci = 0
                for F in F_vals:
                    for k in k_vals:
                        pts = sorted([r for r in rows if r.F == F and r.k == k], key=lambda r: r.d)
                        if not pts:
                            continue
                        ds = [r.d for r in pts]
                        rates = [r.measured_rate for r in pts]
                        # Mark diverging points
                        divs = [r.bound_diverges for r in pts]
                        (line,) = ax.plot(ds, rates, "o-", color=cmap[ci], label=f"F={F},k={k}")
                        for dx, rate, div in zip(ds, rates, divs):
                            if div:
                                ax.scatter([dx], [rate], marker="x", color="red", s=80, zorder=5)
                        ci += 1
                ax.axhline(delta, color="red", linestyle=":", alpha=0.7)
                ax.set_title(f"h={h}")
                ax.set_xlabel("Dimension d")
                if h == h_vals[0]:
                    ax.set_ylabel("Failure rate (Empirical)")
                ax.legend(fontsize=7, loc="upper right")

            fig.suptitle(
                f"Multi-hop ({comp}) | model={model} | delta={delta}\n"
                f"[x] = naive formula predicts OK but rate > delta (Declared divergence)",
                fontsize=10,
            )
            fig.tight_layout()
            fname = out_dir / f"{prefix}multihop-{model}-{comp}.png"
            fig.savefig(fname, dpi=150)
            plt.close(fig)
            print(f"[vsa_bounds] plot: {fname}", file=sys.stderr)


def plot_bound_vs_measured(single: list[SingleResult], out_dir: Path, prefix: str = "") -> None:
    """Scatter: required_dim (Proven formula) vs dimension d, with failure rate as color."""
    plt = _require_mpl()
    if plt is None:
        return

    import numpy as np  # noqa: PLC0415

    ds = np.array([r.d for r in single])
    reqs = np.array([r.required_dim_proven for r in single])
    rates = np.array([r.measured_rate for r in single])
    delta = single[0].delta if single else 0.02

    fig, ax = plt.subplots(figsize=(8, 6))
    sc = ax.scatter(reqs, rates, c=ds, cmap="viridis", alpha=0.7, s=50)
    plt.colorbar(sc, label="Dimension d")
    ax.axhline(delta, color="red", linestyle=":", label=f"delta={delta}")
    ax.axvline(max(ds), color="gray", linestyle="--", alpha=0.3)

    # Region: bound holds AND rate <= delta (candidate Proven regime)
    in_regime = [r for r in single if r.bound_holds and r.bound_respected]
    if in_regime:
        ax.scatter(
            [r.required_dim_proven for r in in_regime],
            [r.measured_rate for r in in_regime],
            marker="o",
            facecolors="none",
            edgecolors="green",
            s=100,
            linewidths=2,
            label="bound holds AND rate<=delta",
        )

    ax.set_xlabel("required_dim (Proven formula)")
    ax.set_ylabel("Measured failure rate (Empirical)")
    ax.set_title(
        "Proven formula vs measured rate\n(green ring = candidate Proven regime; Empirical data)"
    )
    ax.legend()
    fig.tight_layout()
    fname = out_dir / f"{prefix}bound-vs-measured.png"
    fig.savefig(fname, dpi=150)
    plt.close(fig)
    print(f"[vsa_bounds] plot: {fname}", file=sys.stderr)


# ---------------------------------------------------------------------------
# SUMMARY.md — VR-5: data only, no Proven verdicts
# ---------------------------------------------------------------------------


def write_summary(
    single: list[SingleResult],
    multihop: list[MultihopResult],
    out_dir: Path,
    prefix: str = "",
    backend: str = "unknown",
) -> None:
    """Write SUMMARY.md with measured data and candidate regime flags (no Proven verdicts)."""
    path = out_dir / f"{prefix}SUMMARY.md"

    lines: list[str] = []

    lines.append("# VSA Compositional Bounds Experiment — SUMMARY")
    lines.append("")
    lines.append(
        "**Guarantee: Empirical** — all rates are trial-measured.  "
        "The verdict on which subsets admit `Proven` bounds is maintainer analysis "
        "(OQ-F; VR-5)."
    )
    lines.append("")
    lines.append("Capacity formula: `required_dim(m, delta) = ceil(200 * ln(m/delta))`")
    lines.append(f"Citation: {CAPACITY_CITATION}")
    lines.append(f"mu = {MARGIN_MU} (illustrative margin, M-001 probe)")
    lines.append(f"Backend: `{backend}`")
    lines.append("")

    # --- Single sweep ---
    if single:
        lines.append("## Single-hop bundle sweep")
        lines.append("")
        lines.append(
            "Parity check: where `bound_holds=True`, does `measured_rate <= delta`?  "
            "Agreement confirms the Proven formula tracks reality at single-hop."
        )
        lines.append("")
        # Count agreements
        agrees = [r for r in single if r.bound_holds and r.bound_respected]
        disagrees = [r for r in single if r.bound_holds and not r.bound_respected]
        lines.append(
            f"Points where formula holds (d >= required_dim): "
            f"{sum(r.bound_holds for r in single)}/{len(single)}"
        )
        lines.append(f"Of those, measured_rate <= delta (formula upper-bounds rate): {len(agrees)}")
        lines.append(
            f"Of those, measured_rate > delta (formula FAILS to upper-bound): {len(disagrees)}"
        )
        if disagrees:
            lines.append("")
            lines.append(
                "**FLAG: formula violations (Empirical evidence against Proven extension):**"
            )
            for r in disagrees:
                lines.append(
                    f"  m={r.m} d={r.d} required={r.required_dim_proven} "
                    f"rate={r.measured_rate:.4f} delta={r.delta}"
                )
        lines.append("")

        # Table header
        lines.append("| m | d | required_dim | bound_holds | measured_rate | rate<=delta |")
        lines.append("|---|---|---|---|---|---|")
        for r in sorted(single, key=lambda r: (r.m, r.d)):
            lines.append(
                f"| {r.m} | {r.d} | {r.required_dim_proven} | "
                f"{'Y' if r.bound_holds else 'N'} | "
                f"{r.measured_rate:.4f} ({r.failures}/{r.trials}) | "
                f"{'Y' if r.bound_respected else '**N**'} |"
            )
        lines.append("")

    # --- Multi-hop sweep ---
    if multihop:
        lines.append("## Multi-hop composition sweep")
        lines.append("")
        lines.append(
            "Naive extrapolation: effective m = F*k^h (bind_chain/nested_unbind) or h*k "
            "(bundle_of_binds).  **This is Declared** — not a Proven derivation.  "
            "The table shows where the naive formula diverges (predicts OK but rate > delta)."
        )
        lines.append("")

        diverging = [r for r in multihop if r.bound_diverges]
        candidate_proven = [r for r in multihop if r.naive_bound_holds and r.naive_bound_respected]
        lines.append(f"Total multihop points: {len(multihop)}")
        lines.append(
            f"Naive formula holds (d >= naive_required_dim): "
            f"{sum(r.naive_bound_holds for r in multihop)}"
        )
        lines.append(
            f"Candidate regime (formula holds AND rate<=delta — Empirical evidence): "
            f"{len(candidate_proven)}"
        )
        lines.append(f"Formula diverges (holds but rate>delta): {len(diverging)}")
        lines.append("")

        if candidate_proven:
            lines.append("### Candidate regimes (formula upper-bounds rate — Empirical evidence)")
            lines.append("")
            lines.append("| model | comp | F | k | d | h | naive_m | req_d | rate | delta |")
            lines.append("|---|---|---|---|---|---|---|---|---|---|")
            for r in sorted(candidate_proven, key=lambda r: (r.model, r.composition, r.h, r.d)):
                lines.append(
                    f"| {r.model} | {r.composition} | {r.F} | {r.k} | {r.d} | {r.h} | "
                    f"{r.naive_extrapolated_m} | {r.naive_required_dim} | "
                    f"{r.measured_rate:.4f} | {r.delta} |"
                )
            lines.append("")

        if diverging:
            lines.append("### Diverging regimes (formula predicts OK but rate > delta)")
            lines.append("")
            lines.append("| model | comp | F | k | d | h | naive_m | req_d | rate | delta |")
            lines.append("|---|---|---|---|---|---|---|---|---|---|")
            for r in sorted(diverging, key=lambda r: (r.model, r.composition, r.h, r.d)):
                lines.append(
                    f"| {r.model} | {r.composition} | {r.F} | {r.k} | {r.d} | {r.h} | "
                    f"{r.naive_extrapolated_m} | {r.naive_required_dim} | "
                    f"**{r.measured_rate:.4f}** | {r.delta} |"
                )
            lines.append("")

    lines.append("---")
    lines.append("")
    lines.append(
        "**Next steps (maintainer analysis):** The 'candidate Proven subset' rows above are "
        "Empirical evidence that the closed-form bound tracks reality.  "
        "Whether any composition type admits a genuine `Proven` bound requires "
        "a new theorem or reduction to the single-hop Clarkson/Thomas result.  "
        "The diverging rows bound what cannot be covered by the naive extrapolation."
    )

    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"[vsa_bounds] summary: {path}", file=sys.stderr)
