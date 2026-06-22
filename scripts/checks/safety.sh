#!/usr/bin/env bash
# `// SAFETY:` adjacency gate (DN-21 §5 F-3; ADR-014 §8.1): every Rust `unsafe` block / `unsafe fn` /
# `unsafe impl` / `unsafe trait` under crates/ must carry a `// SAFETY:` justification within a small
# window directly above it. ADR-014 deferred this grep as future work; M-681 lands it. Pure
# shell + `git grep` (no toolchain, no new dependency), so it always runs — it never skips on a
# missing tool. A site with no adjacent `// SAFETY:` fails loudly (G2 — never a silent pass).
#
# Honesty (VR-5): this is an `Empirical`/`Declared` regex heuristic, not a parser — the Rust source is
# ground truth. It excludes line/doc comments (so prose mentioning "unsafe" is not a hit) and the
# `unsafe_code` lint attribute (`\bunsafe\b` skips it — no whitespace before `_code`). A string literal
# containing `unsafe {` would be a false positive; none exist today (DN-21 §2 inventory = exactly 6
# sites, all real, all in crates/mycelium-mlir).
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "unsafe // SAFETY: adjacency (ADR-014 §8.1 / DN-21)"

# Lines scanned upward from an `unsafe` site for its justification. Large enough to clear a multi-line
# `// SAFETY:` comment plus an interleaved `#[cfg_attr(not(debug_assertions), allow(unsafe_code))]` and
# a `debug_assert!(...)` precondition line (the established mycelium-mlir pattern), small enough to
# stay genuinely "adjacent".
WINDOW=12

# Candidate sites under crates/: `unsafe` immediately followed by `{` / `fn` / `impl` / `trait`,
# excluding lines whose content begins with `//` (line/doc comments).
mapfile -t hits < <(
  git grep -nE '(^|[^[:alnum:]_])unsafe[[:space:]]+(\{|fn|impl|trait)' -- ':(glob)crates/**/*.rs' \
    | grep -vE ':[0-9]+:[[:space:]]*//' || true
)

if [[ ${#hits[@]} -eq 0 ]]; then
  ok "no Rust \`unsafe\` sites under crates/ (nothing to justify)"
  exit 0
fi

missing=()
for h in "${hits[@]}"; do
  file="${h%%:*}"
  rest="${h#*:}"
  line="${rest%%:*}"
  if (( line <= 1 )); then
    missing+=("$file:$line"); continue
  fi
  start=$(( line > WINDOW ? line - WINDOW : 1 ))
  if ! sed -n "${start},$((line - 1))p" "$file" | grep -q 'SAFETY:'; then
    missing+=("$file:$line")
  fi
done

if [[ ${#missing[@]} -eq 0 ]]; then
  ok "${#hits[@]} \`unsafe\` site(s) each carry an adjacent // SAFETY: (ADR-014)"
  exit 0
else
  fail "${#missing[@]} \`unsafe\` site(s) lack a // SAFETY: within ${WINDOW} lines above (ADR-014 §8.1):"
  printf '        %s\n' "${missing[@]}"
  exit 1
fi
