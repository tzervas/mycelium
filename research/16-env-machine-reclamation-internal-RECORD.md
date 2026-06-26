# Internal Research Dossier: Threading Actual Reclamation into the AOT Env-Machine

**DN-35 Ground Truth — Memory Reclamation at the Seam**

Date: 2026-06-25  
Status: Research extraction (read-only analysis, no code changes)  
Scope: Current state of the memory reclamation architecture + the exact seam where real reclamation threads in

---

## 1. Current State — L1/L2/L3 Layers Implemented vs. Stubbed

### Layer 1 — Affine/Linear Ownership (PRIMARY path)
**Status:** ✅ IMPLEMENTED (mostly implicit; Core IR assumes immutable values moving by default)

- **File:** `crates/mycelium-core/src/node.rs` (Core IR — unchanged by RC passes)
- **Guarantee:** `Exact` by Rust move semantics
- **What's built:**
  - Core IR stays pristine (KC-3) — no RC annotations on `Node`
  - ANF lowering (`mycelium-core/src/lower.rs`) lowers to ANF without RC
  - Values move by default; RFC-0006 (LR-8) enforces immutability
- **What's NOT built:**
  - Static uniqueness/borrowing analysis (Perceus/Lorenzen) — deferred to MEM-4 Phase 3
  - The affine `substrate` lever is scoped to external resources only (RFC-0006); widening to ordinary values is future

### Layer 2 — Optimized Reference Counting (for EXPLICIT sharing)
**Status:** ✅ IMPLEMENTED (static side done; runtime side complete; cross-hypha boundary remains open)

**2a. Static analysis side (MEM-4):**
- **Files:** `crates/mycelium-mir-passes/src/{rc_ir.rs, emit.rs, eval.rs, balance.rs, corpus.rs}`
- **What's built:**
  - **RC-annotated IR** (`RcNode` — mirrors Core IR first-order fragment): `Const/Var/Borrow/MoveUnique/Let/Op/Swap/Dup/Drop/DropAfter` (rc_ir.rs:30–150)
    - `Mode::Owned` (naive default) vs `Mode::Borrowed` (borrow-elided output)
    - `Dup`/`Drop` wrappers for refcount ops
    - `MoveUnique` annotation for `rc==1` reuse sites (Increment 2)
  - **Naive RC emission** (MEM-4·B0): `emit_owned` lowering `Node → RcNode`, all bindings `Owned` (emit.rs)
  - **Borrow elision** (MEM-4·1, Increment 1): `emit_elided` statically removes Dup ops for non-escaping bindings → `Borrow` + `DropAfter` (emit.rs)
  - **Reuse annotation** (MEM-4·2, Increment 2): `emit_reuse` marks sole-owner moves as `MoveUnique` (emit.rs)
  - **Reference RC evaluator** (Q3 differential oracle): `eval` function models RC discipline over straight-line fragment, verifies `semantics-preserved` + `no-use-after-free` (eval.rs:156–239)
  - **Structural balance check** (Q2 soundness): `balance` verifies `1 + dups == uses + drops` invariant (balance.rs)
  - **Measurement harness** (Q5 gate): `corpus` module measures dup-reduction (~91% Dup removal on test corpus); records are `Exact` (corpus.rs)
- **Guarantee tags:**
  - Naive/elided/reuse emission: `Exact` (structural mirrors; insertion deterministic)
  - Semantics preservation (elision): `Empirical` (verified by differential over corpus)
  - Reuse-site soundness: `Exact` (reference evaluator machine-verifies rc==1 at each `MoveUnique` — no false annotation)

**2b. Runtime RC cell (MEM-2):**
- **Files:** `crates/mycelium-std-runtime/src/rc.rs`
- **What's built:**
  - **RcCell<T>** (rc.rs:102–145): non-atomic intra-hypha handle, wraps `std::rc::Rc<T>`
    - `clone_ref()` → increment refcount (line 141–145)
    - `drop_ref()` → probe refcount (line 171–225) **+ emit ReclamationRecord** (line 182–188)
  - **RcProbe outcome** (rc.rs:line 81–90):

    ```rust
    if strong_count == 1:
        ReclamationRecord::new(scope_id, sweep_epoch, RcZero, value_meta_hash)
        → sink.emit(record)  // line 188 — LIVE TRIGGER WIRING for RcZero
        → return UniqueOwner(T)  // FBIP-reuse-eligible
    else:
        → return Shared  // other owners retain
    ```

  - **Guarantee:** `Exact` — one record emitted on last-ref; enforced-by-construction (rc.rs:170)
  - **NOT built:** Cross-hypha atomic RC (Arc) — DN-32 §7 named sub-question (Option A vs B)

