# Research Record 29 — Recursion Depth & Stack Safety (DN-84 / M-979 Phase-1)

| Field | Value |
|---|---|
| **Record** | 29 |
| **Feeds** | M-979 (design D, solve-now) · M-978 · DN-84 |
| **Method** | research → plan → adversarial review → implement (maintainer mandate, 2026-07-03) |
| **Posture** | Evidence base for the RFC. Every claim carries a confidence tag `checked` (read/verified) / `plausible` (inferred) / `asserted` (recalled). Orchestrator-verified findings are marked **[V]**. Decides nothing — surfaces findings + plan-decisions. |
| **Baseline** | `origin/main` @ post-`#1136` (MSRV 1.96.1; parser deep-stacked). Three research agents ran against a stale pre-ADR-041 worktree — their "parse.rs unprotected" claims are **superseded** and excluded here. |
| **Date** | 2026-07-03 |

> Produced by a 14-agent research workflow (8 module mappers + 5 question agents + completeness critic;
> 3 re-run after schema failures) totalling ~2.9M tokens, then orchestrator confirm/correct against source.

---

## 0. Executive summary — the five decisive facts

1. **The `myc run` engine is the *least* protected, not the most.** `mycelium-interp` (the L0 reference
   interpreter `myc run` executes, `mycelium-cli/src/lib.rs:371`) has **no recursion-depth budget at
   all** — `EvalError::DepthLimit` is **defined but never constructed** (`lib.rs:182` def; only
   `fuel.checked_sub` at `lib.rs:561` fires), `step()` takes no depth param, and the crate does **not**
   depend on `mycelium-stack` (no deep worker thread). **[V] checked** — orchestrator-verified. A
   crafted deep-but-fuel-cheap value SIGABRTs `myc run` today. **This is entirely absent from DN-84
   §3's inventory** and is the single most important finding.

2. **The drop-bomb is real and lands in the *frozen* kernel.** `Node` (`mycelium-core/node.rs:37`),
   `Value` (via `Payload::Seq`, `value.rs:231`), `Datum`/`CoreValue` (`datum.rs:28/89`), and L1
   `L1Value` (`eval.rs:49`) all use **derived recursive `Drop`**; **zero manual iterative `Drop`**
   exists in `mycelium-core` (**[V]** `grep 'impl Drop'` → none). A deep chain overflows *on
   destruction* — even on the never-silent **refusal** path (the refused deep AST is still dropped),
   and critically the returned value is dropped on the **caller's ~2 MB stack**, outside any
   `with_deep_stack`. Grow-on-demand does **not** cover Drop. `checked`.

3. **Iteration is only "free" in depth for the tail/`for` shape on the trampoline/AOT — and the stdlib
   doesn't use it.** There is **no TCO** in the L1 evaluator (a 10k tail-recursive `.myc` fn burns ~10k
   depth); the L0 interpreter's *driver loop* is O(1)-stack for tail recursion but its `step` is
   structurally recursive so **non-tail** recursion (the actual `lib/std` idiom — `map`/`filter`/
   `reverse` all build a `Cons` around the recursive call) overflows the plain thread stack at large N.
   `checked`. **Consequence:** a `.myc`-authored compiler pass (M-740) written in the current stdlib
   idiom **cannot** process a 10k-item worklist without either burning the depth budget or risking a
   host overflow — *unless* the evaluators gain an explicit work-stack / TCO first. This is the
   load-bearing fact for solving D **before** the port.

4. **The AOT env-machine is a ready, differential-validated reference implementation of design D.**
   `mycelium-mlir` already runs recursion as a `Vec<Frame>` **explicit heap control stack**
   (`aot.rs:18`, `trampoline.rs`), tail-recursion **loopified** (no `musttail`), with an explicit heap
   depth budget and a real `EvalError::DepthLimit`. `checked`. It was accepted **post-freeze** as
   differential-equal — precedent that an explicit-stack machine can be admitted without breaking the
   oracle. **Port its shape; don't invent one.**

