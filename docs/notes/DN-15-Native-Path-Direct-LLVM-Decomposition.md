# Design Note DN-15 — Native-Path Direct-LLVM Decomposition

| Field | Value |
|---|---|
| **Note** | DN-15 |
| **Status** | **Draft** (2026-06-19; M-373) |
| **Feeds** | RFC-0004 (execution model — §2 revisit clause, §6 inspectability); ADR-009 (no-opaque-lowering, all backends); DN-05 #1 (stack-robustness design requirement; DepthBudget trait); M-348 (libMLIR provisioning — the remaining block); M-373 (this task) |
| **Date** | June 19, 2026 |
| **Decides** | *Design note, not a ratified decision.* Records the honest decomposition of M-348 into a libMLIR-gated half (real ternary dialect lowering; stays blocked) and a direct-LLVM-advanceable half (llvm.rs data-fragment extension; sanctioned by RFC-0004 §2 revisit clause). Presents the strategy and per-increment risk table. No spec is promoted to Accepted here. |
| **Task** | M-373 — Wave-5 native-decomp research/design |

> **Posture (honesty rule / VR-5).** Every claim below is grounded in source code or a cited spec.
> No increment is pre-declared buildable without checking that the toolchain and preconditions are
> present. The libMLIR-gated half is **not buildable here** — stated, not hidden. The
> direct-LLVM-advanceable half is buildable **now** (LLVM 18 tooling present; `llc`/`clang`
> reachable) but carries the risk of textual LLVM IR being more brittle than MLIR dialects for
> complex data layouts; that risk is called out per-increment.

---

## 1. Background: two distinct blockers in M-348

M-348 is titled "Provision libMLIR to unblock the native MLIR→LLVM path" and is labelled
`status:blocked`. Its body bundles two distinct concerns that have **different unblock conditions**:

