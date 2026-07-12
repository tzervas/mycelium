# Design Note DN-119 — L3 Comprehensive Surface Expressibility: Scoping, Reframe, and Phased Plan

| Field | Value |
|---|---|
| **Note** | DN-119 |
| **Status** | **Draft** (2026-07-11). Authored as **READ + a new DN only** (design-scoping reasoner). It **enacts nothing** and **moves no other doc's status** (house rule #3, append-only). It scopes the maintainer directive *"implement the L3 concrete surface grammar comprehensively to express ALL possible Rust (and eventually Python) constructs, closing all remaining gaps for full native capability."* Every claim is `Empirical` where read against the tree (integration tip `b36f0ef4`, 2026-07-11) or `Declared` for a proposed design/plan not yet ratified (VR-5). It recommends **ranked, does not ratify** — the maintainer/orchestrator ratifies (house rule #3/#4). |
| **Decides** | *Proposes, for ratification:* (1) a **reframe of the directive's target** from "L3 grammar accepts every Rust construct verbatim" to "**L3 expresses the native answer to every construct's underlying problem**" (DN-111 taxonomy) — the honest form of "full native capability"; (2) the **verified inventory** of what is genuinely-open L3 *grammar* surface vs what is misattributed to L3 but is actually kernel/runtime/transpiler/deliberate-exclusion work; (3) a **phased decomposition** by construct class, each a scoped buildable unit, ranked by leverage and tagged by lane (semcore-serial vs disjoint); (4) the **honest `checked_fraction` boundary** — the sharp finding that L3-grammar work moves the vet number ~0, the lever is downstream. It does **not** edit `crates/mycelium-l1/**`, `lib/compiler/**`, `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (semcore-lane / integration-owned; FLAGGED up per §9). |
| **Feeds** | RFC-0030 (L3 concrete-surface grammar); DN-99 (surface-gap closure register); DN-109/DN-110/DN-111 (native-translation taxonomy + gap-closure default); the Zero-Hand-Port Delta Ledger (`docs/planning/zero-hand-port-delta-ledger.md`); the `enb` backlog. |
| **Date** | July 11, 2026 |
| **Task** | Comprehensive L3 surface expressibility design-scoping — verified remaining-gap inventory + reframe + phased plan + honest tag boundary. Sibling this session: a design-reasoner authoring the closure-enabler note (~DN-118); this note is the L3-surface scoping half. |

