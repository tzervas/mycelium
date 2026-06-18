# Mycelium — Glossary & Term Index

**Purpose:** the single, separately-maintained reference for Mycelium's **unique terminology** — the
fungal lexicon and the honesty/architecture concepts that are specific to this project. RFCs and notes
*use* these terms and *define* them normatively in their own sections; this document **collects** them
so a reader (human or machine) never has to hunt across the corpus, and so the naming stays coherent.

**How this is organized** (per maintainer direction, 2026-06-16): a summarized **§1 Index** (one line
per term, pointing into the detail) and a fleshed-out **§2 Glossary** (definition, layer, relationships,
the *defining* doc, and a usage note per term). The Index points to the Glossary subsection.

**Authority & grounding.** This document is a **synthesis, not a source of truth** — each entry cites
the doc that *ratifies* it (an RFC/ADR/DN). On any conflict, the cited normative doc wins; fix the
glossary, never the other way around. The naming law and the three-test gate live in **DN-02**;
amendments in **DN-03** (surface/runtime names) and **DN-06** (`phylum`/`nodule`/`colony`). The terse
catalog with mnemonic forms is `docs/notes/Lexicon-Reference.md`.

**Status:** Living. Add a term when it is ratified; never silently rename one (a rename is a
supersession — DN-02's append-only law). Reserved-not-active terms are marked **⟂**.

---

## 1. Index

Alphabetical; the **Detail** column names each term's §2 subsection. *(L)* = part of the fungal lexicon;
*(H)* = honesty/architecture concept. **⟂** = reserved vocabulary, not yet active syntax.

| Term | One-line sense | Layer | Detail |
|---|---|---|---|
| `backbone` ⟂ *(L)* | a declared high-bandwidth transport path (placement artifact) | Runtime | §2.1 |
| `colony` ⟂ *(L)* | a **dynamic** runtime grouping of active `hypha` | Runtime | §2.2 |
| `cyst` ⟂ *(L)* | a content-addressed checkpoint of a dormable computation | Runtime | §2.3 |
| Declared *(H)* | weakest guarantee tag: asserted, always flagged | Formal | §2.4 |
| Empirical *(H)* | guarantee tag from trials (≥1, with a method) | Formal | §2.4 |
| EXPLAIN *(H)* | the mandate that selections/conversions are inspectable | Formal | §2.5 |
| Exact *(H)* | strongest guarantee tag: exact, no error | Formal | §2.4 |
| `forage` ⟂ *(L)* | adaptive placement as a reified RFC-0005 policy | Runtime | §2.6 |
| `fuse` ⟂ *(L)* | lawful state fusion: semilattice merge, meet-composed `Meta` | Runtime | §2.7 |
| `graft` ⟂ *(L)* | a capability contract with external infrastructure | Runtime | §2.8 |
| guarantee lattice *(H)* | `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` | Formal | §2.4 |
| `hypha` ⟂ *(L)* | a single structurally-scoped concurrent execution unit | Runtime | §2.9 |
| `matured` *(L)* | a compiled-and-frozen **scope** (nodule/phylum/program; header/manifest, not a fn modifier) | Surface | §2.10 |
| `mesh` ⟂ *(L)* | gossip/pub-sub overlay with honest probabilistic guarantees | Runtime | §2.11 |
| `Meta` *(H)* | the metadata a `Value` carries (guarantee, provenance, …) | Formal | §2.12 |
| never-silent (G2) *(H)* | no silent failure/swap; refusal is always explicit | Formal | §2.13 |
| `nodule` *(L)* | the **basic** static unit (replaces "module") | Surface | §2.14 |
| `phylum` ⟂ *(L)* | a content-addressed **library-scale** static unit | Surface | §2.15 |
| Proven *(H)* | guarantee tag from a theorem with *checked* side-conditions | Formal | §2.4 |
| `reclaim` ⟂ *(L)* | supervision-tree reclamation of stale runtime units | Runtime | §2.16 |
| `Repr` *(H)* | a value's representation (binary/ternary/dense/VSA) | Formal | §2.12 |
| `spore` *(L)* | a content-addressed deployable / reconstruction manifest | Deploy | §2.17 |
| `substrate` *(L)* | an affine external resource (consumed exactly once) | Surface | §2.18 |
| `swap` *(L/H)* | a certified, never-silent representation change | Formal | §2.19 |
| `thaw` *(L)* | de-maturation: keep one def interpreted inside a `matured` scope | Surface | §2.10.1 |
| `tier` ⟂ *(L)* | an execution-mode switch (interpreted ↔ native) | Runtime | §2.20 |
| `Value` *(H)* | an immutable `(Repr, Payload, Meta)` — the only thing that moves | Formal | §2.12 |
| `wild` *(L)* | the denied-by-default unsafe block (FFI / raw memory) | Surface | §2.21 |
| `xloc` ⟂ *(L)* | explicit, fallible, `Meta`-preserving value movement | Runtime | §2.22 |

**Plural/inflected forms** (prose only; the reserved word is the singular): `phylum`/**`phyla`**,
`nodule`/`nodules`, `colony`/`colonies`, `hypha`/**`hyphae`**.

---

## 2. Glossary

### 2.1 `backbone` ⟂
A **declared or promoted high-bandwidth transport path** — a placement-policy artifact, semantics-free
(it changes performance, never meaning). Runtime tier. **Defining doc:** RFC-0008 §4.5 (T4.3; invariant
RT3). Reserved vocabulary, not active syntax.

### 2.2 `colony` ⟂
A **dynamic runtime grouping of active `hypha`** — a cooperating set of concurrent tasks under a shared
scope, supervision policy, or deployment context. The dynamic counterpart to the static `phylum`/
`nodule`. **Defining doc:** DN-06 (the term); RFC-0008 §4.7 (the structured-concurrency scope it names);
realized in code by `mycelium-mlir::runtime`'s `Colony` (the structured `Scope`). **History:** DN-02 §2
originally used `colony` for the static module; DN-06 reassigned it to this dynamic meaning (higher
T-map fidelity) and moved the static role to `nodule`. Reserved as surface syntax until its construct
lands; the runtime concept is live. *Usage:* "a `colony` of `hypha` supervised by `reclaim`."

### 2.3 `cyst` ⟂
A **content-addressed checkpoint** of a dormable computation — encystment is the dormant-resumable form
(values in scope + the continuation by content hash + the `Meta` to resume honestly). Determinism makes
resume-and-replay sound. Runtime tier. **Defining doc:** RFC-0008 §4.4 (T4.4; invariant RT2). Reserved.

### 2.4 The guarantee lattice — Exact ⊐ Proven ⊐ Empirical ⊐ Declared *(H)*
The **honesty lattice**: every accuracy/guarantee claim is tagged per-model/per-operation on this
four-point order.
- **Exact** — exact, no error (a lossless operation).
- **Proven** — backed by a theorem whose **side-conditions are *checked***; allowed *only* then.
- **Empirical** — backed by trials (≥1, with a named method) — never evidence-free.
- **Declared** — merely asserted; **always flagged**. The honest floor for anything unproven.

**The rule:** *downgrade to stay honest; never upgrade without a checked basis* (VR-5). **Defining doc:**
RFC-0001 (the lattice + `Meta`); the honesty rule, CLAUDE.md / Project Foundation. *Usage:* "a
substituted fallback is at most `Declared`; a swap certificate may carry `Proven`."

### 2.5 EXPLAIN *(H)*
The mandate that every **selection, conversion, or approximation is reified, inspectable, and
explainable** — "no black boxes." One can always answer *why* a representation/policy/route was chosen.
**Defining doc:** RFC-0005 (selection + EXPLAIN); SC-3 (transparent control). *Usage:* "the placement
policy is `EXPLAIN`-able — the deciding artifact is total and inspectable."

### 2.6 `forage` ⟂
**Adaptive placement** — *where* work runs — as a reified RFC-0005 policy with mandatory EXPLAIN (the
third site of the one selection mechanism). Placement affects performance, never meaning (the Legion
lesson). Runtime tier. **Defining doc:** RFC-0008 §4.5 (T4.3; invariant RT3). Reserved.

### 2.7 `fuse` ⟂
**Lawful state fusion** (anastomosis): merging two `hypha`'s state *only* through declared
commutative/associative/idempotent (semilattice) merge operations — the CRDT sufficient condition for
convergence — with the merged value's guarantee the **meet** of the inputs (payload joins up, honesty
meets down). Runtime tier. **Defining doc:** RFC-0008 §4.5/RT6 (T4.2). Reserved.

### 2.8 `graft` ⟂
A **capability contract with external infrastructure**; the capability is an affine `substrate` handle.
Runtime tier. **Defining doc:** RFC-0008 §4.5 (T4.3/T4.5; invariant RT4). Reserved.

### 2.9 `hypha` ⟂
A **single structurally-scoped concurrent execution unit** — a checked computation over immutable
values, living inside a scope that outlives none of its children (structured concurrency). Hyphae never
share state; they exchange `Value`s. The signature concept of the runtime; many `hypha` make a `colony`.
Runtime tier. **Defining doc:** RFC-0008 §4.5 (T4.1; invariants RT1/RT2/RT7); DN-03 §4 (the name).
Reserved.

### 2.10 `matured`
A **scope** (`nodule`/`phylum`/program) **promoted from interpreted to compiled-and-frozen** (stable,
persistent, verified) — it has "grown to a hardened, persistent stage." Every definition reachable in a
matured scope must be `total` (RFC-0007 §4.5, quantified over the scope) and AOT-eligible (RFC-0004 §4),
except those marked `thaw`. **Declared at scope, not per definition (RFC-0017, 2026-06-18):** a
`nodule`/`phylum` via its header (`// @matured: true`), a program/package via its manifest — `matured`
is a header/manifest key and a reserved word, **not** a `fn` modifier (`matured fn …` is retired). Surface tier.
**Defining doc:** RFC-0017 (scope + the surface forms); RFC-0004 §4 / RFC-0007 §4.5 (the gate, unchanged);
DN-02 §7 (the name, over `hardened`/`sclerotium`). *Usage:* "a `matured` nodule takes the AOT path."

### 2.10.1 `thaw`
The **inverse of maturation**: a `thaw fn f` keeps **one** definition **interpreted** inside an
otherwise-`matured` scope (the rare iterate/debug escape hatch — RFC-0017 §4.3). `matured` is
"compiled-and-**frozen**" (§2.10); `thaw` returns *one* definition from frozen to the live, interpreted
state. It only ever de-matures (a no-op flagged by lint in an unmatured scope), weakens **no** advertised
honesty tag (only the execution path/performance changes — the guarantee lattice is path-independent,
NFR-7), and is never-silent + `EXPLAIN`-able. Conventional-clearest (the themed *germinate* is taken by
spore-germination, ADR-013). Active Surface tier. **Defining doc:** RFC-0017 §4.3/§5 (the name, DN-02
three-test gate); DN-03 (changelog pointer). *Usage:* "`thaw fn experimental_shear(…)` — stays
interpreted while the nodule around it is matured."

### 2.11 `mesh` ⟂
The **common mycorrhizal network**: gossip/pub-sub overlay coordination whose delivery/convergence
guarantees are **probabilistic** — so they carry a `ProbabilityBound` (δ) with an honest basis, never
claimed reliable. Runtime tier. **Defining doc:** RFC-0008 §4.3/§4.5 (T4.2; invariant RT5). Also a
diagnostic **route** sink (RFC-0013 §8). Reserved as runtime syntax.

### 2.12 `Value`, `Repr`, `Meta` *(H)*
The core value model. A **`Value`** is an immutable triple `(Repr, Payload, Meta)` — the *only* thing
that ever crosses a `hypha`/channel/node boundary (RT1). **`Repr`** is its representation family
(binary / balanced-ternary / dense / VSA). **`Meta`** is the metadata it carries: its guarantee tag
(§2.4), provenance, and bounds — and it travels with the value across every boundary (WF5). **Defining
doc:** RFC-0001 (the value model, `Meta`, well-formedness WF1–WF8).

### 2.13 never-silent (G2) *(H)*
The substrate's identity invariant: **no silent failure, no silent swap, no silent approximation.**
Every representation change is certified and lexically visible; every out-of-range/failure is an
explicit `Option`/error/refusal that *propagates* — never swallowed. The operational form recurs as I1
(diagnostics/recovery are *additive*, never substitutive). **Defining doc:** Project Foundation (G2);
RFC-0006 S1–S6; RFC-0013/0014 I1. *Usage:* "routing never gates propagation — never-silent holds."

### 2.14 `nodule`
The **basic unit of static organization** inside a `phylum` — definitions, types, implementations. The
on-brand replacement for the generic "module" (the phonetic bridge module→nodule is intentional).
Content-addressed (ADR-003). Surface tier (L2). **File/dir convention (DN-06):** a file's nodule status
is declared in a **header comment**, *not* in the filename/path (which stay conventional — no `nodule`
bloat). **Defining doc:** DN-06; RFC-0006 (L2 surface). **History:** replaces DN-02's static `colony`;
the keyword migration (lexer/parser/AST/grammar/conformance) was **executed by M-358 (2026-06-16)** — the
L1 surface keyword is now `nodule`, and `phylum`/`colony` are reserved-not-active. *Usage:* `// nodule:
geometry.shapes` as the first line of a source file (recognised by `mycelium_l1::parse_nodule_header`).

### 2.15 `phylum` ⟂
A **content-addressed, versioned, library-scale** static unit: a coherent collection of `nodule`s with
a defined public surface — the primary unit of static organization, distribution, and dependency (≈ a
crate/package). A `phylum`'s published artifact is a `spore`. Surface tier; content-addressed at the
phylum level (ADR-003). **Defining doc:** DN-06; RFC-0006. Reserved-not-active until its construct lands.

### 2.16 `reclaim` ⟂
**Supervision-tree reclamation of stale *runtime units*** (never memory — LR-9 makes memory reclamation
automatic; a memory-`reclaim` would contradict it). The home of bounded-cascade supervision
(max-restart-intensity). Runtime tier. **Defining doc:** RFC-0008 §4.5/RT7 (T4.5); RFC-0008 §4.7 (the
bounded cascade); DN-03 §4 (the name + scope clarification). Reserved as syntax; the supervision
mechanism is live (`mycelium_interp::supervise::Supervisor`).

### 2.17 `spore`
A **content-addressed deployable unit** — code + initial values + manifest — that germinates into a
running `hypha`; the published artifact of a `phylum`. The reconstruction-manifest sense (regrow a value
from its recipe) is the degenerate single-value case. Deploy tier. **Defining doc:** DN-02 §2 (the
surface term; schema stays `reconstruction-manifest`); RFC-0003 §6; ADR-013 / RFC-0008 §4.4. *Usage:*
"ship a `spore`; it germinates into a `colony`."

### 2.18 `substrate`
An **affine external resource** (the LR-8 `Resource` kind) — the material a fungus consumes to grow,
**used up exactly once**. Affinity *is* single-consumption; the metaphor teaches the linearity. Surface
tier. **Defining doc:** DN-02 §2 (Ratified); RFC-0006 LR-8. *Usage:* "a file handle is a `substrate` —
consumed once, never aliased."

### 2.19 `swap` *(L/H)*
The **certified, never-silent representation change** — Mycelium's signature operation. A `swap` is
lexically visible (`swap(x, to: …, policy: …)`), carries a certificate, and is *never* a runtime's
silent prerogative; out-of-range is an explicit refusal. A native Mycelium term (not borrowed), it reads
the same in IR, RFCs, and surface. **Defining doc:** RFC-0001 §4.5 (the `Swap` node); RFC-0002 (the swap
certificate + legal pairs). *Usage:* "dense↔sparse is a `swap` (S1), not an `ExecutionMode` switch."

### 2.20 `tier` ⟂
An **execution-mode switch** (interpreted ↔ native) — RFC-0004's existing `ExecutionMode` story, made
observable-equivalent by NFR-7 (the JIT-tiering precedent). *Not* a representation change (that is a
`swap`). Runtime tier. **Defining doc:** RFC-0008 §4.5 (T4.6; invariants RT2/S1). Reserved.

### 2.21 `wild`
The **denied-by-default, lexically-marked unsafe block** — the only place raw FFI / foreign memory is
reachable. Safe Mycelium expresses no leak (LR-9); a `phylum`/`nodule` with no `wild` blocks is
*certified leak-free by construction*. Surface tier. **Defining doc:** DN-02 §5 (Ratified); RFC-0006
LR-9/S6. *Usage:* "`wild { foreign_decode(ptr, len) }` — audited, opt-in, unreachable from safe code."

### 2.22 `xloc` ⟂
**Trans-location**: explicit, fallible, `Meta`-preserving **value movement** between nodes, with
backpressure. It is *not* a representation change (a differing wire format is a `swap`, visible). Runtime
tier. **Defining doc:** RFC-0008 §4.5 (T4.3; invariants RT1/RT4). Reserved.

---

## Meta — changelog

- **2026-06-16 — Created.** The dedicated terminology reference the DN-06 supplement asked for: a
  summarized **§1 Index** (pointing into the detail) over a fleshed-out **§2 Glossary** (definition,
  layer, relationships, defining doc, usage per term), covering the ratified fungal lexicon (DN-02/03/06)
  and the core honesty/architecture concepts (the guarantee lattice, never-silent, EXPLAIN, Value/Repr/
  Meta, swap). A *synthesis* that cites each term's normative source — on conflict the source wins.
  Maintained separately from the RFCs (per maintainer direction, 2026-06-16). Living; reserved-not-active
  terms marked ⟂. Append-only in spirit (a rename is a supersession, never an edit — DN-02's law).
