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

### Layer Model

| Layer | Purpose                              | Theming Approach                  | Typical User                  | Example Terms                  |
|-------|--------------------------------------|-----------------------------------|-------------------------------|--------------------------------|
| L1    | Daily surface syntax & keywords      | Conservative (three-test gate)    | All developers                | `colony`, `consume`, `embody`  |
| L2    | Runtime primitives & architecture    | Aggressive with short mnemonics   | Systems & runtime developers  | `hyph`, `anas`, `xloc`, `sclrt`|
| L3    | Formal semantics & Core IR           | Technical / conventional          | Language designers & compilers| `Repr`, `GuaranteeStrength`    |

**Naming Rules**
- **Short form** (3–5 characters preferred): Used in source code.
- **Full term**: Used in documentation, comments, error messages, and when first introducing a concept.
- Abbreviations must be mnemonic and must not collide with common meanings in Rust, Python, C, Go, or other widely used languages.

---

## 2. Layer 1: Surface Keywords & Syntax

### Themed Terms (Ratified)

| Short Form | Full Term   | Definition | Mnemonic & Rationale | Behaviors | Example |
|------------|-------------|------------|----------------------|-----------|---------|
| `colony`   | Colony      | A bounded, self-sustaining unit of definitions and behavior. | A fungal colony is a living network of hyphae working together. | Owns definitions, controls visibility, participates in the larger network. | `colony math { fn add(...) }` |
| `spore`    | Spore       | An immutable, verifiable, self-contained artifact of code + state + metadata that can germinate into execution. | A biological spore carries everything needed to regrow the organism. | Content-addressed, signed, dispersible. Supports lightweight and durable variants. | `let s = spore { ... }; germinate(s)` |
| `consume`  | Consume     | Acquire and take exclusive ownership of a linear/affine resource. | A fungus consumes substrate exactly once to grow. | Enforces single-use semantics via the type system. | `let res = consume(substrate);` |
| `embody`   | Embody      | Declare inherent behavior and methods attached to a specific type. | The type *embodies* its own capabilities. | Distinct from shared `trait` behavior. | `embody Point { fn distance(self) }` |
| `grow`     | Grow        | Automatically generate or extend behavior on a type. | The system *grows* new capabilities, like fungal extension. | Used for derive-like and generative features. | `grow Debug for Point;` |
| `wild`     | Wild        | Lexically marked, denied-by-default region for unsafe/raw operations. | Growth that has left the safe, cultivated colony. | Only place raw FFI or manual memory is allowed. | `wild { foreign_call(...) }` |
| `matured`  | Matured     | Marks a definition as stable, verified, and eligible for AOT compilation. | The component has grown into a hardened, persistent form. | Used for promotion to stable components. | `matured fn critical_path(...)` |

### Conventional Terms (Retained for Learnability)

`let`, `fn`, `type`, `trait`, `use`, `match`, `if` / `else`, `loop`, `while`, `for`, `break`, `continue`, `return`, `pub`, `self`, `where`, `Result` / `Option` family, `swap`, `policy`, `total` / `partial`.

---

## 3. Layer 2: Runtime Primitives & Architectural Concepts

These primitives form the execution model and distributed systems substrate. Short forms are used in code.

| Short Form | Full Term              | Definition | Mnemonic Hook | Programming Concept | Key Behaviors |
|------------|------------------------|------------|---------------|---------------------|---------------|
| `hyph`     | Hypha                  | Fundamental unit of concurrent execution and exploratory growth. | Active growth path / unit | Lightweight concurrent context with topology | Can branch, participate in anastomosis, be sclerotized |
| `anas`     | Anastomose             | Dynamically fuse or connect execution units for redundancy or collaboration. | Network fusion / connection | Typed channel or state fusion | Supports safe merging via protocols or CRDTs |
| `xloc`     | Translocate            | Move data or resources across the network with prioritization. | Cross-location movement | Efficient routed dataflow | Supports backpressure and path promotion to `rhizo` |
| `sclrt`    | Sclerotize / Sclerotium| Create a durable, resumable checkpoint of execution state. | Scale + shelter / hardened survival structure | Checkpoint, migration, hibernation | Produces a first-class resumable artifact |
| `myco`     | Mycorrhize             | Declare a mutualistic, capability-based contract with infrastructure or other components. | Symbiotic interface | Clean, mutual-benefit boundary | Enforces mutual obligations and isolation |
| `forage`   | Forage                 | Adaptive, signal-driven discovery and placement of work or resources. | Biological foraging behavior | Adaptive scheduling / placement policy | Can integrate feedback or simple learned policies |
| `rhizo`    | Rhizomorph             | High-bandwidth or priority long-distance transport path. | Root-like high-capacity strand | Optimized backbone route | Can be statically declared or dynamically promoted |
| `cmn`      | Common Mycorrhizal Network (Wood Wide Web) | Decentralized signaling and resource-sharing mesh enabling emergent coordination. | Emergent underground network | Gossip / decentralized overlay | Supports pub/sub and indirect resource transfer |
| `dimorph`  | Dimorph                | Switch between execution modes (e.g. dense vs sparse, interpreted vs native). | Dual-form adaptation | Context-aware mode switching | Explicit or policy-driven |
| `reclaim`  | Reclaim (Saprotroph)   | Reclaim or decompose stale or unused resources. | Decomposition and recycling | Resource cleanup / reclamation policy | Can run as background or explicit operation |

---

## 4. Layer 3: Formal / IR / Semantic Concepts

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

- **General development**: Stay in L1 + conventional keywords.
- **Systems programming & runtime work**: Use L2 short forms.
- **Documentation & diagnostics**: Introduce concepts with the full term followed by the short form in parentheses on first use.
- **Error messages**: Prefer clarity — e.g., “Cannot `anas` (anastomose) incompatible `hyph` types.”

---

*End of Lexicon Reference v0.4*