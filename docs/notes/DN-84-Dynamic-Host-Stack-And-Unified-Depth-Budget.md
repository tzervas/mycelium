# Design Note DN-84 — Dynamic Host-Stack Growth + a Unified, Tunable Recursion-Depth Budget

| Field | Value |
|---|---|
| **Note** | DN-84 |
| **Status** | **Draft** (2026-07-03) |
| **Feeds** | M-978 (dynamic-stack-depth hardening) · M-979 (design D — explicit work-stack, solve-now; minted by §11) |
| **Decides** | *Nothing normatively* — advisory design capture. Records the design space for making the recursive frontend passes **crash-proof** (no host-stack `SIGABRT`) with an **essentially unbounded, cleanly-handled** nesting capability, while preserving the never-silent honesty rule (G2), determinism, KC-3, and self-hosting portability. The actual mechanism lands via its own RFC/ADR + issue (M-978) when the direction is ratified. |
| **Date** | 2026-07-03 |
| **Task** | M-978 · M-979 |

> **Posture (transparency rule / VR-5).** Advisory. "Current state" claims cite the actual source
> (`crates/mycelium-l1/src/`, `crates/mycelium-stack/`). This note **does not** claim limitless
> recursion is achievable or free — it makes the honest tension explicit (§4) and recommends a
> direction (§7) that trades "literal infinity" for "no crashes + a high, tunable, **deterministic**
> ceiling hit as an explicit error." No guarantee is upgraded.

---

## 1. Problem / Goal

A developer should be able to **write and run Mycelium without thinking about the implementation's
host-stack depth limits** — deeply nested (generated or recursive-shaped) code should not surprise
them with a process abort. Two concrete goals:

1. **Eliminate stack-overflow `SIGABRT` as a class.** No input — however deeply nested — should ever
   crash the process. Today the host (Rust) call stack is a real, if large, ceiling; exceeding it is
   an uncatchable abort, not a never-silent refusal. This is both a robustness gap and a latent DoS
   surface (a crafted deeply-nested input aborting the compiler/interpreter).
2. **Make the nesting limit uniform, tunable, and toolchain-independent** so it is never a surprise —
   ideally so high (and cheaply growable) that ordinary development never approaches it.

**Honest framing (§4 expands).** "Limitless" is the *feel* we want; it is **not** literally
achievable under the honesty rule — memory is finite and G2 requires an explicit, never-silent
ceiling. The realistic, honest target is: **the host stack is never the binding limit** (no crash),
and the binding limit is an **explicit, high, tunable, deterministic budget** that, when reached, is
an **explicit error** — never an abort.

> **Grounded motivation (not hypothetical).** MSRV 1.96.1 (ADR-041) surfaced exactly this: the
> parser's larger per-frame stack usage pushed a 256-deep guard past the 2 MB test-thread stack, so
> an *intended explicit refusal* became a `SIGABRT` (fixed reactively by moving `parse` onto
> `with_deep_stack`). That was a near-miss of the never-silent guarantee caused solely by a
> toolchain frame-size change — precisely the fragility this note aims to remove structurally.

---

## 2. User stories

- As a **Mycelium developer**, I want deeply nested or recursive-shaped code to either run or be
  **refused with a clear error**, never crash the toolchain, so that I can develop without reasoning
  about the implementation's stack.
- As a **tooling/CI operator**, I want no crafted input to be able to abort `myc` (a DoS), so that the
  compiler is robust on untrusted `.myc`.
- As a **kernel maintainer**, I want the recursion ceiling to be **one coherent, deterministic,
  documented policy** rather than seven scattered constants that drift, so that a toolchain bump can
  never silently move the crash boundary again (§3).
- As a **self-hosting author (boot10/E18-1)**, I want the *depth budget* to be a portable
  language-level primitive and the *host-stack management* to be a swappable host-adapter, so that the
  self-hosted `.myc` frontend carries the honest ceiling while each host manages its own stack.

