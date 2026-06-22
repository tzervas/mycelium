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
# containing `unsafe {` would be a false positive; none exist today. The entire workspace unsafe surface
# is confined to `crates/mycelium-mlir/src/jit.rs` (DN-21 §6 holds the current per-site inventory).
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "unsafe // SAFETY: adjacency (ADR-014 §8.1 / DN-21)"

# Lines scanned upward from an `unsafe` site for its justification. Large enough to clear a multi-line
# `// SAFETY:` comment plus an interleaved `#[cfg_attr(not(debug_assertions), allow(unsafe_code))]` and
# a `debug_assert!(...)` precondition line (the established mycelium-mlir pattern), small enough to
# stay genuinely "adjacent".
WINDOW=12

# Candidate sites under crates/: `unsafe` immediately followed by `{` / `fn` / `impl` / `trait`.
# `git grep` exits 0 when it matches and 1 when it finds nothing — both are normal here — but >=2 means
# a real failure (bad pathspec, repo error). Distinguish them explicitly so a genuine error fails loudly
# (G2) instead of being swallowed into an empty hit list and a false `ok` (the `|| true` trap).
raw=$(git grep -nE '(^|[^[:alnum:]_])unsafe[[:space:]]+(\{|fn|impl|trait)' -- ':(glob)crates/**/*.rs') \
  || grep_rc=$?
grep_rc=${grep_rc:-0}
if (( grep_rc >= 2 )); then
  fail "git grep failed (exit ${grep_rc}) while scanning for \`unsafe\` sites — cannot audit"
  exit 1
fi
# Drop lines whose content begins with `//` (line/doc comments); a no-match here is a legitimate empty
# result, not an error.
hits=()
if [[ -n "$raw" ]]; then
  mapfile -t hits < <(printf '%s\n' "$raw" | grep -vE ':[0-9]+:[[:space:]]*//' || true)
fi

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
  # Require a real justification in the window above, in one of the two ADR-014 §8.1 forms — not a bare
  # `SAFETY:` substring (so a `SAFETY:` inside a string literal or a `//!` doc comment is not mistaken
  # for one):
  #   * `// SAFETY:` — the line-comment form for an `unsafe { … }` block (`//[[:space:]]*SAFETY:`
  #     excludes `//!`/`///` doc comments, whose 3rd char is `!`/`/`, never a space or `S`);
  #   * `# Safety`   — the rustdoc doc-section form for an `unsafe fn`/`impl`/`trait` *declaration* (the
  #     contract clippy's `missing_safety_doc` requires; e.g. `/// # Safety`).
  if ! sed -n "${start},$((line - 1))p" "$file" | grep -qE '//[[:space:]]*SAFETY:|#[[:space:]]*Safety'; then
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
