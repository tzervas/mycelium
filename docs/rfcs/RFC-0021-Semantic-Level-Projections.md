# RFC-0021 — Semantic-Level Projection Framework

| Field | Value |
|---|---|
| **RFC** | 0021 |
| **Status** | **Draft** (G11 is flagged exploratory — "may not be ergonomically viable"; ratification is a maintainer decision, append-only) |
| **Type** | Foundational / normative (once Accepted) |
| **Date** | June 18, 2026 |
| **Depends on** | RFC-0001 §4.6/ADR-003 (content-addressed identity — the primary grounding for all projection semantics); RFC-0006 §3 (L3 projection layer), §4.1 S2/S4/S6 (honest-tags surface, inspectable elaboration, AI-independence); RFC-0007 §4.1 (L1 node budget); ADR-006 (no black boxes); FR-C1 (the exploratory projections requirement); FR-S5 (dual-intelligibility toolchain surface); G11 (semantic-level, not notational, projections — exploratory, ergonomics unverified); G10 (semantic-feedback loop / edit-to-fix recovery) |
| **Coupled with** | RFC-0020 — *Text Grammar (v0 committed surface)* (the committed text grammar is itself one projection among several; cited by number; no standalone file yet); DN-09 (KC-2 verdict = proceed; M-380 opened co-equally); M-380 (the design-active implementation epic); phase-3.md E3-1/E3-2 |

---

## 1. Summary

The KC-2 verdict (DN-09) selected **text syntax + a co-equal structured-projection layer** as the L3
strategy. This RFC designs the projection layer (M-380, E3-1). Its central claim:

> A **projection** is a *total, inspectable function* from the L1/L2 node structure of a
> content-addressed definition (RFC-0001 §4.6/ADR-003) to a rendered surface. Identity is the
> content hash, not the rendered form. A projection is a **view** — it does not create a second
> source of truth and cannot change meaning.

The headline application is the **LLM-facing canonical projection** (FR-S5 dual intelligibility):
the same content-addressed definitions rendered in a grammar-in-context–friendly form with a
constrained-decoding hook, lifting the real-but-modest text-surface leverage the KC-2 run
established without discarding the committed text grammar.

The RFC is in **Draft** because the G11 ergonomics question — whether semantic-level projections
(projecting *meaning and structure*, not just reformatted text) are viable in practice — has not
been confirmed. Two research prompts (§9) must run before this moves to Accepted.

---

## 2. Motivation

### 2.1 The problem projections solve

The KC-2 run (DN-09 §2) showed *weak-but-recoverable* LLM leverage on the Mycelium text surface:
the best arm (7B model + examples primer) reached 70% eventual pass with a +30 pp edit-to-fix lift.
DN-09 §3.1 reads this as the surface being "real-but-modest" rather than collapsed — the
semantic-feedback loop (G10) recovers a large fraction of first-attempt failures.

DN-09 §3.1 frames the projection layer's job precisely: *"projections are not a hedge against
failure but a lever to lift a working surface."* The text grammar is committed and real; projections
are an improvement path on top of it, not a replacement.

Three concrete gaps the text grammar alone cannot close:

1. **Grammar-in-context saturation.** The KC-2 `examples` primer (a grammar-in-context anchor)
   was the dominant knob at 7B — two complete, non-answer programs roughly doubled first-attempt
   pass. But the primer is fixed-size; the context window available at generation time is not.
   An LLM-facing canonical projection can provide *structured, grammar-bound* context inline
   without burning tokens on repeated primer text.

2. **Constrained decoding gap.** The KC-2 run did not exercise constrained decoding (DN-09 §4).
   Grammar-constrained generation requires a machine-consumable grammar artifact paired with a
   projection that maps the decoding grammar to the canonical structure — the text grammar alone
   does not supply a decoding-grammar/projection pair suitable for plugging into a constrained
   decoder.

3. **Multi-audience rendering.** The same content-addressed definition may be most legible to a
   human in idiomatic Mycelium text and most legible to an LLM in a form that minimizes out-of-
   distribution token sequences. Both are valid renderings of the same underlying hash-identified
   structure (RFC-0001 §4.6/ADR-003). Without an explicit projection abstraction the toolchain
   must fork the definition, violating the single-source-of-truth discipline.

### 2.2 Why "semantic-level, not notational" (G11)

G11 distinguishes projections that reformat text (notational — a pretty-printer) from projections
that map *meaning-bearing structure* (semantic-level). The distinction matters because:

- A notational projection of novel syntax yields novel syntax in a different layout. It cannot, for
  example, render a `Swap` node as the familiar `::from()` idiom a pre-trained LLM has seen
  millions of times.
