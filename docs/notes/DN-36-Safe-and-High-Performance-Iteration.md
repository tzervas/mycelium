# Design Note DN-36 — Safe & High-Performance Iteration in the Value-Semantics Model

| Field | Value |
|---|---|
| **Note** | DN-36 |
| **Status** | **Accepted** (2026-06-25; **ratified by maintainer**) — the **two-tier surface** (bounded Tier-1 idiom + budget-gated Tier-2 open form, both desugaring to one tail-recursive `Fix`) and the **§6 high-performance roadmap** are ratified as the **design direction**. Accepted ratifies the *direction*, **not** an implementation: each §6 roadmap item still gates on its own build epic + honest grounding (the recursion-aware FBIP increment is `Declared`/`Empirical`, not `Enacted`). Prior: **Draft** (2026-06-25; direction capture). Append-only; house rule #3. |
| **Feeds** | the **high-performance iteration** leg of the value-semantics surface — the recursion-aware FBIP increment (E12 **Increment 3** / "FBIP reuse-token threading", `docs/planning/E12-Memory-Model-Build-Plan.md:107`; tracked as task #6 "MEM-4 follow-ons … recursion … gated"); the iteration **surface grammar** (intersects the `[]`-grammar wave — DN-31, RFC-0030, epic #27); the kernel early-exit fold question (RFC-0007 §4.8). Builds on **DN-32** (three-layer memory), **DN-33** (MEM-4 static uniqueness), and the reclamation thread (DN-35). |
| **Date** | June 25, 2026 |
| **Decides** | *Nothing normatively* — advisory + design-direction capture. Records (1) that the value-semantics iteration **tension dissolves** by keeping value semantics in the *surface* and sourcing mutation from the *lowering* (Koka's lesson: `for`/`while` desugar to tail recursion; mutation comes from TRMC + Perceus reuse); (2) a recommended **two-tier surface** — a bounded, total-by-construction Tier-1 idiom and a budget-gated Tier-2 open form — **both desugaring to one tail-recursive `Fix` lowering path**; (3) the **safety contract** (mostly already built — `Exact`); and (4) the **high-performance roadmap** that makes the recursion-aware FBIP increment load-bearing, each item carried at its honest grounding strength. |
| **Task** | E12 high-perf iteration / FBIP-reuse-across-`Fix` (the deferred performance leg of the value-semantics loop story) |

> **Posture (transparency rule / VR-5 / G2).** This note **synthesises two sourced research records**
> — `research/21-safe-iteration-internal-RECORD.md` (repo ground truth) and
> `research/22-safe-iteration-prior-art-RECORD.md` (external prior art) — into a design direction. It
> **enacts nothing**: no RFC/ADR/DN status moves, no normative text changes, no code or property test
> ships. The grounding split is load-bearing and is held throughout: the **safety** machinery is
> **built** (`Exact`); the **performance** mechanism (TRMC + Perceus/FBIP reuse) is **`Proven` in the
> external literature** but **`Empirical`/`Declared` for Mycelium** until it is built and benchmarked;
> the recommended **surface** is a **`Declared` design proposal**, not a proven optimum. Every
> uniqueness-conditioned perf cliff and every refuse-to-guess hazard is surfaced, never buried (G2).

---

## §1 Purpose & the tension

Mycelium's value model forbids a mutable loop variable: **LR-8** (immutability) and **LR-9**
(acyclicity) mean there is no cell to step. Yet the maintainer's requirement is uncompromising:
iteration must be **functional AND safe AND high-performance** — constant host stack, constant
(allocation-free) memory for the carried accumulator, at imperative-loop speed. On its face these pull
against each other: "no mutable variable" seems to forbid the in-place mutation that makes an
imperative loop fast.

**The tension dissolves at the surface/lowering boundary.** The external record is unambiguous on this
(`research/22 §3`): a `while`/`for` *surface* does **not** require a mutable loop variable in the
*language*. Koka offers imperative-looking `for`/`while` that **desugar to tail recursion + handlers**
and lower — via TRMC — to a real machine `while`; "the mutability is in the *lowering*, not the
*semantics*." So the answer to "immutability forbids a mutable loop variable" is: **keep
value-semantics in the surface, and source the mutation from the lowering** (TRMC for the stack,
Perceus/FBIP reuse for the heap). Mycelium already takes exactly this shape — `for` is *surface sugar*
that elaborates to a self-recursive `Fix` fold (RFC-0007 §4.8; `crates/mycelium-l1/src/elab.rs`,
`elab_for`). This note's job is to extend that one lowering path so it is not only **safe** (done) but
**fast** (the gap), and to recommend the surface that steers developers onto it.

The premise is not merely compatible with the goal — it is a **tailwind**. The single hardest
precondition the performance literature needs — *guaranteed precise reference counts on an acyclic
heap* — Mycelium has **by construction** (LR-9 acyclicity *is* Perceus's garbage-free precondition;
DN-32 §1, DN-33 §3). The mechanism that turns a functional loop into an imperative one (Perceus reuse)
is therefore *complete and garbage-free here, not best-effort* (`research/22 §2.4` Mycelium-read). The
open work is almost entirely **surface design + the high-perf build-out**, not the memory mechanism's
soundness.

## §2 Current state — ground truth (what is built vs the gap)

Grounded in `research/21` with file:line evidence re-verified against source.

### §2.1 Safe iteration is BUILT (`Exact`)

- **The one iteration form.** `for x in xs, acc = init => body` is the active surface sugar; it
  elaborates to a synthesized self-recursive `Fix` fold (RFC-0007 §4.8; `mycelium-l1/src/elab.rs`
  `elab_for`). A `for`-elaborated fold is **`Total` with zero extension** — it descends structurally on
  a finite acyclic spine (`research/21 §1.3`). **Guarantee `Exact`** (totality inherited by
  construction).
- **The dangerous convention is removed by construction.** `while`/`loop`/`break`/`continue`/`return`
  are **excluded and unreserved** — the parser rejects them with a *teaching* diagnostic
  ("`{word}` is not a Mycelium form — iterate by recursion or `for`…"; `mycelium-l1/src/parse.rs`),
  *because* unbounded iteration would undermine the §4.5 divergence bit (`research/21 §3`). This is the
  never-silent rule applied to a whole syntactic category.
- **Combinator layer.** `crates/mycelium-std-iter` provides `map/filter/scan/fold/reduce/…` — all
  total — and **every eager combinator lowers to / composes the one RFC-0007 §4.8 fold over a finite
  source**, so totality is *inherited, not re-proved* (`std-iter/src/lib.rs`). **`Exact`** (inherited);
  `lazy_unfold` is `Declared`.
- **O(1)-host-stack execution.** Both execution paths run object recursion off the host stack: the
  reference interpreter is small-step over a reified term, and the AOT **env-machine is a trampoline
  over an explicit heap control stack** (`crates/mycelium-mlir/src/aot.rs`, `eval_machine`). Object
  recursion lives on the heap; the host stack is O(1) (the M-347 fix). **`Exact`.**
- **Three explicit, graceful budgets (the never-silent G2 story for a runaway loop).** A runaway loop
  on the AOT path is bounded on one refusal channel by **`fuel`** (time → `FuelExhausted`),
  **`max_depth`** (space → `DepthLimit`; checked at every frame push, `aot.rs:253-255`), and an opt-in
  **`alloc` effect budget** (memory → `EffectBudget`; `aot.rs:260-262`) — each an explicit `EvalError`,
  never an abort/hang/OOM. The depth ceiling is resolved *dynamically* from detected memory headroom
  and is `EXPLAIN`-able (no opaque magic number). **`Exact`** for the limit; **`Declared`** for the
  per-frame-cost estimate.

### §2.2 Fast iteration is half-built — the narrow native fast path

The MLIR→LLVM AOT path **does** compile a loop body to a native iterative loop, but only for a
restricted subset, on a toolchain-gated *direct-LLVM-IR* fallback (`crates/mycelium-mlir/src/llvm.rs`;
the real `dialect::native` MLIR lowering is libMLIR-gated and not present). `lower_tail_fix` recognizes
`App(Fix{λparam. Match param {Lit-arms}}, init)` and **rewrites the `Fix`+`App` into an iterative LLVM
loop** — a header block with **phi nodes** carrying the packed-`i64` accumulator across the back-edge.
The accumulator lives in an **LLVM register (phi), never the host stack and never the heap** — so for a
**`Binary{8}` scalar** accumulator this loop is allocation-free, constant-memory, imperative-speed
(`research/21 §4`). The DN-05 depth ceiling is enforced in the loop header as a graceful `DepthLimit`,
never a SIGSEGV.

Its hard limits are explicit `UnsupportedNode` refusals (G2): single `Fix` only (`FixGroup` mutual
recursion refused); the body must be exactly `λparam. Match param {Lit-arms}` (`Ctor` arms refused);
the accumulator ABI is **`Binary{8}` packed to `i64` only** — *a structural/datum accumulator is not
carried allocation-free here yet*. **`Declared`** for the perf claim (no in-repo benchmark); **`Exact`**
for the never-silent depth refusal.

### §2.3 The gap, precisely — MEM-4's FBIP reuse refuses `Fix`

For a *general* (structural) accumulator, in-place reuse is what makes the loop allocation-free — and
it **does not fire inside loops today**. MEM-4 (`crates/mycelium-mir-passes/`), the static
uniqueness/reuse analysis (DN-33), refuses recursion in **all three emitters**: `emit_owned`,
`emit_elided`, and `emit_reuse` all return `EmitError::UnsupportedNode("Fix")` /
`UnsupportedNode("FixGroup")` (`emit.rs:129-130`, and the shared `emit_ann` path identically). The
reference RC-evaluator (`eval.rs`) is **straight-line only** — any `Fix` unfold returns
`UnsupportedNode`. And the AOT bridge (`rc_plan.rs`) only **audits**, and only the *elided* (not the
*reuse*) emission — it is "an additive audit trail, never a change to how the env-machine manages
values."

**Net (the gap in one line):** a unique *structural* accumulator loop runs on the
**safe-but-allocating** runtime fallback (fresh alloc + RC each iteration), not in-place reuse. Today a
tail loop **does** bound peak memory (Rust's drop discipline reclaims iteration N before N+1 supersedes
it — never accumulates) but it **allocates fresh every iteration**. *Safe and constant-ish memory, but
not allocation-free.* (`research/21 §2`.)

> **Summary (`Exact` where built; `Declared` where claimed but unmeasured):** **safe loops are done;
> fast loops are half-done.** The safety envelope (bounded-by-construction `for`, O(1)-host-stack
> trampoline, three graceful budgets, per-iteration reclamation) is **built and `Exact`**. The
> performance envelope is built **only** for the narrow scalar case on the toolchain-gated native path;
> the general high-perf case — FBIP in-place reuse across `Fix` — is **unbuilt** (refused), and none of
> the perf claims are yet measured.

## §3 The two-mechanism performance model (prior art)

Imperative-grade loops in an immutable/RC language come from **two complementary, independently-valid
mechanisms** (`research/22 §§1–2`). Both are **`Proven` in the literature**, **`Declared` for Mycelium**
until built + benchmarked here.

- **TRMC — constant *stack*.** Tail-recursion-modulo-context (Leijen & Lorenzen, POPL 2023) rewrites a
  function whose recursive call is in tail position *up to a surrounding constructor context* into a
  genuinely tail-recursive loop, via destination-passing (a Minamide-tuple `Ctx(res, hole)` with two
  O(1) ops). `map` becomes a tail loop that allocates one cell per step, writes its address into the
  previous cell's hole, and tail-jumps — building front-to-back in place, constant stack. The generated
  assembly is a tight register loop — *the* `while`-loop instruction sequence. Soundness is **Proven**
  under a linear discipline and over the Perceus heap (Minamide 1998; Leijen & Lorenzen §5).
- **Perceus reuse / FBIP — constant *heap*.** Perceus (Reinking, Xie, de Moura, Leijen, PLDI 2021)
  emits *precise* RC so cycle-free programs are *garbage-free*; reuse analysis pairs a matched
  constructor with a same-size allocation and inserts a `drop-reuse`: **if the consumed value's
  `rc==1`**, reuse its memory in place; **else copy** (semantics preserved, persistence free). In the
  fast path "**there are no more reference counting operations at all**" and "the memory of `xs` is
  directly reused … effectively updating the list in-place." This is byte-for-byte an imperative
  `while`'s memory profile. The firing condition is **uniqueness**: shared (`rc>1`) ⇒ copy — a
  **perf cliff, never a UAF** (the worst case is a copy, never unsafety; `research/22 §2.3`).

Together = imperative-grade loops: Koka's purely functional tree-insertion is **within 10% of the
in-place C++ `std::map`**, at ~1/10th the memory of the Java version; disabling reuse is **>2× slower**
on rbtree (`research/22 §2.4`).

**The premise is a tailwind, not just compatibility.** Acyclic + RC ⇒ Perceus is *complete and
garbage-free by construction* — the "cycles reintroduce tracing-GC drawbacks" caveat **does not apply**
to Mycelium. Value semantics ⇒ the `is-unique` test is the native ownership question Mycelium already
asks; **static uniqueness (MEM-4, DN-33) can promote many reuse checks from runtime (`Empirical`,
`rc==1` probe) to compile-time (`Proven`, statically unique)** — pushing the guarantee tag up where the
analysis can prove it (`research/22 §2.4`).

## §4 The recommended surface — the maintainer's "both" (`Declared`)

A **two-tier surface**, both tiers desugaring to a tail-recursive `Fix` so there is **exactly one
lowering path** (TRMC + reuse) and the value-semantics surface is preserved (Koka's model). This is a
**design proposal — `Declared`**, not a proven optimum.

### §4.1 Tier 1 — bounded, total-by-construction (the idiom, the recommended default)

`fold` / `walk` / `map` / `for n` / `for x in xs` over a **finite** value. The iteration space *is* a
finite value, so these **terminate by construction** — you *cannot* write a runaway loop with them.
This is the clean, paradigm-aligned form (Roc / Futhark / Clojure-`reduce` precedent); it lowers via
TRMC + reuse to a flat, allocation-free native loop. **This is the recommended default the surface
steers developers toward.** Termination tag: **`Proven`** *if* §8's open question (TRMC + structural
termination over a finite acyclic spine) confirms the proof obligation discharges — see §8.

### §4.2 Tier 2 — a sugared open form (the easy on-ramp / escape hatch)

An explicit `loop` / `while cond` sugar token, **gated by a syntactically-required fuel/step budget** so
a hang is a *catchable* `Result::Err(BudgetExhausted)` / effect (`research/22 §5`). The budget is
**unavoidable** — you cannot write an un-budgeted open loop — so non-termination is an explicit, tagged
outcome, **never a silent hang**: VR-5 / never-silent applied to *time*. Termination tag: **`Declared`**
(terminates *if* the budget is set; the budget makes the hang explicit). This reuses, at the surface,
the three graceful budgets already built on the AOT path (§2.1) — `fuel`/`max_depth`/`alloc` are the
runtime realization of a Tier-2 step budget.

**The maintainer wants BOTH, deliberately.** A sugared open token exists for *easy use* (the
imperative-shaped on-ramp), **and** the diagnostics/docs **nudge toward the Tier-1 combinators** for
clean, total-by-construction code. The sugar is present; the teaching surface pushes Tier-1. This is the
honest middle between Futhark (forbids general recursion outright) and an unguarded `while` (zero
safety): Tier-2 is admitted, but only budget-gated and signposted.

### §4.3 `recur`-checked tail position (Clojure precedent)

If a loop / recursive form is written and is **not** actually tail-modulo-cons, the compiler must
**error** — *don't silently build an O(n)-stack loop*. Clojure's `recur` is the precedent: the
never-silent guarantee at the surface. A form that *looks* like a constant-stack loop but isn't is
exactly the hidden cost the house rules forbid.

### §4.4 Token spellings + budget syntax are open (intersects the grammar wave)

The exact token spellings (`loop` / `while` / `for` / `repeat`) and the **budget surface syntax** are
**open questions** that intersect the `[]`-grammar wave (epic #27 / DN-31) and RFC-0030 — this note does
**not** fix them. Per the maintainer's recorded grammar design input in **DN-31** (the future
grammar-supersession wave), fold in: **(a) better line-break / indentation support so the iteration
syntax is human-visible**, and **(b) using `,` to delineate the syntax portions** (the `for x in xs,
acc = init => body` form already uses `,` this way — extend the convention to any Tier-2 form, e.g. an
open loop's `cond, budget, body` portions). Cross-reference DN-31 §(maintainer design input) and
RFC-0030; the binding act is the future grammar RFC/supersession, not this note (VR-5 / G2).

## §5 The safety contract (mostly BUILT — `Exact`)

The safety envelope is the part that already exists, and it is what makes admitting a Tier-2 open form
acceptable at all.

- **Stack-safety** — the trampoline gives O(1) host stack for object recursion on both paths (§2.1).
  **`Exact`** (AOT env-machine + interpreter; but see §6(g): native-codegen TCO/TRMC parity is a
  separate, partly-unbuilt tier — state *which tier* each guarantee holds on).
- **Termination** — Tier-1 **by construction** (`Proven`, pending §8); Tier-2 **by required budget**
  (`Declared`, catchable). No surface form can hang silently.
- **Never-silent budgets** — `fuel` / `max_depth` / `alloc`, one refusal channel, each an explicit
  error (§2.1). **`Exact`.**
- **No mutable loop variable** — the surface never exposes one; the mutation lives in the lowering (§1).
  **`Exact`** (LR-8/LR-9, enforced by the elaboration to `Fix`).

The "dangerous convention" (`while`) is **removed by construction today** (§2.1) and re-admitted **only**
as the budget-gated, `recur`-checked Tier-2 sugar (§4.2/§4.3) — never as an unguarded imperative `while`.

## §6 The high-performance roadmap (the real work — load-bearing)

To deliver constant-memory loops for *general* (structural) accumulators — not just the scalar
native-path case (§2.2) — this note makes the **recursion-aware FBIP increment load-bearing**, tracked
as **task #6** (MEM-4 follow-ons — recursion — gated) / **E12 Increment 3** / "FBIP reuse-token
threading" (`docs/planning/E12-Memory-Model-Build-Plan.md:107`). Each item carries an honest status +
guarantee tag; all Mycelium-specific perf claims are **`Declared`** until built and benchmarked.

| # | Must build | Status | Guarantee (for Mycelium) | Evidence |
|---|---|---|---|---|
| **(a)** | RC emission across `Fix`/`FixGroup` in MEM-4 (currently refused) | UNBUILT | `Declared` | `emit.rs:129-130`; `research/21 §2.2` |
| **(b)** | A *recursive* verifying RC-evaluator (currently straight-line only) — so (a) is differential-checked | UNBUILT | `Empirical` *once built* (differential) | `eval.rs` (straight-line `UnsupportedNode`); DN-33 §8.1 Q3 |
| **(c)** | Last-use / tail-position uniqueness **across the loop back-edge** (the accumulator at the recursive tail call is sole-owner ⇒ its storage threads into the next iteration = the FBIP reuse token) | UNBUILT | `Declared` | `research/21 §2.3`; `is_sole_owned_move` is intra-`let` today |
| **(d)** | A **value-affecting** (not audit-only) reuse path in the AOT env-machine that *consumes* `MoveUnique` to actually reuse | UNBUILT | `Declared` | `rc_plan.rs` (audit-only); `research/21 §2.2` |
| **(e)** | Native codegen beyond scalar `Binary{8}` → **structural/datum** accumulators + `FixGroup`, via LLVM `tailcc` / `musttail` (Lean precedent) | UNBUILT (explicit refusal today) | `Declared` | `llvm.rs` (Binary{8} only, `FixGroup` refused); `research/22 §6` |
| **(f)** | **Region-per-iteration arenas** for non-unique intra-iteration temporaries (reclaimed O(1) at the loop boundary; Mycelium already has regions) | UNBUILT | `Declared` | `research/22 §4`; DN-32 (regions) |
| **(g)** | **Interpreter TCO/TRMC parity** — else the constant-stack guarantee is AOT-only and the tag must say so | UNBUILT (interpreter is small-step O(1)-host-stack but not TRMC-allocation-free) | `Declared` | `research/22 §6` CARE |
| **(h)** | **Benchmarks** to raise the perf claims from `Declared` → `Empirical` | UNBUILT | upgrades the tags | DN-32 §6a; DN-33 §8.1 Q5 |

The composition is: **TRMC** (items toward stack-flatness) + **Perceus reuse across `Fix`** (a–d, the
heap-flatness core) + **native lowering** (e) + **region arenas** (f) for the temporaries reuse can't
claim. Items (a)–(d) build directly on what DN-33 already landed for the *straight-line* fragment
(`emit_reuse`/`MoveUnique`, the verifying RC-evaluator's `UnsoundUnique` check) — the work is extending
them **over recursion / the back-edge**. The differential + structural-invariant soundness discipline
DN-33 §8.1 Q3 ratified applies unchanged: the recursive reuse pass is correct at **`Empirical`** (trials),
**not `Proven`**, until a mechanized proof exists (VR-5 — no upgrade past the basis).

## §7 Transparency / CARE — never-silent performance

The perf mechanism is *uniqueness-conditioned*, so the transparency rule (house rule 2) binds it: a
stray shared reference silently turning an O(1)-memory loop into a copying O(n) one is exactly the
hidden swap the house rules forbid.

- **Per-loop reuse status in `EXPLAIN`.** Report, per loop: **"in-place (statically unique)"** /
  **"in-place (`rc==1` runtime)"** / **"copying (shared)"** — so the uniqueness-conditioned perf cliff
  is never silent (`research/22 §2.4` CARE). The three labels map onto the DN-33 grounding ladder:
  statically unique ⇒ `Proven`-promotable; runtime `rc==1` ⇒ `Empirical`; shared/copy ⇒ the honest
  fallback.
- **TRMC's two-recursive-args ambiguity is refuse-to-guess (OCaml rule).** If two constructor arguments
  are recursive calls, only one can be the tail call — **the compiler must refuse to choose** and
  require explicit disambiguation, never silently pick (`research/22 §1.3`). This is the never-silent
  rule at codegen.
- **Multishot-control unsoundness must be never-silent** (Koka's hybrid-copy: detect a shared context
  at the fill site and copy on demand). **Open question / possible non-hazard:** Mycelium's effects may
  be one-shot/linear, in which case the multishot TRMC hazard **may not even arise** (the simple
  in-place version would always be sound, no hybrid fallback needed) — see §8. Either way the
  resolution must be never-silent, not assumed.
- **Region coarsening must be surfaced** (MLKit's *silent* region leak: a region inferred outside the
  loop grows unboundedly). Mycelium should make the iteration region **explicit or `EXPLAIN`-reported**
  — which region each loop allocation lands in, and that it is reclaimed at the boundary — so a
  coarsened region that turns an O(1)-memory loop into O(n) is **surfaced, never silent**
  (`research/22 §4` CARE).

## §8 Open questions (the deliberation agenda)

1. **Token spellings + budget surface syntax** (`loop`/`while`/`for`/`repeat`; how the required Tier-2
   budget is spelled) — **intersects the `[]`-grammar wave** (DN-31 / RFC-0030 / epic #27); fold in the
   DN-31 maintainer input (line-break/indentation human-visibility; `,` to delineate portions). Not
   fixed here.
2. **Can Tier-1 be tagged `Proven`?** TRMC + structural termination over a finite acyclic spine *should*
   discharge a termination proof obligation — confirm the obligation is actually checked (not merely
   asserted) before tagging `Proven` (VR-5).
3. **Interpreter TCO/TRMC parity** — does the constant-stack/allocation-free guarantee hold on the
   trusted interpreter, or only the AOT path? The tag must say *which tier* (§6(g)).
4. **One-shot vs multishot effects** — does the multishot TRMC unsoundness hazard arise in Mycelium at
   all? If effects are one-shot/linear, the simple in-place TRMC is always sound (§7).
5. **How much can static uniqueness (MEM-4) promote reuse from `Empirical` → `Proven`?** Each loop the
   analysis proves sole-owned across the back-edge upgrades that loop's reuse tag (§3, §7).
6. **Early-exit `Total` fold primitive** — `any`/`all`/`find` are done-flag folds that walk the full
   spine (no early exit in the *cost* sense, because RFC-0007 §4.8 excludes `break`); a true
   short-circuiting `Total` fold is an **open kernel question** (`research/21 §3`). It bears on Tier-1's
   performance for searches.
7. **Sequencing** vs the **E19-1** (`Repr::Seq`/`Bytes`), **reclamation (DN-35)**, and **grammar
   (#27)** threads — the high-perf roadmap (§6) depends on MEM-4 recursion RC, which depends on the
   reclamation/env-machine work; the surface (§4) depends on the grammar wave. State the order.

## §9 Guarantee posture (VR-5) + Definition of Done

**Guarantee posture (the honest split this whole note holds):**

- **Safety = `Exact`** — bounded-by-construction `for`, O(1)-host-stack trampoline, three graceful
  budgets, no mutable loop variable: **built** (§2.1, §5).
- **Performance mechanism = `Proven`-in-literature, `Empirical`/`Declared`-for-Mycelium** — TRMC +
  Perceus/FBIP reuse are proven externally (§3) but **unbuilt and unmeasured here**; the narrow scalar
  native path is built but its perf claim is `Declared` (unbenchmarked, §2.2). Tags upgrade only as §6
  lands and §6(h) benchmarks measure.
- **Surface = `Declared` design proposal** — the two-tier surface (§4), the `recur` check, and the
  token/budget spellings are a recommendation, not a ratified or proven optimum.

**Definition of Done (the gate for Draft → Accepted).** This note moves to **Accepted** when the
maintainer ratifies: (1) the **two-tier surface** (Tier-1 bounded-by-construction default + Tier-2
budget-gated open sugar, both desugaring to the one `Fix` lowering, with `recur`-checked tail position);
and (2) the **high-performance roadmap sequencing** (§6 items (a)–(h), and §8-Q7's ordering against the
E19-1 / reclamation / grammar threads). As with DN-33, Accepted ratifies the **design direction**; it
**enacts no code** — the build is the forward epic, gated on §6's increments landing behind the DN-33
§8.1 Q5 measurement discipline, with the runtime allocating fallback remaining the sound base
throughout. Promotion is append-only; nothing here moves another doc's status (house rule #3).

## §10 Relation to the corpus & grounding

- **Corpus:** RFC-0007 §4.5 (divergence bit) / §4.8 (`Fix` fold, `for` sugar, `break`-exclusion);
  DN-05 (trampoline + budgets); DN-32 (three-layer memory; LR-9 = Perceus precondition; regions);
  DN-33 (MEM-4 static uniqueness — `emit_reuse`/`MoveUnique`, the verifying RC-evaluator, the §8.1 Q3
  differential soundness + Q5 measurement gate); DN-31 + RFC-0030 (the grammar wave, the maintainer
  line-break/`,`-delimiter input); DN-35 (reclamation thread); `docs/planning/E12-Memory-Model-Build-Plan.md`
  (Increment 3 / FBIP reuse-token threading). Value-model basis: **LR-8** (immutability), **LR-9**
  (acyclicity); **KC-3** (small auditable kernel); **VR-5 / G2** (downgrade-don't-overclaim /
  never-silent).
- **Repo ground truth (file:line):** `crates/mycelium-l1/src/elab.rs` (`elab_for`),
  `crates/mycelium-l1/src/parse.rs` (`while`/`loop` teaching rejection), `crates/mycelium-mlir/src/aot.rs`
  (trampoline `eval_machine`; budgets `aot.rs:253-262`), `crates/mycelium-mlir/src/llvm.rs`
  (`lower_tail_fix`, native phi loop, Binary{8}/single-`Fix` limits),
  `crates/mycelium-mir-passes/src/{emit.rs:129-130, eval.rs, rc_plan.rs}` (FBIP refuses `Fix`,
  audit-only), `crates/mycelium-std-iter/src/lib.rs` (combinators inherit totality). Per the api-index
  caveat, source is ground truth; these are the records this note synthesises.
- **External prior art (`Proven`/`Empirical` at source strength; transfer to Mycelium is `Declared`):**
  Leijen & Lorenzen, *Tail Recursion Modulo Context* (POPL 2023); Bour, Clément, Scherer, *Tail Modulo
  Cons* (OCaml, arXiv:2102.09823 — the refuse-to-guess rule); Reinking, Xie, de Moura, Leijen, *Perceus:
  Garbage Free Reference Counting with Reuse* (PLDI 2021); Lorenzen, Leijen, Swierstra, *FP²: Fully
  in-Place Functional Programming* (ICFP 2023); Lean 4 (TRMC/FBIP, `Array.set`/`swap` at `rc==1`,
  `musttail` for guaranteed TCO); Roc / Koka / Clojure (`loop`/`recur`, transients) / Futhark surface
  forms; Tofte et al., MLKit region inference (loop-local regions, the silent region-leak pitfall);
  Grossman/Morrisett et al., Cyclone regions; eBPF bounded loops + EVM/Solana gas metering (never-silent
  non-termination).

---

## Meta — changelog

- **2026-06-25 — Created (Draft, advisory) — authored.** Synthesises `research/21-safe-iteration-internal-RECORD.md`
  (repo ground truth) and `research/22-safe-iteration-prior-art-RECORD.md` (external prior art) into a
  design direction for **safe & high-performance iteration in the value-semantics model**. Records: the
  value-semantics iteration **tension dissolves** by keeping value semantics in the *surface* and
  sourcing mutation from the *lowering* (Koka — `for`/`while` desugar to tail recursion; mutation from
  TRMC + Perceus reuse); **current state** — safe iteration is **built** (`Exact`: bounded `for` → `Fix`
  fold, O(1)-host-stack trampoline, three graceful budgets, `while`/`loop` removed-by-construction with a
  teaching diagnostic, total combinators), fast iteration **half-built** (native phi loop for scalar
  `Binary{8}` only; MEM-4 FBIP reuse **refuses `Fix`/`FixGroup`** so structural-accumulator loops fall
  back to safe-but-allocating); the **two-mechanism perf model** (TRMC = constant stack, Perceus/FBIP =
  constant heap; together within 10% of C++ `std::map`; acyclic+RC is a tailwind); the recommended
  **two-tier surface** (Tier-1 bounded total-by-construction default + Tier-2 budget-gated open sugar,
  both → one `Fix` lowering, `recur`-checked tail position; token/budget spellings open, intersecting
  DN-31/RFC-0030 grammar wave + the maintainer line-break/`,`-delimiter input); the **safety contract**
  (mostly built — `Exact`); the **high-performance roadmap** making the recursion-aware FBIP increment
  (task #6 / E12 Increment 3) load-bearing (RC across `Fix`, recursive RC-evaluator, back-edge
  uniqueness, value-affecting reuse, native structural/`FixGroup` codegen via `tailcc`/`musttail`,
  region-per-iteration arenas, interpreter TRMC parity, benchmarks); **never-silent perf transparency**
  (per-loop `EXPLAIN` reuse status; refuse-to-guess on TRMC two-arg ambiguity + multishot control;
  surface region coarsening); and the **guarantee posture** — safety `Exact`, perf mechanism
  `Proven`-in-literature/`Empirical`-`Declared`-for-Mycelium, surface `Declared`. **Enacts nothing;
  moves no status; changes no normative text.** Promotion past Draft requires the §9 Definition of Done +
  maintainer ratification (the two-tier surface + the §6 roadmap sequencing) — house rule #3,
  append-only. CHANGELOG / Doc-Index / issues.yaml / docs/api-index owned by the integrating parent.
  (Append-only; VR-5; G2.)
- **Ratified Draft → Accepted (2026-06-25).** The maintainer ratified the **two-tier surface** + the
  **§6 high-performance roadmap** as the design direction. The status move accepts the *direction
  only* — it enacts no code and upgrades no guarantee: safety stays `Exact` (already built), perf
  mechanism stays `Proven`-in-literature / `Empirical`-`Declared`-for-Mycelium, surface stays
  `Declared`; each §6 roadmap item still gates on its own build epic (VR-5 — no upgrade past basis).
