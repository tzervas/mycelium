# Spec (Proposed) — Nodule Header & Project Manifest

| Field | Value |
|---|---|
| **Status** | **Accepted** (2026-06-16 — the §7 format choices are ratified by the maintainer: header sigil `// @key: value`; the v0 key set extended with `repository`/`keywords`/`deprecated`; `@updated` author-maintained. Enacted by M-359.) |
| **Scope** | The structured in-file **nodule header**, the per-project **manifest** (`mycelium-proj.toml`), and the **top-down inheritance** that ties them together |
| **Depends on** | DN-06 (`phylum`/`nodule`/`colony`); RFC-0006 (L2 surface); ADR-003 (content-addressing); the honesty rule / G2 (never-silent), VR-5, KC-3 |
| **Feeds** | M-358 (lexicon migration + linter recognises the header); M-359 (this schema, enacted); M-361 (the full toolchain epic); M-346 (stdlib/packaging) |

## 1. Summary

DN-06 fixed that a `nodule`'s status is declared in a **header comment, not a filename**, and set a
minimal v0 form (`// nodule: <name>`). The maintainer prefers a **structured** header carrying useful
metadata in a clean, compact format — license, authors, first publication, last update, version — at
least on a nodule/phylum **root**, with **subnodules inheriting** most of it top-down. This spec designs
that: a closed-key **header schema**, a **`mycelium-proj.toml` manifest** (the pyproject.toml analogue, scoped
for Mycelium), and an **explicit, inspectable inheritance** model.

Three house-rule commitments shape it:
- **No ambient/silent metadata (G2).** A field's effective value and *where it came from* (this file, a
  parent, or the manifest) is always inspectable (`EXPLAIN`); a conflict is an **explicit error**, never
  silently resolved. Unknown header keys are an explicit lint error, never ignored (the X1 posture).
- **Metadata is not identity (ADR-003).** The canonical nodule identity stays the **content hash of its
  definitions**. The header's `version`/`license`/dates are *associated, queryable* metadata — they do
  **not** perturb the content hash (so a date bump is not a new identity; a code change still is).
- **Small, opt-in, KISS (KC-3/YAGNI).** A subnodule needs only `// nodule: <name>`; everything else
  inherits. The closed key set is deliberately small; richer keys are additive, never silently admitted.

## 2. The project manifest — `mycelium-proj.toml`

The canonical project-level metadata + dependencies + surface, in **TOML** (the pyproject/Cargo
precedent — familiar, diffable, one well-known file per project root, not per-source bloat). It is the
**default source** every nodule header inherits from.

The filename is **`mycelium-proj.toml`** (maintainer, 2026-06-16): easily recognised as the *language
project* manifest, and deliberately **not** `phylum.toml` — a Mycelium project is not only a library
(a `phylum`); it may be a **program, a script, or a small implementation**. The `[project]` table's
`kind` records which, so the manifest fits every project shape, and a tiny single-file **script** may
even carry only a nodule header and skip the manifest entirely.

```toml
# mycelium-proj.toml — the manifest of one Mycelium project (library / program / script).
[project]
name        = "geometry"          # the project name (a phylum's dotted-root, or a program/script name)
kind        = "phylum"            # "phylum" (library) | "program" | "script" — the project shape
version     = "1.2.0"             # semver; a human release label over content hashes
license     = "Apache-2.0"        # SPDX identifier (checked against the SPDX list)
authors     = ["Tyler Zervas <…>"]
since       = "2026-01-10"        # first publication (ISO-8601)
summary     = "2D/3D geometry primitives and certified swaps."
repository  = "https://github.com/example/geometry"   # source URL (inherited by headers)
keywords    = ["geometry", "linear-algebra"]          # discovery tags (inherited by headers)
lang        = "mycelium-0"        # the surface-language edition this project targets (MSRV-analogue)

[surface]                          # for a phylum: the PUBLIC nodules it exports (else omitted)
exports     = ["geometry.shapes", "geometry.transform"]

[dependencies]                     # other phyla — content-addressed (ADR-003): by hash AND a version req
numerics    = { phylum = "numerics", version = "^2", hash = "blake3:…" }

[toolchain]                        # optional pins the toolchain (M-361) reads
format      = "mycfmt-0"           # formatter spelling/version — a HARD PIN: mycfmt refuses a mismatch (M-364 §10.3, ratified 2026-06-17)
lints       = "strict"             # lint profile (advisory in the design phase)

[spore]                            # optional: how a phylum/program publishes as a deployable (ADR-013)
include     = ["surface"]          # what germinates; defaults to the public surface
```

Only `[project]` (with `name`/`kind`) is required; the rest is optional and shape-dependent (`[surface]`
is a phylum concern). The filename `mycelium-proj.toml` is conventional (it names the *project*, not a
source file) — it does **not** violate DN-06's "no `nodule` in paths" rule.

