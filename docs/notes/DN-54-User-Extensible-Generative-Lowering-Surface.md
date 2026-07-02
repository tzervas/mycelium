# Design Note DN-54 — User-Extensible Generative-Lowering Surface & Its Checker

| Field | Value |
|---|---|
| **Note** | DN-54 |
| **Status** | **Accepted** (2026-06-27, ratified by maintainer, R5 gate) — **implemented (Rust-first, surface + structural checks); KC-3 + IL-grammar + RHS-elaboration pending — pending ratification** (2026-06-28; M-812 in `mycelium-l1`). **What landed:** the `lower Name[params] = <rhs>` definition surface and `derive Name for T` use-site surface are **active** keywords (parse + AST `Item::Lower`/`Item::Derive` + `LowerDecl`/`DeriveDecl`), and the checker performs the **structural** validations (§4.2/§4.6 partial): `lower`-rule **name uniqueness**, **param-name uniqueness**, and `derive` **rule-name resolution** — all **never-silent** (`CheckError`, G2). The rule is registered in `Env::lower_rules`. **What is NOT yet implemented (deferred to M-812-cont; held `Declared`, VR-5):** (1) the **RHS elaboration to L0** — `crate::elab` does not yet read `Env::lower_rules`, so a `derive` currently emits **no L0 term** (an honest never-silent residual, *not* a fabricated accept — pinned by `tests/checkty.rs::lower_derive_items_add_no_l0_to_an_unrelated_entry`); (2) the **§4.1 IL-grammar RHS type-check** (infer-type the RHS, reject `wild`/mutation/ill-formed terms); (3) the **§6 KC-3 kernel-growth guard** (the elaborated RHS must lower to existing L0 nodes only); (4) **§4.2 cross-rule acyclicity** (only same-nodule name-uniqueness is enforced today); (5) the **§7 verification discipline** (differential / hygiene / round-trip harness). Guards (2)/(3) are only *meaningful* once (1) lands — landing them now would be vacuous no-ops (VR-5: no upgrade past basis). → **Enacted** still gated on the §6/§7 verification harness (DN-38 §7) **and** these completions. Prior: **Proposed** (2026-06-27). |
| **Feeds** | The generative-lowering row of the DN-38 §6 Lowering Map; the `derive` + `reveal` RFC track; the grammar wave (DN-31/RFC-0037); the verification harness (DN-20 tiers); KC-3 invariant tracking. |
| **Date** | June 27, 2026 |
| **Decides** | *Design proposal (all `Declared`; no code):* (1) the **extension-definition surface** — how a user declares a generative-lowering rule written in Mycelium lexicon with an explicit L0/surface RHS; (2) **inspectability-by-construction** — why there is no opaque step; (3) the **checker** — what the compiler enforces on every user-defined lowering; (4) the **verification discipline** — the same §6 differential + hygiene + round-trip the built-in lowerings hold to; (5) the **KC-3 invariant** — user extensions add no kernel node. |
| **Task** | M-812 |

> **Posture (transparency rule / VR-5 / G2).** This note is a **design DN — Proposed, not
> Accepted**. Every guarantee claim is tagged at its established strength. Design-level claims
> are `Declared` (cited to their basis in the corpus, but not ratified). Prior-art parallels
> are `Empirical` (primary-source-verified in `research/25`). No claim is upgraded past its
> basis. The ratified decision that motivates this note — user-extensible generative lowering
> is the direction — is recorded in DN-38 (Accepted) and its 2026-06-27 changelog. Every gap
> is named, not buried (G2).

---

## §1 Background — the ratified decision and its open task

DN-38 (Accepted, 2026-06-26) ratifies **generative lowering** (`derive`) as the mechanism by
which a terse surface term generates an explicit, content-addressed, inspectable L0 artifact.
DN-38 §8.1 settles the naming: the generative construct is **`derive`**, the inspector is
**`reveal`**, and delegation uses **`via`**.

The DN-38 changelog entry of 2026-06-27 **resolves** the previously-open "closed vs.
user-extensible" question in favor of user-extensibility, with the key architectural ruling:

> *A user "macro" is a **user-authored transparent lowering rule**, subject to the **same
> lowering law as the compiler's own passes**. Because the extension is its lowering (surface
> term → explicit L0), every use is `reveal`-able by construction.*

This note designs the **extension-definition surface** and **checker** for that ruling — the
surface form a user writes, what the compiler enforces, and why the by-construction guarantee
holds. It is **gated on** the DN-31/RFC-0037 grammar wave (which the `[]`-delimiter syntax and
derive forms ride) and the DN-38 §7 verification harness.

---

## §2 Why "macro" is the wrong name here (DN-02 three-test gate)

DN-02 §1 requires every candidate keyword to pass the **T-map / T-illuminate / T-learn** gate.
"Macro" fails on two tests:

- **T-map failure.** "Macro" implies — from C-preprocessor and Lisp heritage — *token-level
  text substitution operating before type information is available*, producing text the compiler
  then parses afresh. A Mycelium user-extensible lowering rule operates on the **typed
  AST/L0**, not on tokens, and its RHS is an explicit lowered term, not a template that rewrites
  source text. The metaphor implies behavior the construct does **not** have (disqualified, DN-02
  §1).
- **T-illuminate failure for the key property.** "Macro" actively *teaches* the wrong property
  (token rewriting, possible capture, possible opacity) where Mycelium's construct teaches the
  opposite (typed, structural, inspectable lowering). Calling it a macro would mislead.

The correct name for a **user-authored transparent lowering rule** is resolved by this note at
**`lower`** as the declaration keyword (§3 below; full gate application there). The `derive`
keyword (DN-38 §8.1, ratified) names the *use site* — `derive Foo for T` — where the user
applies a lowering rule (whether built-in or user-defined). `lower` names the *definition
site* where the user authors a new lowering rule.

---

## §3 The extension-definition surface — the `lower` keyword

### §3.1 Keyword proposal: `lower`

A user defines a new generative-lowering rule with the keyword **`lower`**. Running the
DN-02 three-test gate:

- **T-map (fidelity).** `lower` names exactly what the construct does: it lowers a surface
  term to a more explicit (L0) form. It does not imply token substitution, runtime
  indirection, or code generation in an opaque sense — it is a *lowering rule*, which is
  precisely what the Lowering Law (DN-38 §2) governs. **Passes.**
- **T-illuminate (teaching value).** `lower` teaches "this construct defines a descent toward
  L0" — directly reinforcing the seamless-gradient thesis (DN-38 §1) and the Lowering Law.
  A developer reading `lower Foo …` immediately understands: this is a rule for how `Foo`
  terms reduce. **Passes.**
- **T-learn (dual readability).** `lower` is a plain English word with no competing keyword
  meaning in the current reserved-word set (DN-02/DN-03 confirmed; `lower` does not appear in
  the ratified lexicon). It is machine-familiar (compilers lower; the concept is universal
  in PL); it reads clearly for both human and LLM audiences. **Passes.**

**Verdict: `lower` is the recommended extension-definition keyword.** (`FLAG: the maintainer
should confirm this passes the full reserved-word collision check against the current grammar
artifacts — this note cannot run the live parser.`)

### §3.2 Surface form sketch

A user-defined lowering rule has three parts: the rule name, the parameter list (the *intent*
— what the rule abstracts over), and the RHS (the *explicit lowered term* it expands to).
The RHS must be expressible as a term in the L0/L1 surface or an already-defined lower-level
surface; it is **not** an arbitrary code block.

