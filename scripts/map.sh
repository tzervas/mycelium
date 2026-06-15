#!/usr/bin/env bash
# Advisory code map (developer aid; NOT part of `just check` — no pass/fail).
# Enumerates/traces the Rust workspace: crate-to-crate dependency graph (routing), per-crate
# module/item structure with internal `use` edges, and rustdoc including private items.
# Every tool is optional — missing ones skip gracefully. Artifacts land under target/map/.
source "${BASH_SOURCE%/*}/lib.sh"
cd "$REPO_ROOT" || exit 1
section "code map"

if ! have cargo; then skip "no cargo — nothing to map"; exit 0; fi
out="target/map"
mkdir -p "$out"

# 1) Crate-to-crate dependency graph (the "routing" between workspace crates).
if cargo depgraph --help >/dev/null 2>&1; then
  if have dot; then
    if cargo depgraph --workspace-only 2>/dev/null | dot -Tsvg >"$out/crates.svg"; then
      ok "crate dependency graph → $out/crates.svg"
    else
      fail "cargo depgraph | dot failed"
    fi
  elif cargo depgraph --workspace-only >"$out/crates.dot" 2>/dev/null; then
    ok "crate dependency graph (Graphviz source) → $out/crates.dot (install graphviz to render)"
  else
    fail "cargo depgraph failed"
  fi
elif cargo tree --workspace --edges normal >"$out/crates-tree.txt" 2>/dev/null; then
  skip "cargo-depgraph absent — wrote \`cargo tree\` → $out/crates-tree.txt"
else
  skip "cargo-depgraph absent and \`cargo tree\` failed"
fi

# 2) Per-crate module + item structure with internal use-edges.
if cargo modules --help >/dev/null 2>&1; then
  for d in crates/*/ xtask/; do
    [[ -f "${d}Cargo.toml" ]] || continue
    pkg="$(basename "$d")"
    cargo modules structure --package "$pkg" >"$out/$pkg.modules.txt" 2>/dev/null || true
  done
  ok "module structure → $out/<crate>.modules.txt"
else
  skip "cargo-modules absent (run \`just setup\`)"
fi

# 3) Rustdoc including private items — the browsable item graph (no new deps).
if cargo doc --workspace --no-deps --document-private-items >/dev/null 2>&1; then
  ok "rustdoc incl. private items → target/doc/index.html"
else
  skip "cargo doc failed"
fi

echo
ok "map done — see $out/ and target/doc/. (Function-level call graphs are partial in Rust;"
ok "use rust-analyzer's call hierarchy interactively, or cargo-call-stack for hot paths.)"
