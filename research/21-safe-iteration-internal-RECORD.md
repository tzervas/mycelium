# Internal research — Safe + high-performance iteration (loops) in Mycelium's value-semantics memory model

Repo `/home/user/mycelium`, branch `main`, read-only. All claims cite `file:line` / `DOC §`.

**One-line frame.** Mycelium has no mutable loop variable (LR-8 immutable, LR-9 acyclic). Iteration =
`for x in xs, acc = init => body`, a *surface sugar* that elaborates to a synthesized self-recursive
`Fix` fold (RFC-0007 §4.8). "Safe loops" (bounded, never-silent, stack-robust) are **built**. "Fast
loops" (allocation-free, constant-memory, imperative-speed) are **partly built** — allocation-free for
a scalar accumulator on the *native* LLVM path, but **not** for unique-accumulator structural loops on
the interpreter/env-machine path, where MEM-4's FBIP reuse explicitly **refuses** `Fix`.

---

## 1. Execution / safety machinery — the never-silent bounding story

### 1.1 Two execution paths, both O(1)-host-stack for object recursion
- **Reference interpreter** (`mycelium-interp`, the *trusted base*, ADR-007): small-step over a
  reified term → host stack is **O(1)** regardless of object-recursion depth; `Fix` bounded by a
  **fuel** clock surfaced as an explicit `FuelExhausted`, never a hang (DN-05 §1 lines 17-25; RFC-0007
  §4.5).
- **AOT env-machine** (`crates/mycelium-mlir/src/aot.rs`): a **trampoline over an explicit heap
  control stack** (`eval_machine`, aot.rs:375-532). `App`/`Match` push a continuation frame
  (`Frame::Resume`/`Frame::ApplyThen`, aot.rs:231-238) and switch blocks; object recursion lives on
  the **heap**, host stack is **O(1)** (aot.rs:18-28 module doc). This is the M-347 fix; pre-trampoline
  it aborted (host-stack overflow) at ~600 `Fix`-unfolds (DN-05 §1.1 lines 32-55, measured by
  `xtask recursion-probe`).

### 1.2 Three explicit, graceful budgets (the never-silent G2 story for a runaway loop)
A runaway loop on the AOT path is bounded by three named budgets on **one refusal channel**, each an
explicit `EvalError`, never an abort/hang/OOM:

| Budget | Bounds | Checked at | Error |
|---|---|---|---|
| **`fuel`** (time) | `Fix`/`FixGroup` unfolds | every unfold — `fuel.checked_sub(1)` (aot.rs:271, 283) | `EvalError::FuelExhausted` |
| **`max_depth`** (space) | control-stack depth | every frame push — `stack.len() >= max_depth` (aot.rs:253-255, 506-508) | `EvalError::DepthLimit { limit }` |
| **`alloc` effect budget** (memory, opt-in) | control-stack bytes | per frame push, `DEFAULT_PER_FRAME_BYTES` (aot.rs:260-262) | `EvalError::EffectBudget` |

- Default fuel `AOT_FUEL = 1_000_000` (aot.rs:49). Default depth is resolved **dynamically** from
  detected memory headroom — `AutoDepthBudget::default().resolve()` (aot.rs:57-66) reads
  `MemAvailable`/`RLIMIT_AS` via pure-`std` `/proc` (zero `unsafe`), 70% ÷ ~1 KiB/frame, clamped
  `[10_000, 2_000_000]`, conservative static fallback 200_000 (DN-05 §5 DN05-Q5, lines 146-159,
  201-215). The chosen ceiling + its basis are `EXPLAIN`-able via `default_depth_budget()` →
  `DepthResolution`/`DepthBasis` (aot.rs:61-66) — no opaque magic number (G2).
- The `alloc` budget is the RFC-0014 §4.8 ledger threaded via `run_core_with_effects`
  (aot.rs:196-208); absent ⇒ no charge ⇒ unchanged behaviour (I5 opt-in, aot.rs:713-725 test).
- Tests prove all three are graceful, never an abort: `spin()` (a non-productive `Fix`) returns a
  budget error at fuel 50M (aot.rs:650-668); a small ceiling yields `DepthLimit{64}` (aot.rs:671-682);
  an `alloc` overrun yields `EffectBudget` (aot.rs:684-710).

**Status: BUILT. Guarantee: `Exact` (the limit is reached and reported by construction); the
*dynamic* depth derivation's per-frame cost is `Declared`/over-counted, not `Proven` (DN-05 §6, line
215).**

