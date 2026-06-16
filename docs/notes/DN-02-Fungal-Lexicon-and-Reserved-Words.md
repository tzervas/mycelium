# Design Note DN-02 — Fungal Lexicon & Reserved-Word Set

| Field | Value |
|---|---|
| **Note** | DN-02 |
| **Status** | **Resolved** (2026-06-10) — ratified by the maintainer with adjustments (§7); the ratified set below is frozen into the grammar artifacts (RFC-0006 §4.3) |
| **Feeds** | RFC-0006 (surface language; the L3 lexicon + the L1/L2 reserved words); the grammar/conformance corpus (`docs/spec/grammar/`); the L1 prototype crate |
| **Date** | June 10, 2026 |
| **Decides** | the surface vocabulary of Mycelium-the-language: which concepts get a fungal-themed term, which keep a conventional one, the full reserved-word set, operators, literal forms, and the `unsafe`-class boundary |
| **Maintainer direction (this session)** | language name = **Mycelium** (shared with the project); `unsafe` boundary = **reserved now** as a themed, denied-by-default block; sequencing = **this note first, then build**; the **naming law** below; and the §7 ratification: keep `trait` (not `guild`), `spore` yes, **`matured`** (not `hardened`/`sclerotium`), keep `type`, `wild`, `@`-annotation approved, import = `use` |

> This note narrows and frames; ratification is the maintainer's. Where a term is a genuine
> judgment call it is **flagged ⚑** for an explicit decision rather than silently chosen. Nothing
> here is frozen until the grammar artifacts (RFC-0006 §4.3) are cut against a ratified lexicon.

---

## 1. The naming law (maintainer direction, verbatim intent)

> *"Ensure the meanings of the terms map directly to the behavior and meaning of the functionality
> being described by the term. Modules being colonies makes sense. The only terms that are generic
> and reused from other languages should be ones that won't lead to difficulty learning the
> language and will enable more robust, resilient development and readability by both human and
> machine users."*

Operationalized as a **three-test gate** every candidate term must pass to be *themed*; failing any
test means **keep the conventional term**:

- **T-map (fidelity).** Does the fungal metaphor map *accurately* to the behavior — not just
  decoratively? A metaphor that implies behavior the construct does **not** have is *disqualified*
  (it misleads, violating the law). *Worked rejection:* `spawn` for a pure function fails — in
  common usage "spawn" implies starting a concurrent process, which a pure function is not; the
  false implication is worse than the lost theme, so functions stay `fn`.
- **T-illuminate (teaching value).** Does the themed term *teach* something about the behavior that
  the conventional term does not? `colony` for a module teaches "a bounded, self-sustaining network
  of definitions"; `species` for a type teaches little beyond what `type` already conveys.
- **T-learn (dual readability).** Does keeping a conventional term aid learnability/readability for
  **both** human and machine readers (S6: machine fluency = LLM leverage = familiarity; T3.6) *more*
  than theming would? Control-flow and binding keywords (`let`, `if`, `match`) score high here, so
  they stay conventional.

**Net rule:** *theme where the metaphor is accurate **and** illuminating; keep conventional where a
borrowed term is clearer to learn and read.* This is the principled form of RFC-0006's hybrid
posture — theme the Mycelium-unique concepts (which have no familiar baseline to lose), keep the
universal scaffolding conventional.

---

## 2. Proposed lexicon — themed terms (pass all three tests)

