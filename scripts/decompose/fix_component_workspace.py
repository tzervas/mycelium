#!/usr/bin/env python3
"""Course-correction Phase B: make a component repo a standalone, buildable Cargo workspace.

Subcommands:
  plan                 print the repo-level dependency waves (topological, select→runtime applied)
  apply <repo> [...]   transform the named repo working trees in place (idempotent)

Transformations per repo (steer handoff §6.2; DN-143 §4; program ledger Phase B):
  1. `[workspace]` root synthesized from the monorepo template (train version 0.464.0).
  2. Cross-repo `path` deps → `git` deps pinned by rev from the pin table (intra-repo path deps
     kept). The pin table (pins.json) maps repo → fixed main rev; a missing pin is a hard error
     (never a silent fallback — G2).
  3. Scaffolding: rust-toolchain.toml, deny.toml/about.toml/osv-scanner.toml replicas, CI v2
     (fmt --check + clippy -D warnings + test), CROSS-REF.md, docs/spec slice for std repos.
Reads the monorepo checkout at MONO for templates. Writes only inside the target repo dirs.
"""
import json, os, re, shutil, subprocess, sys, collections

BASE = "/home/user"
MONO = f"{BASE}/mycelium"
TRAIN = "0.464.0"
ORG = "https://github.com/tzervas"

CONTAINERS = {
    "mycelium-core": ["mycelium-core", "mycelium-stack", "mycelium-workstack"],
    "mycelium-value": ["mycelium-dense", "mycelium-numerics", "mycelium-vsa"],
    "mycelium-runtime": ["mycelium-sched", "mycelium-rt-abi", "mycelium-interp", "mycelium-cert",
                          "mycelium-diag", "mycelium-select",
                          "mycelium-vsa-decode"],  # select + vsa-decode: DN-143 §4 (approved)
    "mycelium-codegen": ["mycelium-mir-passes", "mycelium-mlir"],
    # l1 converted to container layout in Phase B W5 so its CARGO_MANIFEST_DIR/../../lib and
    # ../../docs/spec/grammar fixture paths resolve identically to the monorepo (zero divergence).
    "mycelium-l1": ["mycelium-l1"],
}
SKIP = {"mycelium-lang", "mycelium-cli-myc", "mycelium"}

def crate2repo():
    m = {}
    for r, cs in CONTAINERS.items():
        for c in cs:
            m[c] = r
    for d in os.listdir(BASE):
        p = os.path.join(BASE, d)
        if d.startswith("mycelium-") and os.path.isdir(os.path.join(p, ".git")) and d not in SKIP:
            if d not in CONTAINERS:
                m.setdefault(d, d)
    return m

C2R = crate2repo()
REPOS = sorted(set(C2R.values()))

# Matches any inline-table mycelium dep that carries a `path` key (optionally with `version`
# and/or other keys — e.g. bench's `{ path = "../x", version = "0.463.1" }`). The rewrite drops
# the version key (git pins don't need it; intra-repo path deps must not pin the old train).
DEP_RE = re.compile(r'^(mycelium-[a-z0-9-]+)(\s*=\s*)\{[^}]*\bpath\s*=\s*"[^"]*"[^}]*\}\s*(#.*)?$')
# Already-pinned git dep line (re-pin support: pins are FROZEN per train — a drifted rev is
# rewritten back to the pin-table rev so the whole train shares one rev per repo; two revs of the
# same git URL would otherwise split into two package identities and E0308 across the graph).
GIT_RE = re.compile(r'^(mycelium-[a-z0-9-]+)\s*=\s*\{\s*git\s*=\s*"[^"]*/([a-z0-9-]+)"\s*,\s*rev\s*=\s*"([0-9a-f]{40})"\s*\}\s*$')

def manifests(repo):
    out = []
    root = os.path.join(BASE, repo)
    for dirpath, dirs, files in os.walk(root):
        if ".git" in dirs:
            dirs.remove(".git")
        if "Cargo.toml" in files:
            out.append(os.path.join(dirpath, "Cargo.toml"))
    return out

def repo_deps(repo):
    deps = set()
    for mf in manifests(repo):
        for line in open(mf):
            m = DEP_RE.match(line.strip())
            if m:
                tr = C2R.get(m.group(1))
                if tr and tr != repo:
                    deps.add(tr)
    return deps

def waves():
    deps = {r: repo_deps(r) for r in REPOS}
    done, order = set(), []
    while len(done) < len(REPOS):
        ready = sorted(r for r in REPOS if r not in done and deps[r] <= done)
        if not ready:
            missing = {r: sorted(deps[r] - done) for r in REPOS if r not in done}
            sys.exit(f"CYCLE / unresolvable: {missing}")
        order.append(ready)
        done |= set(ready)
    return order