```
lower <RuleName>[<ParamList>] = <ExplicitLoweredTerm>
```

Worked sketch (illustrative — grammar rides RFC-0037; all `Declared`):

```mycelium
// A user defines a new derive-able rule "Checksum" that lowers to
// a specific L0 impl pattern (explicit Construct/Match/Lam/App).
lower Checksum[T: Bytes] =
  impl Checksum for T {
    fn checksum(self) -> Binary{32} =
      fold(self.bytes(), Binary{32}::zero, xor_fold_step)
  }

// Use site — the same `derive` keyword the compiler's own rules use:
derive Checksum for MyPacket
```

The RHS (`impl Checksum for T { … }`) is a concrete L1/L2 surface term — itself lowerable
to L0 by the existing lowering passes. The user's rule maps `derive Checksum for T` to
an explicit, named, real Mycelium term. There is no opaque template engine; the RHS is
ordinary Mycelium code, parsed and type-checked by the same front end.

### §3.3 What the RHS may and may not contain

**Permitted:**
- Any expression valid at L1/L2 — `impl`, `fn`, `match`, `Construct`, `Fix`, `Lam`, `App`
  (RFC-0011), trait impls (RFC-0019), effects `!{…}` (RFC-0014), `@matured` gates
  (RFC-0017) — in terms of the rule's type parameters.
- References to the parameters that appear in the `derive` use site (the type `T` being
  derived for, trait bounds, const/width params — the same set allowed in a built-in
  `derive`-like rule).
- Calls to other already-checked user-defined lowering rules (no mutual recursion — §4.2).

**Not permitted (enforced by the checker, §4):**
- Token manipulation or source-text splicing. The RHS is a typed term, not a token stream.
- Calls into `wild { … }` blocks (FFI/host — RFC-0028). A generative lowering rule must be
  pure; the `wild` gate is level-independent (DN-38 §3) and does not vanish under a derive.
- New L0 kernel nodes. The RHS must lower to existing L0 forms (KC-3; §6 below).
- Mutually recursive lower rules (the checker rejects cycles — §4.2).
- Rules parameterized over runtime-only information (generative lowering is a compile-time
  mechanism).

---

## §4 The checker — what the compiler enforces

The DN-38 extensibility ruling states: a user-defined lowering is held to the **same lowering
law as the compiler's own passes**. The checker operationalizes this as a set of
**never-silent enforcements** (G2): a rule that fails any check is **rejected at definition
time**, not silently accepted and then broken at use.

### §4.1 IL-grammar check — the RHS must be a valid lower-level term

After expanding the rule's parameters, the compiler type-checks and grammar-checks the RHS
as an ordinary Mycelium term at the elaboration level of its output — the same grammar check
the nanopass discipline (DN-38 §2, citing Sarkar/Waddell/Dybvig ICFP 2004) applies to every
built-in pass's output. This is the **structural guarantee** that the RHS is a real,
well-formed lowered term and not a token soup.

**Guarantee (`Declared`):** A `lower` rule whose RHS is ill-typed or ill-formed is **rejected**
with a structured diagnostic (RFC-0013, Enacted, M-345) at definition time. No use site can
invoke a rejected rule — so a malformed extension cannot silently produce broken L0.

### §4.2 Termination / acyclicity check

User-defined lowering rules must **not** form cycles (mutual recursion among `lower` rules).
The checker maintains a dependency graph of `lower` rule invocations and rejects any strongly-
connected component of size > 1 with a never-silent diagnostic naming the cycle. This is the
same acyclicity requirement that keeps the built-in lowering passes a DAG — needed for
termination of the lowering pipeline (`Declared`).

Rationale: a cyclic lowering rule could cause the elaboration pipeline to diverge; the
acyclicity check is the structural guarantee that `derive` application terminates. This is
**not** a limitation of what the user can express at L1/L2 (the RHS may use `Fix` for
user-level recursion), but a constraint on the *lowering rules themselves*.

### §4.3 Hygiene — operates on the typed AST, not raw tokens

A user-defined lowering rule operates on the **typed AST** — the compiler's post-parse,
post-type-inference term representation — not on source tokens. This structural hygiene
(Ullrich & de Moura, *Hygienic Macro Expansion for Theorem Proving Languages*,
arXiv:2001.10490, `Empirical`) means:

- **No free-variable capture by construction.** Because the rule's parameters are typed
  and bound at definition time, and the RHS is type-checked in the same lexical scope,
  there is no token-level substitution that could accidentally capture a name from the
  use-site context. Variable capture — the classic unhygienic macro defect — is not
  a runtime check; it is **structurally impossible** for a typed-AST lowering rule.
- **Binding structure is preserved.** The generated term has the same binding structure
  as written in the RHS; names introduced in the RHS do not leak into the use-site scope
  unless the rule explicitly returns them as part of its output type.

**Guarantee (`Declared`):** Every user-defined lowering rule is hygienic by the structure
of the mechanism (typed AST, not token stream). The hygiene-no-capture verification check
(DN-38 §7) confirms this property empirically over generated programs.

### §4.4 Content-addressability

The generated L0 output of a user-defined lowering rule is **content-addressed** (ADR-003,
Unison-style — Exact at the substrate, Declared as applied to the generated output). Two
`derive` applications that produce the same L0 term — whether from the same rule applied
to the same type, or two rules that happen to produce identical L0 — **deduplicate by
hash**. No mutable registry; no interning table; no singleton lifecycle. The value-semantic
singleton-in-a-set is the substrate.

### §4.5 `reveal`-ability — the by-construction guarantee

Because a user-defined lowering rule maps a surface term to an **explicit, named, typed L0
term** (the RHS), `reveal` works on any use of a user extension automatically and
identically to how it works on a built-in `derive`. There is **no additional machinery
needed to make user extensions `reveal`-able**: the extension is its lowering, so the
`reveal` inspector — which shows the real L0 term the kernel runs (DN-38 §5) — already
has the term available.

This is the central inspectability-by-construction argument (§5 below).

### §4.6 Never-silent refusals (G2 checklist)

| Violation | Refusal |
|---|---|
| RHS is ill-typed or fails IL-grammar check | Reject at definition time with structured diagnostic (RFC-0013) |
| `lower` rule dependency graph has a cycle | Reject with cycle-naming diagnostic |
| RHS contains a `wild { … }` block | Reject — generative lowering is a pure compile-time mechanism |
| RHS introduces a new L0 kernel node | Reject — KC-3; kernel never grows for ergonomics |
| Use of a rejected rule at a `derive` site | Reject use site with reference to definition-time error |
| `derive` application produces a term that fails the IL-grammar check of its target level | Reject with structural-mismatch diagnostic |

---

## §5 Inspectability by construction — why this gives macro power with no black boxes

The DN-38 extensibility ruling is precise: because the extension **is** its lowering (surface
term → explicit L0), every use is `reveal`-able by construction. This section unpacks why
this is structurally guaranteed, contrasted with the systems Mycelium rejects.

### §5.1 The black-box failure modes Mycelium rejects

Three failure modes in prior art, each ruled out by construction here:

1. **C preprocessor.** Token-level text substitution before parse; the substituted text may
   fail hygiene (capture any name in the use-site scope), produce ill-formed tokens, or
   generate code whose structure is not visible in any typed AST. There is no `reveal`
   because there is no typed artifact — only a flat token stream. **Ruled out:** Mycelium
   lowering rules operate on the typed AST (§4.3); token manipulation is not possible.

2. **Lombok (and similar compiler-plugin codegen).** Lombok mutates the compiler's internal
   AST during an early compilation phase, generating `impl` methods that are **not in any
   source file** and are not recoverable from the class file alone. The defect is not
   generation — it is opacity (DN-38 §4 point 1/2, `Empirical`/`Declared`). There is no
   `reveal` because the generated code is a side-effect on an invisible internal data
   structure. **Ruled out:** the only output channel of a Mycelium lowering rule is "produce
   an L0 term" — a first-class value in the frozen kernel (DN-38 §4 point 2). The inspectable
   artifact exists by construction; the mechanism cannot accidentally become Lombok.

3. **Unconstrained procedural macros** (e.g. Rust proc-macros, Common Lisp macros). A
   procedural macro is a function from token stream to token stream — Turing-complete,
   side-effect-capable, able to read files, call the network, or generate code whose
   structure is intentionally obfuscated. `cargo expand` can textually dump the output
   (with the lossiness caveat — DN-38 §5, citing dtolnay's own README), but nothing
   *structurally enforces* that the output is a well-typed, hygiene-preserving term.
   **Ruled out:** a `lower` rule's RHS is a **typed Mycelium term**, not a function on
   token streams. It cannot read files, call the network, or produce tokens outside
   the Mycelium grammar. The structural constraints of §4 are enforced by the checker.

### §5.2 The positive argument

A user-defined `lower` rule is a **direct mapping** (surface term → explicit lowered term),
not an opaque computation that happens to produce code. Because:

- The RHS is a real Mycelium term, type-checked and IL-grammar-checked at definition
  time (§4.1);
- The rule is hygienic by the structure of the mechanism (§4.3);
- The generated output is content-addressed (§4.4);
- `reveal` shows the real L0 term the kernel runs (DN-38 §5) —

**the `reveal` tool already has everything it needs** to show the full lowered expansion of
any `derive` application of any user-defined rule, with no special case for user rules vs.
built-in rules. The extension surface and the built-in surface are the **same mechanism**;
user rules are not a separate facility bolted on top.

This is the Mycelium version of what DN-38 §4 calls the "EASIER" result: Lombok opacity is
**structurally impossible** here because the only output channel is "produce an L0 term."
The difference from Rust `#[derive]` (which `cargo expand` can textually dump with the
lossiness caveat) is that Mycelium's `reveal` shows the **actual L0 term** (not a
reconstructed text), so there is no hygiene lossiness to caveat (DN-38 §5, point 1).

### §5.3 Contrast summary

| System | Surface form | Generation mechanism | Black box? | `reveal`-able? |
|---|---|---|---|---|
| C preprocessor | `#define` token template | token substitution (pre-parse) | Yes — no typed artifact | No |
| Lombok | annotation | compiler plugin mutates hidden AST | Yes — side-effect on internal state | No |
| Rust proc-macro | `#[derive(Foo)]` | Turing-complete fn: TokenStream→TokenStream | Partially — `cargo expand` dumps text (lossy) | Lossy only |
| Mycelium `derive` (built-in) | `derive Foo for T` | typed RHS term, IL-grammar-checked | No — L0 term is a first-class value | Yes, exact L0 |
| Mycelium `lower` (user-defined) | `lower Foo[…] = <RHS>` | same mechanism as built-in | No — by construction | Yes, exact L0 (same path) |

---

## §6 KC-3 — user extensions add no kernel node

