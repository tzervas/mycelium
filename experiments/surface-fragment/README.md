# Minimal surface-syntax fragment (M-020) — experiment-only

| Field | Value |
|---|---|
| **Task** | M-020 ([#4](https://github.com/tzervas/mycelium/issues/4)) · P0 · spec (throwaway) |
| **Status** | Ratified-for-experiment, 2026-06-09 |
| **Purpose** | a tiny concrete syntax sufficient to run the **KC-2** LLM-leverage experiment (M-002, #3) |
| **Maps to** | `SPECIFICATION.md` §10.1 |

> ⚠️ **Throwaway, not a committed surface.** This grammar exists **only** to give the KC-2
> experiment something to generate and measure. It is **gated on KC-2** (Foundation §2.2, §2.4): if
> the experiment says novel syntax hurts LLM leverage, this is discarded in favour of projections or
> an embedded DSL (RR-3). It is deliberately *not* under `docs/spec/` and carries **no** normative
> weight. The committed surface — if any — is a later decision.

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

## How M-002 (#3) will use this

The KC-2 experiment asks an LLM to produce programs in **this** fragment vs. a Python-embedded-DSL
baseline, and measures syntactic validity + type-check pass rate + edit-to-fix iterations
(Foundation §6 P0.2; SC-5b; G10). Two dependencies are still open and tracked there:

1. a **parser + type-checker** for this fragment to score "type-checks" (needs the Core IR /
   interpreter, M-101 #11 / M-110 #15); a syntactic-validity-only pass is possible from the grammar
   alone meanwhile;
2. **LLM API access** to run the generation arm.

## Changelog

- **2026-06-09:** initial throwaway fragment (grammar + desugaring + 3 reference programs) to unblock
  M-002. Not a committed surface; gated on KC-2.
