# DN-85 — Multi-Language Transpiler Front-Ends (Python · TypeScript · Java)

| Field | Value |
|---|---|
| **Status** | **Draft** — strategy note; decides nothing normatively (feeds a future epic + RFC/ADR) |
| **Task** | proposed **E35** (front-end abstraction) + per-language epics; ids **proposed, not minted** (mitigation #1) |
| **Related** | DN-34 (Rust→Mycelium transpiler strategy — the parent), M-991/M-1000/M-1006 (the trx2 ladder), `rwr` (the Phase-II rewrite this feeds), DN-26 (self-hosting) |
| **Grounding** | `crates/mycelium-transpile/**` (the current architecture), DN-34 §8.7–§8.10 (the M-991 gap-profiling-instrument verdict, `Empirical`) |
| **Guarantee** | `Empirical`/`Declared` per-claim; the whole note is `Declared` (a proposal) until an epic + RFC ratify it |

> **The maintainer's goal (2026-07-06).** Extend the transpiler to ingest **Python first**, then
> **TypeScript** and **Java**, alongside the current Rust front-end — an accelerated path to port the
> Python **math / scientific / ML-AI** ecosystem into Mycelium, filling the residual gaps after
> transpilation, and **refining the transpiler per-module** (the trx2 method) until it is
> "disturbingly reliable, highly-polished transpilation that minimizes follow-on cleanup."
>
> This note captures that goal, grounds the architecture, and — per house rule #4 — states the **one
> boundary the vision must design around** honestly, up front (§4.2), rather than affirming it whole.

## 1 · Why this is tractable — the backend is already nearly language-agnostic

The current transpiler (`mycelium-transpile`, DN-34) is a **front-end + backend** pipeline, and the
coupling to Rust's `syn` parser is concentrated, not pervasive (`Empirical`, grep of the crate):

- **Front-end (Rust-specific):** `transpile.rs` (item dispatch over `syn::Item`, 16 `syn::` sites),
  `emit.rs` (construct → `.myc` text, 10 sites), `map.rs` (type mapping, 2 sites).
- **Backend (already language-neutral):** `gap.rs` (the never-silent `GapReport` taxonomy — its
  categories are *about the Mycelium target surface*, not the Rust source), and `vet.rs` (the
  `myc check` vet loop — measures `checked_fraction` against the real toolchain, source-agnostic).

So a multi-language transpiler is a **front-end abstraction**, not a rewrite: define a small
**normalized source IR** (the subset of constructs the emitter already understands — items, types,
expressions, patterns, blocks) that each language front-end (`syn` for Rust; `ast`/RustPython/
tree-sitter for Python; the TS compiler API/tree-sitter for TS; JavaParser/tree-sitter for Java)
lowers into, and keep `gap.rs` + `vet.rs` + the `.myc` emitter **shared**. Each front-end owns only
*parse + lower-to-IR*; the gap-profiling + vet + emit machinery is reused verbatim. **The trx2
M-1006 ladder (per-module rip → vet → patch the transpiler → record) transfers directly** — it is
the correct method here and needs no reinvention.

## 2 · Language sequence and the honest ordering trade-off

The maintainer's sequence is **Python → TypeScript → Java** (driven by *value*: the Python sci/ML
corpus is the prize). Worth stating the *technical* ordering honestly (`Declared`):

- **TypeScript and Java are statically typed** (like Rust) — their front-ends map declared types the
  way `map.rs` maps `syn` types today, so they are the **more tractable** front-ends and the natural
  way to *prove the front-end abstraction on easy ground*.
- **Python is dynamically typed** — the **hardest** front-end (§4.1) but the highest *value*.

The tension is real: **Python-first maximizes value but front-loads the hardest front-end.** A
defensible hybrid, offered for the maintainer's decision (not decided here): stand up the **IR + a
second static front-end (TS or Java)** first to *validate the abstraction cheaply*, then bring the
Python front-end with the type-inference machinery §4.1 needs. Either way Python is in scope; this is
only about which front-end *de-risks the refactor*.

## 3 · The method — the trx2 ladder, per language, per module

