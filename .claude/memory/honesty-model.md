## What it is

The **cross-cutting honesty system** — Mycelium's defining feature. Every accuracy/guarantee claim
must be tagged at a specific lattice level with a checked basis; approximation is first-class but
never hidden. This is not advisory policy: the types enforce it mechanically (Meta::new rejects
inconsistent guarantee/bound pairs; propagate is the only sanctioned composition path).

Primary sources: `CLAUDE.md §Non-negotiable house rules`, `docs/Mycelium_Project_Foundation.md`
§3.3 (VR-1/VR-5), §4.1 (Tension B), §8 (ADR-001); `docs/rfcs/RFC-0001-Core-IR-and-Metadata-Schema.md`
§3.4/§4.7; Rust: `crates/mycelium-core/src/guarantee.rs`, `crates/mycelium-core/src/bound.rs`,
`crates/mycelium-core/src/meta.rs`.

---

## Where it lives (key files)

| Item | Path |
|---|---|
| Lattice type + `meet`/`propagate` | `crates/mycelium-core/src/guarantee.rs` |
| `Bound`, `BoundBasis`, `BoundKind`, `NormKind` | `crates/mycelium-core/src/bound.rs` |
| `Meta::new` (invariant enforcement) | `crates/mycelium-core/src/meta.rs` |
| `WfError` variants | `crates/mycelium-core/src/lib.rs` |
| Node grammar (`Swap` WF1/WF2; never-silent) | `crates/mycelium-core/src/node.rs` |
| Normative lattice spec | `docs/rfcs/RFC-0001-Core-IR-and-Metadata-Schema.md` §3.4/§4.7 |
| House rules | `CLAUDE.md §Non-negotiable house rules` |
| Foundation grounding | `docs/Mycelium_Project_Foundation.md` §3.3 (VR-5), §4.1 (Tension B), §8 ADR-001 |

---

## The guarantee lattice

```
Exact ⊐ Proven ⊐ Empirical ⊐ Declared
rank:  0         1           2           3
```

`Exact` is the identity; `Declared` is the bottom (absorbing). Implemented as
`GuaranteeStrength::meet` in `crates/mycelium-core/src/guarantee.rs:61` — weakest wins.

### What each level means and when it is allowed

**`Exact`** (rank 0)
- No approximation whatsoever.
- `bound == None` (M-I1 — enforced by construction).
- Applies to: pure binary/ternary arithmetic, bijective binary<->ternary swaps, constant construction.
- How to use: pass no bound to `Meta::new` with `GuaranteeStrength::Exact`.

**`Proven`** (rank 1)
- Approximate, but a *machine-checked* theorem covers the error/capacity/probability bound.
- `bound.basis == BoundBasis::ProvenThm { citation }` (M-I2 — enforced by construction).
- Example citations: "Clarkson-Ubaru-Yang 2023" (MAP-I bundle capacity), "Thomas-Dasgupta-Rosing 2021".
- CRITICAL: `Proven` is allowed **only** with a theorem whose side-conditions are actually *checked*
  (instantiated via SMT/LH, not merely cited aspirationally). Promoting from `Empirical` to `Proven`
  requires a checked instantiation — never upgrade without a checked basis (CLAUDE.md, VR-5).
- The Liquid-Haskell MAP-I `bundle` probe (`proofs/lh-bundle/`) is the confirming build for KC-1.

**`Empirical`** (rank 2)
- Approximate, with an empirically-validated bound from a measurement or statistical fit.
- `bound.basis == BoundBasis::EmpiricalFit { trials, method }` (M-I3).
- Example: Frady-Sommer Gaussian approximation for HRR capacity. Honest about its weaker basis (G5).
- `trials >= 1` and `method` must be non-empty (vacuous empirical claims rejected by `Bound::well_formed`).
- Applies to: VSA ops whose bounds rest on Gaussian approximation, not a proved non-asymptotic result.

**`Declared`** (rank 3)
- Approximate, with a user-asserted bound not yet validated.
- `bound.basis == BoundBasis::UserDeclared` (M-I4).
- Tooling **MUST** surface a "declared, unverified" marker. Never silently trusted (VR-5).
- Example: a user annotates a custom swap with a tolerance they claim but have not measured.
- `citation` / `method` are not required here, but must not be fabricated.

### Composition rule (RFC-0001 §4.7, `guarantee.rs:70`)

```rust
// The ONLY sanctioned way to derive an operation's result strength:
GuaranteeStrength::propagate(intrinsic_g_f, [g_v1, g_v2, ..., g_vn])
// = meet(g_f, g_v1, g_v2, ..., g_vn)  = weakest of all
```