---

## 3. Current state (grounded)

**Two layers already exist, but they are uncoordinated:**

- **Host stack** — `crates/mycelium-stack`: `with_deep_stack` runs a recursive pass on a **fixed
  256 MB** worker-thread stack (`DEEP_STACK_BYTES`). `parse`/`parse_phylum` (ADR-041), `eval`, and
  `ambient::resolve_report` use it. Fixed size ⇒ still a hard ceiling, just a high one.
- **Explicit semantic budgets** — per-pass depth counters, the never-silent ceilings — but with
  **inconsistent values**:

  | Pass | Constant | Value |
  |---|---|---|
  | parser | `parse.rs::MAX_EXPR_DEPTH` | **256** |
  | evaluator | `eval.rs::DEFAULT_DEPTH` | **64** (tunable via `Evaluator::with_depth`) |
  | ambient | `ambient.rs::MAX_AMBIENT_DEPTH` | 4096 |
  | elaborator | `elab.rs::MAX_ELAB_DEPTH` | 4096 |
  | totality | `totality.rs::MAX_WALK_DEPTH` | 4096 |
  | checker | `checkty.rs::MAX_CHECK_DEPTH` | 4096 |

  The parser's 256 sitting **16× below** the checker's 4096 is the exact inconsistency behind the
  ADR-041 near-miss: a value chosen against one toolchain's frame size, out of step with its siblings.

**The grow-on-demand hybrid is already designed, just unwired.** `crates/mycelium-stack/Cargo.toml`
documents an optional `grow-on-demand` feature wrapping **`stacker::maybe_grow`** (a *safe* API — the
stack-switching `unsafe` is internal to the `stacker` leaf, contained outside the kernel, preserving
`#![forbid(unsafe_code)]` / KC-3 / ADR-014), and states the guiding principle verbatim: *"The explicit
depth budgets stay in the kernel (the portable, self-hosting primitive); this crate is the transitional
Rust-host adapter."* This note builds on that stated intent.

---

## 4. The core tension (why "limitless" must bend to G2 + determinism)

Three of the project's own intents constrain a naive "auto-scale the limit to available memory":

1. **Never-silent (G2).** Out-of-resource must be an **explicit** `Option`/error, never a crash and
   never a silent success. So there is *always* a ceiling; the honest question is only *where* it sits
   and *how* it is hit. "Infinite" is not on the honesty lattice.
2. **Determinism / reproducibility.** If the ceiling is derived from **runtime-available RAM/stack**,
   the same program is **accepted on a big machine and refused on a small one** — the accept/reject
   boundary, and thus "is this program well-formed," becomes machine-dependent. That breaks the
   conformance corpus's determinism and the honesty of a checker verdict.
3. **Self-hosting portability (boot10).** The depth budget ports to `.myc` as a language primitive; a
   RAM-derived limit would make the *self-hosted checker's* verdicts host-dependent — unacceptable for
   a portable semantics.

**Resolution the recommendation rests on:** *decouple the two layers.* Grow the **host stack** on
demand (so it is **never** the binding limit and `SIGABRT` is structurally impossible up to real
memory), and keep an **explicit, deterministic, high, tunable** budget as the semantic ceiling. The
budget is what binds; the host stack simply always has room to reach it. This delivers the developer
experience ("it just works, and if I truly overdo it I get a clear error, never a crash") **without**
sacrificing G2, determinism, or portability.

---

## 5. Realistic nesting depth in practice (what actually needs how much)

*"Limitless is nice in theory but may not be realistic"* — correct. Grounding the actual need in the
language's particulars:

**Hand-authored expression nesting is shallow.** Human-written code rarely nests expressions past
~10–20 levels (parenthesization, nested calls, match arms); readability collapses long before any
machine limit. For this class even the outlier **256 is already generous**, and 4096 is far beyond any
plausible hand-written program. So for *authored control-flow depth* no increase is needed at all — the
value of this work is **crash-proofing and uniformity**, not "more depth."

