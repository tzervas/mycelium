# Design Note DN-94 — Atomics & Memory-Model Direction (closes the CU-8 / gate-D2 design question)

| Field | Value |
|---|---|
| **Note** | DN-94 |
| **Status** | **Draft** (2026-07-08) — a decision-work-up for the maintainer to ratify. **Enacts nothing** and **moves no other doc's status**: it recommends a scope + path for the deferred CU-8 atomics prim (DN-34 §8.16) and the memory-model question behind it, presents the alternative with tradeoffs, and routes the normative commit to the RFC-0027 follow-on. All tags `Declared` unless a cited source holds them higher (VR-5). |
| **Feeds** | **CU-8** (DN-34 §8.16 — atomics prim deferral); the **RFC-0027 follow-on** (the cross-hypha boundary + reclamation-atomicity normative commit); **RFC-0008** RT1 (no shared mutable state); **DN-61** Part B (the R2 distributed-execution agenda, where Option B / atomic-RC lives); the E12 memory-model build plan. |
| **Date** | July 8, 2026 |
| **Decides** | *Proposes, for ratification (nothing enacted):* **the scope and path for atomics in Mycelium.** The recommendation: atomics are a **runtime-internal concern**, not a user-facing surface — because Mycelium's value semantics (LR-8/LR-9), the RT1 "no shared mutable state" invariant, and the **already-ratified Option A** (sole-ownership cross-hypha move; `RcCell<T>: !Send`; no cross-hypha atomic RC — DN-33 §8.1 Q1 / DN-59 §3a) mean the shared mutable memory a user atomics surface would synchronize **does not exist by construction**. Therefore: **document the RC/transfer atomicity model as the answer and defer a user-facing atomics surface (and the full memory-ordering model it would require) past 1.0**, with a never-silent boundary. The CU-8 prim gap is resolved narrowly (a capability-gated runtime-internal prim, or keep the runtime allocator in Rust), **not** by a full C++-style memory-ordering RFC. The alternative (user surface + memory-ordering RFC amending RFC-0027) is tabled in §8 with its tradeoffs. |
| **Task** | trx2 design wave — CU-8 / gate-D2 decision work-up |

> **Posture (transparency rule / VR-5 / G2).** This note **works up a decision**; it does not take
> it — the binding move is the maintainer's ratification and the RFC-0027 follow-on's normative
> commit (house rule #3, append-only). In-repo state assertions are `Empirical` (verified against
> the working tree, file:line cited); the corpus decisions this builds on (RT1, LR-8/LR-9, Option A)
> are cited at the strength their source holds them — **not here-upgraded**. The prior-art survey
> (§6) is `Declared` — external systems' behaviour, cited to their authoritative docs, evidencing
> that the *approach* is proven for *those* systems, not for Mycelium. The recommendation "a user
> atomics surface is unwarranted for 1.0" is **`Declared`-with-argument**: it follows from ratified
> invariants, but "no user program needs it" is an unfalsified claim until real R1 programs stress
> it (§9 R-2). The disconfirming case is surfaced, not buried (§8, §9): if Option B (shared-crosses,
> atomic-RC) is ever adopted at R2, a memory-ordering obligation appears — still runtime-internal,
> but real. Nothing is claimed `Proven` without a checked basis.

---

## §1 Purpose — why this decision was deferred, and what it needs

CU-8 (the atomics prim wave: `fetch_add`, compare-and-swap, …) was **deliberately deferred** in the
trx2 prim-gap closure (`DN-34 §8.16`, item CU-8): *"atomics without a memory model would be unsound"*
— a user-facing atomics surface needs a full memory-ordering model (acquire / release / seq-cst /
relaxed), which is an **RFC-scale decision, not a prim add** (`docs/notes/DN-34-Prim-Gap-Closure-Wave.md`,
§8.16 CU-8 row; the `fetch_add` co-blocker on `std-runtime/region` is recorded there). This DN works up
that decision so the maintainer can settle the two questions it forked into:

1. **Scope.** Does Mycelium expose a **user-facing atomics surface** (which forces a full
   memory-ordering model), or are atomics a **runtime-internal concern** given the value-semantics +
   hypha/colony execution model?
2. **Path.** If a user surface is warranted — the minimal ordering model, and whether it elevates to
   a full RFC (likely amending RFC-0027) vs. a smaller surface. If runtime-internal — document the
   answer and defer the user surface, with a never-silent boundary.

