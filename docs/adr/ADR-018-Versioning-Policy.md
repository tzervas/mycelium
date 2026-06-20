# ADR-018 — Versioning policy: per-crate 0.x SemVer, source-only (no crates.io in the design phase)

| Field | Value |
|---|---|
| **ADR** | 018 |
| **Title** | The workspace's versioning policy: **per-crate `0.x` SemVer**, **source-only** distribution (no crates.io publish during the design phase), and the `CHANGELOG.md` `[Unreleased]` → release-cut mapping |
| **Status** | **Accepted** (2026-06-20) |
| **Date** | 2026-06-20 |
| **Depends on** | ADR-007 (Rust kernel + reference interpreter; MSRV 1.92 — Foundation §8); the squash-only / linear-bisectable-history discipline (CLAUDE.md "Commits & PRs"); RFC-0017 §4.1 (version metadata lives in scope headers / the project manifest, not the term grammar) |
| **Supersedes / Superseded by** | — (first versioning policy; nothing prior to supersede) |
| **Implemented by** | M-384 (release-plz config + the manual-dispatch dry-run workflow) |

## Context

The workspace is **many independent crates** — 44 `mycelium-*` library/tool crates plus `xtask`,
each currently at version `0.0.0`. They mature at **different cadences**: the kernel
(`mycelium-core`, `mycelium-l1`, `mycelium-interp`) is the trusted base (ADR-007) and moves
conservatively; the stdlib capability crates (`mycelium-std-*`) and the toolchain CLIs
(`mycelium-fmt`/`-check`/`-lint`/`-sec`) land and change far more often. There is **no prior
versioning policy** in the corpus: RFC-0017 §4.1 reserves a `// @version:` header slot and shows
`1.2.0` *illustratively*, but it defines **where** a version lives (scope metadata), not **how** the
numbers are assigned or **whether** anything is published. This ADR decides that.

Three forces shape the decision:

- **Independent lifecycles (KC-3, the small-auditable-kernel mandate).** A single workspace-wide
  version would force a churning leaf crate to drag the stable kernel's number forward (or vice
  versa), making the version meaningless as a stability signal and coupling crates that the
  architecture deliberately keeps decoupled.
- **The project is in the design + Rust-first phase.** The corpus in `docs/` is the product
  (`CHANGELOG.md` preamble); the code is landing but **not yet a consumable artifact** with a
  stability promise. Every crate already declares `publish = false` in its manifest — the intent to
  *not* distribute on crates.io is already encoded, just not yet recorded as a decision.
- **Linear, bisectable history (the squash-only discipline, CLAUDE.md).** Every PR lands on `main`
  as one curated squash commit, and Conventional-Commit subjects are required. That gives a clean,
  machine-readable commit stream — exactly what a Conventional-Commits → SemVer-bump tool consumes —
  so a *preview* of version bumps is cheap and honest to derive even before any release happens.

## Decision

**1. Per-crate `0.x` SemVer.** Each crate is versioned **independently** under Semantic Versioning
2.0.0, starting in the `0.y.z` series (pre-1.0). In `0.y.z`, **`y` is the breaking-change axis and
`z` is the additive/fix axis** — the documented SemVer convention for pre-1.0 crates that the Rust
ecosystem and `cargo`'s own resolver treat as the rule (a `0.y` bump is a breaking release; a `0.y.z`
bump is compatible). No crate advances to `1.0.0` without an explicit, separate decision (`1.0` is a
stability promise this project is **not** ready to make — see the gate below). The version is the
single source of truth in each `Cargo.toml`'s `[package].version`; per RFC-0017 §4.1 a published
scope additionally surfaces it in its header/manifest, but the manifest version is authoritative.

*Workspace-wide versioning is rejected* (alternatives, below): it would couple independent
lifecycles and make the number a poor stability signal.

**2. Source-only distribution — no crates.io publish during the design phase.** Crates are consumed
**from source** (path/git dependencies within the workspace; a git tag for any external consumer),
**not** from crates.io. This is already encoded as `publish = false` on every crate, and this ADR
**records it as the policy**, not an accident: a registry publish is a durable public commitment
(a yanked-but-not-deletable version, a name reservation, an implied support surface) that is premature
while the API is pre-`0.1` and changing. Publishing becomes a *future* decision gated on the
capability bar below; until then the release tooling **previews** versions and changelogs but
**publishes nothing** (`release-plz` runs with `publish = false` / no registry push — M-384).

**3. `CHANGELOG.md` `[Unreleased]` → release-cut mapping.** The top-level `CHANGELOG.md` follows Keep
a Changelog 1.1.0 with a single live **`## [Unreleased]`** section holding dated
`### Added/Changed/Fixed …` subsections (the current format). A **release cut** for a crate (or a
coordinated set) is the act of:
  1. renaming the relevant `[Unreleased]` content to a version heading
     **`## [<crate> <x.y.z>] — <ISO-date>`** (or `## [<x.y.z>] — <ISO-date>` for a workspace-coordinated
     cut), leaving a fresh empty `[Unreleased]` above it;
  2. setting that crate's `Cargo.toml` `[package].version`; and
  3. tagging the squash commit on `main` (`<crate>-v<x.y.z>`, or `v<x.y.z>` for a coordinated cut).

The bump is **derived from the Conventional-Commit history** since the crate's last tag (`feat:` ⇒
minor-axis, `fix:` ⇒ patch-axis, a `!`/`BREAKING CHANGE` ⇒ the `0.y` breaking axis), which is exactly
what the M-384 dry-run previews. Because history is squash-only and linear, the commit range per crate
is unambiguous.