1. **Real ternary-dialect lowering** — the `ternary` → `arith`/`vector` → LLVM dialect chain,
   which requires a live libMLIR binding in this environment. This is the half that is truly
   libMLIR-gated (`crates/mycelium-mlir/src/dialect.rs` lines 1–6: "no libMLIR binding; it stands
   in for the real ternary → arith/vector → LLVM lowering, which is deferred").

2. **Direct-LLVM data/closure/recursion codegen** — extending `crates/mycelium-mlir/src/llvm.rs`
   (the *textual* LLVM IR backend) beyond its current bit/trit subset to handle
   `Construct`/`Match`/`App`/`Lam`/`Fix`/`FixGroup` nodes. These nodes are refused at
   `llvm.rs:263–274` with `AotError::UnsupportedNode(...)` and run on the AOT env-machine
   (M-342) instead. This half **does not need libMLIR** — it needs only the LLVM 18 tooling
   already present (`llc`/`clang`).

This note decomposes those two halves explicitly, establishes what can advance now, and documents
the strategy and constraints for the direct-LLVM half.

---

## 2. The libMLIR-gated half (stays blocked — VR-5)

**Scope:** `crates/mycelium-mlir/src/dialect.rs` — the textual ternary-dialect emitter.

`dialect.rs` lines 1–6 state its honest scope: "a TEXTUAL ternary-dialect emitter, 'no libMLIR
binding; it stands in for the real `ternary` → `arith`/`vector` → LLVM lowering, which is
deferred.'" The `emit()` function produces a human-readable MLIR-*style* module for inspection
and differential testing, but it is **not compiled** — there is no libMLIR link to lower it to
actual LLVM IR and thence to machine code.

**Why it stays blocked:** A real ternary → arith/vector → LLVM lowering requires linking against
libMLIR (the C++ MLIR library), creating type objects, building an MLIRContext, registering
dialects, and invoking the lowering pass pipeline — none of which can be done purely in Rust
without the FFI binding. The textual emitter in `dialect.rs` is a faithful **skeleton** of what
that pipeline will do (every stage is dumpable/inspectable — RFC-0004 §6), but it is not the
pipeline itself.

**VR-5 hold:** Every verdict about the ternary MLIR dialect lowering (correctness, performance,
cross-target codegen) stays honestly "not established" until libMLIR is provisioned (M-348). No
claim about this half will be upgraded without the toolchain present.

---

## 3. The direct-LLVM-advanceable half (sanctioned by RFC-0004 §2)

**Scope:** `crates/mycelium-mlir/src/llvm.rs` — the direct textual LLVM IR backend.

RFC-0004 §2 (the backend decision) contains an explicit revisit clause (line 21):

> *"Revisit if: a tiny stable substrate set + modest perf needs would favor a lighter direct-LLVM
> backend."*

The current situation fits: the `llvm.rs` backend is already a live, compiled, differentially
tested direct-LLVM backend for the bit/trit subset (M-301; exercised by the M-302 differential).
`llc` + `clang` are present; no libMLIR is needed. Extending it to handle the data-fragment
(`Construct`/`Match` first, closures/recursion later) is a sanctioned increment of this revisit
clause — it does not supersede the MLIR→LLVM commitment, it fills in the gap while that path
remains libMLIR-blocked.

**What the refusal site says (source, not paraphrase):** `llvm.rs:258–274`:

```
// The native LLVM backend stays the **bit/trit subset** (VR-5): the data + recursion
// fragment (Construct/App/Lam/Fix/FixGroup/Match) needs heap/closure codegen, deferred to
// the MLIR→LLVM backend (RFC-0004 §2). It runs on the `aot::run` env-machine instead — the
// path the three-way differential exercises for these nodes. Explicit refusal, never a
// silent mis-lowering (G2).
Rhs::Construct { .. }
| Rhs::App { .. }
| Rhs::Lam { .. }
| Rhs::Fix { .. }
| Rhs::FixGroup { .. }
| Rhs::Match { .. } => {
    return Err(AotError::UnsupportedNode(
        "data/recursion node (Construct/App/Lam/Fix/FixGroup/Match): the native LLVM \
         subset is bit/trit only; these run on the AOT env-machine (M-342), native \
         codegen deferred to the MLIR→LLVM backend"
            .to_owned(),
    ));
}
```

The comment and error text are the authoritative record. This note does not restate what the code
does — it cites the code and reasons about the next step.

**Inspectability constraint (RFC-0004 §6 / ADR-009):** Every stage must remain dumpable/diffable
(SC-4); each pass preserves `Meta` (WF5); no-opaque-lowering applies to all backends (ADR-009).
The textual LLVM IR emitter satisfies this now — `emit_llvm_ir()` produces one op per output
element, nothing is opaque (RFC-0004 §6, cited in the `llvm.rs` module doc). Any data-fragment
extension must hold the same standard: every `Construct`/`Match` lowering step is inspectable
and dumpable.

---

## 4. Strategy for the direct-LLVM data path

### 4.1 Increment 1 (this wave — M-373 scope): non-recursive Construct/Match only

**What:** Lower `Rhs::Construct` (allocate a tagged struct in textual LLVM IR; write the
constructor tag + field words) and `Rhs::Match` (branch on the tag; bind fields). No closures,
no lambdas, no `App`, no heap recursion, no `Fix`/`FixGroup`.

**Why non-recursive first:** Construct/Match over heap structs in textual LLVM IR is
self-contained and does not require a calling convention for closures or a trampoline for
recursion. The IR is straight-line within each arm (after the tag-branch), which keeps the
emitter auditable and the differential straightforward. This is the only increment where the
risk of textual-IR fragility is manageable without additional infrastructure.

**Heap representation (textual LLVM IR):** Tagged structs allocated with `@malloc` (or a
bump-allocator shim). Each constructor `Ctor { tag: u32, arity: usize }` becomes an `i64*`
pointer to a heap block: `[i64 tag, i64 field_0, i64 field_1, …]`. Fields are `!myc.value`
opaque words (i64 slots). No GC for the first increment — the program is straight-line and the
heap is arena-freed on exit. Out-of-memory is an explicit `AotError::Run` (never silent; G2).

**Match lowering:** Read the tag word (`load i64`), emit an LLVM `switch i64` over the known
constructor tags, bind field words from the struct, continue in each arm. No-match (missing
default when the discriminant is exhausted by patterns) is a compile-time `AotError::UnsupportedNode`
explaining the gap — never a silent fallthrough.

**Inspectability:** Every `switch` arm and field-load is explicit IR; the full module is
dumpable. `Meta` is preserved on the returned `Value` as `Declared` (the tag-dispatch is a
Declared match strategy, not a Proven one — VR-5).

**Guarantee tag:** `Declared` (not `Proven` — the textual IR layout and tag assignment are
declared conventions, not formally verified; the differential against the AOT env-machine
provides the Empirical check once tests land).

### 4.2 Increment 2 (deferred — NEXT wave): closures (App/Lam) + heap

**What:** Lower `Rhs::Lam` as a heap-allocated closure record (function pointer + captured
environment) and `Rhs::App` as an indirect call through the closure pointer.

**Technique (closure-conversion):** Standard closure conversion — each `Lam { param, body }`
becomes a heap struct `[fn_ptr: i64 (fn pointer), env: i64* (captured bindings)]`; `App { func,
arg }` loads the function pointer and calls it with the environment and argument. The conversion
must be explicit (no opaque transform) and each step dumpable.

**Why deferred:** Closure conversion requires a multi-pass transform over the ANF (free-variable
analysis + environment packing), which is substantially more code than Construct/Match. The risk
of textual-IR fragility is higher for indirect calls; the differential is harder to exercise
without a corpus of closure-heavy programs. This earns its own increment and its own differential
extension.

**Guarantee tag when it lands:** At most `Empirical` (differential-tested against the AOT
env-machine) until a stronger basis is established.

### 4.3 Increment 3 (deferred — libMLIR-gated or later): Fix/FixGroup + stack-robustness

**What:** Lower `Rhs::Fix`/`Rhs::FixGroup` (recursive definitions) in textual LLVM IR with an
explicit heap control stack (not the C call stack) so that deep recursion is a graceful limit,
never a SIGSEGV.

**Stack-robustness binding (DN-05 #1):** DN-05 §1 item #1 is the normative requirement:

> "The native backend **must** execute object-level recursion **without an unbounded C stack**
> — a managed/segmented or heap-spilled call stack with an **explicit depth/budget limit** (a
> graceful error, never a SIGSEGV/abort; G2)."

DN-05 §2.4 and DN05-Q4/Q5 define the `DepthBudget`/`StackPolicy` trait for the env-machine's
heap control stack; DN05-Q5 is Resolved (M-349) with `AutoDepthBudget`. For the native path,
the same trait is the right reuse point: the native codegen emits an explicit heap-allocated
continuation stack in LLVM IR (iterative trampoline, not C recursion), and the depth budget is
the same `DepthBudget` abstraction — same interface, different backing mechanism
(`llvm.rs`-emitted trampoline instead of the `aot.rs` Rust control stack).

**Why deferred:** The direct-LLVM trampoline for recursion is significantly more complex than
the env-machine trampoline (the env-machine trampoline rewrites Rust evaluation; the native
trampoline requires emitting continuation-passing IR in LLVM text). This should not land in the
same increment as Construct/Match — the risk of a subtle mis-lowering would make the differential
harder to trust. DN-05 #1 explicitly flags this as a design requirement to be designed in, not
retrofitted; designing it into Increment 3 (not into Increment 1) is the honest sequencing.

**Guarantee tag when it lands:** `Declared` → `Empirical` (differential) → at most `Proven` if
a formal argument is constructed. Never pre-written.

### 4.4 What remains libMLIR-gated (permanent block until M-348)

- The real `ternary` → `arith`/`vector` → LLVM dialect pipeline (`dialect.rs` — the textual
  skeleton cannot be compiled without libMLIR).
- Cross-target codegen for non-host triples (RFC-0004 §9.3: "build is host-target only" until
  the native libMLIR/LLVM backend lands).
- The ternary-dialect inspectability at the MLIR pass level (currently the textual emitter
  provides the dump, but the actual passes are not present).

---

## 5. Per-increment summary table

| Increment | Description | Needs libMLIR? | Tractable in textual LLVM IR now? | Risk |
|---|---|---|---|---|
| **0 — bit/trit subset** | `core.id`, `bit.not/and/or/xor`, `trit.neg/add/sub/mul` | No — already shipped (M-301) | Yes — `llvm.rs` is live | Low (done) |
| **1 — non-recursive Construct/Match** | Tagged-struct alloc + switch-on-tag; straight-line arms; no closures, no recursion | No — textual LLVM IR only | **Yes** (this wave; M-373 design scope) | Low-medium: heap alloc in textual IR is straightforward; the differential against the env-machine is the guard |
| **2 — closures (App/Lam) + heap** | Closure-conversion + indirect call through heap struct | No — but requires free-var analysis pass | Yes in principle, deferred | Medium: closure conversion is a multi-pass transform; textual-IR indirect calls are brittle to mis-encode |
| **3 — recursion (Fix/FixGroup) + stack-robustness** | Iterative trampoline in LLVM IR; explicit heap control stack; DepthBudget trait reused | No — but complex IR emission | Tractable in principle, deferred | High: emitting a correct trampoline in textual IR is error-prone; DN-05 #1 requires no unbounded C stack (G2) — must be designed, not retrofitted |
| **4 — real ternary MLIR dialect lowering** | `ternary` → `arith`/`vector` → LLVM via libMLIR | **Yes — libMLIR-gated** | No — `dialect.rs` is a textual skeleton only | Blocked on M-348; every verdict stays `not established` (VR-5) |

**Column definitions:**
- *Needs libMLIR?* — whether the increment requires a live libMLIR binding (C++ FFI, not just
  LLVM toolchain).
- *Tractable in textual LLVM IR now?* — whether the LLVM 18 toolchain (`llc`/`clang`) present
  in this environment is sufficient, given current `llvm.rs` infrastructure.
- *Risk* — low = straightforward extension of existing pattern; medium = new infrastructure
  needed; high = correctness is hard to check without formal backing or the approach is novel.

---

## 6. What the orchestrator/1B agent needs to know

- **Increment 1 is this wave's scope** — the Construct/Match codegen design and any Rust
  implementation land here (or in sibling leaf 1B if the orchestrator assigned implementation
  to that agent).
- **The `DepthBudget` trait (`crates/mycelium-mlir/src/budget.rs`, M-349) is reusable** for
  Increment 3; the native codegen should share the trait rather than re-invent the abstraction
  (DRY, KC-3).
- **RFC-0004 §11 (new section, r3) is the append-only record of this sanction** — see
  RFC-0004 revision below.
- **The orchestrator must register DN-15 in `docs/Doc-Index.md`** (orchestrator-owned file; this
  leaf agent does not edit it — FLAG).
- **The orchestrator must add M-373 to `tools/github/issues.yaml`** (orchestrator-owned file;
  FLAG).
- **No code in this note** — design-first; the implementation (if any) lands in the leaf or the
  next wave as a separate task.

---

## Meta — changelog

<!-- changelog: 2026-06-19 Draft created (M-373) — records the libMLIR-gated vs direct-LLVM-advanceable decomposition, the Increment-1 (Construct/Match) design strategy, the DN-05 #1 DepthBudget reuse plan for Increment 3, and the per-increment risk table. Append-only. -->
