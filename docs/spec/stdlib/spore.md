# Spec — `std.spore` (the content-addressed deployable unit + the reconstruction-manifest library)

| Field | Value |
|---|---|
| **Status** | **Accepted (library/manifest half)** (2026-06-20, maintainer-ratified per DN-07 — guarantee matrix asserted in tests; §7-Q1 ring reconciled via the RFC-0016 §4.2 erratum; remaining §7 questions are scope/cross-phase calls, not contract violations; was *Implemented (Rust-first, library/manifest half) — pending ratification* 2026-06-18, Draft/needs-design 2026-06-17) — the **library / reconstruction-manifest half** landed as `mycelium-std-spore` (M-522, #163, Batch P5 Tier-A completion): build/identity/manifest/verify over the `mycelium-spore` packager + `std.content` hash + `std.vsa` decode (KC-3 — no new hash, no new trusted code). Identity is the canonical content hash and metadata-invariant (ADR-003); a hash mismatch is an explicit `Err` (C1/G2); probabilistic regrowth is held at the **`Empirical` ceiling** (FR-C2/VR-5, never `Proven`), enforced structurally. The **full native deploy / germination** stays **Phase-6-gated (M-620)** and out of this wave (FLAG §7 Q2). |
| **Module / Ring** | `std.spore` · Ring 1 · Tier A (RFC-0016 §4.2/§4.3 — §7-Q1 ring placement **reconciled to Ring 1** by the RFC-0016 §4.2 erratum, 2026-06-20). |
| **Tracks** | `M-522` (#163) — the Phase-5 task this spec delivers |
| **Scope** | The library form of the **content-addressed deployable unit** (ADR-013): build a `spore` (a hash-identified DAG of code + values + reconstruction manifest + artifact metadata) with a **deterministic content hash**, and author/inspect the RFC-0003 §6 **reconstruction manifest** (the regrow-a-value-from-its-recipe recipe). The narrow `spore(v)` single-value case is the degenerate point of the same surface (ADR-013 §2). |
| **Boundary** | Content-hash **primitives** (`hash_of_value`/`hash_of_def`/`ContentHash`) are `std.content` (M-523) — `spore` **consumes** them, never re-hashes. The reconstruction **operations** (VSA bind/unbind/cleanup/resonator decode) and their **probabilistic ceiling** are `std.vsa` (M-513) — `spore` *packages* a manifest, `vsa` *performs* the regrowth. The **packaging schema/CLI** (`mycelium-proj.toml → spore`, the never-silent publish checks) is owned by **M-368** (`crates/mycelium-spore`, the Accepted/enacted Spore Build & Publish Contract) — `spore` is its library face, not a second source of the schema. The **full native deploy / germination** lands with the Phase-6 native path **M-620** — FLAGGED §7 Q2. A representation change is `std.swap` (M-516). |
| **Depends on** | ADR-013 (`spore` is the content-addressed deployable unit; the manifest is one digest-referenced component; narrow = degenerate); ADR-003 (content-addressed identity is canonical; metadata ≠ identity); RFC-0003 §6 (the reconstruction manifest — contents, modes, schema); the M-368 Spore Build & Publish Contract (`mycelium-proj.toml → spore`, identity-vs-metadata, never-silent publish inputs, EXPLAIN); RFC-0016 §4.1 (the C1–C6 contract), §4.3 (`spore` row), §4.5 (the guarantee matrix); RFC-0001 §4.6 (content-addressing / `hash(def)`), §4.3 (`Meta`, `ReconInfo`); RFC-0014 (declared, bounded effects). |
| **Grounds on** | `std.content` (M-523 — the `ContentHash` ref + `hash_of_*`); `crates/mycelium-spore` (M-368, enacted — `build_spore`/`explain` + CLI over the M-359 `mycelium-proj` reader); `crates/mycelium-proj` (M-359 — the manifest reader, `[surface]`/`[dependencies]`/`[spore]` tables); the `reconstruction-manifest` schema (M-010, ratified; `docs/spec/schemas/reconstruction-manifest.schema.json`) and `mycelium-proj` schema (M-359). KC-3: above the kernel — no new trusted code, no new hash. |

---

## 1. Summary

`std.spore` is the deployable-unit and reconstruction-manifest model made into an ergonomic, value-semantic
library. Its user-facing surface lets a program **build** a `spore` from a project (or from a single value),
ask for its **content hash**, **author** and **inspect** a reconstruction manifest, and **EXPLAIN** exactly
what went into the artifact and why. Its **honesty crux** is two-sided. First, **C4/ADR-003**: a spore's
identity *is* the content-addressed hash of its component DAG (code + values + manifest-by-digest +
dependency-hash edges); **metadata is not identity** — `version`/`authors`/`summary` travel with the spore but
never define it, so the build hash is *deterministic* and metadata-invariant. Second, the **reconstruction
honesty ceiling (FR-C2/VR-5)**: a manifest whose `decode` bottoms out in a probabilistic VSA recovery
(resonator factorization) **inherits that ceiling** — its `bound.strength` MUST NOT exceed `Empirical`, and
`spore` **never** claims exact regrowth where the recipe is probabilistic. Its structural never-silent crux
(**C1/G2**) is that a missing or ambiguous publish/deploy input — a surfaceless phylum, a hashless dependency,
a `version`/`hash` disagreement, an `include` naming a non-export, or a deploy-time hash mismatch — is an
**explicit, traceable error naming the offending input**, never a guessed default and never a partial
artifact. It is Ring 1 / Tier A, and it adds **no trusted code** (KC-3): it consumes `std.content`'s hash,
`std.vsa`'s decode, and the M-368 packager; it mints no new hash and performs no reconstruction itself.

## 2. Scope & module boundary

- **In scope:**
  - **Build** a `spore` from a `mycelium-proj.toml` project — the library face of the M-368 pipeline:
    determine the germination surface (`[surface].exports` or `[spore].include`), gather the reachable
    content-addressed code, resolve `[dependencies]` to concrete hashes, assemble the ADR-013 four-component
    DAG, and content-address it into the **deterministic spore hash** (`blake3:…`).
  - **The degenerate `spore(v)` single-value case** (ADR-013 §2): construct the spore whose payload is `v`'s
    reconstruction manifest — the narrow sense, reachable from the same surface.
  - **Author / read / validate** a reconstruction manifest (RFC-0003 §6): the `{ mode, model, dim, codebooks
    (content-addressed), recipe?, decode, bound }` record — schema-valid by construction, with the
    `IndexedRetrieval` vs `CompositionalReconstruction` mode an explicit, inspectable choice.
  - **EXPLAIN** the build/deploy decision: the identity receipt (what code, which deps, which surface) plus
    the not-identity metadata, as a total function of the manifest + resolved DAG (no learning).
  - **Inspect** a built spore: its identity hash, its component digests, its declared metadata.
- **Out of scope (and who owns it):**
  - **Content-hash primitives** — `ContentHash`, `hash_of_value`, `hash_of_def`, `digest_eq`. Owned by
    **`std.content` (M-523)**; `spore` *embeds* these refs and asks `content` for identities, it never
    re-implements the digest (RFC-0001 §4.6; M-103).
  - **Reconstruction operations + the probabilistic ceiling** — the VSA `bind`/`unbind`/`bundle` and cleanup,
    and the resonator decode that actually *regrows* a value, and the FR-C2 "probabilistic-only, basis ≤
    `Empirical`" guarantee. Owned by **`std.vsa` (M-513)** (RFC-0003/0009). `spore` packages the *manifest*
    (the recipe + the bound certificate); `vsa` *runs* it. The honesty tag on a regrowth is `vsa`'s to
    establish; `spore` only carries it faithfully.
  - **The packaging schema + publish CLI** — the `mycelium-proj.toml → spore` field set, the never-silent
    publish checks, the `spore build`/`spore explain` CLI. Owned by **M-368** (the Accepted/enacted Spore
    Build & Publish Contract; `crates/mycelium-spore`). `spore` is its **library face** and **does not invent
    schema fields** — it consumes whatever M-359/M-368 fix (FLAGGED §7 Q3).
  - **The full native deploy / germination contract** — turning a spore into a running colony on the native
    backend. Lands with the **Phase-6 native path (M-620)**; the artifact wire-schema, signing, and
    germination are deferred there (ADR-013 §4; M-368 §7 v0 scope). FLAGGED §7 Q2.
  - **Representation change** — `std.swap` (M-516). `spore` reports and packages identity; it never converts a
    value's `Repr`.
- **Ring & layering:** Ring 1 (Tier A capability surface, RFC-0016 §4.3). `spore` **wraps** the M-368
  `build_spore`/`explain` entry points and the `reconstruction-manifest` authoring/validation in an
  ergonomic, value-semantic surface; it **consumes** `std.content`'s hash and (for the regrowth path)
  `std.vsa`'s decode. No new trusted code, no new hash, no `wild`/FFI (KC-3). *Ring placement is FLAGGED §7 Q1:
  RFC-0016 §4.2 currently lists `spore` under Ring 2 while §4.3/the stdlib index files it under Tier A;
  this spec follows the Tier-A placement and surfaces the discrepancy rather than silently choosing.*

## 3. Exported-op surface (design sketch)

A value-semantic sketch — enough to fix the surface and feed the guarantee matrix, not a committed grammar.
`Spore`, `ContentHash`, and `ReconManifest` are immutable values; fallible ops return `Result` with an
explicit, named error; effectful ops (build/publish/deploy read the project + write the artifact) declare
their effect (C6). `ContentHash` is **re-used from `std.content`**, not minted here.

```
// illustrative signatures (not a committed surface)

// --- the value types ---
opaque type Spore            // a built artifact: identity hash + component DAG handle (ADR-013)
opaque type ReconManifest    // the RFC-0003 §6 record { mode, model, dim, codebooks, recipe?, decode, bound }
opaque type SporeExplain     // the EXPLAIN receipt: surface + code + deps + values + metadata (not-identity)
type ContentHash             // re-exported from std.content (M-523); identity of the DAG

// --- build (the M-368 pipeline, library face) — declares io (reads project, writes artifact) ---
fn build(project: &ProjPath) -> Result<Spore, PublishErr>   io   // C1: every ambiguous input -> explicit Err
fn build_value(v: &Value)    -> Result<Spore, PublishErr>        // ADR-013 degenerate spore(v); manifest payload
fn identity(s: &Spore)       -> ContentHash                      // total: the deterministic spore hash
fn explain(project: &ProjPath) -> Result<SporeExplain, PublishErr> io  // §6 receipt; writes nothing

// --- the reconstruction manifest (RFC-0003 §6) — authored/validated, NOT executed here ---
fn manifest_of(s: &Spore)    -> Option<ReconManifest>           // None when the spore carries no manifest
fn validate(m: &ReconManifest) -> Result<(), MalformedManifest> // schema + invariant check (C1)
fn manifest_hash(m: &ReconManifest) -> ContentHash             // total: deterministic digest of the recipe
fn mode(m: &ReconManifest)   -> ReconMode                       // IndexedRetrieval | CompositionalReconstruction
fn declared_strength(m: &ReconManifest) -> Strength            // the bound's tag — never above Empirical for Resonator

// --- deploy (Phase-6 native path, M-620) — FLAGGED §7 Q2; declares io ---
fn deploy(s: &Spore, target: &Target) -> Result<Deployed, DeployErr>  io  // C1: hash-mismatch on deploy -> explicit Err

enum PublishErr {
  NoProject, MissingName, MissingKind,           // M-359 manifest errors
  SurfaceUnstated,                               // a phylum with no [surface].exports and no [spore].include
  IncludeNotAnExport(NoduleId),                  // [spore].include names a non-existent / non-export nodule
  HashlessDependency(PhylumId),                  // a dep without a content hash (unpinned, non-reproducible)
  VersionHashDisagreement(PhylumId),             // [dependencies] version requirement vs resolved hash conflict
  DependencyCycle(Vec<PhylumId>),                // a spore is a DAG, not a cycle
  NoSources, OutOfSubsetManifest                 // no .myc sources; an out-of-subset manifest construct
}
enum DeployErr { HashMismatch{ expected: ContentHash, got: ContentHash }, TargetUnavailable, Unsupported }
enum MalformedManifest { BadMode, BadBound, ResonatorOverStrength, MissingRecipe, MissingDecodeParam }
```

> **Note (design choice).** The `deploy`/`Deployed`/`Target` triple and the germination contract are sketched
> to fix the *seam*, not the surface — they are **M-620's** to define (Phase-6 native path). This spec commits
> only that a deploy **hash mismatch is an explicit `DeployErr::HashMismatch` naming both hashes**, never a
> silent partial deploy. See §7 Q2.

## 4. Guarantee matrix (the load-bearing deliverable — RFC-0016 §4.5)

Rows = exported ops, grouped by build / publish / reconstruct / deploy. To be encoded as a checked table
(the RFC-0003 §4 template) and asserted in tests once code lands — never prose only. **Tag legend:** `Exact`
= deterministic, no accuracy semantics (a content hash is a pure function of normalized structure, RFC-0001
§4.6); a reconstruction row **carries the tag `std.vsa` establishes** and is **bounded by `Empirical`** when
the decode is the probabilistic resonator path (FR-C2). `EXPLAIN-able? = yes` means a reified inspectable
artifact exists (the identity receipt / the manifest record / the refusal diagnostic).

| Op | Guarantee tag | Fallibility (explicit error set) | Declared effects | EXPLAIN-able? |
|---|---|---|---|---|
| `build` (project → spore) | `Exact` (deterministic hash; metadata-invariant) | `Err(PublishErr::*)` — surfaceless, hashless dep, version/hash disagreement, bad include, cycle, no sources | `io` (reads project; writes artifact) | yes (the §6 identity receipt) |
| `build_value` (`spore(v)`, degenerate) | `Exact` (deterministic hash) | `Err(PublishErr::*)` (the value-payload subset) | `io` (writes artifact) | yes (receipt) |
| `identity` (spore hash) | `Exact` (deterministic) | total | none | yes (the hash *is* the receipt) |
| `explain` | `Exact` (deterministic; a total function of manifest + DAG) | `Err(PublishErr::*)` (same checks as `build`, writes nothing) | `io` (reads project) | yes (it *is* the EXPLAIN) |
| `manifest_of` | `Exact` (deterministic) | `None` when the spore carries no manifest (never a fabricated empty one) | none | yes (the manifest record) |
| `validate` (manifest) | `Exact` (deterministic schema + invariant check) | `Err(MalformedManifest::*)` — bad mode/bound, missing recipe/decode param, **resonator over-strength** | none | yes (the diagnostic naming the violated field) |
| `manifest_hash` | `Exact` (deterministic) | total | none | n/a |
| `mode` / `declared_strength` (read) | `Exact` (deterministic) | total | none | n/a |
| **`reconstruct` (the regrowth — `std.vsa` op, shown for the seam)** | **`Empirical` (probabilistic decode) / `Exact` (indexed-retrieval exact-hit only)** — *owned by `std.vsa` (M-513), bounded by FR-C2; `spore` carries, never sets it* | `Err`/refusal on non-convergence (a VSA op; `spore` does not perform it) | (vsa's) | yes (the carried `{ε,δ,strength}` bound + decode params) |
| `deploy` (Phase-6, M-620) | `Exact` (deploy verifies the deterministic hash) — *FLAGGED §7 Q2* | `Err(DeployErr::HashMismatch{expected, got})`, `TargetUnavailable`, `Unsupported` | `io` (network/native deploy) | yes (the deploy receipt + the mismatch record) |

**Tag justification (VR-5 — downgrade rather than overclaim):**
- **`Exact` build/identity/explain rows.** A spore's identity is the content-addressed hash of its component
  DAG (ADR-013; ADR-003), and a content hash is a *pure function of normalized structure* (RFC-0001 §4.6).
  There is no accuracy/precision/probability semantics in *building or addressing* an artifact, so the lattice
  floor `Exact` applies directly. The substantive claim is **determinism + metadata-invariance**: the same
  code + values + resolved-dep-hashes produces the **same** spore hash regardless of `version`/`authors`/
  `summary` (the M-368 contract §2/§8, tested there). `build`/`explain` are not `Result`-free only because
  their *inputs* can be ambiguous (the `PublishErr` set) — the *hash function itself* is total and exact.
- **The reconstruction row inherits the VSA ceiling (the honesty crux, FR-C2).** A manifest's `decode` is
  either `IndexedRetrieval` (codebook + cleanup threshold; returns a *stored* atom; bounded-lossy) or
  `CompositionalReconstruction` (recipe + algebraic inverse; can recover *novel* combinations — and, for
  *unknown* factors, the **resonator** path, which is **not guaranteed to converge** and is
  **probabilistic-only**). When the decode is the resonator path, the `bound.strength` **must not exceed
  `Empirical`** — the `reconstruction-manifest` schema *enforces* this (`A6` / FR-C2), and `ReconInfo::new`
  rejects an over-strength basis. `spore` therefore **never** tags a probabilistic regrowth `Proven`/`Exact`,
  and `validate` raises `MalformedManifest::ResonatorOverStrength` on any attempt to author one. `spore`
  *carries* the tag `std.vsa` establishes; it does not set or upgrade it (VR-5).
- **The `deploy` row is honest-by-verification, not honest-by-assumption.** Deploy re-checks the deterministic
  spore hash at the target; a mismatch (the shipped bytes do not hash to the claimed identity) is an explicit
  `DeployErr::HashMismatch{expected, got}` naming both digests — never a silent overwrite, never a partial
  deploy. The row's `Exact` tag is the *verification*'s tag, and the whole row is **FLAGGED §7 Q2** pending
  M-620 (the native-deploy half does not exist yet).
- **No silent default, no partial artifact anywhere (C1/G2).** Every `PublishErr` names the offending input;
  on any error the build aborts *before* emitting an artifact (M-368 §5: "no partial spore is ever written").
  A deploy hash-mismatch aborts before germination. The explicit error *is* the never-silent guarantee.

## 5. §4.1 contract conformance (C1–C6)

- **C1 — never-silent (G2).** Every missing/ambiguous publish input is an explicit, **named** error, never a
  guessed default: a surfaceless phylum → `SurfaceUnstated`; a hashless dependency → `HashlessDependency`; a
  `version`/`hash` disagreement → `VersionHashDisagreement`; an `[spore].include` naming a non-export →
  `IncludeNotAnExport`; a dependency cycle → `DependencyCycle`; no `.myc` sources → `NoSources` (M-368 §5,
  tested). On the reconstruct side, an unconvergent/over-strength manifest is refused, not silently downgraded
  to a plausible value. On deploy, a **hash mismatch is `DeployErr::HashMismatch` naming both hashes**. **No
  partial artifact is ever written** on any error, and **no partial deploy** ever lands.
- **C2 — honest per-op tag (VR-5).** Build/identity/manifest-authoring rows are `Exact`/deterministic (the
  lattice floor; nothing to overclaim). The **reconstruction** row carries *exactly* the tag `std.vsa`
  establishes and is **bounded by `Empirical`** for the probabilistic resonator decode (FR-C2); `spore`
  **never** claims exact regrowth where the recipe is probabilistic, and downgrades to refuse rather than
  upgrade. The schema and `ReconInfo::new` enforce the ceiling; `spore`'s `validate` surfaces a violation as
  `MalformedManifest::ResonatorOverStrength`.
- **C3 — no black boxes / EXPLAIN (SC-3/G11).** `explain` prints, deterministically, the **identity receipt**:
  the spore hash, the germination surface, the code reachable from it, the resolved dependency hashes (with
  the satisfied version), the captured values, and the **not-identity** metadata explicitly marked as such
  (M-368 §6). The reconstruction manifest is itself the inspectable artifact — mode, codebooks (by content
  hash), recipe/role schema, decode procedure + params, and the `{ε,δ,strength}` bound certificate are all
  reified (RFC-0003 §6). A publish/deploy refusal carries an RFC-0013 diagnostic record naming the offending
  input. No opaque heuristic decides a user-visible outcome.
- **C4 — content-addressed, value-semantic (ADR-003).** This is the module's *raison d'être*. A spore's
  identity **is** the content-addressed hash of its component DAG (code by hash + values + manifest-by-digest +
  dependency-hash edges); **metadata is not identity** — `version`/`authors`/`summary`/`license` travel with
  the artifact but never define it, so two builds of the same code+values+deps **collide on the same spore
  hash** (correct, deterministic), and a metadata-only change **preserves** identity (M-368 §2/§8, ADR-003).
  `Spore`/`ReconManifest` are immutable values; every op is a pure function of its inputs + declared effects.
- **C5 — above the kernel (KC-3).** `spore` adds **no trusted code and no new hash**: it consumes
  `std.content`'s `ContentHash`/`hash_of_*` (themselves re-exporting the M-103 kernel digest), wraps the
  M-368 `build_spore`/`explain` (which already use the workspace-pinned `blake3` + `mycelium-core::ContentHash`,
  no new dependency), and reads the `reconstruction-manifest` schema. The reconstruction *compute* is
  `std.vsa`'s. No `wild`/FFI is introduced at the `spore` layer (ADR-014).
- **C6 — declared, bounded effects (RFC-0014).** The pure ops (`identity`, `manifest_of`, `validate`,
  `manifest_hash`, `mode`, `declared_strength`) are `effects: none`. The build/publish/deploy ops **declare
  `io`** on their signatures — `build`/`explain` read the project (and `build` writes the artifact); `deploy`
  performs the native-deploy IO (M-620). No undeclared side effect; the build performs no network resolution
  in v0 (hash-pinned only — M-368 §7), so there is no ambient network read to hide.

## 6. Grounding

- **The deployable-unit definition + narrow=degenerate framing** — **ADR-013** (Accepted): `spore` is the
  content-addressed, hash-identified DAG of (1) code by hash, (2) values with `Meta`, (3) the RFC-0003 §6
  reconstruction manifest as one digest-referenced component, (4) artifact metadata; `spore(v)` constructs the
  degenerate single-value spore (ADR-013 §2). Grounded by research T4.3 (Unison ship-by-hash) / T4.4 (Nix /
  OCI / Wasm content-addressed deployable DAGs).
- **Identity = the content-addressed DAG; metadata ≠ identity** — **ADR-003** (Accepted;
  `docs/Mycelium_Project_Foundation.md` §5) and the **M-368 Spore Build & Publish Contract** §2/§8 (the
  identity-vs-metadata table; "metadata travels with the spore but never defines it"; same code+deps ⇒ same
  spore hash; tested in `crates/mycelium-spore`). The content hash is a pure function of normalized structure
  (**RFC-0001 §4.6**; M-103).
- **The reconstruction manifest + the probabilistic ceiling** — **RFC-0003 §6** (Accepted r4): the
  `IndexedRetrieval` vs `CompositionalReconstruction` distinction; the minimal manifest contents (model, dim,
  content-addressed codebooks, recipe/role schema, decode procedure + params, the `{ε,δ,strength}` bound);
  **factorization is Phase-3 exploratory, probabilistic-only, basis ≤ `Empirical` (FR-C2)**. The
  `reconstruction-manifest` schema (M-010, ratified; `docs/spec/schemas/reconstruction-manifest.schema.json`)
  *enforces* the Resonator invariants and the `Empirical` ceiling. The regrowth itself is **RFC-0009 / `std.vsa`
  (M-513)**.
- **The packaging contract + the manifest source** — **M-368** (the Accepted/enacted Spore Build & Publish
  Contract; `crates/mycelium-spore`): the build pipeline, dependency resolution (hash-authoritative, a hashless
  or disagreeing dep is an explicit error), the never-silent publish checks (no partial artifact, G2), the
  EXPLAIN receipt, and the honest **v0 scope** (single-project, hash-pinned; wire-schema/signing/germination
  deferred to RFC-0008 R2 / M-620). The `mycelium-proj.toml` shape is **M-359** (ratified;
  `docs/spec/schemas/mycelium-proj.schema.json`).
- **The per-op contract C1–C6, the ring/tier placement, the matrix obligation** — **RFC-0016** §4.1 (C1–C6),
  §4.3 (the `spore` row: "the content-addressed deployable / reconstruction-manifest library"), §4.5 (the
  guarantee matrix). The ring discrepancy (§4.2 Ring 2 vs §4.3 Tier A) is FLAGGED §7 Q1.
- **The value model + house rules** — **RFC-0001** (`Value`/`Repr`/`Meta`, §4.3 `ReconInfo`, §4.6
  content-addressing); **RFC-0014** (declared, bounded effects); **VR-5** (honest tags), **G2** (never-silent),
  **G11** (dual projection), **KC-3** (small kernel), **FR-C2** (probabilistic VSA is opt-in, never in the
  kernel, ≤ `Empirical`), **ADR-003** (metadata is not identity).
- **The cross-phase native-deploy dependency** — **M-620** (Phase-6 native path): `docs/planning/phase-5.md`
  records "spore deploy fully lands with the Phase-6 native path (M-620)"; ADR-013 §4 names the wire-schema,
  signing, and germination contract as obligations on the RFC-0008 runtime stages, not this spec. FLAGGED §7 Q2.

## 7. Open questions (FLAGGED — resolve before ratification)

- **(Q1 — RESOLVED 2026-06-20) Ring placement — Ring 1 (Tier A).** RFC-0016 **§4.3** lists `spore` among the
  **Tier A** differentiator modules (and the stdlib index files it there), while RFC-0016 **§4.2** *had* filed
  `spore` under **Ring 2** ("Tier B + `runtime`/`spore`"). **Resolved in the ratification pass:** `spore` is a
  capability surface over the landed `mycelium-spore` + `mycelium-content` crates (a certificate/EXPLAIN
  consumer, no new trusted code — the Ring-1 definition), so the maintainer-authorized **RFC-0016 §4.2 erratum
  (2026-06-20)** reconciled it to **Ring 1, Tier A** (the §4.2 parenthetical was the outlier; §4.3 was already
  correct). Corrigendum only — not a decision reversal; the §4.3 Tier-A taxonomy is unchanged.
- **(Q2) The Phase-6 native-deploy half (M-620) — the cross-phase dependency.** The **full native deploy /
  germination** (turning a spore into a running colony on the native backend) lands with the Phase-6 native
  path **M-620**, and `docs/planning/phase-6.md` **does not yet exist**. The artifact wire-schema, signing, and
  germination contract are deferred there (ADR-013 §4; M-368 §7 names them as new obligations on the RFC-0008
  R2 stages). This spec commits only the **library/build half** (deterministic hash, manifest authoring,
  never-silent publish inputs) and the **deploy seam's honesty** (a deploy hash-mismatch is an explicit
  `DeployErr::HashMismatch`). — *Disposition: FLAGGED as the load-bearing cross-phase dependency; the `deploy`
  matrix row and the `Target`/`Deployed`/germination types are M-620's to fill, never invented here.*
- **(Q3) The packaging schema fields are M-368/M-359's, not this spec's.** This spec **does not invent**
  `mycelium-proj.toml` fields or `spore` schema fields — it consumes the M-359 `[surface]`/`[dependencies]`/
  `[spore]` tables and the M-368 build contract. The `[spore].include` vocabulary (v0: `["surface"]` + explicit
  nodule lists; richer selectors deferred — M-368 §9.2) and the v0 on-disk encoding (named-provisional,
  superseded by the R2 wire-schema — M-368 §9.1) are **M-368's** open items, carried here by reference.
  — *Disposition: FLAGGED; `std.spore`'s surface tracks whatever M-368/M-359 fix, and is re-checked when the R2
  wire-schema lands.*
- **(Q4) The `spore` ↔ `vsa` reconstruction seam — who owns the regrowth entry point.** `spore` **packages and
  validates** the reconstruction manifest; **`std.vsa` (M-513) performs** the decode (cleanup / resonator) and
  **owns the probabilistic ceiling** (FR-C2). Whether the user-facing `reconstruct(spore) -> Result<Value, …>`
  entry point lives in `std.spore` (delegating to `vsa`) or in `std.vsa` (consuming a `spore` manifest) is a
  placement call for the two specs to agree. This spec assumes **`vsa` owns the regrowth op** and `spore`
  exposes only `manifest_of`/`validate`/`declared_strength`. — *Disposition: FLAGGED; co-design with M-513,
  ties to RFC-0016 §8-Q1 (the module-set split).*
- **(Q5) Ergonomics vs the contract for the identity/manifest at the call site.** How implicit may the spore
  hash / the reconstruction tag be at a call site (e.g. an auto-derived identity for any value placed in a
  spore, or an implicitly-carried bound) vs. always-explicit? — *Disposition: FLAGGED; the `spore` instance of
  RFC-0016 §8-Q3 (ergonomics-vs-contract, tension A); default to always-explicit until the per-ring ergonomics
  pass.*

## Meta — changelog

- **2026-06-17 — Draft (needs-design).** Stands up `std.spore` (M-522, #163) — the **content-addressed
  deployable unit + reconstruction-manifest library** under RFC-0016 (Draft): Ring 1 / Tier A. Fixes the scope
  and boundary (consumes `std.content`'s hash (M-523) and `std.vsa`'s decode (M-513); is the library face of the
  Accepted/enacted M-368 packaging contract; defers the full native deploy to the Phase-6 native path M-620;
  representation change is `swap`), the exported-op surface sketch (`build`/`build_value`/`identity`/`explain`;
  manifest `manifest_of`/`validate`/`manifest_hash`/`mode`/`declared_strength`; the deploy seam), and — the
  load-bearing deliverable — the per-op **guarantee matrix**: build/identity/manifest-authoring rows tag
  `Exact`/**deterministic + metadata-invariant** (ADR-003 — same code+deps ⇒ same spore hash, version/authors
  excluded); the **reconstruction row inherits the VSA ceiling** and is **bounded by `Empirical`** for the
  probabilistic resonator decode (FR-C2/VR-5 — `spore` **never** claims exact regrowth where the recipe is
  probabilistic); every missing/ambiguous publish input (surfaceless phylum, hashless dep, version/hash
  disagreement, bad include, cycle, no sources) and every deploy **hash mismatch** is an **explicit, named,
  traceable error**, **never a guessed default and never a partial artifact** (C1/G2). §4.1 conformance
  (C1–C6) stated concretely; grounding traces to ADR-013/003, RFC-0003 §6, the M-368 contract +
  `reconstruction-manifest`/`mycelium-proj` schemas, RFC-0016 §4.1/§4.3/§4.5, RFC-0001 §4.6, RFC-0014,
  VR-5/G2/FR-C2/KC-3. Five questions FLAGGED (the §4.2-vs-§4.3 ring placement; the **Phase-6 M-620 native-deploy
  cross-phase dependency**; the packaging schema fields owned by M-368/M-359; the `spore`↔`vsa` reconstruction
  seam; ergonomics-vs-contract) — the deploy half and the regrowth ceiling are M-620's and M-513's to fill,
  never invented here. No code; no kernel change (KC-3). Append-only.
- **2026-06-18 — Implemented (Rust-first, library/manifest half), pending ratification.** The library / reconstruction-manifest half landed as `mycelium-std-spore` (M-522, #163; Batch P5 Tier-A completion, octopus-merged): build/identity/manifest/verify over the `mycelium-spore` packager + `std.content` hash + `std.vsa` decode (KC-3 — no new hash, no new trusted code). Identity is the canonical content hash and metadata-invariant (ADR-003); a hash mismatch is an explicit `Err(HashMismatch)` (C1/G2); probabilistic regrowth is held at the **`Empirical` ceiling** (FR-C2/VR-5), enforced structurally (`Err(ResonatorOverStrength)`). The §4.5 matrix + round-trip / hash-mismatch-refusal / regrowth-ceiling invariants are tested. FLAGGED: the full native deploy / germination is **Phase-6-gated (M-620)**, out of scope (§7-Q2); wrapping `regrow` in `std.numerics::Approx<Value>` is deferred to a fast-follow to avoid a parallel-leaf cross-dep (§7-Q4); the Ring-1-vs-Ring-2 placement (§7-Q1) awaits RFC-0016 ratification. No kernel change (KC-3). Append-only.
- **2026-06-19 — §7-Q4 (Approx coupling) RESOLVED.** `RegrowthResult` carries the manifest's full
  certificate `Bound` and projects to `std.numerics::Approx<Factorization>` via `as_approx()` —
  strength derived from the bound's basis (`Approx::attach`, never upgraded — VR-5), held at the
  `Empirical` ceiling (FR-C2). It carries `Factorization` (the VSA decode result), not `Value` (that
  mapping is `std.vsa`'s). `std.spore` now depends on `mycelium-std-numerics`. Append-only.
- **2026-06-20 — Accepted (library/manifest half; maintainer ratification, DN-07).** The maintainer ratified
  the Rust-first library/manifest half of this spec: the §4.5 guarantee matrix is asserted in tests, the
  never-silent / `Empirical`-ceiling / hash-mismatch-refusal invariants hold, and the open §7 questions are
  scope/cross-phase calls (Q2 native-deploy is Phase-6-gated M-620; Q3 manifest-mode breadth), not contract
  violations. **§7-Q1 (ring placement) is resolved**: the maintainer-authorized **RFC-0016 §4.2 erratum
  (2026-06-20)** reconciled `spore` to **Ring 1, Tier A** (the §4.2 parenthetical was the outlier; §4.3 was
  already correct). The full native deploy / germination half stays out (Phase-6, M-620). Status moves
  *Implemented (Rust-first, library/manifest half) — pending ratification → Accepted (library/manifest half)*.
  Append-only; no kernel change (KC-3).
