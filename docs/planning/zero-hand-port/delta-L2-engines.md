# Appendix — Layer-2 (Engines) Coverage Audit

> Supporting inventory for `docs/planning/zero-hand-port-delta-ledger.md` §3. Draft, 2026-07-10.
> Grounded evidence backing the ledger's engine-reality synthesis; not independently ratified.

Grounded in: `crates/mycelium-l1/src/{lexer,parse,ast,checkty,elab,eval,mono}.rs`,
`crates/mycelium-core/src/node.rs`, `crates/mycelium-interp/src/{lib,prims}.rs`,
`crates/mycelium-mlir/src/{aot,llvm}.rs`, `lib/compiler/*.myc`, `tools/github/issues.yaml`.

## Engine pipeline (as-built)

- lex = `mycelium-l1/src/lexer.rs` (plus token.rs)
- parse = `mycelium-l1/src/parse.rs` to `ast.rs` Expr/Item/Pattern/Literal
- check = `mycelium-l1/src/checkty.rs` (desugars object/inherent-impl/or-pattern/tuple/lambda)
- L1-eval = `mycelium-l1/src/eval.rs` (tree-walker directly over AST, the trusted surface leg)
- elab = `mycelium-l1/src/elab.rs` : AST to Core IR (`mycelium-core::Node`); refusals =
  `ElabError::Residual`
- L0-interp = `mycelium-interp` : reference interpreter over `Node`
  (Const/Var/Let/Op/Swap/Construct/Match/Lam/App/Fix/FixGroup)
- AOT-env = `mycelium-mlir::aot::run` env-machine over `Node`, **whole v0 calculus** (M-342,
  `Node::is_aot_lowerable` total)
- AOT-llvm = `mycelium-mlir::llvm.rs` native LLVM-IR, **bit/trit subset only**; data plus
  recursion = explicit `AotError::UnsupportedNode`
- mirror = `lib/compiler/*.myc` (DN-26 self-hosting frontend, M-741 ratifies)

AOT has TWO layers. The env-machine covers everything the L0-interp does. The **native LLVM
backend is the real AOT laggard**: bit/trit consts plus Op plus Swap plus straight-line Let only;
every Datum/Closure/Fix/FixGroup value is `UnsupportedNode` (VR-5), i.e. no
`Construct`/`Match`/`Lam`/`App`/`Fix` native codegen. (M-348/M-601 = provision libMLIR plus real
ternary-dialect lowering.)

## Coverage matrix

Legend: Y=handled, D=handled-by-desugar (checker/mono lowers before this stage), S=staged/Residual
(never-silent refusal, L1-eval only), N=not covered, sub=native-LLVM sublayer.

### Expr variants (ast.rs 693-882)

| construct | lex | parse | check | AOT-env | AOT-llvm | L1-eval | L0-interp | .myc-mirror | gap notes |
|-----------|-----|-------|-------|---------|----------|---------|-----------|-------------|-----------|
| Let       | Y | Y | Y | Y | Y(straight-line) | Y | Y | shape Y / logic S | native-llvm only straight-line Let |
| If        | Y | Y | Y | Y | D(to Match) | Y | Y | shape Y / logic S | |
| Match     | Y | Y | Y | Y | N (UnsupportedNode) | Y | Y | shape Y / logic S | native-llvm refuses data match |
| For       | Y | Y | Y | D(to fold/Fix) | N | Y | Y | shape Y / logic S | native-llvm refuses Fix |
| Swap      | Y | Y | Y | Y | Y(bit/trit) | Y | Y | shape Y / logic S | other swap kinds Unsupported in llvm |
| WithParadigm | Y | Y | Y | stripped by ambient resolution | -- | Y | -- | shape Y / logic S | compile-time only, no runtime node |
| Wild(FFI) | Y | Y | Y | S | S | Y | S | shape Y / logic S | no FFI host in v0, Residual |
| Spore     | Y | Y | Y | S(M-260) | S | S | S | shape Y / logic S | deferred E2-5/M-260 |
| Wrapping  | Y | Y | Y | S | S | Y | S | shape Y / logic S | L1-eval only; elab Residual (interp follow-on) |
| Consume   | Y | Y | Y | Y(passthrough) | N | Y | Y | shape Y / logic S | Substrate value model staged (Residual when substrate-typed) |
| Colony/Hypha | Y | Y | Y | Y(seq ref) | N | Y | Y | shape Y / logic S | dual-path: seq ref plus runtime executor |
| Lambda    | Y | Y | Y | D(mono defunc) | N | Y | Y(as Construct) | shape Y / logic S | multi-arg/partial = Residual (no tuple gate); raw Lambda never in elab |
| App       | Y | Y | Y | Y | N (UnsupportedNode) | Y | Y | shape Y / logic S | native-llvm refuses Closure/App |
| Fuse      | Y | Y | Y | D(to join+meta) | N | Y | Y | shape Y / logic S | |
| Reclaim   | Y | Y | Y | Y(Let-chain ref) | N | Y | Y | shape Y / logic S | real RT7 supervision = runtime driver |
| Path      | Y | Y | Y | Y | Y | Y | Y | shape Y / logic S | |
| Lit       | Y | Y | Y | Y | Y(bit/trit) | Y | Y | shape Y / logic S | |
| Ascribe   | Y | Y | Y | D(erased) | D | Y | Y | shape Y / logic S | |
| TupleLit  | Y | Y | Y(to Construct) | D | N | Y | Y | shape Y / logic S | raw TupleLit = Residual in elab; desugared first |

