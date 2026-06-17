#!/usr/bin/env bash
# Mycelium commit-signing setup — Linux/macOS entry point (thin wrapper over the portable engine).
#
# No args  -> read-only SANITY CHECK (is signing installed + configured?).
# --setup  -> configure signing (prompts name/email/comment; reuses a key, generates only if absent
#             or when --new-key forces a rotation). Idempotent + nondestructive + never-silent.
#
# All flags are forwarded verbatim to git-signing-sync.py. See RECONCILE.md for the contract.
#   bash tools/github/git-signing-setup.sh                 # sanity check
#   bash tools/github/git-signing-setup.sh --setup         # configure
#   bash tools/github/git-signing-setup.sh --setup --new-key --upload
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PY=""
for cand in python3 python; do
  if command -v "$cand" >/dev/null 2>&1; then PY="$cand"; break; fi
done
[[ -n "$PY" ]] || { echo "ERROR: no python3/python on PATH" >&2; exit 1; }

exec "$PY" "$HERE/git-signing-sync.py" "$@"
