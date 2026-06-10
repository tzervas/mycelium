# ADR-012: Layered Lexicon and Fungal Runtime Model

| Field | Value |
|-------|-------|
| **ADR** | 012 |
| **Title** | Layered Lexicon and Fungal Runtime Model |
| **Status** | Proposed |
| **Date** | 2026-06-10 |
| **Depends on** | DN-02 (Fungal Lexicon), RFC-0001 (Core IR), RFC-0003 (VSA), RFC-0004 (Execution Model) |

## 1. Context

The Mycelium language requires both a clean, learnable surface syntax and a powerful runtime model capable of expressing adaptive, resilient, distributed computation across heterogeneous substrates (including future underground AI infrastructure).

DN-02 established a conservative, hybrid approach to surface keywords using a strict three-test gate. However, the deeper fungal execution model research (hyphal growth, anastomosis, sclerotia, mycorrhizal symbiosis, foraging, etc.) offers significant value for the runtime and architectural layers.

A purely conservative approach would under-utilize the biological metaphor. A purely aggressive theming approach would harm learnability.

## 2. Decision

Adopt a **three-layer lexicon model**:

- **L1 (Surface)**: Conservative theming using the DN-02 ratified terms + `consume`, `embody`, `grow`.
- **L2 (Runtime & Architecture)**: More aggressive use of fungal concepts with short, mnemonic abbreviations (`hyph`, `anas`, `xloc`, `sclrt`, `myco`, `forage`, `rhizo`, `cmn`, `dimorph`, `reclaim`).
- **L3 (Formal/IR)**: Technical terms from RFC-0001 with light fungal influence where helpful.

Additionally, integrate the comparative analysis showing Mycelium’s position relative to Rust, Unison, MLIR, and others, highlighting its novel contributions around certified representation swapping and monotonic honesty.

## 3. Rationale

- Learnability is a first-class non-functional requirement.
- The fungal metaphor is most powerful at the runtime and systems architecture level.
- Short mnemonic forms in L2 allow expressive power without punishing daily development.
- This structure supports both human developers and future AI agents working with the language.

## 4. Consequences

**Positive**
- Clear separation of concerns.
- Strong mnemonics without cognitive overload.
- Positions Mycelium well for resilient, adaptive, multi-paradigm distributed systems.

**Negative / Risks**
- Requires good documentation and tooling to help developers navigate layers.
- Some L2 terms may still feel unfamiliar initially.

## 5. Implementation

- Add `docs/notes/Lexicon-Reference.md`
- Add `docs/notes/Example-Programs-Reference.md`
- This ADR itself
- Update `docs/Doc-Index.md` and relevant RFCs as needed.

## 6. Status

**Proposed** — Awaiting maintainer review and ratification.

---

*End of ADR-012*