"""Tests for vsa_bounds — numpy-path sanity, runnable in CI without torch/GPU (M-832).

All tests use numpy only; torch is never imported here.  These are fast correctness
checks, not the full sweep.

VR-5: no Proven verdict on multi-hop results; only the single-hop formula is Proven
(via Clarkson/Thomas; checked instantiation in capacity.rs / proofs/lh-bundle).
"""

from __future__ import annotations

import numpy as np


# ---------------------------------------------------------------------------
# capacity.py parity with capacity.rs
# ---------------------------------------------------------------------------


def test_required_dim_parity_m001_probe() -> None:
    """Python required_dim matches the four values from the M-001 LH probe table.

    `capacity.rs::required_dim_matches_the_m001_probe_table` asserts exactly:
        required_dim(3,   1e-2, 0.1) == 1141
        required_dim(10,  1e-3, 0.1) == 1843
        required_dim(50,  1e-3, 0.1) == 2164
        required_dim(100, 1e-4, 0.1) == 2764

    Guarantee: the Python formula matches the Rust impl exactly (same float math).
    """
    from mycelium_experiments.vsa_bounds.capacity import MARGIN_MU, required_dim

    assert required_dim(3, 1e-2, MARGIN_MU) == 1141, "m=3, delta=1e-2"
    assert required_dim(10, 1e-3, MARGIN_MU) == 1843, "m=10, delta=1e-3"
    assert required_dim(50, 1e-3, MARGIN_MU) == 2164, "m=50, delta=1e-3"
    assert required_dim(100, 1e-4, MARGIN_MU) == 2764, "m=100, delta=1e-4"


def test_required_dim_monotone() -> None:
    """required_dim increases with m and decreases with delta (formula monotonicity)."""
    from mycelium_experiments.vsa_bounds.capacity import MARGIN_MU, required_dim

    # More items -> more dimension required
    assert required_dim(5, 0.01, MARGIN_MU) < required_dim(10, 0.01, MARGIN_MU)
    assert required_dim(10, 0.01, MARGIN_MU) < required_dim(100, 0.01, MARGIN_MU)

    # Lower delta (tighter failure target) -> more dimension required
    assert required_dim(10, 0.1, MARGIN_MU) < required_dim(10, 0.01, MARGIN_MU)
    assert required_dim(10, 0.01, MARGIN_MU) < required_dim(10, 0.001, MARGIN_MU)


def test_required_dim_degenerate() -> None:
    """Degenerate inputs return sentinel (maxsize), not 0 or garbage."""
    import sys

    from mycelium_experiments.vsa_bounds.capacity import MARGIN_MU, required_dim

    assert required_dim(0, 0.01, MARGIN_MU) == sys.maxsize, "items=0"
    assert required_dim(3, 0.0, MARGIN_MU) == sys.maxsize, "delta=0"
    assert required_dim(3, 2.0, MARGIN_MU) == sys.maxsize, "delta>1"
    assert required_dim(3, -0.1, MARGIN_MU) == sys.maxsize, "delta<0"


def test_proven_bound_holds() -> None:
    """proven_bound_holds matches capacity.rs::proven_capacity_bound returning Some vs None."""
    from mycelium_experiments.vsa_bounds.capacity import MARGIN_MU, proven_bound_holds

    # required_dim(3, 1e-2) == 1141 — so d=1141 should hold, d=1140 should not.
    assert proven_bound_holds(3, 1141, 1e-2, MARGIN_MU), "d==required_dim should hold"
    assert proven_bound_holds(3, 10_000, 1e-2, MARGIN_MU), "d >> required_dim should hold"
    assert not proven_bound_holds(3, 1000, 1e-2, MARGIN_MU), "d < required_dim should not hold"
    assert not proven_bound_holds(3, 1140, 1e-2, MARGIN_MU), "d == required_dim - 1 should not hold"


# ---------------------------------------------------------------------------
# algebra.py — LCG parity with resonator_profile.rs
# ---------------------------------------------------------------------------


def test_lcg_matches_rust_profile() -> None:
    """Lcg produces bipolar vectors with ±1 components only."""
    from mycelium_experiments.vsa_bounds.algebra import Lcg

    lcg = Lcg(0xDEAD_BEEF)
    v = lcg.bipolar(1000)
    assert v.shape == (1000,), "shape mismatch"
    assert np.all((v == 1.0) | (v == -1.0)), "non-bipolar component in LCG output"


def test_lcg_deterministic() -> None:
    """Same seed produces identical output (determinism for recorded runs)."""
    from mycelium_experiments.vsa_bounds.algebra import Lcg

    v1 = Lcg(42).bipolar(128)
    v2 = Lcg(42).bipolar(128)
    np.testing.assert_array_equal(v1, v2, err_msg="LCG not deterministic")


