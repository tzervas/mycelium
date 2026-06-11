# Changelog

All notable changes to this project are recorded here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Dates are ISO-8601.

This project is in the **design phase**; "changes" here are to the documentation
corpus, not released software. Versioning will begin when the kernel does.

## [Unreleased]

### Added (DN-03 — lexicon amendment; resolves ADR-012 §7.5/§7.6)
- **DN-03** (Resolved): amends DN-02 (append-only) through the three-test gate — **adopt**
  `consume` and `grow` (Surface), **decline** `embody` (inherent methods keep the conventional
  `impl`), **reserve** `for` (the RFC-0007 §4.8 bounded-iteration keyword). Ratifies the
  **canonical-name + one-alias rule** (each themed term: one canonical name + at most one alias —
  *zero* when the canonical is already short — same content hash; ADR-003 makes it free; with the
  abbreviation-vs-synonym precision so the "projection" framing isn't overstated). Ratifies the
  Runtime-tier names against RFC-0008 §4.5's grounded meanings: `anas`→**`fuse`** (RT6 is genuine
  merge, so `weave` was rejected on T-map — woven threads stay distinct), `cmn`→**`mesh`**,
  `myco`→**`graft`** (resolving the collision with the language family name),
  `sclrt`→**`cyst`/`encyst`** (encystment = the dormant, resistant, *resumable* form — the most
  behaviorally-accurate T-map in the set); `xloc`/`dimorph`/`reclaim` kept;
  `hypha`/`forage`/`rhizomorph` carry **no** alias. `reclaim` scope clarified (runtime units,
  never memory). Runtime vocabulary stays reserved-not-active. Lexicon Reference, Example-Programs
  note, and RFC-0008 §4.5 table updated to the ratified names; Doc-Index gains the DN-03 row.

### Added (ADR-013 — `spore` is the deployable unit; resolves ADR-012 §7.4)
- **ADR-013** (Accepted, maintainer deliberation 2026-06-10): `spore` = the
  **content-addressed deployable unit** — a hash-identified DAG of code (ADR-003 definitions,
  ship-by-hash per T4.3), values (with `Meta` intact), the RFC-0003 §6 **reconstruction
  manifest** as one digest-referenced component, and artifact metadata. The narrow ratified
  sense is the **degenerate case**: `spore(v)` constructs the single-value spore (the manifest
  for `v`); the schema name `reconstruction-manifest` is unchanged. Grounded in T4.3/T4.4
  (Nix/OCI/Wasm/Unison convergence on content-addressed artifact DAGs).
- **RFC-0003 → Accepted (r2)**: §6 scope note only — manifest contents, schema, and guarantees
  unchanged. **RFC-0008 R8-Q5** resolved at the scope level (schema/signing/germination contract
  remain the R2 implementation stage's obligation). Lexicon-Reference `spore` flag resolved;
  ADR index gains 012/013 rows.

### Changed (RFC-0007 r2 — bounded iteration; resolves ADR-012 §7.2)
- **RFC-0007 §4.8 (new, r2)**: bounded iteration as **elaboration-defined sugar** over
  structural recursion — no new kernel node. Normative content = the desugaring to a synthesized
  self-recursive helper over *linearly recursive* (nil/cons-shaped) data, classified `Total` by
  the existing §4.5 checker with zero extension (bounded **by construction**: values are finite
  and acyclic). Provisional spelling A — `for x in xs, acc = init => body` — ships in the
  non-normative prototype grammar (`for` reserved, recorded in DN-03); named-args `fold` is the
  planned L2 library form; the ratified spelling is **KC-2-evidence-gated** (T3.6).
  `while`/`loop`/`break`/`continue`/`return` stay excluded and **unreserved**, with *teaching
  diagnostics* where they already error (parse-level juxtaposition + check-level unknown name).
- **Prototype** (`crates/mycelium-l1`): `for` through the whole pipeline — lexer/parser
  (+ teaching diagnostics), T-For with explicit linear-shape refusals, totality (a `for` adds no
  recursion), an **iterative** spine-walk evaluator (long folds cost fuel, never host stack),
  elaboration `Residual` (Fix is outside the evaluation-complete fragment); EBNF + conformance
  corpus (`accept/11`, `reject/08`). **KC-2**: tasks kc2-09 (`for`) / kc2-10 (explicit
  recursion) form the runnable iteration-spelling ablation pair. 44 crate tests green.

### Added (RFC-0008 + Research Pass 4 — the Runtime tier, grounded)
- **Research Record 04** (`research/04-runtime-concurrency-RECORD.md`; findings **T4.1–T4.6**):
  the fourth research pass, grounding the Runtime tier ADR-012 §7.3 flagged as aspirational —
  concurrency units & structured lifetimes (Erlang isolation, nurseries, Kahn/LVars determinism,
  CakeML clocked-semantics extension), state merge & meshes (CRDT convergence, session types,
  epidemic protocols), mobility & placement (Unison ship-by-hash, the Legion
  placement-is-never-semantics separation, Reactive-Streams backpressure, work-stealing bounds
  with side-conditions), durability (CRIU's exception catalogue vs durable-execution's
  determinism requirement; Nix/OCI/Wasm content-addressed artifacts), failure & supervision
  (OTP, FLP, φ-accrual, Waldo et al.), and mode switching (verified deoptimization, CoreJIT).
  Primary-source verified with per-target uncertainty registers; three explicit novelty flags
  (no found precedent: determinism-gated checkpointability; learned-placement-as-inspectable-
  policy; per-value guarantee tags across a distribution boundary).
- **RFC-0008 — Runtime & Concurrency Execution Model** (Draft): the runtime model the Runtime
  vocabulary presupposed, built on Pass 4. **RT1–RT7 runtime invariants** extend S1–S6 to
  concurrency/distribution: values move & state is never shared (RT1); the deterministic
  fragment is the default with *sequential reference semantics* — NFR-7 extends to concurrency
  via the M-210 checker (RT2); nondeterminism is reified as RFC-0005 policies — placement
  becomes the **third site** of the one selection mechanism (RT3); partial failure is explicit,
  distribution transparency forbidden (RT4); runtime guarantees (delivery/convergence/failure
  suspicion) are tagged on the same lattice with `ProbabilityBound`s (RT5); fusion is lawful
  semilattice merge — payload joins, guarantee meets (RT6); runtime lifetimes are structured —
  *a leaked task is not expressible*, extending LR-9 (RT7). RFC-0004's per-node model is
  extended, not changed; the Runtime vocabulary is grounded (§4.5 operational-meaning table)
  but stays **reserved, not active syntax**, pending DN-03 + implementation RFCs. The `spore`
  scope reconciliation (ADR-012 §7.4) and name ratification are deliberately left to the
  RFC-0003 revision and DN-03 respectively. Indexes updated (`docs/rfcs/README.md`,
  `docs/Doc-Index.md`, Lexicon-Reference status notes).

### Added (L1 execution: evaluator, elaboration, three-way differential)
- **L1 fuel-guarded evaluator** (`crates/mycelium-l1/src/eval.rs`; RFC-0007 §4.6): a big-step
  environment machine mirroring M-110's contract — CakeML-style clocked semantics (explicit
  `FuelExhausted`, never a hang; T3.4), dispatching through the *same* trusted prim registry and
  certified binary↔ternary swap engine as the L0 paths (NFR-7). Runs the full checked surface
  (data values, flat `match`, recursion); the stage-0 **dynamic guarantee-index check**
  (RFC-0007 §4.3): asserting `@ g` stronger than a value's tag is an explicit
  `GuaranteeTooWeak` — an annotation may only weaken, never upgrade (VR-5). A separate explicit
  recursion-**depth guard** (`DepthExceeded`) keeps deep recursion an error, never a host stack
  overflow. Checker-unreachable states are explicit `Stuck` errors, never panics (S5/G2).
