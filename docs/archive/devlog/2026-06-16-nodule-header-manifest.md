# Devlog — 2026-06-16 · A header that isn't identity, and a TOML parser we didn't write

> **What this is** (see `docs/notes/Narrative-Capture-and-Authoring.md`): the *narrative* layer — the
> messy middle the RFCs smooth over. Append-only, informal, honest. The RFCs/ADRs/DNs remain the source
> of truth; this is the *story* of how a decision actually got made. Refs point at what shipped.

**Theme.** M-358 left a one-line floor: `// nodule: <name>`. M-359 builds the maintainer's preferred
*structured* header on top — license, authors, dates, version — plus a `mycelium-proj.toml` manifest and
the inheritance that ties them together. Three decisions shaped it; two were the maintainer's to make and
one was ours to refuse.

---

## 1. The choices that were the maintainer's (§7)

The spec was deliberately `Proposed`, not `Accepted`, with three flagged choices — and the house rule is
to *not* ratify defaults unilaterally. So we asked, and got: **(2)** sigil `// @key: value`; **(3)** the
v0 key set *extended* — base six plus `repository`, `keywords`, `deprecated`; **(4)** `@updated`
author-maintained (the toolchain *checks* the ISO date, never stamps it). Only then did the spec move
`Proposed → Accepted` and code land. The asking wasn't ceremony: "add all three optional keys" was a real
redirect from the proposed "keep it minimal" default.

## 2. The dependency we didn't add

The manifest is TOML. The obvious move is to add the `toml` crate. The less-obvious move — the right one —
is to notice that *adding a dependency is a decision*, not a build detail. The workspace keeps its deps
few and vetted (a handful: `serde`, `serde_json`, `blake3`), and a new one is an ADR, not a casual `cargo
add`. (An early draft of this very devlog claimed the workspace was "dependency-free" — it isn't; that
line got corrected before it shipped, which is the honesty rule biting its own author.)

So we wrote a **minimal TOML *subset*** reader: `#` comments, `[table]` headers, and single-line
`key = value` where a value is a basic string, an array, an inline table, or a boolean. The discipline
that makes it honest rather than a foot-gun: it is **named as a subset**, and anything outside the subset
— a bare number, a multi-line array — is an **explicit error**, never a silent misparse (G2). A parser
that quietly drops what it doesn't understand is exactly the black box the house rules forbid.

## 3. The load-bearing line: metadata is not identity

The whole feature is, in a sense, a temptation to violate ADR-003. A header carries a `@version` and an
`@updated` date; a naive design lets those leak into the content hash, and suddenly a date bump is a new
identity and the content-addressing story collapses. The spec fixes it (`§4`/`§5`) and the code honours it
structurally: `mycelium-proj` produces *associated metadata* — a `ResolvedHeader` — and never touches a
`Node`'s `content_hash`. Two files with identical code and different `@updated` dates are the **same**
nodule. The schema says the same thing in its own register: the JSON projection is a sidecar, not the
definition.

## 4. EXPLAIN, because inheritance is where metadata goes ambient

Inheritance is the dangerous part: once a subnodule's `@license` can come from a parent manifest, "where
did this license come from?" stops being obvious — and an unanswerable provenance question *is* ambient
metadata (the G2 failure mode). So every resolved field carries an `Origin` (`local` /
`mycelium-proj.toml`), and `explain()` prints it:

```
nodule: geometry.shapes.circle
  license: Apache-2.0  [mycelium-proj.toml]
  updated: 2026-06-16  [local]
```

The one thing we deliberately *didn't* build: cross-tier "disallowed conflict" detection (a
license-incompatible override). That needs a compatibility matrix, it belongs to the compliance check in
the full-fat toolchain (M-361), and inventing a half-version now would be a worse lie than naming it
deferred. Local-overrides-manifest is an *allowed* override (the spec says so), so v0 records it and moves
on.

Refs: spec `docs/spec/Nodule-Header-and-Project-Manifest.md` (Accepted); issue M-359 (#131);
`crates/mycelium-proj/`; `crates/mycelium-lsp/src/lint.rs`;
`docs/spec/schemas/{nodule-header,mycelium-proj}.schema.json`. No kernel change (KC-3).
