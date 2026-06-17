# Devlog — 2026-06-17 · Folding `spore` — ADR-003 writes the contract

> **What this is** (see `docs/notes/Narrative-Capture-and-Authoring.md`): the *narrative* layer — the
> messy middle the RFCs smooth over. Append-only, informal, honest.

**Theme.** Second tool of the M-361 suite folded: `spore` (`crates/mycelium-spore`), the packager that
turns a `mycelium-proj.toml` into a content-addressed deployable (ADR-013). It's also the first consumer
of the `[surface]`/`[dependencies]`/`[spore]` tables M-359 has been *accepting but not interpreting* since
it landed — so half the job was teaching the manifest reader to type them.

---

## 1. The one decision that mattered: what is identity?

ADR-003 makes this almost mechanical once you commit to it. The spore identity is a BLAKE3 over the
**code-by-hash DAG** — project kind + the germination surface + every `.myc` source file's content hash +
the dependency hash edges — and **nothing else**. Crucially, the `[project]` metadata (`version`,
`authors`, `summary`) is *excluded* from the hash. The test that pins this is the one I trust most: build a
spore, bump `version` from `1.0.0` to `9.9.9`, rebuild — **same spore id**. Then change one bit of a source
file — **different id**. That's ADR-003 made executable: metadata travels with the artifact but never
defines it.

## 2. Refusing to guess the surface

The never-silent rule (G2) shows up here as "a phylum with nothing to germinate is an error, not an empty
spore." A `phylum` with no `[surface].exports` (and no `[spore].include`) is refused; a project with zero
`.myc` files is refused; a dependency without a `hash` is refused — because an unpinned dependency isn't
reproducible, and a non-reproducible "content-addressed" artifact is a contradiction. Every one of these is
exit 3, no partial artifact written. The CLI smoke test walks each path.

## 3. The honest scope line

A real spore (ADR-013) is code + values + reconstruction manifest + signed metadata. v0 ships the part
that's buildable on today's infra: the content-addressed **descriptor** — sources by hash, deps by hash,
surface — with a *named-provisional* on-disk encoding. The wire-schema, signing, and germination contract
are deferred exactly where ADR-013 §4 already parked them (the RFC-0008 R2 stages). I also hashed source
files by **raw bytes** rather than canonicalizing through `mycfmt` first — tempting (formatting-stable
identity!), but mycfmt refuses some files (interior comments), so coupling spore identity to it would make
packaging fragile. Raw-byte hashing is reproducible today; canonicalized hashing is named as a later
refinement.

## 4. Dependency line held (again)

No `cargo add`. `blake3` is already a vetted workspace dep (the kernel hashes with it); `mycelium-core::
ContentHash` is the canonical address type. The spore crate composes existing pieces — the same discipline
M-359 drew at the TOML reader.

## 5. What shipped

`crates/mycelium-spore` (lib + `spore` bin), 5 lib tests (incl. metadata-invariance + every refusal path)
green; the `mycelium-proj` manifest reader now types `[surface]`/`[dependencies]`/`[spore]` (+3 tests);
`cargo fmt`/`clippy -D warnings` clean. The contract moved Accepted → enacted (append-only). Two of the
five M-361 children are now code (M-364 `mycfmt`, M-368 `spore`). Next in order: M-365 (the check driver).

**Refs:** `crates/mycelium-spore/{src/lib.rs,src/bin/spore.rs}`; `crates/mycelium-proj/src/manifest.rs`
(the packaging tables); `docs/spec/Spore-Build-and-Publish-Contract.md` (Accepted/enacted); M-368 (#140).
