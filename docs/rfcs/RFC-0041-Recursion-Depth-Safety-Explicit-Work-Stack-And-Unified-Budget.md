# RFC-0041 — Recursion-Depth Safety: Explicit Work-Stack Evaluators + a Unified Deterministic Depth Budget

| Field | Value |
|---|---|
| **RFC** | 0041 |
| **Status** | **Proposed** (2026-07-03) — authored under the maintainer's DN-84 §11 "solve (D) now" directive + the four ratifications of 2026-07-03 (RR-29 §6). Advances to **Accepted** on maintainer ratification of §4's decided architecture; `Accepted → Enacted` per-stage as each wave lands differential-green (§9). |
| **Type** | Normative — implementation architecture for recursion-depth safety across the L1 evaluator, the L0 reference interpreter, and the frontend passes; **no new L0 node/prim, no grammar/surface change** (KC-3 / DN-56 freeze-compatible — §7). |
| **Date** | 2026-07-03 |
| **Task** | M-979 (design D, solve-now) · M-978 (design B baseline) |
| **Feeds** | DN-84 (the decided direction it implements) · M-740 (the `.myc` self-hosting port implements the settled shape once) |
| **Grounds** | `research/29-recursion-depth-and-stack-safety-RECORD.md` (the verified evidence base) · DN-84 §4/§5/§7/§11 · DN-05 (amended §4.4) · DN-56/M-969 (freeze) · ADR-014/KC-3 (unsafe containment) · RFC-0007 §4.5/4.6 · DN-36 6(g) (TCO) |
| **Decides** | (1) convert the **L1 evaluator + L0 reference interpreter** to explicit heap **work-stack** machines mirroring the differential-validated AOT `Vec<Frame>` env-machine; (2) one **global deterministic** recursion-depth budget (default 4096, tunable) as the never-silent ceiling across all interpreters, with a **byte** co-ceiling; (3) a deterministic **floor + dynamic headroom** reconciliation with the AOT machine (amends DN-05 append-only); (4) **iterative `Drop`** for the recursive `mycelium-core`/`mycelium-l1` value+node types (DN-39 review); (5) **TCO in the evaluators** with EXPLAIN-able elided frames; (6) frontend passes stay **guarded recursion + grow-on-demand** (`stacker::maybe_grow`), converting only `check_list`'s data-spine + the `parse_type_ref` family if a raise demands; (7) an opt-in, non-deterministic, CLI-flag-only, corpus-excluded `--unbounded` mode. |

> **Posture (transparency / VR-5 / G2).** This RFC changes **resource behavior only** — it makes a
> deeply-nested input a **never-silent explicit refusal** (`DepthExceeded`/`DepthLimit`/`OutOfStackBudget`)
> instead of an uncatchable `SIGABRT`, and lets legitimate deep-but-bounded work run. It introduces **no
> new observable value or error semantics**: the three-way differential (L1-eval ≡ L0-interp ≡ AOT,
> NFR-7) must remain green — same values, same error *variants*, same evaluation order — with only the
> depth/stack *resource* profile changed. No guarantee tag is upgraded; every new bound is `Declared`
> (a structural budget) with `Empirical` differential agreement, never `Proven`.

---

## 1. Problem

Mycelium's recursive interpreters and passes overflow the host stack on deep input, turning an intended
never-silent refusal into an uncatchable `SIGABRT` — a robustness gap **and** a DoS surface. `research/29`
mapped it exhaustively; the load-bearing facts:

- **The `myc run` engine (`mycelium-interp`) has *no* depth budget at all** — `EvalError::DepthLimit` is
  defined but never constructed; `step()` recurses on the caller stack; the crate is not deep-stacked. A
  crafted deep value SIGABRTs `myc run`, **remotely reachable via a hostile spore** (RR-29 §0.1, verified).
- **A recursive-`Drop` stack bomb** on the frozen `Node`/`Value`/`Datum`/`L1Value` types overflows *on
  destruction* — even on the refusal path, on the caller stack (RR-29 §0.2, verified).
- **No TCO**, and the stdlib iterates by non-tail recursion, so a `.myc`-authored compiler pass can't
  process a large worklist without burning depth (RR-29 §0.3) — hence solve this **before** M-740 ports it.
