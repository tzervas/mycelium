# W7 — Process-Arena Coverage Audit (RFC-0041 §4.2/§9)

| Field | Value |
|---|---|
| **Produced by** | RFC-0041 W7 Enacted-closure wave, leaf item #8 (process-arena coverage) |
| **Scope** | `crates/mycelium-lsp`, `crates/mycelium-fmt`, `crates/mycelium-doc`, `crates/mycelium-mir-passes`, `crates/mycelium-transpile`, `crates/mycelium-l1` (frontend census) |
| **Kind** | Audit artifact (not an ADR/RFC/DN — no status lattice) — the checked basis for the W7 orchestrator's append-only §9 DoD amendment |
| **Date** | 2026-07-03 |
| **Guarantee tag** | `Empirical` throughout — an audit is a point-in-time read of the source; it can rot. Basis lines cite the specific reachability/allocation reasoning, never asserted without one. |

## 1. Why this audit exists

RFC-0041 §9 (Definition of Done) line 4 states:

> One deterministic budget (depth-on-metric + memory ceiling + **process arena** + frontend
> work-step) governs **every path**.

`ProcessArena` (`crates/mycelium-workstack/src/lib.rs`) landed in **W1** as a type: a shared atomic
byte counter with `reserve`/release RAII semantics, refusing `BudgetError::OutOfBudget` when the
*concurrent sum* of reservations would exceed a per-process ceiling (§4.2 — the multiply-under-
concurrency vector: LSP re-analyses, parallel eval workers, spore batch). **As of this audit
(2026-07-03, `dev` @ `a794084`), zero consumers outside `mycelium-workstack`'s own tests called
`ProcessArena::reserve` / `current_process_bytes` anywhere in the workspace** — confirmed by
`grep -rn "ProcessArena\|current_process_bytes" crates/*/src` returning only the defining crate. So
the §9 DoD line above was **not yet met** for any frontend pass prior to this wave. This audit
enumerates the frontend allocation-proportional passes, classifies each by untrusted-reachability,
and records what this wave charged vs. left explicitly exempt/deferred.

**Ratified disposition (per the W7 kickoff brief):** audit-then-charge-the-untrusted-reachable,
amend the remainder — **not** force fallibility through every infallible signature (disproportionate
API churn for passes unreachable from untrusted input).

## 2. Audit table

