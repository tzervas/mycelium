# Design Note DN-30 — Security Scanning Toolkit (automated detection · standard reporting · honest safe auto-fix)

| Field | Value |
|---|---|
| **Note** | DN-30 |
| **Status** | **Draft** (2026-06-24; direction capture — advisory, non-committal) |
| **Feeds** | a future **security-scanning RFC** (the binding design) + a toolchain-component epic (**E22-1**); extends the `/security-review` skill (manual lens → automated component); relates to RFC-0034 (transparency modes — findings carry provenance/auditability), RFC-0002 (the refinement/translation-validation certificate the safe-fix reuses), RFC-0013/RFC-0014 (structured diagnostics + declarative recovery — the reporting + fix machinery), RFC-0028/ADR-014 (the FFI/`unsafe` vuln surface), DN-28 (registry — the advisory feed home) |
| **Date** | June 24, 2026 |
| **Decides** | *Nothing normatively* — advisory. Records the maintainer's intended direction (2026-06-24): ship a **security scanning toolkit as a first-class toolchain component** that **automatically detects** security flaws (not solely from documented findings), **reports them in a programmatically-leverageable standard**, supports **find-once-report-to-community** disclosure, and can **auto-apply honest, semantics-preserving safe fixes**. Captures the principles + the open questions so the binding RFC is shaped toward this end-state. |
| **Task** | toolchain / security tooling (pre-RFC capture) |

> **Posture (transparency rule / VR-5).** Advisory, forward-looking. **Enacts nothing; moves no status;
> changes no normative text.** Records a *direction* and its grounding in Mycelium's existing machinery.
> The binding design is a future RFC; until then this is a backlog capture (E22-1). No code or guarantee is
> claimed here (VR-5 / G2).

## §1 The gap

Mycelium already has *CI-side* security hygiene — `.gitleaks.toml` (secrets), `scripts/checks/deny.sh`
(cargo-deny supply-chain), `just safety-check` (the RFC-0028 `unsafe`/FFI audit) — and a **manual**
`/security-review` lens (a human/agent reading a diff). What it lacks is a **shippable, Mycelium-native
security toolkit**: a toolchain component that **downstream developers inherit** (like the LSP, linter,
formatter, and the testing toolkit of RFC-0034 §13 / M-796), which **logically enumerates, finds, and
reports** vulnerability classes **automatically** — and can **fix** them honestly. This note captures that
component.

## §2 The principle — automated detection that doesn't *rely* on documented findings

The toolkit **independently** detects security flaws by **logical/semantic analysis over the inspectable
Core IR** (RFC-0001 — no black boxes; the IR is dumpable + analyzable), enumerating vulnerability classes
rather than only matching a list of *documented* findings. It **runs in conjunction** with documented
findings (the `/security-review` corpus, an advisory feed): if a documented finding exists it is correlated;
if the toolkit finds one **independently**, it is reported the same way. The toolchain becomes a
*first-party* source of vulnerability discovery, not a consumer of one.

This is the **never-silent rule (G2) applied to security**: a flaw the toolchain can see is **surfaced, never
hidden**; a swept-under-the-rug "probably fine" is exactly the silent behaviour Mycelium forbids.

## §3 Standard, programmatically-leverageable reporting

Findings are emitted in a **machine-consumable standard** so automated scanning, CI gates, IDE surfacing,
and ecosystem feeds can all consume them without bespoke parsing:

- **SARIF** for tool/IDE/CI integration (the de-facto static-analysis interchange format).
- **CWE** classification per finding (the *class* of weakness) + **CVE/OSV** identifiers where an advisory
  exists (the *instance*).
- A **VEX**-style applicability statement (is this phylum/version actually affected/exploitable?) — content
  addressing (RFC-0001 §4.6) makes "which exact versions are affected" *precise*, not guessed.

Every finding carries **provenance + a confidence tag** on the lattice (`Exact ⊐ Proven ⊐ Empirical ⊐
Declared`, RFC-0034): a *proven* taint-flow vs a *heuristic* pattern-match is **disclosed**, never
conflated — the transparency rule, applied to the scanner's own claims.

## §4 Find-once, report-to-community