**2c. EXPLAIN record + sink contract (MEM-1):**
- **Files:** `crates/mycelium-std-runtime/src/reclamation.rs`
- **What's built:**
  - **ReclamationRecord struct** (reclamation.rs:147–200): five-field RFC-0027 §9 spec
    - `scope_id: ScopeId` (u64 placeholder for canonical scope-tree identity)
    - `sweep_epoch: SweepEpoch` (u64 monotonic counter)
    - `trigger: ReclamationTrigger` — enum `{RcZero, ScopeExit, ChannelClose}` (exhaustive, G2)
    - `value_meta_hash: ContentHash` (synthetic rcplan hash for AOT; real content hash from runtime)
    - `channel_id: Option<ChannelId>` (for cross-boundary events)
  - **ReclamationSink trait** — never-silent contract: `fn emit(&mut self, record: ReclamationRecord)`
    - Every reclamation path must call `sink.emit()`; no silent drops (G2 enforced-by-construction)
  - **Guarantee:** `Declared` — the record type is normatively specified; operational coverage (all triggers wired) is `Declared` (depends on MEM-2/3)

**2d. AOT audit-trail bridge (MEM-4·AOT):**
- **Files:** `crates/mycelium-mlir/src/rc_plan.rs`
- **What's built:**
  - **emit_reclamation_plan()** (rc_plan.rs:120–137): runs `emit_elided()`, pipes through reference RC evaluator `eval()`, emits `ReclamationRecord(RcZero)` per predicted reclamation
    - Consumes the static analysis → produces the audit trail
    - Returns typed `RcPlanError` for out-of-fragment terms (G2 — never-silent empty plan)
  - **run_with_reclamation()** (rc_plan.rs:163–172): runs the trusted AOT env-machine unmodified + emits plan additively
    - `value` computed by `run_core()` — unchanged
    - `reclaimed: Option<usize>` — the audit-trail record count (or None for out-of-fragment)
  - **Honest scope** (rc_plan.rs:11–32): the env-machine **still Rust-manages values** — this is purely observability, not behaviour change
    - Audit trail ≠ actual reclamation yet
    - `value_meta_hash` is synthetic `rcplan:<id>` (abstract machine tracks refs, not content)
  - **Guarantee:** Record count `Exact`; audit-trail ↔ real execution correspondence `Declared` (env-machine Rust-manages)

### Layer 3 — Region-based Batched Reclamation (within scopes)
**Status:** ✅ IMPLEMENTED (L3 data + closure/guard forms complete; scope-tree integration ready)