- The scattered budgets (parser 256, eval 64, the rest 4096) and the fixed 256 MB stack (~24,600-frame
  ceiling) are toolchain-fragile — the ADR-041 near-miss (a frame-size bump SIGABRT'd a 256-deep guard)
  is the direct evidence.

## 2. Goals / Non-goals

**Goals.** No input SIGABRTs any interpreter/pass. Every over-limit is an explicit, deterministic,
machine-independent error. A `.myc` compiler pass can iterate large worklists in bounded depth. The
solution's *settled shape* is what M-740 implements once. Secure-by-design throughout.

**Non-goals.** No new L0 node/prim or surface syntax. No change to observable values/errors/order. No
flat-AST/arena rewrite now (RR-29 §3 — a long-horizon boot10-era option, not this RFC). No conversion of
frontend passes wholesale (rustc declined it; RR-29 §3).

## 3. Design overview — two layers, one solution

Per DN-84 §11 the two "designs" are one solution: **(D)** explicit work-stacks where it pays (the
evaluators), **(B)** guarded recursion + grow-on-demand where conversion isn't worth it (the frontend),
under **one** budget.

```
          ┌─────────────────────────── one GLOBAL deterministic budget (§4.2) ───────────────────────────┐
 frontend │ parser · checker · elaborator · mono · ambient · totality        (B) guarded recursion       │
 (guard)  │   → charge the global budget; run on grow-on-demand host stack (stacker::maybe_grow, coarse)  │
          ├───────────────────────────────────────────────────────────────────────────────────────────── │
 eval     │ L1 evaluator · L0 reference interpreter                          (D) explicit Vec<Frame>      │
 (convert)│   → CEK/work-stack machine mirroring AOT; TCO in tail position; O(1) host stack; byte-capped   │
          ├───────────────────────────────────────────────────────────────────────────────────────────── │
 values   │ Node · Value · Datum · L1Value                                    iterative Drop (DN-39)       │
          └───────────────────────────────────────────────────────────────────────────────────────────────┘
```

## 4. The decided architecture (ratified 2026-07-03)

### 4.1 Convert the evaluators to explicit work-stacks (D) — via **one extracted shared primitive**
Rewrite the **L1 evaluator** SCC (`eval`/`eval_app`/`eval_match`/`eval_for`/`eval_wild`/
`eval_hypha_forage`/`invoke`) and the **L0 reference interpreter** (`step`/`subst`/`node_to_core_value`/
`guarantee_of_value`) as explicit heap **`Vec<Frame>` machines**, mirroring the already-differential-
validated AOT env-machine (`mycelium-mlir` `aot.rs`/`trampoline.rs`). A `Frame` reifies the
defunctionalized continuation (what to do with a child's returned value) — including the L1 evaluator's
post-child scope-restore / `release_if_abandoned` / guarantee-assert steps (RR-29 §1.4). Control recursion
becomes O(1) host stack; the explicit stack's depth is the budget-charged quantity. `eval_for` (already an
iterative spine loop) is the in-tree model. **Definition of correctness:** observationally identical to the
current recursive machine — the M-210/`mycelium-std-conformance` three-way differential stays green.

**DRY (KC-3 / maintainer directive, 2026-07-03): extract the machine, don't triplicate it.** Because all
three consumers (AOT, L0-interp, L1-eval) need the *same* explicit-work-stack shape, the AOT machine's
generic core is **extracted into one small shared primitive** rather than reimplemented three times: a new
leaf crate **`mycelium-workstack`** (low-tier, downward-only per the acyclic-deps invariant DN-68/`acy`;
`#![forbid(unsafe_code)]`) providing the generic `WorkStack<Frame>` driver — push/pop, the budget +
byte-cap charge points, TCO frame-reuse, and the never-silent `DepthExceeded`/`OutOfStackBudget` surface —
parameterized over a per-consumer `Frame`/`Step` type. The AOT machine is **refactored onto it first**
(behavior-preserving, differential-checked), *then* L0-interp and L1-eval are built on the same crate. The
budget primitive (§4.2) lives here too (the kernel-resident, portable knob). **Self-hosting form:** the
same driver is the natural first `lib/compiler/` primitive — a `compiler.workstack` `.myc` nodule — so the
M-740 port reuses one settled contract instead of re-deriving three; the Rust crate is the transitional
host adapter, the `.myc` nodule the portable target (mirrors the DN-84 budget-is-portable / adapter-is-host
split). Extraction is a wave of its own (§7 W3½) so the shared core is validated once before three
consumers depend on it.

### 4.2 One global deterministic budget + byte co-ceiling (B)
Replace the scattered constants with **one workspace-wide `RecursionBudget`** (kernel-resident, the
portable primitive that ports to `.myc`): a `depth` ceiling (**default 4096**, the sibling value; the
parser's 256 and eval's 64 are raised to it) and a **`stack_bytes` co-ceiling** on the work-stack's own
allocation (frame-structs, *not* cloned subtrees). Exceeding either is a never-silent
`DepthExceeded { limit }` / `OutOfStackBudget { bytes }`. Tunable per-invocation (à la
`Evaluator::with_depth`); default is a **fixed constant** (determinism — accept/reject is a function of
`(source, budget)`, never the machine). Headroom to a few tens of thousands is supported by §4.3, raised
only when a real use case witnesses the need (RR-29 §4: byte cap prevents the memory-DoS the heap stack
would otherwise open — ~545 MB at depth 50k without a cap).

### 4.3 Grow-on-demand host stack, coarse placement (B) — **ordering constraint**
Wire `mycelium-stack`'s documented `grow-on-demand` feature (`stacker::maybe_grow`, unsafe contained in
the leaf — ADR-014/KC-3) at **coarse pass-entry points** (§9.2 decision). **Hard ordering (RR-29 §4):** the
fixed 256 MB stack tops out at ~24,600 frames, so grow-on-demand **must land before** any budget raise
above that, and a startup assertion must tie `max depth × max_frame ≤ reserved stack` when grow-on-demand
is off. The coarse-placement segment-overrun obligation (a pass may descend the full budget between guards)
is discharged by **bounded-stride re-check** inside the descent (preferred over `reserve ≥ budget ×
max_frame`, which recreates the frame-size coupling that caused ADR-041). **Never-silent hazard:**
`stacker`/`psm` is a **silent no-op on unsupported platforms** (wasm32/others) — the adapter MUST detect
the no-op case and surface it (G2), never degrade silently to the fixed ceiling.

