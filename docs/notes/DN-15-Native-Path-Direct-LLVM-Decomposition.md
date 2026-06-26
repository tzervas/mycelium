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

**Why non-recursive first:** Construct/Match in textual LLVM IR is self-contained and does not
require a calling convention for closures or a trampoline for recursion. The IR is straight-line
within each arm (after the tag-branch), which keeps the emitter auditable and the differential
straightforward. This is the only increment where the risk of textual-IR fragility is manageable
without additional infrastructure.

**Stack representation (textual LLVM IR — as landed):** Tagged structs allocated with **stack
`alloca [N+1 x i64]`** (not `@malloc`). Each constructor becomes an `[N+1 x i64]*` alloca
pointer: slot 0 holds the tag `i64`; slots 1..N hold field elements consecutively, one `i64` per
element. **Rationale for stack vs heap:** the non-recursive/bounded restriction (no
`Fix`/`FixGroup` in scope) means all allocation depth is statically fixed at codegen time —
there is no possibility of unbounded stack growth, so a heap allocation and an explicit OOM
failure path (`AotError::Run`) are unnecessary. `alloca` is simpler, needs no GC, has zero OOM
path, and the emitted IR is directly auditable without a malloc/free trace.

**Match lowering:** Read the tag word (`load i64` from slot 0 of the alloca), emit an LLVM
`switch i64` over the known constructor tags, bind field elements from the alloca (one `load i64`
+ `trunc i64→i32` per element), continue in each arm. When an ANF default arm is present (`Some`),
it is lowered into the switch's default block and its result merged via phi — matching the
reference interpreter's semantics exactly (no silent divergence; G2). When no ANF default is
present (`None`), the switch default emits an explicit `abort()` — a defined-trap, never raw
`unreachable` UB (G2; WF7 checker proves exhaustive coverage in this case).

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

> **Erratum (2026-06-25, post corpus-alignment audit; cross-ref §9, which already reconciles this).** The
> "permanent block until M-348" framing here and in §2 is **partly superseded**: a **real**
> `ternary`→`arith`/`func`→LLVM dialect lowering **landed** as **M-601**
> (`crates/mycelium-mlir/src/dialect/native.rs`, ~895 lines), **feature-gated behind `mlir-dialect`
> (OFF by default)**. The *default* build still uses the textual skeleton (no libMLIR), so the gating is
> real — but it is "off-by-default, libMLIR-gated", not a permanent block. The §2/§4.4 prose is left intact;
> §9 + code are authoritative. DN-15 stays **Draft**.

+ The real `ternary` → `arith`/`vector` → LLVM dialect pipeline (`dialect.rs` — the textual
  skeleton cannot be compiled without libMLIR).
+ Cross-target codegen for non-host triples (RFC-0004 §9.3: "build is host-target only" until
  the native libMLIR/LLVM backend lands).
+ The ternary-dialect inspectability at the MLIR pass level (currently the textual emitter
  provides the dump, but the actual passes are not present).

---

## 5. Per-increment summary table

| Increment | Description | Needs libMLIR? | Tractable in textual LLVM IR now? | Risk |
|---|---|---|---|---|
| **0 — bit/trit subset** | `core.id`, `bit.not/and/or/xor`, `trit.neg/add/sub/mul` | No — already shipped (M-301) | Yes — `llvm.rs` is live | Low (done) |
| **1 — non-recursive Construct/Match** | Tagged stack-`alloca [N+1 x i64]` + switch-on-tag; straight-line arms; no closures, no recursion; no OOM path (non-recursive/bounded ⇒ static alloc depth) | No — textual LLVM IR only | **Yes** (this wave; M-373 landed) | Low: stack alloca is simpler than heap alloc; no GC; the differential against the interpreter is the guard |
| **2 — closures (App/Lam) + heap** | Closure-conversion + indirect call through heap struct | No — but requires free-var analysis pass | **Yes — landed (M-378; §7)** | Medium: closure conversion is a multi-pass transform; textual-IR indirect calls are brittle to mis-encode |
| **3 — recursion (Fix/FixGroup) + stack-robustness** | **Tail-recursive `Fix` → iterative loop** (host C stack O(1) by construction) + Binary branch primitive + DepthBudget ceiling → graceful `DepthLimit`; **non-tail / `FixGroup` / recursive-data deferred** to the full heap trampoline | No — textual LLVM IR only | **Partial — tail landed (M-379; §8)**; full trampoline deferred | High: emitting a correct trampoline in textual IR is error-prone; DN-05 #1 requires no unbounded C stack (G2) — the tail-loop satisfies it structurally for its fragment, designed in not retrofitted |
| **4 — real ternary MLIR dialect lowering** | `ternary` → `arith`/`vector` → LLVM via libMLIR | **Yes — libMLIR-gated** | No — `dialect.rs` is a textual skeleton only | Blocked on M-348; every verdict stays `not established` (VR-5) |

