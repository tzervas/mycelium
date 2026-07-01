# Spec (Proposed) — Spore build & publish contract (`mycelium-proj.toml` → `spore`)

| Field | Value |
|---|---|
| **Status** | **Accepted** (2026-06-16 — design; **enacted 2026-06-17** by `crates/mycelium-spore` — the `spore` lib + CLI; §9.1 named-provisional encoding ratified). The contract is now code; identity-vs-metadata (ADR-003) and the never-silent publish inputs (G2) are tested. |
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
  metadata: version 1.2.0, license MIT, 1 author   [not identity — ADR-003]
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

## 10. Remote backend — GHCR/OCI dense-map (ADR-037, M-871)

The M-732 local file store gains a **networked sibling** so spores are installable without crates.io,
hosted in the **GitHub Packages container registry (GHCR)**. Fixed by **ADR-037** (Enacted) and
implemented in `mycelium-spore::remote`.

- **Route by explicit scheme (never guessed, G2).** `spore publish`/`resolve --registry <R>`: a bare
  path `<R>` keeps the local store; `oci://<host>[/path]` or `ghcr://<owner>` selects the remote OCI
  backend. `ghcr://X` → `ghcr.io/X`; `oci://localhost…`/`127.*` auto-selects plain-HTTP; any other
  `<scheme>://` is an explicit `InvalidInput`.
- **Mapping (ADR-037 §2).** A spore is one OCI 1.1 artifact (`artifactType
  application/vnd.mycelium.spore.v1`) at `<base>/<name>:<version>`: each **source object** → one OCI blob
  (`application/vnd.mycelium.spore.object.v1`, title `<blake3-hex>.myco`), **deduped by digest** across
  versions; the **dense-map** (`{spore_id, kind, surface, objects[{rel_path, content_hash}], deps}`) → the
  OCI **config** blob (`application/vnd.mycelium.densemap.v1`, a hand-rolled injective length-prefixed
  encoding with a strict never-silent parser — no new dependency, KC-3); `name@version` → the OCI **tag**.
- **Fetch-and-verify on resolve (DN-28 §3; G2).** Every fetched object's bytes must BLAKE3 to its declared
  `content_hash`, and the reconstructed source set must recompute — via the single canonical
  `content_address` (never re-implemented) — to the recorded `spore_id`. A missing object, an
  extra/undescribed blob, a byte mismatch, or a `spore_id` mismatch is an explicit `Integrity` error.
  `resolve -o <dir>` materializes the verified tree + the `mycelium-densemap`.
- **Transport.** `oras` is the v0 wire driver behind the `OciTransport` trait (a pure-Rust OCI client is
  append-only future work); `oras` absent is an explicit `ToolMissing` error, never a silent skip.
- **Version selection.** Exact version or `latest`/`*` (highest tag by the shared version-sort key); a
  SemVer range stays `Unsupported` (ADR-018 deferred), never mis-resolved.
- **Verified end-to-end (Empirical).** Round-trips green against a local `registry:2` (`just
  spore-oci-selftest`) and the **live GitHub Packages registry** — the example phyla `hello` + `std`
  published to `ghcr.io/tzervas/{hello,std}` and resolved back with byte-identical, hash-verified
  `spore_id`s (`just spore-ghcr-dogfood <owner>`; the release-strategy dogfood, ADR-036).
- **Disclosed v0 gap (never-silent).** Remote publish does **not** yet enforce `name@version`
  immutability the way the local store does (OCI tags are mutable; a best-effort client-side pre-check is
  tracked as **M-872**). Stated here + in ADR-037, not silently dropped.

## Meta — changelog

- **2026-07-01 — Remote backend added (§10; ADR-037 Enacted, M-871).** A GHCR/OCI dense-map remote
  backend (`mycelium-spore::remote`) sibling to the M-732 local store: publish/resolve route by
  `--registry` scheme, spores distribute as OCI artifacts (per-object deduped blobs + dense-map config +
  `name@version` tag), resolve is fetch-and-verify (BLAKE3 per object + `spore_id` recompute). Verified
  against a local `registry:2` and the live GitHub Packages registry (the ADR-036 dogfood). Disclosed v0
  gap: remote immutability not yet enforced (M-872). No new dependency (KC-3). Append-only.
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
- **2026-06-17 — Accepted (enacted by `crates/mycelium-spore`, M-368).** The contract is now code: the
  `spore` lib (`build_spore`/`explain`) + CLI (`spore build`/`explain`) over the M-359 manifest (extended
  to interpret `[surface]`/`[dependencies]`/`[spore]`) — **no new dependency** (the workspace-pinned
  `blake3` + `mycelium-core::ContentHash`; KC-3). **Identity = the content-addressed DAG** (project kind +
  germination surface + source files by raw-byte BLAKE3 + dependency hash edges); **metadata excluded** —
  a `version`/`authors` change leaves the spore id unchanged (tested), a code or dep-hash change moves it
  (tested). Never-silent (G2, tested): a phylum with no surface, a project with no `.myc` sources, a
  hashless dependency, or an `[spore].include` naming a non-export is an explicit publish error (exit 3) —
  **no partial artifact**. `EXPLAIN` prints the identity receipt + the not-identity metadata. v0 scope:
  single project, hash-pinned deps, **raw-byte** source hashing, named-provisional descriptor encoding;
  the R2 wire-schema/signing/germination + canonicalized (mycfmt) hashing are deferred (ADR-013 §4).
  Append-only.