| Concept (corpus reference) | Proposed term | Why it maps (T-map / T-illuminate) | Confidence |
|---|---|---|---|
| **Module / namespace of definitions** | `colony` | A colony is a bounded, self-sustaining network of organisms = a cohesive unit of definitions. Maintainer-endorsed. | High |
| **Cross-colony dependency graph** (the content-addressed import network; ADR-003) | `network` / *mycorrhizal* link | The mycorrhizal network is exactly a content-addressed web connecting colonies; the network *model* is themed, the import *keyword* stays `use` (§3, ratified). | High |
| **Affine external resource** (the LR-8 `Resource` kind; consumed exactly once) | `substrate` | A substrate is the material a fungus consumes to grow — used up, **exactly once**. Affinity *is* single-consumption; the metaphor teaches the linearity. | Ratified |
| **Reconstruction manifest** (RFC-0003 §6; the self-contained recipe to regrow a value) | `spore` | A spore is the self-contained dispersal unit that **regrows the whole organism** — exactly a reconstruction manifest. Schema name stays `reconstruction-manifest`; `spore` is the surface term. | Ratified |
| **Promoted stable / AOT component** (RFC-0004 §4; matured, persistent, verified) | `matured` | A *matured* definition has grown from interpreted to compiled-and-frozen — the metaphor teaches "grown to a hardened, persistent stage". (Chosen over the more vivid-but-obscure `sclerotium`.) | Ratified |
| **`unsafe`-class block** (LR-9; denied by default, lexically marked) | `wild` | "Wild" growth is uncultivated, outside the tended colony — code outside the safe, cultivated guarantees. Teaches "you have left the safe culture". | Ratified |

---

## 3. Proposed lexicon — kept conventional (fail T-illuminate or win on T-learn)

| Concept | Term kept | Why not themed |
|---|---|---|
| Local binding | `let` | Universal; theming adds no behavioral insight; high dual-readability (T-learn). |
| Function definition | `fn` | `spawn`/`grow` imply concurrency or mutation the construct lacks (T-map fail); `fn` is universally learnable. |
| Data-type declaration | `type` | `species` is decorative, not illuminating (T-illuminate fail); `type` is universal. Ratified: keep `type`. |
| Trait / typeclass (a shared behavior set; LR-2) | `trait` | Ratified conventional (the themed `guild` was considered and declined): `trait` is machine-familiar and precise; theming costs dual-readability for no behavioral teaching. |
| Import / dependency | `use` | Ratified conventional: the *network* model is themed (§2) but the keyword stays `use` for learnability (Rust-familiar). |
| Pattern match | `match` | Universal control form; theming (`sift`) costs learnability for no teaching gain. |
| Conditional | `if` / `else` | Universal; maximal dual-readability. |
| Booleans / option / result | `true` `false` `Option` `Some` `None` `Result` `Ok` `Err` | Established, precise, machine-familiar; the never-silent partiality (S5) is carried by *using* `Result`, not by renaming it. |
| The representation change | `swap` | **Native Mycelium term already** (RFC-0001 §4.5 `Swap` node) — not borrowed; maximally clear; the signature operation reads the same in IR, RFCs, and surface. |
| Guarantee tags | `Exact` `Proven` `Empirical` `Declared` | Precise established corpus terms (the honesty lattice); renaming would obscure the normative vocabulary. |
| Selection policy | `policy` | Established corpus term (RFC-0005); precise. |
| Totality posture (LR-4/Q4) | `total` / `partial` | Precise technical terms; `total` gates promotion — clarity matters more than theme. |

---

## 4. Operators, punctuation, literals (proposed)

