# ADR-023 — Stability and API Compatibility Guarantees at Full-Language 1.0.0

| Field | Value |
|---|---|
| **ADR** | 023 |
| **Status** | **Accepted** (2026-06-23 — maintainer-ratified; was **Proposed** 2026-06-23, was **Draft** 2026-06-23). The stability scope, dual-version model, deprecation policy, and MIT-only license gate below are **decided** (the §5 questions are resolved). This does **not** declare 1.0.0 reached, nor enact the policy: `Accepted → Enacted` happens at the tagged full-language 1.0.0 release (M-738), once ADR-022's gate closes. Changing a decided criterion means superseding this ADR (house rule #3 — append-only). |
| **Feeds** | E17-1 (Documentation, stability guarantees & 1.0.0 release — M-737) |
| **Decides** | What "stable" means at full-language 1.0.0 — the API-compatibility promise, the dual-version semver enactment (Rust kernel 1.0.0 vs. full-language 1.0.0), the deprecation policy, and the legal-readiness criterion (license). |
| **Grounds** | ADR-018 (per-crate 0.x SemVer, source-only — the policy this ADR enacts for 1.0.0); ADR-021 (1.0.0 kernel/core release gate — the kernel's stability criteria; this ADR extends them to the full language); RFC-0016 §4 (core + stdlib phyla taxonomy); RFC-0017 §4.1 (version metadata in scope headers); Foundation §6 (roadmap — "dual-version model: Rust kernel may reach 1.0.0 first; full language only at stdlib + phyla written in .myc"). |
| **Supersedes / superseded-by** | — (first stability-guarantee ADR; complements ADR-018 which set policy, not the 1.0.0-specific promise). |
| **Date** | 2026-06-23 |

> **Posture (honesty rule / VR-5).** Accepted (criteria ratified), **not Enacted** — nothing
> ships 1.0.0 by this ADR. The §3 criteria are now **decisions** (the §5 open questions are
> resolved in §5'); every policy claim is **`Declared`** (a stated contract, not a proven or
> empirically-measured property — VR-5: policy warrants no `Proven` tag). Claims about the
> *current* repo state (the kernel's gate status, the `LICENSE` file, the license sweep) are
> grounded in the actual repo and dated. `Accepted → Enacted` is M-738's release act, gated on
> ADR-022. Append-only: a decided criterion changes only by a superseding ADR (house rule #3).

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

### 3.1 What "stable" means (decided — Q1)

**Decision (`Declared`).** The 1.0.0 stability promise covers **all four layers below**, each
with the explicit carve-outs named — *a valid program / artifact / client at 1.0.0 keeps
working across every `1.x` release*. No layer is silently excluded (G2). The promise is:

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
  items), the MLIR/LLVM codegen path (performance-path AOT — its *observable results* match the
  trusted interpreter, but its internal IR/ABI is free to change), experimental features, and the
  **reserved-not-active surface keywords** (`fuse`/`mesh`/`graft`/`cyst`/`xloc`/`forage`/
  `backbone`/`tier`/`reclaim` — DN-03 §4; lexed but no production consumes them, so they are
  *reserved*, never *stable API*, until their constructs land in a later `1.x` or `2.0`).

> **Why all four (not a subset).** A stability promise that covered, say, only the surface
> grammar but not the `.spore` artifact format would let a 1.x patch silently break a deployed
> spore — exactly the never-silent failure G2 forbids. The four layers are the four contracts a
> downstream actor (program author, deployer, editor-client author, Rust embedder) actually
> depends on, so each is named and promised explicitly. The carve-outs are the surfaces that are
> *honestly* not yet stable; they are listed, not omitted.

### 3.2 Dual-version semver enactment (decided — Q2, Q6)

**Decision (`Declared`).** The two version lines stay **independent**, and the full-language
version is a **release-event concept, not a SemVer crate** — there is **no** workspace-level
`version` field and **no** `mycelium` top-level crate version (ADR-018 is upheld, not excepted).

- **Kernel/core 1.0.0** advances per ADR-021 (now ADR-022 track **T1**); individual Rust crate
  bumps per ADR-018 (each crate its own per-crate SemVer). The kernel may tag **`core 1.0.0`**
  first — this is a per-crate set of SemVer tags on the trusted base, witnessed by ADR-021's
  Gate A/B (carried into ADR-022 §T1).
- **Full-language 1.0.0** is carried by **(a)** the annotated git tag **`v1.0.0`**, **(b)** the
  `CHANGELOG.md [1.0.0]` section, and **(c)** the ADR-022 gate record moved `Accepted → Enacted`.
  It is **not** a crate version. Rationale: ADR-018 deliberately rejects a workspace `version`
  (identity is per-crate / content-addressed, ADR-003); the full-language milestone is a
  *release event* gated by ADR-022 (E13-1 self-hosting + E18-1 readiness), so a tag + changelog +
  gate-record is the honest carrier — adding a phantom crate version would imply a SemVer
  contract on an aggregate that ADR-018 says does not exist.
- **Distinguishing the two in CHANGELOG / release notes (Q6).** Each release line is **labelled
  explicitly**: a kernel-first tag is written `core 1.0.0` (the per-crate kernel set; ADR-022 T1)
  and the full-language tag is `1.0.0` / `v1.0.0` (ADR-022 whole-gate). The `CHANGELOG.md [1.0.0]`
  section is the **full-language** release and states, in its first line, that it subsumes the
  `core 1.0.0` sub-gate. Release notes name which milestone the tag represents — never an
  unlabelled "1.0.0" that conflates the two (G2).

### 3.3 Deprecation policy (decided — Q3, Q4)

**Decision (`Declared`).** Deprecation is **release-based, never calendar-based, and never
silent** (G2):

- **Marking mechanism.** A deprecated **surface** item carries a `// @deprecated: <replacement>`
  scope/item header note (the same header-comment channel as `@matured`/`@version`,
  Nodule-Header spec §3) and the checker emits a **compile-time warning** naming the replacement
  path. A deprecated **Rust** API item uses `#[deprecated(note = "…")]` (the standard rustc
  warning). Either way the replacement is stated — a deprecation with no migration path is not
  permitted.
- **Period (Q4): at least one full minor release (`1.x`), release-based not calendar-based.** A
  design-phase project with no fixed cadence cannot honestly promise "six months"; "≥ one minor
  release with an active warning" is a *checkable* promise (the item must ship deprecated-with-
  warning in at least one `1.x` before it may be removed). This is the more honest bound (VR-5).
- **Removal is a `2.0.0`-only event** — never in a `1.x` minor or patch. Removing a stable item
  inside `1.x` would break the §3.1 promise.
- **Never-silent (G2).** From the release it is deprecated until the release it is removed, the
  item compiles **with a warning** every time — it is never silently dropped and never silently
  retained-but-broken.

#### 3.3.1 The unstable-marking mechanism (Q3)

**Decision (`Declared`).** At 1.0.0 there is **no surface `@unstable` attribute** — it is
deferred (§3.5 keeps the *mechanism* out of scope as future language/RFC work). Instead:

- **Rust layer:** non-stable items are `pub(crate)` or `#[doc(hidden)]` (the existing convention);
  only the documented `pub` API is promised.
- **Surface layer:** everything in the **ratified grammar** (`docs/spec/grammar/mycelium.ebnf`)
  and the **25 ratified stdlib specs** is stable-by-default at 1.0.0. The only surface items that
  are *not* stable are the **reserved-not-active keywords** (§3.1) — and those are not stable
  *because they are not yet language at all*, which the grammar and DN-03 §4 state explicitly.
  So the set of "stable surface" and "explicitly-not-stable surface" is fully enumerated with no
  silent middle ground (G2). When a surface `@unstable` mechanism is later wanted (to ship a new
  construct as preview inside `1.x`), it is a separate RFC; until then, new surface lands only
  when it is stable, or stays behind the reserved-keyword wall.

### 3.4 License (decided — legal-readiness criterion; Q5)

The project is **MIT licensed only** — no Apache / no dual-license (CONTRIBUTING §Licensing;
house rule #6). This is a **legal-readiness gate criterion at 1.0.0**, not a `1.x` follow-up.
The 1.0.0 release act (M-738) confirms, each as a checked item:

- Every first-party `Cargo.toml` `[package].license` field reads `MIT`.
- Every first-party **shipped** `.myc` nodule's `@license` header reads `MIT` (stdlib under
  `lib/std/**` + the `examples/**` programs). **Test fixtures are excepted** — fixtures under
  `crates/**/tests/` deliberately carry other license strings (Apache / a deliberately-invalid
  SPDX id) to exercise license-field parsing and non-inheritance; they are inputs, not shipped
  artifacts (see the sweep note below).
- The `LICENSE` file at the repo root is the MIT text (✅ confirmed 2026-06-23 — root `LICENSE`
  is `MIT License`, © 2026 Tyler Zervas).
- No **dependency** pulls in a GPL/LGPL/incompatible license, checked by `cargo deny check
  licenses` against `deny.toml`. *Note:* `deny.toml`'s `allow` list permitting `Apache-2.0` /
  BSD / ISC / Unicode is **correct and not a violation** — those govern *third-party
  dependencies* (commonly Apache-or-MIT), which MIT-licensed first-party code may consume. The
  MIT-only rule constrains **first-party** `[package].license` / `@license`, a distinct axis.
- Distributed artifacts (`.spore` packages, VS Code extension, GitHub Linguist) carry the MIT
  SPDX identifier.

**Sweep status (2026-06-23, this ADR's authoring — M-737).** A **repo-wide** sweep of every
first-party **shipped** `.myc` header (`grep -rn '@license' --include='*.myc'`) found **six**
non-MIT violations, all now corrected to `MIT`: `lib/std/result.myc` (the self-hosted stdlib
nodule) and the five `examples/**` programs (`examples/repr-tour/{ambient,swaps,traits,iter}.myc`
and `examples/hello-phylum/hello.myc`). The **only** remaining non-MIT `@license` strings are
**deliberate test fixtures** under `crates/mycelium-proj/tests/fixtures/` —
`crates/mycelium-proj/tests/conformance.rs` (and the `mycelium-proj` source) *assert*
`root.myc`/`mycelium-proj.toml` carry `Apache-2.0` and `bad-header.myc` carries a
deliberately-invalid SPDX id, to prove a locally-declared license is parsed and **not silently
inherited/overridden** (origin tracking) and that a bad id is an explicit error. They are test
*inputs*, not shipped artifacts, so they stay as-is (changing them would defeat the tests and is
outside the first-party shipped scope).

**Gate timing (Q5).** The supply-chain gate already exists — `just deny`
(`scripts/checks/deny.sh`) runs `cargo deny check` (advisories + **licenses** + sources) and is
invoked by the shared check flow (`scripts/checks/all.sh`, so both `just check` and
`just check-full` run it). Its honesty is **environment-tiered today** (ADR-021 Gate A4 / M-652):
*skip-graceful* in local dev when `cargo-deny` is absent, but a *hard failure* (no silent
skip-pass — G2) under `CI=true` or `MYCELIUM_REQUIRE_SUPPLY_CHAIN=1`. **Decision:** no new wiring
is needed pre-1.0 — the existing gate is correct. The M-738 release act simply **runs it in the
strict mode** (`MYCELIUM_REQUIRE_SUPPLY_CHAIN=1`, the durability `just check-full` posture) so the
license check is *green and actually present* — never skip-graceful — when the `v1.0.0` tag is
cut. Adding a *non-graceful* `cargo-deny` to every fast `just check` (Tier-1) commit is rejected:
it would cost the per-commit speed DN-20 protects, and the license axis only needs to be
provably-green *at release*, which the strict release run guarantees.

### 3.5 Out of scope

- The mechanism for marking individual items unstable (that is a separate RFC/ADR or
  a language feature).
- Compatibility across major versions (2.0.0+) — this ADR governs 1.x only.
- Third-party phyla published by the community (outside this repo's stability promise).

## 4. Definition of Done

- [x] The API-compatibility scope is explicitly enumerated (§3.1): the four stable layers
  (surface syntax, Core-IR/cert/interp, LSP wire, Rust crate API) and the explicit carve-outs
  (internals, MLIR/LLVM codegen internals, reserved-not-active keywords) — no silent carve-outs (G2).
- [x] The dual-version semver model is decided (§3.2): kernel `core 1.0.0` per ADR-018 per-crate
  SemVer; full-language `1.0.0` as a release-event (git tag + CHANGELOG + ADR-022 gate record), no
  workspace `version`, no `mycelium` top-level crate. Labelled distinctly in CHANGELOG/notes (Q6).
- [x] The deprecation policy is specified (§3.3): header/`#[deprecated]` marking with a stated
  replacement, ≥ one minor (`1.x`) release-based period, removal only at `2.0.0`, never-silent
  compile-time warning throughout.
- [x] MIT-only license confirmed as a 1.0.0 gate criterion (§3.4): root `LICENSE` is MIT;
  `lib/std/result.myc` Apache→MIT corrected; the `just deny` license check is hard-fail under the
  strict release posture (`MYCELIUM_REQUIRE_SUPPLY_CHAIN=1`) the M-738 act runs — green and present,
  never skip-graceful.
- [x] This ADR moved `Draft → Proposed → Accepted` (this authoring, M-737). `Accepted → Enacted`
  is reserved for the actual 1.0.0 tag act (M-738) — never skip Accepted (house rule #3).
- [ ] M-737 (this issue) marked done in `issues.yaml` (orchestrator step).
- [ ] `Doc-Index.md` and `CHANGELOG.md` updated (orchestrator-owned, not here).

## 5. Open questions for the maintainer

> **Resolved (2026-06-23, M-737 authoring).** All six are decided in §3; recorded here for the
> append-only trail. Reopening a decision means superseding this ADR (house rule #3).

1. **Stability scope confirmation** → **resolved (§3.1):** *all four* layers (surface syntax,
   Core-IR/cert/interp, LSP wire, Rust crate API) are in-scope, each with explicit carve-outs;
   the reserved-not-active keywords and the codegen internals are explicitly out.
2. **Full-language version carrier** → **resolved (§3.2):** no `mycelium` top-level crate, no
   workspace `version` — the full-language 1.0.0 is a *release-event* (annotated `v1.0.0` tag +
   `CHANGELOG [1.0.0]` + ADR-022 gate record). ADR-018's per-crate model is upheld, not excepted.
3. **Unstable mechanism** → **resolved (§3.3.1):** no surface `@unstable` at 1.0.0 (deferred to a
   later RFC); Rust uses `pub(crate)`/`#[doc(hidden)]`; surface is stable-by-default with the
   reserved-keyword wall as the only explicitly-not-stable surface (fully enumerated, G2).
4. **Deprecation period** → **resolved (§3.3):** ≥ one minor (`1.x`) *release-based* period (not
   calendar), with an active compile-time warning throughout; removal at `2.0.0` only.
5. **License sweep gate timing** → **resolved (§3.4):** no new pre-1.0 wiring — the existing
   `just deny` check is run in *strict* mode (non-skip-graceful) as a hard row of the M-738
   release act; not added non-gracefully to the fast `just check` (DN-20 speed).
6. **Dual-version tracking** → **resolved (§3.2):** label explicitly — `core 1.0.0` (kernel,
   per-crate; ADR-022 T1) vs `1.0.0`/`v1.0.0` (full language; ADR-022 whole gate); the
   `CHANGELOG [1.0.0]` section is the full-language release and states it subsumes `core 1.0.0`.

## 6. Grounding & honesty

- **ADR-018** (`Enacted` 2026-06-23): per-crate 0.x SemVer, source-only. This ADR
  **extends** that policy to the 1.0.0 case; it does not supersede it.
- **ADR-021** (`Superseded by ADR-022` 2026-06-23; was `Accepted` 2026-06-21): the kernel/core
  1.0.0 gate (Gate A + B), now carried forward as ADR-022's **T1 core sub-gate**. This ADR adds
  the full-language stability layer on top of that gate.
- **ADR-022** (`Accepted` 2026-06-23): the full-language 1.0.0 release-readiness gate. This ADR
  is the *stability promise* whose enactment (M-738) is gated on ADR-022 closing.
- **MIT-only license claim:** grounded in the repo's root `LICENSE` file (MIT, confirmed
  2026-06-23) and house rule #6. Enforced by `just deny`'s `cargo deny check` — skip-graceful in
  local dev, hard-fail under the strict release posture (`CI=true` / `MYCELIUM_REQUIRE_SUPPLY_CHAIN=1`;
  ADR-021 Gate A4 / M-652). The one first-party violation found at authoring (`lib/std/result.myc`
  Apache→MIT) is fixed in this change.
- **Guarantee tags:** all claims in §3 are `Declared` until enacted; no `Proven` tag is
  warranted for policy (VR-5).
- **Append-only:** status transitions follow `Draft → Proposed → Accepted → Enacted`.
  Changing the decided criteria requires superseding this ADR (house rule #3).

## 7. Changelog

- **2026-06-23 — Accepted (M-737).** Authored: all six §5 open questions resolved in §3 and the
  status moved `Draft → Proposed → Accepted` in one ratification pass (the criteria are decided;
  not yet Enacted — that is M-738 at the `v1.0.0` tag). Decisions: §3.1 four-layer stability scope
  with explicit carve-outs; §3.2 dual-version model (full-language 1.0.0 = a release-event, not a
  crate version; ADR-018 upheld) + distinct CHANGELOG labelling; §3.3 release-based never-silent
  deprecation policy + §3.3.1 no-surface-`@unstable`-at-1.0 convention; §3.4 MIT-only legal gate —
  one first-party violation (`lib/std/result.myc` Apache→MIT) corrected, `just deny` strict-mode
  timing decided (Q5). All §3 claims `Declared` (policy warrants no `Proven` — VR-5). Append-only.
- **2026-06-23 — Draft.** Stub created for the E17-1 / M-737 stability & API-compatibility
  decision at full-language 1.0.0. Grounds in ADR-018 (versioning) and ADR-021 (kernel gate);
  records MIT-only license as a 1.0.0 legal-readiness criterion. All normative choices
  (scope, dual-version model, deprecation, license sweep gate) deferred to authoring.
  Append-only.
