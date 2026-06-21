## What it is

The **Core IR value model** — the single source of truth for all Mycelium values, types, and
expressions. Specified in **RFC-0001** (Accepted, r5) and implemented in `crates/mycelium-core/`.
Honesty is not an overlay: the guarantee lattice, bound types, and metadata schema are *mechanically
enforced by construction* in Rust — an inconsistent `Meta` or `Value` cannot be built.

Status: design complete; Rust-first implementation complete (Phases 0-3 done, Foundation r4).

---

## Where it lives

| Item | Path |
|---|---|
| Crate root, public re-exports | `crates/mycelium-core/src/lib.rs` |
| `Value`, `Payload`, `Trit` | `crates/mycelium-core/src/value.rs` |
| `Repr`, `ScalarKind`, `SparsityClass` | `crates/mycelium-core/src/repr.rs` |
| `Meta`, `Provenance`, `PhysicalLayout`, `PackScheme` | `crates/mycelium-core/src/meta.rs` |
| `GuaranteeStrength`, `meet`, `propagate` | `crates/mycelium-core/src/guarantee.rs` |
| `Bound`, `BoundKind`, `BoundBasis`, `NormKind` | `crates/mycelium-core/src/bound.rs` |
| `Node`, `Alt`, `VarId`, `PolicyRef` | `crates/mycelium-core/src/node.rs` |
| `CoreValue`, `Datum` | `crates/mycelium-core/src/datum.rs` |
| `DataRegistry`, `CtorRef`, `DataDecl` | `crates/mycelium-core/src/data.rs` |
| Content-addressing (`Canon`, BLAKE3, `Names`) | `crates/mycelium-core/src/content.rs` |
| `ContentHash` | `crates/mycelium-core/src/id.rs` |
| `ReconInfo` | `crates/mycelium-core/src/recon.rs` |
| Normative spec | `docs/rfcs/RFC-0001-Core-IR-and-Metadata-Schema.md` |
| Foundation | `docs/Mycelium_Project_Foundation.md` |

---

## Key types and operations

### `Value` (`value.rs:134`)

```rust
pub struct Value {
    repr: Repr,       // paradigm + semantic params — the STATIC type
    payload: Payload, // paradigm-specific bits/trits/scalars/hypervector
    meta: Meta,       // runtime, queryable; NOT part of content identity
}
```

Built only via `Value::new(repr, payload, meta) -> Result<Self, WfError>`, which checks
`repr.well_formed()` and payload-repr agreement (paradigm + length). Deserialization re-runs
these checks; malformed wire data raises an error, never a silent default.

### `Repr` — four closed paradigm kinds (`repr.rs:57`)

```rust
pub enum Repr {
    Binary  { width: u32 },
    Ternary { trits: u32 },
    Dense   { dim: u32, dtype: ScalarKind },        // ScalarKind: F16|BF16|F32|F64
    Vsa     { model: String, dim: u32, sparsity: SparsityClass },
}
```

Wire tag: `"kind": "Binary"|"Ternary"|"Dense"|"VSA"` (note: `Vsa` in Rust, `"VSA"` on wire).
Four kinds are **closed** — a fifth requires RFC + ADR (KC-3). Parameter registries (`ScalarKind`,
`model`, `SparsityClass`) are open/extensible. `dtype` in `Dense` is identity-bearing because
precision bounds embedding error. Physical packing is **not** in the type — it is schedule-staged
and recorded in `Meta.physical` (DN-01; RFC-0001 §4.1; RFC-0004 §5).

### `Payload` (`value.rs:55`)

```rust
pub enum Payload {
    Bits(Vec<bool>),       // len == width
    Trits(Vec<Trit>),      // len == trits; Trit: Neg|Zero|Pos
    Scalars(Vec<f64>),     // len == dim (Dense)
    Hypervector(Vec<f64>), // len == dim (VSA; sparse detail deferred to VSA submodule)
}
```

### `Meta` — runtime metadata (`meta.rs:88`)

```rust
pub struct Meta {
    provenance:     Provenance,             // Root | Derived{op,inputs}; acyclic DAG
    guarantee:      GuaranteeStrength,      // Exact|Proven|Empirical|Declared
    bound:          Option<Bound>,          // None iff Exact (M-I1)
    sparsity:       Option<SparsityObs>,    // measured {active,density}; != declared SparsityClass
    physical:       Option<PhysicalLayout>, // RECORD of schedule-staged packing; not the decision
    reconstruction: Option<Box<ReconInfo>>, // RFC-0003 §6; lossy/holographic forms
    policy_used:    Option<ContentHash>,    // set iff produced by a Swap (ADR-006)
}
```

