# syntax=docker/dockerfile:1
# .devcontainer/Dockerfile — Mycelium's PORTABLE BASE IMAGE: a clean, reproducible, snapshot-
# friendly environment for a `just check`-ready checkout, with minimal follow-on bootstrap.
#
# WHO THIS IS FOR (cross-tool, not Claude-Code-specific): any tool that can point at a custom
# container/OCI image needs no manual per-session toolchain setup here — VS Code Dev Containers,
# GitHub Codespaces, a GitHub Actions `container:` job, a local `docker run`, or another AI coding
# agent (Grok, etc.) that supports a custom base image. See docs/CONTAINER.md for the full picture,
# including the one platform this does NOT plug into directly today: Claude Code on the web does
# not yet support a custom base image (Anthropic docs, verified 2026-07-09) — there, the equivalent
# "bake the bulk" layer is the cloud environment's Setup-script field (see scripts/install-tools.sh
# and docs/CONTAINER.md), with a thin SessionStart hook (.claude/hooks/session-start.sh) on top; this
# image can still be used there as a `docker compose` sidecar container if ever needed.
#
# BAKED IN AT IMAGE-BUILD TIME (so a fresh session's follow-on bootstrap — scripts/bootstrap.sh —
# is a ~1-2s confirmation, not an install; measured on a fully-warm checkout, see docs/CONTAINER.md):
#   - Debian/Ubuntu base + git/curl/build-essential/pkg-config/libssl-dev
#   - Rust, rustup-managed, pinned to the repo MSRV (1.96.1 — rust-toolchain.toml / ADR-041). NEVER
#     bump the RUST_TOOLCHAIN arg's default without an ADR (CLAUDE.md §Toolchain) — override at
#     build time only to test drift, never to silently track "latest stable".
#   - Python 3.13 + uv (CONTRIBUTING.md's Python toolchain), Node >=20 (markdownlint-cli2 floor)
#   - just, cargo-nextest, and every scripts/checks/ tool (shellcheck, codespell, yamllint,
#     markdownlint-cli2, gitleaks, cargo-deny/audit/mutants/public-api, cargo-modules/depgraph) —
#     via scripts/install-tools.sh itself (COPYed in and RUN here), so there is exactly ONE source
#     of truth for "what tools does Mycelium need", shared with the cloud Setup-script path and
#     scripts/bootstrap.sh (CLAUDE.md house rule #5, DRY). MYCELIUM_SKIP_OPTIONAL_CARGO=0 here
#     (unlike the thin per-session bootstrap) because an image build has minutes to spare, not
#     seconds — "bake in as much as possible" per the maintainer's stated priority.
#   - llc/clang (unversioned) — the ALWAYS-ON direct-LLVM AOT backend (crates/mycelium-mlir's
#     default path; RFC-0004 §2 / ADR-034). This is NOT the optional layer below.
#   - The full workspace's THIRD-PARTY DEPENDENCY compile cache, via cargo-chef (`planner`/`warm`
#     stages) — every crate.io dependency across `--workspace --all-targets --all-features` is
#     fetched AND compiled into a fixed CARGO_TARGET_DIR. Verified safe: no crate in this workspace
#     links a native library at compile time (no `-sys` FFI crate, no `links = `) — mycelium-mlir's
#     `mlir-dialect` feature probes for `mlir-opt`/`mlir-translate` as a runtime subprocess, never a
#     build-time link (RFC-0004 §2/ADR-019), so `--all-features` here never needs the opt-in MLIR layer.
#   - A PRE-BUILT WORKSPACE (the `built` stage): a real `cargo build --workspace --all-targets
#     --all-features` of the actual repository as of image-build time, layered on top of the warm
#     dependency cache above — so "the compile-the-world cost is paid ONCE in the snapshot"
#     (maintainer directive). This is genuinely COPYed + compiled into the image (not bind-mount-only
#     — see below for how that stays compatible with a generic, reusable image).
#
# DELIBERATELY NOT BAKED (opt-in — see the `with-mlir` stage at the bottom):
#   - libMLIR (`mlir-opt`/`mlir-translate`) for the OFF-by-default `mlir-dialect` Cargo feature
#     (ADR-019). The light tiers (`just check`) never need it, and the maintainer holds heavy
#     AOT/VSA/GPU work on the desktop (CLAUDE.md "Heavy checks run on the maintainer's desktop").
#     Bloating the default image with libMLIR dev packages for a feature most builds never enable
#     would work against "lean, not kitchen-sink" — build the `with-mlir` stage instead if you need it.
#
# SNAPSHOT BAKES THE BULK; LAUNCH BUILDS THE DELTA (the split this whole image is designed around —
# see docs/CONTAINER.md for the full rationale). The image's `built` stage above is necessarily "as
# of image-build time" — it goes stale the moment a new commit lands. That is expected: this image
# is meant to be rebuilt periodically (not on every commit), and the gap is closed cheaply at
# session/container-launch time, NOT by rebuilding the image per-commit:
#   - When used as a devcontainer / CI checkout via a BIND MOUNT over /workspace (the normal case —
#     .devcontainer/devcontainer.json does this), the mount transparently overlays the baked source
#     with the CURRENT checkout. CARGO_TARGET_DIR is a FIXED image-wide path (not under /workspace),
#     so it is untouched by the mount, and a `cargo build` against the new source reuses every
#     artifact whose fingerprint didn't change (all third-party deps, unchanged first-party crates)
#     and recompiles only what actually changed — a fast DELTA build, not a from-scratch one.
#   - scripts/bootstrap.sh (called by the devcontainer's postCreateCommand, and by Claude Code's
#     .claude/hooks/session-start.sh) is exactly this: apply the current repo state (already true —
#     it's the bind-mounted checkout) and build the delta, plus the minimal residual (env, a fast
#     readiness check). Nothing heavy runs in that launch path — the heavy work already happened here.
#
# Build:  docker build -f .devcontainer/Dockerfile -t mycelium-dev .
# Run:    docker run --rm -it -v "$PWD":/workspace mycelium-dev
# (The devcontainer CLI / VS Code Dev Containers / GitHub Codespaces do this automatically via
# .devcontainer/devcontainer.json.)

