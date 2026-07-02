# Lane F Research Report — Efficient Immutable Value Management

> **Status:** Research handoff — `Empirical` where independently corroborated by multiple sources,
> `Declared` throughout for Mycelium-specific design proposals, `Proven` only for cited mechanized
> external results. Non-normative. Input to RFC-0027, the value model, and RFC-0008 RT6 (`fuse`).
>
> **Date:** 2026-06-24
> **Lane:** F — Efficient immutable value management (copy/mut problem + reclamation + CRDT-merge)
> **Confidence tagging:** per VR-5 — `Declared` (asserted intent / design implication), `Empirical`
> (observed in independent implementations or confirmed by multiple papers), `Proven` (mechanized
> results cited from published work). This report does not upgrade any claim past its checked basis.

---

## 1. Question and Central Thesis

### 1.1 The Question

Mycelium's value model commits every collection mutator to returning a new value with structural
sharing (`std.collections` spec: "Every update (`push`, `insert`, `remove`, `with`) returns a
**new** value with structural sharing"). This is not controversial — it is a direct consequence of
LR-8 (immutable values) and the never-silent G2 rule. But it leaves three apparently-separate
problems:

1. **Copy / mutation efficiency.** Returning a new structurally-shared value is asymptotically
   efficient (`O(log n)` for tree-based structures) but carries concrete costs: pointer-chasing,
   cache misses, and allocation per logical "update." Under heavy write workloads, is the
   functional approach competitive?

2. **Reclamation.** When a value goes out of scope (RFC-0027's open problem), what frees the
   memory backing it? The corpus leans on Rust's drop system today but names this an open design
   question.

3. **CRDT-merge (`fuse`, RT6).** When two hyphae merge state via `fuse`, the payload "joins up
   its state lattice while honesty meets down the guarantee lattice" (RFC-0008 RT6). This is a
   semilattice merge, proven sound in Isabelle/HOL — but what is the *efficient* implementation
   of merge over structurally-shared immutable values?

### 1.2 The Central Thesis to Test

**The thesis:** copy/mut efficiency, reclamation (RFC-0027), and CRDT-merge (`fuse`) are three
facets of one problem — lifetime/uniqueness/sharing management over content-addressed values —
and a Perceus-style reference-counting-with-reuse scheme (or a uniqueness-type discipline
anchored on the affine `substrate`) could unify them:

- refcount → 0 = reclaim (memory freed)
- refcount == 1 = mutate in-place (copy elided)
- refcount > 1 = structural-share (the default today)

**Adversarial verdict: the thesis PARTIALLY SURVIVES, with hard limits.** The unification is
real but not tight. Perceus-style reuse applies within a single execution unit; CRDT-merge is
a fundamentally *multi-party* convergence problem that the refcount alone cannot resolve.
The `substrate` affinity lever exists in the corpus but is currently scoped to *external
resources*, not to ordinary value sharing. The strongest actionable finding is a narrower one:
for single-hypha operations over mutable-in-spirit collections, a refcount==1 reuse check
(FBIP-style) is a valid, well-precedented optimization that Mycelium could adopt without
violating value semantics — and the `Provenance` DAG already provides the correct native host
for tracking which path was taken. Details below.

---

## 2. Corpus Grounding — What Mycelium Has and What It Lacks

### 2.1 What Mycelium Has

**Structural sharing (committed).** Every collection mutator in `std.collections` already
returns a new value with structural sharing (`Seq`/`Map`/`Set`; collections spec §3). This is
the standard persistent-data-structure model (HAMTs for maps/sets, RRB-vectors for sequences
— see §3.1 below for prior art). The guarantee: `O(log n)` per logical update, O(1) identity
comparison via content hash (ADR-003). This is *already specified*; the question is efficiency.

**Content-addressed identity (RFC-0001 §4.6).** Every value carries a `Provenance` DAG:
`Provenance ::= Root | Derived{ op: ContentHash, inputs: [ProvenanceRef] }`. Two values with
the same hash are identical — no deep equality needed (`Exact`, structural). This is the
Unison model applied to all values. `Declared` tag on the *performance* claim (cost of
hashing not yet benchmarked).

**The affine `substrate` / `consume` (DN-02 §2, DN-03 §1, Glossary.md §2.18, RFC-0006 LR-8).**
A `substrate` is an affine external resource — consumed exactly once. This is the corpus's
existing linearity/uniqueness lever. However, it is currently scoped to *external resources*
(file handles, network connections, I/O capabilities), not to the ordinary `Seq`/`Map`/`Set`
values that need efficient mutation. The key limitation: `substrate` affinity guarantees
*single-use* at the language surface, but it is not a *runtime uniqueness probe* — there is
no `isKnownUniquelyReferenced`-style dynamic check used to decide whether to copy or reuse.
This is the gap.

**The guarantee lattice + `propagate` (RFC-0001 §4.7, `guarantee.rs`).** The
`GuaranteeStrength` meet-semilattice (exhaustively verified: commutativity, associativity,
idempotency, identity `Exact`, absorption by `Declared`) propagates honesty monotonically.
This is directly relevant to `fuse` (RT6): merged values' guarantees are composed by `meet`.
Tag: `Proven` (exhaustive over the 4×4 finite lattice — a complete check, not sampling;
`guarantee.rs` tests).

**RT6 `fuse` — join-semilattice discipline with Isabelle proof (`Proven` for the external
mechanism, `Declared` for the Mycelium mapping).** RFC-0008 RT6: "Fusion is lawful merge:
join on payload, meet on guarantee. ... Commutative, associative, and idempotent — the
monotonic-semilattice condition under which convergence is a *theorem* (sufficient, not
necessary — CRDT strong eventual consistency, T4.2; mechanized in Isabelle/HOL)." The
Isabelle proof covers the CRDT/semilattice condition *in general*; the specific Mycelium
`fuse` binding of it is `Declared` (unbuilt).

**RFC-0027 (Draft) — reclamation is an open design question.** The corpus
commits that: (a) no cycle detection is needed (LR-9, acyclic values); (b) `reclaim` is
supervision of runtime *units*, not a memory primitive (DN-03 §4); (c) the RT7 scope tree
provides structured lifetime discipline; (d) channel close protocol gives single-owner
transfer across hyphae. What is *not* decided: how memory backing values is actually freed,
whether sweep-order couples to reclamation order, and the latency bound. All `Declared`.

**RFC-0008 RT1 + channel close (RT7, D-5 from SYNTHESIS).** "The only thing that crosses a
hypha, channel, or node boundary is an immutable `Value` with its `Meta` intact." Affine
(non-`Clone`) `Sender`/`Receiver` provide single-owner cross-hypha transfer at R1 — no
distributed refcount needed (`Empirical`, grounded in the channel close protocol).

### 2.2 What Mycelium Lacks

**A runtime uniqueness probe for ordinary values.** The corpus has `substrate` for external
resources and Rust's refcount for the kernel implementation, but no surface-level or
compiler-supported mechanism to ask "is this value uniquely referenced right now, so I may
safely reuse its allocation?" — the Perceus/FBIP reuse check, Clean's uniqueness type, or
Swift's `isKnownUniquelyReferenced`.

**An in-place mutation path for single-owner values.** Because there is no uniqueness probe,
there is no sanctioned path from "I am the sole owner of this `Seq`" to "I may update the
node in place rather than copying it." Every `push`/`update` necessarily allocates a new node,
even when the old one is about to be dropped. This is the copy/mut inefficiency.

**An explicit reclamation model for value memory.** RFC-0027 is a planning stub. The gap
between "Rust drops it" and "Mycelium's runtime explicitly reclaims it with an EXPLAIN record"
is the entire scope of that RFC.

**A δ-CRDT or structural-diff-aware `fuse` implementation.** RT6 specifies the *algebraic
law* (`fuse` is join); it does not specify how merging two large persistent collections
efficiently finds the delta. Full-state state-based CRDT merge over large structures requires
sending and merging the full state; δ-CRDTs send only the change. This is an open design
question (see §5).

---

## 3. External Prior Art

### 3.1 Persistent / Immutable Data Structures and Structural Sharing

**Okasaki (1996/1998) — purely functional data structures** established the systematic
foundations: lazy evaluation + amortization + persistent trees. Key result: most sequential
data structures have persistent equivalents with the same asymptotic bounds, but with worse
constants due to indirection and allocation. The foundational reference:
[Purely functional data structures — Wikipedia](https://en.wikipedia.org/wiki/Purely_functional_data_structure).

**HAMT (Bagwell 2000/2001, popularized by Clojure).** Hash Array Mapped Tries: `O(log₃₂ n)`
lookup/update with structural sharing, excellent cache utilization relative to balanced BSTs.
The standard prior art for persistent maps and sets. Used in production in Clojure, Scala,
Python (`frozenset`-adjacent), JavaScript (Immutable.js). `Empirical` tag: independently
adopted across many production systems confirms practical viability.
Reference: [Clojure's Persistent Data Structures](https://www.javacodegeeks.com/2026/02/clojures-persistent-data-structures-immutability-without-the-performance-hit.html).

**RRB-Trees (Bagwell & Rompf, EPFL 2011; Stucki et al. ICFP 2017).** Relaxed Radix Balanced
Trees extend persistent vectors (Clojure/Scala's PersistentVector) with O(log n)
concatenation and slicing — critical for `Seq::concat` and `Seq::slice`. The
[immer C++ library](https://public.sinusoid.es/misc/immer/immer-icfp17.pdf) demonstrates
RRB-trees in a systems language with competitive performance. This is the relevant prior art
for Mycelium's `Seq`. Reference:
[RRB-Trees: Efficient Immutable Vectors (Bagwell & Rompf)](https://infoscience.epfl.ch/bitstreams/e5d662ea-1e8d-4dda-b917-8cbb8bb40bf9/download);
[Persistence for the Masses (ICFP 2017)](https://public.sinusoid.es/misc/immer/immer-icfp17.pdf).

**Honest cost model** (`Empirical`): persistent data structures pay real costs: pointer
chasing, O(log n) per update even when amortized, and allocation pressure. For large
structures, cache misses dominate and persistence overhead shrinks relatively; for small
structures (< 32 elements), mutable arrays often win by 5–10×. No persistent structure
matches a mutable array's sequential write throughput. This is the honest constraint that
motivates the reuse techniques in §3.2.

### 3.2 In-Place Reuse Under Value Semantics

**Perceus (Reinking, Xie, de Moura, Leijen — PLDI 2021).** The central prior art for the
thesis. Perceus is an algorithm for precise reference counting with reuse: starting from a
functional core with explicit control flow, it emits RC instructions such that cycle-free
programs are *garbage-free* (only live references retained) and enables *reuse analysis*: when
a value is about to be dropped and a new value of compatible shape is about to be allocated on
the same code path, the old allocation is reused in-place at runtime — if and only if the
runtime refcount is 1. This enables **FBIP (Functional But In-Place)**: writing in-place
mutating algorithms in a purely functional style, exactly as tail-call optimization enables
writing loops with regular function calls.

Key facts (`Empirical` from published benchmarks):
- Competitive with OCaml, GHC on typical functional benchmarks; beats them on FBIP-amenable
  workloads.
- Requires cycle-free values — *exactly* Mycelium's LR-9 guarantee. No cycle detector needed.
- The reuse check is O(1): `if refcount == 1 { reuse allocation } else { allocate fresh }`.
- The reuse token is a *linear* value: a proof that the allocation is uniquely owned, passed
  to the allocator to fill the new value in-place.

References:
- [Perceus: Garbage Free Reference Counting with Reuse (PLDI 2021)](https://pldi21.sigplan.org/details/pldi-2021-papers/7/Perceus-Garbage-Free-Reference-Counting-with-Reuse)
- [Perceus PDF (Microsoft Research)](https://www.microsoft.com/en-us/research/wp-content/uploads/2020/11/perceus-tr-v1.pdf)
- [FP2: Fully In-Place Functional Programming (ICFP 2023)](https://www.microsoft.com/en-us/research/wp-content/uploads/2023/07/fip.pdf)

**FP2 / `fip` and `fbip` keywords (Leijen et al., ICFP 2023).** Extends Perceus to a
*static* fully-in-place discipline. A function marked `fip` is *guaranteed* to run in-place
(no allocation on any path) — this is a *type-checked* guarantee, not a runtime probe. The
`fbip` keyword is the more permissive "in-place but may use stack." This is directly relevant
to Mycelium: a `fip`-annotated function over an affinely-owned value is a statically-verified
in-place mutation, which is exactly what the `substrate` uniqueness lever *could* enable for
ordinary values if the scope were widened.

Reference: [FP2: Fully in-Place Functional Programming (ICFP 2023)](https://webspace.science.uu.nl/~swier004/publications/2023-icfp.pdf)

**Counting Immutable Beans — Lean 4 (de Moura & Ullrich, 2019).** Lean 4's reference
counting scheme, optimized for purely functional programming (same "immutable, acyclic" model
as Mycelium): borrowed references reduce RC update traffic; heuristic borrow-annotation
inference minimizes increments/decrements; reuse optimizations reduce allocator calls. Lean 4
achieves performance competitive with or better than GHC on standard benchmarks. The
no-cycle observation is the same structural argument Mycelium's LR-9 makes.

References:
- [Counting Immutable Beans (arXiv 2019)](https://arxiv.org/pdf/1908.05647)
- [Lean 4 Reference Counting documentation](https://lean-lang.org/doc/reference/latest/Run-Time-Code/Reference-Counting/)

**Destination-Passing Style (DPS, Minamide 1998; recent: DPS for Haskell 2022; Destination
Calculus 2025).** A lower-level technique: instead of returning a new value, a function takes
a *destination* (a pre-allocated slot) and writes into it. A linear typing discipline ensures
each destination is written exactly once. This is the mechanism by which Perceus reuse tokens
work at the code-generation level — the FP2 paper uses DPS as the compilation target for
`fip` functions.

References:
- [Destination-passing style for efficient memory management (2017)](https://dl.acm.org/doi/10.1145/3122948.3122949)
- [Destination Calculus: A Linear λ-Calculus for Purely Functional Memory Writes (2025)](https://arxiv.org/pdf/2503.07489)
- [Formalization and Implementation of Safe Destination Passing in Pure Functional Programming Settings (2025)](https://arxiv.org/pdf/2601.08529)

### 3.3 Uniqueness Types and Static In-Place Guarantees

**Clean uniqueness types (Barendsen & Smetsers, 1990s).** The original static uniqueness
system: a type `*T` (unique `T`) guarantees no other reference exists — the compiler can
legally perform in-place updates. Unique values may be freely mutated; shared values must be
copied before mutation. A uniqueness type is a *global* guarantee (no other reference exists),
whereas a linear type is a *local* obligation (you must use this value exactly once). Both
deliver in-place reuse; uniqueness is the more powerful guarantee.

Reference: [Uniqueness Types and In-Place Updates (Futhark language blog 2022)](https://futhark-lang.org/blog/2022-06-13-uniqueness-types.html);
[Uniqueness type — Wikipedia](https://en.wikipedia.org/wiki/Uniqueness_type).

**Rust's `&mut T` as a uniqueness discipline** (`Empirical`): Rust's borrow checker enforces
a weaker form of uniqueness: at any given moment, either one `&mut T` or N `&T` references
exist, never both. This permits in-place mutation of `&mut T` values. It is the reason the
borrow checker exists — exactly what LR-8/LR-9 make unnecessary at the Mycelium language
surface (but the Rust *kernel* still uses it for the implementation).

**Fractional uniqueness / functional ownership (2023).** Recent work extends uniqueness to
fractional capabilities (read fractions, write 1/1) — a refinement between full uniqueness
and full sharing. Not directly relevant to Mycelium's current design point but noted as a
future-facing research direction.

Reference: [Functional Ownership through Fractional Uniqueness (2023)](https://arxiv.org/pdf/2310.18166).

**Swift copy-on-write (CoW) + `isKnownUniquelyReferenced`.** Swift's standard library value
types (Array, String, Dictionary) use a class-backed buffer and `isKnownUniquelyReferenced`
— a runtime query — to implement CoW: "if refcount == 1, mutate in place; else copy first."
This is the *dynamic* analogue of Clean's *static* uniqueness types, and it is exactly the
Perceus reuse-check mechanism at the application layer. The Swift pattern is well-understood
in production at scale (the entire Swift stdlib). `Empirical` tag: proven viable in a
real-world stdlib context.

Reference: [A Deep Dive into Copy-on-Write in Swift](https://grokkingswift.io/a-deep-dive-into-copy-on-write-in-swift/).

### 3.4 δ-CRDTs and Content-Addressed Merge Structures

**State-based CRDTs and join-semilattices.** A state-based CRDT is a join-semilattice: merge
is commutative, associative, idempotent (`join` = least upper bound). Full-state CRDTs require
sending and merging entire states, which is expensive for large structures. `Empirical` tag
for the general semilattice result; the Isabelle proof RFC-0008 cites makes this `Proven` for
the specific join laws.

**δ-CRDTs (Almeida, Baquero, Shoker 2015/2016).** Delta state CRDTs: instead of shipping
the full state, ship only the recently-applied *delta* — the minimal state fragment whose join
with the current state produces the updated state. A δ-CRDT is still a join-semilattice; the
delta-group is a smaller element of the same lattice. This reduces network traffic from O(state
size) to O(change size) per sync. Highly relevant to `fuse` over large `Seq`/`Map`/`Set`
values at R2 (`xloc`).

References:
- [Efficient State-based CRDTs by Delta-Mutation (core.ac.uk)](https://core.ac.uk/download/pdf/154274608.pdf)
- [In-Memory Distributed State with Delta CRDTs (WorkOS)](https://workos.com/blog/in-memory-distributed-state-with-delta-crdts)

**Merkle-CRDTs (Psaras et al. 2020).** Embed δ-CRDTs in Merkle-DAG nodes: the content hash
of a state node simultaneously identifies the state and enables anti-entropy (two replicas
can exchange only the hashes of their state chains, find the common ancestor, and send only
the divergent sub-DAG). This is directly analogous to Mycelium's `Provenance` DAG
(`Root | Derived{op, inputs}`) — both are acyclic derivation graphs keyed by content hashes.
The Merkle-CRDT paper is the tightest external prior art for the R2 `mesh` + `fuse`
combination.

Reference: [Merkle-CRDTs: Merkle-DAGs meet CRDTs (arXiv 2020)](https://arxiv.org/pdf/2004.00107).

**Unison content-addressed code + git object model.** Unison: every function definition is
identified by a 512-bit hash of its normalized structure + dependencies. Names are pointers
into the hash space. This is the RFC-0001 §4.6 model applied to code rather than values —
Mycelium already adopted this design. The git object model (blobs/trees/commits as
content-addressed Merkle nodes) is the same structure applied to file system snapshots.
Both confirm that content-addressing + immutability + structural sharing are a proven,
production-deployed combination.

References:
- [Unison 1.0: Content-Addressed Code](https://byteiota.com/unison-1-0-content-addressed-code-hits-production/)
- [Unison docs: the big idea](https://www.unison-lang.org/docs/the-big-idea/)

---

## 4. The Unification Analysis

### 4.1 The Core Triangle

The three problems share one invariant: **sharing state**. Formally:

```
refcount(v) → 0  ⟹  v is unreachable; reclaim its allocation     [RECLAMATION]
refcount(v) == 1 ⟹  v has a single owner; reuse in place          [COPY/MUT]
refcount(v) > 1  ⟹  v is shared; structural-share on update        [DEFAULT TODAY]
```

This is precisely the Perceus reuse-check mechanism, the Swift CoW pattern, and the Lean 4
borrowed-reference optimization — three independently-derived convergences on the same
three-way split. The `Provenance` DAG (RFC-0001 §4.6) records *which path was taken* and
makes it EXPLAIN-able: a `Derived{op, inputs}` node whose `op` is a reuse event is
distinguishable from one whose `op` is a copy — no silent elision.

**Where the triangle is tight** (`Empirical`): within a single hypha (single-owner computation
over a linear pipeline), the three cases map cleanly. The refcount is a proxy for ownership:
refcount==1 is a dynamic proof of uniqueness. This is what Perceus, Lean 4, and Swift CoW all
exploit.

**Where the triangle breaks** (`Empirical`, adversarial): for `fuse` (CRDT-merge across
hyphae), the three-way split is not sufficient. Merge requires: (a) both replicas survive the
merge (neither is dropped), (b) the result is the join of both inputs, and (c) the merge must
be idempotent (merging the same delta twice is a no-op). This is *neither* reclamation nor
in-place reuse — it is a *new structural composition*. The refcount alone tells you nothing
about the *merge function*; that is defined by the semilattice law, not by sharing. The
unification of reclamation + copy/mut is tight; the extension to `fuse` is architectural
(semilattice semantics), not mechanical (refcount probe).

**Synthesis**: The thesis is correct as a unification of *reclamation* and *copy/mut
efficiency* under a refcount probe. It is incorrect as an extension to *CRDT-merge*, which is
a separate algebraic problem. Tag: `Empirical` for the narrower claim; `Declared` (and
contested) for the full three-way unification.

### 4.2 Concrete Design Implications

#### 4.2.1 → RFC-0027 (reclamation model)

**Implication F-1 (`Declared`): RFC-0027 should adopt reference counting as the reclamation
mechanism, not tracing GC.** Rationale: LR-9 (acyclic values) eliminates the only hard case
for RC (cycle detection). Perceus and Lean 4 demonstrate that precise RC over acyclic
functional values is competitive with tracing GC and simpler to EXPLAIN (each
increment/decrement is a reified event, auditable). The "no silent GC pause" stance
(RFC-0027 §1) is *naturally* satisfied by RC: there is no stop-the-world phase — each
`rc_dec` is a bounded O(1) event (or O(depth) for a chain drop, which is deterministic and
bounded by tree depth). This is the `Empirical` advantage of RC for Mycelium's model.

**Implication F-2 (`Declared`): The reclamation EXPLAIN record (SYNTHESIS A-1) maps directly
onto a refcount event.** Fields: `scope_id`, `sweep_epoch`, `trigger: RcZero | ScopeExit |
ChannelClose`, `value_meta_hash`, `channel_id?`. The `trigger` field distinguishes the three
cases: `RcZero` (refcount hit 0, deferred to scope exit), `ScopeExit` (scope tree node
closed), `ChannelClose` (channel disconnected, ownership released). This gives the EXPLAIN
record a structural grounding in the refcount mechanism, not just an editorial label.

**Implication F-3 (`Declared`): Sweep-order coupling (RFC-0027 OQ-1) may be partially
resolved by RC.** With reference counting, reclamation of a value follows deterministically
from its last-owner's drop — which is itself ordered by the scope tree (RT7). The sweep-order
is not an *additional* constraint on reclamation; it is *derived from* the scope tree order.
Strong vs. weak coupling becomes: "is a deferred drop (the `scope_exit` trigger) permitted to
out-of-order with sibling scopes?" This is still open, but the RC framing makes the question
precise: deferred drops accumulate per scope and flush at scope-exit, in child→parent order
(the RT7 LIFO for in-scope values, per SYNTHESIS CL-7 — correct for within-scope, separate
from cross-scope channel protocol).

#### 4.2.2 → The Value Model (copy/mut efficiency)

**Implication F-4 (`Declared`): A `reuse` annotation (FBIP-style) on collection mutators
would allow in-place updates when the caller is the sole owner.** Specifically, if a `Seq::push`
caller can statically or dynamically prove uniqueness (`substrate`-typed or refcount==1 probe),
the implementation may update the node in-place rather than allocating a new node. The
*surface* semantics are unchanged (caller observes a new `Seq` value); the
*implementation* is in-place. This is exactly the FBIP principle. The `Provenance` DAG records
the outcome: a `Derived{op: "push_reuse", inputs: [old_hash]}` is distinguishable from
`Derived{op: "push_copy", inputs: [old_hash]}`.

**Implication F-5 (`Declared`): The `substrate` affinity lever is the correct foundation for
extending static uniqueness to ordinary values, not just external resources.** In Clean,
every `*T` (unique) value gets in-place updates; in Mycelium, `substrate` is currently
"external resources only." Widening `substrate`-style uniqueness to ordinary `Seq`/`Map`/`Set`
values would bring Mycelium to the Clean/Koka FBIP design point *statically* — no runtime
refcount probe needed. The cost: a more complex type system and inference discipline. The
payoff: guaranteed in-place performance without dynamic checks. This is a named future
direction, not a current proposal (RFC-0027 scope is memory model; this touches the type
system surface — a separate RFC).

**Implication F-6 (`Empirical`): For the current v0 interpreter phase, structural sharing
is the correct default and RC is sufficient.** At v0 scale, the difference between O(log n)
persistent updates and O(1) mutable updates is not the bottleneck. The right sequencing is:
(1) commit to RC for reclamation (RFC-0027), (2) add FBIP-style reuse checks in the runtime
library (not the type system) as an optimization, (3) consider static uniqueness types as a
Phase-7 or later addition when the compiler is mature enough to infer them. Over-engineering
the uniqueness type system before the interpreter is stable is a KC-3/YAGNI violation.

#### 4.2.3 → `fuse` / RT6 (CRDT-merge efficiency)

**Implication F-7 (`Declared`): `fuse` over persistent collections should use δ-CRDT
structural diffing at R2 (`xloc`/`mesh`), not full-state merge.** Full-state merge of a
large `Seq` or `Map` requires serializing and merging the entire tree — O(n) communication
and O(n log n) merge time. With δ-CRDTs, only the delta (the changed sub-tree) is sent; its
join with the recipient's current state is O(delta size). The `Provenance` DAG already
provides the necessary structure: two replicas can compare their DAG roots (content hashes),
find the common ancestor (`LCA` in the Provenance DAG), and send only the sub-DAG rooted
above the LCA — exactly the Merkle-CRDT anti-entropy protocol.

**Implication F-8 (`Declared`): The `Provenance` DAG is the native host for merge provenance,
making `fuse` results EXPLAIN-able.** A `fuse` result's `Provenance` is
`Derived{op: "fuse_join", inputs: [left_root, right_root]}`. The semilattice law (join is
idempotent) means re-merging the same delta produces the same `Derived` node — content-
addressable idempotency. This is a free structural property of the `Provenance` DAG design.

**Implication F-9 (`Empirical`): RT6's guarantee composition (`meet` on guarantees during
`fuse`) is correct and makes `fuse` provenance-honest.** When `fuse` merges two values with
guarantees `g1` and `g2`, the result has guarantee `meet(g1, g2)` — the weaker of the two
inputs. This is the SYNTHESIS finding (AG-4): the guarantee lattice is the universal honesty
vocabulary. A `fuse` between a `Proven`-tagged replica and an `Empirical`-tagged replica
produces an `Empirical` result — the honesty degrades honestly, never spuriously upgrades.
This is `Proven` for the lattice laws; `Declared` for the `fuse`-specific wiring (unbuilt).

### 4.3 The Unification: What It Resolves and What It Does Not

| Problem | Unified by RC + Reuse? | Mechanism | Tag |
|---|---|---|---|
| Memory reclamation (RFC-0027) | YES | `rc_dec → 0 ⟹ free` | `Empirical` (Perceus/Lean4 precedent) |
| Copy/mut efficiency (in-place update) | YES, conditionally | `rc == 1 ⟹ reuse` | `Empirical` (FBIP/CoW precedent) |
| Structural sharing (default) | N/A (already done) | `rc > 1 ⟹ copy node` | `Exact` (current spec) |
| CRDT-merge (`fuse`) correctness | NO — separate concern | Semilattice join law | `Proven` (external Isabelle) |
| CRDT-merge efficiency (δ) | PARTIALLY — DAG structure helps | δ-CRDT + Provenance DAG LCA | `Declared` (F-7) |
| Guarantee composition across `fuse` | YES — same lattice | `meet(g1, g2)` | `Proven` (exhaustive tests) |

The thesis is real for reclamation + copy/mut. It is *useful but incomplete* for `fuse`:
the `Provenance` DAG provides structural efficiency gains, but `fuse` correctness is
algebraically independent of refcounting.

---

## 5. Honest Costs and Open Research Questions

### 5.1 Honest Costs

**RC overhead** (`Empirical`): Every reference copy and drop must update an atomic counter.
Under high multi-hypha sharing, this can be a cache-line contention bottleneck (the
`Arc::clone`/`Arc::drop` cost in Rust is well-documented). Mitigation: Lean 4's borrowed
references reduce the number of RC updates — a borrow does not increment/decrement; it is
a local view. Mycelium's "values move, state is never shared" (RT1) limits cross-hypha
sharing to what crosses channels, which is already bounded by the channel protocol. Within a
hypha, local values are single-threaded and RC updates are non-atomic — a significant
performance win (`Empirical`, Perceus benchmark data).

**Refcount==1 check is O(1) but not free** (`Empirical`): The FBIP reuse check adds a branch
to every allocating operation. On branch-predictable workloads (tree traversal + update) the
branch is nearly free; on mixed-owner workloads the branch misprediction cost is non-zero.
Perceus benchmarks show net gains on most functional workloads; the honest caveat is that
pathological sharing patterns (many owners of the same collection) degrade to the full-copy
case plus the overhead of failed reuse checks.

**O(log n) structural sharing vs. O(1) mutable arrays** (`Empirical`): For write-heavy
workloads on small collections (< 32 elements), mutable arrays beat persistent HAMTs by 3–10×.
For read-heavy workloads or large collections (> 10K elements) with occasional writes, the
persistent structure's cache behavior and GC avoidance often wins. Mycelium's choice of
structural sharing is the correct default; in-place reuse is an optimization for
write-heavy paths.

**Pointer chasing / cache misses** (`Empirical`): Persistent tree nodes are heap-allocated
and pointer-chased. On modern CPUs with 3-level caches, a tree of depth 7 (10M elements in
a HAMT with branching factor 32) involves 7 pointer chases — potentially 7 L3 cache misses
if the tree is large enough not to fit in L2. This is unavoidable for persistent structures;
the mitigation is node packing (placing logically-related nodes in adjacent memory, an arena
allocator strategy) and the compaction passes that some HAMT implementations use.

**RC for collections with large fanout** (`Empirical`): Dropping a deep tree decrements the
RC of every node transitively. If the tree has n nodes, drop is O(n) in the worst case (all
nodes become free simultaneously). This is the same O(n) as GC's mark phase but is
*incremental* (each `rc_dec` is bounded) and *deterministic* (triggered by scope exit, not
an ambient collector). For Mycelium's "no silent GC pause" stance this is a feature: the
cost is known and auditable. But for latency-sensitive paths, a scope exit that drops a
large tree has a visible cost spike. Mitigation: deferred reclamation (batch the decrements,
flush at a scheduled reclamation epoch) — exactly RFC-0027's sweep epoch model.

### 5.2 Open Research Questions

**OQ-F1 (`Declared`): Should Mycelium adopt dynamic RC with reuse checks (Perceus-style) or
static uniqueness types (Clean/Koka-style) for copy/mut optimization?**
- Dynamic RC (Perceus): no type system change, runtime overhead, works transparently.
- Static uniqueness (Clean/FP2 `fip`): type-system change required, zero runtime overhead on
  the fast path, compile-time errors when reuse is impossible.
- The two are not mutually exclusive: static analysis can eliminate most RC checks, with
  dynamic fallback for the remainder (the FP2 approach). But the design space is wide and
  the right choice depends on how complex Mycelium's type system will become. `substrate`
  is the existing static uniqueness lever; widening it is the conservative path.

**OQ-F2 (`Declared`): How does the RC model interact with Mycelium's `cyst` checkpointing
(RFC-0027 OQ-5)?**
With RC: a `cyst` checkpoint serializes values in scope. After serialization, the original
allocation's refcount can be decremented (checkpoint-and-free). The content-addressed identity
(the hash) survives the free — SYNTHESIS TN-4 and CL-5 established this is *sound in
principle* but requires an empirical serializer property test before shipping. The RC model
makes checkpoint-and-free *mechanically* cleaner: the serializer holds a temporary reference
(RC+1 during serialization), releases it on completion (RC-1), and if this was the last
reference, the allocation is freed. No separate free-after-checkpoint decision needed.

**OQ-F3 (`Declared`): δ-CRDT implementation for `fuse` — which delta representation?**
State-based `fuse` for `Map`/`Set` could use: (a) a full copy of the changed sub-tree (simple,
O(n) worst case), (b) a path-copy diff (O(log n), the natural structural sharing representation),
or (c) an explicit δ-CRDT delta type (requires defining the delta lattice per collection
type). Option (b) is the natural fit for Mycelium: the new node after a `push` or `insert`
already contains exactly the changed path (the root-to-leaf path is what structural sharing
copies). The `Provenance` DAG encodes this path. So the "delta" for `fuse` is the new
provenance chain since the last common ancestor — making the anti-entropy protocol a DAG
walk, not a full-state diff. Whether this is sufficient for R2 is open; weighted reference
counting for distributed refcounts (SYNTHESIS O-8) is a separate question.

**OQ-F4 (`Declared`): How does in-place reuse interact with the guarantee lattice?**
If `Seq::push` runs in-place (refcount==1), the new node's `Provenance` is
`Derived{op: "push_reuse", ...}`. If it copies (refcount>1), it is
`Derived{op: "push_copy", ...}`. The guarantee tag is `Exact` either way (push carries no
approximation). But the EXPLAIN record differs: a reuse event is a distinct operation from a
copy event. The question is whether this distinction should be surface-visible (the caller can
observe which path was taken) or implementation-private (only the EXPLAIN record tracks it).
Perceus treats reuse as fully transparent (the caller sees only the new value, not whether
it was in-place); FP2 `fip` makes it static and visible. Mycelium's never-silent G2 principle
suggests the EXPLAIN record should capture the path, even if the surface value is identical.

**OQ-F5 (`Declared`): Worst-case drop latency for RC chains (the latency bound question,
SYNTHESIS O-7).**
If a `Seq` of 10M elements is dropped, the RC cascade decrements every node transitively.
Is this worst-case O(n) or can it be bounded by the fuel model (RFC-0007 §4.5)? The fuel
clock governs *computation* steps, not allocation events. Extending the fuel model to
reclamation steps is an open question. Without it, the "no silent GC pause" stance has a
caveat: a scope-exit that drops a large tree is a bounded-but-large pause, not a
sub-millisecond one. The RFC-0027 sweep-epoch model could spread this cost across multiple
epochs, bounding per-epoch latency at the cost of deferred reclamation.

---

## 6. Prioritized Design Implications

Ranked by impact on the open RFCs:

1. **Adopt RC as the reclamation mechanism in RFC-0027 (F-1, F-2, F-3).** LR-9 eliminates
   cycles; RC is the natural fit. Defines the reclamation EXPLAIN record trigger field. Makes
   sweep-order coupling derivable from the scope tree (RC drops accumulate per scope, flush at
   scope-exit in child→parent order). Unblocks RFC-0027/r10. (`Declared`, low-regret)

2. **Add runtime reuse checks (FBIP-style) to `std.collections` mutators (F-4).** After RFC-0027
   settles on RC, the mutators can add an O(1) `rc==1` check before allocating a new node.
   Transparent to the caller; EXPLAIN-visible in the Provenance DAG. No type-system change needed.
   (`Declared`, medium-regret — depends on RFC-0027 settling first)

3. **Use the `Provenance` DAG LCA as the δ-CRDT anti-entropy anchor for `fuse` at R2 (F-7, F-8).**
   Two replicas exchange their DAG root hashes; the LCA is the last common ancestor; the
   path above LCA is the delta to exchange. Integrates with the existing `Provenance` DAG type
   without a new primitive. (`Declared`, longer-term — R2 is deferred)

4. **Record `push_reuse` vs `push_copy` in the Provenance DAG for EXPLAIN compliance (F-4, OQ-F4).**
   Cost: one extra field in the `Derived` node. Payoff: the never-silent G2 rule is satisfied
   for the reuse/copy choice. (`Declared`, cheap, can be done with F-2)

5. **Flag static uniqueness types (substrate-widening to ordinary values) as a named future RFC
   (F-5).** Do not attempt now (KC-3/YAGNI); name it as the design point Koka/FP2 reaches,
   so future RFC authors have the prior-art grounding. (`Declared`, deferred)

---

## 7. Adversarial Self-Check

**Claim F-A: "Perceus is directly applicable to Mycelium."**
*Challenge:* Perceus targets a *functional core* language with explicit RC emission at the
IR level; Mycelium's Core IR (RFC-0001) is a typed term language, not a RC-annotated low-level
IR. Perceus's reuse analysis operates on a control-flow graph, not an applicative term.
*Verdict: SURVIVES, qualified.* Perceus's *algorithm* is applicable to Mycelium's Core IR if
the RC emission is added as a lowering pass (RFC-0004's lowering pipeline). The term
structure of the Core IR (acyclic, substitution-based, `Let`/`Op`/`Swap`) is amenable to the
same reuse analysis — Perceus operates on a similar `let`-normal form. The *exact* algorithm
would need adaptation; the *approach* is directly applicable. Down-tag from "direct" to
"requires adaptation." (`Empirical`, not `Proven` — no formal adaptation proof in-repo.)

**Claim F-B: "Acyclicity (LR-9) makes RC correct without a cycle detector."**
*Challenge:* LR-9 applies to Mycelium *values*; the Rust *kernel* implementation uses
`Rc`/`Arc` for some internal data structures that may have cycles. Does this break the
claim?
*Verdict: SURVIVES, scoped.* LR-9 applies to the *Mycelium language surface*; the Rust kernel
is implementation-private. The reclamation model in RFC-0027 governs surface-level *value*
reclamation, not kernel data structure lifetimes (the kernel uses Rust's own safety mechanisms).
The scoping from SYNTHESIS CL-1 applies here: "Mycelium *surface* has no cycles; the kernel
and `wild` still do." Carry only with this scope.

**Claim F-C: "δ-CRDT merge via Provenance DAG LCA is a correct anti-entropy protocol."**
*Challenge:* The Merkle-CRDT paper proves this for specific CRDT types; does it generalize
to arbitrary Mycelium `fuse` operations?
*Verdict: DOWN-TAGGED.* The protocol is correct for *monotone* join-semilattice CRDTs where
the state ordering is consistent with the Provenance DAG ordering. If a Mycelium `fuse`
operation is non-monotone (e.g., a `remove` operation that reduces state), the DAG LCA
anti-entropy may miss necessary state. For *grow-only* CRDTs (add-only `Set`, counter), it
is correct (`Empirical`). For general lattice-valued `fuse` with removals, additional
machinery (tombstones, version vectors) is needed. Tag: `Declared` for the general claim;
`Empirical` for grow-only CRDTs.

**Claim F-D: "RC is latency-bounded per drop."**
*Challenge:* A drop of a deep value tree is O(depth × branching factor), not O(1).
*Verdict: SURVIVES, qualified.* Each individual `rc_dec` is O(1); the *cascade* is O(n)
where n is the number of nodes being freed. "Bounded" means: deterministic, observable, and
EXPLAIN-able — not constant-time. The "no silent GC pause" stance is satisfied (each step is
visible); the latency SLO is not (OQ-F5). This is the same distinction RFC-0027 §1 draws:
"the only acceptable pause is a bounded, logged, never-silent reclamation event." Bounded
means bounded by the tree size, not by a hard real-time constant.

---

## Meta — Changelog

- **2026-06-24 — Created.** Lane F research handoff: efficient immutable value management
  (copy/mut + reclamation + CRDT-merge). Non-normative research artifact. Feeds RFC-0027,
  `std.collections`, RFC-0008 RT6 (`fuse`). Per-claim VR-5 tags throughout.
  Append-only.