`Meta::new(...)` enforces M-I1-M-I4 by construction (no inconsistent `Meta` is buildable).
`Meta::exact(provenance)` is the fast path for `Exact` values (no bound, no physical).
Dynamic metadata is **NOT part of code identity** — excluded from content hashes (RFC-0001 §4.4/4.6).

Schema invariants (enforced by `Meta::new` and re-run on every deserialization):

- **M-I1**: `guarantee == Exact <=> bound == None`
- **M-I2**: `Proven => basis == ProvenThm`
- **M-I3**: `Empirical => basis == EmpiricalFit`
- **M-I4**: `Declared => basis == UserDeclared`; tooling MUST surface "declared, unverified"
- **M-I5**: `physical` is lossless; changing it does not change type or guarantee

### `GuaranteeStrength` (`guarantee.rs:16`)

```rust
pub enum GuaranteeStrength { Exact, Proven, Empirical, Declared }
// Lattice: Exact (rank 0) ⊐ Proven (1) ⊐ Empirical (2) ⊐ Declared (3)
```

Key functions:
- `meet(self, other)` — weakest wins; commutative, associative, idempotent, identity `Exact`
- `propagate(intrinsic, inputs)` — `meet` of intrinsic + all inputs; the **only** sanctioned way
  to derive a result strength (VR-3/VR-5); disclosure can only degrade
- Laws verified exhaustively over all 4x4(x4) tuples (complete finite check)

### `Bound` + `BoundBasis` (`bound.rs`)

```rust
pub struct Bound { pub kind: BoundKind, pub basis: BoundBasis }

pub enum BoundKind {
    Error { eps: f64, norm: NormKind },          // NormKind: L1|L2|Linf|Rel
    Probability { delta: f64 },
    Crosstalk { expected: f64, tail: Option<f64> },
    Capacity { items: u64, dim: u64 },
}
pub enum BoundBasis {
    ProvenThm { citation: String },
    EmpiricalFit { trials: u64, method: String },
    UserDeclared,
}
```

Per ADR-011, `basis` is universal — every `Bound` carries one. `BoundBasis::strength()` derives
the `GuaranteeStrength`. `Bound::well_formed()` rejects: infinite eps/crosstalk, `delta` outside
`[0,1]`, zero trials, empty citation/method.

### `Node` — Core IR grammar (`node.rs:37`, RFC-0001 §4.5 r5)

```rust
pub enum Node {
    Const(Value),
    Var(VarId),                                          // VarId = String
    Let { id: VarId, bound: Box<Node>, body: Box<Node> },
    Op  { prim: Prim, args: Vec<Node> },                 // Prim = String
    Swap { src: Box<Node>, target: Repr, policy: PolicyRef }, // ONLY repr-changing node (WF1/WF2)
    Construct { ctor: CtorRef, args: Vec<Node> },        // r3: saturated data constructor (WF6)
    Match { scrutinee: Box<Node>, alts: Vec<Alt>, default: Option<Box<Node>> }, // r3: flat (WF7)
    Lam { param: VarId, body: Box<Node> },               // r4: function value; closed, first-order
    App { func: Box<Node>, arg: Box<Node> },             // r4: call-by-value
    Fix { name: VarId, body: Box<Node> },                // r4: self-recursion, fuel-clocked
    FixGroup { defs: Vec<(VarId, Box<Node>)>, body: Box<Node> }, // r5 (M-343): mutual recursion
}
```

WF invariants (enforced structurally or by linter, Foundation §5.8):
- **WF1**: only `Swap` changes `Repr`
- **WF2**: every `Swap` carries a `PolicyRef` (mandatory field, enforced by construction)
- **WF6**: `Construct` is fully applied (saturation checked above kernel vs DataRegistry)
- **WF7**: `Match` coverage checked (`Match` on non-enumerable domain must carry `default`)
- **WF8**: no `Swap` introduced silently through `Construct`/`Match` elaboration

`Node::is_repr_changing()` returns `true` only for `Swap`. `Node::is_aot_lowerable()` is now
total over all v0 nodes — the AOT env-machine covers the full calculus (M-342).

