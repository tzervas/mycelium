# Design Note DN-52 — Parsable-vs-Runnable Census (the DN-50 M-807 deliverable)

| Field | Value |
|---|---|
| **Note** | DN-52 |
| **Status** | **Accepted (census complete — 2026-06-27).** All known accepted language constructs classified; no SILENT-GAP class found. The narrow standing-gate design is recommended in §4. |
| **Feeds** | DN-50 (frames the question; OQ-1/OQ-2 ratified 2026-06-27); M-807 (the audit work item); M-719 (conformance over the generic surface — one slice of this census). |
| **Date** | June 27, 2026 |
| **Decides** | (a) Fixes the "runnable" definition (per DN-50 OQ-1): three-way L1-eval ≡ L0-interp ≡ AOT on at least one instantiation. (b) Classifies every accepted language construct as Runs / Explicit-Residual / SILENT-GAP. (c) Recommends the narrow never-silent gate (OQ-2). No SILENT-GAP constructs were found; the never-silent floor holds. |
| **Task** | M-807 |

> **Posture (transparency rule / VR-5 / G2).** This note is **Empirical** — classification is
> based on a read of `crates/mycelium-l1/src/elab.rs`, `src/mono.rs`, `src/checkty.rs`, and
> the test corpus (`tests/differential.rs`, `tests/enablement.rs`, `tests/width_generic.rs`,
> `tests/conformance.rs`, `tests/check.rs`, `tests/phylum.rs`, `tests/std_generic_conformance.rs`).
> Evidence is cited per row. Claims are `Empirical` (read-based, cross-checked against tests);
> no claim is `Proven` (no formal proof). Undetermined rows are flagged, not assumed.

---

## §1 — The "runnable" definition (OQ-1, now fixed)

**DN-50 OQ-1 ruling (2026-06-27):** a construct is **runnable** when it elaborates to a closed
L0 term AND **executes three-way** (L1-eval ≡ elaborate→L0-interp ≡ AOT) on at least one
instantiation — the existing `differential.rs` bar. No new machinery; the three-way harness is
already the standard.

The three classification buckets:

1. **Runs** — elaborates to closed L0; the three-way differential passes (cite: test + path in
   `differential.rs` / `enablement.rs` / `width_generic.rs`).
2. **Explicit-Residual** — accepted by the checker but the elaborator / mono emits an explicit
   `ElabError::Residual` (cite: `elab.rs:line` or `mono.rs:line`). This is the honest, never-silent
   state for staged constructs.
3. **SILENT-GAP** — accepted by the checker but neither runs three-way NOR hits an explicit
   `Residual`. A G2 violation and the dangerous class. None were found (see §3).

---

## §2 — Census table

### 2.1 Representation types and literals