The recommendation below is that the corpus **already answers question 1** (runtime-internal), so
question 2 collapses to a documentation-and-defer path, not a memory-ordering RFC. §§2–7 make that
case; §8 tables the alternative honestly; §§9–12 give the risks, user stories, DoD, and FLAGs.

---

## §2 The core question, restated precisely

An **atomic operation** (`fetch_add`, CAS, atomic load/store with an ordering) exists to give
**race-free, ordered access to memory that is *mutated* and *shared* by more than one concurrent
thread of control.** Its two preconditions are therefore:

- **shared** — more than one execution unit references the same mutable cell, and
- **mutable** — that cell is written after it is shared.

A **memory-ordering model** (the C++11/C++20 `memory_order` lattice, the JMM happens-before relation,
Rust's `Ordering`) is the specification of *which* inter-thread visibility/reordering guarantees each
atomic operation carries. It is, by broad consensus, **the single most intricate part** of the
languages that have one (§6). **You only need it if you have shared mutable memory to order.**

So the scope question is not really "should we add `fetch_add`?" — it is: **does Mycelium have shared
mutable state that a user must synchronize?** If it does not, a user atomics surface has nothing to
operate on, and the memory-ordering model has nothing to specify. §3 shows the corpus has already
decided it does not.

---

## §3 Mycelium has no user-facing shared mutable state — by construction, already ratified

Three ratified layers of the corpus converge on the same point: **the shared mutable memory a user
atomics surface would synchronize does not exist in the language a user writes.**

- **RT1 — "Values move; state is never shared" (RFC-0008 §4.1, Accepted 2026-06-16).** The runtime's
  first invariant: *"The only thing that crosses a hypha, channel, [boundary] is an owned value…
  There is no shared mutable state, no locks, no data races — not as a discipline [but by
  construction]"* (`docs/rfcs/RFC-0008-Runtime-and-Concurrency-Execution-Model.md` §4.1, RT1).
- **LR-8 / LR-9 (RFC-0006).** Values are **immutable** (LR-8 — no aliased mutation) and **acyclic**
  (LR-9). Mutation is value-semantic (a new value, never an in-place aliased write). There is no
  aliased mutable cell for two hyphae to race on.
- **Option A — the cross-hypha sharing boundary, already ratified (DN-33 §8.1 Q1, Accepted 2026-06-25;
  confirmed DN-59 §3a, Accepted 2026-06-28).** When a value crosses a hypha boundary it is **sole
  ownership being *moved*** via the affine `Sender`/`Receiver` channel (non-`Clone` — RFC-0027 §7.3),
  **not** a shared value. The reference-counting cell **`RcCell<T>` stays `!Send`**, so a genuinely
  shared (`rc > 1`) value crossing a hypha boundary is a **compile-time `!Send` type error**, never a
  silent runtime promotion to atomic RC (`docs/notes/DN-59-Reclamation-Strategy-And-Cross-Hypha-Sharing.md`
  §3a, "Never-silent (G2)"). Option B (a shared value crosses, atomic RC engages) is **deferred to R2**
  (the `xloc`/`mesh` distributed-execution group, G8 — DN-59 §8; DN-61 Part B).

The consequence is decisive for CU-8: **within a hypha, execution is single-threaded** (RC is
non-atomic intra-hypha — DN-32 §2.2), and **across hyphae, only sole ownership moves** (Option A). At
no point in the R1 user model do two concurrent threads of control hold the same mutable cell. **A
user-facing atomic operation would have no shared mutable target to act on.** This is not a gap to be
filled later — it is a *designed absence*, the same one that gives Mycelium "no locks, no data races,
by construction" (RT1). The value-semantics + message-passing model **is** the concurrency-safety
mechanism; user atomics are the tool of the shared-memory model Mycelium deliberately does not adopt.

---

## §4 The atomics that *do* exist are runtime-internal — and this is where CU-8 actually came from

Atomics are not absent from Mycelium's implementation. They exist today, **entirely runtime-internal**,
in the Rust kernel — and they are sound, minimal, and already `Exact`-tagged:

- **Monotonic ID/epoch counters** in `mycelium-std-runtime`: `ScopeNodeId` and `RegionEpoch`
  (`crates/mycelium-std-runtime/src/region.rs:91-92, :112, :154` — `AtomicU64` + `fetch_add(1,
  Ordering::Relaxed)`), and `ChannelNodeId` (`network.rs:52, :72`). Each is a strictly-increasing
  process-unique counter, tagged **`Exact`** ("strictly monotonic and unique within process" —
  `region.rs:45-46`, :87). They use **`Relaxed` ordering only** — no acquire/release, because a
  monotonic counter needs *uniqueness*, not *inter-thread visibility of other memory*.

**This is the true origin of CU-8.** The prim gap surfaced not from any user program, but from the
**self-hosted `.myc` runtime frontend**: `gen/myc-drafts/stdlib/std-runtime/region.myc`'s `allocate`
bodies call `fetch_add` on `ScopeNodeId`/`RegionEpoch`, and the `.myc` toolchain has no `fetch_add`
prim — so the file stays "poisoned regardless of the rename" (`DN-34 §8.16`, the `fetch_add`
co-blocker note). **The unmet need is a *runtime-implementer's* need** (expressing the runtime's own
allocator counters in `.myc`), **not a user's need** for shared-memory atomics. That distinction is
the whole decision: the thing CU-8 blocks is runtime-internal machinery, and it can be unblocked
runtime-internally without ever exposing a user atomics surface or a memory-ordering model (§7).

