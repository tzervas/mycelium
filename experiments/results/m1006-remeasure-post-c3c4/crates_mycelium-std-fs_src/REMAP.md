# Remap Manifest — mycelium-std-fs → std.fs

_DN-109 §5.2 provenance ledger. Guarantee: **Declared** — this records proposed/performed structural and idiom choices; it does not certify them (no guarantee-tag upgrade from the manifest's existence alone, VR-5). Rendered from `remap` in `summary.json` — this file is a pure projection, never a second source of truth._

## Nodules (7)

| Target nodule | Operation | Safety | API surface changed | Identity neutral | Sources | Rationale |
|---|---|---|---|---|---|---|
| `std.fs.error` | Keep | Safe | false | true | `crates/mycelium-std-fs/src/error.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.fs.guarantee_matrix` | Keep | Safe | false | true | `crates/mycelium-std-fs/src/guarantee_matrix.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.fs` | Keep | Safe | false | true | `crates/mycelium-std-fs/src/lib.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.fs.metadata` | Keep | Safe | false | true | `crates/mycelium-std-fs/src/metadata.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.fs.options` | Keep | Safe | false | true | `crates/mycelium-std-fs/src/options.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.fs.path` | Keep | Safe | false | true | `crates/mycelium-std-fs/src/path.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.fs.substrate` | Keep | Safe | false | true | `crates/mycelium-std-fs/src/substrate.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |

## Idiom choices (0)

_(none in this run — v0 Mechanical-only auto-fire with no located idiom-choice instrumentation yet; see DN-109 §7-e and this module's doc comment)_
