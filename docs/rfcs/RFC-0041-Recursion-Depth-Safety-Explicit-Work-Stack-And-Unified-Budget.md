# RFC-0041 — Recursion-Depth Safety: Explicit Work-Stack Evaluators + a Unified Deterministic Depth Budget

| Field | Value |
|---|---|
| **RFC** | 0041 |
| **Status** | **Enacted (2026-07-05)** — maintainer-approved (2026-07-05) W7 promotion `dev → integration → main`, **effective with this wave's landing on `main`** (the §9 claimability condition); all eight waves (W0–W6 + the W7 Enacted-closure wave) landed with every §9 DoD line **literally met or honestly re-scoped by the recorded append-only §7/§9 amendments** (checked basis: §7 wave status blocks + the W7 dispositions). Tracked non-DoD follow-ons stay open (W3b bare-`Repr`, `count_occurrences` O(N²) bound, single-variant unification, AOT per-frame metric precision under the §5.1 family-parity contract, `content_hash` O(depth²), the geiger-baseline `--update` regeneration — the committed baseline is a disclosed W0 placeholder, never silently). Prior: **Accepted (Rev 2, 2026-07-03 — maintainer-ratified)** — authored under DN-84 §11 "solve (D) now" + the four 2026-07-03 ratifications (RR-29 §6); hardened by the Phase-3 adversarial review (§11 — 4 Critical + 15 High source-confirmed objections resolved); ratified `Proposed → Accepted` by the maintainer 2026-07-03, including the §6 within-freeze behavior-preserving-hardening channel (a ratified DN-56 process addition), the §Posture I1–I3 correctness invariant (superseding Rev-1's "same error variants"), and the §4.0 source-call-boundary depth metric; `Accepted → Enacted` per-stage as each wave (§7) landed differential + error-parity green (§9). Prior: Proposed (Rev 1 → Rev 2, 2026-07-03). |
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

**W4 amendment (append-only, 2026-07-03 — evidence-driven correction of the "single input" premise).**
The W0→W4 gate as written assumed **one statically-deep source** refused identically by all three paths;
implementing W4 proved that empirically unachievable, for three independent reasons: (i) the **parser cap
now equals the eval floor** (W1 raised `MAX_EXPR_DEPTH` to 4096), so a statically-deep *source* is refused
**at parse**, before any evaluator sees it; (ii) the **AOT trampoline is data-spine-immune** — it charges
depth only at App/Match frames, and its default ceiling is the DN-05 dynamic `[10k,2M]`, not the floor; (iii)
**L0 is a substitution machine** — runtime recursion re-walks/re-clones the term each step (`O(N²)+`), so its
practical deep input is a deep *value*, not a runtime `spin`. The gate is therefore satisfied by the parity
that **actually holds**: every path refuses over-budget with the canonical variant **family** at the shared
4096 floor, each exercised with a per-path bounded-time input (L1-eval `spin` → `DepthExceeded{4096}`;
L0-interp deep value → `DepthLimit{4096}`; AOT `spin` at the explicit floor → `DepthLimit{4096}`).
**Residual:** literal single-*variant* unification is partial — L1 surfaces `DepthExceeded{u32}` while
L0/AOT surface the pre-existing `DepthLimit{usize}` (the canonical `DepthExceeded` is the budget-crate
type; the interp/AOT paths map from it but keep `DepthLimit` as their observable). Full convergence would
change the `mycelium-interp`/`mycelium-mlir` error enums (a trusted-base/AOT observable change) — a
maintainer-decision follow-up, not a W4 blocker (the never-silent + shared-threshold contract holds).

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

