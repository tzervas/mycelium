# Example — the seamless-lowering gradient (a layered HTTPS config fetcher)

**Status: ILLUSTRATIVE design-phase source — NOT runnable.** Mycelium is in the design phase; there
is no executor for surface `.myc` yet. This example exists to show *idiomatic, lexicon-correct*
Mycelium and to **teach one idea**, not to be compiled or run. Treat it as documentation.

File: [`https-downloader-layered.myc`](./https-downloader-layered.myc)

This is the **sibling** of [`binary64-https-downloader.myc`](./binary64-https-downloader.myc) (its
[README](./README-binary64-https-downloader.md)). The first example is the *security-focused*
downloader (a target-surface demo). This one reframes the same HTTPS-fetch shape as a **typical
general-programming task** — fetch a release/config manifest over HTTPS, parse it into a typed
`Config`, integrity-check it, use it — in order to showcase a different thesis.

## The one idea it teaches — the seamless gradient (DN-38 §1)

Mycelium is **one language**, not a stack of dialects. The L0–L3 "levels" name **where the compiler
lowers**, not modes the *programmer declares*. A single program **freely intermixes** high-sugar,
high-ergonomics surface with explicit lower-level forms — because they are the **same program at
different points on the desugaring gradient**, sharing one L0 substrate (DN-38 §1; RFC-0006 §3: "L2
is defined entirely by elaboration to L1"). The low form simply *is* the high form less-sugared.

The example makes that gradient **visible inside one file**, in four kinds of move:

| Move | Where | What it shows |
|---|---|---|
| **── sugared surface (L2/L3) ──** | Part A | `derive` for boilerplate (`Eq, Show, Deserialize for Config`), terse `and_then` pipeline, `type` records + sum types, trait `impl`s, **`via` dependency-injection** |
| **── drop to explicit control (toward L1/L0) ──** | Part B | a hard-capped byte read written as an explicit bounded `for`-fold (precise allocation/bounds control); canonical length-prefixed encoding of identity-bearing bytes (the spore-v1 lesson); explicit `!{io}` at the IO edge; `wild { }` only at the FFI boundary — **each drop says WHY in a comment** |
| **── opt out of the sugar (hand-roll) ──** | Part B (`parse_channel`) | the developer **bypasses the generative layer entirely** and hand-writes the parser — *not* because they must drop low, but because they want a **custom invariant** (`unknown ⇒ Stable`, fail-safe) the `derive`d form would not give. The gradient lets you decline the sugar in the same program. |
| **── the gradient made visible (`reveal`) ──** | Part C | `reveal { … }` showing the terse `for`-fold lowers to a concrete L0 `Fix`/`Match` term (DN-38 §5, abstracted-never-hidden) — the same kind of term you would hand-write |

The point each move teaches the reader: **drop low where direct control makes sense; opt out of the
sugar where you want a non-standard shape; and `reveal` proves the sugared and explicit forms are
the SAME L0 program.** It is not segregated dialects.

### The DI / testability angle

`via` injects a `Deps` record (a `TlsPolicy`, a `Clock`, a `Budget`) through a trait the fetcher
delegates to. That is Dependency-Injection done ergonomically — and it is what makes the fetch
**testable**: swap `Deps` for a stub and the whole pipeline runs with no network. `via` is **static,
by-value forwarding to a held value** — a *conduit*, not an agentive late-binding delegate (DN-38
§8.1: the keyword names the conduit precisely *because* there is no late binding / no chain walk).

## Per-construct surface-status legend (the honesty contract — read this)

The first example took a review ding for calling a Draft *direction* "Accepted." This file does
**not** repeat that: every construct is tagged inline (`// [enacted]` / `// [proposed:DN-31]` /
`// [designed:DN-3x]`) and nothing beyond the `[enacted]` rows is claimed as landed grammar.

| Tier | Tag | Constructs used here | Status — honest |
|---|---|---|---|
| **Enacted (landed today)** | `[enacted]` | `type` sum/record decls, `Binary{N}` **curly** sizing, `<A>` **angle-bracket** generics, `->` return arrow, `match`, `if/then/else`, `trait`/`impl`, `!{io}` effect annotation, `for … , acc = … => …` bounded iteration, `wild { }`, `and_then`/Result bind | Parses against today's `mycelium.ebnf`. **Caveats:** `trait`/`impl` **type-check only** — dictionary-passing L0 elaboration is **STAGED → M-673** (they do not yet RUN); `!{io}` is **checker-only** (M-660, no L0 node, does not run). So those carry a **`Declared`** posture, not a runtime claim. |
| **Proposed (DN-31 direction, Draft/advisory)** | `[proposed:DN-31]` | `[]` for type args / sized types, `=>` as the return arrow, `0t`/`0b` literal sigils, `<>` freed for operators | The DN-31 **direction** — decided-as-direction, **NOT landed** (epic #27 is the binding act, still open). This file uses the **enacted** `<A>`/`{N}`/`->` forms for its spine and only **names** the DN-31 direction in comments — it does **not** silently adopt it. **Not `Accepted` as grammar.** |
| **Designed (greenfield, not built)** | `[designed:DN-37]` / `[designed:DN-38]` | `via` delegation / DI, `derive` generative lowering, `reveal` inspector, super-traits, default method bodies, dynamic dispatch | Per DN-37 §2.2 / DN-38 §6 these are **`Declared` design proposals** with **no surface form in the parser today** (absence verified in those notes). Shown to illustrate the gradient; their presence is **not** a claim they parse. The §8.1 naming (`via` / `derive` / `reveal`) is the ratified *direction*, not landed syntax. |

**Assumed host/std surface (illustrative, not a claim it exists):** `platform_tls.connect`,
`http_collect`, `http_get_capped`'s transport, `parse_toml`, `blake3`/`bytes_eq_ct`, `env_get`,
`be64_bytes`, `++` concat, `session_chunks`/`chunk_len`. These are shown to make the example
concrete; they are not a claim that each exists in `lib/std` today.

## Guarantee posture (VR-5 / G2)

Per-operation and never upgraded past its basis:

- **Exact** — total operations over a finite domain: the `Channel`/`Config` field selections and
  pattern matches, the hand-rolled `parse_channel` (a total selection with a catch-all), the
  finite-spine `for`-fold's *termination*, the canonical length-prefix transform, the `Binary{64}`
  length equality.
- **Declared** — effectful / delegated / host-backed contracts: `fetch`, `http_get_capped`, the
  `via`-delegated TLS handshake and constant-time compare, the `!{io}` edges, `read_credential`'s
  `wild` host read, the `derive`d `Deserialize`, and the cap-predicate rule in `checked_step`. Also
  the `trait`/`impl` paths (elab STAGED → M-673) and `!{io}` (checker-only) carry a `Declared`
  posture because they **type-check but do not run** today.
- **Proven** — **nothing.** No operation here has a theorem with checked side-conditions (VR-5).

Never-silent throughout (G2): every partial step is an `Option`/`Result`; HTTPS-only is an explicit
refusal, the cap aborts mid-stream, an etag mismatch is an explicit `Err`, a missing credential is
an explicit `Err`, and a malformed manifest never half-parses.

## Cross-references

- **DN-38 — Layered-Lowering Atlas** (`docs/notes/DN-38-Layered-Lowering-Atlas.md`): the
  seamless-gradient thesis (§1), the lowering law (§2), the level-independent honesty markers (§3),
  generative lowering / `derive` (§4), the `reveal` inspector (§5), the per-feature Lowering Map
  (§6), and the `via`/`derive`/`reveal` naming (§8.1). **This example is a worked illustration of
  that note.**
- **DN-37 — Object & Behavior Model** (`docs/notes/DN-37-Object-and-Behavior-Model.md`): `type` as
  the single data keyword, traits/`impl`s, the inheritance-emulation menu, **`via` delegation (§3.3)**,
  and the sigil scheme (§5).
- **DN-36 — Safe & High-Performance Iteration** (`docs/notes/DN-36-...md`): the two-tier iteration
  surface and the "`for`-fold and a hand-written `Fix` fold are two points on one gradient, both
  lowering to one tail-recursive `Fix`" principle (§2.1) that Part C's `reveal` makes literal.
- **Sibling example** — [`binary64-https-downloader.myc`](./binary64-https-downloader.myc) /
  [its README](./README-binary64-https-downloader.md): the security-focused first downloader this
  one is adapted from.