| Construct | Bucket | Evidence |
|---|---|---|
| `Binary{N}` literal (`0b…`) — bare | **Runs** | `differential.rs::corpus()` program #0; `lit_value` elab.rs:84 |
| `Ternary{N}` literal (`<…>`) | **Runs** | `differential.rs::corpus()` programs #4/5 |
| Unary/binary bit ops (`not`, `xor`, `and`, `or`) | **Runs** | `differential.rs::corpus()` programs #2/3; `enablement.rs` M-748 |
| Ternary arithmetic (`add`, `sub`, `mul`) | **Runs** | `differential.rs::corpus()` programs #4/5 |
| Binary arithmetic (`add_bin`, `sub_bin`) | **Runs** | `enablement.rs` M-748 three-way |
| Comparison/equality (`eq`, `lt`) over Binary/Ternary | **Runs** | `enablement.rs::eq_binary_width_typed`, `lt_binary_unsigned_magnitude` |
| Infix/prefix operator sugar (`^`, `!`, `+`, `*`) | **Runs** | `differential.rs::corpus()` programs #6-9 (RFC-0025/M-705 sugar-word equivalence) |
| `swap(x, to: T, policy: p)` — certified swap | **Runs** | `differential.rs::corpus()` program #10 (`Binary` to `Ternary`) |
| `Dense{d, s}` type accepted by checker | **Explicit-Residual** — `elaborate` emits an explicit `Residual` for Dense swap targets (`elab.rs` Expr::Swap arm, freeze-ledger W5): `BinaryTernarySwapEngine` does not cover Dense; a Dense-capable engine lands with E2-1/ADR-033. FLAG-1 **RESOLVED** in §5. | `differential.rs::dense_swap_is_an_explicit_residual_on_all_paths`; `runnable_gate.rs` standing-gate row; `elab.rs` Expr::Swap guard added W5 |
| `Seq{T, N}` / `[e1, …]` literal | **Runs** | `enablement.rs::seq_literal_surface_three_way` (M-749 surface, full three-way) |
| `seq_get` / `seq_len` over surface | **Runs** | `enablement.rs::seq_get_surface_three_way`, `seq_len_surface_three_way` |
| `Bytes` / `0x…` literal | **Runs** | `enablement.rs::bytes_literal_surface_three_way` (M-750 surface, full three-way) |
| `bytes_get` / `bytes_len` / `bytes_slice` / `bytes_concat` over surface | **Runs** | `enablement.rs::bytes_get_surface_three_way`, `bytes_len_surface_three_way` |
| `VSA{…}` types | **Explicit-Residual** | `elab.rs:206` — `residual(site, "VSA types are deferred in the L1 v0 prototype")`; checker accepts (`checkty.rs:566`), elaborator refuses explicitly |
| `Substrate{tag}` types | **Explicit-Residual** | `elab.rs:216` — `"Substrate is not a representation type"`; gated at swap check in `checkty.rs:2352` |
| Width-variable reaching elab (`Binary{N}` where N is unresolved) | **Explicit-Residual** | `elab.rs:182/190` — refuses with `"width variable reached elaboration"` (DN-42/M-753) |

### 2.2 Data types, ADTs, and match

| Construct | Bucket | Evidence |
|---|---|---|
| `type T = C1 \| C2(…)` ADT, flat match | **Runs** | `differential.rs::data_corpus()` programs #0-2; `l1_eval_l0_interp_and_aot_agree_on_the_data_and_recursion_fragment` |
| Multi-field constructor (`Mk(T1, T2)`) | **Runs** | `data_corpus()` program #7 (`Pair = Mk(Bool, Bool)`) |
| Nested patterns (Maranget decision tree) | **Runs** | `data_corpus()` programs #2/7; `check.rs::nested_pattern_match_typechecks` |
| Literal-pattern match (`match b { 0b0000 => … }`) | **Runs** | `data_corpus()` program #3 |
| `if` desugaring to `Bool` match | **Runs** | `data_corpus()` program #5; `check.rs` |
| Constructed result with repr field | **Runs** | `data_corpus()` programs #6/4 |
| `for` fold over a list spine (desugars to `Fix` fold) | **Runs** | `data_corpus()` program #10 (`ByteList`); `elab.rs:29` documents the desugar |
| Recovery (`match r { Ok(v) => …, Err(e) => … }`) | **Runs** | `differential.rs::recovery_match_over_a_result_sum_agrees_three_ways` (M-352/RFC-0014) |
| Data type with `Substrate` field (used) | **Explicit-Residual** | `elab.rs:569-591` — registry build skips substrate-typed fields; any use fails explicitly via `DataRegistry::build` |

### 2.3 Functions and recursion

| Construct | Bucket | Evidence |
|---|---|---|
| Simple fn call (acyclic, inlining) | **Runs** | `differential.rs::corpus()` program #11 (`flip(flip(…))`) |
| Self-recursion (`Total` classified) | **Runs** | `differential.rs::self_recursion_elaborates_and_agrees` (r4 closed); `data_corpus()` programs #8/9 |
| Mutual recursion (2-fn, `FixGroup`) | **Runs** | `differential.rs::mutual_recursion_elaborates_and_all_three_paths_agree` (M-343/R7-Q3) |
| Mutual recursion (3-fn cycle, data-building) | **Runs** | `data_corpus()` programs #13/14/15 |
| `Partial`-classified recursion (divergent, fuel-guarded) | **Runs** (L1 only, `FuelExhausted` explicit) / **Explicit-Residual** (elab path) | `differential.rs::a_partial_program_exhausts_fuel_explicitly`; elab.rs:307 refuses Partial body explicitly |
| `thaw` modifier in `matured` scope | **Runs** | `check.rs::structural_recursion_is_total_and_gates_matured` — `thaw` allows non-total fns; the body itself still follows Partial/Total split |

