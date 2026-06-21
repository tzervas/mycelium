# Design Note DN-10 — Remaining L1 Gaps: Elaboration Plans for R7-Q3 and R7-Q4

| Field | Value |
|---|---|
| **Note** | DN-10 |
| **Status** | **Resolved** (2026-06-21 — R7-Q3 enacted by M-391 (2026-06-19): mutually-recursive surface elaboration landed in `mycelium-l1::elab`; R7-Q4 enacted by M-390 (2026-06-18): prim signature table migrated to content-addressed prim declarations. Both items done; capture purpose complete. M-648 editorial sweep. Append-only.) **Draft / Resolved-as-capture** (planning capture, advisory — see posture below). Records the two open L1 work items as concrete, dependency-ordered plans. Neither item is a normative decision; both are build-task framings for the maintainer to schedule. |
| **Decides** | Nothing normatively. Frames the *elaboration* work remaining under R7-Q3 and R7-Q4 as concrete, dependency-ordered steps with differential obligations and one research/spike prompt each. |
| **Feeds** | RFC-0007 §8 (R7-Q3, R7-Q4); RFC-0001 r4/r5 (the `Fix`/`FixGroup` node grammar); `crates/mycelium-l1::elab`; ADR-003 (content-addressed identity); KC-3 (small auditable kernel). |
| **Date** | June 18, 2026 |
| **Task** | (none yet — L1 gap capture; follows M-343 / RFC-0001 r5 `FixGroup` enactment) |

> **Posture (honesty rule / VR-5).** Advisory capture. Neither item reopens a ratified decision;
> both are additive steps that follow from decisions already made (RFC-0001 r5; ADR-003).
> This note is a *planning capture*, not a normative decision — the same role DN-08 played for
> maturation granularity before RFC-0017 ratified it. Where sub-questions are genuinely open this
> note marks them as research/spike prompts; it does not resolve them. Append-only: supersede, do
> not rewrite.

---

## 1. Context: what is and is not settled at L1

RFC-0001 reached r5 (enacted 2026-06-16, M-343) and RFC-0007 was Accepted at r4 (2026-06-15).
Together they close all of R7-Q1 and almost all of R7-Q3:

- **Closed (RFC-0001 r4):** `Fix` as a node (R7-Q1); canonical cycle-ordering for mutually-recursive
  declaration groups so their content-addressed *identity* is stable (R7-Q3, identity half).
- **Closed (RFC-0001 r5 / M-343):** `FixGroup{defs, body}` as the L0/L1 node for mutual recursion;
  the two-case substitution semantics (focus + continuation unfolds); Tarjan SCC decomposition in
  `mycelium-l1::elab`; the three-way M-210 differential extended to mutual-recursion programs
  (R7-Q3, *node* and *self-recursion elaboration* halves).

**What remains open** (the explicit RFC-0007 §8 residuals):

- **R7-Q3 — mutual-recursion *surface* elaboration.** The cycle *identity* is fixed and the L0
  `FixGroup` node is implemented; but the *surface→registry/`FixGroup`* elaboration step for
  mutually-recursive *functions* written in surface syntax is still deferred. The prototype in
  `crates/mycelium-l1::elab` reaches `FixGroup` via the Tarjan SCC path from a call graph; it does
  not yet parse or lower a surface `let rec … and …` (or the Mycelium equivalent) across multiple
  top-level definitions that call each other.
- **R7-Q4 — prim table `Π` as content-addressed declarations.** Currently `Π` is a fixed builtin
  table hard-coded in the elaborator/typechecker; RFC-0007 §8 records it *should* become
  declarations with their own content addresses (ADR-003).

The two gaps are independent: R7-Q3 elaboration is a *surface-to-core* elaboration step; R7-Q4
is an *internal representation* change to how prims are stored and queried.

---

## 2. R7-Q3 — Mutual-recursion surface elaboration

### 2.1 What is already in place (do not redo)

- The canonical cycle-ordering algorithm (RFC-0001 §4.6 / r4): given a strongly-connected group of
  ≥2 declarations, their hashes are computed with cycle occurrences replaced by a placeholder, then
  the members are sorted by those placeholder-substituted hashes to produce the canonical order
  (the Unison recipe). This means hashes are **name-independent and deterministic**, now and
  after the surface grows mutual recursion.
- `FixGroup{defs: [(VarId, Node)], body: Node}` in the L0 node grammar (RFC-0001 §4.5 r5), with
  content-addressing over α-normalized members.
