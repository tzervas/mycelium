# Safe + High-Performance Iteration in Immutable / Value-Semantics / RC Languages — External Prior-Art

**Research question.** How do immutable / value-semantics / reference-counted functional languages
express iteration and make it (a) **safe** (no leaks/UAF, bounded, terminating-or-explicit) and
(b) **as fast and memory-flat as an imperative `while`** — constant stack, constant memory, no
mutable loop variable?

**Scope note for Mycelium.** Mycelium's premise is *immutable + acyclic + content-addressed +
value-semantics + reference-counting + regions*. That premise is unusually **favorable** to this
goal: the single hardest precondition the literature needs — *guaranteed precise reference counts
on an acyclic heap* — Mycelium already has by construction. The mechanism that turns a functional
loop into an imperative one (Perceus reuse / FBIP) is therefore *complete and decidable* here, not
best-effort. The open work is almost entirely **surface design + the termination story**, not the
memory mechanism. Each section flags this EASIER-vs-CARE split.

Confidence tags below follow Mycelium's lattice (Exact ⊐ Proven ⊐ Empirical ⊐ Declared).

---

## 1. Tail-Recursion-Modulo-Cons (TRMC) + Tail-Call Optimization

### 1.1 Plain TCO (the baseline)
A call in **tail position** compiles to a `jump`, reusing the current stack frame instead of
pushing a new one — "Tail calls … can be compiled to a jump instruction rather than a call
instruction, and the current stack frame can be re-used instead of pushing a new frame"
(Lean 4 docs / *Functional Programming in Lean*). This gives **constant stack** for any
accumulator-style loop (`fold`/`walk`). It does **not** help a function that builds a list, because
the constructor sits *between* the recursive call and the return — `Cons(f(x), map(xx,f))` is not
in tail position; naïve `map` uses **stack linear in list length** (Leijen & Lorenzen 2022 §1).

### 1.2 TRMC — the transformation, precisely
TRMC (tail-recursion-**modulo-cons**; generalized to **modulo-context**, TRMC, by Leijen &
Lorenzen 2022) rewrites a function whose recursive call is in tail position **up to a surrounding
constructor context** into a genuinely tail-recursive loop.

**Condition (OCaml manual, ch. "Tail Modulo Constructor"):** a call is transformable when it sits
in a **"tail-modulo-cons position" = tail positions composed with constructor applications** — i.e.
"the only actions after the last function call are heap allocations" (Bour, Clément, Scherer 2021;
CraigFe). OCaml exposes it opt-in as `[@tail_mod_cons]`; Koka and Lean apply it automatically.

**The mechanism (destination-passing / hole calculus).** Instead of returning a value that the
caller wraps, the function is passed a **destination**: a pointer to the *hole* (the not-yet-filled
field) of the partially built result. Leijen & Lorenzen formalize this as a **Minamide tuple**
`Ctx(res, hole)` — `res` points at the whole result object, `hole` points "directly at the field
containing the hole inside the result object" — with two O(1) ops:

```
fun comp(k1, k2) = Ctx( app(k1, k2.res), k2.hole )     // compose contexts
fun app(k, x)    = match k { Id -> x ; Ctx(res,hole) -> { hole := x; res } }   // fill the hole
```

`map` then becomes a tail-recursive loop that, each iteration, allocates one `Cons(f(x), □)`,
**writes its address into the previous cell's `hole`**, and tail-jumps — building the list
**front-to-back in place**, constant stack, no reversal, no closures. At ARM/x86 level the `app`
match "just zero-compares a register" and the loop is a tight `tail call` (paper Appendix F shows
the generated `ldp … / cbz … / b.ne …` loop). This is **exactly the codegen of a hand-written
`while`**.

### 1.3 Soundness + non-linear control (the one real subtlety)
- The hole-calculus instantiation is **proven sound** under a linear typing discipline (Minamide
  1998; Leijen & Lorenzen 2022 §5, **Proven**). A second instantiation over the **Perceus heap
  semantics** proves the in-place update sound *as actual mutation*.
