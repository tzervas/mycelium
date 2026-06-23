# Design Note DN-26 — Self-Hosting Bootstrap Plan

| Field | Value |
|---|---|
| **Note** | DN-26 |
| **Status** | **Draft** (2026-06-23) |
| **Feeds** | E18-1 (self-hosting capstone, M-739) |
| **Decides** | *Nothing normatively* — advisory planning capture. Records the staged plan for porting the Mycelium toolchain from Rust to Mycelium (`.myc`), the sequencing of which components move in which order, and the verification strategy for each stage. The actual port decisions are recorded append-only in their own RFCs/ADRs as they land. |
| **Date** | 2026-06-23 |
| **Task** | E18-1 (M-739) |

> **Posture (honesty rule / VR-5).** Advisory. "Current state" claims cite the actual source
> tree (`crates/mycelium-l1/src/`). No component is pre-declared ported; every stage gate is a
> verifiable criterion, not an intent. The DN-14 self-hosting gate (Resolved 2026-06-23) is the
> immediate prerequisite evidence base; it is cited as ground truth for what the surface can
> currently express.

---

## 1. Problem / Goal

Mycelium's full-language 1.0.0 criterion (ADR-022, forthcoming; DN-25, forthcoming) requires that
the stdlib and all libraries/phyla beyond the bare Rust core be **written in Mycelium and the
toolchain bootstrap from itself** — the canonical self-hosting capstone. This is distinct from
the Rust kernel's 1.0.0 (ADR-021, Accepted) which gates on honesty-integrity durability but
explicitly scopes self-hosting to Phase 5 (ADR-021 §5).

DN-14 (Resolved 2026-06-23) establishes that the surface language is now self-hosting-capable
for pure, polymorphic, generic, trait-bearing modules (all 11 gate-rows `present` or
`conditionally present`). The first self-hosted stdlib nodule (`std.result`, M-649) is on
`main`. The question this note addresses is: **what is the staged port order** — which Rust
components in `crates/mycelium-l1/` move to `.myc`, in what sequence, and how is each stage
verified to preserve correctness?

The DN-14 self-hosting gate says "the surface is close" — this note makes "close" concrete.

---

## 2. User stories / motivating use cases

- As a **stdlib author**, I want to write `std.collections`, `std.math`, and `std.diag` in
  Mycelium rather than Rust, so that the language eats its own cooking and library contracts are
  expressed and checked in the language itself (RFC-0016 §4.1 C1–C6).
- As a **compiler engineer** maintaining the toolchain, I want the bootstrap sequence specified
  before the port begins, so that each stage has a clear before/after criterion and a
  three-way differential witness (Rust-host vs self-host vs AOT), and regressions are caught at
  stage boundaries rather than discovered at the end.
- As a **language user** who wants to audit the compiler, I want the frontend (lexer/parser/
  checker) to be written in Mycelium so that I can read and verify the implementation in the
  same language I use to write programs — a "Mycelium all the way down" audit path.
- As a **downstream app developer** evaluating Mycelium for production, I want evidence that the
  language can build non-trivial real programs (including itself), so that I can assess its
  maturity beyond "hello world"-scale examples.
- As a **maintainer**, I want the bootstrap plan to be append-only and honestly staged, so that
  partial progress is visible and the capstone criterion (full self-hosting) is never
  pre-declared before it is verified.

---

## 3. Scope & decision space

**In scope:**
- The **staged port order**: which components of `crates/mycelium-l1/src/` are ported in which
  order, and what the dependency graph looks like (each stage may depend on previous stages
  being self-hosted and stable).
- The **verification strategy** for each stage: a three-way differential (Rust-hosted compiler
  output ≡ self-hosted compiler output ≡ AOT-compiled output), with the differential graded
  `Empirical` (trials) and witnessed by `cargo-mutants` (VR-5).
- The **bootstrap gate**: a formal criterion for when the toolchain is considered self-hosting
  (the compiler can compile itself and produce an output that compiles itself again —
  "stage 2" in traditional compiler terminology).
