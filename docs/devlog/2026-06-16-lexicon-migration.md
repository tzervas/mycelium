# Devlog — 2026-06-16 · The keyword that had to move: `colony` → `nodule`

> **What this is** (see `docs/notes/Narrative-Capture-and-Authoring.md`): the *narrative* layer — the
> messy middle the RFCs smooth over. Append-only, informal, honest. The RFCs/ADRs/DNs remain the source
> of truth; this is the *story* of how a decision actually got made. Refs point at what shipped.

**Theme.** DN-06 was already ratified (Resolved): the static unit is `nodule`, `phylum` is the
library grouping above it, and `colony` — the word DN-02 had spent on the static module — gets
*reassigned* to the genuinely dynamic thing RFC-0008 §4.7 finally gave us: a runtime grouping of
`hypha`. M-358 is the part where you actually move the keyword. The decision wasn't the hard part;
the *honesty bookkeeping* was.

---

## 1. The rename map (presented before folding)

A pure rename touches more than `s/colony/nodule/g`, because one `Colony` must **not** move:
`mycelium-mlir::runtime::Colony` is the *new dynamic* meaning, born this week. So the rename is scoped
to the **L1 static surface** and its mirrors:

| Surface | Before | After |
|---|---|---|
| Header keyword (source text) | `colony <path>` | `nodule <path>` |
| Lexer token | `Tok::Colony` (module) | `Tok::Nodule` |
| Keyword table | `"colony" => Tok::Colony` | `"nodule" => Tok::Nodule` |
| AST root | `struct Colony` | `struct Nodule` |
| Parser | `parse_colony`, `expect(Tok::Colony, …)` | `parse_nodule`, `expect(Tok::Nodule, …)` |
| Public API | `check_colony` | `check_nodule` |
| Surface printer | `expand_to_source` emits `colony …` | emits `nodule …` |
| Grammar oracle | `program ::= colony_header …` | `program ::= nodule_header …` |
| Conformance corpus | `01-minimal-colony.myc`, `01-no-colony-header.myc`, all `colony` headers | renamed + `nodule` headers |

**Untouched (deliberately):** `mycelium-mlir` (its `Colony` is the dynamic grouping). Scope was ~289
references across `mycelium-l1` + `mycelium-lsp` + the grammar dir.

**Two new reserved words, not active.** `phylum` and `colony` join the keyword table so they can never
be silent identifiers, but no production consumes them — so opening a program with either is an explicit
*“expected a `nodule` header”* (new `conformance/reject/10`). This is the same reserved-not-active
posture DN-06 §4 names for both.

## 2. The grammar diff

```
-program        ::= colony_header item*
-colony_header  ::= 'colony' path
+program        ::= nodule_header item*
+nodule_header  ::= 'nodule' path
```

…plus a prose note in the EBNF header that `phylum`/`colony` are reserved-not-active, and the vocabulary
line now cites DN-06.

## 3. Why this is a rename and not a semantic edit (the ADR-003 worry)

The instruction was load-bearing: *content-addressed identities MUST be unchanged.* The natural fear is
that renaming the program-opening keyword changes a program's hash. It does not, and the reason is
structural: the content hash is computed over the **elaborated L0** (`operation_hash` over the canonical
term / data registry / policy names), never over the surface token stream or the Rust type name. A
program that said `colony foo` and now says `nodule foo` elaborates to the *same* L0 term, so the same
hash. The formatter tests (`fmt.rs`) already encode this for binders (“names are not hashed”); the same
principle covers the header keyword. So the migration is honestly a rename — the test that would have
caught a semantic drift (the three-way differential in `tests/differential.rs`) stays green untouched.

## 4. The marker that lives outside the grammar

DN-06 §6 wants files to *declare* their nodule with `// nodule: <dotted.name>`. The instinct is to put
that in the parser — but comments are **lexer trivia**; they never reach the AST. That turns out to be
the right design, not a limitation: the marker is metadata, and **metadata is not identity** (ADR-003),
so it has no business in the AST or the hash. So `parse_nodule_header` is a small *source-text*
recogniser, total by construction: it returns the marker, `None` for an ordinary first comment, or an
**explicit** error when the author clearly meant a *named* marker (`// nodule:`) but wrote a bad name —
a near-miss is flagged, never silently dropped (G2). The linter (M-141) surfaces a malformed one; the
surface formatter (M-142) re-emits a valid one canonically. The structured `// @key:` header and the
`mycelium-proj.toml` manifest layer on top of this floor — that's M-359, and it needs the maintainer to
ratify three §7 choices first, so it is deliberately *not* in this change.

## 5. The thing that went red (and was honest about it)

Two checks failed on the first green-run, and both were the migration telling the truth:

1. **The Python KC-2 experiment harness** (`experiments/`) carries reference Mycelium programs as
   strings — `colony bench\n…`. The real parser now rejects them: *“expected a `nodule` header …, found
   Colony.”* That error message is itself the proof the reservation works (`colony` lexes as the reserved
   `Tok::Colony`, not an identifier). Fixed the reference strings.
2. **codespell** flagged a misspelling of “unparsable” in `mycelium-mlir/src/budget.rs` — a *pre-existing*
   typo that only surfaced because the check env now has codespell installed (it skips gracefully when
   absent, so the maintainer's runs never caught it). Fixed in its own small commit; not part of the rename.

Refs: DN-06 (Resolved); issue M-358 (#130); `crates/mycelium-l1/src/{token,parse,ast,nodule}.rs`;
`crates/mycelium-lsp/src/{lint,expand}.rs`; `docs/spec/grammar/`. No kernel change (KC-3).
