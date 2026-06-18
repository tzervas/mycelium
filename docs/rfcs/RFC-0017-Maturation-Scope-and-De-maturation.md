# RFC-0017 — Maturation Scope (Module / Library / Program) & De-maturation

| Field | Value |
|---|---|
| **RFC** | 0017 |
| **Status** | **Accepted** (2026-06-18 — maintainer sign-off ratifying DN-08's direction). Lifts `matured` from **per-definition** to **scope** granularity: a `nodule` or `phylum` is matured via its **header**, a program/package via its **`mycelium-proj.toml` manifest**; **`matured fn` (per-definition maturation) is retired** (superseded — no longer expressible). Reserves **`thaw`** (Surface; conventional-clearest) as the rare in-source **de-maturation** marker. **Supersedes the *granularity* of RFC-0007 §4.5** (the per-definition reading); the gate's **soundness argument is unchanged** — `matured ⟹ total` is quantified over the matured scope. |
| **Type** | Foundational / normative |
| **Date** | June 18, 2026 |
| **Supersedes** | RFC-0007 §4.5 *granularity* (the per-definition framing of where `matured` attaches) — append-only; the §4.5 totality/soundness argument is **retained, re-quantified**, not rewritten. Ratifies **DN-08** (→ Resolved). |
| **Depends on** | RFC-0007 §4.5 (the `matured ⟹ total` gate + the totality checker — unchanged); RFC-0004 §4 (the stable-component AOT-eligibility gate — unchanged, applied per reachable definition); DN-06 (`nodule`/`phylum` organization lexicon); the *Nodule Header & Project Manifest* spec (Accepted — the header/manifest this RFC extends); DN-02/DN-03 (reserved words; `thaw` added via the three-test gate, §5); ADR-003 (content-addressed identity); RFC-0006 §4.1 S2/S4/S5 (honest tags surface, inspectable elaboration, explicit partiality); KC-3, G2, VR-5 |
| **Coupled with** | `docs/spec/grammar/mycelium.ebnf` (drops `matured` from `fn_item`, adds `thaw`); `docs/Glossary.md` (`matured`, `thaw`); the conformance corpus; `crates/mycelium-l1` + `mycelium-build` (the enacting tasks, anticipated) |

## 1. Summary

Maturation — the promotion of interpreted code to the compiled-and-frozen AOT path (Glossary
§2.10; RFC-0004 §4) — was, in RFC-0007 §4.5, expressed **per definition** (`matured fn …`). DN-08
captured the maintainer's intent that this is the wrong granularity for the developer workflow:
compilation is a **stable-point** concern that applies to a whole **module (`nodule`)**, **library
(`phylum`)**, or **program/package**, not to an individual function. This RFC ratifies that intent.

It makes three changes and **nothing else**:

1. **Maturation is declared at scope, in metadata, not per-definition in the term grammar.** A
   `nodule` or `phylum` is matured through its **header** (`// @matured: true`); a program/package
   through its **manifest** (`[project].matured` / a build target).
2. **`matured fn` is retired.** Per-definition maturation is no longer expressible (superseded, not
   kept as sugar — maintainer decision). The matured *unit* is always a scope.
3. **De-maturation gets a name and a meaning.** `thaw` (Surface; `germinate` is taken by spore-germination, ADR-013) marks the rare inverse
   — one definition kept **interpreted** inside an otherwise-matured scope, for iteration/debugging —
   never-silent and `EXPLAIN`-able.

The **soundness of the gate is untouched**: `matured ⟹ total` (RFC-0007 §4.5) and the
stable-component checks (RFC-0004 §4) are exactly as before, now **quantified over the matured
scope** (§4.2). The totality checker, the fuel-guarded interpreter, and the content-addressing are
all unchanged.

## 2. Motivation

**Developer workflow (DN-08 §1).** Developers do not reason about compiling individual functions;
they mature a module once it stabilizes, exactly as one does not selectively compile arbitrary
subcomponents of a source file. The ergonomic default must therefore be **coarse-grained
maturation**, with fine-grained **de-maturation** as the rare escape hatch — the opposite of the
per-`fn` default RFC-0007 §4.5 implied.

**Metadata, not term-grammar (KC-3 / S4).** "Which scopes are compiled" is a *build/packaging*
property of an organizational unit, not a property of a term. It belongs with the other scope
metadata (license, version, edition) the *Nodule Header & Project Manifest* spec already governs —
where it is inspectable, inheritable, and `EXPLAIN`-able — not threaded through the L1 term grammar.
This keeps the trusted kernel and the term language unchanged (the node budget does not move).

**Honesty (G2 / VR-5).** A scope's compiled/interpreted boundary, and any per-definition
`thaw` exception within it, must be **never silent** and fully `EXPLAIN`-able — the same
no-black-box discipline maturation already owed at definition scope.

## 3. Guide-level explanation

```mycelium
// nodule: geometry.kernel
// @version: 1.2.0
// @matured: true            ← this whole nodule takes the AOT path (RFC-0004 §4)

fn rotate(p: Dense{2, F32}, theta: Dense{1, F32}) -> Dense{2, F32} = …   // matured (AOT)
fn scale (p: Dense{2, F32}, k:     Dense{1, F32}) -> Dense{2, F32} = …   // matured (AOT)

thaw fn experimental_shear(p: Dense{2, F32}) -> Dense{2, F32} = …   // de-matured: stays interpreted
```

- **A matured scope** is one whose header (`nodule`/`phylum`) or manifest (program/package) declares
  it matured. **Every definition reachable in that scope** must satisfy the existing gate
  (`total` + stable-component checks); if any does not, maturation of the scope is an **explicit
  refusal** that names the offending definition and the failed obligation — never a silent partial
  compile.
- **`thaw fn f`** opts *one* definition **out** of its enclosing scope's maturation: `f` runs
  interpreted even though its nodule is matured. This is the rare fine-grained operation (DN-08 §1):
  you reach for it to iterate on or debug `f` without de-maturing the module around it.
- **There is no `matured fn`.** A single function cannot be matured in isolation; the unit of
  maturation is the scope. (To compile just one function, put it in its own matured `nodule`.)

`EXPLAIN` on any definition answers "is this compiled, and why?": *matured because nodule
`geometry.kernel` is matured (header) and all checks passed*, or *interpreted because `thaw`*,
or *interpreted because the enclosing scope is not matured*.

## 4. Reference-level design (normative)

### 4.1 Surface forms

**Scope maturation is metadata, resolved by the existing header/manifest inheritance** (*Nodule
Header & Project Manifest* §4):

- **`nodule` / `phylum` header key** — `// @matured: <bool>` (default `false`). Added to the spec's
  closed v0 key set (that spec's §3; this RFC is the "explicit decision" its append-only key-set rule
  requires). It is **inherited top-down like the other scope keys** (a matured `phylum`/nodule-root
  matures its subnodules) and **overridable locally** (a subnodule may set `// @matured: false` to
  stay interpreted — a `narrowing` override, always allowed; the inverse, maturing a subnodule under
  an unmatured root, is also allowed and explicit). A malformed value is an explicit error (VR-5).
- **Program/package manifest** — `[project].matured = true` matures the project's own nodules; a
  build may also select scopes via a **target set** (RFC-0004 §4's "explicit target set"), e.g.
  `[build].matured = ["geometry.kernel", "numerics.dot"]`. Spellings illustrative and refinable
  append-only (RFC-0006 surface discipline); the **normative content is that maturation is a
  resolved, inspectable scope attribute**, not a term modifier.

**De-maturation is in-source, definition-level:** `thaw fn …` (and, when methods/other
definition forms land, `thaw` prefixes them the same way). It is the **only** per-definition
maturation control that survives, and it only ever moves a definition *toward* interpreted.

**Retired:** the `matured` prefix on `fn_item` (RFC-0007 §4.5 / the v0 grammar). `matured` is no
longer a term/definition modifier; it remains a **reserved word** (so it can never be a silent
identifier) used as the header/manifest key. A source `matured fn …` is now a **parse error** with
a teaching diagnostic ("maturation is declared per `nodule`/`phylum` in the header — RFC-0017 §4.1").

### 4.2 Totality & gate, quantified over the scope (the soundness-preserving re-reading)

RFC-0007 §4.5 reads "only `total` definitions may be `matured`" over a definition. This RFC reads
the **same gate over a scope**:

> **A matured scope is well-formed iff every definition reachable in it is `total`** (RFC-0007 §4.5,
> unchanged classifier) **and AOT-eligible** (RFC-0004 §4 stable-component checks, unchanged),
> **except** definitions marked `thaw`, which are excluded from the matured set and run
> interpreted.

This is **the same obligation, universally quantified** — not a new or weaker one. Soundness is
immediate from RFC-0007 §4.5: each compiled definition individually satisfies `matured ⟹ total`, so
the scope-level claim is just their conjunction. A `thaw` definition is **not** in the matured
set, so it carries no totality obligation from maturation (it may be `partial`); it runs on the same
fuel-guarded interpreter as any unmatured code (RFC-0007 §3). The totality checker, the clocked
semantics, and "the checker gates packaging, never meaning" (RFC-0007 §4.5) are **verbatim
unchanged**.

**Refusal is explicit and total (G2).** If any non-`thaw` reachable definition in a scope
declared matured fails `total` or a stable-component check, maturing the scope is an **error** that
names the definition and the failed obligation. There is no "compile the parts that pass" — a
matured scope is matured in full (minus its explicit `thaw` exceptions) or not at all.

### 4.3 De-maturation semantics (`thaw`)

`thaw fn f` in a matured scope means: **`f` is excluded from the matured set and executes
interpreted.** Precisely:

- **It only ever de-matures.** `thaw` has no effect in an unmatured scope (the definition was
  interpreted already) — there it is a no-op flagged by a lint (never silently meaningless, G2),
  guiding the author to remove it.
- **It weakens no advertised guarantee (S2/S5).** A definition's **honesty tag** (`Exact ⊐ Proven ⊐
  Empirical ⊐ Declared`) and its certified-swap obligations are properties of its *semantics*, which
  the AOT and interpreted paths share (NFR-7: interp ≡ AOT on the validated observable). So a
  `thaw` definition advertises **the same guarantee tag** it would matured — the only thing that
  changes is its **execution path** (interpreted, not AOT) and therefore its **performance profile**,
  which was never a guarantee the lattice covers. `thaw` may **not** be used to dodge a failed
  totality/stable-component check while still claiming the scope's compiled status — it removes the
  definition from the compiled set entirely, visibly.
- **It is `EXPLAIN`-able and reified (no black box, KC-3/S4).** The execution-path decision for every
  definition — `matured` (which scope, which checks passed) / `thaw` (de-matured here) /
  `interpreted` (scope not matured) — is recorded in the build's maturation record (§4.4) and
  surfaced by `EXPLAIN`.

### 4.4 Registry / EXPLAIN — the maturation record (no black box)

Maturation is **reified**, like selection (RFC-0005) and reconstruction (RFC-0003 §6):

- The build emits, per matured scope, a **maturation record** — content-addressed (ADR-003) over the
  scope's resolved identity — listing each reachable definition with its **route** (`Matured` /
  `Thawed` / `Interpreted`) and, for `Matured`, the **discharged obligations** it satisfied
  (RFC-0004 §4: hash-frozen, spec-ratified, verification discharged). This is the scope-level analogue
  of the `mycelium-build::BuildCertificate` (M-311), which already records the AOT/interpreted route
  and the checked obligations per component — this RFC's record is its **per-scope roll-up**, keyed by
  the resolved scope header/manifest.
- **Metadata is not identity (ADR-003).** Maturation status is *associated* metadata; it does **not**
  perturb a definition's content hash (the same code compiled or interpreted is the same definition).
  This mirrors the header/manifest spec's treatment of `@version`/`@license`.
- `EXPLAIN <def>` and `EXPLAIN <scope>` print the route + provenance (which header/manifest field, via
  which inheritance step, set the maturation), the same `EXPLAIN`-with-provenance the header spec §4
  already requires for every resolved field.

## 5. The `thaw` reservation (DN-02/DN-03 three-test gate, applied here)

`thaw` is added to the Surface reserved set. Per DN-02 §1's three-test gate (the law DN-03 §4
applies to every name), applied here so the lexicon decision is grounded where it is made.

> **Why not the fungal-themed `germinate`?** The intuitive themed inverse of maturation —
> *germination*, a dormant body returning to active growth — is **already the spore-activation /
> deployment term** across the corpus (RFC-0008 §"spore germinating"; **ADR-013** explicitly reserves
> `germinate` for the spore *germination contract*; the *Spore-Build-and-Publish-Contract* spec and the
> shipped `crates/mycelium-spore` use "germination surface" / "germinate" in code + error strings).
> Reusing it for de-maturation would violate **DN-03 §3 (one name per term)** and overload a live
> concept. So this RFC takes the **conventional-clearest** word, exactly as DN-03 keeps `mesh`/
> `reclaim`/`tier`/`for` conventional where a plain word is clearest and theming would cost dual
> readability for no teaching gain.

| Test | Result |
|---|---|
| **T-map** (metaphor fidelity) | **Strong — directly inverts the ratified metaphor.** `matured` is defined as "compiled-and-**frozen**" (Glossary §2.10); **`thaw`** is the exact inverse of *frozen* — the definition returns from the frozen/compiled state to the live, interpreted, iterable state. The "frozen ⇄ thawed" pairing is already latent in the `matured` definition. |
| **T-illuminate** (teaching) | **Good.** Teaches "unfreeze this one back to the live/iterable state, not compiled" — the precise developer intent (debug/iterate). |
| **T-learn** (dual readability) | **High.** Universal, one short word, reads cleanly for human and machine; pairs intuitively with `matured` (= frozen) as its inverse. |

**Passes** → adopt `thaw` (**conventional-clearest**, not themed — the DN-03 precedent; `germinate` is
unavailable, ADR-013). It is **active Surface** (a real production: `thaw fn …`), reserved-and-active
(unlike the reserved-not-active runtime names). Recorded append-only in DN-03's changelog (which defers
term-specific reservations to the ratifying RFC, as it did for `for` → RFC-0007 §4.8). *(Spelling
adjustable in maintainer review; the de-maturation **semantics** §4.3 are the normative content.)*

## 6. Drawbacks

- **A second place to look** for "is this compiled?" — the header/manifest, not the function. Mitigated
  by `EXPLAIN` (§4.4): the route is always answerable from the definition, and the metadata home is
  *where build properties already live* (KC-3), so it is less surface area than threading maturation
  through the term grammar.
- **Coarse default can over-compile** a module with one unstable function. That is exactly what
  `thaw` is for; and a single function that genuinely needs isolated compilation can live in its
  own matured `nodule` (§3).
- **Retiring `matured fn` is a breaking surface change.** Acceptable: the surface is design-phase and
  v0 (RFC-0006); the only artifacts using it are corpus examples + one conformance program, updated in
  lockstep. The change is append-only at the decision level (this RFC supersedes the §4.5 granularity).

## 7. Rationale & alternatives

- **Why metadata over a `matured nodule`/`matured phylum` source keyword?** The header/manifest is
  *already* the inspectable, inheritable, `EXPLAIN`-able home for scope properties (license, version,
  edition, exports). A source keyword would duplicate that machinery and split the "scope attributes"
  story across two surfaces. (Maintainer decision, DN-09-adjacent.)
- **Why retire `matured fn` rather than keep it as sugar?** A surviving per-`fn` form re-legitimizes
  the granularity DN-08 rejects and invites the "compile arbitrary subcomponents" mental model the RFC
  is removing. One unit of maturation (the scope) is DRY/KISS; `thaw` covers the single real
  fine-grained need (de-, not re-, maturation). (Maintainer decision.)
- **Why `thaw` in-source (not metadata too)?** De-maturation is a *local, transient developer
  act* on a specific definition during iteration — it belongs next to the code it affects, visibly,
  and it is the genuinely fine-grained operation. Putting it in the header would hide a per-definition
  exception in scope metadata (the opposite of never-silent).
- **Alternative — totality not quantified, only the "leaf" defs gated** (rejected): would let a matured
  scope contain a reachable `partial` definition compiled via some unmatured call — breaking the
  `matured ⟹ total` invariant. §4.2's universal quantification is the soundness-preserving reading.

## 8. Prior art

Cargo/Rust *profiles* and crate-level optimization (compilation is a crate/profile property, not a
per-`fn` one); Unison's content-addressed definitions with separate-compilation-from-hashing (ADR-003);
GHC's per-module compilation with `{-# NOINLINE #-}`-style local opt-out as the closest analogue to
`thaw` (a local exception to a module-wide policy); Idris/Lean per-definition `partial`/totality
pragmas (the granularity this RFC deliberately moves *away* from for promotion, while keeping their
totality *classification* per-definition).

## 9. Unresolved questions

- **R17-Q1 (illustrative spellings).** The exact manifest spelling for build-target maturation
  (`[project].matured` vs `[build].matured = [...]`) and whether a build profile (RFC-0004 §4 target
  set) subsumes it — a `mycelium-build` (M-311) refinement, append-only; the normative content
  (maturation = a resolved inspectable scope attribute) is fixed.
- **R17-Q2 (`thaw` beyond `fn`).** When methods (`impl`), data, or other definition forms enter
  the grammar (DN-03 §1), confirm `thaw` prefixes them uniformly (expected yes; additive).
- **R17-Q3 (cross-scope inlining under maturation).** Whether a matured scope may inline a `total`
  definition from an *unmatured* scope (it may, by §4.2's per-definition gate — the inlined callee must
  itself pass), and how the maturation record attributes it. Mechanism detail for the enacting build
  task; soundness already covered by §4.2.

## 10. Future possibilities

A `mycfmt`/lint affordance that suggests `thaw` for a definition repeatedly edited inside a
matured scope; maturation-aware caching (the M-312 content-addressed cache keyed by the resolved
maturation record); a profile vocabulary (`debug`/`release`-analogue) over scope maturation once the
runtime (RFC-0008) `tier` switch lands.

## Meta — changelog

- **2026-06-18 — Accepted (maintainer sign-off; ratifies DN-08).** Lifts `matured` from
  per-definition to **scope** granularity: `nodule`/`phylum` matured via header `// @matured: …`,
  program/package via the `mycelium-proj.toml` manifest; **retires `matured fn`** (superseded — per-
  definition maturation no longer expressible); reserves **`thaw`** (Surface, conventional-clearest — `germinate` taken by ADR-013; three-test
  gate §5) as the in-source **de-maturation** marker for the rare interpreted-inside-matured case.
  **Supersedes RFC-0007 §4.5's *granularity*** (append-only) while leaving its `matured ⟹ total`
  soundness argument unchanged — §4.2 quantifies the *same* gate over the matured scope. Reifies the
  maturation route per scope (`EXPLAIN`-able maturation record, §4.4; the M-311 certificate's per-scope
  roll-up); metadata is not identity (ADR-003). Closes DN-08's five open questions (surface forms §4.1,
  totality quantification §4.2, de-maturation §4.3, migration §4.1 [retire], registry/EXPLAIN §4.4).
