# Design Note DN-54 — User-Extensible Generative-Lowering Surface & Its Checker

| Field | Value |
|---|---|
| **Note** | DN-54 |
| **Status** | **Proposed** (2026-06-27) — design DN; captures the extension-definition surface, inspectability-by-construction argument, the compiler checker, the verification discipline, and the KC-3 invariant for **user-extensible generative lowering** (`derive`). Enacts no code; moves no other decision's status (house rule #3). Ratified design direction: DN-38 changelog 2026-06-27 ("extensibility RESOLVED"). Gated on RFC-0037 grammar wave (DN-31) and the §6 verification harness (DN-38 §7). |
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
