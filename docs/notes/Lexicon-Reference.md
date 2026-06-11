# Mycelium Language — Lexicon Reference

**Version:** 0.4  
**Status:** Draft for Review & Ratification  
**Date:** 2026-06-10  
**Maintainer:** Tyler Zervas  
**Architectural Review:** Grok (Software Engineering Lead Architect)

## 1. Purpose & Design Principles

This document is the single source of truth for terminology in the Mycelium programming language.

**Primary Goals**
- Maximize long-term learnability in a language that is inherently complex (multiple first-class representations, certified representation changes, honest guarantees, VSA integration, verified numerics, and content-addressed definitions).
- Provide strong, consistent **mnemonics** that link fungal concepts to real software engineering behaviors.
- Use **short, distinctive abbreviations** for frequent use in code while preserving full terms for documentation and mental models.
- Clearly separate concerns through a three-layer model so developers are not overwhelmed.

### Vocabulary Tiers

> **Note (terminology de-confliction, per ADR-012 §7.1):** these vocabulary **tiers** are named
> **Surface / Runtime / Formal** — *not* "L1/L2/L3", which RFC-0006 §3 already uses for the
> language **layers** (L0 Core IR → L1 kernel calculus → L2 surface → L3 projection). The two are
> orthogonal; reusing the L-numbers inverted them and was a collision. Tiers below.

| Tier        | Purpose                              | Theming Approach                  | Typical User                  | Example Terms                  |
|-------------|--------------------------------------|-----------------------------------|-------------------------------|--------------------------------|
| **Surface** | Daily surface syntax & keywords      | Conservative (three-test gate)    | All developers                | `colony`, `consume`, `grow`    |
| **Runtime** | Runtime primitives & architecture    | Themed, name-stable (DN-03)       | Systems & runtime developers  | `hypha`, `fuse`, `xloc`, `cyst`|
| **Formal**  | Formal semantics & Core IR           | Technical / conventional          | Language designers & compilers| `Repr`, `GuaranteeStrength`    |

> **Status & grounding:** the **Surface** tier extends the Resolved DN-02 set; the additions
> were ratified through the three-test gate in **DN-03** — `consume` and `grow` adopted,
> `embody` **declined** (inherent methods keep `impl`), `for` reserved (RFC-0007 §4.8).
> The **Runtime** tier now has a **drafted model**: RFC-0008 (Runtime & Concurrency Execution
> Model, Draft), grounded by research Pass-4 (`research/04-runtime-concurrency-RECORD.md`,
> T4.1–T4.6), defines each term's operational meaning (RFC-0008 §4.5) under the RT1–RT7 runtime
> invariants. The vocabulary remains *reserved, name-stable — **not active syntax*** until DN-03
> ratifies the names through the three-test gate and implementation RFCs land (RFC-0008 §4.5
> status rule). The **Formal** tier is normative (RFC-0001).

**Naming Rules**
- **Short form** (3–5 characters preferred): Used in source code.
- **Full term**: Used in documentation, comments, error messages, and when first introducing a concept.
- Abbreviations must be mnemonic and must not collide with common meanings in Rust, Python, C, Go, or other widely used languages.

---

## 2. Surface Tier: Keywords & Syntax

### Themed Terms (Ratified)

| Short Form | Full Term   | Definition | Mnemonic & Rationale | Behaviors | Example |
|------------|-------------|------------|----------------------|-----------|---------|
| `colony`   | Colony      | A bounded, self-sustaining unit of definitions and behavior. | A fungal colony is a living network of hyphae working together. | Owns definitions, controls visibility, participates in the larger network. | `colony math { fn add(...) }` |
| `spore`    | Spore       | An immutable, verifiable, self-contained artifact of code + state + metadata that can germinate into execution. | A biological spore carries everything needed to regrow the organism. | Content-addressed, signed, dispersible. Supports lightweight and durable variants. | `let s = spore { ... }; germinate(s)` |
| `consume`  | Consume     | Acquire and take exclusive ownership of a linear/affine resource. | A fungus consumes substrate exactly once to grow. | Enforces single-use semantics via the type system. | `let res = consume(substrate);` |
| `grow`     | Grow        | Automatically generate or extend behavior on a type. | The system *grows* new capabilities, like fungal extension. | Used for derive-like and generative features. | `grow Debug for Point;` |
| `wild`     | Wild        | Lexically marked, denied-by-default region for unsafe/raw operations. | Growth that has left the safe, cultivated colony. | Only place raw FFI or manual memory is allowed. | `wild { foreign_call(...) }` |
| `matured`  | Matured     | Marks a definition as stable, verified, and eligible for AOT compilation. | The component has grown into a hardened, persistent form. | Used for promotion to stable components. | `matured fn critical_path(...)` |

> **`spore` scope — resolved (ADR-013 + RFC-0003 r2, 2026-06-10):** `spore` is the
> **content-addressed deployable unit** (code + values + metadata); the RFC-0003 §6
> reconstruction manifest is **one digest-referenced component**, and `spore(v)` constructs the
> degenerate single-value spore (the manifest for `v`). The schema name stays
> `reconstruction-manifest`. `substrate` (the affine resource kind, DN-02) is the type `consume`
> operates on.

### Conventional Terms (Retained for Learnability)

`let`, `fn`, `type`, `trait`, `use`, `match`, `if` / `else`, `for`, `impl`, `pub`, `self`,
`where`, `Result` / `Option` family, `swap`, `policy`, `total` / `partial`.

