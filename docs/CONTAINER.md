# Container & cloud-session environment

How Mycelium gets a clean, reproducible, `just check`-ready environment with minimal follow-on
bootstrap — for a local devcontainer, Claude Code on the web/mobile, GitHub Actions, and any other
AI coding agent that wants to work in this repo without the maintainer doing manual per-session
toolchain setup.

## The split: snapshot bakes the bulk, launch builds the delta

One design, two consumers. All the heavy, slow, unchanging work — the toolchain, every check tool,
a warmed cargo registry, and a real pre-built workspace — is baked **once** into a snapshot. The
per-session/per-launch script's job is only ever: apply the current repo state, build the
**delta** (an incremental recompile of whatever changed since the snapshot — fast, because
dependencies and most first-party crates are already compiled), and do the minimal residual (env
vars, a bounded readiness build). Nothing heavy runs in the launch path.

```text
┌─────────────────────────────┐        ┌──────────────────────────────────┐
│   SNAPSHOT (baked once)      │        │   LAUNCH (every session, fast)     │
│                               │        │                                    │
│  .devcontainer/Dockerfile     │  --->  │  scripts/bootstrap.sh              │
│  - OS + Rust 1.96.1 + Python  │        │  - confirm toolchain present       │
│    3.13/uv + Node + just      │        │    (~1-1.5s when already baked)    │
│  - every scripts/checks/ tool │        │  - cargo build --workspace         │
│  - warmed cargo registry      │        │    --all-targets --all-features    │
│  - a REAL pre-built workspace │        │    (~0.2s when nothing changed,    │
│    (cargo build, all-features)│        │    bounded by a timeout otherwise) │
└─────────────────────────────┘        └──────────────────────────────────┘
```

Two consumers share this exact split (§Deliverables below): the portable **devcontainer** (any
tool that can point at a custom OCI image) and **Claude Code on the web**, whose platform doesn't
yet support a custom base image — there, the "snapshot" role is played by the cloud environment's
**Setup script** field instead of a Dockerfile (see §Claude Code on the web).

## Deliverables

| File | Role |
|---|---|
| `.devcontainer/Dockerfile` | The portable base image — the snapshot, for any tool that can consume a custom OCI image (VS Code Dev Containers, GitHub Codespaces, a GitHub Actions `container:` job, plain `docker run`, or another AI agent). |
| `.devcontainer/devcontainer.json` | Points VS Code / the `devcontainer` CLI at the Dockerfile; `postCreateCommand` runs the launch script. |
| `scripts/bootstrap.sh` | The ONE thin, universal launch script. Called by the devcontainer's `postCreateCommand`, by Claude Code's SessionStart hook, and directly by any other agent or a human on a bare clone. |
| `.claude/hooks/session-start.sh` | Claude-Code-specific glue: gates on `$CLAUDE_CODE_REMOTE`, then calls `scripts/bootstrap.sh`. Written and validated; **not yet wired into `.claude/settings.json`** — see §Claude Code on the web for why and the exact patch. |
| `scripts/install-tools.sh` | The existing, already-tested check-tool installer (predates this work) — the single source of truth for "what tools does Mycelium need", reused verbatim by both the Dockerfile (image-build time) and the cloud Setup-script field (see below). Not duplicated anywhere. |

## The base image (`.devcontainer/Dockerfile`)

Multi-stage build (`docker build -f .devcontainer/Dockerfile -t mycelium-dev .`):

