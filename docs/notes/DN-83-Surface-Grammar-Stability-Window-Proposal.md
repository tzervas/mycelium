# Design Note DN-83 — Surface-Grammar Stability Window (Proposal)

| Field | Value |
|---|---|
| **Note** | DN-83 |
| **Status** | **Proposed** (2026-07-02) — a proposal for the maintainer, not a ratified decision. Nothing in this note changes any RFC/ADR status or gates a commit; it recommends that the maintainer **Accept** a stability-window policy for the surface grammar going forward. |
| **Feeds** | RFC-0037 (Enacted — surface-grammar deconfliction/layout-independence); RFC-0030 (Enacted — the concrete L3 grammar as the accept/reject oracle); ADR-023 (Accepted — stability/API-compatibility guarantees at full-language 1.0.0, the model this note follows for the *pre-1.0* grammar surface). |
| **Date** | July 2, 2026 |
| **Decides (proposed)** | Whether the surface grammar (`docs/spec/grammar/mycelium.ebnf` + its generated editor artifacts) should enter a **stability window**: a period in which a change to the accepted/rejected surface-syntax set requires an RFC/ADR, rather than landing as an ordinary implementation PR. |
| **Task** | M-924 (grm stability close-out) |

> **Posture (transparency rule / VR-5 / G2).** The claim "the grammar is feature-complete for
> Phase-I" is `Empirical` — grounded in the current conformance corpus (44/44 clean-parse: 27
> `docs/spec/grammar/conformance/accept/*.myc` — including this note's own companion fixture
> `27-first-class-fn-type.myc` pinning the §7 `A => B` fix, §3 — + 17 `lib/std/*.myc`, verified
> against the current tip by this note's own verification run, §3) and the RFC-0037/RFC-0030
> Enacted chain — **not**
> `Proven` (there is no completeness theorem; a corpus is evidence, not exhaustive proof, and
> Phase-II VSA/dense-paradigm surface work is explicitly out of scope below). The proposal itself
> (a stability window + its trigger conditions) is `Declared` — a policy choice being recommended,
> not a measured fact. Every open question is flagged in §6, not silently assumed.

---

## 1. Problem / motivation

The surface grammar has been through **three consecutive supersession waves** in the current
phase: RFC-0025 (operator syntax), RFC-0030 (concrete L3 grammar), and RFC-0037 (bracket-family
deconfliction + layout independence + short repr-keywords + retired-glyph reallocation). Each of
those Enacted a real, user-visible surface change — retired `->`/`<=`/`>=`, moved type args to
`[…]`, added `lambda`, added `bin`/`tern`/`emb`/`hvec`. That churn was the *right* cost while the
grammar was still being decided (RFC-0006's Q3/Q8 open questions, the DN-31 bracket-kind-split
iteration).

That phase is now behind the frontend. As of the current `dev` tip (post M-915/M-916/M-919/M-921/
M-923 — DN-71 Model S, tuple types, fieldspec-fn lowering, operator wiring with `<=`/`>=` retired):

