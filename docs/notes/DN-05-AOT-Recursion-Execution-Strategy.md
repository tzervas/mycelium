# Design Note DN-05 — AOT Recursion Execution Strategy (stack-robustness without bloat)

| Field | Value |
|---|---|
| **Note** | DN-05 |
| **Status** | **Draft — investigation; recommendation presented** (2026-06-16). Design-first / present-before-fold: no code lands with this note. |
| **Feeds** | RFC-0004 (execution model — the native AOT path); ADR-007/009 (interpreter is the trusted base; AOT/native is the perf path); RFC-0007 §4.5 (the fuel clock); `mycelium-mlir::aot` (the env-machine, M-342); G2 (never-silent), KC-3 / KISS / YAGNI (small auditable kernel, no bloat), NFR-7 (the M-210 differential) |
| **Question** | How should object-level recursion execute on the AOT path so it is **stack-robust** (a deep recursion is a graceful explicit limit, never a host-stack abort) and **performant**, *without bloating or destabilising* the project? |
| **Surfaced by** | M-342 (#104): the AOT env-machine recurses on the **host call stack** (O(depth)); the fuel clock bounds *productive work* but depth beyond the host stack **aborts** — worse than a hang. M-347 (#109) tracks the fix. |

> Decision-support note. It records the maintainer's prioritisation (2026-06-16), frames each option
> against Mycelium's constraints, and recommends an approach. Ratification + the build are the
> maintainer's, presented here first.

## 1. The problem, precisely

The reference **interpreter** is stack-robust *for free*: it iterates `step` over a reified term, so
its host stack is **O(1)** regardless of object-level recursion depth (RFC-0007 §4.5; the trusted base,
ADR-007). The M-342 AOT **env-machine** instead uses the **host call stack** for object recursion —
each `Fix` unfold nests Rust frames — so it is **O(depth)**. The fuel clock makes a *non-productive*
recursion an explicit `FuelExhausted`, but a *productive* recursion deeper than the host stack
**aborts** (a crash), which violates never-silent (G2): a limit must be an explicit error, never an
abort and never a hang. Today this never bites (the differential corpus is bounded-depth, and the
interpreter remains the trusted base for deep recursion), but it must be fixed **before the execution
path matures** (maintainer, 2026-06-16).

## 1.1 Empirical grounding (measured, not presumed)

Per the maintainer ("experiment even — better empirical data than guess or presume"), the threshold is
**measured**, not assumed, by `xtask recursion-probe` (a tiny-AST non-productive `spin = (fix f => λx.
f x) c`, so the only deep recursion is at *evaluation* time; the abort depth is found by
binary-searching the fuel in subprocesses). Result (2026-06-16, Linux host, main-thread 8 MB stack):

| Path | Outcome | Host stack |
|---|---|---|
| **Interpreter** (`eval_core`, fuel 5 000 000) | graceful `FuelExhausted` (~6 s) | **O(1)** — no crash at 5M |
| **AOT env-machine** (`run_core_with_fuel`) | graceful to **~593** `Fix`-unfolds; **aborts (stack overflow) by ~601** | O(depth) |

The env-machine survives only **~600 unfolds** before a host-stack **abort**, while the interpreter is
graceful at fuel ≫ 5 000 000. (On the smaller 2 MB test-thread stack the abort is <100 — the threshold
scales with the host stack, exactly the fragility this note removes.) ~600 is shallow enough that the
fix is a genuine priority (M-347, P1), not a theoretical nicety. The probe
(`xtask/src/recursion_probe.rs`) is re-runnable, so the post-fix claim ("a graceful explicit limit,
never an abort") will be **re-measured**, not asserted.

## 2. Strategy & priority (maintainer-set, 2026-06-16)

1. **Native MLIR→LLVM: design stack-robustness in, don't retrofit it** *(highest priority — a design
   requirement banked now; near-term-buildable once libMLIR is provisioned).* The native backend is
   gated on **libMLIR** (absent in *this* environment; RFC-0004 §2, ADR-009). The maintainer intends to
   **provision libMLIR at a near point** — a Windows / Claude Code desktop, or via **WSL** — so this is
   a near-term, not indefinite, deferral (tracked: M-348). What lands now is the *normative
   requirement*: the native path must execute object recursion **without an unbounded C stack** — a
   managed/segmented or heap-spilled call stack with an **explicit depth/budget limit** (a graceful
   error, never a SIGSEGV). Banking it as a constraint is the cheap, durable win: the path is born
   robust.
2. **Explicit control stack / trampoline in the env-machine** *(near-term, buildable now — lead
   implementable fix).* Reify the continuation on the **heap** and evaluate in an **iterative loop**, so
   the env-machine uses **O(1) host stack** like the interpreter. Deep recursion then becomes an
   **explicit budget/limit** (the existing fuel, plus an optional depth ceiling), never an abort —
   making the never-silent guarantee **total** for the AOT path. This is the same robustness the
   interpreter already has, brought to the second path.
3. **Tail-call detection — cautiously, possibly in conjunction** *(optional optimisation, gated on
   "earns its keep")*. Layered *on top of* the trampoline (not instead of it): detect the common
   structural/accumulator tail-recursion and run it in **constant space**. Adopt **only if** it stays
   small and auditable (KC-3/KISS) and a measurement shows it matters; otherwise the trampoline alone
   already removes the abort. **Explicitly guard against bloat** (YAGNI): a tail-call analysis that
   complicates the IR or the trusted differential is *not* worth it for a prototype env-machine.

## 3. Options, against the constraints

| Approach | Stack-robust? | Bloat/risk | Verdict |
|---|---|---|---|
| **Status quo** (host-stack recursion) | No — aborts past host stack | none | Unacceptable long-term (G2). |
| **Bigger host stack / spawn a deep-stack thread** | Only raises the ceiling | low, but a band-aid | Stopgap, not a fix — the abort still exists, just later. |
| **Trampoline / explicit control stack** (#2) | **Yes — O(1) host stack** | moderate (a rewrite of the env-machine loop) but *contained* to `aot.rs`, kernel untouched | **Recommended near-term.** Mirrors the interpreter's proven shape. |
| **Tail-call detection** (#3) | Yes for tail positions; general case still needs the trampoline | analysis complexity (the bloat risk) | **Optional, on top of #2, only if measured to matter.** |
| **Native managed stack** (#1) | **Yes, by design** | n/a today (libMLIR-gated) | **Bank as a requirement now**, build with the backend. |

All must hold: **never-silent** (a depth/budget overrun is an explicit error), **no black boxes** (the
strategy is inspectable), the **trusted base is unchanged** (the interpreter stays O(1)-stack and
authoritative), and the result stays **observationally equivalent** through the M-210 differential
(NFR-7) — including a new bounded-deep-recursion case proving the graceful limit (not an abort).

## 4. Recommendation

- **Now (cheap, durable):** record #1 as a **normative requirement** on the native path in RFC-0004
  (and reference it from the MLIR backend tasks), so stack-robustness is designed in.
- **Next implementable step:** build #2 (trampoline / explicit control stack) in `mycelium-mlir::aot`,
  turning the host-stack abort into an explicit budget/limit; add the M-210 deep-recursion case.
- **Then, only if measured:** consider #3 (tail-call detection) as a constant-space optimisation over
  #2 — kept small, or not at all. Bias to YAGNI; the trampoline already discharges the correctness goal.

This sequencing fixes the *correctness* hazard (the abort) with one contained change (#2), banks the
*native* design (#1) for free, and treats #3 as an earn-it optimisation — improving without bloating.

## 5. Open questions

- **DN05-Q1 — depth limit vs fuel.** Does the trampoline keep *only* the fuel budget, or add a distinct
  **depth ceiling** (control-stack size) with its own explicit error? (Two honest limits, or one.)
- **DN05-Q2 — env representation.** Pair the trampoline with **shared/persistent environments** (Rc /
  immutable map) to cut the per-call clone, or keep clone-by-value for simplicity first? (Perf vs KISS.)
- **DN05-Q3 — tail-call scope.** If #3 is adopted, where is "tail position" detected — at ANF lowering
  (a property on the IR) or in the env-machine loop? Prefer the loop (keeps the IR unbloated).
- **DN05-Q4 — native limit shape.** What is the native path's explicit deep-recursion error
  (segmented-stack overflow → trap → error), and does it share the env-machine's limit semantics?

## 6. Honest scope (VR-5)

#1 is **not buildable here** (libMLIR-gated) — it lands as a design requirement, honestly deferred.
#2 is buildable now and is the near-term fix. #3 is optional and measurement-gated. Until #2 lands, the
**interpreter remains the trusted base** for deep recursion (O(1) stack), and the AOT env-machine is a
bounded-depth differential path — stated, not hidden.

## Meta — changelog

- **2026-06-16 — Draft.** Created for M-347 (#109) at the maintainer's direction to **investigate +
  improve before venturing further** on the execution path. Records the prioritisation (1: bank native
  MLIR→LLVM stack-robustness as a design requirement — libMLIR-gated, not built; 2: explicit control
  stack / trampoline in the env-machine — the near-term buildable fix making never-silent total; 3:
  tail-call detection — cautious, optional, on top of #2, only if it earns its keep without bloating —
  KC-3/YAGNI). Constraints: never-silent (a limit is an explicit error, never an abort or hang), no
  black boxes, trusted interpreter unchanged, M-210 differential holds (NFR-7). Recommendation +
  open questions (DN05-Q1..Q4) recorded. **Empirically grounded** by `xtask recursion-probe` (§1.1):
  the env-machine aborts (host-stack overflow) at **~600** `Fix`-unfolds while the interpreter is
  graceful at fuel 5 000 000 in O(1) stack — measured, not presumed. libMLIR provisioning for #1 is
  near-term (desktop/WSL; M-348). No design enactment lands with this note (the experiment harness is
  measurement, not the fix) — present before folding. Append-only.