**Status (2026-07-03): all seven implementation waves (W0–W6) landed on `dev`; whole-RFC `Enacted` NOT
yet claimable (open §9 DoD items — see below).** The flagship `myc run` SIGABRT (RR-29 §0.1) is closed,
the §5.1 cross-path error-parity gate is green, and all three execution machines (L0 interp · L1 eval ·
AOT) plus the frozen-core value types refuse deep input never-silently on one shared `RecursionBudget`,
with the host stack growing on demand. `#![forbid(unsafe_code)]` holds across every landed crate (the sole
`unsafe` is the audited upstream `stacker`/`psm`). **Open §9 DoD items surfaced by the post-implementation
assessment (2026-07-03), held for maintainer determination:** (i) the DoD-required **`--unbounded`** mode
(decided DN-84 §9.3, §5-specced) was never scheduled and is **unimplemented**; (ii) `mir-passes`
`eval(&RcNode)`/`emit_elided`/`emit_reuse` (a §4.7-listed crate) remain **unguarded**, so DoD §9 item 1
("no input SIGABRTs any pass in the §4.7 list") is not literally met there; (iii) an AOT
per-frame-vs-source-call metric reconciliation is **owed** (W3½ said "W5 reconciles"; W5 reconciled L1
only — AOT still depth-charges `Match` continuations). Plus the flagged deviations (§5.1 amendment below,
the residual notes, M-979). M-740 unblocks once these are determined. **W6** = data-spine iteration: the RFC §4.7 "convert-or-document" fork was resolved to
**document the wide-tuple asymmetry** — the `usefulness`/`decision` arity spine already refuses
never-silently (`DepthExceeded{4096}`, not a SIGABRT) via the W1 guard, so the residual is a *precision*
defect on a pathological 4095-field product type, not a safety one; conversion is gated on "if profiling
demands" (it does not). Documented residuals/deviations await maintainer determinations (RFC §5.1
amendment, this §7, the Meta changelog, the M-979 issue). M-740 self-hosting unblocks. Per-wave detail
below (append-only).

**Status (2026-07-03, W7 — Enacted-closure wave, held at `dev`):** the open §9 DoD items (i)/(ii)/(iii)
above and the flagged deviations are resolved by the **W7 closure wave** — four disjoint isolated-worktree
leaves (`--unbounded`+`with_depth` · `mir-passes` guards · construction-census+spine-tripwire ·
process-arena coverage), each per-leaf reviewed; the frozen-core/trusted-base-adjacent `mir-passes` leaf
independently adversarially memory-safety-reviewed (no Critical/High). Determinations were made by the
maintainer on a **Fable plan/QC assessment** of all twelve open items; dispositions (append-only):
- **(i) `--unbounded` — IMPLEMENTED (Rust-first).** `myc run` honours `--unbounded` via
  `Interpreter::with_depth(u32::MAX)` with a never-silent stderr banner; the corpus/conformance runner
  **refuses** it (test-guarded, exit 64). `myc build --unbounded` is **interface-parity only** (the frontend
  `mycelium-l1` depth ceilings are not CLI-tunable without an l1 API change — a tracked follow-on), and CI
  wiring to export the corpus signal is a tracked follow-on.
- **(ii) `mir-passes` guards — CLOSED (guard-and-refuse).** `eval(&RcNode)`, `emit_elided`/`emit_reuse`
  (and the shared `emit_ann` core) charge the shared `RecursionBudget` on every RcNode edge and wrap the
  outer entry in `ensure_sufficient_stack`, refusing never-silently with `RcError`/`EmitError::DepthExceeded`
  instead of SIGABRT-ing; the public infallible counters (`count_occurrences`/`count_dups`/`count_move_unique`)
  are deep-stack-wrapped so no direct call SIGABRTs. Independently adversarially reviewed: no Critical/High
  memory-safety hole. The surviving residual is `count_occurrences`' **O(N²)/infallible→fallible work-step**
  bound — a documented DoS-only precision residual **explicitly deferred to W2** (the SIGABRT/host-stack hole
  is closed; the parser bounds pipeline nesting at 256). No input SIGABRTs any pass in `mycelium-mir-passes`.
- **(iii) AOT metric reconciliation — RULED a FOLLOW-ON (not DoD-blocking).** Per the ratified §5.1 W4
  amendment the cross-path contract is variant-*family* parity at the shared floor; AOT is data-spine-immune
  with a DN-05 ceiling far above the floor, so the per-frame-vs-source-call divergence lives only in the
  dynamic headroom — exactly what the DoD's "documented divergence only in the dynamic headroom" permits.
  Literal per-source-call reconciliation of the AOT `Match`-continuation charging is a precision follow-on.