5. **The prior art says: convert the *evaluators*, guard the *frontend*.** rustc uses stacker-guarded
   recursion everywhere and **explicitly declined** a wholesale explicit-stack rewrite of parser/
   checker/lowering (even removed most guards in 2024 as a perf win, PR #134153); CEK/OCaml-bytecode/
   CPython-3.11 all put their explicit stacks in the **evaluator**. serde_json's default-128 limit +
   opt-in `unbounded_depth` + `serde_stacker` is DN-84's template almost verbatim. `checked`.

**Net:** the decided direction (D solved now; B as supporting infra) is well-supported — but the map
found the problem is **broader than DN-84 §3 recorded** (the trusted L0 interpreter, the frozen-core
Drop, and several unmapped crates), and it surfaced **four genuine plan-decisions** (§6) that are the
maintainer's, plus a **hard ordering constraint** (grow-on-demand must precede any budget raise).

---

## 1. Verified recursion-site inventory (by pass)

Guard status legend: **G** = charges an explicit depth budget · **H** = *hole* (recurses unguarded) ·
**DS** = runs on the 256 MB `with_deep_stack` worker · **caller-stack** = runs on the ordinary ~2 MB
stack.

### 1.1 Parser cluster (`parse.rs`) — healthiest, post-ADR-041 `checked`
- `parse_expr`/`parse_unary`/`parse_type_ref`/`parse_pattern`/`parse_consume_expr` — **G** (shared
  `MAX_EXPR_DEPTH=256`, paired `enter_depth`/`leave_depth`), **DS**. Lexer/`token`/`error`/`nodule`
  are **fully iterative** (**[V]-adjacent**, agent-checked).
- **List/tuple literals parse FLAT** (`comma_separated`, a `while eat(Comma)` loop) — a `[e1..e100000]`
  literal is **O(1) parser depth**; the `Cons` chain is built **downstream** in `checkty::check_list`.
  `checked`. This localizes the data-spine problem to the checker/elaborator/evaluators, **not** the
  parser.
- Holes: `parse_impl_item` (`parse.rs:1175`) skips the `parse_type_ref` charge for one outer level
  (benign, 1 level); `collect_name_uses` walks a parsed `TypeRef` unguarded (safe only because
  parser-bounded). `checked`.
- Why the ADR-041 near-miss hit *here*: the `parse_type_ref → …→ parse_type_args_opt → comma_separated
  → parse_type_ref` cycle is **5–6 stacked frames per nesting level** (vs 1 for `parse_unary`), so a
  frame-size growth from the toolchain multiplied ~5–6× on exactly that path. `checked`.

### 1.2 Type checker (`checkty.rs`, 377 KB) `checked`
- `Cx::check` (`checkty.rs:3640`) is the **sole** charging site (`MAX_CHECK_DEPTH=4096`, `enter()`
  3648, `DepthGuard` releases), **DS**. Bidirectional — returns `(Ty, Expr)`, so results flow **up** →
  conversion needs **defunctionalized continuations**, not a bare worklist. `very-hard`.
- **The data-spine origin:** `check_list` (`checkty.rs:5568`) **builds the full N-deep `Cons` chain**
  (iterative build loop, 5585) then **recursively checks it** (5591), charging **~1 budget level per
  element** against *any* structurally cons-list-shaped ADT (`cons_list_ctors`, 3565) — the exact
  §5 data-vs-control conflation. A ~4096-element list literal is refused; a same-length `Seq` is
  accepted. **Highest-leverage single fix:** check elements in a flat loop, never build/recurse the
  chain.
- Holes: `usefulness::useful` + `decision::compile_rows` (from `check_match`) recurse **unguarded**;
  type/pattern traversals (`subst_ty`/`unify`/`resolve_pattern`) unguarded (safe only via the parser's
  256). `decision`/`usefulness` add a **distinct spine class: tuple/ctor *arity* → recursion *depth***
  (a wide concrete tuple-pattern), surface-reachable and NOT bounded by the 256 nesting cap.

### 1.3 Elaborator (`elab.rs`, 118 KB) `checked`
- `Elab::expr`/`expr_inner` — **G** (`MAX_ELAB_DEPTH=4096`, sole charge at 1116), **DS**; every
  sub-elaborator re-enters through it. Bottom-up `Result<Node>` → two-phase expand/assemble worklist.
  The Cons/App spine costs **~3 host frames per element**, so budget 4096 ≈ **12k live frames** already.
- Holes: `lower_tree` (Maranget, recurses directly at 1667/1684/1690 — **unbudgeted**), Tarjan
  `strongconnect` (`elab.rs:768`, **unbudgeted**, driven by call-graph size), type resolvers
  (`field_spec`/`ty_to_repr`). `checked`.
- Compounding: `mono::monomorphize` re-walks the whole AST before elab; elab re-invokes
  `infer_type`/`resolve_ty`/`decision::compile` mid-descent → several deep re-walks of one chain
  (an O(N²) smell, also flagged in `mono::rewrite_app` and `mir-passes::count_occurrences`).

### 1.4 L1 evaluator (`eval.rs`, 80 KB) `checked`
- Seven-function mutual-recursion SCC (`eval`/`eval_app`/`eval_match`/`eval_for`/`eval_wild`/
  `eval_hypha_forage`/`invoke`), all **DS**, `eval` charges `DEFAULT_DEPTH=64` **per AST node**.
- **`eval_for` is already the iterative-spine model** (a `loop` over the Cons spine, O(1) depth/element
  — the in-tree realization of §5.2). The env (`scope`) is a single shared `&mut Vec`, already
  stack-shaped → favorable for a CEK conversion, but scope push/pop/`release_if_abandoned`/guarantee
  asserts interleave with post-child work → continuations must reify them.
- **CRITICAL value-side holes (run on the CALLER stack, outside `with_deep_stack`):** `L1Value` derived
  `Clone`/`Drop`/`Debug` over the value spine; `value_contains_substrate_id` (`eval.rs:155`, unguarded,
  reachable from Substrate scope-exit on a deep result); `to_core` (`eval.rs:117`). A deep returned
  `Cons` value SIGABRTs on drop/clone on the caller stack — the **value-side twin of the ADR-041
  near-miss**. `checked`.
- `DEFAULT_DEPTH=64` is **64× below** the sibling 4096 — a ~32-element Vec literal trips it.

### 1.5 L0 reference interpreter (`mycelium-interp`) — **the trusted base, and the biggest hole** **[V]**
- Substitution small-step machine. Outer `eval_core` is a **fuel**-bounded `loop` (O(1) stack); but
  `step`/`subst`/`node_to_core_value`/`guarantee_of_value` **all recurse to term/value depth with NO
  depth budget**, on the **caller stack** (no `mycelium-stack` dep). `DepthLimit` never constructed.
  The `lib.rs:181` doc "O(1)-stack" claim is true of the step-*count* driver, **false** for `step`'s
  structural recursion. **[V] checked.**
- Same drop/clone/Canon-hash exposure on `Node`/`Value`/`Datum`.

### 1.6 mono / satellites `checked`
- `mono.rs` walks guarded by `MAX_WALK_DEPTH` (M-866) but with unguarded sub-walks flagged; `rewrite_app`
  a probable O(N²). `totality::walk_expr` is the **one fully-guarded deep tree-walk** (the model).
  `ambient` guarded (`MAX_AMBIENT_DEPTH=4096`, **DS**, per-body reset). `grade`/`decision`/`usefulness`/
  `affine`/`fuse` carry **no measured ceiling**.

### 1.7 Crates the critic found UNMAPPED (must fold into the plan) `checked`
- **`mycelium-mir-passes`** (AOT RC/ownership path): `emit_owned`↔`emit_args`↔`emit_alt` + `eval(&RcNode)`
  mutually recurse over L0 `Node` with **zero** guard / zero deep-stack; reachable via
  `mycelium-mlir/rc_plan.rs:126`; `count_occurrences` re-walks per binder (O(N²)).
- **`mycelium-transpile`** (the Rust→Myc tool): `emit_expr`↔`map_type`↔`map_pattern` over the `syn` AST,
  unguarded (lower severity — dev tool; `syn::parse_file` itself recurses).
- **`mycelium-core::lower::write_canon`** (`lower.rs:212`) + **`mycelium-lsp` render** — canonical/LLM
  rendering of a post-elaboration (can-exceed-256) `Node`, unguarded, caller-stack.
- **`mycelium-doc`** IR walker (low severity).

---

## 2. The three execution paths & what iteration costs (the decisive question) `checked`

| Shape ↓ / Path → | L1 evaluator (`eval`) | L0 interp (`myc run`) | AOT (mlir) |
|---|---|---|---|
| surface `for` | **O(1) depth** (`eval_for` loop) | Fix-fold (recursion) | loop, O(1) native |
| tail recursion | **O(n) depth** (no TCO) → budget/DS | **O(1)** host stack (driver loop) | loop, O(1) native |
| **non-tail recursion** (stdlib idiom) | **O(n) depth** → trips 64 | **O(n) host stack, unguarded → SIGABRT** | heap trampoline, O(1) native, budgeted |

**Only AOT is O(1)-native for both shapes today.** The stdlib (`iter.myc`/`collections.myc`) iterates by
**non-tail recursion** — the worst cell for both interpreters. So the M-740 `.myc` port inherits O(n)
depth / overflow risk unless the evaluators get an explicit work-stack and/or TCO **first**. This is the
concrete evidence for the maintainer's "solve D now, before the port" call.

---

## 3. External prior art → convert-vs-guard (the shape of the answer) `checked`

- **Guard the frontend, convert the evaluators.** rustc guards (stacker) and declined wholesale
  iterative rewrites of parser/checker/lowering; CEK/OCaml-bytecode/CPython-3.11 put explicit stacks in
  the evaluator. → Convert **L1-eval + L0-interp** to explicit heap work-stacks (design D); keep
  **parser/checker/elaborator** on guarded recursion + grow-on-demand (design B), converting only the
  worst path (`parse_type_ref` family / `check`'s data-spine) if a raise demands it.
- **serde_json is the template**: default deterministic limit (128) + opt-in `unbounded_depth` +
  `serde_stacker` growth → DN-84's 4096-default + `--unbounded` + `stacker::maybe_grow`, near-verbatim.
- **TCO caution (transparency):** CPython/JVM refused TCO for **debuggability** (frames vanish from
  traces). If Mycelium adopts tail-position looping, elided frames must stay **inspectable/EXPLAIN-able**
  (a counted marker), not silently dropped — else TCO fights the no-black-boxes rule. `checked`.
- **`stacker`/`psm` is a *silent no-op on unsupported platforms*** (wasm32/others) → it would silently
  degrade to the fixed stack and could SIGABRT before the deterministic budget fires — a direct **G2
  (never-silent) hazard** the adapter must detect and surface. Reinforces: self-hosting must **not**
  port the host-stack layer, only the portable budget. `checked`.
- **Flat/arena-index AST** (rust-analyzer/Zig/Carbon) dissolves *both* deep-Drop and deep-traversal by
  construction — but is a large retrofit; the **boot10 `.myc` rewrite is the natural place to choose the
  flat shape**, near-term the iterative-Drop + guards buy the same safety far cheaper. `checked`.

---

## 4. Security register (secure-by-design mandate) `checked`

- **Untrusted-input entry points** into the recursive passes: `myc run`/CLI, **`mycelium-lsp`** (parses
  editor buffers), **spore resolve/fetch** (BLAKE3 = *integrity*, **not safety** — a hostile package
  serves deep `.myc`), `fmt`/`lint`/`doc` on arbitrary files, fuzz harnesses. The **L0-interp hole
  (§0.1) is remotely reachable** via a hostile spore. `checked`.
- **Design-D converts stack-DoS → memory-DoS**: at budget 50k × ~10.9 KB checker frame ≈ **545 MB per
  in-flight pass**, capped only by the budget. The RFC **must** specify an explicit **byte** ceiling for
  the work-stack (frame-structs, not cloned subtrees) yielding the same never-silent error. `checked`.
- **Hard ordering constraint:** the fixed 256 MB stack tops out at **~24,600 checker frames**; raising
  the budget toward "tens of thousands" (§9.5) **before** wiring grow-on-demand **re-introduces the
  SIGABRT**. Wire `maybe_grow` first; add a startup assertion tying max-budget to reserved stack when
  grow-on-demand is off. `checked`.
- **Supply chain:** `stacker`/`psm` are **not** yet in `Cargo.lock`; `deny.toml` would admit them but
  no gate scans unsafe, and `scripts/checks/unsafe-per-use.sh` audit-A **excludes `mycelium-stack`** —
  so ADR-014 containment rests on an unprotected `#![forbid(unsafe_code)]` line. Add `mycelium-stack`
  (+`mycelium-l1`) to audit-A; pin exact versions; THIRD-PARTY + `about.toml`. `checked`.
- **Durability gaps:** `cargo-mutants` scope **excludes `mycelium-l1`** (the depth guards are **not**
  mutation-tested — a remove-guard mutant survives); the 3 fuzz targets **never synthesize deep
  nesting**. Make "add `mycelium-l1` to mutants + depth-structured fuzz + remove-guard witness tests" a
  **Definition-of-Done precondition**, not an afterthought. `checked`.
- **`--unbounded` footgun:** must be **CLI-flag-only** (never manifest/env/LSP-config), never-silent
  stderr banner, corpus-excluded, refused in CI. (Today there is *no* config surface to any depth
  constant — the footgun is prospective; **[V]-adjacent**.)
- **Positive baselines to keep:** serde_json's default-128 is the one closed sub-class (no
  `disable_recursion_limit` anywhere — keep it); post-ADR-041 parser/eval/ambient/checker are guarded+DS.

---

## 5. The deterministic-vs-dynamic-budget contradiction (must resolve before the RFC) `checked`

DN-84 §4.2 mandates a **deterministic** budget. But the **shipped, Enacted AOT env-machine already
derives its depth ceiling *dynamically* from measured memory headroom**, clamped `[10_000, 2_000_000]`
(`mycelium-mlir/budget.rs`, DN-05/M-349). So **the same program can already diverge accept/reject by
machine on the AOT leg** of the M-210 three-way differential (L1-eval ≡ L0-interp ≡ AOT) — before DN-84
changes anything. DN-84 never cites DN-05. **The RFC must reconcile** a deterministic 4096-family L1/L0
budget with the AOT machine's memory-derived ceiling so the differential cannot diverge — the biggest
single architectural question.

---

## 6. Plan-decisions surfaced for the maintainer (the RFC will frame; these are yours)

1. **Deterministic vs AOT-dynamic budget (§5).** Options: (a) make all three paths share one
   deterministic budget (change the AOT machine to a fixed default — touches Enacted DN-05); (b) define
   the differential only up to `min(budgets)` and document the divergence above it; (c) a deterministic
   *floor* all paths honor + a dynamic *headroom* above it that the corpus never exercises. **Recommend
   (c)** — preserves determinism where it's observable, keeps AOT's memory-awareness.
2. **Drop-fix freeze disposition.** The iterative `Drop` for `Node`/`Value`/`Datum` lands in **frozen**
   `mycelium-core`. Is a purely-additive, semantics-preserving `Drop` (identity/`PartialEq` untouched) a
   DN-39 promotion review, a freeze amendment, or a core-2.0.0 event? **Recommend:** a DN-39 review
   admitting it as behavior-preserving hardening — *and* note the freeze currently protects a semantics
   that is **already unsound** for deep input (SIGABRT ≠ never-silent), so the fix *restores* the
   guarantee the freeze assumes.
3. **TCO / TRMC scope.** Is interpreter tail-call/constant-stack parity **in scope** for this work
   (DN-36 6(g) flags it as unbuilt; constant-stack is AOT-only today), or explicitly deferred? Bears
   directly on whether the `.myc` port's non-tail stdlib idiom is viable. **Recommend:** in scope for the
   *evaluators* (it's most of the win), with elided frames kept EXPLAIN-able.
4. **Convert-scope ranking.** Confirm the prior-art-backed split: **convert** L1-eval + L0-interp
   (mirror the AOT `Vec<Frame>` machine) + add iterative `Drop`; **guard+grow** the frontend passes,
   converting only `check`'s data-spine (`check_list`) and the `parse_type_ref` family if a raise
   demands. **Recommend:** yes.

---

## 7. Corrections owed to DN-84 §3 (append-only, before the RFC) `checked`

DN-84 §3's "current state" table is **incomplete in a security-relevant way** and must be corrected
append-only (VR-5/G2): add **`mycelium-interp` (no budget at all — the `myc run` engine)**, the
**recursive-`Drop` paths** on `Node`/`Value`/`Datum`/`L1Value`, **`write_canon`**, **`mycelium-mir-passes`**,
**`mycelium-transpile`**, and the **`grade`/`decision`/`usefulness`** unmeasured recursions. The table's
`evaluator | eval.rs::DEFAULT_DEPTH=64` row is the **L1** evaluator; it must not be read as covering the
L0 trusted base. (Applied in the same change that lands this record.)

## Meta — changelog

- **2026-07-03 — Record created (M-979 Phase-1 research).** 14-agent workflow + orchestrator
  confirm/correct. Five decisive facts (§0), verified recursion-site inventory (§1), iteration-cost
  matrix (§2), prior-art convert-vs-guard (§3), security register (§4), the deterministic-vs-AOT-dynamic
  contradiction (§5), four maintainer plan-decisions (§6), and the DN-84 §3 corrections owed (§7).
  Feeds the M-979 RFC (Phase-2). `checked`/`plausible`/`asserted` tagged throughout; **[V]** =
  orchestrator-verified against source.