### 1.3 The kernel recursion node (RFC-0007 `Fix`/`FixGroup`)
- `Node::Fix { name, body }` and `Node::FixGroup { defs, body }` are the kernel recursion nodes;
  the v0 calculus is Accepted r4 (RFC-0007 §4.1-4.8, line 6).
- Lowering: elaboration → ANF (`mycelium-core::lower`); the env-machine evaluates `Rhs::Fix`/`FixGroup`
  by **suspending** a value (`AotVal::Fix`/`FixGroup`, aot.rs:78-104) and unfolding on application
  (`enter_apply`, aot.rs:244-308): a `Fix` binds its own name to a fresh self-suspension and re-enters
  the body under the fuel clock; a `FixGroup` re-binds **every** member name (mutual recursion) then
  enters the focused member (aot.rs:282-305). Totality of a `for`-elaborated fold is **`Total` with
  zero extension** — the helper descends structurally on a finite acyclic spine (RFC-0007 §4.8 lines
  236-238).

---

## 2. Loop memory behavior today + the exact FBIP gap

### 2.1 What reclaims iteration N's garbage
Two different memory stories depending on path:

- **Interpreter / env-machine (the default, trusted path):** values are **Rust-managed**. Each `Fix`
  unfold `clone()`s the environment (`env.clone()` at aot.rs:276-278, 288) and builds fresh `Datum`s
  per `Construct`. Rust's ownership drops a binding's value when its last owner goes out of scope, so
  iteration N's intermediate garbage is reclaimed as N+1's frame supersedes it — peak memory is bounded
  by live-set + control-stack depth, **not** accumulated across all iterations. **But every iteration
  *allocates fresh* (no in-place reuse)** — the runtime is the "safe-but-allocating fallback".
- **Runtime RC tier (`mycelium-std-runtime`, DN-32/MEM-1..3):** `RcCell<T>` wraps `std::rc::Rc<T>`
  (rc.rs:70-104); `drop_ref` probes `rc == 1` → `RcProbe::UniqueOwner(T)` (sole owner, value extractable
  for FBIP reuse) vs `rc > 1` → `Shared` (decrement only) (rc.rs:171-210). `Region` (region.rs:224-356)
  batches scope-exit reclamation (one `ReclamationRecord(ScopeExit)` per deferred entry at
  `Region::close`). **This is the runtime *fallback*** — the `UniqueOwner`/FBIP reuse is the runtime
  probe, explicitly tagged `Declared` (no measurement, rc.rs:44, 54), and the FLAG at rc.rs:65-66 says
  the static elision that would remove the RC ops "is MEM-4 (deferred). The `UniqueOwner` probe here is
  the runtime fallback."

So today a tail loop **does** bound peak memory (it reclaims N before N+1, never accumulates), but it
**allocates fresh each iteration** — it is *safe and constant-ish memory but not allocation-free*.

### 2.2 The exact FBIP gap — MEM-4 refuses `Fix`/`FixGroup`
MEM-4 (`crates/mycelium-mir-passes/`) is the static uniqueness analysis (Perceus/FBIP) that emits and
elides RC ops so a unique accumulator can be **reused in place** instead of reallocated. It has three
increments built (emit.rs):
- `emit_owned` (B0 — naive, `k-1` `Dup`s per binding) — emit.rs:65-132.
- `emit_elided` (Increment 1 — borrow elision, fully-borrowable `let`s → `Borrow` + one `DropAfter`,
  no `Dup`) — emit.rs:285-288, 489-492.
- `emit_reuse` (Increment 2 — `rc==1` sole-owned single move → `MoveUnique`, the **FBIP-reuse-eligible**
  consume point) — emit.rs:304-307, 470-479.

**The gap, precisely (file:line):**
1. **All three MEM-4 emitters refuse recursion.** `emit_owned`: `Node::Fix => Err(UnsupportedNode("Fix"))`,
   `Node::FixGroup => Err(UnsupportedNode("FixGroup"))` (emit.rs:129-130). `emit_ann` (the shared
   borrow-elision + reuse path): identical refusal (emit.rs:436-437). Module doc: "Recursion
   (`Fix`/`FixGroup`) is **out of scope** for this increment … returns an explicit
   `EmitError::UnsupportedNode`" (emit.rs:11-13).
2. **The reference RC-evaluator can't even run a recursive term.** `eval.rs` is straight-line only —
   `App`/`Match`/`Construct`/`Lam` (and thus any `Fix` unfold) return `RcError::UnsupportedNode`
   (eval.rs:234-238, 22-26). The differential corpus is straight-line (eval.rs:26).