- The **interaction with E11-1** (surface completeness): some port stages may be blocked until
  surface constructs are stable (e.g., porting `mono.rs` requires the defunctionalization
  surface RFC-0024 to be ratified; porting `parse.rs` requires RFC-0030's grammar ratification).

**Out of scope:**
- The Rust `mycelium-core` kernel — it stays in Rust for the 1.0.0 kernel gate (ADR-021). The
  port plan covers the L1 frontend and stdlib, not L0.
- Performance optimization of the self-hosted compiler — the initial port target is correctness;
  performance tuning is a follow-on Phase 6+ item.
- Cross-compilation and target bootstrapping (building for a non-host target) — deferred to
  a future ADR.

---

## 4. Tentative stage sketch (advisory, not normative)

The following is a first-pass ordering for discussion, grounded in the dependency structure of
`crates/mycelium-l1/src/`. Concrete decisions belong in per-stage RFCs or ADRs as each port
lands.

| Stage | Component | Key dependency | Verification |
|---|---|---|---|
| 0 | `std.result`, `std.option`, `std.collections` (stdlib modules) | DN-14 gate met (M-649); RFC-0024 HOF | three-way differential; `Empirical` |
| 1 | Lexer (`lexer.rs`, `token.rs`) | RFC-0030 grammar ratified; no Rust FFI in the lexer | lexer output differential Rust ≡ self-hosted |
| 2 | Parser (`parse.rs`, `ast.rs`) | Stage 1 (self-hosted lexer); RFC-0030 ratified | AST differential; conformance corpus |
| 3 | Type checker (`checkty.rs`, `grade.rs`, `decision.rs`) | Stage 2; RFC-0019/RFC-0018 enacted | checker output differential |
| 4 | Elaborator + mono (`elab.rs`, `mono.rs`, `nodule.rs`) | Stage 3; RFC-0024 ratified | L0-output differential; mutant-witnessed |
| 5 | Bootstrap gate (compile the compiler with itself) | Stages 1–4 green | stage-2 bootstrap + three-way differential |

This sketch is **advisory and subject to revision** as the surface language evolves and surface
gaps are discovered during the port. Append-only updates to this note record discovered blockers
or sequencing changes.

---

## 5. Open questions

1. **Port order for `elab.rs` vs `checkty.rs`:** the elaborator calls the checker; should the
   checker be ported before the elaborator, or can they be ported together as a unit? What does
   the dependency direction imply for the stage boundary?
2. **`wild`/FFI execution (DN-14 row 9):** the `wild` block's execution path is staged (no FFI
   host in v0). Any self-hosted component that needs to call Rust (e.g., file I/O for reading
   `.myc` source files) must go through the `@std-sys` + `wild` path. What is the plan for
   self-hosted I/O before `wild` execution is fully wired?
3. **Mutual dependency (`elab.rs` ↔ `nodule.rs` ↔ `parse.rs`):** these modules call each other.
   Can the port handle mutual recursion across module boundaries in the self-hosted compiler, or
   does this require a module-system design decision first?
4. **Three-way differential scope:** the current M-210 differential covers L1-eval ≡ L0-interp ≡
   AOT. When the compiler is self-hosted, what is the new differential? Rust-hosted ≡ self-hosted
   (same L0 output)? Or a stage-2 bootstrap (self-hosted compiler → compiled output compiles
   itself)?
5. **Toolchain build system:** the current build is `cargo`. When the compiler is self-hosted,
   what builds `myc`? A `myc` interpreter invoked from the `justfile`? An AOT-compiled `myc`
   binary? This is the practical bootstrap problem and needs a concrete answer before Stage 5.
6. **Dependency surface for the self-hosted compiler:** which Rust libraries does the L1 frontend
   currently depend on (beyond `std`)? Those dependencies must be either replaced by self-hosted
   Mycelium code or kept as `wild`-accessed primitives.

---

## 6. Grounding / honesty

This note is grounded in:
- **DN-14** (Resolved, 2026-06-23): the self-hosting gate assessment — all 11 feature rows are
  `present` or `conditionally present`; `std.result` self-hosts as concrete evidence.
- **`crates/mycelium-l1/src/`**: the actual Rust source tree being targeted for the port (file
  list above is grounded in `ls crates/mycelium-l1/src/` — the files exist today).
- **RFC-0024** (implemented, pending ratification): HOF is implemented; named functions are
  first-class — a prerequisite for porting the elaborator.
- **ADR-021** (Accepted): the kernel/core 1.0.0 gate explicitly scopes self-hosting to Phase 5;
  this note is the Phase 5 planning capture.

No stage is pre-declared done. No guarantee tag is upgraded. The stage sketch is advisory
(`Declared` planning intent, not `Empirical` evidence); each stage's differential result is
`Empirical` only after trials run.

---

## Meta — changelog

- **2026-06-23 — Draft stub created.** Scope, user stories, decision space, tentative stage
  sketch, and open questions captured as a planning stub. Feeds E18-1 (M-739). Built on DN-14
  (Resolved) and ADR-021 (Accepted). Decides nothing normatively. Status: **Draft** (VR-5 /
  house rule #3).