> **Inherent methods = `impl` (DN-03):** the themed `embody` was evaluated and **declined** —
> `impl` is machine-/human-familiar and theming costs dual readability for no behavioral teaching
> (same logic as `trait`/`type`). Methods are not in the v0 grammar yet; `impl` is the chosen
> term for when they land.
>
> **Control flow (RFC-0007 §4.8 r2; DN-03):** `for` is **reserved** — the keyword of bounded-
> iteration sugar (a `for`-fold that elaborates to structural recursion, `Total` by construction;
> *provisional spelling*, KC-2-gated). `while` / `loop` / `break` / `continue` / `return` stay
> **excluded and unreserved** (unbounded iteration would undermine the divergence bit); the
> toolchain emits teaching diagnostics where they appear, pointing at recursion / `for`.

---

## 3. Runtime Tier: Runtime Primitives & Architectural Concepts

> **Reserved vocabulary, not active syntax — grounded (RFC-0008) and name-ratified (DN-03).**
> These primitives describe the concurrency/distribution execution model defined by **RFC-0008**
> (grounded by research Pass-4, T4.1–T4.6): each term's operational meaning and the runtime
> invariants (RT1–RT7) it must respect are in RFC-0008 §4.5. **DN-03 ratified the names** through
> the three-test gate against those meanings (`anas`→`fuse`, `cmn`→`mesh`, `myco`→`graft`,
> `sclrt`→`cyst`/`encyst`). They remain *reserved* — activation requires each construct's
> RFC-0008 implementation-stage RFC (R1/R2). Each term is **one canonical name + at most one
> sanctioned alias** (§ "The alias rule" below): same content-addressed definition, same hash.

These primitives form the execution model and distributed-systems substrate (RFC-0008).
**Canonical** is the name docs lead with; **Alias** is the DN-03-ratified single short synonym
(or — when the canonical is already short — *none*).

| Canonical | Alias | RFC-0008 meaning (RT invariants) |
|------------------------|----------|---------------------------------------------------|
| `hypha`                | *(none)* | structurally-scoped concurrent computation over immutable values (RT1/RT2/RT7) |
| `anastomose`           | `fuse`   | lawful state fusion: semilattice merge, meet-composed `Meta` (RT6) |
| `translocate`          | `xloc`   | explicit, fallible, `Meta`-preserving value movement with backpressure (RT1/RT4) |
| `sclerotium`           | `cyst` (verb `encyst`) | content-addressed checkpoint of a dormable computation — a cyst *is* a dormant resumable form (RT2) |
| `mycorrhize`           | `graft`  | capability contract with external infra (the capability is an affine `substrate`) (RT4) |
| `forage`               | *(none)* | adaptive placement/discovery as a reified RFC-0005 policy — the third site (RT3) |
| `rhizomorph`           | *(none)* | a declared/promoted transport path; a placement-policy artifact, semantics-free (RT3) |
| `mycorrhizal-network`  | `mesh`   | gossip/pub-sub overlay with honest **probabilistic** guarantees (δ) (RT5) |
| `dimorph`              | *(none)* | execution-mode switch: tiering = RFC-0004 `ExecutionMode`; repr switch = `Swap` (S1) |
| `reclaim`              | *(none)* | supervision-tree reclamation of *runtime units* — **never memory** (LR-9) (RT7) |

> **The alias rule (DN-03 §3).** Each themed term has **one** canonical name and **at most one**
> sanctioned alias — never a family of forms, and *zero* when the canonical is already short and
> pronounceable (`hypha`, `forage`). Because identity is content-addressed (ADR-003) and one
> canonical formatter (M-142, S3) renders it, the canonical and the alias denote the *same
> definition* (same hash): beginners read `anastomose`, experts type `fuse`, at zero identity
> cost. Where the alias is a *synonym* rather than an abbreviation, the two are different words
> bound to one hash — same meaning, not "one word projected twice". The reserved-word set holds
> both spellings.

---

## 4. Formal Tier: Formal / IR / Semantic Concepts

These concepts come primarily from RFC-0001 and remain largely technical.

- **`Repr`** — Representation family (`Binary`, `Ternary`, `Dense`, `VSA`).
- **`GuaranteeStrength`** — Lattice: `Exact` ⊃ `Proven` ⊃ `Empirical` ⊃ `Declared`.
- **`Bound`** — Quantitative bound (`ErrorBound`, `ProbabilityBound`, etc.) with `BoundBasis`.
- **`Swap`** — The only operation allowed to change `Repr`. Always emits a certificate.
- **`Meta`** — Self-describing metadata attached to every value (provenance, guarantee, bound, physical layout, reconstruction info).
- **Content-Addressed Identity** — Definitions are identified by hash of structure + types + static contract.
- **`Stable Component`** — A definition that is content-addressed, verified, and eligible for ahead-of-time compilation (`matured`).

---

## 5. Usage Guidelines

- **General development**: stay in the Surface tier + conventional keywords.
- **Systems programming & runtime work**: use the Runtime short aliases (once activated).
- **Documentation & diagnostics**: introduce a concept with the canonical long form followed by
  its short alias in parentheses on first use.
- **Error messages**: prefer clarity — e.g., "Cannot `fuse` (anastomose) incompatible `hypha`
  types."

---

*End of Lexicon Reference v0.4 (DN-03-amended 2026-06-10).*
