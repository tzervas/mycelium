# DN-89 — A Native AI/ML Library Ecosystem for Mycelium (Post-Self-Hosting)

| Field | Value |
|---|---|
| **Note** | DN-89 |
| **Status** | **Proposed** (2026-07-07; planning capture, DN-17/DN-85/DN-88 posture — advisory, decides nothing normatively) |
| **Task** | proposed tracking epic **E41-1** + issue **M-1021**, gated/blocked on the self-hosting capstone (ids **proposed, not minted** — mitigation #1; the orchestrator must verify both slots are still free before minting) |
| **Related** | **DN-85** (Multi-Language Transpilation and the Single-Language Full-Stack Goal — the strategy this note *applies* to the AI/ML domain; DN-85's transpile/bind/open-source-constraint ladder is the direct ancestor of this note's three-mechanism table) · **DN-86** (Multi-Language Transpiler Front-Ends — the Python-first front-end architecture + the load-bearing C/CUDA-core boundary, §4.2, that this note's mechanism-2/3 split operationalizes for AI/ML specifically) · **ADR-042** (Rust-Base Freeze Now, Full Mycelium Self-Hosting Kernel Included by Decomposition — **Accepted**, not yet Enacted; its two-horizon END-STATE — zero foreign first-party languages, kernel and AOT/codegen backend included — is this note's literal gating capstone) · **DN-26** (Self-Hosting Bootstrap Plan — the staged Rust→`.myc` toolchain port whose completion is this note's precondition) · **DN-14** (Self-Hosting Gate — the surface-readiness evidence DN-26 builds on) · **ADR-038** (Pragmatic Dogfooding / Function-First Release Strategy — §2.8's versioning axis, `1.0.0` ≡ "fully rewritten into Mycelium where appropriate and 100% operational," is the terminal state this note gates on) · **DN-88** (Component-Repo Decomposition — the two-stage interpreter→AOT production-ready bar, §3, reused verbatim below as this note's parity criterion; also the "decomposition gate" ADR-042 pins its zero-foreign-languages end-state to) · **RFC-0028** (FFI & system interface — the `wild`/`@std-sys` vocabulary mechanism 2 binds through) |
| **Grounding** | `docs/notes/DN-85-*.md` §4–§5 (transpile-what-you-can + FFI-bind interim strategy; the open-source constraint + provenance ladder) · `docs/notes/DN-86-*.md` §4.2 (the C/CUDA library-core boundary — numpy/scipy/pandas/pytorch/tensorflow's compute cores are not Python source and do not transpile) · `docs/adr/ADR-042-*.md` §2.1–§2.4 + §3 + §6 (the two-horizon freeze/end-state, DN-39 preserved as boundary-vs-implementation-language, the bootstrap/trust `needs-design` question OQ-1, the native-codegen-backend `needs-design` track OQ-5, the wild-free-`.myc`-kernel goal) · `docs/notes/DN-88-*.md` §3 (the two-stage interpreter-runnable → AOT-compiled production-ready gate, reused here) · `docs/adr/ADR-038-*.md` §2.8 (the `1.0.0` terminal-state versioning axis) |
| **Guarantee** | **`Declared`** throughout — a forward plan; nothing here is implemented, enacted, or checked. Every claim is tagged inline where it draws on checked source vs. the maintainer's stated intent (house rule #4). |

> **The maintainer's vision (assembled from several clarifications, captured near-verbatim).** Once
> the full Mycelium project is **fully self-hosted — zero foreign first-party languages, kernel
> included** — begin building a corpus of **Mycelium-native AI/ML libraries** by porting
> industry-standard Python, TypeScript, and Java AI/ML libraries, so users and developers can do
> AI/ML work natively in Mycelium. Three mechanisms, chosen per component, form a **convergence
> path to full-native** rather than three permanent parallel choices: transpile+patch and FFI
> bindings are **bridges**; clean-room reverse-engineering is a **new native implementation**; the
> terminal goal is **full native Mycelium everywhere**, and a bound backend's connected sources are
> migrated toward clean-roomed native components over time, as first-party components of the same
> repo. Ports **leverage and improve** on the originals — never verbatim clones — using Mycelium's
> own value-semantics, never-silent swaps, and guarantee-tag lattice; behavioral equivalence is the
> floor, measurable improvement is the goal, and each claim is tagged at its real strength.
>
> This note records that intent so it is durable and inspectable when the gating capstone closes. It
> is **not** active work, mints no code, and ratifies no mechanism (DN-85/DN-88 posture, carried
> forward).

---

## 1. Status / posture

**Proposed / `Declared`**, advisory. This note **enacts nothing** — no library is ported, no repo is
created, no FFI binding is wired, no reimplementation begins as a result of it. It **supersedes no
existing decision** (append-only, house rule #3): it **builds on** DN-85 (the general
transpile/bind/reimplement ladder), DN-86 (the front-end architecture + the C/CUDA-core boundary),
and ADR-042 (the near-term Rust-base freeze), and it **applies** that machinery to one domain —
AI/ML libraries — while adding the domain-specific pieces those notes left open: a third mechanism
(clean-room reverse-engineering, §3.3/§6), an explicit convergence path from bridge to native (§4),
and a port-quality principle (§5).

**Gating condition — stated precisely against the current ADR-042 (house rule #4: ground the claim,
don't guess it).** ADR-042 (Accepted 2026-07-07) was reframed, in this same session, to record
*exactly* the maintainer's "fully self-hosted, kernel included" bar as its own two-horizon decision —
this note's gate is not an extrapolation past ADR-042, it is a direct read of it:

1. **NOW (already in force).** No new Rust enters the language-project surface; new functionality is
   authored in `.myc` (ADR-042 §2.1(a)). This horizon is not this note's gate — it is already active
   and does not by itself unblock AI/ML porting.
2. **END-STATE (this note's literal gate).** ADR-042 §2.1(b)/§6 OQ-4 commits, as a `Declared` goal,
   to **zero foreign first-party languages by the DN-88 component-repo decomposition gate** — the
   *entire* first-party project, **including the Rust kernel itself** (the deepest, last self-hosting
   step) and, further out, **the AOT/codegen backend** (a native-Mycelium replacement for the current
   MLIR/LLVM path — ADR-042 §3/OQ-5, its own far-future `needs-design` track). The only foreign
   residue ADR-042 admits even at the fully-native terminal is the **irreducible OS/hardware ABI
   seam** (minimal `wild`, §2.4).

**The DN-39 nuance that makes this coherent (ADR-042 §2.2), carried forward here.** DN-39's kernel
*boundary* is the **set** of trusted components {L0 Core IR, reference interpreter, content-addressing
primitive, guarantee lattice, swap engine} — that set is unchanged. What ADR-042's end-state changes
is the **implementation language** of those components (Rust → Mycelium), which "does not move the
boundary" and "does not promote or demote anything." This note's gate is on that *language* becoming
Mycelium throughout, not on the trusted-component set changing.

**Honesty this note keeps, unchanged by the reframe (VR-5).** ADR-042 itself tags the end-state
`Declared`, not proven: **feasibility of a self-hosted trusted kernel is an open, `needs-design`
question** (ADR-042 §3/§6 OQ-1 — what executes and vouches for a `.myc` kernel; candidates named but
not solved: self-AOT-compilation, a bootstrap seed later eliminated, a "Reflections on Trusting
Trust" reflective-trust discharge, or a new trusted base = the `.myc` kernel source + its lowering
path), and the **native-codegen-backend replacement for MLIR/LLVM is its own large, research-grade,
undesigned track** (§6 OQ-5). This note's gate therefore does not presuppose the end-state is
near, easy, or even guaranteed to land exactly as scoped — it gates on ADR-042's end-state being
**reached** (or on ADR-042 reaching `Enacted`, whose own DoD requires the same thing — §5), and it
inherits ADR-042's own honesty posture on how far off that is rather than restating a weaker or
stronger claim of its own.

This note **gates on the ADR-042 end-state, plainly**: work under E41-1/M-1021 does not begin until
the entire first-party project — kernel included — is `.myc`, zero foreign first-party languages
remain at the DN-88 decomposition gate, and the ADR-042 OQ-1 bootstrap/trust story has been designed
and discharged (ADR-042's own `Accepted → Enacted` conditions, §5). This note does **not** additionally
gate on the native-codegen-backend replacement (ADR-042 OQ-5) landing first — that is the *fully-native*
terminal, a further step ADR-042 itself distinguishes from the nearer decomposition-gate state; an
AI/ML porting wave that starts once the decomposition-gate state is reached would still be building on
an AOT path that is, at that point, MLIR/LLVM-backed — acceptable, since interpreter/AOT parity (§8)
is judged against capability, not implementation provenance of the codegen backend.

**Two-stage per-unit readiness bar (reused, not reinvented).** For any *given* AI/ML library port,
"done" additionally means the DN-88 §3 two-stage production-readiness gate, applied per component:
(1) **interpreter-runnable, with a 100% checkout of the interpreted variant** against the Rust/source
reference behavior it targets; only then (2) **AOT-compiled**. This distinguishes *expected*
interpreter-vs-AOT performance differences (the trusted interpreter is definitionally not the fast
path) from *unacceptable* capability disparities the AOT leg must not silently carry (§8).

**This note is Resolved when** the maintainer confirms the gating read above (§1 point 1 vs 2) and
the first concrete AI/ML porting wave is minted as a scoped RFC/ADR + issues under E41-1.

## 2. Goal

Once the gating capstone (§1) closes, build a corpus of **Mycelium-native AI/ML libraries** — ported
from the industry-standard Python, TypeScript, and Java ecosystems (numpy/scipy/pandas/PyTorch/
TensorFlow/scikit-learn-class Python libraries; TensorFlow.js/ONNX-Runtime-Web-class TypeScript
libraries; DL4J/Tribuo/ND4J-class Java libraries — examples, not a mapping, §7) — so that Mycelium
users and developers can do real AI/ML work **natively**, inside Mycelium's own value-semantics,
guarantee-tag, and provenance model, without leaving the language for the compute layer.

This is the AI/ML-domain application of DN-85's flagship goal ("Mycelium as a single-language
solution for a source ecosystem's entire stack") — DN-85 states the general strategy and its
open-source constraint; this note is the domain corpus, adds the clean-room mechanism DN-85 §5
already anticipated but did not develop, and states the convergence discipline explicitly.

## 3. The three porting mechanisms

Each AI/ML library — or, more often, each **component within** a library (DN-86 §4.2's two-part
split: a pure-source layer plus a compiled compute core) — is ported by exactly one of three
mechanisms, chosen per component, each carrying its own honest guarantee basis. **A component's tag
never overstates which mechanism produced it** (G2/VR-5, the standing discipline DN-85 §7 already
establishes generally; restated here for AI/ML concretely).

### 3.1 Mechanism 1 — Transpile + patch (a BRIDGE)

For **open, transpilable source** — the DN-85/DN-86 front-end path (Python/TypeScript/Java →
Mycelium via `mycelium-transpile`'s per-language front-end, DN-86 §1–§3). The transpiler emits a
first draft (`Declared`); it is then **hand-patched to production readiness** (the M-993/M-1006-style
ladder — rip → vet → patch → record, `/myc-drafts` + `/transpile-vet`). Only vetted against the real
toolchain (`myc check`) does an emission earn `checked_fraction` credit; only a **differential**
against the source-language oracle upgrades a component's behavioral claim to `Empirical`.

- **Applies to:** a library's pure-source layer — API surface, argument validation, dtype/shape
  bookkeeping, pure-language algorithms, orchestration/glue (DN-86 §4.2's "Python layer").
- **Honesty tag:** `Declared` (emission) → `Empirical` (once a differential vets it) — never `Proven`
  without a checked derivation (DN-85 §7.1, generalized).
- **Licensing:** the transpiled+patched result is a **derivative** of the source under transpile —
  its license posture follows the source library's license, is recorded per-component, and does
  **not** default to MIT by virtue of being expressed in `.myc` (see §3.4).
- **Explicitly a BRIDGE**, per §4: as the source language's transpiler + the target Mycelium surface
  mature, less hand-patching is needed per component, but the mechanism itself does not "graduate"
  into mechanism 3 — a transpile+patch component only becomes clean-room-native if it is later
  **reimplemented from spec/behavior**, not merely re-transpiled.

### 3.2 Mechanism 2 — FFI bindings (a BRIDGE, explicitly INTERIM)

For components that **cannot** be transpiled+patched — closed-source or platform-specific
C++/CUDA backends (numpy's C ufuncs, PyTorch/TensorFlow's ATen/cuDNN, any proprietary compiled
kernel). Bound through the existing FFI surface — RFC-0028's `wild`/`@std-sys` vocabulary,
DN-14 row 9 (present, executes three-ways through the prim registry).

- **Applies to:** compiled compute cores DN-86 §4.2 identifies as the load-bearing boundary — "that
  code is not [source] and cannot be transpiled."
- **Honesty tag:** a binding is honestly **a binding to an external artifact** — its guarantee is
  "calls a binary we did not build" (`Declared`, provenance = external binding, DN-85 §5's provenance
  ladder row 1). It is **never** a native port and **never** first-party-MIT (§3.4). `EXPLAIN` on a
  bound call surfaces exactly that: which external artifact, which symbol, which ABI.
- **Explicitly INTERIM.** Per §4, mechanism 2 is the default *only* while the backend has not yet
  been clean-room engineered into native Mycelium; the intent from the outset is migration to
  mechanism 3, not permanent residence at mechanism 2 (a sharpening of DN-85 §4's "as each
  native-language transpiler lands, the bound layer is progressively replaced" — this note extends
  "replaced by a transpile" to "replaced by transpile *or* by clean-room reimplementation," since a
  compiled C++/CUDA core has no source to transpile in the first place, §3.3).

### 3.3 Mechanism 3 — Clean-room reverse-engineering, AI-accelerated (a NEW NATIVE IMPLEMENTATION)

For components where mechanism 1 does not apply (no transpilable source, or the source license
forbids derivative-work transpilation) and mechanism 2 is not the terminal state wanted: reimplement
from **specification and observed behavior** — the published algorithm, the documented numerical
contract, the API's black-box input/output behavior under a test harness — **never from copied
source**. AI-accelerated in the sense that an agent may propose the reimplementation and its test
harness, but the *artifact* under review is Mycelium-native code plus its differential evidence, not
a transformation of the original's source text.

- **Applies to:** any compute core (or pure-source component) the project chooses to bring fully
  native rather than leave bound — the terminal state for every mechanism-2 component (§4), and the
  only option for closed-source-with-no-transpilable-source components (DN-85 §5 row 3).
- **Honesty tag:** a **new native Mycelium implementation**. Its behavioral equivalence to the
  original is a **differential-tested `Empirical` claim** — a suite exercising the observable
  contract (property tests, corpus replay against the original where legally/practically obtainable,
  numerical-tolerance comparison for floating-point kernels) — and is **never** described as "a
  faithful port of X" (that phrase belongs to mechanism 1 only, and even there only once vetted). See
  §6 for the protocol.
- **Licensing:** genuinely **first-party, MIT-only** (§3.4) — the entire point of clean-room
  discipline is that no derivative-work entanglement with the original's license survives into the
  reimplementation.

### 3.4 Licensing posture (per mechanism, cited)

Per CONTRIBUTING §Licensing / ADR-022 §7.3: **the entire Mycelium project is MIT-licensed on every
first-party artifact** — no Apache-2.0, no dual-license. This binds the three mechanisms differently,
and the difference is never blurred:

- **Mechanism 3 (clean-room)** is genuinely first-party and MIT-only from creation — no source of the
  original was read or copied to produce it (§6's protocol is precisely the discipline that keeps
  this true and auditable).
- **Mechanism 1 (transpile+patch)** produces a **derivative** of the source library — its output
  license is **not** automatically MIT; it is recorded per-component against the source library's
  actual license (BSD/Apache-2.0/MIT/etc.), and a copyleft or incompatible license on the source is an
  explicit **gap** (never silently transpiled anyway, G2) requiring either a compatible-license
  original or routing that component to mechanism 3 instead.
- **Mechanism 2 (FFI binding)** does not relicense the bound artifact — the binding code (the Mycelium
  `wild`/`@std-sys` glue) is first-party MIT, but the artifact it calls **keeps its own license**,
  recorded and honored (dynamic-link vs. static-link distinctions, attribution requirements, etc., are
  a per-library legal review, not assumed away here).
- **No component's license status is inferred** — each is recorded explicitly per DN-85 §7.2's
  binding-provenance-tracking open question (Q2), extended here to include the license field.

## 4. The convergence path — bridges to full-native

**The terminal goal is full native Mycelium everywhere.** Mechanisms 1 and 2 are explicitly
**bridges** — practical, honestly-tagged, but not the destination. Mechanism 3 is the only mechanism
that reaches the destination directly.

The migration shape, stated concretely:

- A library ships **today** as (Mycelium-native front layer, via mechanism 1) + (FFI-bound
  C++/CUDA backend, via mechanism 2) — the two-part DN-86 §4.2 split, honestly tagged component by
  component.
- **Over time**, where resourcing allows, the FFI-bound backend's connected sources are **clean-room
  engineered into native Mycelium** (mechanism 3) and **added as first-party components of the same
  repo** — not a separate "alternative implementation" project, but the library's own backend
  migrating in place.
- The library's steady-state trajectory is: (native front + bound back) → (native front + native
  back, the bound layer retired) — **full native Mycelium throughout**, at which point the library
  carries no mechanism-2 components at all.
- A mechanism-1 component's trajectory is simpler: it stays a bridge only until its `checked_fraction`
  reaches production readiness (DN-88 §3's two-stage bar); it does not need a mechanism-3 rewrite
  *unless* its source license blocks mechanism 1 outright (§3.4).

**Never-silent per-component tracking (G2).** Every component in the AI/ML corpus records, at all
times and inspectably (an `EXPLAIN`-able manifest, in the spirit of the DN-85 §7 provenance ladder
and DN-26/DN-88's differential ledgers — the concrete mechanism is future implementation work, not
fixed by this note):

1. **Current mechanism** — transpiled+patched / FFI-bound / clean-roomed-native, per component (not
   per library — a single library is routinely a mix).
2. **Target native state** — whether this component's terminal form is "already native" (mechanism 3
   or a fully-vetted mechanism 1), "native front / bound back, backend migration planned," or
   "binding retained indefinitely" (a rare, explicitly-argued case — e.g., a backend requiring
   hardware-vendor-proprietary microcode with no specifiable behavior to clean-room against).
3. **Guarantee tag** at the component's actual, current strength (§3's per-mechanism tags) — never
   upgraded because the *library* is "mostly native."

This makes migration status **always inspectable**: a user or auditor can ask, of any AI/ML library
in the corpus, exactly which of its components are native today, which are bridged, and toward what.

## 5. Port-quality principle — leverage and improve, never verbatim clones

A Mycelium port or reimplementation is not a mechanical transliteration. It is **refactored and
polished to leverage Mycelium's own advantages** — value-semantics, never-silent swaps, the
guarantee-tag lattice (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`), `EXPLAIN`-ability, content-addressing
and provenance — and, **where possible, measurably improves** on the opaque original in capability,
performance, safety, provenance, traceability, or explainability.

**The honesty split this principle requires (stated explicitly, not softened):**

- **Behavioral equivalence is the FLOOR.** Every port/reimplementation must first clear the §3
  mechanism's own equivalence bar (a vetted transpile differential, or the §6 clean-room `Empirical`
  behavioral-equivalence claim) before any improvement claim is entertained.
- **Improvements are GOALS, tagged at their real strength — never assumed.**
  - **Provenance, traceability, and explainability improve almost *by construction*.** A Mycelium
    port inherits the guarantee-tag lattice, `EXPLAIN`-able selection records, and content-addressed
    provenance simply by being expressed in Mycelium — this is a **structural** consequence of the
    language's design, not a claim that needs a benchmark to support (though it is still stated at
    whatever tag its specific mechanism, §3, actually earns).
  - **Performance and safety gains are EARNED, not automatic.** A `.myc` port is not faster than a
    CUDA kernel *by virtue of being Mycelium* — a perf win must be **measured** (`Empirical`, with the
    benchmark methodology recorded) and a safety gain must be **checked** or explicitly flagged
    `Declared` until it is. This note makes **no claim** that any port will outperform or out-safety
    its original; it states only that improvement, where achieved, is tagged honestly at the strength
    it earns.
- **The note deliberately does not say "better because it's Mycelium."** That phrasing would upgrade
  an aspiration past its checked basis for the earned half of the split (VR-5) — the structural half
  is a real, inspectable property of the language design; the earned half is an empirical claim about
  a specific artifact, made or not made per-artifact, per-benchmark.

## 6. Clean-room protocol (mechanism 3)

The discipline that keeps a clean-room reimplementation genuinely clean-room, MIT-clean, and honestly
tagged:

1. **Spec/behavior-driven, never source-driven.** The reimplementation is derived from: published
   papers/specifications, documented API contracts, and **observed black-box behavior** under a test
   harness (inputs → outputs, including documented edge cases and numerical tolerances) — never from
   reading or transforming the original's source code. Where an agent or engineer has been exposed to
   the original source (e.g., during the DN-86 transpile-attempt phase, or general familiarity), the
   reimplementation is written by a **separately-scoped effort** with only the specification/behavior
   contract as input (the standard clean-room separation-of-roles discipline), and this separation is
   recorded, not assumed.
2. **An equivalence witness, differential-tested.** Behavioral equivalence to the original is
   established by a **differential** over the observable contract — property tests derived from the
   spec, and where legally/practically obtainable, corpus replay comparing outputs against the
   original within a stated numerical tolerance. The witness's coverage (which inputs, which
   tolerance, which edge cases) is recorded alongside the claim, per DN-85 §8's open question Q3
   (acceptable equivalence witnesses), which this note adopts as its own protocol design point rather
   than re-deciding independently.
3. **Provenance records the mechanism, not a false lineage.** The reimplementation's provenance
   metadata names it a **clean-room reimplementation of [library]'s [component] behavior**, cites the
   spec/observed-behavior basis, and **never** says "ported from" or "based on \[library\]'s source" —
   because it wasn't.
4. **MIT-clean by construction.** Because no source of the original was read or transformed, the
   reimplementation carries no derivative-work entanglement and is first-party MIT from creation
   (§3.4) — the clean-room discipline above is precisely what makes this claim true, not merely
   asserted.
5. **Guarantee ceiling.** Behavioral equivalence stays `Empirical` (trial-tested against the observed
   contract) — it is **`Proven`** only for the rare component where a formal specification exists and
   the equivalence is machine-checked against it (VR-5's ordinary bar, no domain-specific exception).
   Most numerical AI/ML kernels (floating-point, GPU-parallel, or involving nondeterministic training
   dynamics) will realistically ceiling at `Empirical` — this note does not pretend otherwise.
6. **Never "a faithful port of X."** The phrase "port" is reserved for mechanism 1 (transpile+patch);
   a mechanism-3 artifact is a **reimplementation**, and its documentation, commit messages, and
   provenance metadata use that word, not "port" (G2/VR-5 — the same discipline DN-85 §5/§7 already
   states generally, restated as an explicit clean-room rule because this is where it is most likely
   to be blurred under time pressure).

## 7. Target library landscape (criteria, not a mapping — mitigation #1)

**This note deliberately does not enumerate a component→library→mechanism mapping.** That mapping is
future work, produced when the gating capstone (§1) closes and a concrete porting wave is scoped
(mirroring DN-88 §1's "the actual decomposition guide and component→repo mapping are produced later,
gated on production-ready"). What this note fixes is the **criteria** a future scoping pass applies:

- **Per-language candidate classes** (examples illustrating the criteria, not commitments):
  - **Python** — numerical/array computing (numpy-class), scientific computing (scipy-class),
    dataframe/tabular (pandas-class), classical ML (scikit-learn-class), deep learning
    (PyTorch/TensorFlow/JAX-class), and narrower domain libraries (tokenizers, model-serving
    runtimes) as demand identifies them — sequenced consistently with DN-85 §3's pure-Python-first,
    sound-type-inference-gated Python arm.
  - **TypeScript** — browser/edge-runtime ML (TensorFlow.js-class), ONNX-runtime-web-class inference
    engines, and JS-ecosystem tooling around model serving.
  - **Java** — enterprise/JVM ML (DL4J/ND4J-class, Tribuo-class), consistent with DN-86 §2's read that
    TypeScript and Java are the more tractable, statically-typed front-ends.
- **Mechanism-selection criteria, per component:**
  1. **Source availability + license compatibility** → eligible for mechanism 1 (transpile+patch) if
     open and the resulting derivative's license is compatible or the source license permits the
     transpile (§3.4); otherwise route to mechanism 2 or 3.
  2. **Transpilability** (is it pure-source in a language the transpiler front-end covers, per DN-86
     §1's IR-lowering architecture) vs. **compiled/platform-specific** (C/C++/CUDA/proprietary
     microcode) → the DN-86 §4.2 boundary decides mechanism 1 vs. {2, 3}.
  3. **Closed-source or unobtainable source, but a specifiable behavioral contract** → mechanism 3
     directly (no mechanism-2 interim needed if a bound artifact isn't even available to bind).
  4. **Closed compiled artifact IS obtainable (vendor SDK, proprietary runtime) but its full behavior
     is impractical to spec/differential-test at acceptable confidence** → mechanism 2, with an
     explicit, tracked intent to revisit mechanism 3 as the spec/behavior base matures (§4).
  5. **Value/demand weighting** — as DN-85 §3 sequences "pure Python first," this note's future
     scoping pass should weight candidates by real usage demand pulling the work, not a big-bang
     enumeration (consistent with DN-85 §2's "extends to a new source language as we encounter it and
     the related need").
- **Explicitly out of scope for this note:** naming specific libraries as committed targets, ordering
  a porting sequence, or estimating effort. Those require the gating capstone's actual state (what the
  Mycelium compute stack — §9 — can carry by then) as an input this note does not yet have.

## 8. Interpreter/AOT parity

Per DN-88 §3's two-stage bar (reused here, not reinvented): a ported AI/ML component reaches
production readiness only after (1) it is **interpreter-runnable with a 100% checkout** against its
reference behavior, and only then (2) it is **AOT-compiled**. This note adds the domain-specific
reading:

- **Expected, acceptable disparity: performance.** The reference interpreter is definitionally not
  the fast path (CLAUDE.md: "MLIR→LLVM is the perf-path AOT; the interpreter is the trusted base").
  An AI/ML kernel running measurably slower under the interpreter than under AOT (or than the
  original's native backend) is expected and not a defect.
- **Unacceptable, must-close disparity: capability.** A **capability gap** between the interpreter
  and AOT legs — an operation the interpreter can execute but the AOT path cannot (or vice versa,
  or with different numerical results) — is not a performance characteristic, it is a **correctness
  gap** and must be closed, never silently shipped as "the AOT version is a subset" (G2).
- **Tie to the semcore dossier's live open question.** `docs/planning/semcore-l0-boundary-dossier.md`
  §7 FLAG-4 records, for the self-hosted toolchain itself, that a self-hosted evaluator run *inside*
  the L1 evaluator "almost certainly cannot complete at today's cost model" (the M-986 TCO gap + M-987
  ~n³ eval cost) — meaning some validation legs may need the **AOT** path rather than the interpreted
  one even to *complete*, independent of which is "faster." AI/ML workloads are exactly the shape
  (large tensors, tight inner loops) where this is likely to recur: a correctness differential that
  is impractical to run interpreted at all is a **capability gap in the validation harness**, not
  license to skip the interpreter leg's *coverage* claim — DN-26 §9/§10 is the owning track for
  resolving the general shape of this tension; this note inherits it as a dependency, not a novel
  problem to re-solve here.
- **General completion criterion, not just AI/ML-specific:** closing interpreter/AOT capability
  disparity is standing toolchain work (DN-26/semcore dossier own it); this note's contribution is
  naming it explicitly as a per-component gate for every AI/ML port, since numerically-heavy code is
  disproportionately likely to expose such a gap first.

## 9. Leveraging Mycelium's differentiators (`Declared`/aspirational)

Mycelium has domain-relevant capabilities most host languages for AI/ML lack natively. This section
names them as **aspirational leverage points** for the port-quality principle (§5) — `Declared`
until a specific port actually exercises and measures them:

- **`mycelium-vsa` (VSA / hyperdimensional computing, RFC-0003/RFC-0009).** Vector-symbolic
  architecture — bind/bundle/permute over MAP-I/BSC/HRR/FHRR models, resonator-network factorization
  (RFC-0009, Enacted) — is a native Mycelium capability with **no direct equivalent** as a first-class
  language primitive in numpy/PyTorch/TensorFlow (those implement HDC, where they do at all, as a
  library on top of dense tensors). A native AI/ML corpus that exposes VSA-based approaches
  (associative memory, symbolic binding, resonator decoding) as first-class alongside conventional
  dense/tensor methods is a **capability** the ported originals do not have — not an improvement *on*
  them, but an addition *beyond* them, honestly distinguished from "faster/safer port of X."
- **Value semantics + never-silent swap.** A representation change (dense↔sparse, quantized↔full
  precision, binary↔ternary) is, in Mycelium, an explicit, auditable `swap` — never a silent
  dtype-coercion the way it routinely is in numpy/PyTorch. A ported AI/ML numerical routine can
  surface every precision/representation decision it makes as an inspectable event, which most
  originals do not.
- **Guarantee tags on numerical results.** A port can attach the `Exact ⊐ Proven ⊐ Empirical ⊐
  Declared` lattice to individual computed values or model outputs (e.g., "this inference result is
  `Empirical` under model M, decoded via resonator convergence with a recorded confidence"), a
  provenance-carrying capability the originals' plain float/tensor outputs lack.
- **Content-addressing / provenance.** A trained model, a dataset transform, or an inference
  pipeline's spore (ADR-013) is content-addressed end to end — reproducibility and lineage tracking
  that most AI/ML ecosystems bolt on via separate MLOps tooling, native here.
- **Honesty split restated (§5):** these are real, structural properties of the language a port
  *inherits* by being written in Mycelium — they do not require a benchmark to claim. Whether a
  *specific* port's use of them constitutes a measurable *improvement* over the original's own
  provenance/safety/perf story is a separate, per-port `Empirical`/`Declared` claim, made when and if
  it is actually measured.

## 10. User stories + Definition of Done

**User stories:**

- As a **Mycelium application developer**, I want production-grade AI/ML libraries available
  natively in Mycelium, so that I can build ML-powered applications end-to-end in one language, one
  guarantee model, without a Python/C++ seam.
- As a **library porter** working an AI/ML component, I want a clear per-component decision procedure
  (transpile vs. bind vs. clean-room) with an explicit honesty tag for each, so that I never
  accidentally overstate what a component is (a binding presented as a port, a reimplementation
  presented as a faithful clone).
- As an **auditor or downstream user** evaluating a ported AI/ML library, I want to inspect, per
  component, its current mechanism, its target native state, and its guarantee tag, so that I can
  assess exactly how much of the library is genuinely native Mycelium today versus bridged.
- As a **safety- or compliance-sensitive user**, I want performance and safety improvement claims
  measured and tagged at their real strength — never asserted as automatic because "it's Mycelium" —
  so that I can trust the corpus's claims are checked, not marketing.
- As the **maintainer**, I want this vision captured durably now, gated explicitly on the self-hosting
  capstone, so that the intent survives to whenever the gate closes without needing to be
  re-articulated from memory.

**Definition of Done (for this note):**

- [ ] Maintainer confirms the §1 gating read (the compound "language-project-frozen" vs.
  "kernel-included" distinction, and which resolution of ADR-042 OQ-1 the terminal gate depends on).
- [ ] The first concrete AI/ML porting wave is minted as a scoped RFC/ADR + issues under the proposed
  **E41-1** epic, once the gate closes.
- [ ] The per-component provenance/mechanism-tracking manifest shape (§4) is concretely designed (this
  note states the requirement, not the schema).
- [ ] The clean-room protocol (§6) is cross-checked against actual legal/licensing guidance before the
  first mechanism-3 component is attempted (this note is not a substitute for that review).

**Definition of Done (for the eventual program, tracked, not claimed here):**

- At least one library per source language (Python/TypeScript/Java) has at least one component ported
  by each of the three mechanisms, each cleared through its mechanism's honesty bar and the DN-88
  two-stage interpreter→AOT gate.
- The per-component mechanism/target-state manifest is live and queryable for the whole AI/ML corpus.
- No component in the corpus is tagged above its checked basis (VR-5 audit, standing).

## 11. Open questions / FLAGs

- **OQ-1 — this note's gate inherits ADR-042's own unresolved bootstrap/trust question (§1).** ADR-042
  §6 OQ-1 (the self-hosted-kernel bootstrap/trust story — self-AOT, a bootstrap seed, reflective-trust
  discharge, or a redefined trusted base) is `needs-design` and unresolved as of this note's authoring.
  Until it is designed and discharged, this note's gate (§1) cannot itself close, since ADR-042's own
  `Accepted → Enacted` condition requires exactly that. Not re-litigated here — tracked at ADR-042's
  level; this note is a downstream dependent, not a second opinion on it.
- **OQ-2 — mechanism-2-to-3 migration triggers.** What concretely triggers a backend's migration from
  FFI-bound to clean-room-native (§4) — a scheduling decision, a resourcing threshold, a maturity bar
  on the spec/behavior base? Not designed here.
- **OQ-3 — the provenance/manifest mechanism.** §4 requires per-component mechanism + target-state
  tracking be "inspectable" but does not fix the concrete artifact (a manifest file, a
  `mycelium-proj.toml` extension, a `docs/api-index/`-style generated index). Design point for the
  first concrete wave.
- **OQ-4 — clean-room legal review.** §6's protocol is an engineering discipline; it has **not** been
  reviewed by anyone with legal/IP expertise. Flagged explicitly — do not treat §6 as a legal clearance
  for mechanism 3 without that review.
- **OQ-5 — inherited from DN-85 §8, unchanged, restated as still-open here.** Q3's "acceptable
  equivalence witness" question and Q4's "per-language differential oracle" question both bear
  directly on §6's protocol and are not re-litigated or resolved by this note — they remain open at
  DN-85's level and this note's clean-room protocol depends on their eventual resolution.
- **OQ-6 — VSA/dense compute-stack maturity as a precondition.** §9's differentiators (native VSA,
  native swap, guarantee-tagged numerics) presuppose the compute-stack tracks (`mycelium-dense`,
  `mycelium-vsa`, `mycelium-numerics`, RFC-0039's native Dense/VSA codegen) are themselves mature
  by the time this program starts — not assumed here, just named as a real dependency this note does
  not itself resource or schedule.

## Meta — changelog

- **2026-07-07 — Created (Proposed).** Captures the maintainer's forward-looking vision for a native
  AI/ML library ecosystem, gated on the self-hosting capstone — read directly against ADR-042 (Accepted
  the same session, reframed mid-authoring of this note to its two-horizon form: NOW = Rust-base freeze;
  END-STATE = zero foreign first-party languages by the DN-88 decomposition gate, kernel *and*
  AOT/codegen backend included, DN-39 preserved as boundary-vs-implementation-language per its §2.2).
  This note's gate is ADR-042's end-state as actually decided, not an extrapolation past it; it carries
  forward ADR-042's own honesty posture that the end-state's feasibility is `Declared`/`needs-design`
  (the OQ-1 bootstrap/trust story), not proven or near (§1/OQ-1). Records the three porting mechanisms (transpile+patch,
  FFI-bind, clean-room reverse-engineering) and their honesty tags, extending DN-85's two-mechanism
  ladder with the clean-room third arm DN-85 §5 anticipated but did not develop. States the
  convergence path from bridge (mechanisms 1/2) to full-native (mechanism 3) with never-silent
  per-component mechanism/target tracking. States the port-quality principle (leverage + improve,
  never verbatim clone) with the structural-vs-earned honesty split (provenance/traceability improve
  by construction; performance/safety gains are earned and must be measured — explicitly rejects
  "better because it's Mycelium" framing for the earned half). Gives the clean-room protocol
  (spec/behavior-driven, MIT-clean, `Empirical`-ceilinged, never "a faithful port"). Gives target
  library landscape as criteria only, no mapping (mitigation #1). Reuses DN-88 §3's two-stage
  interpreter→AOT bar verbatim as the per-component readiness gate and ties the capability-vs-
  performance distinction to the semcore dossier §7 FLAG-4 eval-cost finding. Names VSA/value-
  semantics/guarantee-tags as `Declared`/aspirational differentiators. Decides nothing normatively;
  ratifies no mechanism (VR-5/G2/house rule #4).
