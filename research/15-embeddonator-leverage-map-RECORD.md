# Embeddonator Lift-and-Shift Map

> **Mycelium recording note (2026-06-24).** This is a **recorded research input** — an
> external Grok + Claude value-model research bundle (2026-06-23), preserved here as the
> evidence base for **RFC-0033** and **ADR-025…031** (all landing **Proposed**). It is not
> itself normative. Four corpus corrections apply when reading it:
> 1. **Paths re-mapped to the real tree.** There is no `crates/mycelium-value`; the trusted
>    arbitrary-width ternary lands in **`crates/mycelium-core/src/ternary/`** (reconciled
>    against the existing `Trit` + `core::ternary` M-111 codec — no duplicate `Trit`, and
>    `Limb27`/`PackedTernary` are an explicit YAGNI follow-on). Tasks `VM-NNN` map to
>    canonical **M-754…M-784** under epic **E20-1** (see `docs/planning/value-model-*`).
> 2. **The "silent precision ceiling" is `embeddonator`'s, not Mycelium's.** Mycelium's
>    `core::ternary` is **already never-silent** about the 40-trit cap (`max_magnitude`
>    returns `None` at m ≥ 41; `add`/`mul` return `None` on overflow). The new `BigTernary`
>    **removes the cap by adding a growable path** — it does not fix a silent bug in
>    Mycelium's own code.
> 3. **The RFC-0001 §4.3 `bound.basis` amendment reconciles with the existing, ratified
>    `ADR-011 — BoundBasis-Is-Universal`** — it is not asserted fresh. (That reconciliation
>    is value-model phase V5 / M-781, not landed here.)
> 4. **Collections (`Seq`/`Bytes`) overlap `RFC-0032` / `E19-1`**, which already scopes
>    `Repr::Seq`/`Repr::Bytes`; RFC-0033 references and aligns with it rather than
>    re-deciding it.

**Purpose:** exact inventory of what to extract from the `embeddonator` crates into
Mycelium, where it goes, what to change, and what to leave behind. All sources MIT,
© Tyler Zervas. Commits pinned: `embeddonator-vsa @ b8adac78de76d8216656b77c419903093b9d0d69`,
`embeddonator-core @ 5bf1e7ef58505462cf494c65898760a1ea9323f4`.

## 0. Crate map (what's where)

| Crate | Rust LOC | Relevant content |
|---|---|---|
| `embeddonator` | 0 | empty meta-repo (no code) |
| `embeddonator-core` | ~26.9k | nested vendored copy under `crates/embeddonator/`; the exhaustive trit tests |
| `embeddonator-vsa` | ~18k | **the ternary + VSA implementations** (primary lift source) |
| `embeddonator-interop` | ~2.1k | kernel interop glue (low relevance) |
| `embeddonator-io` | ~3k | IO/profiles (low relevance) |
| `embeddonator-retrieval` | ~6.6k | resonator/search (VSA-adjacent, not value-model) |

> **Hazard:** `embeddonator-core/crates/embeddonator/` duplicates much of
> `embeddonator-vsa`. Treat `embeddonator-vsa @ b8adac78` as canonical for the lift;
> confirm before pulling anything not listed here so a stale fork isn't re-vendored.

## 1. Extraction table

| What | Source (file : lines) | Destination | Action | Liftability |
|---|---|---|---|---|
| `Trit` digit algebra (`mul`, `add_with_carry`, `neg/abs/sign`, `from_i8_exact`, `from_bits`/`to_bits`) | `embeddonator-vsa/src/ternary.rs:47–249` | `crates/mycelium-core/src/ternary/trit.rs` | **DONE** (vendored + adapted: serde optional, `core::`, contract docs) | near-verbatim; algorithms unchanged |
| Base-27 limb (`Tryte3`) + `balanced_mod`/`balanced_div` | `ternary.rs:258–595` | `…/ternary/limb.rs` (`Tryte3`→`Limb27`) | **DONE** (renamed, `i16`→`i32`, VSA-only methods dropped, packing marked layout-only) | near-verbatim |
| Two-limb word (`Word6`) | `ternary.rs:479–571` | (not lifted) | reference only — `{low,high}` is the template for the limbed bignum | — |
| Arbitrary-width ternary **integer** arithmetic | **none upstream** | `…/ternary/big_ternary.rs` | **DONE** (net-new; algorithm-validated 10k fuzz) | n/a — fills the gap |
| **Exhaustive `Trit` tests** (full 27-row `add_with_carry` truth table as `const`) | `embeddonator-core/tests/exhaustive_trit_tests.rs:1–…` (29 `#[test]`) | `crates/mycelium-value/tests/ternary/exhaustive_trit.rs` | **TODO** port — gives instant proof the lifted primitives are correct | direct port |
| **Block-sparse ternary** (`Block{pos,neg}` over 64; `BlockSparseTritVec`; `try_new`/`validate` never-silent; bind/bundle) | `embeddonator-vsa/src/block_sparse.rs:47–520` (11 `#[test]` + proptest in `…/invariants/block_sparse_invariants.rs`) | `crates/mycelium-vsa/src/block_sparse.rs` (VSA paradigm, NOT the ternary-integer crate) | **TODO** lift for B-VSA `SparseBlock` | high; this is the reference impl for the `SparseBlock` verdict |
| **SIMD dense ternary HV** (`PackedTritVec`: 2-bit/trit, AVX2 + AVX-512 VPOPCNTDQ `dot`/`bind`/`bundle`) | `embeddonator-vsa/src/ternary_vec.rs:67–1042` | `crates/mycelium-vsa-accel/` (UNTRUSTED acceleration layer) | **TODO** lift as VSA dense path / layout candidate | high, but `unsafe`+`target_feature` → keep OUT of KC-3 |
| Variable-width-but-**i64-capped** tryte (the cautionary example) | `embeddonator-vsa/src/dimensional.rs:115–230` | — | **DO NOT LIFT** | the bug `big_ternary.rs` replaces |

