# ADR-023 — Stability and API Compatibility Guarantees at Full-Language 1.0.0

| Field | Value |
|---|---|
| **ADR** | 023 |
| **Status** | **Draft** (2026-06-23) |
| **Feeds** | E17-1 (Documentation, stability guarantees & 1.0.0 release — M-737) |
| **Decides** | What "stable" means at full-language 1.0.0 — the API-compatibility promise, the dual-version semver enactment (Rust kernel 1.0.0 vs. full-language 1.0.0), the deprecation policy, and the legal-readiness criterion (license). |
| **Grounds** | ADR-018 (per-crate 0.x SemVer, source-only — the policy this ADR enacts for 1.0.0); ADR-021 (1.0.0 kernel/core release gate — the kernel's stability criteria; this ADR extends them to the full language); RFC-0016 §4 (core + stdlib phyla taxonomy); RFC-0017 §4.1 (version metadata in scope headers); Foundation §6 (roadmap — "dual-version model: Rust kernel may reach 1.0.0 first; full language only at stdlib + phyla written in .myc"). |
| **Supersedes / superseded-by** | — (first stability-guarantee ADR; complements ADR-018 which set policy, not the 1.0.0-specific promise). |
| **Date** | 2026-06-23 |

> **Posture (honesty rule / VR-5).** Draft. Nothing is enacted by this stub. All
> criteria below are **open questions** for the maintainer to decide at authoring time.
> Claims about the current state (what the kernel has today, the license) are grounded
> in the actual repo; the "1.0.0" criteria are proposals, not decisions. Append-only.

---

## 1. Problem & goal

ADR-021 defined the release-readiness gate for the **Rust kernel/core** 1.0.0 (Gate A +
Gate B). ADR-018 set the per-crate SemVer policy (0.x until an explicit decision).
Neither document specifies what the **API-compatibility promise** looks like once 1.0.0
is reached, nor what "stable" means for the **full-language** 1.0.0 (the distinct
milestone where the stdlib and phyla beyond the bare Rust core are written fully in
Mycelium `.myc` and are stable + fully usable). This ADR is that specification.

Three forces shape it:

- **Dual-version model.** The Rust kernel/core (`mycelium-core`, `mycelium-l1`,
  `mycelium-interp`) has its own version and may reach 1.0.0 first (ADR-021 gate valid).
  The full-language 1.0.0 requires the stdlib and all libraries/phyla beyond the bare
  Rust core to be **written fully in Mycelium (`.myc`), stable, and fully usable** —
  a higher bar. The two version lines are independent but the full-language 1.0.0
  subsumes the kernel 1.0.0.
- **Never-silent compatibility promise (G2).** A stability guarantee that silently
  excludes large parts of the surface is not a guarantee. Any carve-out must be explicit
  and tracked (e.g. `#[unstable]` or equivalent mechanism).
- **Legal readiness.** The project is **MIT licensed only** — no Apache/no dual-license.
  This is a 1.0.0 legal-readiness criterion: the license must be confirmed correct and
  consistent across all distributed artifacts before the release tag is cut.

## 2. User stories

- As a **language user**, I want a clear, documented stability promise at 1.0.0, so that
  I can write programs against the Mycelium surface without fear of silent breaking changes.
- As a **library/phylum author**, I want to know which APIs are stable vs. explicitly
  unstable (and why), so that I can decide whether to publish my phylum or keep it
  pre-1.0.
- As a **downstream app developer**, I want the deprecation policy explained (timeline,
  migration path, never-silent removal), so that I can plan upgrades without surprises.
- As a **compiler engineer**, I want the dual-version semver model documented (kernel
  version vs. full-language version), so that crate version bumps are unambiguous and
  consistent with ADR-018.
- As a **maintainer**, I want the MIT-only license requirement confirmed as a 1.0.0
  release criterion, so that legal-readiness is a checked gate item, not an afterthought.
- As a **tool author**, I want the API-compatibility scope (surface syntax, Core-IR,
  LSP wire protocol, Rust crate APIs) to be listed explicitly, so that I know what
  "stable" covers for the toolchain I build on top of.

## 3. Scope & decision space

### 3.1 What "stable" means (open)

Candidates for the API-compatibility promise at 1.0.0:

- **Surface language stability:** the `.myc` grammar, keyword set, operator semantics,
  and standard library module paths are stable — a valid program today compiles correctly
  in any future 1.x release.
- **Core-IR stability:** the Core-IR node set (`mycelium-core`), the certificate format
  (`mycelium-cert`), and the interpreter semantics (`mycelium-interp`) are stable — a
  compiled artifact (`.spore`) produced today loads and runs in any 1.x runtime.
- **LSP stability:** the `mycelium-lsp` wire protocol (the subset of LSP 3.17 it uses)
  is stable — editor clients need not update on a 1.x patch bump.
- **Rust crate public API stability:** the `pub` Rust API of kernel crates is stable
  under SemVer (a breaking change requires a major bump, per ADR-018).
- **What is NOT stable (explicit carve-outs):** internals (`pub(crate)`, `#[doc(hidden)]`
  items), the MLIR/LLVM codegen path (performance-path AOT, not the trusted interpreter
  base), experimental features, any surface item marked `unstable` (mechanism TBD).

### 3.2 Dual-version semver enactment (open)

How are the two version lines tracked?

- **Kernel/core 1.0.0** advances per ADR-021; individual crate bumps per ADR-018.
- **Full-language 1.0.0** requires the additional gate: stdlib + phyla in `.myc`, stable,
  usable (E13-1 self-hosting gate, E17-1 release act). What is the version carrier for
  the "full language"? Options: a workspace-level `version` field (rejected by ADR-018 —
  open question whether the full-language version is an exception), a `mycelium`
  top-level crate, or a release manifest only.

### 3.3 Deprecation policy (open)

A deprecation policy that is never-silent (G2):

- Deprecated items are marked (mechanism TBD — a `@deprecated` attribute, a lint, a doc
  note) and the replacement path is stated.
- A deprecation period of at least one minor release (`1.x`) is guaranteed before removal.
- Removal is a major-version event (2.0.0) — never in a patch.
- Deprecated items are never silently removed: they trigger a warning at compile time
  until the removal release.

### 3.4 License (open — legal-readiness criterion)

The project is **MIT licensed only** — no Apache / no dual-license. Before the 1.0.0 tag
is cut, the maintainer must confirm:

- Every `Cargo.toml` `[package].license` field reads `MIT`.
- The `LICENSE` file at the repo root is correct.
- No dependency in `deny.toml` pulls in a GPL/LGPL/incompatible license (checked by
  `cargo deny check licenses`).
- Distributed artifacts (`.spore` packages, VS Code extension, GitHub Linguist) carry
  the MIT SPDX identifier.

This is a **legal-readiness gate criterion at 1.0.0**, not a 1.x follow-up.

### 3.5 Out of scope

- The mechanism for marking individual items unstable (that is a separate RFC/ADR or
  a language feature).
- Compatibility across major versions (2.0.0+) — this ADR governs 1.x only.
- Third-party phyla published by the community (outside this repo's stability promise).

## 4. Definition of Done

*(To be refined at authoring time.)*

- [ ] The API-compatibility scope is explicitly enumerated: what is stable at 1.0.0 and
  what is not, with no silent carve-outs (G2).
- [ ] The dual-version semver model is decided: how the kernel version and the
  full-language version are tracked and related.
- [ ] The deprecation policy is specified: marking mechanism, deprecation period, removal
  policy, never-silent guarantee.
- [ ] MIT-only license confirmed as a 1.0.0 gate criterion; the `cargo deny check
  licenses` gate is green and non-skip-graceful for the release.
- [ ] This ADR moves `Draft → Proposed → Accepted` at maintainer ratification, then
  `Accepted → Enacted` at the actual 1.0.0 tag (never skip Accepted — house rule #3).
- [ ] M-737 (stability & API-compatibility guarantees issue) closed as done.
- [ ] `Doc-Index.md` and `CHANGELOG.md` updated (orchestrator-owned, not here).

## 5. Open questions for the maintainer

1. **Stability scope confirmation:** which of the four layers (surface syntax, Core-IR,
   LSP wire, Rust crate API) are covered by the 1.0.0 stability promise, and which are
   explicitly out?
2. **Full-language version carrier:** is there a `mycelium` top-level crate whose version
   represents the full language, or is the full-language 1.0.0 a release-manifest-only
   concept (i.e. not a SemVer crate)?
3. **Unstable mechanism:** is there a surface-language `@unstable` attribute, a Rust
   `#[doc(hidden)]` convention, or a separate tracking file for pre-stable items?
4. **Deprecation period:** is one minor release the right minimum, or should it be a
   calendar period (e.g. six months)?
5. **License sweep:** should the `cargo deny check licenses` gate be added to
   `just check` now (pre-1.0) or only as part of the 1.0.0 release act?
6. **Dual-version tracking:** if the kernel reaches 1.0.0 before the full language, what
   label or tag distinguishes the kernel release from the full-language release in the
   CHANGELOG and release notes?

## 6. Grounding & honesty

- **ADR-018** (`Enacted` 2026-06-23): per-crate 0.x SemVer, source-only. This ADR
  **extends** that policy to the 1.0.0 case; it does not supersede it.
- **ADR-021** (`Accepted` 2026-06-21): the kernel/core 1.0.0 gate (Gate A + B). This
  ADR adds the full-language layer on top of that gate.
- **MIT-only license claim:** grounded in the repo's `LICENSE` file and the maintainer
  declaration. Confirmed by `cargo deny check licenses` (if the gate is wired and green —
  currently skip-graceful, ADR-021 Gate A4 / M-652).
- **Guarantee tags:** all claims in §3 are `Declared` until enacted; no `Proven` tag is
  warranted for policy (VR-5).
- **Append-only:** status transitions follow `Draft → Proposed → Accepted → Enacted`.
  Changing the decided criteria requires superseding this ADR (house rule #3).

## 7. Changelog

- **2026-06-23 — Draft.** Stub created for the E17-1 / M-737 stability & API-compatibility
  decision at full-language 1.0.0. Grounds in ADR-018 (versioning) and ADR-021 (kernel gate);
  records MIT-only license as a 1.0.0 legal-readiness criterion. All normative choices
  (scope, dual-version model, deprecation, license sweep gate) deferred to authoring.
  Append-only.
