<!--
  TEMPLATE for a standard-library module spec (docs/spec/stdlib/<module>.md).
  Copy this file, fill every section, delete these HTML comments and any
  "(guidance: …)" notes. Single-template conformance is checked by the §4.1
  doc quality-bar lint. Keep prose tight; the guarantee matrix (§4) is the
  load-bearing deliverable, not the prose.
-->
# Spec — `std.<module>` (`<one-line what it is>`)

| Field | Value |
|---|---|
| **Status** | **Draft (needs-design)** (2026-06-17) — design-first; no code lands until RFC-0016 is Accepted and this spec is ratified (the maintainer's append-only decision). |
| **Module / Ring** | `std.<module>` · Ring `<0 | 1 | 2>` (RFC-0016 §4.2) · Tier `<A | B>` |
| **Tracks** | `<M-5xx>` (#`<issue>`) — the Phase-5 task this spec delivers |
| **Scope** | `<what this module is, in one or two sentences — the exported surface it owns>` |
| **Boundary** | `<what is explicitly OUT of scope, and which adjacent module owns it — e.g. "a representation change is std.swap (M-516), not a std.cmp convert">` |
| **Depends on** | `<the Accepted RFC/ADR this module is the library form of>`; RFC-0016 §4.1 (the contract); RFC-0001 (the value model) |
| **Grounds on** | `<the landed capability crate(s) / corpus this consumes — KC-3: above the kernel>` |

---

## 1. Summary

`<2–4 sentences: what the module is, the user-facing surface, and — explicitly — its HONESTY CRUX (the
silent-default this module structurally forbids, e.g. "parse returns Result, never a sentinel"). State the
ring and that it adds no trusted code (KC-3) / consumes which landed crate.>`

## 2. Scope & module boundary

- **In scope:** `<the operations / types this module owns>`
- **Out of scope (and who owns it):** `<the adjacent surface and its module — make the boundary explicit so
  the modules do not overlap; cite the owning task>`
- **Ring & layering:** `<which ring, what it re-exports vs wraps vs builds new; KC-3 statement>`

## 3. Exported-op surface (design sketch)

`<A signature sketch of the exported types + ops. Value-semantic, immutable-by-default. Fallible ops return
Option/Result. Effectful ops declare their effect on the signature (C6). This is a DESIGN sketch — enough to
fix the surface and feed the guarantee matrix, not a committed grammar.>`

```
// illustrative signatures (not a committed surface)
<type / fn sketches>
```

## 4. Guarantee matrix (the load-bearing deliverable — RFC-0016 §4.5)

Rows = exported ops. Encoded as a checked table (the RFC-0003 §4 template), asserted in tests once code
lands — never prose only.

| Op | Guarantee tag | Fallibility (explicit error set) | Declared effects | EXPLAIN-able? |
|---|---|---|---|---|
| `<op>` | `<Exact \| Proven \| Empirical \| Declared>` | `<the explicit Err/None set, or "total">` | `<none \| io \| time \| rand \| alloc(budget)>` | `<yes (artifact) \| n/a>` |

`<Below the table: justify any tag that is not Exact — cite the theorem (and its CHECKED side-conditions)
for Proven, the method for Empirical, the assertion for Declared. Downgrade rather than overclaim (VR-5).>`

## 5. §4.1 contract conformance (C1–C6)

`<One bullet per clause, stating concretely how THIS module meets it — not restating the rule. FLAG any
clause this module cannot yet fully meet as a §7 open question rather than asserting compliance.>`

- **C1 — never-silent (G2):** `<…>`
- **C2 — honest per-op tag (VR-5):** `<…>`
- **C3 — no black boxes / EXPLAIN (SC-3/G11):** `<…>`
- **C4 — content-addressed, value-semantic (ADR-003):** `<…>`
- **C5 — above the kernel (KC-3):** `<…>`
- **C6 — declared, bounded effects (RFC-0014):** `<…>`

## 6. Grounding

`<Every normative claim cites its basis: the Accepted RFC/ADR, the requirement ids (G/VR/FR/NFR/KC/SC), the
landed crate. No ungrounded "facts" (the house grounding rule).>`

## 7. Open questions (FLAGGED — resolve before ratification)

`<Genuine open questions only, each FLAGGED — never invented into a false-confident design choice (the
planning analogue of G2). Tie cross-module questions to the relevant RFC-0016 §8 item where one applies.>`

- **(Q1) `<…>`** — `<disposition>`

## Meta — changelog

- **2026-06-17 — Draft (needs-design).** `<one-paragraph summary: the module scope, the honesty crux, the
  guarantee-matrix shape, the §4.1 conformance, the grounding, and the FLAGGED questions. No code; no kernel
  change (KC-3). Append-only.>`
