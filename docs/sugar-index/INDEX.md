# Mycelium Sugar Index — the surface-sugar / lowering catalog (v0)

> **Honesty:** `Empirical/Declared — a curated projection of tools/grammar/sugar.yaml (itself a projection of the source DNs + token.rs). sugar.yaml + the cited DNs + token.rs are ground truth; use this index to find where to Read, not as an authoritative reference.`
> Realizes DN-38 §6's per-feature Lowering Map as a generated artifact. Source of truth: `tools/grammar/sugar.yaml` (hand-authored) + `crates/mycelium-l1/src/token.rs` (mechanically cross-checked).
> **Native strategy** (M-1058 follow-up, DN-111): the ratified DN-111 taxonomy — `NativeEquivalent` (alias Adaptation) · `IdiomaticRemapping` (alias Solution) · `Approximation` · `InteropBridge` (alias Bridge), or `unclassified` where no row exists to classify yet (a `Gap`/superseded keyword) — never a fabricated guess (VR-5). See `.claude/skills/native-translate/SKILL.md` for the classification procedure.

## Sugars (identifier keywords — `token.rs::keyword()`, cross-checked against `generate.py`'s `STRUCTURAL_KEYWORDS`)

| Keyword | Status | Grammar rule | Lowering target | Defining doc | Build status | Native strategy |
|---|---|---|---|---|---|---|
| `backbone` | reserved | ebnf: not yet productionized (DN-03 §4 / RFC-0008 §4.5 reserved-not-active); token.rs:447 | Gap — reserved runtime-vocabulary term, no L1 construct yet | DN-03 §4; RFC-0008 §4.5 | Gap | unclassified |
| `colony` | active | ebnf:colony_expr (mycelium.ebnf:344); token.rs:437 | native — structured-concurrency scope grouping hyphae; an expression, not a top-level item | RFC-0008 §4.7 (M-666) | Empirical | NativeEquivalent |
| `consume` | active | ebnf:consume_expr (mycelium.ebnf:328); token.rs:452 | affine acquisition of a Substrate value (LR-8); v0 elaborates to a never-silent Residual | DN-03 §1 (M-664) | Declared | NativeEquivalent |
| `cyst` | reserved | ebnf: not yet productionized (DN-03 §4 / RFC-0008 §4.5 reserved-not-active); token.rs:444 | Gap — reserved runtime-vocabulary term, no L1 construct yet | DN-03 §4; RFC-0008 §4.5 | Gap | unclassified |
| `default` | active | ebnf:default_item (mycelium.ebnf:145); token.rs:485 | native — declares the nodule/phylum default ambient paradigm | RFC-0012 (ambient repr); not an itemized DN-38 §6 row for this specific keyword | Declared | NativeEquivalent |
| `derive` | active | ebnf:derive_item (mycelium.ebnf:204); token.rs:464 | applies a registered `lower` rule at a use site; a nameable, content-addressed (hash-deduped) L0 artifact | DN-54; DN-38 §8.1 (naming settled: `derive` over `grow`) | Empirical | NativeEquivalent |
| `else` | active | ebnf:if_expr (mycelium.ebnf:292); token.rs:481 | native — if_expr alternative separator | none — general L1 surface | Declared | NativeEquivalent |
| `fn` | active | ebnf:fn_item (mycelium.ebnf:169); token.rs:474 | native — the base callable/function-declaration form | none — general L1 surface | Declared | NativeEquivalent |
| `for` | active | ebnf:for_expr (mycelium.ebnf:300); token.rs:483 | elab_for -> self-recursive tail Fix fold over a finite acyclic spine | RFC-0007 §4.8; DN-36 §2.1; DN-38 §6 | Exact | NativeEquivalent |
| `forage` | active | ebnf:hypha (mycelium.ebnf:345, the '@' 'forage' '(' expr ')' prefix); token.rs:446 | native — adaptive placement-policy annotation on a hypha (D-lite surface) | DN-70 D1 (M-906) | Empirical | NativeEquivalent |
| `fuse` | active | ebnf:fuse_expr (mycelium.ebnf:354); token.rs:441 | native — lawful binary merge over the Fuse semilattice (RFC-0008 RT6) | DN-58 §A (M-667) | Empirical | NativeEquivalent |
| `graft` | reserved | ebnf: not yet productionized (DN-03 §4 / RFC-0008 §4.5 reserved-not-active); token.rs:443 | Gap — reserved runtime-vocabulary term, no L1 construct yet | DN-03 §4; RFC-0008 §4.5 | Gap | unclassified |
| `grow` | reserved | ebnf: not productionized — a teaching-diagnostic reject only (parse.rs); token.rs:453 | superseded by `derive` (DN-38 §8.1 / M-812); cannot be used as an identifier (G2) | DN-38 §8.1; M-812 | Declared | unclassified |
| `hypha` | active | ebnf:hypha (mycelium.ebnf:345); token.rs:440 | native — spawns one concurrent task; only valid inside a colony block (RT7 — an orphan hypha is not expressible) | RFC-0008 §4.7 (M-666) | Empirical | NativeEquivalent |
| `if` | active | ebnf:if_expr (mycelium.ebnf:292); token.rs:479 | native — already an L0 form (Match over a boolean scrutinee) | none — general L1 surface | Declared | NativeEquivalent |
| `impl` | active | ebnf:impl_item (mycelium.ebnf:132); token.rs:473 | trait-instance impl: dictionary-passing monomorphization (mono.rs); inherent impl (M-664): desugars to top-level free functions (KC-3, zero kernel growth) | RFC-0019 (trait instance); DN-03 §1 / M-664 (inherent) | Empirical | IdiomaticRemapping |
| `in` | active | ebnf:let_expr (mycelium.ebnf:291); token.rs:478 | native — let_expr binder/body separator | none — general L1 surface | Declared | NativeEquivalent |
| `lambda` | active | ebnf:lambda_expr (mycelium.ebnf:213); token.rs:457 | native (parses) — closure form (Lam); capture/closure SEMANTICS deferred | RFC-0037; M-704 | Declared | Approximation |
| `let` | active | ebnf:let_expr (mycelium.ebnf:291); token.rs:477 | native — already an L0 form (Let); no dedicated desugar pass | none — general L1 surface; not an itemized DN-38 §6 row | Declared | NativeEquivalent |
| `lower` | active | ebnf:lower_item (mycelium.ebnf:203); token.rs:462 | declares a user-extensible generative-lowering rule; RHS elaborated to L0 (KC-3 kernel-growth guard: existing L0 nodes only) | DN-54 | Empirical | NativeEquivalent |
| `match` | active | ebnf:match_expr (mycelium.ebnf:293); token.rs:482 | native — already an L0 form (Match); Maranget usefulness exhaustiveness-checked | none — general L1 surface | Declared | NativeEquivalent |
| `matured` | reserved | ebnf: reserved header-key marker (mycelium.ebnf comment lines 15-18); no term production; token.rs:475 | native — scope-level header key (`// @matured: true`, Nodule-Header spec §3) feeding the totality gate; NOT a term-position construct | RFC-0017 (Enacted); DN-38 §6 (Maturation row) | Exact | NativeEquivalent |
| `mesh` | reserved | ebnf: not yet productionized (DN-03 §4 / RFC-0008 §4.5 reserved-not-active); token.rs:442 | Gap — reserved runtime-vocabulary term, no L1 construct yet | DN-03 §4; RFC-0008 §4.5 | Gap | unclassified |
| `nodule` | active | ebnf:nodule_header (mycelium.ebnf:110); token.rs:433 | native — the basic static unit / module-opening header | RFC-0006 §4.3; DN-06 | Declared | NativeEquivalent |
| `object` | active | ebnf:object_item (mycelium.ebnf:190); token.rs:458 | type + impl + via (checkty.rs Phase 0 structural expansion + Phase 0b via-forwarding impl generation) | DN-53 | Empirical | IdiomaticRemapping |
| `paradigm` | active | ebnf:default_item (mycelium.ebnf:145-146); token.rs:486 | native — sub-keyword of default_item/with_expr naming Binary/Ternary/Dense/VSA | RFC-0012; RFC-0012 §4.2 | Declared | NativeEquivalent |
| `phylum` | active | ebnf:phylum_header (mycelium.ebnf:92); token.rs:436 | native — library/package-scale grouping header (a grouping, not a container) | RFC-0006 §4.3; DN-06; M-662 (activated it — was reserved-not-active before) | Declared | NativeEquivalent |
| `policy` | active | ebnf:swap_expr (mycelium.ebnf:310); token.rs:495 | native — swap_expr sub-keyword naming the swap policy path | same as swap | Declared | NativeEquivalent |
| `priv` | active | ebnf: not yet in mycelium.ebnf (EBNF-drift gap — DN-104 lands after the committed grammar snapshot); token.rs:470 | native — per-constructor seal marker (dual to pub); meaningful only before a constructor name | M-1027 / DN-104 | Empirical | NativeEquivalent |
| `pub` | active | ebnf:type_item (mycelium.ebnf:151, representative — 'pub'? also in trait_item/fn_item/object_item); token.rs:467 | native — cross-nodule/phylum export marker (M-662); granular item-level pub (DN-53 §B) | M-662; DN-53 §B | Empirical | NativeEquivalent |
| `reclaim` | active | ebnf:reclaim_expr (mycelium.ebnf:362); token.rs:449 | native — supervised reclamation scope (RFC-0008 RT7) | DN-58 §B (M-667) | Empirical | NativeEquivalent |
| `spore` | active | ebnf:spore_expr (mycelium.ebnf:320); token.rs:489 | native — deployable/published-artifact constructor | ADR-013 | Declared | NativeEquivalent |
| `swap` | active | ebnf:swap_expr (mycelium.ebnf:310); token.rs:484 | native — the never-silent representation-change primitive itself; it IS the L0 form (target + policy always lexical, S1/WF2), not a sugar lowering to something else | RFC-0006 (surface layer); per-swap guarantee tags live in the numerics/repr code, out of this registry's v0 scope | Declared | NativeEquivalent |
| `thaw` | active | ebnf:fn_item (mycelium.ebnf:169); token.rs:476 | native — de-maturation marker: keeps one fn interpreted inside an otherwise-matured scope | RFC-0017 §4.3 (Enacted) | Exact | NativeEquivalent |
| `then` | active | ebnf:if_expr (mycelium.ebnf:292); token.rs:480 | native — if_expr consequence separator | none — general L1 surface | Declared | NativeEquivalent |
| `tier` | active | ebnf:tier_fn_item (mycelium.ebnf:174, the '@' 'tier' '(' tier_mode ')' prefix); token.rs:448 | native — @tier(compiled\|interpreted) execution-mode hint attached to a fn declaration | DN-58 §C (M-667) | Empirical | NativeEquivalent |
| `to` | active | ebnf:swap_expr (mycelium.ebnf:310); token.rs:494 | native — swap_expr sub-keyword naming the target type | same as swap | Declared | NativeEquivalent |
| `trait` | active | ebnf:trait_item (mycelium.ebnf:161); token.rs:472 | dictionary-passing -> monomorphization + static dispatch (mono.rs; no runtime vtable) | DN-38 §6 (Traits/generics row); RFC-0019 | Empirical | NativeEquivalent |
| `type` | active | ebnf:type_item (mycelium.ebnf:151); token.rs:471 | Construct / Match (RFC-0011 Enacted r3) | DN-37 §2.1/§3.3; DN-38 §6 (Objects row) | Exact | NativeEquivalent |
| `use` | active | ebnf:use_item (mycelium.ebnf:139); token.rs:465 | native — cross-nodule import | none — general L1 surface | Declared | NativeEquivalent |
| `via` | active | ebnf:via_clause (mycelium.ebnf:192); token.rs:461 | generated forwarding impl (delegates to a held field by index; not subtype inheritance) | DN-53 | Empirical | IdiomaticRemapping |
| `wild` | active | ebnf:wild_expr (mycelium.ebnf:317); token.rs:488 | native — audited FFI-floor escape hatch; only legal inside a @std-sys nodule (checker-enforced, never-silent) | RFC-0016 §8-Q6; M-661 | Declared | NativeEquivalent |
| `with` | active | ebnf:with_expr (mycelium.ebnf:289); token.rs:487 | native — explicit ambient-paradigm override scope | RFC-0012 §4.2 | Declared | NativeEquivalent |
| `wrapping` | reserved | ebnf: not yet productionized (RFC-0034 §10/§10.1 reserved opt-out marker); token.rs:493 | Gap — reserved named modular-arithmetic opt-out marker, no term production yet | RFC-0034 §10 (CU-5) | Gap | unclassified |
| `xloc` | reserved | ebnf: not yet productionized (DN-03 §4 / RFC-0008 §4.5 reserved-not-active); token.rs:445 | Gap — reserved runtime-vocabulary term, no L1 construct yet | DN-03 §4; RFC-0008 §4.5 | Gap | unclassified |

## Glyph sugars (single-token, lexed outside `keyword()` — not part of the mechanical cross-check)

| Glyph | Status | Grammar rule | Lowering target | Defining doc | Build status | Native strategy |
|---|---|---|---|---|---|---|
| `?` | active | ebnf: not yet in mycelium.ebnf (EBNF-drift gap, DN-102 §4); lexer: crates/mycelium-l1/src/lexer.rs (single(Tok::Question)) | `let x = e? in body` -> a type-directed match: Result -> `match e { Ok(x) => body, Err($f) => Err($f) }`; Option -> `match e { Some(x) => body, None => None }` (DN-102 §2/§4)  | DN-102 | Declared | NativeEquivalent |

## Not-yet-lexed sugars (documented, no token.rs entry — not part of the mechanical cross-check)

| Name | Status | Grammar rule | Lowering target | Defining doc | Build status | Native strategy |
|---|---|---|---|---|---|---|
| `reveal` | not-lexed | — | tooling — reads the L0 term, emits surface (round-trips in certified mode); no surface grammar production exists yet | DN-38 §5, §8.1 | Gap | unclassified |
