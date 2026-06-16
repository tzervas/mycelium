# Devlog — 2026-06-16 · `mycfmt`: contracting the formatter before writing it

> **What this is** (see `docs/notes/Narrative-Capture-and-Authoring.md`): the *narrative* layer — the
> messy middle the RFCs smooth over. Append-only, informal, honest. The RFCs/ADRs/DNs remain the source
> of truth; this is the *story* of how a decision actually got made. Refs point at what shipped.

**Theme.** Phase-8 opens the M-361 "full-fat toolchain" epic. Its five children (M-364…M-368) got created
and wired under #132 this session; the first one, M-364 (`mycfmt`), is the formatter that M-142's
in-process primitives grow up into. The discipline — same as M-363 last wave — is **design-first: present
the contract before folding.**

---

## 1. Why a formatter needs a *contract*, not just a `--write`

A formatter feels like the most boring tool in the box: re-indent, re-space, done. It is not, because in
Mycelium a definition's **identity is its content hash** (ADR-003; RFC-0001 §4.6). A formatter that
*silently* changed identity would be the purest possible violation of the house rules — a black box that
rewrites meaning while pretending to rewrite whitespace. So the load-bearing question isn't "what does
pretty output look like?" but "**how do we prove formatting never changes identity, and what do we do at
the edge where we can't?**" That question deserves a frozen answer before any code, which is what the
contract (`docs/spec/Mycfmt-Formatter-Contract.md`) is.

## 2. The subtle trap I wanted on the record

M-142 already left two printers in the tree. One is the Core-IR α-normalizing dump (`fmt.rs`). The other
is `expand_to_source` — but that one is wired into the **"expand ambient"** projection (`expand.rs`),
which is fed the *ambient-resolved twin* and therefore **expands** `default paradigm` / `with paradigm`
into longhand. If `mycfmt` naively reused that path it would "format" by *expanding the ambient* — changing
the surface form the author wrote. The fix is one word in the contract: feed the printer the **raw
parse**, not `resolve(parse(...))`. Formatting ≠ expanding. I wrote it down as a §2 subtlety precisely so
it can't sneak back in as a bug.

## 3. The honest edge: refuse, don't guess

The existing surface printer doesn't parenthesize nested expressions and doesn't carry comments (they're
lexer trivia). Either could make `parse ∘ print ∘ parse` *not* the identity for some program — i.e. an
identity change. The tempting move is "best-effort reformat and hope." The honest move (G2 over
convenience) is the contract's §7 boundary: `mycfmt` v0 formats exactly the fragment the conformance
corpus proves round-trips, and **refuses** (a named exit-4 diagnostic) anything else, rather than risk a
lossy rewrite. A formatter you can't trust with identity is worse than no formatter. The `EXPLAIN` mode
makes the guarantee visible: every run prints an **identity receipt** (`blake3:… → blake3:…`, equal), and
a run that can't show equality doesn't write.

## 4. What did *not* happen

No `mycelium-fmt` crate, no `cargo add` of a CLI/TOML parser (that's an ADR, not a build detail — the same
line M-359 drew), no code at all. The deliverable is the contract + the bookkeeping (children created and
wired under #132, idmap/CHANGELOG/Doc-Index updated). Folding follows once the contract is acknowledged.

**Refs:** `docs/spec/Mycfmt-Formatter-Contract.md` (Proposed); M-364 (#136), epic M-361 (#132);
primitives in `crates/mycelium-lsp/src/{fmt,expand}.rs`, `crates/mycelium-l1/src/ambient.rs`;
`[toolchain].format` in `docs/spec/Nodule-Header-and-Project-Manifest.md` §2.
