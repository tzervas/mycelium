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