1. **`toolchain`** — Ubuntu 24.04 + git/curl/build-essential/pkg-config/libssl-dev + the
   always-on `llc`/`clang` (the default direct-LLVM AOT path, RFC-0004 §2/ADR-034 — **not** the
   optional MLIR layer, see below) + Node ≥20 (a pinned official tarball, not curl|bash — the
   markdownlint-cli2 floor; Ubuntu 24.04's distro `nodejs` package is 18) + Rust 1.96.1 via rustup
   (pinned to the repo MSRV — `rust-toolchain.toml`/ADR-041; **never** bump the `RUST_TOOLCHAIN`
   build arg's default without an ADR) + Python 3.13 + uv + `cargo-chef` (build-time only) + every
   `scripts/checks/` tool, by literally `COPY`-ing in and running `scripts/install-tools.sh`
   (`MYCELIUM_SKIP_OPTIONAL_CARGO=0` here, unlike the thin launch path — an image build has minutes
   to spare, so it also bakes the `cargo-modules`/`cargo-depgraph`/`cargo-public-api` introspection
   tools that the per-session bootstrap deliberately skips). It also provisions a **nightly**
   toolchain (minimal profile + `rustdoc`) that `cargo public-api` (the `just api` surface gate) uses
   **only** to build rustdoc-JSON — a nightly-only rustdoc feature; the MSRV-pinned 1.96.1 still
   builds every real artifact (ADR-041), so the nightly never touches a shipped build. Pin it with
   `--build-arg API_TOOLCHAIN=nightly-YYYY-MM-DD` (image build) or `MYC_API_TOOLCHAIN=…` (the
   Setup-script / `just api` / `just api-baseline` paths — one knob, install and gate never disagree)
   so the surface baselines stay reproducible across rebuilds. When no nightly is present the surface
   gate **skips** (never fails) — an absent introspection toolchain leaves the surface unverified,
   not regressed.
2. **`planner`** — copies the full repo (this stage is never copied into the final image) and runs
   `cargo chef prepare` to produce a manifest-only `recipe.json`.
3. **`warm`** — copies only `recipe.json` and runs `cargo chef cook --workspace --all-targets
   --all-features`, compiling every third-party dependency into a fixed `CARGO_TARGET_DIR`
   (`/opt/mycelium-target-cache`, not under `/workspace`). This is the layer that survives almost
   every image rebuild — it invalidates only when `Cargo.lock`/`Cargo.toml` actually change, never
   on a first-party `.rs` edit (that's cargo-chef's entire purpose). Verified safe to run
   `--all-features` unconditionally: no crate in this workspace links a native library at compile
   time (no `-sys` FFI crate, no `links = "..."` key anywhere in the workspace's `Cargo.toml`s) —
   `mycelium-mlir`'s `mlir-dialect` feature probes for `mlir-opt`/`mlir-translate` as a
   **runtime** subprocess, never a build-time link (RFC-0004 §2/ADR-019), so it never needs the
   opt-in MLIR layer even when enabled.
4. **`built`** — copies the real repository (as of image-build time) over the `warm` stage's stub
   skeleton and runs a real `cargo build --workspace --all-targets --all-features --locked`. This
   is the **pre-built workspace**: the compile-the-world cost (dependencies *and* first-party
   crates) is paid once here. This layer necessarily goes stale the moment a new commit lands —
   that's expected; the image is meant to be rebuilt periodically, not per-commit, and
   `scripts/bootstrap.sh` closes whatever gap remains at launch time (see below).
5. **`final`** — the default build target. `WORKDIR /workspace` holds the baked source (useful
   standalone), but the normal use is a **bind mount** over `/workspace` (what
   `devcontainer.json`/CI checkout actions do) — the mount transparently overlays the baked source
   with the current checkout. `CARGO_TARGET_DIR` stays a fixed, non-`/workspace` path, so it is
   untouched by the mount, and a `cargo build` against the new bind-mounted source reuses every
   artifact whose fingerprint didn't change and recompiles only the delta.
6. **`with-mlir`** *(opt-in — not part of the default `final` target)* — adds libMLIR
   (`mlir-opt`/`mlir-translate`) for the off-by-default `mlir-dialect` Cargo feature (ADR-019), by
   reusing the project's own `scripts/setup-mlir.sh` (version-matches the installed LLVM major).
   Build explicitly: `docker build -f .devcontainer/Dockerfile --target with-mlir -t mycelium-dev:mlir .`

### The LLVM vs. MLIR split (read this before assuming "LLVM is optional")

The original brief for this work assumed `mycelium-mlir` broadly "wants LLVM/MLIR" and that the
whole thing should be one optional layer. Checking the actual crate (`crates/mycelium-mlir/Cargo.toml`,
`src/llvm.rs`) shows a sharper split, which this image follows:

- **`llc`/`clang` (unversioned) are baked into the default image, always.** They back the
  **default, always-on** direct-LLVM AOT path (`crates/mycelium-mlir/src/llvm.rs`, RFC-0004 §2 /
  ADR-034) — this is not a Cargo feature at all, just a runtime tool probe. This is small and
  cheap (an `llvm`/`clang` apt package), not the heavy part.
- **libMLIR (`mlir-opt`/`mlir-translate`) is the genuinely heavy, opt-in part** — dev headers and
  static libraries for the off-by-default `mlir-dialect` Cargo feature (ADR-019), which even when
  *enabled* only probes for these tools at runtime and degrades gracefully if absent. This is what
  stays out of the default image (the `with-mlir` stage above), matching the project's own
  `scripts/setup-mlir.sh` / `just setup-mlir` convention (deliberately kept out of `just setup` too
  — "an off-by-default feature must not apt-install/sudo-prompt by default").

Net effect: the default image is lean (no libMLIR dev packages) while still supporting the
project's actual default build/test/AOT path out of the box.

## Claude Code on the web

**Read this before assuming the base image above is what a Claude Code cloud session runs in — it
is not**, and that matters for how the pieces below fit together.

### What the platform actually supports (verified against the current docs, 2026-07-09)

Per <https://code.claude.com/docs/en/claude-code-on-the-web>: *"Replacing the base image with your
own Docker image is not yet supported. Use a setup script to install what you need on top of the
provided image, or run your image as a container alongside Claude with `docker compose`."*

So `.devcontainer/Dockerfile` cannot be the literal sandbox a Claude Code web/mobile session boots
from today. The platform instead offers two other mechanisms, and the "snapshot bakes the bulk /
launch stays thin" split maps onto them directly:

| | **Setup script** | **SessionStart hook** |
|---|---|---|
| Attached to | The cloud environment (a UI setting) | This repository (`.claude/settings.json`) |
| Runs | Once, before Claude Code launches, only when no cached environment exists | Every session start **and resume** |
| Cached? | Yes — filesystem-snapshotted after it completes; skipped on later sessions until the environment's setup script/allowed-hosts change or the ~7-day cache expiry | No — always runs, so it must stay fast |
| Budget | ~5 minutes (documented) | Should be seconds, not minutes |
| Configured by | Pasting a script into the environment's **Setup script** field (cloud environment UI) | Committing `.claude/hooks/session-start.sh` + registering it in `.claude/settings.json` |
| Plays the role of | **the snapshot** (the "bake the bulk" layer) | **the launch script** (apply repo + build the delta) |

This is the same relationship as the Dockerfile vs. `scripts/bootstrap.sh`, realized through the
mechanism this platform actually has today.

### The Setup-script field: paste this in

Set the cloud environment's **Setup script** field to:

```bash
#!/bin/bash
bash scripts/install-tools.sh
```

`scripts/install-tools.sh` already existed before this work and was already written and documented
(in its own header) for exactly this purpose — it is the same script the devcontainer image's
`toolchain` stage runs, so there is one source of truth for "what tools does Mycelium need" between
the portable image and the cloud Setup-script path. The cloud base image already ships a generic
Rust/Python/Node/uv (per the platform's own "Installed tools" table); `install-tools.sh` layers the
project's pinned versions and the rest of the `scripts/checks/` tool set on top, and is idempotent
— a cache rebuild (rare) re-runs it safely.

### The SessionStart hook: written, validated, **not yet registered**

`.claude/hooks/session-start.sh` is written, executable, and validated (see §Verification below):
gated on `$CLAUDE_CODE_REMOTE` (a no-op in local sessions, per Anthropic's documented convention),
it calls `scripts/bootstrap.sh` — the thin launch script that confirms the toolchain (fast when the
Setup script already ran) and builds the delta.

**It is not yet wired into `.claude/settings.json`.** This was a deliberate choice, not an
oversight: `.claude/settings.json` holds this repo's `permissions` and `hooks` configuration, and
per this agent's standing guardrails, no in-conversation agent instruction — including a
maintainer's directive relayed through a coordinator — can authorize a change to that file; only a
human's own message or the permission system itself can. The hook file exists and works; applying
the following merge to `.claude/settings.json`'s `hooks` block is a one-step, human-applied action
(the array form matches Anthropic's own documented example):

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|resume",
        "hooks": [
          {
            "type": "command",
            "command": "\"$CLAUDE_PROJECT_DIR\"/.claude/hooks/session-start.sh"
          }
        ]
      }
    ]
  }
}
```

(Merge this into the existing `hooks` object — keep the current `PreToolUse` branch-guard entry
alongside it; don't replace the file.)

### Why not just use the portable image anyway?

The docs do leave one path open: *"run your image as a container alongside Claude with `docker
compose`."* Cloud sessions do have `docker`/`dockerd`/`docker compose` pre-installed. If a future
need calls for the exact pinned image rather than the cloud base image + Setup script, a
`docker-compose.yml` pointing at `.devcontainer/Dockerfile` would work as a sidecar — this isn't
built out here (out of scope for this change, and the Setup-script path already gets the pinned
toolchain without the added complexity of a sidecar container), but the image is ready for it if
ever wanted.

## The devcontainer (local / VS Code / Codespaces / CI)

```bash
# Build
docker build -f .devcontainer/Dockerfile -t mycelium-dev .

# Run, with the repo bind-mounted (the normal use)
docker run --rm -it -v "$PWD":/workspace mycelium-dev
```

Or open the repo in VS Code with the Dev Containers extension / GitHub Codespaces — both read
`.devcontainer/devcontainer.json` automatically and run `scripts/bootstrap.sh` as the
`postCreateCommand`.

A GitHub Actions job can use the same image via `jobs.<job>.container: mycelium-dev` (built or
pulled from a registry) instead of the current `runs-on: ubuntu-latest` + per-step tool installs in
`.github/workflows/checks.yml` — not changed here (that workflow is manual-dispatch/advisory per
CLAUDE.md's Remote CI policy, and swapping its base is a separate decision), but the image is ready
for it.

## Using this environment from Grok or another AI coding agent

Two paths, depending on what the tool supports:

- **The tool can point at a custom container/base image:** use `.devcontainer/Dockerfile` directly
  — `docker build -f .devcontainer/Dockerfile -t mycelium-dev .`, then run with the repo
  bind-mounted at `/workspace`. Everything needed for `just check` is already baked in.
- **The tool only supports a launch/setup script hook (the CC-web shape):** point it at
  `bash scripts/bootstrap.sh` on session/task start. It is tool-agnostic (no Claude-Code-specific
  assumptions beyond an optional `$CLAUDE_ENV_FILE`/`$CLAUDE_PROJECT_DIR`, both used only if
  present), idempotent, and safe to call every time — fast when the toolchain and build are already
  warm, bounded (`MYCELIUM_BOOTSTRAP_BUILD_TIMEOUT`, default 240s) when they are not.

In both cases the underlying logic is identical (`scripts/install-tools.sh` +
`scripts/bootstrap.sh`) — there is exactly one recipe for "what does Mycelium need", not a
per-tool fork of it.

## Verification status (honesty per CLAUDE.md's transparency rule)

Tagged per the project's own accuracy lattice (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`) rather than
claimed as uniformly "done":

- **`scripts/bootstrap.sh` and `.claude/hooks/session-start.sh` — `Empirical`, directly measured**
  in this repo's actual sandbox (itself a live Claude-Code-remote-style environment): a full
  `bootstrap.sh` run (toolchain confirmation + delta build) took **1.5-1.7s** end-to-end when the
  toolchain and workspace were already warm; the `CLAUDE_CODE_REMOTE` gate, the `$CLAUDE_ENV_FILE`
  write, and the build-timeout fallback path were each exercised directly and behaved as designed;
  `shellcheck -x -e SC1091` (the exact invocation `scripts/checks/shell.sh` uses) is clean on both
  files; a full `cargo test -p mycelium-core --lib` (354 tests) passed using exactly the toolchain
  the hook ensures.
- **Delta-build timings — `Empirical`, directly measured** on this workspace: a cold
  `cargo build --workspace --all-targets --locked` (nothing pre-built) took 1m33s; adding
  `--all-features` from there took a further 39.6s; a steady-state no-op rebuild of the exact
  `cargo build --workspace --all-targets --all-features --locked` invocation `bootstrap.sh` uses
  took 0.24s. These ground the "~0.2s common case, ~1m30s-2m10s cold case" figures used above.
- **`.devcontainer/Dockerfile` — `Declared`, not build-verified end-to-end.** `docker` is present in
  this sandbox but `dockerd` cannot start here (`ulimit: error setting limit (Operation not
  permitted)` — a sandbox restriction, not a Dockerfile defect), so `docker build` could not be run.
  What *was* done: a complete manual re-read for stage-reference correctness, `ARG`/`WORKDIR`
  scoping across stages, the cargo-chef stub-then-real-source overwrite mechanics, and the apt-list
  lifecycle per layer; every external script the Dockerfile calls (`scripts/install-tools.sh`,
  `scripts/setup-mlir.sh`) already exists and is independently exercised elsewhere in this repo;
  `hadolint` was attempted (apt package absent on this base; a pinned GitHub-release binary fetch
  hit a 403 through the environment's proxy) and not obtained, so no static Dockerfile linter ran
  either. **Recommended before relying on this image: `docker build -f .devcontainer/Dockerfile -t
  mycelium-dev .` on a host with a working Docker daemon**, to confirm what static review alone
  cannot.
- **The Setup-script recommendation and the `.claude/settings.json` SessionStart-hook patch** are
  grounded directly in Anthropic's current documentation (fetched and read in full during this
  work, cited above) — `Empirical` as a reading of that source, `Declared` as to whether it matches
  the maintainer's actual cloud environment configuration (not something this agent can inspect).

## FAQ

**Why does the pre-built `built` stage go stale, and is that a problem?** It reflects the repo as
of image-build time; every commit after that is a "delta" the launch script closes. This is
intentional — the image is meant to be rebuilt periodically (e.g. on a schedule, or when
`Cargo.lock` churns a lot), not on every commit. A very stale image just means a longer (but still
dependency-warm, so much faster than cold) delta build at launch.

**Why not use `--release` when warming the build?** `scripts/checks/test.sh` and `lint.sh` (the
gates `just check` runs) both build in the default (debug) profile — matching that profile is what
makes the warm cache actually get hit; a `--release` warm would be a different cache Cargo never
reuses for a debug build.

**Does this bump the Rust/Python version pins?** No. `RUST_TOOLCHAIN`/`PYTHON_VERSION` are build
args whose *defaults* mirror the committed pins (`rust-toolchain.toml`, `Cargo.toml`); changing
those pins is still an ADR decision (CLAUDE.md §Toolchain), not a container detail.
