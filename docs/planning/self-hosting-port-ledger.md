# Self-Hosting Port Ledger — requisite components & logic for the Mycelium-native reimplementation

> **Status:** living planning ledger (append-only; extended as components land). **Advisory** — it
> records intent for the future self-hosting effort; it decides nothing normatively.
>
> **Owners / related:** epic **E18-1** (self-hosting capstone), **DN-26** (self-hosting bootstrap
> plan), **DN-14** (self-hosting gate), the **`boot10`** kickoff. See also `docs/notes/DN-25`
> (road to full-language 1.0.0).

## Purpose

The Rust kernel + reference interpreter is **scaffolding**: the 1.0.0 arc lands components Rust-first
because that is the fastest way to get them *correct*, but the long-term goal (E18-1, ADR-022 §10) is
a **self-hosted implementation in Mycelium-lang itself**. Every Rust-reference component is therefore
a *future Mycelium port*.

This ledger captures — **as each component lands** — the logic + interfaces the self-hosting effort
must reproduce, and flags any **external Rust dependencies** whose semantics must be reimplemented
natively (they are not directly portable to a Mycelium that does not depend on Rust crates). The
point is that the port works from a **spec**, not from archaeology of the Rust encoding.

**Discipline:** at every wave integration, add/refresh the component's row. Keep it honest — a logic
summary is `Empirical/Declared` unless it points at a checked spec; never claim a port is trivial
that isn't.

## External-dependency port considerations

External Rust crates used in the reference are **not** part of the self-hosting surface. Where the
reference leans on one, the self-hosted version must reproduce its *semantics* on Mycelium's own
runtime/stdlib.

| Rust dep | Used by (Rust ref) | Semantics to reproduce | Native Mycelium target |
|---|---|---|---|
| **rayon** | M-860 parallel AOT codegen; M-862 parallel pure-fragment eval | data-parallel `map`/`join` over independent work items with deterministic re-assembly | the native runtime **work-stealing scheduler** (M-861, `mycelium-std-runtime`) — express per-item fan-out over hyphae/colony; determinism via a stable content-key ordering, not thread order |

**On the rayon choice (2026-07-01, maintainer-noted).** Selecting `rayon` over a hand-rolled
`std::thread::scope` implementation is deliberate and *correct for the Rust reference* — it gets the
parallel codegen/eval right faster with a mature, well-tested work-stealing engine. It is **not tech
debt in the Rust reference.** It *is* a **self-hosting port item**: the dogfooded implementation
cannot pull in a Rust crate, so it reimplements the parallel-map/join pattern on Mycelium's own
scheduler (which already has native work-stealing from M-861). Recorded here so the port cost is
visible, not discovered late.

## Component ledger

Per landed component: its logic, the Rust module that hosts it, external deps, and the native-port
consideration. Extended per wave.

| Component | Rust crate/module | Logic summary | External deps | Native-port consideration | Tracking |
|---|---|---|---|---|---|
| Parallel AOT codegen | `crates/mycelium-mlir` (per-function lowering) | independent functions lowered concurrently; stable content-key sort → byte-identical output vs sequential | rayon | reimplement parallel-map on the native scheduler; keep the deterministic ordering (content key, not thread order) | M-860 |
| Parallel pure-fragment eval | `crates/mycelium-interp` (env-machine) | effect-free fragments evaluated concurrently, RT2-preserving; differential vs sequential = identical values | rayon | same native-scheduler port; the purity/effect analysis that gates parallelism is itself a component to port | M-862 |
| *Wave-1 components (M-856b dialect Dense/VSA, M-743 license gate, M-719 stdlib freeze, M-674 recursion budgets)* | *(fill at integration)* | *(fill at integration — logic + interfaces)* | *(note any)* | *(native-port consideration)* | Wave 1 |

<!-- Append new rows below as components land. Keep the External-dependency table in sync. -->

## How this feeds the self-hosting effort

When E18-1 / `boot10` begins porting a subsystem, its first input is this ledger's row(s): the logic
to reproduce and the native substitutes for any Rust dep. A row that is thin is a signal to write the
component's spec *before* porting (a DN or a `docs/spec/` entry), not to reverse-engineer the Rust.