- Every FR/NFR-relevant surface construct named in RFC-0037's Decides field is landed:
  operators, generics `[…]`, effects `!{…}`, guarantee annotations `T @ Strength`, the short
  repr-keywords, and the retired-glyph deconfliction (`->`/`<=`/`>=` teaching-rejected, not
  silently accepted as aliases — verified in `crates/mycelium-l1/src/parse.rs::parse_type_ref_guarded`
  and `checkty.rs`'s operator-lowering table).
- The one known EBNF↔parser gap this task set out to close — the first-class function-type
  production `A => B` (RFC-0024 §3) — is now fixed (§7 below; this note's companion change to
  `docs/spec/grammar/mycelium.ebnf`).
- The editor-grammar drift gate (`just drift-check` / `scripts/checks/drift.sh`) reports the
  committed `tools/grammar/` artifacts current with the lexer, and the tree-sitter/tmLanguage
  self-test + corpus-parse checks are green (§3 — this note's own verification, `Empirical`).

Continued unreviewed churn on the surface grammar past this point has a real cost that the prior
phase's churn didn't: every landed `.myc` file in `lib/std/`, every conformance fixture, every
downstream editor-grammar consumer (VS Code, tree-sitter hosts, LSP semantic tokens — see
`tools/grammar/README.md`), and (looking forward) every user program written against the L3
surface becomes a migration liability on the next syntax change. RFC-0037's own Enacted-chain
history is itself the evidence: each surface revision forced a corpus-wide re-migration (`mycfmt`
reformat + fixture rewrite + editor-grammar regen) — cheap while the corpus was ~40 files and no
external users existed, increasingly expensive as both grow.

## 2. Proposal: a surface-grammar stability window

**Recommendation.** From this note's acceptance forward, a change to the *accepted or rejected*
surface syntax set — anything that would change what parses under `docs/spec/grammar/
mycelium.ebnf` (adding, removing, or altering a production; changing operator precedence/
associativity; changing which token set is reserved/active) — requires an RFC or ADR before
landing, exactly as RFC-0037/RFC-0030 themselves did. This is **not** a new mechanism: it is
naming the existing decision discipline (house rule #3, "append-only decisions") as the *specific*
gate for this *specific* artifact, the way ADR-023 named the same discipline for the full-language
1.0.0 API-compatibility promise.

**Explicitly NOT covered (no RFC/ADR needed):**

- **EBNF↔parser reconciliation** — closing a gap where the EBNF fails to describe syntax the
  parser already accepts (this task's own `A => B` fix, §7) is a *documentation correction*, not a
  grammar change: the parser's accepted-language set does not move. The oracle role stays with the
  corpus (RFC-0006 §4.3): if a correction ever *would* change corpus verdicts, that is a grammar
  change and falls under the gate above, not this carve-out.
- **Editor-grammar regeneration** (`tools/grammar/`) tracking a *landed* lexer/parser change is
  mechanical (`just grammar-gen` + `drift-check`) and carries no independent decision — the RFC/ADR
  that landed the underlying change is the gate; the regen is just keeping the derived artifact
  honest (G2).
- **Phase-II paradigm surface** (VSA/dense-specific syntax not yet designed) is out of scope for
  "the grammar is stable" — the window applies to the **landed Phase-I surface**, not to syntax
  that does not exist yet. A wholly new construct for an undesigned feature is ordinary RFC-gated
  design work, not a "stability window exception."

**Trigger conditions proposed for the window to apply** (any one is sufficient):

1. A change would make a currently-accepted `.myc` program a rejected one, or vice versa, for any
   file in the existing `docs/spec/grammar/conformance/accept/` corpus or `lib/std/`.
2. A change alters operator precedence, associativity, or the reserved/active status of a keyword.
3. A change adds a production interacting with an already-landed one in a way that could shadow or
   reinterpret existing syntax (e.g. a new bracket use inside `[…]`/`{…}`/`(…)`).

## 3. Verification performed for this note (grounding for §1's "feature-complete" claim)

Run against the current worktree tip (branched from `dev` @ `18fdabd`), after this task's own
`A => B` EBNF fix (§7) — commands and results, so the claim is checkable, not asserted:

```text
$ bash scripts/checks/grammar.sh
  ok    grammar artifact + corpus well-formed (27 accept, 30 reject; parser gate in cargo test)

$ bash scripts/checks/drift.sh
  ok    grammar generator self-test (extraction · determinism)
  ok    tools/grammar/ is current with the lexer keyword() table

$ python3 tools/grammar/generate.py --self-test
grammar generator self-test passed (extraction + coverage + determinism)

$ cd tools/grammar/tree-sitter-mycelium && npx tree-sitter-cli@0.25 generate
(no diff against committed src/grammar.json / src/node-types.json / src/parser.c)

$ npx tree-sitter-cli@0.25 parse -q ../../../lib/std/*.myc \
                                  ../../../docs/spec/grammar/conformance/accept/*.myc
(zero ERROR nodes; 17 lib/std + 27 accept-corpus = 44 files — one more than the language-tooling
 wave's prior 43/43 baseline, from this note's own new fixture, §7)

$ cargo test -p mycelium-l1 --test conformance
test accept_corpus_all_parses ... ok
test reject_corpus_all_fails_explicitly ... ok
test the_gate_is_non_vacuous ... ok
test reject_expected_table_has_no_orphaned_entries ... ok
```

This is `Empirical` evidence of *sync* (committed artifacts match a fresh regeneration; corpus
parses clean under both tree-sitter and the actual L1 `parse_phylum` oracle) — it is not a proof
that the grammar is complete for every future need, only that it is internally consistent and
matches the landed parser **today**.

## 4. Definition of Done (for this proposal, if Accepted)

- The maintainer records acceptance (or rejection, or a modified window) of §2's policy — this
  note's `Status` field moves `Proposed → Accepted` (never silently, never skipping — house rule
  #3).
- If accepted: `CONTRIBUTING.md` and/or this repo's PR template gain a short pointer to this note
  so a future contributor proposing a surface-grammar change is routed to the RFC/ADR path rather
  than discovering the gate mid-review. (Out of scope for this note's own diff — DN-83 is
  orchestrator/maintainer-owned for that follow-up, not a leaf edit under M-924.)
- The window's trigger conditions (§2) are precise enough that a reviewer can apply them without
  re-litigating intent per-PR; if practice reveals a condition is too broad/narrow, that is a
  supersession of this note, not a silent reinterpretation.

## 5. Alternatives considered

- **No window (status quo).** Keep treating every surface change as an ordinary implementation PR.
  Rejected as the default going forward: it worked while the grammar was actively being decided
  (RFC-0006's open questions), but nothing currently distinguishes "still deciding" from "decided
  and landed" for a reviewer or an external contributor — the same ambiguity ADR-023 §1 identifies
  for the full-language API promise, one layer down (syntax vs. semantics/API).
- **Full semver-style deprecation policy now** (mirroring ADR-023's dual-version/deprecation
  machinery in full). Considered and deferred: ADR-023 itself is scoped to full-language 1.0.0,
  which Phase-I has not reached; importing its full deprecation-cycle machinery for a pre-1.0
  grammar would over-formalize a much smaller surface (one EBNF file + generated editor grammars)
  relative to the whole-language compatibility promise ADR-023 governs. The lighter "RFC/ADR
  required, no formal deprecation cycle yet" window is proposed as the right-sized intermediate
  step; a future ADR can fold grammar stability into the full 1.0.0 promise when that milestone is
  actually approached.

## 6. Open questions / FLAGs (VR-5 — not silently assumed)

- **FLAG-83-1 (which artifact is the ADR/RFC gate).** This note frames the gate as "changes to
  `mycelium.ebnf`'s accepted/rejected language," but does not itself decide whether the *editor*
  grammars (`tools/grammar/`) ever need independent RFC gating when they diverge from a *landed*
  change (they shouldn't, per §2's carve-out — but that carve-out is this note's own recommendation,
  not yet ratified).
- **FLAG-83-2 (retroactivity).** This note does not propose retroactively re-gating RFC-0037/
  RFC-0030's own already-Enacted changes — only changes proposed *after* this note's acceptance.
  Whether the maintainer wants the window to also cover currently-in-flight but unlanded grammar
  proposals (if any exist outside this task's visibility) is unresolved; flagged, not assumed.
- **FLAG-83-3 (enforcement mechanism).** This note proposes the *policy* only. A machine-checked
  gate (e.g. a script asserting a grammar-touching PR references an RFC/ADR number) is not
  specified here — left as a follow-up decision if the maintainer wants enforcement beyond
  reviewer discipline.

## 7. Companion fix landed with this note (M-924, not itself a grammar-language change)

`docs/spec/grammar/mycelium.ebnf`'s `type_ref` production was missing the first-class
function-type case `A => B` that `crates/mycelium-l1/src/parse.rs::parse_type_ref_guarded`
already implements (right-associative; `@ strength` binds tighter than `=>`; `->` is a
teaching-reject per RFC-0037 D4, not a silent alias — verified against
`crates/mycelium-l1/src/tests/parse.rs::{simple_fn_type_parses_to_basetype_fn,
fn_type_is_right_associative, guarantee_binds_tighter_than_arrow, rfc_0024_map_snippet_parses}`).
This is exactly the §2 EBNF↔parser-reconciliation carve-out: the parser's accepted-language set
does not change, only the EBNF's description of it. `tools/grammar/README.md`'s note that this
production was "flagged as a pending `mycelium.ebnf` addition" is updated to reflect the fix. A new
positive conformance fixture, `docs/spec/grammar/conformance/accept/27-first-class-fn-type.myc`,
pins the fix in the corpus itself (house rule #2, "no black boxes" — the oracle stays the corpus,
not this note's prose): a bare `f: A => B` parameter, the right-associative chain `A => B => C`,
and `A @ Exact => B` (guarantee binds tighter than `=>`). `cargo test -p mycelium-l1 --test
conformance` is green with the new fixture included (§3).

---

## Changelog

- **2026-07-02 — Drafted/Proposed** (M-924, `grm` kickoff stability close-out). Proposes a
  surface-grammar stability window gated on RFC/ADR for future syntax changes, grounded in the
  RFC-0037/RFC-0030 Enacted chain and this note's own drift/corpus verification (§3). No RFC/ADR
  status changed by this note. Status: **Proposed**, pending maintainer decision.
