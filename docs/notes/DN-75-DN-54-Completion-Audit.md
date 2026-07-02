# Design Note DN-75 — DN-54 Completion Audit (M-917): the `lwd`-Landed M-812-cont Checks, Verified

| Field | Value |
|---|---|
| **Note** | DN-75 |
| **Status** | **Resolved** (2026-07-02 — accepted by the wave orchestrator under the maintainer's 2026-07-02 delegation (`Declared`), per the integration-reconcile promotion gate; the §6 **Option 1** recommendation is adopted: the DN-54 completion audit is accepted as-is (all four conditions verified against the tree, not the changelog), DN-54 stays `Accepted`, and E-1 is a one-line editorial clarification for DN-54's next changelog. Was **Recommended — pending orchestrator acceptance** 2026-07-02; that history stands unchanged below — append-only forward transition to Resolved, house rule #3.) The maintainer **delegated this disposition to the orchestrator** (2026-07-02). |
| **Feeds** | Kickoff `grm` DoD row 2 (DN-54 completion — the extension-checker verified complete); **M-918** (the derive-site consumption-model dossier — this audit is its declared input, `depends_on: M-917`); **M-919** (extension-checker enactment — the residual ledger §5 names what its DoD must absorb); the DN-54 **Enacted** gate (DN-54 §9). |
| **Date** | July 2, 2026 |
| **Decides** | *Nothing normatively.* Records (1) the audit method and the audited tip (§1); (2) the §7-harness re-run result (§2); (3) the per-section audit table — landed / residual / N-A with `file:line` evidence, `Empirical` (§3); (4) one editorial discrepancy (§4); (5) the residual ledger — every gap routed to a task or FLAG, none silent (§5); (6) options + the recommendation for the delegated disposition (§6); (7) guarantee posture + DoD (§7). |
| **Task** | M-917 (kickoff `grm` — Phase-I H2a grammar-stability gate) |

> **Posture (transparency rule / VR-5 / G2).** This is a **verification audit**, not a design
> pass: it checks the `lwd`-landed M-812-cont completions (DN-54 changelog, 2026-06-29) against
> DN-54 *as written* and reports what is actually in the tree. Verdicts are **`Empirical`** —
> earned by reading the source at a pinned tip and re-running the §7 harness, never by trusting
> the changelog's own claims (house rule #4: the changelog is the *audited*, not the evidence).
> Every gap is named and routed (G2). The disposition decision is the orchestrator's under the
> maintainer's 2026-07-02 delegation; nothing here is self-ratified.

---

## §1 Scope and method

**Audited claim set.** The DN-54 2026-06-29 changelog entry (the `lwd` kickoff / M-812-cont
landing) claims five completions: (1) RHS elaboration to L0; (2) the §4.1 IL-grammar RHS
type-check; (3) the §6 KC-3 kernel-growth guard; (4) §4.2 cross-rule acyclicity; (5) the §7
verification harness. M-917's DoD (kickoff `grm`, PM table): a per-section audit table
(landed / residual / N-A) recorded `Empirical`; the §7 harness re-run green; residuals become
explicit tasks or FLAGs — none silent.

**Method (`Empirical`).** DN-54 read in full as written (841 lines, including the §10 M-824
addendum); the implementation read directly in `crates/mycelium-l1` at the audited tip; the
harness and the change-scoped unit tests re-run. Evidence below is `file:line` **at the audited
tip** — line numbers drift with later edits; the tip pins them.

**Audited tip:** `629aa12` (the `dev` tip this audit branched from, 2026-07-02).

---

## §2 The §7 harness re-run (`Empirical`, DoD gate)

Re-run on the audited tip, 2026-07-02:

| Command | Result |
|---|---|
| `cargo test -p mycelium-l1 --test lower_derive` | **ok — 5 passed; 0 failed** (`lower_rule_elaboration_structurally_equals_hand_lowered`, `lower_rule_differential_equals_hand_lowered`, `lower_rule_elaboration_stays_in_the_frozen_kernel_kc3`, `lower_rule_elaboration_is_hygienic_no_capture`, `lower_rule_value_round_trip`) |
| `cargo test -p mycelium-l1 --lib lower_` | **ok — 18 passed; 0 failed** (the parse/checkty/elab unit + regression set, including every never-silent refusal test and the derive-isolation guard) |

The DoD's "§7 harness re-run green" gate is met.

---

## §3 Per-section audit table (`Empirical`)

Verdicts: **Landed** (implemented as written, at the stated strength) · **Residual** (an
obligation in DN-54's text not yet implemented — routed in §5) · **N-A** (prose/design content
with no code obligation, or an obligation that cannot arise in v0). All paths are repo-relative;
`l1` = `crates/mycelium-l1`.

| DN-54 § | Obligation as written | Verdict | Evidence (file:line @ `629aa12`) |
|---|---|---|---|
| §2 | "macro" ruled out by the DN-02 gate (naming prose) | N-A | No code obligation. |
| §3.1 | `lower` is an active definition-site keyword | **Landed** | `l1/src/parse.rs:1398` (`parse_lower_decl`), `:1423` (`parse_derive_decl`); tests `l1/src/tests/parse.rs:602,614`; fixtures `docs/spec/grammar/conformance/accept/22-lower-derive.myc`, `reject/26-lower-missing-eq.myc`, `reject/27-derive-missing-for.myc`. |
| §3.2 | Surface form `lower Name[params] = <rhs>`; worked example has an `impl`-block RHS | **Landed** (expression RHS) / **Residual** (item-shaped RHS) | Expression RHS parses + checks + elaborates (rows below). The §3.2 worked example's `impl` block is an **item**, not an `Expr` — not expressible in v0 (`parse_lower_decl` calls the expression parser). Known + FLAGged: DN-54 §10.1(b); routed to M-918/M-919 (§5 R-1). |
| §3.3 | RHS must not contain `wild { … }` | **Landed** | Structural refusal `l1/src/checkty.rs:2013` via `rhs_contains_wild` (`:2077`, depth-budgeted walk); test `l1/src/tests/checkty.rs:366`. |
| §3.3 | RHS must not introduce a new L0 node; no mutual recursion; no token manipulation | **Landed** / N-A | New-node: see §6 row. Cycles: see §4.2 row. Token manipulation: N-A **by construction** — the RHS is a typed `Expr`, no token-stream surface exists. |
| §3.3 | No rules parameterized over runtime-only information | N-A (v0) | v0 rules carry type params only — the synthetic elaboration entry has `value_params: vec![]` (`l1/src/elab.rs:502` body); no runtime-param surface exists to refuse. |
| **§4.1** | **IL-grammar RHS type-check** — ill-typed/ill-formed RHS refused at definition time | **Landed** | `check_lower_rule_rhs_type` `l1/src/checkty.rs:2039–2070` (v0 `infer` over the RHS, rule params in scope as `Ty::Var`, `std_sys: false` — the `wild` gate held closed as defense in depth); invoked over the whole rule set `:1911–1912`; refusal test `l1/src/tests/checkty.rs:341`; acceptance `:352`. *Narrowing note (VR-5):* the check runs at the L1 level via the v0 checker — the "at the elaboration level of its output" per-level nanopass grammar check is realized as L1 type-check **plus** the L0 closed-enum boundary (§6 row), which is what v0's two levels support. `Declared` (structural check, not a theorem), as DN-54 itself tags it. |
| **§4.2** | **Cross-rule acyclicity** — self- and mutual-reference cycles refused | **Landed** | `check_lower_rule_acyclicity` `l1/src/checkty.rs:2127`; edge set = single-segment RHS paths that name other rules, with the ctor/fn false-positive exclusion `:2139–2145`; self-reference refusal `:2168–2180`; iterative (host-stack-safe) DFS `:2185–2214`; tests `l1/src/tests/checkty.rs:383` (self), `:395` (mutual), `:413` (ctor-name false-positive regression). Scope: the **same-nodule** rule set — cross-phylum rules do not exist in v0 (N-A; carried as DN-54 §8 Q6 / §10 OQ-D). |
| §4.3 | Hygiene — typed-AST mechanism, no capture by construction | **Landed** | By construction (typed `Expr` RHS; `%`-fresh binders in elaboration — `%` is not a surface identifier character, `l1/src/elab.rs:502` body); `Empirical` confirmation `l1/tests/lower_derive.rs:187` (every elaborated binder `%`-fresh). |
| §4.4 | Content-addressed, hash-deduplicated derive output | **Residual** | No derive-site output exists yet to address: a `derive` use site is checked (`check_derive_application`, `l1/src/checkty.rs:2096`) but produces no L0 and registers nothing (`Item::Derive` skipped at `:795`; isolation guard `l1/src/tests/checkty.rs:484`). Downstream of the §10 attachment model (§5 R-2). |
| §4.5 | `reveal`-ability of every use, by construction | **Residual** (gated) | The `reveal` inspector is not implemented in `mycelium-l1` (referenced only in a comment, `l1/src/parse.rs:582`). The by-construction argument stands (`Declared` — the L0 term is a first-class value, `elaborate_lower_rule` returns it) but is unexercisable until the DN-38 §5 `reveal` track lands (§5 R-2). |
| §4.6 | Refusal row 1 — ill-typed RHS refused | **Landed** | = §4.1 row; diagnostic `l1/src/checkty.rs:2060–2068`. |
| §4.6 | Refusal row 2 — cycle refused, cycle named | **Landed** | = §4.2 row; diagnostics `:2170–2178`, `:2212+`. |
| §4.6 | Refusal row 3 — `wild` RHS refused | **Landed** | = §3.3 row; `:2013–2023`. |
| §4.6 | Refusal row 4 — new-L0-node RHS refused | **Landed** (by construction) | = §6 row below. |
| §4.6 | Refusal row 5 — use of a rejected rule refused at the `derive` site | **Landed** (by subsumption) | A definition-time refusal fails the whole nodule check (Pass 3e registration loop, `l1/src/checkty.rs:1892–1912`), so no use site of a rejected rule can ever check — the refusal is at least as strong as the row demands, never silent. |
| §4.6 | Refusal row 6 — a `derive` application whose *output* fails its target level's IL-grammar check | **Residual** | No derive-site elaboration exists (§4.4 row) — there is no use-site output to check yet. Downstream of the attachment model (§5 R-2). |
| §5 | Inspectability-by-construction argument (prior-art contrast) | N-A | Prose argument (`Declared`/`Empirical`-at-source per DN-54 §9); its code obligation is the §4.5 row. |
| **§6** | **KC-3 kernel-growth guard** — checked RHS lowers to existing L0 nodes only | **Landed** | `Proven`-by-construction in the **narrow, checked** sense DN-54's changelog states: `elaborate_lower_rule`'s codomain is the closed Rust enum `mycelium_core::Node` (the frozen L0 grammar) — the type system is the checked side-condition (`l1/src/elab.rs:467–502` doc + signature); the §4.6 `wild` refusal closes the surface-growth escape. Harness confirmation `l1/tests/lower_derive.rs:165`. *Honest note (VR-5), already recorded in the test itself (`:153–163`):* the `Node::is_aot_lowerable` assertion is total over the v0 node set — a well-formedness confirmation, **not** an independent KC-3 witness; the substantive guard is the type boundary. The tag as claimed is supportable. |
| **§7.1** | Differential `observe(surface) == observe(lower(surface))` | **Landed** (`Empirical`) / **Residual** (corpus generation + tiering) | Structural identity (one code path) `l1/tests/lower_derive.rs:109`; run-value differential validated through the shared M-210 TV checker (`mycelium_cert::check_core`, `ObservationalEquiv`) `:131`. **Residual:** DN-54 §7.1 as written calls for a *generated* corpus with **DN-20 LOW/HIGH proptest tiering**; the landed corpus is a fixed 4-case table plus one structural-only swap case (`:47–101`) — no property-based generation, no case-count tiering. Narrowed, never dropped; routed (§5 R-3). |
| §7.2 | Hygiene — no free-variable capture over generated programs | **Landed** (`Empirical`) / same corpus **Residual** | `l1/tests/lower_derive.rs:187` (one `let`-binder case). Same fixed-corpus narrowing as §7.1 (§5 R-3). |
| §7.3 | Round-trip `delaborate ∘ lower = id` in `certified` mode | **Landed** (value-level `fast` form) / **Residual** (certified form) | Value round-trip `l1/tests/lower_derive.rs:215`. The certified-mode surface round-trip is not implemented — held `Declared`, explicitly documented in the harness header (`:4–7`); gated on ADR-032 `certified` mode (§5 R-4). |
| §7.4 | Tag posture — `Empirical` from trials; `Proven`-per-run only with a TV witness | **Landed** (posture) / **Residual** (witness path) | `Empirical` is earned (harness + M-210 checker, §2); the `Proven`-per-run certified witness path is not implemented — same `certified` gate (§5 R-4). |
| §8 | Open questions (keyword confirm, mutual-recursion extension, effects, cross-phylum) | N-A | Open by design; Q5/Q6 carried forward into §10 OQ-D/OQ-E. The §3.1 keyword FLAG is de-facto resolved by ratification of DN-54 with `lower` active in the grammar; no live parser collision (the conformance suite passes). |
| §9 | The note's own DoD / status gates | N-A (consistent) | Accepted 2026-06-27; **Enacted correctly still gated** — this audit confirms the gate is real: §4.4/§4.5/§4.6-row-6/§7-tiering/§10 remain open. |
| §10 | Derive-site attachment + parametric instantiation + item-RHS (M-824 design pass) | **Residual by design** | The consume model is the **M-918** dossier's subject (`depends_on: M-917` — this audit). Nullary/monomorphic elaboration is landed; a parametric rule's un-instantiated RHS surfaces as the ordinary `ElabError::Residual` — never-silent (`l1/src/elab.rs:467–502` doc; DN-54 §10.1(c)). |
| *(changelog item 1)* | **RHS elaboration to L0** — `crate::elab` reads `Env::lower_rules` | **Landed** | `elaborate_lower_rule` `l1/src/elab.rs:502` (reads `env.lower_rules` `:503`; `%`-prefixed synthetic nullary entry fed to the ordinary `elaborate` — one code path, the by-construction §7.1 basis); exported `l1/src/lib.rs:67`; registry `Env::lower_rules` `l1/src/checkty.rs:516`; unit tests `l1/src/tests/checkty.rs:435` (elaborates = hand-lowered), `:456` (KC-3), `:469` (unknown-rule refusal). |

**Summary.** All five `lwd`-claimed completions are **landed as claimed, at the tags claimed**
(`Empirical` for the observational identity; `Declared` for the structural checks; `Proven`-
by-construction for the narrow KC-3 type-boundary sense — each tag verified supportable, none
found upgraded past its basis). The residuals are: the §10 consume-model cluster (already
FLAGged by DN-54 itself), the §4.4/§4.5/§4.6-row-6 obligations downstream of it, the §7
corpus-generation/DN-20-tiering narrowing, and the `certified`-mode §7.3/§7.4 forms. None are
silent; all are routed in §5.

---

## §4 Editorial discrepancy found (one; not a defect)

The DN-54 2026-06-29 changelog states the two `low`-era residual guard tests
(`lower_derive_items_add_no_l0_to_an_unrelated_entry`, `lower_rule_rhs_is_stored_not_elaborated`)
were "**replaced** by the real elaboration + guard tests". Audit finding (`Empirical`):
`lower_rule_rhs_is_stored_not_elaborated` is indeed gone (zero matches in the tree), but
`lower_derive_items_add_no_l0_to_an_unrelated_entry` was **retained and repurposed** as a live
isolation guard — it now pins that a `lower`/`derive` pair splices no L0 into an unrelated entry
(`l1/src/tests/checkty.rs:484–498`), which is exactly the §4.4/§10 residual's never-silent
boundary. The retained test is *better* than removal; only the word "replaced" is imprecise for
it. Routed as FLAG E-1 (§5) for the integrating parent's next append-only DN-54 changelog
entry — a one-line clarification, no history rewrite.

---

## §5 Residual ledger — every gap routed, none silent (G2)

| Id | Residual | Routing |
|---|---|---|
| **R-1** | Derive-site attachment/consumption model; parametric instantiation at the `derive` site; item-shaped (`impl`) RHS (`parse_lower_item_rhs`) | **Already tracked**: DN-54 §10 (design pass, Model A recommended) → **M-918** (consume-model dossier, maintainer-ratification prep) → **M-919** (extension-checker enactment). No new task needed. |
| **R-2** | §4.4 content-addressed dedup of derive output; §4.5 `reveal` exercisability; §4.6 row 6 (use-site output check) | All are downstream of R-1's model choice (they need derive output to *exist* somewhere). **FLAG to the orchestrator:** name these three explicitly in **M-919's DoD** (or the implementing RFC's DoD) so they cannot be dropped when the consume model lands. `reveal` itself additionally rides the DN-38 §5 inspector track. |
| **R-3** | §7.1/§7.2 harness corpus is a fixed table — no property-based generation, no DN-20 LOW/HIGH case-count tiering | **FLAG to the orchestrator:** append to **M-919's DoD** (preferred — the generated-corpus differential only becomes fully meaningful once derive sites elaborate, and `mycelium-l1` is the serial lane) or mint a small dedicated M-task if M-919's scope is held tight. Either way the obligation is now explicit, not silent. |
| **R-4** | §7.3 certified-mode `delaborate ∘ lower = id`; §7.4 `Proven`-per-run TV witness | Gated on ADR-032 `certified` mode — already held `Declared` in DN-54 and documented in the harness header. **FLAG (no new task):** rides the certified-mode track; must appear in the DN-54 Enacted checklist. |
| **E-1** | Changelog wording "replaced" vs retained/repurposed guard test (§4) | **FLAG to the integrating parent:** one-line append-only clarification in DN-54's next changelog entry. Editorial only. |

Shared-file updates this note does **not** make (leaf discipline — FLAGged up, not edited):
`CHANGELOG.md`, `docs/Doc-Index.md`, `docs/api-index/`, `tools/github/issues.yaml`
(M-917 status → done + this note's `doc_refs`) — owned by the integrating parent.

---

## §6 Options and recommendation (the delegated disposition)

The maintainer **delegated this decision to the orchestrator (2026-07-02)**. The decision:
how to dispose of this audit — is the extension-checker portion of DN-54 verified complete
enough to unblock the dependent work, and how are the residuals bound?

**Option 1 — Accept the audit; route residuals per §5 (recommended).** The five M-812-cont
completions are verified landed at their claimed tags; M-918 proceeds on this audit as its
input; R-2/R-3 are bound into M-919's DoD (or one small minted task for R-3); R-4 rides the
certified-mode track; E-1 is a one-line changelog clarification. DN-54 stays `Accepted`;
Enacted remains gated exactly as DN-54 §9 states.

**Option 2 — Reopen M-812-cont now to close R-3 (proptest generation + DN-20 tiering) before
M-918.** Against (grounded): the generated-corpus differential's real value arrives when
`derive` sites elaborate — generating over nullary rules only buys thin coverage at the cost
of the repo's one serial lane (`mycelium-l1` — kickoff `grm` swarm method), and delays the
ratification-latency-hiding sequencing the kickoff prescribes (dossiers first). `Declared`
(a scheduling judgment, not a theorem).

**Option 3 — Declare the checker portion complete/Enacted-ready now.** Rejected on the
evidence: §4.4, §4.5, and §4.6 row 6 are obligations **inside DN-54's own text** and are
residual (§3); calling the checker "complete" would upgrade the claim past its basis (VR-5;
house rule #1). The audit's whole value is that this refusal is now grounded, not vibes.

**RECOMMENDATION: Option 1.** It is the only option that neither overclaims (Option 3) nor
spends the serial lane on a low-yield reopen (Option 2), and it leaves every residual bound
to a named DoD line or FLAG (G2). Recorded as **recommended, pending orchestrator acceptance**
under the maintainer's 2026-07-02 delegation — not self-ratified.

---

## §7 Guarantee posture (VR-5) + Definition of Done

- **`Empirical`** — every verdict in §3 (source read at the pinned tip `629aa12`; harness and
  unit suites re-run green, §2). The audit trusts the tree, not the changelog.
- **`Declared`** — the routing judgments in §5/§6 (where a residual is best bound; the Option-2
  scheduling argument). Asserted, flagged as such.
- **No upgrade** — each landed check is recorded at the tag the implementation itself claims
  (`Declared` structural checks; `Empirical` observational identity; narrow `Proven`-by-
  construction KC-3), each verified supportable against its checked basis.

**Definition of Done (M-917 / this note).** Met when: **(a)** the per-section audit table is
recorded `Empirical` with `file:line` evidence (§3 — done); **(b)** the §7 harness re-run is
green and recorded (§2 — done); **(c)** every residual is an explicit task or FLAG, none silent
(§5 — done); **(d)** the disposition recommendation is recorded for the orchestrator's
acceptance under the 2026-07-02 delegation (§6 — done; acceptance itself is the orchestrator's
act, then this note → Resolved, append-only). Enacts no code; moves no other doc's status.

---

## Meta — changelog

- **2026-07-02 — Created (Draft → recommended, pending orchestrator acceptance) — authored
  (M-917, kickoff `grm`).** Records the DN-54 completion audit of the `lwd`-landed M-812-cont
  checks: §2 harness re-run (5/5 + 18/18 green), §3 per-section audit table (`Empirical`,
  file:line at tip `629aa12`), §4 one editorial discrepancy (E-1), §5 residual ledger
  (R-1…R-4 + E-1, all routed), §6 options + the **Option 1** recommendation under the
  maintainer's 2026-07-02 delegation to the orchestrator. CHANGELOG / Doc-Index / issues.yaml
  / docs/api-index owned by the integrating parent (FLAGged, not edited). (Append-only; VR-5;
  G2.)
- **2026-07-02 — Resolved (recommended → Resolved).** Accepted by the wave orchestrator at the
  integration-reconcile promotion gate, under the maintainer's 2026-07-02 delegation (`Declared`).
  §6 **Option 1** adopted: the audit stands, DN-54 remains `Accepted`, and E-1 is carried as a
  one-line editorial clarification for DN-54's next append-only changelog entry. Forward transition,
  append-only (house rule #3); no tag upgraded past its basis (VR-5).
