# RFC-0016 — Core Library & Standard Library

| Field | Value |
|---|---|
| **RFC** | 0016 |
| **Status** | **Enacted** (2026-06-21 — all 25 `mycelium-std-*` crates landed Rust-first, M-501–M-534/M-540/M-541 done; 23-module guarantee matrices asserted in tests; never-silent G2 and honest tags hold across all modules; self-hosting migration half stays Phase-5-C/M-502-gated per KC-2 ruling. Append-only; steps through Accepted below.) **Accepted** (2026-06-17 — maintainer ratification, M-501/DN-07. The §4.1 per-op **contract** (C1–C6), the §4.2 **ring layering**, the §4.3/§4.4 **Tier-A/Tier-B taxonomy** (full 23-module v0 scope), the §4.5 **guarantee-matrix** obligation, and the §4.6 **Rust-first → Mycelium-lang migration** order are ratified. §8 dispositions recorded in §8 + the changelog: **Q1** (full taxonomy; five-candidate floor first; `diag`/`recover` lead migration), **Q2** (phylum `std`; crate-mirrored names; one `core`↔`error` error-value name — a DN-level lexicon call), **Q5** (two-level differential bar), **Q6** (`std-sys` phylum split), and the `BF16→F32` placement (→ `cmp`/`convert`) are **resolved**; **Q3** (ergonomics-vs-contract) accepts the RFC-0012 ambient *direction* with a scheduled per-ring design pass, and **Q4** (`runtime` placement) is deferred to the Phase-7 gate (separate `runtime` phylum) — both **deferred-with-direction**, neither blocking this ratification. The §7 grounding obligation is **discharged** (`research/08-honest-stdlib-prior-art-RECORD.md`). Append-only; supersedes the 2026-06-17 Draft.) |
| **Type** | Foundational / normative (once Accepted) — the standard-library **scope + contract**; the library lives **above** the kernel (KC-3), no kernel change |
| **Date** | 2026-06-17 |
| **Tracks** | **M-346** (the stdlib roadmap epic — this RFC is its named "Core Library RFC" deliverable) and **M-501** (author + ratify). Decomposes into the per-module tasks **M-510…M-534** (Phase 5; `docs/planning/phase-5.md`). |
| **Depends on** | RFC-0001 (the value model — `Value`/`Repr`/`Meta`, the guarantee lattice, content-addressing §4.6, (de)serialization §4.8); RFC-0002 (swap certificates); RFC-0003/0009 (VSA + resonator); RFC-0004 §5 (packing); RFC-0005/ADR-006 (selection + EXPLAIN); RFC-0006/0007 (the surface + L1 calculus the modules are written in); RFC-0008 (the runtime/concurrency surface); RFC-0013/0014/0015 (diagnostics / recovery / auto-baseline); ADR-003 (content-addressing), ADR-007 (Rust-first toolchain), ADR-010 (verified numerics), ADR-013 (`spore`), ADR-014 (`wild`/unsafe); G2 (never-silent), G11 (dual projection), VR-5 (honest tags), KC-2/KC-3, FR-M2/FR-C2/FR-C3, LR-8 (`substrate`) |

---

## 1. Summary

Mycelium has a small auditable kernel (KC-3), a reference interpreter, an AOT path, a surface language
(RFC-0006/0007), and — landed across Phases 1–4 — a set of capability crates (numerics, swaps, VSA,
selection, dense, diagnostics, recovery). What it does **not** yet have is a **standard library**: the
coherent, documented, dogfoodable set of `nodule`s and `phylum`s that make Mycelium *usable* to write
programs in, not just a substrate to study. **M-346** is the roadmap anchor for that work; this RFC is the
**Core Library RFC** M-346's acceptance names — it fixes (1) the **per-operation contract** every stdlib
op must meet, (2) the **module taxonomy** (split into *differentiator* modules unique to Mycelium and
*common* modules every language needs), (3) the **layering** (what sits where relative to the kernel), and
(4) the **Rust-first → Mycelium-lang migration** order (the dogfooding trajectory).

This RFC is **scope + contract**, not implementation. Each module decomposes into its own Phase-5 task
(`M-510…M-534`); each module then follows the normal design-first discipline. Nothing here ratifies a
module's internals — it ratifies *what the library is, what every piece of it must promise, and the order
we build it in*.

## 2. Motivation