An operation's result is **never stronger** than its weakest input or than the operation's own
intrinsic guarantee. Honesty is monotone-downward; there is no upward path through computation.

The `meet` forms a semilattice: commutative, associative, idempotent, identity `Exact`.
Laws verified exhaustively over all 4x4(x4) tuples in `guarantee.rs` tests.

---

## Never-silent (G2, WF1/WF2)

**Every out-of-range, conversion, or fallible operation is an explicit `Option`/`Result`/error.**
Swaps and conversions are never implicit. This is mechanically enforced:

- `Swap` is the **only** `Node` that changes a value's `Repr` (WF1). The type checker rejects
  cross-paradigm `Op` arguments — no coercion rule, no subsumption rule between paradigms (FR-M3).
- Every `Swap` carries a mandatory `policy: PolicyRef` (WF2 — enforced by construction since
  `policy` is a required field in `Node::Swap`, not an `Option`).
- `Value::new` returns `Result<Self, WfError>` — malformed repr or mismatched payload is an
  explicit error, never a panic or silent truncation.
- `Meta::new` returns `Result<Self, WfError>` — M-I1-M-I4 violations surface immediately.
- `Bound::well_formed()` rejects infinite epsilon/crosstalk (vacuous bounds), delta outside [0,1],
  zero-trial empirical fits, empty citations.
- Out-of-range deserialization raises an error (`deny_unknown_fields`, re-runs invariants).
- `Fix`/`FixGroup` unfold under a fuel clock — a non-productive recursion is explicit budget
  exhaustion (`Err(BudgetExhausted)`), never a hang (RFC-0007 §4.5).
- The native LLVM backend refuses data/recursion nodes it cannot lower with explicit
  `UnsupportedNode` errors (VR-5; `mycelium-mlir::llvm`), never a silent partial lowering.

---

## No black boxes / EXPLAIN-able (G2, ADR-006, RFC-0005)

- Every selection/conversion/approximation is **reified and inspectable**.
- Every `Swap` node records `policy_used: Option<ContentHash>` in the result's `Meta` (set by the
  interpreter after a swap). The `PolicyRef` on the node references the policy that *chose* the swap
  (RFC-0005 `SelectionPolicy`).
- The `SelectionPolicy` is a first-class, queryable value — it can be introspected via the LSP or
  CLI (`EXPLAIN`-able). No implicit/"magic" representation selection is allowed (FR-W2, ADR-006).
- `Meta.physical` is a *record* of the schedule-staged packing, not the decision locus; the
  decision lives in the lowering schedule (RFC-0004 §5) and the selector (`mycelium-select`).
- Runtime query surface (RFC-0001 §4.8): `repr_of(v)`, `meta_of(v)`, `provenance_of(v)`,
  `guarantee_of(v)`, `bound_of(v)` — all exposed via LSP (SC-5, NFR-3).

---

## Append-only decisions (CLAUDE.md §Non-negotiable house rules)

ADRs, RFCs, and design notes are **append-only with forward-only status transitions**:

```
Draft/Proposed -> Accepted -> Enacted -> Superseded
Notes: -> Resolved
```

- **`Enacted`** = Accepted + fully implemented/landed. MUST step through `Accepted` first — never
  skip straight to `Enacted`.
- To change an `Accepted`/`Enacted` decision: supersede it (write a new ADR/RFC), do not rewrite.
- `Accepted` decisions in the corpus read "implemented (Rust-first), pending ratification" where
  the impl is Rust-first but self-hosting is not yet complete (Foundation r4 honesty note).
- Per-doc changelog footers are also append-only; statuses only advance.

---

## Ground every claim (G1-G11, CLAUDE.md)

Normative statements MUST cite their grounding:

| Label class | What it covers |
|---|---|
| `G1-G11` | Survey findings (e.g. G1: four-way union is unprecedented; G2: no black boxes; G5: VSA bounds need honest tagging; G10: LLM leverage) |
| `FR-M1..FR-W3` | Functional requirements (FR-M3: no implicit swap; FR-M5: metadata persists) |
| `NFR-1..NFR-7` | Non-functional requirements (NFR-7: interpreter = reference semantics) |
| `VR-1..VR-5` | Verification requirements (VR-5: honest guarantee-strength tagging) |
| `SC-1..SC-5` | Success criteria (SC-3: no silent swaps; SC-5: dual intelligibility) |
| `KC-1..KC-4` | Kill/redirect criteria (KC-1: VSA bounds exist, PASSED) |
| `ADR-001..ADR-011` | Architecture decisions |
| `RFC-0001..` | Normative RFCs |
| `T0.x..T2.x` | Research findings (T0.2: proven VSA capacity bounds exist) |
| `A-E` | Tensions (B: exact vs approximate; A.1: no black boxes) |
| `R1-R8` | Research requirements/falsifiers |