**The real depth driver in Mycelium is desugaring-induced — and it is *data-shaped*, not
control-shaped.** Mycelium's own surface lowerings turn *flat data* into *deep right-leaning chains*:

- **`Vec` list literals (RFC-0040, M-977).** `[e1, …, eN]` desugars to an N-deep
  `Cons(e1, Cons(…, Nil))`. A static table of N entries (the `matrix()` case that motivated M-977)
  becomes an **N-deep** nested expression — depth = **data size**, not logic depth.
- Similar structural lowerings (long `=>` type chains, curried `f(x)(y)(z)…`, nested constructor
  patterns) scale with generated/derived size, not authored complexity.

The crucial language-particular insight: **a 5,000-element data literal is not "deeply nested logic" —
it is flat data that happens to lower to a chain.** Charging it the *recursion-depth* budget conflates
**data size** with **control depth**. A generated lookup table, a VSA codebook, or an embedded
weight/constant tensor (all realistic — cf. the long-term AI/ML-corpus direction) could easily exceed
4096 *elements* while containing zero deep *logic*.

**Implications (feed §6/§7):**

1. **Grow-on-demand stack is the load-bearing fix**, precisely because the realistic large-depth case
   (generated data literals) *must never crash* — and that is exactly the case a fixed stack + a modest
   budget handles worst.
2. **Prefer to not charge flat data-literal lowering against the control-recursion budget at all** —
   or lower list/table literals **iteratively** in the parser/elaborator (RFC-0040's desugaring is the
   candidate site), so a 100,000-entry table is bounded by *memory*, never a recursion guard, and never
   approaches the stack. This cleanly separates "how big is your data" (bounded by RAM; never a crash
   under grow-on-demand) from "how deep is your logic" (a small, honest, deterministic budget).
3. **A realistic default:** keep the **control-depth** budget modest and uniform (the 4096 family is
   already ample for authored logic; **256 is the outlier to raise, not 4096 to lower**), and route
   **data-shaped** depth through iteration / grow-on-demand rather than a larger control budget.
   "Limitless" then applies — honestly — to *data size* (bounded only by memory, never a crash), while
   *control depth* keeps a deterministic, human-scaled ceiling. **This is the realistic reading of the
   goal: not infinite logic-nesting, but "your data can be as big as your RAM, and your logic has a
   generous deterministic ceiling, and neither ever crashes."**

---

## 6. Design space

- **(A) Status quo** — fixed 256 MB `with_deep_stack` + scattered constants. Rejected: the near-miss
  showed the fixed stack + inconsistent budgets are fragile across toolchains.
- **(B) Grow-on-demand host stack + unified deterministic budget** *(recommended, §7).* Wire the
  documented `stacker::maybe_grow` feature; unify the depth constants into one policy that is always
  reachable on the growable stack.
- **(C) Auto-scale the budget to available memory.** Rejected on §4.2/§4.3 (breaks determinism +
  self-hosting portability). Could be an *opt-in, explicitly-non-deterministic* mode (`--unbounded`)
  for interactive use only, never the default, never the corpus-checked path — a possible §9 follow-up,
  not a baseline.
- **(D) Convert deep recursion to an explicit heap-allocated work stack** (iterative rewrite of the
  hot recursive passes). Strongest *performance/robustness* option (no thread stack at all for the
  converted pass) but a large, invasive rewrite of `checkty`/`elab`/`eval`; a long-horizon option,
  not the first step. Worth noting because the self-hosting port (boot10) is itself an opportunity to
  choose an explicit-work-stack shape in the `.myc` rewrite.

---

## 7. Recommended direction (advisory)