- **Adversarial caveat (verified in the paper, not marketing):** the naïve in-place version is
  **unsound under non-linear control** — `call/cc`, `shift/reset`, or **algebraic effect handlers**
  can resume a continuation twice and observe a half-mutated context. Koka's *hybrid* TRMC fixes
  this by using the **runtime reference count** to detect at the fill site whether the context is
  unique; if a multishot continuation made it shared, it **copies on demand** instead of mutating
  (§5.3). Cost: the backtracking `knapsack` benchmark is ~25% slower than the accumulator version
  because it copies more.
- **OCaml gotchas (manual):** TMC across a non-TMC function boundary silently demotes a tail call
  to a non-tail call (warned); and if **two** constructor arguments are recursive calls, only one
  can be the tail call — **the compiler refuses to choose and requires explicit disambiguation**
  (`[@tail_mod_cons]` on the chosen arg). This is a *never-silent* design choice worth copying.

### 1.4 Performance (Empirical)
- Koka: "the TRMc transformed functions are always as fast or faster than … manual alternatives"
  for linear control; map/tmap/rbtree all ≤ the accumulator, CPS, and std variants (Fig. 3).
- vs OCaml's TMC: Koka's Minamide-tuple approach is "competitive with the OCaml approach based on
  direct destination passing style"; Koka's per-element time stays **flat (~0.45s for 100M
  elements) regardless of list size**, while GC variants degrade with size (Fig. 4).

> **Mycelium read.** TRMC is the *constant-stack* half. It is a pure compile-time transformation
> with a Proven soundness basis. **EASIER for Mycelium:** no effect handlers that resume twice ⇒
> the unsound-under-multishot case may not even arise (if Mycelium's effects are one-shot/linear,
> the simple in-place version is always sound — no hybrid fallback needed). **CARE:** adopt OCaml's
> *never-silent* rule — refuse to silently pick which of two recursive args becomes the tail call;
> make it an explicit annotation or error (matches house rule 2).

---

## 2. FBIP-for-loops — the constant-MEMORY mechanism (Perceus / reuse)

TRMC gives constant *stack*. **Perceus reuse / FBIP** gives constant *heap* — the part that makes a
functional loop **allocation-free**, matching an imperative loop's memory profile.

### 2.1 Perceus precise RC + reuse analysis (the mechanism, exactly)
Perceus (Reinking, Xie, de Moura, Leijen, PLDI 2021) emits **precise** RC instructions on a core
language with explicit control flow such that "(cycle-free) programs are **garbage free**" — an
object is freed *the instant* its last reference dies, deterministically (like `malloc`/`free`), not
at scope exit.

On top of precise RC, **reuse analysis** pairs each *matched* constructor in a branch with an
*allocated* constructor **of the same size**, and inserts a `drop-reuse`:

```
fun drop-reuse(x) = if is-unique(x) then { drop children of x; &x }   // reuse this memory
                                    else { decref(x); NULL }          // someone else holds it
```

The reuse token `&x` is attached to the new allocation (`Cons@ru(...)`). After drop-specialization
and dup/fusion, the paper's Fig. 1g shows the punchline for `map`:

> "In the fast path, where `xs` is uniquely owned, **there are no more reference counting operations
> at all!** Furthermore, the memory of `xs` is directly reused to provide the memory for the `Cons`
> node … effectively **updating the list in-place**."

So a tail-recursive (or TRMC) loop over a **unique** structure: (i) constant stack via TRMC/TCO,
(ii) **zero allocation** — each result cell *is* the consumed input cell, mutated in place. That is
byte-for-byte the memory behavior of an imperative `while` over a mutable buffer.

### 2.2 FBIP — "Functional But In-Place"
The paradigm name (Reinking et al. §2.6): *"Just like tail-call optimization lets us write loops
with regular function calls, **reuse analysis lets us write in-place mutating algorithms in a purely
functional way**"* — demonstrated with stackless in-order Morris tree traversal and red-black
rebalancing via `reuse specialization` (only the changed field is re-assigned; unchanged fields are
left untouched in the reused node). Lean 4 calls the same thing FBIP and adds the **visitor-type**
trick: a non-tail recursion is made tail-recursive with an explicit visitor/zipper data type, and
"by relying on **drop-guided reuse** analysis, the allocation of the visitor … can be 'free'."

