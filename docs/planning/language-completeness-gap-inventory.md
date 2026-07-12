# Language-Completeness Gap Inventory — the drive-hard worklist for full native expressibility

| Field | Value |
|---|---|
| **Status** | **Draft — living planning document** (2026-07-12), register style (DN-99 / the zero-hand-port delta ledger). Updated in place; **append rather than overwrite** when a figure changes materially (VR-5 — keep the prior figure visible with a date). It **recommends, does not ratify** (house rule #3) and **moves no other doc's status**. |
| **Grounding** | Read against the worktree tip `30ddfa41` (which contains `origin/dev` `fa53dc46`, the current dev tip). Draft-corpus numbers from the committed `gen/myc-drafts/manifest.json` (generation commit `eb6bc0e2`, a 13-target oracle-mode run) and the DN-122 §8.22 / DN-124 **phylum-mode** baseline (`06b4d7a7`). Every `file:line`/`M-id`/measurement is `Empirical`; every closure design not yet built/ratified is `Declared` (VR-5). |
| **Supersedes (as the current synthesis)** | The gap-class splits and `checked_fraction` figures in `zero-hand-port-delta-ledger.md` §2/§6 (pre-enb-wave, oracle, 812-gap corpus), DN-121 §3, and DN-119 §3/§4 — **re-derived here against the current tree** (mitigation #14: the register has lagged all session; §5 records what was stale). |
| **Frame** | The gate is **not** "express every Rust construct verbatim." It is: for every source-language expression, Mycelium has a **native way to solve the PROBLEM that expression solves**, classified by the ratified **DN-111** taxonomy (**Native Equivalent · Idiomatic Remapping · Approximation · Interop Bridge**). A construct with a ratified native solution — even a different mechanism — is **not a gap**; a problem Mycelium cannot yet natively solve **is**. Reasoning is kept source-language-general so it carries forward to **Python** (each row tags Rust-specific vs general). |

> **Honesty (house rule #4 / VR-5 / G2).** This doc synthesizes; it takes no decision. The central,
> non-sycophantic finding is that **most of the historical "gap mass" is already solved or is a
> measurement artifact** — the genuine, drive-hard language residual is a **small, well-bounded set**,
> and its single largest unsolved piece (`&mut self`/`&mut T` value-semantics mapping) is **un-owned by
> any design note today**. Where an existing analysis's number or lever-ranking is stale, it is corrected
> here on the evidence, even where that cuts against a prior doc's framing.

---

## §1 The classification lens (DN-111, Accepted 2026-07-10)

Every candidate gap is scored on two things: **(a)** does the underlying problem already have a *native
answer*, and of which DN-111 class; **(b)** is that answer **build-ready** (a ratified design exists,
only the build remains) or **design-gated** (needs a Draft DN + ratification first).

- **Native Equivalent** — a first-class native construct fills the role directly (exact, structure-preserving). DN-99 `already-closed`; DN-109 *Mechanical*.
- **Idiomatic Remapping** — same problem, **different** native form/convention (exact, reformed). DN-99 `idiom`; DN-109 F1 / D-classes.
- **Approximation** — a close-but-not-exact native form with the residue **made explicit** (VR-5). DN-99 `idiom` + caveat; DN-109 *Heuristic*.
- **Interop Bridge** — no full native form yet; carry the content across a **marked** `wild`/FFI edge (temporary, never-silent). DN-99 `open`/`transpiler-only` flagged; DN-109 *Judgment*.
- **Genuine gap** — the problem has **no** ratified native answer yet (the only rows that count as real language gaps).

A construct's cell is a `(construct, context)` pair and is **time-indexed** under the unfrozen kernel
(DN-111 §8.1/§8.4) — a "gap" today can be a Native Equivalent after one enabler lands.

---

## §2 The current measured baseline (`Empirical`, re-derived — mitigation #14)

**`checked_fraction` is ~7–8%, and every prior headline number is within measurement-window noise of
that** — do **not** treat any single figure as precise:

| Basis | Figure | Source |
|---|---|---|
| **Phylum-mode union** (the *more correct* basis, DN-124) | **~7.4%** | DN-122 §8.22 WU-A baseline, `06b4d7a7` (`Empirical`) |
| Oracle-mode union, 13-target run | **6.64%** (51/768) | `gen/myc-drafts/manifest.json`, gen `eb6bc0e2` (`Empirical`) |
| Oracle-mode union, 17-target run | 8.18% (59/721) | prior 17-target corpus regen (`Empirical`) |
| Pre-enb-wave (delta-ledger headline) | 7.8% | `zero-hand-port-delta-ledger.md` §1 — **now context, not current** |

**Measurement-basis is itself a lever (DN-124, Accepted 2026-07-12).** Oracle (single-file) `myc check`
has **zero phylum-import visibility**, so a correctly-resolved cross-nodule `use` FAILS oracle mode even
though it is CLEAN under `myc check --phylum`. This makes the whole **Import / external-trait /
cross-phylum-record** family **un-creditable even when the transpiler emits correctly**. Switching the vet
path to phylum-mode will make `checked_fraction` **jump** — that jump is a **basis correction, not lever
progress** (DN-124 M-A: dual-report one cycle, then re-baseline with `Δ_basis` attributed).

**Current gap-class distribution** (`Empirical`, worktree manifest `eb6bc0e2`, 935 gap instances;
robust in *ranking* across both corpus runs, ±1–2 pts in share):

| Class | Share | Note vs the delta-ledger's stale split |
|---|---|---|
| Other / type-coverage | **~23–24%** | **was 40%** — the class **shrank** as `&T`-erasure, DeriveAttr, records split out (STALE-corrected, §5) |
| Import | ~14% | largely a **measurement artifact** under oracle mode (DN-124) |
| Impl | ~12–13% | on this corpus these are **method-BODY** failures (`&mut self`/`write!`), **not** external-trait-def shapes (§4, §8.22) |
| **DeriveAttr** | **~11–12%** | **NEW prominence** — not in the ledger's top-class list; now the 4th-largest (§6) |
| Struct / records | ~6% | + NamedFieldDrop ~5% (faithfulness, DN-123) |
| GenericBound | ~5–6% | bounded-generic surface (M-876) |
| ReservedWord | ~5% | idiom (prefix-rename) — mostly `Default`/`default` collisions (§8.22 finding 3) |
| ModuleDecl | ~4% | distinct class now; runtime cross-nodule exec (M-982/M-1024) |

**DN-99 register tally (`Empirical`):** of 92 enumerated surface gaps, **66 are already closed** (16 landed
+ 50 sanctioned idiom), **10 need only transpiler work**, and **16 carry a genuine language/runtime
residual** (4 `open` + 12 `partial`). The drive-hard worklist below is the *language* residual, re-scored
against decisions that have landed since DN-99 was written.

---

## §3 The prioritized drive-hard worklist (top gaps, ranked by leverage × buildability)

Ranked so the maintainer can "drive hard on all of them" with a clear design-first vs build-now split.
**Leverage** is file-gating-aware (a class that poisons whole files outranks one that gaps a single item).
All leverage figures are `Declared` until the Phase-0 phylum-mode re-measure (DN-124) — **no numeric
target may be committed before it** (VR-5).

| # | Gap (the PROBLEM) | Layer | DN-111 native class | Design status | Owning DN / M-id | Leverage | Src-lang |
|---|---|---|---|---|---|---|---|
| **1** | **External-trait impls** (`impl ForeignTrait for LocalType` — trait-instance across the home boundary) | semantics / checker | **Native Equivalent** (foreign-trait import + closure-extended coherence) | **BUILD-READY** — **DN-122 Accepted 2026-07-12**, §13 prelude-scoped MVP | DN-122; M-1080 *(branch landed, issue needs filing)*; M-876; deps M-1036 ✓, M-1060 | **High** (`Declared`; poisons whole files) — but see caveat ▼ | general (Python: protocols/ABCs) |
| **2** | **`&mut self` / `&mut T` method receiver + parameter** (in-place mutation of a value) | **semantics** | **Idiomatic Remapping** (value-threading / return-new-`Self`) — *mechanism unspecified* | **DESIGN-GATED — UN-OWNED** (fresh DN needed; **being scoped in parallel** — reference, do not duplicate) | *none yet* — §8.22 says **NOT** DN-118, **NOT** cleanly M-876; DN-120 (identity) is adjacent only | **High** — 69/229 Impl sub-issues, the largest method-body class | general (Python mutation more pervasive) |
| **3** | **`write!`/`format!` → value-returning Display**, incl. an **int→string / generic-Display kernel prim** | macro-expansion + **kernel prim** | **Idiomatic Remapping** (Display-as-`Bytes` match) | **DESIGN-GATED** — M-875 (expand-first) `needs-design` + **no Display prim** (`prim.rs` empty, §8.22) | M-875; M-533 (done, residual #70); *Display-prim = new M-id* | **High** — real blocker under 26–30 `&mut Formatter` Impl gaps | general (Python `__str__`/`__repr__`/f-strings) |
| **4** | **Records / named fields** (`Foo{x,y}` construction, `.field` access, named-field patterns; drop-name faithfulness) | grammar (L3-G1) + faithfulness | **Idiomatic Remapping / Native Equivalent** (sugar over positional `Ctor`/`Data`) | **DESIGN-Draft** — **DN-123** (Option A, sugar-over-positional); needs ratify | DN-123; M-1078; M-876 | Med-High — Struct ~6% + NamedFieldDrop ~5%, pervasive | general (Python dataclasses / named args) |
| **5** | **Standard-derive lowering** (`Debug`/`Clone`/`PartialEq`/`Hash`/`Default`/`Ord` derives) | macro / metaprogramming | **Native Equivalent** (the `lower`/`derive` facility) | **PARTLY BUILD-READY** — facility landed (DN-54/M-812); **std-derive library unbuilt**; DN-110 Rank-1 generalization = M-1054 in-progress | DN-54/M-812; DN-110; M-1054 | **Med-High — NEW** (~11–12%, §6) | general (Python decorators/`@dataclass`) |
| **6** | **Cross-nodule runtime execution** (`mod`/`use`/qualified paths *run*, not just check) | **runtime** | **Native Equivalent** (phylum-wide evaluator reusing the check-time import registry) | **BUILD-READY design** (DN-99 A0) — build in progress | M-1024 (ENB-1, in-progress); M-982 | High (unblocks exec; **Import ~14% is mostly its measurement twin** via DN-124) | general |
| **7** | **Phylum-mode vet measurement** (credit correctly-emitted cross-nodule work) | tooling (force-multiplier) | *n/a — measurement enabler* | **BUILD-READY** — **DN-124 Accepted 2026-07-12**, Units 1–3 | DN-124; *Units 1–3 = new M-ids* | **Meta-high** — without it #1/#4/#6 stay un-creditable | general |
| **8** | **Signed ints** (`iN`/`isize`) — a shared std type + emit + idiom | kernel type-vocab (idiom) | **Idiomatic Remapping** (`SInt` ADT over `Binary{N}`; ops already landed) | **BUILD-READY** (ops landed DN-72/M-767; needs shared ADT + emit + note) | M-1029 (ENB-6); M-1034 (ENB-11); DN-121 P4 | Med (high frequency; ops done) | general |
| **9** | **`usize`/`isize` platform width + `char`** | kernel type-vocab (idiom) | **Idiomatic Remapping / Approximation** (`Binary{N}` + never-silent width FLAG; codepoint as `Binary{32}`) | **BUILD-READY** (idiom; DN-121 P5) | M-1029 (ENB-6); DN-121 P5 | Med | general |
| **10** | **Impl-level generic params** (`impl[T] Foo[T]`) | **grammar** (L3-G4) | **Native Equivalent** (mirror the fn/object type-param path) | **DESIGN-GATED** — needs own Draft DN (DN-99 A2) | DN-99 #63/A2; DN-119 Phase B; M-876 | Med (faithful impl-preserving self-host) | general |
| **11** | **Bounded-generic surface** (a *bounded* `Ty::Var`) | kernel type-system | **Native Equivalent** (contained kernel: bound on `Var`) | **DESIGN-GATED** (M-876) | M-876; DN-121 P3 | Med-High (pervades stdlib generic APIs) | general |
| **12** | **Transcendental + ε/δ float numerics** (`sqrt`/`exp`/`ln`/`sin`/`pow`) | kernel prims / runtime | **Approximation** (`flt.*` prims over existing `Float`, `Declared` ε, never-silent domain `Result`) | **BUILD-READY** — **DN-108 Accepted** | DN-108; M-1028 (ENB-5, todo) | Low-Med (numeric-heavy targets only) | general |
| **13** | **Closures / lambdas emit** (env-capturing) | transpiler + semantics | **Native Equivalent** (defunctionalization *already in the language*, RFC-0024 §4A/M-704) | **BUILD-READY** — **DN-118 Accepted** (P1 emit pass); FnMut/`&mut`-capture **flagged** | DN-118; RFC-0024 §4A/M-704 | Med (closures pervade iterator idioms) | general (Python lambdas/closures) |
| **14** | **`?` in general (non-tail) position** (CPS lift) | grammar (L3-G5) / elab | **Native Equivalent** (CPS lift over the existing bind) | **BUILD-READY-ish** — v0 postfix `?` **landed** (DN-102 Accepted); CPS lift is the deferred follow-up | DN-102 (FLAG-try-1); DN-99 #60 | Low (v0 covers the common case) | **Rust-specific** (Python uses exceptions — carry-forward differs) |
| **15** | **Struct-variant match patterns** (`Self::V{a,..}`) + variant-aware `StructLayout` | grammar / transpiler | **Native Equivalent** (positional, resolvability-gated) | **DESIGN-light / build** (§8.22 finding 5 — registry-shape + pre-pass) | DN-99 #29; M-1006; DN-119 L3-G1 | Low (~5% of Impl class) | general |
| **16** | **Conversion-method mapping** (`ToOwned`/`Clone`/`ToString`/`Into` → identity or real surface) | transpiler | **Idiomatic Remapping** | **BUILD-READY** (M-1037 todo) — unblocks the string-literal-pattern corpus win | M-1037; DN-99 #72 | Low-Med | **Rust-specific** |
| **17** | **Never-type `-> !`** (divergence) | runtime / type | **Approximation** (divergence-as-effect prim; `?` decoupled) | **BUILD-READY design** — DN-107 | DN-107; M-1030 (ENB-7, todo) | Low (DN-107: `?` doesn't need it) | **Rust-specific** (Python `NoReturn`) |

**Row-1 caveat (the sharp correction, `Empirical`, §8.22 2026-07-12).** DN-122 closes the external-trait
**definition** shape (`impl ForeignTrait for T`). But re-derived from the committed `*.gap.json`, the
corpus's ~113 `Impl` gaps are dominated by **method-BODY** failures — `&mut self` (row 2), `&mut Formatter`
+ `write!` (row 3), no native `Default`/`Error` prelude trait, qualified-call mangling (confirmed
zero-yield) — **not** the def-shape DN-122 solves. So DN-122's real leverage on *this* corpus is
`Declared` and awaits the phylum-mode re-measure; its headline "~15% / 119 gaps" (DN-121) is the *old*
corpus's Impl-class count, not the current method-body reality.

**Ranking rationale.** Rows 1–7 are the drive-hard core: the highest file-gating leverage (external-trait,
`&mut self`, Display/`write!`, records, derives, cross-nodule exec) plus the **measurement enabler** (row
7) that makes rows 1/4/6 *creditable*. Rows 8–13 are high-frequency but individually lower blast-radius
and mostly build-ready. Rows 14–17 are lower-leverage or common-case-already-covered.

---

## §4 Design-first vs build-now

### Build-now (a ratified/Accepted design exists — just build it)
- **#1 External-trait MVP** (DN-122 Accepted) — checker-first + mono fast-follow + transpiler rule-swap, zero L0/kernel/runtime.
- **#6 Cross-nodule runtime exec** (DN-99 A0; M-1024 in-progress) — phylum-wide evaluator reusing the check-time import registry.
- **#7 Phylum-mode vet** (DN-124 Accepted) — Units 1–3; the force-multiplier that credits #1/#4/#6.
- **#8/#9 Signed / usize / char idiom** (ops landed; M-1029/M-1034) — shared std ADT + transpiler emit + a port-guide note; no kernel change.
- **#12 Transcendentals** (DN-108 Accepted; M-1028) — `flt.*` prims over `Float`.
- **#13 Closure emit** (DN-118 Accepted) — emit the `lambda` surface, let mono defunctionalize; flag FnMut/`&mut`-capture.
- **#14 `?` v0** (landed) — CPS lift is the only residual.
- **#16 Conversion-method mapping** (M-1037).
- **#17 Never-type** (DN-107; M-1030).
- **#5 Standard-derive** — *partly*: the `lower`/`derive` facility is landed (M-812) and DN-110's generalization is M-1054 (in-progress); the **std-derive library** is the build residual (see §4-design for the scope decision).

### Design-first (needs a Draft DN + ratification before building)
- **#2 `&mut self` / `&mut T` value-semantics mapping — the top design-first item, UN-OWNED.** Mint a fresh, narrowly-scoped DN (or fold into an M-876 trait-Self-body sub-scope) that decides: explicit accumulator/builder mapping, in-place-return-`Self` rewrite, or permanently-gapped. **Being scoped in parallel** — this inventory defers the mechanism to that DN and only asserts the *direction* (Idiomatic Remapping via value-threading, ADR-003) and the *ownership FLAG*. DN-120 already established that content-addressed identity + efficient temporary-copy mutation is **solved-by-design** (DN-35 §5, `rc==1` reuse) — so the substrate exists; what is missing is the **receiver-shape surface/emit mapping**.
- **#3 `write!`/`format!` → Display, + a Display/int→string kernel prim.** Two coupled design decisions: (a) M-875 macro expand-first pass (`needs-design`); (b) a value-returning Display-as-`Bytes` rewrite **plus a kernel prim** for non-`Bytes` interpolation (`{n}`/`{limit}`) — `prim.rs` has none (`grep`-empty, §8.22). Without (b), fixing only the `&mut Formatter` signature yields **zero** movement (26/30 bodies interpolate non-`Bytes`).
- **#4 Records ratification** (DN-123 Draft → Accepted) — sugar-over-positional Option A; the design is written, it needs the ratify + build.
- **#5 Standard-derive library scope** — decide which derives lower via the facility vs stay drop-and-hand-write; the mechanism (M-1054/DN-110) is in-flight, the *coverage set* is the design call.
- **#10 Impl-level generic params** (grammar) — DN-99 A2; own Draft DN; HIGH semcore-lane collision.
- **#11 Bounded-generic surface** — M-876 design (a bounded `Ty::Var`).
- **#15 Struct-variant patterns** — a small, moderately-scoped registry-shape design (§8.22 finding 5).

---

## §5 What was STALE in the existing analyses (corrected here)

1. **"Other / type-coverage = 40%"** (delta-ledger §2; DN-121 §1) → **now ~23–24%** (`Empirical`, current corpus). The class shrank as `&T`-erasure, DeriveAttr, and records were split out into their own classes. The delta-ledger's 322/119/117/80/59 split is the **pre-enb-wave 812-gap corpus** — superseded.
2. **`checked_fraction` 7.8%** (delta-ledger headline) → the *phylum-mode* basis is **~7.4%** (DN-122 WU-A) and oracle-mode runs give 6.6–8.2% depending on target set/commit. All ~7–8%; none is a committable target until the DN-124 phylum-mode re-measure (VR-5).
3. **External-trait impls "M-876 needs-design / ~15% design-gated lever"** (DN-121 P1; DN-119 Phase G) → **SUPERSEDED**: **DN-122 Accepted 2026-07-12**, MVP build-ready. AND the sharper §8.22 correction — the current corpus's Impl gaps are **method-body** (`&mut self`/`write!`), not the def-shape DN-122 closes, so the old "15%/119" leverage is not the current reality.
4. **`?` operator "open — no `?` token"** (DN-99 #60) → **STALE**: `Tok::Question` exists, v0 postfix `Try` **landed** (DN-102 Accepted); only the general-position CPS lift remains (DN-119 §2 already flagged this).
5. **Transcendentals "open, deferred in M-718"** (DN-99 #42) → **SUPERSEDED**: DN-108 Accepted; build-ready as `flt.*` prims over existing `Float` (no new numeric type).
6. **Records "needs-design (M-876)"** (DN-121 P2) → DN-123 now supplies the design (Draft, sugar-over-positional).
7. **`&mut self` "owned by DN-118"** (a kickoff-brief premise) → **WRONG**: §8.22 re-read DN-118 in full — it scopes **closures that mutate a captured binding**, not a `&mut self`/`&mut T` method receiver/parameter. That surface is **un-owned**; a fresh DN is needed (row 2).
8. **The old lever-rankings are unreliable** (task premise, confirmed): this session proved external-trait design is now settled (DN-122) and the Impl blocker is method-body, so DN-121's P1→P2→P3 ordering and DN-119's Phase E/F/G leverage claims should be re-read against §3 here.
9. **Content-addressed-identity / temporary-copy mutation "residual to be solved"** (ADR-003 disclosed residual) → **SOLVED-BY-DESIGN** (DN-120: DN-35 §5). Not an open gap.

---

## §6 NEW gaps found (not prominent in the existing analyses)

1. **DeriveAttr is now a top-4 class (~11–12%)** — the standard-derive lowering **library** (`Debug`/`Clone`/`PartialEq`/`Hash`/`Default`/`Ord`) is unbuilt, even though the `lower`/`derive` *facility* landed (M-812) and its generalization is in-flight (M-1054/DN-110). The existing docs treat derives as a minor idiom row (DN-99 #3); at ~11–12% of current gap mass this is a **materially under-weighted lever** (row 5). Native answer exists in principle — this is a **build**, gated on a coverage-set decision.
2. **The `&mut Formatter` + `write!` coupling** — freshly measured (§8.22, 2026-07-12): the single largest clean Impl bucket (30 pure gaps) is **not** a signature gap but the co-located `write!` macro, and **26/30 need a kernel Display/int→string prim that does not exist** (`prim.rs` empty). This concrete kernel-prim gap is not called out as such in DN-121's type-vocabulary scoping (which focused on nominal *types*, not text-rendering *prims*).
3. **`ModuleDecl` as a distinct ~4% class** — the transpiler-visible twin of cross-nodule runtime exec (M-982), separate from `Import`.
4. **No native `Default` / `Error` prelude trait** (§8.22 finding 3) — 14 `ReservedWord` (`default`) + 12 empty `impl Error` gaps hit this same wall; a checker-side decision (want a native `Default`-shaped prelude trait? DN-122-MVP-style) is a small but real design question distinct from the external-trait mechanism.

---

## §7 The deliberate-exclusion set — these are NOT gaps (ratified native solutions, DN-119 §5)

Adding grammar for these would **regress** the language's guarantees; each already has a native answer
delivered by the transpiler emitting the remapping with a never-silent flag (never by new L3 grammar):

| Rust construct | Ratified native solution | DN-111 class | Basis |
|---|---|---|---|
| `&mut` / in-place mutation *(the value-threading target)* | value-threading (return new `T`) | Idiomatic Remapping | ADR-003; DN-109 D12 |
| unbounded `loop` / lazy infinite iterators | recursion / bounded `for`-fold | Idiomatic Remapping | RFC-0007 §4.8 |
| shared mutability `Arc`/`Mutex`/`RefCell` | runtime-tier value substrate (`hypha`/`mesh`, post-Phase-7) | Interop Bridge (runtime tier) | DN-94; RFC-0008 RT1 |
| silent numeric cast `x as T` | explicit `swap(x, to:T, policy:…)` | Idiomatic Remapping | DN-109 D13; S1 |
| implicit `From`-widening on `?` | explicit `map_err(e, conv)?` | Idiomatic Remapping | DN-102 §SP.2 |

> Note the distinction from row 2: the *value-semantics substrate* for mutation is a settled exclusion +
> native solution; what is **un-owned** is the **mechanical `&mut self`-receiver → value-thread mapping**
> the transpiler must emit. The problem is solved *in principle*; the emit mapping is the design gap.

---

## §8 Python carry-forward (flagged, not scoped — VR-5)

No Python-surface analysis exists in the corpus (DN-119 §11). Most rows above are **source-language-general**
(records, generics, closures, mutation, derives/decorators, numerics, cross-module resolution) and carry
forward; the **Rust-specific** rows are tagged in §3 (`?`-operator, `as`-cast, conversion-method mapping,
never-type). Python adds problems with **no Rust analogue** — dynamic/duck typing, exceptions (vs `Result`),
generators/coroutines, decorators, metaclasses, pervasive shared-mutable state — several of which collide
head-on with the §7 exclusion set. Per DN-119 §11, Python needs its **own DN + its own DN-111 classification
pass**, sequenced after the Rust reframe is ratified — folding it in here would be an ungrounded `Declared`
guess (declined).

---

## §9 FLAGs (owned elsewhere — this doc edits none of them)

`Doc-Index.md`, `CHANGELOG.md`, and `issues.yaml` are integration-owned (concurrent-PR pattern: leaves
FLAG, the integrating parent applies once). **FLAG to the integrator:**

- **Doc-Index row:** add a Planning-docs row — `language-completeness-gap-inventory.md — Language-Completeness Gap Inventory (Draft, 2026-07-12)`.
- **CHANGELOG:** append-only `[Unreleased]` entry for this inventory (dated 2026-07-12).
- **M-ids to mint** (highest existing id is **M-1078** on this tree; **M-1079+ free** — mitigation #1, verify at filing):
  - **`&mut self` / `&mut T` value-semantics mapping DN** — mint a Draft-DN slot (**next free is DN-125** on this tree) + a tracking M-id; **this is the top design-first item** (§3 row 2, §4). Note it is **being scoped in parallel** — coordinate, do not double-scope.
  - **Display / int→string kernel prim** — a tracking M-id under the M-875 macro-expand design (§3 row 3, §6.2).
  - **Native `Default` / `Error` prelude-trait decision** — a small design M-id (§6.4).
  - **M-1080** (the DN-122 external-trait MVP; branch `M1080-DN122-external-trait-mvp` merged) — **confirm/file the issue row** (not present in `issues.yaml` on this tree).
  - **DN-124 vet Units 1–3** — file as tracking M-ids (§3 row 7).
- **Cross-refs:** align `doc_refs` on M-876 (split into external-trait ✓DN-122 / records DN-123 / bounded-generic sub-units, per DN-121 FLAG-issues), M-1029/M-1034 (add `corpus:DN-121`), and add `corpus:` pointers to this inventory from the delta-ledger.

---

## §10 Changelog

- **2026-07-12** — initial Draft. Synthesized the current, prioritized **language**-completeness gap
  inventory for full native expressibility of Rust (Python carry-forward flagged), grounded on worktree
  `30ddfa41` (contains `origin/dev` `fa53dc46`) and the committed draft corpus (`eb6bc0e2`) + the
  phylum-mode baseline (`06b4d7a7`). Re-derived the gap-class distribution against the current tree
  (mitigation #14) — corrected nine stale figures/rankings in the prior analyses (§5, incl. Other 40%→24%,
  external-trait now DN-122-Accepted, `?`/transcendentals landed, `&mut self` un-owned not DN-118), surfaced
  four new/under-weighted gaps (§6, incl. DeriveAttr ~11–12% and the missing Display kernel prim), classified
  every gap by the DN-111 native-translation taxonomy (§1/§3), and split the residual into build-now
  (ratified design) vs design-first (needs Draft DN) sets (§4). Recommends, does not ratify (house rule #3);
  every figure `Empirical`/`Declared` at its basis (VR-5). FLAGs the Doc-Index/CHANGELOG/issues rows up (§9).
