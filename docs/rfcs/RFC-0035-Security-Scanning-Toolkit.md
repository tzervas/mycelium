# RFC-0035 — Security Scanning Toolkit (automated detection · standard reporting · honest safe auto-fix)

| Field | Value |
|---|---|
| **RFC** | 0035 |
| **Status** | **Proposed** (2026-06-24) — the binding design lifted from the settled **DN-30** direction capture plus the maintainer's now-answered §7 open questions (the five Decisions, §10). Awaiting ratification (Proposed → Accepted, house rule #3, stepped not skipped). |
| **Type** | Toolchain / normative-once-Accepted — designs a native security-scanning toolchain component over the existing RFC-0001 (inspectable Core IR + content-addressing), RFC-0002 (refinement certificate), RFC-0034 (transparency modes + scoped resolution), DN-28 (registry reconstruction-on-render) substrate. The substrate mechanisms are **unchanged**; this RFC composes them into a security capability. |
| **Date** | 2026-06-24 |
| **Decides** | the v0 vulnerability-class set as a **fixed base with an extensibility seam** (§2 / D1); the reporting standard set — SARIF + CWE + OSV + VEX — with **versioned-pinned, immutable-once-pinned** schemas (§3 / D2); the registry as a **second content-addressed catalog** reusing DN-28 reconstruction-on-render, hosting **screened/anonymized** advisories (§4); the honest semantics-preserving **safe auto-fix** reusing the RFC-0002 certificate, with **per-class fix-strength + a pedantic mode** and a **certified patch registry** (§5 / D3, D5); the **screening policy** as configurable-with-defaults, **mandatory-by-default for high-security classes** (§6 / D4); native + scoped resolution reusing RFC-0034 §6 (§7); the five answered open questions as decisions (§10). |
| **Depends on** | RFC-0001 (inspectable Core IR — the analysis surface; the `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` lattice; content-addressing §4.6); RFC-0002 (the refinement / translation-validation certificate the safe-fix reuses); RFC-0034 (transparency modes + the §6 `@certification` project/phylum/nodule scoped resolution; per-op honest tags); RFC-0013/RFC-0014 (structured diagnostics + declarative recovery — the reporting + fix machinery); RFC-0005 (`PolicyRef` / EXPLAIN — fixes + selections are EXPLAIN-able); RFC-0028 / ADR-014 (the `unsafe`/FFI vuln surface, sharpened by ADR-032 to safe-by-default + explicit per-use escape); DN-28 (registry content-hash DAG + reconstruction-on-render — the advisory feed's host model); **VR-5 / G2** (downgrade-don't-overclaim; never-silent — the transparency contract). |
| **Coupled with** | the `/security-review` skill (becomes a **front-end consumer** of the toolkit's findings, not replaced — §5 / D5); `scripts/checks/deny.sh` + `.gitleaks.toml` + `just safety-check` (the existing CI-side hygiene the toolkit subsumes/correlates, not duplicates); `crates/mycelium-cert/*` (the RFC-0002 certificate machinery a safe-fix reuses); the registry crate (DN-28 / M-732 lineage) — the advisory catalog's host. |
| **Anchor** | **DN-30** (`docs/notes/DN-30-Security-Scanning-Toolkit.md`) — the direction capture this RFC binds. |
| **Task** | epic **E22-1** (`tools/github/issues.yaml`) — the toolchain-component backlog; leaves are minted at this design gate (the forward-epic pattern). |

> **Posture (transparency rule / VR-5).** This RFC **designs the toolkit; it implements nothing.** No
> scanner, no reporter, no fix-certifier, and no registry catalog land alongside it; every statement about
> the toolkit's runtime behaviour is a `Declared` design position to be discharged by implementation (E22-1).
> The machinery it composes — the RFC-0001 inspectable IR, the RFC-0002 certificate + checker, the RFC-0034
> mode resolution, the DN-28 reconstruction-on-render distribution — is **pre-existing and unchanged**; this
> RFC only states *how a security capability is built from it*. The two **worked examples** that would raise
> a key claim from `Declared` to a demonstrated design (a safe-fix refinement-certificate worked example; a
> screening case study) are **explicitly deferred as pre-Accepted work** (§9) — they are **not fabricated
> here** (VR-5 / G2). Where this RFC says "the toolkit detects / reports / fixes", read "the toolkit is
> *designed to* detect / report / fix" — no shipped guarantee is claimed.
>
> **Provenance.** Lifted from **DN-30** (§2–§6 principles + §7 open questions) and the maintainer's
> 2026-06-24 answers to those five questions (§10). Grounded in the existing security tooling
> (`.gitleaks.toml`, `scripts/checks/deny.sh`, `just safety-check`), the `/security-review` rubric's
> recurring-defect bank, and the reuse of RFC-0001/0002/0005/0013/0014/0034 + DN-28 + RFC-0028/ADR-014. The
> external standards (SARIF, CWE, OSV/OSV-Schema, VEX) and prior-art feeds (OSV.dev, GHSA, RustSec, npm
> audit) are cited as prior art (§11), not as anything this RFC ships.

## §1 Problem & Goal

Mycelium has *CI-side* security hygiene today — `.gitleaks.toml` (secrets), `scripts/checks/deny.sh`
(cargo-deny supply-chain), `just safety-check` (the RFC-0028 `unsafe`/FFI audit) — and a **manual**
`/security-review` lens (a human or agent reading a diff). What it lacks is a **shippable,
Mycelium-native security toolkit**: a first-class toolchain component that downstream developers
*inherit* (like the LSP, linter, formatter, and the RFC-0034 §13 / M-796 testing toolkit), which
**logically enumerates, finds, and reports** vulnerability classes **automatically** — and can **fix**
them honestly.

> **Erratum (2026-06-25, post corpus-alignment audit).** For accuracy of the "what it lacks" framing
> above: a **Mycelium-native security crate already exists** — `mycelium-sec` (`myc-sec`, M-367), whose v0
> provides the `wild`-block / ADR-014 `// SAFETY:`-presence audit plus secrets / supply-chain
> orchestration (`crates/mycelium-sec/src/lib.rs`). It is **not** the full automatic vuln-class
> enumerate/find/report/fix toolkit this RFC designs, but this RFC **builds on it** rather than starting
> greenfield; read §1 as understating the existing native surface, not asserting its absence. RFC-0035
> remains Proposed (it implements nothing here); this note adds no status move (VR-5/G2).

**Goal.** Design that component so it:

1. **Detects independently** (§2) — by logical/semantic analysis over the inspectable Core IR (RFC-0001,
   no black boxes), enumerating vulnerability *classes*, not only matching a list of *documented* findings;
   it correlates with documented findings when one exists and reports the same way when it finds one
   independently. This is the **never-silent rule (G2) applied to security**: a flaw the toolchain can see
   is surfaced, never hidden.
2. **Reports in a programmatically-leverageable standard** (§3) — SARIF + CWE + OSV + VEX, each finding
   carrying provenance and an **honest confidence tag** on the lattice (a proven taint-flow vs a heuristic
   pattern-match is disclosed, never conflated — the transparency rule applied to the scanner's own claims).
3. **Reports to the community** (§4) — find-once, report-to-two-sinks: the local CLI report **and** the
   registry, which hosts **screened/anonymized** advisories alongside packages as a second
   content-addressed catalog.
4. **Offers honest safe auto-fixes** (§5) — semantics-preserving fixes that ship an RFC-0002 refinement
   certificate, never a black-box rewrite; the strongest of these populate a **certified patch registry**.
5. **Is native + scoped** (§7) — inherited by downstream developers, configurable per project / nodule /
   granular, never forced.

**Non-goal.** This RFC does not replace the formal `/security-review` (or `/pr-review`) human/agent lenses
(§5 / D5); the toolkit is a **supporting tool** for detection, suggested fixes, and the patch registry, not
a substitute for or a prerequisite to a formal review.

## §2 v0 Vulnerability Classes — fixed base with an extensibility seam (D1)

**Decision (Q1 → D1): the v0 class set is a FIXED base of categories, WITH an extensibility seam.** The
base is closed and versioned (so a scan's coverage is *stated*, never vague); new classes are added through
a declared extension point (so the ecosystem is not frozen to v0's threat model). A class added through the
seam is **never silently folded into the base**: it is a first-class, versioned class with its own CWE
mapping and its own fix-strength bar (§5).

**The v0 fixed base** (grounded in Mycelium's own surfaces and the `/security-review` rubric's recurring
defects; each maps to one or more CWE):

| Class (v0 base) | Surface it watches | Detection basis (honest tag ceiling) |
|---|---|---|
| **`unsafe`/FFI misuse** | the ADR-014 / RFC-0028 `unsafe`/FFI escape sites | structural (every site is an explicit per-use escape — ADR-032) → `Exact` enumeration of sites; misuse classification `Empirical`/`Declared` |
| **Unchecked bounds / out-of-range** | sequence/index/precision ops not guarded by the never-silent `Option`/`Result` (RFC-0001) | IR dataflow → `Proven` where a guard is provably absent, else `Empirical` |
| **`deny-unknown-fields` gaps** | deserialization surfaces accepting unmodelled input | structural schema check → `Exact`/`Empirical` |
| **Depth-unguarded recursive descent** | recursive parsing/walking over untrusted input (stack/DoS) | IR call-graph → `Empirical` (a heuristic recursion-without-bound pattern) |
| **Ambiguous-encoding hash collisions** | content-addressing inputs where distinct values can share an encoding | semantic check against the canonical-encoding rule (RFC-0001 §4.6) → `Proven`/`Empirical` |
| **Taint / injection on the `wild` capability** | data flowing from the `wild` (untrusted) capability into a sink | IR taint-flow → `Proven` taint path vs `Empirical` pattern, **disclosed which** |
| **Supply-chain (content-hash DAG)** | a dependency's content-hash DAG (DN-28) carrying a known-vulnerable node | catalog lookup (§4) + VEX applicability → `Exact` match, `Declared`/`Empirical` exploitability |

Each finding states **which class** (closed enum or extension id), its **CWE**, and an **honest confidence
tag** — never a bare "vulnerable". A class the scanner does **not** cover is reported as *uncovered*, not as
*clean* (G2: absence of a finding in an unrun class is never silent "safe").

> *Design position (`Declared`).* The specific detection algorithm per class — and exactly which tag ceiling
> each can honestly reach in v0 — is implementation work under E22-1; the **table states the design intent**,
> not a measured capability.

## §3 Reporting Standard — SARIF + CWE + OSV + VEX, versioned-pinned schemas (D2)

Findings are emitted in a **machine-consumable standard** so automated scanning, CI gates, IDE surfacing,
and ecosystem feeds consume them without bespoke parsing:

- **SARIF** — the de-facto static-analysis interchange format (tool / IDE / CI integration).
- **CWE** — the *class* of weakness per finding.
- **OSV** (OSV-Schema) — the cross-ecosystem advisory identifier/record where an advisory exists (the
  *instance*); CVE ids are carried through OSV's alias set.
- **VEX** — an applicability statement: *is this content-addressed phylum/nodule version actually
  affected/exploitable?* Content-addressing (RFC-0001 §4.6) makes "which exact versions are affected"
  **precise**, not guessed.

Every finding carries **provenance + a confidence tag** on the lattice (`Exact ⊐ Proven ⊐ Empirical ⊐
Declared`, RFC-0034): a *proven* taint-flow vs a *heuristic* pattern-match is disclosed, never conflated.

**Decision (Q2 → D2): VERSIONED PINNING.** Each emitted report pins the **exact schema version** it
conforms to (SARIF x.y, OSV-Schema x.y, the Mycelium finding-content schema x.y). **A pinned schema version
is immutable once pinned** — a consumer that validated against `finding-schema/1.2` can rely on
`finding-schema/1.2` meaning the same thing forever. **New versions are allowed** (the schema evolves), but
evolution is **additive-by-new-version**, never an in-place mutation of a pinned one. This is the
append-only discipline (house rule #3) applied to wire schemas: a report's schema pin is a stable contract,
and a tampered/renumbered schema is a never-silent failure (the version pin won't validate). The Mycelium
finding-content schema is itself **content-addressed** (RFC-0001 §4.6), so "pinned + immutable" is
mechanically enforced, not merely promised.

## §4 Registry Integration — a second content-addressed catalog (DN-28)

A finding is reported to **two sinks**, not one:

1. **The CLI report** (local) — the SARIF/CWE/OSV/VEX output (§3) for the developer running the scan.
2. **The registry** — the **same registry that hosts packages** (phyla + nodules; DN-28) **also hosts the
   security findings.** A vulnerability found once is shared with the ecosystem there: *find once, report to
   the community* — the discovery cost is paid once, the benefit is ecosystem-wide.

**Screened / anonymized / privatized disclosure.** What the registry hosts is **not** the victim's raw
vulnerable source — it is a **screened** finding: the vulnerable logic is **anonymized/privatized** (secrets
+ PII stripped, proprietary specifics removed, **minimized to the essential vulnerable *pattern***) before
publication. Content-addressing makes this precise: the screened pattern is itself **content-addressed to a
fingerprint**, so *other* scans match the same vulnerable pattern across phyla **without ever seeing the
original source** — detection + mitigation propagate ecosystem-wide while the specific instance stays
private. *Disclose enough to defend, never enough to weaponize.* (The screening **policy** — what
minimization is safe to publish, and who approves it — is §6 / D4.)

**The hosted finding records** (the content model, machine-consumable per §3):

- **What** — the vulnerability class (CWE) + the **screened/minimized** vulnerable-logic pattern
  (anonymized) + its content-addressed fingerprint.
- **Severity** — a CVSS-style score / severity tier.
- **Affected** — precisely which content-addressed phylum/nodule versions (the DN-28 content-hash DAG + a
  VEX applicability statement).
- **Mitigation options that retain the logic** — one or more fixes (§5), each carrying an honest tag that it
  preserves intended behaviour while removing the vuln (the RFC-0002 certificate), so a consumer can review
  or auto-apply.
- **Provenance + confidence** — an honest tag on the finding itself (proven taint-flow vs heuristic, §3).

**Reconstruction-on-render — DN-28's model, reused.** Exactly as the package registry stores a phylum's
**content-hash DAG + manifest** (the dense, verifiable *map*) and reconstructs source bytes from a forge /
object store on use, the **security catalog stores only the findings' hashes + manifest** — the
fingerprints, the severity + affected-version index, the DAG — **not** the heavy finding bodies inline. The
full finding (screened pattern, description, mitigations) is **fetched + hash-verified from the content
store and reconstructed on render.** So the security catalog is **as cheap to host as the package
registry**, and because every finding is content-addressed, a published advisory is **tamper-evident**:
reconstruction verifies it against its hash, so a poisoned or silently-altered advisory **fails the check**
(never-silent integrity, G2). The registry thus hosts **two content-addressed catalogs of the same shape** —
packages **and** findings — both lightweight and reconstructed-on-render.

> *Design position (`Declared`).* The catalog data model, the publish/screen/approve workflow, and the
> embargo/coordinated-disclosure governance are implementation work under E22-1 + the DN-28 registry
> evolution. The **shape is decided** (two content-addressed catalogs, reconstruction-on-render, screened
> entries); the concrete schema and workflow are not yet built.

## §5 Safe Auto-Fixes — honest, certificate-backed, per-class strength (D3, D5)

The toolkit can **auto-apply a safe fix** that **retains the intended logic** while **eliminating or
mitigating** the vulnerability — but **never silently.** The keystone: *"the fix preserves intended
behaviour"* is a **refinement / translation-validation** claim — the **same problem RFC-0002's swap
certificate already solves** (prove artifact B refines reference A under a relation, within a bound). So a
safe-fix ships a **certificate** with two obligations:

- it **eliminates/mitigates** the vulnerability (the security property now holds), **and**
- it **refines the original modulo the vulnerability** (behaviour preserved on the non-vulnerable paths),

each carrying an **honest guarantee tag** (RFC-0034 / VR-5): `Proven` (machine-checked
semantics-preserving + vuln-eliminated), `Empirical` (differentially tested over a corpus), or `Declared`
(asserted, **always flagged** — **never** auto-applied without explicit human acceptance). A fix is
**reified + `EXPLAIN`-able** (RFC-0005 / RFC-0013): the developer sees *what changed, why it's safe, and at
what strength* before accepting. **No black-box "trust me" rewrites** — a fix that cannot state its strength
is `Declared` and gated behind explicit human acceptance (G2 / VR-5).

**Decision (Q3 → D3): per-class fix-strength, with a pedantic mode.** Each vulnerability class carries its
**own minimum fix-requirements.** Higher-severity / higher-security classes **enforce stricter checks** —
e.g. a taint/injection fix on the `wild` capability may *require* `Proven` (or human acceptance) before
auto-application, where a lower-risk lint-grade fix may auto-apply at `Empirical`. A **pedantic mode** raises
every class's bar (e.g. requiring `Proven` across the board, refusing `Empirical` auto-application), for
developers who want the strictest posture. The per-class bar and the pedantic ceiling resolve through the
same scoped mechanism as the scan profile (§7). The bar is **never lowered silently**: weakening a class's
fix-strength is a configuration event, surfaced and EXPLAIN-able.

**Decision (Q5 → D5): the CERTIFIED PATCH REGISTRY.** The strongest fixes — those carrying a
machine-checked (`Proven`) or corpus-tested (`Empirical`) RFC-0002 certificate — populate a **certified
patch registry**, a catalog of *vetted, certificate-backed* fixes keyed (like everything else) by the
content-addressed fingerprint of the vulnerable pattern they remediate. A consumer hitting a known
vulnerable pattern can pull the certified patch, **re-verify its certificate locally** (the certificate
travels with the patch; trust is re-established, not assumed), and apply it. This is the *find-once-fix-once*
twin of §4's *find-once-report-once*. The certified patch registry is **a native toolchain feature** — and,
per D5, the `/security-review` skill's relationship to all of this is that it is a **supporting consumer**,
not a replacement for or a prerequisite to a formal review: the toolkit provides detection, suggested fixes,
and the patch registry; formal review still happens, now better-informed.

> *Design position (`Declared`).* The RFC-0002 certificate reuse is a **design intent**: which fix-classes
> can honestly reach `Proven` vs `Empirical` in v0 is open (DN-30 §7 Q3's residual — the *bar per
> fix-class*), and the worked example that would demonstrate a real safe-fix certificate end-to-end is
> **deferred** (§9), not fabricated here.

## §6 Screening Policy — configurable defaults, mandatory for high-security classes (D4)

**Decision (Q4 → D4): the screening policy is CONFIGURABLE with sensible defaults, and MANDATORY for
high-security classes by default, per-project adjustable.** Before any finding is published to the registry
(§4), it passes a **screening** that minimizes/anonymizes it to the safe-to-publish vulnerable pattern. The
policy governing that screening:

- ships with **sensible defaults** (strip secrets + PII, remove proprietary specifics, minimize to the
  essential pattern, content-address the result);
- is **mandatory by default for high-security classes** (e.g. taint/injection on `wild`, `unsafe`/FFI
  misuse, supply-chain) — these **cannot** be published without passing the screen, and the default cannot
  be silently disabled for them;
- is **per-project adjustable** — a project may tighten the policy (more aggressive minimization, an
  approval gate) or, for lower-risk classes, relax it within bounds — but every adjustment is **explicit and
  surfaced** (a project that loosens screening for a class does so visibly, never silently — G2).

Disclosure governance (embargo windows, coordinated disclosure, who reviews/approves a screening before it
lands) is the workflow layer over this policy; its concrete process is implementation work under E22-1. The
**default-and-mandatory-for-high-security** posture is decided; the approval workflow is not yet built.

## §7 Native + Scoped — RFC-0034 §6 resolution (project / nodule / granular)

Like the testing toolkit (RFC-0034 §13 / M-796), the security toolkit is **native and scope-configurable**,
**reusing the RFC-0034 §6 `@certification`-style resolution** (project / phylum / nodule / granular,
most-specific-wins via RFC-0012 ambient scoping — **no new scoping machinery**):

- a **project** sets a baseline scan profile (which classes are on, which severity gate, the per-class
  fix-strength bar of §5, the screening policy of §6, pedantic on/off);
- it is **overridable per-nodule or per-construct** (granular) — e.g. a nodule handling untrusted input
  raises its taint-class bar; a vetted internal nodule relaxes a lint-grade class.

Downstream developers get automated detection + standard reporting + honest fixes **for free**,
configurable, **never forced** — the §7-ergonomic *give-the-tool-and-the-dial* stance applied to security.
The scope-resolution is the *same mechanism* RFC-0034 already defines; this RFC adds **security profile
fields** to it, not a new resolver.

## §8 Honest tags & never-silent (the transparency contract, applied)

Restating the contract every section above leans on, in one place (VR-5 / G2):

1. **Every finding is tagged at its real strength.** A heuristic pattern-match is `Empirical`/`Declared`; a
   discharged taint-flow or absent-guard proof is `Proven`; an exact site enumeration is `Exact`. The
   scanner **never upgrades** its own claim past what it computed.
2. **Absence of a finding is never silent "safe".** An unrun or uncovered class is reported *uncovered*, not
   *clean*.
3. **No fix is applied without stating its strength.** A `Declared` fix is always flagged and human-gated;
   auto-application is reserved for the strengths a class's bar (§5 / D3) permits.
4. **Every advisory is tamper-evident.** Content-addressing (§4) makes a silently-altered advisory fail
   reconstruction.
5. **Every configuration weakening is surfaced.** Relaxing a scan profile, a fix-strength bar, or a
   screening policy (§5–§7) is an explicit, EXPLAIN-able event, never a silent downgrade.

## §9 Definition of Done

This RFC is **Accepted** when:

- the design (§2–§7) is ratified (Proposed → Accepted, stepped — house rule #3), and the five Decisions
  (§10) are recorded as the binding answers to DN-30 §7;
- the two **worked examples** below are written (they are the **remaining pre-Accepted work**, deliberately
  **not** fabricated in this draft — VR-5 / G2):
  - **(WE-1) A safe-fix refinement-certificate worked example** — one concrete vulnerability (e.g. an
    unchecked-bounds or a taint/injection case), its safe fix, and the **actual** RFC-0002-style certificate
    obligations discharged end-to-end (the vuln-eliminated obligation **and** the refinement-modulo-the-vuln
    obligation), with the honest strength the example genuinely earns. This demonstrates §5's claim is
    realizable rather than merely asserted.
  - **(WE-2) A screening case study** — one real-shaped finding taken from raw vulnerable logic through the
    §6 screening to a published, content-addressed, anonymized pattern, showing the minimization is
    *sufficient to detect + mitigate* and *insufficient to weaponize or leak the source*. This demonstrates
    §4/§6's screening is realizable.

The toolkit's **implementation** (the scanner, reporter, fix-certifier, and the two catalogs) is **E22-1**,
**not** in scope for this RFC's Accepted gate — this RFC designs; it does not implement (VR-5 / G2). The
E22-1 epic Definition of Done (in `tools/github/issues.yaml`) governs the implementation gate.

## §10 Decisions (the five answered open questions)

Each restates a DN-30 §7 open question, the maintainer's 2026-06-24 answer, and tags the decision's strength.

| # | DN-30 §7 question | Decision (2026-06-24) | Strength |
|---|---|---|---|
| **D1** | Q1 — which vuln classes does v0 detect? | A **fixed base** of categories (§2 table) **with an extensibility seam**; an extension class is first-class + versioned, never silently folded into the base. | `Declared` (design decision; per-class detection capability is implementation-measured) |
| **D2** | Q2 — the reporting standard set + schemas? | SARIF + CWE + OSV + VEX, with **versioned pinning**: a pinned schema version is **immutable once pinned**; new versions are allowed (additive-by-new-version, append-only). | `Declared` (decided; the schemas are external standards cited as prior art, §11) |
| **D3** | Q3 — auto-fix verification depth / bar per class? | **Per-class fix-strength**: each class has its own minimum requirements; higher-security classes enforce stricter checks; a **pedantic mode** raises every bar. | `Declared` (the *which-class-reaches-which-strength* bar is residual implementation work — DN-30 §7 Q3) |
| **D4** | Q4 — disclosure governance + screening policy? | **Configurable with sensible defaults**; **mandatory by default for high-security classes** (cannot be silently disabled for them); per-project adjustable, every adjustment surfaced. | `Declared` (policy posture decided; the approval/embargo workflow is implementation work) |
| **D5** | Q5 — relationship to `/security-review`? | A **supporting tool only** — not a replacement for or a prerequisite to a formal review; a native toolchain feature for **detection, suggested fixes, and a certified patch registry**. | `Declared` (relationship decided; the skill's consumption surface is implementation work) |

## §11 Prior Art

- **OSV / OSV.dev + OSV-Schema** — the cross-ecosystem vulnerability database + schema this RFC adopts for
  the advisory identifier/record (§3). Mycelium's contribution is the **content-addressed, screened,
  reconstruction-on-render** hosting (§4), not the schema.
- **GHSA (GitHub Security Advisories)** — coordinated-disclosure advisory model; informs §6's governance
  layer (not yet designed in detail).
- **RustSec / `cargo-audit` / `cargo-deny`** — the supply-chain advisory + lockfile-audit model Mycelium
  already uses CI-side (`scripts/checks/deny.sh`); the §2 supply-chain class generalizes it over the DN-28
  content-hash DAG.
- **`npm audit` / the npm advisory feed** — the ecosystem-feed-of-findings model the registry catalog (§4)
  parallels, distinguished by content-addressing + screening.
- **SARIF (OASIS)** and **CWE (MITRE)** — the interchange format and weakness taxonomy adopted verbatim (§3).

These are cited as the **prior art the design builds on**; none is claimed as implemented in Mycelium.

---

> **Provenance.** Binds **DN-30** (the direction capture) + the maintainer's 2026-06-24 answers to its five
> §7 open questions (§10). Reuses, unchanged, the RFC-0001 inspectable Core IR + content-addressing, the
> RFC-0002 refinement certificate + checker, the RFC-0034 §6 mode resolution + honest-tag lattice, the
> RFC-0013/0014 diagnostics/recovery machinery, and the DN-28 reconstruction-on-render distribution model.
> External standards (SARIF, CWE, OSV, VEX) and feeds (OSV.dev, GHSA, RustSec, npm) are prior art (§11),
> not Mycelium artifacts. **Designs the toolkit; implements nothing** — every runtime-behaviour claim is a
> `Declared` position for E22-1 to discharge; the two worked examples (§9) are deferred pre-Accepted work,
> not fabricated (VR-5 / G2).

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-24 | **Proposed** | Initial proposal — the binding security-scanning design lifted from the settled **DN-30** capture (§2–§6 principles) plus the maintainer's answers to DN-30 §7's five open questions (§10 D1–D5): v0 vuln classes as a **fixed base + extensibility seam** (§2); SARIF+CWE+OSV+VEX reporting with **versioned-pinned, immutable-once-pinned** schemas (§3); the registry as a **second content-addressed catalog** reusing DN-28 reconstruction-on-render, hosting **screened/anonymized** advisories (§4); honest, RFC-0002-certificate-backed **safe auto-fix** with **per-class fix-strength + pedantic mode** and a **certified patch registry** (§5); **screening policy** configurable-with-defaults, mandatory-for-high-security (§6); native + scoped reusing RFC-0034 §6 resolution (§7). **Designs the toolkit; implements nothing** — every runtime claim is `Declared` for E22-1 to discharge; the two **worked examples** (safe-fix refinement-cert; screening case study) are deferred pre-Accepted work, **not fabricated** (§9, VR-5/G2). Anchor: DN-30; backlog: E22-1. |