ARG BASE_IMAGE=docker.io/library/ubuntu:24.04

# ── Stage: toolchain — OS + pinned Rust/Python/uv/Node + every project check-tool ────────────────
FROM ${BASE_IMAGE} AS toolchain
ARG RUST_TOOLCHAIN=1.96.1
ARG PYTHON_VERSION=3.13
ARG UV_VERSION=0.8.17
ARG NODE_VERSION=22.22.2

LABEL org.opencontainers.image.title="mycelium-dev" \
      org.opencontainers.image.description="Mycelium portable dev/CI base image — Rust ${RUST_TOOLCHAIN} + Python ${PYTHON_VERSION}/uv + just + full check-tool set; libMLIR is a separate opt-in stage (with-mlir)." \
      org.opencontainers.image.source="https://github.com/tzervas/mycelium" \
      org.opencontainers.image.licenses="MIT"

ENV DEBIAN_FRONTEND=noninteractive \
    CARGO_HOME=/root/.cargo \
    RUSTUP_HOME=/root/.rustup \
    CARGO_TARGET_DIR=/opt/mycelium-target-cache \
    PATH=/root/.local/bin:/root/.cargo/bin:/usr/local/bin:$PATH

# Base OS deps + the ALWAYS-ON llc/clang (see header). One RUN so the apt lists never survive into
# a layer (Docker layers are additive — `rm` in a LATER layer does not shrink an earlier one).
RUN apt-get update && apt-get install -y --no-install-recommends \
        git curl ca-certificates build-essential pkg-config libssl-dev \
        llvm clang \
        python3 python3-venv python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Node >=20 (markdownlint-cli2's floor; Ubuntu 24.04's distro `nodejs` package is 18 — see
# scripts/install-tools.sh's own comment on this exact gap). Official pinned tarball fetched over
# TLS and extracted directly — NOT curl|bash (CLAUDE.md security policy: "no curl|bash, no piping a
# remote download to a shell"; a tarball fetch-then-extract pipes nothing to an interpreter).
RUN curl --proto '=https' --tlsv1.2 -fsSL -o /tmp/node.tar.xz \
      "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-x64.tar.xz" \
    && tar -xJf /tmp/node.tar.xz -C /usr/local --strip-components=1 \
    && rm -f /tmp/node.tar.xz \
    && node --version && npm --version

# Rust — pinned to the repo MSRV (rust-toolchain.toml / ADR-041); NEVER bump this ARG default
# without an ADR. rustup verifies the SHA-256 of every component it downloads, so the toolchain
# stays integrity-checked even though the *installer script itself* is fetched via a TLS-pinned
# curl|sh of the official, well-known rustup.rs installer (the one curl|bash exception this repo's
# own scripts already make — see experiments/docker/Dockerfile's identical pattern).
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --profile minimal --default-toolchain "${RUST_TOOLCHAIN}" --component clippy,rustfmt

# uv — a pinned version from PyPI (TLS + package-index verified), not curl|sh of a moving installer.
RUN python3 -m pip install --break-system-packages --no-cache-dir "uv==${UV_VERSION}" \
    && uv python install "${PYTHON_VERSION}"

# cargo-chef — ONLY used at image-build time (the planner/warm stages below) to pre-bake the
# dependency-compile cache. It is a build-time-only tool, deliberately NOT part of
# scripts/install-tools.sh (which installs the project's actual dev/check tools).
RUN cargo install --locked cargo-chef

# Every scripts/checks/ tool (shellcheck, codespell, yamllint, markdownlint-cli2 warm, gitleaks,
# cargo-deny/audit/mutants/nextest/public-api, just, pre-commit) — reusing the EXISTING, tested
# installer verbatim rather than re-deriving a package list here (DRY). install-tools.sh touches no
# other repo file, so it can run before the real source ever gets COPYed in.
WORKDIR /workspace-setup
COPY scripts/lib.sh scripts/install-tools.sh ./scripts/
RUN apt-get update \
    && MYCELIUM_SKIP_OPTIONAL_CARGO=0 bash ./scripts/install-tools.sh \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /
RUN rm -rf /workspace-setup
WORKDIR /workspace

# ── Stage: planner — cargo-chef needs the full manifest graph to plan the dependency build. This
# stage's own layer invalidates on every source change (cheap: `cargo chef prepare` only parses
# manifests, it compiles nothing) but it is NEVER copied into the final image — only its
# recipe.json output is (next stage) — so ordinary source edits never bust the expensive layer.
FROM toolchain AS planner
WORKDIR /workspace
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage: warm — bake the THIRD-PARTY DEPENDENCY compile cache. This is cargo-chef's entire
# point: this layer invalidates ONLY when Cargo.lock/Cargo.toml actually change, never on a
# first-party .rs edit, because it is built from recipe.json alone (no other repo file is copied
# in here). This is the layer that survives almost every image rebuild.
FROM toolchain AS warm
WORKDIR /workspace
COPY --from=planner /workspace/recipe.json recipe.json
RUN cargo chef cook --workspace --all-targets --all-features --recipe-path recipe.json \
    && rm -f recipe.json

# ── Stage: built — the PRE-BUILT WORKSPACE (maintainer directive: bake a real `cargo build`, not
# just dependency stubs, so "the compile-the-world cost is paid ONCE in the snapshot"). Copies the
# real repository as of image-build time and compiles it for real, on top of the already-warm
# dependency artifacts from `warm` — so this layer's own cost is just the first-party crates, not
# the whole dependency tree. This is the one layer that invalidates on every source commit if the
# image is rebuilt per-commit; that is expected and fine — periodic image rebuilds are the point,
# and a session's own launch script (.claude/hooks/session-start.sh -> scripts/bootstrap.sh) closes
# whatever gap remains by building only the DELTA against the current checkout (see docs/CONTAINER.md
# "Snapshot bakes the bulk; launch builds the delta").
FROM warm AS built
WORKDIR /workspace
COPY . .
RUN cargo build --workspace --all-targets --all-features --locked

# ── Final stage: the portable base image. WORKDIR /workspace holds a real, buildable, testable
# snapshot of the repo as of image-build time (useful standalone, e.g. `docker run` with no mount).
# When used as a devcontainer / CI checkout via a BIND MOUNT over /workspace (the normal case —
# .devcontainer/devcontainer.json does this), the mount transparently overlays this baked source
# with the current checkout; CARGO_TARGET_DIR stays a FIXED image-wide path (not under /workspace),
# so it is unaffected by the mount and `cargo build`/`test`/`clippy` against the NEW bind-mounted
# source reuses every warm artifact whose fingerprint didn't change and recompiles only the delta —
# exactly the "snapshot bakes the bulk; launch builds the delta" split (see docs/CONTAINER.md).
FROM built AS final
WORKDIR /workspace
CMD ["bash"]

# ── Optional stage: libMLIR for the OFF-by-default `mlir-dialect` feature (ADR-019). NOT part of
# the default `final` target — build it explicitly when you need it:
#   docker build -f .devcontainer/Dockerfile --target with-mlir -t mycelium-dev:mlir .
# Reuses the project's own scripts/setup-mlir.sh (version-matches the installed LLVM major) rather
# than hand-picking a libmlir-N-dev package here — one source of truth, same as the toolchain stage.
FROM final AS with-mlir
WORKDIR /workspace-setup
COPY scripts/lib.sh scripts/setup-mlir.sh ./scripts/
# apt-get update: the toolchain stage already dropped its apt lists to keep the default image lean
# (see above) — this opt-in stage re-fetches them for its own install, then drops them again.
RUN apt-get update \
    && bash ./scripts/setup-mlir.sh \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /
RUN rm -rf /workspace-setup
WORKDIR /workspace
