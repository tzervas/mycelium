# Spec — Standard Library module specs (`std`)

| Field | Value |
|---|---|
| **Status** | **Living index** (2026-06-17) — the per-module design specs that decompose **RFC-0016** (the Core Library RFC). Each module spec is `Draft (needs-design)` until its task's acceptance is met and the maintainer ratifies. |
| **Scope** | The home of the per-module standard-library design specs (`docs/spec/stdlib/<module>.md`). Each spec fixes a module's **scope + boundary**, its **exported-op surface**, and — the load-bearing deliverable — its **guarantee matrix** (RFC-0016 §4.5), proving the module meets the §4.1 contract **per op**, as a checked table rather than prose. |
| **Source of truth above this dir** | `docs/rfcs/RFC-0016-Core-Library-and-Standard-Library.md` (the **scope + per-op contract + taxonomy**); `docs/planning/phase-5.md` (the task decomposition M-510…M-534) |
| **Conformance template** | `docs/spec/stdlib/_TEMPLATE.md` — every module spec follows it (single-template conformance, the §4.1 doc quality-bar lint) |
| **Depends on** | RFC-0016 (the contract + taxonomy); RFC-0001 (the value model — `Value`/`Repr`/`Meta`, the guarantee lattice, content-addressing §4.6); the per-module Accepted RFC/ADR each spec grounds in |

---

## 1. What a module spec is (and is not)

A module spec is **design-first**: it fixes *what the module is, what every exported op promises, and how
that promise is checked* — not a Rust or Mycelium-lang implementation. It is the per-module deliverable the
Phase-5 tasks (`M-510…M-534`) name. Nothing here enlarges the trusted base (KC-3): the stdlib lives **above**
`mycelium-core` and the capability crates, as a certificate/EXPLAIN **consumer**.

The non-negotiable spine of every spec is the **§4.1 contract** (RFC-0016), lifted from the language's own
house rules to library scope. Every exported op must satisfy **all** of:

- **(C1) Never-silent (G2).** Every fallible op returns an explicit `Option`/`Result`/refusal that
  *propagates*; out-of-range / malformed / unsupported input is an explicit error — never a sentinel, a
  silent clamp, or a partial result.
- **(C2) Honest per-op guarantee tag (VR-5).** Any op carrying accuracy/precision/probability semantics
  tags it on `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` in its `Meta`. `Proven` only with a theorem whose
  side-conditions are *checked*; otherwise `Empirical` or `Declared`. **Downgrade to stay honest; never
  upgrade without a checked basis.** An op with no accuracy semantics (e.g. `len`) is `Exact`.
- **(C3) No black boxes / EXPLAIN (SC-3/G11).** Any op that *selects*, *converts*, or *approximates*
  exposes *why* via a reified, inspectable artifact (an RFC-0005 policy + EXPLAIN record, a swap
  certificate, a diagnostic record).
- **(C4) Content-addressed, value-semantic (ADR-003 / RFC-0001).** Data structures are immutable values
  with content-addressed identity where it applies; an op is a pure function of its inputs + declared
  effects. Metadata is **not** identity.
- **(C5) Above the small kernel (KC-3).** The module consumes the kernel/capability crates but never
  enlarges the trusted base. Any `wild`/FFI (ADR-014) is confined to an audited `wild` block and is
  inventoried.
- **(C6) Declared, bounded effects (RFC-0014).** An op with effects (IO, time, randomness) **declares**
  them on its signature; unbounded effects carry an explicit budget where one applies. No undeclared side
  effect.

## 2. The guarantee matrix (every module ships one)

The load-bearing artifact. Rows = exported ops; columns =
`{ guarantee tag · fallibility (the explicit error set) · declared effects · EXPLAIN-able? }`. The
RFC-0003 §4 matrix is the proven template — encoded as data, asserted in tests, **never prose only**. The
matrix is how C1/C2/C3/C6 are *verified* rather than claimed.

## 3. Ring layering (RFC-0016 §4.2)

- **Ring 0 — kernel-adjacent re-exports** (`core`/prelude): the value model, `Option`/`Result`/error
  values, the guarantee-lattice types. Thin, mostly re-exporting `mycelium-core`. *No new trusted code.*
- **Ring 1 — capability surfaces** (Tier A): ergonomic libraries over the landed capability crates
  (`numerics`, `swap`, `vsa`, `dense`, `select`, `diag`, `recover`, `ternary`, `content`). Certificate /
  EXPLAIN **consumers**.
- **Ring 2 — general library** (Tier B): collections, text, math, iter, error, io, etc., written to the
  contract over Ring 0/1.

## 4. Module spec index

### Keystone & gate

