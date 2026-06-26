# ADR-011 — BoundBasis is a property of every Bound

| Field | Value |
|---|---|
| **ADR** | 011 |
| **Status** | **Accepted** |
| **Date** | June 09, 2026 |
| **Supersedes / Superseded by** | Supersedes the implicit decision in **RFC-0001 r1 §4.3** that scoped `basis` to `CapacityBound` only (RFC-0001 revised to **r2** to implement this ADR). |
| **Context refs** | RFC-0001 §3.4/§4.7 (guarantee lattice), §4.3 (M-I2/M-I3/M-I4); RFC-0002 §3; ADR-010; VR-3/VR-5; **G5**. Surfaced as **OQ-3** during M-010 schema ratification (#5). |

## Context

Mycelium's honesty model makes guarantee *strength* derive from **how a bound was obtained**, not
from assertion (RFC-0001 §3.4/§4.7; ADR-010). RFC-0001's own invariants encode this for any
approximate value:

- **M-I2.** `guarantee == Proven  ⟹  bound.basis == ProvenThm{..}`
- **M-I3.** `guarantee == Empirical ⟹ bound.basis == EmpiricalFit{..}`
- **M-I4.** `guarantee == Declared ⟹ bound.basis == UserDeclared`

and RFC-0002 §3 specifies that every swap certificate carries "`Bound` (ε and/or δ) **+ BoundBasis**".

But the RFC-0001 **r1** §4.3 grammar attached `basis` to **only** the `CapacityBound` variant:

```
Bound ::= ErrorBound {eps, norm} | ProbabilityBound {delta}
        | CrosstalkBound {expected, tail?} | CapacityBound {items, dim, basis}
```

So an `ErrorBound`, `ProbabilityBound`, or `CrosstalkBound` had **nowhere to record its basis** —
which means the strength tag of every non-capacity approximate op (float-rounding ε, decode-failure
δ, crosstalk) could not be grounded in *how* it was obtained. That defeats **VR-5** / **G5** (some
bounds are proven, others only Gaussian-empirical; the disclosure must say which) for the entire
non-VSA approximate surface. The invariants and RFC-0002 §3 already assumed the corrected reading;
only the grammar lagged. M-010 schema authoring forced the question (OQ-3).

## Decision

**`BoundBasis` is a universal, required companion of every `Bound`** — a property of `Bound`
itself, not a field of one variant. Normatively:

```
Bound     ::= { kind: BoundKind, basis: BoundBasis }
BoundKind ::= ErrorBound {eps, norm} | ProbabilityBound {delta}
            | CrosstalkBound {expected, tail?} | CapacityBound {items, dim}
```

The guarantee strength derives from `basis` for **all** bound kinds, exactly as M-I2/M-I3/M-I4
state. An `Exact` value carries no `Bound` at all (M-I1), so the universality of `basis` does not
add anything to exact values.

This supersedes the r1 §4.3 grammar; RFC-0001 is revised to **r2** to carry the corrected grammar.
(The companion `NormKind` enumeration `L1|L2|Linf|Rel` made in the same r2 edit is a *registry
population*, not part of this decision — RFC-0001 §4.1 keeps parameter registries open.)

## Consequences

- **(+)** M-I2/M-I3/M-I4 are well-formed for every bound, not just capacity bounds; the ratified
  `bound.schema.json` and `meta.schema.json` (M-010) are corpus-consistent — the grammar now
  matches the contract CI enforces.
- **(+)** Every approximate value — ε, δ, crosstalk, capacity — honestly records the basis of its
  bound, so the `Proven | Empirical | Declared` tag is grounded per op (VR-5, G5).
- **(+)** ADR-010's shared `{ε, δ, strength}` certificate carries `basis` uniformly across both
  bound kernels.
- **(−)** A slightly more verbose `Bound` (an extra required field). Accepted: it is the minimum
  needed to keep the honesty tag grounded.

## Grounding

RFC-0001 §3.4/§4.7 + invariants M-I2/M-I3/M-I4; RFC-0002 §3; ADR-010 (shared certificate); VR-3,
VR-5; survey **G5**. Raised as OQ-3 during M-010 (#5).

> **Footnote — tunable certification (RFC-0034 / ADR-032, 2026-06-24; append-only).** The universal `BoundBasis` applies **within the active certification mode (`certified`)**; the `fast` (default) and `balanced` relaxations are governed by **RFC-0034** (the mode is never silent — G2). The basis machinery is **unchanged**. See **ADR-032**, which supersedes the *unconditional* reading.
