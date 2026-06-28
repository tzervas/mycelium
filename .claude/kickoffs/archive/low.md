# Kickoff `low` — User-Extensible Generative Lowering (`lower`/`derive`) (DN-54 / M-812)

> Stowed kickoff, UID **`low`**. Read `CLAUDE.md`, `.claude/kickoffs/README.md`, and
> **`.claude/kickoffs/_doc-maintenance.md`** (anti-drift) first.

## Metadata
| Field | Value |
|---|---|
| **UID** | low |
| **Head/working branch** | `claude/head/low-generative-lowering` (off `dev`) |
| **Status** | ready (4th post-grammar wave) |
| **Swarm mode** | serial-on-L1 (inline, ~330 LoC) |
| **Depends on** | DN-54 (Accepted), `obj` (can reference object types in a rule RHS), the DN-38 §7 verification harness |

## Scope
Enact **`lower`** (user-extensible generative-lowering rules) + **`derive`** (rule application). A
`lower Name[…] = <RHS>` registers a rule whose RHS is a **typed, real Mycelium term**, IL-grammar-checked
at definition time and `reveal`-able by construction — extensions are transparent surface→L0 rules,
verified exactly like built-in lowerings (never an opaque macro). KC-3: an RHS must lower to **existing**
L0 nodes; the checker rejects any attempt to grow the kernel.

**Issue:** M-812. (Note the `grow → derive` reconciliation, DN-38 §8.1 — settle the `derive` keyword
here, coordinating with the still-reserved `grow`.)

## Grounding (doc_refs)
- `corpus:DN-54` — the extension-definition surface, inspectability-by-construction, the checker rules
  (IL-grammar / acyclicity / hygiene-by-construction / KC-3 / never-silent), the verification discipline;
  `corpus:DN-38` (§6 Lowering Map + §7 harness; §8.1 `grow→derive`).
- `src:crates/mycelium-l1/src/parse.rs` (the `lower` reserved teaching-arm) · `…/ast.rs` (`Item::Lower`)
  · `…/checkty.rs` (`check_lower_decl` + `Env.lower_rules` registry) · `…/elab.rs` (`derive` rule lookup
  with instantiation at elaborate).

## Approach (serial-on-L1, inline)
parse.rs `parse_lower_decl` → ast.rs `Item::Lower(LowerDecl)` → checkty.rs validate RHS (`infer_type`,
reject mutation/FFI/kernel-growth) + register in `Env.lower_rules` → elab.rs `derive` elaboration (look up
rule, instantiate RHS with the target, elaborate). Verification per DN-54: differential `observe(surface)
== observe(lower(surface))` + hygiene corpus + round-trip (`delaborate ∘ lower = id`, certified mode).

## Definition of Done
- [ ] `lower`/`derive` parse, type-check (IL-grammar + acyclicity + hygiene + KC-3 all never-silent, G2),
  and elaborate; a user rule runs three-way and is `reveal`-able. **Zero new L0 nodes (KC-3).**
- [ ] `just check` green; tags honest (VR-5 — `Empirical` post-differential; `Proven`-per-run only with a
  certified-mode witness).
- [ ] **Doc maintenance (anti-drift):** `issues.yaml` M-812 → done; **DN-54 `Accepted → Enacted`**; the
  `grow→derive` reconciliation recorded in **DN-38 §8.1** + `docs/Glossary.md`/DN-02/03 (keyword status);
  `.claude/memory/lang-lexicon-syntax.md`; `mycelium.ebnf` `lower`/`derive` productions (+ `just grammar-gen`);
  `CHANGELOG.md` entry; `docs/api-index/` if API changed.

## Landing
`/wave-land` → `main` after green + self-review + curated squash; backprop. (Completes the approved
R4→R3→DN-53→DN-54 semantic-wave sequence.)