| Pass (crate::fn) | Charges `ProcessArena`? (before) | Untrusted-reachable? | Allocation-proportional? | Disposition (this wave) |
|---|---|---|---|---|
| `mycelium-lsp::project::llm_canonical` (public entry, dispatches `render_node`) | No | **Yes** — an LSP editor buffer is untrusted input by definition (RFC-0041 §5); this is the crate's outermost public render entry | Yes — builds a `String` proportional to node count via nested `format!` | **CHARGED** (§3.1) |
| `mycelium-fmt::format_source_styled_cfg` (→ `format_source`/`format_source_readable`/`format_source_styled`) | No | Yes (conservative) — `.myc` source text of unstated provenance (vendored/spore-resolved dependency, or a CI run over an untrusted PR diff); RFC-0041 §4.7 already classified "the `mycelium-fmt` render family" as a guard hole alongside lsp/transpile/doc/mir-passes on this basis | Yes — builds the canonical `String` proportional to source size | **CHARGED** (§3.2) |
| `mycelium-fmt::flatten_source` (→ `render_flat`) | No | Same as above | Yes | **CHARGED** (§3.2) |
| `mycelium-doc::ir::Node::walk` | No | **No** — `mycelium-doc` is the M-363 documentation **build pipeline**: its `DocModel`/`Node` tree is projected from the local repo's own corpus (RFCs/ADRs/notes/specs) and code + M-359 header metadata, driven by a maintainer/CI-run `myc-doc` binary over the trusted local tree. No consumer wires it to an LSP buffer, spore-remote fetch, or batch compile of user `.myc` source. | N/A (walk itself is a `()`-returning visitor with no owned growth; any allocation lives in the caller's closure) | **EXEMPT** — basis: unreachable from untrusted input (Empirical, this audit date; re-check if `mycelium-doc` ever grows a consumer over user-authored/spore-remote `.myc`) |
| `mycelium-transpile::emit::{emit_expr, emit_block_as_expr, emit_enum, emit_struct, emit_fn, emit_trait, emit_impl, map_pattern}` | No (charges only the depth guard) | **No** — this crate transpiles **trusted first-party Rust source** (the Mycelium kernel's own `.rs` files, via `syn`) into `.myc` for the self-hosting/porting effort (M-740); it does not process user-authored or spore-remote `.myc`/Rust at all. | Yes (already `Result`-returning, so a byte charge would be low-friction if reachability changes) | **EXEMPT** — basis: unreachable from untrusted input (Empirical, this audit date; re-check if `mycelium-transpile` ever runs over third-party/untrusted Rust) |
| `mycelium-mir-passes::emit::count_occurrences` (public) | No | Yes — `mir-passes` runs during compilation of any `.myc` source (batch compile / spore path) | **No** — returns `usize`; a pure counting traversal with no heap growth proportional to input (unlike the renderers above, its own stack frames are O(1) extra state each, already covered by the existing depth guard / `ensure_sufficient_stack`) | **EXEMPT** — basis: not allocation-proportional (Empirical: no `Vec`/`String` growth in the traversal). Its real residual cost is **CPU** (the flagged `O(N²)` re-walk, §4.7), which needs a `RecursionBudget::charge_steps` **work-step** bound, not a byte-arena charge — and per the function's own doc comment, adding that requires an infallible→fallible signature change that "would ripple into `is_fully_borrowable`/`is_sole_owned_move` and the `emit_elided`/`emit_reuse` path" (already flagged pre-existing, not newly discovered here). Out of this item's scope (bytes, not work-steps); tracked as the pre-existing residual. |
| `mycelium-mir-passes::emit::{emit_owned, emit_elided, emit_reuse}` | No (charge depth only) | Yes — same as `count_occurrences` above | Yes — each builds an `RcNode` tree proportional to input size | **DEFERRED** — a concurrent sibling W7 leaf (`claude/leaf/W7-mir-passes-guards`, item #2) is actively editing this exact file (`crates/mycelium-mir-passes/src/emit.rs`) to close the `emit_elided`/`emit_reuse` depth-guard hole. Charging arena bytes here now risks a direct edit collision with in-flight sibling work on the same lines; tracked as a **follow-up** once that branch lands (basis: avoid duplicated/conflicting concurrent edits, not a reachability or allocation-shape judgment — this is the one exemption grounded in swarm scheduling, not the pass's own properties). |
| `mycelium-l1` checker family (`usefulness::useful`, `decision::compile_rows`, `grade`, `checkty`) | Partial — `checkty.rs` already charges `RecursionBudget::charge_steps` (work-steps, not the process arena) | Yes — the L1 checker runs on every compiled `.myc` source | Yes — proportional to arity/column counts (the W6 "wide-tuple" data-spine) | **DEFERRED** — this subtree was the direct subject of the just-landed **W6 (data-spine iteration)** wave (`e091754`/`a794084`); extending process-arena coverage here is a distinct follow-up so as not to destabilize freshly-landed, adversarially-reviewed work in the same commit window. Tracked as a follow-up, not exempted on reachability grounds (it IS untrusted-reachable and allocation-proportional). |
| `mycelium-interp` (`is_pure`/`plan_parallel`, `parallel.rs`), `mycelium-mlir` (`write_canon`, `aot.rs`) | No | Yes | Yes | **OUT OF SCOPE (FLAG, escalation)** — these are the **trusted base** (interpreter / AOT machine). Per this item's explicit stop condition: charging them would ripple a fallible arena-reservation surface into `mycelium-interp`/`mycelium-mlir`'s already-established error paths — a bigger decision than a single leaf item, requiring maintainer/Opus-level review of the trusted-base error-surface change. Not attempted here. |

## 3. What this wave charged

### 3.1 `mycelium-lsp::project::llm_canonical`

**Signature change (contained):** `llm_canonical` was `pub fn llm_canonical(node: &Node) -> String`
(infallible) before this wave. It is now `pub fn llm_canonical(node: &Node) -> Result<String,
BudgetError>`. This is a **local, contained** change:

- The function is `pub` but **not** re-exported at the crate root (`mycelium_lsp::lib.rs` does not
  `pub use` it); grep confirms every call site is inside `mycelium-lsp` itself (its own tests plus
  `llm_canonical_parser.rs`'s roundtrip tests).
- It does **not** ripple into the trusted base (`mycelium-interp`/`mycelium-mlir`) — nothing there
  depends on this LSP-facing projection.
- All in-crate call sites were updated to handle the `Result` (`.expect(...)` in test code, since
  every existing fixture is small enough to fit the default ceiling).

**Mechanism:** a new `pub(crate) fn llm_canonical_with_arena(node, arena: &ProcessArena)` does the
real work — a pre-flight `node_count` pass (an `Exact` deterministic structural count, run on the
same guarded worker stack as the render itself, since an unguarded count on a pathological `Node`
would overflow just as readily as the render) sizes a reservation
(`node_count × ESTIMATED_BYTES_PER_NODE`, `Declared`, not measured), reserved via
`arena.reserve(estimate)?` before calling `render_node`. `llm_canonical` is the thin public wrapper
that supplies the crate's declared default ceiling (`PROCESS_ARENA_CEILING_BYTES = 256 MiB`,
`Declared`).

**Tests** (`crates/mycelium-lsp/src/tests/project.rs`):
- `large_synthetic_input_trips_out_of_budget` — a 5,000-deep `Let` chain against a 1-byte
  `ProcessArena` (via `llm_canonical_with_arena`) refuses `BudgetError::OutOfBudget { kind: Bytes,
  limit: 1, .. }` never-silently.
- `normal_sized_input_passes_unchanged` — the same 5,000-deep chain through the real
  `llm_canonical` (256 MiB default) renders successfully and completely.
- The pre-existing `guard_hole_census.rs::render_node_deep_let_chain` (20,000-deep, ~1.3 MB
  estimated) was updated to `.expect(...)` the now-fallible call — it still passes: the arena wiring
  does not perturb the crate's pre-existing depth-safety guarantee.

### 3.2 `mycelium-fmt` render family

Both public render-family entry points (`format_source_styled_cfg`, reached by `format_source` /
`format_source_readable` / `format_source_styled`; and `flatten_source`) were **already**
`Result<Formatted, FmtError>` — no signature change was needed here, only a new reservation +
a new `FmtError` variant:

- **`FmtError::OutOfBudget(BudgetError)`** — additive (`FmtError` is already `#[non_exhaustive]`),
  exit code **5** (contract §5, a new code — the three pre-existing refusals are 2/3/4), `source()`
  chains the underlying `BudgetError` for `EXPLAIN`-ability.
- Each of `format_source_styled_cfg`/`flatten_source` now delegates to a `pub(crate)
  ..._with_arena` sibling that reserves `src.len() × RENDER_BYTES_PER_SRC_BYTE` (`Declared`
  multiplier = 4, accounting for readable-layout re-indentation/wrapping) against a `ProcessArena`
  before invoking the render family (`render_body_with_comments` / `render_flat`), immediately after
  the source has parsed successfully (so the reservation is sized off real input, not a worst-case
  guess pre-parse). The public functions supply the crate's declared default
  (`PROCESS_ARENA_CEILING_BYTES = 256 MiB`).

**Tests** (`crates/mycelium-fmt/src/tests/arena_coverage.rs`):
- `format_source_trips_out_of_budget_on_tiny_ceiling` / `flatten_source_trips_out_of_budget_on_tiny_ceiling`
  — a 1-byte `ProcessArena` (via the `_with_arena` entry points) refuses `OutOfBudget` never-silently
  for even the crate's smallest realistic fixture.
- `format_source_normal_input_passes_unchanged` / `flatten_source_normal_input_passes_unchanged` /
  `format_source_moderate_input_passes_unchanged` (a 500-fn synthetic source) — all pass unchanged
  under the real default ceiling.
- `out_of_budget_has_its_own_exit_code` — exit code 5, `source()` chains the `BudgetError`.

## 4. What this wave left exempt / deferred (never-silent about the gap)

Per §2's table: `mycelium-doc::ir::Node::walk` and the `mycelium-transpile::emit` family are
**exempt** (basis: not reachable from untrusted input — both operate over the trusted local repo /
first-party Rust source, never user `.myc` or spore-remote content). `mir-passes::count_occurrences`
is **exempt** on a different basis (not allocation-proportional; its real residual is a pre-existing,
already-flagged CPU/work-step concern, not a memory one). `mir-passes::{emit_owned, emit_elided,
emit_reuse}` and the `mycelium-l1` checker family are **deferred** (not exempt) — both are genuinely
untrusted-reachable and allocation-proportional, but charging them now either collides with an
in-flight sibling branch's edits to the same file, or risks destabilizing the just-landed W6 wave.
`mycelium-interp`/`mycelium-mlir` (the trusted base) are explicitly **out of scope** for this item —
escalated, not attempted, per the item's own stop condition.

**Net honesty statement (VR-5):** RFC-0041 §9's "one deterministic budget … governs every path" is
now **closer to true** (2 of the 4 named example passes charged, the LSP editor-buffer and the fmt
CLI's untrusted-`.myc`-text surface — arguably the two highest-exposure entry points), but **not
fully met** — the mir-passes RC-emission passes and the L1 checker's arity spine remain uncharged
against the process arena, tracked above as explicit follow-ups rather than silently left off this
audit.

## Meta — changelog

- **2026-07-03 — Audit created (RFC-0041 W7, leaf item #8).** Enumerated the frontend
  allocation-proportional passes; charged `mycelium-lsp::project::llm_canonical` and the
  `mycelium-fmt` render family against `ProcessArena`; exempted `mycelium-doc`/`mycelium-transpile`
  (unreachable from untrusted input) and `mir-passes::count_occurrences` (not
  allocation-proportional); deferred `mir-passes::{emit_owned,emit_elided,emit_reuse}` (sibling-branch
  collision) and the `mycelium-l1` checker family (post-W6 stabilization); flagged
  `mycelium-interp`/`mycelium-mlir` as an explicit escalation (trusted-base signature ripple, out of
  this item's scope).