A second, *future* runtime-internal atomic appears under **Option B** (if ever adopted at R2): DN-32
§2.2's "RC becomes **atomic** after cross-hypha transfer" would make the RC cell's increment/decrement
atomic. That, too, is **runtime machinery** (the RC protocol), not a user surface — and it is exactly
what Option A defers to R2 (§3). So both the atomics that exist and the atomics that might exist are
runtime-internal; neither is a user-facing surface.

---

## §5 Why "runtime-internal" is the value-semantics-consistent answer

The three-layer memory architecture (DN-32) already places the expensive concurrency machinery
**inside the runtime, engaged by construction of the program, never as a user-visible tax**: affine
move is the default (Layer 1, no RC), RC engages only on explicit sharing (Layer 2), and atomicity
engages only after a cross-hypha transfer under Option B (deferred). Atomics belong to the same tier
as RC and regions: **implementation mechanism, reified in the EXPLAIN/`Provenance` record when it
matters (G2), never a primitive the user reaches for.** This is the memory-tier analogue of
RFC-0034/ADR-032's "pay for what you use, transparently" posture (DN-32 §2): the user writes ordinary
value-semantic code; the runtime picks the cheapest valid mechanism and records the choice.

Exposing user atomics would **contradict** this: it reintroduces shared mutable state (against RT1 /
LR-8), grows the kernel with a memory-ordering model (against KC-3), and hands the user a footgun the
rest of the design spent its invariants eliminating. The value-semantics identity of the language is
precisely *"you do not reason about memory ordering; the model does."*

---

## §6 Prior art — how languages split on exactly this axis (`Declared`)

The prior art divides cleanly along the shared-mutable-state line, and Mycelium sits firmly on the
message-passing side with Erlang and Pony. Every claim here is `Declared` (external systems, cited to
their authoritative documentation).

| System | Shared mutable state? | User atomics + ordering model? | Relevance to Mycelium |
|---|---|---|---|
| **C++11/C++20** | Yes | **Yes** — `std::memory_order` {`relaxed`, `consume`, `acquire`, `release`, `acq_rel`, `seq_cst`}; `consume` is discouraged/effectively deprecated as unimplementable-as-specified | The canonical full model; widely regarded as the hardest corner of the language. The cost of the surface Mycelium would inherit. |
| **Java (JMM, JSR-133, 2004)** | Yes | **Yes** — happens-before, `volatile`, `java.util.concurrent.atomic`, `VarHandle` | A whole formal memory model (JMM) was required to make shared-memory concurrency specifiable. |
| **Rust** | Yes (via `Arc`/`UnsafeCell`) | **Yes** — `std::sync::atomic::Ordering` {`Relaxed`, `Acquire`, `Release`, `AcqRel`, `SeqCst`}, adopting the C++20 model; deliberately omits `consume`. Atomics require explicit shared memory (`Arc<AtomicUsize>`) | Closest kin to Mycelium's kernel language, yet its atomics exist *because* it has `Send + Sync` shared memory — which Mycelium's RT1/Option A remove. |
| **WebAssembly threads** | Yes (shared linear memory) | **Yes** — atomic memory instructions + `wait`/`notify`, a seq-cst/relaxed subset of the C++ model | Even a minimal VM needed a (reduced) ordering model the moment it added *shared memory*. |
| **Erlang / BEAM** | **No** (share-nothing processes) | **No user surface at all** — concurrency is async message passing between isolated processes; the VM manages memory internally | The value-semantics/message-passing precedent: **no user atomics, no memory-ordering model**, safety by isolation. Mycelium's RT1 is the same stance. |
| **Pony** | **No** (reference capabilities enforce it) | **No user surface** — data-race freedom is a *compile-time* guarantee of the capability type system (`iso`/`val`/`ref`/`box`/`tag`/`trn` + `consume`); the **ORCA** protocol does fully-concurrent GC **runtime-internally** | The nearest structural analogue: capabilities ⇒ no shared mutable state ⇒ **no user atomics**; atomicity lives in the runtime (ORCA). Option A is explicitly Pony-`consume`-grounded (DN-59 §3a). |