- **A substrate is not a language you can use.** Everything to date is the *trusted base* + capability
  crates. To write real programs — and to **self-host** the toolchain/diagnostics (the "free of other
  languages" goal, M-346) — Mycelium needs collections, text, IO, math, iteration, error ergonomics, and
  the language-specific surfaces (swap, VSA, ternary, the runtime lexicon) as a **library with a stable,
  documented, honest API**.
- **The honesty rule must not stop at the kernel.** The danger in any stdlib is the silent default: a
  `sort` that loses ties, a `parse` that returns a sentinel, a float op that hides its rounding, a
  collection that silently rehashes. Mycelium's whole thesis is that **every** accuracy/guarantee claim is
  tagged and **never** silent (G2/VR-5). A stdlib that quietly violated that would make the substrate's
  promise a lie at the layer users actually touch. So the **contract** (§4.1) is the load-bearing part of
  this RFC, and it is non-negotiable for every op.
- **The differentiators deserve a first-class surface.** Mycelium's *reason to exist* — certified swaps,
  honest bounds, VSA/HDC, balanced ternary, reified selection/EXPLAIN, structured diagnostics/recovery,
  the fungal runtime — should be the **best-supported, most ergonomic** part of the library, not an
  afterthought bolted beside a generic stdlib. §4.3 (Tier A) is where that lives.
- **Common needs are table stakes.** Equally, no one adopts a language whose stdlib lacks maps, strings,
  and IO. §4.4 (Tier B) captures the ordinary surface every language is expected to have — held to the
  *same* honesty contract, which is what makes Mycelium's version of `collections` or `time` distinctive
  rather than a clone.

## 3. Guide-level explanation

The standard library is organised as one top-level `phylum` (working name `std`) whose `nodule`s are the
modules below. A program does not need all of it; modules are independently usable and content-addressed
(ADR-003). Two tiers, by *intent*, not by importance:

- **Tier A — differentiator modules**: the surfaces that are *Mycelium-shaped* and exist in few or no
  other languages — `swap`, `numerics` (honest ε/δ bounds), `vsa`, `ternary`, `dense`, `select`/`explain`,
  `diag`, `recover`, the `runtime` fungal lexicon, `spore`, and `content`. These trace directly to the
  Accepted RFCs/ADRs that designed them; the stdlib gives them an ergonomic, documented home.
- **Tier B — common modules**: the surfaces every language is expected to provide — `collections`,
  `text`, `math`, `iter`, `error`/`option`/`result`, `io`, `fs`, `serialize`, `time`, `rand`,
  `cmp`/`convert`, `fmt`, `testing`. Ordinary in shape, but each is held to the §4.1 contract — that is
  what makes them *Mycelium's*.

Both tiers are written **Rust-first now** (ADR-007 — the trusted toolchain), then **migrated to
Mycelium-lang** as the surface self-hosts (§4.6; the M-502 readiness gate decides "close enough"). A
module graduates from Rust to Mycelium-lang only when its self-hosted form passes a differential against
the Rust reference (NFR-7-style), so the migration never trades away a checked guarantee.

## 4. Reference-level design (normative once Accepted)

### 4.1 The contract every stdlib operation must meet (the load-bearing rule)

This is the part of the RFC that is **not** open to per-module negotiation. Every operation exported by
any stdlib `nodule` — Tier A or B — must satisfy **all** of:

- **(C1) Never-silent (G2).** No silent failure, no silent loss, no silent approximation. Every fallible
  op returns an explicit `Option`/`Result`/refusal that **propagates**; an out-of-range, malformed, or
  unsupported input is an explicit error, never a sentinel, a clamp, or a partial result. (A `parse`
  failure is `Err`, not `0`; an off-grid float is refused, not re-rounded.)
- **(C2) Honest per-op guarantee tag (VR-5).** Any op whose result carries accuracy/precision/probability
  semantics tags it on the lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`, in its `Meta`. `Proven` only
  with a theorem whose side-conditions are *checked*; otherwise `Empirical` (trials, with a method) or
  `Declared` (asserted, flagged). **Downgrade to stay honest; never upgrade without a checked basis.** An
  op with no accuracy semantics (e.g. `len`) is simply `Exact`.
- **(C3) No black boxes / EXPLAIN (SC-3/G11).** Any op that *selects*, *converts*, or *approximates*
  exposes *why* — via a reified, inspectable artifact (an RFC-0005 policy + EXPLAIN record, a swap
  certificate, a diagnostic record). No opaque heuristic decides a user-visible outcome.
- **(C4) Content-addressed, value-semantic (ADR-003 / RFC-0001).** Library data structures are immutable
  values with content-addressed identity where it applies; an operation is a pure function of its inputs +
  declared effects (RFC-0014). Metadata is **not** identity (ADR-003).
- **(C5) Above the small kernel (KC-3).** The stdlib lives **above** `mycelium-core`; it may *consume* the
  kernel and the capability crates but must not enlarge the trusted base. A module that needs `wild`/FFI
  (ADR-014) confines it to an audited `wild` block (LR-9/S6) and says so.
- **(C6) Declared, bounded effects (RFC-0014).** An op with effects (IO, time, randomness, allocation
  beyond the obvious) **declares** them on its signature; unbounded effects are bounded by an explicit
  budget where one applies (the EffectBudget discipline). No undeclared side effect.

A module's per-op tags are recorded in a **guarantee matrix** in its own task (the RFC-0003 §4 matrix is
the template), so the honesty claims are a checked table, not prose.

### 4.2 Layering & packaging

- **`std` is a `phylum`** (DN-06; library-scale, content-addressed); each module is a `nodule` (or a small
  cluster of nodules). Modules are independently importable.
- **A three-ring layering** keeps KC-3 honest:
  - **Ring 0 — kernel-adjacent re-exports** (`core`/prelude): the value model, `Option`/`Result`/error
    values, the guarantee-lattice types — thin, mostly re-exporting `mycelium-core` (RFC-0001). *No new
    trusted code.*
  - **Ring 1 — capability surfaces** (Tier A): ergonomic libraries over the landed capability crates
    (`numerics`, `swap`, `vsa`, `dense`, `select`, `diag`, `recover`, `spore`). These *consume* the trusted
    base; they are certificate/EXPLAIN **consumers**.
  - **Ring 2 — general library** (Tier B + `runtime`): collections, text, IO, etc., written to the
    contract over Ring 0/1.
- **Packaging**: each module ships in the `std` `phylum`; the publishable artifact is a `spore` (ADR-013).
  Versioning + metadata ride the M-359 `mycelium-proj.toml` manifest; metadata is not identity (ADR-003).

### 4.3 Tier A — differentiator modules (Mycelium-specific)

Each traces to its Accepted/ratifying corpus; the stdlib gives it a documented, ergonomic home. **Bold** =
already has a landed capability crate the module wraps.

| Module | What it is | Grounding | Task |
|---|---|---|---|
| `core` / prelude | the honest value model: `Value`/`Repr`/`Meta`, `Option`/`Result`/error values, the guarantee-lattice tags; Ring-0 re-exports | RFC-0001 | **M-515** |
| **`numerics`** | honest ε/δ-bounded arithmetic helpers over the two bound kernels + shared certificate | ADR-010; M-201/202/203 | **M-512** |
| **`swap`** | the certified, never-silent representation-change library: legal-pair swaps, certificate build/check | RFC-0002; M-120/210/211/231 | **M-516** |
| **`vsa`** / `hdc` | hypervector models (bind/unbind/bundle/permute), cleanup memory, reconstruction manifests, resonator decode | RFC-0003/0009; M-130/240–242/260/350 | **M-513** |
| `ternary` | balanced-ternary arithmetic, `Bit`/`Trit`, packed-ternary helpers | FR-M2; M-111; RFC-0004 §5 | **M-517** |
| **`dense`** | typed dim-tracked dense tensors / embeddings, elementwise + similarity ops with per-op tags | RFC-0001 §4.1; M-230 | **M-518** |
| **`select`** / `explain` | the selection-policy DSL surface + mandatory EXPLAIN records (one mechanism, many sites) | RFC-0005/ADR-006; M-220/221/222 | **M-519** |
| **`diag`** | structured diagnostics — additive presentation over explicit errors (never substitutive, I1) | RFC-0013; M-345 | **M-510** |
| **`recover`** | declarative error recovery + bounded effects (errors-as-values that can *trigger* declared, bounded behaviour) | RFC-0014; M-352 | **M-520** |
| `runtime` / `colony` | the fungal concurrency surface — `hypha`/`fuse`/`colony`/`cyst`/`graft`/`forage`/`backbone`/`mesh`/`tier`/`reclaim`; structured concurrency + supervision | RFC-0008; M-355–357 | **M-521** |
| `spore` | the content-addressed deployable / reconstruction-manifest library | ADR-013; RFC-0003 §6; M-368 | **M-522** |
| `content` / `hash` | content-addressing primitives (the identity model as a first-class library; distinct from hashing-for-maps) | ADR-003; RFC-0001 §4.6 | **M-523** |

`runtime`/`colony` (M-521) is largely **reserved-not-active** vocabulary (Glossary ⟂) until the RFC-0008
constructs land; the stdlib bindings activate construct-by-construct at the Phase-7 gate, so this module is
sequenced against that track (a FLAGGED cross-phase dependency, §8-Q4).

### 4.4 Tier B — common / expected modules

Ordinary surfaces, held to the §4.1 contract. **Bold** = already seeded as an M-346 early candidate.

| Module | What it is | Honesty crux | Task |
|---|---|---|---|
| **`collections`** | persistent/immutable Vec/List, Map, Set, Deque, ordered/sorted variants | value-semantic by default; a rehash/rebalance is not a silent reorder of observable results | **M-511** |
| `text` / `string` | UTF-8 strings, slicing, formatting, parsing | `parse` is `Result`, never a sentinel; lossy encoding is an explicit error (C1) | **M-524** |
| `math` | numeric functions (over the honest numerics where precision matters) | a function with rounding/approximation carries its tag (C2); domain errors explicit | **M-525** |
| `iter` | iterator / fold / transducer combinators over the RFC-0007 §4.8 `for` fold primitive | total + terminating where the kernel guarantees it; laziness is explicit | **M-526** |
| `error` / `option` / `result` | the errors-as-values ergonomics layer (combinators, `?`-style propagation) | the never-silent floor (I1) — propagation is the default, suppression is impossible | **M-527** |
| **`io`** | input/output over `substrate` affine handles (consumed exactly once) | a `substrate` is single-consumption (LR-8); a partial/failed IO is explicit | **M-514** (io half) |
| `fs` | filesystem over substrates | every path/permission failure explicit; no silent create-on-write | **M-528** |
| `serialize` / `encoding` | (de)serialization (RFC-0001 §4.8), JSON/binary/base-N, the dual human/machine projection | round-trip is a checked property; malformed input explicit; serialization is a projection, not identity | **M-514** (ser. half) |
| `time` | clocks, durations, instants | monotonic vs wall-clock is a *typed distinction*, not a silent swap; logical clocks tie to RFC-0008 | **M-529** |
| `rand` | random number generation | nondeterminism is **reified/named** (RT3) — a deterministic-fragment program cannot pull entropy silently | **M-531** |
| `cmp` / `convert` | ordering/equality traits; non-representation conversions (distinct from `swap`) | a lossy `convert` is an explicit fallible op, never a silent narrowing | **M-532** |
| `fmt` | formatting / display | the dual human + machine (JSON) projection (G11), one canonical form | **M-533** |
| `testing` | property / golden / differential test harness (the repo's own discipline as a library) | a skipped/undetermined check is reported, never a silent pass (the `just check` ethos) | **M-534** |

### 4.5 The per-op guarantee matrix (every module ships one)

Each module's task delivers, alongside its code, a **guarantee matrix**: rows = exported ops, columns =
`{guarantee tag, fallibility (the explicit error set), declared effects, EXPLAIN-able? }`. This is the
checked, single-source-of-truth honesty table for the module (the RFC-0003 §4 matrix is the proven
template — encoded as data and asserted in tests, never prose only). The matrix is how C1/C2/C3/C6 are
*verified* rather than claimed.

### 4.6 Rust-first → Mycelium-lang migration (dogfooding)

Per ADR-007 and M-346 ("Rust-first now, Mycelium-lang eventually"):

1. **Phase 5a — Rust reference.** Every module lands first as a Rust crate/`nodule` under the trusted
   toolchain, meeting the §4.1 contract.
2. **M-502 readiness gate.** Self-hosting begins only when the surface language is "self-hosting enough"
   to author the module in Mycelium (M-502 makes that verdict *checkable*, never pre-declared).
3. **Phase 5b — migration with a differential.** A module is re-authored in Mycelium-lang and validated
   against its Rust reference (an NFR-7-style differential). It graduates only when the self-hosted form
   passes — the migration never silently weakens a guarantee.

`diag` + `recover` (M-510/M-520) are the **first** migration targets — the milestones.json Phase-5 charter
names "self-host the RFC-0013/0014 diagnostics + recovery" specifically, and they are the most
honesty-load-bearing, so dogfooding them first proves the contract.

## 5. Drawbacks

- **Surface area is large.** A full stdlib is a lot of design + code held to a strict contract. Mitigated
  by the ring layering (Ring 1 mostly wraps landed crates), the per-module decomposition (independent
  tasks, sequenced by priority), and *not* committing module internals here (scope only).
- **The contract raises the per-op cost.** Tagging, EXPLAIN, declared effects, and a guarantee matrix are
  more work than a conventional stdlib op. This is the *point* (it is the product), but it must stay
  ergonomic or it discourages use — an explicit §8 design tension (the M-344/RFC-0012 "honesty's
  verbosity" lesson applies to the library, not just the surface).
- **Migration risk.** Self-hosting before the surface is ready would be a forcing function in the wrong
  direction; the M-502 gate exists to prevent a premature, dishonest "self-hosted" claim (VR-5).

## 6. Rationale & alternatives

- **Why one `std` phylum, not many independent libraries?** A single content-addressed `phylum` with
  independently-importable nodules gives coherence (one contract, one matrix format, one EXPLAIN style)
  without forcing an all-or-nothing dependency — the best of both (rejected: a constellation of unrelated
  packages, which would let the honesty contract drift per-library).
- **Why split Tier A / Tier B by *intent*?** It makes the roadmap legible (differentiators are
  first-class, commons are table-stakes) without implying Tier B is lower-quality — both meet §4.1.
  (Rejected: a flat alphabetical list, which buries what makes the library Mycelium's.)
- **Why Rust-first, not Mycelium-first?** ADR-007: the trusted toolchain is Rust; the surface isn't
  self-hosting yet. Building Mycelium-first now would be building on sand (and would have no reference to
  differential against). The migration path (§4.6) is the honest route to "free of other languages."

## 7. Prior art

- **Rust `std`** (the `core`/`alloc`/`std` ring split; traits; `Result`/`Option`; the no-`std` floor) —
  the closest model for the ring layering and the errors-as-values ergonomics.
- **Python stdlib** ("batteries included") — the breadth target for Tier B, and a *cautionary* case
  (silent coercions, sentinel returns) the §4.1 contract explicitly forbids.
- **Go stdlib** (small, orthogonal, explicit error returns) — the orthogonality + explicit-error ethos.
- **OCaml / Haskell** (value semantics, total functions, type-class/trait organization) — the
  immutable-by-default + totality posture.
- **Mycelium's own corpus** — the differentiator modules are *not* borrowed: each is the library form of an
  Accepted RFC/ADR (RFC-0002 swap, RFC-0003/0009 VSA, RFC-0005 selection, RFC-0008 runtime, RFC-0013/0014
  diagnostics/recovery, ADR-010 numerics, ADR-013 spore). The prior art there is the research base those
  documents already cite (T0/T1/T2 records).

**Discharged (2026-06-17) — `research/08-honest-stdlib-prior-art-RECORD.md`.** The standing
pre-ratification grounding obligation is met: the cross-language stdlib comparison (Rust/Python/Go/OCaml
module sets → **T8.1–T8.4**) and the "honest stdlib" prior art (refinement-typed / verified / effect-tracked
standard libraries → **T8.5–T8.7**) are traced into the evidence base. The record grounds the §4.4 Tier-B
taxonomy as the *consensus table-stakes spine* (four mature stdlibs), the §4.2 ring layering + §8-Q6
`std-sys` split as the Rust `core`/`std` precedent, and the §4.1 honesty contract (C2/C4/C6) against the
verified-stdlib prior art — while flagging the 4-point honest-degradation lattice (Empirical/Declared rungs)
as Mycelium's **novel, precedent-free** contribution owing its own soundness argument. Findings are tagged
Empirical/Declared, never Proven (VR-5). This clears the §8 gate for ratification.

## 8. Unresolved questions (resolved/deferred at ratification — 2026-06-17)

> **Ratification dispositions (2026-06-17, maintainer; DN-07; append-only).** Each question below carries
> its **→ Resolution**. Q1/Q2/Q5/Q6 and the `BF16→F32` FLAG are **resolved**; Q3 and Q4 are
> **deferred-with-direction** (neither blocks the contract+taxonomy ratification). The questions are kept
> verbatim for the record (append-only); the Resolution lines are the ratified decisions, grounded in
> DN-07 and `research/08-honest-stdlib-prior-art-RECORD.md` (§7). The original bullets read "FLAGGED —
> resolve before ratification"; that gate is now cleared.

- **(Q1) The exact v0 module set + priority order.** §4.3/§4.4 is the *proposed* taxonomy; which modules
  are v0 vs deferred, and their priority, needs maintainer sign-off. (The five M-346 early candidates —
  `diag`/`collections`/`numerics`/`vsa`/`io+serialize` — are the safe v0 floor; the rest are proposed.)
  - **→ Resolution (RESOLVED):** ratify the **full 23-module taxonomy** as the v0 *scope* (all 23 specs are
    `Draft` with no owned-surface conflict — stdlib README §5; grounded as table-stakes in T8.1–T8.4). The
    **M-346 five-candidate floor** (`diag`/`collections`/`numerics`/`vsa`/`io+serialize`) is sequenced
    first, and **`diag`/`recover` lead** the Rust→Mycelium migration (§4.6, the most honesty-load-bearing).
- **(Q2) Naming.** Is the phylum `std`? Do module names match the capability-crate names (`mycelium-vsa`
  → `std.vsa`)? The fungal lexicon (DN-02/06) may prefer themed names for some modules — a DN-level
  decision, not settled here.
  - **→ Resolution (RESOLVED — DN-level lexicon call):** the phylum is **`std`**; module names **mirror the
    capability-crate names** (`mycelium-vsa → std.vsa`, `mycelium-swap → std.swap`) for traceability, using a
    themed lexicon name **only** where the corpus already has one (`spore`, `runtime`/`colony`). `core` and
    `error` agree the **single error-value identifier** (DN-03's one-name-per-term rule). Recorded as an
    append-only lexicon amendment (DN-02/03/06 discipline).
- **(Q3) Ergonomics vs the contract (tension A).** How much of the tag/EXPLAIN/effect machinery is
  *implicit-by-default-but-inspectable* (the RFC-0012 ambient lesson) vs always-explicit at the call site?
  Load-bearing for adoption; needs a design pass per ring.
  - **→ Resolution (DEFERRED-WITH-DIRECTION):** the **direction is accepted** — adopt the **RFC-0012
    ambient-representation precedent** library-wide (implicit at the call site, but always reified, queryable,
    and EXPLAIN-able; never a black box, C3). The per-module resolution (~10 specs FLAG this; stdlib README
    §5) is discharged as **one scheduled per-ring design pass** (a named follow-on task, M-540), not seven
    per-module answers. This is the chief genuine open design question, but it does **not** block ratifying
    the contract + taxonomy.
- **(Q4) `runtime`/`colony` (M-521) sequencing.** The fungal runtime surface depends on the RFC-0008
  constructs activating (Phase 7). Does it live in `std` at all, or in a separate `runtime` phylum gated on
  Phase 7? FLAGGED cross-phase dependency.
  - **→ Resolution (DEFERRED — Phase-7 gate):** home the runtime surface in a **separate `runtime` phylum**
    (or gated sub-phylum) so pure `std` carries no inactive surface, activated **construct-by-construct at the
    Phase-7 gate** as the RFC-0008 constructs land. Placement is deferred; the contract + taxonomy stand
    without it. (`runtime`/`colony` stays *reserved-not-active* vocabulary until then — Glossary ⟂.)
- **(Q5) The migration differential's bar.** What exactly must a self-hosted module match (observable
  results only? tags + EXPLAIN bit-for-bit?) for M-502/§4.6 graduation? Ties to NFR-7's definition for
  library code.
  - **→ Resolution (RESOLVED — jointly with M-502):** a **two-level bar** — (1) **observable-result
    equivalence through the M-210 checker** as the universal floor (matches the existing NFR-7 differential),
    and (2) **tag + EXPLAIN equivalence** as a **stronger, per-module-declared** bar for honesty-load-bearing
    modules whose tags/certificates are themselves observable (`swap` certificates, `select` EXPLAIN,
    `numerics` bounds). M-502 makes the verdict *checkable*; it stays honestly **not established** until a
    concrete L3 surface can author a stdlib module (the §10/A2 KC-2 gate, below).
- **(Q6) `wild`/FFI surface.** Some Tier B (`io`/`fs`/`time`/`rand`) needs OS facilities → `wild` (ADR-014).
  What is the minimal audited FFI floor, and does it live in `std` or a separate `std-sys` phylum so the
  pure-safe `std` stays certified leak-free (LR-9)?
  - **→ Resolution (RESOLVED):** split the audited `wild` floor into a **separate `std-sys` phylum** so pure
    `std` stays leak-free *by construction* (LR-9) and can publish a `wild`-free certification badge (§9).
    The three call sites (`fs` syscalls, `rand` platform entropy, `math` libm transcendentals — stdlib README
    §5; the Rust `core`/`std` precedent, T8.1) consolidate into one `std-sys` boundary. The **minimal audited
    FFI inventory** is a named follow-on deliverable (M-541).
  - **→ Resolution refinement (M-661 — `@std-sys` is a header *attribute*, not a naming convention; the
    `wild` gate; implemented Rust-first, pending ratification).** The `std-sys` boundary is carried by an
    **explicit nodule-header marker `@std-sys`** (surface: `nodule std.sys.fs @std-sys`), *not* by the
    nodule's *name* — a `std.sys.*`-named nodule without the marker is **not** the FFI floor, and a marked
    nodule of any name **is**. The **gate** the L1 reference frontend now enforces: a `wild` block (the
    denied-by-default unsafe escape — LR-9/S6; ADR-014) is legal **iff** (a) it is inside a `@std-sys` nodule
    **and** (b) its enclosing `fn` declares the **`ffi` effect** (`!{ffi}` — `wild` is the `ffi` effect
    *source*, binding to RFC-0014/M-660 coverage). A `wild` in a non-`@std-sys` nodule is a **hard
    `CheckError`** (a never-silent refusal — G2, *not* a lint); a `wild` whose fn omits `!{ffi}` is the
    M-660 under-declaration refusal. The `wild` **body is the trusted/opaque FFI escape — NOT recursively
    type-checked** (it conforms to the expected type by ascription; it is *audited*, not *verified* —
    VR-5/ADR-014), so it requires an expected type (a synthesis position refuses with "ascribe the `wild`
    block's result type"). **Execution stays staged:** with no FFI host in v0, a `wild` block *type-checks +
    gates + is audited* now but does **not run** — it elaborates to an explicit `Residual` (a future
    capability), consistent with the M-657/659/660 staging. Guarantee on the gate: **`Declared`** (a
    structural + audited context gate, not a theorem). The lexical `// SAFETY:`-presence audit (`myc-sec`
    `audit_wild`, ADR-014) is **orthogonal and unchanged** — it inventories + justification-checks every
    `wild`; this typechecker gate is the *context* check. Implemented Rust-first in `crates/mycelium-l1/`
    (`ast.rs` `Nodule.std_sys`; `lexer.rs`/`token.rs` the atomic `@std-sys` token; `parse.rs` `parse_nodule`;
    `checkty.rs` `Cx::check_wild` + the effect-coverage `ffi` source; `elab.rs` the staged `Residual`),
    **pending ratification** — this refinement is append-only and supersedes nothing.

