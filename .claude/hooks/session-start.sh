#!/usr/bin/env bash
# .claude/hooks/session-start.sh — Claude Code SessionStart hook, cloud/web-only.
#
# THIN AND FAST by design (see docs/CONTAINER.md "Claude Code on the web" for the full picture):
# per Anthropic's own SessionStart-hook guidance, this hook "runs each time a session starts or
# resumes, unlike setup scripts which benefit from environment caching" — so it must stay cheap.
# The BULK of toolchain provisioning belongs one layer up, in the cloud environment's **Setup
# script** field (cached across sessions, ~5-minute budget, only reruns on cache-miss) — that
# field should be set to `bash scripts/install-tools.sh` (already written and documented for
# exactly this; see its own header comment). This hook is the belt-and-suspenders gap-filler for
# whatever a cold/uncached session still needs, plus the truly per-session bits that can never be
# baked (env exports, a cheap dependency-cache sanity check).
#
# Cloud-only: a local session already has its own toolchain (or the developer runs
# `bash scripts/bootstrap.sh` / `just setup` by hand) — running this there would only add latency
# for no benefit. Gated on $CLAUDE_CODE_REMOTE, per Anthropic's documented convention.
#
# Idempotent + non-interactive + never-silent: delegates entirely to scripts/bootstrap.sh, which is
# a thin wrapper over the existing, already-idempotent scripts/install.sh (no logic duplicated here
# — CLAUDE.md house rule #5, DRY). Measured ~1.2s end-to-end on an already-warm checkout
# (docs/CONTAINER.md), so running this synchronously on every startup/resume is cheap.
set -euo pipefail

if [[ "${CLAUDE_CODE_REMOTE:-}" != "true" ]]; then
  exit 0
fi

# Resolve the repo root the documented way ($CLAUDE_PROJECT_DIR); fall back to this script's own
# location so `bash .claude/hooks/session-start.sh` still works stand-alone for local testing.
PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
BOOTSTRAP="$PROJECT_DIR/scripts/bootstrap.sh"

# Keep PATH correct for this session's later Bash tool calls. Belt-and-suspenders only: the cloud
# base image already puts these on PATH by default (verified empirically — see docs/CONTAINER.md),
# so this is cheap insurance against a future image change, not a step this hook depends on.
if [[ -n "${CLAUDE_ENV_FILE:-}" ]]; then
  # shellcheck disable=SC2016  # intentional: written verbatim for the session to expand later,
  # not expanded now — CLAUDE_ENV_FILE is sourced by the harness, not by this script.
  echo 'export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"' >> "$CLAUDE_ENV_FILE"
fi

if [[ ! -f "$BOOTSTRAP" ]]; then
  echo "session-start: scripts/bootstrap.sh not found at $BOOTSTRAP — skipping (nothing to bootstrap)" >&2
  exit 0
fi

bash "$BOOTSTRAP"