- A semantic-level projection has access to the L1/L2 node type, the guarantee tag, the elaboration
  path, and the `policy_used` record. It can render a `Swap{from: Binary{32}, to: Ternary{m},
  cert: C}` as an idiomatic construct whose shape a model already knows, while attaching the
  certificate as a structured annotation — retaining full honesty (S2) and inspectability (S4).

The *"may not be ergonomically viable"* qualifier in G11 is honest: building projections over the
full L1/L2 node set requires substantial tooling, and the ergonomics for the projection *author*
(writing and maintaining the mapping functions) are unvalidated. This RFC designs the abstraction
and defers the ergonomics verdict to the §9 research prompts.

### 2.3 Interaction with the committed text grammar (RFC-0020)

The v0 text grammar (RFC-0020, `docs/spec/grammar/mycelium.ebnf`) is the **committed L3 surface**
(RFC-0006 r5, DN-09 §3.1). This RFC does not supersede or compete with it: the text grammar is
**one projection** in the sense of §4.1 — it renders L1/L2 structure as the idiomatic Mycelium
text that humans write directly. Making this explicit has a practical benefit: the text grammar's
canonical formatter (M-142) becomes an instance of the projection machinery, sharing the same
dumpability and diffability requirements (S4/ADR-006), rather than a standalone tool with its own
invariants.

---

## 3. Guide-level explanation

### 3.1 A projection is a view over content-addressed definitions

Every definition in Mycelium is identified by a structural content hash (RFC-0001 §4.6/ADR-003).
Names are bindings to hashes, not identity. Formatting is already described as a projection in
RFC-0001 §4.6: *"two definitions differing only in … formatting have the same hash (formatting is
a projection, ADR-003; §4.8)."*

This RFC generalises that remark into a first-class abstraction. Concretely, a projection `P` is:

```
P : (L1-or-L2 node tree, ContentHash) → Surface
```

where `Surface` is one of: idiomatic Mycelium text; a structured/Unison-style tree view; a
machine-consumable JSON/S-expression form; a constrained-decoding grammar fragment; or any other
inspectable rendering.

The defining constraint is that `P` is **total** (every node in scope has a rendering), **pure**
(deterministic given the node and hash), and **dumpable** (the mapping rules are inspectable
artifacts, not black-box code — ADR-006/S4).

### 3.2 What a projection may and may not do

| Permitted | Forbidden |
|---|---|
| Render node structure in a different surface idiom | Change the meaning of a node |
| Elide syntactic boilerplate present in the text grammar | Elide a `Swap` node or make it invisible |
| Translate guarantee tags into a surface-idiomatic notation | Change a guarantee tag's value |
| Add structural annotations (e.g. inline types, EXPLAIN records) | Introduce a new semantic dependency not in the node |
| Provide a round-trip *back* to the source hash | Produce a rendering whose round-trip yields a different hash |

These constraints are the projection-level restatements of S6 (editor convenience, never a
semantic dependency), S2 (honest tags survive projection), and RFC-0001 §4.6/ADR-003 (the hash
is identity, not the rendered form).

### 3.3 The round-trip (edit) obligation

A projection that supports *editing* (not just viewing) carries an additional obligation: the
edited surface must be re-parseable into an L1/L2 node tree, and that tree must content-address
back to a valid (possibly new) hash. The hash of the edited result is the identity of the new
definition; it is not required to equal the original hash (an edit is a new definition), but the
round-trip must be lossless in the other direction (no edit is silently dropped or merged with
a prior definition). This is the Unison-style edit model: rename-a-binding → new hash → the
old hash persists in the name map until explicitly superseded.

A read-only projection (view-only, no edit path) carries no round-trip obligation; it must only
be dumpable and diffable (ADR-006/S4).

### 3.4 The LLM-facing canonical projection (the headline use — FR-S5)

The LLM-facing canonical projection is the application DN-09 §3.1 motivates directly. Its design
goals:

1. **Grammar-in-context–friendly rendering.** The projection's output is a string in a form whose
   grammar is machine-readable, compact, and front-loaded with the structural constraints (type
   signatures, guarantee tags, swap certificates) a constrained decoder can exploit. It is not
   required to be idiomatic human Mycelium; it *is* required to be a deterministic, reproducible
   rendering of the same content hash.

