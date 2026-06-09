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

## Meta — changelog

- **2026-06-09 (ratified):** initial precise `enc`/`dec` spec for the canonical `8↔6` width, with
  legality condition, the four correctness obligations (M-121), and a worked round-trip + out-of-range
  example. Grounded in RFC-0002 §4/§5 and T2.1 (IOTA TIP-5 / Jones). Append-only henceforth.