**Column definitions:**
+ *Needs libMLIR?* — whether the increment requires a live libMLIR binding (C++ FFI, not just
  LLVM toolchain).
+ *Tractable in textual LLVM IR now?* — whether the LLVM 18 toolchain (`llc`/`clang`) present
  in this environment is sufficient, given current `llvm.rs` infrastructure.
+ *Risk* — low = straightforward extension of existing pattern; medium = new infrastructure
  needed; high = correctness is hard to check without formal backing or the approach is novel.

---

## 6. What the orchestrator/1B agent needs to know

+ **Increment 1 is this wave's scope** — the Construct/Match codegen design and any Rust
  implementation land here (or in sibling leaf 1B if the orchestrator assigned implementation
  to that agent).
+ **The `DepthBudget` trait (`crates/mycelium-mlir/src/budget.rs`, M-349) is reusable** for
  Increment 3; the native codegen should share the trait rather than re-invent the abstraction
  (DRY, KC-3).
+ **RFC-0004 §11 (new section, r3) is the append-only record of this sanction** — see
  RFC-0004 revision below.
+ **The orchestrator must register DN-15 in `docs/Doc-Index.md`** (orchestrator-owned file; this
  leaf agent does not edit it — FLAG).
+ **The orchestrator must add M-373 to `tools/github/issues.yaml`** (orchestrator-owned file;
  FLAG).
+ **No code in this note** — design-first; the implementation (if any) lands in the leaf or the
  next wave as a separate task.

---

## 7. Increment-2 realized design (M-378 — closures App/Lam + heap)

Increment 2 (§4.2) is realized in this wave under M-378. The §4.2 sketch named a heap closure record
`[fn_ptr, env]` + indirect call; this section fixes the concrete ABI, the no-GC strategy, and the
free-variable analysis the implementation lands — all in textual LLVM IR (no libMLIR; sanctioned by
§3 / RFC-0004 §2). The guarantee tag stays **Declared** (hand-written IR + the empirical M-302
differential, not Proven — VR-5). Append-only: §4.2 and the §5 table are unchanged in intent; this
section records what landed.

### 7.1 Closure value ABI (narrow, packed-`i64`)

To keep the first closure increment small and auditable (KISS/KC-3) and the differential tractable,
closures cross the call boundary carrying **8-bit binary values packed into a single `i64`**. A
`Lam` compiles to a top-level function `define i64 @closure_N(i8* %env, i64 %arg)`; its argument and
result are packed `Binary{8}` lanes. Everything else is an explicit `AotError::UnsupportedNode` (G2):
other widths, `Ternary`, datums across the boundary, closures-as-argument or closures-as-result
(currying), an `App` whose function operand does not resolve to a closure, and a top-level program
result that is itself a closure (not printable by the read-back protocol). The narrow ABI proves the
closure machinery — free-variable capture, heap record, indirect call — end-to-end without committing
to a general boxed-value calling convention; widening it (uniform pointer-boxed lanes of any
repr/width) is a later, separable step.

### 7.2 No-GC heap strategy: bump arena, freed at exit

Closures are **heap**-allocated, not stack-allocated. A closure can outlive the function that built
it (it is written into a record and applied later), so a stack `alloca` — Increment-1's choice for
`Construct` (§4.1) — would be a use-after-return for an escaping closure. Increment 2 still excludes
`Fix`/`FixGroup`, so the **total number of closure allocations is statically bounded by program
structure** — no loop can allocate unboundedly. The strategy is therefore a **bump arena**: `@main`
`@malloc`s one block, closure records are bump-allocated from it through a single helper, and the
block is `@free`d before normal completion. Every allocation routes through **one seam**
(`@myc_arena_alloc`) that checks the bump cursor against the arena capacity and takes an explicit
defined-trap (`call @abort`, never raw `unreachable` UB; G2) on over-capacity. The capacity is a
`Declared` compile-time constant — a safe over-estimate the static bound cannot exceed in Increment 2.