2. **Constrained-decoding hook.** The projection ships a paired *decoding grammar* artifact (a
   context-free grammar fragment or a LALR/LL table slice) that a grammar-constrained decoder can
   use to restrict generation to syntactically valid completions. This is the mechanism the KC-2
   evidence points toward: the `examples` primer gains its leverage partly from structural
   regularity, and constrained decoding amplifies that signal (DN-09 §4 — the constrained-decoding
   arm was not run; this projection is the pre-condition for running it).

3. **Honesty tag preservation (S2).** Guarantee tags (`Exact`, `Proven`, `Empirical`, `Declared`)
   and EXPLAIN records survive the projection in a machine-readable annotation format. A model
   consuming the projection can reason about the guarantee structure of the definition it is
   editing, and a model emitting a projection form can include guarantee annotations that are
   checked on round-trip.

4. **`Swap` visibility (S1/S6).** A `Swap` node is never elided by the LLM-facing projection.
   Its rendering may differ from the text grammar (e.g. a structured `<swap from="Binary{32}"
   to="Ternary{9}" cert="..."/>` annotation rather than an infix keyword), but it is always
   present and always carries the certificate reference.

The relationship to the T3.6 ablation (DN-09 §4): the LLM-facing canonical projection is the
tool that makes the "familiar-skinned same-AST" condition of the ablation *runnable*. That
condition requires a projection of the Mycelium L1/L2 node structure into a surface that looks
like a familiar language to a pre-trained model. The projection framework defined here is the
mechanism; the specific familiar-skin projection is a content of the projection, not the
framework. Both are research-prompted in §9.

### 3.5 Projections and the L2 surface grammar (RFC-0020)

The committed text grammar (RFC-0020) is the default, human-primary L3 surface. In the projection
framework it occupies a specific slot:

- It is the **canonical round-trip projection**: every content-addressed definition has exactly one
  α-normalized text rendering, produced by M-142 (the canonical formatter), and that rendering
  parses back to the same hash.
- It is the **edit-primary projection**: humans write directly in it; the LSP, linter, and
  formatter all operate on it.
- It is *not* the only projection: the LLM-facing canonical projection, structured tree views,
  and future projections are siblings, not derivatives.

The projection framework does not change the text grammar's normative status. It adds a shared
interface that the text grammar's tooling (M-142, the linter M-141) must satisfy alongside other
projections: dumpability, diffability, and the invariants of §4.

---

## 4. Reference-level design (normative once Accepted)

### 4.1 The `Projection` interface

A projection is a named, versioned, inspectable artifact:

```
Projection ::= {
    id:        ProjectionId,     // content-addressed over the mapping rules
    name:      String,           // human name (not identity)
    target:    SurfaceKind,      // Text | StructuredTree | LlmCanonical | Custom(String)
    rules:     MappingRules,     // inspectable, dumpable — ADR-006/S4; see §4.2
    edit_cap:  EditCapability,   // ReadOnly | RoundTrip
    version:   SemVer,
}

SurfaceKind ::= Text             // e.g. the committed text grammar (RFC-0020)
              | StructuredTree   // Unison-style structured/code-tree view
              | LlmCanonical     // the FR-S5 LLM-facing projection (§3.4)
              | Custom(String)   // extension point (not core)

EditCapability ::= ReadOnly
                 | RoundTrip { decode_grammar: GrammarArtifact }
```

A `Projection` whose `id` content-addresses its `rules` ensures that any change to the mapping
rules produces a new projection identity, consistent with ADR-003 (names-as-bindings-to-hashes).

### 4.2 Mapping rules: the `ProjectionRule` set

Each rule maps one or more L1/L2 node patterns to a surface fragment. Rules are:

- **Declared** artifacts (inspectable, not compiled closures — ADR-006). The full rule set for
  a projection must be serializable and dumpable by the tooling's `EXPLAIN` surface (SC-5/M-140).
- **Total**: a projection must have a rule matching every node kind in scope. An unmatched node
  is a projection-definition error, not a runtime fallback.
- **Non-overlapping** at the node-kind level (a determinism requirement; ambiguity is a
  definition error, reported explicitly — G2).

The rule format is not specified here (it is a Phase-3 design task under M-380); this RFC
specifies the *obligations* rules must satisfy, not their concrete syntax. Candidate formats
include a pattern-matching table over the L1 node grammar (RFC-0001 §4.5, r4), a trait/typeclass
implementation (L2-level, once L2 lands), or a Rust `impl` block under ADR-006's
dumpability constraint. The choice is an M-380 build decision; all candidates must satisfy §4.3.

### 4.3 Projection-level invariants (the honesty rules at projection scope)

These complement the layer-invariants S1–S6 (RFC-0006 §4.1) at the specific level of the
projection machinery:

- **P1 (no meaning change — S6 at projection scope).** A projection may not change the *meaning*
  of a definition: the L1/L2 node tree from which it renders is the source of semantic truth, and
  the projection output is a view of that tree. Editor convenience (different surface idiom,
  elided boilerplate) is permitted; semantic alteration is not. Formally: if `P(def₁) ≡ P(def₂)`
  as surface strings but `hash(def₁) ≠ hash(def₂)`, the two definitions are *distinct* — the
  projection is not authoritative for identity (Q1, RFC-0006 §8).

- **P2 (honest tags survive — S2 at projection scope).** Every guarantee tag (`Exact ⊐ Proven ⊐
  Empirical ⊐ Declared`) and every `Declared`-flag annotation present in the source node is
  present in the rendered surface, in whatever notation the target `SurfaceKind` uses. A
  projection that elides a `Declared` flag is ill-formed (VR-5 / the honesty rule / CLAUDE.md §1).

- **P3 (Swap nodes never elided — S1/S6).** Any `Swap` node present in the source is present in
  the rendered surface. Its notation may differ from the text grammar; its *presence* is
  non-negotiable (SC-3, WF1/WF2). A projection rule that maps a `Swap` to the empty string is
  a definition error.

- **P4 (identity is the hash — RFC-0001 §4.6/ADR-003).** The canonical identity of a definition
  is its content hash, not its rendered surface under any projection. Two definitions with
  different hashes rendered identically by some projection are two distinct definitions; the
  collision is a projection design flaw (it violates the non-overlapping / determinism condition
  of §4.2, or the round-trip obligation of §3.3). Renaming a binding does not change the hash
  (names are metadata); changing a node does.

- **P5 (dumpable and diffable — S4/ADR-006).** Every projection, its rules, and the rendered
  output for any given input are dumpable via the LSP's stage-dump channel (SC-5/M-140). The
  diff of two renderings of the same definition under different projections is computable and
  meaningful. No projection may hide its rules behind a compiled/opaque artifact.

- **P6 (EXPLAIN survives projection — S2/SC-5).** If a definition carries an `EXPLAIN` record
  (a `policy_used` annotation, a swap certificate, a diagnostic record), that record is either
  present in the rendered surface or attached as a structured annotation. It may not be silently
  dropped. A read-only projection that cannot represent `EXPLAIN` records in its surface must
  carry them as out-of-band structured metadata on the rendered artifact.

### 4.4 The projection registry

The set of available projections for a definition is managed by the **projection registry**, a
content-addressed lookup table keyed by `(ContentHash, ProjectionId) → RenderedSurface`. The
registry is:

- **Not a source of truth** — entries are computed from the definition (the content hash) and the
  projection rules; they are cached for performance, not authoritative.
- **Lazy** — a rendering is computed on first request; no projection eagerly renders all
  definitions.
- **Invalidated on definition change** — when a definition's content hash changes (because the
  node structure changed), all cached renderings for that hash are stale. The registry exposes
  the invalidation surface so tooling can signal to editors that a cached view is out-of-date.
- **Inspectable** — the registry is queryable: "which projections are available for hash H?",
  "show the rules used to produce this rendering", "diff the LlmCanonical and Text renderings of
  H" — all must be answerable without re-running the projection from scratch if the cache is
  warm.

### 4.5 Relationship to the name map and the Unison-style edit model

RFC-0001 §4.6/ADR-003 establish that names are stored separately as a `hash ↔ name` map, and
RFC-0006 §4.1 S3 re-states: "names are bindings to hashes, never identity." The projection
framework extends this:

- A **binding** maps a human-readable name to a content hash. The name is mutable metadata; the
  hash is fixed.
- A **projection** maps a content hash (plus a `ProjectionId`) to a rendered surface. The
  rendering is also mutable metadata (the projection rules may be revised, producing a new
  `ProjectionId` and new renderings).
- An **edit** via a `RoundTrip` projection produces a new content hash; the name map is updated
  to bind the name to the new hash; the old hash persists in the name map history (the Unison
  precedent: old definitions are never deleted, only superseded in bindings).

This design means that the entire audit trail — which definition a name pointed to at each
point in history, which projection rendered it — is recoverable from the name map, the
projection registry, and the content-addressed definition store. No information is lost.

### 4.6 LLM-facing canonical projection: design sketch (FR-S5)

The `LlmCanonical` projection (§3.4) has the following structural design:

