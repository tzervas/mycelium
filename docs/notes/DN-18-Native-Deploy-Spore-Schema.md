# Design Note DN-18 — Native-Deploy Spore Schema

| Field | Value |
|---|---|
| **Note** | DN-18 |
| **Status** | **Draft** (2026-06-20; M-620) — *design-complete, impl-pending* |
| **Feeds** | ADR-013 (spore is the deployable unit); ADR-003 (content-addressed identity); RFC-0004 (execution model — §6 inspectability); ADR-009 (no-opaque-lowering, all backends); DN-15 (native-path decomposition — the execution sibling of this deployment note); M-368 (`mycelium-proj.toml` → spore packaging); M-601 (real native backend); M-630 (the VR-4 cross-backend gate) |
| **Date** | June 20, 2026 |
| **Decides** | *Design note, not a ratified decision.* Records the schema by which a deployable **Spore** (ADR-013) embeds a **natively-compiled artifact** from the `mycelium-mlir` backend, and the two decisions that must be ADRs before the wire-format lands: (1) `mycelium-spore` taking a build-time dependency on `mycelium-mlir`, and (2) the spore wire-schema extension that carries a compiled artifact. Records what is **buildable now** (the crate-local `mycelium_mlir::deploy::NativeArtifact` primitive, landed under M-620) versus what is **impl-pending** (the cross-crate wiring). No spec is promoted to Accepted here. |
| **Task** | M-620 — Deployable Spore units on the native path |

> **Posture (honesty rule / VR-5).** Every claim below is grounded in source code or a cited spec.
> The cross-crate wiring (spore ← mlir) is **not landed here** — it is an ADR-level decision (a
> workspace-manifest change is a decision, not a build detail; CLAUDE.md) and is stated as
> impl-pending, not hidden. The crate-local primitive **is** landed and tested (`crates/mycelium-mlir/src/deploy.rs`).
> Nothing is pre-declared deployable without the native backend actually lowering it; an
> out-of-fragment program is an explicit refusal routed to the proven path (G2), never fragile codegen.

---

## 1. Background: ADR-013, ADR-003, and the M-620 ask

**ADR-013 (Accepted)** makes a **`spore` the content-addressed deployable unit** — a hash-identified
DAG of (1) code, content-addressed by hash (ADR-003, the Unison recipe); (2) values/state with their
`Meta`; (3) the reconstruction manifest (RFC-0003 §6); (4) artifact metadata (provenance, guarantee
certificates, signatures). **ADR-003** fixes that *a definition's identity is its content hash*, so a
compiled artifact keyed by that hash is never stale and is reused across runs/machines; **metadata is
not identity.**

**M-620** asks: *produce a deployable Spore from the native-compiled backend* (`mycelium-mlir`,
M-601), consuming the M-368 packaging layer (`mycelium-proj.toml` → spore). The honesty obligations
(M-620 body): content-addressed identity is canonical (ADR-003); a missing/ambiguous deploy input is
an explicit error, never a guessed default (G2); and the **no-opaque-lowering guarantee (VR-4) holds
end-to-end into the deployed unit**.

The current state (verified against source, 2026-06-20):

- `crates/mycelium-spore/` builds a `Spore { id, kind, surface, sources, deps, name, version }` from
  a `mycelium-proj.toml` (`build_spore`), content-addresses it over **code + deps + surface**
  (`content_address`), and excludes metadata (`name`/`version`) from identity — ADR-003-correct. It
  carries **`.myc` sources by hash**, *not* compiled native artifacts. Its deps are
  `mycelium-proj` + `mycelium-core` + `blake3`; **it does not depend on `mycelium-mlir`.**
- `crates/mycelium-mlir/` is the native backend (direct-LLVM + the M-601 MLIR-dialect path). It is
  **not** a dependency of `mycelium-spore`.

So a Spore today is a *source* deployable; carrying a *natively-compiled* artifact requires (a) the
spore crate to reach the native backend, and (b) the spore wire-format to gain a compiled-artifact
component. Both are decisions, not incidental edits — §4.

---

## 2. What lands now: the crate-local `NativeArtifact` primitive (buildable)

Landed under M-620 in `crates/mycelium-mlir/src/deploy.rs` — the inspectable, content-addressed
descriptor of one natively-compiled program that the Spore layer will embed:

