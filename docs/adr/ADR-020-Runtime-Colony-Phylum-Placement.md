# ADR-020 — `runtime`/`colony` lives in a separate `runtime` phylum, activated construct-by-construct at the Phase-7 gate

| Field | Value |
|---|---|
| **ADR** | 020 |
| **Title** | `runtime`/`colony` module placement: a dedicated `runtime` phylum with a thin `std.runtime` facade, construct-by-construct activation at the Phase-7 gate; v0 API surface for the landed R1 slice (`Scope`/`Colony`/`Network`) |
| **Status** | **Enacted** (2026-06-20; M-521 v0 R1 implementation landed on main — `crates/mycelium-std-runtime`) |
| **Date** | 2026-06-20 |
| **Depends on** | RFC-0008 (Runtime & Concurrency Execution Model — RT1–RT7, §4.3/§4.5/§4.6/§4.7); RFC-0016 §4.3 (`runtime`/`colony` Tier-A module, M-521) + §4.2 (ring layering, KC-3) + §8-Q4 (phylum placement deferred); DN-06 (static `phylum`/`nodule`/`colony` lexicon); RFC-0001 (guarantee lattice); ADR-013 (`spore` — the deployable unit); ADR-014 (`wild`/unsafe policy); ADR-003 (content-addressing); ADR-007 (Rust-first toolchain) |
| **Resolves** | RFC-0016 §8-Q4 (the deferred `runtime`/`colony` phylum-placement question) |
| **Implements** | M-521 (design-first acceptance criterion: "the binding set + the std-vs-separate-phylum decision presented") |

---

## Context

RFC-0016 §4.3 (Accepted, 2026-06-17) places `runtime`/`colony` as a **Tier-A differentiator module** in the standard library, grounded in RFC-0008's Runtime & Concurrency model. §4.3's note is explicit:

> "`runtime`/`colony` (M-521) is largely **reserved-not-active** vocabulary (Glossary ⟂) until the RFC-0008 constructs land; the stdlib bindings activate construct-by-construct at the Phase-7 gate, so this module is sequenced against that track (a FLAGGED cross-phase dependency, §8-Q4)."

§8-Q4 recorded the maintainer's direction at ratification:

> "**→ Resolution (DEFERRED — Phase-7 gate):** home the runtime surface in a **separate `runtime` phylum** (or gated sub-phylum) so pure `std` carries no inactive surface, activated **construct-by-construct at the Phase-7 gate** as the RFC-0008 constructs land."

This ADR formalises that direction as a **decision**: picks the concrete option, specifies the interface boundary, defines which RFC-0008 vocabulary is **activated** in v0 (the landed R1 slice) versus **reserved** (pending L1 syntax and Phase-7 constructs), and states the guarantee tags each activated surface carries.

### What has already landed (read-only references)

- `crates/mycelium-mlir/src/runtime.rs` — the RT2 v0 slice (M-357): structured `Scope<T,E>` (alias `Colony<T,E>`), `Task` trait, `TaskCtx`, `Poll`, `SweepOrder`, `Deadlock`; per-task `Budgets` + `CancelToken` (M-356 C1/C2); `run_sequential` / `run_interleaved` / `run_dataflow`; the RT2 sequentialization differential green.
- `crates/mycelium-mlir/src/channel.rs` — typed SPSC channels (M-357 follow-on): `Network`, `Sender<V>`, `Receiver<V>`, `TrySend<V>`, `TryRecv`; bounded demand-signalled backpressure; explicit close + `Disconnected`; the Kahn-determinism differential (`Empirical`, T4.1) green.

These are **capability-crate implementations**; this ADR decides how the stdlib exposes them as a documented, ergonomic surface with honest per-op guarantee tags.

### The three options (RFC-0016 §8-Q4 framing)

- **Option A — `std.runtime.*`.** `runtime` and `colony` live inside the `std` phylum as Tier-A nodules, alongside the other differentiator modules.
- **Option B — dedicated `runtime` phylum.** A standalone `runtime` phylum with its own content-addressed identity boundary, entirely separate from `std`.
- **Option C — hybrid.** A thin re-exporting facade in `std.runtime` backed by a dedicated `runtime` phylum; users import from `std.runtime` and the `runtime` phylum both exist.

---

## Decision

**Option C — hybrid: a dedicated `runtime` phylum with a thin `std.runtime` facade re-exporting from it.**

The `runtime` phylum is the identity and release boundary; `std.runtime` is a thin, stable facade. The rationale for each axis:

### Why not Option A (pure `std.runtime`)

RFC-0016 §8-Q4's own direction was "separate `runtime` phylum so pure `std` carries no inactive surface." That direction reflects a genuine decoupling need: the runtime vocabulary activates construct-by-construct over Phase 7, which has a different release cadence than the Phase-5/6 stdlib modules (`collections`, `numerics`, `swap`, etc.). Coupling `runtime` to the `std` release cadence would mean:

1. **`std`'s identity boundary includes reserved vocabulary.** A content-addressed `std` phylum that contains reserved-not-active surface (Glossary ⟂) carries dead weight that `std` importers did not opt into — a violation of the "no premature surface" rule (VR-5) at the phylum granularity.
2. **Embedded/bare-metal environments.** Specialised deployment targets (microcontrollers, deterministic RT kernels) may want `std` without any concurrency surface. A phylum-level separation lets them import `std` and never pull in `runtime`, which is not expressible when both live in the same content-addressed unit.
3. **RFC-0016 §4.2 ring layering.** The layering places `runtime` in Ring 2 ("general library + runtime"). Keeping `std` Ring 0/1 clean for embedded/no-runtime environments is the *point* of the ring split (mirroring Rust's `core`/`std` split, T8.1). A `runtime` phylum realises this as a first-class boundary rather than a notional one.

*Tension managed:* this trades some convenience (a single import namespace) for honest decoupling (RFC-0016 §8-Q6's `std-sys` precedent applies here too — phylum splits are the mechanism for confining optional or heavy surfaces).

### Why not Option B (pure separate phylum, no facade)

A bare `runtime` phylum with no `std.runtime` re-export would break the §4.3 Tier-A design, which names the module `runtime`/`colony` within the `std` taxonomy for discoverability and contract coherence. Stdlib users would need to know about a second phylum to access the runtime surface, fracturing the single-contract promise RFC-0016 §4.1 (C1–C6) makes. The §8-Q2 resolution already confirms that `std.runtime` is the *named* module with the themed vocabulary, mirroring the crate name convention (`mycelium-mlir::runtime` → `std.runtime`).

### Why Option C is the right shape

The hybrid resolves both objections:

- The **`runtime` phylum** owns the implementation, the content-addressed identity, the release cadence, and the direct-import path for users who want fine-grained control (or who are building runtime infrastructure on top of `runtime` primitives directly).
- The **`std.runtime` facade** is a thin nodule that re-exports the activated API surface from `runtime`, making it discoverable under the Tier-A `std.*` namespace and holding it to the §4.1 contract. The facade itself is minimal — it adds no new logic, only re-exports with the correct guarantee-matrix annotations.
- **Construct-by-construct activation** is the facade's job: as each RFC-0008 construct lands in Phase 7, the facade's re-exports expand. Reserved vocabulary never appears in the facade until it is activated. The `runtime` phylum can carry experimental/in-progress constructs behind a feature gate without polluting `std.runtime`.

This is precedented: RFC-0016 §8-Q6 splits `std-sys` from `std` for the same reason (optional OS-level surface that not all targets need); this ADR applies the same mechanism to the runtime surface (optional concurrency surface that not all environments need).

---

## v0 API Surface (the activated R1 slice)

The following vocabulary is **ACTIVATED** in v0 — it has a landed Rust implementation (M-357), verified differentials, and honest guarantee tags. It is the export set of `runtime` phylum v0 and the re-export set of `std.runtime` v0.

**Reserved vocabulary stays reserved.** The RFC-0008 §4.5 table names `hypha`, `fuse`, `xloc`, `cyst`, `graft`, `forage`, `backbone`, `mesh`, `tier`, `reclaim` as ratified meanings — but activation "requires an implementation RFC committing each construct's typing and elaboration per RFC-0006 §4.3." None of those constructs have landed implementation RFCs; they remain Glossary ⟂ until Phase 7. This ADR does **not** activate them.

### Types and traits (v0)

| Symbol | Kind | Grounding | Guarantee tag |
|---|---|---|---|
| `Colony<T, E>` | Type alias (`= Scope<T, E>`) | DN-06; RFC-0008 §4.7; `runtime.rs` line 106 | — (structural alias; inherits `Scope` tags) |
| `Scope<T, E>` | Struct | RFC-0008 §4.7 (RT7 structured concurrency); M-357 | `Empirical` — the RT2 sequentialization differential is the evidence (interleaved ≡ sequential over the real env-machine); Kahn T4.1 is the theoretical basis; no mechanized proof in-repo (VR-5) |
| `Task` | Trait | RFC-0008 §4.7 (RT1 purity contract); `runtime.rs` | `Declared` — the purity/RT1 obligation is a declared contract on implementors; checked by the differential, not proven per-impl |
| `TaskCtx<'a>` | Struct | RFC-0008 §4.7 C1/C2; M-356 | `Exact` (structural) |
| `Poll<T, E>` | Enum | RFC-0008 §4.7; `runtime.rs` | `Exact` |
| `SweepOrder` | Enum | RFC-0008 §4.3 (Kahn determinism); `runtime.rs` | `Exact` |
| `Deadlock` | Struct | RFC-0008 §4.3/G2 (explicit, never silent); `runtime.rs` | `Exact` |
| `Network` | Struct | RFC-0008 §4.3 (Kahn process network); `channel.rs` | `Empirical` — Kahn-determinism differential (two fair schedules agree); Kahn T4.1 cited as basis; no mechanized proof (VR-5) |
| `Sender<V>` | Struct | RFC-0008 §4.3 RT1 (SPSC by construction); `channel.rs` | `Exact` (structural — not-`Clone` enforces SPSC at the type level) |
| `Receiver<V>` | Struct | RFC-0008 §4.3 RT1; `channel.rs` | `Exact` (structural) |
| `TrySend<V>` | Enum | RFC-0008 §4.3/G2 (value returned on failure, never dropped); `channel.rs` | `Exact` |
| `TryRecv` | Enum | RFC-0008 §4.3/G2 (explicit close/end-of-stream); `channel.rs` | `Exact` |

### Operations (v0) — guarantee matrix (RFC-0016 §4.5 format)

| Operation | Guarantee tag | Fallibility / explicit errors | Declared effects | EXPLAIN-able? |
|---|---|---|---|---|
| `Scope::new()` | `Exact` | infallible | allocation (bounded: one scope struct) | no |
| `Scope::spawn(task, budgets)` | `Exact` | infallible | allocation (one child slot) | no |
| `Scope::cancel_token()` | `Exact` | infallible | none | no |
| `Scope::run_sequential(self)` | `Empirical` | infallible (returns `Vec<TaskOutcome>`) | task execution (bounded per-task budget); cooperative (never preemptive) | no (schedule is fixed/inspectable) |
| `Scope::run_interleaved(self, trace)` | `Empirical` | infallible | task execution; schedule is deterministic round-robin | no (schedule is fixed; `trace` makes it inspectable) |
| `Scope::run_dataflow(self, order, progress)` | `Empirical` | `Err(Deadlock)` on a stalled network (G2 — explicit, never a hang) | task execution + channel communication | no (deadlock is explicit + inspectable via `Deadlock.parked`) |
| `Network::new()` | `Exact` | infallible | allocation (one epoch cell) | no |
| `Network::epoch()` | `Exact` | infallible | none (pure read) | no |
| `Network::channel(cap)` | `Exact` | infallible | allocation (bounded by `cap`) | no |
| `Sender::try_send(v)` | `Exact` | `Err(TrySend::Full(v))` on backpressure; `Err(TrySend::Disconnected(v))` on hung-up receiver; value always returned, never dropped (G2/C1) | none | no |
| `Sender::is_connected()` | `Exact` | infallible | none | no |
| `Receiver::try_recv()` | `Exact` | `Err(TryRecv::Empty)` when no value + producer live; `Err(TryRecv::Closed)` at end-of-stream | none | no |
| `Receiver::is_connected()` | `Exact` | infallible | none | no |

**`Empirical` basis for `Scope` and `Network`:** the RT2 sequentialization differential (M-357) and the Kahn-determinism differential (M-357 follow-on) are the evidence — both verified by in-repo property tests against the real env-machine. The theoretical basis is Kahn T4.1 (RFC-0008 §4.3). The tag is `Empirical`, not `Proven`, because no mechanized proof (Isabelle/Coq/Lean) of the full scheduler is in-repo (VR-5 — never upgrade without a checked basis).

**`Declared` for `Task` implementors:** the RT1 purity contract ("a task must be pure over immutable values and share no mutable state with siblings") is checked by the differential (a non-pure task would break the RT2 equivalence), but it cannot be statically proven for arbitrary implementors by the Rust type system alone. It is a `Declared` obligation on implementors, clearly documented, and enforced by the differential.

### What is RESERVED (not part of v0, not in the facade)

The following RFC-0008 §4.5 vocabulary is ratified by DN-03 (names are frozen, meanings are grounded) but has **no implementation RFC and no landed construct**. It must not appear in `std.runtime` v0 or the `runtime` phylum v0 public surface:

| Term | Status | Reason deferred |
|---|---|---|
| `hypha` | Reserved (Glossary ⟂) | No L1 surface syntax; no elaboration RFC; Phase-7 track (RFC-0008 §4.6 R2+) |
| `fuse` | Reserved (Glossary ⟂) | Lawful merge (RT6) requires session/protocol typing; deferred (RFC-0008 §4.3 hook only) |
| `xloc` | Reserved (Glossary ⟂) | Distribution (RFC-0008 §4.6 R2); no single-node form |
| `cyst` | Reserved (Glossary ⟂) | Checkpointing requires the matured-scope dormability (RFC-0008 §4.4); not yet an implementation RFC |
| `graft` | Reserved (Glossary ⟂) | External capability contract; needs `substrate` handle integration (LR-8); no implementation RFC |
| `forage` | Reserved (Glossary ⟂) | Placement (RFC-0005 third site, RT3); no implementation RFC; Phase-7 |
| `backbone` | Reserved (Glossary ⟂) | Placement-policy artifact; depends on `forage`; Phase-7 |
| `mesh` | Reserved (Glossary ⟂) | Distribution (RFC-0008 §4.3); gossip/pub-sub; Phase-7 R2 |
| `tier` | Reserved (Glossary ⟂) | Mode-switch construct; the `ExecutionMode` link (RFC-0004/RFC-0008 §4.2); no implementation RFC |
| `reclaim` | Reserved (Glossary ⟂) | Supervision constructs in `mycelium_interp::supervise` (M-356 C4) are the scheduler-independent primitives; the `reclaim` *surface construct* (typing + elaboration per RFC-0006 §4.3) requires its own implementation RFC |

The `mycelium_interp::supervise` types (`CancelToken`, `TaskOutcome`, `RestartIntensity`, `Supervisor`) are **internal runtime primitives**, not part of the `std.runtime` public surface. They are the composition primitives the `Scope`/`Task`/`Colony` surface is built on (M-356), but they are not themselves the public API. M-521-impl must decide whether to re-export them under a `runtime::supervise` sub-nodule or keep them internal; this ADR recommends keeping them internal until the `reclaim` surface construct is activated.

---

## Phylum boundary and content-addressed identity

The `runtime` phylum boundary is defined as:
- **In:** the v0 API surface above, and each Phase-7 construct as it activates
- **Out:** `std-sys` FFI surfaces; the kernel (`mycelium-core`, `mycelium-interp`); the L1 surface constructs (`mycelium-l1`); `std` library modules other than the thin `std.runtime` facade

Content-addressed identity (ADR-003) applies at the phylum level: the `runtime` phylum's content hash is over its public surface (types + operations), not its implementation internals. This means adding a reserved construct to the public surface is a phylum identity change — the mechanism that enforces "no premature surface" at the type level.

The `std.runtime` facade is itself a nodule inside `std`; its content hash changes when the `runtime` phylum exports it re-exports from change. Because the facade adds no new logic, a `runtime` version bump that changes a re-exported type propagates cleanly to `std.runtime` at the next facade release.

**No `wild`/FFI in `runtime` v0.** The v0 surface (single-node, cooperative, no OS threads, no async I/O) needs no OS-level calls. When `mesh` and `xloc` (R2 distribution) eventually land, they will need network I/O — at that point, the `wild` FFI floor follows the §8-Q6 resolution (`std-sys` phylum pattern). A `wild`-in-`runtime` landing requires its own ADR (a decision, not a build detail). The `runtime` phylum is therefore `wild`-free for v0, inheriting the same LR-9 leak-free guarantee as the `std` core modules.

---

## Consequences

### For M-521-impl (what the implementation must deliver)

1. **Create the `runtime` phylum crate** — `crates/mycelium-std-runtime` (following the stdlib crate naming convention), exporting the v0 API surface above. It wraps `mycelium-mlir::runtime` and `mycelium-mlir::channel` with:
   - Re-exported types with public documentation and guarantee annotations in doc comments
   - A **guarantee matrix** (RFC-0016 §4.5) as a doc-tested table in the crate's `lib.rs` and/or a `docs/spec/stdlib/runtime.md` spec
   - The `Colony<T,E>` type alias as the primary ergonomic entry point (DN-06; RFC-0008 §4.7)

2. **Create the `std.runtime` facade nodule** — a thin nodule inside the `std` phylum that re-exports from `mycelium-std-runtime`. It must meet the §4.1 contract (C1–C6) — the guarantee-matrix obligation is discharged by the `runtime` phylum's matrix, which the facade inherits.

3. **Mark all reserved vocabulary as reserved in docs.** Any mention of `hypha`, `fuse`, `cyst`, etc. in the module docs must be clearly marked `RESERVED (Glossary ⟂) — not yet active syntax` with a pointer to RFC-0008 §4.5 and the Phase-7 activation criterion.

4. **Write the per-op guarantee matrix.** The matrix (§v0 API Surface above) is the starting point. M-521-impl delivers it as tested data (not prose only), following the RFC-0003 §4 matrix template.

5. **No `reclaim` surface yet.** The `Supervisor`/`RestartIntensity` composition primitives (M-356 C4) remain in `mycelium_interp::supervise`. The `runtime` phylum may document their existence and provide a re-export under a `runtime::supervise` sub-nodule (not `std.runtime`) if the maintainer decides so — but the `reclaim` *surface construct* (L1 typing + elaboration) is gated on its own implementation RFC.

6. **No `async`/`Future` dependency.** The v0 surface is cooperative (explicit `poll`), not Rust-async. If a future Phase-7 construct needs Rust's `async`/`Future` integration, that is a separate ADR (a decision, not a build detail — it would change the surface shape and potentially the guarantee tags).

### For Phase-7 construct activation (construct-by-construct)

Each activation follows this pattern:
1. An implementation RFC commits the construct's typing + elaboration (per RFC-0006 §4.3)
2. A Rust implementation lands in the `runtime` phylum with honest guarantee tags
3. The `std.runtime` facade re-export expands
4. The guarantee matrix gains a new row (the honesty record)

The ordering within Phase-7 is governed by RFC-0008 §4.6's staging (R1 → R2) and by dependency: `reclaim` surface before `hypha` surface (supervision before raw spawning makes the error model cleaner); `xloc` only after single-node surface is complete; `mesh` only after `xloc`.

### For embedded/no-concurrency environments

Users on embedded targets (no OS scheduler, bare-metal) import `std` without the `runtime` phylum. The `std.runtime` facade becomes a zero-cost absent dependency in their manifest — the phylum separation is the mechanism that makes this possible without `#[cfg]` sprinkled through `std`.

### Costs accepted

- **Two points of import.** A user who imports from `runtime` directly and `std.runtime` in the same program will see the same types under two paths. Content-addressed identity means they *are* the same types (ADR-003); the ergonomic cost is documentation friction. The tradeoff is accepted: the alternative (Option A) would contaminate `std` with reserved vocabulary.
- **One more phylum to maintain.** The `runtime` phylum has its own release cadence, its own content hash, and its own versioning (ADR-018). This is the cost of the explicit decoupling — it is also the mechanism that lets `runtime` evolve at the Phase-7 cadence without dragging `std`.

---

## Alternatives considered

### Option A (pure `std.runtime`) — rejected

Contradicts RFC-0016 §8-Q4's own direction. Contaminating `std`'s identity boundary with reserved vocabulary (Glossary ⟂) violates VR-5 at the phylum level. Makes embedded/no-concurrency environments impossible to serve without `#[cfg]` hacks.

### Option B (pure separate `runtime` phylum, no facade) — rejected

Breaks `std`'s Tier-A discoverability promise. `std.runtime` is the documented name for the runtime module (RFC-0016 §4.3, §8-Q2 resolution); dropping the facade makes the stdlib incomplete as specified. Users expecting `std.runtime` would need to know the crate-level organization of the repo — a leaky abstraction.

### Lazy activation (no phylum split; activate everything at once at Phase-7) — rejected

This is not a valid option for v0. The R1 slice has already landed (M-357); it is in `mycelium-mlir` and the composition primitives are in `mycelium-interp`. M-521's acceptance criterion ("the binding set + the std-vs-separate-phylum decision presented") requires exposing the landed surface now, under the correct phylum structure, so Phase-7 constructs can activate incrementally into a pre-established boundary. Waiting for all of Phase 7 before creating any `runtime` surface would leave landed, tested code unexposed — the opposite of the design-first then implementation-activates-construct-by-construct discipline.

---

## Grounding

- **RFC-0008 §4.3, §4.5, §4.6, §4.7** — the Runtime model, vocabulary table, staging (R1/R2), composition contract (C1–C4); the source of the guarantee tags on all activated v0 surface
- **RFC-0008 RT2/RT6** — RT2 grounds the `Empirical` tag on `Scope` (sequentialization differential); RT6 grounds a future `fuse` `Proven` claim (Isabelle/HOL semilattice proof — side-conditions checked); neither lands in v0
- **RFC-0016 §4.1 (C1–C6), §4.2, §4.3, §8-Q4** — the per-op contract every stdlib op must meet; the ring layering; the Tier-A placement; the deferred-with-direction phylum decision this ADR resolves
- **RFC-0016 §8-Q6 / `std-sys` precedent** — phylum splits for optional/heavy surfaces; this ADR applies the same pattern to the concurrency surface
- **DN-06 §4/§5** — `phylum`/`nodule`/`colony` vocabulary; `Colony` as the dynamic grouping alias; `runtime` phylum as the library-scale unit
- **DN-03 §3/§4** — one name per term; flat; reserved vocabulary stays reserved until activation
- **ADR-003** — content-addressed identity at the phylum level enforces "no premature surface" as a type-level invariant
- **ADR-013** — `spore` is the deployable form of a `phylum`; the `runtime` phylum ships as a `spore`
- **ADR-014** — `wild`/FFI policy; `runtime` v0 is `wild`-free (no OS calls); any FFI landing in `runtime` requires a separate ADR
- **G2 (never-silent)** — `TrySend::Full`/`Disconnected`, `Deadlock`, `TryRecv::Closed` are the concrete G2 applications in the v0 surface; every non-delivering operation returns the value or an explicit error
- **VR-5 (honest guarantee strength)** — `Empirical` (not `Proven`) for `Scope`/`Network`; `Declared` (not `Empirical`) for the `Task` purity contract; no upgrade without a checked basis
- **KC-3 (small auditable kernel)** — the `runtime` phylum lives above the kernel (RFC-0016 §4.2 C5); it consumes `mycelium-mlir` and `mycelium-interp` but adds no kernel code; the cooperative single-threaded scheduler uses `RefCell` with no `unsafe` (borrows never overlap a yield point)
- **Tension A (cross-cutting: explicit, inspectable swap/conversion)** — not directly in scope for the runtime scheduling surface, but the never-silent principle (G2) and the guarantee lattice apply to runtime operations exactly as they do to swaps; the API surface above is designed to the same standard
- **Tension C (human vs AI legibility)** — the `Colony<T,E>` alias for `Scope<T,E>` is the themed vocabulary (DN-06); the generic `Scope` name remains available for human readers familiar with structured-concurrency terminology; both point to the same content-addressed type (ADR-003), so there is no identity split

---

## Meta — changelog

- **2026-06-20 — Proposed.** Formalises RFC-0016 §8-Q4's "separate `runtime` phylum" direction as a concrete decision (Option C — hybrid: a dedicated `runtime` phylum with a thin `std.runtime` re-export facade). Specifies the v0 API surface for the landed R1 slice (`Scope`/`Colony`, `Task`, `TaskCtx`, `Poll`, `SweepOrder`, `Deadlock`, `Network`, `Sender`, `Receiver`, `TrySend`, `TryRecv`) with the per-op guarantee matrix (`Exact`/`Empirical`/`Declared` tags, fallibility, effects, EXPLAIN-ability). Reserves all RFC-0008 §4.5 vocabulary not yet landed (`hypha`/`fuse`/`xloc`/`cyst`/`graft`/`forage`/`backbone`/`mesh`/`tier`/`reclaim`) as Glossary ⟂ until Phase-7 activation. States the `runtime` phylum v0 constraint: `wild`-free (no OS calls; any FFI addition is a separate ADR). Records what M-521-impl must deliver: `crates/mycelium-std-runtime` phylum crate, `std.runtime` facade nodule, guarantee matrix as tested data, no `reclaim`/`hypha`/etc. surface until activation. Grounded in RFC-0008 §4.3/§4.5–§4.7, RFC-0016 §4.1–§4.3/§8-Q4/§8-Q6, DN-06, ADR-003/013/014/018, G2, VR-5, KC-3. Awaiting maintainer ratification (Proposed → Accepted). Append-only.
