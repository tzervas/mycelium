#!/usr/bin/env bash
# scripts/bootstrap.sh — the ONE thin, universal "make this checkout ready to `just check`"
# entrypoint. Idempotent, skip-graceful, never-silent (CLAUDE.md house rules).
#
# This is the single script every environment's glue calls, so there is exactly one source of
# truth for "what does a fresh Mycelium checkout need" (DRY — CLAUDE.md house rule #5):
#   - Claude Code on the web/mobile — .claude/hooks/session-start.sh calls this.
#   - The portable dev container (.devcontainer/Dockerfile) — postCreateCommand calls this, to
#     fill whatever the image build couldn't finish baking (see docs/CONTAINER.md).
#   - Any other AI coding agent (Grok, etc.) or a human — `bash scripts/bootstrap.sh` on a bare
#     clone is the documented entrypoint (docs/CONTAINER.md "Using this environment from another
#     agent").
#
# SNAPSHOT BAKES THE BULK; THIS SCRIPT APPLIES THE REPO STATE AND BUILDS THE DELTA (maintainer
# directive — see docs/CONTAINER.md). Two parts:
#   1. Toolchain confirmation — a THIN wrapper over the existing scripts/install.sh (no logic
#      duplicated); every component it calls already probes-before-acting, so a re-run on an
#      already-provisioned checkout (the common case: the heavy lifting already happened in an
#      image build or a cached cloud Setup script) is a fast confirmation, not a reinstall.
#      Measured ~1.2-1.5s end-to-end on a fully-warm checkout (docs/CONTAINER.md).
#   2. Delta build — `cargo build --workspace --all-targets --all-features` against the CURRENT
#      (bind-mounted / freshly-cloned) checkout, reusing whatever the snapshot already compiled.
#      Empirically measured on this workspace: ~0.2s when nothing changed since the last build (the
#      common case), vs. ~1m30s-2m10s cold/from-scratch (exactly the cost the image bake absorbs —
#      docs/CONTAINER.md). Bounded by a timeout so a worst-case cold environment can't stall session
#      start; on timeout the first `just check`/`cargo test` simply finishes the job.
# Safe to call on every session start/resume either way — fast in the common case, bounded in the
# worst case, never silent about which.
#
# NEVER installs the pre-commit git hooks by default (--with-hooks opts in): pre-commit's EXTERNAL
# hook repos (pre-commit/pre-commit-hooks, gitleaks/gitleaks) are unreachable from a repo-scoped
# Claude-Code-on-the-web session — the scoped GitHub proxy 403s a clone of any repo other than the
# session's own (CLAUDE.md "Pre-commit in repo-scoped remote sessions"). Installing them here would
# either hang/fail on that fetch or silently no-op; the project's sanctioned path in that
# environment is `--no-verify` + the out-of-band scripts/checks/*.sh gates, not a pre-commit hook
# install. A fully-networked local clone or devcontainer can opt in.
#
# Never bumps a pin: everything below reads the MSRV/Python target from the committed pins
# (rust-toolchain.toml / Cargo.toml) via scripts/install.sh — this script cannot silently drift the
# toolchain version (CLAUDE.md §Toolchain).
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib.sh"
cd "$REPO_ROOT" || exit 1

WITH_HOOKS=0
for a in "$@"; do
  case "$a" in
    --with-hooks) WITH_HOOKS=1 ;;
    -h|--help)
      cat <<'EOF'
scripts/bootstrap.sh — thin, idempotent, skip-graceful residual setup for a fresh checkout.

Usage: bash scripts/bootstrap.sh [--with-hooks]

  --with-hooks   also install the pre-commit git hooks (`just hooks`). Needs UNRESTRICTED GitHub
                 access to fetch pre-commit's external hook repos — NOT available from a
                 repo-scoped Claude-Code-on-the-web session (CLAUDE.md). Fine on a fully-networked
                 local clone or devcontainer.