```rust
pub struct NativeArtifact { /* identity, lowered_ir, vr4, faithfulness — all private */ }

impl NativeArtifact {
    pub fn build(node: &Node, identity: ContentHash) -> Result<Self, DeployError>;
    pub fn id(&self) -> &ContentHash;            // ADR-003 identity = the program's content hash
    pub fn lowered_ir(&self) -> &str;            // dumpable LLVM IR — VR-4 evidence (metadata)
    pub fn vr4(&self) -> &CrossBackendGate;      // the M-630 cross-backend no-opaque attestation
    pub fn faithfulness(&self) -> GuaranteeStrength; // Empirical — the differentials (VR-5)
    pub fn same_identity_as(&self, other: &Self) -> bool; // metadata-blind (ADR-003)
    pub fn explain(&self) -> String;             // auditable EXPLAIN at the deployment site
}
```

Design choices, each grounded:

1. **Identity is the program's `ContentHash` (ADR-003), supplied by the caller — *not* recomputed
   from the IR text.** Code identity is the hash of the program, not of its lowering; two builds of
   the same program on different LLVM patch versions are the *same* artifact identity, their carried
   IR merely differing. `same_identity_as` keys on the identity, blind to metadata — the
   "metadata is not identity" rule made executable.
2. **VR-4 travels with the unit (the M-620↔M-630 seam).** The descriptor embeds *both* the program's
   dumpable lowered IR *and* the [`vr4::cross_backend_gate`] attestation (every backend's lowering is
   dumpable — no opaque pass), so the no-opaque-lowering guarantee (RFC-0004 §6 / VR-4) is inspectable
   *at the deployment site*, not only at build time. This is exactly M-620's "VR-4 holds end-to-end
   into the deployed unit".
3. **Never-silent (G2), in its strongest form.** The deploy `identity` is a `ContentHash`, a
   *validated, non-empty* type — a missing/malformed identity is **unrepresentable**, not a runtime
   branch (CLAUDE.md banked guard 2). The remaining failure is a program the native backend cannot
   lower soundly: an explicit `DeployError::NotDeployable` carrying the backend's own EXPLAIN reason,
   routed to the proven path — fragile codegen is **never** shipped (G2/VR-5). (Verified: a `Swap` to
   a non-bit/ternary repr is refused; trit-carry arithmetic, `Construct`/`Match`, bit ops *are*
   deployable on the direct-LLVM backend — the artifact covers more than the element-wise fragment,
   honestly.)
4. **Honest tag (VR-5):** `faithfulness = Empirical` — the lowered IR is the real artifact, its
   semantics evidenced by the interp↔native differentials (M-302/M-602); never `Proven` (no
   machine-checked end-to-end deployment-correctness theorem).

This primitive is the **unit a Spore embeds**; it does not itself reach into the spore crate (that is
§4's deferred wiring), so it lands collision-free inside `mycelium-mlir`.

---

## 3. The target schema: how a Spore embeds a native artifact (design)

When the cross-crate wiring (§4) is sanctioned, a deployable native Spore is the ADR-013 DAG with one
**added component** — a content-addressed compiled-artifact reference:

```text
Spore (native-deploy form)
├── code        — content-addressed .myc sources by hash          (UNCHANGED, ADR-013 #1)
├── values      — initial/captured state with Meta                 (UNCHANGED, ADR-013 #2)
├── manifest    — the reconstruction manifest (RFC-0003 §6)        (UNCHANGED, ADR-013 #3)
├── metadata    — provenance, guarantee certs, signatures          (UNCHANGED, ADR-013 #4)
└── native      — { artifact_id: ContentHash,    ← = the code identity it compiles (ADR-003)
                    target_triple: String,        ← metadata (which machine the .so/.o targets)
                    vr4_attestation: <digest>,    ← the M-630 no-opaque cross-backend gate result
                    lowered_ir_digest: <digest> } ← metadata (the dumpable IR's hash, for audit)
                  }                                  (NEW — the M-620 native-deploy component)
```

**Identity rule (ADR-003), the load-bearing design decision.** The `native` component's
`artifact_id` **is** the code identity it compiles — so it does **not** add a new identity axis: a
native Spore and a source Spore of the *same program* share the code identity; the `native` component
is **metadata on top** (which target, which lowering, the VR-4 attestation digest). The spore's own
content hash (`content_address`) is therefore **left unchanged** — `target_triple`, the IR digest,
and the attestation are *not* folded into spore identity (they are build-target metadata, exactly as
`name`/`version` are excluded today). This keeps "two builds of the same program for two targets are
the same deployable code, with two native attachments" — the Unison/ADR-003 invariant. *(This is the
single most important decision to ratify; an ADR amending ADR-013 should state it explicitly.)*

