# A — External Research: Threading Real RC + Region Reclamation (with In-Place Reuse) into a Big-Step Trampolined Env-Machine

**Scope.** External (web) prior-art research backing a Mycelium design note. Target machine: big-step,
**trampolined** environment machine with an **explicit heap-allocated control stack** (O(1) host stack)
over **A-normal-form (ANF)** IR, plus a separate **RC-annotated IR** with Perceus-style borrow elision and
`rc==1` reuse annotations (modelled today as `RcNode`). Mycelium values are **immutable, acyclic,
content-addressed**.

**Method.** Primary sources were fetched and their PDFs text-extracted locally (pdfminer); the exact
algorithm rules, operational-semantics rules, and theorem statements below are quoted/paraphrased from the
extracted text, not from secondary summaries. Each major claim is adversarially checked against the actual
mechanism (e.g. *what the soundness theorem actually requires*, *what was proven vs. left as future work*).

**Verdict tags** use Mycelium's lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`. "Proven" here means a
published theorem with stated (and, where noted, mechanized) side-conditions; "Empirical" means
benchmark/observation; "Declared" means asserted by authors without a checked basis.

---

## 1. Prior-art survey — exact mechanisms + citations

### 1.1 Perceus (Koka) — precise RC + reuse + specialization [Reinking, Xie, de Moura, Leijen, PLDI'21]

Perceus = *PrEcise Reference counting with rEUse and Specialization*. It takes a functional core with
**explicit control flow**, runs **precise liveness**, and emits `dup`/`drop` so the program is
**garbage-free** (only reachable references retained). Pipeline (paper §2):

1. **dup/drop insertion (§2.2).** Each owned variable is consumed exactly once; `dup x` = `incref`,
   `drop x` = the conditional below. A `match` branch `dup`s its newly-bound pattern fields and `drop`s the
   scrutinee.
2. **Drop specialization (§2.3).** Inline the generic
   `drop(x) { if is-unique(x) then { drop children; free(x) } else decref(x) }`
   at a *known constructor*, then **push `dup` down into branches** and **fuse** `dup`/`drop` pairs. After
   this, in the hot path (all nodes `rc==1`) the `is-unique` test always succeeds and frees with **zero**
   residual RC traffic.
