# Mycelium Tutorial — building a complete program

> **Status: Empirical/Declared.** Every code block in this tutorial is grounded in the
> parser-verified conformance corpus (`docs/spec/grammar/conformance/accept/`). The complete program
> built here is committed as `accept/20-tutorial-classifier.myc` and is parsed on every CI run by
> `crates/mycelium-l1/tests/conformance.rs` (`accept_corpus_all_parses`) — so the program you read is
> the program the parser accepts, never invented (G2 / VR-5). For construct-by-construct lookup, see
> the [language reference](./language-reference.md).

This tutorial builds a small but complete Mycelium program: a **signal classifier** that ingests
byte-sized sensor readings, classifies each, totals a run of readings, and converts a sample into a
ternary representation through a never-silent `swap`. Along the way you meet the surface you will use
in almost every program: nodules, algebraic types, pattern matching, bounded iteration, honest
guarantee tags, effect annotations, and the swap system.

---

## 1. A file is a nodule

Every Mycelium source file (`.myc`) opens with a `// nodule:` header comment naming its nodule, then
a `nodule` declaration:

```mycelium
// nodule: tutorial.classifier
nodule tutorial.classifier
```

The header comment is metadata (it is not part of the program's content-addressed identity); the
`nodule tutorial.classifier` line is the grammar's required program opener. A **nodule** is your
module — the basic unit of organization. Everything else goes underneath it.

> A nodule's items are *unordered for name resolution*: a function may call a sibling defined later
> in the file. You never reorder code to satisfy the compiler.

---

## 2. Model the data with algebraic types

A `type` is a **sum of constructors**. Our readings form a simple linked list of bytes, and a
classification is one of two cases:

```mycelium
type Reading = Empty | Cons(Binary{8}, Reading)

type Class = Low | High
```

- `Reading` is recursive: `Empty` is the end, `Cons(Binary{8}, Reading)` holds one 8-bit byte and
  the rest. `Binary{8}` is the representation type "8-bit binary value".
- `Class` has two nullary constructors, `Low` and `High`.

---

## 3. Classify a sample with `match`

Mycelium is expression-oriented: a function body is one expression. `match` scrutinises a value
against arms `pattern => result`:

```mycelium
fn classify(sample: Binary{8}) -> Class =
  match sample {
    0b00000000 => Low,
    _          => High,
  }
```

The first arm matches the literal all-zero byte (`0b00000000`); the wildcard `_` covers every other
byte. **Match exhaustiveness is checked** — if you forgot the `_`, the compiler would tell you the
match is not exhaustive rather than silently failing at runtime (G2).

---

## 4. Total a run of readings with `for`

`for` is the only iteration construct — a **bounded fold** over a linearly-recursive value, Total by
construction (no infinite loops). We fold the readings into a running 8-bit total:

```mycelium
fn total(rs: Reading) -> Binary{8} @ Declared =
  for r in rs, acc = 0b00000000 => add(acc, r)
```

Read the `for` as: *fold over `rs`, with accumulator `acc` starting at `0b00000000`, combining each
element via `add(acc, r)`.*

Notice the return type: `Binary{8} @ Declared`. The `@ Declared` is an **honest guarantee tag**. We
are *declaring* that the result is an 8-bit total — we have not *proven* an overflow bound, so we do
not write `@ Proven`. This is the honesty rule (VR-5) in action: you tag what you can defend, and
`Declared` is the honest default. (If a checked theorem later bounds the total, you may upgrade the
tag — never before.)

---

## 5. Convert representations with a never-silent `swap`

Suppose a downstream stage needs the sample in **balanced ternary**. You cannot just reinterpret the
bits — a representation change must be *explicit*. That is what `swap` is for:

```mycelium
fn ingest(sample: Binary{8}) -> Ternary{6} !{io} =
  let widened: Binary{8} = sample in
  swap(widened, to: Ternary{6}, policy: rt)
```

Three things are happening:

1. **`let widened: Binary{8} = sample in …`** binds a local, scoped over the body after `in`.
2. **`swap(widened, to: Ternary{6}, policy: rt)`** is the never-silent representation change. Both
   the target (`to: Ternary{6}`) and the conversion policy (`policy: rt`) are written out — you can
   *see* every representation change in the source. Omitting `policy` is a parse error: *"a swap is
   never silent."* An out-of-range conversion is an explicit error, never a silent truncation.
3. **`!{io}`** is an effect annotation: `ingest` declares that it performs the `io` effect. An
   unannotated function is pure; declaring an effect is a checked contract (a function's declared
   effects must cover the effects it actually performs).

---

## 6. The complete program

Putting it together — this is the verified `accept/20-tutorial-classifier.myc` in full:

```mycelium
// nodule: tutorial.classifier
nodule tutorial.classifier

type Reading = Empty | Cons(Binary{8}, Reading)

type Class = Low | High

fn classify(sample: Binary{8}) -> Class =
  match sample {
    0b00000000 => Low,
    _          => High,
  }

fn total(rs: Reading) -> Binary{8} @ Declared =
  for r in rs, acc = 0b00000000 => add(acc, r)

fn ingest(sample: Binary{8}) -> Ternary{6} !{io} =
  let widened: Binary{8} = sample in
  swap(widened, to: Ternary{6}, policy: rt)
```

In ~15 lines you have used: a nodule, two algebraic types (one recursive), pattern matching with
exhaustiveness checking, a Total `for` fold, an honest guarantee tag, an effect annotation, a
let-binding, and a never-silent swap. That is most of the everyday surface.

---

## 7. Where the honesty shows

Three details above are the heart of Mycelium, and worth re-reading:

- **`@ Declared`** — you never overclaim. Accuracy is tagged `Exact ⊐ Proven ⊐ Empirical ⊐
  Declared`, you downgrade to stay honest, and you cannot upgrade without a checked basis (VR-5).
- **`swap(…, to:…, policy:…)`** — you never convert silently. Every representation change is
  lexically visible, with its policy named and inspectable (S1 / G2).
- **`!{io}`** — you never hide a side effect. The effect set is declared and the checker enforces
  that declarations cover what is performed.

> **What is still staged (VR-5 honesty).** At this language version, generics and single-parameter
> traits now *run* — they monomorphize to a closed program and execute three-way (L1-eval ≡ L0-interp ≡
> AOT; M-673 done, width generics M-753, named-fn higher-order args via static defunctionalization
> M-687 / M-715). What stays *checker-only* or explicitly `Residual`: higher-order generics beyond a
> named-fn argument (closures, multi-arg arrows, partial application → M-704), multi-parameter traits /
> associated types, and effect annotations — *checker-only* metadata that do not yet wire to the runtime
> budget (→ M-677). See the [language reference §8](./language-reference.md#8-generics-and-traits) for
> the precise status of each.

---

## 8. Next steps

- **Look up any construct** in the [language reference](./language-reference.md).
- **Add a phylum.** Group several nodules into a library with a `phylum` header and export names with
  `pub` + `use` (reference [§1](./language-reference.md#1-files-nodules-and-phyla)).
- **Read real code.** `lib/std/result.myc` is the first self-hosted stdlib nodule — a generic
  `Result<A, E>` with `map` / `and_then` / `fold` combinators.
- **Explore the corpus.** `docs/spec/grammar/conformance/accept/` holds one focused program per
  construct — each is a parser-verified example.

---

## Changelog

- **2026-06-23 — Created (M-735).** Tutorial walkthrough building the complete, parser-verified
  `accept/20-tutorial-classifier.myc` program, for the full-language 1.0.0 documentation gate
  (E17-1). Every example is grounded in the accept conformance corpus (CI-parsed). Guarantee:
  `Declared`. Append-only.
