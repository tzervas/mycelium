# Design Agent A — Swaps ergonomics & typing (2026-07-17)

| Field | Value |
|-------|--------|
| **Status** | **Draft** (council research — **not** Accepted; does not ratify) |
| **Agent** | A — Swaps ergonomics (use, management, typing) |
| **Model** | grok-4.5 (high effort research) |
| **Honesty** | Claims are **`Declared`** (corpus-grounded design) unless tagged `Empirical`/`Proven` |
| **Scope** | Mycelium only; no product code; no merge PRs |
| **Council** | [DESIGN-COUNCIL-SWAPS-TAGS-2026-07-17.md](./DESIGN-COUNCIL-SWAPS-TAGS-2026-07-17.md) |

> **Posture (VR-5 / G2 / house rule #3).** This recommends. It does **not** move any RFC/DN/ADR
> status. Non-negotiables: never-silent swaps (G2/S1/WF1–WF2), no silent guarantee upgrade (VR-5),
> prefer **deterministic machinery** over ad-hoc convention.

---

## 0. Premises verified (corpus, not invention)

| Premise | Basis | Tag |
|---------|-------|-----|
| `swap` is the **only** Repr-changing node | RFC-0001 §4.5 WF1; typing rule *(Swap)* | Declared (Accepted RFC) |
| Every `Swap` carries `PolicyRef` | RFC-0001 WF2; grammar `swap_expr` | Declared |
| Surface form is always lexical: `swap(e, to: T, policy: p)` | `mycelium.ebnf:309–310`; lang-ref §7; reject `02-swap-missing-policy.myc` | Empirical (conformance present) |
| No elaboration/ambient may insert a `swap` | RFC-0012 I1; RFC-0001 WF8; DN-109 D13 | Declared (Accepted) |
| Legal pairs + certificate kinds | RFC-0002 §3–§5; `swap-certificate.schema.json` | Declared |
| Library surface is explicit `Result<Swapped, SwapError>` | `docs/spec/stdlib/swap.md` §3; `mycelium-std-swap` | Empirical (landed crate) |
| Cert emission/check is **mode-gated**, never-silence is not | DN-29 §3.1; RFC-0034 / ADR-032 | Declared |
| Open ergonomics tension is **explicit** | RFC-0016 §8-Q3 deferred-with-direction; `swap.md` §7-Q2 | Declared |
| Transpile must **flag**, never auto-insert `swap` | DN-109 D13 (S1) | Declared |

Highest free DN slot at research time: **not minted here** (council may later promote a
recommendation into a numbered DN). This file is a **planning artifact**, not a DN.

---

## 1. Pain inventory (author UX/DX)

Concrete friction points an author hits when **using / managing / typing** swaps. Each cites a
file:section. Severity is design judgment (`Declared`), ordered by author-day tax.

### P1 — Mandatory dual annotation at every site (`to:` + `policy:`)

- **What:** Every representation change is
  `swap(x, to: Ternary{6}, policy: rt)` — both target and policy always lexical
  ([lang-ref §7](../../reference/language-reference.md); [ebnf `swap_expr`](../../spec/grammar/mycelium.ebnf)).
- **Why it hurts:** Honest (S1/WF2), but **repetitive** on multi-crossing pipelines. RFC-0001 §5
  already names this: *"Mandatory explicit `swap`s and `PolicyRef`s make representation changes
  wordy (mitigated by tooling/projections; the cost is intentional)."*
- **Not fixable by ambient:** RFC-0012 I1 forbids ambient insertion of swaps; R12-Q2 **rejected**
  default swap policy and preferred free swap sites over block-edge auto-convert.
- **Severity:** High (every multi-paradigm function).

### P2 — Certificate is a second value the author must thread

- **What:** Successful swap yields `Swapped { value, cert }` — never the value alone
  (`stdlib/swap.md` §3; crate `Swapped`). Surface IR typing (`Value<target>`) under-specifies how
  the certificate binds into the surface type story (RFC-0001 *(Swap)* result is `Value<target>`;
  the cert is Meta/side-channel vs first-class pair — dual stories).
- **Open question already filed:** `swap.md` §7-Q2 — explicit `Swapped<T>` vs
  implicit-by-default-but-inspectable (RFC-0012 ambient lesson). Disposition today: **explicit**
  until RFC-0016 §8-Q3 / M-540 per-ring pass resolves.
- **Why it hurts:** Authors either (a) always destructure and ignore `cert` (noise), or (b)
  plumb certs through every intermediate (cognitive load). No settled "bind value, park cert"
  sugar that preserves inspectability.
- **Severity:** High (API ergonomics of the signature op).

### P3 — Unnamed / untyped dynamic `Value` library surface vs typed surface `swap`

- **What:** Landed `std.swap` is **value-typed**: `&Value` in, non-generic `Swapped` out, with
  width/`delta` as extra runtime params (`swap.md` §7-Q4 correction). Surface language wants
  `Binary{8}` → `Ternary{6}` at the type checker.
- **Why it hurts:** Two mental models for one operation: (1) static typed keyword `swap`, (2)
  dynamic named ops `bin_to_tern` / `dense_to_vsa`. Width legality that RFC-0002 §4 / binary-ternary
  spec treat as **type-level** (`B_n ⊆ T_m` or type error) is partly **runtime** on the Rust surface.
- **Severity:** High (typing friction; see §2).

### P4 — Policy authoring is a separate, heavy subsystem

- **What:** `PolicyRef` is a content-addressed selection policy (RFC-0005; ADR-006). `std.swap`
  **consumes** policy, does not author it (`swap.md` Boundary). Tutorial/examples use bare names
  (`rt`, `roundtrip`) without teaching policy construction.
- **Why it hurts:** Author must learn (a) write swap, (b) invent/import a policy path, (c) know
  which policy is legal for which pair — before the first useful program. No **catalog of
  standard policies** with derived strength tags is first-class in the surface tutorial path
  (tutorial §5 shows `policy: rt` as a path token only).
- **Severity:** High (onboarding + management).

### P5 — Legal-pair matrix is normative but not author-facing as a type error UX

- **What:** RFC-0002 §5 table: Binary↔Ternary (in range), F32→BF16, Dense↔VSA, VSA↔VSA
  case-by-case; **no-statable-bound = type error**, not Declared gamble. Out-of-range bijection
  inverse is explicit `Option`/error.
- **Why it hurts:** Author discovers legality late (runtime `SwapError` / checker) unless the
  surface checker encodes the matrix. MissingConversion (RFC-0012) covers *absent* swap, not
  *illegal pair with wrong policy*. Diagnostics for "wrong pair / no bound / out of image" are
  split across checker, cert TV incompleteness (`NotValidated`), and swap errors.
- **Severity:** Medium–High (management + typing).

### P6 — Dual return channels: keyword `swap` vs named std ops

- **What:** Surface keyword + stdlib named functions for the same legal pairs. Naming still open
  historically (`swap.md` §7-Q1; RFC-0016 Q2 resolved phylum `std` but themed op names not fully
  fungal-lexicon'd).
- **Why it hurts:** "Do I call `swap(..., to: Ternary{6}, …)` or `std.swap.bin_to_tern`?" —
  relationship (desugar? equivalent? cert shape identical?) is not one-sentence-clear in the
  tutorial.
- **Severity:** Medium.

### P7 — Mode knobs change cert cost but not the surface tax

- **What:** DN-29 / RFC-0034: `fast` default turns **cert emission/check off**; Axis-B
  never-silence stays on. Swap stays lexically written in every mode (RFC-0002 footnote + ADR-032).
- **Why it hurts:** Author still pays **full syntactic tax** in `fast` while getting **no
  certificate object** — the verboseness is not mode-relative. Signal generation (EXPLAIN trace)
  is mode-tiered; the *source form* is not. This is intentional honesty, but it means
  ergonomics cannot be "solved" by mode alone.
- **Severity:** Medium (fast-path DX).

### P8 — Transpile / port path cannot help without human judgment

- **What:** DN-109 D13: numeric `as` / `.into()` → **flagged candidate swap, never auto-insert**.
  Language-completeness inventory maps silent cast → explicit `swap` as Idiomatic Remapping.
- **Why it hurts:** Zero-hand-port and dogfooding generate a **flag flood** of candidate swaps
  with no mechanical policy/target completion. Evidence of friction, not a product bug: S1 is
  working as designed and still taxes authors.
- **Severity:** Medium (scale of port waves); high for agent-driven migration.

### P9 — Certificate check incompleteness is another explicit branch

- **What:** M-210 TV may `NotValidated` a correct swap → explicit fallback, never silent pass
  (RFC-0002 §2; `CheckError::NotValidated`).
- **Why it hurts:** Author managing certified pipelines must handle: swap `Err`, check `Refuted`,
  check `NotValidated` + fallback — three refusal classes with different recoveries (airlock
  companion documents meet contamination but not this three-way branch ergonomics).
- **Severity:** Medium (certified-mode authors).

### P10 — Swap vs convert boundary is load-bearing and easy to misplace

- **What:** Representation change = `swap`; same-paradigm value conversion = `cmp`/`convert`
  (M-532). BF16→F32 widening is *value-exact* but not a certified swap pair (`swap.md` §7-Q3
  disposition: leave with convert unless maintainer flips).
- **Why it hurts:** Authors think "change the bits/type" = one concept; Mycelium splits by
  **Repr change** vs **value-level**. Tutorial stress is light; inventory mistakes will be common.
- **Severity:** Medium (conceptual; wrong module / missing cert).

---

## 2. Typing friction

How swap **types** and **certificates** burden the author today.

### 2.1 Core IR typing (simple, strict)

```
Γ ⊢ src : Value<R_src>
target : Repr
policy : PolicyRef
─────────────────────────────  (Swap)
Γ ⊢ Swap{src,target,policy} : Value<target>
```

- **Strength:** No coercion; paradigm mismatch is a type error without a written swap (FR-M3).
- **Friction:** Result type is **only** `Value<target>` — certificate and guarantee strength are
  Meta / side products, not part of the static type in RFC-0001's judgment. Surface wants both
  auditability *and* ordinary expression typing.

### 2.2 Legal-pair / bound legality is half-static, half-dynamic

| Check | Ideal locus | Today |
|-------|-------------|-------|
| Cross-paradigm without swap | Static (`MissingConversion`) | Specified (RFC-0012) |
| Illegal pair (no statable bound) | Static type error (RFC-0002 §5) | Declared intent; enforcement split IR vs library |
| Width legality `B_n ⊆ T_m` | Static for fixed widths | Spec'd in `swaps/binary-ternary.md`; Rust API takes `u32` widths |
| Out-of-image inverse | Dynamic `Err`/`None` | Correct (never silent) |
| Policy legal for pair | Static or cert-time | Policy is opaque `ContentHash` / path at surface |
| Bound strength (`Proven` vs `Empirical`) | Derived at cert build (RFC-0002 §3) | Not a surface type parameter |

**Typing burden:** Author cannot write `swap` and have the **type system** carry
`Exact-within-range` vs `Bounded(ε,δ)` as a **return type refinement** without manual Meta /
guarantee annotations (Agent B's lattice surface). The op is typed as "target Repr"; honesty
lives elsewhere.

### 2.3 `Swapped` is non-generic on the landed surface

- Spec correction (`swap.md` §7-Q4): no `Swapped<T>`; dynamic `Value` + cert.
- Surface language examples type the **expression** as `Ternary{6}` (tutorial), eliding how
  `Result`/`Swapped` monadic structure appears in Mycelium-lang (fallibility of swap is
  `Option`/`Result` per G2, but the keyword form in lang-ref is shown as if it were a pure
  expression in the return type of `f` — see tutorial §5 / lang-ref §7 snippets).

**Friction class:** **fallibility is under-illustrated in surface examples.** A function
`fn f(x: Binary{8}) -> Ternary{6} = swap(...)` is only honest if the swap is total for that
pair (legal widths + always-in-range). For partial inverses (`tern_to_bin`), the return type
must be `Option`/`Result` — authors will write total types over partial ops (silent-by-type
lie if the checker doesn't force it).

### 2.4 Ambient helps paradigms, not swaps

- `default paradigm` / `with paradigm` elide paradigm tags (RFC-0012) and can shorten `to: {6}`
  under ambient (accept `12-ambient-representation.myc`: `to: {6}`).
- They **cannot** elide `policy:` or insert the swap. So typing/ergonomics win is partial:
  ambient reduces `to:` noise, not policy or cert noise.

### 2.5 Certificate not in the type → airlock patterns stay manual

- Companion airlocks (`02-guarantee-airlocks.md`) show seal patterns that re-mint tags after
  Exact predicates — **pattern, not frozen API**.
- Swap certificates are the natural airlock artifact for lossy Dense/VSA paths, but there is no
  **typed** "value whose type mentions cert kind / strength" in everyday surface. Author manages
  certs as values, not as type indices.

### 2.6 Summary typing tax

| Tax | Manual today? | Deterministic relief possible? |
|-----|---------------|--------------------------------|
| Write `to:` | Yes (ambient can shorten form) | Partial (ambient + inference of target from ascription) |
| Write `policy:` | Always | Yes — **canonical policy table** keyed by legal pair (still lexical ref) |
| Thread cert | Yes if explicit `Swapped` | Yes — **cert-ambient / effect row** with EXPLAIN query |
| Legal pair | Discover late | Yes — **static legal-pair matrix** in checker |
| Strength tag | Manual / Meta | Yes — **derived** (already RFC-0002 rule); surface should not re-assert |
| Fallibility | Easy to omit in examples | Yes — **type of swap = Result/Option by regime class** |

---

## 3. Options (ranked)

Objective function (weights for Agent A — swaps only):

| Criterion | Weight | Notes |
|-----------|--------|-------|
| **C1 Never-silent / S1 / WF1–2** | hard gate | Option is invalid if it auto-inserts swaps or hides policy identity |
| **C2 Author tax reduction** | 0.30 | Fewer tokens / fewer concepts per honest swap |
| **C3 Typing clarity** | 0.25 | Legal pair, fallibility, cert relationship static where possible |
| **C4 Deterministic machinery** | 0.20 | Reproducible rules > sugar that needs judgment |
| **C5 KISS / KC-3** | 0.15 | No kernel bloat; library/checker preferred |
| **C6 Mode coherence** | 0.10 | Works with fast/balanced/certified (DN-29) |

Legend: **M** = deterministic machinery · **S** = surface sugar · **T** = tooling · **L** = library.

### Rank 1 — **Canonical policy catalog + static legal-pair checker** (M + L)  ★ recommended core

**Mechanism:**

1. Publish a **content-addressed standard policy catalog** (RFC-0005 objects) for each RFC-0002
   legal pair / regime class, e.g. `std.swap.policy.roundtrip_bt`, `…bf16_round`,
   `…dense_vsa_capacity`.
2. Checker encodes the **legal-pair table** as a pure function
   `(R_src, R_target) → Regime | TypeError` (RFC-0002 §5).
3. Optional **deterministic default policy binding**: if `policy:` names a catalog path *or*
   a new form `policy: default` that **elaborates to the unique catalog entry for that pair**
   (still a reified `PolicyRef` on the L0 node — identity preserved, EXPLAIN-able).
   **Not** "omit policy" (parse error today); **not** ambient insertion of the swap.

**Tradeoffs:**

- (+) Directly attacks P1/P4/P5; keeps S1 (swap still written; policy still a lexical slot).
- (+) Deterministic: same pair → same default policy hash (reproducible).
- (+) Mode-independent: works in `fast` (no cert check) and `certified`.
- (−) `policy: default` must be carefully specified so it is not a black box — EXPLAIN must
  expand to the resolved catalog hash (RFC-0012 ambient lesson: elided, not hidden).
- (−) Does not by itself solve cert threading (P2).

**Rejected non-goals:** silent auto-swap; inference of target from usage alone (RFC-0012 rejected).

### Rank 2 — **Typed swap regimes + fallibility-in-the-type** (M)

**Mechanism:**

Classify each legal pair into a **regime typeclass** (not OOP — static):

| Regime | Result type of surface `swap` | Strength derivation |
|--------|-------------------------------|---------------------|
| `LosslessWithinRange` (total enc) | `Result<T, OutOfRange>` or total if widths prove inclusion | `Exact` within range |
| Partial inverse | `Option<T>` / `Result` | `Exact` on image |
| Bounded (ε) | `Result<T, SwapError>` + bound in Meta/cert | from `BoundBasis` |
| Bounded (ε,δ) | same + δ | Empirical default |

Checker refuses a total return type over a partial regime (fixes tutorial honesty gap §2.3).

**Tradeoffs:**

- (+) Attacks P3/P5 and fallibility under-illustration.
- (+) Aligns surface keyword with RFC-0002 classes.
- (−) Surface type system needs regime lattice or traits (`SwapRegime`) — design cost; couples
  to Agent B if guarantee appears in types.
- (−) Does not reduce token count of `swap(...)`.

### Rank 3 — **Cert ambient / "value-forward, cert-queryable"** (M + S) — resolves §7-Q2

**Mechanism (direction already accepted at library level, RFC-0016 §8-Q3):**

- Expression type of `swap` is the **target value type** (or `Result` of it).
- Certificate is **always produced** (in modes that emit) and bound into a **reified, queryable**
  channel: `meta_of` / `cert_of` / EXPLAIN — not requiring destructuring at every site.
- In `certified`, missing cert channel is a hard error; in `fast`, EXPLAIN trace still records
  which policy/target (DN-29 signal generation ≥ middle tier; `fast` lean display).

**Structural guarantee:** it remains **impossible** to obtain a converted value without a
cert/trace identity existing in the runtime Meta story (C1/C3) — only the *call-site syntax*
stops forcing `let Swapped { value, cert } = …`.

**Tradeoffs:**

- (+) Attacks P2 without abandoning never-silent.
- (+) Matches ambient philosophy (elided, inspectable).
- (−) Must not make cert "optional" in certified mode (mode coherence).
- (−) Harder to teach "where did my cert go?" without excellent tooling (T).

### Rank 4 — **Target elision from ascription / return type** (S + M)

**Mechanism:**

```mycelium
fn f(x: Binary{8}) -> Ternary{6} =
  swap(x, policy: std.swap.policy.roundtrip_bt)
// elaborates to: swap(x, to: Ternary{6}, policy: …)
```

Only when a **unique** expected type supplies `to:`. Ambiguous expected type → explicit refuse
(never guess). Still requires written `swap` and `policy` (or Rank-1 default).

**Tradeoffs:**

- (+) Reduces P1 token tax when types already say the target.
- (+) Deterministic under unique expected type.
- (−) Non-local (return type far from site) — partially the black-box concern RFC-0012 rejected
  for *repr inference from usage*; mitigated because the **swap keyword remains** (only `to:`
  elided, like ambient paradigm elision).
- (−) Must not interact badly with overload/generics.

### Rank 5 — **Unified surface: keyword is the only call form; std names are sugar** (L + S)

**Mechanism:**

Declare `std.swap.bin_to_tern(x, m, p)` **desugars** to
`swap(x, to: Ternary{m}, policy: p)` (and dual). One typing rule, one EXPLAIN shape. Named
functions become documentation/discovery entry points, not a second type system.

**Tradeoffs:**

- (+) Attacks P6.
- (+) KISS for authors.
- (−) Width params as type-level vs value-level still need Rank 2.
- (−) Library API freeze (DN-66 / ADR-045 window) — sugar layering must not break differential.

### Rank 6 — **Tooling-first: swap dashboard + autofix candidates** (T)

**Mechanism:**

- LSP/EXPLAIN "every crossing + bound" (already the R12-Q2 auditability path via M-345).
- Autofix: given `MissingConversion { from, to }`, offer **insert**
  `swap(…, to: to, policy: <catalog default>)` — human accepts (never silent insert).
- Transpile: D13 flags become **structured** "candidate swap with suggested policy" not free text.

**Tradeoffs:**

- (+) Huge DX without language change; helps P8.
- (+) Compatible with all ranks above.
- (−) Does not reduce tax for authors without IDE; not sufficient alone for language-complete
  ergonomics (tension A).

### Rank 7 — **Aggressive: omit policy; infer from ambient decision table** — **reject**

RFC-0012 already rejected default swap policy and auto-insert. Omitting `policy:` fails
conformance (`02-swap-missing-policy.myc`). Inferring policy from ambient decision tables
without a lexical slot reopens black boxes (G2/ADR-006).

**Verdict:** **Out of bounds** for this council unless maintainer explicitly reopens R12-Q2
(not recommended here).

---

## 4. Recommendation (Draft only)

### Package: **A-core = Rank 1 + Rank 2 + Rank 3**, with Rank 4/5 as follow-ons and Rank 6 parallel tooling

**Objective-ranked package:**

| Layer | What lands | Kind | Primary pains |
|-------|------------|------|---------------|
| **A1** | Legal-pair matrix in checker + catalog policies | M+L | P4, P5 |
| **A2** | `policy: default` → unique catalog `PolicyRef` (EXPLAIN expands) | M+S | P1, P4 |
| **A3** | Regime-driven result types (total / Option / Result) | M | P3, fallibility lie |
| **A4** | Cert ambient: value-forward, cert-queryable (resolve `swap.md` §7-Q2) | M+S | P2 |
| **A5** | Optional `to:` elision from unique expected type | S | P1 residual |
| **A6** | Named std ops = sugar over keyword | L+S | P6 |
| **A7** | LSP insert-swap + structured transpile candidates | T | P8, P9 UX |

**Why this ranking (adversarial self-check):**

- **Does not violate S1:** every option that survived keeps a written `swap`. Policy default is
  **resolution of a lexical keyword**, not omission of the swap.
- **Does not upgrade guarantees:** strength still **derived** from BoundBasis (RFC-0002 §3);
  Rank 2 only makes fallibility honest.
- **KISS:** prefers checker + library catalog over new kernel nodes. No fifth paradigm; no new
  IR node beyond existing `Swap`.
- **Mode coherent:** A4 must specify cert channel vs EXPLAIN-trace channel per DN-29 tiers;
  A2 works even when cert check is off (`fast`).
- **Disconfirming evidence against "just more sugar":** RFC-0001 already said tooling/projections
  mitigate verbosity; RFC-0012 already spent the ambient budget on paradigms. Further sugar
  without **deterministic tables** (legal pairs + catalog policies) would paper over management
  pain (P4/P5). Hence Rank 1 before Rank 4.

### What we deliberately do **not** recommend

| Temptation | Why not |
|------------|---------|
| Auto-insert swap at block edges | Rejected R12-Q2; taxes interleaving; silent |
| Infer policy with no lexical marker | Breaks WF2 / reject suite |
| Infer Ternary/VSA from Rust types in transpile | DN-109 D12/D13 — judgment only |
| Make `fast` omit writing `swap` | Never-silent is Axis B, mode-independent (DN-29) |
| Generic `Swapped<T>` as the *only* story without ambient | Leaves P2 tax; §7-Q2 already leans ambient direction at library level |

### Adversarial stress-test verdict

| Attack | Package response | Residual risk |
|--------|------------------|---------------|
| `policy: default` is a black box | EXPLAIN must expand hash + catalog id; identity of L0 includes resolved PolicyRef | Tooling lag |
| Author assumes total type on partial inverse | Rank 2 forces Option/Result | Needs checker enforcement before examples update |
| Cert ambient loses audit in certified | Mode rule: certified requires cert materializable | Spec drafting care |
| Catalog freezes wrong policy | Policies are content-addressed; supersede by new hash, don't rewrite | Process discipline |
| Two surfaces diverge again | Rank 5 sugar law | Until A6 lands, document equivalence as Declared |

**Verdict:** Package is **sound under G2/S1** if A2 is specified as *elision of policy path with
mandatory resolve-and-record*, not *absence of policy*. Highest residual risk is **A4 mode
interaction** (needs joint review with Agent B / DN-29).

### Definition of Done (for later maintainer ratification — not claimed now)

A future DN/ADR accepting this package is **Accepted** only when:

1. Legal-pair checker behavior is specified with conformance accept/reject fixtures.
2. Catalog policies exist as content-addressed artifacts with EXPLAIN projections.
3. `policy: default` (or chosen spelling) is grammar + elaboration specified; reject suite
   still forbids *missing* policy keyword/slot if that remains the rule — or documents the
   new form as the only elision.
4. Regime → result-type mapping is normative and tested.
5. §7-Q2 disposition is recorded (explicit `Swapped` vs cert-ambient) with mode matrix.
6. No guarantee tag upgraded without basis; differential holds for std.swap.

---

## 5. Open questions for maintainer

1. **§7-Q2 call:** Prefer **cert ambient (A4)** as the default authoring model, or keep **explicit
   `Swapped` forever** for the signature op (maximum honesty, maximum tax)?
2. **Spelling of policy elision:** `policy: default` vs `policy: _` vs catalog-required named
   path only (no elision — only Rank 1 catalog without A2)?
3. **`to:` elision (A5):** Accept under unique expected type, or keep `to:` always written
   (symmetry with always-written swap)?
4. **BF16→F32 / wideners:** Confirm stay in `convert` (current disposition) so swap typing
   stays "legal pairs only"?
5. **M-540 scope:** Is the per-ring ergonomics pass the right vehicle for A1–A4, or a dedicated
   **Swap Ergonomics DN** (next free DN after council)?
6. **Mycelium-lang keyword vs Rust std.swap:** Until self-host, is the **keyword form** the
   normative author model, with Rust APIs considered reference-only?
7. **Certified incompleteness UX (P9):** Library combinator for "swap then check with named
   fallback" as std sugar, or leave to recover/diag?

---

## 6. Suggested work items (post-council re-rank; ids optional / Declared)

| ID (suggested) | Title | Depends | Kind |
|----------------|-------|---------|------|
| **M-swap-A1** | Static legal-pair matrix in `myc check` / L1 (RFC-0002 §5 table as data) | RFC-0002 | machinery |
| **M-swap-A2** | `std.swap.policy` catalog phylum (content-addressed defaults per pair) | RFC-0005, A1 | library |
| **M-swap-A3** | Surface: `policy: default` elaboration + EXPLAIN expand | A2, grammar | surface |
| **M-swap-A4** | Regime → `Result`/`Option` typing rules for `swap_expr` | A1, RFC-0001 typing note | typing |
| **M-swap-A5** | Resolve `swap.md` §7-Q2: cert ambient design (with DN-29 mode matrix) | RFC-0016 Q3, DN-29 | design→spec |
| **M-swap-A6** | Keyword ↔ named-op desugar equivalence + docs | A4 | library/docs |
| **M-swap-A7** | Optional `to:` elision from unique expected type | A3 | surface |
| **M-swap-A8** | LSP: MissingConversion → insert-swap code action (catalog policy) | A2, M-345 | tooling |
| **M-swap-A9** | Transpile D13: structured candidate swap records (policy suggestion) | A2, DN-109 | transpile |
| **M-swap-A10** | Tutorial/lang-ref honesty pass: fallible swap types; no total lie | A4 | docs |
| **M-540** (existing) | Per-ring ergonomics — **schedule A1–A5 as Ring-1 first** | RFC-0016 Q3 | existing |

Wave placement suggestion: **design capture (this council) → small DN for A1–A5 → implement
checker/catalog before sugar (A3/A7) → tooling parallel.**

---

## 7. Cross-links for Agents B / C

- **Agent B (tags/Meta):** Rank 2/3 touch where guarantee strength appears; A4 cert ambient is
  the swap-side of lattice UX. Airlock patterns need a real `recertify`/`seal` API story.
- **Agent C (synthesis):** Stress-test A2 (`policy: default`) hardest — it is the only new
  elision near a never-silent boundary. Rank 7 rejection should stay rejected unless C finds a
  deterministic form that still records PolicyRef.

---

## Meta

- **2026-07-17 — Draft (Agent A council report).** Corpus-grounded pain inventory, ranked options,
  and package recommendation for swap ergonomics/typing. No RFC/DN status change. No product code.
