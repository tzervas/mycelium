# Design Note DN-111 — The Canonical Rust→Mycelium Native-Translation Taxonomy (the Native-Equivalence Spectrum)

| Field | Value |
|---|---|
| **Note** | DN-111 (next free note number — DN-110 is the prior highest in `docs/Doc-Index.md` + `docs/notes/`, verified free 2026-07-10 against the tree at `dev` tip `4353681c`; a Draft number is cheap to renumber at merge, so it is picked-and-noted, not blocked on). |
| **Status** | **Draft** (2026-07-10). Authored as **READ + a new DN only** — it enacts nothing, ships no code, and **moves no other doc's status** (house rule #3, append-only). It is the **companion taxonomy DN deferred from DN-110** (DN-110 Ratification point 5 + §12.1: the maintainer accepted that this taxonomy warrants its *own* companion DN — not a DN-109 append — but declined to lock DN-110 §2's four labels as canonical; **this note settles the canonical terminology**). It **refines** DN-110 §2's provisional handles into grounded, citable terms, keeping the handles as a legibility mapping. It does **not** edit DN-110 (Accepted — append-only), `issues.yaml`, `CHANGELOG.md`, `Doc-Index.md`, or any code — those are FLAGGED up (§9). |
| **Decides** | *Proposes, for ratification:* (1) the **canonical taxonomy** for mapping a Rust construct's underlying *problem* to Mycelium's native answer — a four-category **native-equivalence spectrum**: **Native Equivalent · Idiomatic Remapping · Approximation · Interop Bridge** — each grounded in established PL/translation vocabulary + this project's existing corpus vocabulary, with DN-110's handles (Adaptation/Solution/Approximation/Bridge) retained as aliases; (2) that **two handles are renamed** (Adaptation→Native Equivalent; Solution→Idiomatic Remapping) for grounded reasons (§4.2), and **two are kept** (Approximation; Bridge→canonical "Interop Bridge") because they are already the best terms (KISS — no rename for its own sake); (3) the **decision procedure + honesty posture** per category (VR-5); (4) the **formal reconciliation** of this *relationship axis* with DN-109 L4's *decidability axis* as **orthogonal** (§6). |
| **Feeds / builds on** | **DN-110 §2/§2.1/§9/§12.1** (the provisional taxonomy this note refines; its Ratification deferred the canonical terms here); **DN-109** (L4 idiom buckets Mechanical/Heuristic/Judgment — the *decidability axis* this taxonomy is the orthogonal *relationship-axis* complement of; its title "Idiom-Optimal Transpilation and **Structural Remapping**" supplies the reused "remapping" vocabulary); **DN-99** (surface-gap closure register — its Status column `open/partial/already-closed/transpiler-only/idiom` is the corpus vocabulary the four categories re-read, row-by-row cited); **DN-106** (surface-sugar transparency + the gap-closure default + the deliberate-exclusion set); **DN-38** (layered-lowering atlas + `reveal`); **DN-54** (`lower`/`derive`, M-812 landed); **DN-55** (static specialization). The **kernel-unfrozen north star** (ADR-045; different native path per problem, zero hand-ports) is the frame; the **transparency lattice** `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` (house rule #1) governs every category's honesty posture. Feeds **M-1056** (`/native-translate` skill) and the sugar-index `native_strategy` column (§7). |
| **Guarantee** | Every design/vocabulary claim here is **`Declared`** (a naming proposal). Where it cites a *landed* mechanism (`lower`/`derive`, M-812) or a *register-verified* closure (DN-99 rows, `file:line`-cited), that underlying fact is **`Empirical`** at its own source; the *terminology* proposed here is `Declared`. No tag is upgraded past its basis (VR-5). |
| **Date** | July 10, 2026 |

> **Posture (transparency rule / VR-5 / G2 / house rule #4).** A recommendation for a maintainer to
> ratify, not a ratification. Three honest findings up front: (1) this note **renames only two of the
> four handles**, and **keeps two** — the maintainer's steer was *"you likely know the actual terms
> better,"* and the honest answer is that two of the four handles (**Approximation**, **Bridge**) are
> already the best-grounded terms, so renaming all four would be jargon for its own sake (KISS/rule #5);
> the two that are renamed have **specific grounded defects** (§4.2), not stylistic ones. (2) The
> taxonomy is **not invented here** — it is the problem-relationship re-reading of vocabulary the corpus
> already uses (DN-99's register Status column, DN-109's L4 buckets, DN-106's gap-closure default),
> refined against the established PL/translation-theory literature; every category cites its basis.
> (3) The sharpest finding is adversarial (§8.1): **the taxonomy classifies a *(construct, context)*
> pair, not a construct** — the same Rust construct lands in *different* cells depending on a semantic
> fact the syntax does not reveal, so the relationship axis often cannot be fixed until the DN-109
> decidability analysis resolves the context. No sycophancy: this note does **not** simply bless the
> four words the maintainer offered.

---

## §1 Frame — what this note settles, and why it is its own DN

DN-110 answered *"what is Mycelium's own native construct for the role Rust fills with macros"* as **one
instance of a general principle**: translating a Rust construct means **mapping its underlying PROBLEM to
Mycelium's own native answer**, never shoehorning Rust's idiom (DN-109 F1 reframe; DN-106 §3a; the
kernel-unfrozen north star). DN-110 §2 named four **native-translation strategies** — **Adaptation /
Solution / Approximation / Bridge** — but the maintainer ratified those labels only as **provisional,
intuitive handles**, explicitly *not* canonical terminology (DN-110 Ratification point 5). The maintainer's
exact steer: *"those were just me trying to give terms that associate with the concept to me… you likely
know the actual terms better."*

This note gives the taxonomy its **proper, established terminology**. Its remit (DN-110 §12.1, ratified as
"companion DN, not a DN-109 append"): (a) settle the canonical terms, refining or replacing the four
handles; (b) formally reconcile the taxonomy's **relationship axis** with DN-109's **decidability axis**;
(c) cross-link every corpus artifact that already instantiates the informal version — **without re-deciding
any of their own settled forks**. This note is *vocabulary + a classification procedure*, not a mechanism
(the mechanism is DN-110/DN-54; the decision engine is DN-109; the transparency policy is DN-106). Per
DN-110 §12.1's SoC argument, the taxonomy's reach — L1 gap-closure *and* transpiler idiom *and* the
kernel-unfrozen north star — exceeds any single existing DN's scope, so it wants a **parent, not a host**.

### §1.1 Verify-first (mitigation #14 — checked against the tree, 2026-07-10)

- **DN-111 slot is free** (`ls docs/notes/` at `dev` tip `4353681c`; DN-110 is the prior highest note).
- **DN-110 is landed and Accepted on `dev`** (`4353681c`, PR #1404); its §2 taxonomy carve-out and §12.1
  home-recommendation are the direct inputs here. This note treats DN-110 as **read-only** (Accepted,
  append-only — house rule #3): it does not edit DN-110; the append-only pointer resolving DN-110 §2's
  provisional-labels carve-out is **FLAGGED** for the maintainer/integrator to add *once this DN lands*
  (§9; M-1057 DoD).
- **M-1057** ("DN-111 — canonical Rust→Mycelium native-translation taxonomy", `status:needs-design`, epic
  E28-1) is this note's tracking issue; **M-1056** ("/native-translate methodology skill") is the
  downstream consumer. Both verified present in `tools/github/issues.yaml` at `4353681c`. This note edits
  neither — it FLAGs the close-out.
- **The sugar-index `native_strategy` column** referenced by the authoring task ("M-1058/task-13") is **not
  present in `issues.yaml` at `4353681c`** — no `M-1058` id and no `native_strategy` field exist yet
  (there is an in-flight `claude/sugar-index-generator` worktree). This note therefore describes the
  *feed* (§7) but **FLAGs the column/issue as to-verify-or-mint** rather than asserting it exists (VR-5:
  do not cite a `Declared` tracker slot as `Empirical`).

---

## §2 Definition of Done (this note, for maintainer ratification — house rule #6)

"Accepted" requires the maintainer to:

1. **Accept or amend the canonical taxonomy (§4)** — the four category names (**Native Equivalent /
   Idiomatic Remapping / Approximation / Interop Bridge**), the handle-mapping, and the two renames
   (Adaptation→Native Equivalent; Solution→Idiomatic Remapping) with their §4.2 justifications. In
   particular: rule on whether the two renames are warranted or whether a handle is preferred despite its
   grounded defect (a deliberate legibility-over-precision choice is the maintainer's to make).
2. **Accept or amend the per-category decision procedure + honesty posture (§5)** and the derivation of
   *why there are exactly four categories* (§4.1's two-question generator).
3. **Confirm the reconciliation with DN-109 (§6)** — that the relationship axis and the decidability axis
   are orthogonal, and accept the §6 2-D grid as the joint classification surface (it re-decides nothing
   in DN-109; it names the second axis DN-110 §2.1 already asserted).
4. **Rule on the §8.1 finding** — that classification is of a *(construct, context)* pair, and that a
   construct's cell is **time-indexed** under the unfrozen kernel (§8.4). This is the note's sharpest
   claim; the maintainer should confirm the taxonomy records *"as of language version X"* honestly.
5. **Authorize the downstream wiring (§7):** that **M-1056**'s `/native-translate` skill emits the DN-111
   canonical enum (handles retained as aliases), and that the sugar-index gains a `native_strategy` column
   valued from this enum — **and decide whether to mint the sugar-index column issue** (the "M-1058" slot,
   not yet in `issues.yaml`; §9 FLAG).
6. **Add the append-only pointer to DN-110 §2** resolving its provisional-labels carve-out (M-1057 DoD) —
   *not done in this note* (DN-110 is Accepted; this note does not rewrite it — §9 FLAG).

Until then this note is **Draft**; all its terminology proposals remain **`Declared`**.

## §3 User stories

- *As a designer classifying a new Rust construct's native Mycelium translation*, I want **one citable,
  canonical taxonomy** — not four different local re-readings scattered across DN-99/DN-106/DN-109/DN-110 —
  so *"this mapping is a Native Equivalent per DN-111"* resolves unambiguously and identically wherever it
  is written (M-1057 user story).
- *As an engineer translating any Rust construct (not just macros)*, I want a **decision procedure** that
  tells me which category a construct falls in **and what the honest native target + explicit delta is**,
  so I classify by evidence against the corpus, not by shoehorning Rust's idiom (DN-110 §4 user story 4).
- *As a reviewer/auditor*, I want each category's **honesty posture** stated (which transparency-lattice
  tag it may carry, and where it must never claim more), so a classification cannot silently upgrade a
  lossy mapping to "exact" (VR-5).
- *As the maintainer*, I want the terminology **grounded in real vocabulary I can look up** (PL/compiler
  translation literature + this repo's own corpus), with the **intuitive handles kept as aliases** so the
  legible shorthand I already reason with is never lost — and I want to be told plainly **where a handle
  was a false friend or under-descriptive**, not flattered by having all four blessed as-is (house rule
  #4: correct over wrongly-affirmed).
- *As the author of the `/native-translate` skill (M-1056) and the sugar-index*, I want a **stable enum**
  of category names to emit into the DN-99/DN-109 manifest and the sugar-index `native_strategy` column, so
  the tooling records *what kind of native relationship* each mapping/sugar bears, machine-queryable.

---

## §4 The canonical taxonomy — the native-equivalence spectrum

### §4.1 Why exactly four — the two-question generator

The four categories are not an arbitrary list; they are **generated** by two orthogonal yes/no questions
about the mapping from a Rust construct's problem to a Mycelium target. This makes the taxonomy principled
(and makes the §8 blurs precisely locatable — they are the boundaries where a question's answer is
context-dependent):

- **Q1 — Exact?** Does a *fully-native* Mycelium form preserve the construct's observable semantics
  **exactly** (faithful / semantics-preserving, in the CompCert sense)?
- **Q2a — (if exact) Direct or reformed?** Is that native form the **direct first-class construct**
  (structure-preserving — Nida's *formal equivalence*), or a **reformulation** into a structurally
  different native form (Nida's *dynamic equivalence*; V&D *transposition/modulation*)?
- **Q2b — (if not exact) Native-with-residue or off-boundary?** Is there a **native form with an explicit,
  bounded residue** (a lossy-but-native approximation), or does faithfulness require **crossing a foreign /
  interop boundary** (no native form; carry the content across an FFI/`wild`-style edge)?

```
                        ┌─ direct, structure-preserving ── Native Equivalent   (Adaptation)
        ┌─ exact? yes ──┤
        │               └─ reformed, different form ────── Idiomatic Remapping (Solution)
Rust ───┤
construct│               ┌─ native form + explicit residue ─ Approximation      (Approximation)
        └─ exact? no ───┤
                        └─ cross a foreign boundary ─────── Interop Bridge      (Bridge)
```

The result is a **monotone spectrum of decreasing native equivalence** (exact-direct ▸ exact-reformed ▸
lossy-native ▸ non-native-boundary), cut cleanly by two binary properties — **{exact?} × {native?}** — the
organizing axis being **degree of native semantic equivalence**.

### §4.2 The canonical terms, the handles, and the "why this term" (grounded)

| Canonical term | DN-110 handle | Kept / Renamed | One-line "why this term" (literature + corpus basis) |
|---|---|---|---|
| **Native Equivalent** *(short: Equivalent)* | Adaptation | **Renamed** | *Full/formal equivalence* — a first-class native construct fills the role **directly**, structure- and semantics-preserving (Nida "formal equivalence"; compiler "direct mapping / one-to-one lowering"). **Renamed because "adaptation" is a false friend:** in the canonical translation-procedures taxonomy (Vinay and Darbelnet) *adaptation* denotes the **last-resort approximate substitution used when no direct equivalent exists** — i.e. the *opposite* of this bucket — and the English word "adapt" (adjust/modify) miscues toward change, when this category is precisely the *no-change, direct-equivalent* case. Corpus: DN-99 `already-closed`/`cl`; DN-109 **Mechanical**. |
| **Idiomatic Remapping** *(short: Remapping)* | Solution | **Renamed** | *Dynamic equivalence* — the same problem solved by a **different native form/convention** (Nida "dynamic equivalence"; V&D *transposition* = word-class shift, *modulation* = viewpoint shift; "idiomatic translation"). **Renamed because "Solution" is overloaded with the genus:** DN-109 F1 literally names the *whole* enterprise *"map PROBLEM to native SOLUTION,"* so using "Solution" for one *species* is confusing, and the label names nothing distinctive (every category is a "solution"). **Reuses DN-109's own title vocabulary — "structural remapping"** (DRY). Corpus: DN-99 `idiom` (different native form canonical); DN-109 F1 / D-classes; DN-106 §3a. |
| **Approximation** | Approximation | **Kept** | *Partial equivalence* — a close-but-**not-exact** native form with the **delta made explicit and honest** (VR-5). **Kept — already the correct PL term:** "approximation" is the established word for a sound/lossy mapping with bounded residue (abstract interpretation, Cousot and Cousot; "lossy translation"). No rename would improve it (KISS). Corpus: DN-99 `idiom` carrying a recorded caveat; DN-109 **Heuristic** (rule + EXPLAIN flag). |
| **Interop Bridge** *(short: Bridge)* | Bridge | **Kept** *(canonical full form)* | *Non-equivalence handled by crossing a marked boundary* — no full native form yet, so carry the content across an FFI/interop edge (a "language bridge" / FFI / interop shim — standard PL vocabulary; Mycelium's `wild`/FFI boundary, ADR-014). **Kept — already the right term;** only qualified to **"Interop Bridge"** to disambiguate from the design-pattern "Bridge." **Temporary and clearly-marked**, never a permanent shoehorn. Corpus: DN-99 `transpiler-only`/`open` kept never-silently flagged; DN-109 **Judgment** + `suggested_idiom`. |

**Net verdict (honest, non-sycophantic):** **2 renamed, 2 kept.** The two renames each correct a *specific
grounded defect* (a false-friend collision with V&D "adaptation"; a genus/species overload of "solution"),
not a matter of taste. The two kept terms are already the best-grounded words in the space, so renaming them
would violate KISS and manufacture jargon. In every case **the maintainer's handle is retained as an
alias** in this note, the tooling (§7), and the cross-refs — the legible shorthand is never lost.

> **Naming law compliance (DN-02 — plain-first, reuse-before-coin).** No new *keyword* is minted (these are
> classification labels, not language constructs). "Idiomatic Remapping" reuses DN-109's existing
> "remapping" vocabulary; "Approximation"/"Bridge" reuse the maintainer's plus the standard terms; only
> "Native Equivalent" is a fresh label, and it is a plain compound of two established words, chosen over the
> false-friend "Adaptation" precisely to avoid coining-by-collision.

### §4.3 The spectrum is monotone and the categories are ordered

Reading left to right, native equivalence **decreases** and distance from a native answer **increases**:
Native Equivalent (exact, direct) ▸ Idiomatic Remapping (exact, reformed) ▸ Approximation (lossy, native)
▸ Interop Bridge (off-boundary). The crisp cuts: **{Native Equivalent, Idiomatic Remapping}** are *exact*
(faithful); **{Approximation, Interop Bridge}** are *inexact*; and **{Native Equivalent, Idiomatic
Remapping, Approximation}** are *native*, while **{Interop Bridge}** crosses out. Under the unfrozen kernel
a construct **migrates leftward over time** as the language closes gaps (§8.4) — the spectrum is a snapshot,
not a fixed assignment.

---

## §5 Per-category decision procedure, honesty posture, and worked examples (grounded)

Each category below gives: its **decision test** (from §4.1's generator), its **honesty posture** (which
transparency-lattice tag it may carry, and the never-silent obligation), and **worked examples** — a real
Rust construct → its Mycelium mapping that already exists or is planned, cited to the corpus. Citations to
`file:line` are `Empirical` at their source (read against the code in DN-99/DN-110); the *classifications*
are `Declared`.

### §5.1 Native Equivalent (Adaptation)

- **Test:** a direct, first-class, structure-preserving native construct exists and is semantics-preserving
  and auto-emittable (Q1 exact = yes, Q2a = direct).
- **Honesty posture:** may carry the **strongest tag its native construct supports** — up to **`Exact`** /
  **`Proven`** where the underlying construct is (e.g. content-addressed identity, ADR-003). It must still
  **never upgrade past the RHS/basis** (VR-5); a Native-Equivalent mapping that would insert a `swap` or
  need a hygiene flag is *not* auto-fire — it downgrades to a flagged Idiomatic Remapping or Approximation
  (the DN-109 §3.2 ratchet, unrelaxed).
- **Worked examples (corpus-cited):**
  - **J1 macro-role:** Rust `#[derive(Clone,…)]` → `lower Name = <rhs>` + `derive Name for T` (DN-54/M-812,
    landed; DN-99 impl-block #2, `parse.myc:3670`, "native + auto-emitted"). The macro-role exemplar
    (DN-110 J1).
  - Rust `struct`/`enum` → data decl `type`/`Construct`/`Match` (DN-109 D3; DN-99 struct-def #4,
    `emit.rs:1652`, M-1006).
  - Rust `Option`/`Result`/`?` → never-silent `Option`/`Result` with `?`→`match` (DN-109 D2, "Mechanical…
    faithful"; DN-102). *Direct* because Mycelium's never-silent result type **is** the native answer.
  - Rust trait declaration → native trait decl (DN-99 trait-decl #12, `parse.rs:723`, M-1013); generic
    bound (DN-99 #5, RFC-0019 §4.1); the bitwise suite (DN-99 #86).

### §5.2 Idiomatic Remapping (Solution)

- **Test:** a fully-native form preserves the semantics **exactly**, but only by **re-expressing** the
  construct in a structurally different native convention (Q1 = yes, Q2a = reformed) — value-semantics
  functional update, errors-as-values, structured/bounded control, explicit never-silent `swap`.
- **Honesty posture:** **exact** (it is faithful — the residue is zero), so it may carry a strong tag —
  **but only when the reformulation is provably semantics-preserving**. This is the category most exposed
  to the §8.1 context trap: a remapping that is exact *under a precondition* (non-aliasing, boundedness)
  **degrades to Approximation or Interop Bridge when the precondition fails**, and the honest tag drops
  accordingly. The EXPLAIN trail records *which* native convention was chosen and why (DN-109 §3.2).
- **Worked examples (corpus-cited):**
  - **Exceptions/`panic!` → errors-as-values** (never-silent `Option`/`Result` or explicit refusal;
    DN-109 D9). The paradigmatic different-native-path: same problem (signal failure), different native
    convention.
  - Rust `&mut` in-place mutation → value-semantics **functional update** (destructure-and-reconstruct;
    DN-106 Part 2) — **exact remapping only when non-aliasing** (see §8.1).
  - Rust `const`/`static` → nullary `fn name() => T` (DN-99 const/static #14, `totality.myc:273`).
  - Rust `if let` → `match` + `if/then/else` (DN-99 if-let #31, `ebnf:292`) — a structural desugar to a
    different native form.
  - Rust unbounded `while`/`loop` → **bounded `for`** / structural recursion (RFC-0007 §4.8; DN-109 D8) —
    *exact only when the loop is provably bounded*.
  - **J4 macro-role:** Rust `const fn`/comptime → **static specialization** (DN-55) — a *different* native
    path to compile-time computation (DN-110 J4).

### §5.3 Approximation

- **Test:** no *exact* native form exists, but there is a **native form with an explicit, bounded residue**
  (Q1 = no, Q2b = native-with-residue) — a dropped capability or a fall-through caveat.
- **Honesty posture:** **the delta is the deliverable.** The tag is at most **`Empirical`** (or `Declared`
  for an un-measured bound), **never `Exact`/`Proven`**, and the residue is **never-silent** — recorded,
  `EXPLAIN`-able, flagged (VR-5 / house rule #2). Presenting an Approximation as exact is the exact defect
  the transparency rule exists to catch. This is DN-109's **Heuristic** bucket (rule + EXPLAIN flag).
- **Worked examples (corpus-cited):**
  - Rust `#[derive(Debug/Clone)]` on a field that is not structurally derivable → **drop Debug/Clone,
    hand-write structural eq, the sub-gap stays never-silent** (DN-99 derive-attr #3, `emit.rs:1538`).
  - `if let … else` fall-through **idiom recorded with a fall-through caveat** (DN-99 #31).
  - Float transcendentals with an **explicit accuracy bound** (DN-108; DN-99 float-transcendentals #42) —
    approximate, bound stated.
  - *(Borderline — §8.2)* `&T` shared-borrow erasure: exact **under value semantics** but flagged
    (DN-109 D4) — sits on the Native-Equivalent/Approximation seam.

### §5.4 Interop Bridge (Bridge)

- **Test:** faithfulness requires **crossing a foreign/interop boundary** — no native form yet, so carry
  the construct's content across an FFI/`wild`-style edge (Q1 = no, Q2b = off-boundary). **Temporary and
  clearly-marked** — a Bridge is a candidate to migrate leftward as the language grows (§8.4), not a
  permanent home.
- **Honesty posture:** **Judgment / never-silent** — the crossing is always flagged, carries a
  `suggested_idiom`, and a **bare refusal is the last resort** (DN-106 §3a). The tag describes the
  *boundary*, not an equivalence; nothing about a Bridge may claim native semantics (VR-5). This is
  DN-109's **Judgment** bucket.
- **Worked examples (corpus-cited):**
  - **J5 macro-role:** Rust `sql!{…}`/`html!{…}` foreign concrete syntax → **library-with-parser** (a
    string/data value + a `certified`-checked parser), or a flagged exclusion with a `suggested_idiom`
    (DN-110 §8.1). SQL/HTML is foreign, carried as data across the boundary.
  - Rust macro invocation not mappable to `lower`/`derive` → transpiler **"tr-only, hand-expand"**
    pre-pass (DN-99 macro-invocation #11, `transpile.rs:300`; DN-100/M-875).
  - Rust `unsafe` → **`wild`** FFI boundary (ADR-014; DN-109 D6) — *never silently transpiled*.
  - Rust `!` never-type → modeled as a **divergent host-effect** (DN-107; DN-99 never-type #88, `open`).
  - Rust `import`/`use` at the FFI boundary → **stays flagged** (DN-99 import-use #1/#13,
    `Category::Import`).

---

## §6 Reconciliation with DN-109's decidability axis (orthogonal — the joint grid)

**One-sentence reconciliation.** This taxonomy is the **relationship axis** — *how native/equivalent* the
target is to the source problem (Native Equivalent ▸ Idiomatic Remapping ▸ Approximation ▸ Interop Bridge)
— and it is **orthogonal** to DN-109 L4's **decidability axis** — *whether the transpiler can soundly pick
the mapping mechanically* (Mechanical / Heuristic / Judgment); the two **correlate but do not coincide**.

DN-110 §2.1 already asserted this orthogonality; this note **formalizes it as a 2-D grid** (rows =
relationship, columns = decidability). Cells that are *populated* show the correlation; cells that are
*empty or rare* show why the axes are genuinely two, not one:

| relationship ↓ / decidability → | **Mechanical** (auto-fire) | **Heuristic** (rule + flag) | **Judgment** (flag, never guess) |
|---|---|---|---|
| **Native Equivalent** | typical (D1–D4 direct, `derive`) | a *hygiene-sensitive* equivalent needing a flag | rare (a direct native form the tool cannot prove safe) |
| **Idiomatic Remapping** | safe problem→native (DN-106 fork-A functional update; `&T` erase, D4) | reformulation with a recorded caveat | **common** — `&mut` aliasing (D7), boundedness (D8), where `syn` cannot prove the precondition |
| **Approximation** | rare (a lossy form the tool can *always* safely emit) | **typical** — the delta *is* the flag (derive-attr #3) | an approximation whose *residue* needs human sign-off |
| **Interop Bridge** | rare | detect-mechanically, place-by-flag (`unsafe`→`wild`, D6) | **typical** — foreign syntax / FFI (J5, macro-invoke #11, never-type) |

The **binding ratchet is DN-109 §3.2, unchanged**: a mapping auto-fires only if it is semantics-preserving,
inserts no `swap` (S1), upgrades no guarantee tag (VR-5), and is EXPLAIN-recorded. **This taxonomy adds a
problem-side vocabulary on top of that decidability ratchet; it does not relax it.** The two axes meet in
one artifact — DN-109's `idiom_choices` EXPLAIN manifest gains a *relationship* field (the DN-111 category)
alongside its existing *class* field (the DN-109 bucket), so a single row records **both** *what native
relationship* and *how decidable* — one trail, not two (DN-109 §7-c "extend, don't add").

---

## §7 How this feeds the tooling (`/native-translate` M-1056 + the sugar-index `native_strategy` column)

- **`/native-translate` (M-1056).** DN-110 §9's six-step decision procedure is the skill's spec; **this
  note supplies the skill's canonical output vocabulary.** The skill's Step-3 classification and Step-6
  register record emit the DN-111 enum — **`NativeEquivalent | IdiomaticRemapping | Approximation |
  InteropBridge`** — with the DN-110 handle as an alias for legibility. M-1056's body already anticipates
  this ("author it now against the provisional labels… expect a terminology-only follow-up edit when
  DN-111 ratifies the canonical names"); this note is that ratification input. **FLAG (§9):** M-1056
  should be updated to the canonical enum once this DN is Accepted — a terminology-only edit; DN-110 §9's
  six steps are unchanged.
- **The sugar-index `native_strategy` column.** Each sugar/lowering rule in the sugar-index gains a
  `native_strategy` field valued from the DN-111 enum, recording *what kind of native relationship* each
  sugar bears — so the index is machine-queryable by relationship (e.g. "list every Approximation sugar and
  its recorded delta"). **Honest scope note (VR-5):** the column and its tracking issue (referenced by the
  authoring task as "M-1058/task-13") are **not present in `issues.yaml` at `dev` tip `4353681c`** — there
  is an in-flight `claude/sugar-index-generator` worktree but no minted `native_strategy` field or `M-1058`
  id. This note specifies the *contract* (the column's value set = this enum) and **FLAGs the column/issue
  to verify-or-mint** (§9); it does not claim the slot exists.
- **The DN-99 register / DN-109 manifest.** Both gain the relationship field as the shared home for a
  classification (§6) — one EXPLAIN trail. Populating existing DN-99 rows with the canonical category is a
  follow-up, not done here (it would touch a shared, integration-owned artifact).

---

## §8 Adversarial stress-test (VR-5 / house rule #4 — where the categories blur)

### §8.1 The sharpest finding — the taxonomy classifies a *(construct, context)* pair, not a construct

The single deepest classification-blur: **the same Rust construct lands in different cells depending on a
semantic fact the syntax does not reveal.** The canonical case is **`&mut` in-place mutation** (DN-109 D7):

- If the `&mut` is **provably non-aliasing**, value-semantics functional update is **exact** → **Idiomatic
  Remapping** (a faithful different-native-form).
- If it **aliases**, functional update is **observably different** (a write through one alias is not seen
  through the other) → the mapping is **lossy** → **Approximation** (residue = the shared-mutation
  semantics), or, where DN-94 RT1 **excludes** shared mutation outright, an **Interop Bridge**/refusal.

`syn` **cannot** prove non-aliasing (it needs rustc MIR `mir_borrowck`; DN-109 §6-Q3). So **the
relationship-axis cell cannot be fixed until the decidability-axis analysis resolves the context.** The two
axes are orthogonal *in principle* (§6), yet in practice **the decidability analysis often SELECTS the
relationship cell** — you sometimes cannot classify the *relationship* until you have done the
*decidability* work. **Consequence for the taxonomy:** a classification is always of a *(construct,
context)* pair, and where the context is unknown the honest record is **a set of cells with the
precondition that discriminates them** — never a single asserted cell (VR-5). The same pattern recurs for
unbounded-loop→`for` (bounded? → Remapping; else → Approximation/Bridge; D8) and `panic`/`unwrap` (the
`None`/error path is a human decision; D9).

### §8.2 The Native-Equivalent ↔ Idiomatic-Remapping seam (what counts as "the same construct")

The Q2a cut — *direct* vs *reformed* — depends on what counts as "structure-preserving." `?`→`match`
(D2) is classified **Native Equivalent** (Mycelium's never-silent `Result` + `?` **is** the direct native
answer, and DN-109 calls it "faithful"), yet mechanically it *is* a desugar to `match` — which is exactly
how `if let`→`match` (§5.2) is classified **Idiomatic Remapping**. The discriminating test proposed:
**Native Equivalent iff a single *named* native construct is the canonical answer and the surface maps to it
structure-preservingly; Idiomatic Remapping iff the native answer requires re-expressing the construct in a
structurally different form** (even if mechanical). This sharpens the line but does not dissolve it — "is
`?`+`Result` one named construct or a desugar?" is a genuine judgment, and reasonable classifiers may
differ. The seam is **narrow and low-stakes** (both are *exact*, so the honesty posture is identical), which
is why it is tolerable — but it must be recorded, not hidden.

### §8.3 The Approximation ↔ Interop-Bridge seam, and the `wild` fold

The Q2b cut — *native-with-residue* vs *off-boundary* — softens when the "foreign" content is embedded as
**checked data**. A `certified`-checked `sql!` parser library is *native Mycelium code*, so by the "stays
inside the language" test it looks like an **Approximation**; by the "the essential content is a foreign
language carried as data" test it is an **Interop Bridge** (DN-110 calls it a Bridge). Discriminating test:
**Interop Bridge iff the construct's essential content is a foreign language/ABI Mycelium does not natively
parse/execute (carried as opaque-to-the-kernel data); Approximation iff the content IS expressed in native
Mycelium but drops a capability.**

**The construct that resists classification entirely — `unsafe`→`wild`.** `wild` (ADR-014) is a
**first-class native construct** (Native Equivalent by the "named native construct" test) whose *entire
purpose* is to be the **interop/escape boundary** (Interop Bridge by the "crosses out" test). Here the
Native-Equivalent and Interop-Bridge **cells coincide** — the native equivalent *is* the bridge. The
spectrum **folds at its ends**: the language's native answer to "I need to leave the safety model" is a
first-class construct *for leaving the safety model*. This is not a defect to fix but a fact to record — the
honest classification of `wild` is *"Native Equivalent whose role is the Interop Bridge,"* carrying the
Interop-Bridge honesty posture (never-silent, flagged) despite being a first-class construct.

### §8.4 Cells are time-indexed under the unfrozen kernel (a construct migrates leftward)

Because the kernel is **unfrozen** (ADR-045 — the north star is *closing expressibility gaps in the
language*), a construct's cell is **as-of-a-language-version**. Today's **Interop Bridge** (a `tr-only`
hand-expand, DN-99) becomes tomorrow's **Native Equivalent** once the grammar/runtime grows the feature
(the DN-99 register's whole purpose is to drive that leftward migration). **Honest consequence (VR-5):**
every classification carries an implicit *"as of language version X,"* and a taxonomy record that omits the
version is under-specified. The `/native-translate` skill (§7) must therefore stamp its output with the
corpus/tree state it classified against (mitigation #14's verify-first step supplies exactly this). A cell
is a **snapshot on a moving target**, not a permanent property of the construct.

### §8.5 Residual honest limits

- The taxonomy is a **relationship vocabulary, not a decision engine** — it does not itself decide a
  mapping (that is DN-109's ratchet) nor build a mechanism (DN-110/DN-54). Mis-reading it as a decision
  procedure would over-claim; it *feeds* the procedure (§7).
- Two of four canonical terms are **`Declared` naming proposals** subject to the maintainer's ratification
  (§2 DoD-1). If the maintainer prefers a handle despite its grounded defect (legibility over the V&D
  false-friend, say), that is a legitimate call this note flags rather than forecloses.

---

## §9 FLAGs (append-only rows the integrating parent applies — this note edits none of these)

`docs/Doc-Index.md`, `CHANGELOG.md`, `tools/github/issues.yaml`, and the Accepted **DN-110** are
integration-/append-only-owned (this note FLAGs; the integrating parent/maintainer applies once). **FLAG to
the integrator (main):**

- **`docs/Doc-Index.md`** — add a Design-Notes row for **`DN-111 — The Canonical Rust→Mycelium
  Native-Translation Taxonomy (the Native-Equivalence Spectrum)`**
  (`docs/notes/DN-111-Canonical-Rust-To-Mycelium-Native-Translation-Taxonomy.md`), Status **Draft
  (2026-07-10)**. Row prose can draw from the header table + §4.
- **`CHANGELOG.md`** — an "Added (design, pending ratification)" entry for DN-111 (the canonical
  native-translation taxonomy companion to DN-110 §2: the four-category native-equivalence spectrum
  — Native Equivalent / Idiomatic Remapping / Approximation / Interop Bridge — with DN-110's handles as
  aliases; two renamed, two kept; reconciled with DN-109's decidability axis as orthogonal).
- **`tools/github/issues.yaml` — M-1057 close-out (do NOT edit here):** on ratification, flip M-1057
  (`needs-design` → `done`) with the landed-basis being this DN; its DoD also calls for an **append-only
  pointer added to DN-110 §2** resolving the provisional-labels carve-out — **that edit to DN-110 is the
  maintainer's/integrator's** (DN-110 is Accepted; this note does not rewrite it).
- **`tools/github/issues.yaml` — M-1056 terminology update (FLAG):** once this DN is Accepted, update
  M-1056's `/native-translate` skill spec to emit the DN-111 canonical enum (handles retained as aliases) —
  a terminology-only edit; DN-110 §9's six steps are unchanged. Add `corpus:DN-111` to M-1056's `doc_refs`.
- **`tools/github/issues.yaml` — sugar-index `native_strategy` column (VERIFY-OR-MINT, FLAG):** the
  authoring task referenced "M-1058/task-13" for a sugar-index `native_strategy` column, but **no `M-1058`
  id and no `native_strategy` field exist in `issues.yaml` at `dev` tip `4353681c`** (there is an in-flight
  `claude/sugar-index-generator` worktree). **The maintainer/integrator should verify the sugar-index
  work-item and, if the column is wanted, mint it** with the contract from §7 (value set = the DN-111
  enum) and `doc_refs: [corpus:DN-111, corpus:DN-110]`. This note does not mint or assert the slot (VR-5).
- **`CLAUDE.md`** — no change proposed.

---

## §10 Changelog

- **2026-07-10** — DN-111 created (**Draft**). The **companion taxonomy DN deferred from DN-110** (DN-110
  Ratification point 5 + §12.1: "companion DN, not a DN-109 append"). Settles the **canonical Rust→Mycelium
  native-translation taxonomy** — a four-category **native-equivalence spectrum** generated by two
  questions ({exact?} × {native?}): **Native Equivalent** (Adaptation — *renamed*, "adaptation" is a V&D
  false friend meaning the opposite), **Idiomatic Remapping** (Solution — *renamed*, "solution" is
  overloaded with the genus; reuses DN-109's "remapping"), **Approximation** (*kept* — already the correct
  PL term), **Interop Bridge** (Bridge — *kept*, canonical full form). **Net: 2 renamed, 2 kept** — renames
  correct specific grounded defects, not taste (KISS/rule #5); DN-110's handles retained as aliases
  throughout. Gave each category its decision test, honesty posture (VR-5 — Native Equivalent/Idiomatic
  Remapping may be exact; Approximation is lossy + never-silent; Interop Bridge is a flagged boundary), and
  corpus-cited worked examples (J1 `derive`, exceptions→errors-as-values, derive-attr drop, `sql!`/`wild`).
  Formalized the reconciliation with DN-109 as an **orthogonal 2-D grid** (relationship × decidability),
  re-deciding nothing in DN-109. Adversarially stress-tested: **classification is of a *(construct,
  context)* pair** — the same `&mut` construct is Idiomatic Remapping (non-aliasing) or
  Approximation/Bridge (aliasing), so the decidability analysis often *selects* the relationship cell
  (sharpest finding); the Native-Equivalent/Remapping seam (`?`→`match`); the Approximation/Bridge seam and
  the **`wild` fold** (the native equivalent *is* the bridge); and **time-indexed cells** (a Bridge
  migrates to a Native Equivalent as the unfrozen kernel closes the gap — every classification is
  "as-of-version-X"). Feeds `/native-translate` (M-1056, canonical enum) and the sugar-index
  `native_strategy` column (verify-or-mint FLAG). `Declared` throughout (a vocabulary proposal);
  `Empirical` only where citing landed `lower`/`derive` or register-verified DN-99 closures at their own
  source. Authored READ + DN only — no edit to DN-110 (Accepted, append-only), `issues.yaml`, `CHANGELOG`,
  `Doc-Index`, or code (FLAGGED §9). Append-only; status advances only by maintainer ratification (house
  rules #3/#4).
</content>
