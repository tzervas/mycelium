#!/usr/bin/env bash
# First-party MIT-only license audit (M-743; ADR-022 §7 convention 3 "MIT-only licensing";
# ADR-023 §3.4 "License (decided — legal-readiness criterion)").
#
# The project is MIT-licensed only — no Apache-2.0, no dual-license, on any first-party
# artifact (CONTRIBUTING §Licensing; house rule #6). `deny.toml` / `scripts/checks/deny.sh`
# already gate the DEPENDENCY axis (cargo-deny's `[licenses] allow = [MIT, Apache-2.0, BSD-*,
# ISC, Unicode-*]`) — that allow-list is correct and NOT a violation: it governs *third-party*
# deps (ADR-023 §3.4 Q2, resolved: "MIT governs first-party only"). This gate is the first-party
# complement deny.toml does not (and should not) cover: it never inspects a dependency's license,
# only this repo's own manifests/sources.
#
# Four checked items, matching ADR-023 §3.4's enumerated Definition of Done exactly:
#   1. The root `LICENSE` file is the MIT text.
#   2. Every first-party workspace-participating `Cargo.toml`'s `[package].license` (or the
#      inherited `[workspace.package].license`) reads exactly `MIT`.
#   3. Every first-party phylum manifest (`mycelium-proj.toml`)'s `license` field reads `MIT` —
#      test fixtures under `crates/**/tests/fixtures/**` are EXCEPTED (ADR-023 §3.4: they
#      deliberately declare `Apache-2.0` / an invalid SPDX id to prove the parser does not
#      silently inherit/override a locally-declared license — they are inputs, not shipped
#      artifacts).
#   4. Every first-party SHIPPED `.myc` nodule that carries an `@license:` header declares `MIT`
#      — `lib/std/**` + `examples/**` per ADR-023 §3.4. Same fixture exception as (3), plus
#      `docs/examples/**` is excepted for the same reason `scripts/checks/safety.sh` excepts it:
#      illustrative teaching walkthroughs, never compiled/published (not shippable). A `.myc`
#      nodule with NO `@license` header is not itself a violation — ADR-023's own fixtures show
#      license is tracked per-nodule-if-present, and a nodule may rely on its phylum's
#      `mycelium-proj.toml` (checked in (3)) instead of redeclaring; only a PRESENT, non-MIT
#      value is a finding here.
#
# Pure shell + git ls-files/grep (no toolchain dependency), so — like safety.sh — it never
# skip-passes on a missing tool: a first-party non-MIT license is always a hard, visible finding
# (G2, never-silent). This is an `Empirical` line/regex heuristic over TOML/comment headers, not a
# real TOML/L1 parser — `Cargo.toml` / `mycelium-proj.toml` / the `.myc` header themselves are
# ground truth (VR-5: tag the check's own strength honestly).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1

status=0
REQUIRED="MIT"

