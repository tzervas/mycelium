# RFC-0041 — Recursion-Depth Safety: Explicit Work-Stack Evaluators + a Unified Deterministic Depth Budget

| Field | Value |
|---|---|
| **RFC** | 0041 |
| **Status** | **Accepted (Rev 2, 2026-07-03 — maintainer-ratified)**. Authored under DN-84 §11 "solve (D) now" + the four 2026-07-03 ratifications (RR-29 §6); **hardened by the Phase-3 adversarial review** (§11 — 4 Critical + 15 High source-confirmed objections resolved); **ratified `Proposed → Accepted` by the maintainer 2026-07-03** — including the §6 **within-freeze behavior-preserving-hardening channel** (a ratified DN-56 process addition), the §Posture **I1–I3 correctness invariant** (superseding Rev-1's "same error variants"), and the §4.0 **source-call-boundary depth metric**. `Accepted → Enacted` per-stage as each wave (§7) lands differential + error-parity green (§9). Prior: Proposed (Rev 1 → Rev 2, 2026-07-03). |
| **Type** | Normative — implementation architecture for recursion-depth safety across the L1 evaluator, the L0 reference interpreter, and the frontend passes; **no new L0 node/prim, no grammar/surface change** (KC-3 / DN-56 freeze-compatible — §6). |
| **Date** | 2026-07-03 |
| **Task** | M-979 (design D, solve-now) · M-978 (design B baseline) |
| **Feeds** | DN-84 (the decided direction it implements) · M-740 (the `.myc` self-hosting port implements the settled shape once) |
| **Grounds** | `research/29-recursion-depth-and-stack-safety-RECORD.md` · DN-84 §4/§5/§7/§11 · DN-05 (amended §4.4) · DN-56/M-969 (freeze) · ADR-014/KC-3 · RFC-0007 §4.5/4.6 · RFC-0014 (effect budgets) · DN-71 §8 (Substrate release) · DN-36 6(g) (TCO) |
| **Decides** | (1) convert the **L1 evaluator + L0 reference interpreter** to explicit heap **work-stack** machines, **each keeping its own frame/loop shape** (L0 stays substitution-based) but sharing (2) one **global deterministic depth budget** on a **single machine-independent metric** (§4.0), plus a **memory ceiling** that counts the *actual* dominant heap (§4.2); (3) a deterministic **floor + dynamic headroom** reconciliation (amends DN-05); (4) **iterative destruction** (`Drop`/`Clone`/hash) across the **full** recursive `mycelium-core`/`mycelium-l1` surface via a **within-freeze behavior-preserving-hardening channel** (§6); (5) **TCO in the evaluators, guarded by a no-pending-post-work precondition** (§4.6); (6) frontend passes stay **guarded recursion + fine-grained `ensure_sufficient_stack`** + a **work-step budget**; (7) an opt-in, non-deterministic, CLI-flag-only, corpus-excluded `--unbounded` mode; (8) **extract only the shared budget + guarded-stack primitive** (`mycelium-workstack`), not one universal machine. |

> **Posture (transparency / VR-5 / G2) — corrected in Rev 2.** This RFC changes **resource behavior only**:
> a deeply-nested input becomes a **never-silent explicit refusal** instead of an uncatchable `SIGABRT`,
> and legitimate deep-but-bounded work runs. The **honest correctness invariant** (Rev-1 over-claimed
> "same error variants" — see §11-C2) is: **(I1)** when *all three* paths accept, they yield the **same
> observable value**; **(I2)** each path, on an over-budget input, **refuses never-silently** (an explicit
> error, never `SIGABRT`/hang); **(I3)** on the **same single metric** (§4.0) all three paths cross the
> accept/reject boundary at the **same threshold** at or below the deterministic floor (§4.4), refusing
> with **one canonical error variant** (§5.1). No new observable *value* semantics; the M-210 differential
> plus a **new cross-path error-parity differential** (§5.1) are the checked basis. No guarantee tag is
> upgraded; every bound is `Declared` with `Empirical` differential agreement, never `Proven`.

---

## 1. Problem

Recursive interpreters/passes overflow the host stack on deep input, turning an intended never-silent
refusal into an uncatchable `SIGABRT` — a robustness gap **and** a DoS surface. `research/29` mapped it;
load-bearing facts: **`myc run`'s L0 interpreter (`mycelium-interp`) has *no* depth budget** (`DepthLimit`
defined-but-never-constructed; verified) — a crafted deep value SIGABRTs `myc run`, remotely reachable via
a hostile spore; a **recursive-`Drop` stack bomb** on the frozen core overflows *on destruction*; **no
TCO** + a non-tail stdlib idiom means a `.myc` compiler pass can't iterate large worklists without burning
depth; and the scattered budgets (parser 256 / eval 64 / rest 4096) + the fixed 256 MB stack (~24.6k
*checker* frames) are toolchain-fragile (the ADR-041 near-miss).

