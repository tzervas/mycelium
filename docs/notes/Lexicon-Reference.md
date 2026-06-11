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

**Naming Rules** (DN-03 — one name per term, flat)
- **One name per concept.** No canonical/alias pairs; pick the single clearest name and stop
  (DN-03 §3 supersedes ADR-012 §7.6's long-form + short-alias scheme).
- The name is **themed where the themed word is itself clearest** (`hypha`, `cyst`, `graft`) and
  **conventional where a plain word is clearer** (`fuse`, `mesh`, `reclaim`) — the DN-02 gate
  applied once, to one name.
- A name must be mnemonic, pronounceable, and must not collide with common meanings in Rust,
  Python, C, Go, or other widely used languages — nor with the language family name (`myco`).

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
> invariants (RT1–RT7) it must respect are in RFC-0008 §4.5. **DN-03 ratified one name per term**
> through the three-test gate against those meanings, rejecting ADR-012 §7.6's canonical+alias
> scheme as needless surface area (DN-03 §3). They remain *reserved* — activation requires each
> construct's RFC-0008 implementation-stage RFC (R1/R2).

These primitives form the execution model and distributed-systems substrate (RFC-0008) — **one
name each**, themed where the themed word is itself clearest, conventional where a plain word is.

| Name | RFC-0008 meaning (RT invariants) |
|------------|---------------------------------------------------|
| `hypha`      | structurally-scoped concurrent computation over immutable values (RT1/RT2/RT7) |
| `fuse`       | lawful state fusion: semilattice merge, meet-composed `Meta` — two states converge into one (RT6) |
| `xloc`       | explicit, fallible, `Meta`-preserving value movement with backpressure ("trans-locate") (RT1/RT4) |
| `cyst`       | content-addressed checkpoint of a dormable computation — encystment *is* the dormant resumable form; `cyst(…)` constructor-style like `spore(…)` (RT2) |
| `graft`      | capability contract with external infra (the capability is an affine `substrate`) (RT4) |
| `forage`     | adaptive placement/discovery as a reified RFC-0005 policy — the third site (RT3) |
| `backbone`   | a declared/promoted high-bandwidth transport path; a placement-policy artifact, semantics-free (RT3) |
| `mesh`       | gossip/pub-sub overlay with honest **probabilistic** guarantees (δ) (RT5) |
| `tier`       | execution-mode switch: tiering interpreted↔native = RFC-0004 `ExecutionMode` (a dense↔sparse *repr* switch is a `Swap`, S1) |
| `reclaim`    | supervision-tree reclamation of *runtime units* — **never memory** (LR-9) (RT7) |

> **One name per term (DN-03 §3).** Each themed term has exactly one name — no canonical/alias
> pairs, no per-audience projection. ADR-012 §7.6 proposed a long-form + short-alias scheme
> ("content-addressing makes the second spelling free"); DN-03 rejected it as a speculative
> benefit at a real cost (two labels per concept to keep in sync). Pick the single clearest name
> and stop — content-addressing still underlies *definition* identity (ADR-003), the lexicon just
> doesn't mint two labels for it.

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
