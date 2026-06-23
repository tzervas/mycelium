# RFC-0031 — Self-Hosted Standard Library Composition

| Field | Value |
|---|---|
| **RFC** | 0031 |
| **Status** | **Draft** (2026-06-23) |
| **Type** | Foundational / normative (once Accepted) — the self-hosting composition model and phylum layout for the stdlib and core libs |
| **Date** | 2026-06-23 |
| **Feeds** | E13-1 (stdlib in Mycelium — the defining full-language-1.0.0 criterion) |
| **Decides** | Which modules are irreducibly Rust vs must be `.myc`; the phylum layout and layering above the bare Rust core; the per-module stability bar; the migration order |
| **Depends on** | RFC-0016 (Core Library — scope/contract/taxonomy; **Enacted**); RFC-0006/RFC-0007 (surface + L1 kernel calculus); RFC-0019 (traits/parametric polymorphism); RFC-0024 (HOF via static defunctionalization — unblocks combinators); ADR-003 (content-addressed identity); ADR-013 (`spore` deployable artifact); KC-2/KC-3 (small auditable kernel, stability bar); G2/VR-5 (never-silent, honesty tags) |
| **Coupled with** | `lib/std/result.myc` (M-649 — first self-hosted nodule, the composition prototype); `docs/spec/stdlib/*.md` (25 module specs); `crates/mycelium-std-*` (Rust-first crates to be superseded); `docs/Doc-Index.md`; E13-1 children M-714…M-719 |
| **Task** | E13-1 (epic) / M-714 (this RFC's authoring task) |

> **Posture (honesty rule / VR-5).** Advisory stub — decides nothing normatively. The
> composition model, phylum layout, and migration order are **open questions** enumerated in §5.
> No module is declared self-hosted until its `.myc` nodule differential-tests pass and the
> corresponding `mycelium-std-*` Rust crate is retired or relegated to the irreducible core.
> `lib/std/result.myc` (M-649) is the only currently self-hosted nodule; all other
> `mycelium-std-*` crates are Rust-first. Claims about "what must be `.myc`" are `Declared`
> positions, not `Proven` ones, until checked by implementation.

---

## 1. Problem / Goal

RFC-0016 (Enacted) fixed the stdlib scope, per-op contract (C1–C6), ring layering, and
Tier-A/Tier-B taxonomy. It deferred the **self-hosting migration** to Phase 5 (M-502-gated,
KC-2 ruling). The full-language-1.0.0 north star is: the stdlib + libraries beyond the bare
Rust core are **written fully in Mycelium (`.myc`)**, stable, and fully usable — not Rust with
`.myc` as a thin veneer.

Today only `lib/std/result.myc` self-hosts (M-649). The remaining 24+ module families are Rust
crates (`mycelium-std-*`). This RFC must answer:

1. Which parts of the system are **irreducibly Rust** (the bare kernel that cannot safely be
   written in `.myc` because the language itself depends on them — bootstrapping, FFI, unsafe
   memory, platform I/O) vs which **must become `.myc`** to satisfy the 1.0.0 criterion?
2. What is the **phylum layout** — how are nodules grouped into phyla, how do they layer on
   the Rust core, and what names/paths do they expose?
3. What is the **per-module stability bar** — the acceptance criteria a `.myc` port must meet
   before the Rust crate is retired?
4. What is the **migration order** — which modules are ported first and why?

## 2. User stories

- As a **language user**, I want to write programs that import `std.collections`, `std.text`,
  and `std.math` as `.myc` nodules, so that my code does not depend on Rust internals and the
  full honesty contract (G2/VR-5) is visible at source level.
- As a **stdlib author**, I want a clear spec of which modules belong to the irreducible Rust
  core vs the `.myc` migration target, so that I know what to port and in what order without
  re-litigating scope at each PR.
- As a **compiler engineer**, I want the phylum layout and layering to be defined before I port
  any module, so that import paths, content-addressed identity (ADR-003), and `spore` packaging
  (ADR-013) are consistent across the migration.
- As a **library/phylum author**, I want the per-module stability bar documented (differential
  test requirements, guarantee-tag obligations, API-stability promises), so that a contributed
  `.myc` phylum can be accepted without ad hoc review criteria.
- As a **maintainer**, I want the migration order to be sequenced by unblock dependencies
  (HOF, traits, operator syntax) and risk (differentiator Tier-A modules first), so that each
  increment is reviewable and the wave does not collapse into a big-bang rewrite.
- As a **downstream app developer**, I want the public API surface of each `.myc` module to be
  stable before I depend on it, so that upgrading Mycelium does not silently break my code.

## 3. Scope and decision space

### In scope

- Defining the boundary between the irreducible Rust core (bootstrapping layer, unsafe memory,
  platform I/O, the L0 interpreter itself) and the `.myc` migration target.
- Specifying the phylum layout: nodule names, grouping into phyla, import path conventions,
  relationship to the 25 `docs/spec/stdlib/*.md` module specs.
- Defining the per-module stability bar (differential test requirements, guarantee tags,
  API-freeze criteria, deprecation of the corresponding Rust crate).
- Specifying the migration order (sequencing by language-surface readiness: HOF/RFC-0024,
  traits/RFC-0019, operator syntax unblocks math; differentiator Tier-A first).
- Relating to `lib/std/result.myc` as the composition prototype: what patterns it establishes
  that all ports must follow.

### Out of scope

- Module internals: each module's own design is covered by `docs/spec/stdlib/<module>.md` and
  its own Phase-5 task (M-515…M-534, M-714…M-719). This RFC is layout + composition, not
  per-module semantics.
- Changes to the RFC-0016 per-op contract (C1–C6) or taxonomy (Tier-A/B) — those are Enacted;
  supersede RFC-0016 to change them.
- The Rust kernel crates (`mycelium-core`, `mycelium-l0`, `mycelium-l1`) — they are the
  irreducible base; this RFC defines the boundary, not the kernel's internals.
- Runtime/concurrency (`std.runtime`, `std.sys`) — Phase-7-gated per RFC-0016 Q4 deferral.

## 4. Definition of Done

- [ ] The irreducible-Rust boundary is defined: a named list of crates/modules that stay Rust
  forever and the reasoning for each (bootstrapping, unsafe, platform ABI).
- [ ] The `.myc` migration target is named: a ranked list of modules with their readiness
  preconditions (which language-surface features must land first).
- [ ] The phylum layout is specified: nodule import paths, phylum groupings, versioning, and
  how they layer on the Rust core — consistent with ADR-003 (content-addressed) and ADR-013
  (`spore`).
- [ ] The per-module stability bar is documented: what a `.myc` port must demonstrate
  (differential tests vs Rust reference, guarantee tags, API-freeze commitment) before the
  Rust crate is retired.
- [ ] The migration order is sequenced and grounded in language-surface dependencies
  (RFC-0024 HOF for combinators, RFC-0019 traits for `iter`/`cmp`, operator syntax for `math`).
- [ ] `lib/std/result.myc` is analysed as the composition prototype and the patterns it
  establishes are codified as requirements for all subsequent ports.
- [ ] This RFC reaches **Accepted** (maintainer ratification) before any M-715…M-719 leaf
  begins implementation.
- [ ] All open questions in §5 are resolved or explicitly deferred with direction.

## 5. Open questions

1. **Irreducible-Rust boundary** — which crates/modules can never safely be `.myc`? Candidates
   for the irreducible core: `mycelium-core` (the value model, registry, guarantee lattice),
   `mycelium-l0` (the interpreter trusted base), `mycelium-l1` (the compiler frontend), the
   `swap`/`cert` machinery, unsafe memory operations, and platform I/O. Is this the right
   cut? What is the decision criterion?
2. **Phylum naming and paths** — RFC-0016 §Q2 resolved: phylum `std`, crate-mirrored names.
   Does `std.result` (already in `lib/std/result.myc`) set the convention for all ports?
   What is the canonical root for the `lib/std/` tree?
3. **Bootstrap circularity** — the `.myc` compiler is written in Rust (L1 frontend); porting
   stdlib modules to `.myc` requires the compiler to compile them. Is there a circularity
   risk, and if so, how is it staged?
4. **Migration order** — RFC-0016 Q1 recommended `diag`/`recover` lead migration; but HOF
   (RFC-0024) is now landed, unblocking `core/prelude` combinators first. What is the
   revised order? Is `core` (Option/Result/Ord/Eq) before `iter` before `collections`?
5. **Stability bar granularity** — must every exported function have a differential test
   (interpretor ≡ Rust reference), or is per-module aggregate coverage sufficient? What is
   the minimum guarantee-tag obligation (all `Declared`, or `Empirical` required for
   performance-sensitive ops)?
6. **Rust crate retirement** — when a `.myc` port is accepted, how is the `mycelium-std-*`
   Rust crate retired (deprecated, removed, or kept as the irreducible-Rust reference
   implementation for differential testing)?
7. **`spore` packaging** — how are `.myc` nodules packaged as `spore` artifacts (ADR-013)?
   Is each phylum one `spore`, or one per nodule? What is the versioning policy?

## 6. Grounding / honesty

- RFC-0016 (Enacted, 2026-06-21) — the scope/contract/taxonomy basis; this RFC extends it.
- `lib/std/result.myc` (M-649, landed) — the only existing self-hosted nodule; the composition
  prototype all claims about "the pattern" must ground in.
- `docs/spec/stdlib/*.md` — 25 module specs (exist, checked 2026-06-23); the design basis for
  each port.
- RFC-0024 (HOF, implemented Rust-first, pending ratification) — unblocks `map`/`and_then`/`fold`.
- RFC-0019 (traits/polymorphism) — unblocks `iter`, `cmp`, `Ord`/`Eq`.
- ADR-003 (content-addressed identity), ADR-013 (`spore`) — packaging constraints.
- KC-2/KC-3, G2, VR-5 — non-negotiable house rules that apply to every `.myc` port.

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-23 | **Draft** | Initial stub — open questions enumerated; no normative decisions. Task: E13-1/M-714. |
