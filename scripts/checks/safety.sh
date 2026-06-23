#!/usr/bin/env bash
# The FFI / unsafe-floor safety gate — two audits (RFC-0028 §4.7; ADR-014 §8.1; DN-21 §5 F-3):
#
#   1. **Rust `// SAFETY:` adjacency** (M-681): every Rust `unsafe` block / `unsafe fn` / `unsafe impl`
#      / `unsafe trait` under crates/ carries a `// SAFETY:` justification within a small window above.
#   2. **Mycelium `wild`-site audit** (M-724; RFC-0028 §4.7): every `wild { … }` block in a *shippable*
#      `.myc` nodule is inside a `@std-sys` nodule, inside a fn declaring `!{ffi}`, and carries a
#      `// SAFETY:` comment.
#
# Pure shell + `git grep` (no toolchain, no new dependency), so it always runs — it never skips on a
# missing tool. A site failing either audit fails loudly (G2 — never a silent pass).
#
# Honesty (VR-5): both audits are `Empirical`/`Declared` regex heuristics, not parsers — the Rust
# source and the L1 checker (`crates/mycelium-l1`) are ground truth. The Rust scan excludes line/doc
# comments and the `unsafe_code` lint attribute. The Mycelium scan **excludes the grammar-conformance
# corpus** (`docs/spec/grammar/conformance/`): those are *parser* fixtures that deliberately exercise
# parse-legal-but-check-illegal forms (e.g. a `wild` block outside `@std-sys`), validated by the
# checker test-suite — not shippable FFI-floor code. The entire workspace `unsafe` surface is confined
# to `crates/mycelium-mlir/src/jit.rs` (DN-21 §6).
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1

status=0

# ─────────────────────────────────────────────────────────────────────────────────────────────────
# Audit 1 — Rust `unsafe` // SAFETY: adjacency (ADR-014 §8.1 / DN-21; M-681).
# ─────────────────────────────────────────────────────────────────────────────────────────────────
section "Rust unsafe // SAFETY: adjacency (ADR-014 §8.1 / DN-21)"

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
else
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
    ok "${#hits[@]} Rust \`unsafe\` site(s) each carry an adjacent // SAFETY: (ADR-014)"
  else
    fail "${#missing[@]} Rust \`unsafe\` site(s) lack a // SAFETY: within ${WINDOW} lines above (ADR-014 §8.1):"
    printf '        %s\n' "${missing[@]}"
    status=1
  fi
fi

# ─────────────────────────────────────────────────────────────────────────────────────────────────
# Audit 2 — Mycelium `wild`-site audit (RFC-0028 §4.7; M-724).
# Every `wild { … }` block in a shippable `.myc` nodule must be (a) in a `@std-sys` nodule, (b) inside
# a fn declaring `!{ffi}`, and (c) carry a `// SAFETY:` comment. A site failing any of these is the
# gate (not a lint — G2). The grammar-conformance corpus is excluded (parser fixtures; see header).
# ─────────────────────────────────────────────────────────────────────────────────────────────────
section "Mycelium wild-site audit (@std-sys + !{ffi} + // SAFETY:; RFC-0028 §4.7)"

# `.myc` files that contain a `wild` *block* keyword (`wild` followed by `{`, not the `wildcard`/`_`
# pattern or the word in prose). Exclude the grammar-conformance corpus. Same exit-code discipline as
# above: >=2 is a real error.
wild_raw=$(git grep -lE '(^|[^[:alnum:]_])wild[[:space:]]*\{' \
  -- ':(glob)**/*.myc' ':(exclude,glob)docs/spec/grammar/conformance/**') || wgrep_rc=$?
wgrep_rc=${wgrep_rc:-0}
if (( wgrep_rc >= 2 )); then
  fail "git grep failed (exit ${wgrep_rc}) while scanning for \`wild\` sites — cannot audit"
  exit 1
fi

wild_files=()
if [[ -n "$wild_raw" ]]; then
  mapfile -t wild_files < <(printf '%s\n' "$wild_raw")
fi

if [[ ${#wild_files[@]} -eq 0 ]]; then
  ok "no shippable .myc \`wild\` sites yet (forward-looking gate; conformance fixtures excluded)"
else
  myc_missing=()
  sites=0
  for f in "${wild_files[@]}"; do
    # (a) the nodule header must carry @std-sys; (b) some fn must declare !{ffi}; (c) a // SAFETY:
    # comment must be present. Per-file granularity (a regex heuristic — the checker is ground truth).
    has_std_sys=0; has_ffi=0; has_safety=0
    grep -qE 'nodule[[:space:]].*@std-sys' "$f" && has_std_sys=1
    grep -qE '!\{[^}]*\bffi\b[^}]*\}' "$f" && has_ffi=1
    grep -qE '//[[:space:]]*SAFETY:' "$f" && has_safety=1
    # Count the wild block sites in this file (for the summary).
    n=$(grep -cE '(^|[^[:alnum:]_])wild[[:space:]]*\{' "$f")
    sites=$(( sites + n ))
    reasons=()
    (( has_std_sys )) || reasons+=("no @std-sys nodule header")
    (( has_ffi ))     || reasons+=("no fn declares !{ffi}")
    (( has_safety ))  || reasons+=("no // SAFETY: comment")
    if (( ${#reasons[@]} > 0 )); then
      myc_missing+=("$f: $(IFS='; '; echo "${reasons[*]}")")
    fi
  done
  if [[ ${#myc_missing[@]} -eq 0 ]]; then
    ok "${sites} shippable .myc \`wild\` site(s) across ${#wild_files[@]} file(s): each in @std-sys + !{ffi} + // SAFETY: (RFC-0028 §4.7)"
  else
    fail "${#myc_missing[@]} .myc file(s) with a \`wild\` block fail the FFI-floor audit (RFC-0028 §4.7):"
    printf '        %s\n' "${myc_missing[@]}"
    status=1
  fi
fi

exit "$status"