- **(Residual cross-module FLAG, not an original §8 item) — the `BF16→F32` placement.** The one unsettled
  *owned-surface* seam (stdlib README §5; `swap` §7-Q3 / `cmp` §7-Q2): which module owns the *lossless
  reverse* `BF16→F32` widening?
  - **→ Resolution (RESOLVED):** lift it to **`cmp`/`convert`** as lossless float widening (a certificate
    certifies a *bounded/lossy* change; a lossless widen needs none), leaving **`swap` to own only the
    certified/lossy direction**. No op is double-owned; this closes the last open owned-surface seam.

> **Honesty note (the planning analogue of G2).** Where a module's intent is **not** yet grounded in the
> corpus, it is **FLAGGED here as an open question, never invented** into a false-confident task. The Tier-B
> commons are grounded in *universal* stdlib expectation + the §4.1 contract (their shape is table-stakes);
> any module whose *Mycelium-specific* behaviour is unclear is a §8 question, not a silent design choice.

## 9. Future possibilities

- **Self-hosted `std`** — the whole library in Mycelium-lang (the M-346 "free of other languages" end
  state), with the Rust reference retired once every module passes its migration differential.
- **A `wild`-free certification badge** — a `phylum` with no `wild` blocks is leak-free *by construction*
  (LR-9); the stdlib can publish which modules clear it, as a first-class honesty signal.
