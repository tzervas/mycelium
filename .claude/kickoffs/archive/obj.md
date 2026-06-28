# Kickoff `obj` — Object-Composition Surface & Granular Visibility (DN-53 / M-811)

> Stowed kickoff, UID **`obj`**. Read `CLAUDE.md`, `.claude/kickoffs/README.md`, and
> **`.claude/kickoffs/_doc-maintenance.md`** (anti-drift) first.

## Metadata
| Field | Value |
|---|---|
| **UID** | obj |
| **Head/working branch** | `claude/head/obj-object-surface` (off `dev`) |
| **Status** | ready (3rd post-grammar wave) |
| **Swarm mode** | serial-on-L1 (inline, ~280 LoC) |
| **Depends on** | DN-53 (Accepted), RFC-0037 grammar (Enacted — `object` keyword already reserved) |

## Scope
Enact the **`object`** composition surface (no OOP `class`) + **granular item-level `pub`**. `object`
**desugars** to `type` + `impl` + `via` in `checkty.rs` (elab unchanged — it sees only the lowered
forms), so it is **zero kernel growth** and `reveal`-able. Field-level `pub` stays deferred (DN-53).

**Issue:** M-811.

## Grounding (doc_refs)
- `corpus:DN-53` — the object keyword, the desugaring (type+impl+via), the transparency invariant,
  granular `pub` model; `corpus:DN-37` (the Q3 object/composition ruling it implements).
- `src:crates/mycelium-l1/src/parse.rs` (the `object` reserved teaching-arm to replace) ·
  `src:crates/mycelium-l1/src/ast.rs` (`Item::Object`) · `src:crates/mycelium-l1/src/checkty.rs`
  (`check_object_decl` desugar at `check_nodule_with`).

## Approach (serial-on-L1, inline)
parse.rs `parse_object_decl` (reuse `parse_type_ref`/`parse_impl_decl`) → ast.rs `Item::Object(ObjectDecl)`
plus `ViaDecl` → checkty.rs desugar to TypeDecl + ImplDecl(s) + FnDecl(s) (delegating to existing checkers),
plus granular `pub` (additive over the `Vis` model; never-silent access refusal). elab.rs: no change. Add
accept/reject conformance fixtures + a three-way differential on a desugared object.

## Definition of Done
- [ ] `object … { … }` parses, desugars, type-checks, and runs three-way (via its `type`/`impl`/`via`
  lowering); granular item-level `pub` enforced never-silently (G2). **No OOP `class`; zero L0 growth (KC-3).**
- [ ] `just check` green; honest tags (VR-5).
- [ ] **Doc maintenance (anti-drift):** `issues.yaml` M-811 → done; **DN-53 `Accepted → Enacted`**;
  `.claude/memory/lang-lexicon-syntax.md` notes `object` now **active** (DN-02/03 status); `mycelium.ebnf`
  gains the `object`/`via` productions (+ `just grammar-gen`); `CHANGELOG.md` entry; `docs/api-index/`
  regenerated if public API changed.

## Landing
`/wave-land` → `main` after green + self-review + curated squash; backprop. Sequence next: **`low`**.