> **Grounding + honesty (transparency rule / VR-5 / G2 / house rule #4).** This note **works up a
> scope and a plan**; it takes no decision (house rule #3). The state of the L3 surface is read
> directly from `crates/mycelium-l1/src/parse.rs` + `ast.rs` + `token.rs` at integration tip
> `b36f0ef4` — the **code is ground truth; the DN-99 register and the delta ledger lag it**
> (mitigation #14). **No sycophancy (house rule #4):** the central finding of this note **cuts against
> the literal directive** — "express ALL Rust in L3 grammar" is *not* the right target for a
> meaningful subset of Rust, because the corpus the maintainer already ratified (ADR-003, DN-94,
> DN-102, DN-106, DN-109) designates those constructs **deliberate exclusions** mapped to native
> solutions, and adding L3 grammar for them would *regress* the language's value-semantics guarantees.
> Following the evidence, not the phrasing, is the maintainer's standing preference (VR-5 applied to
> agreement).

---

## §1 Purpose, grounding, and the layering it must respect

The directive names **L3** — the concrete surface grammar/projections tier in the layering
**L0 core IR ← L1 kernel calculus ← L2 surface term language ← L3 concrete surface grammar**
(RFC-0006 §3; RFC-0030 §1). The transpiler emits L3 text; the language must accept it. The
directive's goal is the **surface half of zero-hand-port**: full native expressibility of the Rust
(later Python) construct space.

This note's first job is to **hold the layer boundary honestly**. Much of what is loosely called an
"L3 surface gap" in the register (DN-99) and the delta ledger is **not** a concrete-grammar
production at all — it is kernel type-vocabulary, trait semantics, runtime execution, a transpiler
rule, or a *deliberate exclusion*. Treating all of it as "implement L3 comprehensively" would
mis-scope the work, mis-assign lanes (semcore-serial vs disjoint), and — for the exclusion set —
build grammar that **defeats the language's own guarantees**. §3–§5 separate these precisely.

**House-rule anchors:** the transparency lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` (rule #1);
never-silent swaps/gaps (rule #2, G2, S1); append-only status (rule #3); grounded claims (rule #4);
small auditable kernel — no grammar for its own sake (rule #5, KISS/YAGNI, KC-3).

---

## §2 Verified state of the L3 concrete surface (`Empirical`, integration tip `b36f0ef4`)

Read directly against `crates/mycelium-l1/src/parse.rs` (2518 lines), `ast.rs`, `token.rs`. The L3
grammar is **substantially complete for the landed surface** (RFC-0030 M-706 close, 2026-06-29). What
is present:

**Items** (`parse_item` / `parse_pub_item`, `parse.rs:465/620`): `phylum`, `nodule`, `fn` (with
bounded generics `[T: Bound]`, effect annotations `!{…}`, tiered/`fuse`/`reclaim` forms), `type` decl
(positional constructors, with **per-constructor visibility seal** `priv` — M-1027/DN-104 landed),
`trait` decl, `impl` item (both trait-instance `impl Trait[args]? for T {…}` and inherent `impl T {…}`,
M-664), `object` decl, `lower` decl, `derive` decl, `use` path, `pub`, `paradigm`.

**Types** (`parse_base_type`, `parse.rs:1579`): `Binary`/`bin`, `Ternary`/`tern`, `Dense`/`emb`,
`VSA`/`hvec`, `Substrate`, `Seq{T,N}`, `Bytes`, `Float` (binary64, ADR-040), `Named(args)`,
paradigm-less ambient repr `{…}`, tuple types `(T,U,…)` (arity ≥ 2, M-826), type-args in `[…]`
(RFC-0037).

**Expressions** (`parse_expr_inner` + dispatch, `parse.rs:1828`): precedence-climbing operator
expressions (full RFC-0025 suite — arithmetic, comparison `cmp_expr`, shift `shift_expr`, bitwise, all
word/glyph desugared), unary, `for` (bounded fold sugar), `let…in`, `if/then/else`, `match`, `swap`
(never-silent, names `to:`+`policy:`), `with paradigm`, `wild {…}`, `wrapping`, `spore`, `consume`,
`colony`, `hypha`, application, `lambda(x)=>e`, and the **`?` postfix try-operator** (`Try`, wired at
`parse.rs:2329` — DN-102 v0 increment **landed and Accepted 2026-07-10**).

**Patterns** (`Pattern` enum, `ast.rs:975`; `parse_pattern_guarded`, `parse.rs:2125`): `Wildcard` (`_`),
`Lit` (bin/trit/bytes/str/float/int/list), `Ctor(name, subs)` (**positional only**), `Ident`,
`Tuple` (arity ≥ 2, M-826), **`Or`** (`p₁ | p₂ | …`, RFC-0020 §9, checker-desugared — **closed**,
`parse_arm` at `parse.rs:2101`).

> **Register-lag corrections (mitigation #14 — the code disproves several DN-99 "open" flags):**
> DN-99 row **#60** marks the `?` operator `open` ("no `?` token") — **stale**: `Tok::Question` exists
> (`token.rs:389`) and `parse_app` consumes it (`parse.rs:2329`); the v0 postfix `?` is **landed**
> (DN-102 Accepted). DN-99 row **#73** marks or-patterns closed — **confirmed** (`Pattern::Or`). Row
> **#37** (per-ctor seal) is landed (M-1027/DN-104), as DN-99 §3 already notes. So the genuine open L3
> *grammar* residual is **much smaller** than a naive read of the 92-row register suggests.

---

## §3 The genuinely-open L3 *grammar* residual (the real L3 work)

These are the only constructs that are (a) Rust surface with real port frequency, (b) not a
deliberate exclusion (§5), and (c) genuinely absent as a **concrete-grammar production** — i.e. actual
L3 work in `crates/mycelium-l1` (the semcore serial lane). Verified absent against the tree.

| ID | Construct class | Evidence (absent) | DN-111 class of the native answer | Lane | Size |
|---|---|---|---|---|---|
| **L3-G1** | **struct/named-field patterns** (`C { a, b }` in `match`) | `Pattern` enum has no `Struct`/record variant (`ast.rs:975`); `parse_pattern_guarded` has no `{`-arm | Idiomatic Remapping (positional `Ctor` pattern) — genuine surface if named-field faithfulness wanted | semcore-serial | S |
| **L3-G2** | **range patterns** (`0..=9`, `'a'..='z'` in `match`) | no `DotDot`/range token in `token.rs` (verified 0 hits) | Idiomatic Remapping (guard `if x >= lo and x <= hi`) — grammar only if demand witnessed | semcore-serial / **idiom** | S |
| **L3-G3** | **`@`-binding patterns** (`n @ 1..=5`) | `Pattern` enum has no `Bind` variant | Idiomatic Remapping (bind + guard) | semcore-serial / **idiom** | S |
| **L3-G4** | **impl-level generic params** (`impl[T] Foo[T]`) | `parse_impl_item` has no `type_params` slot (`parse.rs:1187`; DN-99 #63) | Native Equivalent (mirror the fn/object type-param path) | semcore-serial (**HIGH collision**) | L |
| **L3-G5** | **`?` in general (non-tail) position** | v0 wires postfix `Try`; general-position CPS lift is the DN-102 deferred follow-up (FLAG-try-1) | Native Equivalent (CPS lift over existing bind) | semcore-serial | M |
| **L3-G6** | **range *expressions*** (`a..b`, `a..=b` as values) | no range token; DN-99 #68 | Idiomatic Remapping (`iota` helper / comparisons) — **likely YAGNI as grammar** | **idiom** (no grammar) | S |
| **L3-G7** | **format-string mini-language residual** (`{:.2e}` etc.) | `fmt.myc` partial; DN-99 #70 | Idiomatic Remapping (Display composition) + one runtime prim | idiom + runtime-`enb` | M |

**That is the whole genuine L3-grammar residual: ~7 construct classes, of which only L3-G4
(impl-generics) and L3-G5 (`?` CPS lift) are unambiguously "add a grammar production," and L3-G1 is a
faithfulness nicety.** The rest are Idiomatic Remappings that a DN should likely *decline to add as
grammar* (KISS/YAGNI) unless port frequency proves the idiom too costly. This is the sharp, honest
scope: **the L3 concrete surface is close to complete; "comprehensive L3" is mostly already done.**

---

## §4 Misattributed to "L3 surface" — actually downstream (NOT L3 grammar)

The bulk of the delta-ledger gap mass (~812 instances, §2 of the ledger) and most DN-99 `open`/`partial`
rows are **not L3 concrete-grammar productions.** Adding L3 grammar does not touch them. They belong to
other lanes and other DNs (routed below so the directive is not silently under- or over-scoped):

| Misattributed class | What it actually is | Real lane / owner | Evidence |
|---|---|---|---|
| signed ints, `usize`/`isize`, `char` (DN-99 #26/#44/#45) | **kernel type-vocabulary** (`Binary{N}` mapping / new base) | semcore-serial (kernel), ADR-028/M-874 | delta-ledger §2 "type-coverage 322 (40%)"; `checkty.rs`/`repr.rs` |
| external-trait impls, trait Self-bodies (DN-99 Impl class, 119) | **trait *semantics*** in the checker, not surface | semcore-serial (checker), M-876 | `delta-L3-transpiler.md` §2: "emits but FAILS check" |
| named-field record **type** surface | **deliberate exclusion** — positional constructors only; DN-106 §2/§3 **rejects** record-update literal | n/a (excluded by design) | DN-106; `emit.rs` NamedFieldDrop |
| cross-nodule symbol resolution + **execution** (DN-99 #41) | **runtime** (`eval.rs` single per-nodule env) | semcore-serial (runtime), M-982/ENB-1 | `eval.rs:557`; DN-99 A0 |
| transcendental + ε/δ floats (DN-99 #42) | **kernel prims + ADR** | semcore-serial (kernel), M-1028/ENB-5 | `prims.rs`; ADR-040 |
| macros (invocation + def, 64) | **transpiler expand-first pass** | disjoint (`mycelium-transpile`), M-875 | `transpile.rs:300` |
| `use`/imports (117) | **transpiler symbol table** | disjoint (`mycelium-transpile`), M-1001 | `delta-L3-transpiler.md` §5 |

**The delta ledger's own measurement (§2, `Empirical`) settles this:** *"~75-80% are DOWNSTREAM
(language/kernel surface), ~20-25% transpiler-only … faithful transpiler rules moved
`checked_fraction` by 0; every real gain came from kernel/language surface."* **None** of the
dominant gap mass is L3 concrete-grammar work. So a wave titled "implement L3 comprehensively" would,
if taken literally, spend its effort on the ~7 classes of §3 (near-zero vet impact) while the real
levers sit in the kernel/runtime and transpiler lanes it does not name.

---

## §5 The deliberate-exclusion set — L3 must NOT grow grammar for these (the adversarial core)

This is the load-bearing, non-sycophantic finding. A specific set of Rust constructs is **excluded by
ratified design** and mapped to a **native solution**; adding L3 grammar to accept them *verbatim*
would be a **net regression** of the language's guarantees, not a "closure." "Express ALL Rust" is the
wrong literal target precisely here.

| Rust construct | Ratified native solution | Why L3 grammar for it is *wrong* | Basis (`Empirical`) |
|---|---|---|---|
| `&mut` / in-place mutation | **value-threading** (return new `T`) | Mycelium is value-semantic (NFR-5); a mutable-reference production reintroduces aliasing `syn` cannot prove sound (delta-ledger §5 ceiling) | ADR-003; DN-109 D12; DN-99 #19/#46/#82 |
| unbounded `loop` / lazy infinite iterators | **recursion / bounded `for`-fold** | totality/termination is a language property; an unbounded-loop production defeats the recursion-budget guard (RFC-0041) | RFC-0007 §4.8; DN-99 #40/#83 |
| shared mutability `Arc`/`Mutex`/`RefCell` | **runtime-tier value substrate** (`hypha`/`mesh`), post-Phase-7; no user-facing shared-mut | RT1 excludes shared mutable state by design; surface for it is out of the value-semantics model | DN-94; RFC-0008 RT1; DN-99 #56 |
| silent numeric cast `x as T` | **explicit `swap(x, to:T, policy:…)`** | S1 forbids any inferred/silent representation change; an `as`-production is a black-box swap (rule #2/G2) | DN-109 D13; DN-99 #30 |
| implicit `From`-widening on `?` | **explicit `map_err(e, conv)?`** | an inferred error conversion is a black box the `?` site does not name (rule #2) | DN-102 §SP.2; DN-99 ENB-2 addendum |

**For every row here, "expressing the construct" already has a native answer** — delivered by the
**transpiler emitting the native remapping with a never-silent flag** (DN-109 D6/D13, `suggested_idiom`
M-1045), *not* by L3 grammar. Adding grammar would let a mutation/aliasing/silent-conversion construct
through the front door and force the checker to either reject it anyway (no gain) or accept it and lose
S1/NFR-5/totality (regression). **This is the definition of a construct that should stay a
transpiler-flagged Idiomatic-Remapping, never an L3 production.**

---

## §6 The reframe — the honest form of "full native capability" (recommendation, ranked)

**Objective function (the criteria a "comprehensive L3" plan is scored against):**

| Criterion | Weight | What it rewards |
|---|---|---|
| C1 — Native expressibility of every construct's *problem* | high | the underlying problem has a native answer (grammar OR flagged remapping) |
| C2 — Guarantee preservation (S1/NFR-5/totality/VR-5) | **veto** | no production regresses value-semantics / never-silent / totality |
| C3 — `checked_fraction` leverage | medium | moves the honest vet number |
| C4 — KISS/YAGNI (grammar only when idiom proves too costly) | high | no production added for its own sake |
| C5 — Faithfulness (round-trip readability of ported code) | medium | struct/impl-preserving self-host |

**Ranked recommendation:**

- **R1 (recommended) — Reframe the target to "L3 expresses the native *answer* to every construct's
  problem" (DN-111), which is `{direct L3 production}` for the §3 non-excluded set ∪ `{transpiler
  flagged native-remapping}` for the §4 downstream + §5 excluded sets.** Under this framing "full
  native capability" is **achievable and mostly achieved**; it scores highest on C1/C2/C4 and honestly
  on C3. The genuine L3-grammar work is the small §3 set. *This is the recommendation.*

- **R2 (rejected) — literal "L3 grammar accepts every Rust construct verbatim."** Fails C2 (**veto** —
  §5 productions regress guarantees) and C4 (grammar for constructs the idiom already handles), and
  scores ~0 on C3 (L3 grammar moves `checked_fraction` ~0 per the ledger). Rejected.

- **R3 (partial) — "L3 grammar for the non-excluded gaps only" (§3), defer §4 to their real lanes.**
  This is R1 restricted to the grammar deliverable; sound but under-delivers on "full native
  capability" unless paired with the downstream/transpiler routing (§7). R1 is R3 + the routing, so
  R1 dominates.

**Argument against my own recommendation (VR-5).** R1's weakness: it *relabels* a large part of the
directive as "not L3," which could read as scope-dodging. Two honest rebuttals, and one concession:
(a) the relabel is grounded in the ledger's own `Empirical` measurement (§4) and the ratified
exclusions (§5), not in convenience; (b) R1 still delivers "full native capability" — just across the
correct lanes. **Concession:** if the maintainer's intent is specifically that the **transpiler
should stop flagging and start auto-emitting** more of the §4 downstream set, then the real ask is
**kernel type-vocabulary + trait-semantics work** (M-874/M-876) — which R1 names but does not itself
scope; that is the sibling closure-enabler note's (~DN-118) and the delta-ledger §6 roadmap's job.
R1's boundary with that work should be ratified explicitly (§8 DoD).

---

## §7 Phased decomposition (by construct class, ranked by leverage; lane-tagged)

Each phase is a scoped, buildable unit. **Lane:** `L3-gram` (semcore-serial, `mycelium-l1` grammar) ·
`kern` (semcore-serial, kernel/checker) · `rt` (semcore-serial, runtime) · `tr` (disjoint,
`mycelium-transpile`) · `idiom` (docs, no code). Semcore-serial phases (`mycelium-l1`/`lib/compiler`)
**coordinate with the cloud semcore lane** — they are the collision surface and must not run in
parallel with each other; `tr`/`idiom` phases are disjoint and parallelizable.

**The genuine L3-grammar phases (the directive's literal deliverable):**

- **Phase A — pattern-surface completion (L3-G1/G2/G3, `L3-gram`, S–M).** Add struct/named-field
  patterns; *decide* (per a small DN) whether range and `@`-binding patterns get native productions or
  stay guard-idioms (recommend: struct patterns land; range/`@` stay idiom unless port frequency
  proves otherwise — KISS). **`checked_fraction` impact: low** (faithfulness, not vet). First because
  it is self-contained and reuses the resolvability gate.
- **Phase B — impl-level generic params (L3-G4, `L3-gram`, HIGH collision, L).** `type_params` slot on
  `parse_impl_item` mirroring fn/object; lifetime-erase decision. Needed for a *faithful
  impl-preserving* self-host (C5). Serial on the semcore lane. **Needs its own Draft DN** (DN-99 A2).
  `checked_fraction` impact: low-to-moderate (unblocks faithful impl emission where checker accepts).
- **Phase C — `?` general-position CPS lift (L3-G5, `L3-gram`/elab, M).** DN-102 deferred follow-up
  (FLAG-try-1). Reuses existing bind; no new kernel semantics. Serial. Low vet impact (v0 postfix
  already covers the common case).
- **Phase D — format-string residual (L3-G7, `idiom` + one `rt` prim, M).** Display-composition recipe
  in the port guide + the `{:.2e}` float-precision prim (DN-99 #70, adjacent to ENB-5). Mostly idiom.

**The downstream/transpiler phases that "full native capability" actually depends on (NOT L3 grammar —
routed here so the plan is honest about where the leverage is; owned by the delta-ledger §6 roadmap and
the sibling ~DN-118):**

- **Phase E (highest vet leverage) — kernel type-vocabulary** (signed ints, `usize`/`char`; §4 row 1;
  `kern`, M-874/ENB-6). The dominant 40% gap class; the real `checked_fraction` lever.
- **Phase F — cross-nodule symbol resolution + execution** (`rt`, M-982/ENB-1) — unblocks
  `mod`/`use`/qualified paths at runtime.
- **Phase G — external-trait impls / trait Self-bodies** (`kern`, M-876) — the 15% Impl class.
- **Phase H — transpiler symbol table + macro expand-first pass** (`tr`, disjoint, M-1001/M-875) — the
  ~20-25% transpiler-only mass; parallelizable with the semcore phases.
- **Phase I — the idiom catalogue** (`idiom`) — document the §5 deliberate-exclusion mappings + the §4
  remappings once in the RFC-0031 port guide (DN-99 §4 Track C), so the transpiler's `suggested_idiom`
  flags cite a sanctioned recipe.

**Ordering rationale:** A–C are the directive's literal L3 deliverable and can proceed on the semcore
lane in the A→B→C order (self-contained → HIGH-collision → elab). But **if the goal is measured native
capability (`checked_fraction`), Phase E must lead** — the ledger proves L3 grammar alone moves the
number ~0. The plan therefore recommends **interleaving**: run E/F/G on the kernel/runtime semcore lane
(the real lever) and A/B/C on the grammar semcore lane, with H/I disjoint in parallel — and **not**
front-loading A–C as if they were the capability unlock they are not.

---

## §8 Honest tag boundary + `checked_fraction` impact per phase

- **L3-grammar phases (A/B/C/D): `Declared` designs; `checked_fraction` impact LOW.** This is the
  single most important honesty point of the note (VR-5): the delta ledger §2/§4 *measured* that
  faithful transpiler and grammar-surface additions moved `checked_fraction` by **0** on the 17-target
  corpus; only kernel/language **type** surface (String→Bytes, `and`/`or` prims) moved it. So the
  directive's literal L3-grammar work **raises faithfulness/completeness, not the vet number.** Any
  claim that "comprehensive L3 → higher native capability (measured)" must be **downgraded** to
  "comprehensive L3 → higher *faithfulness*; measured capability is a kernel/runtime lever."
- **Downstream phases (E/F/G): the real `checked_fraction` movers.** Phase E (type-vocabulary) targets
  the 40% dominant class. Expected impact material but **`Declared` until re-measured** — Phase 0 of
  the ledger (fresh `checked_fraction` post-enb-wave) is a prerequisite; the last honest number is
  **~7.8% / 17 targets** (`Empirical`, file-gated lower bound), **not** to be assumed improved.
- **Deliberate-exclusion set (§5): tag stays at the native-remapping's basis, never upgraded.** A
  flagged `&mut`→value-thread or `as`→`swap` emission is `Declared` until a differential witnesses it
  (M-991 / DN-34 §8.7); it is **never** counted as an L3 "closure."
- **No tag in this note is upgraded past its basis.** The register-lag corrections (§2) are
  `Empirical` (read against `parse.rs`/`ast.rs`/`token.rs` at `b36f0ef4`). The gap-class frequencies
  (322/119/117/…) are quoted `Empirical`-per-source from the delta ledger / DN-34, **not re-counted
  here** (declared residual uncertainty).

---

## §9 Definition of Done

**For this DN (done at authoring):**
1. Verified L3-surface state recorded against the tree, with register-lag corrections (§2) — **done**.
2. Genuine-open L3-grammar residual separated from downstream/excluded misattributions (§3/§4/§5) —
   **done**.
3. The reframe recommended, ranked, with an objective table and a self-argument (§6) — **done**.
4. Phased decomposition by construct class, lane-tagged and leverage-ranked (§7) — **done**.
5. Honest `checked_fraction` boundary per phase (§8) — **done**.

**For maintainer ratification (what "Accepted" requires — the reasoner does not self-ratify, house
rule #3/#4):**
6. **Confirm or amend the reframe (R1 §6):** is "full native capability" = "L3 expresses every
   construct's native *answer*" (grammar ∪ flagged remapping), accepting that the §5 set stays
   transpiler-flagged and is **not** given L3 grammar? A yes ratifies the veto criterion C2.
7. **Confirm the L3-grammar deliverable scope (§3/§7 A–D):** which of L3-G1..G7 get native productions
   vs stay idiom (recommend: struct patterns + impl-generics + `?` CPS lift land; range/`@`/range-expr
   stay idiom). Each of Phase B/C (and any range/`@` decision) needs its **own Draft DN** before
   implementation.
8. **Confirm the boundary with the downstream lever (§6 concession):** ratify that Phases E/F/G
   (kernel type-vocabulary / runtime / trait-semantics — the measured `checked_fraction` lever) are
   owned by the delta-ledger §6 roadmap + the sibling closure-enabler note (~DN-118), **not** by this
   L3-surface note, so the two do not double-scope.
9. **Authorize the phase filing** (the integrator owns `issues.yaml`): file Phase A–D as L3-grammar
   tracking issues + the two-to-three sub-DNs; cross-ref Phases E–I to their existing tracked ids
   (M-874/M-876/M-982/M-875/M-1001).
Status stays **Draft** until 6–9 are ratified.

---

## §10 Doc-Index + changelog + issues (FLAGGED up, not applied here)

`docs/Doc-Index.md`, `CHANGELOG.md`, and `tools/github/issues.yaml` are **integration-owned** (the
concurrent-PR pattern: leaves FLAG, the integrating parent applies once). This DN does **not** edit
them. **FLAG to the integrator:**
- add a Design-Notes row for `DN-119 — L3 Comprehensive Surface Expressibility Scoping (Draft)` to
  `Doc-Index.md`;
- add a `CHANGELOG.md` entry (append-only, dated 2026-07-11);
- on ratification of §9, file Phase A–D as L3-grammar tracking issues + the Phase-B/C (and any
  range/`@`) sub-DNs, and cross-ref Phases E–I to M-874/M-876/M-982/M-875/M-1001.
- **Python surface is explicitly out of scope of this note** and needs its **own DN** (§11) — flag it
  as a distinct future workstream, not folded into the Rust L3 plan.

---

## §11 Python — flagged, not scoped (deliberate deferral, VR-5)

The directive names Python as an *eventual* target. **No Python surface analysis exists in the corpus
today** (verified: 0 Python-surface references in DN-99 / RFC-0030). Python's construct space differs
from Rust's in ways that are **not** reducible to the Rust L3 mapping: dynamic/duck typing, exceptions
(vs `Result`), generators/coroutines, decorators, metaclasses, monkey-patching, and pervasive shared
mutable state. Several of these collide **head-on** with the §5 deliberate-exclusion set (shared
mutability, dynamic dispatch without bounds) and would need their own problem→native-solution mapping
under DN-111 before any L3 surface question is even well-posed. **Recommendation:** treat Python as a
**separate DN + a separate DN-111 classification pass**, sequenced *after* the Rust L3 reframe (R1) is
ratified and the exclusion boundary is settled. Folding Python into this note would be a `Declared`
guess with no grounding — declined (house rule #4 / G2).

---

## §12 Changelog

- **2026-07-11** — DN-119 created (**Draft**). Scoped the "comprehensive L3 surface" directive:
  verified the L3 grammar is substantially complete (register-lag corrections for `?`/or-pattern/seal,
  §2); isolated the genuine ~7-class L3-grammar residual (§3) from the downstream/kernel/runtime/
  transpiler misattributions (§4) and the ratified deliberate-exclusion set (§5); recommended the R1
  reframe of "full native capability" to "L3 expresses every construct's native answer" with a veto on
  guarantee-regressing productions (§6); gave a lane-tagged, leverage-ranked phased plan (§7); and
  drew the honest `checked_fraction` boundary — L3 grammar moves the vet number ~0, the lever is
  downstream (§8). `Empirical` where read against the tree (integration `b36f0ef4`); `Declared` for
  the proposed reframe/plan. Authored READ + DN only — no edit to `crates/mycelium-l1/**`,
  `lib/compiler/**`, `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (semcore-lane / integration
  owned; FLAGGED up per §10). Python flagged as a separate future DN (§11). Append-only; status
  advances only by maintainer ratification (house rule #3).
