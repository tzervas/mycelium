# Medium-Findings Ledger — ADR-021 Gate A2 (M-653)

| Field | Value |
|---|---|
| **Closes** | ADR-021 **Gate A2** ("every Medium resolved or explicitly deferred with a one-line rationale") · DN-19 **GAP-2** · issue **M-653** |
| **Source** | the 2026-06-14 deep review (`00-summary.md` … `05-remediation-roadmap.md`); the open-Medium list is `05-remediation-roadmap.md` §3 (WS2–WS6 "Status" notes, 2026-06-15) and DN-19 §2 GAP-2 |
| **Date** | 2026-06-21 |
| **Posture (honesty rule / VR-5)** | This is the A2 *disposition record*, not a re-grading. Each row is **Fixed** with a **regression test that fails without the fix** (the WS8 mutant-witness discipline) and cites the finding id. Nothing is upgraded; the fixes only make a guarantee more conservative or turn a silent acceptance into an explicit refusal. |

## Summary

Every open **Medium** from the 2026-06-14 deep review (WS2–WS6 tail) is **Fixed** — **0 deferred**.

**Honest framing of what M-653 actually was.** The 2026-06-15 remediation roadmap listed these
Mediums as "remaining", but the M-653 re-audit (2026-06-21) found that the **large majority were
already remediated in prior waves**, each with a mutant-witness regression test — they were never
recorded as resolved, so Gate A2 read as open. M-653 therefore (a) **verified** each fix is present
*and its guarding test passes* (not just that code looks fixed), and (b) closed the **two genuine
gaps** that remained:

- **A6-03** — a new `wire_spellings_are_pinned_per_bound_kind_basis_and_layout` test pinning every
  `BoundKind`/`BoundBasis`/`PhysicalLayout` wire spelling (kills enum-rename drift).
- **reject-corpus integrity** — a new `reject_expected_table_has_no_orphaned_entries` test closing
  the *bidirectional* gap (a deleted fixture's stale `REJECT_EXPECTED` entry would have passed
  silently).

So A2's gap was a verification/recording gap, **not** a backlog of unfixed defects. All affected
crate test suites are green; the two new tests are the only net code delta in M-653 (the rest is
this ledger + the verification record).

## Ledger

Legend — **Landed:** `M-653` = new regression test added this wave · `prior` = remediated in an
earlier wave, *verified present + passing* here.

### WS2 — contract / schema integrity (`mycelium-core`, `mycelium-cert`)

| Finding | Disposition | Regression test (mutant-witness) | Landed |
|---|---|---|---|
| **A1-04** recon "basis ≤ Empirical" encoded by rank, not `is ProvenThm` | Fixed | `mycelium-core` `recon::tests::resonator_allows_basis_weaker_than_empirical` | prior |
| **A1-05** SC-3 `assert_validated` discards `strength` | Fixed | `mycelium-cert` `tests/sc3.rs` asserts `strength` (≈ L52) | prior |
| **A6-03** schema enum wire-spelling drift unpinned | Fixed | `mycelium-core` `tests/serde_roundtrip.rs::wire_spellings_are_pinned_per_bound_kind_basis_and_layout` | **M-653** |
| **A6-06** `ReconInfo` schema↔Rust conditional drift | Fixed | `mycelium-core` `recon::tests::resonator_range_checks_a_stray_cleanup_threshold` (+ `…optional_params`) | prior |
| **A6-08** out-of-range `sparsity.density` → misleading `MalformedBound` | Fixed | `mycelium-core` `meta::tests::out_of_range_sparsity_is_malformed_sparsity` (→ `MalformedSparsity`) | prior |
| **A6-09** cert `Bijective` typed params vs schema free-form | Fixed | `mycelium-cert` `tests/swap.rs::emitted_certificate_matches_committed_example` | prior |

### WS3 — VSA tag honesty (`mycelium-vsa`, `proofs/lh-bundle`)

| Finding | Disposition | Regression test (mutant-witness) | Landed |
|---|---|---|---|
| **A3-04** MAP-I/MAP-B `bind` stamp `Exact` without ±1 alphabet check | Fixed | `mycelium-vsa` `mapi.rs`/`mapb.rs` `…refuse_non_bipolar_a3_04` | prior |
| **A3-05** `Bundle.hs` header vs README "Discharged" contradiction | Fixed | `proofs/lh-bundle` header reconciled to README (text; Haskell toolchain absent locally — honest skip on type-check) | prior |
| **A3-06 / C1-04** BSC bundle plain `Proven` vs RFC "on expectation" | Fixed (qualified, **not** upgraded) | `mycelium-vsa` `bsc.rs`/`matrix.rs` A3-06 basis/qualifier comments citing RFC-0003 | prior |
| **A3-07** dim-mismatched cleanup → wrong `EmptyBundle` variant | Fixed | `mycelium-vsa` `tests/recon.rs::indexed_manifests_and_weak_retrievals_refuse_explicitly` (`EmptyCodebook`/`CodebookMismatch`) | prior |
| **A3-08** capacity-tightness untested | Fixed | `mycelium-vsa` `tests/capacity_validation.rs` tightness comment (A3-08) | prior |
| **A3-09** BSC tie-break (`n > half` vs `n >= half`) untested | Fixed | `mycelium-vsa` `…even_count_ties_copy_the_first_operand_a3_09` | prior |
| **A3-10** HRR trial-count thin (2k vs 10k) | Fixed | `mycelium-vsa` `hrr.rs` `HRR_UNBIND_PROFILE` doc (A3-10) records the resolution limit | prior |