## 2. Goals / Non-goals

**Goals.** No input SIGABRTs any interpreter/pass (incl. on destruction and during unwind, and on
no-grow targets). Every over-limit is an explicit, deterministic, machine-independent refusal *on a single
metric*. Legitimate deep work runs; a `.myc` compiler pass iterates large worklists in bounded depth.
Secure-by-design (incl. memory-DoS and CPU-DoS). The settled shape is what M-740 implements once.

**Non-goals.** No new L0 node/prim or surface syntax. No change to observable **values** (I1) or, above
the floor / outside the metric, no promise of identical error variants beyond I2's never-silent guarantee.
No flat-AST/arena rewrite now (a boot10-era option — RR-29 §3). No wholesale conversion of frontend passes.

## 3. Design overview — two layers, one budget, one metric

```
          ┌──── ONE deterministic budget (§4.2) on ONE machine-independent metric (§4.0) ────┐
 frontend │ parser · checker · elaborator · mono · ambient · totality   (B) guarded recursion │
 (guard)  │   + fine-grained ensure_sufficient_stack (§4.3) + a work-step (CPU) budget (§4.2) │
          ├──────────────────────────────────────────────────────────────────────────────────┤
 eval     │ L1 evaluator (env)  ·  L0 reference interpreter (SUBSTITUTION — kept)  (D) convert │
 (convert)│   each its own frame/loop; TCO guarded by no-pending-post-work (§4.6); memory-cap  │
          ├──────────────────────────────────────────────────────────────────────────────────┤
 destroy  │ Node · Value · Datum · CoreValue · L1Value  (+ Clone, Canon hash, unwind-reachable)│
          │   iterative destruction across the WHOLE class (§4.5), within-freeze channel (§6)  │
          └──────────────────────────────────────────────────────────────────────────────────┘
 shared   │ mycelium-workstack (leaf): the depth+memory BUDGET + a guarded-stack helper ONLY —  │
          │   NOT one universal machine; each consumer keeps its bespoke frame type (§4.1)      │
```

## 4. The decided architecture (ratified 2026-07-03; hardened Rev 2)

### 4.0 One machine-independent depth metric (NEW — resolves §11-C1)
The budget is charged on **one metric, uniform across all three IRs and tail iterations: one unit per
source-level call/β-reduction boundary** (a user function application or a `Fix` unfold), **not** per
internal IR node. Rationale: L1 charges per-`Expr`-node today (eval.rs:781), L0 would charge structural
redex nesting, AOT charges per-frame, and L0's *curried binary* `App` vs L1's *n-ary* `App` make
`f(a,b,c)` depth-3 vs depth-1 — so aligning only the scalar (Rev 1) gives three different thresholds for
one source. Charging at the **source-call boundary** is the one quantity all three machines share. **Gate
(W0):** a property test asserting the *same source* hits the *same threshold* on all three paths; data-spine
depth (a `Cons` literal) is charged by element uniformly. Tail iterations (§4.6) do **not** charge depth
(they reuse the frame) — stated explicitly and reconciled with the differential (all three loopify tail
calls, so all agree).

### 4.1 Convert the evaluators; extract only the shared *budget + guarded-stack*, not one machine (resolves §11 workstack1/3/4)
Rewrite the **L1 evaluator** SCC and the **L0 reference interpreter** so control recursion is O(1) host
stack. **Each keeps its own frame/loop shape** — L0 stays a **substitution** machine (fuel-loop + iterative
`subst`/redex-search; it has no environment to reify, RR-29 §1.5), L1 an **env** machine (a `Vec<Frame>`
CEK-style loop reifying its interleaved post-child work — §4.6). The **shared** extraction
(`mycelium-workstack`, a `#![forbid(unsafe_code)]` leaf, downward-only per DN-68) is **only**: the
`RecursionBudget` (depth on the §4.0 metric + the memory ceiling §4.2), the never-silent
`DepthExceeded`/`OutOfBudget` surface, and a thin `ensure_sufficient_stack` guard helper — **not** a
universal `WorkStack<Frame>` forcing three different machines into one abstraction. Each consumer keeps its
bespoke `Frame`. **Deps-cycle fix (§11 workstack4):** the memory ceiling *is* RFC-0014's `Alloc`
`EffectBudget` (interp-resident, `aot.rs:261`); the leaf exposes only **counters/limits**, and the
**charge happens consumer-side** at each machine's frame-push/env-insert site — the leaf never depends on
`interp`. **Common-mode risk (§11 workstack1):** because the three paths would share the budget/guard code,
the differential can no longer cross-validate that shared core; **W0 adds dedicated in-isolation property +
mutation tests** on the budget/guard against a synthetic frame type, and adds `mycelium-mlir` +
`mycelium-workstack` to the mutants/fuzz scope. The `.myc` `compiler.workstack` nodule is the portable form
M-740 reuses. **Budget home (§11 workstack7):** `mycelium-workstack` is created in **W1** (not W3½) so the
budget lands in its final crate once; W3½ extracts only the guarded-stack *machine* refactor.

