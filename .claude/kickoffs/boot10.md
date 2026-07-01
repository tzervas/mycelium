# Kickoff `boot10` — Self-hosting capstone: toolchain + libs in Mycelium (E18-1)

## Metadata

| Field | Value |
|---|---|
| **UID** | boot10 |
| **Head branch** | `claude/head/boot10` |
| **Status** | ready |
| **Swarm mode** | Sonnet |
| **Depends on** | E11-1 — **done** (`s10` archived, 2026-06-29); E13-1 — **done** (`lib10` archived, 2026-07-01) — **both preconditions are now met, so this kickoff is unblocked**; DN-14 (self-hosting gate — Resolved); DN-26 (bootstrap plan — Draft, this kickoff drives M-739 to fill it out) |

---

## RESCOPE (2026-07-01, ADR-036)

This kickoff is now **unblocked** — both preconditions (E11-1/`s10`, E13-1/`lib10`) have landed and
archived. But **ADR-036** (Accepted, maintainer-ratified) reclassifies what its remaining scope
*gates*: the `lang 1.0.0` **tag** needs only the core-lib self-host slice already fixed by
ADR-022 §8 Q1 (satisfied via E13-1) — **not** the full toolchain/compiler self-host this kickoff
builds. M-739…M-742 (this kickoff's whole task list) become the **comprehensive-dogfooding** track:
real, tracked, **within-1.0.0** work, but it gates the project's separate *public-release* milestone
(the repo stays private until dogfooding is complete and validated per ADR-036), not the `lang 1.0.0`
tag act (M-738, `rel10`). M-739…M-742 stay `status:needs-design` in `tools/github/issues.yaml` — no
design work has actually started; this note only corrects the *blocking relationship*, not the task
status.

---

## Scope

The **capstone criterion** of full-language 1.0.0: everything beyond the bare Rust core is
written in Mycelium (`.myc`) and the toolchain bootstraps from itself. This is Phase 5's exit
gate (ADR-021 §5; DN-25/ADR-022 full-lang definition).

Four tasks in dependency order:

1. **M-739** — self-hosting bootstrap plan (DN-26) + the staged port order: which Rust
   components move to `.myc`, in what sequence.
2. **M-740** — port the L1 frontend (lexer/parser/checker, `crates/mycelium-l1/src/`) to
   Mycelium (`.myc`), with a differential against the Rust frontend.
3. **M-741** — ratify the self-hosted toolchain as canonical (DN-14 self-hosting gate met at
   the full-toolchain level).
4. **M-742** — self-hosting CI gate: build the compiler with itself; three-way differential
   (Rust-host vs self-host vs AOT) green + mutant-witnessed.

---

## Epic / issue IDs driven

- Epic **E18-1** (`claude/head/boot10` is its head)
- Issues: M-739, M-740, M-741, M-742

---

## Grounding

- DN-14 (Resolved, 2026-06-23): all 11 self-hosting gate rows are `present` or
  `conditionally present`; `lib/std/result.myc` self-hosts as concrete evidence.
- DN-26 (Draft, 2026-06-23): the bootstrap plan stub; M-739 fills it out with the concrete
  staged port order.
- ADR-021 (Superseded by ADR-022; kernel-gate criteria preserved as track T1): explicitly scopes self-hosting to Phase 5; M-739–M-742 are that scope.
- `crates/mycelium-l1/src/`: the Rust source tree being ported (lexer.rs, token.rs, parse.rs,
  ast.rs, checkty.rs, grade.rs, decision.rs, elab.rs, mono.rs, nodule.rs — all exist today).
- `lib/std/result.myc`: the first self-hosted nodule; the stage-0 evidence base.

---

## Swarm & parallelization pattern

M-739 (bootstrap plan) is a **design task** — no implementation; runs first, serial, as a
single leaf producing the concrete port order. M-740 (port the frontend) is the large
implementation wave; it decomposes into sub-tasks per source file, but the files have mutual
dependencies (`elab.rs` calls `checkty.rs` calls `parse.rs`), so they must be sequenced or
carefully partitioned. The DN-26 §4 stage sketch (lexer first, then parser, then checker, then
elaborator) is the recommended ordering.

**Collision surface:** `crates/mycelium-l1/src/` is serial-on-L1 (each file depends on the
previous stage's output). M-740 sub-tasks must be sequenced per stage; parallel only within a
stage (e.g., if `lexer.rs` and `token.rs` are independent, they can be ported in parallel).
New `.myc` files for the self-hosted frontend live in a new directory (e.g., `lib/compiler/`)
to avoid collision with the Rust source.

**Parallelism plan:**

```
M-739 (design, serial)
  |
  v
M-740 Wave A: port lexer (lexer.rs + token.rs -> lib/compiler/lexer.myc, token.myc)
M-740 Wave B: port parser (parse.rs + ast.rs -> lib/compiler/parse.myc, ast.myc)
M-740 Wave C: port checker (checkty.rs + grade.rs + decision.rs -> lib/compiler/check.myc)
M-740 Wave D: port elaborator + mono (elab.rs + mono.rs + nodule.rs -> lib/compiler/elab.myc)
  |
  v
M-741 (ratification, serial: confirm DN-14 gate at full-toolchain level)
  |
  v
M-742 (CI gate: bootstrap + three-way differential + mutant witness)
```

---

## Sequencing & dependencies

- M-739 (plan) must complete before M-740 begins — the plan determines which stages run in
  what order and where the new `.myc` files live.
- E11-1 (kickoff `s10`) — **done** (2026-06-29) — the parser port requires the stable, ratified
  surface grammar (RFC-0030) and full HOF/closure surface (M-704) it delivered; M-740 Wave B+ is
  clear to proceed on this front.
- E13-1 (stdlib completeness) — **done** (2026-07-01, `lib10` archived) — the ratification
  criterion "everything beyond the bare Rust core is in `.myc`" now has its stdlib half satisfied;
  M-741 is clear to proceed on this front (note: per the RESCOPE above, M-741's *own* result no
  longer gates the `lang 1.0.0` tag — it gates the comprehensive-dogfooding/public-release track).
- M-742 (CI gate) depends on M-741 (ratification) to know what is canonical.

---

## Definition of Done

- DN-26 is filled out with a concrete, agreed port order and stage gates (M-739 done).
- Each stage of M-740 has a passing three-way differential: Rust-hosted compiler output for
  a test program equals self-hosted compiler output equals AOT-compiled output (`Empirical`).
- The self-hosted frontend passes the full L1 conformance corpus
  (`docs/spec/grammar/conformance/accept/` and `.../reject/`) without regression.
- M-741: a ratification document (ADR or appendix to DN-14/DN-26) records that the DN-14
  self-hosting gate is met at the full-toolchain level, with the differential evidence cited
  and graded `Empirical` (VR-5 — no upgrade to `Proven` without a theorem).
- M-742: the CI pipeline (`just check-full` or a new `just bootstrap` recipe) builds the
  compiler with itself and runs the three-way differential automatically; `cargo-mutants` on
  the Rust frontend confirms mutant-witnessing before the port is declared canonical.
- The full-language 1.0.0 capstone criterion (ADR-022, Accepted) is demonstrably met.

---

## Landing

Child branches to `claude/head/boot10` via `--no-ff` octopus merge (per wave).
`claude/head/boot10` to `main` via squash-PR (`/pr-review` + Copilot round before merge).
Orchestrator reconciles `CHANGELOG.md`, `docs/Doc-Index.md`, `tools/github/issues.yaml`,
`docs/api-index/` after the octopus merge, before the squash-PR.

---

## Agent prompt (self-contained brief)

You are running kickoff `boot10` — the self-hosting capstone of Mycelium's full-language 1.0.0.
The repo is `/home/user/mycelium`. Your working branch is `claude/head/boot10`; branch off
`dev` (or `main` if `dev` is current) and push before spawning leaf agents.

**Prerequisite check — CONFIRMED (2026-07-01):** E11-1 (kickoff `s10`) and E13-1 (stdlib
completeness) are both landed on `main` (archived kickoffs `s10`/`lib10`). The prior gate is clear;
M-740 may proceed. Per ADR-036 (2026-07-01), remember this whole task set is the
comprehensive-dogfooding track — real work, but it does not gate the `lang 1.0.0` tag.

**Context:** DN-14 (Resolved, 2026-06-23) confirms the surface is self-hosting capable for
pure, polymorphic, generic, trait-bearing modules. `lib/std/result.myc` is the first self-hosted
nodule and is on `main`. The Rust frontend lives in `crates/mycelium-l1/src/` (lexer.rs,
token.rs, parse.rs, ast.rs, checkty.rs, grade.rs, decision.rs, elab.rs, mono.rs, nodule.rs,
totality.rs, usefulness.rs, eval.rs, error.rs, ambient.rs). DN-26 is the bootstrap plan stub
that M-739 fills out.

**Your four tasks:**

1. **M-739 (bootstrap plan):** Read DN-26 (planning stub) and the full `crates/mycelium-l1/src/`
   file list. Produce a concrete port order: which files move to `.myc` first, which have mutual
   dependencies that require joint porting, what the new directory structure is
   (e.g., `lib/compiler/`), and what the stage gate criterion is for each wave. Record this as
   an append-only update to DN-26 (status stays **Draft**; it becomes `Resolved` when M-741
   lands). This task is design-only — no code changes yet.
2. **M-740 (port the L1 frontend):** Following the port order from M-739, port each stage to
   `.myc`. For each stage: write the `.myc` equivalent of the Rust file(s); run the differential
   (Rust-hosted compiler compiles a test program, self-hosted compiles the same program, compare
   L0 output); add the comparison to the CI. Each stage must be a separate commit with a green
   `just check` before the next stage begins. Graded `Empirical` (differential agreement across
   trials). Flag `Residual` for any Rust construct with no `.myc` equivalent yet.
3. **M-741 (ratification):** When M-740 is complete and E13-1 (stdlib) is landed, write a
   ratification record (an append-only addition to DN-26, or a new ADR if the maintainer
   decides a binding decision is needed) confirming: (a) DN-14 gate is met at the
   full-toolchain level; (b) the differential evidence is cited and graded `Empirical`; (c) the
   full conformance corpus passes against the self-hosted compiler. Do **not** pre-declare
   ratification before the evidence is in hand (VR-5).
4. **M-742 (CI gate):** Add a `just bootstrap` recipe that: (a) compiles the self-hosted
   compiler using the Rust-hosted compiler; (b) uses the self-hosted compiler to compile a test
   program; (c) uses the AOT-compiled output to compile the same test program; (d) diffs (b) and
   (c) for equality. Wire into `just check-full` (Tier 2). Confirm `cargo-mutants` on the Rust
   frontend has survivors that would catch a regression in the differential.

**House rules (mandatory):**
- Orchestrator owns: `tools/github/issues.yaml`, `docs/Doc-Index.md`, `CHANGELOG.md`,
  `docs/rfcs/README.md`, `docs/adr/README.md`, `docs/api-index/`. Never touch these.
- New `.myc` files live in `lib/compiler/` (or the directory M-739 decides); never overwrite
  existing `crates/mycelium-l1/src/` Rust files.
- `just check` must be green before every merge. Report any skip.
- Honesty: graded `Empirical` for differential agreement (trials); never upgrade to `Proven`
  without a checked theorem (VR-5). DN-26 stays **Draft** until M-741 ratification. The
  full-language 1.0.0 capstone criterion is never pre-declared.
- KC-3: the Rust `mycelium-core` kernel does not move to `.myc` in this wave. Do not touch
  `crates/mycelium-core/`. Flag any case where the frontend port requires a core change.
- Append-only: DN-14 and DN-26 may only have changelog entries appended; no rewriting.
