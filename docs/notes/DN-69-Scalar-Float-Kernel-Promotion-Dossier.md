# Design Note DN-69 — DN-39 Promotion Dossier: the Scalar-Float Value Form (Route ii)

| Field | Value |
|---|---|
| **Note** | DN-69 |
| **Status** | **Draft** (2026-07-02 — advisory; **maintainer ratifies**). A **review instance of the DN-39 four-clause default-DENY kernel-promotion bar**, applied to the scalar-float value-form kernel entry that ADR-040 (Proposed, same date) designs. **Recommendation: PROMOTE** — the candidate clears all four conjunctive clauses, the first candidate to do so (DN-39's sole prior candidate was a decisive KEEP-OUT). Default-DENY is overcome **on merit, argued clause-by-clause with per-clause tags** — not waved through because ADR-038 wants the feature. This note enacts nothing and moves no status: kernel entry happens only when the maintainer ratifies **both** this dossier's verdict **and** ADR-040 (the ADR-038 §2.6 double gate), and even then only via M-896…M-900. |
| **Feeds** | The **KC-3 small-auditable-kernel** invariant (house rule #5) and the trusted-core boundary **DN-39 §7** records; **ADR-040** (the design under review); **ADR-038 §2.6** (the double gate this dossier is half of); M-896…M-900 (gated implementation). |
| **Date** | July 2, 2026 |
| **Decides** | *Nothing normatively.* Records (1) the recording-format choice for DN-39 review instances (a new dated note — FLAGged, §1); (2) the precise candidate surface (§2); (3) the clause-by-clause adjudication under the DN-39 §2 bar with honest tags and evidence pointers (§3); (4) the **PROMOTE** recommendation (§4); (5) why this verdict is consistent with DN-39's "encodings are the last thing to axiomatize" principle (§5); (6) the open-question ledger (§6) and the guarantee posture + DoD (§7). |
| **Task** | M-895 (kickoff `enb`, Gap A — Phase-I H1) |

> **Posture (transparency rule / VR-5 / G2).** Advisory dossier for maintainer ratification. The
> author of this dossier also drafted ADR-040 — the review is therefore an **argued case, not an
> independent audit**, and is tagged accordingly (nothing here exceeds `Empirical`; the maintainer's
> ratification is the independent gate, and house rule #4 forbids treating the ADR's own enthusiasm as
> evidence). DN-39 is treated as the authoritative bar and is not modified (append-only): this note is
> a **new instance record**, chosen because DN-39 prescribes no recording format for later reviews
> (its §3 covered "this review batch" only, and §8 OQ-4 anticipated future candidates without fixing a
> venue) — that format choice is itself FLAGged (§1, ADR-040 FLAG-7).

---

## §1 Purpose, venue, and honest scope

ADR-038 §2.6 double-gates the route-(ii) scalar-float `Repr`: "a dedicated future float ADR … plus a
**DN-39 promotion review** (the default-DENY four-clause bar — a scalar float must *earn* kernel
entry)." ADR-040 is the ADR; this note is the review.

**Venue choice (FLAG).** DN-39 records the bar and one adjudicated batch; it does not prescribe where
later instances are recorded. Amending DN-39's §3 table would edit an Accepted note's adjudication
record — against the append-only grain — so this dossier is a **new dated note cross-linking DN-39**.
The maintainer may prefer a different convention for future instances; FLAGged here and as ADR-040
FLAG-7, not guessed.

**Scope.** The dossier adjudicates exactly the candidate ADR-040 §2 defines — nothing wider. It does
not re-open the route decision (ADR-038's), does not adjudicate transcendental/libm prims (explicitly
out of the candidate surface, ADR-040 §2.5), and does not survey other promotion candidates (the
DN-39 OQ-4 posture carries over).

## §2 The candidate — what exactly would enter the trusted core

Per DN-39 §7, the TCB already comprises the L0 Core IR, the reference interpreter
(mycelium-interp / mycelium-l1 / mycelium-core), the content-addressing primitive, the guarantee
lattice, and the swap engine. The candidate adds, inside that existing boundary:

- one `Repr` variant — `Repr::Float { width }`, F64-only frozen-tag width registry
  (`crates/mycelium-core/src/repr.rs`);
- one scalar payload arm and two `Canon` arms (a frozen `REPR_FLOAT` tag; the payload-bits encoding
  with canonical NaN — `crates/mycelium-core/src/content.rs`, today's `fn repr` line 227 and
  `fn payload` line 275 grow one arm each);
- the minimal interpreter prim set of ADR-040 §2.5: `add`/`sub`/`mul`/`div`/`neg` (RNE), partial
  comparison plus the named total order, classification, never-silent conversions
  (`crates/mycelium-interp/src/prims.rs`);
- the identity commitments of ADR-040 §2.3/§3 (canonical NaN; bit-distinct signed zeros; frozen
  width tags).

**Explicitly NOT in the candidate:** transcendentals and anything libm-dependent (`sqrt`/`exp`/`log`/
trig — `std.math`'s surface, M-525/M-541); float literals' surface syntax (M-897 is L1 frontend work
inside the already-trusted interpreter tier, not a new trust class); any Dense/VSA change.

A load-bearing prior fact (`Empirical`, source-read): **f64 bit-handling is already inside the TCB.**
`Canon::f64` (content.rs:156–158) hashes f64 bits into every Dense/Hypervector value's identity today,
and `Payload::Scalars`/`Payload::Hypervector` already carry f64 through the trusted interpreter. The
candidate names and disciplines a trust class that is already present; it does not introduce f64 to
the kernel.

## §3 The bar, clause by clause (DN-39 §2 — conjunctive, default-DENY)

### §3.1 Clause (1) — Foundational: PASS

A `Repr` variant **defines execution semantics**: which values exist, what their identity is, and what
the differential oracle's ground truth says about programs that compute with them. That is the
kernel's own job description — DN-39 §6: the L0 IR and reference interpreter "*define execution
semantics*, the ground truth the differential oracle is rooted in (NFR-7)." A scalar-float value form
cannot be foundational-to-something-above-the-kernel because there is no above-the-kernel place where
a value form with content-address identity can live: identity is computed by the kernel's `Canon`
encoder over kernel `Repr` + `Payload` types (content.rs `fn value`, lines 329–334). Contrast the
DN-39 candidate, which was foundational-to-*deployment* only.

**Tag: `Empirical`** (source-read: `Repr`/`Canon`/`Value` live in `mycelium-core`, inside the DN-39 §7
boundary; the "no above-kernel home for an identity-bearing value form" claim follows from the code
structure as read, not from a theorem). Evidence: `repr.rs:81–127`, `content.rs:227/275/329–334`,
DN-39 §6–§7.

### §3.2 Clause (2) — Unverifiable-from-outside: PASS (the dispositive clause), with an honest split

DN-39's dispositive test: if already-trusted checking machinery *can and does* establish the
obligation, the component must be **verified, not trusted**. The candidate splits cleanly, and the
split is the argument:

- **The definitional core — what `flt.add` MEANS — cannot be verified from outside, because it is the
  ground truth other things are verified against.** There is no more-primitive trusted checker to
  test the reference interpreter's float semantics *against*: the AOT/MLIR path is validated against
  the interpreter (NFR-7), not vice versa. This is DN-39 §6's own criterion for the trusted side —
  "they must be trusted because there is no more-primitive checker to verify them *against*" — and it
  is exactly the respect in which the spore encoding differed: the encoding's sole obligation
  (injectivity) was checkable by an *already-trusted* re-hash round-trip, so trusting it was optional.
  A value form's semantics are definitional; trusting them is not optional, only *locating* them is —
  and the only alternative locations (§3.3) are worse.
- **Everything checkable IS checked, not trusted (the DN-39 "verify, don't trust" half).** The host's
  IEEE-754 conformance is `Empirical` (property/differential tests against reference cases — ADR-040
  §2.6, never upgraded to an axiom); NaN canonicalization becomes a property test (no constructor
  yields a non-canonical NaN); address stability of the added variant becomes a regression test
  (M-896 DoD). The candidate axiomatizes the *definition* and verifies every *conformance claim* —
  the exact opposite of the KEEP-OUT pattern, which asked to axiomatize a checkable obligation.

**Tag: `Empirical`** for the structural claim (NFR-7 direction-of-validation, DN-39 §6, read from the
corpus; no theorem is available for "no more-primitive checker exists" — it is an architecture fact).
The clause passes on the definitional core; the dossier claims **no** pass for any checkable
sub-obligation, which all remain verified.

### §3.3 Clause (3) — Net-trust-reducing: PASS

Keeping the scalar-float value form **out** of the kernel forces exactly the escape hatches clause (3)
names:

- **Route (i), `Dense{dim:1}` scalars (the workaround ADR-038 rejected):** scalar semantics smuggled
  through tensor machinery — a per-op **black box** in which scalar accuracy claims ride ε machinery
  designed for tensor granularity (the ADR-030 lesson: wrong-granularity descriptions are "silently
  wrong at the `Repr` level"). House-rule-#2 violation by construction.
- **Library encodings (f64 bits cast into `Binary{64}`/`Bytes`):** every consumer re-implements
  NaN/zero/identity conventions privately — **duplicated trust** with no shared, inspectable
  commitment; two libraries' "floats" with the same bits could differ semantically, and content
  addresses would say nothing about it. A **silent-trust leak** distributed across the ecosystem.
- **The status quo:** the gap simply persists — readiness §0 blocker-1 (`Exact`) keeps `math`'s f64
  half and `numerics` inexpressible, which in practice pushes float work into host-language escape
  hatches outside the value model entirely.

Against that, the added trusted surface is small (§3.4) and — the prior fact from §2 — **f64
bit-handling is already trusted today** (the hasher and payload paths). Promotion converts an
implicit, undisciplined trust (raw platform bits flowing into identity, NaN nondeterminism and all —
ADR-040 §2.3's surfaced seam) into an **explicit, reviewed, property-guarded commitment** (canonical
NaN, frozen tags, never-silent boundaries). Net trust goes **down**: fewer places where float
semantics can silently vary, one audited place where they are defined.

**Tag: `Empirical`** for the already-trusted-f64 fact (source-read) and the workaround enumeration
(grounded in ADR-038 §2.6's own rejection of route (i) and the corpus's never-silent rules);
**`Declared`** for the net direction ("trust goes down") — it is an argued judgment, the maintainer's
to ratify, not a measured quantity.

### §3.4 Clause (4) — Small + auditable: PASS (kept true by scope discipline)

The candidate surface (§2) is one variant, one payload arm, two encoder arms, one frozen tag, a
single-digit prim set, and two property-test obligations — of the same order as the existing
`Binary`/`Ternary` forms (`Empirical` for the shape as designed; the LOC claim is a **`Declared`
estimate** until M-896 exists, and the KC-3 delta review in M-896's DoD is where it gets checked). No
foreign concern is imported: no I/O, no presentation, no encoding-of-something-else — IEEE-754
binary64 is a single, fixed, publicly specified format, and the one genuinely foreign-concern risk
(**libm** — a large, platform-varying numeric library) is **excluded from the candidate by
construction** (ADR-040 §2.5). The clause-(4) case is *conditional on that exclusion holding*: if
transcendental prims are later proposed for the kernel, that is a **new DN-39 review**, not covered by
this dossier (§6 OQ-2).

**Tag: `Empirical`** (shape, from the design + existing sibling variants) / **`Declared`** (size
estimate), with the condition stated, not buried (G2).

## §4 Verdict — PROMOTE (recommendation; maintainer ratifies)

| Candidate | Verdict | One-line ground |
|---|---|---|
| Scalar-float value form per ADR-040 §2 (`Repr::Float{F64}` + canonical-NaN identity + minimal RNE prim set; libm excluded) | **PROMOTE** (recommended) | Passes all four conjunctive clauses: it *defines* execution semantics and identity (1), its definitional core has no more-primitive checker while every checkable conformance claim stays verified-not-trusted (2), keeping it out forces black-box/duplicated-trust workarounds over f64 machinery the TCB already trusts implicitly (3), and the earned surface is one variant + a single-digit prim set with libm excluded (4). |

Default-DENY is overcome **on merit**: the burden of proof was argued clause-by-clause against the
same bar that produced DN-39's KEEP-OUT, and the dossier's own strongest tag is `Empirical` — the
maintainer's ratification, not this text, is what admits the candidate. A single failed clause would
have meant KEEP-OUT; the maintainer should test §3.2 hardest, since it is the dispositive clause and
its structural claim is architectural rather than theorem-backed.

## §5 Consistency with DN-39's principle — this is not an encoding

DN-39 §6's adopted principle — *"a deterministic encoding is the most testable artifact in the
system, so it is the last thing to axiomatize into the kernel"* — cuts **for** this verdict, not
against it. The spore pre-image was an **encoding of something else**, whose one obligation was
checkable from outside; the float value form is a **semantics-defining value form**, the class of
thing DN-39 §6 places on the trusted side by name (the L0 IR + interpreter "must be trusted because
there is no more-primitive checker"). The two verdicts (KEEP-OUT there, PROMOTE here) are the same
rule applied to opposite sides of DN-39's own split — and this dossier keeps the encoding-shaped parts
of the float work (the `Canon` byte layout's injectivity, host conformance, canonicalization behavior)
on the **verified** side, exactly as that split demands (§3.2, second bullet).

## §6 Open-question ledger

- **OQ-1 (venue for future instances — FLAG).** This dossier chose "new dated DN note cross-linking
  DN-39" as the instance-recording format (§1). Maintainer to confirm or fix a different convention;
  also ADR-040 FLAG-7.
- **OQ-2 (libm/transcendentals are NOT covered).** The clause-(4) pass is conditional on the §2.5
  exclusion. Any future proposal to put `sqrt`/`exp`/`log`/trig **in the kernel** requires a fresh
  DN-39 review; surfacing them above the kernel under honest tags (M-525/M-541) needs none.
- **OQ-3 (width growth).** Each added `FloatWidth` grows the audited surface and swap matrix.
  Recommended posture: widths beyond F64 ride the ADR-040 append-only registry **with a KC-3 delta
  note at implementation**, not a full re-review — but the maintainer may prefer per-width DN-39
  instances. FLAGged, not decided.
- **OQ-4 (the FLAG-5 uniform-NaN settlement).** If NaN canonicalization is extended to the existing
  Dense/Hypervector f64 paths (ADR-040 FLAG-5), that change is identity-affecting for NaN-bearing
  tensors and belongs to the same E20-1 settlement — it is a value-model discipline item, not a new
  promotion, but it is recorded here so it cannot land silently.
- **OQ-5 (self-review limitation).** Dossier author = ADR author (§Posture). If the maintainer wants
  an independent adversarial pass before ratifying, that is a cheap additional gate; this note does
  not claim to substitute for one.

## §7 Guarantee posture & Definition of Done

**ENACTS NOTHING.** Advisory instance of the DN-39 bar; no kernel boundary moves from this note, no
code ships, no status flips. Append-only: DN-39 is cross-linked, never edited.

**Per-clause tags (lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`; VR-5 — nothing upgraded past its
basis):** clause (1) `Empirical` (source-read structure); clause (2) `Empirical` (architecture fact,
no theorem claimed); clause (3) `Empirical` for the evidence + `Declared` for the net-direction
judgment; clause (4) `Empirical` shape + `Declared` size estimate, conditional on the libm exclusion.
The motivating gap is `Exact` (readiness §0). **No `Proven` appears in this dossier** — deliberately:
no side-condition-checked theorem backs any of it, and saying so plainly is the rule (house rule #1).

**Definition of Done (the Draft → Accepted gate).** The maintainer (a) ratifies or rejects the
**PROMOTE** recommendation (either outcome resolves this note honestly — a rejection would gate
M-896…M-900 closed pending a superseding design); (b) rules on OQ-1 (instance venue) and OQ-3 (width
growth posture); (c) ratifies ADR-040 separately — this dossier passing does **not** auto-accept the
ADR, and vice versa (the §2.6 gate is a conjunction). CHANGELOG / Doc-Index / issues.yaml /
docs/api-index are owned by the integrating parent, per the concurrent-PR pattern.

---

## Meta — changelog

- **2026-07-02 — Created (Draft, advisory) — authored (task M-895, kickoff `enb` Gap A).** First
  review instance of the DN-39 four-clause default-DENY kernel-promotion bar, applied to the
  ADR-040 scalar-float value-form candidate (route (ii), ADR-038 §2.6's double gate). Records the
  candidate surface (§2: one `Repr` variant + payload/`Canon` arms + minimal RNE prim set; libm
  excluded), the clause-by-clause adjudication (§3: all four PASS, with the honest
  definitional-core-vs-checkable-claims split on the dispositive clause (2)), the **PROMOTE**
  recommendation (§4 — maintainer ratifies; strongest tag `Empirical`, no `Proven` claimed), the
  consistency argument with DN-39 §6's encodings-last principle (§5), and the open-question ledger
  (§6: instance venue FLAG; libm not covered; width-growth posture; the FLAG-5 uniform-NaN E20-1
  settlement; the self-review limitation). Enacts nothing; moves no status. Shared indices owned by
  the integrating parent. (Append-only; VR-5; G2.)
