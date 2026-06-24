# ADR-013: `spore` Is the Deployable Unit; the Reconstruction Manifest Is One Component

| Field | Value |
|---|---|
| **ADR** | 013 |
| **Title** | `spore` is the content-addressed deployable unit; the RFC-0003 reconstruction manifest is one digest-referenced component of it |
| **Status** | **Accepted** (maintainer deliberation, 2026-06-10) |
| **Date** | 2026-06-10 |
| **Depends on** | DN-02 (ratified `spore` surface term); RFC-0003 §6 (reconstruction manifest, Accepted); ADR-012 §7.4 (the scope-drift flag this resolves); RFC-0008 §4.4 (the runtime model's place for the artifact); ADR-003 (content-addressed identity); research **T4.3/T4.4** |
| **Amends** | RFC-0003 → r2 (a scope *note* on §6; the manifest's content and schema are unchanged) |

## 1. Context

Two meanings of `spore` arose and were silently diverging (ADR-012 §7.4):

- **Narrow (ratified first):** DN-02 §2 / RFC-0003 §6 fix `spore` as the **reconstruction
  manifest** — the self-contained recipe to regrow a *value* (model + codebooks by content hash +
  recipe + decoding procedure + the `{ε, δ, strength}` certificate).
- **Broad (Lexicon Reference v0.4):** an immutable, verifiable artifact of **code + state +
  metadata** that can *germinate into execution* — a deployable unit.

Research Pass 4 grounded the broad sense: three independent artifact ecosystems converge on
**hash-identified DAGs of code + config + state** (Nix store paths, OCI image digests, Wasm
components — T4.4), and Unison demonstrates ship-by-hash code mobility as a *consequence* of
content addressing (T4.3), which Mycelium already adopted (ADR-003). RFC-0008 §4.4 gave the
runtime model a place for the artifact but deliberately did not redefine the term.

## 2. Decision

**`spore` is the content-addressed deployable unit**: a hash-identified DAG of

1. **code** — content-addressed definitions (ADR-003), shipped by hash with dependencies
   resolvable on demand (the Unison recipe, T4.3);
2. **values** — the initial/captured state, as ordinary Mycelium values with their `Meta` intact;
3. **the reconstruction manifest** (RFC-0003 §6, **unchanged**) — one digest-referenced
   component, carrying exactly what it always carried;
4. **artifact metadata** — provenance, guarantee/bound certificates, signatures.

The **narrow sense is the degenerate case**, not a casualty: the surface expression `spore(v)`
constructs the single-value spore whose payload is `v`'s reconstruction manifest — so every
existing use of the term remains correct, and the two meanings are one meaning at two sizes.
The schema name `reconstruction-manifest` is **unchanged** (DN-02 §2 ratified the schema/surface
split); the full deployable-artifact schema (germination contract included) lands with the
runtime implementation stages (RFC-0008 §4.6 R2 / the M-260 reconstruction work), not here.

## 3. Rationale

- **No silent divergence** (the house rule): one term with two unreconciled meanings is exactly
  the kind of ambiguity the corpus forbids; reconciling by *composition* (manifest ⊂ spore)
  preserves both ratified usages.
- **The metaphor is accurate at both sizes** (DN-02 T-map): a biological spore carries everything
  needed to regrow the organism — for a value, that is the manifest; for a computation, that is
  code + state + manifest.
- **The broad sense is grounded, not aspirational** (T4.3/T4.4): content-addressed deployable
  DAGs are mature engineering practice with three independent precedents, and Mycelium's
  identity model already provides the load-bearing property.

## 4. Consequences

**Positive:** the Lexicon Reference's broad definition becomes corpus-consistent; RFC-0008 §4.4
("the manifest is the natural component of the larger artifact") is ratified rather than
provisional; `germinate` has a defined subject when its RFC arrives.

**Negative / costs:** RFC-0003 needs an r2 scope note (done with this ADR); the deployable
artifact's schema, signing story, and germination contract are *new obligations* on the
RFC-0008 implementation stages — deliberately deferred, and flagged as such (R8-Q5 is hereby
resolved at the *scope* level; the schema work remains open).

**Routes to:** DN-03 (records the surface meaning); `docs/Doc-Index.md`; the Lexicon Reference
status note; RFC-0008 §4.4/§8 R8-Q5.

## Meta — changelog

- **2026-06-10 — Accepted.** Maintainer deliberation resolved ADR-012 §7.4 by generalization:
  spore = deployable unit, manifest = component, narrow sense = degenerate case. RFC-0003
  amended to r2 (scope note only).

> **Footnote — tunable certification (RFC-0034 / ADR-032, 2026-06-24; append-only).** Spore content-hash identity is a **compile/deploy-phase** concern and **remains available even when the runtime certification mode is fully off** (`fast`) — deployability survives a cert-off runtime (RFC-0034 §8). Disabling the *compile* spore hash is a separate, explicit, `EXPLAIN`-ed choice (never silent — G2). The identity mechanism is **unchanged**. See **ADR-032**.