### 2.4 Generics (type parameters) and width generics

| Construct | Bucket | Evidence |
|---|---|---|
| Generic type parameter (`List<A>`, `fn first_or<A>`) | **Runs** | `differential.rs::generic_corpus()` programs #1-7; M-673 three-way harness |
| Generic at user-data type arg (`Box<Bit>`) | **Runs** | `generic_corpus()` program #7 (M-673 injectivity fix) |
| Width-generic free fn (`fn id_bits<N>(x: Binary{N}) -> Binary{N}`) | **Runs** | `width_generic.rs::width_generic_identity_binary_8/16`; `width_generic_identity_ternary_3/6` (M-753/DN-42) |
| Width-generic at two distinct widths (multi-width conformance) | **Runs** | `std_generic_conformance.rs` M-719 table (>=2 widths per op) |
| Undetermined width parameter (not inferable) | **Explicit-Residual** | `checkty.rs` refuses at check time; `mono.rs:430-442` also refuses with `"width param … is still a variable"` |
| Multi-parameter trait (v2, deferred) | **Explicit-Residual** | `mono.rs:1315` — `"multi-parameter traits are v2 — RFC-0019 §10"`; `checkty.rs:2919` checker also refuses |
| Associated types | **Explicit-Residual** | `mono.rs:43/140` — listed explicitly in the "still Residual" class |
| Generic fn as HOF arg (still generic, no type-arg context) | **Explicit-Residual** | `mono.rs:1477-1487` — `"is still generic — deferred (RFC-0024 §5)"` |

### 2.5 Single-parameter traits and impls

| Construct | Bucket | Evidence |
|---|---|---|
| Trait def + impl, direct method call (static resolution) | **Runs** | `generic_corpus()` program #3; M-673 three-way harness |
| Bounded generic (`fn use_cmp<T: Cmp>(a: T, b: T)`) | **Runs** | `generic_corpus()` program #4 |
| Width-typed trait + impl (Binary, Ternary; check-only for Dense) | **Runs** (Binary/Ternary) / **Undetermined** (Dense three-way) | `check.rs:1031` coherence sweep; see FLAG-1 in §5 for Dense |
| Cross-nodule trait impl (phylum orphan rule) | **Runs** (check verified) | `phylum.rs` orphan rule tests; elaboration of cross-nodule programs not three-way tested — FLAG-2 in §5 |

### 2.6 HOF / defunctionalization (RFC-0024 §4)

| Construct | Bucket | Evidence |
|---|---|---|
| Named fn as HOF arg to `map`/`and_then`/`fold` over Result | **Runs** | `differential.rs::hof_corpus()` programs #1-6; `l1_eval_l0_interp_and_aot_agree_on_hof_via_defunctionalization` (M-688) |
| Closure / lambda literal as HOF arg | **Explicit-Residual** | `mono.rs:1430-1438` — `"closures / dynamic fn values are deferred (RFC-0024 §5)"` |
| Multi-segment dotted path as fn value | **Explicit-Residual** | `mono.rs:1440-1448` — `"dotted path … only top-level names are first-class"` |
| Recursive HOF combinator re-passing `f` in self-call (M-715 scope) | **Runs** | `mono.rs:1451` M-715 re-pass; `hof_corpus()` exercises recursive Result combinators three-way |

### 2.7 Effect annotations

