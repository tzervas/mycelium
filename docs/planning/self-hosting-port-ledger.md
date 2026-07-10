# Self-Hosting Port Ledger — requisite components & logic for the Mycelium-native reimplementation

> **Status:** living planning ledger (append-only; extended as components land). **Advisory** — it
> records intent for the future self-hosting effort; it decides nothing normatively.
>
> **Owners / related:** epic **E18-1** (self-hosting capstone), **DN-26** (self-hosting bootstrap
> plan), **DN-14** (self-hosting gate), the **`boot10`** kickoff. See also `docs/notes/DN-25`
> (road to full-language 1.0.0).
>
> **Effort/usage forecast:** `docs/planning/dogfooding-effort-and-usage-assessment.md` sizes this track
> (~45M-token floor for the LOC port; realistic all-in ~70–120M once capability-build + differential
> validation are included) — a `Declared` model, refine it to `Empirical` as the first Tier-A ports land.
>
> **Roadmap for ADR-036 (2026-07-01 — maintainer-ratified).** This ledger is the roadmap for the
> project's **comprehensive-dogfooding track**: build each Mycelium-native component **beside** its
> Rust reference → **Rust≡Mycelium differential validation** (extending the interp≡AOT≡JIT
> discipline, RFC-0029 §7.5/M-210) → **replace** the Rust original once tested/benched/validated and
> maintainer-satisfied → the project's **public release** (gated on this track's completion; the
> `lang 1.0.0` tag itself is cut on the Rust reference and is not gated by this ledger's completion
> beyond the existing core-lib self-host slice, ADR-022 §8 Q1). See ADR-036.

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
| ~~**rayon**~~ **— DISSOLVED (2026-07-01)** | *(historical only — see note below; no current user)* | *(no longer applicable)* | *(no longer applicable)* |

