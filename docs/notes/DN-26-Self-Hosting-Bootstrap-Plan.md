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
<!-- erratum 2026-06-25 (applies to every "ADR-021, Accepted" cite in this note): ADR-021 is now
**Superseded by ADR-022** (2026-06-23); its kernel Gate A/B is preserved as ADR-022 track T1. Read each
"ADR-021 (Accepted)" reference as "ADR-021 (Accepted → Superseded by ADR-022); gate carried into ADR-022 T1". -->

DN-14 (Resolved 2026-06-23) establishes that the surface language is now self-hosting-capable
for pure, polymorphic, generic, trait-bearing modules (all 11 gate-rows `present` or
`conditionally present`). The first self-hosted stdlib nodule (`std.result`, M-649) is on
`main`. The question this note addresses is: **what is the staged port order** — which Rust
components in `crates/mycelium-l1/` move to `.myc`, in what sequence, and how is each stage
verified to preserve correctness?

The DN-14 self-hosting gate says "the surface is close" — this note makes "close" concrete.

> **Note (2026-06-25, append-only).** DN-14 **row 9** (`wild`/FFI) has since flipped from
> "conditionally present" to **"present / executes"** (RFC-0028 / M-720–M-721 landed `wild` host
> execution), so the "all 11 gate-rows … *conditionally present*" phrasing above is now **"all 11
> present."** Status unchanged (Draft). (Append-only; VR-5/G2.)

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

## 7. Concrete port order (M-739) — grounded in the actual `mycelium-l1` dependency graph

> **Posture.** This section fills out §4's *advisory sketch* with a **concrete, grounded** port
> order derived from the real intra-crate dependency graph of `crates/mycelium-l1/src/` (measured
> 2026-07-03, `grep 'use crate::' + 'crate::<mod>::'` over every `.rs`). It is still **planning
> intent** (`Declared`): no stage is done, and each stage's differential is `Empirical` only after
> trials run. Status of this note stays **Draft** until M-741 ratifies the port (house rule #3 /
> VR-5). Where a stage boundary depends on an architecturally-significant decision the maintainer
> has not ratified, it is **flagged `[FLAG]`**, not silently decided (G2).

### 7.1 The measured dependency graph

Nineteen source modules (excluding `tests`). Edges are `use crate::<mod>` / `crate::<mod>::…`
**structural** references; intra-doc-comment (`///`) links were excluded (verified: `ast.rs`'s
apparent upward refs to `checkty`/`elab`/`eval`/`parse` are all doc-links, so `ast` is a clean
data-type foundation).