```
LlmCanonicalProjection = {
    id:          content-addressed over the rule set below,
    name:        "llm-canonical-v0",
    target:      LlmCanonical,
    edit_cap:    RoundTrip { decode_grammar: <machine-readable grammar artifact> },
    rules:       {
        // Structural preamble: emitted once per definition, before any expression body.
        // Contains: signature, guarantee tags, any open `Declared` annotations.
        Preamble(def)       → ";;; def <name> : <type> [<guarantee-tag>]"

        // Core node renderings: familiar-idiom where possible; structured where novel.
        Const(v)            → <literal rendering per RFC-0001 §4.6 + T3.1-B>
        Var(x)              → <identifier>
        Let(x, rhs, body)   → "(let [<x> <rhs>] <body>)"   // Clojure-style, regular
        Op(p, args)         → "(<op-name> <args...>)"       // S-expression spine
        Swap(from, to, cert)→ "(swap! <from-expr> :to <to-type> :cert <cert-ref>)"
                              // swap! = familiar "mutation" sigil; :cert always present
        Lam(x, body)        → "(fn [<x>] <body>)"
        App(f, arg)         → "(<f> <arg>)"
        Fix(name, body)     → "(fix <name> <body>)"
        Construct(ctor, ...) → "(<ctor-name> <args...>)"
        Match(scr, alts)    → "(match <scr> (<pat> <body>)...)"

        // EXPLAIN attachment: inline when short, out-of-band reference when large.
        Explain(record)     → ";;; explain: <json-ref-or-inline>"
    }
}
```

The S-expression / Clojure-style spine was chosen as a sketch because:
- S-expressions are maximally regular (low out-of-distribution token overhead for pre-trained LLMs).
- The grammar is trivially machine-readable (a LALR(1) grammar in ~10 rules).
- `swap!` is a familiar Clojure identifier, reusing existing token distributions.
- The `;;; …` comment-preamble idiom is recognized across many languages.

This is a **design sketch, not a commitment**. The concrete rule set is an M-380 build decision,
subject to the research prompts in §9 (specifically: whether this form raises the T3.6 retention
ratio above the bare text surface). A different familiar-skin idiom (Rust-style, Haskell-style,
JSON-AST) may perform better — the measurement decides, not the sketch. The sketch is present
here to make the design-space concrete, not to pre-commit a surface (Declared, not Proven — VR-5).

### 4.7 T3.6 ablation and the fallback trigger