| Doc | Task | Role | Status |
|---|---|---|---|
| [`../../rfcs/RFC-0016-Core-Library-and-Standard-Library.md`](../../rfcs/RFC-0016-Core-Library-and-Standard-Library.md) | M-501 | the contract + taxonomy keystone (every spec traces to its §4.1) | **Draft** — ratification is the maintainer's decision |
| [`self-hosting-readiness.md`](./self-hosting-readiness.md) | M-502 | the *checkable* self-hosting verdict — gates the Mycelium-lang migration half (RFC-0016 §4.6), not the Rust-first specs/impls | **Draft (needs-design)** — verdict: *not yet established* |

**Wave status:** `Draft — landed` = the spec is authored + integrated in the first orchestration wave (and
awaits maintainer ratification with RFC-0016); `anticipated` = in the RFC-0016 taxonomy, scheduled for a
later wave. Cross-module FLAGs reconciled in §5.

### Tier A — differentiator modules (RFC-0016 §4.3)

| Module | Spec | Task | Grounding | Wave status |
|---|---|---|---|---|
| `core` / prelude | [`core.md`](./core.md) | M-515 (#157) | RFC-0001 | **Draft — landed** |
| `swap` | [`swap.md`](./swap.md) | M-516 (#158) | RFC-0002; M-120/210/211/231 | **Draft — landed** |
| `ternary` | [`ternary.md`](./ternary.md) | M-517 (#159) | FR-M2; M-111; RFC-0004 §5 | **Draft — landed** |
| `dense` | [`dense.md`](./dense.md) | M-518 (#160) | RFC-0001 §4.1; M-230 | **Draft — landed** |
| `select` / `explain` | [`select.md`](./select.md) | M-519 (#161) | RFC-0005/ADR-006; M-220/221/222 | **Draft — landed** |
| `content` / `hash` | [`content.md`](./content.md) | M-523 (#164) | ADR-003; RFC-0001 §4.6 | **Draft — landed** |
| `numerics` | — | M-512 | ADR-010; M-201/202/203 | anticipated |
| `vsa` / `hdc` | — | M-513 | RFC-0003/0009 | anticipated |
| `diag` | — | M-510 | RFC-0013; M-345 | anticipated |
| `recover` | — | M-520 | RFC-0014; M-352 | anticipated |
| `runtime` / `colony` | — | M-521 | RFC-0008 | anticipated — Phase-7-gated (§8-Q4) |
| `spore` | — | M-522 | ADR-013; M-368 | anticipated |

### Tier B — common / expected modules (RFC-0016 §4.4)

| Module | Spec | Task | Honesty crux | Wave status |
|---|---|---|---|---|
| `iter` | [`iter.md`](./iter.md) | M-526 (#167) | total/terminating where the kernel guarantees it | **Draft — landed** |
| `math` | [`math.md`](./math.md) | M-525 (#166) | rounding/approx ops carry their tag | **Draft — landed** |
| `error` / `option` / `result` | [`error.md`](./error.md) | M-527 (#168) | propagation is the floor (I1) | **Draft — landed** |
| `cmp` / `convert` | [`cmp.md`](./cmp.md) | M-532 (#172) | lossy convert is explicit + fallible | **Draft — landed** |
| `fmt` | [`fmt.md`](./fmt.md) | M-533 (#173) | dual human/machine projection (G11) | **Draft — landed** |
| `collections` | — | M-511 | value-semantic; no silent reorder | anticipated |
| `text` / `string` | — | M-524 | `parse` → `Result`, lossy encoding explicit | anticipated |
| `io` + `serialize` | — | M-514 | substrate single-consumption (LR-8) | anticipated |
| `fs` | — | M-528 | every path/permission failure explicit; `wild` floor (§8-Q6) | anticipated |
| `time` | — | M-529 | monotonic vs wall a typed distinction | anticipated |
| `rand` | — | M-531 | nondeterminism reified/named (RT3) | anticipated |
| `testing` | — | M-534 | a skipped check is reported, never a silent pass | anticipated |

## 5. Cross-module reconciliation (first design wave)

The wave authored each spec independently, so the seams **between** modules are reconciled here (the
orchestrator's deconfliction job). Each spec FLAGs its own open questions in its §7; below are only the
points that span modules. Most map onto an existing **RFC-0016 §8** question — recorded for the maintainer
to resolve at ratification, **not** silently decided here (the planning analogue of G2).

| Seam | Modules | Reconciliation | Maps to |
|---|---|---|---|
| **The swap ↔ convert boundary** | `swap` (M-516), `cmp`/`convert` (M-532) | **Mostly consistent — one residual placement FLAGGED.** The clear cases agree: a certified *representation* change (binary↔ternary, M-120; the lossy `F32→BF16`, M-211; `Dense↔VSA`, M-231) lives in **`swap`** (certificate-carrying), and ordinary same-paradigm widening/narrowing (`i8→i32`; fallible `i32→i8`) lives in **`cmp`/`convert`**. The **one open sub-seam** is the *lossless reverse* `BF16→F32` widening: `swap.md` §7 FLAGs its placement and proposes it lift to `cmp`/`convert` (lossless float widening, no certificate needed) rather than `swap` — not yet ratified. No op is double-owned today; the `BF16→F32` owner is the FLAG to close. | **FLAGGED** (swap §7-Q3 / cmp §7-Q2) |
| **Numeric ε bounds ownership** | `dense` (M-518), `math` (M-525) | **Consistent deferral.** Both route float-op ε through the verified numerics (`std.numerics`, M-512 / ADR-010) and **cite, never restate** the bound; both tag `Proven` *only* where Higham's side-conditions are checked, else honestly downgrade. The concrete ε constants are M-512's to fill — neither spec fabricated them. | §8-Q1 (module set) |
| **JSON projection overlap** | `fmt` (M-533), `serialize` (M-514, *anticipated*) | **Deferred to when `serialize` lands.** `fmt.to_json` and `serialize`'s JSON both claim "dual human/machine projection". Proposed: one canonical JSON projection that `fmt` delegates to; reconcile when M-514 is authored. | §8-Q1/§8-Q3 |
| **The recovery bridge** | `error` (M-527), `recover` (M-520, *anticipated*) | **Co-design flag.** `error`'s `recover`-bridge signature (`RecoverOutcome`/`PolicyRef`) is owned by `std.recover` (RFC-0014); `error` described it abstractly without fabricating it. Reconcile the exact signature when M-520 lands. | §8-Q1 |
| **content-hash vs hash-for-maps** | `content` (M-523), `collections` (M-511, *anticipated*) | **Boundary stated.** `content` owns *identity* hashing (canonical content-addressing, ADR-003); `collections` owns *non-identity* hashing-for-maps. Kept distinct. | — |
| **Early-termination over a total `for` fold** | `iter` (M-526) → RFC-0007 §4.8 | **Question back to the kernel.** Short-circuit combinators (`any`/`all`/`find`) over the no-`break` total fold either use a done-flag fold (total, walks the full spine) or motivate an early-termination kernel primitive. FLAGGED to RFC-0007, not decided here. | RFC-0007 §4.8 |
| **Naming** (`Bit`/`Trit`, the `std` phylum, re-export names, the canonical error-value identifier) | `core`, `ternary`, `swap`, `content` | All defer to the DN-02/06 lexicon decision; `core` and `error` must agree the **one** error-value name; no spec committed a name the corpus hasn't ratified. | **§8-Q2** |
| **Ergonomics vs the contract** (always-explicit EXPLAIN/certificate/tag/identity-ref at the call site vs implicit-but-inspectable) | `swap`, `select`, `content`, `math`, `error`, `iter`, `fmt` | The single most recurrent tension (tension A). Every affected spec FLAGs it rather than choosing; needs one per-ring design pass, not seven per-module answers. | **§8-Q3** |
| **The migration differential's bar** | `swap`, `self-hosting-readiness` (M-502) | What a self-hosted module must match (observable results vs tags+EXPLAIN bit-for-bit) before its verdict flips. | **§8-Q5** |
| **`wild`/FFI for transcendentals** | `math` (M-525) | Whether `math`'s transcendental floor is a pure trusted routine or libm via `wild` (which would narrow its C5 "no `wild`" claim). | **§8-Q6** |

**Net:** no two specs conflict on an *owned* surface; the only unsettled ownership is the lossless
`BF16→F32` placement (swap §7-Q3 / cmp §7-Q2), FLAGGED above rather than silently assigned. The recurring
items are the *known* RFC-0016 §8 questions (naming §8-Q2, ergonomics §8-Q3), now corroborated from eleven
independent angles — useful signal for the maintainer's ratification pass.

## 6. How this index stays honest

- **Append-only with status transitions**, mirroring the ADR/RFC discipline: a module row moves
  `design landing → Draft (needs-design) → Accepted` only as the spec actually lands and is ratified;
  it never pre-records a module as done.
- **Every spec traces to RFC-0016 + its Accepted corpus**; a module whose Mycelium-specific intent is
  unclear is a FLAGGED open question (RFC-0016 §8), never a silently-invented design choice (the planning
  analogue of never-silent, G2).

## Meta — changelog

- **2026-06-17 — Created (Living index).** Stands up the per-module standard-library spec directory under
  **RFC-0016 (Draft)**: the §4.1 contract reference (C1–C6), the guarantee-matrix obligation (§4.5), the
  ring layering (§4.2), the single-template conformance rule, and the module index keyed to the Phase-5
  tasks (M-510…M-534). Marks the first orchestration wave's `design landing` set (Tier-A differentiators
  `core`/`swap`/`ternary`/`dense`/`select`/`content` + Tier-B pure commons `iter`/`math`/`error`/`cmp`/
  `fmt`); the remainder are `anticipated` for later waves. No code; no kernel change (KC-3). Append-only.
