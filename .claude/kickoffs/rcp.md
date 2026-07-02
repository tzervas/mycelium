# Kickoff `rcp` — Rust-reference completion + acyclic dependency graph (pre-E18-1 closeout)

> **STATUS: PROPOSED — for maintainer review. Do not execute until the maintainer approves this
> kickoff and its plan doc.** Nothing here is started; no issue statuses have been touched.
>
> **⚠ Plan revised (2026-07-01):** the plan doc was revised into the **function-first umbrella
> roadmap** `docs/planning/road-to-1.0.0-and-mycelium-rewrite.md` (governed by **ADR-038**, Accepted
> 2026-07-01; a pointer stub remains at the old path). This kickoff's workstream references map per the roadmap's
> meta-changelog: **A→H0 · §8a→H1 · B–G→H2 · grammar items→H2a**. Read the roadmap first; the "before
> self-hosting" framing below is superseded by ADR-038's function-first sequencing.
>
> **UID:** `rcp` · **Plan (source of truth):** `docs/planning/road-to-1.0.0-and-mycelium-rewrite.md`
> · **Basis:** ADR-022 (T1–T7 done, T8 in-progress, T9 non-gating) · DN-56 (kernel freeze, 0/4
> conditions met) · ADR-036 (self-hosting gates *public release*, not this milestone) · the three
> Fable-5 research digs (2026-07-01: dep-graph / roadmap / open-work surveys).
> **References the doc-maintenance contract** (`_doc-maintenance.md`) as part of its Definition of Done.

## Goal (maintainer directive, 2026-07-01)

**Everything implemented in Rust (the reference implementation), with a clean acyclic crate
dependency graph — no circular deps — enforced structurally.** This milestone precedes and gates
the E18-1 self-hosting rewrite (which is the *next* kickoff, not this one). The reframe that sizes
it: the Rust reference is already far along (zero `todo!`/`unimplemented!` in non-test src; all
incompleteness is explicit never-silent `Residual`/refusal) — this is a **bounded closeout**, not a
build-out. The plan doc's §1 exit criteria define "done"; its §2–§9 workstreams define the work.

## Scope

**In:** the seven workstreams of the plan doc — **A** dep-graph hardening (structural cycle
enforcement, the 3 `cert` dev-cycles, the `mlir→std-runtime` anomaly); **B** language-semantics
remainder (`mycelium-l1`); **C** value-model + AOT refusal lifting (E20-1 tail); **D** runtime R2
and concurrency maturity; **E** toolchain/UX (`myc run`, LSP, M-697, M-848); **F** inject-mode
mechanism (RFC-0038 chain); **G** kernel freeze (DN-56); plus the continuous §9 proof-debt items
as their crates are touched.

**Out:** E18-1 self-hosting execution (next milestone — hand off, don't start); the
maintainer-reserved acts (M-703 kernel tag, M-655, M-381/M-646 LLM runs, M-816, the M-738 release
act); any maintainer decision in the plan's §10 FLAG table (tuple type, ADR-033 FLAG-1, RFC-0037
plus DN-54 checker, RFC-0038 scope, RFC-0035, RFC-0027/DN-32) — those are **ratified by the
maintainer, never decided by an agent** (G2/VR-5); big-bang M-797 (stays the lazy as-touched sweep).

## Shape: multi-session Wave-N (protected heads per workstream)

Too big for one session ⇒ split per the Wave-N workflow, partitioned by **disjoint crate
ownership** (one protected `claude/head/*` base per parallel workstream; PR-only onto heads;
squash only at `main`):

