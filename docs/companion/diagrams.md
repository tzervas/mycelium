# Diagrams — flows, maps, relationships

All diagrams are **Mermaid** (GitHub / many docsites render natively). They are
curatorial; status edges must match Doc-Index when in doubt.

## Guarantee lattice

<a id="guarantee-lattice"></a>

```mermaid
flowchart TB
  Exact --> Proven
  Proven --> Empirical
  Empirical --> Declared
  subgraph meet [Composition = meet]
    direction LR
    X[inputs…] --> M((meet)) --> R[result grade]
  end
  Declared -.->|airlock + basis| Exact
```

## Memory lifecycle

<a id="memory-lifecycle"></a>

```mermaid
stateDiagram-v2
  [*] --> Affine: create unique
  Affine --> Dropped: scope exit L1
  Affine --> Shared: explicit share
  Shared --> Shared: Dup / Borrow
  Shared --> Dropped: RC hits 0
  Dropped --> [*]
  note right of Shared
    L2 elision may remove Dup
    L3 region exit is safety net
  end note
  state Colony {
    [*] --> HyphaA
    [*] --> HyphaB
    HyphaA --> RegionReclaim
    HyphaB --> RegionReclaim
  }
```

## Three trust axes

<a id="three-trust-axes"></a>

```mermaid
quadrantChart
  title Trust space sketch (illustrative)
  x-axis Loose typing --> Strict typing
  y-axis Fast cert --> Certified
  quadrant-1 High rigor ship
  quadrant-2 Careful research
  quadrant-3 Exploratory
  quadrant-4 Strict but uncert
```

*(Quadrant chart is pedagogical only — real axes are three-dimensional; see
[04](04-three-trust-axes.md).)*

## Decision clusters (ADR-045 window)

<a id="decision-clusters"></a>

```mermaid
mindmap
  root((ADR-045 gap-close))
    Mutation loop
      DN-33
      DN-35
      DN-120
      DN-125
    Trust axes
      Lattice
      Cert modes
      DN-126 strictness
    Native L3
      DN-127 format
      DN-128 derive
      DN-129 Init
      DN-130..138
      DN-140 ident
    Physical AOT
      RFC-0039
      ADR-030/031
```

## Mutation loop interlock

```mermaid
flowchart TB
  subgraph past [Earlier machinery]
    DN33[DN-33 uniqueness]
    DN35[DN-35 rc==1 reuse]
  end
  subgraph close [Coherence + surface]
    DN120[DN-120 identity]
    DN125[DN-125 value-thread]
  end
  DN33 --> DN35 --> DN120 --> DN125
  DN125 --> OUT["&mut problem → by-value rebind"]
```

## Crate strata (sketch)

<a id="crate-strata"></a>

```mermaid
flowchart TB
  subgraph kernel [Trusted base]
    core[mycelium-core]
    interp[mycelium-interp]
    l1[mycelium-l1]
  end
  subgraph opt [Untrusted optimizers]
    mir[mycelium-mir-passes]
    mlir[mycelium-mlir]
  end
  subgraph std [stdlib + ports]
    stdr[mycelium-std-*]
    myc[lib/std/*.myc]
  end
  subgraph tools [Toolchain]
    check[mycelium-check]
    tr[mycelium-transpile]
  end
  core --> interp
  core --> l1
  l1 --> check
  l1 --> tr
  mir --> core
  mlir --> core
  stdr --> core
  myc --> check
  tr --> check
```

## Reading graph

```mermaid
flowchart LR
  R[Companion README] --> H[00 How to read]
  H --> T[01 Thesis]
  T --> A[02 Airlocks]
  T --> M[03 Memory]
  A --> X[04 Trust axes]
  M --> X
  X --> D[05 Decision map]
  D --> E[06 Expressibility]
  E --> GAP[gap-analysis leaves]
  D --> DI[Doc-Index]
```
