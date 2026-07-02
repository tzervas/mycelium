# Devlog — 2026-06-17 · Phase 9 Wave B: building the doc pipeline the design promised

> **What this is** (see [the Capture note](../../notes/Narrative-Capture-and-Authoring.md)): the
> *narrative* layer — the messy middle the RFCs smooth over. Append-only, informal, honest. The
> RFCs/ADRs/DNs remain the source of truth; this is the *story* of how it actually got built. Fittingly,
> this entry is itself ingestible by the pipeline it describes (a `devlog` document in the doc-IR).

**Theme.** Wave A ratified the design ([Narrative-Authoring-Pipeline](../../spec/Narrative-Authoring-Pipeline.md),
Accepted); the 2026-06-16 devlog deliberately *resisted building it*. Wave B is the inverse mandate:
**build M-363 for real** — the doc-IR generator + multi-format emitter, and flip the dormant §4.1 lint
to active — without guessing, without a half-build, and green-and-real like the Wave-A gates.

## 1. One IR, many renderers — the decision that paid off in code

The design's keystone (T7.6) is *one content-addressed doc-IR, and HTML/Typst/JSON are renderers of it*.
In code this discharged three house rules mechanically: identity is `blake3:<hex>` reusing the kernel's
`ContentHash` (ADR-003), so **dual-projection-parity** became a literal set-equality of the content
addresses scraped from the HTML view and the JSON view; **no-drift** is "same hash or the check fails";
and stable deep links fell out for free. The lint didn't need a fuzzy notion of "consistent" — it asks
whether two views carry exactly the model's node set.

## 2. "Undocumented is flagged, never invented" had to survive contact with real code

The apiref generator projects `.myc` nodule headers (M-359 `@summary`) and `fn` signatures. The honest
move was to make a missing summary an explicit `undocumented` node — not a blank, not filler. The lint's
`no-hallucinated-prose` check then *counts* those gaps as honest (26 across the corpus) and only errors
if an api-item ever carried prose without provenance. The invariant holds by construction (summaries are
only ever read from source), so the check is a guard, and the gaps are visible rather than papered over.

## 3. The two checks that nearly weren't green-and-real

**Checked examples.** The temptation was to mark every ` ```myc ` fence "checked". But a bare `fn` snippet
isn't a complete program — the L1 parser rightly demands a `nodule` header. So checked examples are *whole*
`.myc` files (the `examples/` projects that already pass `myc-check`) plus an opt-in ` ```myc-checked `
fence for complete programs in prose. Six examples type-check via the *same* `parse → check_nodule`
pipeline the gate uses — real teeth, and a unit test injects a non-compiling "checked" example to prove the
check fails when it should.

**No-dead-xref.** The trap was reddening the gate on every `README`/`research/` link. The resolution rule:
a link to an *ingested corpus* `.md` must resolve (or it's `Dead` → error); a link *outside* the generated
site is `OutOfScope` (links.sh owns external reachability); fragment-level anchoring is best-effort, stated
as such. Over the live corpus that's 18 internal xrefs resolving, 0 dead — and a hermetic test proves a
broken internal link does fail the gate.

## 4. What I deliberately did not do

No new Rust dependency (blake3/serde/serde_json were already vetted; Typst is an *external binary* the gate
skips when absent — no crate, so no ADR). No kernel change (KC-3 — the doc tooling sits above it and reuses
the trusted checker). EPUB is an *honest deferral* recorded in the build, not a half-rendered e-book. And I
resisted scoping the lint to a hand-picked happy-path corpus — it runs over all 97 documents.

## 5. Bonus: the sync engine stops throwing tracebacks

Secondary, bounded: the GitHub PM engine crashed with a raw `CalledProcessError` on a `gh api` 401. Now
every `gh` failure is a classified, remediation-bearing exit (re-auth / scope / rate-limit / network), and a
**least-privilege** preflight computes the *minimal* OAuth scope set from the arg'd ops and — with explicit
consent and an EXPLAIN — automates `gh auth refresh` (changing scopes is a state mutation; opt-in, never
silent — G2). One implementation in the engine; both wrappers route through it.

Refs: `crates/mycelium-doc/` (the pipeline + `myc-doc`); `scripts/checks/myc-doc.sh` (the gate);
[Narrative-Authoring-Pipeline](../../spec/Narrative-Authoring-Pipeline.md) (the contract); `tools/github/
gh-issues-sync.py` (the hardened engine); issue M-363 (#134). No kernel change (KC-3); no new dependency.
