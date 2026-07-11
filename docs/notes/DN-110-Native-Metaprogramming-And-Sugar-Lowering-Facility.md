# Design Note DN-110 — Mycelium's Native Facility for the Role Rust Fills with Macros (Generative-Lowering + Sugar Rules), and the Rust→Mycelium Native-Translation Taxonomy it Instantiates

| Field | Value |
|---|---|
| **Note** | DN-110 (next free note number — DN-109 was the prior highest in `docs/Doc-Index.md`, 2026-07-11; a Draft number is cheap to renumber at merge, so it is picked-and-noted, not blocked on). |
| **Status** | **Accepted** (2026-07-10, maintainer ratification — see the dated "Ratification (maintainer, 2026-07-10)" section below). **Accepted is a DESIGN ratification, NOT Enacted** (house rule #3: Enacted requires stepping through Accepted first, and means *fully implemented/landed*; this note ships no code). The facility's guarantees stay **`Declared`** until implementation + the E1/E3 experiments (DN-110-8.2 deep-dive §7) + `reveal`/M-1051 land — no guarantee tag is upgraded past its basis (VR-5). Originally **Draft** (2026-07-11). Authored as **READ + a new DN only**; at Draft time it **enacted nothing**, shipped no code, and **moved no other doc's status** (house rule #3, append-only). It answers the maintainer's design question — *what is Mycelium's own native construct(s) for the ROLE Rust reaches for macros to fill* — as **one instance of a general Rust→Mycelium native-translation principle** (§2, **provisional terminology — see the Ratification's taxonomy carve-out and the forthcoming companion DN-111**), enumerates the real alternatives, **recommends (ranked)**, adversarially stress-tests, and gives a **skill-derivable operational decision-procedure** (§9). |
| **Decides** | *Proposes, for ratification:* (0) a **general translation taxonomy** — **Adaptation / Solution / Approximation / Bridge** — for mapping any Rust construct's underlying *problem* to Mycelium's own native answer, reconciled with DN-109's L4 idiom buckets and DN-99's register status column (§2); (1) that the macro-role (compile-time codegen, boilerplate, syntactic/DSL abstraction, derive-gen, generative expansion) is met by a **blend of existing + one extended native mechanism**, NOT by importing a surface-syntactic hygienic-pattern-macro system; (2) that the **generative-lowering framework already landed** (`lower`/`derive`, DN-38/DN-54/M-812) is the spine, **generalized** to expression-position **sugar rules** (the `format!`/`matches!` role) under one term-level lowering-rule model; (3) that **`reveal` / expand-on-demand (M-1051)** is the mandatory transparency spine (house rule #2, DN-106); (4) that **compile-time computation** stays on the **static-specialization** path (DN-55); (5) that **foreign-concrete-syntax macros** are declined as primary and handled as a **Bridge** (library-with-parser) or flagged exclusion. It does **not** edit `issues.yaml`, `CHANGELOG.md`, `Doc-Index.md`, the grammar, `crates/mycelium-l1/**`, or `lib/compiler/**` (integration- / semcore-lane owned; §12 lists the FLAGs). |
| **Feeds / builds on** | **DN-109** (L4 idiom framework: Mechanical/Heuristic/Judgment, the EXPLAIN-able ratchet, the F1 "map PROBLEM→native SOLUTION" reframe — the *decidability axis* this note's taxonomy is the *relationship axis* complement of); **DN-99** (surface-gap closure register — the enumerated per-construct native closures this taxonomy generalizes, and the `docs/planning/zero-hand-port-delta-ledger.md` companion); **DN-106** (surface-sugar transparency + the gap-closure default; M-1051 desugar/expand-on-demand); **DN-38** (layered-lowering atlas + generative-lowering mechanism + `reveal` — the seamless-gradient thesis and the lowering LAW); **DN-54** (user-extensible generative-lowering surface + checker — `lower`/`derive`, M-812 landed); **DN-53/DN-37** (`object`/`via`); **DN-55** (static specialization = compile-time-computation native path); **DN-100 / M-1032 / M-875** (the *Rust-macro* transpiler side — the complementary direction this facility is the native **target** of); **RFC-0006** (L0→L1→L2 + S1–S5); **ADR-003** (content-addressed identity); **KC-3** (small kernel). The **kernel-unfrozen north star** (ADR-045; `.claude/memory` — different native path per problem, zero hand-ports via mechanical porting) is the frame. |
| **Guarantee** | Every design claim here is **`Declared`** (a proposal). Where it cites a *landed* mechanism (`lower`/`derive` elaboration, M-812/M-812-cont) or a *register-verified* closure (DN-99 rows, `file:line`-cited), that underlying fact is **`Empirical`** at its own source; the *generalizations* proposed here are `Declared`. No tag is upgraded past its basis (VR-5). |
| **Date** | July 11, 2026 |

> **Posture (transparency rule / VR-5 / G2 / house rule #4).** A recommendation for a maintainer to
> ratify, not a ratification. Two honest findings up front: (1) **Mycelium already has ~80% of the
> macro-role facility** (`lower`/`derive`, DN-54/M-812, landed + three-way-witnessed) — so the right
> answer is largely "**generalize what exists**," not "design a macro system"; (2) the taxonomy in §2 is
> **not invented here** — it is the *problem-relationship* re-reading of classification vocabulary the
> corpus already uses (DN-99's register Status column, DN-109's L4 buckets, DN-106's gap-closure default),
> cited row-by-row. The single genuinely-new design surface is the **expression-position sugar-rule
> hygiene/scoping model** (§8.2), flagged as the sharpest open question. No sycophancy: the recommendation
> deliberately **declines** the reflexive "port Rust `macro_rules!`" answer, and §8 states plainly where it
> cannot express what Rust macros can.

---

## §1 Frame — the ROLE, decomposed (not "design Mycelium macros")

The maintainer named this "macros" as a **vocabulary convenience**, not a prescribed mechanism. The task
is: *what is Mycelium's own native path to each problem Rust solves with a macro* — chosen on **fit with
Mycelium's value-semantics, never-silent, small-kernel paradigm**. This is one instance of the standing
project principle (made explicit in §2): translating a Rust construct means **mapping its underlying
PROBLEM to Mycelium's native answer**, never shoehorning Rust's idiom into Mycelium (DN-109 F1 reframe;
DN-106 §3a; the kernel-unfrozen north star).

Rust's macro system is really **five distinct jobs** wearing one name. Decomposed against Mycelium, each
job is tagged with the §2 translation class it falls under:

| # | Rust's macro job | Rust mechanism | Mycelium's native path (this DN) | §2 class | State today |
|---|---|---|---|---|---|
| J1 | **derive-style generation** (`#[derive(Clone,…)]` → `impl` blocks) | derive proc-macro | **`lower Name = <rhs>` + `derive Name for T`** (DN-54) | **Adaptation** | **Landed** (M-812/M-812-cont, `Empirical`) |
| J2 | **delegation / forwarding boilerplate** (newtype-forward an impl) | derive/attr macro | **`object { … via N : Trait … }`** (DN-53/DN-37) | **Adaptation** | **Landed** (M-811, `Empirical`) |
| J3 | **expression-position sugar / mini-DSL** (`format!`, `matches!`, `vec!`, `assert!`) | function-like `macro_rules!` | **expression-position sugar rules** (this DN's one new surface) | **Solution/Adaptation** | **Gap** — proposed here (`Declared`) |
| J4 | **compile-time computation** (`const fn`, const generics, build-time tables) | `const`/proc-macro | **static specialization** (DN-55, inference-driven, Zig-`comptime` analogue) | **Solution** | **Designed** (DN-55, `Declared`/partial) |
| J5 | **foreign concrete syntax in-source** (`html!{<div/>}`, `sql!`, `quote!`) | proc-macro token rewriter | **library-with-parser** (data/string + `certified`-checked parse) or flagged exclusion | **Bridge** | **Out of scope** (flagged §8.1) |

**The load-bearing observation:** J1/J2 are *already native + landed* (Adaptation); J4 has its own native
different-path answer (Solution, DN-55); J5 is the piece Rust macros do that Mycelium deliberately does
**not** chase natively (Bridge). The **single real gap** is **J3 — expression-position mechanically-lowering
sugar** — and the maintainer already named its resolution shape: *"create the convenience sugar and ensure
it lowers mechanically and reliably"* (DN-106 gap-closure default, Accepted). This DN decides **the facility
that defines those sugars**, because — per the maintainer — *those sugars are largely this facility*, the
primary "mechanically-lowering sugar" gap-closure vehicle in the zero-hand-port north star (what makes most
L1 surface gaps mechanical, hence zero-hand-port-friendly).

### §1.1 Grounding the baseline (mitigation #14 — verified against the code/corpus, 2026-07-11)

- **No `macro`/`derive-macro`/`comptime` node in the grammar or `mycelium-l1`.** The `Expr` variant set is
  `Let · If · Match · For · Swap · WithParadigm · Wild · Spore · Wrapping · Consume · Try · Colony · Lambda
  · App · Fuse · Reclaim · Path · Lit · Ascribe · TupleLit` (DN-106 §2, read against `ast.rs`) — no
  macro/quote/splice node. The only generative constructs are item-level `Item::Lower`/`Item::Derive`
  (DN-54/M-812) and the `object` surface (DN-53/M-811). Native metaprogramming is **greenfield except for
  `lower`/`derive`/`object`**, which already exist and are the correct spine.
- **A reserved-word scan of `.claude/memory/lang-lexicon-syntax.md` finds NO free macro/quote/template
  word to reuse.** The generative vocabulary is already **Active**: `lower` (rule definition), `derive`
  (rule use-site), `via` (delegation), `object` (composition); `reveal` (inspector) is *Ratified,
  not-yet-lexed*; `grow` is superseded-not-active (teaching diagnostic → `derive`). There is **no**
  reserved `macro`/`quote`/`splice`/`comptime`/`template` keyword. **This DN therefore mints no new
  top-level keyword** — §5-A reuses the `lower`/`derive` vocabulary rather than inventing one the lexicon
  deliberately never reserved (plain-first, reuse-before-coin — DN-02 naming law).
- **`reveal` / expand-on-demand is a *ratified capability requirement*, not a built tool.** DN-38 §5
  ratified `reveal`; DN-106's ratification filed **M-1051** ("desugar/expand-on-demand tooling for surface
  sugars, general", `status:todo`, `docs/tero-index/INDEX.md:4831`). The transparency spine this facility
  rests on is designed + required but not yet shipped — an honest dependency, `Declared`, named as a
  ratification gate (§3.4, §8.4).

---

## §2 The general principle — the Rust→Mycelium native-translation taxonomy (Adaptation / Solution / Approximation / Bridge)

> **Ratification carve-out (maintainer, 2026-07-10 — see the "Ratification" section near the end of this
> note for the full record).** The four labels below (**Adaptation / Solution / Approximation / Bridge**)
> are ratified here only as **provisional, intuitive handles** the maintainer is comfortable reasoning
> with today — **not** as canonical, ratified terminology for the general Rust→Mycelium translation
> taxonomy. The canonical taxonomy, with refined/final terms, is **deferred to a forthcoming companion
> DN-111 (Draft, not yet authored)**. Everything in this DN that *uses* the taxonomy (§5–§9, the
> `/native-translate` decision procedure) is ratified on the strength of the *classification behavior* it
> describes, not on these specific words being locked in — do not treat "Adaptation"/"Solution"/
> "Approximation"/"Bridge" as frozen vocabulary pending DN-111.
>
> **Append-only pointer (integrating parent, 2026-07-10) — this carve-out is now RESOLVED.** **DN-111**
> (`docs/notes/DN-111-Canonical-Rust-To-Mycelium-Native-Translation-Taxonomy.md`) has been authored and
> **Accepted** by the maintainer (see its own "Ratification (maintainer, 2026-07-10)" section): the
> canonical terms are **Native Equivalent** (was Adaptation) / **Idiomatic Remapping** (was Solution) /
> **Approximation** (kept) / **Interop Bridge** (kept, qualified). This DN-110 §2 table and its "Adaptation
> / Solution / Approximation / Bridge" vocabulary are **unchanged, append-only** (house rule #3 — this note
> is Accepted and not rewritten); the four handles above now read as **retained aliases** for DN-111's
> canonical terms, not as provisional placeholders awaiting a companion DN. New cross-links should cite
> `corpus:DN-111` for the canonical name and treat this §2 table's terms as the legible shorthand.

The macro-role question is one instance of a **larger principle the maintainer states directly**:
translating a Rust construct maps its underlying **PROBLEM** to Mycelium's own native answer; it does **not**
shoehorn Rust's idiomatic paradigm into Mycelium. Four **native-translation strategies** name *what kind of
relationship* the Mycelium target bears to the Rust source. **This taxonomy is not invented here** — it is
the problem-relationship re-reading of vocabulary the corpus already uses (each row cited):

| Strategy | Definition | Corpus vocabulary it re-reads | Grounded exemplars (DN-99 register, `file:line`-cited) |
|---|---|---|---|
| **Adaptation** | A true first-class Mycelium-native **equivalent** — the problem has a direct native answer, auto-emittable. | DN-99 `closed`/`already-closed` (`cl`); DN-109 **Mechanical** (auto-fire) | impl-block #2 (`parse.myc:3670`, M-664 — "native + auto-emitted"); struct-def #4 (`emit.rs:1652`, M-1006); trait-decl #12 (`parse.rs:723`, M-1013); generic-bound #5 (RFC-0019 §4.1, M-673); bitwise-suite #86 (M-745). **Macro-role: J1 `#[derive]` → `lower`/`derive`** (M-812). |
| **Solution** | Mycelium's own **DIFFERENT** native path to the same problem — the language's intentionally-different conventions (value-semantics/functional-update, structured/bounded control, errors-as-values, explicit never-silent `swap`). | DN-99 `idiom` where a different native form is canonical; DN-109 F1 reframe ("map PROBLEM→native SOLUTION"); DN-106 §3a | const/static #14 (`totality.myc:273` — `const`→nullary `fn name()=>T`); if-let #31 (`ebnf:292` — desugar to `match`+`if/then/else`); mutation→functional-update (DN-106 Part 2, destructure-and-reconstruct); shared-mut #56 EXCLUDED by RT1 (DN-94); unbounded loop→bounded `for` (RFC-0007 §4.8). **Macro-role: J4 comptime → static specialization** (DN-55). |
| **Approximation** | A close-but-**not-exact** native form, with the **delta made EXPLICIT and honest** (VR-5: tag the gap, never paper over it). | DN-99 `idiom` carrying a recorded caveat; DN-109 **Heuristic** (rule + EXPLAIN flag) | derive-attr #3 (`emit.rs:1538` — "drop Debug/Clone; hand-write structural eq; **sub-gap stays never-silent**"); if-let #31 ("idiom recorded **w/ fall-through caveat**"); `&T`-erasure (DN-109 D4, exact-under-value-semantics but flagged). |
| **Bridge** | An interop/compat **crossing** where no full native form exists yet — a **temporary, clearly-marked** boundary, not a permanent shoehorn. | DN-99 `tr-only`/`open` kept never-silently flagged; DN-109 **Judgment** (flag, never guess) + `suggested_idiom` | macro-invocation #11 (`transpile.rs:300` — "tr-only, hand-expand", the DN-100/M-875 pre-pass); import-use #1/#13 (`Category::Import` **stays flagged**); float-transcendentals #42 (`open`, `rt`, ADR-gated); never-type #88 (`open`, model as divergent host-effect); the `wild`/FFI boundary (ADR-014). |

### §2.1 The two axes are orthogonal and complementary (reconcile with DN-109)

DN-109's L4 buckets classify **who can soundly decide** a mapping and **where it closes** (Mechanical =
auto-fire / Heuristic = flag+EXPLAIN / Judgment = flag-never-guess). This taxonomy classifies **what native
relationship** the target bears to the source problem. They are **orthogonal axes** that correlate but do not
coincide:

- **Adaptation** is usually **Mechanical** (direct equivalent, auto-emit), but a *hygiene-sensitive* Adaptation
  can require a flag.
- **Solution** spans **Mechanical** (safe problem→native mappings — DN-109 D4 `&T`-erasure, the mutation→functional
  auto-emit of DN-106 fork A) *and* **Judgment** (where `syn` cannot prove the mapping safe — DN-109 D7 `&mut`
  aliasing, D8 boundedness).
- **Approximation** is characteristically **Heuristic** (the delta is the flag).
- **Bridge** is characteristically **Judgment** / never-silent-refusal-with-`suggested_idiom` (DN-106 §3a: bare
  refusal is the last resort).

The **binding ratchet is DN-109 §3.2, unchanged**: a mapping auto-fires only if it is semantics-preserving,
inserts no `swap` (S1), upgrades no guarantee tag (VR-5), and is EXPLAIN-recorded. This taxonomy adds a
*problem-side vocabulary* on top of that decidability ratchet; it does **not** relax it. Every Approximation's
delta and every Bridge's crossing is a **never-silent, EXPLAIN-able** record (house rule #2 / G2), and no
strategy may present a lossy target as exact (VR-5).

### §2.2 The macro-role facility as an instance of the taxonomy

The recommendation in §6 is exactly this taxonomy applied to the five macro jobs (the J-column of §1's
table): J1/J2 are **Adaptation** (native `lower`/`derive`/`object`, landed); J3 is **Solution/Adaptation** (a
native sugar rule — a mechanically-lowering surface, the DN-106 gap-closure default); J4 is **Solution**
(static specialization, a *different* native path, DN-55); J5 is **Bridge** (library-with-parser, or a flagged
exclusion). The deliberate-exclusion cases (a Rust macro that fakes in-place mutation, or an unbounded
generative loop) are **Solution** — their *problem* maps to functional-update / bounded control, never sugared
over (DN-106 principle 2 exclusion set).

---

## §3 Definition of Done (this note, for maintainer ratification — house rule #6)

"Accepted" requires the maintainer to:

1. **Accept or amend the §2 translation taxonomy** (Adaptation/Solution/Approximation/Bridge) and its
   reconciliation with DN-109's L4 buckets + DN-99's register — and rule on **whether it warrants its own
   companion DN or an append-only extension of DN-109** (§12's flagged recommendation; not decided here).
2. **Accept or amend the §1 role-decomposition** — confirm J1/J2 (`lower`/`derive`/`object`, landed) + J4
   (static specialization, DN-55) as the native answers, so this DN's *new* scope is only **J3** plus the
   transparency spine.
3. **Rule on the recommended mechanism (§6, Rank 1):** generalize the `lower`/`derive` term-level
   lowering-rule model to expression position, rather than a surface-syntactic pattern-macro system (Alt B)
   or a comptime term-construction API (Alt C).
4. **Confirm the transparency gate:** the facility's implementation is **co-gated on `reveal` / M-1051**
   (expand-on-demand), and in `certified` mode on the DN-38 §5 round-trip obligation (`delaborate ∘ lower =
   id`). A sugar with no reveal is a black box (house rule #2) and is disqualified.
5. **Resolve §11's open questions** — above all **§8.2 (expression-position hygiene/scoping)**, which
   DN-54's type-position model (Model A, DN-81/M-973) does **not** settle, and **§8.1** (J5 permanent
   exclusion vs later Bridge library-with-parser).
6. **Authorize a follow-on epic** whose per-change DoD = *"every sugar rule is (a) a mechanical, reliable
   lowering to existing core grammar with zero L0-node growth (KC-3, the DN-54 §6 guard), (b) `reveal`-able
   / reversibly expandable on demand, and (c) hygiene-witnessed by the DN-38 §7 differential+hygiene
   harness; guarantee tags propagate the weakest tag of the RHS, never upgraded (VR-5)."*
7. **Authorize the `/native-translate` methodology skill** (§9, §12) as a follow-up deliverable.

Until then this note is **Draft**; all its design guarantees remain **`Declared`**.

## §4 User stories

- *As a self-hosting engineer porting the Rust stdlib*, I want the `impl_narrow_int!` / `impl_std_error!`
  boilerplate (76 of 82 item-position macro invocations DN-100 §2 measured) to map to a **`lower` rule +
  `derive` sites** (Adaptation), and `format!`/`matches!` to map to **expression sugar rules** (Solution),
  so a macro-heavy Rust module becomes *mechanically* portable instead of hand-expanded.
- *As a Mycelium application developer*, I want to eliminate my own repeated boilerplate with a **named sugar
  I define once that lowers to ordinary code**, and to **`reveal`** the lower form at any point, so the
  abstraction never becomes a black box I can't debug (house rule #2).
- *As a reviewer / auditor*, I want every generated or sugared form to carry an **EXPLAIN-able trail** to the
  exact core grammar it compiles to, so I can review a generative construct without reverse-engineering an
  opaque expander (the Lombok anti-pattern, DN-38 §4).
- *As an engineer translating any Rust construct (not just macros)*, I want a **decision procedure** that
  tells me whether the construct is an Adaptation, Solution, Approximation, or Bridge — and what the honest
  native target + explicit delta is — so I classify by evidence against the corpus, not by shoehorning Rust's
  idiom (a `/native-translate` skill, §9).
- *As the maintainer guarding the kernel (KC-3)*, I want the facility to **add no new L0 node and no new
  observable semantics** — a sugar must be *observationally the identity* of its expansion — so
  metaprogramming power does not cost kernel auditability.

---

## §5 The real alternatives (each a genuinely different native mechanism)

Five candidates for the J3 facility, evaluated on: expressiveness for the role · transparency/reversibility
(house rule #2 / DN-106) · hygiene under value semantics · kernel-complexity cost (KC-3) · value-semantics
fit · and **Rust-macro porting-gap closure** (DN-100 profile).

### §5-A — Generalize the landed `lower`/`derive` term-level lowering framework (RECOMMENDED)

**What it expresses.** DN-54's `lower Name[params] = <rhs>` already defines a generative rule whose RHS is a
*real Mycelium term* elaborated to closed L0; `derive Name for T` applies it at a **type/item** position (J1).
This alternative **generalizes the same model to expression position** (J3): a rule applied at a use site,
where the invocation's arguments are substituted (capture-avoidingly) into the RHS term and the whole lowers
to L0 exactly as a hand-written expression would. One model — a **term-level lowering rule** — covers
derive-style generation *and* function-like expression sugar; `object`/`via` (J2) stay the composition
specialization of the same spine.

**Transparency / reversibility.** *Best-in-class, by construction.* DN-38 §4.1–§4.2: the **only** output
channel of a Mycelium lowering is "produce an L0 term" — a real value in the frozen core, never a side-effect
on a hidden AST — so the inspectable, reusable artifact **exists by construction** and the facility **cannot
become Lombok**. `reveal` (DN-38 §5 / M-1051) shows the actual L0 term with real binding structure (not lossy
text). Reversibility is `delaborate ∘ lower = id`, enforceable in `certified` mode (DN-38 §5). This is exactly
DN-106's expand-on-demand requirement and DN-109 §3.2's EXPLAIN-able ratchet, already the design of record.

**Hygiene.** Because rules produce **L0 terms with real binding structure** (not surface text), hygiene is
**lexical-capture avoidance at elaboration**: rule-introduced binders are alpha-fresh, free identifiers in the
RHS resolve in the **rule's definition scope**, and argument substitution is **capture-avoiding**. Value
semantics removes the *other* hygiene hazard entirely — no shared mutable state means no "action at a distance"
capture. The residual hazard is name capture, handled by the L0-term / alpha-conversion discipline (DN-38 §7
lists hygiene as a first-class verification obligation). **NB (honest):** DN-54 settled the *type-position*
attachment model (Model A, DN-81/M-973); the **expression-position** hygiene/scoping model is **new** and is
flagged §8.2 — a *tractable* hygiene story, not yet a *finished* one.

**Kernel-complexity cost.** *Lowest of the generative options.* **Zero new L0 node** (DN-54 §6 KC-3 guard: the
elaborated RHS lowers to existing L0 nodes only — `Proven`-by-construction over the closed `Node` enum). It
reuses `elab::elaborate_lower_rule`, the §4.1 IL-grammar RHS type-check, the §4.6 purity (`wild`-refusal)
check, and the §4.2 acyclicity check. The genuinely-new cost is the **expression-position argument matcher +
capture-avoiding substitution + hygiene machinery** in the *elaborator* (not the kernel) — bounded but
non-zero (§8.3).

**Value-semantics fit.** *Native.* The RHS is an ordinary value-semantic term; never-silent fallibility
(`Option`/`Result`), structured/bounded control, and structured diagnostics (RFC-0013) apply to it unchanged.
Content-addressed L0 (ADR-003) gives same-intent-same-identity dedup for free.

**Porting-gap closure.** *Highest on the measured corpus.* DN-100 §2 measured ≈93% custom `impl_*!`
boilerplate (J1 — the `lower`/`derive` shape) plus an expression-position `format!`/`matches!` tail (J3). It
does **not** close J5 (§8.1).

### §5-B — Hygienic surface-syntactic pattern macros (a `syntax-rules` / `macro_rules!` analogue)

A declarative matcher-transcriber over **surface syntax** (fragment specifiers + repetition + hygiene). It
operates on tokens/AST *before* elaboration, so its output is surface that must be re-parsed and re-elaborated
— transparency is **surface→surface** and the reveal is a second lowering (weaker than §5-A's direct-to-L0
artifact; `cargo expand`'s README warns text expansion "is a lossy process… a debugging aid only" — the
lossiness *is* hygiene, DN-38 §5). Hygiene must be *engineered into the transcriber* (Scheme's `syntax-rules`
lineage). Kernel/tooling cost is **high** (fragment grammar, repetition engine, hygiene algorithm, a new
surface-macro node) against KC-3/YAGNI. It uniquely enables J5-lite, but the measured corpus does not need it.
**Rejected as primary** — a strictly weaker transparency posture than §5-A for the jobs that dominate.

### §5-C — Procedural / comptime metaprogramming (Zig-`comptime` / Rust-proc-macro analogue)

Compile-time *execution* of Mycelium code constructing L0 terms as first-class values, with reflection. Most
general — but two Mycelium facts blunt it *as a new facility*: (i) the **computation** half is *already*
Mycelium's **static specialization** (DN-55 — Zig-`comptime`-equivalent but **inference-driven**, no explicit
comptime call site), so a separate comptime construct is largely redundant with a landed-design native path;
(ii) a compile-time interpreter + term-construction API + reflection is the **largest kernel cost** of any
option, and value-semantics + never-silent + G2 tightly constrain it. **Not recommended as a new facility**;
its legitimate niche (compile-time tables/specialization) is DN-55's, and its "generator in the same language"
spirit is honored by §5-A's RHS-is-a-real-term model.

### §5-D — A dedicated first-class `sugar` declaration construct (the DN-106/M-1051 direction as a keyword)

A new construct — e.g. `sugar <name>(<params>) = <lowering>` — declaring a surface sugar + its lowering, with
expand-on-demand built in (the literal reading of DN-106's "create the convenience sugar"). Functionally this
**is** §5-A with a different keyword. The lexicon scan (§1.1) found no reserved `sugar`/`macro` word, and
DN-54's `lower` already *is* "define a mechanically-lowering rule." Minting a *parallel* `sugar` keyword beside
`lower` violates DRY and the plain-first/reuse-before-coin naming law (DN-02) and risks two overlapping
generative surfaces. **Folded into §5-A** (the expression-position extension *is* the "sugar rule"); recorded
as a non-adopted *separate* construct, not left implicit (house rule #4). *(A distinct teachable spelling for
the expression-position case is a §8.6 naming sub-question, not a different mechanism.)*

### §5-E — Status quo: blessed compiler desugarings + traits + first-class fns only

Rely on the *existing* set — compiler-built-in sugars (`?`→match DN-102, `for`→structural recursion,
`object`→impls), traits + bounded generics, `lambda`, and `lower`/`derive` for J1 — and add any *new* sugar
only as a **blessed compiler-internal desugaring** (like `?`/`for`), never user-defined. Much of Rust's
function-like-macro need dissolves under value semantics + traits (`format!` → a builder-API + trait). This is
the KISS/YAGNI floor and is *correct for J4/J5 and many J3 cases* — but it makes **every new sugar a
kernel-team decision**, contradicting the maintainer's framing that *those sugars are largely this facility*
(a **user-/port-extensible** vehicle). It is the honest **fallback** if §8.2's hygiene model proves too
costly, and the right answer for blessed-core sugars regardless — but as the *whole* answer it leaves the
zero-hand-port lever under-powered.

---

## §6 Recommendation (ranked; NOT ratified — house rule #4)

**Rank 1 — §5-A: generalize the landed `lower`/`derive` term-level lowering framework to expression position,
with `reveal`/M-1051 as the mandatory transparency spine and `object`/`via` retained for composition.** One
native mechanism — the **term-level lowering rule** — spans derive-style generation (J1, Adaptation, landed),
delegation (J2, Adaptation, landed), and expression-position sugar (J3, Solution/Adaptation, the one new
surface). Compile-time computation (J4) stays on DN-55 static specialization (Solution); foreign-syntax (J5) is
a Bridge / flagged exclusion (§8.1). This wins on **every** binding constraint: transparency/reversibility *by
construction* (L0-term output channel + `reveal`), tractable value-semantics hygiene (real binding structure,
not text renaming), lowest kernel cost (**zero L0-node growth**, DN-54 §6 guard; only elaborator grows), native
value-semantics fit, and the **measured** porting-gap closure (DN-100). It is also the most **DRY/KISS** answer
— it *reuses a landed, ratified, three-way-witnessed mechanism* (M-812/M-812-cont) rather than building a macro
system.

**Rank 2 — §5-E (status quo + blessed desugarings)** as the **honest fallback and co-strategy**: blessed-core
sugars (`?`, `for`, `object`, future core conveniences) remain compiler-internal desugarings regardless; Rank 2
is what we fall back to for J3 if §8.2's hygiene model proves too costly to build safely. Ranks 1 and 2 are
**complementary** — Rank 1 is the *user-/port-extensible* layer atop Rank 2's *blessed-core* layer.

**Not recommended:** §5-B (pattern macros — weaker transparency, engineered hygiene, high kernel cost, YAGNI
on the measured corpus) and §5-C (a *new* comptime facility — redundant with DN-55, largest kernel cost). §5-D
is **folded into Rank 1**. J5 is out of scope (§8.1).

**Why this is the Mycelium-native answer, not the reflexive Rust one.** Rust needs `macro_rules!`/proc-macros
partly *because* its macro output is surface tokens over a language without a frozen inspectable core, so
hygiene and expand-tooling are hard-won add-ons. Mycelium's **frozen, content-addressed L0 + the
seamless-gradient lowering law** (DN-38) invert that: generation *is* "produce an L0 term," so transparency,
dedup, and hygiene are substrate properties, and the native facility is a **term-level lowering rule**, not a
token-level rewriter. This is §2 in action: the macro role's *problem* (eliminate boilerplate / abstract
syntax while staying inspectable) maps to a native **Adaptation** (`lower`/`derive`) + **Solution** (sugar
rules), not a shoehorned port of Rust's mechanism.

### §6.1 How the recommended construct composes with already-documented closures (no duplication)

The recommendation is a *vehicle for* existing DN-99 register closures, not a re-decision of them:

- **Composes with, does not replace, the landed `lower`/`derive`** (DN-54/M-812; DN-99 rows #2/#3). J1 derive
  is already the Adaptation; §5-A only *extends its application position* to expressions.
- **Composes with `object`/`via`** (DN-53/M-811; DN-99 #2) for J2 delegation — unchanged.
- **Consumes the DN-106 gap-closure default** for J3: each new sugar rule *is* "a convenience sugar that lowers
  mechanically and reliably" (DN-106 principle 2) — this DN supplies the *facility*, DN-106 supplies the
  *policy*, M-1051 supplies the *reveal tool*.
- **Defers J4 to DN-55** (static specialization) and **J5 to a Bridge** (library-with-parser) — it does not
  duplicate or re-open either.
- **Records into DN-109's EXPLAIN-able manifest** (§3.2/§5.2): a sugar rule's idiom choice is a Mechanical (or
  Heuristic) entry in the same `idiom_choices` ledger, so the transpiler-side and language-side transparency
  trails are one artifact, not two.

---

## §7 Criteria table (the objective function)

Scores are **`Declared`** design judgments (H = strong fit, M = partial, L = weak), against the binding
house-rule constraints. Rank 1 = §5-A.

| Criterion (basis) | §5-A generalize `lower`/`derive` | §5-B pattern macros | §5-C comptime | §5-D `sugar` keyword | §5-E status quo |
|---|:--:|:--:|:--:|:--:|:--:|
| **Transparency / reversible reveal** (rule #2, DN-106, DN-38 §5) | **H** (L0-term artifact, `reveal`) | M (surface→surface, lossy text) | M (opaque generator risk) | **H** (= §5-A) | H (blessed, inspectable) |
| **Hygiene under value semantics** (DN-38 §7) | **H** tractable (real binding struct) | M engineered transcriber hygiene | M (reflection capture) | **H** (= §5-A) | H (compiler-controlled) |
| **Small-kernel / KC-3 cost** (rule #5) | **H** zero L0-node growth | L (macro node + fragment/hygiene engines) | L (interp + reflection API) | M (parallel keyword, DRY-cost) | **H** (nothing new) |
| **Value-semantics fit** (RT1/G-items) | **H** RHS is a real value-semantic term | M (token-level, semantics-agnostic) | M (staged-eval constraints) | H (= §5-A) | H |
| **Porting-gap closure J1–J3** (DN-100 profile) | **H** ≈93% J1 + J3 tail | H (also some J5) | M (J4 only, already DN-55) | H (= §5-A) | L (no extensible J3) |
| **Reuse of landed work / DRY** (rule #5, M-812) | **H** extends landed `lower`/`derive` | L (new subsystem) | L (new subsystem) | L (parallel to `lower`) | **H** (nothing new) |
| **Expresses foreign-syntax DSLs J5** (honest) | **L** (Bridge, §8.1) | **H** | **H** | L | L |

The single column §5-A trails on is **J5** (foreign concrete syntax) — the deliberate Bridge (§8.1), the exact
place §5-B/§5-C would "win" at a transparency/kernel cost the project has chosen not to pay.

---

## §8 Adversarial stress-test of the recommendation (VR-5 / house rule #4 — where it breaks)

### §8.1 The expressiveness ceiling — J5 foreign concrete syntax (top concern)

A **term-level** lowering rule can only match/rewrite what already **parses as a Mycelium expression or type**.
It therefore **cannot** express a macro embedding a *foreign concrete syntax* — `html!{<div>…}`,
`sql!{SELECT…}`, a bespoke token grammar — which `macro_rules!`/proc-macros (§5-B/§5-C) *can*. **This is a real
capability Rust has and Rank 1 does not.** Honest disposition: (a) the **measured** corpus barely uses J5
(DN-100 §2 is ≈93% `impl_*!` boilerplate); (b) where J5 is genuinely wanted, the native path is a **Bridge** —
a **library taking a string/data value + a parser** (a runtime/`certified`-checked value, never a compile-time
token rewriter) — or a flagged exclusion with a `suggested_idiom` (DN-109 D6; DN-106 §3a) — **never a silent
gap.** But the ceiling must be stated, not hidden: **Rank 1 closes the boilerplate/expression-sugar role, not
the embed-a-foreign-language role.** Whether J5 is a *permanent* exclusion or a *later* Bridge is a genuine
open fork (§11.2).

### §8.2 Hygiene/scoping for expression-position rules is a NEW model, not inherited (sharpest finding)

DN-54 settled the **type/item-position** attachment/consumption model (Model A, DN-81/M-973). **Expression
position is different** and its questions are **open**: (i) do free identifiers in a rule's RHS resolve at the
**definition site** (true hygiene) or the **use site** (Rust's `$crate`/hygiene-escape need)? (ii) how are
use-site argument terms substituted **capture-avoidingly** when the RHS introduces its own binders? (iii) does
an expression sugar introducing an **affine `Substrate` binder** (DN-71) or a `consume` interact soundly with
the use-site's affine accounting? Reusing the type-position model does **not** answer these. **This is the one
genuinely-new design surface of the whole facility**, flagged as the primary §3.5 ratification question —
mis-designing it turns a "hygienic" sugar into a silent capture bug (the exact class house rule #2 exists to
prevent). Recommended stance (`Declared`, to be ratified): **definition-site resolution + capture-avoiding
substitution as the default** (the `syntax-rules`/Lean-macro consensus), with any use-site-capture escape an
*explicit, flagged* opt-in.

> **Cross-ref (added at ratification, 2026-07-10 — append-only).** This stance is worked forward,
> ground-truthed against the codebase, and given a concrete mechanism + five-experiment validation plan in
> the companion note **`docs/notes/DN-110-8.2-hygiene-deepdive.md`**: def-site resolution plus **`%`-namespace
> freshening** of RHS binders (reusing the landed `Elab::fresh` gensym) plus partition-safe substitution of
> use-site argument terms plus affine/type checking on the *expanded* L0. Ratified as the basis for this
> section — see the "Ratification" section below.

### §8.3 "Zero kernel growth" ≠ "zero complexity" (KC-3 honesty)

The KC-3 headline — **zero new L0 node** — is real and `Proven`-by-construction for the *kernel* (DN-54 §6,
closed `Node` enum). But it must **not** be oversold: the expression-position extension grows the **elaborator**
(an argument matcher, a capture-avoiding substitution engine, hygiene bookkeeping). That is bounded (reuses
DN-54's elaboration path, adds no L0 node) and auditable, but **not free**. Honest tag: kernel L0 surface
`Proven`-unchanged; elaborator complexity **grows `Declared`-bounded**. If §8.2's hygiene model needs a large
renaming/substitution subsystem, the KISS/YAGNI calculus tips toward Rank 2 (§5-E) for J3 — a fallback the
recommendation deliberately keeps open.

### §8.4 The transparency spine is a promise, not yet a tool

The facility's headline property — *hides nothing, reveal on demand* — currently rests on **`reveal`
(ratified, not-yet-lexed)** + **M-1051 (`todo`)**. Until those ship, a sugar's "transparency" is a **design
promise (`Declared`)**, not a built guarantee. If `reveal`/expand-on-demand never ships or is lossy, the
facility **violates house rule #2** (a generative construct with un-inspectable output is disqualified — the
task's own gate). **Mitigation:** §3.4 co-gates implementation on M-1051, and `certified` mode enforces the
DN-38 §5 round-trip check so the inspector is a *check*, not merely a viewer. This dependency is named, not
buried.

> **§8.4 addendum (2026-07-10, following the E1+E3 hygiene-experiment go/no-go) — append-only, does not
> edit the §8.4 text above.** **M-1051 Increment-3** (`crates/mycelium-l1/src/reveal.rs`'s
> `certified_roundtrip`) and the **E3 experiment** (`crates/mycelium-l1/src/tests/
> reveal_roundtrip_e3.rs`, the companion note's own "E3 Result" addendum) are now **built and run** —
> narrowing, but not closing, the gap this subsection names. The reveal spine moves
> **`Declared → Empirical`, but ONLY for the L0-term-level round-trip claim on the reparseable
> fragment** identified by the companion note's STEP-0 finding (`Const`/`Var`/`Let`(-of-those); every
> `Op`/`Lam`/`Fix`/`FixGroup`/`Construct`/`Swap` stays out-of-contract/unbuilt for the *surface* path,
> empirically, independent of `%`-freshening). This is **not** a blanket upgrade of "transparency is a
> built guarantee" — the source-span `site` resolver, a `reveal` CLI, and `certified`-mode wiring into
> the checker (M-1051 Increment-2 and beyond) remain unbuilt, and the facility itself (M-1054) has not
> landed. See `docs/notes/DN-110-8.2-hygiene-deepdive.md`'s "E3 Result" addendum for the full,
> narrowly-scoped record (VR-5 — no upgrade past what was actually checked).

### §8.5 The guarantee-tag-upgrade trap (VR-5)

A lowering rule must **never** let its expansion claim a stronger guarantee tag than its RHS supports — e.g. a
`derive` emitting a `Proven`-tagged impl from an `Empirical`/`Declared` basis. The facility must **propagate the
weakest tag** of the RHS to the generated artifact (the transparency-lattice ratchet, house rule #1) — a
concrete checker obligation extending DN-54's §4.1 RHS check, flagged so it is designed in, not discovered
later.

### §8.6 Naming sub-question (minor)

Whether the expression-position case reuses the `lower`/`derive` spelling verbatim or gets a distinct teachable
spelling (the §5-D "sugar" reading) is a **naming** decision (DN-02 law: plain-first, reuse-before-coin,
T-map/T-illuminate/T-learn gates), **not** a different mechanism. Recorded so it is a deliberate choice at
ratification, not a silent default.

---

## §9 Operational decision-procedure (skill-derivable — the `/native-translate` methodology)

This section is worded so a reusable skill (working name **`/native-translate`**, FLAGGED §12) can be authored
directly from it. **Input:** one Rust construct / surface gap. **Output:** a classified native-translation
record (a DN-99-register-style row). The procedure operationalizes §2 + DN-109's ratchet, grounded in cited
corpus lookups (prefer tero-cited over hand-grep — VR-5).

**Step 1 — Verify-first (mitigation #14).** Before classifying, confirm the construct is not already closed:
`mcp__tero__query_by_id` / `text_search` the DN-99 register + the delta ledger for the construct; grep the
grammar/`mycelium-l1`/`lib/compiler` for an existing node or closure. If already closed, record the landed
basis instead of re-deciding (the codebase is ground truth).

**Step 2 — Identify the underlying PROBLEM, not Rust's mechanism.** Ask "what does this construct *accomplish*
for the programmer?" — not "what Rust feature is it?" (DN-109 F1; the whole point of §2). E.g. `#[derive(Clone)]`'s
problem is *"generate a structural operation for a type"*, not *"run a proc-macro."*

**Step 3 — Classify against the §2 taxonomy** (a decision tree, correlated with DN-109's decidability axis):

1. **Is there a direct first-class Mycelium-native equivalent, auto-emittable and semantics-preserving?**
   → **Adaptation** (DN-99 `closed`/`cl`; DN-109 Mechanical). *Lookup:* DN-99 register `closed` rows; the
   lexicon Active constructs.
2. **Else, does Mycelium solve the same problem by a DIFFERENT native convention** (value-semantics/functional
   update, bounded control, errors-as-values, explicit `swap`)? → **Solution** (DN-99 `idiom` different-path;
   DN-109 F1 / D-classes; DN-106 §3a). *Lookup:* DN-106 (mutation→functional), RFC-0007 §4.8 (`for`), DN-94/RT1
   (excluded shared-mut), the delta ledger §4/§5. If the mapping is safe/mechanical → auto-emit; if it needs
   judgment `syn` can't supply (aliasing/boundedness) → flag WITH `suggested_idiom`, not bare refusal.
3. **Else, is there a close native form but with a real delta** (dropped capability, fall-through caveat)?
   → **Approximation** (DN-99 `idiom` + caveat; DN-109 Heuristic). *Rule:* the delta MUST be explicit, honest,
   and never-silent (VR-5); tag the gap, do not paper over it.
4. **Else (no full native form yet)** → **Bridge** (DN-99 `tr-only`/`open`; DN-109 Judgment). A temporary,
   clearly-marked crossing (transpiler flag, `wild`/FFI boundary, library-with-parser) with a `suggested_idiom`;
   bare never-silent refusal only as the last resort (DN-106 §3a).

**Step 4 — Apply the DN-109 §3.2 ratchet to any auto-fire.** A mapping auto-emits only if it is (1)
semantics-preserving under value semantics, (2) inserts no `swap` (S1), (3) upgrades no guarantee tag (VR-5),
(4) is EXPLAIN-recorded. Any clause failing → downgrade from Adaptation/Solution-auto to a flagged
Approximation/Bridge.

**Step 5 — For the metaprogramming/sugar case specifically:** if the construct's native answer is "a
mechanically-lowering surface sugar" (DN-106 gap-closure default), author it as a **§5-A term-level lowering
rule** — RHS a real core-grammar term, zero L0-node growth (DN-54 §6), `reveal`-able (M-1051). Confirm it is
NOT in the deliberate-exclusion set (DN-106 principle 2) before sugaring.

**Step 6 — Produce the artifact:** a register-style record with columns *{construct, problem, class
(Adaptation/Solution/Approximation/Bridge), native target (with `file:line`/DN cite), explicit delta, honest
tag (Exact/Proven/Empirical/Declared), reveal-able lowering path, DN-109 bucket, never-silent behavior}*.
Emit it into the DN-99 register / DN-109 `idiom_choices` manifest — one shared EXPLAIN trail, not a new
artifact (DN-109 §7-c "extend, don't add").

---

## §10 Relation to the Rust-macro transpiler side (DN-100 / M-1032 / M-875) — complementary, not superseding

DN-100 / M-1032 (ENB-9) and the M-875 stub concern the **Rust-macro side**: a `cargo expand` / expand-first
pre-pass that expands *Rust* macros **before** transpilation. **This DN is the native TARGET those expansions
map INTO**, and the two meet cleanly:

- A Rust `#[derive(…)]` / `impl_*!` macro (DN-100 §2's dominant J1 population) → a Mycelium **`lower` rule +
  `derive` sites** (Adaptation). The expand-first pass and the native `derive` facility are **two ends of one
  bridge**: the transpiler need not even *expand* an `impl_*!` if it can map the macro to a `lower` rule
  directly — a **higher-fidelity** path than text expansion (and one that sidesteps DN-100 §3's "expanded
  `Narrow` impls still gap" problem, because the native `derive` target is idiomatic, not desugared-past-the-
  surface).
- A Rust `format!`/`matches!`/`vec!` (J3) → a Mycelium **expression sugar rule** (§5-A extension) or a
  builder-API + trait (§5-E), not a hand-expansion.

**Disposition (FLAG, §12):** M-875 / M-1032 stay scoped to the **transpiler-side** expand-first question,
already answered by **DN-100** — this DN does **not** supersede them. It **redirects/cross-links**: M-875 and
M-1032 gain a `corpus:DN-110` `doc_refs` pointer noting the *native target facility* is designed here, and a
**new implementation epic** is FLAGGED for the native facility itself.

---

## §11 Open questions (NOT decided here — house rule #3 / VR-5)

1. **§8.2 — the expression-position hygiene/scoping model** (definition-site vs use-site resolution;
   capture-avoiding substitution; affine-`Substrate`/`consume` interaction). *The single most consequential
   open question; the facility's soundness rests on it.*
2. **§8.1 — J5 foreign-syntax macros:** permanent deliberate-exclusion, or a later **Bridge**
   (library-with-parser, string/data + `certified`-checked parser)? (Neither the corpus nor the maintainer's
   framing forces this; flagged, not guessed.)
3. **§2 taxonomy home:** its own **companion DN** as the conceptual parent, or an **append-only extension of
   DN-109**? (Reasoned recommendation in §12 — FLAGGED for the maintainer, not decided here, to avoid racing
   this DN on a Doc-Index number.)
4. **§8.6 / §5-D naming:** reuse `lower`/`derive` verbatim for expression position, or a distinct teachable
   spelling? (A naming decision under the DN-02 gates, not a mechanism change.)
5. **Elaboration architecture:** the expression-position matcher/substitution as a *separate nanopass*
   (per-pass IL-grammar checkable, DN-38 §2/§8.2) vs folded into `elab.rs` — the same fork DN-38 §8.2 left open,
   now with an expression-position instance.
6. **`reveal` sequencing (M-1051):** ship a v0 inspector gating this facility, and at what fidelity
   (text-dump-with-caveat vs true L0-term view vs the `certified` round-trip)? (DN-38 §8.3.)
7. **Compile-time computation boundary (DN-55):** confirm static specialization fully owns J4, or name the
   residual J4 need it does not cover.

---

## §12 FLAGs (append-only rows the integrating parent applies — this note edits none of these)

`docs/Doc-Index.md`, `CHANGELOG.md`, and `tools/github/issues.yaml` are **integration-owned** (leaves FLAG,
the integrating parent applies once). **FLAG to the integrator (main):**

- **`docs/Doc-Index.md`** — add a Design-Notes row for **`DN-110 — Mycelium's Native Facility for the Role Rust
  Fills with Macros (Generative-Lowering + Sugar Rules)`** (`docs/notes/DN-110-Native-Metaprogramming-And-Sugar-Lowering-Facility.md`), Status **Draft (2026-07-11)**. Row prose can draw from the header table + §2 + §6.
- **`CHANGELOG.md`** — an "Added (design, pending ratification)" entry for DN-110 (the native-metaprogramming
  facility DN + the Adaptation/Solution/Approximation/Bridge translation taxonomy; ranked alternatives;
  recommends generalizing `lower`/`derive` to expression-position sugar rules with `reveal` as the transparency
  spine).
- **`tools/github/issues.yaml` — M-875 disposition: redirect, do NOT supersede.** M-875 stays scoped to the
  *transpiler-side* expand-first design (already answered by DN-100). Add `corpus:DN-110` to its `doc_refs`
  with a note: *"native TARGET facility for the macro role is designed in DN-110 (complementary — a Rust
  `#[derive]`/`impl_*!` maps to a Mycelium `lower`/`derive`; the transpiler-side expansion is DN-100)."*
- **`tools/github/issues.yaml` — M-1032 disposition:** unchanged scope (transpiler ENB-9); optionally add
  `corpus:DN-110` cross-ref for the native-target relationship (§10).
- **`tools/github/issues.yaml` — NEW issue to mint (native-facility implementation epic, design-gated):**
  e.g. **"native metaprogramming/sugar facility — generalize `lower`/`derive` to expression-position sugar
  rules (DN-110 Rank 1)"**, `depends_on: [M-812, M-1051]` (builds on landed `lower`/`derive`; co-gated on
  `reveal`/expand-on-demand), `doc_refs: [corpus:DN-110, corpus:DN-54, corpus:DN-38, corpus:DN-106]`, language/
  self-hosting phase. Its DoD = §3.6. Its **§8.2 hygiene model** should be its own gating sub-design (a Draft DN
  or RFC) before code.
- **`tools/github/issues.yaml` — NEW issue to mint (methodology skill, follow-up deliverable):** a
  **`/native-translate` skill** operationalizing §9 (classify a Rust construct as Adaptation/Solution/
  Approximation/Bridge; run the cited corpus/tero lookups; emit the register-style record into the DN-99 /
  DN-109 manifest). `doc_refs: [corpus:DN-110, corpus:DN-109, corpus:DN-99, corpus:DN-106]`. **Do not author the
  skill here** — FLAGGED as a follow-up (§9 is its authoring spec).
- **`CLAUDE.md`** — no change proposed.

### §12.1 Reasoned recommendation on the §2 taxonomy's home (FLAG — maintainer decides; §11.3)

**Recommendation: a small, focused *companion DN* as the conceptual parent — NOT an append-only extension of
DN-109** — but scoped tightly to the *classification vocabulary + decision procedure*, explicitly deferring the
*decision engine + EXPLAIN manifest* to DN-109 and the *sugar-lowering mechanism* to DN-110/DN-106.

**Tradeoff (both sides, honestly):**

- *For a companion DN.* The taxonomy is **broader than DN-109's transpiler-idiom + L5-structural scope** — it
  is a general translation *principle* already referenced (under different local vocabularies) by **≥4 corpus
  artifacts**: DN-99's register Status column, DN-109's L4 buckets, DN-106's gap-closure default + exclusion
  set, and DN-110 (this note). A cross-cutting principle cited by that many places benefits from **one citable
  anchor** that each instantiates ("this mapping is an Adaptation per DN-1xx"), rather than being buried as an
  appendix to a transpiler DN whose title (idiom-optimal transpilation) under-scopes it. The companion-DN cost
  is **low** — it is a *vocabulary + decision-procedure* doc (§2 + §9), not a mechanism — and it is the natural
  home for the `/native-translate` skill's methodology.
- *For a DN-109 extension.* DN-109 already carries the L4 buckets, the ratchet, the EXPLAIN manifest, and the F1
  "map PROBLEM→native SOLUTION" reframe; the taxonomy is arguably the *problem-side vocabulary* for the same
  framework, and DN-109 already took **append-only ratification addenda** (its F1 reframe). Co-locating keeps
  the vocabulary beside the decision engine and the manifest that records it — *one* place to look. **Con:**
  DN-109 is **Accepted** and transpiler/structural-scoped; grafting a *general* principle (that also governs L1
  language gap-closure, DN-106/DN-99 territory, not just transpiler idiom) onto it **under-scopes and buries**
  the principle, and appending normative *new* vocabulary to an already-Accepted DN strains the append-only
  discipline (an addendum should *resolve/record*, not introduce a new cross-cutting framework).

**Why the companion DN wins on balance:** the taxonomy's reach (L1 gap-closure *and* transpiler idiom *and* the
kernel-unfrozen north star) exceeds any single existing DN's scope, so it wants a **parent**, not a **host**.
The decisive tie-breaker is **separation of concerns (KC-3/SoC)**: DN-109 owns the *decidability axis + engine*,
DN-106 owns the *sugar-transparency policy*, DN-110 owns the *metaprogramming mechanism* — a companion DN owns
the *relationship-axis vocabulary + classification procedure* that all three instantiate, each cross-linking to
it. This is cleaner than overloading DN-109. **But this is a genuine judgment call, flagged for the maintainer,
not decided here** — and I do **not** author that companion DN (it would race this one on a Doc-Index number,
and its scope is the maintainer's to set).

---

## Ratification (maintainer, 2026-07-10)

**Recorded decision (append-only — this note's original §1–§12 text above is unchanged; this section adds
the ratification, per house rule #3):**

1. **§1 role decomposition accepted.** The five-job decomposition of the macro role (J1 derive-gen / J2
   delegation / J3 expression sugar / J4 compile-time computation / J5 foreign-syntax DSL) is confirmed:
   J1/J2 are already native + landed (`lower`/`derive`/`object`, M-811/M-812 — Adaptation), J4 is native
   via static specialization (DN-55 — Solution), J5 is a Bridge / flagged exclusion (§8.1), and **J3 —
   expression-position sugar — is the single real gap** this note's new scope covers.
2. **Rank-1 accepted.** §6's Rank 1 — generalize the landed `lower`/`derive` term-level lowering framework
   (DN-54/M-812) to expression position, with `reveal`/M-1051 as the mandatory transparency spine and
   `object`/`via` retained for composition (§5-A) — is accepted as the recommended mechanism, over §5-B
   (pattern macros), §5-C (comptime), and §5-D (a parallel `sugar` keyword, folded into Rank 1 as drafted).
   Rank 2 (§5-E, blessed-core desugarings) is confirmed as the complementary fallback layer, not a
   competing choice.
3. **§8.2 hygiene mechanism accepted as the basis.** The expression-position hygiene/scoping model —
   def-site resolution of RHS free identifiers, `%`-namespace freshening of RHS binders (reusing the
   landed `Elab::fresh` gensym), partition-safe substitution of use-site argument terms, and affine/type
   checking on the *expanded* L0 — is accepted as the design of record for §8.2, on the basis of the
   companion note **`docs/notes/DN-110-8.2-hygiene-deepdive.md`** (itself ratified at this same
   maintainer pass — see that note's own "Ratification" section). That note's §9 tractability verdict
   ("keep Rank-1; §8.2 downgraded from 'sharpest open question' to 'mechanism identified, dominant hazard
   already mitigated, two bounded residuals to prototype'") is accepted, and its E1 + E3 experiments are
   commissioned as the Rank-1 go/no-go for J3 (tracked via a new follow-up issue, §Follow-up below). The
   note's own §10 residual open questions (OQ-H1…OQ-H6) are **not** dispositioned by this pass — they
   stay open, flagged rather than guessed (G2/VR-5), carried into the follow-up implementation epic and
   the E1+E3 experiment issue.
4. **CRITICAL — Accepted is a design ratification, NOT Enacted (house rule #3 / VR-5).** This ratifies
   the *design* named in points 1–3 above — the role decomposition, the Rank-1 mechanism choice, and the
   §8.2 hygiene basis. It does **not** ratify a built, tested, or verified mechanism: no code has landed
   for J3's expression-position sugar rules, `reveal`/M-1051 (the mandatory transparency spine this
   facility is co-gated on, §3.4/§8.4) has not shipped, and the E1/E3 hygiene experiments have not run.
   **The facility's guarantees stay `Declared` throughout** — no guarantee tag anywhere in this note or
   its companion is upgraded past its basis by this ratification (VR-5). Per house rule #3, `Enacted`
   requires stepping through `Accepted` first and means *fully implemented/landed, complete and stable* —
   this DN **must not** be treated as Enacted, and will not move to Enacted until: (a) the expression-
   position argument matcher, capture-avoiding substitution, and hygiene machinery described in §5-A/§4
   of the companion note are implemented in the elaborator; (b) the E1 and E3 experiments (and, per the
   companion note's own recommendation, E2/E4/E5) have run with a PASS verdict; and (c) `reveal`/M-1051
   has shipped and its round-trip fidelity has been checked (E3's own gating criterion — if E3 fails,
   the honest fallback per this note's own §6 is Rank-2, blessed-core desugarings only, for J3). Until
   then, any claim that native metaprogramming/sugar-rules are "available" or "hygienic" in Mycelium is
   `Declared`, not `Empirical` or `Proven` — say so plainly (house rule #4).
5. **Taxonomy carve-out (§2) — the four labels are provisional, not canonical.** The maintainer accepts
   §2's *classification behavior* (the four-way relationship-axis split and its use throughout §5–§9) as
   the working basis for this note's own recommendation, but the specific terms **Adaptation / Solution /
   Approximation / Bridge** are ratified only as **provisional, intuitive handles** — convenient for
   reasoning today, **not** locked in as canonical taxonomy vocabulary. The canonical Rust→Mycelium
   translation taxonomy, with refined/final terminology, is **deferred to a forthcoming companion
   DN-111 (Draft, not yet authored)** — per §12.1's own reasoned recommendation that this taxonomy
   warrants its own companion DN rather than a DN-109 extension (that home question is itself accepted:
   companion DN, not a DN-109 append). A pointer to this carve-out has been added at §2 (append-only,
   2026-07-10); do **not** treat "Adaptation"/"Solution"/"Approximation"/"Bridge" as frozen pending
   DN-111's authoring and ratification.
6. **§8.1 (J5 permanent exclusion vs. later Bridge) and the remaining §11 open questions are NOT
   dispositioned here** — genuinely unresolved, flagged rather than guessed (G2/VR-5), unchanged by this
   ratification pass.

### Follow-up (filed at this ratification — see `tools/github/issues.yaml` for the minted ids)

- A **native-facility implementation epic** (§12's FLAGGED issue), `depends_on: [M-812, M-1051]`, DoD
  from §3.6, noting the §8.2 hygiene model is its own sub-design carrying OQ-H1…OQ-H6.
- An **E1 + E3 hygiene experiment prototype** issue, `depends_on: [M-1051]` (E3's `reveal` round-trip
  needs M-1051; M-919, the affine tracker E5 needs, is already landed), noting OQ-H1 as the expression
  analogue of DN-54 §10 OQ-D.
- A **`/native-translate` skill** issue (§9's authoring spec, §12's FLAGGED follow-up deliverable).
- A **DN-111 taxonomy companion DN** authoring task (point 5 above).
- **M-875 / M-1032 redirect, not supersede** (§10, §12's FLAG): both stay scoped to the transpiler-side
  expand-first question, already answered by DN-100; both gain a `corpus:DN-110` `doc_refs` cross-ref
  noting DN-110 is the complementary native-target facility, not a re-decision of their scope.

---

## §13 Changelog

- **2026-07-10 (following the E1+E3 hygiene-experiment go/no-go) — §8.4 addendum.** Appended a
  dated addendum to §8.4 (append-only, no edit to the pre-existing §8.4 text): **M-1051
  Increment-3** (`certified_roundtrip`) and the **E3 experiment** are now built and run (see
  `docs/notes/DN-110-8.2-hygiene-deepdive.md`'s "E3 Result" addendum). The reveal spine moves
  `Declared → Empirical` **narrowly** — the L0-term-level round-trip claim, scoped to the
  reparseable fragment only — never a blanket "transparency is built" upgrade; M-1051 Increment-2,
  a `reveal` CLI, `certified`-mode checker wiring, and the M-1054 facility itself remain unbuilt.
  DN-110 stays `Accepted`, NOT `Enacted` (VR-5).
- **2026-07-10** — **Ratified (maintainer, house rule #3).** Status **Draft → Accepted** (design
  ratification, **NOT Enacted** — VR-5, the facility's guarantees stay `Declared` until implementation +
  the E1/E3 experiments + `reveal`/M-1051 land). §1's role decomposition confirmed; §6's Rank-1 mechanism
  accepted; §8.2's hygiene mechanism accepted as the basis, citing the companion
  `docs/notes/DN-110-8.2-hygiene-deepdive.md` (itself ratified at this pass). §2's taxonomy labels
  (Adaptation/Solution/Approximation/Bridge) ratified only as **provisional handles**, not canonical
  terminology — the canonical taxonomy is deferred to a forthcoming companion **DN-111 (Draft)**. §8.1
  (J5) and the remaining §11 OQs stay open, not dispositioned. Follow-up filed: a native-facility
  implementation epic, an E1+E3 hygiene-experiment prototype issue, a `/native-translate` skill issue, a
  DN-111 authoring task, and an M-875/M-1032 `corpus:DN-110` cross-ref redirect (not a supersede). See the
  "Ratification (maintainer, 2026-07-10)" section above for the full record. Append-only — the original
  §1–§12 design record above is unchanged.
- **2026-07-11** — DN-110 created (**Draft**). Answers the maintainer's design question — Mycelium's native
  construct(s) for the role Rust fills with macros — framed as **one instance of a general Rust→Mycelium
  native-translation taxonomy** (Adaptation / Solution / Approximation / Bridge, §2), grounded row-by-row in the
  existing corpus (DN-99 register Status column, DN-109 L4 buckets, DN-106 gap-closure default) and reconciled
  with DN-109's decidability axis as its orthogonal relationship-axis complement. Decomposed the macro role into
  five jobs (J1 derive-gen / J2 delegation / J3 expression sugar / J4 compile-time computation / J5
  foreign-syntax DSL), finding J1/J2 already native+landed (`lower`/`derive`/`object`, M-811/M-812 —
  Adaptation), J4 native via static specialization (DN-55 — Solution), J5 a Bridge/flagged exclusion, and **J3
  the single real gap**. Enumerated five alternatives and **recommended (ranked, unratified)** generalizing the
  landed `lower`/`derive` **term-level lowering-rule** framework to expression position, with `reveal`/M-1051 as
  the mandatory transparency spine — winning on transparency-by-construction (L0-term artifact), tractable
  value-semantics hygiene, **zero L0-node growth** (KC-3, DN-54 §6 guard), DRY reuse of landed work, and the
  **measured** porting-gap closure (DN-100). Added a **skill-derivable operational decision-procedure** (§9,
  for a `/native-translate` skill). Adversarially stress-tested: the J5 foreign-syntax ceiling, the **new**
  expression-position hygiene/scoping model (sharpest open question, §8.2), "zero kernel growth ≠ zero
  complexity," the not-yet-built `reveal` dependency, and the guarantee-tag-upgrade trap. FLAGGED (§12): the
  Doc-Index row, the M-875/M-1032 redirect (not supersede), a native-facility implementation epic, the
  `/native-translate` skill issue, and a **reasoned recommendation that the §2 taxonomy warrants its own
  companion DN** (not a DN-109 extension) — all for the maintainer, none authored here. `Declared` throughout
  (design proposal); `Empirical` only where citing landed `lower`/`derive` or register-verified DN-99 closures
  at their own source. Authored READ + DN only — no edit to `issues.yaml`/`CHANGELOG`/`Doc-Index`/grammar/
  `mycelium-l1`/`lib/compiler` (FLAGGED up). Append-only; status advances only by maintainer ratification
  (house rule #3/#4).
</content>