WS_BLOCK = """
# --- standalone workspace root (course-correction Phase B, 2026-07-18; steer handoff §6.2) ---
{workspace_table}

[workspace.package]
version = "{train}"
edition = "2021"
rust-version = "1.96.1"
license = "MIT"
repository = "{org}/{repo}"

[workspace.dependencies]
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
blake3 = "1"

[workspace.lints.rust]
# ADR-014: unsafe permitted-but-documented; the check is clippy::undocumented_unsafe_blocks.
unsafe_code = "allow"

[workspace.lints.clippy]
all = {{ level = "warn", priority = -1 }}
undocumented_unsafe_blocks = "warn"

# G2 never-silent arithmetic: release builds panic on overflow (monorepo parity).
[profile.release]
overflow-checks = true
"""

CI_V2 = """name: ci
on:
  workflow_dispatch:
  push:
    branches: [main]
  pull_request:
    branches: [main]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.96.1
        with:
          components: rustfmt, clippy
      - run: cargo fmt --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test
"""

OWNING_DOCS = {  # static owning-docs map (bounded; monorepo Doc-Index is the full record)
    "mycelium-core": "RFC-0001 · RFC-0033 · ADR-003 (value model, content addressing, guarantee lattice)",
    "mycelium-value": "RFC-0033 · ADR-010/011 (dense/numerics/VSA kernels)",
    "mycelium-runtime": "RFC-0002 · RFC-0034 · RFC-0013 (certs, modes, diagnostics, interpreter)",
    "mycelium-codegen": "ADR-007 (MLIR→LLVM AOT path)",
    "mycelium-l1": "RFC-0006 · RFC-0012 (surface language, elaboration)",
    "mycelium-transpile": "DN-34 · DN-124 · DN-135 (gap-profiling transpiler)",
    "mycelium-bench": "E-BENCH (backend differential bench harness)",
}

def std_slice(repo):
    if repo.startswith("mycelium-std-"):
        mod = repo[len("mycelium-std-"):]
        # sys-host shares the sys slice
        cand = {"sys-host": "sys"}.get(mod, mod)
        p = f"{MONO}/docs/spec/stdlib/{cand}.md"
        return p if os.path.exists(p) else None
    return None