## 2. The gap, precisely

There is **no arbitrary-width balanced-ternary integer arithmetic** in any
embeddonator crate. The only variable-width path is `dimensional.rs::Tryte`:

```rust
// embeddonator-vsa/src/dimensional.rs
pub fn from_i64(mut value: i64, num_trits: usize) -> Self { /* value /= 3 loop */ }
pub fn max_value(num_trits: usize) -> i64 {
    (3i64.pow(num_trits as u32) - 1) / 2     // 3i64.pow(41) OVERFLOWS i64
}
```

For `num_trits ≥ 41`, `3i64.pow(…)` overflows — panic in debug, **silent wrap in
release**. So the current variable-width tryte doesn't merely *cap* at 40 trits, it
*silently corrupts* above it. This is the concrete justification for the B-Ternary
"arbitrary-width now" verdict, and `big_ternary.rs` fixes it: the growable path grows
(`3^41` is exact, width 42), and the fixed-width boundary is never-silent (`None` on
overflow). Validated against this exact `3^41` case.

## 3. Why the primitive lift is trustworthy

- `add_with_carry` is exhaustively tested upstream as a **`const` 27-row truth table**
  in `exhaustive_trit_tests.rs` — every `(a,b,carry_in) → (sum,carry)` is pinned.
- `block_sparse.rs` has unit tests **plus** property tests
  (`invariants/block_sparse_invariants.rs`).
- The primitives are `const fn`, total, and never-silent (`from_i8_exact`,
  `Limb27::from_i8`, `unpack`, `from_bits` all return `Option`). They drop straight into
  a minimal trusted kernel with the serde dependency made optional.

## 4. Boundaries to respect (KC-3 hygiene)

1. **Integer vs VSA separation (SoC).** Trit-wise `mul`/`bundle`/`dot` are *VSA bind/
   bundle/similarity*, not integer arithmetic. The integer limb (`limb.rs`) drops them;
   they live in the VSA crate. The shared atom is `Trit` itself.
2. **`unsafe` SIMD stays untrusted.** `PackedTritVec`'s value is its AVX paths. Isolate
   in `mycelium-vsa-accel` (property-tested, but the kernel never depends on it for
   correctness). The kernel keeps the scalar `const fn` primitives.
3. **Packing is layout, not repr (DN-01).** `Limb27::pack`/`unpack` and the 2-bit/trit
   `PackedTritVec` encoding are `physical-layout` concerns. The kernel stores `[Trit]`;
   the packer is downstream. Note the two crates use *different* packings (3 trits/byte
   vs 2 bits/trit-for-SIMD) — that's fine, choose per layer; neither is the 5-trits/byte
   density optimum, which is yet another valid layout choice.

## 5. Suggested commit sequence (conventional commits)

1. `feat(value-model): vendor balanced-ternary trit + limb primitives from embeddonator-vsa`
   — `trit.rs`, `limb.rs`, `mod.rs`, `PROVENANCE.md`.
2. `test(value-model): port exhaustive trit truth-table tests from embeddonator-core`
   — `tests/ternary/exhaustive_trit.rs`.
3. `feat(value-model): arbitrary-width balanced-ternary integer arithmetic`
   — `big_ternary.rs` + its tests (never-silent fixed-width boundary).
4. `feat(vsa): lift block-sparse ternary hypervector (BSDC) from embeddonator-vsa`
   — `mycelium-vsa/src/block_sparse.rs` (+ proptest).
5. `perf(vsa-accel): lift SIMD PackedTritVec into untrusted acceleration layer`
   — `mycelium-vsa-accel/` (isolated `unsafe`).