1. **Wire the grow-on-demand hybrid** (`grow-on-demand` feature → `stacker::maybe_grow`) inside
   `mycelium-stack`, contained there (KC-3/ADR-014 untouched — the kernel stays `unsafe`-free). Place
   `maybe_grow` guards at the genuine recursion points of the deep passes so the host stack grows in
   segments only when actually deep; a shallow pass pays nothing (the crate already notes "a shallow
   pass uses only a few pages").
2. **Unify the depth budgets into one coherent, documented policy** — a single high default (the
   4096-family, not the outlier 256), exposed as **one tunable knob** (per-invocation, à la
   `Evaluator::with_depth`), with each pass charging the *same* shared budget concept. The budget stays
   the **kernel-resident, portable** primitive (so it ports to `.myc`); host-stack growth stays the
   **host adapter**.
3. **Keep it deterministic:** the default budget is a **fixed constant**, not RAM-derived. Raising it
   is an explicit, opt-in act (a flag / builder call), and a program's accept/reject verdict is a
   function of `(source, budget)` only — never the machine. Reaching the budget is an **explicit
   never-silent error** (the existing `DepthExceeded`/`ParseError` shapes), now guaranteed to fire
   **before** any stack condition because the stack grows to meet it.
4. **Net developer-facing contract:** *"You will never crash from nesting. You have a very high default
   ceiling you can raise. If you exceed it you get a clear, explicit error — deterministically, the same
   on every machine."* That is the honest form of "limitless."

---

## 8. Performance, KC-3, and self-hosting fit

- **Performance.** `stacker::maybe_grow` is a cheap remaining-stack check (a branch) at guarded call
  sites, spilling to a fresh segment only near a red-zone; negligible in the common shallow case.
  Unifying the constants is free. Option (D)'s explicit work-stack would be the larger perf lever if a
  pass ever proves hot — deferred.
- **KC-3 / ADR-014.** All upstream `unsafe` stays contained in the `stacker` leaf, behind the
  `mycelium-stack` adapter — never in kernel or Mycelium-authored source. The trusted base stays
  `#![forbid(unsafe_code)]`.
- **Self-hosting (boot10/E18-1).** The two-layer split *is* the self-hosting-correct shape: the `.myc`
  frontend carries the portable depth budget (a plain counter), and whichever host runs it manages its
  own stack (Rust host via `stacker` today; a future native runtime its own way). DN-26's port should
  treat the depth budget as a first-class ported primitive and the host-stack adapter as **not**
  ported. This note and DN-26 §7.2 (frontend/kernel boundary) are consistent.

---

## 9. Open questions

1. Is one **global** budget right, or should structurally-different passes (expression nesting vs
   type-arg nesting vs pattern nesting) keep **separate** budgets with a shared *policy* for their
   defaults? (The near-miss argues at least for a shared policy, if not a single number.)
2. Where exactly do `maybe_grow` guards belong — at every `enter_depth`, or only at the coarse
   pass-entry points — to minimize the per-call cost while guaranteeing no segment is ever overrun?
3. Should an **opt-in non-deterministic `--unbounded`** interactive mode (design (C)) exist for REPL/
   exploration, explicitly excluded from the corpus and clearly flagged as machine-dependent?
4. Does any pass want **design (D)** (explicit heap work-stack) now, or is that strictly a
   post-self-hosting perf item? (Likely the latter; the `.myc` rewrite is the natural place to choose
   it.)
5. What is the **budget default**? 4096 (the current sibling value) is already far beyond hand-written
   code; is there a realistic generated-code case that wants more, and if so what witnesses it?

---

## 10. Grounding / honesty

Grounded in: `crates/mycelium-stack/{Cargo.toml,src/lib.rs}` (the fixed 256 MB `with_deep_stack` + the
documented, unwired `grow-on-demand`/`stacker` hybrid + the "budgets are the portable primitive"
principle); the six depth constants enumerated in §3 (source-cited); ADR-041 (the MSRV bump whose
frame-size change surfaced the near-miss); ADR-014/KC-3 (unsafe containment); DN-26 §7.2 (the
frontend/kernel boundary this is consistent with); G2 + VR-5 (never-silent, no over-claim). Nothing is
declared implemented; this note decides nothing normatively — it records the space and a recommended
direction for M-978's RFC/ADR to ratify.

---

## 11. Maintainer decisions (2026-07-03)

The design space (§6) and open questions (§9) are now **decided by the maintainer**. Recorded
append-only; these fix the direction the follow-on RFC/ADR implements. DN-84 remains **Draft** (the
mechanism still lands via that RFC/ADR); this section is the ratified *direction*, not the
implementation.

- **§6 direction → (B) is the baseline, with (D) elevated to active design/research.** Implement
  **(B)** — grow-on-demand host stack (`stacker::maybe_grow`) + a unified deterministic budget — as
  the mechanism. **(D)** (explicit heap work-stack) is **planned for eventual implementation**, backed
  by **deeper research**, and its **design work begins now** (§9.4) rather than being deferred strictly
  to post-self-hosting. *Reconciliation of §6 ("D eventual") with §9.4 ("D now"): **B is implemented
  now; D's design + research are active now; D's implementation is the eventual target.** Flagged for
  confirmation — if the intent was "implement D now instead of B," this note is corrected.*
- **§9.1 budget shape → one global budget + a shared policy.** A single workspace-wide recursion-depth
  budget governs all passes, under one documented policy (not seven independent constants). This is the
  kernel-resident, portable primitive that ports to `.myc` (boot10).
- **§9.2 `maybe_grow` placement → coarse pass-entry points only.** Guards go at the pass entries
  (`parse`/`check`/`elab`/`eval`/…), **not** at every `enter_depth` call — minimizing per-call cost —
  while still guaranteeing no stack segment is ever overrun (each pass reserves/grows enough headroom
  for its whole descent up to the global budget). *Design obligation for the RFC: prove the coarse
  placement cannot overrun a segment between guards (e.g. reserve ≥ `budget × max_frame` at entry, or
  re-check at a bounded stride).*
- **§9.3 `--unbounded` mode → yes, opt-in, non-deterministic, REPL/exploration only.** Design **(C)**
  ships as an explicitly opt-in, **clearly-flagged non-deterministic** interactive mode for REPL/
  exploration — **excluded from the conformance corpus** and never the default (so determinism/G2 hold
  for the normal path; the mode's machine-dependence is surfaced, never silent).
- **§9.4 design (D) → begin now.** The explicit heap-allocated work-stack is **not** treated as a
  strictly-post-self-hosting perf item; its design + deeper research start now (see §6 reconciliation).
  The `.myc` self-hosting rewrite (boot10) is the natural place to choose an explicit-work-stack shape
  for the ported passes.
- **§9.5 budget default → 4096 for now, headroom to tens of thousands.** Keep **4096** as the default
  control-depth budget (ample for authored logic). Future use cases *may* want to scale to a **few tens
  of thousands** deep — the tunable knob (§7.2) plus grow-on-demand (§7.1) must comfortably support that
  range without a redesign; the exact ceiling stays demand-driven (raise when a real witnessing use case
  appears, never speculatively).

**Net decided direction:** implement (B) with a single global deterministic budget (default 4096,
tunable, growable to tens-of-thousands) + coarse-placed grow-on-demand guards + an opt-in
non-deterministic `--unbounded` REPL mode; concurrently open the (D) explicit-work-stack design/research
track for eventual implementation. Data-shaped desugaring depth (§5) is routed through
iteration/grow-on-demand, not the control budget.

**Correction (2026-07-03, maintainer — resolves the §6/§9.4 reconciliation flag).** The flagged
"B now / D eventual" reading is **superseded**: the maintainer confirms **(D) is solved now** —
*"now is as good a time as any; if we're making these kinds of refactors and changes, better to sort
it out sooner rather than later — get the complexity of the problem and solution figured out and
solve that problem first, before figuring out its implementation into the dogfooded implementation."*
That is: the depth/stack problem is **fully worked now in the Rust reference era** — the settled
shape is what the M-740 `.myc` port then implements once, rather than retrofitting it later. The (B)
components that stay load-bearing under (D) — the single global deterministic budget (§9.1/§9.5) and
host-stack management for any pass that remains host-recursive — are **supporting infrastructure of
the one solution**, not a separate deferred track. Where (D) lands first (which passes convert, which
keep guarded recursion) is an output of the mandated method below, not pre-decided here.

**Mandated method (maintainer, 2026-07-03): research → plan → adversarial review → implement.**
The solution is developed in that strict order: (1) a comprehensive **research** pass — map every
recursion site, its shape, its conversion cost; the language's own iteration/TCO mechanics (what a
`.myc` work-stack loop actually costs today — decisive for the dogfooded form); prior decisions
(RFC-0007 §4.5/§4.6, DN-05, DN-56 freeze conditions, RFC-0024 defunctionalization); external prior
art (rustc's `ensure_sufficient_stack`, explicit-stack/CEK interpreters, flat-AST arenas); and the
blast radius (depth-keyed tests, DN-80 reject ledger, conformance corpus). (2) A **plan/RFC** fixing
the architecture. (3) An **adversarial review** pass that tries to break the plan before code exists.
(4) Only then, staged implementation in isolated worktrees (disjoint-by-construction waves).
**Secure-by-design is a standing requirement:** the work is periodically re-examined adversarially —
crafted-input DoS (untrusted `.myc` reaching `parse`/LSP/spore-resolve), the `stacker`/`psm` unsafe +
supply-chain surface (must stay contained in the leaf per ADR-014), memory-exhaustion vectors of a
raised budget, and the `--unbounded` mode as a footgun — so the hardening itself never introduces a
new vulnerability class.

---

## Meta — changelog

- **2026-07-03 — §11 correction appended (maintainer; append-only, no status move).** The B-now/
  D-eventual reconciliation is superseded: **(D) is solved now**, before the M-740 `.myc` port absorbs
  the shape; (B)'s budget + host-stack pieces are supporting infrastructure of the one solution.
  Mandated method recorded: **research → plan → adversarial review → implement**, with
  **secure-by-design periodic adversarial passes** (DoS surface, `stacker`/`psm` containment,
  memory-exhaustion, `--unbounded` footgun). M-979 rescoped accordingly; M-740 now depends on the
  settled design. Status stays **Draft**. (M-978/M-979.)
- **2026-07-03 — §11 added: maintainer decisions (append-only, no status move).** §6 → **(B)** baseline
  (grow-on-demand + unified deterministic budget) with **(D)** explicit-work-stack elevated to active
  design/research now, eventual implementation; §9.1 → one **global** budget + shared policy; §9.2 →
  `maybe_grow` at **coarse pass-entry points only**; §9.3 → **yes** to an opt-in non-deterministic
  `--unbounded` REPL mode (corpus-excluded); §9.4 → begin **(D)** now; §9.5 → default **4096**, headroom
  to tens-of-thousands, demand-driven. B/D "now vs eventual" reconciliation flagged for confirmation.
  Status stays **Draft** (feeds M-978's RFC/ADR). (M-978.)
- **2026-07-03 — Draft created (M-978).** Captures the design space for crash-proof, essentially-
  unbounded nesting via (B) grow-on-demand host stack + a unified deterministic tunable depth budget,
  with the honest tension (§4: "limitless" bends to G2 + determinism + self-hosting portability) made
  explicit and a recommended direction (§7). Built on the already-documented `mycelium-stack`
  grow-on-demand hybrid and motivated by the ADR-041 near-miss. Decides nothing normatively; feeds
  M-978. Status: **Draft** (VR-5 / house rule #3).
