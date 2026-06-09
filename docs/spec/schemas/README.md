# Core data-contract schemas (M-010)

The machine-readable contracts the build compiles against — a faithful JSON-Schema
(draft 2020-12) projection of the Accepted design corpus. **These schemas add no new design;**
each is a 1:1 rendering of a cited RFC/ADR section. Where JSON Schema cannot express an
invariant, the schema records it in a `$comment` with a pointer to the Phase-1 code check that
enforces it — never a silent gap (the honesty rule applied to the contracts themselves).

Ratified per **M-010** ([#5](https://github.com/tzervas/mycelium/issues/5)); see
`docs/planning/phase-0.md` §3/§6.1 for the plan and the canonical-set derivation.

## The set (10)

| `<name>.schema.json` | Models | Source |
|---|---|---|
| `repr` | `Repr` (Binary/Ternary/Dense/VSA); closed kinds, open registries | RFC-0001 §4.1 |
| `value` | `Value = {repr, payload, meta}`; the self-describing wire form | RFC-0001 §4.2, §4.8 |
| `meta` | `Meta` (7 fields) + invariants **M-I1…M-I4** encoded as conditionals | RFC-0001 §4.3 |
| `guarantee` | `GuaranteeStrength` lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` | RFC-0001 §3.4, §4.7 |
| `bound` | `Bound` + `BoundBasis` (Proven/Empirical/Declared basis) | RFC-0001 §4.3 (r2); ADR-010; ADR-011 |
| `provenance` | `Provenance` content-hash derivation DAG | RFC-0001 §4.6 |
| `physical-layout` | `PhysicalLayout` + `PackScheme` (the schedule *record*) | RFC-0001 §4.1/§4.3; DN-01; RFC-0004 §5 |
| `swap-certificate` | `SwapCertificate` (`Bijective` \| `Bounded`); never silent | RFC-0002 §3–§5 |
| `policy` | `SelectionPolicy` + `PolicyRef` + mandatory EXPLAIN trace | RFC-0005 |
| `reconstruction-manifest` | `ReconInfo` (indexed retrieval vs compositional reconstruction) | RFC-0003 §6 |

## Convention (enforced by `scripts/checks/schema.sh`)

```
docs/spec/schemas/<name>.schema.json
docs/spec/schemas/examples/<name>/valid/*.json     # MUST validate
docs/spec/schemas/examples/<name>/invalid/*.json   # MUST NOT validate
```

`just schema` (= `scripts/checks/schema.sh`) checks every schema against the draft-2020-12
metaschema and runs each example through its schema. Invalid examples are chosen to exercise each
schema's **honesty-load-bearing** constraint (e.g. a `Declared` value claiming a `ProvenThm`
basis, or an `Exact` value carrying a `bound`, both *fail* — proving M-I1/M-I4 bite).

Each schema carries an `x-mycelium` block recording `status: ratified`, the `source` section, and
`ratified_against`/`ratified_on` so the ratification basis is inspectable.

## What CI checks vs. what code checks

JSON Schema validates *shape* and the *single-instance* invariants (M-I1…M-I4 guarantee↔bound
consistency, enum membership, required fields, discriminated unions). It **cannot** express the
cross-value / behavioural invariants, which are enforced by Phase-1 code and noted in each
schema's `$comment`:

- **WF4 / content-addressing purity** → M-103.
- **M-I5 lossless `physical`** (relates `physical` to `payload`) → M-101/M-112.
- **meet-composition of `guarantee`** (the 4×4 lattice) → M-102.
- **provenance DAG acyclicity** → M-103.
- **legal swap-pair table** (RFC-0002 §5) → M-120/M-150.
- **policy determinism / totality** → the policy engine (later RFC + impl).

## Open questions — resolved

Three corpus clarifications were surfaced while projecting the RFCs and have since been resolved
(2026-06-09):

- **OQ-3** (`bound`) — **Resolved by ADR-011 (RFC-0001 r2).** `basis` is now a required companion
  of *every* `Bound`, not just `CapacityBound`, reconciling the §4.3 grammar with invariants
  M-I2/3/4 and RFC-0002 §3. The r1 §4.3 grammar is formally superseded.
- **OQ-4** (`bound`) — **Resolved.** `NormKind` is enumerated `L1 | L2 | Linf | Rel` as an
  extensible registry (RFC-0001 §4.3 r2), matching the `ScalarKind`/`PackScheme` treatment.
- **OQ-5** (`policy`) — **Deferred (by design), tracked.** RFC-0005 intentionally defers the
  concrete predicate grammar to a later RFC; `rules[].when` stays a declared object. Tracked by the
  Phase-2 epic **E2-6 "Selection policy + EXPLAIN"** (#33).