| Construct | Bucket | Evidence |
|---|---|---|
| Effect annotation `!{eff}` on fn signature | **Runs** | `check.rs::an_effect_annotated_fn_parses_and_checks`; annotation erased at elaboration (elab.rs:5/371/860/883 — "a grade has no L0 form") |
| Effect coverage check (under-declaration refused) | **Runs** (checker) | `check.rs::an_unannotated_caller_of_an_effectful_fn_is_a_check_error` |
| Effect budget ledger threading (RFC-0014 §4.8) | **Runs** | `differential.rs::the_effect_ledger_is_meaning_preserving_on_the_recovery_match` (M-353) |
| `ffi` effect + `wild` body (effectful FFI fn) | **Runs** (with host op) | `differential.rs::wild_ffi_execution_agrees_three_ways` (M-720/M-721) |

### 2.8 `wild` / FFI

| Construct | Bucket | Evidence |
|---|---|---|
| `wild { name(args…) }` in `@std-sys` nodule — host-call form | **Runs** (with capability) | `differential.rs::wild_ffi_execution_agrees_three_ways` — three-way with mock `wild:echo` (M-720/M-721) |
| `wild` body not in host-call form | **Explicit-Residual** | `differential.rs::a_wild_body_that_is_not_a_host_call_form_is_an_explicit_residual`; elab.rs:915 |
| `wild` with ungranted host op | **Explicit-Residual** (runtime) | `differential.rs::an_ungranted_wild_host_op_is_an_explicit_refusal` — L0-interp and L1-eval refuse explicitly |

### 2.9 Grading (`@ g`)

| Construct | Bucket | Evidence |
|---|---|---|
| `@ Exact`, `@ Proven`, `@ Declared`, `@ Empirical` ascriptions | **Runs** | Grade is statically checked by `crate::grade` + **erased** at elaboration (elab.rs:5/371/860/883/1219/1329 — "a grade has no L0 form — KC-3"). Body runs unchanged. `check.rs:1402` covers `@ Proven` on a certified swap fn. |

### 2.10 Colonies and hyphae (`colony { hypha … }`)

| Construct | Bucket | Evidence |
|---|---|---|
| Single-hypha colony | **Runs** | `differential.rs::corpus()` program #14; RT2 corpus |
| Multi-hypha colony (sequential reference ≡ concurrent RT2) | **Runs** | `differential.rs::colony_concurrent_run_equals_the_sequential_reference_rt2`; `prop_colony_concurrent_value_is_its_last_hypha_for_any_leading_count` (M-666) |
| Colony hypha driving recursive fn | **Runs** | RT2 corpus program #3 |
| Colony hypha involving a swap | **Runs** | RT2 corpus program #4 |
| Orphan `hypha` outside a colony | **Rejected by parser** | `conformance.rs::REJECT_EXPECTED` "13-orphan-hypha.myc" |

### 2.11 Phyla and cross-nodule constructs

| Construct | Bucket | Evidence |
|---|---|---|
| Phylum header + multiple nodules (parse + check) | **Runs** (check) | `phylum.rs::a_phylum_header_with_two_nodules_parses_into_two_nodule_blocks` |
| Cross-`use` of `pub fn` / `pub type` across nodules | **Runs** (check) | `phylum.rs::nodule_b_uses_a_pub_fn_from_nodule_a_and_type_checks` |
| Cross-nodule `impl` (phylum orphan rule) | **Runs** (check) | `phylum.rs` orphan rule tests |
| Phylum-of-one (bare nodule) | **Runs** | `phylum.rs::a_header_less_single_nodule_is_a_phylum_of_one` |
| Full three-way over cross-nodule program | **Runs** — `elaborate(env_b, "main")` on nodule B's merged env finds `helper` from A (imported into `fns` by `check_nodule_with`); all three paths agree. FLAG-2 **RESOLVED** in §5. | `differential.rs::cross_nodule_program_runs_three_way` (W5/freeze-ledger) |

### 2.12 `spore` deployable artifact

| Construct | Bucket | Evidence |
|---|---|---|
| `spore(…)` expression | **Explicit-Residual** | `elab.rs:879` — `residual(site, "\`spore\` is deferred (E2-5/M-260)")` |

### 2.13 Reserved-but-inactive surface

