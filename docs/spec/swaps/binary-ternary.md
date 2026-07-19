# Binary ↔ Ternary canonical encoding (M-012)

| Field | Value |
|---|---|
| **Task** | M-012 ([#7](https://github.com/tzervas/mycelium/issues/7)) · P1 · spec |
| **Status** | **Ratified** (2026-06-09) — precise `enc`/`dec` for the canonical widths + a worked example |
| **Normative source** | RFC-0002 §4 (bijection semantics) and §5 (legal-pair table); T2.1 (IOTA TIP-5; Douglas W. Jones) |
| **Implements / proves** | implementable by M-120 (#18); provable by M-121 (#19) |
| **Contract** | the swap emits a `Bijective` [`swap-certificate.schema.json`](../schemas/swap-certificate.schema.json) |

This pins the **precise** encode/decode that RFC-0002 §4 specifies in prose: digit semantics,
chosen widths, range, rounding, and out-of-range behaviour — at the level M-120 can implement and
M-121 can machine-check. It is the only genuinely **bijective/provable** swap class (`LosslessWithinRange`,
Exact within range); everything else is bounded/probabilistic (RFC-0002 §5).

## 1. Digit semantics (balanced ternary)

A **trit** is a digit in `{−1, 0, +1}` (written `−, 0, +`). An `m`-trit balanced-ternary number with
digits `t_{m−1} … t_0` (most-significant first) denotes the integer

```
value(t) = Σ_{i=0}^{m−1} t_i · 3^i ,   t_i ∈ {−1, 0, +1}.
```

Properties used downstream (RFC-0002 §4; Knuth 4.1):

- **Symmetric range.** An `m`-trit value lies in `T_m = [ −(3^m − 1)/2 , +(3^m − 1)/2 ]`.
- **Negation = digit-wise sign flip.** `value(−t) = −value(t)` where `(−t)_i = −t_i`. (No two's-complement
  asymmetry — balanced ternary is exactly symmetric.)
- **Rounding ≡ truncation.** Dropping the low trits rounds to nearest (ties impossible); relevant
  only when *scaling* reals, which this integer-exact bijection does not do — noted for M-12x reuse,
  out of scope here.

## 2. Domains and the legality condition

- **Binary side `Bin_n`:** an `n`-bit **two's-complement** integer, range `B_n = [ −2^{n−1}, 2^{n−1} − 1 ]`.
- **Ternary side `Tern_m`:** `m` balanced trits, range `T_m` (above).

The pair `(n, m)` is **legal for a lossless swap** iff every binary value is representable in ternary:

```
B_n ⊆ T_m   ⇔   2^{n−1} ≤ (3^m − 1)/2 .
```

When this fails the pair is a **type error** (RFC-0002 §5), not a `Declared` gamble. When it holds,
`Tern_m` is strictly larger (`|T_m| = 3^m > 2^n = |B_n|`), so a **total** bijection is impossible
(RFC-0002 §4) and the inverse is **partial** — defined only on the image. Hence `LosslessWithinRange`.

## 3. `enc` and `dec` (normative)

```
enc : Bin_n → Tern_m            -- total on B_n (given a legal pair)
dec : Tern_m → Option Bin_n     -- partial: defined only on enc's image
```

**`enc(b)`** — let `v = value_twos_complement(b) ∈ B_n`. Emit the unique `m`-trit balanced
representation of `v` (algorithm below). Total and well-defined because `B_n ⊆ T_m`.

**`dec(t)`** — let `v = value(t) ∈ T_m`.

- if `v ∈ B_n` → `Some( twos_complement_n(v) )`;
- else → `None` — an **explicit** out-of-range result, **never silent** (SC-3; RFC-0002 §4).

### 3.1 Balanced-ternary digit extraction (the `enc` core)

For non-negative `v` (encode `−v` by flipping all output trits, §1):

```
for i in 0 .. m−1:
    r ← v mod 3
    if r == 2:  t_i ← −1 ;  v ← (v + 1) / 3      # borrow: 2 ≡ −1 (mod 3), carry up
    else:       t_i ←  r ;  v ←  v / 3
assert v == 0                                     # guaranteed when value ∈ T_m
```

This yields the unique balanced expansion (Knuth). `dec`'s value computation is the Horner form of
`value(t)`; the optional `twos_complement_n` re-encodes the integer into `n` bits.

## 4. Correctness properties (the M-121 proof obligations)

For any legal `(n, m)`:

- **(P1) Left-inverse / injectivity (round-trip).** `∀ b ∈ Bin_n. dec(enc b) = Some b`.
- **(P2) Partial right-inverse on the image.** `∀ t ∈ Tern_m. dec t = Some b ⟹ enc b = t`.
- **(P3) Exactness.** Within range the swap is `guarantee = Exact`, `bound = None` (M-I1).
- **(P4) Never silent.** Out-of-range decode is `None`/error, never a coerced value (SC-3).

P1/P2 are **SMT-dischargeable for fixed widths** and provable by `decide`/computation in Coq
(RFC-0002 §4); M-121 (#19) supplies the machine-checked artifact referenced by `proof_ref`.

## 5. Canonical width and worked example: `n = 8`, `m = 6`

Legality: `2^{7} = 128 ≤ (3^6 − 1)/2 = 364` ✓. So `B_8 = [−128, 127] ⊆ T_6 = [−364, 364]`.
(IOTA TIP-5 groups trits 6-per-byte, T2.1; this is the byte-aligned canonical instance.)

**Encode the byte `0b1011_0010`:**

1. Two's-complement value: `0b1011_0010 = 178 (unsigned) = 178 − 256 = −78`.
2. Balanced-ternary of `78` (LSB-first via §3.1): `[0, +1, 0, 0, −1, 0]`
   (check: `1·3 + (−1)·81 = 3 − 81 = −78`… computed on `78`: `(−1)·81 + (+1)·3 = −78` after the sign step).
3. Negate (since `v < 0`): flip every trit → MSB-first `⟨0, −1, 0, 0, +1, 0⟩`.
   Value: `(−1)·3^4 + (+1)·3^1 = −81 + 3 = −78` ✓.

So `enc(0b1011_0010) = ⟨0, −1, 0, 0, +1, 0⟩` (MSB-first), `guarantee = Exact`.

**Decode it back:** `value(⟨0,−1,0,0,+1,0⟩) = −78 ∈ [−128,127]` → `Some(twos_complement_8(−78)) = Some(0b1011_0010)`. Round-trip holds (P1).

**Out-of-range decode (P4):** the all-`+` 6-trit value `⟨+,+,+,+,+,+⟩ = 364 ∉ B_8` → `dec = None`
(explicit; `364` exceeds the 8-bit signed range). The swap that produced such a ternary value cannot
be inverted into `Bin_8` and says so — never a silent wrap to `−128`/`108`.

## 6. Certificate & guarantee

A binary↔ternary swap over a legal `(n, m)` emits a **`Bijective`** `SwapCertificate` referencing the
once-per-`(n,m)` round-trip lemma (P1/P2) by content hash, with `params = {width: n, trits: m}`
(RFC-0002 §3). `guarantee = Exact`, `bound = None`. Out-of-range is the `Option`/error path, not a
certificate.

## Amendment — 2026-07-18: W-1 binary width canon corrective (append-only)

**Status of this amendment.** Captured 2026-07-18 from the maintainer's binding corrective
(`docs/planning/design-steer-2026-07-17/PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §2, "Corrective
W-1"). This section **amends by addition**, not by rewrite — §1-§6 above (the M-012 `8↔6` spec, ratified
2026-06-09) stand unchanged and remain a valid, retained worked example. Implementation (the sweep listed
in §2.2 item 5 of the corrective) is **Phase-C work** (the program handoff's §5 wave W-D) and is
**pending** — this section records the corrected canon; it does not itself land code. The doc's own
`Status` field above is left unchanged (**Ratified**, 2026-06-09): this is an additive capture of a
binding steer, not an independent re-ratification.

### A.1 The canon corrects: 8/6 was never a ratified default

§1-§6 above document the `n=8, m=6` pair as a **worked example** of the general `enc`/`dec` construction
— it was never a `Default`/literal-grammar canon. The maintainer's 2026-07-17 repository audit (recorded
in the program handoff §2.1) confirms no literal grammar or `impl Default for Repr` exists anywhere in
the tree for any width; the 8-bit canon was **de facto**, established by which pairs the stdlib happened
to export and which literals appeared at non-test kernel/runtime sites, not by any ratified default. The
corrective below replaces that de facto canon; it does not revise the M-012 proof obligations (§4 P1-P4
above), which hold for **every** legal `(n, m)` pair, `8↔6` included.

### A.2 Canonical width

**`Binary{64}` is the canonical width** wherever a width must be assumed, exemplified, or exported as the
primary form; **`Binary{32}`** is the recognized common fallback. `Binary{8}`/`Ternary{6}` and
`Binary{4}`/`Ternary{3}` are **demoted to embedded profiles and test vectors** — retained (§5's worked
example above stays valid and useful for small-width verification), but no longer the assumed default.
**All widths remain first-class and supported** — the corrective narrows which width is *assumed* when
none is stated; it does not narrow which widths are legal (the legality condition of §2 above is
unchanged and applies at every width).

### A.3 Canonical bijection pairs

Two pairs are canonical going forward, both `LosslessWithinRange` per RFC-0002 §4 exactly as §2-§6 above
already establish (the proof shape is unchanged; only the featured widths change):

- **`Binary{32} ↔ Ternary{21}`** — legal per §2's condition (`2^(n−1) ≤ (3^m−1)/2`): `binary{32}` needs
  21 trits (`3^20 < 2^32 ≤ 3^21`, per the program handoff §2.1), which already fits within the kernel's
  `i64`-exact conversion-utility ceiling of `m ≤ 40` — **available now**.
- **`Binary{64} ↔ Ternary{41}`** — legal per the same §2 condition: `binary{64}` needs 41 trits
  (`3^40 < 2^64 ≤ 3^41`, per the program handoff §2.1), but **behind enablement item E-W1** (§A.5 below).
  The kernel's arbitrary-width ternary *arithmetic* has no ceiling (`BigTernary`, landed, Exact,
  never-overflows — M-756, per RFC-0033/ADR-029), but the narrower **conversion utilities**
  (`int_to_trits`/`trits_to_int`/`max_magnitude`) still route through `i64` and return `None` at
  `m ≥ 41` (`crates/mycelium-std-ternary/src/arithmetic.rs:43-70`,
  `crates/mycelium-core/src/ternary/mod.rs:108-131` — both already never-silent about the ceiling, per
  their own doc comments; this is a narrower enablement gap than the arithmetic core, not a bug). So
  `Binary{64} ↔ Ternary{41}` is **legal but not yet constructible via the existing conversion-utility
  fast path** until E-W1 lands.

### A.4 Length/count canon (pointer)

Length- and count-typed returns across the swap surface (for example a row-count or byte-count return)
standardize on `Binary{64}` under this corrective, for `usize` parity with the transpiler's
`u64`/`usize` → width-64 mapping. The specific site-list fix (including `lib/std/swap.myc`'s
`matrix_len`/`bytes_len` mismatch) is recorded in `docs/spec/stdlib/swap.md`'s companion 2026-07-18
amendment, not restated here — this spec does not own the length-typed surface.

### A.4b Landed follow-up (2026-07-19, append-only — supersedes the "pending" framing above)

The E-W1 enablement and the §A canonical-pair sweep **have landed** (course-correction W-D,
2026-07-18): `mycelium-core::ternary` and `mycelium-std-ternary::arithmetic` widened `i64` → `i128`
(new ceiling m ≤ 80; m ≥ 81 explicit `None`), making `Binary{64}↔Ternary{41}` constructible via
the conversion-utility fast path, and `lib/std/swap.myc` gained the `bin64_to_tern41`/
`tern41_to_bin64` (and 32/21) exports, `myc check`-clean and exercised end-to-end against the live
Rust oracle. M-1119 is `done` with a landed-basis note in `tools/github/issues.yaml`. The
"pending"/"behind E-W1" wording in §A.3/§A.5 above is retained as the capture-time record; this
note is the landed status (same pattern as `docs/spec/stdlib/ternary.md`'s 2026-07-18 amendment).

### A.5 Enablement item E-W1 (tracked separately)

Lifting the `int_to_trits`/`trits_to_int`/`max_magnitude` `i64` ceiling (routing `m > 40` through `i128`
or the existing `BigTernary` path) is tracked as its own work item — see `docs/spec/stdlib/swap.md`'s
companion amendment for the tracking-id proposal (**M-1119**, proposed to the integrating parent and not
filed in `tools/github/issues.yaml` by this capture — `issues.yaml` is orchestrator-owned). `M-758`
(`PackedTernary`, the limbed perf path) remains YAGNI/benchmark-gated and is **not** activated by this
corrective (the program handoff §2.2 item 4 is explicit on this point).

### A.6 Sweep-list pointer

The full non-test-site sweep (kernel `Repr::Binary { width: 8 }` literals across
`mycelium-interp`/`mycelium-mlir`/`mycelium-std-spore`/`mycelium-std-content`/`mycelium-std-select`/
`mycelium-lsp`/`mycelium-core`, plus `docs/lib-index/INDEX.md` regeneration) is enumerated in
`PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §2.1/§2.2 item 5 — this spec does not duplicate that list;
that document is the tracking source for the Phase-C sweep.

## Meta — changelog

- **2026-06-09 (ratified):** initial precise `enc`/`dec` spec for the canonical `8↔6` width, with
  legality condition, the four correctness obligations (M-121), and a worked round-trip + out-of-range
  example. Grounded in RFC-0002 §4/§5 and T2.1 (IOTA TIP-5 / Jones). Append-only henceforth.
- **2026-07-18 — W-1 corrective captured (Amendment, append-only; see §A above).** Records the
  maintainer's binding width-canon corrective: `Binary{64}` canonical (`Binary{32}` recognized
  fallback); `8↔6`/`4↔3` demoted to embedded profiles and retained test vectors (§1-§6 above unchanged);
  canonical bijection pairs `Binary{32}↔Ternary{21}` (available now) and `Binary{64}↔Ternary{41}` (behind
  enablement item E-W1, proposed as **M-1119**); both stay `LosslessWithinRange` per RFC-0002 §4 with the
  same §4 proof obligations. No implementation lands with this capture (Phase-C, pending). `Status` field
  unchanged (**Ratified**, 2026-06-09) — this is an additive amendment, not a re-ratification.