- **Community modules** under the same §4.1 contract — the contract + guarantee-matrix format is the
  mechanism that lets third-party `phylum`s be held to Mycelium's honesty bar, not just the first-party
  `std`.
- **A `no-std`-equivalent floor** — Ring 0 alone (the honest value model) for embedded / kernel-adjacent
  use, mirroring Rust's `core`.

## Meta — changelog

- **2026-06-25 — Locator erratum (post corpus-alignment audit; Status unchanged — Enacted).** The Status header and the 2026-06-21 Enacted entry below cite "**all 25 `mycelium-std-*` crates landed**"; the tree now has **26** `mycelium-std-*` crate directories — the extra is **`mycelium-std-sys-host`**, which post-dates enactment. Stale-by-one count only (the 23-module v0 taxonomy + guarantee matrices are unchanged); the "25" figures are left intact as the at-enactment record. Append-only.
- **2026-06-22 — §8-Q6 refinement: the `@std-sys` header marker + the `wild` gate landed Rust-first (M-661, E7-1; append-only, pending ratification).** The audited FFI floor is now *enterable*: `@std-sys` is an explicit **nodule-header attribute** (`nodule std.sys.fs @std-sys`), **not** a naming convention, and a `wild` block (LR-9/S6; ADR-014) type-checks **iff** it is in a `@std-sys` nodule **and** its `fn` declares the **`ffi` effect** (`!{ffi}` — `wild` is the `ffi` source, binding to RFC-0014/M-660); a `wild` elsewhere is a **hard `CheckError`** (G2, not a lint), and an undeclared `ffi` is the M-660 coverage refusal. The `wild` body is the **trusted/opaque** FFI escape — not recursively type-checked (audited, not verified — VR-5/ADR-014) — so it needs an ascribed/expected type (synthesis refuses). **Execution stays staged** (no FFI host in v0 → an explicit elaboration `Residual`); guarantee on the gate **`Declared`**. The `myc-sec` `// SAFETY:`-presence audit is orthogonal and unchanged. Implemented in `crates/mycelium-l1/` (lexer/`token`/`parse`/`ast`/`checkty`/`elab`) + `mycelium-l1` tests + conformance `accept/18-wild-std-sys.myc`; verified by `cargo fmt`/`clippy -D warnings`/`cargo test -p mycelium-l1` green. This refinement pins the §8-Q6 resolution; it is append-only and supersedes nothing. (DN-14 §3 row 9 flips to *conditionally present (audited, std-sys context; type-checks + gates; execution staged)*.)
- **2026-06-21 — Accepted → Enacted (M-648 editorial sweep).** All 25 `mycelium-std-*` crates landed Rust-first (M-501–M-534, M-540, M-541): the 23-module Tier-A/Tier-B guarantee matrices are asserted in tests (1 883 + 722 + 230 tests across the wave), never-silent G2 holds across all modules, and the per-op `EXPLAIN` obligation is met. The Mycelium-lang self-hosting migration half (M-502, Phase-5-C) remains open per KC-2 gate ruling and does not block enactment of the Rust-first scope. Append-only.
- **2026-06-20 — Erratum (§4.2 ring layering; maintainer-authorized, stdlib ratification pass).** Reconciled an
  internal inconsistency: §4.2 listed `spore` under **Ring 2** while §4.3 (Tier A table), the stdlib index, and
  `spore.md` place it Tier A / **Ring 1**. `spore` is an ergonomic capability surface over the landed
  `mycelium-spore` + `mycelium-content` crates (a certificate/EXPLAIN consumer, no new trusted code — KC-3),
  which is the Ring-1 definition; §4.2's parenthetical was the outlier. Corrected §4.2 to file `spore` under
  Ring 1 (added to the capability-surface list; removed from the Ring-2 parenthetical). Corrigendum only —
  **not** a decision reversal (the Tier-A taxonomy of §4.3 is unchanged); the Accepted status stands. This
  unblocks `spore.md`'s §7-Q1 ratification. Append-only; no kernel change (KC-3).
