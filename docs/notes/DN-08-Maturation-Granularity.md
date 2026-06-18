# Design Note DN-08 — Maturation Granularity (`matured` at module/library/program scope)

| Field | Value |
|---|---|
| **Note** | DN-08 |
| **Status** | **Draft** (2026-06-18; captured maintainer intent, direction open) — advisory. It does **not** alter RFC-0007's Accepted per-definition `matured` gate; a future RFC-0007 amendment (or successor) would ratify any change. |
| **Feeds** | RFC-0007 §4.5 (the `matured ⟹ total` gate — currently **per-definition**); Glossary §2.10 (`matured` = "promoted, stable, compiled-and-frozen"; "takes the AOT path"); DN-06 (the static-organization lexicon: `phylum` = library-scale, `nodule` = basic unit/module; `colony` = *dynamic* runtime grouping); DN-02/DN-03 (reserved words — `matured`); `docs/spec/grammar/` |
| **Date** | June 18, 2026 |
| **Decides** | *Nothing normatively.* Records the intended **scope** at which `matured` is expressed and normally applied, the developer-workflow rationale, and the open questions a future ratification must close. |
| **Task** | (none yet — kernel/AOT maturation-ergonomics capture; relates to RFC-0007) |

> **Posture (honesty rule).** Advisory capture of maintainer intent. RFC-0007's status is
> untouched. Where this note proposes a direction it is a grounded proposal for the maintainer to
> accept or reject (the planning analogue of G2), never a decision. Append-only: a future change
> would **supersede** the relevant RFC-0007 clause, not rewrite it.

---

## 1. The intent being captured

Today `matured` attaches to a **single definition**: RFC-0007 §4.5 gates it on totality ("only
`total` definitions may be `matured`") and Glossary §2.10 says a `matured` definition "takes the AOT
path". The surface form is `matured fn …`.

The maintainer's intent is that maturation should be expressed — and, in the normal workflow,
*applied* — at **coarser scopes**, mapped onto the ratified organization lexicon (DN-06):

- **`matured` applies at module (`nodule`), library (`phylum`), and program/package level — each.**
  These are the natural units a developer promotes to the compiled (AOT) path.
- **Per-definition maturation is atypical.** A developer generally does **not** maturate a single
  `fn`/method. They reach a stable point for a whole module/script and mature *that*. The
  subcomponents of a Mycelium file/`nodule` are **not** selectively compiled in the normal workflow.
- **The fine-grained operation that does make sense is the inverse, and it is rare:** selectively
  shifting *one* subcomponent **back to interpreted** inside an otherwise-matured scope — a targeted
  *de*-maturation, e.g. to iterate on or debug it. Even this is highly atypical.

**Rationale (developer workflow).** Compilation is a *stable-point* concern. Developers do not think
about compiling individual functions; they mature a module once it stabilizes — just as one would
not selectively compile arbitrary subcomponents of a source file. So the ergonomic default is
**coarse-grained maturation**, with fine-grained **de-maturation** as the escape hatch — not
fine-grained maturation as the unit of promotion.

## 2. Relationship to the current (Accepted) design

RFC-0007 r4 (Accepted) defines the `matured` **gate** (`matured ⟹ total`) over definitions; its
soundness rests on the totality check, not on the *granularity* at which `matured` is written. This
note leaves that gate intact. It proposes that the **scope** `matured` attaches to be lifted to
`nodule`/`phylum`/program, with the totality obligation quantified over the matured scope (every
definition reachable in a matured `nodule` must be `total` — the same gate, just read over the
scope). The per-`fn` form would then be either retained as sugar / for the rare case, or reframed.

## 3. Open questions (for a future RFC-0007 amendment / successor)

1. **Surface forms.** How is module/library/program maturation written — `matured nodule …`, a
   `phylum`-level annotation, a manifest/header declaration (DN-06 declares `nodule` status by header,
   not filename), or a build-target attribute? Which keyword carries "program/package"?
2. **Totality quantification.** Confirm "matured `nodule` ⟹ every reachable definition `total`" is the
   intended reading and that it composes cleanly with the existing per-definition gate.
3. **Selective de-maturation.** The inverse operation's surface form + semantics (shift one
   subcomponent back to interpreted within a matured scope), and whether it weakens any guarantee the
   matured scope advertised (it must stay never-silent / EXPLAIN-able — KC-3, G2).
4. **Migration.** Does `matured fn` stay (as sugar / rare case), get deprecated, or get superseded?
   (Append-only — a change supersedes the RFC-0007 clause.)
5. **Registry / EXPLAIN.** How a coarser maturation unit is reified and `EXPLAIN`-able (no black
   boxes, KC-3) — and how the AOT/interpreted boundary is recorded per scope.

## 4. Not in scope

This note does not fix surface syntax, amend RFC-0007, or touch the `matured ⟹ total` soundness
argument. It records the intended granularity + the questions a ratification must answer.

---

## Changelog

- **2026-06-18** — Initial capture (Draft). Records the maintainer intent that `matured` apply at
  `nodule`/`phylum`/program scope (coarse-grained), with per-`fn` maturation atypical and selective
  *de*-maturation the rare fine-grained operation. Grounded in RFC-0007 §4.5, Glossary §2.10, DN-06.