A thin wrapper over `scripts/install.sh --rust --python --checks` (never duplicates its logic).
Every step probes before acting, so this is fast (~1-2s) when the toolchain is already present —
the common case when called from an image build's postCreateCommand or a warm cloud session.
See docs/CONTAINER.md for how this fits Claude Code on the web, the devcontainer, and other agents.
EOF
      exit 0 ;;
    *) fail "bootstrap.sh: unknown arg '$a' (try --help)"; exit 2 ;;
  esac
done

section "mycelium bootstrap — residual setup (rust + python + check tools$( [[ $WITH_HOOKS -eq 1 ]] && printf ' + pre-commit hooks'))"

COMPONENTS=(--rust --python --checks)
[[ $WITH_HOOKS -eq 1 ]] && COMPONENTS+=(--hooks)

# MYCELIUM_SKIP_OPTIONAL_CARGO keeps this thin: the cargo-modules/depgraph/public-api introspection
# tools are the slow compile tail and aren't part of `just check` (scripts/install-tools.sh header
# comment) — never worth paying for on a per-session bootstrap. A caller that DOES want them (e.g.
# building the full devcontainer image, which has build-time budget to spare) can override by
# exporting MYCELIUM_SKIP_OPTIONAL_CARGO=0 before calling this script.
MYCELIUM_SKIP_OPTIONAL_CARGO="${MYCELIUM_SKIP_OPTIONAL_CARGO:-1}" \
  bash "$SCRIPT_DIR/install.sh" "${COMPONENTS[@]}"

# "Apply the repo state, build the DELTA" (maintainer directive) — the snapshot (this container's
# base image's `built` stage, or a prior bootstrap run this session) already paid for a full
# `--all-features` build as of ITS point in time; this brings that up to date against the CURRENT
# checkout. cargo's own fingerprinting does the "what changed" work for free — this is just the
# same invocation the image baked, re-run against whatever's different now.
#
# Empirically measured on this workspace (docs/CONTAINER.md "Snapshot bakes the bulk; launch builds
# the delta" records the method): ~0.2s when nothing changed since the last build (the common case —
# a warm image or a resumed session), ~1m30s-2m10s for a truly cold/from-scratch build (exactly the
# cost the image bake exists to absorb, so a session almost never pays it). Bounded by a timeout so
# a worst-case cold environment (this script running WITHOUT the baked image) can't stall session
# start indefinitely — never-silent: on timeout or a real compile error, this prints what happened
# and moves on rather than blocking the session; nothing is lost, only deferred to the first
# `just check`/`cargo test` a user or agent runs.
if have cargo; then
  section "build the delta (cargo build --workspace --all-targets --all-features, bounded)"
  if cargo fetch --locked --quiet 2>/dev/null; then
    ok "cargo fetch: dependency registry reachable and warm"
  else
    skip "cargo fetch failed (offline, or Cargo.lock drift) — the build below will surface it if it matters"
  fi
  build_err="$(mktemp "${TMPDIR:-/tmp}/myc-bootstrap-build.XXXXXX")"
  budget="${MYCELIUM_BOOTSTRAP_BUILD_TIMEOUT:-240}"
  build_rc=0
  timeout "$budget" cargo build --workspace --all-targets --all-features --locked --quiet 2>"$build_err" || build_rc=$?
  if [[ $build_rc -eq 0 ]]; then
    ok "cargo build: workspace is up to date with the current checkout (delta applied)"
  elif [[ $build_rc -eq 124 ]]; then
    skip "cargo build: exceeded the ${budget}s bootstrap budget — still incomplete; the first \`just check\`/\`cargo test\` will finish it"
  else
    skip "cargo build reported errors — surfacing, not fixing (never-silent; this script's job is setup, not the fix):"
    tail -20 "$build_err" | sed 's/^/    /'
  fi
  rm -f "$build_err"
fi

echo
ok "bootstrap done — \`just check\` (or \`just test-fast\` for the sub-second pre-commit loop) is ready to run"
[[ $WITH_HOOKS -eq 0 ]] && echo "  (pre-commit hooks were NOT installed — pass --with-hooks on a fully-networked clone, or see CLAUDE.md's --no-verify guidance for repo-scoped remote sessions)"
exit 0
