# RFC-0031 — Self-Hosted Standard Library Composition

| Field | Value |
|---|---|
| **RFC** | 0031 |
| **Status** | **Accepted** (2026-06-23) — composition model, irreducible-Rust boundary, phylum layout, stability bar, and surface-readiness-sequenced migration order ratified (§5 D1–D7). Was Draft (2026-06-23). |
| **Type** | Foundational / normative (once Accepted) — the self-hosting composition model and phylum layout for the stdlib and core libs |
| **Date** | 2026-06-23 |
| **Feeds** | E13-1 (stdlib in Mycelium — the defining full-language-1.0.0 criterion) |
| **Decides** | Which modules are irreducibly Rust vs must be `.myc`; the phylum layout and layering above the bare Rust core; the per-module stability bar; the migration order |
| **Depends on** | RFC-0016 (Core Library — scope/contract/taxonomy; **Enacted**); RFC-0006/RFC-0007 (surface + L1 kernel calculus); RFC-0019 (traits/parametric polymorphism); RFC-0024 (HOF via static defunctionalization — unblocks combinators); ADR-003 (content-addressed identity); ADR-013 (`spore` deployable artifact); KC-2/KC-3 (small auditable kernel, stability bar); G2/VR-5 (never-silent, honesty tags) |
| **Coupled with** | `lib/std/result.myc` (M-649 — first self-hosted nodule, the composition prototype); `docs/spec/stdlib/*.md` (25 module specs); `crates/mycelium-std-*` (Rust-first crates to be superseded); `docs/Doc-Index.md`; E13-1 children M-714…M-719 |
| **Task** | E13-1 (epic) / M-714 (this RFC's authoring task) |

> **Posture (honesty rule / VR-5).** This RFC decides the *composition model* — the boundary,
> layout, stability bar, and migration order (§5 D1–D7) — **normatively**. It does **not** declare
> any module self-hosted: a module is self-hosted only when its `.myc` nodule clears the §5 D5
> stability bar (differential tests pass, honest per-op tags, frozen signature). As of acceptance,
> `lib/std/result.myc` (M-649), `lib/std/option.myc`, and `lib/std/cmp.myc` (M-715, Tier-0) are
> self-hosted and differential-tested; all other `mycelium-std-*` modules remain Rust-first and
> migrate per the §5 D4 surface-readiness order. The central honesty stance (§5 D4): **only the
> structural/polymorphic core is executable today** — the `collections`/`text`/`math` ports are
> *blocked* on kernel prims (binary arithmetic, a reduce-to-`Bool` comparison, a sequence
> representation) that are not yet surfaced. This RFC sequences those ports **behind** their enabling
> prims rather than claiming a self-hosting it cannot yet demonstrate (VR-5). Claims about "what must
> be `.myc`" are `Declared` positions checked by implementation as each port lands.

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

- [x] The irreducible-Rust boundary is defined: a named list of crates/modules that stay Rust
  forever and the reasoning for each (bootstrapping, unsafe, platform ABI). → **§5 D1**.
- [x] The `.myc` migration target is named: a ranked list of modules with their readiness
  preconditions (which language-surface features must land first). → **§5 D4**.
- [x] The phylum layout is specified: nodule import paths, phylum groupings, versioning, and
  how they layer on the Rust core — consistent with ADR-003 (content-addressed) and ADR-013
  (`spore`). → **§5 D2, D7**.
- [x] The per-module stability bar is documented: what a `.myc` port must demonstrate
  (differential tests vs Rust reference, guarantee tags, API-freeze commitment) before the
  Rust crate is retired. → **§5 D5, D6**.
- [x] The migration order is sequenced and grounded in language-surface dependencies
  (RFC-0024 HOF for combinators, RFC-0019 traits for `iter`/`cmp`, prims for `math`). → **§5 D4**.
- [x] `lib/std/result.myc` is analysed as the composition prototype and the patterns it
  establishes are codified as requirements for all subsequent ports. → **§5 D5 (the prototype
  pattern); M-715 `option.myc`/`cmp.myc` follow it.**
- [x] This RFC reaches **Accepted** (maintainer ratification) before any M-715…M-719 leaf
  begins implementation. → **this revision; M-715 Tier-0 lands alongside acceptance.**
- [x] All open questions in §5 are resolved or explicitly deferred with direction. → **§5 D1–D7.**

## 5. Decisions (D1–D7) — resolving the open questions

> Each decision resolves the like-numbered open question from the Draft. They are **normative** for
> the migration; honesty (VR-5) governs every claim — nothing is declared self-hosted ahead of a
> passing differential test.

### D1 — The irreducible-Rust boundary

The following stay **Rust forever** (the bare kernel the language's own compilation/evaluation/trust
depends on); everything *above* this line is a `.myc` migration target:

| Stays Rust | Why (the criterion below) |
|---|---|
| `mycelium-core` | The value model itself (`Value`/`Repr`/`Meta`, the guarantee lattice, content-addressing). A language cannot define in itself the values it *is* — bootstrap floor (c). |
| `mycelium-l0` | The trusted reference interpreter — the evaluation semantics and the root of trust (KC-3) (a). |
| `mycelium-l1` | The bootstrap compiler frontend (lex/parse/check/elaborate/monomorphize). A self-hosted *compiler* is explicitly **out of scope** (RFC-0016 / KC-2: stdlib-only self-hosting) (a). |
| `mycelium-cert`, `mycelium-swap` | The certificate machinery + swap engines — the verifiable-swap trust root (RFC-0002) (b). |
| `mycelium-interp::prims` | The elementwise/arithmetic primitive registry (`bit.*`, `trit.*`) — the FFI to the value model the surface bottoms out on (c). |
| `mycelium-mlir` | AOT codegen — LLVM/MLIR FFI, `unsafe` (d). |
| `mycelium-std-sys` | Platform I/O + `wild`/FFI (unsafe memory, platform ABI; E14-1/`ffi10`) (d). |

**Decision criterion.** A module stays Rust **iff** it is (a) the **trust root** the certificate/
evaluation guarantees rest on, (b) the **bootstrap floor** the language's own compilation depends on,
(c) the **value-model FFI** (the prims / `core` types the surface bottoms out on), or (d) an
**unsafe / platform-ABI** boundary. A module that is none of these is a `.myc` migration target.

### D2 — Phylum naming and paths

Confirmed (extends RFC-0016 §Q2): phylum **`std`**; nodule path **`std.<module>`**; file
**`lib/std/<module>.myc`**; crate-mirrored names. `lib/std/` is the canonical root and
`lib/std/mycelium-proj.toml` is the phylum manifest (`[surface] exports`). `std.result` sets the
convention; `std.option` and `std.cmp` (M-715) follow it. A sub-family MAY use a subdirectory —
`lib/std/collections/vec.myc` with nodule path `std.collections.vec` — once it grows past one nodule.

### D3 — Bootstrap circularity

**No circularity for the stdlib.** The `.myc` stdlib is compiled by the Rust L1 frontend, which is
itself Rust (D1) — the stdlib never compiles the compiler. Staging is linear: *Rust kernel →
compiles `.myc` stdlib*. The only constraint is RFC-0016 **ring layering**: a `.myc` nodule may
`use` only nodules at or below its ring, so there is no `use`-cycle among the ported modules. (A
self-hosted *compiler* would reintroduce circularity — it is out of scope, D1.)

### D4 — Migration order (surface-readiness sequenced) — **the honesty crux**

The order is sequenced by **kernel-prim / language-surface readiness**, not by module taxonomy alone.
Today the surface bottoms out on a *small* prim set — `bit.not`/`bit.xor` (binary) and
`trit.neg/add/sub/mul` (ternary), plus `core.id` — with ADTs, generics, HOF (RFC-0024), traits
(RFC-0019/M-673), and `match`. That set is enough for the *structural/polymorphic* core and nothing
heavier. The migration therefore tiers:

| Tier | Modules | Executable today? | Enabling surface still needed |
|---|---|---|---|
| **Tier 0** | `core`/prelude structure: `Option`/`Result` ADTs + HOF combinators; finite-type `Ordering`/`Eq`/`Ord` (match-defined) | **Yes** — landed: `result.myc` (M-649), `option.myc`, `cmp.myc` (M-715) | — (RFC-0024 HOF already landed) |
| **Tier 1** | `cmp`/`convert` over width types `Binary{N}`/`Ternary{N}`; `math`/`numerics` | **No** | A reduce-to-`Bool` **comparison/equality prim**; **binary arithmetic** prims (only ternary arith + binary `not`/`xor` are surfaced) |
| **Tier 2** | `collections` (Vec/Map/Set); `iter`; `text`/`fmt` | **No** | A **sequence/collection value representation** + indexing; a **string/byte-sequence** value + codepoint ops |

Honest consequence (VR-5): **M-716 (collections), M-717 (text+fmt), and the width-typed part of
M-718 (math) are blocked** on prims/representations that do not yet exist — they are *not* claimed
self-hosted, and their issues carry the explicit precondition. The executable order is: **`core`
first (now)** → the comparison/arithmetic prims land → **`cmp`(width)/`math`** → the sequence/string
representations land → **`collections`/`iter`/`text`/`fmt`**. This supersedes RFC-0016 Q1's
`diag`/`recover`-first suggestion (HOF landing made the structural core the cheapest honest start).

### D5 — Stability bar (and the prototype pattern)

A `.myc` port is **self-hosted** only when, per **exported op**:
1. **≥1 differential test** asserts `.myc` eval ≡ the `mycelium-std-*` Rust reference — *three-way*
   where the op runs to closed L0 (L1-eval ≡ L0-interp ≡ AOT), per the `std_result.rs`/`std_option.rs`/
   `std_cmp.rs` harness (the prototype pattern: `include_str!` the nodule verbatim as the single
   source of truth, append a typed driver pinning generics, assert all three paths + the reference).
2. an **honest per-op guarantee tag** at the honestly-supportable strength — total finite/structural
   ops `Exact`; a generic type-level contract `Declared`; a measured bound `Empirical`; **never
   `Proven`** without a checked theorem (VR-5);
3. a **documented, frozen public signature** (the API-freeze commitment).
Per **module**: the spec's aggregate guarantee matrix is reproduced and *every* exported op is
covered. The bar is **per-op**, not per-module-aggregate (a single uncovered op leaves the module not
self-hosted — G2: no silent gaps).

### D6 — Rust crate retirement

The `mycelium-std-*` Rust crate is **kept as the differential-test reference oracle** (not deleted)
through the migration. Once a `.myc` port clears D5, the Rust crate's public API is marked
`#[deprecated(note = "self-hosted as std.<module>; Rust crate retained as differential oracle")]`.
**Final removal is a separate post-Enactment decision** — a module's Rust reference is the cheapest
honest oracle, so it survives until RFC-0031 is Enacted and the maintainer retires it explicitly.

### D7 — `spore` packaging

**One `spore` per phylum** (`std`), versioned as a unit (ADR-013 deployable artifact; ADR-003
content-addressed). Individual nodules are content-addressed *within* the phylum spore; nodule
versioning rides the phylum version. Rationale: the stdlib is consumed as a whole, so per-nodule
spores would multiply the dependency surface with no isolation benefit at this maturity. (A future
phylum split — e.g. an optional `std-collections` spore — is a packaging decision deferred to when a
consumer needs the smaller surface.)

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
| 2026-06-23 | **Accepted** | M-714: composition model ratified — §5 D1 (irreducible-Rust boundary + criterion), D2 (phylum layout), D3 (no bootstrap circularity), D4 (surface-readiness-tiered migration order; the executable core is Tier-0, `collections`/`text`/`math` blocked on prims — VR-5), D5 (per-op stability bar + the `std_result`/`std_option`/`std_cmp` prototype pattern), D6 (Rust crate kept as differential oracle), D7 (one spore per phylum). M-715 Tier-0 (`option.myc`/`cmp.myc`) lands alongside. |
| 2026-06-23 | **Draft** | Initial stub — open questions enumerated; no normative decisions. Task: E13-1/M-714. |
