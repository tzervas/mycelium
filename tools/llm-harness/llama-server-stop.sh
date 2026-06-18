#!/usr/bin/env bash
# Stop running llama.cpp servers — e.g. an orphan left by a manual `llama-server … &`
# (the one that lost the port race: "couldn't bind HTTP server socket … port 8080").
#
# Usage:
#   llama-server-stop.sh            # stop ALL llama-server processes
#   llama-server-stop.sh 8080       # stop only the server on --port 8080
#
# Matches the process NAME exactly (`llama-server`) — NOT any cmdline containing the
# string, so it never targets this script. SIGTERM, brief wait, then SIGKILL survivors.
set -uo pipefail

PORT="${1:-}"
self_pid=$$

# Candidate PIDs whose executable name is exactly `llama-server`.
cands=()
if command -v pgrep >/dev/null 2>&1; then
  while IFS= read -r pid; do [ -n "$pid" ] && cands+=("$pid"); done < <(pgrep -x llama-server 2>/dev/null || true)
else
  for d in /proc/[0-9]*; do
    a0=$(tr '\0' '\n' < "$d/cmdline" 2>/dev/null | head -n1) || continue
    [ "$(basename -- "$a0" 2>/dev/null)" = "llama-server" ] || continue
    cands+=("${d#/proc/}")
  done
fi

# Optional --port filter.
pids=()
for pid in "${cands[@]}"; do
  [ "$pid" = "$self_pid" ] && continue
  if [ -n "$PORT" ]; then
    cl=$(tr '\0' ' ' < "/proc/$pid/cmdline" 2>/dev/null) || continue
    [[ "$cl" == *"--port $PORT"* || "$cl" == *"--port=$PORT"* ]] || continue
  fi
  pids+=("$pid")
done

if [ "${#pids[@]}" -eq 0 ]; then
  echo "No llama-server processes found${PORT:+ on port $PORT}."
  exit 0
fi

echo "Stopping llama-server: ${pids[*]}"
kill "${pids[@]}" 2>/dev/null || true
sleep 2
for pid in "${pids[@]}"; do
  if kill -0 "$pid" 2>/dev/null; then
    kill -9 "$pid" 2>/dev/null && echo "  SIGKILL $pid (did not exit on SIGTERM)"
  fi
done
echo "Done."