| Construct | Bucket | Evidence |
|---|---|---|
| `consume` keyword | **Rejected by parser** | `conformance.rs::REJECT_EXPECTED` "18-consume-reserved-not-active.myc" |
| `grow` keyword | **Rejected by parser** | `conformance.rs::REJECT_EXPECTED` "19-grow-reserved-not-active.myc" |
| Runtime vocab outside legal surface | **Rejected by parser** | `conformance.rs::REJECT_EXPECTED` "12-runtime-vocab-reserved-not-active.myc" |
| `impl` as identifier | **Rejected by parser** | `conformance.rs::REJECT_EXPECTED` "14-impl-reserved-ident.myc" |

---

## §3 — SILENT-GAP verdict

**No SILENT-GAP constructs were found.**

Every construct accepted by the Mycelium checker (`checkty.rs`) either:

- Runs three-way (elaborates to closed L0; L1-eval ≡ L0-interp ≡ AOT); OR
- Hits an explicit `ElabError::Residual` in `elab.rs` or `mono.rs` with a human-readable
  `what` string naming the staging reason; OR
- Is rejected by the parser before reaching the checker (reserved-not-active surface).

The explicit `Residual` discipline — applied at both the generic/mono pre-pass (`mono.rs`)
and the elaboration pass (`elab.rs`) — appears consistently enforced. The never-silent policy
(G2) holds at the implementation frontier as observed in this audit.

**Qualification (VR-5 / Empirical):** this is a code-read + test-corpus audit, not a formal
proof. The two flagged undetermined cases (§5) are not current SILENT-GAP risks but should be
verified before the corpus that covers them is extended.

---

## §4 — Narrow standing-gate design (DN-50 OQ-2 recommendation)

**Goal:** forbid SILENT-GAP constructs — enforce "checker accepts ⇒ either runs three-way OR
hits an explicit `Residual`." This is NOT a must-run gate; staged constructs with explicit
`Residual`s are clean.

### Gate structure

A test over the accept corpus (`docs/spec/grammar/conformance/accept/`) that drives each
fixture through the full pipeline and asserts one of two clean outcomes:

```
for each src in accept_corpus:
    env = check_nodule(parse(src))   // gate 1 already verifies this
    result = elaborate(env, main)
    assert result.is_ok()            // RUNS: elaborated to closed L0
        || result.is_err_and(|e| matches!(e, ElabError::Residual { .. }))
        // EXPLICIT-RESIDUAL: honest staging, never a panic or UnknownFn
```

The assertion fails if `elaborate` returns any error other than `ElabError::Residual{..}`
on a checker-accepted, corpus-member program.

### Placement

A new test function `accept_corpus_all_elaborate_or_explicit_residual` in
`crates/mycelium-l1/tests/conformance.rs` — alongside the existing `accept_corpus_all_parses`
and `reject_corpus_all_fails_explicitly` tests. The corpus directory and the `myc_files`/
`corpus_dir` helpers are already there.

### Sketch

```rust
#[test]
fn accept_corpus_all_elaborate_or_explicit_residual() {
    use mycelium_l1::{check_nodule, elaborate, parse};
    for path in myc_files("accept") {
        let src = std::fs::read_to_string(&path).unwrap();
        let env = check_nodule(&parse(&src)
            .unwrap_or_else(|e| panic!("{}: must parse: {e}", path.display())))
            .unwrap_or_else(|e| panic!("{}: must check: {e}", path.display()));
        // Skip fixtures with no `main` (grammar-only specimens).
        if env.fn_decl("main").is_none() { continue; }
        let result = elaborate(&env, "main");
        assert!(
            result.is_ok()
                || matches!(result, Err(mycelium_l1::elab::ElabError::Residual { .. })),
            "{}: checker-accepted program must elaborate OR hit an explicit Residual \
             (SILENT-GAP detected — G2/DN-52 §4); got: {:?}",
            path.display(), result.err()
        );
    }
}
```

**Scope note (honest — VR-5):** the accept corpus covers grammar-level fixtures. A broader gate
over the full range of `check_nodule`-accepted programs (including fixtures from `check.rs` and
`std_generic_conformance.rs`) would require a richer corpus or a programmatic fixture generator.
The narrow scope above is correct and actionable immediately.