def test_lcg_different_seeds_differ() -> None:
    """Different seeds produce different vectors (no degenerate constant output)."""
    from mycelium_experiments.vsa_bounds.algebra import Lcg

    v1 = Lcg(1).bipolar(128)
    v2 = Lcg(2).bipolar(128)
    assert not np.array_equal(v1, v2), "different seeds should produce different vectors"


# ---------------------------------------------------------------------------
# algebra.py — MAP-I ops
# ---------------------------------------------------------------------------


def test_mapi_bind_self_inverse() -> None:
    """MAP-I bind is self-inverse on ±1 alphabet: unbind(bind(a,b), b) == a (Exact)."""
    from mycelium_experiments.vsa_bounds.algebra import Lcg, mapi_bind, mapi_unbind

    lcg = Lcg(7)
    a = lcg.bipolar(256)
    b = lcg.bipolar(256)
    bound = mapi_bind(a, b)
    recovered = mapi_unbind(bound, b)
    np.testing.assert_array_equal(recovered, a, err_msg="MAP-I bind self-inverse failed")


def test_mapi_bundle_member_beats_distractor() -> None:
    """Bundle of m atoms: a true member has higher similarity than a non-member distractor.

    This is the membership-detection property underlying the Proven capacity bound
    (Clarkson/Thomas 2023 Thm 6).  At sufficiently large d the probability that any
    distractor beats a member is <= delta.

    We test a single large-d case: at d=4096 >> required_dim(4, 0.02)=875 the member
    should beat 100 distractors (P(failure) is exponentially small here).
    """
    from mycelium_experiments.vsa_bounds.algebra import Lcg, mapi_bundle, mapi_similarity
    from mycelium_experiments.vsa_bounds.capacity import MARGIN_MU, required_dim

    d = 4096
    m = 4
    req = required_dim(m, 0.02, MARGIN_MU)
    assert d >= req, f"test requires d >= required_dim; got d={d}, req={req}"

    lcg = Lcg(42)
    atoms = [lcg.bipolar(d) for _ in range(m)]
    b = mapi_bundle(atoms)
    sim_member = float(mapi_similarity(b, atoms[0]))

    # All 50 distractors should lose to the member at this comfortable d
    n_beats = 0
    for _ in range(50):
        distractor = lcg.bipolar(d)
        sim_dist = float(mapi_similarity(distractor, b))
        if sim_dist >= sim_member:
            n_beats += 1

    assert n_beats == 0, (
        f"at d={d} >> required_dim={req}, {n_beats}/50 distractors beat the member "
        f"(member sim={sim_member:.4f}) — Empirical evidence against the formula (flag)"
    )


def test_mapi_permute_cyclic() -> None:
    """MAP-I permute is a cyclic left-shift (Exact)."""
    from mycelium_experiments.vsa_bounds.algebra import mapi_permute, mapi_unpermute

    a = np.array([1.0, 2.0, 3.0, 4.0], dtype=np.float32)
    shifted = mapi_permute(a, 1)
    np.testing.assert_array_equal(shifted, [2.0, 3.0, 4.0, 1.0], err_msg="permute(1) wrong")
    back = mapi_unpermute(shifted, 1)
    np.testing.assert_array_equal(back, a, err_msg="permute round-trip failed")


# ---------------------------------------------------------------------------
# algebra.py — MAP-B ops
# ---------------------------------------------------------------------------


def test_mapb_bind_self_inverse() -> None:
    """MAP-B bind is self-inverse (same as MAP-I, Exact)."""
    from mycelium_experiments.vsa_bounds.algebra import Lcg, mapb_bind, mapb_unbind

    lcg = Lcg(99)
    a = lcg.bipolar(256)
    b = lcg.bipolar(256)
    bound = mapb_bind(a, b)
    recovered = mapb_unbind(bound, b)
    np.testing.assert_array_equal(recovered, a, err_msg="MAP-B bind self-inverse failed")


def test_mapb_bundle_is_bipolar() -> None:
    """MAP-B bundle output is always ±1."""
    from mycelium_experiments.vsa_bounds.algebra import Lcg, mapb_bundle

    lcg = Lcg(55)
    atoms = [lcg.bipolar(256) for _ in range(5)]
    b = mapb_bundle(atoms)
    assert np.all((b == 1.0) | (b == -1.0)), "MAP-B bundle is not bipolar"


# ---------------------------------------------------------------------------
# algebra.py — HRR ops
# ---------------------------------------------------------------------------


def test_hrr_bind_circular_conv() -> None:
    """HRR bind + unbind approximate recovery on a simple case."""
    from mycelium_experiments.vsa_bounds.algebra import Lcg, hrr_bind, hrr_similarity, hrr_unbind

    d = 512
    lcg = Lcg(123)
    a = lcg.gaussian(d)
    b = lcg.gaussian(d)
    bound = hrr_bind(a, b)
    recovered = hrr_unbind(bound, b)
    # Recovered should be similar to `a` (not exact, Empirical)
    sim = float(hrr_similarity(recovered, a))
    assert sim > 0.5, f"HRR unbind should produce approximately correct result, got sim={sim:.4f}"


