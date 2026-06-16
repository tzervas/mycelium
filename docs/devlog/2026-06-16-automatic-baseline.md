# Devlog — 2026-06-16 · Making "automatic" safe to say

> **What this is** (see `docs/notes/Narrative-Capture-and-Authoring.md`): the *narrative* layer — the
> messy middle the RFCs smooth over. Append-only, informal, honest. The RFCs/ADRs/DNs remain the source
> of truth; this is the *story* of how a decision actually got made. Refs point at what shipped.

**Theme.** RFC-0015 is the part of the DynEL inspiration that's the most tempting and the most dangerous:
*automatic* baseline error handling and logging, with no boilerplate. "Automatic" and "honest" usually
fight. The whole job of M-362 was to find the line where they don't.

---

## 1. The line: additive can be automatic; control flow cannot

The load-bearing realization (RFC-0015 §4.1, A1) is almost embarrassingly simple once you see it:
**logging/presentation never changes control flow, so it is safe to apply everywhere; recovery changes
control flow, so it can never be implicit.** RFC-0013 already made presentation provably additive (its
`present()` hands the error back *unchanged* — I1 as a return type, not a promise). That single property
is what licenses auto-application. The baseline can route a swap refusal to a durable audit sink and log
it at medium detail — and it *still* cannot stop that refusal from propagating, because a
`DiagnosticPolicy` has no variant that could. The type system carries the invariant; M-362 just had to
not add a way around it.

So the enactment split cleanly: `derive_baseline` produces a `DiagnosticPolicy` (auto, default-on),
`recovery_profile` produces a `RecoveryPolicy` (explicit, default-off, opt-in). Two functions, and the
honesty boundary is the gap between them.

## 2. "Dynamically derived" had to mean "total function," not "magic"

DynEL's other word is *dynamic* — the baseline adapts to how the program is structured. The honest
reading (A4) is: a **total, deterministic function of the registry** — `baseline_for_class` is a closed
`match` with a documented rationale per arm and a safe fallback, not a heuristic and certainly not the
`eval(exception_str)` DynEL actually shipped (the anti-pattern DN-04 already closed with the
looked-up-never-evaluated `ClassName`). "Dynamic" means *computed from the mapping*, every time, the same
way. The EXPLAIN exists precisely to make that auditable: every class prints its level, route, and *why*.

The research dug up the cautionary twin: Python's `logging.basicConfig`. It's the ergonomic baseline
everyone copies — and it's ambient global state that **silently no-ops** if a handler already exists.
That's the exact failure mode A4/§8-Q1 rules out: the baseline must be *materialized and per-target*, not
a hidden global that quietly does nothing. OTP was the positive model on the recovery side — default
loggers on, supervision explicit and *bounded* (intensity/period). That's almost verbatim our A1/A2 split,
which is reassuring: someone shipped this exact boundary at scale for decades.

## 3. What "resilient" honestly contains

The recovery profiles were the place it would have been easy to over-promise. `resilient` sounds like it
should do something clever. It does exactly one honest thing: bounded `retry(≤3)` on the classes you
*explicitly* hand it. Not "the transient ones" inferred by magic — the ones you name. `strict` is emptier
still: it recovers nothing, everything propagates. Both are real RFC-0014 `RecoveryPolicy` objects (no new
mechanism), both bounded (I4), both opt-in twice over (you pick the profile *and* the classes — I5). The
profile that does the least is the default, and that is the point.

The one thing left deliberately undone: per-`nodule`/per-`phylum` baseline configuration. That wants the
M-359 manifest's inheritance, and inventing a second half-version of inheritance here would be the worse
lie. Named as future (§9), not faked.

Refs: RFC-0015 (Accepted); `research/06-automatic-baseline-diagnostics-RECORD.md` (T6.1–T6.5); issue
M-362 (#133); `crates/mycelium-lsp/src/baseline.rs`. No kernel change (KC-3).
