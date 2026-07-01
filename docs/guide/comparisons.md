# How Mycelium compares — and why

One-line purpose: an honest, fair comparison against the language/library families Mycelium is
most often confused with — what's shared, what's different, and why the difference exists.

Mycelium is not trying to be a faster general-purpose systems language, a better ML framework,
or a novel dependently-typed proof assistant. Each comparison below is made fairly — shared
ground and genuine differences.

## Contents

- [vs. typed systems languages](#vs-typed-systems-languages-rust-haskell-ml-family)
- [vs. ML / array languages and Python scientific stack](#vs-ml--array-languages-and-python-scientific-stack)
- [vs. VSA / HDC libraries](#vs-vsa--hdc-libraries-torchhd-resonator-network-implementations)
- [vs. verification-oriented languages](#vs-verification-oriented-languages-compcert-fstar-lean-dafny)

## vs. typed systems languages (Rust, Haskell, ML family)

**Shared:** strong static types, explicit ownership/lifetime reasoning, composition over
inheritance, small auditable kernel (KC-3), no hidden behavior; content-addressed definitions
draw from Unison.

**Different:** none of them treat dense embeddings or VSA/HDC as first-class type families with
a shared type system. In Rust or Haskell, moving a value from a ternary representation to a VSA
hypervector is a user-written, uncertified function call; the type system has no stake in the
accuracy claim. Mycelium's `swap` is the *only* such operation, and it must emit a certificate.
The guarantee lattice is part of the *type* of a value, not a documentation annotation.

**Why:** the survey found no existing system unifies even two of {binary, balanced ternary, dense
embedding, sparse/dense VSA} as co-equal, first-class substrates with verifiable inter-conversion
(G1). The four-way union with certified swaps is the novel integrative contribution.

## vs. ML / array languages and Python scientific stack

**Shared:** first-class dense vector/matrix operations; numeric precision tagging; the accuracy
requirement around float approximation recalls Rosa/Daisy/Gappa.

**Different:** NumPy/PyTorch treat conversion silently — a `.half()` call in PyTorch does not
emit a certificate describing the precision loss, and there is no guarantee lattice ensuring the
accuracy claim propagates correctly through a pipeline. VSA operations (if present at all) are
a library on top of the type system, not a first-class type family. Mycelium's `Dense` and `VSA`
types are at the same level as `Binary` and `Ternary`; swapping between them is the same
certified `swap` operation.

**Why:** ML practitioners routinely suffer from silent precision loss and from the impossibility
of auditing what happened to a pipeline's accuracy claims. Mycelium addresses this structurally,
not through documentation conventions.

## vs. VSA / HDC libraries (torchhd, resonator-network implementations)

**Shared:** the MAP-I algebra (`bind`/`unbind`/`bundle`/`permute`/`cleanup`), per-model
guarantee matrices, with capacity bounds and crosstalk stated.

**Different:** torchhd (and similar libraries) sit above PyTorch as a numeric layer; the host
language's type system knows nothing about the hypervector type or its bounds. Mycelium's `VSA`
type is a first-class type family in the *language's* core type system. The capacity bound is a
`Proven` or `Empirical` guarantee tag on the *value*, not a comment in the source. The `bundle`
probe (`proofs/lh-bundle/`) confirmed that MAP-I capacity admits `Proven` tags under the
Clarkson-Ubaru-Yang / Thomas-Dasgupta-Rosing non-asymptotic bounds — so "bounds exist"
is checked, not declared.

HRR/FHRR are the VSA weak link (RR-13): non-self-inverse bind means unbind is lossy
(`Empirical` only); prefer MAP/BSC for compositional work where `Proven` tags are needed.

**Why:** the survey found no VSA library that integrates with a language-level type system
providing certified inter-substrate swaps (G1). Building the VSA submodule in-language (not just
in a library) is what enables the certified swap infrastructure to cover VSA↔binary/ternary/dense
paths.

## vs. verification-oriented languages (CompCert, Fstar, Lean, Dafny)

**Shared:** translation-validation (per-swap certificate checking, not whole-engine proof, VR-4),
the "no black boxes" principle, explicit about what is and is not proven.

**Different:** CompCert-style verified compilers prove a *compiler* correct once; Mycelium uses
translation-validation to prove each *instance* of a swap or lowering correct. Mycelium does not
require the user to write proofs — the guarantee lattice is inferred by meet-composition from
per-op tags, and proofs live in the implementation, not exposed to the surface language. Mycelium
is also multi-substrate (four representation families), which no existing verification-oriented
language treats as first-class.

**Why:** whole-engine proofs (CompCert-style) are high-cost; per-swap translation validation
(the "certificate checker in Rust" approach from ADR-010) gives guarantees at per-swap
granularity with manageable overhead. The KC-4 gate (cert-overhead budget) confirms the overhead
is within budget: ≤ 2× the swap cost for swaps whose own cost exceeds the check, ≤ 5 µs
absolute (ADR-021 A5, measured — `cargo xtask kc4`).

---

**See also:** [Why & design](why-and-design.md) · [Guarantees & verification](guarantees-and-verification.md) ·
[Workspace map](workspace-map.md)

[← Back to README](../../README.md)