## 3. The nodule header (in-file, structured, compact)

The **first non-blank line** of a Mycelium source file declares the nodule; optional `@`-prefixed
metadata lines follow. Compact, greppable, one closed key set shared with the manifest.

**Required marker** (unchanged from DN-06): `// nodule: <dotted.name>`.

**Optional metadata lines** (`// @<key>: <value>`), closed v0 key set:

| Key | Meaning | Inherits? |
|---|---|---|
| `@version` | semver release label | from `project.version` |
| `@license` | SPDX id | from `project.license` |
| `@authors` | comma-separated | from `project.authors` |
| `@since` | first publication (ISO date) | from `project.since` |
| `@updated` | last update (ISO date) | **per-file** (not inherited — each file tracks its own; author-maintained, §7.4) |
| `@summary` | one-line description | none (per-file) |
| `@repository` | source URL | from `project.repository` |
| `@keywords` | comma-separated discovery tags | from `project.keywords` |
| `@deprecated` | `true`/`false` or a reason string — flags the nodule superseded | **per-file** (not inherited — a nodule flags *itself*) |

A **phylum/nodule root** carries the fuller header; a **subnodule** typically carries only the marker:

```mycelium
// nodule: geometry.shapes        ← a nodule root
// @version: 1.2.0
// @license: Apache-2.0
// @authors: Tyler Zervas
// @since:   2026-01-10
// @updated: 2026-06-16
// @summary: 2D shape primitives and area/perimeter ops.

…definitions…
```

```mycelium
// nodule: geometry.shapes.circle  ← a subnodule: license/authors/version/since INHERITED
// @updated: 2026-06-16            ← only its own last-update (and any override) is local

…definitions…
```

An **unknown `@key`** is an explicit lint error (never silently ignored — X1/never-silent). A malformed
value (e.g. a non-SPDX `@license`, a non-ISO `@updated`) is an explicit error, never a guess (VR-5).

## 4. Top-down inheritance & resolution

The **effective header** of a file is resolved most-specific-first, and is always inspectable:

```
in-file @key  >  nearest ancestor nodule-root header  >  mycelium-proj.toml [project]  >  toolchain default
```

- **Inherited fields** (`version`/`license`/`authors`/`since`/`repository`/`keywords`): omit them in a
  subnodule and they take the phylum value; set them to **override** locally (a local override that
  *narrows* — e.g. a stricter license — is allowed; one that **conflicts** with the phylum in a disallowed
  way, e.g. a license-incompatible value, is an **explicit error**, never silently applied).
- **Per-file fields** (`@updated`, `@summary`, `@deprecated`): not inherited — each file owns them (a
  `@deprecated` flag marks *that* nodule superseded, never its children by inheritance).
- **`EXPLAIN`-able (no black box).** The toolchain can print a file's *resolved* header annotated with
  each field's **source** (`local` / `nodule-root` / `mycelium-proj.toml`), so "where did this license come
  from?" is always answerable — the same no-ambient-metadata discipline G2 applies to errors.
- **Identity unaffected (ADR-003).** Resolution produces *associated metadata*; the nodule's content
  hash is over its definitions only. Two files with identical code but different `@updated` have the
  **same** content identity (dates are not semantic).

## 5. Honesty & house-rules tie-in

- **G2 / never-silent:** unknown keys, malformed values, and conflicting overrides are **explicit**
  errors; nothing about metadata is silently dropped or coerced.