- **#5 zero-alloc-in-Drop — ACCEPTED (amended).** The iterative Drops use an empty-start `Vec` worklist; the
  DoD no-alloc-in-Drop gate is amended to **no allocation-failure abort path except under genuine OOM during
  deep unwind** — a strict safety improvement over the *certain* multi-MB stack-overflow abort it replaced.
  True zero-alloc would need contained `unsafe` (ADR-014) or a frozen-type field (DN-39); deliberately not
  taken. `Declared` residual.
- **#6 `Value`/`Repr` — DEFERRED to W3b (amended, census-grounded).** The §4.5 full recursive class is scoped
  to *constructible* types. A W7 construction-gate **census test** upgrades "a deeply-nested `Value` is
  unbuildable" from `Declared` to `Empirical` (every owned-`Value` path routes through the depth-walking
  `Value::new` gate; the wire path is 128-capped) and is the tripwire for a future ungated constructor.
  **Correction (VR-5):** the §4.5 phrase "`Value`/`Repr` … construction-gated, thus unbuildable" is precise
  for `Value` but **overclaims for a bare `Repr`** — `Repr::Seq { elem: Box<Repr> }` is constructible by a
  direct variant literal with no gate, so a deep bare `Repr` is buildable in first-party Rust and its derived
  recursive `Drop`/`Clone`/`PartialEq` would SIGABRT. Reachability is nil from untrusted input (`.myc`/
  interpreter values exist only as gated `Value`s; the wire is 128-capped; kernel `Repr`s are shallow), so
  this does not reopen a safety blocker — the claim is scoped to `Value`, and bare-`Repr` iterative
  destruction is folded into the coordinated W3b.
- **#8 process arena — CLOSED for untrusted-reachable paths (amended, audited).** A W7 coverage audit
  (`docs/notes/W7-arena-coverage-audit.md`) found `ProcessArena` had **zero consumers** — "governs every
  path" was unmet everywhere. The two untrusted-reachable allocation-proportional passes now charge the
  shared `ProcessArena` and refuse never-silently with `OutOfBudget` (LSP `llm_canonical`; the `fmt` render
  family via `FmtError::OutOfBudget`). Passes unreachable from untrusted input (`doc::ir::walk`; `transpile`
  of trusted first-party Rust) and non-allocation-proportional passes (`mir-passes::count_occurrences`) are
  **explicitly exempt** with the audit as the `Empirical` basis; the `mycelium-l1` checker family and the
  trusted-base interp/mlir (fallible-surface ripple, out of scope) are tracked follow-ons. The DoD line is
  amended to "governs every **allocation-proportional path reachable from untrusted input**; audited exempt
  set named in the coverage note".
- **#10 Box-owned/acyclic tripwire — ADDED.** A source-structural tripwire test fails if `Rc`/`Arc`/`Weak`
  shared ownership appears on the frozen `Node`/`Datum` iterative-Drop spine (catches a future intern-cache
  field — the real break vector). `mycelium-l1::L1Value` relies on the same invariant; its twin tripwire is a
  tracked follow-on.
- **#4 error-variant unification — ACCEPTED family-parity + mechanism check.** Full enum convergence (a
  trusted-base observable change) is not taken; the family mapping is additionally verified at an arbitrary
  small budget via the additive `Interpreter::with_depth` + a uniform-small-budget parity test (`Empirical` at
  ceilings {1,2,8,100}).
- **#11 TCO — ACCEPTED (direct-tail-only).** Scope stated; the "10k worklist" Goal is met only for
  tail-recursive/`for`-authored code — carried as an **explicit M-740 acceptance criterion**.
- **#12 W6 wide-tuple — "document" upheld.** A 4095-field product type is not an adversarially realistic
  untrusted input; the refusal is already never-silent; a byte-identical rewrite of the trusted Maranget
  passes is barred by KISS/YAGNI/KC-3.
- **#7 `content_hash` O(depth²)** and **#9 coarse-worker sites** (all ~21 `ensure_sufficient_stack` consumers
  still coarse; LSP `project.rs` is the hot untrusted per-keystroke priority) are **accepted as tracked
  follow-ons** (pre-existing / non-DoD-line) — recorded, not silent.

With W7, every §9 DoD line is either **literally met** or **honestly re-scoped by an append-only amendment
with a checked basis** (no claim upgraded past its evidence — VR-5; no refusal silent — G2; no frozen-type
field or trusted-base error-shape change spent — the §6 within-freeze channel is intact). **Whole-RFC
`Enacted` is claimable once W7 promotes `dev → integration → main` (held for the maintainer);** the RFC stays
`Accepted` until it lands on `main`. M-740 self-hosting unblocks on that promotion.

