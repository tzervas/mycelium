# Design Note DN-34 — Rust→Mycelium Transpiler Strategy (Self-Hosting Acceleration)

| Field | Value |
|---|---|
| **Note** | DN-34 |
| **Status** | **Draft (advisory)** (2026-06-25) — strategy capture for a **future** phase. Records how a Rust→Mycelium transpiler would accelerate the Mycelium self-hosting rewrite (the `stdlib-and-libraries-in-Mycelium` long pole), leveraging the maintainer's existing **py2rust** + **py-rust-bridge** projects as the seed. **Enacts nothing; ships no code.** The phase is **gated on the Mycelium surface (L1/L2/L3 + stdlib API) being mature enough to be a transpilation *target*** — it is not begun now. |
| **Feeds** | **DN-26** (Self-Hosting Bootstrap Plan — this is the *mechanism* that does the bulk of the rewrite), **DN-27** (Post-1.0.0 Repository Decomposition — the transpiled output is split into component repos), **RFC-0028** (FFI & System Interface — the Rust↔Mycelium interop the transition relies on), **M-502** (stdlib-in-Mycelium migration), **ADR-021/022 + DN-25** (the 1.0.0 gates that schedule self-hosting post-core-1.0). **Generalized by DN-85** — this Rust→Mycelium note is the **first arm** of a multi-language transpilation program (Python/C/C++/Fortran/Cython/CUDA…) whose flagship goal is a single-language Mycelium full stack; the py2rust/py-rust-bridge seeds below are the Python-arm foreshadowing. |
| **Date** | June 25, 2026 |
| **Decides** | *Nothing normatively* — advisory direction capture. Records (1) that a **Rust→Mycelium transpiler** (input: the project's own Rust crates; output: Mycelium surface) is the intended **bulk-rewrite mechanism**; (2) the **maintainer's py2rust + py-rust-bridge** projects are the architectural seed (AST-walk transpilation + never-silent compatibility analysis + the FFI bridge); (3) a **construct-mapping sketch** (Rust → Mycelium) and a **never-silent "flag, don't guess" analyzer** as the load-bearing design; (4) the **phasing** — isolated branch, transpile-then-refine, output decomposed into component repos. |
| **Task** | Self-hosting / Mycelium-rewrite phase (future; M-502 / DN-26 / DN-27) |

> **Posture (transparency rule / VR-5 / G2).** This note is **strategy capture**, not a committed design.
> It **enacts nothing** and **ships no code**. The transpiler phase is **future work**, explicitly gated
> on the Mycelium surface being a viable target; nothing here begins it. Every claim about effort or
> coverage is **`Declared`** (a plan, not a measurement). The seed projects (py2rust, py-rust-bridge)
> are **early-stage skeletons** — their *architecture* transfers (`Empirical`: they exist and define a
> working shape), their *completeness* does not (they are honestly described as early-stage). The
> maintainer's standing intent (provided 2026-06-25): use a transpiler to do the **bulk** of the
> Mycelium rewrite of this project, done in an **isolated branch**, then **busted out into component
> repositories** when complete and verified.

## §1 Why a transpiler — and why Rust→Mycelium

Mycelium's full-language 1.0.0 has a **long pole**: the standard library and libraries must
themselves be **written in Mycelium** (ADR-022 / DN-25 track; M-502), and ultimately the kernel
self-hosts (DN-26). Today the entire implementation is **Rust** (50 crates). Hand-rewriting that
corpus into Mycelium is enormous and error-prone.

A **Rust→Mycelium transpiler** turns that hand-rewrite into a **transpile-then-refine** loop: it
mechanically converts the bulk of the Rust source to Mycelium surface, **flagging** (never silently
guessing) the constructs that need human attention, so the human effort concentrates on the hard
residue rather than the boilerplate. This is the direct analogue of the maintainer's existing
**py2rust** (Python→Rust) work, retargeted: **source = the project's own Rust; target = Mycelium.**

## §2 The seed — py2rust + py-rust-bridge (maintainer's prior art)

Two existing maintainer projects define the reusable architecture:

- **py2rust** (Python→Rust transpiler). Architecture worth transferring:
  - **AST-based transpilation** — parse the source to an AST, walk it, emit target code
    (`PythonToRustTranspiler`). For Rust→Mycelium the input parser is the Rust ecosystem's
    [`syn`](https://docs.rs/syn) (a full Rust AST), so the transpiler is naturally a **Rust tool**
    consuming `syn`'s AST and emitting Mycelium surface.
  - **A `CompatibilityAnalyzer`** that **flags un-transpilable patterns** (imports, classes,
    try/except, lambdas) for manual conversion instead of emitting wrong code. This *flag-don't-guess*
    discipline is **exactly Mycelium's G2 / never-silent ethos** — the single most important property
    to carry over (§4).
  - A two-command CLI (`transpile`, `analyze`) — the same shape fits a Rust→Mycelium tool.
- **py-rust-bridge** (Python↔Rust FFI / SFI bridge). It generates PyO3/cbindgen bindings and analyzes
  Rust for cross-language exposure. Its relevance: the **transition period**. A partially-rewritten
  system has Mycelium and Rust components that must **interoperate** across the boundary — exactly
  what **RFC-0028 (FFI & System Interface)** governs in Mycelium. The bridge's binding-generation +
  interop-analysis approach informs the Mycelium↔Rust FFI shims that let the rewrite proceed
  **incrementally** (one crate at a time) rather than big-bang.

(The seed projects are **MIT** — license-compatible with Mycelium's MIT-only rule, ADR-022 §7. The
actual seed code is **not vendored here**; it lands in the transpiler phase's isolated branch.)

## §3 Construct mapping sketch (Rust → Mycelium) — `Declared`

A first-cut mapping the transpiler would implement (each refined when the target surface settles):

| Rust construct | Mycelium target | Notes |
|---|---|---|
| `fn` / closures | nodule function / L1 `Lam` (RFC-0007) | only *named* fns-as-value via RFC-0024 (**Proposed / pending ratification**, not Accepted); **environment-capturing closures are auto-`Impossible`** and must be flagged (research-18 §3; DN-14 — transitive HOF stays `Residual`) |
| `struct` / `enum` | data declarations → `Construct` / `Match` (RFC-0011, the data registry) | |
| `match` | `Node::Match` (flat, checked-exhaustive) | Maranget lowering already exists |
| ownership / `&` / `&mut` borrows | the **three-layer memory model** (DN-32): affine move · RC · regions | Rust ownership/borrow facts come from a **rustc/rust-analyzer front-end** (authoritative source = rustc MIR `mir_borrowck`); `syn` is syntax-only, no ownership facts. **MEM-4 is *not* this analyzer** — it is a downstream RC-insertion/elision optimizer over Mycelium **Core IR** (currently intraprocedural / straight-line / non-escaping; `Lam` params Owned, recursion refused). The Rust→affine *mapping* is the transpiler's own job; MEM-4 can later *optimize* the emitted RC (see §3 closing ¶). |
| `Result` / `Option` / `?` | never-silent `Option`/`Result` (`std.error`, G2) | the types are a natural fit (Rust is already explicit), but **the `?` operator is absent from the v0 grammar** — lower to an explicit `match` (research-18 §2.1) |
| traits / generics | RFC-0019 traits + monomorphization (M-673 landed) | |
| `unsafe` | `wild` (ADR-014 — explicit per-use, source-visible) | flagged; never silently transpiled |
| numeric/approx ops | guarantee-tagged ops (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`) | the transpiler emits honest tags or **flags** where it cannot establish one |
| macros / `build.rs` / FFI | **flag for manual conversion** (analyzer) | the hard residue |

The synergy, stated correctly: **Mycelium's memory model is an *output-optimization* asset, not an
*input-analysis* one.** Rust *as compiled by rustc* does encode the ownership facts the affine mapping
wants — but those facts are recoverable only from a **rustc/rust-analyzer front-end** (authoritative:
rustc MIR `mir_borrowck`), **not** from `syn` (syntax-only, no ownership facts — research-19 §1.2) and
**not** from MEM-4. MEM-4 (`mycelium-mir-passes`) consumes Mycelium **Core IR** *after* the transpiler
has already produced it; it is a downstream RC-insertion/elision pass (intraprocedural, straight-line,
non-escaping today — recursion refused, `Lam` params Owned), so it cannot recover Rust's cross-function
move/borrow structure. The honest division of labor: the transpiler does the **Rust→affine mapping**
itself (front-end facts → DN-32 layers, flagging what it can't establish), and MEM-4 can *later*
**optimize** the emitted Mycelium RC. Conflating the three analyses (rustc borrowck ≠ `syn` syntax ≠
MEM-4 RC-elision) is a category error this note previously made; corrected here per the 2026-06-25
alignment audit (research-18 §3 / research-19 §1.2). *(Open: ownership-mapping fidelity, §6 Q3.)*

## §4 The load-bearing principle — flag, don't guess (G2)

The single most important property carried from py2rust's `CompatibilityAnalyzer`: **a construct the
transpiler cannot faithfully convert is surfaced as an explicit, located flag — never emitted as
plausible-but-wrong Mycelium.** This is `analyze` as a first-class output, not an afterthought:

- Every transpiled artifact carries a **manifest** of what was auto-converted vs flagged-for-review,
  with locations and reasons (an EXPLAIN trail — RFC-0005/0013).
- A flagged construct blocks "done" until a human resolves it — the transpile-then-refine loop's
  ratchet. This mirrors the house rule (`/dev-workflow`: keep an explicit refusal + a clear message;
  never ship fragile output to look complete).

## §5 Phasing (future; gated)

1. **Prerequisite gate.** The Mycelium **surface (L1/L2 + the stdlib API)** must be mature enough to
   be a transpilation *target* — i.e. the constructs in §3 must be expressible. This is post-core-1.0
   (ADR-021/022), aligned with the M-502 / DN-26 schedule. **Not now.**
2. **Seed + retarget.** Stand up the transpiler in its **own isolated branch**, seeding from
   py2rust's AST-walk + analyzer architecture, retargeted to `syn` (Rust AST) → Mycelium surface,
   with the §4 flag-don't-guess analyzer first.
3. **Incremental, interop-bridged rewrite.** Transpile crate-by-crate (leaf crates first), using
   RFC-0028 FFI shims (informed by py-rust-bridge) so transpiled-Mycelium and not-yet-transpiled-Rust
   **interoperate** during the transition — never a big-bang cutover.
4. **Refine the flagged residue** by hand (the analyzer's manifest is the worklist).
5. **Verify** each transpiled crate against its Rust original (the same **differential** discipline
   used throughout: behaviour-equivalence + the guarantee-tag review).
6. **Decompose** the verified Mycelium output into **component repositories** (DN-27) — the rewrite
   and the repo-split land together.

## §6 Open questions (deliberation agenda for when the phase opens)

1. **Transpiler home & language.** A Rust tool over `syn` (native AST, runs in the workspace) vs
   extending the Python py2rust. The `syn` route keeps it in-ecosystem and lets it emit
   `mycelium-core` terms directly — but note `syn` is **syntax-only**: recovering Rust ownership/borrow
   facts requires a rustc/rust-analyzer front-end (rustc MIR `mir_borrowck` is authoritative; research-19
   §1, §6 Q7). `mycelium-mir-passes` (MEM-4) is reusable only as a downstream RC-*optimizer* over the
   emitted Core IR, not as the front-end ownership analyzer (see §3).
2. **Target surface level.** Transpile to the **L2 surface** (ergonomic, human-refinable) vs straight
   to **Core IR** (mechanical, less reviewable). L2 is likely better for the refine loop.
3. **Ownership mapping fidelity.** How much of Rust's borrow structure maps cleanly onto the DN-32
   layers vs needs flagging — measure on a sample crate before committing.
4. **Interop boundary.** How much RFC-0028 FFI shimming the incremental transition needs, and whether
   py-rust-bridge's binding-gen is reused or reimplemented Mycelium-side.
5. **Verification bar.** Per-crate differential equivalence (behaviour) + guarantee-tag preservation —
   the acceptance criterion for "this crate is rewritten".
6. **Scope of "bulk".** A realistic auto-conversion fraction target (the rest flagged) — `Declared`
   until measured on a pilot crate.

## §7 Relation to the corpus & grounding

- **Corpus:** DN-26 (self-hosting bootstrap — the mechanism this note supplies), DN-27 (post-1.0
  repository decomposition — the output target), RFC-0028 (FFI/interop — the transition bridge),
  RFC-0007/0011/0019/0024 (the target surface: L1 calculus, data + match, traits, HOF), DN-32 /
  DN-33 / `mycelium-mir-passes` (the memory model; MEM-4 is a downstream RC-*optimizer* over the
  emitted Core IR — not the Rust-ownership analyzer, which is a rustc/RA front-end's job; see §3),
  ADR-014 (`wild`/unsafe mapping), ADR-021/022 + DN-25 (the 1.0.0 gates scheduling self-hosting),
  G2 / VR-5 (never-silent / honest tags — the analyzer's core discipline), MIT-only (ADR-022 §7 — the
  seed projects are MIT-compatible).
- **Seed prior art (maintainer-provided, 2026-06-25):** **py2rust** (Python→Rust transpiler:
  AST-walk + `CompatibilityAnalyzer`) and **py-rust-bridge** (Python↔Rust FFI/SFI bridge: PyO3/
  cbindgen binding generation + Rust interop analysis). Architecture `Empirical` (the projects
  exist); transfer to Rust→Mycelium is `Declared`.

## §8 PoC results — the first code spike (M-873, kickoff `trx`, 2026-07-01) — `Empirical`

The **first code** against this strategy landed as `crates/mycelium-transpile` (a spike, not the
gated full phase of §5 — DN-34 stays **Draft**; nothing here is Enacted). It reads one Rust crate's
`syn` AST and emits (a) a best-effort `.myc` for expressible constructs and (b) a never-silent,
structured **gap report** (`{file, line, rust_construct, reason, category}` JSON). PoC target:
`crates/mycelium-std-cmp`, diffed against its hand-written twin `lib/std/cmp.myc` (M-714/DN-66).

### §8.1 Seed correction (`Empirical`, from reading the repos)
DN-34 §2 posted the `py2rust`/`py-rust-bridge` seed as "architecture transfers, completeness does
not." A direct read (2026-07-01) confirms and **sharpens** that: both are ~150-line early scaffolds;
`py2rust`'s `CompatibilityAnalyzer` is an **allowlist of four known-bad constructs with a silent
pass-through default** — i.e. the *opposite* of never-silent. There is no reusable visitor, mapping
registry, structured gap record, or `syn` usage to lift. **Correction carried into the PoC:** the
transpiler is built on `syn` with an **exhaustive dispatch** whose fallback arm *always records a
gap* (never an allowlist). So the seed transferred a *naming/CLI shape* and a cautionary
anti-pattern, not an implementation — the §4 flag-don't-guess principle had to be *built*, not
inherited. (This does not change §2's posture; it grounds it with measured specifics.)

### §8.2 Measured expressibility on `std-cmp` (`Empirical` — the DN-34 §6-Q6 "auto-conversion fraction")
Against the **current** surface, **without macro expansion**: **4 of 111 non-test top-level items
emitted ≈ 3.6% expressible**; 112 gap records (incl. sub-item gaps). This is the pilot-crate
measurement §6-Q6 asked for — but it is a **lower bound**, because the dominant blocker is
un-expanded macros (see the backlog). Emitted (all grammar-checked against
`docs/spec/grammar/mycelium.ebnf`, but **unvalidated** — no Mycelium parser/checker confirms the
output, tagged `Declared`): `type Ordering`, inherent `impl Ordering { reverse }`, tuple-payload
`type Bf16Bits`, one `use` import. **Diff vs the twin:** `Ordering` and `reverse` are genuine
matches; the twin's other helpers (`is_lt`, `cmp{N}`, …) are its own hand-refinements absent from the
Rust source; `Bf16Bits` is emitted-with-no-twin-counterpart. Never a silent mismatch — every
non-emitted top-level item is in the gap report (property-tested).

**Honesty note (G2/VR-5, resolved during review):** an initial pass emitted 12 numeric-widening
`impl Widen[…] for …` blocks with a fabricated `from(self)` body (the `as`-conversion has no
established Mycelium surface form, and `from` is not a builtin). That is exactly the
plausible-but-wrong emission §4 forbids; it was **reclassified to gaps** (dropping 16→4 emitted).
The emitter now gaps *any* fn/impl body it cannot faithfully lower rather than inventing one.

**Follow-on lift — DN-41 `width_cast` faithful emission (2026-07-01, same PR series).** The 3.6%
figure above was *pre-conversion-surface*. A hardening pass then wired the emitter to the **landed,
Accepted `width_cast(value: Binary{N}, into: Binary{M})` prim (DN-41)**: unsigned `Binary` widening
`impl`s now emit a **real** `width_cast(self, <Binary{M} witness>)` body (the witness is a synthesized
all-zero `BinLit` of exactly `M` bits — grammar-confirmed width-from-content, RFC-0020; DN-41 §3 says
the witness's bits are unused, so this is faithful), not the fabricated `from`. This raised **std-cmp
from 4→14 emitted (3.6%→12.6%)** — 10 conversion `impl`s became genuine emissions. What stays gapped
is honest: **signed**-integer widening (ADR-028 scoped `Binary` sign-free — a real semantic gap, not a
shortcoming), `bool`-`Self` widening (no witness path), and all **narrowing** (DN-41 makes it
fallible/`Result`, which a single `= expr` body can't express). This is the principle in action: emit
a body **iff** it maps to a *confirmed real* surface form, else gap it (never guess a form).

### §8.3 Prioritized surface-feature backlog (the demand data for E18-1 `needs-design`) — `Empirical` counts / `Declared` rankings
Ranked by measured frequency times blocking value on `std-cmp`. This is the first-class output the
kickoff asked for — real demand data, not a guessed roadmap:

| # | Missing capability | Gap count (std-cmp) | Note |
|---|---|---|---|
| 1 | **Macro handling** (`macro_rules!` and invocations) | 62 (~55%) | The dominant blocker. Best addressed transpiler-side by **expand-first** (`cargo expand`/rustc) — turns these into ordinary impls — more than by a Mycelium macro surface. An *architecture* decision (§6-Q1 addendum), not only a language gap. |
| 2 | **Trait `impl`s and conversion/`as`-cast op bodies** | 27 | Numeric widening/narrowing (`self as T`) has no expressible body; reconcile with **DN-41 width-cast** — a genuine surface gap. |
| 3 | **Trait definitions** (default-method bodies, `Self`, supertrait bounds) | 5 | `trait_item` exists; `Self`-referent default bodies are the gap. |
| 4 | **Trait-bounded generics** (`<T: Bound>`) | 4 | `[T]` type-params landed (M-656/7); the *bound* surface is the gap. |
| 5 | **Struct-like / generic-payload enum variants** (error enums) | 2 | `ClampError<T>` / `NarrowError` multi-field, generic-payload constructors. |
| 6 | **Derive attributes** (`#[derive(...)]`) | 3 | reconcile with DN-54 `derive` elaboration. |
| 7 | **Named-field structs** (beyond single-positional tuple) | 1 | `MatrixRow`. |

Tail gaps: associated consts, inner attributes (`#![…]`), and multi-statement fn bodies (the last
did not dominate on this declarative crate but will elsewhere — a Mycelium fn body is one `= expr`).

**Load-bearing conclusion:** on a real crate the current surface expresses ~4% *directly*, but
**~55% of the residue is macro-generated** — so the highest-leverage next step is **transpiler-side
macro expansion**, after which the language-surface gaps (rows 2–7) become the true `needs-design`
worklist. This re-weights the §5 "bulk mechanism" cost model: expansion converts a large mechanical
fraction cheaply; the irreducible human/design work is the surface gaps, not the boilerplate.

### §8.4 Token cost (`Empirical` subagents · `Declared` orchestrator overhead)
The build-the-transpiler and PoC spike (§5a rows) cost, **measured**: scoping 83k, emitter build
254k, one review-correction round 207k = **545k subagent tokens**, plus orchestrator overhead (not
self-measurable, est. ~0.3–0.4M) so **fully-loaded ≈ 0.85–0.95M tokens** for ~2.5k Rust LOC plus
fixtures. This sits **at/below the low end of §5a's `Declared` "first spike ~1–3M"** estimate — the
first real data point, suggesting the §5a build/spike figures were, if anything, conservative. The
§5a rows are annotated with this measurement; the full execute-plus-refine figure remains `Declared`
(unmeasured).

### §8.5 Union across the core-lib slice (`Empirical`, 6 crates) — the demand-grounded backlog
The hardening pass added a directory/batch mode and ran the transpiler over the Rust crates backing
**6 of the 8** core-lib twins (`fixtures/UNION-BACKLOG.md` + `union-backlog.json`):

| Twin | Rust crate | Non-test items | Emitted | Expressible % |
|---|---|---:|---:|---:|
| `std.cmp` | `mycelium-std-cmp` | 111 | 14 | 12.6% |
| `std.iter` | `mycelium-std-iter` | 55 | 10 | 18.2% |
| `std.collections` | `mycelium-std-collections` | 31 | 10 | 32.3% |
| `std.text` | `mycelium-std-text` | 65 | 2 | 3.1% |
| `std.fmt` | `mycelium-std-fmt` | 32 | 0 | 0.0% |
| `std.math` | `mycelium-std-math` | 52 | 7 | 13.5% |
| **grand union** | — | **346** | **43** | **12.4%** |

**Re-ranked backlog across the whole slice** (supersedes the §8.3 single-crate ranking as the broader
demand signal): (1) **unsupported *types* — 121 gaps / 36%** (`String`/`text`, `usize`/`isize`,
`char`, closures, references; and **signed integers** — a *real* ADR-028 sign-free consequence, not a
transpiler shortcoming, needing a design decision); (2) **macros — 73 / 22%** (`macro_rules!` + item
invocations); (3) **trait-bounded generics — 39 / 12%**; (4) whole-impl failures — 37 / 11%;
(5) **named-field structs / record types — 23 / 7%**; (6) payload-variant enums — 11; (7) derive attrs
— 8; (8) trait defs w/ `Self` bodies + supertraits — 5. So at slice scale the #1 lever shifts from
"macros" to **surface type-coverage** (String/text, platform-width ints, signed `Binary`), with macro
expansion #2 — both `needs-design`/architecture items for E18-1.

### §8.6 A grounded self-hosting data point: 2 of the 8 "twins" have NO Rust origin (`Empirical`)
`std.option` and `std.result` were **excluded** from the union corpus because a grep for `enum
Option`/`enum Result` across every `crates/*/src/**/*.rs` found **zero** matches — their `.myc` was
authored **directly in Mycelium** (self-hosted: `option.myc`→M-715, `result.myc`→M-649 "first stdlib
module written in Mycelium-lang"), with no Rust prototype to transpile. Flagged, not substituted
(VR-5/G2). This is a real signal for the self-hosting narrative: part of the core-lib slice is
*already* Mycelium-native, so the transpiler's job is the Rust-backed remainder, not the whole.

### §8.7 The transpile → `myc check` vet loop, and the baseline it exposes (M-1000, kickoff `trx2`) — `Empirical`

Everything in §8.2/§8.5 measured **`expressible_fraction`** — "some `.myc` text was emitted for the
item". That number never ran the toolchain over the emission, so it **over-counts** by construction: an
emitted fragment that does not parse or type-check still counts as expressible. M-1000 closes the loop.
The transpiler now has a `--vet` mode (and `src/vet.rs` + `scripts/checks/transpile-vet.sh`) that runs
the **real** `myc check` oracle (`crates/mycelium-check`, the per-file oracle mode
`scripts/checks/myc-dogfood.sh` uses) over every emitted `.myc`, folds each outcome into a structured
never-silent vet record (`vet.json`: exit class + first diagnostic), and reports a second metric:

- **`checked_fraction`** — **myc-check-clean** coverage. Denominator: **non-test top-level items** (the
  *same* denominator as `expressible_fraction`, stated, so the two are directly comparable and
  `checked_fraction ≤ expressible_fraction` always holds). Numerator: **file-gated** — `myc check` is a
  per-file verdict, so a file's emitted items are credited only when the file's *entire* emitted `.myc`
  is clean; a file that fails contributes `0` (we never guess which item broke a failing file — VR-5/
  G2). So `checked_fraction` is an honestly-conservative all-or-nothing-per-file lower bound. An oracle
  that cannot be *run at all* (binary absent) is recorded as `ToolUnavailable` — **never** counted as
  clean.

**Guarantee:** the emitted `.myc` stays `Declared`; the vet verdict is `Empirical` (measured by the real
toolchain — never `Proven`: the oracle's own depth is name-visibility, M-365).

**Baseline the vet loop exposes (`Empirical`, measured over the current emitter, kickoff `trx2`).** The
gap between "emitted" and "checks" is stark — the number that matters for the port is near-zero
everywhere, because on **every** representative target at least one emitted construct poisons the whole
file's parse/check:

| Target | Kind | non-test items | `expressible_fraction` | `checked_fraction` | dominant poison |
|---|---|---:|---:|---:|---|
| `mycelium-l1/src/eval.rs` | semcore | 42 | 11.9% (5) | **0.0%** (0) | reserved-word patterns (`Exact`/`Proven`/…) → parse error |
| `mycelium-l1/src/fuse.rs` | semcore | 10 | 20.0% (2) | **0.0%** (0) | emitted items are both unresolved `use`s |
| `mycelium-std-time/src` | stdlib | 37 | 10.8% (4) | **0.0%** (0) | unresolved `use` → check error |
| `mycelium-std-rand/src` | stdlib | 34 | 14.7% (5) | **0.0%** (0) | unresolved `use` + unknown prim (`rotate_left`) |
| `mycelium-std-cmp/src` | pilot | 111 | 12.6% (14) | **0.0%** (0) | unresolved `use` + `impl` of undefined trait `Widen` |

**The two poison classes the vet loop ranks** (which `expressible_fraction` was blind to) are
(1) **unresolved `use` imports** — the emitter renders a Rust `use extern_crate::Sym` as
`use extern_crate.Sym;`, but that path resolves to no Mycelium nodule (the transpiler has no
cross-nodule symbol table), so the oracle rejects it — **universal** across the surface; and
(2) **reserved-word collisions** — a Rust identifier that is a Mycelium reserved word (`Exact`, `F16`,
`Binary`, …) emitted verbatim into pattern/constructor/type position fails to **parse**. These are the
demand data M-1001 acts on, and the re-ranking is itself a finding: the highest-value lever for
*checked* coverage is **not** §8.3's #1 (macros, which block *emission*) but the constructs that poison
an otherwise-clean file's *check* — the vet loop measures a different thing than the emission heuristic,
and says so.

### §8.8 Closing the top vet-blocking gap classes, and the M-991 go/no-go (M-1001, kickoff `trx2`) — `Empirical`

M-1001 acts on §8.7's re-ranking: it closes the two **checked_fraction-blocking** classes the vet loop
surfaced, both as honest flag-don't-guess corrections (the §8.2 `from(self)` precedent), not new
emissions:

1. **Unresolved `use` imports → gapped** (`Category::Import`). `dispatch_use` no longer emits a `use`
   line: the transpiler has no cross-nodule symbol table, so it cannot confirm any imported path
   resolves to a declared nodule, and the vet loop shows these imports fail `myc check` every time. An
   emitted `use` was also the *universal* poison — one unresolved import fails the whole draft's check.
   Flagging it (the gap's snippet still carries the original `use …;` for the human port) is the same
   stance `map_type`/`emit_expr` already take on qualified paths/calls (§4/§8.2).
2. **Reserved-word collisions → gapped** (`Category::ReservedWord`, `src/reserved.rs`). A Rust
   identifier that is a Mycelium reserved word (`Exact`/`F16`/`Binary`/… — a verbatim snapshot of
   `mycelium-l1`'s lexer keyword table, drift-guarded by a dev-dep test) emitted into
   pattern/constructor/type/fn position fails to **parse**; the transpiler has no sanctioned
   auto-rename (the port's per-type ctor prefixing is a human decision), so a collision is gapped, not
   emitted un-parseably.

**Before → after `checked_fraction` (`Empirical`, before = §8.7 baseline, after = M-1001 emitter).** Both
representative wins go from 0 to positive — a **semcore module** and a **stdlib crate**, per the DoD:

| Target | Kind | non-test | `checked` before → after | `expressible` before → after | after-state |
|---|---|---:|---:|---:|---|
| `mycelium-l1/src/eval.rs` | **semcore** | 42 | 0.0% → **2.4%** (1) | 11.9% → 2.4% | **Clean** — `type ForageError` checks; the 3 unresolved `use`s + the reserved-word `strength_of` body are now gaps |
| `mycelium-std-time/src` | **stdlib** | 37 | 0.0% → **8.1%** (3) | 10.8% → 8.1% | **Clean** — 3 `type`s check; the 1 `use` is now a gap |
| `mycelium-std-cmp/src` | pilot | 111 | 0.0% → 0.0% | 12.6% → 11.7% | still CheckError — residual: the 10 `Widen` impls fail check (`impl` for **undefined external trait**) |
| `mycelium-std-rand/src` | stdlib | 34 | 0.0% → 0.0% | 14.7% → 11.8% | still CheckError — residual: an emitted method-call to a **non-prim** (`rotate_left`) |
| `mycelium-l1/src/fuse.rs` | semcore | 10 | 0.0% → 0.0% | 20.0% → 0.0% | honest zero — `fuse.rs`'s only "emissions" *were* the two unresolved `use`s; the emitted nodule is now empty (and trivially clean) |

The pattern is the vet-loop thesis in action: `expressible_fraction` **fell** where it was over-counting
(the fake `use`/reserved-word emissions), `checked_fraction` **rose** where a real emission was being
poisoned, and on the clean files the two now **coincide** — the honest signal.

**Residual gap-class worklist, ranked by count** (`Empirical`, union over the l1 semcore SCC plus
`std-time`, `std-rand`, and `std-cmp`; 673 gaps). These are the follow-on backlog for *checked*
coverage, distinct from §8.3's *emission* ranking:

| Rank | Class | Count | For `checked_fraction`, this is… |
|---|---:|---:|---|
| 1 | **Other** (unsupported types/exprs) | 274 | mostly type-coverage — `String`/text, **signed ints** (ADR-028 sign-free `Binary`), `usize`/`isize`, `char`, closures, references; a *language-surface* gap (§8.5 #1). |
| 2 | **Impl** (whole-impl failures) | 93 | the `Widen`/external-trait impls that emit but fail check (`impl` for unknown trait). A synthetic trait-def was tried and **fails** (`unknown type Self` / arg-type mismatch) — a real trait-surface gap, not cheaply closeable. |
| 3 | **Struct** (named-field/record) | 70 | no record/product-type surface (§8.3 row 7) — language design. |
| 4 | **Import** (`use`) | 69 | **now correctly gapped (M-1001)** — resolvable only by a cross-nodule symbol table / project-mode vetting, not single-file oracle. |
| 5 | **MacroInvocation/MacroDef** | 64 | blocks *emission*, not *check* — an un-emitted macro is absent, not a poison; hence **lower** priority for `checked_fraction` than the §8.3 ranking implied. |
| 6 | **GenericBound** (34), **PayloadVariant** (21), **DeriveAttr** (19), **ReservedWord** (14, now gapped), **Trait** (8), **MultiStmtBody** (3) | — | the surface/design tail. |

**M-991 assessment (go/no-go — this discharges M-991's DoD).** On the heavy semcore core the
transpiler's *direct* `checked_fraction` is very low (`eval` 2.4%; most SCC modules 0% — their content is
multi-statement bodies, external-trait impls, and reserved-word-colliding type vocabularies the current
surface cannot express): **NO-GO as an automated bulk transpiler for the 15k-line semcore port** — the
residue is irreducible language-surface/human design work, not boilerplate a transpiler converts cheaply
— but **GO as a never-silent gap-profiling instrument**, because the vet loop turns "hand-porting is
brutal" into a *ranked, real-toolchain-vetted* worklist of exactly which surface gaps block the port
(the table above), which is the leverage §8.5 predicted and now grounds with *checked*, not merely
*emitted*, numbers. The documented transpile → vet → fix loop is `scripts/checks/transpile-vet.sh` +
`--vet` (§8.7).

### §8.9 The wave-1 rip-through: `gen/myc-drafts/` over the full port surface (M-1002/M-1003, kickoff `trx2` E-B) — `Empirical`

§8.7/§8.8 measured the vet loop on a five-file *representative sample*. M-1002/M-1003 run it over the
**entire** maintainer-confirmed wave-1 port surface (E33-1 launch-scope record) — all five
`mycelium-l1` semantic-core files plus **all twelve** unported stdlib crates, not a sample — into a
dedicated, greenfield staging tree (`gen/myc-drafts/`, outside `lib/` so `/myc-dogfood` never sees
these `Declared` drafts) with a deterministic manifest (`gen/myc-drafts/{MANIFEST.md,manifest.json}`,
regenerated by `gen/myc-drafts/regenerate.sh` — pure orchestration over the existing `--vet` CLI, no
new transpiler logic). Two runs at the same commit produce a byte-identical manifest and every
`.myc`/`.gap.json`/`vet.json` artifact (verified: a full-tree `sha256sum` over all 158 generated
files matched across two consecutive regenerations).

**Guarantee tags unchanged from §8.7/§8.8:** emission stays `Declared`; the vet verdict is
`Empirical` (measured by the real `myc check` oracle, never `Proven`).

**Per-target results (`Empirical`, all 17 wave-1 targets, non-test-item denominator stated per
§8.7).** `checked` = myc-check-clean items (file-gated numerator); `expressible` = emission-only:

| Target | Kind | non-test items | emitted | checked | `expressible_fraction` | `checked_fraction` |
|---|---|---:|---:|---:|---:|---:|
| `checkty.rs` | semcore | 110 | 0 | 0 | 0.0% | **0.0%** |
| `elab.rs` | semcore | 37 | 0 | 0 | 0.0% | **0.0%** |
| `eval.rs` | semcore | 42 | 1 | 1 | 2.4% | **2.4%** |
| `mono.rs` | semcore | 46 | 0 | 0 | 0.0% | **0.0%** |
| `fuse.rs` | semcore | 10 | 0 | 0 | 0.0% | **0.0%** |
| `std-conformance` | stdlib | 0 | 0 | 0 | 0.0% | **0.0%** (crate is intentionally test-only — honest 0/0, not a defect) |
| `std-content` | stdlib | 35 | 3 | 3 | 8.6% | **8.6%** |
| `std-dense` | stdlib | 20 | 0 | 0 | 0.0% | **0.0%** |
| `std-fs` | stdlib | 53 | 7 | 5 | 13.2% | **9.4%** |
| `std-io` | stdlib | 63 | 6 | 3 | 9.5% | **4.8%** |
| `std-numerics` | stdlib | 27 | 1 | 1 | 3.7% | **3.7%** |
| `std-rand` | stdlib | 34 | 4 | 0 | 11.8% | **0.0%** |
| `std-runtime` | stdlib | 145 | 18 | 9 | 12.4% | **6.2%** |
| `std-sys` | stdlib | 63 | 1 | 1 | 1.6% | **1.6%** |
| `std-sys-host` | stdlib | 6 | 2 | 2 | 33.3% | **33.3%** |
| `std-time` | stdlib | 37 | 3 | 3 | 8.1% | **8.1%** |
| `std-vsa` | stdlib | 31 | 0 | 0 | 0.0% | **0.0%** |
| **Union (all 17)** | — | **759** | **46** | **28** | **6.1%** | **3.7%** |

`eval.rs` (2.4%) and `std-time` (8.1%) exactly reproduce §8.8's post-M-1001 measurements —
cross-validating that the wave-1 driver and the standalone `--vet` invocation agree. Two new
`CheckError`-poisoned targets (`std-fs`, `std-io`, `std-rand`, `std-runtime` — 5 files vetted
`CheckError` out of 56 vetted total, 51 `Clean`) sit alongside the honest zeros
(`checkty`/`elab`/`mono`/`fuse`/`std-dense`/`std-vsa`/`std-conformance` — 7 of 17 targets at exactly
0.0% `checked_fraction`, matching the ~0–8% range E-A's verdict (§8.8) predicted for the port
surface generally).

**Union residual gap-class worklist (`Empirical`, 812 gaps across all 17 targets), ranked:**

| Rank | Class | Count | Note |
|---|---:|---:|---|
| 1 | **Other** (type-coverage) | 322 | the dominant class again — confirms §8.8's #1 finding holds at full wave-1 scale, not just the 5-file sample. |
| 2 | **Impl** | 119 | external-trait / whole-impl failures (the `Widen`-class residue from §8.8). |
| 3 | **Import** (`use`) | 117 | correctly gapped per M-1001 — universal across the surface, as §8.7 found. |
| 4 | **Struct** | 80 | record/product-type surface — language design, not a transpiler defect. |
| 5 | **GenericBound** | 59 | bounded-generic surface (§8.3/§8.5 backlog, still open). |
| 6 | **DeriveAttr** (42), **TestItem** (33, out-of-scope by design), **PayloadVariant** (19), **ReservedWord** (8, now gapped per M-1001), **MacroInvocation** (6), **MultiStmtBody** (4), **Trait** (3) | — | the tail, consistent in composition with §8.8's ranking. |

**What this confirms (discharges E33-1's wave-1 DoD; feeds M-1006).** The full-surface run does
**not** overturn E-A's M-991 go/no-go (§8.8) — it *grounds* it at full scope instead of a five-file
sample: real `checked_fraction` on the actual port surface is **~0–8%** (union 3.7%; `std-sys-host`'s
33.3% is the high outlier, on a 6-item crate too small to generalize from), confirming the maintainer's
calibration that this rip-through's value is the **vetted draft corpus + manifest + per-target gap
profile**, not bulk-ported code. Every one of the 17 targets produced *some* artifact (draft `.myc` +
gap/vet record) — zero silent holes, zero hard transpile-parse failures (all 17 targets: `syn` parsed
every file; `manifest.json`'s `status` field is `"ok"` for all 17).

**Lessons (seed the M-1006 ladder's phase-1 input):**

1. **The type-coverage gap (`Other`, 322/812 — 40%) is the single highest-leverage lever for
   *checked* coverage at scale**, exactly as §8.8's 5-file sample suggested (274/673 there, also
   ~41%) — this ranking is now confirmed stable across a 17-target, 3×-larger corpus, not an artifact
   of the sample. Closing common sub-cases (signed integers under ADR-028, `String`/text, `usize`/
   `char`) is E18-1 `needs-design` work, not a quick transpiler fix.
2. **`checked_fraction` stays near-zero even where `expressible_fraction` is non-trivial** (e.g.
   `std-rand` 11.8% expressible → 0.0% checked; `std-io` 9.5% → 4.8%) — the CheckError residue
   (external-trait impls, non-prim method calls) is a *different* blocker class than the
   emission-blocking one, confirming §8.7's original thesis holds at scale: emission coverage and
   check-clean coverage are genuinely different metrics that must both be tracked.
3. **A crate can be honestly `0/0`** (`std-conformance` — intentionally test-only, no library
   surface) — the manifest represents this as a real, explained zero rather than an error, which is
   the correct never-silent behavior (G2) and should be expected again as the M-1006 ladder covers
   more of the corpus (test-harness-only crates are not uncommon).
4. **Destination-convention metadata (semcore → `compiler.semcore` single-nodule merge,
   FLAG-ast-5/FLAG-parse-2 ctor-prefixing) has to be carried *alongside* the transpile output, not
   derived from it** — the transpiler has no notion of "these five files merge into one nodule"; the
   manifest's per-target `note` field is currently a hand-maintained annotation, not a measurement.
   Any future automation that tries to derive nodule-merge destinations from Rust source structure
   is a distinct, harder problem than transpilation itself.
5. **The driver's own hazards were real, not hypothetical**: an early cut of the manifest generator
   baked this checkout's *absolute* filesystem path into `vet.json` (via the out-dir argument handed
   to `mycelium-transpile --vet`) — caught and fixed by insisting on repo-root-relative out-dirs
   before treating the manifest as deterministic; a similar early cut crashed on a target whose input
   was simply missing rather than recording a graceful `transpile_failed` row. Both are now the
   standing shape of `gen/myc-drafts/regenerate.sh`/`manifest_gen.py` — flagged here so the M-1006
   ladder's driver (however it's built) inherits the same discipline rather than rediscovering it.

### §8.10 M-1006 phase-1: attacking the §8.9 812-gap worklist — transpiler hardening (kickoff `trx2` E-B, epic E33-1) — `Empirical`

The first phase of the **M-1006 whole-corpus rip-through ladder**. Input: §8.9's ranked residual
worklist (812 gaps). Recipe: the `/myc-drafts` 5-step ladder phase (bounded target set → rip → patch
the transpiler → record → feed lessons forward). Target set: the **same 17 wave-1 targets** as §8.9
(phase-1 refines the port-surface pass before the ladder expands beyond it, per M-1006's two-stage
plan). Run as **two disjoint-file leaves** (collision-free by construction — CLAUDE.md §Swarm): a
map-side leaf (`crates/mycelium-transpile/src/map.rs`) and an emit-side leaf (`src/emit.rs`),
octopus-merged and re-vetted with the real `myc check` oracle.

**Baseline note (reconciles with §8.9).** These numbers are measured against a **merged-base
regeneration** (still 812 gaps), not §8.9's `e075c5fb` snapshot directly: the wave-1
declaration-site reserved-word guard (§8.8/M-1001), merged down from `dev` between §8.9 and this
phase, had already reclassified 16 gaps `Other` → `ReservedWord` (`Other` 322 → 306, `ReservedWord`
8 → 24). That is a **taxonomy refinement, not a coverage change** (total 812, `checked_fraction`
unchanged) — recorded so the §8.10 deltas below reconcile with §8.9's table (never-silent, G2).

**Transpiler fixes landed (each grammar-grounded; emission stays `Declared`):**

1. **Concrete generic type-applications now map** (map.rs). A single-segment named generic
   `Head<A, …>` maps to `Head[A, …]` via `type_args` (`docs/spec/grammar/mycelium.ebnf` §`base_type`
   line 258 + `type_args` line 265, RFC-0037 D1 — square brackets), **only** when every angle arg is
   itself a mappable *type* (recursing through the guarded `map_type`, so nested applications like
   `Result<Option<u32>, E>` → `Result[Option[Binary{32}], E]` work). Lifetime/const-generic/
   associated-binding args, qualified multi-segment paths, unmappable args, and reserved-word heads
   all **stay gapped** (never a partial emission; VR-5). A deliberate honesty refinement over the
   naive design: an arg that itself gaps **propagates that arg's own precise `GapReason`** (so
   `Option<String>` gaps as the `String`-has-no-base_type reason, not a blanket `GenericBound`) —
   matching the existing tuple-arm precedent and naming the *real* blocker (G2).
2. **Three grammar-grounded expression literal arms** (emit.rs), each never-silent: **string →
   `StrLit`** (§`literal`/`StrLit`; re-escapes into Mycelium's escape set, gaps a control char with
   no Mycelium escape rather than leaking a raw byte), **float → `FloatLit`** (ADR-040 §2.4: a
   literal is a conversion boundary — a non-finite `1e999` **refuses**, never silently ±inf; Rust-only
   shapes like `2.` gap rather than reshape), and **array → `ListLit`** (`[x; N]` repeat gaps
   explicitly — `ListLit` has no repeat form).
3. **Sharpened `MultiStmtBody` diagnostics** (emit.rs) — a rejected block body now names the
   offending statement kind (nested item / macro-statement / value-discarding stmt-expr) instead of a
   generic reason.

**Measured before → after (`Empirical`, union over all 17 targets, non-test denominator):**

| Metric | Before (merged base) | After | Δ |
|---|---:|---:|---:|
| `expressible_fraction` (emitted / 759) | 6.06% (46) | **6.19% (47)** | **+1 item** |
| `checked_fraction` (myc-check-clean / 759) | 3.69% (28) | 3.69% (28) | flat |
| `GenericBound` gaps | 59 | **46** | **−13** |
| `Other` gaps | 306 | 315 | +9 |
| `MultiStmtBody` gaps | 4 | 6 | +2 |
| `ReservedWord` gaps | 24 | 25 | +1 |
| total gaps | 812 | **811** | −1 |

The one newly-emitted item is `std-io/src/io.rs::read_all`, unblocked purely by the nested-generic
mapping (`Result<Vec<u8>, IoError>` → `Result[Vec[Binary{8}], IoError]`). **`checked_fraction` is
flat** because that item emits but is not yet `myc check`-clean (a downstream name-resolution
blocker, a different class). The `GenericBound −13` is the honest transpiler win; the **`Other +9` /
`MultiStmtBody +2` is the expected never-silent cascade** — once a signature's *type* maps, the item
stops masking on the type and surfaces its *deeper* real blocker (multi-statement bodies, field
access), which is exactly the gap-profiling instrument doing its job (the item still gaps, but now
names the true blocker). The string/float/array arms produce **zero corpus delta** — those literals
appear in the corpus only nested inside constructs that gap earlier, so the closes are
*correct-but-currently-unreached*; they remove **future** false gaps (proven by fixtures + synthetic
demos) and become live if the type-side gaps are ever closed.

**Residual gap-class worklist enumerated + out-of-scope declaration (the M-1006 DoD — the stopping
point recorded, never silent, G2).** The transpiler-fixable surface on the current corpus is now
substantially exhausted; the dominant residue is **language-surface design, not transpiler
boilerplate** — confirming §8.8/§8.9's M-991 verdict a third time:

- **Type-coverage scalars** (`Other`, signed integers / `String`·`str` / `char` / `isize`·`usize` /
  `f32`·`f64` / unit) — **out-of-scope for the transpiler**; each needs a kernel/grammar repr
  decision (E18-1 `needs-design`), not an emitter change. Mapping any onto an existing arm would
  misrepresent semantics (VR-5).
- **Named-field structs (`Struct`, 80) + named-field variants (subset of `PayloadVariant`)** —
  **KEEP GAPPED** (grounded design decision, this phase): the grammar's `constructor` is
  positional-only *and there is no value field-projection surface at all* (the only field reference
  in the whole grammar is `object_item`'s `via Int :` by-index delegation, line 192; `self.0` tuple
  projection is itself gapped). Emitting a named-field struct as a positional constructor would drop
  semantically-meaningful names **and** leave every `foo.a`/`self.mode` body access with no surface
  to rewrite to (14 field-access + 1 struct-literal gaps would remain in the committed corpus) — a
  lossy `Declared` transform producing an un-usable draft. Closing it is record/product-type design
  (E18-1), consistent with §8.9's "language design, not a transpiler defect" label. *(A draft-only
  positional skeleton with field names preserved as doc-comments is a possible future behind explicit
  maintainer sign-off — deliberately not implemented, G2: recorded, not silently done.)*
- **`Import` (117)** — correctly gapped (M-1001): no cross-nodule symbol table to confirm
  resolution; a resolution concern for port/differential time, not transpiler emission.
- **`GenericBound` residual (46)** — bounded generics, impl-block generic params (`impl_item` has no
  type-params surface), and lifetimes: the §8.3/§8.5 design-open backlog.
- **`DeriveAttr` (42)** — Rust built-in derives (`Debug`/`Clone`/…) have **no** Mycelium `lower` rule
  for `derive_item` (line 204) to resolve `derive Name for T` against, so mapping them would emit
  un-`myc check`-able text. Out-of-scope.
- **`MacroInvocation` (6)** — no macro system in the grammar. **`ReservedWord` (25)** — correctly
  gapped. **`TestItem` (33)** — out of scope by design (excluded from the denominator).

**Flagged (never-silent, no change made):** `1f64` is classified by `syn` as `Lit::Int` (suffix
stripped) and emits as Mycelium `Int 1` — a pre-existing float→int infidelity uniform with the
existing Int-arm suffix-dropping; flagged for a maintainer note if float-literal fidelity matters,
not silently special-cased.

**Lessons (feed the next ladder phase).** (1) The type-application close is the model for the honest
phase yield: a whole gap *sub-class* removed, unblocking nested-generic signatures corpus-wide, at
the cost of surfacing the next-layer blockers (a net-positive information gain, flat `checked`).
(2) The current-corpus transpiler-fixable surface is near-exhausted — the M-1006 ladder's *coverage*
growth now depends on either expanding the target set beyond the port surface (later phases) or on
E18-1 language-surface design closing the scalar/record classes; the transpiler alone cannot move
`checked_fraction` much further on this corpus. Recorded so the next phase is scoped to that reality,
not to a false "keep closing gaps" expectation.

**Guarantee tags unchanged:** emission `Declared`; vet verdict `Empirical` (real `myc check`).
**Status unchanged (Draft)** — a ladder phase, enacts nothing further.

### §8.11 M-1006 phase-1 (cont.): shared-reference type erasure — transpiler hardening (kickoff `trx2` E-B, epic E33-1) — `Empirical`

The next transpiler-hardening increment of the **M-1006 ladder**, continuing §8.10's phase-1 pass
(append-only — §8.10 already landed on the working tier; this extends it, it does not rewrite it).
Same recipe (the `/myc-drafts` ladder phase), **same 17 wave-1 targets** as §8.9/§8.10 (phase-1
refines the port-surface pass before the ladder expands the target set). Its **before-baseline is
exactly §8.10's after-state** — 759 non-test items, 47 emitted (6.19%), 28 myc-check-clean (3.69%),
811 gaps — so the deltas below chain cleanly onto §8.10 (reconfirmed by a fresh regeneration at the
merged base).

**Gap class attacked — the largest tractable sub-slice of `Other` (type-coverage).** §8.9/§8.10
rank `Other` #1 (315 gaps), but `Other` is a grab-bag. Profiling the committed `.gap.json` corpus
resolved it: **160 of the 315 `Other` gaps (~51%) are "unsupported Rust type form", and 156 of those
are reference types `&T`** (150 shared `&T`/`&'a T`, 6 mutable `&mut T`) — a single, well-defined,
grounded sub-slice, far more concrete than the scalar type-coverage residue (`String`/signed-int/
`char`) that §8.10 declared out-of-scope for the transpiler (each of those needs a kernel/grammar
repr decision — E18-1). References, by contrast, have a **faithful** mapping already precedented in
the emitter.

**Transpiler fix landed (grammar-grounded; emission stays `Declared`).** One arm in `map.rs`
(`Type::Reference`): a **shared** reference `&T` / `&'a T` **erases** to its referent's mapping
(`map_type(&r.elem)`). Grounding: Mycelium is value-semantic (ADR-003 — no reference types; the
grammar's `base_type`/`type_ref`, `docs/spec/grammar/mycelium.ebnf` §`base_type`, has no `&` form),
so a shared borrow and the value it borrows denote the *same* `T`. This is the **type-position
analogue of an erasure the emitter already performs**: `emit.rs` erases `&expr` (`Expr::Reference`)
and `&pat` (`Pat::Reference`) reference-transparently, and the hand-port itself renders Rust
`fn cmp(&self, other: &Ordering)` as value params `fn cmp(a: Ordering, b: Ordering)`
(`lib/std/cmp.myc`) — so `&T` → `T` is exactly how a human port writes it, not a guess. The lifetime
is erased with the reference (no grammar surface). A referent that itself has no mapping still gaps,
propagating its **own** precise reason (`&str` → `str` gap, `&[u8]` → slice gap) — never a partial
emission (VR-5/G2). A **mutable** reference `&mut T` is **not** erased — in-place mutation has no
value-semantic correspondence (the same stance the existing `&mut self` receiver gap takes), so it
stays an explicit `Other` gap rather than silently dropping the mutation. Six new unit tests pin the
paths (`shared_ref_params_emit`, `mut_ref_param_gapped`, `shared_ref_to_str_still_gapped`, plus the
`map_type` corpus rows and the `mutable_reference_is_gapped_not_erased` /
`shared_reference_to_unmapped_referent_surfaces_referent_reason` regression guards).

**Measured before → after (`Empirical`, union over all 17 targets, non-test denominator 759):**

| Metric | Before (= §8.10 after) | After | Δ |
|---|---:|---:|---:|
| `expressible_fraction` (emitted) | 6.19% (47) | **6.46% (49)** | **+2 items** |
| `checked_fraction` (myc-check-clean) | 3.69% (28) | 3.69% (28) | flat |
| `Other` gaps | 315 | **301** | **−14** |
| `ReservedWord` gaps | 25 | 33 | +8 |
| `MultiStmtBody` gaps | 6 | 10 | +4 |
| `DeriveAttr` gaps | 42 | 43 | +1 |
| total gaps | 811 | **810** | −1 |

The two newly-emitted items are `std-content/lib.rs::digest_eq` (Rust `fn digest_eq(a: &ContentHash,
b: &ContentHash) -> bool`, now emitting `fn digest_eq(a: ContentHash, b: ContentHash) => Bool = …`)
and `std-sys/fs.rs::exists` (`&Path` erased). **`checked_fraction` is flat** because both emit
faithfully yet fail `myc check` on a **name-resolution** blocker — `unknown type ContentHash` /
`unknown type Path` (the referent types are declared in *other* nodules the single-file draft cannot
see) — a *different* blocker class than emission, exactly as §8.10 §Lesson-2 and §8.7's original
thesis predicted. The `Other −14` is the honest transpiler win; the **`ReservedWord +8` /
`MultiStmtBody +4` / `DeriveAttr +1` is the expected never-silent cascade** — once a `&T` stops
masking a signature, the item surfaces its *deeper* real blocker (a reserved-word type/ctor, a
multi-statement body, a dropped derive), which is the gap-profiling instrument doing its job (the
item still gaps, but now names the true blocker, refining the ranked worklist). Of the ~150 shared
references, only these ~14 had the reference as their item's *sole* `Other`-class blocker; the rest
sit in items with a further `Other`-class blocker (an unmappable `&str`/`&[u8]`/`&dyn T` referent, or
another unmapped param), so erasure reclassifies within `Other` rather than removing the item's gap —
honest, and the reason the net `Other` move is modest.

**Lesson (feeds the next ladder phase).** This is a **small but honest** win, and it **confirms
§8.10's near-exhaustion thesis a fourth time**: the largest remaining well-defined transpiler-fixable
sub-slice (references) closes cleanly and faithfully, yet moves `checked_fraction` by **zero** —
because the check-clean ceiling on this fixed corpus is gated by **name-resolution** (single-file
drafts referencing types/fns declared in sibling nodules) and **language-surface design**, not by the
transpiler's emission surface. There is no transpiler-only change that moves `checked_fraction` on
this corpus; the value delivered is a *refined gap profile* (references removed as a masking blocker
corpus-wide, the deeper blockers now ranked) and *more portable drafts* (references erased the way a
hand-port writes them). The M-1006 ladder's *checked* growth must come from **expanding the target
set** (later phases — cross-nodule project-mode vetting would resolve the `ContentHash`/`Path`-class
name errors) or from **E18-1 language-surface design**, not from further transpiler emission arms on
this fixed 17-target set. Recorded so the next phase is scoped to that reality (G2).

**Guarantee tags unchanged:** emission `Declared`; vet verdict `Empirical` (real `myc check`).
**Status unchanged (Draft)** — a ladder phase, enacts nothing further.

---

## Meta — changelog

- **2026-06-25 — Created (Draft, advisory).** Captures the **Rust→Mycelium transpiler** strategy for
  accelerating the Mycelium self-hosting rewrite, seeded from the maintainer's **py2rust** (AST-walk
  transpilation + never-silent compatibility analysis) and **py-rust-bridge** (FFI/SFI interop)
  projects. Records the construct-mapping sketch (Rust → Mycelium, incl. reusing the MEM-4 ownership
  analysis), the **flag-don't-guess** analyzer as the load-bearing G2 principle, the phasing
  (isolated branch → incremental interop-bridged transpile-then-refine → differential verify →
  DN-27 component-repo decomposition), and the §6 open questions. **Gated** on the Mycelium surface
  being a viable target — **enacts nothing, ships no code, begins no phase.** All Mycelium-specific
  effort/coverage claims `Declared`; seed-architecture `Empirical`, its transfer `Declared`. Feeds
  DN-26 / DN-27 / RFC-0028 / M-502. (Append-only; VR-5; G2.)
- **2026-06-25 — §3 correction (Draft amendment; alignment audit).** Fixed a category error in the
  ownership row + §3 closing paragraph: Rust ownership/borrow facts must come from a **rustc/
  rust-analyzer front-end** (authoritative = rustc MIR `mir_borrowck`); `syn` is syntax-only and
  **MEM-4 (`mycelium-mir-passes`) is *not* the transpiler's ownership analyzer** — it is a downstream
  RC-insertion/elision optimizer over Mycelium **Core IR** (intraprocedural / straight-line /
  non-escaping; recursion refused). MEM-4 is reframed as an *output-optimization* asset, not an
  *input-analysis* one. Also annotated the §3 fn/closures + `Result` rows with real status: RFC-0024
  is **Proposed / pending-ratification** (only named fns-as-value; capturing closures auto-`Impossible`,
  flagged) and the `?` operator is **absent from the v0 grammar** (lower to explicit `match`). §6 Q1
  and §7 corpus echoes corrected in lockstep. Status unchanged (**Draft**); enacts nothing. (Append-only;
  VR-5; G2.)
- **2026-07-01 — §8 added: PoC results (M-873, kickoff `trx`).** Records the first **code** spike —
  `crates/mycelium-transpile` (syn-based, exhaustive-dispatch, never-silent gap report) run on
  `mycelium-std-cmp` and diffed against `lib/std/cmp.myc`. Measured (`Empirical`): **3.6%** of the
  crate expressible against the current surface *without* macro expansion (the §6-Q6 auto-conversion
  fraction, a lower bound); the **prioritized surface-feature backlog** (§8.3 — macros ~55%, then
  conversion-op bodies / traits / bounded-generics / payload-variants / structs) as the E18-1
  `needs-design` demand data; and a **~0.85–0.95M-token** fully-loaded cost, at/below §5a's `Declared`
  "first spike" estimate. §8.1 sharpens the §2 seed posture with the measured specifics (the seed's
  analyzer is a silent-pass allowlist — the anti-pattern; the PoC built the flag-don't-guess layer on
  `syn` instead). §8.2 logs a review fix where 12 fabricated `from(self)` bodies were reclassified to
  gaps (G2/VR-5 — never emit plausible-but-wrong). **Status unchanged (Draft, advisory)** — a spike,
  not the gated full phase (§5); enacts nothing further. (Append-only; VR-5; G2.)
- **2026-07-01 — §8.2/§8.5/§8.6 extended: hardening follow-on (M-873).** DN-41 `width_cast` faithful
  conversion emission (std-cmp 3.6%→**12.6%**; real prim, not fabricated — §8.2 follow-on note),
  directory/batch CLI mode, and the **union gap-report across 6 core-lib crates** (grand union
  **12.4%**; §8.5 re-ranks the backlog — unsupported *types* #1 at 36%, macros #2 at 22%). §8.6 records
  the grounded finding that `std.option`/`std.result` have **no Rust source** (self-hosted, M-715/M-649
  — excluded, not substituted; VR-5/G2). All numbers `Empirical` (measured over the run). **Status
  unchanged (Draft)** — still a spike; the type-coverage + macro-expansion levers are E18-1
  `needs-design`. (Append-only; VR-5; G2.)
- **2026-07-06 — §8.7 added: the transpile → `myc check` vet loop (M-1000, kickoff `trx2`).** Closes the
  loop `expressible_fraction` left open — a `--vet` mode + `src/vet.rs` + `scripts/checks/transpile-vet.sh`
  run the **real** `myc check` oracle over every emitted `.myc` and report **`checked_fraction`**
  (myc-check-clean, file-gated, denominator = non-test items — stated) alongside the emission-only
  figure. Records the `Empirical` baseline it exposes: `checked_fraction` is **0.0% on every
  representative target** (semcore `eval`/`fuse`, stdlib `std-time`/`std-rand`, the `std-cmp` pilot) —
  each poisoned by an unresolved `use` and/or a reserved-word/undefined-trait emission that fails the
  toolchain even though `expressible_fraction` counted it. Re-ranks the backlog for *checked* coverage
  (the poison classes, not §8.3's emission-blocking macros) as M-1001's demand data. Vet verdict
  `Empirical`; emission stays `Declared`. **Status unchanged (Draft)** — a spike, enacts nothing.
  (Append-only; VR-5; G2.)
- **2026-07-06 — §8.8 added: closed the top vet-blocking gap classes + M-991 go/no-go (M-1001, kickoff
  `trx2`).** Acts on §8.7's re-ranking: `dispatch_use` now **gaps** every `use` (`Category::Import` — no
  cross-nodule symbol table to confirm resolution; it was the universal check-poison), and a reserved-word
  guard (`src/reserved.rs`, a drift-guarded snapshot of `mycelium-l1`'s lexer keywords) **gaps** any Rust
  identifier that collides with a Mycelium keyword (`Category::ReservedWord` — it would fail to parse) —
  both honest flag-don't-guess corrections, not new emissions. Measured before → after `checked_fraction`
  (`Empirical`): the **semcore** `eval` **0.0% → 2.4%** and the **stdlib** `std-time` **0.0% → 8.1%**
  (both now myc-check-**Clean**), with `expressible_fraction` falling where it over-counted and the two
  coinciding on clean files. Records the ranked residual worklist (673 gaps: `Other`/type-coverage 274,
  external-trait `Impl` 93, `Struct` 70, …) and the **M-991 go/no-go**: **NO-GO** as a bulk semcore
  transpiler (direct `checked_fraction` ~0–2% — the residue is language-surface design, not boilerplate),
  **GO** as a never-silent, real-toolchain-vetted gap-profiling instrument. Emission `Declared`, vet
  `Empirical`. **Status unchanged (Draft)** — a spike, enacts nothing. (Append-only; VR-5; G2.)
- **2026-07-06 — §8.9 added: the wave-1 rip-through over the full port surface (M-1002/M-1003,
  kickoff `trx2` E-B, epic E33-1).** Runs the M-1000 vet loop over **all 17** maintainer-confirmed
  wave-1 targets (the 5-file semcore SCC + all 12 unported stdlib crates — not the §8.7/§8.8 5-file
  sample) into a new greenfield staging tree `gen/myc-drafts/` (outside `lib/`, never dogfood-gated,
  never imported — drafts graduate into `lib/` only when hand-vetted during M-993), with a
  deterministic manifest (`gen/myc-drafts/{MANIFEST.md,manifest.json}`, regenerated by
  `gen/myc-drafts/regenerate.sh`; verified byte-identical across two runs at the same commit,
  full-tree `sha256sum` over all 158 generated files). Measured (`Empirical`): union
  `checked_fraction` **3.7%** (28/759 non-test items), `expressible_fraction` **6.1%** (46/759);
  `eval.rs` (2.4%) and `std-time` (8.1%) exactly reproduce §8.8's post-M-1001 numbers
  (cross-validation); 7 of 17 targets are an honest **0.0%** `checked_fraction` (including
  `std-conformance`'s honest `0/0` — a test-only crate with no library surface); zero silent holes,
  zero hard transpile-parse failures across all 17 targets. Confirms — does not overturn — E-A's
  M-991 go/no-go (§8.8) at full wave-1 scope: real port-surface `checked_fraction` runs **~0–8%**,
  and the residual gap-class ranking (`Other`/type-coverage 322/812 ≈ 40%, `Impl` 119, `Import` 117,
  `Struct` 80, `GenericBound` 59, …) reproduces §8.8's ranking at 3× the corpus size — the M-1006
  ladder's phase-1 input. Records 5 lessons (type-coverage is still the top lever; checked and
  expressible fractions diverge independently per target; an honest `0/0` crate is expected, not an
  error; destination-convention metadata — semcore's single-nodule merge, FLAG-ast-5/FLAG-parse-2 —
  must be carried alongside the transpile output, not derived from it; and two real driver hazards
  — an absolute-path leak into `vet.json`, and a crash-on-missing-input — caught and fixed during
  this wave, now the standing shape of the driver for M-1006 to inherit). Emission `Declared`, vet
  `Empirical`. **Status unchanged (Draft)** — a spike, enacts nothing further. (Append-only; VR-5;
  G2.)
- **2026-07-06 — §8.10 added: M-1006 phase-1 transpiler hardening against the §8.9 worklist
  (kickoff `trx2` E-B, epic E33-1).** First ladder phase over the same 17 wave-1 targets, run as two
  disjoint-file leaves (map-side + emit-side, octopus-merged). Landed three grammar-grounded fixes:
  concrete generic type-applications now map to `type_args` (`Head<A,…>` → `Head[A,…]`, recursive,
  never-partial); string/float/array expression literal arms (`StrLit`/`FloatLit`/`ListLit`, each
  never-silent — non-finite floats and un-escapable control chars refuse rather than emit garbage);
  and sharpened `MultiStmtBody` diagnostics. Measured (`Empirical`): union `expressible_fraction`
  6.06% → 6.19% (46 → 47 emitted; `std-io::read_all` unblocked via a nested
  `Result[Vec[Binary{8}], IoError]`), `checked_fraction` flat at 3.69% (the new item emits but is not
  yet myc-check-clean), `GenericBound` 59 → 46 (−13) with an honest `Other`/`MultiStmtBody` cascade
  (deeper real blockers surface once the masking type gap is closed). Records the M-1006-DoD residual
  enumeration + out-of-scope declaration: type-coverage scalars, named-field structs/variants (KEEP
  GAPPED — no field-projection surface; a grounded design decision), `Import`, bounded `GenericBound`,
  and Rust built-in `DeriveAttr` are all language-surface design (E18-1), not transpiler defects —
  the current-corpus transpiler-fixable surface is near-exhausted (the stopping point recorded, G2).
  Emission `Declared`, vet `Empirical`. **Status unchanged (Draft)** — a ladder phase, enacts nothing
  further. (Append-only; VR-5; G2.)
- **2026-07-07 — §8.11 added: M-1006 phase-1 (cont.) — shared-reference type erasure (kickoff
  `trx2` E-B, epic E33-1).** The next transpiler-hardening increment on the same 17 wave-1 targets,
  continuing §8.10 (append-only). Profiling resolved `Other`'s grab-bag: 156 of its 315 gaps are
  reference types `&T` — the largest well-defined tractable sub-slice. Landed one `map.rs` arm: a
  **shared** reference `&T`/`&'a T` **erases** to its referent's mapping (value semantics, ADR-003 —
  no `&` in the grammar; the type-position twin of the emitter's existing `&expr`/`&pat` erasure, and
  how the hand-port writes Rust `&Ordering` as value `Ordering`, `lib/std/cmp.myc`); a **mutable**
  `&mut T` stays an explicit gap (mutation has no value-semantic correspondence — the `&mut self`
  stance); an unmappable referent still gaps with its own precise reason (never a partial emission).
  Six new unit tests. Measured (`Empirical`, union / 759): `expressible_fraction` 6.19% → **6.46%**
  (47 → 49; `digest_eq` via `&ContentHash`, `exists` via `&Path`), `checked_fraction` **flat** at
  3.69% (both new items emit faithfully but fail `myc check` on a name-resolution blocker — `unknown
  type ContentHash`/`Path`, the referents live in sibling nodules — a different class than emission),
  `Other` 315 → **301** (−14) with the expected never-silent cascade (`ReservedWord` +8, `MultiStmtBody`
  +4, `DeriveAttr` +1 — deeper real blockers surface once a `&T` stops masking the signature); total
  gaps 811 → 810. Lesson: a **small but honest** win that **confirms §8.10's near-exhaustion thesis a
  fourth time** — the check-clean ceiling on this fixed corpus is gated by name-resolution and
  language-surface design, not transpiler emission, so `checked` growth needs target-set expansion
  (cross-nodule project-mode vetting) or E18-1, not further emission arms. Emission `Declared`, vet
  `Empirical`. **Status unchanged (Draft)** — a ladder phase, enacts nothing further. (Append-only;
  VR-5; G2.)