**3a. Region batching (MEM-3):**
- **Files:** `crates/mycelium-std-runtime/src/region.rs`
- **What's built:**
  - **Region struct**: collects defer entries in a Vec, emits batched `ScopeExit` records at close
  - **RegionEpoch** (u64 monotonic counter): encodes child→root LIFO + weak sibling coupling (RFC-0027 OQ-1 resolved)
  - **region.close(sink)** → iterates deferred hashes, emits one `ReclamationRecord(ScopeExit, scope_id, epoch, hash)` per defer (region.rs docs + close method)
  - **Guarantee:** `Exact` — one record per defer; monotonic epoch encodes nesting order (inherited from `close`'s counter increment)

**3b. Structured-concurrency wrapper (MEM-3 executor integration):**
- **Files:** `crates/mycelium-std-runtime/src/scope_region.rs`
- **What's built:**
  - **with_region(sink, body)** (scope_region.rs:128–139): closure-form entry point
    - Opens Region → runs `body(&mut region)` → unconditionally closes on return → emits all batched ScopeExit records
    - **Guarantee:** `Exact` — close always called (normal path); enforced-by-construction (no conditional branch can skip it)
  - **RegionScope** (scope_region.rs:182–197): explicit-close guard for interleaved deferrals
    - Caller must call `close(sink)` explicitly; debug-build panic if dropped with pending entries
    - **Guarantee:** `Exact` — consuming `close` ensures one call; double-close is a type error
  - Nested `with_region` calls produce `inner_epoch < outer_epoch` by construction (monotonic counter — Exact)

**3c. Live executor wiring:**
- **FLAG:** ScopeExit trigger wiring into the **live Scope/Runtime** is deferred — region/scope_region are standalone data structures + guards
  - The runtime tier (`mycelium-std-runtime` Scope, task, executor) must call `with_region` / `RegionScope::close` in its scope-exit paths
  - This is explicitly out of scope for the MEM-3 crate (a leaf in the build plan); the **orchestrator must wire it** during integration

---

## 2. The Env-Machine Seam — Where Real Reclamation Threads In

### The exact function + lines: THE CALL CHAIN

```
1. run_core(node, prims, swap)
   ↓ (aot.rs:147–153)
2. run_core_with_budget(node, prims, swap, fuel, max_depth)
   ↓ (aot.rs:171–181)
3. run_core_with_effects(node, prims, swap, fuel, max_depth, budgets)
   ↓ (aot.rs:196–208)
4. lower_to_anf(node)  // mycelium-core/lower
   ↓
5. eval_machine(top_anf, top_env, prims, swap, fuel, max_depth, budgets)
   ↓ (aot.rs:375–450+)
   ↳ loop: idx += 1 per binding; Step::Bind or Step::Switch
   ↳ match binding.rhs: Const, Alias, Op, Swap, Construct, Match, App, Lam
   ↳ Step::Bind(name, val) → env.insert(name, val)
        ↓ (aot.rs:417–449)
        └─ val is an AotVal enum (Core/Closure/Fix/FixGroup)
```

### The VALUE REPRESENTATION in the machine

**AotVal enum** (aot.rs:78–104):

```rust
enum AotVal {
    Core(CoreValue),        // repr value or datum — normal form
    Closure { param, body, env },  // lambda closure
    Fix { name, body, env },       // recursive suspension
    FixGroup { defs, which, env }, // mutual recursion group
}
```

**Env type** (aot.rs:106):

```rust
type Env = HashMap<Atom, AotVal>;
```

**Key observation:** Currently, **each binding produces an `AotVal` that is immediately inserted into the environment by value**. When the binding goes out of scope, Rust's `Drop` runs on the `HashMap` value — **Rust-managed memory only**.

### HOW RUST CURRENTLY MANAGES VALUES

1. **AotVal::Core(CoreValue)** — contains `Value` (repr) or `Datum` (data)
   - Allocated by primitives/swaps/constructors → dropped when environment entry is removed
   - No explicit refcount; Rust ownership model handles it

2. **AotVal::Closure / Fix / FixGroup** — the `body: Rc<Anf>` and `env: Env` (HashMap)
   - HashMap values are cloned when inserted; dropped when overwritten or environment exits scope
   - Rc is Rust's reference counting — it's there for the **AST blocks** (Anf), not **values**

3. **No actual Mycelium-level RC cell** anywhere in the machine — just Rust's Rc for AST sharing

### THE EXACT SEAM WHERE REAL RECLAMATION THREADS IN

**Point A: Step::Bind() result handling (aot.rs:415–450)**

```rust
// Current code (line 415–449):
let step: Step = {
    let binding = &block.bindings()[idx];
    let name = binding.name.clone();
    match &binding.rhs {
        Rhs::Const(v) => Step::Bind(name, AotVal::Core(CoreValue::Repr(v.clone()))),
        Rhs::Alias(a) => Step::Bind(name, lookup(&env, a)?),
        Rhs::Op { prim, args } => {
            // … evaluate prim → Value …
            Step::Bind(name, AotVal::Core(CoreValue::Repr(result)))
        }
        // … etc …
    }
};

// Then:
if let Step::Bind(n, v) = step {
    env.insert(n, v);  // Line ~397: binding added to environment
    idx += 1;
}
```

**The seam:** Between `Step::Bind(name, val)` and `env.insert(name, val)`, **the reclamation infrastructure must hook in**:

1. **Wrap each value in an RcCell<T>** — the value becomes `AotVal::Core(CoreValue::Repr(v))` wrapped as `RcCell<CoreValue>`
2. **Clone_ref on read** (Alias case) — when looking up an existing binding for reuse, increment its refcount
3. **Drop_ref on scope-exit** — when a binding is no longer in scope (overwritten or scope ends), call `drop_ref()` to:
   - Check if rc==1 (UniqueOwner → FBIP-reuse-eligible)
   - Emit ReclamationRecord(RcZero) if it's the last reference
   - Reclaim the allocation (Rust's Rc handles the memory)

**Point B: Environment cleanup at block-exit (aot.rs:390–412)**

```rust
// Current loop: (line 389–412)
loop {
    if idx >= block.bindings().len() {
        let val = lookup(&env, block.result())?;
        match stack.pop() {
            None => return Ok(val),  // Line 394: return to caller
            Some(Frame::Resume(c)) => {
                // Resume in parent block with new env
                let mut e = c.env;
                e.insert(c.name, val);
                block = c.block;
                env = e;  // Line 400: old env is dropped here
                idx = c.idx;
            }
            // …
        }
    }
    // …
}
```

**The seam:** When `env` goes out of scope (line 400), every binding in the old env **must have its `drop_ref()` called before the HashMap is dropped**. This is the **scope-exit reclamation** point (MEM-3).

### PROPOSED CHANGES (high-level structure)

1. **Wrap AotVal in RC:**

   ```rust
   // Current:
   enum AotVal {
       Core(CoreValue),
       Closure { param, body, env },
       // …
   }
   
   // Proposed:
   enum AotVal {
       Core(RcCell<CoreValue>),  // ← wrapped
       Closure { param, body, env },  // env must also wrap its values
       // …
   }
   ```

2. **Modify lookup / bind / drop logic:**
   - `lookup(env, atom)` now calls `val.clone_ref()` on return (Dup emission equivalent)
   - `Step::Bind(name, val)` wraps bare values in `RcCell::new(val)`
   - Environment cleanup must explicitly call `drop_ref()` on each binding with proper scope_id/sweep_epoch

3. **Thread ReclamationSink through eval_machine signature:**

   ```rust
   // Current (line 375):
   fn eval_machine(
       top: Rc<Anf>,
       top_env: Env,
       prims: &PrimRegistry,
       swap: &dyn SwapEngine,
       fuel: &mut u64,
       max_depth: usize,
       budgets: &mut Budgets,
   ) -> Result<AotVal, EvalError>
   
   // Proposed:
   fn eval_machine(
       top: Rc<Anf>,
       top_env: Env,
       prims: &PrimRegistry,
       swap: &dyn SwapEngine,
       fuel: &mut u64,
       max_depth: usize,
       budgets: &mut Budgets,
       sink: &mut dyn ReclamationSink,      // ← new
       scope_id: ScopeId,                   // ← new
       sweep_epoch: &mut SweepEpoch,        // ← new (mutable for nested scope tracking)
   ) -> Result<AotVal, EvalError>
   ```

4. **Integrate regions at hypha-exit:**
   - Currently missing: the live executor doesn't call `with_region()` or `RegionScope::close()`
   - Future: each hypha's body must be wrapped in a region so ScopeExit records fire at hypha-exit

---

## 3. The Reference RC Evaluator as Correctness Oracle

**File:** `crates/mycelium-mir-passes/src/eval.rs`

**Role:** An **abstract machine** (references + reclamation, not data) that serves as the executable semantics for the borrow-elision analysis. It **computes the same multiset of reclaimed allocations** as the naive full-ownership emission, proving that elision is **semantics-preserving**.

### How it works (eval.rs:156–239)

```rust
pub fn eval(node: &RcNode) -> Result<EvalReport, RcError> {
    let mut m = Machine::new();  // Allocation counter + RC map + reclamation log
    let env = HashMap::new();
    let result = go(node, &env, &mut m)?;
    Ok(EvalReport {
        result,              // Allocation that escapes
        reclaimed: m.reclaimed,  // All reclaimed allocations, in order
    })
}
```

**Machine state:**
- `next: AllocId` (u64 counter)
- `rc: HashMap<AllocId, i64>` (reference count per allocation)
- `reclaimed: Vec<AllocId>` (reclamations in order)

**Key operations:**
- `Const`/`Op`/`Swap` → `alloc()` (fresh allocation with rc=1)
- `Dup` → `dup(a)` (rc += 1)
- `Var` → `dec(a)` (move: rc -= 1; reclaim at 0; error on rc < 0)
- `Borrow` → `assert_live(a)` (read: rc unchanged; error if rc <= 0)
- `MoveUnique` → verify rc==1, then `dec(a)` (Increment 2 soundness check)
- `Drop` / `DropAfter` → `dec(a)` at different points in the term

### Differential harness (eval.rs:250–270)

```rust
pub fn differential(owned_ir, elided_ir) -> bool {
    let owned_report = eval(owned_ir)?;
    let elided_report = eval(elided_ir)?;
    owned_report.reclaimed_sorted() == elided_report.reclaimed_sorted()
}
```

**What it proves:**
- Both emissions reclaim the **same multiset** of allocations (order-independent)
- The elided IR has **no use-after-free** (asserts live on all Borrow nodes)
- **Dup count strictly reduces** (elision removes Dups)
- **Result allocation is identical** (both escape the same value)

### Why this is the oracle for DN-35

1. **The env-machine is a second execution path** (alongside `run_core`, the AOT path). If real reclamation is threaded into `eval_machine`, the observable reclamation record stream **must match the abstract machine's reclamation log**.

2. **It's differential:** comparing owned vs. elided establishes that **no observable semantics change**.

3. **It's machine-verified:** the evaluator catches unsound annotations (`RcError::UnsoundUnique` on MoveUnique where rc≠1 — line 187–195).

4. **For DN-35's design:** the seam must ensure that when `RcCell<T>::drop_ref()` is called in the env-machine, the conditions under which it reclaims (`rc==1`) **match the abstract machine's prediction** from `emit_reclamation_plan()`.

---

## 4. Constraints the Design MUST Honor

### C1: Content-Address Identity vs. In-Place Reuse

**Tension:** RFC-0001 §4.6 defines values by content hash (immutable, addressable by content). But `rc==1` FBIP reuse reuses **storage in-place**, changing the pointer identity.

**Constraint:** Mycelium's **value identity is content-address, not pointer-address**. FBIP reuse is **storage-level**, never surface-visible. The Provenance DAG records the reuse choice (`push_reuse` vs `push_copy` — G2 / never-silent), but the value is **semantically the same** (its content hash doesn't change).

**Implication for the seam:** When `RcCell<T>::drop_ref()` returns `UniqueOwner(T)`, the codegen **may reuse the allocation** (future FBIP threading), but the value's **content address is invariant** — it's `rcplan:<id>` in the audit trail, not the Rust pointer.

### C2: KC-3 — Keep the Trusted Core Small

**Constraint:** The Core IR (`mycelium-core/src/node.rs`) **must remain untouched**. All RC logic lives in separate crates (`mycelium-mir-passes` for static analysis, `mycelium-std-runtime` for runtime cells).

**Implication for the seam:** The AOT env-machine (`mycelium-mlir/src/aot.rs`) is close to the boundary but **not inside the kernel**. Reclamation hooks can thread into `eval_machine` without violating KC-3. However:
- No RC operations should creep into `mycelium-core/lower.rs` (ANF lowering stays pristine)
- The `RcCell` logic stays in `mycelium-std-runtime` (the runtime module, not core)

### C3: G2 — Never-Silent Reclamation

**Constraint:** Every reclamation event **must emit a ReclamationRecord** (RFC-0027 §9). A "silent drop" is a G2 violation.

**Implication for the seam:** When a binding goes out of scope in `eval_machine` or an `RcCell<T>` is dropped, **there is no `Drop` impl that emits a record** (a sink cannot be passed through Drop without violating KC-3). The **caller must explicitly call `drop_ref(sink, scope_id, sweep_epoch, hash)`** at every scope-exit point.

- If reclamation happens without an emitted record, it's a bug.
- If a record is not produced, the audit trail is incomplete — this must fail a test.

### C4: Acyclic + Immutable Values (RC Soundness)

**Constraint:** LR-9 guarantees values are acyclic. LR-8 guarantees they're immutable.

**Implication for the seam:**
- **No cycle collector is needed** — RC is complete (Perceus guarantee)
- **No write barriers** — immutability means no mutation-alias hazards
- **Reference-counting order doesn't matter for safety** — only for observable behaviour (audit trail, FBIP reuse eligibility)

The env-machine's reclamation order can be **weak (concurrent siblings) or strong (total across siblings)** — acyclicity makes both safe (RFC-0027 §7.1, DN-32 §3).

### C5: The Runtime RcCell Fallback Must Always Be Present

**Constraint:** The MEM-4 static analysis (emit_elided) is **additive only**. A bug in the analysis is a **missed optimization**, never unsafety.

**Implication for the seam:** The env-machine **must never skip the runtime RcCell probe**. Even if static analysis says "this is a Borrow" (no Dup), the **machine must still preserve the invariant that refcounts are tracked and decremented correctly**. The soundness of the entire system rests on:

1. `RcCell<T>` always correctly probes rc at drop time
2. MEM-4's analysis only removes the **Dup op**, not the structural RC invariant
3. If the analysis predicts "no Dup needed", the code does not execute `clone_ref()`, but the runtime still does the `drop_ref()` when the binding exits scope

---

## 5. Open Questions for DN-35

These are the concrete design decisions the note must make, **each grounded in a repo fact**.

### Q1: Atomic RC after cross-hypha transfer — Option A or B?

**Grounding:** DN-32 §7 / RFC-0027 §12 names the sub-question explicitly.

**Current fact:**
- `RcCell<T>` is `!Send + !Sync` (from `std::rc::Rc<T>` — rc.rs:94, line 58)
- Cross-hypha transfer rides the affine channel protocol (RFC-0027 §7.3 — no RC across boundaries)

**Decision points for DN-35:**
- **Option A (proposed by DN-33 §8.1):** Sole-ownership move only. Cross-hypha transfer = affine move (Pony-iso / Rust-Box style). `RcCell<T>` stays intra-hypha, `!Send`. Simpler; keeps RC strictly non-atomic intra-hypha.
- **Option B:** Shared values may cross (atomic RC engages). Introduces `Arc<RcCell<T>>` or an atomic variant. Reaches toward Pony ORCA's deferred weighted refcount model. Deferred to R2/xloc.

**For DN-35:** If you choose Option A (recommended for R1), the seam is simpler — `eval_machine` never threads shared values across a hypha boundary. If Option B, the env-machine must handle conversion to atomic RC at the channel boundary.

### Q2: Does the env-machine compute the content hash in-place, or rely on external hash supply?

**Grounding:** rc_plan.rs:102–105 uses synthetic `rcplan:<id>` hashes (abstract machine tracks refs, not content).

**Current fact:**
- AOT env-machine produces values but **does not compute content hashes** (that's a Core-tier responsibility — ContentHash from mycelium-core)
- rc_plan.rs synth_hash() manufactures `rcplan:<id>` because the abstract RC machine has no content data

**Decision for DN-35:**
- **Proposed:** At real reclamation time in `eval_machine`, the environment must **supply the value's real ContentHash** (computed during allocation or stored in RcCell metadata).
- **Alternative:** Audit records use synthetic hashes; real hashing is deferred to a post-processing step (lower confidence in audit trail).

### Q3: Nested scope tracking — one ScopeId or a tree?

**Grounding:** ScopeId is a u64 placeholder (reclamation.rs:52). RegionEpoch is the current nesting tracker (region.rs).

**Current fact:**
- RegionEpoch is a monotonic counter — inner < outer (region.rs docs)
- ScopeId is flat (no hierarchy encoded) — awaits the canonical scope-tree identity from runtime tier

**Decision for DN-35:**
- **Proposed:** Use RegionEpoch as the **scope nesting marker** in the seam. Each `eval_machine` invocation gets a ScopeId (placeholder u64); nested calls increment RegionEpoch and nest the region within the parent.
- **Or:** Introduce a scope-tree identity type (parent: ScopeId, child_epoch: RegionEpoch) so audit records encode the **full nesting context**.

### Q4: When does the evaluator sink attach — at load time or execution time?

**Grounding:** Currently, `run_with_reclamation()` (rc_plan.rs:163) supplies a sink at **execution time**, but it's an *audit trail emitter*, not a **real reclamation driver**.

**Decision for DN-35:**
- **For real reclamation:** The sink must be threaded through `eval_machine` at **execution time** (not load time). Each hypha spawns with a sink reference, which is passed down through all scope-exit paths.
- **Implication:** `run_core_with_effects` (line 196) signature must grow a `sink: &mut dyn ReclamationSink` parameter (and probably `scope_id`, `sweep_epoch` too).

### Q5: What happens to AST-level Rc<Anf> — does it interact with value-level RC?

**Grounding:** Currently, the machine shares AST blocks via `Rc<Anf>` (aot.rs:84, 222, etc). This is **separate from value-level RC**.

**Constraint:** AST sharing is a **codegen detail** (not a value-level concern). The two must not conflate.

**For DN-35:** The seam must **keep them separate**:
- `Rc<Anf>` remains for **code-level sharing** (blocks, continuations)
- `RcCell<CoreValue>` is for **value-level sharing** (the data computed and bound in the environment)
- No value-level RC metadata leaks into AST blocks; no AST sharing affects value reclamation order.

### Q6: Full pipeline integration — does rc_plan.rs run *before* eval_machine or in-parallel?

**Grounding:** Currently, rc_plan.rs runs separately: `emit_reclamation_plan()` (line 120) computes a static audit trail, then `run_core()` runs the value computation independently.

**Design choice for DN-35:**
- **Current (audit-trail only):** Static plan + runtime execution are independent. Plan is observability, not behaviour change.
- **Real reclamation:** The plan could **drive** the reclamation — supply the static predictions (MoveUnique sites, scope-exit epochs) to the machine, and the machine uses them to optimize (skip runtime probes where static analysis is certain).
  - **Implication:** The RC evaluator becomes a **differential test** (static predictions vs. runtime actuals must match).

### Q7: Handling recursion in the seam — Fix/FixGroup RC lifetime

**Grounding:** eval.rs refuses `Fix`/`FixGroup` (line 234–237); rc_plan.rs explicitly returns `RcPlanError::Emit` for out-of-fragment terms (rc_plan.rs:23–27).

**Current fact:** Recursion is a **known gap** — MEM-4 Phase 3 / DN-33 §6 flags it as a deferred increment.

**For DN-35:** The seam must **not silently handle recursion incorrectly**. Options:
- **Option A:** Refuse recursion in `eval_machine` if real reclamation is threaded in (mirror the abstract machine's limitation).
- **Option B:** Sequence RC emission for `Fix`/`FixGroup` before threading into the seam (future phase).
- **Document** the limitation explicitly (never-silent — G2).

---

## 6. Key File:Line Index — Quick Reference

### Design Documents
| File | Section | Lines | Content |
|---|---|---|---|
| `docs/notes/DN-32-Three-Layer-Hybrid-Memory-Architecture.md` | §2 The three layers | 44–109 | L1/L2/L3 architecture; §2.2 is RC details |
| `docs/notes/DN-32-Three-Layer-Hybrid-Memory-Architecture.md` | §6b KC-3 tension | 218–228 | Why L2 static analysis is the hardest leg |
| `docs/notes/DN-32-Three-Layer-Hybrid-Memory-Architecture.md` | §7 Cross-hypha sub-question | 247–278 | Option A vs B boundary (unresolved by design) |
| `docs/notes/DN-33-Layer1-Static-Uniqueness-Analysis.md` | §2 Additive principle | 48–71 | "Soundness required, completeness optional" |
| `docs/notes/DN-33-Layer1-Static-Uniqueness-Analysis.md` | §8 Design Q1–Q7 | – | Decisions the maintainer ratified |
| `docs/rfcs/RFC-0027-Memory-Management-and-Reclamation.md` | §7 RC decision | 217–249 | Mechanism choice; LR-9 as precondition |
| `docs/rfcs/RFC-0027-Memory-Management-and-Reclamation.md` | §8 Guarantee tagging | 283–302 | Honest strength of claims (Proven-modulo, Empirical, Declared) |
| `docs/rfcs/RFC-0027-Memory-Management-and-Reclamation.md` | §9 EXPLAIN record | 305–333 | Field set; never-silent contract |
| `docs/rfcs/RFC-0027-Memory-Management-and-Reclamation.md` | §10.1–§10.3 | 340–404 | RC unifies reclaim + copy/mut; sweep-order derives from scope tree |
| `docs/planning/E12-Memory-Model-Build-Plan.md` | Waves 1–4 | 48–106 | Current completion status; MEM-1/2/3 done, MEM-4 increments 1–2 done |

### Static Analysis (MEM-4)
| File | Lines | Content |
|---|---|---|
| `crates/mycelium-mir-passes/src/rc_ir.rs` | 1–150 | RC-annotated IR; Mode::Owned/Borrowed; Dup/Drop/DropAfter/MoveUnique nodes |
| `crates/mycelium-mir-passes/src/emit.rs` | (full file) | emit_owned, emit_elided, emit_reuse lowerings; Node → RcNode |
| `crates/mycelium-mir-passes/src/eval.rs` | 1–80 | RcError enum; never-silent error types |
| `crates/mycelium-mir-passes/src/eval.rs` | 103–150 | Machine struct; alloc/dup/dec/assert_live |
| `crates/mycelium-mir-passes/src/eval.rs` | 152–239 | eval() function; go() matcher; differential oracle |
| `crates/mycelium-mir-passes/src/balance.rs` | (full file) | Structural invariant check: 1 + dups == uses + drops |
| `crates/mycelium-mir-passes/src/corpus.rs` | (full file) | Measurement harness; corpus.measure_mem4() counts reuse sites |

### Runtime RC Cell (MEM-2)
| File | Lines | Content |
|---|---|---|
| `crates/mycelium-std-runtime/src/rc.rs` | 1–68 | Module doc; design decisions; guarantee tags per-op |
| `crates/mycelium-std-runtime/src/rc.rs` | 102–145 | RcCell<T> struct; new, clone_ref, refcount methods |
| `crates/mycelium-std-runtime/src/rc.rs` | 171–225 | drop_ref() — the **live RcZero trigger**; probe + emit |
| `crates/mycelium-std-runtime/src/rc.rs` | 56–90 | RcProbe enum; UniqueOwner vs Shared outcome |

### Reclamation Record (MEM-1)
| File | Lines | Content |
|---|---|---|
| `crates/mycelium-std-runtime/src/reclamation.rs` | 1–37 | Module doc; placement rationale; trigger wiring FLAGs |
| `crates/mycelium-std-runtime/src/reclamation.rs` | 51–76 | ScopeId, ChannelId, SweepEpoch types (u64 placeholders) |
| `crates/mycelium-std-runtime/src/reclamation.rs` | 79–112 | ReclamationTrigger enum; exhaustive; G2-enforced |
| `crates/mycelium-std-runtime/src/reclamation.rs` | 116–200 | ReclamationRecord struct; five-field set; constructor API |
| `crates/mycelium-std-runtime/src/reclamation.rs` | 220–260 | ReclamationSink trait; never-silent contract |

### Region Batching (MEM-3)
| File | Lines | Content |
|---|---|---|
| `crates/mycelium-std-runtime/src/region.rs` | 1–80 | Module doc; batching model; monotonic epoch + LIFO |
| `crates/mycelium-std-runtime/src/region.rs` | (rest) | Region struct; defer/close; ClosedRegion summary |
| `crates/mycelium-std-runtime/src/scope_region.rs` | 1–65 | Module doc; two entry points; with_region vs RegionScope |
| `crates/mycelium-std-runtime/src/scope_region.rs` | 128–139 | **with_region()** — closure-form scope-exit entry; guarantee |
| `crates/mycelium-std-runtime/src/scope_region.rs` | 182–197 | **RegionScope** — explicit-close guard; never-silent design |

### AOT Env-Machine (The Seam)
| File | Lines | Content |
|---|---|---|
| `crates/mycelium-mlir/src/aot.rs` | 1–32 | Module doc; full v0 calculus; stack-robust trampoline; forbid(unsafe) |
| `crates/mycelium-mlir/src/aot.rs` | 57–66 | resolve_max_depth, default_depth_budget (budget resolution logic) |
| `crates/mycelium-mlir/src/aot.rs` | 68–104 | **AotVal enum** — the value representation (Core/Closure/Fix/FixGroup) |
| `crates/mycelium-mlir/src/aot.rs` | 106 | **Env = HashMap<Atom, AotVal>** |
| `crates/mycelium-mlir/src/aot.rs` | 147–153 | **run_core()** entry point |
| `crates/mycelium-mlir/src/aot.rs` | 171–181 | **run_core_with_budget()** |
| `crates/mycelium-mlir/src/aot.rs` | 196–208 | **run_core_with_effects()** — where budgets thread in |
| `crates/mycelium-mlir/src/aot.rs` | 375–450+ | **eval_machine()** — the trampoline loop; **THE SEAM** |
| `crates/mycelium-mlir/src/aot.rs` | 390–412 | Loop body: idx check, block-exit, stack-pop, resume (scope-exit reclamation point) |
| `crates/mycelium-mlir/src/aot.rs` | 415–449 | **Step evaluation** — Const/Alias/Op/Swap binding (RC hook points A1–A5) |
| `crates/mycelium-mlir/src/aot.rs` | 220–239 | select_arm() match descent (no RC needed; data selection only) |

### AOT Audit-Trail Bridge (MEM-4·AOT)
| File | Lines | Content |
|---|---|---|
| `crates/mycelium-mlir/src/rc_plan.rs` | 1–32 | Module doc; honest scope (env-machine Rust-manages values) |
| `crates/mycelium-mlir/src/rc_plan.rs` | 42–53 | AOT_TOP_SCOPE, AOT_SWEEP_EPOCH (placeholders) |
| `crates/mycelium-mlir/src/rc_plan.rs` | 55–79 | RcPlanError enum; Emit vs Eval failure distinction (G2) |
| `crates/mycelium-mlir/src/rc_plan.rs` | 95–105 | synth_hash() — synthetic rcplan:<id> content address |
| `crates/mycelium-mlir/src/rc_plan.rs` | 120–137 | **emit_reclamation_plan()** — runs emit_elided → eval → emit records |
| `crates/mycelium-mlir/src/rc_plan.rs` | 140–149 | **RcRun struct** — value + Optional reclaimed count |
| `crates/mycelium-mlir/src/rc_plan.rs` | 163–172 | **run_with_reclamation()** — execution + additive audit trail |

### Core IR (Untouched — KC-3)
| File | Lines | Content |
|---|---|---|
| `crates/mycelium-core/src/node.rs` | (full file) | Core IR Node grammar — **RC passes must NOT touch this** |
| `crates/mycelium-core/src/lower.rs` | (full file) | ANF lowering — **RC annotations applied AFTER, not here** |
| `crates/mycelium-core/src/id.rs` | (full file) | ContentHash, VarId, CtorRef — value identity |

---

## Summary — The Ground Truth for DN-35

### What's Built
1. **L1 (affine primary):** Core IR + move semantics; static analysis deferred
2. **L2 (optimized RC):** RcCell runtime probe (live RcZero trigger); static emit_elided/emit_reuse with differential verification; MEM-4·AOT audit-trail bridge
3. **L3 (region batching):** Region + RegionScope; batched ScopeExit records at scope-exit (observer ready, executor integration deferred)

### What's Missing
1. **Real reclamation threading:** The env-machine still uses Rust's automatic `Drop` for values. The seam is clear (lines 390–449 of aot.rs), but no RcCell wrapping or drop_ref() calls yet.
2. **Executor integration:** The live Scope/Runtime does not call `with_region()` or `RegionScope::close()` at hypha-exit. The region machinery is standalone.
3. **Cross-hypha atomic RC:** Option A (sole-move-only) vs Option B (shared-crosses-atomic) unresolved; affects future R2 xloc wiring.
4. **Recursion RC:** Fix/FixGroup are explicitly out-of-scope in the static analysis; MEM-4 Phase 3 deferred.

### The Exact Seam
**File:** `crates/mycelium-mlir/src/aot.rs`  
**Function:** `eval_machine()` (lines 375–450+)  
**Points:**
- **A (value wrapping):** Step::Bind result → `env.insert()` (line ~397): wrap in RcCell, clone_ref on read (Alias)
- **B (scope-exit):** Block-exit when idx ≥ bindings.len() (line ~390) or environment replaced on Resume (line 400): call drop_ref() on each binding in old env
- **C (sink threading):** Add `sink: &mut dyn ReclamationSink`, `scope_id: ScopeId`, `sweep_epoch: &mut SweepEpoch` parameters to eval_machine and propagate down

### The Oracle
**File:** `crates/mycelium-mir-passes/src/eval.rs`  
**Function:** `eval()` (lines 156–164) + differential() harness  
**Use:** The abstract RC machine predicts which allocations will be reclaimed. The seam's real reclamation must **emit one ReclamationRecord per reclamation, at the same order/time as the evaluator predicts** (mod scheduling concurrency).

---
