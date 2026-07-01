#!/usr/bin/env bash
# Validate that every tracked JSON / YAML / TOML file parses.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "structured data (json/yaml/toml parse)"

if ! have python3; then skip "python3 not found"; exit 0; fi
tracked '*.json' '*.yaml' '*.yml' '*.toml'
if [[ ${#TRACKED[@]} -eq 0 ]]; then skip "no structured files"; exit 0; fi

python3 - "${TRACKED[@]}" <<'PY'
import sys, json
try:
    import yaml
except Exception:
    yaml = None
try:
    import tomllib
except Exception:
    tomllib = None

bad = 0
for p in sys.argv[1:]:
    try:
        if p.endswith(".json"):
            json.load(open(p, encoding="utf-8"))
        elif p.endswith((".yaml", ".yml")):
            if yaml is None:
                continue
            list(yaml.safe_load_all(open(p, encoding="utf-8")))
        elif p.endswith(".toml"):
            if tomllib is None:
                continue
            tomllib.load(open(p, "rb"))
    except Exception as e:
        print(f"  parse error: {p}: {e}")
        bad += 1
sys.exit(1 if bad else 0)
PY
rc=$?
if [[ $rc -eq 0 ]]; then ok "all structured files parse"; else fail "structured parse errors"; fi
exit $rc
