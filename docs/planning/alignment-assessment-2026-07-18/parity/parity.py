#!/usr/bin/env python3
"""Content-parity engine: classify every component-repo file against the
monorepo archive tree aad96b7a (blob-SHA identity), both directions.

Classes:
  IDENT_EXPECTED   blob identical at the mapped baseline path (MODE_CHANGED noted)
  IDENT_ELSEWHERE  blob exists in baseline but at other path(s)
  MODIFIED         mapped baseline path exists, blob differs (WS_ONLY vs SUBSTANTIVE)
  NEW_SCAFFOLD     not in baseline; on the decomposition scaffolding allowlist
  NEW_UNEXPLAINED  not in baseline anywhere; not scaffolding  (superset-violation class)
Reverse:
  MISSING          baseline file under the repo's mapped prefix absent from the component
Read-only: only reads git object stores; writes TSVs to the evidence dir.
"""
import subprocess, os, sys, collections, tempfile

MONO = "/home/user/mycelium"
BASE = "/home/user"
ARCHIVE = "aad96b7a425710db5e91094d4fc2ca21a129e41a"
EV = "/tmp/claude-0/-home-user/990ca46f-60f8-5690-b67a-f6a068438f1d/scratchpad/evidence/parity"
os.makedirs(EV, exist_ok=True)

CONTAINERS = {"mycelium-core", "mycelium-value", "mycelium-runtime", "mycelium-codegen"}
SPECIAL = {"mycelium-lang", "mycelium-cli-myc"}
SCAFFOLD = {"README.md", "LICENSE", ".github/workflows/ci.yml", ".gitignore", "components.lock"}

def ls_tree(repo, ref):
    out = subprocess.run(["git", "-C", repo, "ls-tree", "-r",
                          "--format=%(objectmode)\t%(objectname)\t%(path)", ref],
                         capture_output=True, text=True, check=True).stdout
    m = {}
    for line in out.splitlines():
        mode, blob, path = line.split("\t", 2)
        m[path] = (mode, blob)
    return m

def cat_blob(repo, blob):
    return subprocess.run(["git", "-C", repo, "cat-file", "-p", blob],
                          capture_output=True, check=True).stdout

def ws_only_diff(a, b):
    na = b"".join(a.split())
    nb = b"".join(b.split())
    return na == nb

base = ls_tree(MONO, ARCHIVE)
rev = collections.defaultdict(list)
for p, (mode, blob) in base.items():
    rev[blob].append(p)

repos = sorted(d for d in os.listdir(BASE)
               if d.startswith("mycelium-") and os.path.isdir(os.path.join(BASE, d, ".git")))

def map_expected(repo, path):
    """component path -> candidate baseline path (or None if no mapping)."""
    if repo in CONTAINERS:
        return path                      # crates/<crate>/... identity
    if repo == "mycelium-lang":
        return None
    if repo == "mycelium-cli-myc":
        return path                      # lib/..., experiments/... identity candidate
    return f"crates/{repo}/{path}"       # hoisted single-crate

def source_prefixes(repo):
    """baseline prefixes owned by this repo, for the reverse MISSING check."""
    if repo in CONTAINERS:
        crates = sorted(os.listdir(os.path.join(BASE, repo, "crates")))
        return [f"crates/{c}/" for c in crates]
    if repo in SPECIAL:
        return []
    return [f"crates/{repo}/"]

summary_rows = []
global_counts = collections.Counter()
mod_details = []
new_unexplained = []

for repo in repos:
    rp = os.path.join(BASE, repo)
    comp = ls_tree(rp, "HEAD")
    rows, counts = [], collections.Counter()
    for path, (mode, blob) in sorted(comp.items()):
        exp = map_expected(repo, path)
        klass, note, bpath = None, "", ""
        if blob in rev:
            if exp and exp in base and base[exp][1] == blob:
                bmode = base[exp][0]
                if bmode != mode:
                    klass, note = "IDENT_EXPECTED", f"MODE_CHANGED {bmode}->{mode}"
                else:
                    klass = "IDENT_EXPECTED"
                bpath = exp
            else:
                klass = "IDENT_ELSEWHERE"
                bpath = ";".join(rev[blob][:3])
        else:
            if exp and exp in base:
                klass = "MODIFIED"
                bpath = exp
                a = cat_blob(MONO, base[exp][1])
                b = cat_blob(rp, blob)
                note = "WS_ONLY" if ws_only_diff(a, b) else "SUBSTANTIVE"
                mod_details.append((repo, path, exp, base[exp][1], blob, note))
            elif path in SCAFFOLD:
                klass = "NEW_SCAFFOLD"
            else:
                klass = "NEW_UNEXPLAINED"
                new_unexplained.append((repo, path, blob))
        counts[klass] += 1
        rows.append(f"{path}\t{klass}\t{bpath}\t{note}")
    # reverse: MISSING
    missing = []
    for pref in source_prefixes(repo):
        for bp in base:
            if bp.startswith(pref):
                mapped = bp if repo in CONTAINERS else bp[len(pref):]
                if repo not in CONTAINERS:
                    # hoisted: baseline crates/<repo>/X -> component X
                    pass
                if mapped not in comp:
                    missing.append(bp)
                    counts["MISSING"] += 1
                    rows.append(f"{bp}\tMISSING\t{bp}\t")
    with open(os.path.join(EV, f"{repo}.tsv"), "w") as f:
        f.write("component_path\tclass\tbaseline_path\tnote\n")
        f.write("\n".join(rows) + "\n")
    global_counts.update(counts)
    summary_rows.append((repo, len(comp), counts))

# global asset-routing reverse check
assets = {"lib/": [], "editors/": [], "fuzz/": [], ".devcontainer/": []}
comp_blob_union = set()
for repo in repos:
    for path, (mode, blob) in ls_tree(os.path.join(BASE, repo), "HEAD").items():
        comp_blob_union.add(blob)
for bp, (mode, blob) in base.items():
    for pref in assets:
        if bp.startswith(pref):
            assets[pref].append((bp, blob in comp_blob_union))

with open(os.path.join(EV, "SUMMARY.tsv"), "w") as f:
    f.write("repo\tfiles\tIDENT_EXPECTED\tIDENT_ELSEWHERE\tMODIFIED\tNEW_SCAFFOLD\tNEW_UNEXPLAINED\tMISSING\n")
    for repo, n, c in summary_rows:
        f.write(f"{repo}\t{n}\t{c['IDENT_EXPECTED']}\t{c['IDENT_ELSEWHERE']}\t{c['MODIFIED']}\t{c['NEW_SCAFFOLD']}\t{c['NEW_UNEXPLAINED']}\t{c['MISSING']}\n")

print("=== GLOBAL ===")
for k in ("IDENT_EXPECTED", "IDENT_ELSEWHERE", "MODIFIED", "NEW_SCAFFOLD", "NEW_UNEXPLAINED", "MISSING"):
    print(f"{k}: {global_counts[k]}")
print("\n=== MODIFIED detail ===")
for r in mod_details:
    print("\t".join(r[:3]) + f"\t{r[5]}")
print("\n=== NEW_UNEXPLAINED (superset-violation candidates) ===")
for repo, path, blob in new_unexplained:
    print(f"{repo}\t{path}\t{blob}")
print("\n=== ASSET ROUTING (baseline files under prefix -> landed in any component?) ===")
for pref, entries in assets.items():
    landed = sum(1 for _, ok in entries if ok)
    print(f"{pref}: {len(entries)} baseline files, {landed} present in some component repo")
print("\nper-repo summary: evidence/parity/SUMMARY.tsv")
