# Kickoff `lwd` — DN-54 lower/derive elaboration + KC-3 kernel-growth guard (M-812-cont)

> Stowed kickoff, UID **`lwd`**. Read `CLAUDE.md`, `.claude/kickoffs/README.md`, and
> **`.claude/kickoffs/_doc-maintenance.md`** (anti-drift) first.

## Metadata
| Field | Value |
|---|---|
| **UID** | lwd |
| **Working branch** | `claude/work/lwd-lower-derive-elab` (off `dev`) |
| **Status** | ready — `low` (M-812) landed the `lower`/`derive` **surface + structural checks**; this lands the **load-bearing safety** it deferred |
| **Swarm mode** | serial-on-L1 (inline; `elab.rs` + `checkty.rs`) |
| **Depends on** | M-812 (landed — surface + structural checks); DN-54 (Accepted, pending ratification) |

## Scope
Close the **deferred completion of DN-54** (`M-812-cont`). `low` (M-812) landed the `lower`/`derive`
surface keywords + structural checks (rule/param-name uniqueness, derive name-resolution), but `derive`
currently emits **no L0** — an honest never-silent residual (pinned by
`mycelium-l1/src/tests/checkty.rs::lower_derive_items_add_no_l0_to_an_unrelated_entry`). This wave lands
the parts that only become meaningful once `derive` actually elaborates:
1. **RHS elaboration to L0** — `derive` rewrites to its generated L0 (the `crate::elab` site that reads
   `Env::lower_rules`), never-silent on a malformed rule (G2).
2. **§4.1 IL-grammar RHS type-check** — `infer_type` the rule RHS; reject mutation/FFI/`wild` in a
   lowering rule.
3. **§6 KC-3 kernel-growth guard** — the load-bearing safety: a `lower` rule must **not** grow the L0
   kernel surface (the whole point of DN-54 — meaningful only once (1) lands).
4. **§4.2 cross-rule acyclicity** — reject a cyclic `lower` rule set.
5. **§7 verification harness** — differential / hygiene / round-trip tests.

On completion: **DN-54 → Enacted** (surface + structural + elaboration + KC-3 guard all landed).

## Grounding (doc_refs)
- `corpus:DN-54` — the generative-lowering surface + the deferred-safety boundary (§4.1/§4.2/§6/§7).
- `corpus:DN-38#§8.1` — `grow → derive` supersession / lowering-atlas placement.
- `src:crates/mycelium-l1/src/elab.rs` (the elaboration site — must read `Env::lower_rules`) ·
  `src:crates/mycelium-l1/src/checkty.rs` (`Env::lower_rules`, the structural pass to extend) ·
  the guard test `src:crates/mycelium-l1/src/tests/checkty.rs`.

## Approach (serial-on-L1, inline)
Extend the checker's `lower`-rule pass to type-check the RHS (§4.1) and assert acyclicity (§4.2); wire
`elab.rs` to expand `derive` applications through `Env::lower_rules` to L0 (1), with the **KC-3
kernel-growth guard** (§6) asserting the generated L0 introduces no new kernel surface — never-silent on
violation (G2). Replace the no-L0 residual + its guard test with the real elaboration + a §7 harness
(differential `observe(derive x) == observe(hand-lowered x)`, hygiene, round-trip). Honest tags: the
elaboration earns `Empirical` via the differential; the KC-3 guard is `Proven`-by-construction that no
L0 node is added.

## Definition of Done
- [ ] `derive` elaborates to L0 (no longer a residual); the **KC-3 kernel-growth guard** rejects any rule
  that would grow the kernel (never-silent, G2); cross-rule cycles rejected; RHS type-checked.
- [ ] `just check` green; the §7 harness (differential + hygiene + round-trip) green; honest tags
  (elaboration `Empirical`; KC-3 guard `Proven`-by-construction).
- [ ] **Doc maintenance (anti-drift):** `issues.yaml` **M-812-cont → done**; **DN-54 → Enacted**
  (surface + structural + elaboration + guard all landed; step through, maintainer nod); `CHANGELOG.md`
  entry; `docs/api-index/` if `elab`/`checkty` public API changed; the residual guard test replaced.

## Landing
`/wave-land` → `dev → integration → main` after green + `/pr-review` + curated squash; backprop.
**Serializes on the `crates/mycelium-l1/src/{checkty,elab}.rs` L1 track** — one L1 wave in flight at a
time across `lwd`/`s10`/`srf`/`hof`/`strm`/`r10`(M-712).
