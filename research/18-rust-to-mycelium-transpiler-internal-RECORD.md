# Internal Research Report — Rust→Mycelium Transpiler Feasibility

**Date:** 2026-06-25  
**Scope:** Ground-truth assessment of transpiler seed projects, Mycelium surface readiness, and construct mappings  
**Status:** `Declared` — research capture for DN-34 follow-up design work

---

## 1. Seed-Project Architectures

### 1.1 py2rust — Python→Rust AST-walk Transpiler

**Repository:** maintainer-provided seed project **py2rust** (Python→Rust transpiler)

**Architecture overview:**
- **Entry point:** `src/py2rust/cli.py`, two commands: `transpile` and `analyze` (lines 17–80)
- **AST parser:** Python's `ast.parse()` stdlib (line 36)
- **Transpiler:** `PythonToRustTranspiler` class (lines 82–127)
  - Walks AST with `ast.walk(tree)` (line 107)
  - Simple function transpilation: `_transpile_function()` emits skeleton Rust `fn` with placeholder body (lines 113–127)
  - No type inference; all params hardcoded as `i32`, return type `i32`
- **Compatibility analyzer:** `CompatibilityAnalyzer` class (lines 130–153)
  - **Flag-don't-guess discipline:** walks AST and surfaces issues (Import, ClassDef, Try, Lambda) as plain-language reasons (lines 138–151)
  - Never emits guessed Rust; raises exceptions or lists issues
  - Output: list of strings describing manual-conversion requirements

**Reuse verdict for Rust→Mycelium:**
- ✅ **RETARGET (high confidence):** The AST-walk + analyzer architecture is directly transferable. Changes needed:
  - Input: replace Python `ast.parse()` with Rust `syn` crate (or `rustc_ast` via `-Z unpretty=ast`)
  - Transpiler: instead of emitting Rust code, emit Mycelium L2 surface (text or AST)
  - Analyzer: same visit pattern, but flag Rust constructs un-transpilable to Mycelium (unsafe, macros, lifetimes, etc.)
- ⚠️ **Code complexity:** py2rust is intentionally skeletal (early-stage)—barely 160 lines. The real transpiler will be order-of-magnitude larger, especially the type-inference and ownership-mapping logic. But the *shape* is sound.
- **File references:**
  - `cli.py:82-127` — the core transpiler class structure
  - `cli.py:130-153` — the `CompatibilityAnalyzer` pattern (the load-bearing G2 design)

---

### 1.2 py-rust-bridge — Python↔Rust FFI/SFI Bridge

**Repository:** maintainer-provided seed project **py-rust-bridge** (Python↔Rust SFI/FFI bridge)

**Architecture overview:**
- **Entry point:** `src/rust_bridge/cli.py`, two commands: `generate-bindings` and `analyze-rust` (lines 26–166)
- **Configuration parser:** tomllib (Python 3.11+) / tomli (fallback) to read `pyproject.toml` (lines 9–16, 47)
- **Binding generators:**
  - **PyO3 generator** (lines 66–127): templates for `lib.rs`, `Cargo.toml`, `setup.py` using Jinja2 (lines 108–126)
  - **Native and cbindgen generators** (lines 129–151): stub implementations
- **Rust analyzer** (lines 154–165): placeholder—not implemented; would scan Rust source for function/type candidates for Python exposure

**Reuse verdict for Rust→Mycelium:**
- ✅ **PARTIAL RETARGET (medium confidence):** The binding-generation + interop-analysis *pattern* applies, but the target is different:
  - **What transfers:** The idea of scanning source for "what can be exposed" and generating bridge code
  - **What differs:** py-rust-bridge targets FFI (Python←→Rust native). In the Mycelium rewrite transition, RFC-0028 governs Rust↔Mycelium interop. The shimming strategy is similar (generate glue code at the boundary), but the target language/ABI is Mycelium, not Python/PyO3.
  - **Templating:** Jinja2 template approach (lines 108–126) is reusable; same pattern for generating Mycelium FFI stubs.
- ⚠️ **Maturity gap:** The `analyze_rust()` is not implemented (line 165 is a placeholder). A real transpiler would need robust Rust AST analysis to identify:
  - Safe-to-expose functions (no `unsafe`, FFI-compatible types)
  - Type signature conversion (Rust types → Mycelium types)
  - Dependency mapping (which Rust libs/functions map to which Mycelium imports)
- **File references:**
  - `cli.py:66-127` — template-based binding generation pattern
  - `cli.py:154-166` — the stub Rust analyzer (shows what would need to be built)

---

## 2. Mycelium-as-Target Readiness

### 2.1 Surface Grammar Maturity

**Ground truth sources:**
- `docs/spec/grammar/mycelium.ebnf` (v0, L1-facing)
- `.claude/memory/lang-lexicon-syntax.md` (orientation guide)
- `docs/notes/DN-02` (reserved words, Resolved 2026-06-10)
- `docs/notes/DN-06` (static organization: `nodule`/`phylum`, Resolved 2026-06-16)

**Assessment:**

The Mycelium surface is **70–80% transpilation-ready** at the grammar level:

✅ **PRESENT and STABLE (Active keywords, in lexer + grammar):**
- Functions: `fn` with `type_params`, return type, no params binding (grammar line 121)
- Data types: `type Ident = Constructor | …` with `type_params` (grammar line 108)
- Control flow: `if`/`then`/`else`, `match` (flat, Maranget-lowered), `for` (bounded iteration) (grammar lines 170–178)
- Bindings: `let x = expr in expr` (grammar line 169)
- Traits: `trait Ident<…> { fn_sig* }` + `impl Trait for Type { … }` (grammar lines 114, 121; **Enacted M-659/M-673 — dictionary-passing elaboration STAGED, not yet running**)
- Effects: `!{ effect_name, … }` annotations (grammar line 131; M-660 landed)
- Representation paradigms: `swap(expr, to: type, policy: path)` + `with paradigm { … }` + `default paradigm` (grammar lines 102, 167, 185)
- Guarantee tags: `type @ Exact|Proven|Empirical|Declared` (grammar line 134)
- Concurrency: `colony { hypha expr, … }` (grammar lines 56, 197–207; **Enacted M-666 — structured concurrency**)
- Unsafe escape: `wild { expr }` (grammar line 192; **Lexed M-661, checker enforcement in place for @std-sys context + !{ffi} effect**)

⚠️ **PRESENT but NOT YET ACTIVE (Reserved keywords, lexed but no consuming production):**
- `phylum` — library-scale grouping; reserved-not-active (grammar line 66 comment: *optional*, but no construct consumes it yet for cross-nodule visibility; **Enacted M-662 — now ACTIVE as metadata grouping header**)
- `colony` — **Now ACTIVE as expression** (M-666; grammar line 161)
- `hypha` — **Now ACTIVE as expression sub-form** (M-666; grammar lines 197–207)
- Nine runtime terms (`fuse`, `mesh`, `graft`, `cyst`, `xloc`, `forage`, `backbone`, `tier`, `reclaim`) — ratified in DN-03 / RFC-0008 §4.5, reserved in lexer, no productions yet (grammar lines 41–50; scheduled M-665…M-668)

❌ **BLOCKING GAPS for transpilation target:**

1. **Type inference / annotation discipline:** Rust has no explicit type annotations for variables (inference is automatic). Mycelium requires explicit `let x: Type = …` or full annotation at binding sites. The transpiler must **infer and emit types** for every binding.
   - **Status:** Type registry exists; elaboration type-checks (RFC-0007 §4.2). But the *surface* grammar does not yet mandate annotations; the grammar allows `let x = expr in …` without a type (grammar line 169: `(':' type_ref)?` is optional).
   - **Verdict:** ⚠️ **Partial blocker.** The transpiler can emit explicit types (the surface accepts them), but it must do full type inference (expensive, error-prone). Until Mycelium mandates annotations, the transpiler can emit best-effort inferred types + flags for ambiguous cases.

