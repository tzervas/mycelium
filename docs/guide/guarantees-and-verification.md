# The guarantee lattice, in practice

One-line purpose: how the `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` lattice is assigned, composed,
and split-verified across swap classes.

## Contents

- [The four tags](#the-four-tags)
- [The split verification regime](#the-split-verification-regime)

## The four tags

Every operation in the kernel and standard library carries one of four guarantee tags,
never upgraded without a checked basis (VR-5):

| Tag | Meaning | When it applies |
|---|---|---|
| `Exact` | No accuracy loss; result is the mathematical ideal | Binary arithmetic, `len`, boolean ops, lossless widening |
| `Proven` | Follows from a theorem whose side-conditions are checked | Binary↔ternary bijectivity (Z3); MAP/BSC `bundle` capacity (Clarkson-Ubaru-Yang / Thomas-Dasgupta-Rosing, ratified by the LH probe) |
| `Empirical` | Validated across ≥10⁴ randomized trials; bound stated and measured | FHRR/HRR `unbind` crosstalk; float ε bounds not yet reduced to a Proven basis |
| `Declared` | User-asserted or open research prompt; always flagged | Unverified user bounds; open T3.6 retention ablation |

The lattice composes by *meet* (weakest wins) through operations, so a composed result can never
spuriously claim a stronger guarantee than its inputs. Out-of-range input is an explicit
`Result`/`Option`, never a silent clamp or fallback.

The tag is written into the function signature itself, not asserted in a comment — three
structurally similar swaps, three honestly different tags
([`examples/repr-tour/swaps.myc`](../../examples/repr-tour/swaps.myc), checked clean by
`cargo run -p mycelium-cli --bin myc -- check`):

```mycelium
fn widen(x: Binary{8}) => Ternary{6} @ Declared =
  swap(x, to: Ternary{6}, policy: roundtrip);

fn narrow(x: Dense{768, F32}) => Dense{768, BF16} @ Empirical =
  swap(x, to: Dense{768, BF16}, policy: bf16_round);

fn certified(x: Dense{768, F32}) => Dense{768, BF16} @ Proven =
  swap(x, to: Dense{768, BF16}, policy: bf16_round);
```

**Honesty note on this snippet:** it is a syntax demonstration (source: `examples/repr-tour/`'s own
header comment), not a claim about the "real" guarantee owed to each specific swap — `widen`'s
`@ Declared` here is illustrative, and is *not* the tag the binary↔ternary swap actually carries
elsewhere (that one is `Proven` bijective — see the split-verification table below). What the
snippet does demonstrate honestly: `narrow` and `certified` are the *same* BF16-rounding call at
two different assurance levels — `@ Empirical` when only trial data backs the ε bound, `@ Proven`
once the cited theorem's side-conditions are checked — and the code shape doesn't change; only the
checked basis, and its type-level tag, does.

## The split verification regime

ADR-002 splits how a swap's guarantee is established, by swap class:

| Swap class | Guarantee | Mechanism |
|---|---|---|
| binary ↔ ternary | `Proven` bijective | Round-trip proof (Z3) + property tests; `LosslessWithinRange` — `Option`-typed, never silent |
| ↔ dense embedding / VSA | `Proven` or `Empirical` bounded/probabilistic | Per-swap certificate (translation-validation model, VR-4): typed `{ε, δ, strength}` certificate |

See [Why & design](why-and-design.md) for how the lattice fits into the broader set of core
design ideas, and [Workspace map](workspace-map.md) for the crates (`mycelium-cert`,
`mycelium-numerics`) that implement it, plus the proof artifacts that back the `Proven` tags.

---

**See also:** [Why & design](why-and-design.md) · [Workspace map](workspace-map.md) ·
[How Mycelium compares](comparisons.md)

[← Back to README](../../README.md)
