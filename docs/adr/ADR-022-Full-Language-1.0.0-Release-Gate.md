# ADR-022 — Full-Language 1.0.0 Release-Readiness Gate (dual-version)

| Field | Value |
|---|---|
| **ADR** | 022 |
| **Status** | **Accepted** (2026-06-23 — maintainer-ratified scope). The *criteria* below for a **full-language** 1.0.0 are agreed; this does **not** declare 1.0.0 reached. `Accepted → Enacted` happens at the tagged full-language 1.0.0 release, once every track's Definition of Done (§5) closes. Changing the *criteria* themselves means superseding this ADR (house rule #3 — append-only). |
| **Decides** | What a **full-language** Mycelium `1.0.0` means — broader than the kernel/core. Establishes a **dual-version model** (a `core` version ⟂ a `lang` version) and the full-language release Definition of Done, partitioned into tracks T1–T9 (each a tracked epic). |
| **Supersedes** | **ADR-021** (1.0.0 Release-Readiness Gate — *kernel/core* scope). ADR-021's kernel Gate A/B is **preserved, not discarded**: it is carried forward here as the **core/kernel 1.0.0 sub-gate** (track T1). The kernel work is incorporated, not invalidated. |
| **Grounds** | ADR-021 (the kernel gate, now T1); ADR-018 (per-crate SemVer policy); DN-19 (Road to 1.0.0 — kernel); DN-25 (Road to Full-Language 1.0.0 — the program map); DN-14 (self-hosting gate); Foundation §6 (roadmap), §2 (SC/KC); CHANGELOG (versioning begins "when the kernel stabilizes"). |
| **Date** | 2026-06-23 |

> **Posture (honesty rule / VR-5).** This ADR records *criteria*, maintainer-ratified. It asserts no
> release. Per-track status is reported at the strength actually evidenced (✅ met / ⏳ open / ◻
> decision-pending) — never upgraded without a checked basis. The "ship full-language 1.0.0" act is a
> later, separate, append-only decision (track T9 / M-738). Nothing here moves any spec to
> `Accepted`/`Enacted`.

---

## 1. Why this ADR exists — the maintainer's clarification (2026-06-23)

ADR-021 §2 scoped the project's `1.0.0` to the **kernel/core** (the verified value-semantics
substrate) and deliberately pushed the concrete surface language, runtime maturity, full stdlib, and
self-hosting to a tracked `1.x`. That framing keeps the *verified substrate* from being held hostage
to surface ratification — and it remains correct **for the core**.

But the maintainer has clarified what a **full-language** 1.0.0 means for the project *writ large*:

> The core can be at 1.0.0, but the full language pack stays **below 1.0.0 — independent of the core
> version** — until it is a fully developed, fully functional language with all critical criteria for
> full usability. A **full stdlib and core libs are part of 1.0.0**. The core components may be written
> in Rust initially and still be 1.0.0 *for the core*; but the project reaches a **full 1.0.0 language**
> only once the **libraries/phyla that extend beyond the bare Rust core are written fully in Mycelium
> (`.myc`, self-hosted) and stable**.