- **Representation-typed literals (S1, Q6 — universal-until-elaboration):** binary `0b1011_0010`,
  balanced-ternary `<+0--0>` (MSB-first over `{+,0,-}`, matching `binary-ternary.md`), decimal
  integers, dense/scalar arrays `[1.5, -2.25]`, VSA via Ada-2022-style literal functions on the
  model. **No defaulting across representation families** (stricter than Rust's `i32` default): a
  literal is universal until elaboration assigns exactly one `Repr`, and an ambiguous one is an
  explicit error, never a silent default.
- **The swap form** keeps named arguments for never-silent legibility:
  `swap(x, to: Ternary{6}, policy: roundtrip)` (the target and policy are always lexically present
  — S1/WF2).
- **Type ascription** `x: Binary{8}`; **arrow** `->` for function result types; **fat arrow** `=>`
  for match arms; `=` for definitions. Conventional, high dual-readability.
- **Guarantee annotation** surfaced as a type-level index (LR-6), e.g. `Ternary{6} @ Exact` ⚑ — the
  `@strength` spelling is a flagged proposal (alternatives: a keyword, a refinement brace).

---

## 5. The `wild` block (LR-9 / S6 — reserved now)

Safe Mycelium has **no manual allocation/free and no raw memory** — a leak is not expressible
(LR-9). The single escape hatch is a lexically-marked, **denied-by-default** `wild` block, the only
place raw FFI / foreign memory is reachable:

```text
fn parse(bytes: Bytes) -> Result(Tree, Error) = …   // safe: no leak expressible

wild {                       // audited, opt-in; safe code cannot reach in here
    foreign_decode(ptr, len) // raw FFI / foreign memory only valid inside `wild`
}
```

Properties the grammar/checker must enforce (per LR-9/S6): `wild` is **not reachable from safe
code without the keyword**; the block is the minimal auditable unit; the toolchain can enumerate
every `wild` site (the audit story); and a colony with no `wild` blocks is *certified leak-free by
construction*. Reserving the keyword now fixes the boundary in the spec before FFI exists (Q8).

---

## 6. What is deliberately **not** reserved (and why)

- **No themed control flow** (`loop`/`while`/recursion words) — recursion is `fn` + the totality
  posture (§3); loops are not a core form at L1 (recursion is). Reserving fewer words keeps the
  grammar small (KC-3 spirit) and learnable.
- **No themed punctuation** — symbols are universal; theming them would wreck machine readability.
- **The guarantee/honesty vocabulary stays normative-as-is** — it is shared across IR, RFCs, and
  surface; one spelling everywhere.
- **Identifiers may freely use fungal names** without being reserved — a user can name a value
  `hypha` or a colony `armillaria`; theming the *culture* of naming (stdlib, examples, tooling)
  carries identity without enlarging the reserved set.

---

## 7. Judgment calls — **resolved by maintainer ratification (2026-06-10)**

1. **Trait term** — ratified **`trait`** (conventional); the themed `guild` was declined.
2. **`spore`** for the reconstruction manifest (surface term; schema stays
   `reconstruction-manifest`) — ratified **yes**.
3. **Promoted stable component** — ratified **`matured`** (over `hardened`/`sclerotium`).
4. **Type term** — ratified **`type`** (kept conventional).
5. **Unsafe block** — ratified **`wild`**.
6. **Guarantee-annotation spelling** — ratified **`T @ Exact`** (the `@strength` index, LR-6).
7. **Import keyword** — ratified **`use`**; the `network` theme lives in the dependency model, not
   the keyword.

---

## 8. How this routes to the build (RFC-0006 §4.3)

On ratification, the agreed set becomes:

- the **terminal/keyword table** in `docs/spec/grammar/` (W3C-notation EBNF + the LR(1) oracle);
- the **reserved-word list** the lexer and the M-141 linter enforce (a fungal identifier that
  collides with a reserved word is an explicit diagnostic, never silently shadowed);
- fixtures in the **accept/reject conformance corpus** exercising every keyword and the `wild`
  boundary;
- the canonical-formatter (M-142) spelling, which is what gets content-addressed (S3).

Until ratified, the L1 prototype uses this set **provisionally** and flags it as non-normative.

---

## Meta — changelog & maintenance

- **2026-06-16 — §2 `colony` line superseded by DN-06 (append-only).** DN-06 reassigns **`colony`**
  from its §2 static meaning ("Module / namespace of definitions") to a **dynamic runtime grouping of
  `hypha`** (RFC-0008 §4.7), moving the static role to the new term **`nodule`** (the basic unit) with
  **`phylum`** as the library-scale level above it. This supersedes the §2 `colony` row only; the rest
  of DN-02 stands. Justification: a higher-fidelity T-map now that RFC-0008 has a genuine dynamic
  grouping (DN-06 §3). The keyword migration is tracked as M-358. *Supersede-don't-edit:* §2's text is
  left intact above, superseded by reference per DN-02's own append-only law.
- **2026-06-10 — Draft.** Initial proposal: the naming law (three-test gate from maintainer
  direction), the themed set, the conventional-kept set, literals/operators, the `wild` unsafe
  boundary, and seven flagged judgment calls (§7). Language name = Mycelium (shared).
- **2026-06-10 — Resolved.** Maintainer ratified §7: keep **`trait`** (declined `guild`); **`spore`**
  yes; **`matured`** (over `hardened`/`sclerotium`); keep **`type`**; **`wild`**; **`@`**-annotation
  approved; import = **`use`**. The ratified set is frozen into the grammar artifacts (RFC-0006
  §4.3). Superseding any term is a new note, not a rewrite.