A vulnerability found once in a phylum is **shared with the ecosystem**: the toolkit publishes to an
**advisory feed** (OSV-shaped) keyed by the phylum's **content-hash DAG** (DN-28's registry model), so
consumers are notified precisely which content-addressed versions are affected and which fix supersedes
them. *Find once, report to the community* — the discovery cost is paid once and the benefit is ecosystem-wide.
(Disclosure governance — embargo windows, coordinated disclosure — is an explicit open question, §7.)

## §5 Honest, semantics-preserving safe auto-fix

The toolkit can **auto-apply a safe fix** that **retains the intended logic** while **eliminating or
mitigating** the vulnerability — but **never silently**. The keystone: *"the fix preserves intended
behaviour"* is a **refinement / translation-validation** claim — the **same problem RFC-0002's swap
certificate already solves** (prove artifact B refines reference A under a relation, within a bound). So a
safe-fix ships a **certificate**:

- it **eliminates/mitigates** the vulnerability (the security property now holds), **and**
- it **refines the original modulo the vulnerability** (behaviour preserved on the non-vulnerable paths),

each carrying an **honest guarantee tag** (RFC-0034 / VR-5): `Proven` (machine-checked semantics-preserving
+ vuln-eliminated), `Empirical` (differentially tested over a corpus), or `Declared` (asserted, **always
flagged** — never auto-applied without review). A fix is **reified + `EXPLAIN`-able** (RFC-0005/RFC-0013):
the developer sees *what changed, why it's safe, and at what strength* before accepting. **No black-box
"trust me" rewrites** — a fix that cannot state its strength is `Declared` and gated behind explicit human
acceptance (G2/VR-5).

## §6 Native + scoped (a toolchain component, inherited)

Like the testing toolkit (RFC-0034 §13 / M-796), the security toolkit is **native and scope-configurable**,
reusing the §6 `@certification`-style resolution (project / nodule / granular, most-specific-wins via
RFC-0012 ambient scoping): a project sets a baseline scan profile (which classes, which severity gate),
overridable per-nodule or per-construct. Downstream developers get automated detection + standard reporting
+ honest fixes **for free**, configurable, never forced — the §7-ergonomic *give-the-tool-and-the-dial*
stance applied to security.

## §7 Open questions

1. **Vulnerability classes first.** Which classes does v0 detect? (Candidates grounded in Mycelium's own
   surfaces: `unsafe`/FFI misuse (ADR-014/RFC-0028), the recurring honesty-rule defect seams from the
   `/security-review` rubric — unchecked bounds, deny-unknown-fields gaps, depth-unguarded recursive
   descent over untrusted input, ambiguous-encoding hash collisions — taint/injection on the `wild`
   capability, supply-chain via the content-hash DAG.)
2. **The reporting standard set.** SARIF + CWE + OSV + VEX is the proposed baseline — confirm + pin schemas.
3. **Auto-fix verification depth.** How much of "refines the original" is `Proven` (checked) vs `Empirical`
   (corpus-tested) in v0? The RFC-0002 checker is the reuse vehicle; the bar per fix-class is open.
4. **Disclosure governance.** Embargo / coordinated-disclosure policy for the community feed (§4).
5. **Relationship to `/security-review`.** Does the skill become the *interactive front-end* to the
   automated toolkit, or stay a separate manual lens? (Lean: the skill consumes the toolkit's findings.)

## §8 Definition of Done (this note)

A future security-scanning **RFC** is shaped by this capture (the principles §2–§6 + the open questions §7),
and the toolchain-component **epic E22-1** carries the backlog. This note **enacts nothing** (VR-5/G2);
it is the direction record, superseded (append-only) by that RFC when the design settles.

---

> **Provenance.** Grounded in the existing security tooling (`.gitleaks.toml`, `scripts/checks/deny.sh`,
> `just safety-check`), the `/security-review` rubric's recurring-defect bank, and the reuse of RFC-0002
> (refinement certificate), RFC-0013/0014 (diagnostics/recovery), RFC-0034 (transparency modes + scoped
> resolution), RFC-0001 (inspectable Core IR + content addressing), and DN-28 (registry advisory feed).
> Advisory only; no normative claim (VR-5 / G2).
>
> **Revision history.** *2026-06-24* — initial Draft (direction capture; the security-scanning toolkit as a
> shippable, native, honest toolchain component).
