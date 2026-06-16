# Spec (Proposed) — Nodule Header & Phylum Manifest

| Field | Value |
|---|---|
| **Status** | **Proposed** (2026-06-16 — maintainer requested a *structured* header + a manifest; format choices flagged for sign-off in §7) |
| **Scope** | The structured in-file **nodule header**, the per-phylum **manifest** (`phylum.toml`), and the **top-down inheritance** that ties them together |
| **Depends on** | DN-06 (`phylum`/`nodule`/`colony`); RFC-0006 (L2 surface); ADR-003 (content-addressing); the honesty rule / G2 (never-silent), VR-5, KC-3 |
| **Feeds** | M-358 (lexicon migration + linter recognises the header); M-359 (this schema, enacted); M-361 (the full toolchain epic); M-346 (stdlib/packaging) |

## 1. Summary

DN-06 fixed that a `nodule`'s status is declared in a **header comment, not a filename**, and set a
minimal v0 form (`// nodule: <name>`). The maintainer prefers a **structured** header carrying useful
metadata in a clean, compact format — license, authors, first publication, last update, version — at
least on a nodule/phylum **root**, with **subnodules inheriting** most of it top-down. This spec designs
that: a closed-key **header schema**, a **`phylum.toml` manifest** (the pyproject.toml analogue, scoped
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

## 2. The phylum manifest — `phylum.toml`

The canonical, phylum-level metadata + dependencies + surface, in **TOML** (the pyproject/Cargo
precedent — familiar, diffable, one well-known file per phylum root, not per-source bloat). It is the
**default source** every nodule header inherits from.

```toml
# phylum.toml — the manifest of one phylum (the root of a library-scale unit).
[phylum]
name        = "geometry"          # the phylum name (dotted-root of its nodules)
version     = "1.2.0"             # semver; a human release label over content hashes
license     = "Apache-2.0"        # SPDX identifier (checked against the SPDX list)
authors     = ["Tyler Zervas <…>"]
since       = "2026-01-10"        # first publication (ISO-8601)
summary     = "2D/3D geometry primitives and certified swaps."
lang        = "mycelium-0"        # the surface-language edition this phylum targets (MSRV-analogue)

[surface]                          # the PUBLIC nodules this phylum exports (everything else is internal)
exports     = ["geometry.shapes", "geometry.transform"]

[dependencies]                     # other phyla — content-addressed (ADR-003): by hash AND a version req
numerics    = { phylum = "numerics", version = "^2", hash = "blake3:…" }

[toolchain]                        # optional pins the toolchain (M-361) reads (advisory in design phase)
format      = "mycfmt-0"           # formatter spelling/version
lints       = "strict"             # lint profile

[spore]                            # optional: how this phylum publishes as a deployable (ADR-013)
include     = ["surface"]          # what germinates; defaults to the public surface
```

Sections beyond `[phylum]` are optional; `[phylum]` is required for a phylum root. The manifest's
filename `phylum.toml` is conventional (it names the *unit*, not a source file) — it does **not**
violate DN-06's "no `nodule` in paths" rule.

## 3. The nodule header (in-file, structured, compact)

The **first non-blank line** of a Mycelium source file declares the nodule; optional `@`-prefixed
metadata lines follow. Compact, greppable, one closed key set shared with the manifest.

**Required marker** (unchanged from DN-06): `// nodule: <dotted.name>`.

**Optional metadata lines** (`// @<key>: <value>`), closed v0 key set:

| Key | Meaning | Inherits? |
|---|---|---|
| `@version` | semver release label | from `phylum.version` |
| `@license` | SPDX id | from `phylum.license` |
| `@authors` | comma-separated | from `phylum.authors` |
| `@since` | first publication (ISO date) | from `phylum.since` |
| `@updated` | last update (ISO date) | **per-file** (not inherited — each file tracks its own) |
| `@summary` | one-line description | none (per-file) |

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
in-file @key  >  nearest ancestor nodule-root header  >  phylum.toml [phylum]  >  toolchain default
```

- **Inherited fields** (`version`/`license`/`authors`/`since`): omit them in a subnodule and they take
  the phylum value; set them to **override** locally (a local override that *narrows* — e.g. a stricter
  license — is allowed; one that **conflicts** with the phylum in a disallowed way, e.g. a
  license-incompatible value, is an **explicit error**, never silently applied).
- **Per-file fields** (`@updated`, `@summary`): not inherited — each file owns them.
- **`EXPLAIN`-able (no black box).** The toolchain can print a file's *resolved* header annotated with
  each field's **source** (`local` / `nodule-root` / `phylum.toml`), so "where did this license come
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

1. **Manifest filename** — `phylum.toml` (chosen: names the unit, TOML like Cargo/pyproject). Alt:
   `mycelium.toml` (language-branded). 
2. **Header sigil** — `// @key: value` (chosen: compact, greppable, colon-consistent with `// nodule:`).
   Alt: a fenced block, or TOML-in-comment for a single parser.
3. **Closed v0 key set** — `version`/`license`/`authors`/`since`/`updated`/`summary` (+ the `nodule:`
   marker). Add/remove any? (e.g. `repository`, `keywords`, `deprecated`.)
4. **`@updated` discipline** — per-file, author-maintained vs. tool-stamped on format. (Chosen:
   author-maintained v0; a `mycfmt` auto-stamp is an additive M-361 option.)

## Meta — changelog

- **2026-06-16 — Proposed.** Drafted at the maintainer's request for a *structured* nodule header (over
  DN-06's minimal `// nodule:` marker): a closed-key in-file header (`// @key: value`), a `phylum.toml`
  manifest (the pyproject/Cargo analogue, scoped for Mycelium), and an explicit, `EXPLAIN`-able top-down
  inheritance (in-file → nodule-root → `phylum.toml`). Honesty-aligned: no ambient metadata (unknown
  keys/conflicts are explicit errors — G2), metadata is **not** identity (the content hash stays
  canonical — ADR-003), declared-only license/version (VR-5), tooling-layer (KC-3). Records the
  long-term full-fat toolchain as M-361 and this schema's enactment as M-359. Format choices (§7) are
  flagged for maintainer sign-off; **no code lands** until ratified. Append-only.