- Tarjan SCC decomposition in `mycelium-l1::elab`: the elaborator already detects strongly-
  connected components in the call graph of already-desugared definitions and emits `FixGroup` for
  groups of ≥2 (self-recursive singletons stay on `Fix`). This is the back-end of the elaboration
  chain.

### 2.2 What the elaboration step must do (the deferred part)

The deferred part is the *front-end* of the elaboration chain: accepting, parsing, and lowering a
**mutually-recursive surface definition group** (≥2 top-level functions that refer to each other)
into the call-graph representation that the existing Tarjan SCC path then consumes.

Concretely, the elaboration must:

1. **Group collection.** Identify the set of top-level definitions whose bodies form a
   strongly-connected component under direct-call reference (i.e., `f` calls `g` and `g` calls
   `f`, possibly transitively). In Mycelium's first-order v0 surface this is purely a name-lookup
   pass over the elaborated call sites — no higher-order analysis needed.

2. **Order and hash the group.** Apply the RFC-0001 r4 canonical cycle-ordering to the collected
   group (replace each member's own-group occurrences with the `FixGroup` placeholder; hash; sort).
   This step is already implemented in the content-addressing layer; the elaboration step only needs
   to invoke it with the right inputs.

3. **Emit `FixGroup`.** Produce a `FixGroup{defs, body}` node where `defs` lists the group's
   members in canonical order and `body` is the continuation. The Tarjan SCC path in
   `mycelium-l1::elab` already does this once it has the SCC; the new front-end just needs to feed
   it mutually-recursive surface definitions as an SCC rather than requiring the call graph to be
   pre-built from a single compound definition.

4. **Type-check the group.** Apply `T-Fix` / the `FixGroup` typing rule from RFC-0007 §4.4 over
   the group simultaneously (each member's type annotation must satisfy `Γ, f₁:τ₁ … fₙ:τₙ ⊢ eᵢ : τᵢ`
   for each member `fᵢ`). In v0 (monomorphic), the type annotations are required at the group
   boundary — inference across the group is not in scope yet.

5. **Totality classification.** Feed the `FixGroup` to the §4.5 mutual structural descent
   classifier (enacted 2026-06-16): assign a designated argument position `p(fᵢ)` to each member
   and check that every inter-member call passes a structural piece of `p(f)` to `p(g)`. This is
   already implemented in `crates/mycelium-l1::totality`; the elaboration step does not extend it,
   only feeds it.

### 2.3 Why identity-first means this is purely additive

Because RFC-0001 r4 fixed the cycle-ordering **before** the surface elaboration was written, the
hashes produced by step 2 above are identical to what any future surface-level representation of
the same group would produce. No existing hash will move when the surface grows mutual recursion:
the `FixGroup` node for a group `{f, g}` has the same content address regardless of whether
the group arrived via the Tarjan SCC path (current) or via an explicit `let rec … and …` surface
form (future). This is the "identity-first" dividend (ADR-003): the spec commitment (canonical
ordering) was made separately from and before the implementation of the elaboration step, so the
elaboration is purely additive — no existing content addresses are invalidated.

### 2.4 Differential obligation

NFR-7 (differential equivalence) applies directly. On any mutually-recursive program in the v0
surface:

```
L1-eval ≡ elaborate→L0-interp
```

The M-210 three-way differential was extended to mutual-recursion programs (ping/pong, even/odd,
a constructive group, a 3-cycle) in M-343 (RFC-0001 r5 changelog). The elaboration step above must
pass that same differential — its only new obligation is that *surface-written* mutually-recursive
programs produce the same `FixGroup` (and thus the same evaluation) as if they had been constructed
programmatically through the existing Tarjan path.

A witness test: define `ping` and `pong` in surface syntax (mutual calls), elaborate via the new
front-end, and assert the resulting L0 `FixGroup` matches the one the Tarjan path would produce
from the same call graph. This is a regression/identity test, not a new theorem.

### 2.5 Build-task framing

**Dependency order:**

1. The content-addressed cycle-ordering (done, RFC-0001 r4).
2. `FixGroup` node + substitution semantics (done, RFC-0001 r5).
3. Tarjan SCC path in `mycelium-l1::elab` (done, M-343).
4. Mutual structural descent totality classifier (done, 2026-06-16 §4.5 extension).
5. **This task:** surface parsing + call-graph construction for top-level mutually-recursive
   definitions → feeds the existing Tarjan path → existing `FixGroup` emission.

**Scope of the task:** `crates/mycelium-l1::elab` (the elaboration front-end); no changes to
`mycelium-core`, `mycelium-interp`, or the totality checker. Add to the M-210 differential: ≥2
surface-written mutual-recursion programs (a data/matching variant and a pure-function variant).

### 2.6 Research/spike prompt

> **R7-Q3 surface elaboration spike.** The open sub-question is the *surface grammar* for
> mutual recursion in Mycelium's v0 first-order syntax: does the language use explicit grouping
> (`let rec f = … and g = …`), rely on *declaration order* + a fixpoint-at-the-nodule-boundary
> (every top-level function in a `nodule` is mutually visible — the Unison/ML module semantics),
> or require an explicit `mutually_recursive { … }` block? The choice affects the elaboration
> front-end (step 1 in §2.2) and the grammar (`docs/spec/grammar/mycelium.ebnf`). The underlying
> *elaboration mechanics* (steps 2–5) are already settled; only the surface parsing question is
> open. Spike: prototype all three options against the existing Tarjan path, measure the
> grammar/diagnostic impact, and record the choice append-only (a new DN or RFC-0007 amendment).
> Feeds: RFC-0007 §8 R7-Q3 (surface half); `docs/spec/grammar/`; the KC-2 surface commitment
> (DN-09 §3.1 — append-only refinement, not a reversal).
>
> **Resolved (2026-06-19).** RP-6 decided — **candidate 2: nodule-wide mutual visibility, no new
> syntax** (DN-13). The surface front-end already realizes it (nodule-wide visibility in `checkty`
> Pass 2/3 + the Tarjan→`FixGroup` path in `mycelium-l1::elab`, from M-343); **M-391** confirms it
> (M-210 differential + identity + never-silent). Append-only.

---

## 3. R7-Q4 — Prim table Π as content-addressed declarations

### 3.1 Current state

The prim signature table `Π` (RFC-0007 §4.4: `Π(p) = (τ₁…τₙ) → τ`) is a **fixed builtin table**
hard-coded in the elaborator and typechecker. Each primitive (e.g. `add_binary`, `bundle`, `swap`)
has a statically-known arity and type signature that is looked up by name at typecheck time. The
table is sound for v0 because the set of primitives is fixed and small.

### 3.2 What the migration should do

RFC-0007 §8 R7-Q4 records the direction: `Π` should become **declarations with their own content
addresses** (ADR-003), consistent with how data declarations already live in the content-addressed
registry `Σ` (RFC-0001 §4.6 / r3). Concretely:

1. **Represent each prim as a declaration.** A prim declaration has: a canonical name (stored
   separately, as with all names — ADR-003); a type signature `(τ₁…τₙ) → τ` (the same content
   that `Π` currently stores); and an *intrinsic guarantee* `g_f` (RFC-0001 §4.7: the prim's own
   contribution to the guarantee meet). The declaration is hashed over its α-normalized type
   signature (names are not identity).

2. **Store prim declarations in the registry.** Alongside data declarations in `Σ`, or in a
   parallel `Π`-registry with the same content-addressing rules. A prim reference in a term
   becomes a content hash, not a name string — exactly as `CtorRef = #T#i` replaced constructor
   names (RFC-0001 §4.6 r3).

3. **Retire the hard-coded table.** The elaborator and typechecker look up `Π` by content hash
   rather than by name. The *initial population* of the registry is the same fixed set of builtins;
   the migration is internal (no user-visible change in v0).

4. **Gain: EXPLAIN over prims.** Because each prim is now a registry entry, the `EXPLAIN` channel
   (G2, SC-3) can report which prim was selected, its full type signature, and its intrinsic
   guarantee — exactly as it does for data constructors. A prim call is no longer a black box.

5. **Gain: separate-compilation story.** Content-addressed prim declarations are the foundation for
   a future "prim extension" mechanism: a new paradigm's prims arrive as additional registry
   entries, not as code changes to the fixed table. This follows from ADR-003's general principle
   (names-as-metadata, hashes-as-identity) and KC-3's preference for a small kernel with open
   parameter registries (RFC-0001 §4.1).

### 3.3 Why v0's fixed table is sound meanwhile

The current fixed table is sound because: (a) the set of prims is closed and small in v0; (b)
every `Op{prim, args}` node is typechecked against `Π` before entering the registry, so no
ill-typed prim call survives; (c) the table is deterministic and statically known — there is no
hidden selection or approximation. The migration to content-addressed declarations is a *uniformity
and extensibility* improvement, not a correctness fix. The honesty rule permits marking this
`Declared` at the migration boundary only because the current behavior is sound and the improvement
is future work; it does not require an immediate fix (VR-5: downgrade to stay honest, but v0's
fixed table is not a dishonest design choice).

### 3.4 Differential obligation

The migration must preserve `Π`-lookup semantics exactly: for every prim `p` in the current fixed
table, `Π_new(hash(p)) = Π_old(name(p))`. The existing M-210 differential (L1-eval ≡
elaborate→L0-interp) is a sufficient regression check because every prim call passes through `Π`;
the migration does not need a new differential, only a green `cargo test` after the registry
switch.

### 3.5 Build-task framing

**Dependency order:**

1. Data declaration registry `Σ` with content-addressing (done, RFC-0001 r3/r4).
2. Prim declaration schema (a new registry entry type, parallel to data declarations).
3. Populate the registry from the current fixed table (a one-time migration; same entries, now
   hashed and stored as registry declarations).
4. Update the elaborator and typechecker to look up `Π` by hash rather than by name.
5. Update EXPLAIN / the LSP to surface prim declarations as inspectable registry entries.

**Scope:** `crates/mycelium-core` (registry schema + population); `crates/mycelium-l1::elab` and
`mycelium-l1::check` (lookup path); `mycelium-lsp` (EXPLAIN/dump). No change to L0 node grammar
or the reference interpreter.

### 3.6 Research/spike prompt

> **R7-Q4 prim-declaration schema spike.** The open sub-question is the *intrinsic guarantee*
> field: how is a prim's `g_f` specified, and can it be checked rather than asserted? For prims
> whose behavior is mathematically exact (e.g. `add_binary` over exact inputs), `g_f = Exact`
> is obvious and uncontroversial. For prims like `bundle` (VSA superposition, inherently
> `Empirical`), the `g_f` is grounded in a cited theorem (RFC-0003 §5; T0.2) — but is the theorem
> citation stored *with* the prim declaration (making the `Proven`/`Empirical` distinction
> content-addressable) or separately? Spike: prototype a prim declaration schema that includes the
> `BoundBasis` field (RFC-0001 §4.3) alongside the type signature, and verify that the guarantee
> meet (RFC-0001 §4.7) over prim calls produces the same tags as the current hard-coded intrinsic
> guarantees. Feeds: RFC-0007 §8 R7-Q4; RFC-0001 §4.3 `BoundBasis`; ADR-003; KC-3.

---

## 4. Dependency map and scheduling note

```
RFC-0001 r4 (Fix node, cycle identity)
    └── RFC-0001 r5 (FixGroup node, M-343)
            └── R7-Q3 surface elaboration (this note §2)
                    └── surface grammar choice (§2.6 spike → DN or RFC-0007 amendment)

RFC-0001 r3 (data registry Σ)
    └── R7-Q4 prim declarations (this note §3)
            └── prim BoundBasis schema (§3.6 spike)
```

The two gaps are **independent**: R7-Q4 can proceed before, after, or in parallel with R7-Q3's
surface grammar choice. Neither blocks any currently-open RFC (RFC-0006 Q3 stage-1 grading does
not depend on the prim-as-declarations migration; RFC-0007 §4.2's registry is already clean for
data declarations). Both are purely additive in the RFC-0001 r5/RFC-0007 r4 world.

---

## Changelog

- **2026-06-21 — Resolved (M-648 editorial sweep).** Both planned items are enacted: R7-Q4 (M-390, 2026-06-18) — prim signature table `Π` migrated to content-addressed prim declarations in `mycelium-core::data`; R7-Q3 (M-391, 2026-06-19) — mutually-recursive surface elaboration (Tarjan SCC → `FixGroup`) landed in `mycelium-l1::elab`, resolving the surface-grammar spike (RP-6). This note's capture purpose is complete. Append-only.
- **2026-06-18 — Draft / Resolved-as-capture.** Initial capture of the two remaining L1 work
  items (R7-Q3 surface elaboration, R7-Q4 prim-table migration) as concrete, dependency-ordered
  plans. Grounded in RFC-0007 §8, RFC-0001 r4/r5, RFC-0006 §4/§8, ADR-003, KC-3. Advisory
  posture; append-only.