```
token  ⇄  error                     (small 2-node cycle: error wraps tokens; token names error kinds)
lexer   → token, error
ast     → (none — pure data types; DN-02 vocabulary)
nodule  → (none — standalone `// nodule:` header parser)
parse   → ast, error, lexer, token
totality→ ast
ambient → ast
substrate→ (none structural)
grade   → ast, checkty
usefulness → checkty
decision→ checkty, usefulness
affine  → checkty
fuse    → ast, checkty, eval
checkty → affine, ambient, ast, usefulness   (+ calls decision, elab, eval, fuse, grade, substrate, totality)
elab    → ast, checkty, decision             (+ calls eval, mono, totality, usefulness)
eval    → ast, checkty, elab                 (+ calls substrate)
mono    → ast, checkty, elab, totality       (+ calls fuse, grade)
```

**The decisive finding.** `checkty`, `elab`, `eval`, `mono`, `fuse`, `decision`, `usefulness`,
`grade`, `affine` form **one strongly-connected component** (SCC): `checkty ↔ elab ↔ eval`,
`checkty ↔ affine`, `checkty ↔ usefulness ↔ decision`, `checkty ↔ grade`, `eval ↔ fuse`,
`elab ↔ mono`. They call each other cyclically and **cannot be ported one file at a time** — a
partial port would leave dangling references at every stage boundary. This SCC is ~780 KB of Rust
(`checkty` 377 KB + `mono` 160 KB + `elab` 118 KB + `eval` 80 KB + the rest) — the bulk of the
frontend. `ast`, `ambient`, `totality`, `substrate` are **leaf dependencies** of the SCC (they are
called *by* it but do not call *into* it), so they port **before** it, cleanly.

This retires §5 open questions **#1** and **#3**: the checker and elaborator are *not* separable
into ordered stages — they are a single mutually-recursive porting unit. Mycelium already supports
exactly this: **DN-14 row 3** (`present`) — nodule-wide mutual recursion via Tarjan-SCC →
`FixGroup` (`elab.rs` `FixGroup`; DN-13; M-343 + M-391). So the SCC maps onto **one nodule** whose
functions form a nodule-wide `FixGroup`; mutual recursion is free *within* a nodule, so the port
does not need a new module-system feature to express the semantic core.

### 7.2 The frontend / kernel boundary (what stays Rust — KC-3)

`mycelium-l1` links four Rust crates: `mycelium-core` (L0 value/IR), `mycelium-interp` (the trusted
reference evaluator + prim/swap registries), `mycelium-cert`, `mycelium-select` (RFC-0005 selection
for `forage`), plus a host-stack helper. **Per KC-3 and §3-out-of-scope, none of these move in this
wave.** The self-hosted frontend is `source text → L0 program`; the **Rust kernel evaluates the L0**
(the same trusted base the three-way differential already pivots on). Concretely: the ported `.myc`
frontend calls back into `mycelium-core`/`interp` primitives (L0 construction, the prim registry)
through the **`@std-sys` + `wild` FFI seam** — which DN-14 row 9 confirms now *executes* (RFC-0028;
M-720/M-721), retiring §5 open question **#2** for the I/O and kernel-callback paths.

### 7.3 Staged port order

| Stage | Unit → new `.myc` nodule(s) | Rust source | Enables | Stage gate (all `Empirical`) |
|---|---|---|---|---|
| **0** *(done)* | stdlib self-host | `lib/std/*.myc` (17 modules on `main`) | evidence base | already green (M-649…M-719) |
| **1 — Tokens+Lexer** | `lib/compiler/token.myc`, `lib/compiler/lex.myc` (`token`+`error` co-ported — the 2-cycle — then `lexer`) | `token.rs`, `error.rs`, `lexer.rs` | source → token stream | token-stream differential Rust≡self over the accept-corpus |
| **2 — Nodule header** | `lib/compiler/nodule.myc` | `nodule.rs` | `// nodule:` header parse | header-parse differential (standalone; can run parallel to Stage 1) |
| **3 — AST+Parser** | `lib/compiler/ast.myc`, `lib/compiler/parse.myc` | `ast.rs`, `parse.rs` | token stream → AST/phylum | AST differential + **full L1 accept/reject conformance corpus** parity |
| **4 — Leaf semantics** | `lib/compiler/ambient.myc`, `lib/compiler/totality.myc`, `lib/compiler/substrate.myc` | `ambient.rs`, `totality.rs`, `substrate.rs` | SCC leaf deps in place | unit differentials; no SCC refs yet |
| **5 — Semantic core (SCC, one nodule)** | `lib/compiler/semcore.myc` (a single nodule; `checkty`+`elab`+`eval`+`mono`+`fuse`+`decision`+`usefulness`+`grade`+`affine` as one nodule-wide `FixGroup`) | the 9 SCC `.rs` files | AST → checked, elaborated **closed L0** | **L0-output differential** Rust≡self over the corpus; `cargo-mutants` witness on the Rust SCC |
| **6 — Bootstrap gate** | `just bootstrap` recipe (M-742) | — | compiler compiles itself | stage-2 bootstrap + **three-way** differential (Rust-host ≡ self-host ≡ AOT) |

Stages 1–4 are **each a separate green-`just check` commit** (small, auditable — KC-3). Stage 5 is
the large one and may itself be committed incrementally *within the single nodule* (function group by
function group) as long as the nodule compiles at each commit; but it lands as **one porting unit**,
not as ordered sub-stages, because of the SCC. **`[FLAG] Stage-5 packaging** — SCC-as-one-nodule vs a
`compiler` *phylum* with the SCC in one nodule and the leaves (Stage 4) in sibling nodules — is an
architecturally-significant choice deferred to M-740's first commit / maintainer call; the DN-14
row-3 mechanism supports the single-nodule form today, so that is the **recommended default**, but
the phylum form (cleaner audit surface, cross-nodule `pub` boundaries) is the alternative and is
flagged, not silently foreclosed.

### 7.4 Directory structure

New self-hosted frontend lives under **`lib/compiler/`** (a new phylum, sibling to `lib/std/`);
existing `crates/mycelium-l1/src/*.rs` is **never overwritten** — the Rust frontend stays the
trusted differential oracle until M-741 ratifies the port canonical. Differential harnesses live in
`crates/mycelium-l1/tests/` (the established `std_*.rs` pattern), reading both the Rust output and
the self-hosted output for the same input program.

## 8. Resolutions to §5 open questions (M-739)

1. **`elab` vs `checkty` order** → **Resolved: neither-before-the-other.** They are in the same SCC
   (§7.1); ported together as one nodule-wide `FixGroup` (Stage 5). Grounded in DN-14 row 3.
2. **`wild`/FFI for self-hosted I/O + kernel callbacks** → **Resolved (path exists).** Source-file
   I/O and L0/prim-registry callbacks go through `@std-sys` + `wild`, which **executes** today
   (DN-14 row 9; RFC-0028; M-720/M-721). The self-hosted frontend's `main`/driver nodule is
   `@std-sys` and declares `!{ffi}` at each host-call site (audited, not verified — VR-5/ADR-014).
3. **Mutual dependency `elab ↔ … ↔ parse`** → **Resolved.** `parse` is *not* in the SCC (it depends
   only on `ast`/`lexer`/`token`, Stage 3); the mutual recursion is confined to the semantic SCC and
   handled nodule-wide (see #1). No cross-nodule mutual-recursion feature is required.
4. **Three-way differential scope** → **Two differentials, staged.** Stages 1–5: **Rust-host ≡
   self-host** on the *same L0 output* for the corpus (`Empirical`). Stage 6: the **stage-2
   bootstrap** three-way (Rust-host ≡ self-host ≡ AOT), reusing the existing M-210 harness pivoted on
   the L0 the ported frontend now produces.
5. **Toolchain build system** → **Recommended (advisory).** Until Stage 6, `cargo` builds the Rust
   `myc` and the `just bootstrap` recipe drives the self-hosted frontend *through* the Rust-hosted
   `myc` interpreter (or its AOT binary) over `lib/compiler/*.myc`. A fully `cargo`-free build is a
   post-1.0 follow-on (§3 out-of-scope). **[FLAG]** the canonical Stage-6 driver (interpreted vs
   AOT-compiled `myc`) is an M-742 decision.
6. **Rust dependency surface** → **Enumerated (§7.2):** `mycelium-core`, `mycelium-interp`,
   `mycelium-cert`, `mycelium-select`, + host-stack helper. All are the **L0 kernel** and stay Rust
   (KC-3); the frontend reaches them via `wild`, not by porting them. Any frontend port step that
   *appears* to need a `mycelium-core` change is a **FLAG-up**, not an in-wave core edit.

**Definition of Done for M-739 (this section):** DN-26 now carries a concrete, grounded staged port
order (§7.3), a directory structure (§7.4), the frontend/kernel boundary (§7.2), and resolutions to
all six §5 open questions (§8), with the two architecturally-significant choices explicitly `[FLAG]`ged
for M-740/M-742 rather than pre-decided. Status stays **Draft** → becomes **Resolved** with M-741
(house rule #3). M-739 is design-only: no code changed.

## 9. Flag resolutions (maintainer-decided, 2026-07-03)

The two architecturally-significant `[FLAG]`s raised in §7.3 / §8 are now **resolved by the
maintainer** (2026-07-03). Recording append-only; the plan above stands, with the flagged branches
fixed as below.

1. **Stage-5 packaging (§7.3 FLAG) → the `compiler` *phylum*, with the semantic SCC as one nodule.**
   `lib/compiler/` is a **phylum** (RFC-0006 §4.3): the semantic core SCC
   (`checkty·elab·eval·mono·fuse·decision·usefulness·grade·affine`) is **one nodule** within it (so
   its functions form a single nodule-wide `FixGroup` — mutual recursion for free, DN-14 row 3), and
   the leaf stages (`token·lex·nodule·ast·parse·ambient·totality·substrate`) are **sibling nodules**
   in the same phylum, exporting across nodule boundaries with `pub` + cross-nodule `use` (DN-14
   row 10, `present`). This takes the phylum alternative of the flag (cleaner audit surface + explicit
   `pub` boundaries) **and** keeps the SCC monolithic-as-a-nodule (the mechanism that makes the
   mutual recursion expressible today). Chosen over the single-nodule-for-everything form.
2. **Stage-6 bootstrap driver (§8 Q5 FLAG) → validate on the interpreted `myc` first, then on the
   AOT-compiled `myc`.** The self-hosted frontend is proven on the **interpreted** `myc` runtime
   first (the trusted reference base — Rust-host ≡ self-host L0-output differential over the corpus,
   `Empirical`); **once that is validated**, the same `.myc` is **AOT-compiled** and the AOT build is
   validated in turn. This deliberately exercises **both runtimes on the identical `.myc` source** and
   asserts they agree — the stage-2 bootstrap three-way (Rust-host ≡ self-host-interpreted ≡
   self-host-AOT) — so the port proves out the interpreter and the AOT path together, not just one.
   The interpreted pass is the gate; the AOT pass is the follow-on confirmation, never skipped (G2).

These resolutions **do not** move DN-26's status (still **Draft** → Resolved with M-741) and add no
code; they only fix the two branch points so the M-740 wave can proceed without re-deciding mid-port.

## Meta — changelog

- **2026-07-03 — §9 added: the two §7.3/§8 `[FLAG]`s resolved by the maintainer (append-only, no status
  move).** Stage-5 packaging → the **`compiler` phylum with the semantic SCC as one nodule** (leaves as
  sibling nodules; chosen over single-nodule-for-everything). Stage-6 bootstrap → **validate on the
  interpreted `myc` first, then AOT-compile and validate that** — both runtimes run the identical
  `.myc` and must agree (stage-2 three-way). Unblocks the M-740 wave without mid-port re-deciding.
  Status stays **Draft**. (M-739/E18-1.)
- **2026-07-03 — §7 + §8 added: concrete port order (M-739; append-only, no status move).** Filled out
  §4's advisory sketch with a **grounded** staged port order derived from the measured
  `crates/mycelium-l1/src/` dependency graph. Key finding: `checkty`/`elab`/`eval`/`mono`/`fuse`/
  `decision`/`usefulness`/`grade`/`affine` form **one SCC** — the semantic core ports as a single
  nodule-wide `FixGroup` (Stage 5), not file-by-file; `ast`/`ambient`/`totality`/`substrate` are leaf
  deps that port first; `token`/`lexer`/`nodule`/`parse` are the clean Stages 1–3. Self-hosted
  frontend lives in a new `lib/compiler/` phylum; the L0 kernel (`mycelium-core`/`interp`/`cert`/
  `select`) stays Rust (KC-3), reached via `@std-sys` + `wild` (DN-14 row 9, executes). Resolved all
  six §5 open questions; two architecturally-significant choices (Stage-5 nodule-vs-phylum packaging;
  Stage-6 bootstrap driver) `[FLAG]`ged for M-740/M-742, not pre-decided (G2). Design-only — no code
  changed. Status remains **Draft** (→ Resolved with M-741, house rule #3 / VR-5). (M-739; E18-1.)
- **2026-06-25 — §1 freshness note (append-only; no status move).** Per an alignment audit, noted
  that DN-14 row 9 (`wild`/FFI) has since flipped from "conditionally present" to "present/executes"
  (RFC-0028 / M-720–M-721), so §1's "all 11 … conditionally present" reads "all 11 present." Status
  remains **Draft**. (Append-only; VR-5; G2.)
- **2026-06-23 — Draft stub created.** Scope, user stories, decision space, tentative stage
  sketch, and open questions captured as a planning stub. Feeds E18-1 (M-739). Built on DN-14
  (Resolved) and ADR-021 (Accepted). Decides nothing normatively. Status: **Draft** (VR-5 /
  house rule #3).
