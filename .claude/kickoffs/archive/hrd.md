# Kickoff `hrd` — RFC-0028 Input-Validation Hardening (A1/A2/A3, must-fix before E14-1)

> Stowed kickoff, UID **`hrd`**. Read `CLAUDE.md`, `.claude/kickoffs/README.md`, and
> **`.claude/kickoffs/_doc-maintenance.md`** (anti-drift) first.

## Metadata
| Field | Value |
|---|---|
| **UID** | hrd |
| **Head/working branch** | `claude/head/hrd-input-validation` (off `dev`) |
| **Status** | ready (**CRITICAL** — A1 is an active parser DoS in the shipped L1 parser) |
| **Swarm mode** | serial-on-L1 (inline; small, security-sensitive) |
| **Depends on** | RFC-0028 §4.4 (Accepted/signed-off — the host-encoding validation bridge), DN-40 (the A1/A2/A3 findings) |

## Scope
Close the three DN-40 input-validation gaps that **RFC-0028 §4.4 commissioned as must-fix before any FFI
(E14-1) work**:
- **A1 (CRITICAL, `Proven`):** unbounded type-subgrammar recursion → stack overflow → SIGABRT.
  `crates/mycelium-l1/src/parse.rs:685-700` (+ `816-823`) — no depth guard on `parse_type_ref`/nested
  type args. A hostile/large input crashes the parser. **Add a recursion-depth guard** (a bounded
  `depth` counter → explicit `ParseError` past a named `PARSE_MAX_DEPTH`, never-silent G2).
- **A2 (HIGH, `Proven`-by-structure):** the same overflow shape on `parse_pattern`
  (`parse.rs:1125-1146`) — apply the same depth guard.
- **A3 (HIGH, `Proven`):** dep-hash stored as free-text `Option<String>` instead of a typed
  `ContentHash` — `crates/mycelium-proj/src/manifest.rs:332`. **Parse-into-typed** (RFC-0028 §4.4.1):
  validate/parse the hash into a `ContentHash` at the boundary; reject malformed (never trust raw).

## Grounding (doc_refs)
- `corpus:RFC-0028#§4.4` — the ratified parse-into-typed / injective-encode / bounded obligations + the
  A1/A2/A3 must-fix-before-E14-1 sequencing (`docs/rfcs/RFC-0028-FFI-and-System-Interface.md` §4.4).
- `corpus:DN-40` — the input-validation architecture + the A1/A2/A3 findings with exact line numbers.
- `src:crates/mycelium-l1/src/parse.rs:685` · `src:crates/mycelium-l1/src/parse.rs:1125` ·
  `src:crates/mycelium-proj/src/manifest.rs:332`.

## Approach (serial-on-L1, inline)
A1/A2: thread a `depth: usize` (or a `Parser`-field counter) through the recursive type/pattern descent;
increment on entry, decrement on exit (or RAII guard); past `PARSE_MAX_DEPTH` return a never-silent
`ParseError::TooDeep { limit }` naming the limit (G2). Add fuzz-style regression fixtures (deeply-nested
type + pattern) under `reject/` asserting the explicit refusal (NOT a crash). A3: introduce/parse a
`ContentHash` at the manifest boundary; a malformed hash is an explicit error. Pick `PARSE_MAX_DEPTH` with
a stated rationale (Declared until measured).

## Definition of Done
- [ ] Deeply-nested type/pattern input → explicit never-silent `ParseError` (no SIGABRT/stack overflow);
  reject fixtures + a fuzz-smoke (`cargo fuzz run fuzz_l1_parse` no panic) prove it. A3 dep-hash is typed.
- [ ] `just check` green (incl. conformance + the new reject fixtures); honest tags (the guard is
  `Proven`-by-construction that recursion is bounded; the chosen depth limit is `Declared`).
- [ ] **Doc maintenance (anti-drift):** `issues.yaml` — close the DN-40 A1/A2/A3 items / the RFC-0028
  §4.4 commissioned-fix task → done; **RFC-0028 §4.4.4** note A1/A2/A3 **closed** (append-only); DN-40
  status updated; `CHANGELOG.md` entry; `docs/api-index/` if `ParseError`/`ContentHash` public API changed.

## Landing
`/wave-land` → `main` after green + `/security-review` + `/pr-review` self-review + curated squash; backprop.
(Unblocks E14-1/M-722 — the FFI host-encoding bridge is sequenced to land *after* these.)