- **VR-5:** an `@license`/`@version` is taken as **declared** metadata (it is the author's assertion);
  the toolchain may *check* SPDX validity and semver well-formedness but never *fabricates* a value.
- **KC-3 / YAGNI:** the header/manifest are a **tooling-layer** concern — zero kernel involvement; the
  closed key set stays small, extended only by an explicit decision.
- **ADR-003:** content-addressing remains canonical identity; this schema is the human/release layer on
  top, never a replacement for the hash.
- **Append-only:** the key set and manifest schema evolve by supersession (a removed/renamed key is a
  decision), mirroring DN-02's naming law.

## 6. Toolchain context (the full-fat suite, long-term)

The maintainer's framing, recorded: a **full Mycelium toolchain** — formatting, correctness/type
checking, error/issue checking **and fixing**, security checks — is a **long-term, prod-release**
requirement (the "full-fat suite"), beyond today's LSP + interpreter + AOT. This header/manifest schema
is the **near-term** slice the maintainer prioritised; it is also the metadata substrate the wider
toolchain (lints that read `[toolchain]`, a packager that reads `[spore]`, a license-compliance check
that reads resolved `@license`) will consume. The epic is tracked as **M-361** (anticipated), the schema
enactment as **M-359**; the header-marker recognition rides with **M-358** (linter/formatter).

## 7. Open choices — flagged for ratification

These are the consequential format choices; the rest of the spec follows from them. Sensible defaults
are chosen above; please confirm or redirect (append-only either way):

1. **Manifest filename — RESOLVED: `mycelium-proj.toml`** (maintainer, 2026-06-16). Easily recognised as
   the *language project* manifest, and **not** `phylum.toml` — a project may be a library (`phylum`), a
   **program**, a **script**, or a small implementation; the `[project].kind` field records which. (The
   earlier `phylum.toml`/`mycelium.toml` candidates are superseded.)
2. **Header sigil — RESOLVED: `// @key: value`** (maintainer, 2026-06-16). Compact, greppable,
   colon-consistent with `// nodule:`, and parsed by a small hand-written line reader (no TOML dep in the
   in-file header). The fenced-block and TOML-in-comment alternatives are superseded.
3. **Closed v0 key set — RESOLVED (extended)** (maintainer, 2026-06-16): the base
   `version`/`license`/`authors`/`since`/`updated`/`summary` **plus `repository`, `keywords`, and
   `deprecated`** (9 keys), over the required `nodule:` marker. The set stays closed — an unknown `@key`
   is an explicit lint error (G2); further keys are additive only by a later explicit decision.
4. **`@updated` discipline — RESOLVED: author-maintained v0** (maintainer, 2026-06-16). The author owns
   `@updated`; the toolchain only *checks* it is a valid ISO-8601 date (VR-5 — checked, never fabricated).
   A `mycfmt` auto-stamp stays an additive M-361 option, deliberately out of v0.

## Meta — changelog

- **2026-06-16 — Accepted; §7 choices ratified; enacted (M-359).** The maintainer ratified the three open
  §7 choices: **(2) header sigil `// @key: value`**; **(3) the v0 key set extended** with `repository`,
  `keywords`, and `deprecated` (9 keys over the `nodule:` marker, closed); **(4) `@updated`
  author-maintained** (checked-not-stamped, VR-5). Status moves **Proposed → Accepted**. Enacted in code by
  **M-359**: the `mycelium-proj` crate (structured-header parser + a minimal, auditable TOML-subset
  manifest reader — adding **no new** external dependency, keeping the workspace's deps few and vetted —
  plus the top-down inheritance resolver with per-field provenance and an `EXPLAIN`), JSON schemas
  (`docs/spec/schemas/nodule-header.schema.json`, `mycelium-proj.schema.json`) + valid/invalid examples,
  and the M-141 linter check (unknown key / bad SPDX / non-ISO date / malformed value = explicit error,
  G2). Metadata is **not** identity — the content hash stays canonical (ADR-003). Append-only.
- **2026-06-16 — Manifest filename RESOLVED + scope broadened.** The manifest is **`mycelium-proj.toml`**
  (maintainer) — recognised as the *language project* manifest, and not `phylum.toml`: a Mycelium project
  is not only a library (`phylum`) but may be a **program / script / small implementation**; the
  `[project]` table (with `kind`) replaces `[phylum]` so the manifest fits every shape, and a tiny script
  may carry only a nodule header. The doc is renamed to *Nodule Header & Project Manifest*. (§7 choice #1
  resolved.) Append-only.
- **2026-06-16 — Proposed.** Drafted at the maintainer's request for a *structured* nodule header (over
  DN-06's minimal `// nodule:` marker): a closed-key in-file header (`// @key: value`), a `mycelium-proj.toml`
  project manifest (the pyproject/Cargo analogue, scoped for Mycelium — library/program/script via
  `[project].kind`), and an explicit, `EXPLAIN`-able top-down inheritance (in-file → nodule-root →
  `mycelium-proj.toml`). Honesty-aligned: no ambient metadata (unknown
  keys/conflicts are explicit errors — G2), metadata is **not** identity (the content hash stays
  canonical — ADR-003), declared-only license/version (VR-5), tooling-layer (KC-3). Records the
  long-term full-fat toolchain as M-361 and this schema's enactment as M-359. Format choices (§7) are
  flagged for maintainer sign-off; **no code lands** until ratified. Append-only.
- **2026-06-17 — `[toolchain].format` posture clarified (M-364).** The §2 example previously labelled the
  whole `[toolchain]` table "advisory in design phase". `mycfmt` (M-364, enacted) is the first consumer and
  reads `format` as a **hard pin** (a version mismatch is an explicit refusal — M-364 §10.3, ratified
  2026-06-17); `lints` stays advisory in the design phase. The example comment is reconciled accordingly so
  the suite's config semantics are consistent across specs. Metadata is still not identity (ADR-003).
  Append-only.
