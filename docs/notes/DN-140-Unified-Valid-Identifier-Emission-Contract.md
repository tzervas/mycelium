# Design Note DN-140 — A Unified Valid-Identifier Emission Contract for the Rust→Mycelium Transpiler

| Field | Value |
|---|---|
| **Note** | DN-140 |
| **Status** | **Accepted** (2026-07-13) — ratified by the strict project DN-review gate (a 3-round adversarial pass over grounding · VR-5 · G2 · append-only · native-solution · KC-3/KISS/DRY · adversarial injectivity · DoD · consistency; all nine clean). The type↔method boundary guarantee's upgrade to *injective by construction* rests on the **checked** length-prefix (prefix-free) decode argument of §7 — `vt`/`vm` are non-empty and non-digit-first by `valid_ident`'s totality contract (output matches `^[A-Za-z_][A-Za-z0-9_]*$`), so the maximal-decimal-run decode is unambiguous and the encode is injective (independently re-derived at the gate; no counterexample). The **mechanism** tag stays `Declared` until the build's length-prefix-injectivity and totality-over-Unicode property tests witness it (house rule #1 — no upgrade past a checked basis). Ratification builds nothing: DN-139 → **Superseded by DN-140** and every shared-file row (§10) remain **FLAGGED** for the integrating parent, not applied here. Forward from: **Draft** (2026-07-13) — reasoned against `dev@2cd9b773` and the DN-139 draft at `claude/leaf/phase2-next-waves-scoping@ee33e4dc`; design-only (every mechanism `Declared` until differential-witnessed against the real `myc check` oracle, VR-5); edits **only** itself (`crates/**`, `CHANGELOG.md`, `docs/Doc-Index.md`, `tools/github/issues.yaml` all FLAGGED, §10). |
| **Task** | Decide the **single, unified valid-Mycelium-identifier emission contract** the transpiler applies to *every* Rust name that lands in an identifier position. Today the same underlying defect — "a Rust name that is not a legal Mycelium identifier" — is handled by two disconnected mechanisms with different fates: (1) **reserved-word collisions** are gapped whole-item (`crates/mycelium-transpile/src/reserved.rs::guard_ident`) or, for nodule-path *segments*, escaped `word→word_kw` (`sanitize_nodule_path`); DN-139 proposes extending that escape to program identifiers. (2) **Illegal-character shapes** — generic brackets leaking into a mangled name (`mangled_inherent_fn_name`'s `{self_ty_text}__{method}` → `DeclaredTime[T]__new`, `emit.rs:4302-4304`) — are a **hard `myc check` parse failure**, not even a clean gap (the "D4-mangling" regression that dipped std-time `checked_fraction` 6.34%→5.35%). Both are the *same problem*: map an arbitrary Rust identifier-position name to a valid Mycelium identifier, deterministically and injectively, applied consistently at the declaration **and** every reference across all files and all positions, so cross-file consistency holds **by construction**. |
| **Decides** | *Proposes, for ratification:* **one total function `valid_ident`** — a pure, deterministic, position-independent map from an arbitrary string to a legal Mycelium identifier — that **subsumes** the reserved-word escape (DN-139), the nodule-path-segment escape (`sanitize_nodule_path`), and the currently-unhandled illegal-character class (brackets `[]`, path separators `::`, angle brackets, commas, whitespace, and every other char outside `[A-Za-z0-9_]`). The map is (a) **identity on already-valid, non-reserved names** (so a stable exported symbol is never touched); (b) a fixed `_kw` suffix on a valid-shaped reserved word (DN-139's rule, subsumed); (c) a delimited per-character escape (`_u{HEX}_`, the Unicode scalar value in hex — covering illegal ASCII punctuation **and** legal-Rust non-ASCII identifiers alike), followed by the `_kw` step if the escaped form is itself reserved. `valid_ident` is **idempotent** (`valid_ident(valid_ident(x)) = valid_ident(x)`), which is what prevents double-escaping when it composes with the mangler and the nodule-path sanitizer. The **type↔method boundary** in the composed inherent-fn mangle is made injective **by construction** via a length-prefix encoding (§7, maintainer input 2026-07-13), so the naive-`__` boundary collision (§8①) is *eliminated*, not merely caught. The one residual the char-escape map cannot prevent by construction (§8②) is caught by a **build-time per-unit self-collision GAP** — never a silent overwrite (G2). |
| **Feeds** | DN-139 (subsumes it — DN-139's `word→word_kw` becomes the reserved-word branch of this contract; recommend DN-139 → **Superseded by DN-140** on ratification, append-only); `crates/mycelium-transpile/src/reserved.rs` (`sanitize_nodule_path` becomes the per-segment specialization of the same contract); `crates/mycelium-transpile/src/emit.rs` (`mangled_inherent_fn_name` @`4302-4304`, its resolution-key recorder `record_local_mangled_assoc_fn` @`324`, and `emit/calls/qualified_assoc.rs` — the D4 composition site, §7); `docs/planning/DN-136-phase2-wave3-worklist.md` §3 (ReservedWord rank-7, 53 gaps) + the D4 bracket-regression hot-fix in flight. |
| **Grounds on** | **G2 (never-silent)** — the rewrite is *reified in the emitted text itself* (`Exact_kw`, and a length-prefixed inherent-fn mangle like `_3Foo3new`, are visible to a reader/diff and mechanically invertible — EXPLAIN) plus an emitted `// Declared: renamed …` comment; a residual collision is a build-time GAP, never a hidden substitution. **DRY / KC-3 / KISS** — one function, one `_kw` suffix (`reserved::RESERVED_SEGMENT_SUFFIX`), one `_u{HEX}_` char-escape rule; the two existing escapers (`guard_ident`, `sanitize_nodule_path`) collapse into special cases rather than a third convention. **The injectivity property at `reserved.rs:159-166`** — the load-bearing precedent: a deterministic, injective, constant-suffix escape is cross-file-sound *without coordination* because the rule *is* the coordination. **VR-5** — the mechanism is `Declared` (unbuilt); the type↔method **boundary** injectivity is *by construction* (length-prefix, §7 — an argument stated, property-tested at build); the remaining char-escape aliasing is a documented residual with a build-time self-collision GAP as the real guarantee, **never** claimed `Proven`-globally-injective; and this note **corrects** the landed D4 mangler's overstated "collision-free by construction" claim (§8①) — the naive `__` scheme was *not* injective; the length-prefix scheme is. |
| **Definition of Done** | §9. In one line: **Accepted** requires the gate to confirm (a) the verified reject set (§2) is the *complete* set of Mycelium-illegal identifier shapes, checked against the lexer, not assumed; (b) `valid_ident` is total, deterministic, position-independent, and idempotent, and its reserved-word and illegal-char output spaces are provably **disjoint** (so the combined rule cannot alias where each branch alone would not — §4/§8); (c) the residual non-injectivity (§8) is the *only* one, is `Declared`-honest, and is caught by the never-silent build-time self-collision GAP rather than a silent overwrite; (d) the exported-surface boundary is drawn as §6 states (colliding/invalid names escaped globally; only non-colliding stable exports excluded); and (e) the D4 mangler composes with `valid_ident` (validate the parts *before* joining) so the bracket-regression class cannot recur, and the mangler's own injectivity claim is restated at the honest `Declared`-plus-GAP basis. |

---

## §1 The problem, precisely — one problem, two live facets

Read against the lexer (`crates/mycelium-l1/src/lexer.rs:709-714`) and the reserved snapshot
(`crates/mycelium-transpile/src/reserved.rs`), a **legal Mycelium identifier** is exactly a token that
(i) matches `^[A-Za-z_][A-Za-z0-9_]*$` — `is_ident_start` = `is_ascii_alphabetic() || '_'`,
`is_ident_continue` = `is_ascii_alphanumeric() || '_'` — **and** (ii) is not a member of `RESERVED`
(the lexer tokenizes a reserved word as a keyword, never an `Ident`). Anything else, placed in an
identifier position in the emitted `.myc`, is a **hard parse failure** — not a clean gap.

The transpiler produces Rust names that violate (i) or (ii) from several sources, handled today by
**disconnected** mechanisms with different fates:

- **Facet 1 — reserved-word collision (violates ii).** `pub enum Strength { Exact, … }`
  (`crates/mycelium-l1/src/ast.rs:695` — the guarantee-strength enum; `mycelium-transpile` has no
  `ast.rs`) makes `Exact` a variant; `Strength::Exact` is
  referenced ×34 **cross-file**; `::Binary` ×226 cross-file, all `pub`; plus fn/closure params,
  inherent-impl method names, struct/tuple constructor names, and match patterns. `Exact`, `Binary`,
  `nodule`, `default`, … are all in `RESERVED`. Fate today: a whole-item `Category::ReservedWord` gap,
  or (for nodule-path segments only) a `word→word_kw` escape. DN-139 proposes generalizing the escape
  to program identifiers — **globally**, at declaration and every reference — grounded in the
  `reserved.rs:159-166` injectivity property. *(That global-rule reasoning is settled input; this note
  does not re-derive it — it subsumes it.)*
- **Facet 2 — illegal-character shape (violates i).** `mangled_inherent_fn_name(self_ty_text,
  method)` = `format!("{self_ty_text}__{method}")` (`emit.rs:4302-4304`). When `self_ty_text` embeds
  generic brackets — `DeclaredTime[T]` — the join yields `DeclaredTime[T]__new`. The `[` and `]` are
  **not** `is_ident_continue` characters, so this is not even a gap: it is un-parseable text emitted
  into the `.myc`, a **hard `myc check` parse failure** that regressed std-time `checked_fraction`
  6.34%→5.35%. The same class covers any other illegal char that can leak from rendered type text into
  an identifier position: `::` (path separator), `<` `>` (if any path renders angle brackets), `,`
  (multi-arg generics), whitespace, `&` `*` `'` (ref/ptr/lifetime sigils).

**These are one problem.** Both facets are: *a Rust identifier-position name is not a legal Mycelium
identifier; emit a legal one deterministically, injectively, at the declaration and every reference,
across all files and both pattern and value positions, so cross-file consistency holds by
construction.* The value of a **unified** contract over two point-fixes is that the injectivity
argument, the cross-file-consistency argument, the transparency (reification) argument, and the
exported-surface boundary are stated **once**, over the whole illegal-identifier domain — and the
combined rule is checked for the aliasing that neither facet exhibits alone (§4, §8).

## §2 The reject set — verified against the grammar, not assumed

The Definition of Done requires the *complete* set of Mycelium-illegal identifier shapes to be checked,
not assumed. Verified against `crates/mycelium-l1/src/lexer.rs` (`@dev 2cd9b773`):

| Shape | Legal in a Mycelium identifier? | Basis |
|---|---|---|
| `[A-Za-z]` | start + continue | `is_ident_start`/`is_ident_continue` (lexer:709-714) |
| `[0-9]` | continue only (never start) | `is_ident_continue` = `is_ascii_alphanumeric()` |
| `_` | start + continue | explicit `\|\| c == '_'` in both (lexer:710,714) |
| Any non-ASCII (`≥U+0080`) — including a **legal-Rust** non-ASCII identifier (`struct Δ;`, `fn café()`; Rust stabilized non-ASCII idents in 1.53, and MSRV is 1.96.1, so these are in-domain) | **rejected by Mycelium** — but *not* GAP'd: escaped by branch 3 (`Δ`→`_u394_`, §4) | both Mycelium predicates gate on `is_ascii_*` (lexer:709-714), so Mycelium identifiers are ASCII-only; the *source* Rust identifier is not necessarily ASCII, so branch 3 must (and does) handle a non-ASCII code point, not just ASCII punctuation |
| `[` `]` `<` `>` `(` `)` `{` `}` | **rejected** | not alphanumeric/underscore — the bracket-regression chars |
| `:` (`::`), `.`, `,`, `;` | **rejected** | punctuation, own token kinds |
| ` ` (space), tab, newline | **rejected** | whitespace, splits tokens |
| `&` `*` `'` `#` `@` `$` `?` `!` `+` `-` `/` `=` | **rejected** | operator/sigil chars, own token kinds |
| A reserved word of legal shape (`Exact`, `Binary`, `default`, …) | **rejected as an `Ident`** | `RESERVED` — lexed as a keyword (`reserved.rs:37-112`) |

Two grounding facts the contract leans on, both verified:

1. **Keyword rejection is context-free.** `mycelium-l1`'s `token::keyword` is a pure function of the
   token text — no Mycelium keyword is *contextual* (legal as an identifier in some positions and not
   others). *Consequence:* a reserved word is illegal in **every** identifier position — type, value,
   pattern — uniformly. This is what makes a **position-independent** `valid_ident` sound; it directly
   answers the "valid in one position but not another" adversarial case (§8): that case does not arise
   for the base identifier grammar. *(`Declared` — read of the lexer/`token.rs` snapshot; a build-time
   assertion belongs in the DoD.)*
2. **Mycelium identifiers are ASCII-only; Rust source identifiers are *not*.** Mycelium's lexer gates
   both predicates on `is_ascii_*` (lexer:709-714), so a Mycelium identifier is ASCII — this is a
   verified fact about the *target*. It is **not** true of the general *source* domain: Rust has had
   stable non-ASCII identifiers since 1.53 (MSRV 1.96.1 admits them), so `struct Δ;` / `fn café()` are
   legal Rust that `valid_ident` must map to a legal *ASCII* Mycelium identifier. The **ASCII-output**
   restriction is therefore a *verified target constraint*; the earlier draft's "both languages are
   ASCII" was **overstated** (a VR-5 correction — the source side is a `Declared`, corpus-scoped
   observation at most: the current `mycelium-*` corpus happens to be ASCII-only, but the contract must
   not *rely* on that). *Consequence:* because the ASCII output alphabet has no character guaranteed
   absent from every raw name, a *provably-globally-injective* scheme cannot use an out-of-alphabet
   sentinel; it must reuse `_`/alphanumerics, which is the root of the one documented char-escape
   residual (§8②) — exactly the constraint `sanitize_nodule_path` already accepted, same honest
   resolution (documented residual + never-silent GAP). *(The type↔method **boundary** is a separate
   matter — it is made injective by construction without any sentinel, via length-prefix; §7/§8①.)*

## §3 Alternatives, objective function, ranked recommendation

### Objective function (criteria table)

| Criterion | Weight | Why it matters here |
|---|---|---|
| **G2 (never-silent)** | critical (veto) | No silent substitution or silent collision; every rewrite reified + EXPLAIN-able, every unpreventable collision a build-time GAP |
| **Cross-file consistency by construction** | critical | Declaration and every reference — across independently-emitted files, in pattern and value position — must agree with *zero* coordination (the transpiler emits file-at-a-time) |
| **Injectivity (no two distinct Rust names collapse)** | critical | A collapse silently merges two symbols — a correctness defect, not cosmetics (esp. distinct generic instantiations sharing a method name — the D4 flat-namespace hazard) |
| **Covers the whole reject set** | high | Must handle reserved words **and** illegal chars in one rule — the unification is the point |
| **DRY / KC-3 / KISS** | high | Reuse the landed `_kw` suffix + injectivity property; collapse `guard_ident`/`sanitize_nodule_path` into special cases, not a third scheme |
| **Exported-surface honesty** | high | A colliding export has no usable verbatim form → must escape; a non-colliding stable export must be preserved exactly |
| **Coverage leverage** | medium | 53 ReservedWord gaps (rank 7) + the D4 bracket-regression class + all future coincidental collisions |

### Alt A — one global, deterministic, injective `valid_ident` over the whole reject set (RECOMMENDED)

A single pure function (§4) applied at every declaration and every reference, position-independent,
covering reserved words *and* illegal characters, with the two existing escapers subsumed as special
cases and a build-time self-collision GAP as the never-silent backstop.

- **Cross-file consistency:** by construction. `valid_ident` is a pure function of the token alone, so
  every file that emits a declaration or reference of name `N` independently computes the identical
  `valid_ident(N)` — the `reserved.rs:159-166` argument, generalized from reserved-word segments to the
  whole reject set. No coordination.
- **Injectivity:** holds on already-valid and reserved-word inputs (constant suffix); holds on
  illegal-char inputs (per-char hex is injective on chars); the two output spaces are **disjoint**
  (§4), so the *combined* rule introduces no new aliasing; and the type↔method **boundary** is injective
  by construction via length-prefix (§7), not merely GAP-caught. The **one** residual (a raw valid name
  that literally contains the escape sentinel `_u{HEX}_`) is `Declared`, vanishingly unlikely, and
  build-time-GAP-caught (§8②) — the identical honesty posture as `sanitize_nodule_path`'s documented
  `fuse_kw.rs` residual.
- **G2:** the escaped name is visible in the emitted text and mechanically invertible (EXPLAIN);
  every rewrite carries a `// Declared: renamed …` line; every unpreventable collision is a build-time
  GAP naming both originals — never a silent overwrite.
- **DRY/KISS:** one function; `guard_ident` and `sanitize_nodule_path` become call-throughs; `_kw` is
  reused, not re-invented.
- **Verdict:** **Rank 1.**

### Alt B — two separate point-fixes (DN-139 for reserved words; a bespoke bracket-strip in the mangler)

Keep DN-139's reserved-word escape and, separately, strip/replace brackets inside
`mangled_inherent_fn_name`.

- **Covers the reject set:** partially and un-unifiedly. Two conventions, two injectivity arguments,
  two transparency stories — and, critically, **no single place that checks the two rules do not alias**
  (a name escaped by the bracket-fix and a name escaped by the reserved-fix could, without a shared
  contract, collide, and nothing would notice). A naive bracket-strip (`[`,`]`→`_` or deletion) is
  **not injective**: `Foo[T]` and `Foo_T_` and `Foo[U]`→… collapse, silently merging distinct generic
  instantiations — reintroducing the exact flat-namespace hazard D4 exists to prevent.
- **DRY:** fails — a second escape convention.
- **Verdict:** **Rank 2** — closes both facets but forfeits the unified injectivity guarantee and risks
  a silent generic-instantiation collapse; strictly dominated by Alt A.

### Alt C — a reserved-namespace prefix (escape every colliding/illegal name with a sigil prefix)

E.g. prefix escaped names with a marker (`myc_`, or a leading `_`).

- Mycelium has **no** raw-identifier syntax (verified: no `r#`/`raw_ident` in `lexer.rs`/`token.rs`),
  so a sigil must be built from the legal alphabet. A leading `_` is not injective against a real
  `_Exact`; a `myc_` prefix on *every* name is ugly, changes stable exports needlessly, and is still
  just a worse suffix scheme. A prefix-only-on-collision is behaviorally the suffix scheme with poorer
  ergonomics and no reuse of the landed `_kw` convention.
- **Verdict:** **Rejected** — no advantage over the `_kw` suffix; loses DRY.

### Alt D — relax the Mycelium lexer/grammar to admit the raw names (a raw-identifier escape, e.g. `r#nodule`, or bracket tolerance)

- **Tail wags dog / KC-3 violation.** Changing the *language* to accommodate the *transpiler* enlarges
  the small trusted kernel for a tooling convenience — the wrong direction. A raw-identifier feature is
  a genuine language-design decision (its own ADR/RFC), unevidenced as independently wanted.
- **Cannot be the unified answer even if adopted.** A raw-ident escape (`r#nodule`) could rescue Facet 1
  (reserved words) but **not** Facet 2 — brackets inside an identifier are fundamentally ambiguous with
  generic-argument syntax; no lexer relaxation can admit `DeclaredTime[T]__new` as one identifier
  without breaking `[]` as an index/generic delimiter. So Alt D is, at best, a partial complement, not
  a replacement.
- **Verdict:** **Rejected** for this contract (out of scope; disproportionate; incomplete). *Noted as a
  possible future language-level ergonomics option for Facet 1 only — see the standing "LANGUAGE closes
  expressibility gaps" direction — but the transpiler must not block on it, and it never covers Facet 2.*

### Ranked recommendation

**Alt A ≻ Alt B ≻ Alt C ≡ Alt D (rejected).** Adopt **Alt A**: one deterministic, injective,
position-independent `valid_ident` over the whole reject set, subsuming DN-139 and
`sanitize_nodule_path`, composed into the D4 mangler (§7), with the exported-surface boundary of §6 and
the never-silent build-time self-collision GAP of §8. Alt B is the correct fallback if the gate wants
the two facets landed independently, but it forfeits the single-injectivity guarantee this note exists
to provide.

## §4 The `valid_ident` function — signature, contract, mapping

**Proposed signature** (illustrative; the build issue fixes the exact types):

```
/// Map an arbitrary identifier-position string to a legal Mycelium identifier.
/// Total, deterministic, position-independent, idempotent. Never fails to produce a
/// legal identifier; a residual cross-name collision is caught by the caller's
/// per-unit self-collision check (a build-time GAP), never resolved by guessing.
fn valid_ident(raw: &str) -> ValidIdent

struct ValidIdent {
    text: String,                 // guaranteed to match ^[A-Za-z_][A-Za-z0-9_]*$ and !is_reserved
    rewrite: Option<Rewrite>,     // None iff text == raw (identity); else the reified explanation
}
struct Rewrite { original: String, kind: RewriteKind /* Reserved | IllegalChars | Both */, note: String }
```

**Contract:**

- **Totality.** For every input, `text` is a legal Mycelium identifier (matches the grammar of §2,
  and is not reserved). *(The empty string and a leading-digit input — neither of which a Rust
  identifier can be — are handled defensively by prefixing `_`; stated for totality, not because the
  corpus needs it.)*
- **Identity on already-valid, non-reserved names.** If `raw` matches the grammar and `!is_reserved`,
  `valid_ident(raw).text == raw` and `rewrite == None`. *(This is what preserves stable exports, §6.)*
- **Determinism + position-independence.** `text` is a pure function of `raw` — no dependence on
  position (type/value/pattern), file, or surrounding scope. *(The cross-file-consistency load-bearer,
  §5.)*
- **Idempotence.** `valid_ident(valid_ident(raw).text).text == valid_ident(raw).text`, because the
  output is always a legal non-reserved identifier and so hits the identity branch. *(Prevents
  double-escaping under composition, §7.)*

**The mapping** (three disjoint branches):

1. **Already valid, non-reserved** → identity.
2. **Valid shape, reserved** (`Exact`, `Binary`, `default`, …) → `raw + RESERVED_SEGMENT_SUFFIX`
   (`Exact → Exact_kw`). *This is exactly DN-139's rule and `sanitize_nodule_path`'s per-segment rule —
   subsumed, one suffix.* Injective on reserved words (constant suffix); no `RESERVED` entry ends in
   `_kw` (already drift-tested at `reserved.rs:162-163`), so the output is never itself reserved.
3. **Contains ≥1 illegal char** (any char outside `[A-Za-z0-9_]` — ASCII punctuation **or** a
   non-ASCII code point) → replace **each** such char `c` with the delimited token `_u{HEX}_`, where
   `{HEX}` is `c`'s **Unicode scalar value** in uppercase hex, **variable width** (not fixed to two
   digits — a fixed 2-digit ASCII field could not represent `Δ` = U+0394): `[`→`_u5B_`, `]`→`_u5D_`,
   `:`→`_u3A_`, `<`→`_u3C_`, `,`→`_u2C_`, ` `→`_u20_`, `&`→`_u26_`, `'`→`_u27_`, and the non-ASCII
   cases `Δ`(U+0394)→`_u394_`, `é`(U+00E9)→`_uE9_`. So `DeclaredTime[T]` → `DeclaredTime_u5B_T_u5D_`,
   and distinct instantiations stay distinct: `DeclaredTime[U]` → `DeclaredTime_u5B_U_u5D_`; a legal
   non-ASCII Rust type `Δ` → `_u394_`. If the escaped result is *itself* reserved (it never is in
   practice — it contains a `_u` token and no reserved word does), apply step 2 as well (`kind = Both`).
   *This restores **Totality** for the non-ASCII case the earlier draft silently mis-covered (VR-5): a
   legal-Rust `struct Δ;` no longer has an under-specified escape — it maps to `_u394_`, deterministic
   and injective, never a silent gap.*

**Why the variable-width delimited hex, not a `_`-collapse or a fixed field.** A collapse of illegal
runs to `_` is not injective and would silently merge distinct generic instantiations (the D4 hazard).
A *fixed* 2-hex-digit field cannot even represent a non-ASCII scalar (§3 patch). Per-code-point
variable-width hex is injective **on the escaped characters** and total over all of Unicode. The
delimiters (`_u…_`, with the trailing `_` closing the hex run) keep adjacent escapes from running
together and make each escape uniquely decodable (`[]`→`_u5B__u5D_`, unambiguously two escapes; the
closing `_` ends the hex run, so `_u5B_` cannot be misread as `_u5B5…`).

**Combined-rule non-aliasing (the unification's key new argument).** The reserved-branch output space
is `{ w + "_kw" : w ∈ RESERVED }` — every element contains `_kw` and no `_u{HEX}_` token (no reserved
word contains `_`). The illegal-branch output space is `{ strings containing ≥1 _u{HEX}_ token }`.
These two sets are **disjoint**: a reserved-escape output can never equal an illegal-escape output,
because the former contains no `_u{HEX}_` token and the latter always does. *Therefore the combined
rule aliases exactly where the reserved-only rule would, plus exactly where the illegal-only rule
would — and never across the two.* This is the property Alt B cannot state (it has no single output
space to reason over) and the reason a **unified** contract is safer than two point-fixes. *(`Declared`
— an argument over the two output-space shapes; a build assertion/property test belongs in the DoD.)*

## §5 Application discipline — cross-file consistency **by construction**

The load-bearing precedent is `sanitize_nodule_path`'s soundness (`reserved.rs:159-166`): because the
escape is a **deterministic function of the token alone**, every file that emits the token independently
reproduces the same rewrite, so the global result is consistent **with no cross-file coordination —
the rule is the coordination.**

Generalized to `valid_ident`: the emitter applies `valid_ident` at **every** identifier position it
controls — the **declaration** (`enum Strength { Exact }` → `enum Strength { Exact_kw }`), and **every
reference**, in **both** positions: value/expression (`Strength::Exact` → `Strength::Exact_kw`, the
×34 cross-file case) and **pattern** (`match s { Strength::Exact => … }` → `… Strength::Exact_kw =>
…`). Because `valid_ident` is position-independent (§2 fact 1 — keyword rejection is context-free, so
the escape must *not* vary by position or a pattern and its value form would desync), the declaration
and every reference across all 226 `::Binary` sites and all files land the identical escaped spelling
**without** the transpiler ever needing to see two files at once. This is precisely the unsoundness the
DN-139 gate first flagged in the original same-file draft and the reason the rule must be global; DN-140
inherits the global discipline and extends it to the illegal-char branch (which has the *same* purity,
so the same argument applies verbatim).

## §6 The exported-surface boundary, redrawn honestly

DN-139's earlier objection (and the general worry): "a `pub enum` variant is an external contract — a
transpiler pass must not rename it." The redraw:

- **A colliding or illegal name has no usable verbatim Mycelium form at all.** `pub enum Strength {
  Exact }` cannot emit `Exact` — the lexer tokenizes it as the `Exact` keyword; `DeclaredTime[T]__new`
  cannot emit those brackets. The raw name **does not parse** in *any* position, `pub` or not.
  *Therefore there is no stable exported contract to preserve* — external stability is **already
  impossible** for that name. Escaping it globally is strictly better than gapping it (a gap exports
  *nothing*), and determinism is what turns the escape into a *real* contract: any external consumer
  (another nodule, a hand-written caller) computes the identical `valid_ident(Exact) = Exact_kw` and
  references it consistently. The escaped spelling **is** the exported surface.
- **The exclusion applies only to names that are already legal, non-colliding identifiers** and merely
  happen to be `pub`. Those hit branch 1 (identity) — their exported spelling is preserved **exactly**,
  never renamed. This is the honest line: *escape the unusable; preserve the usable.*

So the boundary is not "public vs private" (DN-139's earlier cut, which would leave the 226 cross-file
`::Binary` references and the ×34 `Strength::Exact` permanently gapped) but **"colliding/illegal vs
stable-and-legal"**. The `pub`-ness of a colliding name is irrelevant — it was never a usable contract.

## §7 Composition — the D4 mangler and the nodule-path sanitizer

**`mangled_inherent_fn_name` (the D4 site, `emit.rs:4302-4304`).** Two defects compose here: (i) the
bracket regression — the mangler joins `{self_ty_text}__{method}` from **raw** type text, so an illegal
char survives into the identifier; and (ii) the `__` boundary is not injective — `__` is *legal inside*
an identifier (`fn bar__baz`, `impl Foo__bar`), so the type↔method split is ambiguous (§8①). The
current landed mangle is simultaneously the internal resolution **key** (`record_local_mangled_assoc_fn`
@`emit.rs:324`) **and** the final emitted `fn` identifier that `myc check` parses (`emit/calls/
qualified_assoc.rs`) — so the final form must be a legal Mycelium identifier; a literal `:` (or `__`
with an unambiguous meaning) cannot survive in it.

**Two-part fix.** First, validate the parts: `vt = valid_ident(self_ty_text).text`, `vm =
valid_ident(method).text` — each is now a legal identifier with a **non-digit first char** (grammar,
§2), so no illegal char can survive (kills defect i). Second — the maintainer's 2026-07-13 IR-delimiter
insight — make the boundary injective **by construction** with a **length-prefix encoding** (the
Itanium/Rust-`v0`-mangling shape), because the mangle is an intermediate representation and this is the
right layer to fix the boundary:

```
// mangle = "_" ++ dec(len(vt)) ++ vt ++ dec(len(vm)) ++ vm     (dec = decimal char count)
//   impl Foo { fn new }              -> _3Foo3new
//   impl Foo { fn bar__baz }         -> _3Foo8bar__baz
//   impl Foo__bar { fn baz }         -> _8Foo__bar3baz          (distinct from the above — boundary is unambiguous)
//   impl DeclaredTime[T] { fn new }  -> _23DeclaredTime_u5B_T_u5D_3new
```

**Injectivity by construction (the checked basis for the VR-5 upgrade).** The map `(vt, vm) ↦ "_" ‖
dec(|vt|) ‖ vt ‖ dec(|vm|) ‖ vm`, where `vt`/`vm` are legal identifiers (first char non-digit), is
injective: given the output, strip the leading `_`, read the maximal decimal-digit run = `dec(|vt|)`
(unambiguous because `vt`'s first char is non-digit, so the run ends exactly at `vt`), take the next
`|vt|` chars = `vt`, then repeat for `dec(|vm|)`/`vm`. The decode is total and unique ⇒ the encode is
injective. This is the classic length-prefixed (prefix-free) argument. So the **type↔method boundary
collision of §8① is eliminated**, not GAP-caught — `_3Foo8bar__baz ≠ _8Foo__bar3baz`. The result is a
valid identifier (leading `_`, all continue-chars) and never reserved (starts `_<digit>`). **VR-5
upgrade, with basis:** the boundary guarantee legitimately rises from the earlier draft's
"`Declared`-plus-GAP" to **injective by construction** (the argument above is the checked basis; a
build property test — `enc` round-trips / no two distinct `(vt,vm)` collide — pins it, and until built
the mechanism tag stays `Declared` per house rule #1). This also **corrects** the landed mangler's
"collision-free by construction (two distinct items can never share both type and method name)" claim:
that was about the *pairs* being distinct, but the naive `__`-join *lost the boundary* — length-prefix
is what actually makes the claim true.

*Alternatives considered for the boundary (picked length-prefix on merit):* **(a′) a `:`-delimited
key + escape** — use the identifier-illegal `:` as the internal key delimiter (unambiguous, injective
key: `Foo:bar__baz` vs `Foo__bar:baz` split cleanly), then derive the emitted identifier by escaping
`:`→`_u3A_` (§4). This *works* (`Foo_u3A_bar__baz ≠ Foo__bar_u3A_baz`), but the emitted boundary now
rides the `_u3A_` sentinel, re-introducing the §8② char-escape residual *on the boundary itself* —
strictly weaker than length-prefix, which needs **no** sentinel for the boundary. **(b) structural IR**
— carry `(type, method)` as structure through the IR and synthesize the final identifier only at
codegen (boundary structural, not textual). Cleanest in principle and fully injective, but it requires
the downstream consumer to accept the structure *before* final-identifier emission — a materially larger
change to the key/emit pipeline (the mangle is currently *both* key and identifier), i.e. more blast
radius than the evidenced need (KISS/YAGNI). **Chosen: (a) length-prefix** — it makes the boundary
injective by construction with a purely local change to `mangled_inherent_fn_name` (+ its key recorder),
needs no illegal-char escaping for the boundary, and stays a valid identifier. Readability cost (a
length-prefixed name is less greppable than `Foo__new`) is mitigated by the emitted `// Declared:
renamed …` comment naming the original `impl Type { fn method }`, and the mangle stays mechanically
invertible.

**Today's landed backstop (so the improvement is framed honestly, not as fixing a silent bug).** Even
without this fix, the naive `__` collision is **not** a silent misbind: the transpiler emits two
identical top-level `fn` decls, and the downstream checker's `register_nodule_decls`
(`crates/mycelium-l1/src/checkty.rs:2514`) rejects the duplicate at `checkty.rs:2564` (`"duplicate
function"`) — a **hard `myc check` failure**, the same class as the bracket regression, never a silent
wrong binding. So length-prefix is a *genuine strengthening* (it eliminates the collision so both decls
are distinct and both check), and the residual build-time per-unit GAP (§8②/⑤) is an *improvement in
diagnosis* (a precise transpile-time gap naming both originals) over a downstream duplicate-fn hard
fail — not the repair of a silent defect.

**`sanitize_nodule_path` (`reserved.rs:187`).** This is the **per-segment specialization** of
`valid_ident`'s branch 2: split on `.`, apply the reserved-word escape to each segment, rejoin. It
operates on a **disjoint domain** (dotted file-layout paths) from `valid_ident`'s callers (bare program
symbols), so the two never touch the same string and **cannot double-escape**. Even if they did,
idempotence (§4) makes double application a fixpoint (`valid_ident(Exact_kw) = Exact_kw`, since
`Exact_kw` is legal and non-reserved). Recommendation: on ratification, redefine `sanitize_nodule_path`
as `join(".", segments.map(seg => valid_ident_reserved_branch(seg)))` so there is **one** injectivity
property in the codebase, not two — but this is a DRY refactor, not a behavior change (append-only:
`sanitize_nodule_path`'s existing doc-comment is not rewritten; a new section cites this note).

## §8 Adversarial stress-test (house rule #4 / VR-5)

**① The `__` boundary collision — the one that most nearly broke the *naive* scheme, now RESOLVED by
construction (§7).** Construct two **distinct** Rust inherent-impl items whose *naive* `Type__method`
mangles collide as strings:

- `impl Foo { fn bar__baz(...) }` — a method literally named `bar__baz` is **legal Rust** — naive
  mangle `Foo__bar__baz`.
- `impl Foo__bar { fn baz(...) }` — a type literally named `Foo__bar` — naive mangle `Foo__bar__baz`.

Distinct `(type, method)` pairs → **identical** naive string, because `__` is legal *inside* an
identifier so the `__`-join loses the boundary. This falsifies the landed mangler's "collision-free by
construction" claim (which reasoned about the *pairs*, not the joined *string*). **Resolution — the
maintainer's IR-delimiter insight (§7):** the mangle is an intermediate representation, so the boundary
is carried by a **length-prefix** (`_3Foo8bar__baz` vs `_8Foo__bar3baz`), which is **injective by
construction** (the decode argument in §7). The collision is therefore **eliminated**, not GAP-caught —
this class cannot occur. *(Framing, VR-5: even the naive scheme was never a silent misbind — the
downstream `register_nodule_decls` duplicate-fn refusal, `checkty.rs:2564`, hard-fails it at `myc
check`; length-prefix removes the collision so both decls are distinct, §7.)* This was the case that
most nearly broke the *original* recommendation; the maintainer input converts it from an honest
`Declared`-residual into a by-construction elimination — a real strengthening.

**② The one residual that remains — the char-escape sentinel self-collision (now the honest headline).**
*Could a reserved-word escape and an illegal-char escape collide?* No — §4's disjoint-output-space
argument: reserved-escape outputs contain `_kw` and no `_u{HEX}_` token; illegal-escape outputs always
contain a `_u{HEX}_` token. Disjoint. *The residual that does remain:* a raw name that is **already a
legal identifier** and literally contains the sentinel substring `_u5B_` — e.g. a real Rust name
`Foo_u5B_` — hits branch 1 (identity) → `Foo_u5B_`, while `Foo[` (illegal) → `Foo_u5B_`. Collision.
Likewise a source name literally spelled `_u394_` collides with the non-ASCII escape of `Δ`. This is
the **single** residual class, structurally identical to `sanitize_nodule_path`'s documented `fuse_kw.rs`
residual: vanishingly unlikely (no real Rust identifier is spelled `_u5B_`/`_u394_`), `Declared` (not
`Proven`-globally-injective — §2 fact 2: the ASCII *output* alphabet forbids an out-of-alphabet
sentinel that would make it provable), and — critically — **caught by the build-time per-unit
self-collision GAP**, so it is diagnosable, never silent. A fully provable scheme exists (route *every*
name through the encoder, escaping the sentinel introducer too), but it turns `my_func` into
`my_u5F_func` — unreadable, un-grep-able, a KISS/EXPLAIN regression; correctly rejected in favor of the
documented residual, mirroring `sanitize_nodule_path`'s explicit same choice (`reserved.rs:168-177`).
The *same* residual class also covers the mangled name vs. a user fn literally named like `_3Foo3new`
(§7) — one class, one GAP backstop.

**③ A name valid in one position but not another.** Checked against §2 fact 1: Mycelium keyword
rejection is context-free — a reserved word is illegal in **every** identifier position, and a legal
identifier is legal in every position. So the base grammar has no position-sensitive identifier.
`valid_ident` is therefore soundly position-independent. The one *apparent* counter — a unit/tuple
struct where the **constructor name equals the type name** (`struct Exact;` used as both a type and a
value/pattern constructor) — is *not* a counterexample: the same token `Exact` is escaped to the same
`Exact_kw` in both its type occurrence and its value/pattern occurrence, so the type-name ==
constructor-name identity is **preserved**, not broken (this is DN-139's already-resolved
constructor-name==type-name finding; it holds here because `valid_ident` is position-independent by
construction).

**④ Interaction with `sanitize_nodule_path` — double-escape or cross-collision?** Covered in §7: the
two operate on disjoint domains (dotted paths vs bare symbols) and never touch the same string; and
even under (hypothetical) re-application, idempotence makes escaping a fixpoint. No double-escape. No
cross-collision (a nodule path is a header, not a program-symbol reference, so the two output spaces
are never compared in one namespace).

**⑤ The `_kw`/hex-escape self-collision (a name whose *escaped* form pre-exists as a distinct real
name).** E.g. a fn already has a `nodule` parameter **and** a `nodule_kw` local; escaping `nodule` →
`nodule_kw` would shadow the real `nodule_kw`. Resolution (inherited from DN-139, generalized):
**GAP** — never silently shadow, never probe a second suffix (`_kw2`), because guessing a collision-free
spelling is exactly the "plausible but wrong" pattern G2 forbids. The build-time per-unit
self-collision check is the single mechanism that catches the residual cases ② and ⑤ (and the
mangle-vs-user-name residual, §7) uniformly — one never-silent backstop for every residual the
deterministic map cannot prevent. *(Case ① — the type↔method boundary — is not in this set: it is
eliminated by construction via the length-prefix encoding, §7, so it needs no GAP.)*

## §9 Definition of Done

**Accepted** (by the maintainer / strict DN-review gate) requires confirmation that:

1. **The reject set (§2) is complete** — checked against `mycelium-l1/src/lexer.rs`
   (`is_ident_start`/`is_ident_continue`) and the `RESERVED` snapshot, **including the non-ASCII case**
   (legal-Rust `struct Δ;`/`fn café()` route to branch 3's `_u{HEX}_` Unicode-scalar escape, not a
   gap), with the two grounding facts verified, not assumed: keyword rejection is context-free; and the
   **ASCII restriction is on the *output* (Mycelium) only** — the *source* Rust domain is not ASCII
   (VR-5 correction to the earlier "both languages are ASCII" overstatement).
2. **`valid_ident` is total, deterministic, position-independent, and idempotent**, and its
   reserved-branch and illegal-branch **output spaces are disjoint** (§4) — so the combined rule
   introduces no aliasing beyond each branch's own.
3. **The type↔method boundary is injective by construction** (§7 length-prefix; the decode argument is
   the checked basis for the VR-5 upgrade — property-tested at build), so the §8① `__`-boundary
   collision is **eliminated**, not GAP-caught. **The char-escape sentinel self-collision (§8②) is then
   the *only* residual**, is `Declared`-honest (not claimed `Proven`-globally-injective — §2 fact 2),
   and is caught by the **never-silent build-time per-unit self-collision GAP** (which also covers the
   escaped-form self-collision §8⑤ and the mangle-vs-user-name residual §7).
4. **The exported-surface boundary is drawn as §6** — colliding/illegal names escaped globally
   (declaration + every reference, all files, both positions); only **non-colliding, legal** stable
   exports excluded (identity branch).
5. **The D4 mangler composes with `valid_ident` and length-prefixes the boundary** (§7): validate the
   parts (kills the bracket-regression class — no illegal char survives) **and** encode the type↔method
   boundary with the length-prefix scheme (`_<len>vt<len>vm`), making the boundary injective by
   construction (chosen over the `:`-key+escape and structural-IR alternatives, §7). The landed
   mangler's overstated "collision-free by construction" claim is thereby *made true* (it was false for
   the naive `__` join), and the change is framed honestly as a strengthening over today's downstream
   duplicate-fn hard-fail (`checkty.rs:2564`), not the repair of a silent bug.
6. **DN-139 is subsumed** — its `word→word_kw` is the reserved-word branch of this contract; on
   ratification DN-139 → **Superseded by DN-140** (append-only, house rule #3), and
   `sanitize_nodule_path` is (recommended) refactored to the per-segment specialization (DRY, no
   behavior change).

**Then, the build DoD** (a FLAGGED build issue — this note builds nothing):

- A shared `valid_ident` helper (subsuming `guard_ident`/`sanitize_nodule_path`) applied at every
  identifier declaration + reference site the emitter controls, in both pattern and value positions.
- Every non-identity rewrite emits a reified `// Declared: renamed <original> -> <escaped> (<why>,
  DN-140)` comment, so the substitution is `EXPLAIN`-visible in the emitted text (never only in a
  tool-internal log).
- The mangler (`emit.rs:4302-4304` + its key recorder `record_local_mangled_assoc_fn` @`324`)
  `valid_ident`-validates its parts **and** length-prefix-encodes the type↔method boundary
  (`_<len>vt<len>vm`, §7) — both the internal resolution key and the emitted identifier derive from the
  same injective encoding.
- A **per-unit self-collision check**: if two distinct originals map to the same emitted identifier in
  one nodule, **GAP both** (precise reason, both originals named), never overwrite — the backstop for
  the §8② char-escape residual (the §8① boundary collision is eliminated by construction, no GAP
  needed).
- Property tests: **totality over Unicode** (every input — ASCII, illegal-punctuation, and a non-ASCII
  code point like `Δ`/`é` — yields output matching the §2 grammar and `!is_reserved`); idempotence;
  branch-output-space disjointness; **length-prefix boundary injectivity** (no two distinct `(vt, vm)`
  pairs encode equal, and `enc` round-trips via the decode of §7 — the checked basis for the boundary
  guarantee); and char-escape injectivity **on the evidenced corpus** (with the §8② residual
  documented, not asserted-away).
- **No guarantee upgraded past `Declared`** until a differential (a hand round-trip per position class:
  does the escaped `.myc` behave identically to the Rust source, and does `myc check` accept it)
  witnesses byte/behavior faithfulness (VR-5). The headline number to move: the D4 bracket-regression
  reversal (std-time `checked_fraction` back over 6.34%) plus the 53 ReservedWord gaps closed.

## §10 FLAGs (shared files — NOT edited here; for the integrating parent)

Append-only, dated rows for the integrating parent to apply (this note edits **only** itself):

- **`docs/Doc-Index.md`** — add a DN-140 row (Notes table, after DN-139): *"Unified Valid-Identifier
  Emission Contract — one deterministic, injective, position-independent `valid_ident` subsuming the
  reserved-word escape (DN-139), the nodule-path-segment escape (`sanitize_nodule_path`), and the
  unhandled illegal-character class (generic brackets/`::`/etc., the D4 `mangled_inherent_fn_name`
  regression). `Empirical` read against `dev@2cd9b773` + DN-139@`ee33e4dc`; `Declared` mechanism until
  built + differential-witnessed. Draft."* And mark the DN-139 row **Superseded by DN-140** on
  ratification (append-only — DN-139's row text is not rewritten).
- **`CHANGELOG.md`** — under Unreleased/Added: *"DN-140 (Draft) — unified valid-identifier emission
  contract for the Rust→Mycelium transpiler; subsumes DN-139; specs the fix for the D4 generic-bracket
  parse-failure regression."*
- **`tools/github/issues.yaml`** — a FLAGGED build issue for the `valid_ident` implementation
  (subsume `guard_ident`/`sanitize_nodule_path`, compose into `mangled_inherent_fn_name`, per-unit
  self-collision GAP, length-prefix mangle, property tests, differential witness); `doc_refs:
  corpus:DN-140`, `src:crates/mycelium-transpile/src/reserved.rs`,
  `src:crates/mycelium-transpile/src/emit.rs:4302`.
- **DN-139** (`claude/leaf/phase2-next-waves-scoping@ee33e4dc`) — recommend DN-139 → Superseded by
  DN-140 on ratification (its sound global reserved-word rule is preserved as this contract's
  reserved-word branch — nothing is lost, it is generalized). **FLAG only, not applied here:** DN-139
  lives on another branch not reachable from this one, and the supersede is moot until DN-140 itself
  ratifies — so this note neither edits DN-139 nor its status; the integrating parent/maintainer
  applies the supersede at ratification time.

---

## Changelog (this note)

- 2026-07-13 — **Draft** created (`@dev 2cd9b773`; DN-139 read at `ee33e4dc`). Unifies the
  reserved-word collision class and the D4 generic-bracket-in-identifier regression into one
  `valid_ident` contract; verifies the Mycelium reject set against the lexer (§2); recommends **Alt A**
  (one global deterministic injective position-independent function subsuming DN-139 +
  `sanitize_nodule_path`) over two point-fixes (Alt B), a sigil prefix (Alt C), and lexer relaxation
  (Alt D); redraws the exported-surface boundary as colliding/illegal-vs-stable-legal (§6); composes
  into the mangler to kill the bracket-regression class (§7); and — the adversarial headline (§8①) —
  surfaces a pre-existing non-injectivity in the D4 `__` separator, downgrading its "collision-free by
  construction" claim to the honest `Declared`-plus-never-silent-GAP basis. Recommends, does not
  ratify (house rule #3). Shared-file rows FLAGGED (§10), not edited.
- 2026-07-13 — **Re-gate patch** (`@dev 2cd9b773`), addressing the strict DN-review gate's four defects
  plus the maintainer's IR-delimiter design input (Alt A kept; the gate credited the framing,
  §4/§6/§7/§8① and the Alt B/C/D rejections as sound). (1) **Citations corrected against ground truth:**
  `mangled_inherent_fn_name` is `emit.rs:4302-4304` (not 4428 — verified); `enum Strength { Exact }` is
  `crates/mycelium-l1/src/ast.rs:695` (not `mycelium-transpile`, which has no `ast.rs`). (2) **§2 fact 2
  restated (VR-5):** ASCII is a *target*-only constraint — Rust has stable non-ASCII identifiers since
  1.53 (MSRV 1.96.1), so "both languages are ASCII" was overstated. (3) **Non-ASCII totality fixed:**
  branch 3's escape widened from a fixed 2-hex-digit ASCII field (`_x{HH}_`, which could not represent
  `Δ`) to a variable-width Unicode-scalar hex (`_u{HEX}_`); the case is now added to §2, §4, §8②, §9(1).
  (4) **checkty.rs backstop credited:** the naive `__` collision is caught today by
  `register_nodule_decls`'s duplicate-fn refusal (`checkty.rs:2564`) — a downstream hard-fail, not a
  silent misbind. (5) **Maintainer input folded (§7/§8① strengthening):** the type↔method boundary is
  made **injective by construction** via a **length-prefix** mangle (Itanium/`v0` shape) chosen over a
  `:`-key+escape and a structural-IR alternative — so §8① is **eliminated**, not GAP-caught, and the
  boundary guarantee legitimately upgrades from `Declared`-plus-GAP to by-construction (with the decode
  argument as the checked basis; §8② char-escape residual remains the sole GAP-backstopped residual).
  Still Draft; re-gate pending.
- 2026-07-13 — **Accepted** by the strict project DN-review gate (the final of a 3-round adversarial
  re-gate). Round 1 blocked and drove four fixes: the `mangled_inherent_fn_name` citation
  (`emit.rs:4428`→`4302-4304`), the `enum Strength` crate (→`crates/mycelium-l1/src/ast.rs:695`; the
  transpile crate has no `ast.rs`), the §2-fact-2 ASCII overstatement (restated as a *target*-only
  constraint — Rust admits non-ASCII identifiers since 1.53, MSRV 1.96.1), and non-ASCII totality
  (branch 3's fixed 2-hex `_x{HH}_` widened to variable-width Unicode-scalar `_u{HEX}_`, e.g.
  `Δ`→`_u394_`). Round 2 folded the maintainer's IR-delimiter input — the type↔method boundary is made
  injective **by construction** via a length-prefix mangle (§7/§8①), which the gate re-derived and
  attacked without finding a counterexample (the non-digit-first-char premise is forced by
  `valid_ident`'s totality; the sole remaining residual is the char-escape/mangle-vs-user-name class of
  §8②, caught never-silently by the build-time self-collision GAP and today's downstream
  `register_nodule_decls` duplicate-fn refusal, `checkty.rs:2564`) — and flagged a citation
  transposition (`record_local_mangled_assoc_fn` `emit.rs:4324`→`324`, defined at 324, invoked at 4712).
  Round 3 confirmed that fix resolves and ratifies. No mechanism is built by this note; every guarantee
  tag stays `Declared` until the FLAGGED build issue lands its property tests + differential witness
  (VR-5 / house rule #1). DN-139 → Superseded and the §10 shared-file rows remain **FLAGGED**, not
  applied here (the integrating parent applies them on landing).
