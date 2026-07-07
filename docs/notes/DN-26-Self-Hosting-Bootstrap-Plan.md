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

## 10. L0 `Value`/`Repr`/`FieldSpec` boundary decision (M-1012) — Option A (in-language mirror) adopted

> **Posture (append-only; VR-5/G2).** This section records the **maintainer's decision** on the
> architecturally-significant `[FLAG]` §7.2 named and M-1012 carried — how the self-hosted
> `semcore.myc` elaborator represents the kernel L0 `Value`/`Repr`/`FieldSpec` types at the
> frontend/kernel seam. It fulfils **M-1012's Definition of Done** ("a recorded decision, appended to
> DN-26, selecting the L0 `Value`/`Repr` representation") and **unblocks M-1013** (increments 10 + 12
> build against this model). It **decides nothing new about the kernel** and **adds no code**; like §9
> it fixes a branch point so the port can proceed. DN-26's status is **unchanged — Draft** (→ Resolved
> with M-741, house rule #3). Grounded in `docs/planning/semcore-l0-boundary-dossier.md` (the M-1012/
> M-1013 decision-support dossier, `Declared` analysis + recommendation).

**Decision — adopt Option A, the in-language mirror model.** The kernel L0 vocabulary
(`mycelium_core::{Value, Repr, FieldSpec, FieldTyRef, PolicyRef, ScalarKind, SparsityClass, Payload}`)
is declared as **plain Mycelium ADTs in `semcore.myc`**, mirroring the Rust kernel field-for-field
(the dossier §2.1 prefix scheme, disjoint from the existing `Ty`/`Wd`/`Mp`/`Hd` families), and the pure
lowering helpers (`lit_value`/`type_repr`/`field_spec`/`ty_to_repr`/`ty_to_field_ty_ref`/`scalar_kind`/
`sparsity_class`) construct **those mirror values**. The self-host ≡ Rust-host differential compares the
mirror's **canonical rendering** (serialization / content-hash) against the Rust oracle's real kernel
value (dossier §2.2 — a genuine independent comparison, the `bytes_eq`/content-hash posture increment 4
already uses). This is the same move `semcore.myc` already made one layer up (FLAG-semcore-1/-2 mirror
`checkty.rs`'s `Ty`/`DataInfo`); Option A extends it one layer down to the kernel's `Value`/`Repr`. This
takes the dossier's **§5 recommendation** ("Option A — the in-language mirror model — with the `wild`
seam reserved for the *execution/materialization* boundary only, never for per-descriptor
construction").

**The `wild` seam is reserved for the single materialization crossing only.** The `@std-sys` + `wild`
FFI seam (DN-14 row 9; §7.2) is used **only** to hand the *finished* L0 program to the trusted kernel to
run it (the dossier §5 "hybrid" — one `wild:` prim that takes the serialized mirror `Node` wire form and
asks the kernel to deserialize it into a real `mycelium_core::Node` for execution/AOT). It is **not**
used as a per-descriptor constructor factory — Option B was declined because the `wild`/prim seam is
`Value`-currency by construction (`Repr`/`FieldSpec` are descriptors *of* a `Value`, not `Value`s), its
return-type story for structured non-`Value` objects is **unresolved** in the corpus (dossier §3.3.1,
flagged G2), opaque handles would forfeit the self-hosted frontend's in-language inspectability (house
rule #2), and comparing kernel-minted handles to the same kernel would weaken the Stage-5 differential
toward circularity (§3.3.3). The mirror's standing drift/duplication tax (dossier §2.3/§5, the strongest
argument *against* Option A) is **bounded** — append-only, frozen-tag, never-silent (drift ⇒ a red L0
differential, not a wrong answer) — and is **further bounded by ADR-042** (the Rust-base freeze: freezing
new kernel/Rust types means the mirror only ever chases a frozen, append-only target, materially
weakening the duplication cost that would have tipped the balance toward B if the L0 vocabulary were
expected to churn).