def apply(repo, pins):
    rp = os.path.join(BASE, repo)
    changed = []
    is_container = repo in CONTAINERS
    root_manifest = os.path.join(rp, "Cargo.toml")
    ws_table = ('[workspace]\nresolver = "2"\nmembers = [\n' +
                "".join(f'    "crates/{c}",\n' for c in CONTAINERS[repo]) + "]") if is_container \
               else "[workspace]"
    block = WS_BLOCK.format(workspace_table=ws_table, train=TRAIN, org=ORG, repo=repo)
    if is_container:
        if not os.path.exists(root_manifest):
            open(root_manifest, "w").write(
                f"# {repo} — standalone workspace (extracted from tzervas/mycelium; see CROSS-REF.md)\n"
                + block.lstrip())
            changed.append("Cargo.toml (new workspace root)")
    else:
        txt = open(root_manifest).read()
        if "[workspace]" not in txt:
            open(root_manifest, "a").write(block)
            changed.append("Cargo.toml (+workspace block)")
    # dep rewrite
    edges = []
    for mf in manifests(repo):
        lines = open(mf).readlines()
        out, dirty = [], False
        for line in lines:
            m = DEP_RE.match(line.strip())
            if m:
                dep, target = m.group(1), C2R.get(m.group(1))
                if target and target != repo:
                    if target not in pins:
                        sys.exit(f"{repo}: no pin for dependency repo {target} (dep {dep}) — "
                                 f"fix {target} first (topological order; G2 hard error)")
                    out.append(f'{dep} = {{ git = "{ORG}/{target}", rev = "{pins[target]}" }}\n')
                    dirty = True
                    edges.append((dep, target, pins[target]))
                    continue
                if target == repo and "version" in line:
                    # intra-repo path dep: strip the stale train version key
                    out.append(f'{dep} = {{ path = "../{dep}" }}\n')
                    dirty = True
                    continue
            g = GIT_RE.match(line.strip())
            if g:
                dep, target, rev = g.group(1), g.group(2), g.group(3)
                want = pins.get(target)
                if want and want != rev:
                    out.append(f'{dep} = {{ git = "{ORG}/{target}", rev = "{want}" }}\n')
                    dirty = True
                    edges.append((dep, target, want))
                    continue
                if want:
                    edges.append((dep, target, rev))
            out.append(line)
        if dirty:
            open(mf, "w").writelines(out)
            changed.append(os.path.relpath(mf, rp))
    # scaffolding
    tc = os.path.join(rp, "rust-toolchain.toml")
    if not os.path.exists(tc):
        shutil.copy(f"{MONO}/rust-toolchain.toml", tc)
        changed.append("rust-toolchain.toml")
    for f in ("about.toml", "osv-scanner.toml"):
        d = os.path.join(rp, f)
        if not os.path.exists(d):
            shutil.copy(f"{MONO}/{f}", d)
            changed.append(f)
    dt = os.path.join(rp, "deny.toml")
    if not os.path.exists(dt):
        src = open(f"{MONO}/deny.toml").read()
        open(dt, "w").write("# Replicated from tzervas/mycelium (Phase B). The monorepo's\n"
                            "# `cargo xtask deps` acyclicity gate is monorepo-only; cross-repo\n"
                            "# layering is enforced by the pinned dep graph (CROSS-REF.md).\n" + src)
        changed.append("deny.toml")
    ci = os.path.join(rp, ".github", "workflows", "ci.yml")
    if open(ci).read() != CI_V2:
        open(ci, "w").write(CI_V2)
        changed.append(".github/workflows/ci.yml (v2)")
    # docs slice
    sl = std_slice(repo)
    if sl:
        dd = os.path.join(rp, "docs", "spec", "stdlib")
        os.makedirs(dd, exist_ok=True)
        dst = os.path.join(dd, os.path.basename(sl))
        if not os.path.exists(dst):
            shutil.copy(sl, dst)
            changed.append(f"docs/spec/stdlib/{os.path.basename(sl)}")
    # CROSS-REF.md
    rows = []
    for dep, target, rev in sorted(set(edges)):
        tree = subprocess.run(["git", "-C", os.path.join(BASE, target), "rev-parse", f"{rev}^{{tree}}"],
                              capture_output=True, text=True)
        th = tree.stdout.strip() if tree.returncode == 0 else "(tree hash: fetch dep rev locally to resolve)"
        rows.append(f"| {dep} | {ORG}/{target} | `{rev}` | tree `{th}` | Rust API of `{dep}` "
                    f"(see monorepo `docs/api-index/INDEX.md#{dep}`) |")
    docs_note = OWNING_DOCS.get(repo)
    if not docs_note and repo.startswith("mycelium-std-"):
        mod = repo[len('mycelium-std-'):]
        docs_note = f"`docs/spec/stdlib/{ {'sys-host':'sys'}.get(mod, mod) }.md` (slice in this repo) · RFC-0016"
    docs_note = docs_note or "see monorepo `docs/Doc-Index.md`"
    cr = (f"# CROSS-REF — {repo}\n\n"
          f"Mycelium-internal dependencies only (steer handoff §6.1; external crates stay in Cargo\n"
          f"metadata). Pinned revs are the fixed (buildable) tips recorded by the Phase-B wave;\n"
          f"content hash = git tree hash of the pinned rev.\n\n"
          f"| Interface consumed | Repo | Pinned rev | Content hash | Notes |\n|---|---|---|---|---|\n"
          + ("\n".join(rows) if rows else "| (none — leaf component) | — | — | — | — |")
          + f"\n\n**Owning docs:** {docs_note}.\n"
          f"**Source provenance:** extracted from `tzervas/mycelium` archive `aad96b7a…`; fixed by\n"
          f"the course-correction Phase B (workspace root, git pins, toolchain + supply-chain\n"
          f"replicas, CI v2). Full program record: monorepo\n"
          f"`docs/planning/course-correction-2026-07-18/PROGRAM.md`.\n")
    crp = os.path.join(rp, "CROSS-REF.md")
    if not os.path.exists(crp) or open(crp).read() != cr:
        open(crp, "w").write(cr)
        changed.append("CROSS-REF.md")
    return changed

if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "plan"
    if cmd == "plan":
        for i, w in enumerate(waves(), 1):
            print(f"W{i}: {' '.join(w)}")
    elif cmd == "apply":
        pins = json.load(open(sys.argv[2]))
        for repo in sys.argv[3:]:
            ch = apply(repo, pins)
            print(f"{repo}: {len(ch)} changes: {', '.join(ch)}")
    else:
        sys.exit(f"unknown subcommand {cmd}")