### 4.4 Deterministic floor + dynamic headroom (amends DN-05, append-only)
The AOT env-machine's memory-derived ceiling (DN-05/M-349, `[10k, 2M]`) is reconciled with the
deterministic mandate as **a deterministic floor all three paths honor** (the global default, ≤ which the
conformance corpus stays) **plus dynamic headroom above it that the differential never exercises**. So the
observable accept/reject boundary is machine-independent up to the floor; the AOT machine keeps its
memory-awareness above it. DN-05 gains an append-only amendment recording this; the three-way differential
is defined **at or below the floor**.

### 4.5 Iterative `Drop` (DN-39 review)
Hand-write iterative `Drop` (the take-and-loop pattern) for `Node`/`Value`/`Datum`/`CoreValue`
(`mycelium-core`) and `L1Value` (`mycelium-l1`), so deep chains free without recursion — closing the
class the budget alone cannot (a refused deep AST is still dropped, on the caller stack). Purely additive,
**semantics-preserving** (identity/`PartialEq`/content-hash untouched). Lands via a **DN-39 promotion
review** (ratified disposition): the freeze currently protects a semantics already **unsound** for deep
input (`SIGABRT` ≠ never-silent), so this **restores** the guarantee the freeze assumes — it is
behavior-preserving hardening, not a semantic change. (Deep `Clone`/content-hash walks get the same
iterative treatment where they run on untrusted-reachable paths.)

### 4.6 TCO in the evaluators, EXPLAIN-able (in scope)
Tail-position calls in the L1/L0 evaluators reuse the current frame (loop, not recurse), so tail-recursive
`.myc` (and the future self-hosted passes) run in O(1) depth. **Transparency requirement (no black
boxes):** elided tail frames are **not silently dropped** — they are recorded as a **counted marker**
(an EXPLAIN-able "N tail frames elided" entry), preserving auditability the way CPython/JVM's TCO refusal
protected tracebacks (RR-29 §3). Non-tail nesting still charges the budget.

