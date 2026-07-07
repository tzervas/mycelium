# Design Note DN-85 — Multi-Language Transpilation and the Single-Language Full-Stack Goal

| Field | Value |
|---|---|
| **Note** | DN-85 |
| **Status** | **Proposed** (2026-07-06) — maintainer direction; records a program direction, ratifies no mechanism. |
| **Feeds** | the trx2 transpiler program (DN-34 · M-991 · M-1000…M-1006) · `rwr` (Phase-II) · RFC-0028 (FFI) |
| **Decides** | *Nothing normatively.* Captures the maintainer's decided **direction** that the Rust→Mycelium transpiler (DN-34, Rust-only) generalizes into a **multi-source-language** program whose flagship goal is a **single-language (Mycelium) full stack**. Each concrete language frontend, its soundness gate, and its provenance policy land via their own RFC/ADR + issues when built. |
| **Date** | 2026-07-06 |
| **Task** | (program-level; per-language issues minted per wave — mitigation #1) |

> **Posture (transparency rule / VR-5 · never-silent G2).** Advisory and **aspirational**. This note
> makes a large, long-horizon goal explicit **without** upgrading any guarantee: a transpiled artifact
> stays `Declared` until a per-language differential vets it (the M-991 discipline generalized), and a
> **binding** or a **reverse-engineered reimplementation** is *never* labeled a faithful transpile —
> its provenance tag says exactly what it is. Nothing here is tag-gating for `lang 1.0.0`
> (ADR-036 §2.1–§2.2 — not tag-gating; the public release is separately gated on **functional
> usability** per **ADR-038**, decoupled from self-hosting/rewrite completion, which continues
> post-public as ADR-038 Phase II).

---

## 1. The flagship goal

**Mycelium as a single-language solution for a source ecosystem's *entire* stack.** Today a real
system is polyglot by necessity — a Python application over C++/CUDA kernels, a scientific pipeline
over Fortran, a service over C. The goal is to **collapse that polyglot stack into one language,
one toolchain, and one guarantee model** (the honesty lattice `Exact ⊐ Proven ⊐ Empirical ⊐
Declared`, per-op provenance) end to end — application code, its native extensions, and the
compute kernels all expressed in Mycelium.

The benefit is not novelty for its own sake: it is **one language for the full stack** — a single
build, a single debugger, a single memory/value model, a single provenance record from the top of
the app to the GPU kernel — replacing the seams (FFI boundaries, ABI mismatches, split toolchains,
divided guarantees) that a polyglot stack pays for continuously.

## 2. Scope — a demand-driven, multi-language program

The transpiler is **not** Rust-specific in the limit; Rust (DN-34, `trx`/`trx2`) is the **first
arm**, chosen because it accelerates self-hosting (`boot10`/M-993). The program **extends to a new
source language as we encounter it and the related need** — not a big-bang, but pulled by real
targets. The anticipated arms, in rough order of likely demand:

- **Rust** — in flight (DN-34 · trx2). The transpiler-hardening engine (M-1006 ladder) also
  develops the reusable machinery every later arm inherits.
- **Python** — the next frontier (§3).
- **C · C++ · Fortran · Cython · CUDA · …** — added as targets and need arise.

Each new arm **extends** the transpiler (new frontend, new gap taxonomy, new grammar mapping,
new differential oracle) and **feeds its lessons back** into the shared core — the same
transpile → vet → patch → learn loop the M-1006 ladder runs for Rust, generalized per language.

## 3. Python first — pure Python, then the extensions

Python is the first non-Rust arm, sequenced deliberately:

1. **Pure Python first.** The initial Python targets are the **pure-Python** parts of a library —
   no C-extensions, no CUDA. This keeps the first arm tractable and lets the hard problem be
   isolated.
2. **The gate: sound type inference.** Python is dynamically typed; a faithful Mycelium
   transpilation needs types. **The soundness of the inferred types is the gate** to extending
   further — inference must be polished "to a sound degree" before the remaining extensions are
   added. (Honesty corollary: an *inferred* type is `Declared`/`Empirical` per its basis, never
   `Proven` without a checked derivation — VR-5. An unsound inference is a **gap**, never a silent
   guess — G2.)
3. **Then the extensions, for full coverage.** Once inference is sound, add the remaining layers
   (Cython, C-extensions, CUDA) so that **the whole library** — not just its Python surface — can
   be transpiled fully into Mycelium. Full-library transpilation is the arm's completion criterion.

## 4. The interim strategy — transpile what you can, bind the rest

Before a given native language's transpiler exists, a library that spans languages is handled by
**layering, honestly**:

- Transpile the layer the toolchain already covers (e.g. the **pure-Python** of PyTorch /
  TensorFlow), and keep **bindings (FFI)** to the native **C++/CUDA** backend **until the
  respective transpilers are sorted**.
- The result is a **Mycelium front + FFI-bound native back** — a *partial* port. It is tagged as
  exactly that: the Python layer is transpiled (`Declared`→vetted), the native layer is **bound**
  (its guarantee is the binding's, not a port's). A binding is **never** presented as a transpile
  (G2/VR-5). The FFI surface is the existing one (RFC-0028 · the `wild`/`@std-sys` vocabulary).
- As each native-language transpiler lands, the bound layer is **progressively** replaced by a
  transpiled one — the same progressive-replacement shape as `rwr`'s Mycelium rewrite, applied
  across languages.

## 5. The open-source constraint — and the closed-source fallbacks

**Transpilation requires source.** The program only fully applies to **open-source** libraries.
Where a native component's **C++/CUDA source is unavailable** (closed / proprietary), it **cannot
be transpiled**, and must strictly fall back to one of:

1. **Bindings only** — FFI to the shipped, compiled artifact. The component stays external; its
   guarantee is "calls a binary we did not build" (`Declared`, provenance = external binding).
2. **A Mycelium-native reverse-engineered reimplementation** — reproduce the *behavior* natively in
   Mycelium from its observable contract. This is **new first-party code**, not a transpile: it
   carries its own tests, its own guarantee tags, and an explicit **equivalence witness** against
   the original where one can be constructed (differential over the observable contract). It is
   **never** labeled a port of source it never saw (G2/VR-5).

The provenance ladder, made explicit (the tag on every component says which rung it is on):

| Situation | Path | Provenance / guarantee basis |
|---|---|---|
| Source available **+** its transpiler exists | **Transpile** | `Declared` → `Empirical` once a differential vets it |
| Source available, transpiler **not yet** built | **Transpile the coverable layer + FFI-bind the rest** | mixed: transpiled layer per its vet; bound layer = the binding's basis |
| Source **unavailable** (closed) | **FFI-bind the artifact**, *or* **reverse-engineer a Mycelium-native reimpl** | binding = external-call basis; reimpl = first-party, its own tests + an equivalence witness |

## 6. Relationship to the existing tracks (no duplication)

- **DN-34 / trx / trx2** — the **Rust arm** + the transpiler-hardening engine. This note
  *generalizes* DN-34's strategy across languages; DN-34 stays the Rust-specific record. The
  **M-1006 ladder** is Rust phase-1 of this larger program, and the machinery it hardens is what
  later arms reuse.
- **`boot10` / M-993** — self-hosting the Mycelium compiler (Rust → Mycelium). Distinct lane
  (FLAG-V2, rwr): self-hosting is `boot10`'s, this program *accelerates* it (the Rust arm) rather
  than replacing it.
- **`rwr` (Phase-II)** — the progressive Mycelium **rewrite**. This multi-language program is a
  broader, longer-horizon **product** direction; where they touch, trx2/DN-85 outputs are
  **inputs** to `rwr`'s port-wave manifests (M-947…M-957), coordinated via issues, never duplicated.
- **RFC-0028 (FFI & system interface)** — the binding substrate the interim + closed-source paths
  lean on.

## 7. Honesty posture (binding)

1. A transpiled artifact is `Declared` until a **per-language differential** vets it — the M-991
   verdict generalized: a transpiler is a **gap-profiling instrument** until its vet loop + oracle
   exist for that language.
2. **Bindings** and **reverse-engineered reimplementations** carry their **own** provenance and
   **never masquerade as transpiles** (§5). The tag names the rung.
3. **Inferred types** (Python) are graded by their basis, never upgraded past a checked derivation
   (VR-5); an unsound inference is a surfaced **gap**, not a silent assumption (G2).
4. Everything in this note is `Proposed`/`Declared` **direction**. Concrete frontends, soundness
   gates, and provenance mechanisms are ratified per language via their own RFC/ADR + issues.

## 8. Open questions / Definition of Done (for the *note*, not the program)

This note is **Resolved** when the maintainer confirms the direction and the **first concrete arm**
(Python-pure) is minted as a scoped RFC/ADR + issues. It leaves open, for those follow-ons:

- **Q1 — "sound type inference" as a gate.** What soundness bar must Python inference clear before
  the extensions are added? (A checked-derivation discipline; unsound → gap.)
- **Q2 — binding provenance tracking.** How is a bound-vs-transpiled boundary recorded per symbol so
  a mixed library's guarantee map is honest and inspectable (`EXPLAIN`-able)?
- **Q3 — reverse-engineering equivalence witnesses.** What counts as an acceptable equivalence
  witness for a Mycelium-native reimplementation of a closed component (differential over the
  observable contract; property tests; where `Proven` is even possible)?
- **Q4 — per-language differential oracle.** Each arm needs a runnable oracle (as the Rust arm has
  `myc check` + the stage differentials); what is Python's, C's, CUDA's?
- **Q5 — sequencing vs. the flip.** Where does the multi-language program sit relative to the
  public flip (`flp`) and `rwr`? (Direction: it is **post** the Python-first soundness gate; it does
  not gate `lang 1.0.0`.)

## Changelog

- 2026-07-06 — Created, **Proposed** (maintainer direction, this session). Records the
  multi-language transpilation program, the single-language-full-stack flagship goal, the
  Python-first / sound-inference-gated sequencing, the interim transpile-what-you-can + FFI-bind
  strategy, and the open-source constraint with its bindings / reverse-engineering fallbacks.