### Items (ast.rs 173-208)

| item | lex | parse | check | AOT-env | AOT-llvm | L1-eval | .myc-mirror | notes |
|------|-----|-------|-------|---------|----------|---------|-------------|-------|
| Use | Y | Y | Y | n/a | n/a | Y | Y (nodule.myc/parse.myc) | resolution-time |
| Default(paradigm) | Y | Y | Y(stripped) | -- | -- | Y | Y | ambient |
| Type/Ctor | Y | Y | Y | Y | subset | Y | shape Y / logic S | |
| Trait/Impl | Y | Y | Y | D(dict) | N | Y | shape Y / logic S | |
| Fn | Y | Y | Y | Y | subset | Y | shape Y / logic S | |
| Object | Y | Y | D(to Type+Impl+Fn) | via lowered | -- | Y | shape Y / logic S | DN-53 desugar |
| Lower/Derive | Y | Y | Y | D(expand RHS) | -- | Y | shape Y / logic S | DN-54; item-RHS = M-973 |
| InherentImpl | Y | Y | D(to Fn) | via lowered | -- | Y | shape Y / logic S | DN-03 desugar |

### Patterns / Literals

Wildcard/Lit/Ctor/Ident/Tuple/Or patterns: lex plus parse plus check Y (Tuple to Ctor, Or to
multi-arm desugar); run via Match. Literals Bin/Trit/Int/AmbientInt/List/Bytes/Str: full
lex/parse/check/eval; AmbientInt resolved plus rewritten by checker (never reaches elab);
AOT-llvm literal support = bit/trit plus i64 only.

## Findings

### Biggest laggard: the native LLVM AOT backend (`mycelium-mlir::llvm.rs`)

The AOT env-machine is total over the v0 calculus (M-342), so "AOT" is only gapped at the
**native-codegen** sublayer: it compiles Const/Var/straight-line Let/Op/Swap over bit/trit/i64 and
**explicitly refuses** every data plus recursion node (Construct/Match/Lam/App/Fix/FixGroup =
`UnsupportedNode`). This is the sanctioned VR-5 gap tracked by M-348 (libMLIR provision, UNBLOCKED
2026-06-20 on Linux) plus M-601 (real ternary-dialect lowering) plus M-373 (direct-LLVM data
extension).

### Counts (19 Expr-class constructs)

- Fully covered lex to parse to check to AOT-env to L1-eval to L0-interp: ~14 (Let, If, Match,
  For, Swap, Consume, Colony, Lambda, App, Fuse, Reclaim, Path, Lit, Ascribe, TupleLit, via
  desugar where noted).
