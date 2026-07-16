# Remap Manifest — mycelium-std-sys → std.sys

_DN-109 §5.2 provenance ledger. Guarantee: **Declared** — this records proposed/performed structural and idiom choices; it does not certify them (no guarantee-tag upgrade from the manifest's existence alone, VR-5). Rendered from `remap` in `summary.json` — this file is a pure projection, never a second source of truth._

## Nodules (8)

| Target nodule | Operation | Safety | API surface changed | Identity neutral | Sources | Rationale |
|---|---|---|---|---|---|---|
| `std.sys.fs` | Keep | Safe | false | true | `crates/mycelium-std-sys/src/fs.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.sys.guarantee_matrix` | Keep | Safe | false | true | `crates/mycelium-std-sys/src/guarantee_matrix.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.sys.io` | Keep | Safe | false | true | `crates/mycelium-std-sys/src/io.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.sys` | Keep | Safe | false | true | `crates/mycelium-std-sys/src/lib.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.sys.math` | Keep | Safe | false | true | `crates/mycelium-std-sys/src/math.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.sys.rand` | Keep | Safe | false | true | `crates/mycelium-std-sys/src/rand.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.sys.sys` | Keep | Safe | false | true | `crates/mycelium-std-sys/src/sys.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.sys.time` | Keep | Safe | false | true | `crates/mycelium-std-sys/src/time.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |

## Idiom choices (0)

_(none in this run — v0 Mechanical-only auto-fire with no located idiom-choice instrumentation yet; see DN-109 §7-e and this module's doc comment)_
