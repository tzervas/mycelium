# Design Note DN-06 — Static Organization & Dynamic Grouping: `phylum` / `nodule` / `colony`

| Field | Value |
|---|---|
| **Note** | DN-06 |
| **Status** | **Resolved** (2026-06-16) — ratified by the maintainer this session (the proposal below + this note's deconfliction); the static keyword migration it requires is tracked as **M-358** (staged, not yet executed) |
| **Amends / supersedes** | **DN-02 §2** (Resolved, append-only) — **supersedes** its `colony = module/namespace` line (supersede-don't-edit, per DN-02's own rule); complements **DN-03** (the lexicon amendment + one-name-per-term rule). DN-02/DN-03 are **not** rewritten — DN-02's changelog records the partial supersession. |
| **Feeds** | RFC-0006 (the L2 static surface: `phylum` / `nodule`); RFC-0008 (the Runtime tier: `colony` as a dynamic grouping of `hypha`, §4.7); the grammar/conformance corpus (`docs/spec/grammar/`); the L1 prototype reserved-word set |
| **Date** | June 16, 2026 |
| **Decides** | the on-brand vocabulary for **static organization** (`phylum` = library-scale, `nodule` = the basic unit, replacing the generic "module") and **dynamic runtime grouping** (`colony` = a runtime grouping of active `hypha`), and the **deconfliction** of `colony`'s reassignment from its DN-02 static meaning |

> Maintainer direction (2026-06-16): *"capture the following convention proposal and ensure we
> utilize and ratify this going forward after deconflicting any collision."* This note captures it,
> resolves the one real collision (`colony`), runs the DN-02 three-test gate, and ratifies. Per
> DN-02's append-only law, reassigning a ratified term is done by **superseding in a new note**, not
> by editing DN-02.

---

## 1. The proposal (as submitted)

Mycelium themes runtime concepts (`hypha`, `cyst`, `spore`, `mesh`, `reclaim`). The static module
system (RFC-0006 L2) used the generic "module" (surfaced as `colony` in DN-02). This introduces
distinctive, on-brand terms for both static organization and dynamic runtime groupings:

1. **`phylum` (static, high level).** A content-addressed **library-scale** unit: a coherent,
   versioned collection of `nodule`s with a defined public surface. The primary unit of static
   organization, distribution, and dependency (≈ a crate/package/library). *Metaphor:* a taxonomic
   rank — a major division of code.
2. **`nodule` (static, mid level).** The **basic unit** of static organization inside a `phylum`:
   contains definitions, types, and implementations. The direct replacement for the generic
   "module" (an L2 surface feature per RFC-0006). *Metaphor:* a small, self-contained growth
   (common in fungal and root systems).
3. **`colony` (dynamic, runtime).** A **runtime grouping of active `hypha`** — a cooperating set of
   concurrent tasks under a shared scope, supervision policy, or deployment context. The
   dynamic/runtime counterpart to static organization; the home for structured-concurrency scopes,
   supervision trees (`reclaim`), and deployment units of running tasks. *Metaphor:* a living group
   of organisms cooperating together.

---

## 2. The collision and its deconfliction (the load-bearing decision)

`phylum` and `nodule` are **new** — a corpus-wide search finds **zero** prior uses, so they
introduce no collision. **`colony` is different and must be deconflicted:**

- **DN-02 §2 ratified `colony` = "Module / namespace of definitions"** ("a bounded, self-sustaining
  network of organisms = a cohesive unit of definitions", *Maintainer-endorsed*, High confidence),
  and DN-02 §1's naming law quotes the maintainer directly: *"Modules being colonies makes sense."*
  It is the L1 top-level compilation-unit keyword today — **226 references across the crates**, plus
  the grammar/EBNF, the LR(1) oracle, and **23 files** in the accept/reject conformance corpus.
- This proposal **reassigns `colony` to the dynamic runtime meaning**, so the *static* role it held
  must move. The natural, proposal-consistent mapping (the proposal defines `nodule` as "the direct
  replacement for the generic term *module*"):

  > **`colony`'s former static role (module / namespace of definitions) → `nodule`.**
  > **`phylum`** is the new library-scale level *above* nodules (no prior static level existed).
  > **`colony`** is freed and **reassigned** to the dynamic runtime grouping of `hypha`.

  This is a **supersession** of DN-02 §2's `colony` line (recorded in DN-02's changelog,
  append-only), **not** a contradiction of the naming law — "modules being colonies" was apt for a
  world with no runtime grouping; now that RFC-0008 has a *genuine* dynamic grouping of `hypha`, the
  metaphor maps **more** accurately with `colony` on the living, running thing and `nodule` on the
  static unit (see §3 T-map).

**Migration (required for coherence; staged as M-358).** Until executed, the L1 surface keyword
`colony` is the **deprecated spelling of `nodule`** (pending migration), and the runtime `colony`
concept is realized by `mycelium-mlir::runtime` (the structured-concurrency scope, §5). The
migration — keyword `colony` → `nodule` across the lexer/parser/AST/checker, the EBNF + LR(1)
oracle, the conformance corpus, and the prototype crate — is mechanical but wide and touches the
grammar contract, so it is its **own** tracked task rather than folded ad hoc here.

---

## 3. The DN-02 three-test gate (each new term)

Per DN-02 §1: theme only where the metaphor is **accurate (T-map)** and **illuminating
(T-illuminate)**, and where it does not cost **dual readability (T-learn)** more than it gives.

| Term | T-map (fidelity) | T-illuminate (teaching) | T-learn (dual readability) | Verdict |
|---|---|---|---|---|
| **`phylum`** | A phylum is a *major taxonomic division* grouping related organisms — maps to a versioned library grouping related nodules. Accurate. | Teaches "a large, coherent division with a public boundary," beyond what "package" conveys about content-addressed cohesion. | A new concept (no library level existed); a themed term loses no familiar baseline (DN-02's rule for unique concepts). | **Theme** |
| **`nodule`** | A nodule is a *small, self-contained growth* — maps to a bounded unit of definitions. Accurate; strictly better than "module" which is generic. | Teaches "a small self-contained structure," reinforcing content-addressed boundaries. | "Module" is familiar, but `nodule` is one letter away and unmistakable — negligible learnability cost, clear thematic gain; the maintainer law prefers themed-where-illuminating. | **Theme** (replaces "module"/static `colony`) |
| **`colony`** (dynamic) | A colony is a *living group of cooperating organisms* — maps to a runtime grouping of active `hypha` far more accurately than to a static file of definitions (which never "lives" or "cooperates"). The reassignment **improves** T-map. | Teaches "a cooperating, supervised, living group of tasks" — exactly RFC-0008 §4.7's structured-concurrency scope + `reclaim` supervision. | Themed runtime concept (like `hypha`); no conventional baseline to lose. | **Theme** (reassigned static → dynamic) |

The reassignment is itself *justified by the gate*: `colony` on the dynamic grouping is a **higher-
fidelity** T-map than `colony` on a static unit, and `nodule` is a higher-fidelity T-map for the
static unit than the reused `colony` was.

---

## 4. Relationship summary

| Term | Layer | Role | Relationship | Status |
|---|---|---|---|---|
| `phylum` | Static | Library / subsystem (content-addressed, versioned) | Contains one or more `nodule`s | **Reserved** (this note); activates with its construct (RFC-0006) |
| `nodule` | Static | Basic static organizational unit (defs/types/impls) | Lives inside a `phylum` | **Ratified**; supersedes static `colony` — migration M-358 **executed 2026-06-16** |
| `colony` | Dynamic | Runtime grouping of active `hypha` | Contains running `hypha`; supervised by `reclaim` (RFC-0008 §4.7) | **Reserved/ratified**; realized by `mycelium-mlir::runtime` (§5) |
| `hypha` | Dynamic | Single concurrent execution unit | Lives in a `colony` | Already ratified (DN-03 §4; RFC-0008 §4.5) |
| `spore` | Deploy | Published artifact of a `phylum` | The deployable form of a phylum (ADR-013) | Already ratified (DN-02 §2) |

Content-addressed identity (ADR-003) holds at **both** the `phylum` and `nodule` levels (each is a
hashable unit with a public surface), consistent with the existing content-addressing of definitions.

---

## 5. How this routes to the build

- **Static surface (RFC-0006 §4.3 / grammar).** `nodule` replaces the static `colony` keyword;
  `phylum` is reserved as the library grouping above it. Both join the reserved-word set the lexer +
  M-141 linter enforce, and the accept/reject conformance corpus. **Executed by M-358** (the keyword
  migration), so the grammar contract changes in one auditable task.
- **Runtime (RFC-0008 §4.7).** `colony` names the dynamic grouping of `hypha`. The v0 realization is
  already in code: the structured-concurrency **scope** in `mycelium-mlir::runtime` (M-357) *is* a
  `colony` — a grouping of cooperative tasks (proto-`hypha`) under a shared cancel scope + per-task
  budgets, the home a `reclaim` supervisor (M-356) attaches to. New runtime code adopts the term
  **going forward** (the type carries a `Colony` alias; the surface keyword waits on M-358).
- **Reserved, not yet active syntax.** Like the DN-03 Runtime names, `phylum`/`colony` remain
  *reserved vocabulary* until their constructs land with typing + elaboration (RFC-0006 §4.3 / the
  RFC-0008 R1+R2 stages). `nodule` activates when M-358 swaps the keyword.

## 6. Resolved dispositions (supplement, 2026-06-16)

- **On-disk naming — RESOLVED: header comment, not filename.** A file's status as a `nodule` is
  declared in a **header comment**, **not** in the file/directory name. File and directory names stay
  **simple and conventional** — forcing `nodule` into paths is bloat/clunk for no gain (developers
  readily learn that `nodule` is Mycelium's word for "module"; the phonetic bridge module→nodule helps).
  This supersedes DN-06's earlier "flexible v0" disposition.
  - **Header format.** The declaration is a comment on the **first non-blank line** of a Mycelium
    source file: **`// nodule: <dotted.name>`** (the name is the logical path within its `phylum`), or
    the bare **`// nodule`** when an explicit name is not needed. The maintainer prefers a **structured**
    header beyond this minimal marker — license, authors, first/last dates, version — on a nodule/phylum
    **root**, with **subnodules inheriting** most fields top-down, plus a `mycelium-proj.toml` **manifest** (the
    pyproject.toml analogue). That schema is designed in **`docs/spec/Nodule-Header-and-Project-Manifest.md`**
    (Proposed, 2026-06-16): a closed-key `// @key: value` header, the manifest, and explicit
    `EXPLAIN`-able inheritance — metadata is **not** identity (the content hash stays canonical, ADR-003),
    unknown keys/conflicts are explicit errors (G2). The bare marker remains valid for a subnodule (the
    rest inherits). The M-141 linter recognises the header; content-addressed identity (ADR-003) stays
    canonical.
- **"Module" → "nodule" in the corpus — RESOLVED.** RFCs and formal docs use **`nodule`** going forward
  in place of the generic "module"; existing occurrences migrate opportunistically (the surface *keyword*
  migration is the mechanical M-358).
- **Terminology Glossary + Index — RESOLVED: a dedicated, separately-maintained document.** Unique
  Mycelium terms get a **dedicated** reference (`docs/Glossary.md`), maintained **separately** rather
  than embedded into every RFC. Per the maintainer's framing: an **Index** is the *summarized* layer
  (one line per term, with pointers) and the **Glossary** is the *detailed* layer (definition,
  relationships, defining doc, usage); **the Index points into the Glossary subsection.** Each entry is
  a *synthesis* citing its normative source (RFC/ADR/DN) — on conflict the source wins. (Created
  2026-06-16.)
- **Singular/plural forms.** Documented: `phylum` / **`phyla`**, `nodule` / `nodules`, `colony` /
  `colonies`, `hypha` / **`hyphae`**. The reserved word is the singular; plurals are prose only
  (never identifiers reserved).

---

## Meta — changelog & maintenance

- **2026-06-22 — `phylum` activated (M-662, E7-1; append-only, supersedes nothing).** The library-scale
  grouping `phylum` — added as **reserved-not-active** by the M-358 entry below — is now an **active L1
  construct**. An optional `phylum <path>` header (the §6 header model: metadata, never content-addressed
  identity — ADR-003) groups one-or-more `nodule` blocks in one source file; a header-less single nodule is
  a phylum-of-one. Cross-nodule visibility is `pub`/`use` (specific + glob), and the RFC-0019 §4.5 orphan
  rule is enforced phylum-wide (pub-blind). Realized in `crates/mycelium-l1` (parse/check/print) + the
  grammar oracle (`mycelium.ebnf` + README; `conformance/accept/19`; the former reserved-not-active
  `reject/10` is repurposed to the phylum-no-nodule parse refusal). The remaining reserved-not-active
  runtime vocabulary (DN-03 §4) is unchanged. **No semantic change to identity** (ADR-003). Append-only.
- **2026-06-16 — M-358 executed (the keyword migration).** The static surface keyword `colony` → `nodule`
  migration this note required is **done**: the lexer/token/parser/AST/checker/elaborator
  (`crates/mycelium-l1`), the LSP surface (`crates/mycelium-lsp`), the grammar oracle
  (`docs/spec/grammar/mycelium.ebnf` + README), and the full accept/reject conformance corpus now spell the
  static unit `nodule`. `phylum` (library grouping) and `colony` (now the RFC-0008 §4.7 dynamic grouping)
  are added as **reserved-not-active** keywords (they lex as keywords — never silent identifiers — but no
  L1 construct consumes them, so neither opens a program; `conformance/reject/10`). The §6 `// nodule:`
  header marker is recognised by `mycelium_l1::parse_nodule_header` and wired into the M-141 linter
  (`lint_nodule_header`) and the M-142 surface formatter (the marker is preserved across a canonical
  re-print). **No semantic change** — content-addressed identity is over elaborated L0, never the surface
  keyword (ADR-003); the conformance corpus and `scripts/checks/all.sh` are green. Append-only; this records
  execution and supersedes nothing.
- **2026-06-16 — Supplement: open questions resolved (§6).** The maintainer's updated proposal resolves
  DN-06's open questions: **(a) on-disk naming** — a nodule is declared by a **header comment**
  (`// nodule: <name>`, or bare `// nodule`) on the first non-blank line, **never** in the filename/path
  (paths stay conventional; supersedes the earlier "flexible v0"); **(b)** RFCs/docs use **`nodule`**
  in place of "module" going forward; **(c)** a **dedicated, separately-maintained Glossary + Index** is
  created (`docs/Glossary.md`) — a summarized Index pointing into a detailed Glossary, each entry citing
  its normative source. Append-only; the §6 disposition replaces only DN-06's own prior "flexible"
  on-disk answer.
- **2026-06-16 — Resolved.** Maintainer directed capturing + ratifying the `phylum`/`nodule`/`colony`
  convention "after deconflicting any collision." Deconfliction: `phylum`/`nodule` are new (no
  collision); **`colony` collides with DN-02 §2's ratified `colony = module`** (226 code refs +
  grammar + 23 conformance files) and is **reassigned** static → dynamic, with the static role moving
  to **`nodule`** and **`phylum`** added above it — a supersession of DN-02 §2 (append-only; DN-02's
  changelog records it), justified by a higher-fidelity T-map (§3). The static keyword migration
  (`colony` → `nodule`, the grammar contract included) is tracked as **M-358** and staged, not yet
  executed; until then `colony` is the deprecated spelling of `nodule`. The runtime `colony` concept
  is realized now by `mycelium-mlir::runtime` (M-357). Reserved-not-active for `phylum`/`colony`.
  Append-only; supersedes DN-02 §2's `colony` line only.
