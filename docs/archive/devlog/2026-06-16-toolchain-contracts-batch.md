# Devlog — 2026-06-16 · Contracting the rest of the toolchain (and lifting the §8 gate)

> **What this is** (see `docs/notes/Narrative-Capture-and-Authoring.md`): the *narrative* layer — the
> messy middle the RFCs smooth over. Append-only, informal, honest.

**Theme.** With the M-364 `mycfmt` contract presented, the maintainer chose **design all remaining
contracts first** (fold none) and **ratify the §8 build stack**. So this batch is four more design-first
contracts — M-365 (check driver), M-366 (lint+fix), M-367 (security), M-368 (spore) — plus the M-363 §8
ratification. The point of doing them as a batch is that the maintainer can ratify the *shape of the whole
suite* before a single line of tooling lands.

---

## 1. The §8 gate, lifted the honest way

M-363's build stack was a real dependency decision (mdBook vs Sphinx vs custom-IR+Pandoc/Typst), and the
spec parked it as the maintainer's gate. Ratified: custom in-repo doc-IR + Typst + static HTML. The
discipline note: I moved the spec Proposed → Accepted **append-only** — the status table shows the new
state, the §8 options stay verbatim, and the changelog footer records the transition. And I drew the line
the maintainer would want drawn anyway: **ratifying the design is not scheduling the build.** M-363's build
stays a separate, not-yet-scheduled task; what the ratification actually unblocks is M-366's §4.1 doc lint,
which can now be *specified* against a known IR.

## 2. The one rule every contract turned out to share

Writing four tools back-to-back, the same sentence kept reappearing in different costumes: **never a silent
pass.** It's the formatter's identity receipt (M-364); it's the check driver refusing to hide a
`NotValidated` (M-365); it's the lint tool's "a control-flow fix is a *scaffold*, never auto-applied"
(M-366, the RFC-0014 I1/I5 line); it's the security tool's "a missing scanner is *reduced coverage*, not a
clean bill" (M-367); it's the packager refusing a hashless dependency (M-368). G2 isn't a checkbox per
tool — it's the spine of the suite. The contracts are really five spellings of one promise.

## 3. The sharpest boundary: M-366's scaffold tier

The temptation in a "lint + auto-fix" tool is to make `--fix` smart. The honest design makes it *dumb on
purpose*: `--fix` only touches behaviour-preserving edits. Anything that changes control flow — inserting a
`swap`, adding an RFC-0014 recovery handler in response to the RFC-0015 §9 "this class is only logged"
advisory — is offered as a **scaffold**: an explicit, bounded skeleton with a `todo` body that the author
completes. A tool that silently inserted recovery would be precisely the implicit-control-flow change
RFC-0014 forbids (I1/I5; A2 of RFC-0015's honesty boundary). So the fix model has three tiers — suggest /
apply / scaffold — and the bright line is "would this change behaviour? then the human decides."

## 4. The packager's quiet insight (M-368)

ADR-003 makes the spore contract almost write itself: **identity is the content-addressed DAG; everything
in `[project]`/`[spore]` is metadata.** Two builds of the same code+deps yield the same spore hash no
matter the `version`/`authors`. The honest scope call was to ship a v0 that builds the DAG + its hash +
EXPLAIN now, and defer the wire-schema/signing/germination contract to the RFC-0008 R2 stages exactly where
ADR-013 §4 already parked them — rather than invent a schema this task isn't chartered to freeze.

## 5. What did *not* happen

No crate, no `cargo add`, no code. Five contracts, the §8 ratification, and the bookkeeping (Doc-Index +
CHANGELOG + this entry, append-only). Folding any of them is a clean next step against a frozen contract.

**Refs:** `docs/spec/{Myc-Check-Driver,Lint-and-Autofix,Security-Checks,Spore-Build-and-Publish}-Contract.md`
(Proposed); `docs/spec/Narrative-Authoring-Pipeline.md` (Accepted, §8 ratified); M-365 #137 · M-366 #138 ·
M-367 #139 · M-368 #140, epic M-361 #132.