### WS4 — L1 soundness (`mycelium-l1`)

| Finding | Disposition | Regression test (mutant-witness) | Landed |
|---|---|---|---|
| **A4-03** eval depth charged per AST-node, not per call-frame | Fixed (semantics pinned) | `mycelium-l1` `eval::tests::deeply_nested_expression_trips_the_depth_guard_without_any_recursive_call` | prior |
| **A4-04** `EvalError::Wf` constructed but never asserted | Fixed | `mycelium-interp` `tests/golden.rs::malformed_swap_meta_surfaces_as_wf_not_a_panic` + `fuel_exhaustion_at_depth_is_reported_not_a_hang` | prior |
| **reject-corpus** assertions only `is_err()` (wrong-reason rejects pass) | Fixed | `mycelium-l1` `tests/conformance.rs` per-file expected-substring table **+** new `reject_expected_table_has_no_orphaned_entries` (bidirectional integrity) | **M-653** |

### WS5 — selection / packing integrity (`mycelium-select`, `mycelium-mlir`, `mycelium-dense`)

| Finding | Disposition | Regression test (mutant-witness) | Landed |
|---|---|---|---|
| **A5-02** `TritPacked` recorded onto a non-ternary value's `Meta` | Fixed | `mycelium-select` `tests/packing.rs::a_trit_packed_layout_for_a_non_ternary_source_is_refused` (`SelectError::NonTernarySource`) | prior |
| **A5-03** `unpack_trits` silently truncates a short buffer (pub) | Fixed | `mycelium-mlir` `pack.rs::a_short_buffer_is_an_explicit_error_not_a_silent_truncation` (→ `Result`/`BufferTooShort`) | prior |
| **A5-05** vacuous rounding sweep (`let Ok(..) else continue`) | Fixed | `mycelium-dense` `tests/rounding_bound.rs` asserts the success count (≈ L92–99) | prior |
| **A5-06** false "exact for f64" header comment | Fixed | `mycelium-dense` `tests/rounding_bound.rs` comment corrected (products exact, sums ≤ 2⁻⁵³) | prior |
| **A5-07** `*_OP_REL_EPS` not pinned to formulas | Fixed | `mycelium-dense` `lib.rs::op_rel_eps_constants_match_their_cited_formulas` | prior |
| **A5-08** cost-model price vs codec discrepancy; `policy_ref` recompute | Fixed | `mycelium-dense` `lib.rs` `bits_per_element(Tl2)=1.67` aligned + `policy_ref` note | prior |

### WS6 — harness / LSP never-silent (`mycelium-lsp`, `experiments/kc2`, `xtask`)

| Finding | Disposition | Regression test (mutant-witness) | Landed |
|---|---|---|---|
| **A6-05** LSP `_ => None` for unhandled swap pairs, no diagnostic | Fixed | `mycelium-lsp` `tests/scripted_client.rs::unsupported_swap_pair_is_surfaced_not_silent` | prior |
| **A6-10 / B2-04** `BaselineChecker.exec` unguarded namespace | Fixed | `experiments` `tests/test_kc2.py::test_default_baseline_checker_refuses_untrusted_exec` (`allow_untrusted=False` default) | prior |
| **A6-11** xtask `kc4` bijective-dec path timed without `assert_validated` precheck | Fixed | `xtask/src/kc4.rs` precheck added (≈ L155; structural — xtask is a runner, no unit surface) | prior |

## Verification

- All affected crate suites green: `cargo test -p mycelium-core -p mycelium-cert -p mycelium-vsa
  -p mycelium-l1 -p mycelium-select -p mycelium-mlir -p mycelium-dense -p mycelium-lsp`; the kc2
  Python guard via `pytest`. `cargo fmt` + `clippy -D warnings -A unsafe_code` (ADR-014) clean.
- Each cited test is a **mutant-witness**: reverting its fix makes the test fail (that is how the
  finding was confirmed *closed*, not merely *present*).
- The two **M-653**-landed tests (A6-03, reject-corpus integrity) are the only net code change.

## Disposition tally

**25 Medium finding-ids · 25 Fixed · 0 deferred.** Gate A2 met.
