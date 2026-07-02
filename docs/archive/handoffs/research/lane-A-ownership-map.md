# Lane A — Rust Ownership/Borrow/Lifetime/`unsafe` → Mycelium Value-Semantics Map

**Lane:** A  
**Date:** 2026-06-24  
**Author:** Research Agent (Sonnet-4.6)  
**Status:** Decision-grade research handoff — `Empirical` findings, `Declared` where noted  
**Grounding basis:** Mycelium corpus (RFC-0001, RFC-0002, RFC-0007, RFC-0008, RFC-0027, RFC-0028,
DN-02, DN-03, Glossary.md, ADR-014, ADR-032) + external prior art (cited inline)

---

## 1. Question

How does each major Rust ownership/borrow/lifetime/`unsafe` construct map onto Mycelium's
value-semantics model? Which mappings are faithful (semantics preserved), which are lossy
(partial analogues exist), which are blocked on RFC-0027 (the Draft memory-management RFC), and
which require a human decision because Mycelium has deliberately chosen a different design
point? What is the shape of the residue, and where does each residue item live in Mycelium's
design space?

This question matters because: (a) Mycelium's Rust kernel leans on Rust's ownership/borrow
checker for memory safety of the *implementation*, while the *Mycelium language surface* makes
very different guarantees; (b) any RFC or implementation work that touches the Mycelium
language's memory/reclamation model (principally RFC-0027, currently Draft) must understand what
Rust constructs it is replacing, re-encoding, or deliberately omitting; (c) the translation
validation story (RFC-0002's swap certificate; ADR-002's split regime) needs to know which
Mycelium semantics cross-compile faithfully from Rust's model and which do not.

---

## 2. Mycelium Corpus Grounding

### 2.1 Core value model (RFC-0001)

Mycelium values are immutable triples `(Repr, Payload, Meta)`. Every value crossing any boundary
— hypha, channel, or node — is immutable and carries its metadata intact (RFC-0001 WF5;
RFC-0008 RT1). The key consequences for the ownership map:

- **No aliased mutation.** A Mycelium value cannot be mutably aliased. There is nothing in the
  value model that corresponds to `&mut T` because no value is mutable after construction
  (RFC-0006 LR-8). The standard Rust aliasing problem — the borrow checker exists to prevent
  `&mut T` and `&T` or two `&mut T` from co-existing — simply does not arise for Mycelium
  values.
- **No cycles.** Values are acyclic (RFC-0006 LR-9; RFC-0008 §3). There is no analogue of Rust's
  `Rc<RefCell<T>>` cycle problem; cycle-breaking GC is explicitly stated as outside scope (RFC-0027
  §3).
- **Representation is part of the type.** `Binary{8}` and `Ternary{6}` are different types; no
  implicit coercion, no "fat pointer" with a vtable (RFC-0001 §3.3; WF1).

### 2.2 Affine resources — `substrate` and `consume` (DN-02, DN-03, Glossary.md §2.18)

The one Mycelium surface construct with a linearity/affinity flavor is `substrate`: an affine
external resource consumed exactly once (Glossary.md §2.18; DN-02 §2 "Ratified"; DN-03 §1
adopts `consume` as its keyword). This is the language's counterpart to Rust's move semantics
applied to non-Copy types, but scoped specifically to *external resources* (file handles, network
connections, I/O capabilities). The analogy is close but not identical: in Rust, every non-Copy
type is move-semantics by default; in Mycelium, the base case is immutable value semantics, and
`substrate` is a narrower, explicitly-labelled category for things that have a single-use
external contract.

### 2.3 `wild` — the unsafe block analogue (DN-02 §5, RFC-0028, ADR-014, Glossary.md §2.21)

`wild` is Mycelium's lexically-marked escape hatch to raw FFI and foreign memory, analogous to
Rust's `unsafe` block. Unlike Rust's `unsafe`, which is a lint-softened permission, `wild` is
**denied by default** (DN-02 §5) — a `wild` block anywhere outside a `@std-sys`-annotated
nodule with `!{ffi}` effect is a hard `CheckError` (RFC-0028 §4.1). This is a *stricter* policy
than Rust's `unsafe`. ADR-014 records that the Mycelium Rust kernel itself uses a
permitted-but-warned policy (`unsafe_code = "warn"`), with per-site `// SAFETY:` justifications.
The Mycelium-language `wild` and the Rust-kernel `unsafe` are entirely separate layers.

