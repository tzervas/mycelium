# Spec — `std.recover` (the reified declarative-recovery surface: every error recovered or re-propagated, never dropped)

| Field | Value |
|---|---|
| **Status** | **Accepted (Rust-first half)** (2026-06-20, maintainer-ratified per DN-07 — guarantee matrix asserted in tests; open §7/§8 questions are design/scope calls, not contract violations; was *Implemented (Rust-first half) — pending ratification* 2026-06-18, Draft/needs-design 2026-06-17) — the **Rust-first library half** of M-520 landed as `mycelium-std-recover` (M-520, #156, Batch P5 Tier-A completion): the `Outcome`/`Resolution` sums (**no `Dropped` variant** — I1), the closed v0 action set, the content-addressed `RecoveryPolicy`/`PolicyRef`, budgeted effects via `mycelium_interp::budget`, and the never-silent `handle_classified` driver. The recovered tag is honest (`Ok`→floor, `fallback`→`Declared`, never laundered up — I2/VR-5; fixes the P5-B exact-tag bug), guarantee matrix asserted in tests. The Mycelium-lang migration (M-502-gated) remains. |
| **Module / Ring** | `std.recover` · Ring `1` ([RFC-0016 §4.2](../../rfcs/RFC-0016-Core-Library-and-Standard-Library.md) — capability surface) · Tier `A` ([RFC-0016 §4.3](../../rfcs/RFC-0016-Core-Library-and-Standard-Library.md)) |
| **Tracks** | `M-520` (#156) — the Phase-5 task this spec delivers: self-host the [RFC-0014](../../rfcs/RFC-0014-Declarative-Error-Recovery-and-Bounded-Effects.md) reified recovery subsystem (today `crates/mycelium-lsp/src/recover`, Rust) into Mycelium-lang. |
| **Scope** | The library home of the **reified declarative recovery** surface: the result-sum `Outcome`, the **closed** recovery-action set (`fallback`/`retry`/`escalate`/`cleanup_then_propagate`), the registry-shared on-`ErrorClass` recovery **policy** (a content-addressed `PolicyRef` artifact), declared + **budgeted** effects with a graceful `EffectBudgetExhausted`, and — the spine — the **never-silent `handle`**: every error is recovered (with an honest tag) or re-propagated, **never dropped** (I1). |
| **Boundary** | OUT of scope: (a) the diagnostic *record / trace / debug-info substrate* an error carries — owned by `std.diag` (M-510, the library form of [RFC-0013](../../rfcs/RFC-0013-Structured-Diagnostics-and-Reified-Error-Policy.md)); `recover` decides what to **do** about a diagnosed error, it does not present it. (b) The errors-as-values *combinator ergonomics* (`map`/`and_then`/`?`-propagate/`unwrap`) — owned by `std.error` (M-527); `recover` is the **bridge target** `error` hands an error value to, and it **owns the concrete `RecoverOutcome`/`PolicyRef` signature `error` deferred** (see §3, §7-Q1). (c) The runtime budget *enforcement plumbing* (the env-machine ledger, `EvalError::EffectBudget`) — owned by `mycelium-interp` (M-353); `recover` **declares + spends** budgets through that one mechanism, it does not re-implement it. |
| **Depends on** | [RFC-0014](../../rfcs/RFC-0014-Declarative-Error-Recovery-and-Bounded-Effects.md) **(Accepted — Enacted, M-352/M-353)** — the reified recovery subsystem this module is the self-hosted library form of (the `Outcome`, the closed action set, the on-`ErrorClass` policy, I1–I5); RFC-0014 §4.1/§4.2 (errors-as-propagating-values + the never-silent invariant), §4.3 (handling = L0 `Match`, no new kernel node), §4.4 (reified policy + `PolicyRef`), §4.5 (declared, bounded effects); [RFC-0016 §4.1](../../rfcs/RFC-0016-Core-Library-and-Standard-Library.md) (the C1–C6 contract); RFC-0001 (the value model — `Option`/`Result`/structured error sums, the guarantee lattice). |
| **Grounds on** | the landed RFC-0014 Rust subsystem `crates/mycelium-lsp/src/recover` (M-352, Accepted 2026-06-16 — `Outcome`/`Resolution`/`StructuredError`, the closed `RecoveryAction` set, the content-addressed `RecoveryPolicy`, the never-silent `handle`) and the shared budget primitive `mycelium_interp::budget` (M-353 — `EffectKind`/`EffectBudget`/`Budgets`/`EffectBudgetExhausted`, one enforcement mechanism); the **self-hosted diagnostics** `std.diag` (M-510, the diagnostic record/registry this builds *on*). KC-3: above the kernel — no new trusted code, no new L0 node (recovery elaborates to `Match`, NFR-7). |

---

## 1. Summary

`std.recover` is the standard-library, **self-hosted** form of the RFC-0014 reified recovery subsystem
(M-520; today the Rust `crates/mycelium-lsp/src/recover`). Its user-facing surface is the result-sum
`Outcome`, the **closed** recovery-action set, a registry-shared **on-`ErrorClass`** recovery *policy* (a
content-addressed, `EXPLAIN`-able `PolicyRef` artifact), and the never-silent **`handle`** driver. Its
**honesty crux** is **I1 / C1 (G2)** in its operational, control-flow form — *the never-silent handle*:
for every error, `handle` yields **either** a `Recovered` value (carrying an *honest* guarantee tag —
never upgraded, I2) **or** a re-propagated error (possibly transformed) — and there is **no third,
"dropped" outcome**. A Mycelium program **may legitimately halt, refuse, or fail** for a specific error
case; what this module structurally forbids is an error *silently vanishing*. Recovery's job is to make
failure **robust and legible**: every error is acted on through a *reified, inspectable* policy (carrying
the RFC-0013 diagnostic record, trace, and debug info) and either recovered or re-propagated — and when a
recovery action exhausts its **declared, bounded** effect budget, that is a graceful, explicit
`EffectBudgetExhausted` outcome, itself legible, **not** a silent stall. Ring 1, Tier A; it adds **no
trusted code** and **no new kernel node** (KC-3, NFR-7) — recovery elaborates to an L0 `Match` over the
error sum.

## 2. Scope & module boundary

- **In scope:** the result-sum `Outcome<τ>` = `Ok(τ) | Err(ε)` (the `Err` payload is the RFC-0001/RFC-0013
  structured error, `diag`-owned); the **`Resolution<τ>`** result of handling (`Recovered | Propagated`,
  with **no** drop variant); the **closed v0 recovery-action set** — `fallback(value)`, `retry(<=N)`,
  `escalate(class')`, `cleanup_then_propagate(effect)` (RFC-0014 §4.4; §8 resolved closed); the **reified
  recovery policy** (an `on <ErrorClass> => <action>` map resolved through `diag`'s shared error-class
  registry, content-addressed to a `PolicyRef`); the **declared, bounded effects** surface (`EffectKind`
  set, per-kind `EffectBudget`, the `Budgets` ledger, the compositional `check_effects` no-undeclared-effect
  check) re-exported from `mycelium_interp::budget`; the never-silent **`handle`** driver; and — owned here
  — the concrete **`RecoverOutcome` / `PolicyRef` bridge signature** that `std.error` (M-527) hands an error
  value into.
- **Out of scope (and who owns it):**
  - The diagnostic *record / trace / debug-info* substrate the error carries, the error-class **registry**,
    and any *presentation* of an outcome — `std.diag` (M-510, library form of RFC-0013). `recover` resolves
    classes through that registry and acts on the diagnosed error; it never renders one. (RFC-0014 §4.9 — a
    diagnostic policy never changes control flow; a recovery policy's presentation is delegated to diag.)
  - The errors-as-values *combinator* ergonomics (`map`/`map_err`/`and_then`/`or_else`/`?`-propagate/
    `unwrap`-family) — `std.error` (M-527). That module *bridges into* this one; this module owns the
    concrete outcome/policy types it bridges to.
  - The runtime budget-*enforcement* machinery (the env-machine ledger, the `EvalError::EffectBudget`
    channel that sits beside `FuelExhausted`/`DepthLimit`) — `mycelium-interp` (M-353). `recover` declares
    and *spends* budgets over that one mechanism; it does not own the clock.
- **Ring & layering:** Ring 1 (RFC-0016 §4.2 — a capability surface over a landed crate). `recover` is the
  self-hosted **re-implementation in Mycelium-lang** of the RFC-0014 subsystem, held bit-for-tag against
  the Rust origin by a differential (NFR-7). It adds **no trusted code**: the never-silent `handle` and the
  policy are ordinary functions/values; handling **elaborates to an existing L0 `Match`** over the error sum
  (RFC-0014 §4.3 — **no new kernel node**, KC-3); budget enforcement lives in `mycelium-interp` (M-353), not
  here. No `wild`/FFI (ADR-014).

## 3. Exported-op surface (design sketch)

A signature sketch — value-semantic, immutable-by-default. `Outcome`/`Resolution` are immutable value sums;
`handle` is total (it always yields a `Resolution`, never diverges past its budgets); the policy builder
(`on`) is fallible (an unknown class is an explicit configuration error). Effectful actions (`retry`,
`cleanup_then_propagate`) declare + budget their effect on the signature (C6). This fixes the surface and
feeds the §4 matrix; it is **not** a committed grammar. **The surface-language `Match`/effect-budget
spelling is owned by RFC-0014 §4.3 / RFC-0006 (KC-2-gated) and M-353 — not invented here** (see §7-Q2).

```text
// illustrative signatures (not a committed surface). τ is a type var; ε is the diag-owned structured error.

// ---- the substrate: the result sum (Err payload is the diag/RFC-0013 structured error) ----
type Outcome<τ>    = Ok(τ) | Err(ε)            // RFC-0014 §4.1; errors propagate by default
type ErrorClass                                // a name resolved through diag's shared registry (NEVER eval'd — X1)

// ---- the outcome of handling: recovered OR re-propagated; NO "dropped" variant (I1) ----
type Resolution<τ> =
    | Recovered  { value: τ, guarantee: Guarantee, policy: Option<PolicyRef> }   // honest tag, never upgraded (I2)
    | Propagated { error: ε,                       policy: Option<PolicyRef> }   // additive over the explicit error

// ---- the closed v0 recovery-action set (§4.4; §8 resolved closed — user actions are a §9 future) ----
type RecoveryAction =
    | fallback(value)                 // recover with an honest `Declared` value (I2)
    | retry(max_attempts: Nat)        // re-attempt, BOUNDED (I4); on exhaustion the ORIGINAL error propagates
    | escalate(to: ErrorClass)        // transform + re-propagate (still explicit)
    | cleanup_then_propagate(effect: EffectKind)  // run a BOUNDED effect, then let the error continue (additive)

// ---- the reified, content-addressed recovery policy (RFC-0005 pattern; ADR-006) ----
type RecoveryPolicy           // an `on <ErrorClass> => <action>` map; identity = PolicyRef (content hash)
type PolicyRef = ContentHash  // RFC-0001 §4.6 / ADR-003 — the policy's content address (EXPLAIN-able)

on        : RecoveryPolicy, Registry, class: Str, RecoveryAction -> Result<RecoveryPolicy, UnknownClass>  // X1
policy_ref: RecoveryPolicy -> PolicyRef
action_for: RecoveryPolicy, ErrorClass -> Option<RecoveryAction>

// ---- declared, bounded effects (re-exported from mycelium-interp / M-353; one enforcement mechanism) ----
type EffectKind   = Retry | Alloc | Io | Cascade | Time | Named(Str)   // closed kernel + user-declared names
type EffectBudget = Attempts(Nat) | Depth(Nat) | Bytes(Nat) | Fuel(Nat)
type Budgets                                   // the ledger; an effect with NO declared budget cannot run (I5)
consume      : Budgets, EffectKind, amount: Nat -> Result<Budgets, EffectBudgetExhausted>   // graceful overrun (I4)
check_effects: declared: Set<EffectKind>, performed: Set<EffectKind> -> Result<Unit, UndeclaredEffect>  // I3

// ---- the never-silent DRIVER: every error recovered OR re-propagated, NEVER dropped (I1) ----
//   declared effects: {retry, cascade, alloc, ...} as the policy's actions require; all BUDGETED (C6/I4).
handle : Outcome<τ>, RecoveryPolicy, &mut Budgets, attempt: (() -> Outcome<τ>) -> Resolution<τ>
//   Ok(v)       -> Recovered(v, v's own guarantee, policy: none)   -- nothing to recover
//   Err(e), no matching rule -> Propagated(e UNCHANGED, policy: none)   -- (I1) the floor: re-propagate
//   fallback    -> Recovered(value, Declared, policy)              -- honest tag (I2)
//   retry(<=N)  -> Recovered(.., policy) | Propagated(original e)  -- additive on exhaustion
//   escalate    -> Propagated(transformed e, policy)
//   cleanup_then_propagate -> Propagated(original e, policy)       -- bounded cleanup; overrun graceful, skips cleanup

// ---- THE BRIDGE std.error (M-527) hands into — concrete signature OWNED HERE (resolves error.md §7-Q1) ----
//   `error.recover : Result<τ,ε>, PolicyRef -> RecoverOutcome<τ,ε>` is, concretely:
type RecoverOutcome<τ> = Resolution<τ>         // the SAME sum: Recovered | Propagated; NO drop variant (I1)
recover : Outcome<τ>, RecoveryPolicy, &mut Budgets, attempt: (() -> Outcome<τ>) -> RecoverOutcome<τ>  // = handle
```

> **Bridge note (owned here; closes `error.md` §7-Q1).** `std.error`'s `recover` bridge described
> `RecoverOutcome` / `PolicyRef` *abstractly*, deferring the concrete shape to this module. Resolved:
> **`RecoverOutcome<τ>` *is* `Resolution<τ>` = `Recovered { value, guarantee, policy } | Propagated { error,
> policy }`** — a two-variant sum with **no drop variant** (so I1 is a property of the type, exactly as the
> Rust `Resolution` enforces it), and **`PolicyRef` is the policy's `ContentHash`** (RFC-0001 §4.6 / ADR-006).
> The bridge is `handle` itself. `error.md`'s contract — *"outcome is `Recovered | Propagated`, no drop
> variant (I1), honest inherited tag (I2)"* — holds verbatim. (The bare `?`-propagate and the `unwrap`-family
> stay in `error`; this module owns only the typed handoff target.)

## 4. Guarantee matrix (the load-bearing deliverable — [RFC-0016 §4.5](../../rfcs/RFC-0016-Core-Library-and-Standard-Library.md))

Rows = exported ops. Encoded as a checked table (the RFC-0003 §4 template); asserted in tests once the
self-hosted code lands, **never prose only** — and held **differentially** against the Rust origin (NFR-7).
Columns: **guarantee tag** · **fallibility** (the explicit outcome/error set) · **declared + budgeted
effects** · **EXPLAIN-able?**. **No row permits a silent drop of an error (RFC-0014 I1 / RFC-0016 C1):**
every row either *recovers* explicitly (honest tag), *re-propagates*, or *refuses an effect budget gracefully*
— none of them is "silently returns success" or "the error disappears".

| Op | Guarantee tag | Fallibility (explicit outcome / error set) | Declared + budgeted effects | EXPLAIN-able? |
|---|---|---|---|---|
| `handle` (the never-silent driver) | inherits the recovered value's honest tag (`Declared` for a `fallback`; the attempt's own tag for a `retry` success; `Exact` for a clean `Ok` pass-through) — **never upgraded** (I2/VR-5) | total over its budgets — yields `Recovered \| Propagated`; **no drop** (I1). A budgeted action overrun is a graceful `EffectBudgetExhausted` routed through the outcome (I4) | the policy's actions' effects (`retry`, `cascade`, `alloc`, `io`), **each budgeted** (I3/I4); a clean `Ok`/`fallback`/`escalate` path is effect-free | **yes** — every outcome records the acting `PolicyRef` (C3) |
| `recover` (the `std.error` bridge) | = `handle` — **inherits** the policy's honest tag, never upgrades it (I2) | total over its budgets — `RecoverOutcome = Recovered \| Propagated`; **no drop** (I1) | = `handle` (the policy's declared, budgeted effects) | **yes** — `PolicyRef` recorded on every outcome |
| `fallback(value)` (action) | **`Declared`** — a substituted fallback has no checked basis (I2/VR-5) | total — always `Recovered(value, Declared)` (the one always-recovering action) | none | yes (action is in the policy's `EXPLAIN`) |
| `retry(<=N)` (action) | the successful attempt's **own** tag; on exhaustion no value is produced | total over the budget — `Recovered` on an attempt's `Ok`, else `Propagated(original error)`; **bounded** by `<=N` (I4) | **`retry`, budgeted** `Attempts(N)`; overrun → graceful `EffectBudgetExhausted` (I4) | yes |
| `escalate(class')` (action) | n/a (re-propagates an error, not a value) | total — always `Propagated(transformed error)` (still explicit; never a drop — I1) | none (pure transform) | yes |
| `cleanup_then_propagate(eff)` (action) | n/a (re-propagates the original error) | total — runs a **bounded** cleanup then `Propagated(original error)`; a budget overrun skips the cleanup *only*, the error propagates regardless (I1) | the declared cleanup `effect`, **budgeted** (I4); overrun graceful, additive | yes |
| `on` (policy registration) | `Exact` (builds a value) | `Err(UnknownClass)` — a class not in the `diag` registry is an explicit configuration error, never an eval'd string (X1) | none | yes (the resulting `RecoveryPolicy` is content-addressed) |
| `policy_ref` / `action_for` | `Exact` | total | none | yes (`policy_ref` *is* the `EXPLAIN` identity) |
| `consume` (budget ledger) | `Exact` | `Err(EffectBudgetExhausted)` — the **graceful, explicit** overrun (I4), never a hang/OOM | the consumed `EffectKind`, **bounded** | yes (the overrun names kind + requested + remaining) |
| `check_effects` (I3) | `Exact` | `Err(UndeclaredEffect)` — a performed-but-undeclared effect is an explicit checker error (I3) | none (a static check) | yes (names the undeclared effect) |

**Justification of the tags (downgrade to stay honest — VR-5):**
- `handle` / `recover` carry **no fixed tag** — they **inherit** the honest guarantee the recovered value
  carries (`Declared` for a `fallback`, the attempt's tag for a `retry`, `Exact` for a clean `Ok`/`escalate`
  pass-through). This module **never launders that tag upward** (RFC-0014 I2 / VR-5). The op is total *over
  its declared budgets* — a budget overrun is an explicit outcome, not non-termination (RFC-0014 §4.7:
  budgeted ⟹ terminating).
- `fallback` is **`Declared`** by construction: a substituted value is *asserted*, not proven, so it is at
  most `Declared` (flagged) unless it has an independent checked basis (I2). This is the only action with a
  fixed tag; the tag describes the *recovered value's* honesty, exactly as the Rust `handle` sets
  `GuaranteeStrength::Declared` for a `Fallback`.
- The policy/ledger/check ops (`on`, `policy_ref`, `action_for`, `consume`, `check_effects`) are value /
  registry / budget operations with **no accuracy semantics**, hence **`Exact`** (the `len`-style case,
  RFC-0016 C2). Their *fallibility* (not their tag) is where the honesty lives: `UnknownClass`,
  `EffectBudgetExhausted`, `UndeclaredEffect` are all explicit, never silent.

**The invariants this matrix encodes (RFC-0014 §4.2/§4.5):**
- **(I1) Never-silent — the spine.** Every `handle`/`recover` row yields `Recovered | Propagated` and the
  `Resolution`/`RecoverOutcome` type has **no drop variant** — a mutant handler that discarded an unmatched
  error *cannot be expressed* in this surface (the type enforces it, as the Rust `Resolution` does). An
  unmatched class re-propagates the error **unchanged**.
- **(I3) Effects are declared.** `check_effects` makes a performed-but-undeclared effect an explicit
  `UndeclaredEffect` (no unknown side effects) — a *check*, never an inference.
- **(I4) Budgeted effects overrun gracefully.** `retry`/`cascade`/`alloc`/`time` are budgeted; `consume`
  returns an explicit `EffectBudgetExhausted` (kind + requested + remaining), never a hang/OOM — routed
  through the same one mechanism (`mycelium-interp`, M-353) the runtime uses for fuel/depth.
- **(I5) Tightly scoped by default; broader is opt-in.** The `Budgets` ledger refuses any effect with **no
  declared budget** (`consume` on an absent budget is an immediate `EffectBudgetExhausted`); a broader effect
  arrives only by *declaring its budget* — never implicitly.

## 5. §4.1 contract conformance (C1–C6)

- **C1 — never-silent (G2) — THE NEVER-SILENT HANDLE (I1).** *The crux.* `handle` is total and its result
  type, `Resolution<τ> = Recovered | Propagated`, has **no "dropped" variant**: every error is either
  recovered (an explicit value with an honest tag) or re-propagated (an explicit error, possibly
  transformed) — there is no policy, action, or effect that can make an error *neither surface nor be
  explicitly recovered*. An `Err` with no matching policy rule re-propagates **unchanged**; a `retry` that
  exhausts re-propagates the **original** error (additive); a `cleanup_then_propagate` whose effect overruns
  *still* propagates the original error. **A program may legitimately halt/refuse on a `Propagated` error —
  that is robust, legible failure, not a violation; what C1 forbids is the error vanishing.** A budget
  overrun is itself a legible `EffectBudgetExhausted`, not a silent stall.
- **C2 — honest per-op tag (VR-5).** `handle`/`recover` **inherit** the recovered value's honest guarantee
  and never upgrade it; a `fallback` is at most `Declared` (I2); `retry` carries the successful attempt's own
  tag; an `Ok`/`escalate` pass-through preserves the existing tag. The policy/ledger ops are `Exact` (no
  accuracy semantics). Downgrade is the rule; this module has **no path** to launder a weaker guarantee into
  a stronger one.
- **C3 — no black boxes / EXPLAIN (SC-3/G11).** The recovery policy is a **reified, content-addressed
  artifact** (`PolicyRef` = its `ContentHash`, RFC-0005/ADR-006 pattern): every recovered or re-propagated
  outcome **records the acting `PolicyRef`**, so *"which policy acted on this error, and what does it do?"*
  is always answerable. The on-`ErrorClass` mapping is an inspectable, diffable map (sorted, content-hashed);
  the chosen action is one of a **closed**, named set; the effect budgets are themselves `EXPLAIN`-able
  (kind + bound). Nothing about the selected action or the budget decision is hidden or ambient.
- **C4 — content-addressed, value-semantic (ADR-003 / RFC-0001).** `Outcome`/`Resolution`/`RecoveryPolicy`
  are immutable value sums; `handle` is a pure function of `(outcome, policy, budgets, attempt)` plus its
  *declared* effects (the `attempt` thunk and the budget ledger are the only effect surface, and both are
  explicit). The `PolicyRef` is a deterministic content hash over the policy's canonical sorted rules
  (BLAKE3 in the origin); the guarantee tag and `PolicyRef` ride the outcome as **metadata, which is not
  identity** (ADR-003).
- **C5 — above the small kernel (KC-3 / NFR-7).** `recover` adds **no trusted code and no new L0 node**:
  handling **elaborates to an existing `Match`** over the error sum (RFC-0014 §4.3 — the
  `Match`-over-error-sums lowering target is already differentially verified in `mycelium-l1`); the policy
  and driver are ordinary functions/values; budget enforcement lives in `mycelium-interp` (M-353), not in
  the kernel calculus. The self-hosted form is held against the Rust origin by a differential (NFR-7). No
  `wild`/FFI (ADR-014).
- **C6 — declared, bounded effects (RFC-0014).** Recovery actions carry **declared, budgeted** effects:
  `retry` declares `retry` bounded `Attempts(N)`; `cleanup_then_propagate` declares its `effect` bounded in
  the ledger; a clean `Ok`/`fallback`/`escalate` path is effect-free. `check_effects` enforces I3 (no
  undeclared effect). Every budget overrun is an explicit, graceful `EffectBudgetExhausted` (I4), routed
  through the **one** enforcement mechanism (`mycelium-interp`, M-353) the runtime uses for fuel/depth — and
  the default is tightly scoped: an effect with no declared budget cannot run (I5).

## 6. Grounding

- The reified recovery subsystem this module self-hosts — the result-sum `Outcome`, the **closed** action
  set, the **on-`ErrorClass`** reified policy, declared + budgeted effects, the never-silent `handle`, and
  the invariants **I1–I5** — is **RFC-0014 §4.1–§4.8 (Accepted — Enacted, M-352/M-353)**. The Mycelium-lang
  port mirrors the landed Rust `crates/mycelium-lsp/src/recover` (M-352: `Outcome`/`Resolution`/
  `StructuredError`, `RecoveryAction`, content-addressed `RecoveryPolicy`, `handle`) and the shared budget
  primitive `mycelium_interp::budget` (M-353: `EffectKind`/`EffectBudget`/`Budgets`/`EffectBudgetExhausted`,
  `check_effects`).
- The **never-silent** spine is **RFC-0014 §4.2 I1** (a handler recovers explicitly or re-propagates; an
  error can neither vanish unobserved nor be dropped) and **RFC-0016 §4.1 C1** (G2). The **honest tag**
  discipline is **RFC-0014 I2** + **VR-5 / RFC-0016 C2** (a fallback is at most `Declared`; recovery only
  downgrades).
- The **declared, bounded effects** discipline — declared (I3), budgeted with a graceful overrun (I4),
  tightly scoped/opt-in (I5), one enforcement mechanism over separate named budgets — is **RFC-0014 §4.5**
  and its §8 dispositions, enacted in **M-353** (the ledger lifted into `mycelium-interp`, an overrun is
  `EvalError::EffectBudget` beside `FuelExhausted`/`DepthLimit`).
- **No new kernel node / KC-3 / NFR-7:** handling = L0 `Match` over the error sum is **RFC-0014 §4.3/§4.8**
  (differentially verified in `mycelium-l1`); the self-hosting trajectory and the differential-against-Rust
  acceptance are **RFC-0014 §9 (self-hosting)** + **NFR-7** + the **milestones.json Phase-5 charter**
  ("self-host the RFC-0013/0014 diagnostics + recovery").
- The **diag boundary** (`recover` builds on the diagnostic record/registry; it acts, diag presents) is
  **RFC-0014 §4.9** + **RFC-0013 §4.5** (the shared error-class registry, resolved — never an eval'd string,
  X1) as delivered by **`std.diag` (M-510)**.
- The **error bridge** this module owns the concrete signature for is **`std.error` (M-527)** `error.md` §7-Q1
  (the `RecoverOutcome`/`PolicyRef` it described abstractly and deferred here) + **RFC-0016 §5** ("the
  recovery bridge" seam).
- The module's task, ring (1), tier (A), and the §4.5 guarantee-matrix obligation are **RFC-0016
  §4.2/§4.3/§4.5**; the C1–C6 contract is **RFC-0016 §4.1**. The value model + lattice + content-addressing
  are **RFC-0001** (§4.3 lattice, §4.6 content-addressing); the reified-policy pattern is **RFC-0005 /
  ADR-006**; metadata-is-not-identity is **ADR-003**.

## 7. Open questions (FLAGGED — resolve before ratification)

- **(Q1) The concrete `RecoverOutcome` / `PolicyRef` bridge signature — OWNED here, RESOLVED, awaiting
  `error` reconciliation.** This module fixes it (§3): `RecoverOutcome<τ> = Resolution<τ> = Recovered { value,
  guarantee, policy } | Propagated { error, policy }` (no drop variant — I1), `PolicyRef = ContentHash`, and
  the bridge op `recover` *is* `handle`. *Disposition:* this is the resolution of `error.md` §7-Q1 / the
  RFC-0016 §5 "recovery bridge" seam; the orchestrator should reconcile the two specs to this signature
  (`error` keeps the abstract description; `recover` is the source of truth). The contract `error` stated —
  `Recovered | Propagated`, no drop, honest inherited tag — holds verbatim, so the reconciliation is naming,
  not redesign.
- **(Q2) Surface-language expressibility — the FLAGGED self-hosting dependency.** The self-hosted port
  **depends on the Mycelium surface supporting (a) an error-sum `Match`** (the L0 lowering target recovery
  elaborates to — RFC-0014 §4.3) **and (b) the effect-budget surface (M-353)** being *expressible in
  Mycelium-lang* (declaring an effect set + per-kind budget on a signature, spending it through the ledger).
  The error-sum `Match` is already differentially verified at L0 (`mycelium-l1`), but the **surface spelling**
  of effect annotations + budgets is KC-2-gated (RFC-0006) and **not yet committed**. *Disposition:* FLAGGED
  as a genuine dependency; this spec does **not** invent the effect-annotation/`Match` syntax — it commits
  only to the *semantics* (the closed action set, the budgeted-effect discipline) and threads whatever
  surface RFC-0006 / M-353 fix. Blocks the self-hosted *implementation*, not this design.
- **(Q3) The self-hosting differential bar (NFR-7) — what must match?** Acceptance requires the self-hosted
  `recover` to differential against the Rust origin. Must it match *observable outcomes* (`Recovered`/
  `Propagated` value + class), or also the *exact* honest tag and the *exact* `PolicyRef` content hash
  bit-for-bit (which would pin the canonical hashing — BLAKE3 over sorted rules — into the self-hosted form)?
  *Disposition:* FLAGGED; ties to **RFC-0016 §8-Q5** (the migration differential's bar) and
  `self-hosting-readiness.md` (M-502). The honesty floor (I1, honest-tag I2) holds under either bar; the open
  question is the *strength* of the equivalence, not whether one is required.
- **(Q4) `cleanup_then_propagate` budget-overrun visibility.** In the Rust origin, a `cleanup_then_propagate`
  whose cleanup effect overruns its budget **swallows the `EffectBudgetExhausted` *for the cleanup only*** and
  propagates the original error regardless (I1-correct — the original error never vanishes). But the *cleanup's*
  failure is currently not surfaced in the outcome. *Disposition:* FLAGGED — should the swallowed cleanup
  overrun be **recorded in the outcome** (e.g. an annotation on the `Propagated` error, or via the `diag`
  record) so it is legible rather than discarded? The original error's I1 holds either way; this is about the
  *cleanup's* own legibility. Ties to the `diag` (M-510) seam and RFC-0014 §4.9. Recommend recording it (the
  failure-legibility directive favors surfacing it).
- **(Q5) Ergonomics vs. the contract at the recovery call site.** Whether the acting `PolicyRef` / the chosen
  action / the budget state must be *always explicit at the call site* or *implicit-but-inspectable* (recorded,
  surfaced on demand via `EXPLAIN`) is the project-wide tension A. *Disposition:* FLAGGED to **RFC-0016 §8-Q3**
  (ergonomics vs. the contract) — the same recurring item `swap`/`select`/`error` flag; needs one per-ring
  design pass, not a per-module answer. C1/C3 hold under either choice (the outcome always records its
  `PolicyRef`; the only question is call-site visibility).

## Meta — changelog

- **2026-06-17 — Draft (needs-design).** Stands up the `std.recover` (M-520, #156; Ring 1, Tier A) module
  spec under RFC-0016 (Draft): the **self-hosted** Mycelium-lang form of the RFC-0014 reified recovery
  subsystem (Accepted — Enacted; Rust origin `crates/mycelium-lsp/src/recover`, M-352, + the shared budget
  primitive `mycelium_interp::budget`, M-353). Fixes the **scope + boundary** (the result-sum `Outcome`, the
  **closed** action set `fallback`/`retry`/`escalate`/`cleanup_then_propagate`, the registry-shared
  on-`ErrorClass` reified policy + content-addressed `PolicyRef`, declared + budgeted effects with a graceful
  `EffectBudgetExhausted`, and the never-silent `handle`; bounded against `diag`'s diagnostic *record/trace*
  (M-510, which `recover` builds **on**), `error`'s *combinator ergonomics* (M-527, which **bridges into**
  this module), and `mycelium-interp`'s budget *enforcement* (M-353)). Owns and **resolves the concrete
  `RecoverOutcome`/`PolicyRef` bridge signature** `error.md` §7-Q1 deferred (`RecoverOutcome<τ> =
  Resolution<τ> = Recovered | Propagated`, no drop variant — I1; `PolicyRef = ContentHash`; the bridge op *is*
  `handle`). Delivers — the load-bearing deliverable — the per-op **guarantee matrix** (`handle`/`recover`,
  the four closed actions, policy registration, the budget ledger, `check_effects`), with the central
  property that **no row permits a silent drop** (RFC-0014 **I1** / RFC-0016 **C1**): `handle` inherits an
  *honest* tag (never upgraded — I2), the matrix encodes **I1/I3/I4/I5**, and a budget overrun is a graceful,
  explicit `EffectBudgetExhausted` (I4). States the **§4.1 conformance** (C1 the spine — the never-silent
  handle: every error recovered or re-propagated, *never dropped*; a program **may** legitimately halt on a
  `Propagated` error — robust, legible failure, not a violation; C5/NFR-7 — recovery elaborates to L0 `Match`,
  no new kernel node), the **grounding** (RFC-0014 §4.1–§4.9/I1–I5, M-352/M-353, RFC-0016 §4.1–§4.5, RFC-0013
  §4.5/X1, RFC-0001/RFC-0005/ADR-003/ADR-006, NFR-7), and **five FLAGGED questions** (the bridge signature —
  resolved here, awaiting `error` reconciliation; the surface-language `Match`/effect-budget expressibility
  dependency — M-353/RFC-0006; the self-hosting differential bar — RFC-0016 §8-Q5; `cleanup_then_propagate`
  overrun visibility; the call-site ergonomics-vs-contract tension — RFC-0016 §8-Q3). **No code; no kernel
  change (KC-3 — recovery elaborates to an existing `Match`, no new L0 node).** Append-only.
- **2026-06-18 — Implemented (Rust-first half), pending ratification.** The Rust-first library half landed as `mycelium-std-recover` (M-520, #156; Batch P5 Tier-A completion, octopus-merged): the `Outcome`/`Resolution` sums (**no `Dropped` variant** — I1), the closed v0 `RecoveryAction` set, the content-addressed `RecoveryPolicy`/`PolicyRef`, declared + budgeted effects via `mycelium_interp::budget` (graceful `EffectBudgetExhausted`), and the never-silent `handle_classified` driver carrying a `mycelium_diag::Diag`. The recovered tag is honest (`Ok`→floor, `fallback`→`Declared`, `retry`→inherited; never laundered up — I2/VR-5; **fixes the P5-B exact-tag bug**); the §4.5 matrix + the never-drops / tag-≤-policy / budget-bounded invariants are property-tested. FLAGGED for fast-follow: re-export the concrete `Outcome` from `std.error`'s abstract stub (§7-Q1); add `EffectBudget::Io`/`Named` variants to `mycelium-interp` for cleanup budgets on those effect kinds. The **self-hosting half** (Mycelium-lang) stays **Batch P5-C, M-502-gated**. No new kernel node (KC-3 — recovery elaborates to an existing `Match`). Append-only.
- **2026-06-19 — B1 (EffectBudget Io/Named gap) RESOLVED.** `mycelium-interp::budget::EffectBudget`
  gains `Ops(u64)` (→ `EffectKind::Io`) and `Named(String, u64)` (→ `EffectKind::Named`) — one budget
  variant per effect kind — so `cleanup_then_propagate` budgets an `Io`/`Named` cleanup effect
  **directly** (the M-520 leaf's Retry/Attempts proxy is removed). `EffectBudget` is no longer `Copy`
  (the `Named` variant carries its name). Additive kernel change, property-tested both sides; the
  ledger/`consume` enforcement path is unchanged (KC-3 — no new L0 node). Append-only.

- **2026-06-20 — Accepted (Rust-first half) (maintainer ratification, DN-07).** The maintainer ratified this Rust-first spec: the §4.5 guarantee matrix is asserted in tests, never-silent fallibility and honest per-op tags hold, and the open §7/§8 questions are design/scope calls, not contract violations. No guarantee tag was upgraded without a checked basis (VR-5). The self-hosted half stays M-502-gated. Status moves *Implemented (Rust-first half) — pending ratification → Accepted (Rust-first half)*. Append-only; no kernel change (KC-3).