- **Elaboration to L0 on the evaluation-complete fragment** (`crates/mycelium-l1/src/elab.rs`;
  RFC-0007 §4.6): acyclic calls inline (CBV order preserved via `Let` bindings); bodies must
  reduce to `Const/Var/Let/Op/Swap` residue; recursion (`Fix`), `match`/`if`, data construction,
  and dynamic guarantee indices are explicit **`Residual` refusals — never a partial artifact**.
  Includes the shared surface→kernel bridge (literals, repr resolution) and the documented v0
  **policy-name reference** stand-in (deterministic, domain-separated; honest about deferring
  RFC-0005 name→policy-object binding) shared by both execution paths.
- **The RFC-0007 §4.6 differential** (`crates/mycelium-l1/tests/differential.rs`; NFR-7): on a
  10-program fragment corpus, **L1-eval ↔ elaborate→L0-interp ↔ AOT** agree on the observable
  (`repr + payload + guarantee`), with every agreeing pair validated through the **M-210 shared
  TV checker** (`ObservationalEquiv`) and a control asserting the checker rejects a genuinely
  divergent pair. Outside-the-fragment behavior is pinned too: elaboration refuses (`Residual`)
  while L1-eval runs — including a `Total`-classified structural recursion that terminates and a
  `Partial` one that exhausts fuel explicitly. 31 crate tests; `just check` green.

### Added (KC-2 harness)
- **KC-2 LLM-leverage harness** (M-002 structural deliverable; Foundation §6 P0.2; SC-5b; G10):
  `experiments/mycelium_experiments/kc2/` — the **fixed 8-task benchmark** (minimal Mycelium
  surface fragment vs a **Python-embedded DSL baseline**, both arms carrying checked reference
  solutions that prove the benchmark well-posed), the `myc-check` CLI oracle
  (`crates/mycelium-l1/src/bin/myc-check.rs`: parse / typecheck / task-signature conformance with
  distinct exit codes — no AI in the judging loop, S6), and the generate→check→feedback harness
  measuring **syntactic validity**, **first-attempt type-check pass rate** (the SC-5b number),
  and **edit-to-fix iterations**. *Running* the experiment remains blocked on LLM API access
  (the documented M-002 external blocker); the report hard-codes
  `verdict: not established` — never pre-written (VR-5). Baseline-arm execution is in-process
  `exec` and documented as requiring a disposable sandbox for untrusted model output. 8 pytest
  tests; `just check` green.

### Added (L1 static analysis + lexicon integration)
- **L1 typechecker + structural totality checker** (`crates/mycelium-l1`, RFC-0007 §4.4/§4.5):
  the v0 monomorphic typechecker over the data registry (declarations-as-registry), exhaustiveness
  checked (W7, never assumed), representation-typed literals, generics/`spore`/`wild` as explicit
  refusals; a Foetus-style structural-descent totality classifier whose verdict gates `matured`
  (mutual recursion stays Partial — R7-Q3). 8 tests; clippy clean.