Ungrounded "facts" are forbidden. Open questions must be labeled as such. Downgrade to stay
honest; never claim `Proven` without a checked theorem instantiation.

---

## How honesty shows up in code

### Guarantee tags on values

Every `Value` carries `meta.guarantee`. The only way to produce a `Proven` value is to supply a
`Bound` with `BoundBasis::ProvenThm { citation }` and pass it through `Meta::new` — the type
system makes the alternative unrepresentable (M-I2, `meta.rs:273`).

### Never-silent fallibility

- `Value::new` -> `Result<Value, WfError>` (repr + payload mismatch)
- `Meta::new` -> `Result<Meta, WfError>` (M-I1..M-I4, malformed bound/sparsity)
- `DataRegistry::lookup` -> `Option<&DataDecl>` (missing ctor)
- `Node::Swap` carries `policy: PolicyRef` (mandatory, no `Option`) — WF2 by construction
- Interpreter fuel clock: `Fix`/`FixGroup` return `Err(BudgetExhausted)` rather than hanging

### EXPLAIN traces

The provenance DAG (`Meta.provenance: Provenance`) tracks `Root | Derived { op, inputs }` where
`op` and `inputs` are `ContentHash`es of the producing operation and its inputs respectively.
`operation_hash(prim)` in `content.rs` generates the `op` hash. The full DAG can be walked to
reconstruct how any value was produced — the EXPLAIN surface (Foundation §5.8).

### Guarantee propagation in the interpreter

The reference interpreter (`crates/mycelium-interp/`) calls `GuaranteeStrength::propagate` when
evaluating `Op` nodes — the intrinsic guarantee of the `Prim` meets with the input guarantees.
Result values are constructed with the propagated guarantee, never with an upgraged one.

---

## Read more

- `CLAUDE.md §Non-negotiable house rules` — the five rules in enforceable form
- `docs/Mycelium_Project_Foundation.md` §3.3 VR-5, §4.1 Tension B, §8 ADR-001
- `docs/rfcs/RFC-0001-Core-IR-and-Metadata-Schema.md` §3.4 (worked example), §4.3 (M-I1..M-I5),
  §4.7 (lattice + bound composition), §6 (rationale for 4-point lattice)
- `crates/mycelium-core/src/guarantee.rs` — exhaustive law tests
- `crates/mycelium-core/src/meta.rs` — `check_guarantee_bound` function
- `crates/mycelium-core/src/bound.rs` — `Bound::well_formed`, `BoundBasis::strength`
- `crates/mycelium-numerics/` — two bound-composition kernels (ADR-010): `error.rs`
  (affine arithmetic for epsilon) and `prob.rs` (union-bound for delta)

---

## Gotchas

- **Never upgrade a guarantee without a checked basis.** Promoting `Empirical` to `Proven` requires
  a theorem instantiation (SMT/LH). Citing a theorem without checking its side-conditions is still
  `Declared` at best.
- **`Declared` is not a placeholder.** It carries real semantics: tooling MUST flag it; it must
  not be used silently as a temporary until tests are written.
- **Two bound kernels, not one** (ADR-010 / T0.1c). Epsilon (error magnitude) and delta (failure
  probability) compose under different algebras that meet at `{epsilon, delta, strength}`. They
  are NOT unified; using the wrong kernel produces unsound bounds.
- **`Enacted` must step through `Accepted`** — never jump straight from `Draft` to `Enacted`.
  Check the RFC/ADR header before changing status in a PR.
- **Append-only for agents too.** Agent code should never edit an existing ADR/RFC body to change
  a decision — only append a supersession entry or a new revision note (RFC-0001 has r0..r5 as
  append-only revision entries, e.g. `meta.rs` changelog footer).
- **Empirical basis requires evidence.** `BoundBasis::EmpiricalFit { trials: 0, .. }` is
  rejected by `Bound::well_formed()` — zero-trial empirical claims are unsound.
- **`physical` layout is a record, not a guarantee.** Changing `PhysicalLayout` with
  `Meta::with_physical` does not affect `guarantee` or `bound` (M-I5, `meta.rs:157`).
- **HRR/FHRR unbind is `Empirical` only** — bind is not self-inverse (RR-13). Prefer MAP/BSC
  for compositional work where `Proven` bounds are needed.
- **The kill criteria apply at every phase gate** (KC-1..KC-4). KC-1 passed (proven VSA bounds
  exist); KC-2 verdict = proceed (weak LLM leverage, not irrecoverable collapse). KC-3 and KC-4
  remain live circuit-breakers.