**This seam is exactly where Increment 3 (DN-05 #1) attaches.** When `Fix`/`FixGroup` make allocation
unbounded, the fixed capacity is replaced by an `AutoDepthBudget`-resolved ceiling
(`crates/mycelium-mlir::budget`; M-349) and the over-capacity `@abort` becomes a graceful,
`DepthLimit`-style refusal (matching the env-machine's `EvalError::DepthLimit`). The arena is
*designed in* as that attachment point now, not retrofitted later — the honest sequencing DN-05 #1
requires.

### 7.3 Closure conversion (free-variable analysis → record → indirect call)

A `Lam { param, body }` carries no captured environment in the node (the Core IR lambda is closed
except for `param` and globals); its **free variables** are the body's referenced `Named` atoms that
are neither `param` nor locally bound. A lexical free-variable analysis — recursing into nested
lambdas and match arms, removing each scope's binders — computes the capture set deterministically.
`Lam` lowering then (1) packs each captured `Binary{8}` lane to `i64` and writes the closure record
`[fn_ptr | capture_0 | … | capture_k]` into the arena, and (2) emits `@closure_N`, whose body unpacks
`%arg` to the `param` lane and each capture from `%env`, lowers `body`, and packs the result to
`ret i64`. `App { func, arg }` loads `fn_ptr` from record slot 0, points `%env` at slot 1, packs
`arg`, emits the indirect `call i64 %fp(i8* %env, i64 %arg)`, and unpacks the result. Every stage is
explicit, dumpable textual IR — no opaque pass (RFC-0004 §6 / ADR-009 / VR-4).

### 7.4 What stays refused (G2)

`Fix`/`FixGroup` remain explicit `UnsupportedNode` (Increment 3). Closures over non-`Binary{8}`
values, datums across the boundary, currying, non-closure `App` heads, and closure-valued program
results are all explicit refusals — never a silent mis-lowering, never an upgraded guarantee (VR-5).

---

## 8. Increment-3 realized design (M-379 — tail-recursion + Binary branch + stack-robustness)

Increment 3 (§4.3) discharges the **DN-05 #1** stack-robustness requirement for the native path —
recursion must run *without an unbounded C stack*, with an explicit depth limit, a graceful error,
never a SIGSEGV. The reference to mirror is the AOT env-machine (`aot.rs`): a heap control stack +
`DepthBudget`→`DepthLimit`. Two maintainer-resolved forks fix the realized scope; the guarantee tag
stays **Declared** (hand-written IR + the empirical M-302 differential, not Proven — VR-5).
Append-only: §4.3 and the §5 table are unchanged in intent; this section records what landed.

### 8.1 Mechanism — tail-recursion as a loop (not yet the full trampoline)

A **tail-position** self-recursive `Fix { name, body }` (every recursive `App` of `name` is the body's
tail) lowers to an **iterative LLVM loop**: the function parameters become loop variables (`phi`
nodes at a loop header), a tail self-call becomes a **back-edge** that updates those variables and
re-enters the header, and a non-recursive (base-case) result is a `ret`. The host **C stack is O(1)
by construction** — DN-05 #1's "no unbounded C stack" is satisfied *structurally*, not by a calibrated
guard against the platform stack limit (the weakness of a budget-guarded C-recursion approach, which
is why it was not chosen). This is the honest first step of DN-15 §4.3: it covers the tail fragment
fully and compliantly; the **full defunctionalized heap trampoline** (general non-tail recursion,
mirroring `aot.rs`'s `Frame` stack) is deferred to a later increment.

### 8.2 Depth budget → graceful `DepthLimit`

Loop iterations are bounded by an **`AutoDepthBudget`-resolved** ceiling (`crates/mycelium-mlir::budget`;
M-349 — the *same* trait the AOT env-machine uses, reused not re-invented). A depth counter is checked
each iteration; exceeding the ceiling raises a **graceful `DepthLimit`** through the read-back protocol
(a distinct sentinel, mapped to an explicit `AotError::DepthLimit`), never a SIGSEGV (G2). This is the
Increment-2 `@myc_arena_alloc` never-silent over-capacity seam (§7.2) generalized from an allocation
ceiling to a recursion-depth ceiling — the attachment point designed in at Increment 2, now realized.

### 8.3 Binary branch primitive (the base-case conditional)

A terminating recursion needs a base case; the narrow `Binary{8}` ABI had no branch primitive. `Match`
is extended to a **repr-lane scrutinee with `Lit` arms**: the scrutinee `Binary{8}` lane is compared
against each literal `Binary` value (an equality test on the packed `i64`) and branched — distinct
from the Increment-1 `Match` over a `Datum` scrutinee with `Ctor` arms (tag-switch). Both forms remain
explicit, dumpable IR (no opaque pass).

### 8.4 Differential parity (the error contract)

For a **terminating** tail-recursive `Binary{8}` program the native result is value-checked interp ≡
AOT ≡ native (M-302/M-210). For a **non-terminating** recursion, the native path must reach a graceful
`DepthLimit` in **parity** with the interpreter's non-productive refusal — the reference interpreter is
O(1)-stack and refuses by `FuelExhausted`; both are explicit, never-silent refusals (never an abort),
which is the equivalence the differential asserts for that class.

### 8.5 What stays refused (G2)

**Non-tail** recursion (a recursive `App` not in tail position), **`FixGroup`** (mutual recursion), and
**recursive heap data** (self-referential constructors) are all explicit `UnsupportedNode` — never a
silent mis-lowering, never an upgraded guarantee (VR-5). The real `ternary` MLIR dialect stays
libMLIR-gated (M-348).

A further codegen-shape limitation (surfaced in the M-379 review): a **`Match` in a tail arm's
*pre-tail* binding sequence** — e.g. computing the recursion step via a nested `Match`,
`App(self, Match …)` — is also refused. The tail-loop's back-edge `phi` records the `recur`
predecessor block before lowering the pre-tail bindings; a `Match` introduces basic blocks, so the
back-edge would actually branch from the `Match`'s merge block, leaving the `phi` with stale
predecessors (LLVM "PHI node entries do not match predecessors"). Supporting it requires threading the
*current* block label through the back-edge; deferred. The program is still semantically valid (the
reference interpreter evaluates it) — the boundary is an honest native-codegen limitation, surfaced as
an explicit `UnsupportedNode` (G2), never fragile/incorrect IR.

---

## 9. libMLIR unblock — provisionable on Linux (M-603; 2026-06-20)

This section records, append-only, that the **libMLIR-gated half** of M-348 (§2; the §5 table's
**Increment 4 — real ternary MLIR dialect lowering** row) is **no longer blocked on Linux**. The
`M-348` "libMLIR absent" premise — the basis for §2's "stays blocked" and §4.4's "permanent block
until M-348" — has been **checked false on Linux** (VR-5: upgrade only on a checked basis; the basis
is the verified install + working pipeline below). Nothing above is rewritten — DN-15 remains a
**Draft** design note (its Status line is unchanged; history is not rewritten), and this section only
adds the new, grounded finding.

**Verified facts (Linux; the same facts ADR-019 Context records).** `apt-get install -y
--no-install-recommends libmlir-18-dev mlir-18-tools` installs candidate **`1:18.1.3-1ubuntu1`**,
**version-matched to the installed LLVM 18.1.3** (`llc --version` ⇒ `Ubuntu LLVM version 18.1.3`),
providing `/usr/bin/mlir-opt-18`, `/usr/bin/mlir-translate-18`, and `libMLIR.so.18.1`. The pipeline
`mlir-opt-18 --convert-func-to-llvm --convert-arith-to-llvm --reconcile-unrealized-casts | mlir-translate-18 --mlir-to-llvmir`
emits **valid LLVM IR**. So the §2 / §4.4 premise ("no libMLIR binding in this environment") does not
hold on Linux any more.

**Made durable (the decision, not a one-off).** Provisioning is now reproducible via
`scripts/setup-mlir.sh` (derives the LLVM major from the installed `llc`, then `clang`; installs the
**version-matched** `libmlir-$MAJOR-dev` + `mlir-$MAJOR-tools` through the distro package manager only
— no `curl | bash`; idempotent; skips gracefully when LLVM / `apt-get` / the packages are absent),
intended to be wired into `just setup`. **ADR-019 (Accepted, 2026-06-20)** records the toolchain
decision: libMLIR is the **optional**, version-matched build dependency of the **off-by-default**
`mlir-dialect` Cargo feature of `mycelium-mlir` — **not** the default build.

**What changes, and what stays honest.** The **real** `ternary` → `arith`/`vector` → LLVM dialect
lowering (the §2 / §4.4 libMLIR-gated work) is **M-601** (provisionable + testable on Linux now;
tagged at its own honestly-supportable strength — §2's textual `dialect.rs` skeleton stays the
inspectable stand-in where libMLIR is absent), and the **three-way differential** (interp ≡ AOT ≡
native dialect) is **M-602** (measured; no pre-written target). Because the `mlir-dialect` feature is
**OFF by default** and probes for the tools (the `llc`/`clang` `ToolchainMissing` skip idiom of §3,
generalized), the **default build and `cargo test` stay green without libMLIR** — an explicit,
never-silent skip (G2/VR-5), exactly the posture this note has held throughout.

**Status movement (prose; the §5 table is unchanged — append-only).** The §5 table's **Increment 4 —
real ternary MLIR dialect lowering** row reads "**Yes — libMLIR-gated** … Blocked on M-348; every
verdict stays `not established` (VR-5)." With the premise now checked-false on Linux, that increment's
status moves **from "blocked on M-348 / not established" toward "provisionable + in-progress under
M-601"**. The table text itself is left exactly as written (append-only is honoured); this paragraph
is the prose record of the status change. Increment 4's *verdicts* (correctness, the M-602
differential) remain **not yet established** until M-601 lands and M-602 measures them — the unblock is
of *provisioning*, not of the *verdict* (VR-5).

---

## Meta — changelog

<!-- changelog: 2026-06-19 Draft created (M-373) — records the libMLIR-gated vs direct-LLVM-advanceable decomposition, the Increment-1 (Construct/Match) design strategy, the DN-05 #1 DepthBudget reuse plan for Increment 3, and the per-increment risk table. Append-only. -->
<!-- changelog: 2026-06-19 §7 added (M-378) — realized Increment-2 design: narrow packed-i64 closure ABI; bump-arena no-GC strategy with the single alloc seam where Increment-3's DepthBudget ceiling attaches (DN-05 #1); free-variable-analysis closure conversion. §5 table Increment-2 row marked landed. Guarantee stays Declared (VR-5). Append-only. -->
<!-- changelog: 2026-06-19 §8 added (M-379) — realized Increment-3 design: tail-position Fix → iterative LLVM loop (host C stack O(1) by construction, DN-05 #1 compliant), bounded by an AutoDepthBudget ceiling (M-349) → graceful DepthLimit (the Inc-2 arena seam generalized to a depth counter); a Binary branch primitive (Match repr-lane scrutinee + Lit arms) for the base case. Non-tail recursion, FixGroup, and recursive heap data stay UnsupportedNode (G2). §5 table Increment-3 row marked partially landed (tail only). Guarantee stays Declared (VR-5). Append-only. -->
<!-- changelog: 2026-06-19 §8.5 refined (M-379; PR #224 review) — recorded a further deferred native-codegen-shape limitation: a Match in a tail arm's pre-tail binding sequence (step computed via Match) would invalidate the loop back-edge phi, so it is an explicit UnsupportedNode (needs current-block tracking through the back-edge; deferred). The program stays semantically valid (interpreter evaluates it); honest boundary, never fragile IR (G2). Append-only. -->
<!-- changelog: 2026-06-20 §9 added (M-603) — recorded the libMLIR unblock: the M-348 "libMLIR absent" premise is checked-false on Linux (apt `libmlir-18-dev` + `mlir-18-tools`, candidate 1:18.1.3-1ubuntu1, version-matched to LLVM 18.1.3; the --convert-*-to-llvm | mlir-translate --mlir-to-llvmir pipeline emits valid LLVM IR). Made durable via scripts/setup-mlir.sh + ADR-019 (Accepted): libMLIR is the optional, version-matched build dep of the OFF-by-default mlir-dialect feature, so the default build/test stay green without it (G2/VR-5). Real dialect lowering is M-601; the three-way differential is M-602. §5 table Increment-4 status moves (in prose) from blocked-on-M-348 toward provisionable + in-progress under M-601; the table text is unchanged. Append-only. -->
