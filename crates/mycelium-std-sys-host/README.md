# mycelium-std-sys-host

Production **host wiring** for the Mycelium standard library (RFC-0028 §4.5; M-722/M-723).

The pure `std` crates (`mycelium-std-rand`, `mycelium-std-time`, …) keep their OS contact behind
**injectable seams** — `EntropySource`, `ClockSource` — so they stay `wild`-free and testable with
deterministic stubs. The audited OS floor (`/dev/urandom`, `std::time`, …) lives in exactly one
place: `mycelium-std-sys` (LR-9 / RFC-0016 §8-Q6).

This crate is the **production glue** that fills those seams with the real floor:

- [`OsEntropy`] — `EntropySource` backed by `mycelium-std-sys::rand` (`/dev/urandom`).
- [`OsClock`] — `ClockSource` backed by `mycelium-std-sys::time` (monotonic + wall + logical).

It is the only crate that depends on **both** the audited floor and the pure std crates, so the
dependency direction stays honest: pure std → (seam) ← host wiring → floor. No `unsafe`, no kernel
coupling.

## Honesty (VR-5 / G2)

Every read is **`Declared`** — a genuine OS source, but no checked precision/quality theorem or
corpus. Failures are explicit `Result`/`Option` (no silent zero-fill, no clock wrap/clamp).

## Scope note

The Mycelium-surface `wild:`-dispatch encoding for the byte-oriented `io`/`fs` ops (so a Mycelium
`wild { io.write(…) }` block reaches these functions) is the **RFC-0028 §4.4 per-op host encoding**,
which is deferred to the `@std-sys` author and not yet committed. This crate supplies the Rust-level
production wiring the std crates need today; the surface encoding follows when §4.4 is decided.