**VR-4 into the deployed unit.** The `vr4_attestation` digest is the hash of the M-630
`CrossBackendGate::explain()` (byte-deterministic — pinned in `tests/vr4_cross_backend.rs`), so the
deployed unit carries a verifiable "no opaque pass on any backend" claim that a consumer can
re-derive and check. This is the end-to-end VR-4 obligation M-620/M-630 require, realized as carried,
content-addressed evidence rather than a build-time-only property.

---

## 4. What is impl-pending (the two ADR-level decisions — FLAGGED, not landed)

Both are deliberately **not** done in this note's task; each is an ADR-level change owned above the
`mycelium-mlir` crate boundary (workspace manifest + spore crate + wire-schema), and is flagged up:

1. **`mycelium-spore` → `mycelium-mlir` build dependency.** Embedding a compiled artifact means the
   spore crate must reach the native backend. This is a **workspace `Cargo.toml` + `mycelium-spore/Cargo.toml`
   change** — a dependency decision, *not* a build detail (CLAUDE.md: "don't silently … that's a
   decision (ADR)"). It also widens the spore crate's compile graph (and its `blake3`/LLVM-tooling
   surface). **Decision required:** an ADR sanctioning the dependency direction (spore depends on the
   backend, not vice-versa) and confirming it does not create a cycle (mlir does **not** depend on
   spore/proj — verified, so the direction is acyclic).

2. **The spore wire-schema extension (the `native` component) + its `docs/spec/schemas/` schema.**
   The current spore encoding is the "named-provisional v0" text (`bin/spore.rs`); a binary/JSON wire
   form that carries the `native` component (and a `physical-layout`-style committed schema with
   `#[serde(deny_unknown_fields)]` + on-deserialize re-validation — CLAUDE.md banked guard 3) is an
   RFC-0008-class artifact, **deferred per ADR-013 §4** (the spore wire-format is future work). The
   schema in §3 is the design input for that RFC; it is not committed as a schema here.

Until those two land, the **`NativeArtifact` primitive (§2) is the complete, tested, crate-local
realization** — it produces exactly the content the `native` component will carry, so the cross-crate
step is wiring, not new design.

---

## 5. Status & grounding summary

| Item | Status | Grounding |
|---|---|---|
| `NativeArtifact` descriptor (identity, dumpable IR, VR-4 attestation, Empirical tag) | **Landed** (M-620) | `crates/mycelium-mlir/src/deploy.rs`; ADR-013/ADR-003; RFC-0004 §6; VR-4/VR-5 |
| Content-addressed identity = program hash; metadata not identity | **Landed** (in the primitive) | ADR-003; `same_identity_as` |
| VR-4 carried into the deployed unit | **Landed** (in the primitive) | M-630 `cross_backend_gate`; RFC-0004 §6 |
| Missing/ambiguous input → explicit refusal (G2) | **Landed** | `ContentHash` non-empty by type; `DeployError::NotDeployable` |
| Spore `native` component schema | **Design-complete, impl-pending** | §3; needs the RFC-0008 wire-format (ADR-013 §4 deferral) |
| `mycelium-spore` ← `mycelium-mlir` dependency | **Impl-pending (ADR-level)** | §4.1; CLAUDE.md decision rule — **FLAGGED up** |

This note is **design-complete** for M-620: the deployable-artifact primitive is landed and tested,
the Spore-embedding schema is specified, and the two cross-crate decisions are recorded as
impl-pending ADR-level work rather than smuggled into a build. Honest boundary throughout (G2/VR-5).

---

## Meta — changelog

<!-- changelog: 2026-06-20 Draft created (M-620) — records the native-deploy Spore schema: the crate-local NativeArtifact primitive (landed in mycelium-mlir/src/deploy.rs — content-addressed identity = program hash, dumpable IR + the M-630 VR-4 attestation carried into the deployed unit, Empirical tag, never-silent refusal), the target Spore `native`-component schema (design-complete), and the two impl-pending ADR-level decisions (spore ← mlir dependency; the wire-schema extension, deferred per ADR-013 §4). Design-complete, impl-pending. Append-only. -->
