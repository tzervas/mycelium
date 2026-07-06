# Design Note DN-34 — Rust→Mycelium Transpiler Strategy (Self-Hosting Acceleration)

| Field | Value |
|---|---|
| **Note** | DN-34 |
| **Status** | **Draft (advisory)** (2026-06-25) — strategy capture for a **future** phase. Records how a Rust→Mycelium transpiler would accelerate the Mycelium self-hosting rewrite (the `stdlib-and-libraries-in-Mycelium` long pole), leveraging the maintainer's existing **py2rust** + **py-rust-bridge** projects as the seed. **Enacts nothing; ships no code.** The phase is **gated on the Mycelium surface (L1/L2/L3 + stdlib API) being mature enough to be a transpilation *target*** — it is not begun now. |
| **Feeds** | **DN-26** (Self-Hosting Bootstrap Plan — this is the *mechanism* that does the bulk of the rewrite), **DN-27** (Post-1.0.0 Repository Decomposition — the transpiled output is split into component repos), **RFC-0028** (FFI & System Interface — the Rust↔Mycelium interop the transition relies on), **M-502** (stdlib-in-Mycelium migration), **ADR-021/022 + DN-25** (the 1.0.0 gates that schedule self-hosting post-core-1.0). |
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
