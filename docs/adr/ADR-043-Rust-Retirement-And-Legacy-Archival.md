# ADR-043 — Rust Retirement & Legacy Archival

| Field | Value |
|---|---|
| **ADR** | 043 |
| **Status** | **Accepted** (2026-07-07 — maintainer directive, in force going forward). Records how a Rust crate/lib is **retired** once its Mycelium (`.myc`) port is **fully validated**: the retire-when-proven gate, the dedicated **archive branch** that keeps the full legacy Rust for posterity + audit, the per-crate housekeeping that follows removal, and the end-state this converges to. **Enacts nothing** — retirement happens **per-crate**, as individual ports clear the gate; `Accepted → Enacted` only when the whole migration is complete and the archive branch holds the full legacy (house rule #3 — must step through Accepted first, same posture as ADR-042). |
| **Decides** | (1) **Retire-when-proven gate.** A Rust crate/lib is retired **only after** its `.myc` port is fully validated: the **myc-dogfood dual witness** (M-989 — the Rust `cargo test` differential **and** native `myc check`/`mycfmt`/`myc-lint` green), the **two-stage runtime bar** (DN-26 §9: interpreter parity first, AOT confirmation second where applicable), and the crate's guarantee tags/behaviors preserved (no downgrade without a checked basis). No retirement on a partial/unproven port — the Rust stays the live oracle until the `.myc` earns replacement. (2) **Archive branch — never lose the Rust.** Retired Rust is preserved on a dedicated, **protected, persistent archive branch** (recommend `archive/rust-legacy`, **FLAGged** for the maintainer to confirm) holding the **full legacy Rust codebase**, not tombstoned files — for posterity, audit, and as a re-derivable differential oracle even after retirement. Never deleted, never force-pushed, never squashed away; joins the protected-branch set (`MYC_PROTECTED_BRANCHES`, CLAUDE.md mitigation #10). (3) **Remove from the active tree + housekeep.** Once archived, the Rust is removed from the active workspace (single-language tree, no dual-maintenance/confusion). Housekeeping is **incremental, per retirement** — not a big-bang sweep: workspace `Cargo.toml` members, the check gates, `docs/api-index/`, `docs/Doc-Index.md`, the acyclic-deps invariant, MSRV/toolchain refs. (4) **End state.** All non-Mycelium first-party code archived and replaced; the active tree is pure Mycelium; the archive branch holds the complete Rust legacy. |
| **Relates / hardens** | **Operationalizes ADR-042** — ADR-042 §2.1(b)/§3 sets the zero-foreign-first-party-languages **end-state** and is explicit that it "asserts no implementation progress: no Rust is deleted" by that ADR alone; this ADR is the **retirement mechanism** ADR-042 left unspecified — how a crate actually leaves the active tree once its port earns it, without losing the Rust. Does **not** amend ADR-042 (append-only); ADR-042's DN-39 boundary/language distinction (§2.2 — the trusted *component set* is unchanged, only its *implementation language* moves) is unaffected by retirement of **non-kernel** crates, and the kernel's own eventual retirement remains gated by ADR-042 §3 OQ-1 (the bootstrap/trust story), not decided here. **Grounds the "proven" gate in M-989** (`scripts/checks/myc-dogfood.sh`, the native-toolchain dual witness) and **DN-26 §9** (the interpreted-first, AOT-second Stage-6 resolution — the two-stage runtime bar this ADR generalizes to every crate's retirement, not just the compiler self-hosting stages). **Extends CLAUDE.md's tiered/protected-branch model** (mitigation #10, `MYC_PROTECTED_BRANCHES`, `scripts/checks/branch-guard.sh`) with a new persistent protected branch. Does not amend RFC-0016 §4.6, ADR-038, or ADR-041. |
| **Grounds** | Maintainer directive (2026-07-07, this session — recorded verbatim in intent; binds as Accepted); **ADR-042** (the freeze/full-self-hosting policy this operationalizes, §2.1(b)/§2.2/§3); **DN-26** (§9 flag-2 resolution — the interpreted-first/AOT-second Stage-6 bar; §7.3 Stage-5/6 differential discipline); **`scripts/checks/myc-dogfood.sh`** (M-989 — the dual-witness gate this ADR names as the "proven" criterion); **CLAUDE.md** (`dev → integration → main` tiers; mitigation #10 branch-guard + `MYC_PROTECTED_BRANCHES`; `scripts/checks/branch-guard.sh`); **`scripts/checks/deps-acyclic.sh`** (the acyclic-deps invariant cited in the housekeeping list); KC-3, G2, VR-5. |
| **Date** | 2026-07-07 |

> **Posture (transparency rule / VR-5 / house rule #4).** This ADR records *process/policy* from the
> maintainer's explicit 2026-07-07 direction: it does not itself retire any crate, does not assert any
> port is proven, and upgrades no guarantee tag. The **retire-when-proven gate is the never-silent guard**
> against premature loss (G2) — a crate's Rust is removed from the active tree only on a **checked** basis
> (the dual witness + the two-stage runtime bar), never on intent or partial progress. The **full
> Mycelium end-state** is a `Declared` goal inherited from ADR-042 (feasibility unproven there, unchanged
> here). Open mechanics — the archive branch's exact name, one-branch-vs-per-phylum, and the retirement-PR
> process — are **flagged for the maintainer**, not guessed (§5).

---

## 1. Context

ADR-042 fixes the **authoring** side of the Rust→Mycelium migration: no new Rust surface (the freeze),
and an explicit end-state — the entire first-party project, kernel included, rewritten to `.myc`, zero
foreign first-party languages by the DN-88 decomposition gate. It is careful to say what it does **not**
do: "no Rust is deleted, no module is declared ported" (ADR-042 posture note) — it sets the destination,
not the mechanism for leaving Rust behind crate by crate.

That mechanism is the gap this ADR fills. As `.myc` ports land and pass the M-989 dogfood dual witness
(`scripts/checks/myc-dogfood.sh` — the Rust differential **and** the native `myc` toolchain agreeing) and
the DN-26 two-stage runtime bar (interpreter parity first, AOT confirmation second), the maintainer's
2026-07-07 directive is explicit: **retire the Rust once proven**, to keep the active tree tidy — but
**never lose it**. The Rust is stowed on a dedicated archive branch, for posterity and audit, and as a
oracle that remains re-derivable even after the active tree no longer carries it.

This is squarely process/governance, the same register as ADR-042: it decides *when* a crate is allowed
to leave the active tree and *how* its history is preserved, not which crates are ready today.

## 2. Decision

### 2.1 Retire-when-proven gate (VR-5/G2)

A Rust crate/lib is retired **only** when **all** of the following are checked, not asserted:

1. **The myc-dogfood dual witness is green (M-989).** Both legs of `scripts/checks/myc-dogfood.sh` agree:
   the existing Rust `cargo test` differential harness **and** the native toolchain running directly over
   the `.myc` port (`myc check` — core parity; `mycfmt`/`myc-lint` — advisory). This is the same
   "old-Rust + new-`myc` parity" discipline the dogfood gate already runs for `lib/compiler/`; retirement
   generalizes it to every crate's port, not just the self-hosting frontend.
2. **The two-stage runtime bar is cleared (DN-26 §9).** The port is proven on the **interpreted** `myc`
   runtime first — the trusted reference base — via a Rust-host ≡ self-host output differential over the
   crate's corpus; **then**, where an AOT path applies to the crate, the same `.myc` is AOT-compiled and
   that build is validated in turn. The interpreted pass is the gate; the AOT pass (where applicable) is
   the follow-on confirmation, never skipped (mirroring DN-26 §9's Stage-6 resolution, generalized).
3. **Guarantee tags and behaviors are preserved.** The crate's per-operation guarantee tags carry over at
   the same or better strength — no silent downgrade (VR-5); any legitimate downgrade is explicit and
   justified, not incidental to the port.

**No retirement on a partial or unproven port.** Until all three are checked, the Rust crate remains the
**live oracle** — it is not touched, not marked deprecated, not scheduled for removal. This is the
never-silent guard the maintainer's directive implies: "archive once proven fully working" reads as a
gate, not a target date.

### 2.2 Archive branch — never lose the Rust

Retired Rust is preserved on a **dedicated, protected, persistent archive branch** holding the **full
legacy Rust codebase** — not a tombstone, not a pointer comment, not a squashed final-state snapshot
alone, but the retained history — for two purposes: **posterity/audit** (a durable, discoverable record
of what the project's Rust era actually was), and a **re-derivable differential oracle** even after a
crate leaves the active tree (if a regression is later suspected in a retired crate's `.myc` successor,
the archive branch's Rust can still be checked out and re-run against it).

**Recommended name: `archive/rust-legacy`.** This is a **FLAG for the maintainer to confirm**, not a
settled name (§5 OQ-1) — the ADR proceeds on this working name pending that confirmation.

The archive branch:
- is **never deleted, never force-pushed, never squashed away** — the same durability guarantee `main`
  carries, applied to a branch whose entire purpose is preservation;
- **joins the protected-branch set** (`MYC_PROTECTED_BRANCHES`, default `main integration dev
  claude/head/*` per `scripts/checks/branch-guard.sh` / CLAUDE.md mitigation #10) — direct pushes are
  blocked the same way, so accidental history-rewrite on the archive branch is caught by the same
  mechanism that protects `main`;
- is **not** a working branch — nothing is developed on it; it only ever receives a retired crate's final
  Rust state (mechanics: §5 OQ-4).

### 2.3 Remove from the active tree + housekeep

Once a crate's Rust is safely on the archive branch, it is **removed from the active workspace** — the
project stays single-language in the tree that is actually built, tested, and read day to day, avoiding
the dual-maintenance and "which one is authoritative" confusion of carrying both forms live.

Housekeeping is **incremental, at each per-crate retirement** — never a deferred big-bang sweep, so the
workspace stays coherent at every commit:

- **Workspace `Cargo.toml`** — the retired crate's member entry (and any now-dangling path/feature refs)
  removed.
- **The check gates** — `justfile` recipes, CI job matrices, and any script (`scripts/checks/*.sh`) that
  named the crate directly are updated so they don't fail-closed on a path that no longer exists.
- **`docs/api-index/`** — regenerated (`just docs-index`) so the retired crate's symbols are dropped, not
  left stale.
- **`docs/Doc-Index.md`** — any row or cross-reference naming the crate updated to point at its `.myc`
  successor.
- **The acyclic-deps invariant** (`scripts/checks/deps-acyclic.sh`) — re-verified after removal; a crate
  is only retirable once nothing in the active tree still depends on its Rust form (leaf-most Rust
  crates retire before their Rust dependents, the same ordering discipline DN-26 already uses for the
  compiler's own stage sequence).
- **MSRV/toolchain refs** — checked for whether the retired crate was the sole reason for a pin (unlikely
  in general, but never silently assumed unaffected — G2).

### 2.4 End state

**All non-Mycelium first-party code archived and replaced with full Mycelium.** The active tree is pure
Mycelium; the archive branch holds the complete Rust legacy. This is the same terminal state ADR-042
§2.1(b) already names (zero foreign first-party languages at the DN-88 decomposition gate); this ADR
supplies the **crate-by-crate mechanism** — retire-when-proven, archive-don't-delete, incremental
housekeeping — by which that end-state is actually reached rather than merely declared.

## 3. Consequences

- **Auditability is preserved, not traded away for tidiness.** The active tree gets the single-language
  clarity the maintainer wants; the archive branch keeps the full audit trail and a working oracle, so
  tidiness costs nothing in provenance.
- **The gate is the safeguard against premature loss.** Because retirement requires the checked dual
  witness + two-stage runtime bar (§2.1), a crate cannot be archived-and-removed on optimism; the Rust
  stays authoritative until its replacement has actually earned the swap.
- **Per-crate scope keeps retirement small and reviewable.** Each retirement is its own scoped change
  (archive-push + removal + the §2.3 housekeeping list), fitting the existing scoped-PR discipline
  (CLAUDE.md DN-65) rather than becoming a separate, unbounded program.
- **Ordering follows the dependency graph.** The acyclic-deps check (§2.3) means retirement proceeds
  leaf-first among Rust crates — the same shape DN-26 already uses for the compiler's own stage order —
  so no crate is archived while something else in the active tree still needs its Rust form.
- **The kernel is explicitly out of near-term scope.** ADR-042 names the kernel rewrite as the deepest,
  last self-hosting step, gated additionally by its own OQ-1 bootstrap/trust design. This ADR's gate
  applies to the kernel too, in principle, but its retirement is far-future and doubly gated — nothing
  here accelerates or pre-judges that step.
- **A new protected branch is a small, permanent addition to the branch-guard surface.** `MYC_PROTECTED_BRANCHES`
  grows by one entry; every future branch-guard invocation (commit-mode and push-mode) now also protects
  the archive branch by construction, once wired in (§5 OQ-1/OQ-5).
- **The archive branch grows monotonically and is never pruned.** This is the accepted cost of "never
  lose the Rust" — by design, not an oversight.

## 4. Alternatives considered

- **Keep both Rust and Mycelium live forever (dual-maintenance).** Rejected: contradicts ADR-042's
  end-state, doubles the maintenance burden, and invites exactly the "which implementation is
  authoritative" confusion the maintainer's directive is aimed at removing.
- **Delete the Rust outright with no archive.** Rejected: loses the audit trail and the re-derivable
  oracle outright, which is precisely what "don't lose the Rust" forbids; a bare `git log` history on a
  branch that later gets pruned or force-pushed is not a durable enough guarantee (G2 — a destructive
  step with no recourse is exactly the kind of silent loss the house rules forbid).
- **Archive as in-tree tombstone files (stub/comment pointing at a git SHA) instead of a branch.**
  Rejected in favor of a dedicated branch: a tombstone's pointer can be invalidated by later history
  rewrites elsewhere, and it does not offer a directly re-runnable oracle (checking out a branch and
  running its tests is far lower-friction than reconstructing a deleted crate from a commit SHA years
  later).
- **One archive branch per retired phylum, rather than a single unified archive branch.** Considered;
  not adopted here — flagged as an open question (§5 OQ-2) rather than decided, since it changes the
  protected-branch-set shape (`N` protected branches vs `1`) and the maintainer should weigh in.

## 5. Open questions / FLAGs

- **OQ-1 — archive branch name (maintainer to confirm).** Recommend `archive/rust-legacy`; not settled.
- **OQ-2 — one unified archive branch vs one per retired phylum.** A single branch keeps the protected-set
  and tooling simple (one name to remember, one place to look); per-phylum branches give a tighter,
  independently-prunable-in-principle (though §2.2 forbids pruning) scope per crate. Leaning toward a
  single unified branch for simplicity, but this is a maintainer call, not decided here.
- **OQ-3 — full history vs a final-state snapshot per crate at archival time.** "Retains the full legacy
  Rust codebase" (the maintainer's directive) reads as full history, not a squashed snapshot; this ADR
  proceeds on that reading but flags it for explicit confirmation, since it affects how the archival
  merge is performed (§5 OQ-4).
- **OQ-4 — the archival mechanics: how does a crate's Rust actually land on the archive branch?**
  Candidates to name (not resolved here): (i) a per-retirement merge/graft of the crate's directory
  history into the archive branch, preserving its commit history; (ii) a scripted "archive-crate" step
  (a dedicated `scripts/archive-crate.sh`, analogous to `branch-guard.sh`/`deps-acyclic.sh`) that performs
  the archive-branch update as one auditable, repeatable operation rather than ad hoc git surgery each
  time. Recommend a dedicated script once the first retirement is attempted, so the mechanism is proven
  once and reused, not reinvented per crate.
- **OQ-5 — the retirement-PR process.** Is "prove the port" and "retire the crate" one PR or two? Recommend
  two: a port-completion PR that lands the `.myc` port and clears the §2.1 gate, followed by a distinct
  "retire crate X" PR that performs the archive-branch update + active-tree removal + §2.3 housekeeping —
  keeping each PR scoped and reviewable per the existing DN-65 discipline. Also open: who/what is
  permitted to push to the archive branch once it is protected (§2.2) — likely only this dedicated
  retirement step, analogous to how `main` only ever advances via the squash-PR mechanism, never a
  direct push. Flagged, not decided.
- **OQ-6 — the kernel's own eventual retirement.** In scope for this gate in principle (§3), but
  additionally gated by ADR-042 §3 OQ-1's unresolved bootstrap/trust story for a self-hosted kernel; this
  ADR does not attempt to resolve that, and no kernel crate is a near-term retirement candidate under
  this decision.

## 6. User stories + Definition of Done

**User stories:**

- As a maintainer, I want a Rust crate removed from the active tree only after its `.myc` port is proven
  equivalent, so that I never lose working functionality to an unproven rewrite.
- As a future auditor or contributor, I want retired Rust code preserved and easily locatable, so that I
  can compare behavior, investigate a suspected regression, or recover project history without
  spelunking through squashed or pruned commits.
- As an agent performing a per-crate retirement, I want a concrete housekeeping checklist, so that each
  retirement is small, complete, and never leaves the workspace in a half-consistent state (a crate
  half-removed from `Cargo.toml` but still referenced in `docs/api-index/`, for instance).
- As the project approaching its full-Mycelium end-state, I want retirement to be a checked, per-crate
  gate rather than an aspiration, so that "fully Mycelium" is eventually a verifiable claim, not an
  assertion.

**Definition of Done:**

*For this ADR (the decision record):*

- [x] Maintainer directive recorded faithfully: the retire-when-proven gate, the archive branch (never
  lose the Rust), the per-crate active-tree removal + housekeeping, and the end-state — `Status:
  Accepted` (2026-07-07, in force going forward).
- [ ] Indexed in `docs/adr/README.md` and `docs/Doc-Index.md`; `CHANGELOG.md` records the decision
  *(orchestrator-owned — proposed in the companion report, applied by the integrating parent)*.
- [ ] Archive branch name confirmed by the maintainer (§5 OQ-1) and added to `MYC_PROTECTED_BRANCHES` /
  CLAUDE.md's protected-branch documentation.
- [ ] `Accepted → Enacted` **only when** (never by ratification alone — house rule #3): the archive branch
  exists and demonstrably holds the full legacy Rust of every crate retired to date (a checked state, not
  an intent), **and** — for the full end-state — every first-party Rust crate has been retired under this
  gate, the active tree is pure Mycelium, and the archive branch holds the complete legacy (mirroring
  ADR-042's own `Accepted → Enacted` criterion for the broader migration).

*For the policy it enacts (the standing per-crate gate, checked at every retirement):*

- [ ] The myc-dogfood dual witness (M-989) is green for the crate's `.myc` port — Rust differential and
  native `myc check`/`mycfmt`/`myc-lint`.
- [ ] The two-stage runtime bar is cleared — interpreter parity first, AOT confirmation second where
  applicable (DN-26 §9).
- [ ] Guarantee tags/behaviors are preserved, with any legitimate downgrade explicit and justified
  (VR-5), never incidental.
- [ ] The crate's Rust is pushed/merged onto the archive branch **before** its removal from the active
  tree — never remove-then-archive (G2: the un-lost-Rust invariant must hold at every intermediate
  state, not just at completion).
- [ ] The same retirement PR updates workspace `Cargo.toml`, the check gates, `docs/api-index/`,
  `docs/Doc-Index.md`, the acyclic-deps invariant, and MSRV/toolchain refs as needed (§2.3) — no
  deferred cleanup.
- [ ] `CHANGELOG.md` records the retirement: which crate, which port PR proved it, which differential/
  dogfood run cleared the gate.

## 7. Grounding / honesty

- Maintainer directive, 2026-07-07 (this session) — the retire-when-proven + archive + housekeep +
  end-state doctrine §2 records; binds as `Accepted`.
- **ADR-042** §2.1(b)/§2.2/§3 — the end-state and the DN-39 boundary/language distinction this ADR
  operationalizes without amending (append-only, house rule #3); ADR-042's own posture note ("no Rust is
  deleted" by that ADR alone) is the gap this ADR fills.
- **DN-26** §9 (flag-2 resolution: interpreted-first, AOT-second Stage-6 validation) and §7.3 (the
  Stage-5/6 differential discipline) — the two-stage runtime bar generalized here to every crate's
  retirement, not only the self-hosting compiler stages.
- **`scripts/checks/myc-dogfood.sh`** (M-989) — the dual-witness mechanism named as the "proven" gate's
  first checked criterion.
- **CLAUDE.md** — the `dev → integration → main` tiered/protected-branch model and mitigation #10
  (`scripts/checks/branch-guard.sh`, `MYC_PROTECTED_BRANCHES`) — the archive branch is a new member of
  that protected set, not a new mechanism.
- **`scripts/checks/deps-acyclic.sh`** — the acyclic-deps invariant cited in the per-crate housekeeping
  list (§2.3).
- KC-3 (small auditable kernel; a single-language active tree serves this), G2 (never-silent — the gate
  and the archive-before-remove ordering are both never-silent guards), VR-5 (no claim, including "this
  crate is retired," above its checked basis).

---

## Meta — changelog

- **2026-07-07 — Accepted (maintainer directive).** Records the **Rust retirement + legacy archival**
  policy operationalizing ADR-042's migration: (1) a **retire-when-proven gate** — a Rust crate retires
  only once its `.myc` port clears the M-989 myc-dogfood dual witness, the DN-26 two-stage runtime bar
  (interpreter parity first, AOT second where applicable), and preserves guarantee tags/behaviors; (2) a
  dedicated **protected, persistent archive branch** (recommended `archive/rust-legacy`, FLAGged for
  maintainer confirmation) holding the **full** legacy Rust codebase for posterity/audit and as a
  re-derivable oracle, joining `MYC_PROTECTED_BRANCHES`; (3) **per-crate, incremental housekeeping** on
  removal — workspace `Cargo.toml`, check gates, `docs/api-index/`, `docs/Doc-Index.md`, acyclic-deps,
  MSRV refs — never a big-bang sweep; (4) the **end-state**: all non-Mycelium first-party code archived
  and replaced, active tree pure Mycelium, archive branch holds the complete legacy. **Enacts nothing** —
  retirement happens per-crate as ports prove out; `Accepted → Enacted` only when the archive branch
  holds the full legacy of every crate retired so far, and (full end-state) the migration is complete
  (house rule #3). Open questions flagged, not guessed: archive-branch name (OQ-1), one-branch-vs-per-phylum
  (OQ-2), full-history-vs-snapshot (OQ-3), archival mechanics (OQ-4), the retirement-PR process (OQ-5),
  the kernel's own doubly-gated eventual retirement (OQ-6). Doc-Index / README / CHANGELOG rows owned by
  the integrating parent. (VR-5 / G2 / house rule #4.)