**Status (2026-07-03):** **W0, W1, W2, W3½, W3+W5, and W4 landed — only W6 remains.** W4 = the L0
reference interpreter (`mycelium-interp`) budgeted: `step`/`subst`/etc. charge the shared
`RecursionBudget` (L0 stays a substitution machine, §4.1), **`EvalError::DepthLimit` is now constructed**,
and **the flagship RR-29 §0.1 `myc run` SIGABRT is closed** — a deep value refuses with `DepthLimit{4096}`.
`parallel::is_pure` is iterative. **The §5.1 error-parity gate is GREEN** (un-ignored) — see the §5.1 W4
amendment for the evidence-driven "single input → per-path bounded input" correction and the partial
single-variant-unification residual. Independently adversarially reviewed. W4 `Enacted`; RFC stays Accepted.

**Status (2026-07-03):** **W0, W1, W2, W3½, and the W3+W5 pair landed** (maintainer-approved past the
checkpoint). W3 = frozen-core iterative destruction (`Node`/`Datum`/`CoreValue` iterative
`Drop`/`Clone`/`PartialEq`/`Canon` via the DN-56 §6 within-freeze channel — bit-identical observable,
mutation-witnessed, `forbid(unsafe)` intact; plus `doc::ir::Node`). W5 = the L1-eval CEK `Vec<Frame>`
machine (O(1) host stack, cleanup-on-unwind), iterative `L1Value`, TCO under the no-pending-post-work
precondition (release/assert never skipped — witness tests pass), and `DEFAULT_DEPTH` 64→4096. Honest
deviations flagged for maintainer: zero-alloc-in-Drop not achievable under safe-Rust + no-new-field
(empty-start `Vec` worklist used — OOM-unwind edge remains); `Value`/`Repr` deferred to a coordinated
W3b (deep values are construction-gated, thus unbuildable); pre-existing `content_hash` O(depth²) for
deep binders. **Remaining: W4** (L0 interp — closes the `myc run` SIGABRT and turns the §5.1 error-parity
gate green), then **W6**.

**Status (2026-07-03):** **W0, W1, W2, and W3½ landed — the full pre-checkpoint set.** W3½ = a
behavior-preserving extraction: the AOT `Vec<Frame>` env-machine (`mycelium-mlir`) now charges the
shared `mycelium_workstack::RecursionBudget` (both frame-push sites, `DepthGuard` per frame) and grows
via `ensure_sufficient_stack`, with `BudgetError::DepthExceeded` mapped to the unchanged
`EvalError::DepthLimit` at the same threshold — the recursion + three-way differentials stay green with
zero expected-value edits (oracle unmoved). The AOT `Frame` size pin (W2 residual) is resolved. Honest
flag: the AOT still charges per-frame (App and Match continuations), identical to pre-W3½ — the
per-frame-vs-source-call reconciliation is W5's. W3½ `Enacted` (AOT scope). **Next: the maintainer
checkpoint before W3 (frozen-core iterative destruction), W4 (L0-interp work-stack), W5 (L1-eval CEK +
TCO + eval raise) — all three touch the frozen trusted base / swap the reference machine.**

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

