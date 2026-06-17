# Devlog — 2026-06-17 · Folding `myc-lint` — when the honest auto-fix is *no* auto-fix

> **What this is** (see `docs/notes/Narrative-Capture-and-Authoring.md`): the *narrative* layer — the
> messy middle the RFCs smooth over. Append-only, informal, honest.

**Theme.** Fifth and final tool of the M-361 suite folded: `myc-lint` (`crates/mycelium-lint`). The
contract was deliberately held at Proposed "for first-implementation confirmation" of two things — the
safe-edit boundary and the doc-lint dormancy. Folding it *is* that confirmation, and the answer to the
first one is the interesting part.

---

## 1. The first-impl finding: there is no safe auto-fix (yet)

"Lint + **auto-fix**" sets an expectation: `--fix` rewrites your code. But when I went through the M-141
lints one by one asking "is there a *behaviour-preserving* edit here?", the answer was no, every time:

- `implicit-swap` → inserting a `swap` **changes control flow** → scaffold, not apply.
- `placeholder-policy` → can't fabricate a real PolicyRef → suggest.
- `unverified-bound` → can't fabricate a verification → suggest.
- `free-variable` → a real bug; the fix is a human decision → suggest.
- `nodule-header` → a malformed value is never invented (VR-5) → suggest; and *canonicalizing* a
  well-formed header is already `mycfmt`'s job.

So v0 maps every fix to **suggest** or **scaffold**, and `--fix` honestly applies **nothing** — and prints
exactly that. A "lint+fix" tool whose `--fix` rewrites zero lines sounds like a failure; it's the opposite.
The never-silent rule (G2) means the tool would rather do nothing than silently change your meaning. The
*value* is in the suggest/scaffold output — especially the scaffolds, which hand you the explicit `swap`
skeleton or the bounded recovery handler to complete yourself.

## 2. The scaffold is where RFC-0014's invariants live

The RFC-0015 §9 advisory ("this class is only logged — add a handler?") becomes, in `myc-lint`, a
**recovery scaffold**: an explicit `handle <class> with resilient { retry(max=3) … }` skeleton, generated
from the M-362 `RecoveryProfile`. It is *always* a scaffold — never applied — because adding recovery is
the author's declared, bounded, opt-in choice (RFC-0014 I1/I5), never the tool's. I shipped the scaffold
*generator* even though the lint that would trigger it is deferred (L1 has no effect-declaration surface
yet) — so the actionable capability exists ahead of the trigger, named honestly.

## 3. Dormant-but-defined, not faked

The §4.1 doc quality-bar lint (the 8 checks) is *named* — `DOC_QUALITY_CHECKS` lists all eight — but it
consumes the M-363 doc IR, which isn't built. So it ships **dormant**: `doc_lint_status()` says so, and it
doesn't block the gate. Defined, not pretended-active. This is the §8 ratification cashing out: the doc
lint *could* be specified once the build stack was chosen, and now it's a named placeholder waiting for its
input.

## 4. The suite is complete

With `myc-lint` folded, all five M-361 children are code: `mycfmt`, `spore`, `myc-check`, `myc-sec`,
`myc-lint`. Five tools, five crates, each above the kernel (KC-3), **zero new external dependencies** across
the whole suite, every contract Accepted/enacted. The through-line that held from the contracts into the
code: **never a silent pass.** `mycfmt` refuses rather than change identity; `myc-check` refuses the empty
pass; `spore` refuses the unpinned dep; `myc-sec` refuses to call reduced coverage a clean bill; `myc-lint`
refuses to auto-rewrite. The honesty rule isn't a checkbox per tool — it's the spine of the toolchain.

**Refs:** `crates/mycelium-lint/{src/lib.rs,src/bin/myc-lint.rs}`; `docs/spec/Lint-and-Autofix-Contract.md`
(Accepted/enacted); M-366 (#138); the suite epic M-361 (#132).
