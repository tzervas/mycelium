# mycelium-cli

> `myc` — the one-command toolchain driver (M-733): `myc init|build|check|test|run` over a Mycelium phylum, with DN-22 structured, actionable diagnostics.

**Tier:** tooling  ·  **Status:** Rust-first implementation  ·  **License:** MIT

## Overview

`mycelium-cli` provides the single front door over the Mycelium toolchain. `myc init` scaffolds a
phylum, `myc build` packages it as a content-addressed spore (M-368), `myc check` type-checks it
via the L1 front-end, `myc test` runs available verification, and `myc run` is explicitly not yet
wired — it says so with an actionable `Report` rather than a silent stub. Every user-visible failure
is a structured `Report` with a stable code, human-readable message, optional source location, and
actionable help; no raw Rust panic ever reaches the user (G2). The driver orchestrates real library
APIs directly — no fragile subprocess plumbing.

## Key items

- `myc` binary — the single toolchain entry point.
- `Report` — structured, actionable diagnostic (code, message, location, help, exit code); renders as `error[<code>]: <message>`.
- `init` / `build` / `check` / `test` subcommands — each does real end-to-end work.
- `run` subcommand — explicitly not yet wired; reports its unimplemented status honestly.

## Design references

- M-733, M-368, M-359
- E16-1
- DN-22
- RFC-0013
- G2, VR-5

## Role in the workspace

Depends on `mycelium-proj`, `mycelium-spore`, `mycelium-l1`, and `mycelium-cli-common`; provides the unified toolchain CLI above the kernel (KC-3). See the [workspace overview](../../README.md).