3. **The AOT-tier bridge only audits, and only the elided (not reuse) emission.** `rc_plan.rs`
   `emit_reclamation_plan` calls `emit_elided` (rc_plan.rs:126) — **not** `emit_reuse` — and is
   explicitly an **additive audit trail, never a change to how the env-machine manages values**
   (rc_plan.rs:13-18); its correspondence to real reclamation is `Declared` (rc_plan.rs:117-119). A
   `Fix` term is the benign `RcPlanError::Emit(UnsupportedNode)` (rc_plan.rs:62-65).

**Net:** FBIP constant-memory reuse (`MoveUnique`) **does not fire inside loops** — `for`/`Fix` bodies
are never reached by `emit_reuse`. A unique accumulator loop runs on the **safe-but-allocating** runtime
fallback (fresh alloc + RC each iteration), not in-place reuse.

### 2.3 What the "recursion-RC / FBIP-for-loops" increment must add
To make unique-accumulator loops allocation-free, the deferred increment (the prompt's "R16-Q7 /
DN-33 Phase-3"; tracked as task #6 "MEM-4 follow-ons (… recursion …) — gated", and
`docs/planning/E12-Memory-Model-Build-Plan.md:107` "Increment 3 (full FIP static guarantee, Phase 3)
/ the FBIP reuse-token threading") must specify/build:
1. **RC emission across `Fix`/`FixGroup`** — extend `emit_owned`/`emit_ann` past the
   `UnsupportedNode("Fix")` refusal: balance the recursive binder, the self-reference, and the unfold
   boundary (RC of recursive bindings is harder — emit.rs:11-13 / DN-33 §6).
2. **A reference RC-evaluator over the recursive/control-flow fragment** — extend `eval.rs` past the
   straight-line `UnsupportedNode` so the differential can *verify* the recursive emission
   semantics-preserving (eval.rs:22-26, 234-238).
3. **Last-use / tail-position uniqueness across the back-edge** — recognize that the accumulator at the
   recursive tail call is the sole owner (`rc==1`) so its storage threads into the next iteration
   (the FBIP reuse token), turning per-iteration `Construct` into in-place mutation of a uniquely-owned
   cell. (Today `is_sole_owned_move` is intra-`let` only — emit.rs:470-479.)
4. **Wire reuse (not just elision) into a value-affecting path** — `rc_plan.rs` would need to call
   `emit_reuse` and the env-machine (or native codegen) would need to *consume* `MoveUnique` to actually
   reuse, not merely audit (rc_plan.rs:126, 13-18).

**Status: UNBUILT (gated). The pieces it builds on — `emit_reuse`/`MoveUnique`, the verifying
RC-evaluator's `UnsoundUnique` check (eval.rs:46-53, 187-200) — are BUILT for the straight-line
fragment.**

---

## 3. Surface-form inventory

| Form | State | Evidence |
|---|---|---|
| **`for x in xs, acc = init => body`** | **ACTIVE** — the one iteration form | lexer `Tok::For` (token.rs:114-115, keyword map line 330); parser `parse_for` (parse.rs:1054-1066); elaborator `elab_for` lowers it to the synthesized `Fix` fold (elab.rs:710-716, 1233+) |
| `while`, `loop`, `break`, `continue`, `return` | **EXCLUDED + UNRESERVED** — emit a *teaching* diagnostic, never parse | parser explicitly rejects with "`{word}` is not a Mycelium form — iterate by recursion or `for`…" (parse.rs:1012-1046); RFC-0007 §4.8 lines 257-262; lexicon line 161-163. Reason: unbounded iteration would undermine the §4.5 divergence bit. |
| `fold`/`repeat`/`iter` as keywords | **ABSENT** (not lexed) | no tokens in token.rs; named-args `fold(xs, from:, with:)` is slated as an ordinary **L2 library function once lambdas land**, same elaboration, no new syntax (RFC-0007 §4.8 lines 252-254) |
| open recursion (`fn f(…) = … f(…) …`) | **ACTIVE** — the general mechanism `for` desugars to | RFC-0007 §4.8 (Fix); `for` is "bounded iteration sugar over structural recursion" (token.rs:114) |

**Iterator combinator crate — BUILT (eager, total).** `crates/mycelium-std-iter` (M-526) provides
`map/filter/scan/enumerate/flat_map/fold/reduce/count/any/all/find/position/zip/zip_exact/chain/
take/skip/step_by/transduce` + an explicitly-typed lazy surface (`Lazy::unfold`/`lazy_take`)
(lib.rs:1-98). **Every eager combinator lowers to / composes the one RFC-0007 §4.8 `for` fold over a
finite source**, so totality is *inherited*, not re-proved (lib.rs:11-18) — guarantee `Exact`
(inherited) for all eager ops, `Declared` for `lazy_unfold` (lib.rs:519-533).

**Bounded vs open / short-circuit state:**
- Bounded-iteration combinators are **total by construction** (finite acyclic spine).
- Open/unbounded sources are *type-segregated*: only `Lazy<E>` is unbounded, and the **only** way back
  to a total `Foldable` is an explicit `lazy_take(_, n)` with a visible `Nat` bound (lib.rs:446-458) —
  the unbounded→bounded transition is never an implicit cutoff.
- **Short-circuit is a `Foldable`-spine *cost* gap, FLAGGED open:** `any`/`all`/`find`/`position` are
  **done-flag folds** — total, but they **walk the full spine** (no early exit in the cost sense)
  because RFC-0007 §4.8 excludes `break`; a true early-termination `Total` fold primitive is an **open
  kernel question** (lib.rs:20-31, FLAG Q3). `Foldable<E>` itself is a monomorphic `Vec<E>` stand-in
  pending the RFC-0007 trait story (lib.rs:33-43, FLAG Q2).

---

## 4. Native-codegen status

The MLIR→LLVM AOT path **does compile loop bodies to a native iterative loop** — for a restricted
subset — but it is a *direct-LLVM-IR* fallback, **not** the env-machine and **not** the deferred MLIR
dialect.

- **Two backends in `mycelium-mlir`:**
  - The default `aot.rs` env-machine **interprets** (model/differential path) — §1 above. It does *not*
    emit machine code.
  - `llvm.rs` is a **direct-LLVM-IR backend** (RFC-0004 §2 fallback): it emits textual LLVM IR and
    drives `llc` + `clang` to a native executable (`compile_and_run`, llvm.rs:2451-2480). Gated on the
    **toolchain being present** — absent `llc`/`clang` ⇒ `AotError::ToolchainMissing`, callers
    **skip** (house idiom, llvm.rs:78-81, 2484-2495; smoke test skips at llvm.rs:2709-2721).
  - The **real MLIR dialect lowering** (`dialect::native`, feature `mlir-dialect`) is **OFF by default,
    libMLIR-gated, not present in this environment** (lib.rs:6-17, 74; Cargo.toml:19-21).
- **Native tail-recursive loop (the fast path that *is* built):** `lower_tail_fix` (llvm.rs:1537+)
  recognizes `App(Fix{λparam. Match param {…}}, init)` (Increment-3, RFC-0004 §11.6 / DN-05 #1) and
  **rewrites the Fix+App into an iterative LLVM loop** — a header block with **phi nodes** carrying the
  packed-`i64` accumulator and a depth counter across the back-edge (llvm.rs:1665-1735), a `switch`
  dispatch, and tail-classified arms. The accumulator lives in an **LLVM register (phi), never the host
  stack and never the heap** — so for a `Binary{8}` scalar accumulator this loop is **allocation-free,
  constant-memory, imperative-loop-speed** (no stack recursion). The DN-05 depth ceiling is enforced in
  the loop header as a graceful `AotError::DepthLimit` (llvm.rs:96, 1643-1647), never a SIGSEGV.
- **Hard limits of the native loop (explicit `UnsupportedNode`, G2):** single `Fix` only — `FixGroup`
  (mutual recursion) refused (llvm.rs:585-587, 690-693); the Fix body must be exactly
  `λparam. Match param {Lit-arms}` (llvm.rs:1554-1604); `Ctor` arms on the recursion param refused
  (llvm.rs:1620-1626); the accumulator ABI is **`Binary{8}` packed to `i64`** only (llvm.rs:1637-1640,
  1034-1051) — a *structural/datum* accumulator is **not** carried allocation-free here yet.

**Status: the native iterative loop is BUILT (toolchain-gated; bit-subset, scalar accumulator) — the
true MLIR dialect is libMLIR-gated/UNBUILT. Guarantee `Declared` for the perf claim (no in-repo
benchmark of the native loop), `Exact` for the never-silent depth refusal.**

---

## 5. What a "safe + high-performance iteration" design must specify/build

Each item tagged **status** (built / fallback-only / unbuilt) and an **honest guarantee tag**.

| # | Must specify / build | Status | Guarantee | Evidence |
|---|---|---|---|---|
| 1 | Bounded-by-construction iteration form (`for` → `Total` `Fix` fold) | **BUILT** | `Exact` (totality inherited) | RFC-0007 §4.8; elab.rs:1233; std-iter lib.rs:11-18 |
| 2 | O(1)-host-stack execution of object recursion (trampoline) | **BUILT** (env-machine) | `Exact` | aot.rs:375-532; DN-05 §1.1 |
| 3 | Never-silent runaway-loop bounding (fuel + depth + alloc, one channel) | **BUILT** | `Exact` (limit); `Declared` (per-frame cost) | aot.rs:253-262, 271-283; DN-05 DN05-Q5 |
| 4 | Dynamic, `EXPLAIN`-able depth budget (not a magic constant) | **BUILT** | `Declared` (1 KiB/frame estimate) | aot.rs:57-66; DN-05 §2.4/§5 |
| 5 | Eager combinator layer (map/filter/fold/…) over the one fold | **BUILT** | `Exact` (inherited) | std-iter lib.rs |
| 6 | Per-iteration reclamation so peak memory doesn't accumulate (Rust-managed / RC fallback) | **BUILT (fallback)** | `Exact` (drop discipline); `Declared` (RC reuse perf) | aot.rs env clone+drop; rc.rs:44,54,65-66 |
| 7 | True early-exit (`break`-equivalent) `Total` fold primitive (short-circuit *cost*, not just done-flag) | **UNBUILT** (open kernel Q) | n/a — open | std-iter lib.rs:20-31 (FLAG Q3); RFC-0007 §4.8:259 |
| 8 | Native iterative loop codegen (Fix+App → LLVM phi loop), allocation-free **scalar** accumulator | **BUILT (toolchain-gated, bit subset)** | `Declared` (perf, unmeasured); `Exact` (depth refusal) | llvm.rs:1537+, 1665-1735 |
| 9 | MLIR dialect lowering (the ratified perf path) | **UNBUILT** (libMLIR-gated) | `Declared` | lib.rs:6-17,74; RFC-0004 §2 |
| 10 | **FBIP/Perceus static reuse across `Fix`** — unique-accumulator structural loops allocation-free (in-place reuse, not realloc) | **UNBUILT** (gated; the headline gap) | `Empirical` *once built* (differential), not `Proven` | emit.rs:129-130,436-437; eval.rs:234-238; rc_plan.rs:126,13-18 |
| 11 | RC-emission + verifying RC-evaluator extended over recursion/control-flow (so #10 is differential-checked) | **UNBUILT** (straight-line only today) | — | emit.rs:11-13; eval.rs:22-26 |
| 12 | `MoveUnique` consumed by a **value-affecting** path (not audit-only), incl. native datum accumulators | **UNBUILT** | `Declared` | rc_plan.rs:13-18 (audit-only); llvm.rs:1637-1640 (Binary{8} only) |
| 13 | Native `FixGroup` (mutual recursion) + structural/datum accumulator in the native loop | **UNBUILT** (explicit refusal) | — | llvm.rs:585-587, 1620-1626 |
| 14 | Measurement of the perf claims (KC-2 kc2-09/kc2-10 iteration tasks; native-loop & reuse benchmarks) | **UNBUILT** (benchmarks pending) | upgrades `Declared`→`Empirical` | RFC-0007 §4.8:254; rc.rs:44; DN-32 §6a |

### The gap in one paragraph
**Safe loops are done; fast loops are half-done.** The *safety* envelope (bounded-by-construction `for`,
O(1)-host-stack trampoline, three explicit graceful budgets, dynamic EXPLAIN-able depth ceiling,
per-iteration reclamation) is **built and `Exact`** on both the trusted interpreter and the AOT
env-machine (items 1-6). The *performance* envelope is built only for the **narrow scalar case on the
toolchain-gated native path**: `lower_tail_fix` turns a `Binary{8}`-accumulator `Fix` loop into an
allocation-free LLVM phi loop (item 8) — but the **general high-perf case is unbuilt**: MEM-4's FBIP
in-place reuse **explicitly refuses `Fix`/`FixGroup`** in all three emitters and in the verifying
RC-evaluator (items 10-11), so a *unique structural accumulator* loop falls back to the
safe-but-allocating runtime (fresh alloc + RC every iteration), with reuse present only as an
audit-trail (`emit_elided`, not `emit_reuse`) and a `Declared` runtime probe. The ratified MLIR dialect
that would generalize native codegen is libMLIR-gated/unbuilt (item 9), and none of the perf claims are
yet measured (`Declared`, item 14). So the design must specify: (a) an early-exit `Total` fold
primitive (item 7, open kernel question), and (b) **recursion-aware FBIP reuse** — RC emission across
`Fix`, a recursive verifying RC-evaluator, last-use uniqueness across the loop back-edge, a
value-affecting (not audit-only) reuse path, and the differential + benchmarks to move the guarantee
from `Declared` honestly up to `Empirical` (items 10-14).
