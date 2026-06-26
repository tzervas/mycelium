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

## §4 Find-once, report-to-community — the registry as a security-advisory host

A finding is reported to **two sinks**, not one:

1. **The CLI report** (local) — the SARIF/CWE output (§3) for the developer running the scan.
2. **The registry** — the **same registry that hosts packages (phyla + nodules; DN-28) ALSO hosts the
   security findings.** A vulnerability found once is **shared with the ecosystem** there: the registry is
   the security-advisory database, not just a package host.

*Find once, report to the community* — the discovery cost is paid once and the benefit is ecosystem-wide.

**Screened / anonymized / privatized disclosure (the keystone of §4).** What the registry hosts is **not**
the victim's raw vulnerable source — it is a **screened** finding: the vulnerable logic is
**anonymized/privatized** (secrets + PII stripped, proprietary specifics removed, **minimized to the
essential vulnerable *pattern***) before it is published. Content-addressing makes this precise: the screened
pattern is itself **content-addressed to a fingerprint**, so *other* scans can match the same vulnerable
pattern across phyla **without ever seeing the original source** — detection + mitigation propagate
ecosystem-wide while the specific instance stays private. Disclose enough to defend, never enough to weaponize.

**The hosted finding records** (the content model, machine-consumable per §3):
- **What** — the vulnerability class (CWE) + the **screened/minimized** vulnerable-logic pattern (anonymized)
  + its content-addressed fingerprint.
- **Severity** — a CVSS-style score / severity tier.
- **Affected** — precisely which content-addressed phylum/nodule versions (the DN-28 content-hash DAG +
  a VEX applicability statement — is this version actually exploitable?).
- **Mitigation options that retain the logic** — one or more fixes (§5), each carrying an **honest** tag that
  it preserves intended behaviour while removing the vuln (the RFC-0002 refinement certificate), so a
  consumer can review or auto-apply.
- **Provenance + confidence** — an honest tag on the finding itself (proven taint-flow vs heuristic, §3).

**The catalog is lightweight — reconstruction-on-render (DN-28's model, reused).** Exactly as the package
registry stores a phylum's **content-hash DAG + manifest** (the dense, verifiable *map*) and reconstructs the
source bytes from a forge / object store on use, the security catalog stores only the findings' **hashes +
manifest** — the fingerprints, the severity + affected-version index, the DAG — **not** the heavy finding
bodies inline. The full finding (screened pattern, description, mitigations) is **fetched + hash-verified
from the content store and reconstructed on render**. So the security catalog is **as cheap to host as the
package registry** (the same reconstruction-based distribution), and because every finding is
content-addressed, a published advisory is **tamper-evident**: reconstruction verifies it against its hash,
so a poisoned or silently-altered advisory **fails the check** (never-silent integrity, G2). The registry
thus hosts **two content-addressed catalogs of the same shape** — packages **and** findings — both
lightweight and reconstructed-on-render.

(Disclosure governance — embargo windows, coordinated disclosure, the **screening policy** that decides what
is safe to publish — is an explicit open question, §7.)

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
4. **Disclosure governance + screening policy.** Embargo / coordinated-disclosure for the community feed,
   and — the harder one — the **screening/anonymization policy** (§4): what minimization makes a hosted
   vulnerable-logic pattern safe to publish (detectable + mitigable, but not weaponizable and not leaking
   the victim's proprietary source)? Who reviews/approves a screening before it lands in the registry?
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
> shippable, native, honest toolchain component). *2026-06-24 (rev.)* — §4 expanded: a finding reports to
> **two sinks** (CLI + the registry); the **registry hosts security findings alongside packages** (DN-28),
> as **screened/anonymized/privatized** entries (vulnerable logic minimized to a content-addressed pattern
> fingerprint — detectable + mitigable ecosystem-wide without exposing the victim's source); the hosted
> finding content model (what / severity / affected content-addressed versions / logic-retaining mitigations
> / honest confidence) is stated; §7 Q4 gains the **screening policy** governance question. *2026-06-24
> (rev. 2)* — §4: the security catalog **reuses DN-28's reconstruction-on-render model** — the registry
> stores the findings' hashes + manifest (the dense, verifiable map) and reconstructs + hash-verifies the
> finding bodies on render, so the catalog is as **lightweight** as the package side and a tampered advisory
> fails its hash (never-silent integrity). The registry hosts two content-addressed catalogs of the same
> shape — packages and findings. Cross-ref in DN-28 §5 + E22-1. *2026-06-24 (rev. 3 — feeds RFC-0035)* —
> the binding design this note shaped is now drafted as **RFC-0035 — Security Scanning Toolkit**
> (`docs/rfcs/RFC-0035-Security-Scanning-Toolkit.md`, **Proposed**), which lifts §2–§6 and **decides the
> five §7 open questions** per the maintainer's 2026-06-24 answers (D1 fixed-base-classes + extensibility ·
> D2 SARIF/CWE/OSV/VEX with versioned-pinned immutable schemas · D3 per-class fix-strength + pedantic mode ·
> D4 configurable-default screening, mandatory for high-security classes · D5 `/security-review` as a
> supporting tool only + a certified patch registry). This note stays **Draft** (append-only direction
> record); RFC-0035 carries the binding design and moves toward `Accepted` (its two worked examples are the
> remaining pre-Accepted work). Append-only — DN-30's existing prose is unchanged (VR-5/G2).