ADR-021 cannot carry this (changing its criteria = supersession, house rule #3), so this ADR
supersedes it — **preserving** the kernel gate as a sub-gate and **adding** the full-language scope.

## 2. The dual-version model (`core` ⟂ `lang`)

Two **independent** SemVer axes (ADR-018 governs per-crate mechanics; this ADR governs the two
*project-level* version lines):

| Axis | What it versions | 1.0.0 governed by | May reach 1.0.0… |
|---|---|---|---|
| **`core`** | the Rust kernel/core (Core IR, interpreter, certified swaps, numerics, VSA/dense, selection, the trusted toolchain) | the inherited ADR-021 Gate A/B — **track T1** here | **first**, while still Rust-backed |
| **`lang`** | the project / language writ large (surface language, runtime, **stdlib + libs in Mycelium**, FFI, tooling, docs, self-hosting) | the full Definition of Done — **all** of §5 | only after the whole gate closes; **stays `< 1.0.0` regardless of `core`'s version** until then |

So `core 1.0.0` and `lang 0.x` coexist honestly: the substrate is stable while the language pack is
still maturing. `lang` never borrows `core`'s version number (G2 — no silent over-claim).

## 3. Scope — full-language 1.0.0 is "fully usable, libs in Mycelium"

In scope for `lang 1.0.0` (each a tracked epic, see §5 + DN-25):
**core sub-gate (T1)**, surface-language completeness (T2), runtime & concurrency execution (T3),
**the standard library written in Mycelium (T4)**, FFI & system interface (T5), native AOT maturity
(T6), toolchain/IDE/distribution (T7), documentation & stability guarantees (T8), and
**self-hosting of everything beyond the bare core (T9)**. T4 + T9 are the defining criteria of the
maintainer's clarification.

Out of scope for `1.0.0` (tracked, but `1.x`/`2.0`): semantic-level projections to other languages
(RFC-0021 exploratory), resonator-only probabilistic pipelines beyond the VSA submodule, and any
"nice to have" not required for *full usability*. Deferrals stay named, never silent.

## 4. Track T1 — the core/kernel 1.0.0 sub-gate (inherited from ADR-021)

ADR-021's **Gate A** (A1 zero-High; A2 Medium ledger; A3 durability — mutants/proptest/fuzz; A4
`just check` green incl. `cargo deny`/`cargo audit`; A5 KC-4 cert-overhead budget) and **Gate B** (B1
RFC-0003/0006/0007 Accepted; B2 KC-2 verdict) are carried forward **verbatim** as the criteria for
`core 1.0.0`. Status as of supersession: A1/A5/B1/B2 ✅ met; A2/A3/A4 ⏳ open. This sub-gate is
**epic E10-1** (issues M-700–M-703). The core may tag `1.0.0` the moment T1 closes — it does **not**
wait for T2–T9.

## 5. Definition of Done — the full-language 1.0.0 gate

`lang 1.0.0` is reached when **every** track's Definition of Done is met. Each track is an epic with
its own per-issue DoDs (DN-25 is the map; the epics carry the detail). Summary criteria:

| Track | Epic | Done when (summary) | Status |
|---|---|---|---|
| **T1 Core sub-gate** | E10-1 | ADR-021 Gate A/B all ✅; `core 1.0.0` tagged; ADR-021 → Enacted | ⏳ A2/A3/A4 open |
| **T2 Surface completeness** | E11-1 (+E7-1/E7-3/E7-5) | full HOF/closures; operator syntax (RFC-0025); committed L3 EBNF grammar (RFC-0030, RFC-0006 Q3/Q8 resolved); generics/traits/effects stable | ⏳ open |
| **T3 Runtime & concurrency** | E12-1 (+E7-2) | real scheduler; full RFC-0008 vocabulary executes; deadlock-freedom checked; memory reclamation (RFC-0027); supervision/cancellation | ⏳ open |
| **T4 Stdlib in Mycelium** | E13-1 | the stdlib + core libs **written in `.myc`** (RFC-0031), differential-tested, stable APIs; Rust std-`*` beyond the bare core superseded by `.myc` | ⏳ open |
| **T5 FFI & system** | E14-1 | capability-based FFI (RFC-0028); `wild` executes; real io/fs/sys bindings; ADR-014 unsafe floor confined + audited | ⏳ open |
| **T6 Native AOT maturity** | E15-1 (+E6-1) | full libMLIR lowering; EXPLAIN-able optimization passes (RFC-0029); JIT; interp ≡ AOT ≡ JIT differential durable | ⏳ open |
| **T7 Toolchain/IDE/dist** | E16-1 (+E9-1) | full LSP (completions/hover/semantic tokens); highlighting shipped (RFC-0026); package publish/resolve; reproducible install | ⏳ open |
| **T8 Docs & stability** | E17-1 | language reference + tutorial; per-module stdlib docs; stability/API-compat guarantees (ADR-023); `lang` SemVer enacted; **MIT-only licensing audited** (§7) | ⏳ open |
| **T9 Self-hosting** | E18-1 | the toolchain + all beyond-core libs are Mycelium; self-hosting CI gate green (DN-14, DN-26); three-way differential (Rust-host ≡ self-host ≡ AOT) | ⏳ open |