Unchanged from DN-34 §8 / M-1006 / the `/myc-drafts` + `/transpile-vet` skills:

1. **Front-end lowers** a source module to the normalized IR (parse + construct-map, never-silent gaps).
2. **Emit + vet:** the shared backend emits `.myc`, `myc check`-vets each emission, classifies every
   failure back into `gap.rs` (so accuracy is measured against the *real toolchain*, not "text emitted").
3. **Patch the transpiler** on the phase's top *transpiler-fixable* gap classes; record before/after
   `expressible_fraction` vs `checked_fraction`.
4. **Record** a per-phase manifest + a DN-34-style §8.x append; lessons feed the next phase.

Emission stays **`Declared`** until a differential vs the source oracle upgrades it (`Empirical`) — the
corpus honesty contract (`gen/myc-drafts/`) binds every language, not just Rust.

## 4 · The boundaries the vision must design around (house rule #4 — stated, not softened)

### 4.1 Python's dynamic typing lowers the initial ceiling (`Empirical`, by analogy to DN-34 §8.10)

Rust/TS/Java hand the transpiler declared types; Python does not (type hints are optional and
partial). A Python front-end must lean on hints where present, **infer** where it can, and **gap**
(never guess — VR-5) where it cannot. Consequence, predicted from the DN-34 §8.10 finding that
type-coverage is already the #1 gap class *even with Rust's explicit types*: **Python's initial
`checked_fraction` will start *lower* than Rust's**, and closing it is partly type-inference
engineering and partly Mycelium language-surface design (not a quick emitter fix).

### 4.2 The C/CUDA library cores do NOT transpile — the load-bearing correction

**This is the boundary the "port all the Python math/sci/ML libraries" goal must be built around.**
The *value* of numpy / scipy / pandas / pytorch / tensorflow is in their **compiled cores** —
numpy's ndarray + ufuncs are **C**; scipy is **C/Fortran**; pandas is **C/Cython**; pytorch/TF are
**C++/CUDA (ATen/cuDNN)**. **That code is not Python source and cannot be transpiled** — the Python
the transpiler ingests only *calls into* those extensions.

So transpiling these libraries yields the **pure-Python layer** (public API shells, argument
validation, dtype/shape bookkeeping, pure-Python algorithms, orchestration/glue) and **gaps every
call into the compiled core** (never-silent — the gap report says exactly which). "Feed numpy in, get
Mycelium-numpy out" is therefore **not** achievable by transpilation alone. The honest, and still
very valuable, reframe is a **two-part port**:

1. **Transpile the Python layer** (the accelerated path this note is about) — real leverage on the
   API surface + pure-Python code.
2. **Back the compute with Mycelium-native kernels or an FFI bridge.** Mycelium already *has* the
   native home for this — `binary/ternary/dense/VSA` value semantics, `mycelium-dense`/
   `mycelium-vsa`/`mycelium-numerics`, and the MLIR→LLVM AOT with native Dense/VSA codegen
   (RFC-0039). The ported APIs map onto **Mycelium's own compute stack**; during transition, `wild`/
   `@std-sys` FFI can bridge to the C libs (ADR-014/RFC-0028). This is a **major, separate
   engineering track**, not transpiler output — and pretending otherwise would be exactly the
   "plausible but wrong" emission VR-5/G2 forbid.

**Net:** the transpiler accelerates the *Python-level* port; the *compute* is Mycelium-native work.
Both are worth doing; only the first is "transpilation."

### 4.3 "Minimize all follow-on work" is bounded by Mycelium surface coverage, not transpiler polish

The trx2 verdict (M-991, DN-34 §8.8–§8.10, `Empirical`) is the standing evidence: the residual after
transpilation is frequently **language-surface design** (no *confirmed* Mycelium surface for the
source construct — record types, certain scalars, imports), **not** a transpiler defect. Refining the
transpiler closes the **transpiler-fixable** frontier; it **cannot** close a design-residual (that
needs Mycelium language design or a native reimplementation). So the achievable north star is
precisely stated: **push the myc-check-clean `checked_fraction` as high as the confirmed Mycelium
surface allows, and cleanly delineate the design-residual** — *not* "zero follow-on work," which
would over-claim past the surface (VR-5). "Disturbingly reliable" is a real target **for the covered
subset**; the ladder's honesty is that it never hides the uncovered remainder.

