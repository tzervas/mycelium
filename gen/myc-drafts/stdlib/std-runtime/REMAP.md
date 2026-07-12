# Remap Manifest — mycelium-std-runtime → std.runtime

_DN-109 §5.2 provenance ledger. Guarantee: **Declared** — this records proposed/performed structural and idiom choices; it does not certify them (no guarantee-tag upgrade from the manifest's existence alone, VR-5). Rendered from `remap` in `summary.json` — this file is a pure projection, never a second source of truth._

## Nodules (14)

| Target nodule | Operation | Safety | API surface changed | Identity neutral | Sources | Rationale |
|---|---|---|---|---|---|---|
| `std.runtime.colony` | Keep | Safe | false | true | `crates/mycelium-std-runtime/src/colony.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.runtime.dataflow` | Keep | Safe | false | true | `crates/mycelium-std-runtime/src/dataflow.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.runtime.guarantee_matrix` | Keep | Safe | false | true | `crates/mycelium-std-runtime/src/guarantee_matrix.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.runtime` | Keep | Safe | false | true | `crates/mycelium-std-runtime/src/lib.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.runtime.network` | Keep | Safe | false | true | `crates/mycelium-std-runtime/src/network.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.runtime.policy_mech` | Keep | Safe | false | true | `crates/mycelium-std-runtime/src/policy_mech.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.runtime.r2_residual` | Keep | Safe | false | true | `crates/mycelium-std-runtime/src/r2_residual.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.runtime.rc` | Keep | Safe | false | true | `crates/mycelium-std-runtime/src/rc.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.runtime.reclamation` | Keep | Safe | false | true | `crates/mycelium-std-runtime/src/reclamation.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.runtime.region` | Keep | Safe | false | true | `crates/mycelium-std-runtime/src/region.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.runtime.scheduler` | Keep | Safe | false | true | `crates/mycelium-std-runtime/src/scheduler.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.runtime.scope_region` | Keep | Safe | false | true | `crates/mycelium-std-runtime/src/scope_region.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.runtime.supervision` | Keep | Safe | false | true | `crates/mycelium-std-runtime/src/supervision.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |
| `std.runtime.task` | Keep | Safe | false | true | `crates/mycelium-std-runtime/src/task.rs` | structure-preserving 1:1 (DN-109 §5.3-B v0 default: every emitted nodule keeps its source file's mod-tree position, no restructuring proposed) |

## Idiom choices (0)

_(none in this run — v0 Mechanical-only auto-fire with no located idiom-choice instrumentation yet; see DN-109 §7-e and this module's doc comment)_
