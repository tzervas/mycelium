# ADR-010 — Verified-Numerics Foundation

| Field | Value |
|---|---|
| **ADR** | 010 |
| **Status** | **Accepted** (ratified on the follow-up research pass; the bundling probe T0.2 remains the one confirming experiment — see "Remaining confirmation") |
| **Date** | June 08, 2026 |
| **Context refs** | RFC-0001 §4.7 (bound composition); survey Area 5; Research Findings **T0.1**, **T0.2** |
| **Supersedes** | ADR-010 (Proposed) |
| **Blocks (now unblocks)** | RFC-0001 §4.7; RFC-0002 (swap bounds); RFC-0003 (VSA bounds) |

## Context

Mycelium needs (a) sound *bounds* on approximate operations — error magnitude (ε), crosstalk, failure probability (δ) — and (b) a way to *compose* them and tag them by guarantee strength. The research pass (T0.1) assessed the candidate foundations against Mycelium's actual needs and reached a clear verdict.

## Decision

**Two bound kernels meeting at one shared certificate.** A single composition algebra does **not** unify ε-magnitude and δ-probability (T0.1c — settled negative result for now): error magnitudes compose through interval/affine/Taylor arithmetic; failure probabilities compose through couplings/union-bound and pre-expectation calculi. These are different monoids.

1. **`ErrorBound` kernel (ε).** Core semantic domain = **affine arithmetic** (sound, compositional, correlation-aware — the sweet spot for "weakest-wins meet over composition"). Design borrowed from **Daisy/Rosa** (real-valued spec → synthesized certified bound) and **FloVer** (the first tool with *formalized* affine arithmetic; a verified checker in Coq/HOL4 with a CakeML-extracted binary). Handles float-`dtype`-precision concerns directly.
2. **`ProbBound` kernel (δ).** Start with the **union bound** (lightest; natural for "decode succeeds w.p. ≥ 1−δ" and "P(any of N retrievals fails) ≤ Σ failure probs"). Reserve **apRHL**-style approximate couplings (judgments carry `⟨ε,δ⟩`; the `[SEQ]` rule composes multiplicatively in ε, additively in δ) for *relational* reference-vs-implementation certificates, and the **weakest-pre-expectation** calculus (Kaminski/Katoen) for quantitative/expected-value invariants.
3. **Shared certificate.** Both kernels reduce to one record `{ε, δ, strength ∈ {Exact, Proven, Empirical, Declared}}`. Composition is per-kernel; **`strength` composes by meet** (weakest wins), exactly as RFC-0001's lattice requires.
4. **One sanctioned cross-kernel inference**: the accuracy→probability coupling (the aHL∘apRHL pattern — "privacy/failure depends on accuracy"), i.e. an `ErrorBound` may feed a `ProbBound`. No other cross-kernel mixing.
5. **VSA crosstalk content is NOT supplied by these tools.** The verified-numerics machinery is the right *pattern and home* (exact spec + certified bound + checkable certificate) and owns float round-off; VSA crosstalk/capacity *content* comes from the concentration-inequality literature (RFC-0003 / T0.2), tagged via the cited-theorem-with-checked-instantiation method.

## Trusted base

**Certificate-checker-in-Rust, not prover-hosting** (the dominant proven architecture — FloVer, CompCert/seL4 translation validation, Dandelion, Crellvm). Two tiers:
- **Tier i (pragmatic, default):** a Rust checker re-validates the cheap decidable core of each certificate (affine re-evaluation; union-bound arithmetic; SMT UNSAT-core replay), trusting Coq/HOL only for once-proved meta-theorems.
- **Tier ii (high-assurance):** link a CakeML/Coq-extracted verified checker (FloVer-style) as a separate verified binary, for the strongest `Proven` tags.

Hosting a full prover in-process is rejected — it bloats the trusted base.

## Consequences
- **RFC-0001 §4.7 is unblocked:** composed approximate results no longer default to `Declared`; they carry `Proven`/`Empirical` per the cited-theorem-with-checked-instantiation pattern, composed by the two kernels.
- One bound vocabulary across RFC-0001/0002/0003; the same certificate format the RFC-0002/0004 translation-validation checker consumes.
- Two kernels = more surface than one, accepted as inherent (T0.1c).

## Remaining confirmation
The make-or-break experiment (T0.2 / KC-1) is **favorable on the literature**: proven non-asymptotic bundling bounds exist (Clarkson-Ubaru-Yang 2023; Thomas-Dasgupta-Rosing 2021). The one confirming build is the Liquid-Haskell `bundle` instantiation (RFC-0003 §research-probe): encode a capacity refinement instantiating those theorems and confirm Z3 discharges the arithmetic. Success ratifies the whole "axiomatized-theorem + checked-instantiation" strategy; this ADR is Accepted on the expectation it will.

## Grounding
T0.1 (a–d), T0.2; survey Area 5; RFC-0001 §4.7 / guarantee lattice.