### 4.7 Frontend: guard + grow, convert only if needed
Parser/checker/elaborator/mono/ambient/totality stay **guarded recursion** on the grow-on-demand stack
(rustc's validated choice). **Close the guard holes** RR-29 §1 found: `usefulness`/`decision`/`lower_tree`/
Tarjan `strongconnect`/type-resolvers/`write_canon`/`mycelium-mir-passes`/`value_contains_substrate_id`/
`to_core` all charge the global budget. **Route data-shaped depth through iteration** (RR-29 §5.2): make
`check_list` (`checkty.rs:5568`) check the N elements in a flat loop against the cons field type instead of
building+recursing the N-deep `Cons` chain — the highest-leverage single fix, so a large data literal is
bounded by *memory*, not the control budget. Convert the `parse_type_ref` family to an explicit stack only
if profiling after a budget raise demands it (it has the worst 5–6 frames/level ratio).

## 5. Security requirements (secure-by-design, standing)

Non-negotiable, part of the Definition of Done (RR-29 §4):
- **Byte-capped work-stack** (§4.2) — the heap machine converts stack-DoS to memory-DoS; the cap closes it.
- **Untrusted-input coverage:** the guarantee is measured against **spore-resolved remote `.myc`** and
  **LSP editor buffers**, not just local files (RR-29 §4). The L0-interp hole (§1) is the priority fix.
- **Durability gates (DoD preconditions, not afterthoughts):** add `mycelium-l1` (+ `mycelium-stack`) to
  `cargo-mutants` scope with **remove-guard witness tests** (a deleted `enter_depth` / `>`-flip mutant must
  be killed by a `deep input → DepthExceeded` test); add **depth-structured fuzz targets** (a generator
  emitting N-deep expr/type/pattern/value nesting) over parse/check/elaborate/interp + the value walks —
  the current 3 fuzz targets never synthesize deep nesting.
- **`--unbounded` (§4 decision 7):** CLI-flag-**only** (never manifest/env/LSP-config), never-silent stderr
  banner each run, **corpus-excluded**, **refused in non-interactive/CI** contexts; a test asserts the
  corpus runner rejects it.
- **Supply chain:** add `mycelium-stack` (+`mycelium-l1`) to `scripts/checks/unsafe-per-use.sh` audit-A so
  the `#![forbid(unsafe_code)]` line can't be silently dropped; pin `stacker`/`psm` exact versions; record
  in `THIRD-PARTY-LICENSES` + `about.toml`; a geiger baseline so a new unsafe dep is never silent.
- **Keep** serde_json's default-128 limit (never introduce `disable_recursion_limit`); re-audit if any
  binary `Value`/`Node` codec (bincode/CBOR) is ever added (RR-29 §4).
- **Periodic adversarial re-review** (maintainer mandate): each wave re-runs the attacker lens over the
  new surface before it lands.

## 6. Freeze compatibility (KC-3 / DN-56)

Verified freeze-clean (RR-29 §1.4/§0.4): converting the evaluators' recursion strategy adds **no L0 node,
no prim, no surface syntax** — it is an implementation swap **behind** the frozen semantics boundary, and
the DN-56 §6 diff policy governs the *kernel surface* (nodes/prims/observable behavior), which is
unchanged. The **one** frozen-core edit (iterative `Drop`, §4.5) goes through a **DN-39 review** as
behavior-preserving hardening. Everything else lives in the unfrozen `mycelium-l1` tier or the leaf
`mycelium-stack`. The M-740 `.myc` port then implements the settled shape once.

## 7. Staged implementation plan (waves — Phase 4, isolated worktrees)

Each wave is disjoint-by-construction, lands differential-green, and is adversarially re-reviewed before
merge (per the mandate). Ordering respects §4.3's hard constraint.

| Wave | Scope | Gate |
|---|---|---|
| **W0 — Safety net** | depth-structured fuzz targets + `mycelium-l1`→mutants + remove-guard witness tests; the `DepthLimit`-never-constructed + `write_canon`/`is_pure`/Drop **guard-hole census** as failing tests | new tests red→green as waves land; no behavior change |
| **W1 — Unified budget** | one `RecursionBudget` (depth default 4096 + byte cap); wire it through the frontend guard holes (close §4.7); raise parser 256 / eval 64 to the global default | full differential green; guard-hole tests pass |
| **W2 — Grow-on-demand** | wire `stacker::maybe_grow` (coarse, bounded-stride) in `mycelium-stack`; no-op-platform detection (G2); startup budget≤stack assertion; supply-chain (§5) | build on all targets; no-op surfaced, not silent |
| **W3 — Iterative Drop** | `Drop` for `Node`/`Value`/`Datum`/`L1Value` (+ untrusted-reachable `Clone`/hash) via DN-39 review | deep-chain drop test passes; identity/eq unchanged |
| **W3½ — Extract `mycelium-workstack`** | extract the AOT env-machine's generic `WorkStack<Frame>` core (+ the budget primitive) into one new leaf crate (`#![forbid(unsafe_code)]`, downward-only per DN-68); **refactor the AOT machine onto it** behavior-preserving | AOT differential green on the extracted crate; one shared core before three consumers depend on it |
| **W4 — L0-interp work-stack** | convert `mycelium-interp` `step`/`subst`/read-off onto `mycelium-workstack` + construct `DepthLimit`; route `myc run` through the budget | three-way differential green; `myc run` refuses deep input, never aborts |
| **W5 — L1-eval work-stack + TCO** | convert the eval SCC onto `mycelium-workstack`; TCO with EXPLAIN-able elided-frame markers | differential green; tail-recursive `.myc` at O(1) depth |
| **W6 — Data-spine iteration** | `check_list` flat-loop element check (no Cons-chain build); any remaining frontend conversion if profiling demands | large data literal bounded by memory, not budget; corpus green |

M-740 pass-authoring is unblocked once W5 lands (the `.myc` iteration idiom becomes viable). The
`mycelium-workstack` core (W3½) is the first `lib/compiler/` primitive the port reuses as a
`compiler.workstack` `.myc` nodule.

**Orchestration of these waves (maintainer directives, 2026-07-03).** Disjoint waves run as **parallel
workflows in isolated worktrees**; each feeds its completed component to an **integrator via a
feed-as-ready `pipeline()`** (no barrier — integration of a green component begins as soon as its deps are
met, overlapping production). **Model roles:** the **Fable** orchestrator plans + QC-reviews; **leaf /
implementation agents are Opus or Sonnet** (scoped to complexity — ADR-038). A mechanical **Fable→Opus
safety-fallback** guards against false-positive safety flags on the biological lexicon / security lens
(auto-retry a died/refused agent on Opus, never silent). Dependency edges that force ordering: W0 before
all; W2 (grow-on-demand) **before** any budget raise (§4.3); W3½ **before** W4/W5; the rest parallelize.

## 8. Alternatives considered (and why not)

- **Convert everything to explicit stacks** — rustc declined it as not worth the cost (RR-29 §3); larger,
  riskier surface. Rejected in favor of convert-eval/guard-frontend.
- **Guard-only (no conversion)** — leaves the evaluators depth-limited; the `.myc` port keeps burning depth
  per element. Fails Goal "large worklist in bounded depth." Rejected.
- **Auto-scale budget to RAM** — machine-dependent accept/reject; breaks determinism + self-hosting
  portability (DN-84 §4). Rejected except as the corpus-excluded `--unbounded` mode.
- **Flat/arena-index AST** — dissolves deep-Drop + deep-traversal by construction but is a large retrofit;
  deferred to the boot10 `.myc` rewrite as the natural place to choose it (RR-29 §3).

## 9. Definition of Done

- No input SIGABRTs any interpreter/pass, verified by the depth-structured fuzz suite (incl.
  spore-remote-`.myc` and LSP-buffer shapes) and remove-guard mutants (all killed).
- `myc run` refuses a deep value with an explicit `DepthLimit`, never an abort (W4).
- The three-way differential (L1-eval ≡ L0-interp ≡ AOT) is green **at or below the deterministic floor**;
  documented divergence only in the dynamic headroom.
- One global deterministic budget (+ byte cap) governs every interpreter; the scattered constants are
  retired; the accept/reject boundary is machine-independent up to the floor.
- Iterative `Drop` lands via a merged DN-39 review; deep-chain construction *and destruction* are safe.
- Tail-recursive `.myc` runs at O(1) depth with EXPLAIN-able elided frames.
- Supply-chain + `--unbounded` guards (§5) in place and tested.
- DN-05 amended (floor+headroom); DN-84 → the mechanism is enacted; RR-29's guard-hole census is closed.
- Each of the seven guarantee/behavior claims graded honestly (`Declared` budget, `Empirical` differential;
  no `Proven` without a checked theorem — VR-5).

## Meta — changelog

- **2026-07-03 — Proposed (M-979 Phase-2).** Authored from `research/29` + the four maintainer
  ratifications (RR-29 §6): floor+headroom budget, DN-39 Drop admission, TCO in evaluators, convert-eval/
  guard-frontend. Decided architecture (§4), security requirements (§5), freeze compatibility (§6), a
  seven-wave staged plan (§7), and DoD (§9). No new L0 node/prim/surface (KC-3/DN-56-clean). Status
  **Proposed** → Accepted on ratification; per-stage Enacted as waves land differential-green. (VR-5/G2.)
