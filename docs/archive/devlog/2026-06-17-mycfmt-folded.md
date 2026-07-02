# Devlog — 2026-06-17 · Folding `mycfmt` — the runtime guard *is* the contract

> **What this is** (see `docs/notes/Narrative-Capture-and-Authoring.md`): the *narrative* layer — the
> messy middle the RFCs smooth over. Append-only, informal, honest.

**Theme.** With the M-364 contract ratified, the maintainer said "proceed with folding." So `mycfmt` is
now code (`crates/mycelium-fmt`) — the first folded tool of the M-361 suite. The whole job came down to
one decision that the contract had already anticipated: **how do you guarantee formatting never changes
identity, and what do you do at the edge where you can't?**

---

## 1. The guard is the guarantee

The contract's C1 (identity-preservation) reads like a property you'd verify with a big test. The honest
implementation makes it a **runtime guard**: format, then re-parse the output and compare the surface AST
to the input's; if they differ, **refuse** (exit 4) and emit nothing. That flips C1 from "a property we
hope holds" to "a property that holds by construction per run" — an identity-changing format is
*structurally unemittable*. The conformance corpus test then just confirms the whole `accept/` set stays
in-scope (it does; nothing was refused). This is the never-silent rule (G2) turned inward on the tool
itself: mycfmt would rather refuse than risk a lie about identity.

## 2. The trap the contract warned me about, avoided

§2 of the contract flagged it in advance: `expand_to_source` is *shared* with the "expand ambient"
projection, which is fed the resolved twin and therefore **expands** `default paradigm` / `with paradigm`.
Folding, I fed it the **raw parse** instead — so the ambient is preserved, not expanded. One line, but it's
the difference between a formatter and a different tool wearing a formatter's name. Writing the contract
first is what kept that from becoming a bug.

## 3. The comment problem, resolved the honest way

Comments are lexer trivia — they never reach the AST, so the AST round-trip guard is *blind* to a dropped
comment (parse ignores it either way). That means the C1 guard alone would silently delete comments. The
fix is a separate, explicit scan: preserve the **leading comment block** (the file doc-comment the whole
corpus uses) verbatim, and **refuse** (exit 4) any file with an **interior** comment rather than drop it.
So every one of the 12 `accept/` programs formats cleanly (they only have leading comments), and a
trailing `// id` on a function body is a named refusal, not a quiet deletion. Full comment-preserving
formatting needs the parser to attach trivia — that's a later task, named as deferred, not faked.

## 4. The dependency line held

No `cargo add`. The CLI is hand-rolled arg parsing (the `myc-check` pattern); the only new code that
touches the manifest is teaching `mycelium-proj` to *interpret* `[toolchain]` (it already *accepted* it
since M-359 — mycfmt is just its first consumer). `[toolchain].format` is the ratified **hard pin**: a
mismatch refuses, it doesn't quietly format with the wrong rules.

## 5. What shipped

`crates/mycelium-fmt` (lib + `mycfmt` bin), 7 unit + 3 conformance tests green, `cargo fmt`/`clippy -D
warnings` clean, `just`-parity `all.sh` green. The contract moved Accepted → enacted (append-only). Next
in dependency order: M-368 (spore packaging).

**Refs:** `crates/mycelium-fmt/{src/lib.rs,src/bin/mycfmt.rs,tests/conformance.rs}`;
`crates/mycelium-proj/src/manifest.rs` (the `[toolchain]` reader); `docs/spec/Mycfmt-Formatter-Contract.md`
(Accepted/enacted); M-364 (#136).