**4. Honest release-capability gate — not now.** A *real* release (a tagged, distributed version that
anyone may depend on) must clear a capability bar this project has **not** yet reached. The bar, stated
honestly so it is not mistaken for a present capability:
- the kernel API is past `0.1` and the `cargo public-api` baselines (`docs/spec/api/`) are stable
  enough that breaking changes are deliberate and reviewed, **not** routine;
- the MVP execution path exists end-to-end (the native/interpreter differential, NFR-7) so a released
  artifact actually *does* something stable;
- if/when crates.io publishing is turned on, that flip is its **own** superseding decision (it
  changes `publish` and the workflow's push behaviour — never a silent config edit).

Until the bar is met, versioning is **internal bookkeeping + a dry-run preview**: numbers and
changelogs are maintained and previewed; **nothing is tagged-as-released or published automatically.**

## Consequences

- **Each crate's version means what it says.** A stable kernel can sit at `0.3.x` while a churning
  tool crate races ahead at `0.12.x`; neither distorts the other. The number is an honest
  per-crate stability signal (G2 — no hidden coupling).
- **No premature public commitment.** Nothing on crates.io means no name-squatting, no un-deletable
  bad version, no implied support contract while the API is still pre-`0.1`. Distribution turns on
  only when the capability gate is cleared, by a deliberate superseding decision.
- **The dry-run is genuinely useful now.** The squash-only Conventional-Commit history lets M-384
  preview *correct* per-crate bumps + changelog fragments on every manual dispatch — a rehearsal of
  the release that costs nothing and publishes nothing, matching the repo's "no automatic remote CI"
  posture.
- **A coordination cost for cross-crate breaking changes.** Per-crate versions mean a change that
  ripples across several crates is several coordinated bumps, not one. This is the accepted price of
  decoupled lifecycles; the dry-run surfaces the full set so the coordination is visible, not missed.
- **`0.y` SemVer is a convention, not a `cargo`-enforced invariant.** `cargo` treats `0.y.z`
  compatibility by the documented rule, but SemVer correctness of any given bump still rests on the
  `cargo public-api` baseline diff being reviewed (additions-only unless a breaking `0.y` bump is
  intended). The policy is **Declared** (asserted process), with the baseline gate as its check — it
  is not a proven property.

## Alternatives considered

- **Workspace-wide single version (one number for all crates).** Simpler to reason about and to tag,
  but it **couples independent lifecycles** (KC-3 keeps crates decoupled by design): a leaf crate's
  churn would force-bump the stable kernel, draining the version of meaning. Rejected — the decoupling
  the architecture buys would be thrown away at the version layer.
- **Publish to crates.io now (per-crate `0.x` on the registry).** Rejected as **premature**: a
  registry publish is a durable public commitment while the API is pre-`0.1` and changing weekly;
  `publish = false` already encodes the intent, and source/git consumption covers every present need.
  Revisit via a superseding decision once the capability gate is met.
- **Go straight to `1.0.0` per crate.** Rejected: `1.0` is a stability promise (SemVer's whole point);
  making it now would be dishonest (VR-5 in spirit — claiming a stability guarantee we cannot back),
  exactly the kind of upgrade-without-basis the honesty rule forbids.
- **No policy / ad-hoc versions.** Rejected: leaves the `0.0.0` placeholders and the dry-run tooling
  (M-384) without a rule to apply, and lets versions drift ungrounded — the opposite of "ground every
  claim".

## Grounding

ADR-007 (the Rust kernel + reference interpreter this versions; MSRV 1.92 — Foundation §8); the
**squash-only, linear-bisectable history + Conventional-Commit** discipline (CLAUDE.md "Commits &
PRs") that makes per-crate bump derivation unambiguous; RFC-0017 §4.1 (version metadata lives in scope
headers / the project manifest, not the term grammar — this ADR assigns the numbers RFC-0017 only
*locates*); the universal `publish = false` already in every crate `Cargo.toml` (the source-only intent
this ADR records). House rules: **KC-3** (small, decoupled, auditable kernel — the case for per-crate
over workspace-wide versions); **G2 / "no black boxes"** (a version is an honest, non-hidden
per-crate stability signal; turning on publishing is never a silent config flip); **VR-5 / the honesty
rule** (no `1.0` stability claim without a checked basis — downgrade to `0.x` to stay honest); the
"design + Rust-first" framing of `CHANGELOG.md` (versioning is bookkeeping until the capability bar is
met). The release-capability gate is **Declared** (an asserted, flagged process bar), with the
`cargo public-api` baseline diff (`docs/spec/api/`) and the NFR-7 differential as its checks — not a
proven property.

## Meta — changelog

- **2026-06-20 — Accepted.** First versioning policy for the workspace. Decides **per-crate `0.x`
  SemVer** (independent lifecycles, `0.y` = breaking / `0.y.z` = compatible per the documented Rust
  convention) over a workspace-wide number (KC-3 — keep decoupled crates decoupled at the version
  layer); **source-only distribution** with **no crates.io publish in the design phase** (records the
  existing `publish = false`; a registry flip is a future superseding decision gated on the capability
  bar); and the **`CHANGELOG.md` `[Unreleased]` → release-cut mapping** (rename to
  `## [<crate> <x.y.z>] — <date>`, set `Cargo.toml` version, tag the squash commit; the bump derived
  from Conventional-Commit history — exactly what M-384 previews). States an honest
  **release-capability gate** (Declared) — versioning is internal bookkeeping + a dry-run preview until
  the API stabilises past `0.1` and the MVP execution path exists; nothing is published automatically.
  Implemented by M-384 (release-plz dry-run, manual-dispatch-only). Append-only — to change this
  policy, supersede this ADR.
