#!/usr/bin/env bash
# Validate JSON Schemas (draft 2020-12) and any example instances.
# Convention (per M-010): docs/spec/schemas/<name>.schema.json with examples under
#   docs/spec/schemas/examples/<name>/valid/*.json   (must validate)
#   docs/spec/schemas/examples/<name>/invalid/*.json (must NOT validate)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "json-schema"

dir="docs/spec/schemas"
if [[ ! -d "$dir" ]]; then skip "no $dir yet (added by M-010)"; exit 0; fi
if ! have check-jsonschema; then skip "check-jsonschema not found — run \`just setup\`"; exit 0; fi

shopt -s nullglob
schemas=("$dir"/*.schema.json)
if [[ ${#schemas[@]} -eq 0 ]]; then skip "no *.schema.json in $dir"; exit 0; fi

rc=0
check-jsonschema --check-metaschema "${schemas[@]}" || rc=1
for s in "${schemas[@]}"; do
  name="$(basename "$s" .schema.json)"
  for v in "$dir/examples/$name/valid"/*.json; do
    check-jsonschema --schemafile "$s" "$v" || rc=1
  done
  for iv in "$dir/examples/$name/invalid"/*.json; do
    if check-jsonschema --schemafile "$s" "$iv" >/dev/null 2>&1; then
      fail "invalid example unexpectedly validated: $iv"; rc=1
    fi
  done
done
if [[ $rc -eq 0 ]]; then ok "schemas + examples valid"; else fail "schema validation errors"; fi
exit $rc