**DoD closure (2026-07-03, W7 — append-only).** Each item's disposition is recorded in the §7 W7 status
block above. Items **literally met** by W7: `--unbounded` guards + test; no input SIGABRTs any `mir-passes`
pass; the `Value` construction-gate + Box-owned spine tripwire. Items **honestly re-scoped by amendment**
(checked basis named): no-alloc-in-Drop → "no abort except genuine OOM during deep unwind" (#5, `Declared`);
the full §4.5 class → *constructible* types with `Value`/`Repr` deferred to W3b, plus the bare-`Repr`
overclaim correction (#6, census `Empirical`); the process arena → "every allocation-proportional path
reachable from untrusted input" with an audited exempt set (#8, audit `Empirical`); the AOT metric ruled a
precision follow-on under the §5.1 family-parity contract (#3). Tracked non-DoD follow-ons: `content_hash`
O(depth²) (#7), the ~21 coarse-worker sites (#9, LSP hot path first), the `L1Value` spine-tripwire twin, and
the `count_occurrences` O(N²) work-step bound. The RFC stays `Accepted`; **`Enacted` is claimable when W7
lands on `main`.**

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

- **2026-07-06 — AOT parity for the M-994 perf wins: (b) landed via §6, (a) is a tracked decision
  (append-only, no status move; M-995/M-996).** Applying the M-994 evaluator wins to the AOT
  (`mycelium-mlir`, the perf path — **not** in the DN-56 freeze scope; the frozen `mycelium_core`
  types are untouched). **(b) — landed (M-995):** the AOT env-machine (`aot.rs::run_core`) had the
  same per-reference O(nodes) deep-copy (`AotVal::Core` clone on every lookup + match-bind, measured
  ~n³ p=2.98); fixed with an **AOT-local `AotVal::Data(Rc<AotDatum>)`** cons cell (the interpreter's
  `Arc`-on-`Data.fields` couldn't port — those fields live in the frozen `Datum`). O(1) reference +
  destructure; measured p 2.98→~2.3–2.5, 13×/21×/35× at n=100/200/400; iterative `to_core`/`Drop`
  (deep-spine safe). **§6 bar met:** the AOT `-p mycelium-mlir` suite is green and results are
  **byte-identical** (`ObservationalEquiv` + M-210 `Validated{Exact}`, the §5.1 family-parity
  contract); `aot_frame_size` pin holds; zero new `unsafe`. Less clean than the interpreter's n³→n²
  (the env-machine's HashMap-env clone is the residual — a future env-rep change could recover a
  cleaner n²). **(a) — NOT landed, decision-gated (M-996):** the env-machine has no TCO, but adding it
  is a behavior-changing new feature (a divergent tail loop shifts `DepthLimit`→`FuelExhausted`,
  breaking the pinned graceful-error test) — a maintainer decision under the §5.1 family-parity + the
  tracked "AOT per-frame metric precision" residual, not a §6 behavior-preserving landing (the native
  LLVM path already has O(1) TCO for the canonical tail-`Fix`). RFC-0041 stays **Enacted**.
  (M-995/M-996; E18-1-adjacent; VR-5/G2.)
- **2026-07-06 — §6 within-freeze hardening: O(1) `L1Value::Data` clone via `Arc` sharing (M-994 fix
  (b); M-987 ~n³→~n²; append-only, no status move).** A clean use of the §6 behavior-preserving
  channel: `Data`'s `fields` wrapped in `Arc<Vec<L1Value>>` so a clone is a refcount bump instead of
  an O(nodes) spine rebuild (the confirmed root of M-987's ~n³ — `eval_path` deep-copies on every var
  reference). Sound because `Data` is immutable+acyclic by construction; `Arc` (not `Rc`) because
  `L1Value` must be `Send+Sync` behind the evaluator's `Mutex`. Derived `Clone` (the ~60-LOC iterative
  clone deleted); `Drop` reworked to stay iterative for a uniquely-owned deep spine (`Arc::get_mut`)
  while shared subtrees drop O(1) — the 200k-deep `guard_hole_census` no-SIGABRT invariant holds.
  **§6 bar met (I1–I3):** the **M-210 differential (32/32) + error-parity are green and UNCHANGED**
  (no fingerprint/error edited); identical values/errors/order. Measured (debug, `Empirical`): fitted
  complexity p 2.96→1.86–2.01 (~n³→~n²), 14×/30×/64× speedup at n=100/200/400. With fix (a) (depth,
  §4.6 amendment below) + (b) (cost) both landed, the DN-26 §9 flag-2 interpreted-first Stage-6 gate is
  practical. **M-987 → done; M-994 → done.** RFC-0041 stays **Enacted**. (M-994 fix (b); E18-1-adjacent;
  VR-5/G2.)
- **2026-07-06 — §4.6 amendment: widen the TCO precondition through tail-transparent frames
  (M-994 fix (a); maintainer-approved via the §6 channel; append-only, no status move).** §4.6's TCO
  precondition ("no pending post-work") was **too narrow**: it treated a `Frame::MatchPop`/`Frame::LetPop`
  above the caller's `InvokePost` as pending work, so a tail call made **inside a `match` arm or `let`
  body** was never elided — and since every terminating loop needs a `match`, **no in-language loop
  could exceed the 4096 depth budget** (this was M-986, pinned in `compiler_stage3.rs`). The amendment
  refines the precondition to look **through** any run of `MatchPop`/`LetPop` — which are
  *observationally transparent to the value* (they only restore scope) — so a tail call under them is
  still in tail position (its result **is** the enclosing function's result). Implementation
  (`eval.rs::enter_call`, ~47 LOC): **peek** past the transparent frames (non-tail path byte-for-byte
  unchanged), then on commit **drain** them executing each one's scope cleanup eagerly (incl. the M-904
  `LetPop` Substrate scope-exit release for a let-bound handle that does not escape into `argv` — never
  a silent leak). This **completes** §4.6's ratified TCO intent (Decides item 5); it is not new kernel
  surface. **Landing channel + sign-off (§6 / DN-56 freeze):** the change is value-preserving for
  *terminating* programs (the M-210 differential + the `compiler_stage*` fingerprint parity are all
  **unchanged** — verified), but it **shifts the runs-vs-refuses frontier** (programs that returned
  `DepthExceeded` now return a value), so it is not purely §Posture-I2-behavior-preserving and required
  an **explicit maintainer sign-off** (2026-07-06, the M-994 decision) rather than the routine §6
  channel alone. Justification recorded (checked, VR-5): recursion+`match` programs run **only** on the
  L1-eval path (outside the L0-elaboration fragment) and the L0 reference interpreter has **no TCO**, so
  there is **no L0 oracle for these deep loops to diverge from** — the I3 cross-path-parity exposure is
  nil; `depth_metric_parity` (static §4.0 metric + the *non-tail* witness) stayed green. Correctness
  guard proven: a **non-tail** self-call (`sum(n)=add_u(n, sum(n-1))`) still refuses
  `DepthExceeded{4096}` (no over-elision). The two M-986 pins are flipped to assert the closed behavior
  (a 10,000-iteration `match` loop now returns `Ok`; the 150-item nodule that refused at `depth=512` now
  passes). **M-986 → done.** The complementary **M-987** (~n³ L1-eval cost) stays open — demonstrated
  live: an 800-item parse now runs *depth*-wise but is ~n³ *slow* — and is addressed by M-994 fix (b)
  (`Rc`-share `L1Value::Data`), which lands through this §6 behavior-preserving channel proper. RFC-0041
  stays **Enacted** (this is an append-only §4.6 refinement). (M-994 fix (a); E18-1-adjacent; VR-5/G2.)
- **2026-07-05 — `Accepted → Enacted` (maintainer-approved W7 promotion; effective with this landing
  on `main`).** The maintainer approved the full promotion (2026-07-05, session review); the reconciled
  W0–W7 wave moves `dev → integration → main` by this landing — the §9 claimability condition is met
  the moment this text reaches `main`. Enactment basis (checked, VR-5): every §9 DoD line is literally
  met or honestly re-scoped by the recorded append-only §7/§9 amendments — flagship `myc run` SIGABRT
  closed (refuses `DepthLimit{4096}`), §5.1 error-parity green un-ignored, one deterministic budget on
  every path, iterative destruction landed via the §6 within-freeze channel, `--unbounded` implemented +
  corpus-refused, `#![forbid(unsafe_code)]` intact (sole unsafe = the exact-pinned, forbid-line-guarded
  upstream `stacker`/`psm` — no first-party dep-tree unsafe audit exists; the geiger baseline is a
  disclosed placeholder, tracked in the Status follow-ons). Non-DoD follow-ons stay tracked (see
  Status). M-978/M-979 → done; DN-84 → Resolved. Append-only (house rule #3).
- **2026-07-03 — W6 landed; RFC-0041 Phase-4 (W0–W6) COMPLETE (M-979).** Final wave, assess-then-act:
  the §4.7 "convert-or-document" fork resolved to **document the wide-tuple asymmetry** — `usefulness`/
  `decision` recurse on tuple/ctor arity, a 4095-field product type false-refuses at the floor, **but** it
  refuses **never-silently** (`DepthExceeded{4096}` on the deep-stack worker, not a SIGABRT — the W1 guard
  meets the DoD), so it is a *precision* residual on a pathological input, not a safety one; §7 gates the
  conversion on "if profiling demands" (it does not); a byte-identical rewrite of the trusted branching
  Maranget passes is high-risk/zero-benefit (KISS/YAGNI/KC-3). Docs+tests only (differential + conformance
  byte-identical); the conversion seam + boundary witness tests are recorded. Maintainer may overrule →
  convert if 4095-arity is adversarially realistic. **All seven waves landed; the never-silent + shared-
  budget + grow-on-demand recursion-safety contract holds end-to-end; M-740 self-hosting unblocks.** RFC
  stays Accepted; open determinations flagged. (VR-5/G2.)
- **2026-07-03 — W4 landed (L0 interp budgeted work-stack; the flagship SIGABRT closed; M-979).**
  `mycelium-interp` (`step`/`subst`/`node_to_core_value`/`guarantee_of_value`/`select_arm`) charges the
  shared `RecursionBudget` (L0 stays a substitution machine, §4.1; `subst` fallible; `eval_core` on the
  grown stack); **`EvalError::DepthLimit` constructed** via `From<BudgetError>`, so a deep value refuses
  `DepthLimit{4096}` instead of SIGABRT-ing `myc run` (RR-29 §0.1 closed). `parallel::is_pure` iterative.
  **§5.1 error-parity gate GREEN** — rewritten per the §5.1 W4 amendment (the "single statically-deep
  input" premise was empirically unachievable: parser-cap==floor, AOT data-immunity, L0 `O(N²)`), asserting
  the real per-path bounded-input parity at the shared floor; partial single-variant residual noted.
  Independently adversarially reviewed. Only W6 remains. W4 `Enacted`; RFC stays Accepted. (VR-5/G2.)
- **2026-07-03 — W3+W5 landed (frozen-core iterative destruction + L1-eval CEK; M-979).** Maintainer-
  approved past the checkpoint. W3: `mycelium-core` `Node`/`Datum`/`CoreValue` iterative
  `Drop`/`Clone`/`PartialEq`/`Canon` via the DN-56 §6 within-freeze channel (bit-identical vs a recursive
  oracle, mutation-witnessed at 100k, M-210 green, `forbid(unsafe)` intact, Box-owned invariant confirmed,
  E0509 blast radius = 3 sites total); `doc::ir::Node` iterative Drop. W5: the L1-eval 7-fn SCC → CEK
  `Vec<Frame>` machine (O(1) host stack, error-path cleanup), iterative `L1Value`, TCO under the
  no-pending-post-work precondition (both mandatory witnesses pass), EXPLAIN ring buffer, `DEFAULT_DEPTH`
  64→4096. Deviations flagged (VR-5): zero-alloc-Drop not achievable in safe Rust w/o a new field (empty
  `Vec` worklist — OOM-unwind edge); `Value`/`Repr` → coordinated W3b (deep values construction-gated);
  `content_hash` O(depth²) for deep binders (pre-existing). Independently adversarially reviewed for
  memory safety before landing. §5.1 error-parity still `#[ignore]` (needs W4). W3+W5 `Enacted`; RFC stays
  Accepted. (VR-5/G2.)
- **2026-07-03 — W3½ landed (AOT env-machine extraction; M-979).** Behavior-preserving: the AOT
  `Vec<Frame>` env-machine (`mycelium-mlir` `aot.rs`) charges the shared
  `mycelium_workstack::RecursionBudget` at both frame-push sites (`DepthGuard` per frame) and grows via
  `ensure_sufficient_stack`, replacing its ad-hoc `stack.len() >= max_depth` ceiling.
  `BudgetError::DepthExceeded` maps to the unchanged `EvalError::DepthLimit` at the same threshold —
  `recursion_differential.rs` + the three-way `differential.rs` stay green with zero expected-value
  edits (the oracle is unmoved). AOT `Frame` size pin added (W2 residual resolved). Honest flag: the AOT
  charges per-frame (App + Match continuations), identical to pre-W3½ — W5 reconciles the metric. Last
  pre-checkpoint wave; W3/W4/W5 are the maintainer-gated frozen-core/reference-machine waves. W3½
  `Enacted`; RFC stays Accepted. (VR-5/G2.)
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