3. **Reuse analysis (§2.4) — the core mechanism.** *Per `match` branch*, pair each matched pattern with an
   allocated constructor **of the same size** in that branch (in `map`, `xs:Cons` pairs with the result
   `Cons`). When paired **and the matched object is not live past the match**, rewrite `drop(xs)` into a
   **`drop-reuse(xs)`** that returns a **reuse token** `ru`, attached to the paired allocation as `Cons@ru`:

   ```
   fun drop-reuse(x) { if is-unique(x) then { drop children of x; &x } else { decref(x); NULL } }
   // Cons@ru(...) :  if ru!=NULL then in-place-write the fields of *ru  else malloc a fresh Cons
   ```

   So the **exact "match a drop to a downstream same-shape allocation" rule** is: *(a) same constructor
   size, (b) same branch, (c) scrutinee dead after the match → emit `drop-reuse`, thread the token to the
   allocation, and at runtime branch on `is-unique`.* The token is **NULL on the shared path** (fresh
   alloc) and the cell address on the unique path (in-place overwrite). This is the FBIP ("functional but
   in-place") enabler.
4. **Reuse specialization (§2.5).** If the reused constructor keeps **≥1 field unchanged**, specialize so
   only the changed fields are written (relevant to red-black-tree rebalancing). Skipped when all fields
   are reassigned (e.g. `map`).

**Soundness — exactly what is proven (Proven, with side-conditions):** Perceus is formalized in a **linear
resource calculus λ₁** with an explicit RC heap. The judgement `Δ | Γ ⊢ e ⇝ e′` separates a **borrowed**
environment `Δ` (used, not consumed — `dup` may read it, `drop` may not) from an **owned** linear
environment `Γ` (each member multiplicity exactly 1). Theorems:

- **Thm 1 (soundness):** the RC heap semantics agrees with the standard semantics.
- **Thm 2 (leaves no garbage):** every intermediate heap entry is **reachable** (Def. 1). **Adversarial
  check:** the paper states explicitly that **to capture the essence of precise RC, λ₁ does not model
  mutable references**, and "*mutable references are indeed the only source of cycles*"; the generalization
  with mutation would weaken Thm 2 to "reachable **or part of a cycle**." → *The garbage-free guarantee is
  conditional on acyclicity, which it gets from the absence of mutable references.*
- **Thm 3 (translation sound)** + **Thm 4 (Perceus is precise and garbage-free):** every non-`dup`/`drop`
  intermediate state is garbage-free.

**Cycles (§2.7.4) — adversarial:** RC "cannot release cyclic data structures." Koka's mitigation is
**static**: inductive/coinductive immutable types "can be shown" to be acyclic, and only mutable references
can form cycles; programmer must break those by clearing a cell (same as Swift). **This is the single most
important external finding for Mycelium:** with **immutable + acyclic** values, RC is **complete** — the one
fundamental limitation of RC does not apply, and Thm 2 holds in its strong (non-cycle) form.

**Concurrency cost (§2.7, performance):** Koka tags each object **thread-shared or not** (an internal
`tshare` recursively marks an object + children; objects start unshared and can never be un-shared). The RC
field is **sign-bit encoded**: non-negative ⇒ unshared (cheap non-atomic `inc`/`dec`), negative ⇒
thread-shared ⇒ atomic ops. The fast path uses a `memory_order_relaxed` read of the sign bit to *avoid*
atomics entirely. → Directly relevant to Mycelium's cross-hypha story (§5).

### 1.2 Reference Counting with Frame-Limited Reuse [Lorenzen & Leijen, OOPSLA'22 / MSR-TR-2021-30]

Re-engineers Perceus reuse to be **simpler and more robust**:

- **Drop-guided reuse (§2.3) — the improved algorithm.** *Do reuse analysis **after** Perceus dup/drop
  insertion, not before.* Because Perceus drops are already **precise** (an object is live until immediately
  before its `drop`), each `drop` is *exactly* the point a cell becomes free. So: track the **known size** of
  each variable (updated at each branch pattern); when a `drop(x)` is reached, if a **later allocation of the
  same size** exists on that path, rewrite `drop` → **`dropru`** (returns a reuse token) and the allocation
  → `ctor@r`. Tokens are provably **used 0 or 1 times, never captured under a lambda or passed as an
  argument**, so a per-branch check inserts `free(r)` where a token is unused. This is strictly cleaner than
  Perceus's "algorithm D" upfront pairing and is **robust to small program transformations**.
- **Frame-limited reuse (§2.6) — the bound that matters for a trampolined machine.** Drop-guided reuse is
  *not* garbage-free and *not* unconditionally "safe-for-space": between `dropru` and the constructor that
  consumes the token there can be a **recursive call** (e.g. in `map`, the token `r` is live across
  `map(xx,f)`). So held memory is bounded by **a constant × the current number of stack frames** — they
  call this **frame-limited** (formalized in §3, *proven* frame-limited in §3.4). **Adversarial check &
  Mycelium consequence:** in a **trampolined env-machine the "stack" is the explicit heap-allocated control
  stack**, so a live reuse token pins one cell **per suspended frame on the heap stack**. The frame-limited
  bound therefore translates to *peak reuse-pinned memory ≤ const × control-stack depth* — an explicit,
  auditable quantity in our machine, not a hidden host-stack property. They note Ullrich–de Moura's
  borrow-inference and algorithm D are **not** frame-limited (can pin arbitrary memory).

### 1.3 Counting Immutable Beans (Lean 4) [Ullrich & de Moura, IFL'19]

The origin of explicit-IR RC for pure FP and of `reset`/`reuse`. Compiles `λpure → λRC`:

- **`inc`/`dec` IR instructions.** A function taking an **owned** argument must **consume** it (pass it on,
  or `dec` it); a **borrowed** argument must not. `inc x` creates a token, `dec x` consumes one.
- **`reset`/`reuse` (§4) — exact operational semantics (Fig. 2).** `let y = reset x` then
  `let z = reuse y in ctor_i w`:
  - **Reset-Uniq** (`x` rc==1): eagerly `dec`s the children, replaces fields with a sentinel `□`, returns
    the **location** `l` of the now-invalid cell.
  - **Reset-Shared** (`x` rc≠1 / shared): behaves like `dec x` and returns the sentinel `□`.
  - **Reuse-Uniq** (`reuse l …`): writes the new constructor into cell `l` in place (`reuse x in ctor_i y`
    asserts the old cell's size is compatible).
  - **Reuse-Shared** (`reuse □ …`): falls back to a fresh `ctor_i y` allocation.
  This is the **runtime safety valve**: the `reset`/`reuse` *pair* cannot be naively fused, precisely
  because intermediate code may change the refcount — the token carries the unique-vs-shared decision made
  at `reset` time. Insertion is by a transform `δreuse` that, in each `case` arm, runs a **dead-variable
  search `D`** from the match variable and **substitutes `S`** a same-size constructor with a `reuse`.
- **Borrow inference (§5) — heuristic, *not required for correctness*.** `collectO` (Fig. 4) over-collects
  which parameters **must be owned** (used in a `reset`, or passed to an owned-parameter position),
  computed to a **fixpoint** over (mutually) recursive functions; everything else can be **borrowed** to
  elide `inc`/`dec`. The paper is explicit that this is a heuristic; correctness holds for any β/δ.
- **Adversarial check on "Proven":** at publication the authors state they are *"working on a formal
  correctness proof of the compiler … using a type system based on intuitionistic linear logic."* → **Beans'
  guarantee was Empirical/Declared at publication**, not a published proof. The *proof* for this design
  family is what Perceus (1.1) and FP² (1.6) later supplied. **Tag accordingly.**

### 1.4 Swift ARC + uniqueness / COW

`isKnownUniquelyReferenced(_:)` (built on the compiler builtin `Builtin.isUnique`) is the **runtime
uniqueness check** behind copy-on-write: a value-type wrapper holds a reference to a class-backed buffer; a
mutation first checks uniqueness and **deep-copies only if rc ≥ 2**, else mutates in place. This is the same
unique-vs-shared dynamic gate as Beans `reset`, exposed as a **library API** rather than a compiler IR op.
**Adversarial note:** Swift ARC is **atomic by default** (ObjC interop, no thread-sharing partition), so its
per-op cost is higher than Koka's sign-bit scheme; Swift relies on COW + ARC-optimizer elision rather than
whole-program precise RC. Swift also leaves **cycle handling to the programmer** (`weak`/`unowned`) — the
same posture Perceus cites.

### 1.5 OCaml 5 multicore (immutability angle)

Relevant only as a data point on *immutability lowering memory-barrier cost*: for **immutable** objects the
initializing writes need **no barriers** and reads need **no barriers** in the multicore memory model; only
mutable fields require the read/write-barrier machinery. → Confirms the general principle that immutability
*removes* synchronization obligations — the same reason Mycelium can keep cross-hypha RC cheap. OCaml itself
uses tracing GC, not RC, so it is **not** a reclamation-design model here, only an immutability/concurrency
data point.

### 1.6 FP² — Fully in-Place Functional Programming [Lorenzen, Leijen, Swierstra, ICFP'23]

The **static-guarantee** capstone, and the most important model for "trusted core small + Proven." A
**linear FIP calculus λfip** identifies *exactly when* a pure function can run with **no (de)allocation and
constant stack space**, *provided its arguments are not shared*.

- **Mechanism — reuse credits `◇k` (diamonds).** A `match!` (destructive match) on a constructor of arity
  `k` yields its children **plus a linear "reuse credit" `◇k`** (after Hofmann's *space credits*). A
  same-size allocation **consumes** a `◇k`. Reuse credits are **linear and cannot be split or merged**. So
  in-place reuse is expressed as a **resource in the type system** — the FIP check is a *syntactic linearity
  check*, compiled down to a Perceus reuse token `ru` at runtime.
- **Proven (mechanized), with side-conditions.** The extracted theorems:
  - **Thm 1 / Thm 7 (store/heap soundness for well-formed FIP programs).**
  - **Thm 2 (a FIP program reduces in-place):** for any reduction `S | e ↦→* S′ | e′`, **`|S| = |S′|`** —
    the store size never changes (no alloc, no free).
  - **Thm 3 (FBIP program can only deallocate)** — the FBIP relaxation.
  - **Thm 4 / Thm 8 (a FIP program uses constant stack space).**
  - **Thm 5 / Thm 9 (TRMC / tail-recursion-modulo-cons / TRMReC transformations sound).**
  - **Thm 6 (heap semantics sound for well-formed Perceus programs)** — "the dynamic reference count is
    always correct," noted as backed by a **mechanized formalization**.
  - **Side-condition that makes them true:** the owned environment `Γ` is used **linearly** (borrowing only
    in `let`), arguments are **not shared**, and there are **no reuse credits in a borrowed set**. FIP is a
    **syntactic** property (no full linear *type* system needed) — which is exactly the "separately checkable
    pass" property Mycelium wants.

---

## 2. Where reclamation should live in a trampolined env-machine — recommendation

**Design axis:** (a) compile-time-inserted `dup`/`drop`/`reset`/`reuse` ops the machine **interprets**
(today's `RcNode`), vs (b) a **runtime RC cell** the host manages, vs (c) **hybrid**.

**Recommendation: hybrid, but with a sharp split — *static decisions, dynamic verification*.**

1. **Reclamation events are first-class IR ops, inserted by a separate pass, interpreted by the core.** Keep
   the Beans/Perceus model: `dup`, `drop`, `drop-reuse`/`reset`, `reuse@token`, and a region `ScopeExit` op
   are **explicit nodes** in the RC-annotated IR (you already have `RcNode`). The **trusted evaluator** does
   only the minimal dynamic thing — `incref`, `decref`, the `is-unique` test, the in-place-write-or-malloc
   branch — and treats *where* those ops live as given by an **untrusted, separately-verifiable pass**. This
   is precisely Beans' λpure→λRC and Perceus' `⇝`: the optimization is outside the audited core. (Aligns
   with KC-3 "small auditable kernel" and the "no black boxes" rule — every reclamation event is a reified,
   `EXPLAIN`-able node.)
2. **A runtime RC cell still exists** (one count per heap value), because the **unique-vs-shared decision is
   inherently dynamic** (Beans Reset-Uniq vs Reset-Shared; Swift `isUnique`). You cannot make in-place reuse
   sound by static analysis alone unless you adopt the *full* FIP linearity discipline (§6 option B). The
   count is the dynamic witness the interpreted ops consult.
3. **Why not pure-runtime (option b)?** Pure runtime RC (Swift-style, every `inc`/`dec` at every binding)
   pays the worst per-op cost and discards the precise-liveness garbage-freeness that explicit-control-flow
   IR buys (Perceus Thm 4). In a **big-step** evaluator the host would otherwise have to infer liveness from
   the host call structure — exactly the information the trampoline *erased* by reifying control on the heap.
4. **Why the trampoline makes the IR-op approach not just possible but necessary.** With an explicit
   heap-allocated control stack, **host-stack unwinding no longer runs your `drop`s** (the Rust pattern Beans
   contrasts against, where C++/Rust `Drop` rides stack unwinding). So drops **must** be explicit IR ops the
   trampoline schedules — there is no host frame to hang a destructor on. This is a *forcing function* toward
   option (a)/(c), and it is an advantage: reclamation becomes **explicitly scheduled and auditable**, never
   an implicit unwinding side-effect.
5. **Threading reuse tokens through the heap control stack.** A reuse token is a value (`NULL`/sentinel or a
   cell address) — store it **in the control-stack frame / continuation record**, not the host stack. Then
   §1.2's **frame-limited** bound becomes a *machine-visible invariant*: peak reuse-pinned memory ≤ const ×
   heap-control-stack depth, countable and assertable (good for a property test / `EXPLAIN`).

**Net:** *compile-time-inserted ops (a) for scheduling, a runtime RC cell (b) as the dynamic witness, with
the dynamic surface minimized to `incref`/`decref`/`is-unique`/`reuse-or-alloc`* — option **(c) hybrid**,
weighted toward static scheduling.

---

## 3. Reuse-token threading algorithm — concrete, for the ANF/RC-IR machine

Adopt **drop-guided reuse** (§1.2) over upfront pairing (§1.1) — it is simpler, robust, and frame-limited.
Pass order on the RC-annotated IR (after ANF, after Perceus-style dup/drop insertion):

```
INPUT: ANF IR with precise dup/drop already inserted (garbage-free).
STATE: sizeOf : Var -> KnownSize     // refined at each match arm to the matched constructor's size/shape
       tokens : set of live reuse tokens

For each instruction in program order within a function body:
  on `match x { C_i(f...) -> arm_i }`:
      within arm_i, set sizeOf(x') for each bound field; record sizeOf(x) = size(C_i)
  on `drop(x)` where sizeOf(x) is known:
      if ∃ a later same-path allocation `alloc C_j(...)` with size(C_j) == sizeOf(x)
         AND x is dead after this point (Perceus already guarantees liveness-precision):
            rewrite  drop(x)            ==>  let r = drop-reuse(x)    // r : ReuseToken
            rewrite  alloc C_j(...)     ==>  C_j@r(...)               // token-consuming alloc
            tokens += r
      else: leave drop(x) as-is
  at each branch leaf:
      for each r in tokens not consumed on this branch:  insert free(r)   // 0-or-1 use property
```

**Runtime semantics the trusted core interprets** (Beans Fig. 2 / Perceus §2.4):

```
drop-reuse(x):  if is-unique(x) then { decref children of x; return &x }   // token = cell address
                else            { decref(x);                return SENTINEL }
C_j@r(args):    if r != SENTINEL then { write args into *r; return r }      // in-place (FBIP)
                else                 { return alloc C_j(args) }             // fresh
```

**Soundness side-conditions to enforce (so the pass is *Proven*, not *Declared*):**
- **(S1) Same size/shape.** Reuse only when `size(C_j) == sizeOf(x)` (Beans "asserts size compatible").
- **(S2) Liveness-precision.** `x` must be dead after the `drop` — inherited from garbage-free dup/drop
  insertion (Perceus Thm 4). Do **not** reuse from a still-live or borrowed value.
- **(S3) Token discipline.** A token is used **0 or 1 times**, **never captured under a lambda, never passed
  as an argument** (Lorenzen–Leijen) — this is what lets the local per-branch `free(r)` be correct and keeps
  the analysis intraprocedural.
- **(S4) Dynamic gate is mandatory.** The `is-unique` test stays at runtime; static analysis chooses *where*
  reuse is *possible*, the RC cell decides whether it is *taken* (Beans Reset-Shared fallback). Never elide
  the gate unless the value is FIP-linear (§6).
- **(S5) Content-address guard.** See §4 — reuse is only sound for a cell whose **content-hash identity is
  not depended upon by a live alias**; with rc==1 + acyclic this is automatic.

---

## 4. Content-address vs in-place reuse — resolution

**The tension (Mycelium-specific, *harder* than Koka/Lean):** in-place reuse **mutates a cell's bytes**;
Mycelium value identity = **hash of content**. If anything still holds the **old** content-hash identity of
that cell, in-place mutation silently changes what that identity points to — a correctness break that Koka
and Lean never face (their values have *pointer* identity, not *content* identity).

**Resolution — three layered observations from the survey:**

1. **rc==1 ⇒ no live alias ⇒ no observer of the old identity.** The reuse gate fires **only** when the cell
   is **uniquely owned** (Beans Reset-Uniq, Perceus `is-unique`, Swift `isUnique`, Clojure transients). If
   rc==1, **no other reference exists**, so no live computation can be holding the old content-hash to
   observe the change. The mutation is **unobservable** — the value is *consumed* and a new value is *born*
   in the same memory. This is exactly Clojure's **transient** discipline: a persistent structure is
   *invalidated* (`transient!`), mutated in place while uniquely owned, then *re-frozen* (`persistent!`);
   the old handle is contractually dead. The reset→reuse pair **is** transient→persistent at cell
   granularity. → **This is the principled answer: content-address identity is preserved because identity is
   a property of *live* values, and reuse only touches *dead* ones.**
2. **When must reuse copy-on-identity (i.e. *not* reuse in place)?** Whenever the **old identity could still
   be observed**:
   - **rc ≥ 2** (shared): always fall back to fresh alloc (Reset-Shared). Already handled by (S4).
   - **The cell is interned in a content-address table / hash-cons map** and that table holds a reference
     (so effective rc ≥ 2 via the table). → The intern table **must** count as a reference, or reuse will
     dangle the table entry. **Recommendation:** treat the global hash-cons table as a **weak** map (entries
     don't pin), and on `drop-reuse` of a uniquely-owned **interned** cell, **evict the table entry first**
     (its hash key is about to become stale), then reuse. If you cannot evict atomically, **copy-on-reuse**
     (allocate fresh) — never mutate a cell the intern table still indexes by its *old* hash.
   - **Hash memoization on the cell.** If a value caches its own content-hash inline, in-place reuse must
     **invalidate/recompute** the cached hash as part of the `reuse` write (it's writing new content).
3. **Persistent-structure precedent (HAMT / Clojure / Bagwell).** Persistent structures get in-place-like
   performance via **path copying + structural sharing**: an update copies only the root→leaf path and
   shares the rest. Shared (rc≥2) sub-nodes are **never** mutated; only the freshly-copied spine is. This is
   the *same invariant* as RC reuse — "mutate only what you uniquely own" — and is the fallback when reuse
   can't fire: **copy the path, share the rest.** Unison's runtime is the cleanest content-addressed
   precedent: source is hashed (SHA3-512, names→positional), but the **runtime keeps a separate
   compiled/closure-converted representation** keyed by hash (`Map Reference Int`), and **execution operates
   on the runtime representation, not the hash**. → **General principle: decouple the hashed identity from
   the runtime cell.** Reuse mutates the *runtime cell*; the *content-hash identity* lives in a separate
   (weak, evict-on-reuse) index. When a value's hash is actually demanded, it is computed/looked-up against
   the index, which the reuse path keeps consistent.

**Concrete rule for Mycelium:** *Reuse a cell in place iff (rc==1 ∧ same-shape ∧ dead-after) AND (the cell
is not currently pinned in the content-address index, OR the index entry is evicted as part of the reuse).*
Otherwise **copy-on-reuse**. This makes the content-address property a **checked side-condition (S5)** of the
reuse pass, not a hope.

---

## 5. Coupling RC (`RcZero` trigger) with region/scope batched reclamation (`ScopeExit` trigger)

**Goal:** combine per-value RC reclamation with **batched** scope/region reclamation **without double-free
or missed-free**.

**Prior art — RC + regions is well-trodden:**
- **Gay & Aiken, "Memory Management with Explicit Regions" (PLDI'98) + RC.** Regions carry a
  **reference count of pointers into them**; a region is freed when its count hits zero. Key efficiency
  trick: **references *internal* to a region don't update counts** — only **cross-region** pointers are
  counted. Explicit region deletion is allowed but **not statically guaranteed to succeed** (it's
  dynamically gated by the count). → The canonical "RC-gates-region-free" design.
- **Terauchi & Aiken, "Memory Management with Use-Counted Regions."** Use-counts on regions, same family.
- **MLKit (Tofte–Talpin region inference) + tracing GC.** Region inference statically inserts
  alloc/dealloc; combined with GC as a backstop for "very dynamic" lifetimes. → Establishes regions as a
  **complementary** static layer over a precise dynamic layer, not a replacement.

**Recommendation for Mycelium — two-trigger, single-owner, count-respecting:**

1. **Stratify ownership: every heap value belongs to exactly one region/scope at a time.** A value's *region
   membership* is the batch unit; its *RC* is the fine-grained unit.
2. **`RcZero` is the eager trigger; `ScopeExit` is the batch trigger — and they must not both free the same
   cell.** Make the rule: **`ScopeExit` frees only cells still at rc>0 that are region-internal and dead by
   scope-end; `RcZero` frees a cell the instant its count hits zero.** A cell freed by `RcZero` is **removed
   from its region's free-list** at that moment, so `ScopeExit` cannot re-free it. Conversely, a cell still
   rc>0 at `ScopeExit` only gets batch-freed if **no cross-region reference escapes** — exactly Gay–Aiken's
   "count pointers into the region" check. **The region count is the missed-free guard; the per-cell rc + free-list
   removal is the double-free guard.**
3. **Cross-region/cross-scope references are the only ones that need a count.** Internal references can be
   borrow-elided (Beans borrow inference §1.3): a value used only within its owning scope never touches the
   RC at scope boundaries. This is the Gay–Aiken "internal references are free" optimization restated for
   values.
4. **Escape analysis decides region vs RC.** A value that **never escapes** its scope ⇒ region-batch-freed at
   `ScopeExit` with **no per-op RC at all** (the FIP/region win). A value that **may escape** ⇒ carries a
   real RC and is reclaimed by whichever trigger hits zero/exit first, with free-list removal preventing the
   other from acting. **Never-silent (G2):** an escape that the analysis cannot prove absent → conservative
   *RC-managed*, flagged, never silently region-freed.
5. **Ordering invariant (the actual double-free/missed-free proof obligation):** at `ScopeExit`, process in
   the order *(i) run `drop` for each scope-owned root → (ii) cascade `RcZero` frees → (iii) batch-free the
   region arena for whatever remains region-internal and now unreferenced.* Cells that escaped (rc>0 via a
   cross-region ref) are **promoted** to the parent region, never freed. This is the one place a mechanized
   invariant ("a cell is freed by exactly one of {RcZero, ScopeExit}, and only when unreachable") should be
   stated and property-tested.

**Adversarial caveat:** RC+region coupling has **no off-the-shelf published soundness theorem** as tight as
Perceus' single-mechanism Thm 2. Gay–Aiken give a *design* and safety argument, not a mechanized proof of the
*combined* eager+batch protocol. → Tag the coupled protocol **Empirical/Declared** until Mycelium states and
checks its own "freed by exactly one trigger" invariant (see §7).

---

## 6. Keeping the trusted core small — patterns

The literature gives two distinct strategies; Mycelium should use **A as the floor and B as the ceiling.**

**(A) Reclamation as a separate, untrusted, re-checkable pass (Beans / Perceus posture).** The trusted core
interprets a *fixed, tiny* set of ops (`inc`, `dec`, `is-unique`, `reuse-or-alloc`, `scope-exit-free`); the
*placement* of those ops is produced by an **untrusted compiler pass** (`δreuse`, the `⇝` translation). Two
properties make this safe even if the pass is buggy:
- **Dynamic safety valve.** Beans Reset-Shared / Perceus NULL-token / Swift `isUnique`: a wrong static
  "reuse here" decision **degrades to a correct fresh allocation at runtime**, it does not corrupt memory.
  So a buggy reuse pass costs *performance*, not *soundness* — the worst case is "no reuse," never
  "use-after-free." **This is the key small-trusted-core lever:** the audited core's invariant is just "rc
  correct + reuse only when `is-unique`," and that invariant is **local** and **machine-checkable per op**.
- **Independent re-checkability.** Because the ops are explicit IR nodes, a *verifier* can re-derive that
  every value is consumed exactly once (the linear `Δ|Γ` discipline) **without trusting the pass that
  inserted them** — analogous to proof-carrying code / a typechecked IR.

**(B) Reclamation guaranteed by a separately-checkable *type/linearity* discipline (FP²/λfip posture).** For
the subset of code that is **FIP-linear**, the reuse is *statically guaranteed* (reuse credits `◇k` consumed
linearly; Thm 2 `|S|=|S′|`), so the runtime `is-unique` gate can in principle be **elided** — the strongest
"core does almost nothing" result. The check is **syntactic** (no full linear type system — FP² stresses
this), i.e. a **separate pass that *certifies* rather than *transforms*.** → Mycelium's `fast` (default) mode
uses (A) with the dynamic gate retained; a `certified`/FIP-annotated mode uses (B) to *prove* no-alloc /
constant-stack and drop the gate (ADR-032 `Proven` tier).

**Pattern summary:** *the audited interpreter knows only how to inc/dec/test-unique/reuse-or-alloc; a swap
between "dynamically-gated reuse" (A) and "statically-certified reuse" (B) is itself a reified, EXPLAIN-able
choice, never silent.*

---

## 7. Correctness / guarantee posture (Proven vs Empirical) for Mycelium

| Claim | Source basis | Mycelium tag | Checked side-conditions |
|---|---|---|---|
| Precise RC leaves no garbage (every heap cell reachable) | Perceus **Thm 2**, mechanized-style proof in λ₁ | **Proven** | **no mutable references ⇒ acyclic**; explicit control flow; linear `Δ\|Γ` consumption. Mycelium **satisfies the acyclicity premise by construction** (immutable+acyclic). |
| RC is *complete* for Mycelium values (no leak from cycles) | Perceus §2.7.4 ("inductive/coinductive immutable types … never cyclic"); only mutable refs make cycles | **Proven** *for the acyclic fragment* | values are immutable & acyclic — the premise is a Mycelium invariant, must be enforced/maintained. |
| In-place reuse is sound (no use-after-free) | Beans Reset-Uniq/Reset-Shared dynamic gate; Perceus `is-unique` | **Proven** (given the gate) — but Beans' *compiler* proof was **future work at publication** | (S1) same size, (S2) dead-after, (S4) runtime `is-unique` gate retained, (S3) 0/1-use token. |
| FIP program does **no (de)allocation**, **constant stack** | FP² **Thm 2 / Thm 4**, mechanized | **Proven** | FIP-linear (`match!`, reuse credits `◇k` consumed linearly), **arguments not shared**, no borrowed reuse credits. Only for code that *passes the FIP check*. |
| Drop-guided reuse holds ≤ const × stack-frames extra memory | Frame-Limited Reuse **§3.4**, proven frame-limited | **Proven** (frame-limited, *not* garbage-free) | token used 0/1×, not captured/passed; "stack" = **heap control-stack depth** in our machine. |
| Reuse preserves **content-address identity** | No external proof — *Mycelium-specific* | **Empirical → must become Declared-with-check** | (S5): reuse only if not pinned in the (weak) intern index, else evict-then-reuse or copy. **New obligation, no prior art.** |
| Coupled RC + region (`RcZero` ⊕ `ScopeExit`) frees each cell exactly once | Gay–Aiken *design* (RC-gated region free); no mechanized combined proof | **Empirical/Declared** | free-list removal on `RcZero`; region-pointer-count > 0 ⇒ no batch-free; single-owner; escape→promote. **Mycelium must state & property-test its own invariant.** |
| Borrow inference correctness | Beans §5 — explicitly "**heuristic … not required for correctness**" | **Empirical** (perf only) | β/δ well-formed; correctness independent of the heuristic's choices. |

**Honest bottom line on the proof frontier:** the **single-mechanism** results (precise RC garbage-freeness,
FIP no-alloc/constant-stack, frame-limited reuse) are **Proven** and Mycelium *inherits their premises for
free* via immutability+acyclicity. The **two genuinely novel Mycelium obligations** — (i) reuse vs
content-address identity, (ii) the eager-RC ⊕ batched-region exactly-once protocol — have **no off-the-shelf
proof** and must be discharged by Mycelium's own checked side-conditions and property tests before any
`Proven` tag (VR-5: don't upgrade assent past its basis).

**Where Mycelium is EASIER:** no cycles ⇒ RC is **complete**, Perceus Thm 2 holds in its strong form, no
cycle-collector needed; no mutable aliasing ⇒ the unique-vs-shared gate is the *only* dynamic check and FIP
linearity is natural. **Where Mycelium is HARDER:** **content-address identity** — in-place reuse can
invalidate a hash-keyed identity that pointer-identity languages (Koka/Lean/Swift) never expose; this is the
one place the survey offers *patterns* (transients, weak intern tables, Unison's decoupled runtime rep) but
**no proof**, so it is the highest-priority new side-condition.

---

## 8. Annotated bibliography (URLs)

**Core RC + reuse (read these first):**
- **Reinking, Xie, de Moura, Leijen — "Perceus: Garbage Free Reference Counting with Reuse," PLDI 2021**
  (Distinguished Paper). The reuse-token/`drop-reuse` mechanism, drop specialization, λ₁ linear resource
  calculus, Thms 1–4, cycle discussion. Paper PDF: <https://xnning.github.io/papers/perceus.pdf> ·
  Extended TR: <https://www.microsoft.com/en-us/research/wp-content/uploads/2020/11/perceus-tr-v1.pdf> ·
  ACM: <https://dl.acm.org/doi/10.1145/3453483.3454032> · MSR:
  <https://www.microsoft.com/en-us/research/publication/perceus-garbage-free-reference-counting-with-reuse-2/>
- **Lorenzen & Leijen — "Reference Counting with Frame-Limited Reuse," ICFP/OOPSLA 2022 (MSR-TR-2021-30).**
  Drop-guided reuse (reuse *after* dup/drop insertion); frame-limited bound and its §3.4 proof; why
  algorithm D and borrow inference are *not* frame-limited. PDF:
  <https://www.microsoft.com/en-us/research/wp-content/uploads/2021/11/flreuse-tr-v1.pdf> · ACM:
  <https://dl.acm.org/doi/10.1145/3547634>
- **Ullrich & de Moura — "Counting Immutable Beans," IFL 2019.** Origin of `reset`/`reuse`, λRC `inc`/`dec`
  IR, owned vs borrowed references, borrow-inference heuristic (`collectO`, fixpoint), `δreuse` insertion.
  Note: compiler correctness proof was *future work* at publication. arXiv:
  <https://arxiv.org/abs/1908.05647> · PDF: <https://arxiv.org/pdf/1908.05647> · Appendix:
  <https://lean-lang.org/papers/beans_appendix.pdf>
- **Lorenzen, Leijen, Swierstra — "FP²: Fully in-Place Functional Programming," ICFP 2023.** λfip calculus,
  reuse credits `◇k`, `match!`; Thm 2 (`|S|=|S′|`, no (de)alloc) and Thm 4 (constant stack), TRMC soundness;
  syntactic (not full type-system) FIP check. PDF:
  <https://www.microsoft.com/en-us/research/wp-content/uploads/2023/07/fip.pdf> ·
  <https://webspace.science.uu.nl/~swier004/publications/2023-icfp.pdf> · ACM:
  <https://dl.acm.org/doi/10.1145/3607840> · blog:
  <https://www.microsoft.com/en-us/research/blog/fp2-fully-in-place-functional-programming-provides-memory-reuse-for-pure-functional-programs/>

**Runtime / language references:**
- **Lean 4 — Reference Counting (language reference).** `isShared`, `dbgTraceIfShared`, reuse-on-same-size,
  persistent (multi-thread) vs unshared partition, copy-shared/mutate-unshared primitives.
  <https://lean-lang.org/doc/reference/latest/Run-Time-Code/Reference-Counting/>
- **Koka language book** (FBIP, reuse in practice): <https://koka-lang.github.io/koka/doc/book.html> ·
  source: <https://github.com/koka-lang/koka>
- **Swift — `isKnownUniquelyReferenced` / COW.** Runtime uniqueness gate behind copy-on-write; `Builtin.isUnique`.
  <https://www.hackingwithswift.com/example-code/language/how-to-safely-use-reference-types-inside-value-types-with-isknownuniquelyreferenced>
  · ARC optimization: <https://apple-swift.readthedocs.io/en/latest/ARCOptimization.html>

**Regions + RC, and RC+region coupling:**
- **Gay & Aiken — "Memory Management with Explicit Regions," PLDI 1998 / the RC system.** Region reference
  counts; internal references not counted; dynamically-gated region deletion. Thesis:
  <https://theory.stanford.edu/~aiken/publications/theses/gay.pdf> · ACM:
  <https://dl.acm.org/doi/10.1145/277650.277748>
- **Terauchi & Aiken — "Memory Management with Use-Counted Regions."**
  <https://apps.dtic.mil/sti/tr/pdf/ADA603317.pdf>
- **Tofte & Talpin region inference / MLKit + GC.** Region inference algorithm:
  <https://elsman.com/mlkit/pdf/toplas98.pdf> · combining regions + tag-free generational GC:
  <https://www.cambridge.org/core/journals/journal-of-functional-programming/article/integrating-region-memory-management-and-tagfree-generational-garbage-collection/782D317A9B811CD99FA0E924A35B6A58>
- **Cyclone region-based memory management** (region polymorphism, scoped regions):
  <https://www.cs.umd.edu/projects/cyclone/papers/cyclone-regions.pdf>

**Content-addressing / persistent structures (the identity tension):**
- **Unison — content-addressed code; runtime keeps a separate compiled rep keyed by hash.**
  <https://www.unison-lang.org/docs/the-big-idea/> · runtime issue (compiled-rep sharing by hash):
  <https://github.com/unisonweb/unison/issues/1055> · hashing standardization:
  <https://github.com/unisonweb/unison/issues/2373>
- **Hash consing** (structural identity via single shared object; pointer-equality structural eq):
  <https://en.wikipedia.org/wiki/Hash_consing>
- **Bagwell HAMT + Clojure persistent structures / transients** (path copying, structural sharing,
  unique-ownership transient mutation): <https://en.wikipedia.org/wiki/Hash_array_mapped_trie> ·
  Steindorfer & Vinju, "Optimizing HAMTs" (OOPSLA'15):
  <https://michael.steindorfer.name/publications/oopsla15.pdf>

**Concurrency / actor cross-boundary RC (for the later cross-hypha story only):**
- **Clebsch et al. — "Orca: GC and Type System Co-Design for Actor Languages," OOPSLA 2017.** Deferred,
  distributed, weighted RC over **causal** messaging; sound (collects only dead) and complete (eventually
  collects all). <http://janvitek.org/pubs/oopsla17a.pdf> · ORCA paper:
  <https://www.ponylang.io/media/papers/OGC.pdf> · tutorial:
  <https://tutorial.ponylang.org/appendices/garbage-collection.html>
- **OCaml 5 multicore memory model** (immutable objects need no read/write barriers — immutability lowers
  sync cost): <https://github.com/ocaml-multicore/docs/blob/main/ocaml_5_design.md> ·
  <https://arxiv.org/pdf/2004.11663>