DN-09 §4 records the T3.6 falsification ablation as an honest open follow-up: the five-condition
comparison (bare novel syntax · +grammar-in-context · +constrained decoding · familiar-skin same-
AST · embedded DSL) with the explicit falsification threshold (retention ratio < ~70% of the
familiar-skin condition's pass@1 ⇒ L3 must become a projection of known syntax).

This RFC is the design that makes the "familiar-skin same-AST" condition *runnable*: the
`LlmCanonical` projection with `RoundTrip` capability is exactly that condition, applied to the
Mycelium L1/L2 node structure. If the T3.6 ablation is run and the text grammar fails the
threshold, the L3 strategy migrates to **LlmCanonical-primary** (the projection becomes the
default authoring surface), not to an embedded DSL (the embedded-DSL fallback, RR-3, remains
available but is last-resort — DN-09 §3).

The threshold trigger is: if the text-grammar arm (RFC-0020) retains < ~70% of the LlmCanonical
arm's pass@1 on composition tasks, open an RFC that supersedes RFC-0020's "text-primary" status
and promotes `LlmCanonical` to the primary L3 surface. That RFC is a maintainer decision
(append-only); this RFC does not pre-write it.

---

## 5. Drawbacks

1. **Projection author burden.** Writing a total, dumpable, non-overlapping rule set for the full
   L1/L2 node grammar (currently ~15 node kinds in r4, growing as L2 lands) requires sustained
   effort per projection. This is real cost — it is the burden G11 flags as potentially
   unergonomic. The §9 research prompts must establish whether the cost is acceptable before
   ratification.

2. **Registry complexity.** The projection registry is a new caching/invalidation surface in the
   toolchain. It must integrate with the LSP (M-310), the build system (M-311/M-312), and the
   content-addressed store (ADR-003). This adds implementation surface to an already-deep stack.
   Mitigation: the registry is lazy and not on the critical path for the interpreter or type
   checker; it lives above L0/L1 (KC-3), never inside the trusted kernel.

3. **Projection drift.** If the L1 node grammar grows (RFC-0001 revisions) and a projection's
   rule set is not updated, the projection becomes partial. The totality requirement (§4.2)
   catches this at definition time, but it means every node addition to L1 is a breaking change
   for all existing projections. Mitigation: the canonical text grammar (RFC-0020) is maintained
   by the same team that maintains the node grammar; other projections are best-effort and must
   declare their node-kind scope.

4. **LLM-facing projection may not help.** If the T3.6 ablation shows that the `LlmCanonical`
   projection does not raise the retention ratio above the text grammar baseline, the LLM-facing
   projection provides no measurable benefit and the implementation cost is unjustified. The §9
   research prompts are the gate; this RFC does not claim the benefit in advance (Empirical, not
   Proven — VR-5).

---

## 6. Rationale & alternatives

### 6.1 Why projections over multiple committed syntaxes

The alternative is to commit two or more independent surface syntaxes (a "human syntax" and a
"machine syntax") separately, each with its own parser, formatter, and spec. This violates the
single-source-of-truth principle: the two syntaxes must be kept in semantic sync, which requires
continuous translation validation. The projection framework instead keeps one semantic source
(the L1/L2 node tree, content-addressed) and generates multiple surfaces from it — the same
lesson GHC Core and Unison draw from their respective architectures.

### 6.2 Why not a dedicated compiler IR-to-text pass

A dedicated IR-to-text pass (like GHC's `ppr`) is a notational projection, not a semantic-level
one. It renders the IR in a fixed human-readable notation but does not expose a
constrained-decoding grammar, does not attach honesty tags in a machine-readable form, and is
not designed to be swapped out for a different surface. The projection abstraction is strictly more
general while subsetting to the same behavior when `SurfaceKind = Text` (§3.5).

### 6.3 Why S-expression sketch for the LLM-facing projection

The S-expression spine minimizes structural novelty (the grammar is trivially regular), reuses
the highest-frequency token shapes in pre-trained LLM corpora (parentheses, identifiers, string
literals), and provides a natural nesting structure that matches the recursive L1 node tree. The
main alternative (JSON-AST) is more verbose; Rust-style is more idiomatic but has
higher out-of-distribution overhead for swap-specific constructs. The sketch is a starting point,
not a conclusion — the T3.6 ablation decides.

### 6.4 Why require rules to be dumpable (ADR-006)

ADR-006's no-black-boxes constraint applies to every component that selects, converts, or
approximates. A projection *selects* a rendering for each node kind; making the selection rules
opaque (a compiled Rust function whose source is unavailable) would violate the same principle
that applies to swap policies and selection policies (RFC-0005). The dumpability requirement is
not an extra burden — it is the invariant the corpus has applied consistently since RFC-0001.

---

## 7. Prior art

- **Unison** (content-addressed definitions, names-as-metadata — the direct source of ADR-003 and
  §4.5's edit model). Unison's "abilities" and "namespaces" demonstrate that content-addressed
  code survives multi-surface rendering; its `ucm` codebase uses a structured view alongside text.
- **JetBrains MPS** (projectional editing — the origin of "projection" as a PL term). MPS
  demonstrates that multiple projections of the same AST are practically viable but require
  substantial tooling investment. The G11 ergonomics concern is directly motivated by MPS's cost
  experience.
- **GHC Core / STG** (notational projections over a fixed IR; ppr passes). GHC demonstrates that
  a small trusted core with multiple human-facing views is a tractable architecture; this RFC
  targets the same pattern at the L3 surface level.
- **Racket** (`syntax` objects + macro expanders as projections). The Racket macro system projects
  high-level surface forms to core forms, with full source-location preservation — the equivalent
  of S4 (inspectable elaboration) at the macro level.
- **Grammar-constrained decoding** (GBNF, Guidance, Outlines; RFC-0006 §8 Q1 / T3.6). The
  constrained-decoding hook in §4.6 is directly motivated by published results showing grammar-
  constrained generation improves syntactic validity for domain-specific languages.
- **Grammar prompting / MTOB** (the T3.6 hypothesis source; RFC-0006 §8 Q1). The evidence base
  the KC-2 Q1 hypothesis rests on; the `examples` primer in the KC-2 run is a lightweight
  analogue of book-quality grammar-in-context (DN-09 §2).

---

## 8. Unresolved questions

- **Q-P1 (projection rule format).** What is the concrete syntax for `MappingRules` (§4.2)? A
  pattern table, a trait implementation, a data file? The choice affects both ergonomics and
  dumpability. *Deferred to M-380; not blocking this RFC's design.*

- **Q-P2 (familiar-skin idiom for the LLM-facing projection).** Is the S-expression sketch
  (§4.6) the right familiar-skin idiom, or would a Rust-style or Haskell-style idiom retain more
  leverage for the frontier models used in the T3.6 ablation? *Decided by the §9 research prompt
  RP-2; not ratifiable before that run.*

- **Q-P3 (projection registry persistence).** Should the registry be an in-process cache, a
  content-addressed on-disk store, or part of the build-system certificate (M-311/M-312)?
  *Deferred to the M-380 implementation; design sketch in §4.4 is non-prescriptive.*

- **Q-P4 (round-trip grammar format).** What is the concrete `GrammarArtifact` type in
  `EditCapability::RoundTrip`? A CFG in EBNF (matching `docs/spec/grammar/`), a LALR table,
  a GBNF file compatible with popular constrained-decoding libraries? *Decided by the §9 research
  prompt RP-1 (constrained-decoding hook ergonomics) and M-380.*

- **Q-P5 (text grammar as a projection instance).** RFC-0020 (the committed text grammar) is
  cited as "one projection among several" in §3.5, but RFC-0020 predates this RFC and its
  tooling (M-142) was not designed against the `Projection` interface. Does RFC-0020 need a
  normative erratum to satisfy §4.3 (P1–P6), or is the relationship declaratory only? *A
  maintainer decision; not blocking this RFC's design.*

---

## 9. Research prompts (must run before Draft → Accepted)

**Status: PARTIALLY ADVANCED — `research/11-semantic-projection-framework-RECORD.md` (2026-06-18).**
The *design-decidable* parts are grounded; the *empirical* gate is honestly **not** discharged. The
record grounds the projection model + P1–P6 as established prior art (Unison/MPS) with a
locally-checkable honesty overlay, answers the dual-rendering question (RP-4 sub-q 3 — one architecture
suffices), assesses authoring as **feasible** at single-engineer scale (grounded in the existing
`mycelium-lsp::feedback` node-walk; a *measured* cost study stays open), and recommends the Unison
posture for human usability (edit-in-text; projections read-mostly + opt-in round-trip — RP-4 sub-q 1).
**The LLM-leverage gate (the T3.6 retention-ratio ablation; the "canonical projection raises leverage"
claim) is irreducibly empirical and remains OPEN** — `research/11` supplies a turnkey five-arm protocol
over the existing `experiments/` harness, **not** a result; per VR-5 no leverage may be asserted without
the run. A maintainer **could ratify the framework/design** (the RFC-0006 r5 carve-out pattern) **with
the LLM-leverage claim explicitly carved out as empirically open**; full ratification still requires the
run. The original prompts are retained below verbatim (append-only).

These two prompts are the gate on ratification: the G11 ergonomics question and the LLM-facing
projection leverage question are both **open** (Declared, not Empirical). Moving to Accepted
without running them would upgrade the guarantee without a checked basis (the honesty rule,
CLAUDE.md §1; VR-5).

---

### RP-1 — G11 ergonomics: are semantic-level projections viable for projection *authors*?

**Question.** A semantic-level projection requires a total, dumpable, non-overlapping rule set
over the full L1/L2 node grammar. Is the authoring cost acceptable in practice? Specifically:

1. Can the `LlmCanonical` projection rule set (§4.6) be authored and maintained by a single
   engineer as the L1 node grammar grows through its current ~15-node set (r4) toward the L2
   additions (type inference, traits, modules)?
2. Does the tooling burden (dumpability, totality checker, invalidation) make projection
   authoring significantly more expensive than maintaining a standalone pretty-printer?
3. Are there projection-rule formats (§Q-P1) that reduce the authoring burden to the level of a
   well-maintained Rust `Display` implementation, while satisfying the §4.3 invariants?

**Method.** Prototype the `LlmCanonical` projection rule set over the current L1 node grammar
(RFC-0001 r4: `Const | Var | Let | Op | Swap | Lam | App | Fix | Construct | Match`). Measure:
authoring time; number of rules; coverage of the totality requirement; whether the resulting
rule set is legibly dumpable. Run the same exercise for a second projection (e.g. a structured
JSON-AST view) to establish whether the cost is projection-specific or systemic. Report the
finding as `Empirical` (measured) or `Declared` (asserted without measurement) — do not pre-write
a conclusion.

**Outcome feeds.** If the authoring burden is acceptable → proceed to M-380 full implementation.
If the burden is prohibitive at current L1 scope → the projection framework needs a simpler rule
format before ratification (revise §4.2 and re-draft); G11 stays open.

---

### RP-2 — LLM-facing canonical projection: does it raise leverage above the bare text surface?

**Question.** Does the `LlmCanonical` projection (§4.6) raise the LLM-leverage retention ratio
above the committed text grammar (RFC-0020), and does it clear the T3.6 falsification threshold?
This is the strong-form Q1 hypothesis from RFC-0006 §8 that the KC-2 run did not establish
(DN-09 §4): "novel-but-regular syntax retains most leverage" — still `supported-but-not-confirmed`.

**Method.** Run the T3.6 five-condition ablation as specified in DN-09 §4 and RFC-0006 §8 Q1:

1. Bare novel text grammar (baseline — the KC-2 arm, already run).
2. Text grammar + book-quality grammar-in-context primer.
3. Text grammar + constrained decoding (requires the `RoundTrip.decode_grammar` from §4.6).
4. **LlmCanonical projection** + grammar-in-context (the "familiar-skin same-AST" condition this
   RFC enables).
5. Embedded DSL in a host language (the RR-3 fallback condition).

Report the **retention ratio** of each arm against arm 4 (familiar-skin baseline), with
the explicit falsification threshold applied: if arm 2 (text + grammar-in-context) retains
< ~70% of arm 4's pass@1 on composition tasks across ≥3 seeds and ≥1 frontier model, open the
RFC that migrates L3 to `LlmCanonical`-primary.

**Outcome feeds.** If arm 4 materially outperforms arm 2 and the threshold is met → ratify the
`LlmCanonical` projection and escalate M-380 priority. If arm 4 does not materially outperform
arm 2 → the LLM-facing projection provides convenience but not leverage; retain as opt-in, deprioritize. If arm 2 fails the threshold → trigger the L3 migration path (§4.7).

**Honesty constraint.** The T3.6 ablation is the designed rigor for this question (RFC-0006 §8
Q1; T3.6). Running only a subset of the five arms and claiming the projection "works" would be an
ungrounded upgrade (VR-5). Either run all five arms or report the finding at the strength of
the arms actually run. The threshold applies only when arm 4 is present; absent arm 4 there is no
falsification comparison.

---

## 10. Future possibilities

- **Guarantee-polymorphic projections.** A projection that adapts its rendering based on the
  guarantee tag of each node — e.g., rendering `Declared` values with a visible warning annotation
  in human-facing text, but with a compact `[D]` sigil in the LLM-facing form — would make
  guarantee visibility a tunable parameter of the projection, not a fixed convention.
- **Diff-aware projections.** A projection that, given two content hashes, renders the *diff*
  between them in a surface-appropriate form (not a raw AST diff but a user-facing "what changed"
  narrative). This is the projection layer's analogue of RFC-0004 §6's lowering-stage diff.
- **Projection composition.** A meta-projection that composes two projections — e.g., render to
  `LlmCanonical` form and then apply a post-processor that inlines EXPLAIN records — as a first-
  class named artifact. This generalizes the pipeline without requiring a bespoke third projection.
- **Editor-native projections.** Once an LSP-integrated projection editor exists (a long Phase-4/5
  item), developers could author and preview projections live against the current definition corpus,
  with the totality and non-overlapping checks running as IDE diagnostics. This is the MPS-inspired
  vision G11 points at, made tractable by the small L1 node set and the content-addressed
  architecture.

---

## Meta — changelog

- **2026-06-18 (r0) — Draft authored.** Initial design draft: the semantic-level projection
  framework (FR-C1/G11/M-380), motivated by the KC-2 verdict (DN-09) and the real-but-modest text-
  surface leverage it established. Covers the projection model (§3–§4), invariants P1–P6, the
  LLM-facing canonical projection (FR-S5, §3.4/§4.6), the T3.6 ablation trigger (§4.7), and two
  research prompts (§9) gating ratification. Status stays **Draft** pending RP-1 (G11 ergonomics)
  and RP-2 (LLM-facing leverage measurement). Ratification is a maintainer decision, append-only.
- **2026-06-18 — §9 PARTIALLY ADVANCED (design grounded; empirical gate open) — `research/11`.** The
  design-decidable parts are grounded (the projection model + P1–P6 as Unison/MPS prior art with a
  locally-checkable honesty overlay; one dual-rendering architecture, RP-4 sub-q 3; authoring
  feasibility grounded in the existing node-walk; the Unison human-usability posture, RP-4 sub-q 1).
  The **LLM-leverage gate (the T3.6 ablation / RP-1) remains OPEN** — `research/11` gives a turnkey
  protocol over the `experiments/` harness, not a result (VR-5; no leverage asserted without the run).
  **Status stays Draft:** the framework/design is ratification-ready (carve-out pattern); the empirical
  gate is the one part of this four-RFC wave that analysis alone cannot close. No normative rule
  changed. Append-only.
