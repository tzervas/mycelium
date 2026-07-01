#!/usr/bin/env bash
# scripts/docs-container.sh — build/run the local docs site container (docs/Containerfile).
# Advisory tooling; NOT part of `just check`. Prefers podman, falls back to docker, errors clearly
# if neither is present (never silent — G2).
#
# Usage:
#   scripts/docs-container.sh build   # podman/docker build -t mycelium-docs -f docs/Containerfile .
#   scripts/docs-container.sh run     # podman/docker run --rm -p 8080:8000 mycelium-docs
#
# Also reachable via `just docs-container-build` / `just docs-container-run`.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib.sh"
cd "$REPO_ROOT" || exit 1

IMAGE="mycelium-docs"
PORT="${MYC_DOCS_PORT:-8080}"

section "docs container (docs/Containerfile — advisory, not a gate)"

if have podman; then
  ENGINE=podman
elif have docker; then
  ENGINE=docker
  skip "podman not found — falling back to docker"
else
  fail "neither podman nor docker is installed — install one to build/run the docs container"
  echo "  Podman: https://podman.io/docs/installation"
  echo "  Docker: https://docs.docker.com/engine/install/"
  exit 1
fi
ok "container engine: $ENGINE"

cmd="${1:-}"
case "$cmd" in
  build)
    section "building $IMAGE ($ENGINE build -f docs/Containerfile .)"
    "$ENGINE" build -t "$IMAGE" -f docs/Containerfile .
    ok "built $IMAGE"
    ;;
  run)
    section "running $IMAGE (http://localhost:$PORT)"
    if ! "$ENGINE" image inspect "$IMAGE" >/dev/null 2>&1; then
      fail "image '$IMAGE' not found — run 'scripts/docs-container.sh build' (or 'just docs-container-build') first"
      exit 1
    fi
    echo "  Serving on http://localhost:$PORT — Ctrl-C to stop."
    "$ENGINE" run --rm -p "${PORT}:8000" "$IMAGE"
    ;;
  *)
    fail "usage: $0 <build|run>"
    exit 64
    ;;
esac
