# Kickoff `hof` — Full / Dynamic Higher-Order Functions (R3 / M-704)

> Stowed kickoff, UID **`hof`**. Read `CLAUDE.md`, `.claude/kickoffs/README.md`, and
> **`.claude/kickoffs/_doc-maintenance.md`** (anti-drift) first.

## Metadata
| Field | Value |
|---|---|
| **UID** | hof |
| **Head/working branch** | `claude/head/hof-closures` (off `dev`) |
| **Status** | ready (2nd post-grammar wave: R4 → **R3** → DN-53 → DN-54) |
| **Swarm mode** | serial-on-L1 (inline; the heaviest of the semantic waves, ~930 LoC) |
| **Depends on** | RFC-0037 grammar (Enacted — the `lambda` keyword + `Expr::Lambda` parse node already land) |

## Scope
Close the dynamic-HOF residual: **closures (environment capture), partial application, dynamic
fn-flow** — today `lambda` parses to a never-silent `Residual` (RFC-0037 D5 / M-704). Implement via
**Reynolds defunctionalization + closure-struct lowering** (KC-3-safe — **no new L0 node**; closures
become tagged structs + a generated `apply` dispatcher, extending the landed named-fn defunctionalizer).

**Issue:** M-704. Unblocks stdlib combinators (capturing `map`/`filter`/`fold`) and the self-hosting
capstone (E13-1/E18-1, T4/T9).

## Grounding (doc_refs)
- `corpus:RFC-0024` — HOF via static defunctionalization; §5 residuals (the closures/partial-app/
  dynamic-fn-flow gap this closes). The named-fn case is already landed (M-685–688).
- `src:crates/mycelium-l1/src/mono.rs` — the defunctionalizer (`rewrite`/`emit_fn`); where dynamic HOF
  emits `Residual` today.
- `src:crates/mycelium-l1/src/checkty.rs` (lambda typing → `Ty::Fn`) · `src:crates/mycelium-l1/src/
  elab.rs` (the `Expr::Lambda` Residual to replace) · `src:crates/mycelium-l1/src/ast.rs` (`Expr::Lambda`).

## Approach (serial-on-L1, inline)
checkty.rs (infer closure capture set + type → `Ty::Fn`) → elab.rs (emit the synthetic closure struct +
capture binding) → **mono.rs** (the only wave touching it: `rewrite_closure_expr` near `emit_fn` —
Reynolds defunctionalization, fn-tag sum, generated `apply`) → parser already has `lambda` (extend it
for type/const-param lambdas if needed, currently refused). Three-way differential per closure shape.

## Definition of Done
- [ ] `lambda(...) => ...` with environment capture, partial application, and dynamic fn-flow
  **elaborate + evaluate** — the `Expr::Lambda` `Residual` is gone (G2); each shape has a three-way
  differential test (`Empirical`). **No new L0 kernel node** (KC-3 — verify the node budget unchanged).
- [ ] A capturing stdlib combinator (e.g. `map` with a closure) runs three-way as the consuming proof.
- [ ] `just check` green; honest tags (VR-5).
- [ ] **Doc maintenance (anti-drift):** `issues.yaml` M-704 → done; **RFC-0024** §5 residual → resolved,
  and RFC-0024 `Accepted → Enacted` (full HOF landed); the RFC-0037 `lambda`/`Expr::Lambda` "pending
  M-704" notes (token.rs/ast.rs/checkty/elab docstrings + DN-57/RFC-0037) updated to "implemented";
  `.claude/memory/language-execution.md` lexicon note; `mycelium.ebnf` lambda production finalized;
  `CHANGELOG.md` entry; `docs/api-index/` regenerated if the public API changed.

## Landing
`/wave-land` → `main` after green + self-review + curated squash; backprop down. Sequence next: **`obj`**.