**KC-3 (house rule #5):** the kernel (L0/L1) never grows for ergonomics. The Lowering Law
(DN-38 §2) makes this the constraint on every new lowering pass: a new surface feature adds
an *early* lowering pass (which reduces it to existing L0 forms), not a new L0 node.

For user-defined lowering rules: the RHS of a `lower` rule must be expressible as a term
built from **existing L0 nodes** — `Construct`, `Match`, `Lam`, `App`, `Fix`, `Lit`, `Swap`
(RFC-0011, Enacted r3). The checker (§4.1) enforces this by IL-grammar-checking the fully-
elaborated RHS against the frozen L0 grammar. A RHS that requires a new L0 concept is
**rejected** — not as a policy call at review time, but as a **structural refusal** of the
checker (§4.6).

This means user extensions are **strictly additive at the surface** and **strictly
non-additive at the kernel**. The kernel remains the small, auditable, theorem-provable
base (KC-3); user extensions live entirely in the elaboration layer above it. A user adding
a `lower` rule cannot expand the kernel even if they want to — the checker prevents it.

**Guarantee (`Declared`):** every successfully checked `lower` rule lowers to a term in the
existing L0 grammar. The kernel size (number of node types in RFC-0011) is an invariant
the checker maintains.

---

## §7 Verification discipline — same as §6 built-in lowerings

DN-38 §7 defines the three-check verification discipline for every lowering pass. User-
defined lowering rules are held to the **same discipline**, not a lighter one. This is the
"its observational-identity claim is earned, not asserted" requirement (VR-5).

### §7.1 Differential — `observe(surface) == observe(lower(surface))`

For each user-defined lowering rule, the verification harness generates a corpus of `derive`
applications (surface) and their fully-lowered L0 expansions, and asserts they produce the
same observable value. This is the DN-38 §7 per-pass differential, applied to each rule.

**Tiered (DN-20):** LOW proptest cases every commit; HIGH on release. The property is never
dropped — only the case count is tiered (`Declared`, follows from DN-20 + DN-38 §7).

**Grounding:** value semantics make "observable meaning" clean — the oracle is value equality
of results, no aliasing or effect interleaving to quantify over (DN-38 §7, `Declared`).

### §7.2 Hygiene — no free-variable capture

The verification harness generates programs where the `derive` use site has names that could
be accidentally captured by the rule's RHS (free variables in the expansion scope), and
asserts no capture occurs. For user-defined rules, the structural guarantee (§4.3) makes
this a **confirmation** rather than a defense: the structural guarantee is `Declared`; the
empirical tests confirm it on the generated corpus (`Empirical`).

### §7.3 Round-trip — `delaborate ∘ lower = id` (certified mode)

The inspector obligation (DN-38 §5): in `certified` mode (ADR-032, tunable certification),
the round-trip `delaborate ∘ lower = id` is a testable property — the lowered L0 term
re-elaborates to the same surface term. For user-defined rules, this same obligation applies.

**Gated on `certified`** (the `fast` default skips the round-trip test; `certified` mode
enables it). **Not** dropped — gated (VR-5; the property exists, it is just not checked on
every `fast` build).

### §7.4 Tag posture

A user-defined lowering rule's **observational-identity claim** is:

- `Empirical` once the differential + hygiene corpus tests pass (trials support it);
- `Proven`-per-run only when the `certified`-mode translation-validation witness (DN-38 §7,
  Necula PLDI 2000) certifies the specific run.

The rule author cannot self-attest a stronger tag; the checker surfaces the tag earned by
the verification evidence (VR-5 — no upgrade past the basis).

---

## §8 Open questions and sequencing gates

1. **Grammar wave gate.** The `lower` keyword and `derive` use-site syntax ride the
   DN-31/RFC-0037 grammar wave. This note cannot be implemented before that wave lands.
   **Note (orchestrator, integration 2026-06-27):** RFC-0037 (the binding grammar
   deconfliction + layout-independence RFC) was authored in this same wave, so the `lower`
   keyword and `derive` use-site syntax now ride RFC-0037 / DN-31 / RFC-0030. The
   user-extensible lowering surface itself will still need its own enacting RFC (or an
   extension of RFC-0030) once this DN is accepted.

2. **Verification harness gate.** The §7 differential + hygiene + round-trip checks require
   the DN-38 §7 verification harness to exist. This DN's DoD cannot be met before the
   harness is operational.

3. **`lower` vs. alternative spellings.** The T-map/T-illuminate/T-learn analysis in §3.1
   recommends `lower`; open to the maintainer's review. Alternative candidates: `rule`
   (conventional, weak T-illuminate — teaches little about *what kind* of rule); `rewrite`
   (T-map concern — implies arbitrary rewriting, not necessarily to a typed term; `grow`
   is now superseded by DN-03 changelog; `template` fails T-map (implies text templates).
   `lower` is the cleanest single-word form that names the actual mechanism.
   `FLAG: maintainer review requested on the keyword.`

4. **Mutual-recursion policy.** §4.2 prohibits cycles among `lower` rules. This is the
   conservative safe-default; a future DN could allow a restricted form (e.g., rules that
   prove termination via a structural metric). Recorded here so the restriction is explicit
   (G2/append-only — not reversed, extended later).

5. **Rule-parametric effects.** What if a user-defined lowering rule's RHS legally uses
   `!{io}` effects? The rule must declare its effect signature; the use-site picks it up.
   The interaction with the effect-budget system (RFC-0014, Enacted) is an open design
   detail — not a blocker, but needs specifying in the RFC.

6. **Cross-phylum lowering rules.** Can a `lower` rule defined in one phylum be `derive`d
   in another? This is the expected use case (a library author ships a reusable rule);
   the content-addressed identity (§4.4) and the checker (§4.1) compose across phyla.
   The inter-phylum import model is the existing `use` mechanism (DN-02/DN-03) — no new
   mechanism needed, but the RFC should confirm scope.

---

## §9 Guarantee posture (VR-5) + Definition of Done

**Grounding posture (held throughout):**

- **Built / `Exact`** — the L0/L1 layering (RFC-0006 §3), the observational-identity invariant
  (RFC-0012, Enacted), interp≡AOT (NFR-7), the built lowerings (RFC-0011, RFC-0014, RFC-0017,
  RFC-0019, all Enacted). Content-addressing is Exact at the substrate (ADR-003).
- **Designed / `Declared`** — the **`lower` keyword and surface form** (§3), the **checker
  rules** (§4), the **by-construction inspectability argument** (§5), the **KC-3
  enforcement** (§6), the **verification discipline application to user rules** (§7), and
  the **open questions** (§8). Their prior-art parallels (Racket Macro Stepper, Lean `pp.all`,
  Rust proc-macro / `cargo expand`, Lombok contrast case — all `Empirical` at source;
  Ullrich & de Moura hygiene — `Empirical`) are cited, but their Mycelium mappings are
  `Declared`-with-argument, not ratified.
- **Gaps** — grammar wave (RFC-0037) is not yet proposed; verification harness is not yet
  built; `lower` keyword is not yet reserved; implementation is not yet started. Named,
  not buried (G2).

**Definition of Done (the gate for Proposed → Accepted).** This note is `Accepted` when the
maintainer ratifies: **(a)** the **`lower` keyword** (or an alternative the maintainer
selects) as the extension-definition surface; **(b)** the **checker rules** (§4 — IL-grammar
check, acyclicity, hygiene-by-construction, KC-3 enforcement, never-silent refusals);
**(c)** the **by-construction inspectability argument** (§5) as the design guarantee; **(d)**
the **same §7 verification discipline** for user rules (differential + hygiene + round-trip,
tiered, `certified` mode for round-trip); and **(e)** the **KC-3 invariant** (user extensions
add no kernel node). Acceptance feeds: an implementation RFC for the grammar-wave surface
form (riding DN-31/RFC-0037), a checker implementation task, and the verification harness
extension task. **Still enacts no code.** Append-only; VR-5; G2; nothing here moves another
doc's status.

---

## §10 Relation to corpus

- **DN-38** (Accepted, 2026-06-26) — the ratifying atlas; the extensibility ruling in its
  2026-06-27 changelog is the direct basis for this note. The `derive` + `reveal` + `via`
  naming is settled there (§8.1). This note operationalizes the ruling.
- **RFC-0006** §3 (L0–L3 layering; "L2 defined entirely by elaboration to L1") + §4.1 S1–S6.
- **RFC-0011** (Enacted r3) — the L0 node set; KC-3 enforcement targets this grammar.
- **RFC-0012** (Enacted) — ambient = observationally the identity; the observational-identity
  claim user rules must earn.
- **RFC-0013** (Enacted, M-345) — structured diagnostics; all checker refusals (§4.6) use
  this diagnostic system.
- **RFC-0014** (Enacted) — declared effects; user rules that use `!{…}` must declare them.
- **RFC-0017** (Enacted) — `@matured`; user rules may reference this gate in their RHS.
- **RFC-0019** (Enacted) — traits/coherence/monomorphization; user rules are parameterized
  over trait bounds in the same way built-in `derive` rules are.
- **ADR-003** — content-addressed identity (Unison-style); user-rule output deduplication.
- **ADR-032** — tunable certification; `certified` mode gates the round-trip check (§7.3).
- **DN-02 / DN-03** — naming law + three-test gate; the `lower` keyword recommendation is
  grounded here (§3.1).
- **DN-20** — tiered testing; the LOW/HIGH proptest-case tiers for the differential (§7.1).
- **DN-31 / RFC-0030** — the grammar wave this note's surface form is gated on.
- **KC-3** (house rule #5) — kernel never grows; §6 is the operationalization for user rules.
- **G2** (never-silent) — all checker refusals are explicit; no silent accepts.
- **VR-5** (downgrade-don't-overclaim) — tag discipline throughout.
- **External prior art (`Empirical` at source; Mycelium mapping `Declared`):**
  Ullrich & de Moura, *Hygienic Macro Expansion for Theorem Proving Languages*
  (arXiv:2001.10490) — hygiene by typed AST; Lean 4 `pp.all` (injective delaborator,
  round-trip); Culpepper & Felleisen, Racket Macro Debugger (step-wise expansion) — contrast
  cases for what `reveal` supersedes; dtolnay, `cargo expand` (lossiness caveat — text-dump
  is lossy); Lombok contrast case (AST mutation opacity); Sarkar, Waddell, Dybvig, *A
  Nanopass Infrastructure* (ICFP 2004) — IL-grammar-checked passes; Necula, *Translation
  Validation for an Optimizing Compiler* (PLDI 2000) — certified-mode witness. Full URLs in
  `research/25-layered-lowering-and-generative-sugar-prior-art-RECORD.md`.

---

## §10 Derive-site attachment — design options (addendum, 2026-06-29, M-824)

> **Design-pass posture (VR-5 / G2).** This section is a **design pass for maintainer
> ratification**. DN-54 remains `Accepted`; no status is advanced here. All new claims are
> `Declared` (design, not ratified). Every normative claim cites its corpus basis. This section
> is append-only: §§1–9 are not modified.

---

### §10.1 The underdetermined residual — what is open

The M-812-cont landing (2026-06-29 changelog entry) surfaces two tightly coupled open facets:

**(a) Attachment model.** `lower` defines *what* the generated L0 is. `derive Name for T`
instantiates it. But **where does the resulting L0 live in the program?** The v0 elaborator
(`elaborate(env, entry)`) produces one L0 `Node` from one nullary function entry — a
self-contained lambda value. A `derive` application must place its output somewhere so that
code that uses `derive Checksum for MyPacket` can refer to the resulting checksum
implementation. Two broad candidate models are identified here (§10.3); others are named as
open (§10.6).

**(b) Item-not-Expr RHS gap.** The §3.2 worked example has a `lower` rule whose RHS is:

```mycelium
lower Checksum[T: Bytes] =
  impl Checksum for T {
    fn checksum(self) -> Binary{32} = …
  }
```

An `impl` block is an **item** (a declaration), not an **expression**. The v0 parser
(`parse_lower_decl`) calls `parse_expr` for the RHS, which cannot accept an `impl` item.
This is not a bug in the landed code — the landed nullary elaboration correctly handles
expression-shaped RHS — but it is a **structural gap between the design intent (§3.2) and
the v0 grammar capability**: the canonical motivating use case (derive a trait implementation)
is not yet expressible. This addendum designs paths to close it.

**(c) Parametric instantiation.** A `lower Name[T]` rule whose RHS references `T` has no
monomorphic L0 until a `derive Name for ConcreteType` instantiates `T`. The current
elaboration of such rules produces `ElabError::Residual` (never-silent, G2). The residual
is correct; the question is how the attachment model and the monomorphization path
(`mono.rs::emit_fn` — RFC-0019 / DN-55) compose at the derive site.

---

### §10.2 Framing: what a satisfactory model must do

Before enumerating candidates, the criteria (`Declared` — design gates, not ratified):

1. **The generated L0 must be named and reachable** — call sites of `derive Checksum for
   MyPacket` must be able to resolve `Checksum` for `MyPacket` without re-running the rule.
   This is a content-addressability constraint (ADR-003): equal `derive` applications must
   deduplicate, and the identity must be derivable from `(rule_name, concrete_type_args)`.

2. **No new L0 node (KC-3).** The attachment mechanism may add **no** new L0 node. Models
   that require a new "impl slot" node or a "derived-instance pointer" node are rejected by
   the same structural guard that enforces §6. (`Declared` — follows from KC-3 + §6.)

3. **Coherence-compatible (RFC-0019).** The trait-impl coherence system (RFC-0019 §4.2 —
   orphan rule, global uniqueness) must still hold. A `derive` application that generates an
   `impl` must be treated by the coherence checker identically to a hand-written `impl`.
   (`Declared` — follows from RFC-0019.)

4. **Never-silent (G2).** A `derive` application that conflicts with an existing `impl`
   (coherence violation) or whose RHS does not type-check at the concrete type must be an
   explicit error, never a silent accept or silent discard.

5. **Reveal-able (§4.5).** The attachment must not hide the generated L0. `reveal` must show
   the same L0 term regardless of which attachment model is chosen. (`Declared` — follows from
   §4.5 + DN-38 §5.)

---

### §10.3 Candidate attachment models

Two primary candidates are enumerated below, with a third noted as an open extension (§10.6).

---

#### Model A — Sibling-item injection

**Mechanism (`Declared`):** `derive Name for T` is elaborated as if the user had written the
RHS as a sibling item in the same nodule, with `T` substituted for the rule's type parameter.
The elaborator — after monomorphizing the rule's RHS at `T` — inserts the resulting item
(e.g. an `impl` block) into the nodule's item list at the derive site, as a co-equal sibling
declaration. The generated item is content-addressed from `(rule_name, T)` and registered in
`Env` under the same namespace as hand-written `impl` blocks.

**How it addresses the item-not-Expr gap (`Declared`):** The RHS of a `lower` rule, when
the rule is item-shaped, is parsed as an **item template** rather than an expression. A new
`parse_lower_item_rhs` arm in the parser accepts item-shaped RHS forms (currently: `impl …
for …` with a concrete body parameterized over the rule's type vars). At elaboration time, the
monomorphizer substitutes `T` throughout the item body — exactly as `mono.rs` substitutes a
type parameter into a function body (RFC-0019 §4.3 / DN-55 §2.1). The resulting closed item
is inserted as a sibling.

**How it handles parametric instantiation (`Declared`):** Monomorphization (`mono.rs`) is the
natural vehicle. The rule's type parameter `T` is treated exactly like a generic function's
type parameter: the `derive` use site provides the concrete type, and `mono.rs` produces the
closed item. If `T` is undetermined at the `derive` site, the elaborator emits
`ElabError::Residual` — never-silent, the same behavior as today (G2/VR-5).

**Coherence (RFC-0019 §4.2 / Declared):** The injected `impl` is visible to the coherence
checker as a sibling item, so global-uniqueness holds: a second `derive Checksum for MyPacket`
in the same program is a duplicate `impl Checksum for MyPacket` — caught by the existing
coherence pass, never-silent.

**`reveal` (§4.5 / Declared):** `reveal` shows the content-addressed L0 of the injected
item. No special case needed: the sibling item is a real program item that went through the
full elaboration pipeline.

**KC-3 impact (`Declared`):** No new L0 node. The injected `impl` lowers to existing
`Construct`, `Lam`, `App`, `Match`, `Fix` nodes (RFC-0007 §4.1). The injection mechanism
itself is an elaboration-phase rewrite, not a new kernel concept.

**Machinery cost (`Declared`):** Requires:
- A `parse_lower_item_rhs` parser variant to accept item-shaped RHS.
- An `elaborate_derive_as_sibling` elaboration path that calls `mono.rs` on the RHS and
  inserts the result as a new `Item` in the nodule's item list.
- A de-duplication guard: if a monomorphic result is already in `Env`, the `derive` is a
  no-op (or a coherence error if the existing item differs). Content-addressing (ADR-003)
  provides the de-dup key.

---

#### Model B — Registry-of-derived-impls (derived-impl table)

**Mechanism (`Declared`):** `derive Name for T` does **not** inject a sibling item. Instead,
a separate **derived-impl table** (a side-structure in `Env` or a companion data structure) is
populated with an entry `(rule_name, concrete_T) → L0_node`. The consuming path — trait
method dispatch, `reveal` queries — looks up the derived-impl table in addition to the
hand-written `impl` namespace.

**How it addresses the item-not-Expr gap (`Declared`):** Same parser variant needed as Model
A for item-shaped RHS. The difference is purely in the output: instead of injecting a sibling
item, the elaborator stores the monomorphized L0 in the derived-impl table under the content-
addressed key.

**How it handles parametric instantiation (`Declared`):** Same monomorphization path as Model
A. The derived-impl table entry is keyed on `(rule_name, monomorphized_type_args)`. Residual
on undetermined `T` — same never-silent behavior.

**Coherence (`Declared`):** Requires coherence checking to cover *both* the hand-written
`impl` namespace *and* the derived-impl table. A `derive` whose entry would conflict with an
existing hand-written `impl` must be a coherence error. This is an additional checking surface
not required by Model A (where the injected `impl` is already in the normal coherence path).

**`reveal` (§4.5 / `Declared`):** Requires `reveal` to query the derived-impl table. The L0
term is available (it was stored); the mechanism is a lookup rather than re-elaboration.

**KC-3 impact (`Declared`):** The derived-impl table is an elaboration-phase data structure,
not a kernel node — KC-3 is satisfied. However, the **consumption path** (how a call site
resolves `impl Checksum for MyPacket` when the only source is a derived entry) requires the
trait-dispatch machinery to be aware of the table. This is more elaboration-phase surface than
Model A, which simply makes the derived impl a peer of hand-written impls.

**Machinery cost (`Declared`):** Requires:
- The same `parse_lower_item_rhs` parser variant.
- A `DerivedImplTable` structure in `Env` (a new data structure, not a new L0 node).
- Extended coherence checking that covers both namespaces.
- Extended trait-dispatch that queries the table.
- Extended `reveal` that queries the table.

---

### §10.4 Honest tradeoff table

All entries `Declared`.

| Criterion | Model A — Sibling injection | Model B — Derived-impl registry |
|---|---|---|
| **KC-3 impact** | None — no new L0 node; no new `Env` data structure for dispatch | None — the registry is an elaboration artifact, but dispatch and coherence must be extended to cover it |
| **New machinery** | `parse_lower_item_rhs` (parser); `elaborate_derive_as_sibling` (elab); de-dup guard (trivial via ADR-003 content key) | `parse_lower_item_rhs`; `DerivedImplTable` (new `Env` field); extended coherence + dispatch + reveal |
| **Coherence integration** | Free — injected `impl` enters the existing coherence pass unchanged | Requires coherence to cover two namespaces; explicit dual-lookup invariant |
| **`reveal` integration** | Free — the injected item is a first-class item in the elaborated nodule | Requires `reveal` to query the registry; a new query arm |
| **Ergonomics** | Derived impls behave exactly like hand-written impls — no "second class" split | Derived impls are a separate namespace; user-visible distinction possible (good for debuggability, bad for simplicity) |
| **Never-silent (G2)** | Coherence error on duplicate — same path as hand-written impls | Coherence error requires explicit dual-namespace check; more surface for a missed case |
| **KISS / KC-3 preference** | Strongly favored — fewer concepts, uses existing elaboration paths | Weaker — adds a new data structure and three new query/check arms |
| **Debuggability** | `reveal` shows a real program item; the injected item is browsable in the elaborated item list | `reveal` shows a registry entry; not a "real" item unless promotion is added |

---

### §10.5 Recommendation (for maintainer ratification)

**Recommended: Model A — sibling-item injection** (`Declared` — design recommendation,
not ratified).

Rationale:
- Model A has strictly less new machinery. The three extension points Model B requires
  (extended coherence, extended dispatch, extended `reveal`) are all eliminated because
  the injected impl is a peer of hand-written impls in the existing elaboration paths.
- KC-3 preference (house rule #5): fewer concepts, composition over novel structure. A
  new `DerivedImplTable` in `Env` is a new concept that must be maintained across the
  elaboration, coherence, dispatch, and `reveal` surfaces. Model A adds no such concept.
- Coherence by construction (RFC-0019): Model A makes the coherence invariant hold
  structurally — a derived impl *is* an impl entry, so the existing global-uniqueness check
  covers it without extension.
- Reveals are already exact (§4.5) — the injected item went through the full elaboration
  pipeline, so `reveal` needs no special case.
- The item-not-Expr parser gap is equally addressed by both models; the parser extension
  (`parse_lower_item_rhs`) is shared.

**Disconfirming argument (stated, not buried — VR-5):** Model B has one advantage: it
preserves an explicit record of *which impls came from `derive`* vs hand-written, which
could be useful for tooling (e.g. IDE "this impl was generated by `lower Checksum`"). Model
A discards that provenance once the impl is injected. Counter: content-addressing (ADR-003)
means the impl's hash encodes its origin; `reveal` can reconstruct the rule-name from the
generation path if the provenance is recorded in metadata (RFC-0001 §4.3 — `provenance`
field). This is `Declared` — the provenance-metadata path is not designed in detail here;
it is recorded as an **open question for the implementing RFC** (§10.6 OQ-A).

---

### §10.6 Open questions for the implementing RFC

**OQ-A. Provenance metadata (`Declared` — open).** Should `derive`-generated impls carry a
`provenance` tag (RFC-0001 §4.3) that records `(rule_name, instantiation_args)` so that
tools can distinguish them from hand-written impls? The metadata field exists; the question
is whether the elaborator populates it and whether the surface grammar exposes a query.

**OQ-B. Item-RHS parser scope (`Declared` — open).** Which item forms are legal as a `lower`
rule RHS? The minimum needed for the §3.2 use case is `impl Trait for T { … }`. A broader
set (e.g. `type` aliases, standalone `fn` items) may be useful but carries more parser
surface. The implementing RFC should enumerate the supported set explicitly (G2 — no silent
over-generalization).

**OQ-C. Mixed expr-and-item rules (`Declared` — open).** Should a `lower` rule be able to
generate *both* an expression-shaped result *and* one or more sibling items? E.g. a rule
that generates both a helper `type` alias and an `impl`. This is a future extension; the
v1 design should support it if the parser architecture does not preclude it.

**OQ-D. Cross-phylum derive and coherence scope (`Declared` — open, carries §8 Q6 forward).**
DN-54 §8 Q6 notes that cross-phylum `derive` is the expected use case. Under Model A,
sibling injection in a *different* phylum from the rule definition is an inter-phylum item
insertion — the coherence and namespace model must specify where the injected item "lives."
This is the phylum-level attachment scope question; the implementing RFC must settle it.

**OQ-E. Effect annotation on item-RHS rules (`Declared` — open, carries §8 Q5 forward).**
A `lower` rule whose item-shaped RHS contains effectful methods (using `!{io}`, RFC-0014)
must declare its effect signature. The item-RHS case may interact with effect-budget
propagation differently from the expression-RHS case. Recorded for the implementing RFC.

---

### §10.7 Sequencing gate and Definition of Done for this addendum

This section is a **design pass for ratification**. DN-54 remains `Accepted`. This addendum
is complete (as a design document) when the maintainer has reviewed and signalled one of:

- Accept Model A (or Model B, or a maintainer-chosen variant) → the attachment model is
  ratified and the implementing RFC can proceed.
- Request a revised design pass with different candidate models or criteria.
- Flag specific open questions (§10.6) as blocking before ratification.

The **implementing RFC** (a new RFC, or an extension of RFC-0030/RFC-0037 — to be decided)
carries the normative item-RHS parser, the sibling-injection elaboration path, and the
updated coherence and `reveal` surfaces. That RFC is the gate for status advancement toward
`Enacted` — not this addendum.

**This section does NOT advance DN-54 toward `Enacted`** (house rule #3: status must step
through `Accepted`; `Enacted` requires the implementing RFC and the §7 verification harness
to be both green and ratified). It resolves the FLAG recorded in the 2026-06-29 changelog
entry by offering a grounded, ratifiable design position.

---

## Meta — changelog

- **2026-06-27 — Created (Proposed) — authored.** Operationalizes the DN-38 2026-06-27
  extensibility ruling (user-extensible generative lowering, transparent by construction) into
  a concrete surface design. Records: **(1)** DN-02 three-test gate ruling out "macro" (§2) in
  favor of `lower` as the definition-site keyword; **(2)** the `lower` keyword proposal with
  T-map/T-illuminate/T-learn analysis (§3.1) and the surface-form sketch with a worked example
  (§3.2/§3.3); **(3)** the checker (§4 — IL-grammar check, acyclicity, hygiene-by-construction,
  content-addressability, `reveal`-ability, never-silent refusals); **(4)** the
  inspectability-by-construction argument (§5 — C-preprocessor, Lombok, and proc-macro
  contrast; positive argument; contrast summary table); **(5)** KC-3 enforcement (§6 — user
  extensions add no kernel node, checker enforces this structurally); **(6)** verification
  discipline (§7 — differential, hygiene, round-trip, tag posture — same as built-in
  lowerings, VR-5); **(7)** open questions and sequencing gates (§8 — grammar wave, harness,
  keyword review, mutual-recursion policy, effect interaction, cross-phylum scope);
  **(8)** guarantee posture (§9 — Declared throughout; DoD stated). Enacts nothing; moves no
  status; CHANGELOG / Doc-Index / issues.yaml / docs/api-index owned by integrating parent.
  (Append-only; VR-5; G2.)
- **2026-06-28 — Implemented (Rust-first, surface + structural checks); KC-3 + IL-grammar +
  RHS-elaboration deferred — pending ratification.** M-812 partial landing in `mycelium-l1`
  (obj+low integration wave). **Landed:** `Tok::Lower` + `Tok::Derive` become **active** keywords
  (settling the `grow → derive` reconciliation, DN-38 §8.1 — `grow` at item position now emits a
  teaching diagnostic pointing at `derive`); AST `Item::Lower(LowerDecl)` + `Item::Derive(DeriveDecl)`
  (`ast.rs`); `parse_lower_decl` (`lower Name[params]? = <rhs>`) + `parse_derive_decl`
  (`derive Name for T`) at item position, both **rejected at expression position** with teaching
  diagnostics (`parse.rs`, G2); ambient pass-through + surface re-render (`print_lower_decl` /
  `print_derive_decl`, `ambient.rs`); checker Pass 3e (`check_lower_decl` / `check_derive_application`,
  `checkty.rs`) — **never-silent structural** checks: rule-name uniqueness, param-name uniqueness,
  `derive` name-resolution; rule registered in `Env::lower_rules`; conformance fixtures
  `accept/22-lower-derive.myc`, `reject/26-lower-missing-eq.myc`, `reject/27-derive-missing-for.myc`,
  and the `reject/19-grow-reserved-not-active.myc` update; unit tests in `src/tests/{parse,checkty}.rs`
  (22 sections green). **Deferred (M-812-cont; held `Declared`, VR-5 — named, not buried, G2):**
  (1) **RHS elaboration to L0** — `crate::elab` does not yet read `Env::lower_rules`, so a `derive`
  emits **no L0 term** today (an honest never-silent residual, not a fabricated accept — pinned by
  the integration guard tests `lower_derive_items_add_no_l0_to_an_unrelated_entry` +
  `lower_rule_rhs_is_stored_not_elaborated`); (2) the **§4.1 IL-grammar RHS type-check**;
  (3) the **§6 KC-3 kernel-growth guard**; (4) **§4.2 cross-rule acyclicity**; (5) the **§7
  verification harness** (differential / hygiene / round-trip). Guards (2)/(3) are meaningful only
  once (1) lands. CHANGELOG / Doc-Index / issues.yaml / docs/api-index reconciled by the integrating
  parent. (Append-only; VR-5; G2.)
- **2026-06-29 — Implemented (Rust-first): RHS elaboration + KC-3 guard + §4.1/§4.2/§4.6 + §7
  harness landed — pending ratification.** M-812-cont landing in `mycelium-l1` (the `lwd` kickoff),
  closing the five deferred completions from the 2026-06-28 entry. **Landed:**
  **(1) RHS elaboration to L0** — `crate::elab::elaborate_lower_rule(env, rule_name)` now **reads
  `Env::lower_rules`** and lowers a rule's RHS to a closed L0 `Node` by running it through the **same**
  path a hand-written nullary `fn` body takes (a `%`-prefixed synthetic entry inserted into a cloned
  `Env`, fed to the existing `elaborate`), so the §7.1 differential holds *by construction*. Tag:
  **`Empirical`** (observational identity earned by the §7 trials + the M-210 `check_core` validation,
  never self-attested). **(2) §4.1 IL-grammar RHS type-check** — `check_lower_rule_rhs_type` runs the
  v0 checker (`infer_type`) over each rule's RHS with the rule's params in scope as `Ty::Var`; an
  ill-typed RHS is refused at definition time (`Declared`, G2). *Discovery (G2 — surfaced, not
  buried):* the prior structural-only check accepted `lower X = true`, but lowercase `true`/`false`
  are **not** L1 names — that RHS is ill-typed; it now refuses, and the fixtures/tests were corrected
  to the real prelude `Bool` constructors `True`/`False`. **(3) §6 KC-3 kernel-growth guard** —
  **`Proven`-by-construction** in the narrow, *checked* sense: the elaborator's codomain is the
  **closed** Rust enum `mycelium_core::Node` (the frozen L0 grammar), so a `lower` rule can introduce
  **no new kernel node** — the type system is the checked side-condition. The one *surface*-growth a
  rule could otherwise smuggle in (a host op) is closed by **(5) §4.6 purity** — `check_lower_decl_
  structural` + `rhs_contains_wild` refuse a `wild { … }` block in a rule RHS structurally (so the
  refusal holds even in an `@std-sys` nodule). The harness confirms KC-3 non-vacuously via
  `Node::is_aot_lowerable` (total over the frozen node set). **(4) §4.2 cross-rule acyclicity** —
  `check_lower_rule_acyclicity` builds the rule-reference graph (a rule's edges are the single-segment
  paths in its RHS that name *other rules*) and refuses self- and mutual-reference cycles via an
  iterative DFS (`Declared`, G2). **(5) §7 verification harness** — `tests/lower_derive.rs`:
  structural identity, run-value differential (through M-210 `check_core`), KC-3-stays-in-fragment,
  §7.2 hygiene (`%`-fresh binders ⇒ no capture), §7.3 value round-trip. The two `low`-era residual
  guard tests (`lower_derive_items_add_no_l0_to_an_unrelated_entry`,
  `lower_rule_rhs_is_stored_not_elaborated`) are **replaced** by the real elaboration + guard tests.
  `cargo test -p mycelium-l1` green (all suites); `cargo fmt` + `clippy -D warnings` clean.
  **FLAG — `derive`-site consumption / attachment semantics UNDERDETERMINED (held `Declared`; G2,
  not guessed).** DN-54 specifies *what a rule produces* (an explicit L0 term — now landed) but **not
  how a `derive Name for T` use site's instantiated L0 attaches to / is referenced by the surrounding
  program**: v0 `elaborate(env, entry)` produces *one* L0 `Node` from a nullary entry fn, with no
  registry-of-derived-impls consumption path, and the note's §3.2 worked-example RHS is an `impl`
  block — an **item**, not an `Expr`, so it is **not expressible** as a v0 rule RHS (which
  `parse_expr` requires). Two facets are open: **(a)** the *attachment* model (where a `derive`'s L0
  lives in a program); **(b)** *parametric instantiation* — a `lower Name[T]` whose RHS mentions `T`
  has no monomorphic L0 until a `derive` instantiates `T`, which the present elaboration surfaces as
  the ordinary generic `ElabError::Residual` (never-silent). Per the maintainer's standing rule (a
  correct partial landing beats a guessed elaboration — G2/VR-5), the **nullary/monomorphic** rule
  elaboration is landed and the attachment + parametric-instantiation model is **left for maintainer
  ratification** (this is the residual that supersedes the 2026-06-28 deferred set, narrowed to
  consumption-semantics only). **Status NOT advanced here** (Accepted→Enacted is the maintainer's /
  integrating parent's step through `Accepted` — house rule #3); this entry records the impl
  Rust-first and resolves the deferred-safety boundary. CHANGELOG / Doc-Index / issues.yaml /
  docs/api-index reconciled by the integrating parent. (Append-only; VR-5; G2.)
- **2026-06-29 — Design-pass addendum (M-824): §10 derive-site attachment — design options.**
  Appended §10 (this addendum) to resolve the FLAG in the 2026-06-29 impl entry. Enumerates
  two candidate attachment models — **(A) sibling-item injection** (derive produces a sibling
  `impl` item in the nodule, entering the existing coherence and dispatch paths) and **(B)
  derived-impl registry** (a side-table in `Env`, extended coherence and dispatch). Includes an
  honest tradeoff table and a recommendation to ratify Model A (fewer new concepts, KC-3-
  preferred, coherence by construction). Identifies `parse_lower_item_rhs` as the shared parser
  extension needed for item-shaped RHS. Records five open questions (provenance metadata,
  item-RHS parser scope, mixed expr-and-item rules, cross-phylum attach scope, effect annotation)
  for the implementing RFC. **DN-54 remains `Accepted`; no status advanced.** All new claims are
  `Declared` (design pass — VR-5). CHANGELOG / Doc-Index / issues.yaml reconciled by the
  integrating parent. (Append-only; VR-5; G2.)
- **2026-07-02 — Extension-checker enactment of DN-71 Model S (M-919); status held at `Accepted`
  (partial — VR-5), one FLAG raised.** Kickoff `grm`, task M-919, following the DN-75 (M-917)
  completion audit and the DN-71 (M-901) affine-consume-model sign-off. Two distinct findings:
  1. **A real coverage gap in the extension-checker, found and closed.** `check_lower_rule_rhs_type`
     (`crates/mycelium-l1/src/checkty.rs`) ran the RHS type-check with a permanently **inert** affine
     tracker, reasoned only from "a `lower` rule has no value parameters, so no `Substrate` binding
     is ever in scope" (`crates/mycelium-l1/src/affine.rs` module docs, pre-fix). That reasoning
     missed DN-54 §3.3's own permission for a rule's RHS to call other already-checked top-level
     `fn`s — nothing restricts those callees' return types, so a `let s = acquire() in …` inside a
     `lower` rule's RHS can legally bind a real `Substrate{tag}` value (`acquire`'s own body having
     used `wild` in an `@std-sys` nodule). A reproduction confirmed the gap **non-vacuously**: a
     same-rule double-consume of such a helper-acquired `Substrate` type-checked silently under the
     inert tracker. **Fixed**: the tracker is now seeded **active** (`Tracker::seeded(&[])`, an empty
     initial scope — a rule has no value parameters to seed from, but every subsequent
     `let`/lambda/match binder in the RHS walk is tracked exactly as `check_fn_body` tracks one, so
     DN-71 Model S §4.2's double-consume refusal now fires inside a `lower` rule's RHS too, citing
     DN-71 by name). Reject-case conformance:
     `lower_rule_rhs_double_consume_of_a_helper_acquired_substrate_is_refused`; the accept-side
     regression guard: `lower_rule_rhs_single_consume_of_a_helper_acquired_substrate_checks`
     (both `crates/mycelium-l1/src/tests/checkty.rs`). `cargo test -p mycelium-l1` green (all
     suites); `cargo fmt` + `clippy -D warnings -A unsafe_code` clean. Guarantee: the double-consume
     check itself is `Empirical` (M-903's own tag, unchanged); this fix only extends its *coverage*
     to a checker context that previously bypassed it — no tag is upgraded past its basis (VR-5).
  2. **FLAG — the M-918/M-919 `issues.yaml` framing conflates two different constructs; DN-54 §10's
     own subject remains genuinely unresolved.** `tools/github/issues.yaml`'s M-918 entry records
     the DN-54 §10 **derive-site attachment** question ("consumption model") as "RESOLVED: DN-71
     Model S". Reading DN-71 itself (its own §3 and §7) refutes that framing: DN-71 Model S is the
     **affine `Substrate`/`consume` execution model** (an interpreter-level opaque handle with
     static use-once checking) — a construct DN-71 §3 explicitly distinguishes from DN-54 §10's
     **attachment model** (where a `derive`'s generated L0 lives — Model A sibling-injection vs.
     Model B registry, §10.3–§10.5). DN-71 §7 states plainly these "are two different constructs
     sharing the word 'consume'" and even recommends M-918 be renamed to "attachment model" to stop
     the collision — the opposite of treating them as one resolved model. No maintainer ratification
     of DN-54 §10's attachment model (Model A vs. B) was found anywhere in the corpus (checked:
     `docs/planning/Blocked-Decisions-Ratification-Map.md`, `git log --all` for "attachment"/"Model
     A", `docs/notes/DN-76-*`) — DN-76 line 144 still lists it as a live **maintainer gate**
     (`grm M-918/M-919`), unresolved. So: **item 1 above genuinely satisfies "enact DN-71 Model S in
     the extension-checker"** (the literal M-919 DoD text), but it does **not** resolve DN-54 §10's
     own attachment-model question, and the `issues.yaml` M-918 entry's "MET BY: DN-71 Model S"
     wording should be corrected by the orchestrator (issues.yaml is orchestrator-owned; not edited
     here — G2, flagged not guessed). The DN-75 residual ledger's R-1/R-2/R-3/R-4 (attachment model,
     §4.4/§4.5/§4.6-row-6 downstream obligations, §7 corpus/DN-20 tiering, `certified`-mode
     round-trip) are **all still open** — none are touched by this task.
  3. **Editorial correction carried forward from DN-75 FLAG E-1** (docs/notes/DN-75, §4): the
     2026-06-29 changelog entry's claim that `lower_derive_items_add_no_l0_to_an_unrelated_entry`
     was "replaced" is imprecise — that test was retained and repurposed as a live isolation guard
     (`crates/mycelium-l1/src/tests/checkty.rs`, still present). Recorded here per E-1's routing
     (a one-line clarification, no history rewrite).
  **DN-54 status: held at `Accepted` (honest partial — VR-5), NOT stepped to `Enacted`.** Per house
  rule #3 and DN-54 §9's own gate, `Enacted` requires §10's attachment model to be ratified and
  implemented plus the DN-75 residuals (R-1 through R-4) closed; none of that happened in this task.
  What *did* land is real and verified (item 1), but it is a checker-coverage fix for the
  already-Accepted affine `Substrate` construct, not a completion of DN-54's own open design
  question. CHANGELOG / Doc-Index / issues.yaml / docs/api-index reconciled by the integrating
  parent (FLAGged, not edited here). (Append-only; VR-5; G2.)