### 2.4 Memory management — RFC-0027 (Draft, planning stub only)

RFC-0027 is a Draft planning stub with *no normative decisions*. It identifies the gap: the
current implementation relies on Rust's drop/ownership system for reclamation of runtime values,
which is correct but implicit. The RFC identifies five open questions (§5), none yet resolved:
whether reclamation is purely Rust-drop-order or exposes explicit "reclaim regions"; how
reclamation couples to the sweep-order (RFC-0008 §4.3); the scope of `reclaim`; whether latency
is a hard real-time bound; and the `cyst`/checkpoint interaction. All claims here are
`Declared` until RFC-0027 advances.

### 2.5 Runtime concurrency and structured lifetimes (RFC-0008)

RFC-0008 RT7 establishes that runtime lifetimes are structured: a scope does not exit until all
its children complete, are cancelled, or are detached. "In safe Mycelium a leaked task is not
expressible" (RFC-0027 §1). This gives the concurrency model a *scope-based* lifetime discipline
analogous to Rust's lexical lifetimes, but the mechanism is the structured-concurrency scope
tree, not a borrow checker. The analogy: Rust lifetimes prevent dangling references at compile
time; Mycelium structured lifetimes prevent dangling tasks at runtime.

### 2.6 FFI and the boundary (RFC-0028)

The FFI model decides that `wild` blocks lower to `Op { prim: "wild:…" }` using the existing
Core IR `Op` node — no new kernel node (KC-3). The capability model is build-time (`@std-sys`
gate); runtime-enforced capabilities are deferred. All FFI operations carry `Declared` guarantee
tags (RFC-0028 §4.6; VR-5). This is Mycelium's analogue of Rust's `extern "C"` + `unsafe`
blocks, but confined by a capability gate that Rust lacks.

### 2.7 Content-addressing and code identity (RFC-0001 §4.6)

Rust uses nominal typing (types are identified by their declaration path); Mycelium uses
content-addressing (types are identified by their structural hash, following Unison). This
difference is fundamental to the ownership mapping: Rust's lifetime annotations (`'a`) are part
of a nominal typing system where the borrow checker tracks named regions; Mycelium has no
analogous named-region mechanism because values are immutable and the language deliberately has
no references.

---

## 3. External Prior Art

### 3.1 Rust's borrow checker and its formal semantics

**RustBelt** ([paper](https://people.mpi-sws.org/~dreyer/papers/rustbelt/paper.pdf)) provides a
semantic interpretation of Rust's ownership types using the higher-order concurrent separation
logic Iris, proving type soundness for safe and unsafe Rust. The key insight is that `&T`
(shared reference) corresponds to a *read-only* assertion of resource ownership in separation
logic, while `&mut T` (mutable reference) corresponds to *exclusive* ownership. The Iris project
([iris-project.org](https://iris-project.org/)) has extended this to concurrent programs and,
in 2024, to verifying CompCert C programs via an Iris instance
([An Iris Instance for Verifying CompCert C Programs](https://iris-project.org/pdfs/2024-popl-vst-on-iris.pdf)).

**Stacked Borrows** ([POPL 2020](https://plv.mpi-sws.org/rustbelt/stacked-borrows/paper.pdf))
is an operational aliasing model for Rust's memory, tracking tags on a per-location stack. It
defines the aliasing discipline that both safe and unsafe Rust code must respect for the compiler
to preserve semantics. Key point for Mycelium: `transmute` in Rust is undefined behavior under
Stacked Borrows when it turns a shared reference into a writable raw pointer, because the tag
the borrow stack expects is not present. Mycelium's value-immutability sidesteps all of this.

**Place Capability Graphs** ([arxiv:2503.21691](https://arxiv.org/html/2503.21691v4), 2025) is
a 2025 model of Rust's type-checking results as capability graphs — a general-purpose model
designed to enable downstream tools to exploit ownership/borrow properties without re-running
the borrow checker.

### 3.2 Linear and affine type systems

The foundational theory for Mycelium's `substrate`/`consume` is **linear type theory** (Girard
1987), which requires every value to be used *exactly once*, and the slightly weaker **affine
type theory** (Wadler 1990), which requires values to be used *at most once*. Rust uses an
affine type system (non-Copy types may be dropped without explicit use). Mycelium's `substrate`
is affine in the same sense — a `substrate` is consumed exactly once, and not consuming it is an
error. Cornell's lecture notes on linear types
([CS 6110](https://www.cs.cornell.edu/courses/cs6110/2017sp/lectures/lec30.pdf) and
[CS 4110](https://www.cs.cornell.edu/courses/cs4110/2018fa/lectures/lecture29.pdf)) formalize
this precisely.

**Cyclone** ([Linear Regions Are All You Need](https://www.ccs.neu.edu/home/amal/papers/linrgn.pdf))
combined region-based memory management with linear/affine types, and directly influenced Rust's
lifetime+region design. Cyclone's explicit region annotations (`'a`) are the ancestor of Rust's
lifetime parameters. Mycelium deliberately omits both: because values are immutable and acyclic,
no region tracking is needed.

