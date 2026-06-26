# Design Note DN-38 — The Layered-Lowering Atlas & Generative Sugar (One Seamless Gradient to L0)

| Field | Value |
|---|---|
| **Note** | DN-38 |
| **Status** | **Accepted** (2026-06-26; **ratified by maintainer**) — the **seamless-gradient thesis**, the **lowering law** (every feature lowers to L0 with the same observable meaning; small IL-grammar-checked semantics-preserving passes; kernel never grows — KC-3), the **honest-refinement** rule, the **generative-lowering + inspectable-desugaring** construct set, and the **§8.1 naming** (delegation **`via`** · generative **`derive`** · inspector **`reveal`**) are ratified as the **design direction**. **Accepted ratifies the direction, not every open question:** §8's **architectural** questions — nanopass-separate-passes vs single `elab.rs`; `reveal` v0 vs post-core; round-trip obligation scope; sequencing vs the `[]`-wave/E19-1/DN-35; the `@matured` gap — remain **open** and tracked. No guarantee upgraded (layering + observational-identity + built lowerings `Exact`; framing + construct + naming `Declared` — VR-5). Prior: **Draft** (2026-06-25; direction capture). Append-only; house rule #3. Enacts no code. |
| **Feeds** | the **unifying lowering atlas** the feature DNs hang off — **DN-36** (iteration → `Fix` fold), **DN-37** (objects + the settled sigil scheme + delegation `~>`/`via`), **DN-35** (reclamation), the **generative-lowering** construct (`derive`), and the **inspector** (`reveal`) — naming settled §8.1; intersects the `[]`-grammar wave (**DN-31**, **RFC-0030**, **epic #27**), **E19-1** (#25, value-model `Repr::Seq`/`Bytes`), and the **`@matured` inheritance** epic (#26). |
| **Date** | June 25, 2026 |
| **Decides** | *Nothing normatively* — advisory + design-direction capture. Records (1) the **seamless-gradient thesis** (the L0–L3 "levels" are how the *compiler lowers*, not modes the *programmer declares* — one language, a desugaring gradient, freely intermixing high-sugar and low-explicit forms); (2) the **lowering law** that earns the seamless mix (every feature lowers to L0 with the same observable meaning; each pass is small, IL-grammar-checked, semantics-preserving; the kernel never grows — KC-3); (3) the **honest refinement** (levels invisible/unannotated, but `wild`/`!{io}`/`@matured`/guarantee tags stay explicit and **level-independent**); (4) **generative lowering** (terse-in → explicit, inspectable, content-addressed L0-out — `derive`, `#[derive]`-grade, never Lombok-magic) + the **inspectable-desugaring** affordance (`reveal` shows the real L0 term, with a round-trip discipline for `certified`); (5) the **per-feature Lowering Map** (the living checklist at the heart of the atlas); and (6) the **verification discipline** (differential + hygiene + round-trip, tiered LOW/HIGH per DN-20, round-trip gated by `certified` mode). |
| **Task** | the unifying layered-lowering atlas + generative-sugar direction (the frame the feature DNs hang off) |

> **Posture (transparency rule / VR-5 / G2).** This note synthesises an external prior-art record
> — `research/25-layered-lowering-and-generative-sugar-prior-art-RECORD.md` (primary-source-checked:
> pit-of-success / progressive disclosure / illegal-states-unrepresentable; Racket "languages as
> libraries" + the language tower; nanopass; CompCert; inspectable desugaring — cargo-expand / Racket
> Macro Stepper / Lean `pp.all`; generative lowering GOOD-vs-Lombok; Hylo/Val value-semantic delegation;
> Unison/Nix content-addressed generation; translation-validation) — with the **already-landed**
> Mycelium corpus (RFC-0006 layering + S1–S6 honesty invariants; RFC-0012 ambient = observationally the
> identity; NFR-7 interp≡AOT; the feature DNs) into a design direction. It **enacts nothing**: no
> RFC/ADR/DN status moves, no normative text changes, no code or property test ships. The grounding
> split is load-bearing and held throughout: the **layering + observational-identity invariant + the
> built lowerings** are **`Exact`/built** (cited file/section); the **seamless-gradient framing**, the
> **generative-lowering construct**, the **`reveal` inspector**, and the **naming** are **`Declared`
> design proposals** (their prior-art mechanisms are `Empirical`/`Proven`-at-source, but their Mycelium
> mappings are `Declared`-with-argument — not ratified, no tag upgraded past its basis). Every gap is
> named, not buried (G2).

---

## §1 Thesis — one seamless language, a desugaring *gradient*, not a stack of dialects

Mycelium is **one language**. The L0–L3 "levels" (RFC-0006 §3) name **where the compiler lowers**,
not modes the **programmer declares**. There is no `// L1` pragma, no "drop into L0" ceremony, no
dialect boundary to cross. A program **freely intermixes high-sugar and low-explicit forms in the same
expression** — a `for`-comprehension beside a hand-written `Fix` fold, a `~>`-delegation beside an
explicit forwarder, an inferred type beside a fully-annotated one — because they are **the same program
at different points on the gradient**, sharing one L0 substrate and one value semantics. High and low
**interoperate seamlessly** not by convention but *by construction*: the low form simply *is* the high
form less-sugared, lowered through the same passes.

This is the established corpus shape stated as a user-facing principle. RFC-0006 §3 already fixes it:
"**L2 is defined entirely by elaboration to L1** … there is no L2 semantics independent of [its
desugaring]," and L1 in turn elaborates to L0, the frozen semantic ground truth ("the reference
interpreter executes L0; certificates and differentials speak about L0 values"). The programmer writes
**anywhere on L1–L3** and the meaning is **always** the L0 it lowers to.

