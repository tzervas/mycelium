# GAP-2 Medium-Findings Ledger — full working notes & evidence (handoff)

| Field | Value |
|---|---|
| **Produced by** | leaf agent `claude/gap2-medium-ledger`, 2026-06-24 |
| **For** | DN-19 §2 GAP-2 / ADR-021 Gate A2 maintainer ratification |
| **Status** | **DRAFT verification record** — Gate A2 needs maintainer sign-off; this is not the sign-off |
| **Scope** | Verify (don't assume) every open Medium from the 2026-06-14 deep review listed in DN-19 §2 GAP-2 |

This file is the archival dump of the per-finding working notes, grep results, and `file:line`
citations behind the concise ledger appended to `DN-19-Road-to-1.0.0.md`. Keep complex analysis
here; the DN-19 subsection is the short maintainer-facing summary.

## Method (grounding discipline, VR-5)

The 2026-06-15 remediation roadmap listed ~25 Medium ids across WS2–WS6 as "remaining". A prior
ledger — `docs/reviews/2026-06-14-deep-review/06-medium-findings-ledger.md` (M-653, 2026-06-21) —
already claims all 25 **Fixed** (23 "prior", 2 "M-653"), but Gate A2 still reads ⏳ in ADR-021/DN-19
because no maintainer sign-off was recorded. This agent did **not** trust that ledger: each cited
regression test / variant / code marker was re-located in the *current* tree (`origin/main` tip
`db4a6be`) by name, and a representative subset was *executed* to confirm the tests pass (not merely
exist). Verdicts below are grounded in those grep hits and test runs; where a citation was imprecise
the discrepancy is recorded honestly rather than smoothed over.

## Enumeration

DN-19 §2 GAP-2 lists exactly these open Mediums (25 finding-ids, matching the 06-ledger):

- **WS2:** A1-04, A1-05, A6-03, A6-06, A6-08, A6-09
- **WS3:** A3-04, A3-05, A3-06/C1-04, A3-07, A3-08, A3-09, A3-10
- **WS4:** A4-03, A4-04, reject-corpus assertion strength
- **WS5:** A5-02, A5-03, A5-05, A5-06, A5-07, A5-08
- **WS6:** A6-05, A6-10/B2-04, A6-11

## Per-finding evidence

### WS2 — contract / schema integrity

| Finding | Verdict | Evidence (`file:line`) |
|---|---|---|
| A1-04 | FIXED | `crates/mycelium-core/src/recon.rs:394` `resonator_allows_basis_weaker_than_empirical` (basis encoded by rank, not `is ProvenThm`). |
| A1-05 | FIXED | `crates/mycelium-cert/tests/sc3.rs:49-54` asserts `Validated { strength }` equals `expected` — the discarded-`strength` mutant now caught. |
| A6-03 | FIXED (M-653) | `crates/mycelium-core/tests/serde_roundtrip.rs:364` `wire_spellings_are_pinned_per_bound_kind_basis_and_layout`. **Executed: 1 passed.** |
| A6-06 | FIXED | `crates/mycelium-core/src/recon.rs:426` `resonator_range_checks_a_stray_cleanup_threshold` + `:477` `resonator_range_checks_optional_params`. |
| A6-08 | FIXED | `crates/mycelium-core/src/meta.rs:130` returns `WfError::MalformedSparsity` (not `MalformedBound`); test `:426` `out_of_range_sparsity_is_malformed_sparsity`. |
| A6-09 | FIXED | `crates/mycelium-cert/tests/swap.rs:120` `emitted_certificate_matches_committed_example`. |

### WS3 — VSA tag honesty

| Finding | Verdict | Evidence (`file:line`) |
|---|---|---|
| A3-04 | FIXED | `crates/mycelium-vsa/src/mapi.rs:408` + `mapb.rs:311` `value_bind_unbind_refuse_non_bipolar_a3_04`. |
| A3-05 | FIXED (honest skip) | `proofs/lh-bundle/src/Bundle.hs:9-15` header reconciled to README "DISCHARGED" with an explicit note that cabal/z3 are absent locally so it is a *comment reconciliation against the README run log*, not a re-run. Honest: the LiquidHaskell discharge is `Declared`-here (README is the checkable artifact), not re-verified by this agent. |
| A3-06/C1-04 | FIXED (qualified, **not** upgraded) | `crates/mycelium-vsa/src/bsc.rs:155` + `matrix.rs:45` A3-06/C1-04 basis comment: the `Proven` is the literature's *operation-level, on-expectation* tag, citing RFC-0003. No tag upgrade. |
| A3-07 | FIXED | `crates/mycelium-vsa/tests/recon.rs:162` `indexed_manifests_and_weak_retrievals_refuse_explicitly`; `EmptyCodebook`/`CodebookMismatch` variants present in `src/recon.rs`, `src/lib.rs`, `src/resonator.rs`. |
| A3-08 | FIXED (comment-pinned) | `crates/mycelium-vsa/tests/capacity_validation.rs:84-86` A3-08 tightness comment — the Clarkson/Thomas bound is sufficient-only; tamper-protection is the pinned-constant unit test. Disposition is a *documented test-scope note*, not a new behavioral test. |
| A3-09 | FIXED | `crates/mycelium-vsa/src/bsc.rs:309` `even_count_ties_copy_the_first_operand_a3_09`. **Executed: 2 passed.** |
| A3-10 | FIXED (documented limit) | `crates/mycelium-vsa/src/hrr.rs:42` `HRR_UNBIND_PROFILE` + doc records the 2k-trial resolution limit; used in `tests/empirical_profiles.rs:197`. Disposition documents the thinness, does not raise to 10k (would be a perf/scope decision). |

### WS4 — L1 soundness

| Finding | Verdict | Evidence (`file:line`) |
|---|---|---|
| A4-03 | FIXED (semantics pinned) | `crates/mycelium-l1/src/eval.rs:1079` `deeply_nested_expression_trips_the_depth_guard_without_any_recursive_call`. |
| A4-04 | FIXED | `crates/mycelium-interp/tests/golden.rs:630` `malformed_swap_meta_surfaces_as_wf_not_a_panic` + `:604` `fuel_exhaustion_at_depth_is_reported_not_a_hang`. (Disposition lives in `mycelium-interp`, not `mycelium-l1` — the `EvalError::Wf` assert path is exercised there.) |
| reject-corpus | FIXED (M-653) | `crates/mycelium-l1/tests/conformance.rs:172` `reject_expected_table_has_no_orphaned_entries` (bidirectional integrity) + per-file expected-substring table. |

### WS5 — selection / packing integrity

| Finding | Verdict | Evidence (`file:line`) |
|---|---|---|
| A5-02 | FIXED | `crates/mycelium-select/tests/packing.rs:149` `a_trit_packed_layout_for_a_non_ternary_source_is_refused`; `SelectError::NonTernarySource` at `src/lib.rs:563,848`. |
| A5-03 | FIXED | `crates/mycelium-mlir/src/pack.rs:432` `a_short_buffer_is_an_explicit_error_not_a_silent_truncation`; `PackError::BufferTooShort` at `:50,250`. **Executed: 1 passed.** |
| A5-05 | FIXED | `crates/mycelium-dense/tests/rounding_bound.rs:54-98` asserts `successes >= total - total/100` (kills the vacuous `else { continue }`). **Executed: 2 passed.** |
| A5-06 | FIXED | `crates/mycelium-dense/tests/rounding_bound.rs:3-8` corrected header: products/scale exact, sums ≤ ~2⁻⁵³. |
| A5-07 | FIXED | `crates/mycelium-dense/src/lib.rs:456` `op_rel_eps_constants_match_their_cited_formulas`. |
| A5-08 | FIXED — **citation discrepancy noted** | The fix is present but the 06-ledger's citation `mycelium-dense/lib.rs::bits_per_element(Tl2)=1.67` is imprecise. Actual symbol is `packing_bits_per_element` in **`crates/mycelium-select/src/lib.rs:310`** (TL2 = 1.67 b/w), with the codec realization at **`crates/mycelium-mlir/src/pack.rs:22-24,387`** ("A5-08 — resolved (M-360 real-layout increment)"; the `1.67 b/w` codec now matches the selector cost model) and the `policy_ref()` recompute note at `mycelium-select/src/lib.rs:637`. `mycelium-dense` has no `bits_per_element`/`1.67`. Verdict FIXED; flag the citation crate/symbol fix-up for the maintainer. |

### WS6 — harness / LSP never-silent

| Finding | Verdict | Evidence (`file:line`) |
|---|---|---|
| A6-05 | FIXED | `crates/mycelium-lsp/tests/scripted_client.rs:176` `unsupported_swap_pair_is_surfaced_not_silent`. |
| A6-10/B2-04 | FIXED | `experiments/tests/test_kc2.py:45` `test_default_baseline_checker_refuses_untrusted_exec`; `allow_untrusted` defaults `False` (`experiments/mycelium_experiments/kc2/checkers.py:134-138`, `runner.py:75`). |
| A6-11 | FIXED | `xtask/src/kc4.rs:62` `assert_validated` helper + prechecks at `:128,155,183` (structural — xtask is a runner, no unit-test surface). |

## Tests executed (grounding the "passes, not just present" claim)

- `cargo test -p mycelium-core --test serde_roundtrip wire_spellings_are_pinned` → 1 passed (A6-03).
- `cargo test -p mycelium-vsa --lib even_count_ties_copy` → 2 passed (A3-09, + mapb sibling).
- `cargo test -p mycelium-dense --test rounding_bound` → 2 passed (A5-05/A5-06).
- `cargo test -p mycelium-mlir --lib a_short_buffer_is_an_explicit_error` → 1 passed (A5-03).

A representative subset was run (one per workstream where a Rust unit/integration target exists), not
the full 25 — the remaining citations were verified *present by name* in the current tree. A full
`cargo test` across all affected crates is the maintainer's pre-ratification confirmation step.

## Disposition tally

**25 finding-ids · 25 FIXED · 0 DEFERRED · 0 N-A · 0 FLAG-FOR-MAINTAINER (verdict).**

One **citation FLAG** (not a verdict change): A5-08's prior 06-ledger citation points at the wrong
crate/symbol (`mycelium-dense` vs the real `mycelium-select`/`mycelium-mlir`); the fix itself is
present and correct.

## Honest caveats for the maintainer

1. **This is a DRAFT.** Gate A2 ("every Medium resolved or explicitly deferred") requires the
   maintainer's sign-off (ADR-021 §6). This agent verifies the *technical* basis; it does not ratify.
2. **A3-05 is `Declared`-locally.** The LiquidHaskell discharge cannot be re-run here (cabal/z3
   absent). The README run-log is the checkable artifact; the header is reconciled text only.
3. **A3-08 / A3-10 dispositions are documentation/scope notes**, not new behavioral assertions —
   honest "Fixed (documented)" rather than "Fixed (new test)". The maintainer may judge whether the
   documented scope is acceptable for 1.0.0 or wants a behavioral test added (a 1.0.1 candidate).
4. **A5-08 citation fix-up** (above) should be corrected in the 06-ledger / future records.
5. The prior 06-ledger already claimed these Fixed; this record *re-grounds* that claim against the
   live tree so Gate A2 can be closed on verified evidence rather than an unverified prior assertion.