**Cogent** ([Cambridge Core](https://www.cambridge.org/core/journals/journal-of-functional-programming/article/cogent-uniqueness-types-and-certifying-compilation/47AC86F02534818B95A56FA1A283A0A6))
is a functional systems language with a *uniqueness type system* — stronger than affinity,
requiring that at each use point, only one reference to the value exists. Cogent uses uniqueness
types to eliminate GC, support in-place destructive update, and produce a certifying compiler
that generates C code with a refinement theorem. Uniqueness types are strictly stronger than
Mycelium's value semantics (which has no in-place mutation at all), but Cogent's certifying
compilation approach (producing a proof of C code correctness via translation validation) is
directly relevant to Mycelium's swap-certificate model (RFC-0002).

### 3.3 Pony reference capabilities

Pony's six **reference capabilities** ([Pony Tutorial](https://tutorial.ponylang.io/reference-capabilities/reference-capabilities.html))
— `iso`, `val`, `ref`, `box`, `trn`, `tag` — form a capability lattice that Pony uses to make
concurrent programs safe without locks. The capability subtype order is:
`iso → trn → ref, trn → val, ref → box, val → box → tag`. Key mappings relevant to Mycelium:

- Pony `val` (immutable, shareable across actors) is the closest Pony analogue to Mycelium's
  plain `Value` — immutable, freely shareable, no aliasing concerns.
- Pony `iso` (isolated, exclusively owned) corresponds most closely to Mycelium's `substrate`
  — an affine resource that can be transferred between actors.
- Pony `ref` (mutable, not shareable across actors) has no direct Mycelium analogue because
  Mycelium values are always immutable.
- Pony's `consume` keyword (explicit transfer of `iso`) is the direct ancestor of Mycelium's
  `consume` keyword for `substrate` (the naming is not coincidental; DN-03 §1 adopts `consume`
  through the three-test gate, and the behavioral meaning — "transfer exclusive ownership" — is
  identical).

The key difference: Pony has an `object` model with mutable state and a borrow/capability
system that manages that state across actors. Mycelium has no mutable object model at all —
it achieves the same safety goal (RT1: no shared mutable state) by eliminating the mutability,
not by tracking capabilities over it.

### 3.4 Separation logic and Iris

The Iris framework ([iris-project.org](https://iris-project.org/); lecture notes:
[cs.au.dk/~birke](https://cs.au.dk/~birke/papers/iris-lecture-notes.pdf)) provides a concurrent
higher-order separation logic. In Iris, `x ↦ v` asserts *exclusive ownership* of a heap cell —
only one thread holds this assertion at any time. This is the formal model behind `&mut T` in
Rust. Mycelium's value model makes a different choice: rather than a *heap-based* ownership
assertion, Mycelium ownership lives at the *language level* through immutability and structured
scoping.

### 3.5 Translation validation

**CompCert** ([Formal Verification of a Realistic Compiler, CACM](https://cacm.acm.org/research/formal-verification-of-a-realistic-compiler/))
is the gold standard for verified compilation. Translation validation — proving each
*compilation instance* correct — is the mature, proven pattern (RFC-0002 §2 adopts it
explicitly, citing CompCert/Valex/Crellvm). The 2024 paper
[Modeling Dynamic (De)Allocations of Local Memory for Translation Validation](https://arxiv.org/pdf/2403.05302)
extends translation validation to dynamic local memory allocation, which is relevant to
RFC-0027's eventual memory-model decisions.

---

## 4. The Mapping Table

Legend for Verdict column:
- **FAITHFUL** — Mycelium has a clean analogue; semantics are preserved or strengthened.
- **REENCODED** — Mycelium achieves the same safety property through a different mechanism (not
  the same construct, but the gap is intentional and the guarantee is not weaker).
- **NARROWED** — Mycelium has a partial analogue that covers a strict subset of the Rust use
  cases; the rest is deliberately out of scope.
- **BLOCKED-ON-RFC-0027** — the mapping depends on decisions not yet made in the Draft RFC-0027.
- **NEEDS-HUMAN** — the mapping cannot be resolved without a Mycelium language design decision;
  flagged here as an open question.
- **ABSENT-BY-DESIGN** — Mycelium has no analogue and this is a deliberate design choice
  grounded in the value-semantics position.

Guarantee tags on each verdict per VR-5: `Exact` where a structural proof is available;
`Empirical` where the corpus grounding is explicit but no formal proof exists; `Declared` where
the mapping is asserted from design intent with no mechanized check.

| # | Rust Construct | Mycelium Analogue | Verdict | Guarantee | Grounding |
|---|---|---|---|---|---|
| 1 | **Move semantics / ownership transfer** (non-Copy `T` moved out of scope) | Immutable value passed over a channel or returned from a function. Values are not "owned" in the Rust sense — they are immutable and freely shareable once created; exclusive access is not required because there is no mutation. | REENCODED | Empirical | RFC-0001 §3.1 (immutable); RFC-0008 RT1 (values cross boundaries); LR-8 (no aliased mutation). Empirical: grounded in corpus but no formal proof of equivalence. |
| 2 | **`&T` — shared reference (read-only borrow)** | No surface construct; corresponds to ordinary value usage. In Mycelium, *every* value binding is effectively a shared read-only reference because values are immutable — there is no distinction between "the value" and "a shared reference to the value." | REENCODED | Empirical | RFC-0001 §3.1 (immutability by construction); RFC-0008 RT1. The Rust distinction between `T` (owned) and `&T` (borrowed) collapses in Mycelium. |
| 3 | **`&mut T` — exclusive mutable reference** | No analogue in safe Mycelium. Mutable references exist in the Rust kernel implementation, not in the Mycelium language surface. Inside `wild` blocks, the underlying Rust host layer may hold `&mut T` to foreign buffers, but this is opaque to Mycelium programs. | ABSENT-BY-DESIGN | Exact | RFC-0006 LR-8 (no aliased mutation is the design position, not a gap); RFC-0028 §4.2 (FFI / `wild` is opaque). |
| 4 | **Lifetimes `'a` — borrow region annotations** | No surface lifetime annotations. Mycelium achieves the same "no dangling reference" property through: (a) immutability — a value cannot be mutated through an alias, so dangling-reference bugs are a narrower category; (b) structured concurrency scopes (RFC-0008 RT7) — a scope outlives all its children, so a child cannot hold a reference to a value that goes out of scope before it. These are runtime structural guarantees, not compile-time borrow-region annotations. | REENCODED | Empirical | RFC-0008 RT7 (structured lifetimes); RFC-0006 LR-8/LR-9 (immutable + acyclic). Empirical: the scope-tree guarantee is corpus-stated but no formal proof of equivalence to Rust's lifetime system is in-repo. |
| 5 | **`Box<T>` — heap allocation, unique ownership** | No surface `Box`. Heap allocation is an implementation detail invisible to Mycelium programs (RFC-0027 §3 notes the current runtime relies on Rust's ownership + drop system; RFC-0027 §5 leaves the allocator out of scope). Values live wherever the runtime places them; the Mycelium type carries no heap-allocation annotation. | REENCODED | Declared | RFC-0027 §3 (current implementation), §5 (allocator = out of scope). Declared: RFC-0027 is a Draft stub with no normative decisions. |
| 6 | **`Rc<T>` — reference-counted shared ownership (single-threaded)** | No analogue. Reference counting is an *implementation mechanism* for reclamation; Mycelium programs do not express reclamation strategy. The runtime may choose reference counting internally (BLOCKED-ON-RFC-0027), but it is not surface-visible. | BLOCKED-ON-RFC-0027 | Declared | RFC-0027 §5 OQ (model choice between Rust-drop-order and explicit reclaim regions is open). |
| 7 | **`Arc<T>` — atomically reference-counted shared ownership (multi-threaded)** | No analogue (same reasoning as `Rc<T>`). The runtime concurrency model (RFC-0008) says values cross hypha/channel boundaries; the reclamation mechanism for values that have crossed boundaries is an RFC-0027 open question (§5: "ownership transfer at hypha/channel boundaries"). | BLOCKED-ON-RFC-0027 | Declared | RFC-0027 §1 ("the reclamation model must be decided"), §4 DoD item 1 (ownership transfer at boundaries). |
| 8 | **Interior mutability — `Cell<T>`, `RefCell<T>`, `Mutex<T>`, `RwLock<T>`** | No analogue in the Mycelium language. The entire concept of interior mutability (mutating through a shared reference) is outside Mycelium's value model. Immutability is not just "the default" — it is constitutive of `Value`. There is no `Cell`-equivalent because there is no mutable location. In safe Mycelium, shared mutable state simply does not exist (RFC-0008 §2: "The single hardest problem of concurrent runtimes — shared mutable state — *does not exist in this language*"). | ABSENT-BY-DESIGN | Exact | RFC-0006 LR-8 (no aliased mutation — the design position); RFC-0008 §2/RT1. |
| 9 | **`Drop` trait — deterministic destructor** | Partial analogue via the `reclaim` runtime construct (reserved, not yet active). `reclaim` is for *stale runtime units* (hyphae/tasks), not for individual values (DN-03 §4 clarifies: "`reclaim` reclaims stale runtime units, **never memory** — LR-9 makes memory reclamation automatic"). The exact mechanism of value-level reclamation is BLOCKED-ON-RFC-0027. | NARROWED + BLOCKED-ON-RFC-0027 | Declared | DN-03 §4 (scope of `reclaim`); RFC-0027 §1 (the gap: current implementation uses Rust drop implicitly); RFC-0027 §5 OQ1 (drop-order vs. explicit reclaim regions). |
| 10 | **`unsafe` block — permission to bypass type/memory safety** | `wild` block — the denied-by-default FFI escape. Key differences: (a) `wild` is stricter than `unsafe` — denied by default, requires capability annotation (`@std-sys` + `!{ffi}`), restricted to a single-host-call body grammar; (b) `unsafe` in Rust is a lint-escalated warning (ADR-014 for the Mycelium kernel); (c) the Mycelium language `wild` and the Rust-kernel `unsafe` are separate layers. | FAITHFUL (semantics preserved; `wild` is strictly stronger) | Empirical | DN-02 §5 (Ratified; wild = denied-by-default); RFC-0028 §4.1 (capability model); ADR-014 (Rust-kernel `unsafe` policy). Empirical: the capability model is normative but not formally verified. |
| 11 | **Raw pointers `*const T`, `*mut T`** | No surface raw pointers in Mycelium. Raw-pointer-level operations are confined to `wild` blocks (FFI escape) and to the Rust kernel implementation (ADR-014, ADR-007). The Mycelium language has no pointer type in its value model — values are content-addressed aggregates, not memory locations. | ABSENT-BY-DESIGN | Empirical | RFC-0001 §3.1 (the value model has no pointer type); RFC-0028 §4.2 (FFI body grammar: host-call form only, not pointer arithmetic); ADR-014 (raw pointers stay in the Rust kernel). |
| 12 | **`transmute` — unchecked type reinterpretation** | The explicit `swap` construct (RFC-0001 §4.5 `Swap` node; RFC-0002). Key differences: (a) `transmute` is unchecked and produces undefined behavior under Stacked Borrows if aliasing is violated; Mycelium's `swap` carries a certificate and is never silent (RFC-0001 WF1/WF2); (b) a `swap` with no statable bound is a type error, not just UB (RFC-0002 §5 legal-pair table); (c) the guarantee lattice records exactly what the swap cost — `Exact` for lossless, `Bounded` for lossy, explicit error for impossible pairs. | FAITHFUL (and safer than `transmute`) | Empirical | RFC-0001 WF1 (every Repr change is a `Swap`); RFC-0002 §4 (bijection semantics), §5 (legal pair table). Empirical: the swap certificate is normative; formal verification of the certificate checker is future work (RFC-0002 §7). |
| 13 | **Copy types — `Copy` trait, bitwise copy** | Structural value sharing. In Mycelium, every value is immutable, so "sharing" a value is always safe — the value cannot be modified through any reference. The distinction between `Copy` (cheap bitwise copy, multiple bindings valid) and non-`Copy` (moved, one binding at a time) collapses: all Mycelium values behave like `Copy` from a safety standpoint, but the runtime may optimize by reference-counting or passing pointers internally (implementation detail, invisible to programs). | REENCODED | Empirical | RFC-0001 §3.1 (immutability); RFC-0008 RT1 (values cross boundaries safely). The runtime optimization question is Declared pending RFC-0027. |
| 14 | **`Pin<T>` — pinning values in memory (no-move guarantee)** | No analogue. `Pin` exists in Rust to support self-referential types and async state machines that cannot be moved. Mycelium's async/concurrent model (RFC-0008) uses structured concurrency with content-addressed `cyst` checkpoints — there are no self-referential types (values are acyclic, LR-9) and no in-place async state machines with pinned references. The need for `Pin` does not arise. | ABSENT-BY-DESIGN | Declared | RFC-0006 LR-9 (acyclic values); RFC-0008 §4.4 (`cyst` checkpointing — values-plus-continuation, not self-referential state). Declared: no formal proof that the `cyst` model eliminates all `Pin`-equivalent needs. |
| 15 | **Lifetime bounds on trait objects `dyn Trait + 'a`** | No analogue. Mycelium's trait system (RFC-0019) is parametric polymorphism over the content-addressed type system; there are no dynamic dispatch objects with lifetime parameters because: (a) all values are immutable (no need to track aliasing lifetime); (b) the content-addressed type system uses structural hashing, not nominal lifetime variables. | ABSENT-BY-DESIGN | Declared | RFC-0019 (Accepted — traits and parametric polymorphism); RFC-0001 §4.6 (content-addressing). Declared: RFC-0019 is Accepted but not yet fully enacted; the interaction with lifetime elimination is stated design intent. |
| 16 | **`static` lifetime — `'static` bound** | Partial analogue: a `matured` nodule or phylum (RFC-0017) contains compiled-and-frozen definitions that are stable across executions. A `matured` definition is the closest analogue to a Rust `'static` binding — it is durable, not tied to any particular runtime scope. However, `matured` is a *scope* qualifier on a compiled unit, not a type-level lifetime parameter. | NARROWED | Empirical | RFC-0017 (`matured` scope definition); Glossary.md §2.10. The mapping is close but not exact — Rust's `'static` spans the whole program lifetime; Mycelium's `matured` spans a compiled artifact's stable identity. |

---

## 5. Faithful Subset vs. Residue Analysis

### 5.1 The faithful subset (6 items — FAITHFUL or REENCODED with strong grounding)

These are constructs where Mycelium's value-semantics position achieves the same safety goal as
Rust's ownership system, through a different (often simpler) mechanism:

| # | Rust construct | Mechanism of faithfulness |
|---|---|---|
| 1 | Move semantics | Replaced by immutability — no move needed when there is nothing to mutate. |
| 2 | `&T` shared reference | Collapsed into ordinary value binding — all Mycelium values are implicitly shared-read-only. |
| 4 | Lifetimes `'a` | Replaced by structured-concurrency scopes (RT7) — the scope tree is the runtime lifetime manager. |
| 10 | `unsafe` block | `wild` block — faithful analogue but with stricter capability gating. |
| 12 | `transmute` | Explicit `swap` — faithful but safer (certified, never silent, illegal pairs are type errors). |
| 13 | `Copy` trait | Immutability makes all values safely shareable without an explicit `Copy` marker. |

### 5.2 The residue — where gaps live

The residue groups into three categories:

#### 5.2.1 Value-semantics re-encoding (deliberate design; well-grounded)

These items have no Mycelium surface analogue *because the value-semantics position eliminates
the need*:

- **`&mut T`** (row 3): No mutable references = no aliased mutation = the borrow checker's core
  problem does not arise.
- **Interior mutability** (row 8): `Cell`/`RefCell`/`Mutex` exist to allow mutation through
  shared ownership; with no mutable values, they are not needed.
- **Raw pointers** (row 11): No pointer type in the value model; FFI pointers are confined to
  `wild` blocks.
- **`Pin<T>`** (row 14): No self-referential types (acyclic by LR-9) → no need to pin.
- **`dyn Trait + 'a`** (row 15): No mutable aliasing → no lifetime parameters on trait objects.

These are not gaps — they are the intended consequence of LR-8/LR-9/RFC-0008 RT1.

#### 5.2.2 Runtime-model constructs (BLOCKED-ON-RFC-0027)

These are items where Mycelium's position differs from Rust's but the precise mechanism is *not
yet decided* — it depends on RFC-0027's outcome:

- **`Box<T>`** (row 5): The heap-allocation strategy is an unresolved implementation question
  (RFC-0027 §5 OQ: "explicit reclaim regions" vs. "Rust drop-order").
- **`Rc<T>`** (row 6): Reference counting as an internal reclamation strategy is an open model
  question.
- **`Arc<T>`** (row 7): Same as `Rc<T>`, with the added question of ownership transfer at
  hypha/channel boundaries (RFC-0027 §1/DoD-1).
- **`Drop` trait** (row 9): The `reclaim` keyword is reserved but not active; the interaction
  between scope-exit, reclamation, and the sweep-order (RFC-0008 §4.3) is RFC-0027's core
  decision.

Until RFC-0027 resolves these questions, the mapping for rows 5–7 and 9 remains `Declared`.

#### 5.2.3 Narrowed analogues (partial mapping, acknowledged)

- **`substrate`/`consume`** (not in the table above — this is a Mycelium construct with no
  single Rust row; it maps to Rust's non-Copy move semantics *for external resources only*).
  The subset difference: Rust applies affinity to *all* non-Copy types; Mycelium applies it
  only to explicitly-labelled `substrate` resources. The rest of the language is value-semantics
  (effectively Copy).
- **`'static` vs. `matured`** (row 16): `matured` is a compiled-scope notion, not a type-level
  lifetime. The analogy is meaningful but inexact.

---

## 6. Open Research Questions

**OQ-A1** (`NEEDS-HUMAN`): Does Mycelium need any form of *compile-time* ownership/borrow
checking for `substrate` values, or is the current typed-effect annotation (`!{ffi}` +
`@std-sys`) sufficient? Rust's borrow checker prevents use-after-move at compile time; Mycelium's
current model relies on the single-use property being enforced by the type system for `substrate`
but no explicit borrow checker is specified. The design decision is open.

**OQ-A2** (`NEEDS-HUMAN`, BLOCKED-ON-RFC-0027): Should Mycelium expose a surface-level
*reclamation construct* (analogous to Rust's `Drop`) for non-FFI use cases, or is the structured
concurrency scope tree (RT7) sufficient as the sole lifetime manager? RFC-0027 §5 OQ1 asks this
directly.

**OQ-A3** (`NEEDS-HUMAN`, BLOCKED-ON-RFC-0027): The `cyst`/checkpoint model (RFC-0008 §4.4)
serializes a "values-plus-continuation" into content-addressed storage. If a `cyst` holds a
`substrate`, does the checkpoint *consume* the substrate (checkpoint-and-free, RFC-0027 §5 OQ5)?
This is the resource-crossing-a-checkpoint problem, analogous to Rust's `!Send` and `!Sync`
bounds.

**OQ-A4** (Research): Cogent's uniqueness type system ([Cambridge Core](https://www.cambridge.org/core/journals/journal-of-functional-programming/article/cogent-uniqueness-types-and-certifying-compilation/47AC86F02534818B95A56FA1A283A0A6))
provides a certifying compiler for C via translation validation — the same general approach as
RFC-0002's swap certificate. Does Cogent's refinement theorem provide a usable template for
RFC-0027's reclamation model, or is the value-semantics position (acyclic, immutable) sufficient
to use a simpler approach than Cogent's full uniqueness tracking?

**OQ-A5** (Research): Place Capability Graphs ([arxiv:2503.21691](https://arxiv.org/html/2503.21691v4))
(2025) propose a general model of Rust's type-checking results that can enable downstream tools.
Could a subset of this model be used to specify the `substrate`/`consume` borrow discipline in
Mycelium, providing a compile-time checker for `substrate` affinity without importing the full
Rust borrow checker?

**OQ-A6** (Research, BLOCKED-ON-RFC-0027): The Spegion language ([arxiv:2506.02182](https://arxiv.org/pdf/2506.02182))
(2025) proposes "implicit and non-lexical regions with sized allocations" — a region-based memory
model that does not require explicit lifetime annotations. If RFC-0027 decides to expose
"reclaim regions" (RFC-0027 §5 OQ1 option B), Spegion's implicit-region approach may be
relevant as an alternative to explicit Rust-style lifetime parameters.

---

## 7. Summary: The Highest-Value Finding

**The `&mut T` / interior-mutability cluster is absent-by-design, not a gap.** The six Rust
constructs that exist primarily to manage mutable aliasing (`&mut T`, `Cell`, `RefCell`, `Mutex`,
`RwLock`, raw pointers, `Pin`) have no Mycelium surface analogues — and this is the *intended
consequence* of the value-semantics position (LR-8/LR-9/RT1), not an oversight. The borrow
checker's core reason for existence is to prevent two `&mut T` from co-existing, or `&T` and
`&mut T` from co-existing; Mycelium eliminates this by making all language-level values
immutable. This is the same property that Erlang/OTP achieves via message-passing and pure
functions — RFC-0008 §2 explicitly cites this grounding (T4.1/T4.5).

The *actual* open design work — the RFC-0027 cluster (rows 5, 6, 7, 9) — is about *reclamation*
mechanics (when and how memory backing values is recovered), not about *aliasing safety* (which
is already resolved by immutability). This is a much narrower problem than Rust's borrow checker
solves, and the structured-concurrency scope tree (RT7) already provides the structural
guarantee that no scope outlives its children, giving the reclamation model a clean hook.

---

## Meta — Document info

- **Confidence overall:** `Empirical` on corpus-grounded mappings; `Declared` on RFC-0027-blocked items.
- **Grounding basis:** RFC-0001, RFC-0002, RFC-0006, RFC-0007, RFC-0008, RFC-0027, RFC-0028, RFC-0019,
  ADR-014, ADR-032, DN-02, DN-03, Glossary.md; external: RustBelt, Stacked Borrows, Cogent,
  Pony reference capabilities, Iris framework, Cyclone, Place Capability Graphs, Spegion.
- **External citations (full list):**
  - [RustBelt: Securing the Foundations of the Rust Programming Language](https://people.mpi-sws.org/~dreyer/papers/rustbelt/paper.pdf)
  - [Stacked Borrows: An Aliasing Model for Rust (POPL 2020)](https://plv.mpi-sws.org/rustbelt/stacked-borrows/paper.pdf)
  - [Place Capability Graphs: A General-Purpose Model of Rust's Ownership and Borrowing Guarantees (2025)](https://arxiv.org/html/2503.21691v4)
  - [Functional Ownership through Fractional Uniqueness](https://arxiv.org/pdf/2310.18166)
  - [A Hybrid Approach to Semi-Automated Rust Verification](https://arxiv.org/html/2403.15122v1)
  - [Cogent: Uniqueness Types and Certifying Compilation](https://www.cambridge.org/core/journals/journal-of-functional-programming/article/cogent-uniqueness-types-and-certifying-compilation/47AC86F02534818B95A56FA1A283A0A6)
  - [Pony Reference Capabilities Tutorial](https://tutorial.ponylang.io/reference-capabilities/reference-capabilities.html)
  - [Iris Project](https://iris-project.org/)
  - [An Iris Instance for Verifying CompCert C Programs (POPL 2024)](https://iris-project.org/pdfs/2024-popl-vst-on-iris.pdf)
  - [Iris Lecture Notes: Higher-Order Concurrent Separation Logic](https://cs.au.dk/~birke/papers/iris-lecture-notes.pdf)
  - [Linear Regions Are All You Need (Cyclone)](https://www.ccs.neu.edu/home/amal/papers/linrgn.pdf)
  - [CS 6110 Lecture 30: Linear Type Systems](https://www.cs.cornell.edu/courses/cs6110/2017sp/lectures/lec30.pdf)
  - [Modeling Dynamic (De)Allocations of Local Memory for Translation Validation](https://arxiv.org/pdf/2403.05302)
  - [Formal Verification of a Realistic Compiler (CompCert, CACM)](https://cacm.acm.org/research/formal-verification-of-a-realistic-compiler/)
  - [Spegion: Implicit and Non-Lexical Regions with Sized Allocations (2025)](https://arxiv.org/pdf/2506.02182)
  - [Uniqueness is Separation (2025)](https://arxiv.org/pdf/2602.06386)
  - [Value Semantics | Modular/Mojo](https://docs.modular.com/mojo/manual/values/value-semantics/)
- **Append-only; do not edit — supersede with a new handoff document if conclusions change.**
