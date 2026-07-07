# ADR-044 — Async Runtime (tokio + axum) for the mycelium-tero HTTP Front

| Field | Value |
|---|---|
| **ADR** | 044 |
| **Status** | **Accepted** (2026-07-07 — maintainer-authorized). Adopts **`tokio` + `axum`** as the async runtime + web framework for the `mycelium-tero` HTTP API front (M-1017 / DN-87 §2.3) — the **first async runtime in the workspace**. Scope is confined to `mycelium-tero` (`tools` tier); the kernel and every other crate remain synchronous / std-only. `Accepted → Enacted` once the front lands with `cargo build`/`cargo test` green, `cargo deny` green, and `THIRD-PARTY-LICENSES.md` regenerated (the Definition of Done below). |
| **Decides** | `mycelium-tero`'s HTTP/JSON front (`tero-http`) is an **`axum`** app on the **`tokio`** runtime. `tokio` (features `rt-multi-thread`/`macros`/`net`/`sync`/`io-util`) + `axum` are hoisted to `[workspace.dependencies]` (DN-17) and consumed **only** by `mycelium-tero`. Plain HTTP only — **no** TLS / `rustls` / `ring` (the token-scoped floor binds `127.0.0.1`; TLS is a reverse-proxy's job). The MCP front (`tero-mcp`) stays **synchronous std stdio** — no async needed for a stdio JSON-RPC loop. |
| **Grounds** | Maintainer directive (2026-07-07): the HTTP API is a genuinely **concurrent, multi-client server surface** — unlike every prior crate, which is batch/CLI/library. A real async web framework is the fit-for-purpose choice; a hand-rolled `std::net` server was the considered alternative (below) and explicitly not chosen. `axum`/`tokio`/`hyper` and their transitive tree are MIT/Apache-2.0/BSD/ISC/Unicode — already covered by `deny.toml`/`about.toml`'s allow-list (`cargo deny check` green, no new license entry). |
| **Amends** | Nothing structurally. It is the **first** exception to the workspace's de-facto "synchronous, std-only" discipline, scoped by construction (below) so the trusted base (ADR-007 / KC-3) is untouched. |
| **Date** | 2026-07-07 |

> **Posture (transparency rule / VR-5).** This ADR records a *decision*, maintainer-authorized. The
> async surface is `Declared` — mechanical transport plus a token check; no security/performance
> *proof* is claimed. The kernel/interpreter (KC-3 trusted base) is **not** touched: it stays
> synchronous std-only. No guarantee tag is upgraded.

---

## 1. Why this decision

The Mycelium workspace has, to date, exactly three external runtime dependencies (`serde`,
`serde_json`, `blake3`) and is otherwise entirely synchronous, std-only — a deliberate KC-3
"small, auditable kernel" posture. M-1017 (DN-87 §2.3) adds an **HTTP/JSON API front** so any agent
platform (Grok, curl, anything) can query the transparent memory engine over the network. Unlike
every prior crate, that front is a **concurrent, long-lived, multi-client server** — the one place
where an async runtime + a routing/extraction framework earn their weight.

The maintainer weighed this against the alternative (a minimal synchronous `std::net::TcpListener`
server — zero new deps, fully in the existing discipline) and chose the real framework: `axum` on
`tokio`. This ADR records that choice, its scope, and the supply-chain consequences, rather than
letting the workspace's first async runtime land unremarked (a dependency of this weight is a
decision, not a build detail — CLAUDE.md / CONTRIBUTING).

## 2. Scope — confined by construction

- **Only `mycelium-tero` (tier `tools`) depends on `tokio`/`axum`.** The dependency-strata gate
  (`xtask deps`, DN-68) enforces the tiering: `mycelium-tero` is `tools`, so it may depend downward
  on `core`/`std` but nothing may depend *up* on it. The kernel (`core`) and library (`std`) tiers
  cannot acquire an async edge through tero.
- **The MCP front stays synchronous** (std stdio JSON-RPC). Async is confined to the one HTTP
  server binary + `src/front/http.rs`; the shared engine, the query core, and the MCP front are
  sync and unchanged.
- **No TLS.** Plain HTTP on `127.0.0.1` only, so `rustls`/`ring`/`openssl` are **not** pulled — the
  transitive tree stays MIT/Apache-2.0/BSD/ISC/Unicode, all already allow-listed.

## 3. Supply-chain consequences (recorded, never silent — G2)

- `tokio`, `axum`, and their transitive tree (`hyper`, `tower`, `http`, `mio`, `bytes`, …) — ~40
  crates — enter `Cargo.lock`. Every license is in `deny.toml`'s allow-list; `cargo deny check`
  passes (advisories / bans / licenses / sources all `ok`).
- `THIRD-PARTY-LICENSES.md` is **regenerated** (`just licenses` / `cargo about generate`) so the
  notice-preservation record picks the new crates up (the drift gate, `scripts/checks/licenses.sh`).
- `[workspace.dependencies]` gains `tokio` + `axum` (single-point version pins, DN-17). No other
  crate references them.

## 4. Alternatives considered

- **Synchronous `std::net::TcpListener` server (no new deps).** Fully in the existing discipline,
  curl-able, zero supply-chain growth. Rejected by the maintainer in favor of a real, ergonomic
  async framework for a genuinely concurrent server surface.
- **`hyper` alone (no `axum`).** Lower-level; `axum` adds routing/extraction ergonomics on top of
  the same `hyper`/`tower` stack already pulled in, at negligible extra cost.
- **A different runtime (`async-std`/`smol`).** `tokio` is the de-facto ecosystem standard and what
  `axum`/`hyper` target; no reason to diverge.

## 5. Definition of Done

- `mycelium-tero` builds + tests green with `tokio`/`axum` (the M-1017 front + its parity/auth/smoke
  tests). ✅ (this change)
- `cargo deny check` green over the new tree; `THIRD-PARTY-LICENSES.md` current. ✅ (this change)
- The async surface is confined to `mycelium-tero` — `xtask deps` green, no upward-tier edge. ✅
- `Accepted → Enacted` recorded here once the M-1017 front has landed on `main`.

## Changelog

- 2026-07-07 — Created, **Accepted** (maintainer-authorized). Adopts `tokio` + `axum` for the
  `mycelium-tero` HTTP front (M-1017 / DN-87 §2.3) — the workspace's first async runtime, scoped to
  the `tools`-tier tero crate; kernel/library tiers and the MCP front stay synchronous std-only.
  Records the supply-chain consequences (deny green, licenses regenerated) and the considered
  synchronous-server alternative.
