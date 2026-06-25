# Design Note DN-35 — Env-Machine Reclamation (DN-32 Phase 2 / RFC-0027 §10)

| Field | Value |
|---|---|
| **Note** | DN-35 |
| **Status** | **Draft** (2026-06-25; direction capture — advisory, non-committal). Deliberates and recommends; **ratifies nothing, enacts nothing, ships no code**. Promotion past Draft is gated on the §10 Definition of Done + maintainer ratification (house rule #3, append-only). |
| **Feeds** | the **deferred big step** past the RFC-0027 §9 audit trail — making the AOT env-machine perform *actual* Mycelium-level reclamation (free / in-place reuse / region batching) driven by the RC analysis (E12 **Increment 3** / "FBIP reuse-token threading", `docs/planning/E12-Memory-Model-Build-Plan.md:107`; tracked as **task #6**, "MEM-4 follow-ons … recursion … gated"). The **load-bearing prerequisite** behind DN-36's high-performance loops — constant-memory FBIP iteration *needs* this real reclamation (DN-36 §6 items (a)–(d)). Builds on **DN-32** (three-layer memory; §6b KC-3 tension; §7 cross-hypha), **DN-33** (MEM-4 static uniqueness; the ratified §8.1 Q1–Q3), and **RFC-0027** (§9 audit record, §10 mechanism, §11 OQs). |
| **Date** | June 25, 2026 |
| **Decides** | *Nothing normatively* — advisory + design-direction capture. Records (1) that env-machine reclamation is **architecturally significant** (it mutates the *trusted* env-machine, DN-32 §6b's KC-3 tension) and therefore **design-first**; (2) a recommended design — **"static decisions, dynamic verification"**: explicit reclamation IR ops inserted by an *untrusted, re-checkable* pass, with the trusted core interpreting only `incref`/`decref`/`is-unique`/`reuse-or-alloc` + a runtime RC cell as the dynamic witness, adopting **drop-guided reuse** (Frame-Limited Reuse, Lorenzen–Leijen) over Perceus's upfront pairing; (3) the one **novel Mycelium obligation** (in-place reuse vs content-address identity) and the **RC ⊕ region** exactly-once coupling, each at its honest grounding strength; (4) the **`fast`/`certified`** trusted-core-small split (ADR-032); (5) an **open-question ledger** (Q1–Q7 + live-executor wiring) marking the gating set; and (6) the **smallest sound first increment** + sequencing. |
| **Task** | E12 Increment 3 / env-machine reclamation (the deferred behaviour-change leg past the §9 audit trail) |

> **Posture (transparency rule / VR-5 / G2).** This note **synthesises two sourced research records**
> — `research/16-env-machine-reclamation-internal-RECORD.md` (repo ground truth: the `eval_machine`
> seam, the `RcCell` probe, the reference RC-evaluator as oracle, the open questions Q1–Q7) and
> `research/17-env-machine-reclamation-prior-art-RECORD.md` (external prior art + the recommended
> design) — into a design direction. It **enacts nothing**: no RFC/ADR/DN status moves, no normative
> text changes, no code or property test ships. The grounding split is load-bearing and held
> throughout: the single-mechanism prior-art results (precise-RC garbage-freeness, FIP
> no-alloc/constant-stack, frame-limited reuse) are **`Proven` at their source** and Mycelium inherits
> their premises via immutability + acyclicity; the **two Mycelium-specific obligations** (reuse vs
> content-identity; eager-RC ⊕ batched-region exactly-once) are **`Empirical`/`Declared`** until
> Mycelium states + property-tests them; the §2 audit-trail correspondence is **`Declared`** because
> the env-machine Rust-manages values today. Every reclamation event is reified and `EXPLAIN`-able
> (no black boxes); every refusal is never-silent (G2).

---

## §1 Purpose & honest scope

RFC-0027 fixed the reclamation **mechanism** (precise reference counting; LR-9 acyclicity *is*
Perceus's garbage-free precondition). DN-32 settled the **three-layer** runtime architecture; DN-33
ratified the **MEM-4 static uniqueness** design (additive, sound-but-incomplete, the runtime `RcCell`
probe as fallback). MEM-1/2/3 landed the runtime substrate, and the RFC-0027 §9 AOT **audit-trail
bridge** (`crates/mycelium-mlir/src/rc_plan.rs`) now consumes the static analysis at execution time.

**This note is about the next, deferred step:** making the AOT env-machine perform **actual
Mycelium-level reclamation** — free / in-place reuse / region batching — *driven by* the RC analysis,
rather than today's state where the env-machine **Rust-manages values** and the §9 records are an
**observable audit trail only, not a behaviour change**. The module honesty note says this plainly:
"the AOT env-machine … **still Rust-manages the values** — it does not perform Mycelium-level
reclamation … Threading actual reclamation into the env-machine is the deferred big step (E12 /
RFC-0027 §10)" (`rc_plan.rs:13-20`).

**Why design-first (this note) before any implementation.** This step is **architecturally
significant** because it mutates the **trusted** env-machine — the AOT interpreter that, with the
reference interpreter, is Mycelium's trusted execution base. DN-32 §6b names this exactly as the KC-3
tension: a reclamation/uniqueness mechanism "adds compiler surface, inference rules, and a correctness
obligation." Growing behaviour inside the trusted machine is precisely the move that warrants a
deliberated, research-backed target rather than a guess (KC-3 / G2 — flag, don't guess).

**The discipline that makes this tolerable: it is additive.** As with MEM-4 (DN-33 §2), the whole
design is shaped so that a bug is **a wrong audit record / a missed optimization / a caught dynamic-gate
copy — never a wrong value.** The runtime `RcCell` probe (`rc.rs`) stays the sound fallback: if static
analysis guesses wrong, the value degrades to a fresh allocation, never a use-after-free. State this as
the load-bearing invariant the whole note preserves:

> **Env-machine reclamation only ever *schedules* frees/reuse the runtime RC cell still verifies. A
> wrong static decision costs performance (an extra copy / a missed reuse / an over-conservative audit
> record), never memory safety.** The dynamic `is-unique` gate is the mandatory safety valve.

## §2 Current state — ground truth (`research/16`, file:line re-verified)

Grounded in `research/16` with each claim re-checked against source on this branch.

### §2.1 The runtime + static substrate is BUILT

- **L2 runtime RC cell (MEM-2).** `crates/mycelium-std-runtime/src/rc.rs` — `RcCell<T>` wraps
  `std::rc::Rc<T>` (`rc.rs:102`); `clone_ref` increments (`rc.rs:141`); `drop_ref` (`rc.rs:171`)
  probes `Rc::strong_count` (`rc.rs:178`): `== 1` ⇒ `RcProbe::UniqueOwner(T)` (FBIP-reuse-eligible) +
  **emit one `ReclamationRecord(RcZero)`** via the sink (`rc.rs:188`, enforced-by-construction — G2);
  `> 1` ⇒ `RcProbe::Shared`. `RcCell<T>` is **`!Send + !Sync`** by construction (`rc.rs:23`; the
  DN-33 §8.1 Q1 → Option A commitment). **`Exact`** per-op (reads `strong_count` at call time).
- **L1 EXPLAIN/audit record + sink (MEM-1).** `reclamation.rs` — `ReclamationRecord` (five-field
  RFC-0027 §9 set: `scope_id`, `sweep_epoch`, `trigger`, `value_meta_hash: ContentHash`,
  `channel_id`; `reclamation.rs:148`); `ReclamationTrigger ∈ {RcZero, ScopeExit, ChannelClose}`
  (exhaustive, G2; `reclamation.rs:93`); the `ReclamationSink` never-silent emit contract. `ScopeId`
  / `SweepEpoch` are `u64`-backed placeholders (`reclamation.rs:52,76`).
- **L3 region batching (MEM-3).** `region.rs` — a `Region` accumulates deferred entries and emits one
  `ReclamationRecord(ScopeExit)` per entry at close (`region.rs:44`); `RegionEpoch` is a monotonic
  counter advanced once per `Region::close` (`region.rs:147,154`), bridged to `SweepEpoch`. **`Exact`**
  per-entry. (Live-executor wiring of `with_region` / `RegionScope::close` into the runtime scope-exit
  paths is still deferred — `research/16 §1 3c`.)
- **MEM-4 static analysis (`mycelium-mir-passes`).** The RC-annotated IR (`rc_ir.rs`), the
  `emit_owned`/`emit_elided`/`emit_reuse` lowerings (`emit.rs`), the structural balance check
  (`balance.rs`), the Q5 corpus (`corpus.rs`), and the **reference RC-evaluator** (`eval.rs`).
- **The §9 AOT audit-trail bridge (MEM-4·AOT).** `rc_plan.rs` — `emit_reclamation_plan` runs
  `emit_elided` → `eval` → one `ReclamationRecord(RcZero)` per predicted reclamation
  (`rc_plan.rs:120`); `run_with_reclamation` runs the **unmodified** env-machine + emits the plan
  **additively** (`rc_plan.rs:163`). The record **count** is `Exact`; the audit-trail ↔ real-execution
  **correspondence is `Declared`** because the env-machine Rust-manages values (`rc_plan.rs:117-119`).
  `value_meta_hash` is the synthetic `rcplan:<id>` identity (`synth_hash`, `rc_plan.rs:102`), not the
  value's true RFC-0001 §4.6 content address — `Declared`.

### §2.2 The exact seam — where real reclamation threads in

The env-machine call chain (`crates/mycelium-mlir/src/aot.rs`): `run_core` (`aot.rs:147`) →
`run_core_with_budget` (`aot.rs:171`) → `run_core_with_effects` (`aot.rs:196`) → `eval_machine`
(`aot.rs:375`) — the **trampoline loop over an explicit heap control stack** (O(1) host stack; the
DN-05 / M-347 property DN-36 §2.1 leans on).

Values are the `AotVal` enum — `Core(CoreValue)` / `Closure{param,body,env}` / `Fix{name,body,env}` /
`FixGroup{defs,which,env}` (`aot.rs:68-104`) — held in `type Env = HashMap<Atom, AotVal>` (`aot.rs:106`).
Today each binding produces an `AotVal` **inserted into the environment by value**; when the binding
leaves scope, **Rust's `Drop` runs on the `HashMap` value — Rust-managed memory only.** The `Rc<Anf>`
in closures/`Fix` is Rust's RC for the **AST blocks**, *not* value-level RC (the two must stay
separate — §8 Q5).

The three seam points (`research/16 §2`):

- **A — value wrapping at `Step::Bind`.** The `match &binding.rhs { … }` produces a `Step::Bind(name,
  val)` (`aot.rs:421-471`) consumed by `env.insert(name, v)` (`aot.rs:521-522`). The hook: wrap each
  value in `RcCell<CoreValue>`; `clone_ref` on the `Alias` read path (`aot.rs:422` — the `Dup`
  equivalent).
- **B — scope-exit cleanup.** When a block's bindings are exhausted (`idx >= block.bindings().len()`)
  or `env` is replaced on a `Resume` frame pop, every binding in the departing env **must have
  `drop_ref` called before the `HashMap` drops** — the MEM-3 scope-exit reclamation point.
- **C — sink threading.** `eval_machine` (and the `run_core_with_effects` entry) must grow `sink:
  &mut dyn ReclamationSink`, `scope_id: ScopeId`, and `sweep_epoch: &mut SweepEpoch` parameters,
  propagated down every scope-exit path (signature growth — §8 Q4).

### §2.3 The reference RC-evaluator as the correctness oracle

`crates/mycelium-mir-passes/src/eval.rs` is an **abstract machine** (references + reclamation, not
data): `Machine{ next: AllocId, rc, reclaimed: Vec<AllocId> }`; `Const/Op/Swap` → `alloc` (rc=1);
`Dup` → `dup`; a move → `dec` (reclaim at 0; error on rc<0); `Borrow` → `assert_live`; `MoveUnique` →
verify rc==1 then `dec` (else `RcError::UnsoundUnique`, `eval.rs:187-195`). `eval` returns the
**reclaimed allocations in order** (`eval.rs:156`); `differential` compares the owned vs elided
reclamation **multiset** (order-independent). It is **straight-line only** — `Construct`/`Match`/`Lam`/
`App` (and recursion) return `RcError::UnsupportedNode` (`eval.rs:234-237`; G2 — never-silent).

**Why it is the oracle for DN-35:** the env-machine is a second execution path. When real reclamation
is threaded into `eval_machine`, the observable `ReclamationRecord` stream — *which* allocations are
reclaimed, and that `drop_ref` returns `UniqueOwner` exactly where the abstract machine predicts rc==1
— **must match the reference RC-evaluator's reclamation multiset** (mod scheduling concurrency). Real
reclamation is correct iff it agrees with the oracle.

## §3 Recommended design — "static decisions, dynamic verification" (`research/17 §2`)

**Recommendation: hybrid, sharply split — *static decisions, dynamic verification*.** The design axis
is (a) compile-time-inserted IR ops the machine *interprets* vs (b) a runtime RC cell the host manages
vs (c) hybrid. Mycelium adopts **(c), weighted toward static scheduling**, with a *minimal* dynamic
surface.

1. **Reclamation events are first-class IR ops, inserted by a separate *untrusted, re-checkable*
   pass, interpreted by the trusted core.** Keep the Beans/Perceus model: `dup`, `drop`,
   `drop-reuse`/`reset`, `reuse@token`, and a region `ScopeExit` op are **explicit nodes** in the
   RC-annotated IR (Mycelium already has `RcNode`, `rc_ir.rs`). The **trusted evaluator does only the
   minimal dynamic thing** — `incref`, `decref`, the `is-unique` test, the in-place-write-or-malloc
   branch — and treats *where* those ops live as given by the (untrusted) pass. This is exactly the
   Beans `λpure → λRC` and Perceus `⇝` posture (Ullrich–de Moura, IFL'19; Reinking–Xie–de Moura–Leijen,
   PLDI'21): the optimization is **outside the audited core** (KC-3; "no black boxes" — every
   reclamation event is reified + `EXPLAIN`-able).
2. **A runtime RC cell still exists** (`RcCell`, one count per heap value), because the
   **unique-vs-shared decision is inherently dynamic** (Beans Reset-Uniq vs Reset-Shared; Swift
   `isKnownUniquelyReferenced`). You cannot make in-place reuse sound by static analysis alone unless
   you adopt the full FIP linearity discipline (§7(B)). The count is the dynamic witness the
   interpreted ops consult.
3. **The trampoline FORCES this — and it is an advantage.** With an explicit heap-allocated control
   stack, **host-stack unwinding no longer runs your `drop`s** (the Rust pattern Beans contrasts
   against, where C++/Rust `Drop` rides stack unwinding). So drops **must** be explicit IR ops the
   trampoline schedules — there is no host frame to hang a destructor on. The trampoline thus turns
   reclamation into something **explicitly scheduled and auditable**, never an implicit unwinding
   side-effect — a direct match for "no black boxes" + the never-silent §9 record (`research/17 §2.4`).
   (This is also why §2.2 seam B must *explicitly* `drop_ref` each departing binding: there is no Rust
   destructor that can carry the sink without violating KC-3 — `research/16 §C3`.)
4. **Adopt drop-guided reuse over Perceus's upfront pairing.** Use **Frame-Limited Reuse**
   (Lorenzen & Leijen, OOPSLA'22): run reuse analysis *after* precise dup/drop insertion, so each
   `drop` is exactly the point a cell becomes free; when a `drop(x)` of a known size meets a later
   same-size allocation on the path, rewrite `drop → dropru` (yields a reuse token) and `alloc →
   ctor@r`. Simpler and **robust to small transformations** than Perceus "algorithm D" pairing, and —
   critically — **frame-limited**: peak reuse-pinned memory ≤ `const × control-stack depth`. In a
   trampolined machine the "stack" *is* the explicit heap control stack, so this bound becomes a
   **machine-visible, assertable quantity** — store the reuse token in the continuation/control-stack
   frame, and "peak reuse-pinned memory ≤ const × heap-control-stack depth" is countable for a property
   test / `EXPLAIN` (`research/17 §1.2, §2.5`).
5. **The runtime `is-unique` gate is the mandatory safety valve (never elide it in `fast` mode).**
   `dropru(x): if is-unique(x) then {decref children; &x} else {decref(x); SENTINEL}`; `ctor@r: if r ≠
   SENTINEL then write in place(*r) else alloc fresh`. A wrong static "reuse here" decision **degrades
   to a correct fresh allocation at runtime** — the worst case is "no reuse," never "use-after-free"
   (Beans Reset-Shared / Perceus NULL-token / Swift COW). This *is* §1's additive invariant, made
   operational.

**Net:** compile-time-inserted ops for *scheduling* + a runtime RC cell as the *dynamic witness*, with
the trusted dynamic surface minimized to `incref`/`decref`/`is-unique`/`reuse-or-alloc`.

## §4 Where the premise pays off — EASIER (`research/17 §7`)

Mycelium's value model is a **tailwind**, not merely compatible:

- **Acyclic + RC ⇒ Perceus is complete and garbage-free by construction.** Perceus Thm 2 (garbage-free:
  every heap cell reachable) holds in its **strong, non-cycle form** *because* the values are immutable
  and acyclic — "mutable references are the only source of cycles," and Mycelium has none (LR-8/LR-9;
  Perceus §2.7.4). **No cycle collector is needed.** This is the single most important external finding:
  the one fundamental limitation of RC does not apply here (**`Proven` for the acyclic fragment** at the
  source; Mycelium satisfies the premise by construction).
- **Static uniqueness (MEM-4) can promote checks `Empirical → Proven`.** The `is-unique` question *is*
  the native ownership question value semantics already asks. Where MEM-4 (DN-33) proves a value
  sole-owned, the runtime `rc==1` probe (`Empirical`) is replaced by a compile-time certainty
  (`Proven`-promotable) — pushing the guarantee tag up exactly where the analysis can prove it (DN-36
  §7's three-label EXPLAIN ladder).

## §5 The one novel obligation — HARDER: in-place reuse vs content-address identity (`research/17 §4`)

This is the **genuinely Mycelium-specific** problem, harder than Koka/Lean/Swift, with **no prior
proof**. In-place reuse mutates a cell's bytes; Mycelium **value identity = content hash** (RFC-0001
§4.6 / ADR-003). If anything still holds the **old** content-hash identity of that cell, in-place
mutation silently changes what that identity points to — a correctness break the pointer-identity
languages never face.

**Resolution — a checked side-condition, not a hope:**

- **rc==1 ⇒ no live alias ⇒ no observer of the old identity.** Reuse fires **only** at rc==1 (Beans
  Reset-Uniq / Perceus `is-unique` / Clojure transients). If rc==1, no other reference exists, so no
  live computation holds the old content-hash to observe the change: the mutation is **unobservable** —
  the value is *consumed* and a new value is *born* in the same memory. This is Clojure's
  **transient** discipline (`transient!` → mutate-while-uniquely-owned → `persistent!`; the old handle
  is contractually dead) at cell granularity. Content-address identity is preserved because **identity
  is a property of *live* values, and reuse only touches *dead* ones.**
- **The intern / hash-cons table is a *weak* index.** If a uniquely-owned cell is interned in a
  content-address table that holds a reference, its effective rc ≥ 2 via the table. Treat the global
  hash-cons table as a **weak map** (entries don't pin); on `drop-reuse` of an interned cell, **evict
  the table entry first** (its hash key is about to go stale), *then* reuse; if you cannot evict
  atomically, **copy-on-reuse**. Decouple the hashed identity from the runtime cell (Unison's pattern:
  source hashed, runtime keeps a separate compiled representation keyed by hash). If the cell caches
  its own content-hash inline, the `reuse` write must invalidate/recompute it.

**Concrete rule for Mycelium:** *reuse a cell in place iff (rc==1 ∧ same-shape ∧ dead-after) AND (the
cell is not pinned in the content-address index, OR the index entry is evicted as part of the reuse);
otherwise copy-on-reuse.* **Tag: `Empirical`/`Declared`** — there is no off-the-shelf proof; the survey
offers *patterns* (transients, weak intern tables, Unison's decoupled rep) but **no theorem**, so this
becomes a **Mycelium-stated, property-tested side-condition (S5)** before any `Proven` tag (VR-5).

## §6 RC ⊕ region coupling — exactly-once (`research/17 §5`)

Combining per-value RC (`RcZero`) with batched region/scope reclamation (`ScopeExit`) **without
double-free or missed-free** is well-trodden in *design* (Gay & Aiken, "Memory Management with Explicit
Regions," PLDI'98: a region counts *pointers into it*; internal references are free; deletion is
dynamically gated by the count) but has **no off-the-shelf mechanized soundness theorem** as tight as
Perceus's single-mechanism Thm 2.

**Recommended two-trigger, single-owner, count-respecting protocol:**

- **`RcZero` is the eager trigger; `ScopeExit` is the batch trigger — they must not both free a cell.**
  A cell freed by `RcZero` is **removed from its region's free-list at that moment** (the **double-free
  guard**). `ScopeExit` batch-frees only cells still rc>0 that are **region-internal with zero
  cross-region references** (Gay–Aiken's "count pointers into the region" check — the **missed-free
  guard**). Cells that **escaped** (rc>0 via a cross-region ref) are **promoted to the parent region,
  never freed**.
- **Cross-region references are the only ones that need a count** (internal references borrow-elided —
  MEM-4); escape analysis decides region-batch vs RC.
- **Ordering invariant (the proof obligation):** at `ScopeExit`, (i) run `drop` for each scope-owned
  root → (ii) cascade `RcZero` frees → (iii) batch-free whatever remains region-internal and now
  unreferenced. State and property-test the invariant: **"a cell is freed by exactly one of {RcZero,
  ScopeExit}, and only when unreachable."**
- **Never-silent (G2):** an escape the analysis cannot prove absent → conservatively *RC-managed +
  flagged*, never silently region-freed.

**Tag: `Empirical`/`Declared`** — Gay–Aiken give a design + safety argument, not a mechanized proof of
the *combined* eager+batch protocol; this is a **Mycelium-owned, property-tested** exactly-once
invariant until checked.

## §7 Trusted-core-small (KC-3) + tunable certification (ADR-032)

The literature gives two strategies; Mycelium uses **(A) as the floor, (B) as the ceiling**
(`research/17 §6`), tied to ADR-032 tunable certification:

- **(A) `fast` (default) — reclamation as a separate, untrusted, re-checkable pass + the retained
  dynamic gate.** The audited core interprets a *fixed, tiny* op set; placement is produced by an
  untrusted pass whose two safety properties hold even if it is buggy: the **dynamic safety valve** (a
  wrong reuse decision degrades to a fresh alloc — §3.5) and **independent re-checkability** (a verifier
  can re-derive "every value consumed exactly once" without trusting the pass). A buggy pass costs
  **performance, not soundness** — the `Empirical` tier (the runtime `rc==1` probe is the witness).
- **(B) `certified` — an FP² FIP-linearity check that *proves* no-alloc / constant-stack and can drop
  the gate.** For the FIP-linear subset (FP², Lorenzen–Leijen–Swierstra, ICFP'23: reuse credits `◇k`
  consumed linearly; Thm 2 `|S| = |S′|`, no (de)alloc; Thm 4 constant stack — mechanized, **`Proven`**
  with side-conditions), reuse is *statically guaranteed*, so the runtime `is-unique` gate can in
  principle be **elided** — the strongest "core does almost nothing" result. The check is **syntactic**
  (no full linear type system), i.e. a separate pass that *certifies* rather than *transforms* — the
  `Proven` tier.

The swap between dynamically-gated reuse (A) and statically-certified reuse (B) is itself a **reified,
`EXPLAIN`-able choice, never silent** (ADR-032). DN-36 §7's three-label per-loop EXPLAIN ("in-place
(statically unique)" / "in-place (`rc==1` runtime)" / "copying (shared)") is the surface of this ladder.

## §8 Open-question ledger — the DN-35 decision agenda (`research/16 §5`)

Each question is grounded in a repo fact; the **gating set** for the first increment is marked. (Q1–Q3
mirror DN-33 §8 but for the *env-machine behaviour-change* surface, where DN-33 settled the *static
analysis* surface.)

| # | Question | Status / what it gates | Grounding |
|---|---|---|---|
| **Q1** | Cross-hypha atomic RC — Option A (sole-move-only / affine boundary) vs B (shared-crosses-atomic) | **A ratified for R1 by DN-33 §8.1 Q1**; `RcCell<T>` stays `!Send`; B → R2 (`xloc`/`mesh`). *Not gating* the first increment (the straight-line fragment never crosses a boundary). | `rc.rs:23` (`!Send + !Sync`); DN-33 §8.1 Q1 |
| **Q2** | Real vs synthetic `ContentHash` at reclamation time | Synthetic `rcplan:<id>` today (`Declared`); real reclamation should supply the value's RFC-0001 §4.6 hash (stored in `RcCell` metadata or computed at alloc). **Gates §5** (the content-identity side-condition needs a real hash to evict/invalidate against). | `rc_plan.rs:102` (`synth_hash`) |
| **Q3** | Flat `ScopeId` vs scope-tree identity | `ScopeId` is a flat `u64` placeholder; `RegionEpoch` is the current nesting marker (monotonic, inner<outer). Choose: reuse `RegionEpoch` as the seam's nesting marker, or introduce a scope-tree identity `(parent, child_epoch)`. **Partially gating** (the first increment can ship with flat `ScopeId` + a single region). | `reclamation.rs:52`; `region.rs:147` |
| **Q4** | Thread `sink`/`scope_id`/`sweep_epoch` through `eval_machine` (signature growth) | The mechanical prerequisite — `run_core_with_effects` + `eval_machine` grow three params, propagated down. **GATING** — nothing real reclaims without the sink in the machine. | `aot.rs:196,375` (seam C) |
| **Q5** | Keep AST-level `Rc<Anf>` separate from value-level RC | `Rc<Anf>` is a codegen detail (block/continuation sharing); `RcCell<CoreValue>` is value-level. They **must not conflate** — no value-RC metadata leaks into AST blocks; AST sharing must not affect reclamation order. **Design constraint** (not a choice — an invariant to preserve). | `aot.rs:68-104` (`Rc<Anf>` in `Closure`/`Fix`) |
| **Q6** | Does the static plan **drive** runtime reclamation, or stay audit-only? | Today the plan is observability (`run_with_reclamation` runs the unmodified machine + emits additively). Real reclamation makes the plan *drive* the machine (skip runtime probes where MEM-4 is statically certain), turning the RC-evaluator into a **differential test** (static predictions vs runtime actuals must match). **GATING the *meaning* of the increment** — "real reclamation" *is* answering Q6 = drive. | `rc_plan.rs:163` (audit-only today) |
| **Q7** | Recursion (`Fix`/`FixGroup`) RC lifetime | **Refused today** — `eval.rs` returns `UnsupportedNode` for non-straight-line terms; `emit.rs:129-130` refuses `Fix`/`FixGroup`. The seam must **not silently mis-handle recursion**: refuse it in `eval_machine`'s real-reclamation path (mirror the abstract machine's limitation) until a recursive RC-evaluator + back-edge uniqueness land (DN-36 §6 (a)–(c)). **Must stay never-silent** (G2). *Deferred, not gating* the first (straight-line) increment. | `eval.rs:234-237`; `emit.rs:129-130` |
| **+** | Live-executor wiring (`with_region` / `RegionScope::close` at hypha-exit) | Standalone region machinery is built; the live Scope/Runtime does not yet call it. Needed for the `ScopeExit` trigger to fire on real execution (the §6 coupling). **Gating §6**, not the first straight-line free. | `research/16 §1 3c` |

**The gating set for the first increment:** **Q4** (thread the sink — mechanical prerequisite), **Q6 =
drive** (the increment's *definition*), and **Q2** (only insofar as §5's content-identity check needs a
real hash — the *minimal* first increment can defer reuse and free with synthetic hashes, freeing only,
no in-place reuse). Q1/Q3/Q5/Q7 and live-executor wiring are **not gating** the smallest sound first
step (§9).

## §9 Sequencing / increments

The smallest sound first increment, and the order after it. Each increment is **differential-checked
against the reference RC-evaluator** (the §2.3 oracle) *and* the existing interp≡AOT differential
(NFR-7); the runtime `is-unique` gate is retained throughout (§3.5), and the runtime `RcCell` probe
stays the sound fallback.

1. **Increment 1 — real free for the straight-line fragment (smallest sound step).** Thread
   `sink`/`scope`/`sweep_epoch` through `eval_machine` (Q4); wrap `AotVal` values in
   `RcCell<CoreValue>` (seam A); call `drop_ref` at the scope-exit / overwritten-bind seams (seam B).
   This is **real Mycelium-level reclamation for `Const/Let/Op/Swap`** — the env-machine *frees*
   (not just audits), the plan now *drives* (Q6), **keeping the dynamic `is-unique` gate** and **no
   in-place reuse yet** (free-only; synthetic or real hash per Q2). Differential: the `ReclamationRecord`
   stream must equal the RC-evaluator's reclamation multiset.
2. **Increment 2 — drop-guided reuse (FBIP).** Add `dropru`/`ctor@r` token threading (§3.4) so a
   uniquely-owned same-shape allocation reuses storage in place — with the §5 content-identity
   side-condition (S5) stated + property-tested, and the §1.2 frame-limited bound asserted over the heap
   control-stack depth.
3. **Increment 3 — FP²-certified gate-elision (`certified` mode).** The FIP-linearity check (§7(B))
   that proves no-alloc / constant-stack and *drops* the runtime gate for the certified subset
   (ADR-032 `Proven` tier).
4. **Recursion (later) — `Fix`/`FixGroup` RC.** Requires the recursive RC-evaluator + back-edge
   uniqueness (DN-36 §6 (a)–(c)); **stay never-silent on refusal** until it lands (Q7). This is the leg
   DN-36's constant-memory loops depend on.

The RC ⊕ region coupling (§6) + live-executor wiring slot in alongside, gated on the exactly-once
invariant being stated + property-tested.

## §10 Guarantee posture (VR-5) + Definition of Done

**Guarantee posture (the honest split this note holds):**

- **Single-mechanism prior-art results = `Proven` *at their source*; Mycelium inherits their premises.**
  Precise-RC garbage-freeness (Perceus Thm 2, given acyclicity — Mycelium has it by construction);
  FIP no-alloc/constant-stack (FP² Thm 2/4, mechanized, given FIP-linearity + unshared args);
  frame-limited reuse (Lorenzen–Leijen §3.4, given 0/1-use uncaptured tokens). Mycelium satisfies the
  acyclicity premise by LR-8/LR-9; the others are conditions the pass must *enforce*.
- **The two Mycelium-specific obligations = `Empirical`/`Declared`** until stated + property-tested:
  (i) **reuse vs content-address identity** (§5 — no prior proof; the highest-priority new
  side-condition); (ii) **eager-RC ⊕ batched-region exactly-once** (§6 — Gay–Aiken design, no combined
  mechanized proof).
- **Current §9 audit trail = `Exact` count, `Declared` correspondence** — the env-machine Rust-manages
  values today (`rc_plan.rs:117-119`); the synthetic `value_meta_hash` is `Declared` (Q2).
- **The recommended design (static-decisions/dynamic-verification; drop-guided reuse; the `fast`/
  `certified` split) is a `Declared` design proposal**, not a ratified or proven optimum.

**Definition of Done (the gate for Draft → Accepted).** This note moves to **Accepted** when the
maintainer ratifies the design: (1) **static-decisions / dynamic-verification** with the trusted core
interpreting only `incref`/`decref`/`is-unique`/`reuse-or-alloc`; (2) **drop-guided (Frame-Limited)
reuse** over upfront pairing, with the runtime `is-unique` gate as the mandatory safety valve; (3) the
**content-address side-condition** (§5 — reuse only at rc==1, weak intern table, evict-then-reuse-or-copy);
(4) the **RC ⊕ region exactly-once invariant** (§6 — single-owner, free-list removal on `RcZero`,
region-count missed-free guard, escape→promote); and (5) the **`fast`/`certified` split** (ADR-032). As
with DN-33, Accepted ratifies the **design direction**; it **enacts no code** — the build is the forward
epic (E12 Increment 3 / task #6), beginning with §9's Increment 1 (real free for the straight-line
fragment) behind the §2.3 oracle's differential, with the runtime `RcCell` probe remaining the sound
fallback throughout. Promotion is append-only; nothing here moves another doc's status (house rule #3).

## §11 Relation to the corpus & grounding

- **Corpus:** RFC-0027 §7.1 (acyclicity = Perceus precondition), §7.3 (affine channel transfer), §9
  (the EXPLAIN/audit record — never-silent contract), §10 (the reclamation *mechanism* — RC unifies
  reclaim + copy/mut), §11 (open questions); DN-32 §2.2 (Layer-2 static uniqueness), §6b (the KC-3
  tension this note's behaviour-change confronts), §7 (cross-hypha sub-question); DN-33 (MEM-4 additive
  principle §2; the ratified §8.1 Q1 → Option A / Q2 → separate RC-IR / Q3 → differential `Empirical`);
  DN-36 (the high-performance loops this note is load-bearing for — §6 items (a)–(d) need real
  reclamation across `Fix`); ADR-032 (tunable certification — the `fast`/`certified` split); ADR-003 /
  RFC-0001 §4.6 (content-address value identity — the §5 tension). Value-model basis: **LR-8**
  (immutability), **LR-9** (acyclicity); **KC-3** (small auditable kernel); **VR-5 / G2**
  (downgrade-don't-overclaim / never-silent). Build plan: `docs/planning/E12-Memory-Model-Build-Plan.md:107`
  (Increment 3 / FBIP reuse-token threading).
- **Repo ground truth (file:line):** `crates/mycelium-mlir/src/aot.rs` (`AotVal:68-104`, `Env:106`, the
  `run_core:147` → `run_core_with_budget:171` → `run_core_with_effects:196` → `eval_machine:375` chain;
  `Step::Bind`/`env.insert:421-522` = seam A/B; seam C = the missing sink params);
  `crates/mycelium-mlir/src/rc_plan.rs` (the §9 audit-trail bridge; honest scope `:13-20`; `synth_hash:102`;
  `emit_reclamation_plan:120`; `run_with_reclamation:163`); `crates/mycelium-std-runtime/src/rc.rs`
  (`RcCell:102`, `clone_ref:141`, `drop_ref:171`, the `RcZero` emit `:188`, `!Send:23`);
  `crates/mycelium-std-runtime/src/reclamation.rs` (`ReclamationRecord:148`, `ReclamationTrigger:93`,
  `ScopeId:52`/`SweepEpoch:76`); `crates/mycelium-std-runtime/src/region.rs` (`RegionEpoch:147`, the
  batched `ScopeExit:44`); `crates/mycelium-mir-passes/src/eval.rs` (the oracle — `eval:156`, the
  reclamation log, `UnsupportedNode:234-237`, `UnsoundUnique:187`); `crates/mycelium-mir-passes/src/emit.rs`
  (`Fix`/`FixGroup` refusal `:129-130`). Per the api-index caveat, source is ground truth; these are the
  records this note synthesises.
- **External prior art (`Proven`/`Empirical` at source strength; transfer to Mycelium is `Declared`):**
  Reinking, Xie, de Moura, Leijen, *Perceus: Garbage Free Reference Counting with Reuse* (PLDI 2021,
  doi:10.1145/3453483.3454032 — `drop-reuse`/reuse-token, λ₁ linear resource calculus, Thms 1–4, the
  acyclicity-conditioned garbage-free theorem); Lorenzen & Leijen, *Reference Counting with
  Frame-Limited Reuse* (OOPSLA 2022, doi:10.1145/3547634 — drop-guided reuse, the frame-limited §3.4
  bound); Ullrich & de Moura, *Counting Immutable Beans* (IFL 2019, arXiv:1908.05647 — `reset`/`reuse`
  operational semantics, owned-vs-borrowed, the dynamic gate; compiler proof was future work at
  publication); Lorenzen, Leijen, Swierstra, *FP²: Fully in-Place Functional Programming* (ICFP 2023,
  doi:10.1145/3607840 — λfip, reuse credits `◇k`, Thm 2 `|S|=|S′|` / Thm 4 constant stack, mechanized);
  Gay & Aiken, *Memory Management with Explicit Regions* (PLDI 1998, doi:10.1145/277650.277748 — the
  RC-gated region-free design, internal references uncounted); Swift `isKnownUniquelyReferenced` /
  `Builtin.isUnique` (the runtime uniqueness gate behind COW); Clojure transients + Bagwell HAMT
  (path-copying, unique-ownership transient mutation); Unison content-addressed runtime (decoupled
  hashed identity vs runtime cell).

---

## Meta — changelog

- **2026-06-25 — Created (Draft, advisory) — authored.** Research-backed direction capture for
  **env-machine reclamation** — the deferred big step past the RFC-0027 §9 audit trail (DN-32 Phase 2 /
  RFC-0027 §10), synthesising `research/16-env-machine-reclamation-internal-RECORD.md` (repo ground
  truth: the `eval_machine` seam, the `RcCell` probe, the reference RC-evaluator oracle, the open
  questions Q1–Q7) and `research/17-env-machine-reclamation-prior-art-RECORD.md` (external prior art +
  recommended design). Records: the **honest scope** (today the env-machine **Rust-manages values** and
  the §9 records are an **audit trail only** — `rc_plan.rs:13-20`; real reclamation is **additive**, a
  bug is a wrong audit record / missed optimization / a caught dynamic-gate copy, **never a wrong
  value**, with the runtime `RcCell` probe the sound fallback); the recommended design
  **"static decisions, dynamic verification"** (explicit reclamation IR ops from an untrusted,
  re-checkable pass; the trusted core interprets only `incref`/`decref`/`is-unique`/`reuse-or-alloc`;
  the trampoline *forces* explicit scheduled drops = auditable, no black boxes; **drop-guided
  (Frame-Limited) reuse** over Perceus upfront pairing, frame-limited bound machine-visible on the heap
  control stack; the `is-unique` gate the mandatory safety valve); the **EASIER** payoff (acyclic+RC ⇒
  Perceus complete & garbage-free, no cycle collector; MEM-4 promotes checks `Empirical → Proven`); the
  one **HARDER** obligation (**in-place reuse vs content-address identity** — reuse only at rc==1, weak
  intern table, evict-then-reuse-or-copy; `Empirical`/`Declared`, no prior proof); the **RC ⊕ region**
  exactly-once coupling (Gay–Aiken; `RcZero` free-list removal = double-free guard, region-count =
  missed-free guard, escape→promote; `Empirical`/`Declared`); the **trusted-core-small** `fast`
  (dynamic gate, `Empirical`) / `certified` (FP² FIP-linearity, gate-elided, `Proven`) split tied to
  **ADR-032**; the **open-question ledger** Q1–Q7 + live-executor wiring (Q1 → A ratified by DN-33; Q2
  real-vs-synthetic hash; Q3 flat-vs-tree ScopeId; Q4 sink threading — gating; Q5 keep `Rc<Anf>`
  separate; Q6 drive-vs-audit — the increment's definition; Q7 recursion refused, never-silent), with
  the **gating set** (Q4 + Q6=drive + Q2-for-reuse) marked; the **increment sequence** (real free for
  the straight-line fragment → drop-guided reuse → FP²-certified gate-elision → recursion later, each
  differential-checked against the reference RC-evaluator + the interp≡AOT differential); and the
  **guarantee posture** (single-mechanism prior-art `Proven` at source / inherited premises; the two
  Mycelium obligations `Empirical`/`Declared`; §9 audit trail `Exact` count / `Declared` correspondence;
  the design a `Declared` proposal). Notes this is the **load-bearing prerequisite** behind DN-36's
  high-performance loops (constant-memory FBIP needs this real reclamation). **Enacts nothing; moves no
  status; changes no normative text.** Promotion past Draft requires the §10 Definition of Done +
  maintainer ratification (house rule #3, append-only). CHANGELOG / Doc-Index / issues.yaml /
  docs/api-index owned by the integrating parent. (Append-only; VR-5; G2.)
