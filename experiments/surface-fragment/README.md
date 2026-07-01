# Minimal surface-syntax fragment (M-020) — experiment-only

| Field | Value |
|---|---|
| **Task** | M-020 ([#4](https://github.com/tzervas/mycelium/issues/4)) · P0 · spec (throwaway) |
| **Status** | Ratified-for-experiment, 2026-06-09 |
| **Purpose** | a tiny concrete syntax sufficient to run the **KC-2** LLM-leverage experiment (M-002, #3) |
| **Maps to** | `SPECIFICATION.md` §10.1 |

> ⚠️ **Throwaway, not a committed surface — and KC-2 has since returned its verdict.** This
> grammar was written **only** to give the KC-2 experiment something to generate and measure,
> gated on KC-2 (Foundation §2.2, §2.4): if the experiment showed novel syntax hurts LLM leverage,
> it would be discarded in favour of projections or an embedded DSL (RR-3). **That gate is now
> resolved** — KC-2 returned **proceed**
> ([`DN-09-KC-2-Verdict.md`](../../docs/notes/DN-09-KC-2-Verdict.md), 2026-06-18), selecting the L3
> strategy (committed text syntax + a co-equal structured-projection layer). This document is
> preserved as the historical record of the throwaway fragment KC-2 actually ran against; it is
> still deliberately *not* under `docs/spec/` and carries **no** normative weight — the committed
> surface's authoritative definition, if it has since diverged from this fragment, lives under
> `docs/spec/grammar/` (RFC-0006/RFC-0007), not here.

It covers exactly the three things M-002 needs to exercise: **declare-value**, **swap** (the
never-silent representation change), and **one VSA op** (`bundle`). Everything desugars to the
Core IR node set (RFC-0001 §4.5: `Const | Var | Let | Op | Swap`).

## Grammar (EBNF)

```ebnf
program     ::= statement*
statement   ::= "let" ident (":" type)? "=" expr

type        ::= "Binary"  "{" "width" ":" nat "}"
              | "Ternary" "{" "trits" ":" nat "}"
              | "Dense"   "{" "dim" ":" nat "," "dtype" ":" scalar "}"
              | "VSA"     "{" "model" ":" ident "," "dim" ":" nat "," "sparsity" ":" sparsity "}"
sparsity    ::= "Dense" | "Sparse" "{" "max_active" ":" nat "}"
scalar      ::= "F16" | "BF16" | "F32" | "F64"

expr        ::= literal | ident | swap | call
swap        ::= "swap" "(" expr "," "to" ":" type "," "policy" ":" ident ")"
call        ::= ident "(" arglist? ")"
arglist     ::= arg ("," arg)*
arg         ::= expr | "[" (expr ("," expr)*)? "]"     (* list literal, e.g. bundle([...]) *)

literal     ::= binlit | tritlit | num
binlit      ::= "0b" ("0" | "1" | "_")+
tritlit     ::= "<" trit ("," trit)* ">"               (* MSB-first balanced trits *)
trit        ::= "+" | "0" | "-"
num         ::= "-"? digit+
ident       ::= (letter | "_") (letter | digit | "_" | "-")*
nat         ::= digit+
```

Notes: types are the four `Repr` kinds (matches [`repr.schema.json`](../../docs/spec/schemas/repr.schema.json)).
`swap` is the only representation-changing form and **must** carry a `policy` (WF1/WF2; RFC-0001 §4.5).
There is **no implicit conversion** between paradigms — mixing them in a `call` is a type error
(the surface mirrors the kernel rule, RFC-0001 §3.3).

## Desugaring to Core IR

| Surface | Core IR node |
|---|---|
| `let x : T = e` | `Let{ id: x, bound: ⟦e⟧, … }` with declared type `T` |
| `0b…` / `<…>` / `n` | `Const{ value: Value{repr, payload, meta=Exact} }` |
| `x` | `Var{ id: x }` |
| `swap(e, to: T, policy: P)` | `Swap{ src: ⟦e⟧, target: T, policy: P }` |
| `f(a, b)` / `bundle([…])` | `Op{ prim: f, args: [⟦a⟧, …] }` |

## Reference programs (the KC-2 gold set)

See `examples/`. Each is a tiny task with a known-correct form:

- [`examples/roundtrip.myc`](examples/roundtrip.myc) — declare a byte, swap to ternary and back
  (exercises declare-value + swap; the `LosslessWithinRange` path, M-012).
- [`examples/bundle.myc`](examples/bundle.myc) — declare three MAP-I hypervectors and `bundle` them
  (exercises declare-value + one VSA op; the `Proven` capacity bound, M-001).
- [`examples/type-error.myc`](examples/type-error.myc) — a **negative** example: adding a `Binary`
  to a `Ternary` with no `swap` → the type checker must reject it (the no-implicit-conversion rule).

## How M-002 (#3) used this

The KC-2 experiment (`experiments/mycelium_experiments/kc2/`) asked an LLM to produce programs in
**this** fragment vs. a Python-embedded-DSL baseline, and measured syntactic validity + type-check
pass rate + edit-to-fix iterations (Foundation §6 P0.2; SC-5b; G10). The two dependencies this
originally listed as open are both resolved: the fragment is scored by the real `myc-check`
(parse + typecheck + signature), and the local llama.cpp harness (`tools/llm-harness/`) supplied
the generation arm — see [`../README.md`](../README.md) and
[`../KC2-RUNBOOK.md`](../KC2-RUNBOOK.md) for the runnable end-to-end procedure, and
[`DN-09-KC-2-Verdict.md`](../../docs/notes/DN-09-KC-2-Verdict.md) for the measured run + verdict.

## Changelog

- **2026-06-30:** updated the framing to reflect the KC-2 verdict (DN-09, 2026-06-18, proceed) —
  the "gated on KC-2" language and the "still open" dependencies were stale; both are resolved.
  No grammar/desugaring changes.
- **2026-06-09:** initial throwaway fragment (grammar + desugaring + 3 reference programs) to unblock
  M-002. Not a committed surface; gated on KC-2.