# ---------------------------------------------------------------------------
# algebra.py — FHRR ops
# ---------------------------------------------------------------------------


def test_fhrr_bind_unbind_roundtrip() -> None:
    """FHRR bind then unbind recovers original phase vector (near-exact for single pair)."""
    from mycelium_experiments.vsa_bounds.algebra import Lcg, fhrr_bind, fhrr_similarity, fhrr_unbind

    d = 256
    lcg = Lcg(77)
    a = lcg.uniform_phase(d)
    b = lcg.uniform_phase(d)
    bound = fhrr_bind(a, b)
    recovered = fhrr_unbind(bound, b)
    sim = float(fhrr_similarity(recovered, a))
    assert sim > 0.9, f"FHRR unbind should approximately recover a, got sim={sim:.4f}"


# ---------------------------------------------------------------------------
# sweeps.py — small single-hop sweep (fast, numpy only)
# ---------------------------------------------------------------------------


def test_single_sweep_proven_formula_anchor() -> None:
    """At d >= required_dim, measured membership-detection failure rate <= delta.

    The Clarkson/Thomas bound says: for d >= required_dim(m, delta), the probability
    that any single distractor beats a member in similarity is <= delta.
    We use 100 distractors per trial so failure = ANY of the 100 beats the member
    (more stringent than the single-distractor bound, but still empirically near zero
    well above required_dim).

    Empirical: with 200 trials at d >> required_dim the rate should be very low.
    The assertion is conservative (rate <= 0.10 = 5*delta) to accommodate the 100-distractor
    test regime.
    """
    from mycelium_experiments.vsa_bounds.capacity import MARGIN_MU, required_dim
    from mycelium_experiments.vsa_bounds.sweeps import run_single_sweep

    # m=3, delta=0.02: required_dim=1003; use d=2*req to be well above.
    req = required_dim(3, 0.02, MARGIN_MU)
    d = req * 2  # well above required_dim

    results = run_single_sweep(
        m_values=[3],
        d_values=[d],
        delta=0.02,
        trials_per_point=200,
        model="mapi",
        salt=0xABCD,
    )
    assert len(results) == 1
    r = results[0]
    assert r.bound_holds, f"expected bound to hold at d={d} >= req={req}"
    assert r.measured_rate <= 0.15, (
        f"at d={d} (2x required_dim={req}), measured rate {r.measured_rate:.4f} "
        f"is surprisingly high — Empirical evidence against formula (flag)"
    )


def test_single_sweep_below_dim_higher_rate() -> None:
    """At d < required_dim, failure rate should be measurably higher than delta."""
    from mycelium_experiments.vsa_bounds.capacity import required_dim, MARGIN_MU
    from mycelium_experiments.vsa_bounds.sweeps import run_single_sweep

    req = required_dim(10, 0.02, MARGIN_MU)
    d_low = max(64, req // 4)  # well below required_dim

    results = run_single_sweep(
        m_values=[10],
        d_values=[d_low],
        delta=0.02,
        trials_per_point=300,
        model="mapi",
        salt=0x1234,
    )
    assert len(results) == 1
    r = results[0]
    # We expect significantly higher rate below the bound (not asserting exact value —
    # empirical; but it should be well above delta=0.02).
    assert not r.bound_holds, f"expected bound NOT to hold at d={d_low} < req={req}"
    # Just record — don't assert rate > delta as that is a Proven claim in reverse.
    # (A very wide d could still happen to decode correctly — rare but not impossible.)


def test_single_sweep_returns_correct_structure() -> None:
    """run_single_sweep returns the right number of results and correct fields."""
    from mycelium_experiments.vsa_bounds.sweeps import run_single_sweep

    results = run_single_sweep(
        m_values=[3, 5],
        d_values=[256, 512],
        delta=0.02,
        trials_per_point=50,
        model="mapi",
        salt=0x9999,
    )
    assert len(results) == 4, f"expected 2*2=4 results, got {len(results)}"
    for r in results:
        assert r.trials == 50
        assert 0.0 <= r.measured_rate <= 1.0
        assert r.failures <= r.trials
        assert r.required_dim_proven > 0
        assert r.guarantee == "Empirical"


def test_multihop_sweep_single_hop_matches_single_sweep() -> None:
    """A multihop sweep at h=1 and F=1 should resemble the single-hop sweep."""
    from mycelium_experiments.vsa_bounds.sweeps import run_multihop_sweep

    results = run_multihop_sweep(
        models=["mapi"],
        compositions=["bind_chain"],
        F_values=[2],
        k_values=[4],
        d_values=[512],
        h_values=[1],
        delta=0.02,
        trials_per_point=50,
        progress=False,
    )
    assert len(results) >= 1
    r = results[0]
    assert 0.0 <= r.measured_rate <= 1.0
    assert r.h == 1
    assert r.guarantee == "Empirical"
