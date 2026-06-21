# Selection-Explain Memory — Reified Selection Policies and EXPLAIN

**Normative sources:**
- RFC-0005 (Accepted) `docs/rfcs/RFC-0005-Selection-Policy-Language.md`
- ADR-006 (reified policies, not tracked in separate ADR file — folded into RFC-0005)
- RFC-0010 (Accepted) `docs/rfcs/RFC-0010-Decode-Methodology-Selection.md`

---

## What it is

A reified, inspectable, content-addressed decision-table mechanism for automatic representation
selection. The core principle: no black boxes (G2). Every selection (swap target, packing scheme,
decode methodology) is made by an explicit `SelectionPolicy`, emits a mandatory `Explanation`
trace, is deterministic, and is always answerable by content hash.

Not a learned optimizer, not a heuristic, not opaque. The expressiveness ceiling (non-Turing-
complete predicates) is the feature, not a limitation (RFC-0005 §2).

---

## Where it lives

- **Crate:** `crates/mycelium-select/src/lib.rs` (standalone, depends on `mycelium-core` only;
  KC-3 / SoC — outside the trusted kernel)
- **VSA decode integration:** `crates/mycelium-vsa/src/decode_select.rs`

Key exports from `mycelium-select`:
- `SelectionPolicy` — the reified decision table
- `SelectionInputs` — queryable projection of `(Repr, Meta)` + optional `DecodeFacts`
- `Predicate` — the non-Turing-complete predicate language
- `Explanation` — the mandatory EXPLAIN record
- `select()` — the single entry point (returns `(Candidate, Explanation)`)
- `explain()` — EXPLAIN without executing (total, deterministic)
- `select_swap_target()` — site adapter for RFC-0002 swap targets
- `select_packing()` — site adapter for RFC-0004 packing schedules
- `select_decode_method()` — site adapter for RFC-0010 decode methodology
- `PolicyRegistry` — resolves `PolicyRef` -> `SelectionPolicy`
- `CostModel`, `Candidate`, `Action`, `Rule`, `PolicyError`, `SelectError`

---

## Key types

### `SelectionPolicy` (`lib.rs:399`)

Fields (private, access via methods):
- `name: String` — display name (part of content-addressed identity)
- `candidates: Vec<Candidate>` — finite enumerable set
- `rules: Vec<Rule>` — ordered decision table (first match wins)
- `default_choice: usize` — mandatory default arm (totality guarantee)
- `cost: CostModel` — explicit cost function

Constructor `SelectionPolicy::new()` validates up front: non-empty candidates, every
`Choose(i)` and default arm in range, finite positive cost weight, finite float literals in
predicates. Constructed policy is TOTAL by construction. Deserialization re-validates (`lib.rs:417`).

`policy_ref() -> ContentHash` — SHA256 of canonical JSON serialization (`lib.rs:497`).
Records in `Meta.policy_used` so "which policy chose this?" is always answerable.

### `SelectionInputs` (`lib.rs:88`)

Queryable projection drawn from `(Repr, Meta)`:
- `src: Repr`, `guarantee: GuaranteeStrength`, `bound: Option<Bound>`, `sparsity: Option<SparsityObs>`
- `decode: Option<DecodeFacts>` — RFC-0010 decode site only

Key: inputs are **exact metadata** (proven/declared bounds, dtype, sparsity class), never sampled
estimates. This avoids the cardinality-estimation opacity of database cost-based optimizers (RFC-0005 §2).

`from_meta(src, meta)` — swap/packing sites; `with_decode(facts)` — decode site.

### `DecodeFacts` (`lib.rs:73`)

- `factors: u32` — F (number of codebooks)
- `capacity: u128` — `prod_i k_i` (saturating)
- `dim: u32` — hypervector dimension d
- `in_regime: bool` — `MAPI_RESONATOR_PROFILE.check()` result (a fact, not a sampled estimate)

