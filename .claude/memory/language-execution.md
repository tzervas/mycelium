# Language & Execution — Memory File

**Status: Empirical/Declared** — source + RFC/ADR are ground truth; this is an orientation aid.
Not normative. Cite from here, verify in source.

---

## What it is

The L1 concrete surface language (`.myc` files), the trusted reference interpreter, and the AOT
performance path (MLIR→LLVM). Source ground truth is `docs/spec/grammar/mycelium.ebnf` (NORMATIVE
oracle). The layer cake (RFC-0006 §3): L0 (frozen, trusted) → L1 (kernel calculus) → L2/L3
(surface; not yet ratified). Only L0 is the trusted base; everything above is elaboration.

---

## Where it lives

| Crate | Role | Key files |
|---|---|---|
| `mycelium-l1` | Parser, typechecker, evaluator, elaborator | `src/{lexer,parse,ast,checkty,totality,elab,eval,nodule,ambient,usefulness}.rs` |
| `mycelium-interp` | Reference interpreter (L0, trusted base) | `src/{lib,prims,swap,budget,supervise}.rs` |
| `mycelium-mlir` | AOT path: env-machine, LLVM IR, dialect, inject | `src/{aot,llvm,dialect,inject,jit,budget,runtime,simd,specialize}.rs` |
| `mycelium-build` | Stable-component gate + build certificates | `src/{lib,cache,target}.rs` |

Key specs: `docs/spec/grammar/mycelium.ebnf` · `docs/rfcs/RFC-0004-Execution-Model-and-Stable-Component.md` ·
`docs/rfcs/RFC-0007-L1-Kernel-Calculus.md` · `docs/rfcs/RFC-0011-L0-Match-and-L1-in-Core-IR.md` ·
`docs/adr/ADR-009` (hybrid execution) · `docs/adr/ADR-016-Interpreted-Compiled-ABI.md` ·
`docs/adr/ADR-017-Hot-Inject-Recompiled-Definitions.md`.

---

## L1 concrete surface (`.myc` files)

A `.myc` file is a **nodule**: `nodule <path>` header followed by items (RFC-0006 §3; grammar in
`docs/spec/grammar/mycelium.ebnf`). The nodule **keyword** in a source file is the grammar opener;
the `// nodule:` / `// nodule` comment is the separate in-file **header marker** (DN-06 §6;
`mycelium-l1/src/nodule.rs`).

**Paradigm types** (ast.rs:17): `Binary{N}` (N-bit), `Ternary{N}` (N-trit, balanced `+0-` encoding),
`Dense{dim, scalar}` (dense embedding; scalar ∈ F16/BF16/F32/F64), `VSA{model, dim, sparsity}`
(hypervector). Also `Substrate{name}` (affine external resource, LR-8).

**Paradigm-less repr** `{N}` / `{N, scalar}` / `{model, dim, sparsity}` — the enclosing ambient
supplies the paradigm (RFC-0012 §4.2); an unresolved ambient or shape mismatch is an explicit
error, never a silent coercion (ast.rs:42–60).

**Items** (ast.rs:63): `use path`, `default paradigm P` (nodule-scope ambient), `type` (ADT), `trait`,
`fn`. `thaw fn` keeps one definition interpreted inside a matured scope (RFC-0017 §4.3; ast.rs:122).

**Guarantee-annotated types** `T @ g` where `g ∈ Exact | Proven | Empirical | Declared` (the
honesty lattice, ast.rs:228). The `guarantee: None` form means "supplied by checked context."

**Expressions** (ast.rs:241): `let / if / match / for / swap / with paradigm / wild / spore / app`.

**`swap` (never-silent, S1/WF2)**: `swap(value, to: TypeRef, policy: path)` — target AND policy
are always explicitly written; omitting `policy` is a parse error with a diagnostic (lib.rs:72–78;
ebnf:130).

**`match`**: covers data constructors and `Binary`/`Ternary` literal patterns AND nested patterns
(M-320). Coverage checked by the **Maranget usefulness algorithm** (`usefulness.rs`) — both
exhaustiveness and redundancy are checked (W7), never assumed.

**`for`**: bounded fold sugar over a linearly-recursive data value; Total by construction
(RFC-0007 §4.8; ast.rs:271).

**`wild { body }`**: the denied-by-default unsafe block (LR-9/S6).

**`matured fn` at item position** is a parse error with a teaching diagnostic pointing at the
`// @matured: true` header form and `thaw fn` (lib.rs:117–133; RFC-0017 §4.1).

**Literals** (ast.rs:355): `Bin("…")` = `0b…` verbatim; `Trit("…")` = `<+0->` verbatim;
`Int(i64)` = bare decimal (unresolved until ambient in scope); `AmbientInt` (produced by the
resolution pass only; never reaches elaboration). `Literal` is `#[non_exhaustive]` (ast.rs:354).

---

## Parser + `myc-check` exit codes

Parser: `mycelium_l1::parse(src) -> Result<Nodule, ParseError>`. Never panics, never silently
accepts. Ternary literal disambiguation (`<` = trit literal vs type-arg): lexer lookahead
(lib.rs:26). All malformed inputs are explicit `ParseError` with source position (S5/G2).