- AOT-native (llvm.rs) gapped: ~13 of 19 (everything touching data/closure/recursion).
- Genuinely L0-staged (Residual, L1-eval only, not just native-llvm): Spore (M-260), Wrapping,
  Wild/FFI, WithParadigm (compile-time strip) = ~3-4.
- `.myc`-mirror: AST **shape** 100% mirrored (ast.myc has all 18 Expr plus all Item/Pattern/Literal
  variants; parse.myc mirrors the parser). Semantic **logic** (check/elab/eval/mono/fuse) is the
  mirror gap; Stage 5 `semcore.myc` is PARTIAL (pure leaves ported: usefulness/decision/affine/
  grade plus checkty type-algebra plus mono mangling; the entangled checkty/elab/eval/mono core NOT
  ported).

### Mirror progress (DN-26 / M-741)

Stages 1-4 landed and differential-green: token, lex, nodule, ast, parse, substrate, totality,
ambient. Stage 5 (semcore SCC) partial (increments 1-8+, M-1007..M-1013). Stage 6
(`just bootstrap`, M-742) open. So the mirror covers the whole FRONTEND (lex/parse) but the
checker/elaborator/interpreter core is only partially self-hosted, this is the mirror's laggard,
symmetric to native-LLVM on AOT.

### Visitor-DRY meta-gap: CONFIRMED (real L2 meta-gap)

Grep of a recent variant (`Expr::TupleLit`, `Expr::Reclaim`) shows each new Expr variant touches
**8 production walkers in mycelium-l1** (parse, ambient, checkty, elab, eval, grade, mono,
totality) plus fmt/lib.rs, plus **~5 mirror sites** (ast.myc, parse.myc, ambient.myc,
totality.myc, semcore.myc) and multiple test census files. The only shared abstraction is
`totality::walk_expr`, a **depth-budgeted callback HOF** reused for pure *collection* passes
(call-set, cycle, free-var) — NOT a semantic visitor: every pass that needs per-variant *handling*
still writes its own exhaustive `match`. There is **no** trait-based `ExprVisitor`/fold that
centralizes per-variant logic. Partial mitigations exist (Rust exhaustiveness catches missed arms
at compile time; `Literal` is `#[non_exhaustive]`; `mycelium-fmt` has a `guard_hole_census.rs` W0
net; lsp does NOT walk Expr so it is not an N-site). Net: adding one construct is still an ~13-site
edit, a standing tax on every future construct.

### Reconciliation to tracked issues

- Native AOT data/recursion gap: M-342 (env-machine done), M-348 (libMLIR, unblocked), M-601/M-602/
  M-603 (real dialect plus three-way diff), M-373 (direct-LLVM data ext). M-347/M-349 (recursion
  robustness/dynamic budget) DONE.
- Mirror: M-740 (scaffold), M-741 (ratify port canonical, open), M-742 (bootstrap gate, open),
  M-989/boot10/DN-26 (self-host frontend). Stage 5 increments M-1007..M-1013.
- Visitor-DRY meta-gap: **NOT ticketed** as a standalone item (guard-hole census is RFC-0041 §4.7
  W0/W1 scoped to fmt stack-safety, not an ast-walker DRY refactor) — highest-leverage un-ticketed
  L2 work, now filed as M-1041 (see the ledger and DN-109).

### Highest-leverage engine work

1. Native-LLVM data/recursion codegen (M-373/M-601), closes the single biggest per-construct AOT
   gap.
2. A DRY `ExprVisitor`/fold abstraction across the 8 mycelium-l1 walkers plus a mirror-parity
   generator, removes the ~13-site tax that slows EVERY future construct (the meta-gap, filed as
   M-1041).
3. Stage-5 semcore mirror (checkty/elab/eval/mono port, M-741), the mirror laggard.

### Un-groundable / flagged

- Exact per-op guarantee grades (Empirical vs Declared) per construct not re-verified here, taken
  from in-file doc headers.
- "3 copies of classify_expr" from the brief is NOT literally present: only ONE `classify_expr`
  reference found (a test in compiler_stage3_ast.rs). The real replicated-walker count is the ~8
  mycelium-l1 exhaustive matches above, the meta-gap is real but its shape is walker-replication,
  not three classify_expr copies specifically.
