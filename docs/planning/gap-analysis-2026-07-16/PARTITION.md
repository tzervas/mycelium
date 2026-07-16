# Gap analysis partition — 2026-07-16

Fractal swarm: **Epic = scope group**, **Leaf = one crate**.
Model: **grok-composer-2.5-fast** only.
Goal: exact residual to (1) full Rust implementation completion (2) transpile-to-Mycelium readiness.
Update against: language-completeness-gap-inventory, DN-136 phase2 worklist, DN-99, zero-hand-port-delta-ledger, DN-34.

## kernel (7)
- `mycelium-cert`
- `mycelium-core`
- `mycelium-dense`
- `mycelium-numerics`
- `mycelium-select`
- `mycelium-vsa`
- `mycelium-vsa-decode`
## runtime (5)
- `mycelium-interp`
- `mycelium-rt-abi`
- `mycelium-sched`
- `mycelium-stack`
- `mycelium-workstack`
## frontend (1)
- `mycelium-l1`
## aot (2)
- `mycelium-mir-passes`
- `mycelium-mlir`
## stdlib (27)
- `mycelium-std-cmp`
- `mycelium-std-collections`
- `mycelium-std-conformance`
- `mycelium-std-content`
- `mycelium-std-core`
- `mycelium-std-dense`
- `mycelium-std-diag`
- `mycelium-std-error`
- `mycelium-std-fmt`
- `mycelium-std-fs`
- `mycelium-std-io`
- `mycelium-std-iter`
- `mycelium-std-math`
- `mycelium-std-numerics`
- `mycelium-std-rand`
- `mycelium-std-recover`
- `mycelium-std-runtime`
- `mycelium-std-select`
- `mycelium-std-spore`
- `mycelium-std-swap`
- `mycelium-std-sys`
- `mycelium-std-sys-host`
- `mycelium-std-ternary`
- `mycelium-std-testing`
- `mycelium-std-text`
- `mycelium-std-time`
- `mycelium-std-vsa`
## toolchain (12)
- `mycelium-build`
- `mycelium-check`
- `mycelium-cli`
- `mycelium-cli-common`
- `mycelium-diag`
- `mycelium-doc`
- `mycelium-fmt`
- `mycelium-lint`
- `mycelium-lsp`
- `mycelium-proj`
- `mycelium-sec`
- `mycelium-spore`
## transpile (1)
- `mycelium-transpile`
## bench_cert (1)
- `mycelium-bench`

**Total crates:** 56