- **Lexicon integration & architect review** (ADR-012 §7; `Lexicon-Reference.md`,
  `Example-Programs-Reference.md`, `Doc-Index.md`): verified the maintainer's three new lexicon
  documents against the corpus and integrated them. **Applied:** de-conflicted the lexicon
  "L1/L2/L3" tier labels (which collided with RFC-0006's language layers L0–L3) → renamed
  **Surface / Runtime / Formal**; fixed example bracket typos; added grounding notes. **Flagged for
  the maintainer (ADR-012 §7):** the Runtime tier (`hyph`/`anas`/`xloc`/…) is an *aspirational,
  ungrounded* concurrency/distribution model needing a Runtime RFC (RFC-0008) + research Pass-4 and
  reconciliation with RFC-0004; imperative `loop`/`while` contradicts the functional core
  (RFC-0007 §6); `spore` scope drifted from RFC-0003's reconstruction manifest; new Surface terms
  (`consume`/`embody`/`grow`) need a DN-02 amendment through the three-test gate (`embody` weakest);
  several short forms (`sclrt`/`cmn`/`anas`/`myco`) recommended for refinement; example
  bound-kind/partiality corrections. No contradictions found with ADR-010/011, the guarantee
  lattice, or content-addressing.

### Changed (RFC-0006 language-layer requirements)
- **RFC-0006 → r3 (Draft): two foundational language requirements** (maintainer direction;
  grounded in T3.5). **S6 self-sufficiency / AI-independence** — Mycelium is a complete software-
  engineering language whose parser/checker/elaborator/interpreter/AOT path are ordinary
  deterministic software runnable with **no AI/LLM in the loop**; models are an optional
  co-authoring convenience, never a runtime/compile-time/semantic dependency (remove every model
  and the language still builds, checks, runs, and reproduces bit-for-bit). This bounds KC-2: it
  can only choose the L3 surface, never make the language *need* a model. **LR-9 memory safety by
  construction** — Rust-grade safety *outcomes* without the borrow checker: value semantics
  removes use-after-free/data-races/double-free from the model, the language exposes no manual
  alloc/free (automatic deterministic reclamation — Perceus + region inference), the sole leak
  vector (external resources) is closed by the affine `Resource` kind, and any unsafe op is
  denied-by-default + lexically marked — *in safe Mycelium a memory leak is not expressible*. New
  open question Q8 (reclamation mechanism, cycle handling, `unsafe` spelling).

### Added
- **L1 grammar infrastructure + parser prototype** (`docs/spec/grammar/`, `scripts/checks/grammar.sh`,
  `crates/mycelium-l1`; RFC-0006 §4.3; **non-normative until RFC-0006 ratifies**): the WebAssembly-spec
  pattern (T3.1-B) made real. **`docs/spec/grammar/mycelium.ebnf`** — the normative v0 surface grammar
  in W3C notation (not ISO 14977), over the ratified DN-02 vocabulary (`colony`, `use`, `type`,
  `trait`, `fn`, `matured`, `let`/`in`, `if`, `match`, `swap`, `wild`, `spore`, `Substrate{…}`, the
  `T @ Strength` honesty index, representation-typed literals). **A conformance corpus** of 10
  `accept/` + 7 `reject/` `.myc` programs, each with an explanatory header — the corpus is the ground
  truth, not any single parser. **`grammar.sh`** (wired into `just check`/CI) structurally validates
  the artifacts; **`mycelium-l1`** is the real parser gate — a hand-written, dependency-free lexer +
  recursive-descent parser producing an inspectable AST, with `tests/conformance.rs` asserting every
  `accept/` parses and every `reject/` fails with an **explicit `ParseError` (never a panic, never a
  silent accept** — S5/G2). The lexer disambiguates the one tricky token (`<` opening a ternary
  literal vs a type-arg list) by lookahead; a malformed ternary literal is an explicit error. First
  increment of the L1 track (RFC-0006 §3) — typechecker, Maranget match compiler, structural totality
  checker, and L0 elaboration land next.
- **DN-02 (Resolved) — Fungal Lexicon & Reserved-Word Set** (`docs/notes/DN-02-Fungal-Lexicon-and-Reserved-Words.md`;
  feeds RFC-0006 §4.3): the surface vocabulary of Mycelium-the-language, drafted then **ratified by
  the maintainer** the same day. Codifies the **naming law** as a three-test gate (T-map fidelity /
  T-illuminate teaching-value / T-learn dual-readability) — *theme where the fungal metaphor is
  accurate and illuminating; keep conventional where a borrowed term is clearer to learn and read*.
  Ratified themed set: `colony` = module, `network` = the content-addressed dependency web,
  `substrate` = the affine external-resource kind, `spore` = reconstruction manifest (schema stays
  `reconstruction-manifest`), `matured` = promoted stable/AOT component, `wild` = the
  denied-by-default unsafe block. Ratified conventional: `let`, `fn`, `type`, `trait`, `match`,
  `if`, `swap` (a native corpus term), `use`, the guarantee tags; guarantee annotation `T @ Exact`.
  Literals universal-until-elaboration (no cross-family defaulting). Language name = **Mycelium**
  (shared). Status **Resolved** — the set is now frozen into the grammar artifacts.
- **Research Pass 3 — language-layer targets T3.1–T3.6** (`research/03-language-layer-RECORD.md`;
  grounds RFC-0006 Q1–Q6): four parallel primary-source deep-dives. Headlines: every surveyed
  kernel (GHC Core, Lean, Coq, Unison) keeps ~10–16 expression nodes with **data declarations in
  a registry/environment layer** and Unison gives the cycle-hashing recipe (T3.1); the guarantee
  lattice is formally an **integrity lattice** — silent upgrade = IFC's *endorsement*, gated here
  by a checked certificate — and graded coeffects (Granule-style) subsume flat labels, with
  refinements reserved for certificate side-conditions (T3.2); GHC levity polymorphism's two
  restrictions + monomorphization give the LR-5 restriction set (T3.3); divergence-only effect
  tracking (Koka's `div`, degenerate) + Lean's `partial`-opaque split + CakeML clocked semantics
  settle Q4/LR-4 (T3.4); ownership/borrowing confirmed **not applicable** to value semantics
  (Hylo/Swift), linearity deferred to a reserved affine `Resource` hook (T3.5); and the measured
  LLM evidence (MultiPL-E/T, MTOB, SynCode, grammar-aligned-decoding distortion) yields a
  five-condition KC-2 design with an explicit falsification threshold (T3.6). Honest-uncertainty
  register included; two pieces flagged **novel with no found precedent** (grading + runtime
  certificates; totality gating AOT promotion). **RFC-0006 revised to r2 (still Draft)**: §8
  positions per question, new Q7; §4.2 postures updated.
- **RFC-0006 (Draft) — Surface Language, Grammar & Term-Language Layering**
  (`docs/rfcs/RFC-0006-Surface-Language-and-Term-Layering.md`; SPEC §10.2's deferred "later RFC"):
  the deliberation artifact that nails down the language architecture *before* implementation
  accretes a de-facto one. Fixes now: the **L0–L3 layering** (Core IR → kernel calculus → surface
  term language → KC-2-gated projection layer; only L0/L1 trusted — KC-3), the **syntactic honesty
  invariants S1–S5** (never-silent swap stays lexically visible through every layer; guarantee
  tags are part of every binding's observable interface; content-addressed identity; inspectable
  elaboration; explicit partiality), the **capability targets LR-1…LR-8** ("Rust-class and beyond"
  made checkable: ADTs, coherent traits, content-addressed modules, totality-postured recursion,
  plus the beyond-Rust core — Repr polymorphism and guarantee-indexed types; ownership/borrowing
  flagged as likely-not-applicable to a value-semantics substrate), and the **grammar/spec
  discipline** (EBNF + machine-readable grammar artifacts + conformance corpus, mirroring the
  schema pattern). Defers exactly one thing, deliberately: the concrete L3 syntax, which the
  corpus already gates on the KC-2 experiment (M-002; RR-3). Status **Draft** — ratification is a
  maintainer decision. Indexed in `docs/rfcs/README.md`, `docs/Doc-Index.md`, SPEC §10.2.
- **Selection-policy language + mandatory EXPLAIN + site wiring** (`mycelium-select` — a new
  crate — plus the `mycelium-lsp` EXPLAIN channel, **M-220/M-221/M-222**, Phase 2; RFC-0005;
  ADR-006; SC-5): realizes RFC-0005 §2's decision verbatim. **M-220:** `SelectionPolicy` — an
  ordered decision table (`Predicate` over queryable `Meta`: dtype, guarantee, ε bounds, sparsity —
  *exact* metadata, never sampled estimates) over a finite `Candidate` set (`Repr` | `PackScheme`),
  with an explicit `CostModel` (cost = weight × storage **bits**, a real declared unit) and a
  mandatory default arm — total and terminating *by construction* (validated constructor; wire
  forms re-validated on deserialize); deterministic (first-match precedence; `Cheapest` ties break
  to lowest index); **content-addressed** (`policy_ref()` = hash of the canonical serialization —
  RFC-0005 §3); first-class deterministic overrides. **M-221:** every selection emits a
  serializable `Explanation` `{policy ref, inputs considered, cost of every candidate, matched
  rule, chosen, override state}`; `explain(policy, inputs)` is total and deterministic; the
  `mycelium-lsp` facade surfaces it as the fifth artifact kind (`analyze_with(node, &PolicyRegistry)`
  re-derives the trace at each resolvable swap site and raises a `policy-divergence` warning when
  the node's target disagrees with the policy's choice — surfaced, never silent). **M-222:** one
  mechanism, two sites — `select_swap_target`/`select_packing` are thin adapters over the single
  `select` (a wrong-kind candidate at a site is an explicit refusal); the wiring test drives an
  auto-selected target through the real interpreter + `CertifiedSwapEngine` and the result records
  `Meta.policy_used = PolicyRef` (the packing site is consumed by E2-7/M-250). 15 new tests across
  policy semantics, EXPLAIN, LSP surfacing, and the swap-site wiring.
- **KC-4 cert-overhead measurement + SC-3 global exit** (`xtask kc4` +
  `mycelium-cert/tests/sc3.rs`, **M-212**, Phase 2; Foundation KC-4; SC-3; RFC-0002 §2):
  `cargo run --release -p xtask -- kc4` times every implemented swap kind and its M-210
  certificate check (no bench dependency; refuses debug builds — their numbers would be dishonest
  to record). **Measured 2026-06-10** (containerized runner, indicative): bijective check ≈1.6–1.7 µs
  (~1.3× its ~1.3 µs swap — it re-derives the swap), bounded `Dense{768}` check ≈2.0 µs (~0.13× its
  ~16 µs swap), observational pair ≈10 ns. Honest verdict: per-swap checking costs the same order
  as the swap itself — the KC-4 downgrade path is **not triggered on this evidence**; a *ratified*
  numeric budget remains a pending maintainer decision (recorded in `phase-2.md` §6.7, not
  pre-written as "within budget"). The SC-3 global test pins the whole surface: every implemented
  legal-pair row emits a certificate that validates through the one checker, and every
  rejected/unimplemented row is an explicit error — never silent, anywhere.
- **First Bounded/lossy swap — Dense `F32 → BF16`** (`mycelium-cert::dense`, **M-211**, Phase 2;
  RFC-0002 §3/§5; ADR-010 §1): establishes the split regime (ADR-002) alongside the bijective
  binary↔ternary class. `dense_f32_to_bf16` rounds to-nearest-even and emits a
  `SwapCertificate::Bounded` carrying the proven per-element relative rounding bound
  `{Rel, u = 2^−8}` with a `ProvenThm` basis — the strength is *derived from how the bound was
  obtained, never asserted* (RFC-0002 §3), and the theorem's side-conditions are **checked per
  element**: finite, exactly an `f32`, zero-or-normal, no overflow on rounding; each violation is
  a typed explicit `SwapError` (`NonFinite`/`NotAnF32`/`SubnormalUnsupported`/`RoundOverflow`),
  never a silent coercion. Approximate sources are refused (`ApproximateSource`) until the E2-1
  composition rule exists — refusal, never fabrication. The certificate **validates through the
  M-210 shared checker**, a tampered conversion is caught (tier-i rejection), and a new
  `CertifiedSwapEngine` serves the complete certified surface (bijective + bounded + identity),
  explicit `UnsupportedSwap` for everything else. 11 tests incl. a 20k-sweep soundness property
  for the `2^−8` bound and ties-to-even spot checks.
- **Single shared translation-validation certificate checker** (`mycelium-cert::check`, **M-210**,
  Phase 2; RFC-0002 §2; RFC-0004 §3; T1.1): one `check(A, B, R, claimed, evidence)` answering "does
  artifact B refine reference A under relation R within the claimed `{ε,δ,strength}`?" — build once,
  use twice. Three `RefinementRelation` instances: **Bijection** (the M-120 binary↔ternary cert —
  lemma reference + `legal_pair` side-condition checked, then structural *re-derivation equality*
  against B), **BoundedSimilarity** (lossy swaps — the measured A↔B deviation and the claim are both
  re-validated through the E2-4 `mycelium-numerics` tier-i checker; a claim tighter than its
  certificate, a certificate tighter than the measured instance, or a strength upgrade past the
  basis (VR-5) is rejected), and **ObservationalEquiv** (interp↔AOT over the NFR-7 observable —
  the **M-151 differential is folded in** as an instance and now validates every corpus pair
  through this checker). TV incompleteness is an explicit `NotValidated{reason, fallback}` with the
  `UseReference` fallback path — **never a silent pass** (RFC-0002 §2). `mycelium-numerics` now
  exports `basis_strength` (the M-I2…M-I4 basis→strength mapping) for certificate consumers.
  16 checker tests cover all three instances and every refusal path.
- **Interpreter composes approximate inputs honestly** (`mycelium-interp::prims`, **M-204**, Phase 2;
  RFC-0001 §4.7; ADR-010): retires the Phase-1 blanket `ApproxCompositionUnsupported` refusal for
  composable inputs. `exact_result` → `compose_result`: exact-over-exact stays `Exact`/`bound=None`
  (M-I1); over an approximate input it composes per a per-prim `ApproxRule` — `core.id` passes the
  bound through verbatim (citation preserved), `trit.add`/`sub`/`neg` carry the sound affine ε
  composition via `mycelium_numerics::compose_error_bound` (strength `meet`s to the weakest input,
  basis re-derived so M-I2…M-I4 hold), and `bit.*` / `trit.mul` still refuse (no defined ε rule —
  honest, never a fabricated bound). Five new golden tests cover additive ε composition (Proven⊕Proven
  → Proven, ε sums), negation (ε preserved), `core.id` passthrough, meet-down to Declared, and the
  explicit `trit.mul` refusal; the Phase-1 `bit.not` refusal test still holds. **Closes the documented
  Phase-1 honesty gap** (the interpreter previously could not compose approximate inputs).
- **Verified-numerics foundation — two bound kernels + shared certificate + tier-i checker**
  (`mycelium-numerics`, **M-201/M-202/M-203**, Phase 2; ADR-010; RFC-0001 §4.7; SPEC §10.7): a new
  crate realizing ADR-010's two-kernels-one-certificate decision, deliberately *outside*
  `mycelium-core` (KC-3/SoC — the trusted kernel stays small; numerics is a certificate consumer).
  **`error`** composes ε through **affine arithmetic** — `AffineForm` (`x₀ + Σxᵢ·εᵢ`) with *exact*
  linear ops (correlated noise symbols cancel) and a sound `mul` (second-order remainder onto a fresh
  symbol), and the scalar `ErrorBound{eps,norm}` projection (`add`/`sub`/`neg`/`scale`/`mul`).
  **`prob`** composes δ through the **union bound** (`min(1,Σδ)`) and the apRHL `[SEQ]` rule
  (`ApRhlJudgment` — ε adds as the `e^ε` factors multiply, δ adds, both saturating). They meet at the
  shared **`Certificate{eps,delta,strength}`** (`strength` by `meet`), with a **tier-i Rust checker**
  (`check_error_claim`/`check_union_claim`) that re-derives a composition and **rejects any claim
  tighter than the re-derivation** — never a silent pass (RFC-0002 §2) — and the one sanctioned
  cross-kernel rule `accuracy_to_probability` (ADR-010 §4). The three normative properties
  (**Soundness, Monotonicity, Determinism**; RFC-0001 §4.7) are property-tested over 20k-trial inline
  loops (Phase-1 house style — no `proptest`/`rand` dep); 17 tests green, clippy `-D warnings` clean.
- **Phase-2 plan + epic decomposition** (`docs/planning/phase-2.md`; **Phase 2**; Foundation §6;
  SPEC §10.7–§10.10): decomposed the seven Phase-2 epics (#28–#34) into 18 issue-coupled `M-2xx`
  build tasks (#48–#65), created as sub-issues of their epics and joined into `tools/github/idmap.tsv`.
  The plan mirrors `phase-1.md`: readiness table, batch/parallelization structure, the critical path
  (the ADR-010 ε/δ numerics kernels as keystone — they gate every honest approximation downstream),
  and an honest Phase-1→2 re-run of the kill criteria (KC-1 confirmed/no-regression; KC-2
  open/blocked on external LLM access; KC-3 holding — numerics + selection land as their own crates
  to keep the kernel auditable; KC-4 first-measurable when the shared checker lands). Planning
  artifact only — cites the corpus, introduces no requirements.
- **MLIR→LLVM AOT path — ternary-dialect skeleton + runnable AOT artifact** (`mycelium-mlir`,
  **M-150**, Phase 1; RFC-0004 §2/§6; ADR-007; T1.5): `dialect::emit` renders the lowered A-normal
  form as a textual `ternary`-dialect MLIR-style module (one op per binding, all attributes inline —
  the no-opaque-pass anchor), and `aot::run` is the **runnable artifact for the subset** — an
  independent big-step env-machine that executes the lowered ANF directly. Native libMLIR/LLVM
  codegen is **deferred** (Phase 3 matures it; honestly scoped as a textual skeleton + execution
  model, not a compiler).
- **Interp↔AOT differential** (`mycelium-mlir` tests, **M-151**, Phase 1; NFR-7; VR-4; RR-12): a
  harness runs a kernel corpus under both the M-110 reference interpreter (small-step substitution)
  and the M-150 AOT artifact (big-step env-machine over the lowered ANF) and asserts **observable
  equivalence** (repr + payload + guarantee); divergence fails CI. The two paths differ in IR shape
  and evaluation strategy, sharing only the trusted primitive/swap semantics — so the differential
  catches lowering/scheduling/ordering divergence (the cheap baseline preceding per-artifact
  translation validation in Phase 2). A control test confirms the harness discriminates.
- **LSP feedback facade** (`mycelium-lsp::feedback`, **M-140**, Phase 1; FR-S5; Foundation §5.8;
  SC-5): `analyze(node)` exposes the **four** semantic-feedback artifact kinds over one surface —
  (1) typecheck/invariant **diagnostics** (linter), (2) **swap certificates** for statically-
  resolvable swap sites, (3) per-value **bound/guarantee annotations**, (4) **lowering-stage dumps**.
  A failed/unsupported swap is surfaced on the diagnostics channel, never silent. Verified by a
  **scripted-client** integration test driving all four channels (incl. a Proven bound, an
  out-of-range swap, and invariant violations).
- **Canonical formatter** (`mycelium-core::lower::format` + `mycelium-lsp::fmt`, **M-142**, Phase 1;
  RFC-0001 §4.8; ADR-003): a canonical textual normal form that **α-normalizes binder names**
  (`v0, v1, …`), so definitions differing only in names render to identical text and share one
  `content_hash` — reformatting is a projection that never changes content-addressed identity (tested:
  renamed defs format identically and hash equally; formatting leaves identity untouched; free
  variables keep their names).
- **Invariant linter** (`mycelium-lsp::lint`, **M-141**, Phase 1; SC-3; G2; FR-M3; VR-5): static,
  inspectable lints over a Core IR program, emitted as `Diagnostic`s for authoring tools — `implicit-swap`
  (an `Op` mixing paradigms implies a conversion that must be an explicit `Swap`), `unverified-bound`
  (a `Declared` value must always be surfaced, never silently trusted), `placeholder-policy` (a swap
  citing a stub rather than a real `PolicyRef`), and `free-variable` (an open term). Each lint has a
  positive and a negative test. Introduces the toolchain crate `mycelium-lsp` (FR-S5), kept out of
  the auditable kernel (KC-3 — depends on core/interp/cert, nothing depends on it).
- **Inspectable lowering — ≥2 dumpable/diffable stages** (`mycelium-core::lower`, **M-112**, Phase 1;
  RFC-0004 §5/§6; SC-4; WF5): a backend-agnostic lowering pipeline. `stages(node)` returns **`core`**
  (the canonical Core IR tree dump) → **`substrate`** (an A-normal form flattening nested
  `Op`/`Swap`/`Let` to a linear binding list — the pre-codegen shape backends consume), each binding
  whose result repr is statically known (`Const`, `Swap` target) annotated with its **scheduled
  `PhysicalLayout`** (the default schedule, `I2_S` for ternary; RFC-0004 §5 / DN-01). Dumps are
  canonical (deterministic — structurally identical programs render identically, SC-4) and `Meta`
  guarantee tags survive lowering (WF5). `Op`-result layout is left explicitly unannotated (no
  operator typing yet — the omission is honest, not silent; G2).
- **Cleanup / item memory** (`mycelium-vsa::cleanup`, **M-132**, Phase 1; FR-S4; RFC-0003 §3): a
  labelled associative memory (`CleanupMemory`) that snaps a noisy query — an *approximate* `unbind`
  result or a `bundle` decode — to the nearest stored atom by similarity, returning a `Match { label,
  index, confidence, margin }`. The confidence (match cosine) and margin (gap to the runner-up) make
  approximate unbind *usable* and *inspectable* (the retrieval decision is reported, never a hidden
  nearest-neighbour pick; G2). Tested incl. the role⊗filler record-decode use case (bundle two bound
  pairs, unbind by a role, clean up to the right filler).
- **MAP-I bundle capacity bound — `Proven` via checked instantiation** (`mycelium-vsa::capacity`,
  **M-131**, Phase 1; RFC-0003 §5; ADR-010; SC-2; KC-1): `required_dim(m, δ) = ⌈(2/μ²)·ln(m/δ)⌉`
  (μ=0.1) and `proven_capacity_bound` / `MapI::bundle_values_certified`, which attach a **`Proven`**
  `CapacityBound` (basis `ProvenThm`, citing Clarkson-Ubaru-Yang 2023 / Thomas-Dasgupta-Rosing 2021)
  **iff** the checked side-condition `dim ≥ required_dim` holds — exactly the M-001 axiomatized-
  theorem + checked-instantiation pattern (the formula is cited, not re-proven). An undersized
  dimension returns an explicit `InsufficientCapacity` error rather than an unbacked `Proven` tag
  (M-I2/VR-5). `required_dim` reproduces the four M-001 probe settings (1141/1843/2164/2764).
  **Acceptance — ≥10⁴-trial empirical validation (SC-2):** over 10,000 independent trials at
  `dim ≥ required_dim(3, 1e-2)`, the measured nearest-neighbour retrieval-failure rate stays `≤ δ`.
- **VSA submodule — `VsaModel` trait + MAP-I** (`mycelium-vsa`, **M-130**, Phase 1; RFC-0003 §3–§4;
  ADR-008; T2.6): a composition-style `VsaModel` trait (`bind`/`unbind` + self-inverse flag,
  `bundle`, `permute`/`unpermute`, `similarity`, and the honest per-op intrinsic guarantee) and its
  first model **MAP-I** — `bind`/`unbind` are self-inverse and **`Exact`** (elementwise product),
  `permute` is **`Exact`** (cyclic shift), `bundle` is elementwise superposition. Value-level
  adapters for the Exact ops carry honest `Derived` provenance. **Dependency-gated** (ADR-008): the
  crate depends on `mycelium-core` but the kernel does not depend on it — VSA values stay
  type-checkable in the kernel without pulling in this algebra (KC-3). Tests: bind/unbind round-trip
  exactly, permute is invertible/cyclic, a bundle is far more similar to its members than to a
  stranger, dim-mismatch/empty-bundle are explicit errors. The `bundle` **`Proven`** capacity bound
  (M-I2: a *value*-level Proven bound needs a checked basis) is deferred to **M-131** — not stamped
  here (VR-5).
- **Binary↔ternary certified swap** (`mycelium-cert` + `mycelium-core::binary`, **M-120**, Phase 1;
  RFC-0002 §3/§4): `enc`/`dec` per `docs/spec/swaps/binary-ternary.md` over a legal `(n, m)` pair,
  emitting a `SwapCertificate::Bijective` (`LosslessWithinRange`) that references the once-per-pair
  round-trip lemma (`lemma_ref`) bound by concrete `params`. `enc` is total on `B_n`; `dec` is the
  **partial** inverse — a value outside the binary range is an explicit `SwapError::OutOfRange`
  (P4), an illegal pair is a **type error** (`IllegalPair`, RFC-0002 §5), never a `Declared` gamble.
  Within range the result is `Exact`/`bound = None` (P3, M-I1) and records `policy_used` + `Derived`
  provenance. A `BinaryTernarySwapEngine` plugs the swap into the M-110 interpreter. **Acceptance —
  `dec(enc x) = Some x` exhaustively over all 256 bytes** (8↔6, SC-1); serializer output pinned to a
  committed `swap-certificate` example validated against the schema in CI (SC-3). Adds a
  two's-complement codec `mycelium-core::binary` (exhaustively round-trip-tested).
- **Binary↔ternary round-trip proof** (`proofs/binary-ternary-roundtrip/`, **M-121**, Phase 1;
  VR-1/SC-1): the SMT-LIB2 injectivity obligation for the 8↔6 pair — **discharged by Z3 4.16.0
  (`unsat`)**: no two distinct 6-trit vectors collide ⟹ the value map is a bijection onto
  `[−364, 364] ⊇ B_8` ⟹ `dec(enc b) = b` (P1/P2). Wired into `scripts/checks/proofs.sh`
  (skip-graceful without z3); the lemma identity matches `mycelium_cert::roundtrip_lemma_ref()`. P3/P4
  are additionally decided by the M-120 exhaustive Rust corpus. (The fixed `8↔6` instance; a
  width-generic proof is future work — each legal pair gets its own discharged lemma.)
- **Balanced-ternary arithmetic** (`mycelium-core::ternary` + `mycelium-interp`, **M-111**, Phase 1;
  FR-M2): the single home for the balanced-ternary integer codec (`int ↔ trits`, MSB-first, the
  §3.1 digit-extraction algorithm) and fixed-width digit-wise arithmetic — `neg` (digit-wise sign
  flip = value negation), ripple-carry `add`/`sub`, and shifted-add `mul`. Out-of-range results are
  an explicit `None`/`EvalError::Overflow`, **never** a silent wrap (SC-3). The interpreter gains
  `trit.neg/add/sub/mul` primitives over it. **Acceptance — property-tested vs an `i64` oracle by
  exhaustion** over all operand pairs at widths `m ≤ 4` (and the codec round-trip/neg at `m ≤ 5`):
  in range the digit-wise result equals the encoded integer result, out of range it overflows.
  Grounded in `docs/spec/swaps/binary-ternary.md` §1/§3.1; reused by the M-120 swap.
- **Reference interpreter** (`mycelium-interp`, **M-110**, Phase 1): the trusted, executable
  **small-step operational semantics** for the Core IR, closing SPEC §10.3 (RFC-0004 §2; ADR-009;
  NFR-7). Call-by-value substitution over closed `Node`s with the rules E-Let-Bind/Step,
  E-Op-Arg/Apply, E-Swap-Arg/Apply (documented in the crate). An extensible **primitive registry**
  (`PrimRegistry`) ships the exact elementwise built-ins (`core.id`, `bit.not/and/or/xor`,
  `trit.neg`); a **`SwapEngine`** hook ships the trivial same-`Repr` `IdentitySwapEngine`. Results
  thread metadata honestly — guarantee by `meet` (RFC-0001 §4.7), provenance `Derived{op, inputs}`
  over content hashes (§4.6), `policy_used` on swaps. **Never silent**: free variables, unknown/
  ill-typed prims, unsupported cross-paradigm swaps, approximate-input composition (no bound kernel
  yet — ADR-010/E2-4), and fuel exhaustion are all explicit `EvalError`s. 20-case golden corpus.
  Adds `mycelium_core::operation_hash` (provenance op identity for prims). Scope boundary:
  balanced-ternary arithmetic + oracle property tests are **M-111**; the certified binary↔ternary
  swap + proof are **M-120/M-121**.
- **Guarantee `meet`-composition** (`mycelium-core::guarantee`, **M-102**, Phase 1):
  `GuaranteeStrength::meet` (the weakest-wins greatest-lower-bound) plus `propagate`/`meet_all` for
  the RFC-0001 §4.7 rule `guarantee(result) = meet(inputs…, g_f)`, and `TOP`/`ALL` constants. The
  meet-semilattice laws — commutativity, associativity, idempotence, identity `Exact`, `Declared`
  absorbing — are verified by **exhaustion** over all 4×4(×4) tuples (complete for the finite
  lattice, not sampled). Honesty can only degrade, never spuriously upgrade (VR-3/VR-5).
- **Content-addressing** (`mycelium-core::content`, **M-103**, Phase 1): `Node::content_hash` /
  `Value::content_hash` — a BLAKE3 hash over an injective, domain-separated, length-prefixed
  encoding of the *identity-bearing* content: the α-normalized structure (bound vars as de Bruijn
  indices, binder names dropped), types-with-`Repr`, constant literals, operator names, and swap
  target+policy. Dynamic `Meta` (provenance, bounds, sparsity, `policy_used`) is excluded. Adds a
  separable `hash ↔ name` table (`Names`) for names-as-metadata, `ScalarKind::tag`, and
  `ContentHash::from_parts`/`algo`/`digest`. Acceptance met: identical defs collide; trivial (α)
  renames don't change identity; a paradigm/precision/literal/operator change does (RFC-0001 §4.6;
  ADR-003).
- **Core IR (de)serialization** (`mycelium-core`, **M-104**, Phase 1): `serde`
  `Serialize`/`Deserialize` for `Value`/`Meta`/`Repr`/`Bound`/`Provenance`/… emitting *exactly* the
  ratified JSON data contracts (`kind`/`class`/`layout` tags; `VSA`/`BF16`/`TL1`/`TL2` renames;
  `payload` as `{bits|trits|scalars|hypervector}` with MSB-first bit/trit strings; `bound` modelled
  by presence; flat `kind`+`basis` `Bound`). `Deserialize` routes `Value`/`Meta` through their
  checked constructors, so M-I1…M-I4 and payload↔repr mismatches are rejected on the wire — never
  silently accepted. Faithful round-trip (`deserialize(serialize(v)) == v` incl. `Meta`) is tested
  over a corpus spanning all four paradigms × every guarantee/bound/basis/layout; serializer output
  is pinned to three new committed `value` examples (ternary/dense/vsa) that `scripts/checks/schema.sh`
  validates against `value.schema.json` in CI (RFC-0001 §4.8).
- **Core IR data structures** (`mycelium-core`, **M-101**, Phase 1): Rust types mirroring the
  ratified schemas — `Repr`/`ScalarKind`/`SparsityClass`, the `GuaranteeStrength` lattice,
  `Bound`/`BoundBasis`/`BoundKind`/`NormKind` (ADR-011: `basis` universal), `Meta` (with
  `Provenance`, `SparsityObs`, `PhysicalLayout`/`PackScheme`), `Value`/`Payload`, `ContentHash`,
  and the `Node` grammar (closes the core of SPEC §10.2; RFC-0001 §4.5). The honesty invariants
  **M-I1…M-I4** and payload↔repr/repr well-formedness are enforced **by construction**
  (`Meta::new`, `Value::new` → `WfError`). 17 unit tests; `fmt`/`clippy -D warnings`/`test` green on
  MSRV 1.92.
- **Minimal surface-syntax fragment** (`experiments/surface-fragment/`, **M-020**): a throwaway,
  experiment-only concrete syntax (EBNF + desugaring to the Core IR nodes + 3 reference programs:
  swap round-trip, VSA `bundle`, and a no-implicit-conversion type-error) to feed the KC-2
  experiment. **Not** a committed surface — gated on KC-2 (hence under `experiments/`, not
  `docs/spec/`). Linked from `SPECIFICATION.md` §10.1.
- **Binary↔ternary encoding spec** (`docs/spec/swaps/binary-ternary.md`, **M-012**): precise
  `enc`/`dec` for the canonical `8↔6` width — balanced-ternary digit semantics, the legality
  condition `B_n ⊆ T_m`, `LosslessWithinRange` with an `Option`-typed (never-silent) inverse, the
  four M-121 correctness obligations, and a worked round-trip + out-of-range example (RFC-0002
  §4/§5; T2.1). Linked from `SPECIFICATION.md` §6/§10.4.
- **Python tooling skeleton** (`experiments/`, **M-092**): a UV-managed project targeting
  **Python 3.13** (ADR-007) with a `dev` group (pytest, pytest-cov, ruff, black), a trivial
  importable module + passing smoke test, and a committed `uv.lock`. `scripts/checks/test.sh` runs
  it via `uv run --frozen pytest` under the pinned interpreter, so it joins the `just check`/CI
  suite (skip-graceful when uv is absent).
- **Rust workspace skeleton** (**M-091**): a 6-crate Cargo workspace (`mycelium-core`,
  `mycelium-interp`, `mycelium-vsa`, `mycelium-mlir` stub, `mycelium-cert` stub, `xtask`) with
  **MSRV pinned to 1.92** via `rust-toolchain.toml` + `rust-version` (ADR-007), workspace lints
  (`unsafe_code = forbid`, clippy warn), and a smoke test per crate. `cargo fmt --check`,
  `clippy -D warnings`, and `cargo test` are all green on 1.92. Adds `scripts/checks/test.sh` +
  `just test`, wired into the `just check`/CI suite (skip-graceful when a toolchain is absent), so
  test parity now holds local↔CI. Fixes a malformed `Cargo.lock` line in `.gitignore`.
- **M-001 probe scaffold** (`proofs/lh-bundle/`): the Liquid-Haskell MAP-I `bundle`
  capacity-refinement module + cabal project + writeup, encoding the axiomatized-theorem +
  checked-instantiation strategy with ≥3 concrete `(d,m,δ)` settings (RFC-0003 §5; T0.2). **Not yet
  discharged** — no GHC/LH/Z3 in this environment — so KC-1 stays `passed (literature)`; the
  derivation table is the independently-checkable artifact. Establishes `proofs/<name>/` as the
  home for machine-checkable proofs (resolves OQ-2).
- **`SPECIFICATION.md` skeleton** (`docs/spec/SPECIFICATION.md`, **M-011**): the consolidation index
  over the corpus — §1–§9 reconciled to RFC-0001 (r2)/RFC-0002…0005/ADR-010/011/DN-01 and pointed at
  the ratified `docs/spec/schemas/` contracts; §10 enumerates the open build items, each linked to a
  live issue (no floating TODOs). Status `consolidating-draft → ratified-skeleton`.
- **ADR-011 — `BoundBasis` is a property of every `Bound`** (`docs/adr/ADR-011-...md`, Accepted):
  formally supersedes the implicit RFC-0001 r1 §4.3 decision that scoped `basis` to `CapacityBound`
  only, so every approximate value (ε, δ, crosstalk, capacity) honestly records how its bound was
  obtained (VR-5, G5). Resolves OQ-3.
- **Core data-contract schemas** (`docs/spec/schemas/`, **M-010**): the 10 ratified JSON Schemas
  (draft 2020-12) — `repr`, `value`, `meta`, `guarantee`, `bound`, `provenance`,
  `physical-layout`, `swap-certificate`, `policy`, `reconstruction-manifest` — each a faithful
  projection of its source RFC/ADR section, plus ≥1 valid and ≥1 invalid example per schema (the
  invalids exercise the honesty-load-bearing invariants M-I1/M-I4). `just schema` validates the
  set in CI. The OQ-3/OQ-4/OQ-5 clarifications surfaced here are now resolved (see below /
  `docs/spec/schemas/README.md`).
- **Phase-0 working plan** (`docs/planning/phase-0.md`): the first issue-coupled expansion of
  Foundation §6, mapping the nine Phase-0 tasks (M-001/002/010/011/012/020/090/091/092) to their
  GitHub issues, the critical path, honest KC-1/KC-2 gate status, the proposed canonical
  data-contract schema set, and the author-then-ratify reframing for M-010/M-011 (the
  `docs/spec/` artifacts they ratify do not exist yet).
- Initial **design baseline**: project charter (`docs/Mycelium_Project_Foundation.md`, r3),
  document index (`docs/Doc-Index.md`), five RFCs (RFC-0001…0005, all Accepted),
  ADR-010 (Accepted), design note DN-01 (Resolved), and two research records
  (`research/01`, `research/02`).
- Repository scaffolding: `README.md`, `LICENSE` (MIT), `CONTRIBUTING.md`,
  `.gitignore` (Rust + Python), and index/process READMEs for `docs/adr/` and `docs/rfcs/`.
- **GitHub PM bootstrap** (`tools/github/`): `issues.yaml` / `labels.json` / `milestones.json`,
  the `mcp-bootstrap.md` runner + `gh-bootstrap-local.sh`, the `project-v2-spec.md` board spec,
  and the `idmap.tsv` task→issue map.
- **Agent tooling**: `CLAUDE.md` and `.claude/skills/` (`pr-review`, `security-review`,
  `dev-workflow`, `docs-review`, `changelog`) operationalizing the `CONTRIBUTING.md` house rules.
- **Local check tooling** with local↔CI parity: `justfile` + `scripts/checks/*` (markdownlint,
  offline link/cross-reference, json-schema, codespell, shellcheck, secret scan, fmt/lint),
  `.pre-commit-config.yaml`, and a manual-dispatch **advisory** GitHub Actions workflow.

### Changed
- **Proofs wired into the check suite** (`scripts/checks/proofs.sh` + `just proofs`): runs the
  LiquidHaskell `bundle` probe (`LC_ALL=C.UTF-8 cabal build`, a green build ⟺ LH `SAFE`),
  skip-graceful when GHC/cabal/z3 are absent. Added to `just check`/`just ci`; the manual-dispatch CI
  workflow now sets up GHC 9.8.2 + cabal + z3 (with a cabal/dist-newstyle cache) so the proof
  verifies on a manual run. (Whole suite remains `workflow_dispatch`-only.)
- **KC-1 confirmed (build)** (**M-001**): the Liquid-Haskell MAP-I `bundle` capacity refinement
  (`proofs/lh-bundle/`) type-checks **`SAFE` (16 constraints)** and Z3 discharged all four `(d,m,δ)`
  instantiations (GHC 9.8.2 · LiquidHaskell 0.9.8.2 · Z3 4.8.12), ratifying the axiomatized-theorem +
  checked-instantiation strategy (RFC-0003 §5; ADR-010). KC-1 moves `passed (literature) → confirmed
  (build)` in the Foundation §2.4 and Doc-Index §3/§4. (The Clarkson/Thomas theorem remains cited,
  not re-proven — by design.) Haskell build output (`dist-newstyle/`, `.liquid/`) gitignored;
  codespell skips them.
- **Docs/parity CI hardened** (`.github/workflows/checks.yml`, **M-090**): the manual-dispatch
  advisory workflow now sets up **uv** (so the `experiments/` Python 3.13 tests actually run) and
  **Rust** (pinned via `rust-toolchain.toml`, so fmt/clippy/test run), and adds an advisory
  **Codecov** upload of the experiments coverage. Markdown-lint + offline link-check + schema
  validation already covered `docs/**` and the schemas via `just ci`; the PR template was already
  wired. Posture unchanged: `workflow_dispatch` only, non-blocking (no auto-triggers — CLAUDE.md).
- **RFC-0001 → r2** (status stays Accepted): §4.3 `Bound` grammar revised per **ADR-011** —
  `BoundBasis` factored out to a required companion of *every* `Bound` (was: `CapacityBound` only),
  and `NormKind` enumerated `L1|L2|Linf|Rel` as an extensible registry (resolves OQ-4). The r1 §4.3
  grammar is formally superseded; indexes (`Doc-Index.md`, `docs/rfcs/README.md`,
  `docs/adr/README.md`) and the `bound` schema updated to match.

### Changed (baseline-review consistency pass)
- ADR-001 promoted to firmly **Accepted**; the "no statistical approximation vs
  fully-disclosed approximation" definitional question recorded as **settled**
  (fully-disclosed), consistent with the KC-1 pass and the guarantee lattice.
- Foundation §5.2 core-model sketch marked **superseded by RFC-0001** (packing is
  now schedule-staged, not in the type; guarantee lattice is the four-point form).
- Foundation §5.6 updated: **MLIR→LLVM** recorded as the committed AOT path
  (ADR-007 / RFC-0004), not a candidate.
- Foundation §6 Phase 0 annotated with post-research status (largely complete;
  remaining: the Liquid-Haskell `bundle` probe and the KC-2 LLM-leverage experiment).
- `README.md` decisions table: fixed a placeholder reference for the
  "no implicit conversion" rule (grounded in RFC-0001 §3.3 / FR-M3).
- `docs/Doc-Index.md`: the two research rows now point to the in-repo records.

### Fixed
- Markdown hygiene surfaced by the new check tooling: normalized emphasis to the corpus
  asterisk style and added a missing trailing newline (`README.md`,
  `research/01-prior-art-survey-RECORD.md`, `docs/notes/DN-01-Packing-Placement-Tradeoffs.md`).
- Copilot PR-review findings (PR #1, #42) addressed: corrected the binary↔ternary swap's partial
  right-inverse in RFC-0002 §4 (`dec y = Some x ⟹ enc x = y`; the prior `enc y = …` was a type
  error since `enc : Bin_n → Tern_m`); resolved a P0.3 status contradiction in the Foundation
  Meta section (P0.3 is already resolved per §6); corrected stale references in `tools/github/`
  (`gh-bootstrap.sh`, `docs/planning/*`, `project-v2-spec.md`); `gh-bootstrap-local.sh` now
  honors each milestone's `state` instead of hardcoding `open`.
- Tooling self-lint: `scripts/*` made shellcheck/ruff/markdownlint-clean (cd-failure guards,
  if/then/else over `A && B || C`, split imports, fenced-block spacing).

### Security
- `.gitleaks.toml`: removed an allowlist **regex** (`AKIA[0-9A-Z]{16}`) that exempted the AWS
  access-key-ID *pattern* from scanning — it would have suppressed detection of a real leaked
  key. The path allowlist is retained; pattern-level allowlisting is documented as forbidden.

### Open
- One confirming build: the Liquid-Haskell `bundle` capacity-refinement probe (RFC-0003 §5).
- One existential question: **KC-2 / LLM leverage** (the E4 experiment) — not yet settled.
- Decomposed task/issue set and phase planning documents — *forthcoming* (`docs/planning/`).