### 2.3 The firing condition (the critical premise)
In-place reuse fires **iff the consumed value's runtime reference count is 1** (`is-unique`).
Lean states it for arrays directly: *"`Array.set` and `Array.swap` will mutate an array if its
reference count is one, rather than allocating a modified copy … if `Array.swap` holds the only
reference … no other part of the program can tell that it was mutated rather than copied."* If the
value is **shared** (rc > 1), the same code path **copies** — semantics preserved, persistence
free, performance degrades to the functional baseline. **Never silent, never unsafe:** the worst
case is a copy, never a UAF.

### 2.4 Performance (Empirical)
- Headline: "on the tree insertion benchmark, the purely functional Koka implementation is **within
  10% of … the in-place mutating algorithm in C++ (`std::map`)**." Koka uses ~1/10th the memory of
  the Java version on the same benchmark.
- Disabling reuse ("no-opt") is **>2× slower** on rbtree → reuse is the load-bearing optimization.
- Counter-example that the literature is honest about: when the input is **shared** (the Koka `map`
  microbench maps over a *shared* list and sums it), "Perceus cannot apply reuse here" — it copies.
  Reuse buys you nothing for genuinely persistent / shared data; it is a *uniqueness-conditioned*
  win.

> **Mycelium read — this is where the premise pays off the most.**
> - **Acyclic + RC ⇒ Perceus is *complete and garbage-free by construction*.** The whole "cycles
>   reintroduce tracing-GC drawbacks" caveat (Perceus §1) **does not apply** to Mycelium. Precise RC
>   is exact, not approximate.
> - **Value semantics ⇒ the `is-unique` test is the natural ownership question** Mycelium already
>   asks (cf. the repo's MEM-4 static-uniqueness work). Static uniqueness can *promote* many
>   `is-unique` checks from a runtime branch to a compile-time certainty — pushing the guarantee tag
>   from Empirical (runtime rc==1) toward Proven (statically unique).
> - **CARE:** reuse is *conditional* on uniqueness. The transparency rule demands this be
>   **inspectable**: `EXPLAIN` should report, per loop, "in-place (statically unique)" /
>   "in-place (rc==1 at runtime)" / "copying (shared)" — never let the perf cliff be silent. A loop
>   that *looks* O(1)-memory but silently copies because a stray reference escaped is exactly the
>   kind of hidden swap the house rules forbid.

---

## 3. Surface-form survey — `while` vs bounded combinators vs open recursion

The design axis: **safety (can't-run-away, terminates-by-construction)** vs **flexibility
(open-ended iteration)**. Survey of how immutable/value-semantics languages land it:

| Language | Surface for iteration | `while`? | Termination story | In-place mechanism |
|---|---|---|---|---|
| **Roc** | `List.walk`/fold + tail recursion (TCO) | **No `while`/`for` keyword** | structural (fold over finite list) or open recursion (programmer's burden) | opportunistic RC reuse: a *unique* list is mutated in place by `List.map`/`keep_if`/`set` (no alloc) |
| **Elm / Haskell** | `foldl'`/`foldr`/recursion; Haskell laziness | No (Elm) / library only | non-total (Haskell) / structural-ish | Haskell relies on GHC strictness + GC; no RC reuse |
| **Koka** | effectful `for`/`foreach`/`while` **as library/handlers**, plus TRMC recursion | **Yes — but desugars** to tail-recursion + handlers; "compiles down to the equivalent of a `while` loop" | `while`/`for` use a budget/condition like imperative; recursion is open | Perceus reuse + TRMC |
| **Clojure** | `loop`/`recur` (explicit, checked tail), `reduce`/`transduce`, **transients** | `recur` only (not open recursion) | `recur` is a bounded jump; `reduce` is structural | **transients**: a local, single-threaded, short-lived *mutable* window over a persistent structure (2–3× faster, "close to Java mutable") |
| **Futhark** | `loop x = e for i < n do …` / `while`; SOACs (`map`/`reduce`/`scan`) | **Yes**, but "primarily to express … recursive functions with a **fixed iteration count**"; **no general recursion** | **`for` count or `while` cond**; all uniqueness violations caught at **compile time** | **uniqueness types** — `*[]` arg is consumed; in-place update checked statically, no runtime test |
| **Unison** | recursion + library combinators; abilities (effects) | via abilities | open recursion | GC-based (not RC reuse) |
| **Lean 4** | tail recursion + TRMC + FBIP; `for … in` via `ForIn` typeclass desugaring | `for`/`do` sugar over folds | structural (over finite collection) / open recursion | Perceus reuse; `Array.set`/`swap` in-place at rc==1 |

**Two robust patterns emerge:**

1. **Bounded-by-construction** (`for i < n`, `fold`/`walk` over a *finite* structure, Clojure
   `reduce`). Terminates because the iteration space is a value with known size. **Safest** — you
   *cannot* write a runaway loop. Futhark deliberately offers **only** this (plus a `while`) and
   **forbids general recursion** so the compiler can always reason about the loop.
2. **Open recursion / `while cond`** — maximally flexible, but termination is the programmer's
   obligation; needs a *budget/fuel* to be never-silent (Section 5).

Koka's lesson is important and reassuring: **a `while`/`for` *surface* does not require a mutable
loop variable in the language.** Koka offers imperative-looking `for`/`while` that **desugar to
tail-recursion + effect handlers** and lower (via TRMC) to a real machine `while`. The mutability
is in the *lowering*, not the *semantics*. This directly answers the maintainer's "immutability
forbids a mutable loop variable" tension: **you keep value semantics in the surface and source the
mutation from Perceus reuse + TRMC at lowering.**

> **Recommendation for Mycelium (Declared — a design proposal, not a proven optimum):**
> Offer a **two-tier surface**:
> 1. **Primary, total-by-construction tier** — `fold`/`walk`/`map`/`for n`/`for x in xs` over a
>    *finite* value. These **terminate by construction** (the iteration space is a finite value),
>    carry a `for n` count or a finite collection, and lower through TRMC+reuse to a flat,
>    allocation-free native loop. This is the ergonomic default and the safe default — you *cannot*
>    write a runaway loop with it. (Roc/Futhark/Clojure-`reduce` precedent.)
> 2. **Secondary, open tier** — an open `while cond` / unbounded recursion **gated by an explicit,
>    required fuel/step budget** (Section 5), so non-termination is a *catchable `Result`/effect*,
>    not a hang. Surface it as `while cond within budget` (or a `loop`-with-budget) so the budget is
>    syntactically unavoidable — the never-silent rule applied to time/iteration.
>
> Make `while`/`for` **sugar that desugars to tail-recursion** (Koka's model), so there is exactly
> one lowering path (TRMC + reuse) and the value-semantics surface is preserved. Give Clojure-style
> **`recur`-checked tail position**: if the user writes the recursive/loop form and it is *not*
> actually tail-modulo-cons, **error** (don't silently build an O(n)-stack loop) — the never-silent
> guarantee at the surface.

---

## 4. Region-per-iteration / arena loops

Goal: bound **peak** memory to *one iteration's working set* by allocating iteration-local garbage
in an arena that is reclaimed at the iteration boundary — for the data that *isn't* reused in place
(temporaries, intermediate non-unique structures).

- **MLKit (Tofte–Talpin region inference).** Compiler *infers* region lifetimes; allocation +
  deallocation directives are inserted automatically. Regions are stack-like (bump-allocate,
  pop-all at region end). A loop body whose allocations all live in a **letregion** placed *inside*
  the loop has its garbage reclaimed **every iteration** in O(1) (pop the bump pointer) — constant
  peak memory regardless of trip count. The classic risk MLKit documents: a region whose lifetime
  is inferred *outside* the loop **grows unboundedly** ("region leak"); the fix is making the region
  loop-local. (*Programming with Regions in the MLKit*, Tofte et al.)
- **Cyclone (Grossman, Morrisett et al.).** *Explicit* region construct (`region r { … }`) +
  region-polymorphic functions, with a type system (region subtyping, the `outlives` relation,
  effects) that **statically prevents a pointer from escaping its region** — no UAF. Later versions
  added unique pointers and RC objects. The explicit form trades inference for predictability — you
  *see* the iteration arena.
- **Vale regions (Ovadia).** *Loop-local / immutable region borrowing*: temporarily treat
  pre-existing memory as immutable so iteration over it is zero-cost, and attach a `BumpAllocator`
  to an isolate so a loop's temporaries bump-allocate and are freed at region exit. (Note: Vale's
  baseline is generational references, **not** RC — so it's a *surface/region-discipline* precedent
  for Mycelium, not a memory-mechanism one.)

**Reclaim-at-boundary mechanism (common to all three):** the arena is a bump pointer; "free" is
resetting the pointer to the iteration's start mark. No per-object freeing, no fragmentation,
constant time.

> **Mycelium read.** Mycelium already has **regions**. Two complementary tools, and they compose:
> - **Perceus reuse (Section 2)** handles the *result* structure — the thing carried across
>   iterations — making it allocation-free when unique.
> - **A region-per-iteration arena** handles the *intra-iteration temporaries* — non-unique
>   intermediates that can't be reused — reclaimed in O(1) at the loop boundary, bounding peak to
>   one iteration's working set.
> - **EASIER:** acyclic + RC means region reclaim never has to scan for cycles; a region pop is a
>   pure bump-reset. **CARE:** the MLKit "region inferred too coarsely ⇒ unbounded growth" failure
>   is real and *silent* in MLKit. Mycelium should make the iteration region **explicit or
>   `EXPLAIN`-reported** (which region each loop allocation lands in, and that it's reclaimed at the
>   boundary), so a coarsened region that turns an O(1)-memory loop into O(n) is **surfaced, never
>   silent** (house rule 2).

---

## 5. Bounded / never-silent non-termination

If the surface allows an open `while`/recursion, non-termination must be an **explicit, catchable
outcome**, not a hang. Prior art:

- **eBPF verifier (Linux ≥ 5.3).** A program loads **only if** the verifier proves every loop
  terminates: it **unrolls** loops and explores iterations until it proves an exit condition becomes
  true *or* hits the **instruction-complexity limit** (1M); otherwise the program is **rejected at
  load time**. `bpf_loop()` helper gives a bounded-iteration primitive the verifier trusts.
  *"eBPF programs … accepted only if the verifier can ensure … an exit condition … guaranteed to
  become true."* This is **termination-or-reject** — the strongest form, but it requires the loop
  bound be statically analyzable (no truly open loops).
- **Gas / fuel metering (smart contracts — EVM; Solana eBPF).** Every operation deducts from a
  **gas budget**; exhausting it **aborts with an explicit out-of-gas exception** (state reverts),
  turning non-termination into a *deterministic, catchable* failure. Solana runs eBPF but
  "doesn't use the verifier step" — instead it **meters** at runtime. This is **termination-or-
  explicit-error at runtime** — works for *fully* open loops, at the cost of a per-step counter.
- **Total functional languages (Agda, Idris `total`, Coq).** Accept only **structural recursion**
  (recurse on a strict sub-term) or well-founded recursion with a decreasing measure ⇒ **every
  function provably terminates** (`Proven`). Maximal safety; the ergonomic cost is real:
  genuinely-open computations (servers, interpreters) must be modeled with *coinduction* / explicit
  fuel, which is awkward.

**The ergonomic-cost gradient (the honest tradeoff):**

| Approach | Termination guarantee | Cost |
|---|---|---|
| Structural recursion only (Futhark `for n`, total FP) | **Proven**, compile-time | Can't express open loops directly; restructure as fold/fixed-count |
| Verifier-style bound (eBPF) | **Proven** at load, else reject | Loop bound must be statically analyzable |
| Fuel / gas budget (EVM, Solana) | **Declared→checked at runtime**: catchable out-of-budget | Per-step counter overhead; pick a budget |
| Unbounded `while` (imperative default) | none — can hang | Zero cost, zero safety |

> **Recommendation for Mycelium.** Tier the guarantee tag to match the surface (Section 3):
> - The **bounded tier** (`for n`, `fold`/`walk` over finite values, structural recursion)
>   **terminates by construction** → tag **`Proven`**; no runtime counter, no cost. This is the
>   default, and it covers the large majority of real loops.
> - The **open tier** (`while cond`, open recursion) **requires an explicit fuel/step budget**;
>   exhausting it yields a never-silent `Result::Err(BudgetExhausted)` / catchable effect — tag the
>   loop **`Declared`** (terminates *if* the budget is set; the budget makes the hang an explicit,
>   tagged outcome). Make the budget **syntactically required** so you cannot write an un-budgeted
>   open loop — this is VR-5 / never-silent applied to *time*: don't claim termination past its
>   basis. Ergonomic cost is paid **only** by code that actually needs unbounded iteration.

---

## 6. Native codegen of functional loops

Does the functional form cost anything vs a hand-written `for` at the machine level? **No, given the
right lowering.**

- **LLVM tail-call elimination → loop.** "In tail recursion elimination, the call and return
  instructions are transformed to a 'goto' instruction, turning the recursive call into a loop"
  (LLVM Cookbook). A self-tail-recursive function with the same arguments becomes a back-edge — the
  identical CFG a `while` produces. LLVM's `tailcallelim` + later loop passes (rotation, LICM,
  strength-reduction) then optimize it exactly as an imperative loop.
- **`tailcc` / `musttail` — *guaranteed* TCO.** Generic TCO is best-effort; LLVM's **`tailcc`**
  calling convention and the **`musttail`** marker make tail calls *guaranteed* (required for
  mutual/non-self tail recursion and for languages that depend on TCO for correctness). **Lean's
  MLIR-based backend uses LLVM `musttail` to guarantee tail-call elimination** that the C backend
  could not promise — concrete evidence that an SSA/MLIR→LLVM pipeline lowers functional loops to
  native loops with **no overhead and a hard guarantee**, not a hope.
- **MLIR progressive lowering.** MLIR lowers high-level dialects stepwise to LLVM-IR; a functional
  loop / TRMC form can be represented and lowered to `scf`/`cf` loop ops or to `tailcc` calls, then
  to the same machine loop. ("Lambda the Ultimate SSA" — optimizing functional programs in SSA —
  shows functional IR and imperative SSA loops converge.)
- **The destination-passing codegen** of TRMC (Section 1) is itself the proof by example: Leijen &
  Lorenzen's Appendix F shows the generated assembly is a tight register loop
  (`ldp`/`cbz`/`b.ne`) — *the* `while`-loop instruction sequence.

> **Mycelium read.** Mycelium's path is interpreter (trusted base) + **MLIR→LLVM AOT** (per
> CLAUDE.md). This is *exactly* the Lean pipeline. **EASIER:** emit TRMC'd loops as `tailcc`/
> `musttail` calls (or directly as `scf`/`cf` loops) and LLVM produces native-loop codegen with a
> hard guarantee — the functional surface costs **nothing** at the machine level. **CARE:** the
> trusted *interpreter* must also not blow the stack on these loops — implement TRMC/TCO in the
> interpreter (trampoline or explicit loop), or the "constant stack" guarantee holds only in the AOT
> path and the interpreter becomes a silent exception to the claim (transparency: the guarantee tag
> must say *which tier* it holds on).

---

## Where the immutable+acyclic+RC premise makes this EASIER vs needs CARE

**EASIER (the premise is a tailwind):**
- **FBIP / Perceus reuse is *complete and garbage-free* on an acyclic RC heap** — the cycle caveat
  that limits RC elsewhere simply doesn't apply. The mechanism that makes a functional loop
  allocation-free is *the* mechanism Mycelium's memory model already wants.
- **Value semantics ⇒ `is-unique` is the native ownership question**; static uniqueness (MEM-4) can
  promote many reuse checks from runtime (Empirical) to compile-time (Proven).
- **Regions already exist** ⇒ region-per-iteration arenas for intra-iteration temporaries are a
  natural, O(1)-reclaim complement to reuse.
- **MLIR→LLVM AOT already planned** ⇒ `tailcc`/`musttail` give zero-overhead native loop codegen
  with a hard guarantee (Lean precedent).
- **No mutable loop variable needed in the surface** — Koka proves a `while`/`for` *surface* can
  desugar to tail-recursion and source all mutation from the lowering. The maintainer's tension
  dissolves: value-semantics surface + imperative-grade lowering.

**CARE (surface + ergonomics + transparency, not the mechanism):**
- **Reuse is uniqueness-conditioned** → a stray shared reference silently turns an O(1)-memory loop
  into a copying O(n) one. `EXPLAIN` must report per-loop reuse status (in-place vs copying) — a
  never-silent perf cliff (house rule 2).
- **Termination story is a real design choice** — bounded tier `Proven` for free; open `while`
  needs a *syntactically required* fuel budget so a hang is a catchable, tagged outcome, never a
  silent hang (VR-5 applied to time).
- **TRMC's two-recursive-args ambiguity** and the **multishot-control unsoundness** must be handled
  never-silently (error / hybrid-copy) — copy OCaml's "refuse to guess" and Koka's hybrid fallback.
- **Interpreter parity** — TRMC/TCO must hold in the trusted interpreter too, or the constant-stack
  guarantee is AOT-only and the tag must say so.
- **Region coarsening** (MLKit's silent region leak) must be surfaced, not inferred away silently.

---

## 7. Annotated bibliography (primary sources, with URLs)

**TRMC / tail-call**
- Daan Leijen & Anton Lorenzen, *Tail Recursion Modulo Context — An Equational Approach*, POPL
  2023 / MSR-TR-2022-18. The canonical TRMC paper: generic modulo-context algorithm (4 equations),
  Minamide-tuple in-place implementation, Perceus-heap soundness proof, hybrid approach for
  effect-handler/non-linear control, Koka benchmarks. *Primary, Proven soundness.*
  https://www.microsoft.com/en-us/research/wp-content/uploads/2022/07/trmc.pdf
- Frédéric Bour, Basile Clément, Gabriel Scherer, *Tail Modulo Cons*, 2021 (arXiv:2102.09823) —
  OCaml's TMC design + `[@tail_mod_cons]`. https://arxiv.org/abs/2102.09823
- *Tail Modulo Cons, OCaml, and Relational Separation Logic*, POPL 2025 (arXiv:2411.19397) —
  mechanized soundness of OCaml TMC. https://arxiv.org/pdf/2411.19397
- OCaml Manual, ch. "The Tail Modulo Constructor program transformation" — **normative conditions**:
  tail-mod-cons positions, the cross-boundary demotion warning, and the *refuse-to-guess* rule for
  two recursive constructor args. https://ocaml.org/manual/5.1/tail_mod_cons.html
- CraigFe, *Tail recursion modulo cons* — accessible intro (note: predates OCaml's shipped impl).
  https://www.craigfe.io/posts/tail-recursion-modulo-cons/

**Perceus / FBIP / reuse**
- Alex Reinking, Ningning Xie, Leonardo de Moura, Daan Leijen, *Perceus: Garbage Free Reference
  Counting with Reuse*, PLDI 2021. Precise RC, reuse analysis (`drop-reuse`/reuse token), reuse
  specialization, FBIP, the "within 10% of C++ `std::map`" benchmark. *Primary, the core mechanism.*
  https://xnning.github.io/papers/perceus.pdf
- Anton Lorenzen, Daan Leijen, *Reference Counting with Frame-Limited Reuse* (FP²), MSR 2021/ICFP —
  bounds reuse to the current frame. https://www.microsoft.com/en-us/research/wp-content/uploads/2021/11/flreuse-tr.pdf
- Anton Lorenzen, Daan Leijen, Wouter Swierstra, *FP²: Fully in-Place Functional Programming*,
  ICFP 2023. https://webspace.science.uu.nl/~swier004/publications/2023-icfp.pdf
- Sebastian Ullrich & Leonardo de Moura, *Counting Immutable Beans: RC optimized for purely
  functional programming* (Lean's origin of reuse). researchgate (search title).
- Lean 4 docs — *Tail Recursion* (loops, TRMC, in-place map): https://lean4.dev/language/control-flow/tail-recursion
  ; *Functional Programming in Lean*, "Programs and Proofs / Summary" (`Array.set`/`swap` mutate at
  rc==1): https://leanprover.github.io/functional_programming_in_lean/programs-proofs/summary.html

**Surface forms**
- Roc, *Functional* (no `while`; recursion + `List.walk`/fold + TCO):
  https://www.roc-lang.org/functional ; Roc builtins `List` (unique-list in-place `keep_if`/`map`):
  https://www.roc-lang.org/builtins/alpha4/List/
- *Reference Counting with Reuse in Roc* (Utrecht MSc thesis — Roc's reuse implementation):
  https://studenttheses.uu.nl/bitstream/handle/20.500.12932/44634/Reference_Counting_with_Reuse_in_Roc.pdf
- Koka book (effectful `for`/`while`/`foreach`, TRMC lowering "to the equivalent of a while loop"):
  https://koka-lang.github.io/koka/doc/book.html
- Clojure — *Transients* (mutable window over persistent structures, the safety guarantees):
  https://clojure.org/reference/transients ; `loop`/`recur` vs `reduce` thread:
  https://groups.google.com/g/clojure/c/Q9KrgqyToIo
- Troels Henriksen et al., *Futhark: Purely Functional GPU-Programming with Nested Parallelism and
  In-place Array Updates*, PLDI 2017 — `loop … for/while … do`, uniqueness types, **no general
  recursion**, all uniqueness violations caught at compile time.
  https://futhark-lang.org/publications/pldi17.pdf ; language guide:
  https://futhark-book.readthedocs.io/en/latest/language.html

**Regions**
- Mads Tofte et al., *Programming with Regions in the MLKit* — region inference, loop-local
  regions, the region-leak pitfall. https://elsman.com/pdf/mlkit-4.7.16.pdf
- Dan Grossman, Greg Morrisett et al., *Region-Based Memory Management in Cyclone* — explicit
  regions, escape-preventing type system. https://www.cs.umd.edu/projects/cyclone/papers/cyclone-regions.pdf
- Evan Ovadia (Vale) — *Zero-Cost Memory Safety with Vale Regions* / immutable region borrowing /
  bump-allocator isolates (surface precedent; baseline is generational refs, not RC):
  https://verdagon.dev/blog/zero-cost-memory-safety-regions-overview

**Bounded / never-silent termination**
- eBPF Docs — *Loops* (bounded loops, `bpf_loop`, complexity limit, reject-at-load):
  https://docs.ebpf.io/linux/concepts/loops/ ; *Verifier*:
  https://docs.ebpf.io/linux/concepts/verifier/
- Matt Rickard, *Smart Contract Language Runtimes* (gas metering, Solana eBPF without verifier,
  metering instead): https://mattrickard.com/smart-contract-runtimes
- (Total FP: Agda/Idris `total`, Coq structural/well-founded recursion — standard references.)

**Native codegen**
- MLIR, *Toy Ch.6: Lowering to LLVM*: https://mlir.llvm.org/docs/Tutorials/Toy/Ch-6/
- LLVM Cookbook, *Tail call optimization* (call/return → goto → loop):
  https://www.oreilly.com/library/view/llvm-cookbook/9781785285981/ch07s07.html
- *Lambda the Ultimate SSA: Optimizing Functional Programs in SSA* (arXiv:2201.07272) — functional
  IR ↔ SSA loops. https://arxiv.org/pdf/2201.07272
  (Lean's MLIR backend using `musttail` for guaranteed TCO is noted across Lean compiler discussion.)