`myc-check` exit codes (check/src/lib.rs:88–99):
- **0** — clean (no findings)
- **2** — any parse error
- **3** — any type/check error
- **5** — project resolution failure (no `.myc` sources, unreadable file)

---

## Typechecker + totality

`check_nodule` (checkty.rs) — bidirectional typechecker (RFC-0007 §4.4). Every refusal is an explicit
`CheckError {site, message}` (checkty.rs:43); nothing is ever guessed (G2/VR-5). **Generics now check
and monomorphize** (M-656/M-657): type-params infer by `unify`; **width-generics** (M-753/DN-42) make
representation width a const-generic param — `Ty::Binary(Width)`/`Ty::Ternary(Width)` with `Width::{Lit,
Var}`, bound **same-paradigm-only** (cross-paradigm + width-mismatch are explicit refusals, never a swap
— S1), pinned per call site by `mono` (undetermined → never-silent `Residual`, never a default).
**M-753 width-var pass-through:** `unify` binds a width var against another width var (the type-var
pass-through analog), so **recursive / delegated** width-generics (e.g. self-hosted `map_get<N,V>`,
`le<N>`→`cmp<N>`) check, with `mono` resolving the concrete width from the substituted scope. (M-718 is
the *stdlib surface* that consumes this feature, not the feature itself.) The recursive-HOF / RFC-0024
defunctionalization gap (recursion through a function *parameter*) is **distinct** and is now **CLOSED**
(M-715, rsm S3): a HOF parameter re-passed at a recursive call site (`map(rest, f)`) is threaded through
`mono::resolve_fn_args` as the **same** static specialization, so the self-hosted `lib/std/iter.myc`
combinators (`map`/`filter`/`foldl`/`any`/`all`/`find`) run three-way. Still deferred (M-704): closures,
multi-arg arrows, partial application, dynamically-flowing fn values. `spore` and `wild` blocks are
checker-validated per their own rules.

`Totality` (totality.rs): `Total | Partial`. Classification is **sound, not complete** (Foetus-style
structural descent; totality.rs:1–23) — a wrong verdict can mis-gate `matured` promotion, but
**never changes what a program computes** (semantics stay with the fuel-guarded evaluator). Mutual
recursion is handled via SCC + position-assignment search (MAX_ASSIGNMENTS = 4096).

`matured` gate (RFC-0017 + RFC-0004 §4): declared at **scope** (nodule/phylum header
`// @matured: true`; or `mycelium-proj.toml`). Every reachable non-`thaw` definition must pass
`Total` + AOT-eligibility. `thaw fn` exempts one definition.

---

## Core IR (L0)

Ten node kinds (RFC-0007 §3; interp/src/lib.rs:1–23):

```text
L0 (frozen): Const | Var | Let | Op | Swap
L1 (r4):     Lam   | App | Construct | Match | Fix
```

Plus `FixGroup` for mutual recursion (RFC-0001 r5; interp/src/lib.rs:480).

`Construct` is fully saturated (W6). `Match` is flat (single-level; nested patterns compiled to
flat by the Maranget elaborator). `Fix` gives general recursion; non-productive → explicit
`FuelExhausted`, never a hang (RFC-0007 §4.5; CakeML clock). `FixGroup` handles mutual recursion.

---

## Reference interpreter (trusted base)

`mycelium-interp` — the **meaning** of a program. AOT is differential-tested against it, never
the other way (NFR-7; interp/src/lib.rs:1–7; ADR-009).

Call-by-value small-step operational semantics (E-Let/E-Op/E-Swap/E-Con/E-Match/E-Lam/E-App-Beta/
E-Fix rules; interp/src/lib.rs:15–98). The interpreter is O(1)-host-stack by construction (no
unbounded C-stack growth; RFC-0004 §2).

`Interpreter::eval(&node) -> Result<Value, EvalError>` — repr values only.
`Interpreter::eval_core(&node) -> Result<CoreValue, EvalError>` — repr OR data (r3 fragment).
Default fuel: 1,000,000 steps. Override with `.with_fuel(n)`.

`EvalError` variants (all explicit; interp/src/lib.rs:131–218): `FreeVariable`, `UnknownPrim`,
`PrimType`, `ApproxCompositionUnsupported`, `UnsupportedSwap`, `Overflow`, `FuelExhausted`,
`DepthLimit`, `EffectBudget`, `Swap`, `Wf`, `NonExhaustiveMatch`, `DataMalformed`,
`GuaranteeMeetUnsupported`, `DataResult`, `ApplyNonFunction`, `FunctionResult`.

**Guarantee propagation** (RFC-0001 §4.7): result guarantee = meet of inputs ∧ op's intrinsic
strength. `Match` result guarantee met with scrutinee's (RFC-0011 §4.6); non-`Exact` data
scrutinee → explicit `GuaranteeMeetUnsupported` (r3 boundary; never a fabricated bound).

---

## MLIR→LLVM AOT path (performance path)

`mycelium-mlir` (mlir/src/lib.rs:1–57). Four sub-paths:

1. **`dialect::emit`** — textual ternary-dialect rendering of lowered ANF; always available (no
   toolchain needed); per-stage-dumpable, no-opaque-pass anchor (RFC-0004 §6).
2. **`dialect::native`** (`mlir-dialect` feature, OFF by default; M-601) — real `arith`/`func`
   MLIR → `mlir-opt`/`mlir-translate` → LLVM IR → `clang` → native. Bit/trit element-wise
   fragment only; skips gracefully when libMLIR absent (ADR-019). Guarantee: `Empirical`.
3. **`aot::run` / env-machine** — big-step evaluator over lowered ANF (M-151/NFR-7). Two-path
   differential vs the reference interpreter. Stack-robust: trampoline over heap control stack
   with explicit `DepthLimit` ceiling (M-347; RFC-0004 §2 normative requirement).
4. **`llvm::compile_and_run`** — direct LLVM IR backend (M-301); bit subset only; emits textual
   LLVM IR, drives `llc`+`clang`, reads result back; `ToolchainMissing` on absent tools (never
   silent). Third compiled path in the three-way differential (M-302).

**`inject::Image`** (M-341; ADR-016/017) — in-process hot-inject: `ContentHash → entry` dispatch
table. Injection = load content-addressed unit + register new hash → entry; **never mutates a live
entry** (immutability dissolves atomicity hazard). Call ABI: `call(def: ContentHash, args:
[CoreValue]) -> Result<CoreValue, AbiError>`. Value ABI: RFC-0001 §4.8 self-describing wire form.
Recompile set = changed dependency-closure by hash reachability (`inject::recompile_closure`).

**Stable-component gate** (RFC-0004 §4; `mycelium-build`): AOT-eligible iff (1) content-addressed
and hash-frozen, (2) spec ratified, (3) verification obligations discharged. Promotion is a
**deliberate act** gated on automatic checks; everything else runs interpreted/JIT.

**`DepthBudget`** (mlir/budget.rs): derived from `MemAvailable`/`RLIMIT_AS` with conservative
static fallback; `EXPLAIN`-able basis; never a magic constant; never an abort.

---

## Key invariants (honesty)

- **Never-silent swap (S1/WF2):** `swap` target + policy always written; no rule may introduce a
  `Swap` silently (W8 at kernel boundary).
- **No partial artifact:** elaboration outside the evaluation-complete fragment → explicit `Residual`
  refusal, never a partial L0 term (lib.rs:13).
- **Totality gate is for promotion, not meaning:** `matured` gates AOT eligibility; it never changes
  what a Partial program computes.
- **The interpreter is the reference** (ADR-009): AOT agrees with interpreter (NFR-7); a divergence
  is a correctness loss, not an alternative interpretation.
- **`ApproxCompositionUnsupported`**: no defined ε-propagation rule for approximate inputs in the
  logical bit/trit ops (ADR-010/M-204); refused explicitly, never a silent degradation.
- **Parser never panics** (S5/G2): every malformed input is an explicit `ParseError` with position.

---

## Read more

- `crates/mycelium-l1/src/lib.rs` — module-level honesty summary + test cases
- `crates/mycelium-l1/src/ast.rs` — full AST types
- `crates/mycelium-interp/src/lib.rs` — small-step rules + all `EvalError` variants
- `crates/mycelium-mlir/src/lib.rs` — AOT sub-path overview
- `docs/spec/grammar/mycelium.ebnf` — NORMATIVE grammar oracle
- `docs/rfcs/RFC-0007-L1-Kernel-Calculus.md` — ten-node budget, typing, totality
- `docs/rfcs/RFC-0004-Execution-Model-and-Stable-Component.md` — trusted base, AOT, continuum
- `docs/rfcs/RFC-0011-L0-Match-and-L1-in-Core-IR.md` — Construct/Match/Lam/App/Fix in L0
- `docs/adr/ADR-016-Interpreted-Compiled-ABI.md` — call ABI + value ABI (wire form)
- `docs/adr/ADR-017-Hot-Inject-Recompiled-Definitions.md` — hot-inject mechanics

---

## Gotchas

- `matured fn` at item position is a **parse error** (RFC-0017; the modifier was retired). Use
  `// @matured: true` in the header or `thaw fn` to de-mature a single def.
- `phylum` and `colony` are **reserved but inactive** keywords — they lex as keywords (no silent
  identifier), but no production consumes them yet (lib.rs:87–95).
- `AmbientInt` literal is produced only by the ambient resolution pass; it must never appear in
  the typechecker input (the checker defends against a residual one — ast.rs:365).
- The `mlir-dialect` feature is **OFF by default**; you must run `just setup-mlir` and build with
  `--features mlir-dialect` to exercise `dialect::native`.
- `DepthLimit` is raised by the AOT env-machine only; the reference interpreter is O(1)-stack and
  never raises it.
- `NonExhaustiveMatch` is unreachable for checked programs (the checker proves coverage via
  Maranget), but is kept as the explicit never-silent kernel fallback (G2).
- Three-way differential (M-210): L1-eval ↔ elaborate→L0-interp ↔ AOT; use the M-210 shared
  checker to verify agreement — never assume the compiled path is correct by default.