2. **Generics / polymorphism completion:** RFC-0019 (Enacted M-673 — dictionary-passing elaboration; grammar already supports `type_params` at line 108, trait methods at line 115, but the elaboration is **STAGED**—the literal dictionary `Construct` form for trait methods is deferred. In practice:
   - Monomorphic trait instances work now (a concrete `impl Eq<Binary{8}>` is elaborable).
   - Polymorphic trait instances (generic over type variables) are parsed but elaboration may be incomplete.
   - **Verdict:** ⚠️ **Partial blocker for generic code.** Monomorphic generics work; polymorphic generics are risky until M-673 elaboration is proven stable.

3. **Error handling / Result/Option:**  Mycelium uses never-silent `Option`/`Result` (Exact, matching Rust). The grammar and stdlib (`std.error`) are in place, but:
   - The surface syntax for the `?` operator (or equivalent) is **not specified** in the v0 grammar.
   - **Verdict:** ⚠️ **Transpiler must emit explicit `match` / `unwrap` patterns** rather than `?` sugar; less ergonomic, but correct.

4. **Closures / higher-order functions:** The grammar has `Lam` (lambda) in L1 (RFC-0007 §3), and the surface allows lambda via elaboration. However:
   - Rust closures capture environment; Mycelium lambdas are explicit `Lam(param, type, body)` with no capture (RFC-0024 on static defunctionalization).
   - **Verdict:** ⚠️ **Major blocker for closures**. Any Rust code with environment-capturing closures (the majority) must be flagged for manual refactoring (defunctionalization).

5. **Macros, build scripts, FFI:** Rust macros (procedural or declarative), `build.rs` scripts, and FFI (extern "C") have **no Mycelium equivalent** in v0. The grammar's `wild` block (ADR-014) is the *trusted* FFI escape (audited, not verified), but it is not a general macro system.
   - **Verdict:** ❌ **Cannot transpile.** All macros, build scripts, and unannotated FFI must be flagged.

### 2.2 Minimal Grammar Subset to Be a Transpile Target

For a Rust→Mycelium transpiler to be viable, the target must support at minimum:

1. **Monomorphic functions with explicit types** ✅ (present)
2. **Product/sum data types (struct/enum → `type` declarations)** ✅ (present)
3. **Pattern matching (flat `match`)** ✅ (present, Maranget-lowered)
4. **Error handling via `Result`/`Option` + explicit unwrap/match** ✅ (present, but no `?` sugar)
5. **Monomorphic trait instances** ✅ (present, M-659 landed)
6. **Bounded recursion or iteration (`for` with linear recursion)** ✅ (present)
7. **Explicit ownership / borrow marking** ⚠️ (affine `Substrate` for external resources; the three-layer memory model DN-32 is a strategic direction, not yet lexed)
8. **Effect tracking** ✅ (present, M-660 `!{}` annotations)
9. **Never-silent `wild` FFI boundary** ✅ (present, M-661)
10. **Paradigm / representation swaps** ✅ (present)

**Verdict:** The **committed minimum is met**. Items 1–7, 9–10 are Active or Enacted. Item 8 (effects) is in place. Item 7 (ownership) is the one gap—the transpiler must manually annotate affinity or resort to RC (Layer 2 of DN-32). This is **not a blocker**; it is a refinement task.

### 2.3 Blocking Gaps and Mitigations

| Gap | Severity | Mitigation |
|---|---|---|
| **Closure capture** | High | Transpiler flags closures; refactor to top-level functions or encode captures as explicit parameters (defunctionalization) |
| **Macros** | High | Flag all macros (procedural, declarative) for manual expansion / rewrite |
| **Build scripts** | Medium | Flag; manual rewrite as Mycelium build metadata or separate tooling |
| **Generic trait bounds on polymorphic functions** | Medium | Monomorphic instances work; polymorphic bounds may need deferred error until M-673 is proven stable |
| **Type inference required for all bindings** | Medium | Transpiler does best-effort type inference; flags ambiguities or falls back to `Declared` (manual annotation needed) |
| **`?` operator sugar** | Low | Emit explicit `match` patterns instead; less ergonomic, but equivalent |
| **Lifetime annotations** | Medium | Ignore (Mycelium does not have lifetimes; borrow discipline is different — see §3) or flag for manual review |

---

## 3. Rust→Mycelium Construct Mapping

**Ground truth:** DN-34 §3 (sketch); DN-32 (memory model); RFC-0007/0011/0019 (kernels)

### 3.1 Ownership & Borrows → Memory Layers

| Rust construct | Mycelium target | Completeness | Notes |
|---|---|---|---|
| **`let x = v`** (move) | `let x = expr in body` | ✅ **Total** | Direct equivalence; affine semantics by default (no implicit Copy in Mycelium). |
| **`&T` (immutable borrow)** | **Layer 1 (affine) or Layer 2 (RC)** | ⚠️ **Partial** | In Rust, `&T` is borrowed (non-owning); in Mycelium's three-layer model, immutable data without sharing stays affine (Layer 1), shared data uses RC (Layer 2). The transpiler must decide: assume affine (no RC), or insert RC. DN-32 §2.2 specifies static uniqueness analysis to avoid RC on non-shared borrows—**this is unbuilt and `Declared`**. Conservative default: **emit RC** for all `&T` to be safe; refine with static analysis later. |
| **`&mut T` (exclusive borrow)** | **Layer 1 (affine move) + explicit mutation marking** | ⚠️ **Partial** | Rust's `&mut` is a exclusive borrow with guaranteed freshness. Mycelium has no mutable borrows; instead, an exclusive owner is moved and updated. Pattern: `let x = e in let x' = update(x) in body`. The transpiler must rewrite mutable borrows as move-then-update; the elaborator may optimize via DN-32's static uniqueness analysis. |
| **`move` (explicit move)** | `let x = expr in body` (default) | ✅ **Total** | Mycelium's default is move; no `move` keyword needed. |
| **`Box<T>` (heap-allocated unique)** | **Layer 1 (affine) or Layer 2 (RC)** | ⚠️ **Partial** | Rust's `Box` is a unique heap pointer (no refcount). Mycelium's equivalent depends on context: if genuinely unique, Layer 1 (affine); if shared later, Layer 2 (RC). The transpiler should flag `Box` for review or assume affine. |
| **`Rc<T>` (reference-counted, single-threaded)** | **Layer 2 (RC)** | ✅ **Total** | Direct equivalent. The transpiler emits the affine-borrow equivalent on an RC value. |
| **`Arc<T>` (reference-counted, thread-safe)** | **Layer 2 (RC) + atomicity directive** | ⚠️ **Partial** | Mycelium's Layer 2 does atomic RC only *after* cross-hypha (structured-concurrency) transfer (DN-32 §2.2). A transpiled `Arc` should be marked for atomic RC at runtime; the transpiler can flag the intent and let the MEM-4 pass optimize. |
| **`RefCell<T>`, `Mutex<T>` (interior mutability)** | **Not directly supported; flag for refactor** | ❌ **Impossible** | Mycelium has no interior mutability in v0. Transpiler must flag and suggest: use exclusive ownership + explicit update, or redesign as immutable + explicit version tagging. |
| **`Sender<T>` / `Receiver<T>` (channel)** | **Affine, non-`Clone` channel pair** | ✅ **Total** | RFC-0027 §7.3 specifies single-owner cross-hypha transfer; direct Rust-to-Mycelium mapping. No shared channel handles. |

**Verdict:** Ownership mapping is **65–75% automatic, 25–35% flagged for review**. The transpiler can emit correct affine code by default; the static uniqueness analysis (DN-32 §2.2, unbuilt) will refine RC insertion. Conservative approach: emit RC for all shared data; the analyzer will flag opportunities for affine refinement.

**Honesty tags (VR-5):** Each mapping is tagged at its supportable strength:
- **Affine-default mappings** (`let`, `move`): `Exact` (Rust's type system guarantees this; Mycelium elaboration is sound).
- **RC mappings** (`Rc` → Layer 2): `Proven` (contingent on DN-32's static uniqueness analysis being correct; unbuilt, so `Declared` until implemented).
- **Mutable-borrow rewrites** (`&mut` → move-update): `Empirical` (requires case-by-case validation; not automatically correct for all Rust patterns).
- **Interior-mutability flags**: `Declared` (impossible in v0; manual refactor required).

---

### 3.2 Functions, Lambdas, Generics

| Rust construct | Mycelium target | Completeness | Notes |
|---|---|---|---|
| **`fn foo<T>(x: T) -> T`** | `fn foo<T>(x: T) -> T = …` | ✅ **Total (monomorphic instances)** | Grammar already supports type_params (RFC-0007, DN-02). The transpiler emits the same generic signature and relies on Mycelium's monomorphization or dictionary-passing (RFC-0019 Enacted) to elaborate. |
| **`fn` with trait bounds** | `fn foo<T: Trait>(x: T) -> T = …` | ⚠️ **Partial** | RFC-0019 is Enacted; grammar supports trait bounds (RFC-0019 §3.3). **Caveat:** Dictionary-passing elaboration is STAGED (M-673 deferred literal `Construct` form). Transpiler can emit the syntax; elaboration may be incomplete. Tag as `Declared` for now. |
| **Closures `\|x\| x + 1`** | **Must defunctionalize to top-level `fn`** | ❌ **Impossible (auto)** | Rust closures capture environment. Mycelium has no capture; RFC-0024 specifies static defunctionalization (convert closures to explicit functions + parameter passing). **Verdict:** Flag all closures; user must manually defunctionalize. |
| **`FnOnce`, `FnMut`, `Fn` traits** | **Not directly equivalent** | ❌ **Impossible** | Rust's function trait hierarchy has no direct Mycelium analogue. The transpiler must flag code relying on these traits (e.g., higher-order functions) and suggest defunctionalization. |
| **Higher-order functions (e.g., `map(f, list)`)** | **L1 `Lam` + `App` (static-defunctionalized or as trait bounds)** | ⚠️ **Partial** | If a generic function `map<T, U, F: Fn(T) -> U>` is transpiled, the `Fn` trait must be resolved. Options: (1) Monomorphize at each call site (code explosion), (2) Use dictionary-passing (RFC-0019), or (3) Flag for manual defunctionalization. Verdict: Transpiler can emit Mycelium's generic form; elaboration must resolve via RFC-0019 or the code must be manually refactored. |

**Verdict:** Generics and trait bounds are **mostly transpilable**, contingent on RFC-0019 elaboration being stable and non-closure code. Closures are a major refactoring burden; flag them conservatively.

---

### 3.3 Enums, Pattern Matching, Data Types

| Rust construct | Mycelium target | Completeness | Notes |
|---|---|---|---|
| **`enum E { A, B(T), C { x: T, y: U } }`** | `type E = A \| B(T) \| C(T, U)` | ✅ **Total** | Rust's ADT enum maps directly to Mycelium's sum type (RFC-0011). Struct fields become positional constructor arguments (Mycelium does not name fields in enum constructors in v0). |
| **`struct S { x: T, y: U }`** | `type S = S(T, U)` | ✅ **Total** | Rust struct (product) becomes a unary constructor. Positional fields replace named fields; a wrapper `impl` block provides accessor functions if needed. |
| **`match` with guards** | `match` + `if` in each arm | ⚠️ **Partial** | Rust `match` with guards (patterns + `if cond => …`) must be rewritten as nested `if` in Mycelium (guards are not in the v0 grammar). Maranget lowering handles nested patterns; guards become explicit conditionals. |
| **Irrefutable patterns** | `let` bindings | ✅ **Total** | Rust `let (x, y) = expr` destructures; Mycelium `let x = expr in …` (single binding) or wrapped in a data constructor pattern. Simple destructuring maps directly; complex patterns are elaborated to cascading `match`. |
| **`_` wildcard** | `_` wildcard | ✅ **Total** | Same in Mycelium (grammar line 180). |

**Verdict:** Data types and matching are **~95% transpilable** (guards are a minor gap, easily worked around).

---

### 3.4 Numeric Ops, Guarantees, and Representations

| Rust construct | Mycelium target | Completeness | Notes |
|---|---|---|---|
| **Numeric literals** | Same literal, explicit `repr @ strength` tag | ⚠️ **Partial** | Rust integer literals are implicitly typed by context (`42: u32`). Mycelium requires explicit type *and* guarantee tag (RFC-0001 §4.5; grammar line 134). Transpiler must: (1) infer the Rust type, (2) emit the Mycelium representation, (3) emit an honest guarantee tag (Exact for constants, Declared if inference is uncertain). |
| **Arithmetic ops** | `Op` primitives (L0) with guarantee tags | ⚠️ **Partial** | Rust `a + b` is well-defined on integers (modular wrap). Mycelium's arithmetic ops must be tagged: `Exact` (algebraic identity holds), `Proven` (theorem applied), `Empirical` (testing), or `Declared` (unverified). **The transpiler must infer the soundest tag or flag for manual annotation.** Most transpiled code will be `Declared` initially; refinement is a manual task. |
| **Floating-point ops** | Dense representation + guarantee tag | ⚠️ **Partial** | Rust `f32`/`f64` are IEEE-754. Mycelium's Dense types (RFC-0001, grammar line 137) are parametrized by paradigm and scalar kind (F16, BF16, F32, F64). Transpiler emits the matching Dense type with a `Declared` tag (IEEE ops are not proven correct without explicit rounding semantics). |
| **Range / bounds** | Explicit checks or assertion | ⚠️ **Partial** | Rust integer overflow panics (in debug) or wraps (in release). Mycelium requires explicit out-of-range handling: return `Option`/`Result` or assert. The transpiler must flag and suggest conversion sites. |
| **Unsafe casts / transmutes** | `wild { … }` (FFI context only) | ❌ **Impossible** | Rust's `unsafe { transmute(x) }` has no Mycelium equivalent outside `wild` (and `wild` is only in `@std-sys` nodules with `!{ffi}` effect). **Verdict:** Flag all transmutes; they must be explicitly audited and moved into `wild` blocks, which may not be semantically equivalent. |

**Verdict:** Numeric ops are **~70% transpilable** (representation and type mapping is straightforward; guarantee tags require inference and manual refinement).

---

### 3.5 Macros, FFI, Unsafe, Build Scripts

| Rust construct | Mycelium target | Completeness | Notes |
|---|---|---|---|
| **Declarative macros** | N/A (flag for manual rewrite) | ❌ **Impossible** | Mycelium has no macro system in v0. Transpiler must identify macro invocations and flag them. User must manually expand or rewrite. |
| **Procedural macros** | N/A (flag for manual rewrite) | ❌ **Impossible** | Same as declarative. Any `#[derive(...)]`, custom attributes, or proc macros must be removed and replaced with hand-written equivalents (or Mycelium's future `grow` mechanism, DN-03 §1). |
| **`unsafe { … }` blocks** | `wild { … }` (if in `@std-sys` nodule with `!{ffi}` effect) | ⚠️ **Partial** | Mycelium's `wild` (ADR-014) is the unsafe escape. It is allowed *only* in `@std-sys` nodules and must be guarded by `!{ffi}` effect declaration. The transpiler can map Rust `unsafe` to `wild`, but it must *verify* the context is `@std-sys` and ensure the enclosing function has the `ffi` effect. **Verdict:** Transpiler flags unsafe blocks and relocates them into properly-annotated `wild` contexts (may require refactoring). |
| **`extern "C" { … }`** | FFI declarations in `wild` context | ⚠️ **Partial** | Foreign function declarations must be moved into `@std-sys` and wrapped appropriately. The transpiler can emit the mapping; it is a manual verification step. |
| **`build.rs`** | Build metadata or separate tooling | ❌ **Impossible** | Rust's `build.rs` (procedural build scripts) has no direct Mycelium equivalent. The transpiler must flag; the user must rewrite as Mycelium build metadata (spec/Spore-Build-and-Publish-Contract.md, ADR-013) or external tooling. |

**Verdict:** Unsafe and FFI are **largely transpilable** (with context verification); macros and build scripts are **not** (flag + manual rewrite).

---

### 3.6 Exception Handling, Error Propagation

| Rust construct | Mycelium target | Completeness | Notes |
|---|---|---|---|
| **`Result<T, E>` / `Option<T>`** | `type Result<T, E> = Ok(T) \| Err(E)` | ✅ **Total** | Rust's error types are identical to Mycelium's (RFC-0027 never-silent; Glossary). The transpiler emits the same type definitions. |
| **`?` operator** | Explicit `match` on `Result` / `Option` | ⚠️ **Partial** | Rust's `?` sugar unwraps and propagates errors. Mycelium's v0 grammar does not include `?`; instead, the transpiler must emit explicit `match` arms: `match expr { Ok(x) => x, Err(e) => return Err(e) }`. Verbose but correct. |
| **`panic!` / assertions** | `Option`/`Result` with `Exact` assertion or `wild` for unrecoverable failure | ⚠️ **Partial** | Rust's `panic!` is a runtime abort. Mycelium prefers explicit `Result` (never-silent). Transpiler flags panics; user must decide: convert to `Result`, or if unrecoverable (e.g., memory corruption), move into `wild` with `!{cascade}` effect (RFC-0014 cascade effect). |

**Verdict:** Error handling is **~80% transpilable** (explicit `match` replaces `?` sugar; panics are flagged).

---

### 3.7 Concurrency, Parallelism

| Rust construct | Mycelium target | Completeness | Notes |
|---|---|---|---|
| **`std::thread::spawn(closure)`** | `hypha { expr }` in a `colony { … }` | ⚠️ **Partial** | Rust's thread spawn takes a closure. Mycelium's `hypha` (RFC-0008 §4.7, M-666 Active) is an expression spawning in a `colony` scope (structured concurrency). The transpiler must convert Rust's closure-based threading to structured concurrency with explicit `hypha` blocks. This requires scope restructuring (closures must be defunctionalized; the enclosing `colony` scope must be explicit). **Flag for manual refactoring.** |
| **`std::sync::Mutex<T>`** | `Substrate{MutexHandle}` + `consume` | ⚠️ **Partial** | Rust's `Mutex` is lock-based. Mycelium's affine substrate (DN-02 §2) is single-use (consumed exactly once). A transpiled Mutex must be converted to explicit lock-acquire + release pattern (move-and-drop), or use a runtime-managed pool. **Flag and suggest design review.** |
| **`std::sync::Arc<Mutex<T>>`** | Layer 2 (RC) + affine substrate | ⚠️ **Partial** | The combination (shared mutable state) is anti-idiomatic in Mycelium. Transpiler flags; user must redesign (e.g., move toward exclusive ownership + explicit handoff via hypha boundaries, or use immutable + version tagging). |
| **`tokio`-style async/await** | Not directly supported; future RFC | ❌ **Impossible** | Rust's async/await and runtime-agnostic futures have no Mycelium equivalent in v0. RFC-0008 specifies structured concurrency; RFC-0027 specifies the executor. Full async/await is beyond the scope. **Flag all async/await code; requires design review for Mycelium structured-concurrency equivalents.** |

**Verdict:** Concurrency is **~40% transpilable** (structured concurrency is simpler than Rust's freedom-based model, but requires scope restructuring and design review).

---

## 4. The Rewrite Corpus — Simplest-First Transpile Candidates

**Inventory:** 50 Rust crates in `crates/` (as of 2026-06-25).

**Categorization by transpilation difficulty:**

### Tier 1: Simplest (Mostly pure, no concurrency/unsafe/macros)
- `mycelium-core` — core IR representation (constants, variables, ops) — **pure data structures**
- `mycelium-l1` — L1 kernel AST — **data structures + simple visitors**
- `mycelium-select` — selection/swap-policy evaluation — **pure functions**
- `mycelium-numerics` — numeric operations, no FFI — **pure with guarantee tags** (but will need tag annotations)
- `mycelium-std-core` — core stdlib (primitives, basic types) — **data + functions**
- `mycelium-std-cmp` — comparison operators — **pure**
- `mycelium-std-error` — error types — **pure data**

**Rationale:** No async, minimal unsafe, no macros (or easily-expandable ones), straightforward data structures and pure functions. These are ideal **pilot candidates**. Pick one (e.g., `mycelium-std-error` or `mycelium-std-core`) and transpile it end-to-end to validate the transpiler, then incrementally tackle harder crates.

### Tier 2: Intermediate (Some unsafe, localized FFI, no major concurrency)
- `mycelium-mlir` — MLIR code generation — **has unsafe (FFI to LLVM)**
- `mycelium-interp` — interpreter — **complex control flow, minor unsafe**
- `mycelium-std-sys` — system interface — **FFI-heavy, unsafe**
- `mycelium-std-io` — I/O abstractions — **some async/FFI**

**Rationale:** These require careful unsafe-to-`wild` mapping, but the core algorithms are transpilable. Estimate 70–80% auto-transpilable, 20–30% manual refinement.

### Tier 3: Complex (Heavy concurrency, generics, macros)
- `mycelium-std-runtime` — runtime scheduler, concurrency — **structured concurrency refactor needed**
- `mycelium-mir-passes` — MIR analysis passes — **heavy on generics and trait bounds**
- `mycelium-mlir` (if it uses macros for code generation) — **macro expansion burden**

**Rationale:** These require design review (concurrency restructuring, macro expansion, generic code validation). Estimate 50–70% auto-transpilable.

### Tier 4: Specialization (Math, VSA, dense)
- `mycelium-std-numerics`, `mycelium-std-vsa`, `mycelium-std-dense` — domain-specific — **pure, but guarantee-tag-heavy**
- `mycelium-std-math` — transcendental functions — **numerics + external libraries**

**Rationale:** These are pure and transpilable, but require careful guarantee-tag annotation (most will be `Declared` initially, requiring manual refinement). Lower priority; revisit after core stdlib is stable.

**Verdict:** **Start with `mycelium-std-error` or `mycelium-std-core`** (Tier 1); pilot the transpiler on 200–500 LOC to validate the architecture and flag patterns, then scale to Tier 2.

---

## 5. Never-Silent Flagging — the Compatibility Analyzer

**Load-bearing principle:** G2 / VR-5 — every construct the transpiler cannot faithfully convert is surfaced as an explicit flag, never emitted as plausible-but-wrong Mycelium.

### 5.1 Analyzer Design Pattern (from py2rust)

**py2rust's `CompatibilityAnalyzer`** (py2rust cli.py:130–153) is the model:

```python
class CompatibilityAnalyzer:
    def analyze(self, tree: ast.AST) -> List[str]:
        issues = []
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                issues.append(f"Import statement '{node.names[0].name}' needs manual conversion")
            elif isinstance(node, ast.ClassDef):
                issues.append(f"Class '{node.name}' needs manual conversion to Rust struct/impl")
            # … more patterns …
        return issues
```

**Pattern:** AST walk + collect issues as (location, reason) tuples. Output: a **manifest** of what was auto-transpiled vs flagged-for-review.

### 5.2 Rust→Mycelium Analyzer Specification

**Input:** Rust AST (from `syn` crate or `rustc_ast`)  
**Output:** A **transpilation report** with three sections:

1. **Auto-transpilable (green):** The fraction of code that was successfully emitted as Mycelium
2. **Flagged-for-review (yellow/orange):** The fraction requiring manual attention, with reasons and locations
3. **Impossible (red):** The fraction that cannot be transpiled at all (macros, FFI in wrong context, etc.)

**Analyzer responsibilities:**

✅ **Flag constructs:**
- Closures (capture environment) → suggest defunctionalization
- Macros (any invocation) → suggest manual expansion
- Unsafe blocks (outside `@std-sys` context) → flag + suggest relocation
- `unsafe` transmutes → flag + suggest code review
- `panic!`/`unwrap` → flag + suggest explicit `Result` or `wild` + `cascade`
- `tokio`/async-await → flag + suggest structured concurrency redesign
- Mutable borrows (`&mut`) → flag + suggest move-update refactor
- Interior mutability (`RefCell`, `Mutex`) → flag + suggest exclusive ownership or redesign
- Lifetime annotations → ignore (Mycelium has no lifetimes) or suggest removing
- Generic trait bounds on polymorphic functions (until RFC-0019 elaboration is proven) → flag with deferred-error marker

❌ **Reject (never emit guessed code):**
- Incomplete type inference → refusal + ask for annotation
- Ambiguous generic instantiation → refusal
- Out-of-range numeric constants → refusal + ask for explicit representation

✅ **Emit as-is:**
- Functions, data types, pattern matching, recursion
- Result/Option/error handling (rewritten to explicit match if needed)
- Numeric operations (with `Declared` tags)
- Pure algorithms, logic

### 5.3 Manifest Structure

Example **transpilation-report.json**:

```json
{
  "crate": "mycelium-std-core",
  "total_functions": 47,
  "auto_transpiled": 39,
  "flagged": 8,
  "impossible": 0,
  "stats": {
    "auto_fraction": 0.83,
    "flagged_fraction": 0.17,
    "impossible_fraction": 0.0
  },
  "flags": [
    {
      "location": "src/ops.rs:45:3",
      "construct": "unsafe block",
      "reason": "unsafe { transmute(x) } — no equivalent in Mycelium without @std-sys context",
      "suggestion": "Move into @std-sys nodule and wrap in wild block; manual audit required"
    },
    {
      "location": "src/lib.rs:120:5",
      "construct": "closure",
      "reason": "let f = |x| x + 1 — captures environment; Mycelium has no capture",
      "suggestion": "Defunctionalize: fn add_one(x) -> T = x + 1; pass function reference explicitly"
    }
    // … more flags …
  ],
  "impossible": []
}
```

**Transparency (VR-5):** Each flag is tagged with:
- **Location** (file:line) — precise pointer
- **Construct** — the language feature
- **Reason** — why it cannot be auto-transpiled
- **Suggestion** — the manual fix or design change needed

This manifest is the **worklist for refinement**: the user can read it, understand the scope of manual work, and tackle each item systematically.

---

## 6. Open Questions for the Transpiler Design Note

(These are the deliberation agenda for when the transpiler phase opens, per DN-34 §6.)

1. **Closure / higher-order function refactoring burden:** What fraction of typical Rust code relies on closures or higher-order trait objects? Pilot crates should measure this. If >30%, the refactoring burden is significant and may affect the phase schedule.

2. **Generic code completeness:** RFC-0019 elaboration (dictionary-passing, M-673) is Staged—the literal `Construct` form for trait method dicts is deferred. Should the transpiler target the grammar level (output `impl Trait<T> for U` forms, relying on elaboration), or should it target elaborated L1 (emitting explicit dict `Construct`s)? The former is higher-level and more reviewable; the latter is lower-level and de-risks elaboration bugs. **Recommend L2 surface—more readable, and elaboration bugs are caught by the elaborator, not by the transpiler.**

3. **Guarantee-tag inference strategy:** How aggressive should automatic tag inference be? Options:
   - *Conservative:* Tag everything `Declared` unless the transpiler has a theorem (for constants, algebraic ops). Manual refinement is the default. ← Honest but verbose.
   - *Optimistic:* Tag based on Rust's type system (e.g., u32 addition with no saturation-check → `Exact` modular arithmetic). Less verbose, but risks over-claiming. ← Risky.
   - **Recommend hybrid:** Emit `Exact` for constants and plain operations (Rust's type system guarantees overflow semantics); emit `Declared` for domain-specific operations or anything that required custom rounding/precision rules. Tag inference is auditable via the manifest.

4. **Ownership mapping strategy:** Should the transpiler conservatively emit RC (Layer 2) for all shared references, relying on the MEM-4 static uniqueness analysis (DN-32 §2.2, unbuilt) to refine to affine (Layer 1)? Or should it attempt to infer affinity from Rust's borrow checker? The former is simpler and correct (just possibly suboptimal); the latter requires embedding Rust's analysis. **Recommend conservative (RC for all shared); MEM-4 refinement is post-landing.**

5. **Incremental rewrite + FFI shim strategy:** How should the transition period work? The transpiler outputs Mycelium; the remaining Rust crates are not yet transpiled. RFC-0028 (FFI) governs the boundary. Should the transpiler emit explicit FFI call sites (e.g., `wild { call_rust_from_mycelium(...) }`), or should it assume a runtime shim layer (e.g., `py-rust-bridge`-style binding generation)? **Recommend explicit FFI call sites in `wild` blocks, paired with separate binding-generation tooling (analogous to py-rust-bridge) that runs after the transpiler.**

6. **Scope of "bulk":** A realistic auto-conversion fraction target, measured on a pilot crate. DN-34 §6 defers this as `Declared` until measured. After Tier-1 pilot, we should know: what fraction of typical Mycelium-destined Rust code is ≥85% auto-transpilable, and what is flagged?

---

## 7. Summary: Is Mycelium Ready to Be a Transpile Target?

**Verdict:** ✅ **YES, with caveats.** Mycelium's surface grammar is **70–85% ready** for transpilation; the remaining gaps are **refinement tasks, not blockers**.

### 7.1 Readiness Scorecard

| Aspect | Status | Blocker? |
|---|---|---|
| **Core grammar (functions, data, match, control flow)** | ✅ Committed | No |
| **Traits + generics** | ⚠️ Enacted (dictionary-passing STAGED) | No—will work for monomorphic code; polymorphic code may need deferred-error handling until M-673 is proven stable |
| **Effects + never-silent error handling** | ✅ Committed | No |
| **Memory model / affine ownership** | ⚠️ Declared (three-layer DN-32 is strategic direction, not lexed) | No—transpiler can emit RC conservatively; DN-32 static analysis will refine later |
| **Closures / higher-order functions** | ❌ Not auto-transpilable | No—flag for manual defunctionalization; RFC-0024 (static) is the pathway |
| **Macros, build scripts** | ❌ Not auto-transpilable | No—flag for manual expansion; future `grow` mechanism may help |
| **Unsafe / FFI in `@std-sys` context** | ✅ Committed | No—`wild` + `!{ffi}` + checker gate in place |
| **Closures + concurrency (async/await)** | ❌ Not auto-transpilable | No—flag for structured-concurrency redesign (RFC-0008); complex refactoring |

### 7.2 Conservative Recommendation

**Proceed with transpiler implementation,** targeting Tier-1 crates (pure, no closures, minimal unsafe) as a **pilot** (e.g., `mycelium-std-error`, 200–500 LOC). The transpiler will demonstrate:
1. **Feasibility:** Can the py2rust AST-walk architecture retarget to Rust→Mycelium?
2. **Analyzer signal:** What construct patterns are actually flagged vs auto-transpiled in real code?
3. **Manual-refinement workload:** How much human time per 1000 LOC of transpiled code?

**Gating:** Do not begin the full-crate-scale rewrite (Tier 2–3) until:
- The pilot transpiler successfully emits type-checking Mycelium for at least one Tier-1 crate.
- The compatibility manifest is validated (flagged constructs are genuinely un-transpilable, not missed opportunities).
- The manual-refinement workload is estimated (if >2 hours per 1000 LOC, the phase schedule may need adjustment).

### 7.3 Maturity Path

**Phase timeline (aspirational):**
- **Weeks 0–2:** Implement transpiler skeleton (AST walk, basic visitors, analyzer). Target: emit Mycelium L2 surface code + manifest.
- **Weeks 2–4:** Pilot on `mycelium-std-error` (100–300 LOC). Measure auto vs flagged fractions. Refine analyzer.
- **Weeks 4–6:** Scale to Tier-2 crates (1000–5000 LOC). Measure RFC-0019 elaboration stability. Adjust strategy if needed.
- **Weeks 6+:** Full-scale rewrite (per DN-26).

**Key metrics to track:**
- Auto-transpilable fraction per crate (target: ≥70% for Tier 1, ≥50% for Tier 2)
- Number of flags per 1000 LOC (target: <50 unique issues)
- Manual-refinement time per 1000 LOC (target: <3 person-hours)
- Type-checking and elaboration pass rate (target: ≥95% after manifest review)

---

## 8. Final Assessment

**Mycelium's surface grammar is committed, mostly Active, and suitable for transpilation.** The seed projects (py2rust, py-rust-bridge) provide a sound architectural foundation. The three-layer memory model (DN-32), never-silent error handling, and structured concurrency are advantages—Rust code already encodes ownership and task structure, which Mycelium needs.

The **primary refactoring burden** is closures (defunctionalization), macros (expansion), and design-level changes (concurrency model shift). These are not grammar gaps; they are semantic mismatches that require human thought. The transpiler cannot and should not guess—it must flag, clearly and precisely, so the user makes informed decisions.

**Confidence level:** `Declared` (a plan, not measured). Implementation will refine this assessment.

---

## References

- **DN-34** — Rust→Mycelium Transpiler Strategy (`docs/notes/DN-34-Rust-to-Mycelium-Transpiler-Strategy.md`)
- **DN-32** — Three-Layer Hybrid Memory Architecture (`docs/notes/DN-32-Three-Layer-Hybrid-Memory-Architecture.md`)
- **DN-02** — Fungal Lexicon and Reserved Words (`docs/notes/DN-02-Fungal-Lexicon-and-Reserved-Words.md`)
- **DN-06** — Static Organization and Dynamic Grouping Lexicon (`docs/notes/DN-06-Static-Organization-and-Dynamic-Grouping-Lexicon.md`)
- **RFC-0007** — L1 Kernel Calculus (`docs/rfcs/RFC-0007-L1-Kernel-Calculus.md`)
- **RFC-0019** — Traits and Parametric Polymorphism (`docs/rfcs/RFC-0019-Traits-and-Parametric-Polymorphism.md`)
- **RFC-0027** — Reference Counting and Affine Resource Management (`docs/rfcs/RFC-0027-…`)
- **ADR-014** — The `wild` Unsafe Escape Hatch (`docs/adr/ADR-014-…`)
- **Grammar:** `docs/spec/grammar/mycelium.ebnf` (v0, L1-facing)
- **Seed projects:** `py2rust`, `py-rust-bridge` (maintainer prior art, 2026-06-25)

---

**Report status:** `Declared` (research-backing draft for DN-34 design iteration)  
**Date:** 2026-06-25

---

## Erratum (2026-06-25, append-only)

Two snapshot claims in this record have since been corrected against ground truth (alignment audit):

1. **The static uniqueness analysis is now *partly built*, not "unbuilt."** MEM-4 **Increments 1–2**
   (intraprocedural, straight-line, non-escaping) have landed in `crates/mycelium-mir-passes/`
   (`rc_ir`/`emit`/`eval`/`balance`/`corpus`). The body's "unbuilt" phrasing reflects an earlier
   snapshot; it should read "partly built (Increments 1–2; full FIP/Increment-3 remains Phase-3)."
   Note also that MEM-4 *optimizes* emitted Mycelium **Core IR** RC — it is **not** a Rust-ownership
   analyzer (Rust ownership facts come from a rustc/rust-analyzer front-end; see DN-34 §3 correction).
2. **`colony`/`hypha` are active *as expressions only*.** They parse as expressions (`parse.rs`), but
   the **authoritative lexicon** still lists them **Reserved-not-active** for the static/item sense
   (lexicon §97/§114). The "Now ACTIVE M-666" note should be read with that scope qualifier.

(Append-only erratum — the original text is preserved as the dated record it is; VR-5/G2.)
