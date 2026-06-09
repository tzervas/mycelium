# Binary↔ternary round-trip proof (M-121)

| Field | Value |
|---|---|
| **Task** | M-121 ([#19](https://github.com/tzervas/mycelium/issues/19)) · P1 · verification |
| **Status** | **Discharged (build)** — Z3 returns `unsat` on the injectivity obligation (2026-06-09) |
| **Proves** | the round-trip/injectivity lemma referenced by every `Bijective` swap certificate (M-120) |
| **Normative source** | RFC-0002 §4 (P1/P2); `docs/spec/swaps/binary-ternary.md` §4; T2.1 |
| **Discharged by** | `z3 -smt2 roundtrip_8x6.smt2` ⇒ `unsat` (Z3 4.16.0); also exhaustively decided in Rust |
| **Lemma identity** | `mycelium_cert::roundtrip_lemma_ref()` = `operation_hash("lemma.binary_ternary.roundtrip.v1")` |

## What is proved

The proof obligation (`roundtrip_8x6.smt2`) is **injectivity of the balanced-ternary value map on
`T_6`** (the SMT-dischargeable fixed-width statement RFC-0002 §4 anticipated): no two *distinct*
6-trit vectors denote the same integer. The SMT file asserts the negation (a collision) and Z3
reports `unsat` — so no collision exists.

Injectivity discharges the round-trip (P1/P2) for the canonical `n=8, m=6` pair:

- `|T_6| = 3^6 = 729` vectors map injectively into the integer range `[−364, 364]`, which has
  exactly `729` elements → the map is a **bijection** onto `[−364, 364]`.
- So every integer in `[−364, 364]` has a **unique** 6-trit representation; `enc` (the §3.1
  digit-extraction) produces it and `dec` (the value map) inverts it.
- `B_8 = [−128, 127] ⊆ [−364, 364]`, hence `enc` is total on `B_8` and `dec(enc b) = b` for every
  `b ∈ B_8` (**P1**), with `dec` a partial inverse on the image (**P2**).

**P3** (`Exact` within range, `bound = None`) and **P4** (out-of-range `dec` = `None`/error, never
silent) are structural properties of the implementation; they are additionally **decided by
exhaustive computation** in `crates/mycelium-cert/tests/swap.rs` (`roundtrip_8x6_exhaustive` walks
all 256 bytes; `out_of_range_decode_is_explicit` checks the `364 ∉ B_8` case). For this fixed,
finite width, exhaustive enumeration is itself a complete decision procedure — the "Coq `decide`"
route the issue admits — and the SMT obligation is the portable, solver-checkable form that also
generalizes the argument.

## What is *not* claimed

This proves the *fixed-width* `8↔6` instance (the canonical byte-aligned pair, T2.1). A
width-generic proof (`∀ n,m. legal(n,m) ⟹ …`) is future work; the per-`(n,m)` lemma is exactly the
"once-per-swap-kind" artifact the `Bijective` certificate references by `lemma_ref` (RFC-0002 §3),
so additional legal pairs each get their own discharged instance.

## Running

```
z3 -smt2 roundtrip_8x6.smt2      # expected: unsat
```

`scripts/checks/proofs.sh` runs this when `z3` is present and skips gracefully otherwise (the Rust
exhaustive corpus still runs in `just test`).

## Changelog

- **2026-06-09:** authored the injectivity obligation and discharged it with Z3 4.16.0 (`unsat`);
  wired into `scripts/checks/proofs.sh`. M-121.