**The pattern is unambiguous:** every language with a user atomics surface has it **because it has
shared mutable memory** (C++, Java, Rust, Wasm). Every language *without* shared mutable state
(Erlang, Pony) exposes **no user atomics surface and no memory-ordering model** — it does not need
one, and its runtime handles real hardware atomicity internally. Mycelium's ratified invariants
(RT1 + LR-8/LR-9 + Option A) place it with Erlang and Pony. This is the central evidence for the
recommendation, and it is the case the maintainer directive named as decisive. *(Exact source URLs
for each row are collected in the changelog footer's reference note; all `Declared`.)*

---

## §7 Recommendation — runtime-internal + defer the user surface (the path)

**Primary recommendation (for ratification): treat atomics as a runtime-internal concern; do NOT
open a memory-ordering RFC or ship a user-facing atomics surface for 1.0.** Concretely:

1. **Document the answer, don't build a surface.** Fold a one-section statement into the **RFC-0027
   follow-on** (the vehicle already slated to move Option A into RFC-0027's body — DN-59 §3a "route
   the normative commit… to the RFC-0027 follow-on"): *Mycelium exposes no user-facing atomics
   surface; inter-hypha safety is provided by value semantics + message passing (RT1) and the affine
   Option-A move; runtime-internal atomicity (RC counters; the deferred Option-B atomic RC) is
   governed by the RC/transfer model, not a user primitive.* This DN is the rationale record behind
   that statement.
2. **Resolve the CU-8 prim gap narrowly, runtime-internally** (two viable sub-options, maintainer's
   choice — both keep atomics out of the user surface):
   - **(7a) A capability-gated runtime-internal `atomic` prim.** Expose `fetch_add` (and, only if a
     concrete runtime need arises, CAS) as a **runtime/stdlib-only** prim, usable from
     `std-runtime`-tier `.myc` code but **not** general user code — gated the way ADR-014's warned
     `unsafe` escape is gated, and **restricted to `Relaxed`** (the only ordering the existing
     counters use — `region.rs:46`). This unblocks the self-hosted runtime allocator port with the
     *minimum* surface and **no user-facing memory-ordering model**. A fuller ordering vocabulary is
     added only if/when a runtime component provably needs acquire/release — never speculatively
     (YAGNI/KC-3).
   - **(7b) Keep the runtime allocator counters in Rust.** Do not port `region.myc`/`network.myc`'s
     atomic counters to `.myc` at all; leave them Rust-kernel-resident (as they are today, `Exact`),
     and mark those specific `.myc` frontend bodies as intentionally Rust-backed. Zero new prim; the
     `.myc` self-hosting frontier simply stops at the runtime allocator boundary, never-silently
     recorded.
   **Recommended sub-option: (7a) restricted to `Relaxed`, capability-gated** — it advances `.myc`
   self-hosting (the boot10/M-989 direction) without a user surface and without an ordering model,
   and it is honestly minimal. (7b) is the strictly-more-conservative fallback if even a gated prim
   is judged premature.
3. **Park the full memory-ordering model with Option B, in the R2 group.** A memory-ordering model
   (acquire/release/seq-cst/relaxed) becomes *relevant* only if **Option B** (shared values cross a
   hypha boundary, atomic RC engages) is ever adopted — and Option B is already deferred to the **R2
   distributed-execution RFC (G8)** (DN-59 §8; DN-61 Part B). Even there it would be a **runtime**
   ordering obligation (the atomic-RC protocol's own acquire/release), scoped inside that RFC — still
   not automatically a user surface. **No standalone atomics/memory-model RFC is warranted now.**

**Net:** CU-8 is closed as *"runtime-internal; user surface deferred past 1.0 with rationale"*; the
prim gap is unblocked by (7a)/(7b); the memory-ordering model is neither built nor RFC'd until Option
B forces the question at R2. This is the low-regret path: it takes no decision that a later shared-
memory need could not still take, and it keeps the kernel and the user surface minimal.

---

## §8 The alternative — a user-facing atomics surface + memory-ordering RFC (tabled, with tradeoffs)

Presented honestly so the maintainer weighs a real option, not a strawman.

**Shape.** Adopt Option B's premise *at the user level*: permit shared mutable atomic cells
(`Atomic<T>`) in user code, and ratify a memory-ordering model — minimally a **two-point** model
(`Relaxed` + `SeqCst`, the Wasm-threads subset) rather than the full six-point C++ lattice — via an
RFC that **amends RFC-0027** (the memory-management decision) and touches RFC-0008 (RT1 would need a
carve-out).

**What it would buy.** (i) Lock-free user data structures and certain high-contention patterns
expressed directly rather than via message passing; (ii) a more familiar surface for programmers
arriving from Rust/C++; (iii) the `.myc` runtime port needing no capability gate.

**Why it is not recommended (the tradeoffs).**
- **It contradicts a ratified invariant.** RT1 ("state is never shared") and LR-8 (immutable values)
  would need a carve-out — reintroducing exactly the shared mutable state the design eliminated. That
  is a supersede-scale change to Accepted decisions (house rule #3), not an additive prim.
- **It imports the hardest surface in language design.** §6: the memory-ordering model is the most
  error-prone corner of C++/Java/Rust. Even the "minimal" two-point model is a formal-semantics
  obligation and a permanent kernel/spec cost (against KC-3).
- **The prior art disfavors it for this language class.** The two value-semantics/no-shared-state
  languages (Erlang, Pony) deliberately *do not* expose it and are not considered impoverished for
  it; they achieve concurrency safety by construction. Adopting user atomics would move Mycelium off
  the message-passing side of the split it was designed onto.
- **No demonstrated need.** There is no R1 use case on record that value semantics + message passing
  cannot serve; adopting a user surface to serve a hypothetical is against YAGNI. (This is the honest
  weak point of the *primary* recommendation too — see §9 R-2 — but the burden of proof sits with the
  feature, not its absence.)

**When to revisit.** If real R1 programs demonstrate a class of problems that message passing +
value semantics genuinely cannot express efficiently, this alternative should be re-opened — as an
RFC amending RFC-0027, coupled to the R2/Option-B work, **not** as a prim add. §9 R-2 is the trigger
condition.

---

## §9 Risks

- **R-1 (Self-hosting frontier stalls if CU-8 stays fully deferred).** The `.myc` runtime allocator
  port is blocked on `fetch_add` today. *Mitigation:* §7.2 (7a)/(7b) unblock it runtime-internally
  without a user surface — the recommendation does not leave CU-8 blocked, it resolves it narrowly.
- **R-2 (The "no user need" claim is `Declared`, not proven).** "No R1 program needs shared-memory
  atomics" is an argument from ratified invariants, **not** a measured fact — it is unfalsified until
  real programs stress it (the same honest-risk shape DN-59 §3a flags for Option A's ergonomics).
  *Mitigation:* the boundary is never-silent (§10) — a program that *reaches* for a user atomic gets
  a loud refusal naming the message-passing alternative, so the pressure is visible, not hidden. If
  the refusal fires often on genuine need, §8 is the re-open path.
- **R-3 (A capability-gated prim (7a) could leak into user code).** A runtime-only prim that is not
  actually gated becomes a de-facto user surface. *Mitigation:* enforce the gate the way the
  guarantee-matrix test enforces reserved-vocabulary exclusion (ADR-020 §5 precedent, cited in
  DN-78 §3 B-3); a prim escaping its gate is a test failure, not a silent surface.
- **R-4 (Option B at R2 reopens the ordering question).** If R2 adopts Option B, a runtime
  memory-ordering obligation appears. *Mitigation:* this is expected and scoped — it lives in the R2
  distributed-execution RFC (G8), tagged and deferred already (DN-61 Part B); this DN does not
  preempt it, only records that it stays runtime-internal there too.

---

## §10 Never-silent boundary (G2)

The absence of a user atomics surface must be **loud, not a silent gap**:

- A `.myc` user program that references an atomic/`fetch_add`/CAS user primitive gets an **explicit
  typed refusal** at parse/check time naming (i) that Mycelium has no user atomics surface by design,
  and (ii) the message-passing / value-semantics alternative — the same teaching-diagnostic posture
  as the reserved-vocabulary refusals (`mycelium-l1/src/parse.rs`; DN-78 §3 B-3). Never a silent
  "unknown identifier".
- The runtime-internal atomics that **do** exist keep their `Exact` tags and their `Relaxed`-only
  discipline documented at their definition (`region.rs`/`network.rs` — already so).
- Under (7a), the capability-gated prim's every use is recorded/inspectable (it is runtime-tier,
  EXPLAIN-adjacent), and its gate is regression-tested (R-3).

---

## §11 User stories

- **As a Mycelium application developer,** I want to write concurrent code over immutable values with
  hyphae + channels, so that I get data-race freedom by construction and **never** have to reason
  about a memory-ordering model — and if I mistakenly reach for a shared-memory atomic, I get a clear
  refusal pointing me at message passing, not a silent failure.
- **As a runtime/stdlib implementer,** I want to express the runtime's own monotonic counters (and,
  later, the Option-B atomic RC) in `.myc` without a full user memory model, so that the self-hosted
  frontend can advance (boot10/M-989) with the *minimum* atomic surface, capability-gated and
  `Relaxed`-restricted.
- **As the maintainer,** I want the atomics/memory-model question settled with a grounded scope + path
  and the alternative fairly tabled, so that CU-8 stops being an open blocker and no memory-ordering
  RFC is opened before a real need justifies its permanent kernel cost.

---

## Definition of Done

This DN is **Draft**. It is **Accepted** when the maintainer ratifies the §7 direction (or selects
the §8 alternative, or a variant). It is **Resolved** when, after acceptance:

1. the chosen §7 sub-option (7a capability-gated `Relaxed` prim, or 7b keep-in-Rust) is recorded on
   **CU-8** in `issues.yaml` and the CU-8 blocker is re-scoped/closed accordingly (integrator-owned —
   FLAG-1); **and**
2. the **RFC-0027 follow-on** carries the one-section normative statement (§7.1) that Mycelium has no
   user atomics surface and runtime atomicity is governed by the RC/transfer model; **and**
3. this note is linked from DN-34 §8.16's CU-8 row as the decision record (integrator-owned — FLAG-2).

A maintainer amendment to the direction supersedes by dated section — never a rewrite (append-only).

---

## FLAGs (up to the integrating parent — not edited from this leaf)

| FLAG | What it is | Who |
|---|---|---|
| **FLAG-1** | **`issues.yaml` — CU-8 re-scope.** Record the ratified §7 direction on CU-8 (runtime-internal; user surface deferred; sub-option chosen) and re-scope/close the blocker. Orchestrator/integration-tier owned — not touched from this branch. | Integrator |
| **FLAG-2** | **`docs/Doc-Index.md` + DN-34 §8.16 cross-link.** Register the DN-94 Doc-Index row and link DN-94 from DN-34 §8.16's CU-8 row as the decision record. Integration-tier owned. | Integrator |
| **FLAG-3** | **`CHANGELOG.md`** entry for DN-94 (Added — Draft note). Integration-tier owned — not touched from this leaf. | Integrator |
| **FLAG-4** | **DN slot verification.** DN-94 assigned by the wave; the integrator re-verifies the slot is free at merge (mitigation #1) before registering the Doc-Index row. | Integrator |
| **FLAG-5** | **RFC-0027 follow-on hand-off.** The normative "no user atomics surface" statement (§7.1) belongs in the RFC-0027 follow-on, not this DN; routed there per DN-59 §3a's "route the normative commit to the follow-on". | Integrator / follow-on author |

---

## Meta — changelog

- **2026-07-08 — Created (Draft; trx2 design wave, CU-8 / gate-D2 work-up).** Works up the atomics &
  memory-model decision deferred by DN-34 §8.16 CU-8. **Recommends: atomics are runtime-internal, not
  a user surface** — grounded on RT1 (RFC-0008 §4.1, no shared mutable state), LR-8/LR-9 (immutable,
  acyclic values), and the **already-ratified Option A** (DN-33 §8.1 Q1 / DN-59 §3a: sole-ownership
  cross-hypha move, `RcCell<T>: !Send`, no cross-hypha atomic RC; Option B deferred to R2/G8). Shows
  (§4) the atomics that exist are runtime-internal `Exact`/`Relaxed` counters
  (`region.rs`/`network.rs`) and that CU-8 arose from the self-hosted `.myc` runtime frontend
  (`gen/myc-drafts/…/region.myc`'s `fetch_add`), a runtime-implementer need, not a user need.
  **Path (§7):** document the answer in the RFC-0027 follow-on; resolve the CU-8 prim gap narrowly via
  a capability-gated `Relaxed`-only runtime prim (7a, recommended) or keep-in-Rust (7b); park the full
  memory-ordering model with Option B in the R2 group — **no standalone memory-model RFC now.** The
  alternative (user surface + memory-ordering RFC amending RFC-0027) is tabled with tradeoffs (§8).
  Prior-art survey (§6, `Declared`): C++11/C++20, Java (JMM), Rust, Wasm-threads **have** user atomics
  because they have shared mutable memory; Erlang/BEAM and Pony (value-semantics/message-passing) have
  **no** user atomics surface and handle atomicity runtime-internally — Mycelium sits with the latter.
  User stories + DoD + risks (R-2: the "no user need" claim is `Declared`, never-silent boundary is
  the mitigation). Enacts nothing; moves no other doc's status; all recommendation tags
  `Declared`-with-argument, prior art `Declared`, in-repo state `Empirical`. **⚠️ Ratification by the
  maintainer required to move Draft → Accepted (house rule #3, append-only).** CHANGELOG.md /
  Doc-Index.md / issues.yaml / api-index are integration-tier owned (FLAGs 1–5). (Append-only; VR-5;
  G2.)
  - **Prior-art reference note (§6, all `Declared`; URLs spot-checked during authoring — re-verify on
    desktop before the note lands):** C++11/C++20 `std::memory_order` (six values; `consume`
    discouraged as unimplementable-as-specified) — `en.cppreference.com/w/cpp/atomic/memory_order`
    plus WG21 **P0371R1** *"Temporarily discourage memory_order_consume"*
    (`open-std.org/jtc1/sc22/wg21/docs/papers/2016/p0371r1.html`). Java Memory Model — **JSR-133**
    (`jcp.org/en/jsr/detail?id=133`), normative in *JLS* Ch. 17
    (`docs.oracle.com/javase/specs/jls/se17/html/jls-17.html`); `volatile` happens-before,
    `java.util.concurrent.atomic` / `VarHandle`. Rust — `doc.rust-lang.org/std/sync/atomic/enum.Ordering.html`
    ("the same as those of C++20"; `Relaxed/Acquire/Release/AcqRel/SeqCst`, no `Consume`) +
    `doc.rust-lang.org/nomicon/atomics.html`. WebAssembly threads — the proposal Overview
    (`github.com/WebAssembly/threads/blob/main/proposals/threads/Overview.md`): `shared` memory +
    atomic load/store/`rmw`/`cmpxchg`/`fence` + `wait`/`notify`, "all atomic memory access
    instructions are sequentially consistent". Erlang/BEAM — share-nothing message passing, no user
    atomics surface (`erlang.org/doc/system/conc_prog.html`,
    `erlang.org/doc/system/ref_man_processes.html`). Pony — reference capabilities
    (`iso`/`trn`/`ref`/`val`/`box`/`tag`) give compile-time data-race freedom, no user atomics/locks
    (`tutorial.ponylang.io/reference-capabilities/reference-capabilities.html`); **ORCA** fully-concurrent
    GC with no STW/barriers, runtime-internal — Clebsch, Franco, Drossopoulou, Yang, Wrigstad, Vitek,
    *"Orca: GC and Type System Co-Design for Actor Languages"*, OOPSLA 2017, `doi.org/10.1145/3133896`.
    (Fetch caveat: cppreference returned 403 to the fetcher and the ORCA PDF was unparseable — those two
    rows are stated from the standard's `memory_order` definition + P0371R1, and from the ACM abstract,
    respectively; the URLs remain the authoritative cites.)
</content>
</invoke>
