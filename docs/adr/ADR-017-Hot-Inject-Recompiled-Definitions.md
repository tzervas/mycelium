# ADR-017 — Hot-inject recompiled definitions: content-addressed dynamic linking, immutable-by-construction

| Field | Value |
|---|---|
| **ADR** | 017 |
| **Title** | Inject newly-compiled definitions into a running image without recompiling/relinking the whole binary, via a hash-keyed dispatch table + content-addressed dynamic linking — safe because definitions are immutable (a change is a new hash, never an in-place mutation) |
| **Status** | **Proposed** (drafted 2026-06-16; enacts RFC-0004 r2 §10 **OQ-2**) |
| **Date** | 2026-06-16 |
| **Depends on** | ADR-016 (the interpreted↔compiled ABI this rides); RFC-0004 §9 (the continuum) / §10 OQ-2; RFC-0001 §4.6 (content-addressing) / §4.7-equivalent immutability; ADR-003 (Unison identity); ADR-009 (hybrid execution); M-340 (in-process `dlopen` JIT — the seed) |
| **Resolves** | RFC-0004 §10 **OQ-2** (hot-inject of recompiled definitions) |

## Context

The most ambitious item in the maintainer's execution vision (RFC-0004 §9): *"compile only the
changed definitions and inject them into a running/compiled image without recompiling or relinking
the whole binary."* RFC-0004 §10 flagged it as OQ-2 ("glorious if doable — must be reliable and
robust"). The classic obstacle to hot code loading is **atomicity** — swapping running code in place
risks tearing in-flight calls, version skew, and corruption.

The decisive observation: **Mycelium definitions are immutable and content-addressed.** A
definition's identity *is* its hash (ADR-003); "editing" a definition does not mutate it — it
produces a **new definition with a new hash** (RFC-0001 §4.6), and any dependent that referenced the
old hash is *itself* a new definition with a new hash. So a "change" is never an in-place mutation of
anything; it is the *appearance of new immutable definitions*. That dissolves the atomicity problem.

## Decision

**1. A hash-keyed dispatch table (ADR-016's call ABI) is the injection point.** The running image
holds the `ContentHash → entry` table ADR-016 defines. A call to a definition resolves through it
(compiled entry if present, else interpret — RFC-0004 §9.1).

**2. Inject = load a content-addressed unit + register its hash — never mutate running code.** Each
compiled definition is a **separately-loadable unit keyed by its hash** (content-addressed dynamic
linking; the M-340 in-process `dlopen` JIT is the prototype path). To inject: load the unit, then
register `hash → entry` in the dispatch table. There is **no in-place patching** of existing code.

**3. "Compile only what changed" falls out of content-addressing — no diffing required.** Editing a
definition yields a new hash; its transitive dependents (which name the old hash) get new hashes too;
everything else keeps its hash *and its already-compiled entry*. So the recompile set is **exactly
the changed dependency-closure** (the set of new hashes), computed by hash reachability, not by a
heuristic file/AST diff. Unchanged definitions are never recompiled and never re-injected.

**4. The atomicity hazard dissolves (immutability + content-addressing).** Because a new version is a
**new hash under a new entry**, injection never overwrites a live entry:
- in-flight calls to the old hash **complete on the old code** (it is still loaded, still valid for
  *its* hash);
- new callers — which, being recompiled, reference the **new** hash — dispatch to the new entry.

There is no window in which a caller sees half-old/half-new code. Reclaiming an old unit is safe once
no caller references its hash (the same precise, cycle-free reclamation the value model already
enjoys, RFC-0001 §4.7 immutability). Registration of a single `hash → entry` is the only mutation,
and it is a single-word, publish-once operation (a new key, never an overwrite).

**5. Never-silent at the dispatch boundary (G2/SC-3).** A call to a hash with no compiled entry and
no interpretable definition is an **explicit refusal**, never a guess; a failed unit load is an
explicit error, never a partial registration. Injection is **inspectable/EXPLAIN-able** like every
selection: which hash resolved to which entry (compiled vs interpreted) is queryable.

## Consequences

- **The dev loop the maintainer wants:** interpret what's in flux; compile what's ready; inject a
  recompiled definition into the running image and keep going — without recompiling the unchanged
  rest, and without a relink. The unit of recompile *and* of injection is the **content-addressed
  definition**.
- **Reliability by construction, not by locking.** No read-copy-update of live code, no stop-the-world
  patch — immutability + new-hash-new-entry replace the usual hot-patch machinery.
- **Rides ADR-016's ABI** (the dispatch table + the value boundary) and the deferred MLIR→LLVM codegen
  for *native* units; the in-process JIT (M-340) is the working prototype substrate meanwhile.

## Alternatives considered

- **Whole-image in-place binary patching** (rewrite the running binary). Rejected: platform-specific,
  fragile, and it *re-introduces* the atomicity hazard that immutability otherwise dissolves.
- **Mutable definitions with versioned entries** (overwrite an entry on change). Rejected: it
  reintroduces in-place mutation of live dispatch and the tearing hazard; content-addressing's
  new-hash-new-entry is strictly safer and needs no version field (ADR-016).
- **Recompile-and-relink the whole artifact on any change.** Rejected as the *only* path (it is the
  `--fat`/release build, RFC-0004 §9.2): it defeats rapid iteration; hot-inject is the dev-loop
  complement, not a replacement for a full build.

## Scope / honesty (VR-5)

This ADR decides the **mechanism and its safety argument**; the **native codegen** that produces
injectable units is the deferred MLIR→LLVM backend (RFC-0004 §2), and the cross-process unit format
is OQ-3 (RFC-0004 §10). A **working in-process prototype** is tractable *now* on M-340's `dlopen`
JIT (a hash-keyed dispatch table + load-and-register), and is the recommended first build step once
this ADR is ratified — but it is **not** built by this ADR. Until then, this is a design with an
honest deferral, not a shipped capability.

## Meta — changelog

- **2026-06-16 — Proposed.** Drafts OQ-2 (RFC-0004 §10): hot-inject recompiled definitions via a
  hash-keyed dispatch table (ADR-016) + content-addressed dynamic linking, with the key safety
  argument that **immutability + content-addressing dissolve the atomicity hazard** (a change is a
  new hash under a new entry, never an in-place mutation; in-flight calls finish on old code, new
  callers dispatch to new code) and that the recompile set is **exactly the changed
  dependency-closure** by hash reachability. The M-340 `dlopen` JIT is the prototype substrate;
  native codegen + the cross-process unit format are deferred. Awaiting maintainer ratification
  (Proposed → Accepted). Append-only.