### `Predicate` (`lib.rs:134`) — non-Turing-complete, total, terminating

- `Always`, `SrcKindIs(ParadigmKind)`, `DtypeIs(ScalarKind)`, `GuaranteeAtLeast(GuaranteeStrength)`
- `ErrorEpsAtMost(f64)` — bound.kind must be Error with eps <= x
- `DeclaredSparse` — `Vsa` with `SparsityClass::Sparse`
- `CapacityAtMost(u128)` — decode site only (`prod_k <= x`)
- `FactorsAtMost(u32)` — decode site only
- `InResonatorRegime` — decode site only (the `in_regime` fact)
- `All(Vec<Predicate>)`, `Any(Vec<Predicate>)`, `Not(Box<Predicate>)`

`eval(inputs)` — structural recursion on finite data, always terminates, always returns bool.
`literals_finite()` — validates no NaN/Inf in `ErrorEpsAtMost` (prevents policy-ref collisions, A5-01).

### `Explanation` (`lib.rs:516`) — mandatory EXPLAIN record

- `policy: ContentHash` — the policy that decided
- `policy_name: String`
- `inputs: SelectionInputs` — what was considered
- `costs: Vec<CandidateCost>` — every candidate with explicit cost
- `matched_rule: Option<usize>` — which rule fired (None = default arm or override)
- `chosen_index: usize`, `chosen: Candidate`
- `overridden: bool` — whether a forced override bypassed the table

### `CostModel` (`lib.rs:281`)

Explicit storage-bits footprint — real declared units, not arbitrary internal units:
- `Repr::Binary{w}` = w bits
- `Repr::Ternary{t}` = 2t bits (DN-01 two-bit-per-trit reference)
- `Repr::Dense{dim, dtype}` = dim * dtype_bits
- `Repr::Vsa` dense = dim x 64 (f64); sparse = max\_active x 96 (32-bit index + f64 value)
- `Packing` = bits/element * source element count
- `Decode` site: `BruteForceExact` = capacity (grows with prod_k); `Resonator`/`Refuse` = 0

---

## The three RFC-0005 sites (one mechanism)

**"One mechanism, two (now three) sites" — no parallel selectors (DRY/SoC). RFC-0005 §4.**

1. **Swap target** (`select_swap_target`) — RFC-0002: which `Repr` to swap to. Candidate must
   be `Candidate::Repr(r)`; wrong kind -> `SelectError::WrongSiteKind`.
2. **Packing schedule** (`select_packing` / `select_layout` / `record_packing_layout`) — RFC-0004
   §5: which `PackScheme` for ternary lowering. Candidate must be `Candidate::Packing(s)`.
   Non-ternary source -> `SelectError::NonTernarySource` (A5-02). Consumed by E2-7/M-250.
   Default: `bitnet_packing_policy()` — I2S/TL1/TL2, `Always -> Cheapest` (TL2 wins at 1.67 b/w).
3. **Decode methodology** (`select_decode_method`) — RFC-0010: `BruteForceExact | Resonator | Refuse`.
   Candidate must be `Candidate::Decode(m)`. Implemented in `decode_select.rs`.

---

## EXPLAIN trace and the no-black-boxes rule (G2, SC-5)

Every selection emits an `Explanation` — no selection without one. `select()` always returns
`(Candidate, Explanation)`. `explain(policy, inputs)` returns the trace without side effects
(total, no Result — un-overridden selection on a validated policy cannot fail).

The trace answers: what inputs were considered, what each candidate cost, which rule fired, what
was chosen, whether it was overridden.

**LSP channel (SC-5):** `explain_decode_method()` in `decode_select.rs:127` exposes the mandatory
EXPLAIN for decode choices to the LSP surface ("why was this decode method chosen?").

`PolicyRegistry` (`lib.rs:726`) resolves a recorded `PolicyRef` back to the policy — so the trace
plus the registry fully reconstructs the decision. Tooling consumes this.