### 4.2 One budget on the metric + a memory ceiling that counts the *real* dominant heap (resolves §11-C3/High)
Retire the scattered constants for **one workspace `RecursionBudget`**: a `depth` ceiling on the §4.0
metric (**default 4096**; parser 256 / eval 64 raised to it — but eval's raise is **held to W5**, §7) and a
**memory ceiling** that counts the **actual dominant allocation**, not just frame-structs (Rev-1's
frame-struct-only cap was theatre — §11-C3): (a) **L0 substitution duplication** — `subst` clones the value
per `Var` occurrence (`lib.rs:695`), so `(λx.C[x,…,x]) v` duplicates `v` k× per β-step → **exponential heap
at trivial fuel** (fuel bounds step *count*, not per-step copy size); the ceiling counts **live
substituted-term bytes**. (b) **Captured environments** (`aot.rs` frame `Env=HashMap`; L1 `scope` Vec) —
counted at the insert site via `F::heap_bytes()`. **Determinism invariant (§11 High obj7/det):** a startup
assertion **`mem_ceiling ≥ depth_floor × max_frame_bytes(over ALL three machines)`** so the memory ceiling
can **never** bind at or below the depth floor — keeping the (frame-size-dependent, hence
machine-dependent) memory limit **off** the observable accept/reject boundary within the floor. A per-target
`max_frame_bytes` **baseline gate** in CI trips on frame growth (so a toolchain bump fails CI, not
production — the ADR-041 lesson). **Process-wide arena (§11 High obj11):** a shared atomic byte counter
every work-stack charges against, refusing `OutOfBudget` when the *sum* over concurrent passes (LSP
re-analyses, `eval_core_parallel` workers, spore batch) would exceed a per-process ceiling — the per-pass
cap alone multiplies under concurrency. **Frontend work-step (CPU) budget (§11 High obj12):** the frontend
carries only depth today; §4.7's `check_list` flatten *admits* large-N literals that then flow into the
`O(N²)` re-walks (`mono::rewrite_app`, `mir count_occurrences` `emit.rs:186`) → CPU-DoS. Add a node-visit
**work-step budget** that refuses never-silently before N grows large enough to make the un-fueled `O(N²)`
walks a DoS (and fix/defer the named `O(N²)` re-walks before the raise). All limits deterministic (fixed
defaults, tunable per-invocation); accept/reject a function of `(source, budget)` on the §4.0 metric.

### 4.3 Grow the host stack: fine-grained `ensure_sufficient_stack`, runtime-gated (resolves §11 High det14/sec19/sec20)
Rev 1's "coarse `maybe_grow` + bounded-stride re-check" was an internal contradiction (a stride `S` is
overrun-safe only if `red_zone ≥ S × max_frame` — the exact frame-size coupling it claimed to avoid).
**Rev 2:** use **fine-grained `stacker::ensure_sufficient_stack`** (stride-1, rustc's pattern) with a fixed
generous `red_zone`, at each genuine recursion point of the still-recursive guarded passes — `unsafe`
contained in the `mycelium-workstack`/`mycelium-stack` leaf (ADR-014/KC-3). **No-grow targets (wasm32; psm
is a silent no-op — RR-29 §3):** the startup assertion and floor-honorability are gated on **runtime
growth-availability** (`stacker::remaining_stack` probe), **not** the cargo feature flag. Where the
physical (post-grow) ceiling `< floor × max_frame`, the machine **refuses to start with an explicit
diagnostic** (or surfaces a lower ceiling) — **never** a silent SIGABRT below the floor (so §4.4's "floor
is machine-independent" holds *only where growth is available*, stated honestly). The no-op case is
**detected and surfaced** (G2), never a silent degrade.

### 4.4 Deterministic floor + dynamic headroom (amends DN-05, append-only)
The AOT machine's memory-derived ceiling (DN-05/M-349, `[10k, 2M]`) is reconciled as **a deterministic
FLOOR all three paths honor** (the global default 4096, on the §4.0 metric; the corpus stays ≤ floor)
**plus dynamic headroom above it the differential never exercises**. The **AOT path in the differential is
pinned to the floor** via `run_core_with_budget`'s explicit ceiling (the dynamic `[10k,2M]` applies only
outside the differential, as headroom); the memory ceiling and byte accounting apply to AOT too (§4.2
invariant). So the observable boundary is deterministic up to the floor on one metric; memory-awareness
survives above it. DN-05 gains an append-only amendment.

### 4.5 Iterative destruction across the FULL recursive class (resolves §11 High freeze8, Med freeze9/10, Low freeze11)
Rev 1 under-counted ("one frozen-core edit"). The **complete** recursive-destruction surface, all made
iterative: derived **`Drop`** *and* derived **`Clone`** (the front-door `let mut current = node.clone()`
at `lib.rs:555` SIGABRTs *before* `step` runs — so `myc run` still aborts after W4 unless Clone is fixed),
**`Canon::node`/`content_hash`**, and **`PartialEq`**, over `Node`/`Value`/`Datum`/`CoreValue`/`L1Value` —
**plus the unwind-reachable types** (`Frame`, captured `Env`, checker `Ty`, elaborator intermediates): a
deep guarded pass that panics unwinds ~budget-deep dropping frame locals, so any still-recursive reachable
type re-overflows → **double-panic abort** (the very SIGABRT we set out to kill). Mechanics: a **single
shared heterogeneous worklist** across the cross-recursive `Datum↔CoreValue↔Value`/`Node` cluster
(a per-type worklist re-recurses at each type hop); **`mem::replace`/`take`-based moves** — because `impl
Drop` **forbids by-value field move-outs (E0509)**, every owned destructure (e.g. `eval_for`'s
`let L1Value::Data{..} = spine` at `eval.rs:1154`) is converted to by-ref + `mem::replace`; the iterative
`Drop` **avoids allocation during unwind** (intrusive next-pointer / preallocated scratch — a `Vec::alloc`
inside `Drop` during OOM/unwind is itself an abort). **Recorded precondition (Low freeze11):** iterative
`Drop` is double-free-safe *only* under the **Box-owned / acyclic / no-shared-spine** invariant (holds
today); a future `Rc`/`Arc` intern cache on the spine **invalidates** it — noted as a checked precondition.
**Honesty:** this is **not** "purely additive" — it couples to the W5 eval-SCC rewrite (E0509 blast radius)
and lands **coordinated with W5**, not before it (§7).

### 4.6 TCO guarded by a no-pending-post-work precondition (resolves §11-C4, Med tco31/32)
TCO in the evaluators — **but a "tail" call in Mycelium is not truly tail**: `invoke` runs
`release_if_abandoned` (Substrate release + `ReleaseEvent`, DN-71 §8/G2) and the **return-guarantee assert**
*after* the body (`eval.rs:636-642`). Naive frame-reuse would **silently skip** both — leaking a Substrate
handle (missing `ReleaseEvent`) and accepting a value with a weaker guarantee than the callee's `@Proven`
return demands (a VR-5 violation). **Checked precondition:** apply frame-reuse **only** when the caller's
`invoke` frame has **no pending post-work** — no `sig.ret.guarantee` index and no Substrate-typed params;
otherwise keep the frame and charge depth. **Witness tests:** a tail call from a fn with a return-guarantee
index, and from a fn with a Substrate param, must NOT be TCO'd (assert the release/assert still fire).
**Honest scope (tco31):** TCO gives non-tail stdlib idioms (`map`/`filter`/`reverse`) bounded-depth
**safety**, not unbounded **success** — a 10k non-tail recursion still refuses at 4096; meeting the "10k
worklist" Goal requires the stdlib be authored **tail-recursive or via `for`** (an explicit dependency for
M-740). **EXPLAIN marker (tco32):** the elided-frame record is **not** a bare count — it carries per-callee
identity + iteration count, and a **bounded ring buffer of the last K elided calls**, so a deep tail chain
that errors yields an actionable trace (house rule #2 in substance, not just letter).

### 4.7 Frontend: guard + grow + work-budget; close the FULL guard-hole set (resolves §11 High comp41-48)
Frontend passes stay guarded recursion on the grown stack (§4.3) + the work-step budget (§4.2). **Close
every guard hole RR-29 mapped** — Rev 1's list was narrower than the research (§11 High): add
**`mycelium-lsp` `render_node` + `lint::walk`** (the named-priority editor-buffer surface, no
`mycelium-stack` dep today), the **`mycelium-fmt` render family** (widened by the parser raise),
**`mycelium-transpile`** + **`mycelium-doc`** IR walkers, **`is_pure`/`plan_parallel`** (`parallel.rs`),
`write_canon` (`lower.rs:212`), `mycelium-mir-passes`, and the checker's `usefulness`/`decision`/`grade`
(independently verify `affine`/`fuse` before waving them off) — each charges the budget or wraps in the
guard. **`check_list` flat-loop** (route data-shaped depth through iteration, not the control budget) —
**and apply the same treatment to its structural twin** `usefulness::useful`/`decision::compile_rows` (the
tuple/ctor-*arity*→depth spine, surface-reachable, not bounded by the 256 nesting cap), or document the
wide-tuple asymmetry explicitly. Convert `parse_type_ref` to an explicit stack only if profiling after a
raise demands it. **DoD item 1 is scoped to this concrete crate list** (so "no input SIGABRTs any pass" is
literally true, not aspirational).

## 5. Security requirements (secure-by-design, standing)

Part of the DoD (RR-29 §4 + §11 security lens):
- **Real memory accounting + process arena + frontend work-budget** (§4.2) — closes the memory-DoS
  (incl. L0 exponential substitution) and CPU-DoS the frame-struct cap missed.
- **Untrusted-input coverage** measured against **spore-resolved remote `.myc`** and **LSP editor buffers**,
  not just local files; the L0-interp hole (§1) is the priority.
- **Durability gates (DoD preconditions):** add `mycelium-l1` **+ `mycelium-mlir` + `mycelium-workstack`**
  to `cargo-mutants` with **remove-guard witness tests**; **depth-structured fuzz** over
  parse/check/elaborate/interp + the value walks + the shared budget/guard in isolation; **census tests are
  `#[ignore = "Wn"]`-tagged** (or REJECT_EXPECTED xfail) so the intentional W0→W3/W4 red window doesn't
  block change-scoped gates (§11 Med wave40).
- **`--unbounded`:** CLI-flag-**only** (never manifest/env/LSP-config), never-silent banner, corpus-excluded,
  refused in CI — with a test that the corpus runner rejects it.
- **Supply chain:** add `mycelium-stack`/`mycelium-workstack` (+`mycelium-l1`) to
  `scripts/checks/unsafe-per-use.sh` audit-A; pin `stacker`/`psm` exact versions; THIRD-PARTY + `about.toml`;
  a geiger baseline so a new unsafe dep is never silent.
- **Keep** serde_json's default-128 limit; re-audit if any binary `Value`/`Node` codec is added.
- **Periodic adversarial re-review** each wave before merge.

### 5.1 Error parity + the differential gate (NEW — resolves §11-C2)
Rev 1's differential compared only **success values** (`differential.rs:136`) and the paths already diverge
`DepthLimit{usize}` vs `L1Error::DepthExceeded{u32}` vs `FuelExhausted` (`recursion_differential.rs:148`
tolerates it). Rev 2: **pick one canonical over-budget error variant + width** shared across paths
(reconcile `DepthLimit`/`DepthExceeded`/`FuelExhausted` first — which budget "wins" on a given input is
part of the §4.0 metric contract), and **add a cross-path error-parity+threshold differential over deep
inputs as a W0 gate**: for an input past the floor, all three paths must refuse with the canonical variant
at the same metric threshold. This gate **fails today** — reconciling the existing divergence is a W0
precondition, not an afterthought.

## 6. Freeze compatibility (KC-3 / DN-56) — a defined within-freeze hardening channel (resolves §11 High freeze6)
Converting the evaluators' recursion strategy adds **no L0 node/prim/surface** — an implementation swap
behind the frozen semantics boundary (DN-56 §6 governs the kernel *surface*, unchanged; RR-29 §1.4/§0.4).
The frozen-core **iterative-destruction** edits (§4.5) do **not** fit DN-39 (a default-DENY bar for
*admitting a new component into the TCB*, not editing a frozen type's impls — §11 High freeze6). This RFC
defines a scoped **within-freeze "behavior-preserving hardening" channel** with an explicit bar: **(a)** no
observable value/error/order change (I1/I2/I3); **(b)** M-210 + error-parity differential green; **(c)**
mutation-witnessed; **(d)** a stated line — *only* recursion→iteration destruction/traversal transforms on
existing types, never a new type/variant/field or a semantic change (those remain DN-39-promotion or a
`core 2.0.0` supersession). Rationale the maintainer ratified: the freeze currently protects a semantics
**already unsound** for deep input (`SIGABRT` ≠ never-silent) — this **restores** the guarantee the freeze
assumes. **This channel itself is a maintainer-ratified process addition (flagged for the DN-56 append-only
pointer).**

## 7. Staged implementation plan (waves — Phase 4, isolated worktrees; ownership + ordering explicit)

Each wave lands differential-green (M-210 **+** the §5.1 error-parity gate) and is adversarially re-reviewed
before merge. **Shared-file owners** (resolves §11 Med wave39): root `Cargo.toml` + `.cargo/mutants.toml` are
**integrator-owned** — waves FLAG edits, the integrator applies them (feed-as-ready). **Ordering edges are
in the table**, not just prose (resolves §11 High wave35).

**Status (2026-07-03):** **W0, W1, and W2 landed.** W2 = the host-stack **grow** infrastructure:
`mycelium-stack` gains a fine-grained runtime-gated `stacker` grow (exact-pinned `=0.1.24`; still
`#![forbid(unsafe_code)]` — no authored unsafe, the switch is contained upstream) with a never-silent
no-grow refusal (`growable_ceiling_honors_floor`, wasm-safe); `mycelium-workstack` routes
`ensure_sufficient_stack` through it (layered on the worker base, non-regressing) and adds the
`check_startup` `mem_ceiling ≥ floor × MAX_FRAME_BYTES(384)` gate; a frame-size CI baseline pins the
value structs. Honest scope: the *per-recursion-point* stride-1 grow is consumer-side wiring for
W3½/W4/W5; the AOT `Frame` in-crate pin is a tracked residual. W2 `Enacted` for the grow/startup scope.

**Status (2026-07-03):** **W0 and W1 landed.** W0 = the safety-net gates (§4.0 metric test, §5.1
error-parity differential `#[ignore="W5"]`, the RR-29 guard-hole census, depth-structured fuzz, and the
mutants/unsafe-audit scope). W1 = the **`mycelium-workstack`** budget crate (`RecursionBudget` on the
metric, a memory ceiling, a `ProcessArena`, the canonical `BudgetError::DepthExceeded{u32}`, and the
`ensure_sufficient_stack` W1 passthrough) **plus** the frontend guard-hole wiring (`mycelium-l1` checker,
the `check_list` data-vs-control iteration fix, the parser 256→4096 raise, and
`fmt`/`lsp`/`transpile`/`doc`/`mir-passes`) — the 14 frontend census tests un-ignored and green.
**Scoping correction (G2):** `write_canon` (frozen core) and `is_pure`/`plan_parallel` (interp trusted
base) were re-tagged off W1 to W3/W4 (maintainer checkpoint). **W1 residuals** (tracked, not silent): the
recursive-`Drop` bomb on deep fixtures (W3 class; `mycelium-doc::ir::Node` is a newly-found member); in
`mir-passes`, `eval(&RcNode)`/`emit_elided`/`emit_reuse` and the `count_occurrences` O(N²) re-walk (W2);
and `syn`'s own unbudgeted recursion (third-party, dev-tool). **W2–W6 pending**; W3/W4/W5 gated on the
maintainer frozen-core checkpoint.

| Wave | Scope | Depends on | Gate |
|---|---|---|---|
| **W0 — Safety net + metric + error parity** | depth-structured fuzz; `mycelium-l1`+`mlir`+`workstack`→mutants; the §4.0 metric property test; the §5.1 error-parity differential (canonical variant reconciled); guard-hole census (`#[ignore="Wn"]`-tagged) | — | new gates exist + tagged; metric test passes; error-parity gate defined |
| **W1 — Budget crate + frontend wiring** | create `mycelium-workstack` (final home); one `RecursionBudget` (depth-on-metric + memory ceiling + work-step); wire through frontend guard holes (§4.7); **parser 256→4096** (eval's raise HELD to W5) | W0 | full differential + error-parity green; census (frontend subset) closed |
| **W2 — Grow (fine-grained)** | `ensure_sufficient_stack` (stride-1) in the leaf; runtime grow-probe; wasm/no-grow refuse-to-start; startup `mem_ceiling ≥ floor×max_frame` assertion; frame-size CI baseline; supply-chain (§5) | W0 | builds all targets; no-op surfaced; assertion holds |
| **W3½ — Extract guarded-stack machine** | refactor the AOT env-machine's loop onto the shared guard/budget **behavior-preserving** (budget already in W1's crate) | W1, W2 | AOT differential + error-parity green (oracle unmoved) |
| **W3 — Iterative destruction (full class)** | iterative `Drop`+`Clone`+`Canon`+`PartialEq` over the full §4.5 set incl. unwind-reachable; E0509 move-out audit; no-alloc-in-Drop | W1; **coordinated with W5** | deep-chain construct+destruct+unwind tests; identity/eq bit-identical; within-freeze bar (§6) |
| **W4 — L0-interp (substitution) work-stack** | iterative `step`/`subst`/read-off onto the shared budget; construct `DepthLimit`; route `myc run` through it; front-door `node.clone()` (W3) iterativized | W1, W2, W3½, (W3 Clone) | 3-way + error-parity green; `myc run` refuses deep input, never aborts |
| **W5 — L1-eval (env) work-stack + TCO + eval raise** | eval SCC → CEK `Vec<Frame>` (reify post-child work, §4.6); TCO w/ no-pending-post-work precondition + witness tests + EXPLAIN ring-buffer; **now raise eval 64→4096** | W1, W2, W3½, W3 | differential + error-parity green; tail-safe; TCO witnesses pass |
| **W6 — Data-spine iteration** | `check_list` + `usefulness`/`decision` flat-loop; residual frontend conversion if profiling demands | W1, W5 | large data literal bounded by memory+work-budget, not depth; corpus green |

Disjoint waves run as **parallel workflows in isolated worktrees**, feeding an **integrator via feed-as-ready
`pipeline()`**. **Model roles:** **Fable** plans/QC-reviews; **leaf/impl agents = Opus/Sonnet** (ADR-038);
mechanical **Fable→Opus safety-fallback**. **Scoped in-branch doc updates as-you-go**; the integrator
reconciles only the shared indices. M-740 pass-authoring unblocks once W5 lands.

## 8. Alternatives considered
Convert-everything (rustc declined it — larger/riskier); guard-only (leaves evaluators depth-limited —
fails the worklist Goal); auto-scale-to-RAM (machine-dependent — rejected except `--unbounded`); flat/arena
AST (dissolves the class but a large retrofit — deferred to boot10); **one universal `WorkStack<Frame>`**
(rejected Rev 2 — L0 is substitution, not env; a shared machine is a leaky over-abstraction, §11 workstack3).

## 9. Definition of Done
- No input SIGABRTs any pass **in the §4.7 crate list**, on construction, destruction, **or unwind**, incl.
  spore-remote/LSP shapes and no-grow targets (verified by depth-structured fuzz + remove-guard mutants).
- `myc run` refuses a deep value with the canonical error, never an abort (W4, incl. the front-door Clone).
- M-210 **and** the §5.1 cross-path error-parity+threshold differential are green **at/below the floor** on
  the §4.0 metric; documented divergence only in the dynamic headroom.
- One deterministic budget (depth-on-metric + memory ceiling + process arena + frontend work-step) governs
  every path; the `mem_ceiling ≥ floor×max_frame` invariant + frame-size CI baseline hold.
- Iterative destruction lands via the §6 within-freeze channel; the full §4.5 class is proven-complete
  (a drop/clone/unwind-depth witness per recursive type incl. an alternating-type deep chain).
- TCO is precondition-guarded (release/assert never skipped — witness tests); tail chains have actionable
  EXPLAIN; the non-tail-idiom scope is stated.
- Supply-chain + `--unbounded` guards in place and tested; `O(N²)` re-walks fixed or explicitly deferred
  before the raise.
- DN-05 amended (floor+headroom); the §6 within-freeze channel recorded (DN-56 pointer); RR-29's guard-hole
  census closed; every claim graded honestly (`Declared`/`Empirical`, no `Proven` — VR-5).

## 10. (reserved)

## 11. Phase-3 adversarial review — confirmed objections & resolutions
An 8-lens adversarial-review workflow (Opus/Sonnet leaves; ~1.15M tokens; 1 refuted) attacked Rev 1 before
implementation. **4 Critical + 15 High source-confirmed** objections, all resolved above:
- **C1** no common depth metric → **§4.0** (charge at source call/β boundary).
- **C2** "same error variants" unverified + already false → **§Posture (I1–I3)** + **§5.1** error-parity gate.
- **C3** byte cap missed the dominant memory (L0 subst duplication, captured envs) → **§4.2** real accounting,
  process arena, and a frontend work-step budget.
- **C4** TCO silently drops Substrate release + return-assert → **§4.6** no-pending-post-work precondition.
- **High:** E0509 move-out breakage + fuller frozen-core class (Clone/Canon) → **§4.5**, coordinated with W5;
  path-dependent byte cap → **§4.2** invariant + CI baseline; extraction common-mode + L0-is-substitution +
  deps-cycle → **§4.1** (extract budget+guard only, consumer-side charge, isolation tests); no process arena
  → **§4.2**; no frontend CPU budget → **§4.2**; wasm/no-grow SIGABRT + coarse-vs-stride contradiction →
  **§4.3** (fine-grained, runtime-gated); W1-raise-before-W2-grow + eval excluded → **§7** (eval raise → W5,
  ordering in table); census narrower than RR-29 → **§4.7**; DN-39 wrong instrument → **§6** within-freeze
  channel.
- **Med/Low:** continuation/scope-restore spec, cross-recursive Drop worklist, unwind-reachable + no-alloc
  Drop, AOT bound composition, budget-crate relocation, `~24.6k` qualifier, tuple-arity twin, TCO marker
  content + non-tail honesty, shared-file owners, census red-window tags, Box-owned Drop invariant — all
  folded into §4.2/§4.3/§4.5/§4.6/§4.7/§7. Full attack transcript: the workflow journal (run
  `wf_1b970eae-a75`).

## Meta — changelog

- **2026-07-03 — W2 landed (host-stack grow; M-979).** `mycelium-stack` gains a fine-grained,
  runtime-gated `stacker` grow (exact-pinned `=0.1.24`, `psm 0.1.31`; still `#![forbid(unsafe_code)]` —
  the switch is contained upstream, ADR-014) with a never-silent no-grow refusal
  (`growable_ceiling_honors_floor`); `mycelium-workstack` routes `ensure_sufficient_stack` through it
  (layered on the worker base — non-regressing) and adds `check_startup` (`mem_ceiling ≥ floor ×
  MAX_FRAME_BYTES`) plus a frame-size CI baseline (relocated to `mycelium-l1/tests/` — a workstack
  dev-dep back-edge would have closed a normal+dev cycle the acyclic gate rejects). Supply chain
  (THIRD-PARTY regen, unsafe-audit, geiger placeholder) reconciled. Honest scope: per-recursion-point
  grow is W3½/W4/W5 consumer wiring; the AOT `Frame` pin is a tracked residual. W2 `Enacted` (grow scope);
  RFC stays Accepted. (VR-5/G2.)
- **2026-07-03 — W1 landed (budget crate and frontend wiring; M-979).** The `mycelium-workstack` leaf
  crate (`RecursionBudget` on the §4.0 metric, a memory ceiling, a `ProcessArena`, the canonical
  `BudgetError::DepthExceeded{u32}`, the `ensure_sufficient_stack` W1 passthrough, and the §4.2 invariant
  fn; `#![forbid(unsafe_code)]`, DN-68 downward-only, consumer-side charge) and the frontend guard-hole
  wiring (`mycelium-l1` checker, the `check_list` iteration fix, the parser 256→4096 raise, and
  `fmt`/`lsp`/`transpile`/`doc`/`mir-passes`). 14 frontend census tests un-ignored and green; full
  `just check` green (differential and census; error-parity stays `#[ignore="W5"]`). Trusted-base holes
  (`write_canon`, `is_pure`/`plan_parallel`) re-tagged off W1 to W3/W4 (maintainer checkpoint). Residuals
  tracked (Drop bomb incl. new `doc::ir::Node`; `mir` `eval(&RcNode)` and O(N²); `syn`). W1 `Enacted` for
  the frontend scope; RFC stays
  Accepted. (VR-5/G2 — Empirical/Declared, no Proven.)
- **2026-07-03 — W0 landed (Phase-4 safety net; M-979).** First wave, no behavior change / no
  frozen-core edits: the §4.0 metric property test, the §5.1 error-parity differential
  (`#[ignore="W5"]`; canonical over-budget variant fixed as `DepthExceeded{u32}` on the metric,
  reconciling the interp/AOT `EvalError::DepthLimit{usize}` in W4/W3½), the RR-29 guard-hole census
  (per-crate `#[ignore="Wn"]` real-repro tests across eight crates), depth-structured fuzz (the interp
  target reproduces the known `SIGABRT`), and the `just mutants` / unsafe-audit scope additions
  (`mycelium-l1`, `mycelium-mlir`, `mycelium-stack`). RFC stays **Accepted**; W1–W6 pending. Every
  claim `Declared`/`Empirical`, no `Proven` (VR-5/G2).
- **2026-07-03 — Ratified `Proposed → Accepted` (Rev 2; maintainer).** The maintainer's "accept/ratify"
  directive ratifies the adversarially-hardened Rev 2, **including** the three items flagged for their
  call: the §6 **within-freeze behavior-preserving-hardening channel** (a DN-56 process addition — DN-56
  gains an append-only pointer), the §Posture **I1–I3 correctness invariant** (dropping Rev-1's over-claimed
  "same error variants"), and the §4.0 **source-call-boundary depth metric**. Phase-4 implementation (the
  seven waves §7) is unblocked; `Accepted → Enacted` per-stage as each wave lands differential + error-parity
  green. Append-only (VR-5/house rule #3).
- **2026-07-03 — Rev 2: adversarial-review hardening (M-979 Phase-3).** Phase-3 review found 4 Critical +
  15 High source-confirmed flaws in Rev 1's spec (the strategy held; the spec did not). Resolved all: a
  single machine-independent depth **metric** (§4.0); an honest correctness invariant + **error-parity
  differential** (§Posture/§5.1); a **real memory ceiling** (L0 subst duplication + envs) + process arena +
  frontend work-step budget (§4.2); **TCO precondition** so Substrate release/return-assert are never
  skipped (§4.6); **full iterative-destruction class** incl. Clone/Canon/unwind + E0509 audit, coordinated
  with W5 (§4.5); **fine-grained runtime-gated grow** + wasm contract (§4.3); **extract budget+guard only**
  (L0 stays substitution; deps-cycle fix) + isolation tests (§4.1); a defined **within-freeze hardening
  channel** replacing the DN-39 mis-fit (§6); wave ordering/owners/census-tags (§7). Status stays
  **Proposed** → Accepted on ratification. (VR-5/G2.)
- **2026-07-03 — Rev 1 / Proposed (M-979 Phase-2).** Initial architecture from `research/29` + the four
  maintainer ratifications; extract shared primitive; 7-wave plan. (Superseded in-place pre-ratification by
  Rev 2 — append-only history preserved in git; this RFC is not yet Accepted.)