### `CoreValue` and `Datum` (`datum.rs`)

```rust
pub enum CoreValue { Repr(Value), Data(Datum) }
pub struct Datum { ctor: CtorRef, fields: Vec<CoreValue>, guarantee: GuaranteeStrength }
```

`Datum` carries a guarantee summary (meet of field guarantees) but **no `Bound`** — bounds live
on leaf representation values. Content-addresses over `ctor || fields` (summary excluded). The
`Value<R>` type is unchanged by r3 (sibling type, not a reshaping — KISS/YAGNI/KC-3).

---

## Content-addressing (`content.rs`, RFC-0001 §4.6, ADR-003)

```
hash(def) = BLAKE3( normalize(structure) || types_with_repr || static_contract )
```

**Hashed (identity-bearing):** normalized node structure (de Bruijn indices for bound vars),
`Repr` including paradigm, literal payloads, operator names, swap `target`+`policy`, `CtorRef`
pairs (`#T#i`).

**Not hashed (metadata):** human names (stored in separable `Names` map), source spans, comments,
and all dynamic `Meta` (provenance, sparsity, realized bounds, `policy_used`).

Consequences: paradigm change -> different hash; alpha-rename -> same hash; metadata change -> same
hash. `Names::bind(hash, name)` stores human names outside identity. `operation_hash(prim)` is
domain-separated (tag `0x07`) from structural hashes.

---

## Key invariants (honesty)

1. **No implicit conversion.** `Swap` is the only repr-changing node; cross-paradigm `Op` args
   are rejected at the type level — no coercion rule (FR-M3, WF1).
2. **Guarantee degrades, never spuriously upgrades.** `propagate` is the only sanctioned path;
   result strength never exceeds weakest input (VR-3/VR-5, ADR-001).
3. **`Meta` invariants are unrepresentable to violate.** M-I1-M-I4 enforced by `Meta::new`
   and re-run on every deserialization — malformed wire data raises an error, never silent.
4. **Metadata != identity.** `Meta` excluded from `content_hash()`; names in `Names` table.
5. **Static contract IS identity.** `Repr`, literal payload, operator names, swap contracts hashed.
6. **No black boxes.** Every `Swap` carries a `PolicyRef`; `physical` is a record, not the
   decision. Runtime query surface: `repr_of`, `meta_of`, `guarantee_of`, `bound_of`,
   `provenance_of` exposed via LSP (RFC-0001 §4.8, SC-3/SC-5).

---

## Read more

- `docs/rfcs/RFC-0001-Core-IR-and-Metadata-Schema.md` §4.1-4.8 — normative spec
- `docs/Mycelium_Project_Foundation.md` §3 (FR/NFR/VR), §8 (ADRs 001-011)
- `docs/rfcs/RFC-0011*.md` — data + matching fold into kernel (r3)
- `docs/rfcs/RFC-0007*.md` — Lam/App/Fix/FixGroup (r4/r5)
- `docs/rfcs/RFC-0002*.md` — swap certificates and legal pairs
- `docs/rfcs/RFC-0005*.md` — SelectionPolicy language (PolicyRef)
- ADR-003 (content-addressing / names-as-metadata), ADR-010 (bound kernels), ADR-011 (universal basis)

---

## Gotchas

- `Repr::Vsa` in Rust serializes as `"VSA"` on the wire (serde rename).
- Physical packing lives in `meta.rs`, not `repr.rs`; two ternary values of the same
  `Ternary{trits:6}` may differ in `physical` but are the same type (DN-01).
- `Dense::dtype` IS identity-bearing; `physical` is NOT (lossless, M-I5).
- `Datum` carries no `Bound` — bounds live on leaf `Value` fields inside the datum.
- `FixGroup` emitted only for SCCs of >=2 by the elaborator; single self-recursion stays `Fix`.
- `Lam`/`App`/`Fix` carry no type annotations in L0 (post-typecheck core is untyped, like `Let`).
- Two structurally-identical closed lambdas of different param types collide in content hashing
  (deliberate alpha-equivalence; RFC-0001 r4 §4.2 — defensible, not a correctness issue).
- Wire deserialization uses `deny_unknown_fields` — extra JSON fields are rejected (A6-02).
- `EmpiricalFit { trials: 0 }` or empty `method`/`citation` is `well_formed() == false`.