---

## Decode methodology selection (RFC-0010)

`decode_method_policy(enum_budget)` (`decode_select.rs:70`) — three candidates:

```text
[BruteForceExact, Resonator, Refuse]
Rule 1: CapacityAtMost(enum_budget) -> BruteForceExact   (prefer Exact while tractable)
Rule 2: InResonatorRegime           -> Resonator          (Empirical, in-regime only)
Default:                            -> Refuse             (explicit, never silent — G2)
```

`DEFAULT_ENUM_BUDGET = MAPI_RESONATOR_PROFILE.max_capacity = 4096` (`decode_select.rs:53`).
Guarantee-maximal default: every in-regime request gets the stronger Exact brute-force decode.

**Honesty floor (RFC-0010 §4.5):** a forced override CANNOT escape:
- Forced `BruteForceExact` with `prod_k > enum_budget` -> `DecodeRefused`
- Forced `BruteForceExact` on non-identifiable instance -> `NonIdentifiable` (no Exact claim if
  the true tuple is not the unique global arg-max)
- Forced `Resonator` outside regime -> `OutsideEmpiricalProfile` / `DecodeRefused`
Tag is never upgraded (VR-5).

`DecodeSelection` result (`decode_select.rs:141`) carries:
- `method: DecodeMethod`, `explanation: Explanation`
- `factors: Vec<Match>` — per-slot recovered atoms
- `guarantee: GuaranteeStrength` — read off the chosen arm (Exact for brute-force, Empirical for
  resonator); never asserted independently (RFC-0010 §4.4 / VR-5)
- `resonator_trace: Option<ResonatorTrace>` — EXPLAIN on the loop too

---

## Never-silent error surface

`PolicyError` (construction-time, `lib.rs:351`): `NoCandidates | IndexOutOfRange | BadCost | BadPredicateLiteral`

`SelectError` (call-time, `lib.rs:539`): `OverrideOutOfRange | WrongSiteKind | NonTernarySource`

All explicit. No silent fallbacks. No coercions. A policy that cannot be constructed is
rejected up front — the construction errors are exhaustive (G2).

---

## Gotchas

- **`policy_ref()` recomputed on every `select()` call** (perf nit A5-08, `lib.rs:636`):
  a full serialize + hash. Acceptable at current scale; memoize if `select()` lands on a hot path.
- **Non-ternary source at packing site** -> `NonTernarySource` (A5-02). A `TritPacked` layout
  only describes how trits sit in bytes; recording it on Binary/Dense/Vsa is a silent mis-tag.
- **Predicate float literals must be finite** — `ErrorEpsAtMost(NaN)` and `ErrorEpsAtMost(+Inf)`
  both serialize to JSON `null`, creating a policy-ref collision (A5-01). `BadPredicateLiteral`.
- **`Action::Cheapest` tie-break** is deterministic: lowest candidate index wins (`lib.rs:652`).
- **The decode site cost for Decode candidates is operational-count, not storage bits** — brute
  force cost = `capacity` (so it is chosen only when small); resonator/Refuse = 0 (`lib.rs:339-347`).
  The decode table decides by explicit predicate arms (not Cheapest), so the cost lines in EXPLAIN
  are informational only for that site.
- **`Deserialize` re-validates** `SelectionPolicy` (`lib.rs:417`) — wire data is never silently
  trusted; the same totality contract holds for deserialized policies.

---

## Read more

- `docs/rfcs/RFC-0005-Selection-Policy-Language.md` — §2 design (EXPLAIN, determinism, override),
  §3 reification/content-addressing, §4 scope (one mechanism, three sites)
- `docs/rfcs/RFC-0010-Decode-Methodology-Selection.md` — §4 decode table + honesty floor
- `crates/mycelium-select/src/lib.rs` — complete implementation (single file)
- `crates/mycelium-vsa/src/decode_select.rs` — decode site integration + brute-force oracle
