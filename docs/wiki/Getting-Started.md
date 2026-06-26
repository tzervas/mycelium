# Getting Started

## Prerequisites

- **Rust** — MSRV **1.92** pinned (`rust-toolchain.toml`; ADR-007). `cargo fmt`, `cargo clippy`,
  `cargo test`.
- **Python 3.13/3.14** via **UV** — for the docs/tooling checks (`pytest`, `ruff`).
- **`just`** — the single source of truth for the local↔CI check loop.

## Clone & build

```sh
git clone https://github.com/tzervas/mycelium.git
cd mycelium
just setup     # best-effort install of the check tools (uv tool / npx / cargo-nextest)
cargo build    # build the workspace
```

## The check loop (local↔CI parity)

One source of truth (`just`); pre-commit and CI route through the same recipes. **Run before every
commit.**

```sh
just            # list all recipes
just test-fast  # Tier 0 (pre-commit): change-scoped unit/regression tests — fastest
just check      # Tier 1 (default; = `just ci`): change-scoped tests + all gates
just check-full # Tier 2 (release/durability): full workspace, mutants + fuzz smoke
just fmt        # auto-format (rust + python)
just hooks      # install the pre-commit hooks
```

Checks **skip gracefully** when a tool or language isn't present yet (most language-surface code is
still being built). The three test tiers (DN-20) keep the pre-commit loop fast while the heavy gates
run on release.

## Repository layout

- `crates/` — the 50 Rust crates (kernel · compiler · runtime · stdlib · tooling). See the
  [Crate Index](Crate-Index) and [Architecture](Architecture).
- `docs/` — the design corpus: `rfcs/`, `adr/`, `notes/` (DNs), `spec/`, `planning/`; status is in
  [`docs/Doc-Index.md`](https://github.com/tzervas/mycelium/blob/main/docs/Doc-Index.md).
- `docs/api-index/` — the committed agent-facing API index (see [API Reference](API-Reference)).
- `research/` — the evidence base the corpus traces to.
- `justfile`, `.pre-commit-config.yaml`, `scripts/` — the check tooling.

## Contributing

The house rules (small auditable kernel; the transparency & auditability rule; never-silent;
append-only decisions; grounded claims) and the tiered branch workflow are in the repository
`CONTRIBUTING.md` and `CLAUDE.md`. Every PR states which `FR/NFR/VR/SC` it advances (or which
ADR/RFC it implements) and **how it was verified**. The project is **MIT-licensed only**.