**The headline criterion (maintainer):** `lang 1.0.0` is **not** reached while any library/phylum that
*could* be Mycelium is still Rust-only. The bare core may stay Rust (T1); everything above it must be
`.myc`, stable, and fully usable (T4 + T9).

## 6. Sequencing (high level; DN-25 has the dependency graph)

T1 (core gate) and T2 (surface) run first and in parallel; T4 (stdlib in Mycelium) depends on T2;
T3/T5 (runtime/FFI) enable T4's system-touching modules; T6 (AOT) is perf, gated on T1; T7/T8 are
continuous; T9 (self-hosting) is the capstone, depending on T2 + T4. The `core 1.0.0` tag (T1) can
land long before `lang 1.0.0`.

## 7. Conventions this ADR establishes (project-wide, maintainer-set 2026-06-23)

1. **User stories on every epic and issue.** Each epic/issue body carries explicit *user stories*
   (role → capability → benefit) capturing realistic use cases + the problems it must resolve, so work
   is grounded in real usage, not abstraction. (Mirrored into CONTRIBUTING.md / CLAUDE.md.)
2. **Definition of Done on every decision and work item.** Every ADR/RFC/DN and every epic/issue
   carries an explicit, checkable **Definition of Done** — the honest "what must be true to call this
   complete." (This ADR's §5 is itself the gate's DoD.)
3. **MIT-only licensing.** The entire project is **MIT licensed** — no Apache-2.0, no dual-license, on
   any first-party artifact. (The root `LICENSE` + workspace `Cargo.toml` are already MIT; example
   manifests and reference-doc samples are normalized to MIT. The `mycelium-proj` SPDX *parser* still
   recognizes other SPDX ids — accepting an identifier is parser correctness, not a project-license
   claim. The `deny.toml` *dependency* license allow-list is a separate policy — see §8 open question.)

## 8. Open questions

- **Q1 (T9 hard-block?):** Is full self-hosting (T9) a hard 1.0.0 blocker, or may a first `lang 1.0.0`
  ship with a documented, tracked self-hosting remainder (→ `1.1`)? (Recommended: hard-block the
  *beyond-core libs in Mycelium* (T4) but allow the *toolchain* self-host to trail to 1.1 if T4 + the
  surface are stable. Maintainer call.)
- **Q2 (dependency license policy):** `deny.toml` allows MIT/Apache-2.0/BSD/ISC/Unicode **for
  third-party Rust dependencies** (the kernel is Rust). "MIT-only" governs *first-party* code; should
  the *dependency* allow-list also be tightened (would exclude much of the Rust ecosystem)? Flagged for
  the maintainer — left unchanged pending that decision (never silently narrowed).
- **Q3 (lang version start):** Does `lang` versioning start at `0.1.0` now (CHANGELOG currently frames
  one project version)? ADR-018 mechanics + this dual axis need a one-line reconciliation.

## 9. Grounding & honesty

Every track in §5 maps to a real epic (E10-1…E18-1) with grounded child issues; the core sub-gate
(T1) cites ADR-021's measured rows. The "stdlib is still mostly Rust" claim is grounded in the crate
survey (only `lib/std/result.myc` self-hosts today; `crates/mycelium-std-*` are Rust). No tag is
upgraded; no spec is moved to Accepted/Enacted by this ADR. The program map + sequencing live in
DN-25; this ADR is the gate of record.

---

## Changelog
- 2026-06-23 — Created. **Accepted** (maintainer-ratified scope). Supersedes ADR-021 (kernel gate →
  carried forward as track T1). Establishes the dual-version (`core` ⟂ `lang`) model, the full-language
  1.0.0 Definition of Done (T1–T9 → epics E10-1…E18-1), and three project-wide conventions (user
  stories, Definition of Done, MIT-only licensing). Program map: DN-25.