The **ergonomic feature superset** — macros, traits, super-traits, delegation (`~>`), generative
lowering, the iteration sugar (DN-36), the object/sigil sugar (DN-37) — is **batteries-included,
packaged into the base language**, and exists to **abstract L0 away** so you rarely hand-write the
bottom. But the house rule is **abstracted, never hidden** (no black boxes, house rule #2; RFC-0006 §4.1
S4 "inspectable elaboration: every L2→L1→L0 step is dumpable and diffable; the elaborator is not a black
box"). L0 is always one `reveal` away (§5).

**Prior art, taken one step further.** This is the "no two-language problem" of Racket's *programmable
programming language* (Felleisen et al., CACM 2018) and *Languages as Libraries* (Tobin-Hochstadt,
St-Amour, Culpepper, Flatt, Felleisen, PLDI 2011): a tower of languages, each a library lowering into
the next, bottoming out at a tiny core (`#%kernel`). Mycelium takes it **further**: the low level is not
a *different* language you drop into (Racket's `#lang` selects a language) — it is the **same** language
*less-sugared*, with one value semantics throughout. There is no `#lang` to choose; the gradient is
intrinsic. Grounding: the *framing* is `Declared`-with-argument (it follows from RFC-0006 §3 + S4, which
are built/Accepted); the prior-art parallel is `Empirical` (primary-source-verified).

## §2 The lowering LAW — what earns the seamless mix

Free intermixing (§1) is *sound* **only because** every surface feature lowers to the same L0 with the
same observable meaning. State this as the organizing law of the atlas:

> **The Lowering Law.** *Every surface feature lowers to L0. Each lowering pass is **small**,
> **IL-grammar-checked**, and **semantics-preserving** (`observe(surface) == observe(lower(surface))`).
> The kernel (L0/L1) **never grows** for ergonomics (KC-3).*

This is one property seen from two seats:

- **The user's seat — a seamless gradient.** Because every high form means exactly its lowered low
  form, mixing them is meaning-preserving by construction. The user never reasons about a level
  boundary because there is no semantic discontinuity at one.
- **The compiler's seat — a verified tower.** The law is the nanopass discipline (Sarkar, Waddell,
  Dybvig, *A Nanopass Infrastructure*, ICFP 2004; Keep & Dybvig, ICFP 2013 — Chez Scheme is 50+
  passes): "a compiler comprised of many single-task passes with a **well-defined intermediate language
  between each pass**," where the framework **auto-generates a verification pass** checking each pass's
  output conforms to its IL grammar. CompCert (Leroy, CACM 2009) is the same shape with a forward-
  simulation proof per pass. Small, grammar-checked, per-pass-checkable lowerings are *exactly* what
  makes "each pass preserves meaning" tractable, and what keeps the small core + its proofs untouched as
  new surface features add *early* passes (KC-3).

Mycelium already states the two halves of this law normatively:

- **Semantics-preservation at the surface** — RFC-0012 (**Enacted**, M-344) makes the ambient "pure
  *surface* elaboration … it never inserts" a representation change; it is **observationally the
  identity**. RFC-0006 §4.1 **S1** (never-silent swap, lexically visible at every layer) and **S4**
  (inspectable elaboration) bind L1–L3. This is the user-facing "the sugar is the identity."
- **interp≡AOT** — **NFR-7** (execution-path equivalence): "the interpreter is the executable reference
  semantics; AOT/JIT output must be observably equivalent to it, validated by the same translation-
  validation machinery used for swaps … two execution paths must never mean two semantics." This is the
  compiler-internal differential that keeps the lowered form honest **down to native code**.

So the seamless gradient and the verified tower are the **same** property. The law is the contract that
the gradient is real rather than a slogan; §6 (the map) lists which features currently satisfy it
(built) versus which assert it (designed), and §8 audits the verification.

## §3 The honest refinement — levels invisible, honesty markers explicit

The seamless gradient would be a transparency *regression* if it erased the audit trail — so it does
not. The refinement: **levels are inherent and unannotated, but a small set of markers stay explicit,
independent of level.** These surface *what the code does*, which never-silent honesty (G2 / VR-5)
requires no matter how sugared the surroundings — **not** "you are now in a lower tier":

| Marker | What it surfaces | Basis | Why level-independent |
|---|---|---|---|
| **`wild { … }`** | FFI / host-execution / capability use | RFC-0028 §4.1 (the `wild`/`@std-sys` gate; ADR-014 unsafe floor) | Touching the OS is a property of the *code*, not of how sugared it is — denied-by-default, lexically marked, at any level (RFC-0006 LR-9 / S6). |
| **`!{io}`** (and the effect set) | declared, bounded effects | RFC-0014 (**Enacted**, M-352/M-353 — budgeted effects, `EffectBudget`) | An effect is observable behavior; the never-silent rule (G2) requires it declared regardless of surrounding sugar. |
| **`@matured`** | the totality / maturation gate | RFC-0017 (**Enacted** — scope-granularity `matured ⟹ total`) | A guarantee that a scope is total is a *claim*, surfaced wherever it is made (§9, VR-5). |
| **guarantee tags** (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`) | the strength of every accuracy/guarantee claim | RFC-0006 §4.1 **S2** ("honest tags surface") + house rule #1 | The tag *is* the claim's honesty; it cannot be sugared away (VR-5). |

The principle, sharply: **levels seamless; capabilities, effects, maturation, and guarantees explicit
and level-independent.** That is precisely what lets you mix high and low freely *without* losing the
audit trail — the markers that matter for honesty are orthogonal to the lowering gradient, so erasing
the gradient costs the user nothing in transparency. Grounding: the four markers are **built** (`wild`,
`!{io}`, `@matured` enacted as cited; guarantee tags are the house lattice) — `Exact`/`Empirical` at
their sources; the *framing them as level-independent* is `Declared`-with-argument (follows from S2/S6 +
the never-silent rule).

## §4 Generative lowering — the mechanism

The atlas's headline new mechanism: a **terse surface** (a few parameters) that **generates an
explicit, inspectable, reusable, content-addressed L0 artifact** — "few-params-in → explicit-lowered-
code-out" (the maintainer's FSVP-delegation design input). The legitimacy rule is drawn straight from
`research/25` Topic 2, and Mycelium's substrate makes the good case structural:

1. **Generation is legitimate only when it produces a nameable, inspectable, reusable L0 term.** The
   GOOD cases — Rust `#[derive]` (real `impl` blocks, recoverable via `cargo expand`), Kotlin `by`
   (compiler generates visible forwarding methods), Go embedding (statically-resolved promotion), Zig
   `comptime` (the generator *is* the same language, run earlier) — all produce a first-class artifact.
   The **anti-pattern is Lombok**: it mutates the compiler AST so "the code you write isn't the code
   that runs," with **no source artifact** and **undebuggable** generated methods. The defect is *not*
   generation (`#[derive]` generates just as much) — it is **opacity**.
2. **The Lombok failure mode is structurally impossible here [EASIER for Mycelium].** Because the
   *only* output channel of a Mycelium lowering is "produce an L0 term" — a real value in the frozen
   core, not a side-effect on an invisible internal AST — the inspectable-reusable artifact exists **by
   construction**. Mycelium gets `#[derive]`-grade transparency for free and **cannot accidentally
   become Lombok**.
3. **Same intent → same identity, for free [EASIER for Mycelium].** Content-address the generated L0
   term (ADR-003, Unison-style: "each definition is identified by a hash of its syntax tree … identical
   structure → same hash even under different names"). Identical generated lowerings **deduplicate by
   hash** — no mutable registry, no interning table, no singleton lifecycle. The "value-semantic
   singleton-in-a-set" *is the substrate*, not a feature to build (Unison; Nix content-addressed
   derivations give the compile-time dual — *early cutoff*: unchanged output hash ⇒ downstream reuse).
4. **Structured errors by default.** A generated lowering that can fail surfaces a **structured
   diagnostic** (RFC-0013, **Enacted**, M-345 — the DynEL-inspired reified, content-addressed, dual
   human/JSON diagnostic with a per-definition `on <ErrorClass> => …` policy), never an opaque panic
   (G2). Generation never silently swallows a malformed-intent error.
5. **Delegation is static, by-value forwarding — never a runtime chain walk.** `research/25` Topic 3:
   Hylo/Val mutable-value-semantics "bans sharing instead of mutation"; delegation is forwarding to a
   *held value* via projections, exclusivity replacing reference identity. Lieberman-style prototype-
   chain delegation (shared mutable prototype, late binding, cyclic `self`-back-reference) is
   **forbidden by construction** under Mycelium's immutable + acyclic + value model — a feature, not a
   gap. This is exactly DN-37 §3.3's `~> ` (frontend sugar → generated forwarders, no kernel change).

**Naming — options to surface for the maintainer** (per the house rule: plain/ergonomic first; coin
only when more mnemonic *and* non-colliding; reserve the fungal lexicon for its own meanings — DN-37 §5
sigil scheme). These are **open** (§8), not decided:

- **the generative construct** — **`derive`** (conventional, instantly legible — Rust/Haskell
  precedent) **vs** a coined **`weave`** (more mnemonic for "weave an explicit artifact from terse
  intent"; verified non-colliding with the lexed keyword set, like DN-37's free-sigil check). Plain-
  first leans `derive`; `weave` is admissible only if judged materially more mnemonic.
- **delegation** — the **`~>`** operator (DN-37 §3.3 / §5, settled glyph) **plus** a keyword spelling
  (**`via`** / **`delegate`**) for the header form.
- **the inspector** — **`reveal`** (on-brand: "reveal the lowered truth") **vs** **`expand`**
  (conventional, cargo-expand precedent). See §5.

Grounding: the GOOD-vs-Lombok rule is `Empirical` (primary-source-verified); the **Mycelium generative-
lowering construct is greenfield, `Declared`** (no surface form exists today — DN-37 §2.2 confirms `~>`,
decorators, and the like are unbuilt); the content-addressed-dedup *win* is `Exact` at the substrate
(ADR-003 is built) but `Declared` as applied to a not-yet-built generator.

## §5 Inspectable desugaring — the transparency affordance

`reveal` (working name) is the atlas's user-facing transparency tool: it shows the **real L0 term the
kernel runs**, the inspector counterpart of §1's "abstracted, never hidden." The quality ladder from
`research/25` Topic 1 runs lossy-text-dump → core-term-print-with-round-trip → stepped/hygiene-aware/
layer-hideable browser; Mycelium should target the **top** because its IR is a real frozen core, not
reconstructed text:

- **View the real artifact, not a text re-render.** `cargo expand`'s own README warns macro expansion
  to text "is a lossy process … a debugging aid only" — the lossiness is hygiene. A text-dump expander
  *silently lies* about binding structure. `reveal` must show the **actual L0 term** (the never-silent
  rule applied to the inspector itself). **[EASIER]**: Mycelium gets real binding structure from the IR;
  Racket's Macro Stepper must reconstruct it.
- **Layer-hideable inspection.** Mirror Racket's *macro hiding* — let the developer stop lowering at any
  layer (view at L2, or fully to L0), stepping pass-by-pass — so the L0–L3 tower is *navigable*. This is
  the §2 nanopass tower made browsable, and the progressive-disclosure DX (Nielsen/NN-g): show the
  detail the user needs, when they need it.
- **Round-trip discipline (for `certified`).** Lean 4's `set_option pp.all` makes the delaborator
  injective so the printed surface **re-elaborates back** to the same core term. Adopt
  `delaborate ∘ lower = id` as an obligation **gated by `certified` mode** (ADR-032 tunable
  certification — `fast` default · `certified` on request): in `certified` mode the inspector itself
  becomes a transparency *check*, not just a viewer (VR-5).

This is the **pit of success** (Mariani/Abrams) + **progressive disclosure** + **make illegal states
unrepresentable** (Minsky) DX in one: the terse path is the correct path, and the expansion **teaches
rather than hides** — `reveal` turns the gradient into a learning surface. Grounding: the inspectors
cited are `Declared`/`Empirical` at source; the `reveal` design + the certified round-trip obligation
are **`Declared` greenfield** for Mycelium.

## §6 The per-feature Lowering Map — the living checklist (the heart of the atlas)

Every surface feature → the layer it is written at → its desugaring/lowering path → its L0 form → build
status, tagged honestly. This is the **atlas the feature DNs hang off**: DN-36 owns the iteration row,
DN-37 owns the object/sigil rows, DN-35 will own the reclamation thread the value-model rows depend on.
The table is **append-only and meant to grow** (each new surface feature adds a row + an early pass —
KC-3, §2).

| Surface feature | Written at | Desugaring / lowering path | L0 form | Build status (honest tag) |
|---|---|---|---|---|
| **Iteration** `for` / `fold` / `map` / `loop` | L2/L3 | `elab_for` → self-recursive tail `Fix` fold (RFC-0007 §4.8; DN-36 §2.1) | `Fix`/`Match`/`App` over a finite acyclic spine | **Built** `Exact` (safe loop); FBIP in-place perf = **`Declared`** roadmap (DN-36 §6) |
| **Objects** `type` / `impl` / `~>` / `@`-decorator | L2 | traits monomorphized + generated forwarders (DN-37 §2.1, §3.3) | `Construct` / `Match` / `Lam` / `App` (RFC-0011 **Enacted r3**) | **Foundation built** `Exact` (`type`+`Match`/`Construct`, traits); emulation menu (`~>`, decorators, super-traits, default bodies) **`Declared`** (DN-37 §2.2) |
| **Sigils** `# $ ? ~ ~>` | L2/L3 surface | ops / annotations / delegation (DN-37 §5) | operators → L0 ops; rest → annotations | **Operators built** `Exact` (RFC-0025 *Proposed*, impl Rust-first/Empirical); `#`/`$`/`?`/`~`/`~>` **reserved**, `Declared` (lands on `[]`-grammar wave #27) |
| **Traits / generics** `trait`/`impl`/`f<T: A+B>` | L2 | dictionary-passing → monomorphization + static dispatch (`mono.rs`) | specialized `Lam`/`App` (no runtime vtable) | **Built** `Empirical` (tested checker); coherence-result tag `Declared`-with-argument (RFC-0019 **Enacted**, not machine-checked) |
| **Effects** `!{io}` | L2/L3 (level-independent marker) | declared + budgeted effect → ledger; `Match`-over-error-sums | L0 `Match` over result/error sums (no new node) | **Built** `Exact` (RFC-0014 **Enacted**, M-352/M-353; `EffectBudget`) |
| **Ambient repr** (omitted paradigm) | L2/L3 | surface elaboration filling an omitted paradigm — **observationally the identity** | unchanged L0 (never inserts a `Swap`) | **Built** `Exact` (RFC-0012 **Enacted**, M-344) |
| **Maturation** `@matured` | L2/L3 (level-independent marker) | scope-granularity resolver → totality gate (`matured ⟹ total`) | L0 unchanged; the gate is a check | **Built** `Exact` (RFC-0017 **Enacted**); §4.1 *top-down inheritance* gap = **`Declared`** (epic #26 / D1) |
| **Value model** `Repr::Seq` / `Repr::Bytes` | L1 value model | new `Repr` cases in the value model (ADR-025/026/027) | extended L0 value model | **Gap** `Declared` (E19-1 / #25 / D2 — ratified, unbuilt) |
| **Generative** `derive` / `weave` (§4) | L2/L3 | terse params → generated, content-addressed L0 artifact | a nameable, hashed L0 term (dedup by hash) | **Greenfield** `Declared` (no surface form today; DN-37 §2.2) |
| **Inspector** `reveal` / `expand` (§5) | tooling | print the L0 term; round-trip in `certified` | (reads L0, emits surface) | **Greenfield** `Declared` (cargo-expand/Lean/Racket precedent `Empirical`) |

Reading the map: the **bottom substrate and the core lowerings are built `Exact`** (iteration safety,
`type`/`Match`/`Construct`, traits/monomorphization, effects, ambient-identity, maturation gate); the
**ergonomic top** (the emulation menu, generative lowering, `reveal`) and the **value-model additions**
are **`Declared`** — designed or greenfield, sequenced behind the grammar (#27), E19-1 (#25), and
reclamation (DN-35) threads (§8). No row is omitted because it is unbuilt (G2); the gap *is* the entry.

## §7 Verification discipline — what keeps "the sugar is the identity" honest

The Lowering Law (§2) is a *claim*; this section is what discharges it rather than asserting it
(`research/25` Topic 5). Each lowering pass carries three checks, tiered LOW/HIGH per **DN-20**:

- **Differential** — `observe(surface) == observe(lower(surface))` on a generated corpus (the sugared
  program and its desugared L0 are two implementations-of-the-same-meaning; Csmith-lineage differential/
  metamorphic testing). This is the per-pass version of the NFR-7 interp≡AOT differential that already
  guards the back end. **[EASIER]**: value semantics make "observable meaning" clean to define — the
  oracle is just *value equality of results*, no aliasing/effect-interleaving to quantify over.
- **Hygiene-no-capture** — generate surface programs, expand, assert no free-variable capture across the
  expansion (Ullrich & de Moura, *Hygienic Macro Expansion for Theorem Proving Languages*,
  arXiv:2001.10490 — capture is a *correctness* bug, not a style nit). Hygiene is itself a testable
  property.
- **Round-trip** — `delaborate ∘ lower = id` as a property (Lean's injective `pp.all`), the §5
  inspector obligation made a test.

The **case count is tiered, never the property** (DN-20 transparency): LOW proptest cases every commit,
HIGH on release — no property is dropped. **`certified` mode (ADR-032)** raises the bar: the round-trip /
**translation-validation witness** (Necula, *Translation Validation for an Optimizing Compiler*, PLDI
2000 — validate *this run*, emit a checkable certificate) is **gated on `certified`**, where the `fast`
default runs the differential corpus. This is how "each pass preserves observable meaning" becomes a
**checked** claim — `Empirical` from the differential corpus, `Proven`-per-run only when the validator
certifies the run (VR-5 — no upgrade past the basis). It is the discipline that makes §2's law *earned*
rather than asserted.

## §8 Open questions (the deliberation agenda)

1. **Construct naming** — `derive` (conventional) vs `weave` (coined, mnemonic); `via` vs `delegate`
   for the delegation keyword; `reveal` vs `expand` for the inspector (§4–§5). Plain-first; coin only on
   a clear mnemonic win (house naming rule).
   - **Resolved (2026-06-25, maintainer delegated the call): the delegation keyword is `via`.** Rationale:
     `delegate` connotes *entrusting authority to a deputy that then acts on your behalf* — an agentive,
     late-binding flavor Mycelium's delegation **does not have** (§line 152 here / DN-37 §3.3: static,
     by-value forwarding to a *held value*, no late binding, no chain walk), so it would over-claim
     dynamism (VR-5 applied to naming). `via` names the **conduit** ("by way of / through"), claiming only
     "obtained by way of" — the literal truth of static forwarding; it is the **prepositional twin of the
     `~>` flow-glyph** (sigil + keyword express one concept, not two), it matches the **Kotlin `by`
     precedent** the design already cites (a preposition, not the pattern-name), and it carries the
     transport-network metaphor without being forced. Discoverability cost (it is not the textbook pattern
     name) is recovered by still *calling* the feature "delegation" everywhere except the keyword.
   - **Resolved (2026-06-25): the generative-lowering construct is `derive`, the inspector is `reveal`.**
     `derive` over the coined `weave` — **plain-first** (house naming rule): `derive` is the conventional
     term (Rust `#[derive]`, Haskell `deriving`), instantly discoverable, and the coinage cleared no
     mnemonic bar high enough to justify departing from it. `reveal` over `expand` — `expand` overloads
     **macro-expansion** (`cargo expand`, Lean `pp.all`), whereas the construct shows the *real, already-
     lowered L0 term* (not a re-expansion step); `reveal` names that disclosure without the collision and
     reinforces the **abstracted-never-hidden** thesis (§2: the L0 form is always `reveal`-able). With
     this, **§8.1 Construct naming is fully settled** (delegation `via`; generative `derive`; inspector
     `reveal`).
2. **Nanopass vs single elaboration** — how much of the gradient is implemented as *separate*
   IL-grammar-checked passes (the full nanopass tower, §2) versus the **single `elab.rs` elaboration**
   today? Nanopass buys per-pass checkability and failure isolation; one elaboration is simpler. Where
   on that spectrum to sit is a real architectural choice.
3. **`reveal` sequencing** — ship a v0 inspector (even a text dump with the lossiness caveat surfaced),
   or defer the layer-hideable/round-trip inspector to post-core? A lossy v0 must be **labelled** lossy
   (never-silent).
4. **Round-trip obligation scope** — is `delaborate ∘ lower = id` `certified`-only (§5/§7), or a
   universal target? Certified-only keeps the `fast` path cheap; universal is stronger but costlier.
5. **Sequencing vs the dependent threads** — the value-model rows (E19-1 / #25) gate generative-lowering
   over sequences; the sigil/grammar rows ride the `[]`-grammar wave (#27); FBIP perf rides reclamation
   (DN-35) + DN-36 §6. State the order: substrate (E19-1, reclamation) → grammar (#27) → ergonomic top
   (generative lowering, `reveal`).
6. **`@matured` inheritance gap** (epic #26 / D1) — the §4.1 top-down inheritance is unbuilt; it affects
   how the maturation marker composes across the gradient (§3/§6).

## §9 Guarantee posture (VR-5) + Definition of Done

**Grounding posture (held throughout):**

- **Built / `Exact`** — the L0–L3 layering (RFC-0006 §3, S1–S6 **Accepted**); the observational-identity
  invariant (RFC-0012 **Enacted**); interp≡AOT (NFR-7); and the built lowerings in the §6 map (iteration
  safety, `type`/`Match`/`Construct` RFC-0011 **Enacted r3**, effects RFC-0014 **Enacted**, ambient
  RFC-0012 **Enacted**, maturation RFC-0017 **Enacted**). Traits/coherence are `Empirical` (tested
  checker) with the coherence-result `Declared`-with-argument (RFC-0019).
- **Designed / `Declared`** — the **seamless-gradient framing** (§1), the **lowering law as an
  organizing principle** (§2), the **level-independent-marker refinement** (§3), the **generative-
  lowering construct** (`derive`/`weave`, §4), the **`reveal` inspector** + certified round-trip (§5),
  and the **naming** (§8). Their prior-art mechanisms (nanopass, CompCert, `#[derive]`, Kotlin `by`,
  Hylo MVS, Unison/Nix content-addressing, Racket Macro Stepper, Lean `pp.all`, translation validation)
  are **`Empirical`/`Proven`-at-source** (primary-source-verified in `research/25`), but the **Mycelium
  mappings are `Declared`-with-argument** — cited to their basis, **not** ratified, **no tag upgraded
  past that basis** (VR-5).
- **Gaps** — the value-model rows (E19-1 / #25) and generative-lowering / `reveal` are greenfield;
  named, not buried (G2).

**Definition of Done (the gate for Draft → Accepted).** This note is `Accepted` when the maintainer
ratifies: **(a)** the **lowering law** (§2) as the organizing invariant of the atlas; **(b)** the
**seamless-gradient framing** + the **level-invisible / honesty-explicit refinement** (§1, §3); **(c)**
the **generative-lowering construct + `reveal`** direction and the **naming** (`derive` vs `weave`; the
delegation keyword; `reveal` vs `expand` — §4, §5, §8.1); and **(d)** the **verification discipline**
(§7 — differential + hygiene + round-trip, tiered, with the certified-mode witness). Ratification moves
Draft → Accepted (a legal forward step, house rule #3) and feeds: a generative-lowering surface RFC
(riding DN-31/RFC-0030/epic #27 for its grammar), an inspector (`reveal`) RFC, and the verification
harness work behind DN-20's tiers. **Still enacts no code** — the atlas is the deliverable; the build is
the forward epic, with the runtime/elaboration as the sound base throughout. Append-only; VR-5; G2;
nothing here moves another doc's status.

## §10 Relation to the corpus & grounding

- **Corpus (built / Accepted basis):** RFC-0006 §3 (L0–L3 layering; "L2 defined entirely by elaboration
  to L1") + §4.1 S1–S6 (never-silent swap, honest tags, content-addressed identity, **S4** inspectable
  elaboration, explicit partiality, S6 AI-independence) + LR-9; RFC-0012 (**Enacted** — ambient =
  observationally the identity); RFC-0007 §4.8 (`Fix` fold, `for` sugar); RFC-0011 (**Enacted r3** —
  `Construct` + flat `Match`); RFC-0013 (**Enacted** — structured/DynEL-inspired diagnostics);
  RFC-0014 (**Enacted** — declared/budgeted effects); RFC-0017 (**Enacted** — `@matured` scope gate);
  RFC-0019 (**Enacted** — traits/coherence/monomorphization); RFC-0025 (**Proposed** — operator sugar,
  frontend-only); RFC-0028 (`wild`/`@std-sys` FFI gate); ADR-003 (content-addressed identity, Unison-
  style); ADR-032 (tunable certification — `fast`/`certified`); NFR-7 (interp≡AOT); KC-3 (small kernel
  never grows); G2 (never-silent); VR-5 (downgrade-don't-overclaim); DN-20 (the LOW/HIGH test tiers).
- **Feature DNs this atlas unifies:** **DN-36** (iteration → `Fix` fold; FBIP perf roadmap), **DN-37**
  (objects + sigil scheme + delegation `~>`), and the future **DN-35** (reclamation) — each owns its
  row(s) in the §6 map; this note is the frame they hang off.
- **External prior art (`Empirical`/`Proven` at source; transfer to Mycelium `Declared`):** Mariani /
  Abrams (pit of success); Nielsen / NN-g (progressive disclosure); Minsky (make illegal states
  unrepresentable); Tobin-Hochstadt, St-Amour, Culpepper, Flatt, Felleisen, *Languages as Libraries*
  (PLDI 2011) + Felleisen et al., *A Programmable Programming Language* (CACM 2018); Sarkar, Waddell,
  Dybvig, *A Nanopass Infrastructure* (ICFP 2004) + Keep & Dybvig (ICFP 2013); Leroy, *CompCert* (CACM
  2009); dtolnay, *cargo-expand* (the lossiness caveat); Culpepper & Felleisen, *A Stepper for Scheme
  Macros* (2006) + the Racket Macro Debugger (macro hiding); Lean 4 `set_option pp.all` (injective
  delaborator / round-trip); Ullrich & de Moura, *Hygienic Macro Expansion …* (arXiv:2001.10490); Rust
  `#[derive]`, Kotlin `by`, Go embedding, Zig `comptime` (GOOD generative lowering) vs **Lombok** (the
  opaque anti-pattern); Racordon, Shabalin, Zheng, Abrahams, Saeta, *Native Implementation of Mutable
  Value Semantics* (arXiv:2106.12678) + Hylo/Val (value-semantic delegation; projections/subscripts);
  Lieberman, *Using Prototypical Objects …* (OOPSLA 1986 — the contrast case); Unison (content-addressed
  code = hash of AST + deps; dedup); Nix content-addressed derivations (early cutoff); Necula,
  *Translation Validation for an Optimizing Compiler* (PLDI 2000). Full URLs in
  `research/25-layered-lowering-and-generative-sugar-prior-art-RECORD.md`. Per the api-index caveat,
  source is ground truth; this note synthesises the records.

---

## Meta — changelog

- **2026-06-25 — Created (Draft, advisory) — authored.** Synthesises
  `research/25-layered-lowering-and-generative-sugar-prior-art-RECORD.md` (external prior art, primary-
  source-checked) with the landed corpus (RFC-0006 layering + S1–S6; RFC-0012 ambient = observationally
  the identity, **Enacted**; NFR-7 interp≡AOT; the feature DNs) into **the unifying layered-lowering
  atlas**. Records: **(1)** the **seamless-gradient thesis** — one language, a desugaring *gradient* not
  a stack of dialects; the L0–L3 "levels" are how the *compiler lowers*, not modes the *programmer
  declares*; a program freely intermixes high-sugar and low-explicit forms because they are the **same
  program at different points on the gradient** (RFC-0006 §3: "L2 defined entirely by elaboration to
  L1"); the ergonomic feature superset is batteries-included and **abstracts L0 away — abstracted, never
  hidden** (S4); Racket "languages as libraries"/programmable-programming-language taken further (the low
  level is the *same* language less-sugared, not a different one). **(2)** The **lowering law** that
  earns the seamless mix — *every feature lowers to L0 with the same observable meaning; each pass is
  small, IL-grammar-checked, semantics-preserving; the kernel never grows (KC-3)* — the user-facing
  "seamless gradient" and the compiler-internal "verified tower" (nanopass — Sarkar/Dybvig; CompCert) as
  one property from two seats, grounded in RFC-0012 observational-identity + S1/S4 + NFR-7. **(3)** The
  **honest refinement** — levels invisible/unannotated, but `wild` (RFC-0028), `!{io}` (RFC-0014),
  `@matured` (RFC-0017), and guarantee tags (S2) stay explicit and **level-independent** — they surface
  *what the code does*, not "you're now in a lower tier"; so levels seamless, capabilities/effects/
  guarantees explicit, audit trail preserved across free mixing. **(4)** **Generative lowering** — terse-
  params-in → explicit, inspectable, reusable, **content-addressed** L0-out (`#[derive]`/Kotlin `by`/Go
  embedding/Zig comptime GOOD; **Lombok** opaque-magic BAD — structurally impossible here because the
  only output channel is "produce an L0 term" [EASIER]); same-intent→same-identity for free via ADR-003
  content-addressing (Unison/Nix — dedup by hash, no mutable registry; the value-semantic singleton-in-a-
  set is the substrate); structured Dynel/RFC-0013 errors by default; delegation = static by-value
  forwarding (`~>`, Hylo MVS), never a prototype chain. **(5)** **Inspectable desugaring** — `reveal`
  shows the *real L0 term the kernel runs* (not a lossy text re-render — cargo-expand's own caveat),
  layer-hideable (Racket macro-hiding), with a `delaborate ∘ lower = id` round-trip gated by `certified`
  mode (ADR-032; Lean `pp.all`); the pit-of-success + progressive-disclosure DX where the terse path is
  the correct path and the expansion teaches. **(6)** The **per-feature Lowering Map** (the living,
  append-only checklist at the heart of the atlas) — iteration, objects/`~>`, sigils, traits, effects,
  ambient, maturation, value model, generative, inspector — each row: surface feature → layer → lowering
  path → L0 form → honest build status (built `Exact` / designed `Declared` / greenfield); the atlas the
  feature DNs (DN-36/37, future DN-35) hang off. **(7)** The **verification discipline** — per-pass
  differential (`observe(surface) == observe(lower(surface))`) + hygiene-no-capture + round-trip, tiered
  LOW/HIGH per DN-20 (case count tiered, never the property), with the round-trip / translation-
  validation witness (Necula) gated on `certified` (ADR-032). **(8)** Open questions — construct naming
  **resolved §8.1** (delegation `via`; generative `derive`; inspector `reveal`); still open: nanopass
  separate passes vs single `elab.rs` elaboration; `reveal` v0 vs post-core; round-trip obligation scope
  (certified-only?); sequencing vs the `[]`-grammar wave (#27), E19-1 (#25), reclamation (DN-35);
  `@matured` inheritance gap (#26). **(9)** Guarantee posture — layering + observational-identity + built lowerings `Exact`; the
  seamless-gradient framing + generative-lowering construct + `reveal` + naming `Declared` (prior-art
  mechanisms `Empirical`/`Proven`-at-source, Mycelium mappings `Declared`-with-argument, no tag upgraded
  past basis — VR-5). DoD = maintainer ratifies the law + the framing + the naming + the verification
  discipline (Draft→Accepted). **Enacts nothing; moves no status; changes no normative text.** CHANGELOG
  / Doc-Index / issues.yaml / docs/api-index owned by the integrating parent. (Append-only; VR-5; G2.)
- **Ratified Draft → Accepted (2026-06-26).** The maintainer ratified the seamless-gradient thesis, the
  lowering law, the honest-refinement rule, the generative-lowering + inspectable-desugaring construct
  set, and the §8.1 naming (`via` / `derive` / `reveal`) as the design direction. The status move accepts
  the *direction only* — it enacts no code and upgrades no guarantee (layering + built lowerings stay
  `Exact`; framing + construct + naming stay `Declared` — VR-5). §8's **architectural** questions
  (nanopass-vs-single-elaboration; `reveal` v0 sequencing; round-trip scope; wave sequencing; the
  `@matured` gap) **remain open** (Accepted ≠ all questions closed).