## 5 · Running the toolchain against the self-hosted `.myc` codebase (maintainer goal, 2026-07-06)

Today the transpiler, the VSA checks, and the toolchain run against the **Rust** reference codebase —
correct for now. A **key future enablement**: once `boot10` self-hosting matures (the `.myc` L1
frontend + semcore SCC, M-993/M-740/M-742), run the toolchain — the `myc check` vet loop first, then
the broader checks/experiments — **through the self-hosted `.myc` compiler**, not the Rust one, so
the multi-language transpiler is vetted by the dogfooded toolchain (the `/myc-dogfood` dual-witness
extended from "both accept the frontend" to "the `.myc` toolchain *is* the checker"). This is the
self-hosting endgame and is **owned by the `boot10`/DN-26 track** — recorded here as the transpiler's
dependency on it, to be minted as a self-hosting milestone there (not duplicated). Until then the
Rust toolchain is the trusted base (DN-26); the switch is gated on self-hosting parity, never silent.

## 6 · Proposed shape (ids PROPOSED, not minted — mitigation #1)

- **E35 — Transpiler front-end abstraction:** extract the normalized source IR; refactor the Rust
  front-end onto it (behaviour-preserving — the trx2 corpus numbers must not regress); keep
  `gap.rs`/`vet.rs`/emit shared. **The de-risking first step** (no new language yet).
- **E36 — Python front-end + math/sci/ML port ladder:** the `ast`/RustPython/tree-sitter front-end +
  type-inference; then the M-1006-style per-module ladder over a chosen corpus (start pure-Python:
  e.g. a small typed-hint-rich library), with §4.2's two-part boundary explicit in every manifest.
- **E37 / E38 — TypeScript / Java front-ends:** the typed front-ends (§2) — candidates to precede E36
  to validate the abstraction.
- Cross-cutting: the §5 self-hosted-`.myc` toolchain switch → a `boot10`/DN-26 milestone.

Sequencing: **after** the current trx2 M-1006 ladder + `boot10` have matured (this is `rwr`-adjacent,
Phase-II-ish); the front-end abstraction (E35) could begin earlier as pure refactoring once the trx2
corpus is a stable regression baseline.

## 7 · Definition of Done (for the note → its epic)

- A ratified **front-end IR** + the Rust front-end refactored onto it with **no regression** in the
  trx2 corpus `checked_fraction`/`expressible_fraction` (the DN-34 numbers are the baseline).
- At least one **non-Rust** front-end landing per-module ladder phases with recorded before/after
  metrics and a DN-34-style §8.x record, emission `Declared`/vet `Empirical`.
- §4.2's two-part boundary is **operationalized**: every library-port manifest states, per module,
  what transpiled (Python layer) vs what is gapped-to-native/FFI (compute core) — never a silent
  "ported" claim over a C/CUDA core (G2).
- The §5 self-hosted-`.myc` switch has a checked parity gate before it flips (owned by `boot10`).

## Meta — changelog

- **2026-07-06 — Created (Draft).** Captures the maintainer's multi-language-transpiler goal
  (Python→TS→Java front-ends to accelerate porting the Python math/sci/ML corpus), grounds the
  front-end-abstraction architecture in the current `mycelium-transpile` split (backend already
  language-neutral), transfers the trx2 M-1006 ladder as the method, and — per house rule #4 —
  states the three boundaries the vision must design around: Python dynamic typing (§4.1), the
  **C/CUDA library-core boundary** (§4.2, the load-bearing correction — the compiled cores are not
  Python source and do not transpile; the port is two-part: transpile the Python layer + native/FFI
  the compute), and follow-on-work bounded by Mycelium surface coverage (§4.3). Records the
  self-hosted-`.myc` toolchain-switch goal (§5) as a `boot10`/DN-26 dependency. Ids proposed, not
  minted (mitigation #1). Decides nothing normatively. (Append-only; VR-5; G2.)