# Extract the (single-level) TOML table body starting at a `^[<name>]` header, up to the next
# line starting with `[`, or EOF. Heuristic (documented above): a manifest that puts an
# unrelated field AFTER a nested subtable header inside the same logical table would confuse
# this — none of the manifests audited here do (subtables, where present, come last).
toml_table() { # $1=file $2=header (e.g. "package", "workspace.package")
  # Bracket-class escaping (`[[]`/`[.]`/`[]]`) rather than backslash-escaping: a backslash inside
  # an awk dynamic-regex STRING triggers gawk's "escape sequence treated as plain" warning noise
  # even though it is functionally correct — bracket classes match the same literals with none.
  local hdr="^[[]${2//./[.]}[]]\$"
  awk -v hdr="$hdr" '
    $0 ~ hdr { f=1; next }
    /^\[/    { f=0 }
    f        { print }
  ' "$1"
}

# ─────────────────────────────────────────────────────────────────────────────────────────────
# 1. Root LICENSE file.
# ─────────────────────────────────────────────────────────────────────────────────────────────
section "root LICENSE file (ADR-023 §3.4)"
if [[ ! -f LICENSE ]]; then
  fail "no LICENSE file at repo root"
  status=1
elif ! head -1 LICENSE | grep -qE '^MIT License[[:space:]]*$'; then
  fail "LICENSE file's first line is not the MIT license header: $(head -1 LICENSE)"
  status=1
else
  ok "LICENSE is the MIT text"
fi

# ─────────────────────────────────────────────────────────────────────────────────────────────
# 2. Workspace + member Cargo.toml `[package].license` fields.
# ─────────────────────────────────────────────────────────────────────────────────────────────
section "Cargo.toml license fields (ADR-023 §3.4)"

if [[ ! -f Cargo.toml ]]; then
  fail "no root Cargo.toml — cannot audit workspace license"
  status=1
else
  # `|| true` on the trailing pipeline: under `set -o pipefail` a table with no license line makes
  # `grep` exit 1, which — unmasked — would abort the whole script via `set -e` right here, BEFORE
  # the `[[ -z ]]`/mismatch check below ever gets to print the "license is missing" finding (a
  # never-silent violation, G2). Masking lets `ws_license` come back empty and fall through to that
  # check instead of vanishing as a bare, unexplained non-zero exit.
  ws_license=$(toml_table Cargo.toml "workspace.package" \
    | grep -E '^[[:space:]]*license[[:space:]]*=' | head -1 \
    | sed -E 's/^[^=]*=[[:space:]]*"([^"]*)".*/\1/' || true)
  if [[ "$ws_license" != "$REQUIRED" ]]; then
    fail "root Cargo.toml [workspace.package].license is '${ws_license:-<missing>}', expected MIT"
    status=1
  else
    ok "[workspace.package].license = \"MIT\""
  fi

  # Workspace members, parsed from the `members = [ ... ]` array (may span multiple lines).
  mapfile -t members < <(sed -n '/^members[[:space:]]*=[[:space:]]*\[/,/^\]/p' Cargo.toml \
    | grep -oE '"[^"]+"' | tr -d '"')

  # Every tracked Cargo.toml this repo owns, so an addition never silently falls outside scope.
  mapfile -t all_manifests < <(git ls-files -- '*Cargo.toml')

  in_scope=("Cargo.toml")
  for m in "${members[@]}"; do
    in_scope+=("$m/Cargo.toml")
  done

  # Anything tracked but NOT root/a workspace member is out of THIS gate's scope — never drop it
  # silently: name it and why. Currently: fuzz/Cargo.toml, a standalone `[workspace]` cargo-fuzz
  # manifest deliberately excluded from the main workspace (its own header comment) and never
  # published (`publish = false`) — not a shipped first-party artifact in the ADR-023 §3.4 sense.
  excluded=()
  for f in "${all_manifests[@]}"; do
    found=0
    for s in "${in_scope[@]}"; do [[ "$f" == "$s" ]] && { found=1; break; }; done
    (( found == 0 )) && excluded+=("$f")
  done
  if [[ ${#excluded[@]} -gt 0 ]]; then
    printf '  %snote%s  out of scope (standalone/non-published, not a workspace member): %s\n' \
      "$C_YEL" "$C_RST" "${excluded[*]}"
  fi

  bad=()
  for rel in "${in_scope[@]}"; do
    [[ "$rel" == "Cargo.toml" ]] && continue # already checked above via workspace.package
    if [[ ! -f "$rel" ]]; then
      bad+=("$rel: listed as a workspace member but file is missing")
      continue
    fi
    # `|| true`: see the workspace.package fix above — a missing [package].license would otherwise
    # make `grep` exit 1 and abort the script under `set -e` before the `-z` finding below can fire.
    line=$(toml_table "$rel" "package" | grep -E '^[[:space:]]*license(\.workspace)?[[:space:]]*=' | head -1 || true)
    if [[ -z "$line" ]]; then
      bad+=("$rel: [package].license is missing")
    elif printf '%s' "$line" | grep -qE '^[[:space:]]*license\.workspace[[:space:]]*=[[:space:]]*true'; then
      : # inherits workspace.package.license, already validated == MIT above
    else
      lit=$(printf '%s' "$line" | sed -E 's/^[^=]*=[[:space:]]*"([^"]*)".*/\1/')
      if [[ "$lit" != "$REQUIRED" ]]; then
        bad+=("$rel: [package].license = \"${lit}\", expected MIT")
      fi
    fi
  done

  if [[ ${#bad[@]} -eq 0 ]]; then
    ok "${#in_scope[@]} in-scope Cargo.toml manifest(s) (root + ${#members[@]} workspace member(s)) are MIT"
  else
    fail "${#bad[@]} Cargo.toml manifest(s) are not confirmed MIT:"
    printf '        %s\n' "${bad[@]}"
    status=1
  fi
fi

# ─────────────────────────────────────────────────────────────────────────────────────────────
# 3. Phylum manifests (mycelium-proj.toml `license` field).
# ─────────────────────────────────────────────────────────────────────────────────────────────
section "mycelium-proj.toml phylum manifest licenses (ADR-023 §3.4)"

mapfile -t proj_manifests < <(git ls-files -- '*mycelium-proj.toml')
bad=()
skipped=0
for f in "${proj_manifests[@]}"; do
  case "$f" in
    */tests/fixtures/*|tests/fixtures/*)
      skipped=$((skipped + 1))
      continue
      ;;
  esac
  # `|| true`: same fail-safe as the two Cargo.toml sites above — a missing [project].license would
  # otherwise make `grep` exit 1 and abort the script under `set -e` before the finding below fires.
  line=$(toml_table "$f" "project" | grep -E '^[[:space:]]*license[[:space:]]*=' | head -1 || true)
  if [[ -z "$line" ]]; then
    bad+=("$f: [project].license is missing")
    continue
  fi
  lit=$(printf '%s' "$line" | sed -E 's/^[^=]*=[[:space:]]*"([^"]*)".*/\1/')
  if [[ "$lit" != "$REQUIRED" ]]; then
    bad+=("$f: [project].license = \"${lit}\", expected MIT")
  fi
done

if (( skipped > 0 )); then
  printf '  %snote%s  excepted (deliberate ADR-023 §3.4 test fixtures, not shipped): %d file(s)\n' \
    "$C_YEL" "$C_RST" "$skipped"
fi
if [[ ${#proj_manifests[@]} -eq 0 ]]; then
  skip "no mycelium-proj.toml manifests tracked yet"
elif [[ ${#bad[@]} -eq 0 ]]; then
  ok "$(( ${#proj_manifests[@]} - skipped )) shipped phylum manifest(s) are MIT"
else
  fail "${#bad[@]} phylum manifest(s) are not confirmed MIT:"
  printf '        %s\n' "${bad[@]}"
  status=1
fi

# ─────────────────────────────────────────────────────────────────────────────────────────────
# 4. Shipped .myc `@license:` headers.
# ─────────────────────────────────────────────────────────────────────────────────────────────
section "shipped .myc @license headers (ADR-023 §3.4)"

mapfile -t myc_hits < <(git grep -nE '@license:' -- '*.myc' 2>/dev/null || true)
bad=()
excepted=0
present=0
for h in "${myc_hits[@]}"; do
  file="${h%%:*}"
  rest="${h#*:}"
  val_raw="${rest#*:}"
  case "$file" in
    */tests/fixtures/*|tests/fixtures/*|docs/examples/*)
      excepted=$((excepted + 1))
      continue
      ;;
  esac
  present=$((present + 1))
  val=$(printf '%s' "$val_raw" | sed -E 's/^[^:]*@license:[[:space:]]*//' | sed -E 's/[[:space:]]+$//')
  if [[ "$val" != "$REQUIRED" ]]; then
    bad+=("$file: @license: ${val}, expected MIT")
  fi
done

if (( excepted > 0 )); then
  printf '  %snote%s  excepted (test fixtures + non-shippable docs/examples/ teaching walkthroughs, per ADR-023 §3.4 / safety.sh precedent): %d file(s)\n' \
    "$C_YEL" "$C_RST" "$excepted"
fi

# .myc files under the shipped trees with no @license header at all are not a finding: ADR-023's
# own fixtures show license is tracked per-nodule only when declared, and a nodule may rely on
# its phylum's mycelium-proj.toml (checked in section 3) instead. Report the count for visibility
# (G2), never fail on it.
mapfile -t shipped_myc < <(git ls-files -- 'lib/std/*.myc' 'examples/*.myc')
no_header=0
for f in "${shipped_myc[@]}"; do
  grep -q '@license:' "$f" 2>/dev/null || no_header=$((no_header + 1))
done
if (( no_header > 0 )); then
  printf '  %snote%s  %d shipped .myc file(s) carry no @license header (governed by their phylum'"'"'s mycelium-proj.toml, checked above)\n' \
    "$C_YEL" "$C_RST" "$no_header"
fi

if [[ ${#bad[@]} -eq 0 ]]; then
  ok "$present shipped .myc @license header(s) declare MIT"
else
  fail "${#bad[@]} shipped .myc @license header(s) are not MIT:"
  printf '        %s\n' "${bad[@]}"
  status=1
fi

exit "$status"
