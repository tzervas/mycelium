# Spec (Proposed) — Spore build & publish contract (`mycelium-proj.toml` → `spore`)

| Field | Value |
|---|---|
| **Status** | **Proposed** (2026-06-16 — the M-368 packaging/publishing contract; design-first, present before folding) |
| **Scope** | The contract for building a **`spore`** (the content-addressed deployable unit) from a `mycelium-proj.toml` project: which manifest tables are consumed, what is identity vs metadata, the dependency-resolution rule, the never-silent publish-input checks, the `EXPLAIN`, and the test plan |
| **Depends on** | ADR-013 (**`spore` is the content-addressed deployable unit** — a hash-identified DAG of code + values + reconstruction manifest + artifact metadata); ADR-003 (content-addressed identity is canonical; metadata ≠ identity); M-359 (`mycelium-proj.toml` + the accepted-but-uninterpreted `[surface]`/`[dependencies]`/`[spore]` tables, `mycelium_proj::{parse_manifest, Manifest}`); RFC-0003 §6 (the reconstruction manifest — one component, unchanged); DN-06 / Glossary (a spore *germinates into a colony*); G2 (never-silent); KC-3 (tooling above the kernel) |
| **Feeds** | M-361 (the full-fat toolchain — `spore` is its packager); the RFC-0008 R2 runtime (the deployable artifact's germination contract — **deferred** there, ADR-013 §4) |
| **Grounds on** | ADR-013 (the four spore components, the narrow=degenerate framing); the manifest reader `crates/mycelium-proj/src/manifest.rs` (the `[spore]`/`[surface]`/`[dependencies]` tables it already accepts); ADR-003 |

## 1. Summary

A **`spore`** is Mycelium's deployable unit: ADR-013 fixes it as a **content-addressed, hash-identified
DAG** of (1) code (content-addressed definitions, shipped by hash), (2) values (initial/captured state
with `Meta` intact), (3) the RFC-0003 §6 reconstruction manifest (one digest-referenced component), and
(4) artifact metadata (provenance, certificates, signatures). This contract specifies the **build step**
that turns a `mycelium-proj.toml` project into such a spore — `mycelium_proj`'s `[spore]`/`[surface]`/
`[dependencies]` tables, accepted-but-not-interpreted since M-359, get their **first consumer** here.

The load-bearing rule is ADR-003: **identity is the content-addressed DAG; everything in `[project]`/
`[spore]` is metadata, not identity.** Two builds of the same code+values+deps produce the **same spore
hash** regardless of `version`/`authors`/`summary`. The contract is presented design-first; no packaging
code lands until it is acknowledged (the M-368 gate).

## 2. What is identity, what is metadata (ADR-003)

| In the spore | Role | Hashed into spore identity? |
|---|---|---|
| code (definitions by content hash) | component (1) | **yes** — identity |
| values (state + `Meta`) | component (2) | **yes** — identity |
| reconstruction manifest (RFC-0003 §6) | component (3) | **yes** (by digest reference) |
| resolved dependency hashes | the DAG edges | **yes** — identity |
| `[project].version` / `authors` / `summary` / `license` / `keywords` | artifact metadata (4) | **no** — associated, queryable |
| `[spore].include` selection | a *build input* that selects which code enters (1) | indirectly: it changes *what code is in the DAG*, so it changes identity by changing the content, not by being hashed itself |

The honest one-liner: **metadata travels with the spore but never defines it** (ADR-003; M-359's "metadata
≠ identity" extended to the artifact).

## 3. The build pipeline

```
mycelium-proj.toml  ─►  (a) parse+validate manifest (M-359)
                        (b) determine the germination surface  ([surface].exports, or [spore].include)
                        (c) gather the content-addressed code reachable from that surface
                        (d) resolve [dependencies] to concrete hashes (§4)
                        (e) assemble the ADR-013 DAG (code + values + manifest + metadata)
                        (f) content-address the DAG  ─►  spore hash (blake3:…)
```

Every step is total and inspectable; any ambiguity is an **explicit error** (§5), never a guessed default.

## 4. Dependency resolution (ADR-003)

The `[dependencies]` table names other phyla **by content hash AND a version requirement** (the M-359
inline-table shape, already accepted: `numerics = { phylum = "numerics", version = "^2", hash = "blake3:…" }`):

- The **hash is authoritative** (ADR-003 — content-addressed). The `version` is a human-facing requirement
  *checked against* the resolved hash's declared version; a `version`/`hash` disagreement is an **explicit
  error**, never silently preferring one (G2).
- A dependency named without a `hash` is an **explicit error** in v0 (no network resolution yet — a
  hashless dep would be an unpinned, non-reproducible input; refuse rather than guess).
- A dependency **cycle** is an explicit error (a spore is a DAG — ADR-013).
- Resolution is reproducible: same manifest → same resolved hash set → same spore hash.

## 5. Never-silent publish inputs (G2)

A `spore` is a release artifact, so a missing or ambiguous input must **fail loudly**, never default:

| Condition | Behaviour |
|---|---|
| no `[project]` / missing `name`/`kind` | explicit error (already enforced by the M-359 reader) |
| a `phylum` with no `[surface].exports` and no `[spore].include` | explicit error — nothing to germinate; the surface must be stated, not guessed |
| `[spore].include` names a nodule not in the project | explicit error (a typo'd surface ships the wrong thing) |
| a `[dependencies]` entry without `hash` | explicit error (§4) |
| `version`/`hash` disagreement | explicit error (§4) |
| an out-of-subset manifest construct | explicit error (the M-359 reader already does this) |

No partial spore is ever written: on any error the build aborts before emitting an artifact.

## 6. `EXPLAIN` / no black box

`spore --explain` prints, deterministically, exactly what went into the artifact and why:

```
spore: geometry  →  blake3:…              ← the spore identity (component DAG hash)
  surface:  geometry.shapes, geometry.transform   [from [surface].exports]
  code:     17 definitions reachable from the surface
  deps:     numerics  blake3:…  (version ^2 satisfied by 2.3.1)   [from [dependencies]]
  values:   0 captured
  metadata: version 1.2.0, license Apache-2.0, 1 author   [not identity — ADR-003]
```

The identity line is the receipt: change the code or a dep hash and it changes; change `version`/`authors`
and it does **not**. EXPLAIN is a total function of the manifest + resolved DAG (no learning).

## 7. CLI surface & scope

Hand-rolled CLI (the `myc-check` pattern — **no new dependency**; the manifest reader is the existing
M-359 subset reader, the hasher is the workspace-pinned `blake3`):

```
spore build   [--config <mycelium-proj.toml>] [-o <out>]   # build + write the spore artifact
spore explain [--config <mycelium-proj.toml>]              # §6, write nothing
```

**v0 scope (honest):** builds a spore from a **single project** with **fully-pinned** (`hash`-carrying)
dependencies; the **artifact wire-schema, signing, and the germination contract** are *deliberately
deferred* to the RFC-0008 R2 runtime stages (ADR-013 §4 names them as new obligations on those stages, not
here). v0 produces the content-addressed DAG + its hash + the `EXPLAIN`; the on-disk serialization format
is the minimal reproducible encoding, named as v0 and superseded when the R2 schema lands. Network/registry
dependency resolution is out of scope (v0 is hash-pinned only). No kernel change (KC-3).

## 8. Test plan (acceptance gate)

1. **Identity / reproducibility** — same project builds to the same spore hash; changing only `version`/
   `authors`/`summary` leaves the hash unchanged (ADR-003); changing a definition or a dep hash changes it.
2. **Surface** — `[surface].exports` (or `[spore].include`) selects the reachable code; a missing surface
   on a `phylum` → explicit error; an `include` naming a non-existent nodule → explicit error.
3. **Dependency resolution** — a hashless dep → error; a `version`/`hash` disagreement → error; a cycle →
   error; a satisfied `version` over a pinned `hash` → ok and recorded in EXPLAIN.
4. **Never-silent** — every §5 row is exercised; no partial artifact is written on any error.
5. **EXPLAIN** — deterministic; the identity receipt is stable and metadata-invariant.

The honesty tag: reproducibility/identity is **Empirical** until the property test is green over the
fixtures, then **Proven** for the v0 single-project, hash-pinned fragment (explicitly *Declared*-deferred
for the R2 schema/signing/germination obligations).

## 9. Open questions (flagged, not decided)

1. **v0 on-disk encoding** — **Ratified (2026-06-17): a named-provisional encoding is acceptable** for v0
   (the minimal reproducible serialization), explicitly marked as provisional and **superseded** when the
   RFC-0008 R2 wire-schema lands (append-only; not silently re-defined).
2. **`[spore].include` vocabulary** — v0 supports `["surface"]` (the public surface) and explicit nodule
   lists; richer selectors (globs, exclusions) are deferred.
3. **Signing** — out of v0 (an artifact-metadata component (4) obligation, deferred with the R2 schema).

## Meta — changelog

- **2026-06-16 — Proposed (M-368 design).** The spore build & publish contract, design-first. Specifies
  building the ADR-013 content-addressed deployable unit from a `mycelium-proj.toml` (the first consumer of
  M-359's accepted-but-uninterpreted `[surface]`/`[dependencies]`/`[spore]` tables): the build pipeline,
  the **identity-vs-metadata** split (ADR-003 — same code+deps ⇒ same spore hash regardless of
  version/authors), hash-authoritative **dependency resolution** (a hashless or disagreeing dep is an
  explicit error), the never-silent publish-input checks (**no partial artifact**, G2), an `EXPLAIN`
  identity receipt, and an honest **v0 scope** (single-project, hash-pinned; the wire-schema/signing/
  germination contract deferred to RFC-0008 R2 per ADR-013 §4). **No new dependency**; above the kernel
  (KC-3). No code lands until acknowledged. Append-only.
- **2026-06-17 — Open question §9.1 ratified.** v0 may ship a **named-provisional on-disk encoding** (the
  minimal reproducible serialization), superseded append-only when the RFC-0008 R2 wire-schema lands.
  §9.2 (`[spore].include` vocabulary) and §9.3 (signing) remain deferred to the first implementation pass.
  Append-only.