**On the rayon choice — SUPERSEDED, the port item is dissolved (2026-07-01, Wave-1 reconciliation).**
The entry above originally recorded `rayon` (M-860's first pass, commit `a8e608e`) as a deliberate,
correct-for-the-Rust-reference choice that would nonetheless need a self-hosting port later. That
premise no longer holds: the SAME DAY, once the M-861 work-stealing Scheduler was **relocated to a
new foundational crate, `mycelium-sched`** (PR #864, below `mycelium-interp`, breaking the
`interp`↔`std-runtime` dependency cycle), both M-860 (commit `b6107b8`) and M-862 (commit `ed2ac9c`)
were reworked to dispatch through `mycelium_sched::scheduler::Scheduler::run_indexed` directly, and
`rayon` was **removed entirely** (confirmed: zero `rayon` references in `Cargo.lock` or any workspace
`Cargo.toml`, 2026-07-01). **There is nothing left to reproduce for self-hosting on this axis** — the
Rust reference itself no longer depends on an external crate here; it depends on `mycelium-sched`, a
**first-party** crate that is itself squarely inside the self-hosting effort's own future-port surface
(the runtime/scheduler, not a third-party semantics-reproduction exercise). This is a genuine
reduction in self-hosting scope, not just a rename: porting `mycelium-sched`'s `Scheduler` follows the
project's own conventions and is tracked under the ordinary runtime-component port path below, rather
than as a bespoke "reimplement rayon" exercise.

## Component ledger

Per landed component: its logic, the Rust module that hosts it, external deps, and the native-port
consideration. Extended per wave.

| Component | Rust crate/module | Logic summary | External deps | Native-port consideration | Tracking |
|---|---|---|---|---|---|
| Parallel AOT codegen | `crates/mycelium-mlir` (per-function lowering) | independent functions lowered concurrently; stable content-key sort → byte-identical output vs sequential | **none** (was rayon; dissolved 2026-07-01 — see note above) | **Already native** as of 2026-07-01: dispatches via `mycelium_sched::scheduler::Scheduler::run_indexed`, a first-party crate. Nothing rayon-shaped to reproduce; the remaining port item is the ordinary one of porting `mycelium-sched`'s `Scheduler` itself to Mycelium-lang (tracked with the rest of the runtime, not as a bespoke exercise) | M-860 |
| Parallel pure-fragment eval | `crates/mycelium-interp` (env-machine) | effect-free fragments evaluated concurrently over the outermost independent argument batch (no nesting — the current interim cap), RT2-preserving; differential vs sequential = identical values; choice reified in an EXPLAIN-able `ParallelPlan` | **none** (was rayon; dissolved 2026-07-01 — see note above) | **Already native**, same substrate as M-860 above (`Scheduler::run_indexed`). The purity/effect analysis that gates parallelism (`is_pure`) is itself a component to port; the bounded-to-top-level-batch limitation (no nested submission) carries forward to the self-hosted version until M-864 lands | M-862 |
| MLIR-dialect Dense/VSA codegen | `crates/mycelium-mlir/src/dialect/native/{dense,vsa}.rs` | lowers Dense/VSA ops through the real `arith`/`func`/`math` MLIR dialect, mirroring `dense_codegen.rs`/`vsa_codegen.rs` | libMLIR (`mlir-opt`/`mlir-translate`, external toolchain, not a Rust crate) | **No self-hosting port item** — this is AOT-backend codegen that drives the real LLVM/MLIR C++ toolchain; it stays Rust/FFI-hosted regardless of whether the frontend/checker/stdlib self-host (analogous to how a self-hosted compiler still typically keeps its LLVM integration in a host language). Noted here so the omission is explicit, not silently skipped | M-856b |
| Stdlib stable-API freeze | `crates/mycelium-std-*` (all 26 crates) | dated, grounded snapshot of the current public-API + guarantee-matrix baseline for every `mycelium-std-*` crate — **this snapshot is exactly the spec the self-hosting `.myc` ports must match**, per crate | n/a (documentation/audit, not code) | **Direct input to the self-hosting effort**: DN-66's per-crate table (spec status, `GUARANTEE_MATRIX` location, public-surface size) is the reference each `.myc` port's D5/D6 conformance bar is checked against; DN-66 §3 also grounds why the 5 existing same-named `.myc` prototypes don't yet qualify as full ports | M-719 (DN-66) |
| Recursion-depth budgets (totality + ambient passes) | `crates/mycelium-l1/src/{totality,ambient}.rs` | explicit `MAX_WALK_DEPTH`/`MAX_AMBIENT_DEPTH` (4096) budgets on the passes' own AST-descent, refusing cleanly past them rather than relying on a host-stack limit | none | **Self-hosting-portable by the issue's own design intent**: the explicit-budget discipline (vs. relying on the host call stack) is exactly the pattern a Mycelium-native frontend needs, since a self-hosted checker/elaborator has no Rust host stack to fall back on. The `mycelium-stack` deep-worker-stack adapter is explicitly transitional (Rust-hosted only); the budget itself is the portable primitive. **Known gap, not yet closed:** the sibling `mono.rs` `free_vars`/`pattern_binders` recursion remains unbounded — a follow-up, out of this item's own scope | M-674 |
| Rust→Mycelium transpiler PoC + surface-feature backlog | `crates/mycelium-transpile` (syn AST-walk → best-effort `.myc` + never-silent gap report; batch/dir mode) | reads a Rust crate's `src/`, emits the expressible fraction + a structured `{file,line,rust_construct,reason,category}` gap report; diffed against the `.myc` twin. **Union over 6 core-lib crates: 43/346 ≈ 12.4% expressible** (`Empirical`); per-crate 0–32%. Faithful DN-41 `width_cast` emission for unsigned `Binary` widening (std-cmp 3.6%→12.6%). Re-ranked backlog: unsupported **types** #1 (36% — String/text, usize/isize, char, closures, **signed ints** = ADR-028 sign-free consequence), macros #2 (22%), trait-bounded generics #3 | `syn`/`quote`/`serde` (scoped to the transpiler crate only, KC-3 — not the kernel/self-hosting surface) | **Acceleration tooling, not a port target** — the DN-34 *mechanism* for the bulk rewrite + the first-class **prioritized surface-feature backlog** (DN-34 §8.3/§8.5) grounding E18-1's `needs-design` work. **Grounded finding:** `std.option`/`std.result` have no Rust source — already self-hosted (M-715/M-649), so the transpiler's scope is the Rust-backed remainder (DN-34 §8.6). Measured cost: PoC ~0.85–0.95M + hardening; converts the assessment's `Declared` rows to `Empirical` (§5a). Still a spike: DN-34 Draft; full phase gated on surface maturity | M-873 (DN-34 §8) |
| *(license audit gate, M-743)* | `scripts/checks/license-first-party.sh` | MIT-only first-party license enforcement | n/a | **Not applicable** — a project-tooling/CI gate script, not Mycelium-language-implementation logic; outside this ledger's scope (nothing here for the self-hosting effort to reproduce) | M-743 |

### `opp`-wave stdlib nodule ports — measured transpiler-assist % (M-935)

The Phase-I `opp` wave (E29-1) ported nine Rust-reference nodules to self-hosted `.myc`
(`lib/std/`, differential-tested Rust-ref ≡ `.myc`, M-926…M-934). For each, the **transpiler-assist
%** below is the fraction of the landed `.myc` port that the `mycelium-transpile` PoC (M-873)
could mechanically emit from the Rust reference — the rest was hand-written. All values are
**`Empirical`** (measured against the landed port, not projected). They are low by design: much of
each nodule is either genuinely new self-hosted surface or, for the D1-kernel-boundary nodules, a
thin re-export/mint layer whose Rust half stays Rust (see the note below), so there is little
Rust body for the transpiler to convert.

| Nodule (`.myc`) | Rust reference | Transpiler-assist % (`Empirical`) | Tracking |
|---|---|---|---|
| `std.diag` (`lib/std/diag.myc`) | `crates/mycelium-diag` / `mycelium-std-diag` | ~0% | M-926 |
| `std.core` (`lib/std/core.myc`) | `crates/mycelium-core` / `mycelium-std-core` | ~13.6% | M-927 |
| `std.select` (`lib/std/select.myc`) | `crates/mycelium-select` / `mycelium-std-select` | ~0% | M-928 |
| `std.swaps` (`lib/std/swap.myc`) | `crates/mycelium-std-swap` | ~9.1% | M-929 |
| `std.recover` (`lib/std/recover.myc`) | `crates/mycelium-std-recover` | ~3.4% | M-930 |
| `std.error` (`lib/std/error.myc`) | `crates/mycelium-std-error` | ~3.4% | M-931 |
| `std.testing` (`lib/std/testing.myc`) | `crates/mycelium-std-testing` | ~18% | M-932 |
| `std.ternary` (`lib/std/ternary.myc`) | `crates/mycelium-std-ternary` | ~10% | M-933 |
| `std.spores` (`lib/std/spore.myc`) | `crates/mycelium-std-spore` | ~0% | M-934 |

**D1-kernel-boundary halves stay Rust (never-silent, G2).** Five of the nine — `std.diag`,
`std.core`, `std.select`, `std.swaps`, `std.spores` — sit on the D1 kernel boundary: their
self-hosted `.myc` is a thin surface over kernel facilities that remain Rust-hosted (the diag/core/
select/swap/spore **re-export** surfaces, and the content-address/`hash`-**minting** primitives). The
`.myc` port covers the surface; the kernel half is deliberately not ported and is not counted as
"expressible" — which is why the four re-export/mint nodules (`diag`/`select`/`spores` ~0%,
`swaps` ~9.1%, `core` re-export portion) score low. This is the same KC-3 trusted-base boundary the
MLIR/Dense-VSA rows above record: a self-hosted stdlib still keeps its kernel-mint/kernel-re-export
half in the host until the kernel itself self-hosts.

### `spw`-wave (Wave-0 pilot) stdlib nodule ports — measured transpiler-assist % (M-1020…M-1022)

The `spw` Wave-0 pilot (kickoff `spw`, E33-1) ported three unported stdlib crates to self-hosted
`.myc`, each differential-witnessed (three-way L1≡L0≡AOT **plus** a live Rust-oracle comparison —
`Empirical`) and independently, adversarially re-verified (accept, not forced-green). The
transpiler-assist % is the fraction of the landed `.myc` the `mycelium-transpile` PoC (M-873) could
mechanically emit; the rest is hand-written. A **STEP-0** re-run of the CURRENT transpiler measured
**ZERO checked-% delta** vs the committed `gen/myc-drafts/` manifest on all three (the two emitter
features landed since — DN-51 narrow-cast→truncate, D3 operand inference — are orthogonal to these
targets' gap classes), so net assist to the shipped nodules is ~0% (only already-clean draft
enums/types graduated verbatim). All values `Empirical` (measured against the landed port).

| Nodule (`.myc`) | Rust reference | Transpiler-assist % (`Empirical`) | Ported subset / residual (enabler-blocked, not effort-blocked) | Tracking |
|---|---|---|---|---|
| `std.numerics` (`lib/std/numerics.myc`) | `crates/mycelium-std-numerics` | ~0% (checked 7.4%) | PORTED: the honesty-crux strength surface (Guarantee/BoundBasis lattice + meet, `Approx[A]` carrier, `NumErr`/`CheckErr`). RESIDUAL (→ `enb`): the float-valued ε/δ magnitude surface (no scalar-Float VALUE in the runtime, Gap A/M-895/M-896), the sealed FR-N3 `ProvenThm` witness (`Approx::proven` omitted, not faked), the `&mut Formatter` Display (ADR-003) | M-1020 |
| `std.time` (`lib/std/time.myc`) | `crates/mycelium-std-time` | ~0% (checked 18.9%) | PORTED: the full value-semantic surface (4 instant/duration types, the complete comparison surface via uncapped `lt_s`, deterministic `ManualClock`, declared-effect wrappers, 11-row matrix). RESIDUAL (→ `enb`): signed 128-bit duration/instant arithmetic (kernel `TC_MAX_WIDTH=64` cap), saturating advance (never-silent mismatch), TimeErr payloads/Display; OS clock is host-FFI (→ M-541) | M-1021 |
| `std.content` (`lib/std/content.myc`) | `crates/mycelium-std-content` | ~0% (checked 14.3%) | PORTED: the content-addressing surface (`digest_eq` via M-912 `bytes_eq`, `ContentRef`/`RefKind` accessors, hand-rolled recursive `parse_ref`/`content_ref_from_str`, 7-row matrix, `NameRegistry` assoc-list). RESIDUAL (→ `enb`): `hash_of_value`/`hash_of_def` (kernel structural-hash normalizer, RFC-0031 D1); a `bytes.find`/`split_once` prim would collapse the hand-rolled scanners | M-1022 |

**Retirement (ADR-043): NOT triggered for any of the three (as-proven, per-module).** Each is an honest
partial port — the Rust oracle crate is not fully replaced (the residual above stays Rust) — so
`retireReady=false`; no crate is archived/removed this wave. Retirement waits until a module's port is
FULLY validated (its residual enablers land + a whole-crate differential clears). The surfaced enabler
blockers are consolidated for the `enb` epic (E28-1) as **M-1023** (never silently dropped, G2).

### Forward items (not yet landed — tracked for when they do)

| Component | Status | What it will add to this ledger when it lands |
|---|---|---|
| Persistent bounded work-stealing pool | `needs-design`, M-864 | Replaces `mycelium-sched`'s current per-call-spawn `Scheduler::run_indexed` with a persistent, nested-submission-safe pool. When landed, update the M-860/M-862 rows above (or add a new `mycelium-sched` row) with the pool's own logic + the ratified `run_indexed` contract change, since the self-hosted port target changes shape (pool lifecycle + nested-join wait-loop, not just per-call spawn) |
| AOT-runtime concurrency + async parity with the interpreter | `needs-design`, M-865 | Extends the AOT-compiled path to run Colony/hypha/async programs on the same `mycelium-sched` substrate as the interpreter. When landed, add a row for the AOT-side concurrency surface + note whether it introduces any AOT-only runtime logic that itself becomes a distinct self-hosting port item (vs. reusing the interpreter's already-tracked one) |

<!-- Append new rows below as components land. Keep the External-dependency table in sync. -->

## How this feeds the self-hosting effort

When E18-1 / `boot10` begins porting a subsystem, its first input is this ledger's row(s): the logic
to reproduce and the native substitutes for any Rust dep. A row that is thin is a signal to write the
component's spec *before* porting (a DN or a `docs/spec/` entry), not to reverse-engineer the Rust.