- **2026-06-17 — Draft → Accepted (maintainer ratification; M-501/DN-07).** The maintainer ratified the
  RFC-0016 core: the §4.1 per-op **contract** (C1–C6), the §4.2 **ring layering**, the §4.3/§4.4
  **Tier-A/Tier-B taxonomy** (full **23-module v0 scope**), the §4.5 **guarantee-matrix** obligation, and the
  §4.6 **Rust-first → Mycelium-lang migration** order — each corroborated by 23 `Draft` specs with no
  owned-surface conflict (stdlib README §5) and grounded as table-stakes in `research/08` (T8.1–T8.4). **§8
  dispositions** (recorded inline in §8): **Q1** RESOLVED (full taxonomy; M-346 five-candidate floor first;
  `diag`/`recover` lead migration); **Q2** RESOLVED (phylum `std`; crate-mirrored module names; one
  `core`↔`error` error-value name — a DN-level lexicon amendment); **Q5** RESOLVED (two-level differential
  bar — M-210 observable-result floor + per-module tag/EXPLAIN equivalence, with M-502); **Q6** RESOLVED
  (`std-sys` phylum split for the audited `wild` floor; FFI inventory → M-541); the **`BF16→F32`** residual
  FLAG RESOLVED (lossless reverse → `cmp`/`convert`; `swap` keeps the certified/lossy direction). **Q3**
  DEFERRED-WITH-DIRECTION (accept the RFC-0012 ambient direction; schedule one per-ring ergonomics design
  pass → M-540) and **Q4** DEFERRED (the `runtime` phylum's Phase-7-gated placement) — neither blocks the
  contract+taxonomy. The **§7 grounding obligation is discharged** (`research/08-honest-stdlib-prior-art-RECORD.md`,
  T8.1–T8.7), clearing the standing gate. The concrete L3 *authoring* surface stays **KC-2-gated** (RFC-0006
  §10; self-hosting capability #3, M-502) — so the Mycelium-lang *migration half* of M-510…M-520 waits, while
  the Rust-first specs/implementations proceed. Append-only status transition; supersedes the Draft below; no
  kernel change (KC-3). Lineage: M-346 → RFC-0016 (now Accepted) → per-module specs + M-540/M-541 follow-ons.
- **2026-06-17 — Draft (Proposed).** Captures the standard-library scope the M-346 epic anchors and M-501's
  acceptance names: the **per-op contract** every stdlib op must meet (C1–C6 — never-silent, honest tags,
  EXPLAIN, content-addressed, above-the-kernel, declared/bounded effects), the **module taxonomy** split
  into Tier A *differentiator* modules (swap/numerics/vsa/ternary/dense/select/diag/recover/runtime/spore/
  content — each the library form of an Accepted RFC/ADR) and Tier B *common* modules (collections/text/
  math/iter/error/io/fs/serialize/time/rand/cmp+convert/fmt/testing — table-stakes, held to the same
  contract), the **ring layering** (KC-3-preserving), the **per-module guarantee matrix** obligation, and
  the **Rust-first → Mycelium-lang migration** (dogfooding, gated by the M-502 readiness verdict). Six §8
  questions FLAGGED (module set/priority, naming, ergonomics-vs-contract, runtime sequencing, the migration
  bar, the `wild`/FFI floor) and the §7 prior-art `research/` record obligation recorded — both must clear
  before ratification. Decomposes into Phase-5 tasks M-510…M-534 (`docs/planning/phase-5.md`). No code with
  this draft; ratification + per-module folding is the maintainer's append-only decision (M-501). No kernel
  change (KC-3). Lineage: M-346 (stdlib epic) → **RFC-0016** (this scope) → per-module RFCs/tasks. Append-only.