---

## §5 — FLAGs for the orchestrator

**FLAG-1 (Dense three-way) — RESOLVED (W5/freeze-ledger, 2026-06-27):** `Dense{d, s}` swap
targets were `Undetermined` because `elaborate` returned `Ok(Node::Swap{Dense})` while every
runner (`BinaryTernarySwapEngine`, AOT) refused explicitly — an elaboration-level gap in the
DN-50 narrow gate. Fix (freeze-ledger W5): the `Expr::Swap` elaboration arm in `elab.rs` now
checks `if matches!(target_repr, Repr::Dense { .. }) { return residual(…) }`, making all paths
consistent. Classification: **Explicit-Residual** (a Dense-capable swap engine lifts this with
E2-1/ADR-033). Evidence: `Empirical` — `differential.rs::dense_swap_is_an_explicit_residual_on_all_paths`
and the `runnable_gate.rs` standing-gate row.

**FLAG-2 (Cross-nodule three-way) — RESOLVED (W5/freeze-ledger, 2026-06-27):** Cross-nodule
programs were `Undetermined` for the differential (check-tested but not three-way tested).
Finding: `elaborate(env_b, "main")` on nodule B's merged env works correctly — `check_nodule_with`
merges imported `pub` functions into `env.fns` (lines 1223-1224 of `checkty.rs`), so `elaborate`
finds imported functions transparently. All three paths (L1-eval ≡ L0-interp ≡ AOT) agree.
Classification: **Runs**. Evidence: `Empirical` — `differential.rs::cross_nodule_program_runs_three_way`.

**FLAG-3 (Partial-recursion bi-modal status):** `Partial`-classified programs run on L1-eval
(with explicit `FuelExhausted`) and emit an explicit `Residual` on elaboration (`elab.rs:307`).
The census rows list this as "Runs (L1 only) / Explicit-Residual (elab)" for clarity. The
standing gate in §4 must account for this: `elaborate(env, "spin")` on a Partial body yields
`Err(Residual{..})`, which is the correct outcome — the gate passes.

---

## §6 — Relationship

- **DN-50** — frames the question; OQ-1/OQ-2 ruling (2026-06-27) specified the deliverable.
  This note completes DN-50's §5 Definition of Done.
- **M-807** — the audit work item completed here.
- **M-719** — generic-surface conformance slice; three-way differential tests are the primary
  evidence base for §2.4/§2.5.
- **M-673** (monomorphization), **M-688** (HOF defunctionalization), **M-343** (mutual
  recursion `FixGroup`), **M-666** (colony RT2 differential), **M-720/M-721** (`wild` FFI
  floor) — each wave's work is the evidence base for the corresponding census row.
- **M-715** (recursive-HOF re-pass) — `mono.rs:1451` closes the previously-flagged recursive
  HOF combinator gap; `hof_corpus()` confirms three-way runs.

---

## Changelog

- **2026-06-27** — Created (Accepted, census complete). `Empirical` audit of `elab.rs`,
  `mono.rs`, `checkty.rs`, and the full test corpus. No SILENT-GAP constructs found. Dense
  three-way and cross-nodule three-way gaps flagged as undetermined. Narrow standing-gate
  design sketched in §4. M-807 deliverable for DN-50 §5 DoD. Leaf agent LDW0-B.
- **2026-06-27** — FLAG-1 and FLAG-2 **RESOLVED** (W5/freeze-ledger). FLAG-1 (Dense three-way):
  elab.rs Expr::Swap arm now emits explicit `Residual` for Dense targets → classification updated
  to `Explicit-Residual`. FLAG-2 (cross-nodule three-way): new three-way differential test
  confirms cross-nodule programs **Run** (merged env works transparently). §5 updated with
  resolution evidence and test refs. The narrow standing gate is wired in
  `crates/mycelium-l1/tests/runnable_gate.rs` (DN-56 §5.1 freeze-gate condition #1). `Empirical`.
