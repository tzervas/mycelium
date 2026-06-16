# ADR-016 — The interpreted↔compiled ABI: a content-addressed, wire-form value boundary

| Field | Value |
|---|---|
| **ADR** | 016 |
| **Title** | The boundary between interpreted L0 and compiled stable components: dispatch a compiled definition by its **content hash**, and cross values in the **self-describing wire form** (RFC-0001 §4.8), with a zero-copy fast-path as a later optimization |
| **Status** | **Proposed** (drafted 2026-06-16; enacts RFC-0004 r2 §10 **OQ-1**) |
| **Date** | 2026-06-16 |
| **Depends on** | RFC-0004 §4 (stable-component gate) / §9 (interpreted↔compiled continuum) / §10 OQ-1; RFC-0001 §4.6 (content-addressing) / §4.8 (self-describing wire form); ADR-003 (Unison identity); ADR-007 (Rust trusted base); ADR-009 (hybrid execution); M-340 (in-process JIT) |
| **Resolves** | RFC-0004 §10 **OQ-1** (the interpreted↔compiled ABI) |
| **Blocks** | ADR-017 (hot-inject rides this ABI) |

## Context

RFC-0004 §9 makes execution a *per-definition continuum*: interpreted definitions and compiled
stable components coexist in one run. **In-process today** they share Rust value types, so there is
no real boundary. A **persistent compiled-artifact store** — reused across processes and machines
(the "don't recompile what hasn't changed" win, RFC-0004 §9.1) — needs a *stable* boundary: a
**call ABI** (how the interpreter invokes a compiled definition) and a **value ABI** (how a
`CoreValue` crosses). RFC-0004 §10 flagged this as OQ-1; this ADR decides it.

The decisive context is that Mycelium is **content-addressed** (ADR-003): a definition's identity
*is* its hash, and a value's identity is its repr+payload / ctor+fields (RFC-0001 §4.6). That makes
the hard ABI problems — versioning, staleness, identity — dissolve, if the ABI is *keyed on the
hashes that already exist* rather than on ad-hoc layout.

## Decision

**1. Call ABI — dispatch by content hash.** A compiled stable component is invoked by the
**content hash of the definition** it compiles. The artifact store is a map `ContentHash → entry`;
the interpreter, reaching a call to a definition whose hash has a compiled entry, dispatches to it
(otherwise it interprets — the continuum, RFC-0004 §9.1). The signature at the boundary is
uniformly:

```text
call(def: ContentHash, args: [CoreValue]) -> Result<CoreValue, AbiError>
```

**Versioning is free and staleness is structurally impossible (ADR-003).** A compiled entry for
hash `H` is valid *iff* the definition still hashes to `H`. A change to a definition is a **new
hash** (RFC-0001 §4.6), so it gets a new entry — an old compiled entry can never be silently
applied to a changed definition. There is **no version field** to drift; the hash is the version.

**2. Value ABI — the self-describing wire form is canonical (RFC-0001 §4.8).** A `CoreValue`
crosses the boundary in the **already-specified, content-addressed wire form**:
`[Repr] ‖ [Meta] ‖ [payload]` for a representation value, `[CtorRef] ‖ [fields]` for a datum (the
datum's guarantee **summary recomputed from fields**, never trusted from the wire — RFC-0001 §4.2
r3 / §4.8). This is portable (cross-process, cross-machine, cross-language), faithfully
round-trippable (`deserialize(serialize(v)) ≡ v`), and needs no new format — the wire form *is* the
ABI. Honesty rides along: the `Meta`/guarantee crosses with the value, so a compiled component can
never silently drop or upgrade a guarantee (VR-3/VR-5, WF5).

**3. A zero-copy fast-path is a later optimization, not the foundation.** For a same-process,
same-version crossing the boundary may later skip serialization and pass a shared in-memory
representation — but **only** as an optimization *over* the canonical wire ABI, chosen when measured
to matter, never as the primary contract. Robust/portable first (the maintainer's stated priority);
fast-path when it earns it. The fast-path must be observably equivalent to the wire path (the M-210
checker validates it, NFR-7), or it is not admitted.

**4. The boundary is in the toolchain, not the kernel (KC-3).** The trusted base is the reference
interpreter (ADR-007/009); this ABI is how a *separately-compiled* artifact plugs into it. The
artifact store, the dispatch table, and the (de)serialization live in the build/runtime layer
(`mycelium-build` / a runtime crate), depending on `mycelium-core` for `ContentHash` + the wire
form — nothing in the kernel depends on them.

## Consequences

- **Incremental + cacheable for free.** Because the call ABI is hash-keyed and the value ABI is
  content-addressed, a compiled artifact is *never stale* and is reusable across runs/machines with
  no dependency bookkeeping (RFC-0004 §9.1; M-311/M-312 already cache build certificates this way).
- **Honesty crosses the boundary.** `Meta`/guarantee travel in the wire form (WF5), so the §4
  obligations (a compiled component's reference-equivalence to the interpreter) are checkable on the
  observable both paths produce.
- **A serialization cost per crossing** in the canonical path — the price of portability; the
  zero-copy fast-path (decision 3) buys it back where it matters, validated, never assumed.
- **This unblocks hot-inject (ADR-017):** a hash-keyed dispatch table is exactly what injecting a
  recompiled definition registers into.

## Alternatives considered

- **A stable C-ABI in-memory struct layout as the *primary* boundary** (zero-copy first). Fastest,
  but couples compiled code to a fixed memory representation that is fragile across versions and
  harder to evolve — optimizing for speed before the robust portable path exists. **Rejected as the
  foundation** (kept as the decision-3 fast-path, over the wire ABI).
- **A bespoke binary ABI distinct from the wire form.** Rejected: it would duplicate RFC-0001 §4.8
  and risk the two drifting; the self-describing wire form already round-trips faithfully and carries
  `Meta`.
- **A version number on compiled artifacts.** Rejected as redundant: content-addressing *is* the
  version (ADR-003) — a version field would be a second, drift-prone source of truth.

## Scope / honesty (VR-5)

This ADR decides the **ABI shape**; the **codegen** that produces compiled artifacts is the deferred
MLIR→LLVM backend (RFC-0004 §2). Until it lands, the only "compiled" path is the in-process JIT
(M-340), where the boundary is in-process Rust values — this ADR is the contract that the persistent,
cross-process store will honor when the backend arrives. Open follow-ons: the artifact-store
packaging format (RFC-0004 §10 OQ-3) and the fast-path's exact same-process representation (a later
measurement-driven decision).

## Meta — changelog

- **2026-06-16 — Proposed.** Drafts OQ-1 (RFC-0004 §10): dispatch compiled stable components by
  content hash; cross `CoreValue`s in the RFC-0001 §4.8 self-describing wire form (canonical), with a
  zero-copy fast-path as a later, validated optimization. Grounded in ADR-003 (content-addressing
  makes versioning/staleness dissolve) and ADR-007/009 (the boundary is toolchain, not kernel).
  Codegen deferred to the MLIR→LLVM backend (RFC-0004 §2). Awaiting maintainer ratification
  (Proposed → Accepted). Append-only.