| Head (proposed) | Workstream | Ownership / collision profile |
|---|---|---|
| `claude/head/rcp-deps` | **A — first, serial, short** | Workspace manifests, the new check (`cargo-deny`/xtask), `cert`/`mlir` seams. Everything else launches only after A's enforcement is on `dev`. |
| `claude/head/rcp-lang` | **B** | `mycelium-l1` — the **high-collision crate ⇒ serial-on-shared-files**: ONE lane, sequential PRs, no fan-out inside l1 (the `mycelium-l1` collision rule from the Wave-N playbook). |
| `claude/head/rcp-value` | **C** | `mlir` / `dense` / `vsa` + value-model crates — parallel-leaf octopus inside the head. |
| `claude/head/rcp-runtime` | **D** | `sched` / `interp` / `std-runtime`. **Seam watch:** if F lands mechanism code in `interp`, sequence those PRs with F or raise ownership to the shared parent. D must not pull `cert` into interp (the latent cycle — A's check makes it a hard error). |
| `claude/head/rcp-tool` | **E** | `cli` / `lsp` / justfile tooling. |
| `claude/head/rcp-sec` | **F** | The RFC-0038 inject-mode surface (`sec` + wherever M-836…M-849 land). Design-first; deviations FLAG up. |
| *(no separate head)* | **G** | Kernel freeze runs **last**, in the final integration session — it reviews the merged result, it doesn't race it. |

Cross-session continuity rides `issues.yaml` (`depends_on` + body notes), never another session's
files. Heads complete and self-integrate first; the final session octopus-merges heads, reconciles
shared files once (`CHANGELOG`/`Doc-Index`/`api-index`/issue close-out at the integration tier),
runs Workstream G, and squash-PRs to `main` per `/wave-land`.

## Efficiency / swarm playbook

- **Sonnet swarm default**; leaf work is well-specified closeout — no Opus fan-out needed. Reserve
  Opus for the G-milestone KC-3 review and any FLAG adjudication prep.
- **`/wave` per head** — one isolated worktree per agent (mitigation #11, `/worktree-guard --leaf`),
  change-scoped checks only at leaves (`cargo test -p <crate>`, DN-20), `/pr-land` review loop for
  every PR up the tree; leaves never touch `CHANGELOG`/`Doc-Index`/`api-index`/issue close-out
  (FLAG up; the integration tier reconciles once).
- **A first, alone.** Do not launch B–F heads until A's structural check is merged to `dev` —
  the whole point is that B–F land *under* cycle-enforcement.
- **B is one serial lane.** Its items are also the most decision-gated: batch the ratifications
  (plan §10 FLAG table; reconcile with `docs/planning/Blocked-Decisions-Ratification-Map.md`)
  **before** launching the B session, so the lane doesn't stall mid-flight. Start B with M-866
  (the recursion-safety bug — ready now, no decision needed).
- **Explore agents for read-heavy scoping** (per-workstream issue verification against
  `issues.yaml`, refusal-site inventories) — conclusions only, protect orchestrator context
  (mitigation #6). Commit + push on the `wip(batch M/N)` cadence (mitigations #5/#9); split
  commit and push commands (mitigation #12).

## Opening moves (once approved)

1. **Ratification batch:** maintainer dispositions the plan §10 FLAG table (or explicitly defers
   rows — a deferred decision parks its dependent items, it doesn't block the rest).
2. **Session 1 (`rcp-deps`, Workstream A):** branch the head off current `dev`; verify the dig's
   graph facts at HEAD (`cargo metadata` re-survey — `Empirical`, recorded); land A1 enforcement,
   then A2/A3/A4 as scoped PRs; PR the head to `dev` via `/pr-land`.
3. **Fan out sessions B–F** per the head table, each opened with `/kickoff rcp` + its workstream
   name; verify each workstream's M-xxx ids against `issues.yaml` at session start (the plan's ids
   are `Empirical` leads from the digs, not re-verified line-by-line).
4. **Final session:** integrate heads, Workstream G, §1 exit-criteria audit, closeout note,
   squash-PR to `main`, propagate down (`scripts/sync-heads.sh`), hand off to the E18-1 kickoff.

## Definition of Done

- Plan §1 exit criteria all met or maintainer-deferred with record: workstreams landed/deferred;
  acyclic invariant structurally enforced and green (normal + dev deps; `interp` std-free);
  DN-56 4/4 with citations; ADR-022 T8 closeable modulo reserved acts; every surviving refusal
  re-affirmed with a tracker item; closeout note written.
- **Doc-maintenance (`_doc-maintenance.md`):** issue statuses moved as work actually lands (never
  ahead of it); CHANGELOG + per-doc footers reconciled at the integration tier; `doc_refs` valid;
  `docs/api-index/` regenerated by the integrating parent after public-API changes.
- Guarantee tags at supportable strength everywhere (VR-5 — notably RT2 stays `Empirical` unless
  the Kahn side-conditions are machine-checked); no decision made by an agent that the plan FLAGs
  as the maintainer's.
- E18-1 handoff explicit: the next kickoff can cite "Rust reference complete" as a met
  precondition.

## Prerequisites the maintainer supplies at session start

1. **Approval of this kickoff + the plan doc** (this is a proposal — nothing runs before that).
2. **The ratification batch** (plan §10 FLAG table) — at minimum: the tuple-type decision and
   ADR-033 FLAG-1 before Workstream B's gated rows; RFC-0038 scope before F; RFC-0035 before
   E22-1; RFC-0037 + DN-54 checker before G's condition 3.
3. **Head-branch creation/protection** for the `claude/head/rcp-*` set (protected, PR-only).