**Wild-free-first refinement (ADR-042).** Per ADR-042's **wild-free-core goal** — `wild`/FFI minimised,
used only where strictly necessary — **even the single materialization crossing is to be investigated
for a wild-free path** before it is spent: prefer a safe in-language route to hand the L0 across the seam
if one exists, and reserve `wild` for where it is genuinely irreducible (the kernel must ultimately run
the L0, and DN-14 row 9's `wild` callback may prove to be that irreducible floor). This is a **`Declared`
target**, not a proven-reachable state (VR-5): the materialization crossing may turn out to require
`wild` without moving the DN-39 kernel boundary (forbidden) — the decision sets the *preference*, and the
residual `wild`-site count is tracked toward zero, each site audited (`just safety-check`) and
never-silent (G2). **Horizon note (ADR-042 end-state):** this "materialize the finished L0" crossing is a
frontend↔kernel **FFI seam only while the kernel is Rust**. Under ADR-042's end-state — the kernel itself
rewritten to `.myc`, zero foreign first-party languages by the DN-88 decomposition gate — the crossing is
**no longer a cross-language FFI seam**; it becomes an **internal concern of the self-hosted `.myc`
kernel**, and its wild-vs-wild-free status folds into the kernel-rewrite / bootstrap-trust design (ADR-042
§6 OQ-1). Per ADR-042 the only foreign residue expected in the terminal fully-native state is the
**irreducible OS/hardware ABI seam** (syscalls, hardware intrinsics) — the one place minimal `wild` is
expected to survive. See ADR-042 §2.4 / §3 / OQ-1.

**`policy_name_ref`'s hash (dossier §4, decided with the boundary).** `policy_name_ref` produces a
`PolicyRef` via a domain-separated content hash (`operation_hash("policy-name.v0:…")`), a narrow instance
of the same A/B question. Resolution, consistent with the wild-free-first refinement: **re-implement the
domain-separated hash in-language** if it is cheap to match the kernel byte-for-byte (keeping `PolicyRef`
a mirror value everywhere, no `wild`), **else** a single `wild:` hash prim — but the `PolicyRef` stays a
mirror value in all other positions either way. The in-language re-implementation is preferred where it
matches byte-for-byte (wild-free); a mismatch risk is never-silent (every `PolicyRef` would diverge in
the differential, G2).

**Landing order (dossier §4, boundary-independent).** The two arithmetic-free enum→enum helpers
`scalar_kind` and `sparsity_class` land **first** — they construct no kernel value (only the small mirror
`ScalarK`/`SparsityC` enums under Option A), so they de-risk increment 7 without prejudging anything.

**FLAG carried forward — the evaluator leg likely needs the AOT path, not the interpreter (dossier §7
FLAG-4).** Independent of this boundary pick, the self-hosted **evaluator** run *inside* the L1 evaluator
"almost certainly cannot complete at today's cost model" (M-986 TCO gap + M-987 ~n³ eval cost; §9 flag-2;
the M-994 epicenter). Increment 12 (eval) and increment 14 (the whole-program L0 differential) may need
the **AOT leg** rather than the interpreted one per §9 flag-2. This firms up only as increments 8–11
land; recorded here (not silently pre-scoped) so M-1013's `needs-design` eval-feasibility question stays
visible (G2/VR-5). The stale-status observations in dossier §7 FLAG-5 (M-1007 `in-progress` but DONE;
M-1011 `todo` but partially landed) are **FLAGged to the orchestrator** for reconciliation against the
codebase (mitigation #14), not changed here.

**This fulfils M-1012's DoD** (a recorded decision appended to DN-26 selecting the L0 representation) and
**unblocks M-1013** (the elab core + eval legs now have a fixed L0 model to build against). DN-26 status
stays **Draft**.

### 10.1 Implementation note (M-1012) — the differential method actually adopted

> **Posture (append-only; VR-5/G2).** This subsection does not alter the Option-A decision above — the
> in-language mirror model stands as decided, unchanged. It corrects and HONESTLY records the specific
> comparison **method** the M-1012 landing (increment 7, `compiler_stage5_elab.rs`) actually implements,
> which differs from this section's original phrasing above ("the mirror's canonical rendering
> (serialization / content-hash) against the Rust oracle's real kernel value"). That phrasing described
> a *target posture*; the shipped differential reached the same independence goal by a simpler route.
> Downgrading a `Declared`-but-inaccurate description to an accurate one is what VR-5 requires — never
> silently leaving the corpus saying something the code doesn't do.

**Method actually implemented — source-text encoding + `.myc`-side structural equality, not
serialization/content-hash.** For each fixture: (1) the REAL Rust oracle (`elab::lit_value`/
`type_repr`/`field_spec`/`ty_to_repr`/`ty_to_field_ty_ref`/…) runs on the fixture, producing a genuine
`mycelium_core::{Value,Repr,FieldSpec,…}`; (2) that Rust value is encoded — by a small family of Rust
`encode_core_*` functions living in the test file, never by `serde` or a content-hash — directly to
**`.myc` SOURCE TEXT**: a literal expression in the mirror ADTs (e.g.
`Ok(Val(RBinary(4), PlBits(Cons(0b1, …)), MtExactRoot))`); (3) that literal is spliced into a driver
program alongside a call to the `.myc` port's own helper (e.g. `lit_value(Bin("1010"))`), evaluated
independently; (4) BOTH `.myc`-side values are compared by hand-written `.myc` structural-equality
functions (`value_eq`/`repr_eq`/`payload_eq`/… — FLAG-semcore-28) evaluated **inside the L1
interpreter**, yielding a `Bool`; (5) the Rust harness (`assert_l1_only_u32`) asserts that `Bool` against
the expected `True`/`False`. No canonical rendering, serialization, or content-hash of either side is
computed at any step — the comparison is a plain in-language structural equality over two
independently-constructed mirror values.

**Trust properties (honestly bounded, not assumed).**

- **WF7 (checked-exhaustive `match`, RFC-0011 §4.3 / RFC-0001) guards the MISSING-variant class.**
  Every `_eq` comparator is a flat `match` over a mirror ADT, and the `.myc` checker refuses to compile
  a `match` that omits a constructor of the scrutinee's type (no silent fallthrough is expressible). A
  comparator that forgot to handle an entire `Repr`/`Payload` variant is a compile-time error, not a
  silently-wrong differential — this class is bounded `Proven` (checked by the compiler, not merely
  argued).
- **WF7 does NOT guard the PRESENT-but-WRONG-arm class** (an arm that compiles, covers its constructor,
  but compares the wrong field or is miswired to always agree) — that residual risk is exactly what the
  M-1012 patch's non-vacuity discipline (`compiler_stage5_elab.rs::elab_witness_discriminates` — every
  introduced `_eq` comparator gets at least one must-return-`False` case, isolating the one dimension its
  arm guards) exists to bound. The two mechanisms are complementary, at different strengths: WF7 bounds
  the missing-arm class structurally (`Proven`); the non-vacuity discipline bounds the wrong-arm class
  empirically (`Empirical` — a finite, isolating case set, not an exhaustive proof over every possible
  wrong wiring).
- The `.myc`-side comparators are themselves hand-written, not derived — they restate BY HAND the Rust
  kernel's derived `PartialEq` (FLAG-semcore-28). Their correctness is bounded by this differential's own
  case coverage, not independently checked against `mycelium_core`'s real `PartialEq` impls.

**Future option, recorded not adopted (an M-1013+ design input).** As ported output types grow
(recursive `Value::Seq`, `Hypervector`/dense payloads, a non-trivial `Meta`), the hand-written
`.myc`-side `_eq` family's trust surface grows with it — each new variant needs its own hand-verified
comparator arm plus a non-vacuity case. A heavier alternative for later, larger increments, kept here as
a documented option rather than adopted now: **harness marshalling** — decode the `.myc` mirror's
OUTPUT back into the real `mycelium_core::Value`/`Repr`/… (a `wild:` or `@std-sys` decode step, or a
pure-Rust decoder over the mirror's `.myc`-side rendering) and compare it to the Rust oracle's value
using Rust's own trusted `#[derive(PartialEq)]` — or compare `mycelium_core::Value` content-hashes
directly (the posture this section originally described above). Either route removes the hand-written
`.myc`-side `_eq` family from the trust path entirely, at the cost of a decode/marshalling seam. Not
needed at increment-7 scale (8 comparator families, each non-vacuity-cased this wave); tracked as an
M-1013 design input, not a commitment.

**→ Adopted at §10.2 (2026-07-07, M-1013 STEP 2).** The harness-marshalling option above is no longer
merely recorded: it is the **adopted** Stage-5 differential method as of the M-1013 STEP 2 retrofit.
This paragraph is kept as written (its increment-7-scale reasoning was accurate at landing);
§10.2 records the adoption and supersedes its closing "not a commitment" clause.

### 10.2 Marshalling adopted as the differential method (M-1013 STEP 2)

> **Posture (append-only; VR-5/G2).** This subsection does not alter the Option-A decision (§10) — the
> in-language mirror model stands. It records the maintainer's directive (2026-07-07) to **adopt harness
> marshalling** (the option §10.1 recorded but did not commit to) as the Stage-5 differential method,
> superseding §10.1's closing "tracked as an M-1013 design input, not a commitment." No status move
> (DN-26 stays **Draft** → Resolved with M-741).

**Adopted method — decode the port's mirror output to the real kernel type, compare with Rust's
trusted `==`.** For each fixture: (1) the REAL Rust `elab::*` oracle runs, producing a genuine
`mycelium_core::{Value,Repr,FieldSpec,…}`; (2) the `.myc` port helper is evaluated *directly* — the
driver's `main` returns the mirror value itself (e.g. `fn main() => Result[Repr,Bytes] =
type_repr(…);`), not a `Bool`; (3) a never-silent Rust **decoder** family (`decode_*` in
`compiler_stage5_elab.rs`) walks the resulting `L1Value` mirror ADT and rebuilds the real
`mycelium_core` type — the inverse of the mirror constructors, panicking on an unexpected constructor
(G2); (4) the decoded value is compared to the oracle's with **Rust's own derived `==`** (`assert_eq!`).
This **removes the hand-written `.myc`-side `_eq` family (the retired FLAG-semcore-28 comparators,
deleted from `semcore.myc`) from the trust path entirely** — the comparator is now `mycelium_core`'s own
`#[derive(PartialEq)]`, the very thing the mirror was hand-restating. The trust surface shrinks to the
decoder: small, one-directional, and guarded (below).

**Why now (the directive's rationale).** §10.1 bounded the old method's residual wrong-arm risk
`Empirical` via a per-comparator non-vacuity discipline, but flagged that the hand-written `.myc` `_eq`
family is *itself untrusted* (it restates the derived `PartialEq` by hand) and that its trust surface
*grows* with every ported output type. Marshalling is the trust foundation for the ~11 remaining M-1013
increments (dossier §6.3): it eliminates that growing hand-written surface up front rather than carrying
it through the heavy core. It is therefore the differential method for **all** M-1013 increments, not
just the increment-7 retrofit.

**The migrated non-vacuity discipline — decoder discrimination.** With the comparator now Rust's trusted
`==`, a wrong port output fails `assert_eq!` by construction, so §10.1's "the comparator isn't vacuously
`True`" obligation dissolves. The new trust surface is the decoder, whose failure mode is *dropping a
dimension* (mapping distinct mirror values onto one Rust value → a silent false pass). The non-vacuity
discipline **migrates** to the decoder: for every decoder arm, decode two mirror values differing in
exactly one dimension and assert the decoded Rust values are unequal
(`compiler_stage5_elab.rs::marshal_discriminates`). `Meta`/`FloatW` remain the documented
single-inhabitant exceptions (no distinguishing pair is constructible — each becomes an addable case the
moment it gains a second constructor). This is the binding template for M-1013.

**Comparison choice — derived `==` now, content-hash when floats arrive.** Every marshalling target
except `Value` derives `Eq`; `Value` derives only `PartialEq` (its `Payload` can hold `f64`), but this
increment's payloads are `Bits`/`Trits`/`Bytes` only and its `Meta` is the constant `exact(Root)` on
both sides, so derived `==` is total and identity-faithful — the honest, sufficient primary. Derived
`==` becomes unsound at the first float-bearing increment (`RFloat`/`RDense`/`RVsa` with a real `f64`
payload): a `NaN` payload is unequal to itself under `==` (failing `assert_eq!` against itself), and
signed-zero conflates bit-distinct identities. At that point the value comparison switches to
**content-hash equality** (`CoreValue::Repr(v).content_hash()`, which normalizes `NaN` and excludes
`Meta` — there is no `Value::content_hash`), the posture §10 originally described. Recorded here so the
switch is a planned, never-silent step (VR-5/G2).

## Meta — changelog

- **2026-07-07 — §10.2 added + FLAG-semcore-28 `_eq` family retired: harness MARSHALLING adopted as the
  Stage-5 differential method (append-only, no status move; M-1013 STEP 2).** Per the maintainer's
  directive, the differential now decodes the `.myc` port's mirror output back into the real
  `mycelium_core` type (a never-silent Rust `decode_*` family in `compiler_stage5_elab.rs`) and compares
  with Rust's own trusted derived `==` — removing the hand-written `.myc`-side `_eq` comparators (the
  retired FLAG-semcore-28 family, deleted from `semcore.myc`) from the trust path. The M-1012
  non-vacuity discipline migrates from the comparator to the decoder (`marshal_discriminates`:
  distinct-in-one-dimension mirror values must decode to unequal Rust values). Derived `==` is primary;
  content-hash is the recorded switch for the first float-bearing increment. Adopts §10.1's
  recorded-not-committed option and becomes the method for all M-1013 increments (dossier §6.3). Does not
  alter the Option-A decision. Status stays **Draft** (→ Resolved with M-741). (M-1013; E18-1; VR-5/G2.)

- **2026-07-07 — §10.1 added: an "Implementation note" honestly reconciling §10's described
  differential method with the one M-1012 actually shipped (append-only, no status move; M-1012 PR
  review patch).** §10's original text described the differential as comparing "canonical rendering
  (serialization/content-hash)" against the Rust oracle; the increment-7 landing
  (`compiler_stage5_elab.rs`) instead encodes the Rust oracle's value to `.myc` SOURCE TEXT and
  compares it to the port's own value with hand-written `.myc` structural-equality functions
  (`value_eq`/`repr_eq`/… — FLAG-semcore-28), evaluated in the L1 interpreter — no serialization or
  content-hash step anywhere. §10.1 records the actual method, its trust properties (WF7
  match-exhaustiveness bounds the missing-variant class `Proven`; the M-1012 patch's mandatory
  non-vacuity discipline bounds the wrong-arm class `Empirical`), and documents **harness
  marshalling** (decode to real `mycelium_core` types + Rust's derived `==`, or content-hash
  comparison) as a recorded-not-adopted option for heavier M-1013 increments. Does not alter the
  Option-A decision. Status stays **Draft** (→ Resolved with M-741). (M-1012; E18-1; VR-5/G2.)
- **2026-07-07 — §10 added: the L0 `Value`/`Repr`/`FieldSpec` boundary decided — Option A, the
  in-language mirror model (append-only, no status move; M-1012).** The maintainer picked the
  frontend/kernel L0-boundary `[FLAG]` M-1012 carried (§7.2): declare the kernel L0 vocabulary as plain
  Mycelium ADTs in `semcore.myc` (mirroring `mycelium_core` field-for-field), diffed against the Rust
  oracle by canonical serialization/content-hash, with the `@std-sys` + `wild` seam **reserved for the
  single "materialise the finished L0" crossing only** — never a per-descriptor constructor factory
  (Option B declined: `Value`-currency seam, unresolved non-`Value` return-type story, opaque handles
  forfeit in-language inspectability, differential circularity — dossier §3.3/§5). Per **ADR-042**'s
  wild-free-core goal, **even that crossing is to be investigated for a wild-free path** (`wild` only
  where strictly necessary; `Declared` target, not a proven state — VR-5). `policy_name_ref`'s hash
  decided the same way (re-implement domain-separated hash in-language if byte-for-byte cheap, else one
  `wild:` prim). `scalar_kind`/`sparsity_class` (the arithmetic-free set) land first, boundary-
  independent. FLAG carried: the self-hosted **evaluator** leg (increments 12/14) likely needs the AOT
  path, not the interpreter, at today's cost model (M-986/M-987; §9 flag-2). Grounded in
  `docs/planning/semcore-l0-boundary-dossier.md` (§5 recommendation + §7 FLAGs). **Fulfils M-1012's DoD;
  unblocks M-1013.** Stale-status observations (M-1007/M-1011, dossier §7 FLAG-5) FLAGged for
  orchestrator reconciliation (mitigation #14), not changed here. Status stays **Draft** (→ Resolved with
  M-741). (M-1012; E18-1; VR-5/G2.)
- **2026-07-07 — Stage-5 increments 3–6 landed (the pure/leaf cluster; append-only, no status move;
  M-993/M-1008/M-1009/M-1010/M-1011).** The tractable pure/leaf wave (E18-1) landed the four
  parallelizable increments into `semcore.myc`, each with a live-oracle Rust differential from an
  in-crate `src/tests/` module (no logic `*.rs` or visibility changed — only the new test modules +
  their `mod` lines): increment 3 (M-1008 — checkty `unify`/`resolve_ty` + the synthetic-tuple
  helpers `tuple_type_name`/`tuple_ctor_name`/`synthetic_tuple_data`, plus the `dec_u32` base-10
  renderer; `compiler_stage5_unify.rs`, 8 tests — the never-silent unification conflict path asserted
  `Err`); increment 4 (M-1009 — the mono name-mangling family `mangle_ty`/`scalar_tag`/`mangle_decl`/
  `mangle_ctor`/`mangle_method`/`mangle_arrow`/`apply_fn_name`/`mangle_ty_or_fn`/
  `mangle_decl_with_wargs`/`mangle_hof_decl`; `compiler_stage5_mangle.rs`, 7 tests — direct
  bytes-equality vs `mono::mangle_*`, injectivity boundary exercised); increment 5 (M-1010 — mono
  `free_vars`/`free_vars_at` + `pattern_binders`/`pattern_binders_at`; `compiler_stage5_freevars.rs`,
  4 tests — the mutable scope/output accumulators collapsed to threaded values, shadowing preserved);
  increment 6 (M-1011 — checkty `lit_ty_of`/`literal_key`/`normalize_pattern`;
  `compiler_stage5_normpat.rs`, 5 tests — the normalized matrix `Pat` + binder occurrences asserted,
  feeding the increment-1 `useful` gate transitively). Native `myc check` `ok`; 47 Stage-5 tests
  total green. Deliberate, documented deviations (never silent — G2/VR-5): FLAG-semcore-13
  (`unify`'s `&mut BTreeMap` → threaded assoc-list), -14 (a vestigial dead `unify` arm collapsed),
  -15/-16 (`dec_u32` introduced in incr 3, reused in incr 4), -17 (`mangle_ty_in_ty`/`item_key`
  DEFERRED — module-private, no reachable oracle, not ported rather than land un-witnessed), -18
  (free-vars scope value-threading), -19 (`literal_key` `Int`/`AmbientInt` keys simplified —
  unreachable dead arms), -20 (`infer_type` DEFERRED — a wrapper over the un-ported inference engine,
  not a leaf), -21 (`normalize_pattern`'s binder accumulator threaded). Also folded in the two PR
  #1231 review nits (top-level concrete arms in the typealg `has_var`/`subst_ty` cases). Status stays
  **Draft** (→ Resolved with M-741). (M-993; E18-1; VR-5/G2.)
- **2026-07-07 — M-993 staged port plan (§7.3 Stage-5 interior) + Stage-5 increment 2 landed
  (append-only, no status move; M-993/M-1007).** The semcore heavy-core port (M-993, now
  `in-progress`) is a multi-wave effort, not one shot: the ~14k-line SCC remainder decomposes into
  twelve dependency-respecting increments (3..14). Because the SCC is one nodule (§9 flag-1), mutual
  recursion is free within `semcore.myc` (FixGroup), so the sequencing constraint is not "compile
  independently" but three real gates: (a) data-model prerequisites (a cluster referencing a
  not-yet-modeled value/IR type waits, or its wave introduces that model); (b) differential
  feasibility (each cluster needs a reachable live Rust oracle — `pub`/`pub(crate)` from an in-crate
  `src/tests/` module — plus encodable I/O); and (c) the M-986/M-987 kernel cost/depth walls (lifted
  by M-994, but still acute for the whole-nodule checkty leg and the eval leg). The increments split
  into a tractable pure/leaf cluster (parallelizable, ~200–400 Rust lines each, clean live-oracle
  differentials) and a heavy sequential core. Tractable waves, minted under E18-1: increment 3
  (M-1008, checkty `unify`/`resolve_ty`/tuple helpers); increment 4 (M-1009, the mono name-mangling
  family plus a `u32`→decimal helper); increment 5 (M-1010, mono `free_vars`/`pattern_binders`);
  increment 6 (M-1011, checkty literal/pattern typing); increment 7 (M-1012, the L0 `Value`/`Repr`
  model plus elab pure lowering helpers — carries the §7.2 frontend/kernel-boundary `[FLAG]`,
  `needs-design`). Heavy sequential core, tracked as M-1013 (`needs-design`, sub-decomposed as the
  leaves land): increment 8 (checkty registration and the `Env` model), increment 9 (the
  bidirectional checker `Cx`), increment 10 (elab AST→L0 lowering), increment 11 (mono core),
  increment 12 (the value-semantics `eval` restatement — the acute M-994 feasibility risk, may lean
  on the AOT leg per §9 flag-2), increment 13 (`fuse`, which runs the ported evaluator), and
  increment 14 (the whole-program L0-output differential plus the `cargo-mutants` witness — the §7.3
  Stage-5 gate proper, feeding M-741). Increment 2 (M-1007) landed this wave: checkty's four pure
  type-algebra leaves (`has_var`/`type_head`/`subst_ty`/`param_subst`) into `semcore.myc`, native
  `myc check` `ok`, a live-oracle Rust differential (`compiler_stage5_typealg.rs`, 6 tests / ~46
  cases; no logic `*.rs` touched; documented in-file as FLAG-semcore-11/12). No `crates/mycelium-l1/
  src/` logic or visibility changed (only the in-crate `src/tests/` module + its one `mod` line).
  Status stays **Draft** (→ Resolved with M-741). (M-993; E18-1; VR-5/G2.)
- **2026-07-06 — M-994 RESOLVED: both kernel fixes landed; interpreted-first Stage-6 now practical
  (append-only, no status move; M-740/M-994).** Following the decision below, **both** fixes are on
  `dev`: **(a)** the RFC-0041 §4.6 TCO-precondition widening (M-986 → done — deep `match`/`let` loops
  now exceed the 4096 budget) and **(b)** O(1) `L1Value::Data` clone via `Arc` structural sharing
  (M-987 → done — measured ~n³→~n², p 2.96→~1.9, 14×/30×/64× at n=100/200/400; the M-210 differential
  stayed green and UNCHANGED, so it landed through the RFC-0041 §6 behavior-preserving channel). The
  DN-26 §9 flag-2 **interpreted-first Stage-6 gate is therefore practical** at compiler scale; (c) AOT
  remains the fallback for inputs beyond (a)+(b)'s reach, not a substitute for the interpreted witness.
  **M-994 → done.** The eval-side blocker on the semcore heavy-core port (**M-993**) is cleared — it
  now depends only on the port work itself + M-741. DN-26 stays **Draft** (→ Resolved with M-741).
  (M-994; E18-1; VR-5/G2.)
- **2026-07-06 — M-994 DECIDED: fix the kernel (a) then (b); interpreted-first Stage-6 becomes
  viable (append-only, no status move; M-740/M-994).** The open question recorded in the entry below
  (the §9 flag-2 interpreted-first Stage-6 gate impractical at compiler scale under M-986/M-987) is
  **now decided by the maintainer** (2026-07-06), on the basis of an investigation + spike: **land (a)
  then (b), keep (c) AOT as the fallback.** **(a) — landed:** widen the L1 evaluator's TCO precondition
  to see through tail-transparent `MatchPop`/`LetPop` frames (`eval.rs::enter_call`, ~47 LOC), an
  append-only **RFC-0041 §4.6 amendment** completing that section's ratified TCO intent, maintainer-
  signed-off via the §6 within-freeze channel (it shifts the runs-vs-refuses frontier, but there is no
  L0 oracle for these deep loops to diverge from — L0 has no TCO — and the M-210 differential +
  fingerprint parity are unchanged). This closes **M-986** (deep `match`/`let` loops now exceed the
  4096 budget; the two pins flipped to assert the closed behavior; a non-tail self-call still refuses,
  proving no over-elision). **(b) — next:** make `L1Value::Data` clone O(1) via `Rc` structural sharing
  (sound: `Data` is immutable+acyclic by construction) — the confirmed root of **M-987**'s ~n³ (every
  variable reference deep-copies an O(nodes) value); expected ~n³→~n², behavior-preserving so it lands
  through the §6 channel proper (M-210 differential its bar). **Consequence for this plan:** (a) unlocks
  the *depth* half and (b) the *cost* half; **together they make the §9 flag-2 interpreted-first Stage-6
  gate practical** (the two were shown complementary — with (a) alone an 800-item parse now runs but is
  ~n³ slow). Once both land, **M-993** (the semcore heavy-core port) is unblocked on the eval side;
  (c) AOT stays the fallback for inputs beyond (a)+(b)'s reach, not a substitute for the interpreted
  witness. DN-26 stays **Draft** (→ Resolved with M-741). (M-994; E18-1; VR-5/G2.)
- **2026-07-06 — Stage 5 increment 1 landed (partial `semcore.myc`); the heavy-core L0
  differential is feasibility-gated — an OPEN QUESTION, not decided (append-only, no status move;
  M-740).** §7.3 row 5 (`compiler.semcore`, §9 flag-1's single-nodule SCC) begins as a **partial**
  increment: the type vocabulary (`Ty`/`Width`/`DataInfo`/`CtorInfo`/`Pat`, data only) + the
  Maranget `usefulness`+`decision` pipeline + `affine` + `grade` — the sub-core depending on
  checkty's *types*, not its logic or the evaluator. Gate
  `crates/mycelium-l1/src/tests/compiler_stage5_semcore.rs` 17/17 (`Empirical`): a **true
  live-oracle** in-crate differential (white-box `use crate::usefulness/decision/affine/grade`
  against the real Rust functions on small synthetic inputs — not hand-derived; perturbation-checked
  to fail loudly). Native `myc check` = `ok`. No `crates/mycelium-l1/src/` logic or visibility was
  changed. **This is the point where the §9 flag-2 *interpreted-first* Stage-6 gate's practicality
  (flagged 2026-07-05) actually bites.** Deferred, feasibility-gated on M-986 (the evaluator elides
  only bare-body tail calls → no in-language loop exceeds the 4096 depth budget) and M-987 (~n³
  L1-eval cost): the heavy entangled core `checkty`/`elab`/`eval`/`mono`, `fuse` (which *runs* the
  evaluator), the **whole-program L0-output differential**, and the `cargo-mutants` witness. A
  self-hosted checker/elaborator run inside the L1 evaluator over a whole program almost certainly
  cannot complete under the current kernel — so the DN-26 §7.3 Stage-5 "L0-output differential" as
  specified is **not achievable at compiler scale today**. This note **decides nothing**: whether
  the lift comes from widening the kernel's tail position (M-986), reducing eval cost (M-987), or
  leaning on the AOT leg for the Stage-5/6 gate is a **maintainer decision**, recorded here rather
  than silently re-scoped (G2/VR-5). Minted: **M-993** (heavy-core port), **M-994** (the
  L0-differential-feasibility question). Status remains **Draft** (→ Resolved with M-741). (M-740
  Stage 5 increment 1; E18-1; VR-5/G2.)
- **2026-07-06 — Stage 4 landed: the three SCC leaf nodules (append-only, no status move;
  M-740).** §7.3's leaves — `compiler.substrate`/`compiler.totality`/`compiler.ambient` (the `.myc`
  port of `substrate.rs`/`totality.rs`/`ambient.rs`) — implemented and gated (Rust differentials
  `compiler_stage4_substrate.rs` 5/5 · `_totality.rs` 6/6 · `_ambient.rs` 4/4, `Empirical`). All
  three depend only on `ast` (or nothing), so the entangled semantic core is untouched. **New
  capability this wave: the native toolchain now vets the self-hosted path.** `myc check` (the real
  `mycelium-check` binary) reports `ok` on all three new nodules *and* the five prior ones
  (`token`/`lex`/`nodule`/`ast`/`parse`) — a second, independent witness alongside the Rust
  differential; `mycfmt` parses all eight but reports them non-canonical and *refuses* two
  (`lex.myc`/`parse.myc`) on the M-690 formatter limitation. `lib/compiler/` is not yet a
  `mycelium-proj.toml` project root, so `just myc-check`/`myc-fmt` skip it — to be lifted by a
  repeatable dual-tooling (old Rust differential + new `myc`) parity gate (`/myc-dogfood`), scoped
  to **light** checks (heavy VSA / GPU-bound work routes to local via session teleport, never the
  remote gate). Honest limits recorded in-file: substrate's `Arc<AtomicBool>` cross-alias
  consume-once is **not representable** in a pure-value port (FLAG-substrate-1, documented not
  faked); totality narrows `BTreeMap`/`BTreeSet`→sorted assoc-list with a deterministic-order
  precondition and specializes the `&mut impl FnMut` walks (FLAG-totality-1/2); ambient's
  differential covers 8 hand-built fixtures + 4 refusals but **no raw corpus files** — a
  *structural* limit (it consumes an already-parsed `Nodule` and cannot reach `compiler.parse`
  cross-nodule per M-982; a source file needs an AST-serializer bridge, deferred), **not** the
  M-987 cost wall (FLAG-ambient-6). Status remains **Draft** (→ Resolved with M-741). (M-740
  Stage 4; E18-1; VR-5/G2.)
- **2026-07-05 — Stage-3 review cycle: three kernel-side findings bear on this plan's later
  stages (append-only, no status move; M-740 / PR #1166).** The review-cycle patch converted all
  27 source-length-bounded loops in `parse.myc` to accumulator+reverse **direct-tail shape**
  (RFC-0041 §7 W7 amendment 11) and surfaced: **M-986** — the L1 evaluator's TCO elides only
  bare-body self-calls (tail calls inside `match`/`let` are never elided), so **no in-language
  loop can iterate past the 4096 depth budget today**; **M-987** — L1-eval cost is ~n³ in input
  token count (0.6 s → 26 s → 133 s at 200 → 752 → 1,252 tokens, debug build); **M-988** — mono
  re-inference rejects generic bare `Nil` the checker accepted. Consequence for this plan, stated
  plainly (VR-5): until M-986/M-987 lift, the §9 flag-2 **interpreted-first** Stage-6 bootstrap
  gate is impractical at compiler scale under L1-eval, and Stage-4/5 differential legs face the
  same per-file cost wall the Stage-3 lib leg hit (6-file subset, M-984). This note **decides
  nothing** — flag-2's interpreted-first intent stands; whether the lift comes from the kernel
  TCO widening, evaluator cost work, or leaning on the AOT leg is an open M-741/M-742-adjacent
  question, recorded here rather than silently re-planned (G2). Status remains **Draft**.
  (M-740 Stage 3 review cycle; E18-1.)
- **2026-07-05 — Stage 3 landed; two namespace findings recorded (append-only, no status move;
  M-740).** §7.3 row 3 (`compiler.ast` + `compiler.parse`) is implemented and gated: both `parse`
  and `parse_phylum` ported end-to-end, classification parity with the Rust oracle over the full
  conformance corpus on both legs (accept 27/27, reject 30/30, zero divergences) plus a preorder
  constructor-tag AST fingerprint on every accepted leg (`Empirical`; L1-eval leg only per M-981;
  gates `compiler_stage3_ast.rs` 26/26 + `compiler_stage3.rs` 4/4). Two findings bear on this
  plan's later stages, recorded for the §7.3 Stage-5 semcore packaging (§9 flag-1): **(a)** the
  per-nodule constructor namespace is **flat** — bare variant names reused across *different*
  enums collide even when none is a reserved word (ast.myc FLAG-ast-5; resolved with per-type
  prefixes, the `collections.myc` precedent); **(b)** combining two frontend stages in one nodule
  additionally collides the lexer's keyword-constructor family with AST constructors (parse.myc
  FLAG-parse-2, 31 names) — so the semcore nodule, which merges nine Rust modules, must budget
  for a systematic prefixing discipline up front. The §7.4 self-containment workaround (M-982)
  continues to hold through Stage 3 (parse.myc carries its own token+lexer+AST copy,
  FLAG-parse-1). Follow-ons minted: M-983 (one-eval differential-harness retrofit), M-984
  (full lib-tree parser sweep once the M-981 economics lift). Status remains **Draft**
  (→ Resolved with M-741). (M-740 Stage 3; E18-1; VR-5/G2.)
- **2026-07-05 — Stage-2 naming correction recorded (append-only, no status move; M-740).** The §7.3
  Stage-2 nodule name `compiler.nodule` is **unspellable** in surface syntax: `nodule` is a reserved
  word (DN-02), so the declaration `nodule compiler.nodule;` cannot parse — the second path segment
  lexes as the `Nodule` keyword, never an identifier (the token.myc FLAG-token-3 keyword-collision
  class surfacing at the nodule-NAME level, discovered by the Stage-2 port itself). The stage ships
  as **`compiler.nodule_header`** (`lib/compiler/nodule.myc`; FLAG-nodule-5 in-file). Note the
  faithfully-preserved asymmetry: the comment-MARKER grammar (`parse_nodule_header`) accepts
  `nodule` as a dotted-path segment — it validates raw identifier text, keyword-blind, exactly like
  the Rust oracle — only the surface `nodule <dotted>;` declaration cannot spell it. §7.3's row is
  not rewritten (append-only); read its `compiler.nodule` as `compiler.nodule_header`. Status
  remains **Draft**. (M-740 Stage 2; E18-1; VR-5/G2.)
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
