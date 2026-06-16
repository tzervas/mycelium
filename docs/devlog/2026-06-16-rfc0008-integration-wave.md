# Devlog — 2026-06-16 · The RFC-0008 integration wave (and the lexicon/metadata follow-ons)

> **What this is** (see `docs/notes/Narrative-Capture-and-Authoring.md`): the *narrative* layer — the
> messy middle the RFCs smooth over. Append-only, informal, honest. The RFCs/ADRs/DNs remain the source
> of truth; this is the *story* of how a few decisions actually got made. Refs point at the commits/docs
> that shipped.

**Theme of the wave.** Phase 4 had just delivered the diagnostics (RFC-0013) and recovery (RFC-0014)
subsystems with their *runtime halves deliberately deferred* to "the RFC-0008 integration." This session
wired those deferred halves in, ratified RFC-0008, lifted RFC-0014's single-task boundary, and then —
prompted by the maintainer — captured several naming/metadata/authoring design points.

---

## 1. "Where does the shared budget primitive live?" — the dependency-graph problem (M-353)

**Problem.** RFC-0014 §4.8 deferred wiring the recovery `Budgets` ledger into the runtime so a budgeted
effect overruns *gracefully at runtime exactly as recursion does* (`FuelExhausted`/`DepthLimit`). But the
ledger lived in `mycelium-lsp` (tooling) and the env-machine that had to enforce it lived in
`mycelium-mlir` — two **sibling** crates with no edge between them.

**Why it mattered.** Putting the shared type in the wrong crate would either (a) force a dependency cycle
(`mlir → lsp`), or (b) tempt us to grow the *kernel* with an effect concept — violating KC-3 and the
ratified "no kernel hook" disposition.

**Approach — and what we didn't do.** We did *not* invent an L0 effect node (the obvious-but-wrong move:
it would have made effects kernel-visible). Instead we found the **common ancestor**: both `lsp` and
`mlir` depend on `mycelium-interp`, where the fuel clock and `EvalError` already live. Lifting
`EffectKind`/`EffectBudget`/`Budgets` into `mycelium_interp::budget` made it the *shared budget-resolution
surface* both sides consume, with **zero** new kernel nodes — the same "find the no-cycle home" move the
later concurrency primitives reused (M-356).

**Resolution.** An overrun routes through a new `EvalError::EffectBudget` — one runtime refusal channel
over *separate named budgets* (the ratified §8 disposition). The env-machine charges a declared `alloc`
budget per control-stack frame as the *opt-in sibling of the DN-05 depth ceiling* (same per-frame-bytes
reasoning) — absent ⇒ identical behaviour. Refs: commit `08ef056`; RFC-0014 §4.8 changelog.

**Lesson worth a blog paragraph.** "Honest integration sometimes means *moving a type down* to the
common ancestor rather than *adding a dependency across* — the dependency graph is a design constraint,
not an afterthought."

---

## 2. The honesty boundary for *automatic* recovery (RFC-0015)

**Problem.** The maintainer wants DynEL-style *automatic baseline* error handling/logging — auto-wrapping
for QoL. But "automatic error handling" is precisely where a **silent black box** could creep in, which
the whole project forbids (G2).

**The insight that unlocked it.** Split the automation by *what it touches*: **presentation/logging is
additive** (RFC-0013 never changes control flow — its I1), so it is *always safe to auto-apply*;
**recovery changes control flow**, so it must stay **opt-in, declared, bounded** (RFC-0014 I3/I4/I5) and
can only ever be *scaffolded* or applied as a *named opt-in profile*. Automatic presentation: yes.
Automatic control-flow change: never implicit.

**Resolution.** RFC-0015 §4.1 fixes this as the load-bearing rule before any code. Refs: commit
`c05ca7d`; RFC-0015 Draft.

**Lesson.** "The safe-to-automate boundary fell exactly on an invariant we'd already drawn (I1, additive
presentation). Good invariants pay forward."

---

## 3. The `colony` naming collision (DN-06)

**Problem.** The proposal wanted `colony` for the *dynamic* runtime grouping of `hypha`. But DN-02 had
**already ratified** `colony` = the static module — 226 references, the grammar, the conformance corpus —
and the naming law is *append-only* ("supersede, don't edit").

**Approach.** Rather than treat the collision as a blocker, we ran the DN-02 **three-test gate** on the
*reassignment* and found it actually *improves* fidelity: a "colony" is a *living, cooperating group* —
which maps to running tasks far better than to a static file that never "lives." So `colony` moves to the
dynamic meaning, its static role becomes `nodule`, and `phylum` is the new library level above nodules.

**Resolution.** A supersession recorded in DN-02's changelog (append-only), the mechanical keyword
migration staged as its own task (M-358) precisely because it touches the grammar contract. Refs: DN-06
(Resolved); commit `b34ba39`.

**Lesson.** "An append-only rule isn't a straitjacket — it forces you to *justify* a rename against the
same gate the original passed, and sometimes the rename is the *more* honest mapping."

---

## 4. `reclaim` supervision: bounding a cascade without a clock (M-356)

**Problem.** Erlang/OTP's max-restart-intensity bounds a restart storm as *N restarts within a time
window T*. But Mycelium **defers wall-clock time** (RFC-0008 R8-Q3) — so a naive port would smuggle in an
ungrounded time dependency.

**Approach — the combined answer.** The maintainer asked for "a combined solution between cascade budget
and time-windowed intensity." We bounded the cascade on **both axes**: a *total* restart cap via the
RFC-0014 `cascade` **effect budget** (reusing M-353's unified channel), and a *rate* cap via a windowed
intensity over a **logical clock** (a deterministic monotonic counter the supervisor advances) — not the
wall clock. Honest *now*, gains real-time semantics only when R8-Q3 lands.

**Resolution.** `mycelium_interp::supervise::Supervisor`, exceeding either bound an explicit
`Escalation` — a declared, bounded cascade, never a storm. Refs: commit `df3440b`; RFC-0008 §4.7.

**Lesson.** "When a borrowed pattern depends on something you've deferred (time), find the *deterministic
substrate* you already have (a logical clock + the existing budget mechanism) and bound on it honestly."

---

## 5. A small one: context compaction hid a landed commit

Mid-session, a long context was summarized and the agent briefly "forgot" that **M-357 had already
landed** (the RT2 fork/join runtime). Caught it by reading `git log` before editing bookkeeping, and
reconciled a stale `needs-design` label → `done`. **Lesson:** trust the repo state (git log, the docs)
over working memory; reconcile, don't assume. Refs: commit `24bc63d` (the hidden one), reconciled in a
later bookkeeping pass.

---

*Seed entry — demonstrates the `problem → why → approach (incl. the road not taken) → resolution →
lesson` format the authoring pipeline (blog/book asides) will draw on.*
