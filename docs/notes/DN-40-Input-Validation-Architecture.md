# Design Note DN-40 — Input-Validation Architecture (Only-Intended-Inputs Across the Stack)

| Field | Value |
|---|---|
| **Note** | DN-40 |
| **Status** | **Draft (advisory)** (2026-06-26; **awaiting maintainer ratification**) — a ratifiable **recommendation**, enacting nothing. Records the maintainer-commissioned **input-validation architecture review**: *devise a highly concurrent, high-throughput system for handling input validation across the stack so only intended inputs can occur*, and *consider where we do need input validation and other secure best practices*. Answers both halves: **(a) WHERE** validation is needed — a ranked **gap ledger** over the 5 audited boundaries / 10 finding clusters (§3); and **(b) the ARCHITECTURE** — one closed-grammar recognizer per boundary minting an immutable, canonical, bounded typed value, which is *simultaneously* the security mint and the lock-free concurrency fan-out (§4–§5). Motivated by the spore `content_address` injectivity finding (DN-39 §5 / PR #617, now **FIXED**) as the **reference pattern**. **Three actionable security items** lead the ledger: a **CRITICAL `Proven`** type-subgrammar stack-overflow DoS and a **HIGH `Proven`** pattern-subgrammar DoS in the L1 parser (un-guarded recursion, no `MAX_EXPR_DEPTH` charge → uncatchable `SIGABRT`), and a **HIGH `Proven`** parse-don't-validate gap on the identity-bearing dependency hash (free-text `String`, the existing `ContentHash::parse` smart constructor unused). Per-finding tags held at the finding's own honest basis (exhibited code gaps `Proven`; architecture + Mycelium-specific design `Declared`; prior-art mechanisms `Empirical`/`Proven`-at-source) — no severity inflated (VR-5). Append-only; house rule #3. Enacts no code, moves no decision status — it **records gaps and recommends fixes**; each fix is a separate forward decision the maintainer ratifies/directs. |
| **Feeds** | the **never-silent / no-black-boxes** house rules (#1/#2, G2) and **KC-3 small-auditable-kernel** (#5) at every external trust edge: the **L1 parser** (`crates/mycelium-l1/src/parse.rs`, the `myc-check` M-002 oracle whose own contract is *"return an explicit error, never crash"*, parse.rs:14-20), the **manifest reader** (`crates/mycelium-proj`), the **content-addressing / spore** chain (`crates/mycelium-spore`, ADR-003), the **registry store** (`crates/mycelium-spore/src/registry.rs`), the **value serde boundary** (`crates/mycelium-core`), and the **std-sys FFI/syscall floor** (`crates/mycelium-std-sys`, RFC-0028). Direct sibling to **DN-39** (the spore v1 length-prefix fix is the canonical reference here) and **DN-30** (security-scanning toolkit). The unbuilt **Value↔native host-encoding bridge** (RFC-0028 §4.4, epic **E14-1** / **M-722**) is named as the hardest, least-specified boundary — its deferred-encoding clause must mandate the discipline below **before** it ships. |
| **Date** | June 26, 2026 |
| **Decides** | *Nothing normatively* — advisory + recommendation capture for maintainer ratification. Records (1) **where** input validation is needed — the §3 **gap ledger** (the 3 actionable items + the mediums + the lows, each at its finding's own honest severity/status); (2) the **six secure-input principles** (§2 — parse-don't-validate · canonicalization/injective-encoding · bounds/resource-limits · allowlist-over-denylist · never-silent G2 · illegal-states-unrepresentable); (3) the **only-intended-inputs architecture** (§4 — one closed recognizer per boundary, typed-value mint, grounded in the existing model spots: `ContentHash`, the `// @key` header reader, the value serde re-parse); (4) the **concurrency & throughput** fit (§5 — validate-once-at-the-boundary then trust the immutable value ⇒ lock-free data-parallel fan-out, no TOCTOU, full validation at memory bandwidth); (5) the **prior art** it cites (§6); (6) the **generalizable principle** (§7); and (7) the **open-question ledger** (§8) + honest per-claim guarantee posture (§9). |
| **Task** | Input-validation architecture review (maintainer-commissioned, 2026-06-26; advisory — task #34; enacts nothing) |

> **Posture (transparency rule / VR-5 / G2).** This note records an **advisory architecture review**:
> *devise a highly concurrent, high-throughput input-validation system so only intended inputs can occur*,
> and *consider where validation and secure best-practices are needed*. It **enacts nothing** — no code,
> no property test, and no RFC/ADR/DN status moves *from this note*; each fix it names is a separate,
> forward-only decision the maintainer ratifies/directs. The grounding split is load-bearing and held
> throughout: the **exhibited code gaps** (§3) are `Proven` — each tied to a `file:line` the survey read
> (the type-subgrammar overflow is reported PoC'd to `SIGABRT`; the pattern-subgrammar case is `Proven`
> **by structure**, the same overflow shape but **not separately PoC'd** — stated as such, not inflated);
> the **prior-art mechanisms** (§6) are `Empirical` (netstrings/simdjson — measured/observed) or
> `Proven`-at-source (the LANGSEC parser-equivalence undecidability theorem; data-race-freedom under
> immutability at the memory-model level); the **Mycelium-specific architecture** (§4–§5) and the
> **generalizable principle** (§7) are `Declared` — asserted design directions for ratification, **not**
> proven properties of the current code. Severities are **not** inflated (§9): exploitable-now =
> critical/high, real-but-bounded = medium, defense-in-depth = low, each gated by its real existing gate
> and labelled. The stack's many model spots are reported as genuine strengths, not damned with faint
> praise. Assent is on independently-verified merit, not deference (VR-5 applied to agreement, house
> rule #4).

---

## §1 Purpose & honest scope

The maintainer commissioned a review with two halves, both answered here:

1. **Devise** a *highly concurrent, high-throughput* input-validation system *so only intended inputs can
   occur* — the **architecture** (§4) and its **concurrency/throughput** fit (§5).
2. **Consider where** input validation and other secure best-practices are needed — the **gap ledger**
   (§3), a ranked, file:line-grounded survey of the 5 audited external boundaries and the 10 finding
   clusters within them.

The review is **motivated by, and patterned on, the spore `content_address` injectivity finding** (DN-39
§5; **FIXED** in PR #617 — the v0 unescaped delimited pre-image moved to a **v1 length-prefixed
netstring-style** encoding, injective by construction, with an adversarial property test). That fix is the
**reference pattern** this note generalizes: *recognize exactly the intended language, mint an unforgeable
typed value, encode it injectively.* The headline finding is honest about where the stack already stands:
the content-addressing / registry / CLI boundaries are **largely model-grade**, but **two `Proven`
stack-overflow DoS gaps** in the L1 parser's type/pattern subgrammars and a **parse-don't-validate gap on
the identity-bearing dependency hash** are real, and **must land before the FFI host-encoding bridge** (the
hardest boundary) is built.

This note **enacts nothing**. It is a recommendation the maintainer ratifies; each fix it names is a
separate, forward-only decision/issue, **never** a silent change of code or status from this note.

---

## §2 The six secure-input principles

The architecture rests on six principles, each tagged at its honest basis (§6 sources the externals):

- **P1 — Parse, don't validate** *(`Declared` maxim; the underlying total-function/refinement-type
  reasoning is `Proven` in type theory)*. A *parser* consumes less-structured input and produces a
  more-structured **refined type** that carries the proof forward; a *validator* checks and throws the
  evidence away, forcing every downstream caller to re-check or assume. Get data into its most precise
  representation **at the boundary**, before it is acted upon. The refined value is trustworthy **only if
  its constructor is the sole gate** — an escape hatch silently reintroduces *shotgun parsing*.
- **P2 — Canonicalization / injective encoding** *(`Declared`/spec-level; the bug class is `Empirical`)*.
  One wire form per value: length-prefix or escape every variable-length field so no delimiter can forge a
  boundary; reject anything outside the encoder's image (`decode(x)` must fail unless
  `encode(decode(x)) == x` byte-for-byte). This is exactly the spore v0→v1 fix (DN-39 §5) and the netstring
  rule (Bernstein). **Injectivity of the encoding is necessary but NOT sufficient** — it faithfully commits
  whatever it is handed (see the `blake3:abc` finding, §3).
- **P3 — Bounds / resource limits** *(`Declared`)*. Every input-driven recursion charges a shared depth
  budget and refuses past the limit with a **named error, never a host-stack abort**; every input-driven
  allocation/read is **capped before the alloc**. The parser is both the model (`parse_expr`/`parse_unary`
  charge `MAX_EXPR_DEPTH`) and the cautionary tale (the type/pattern subgrammars do **not** — §3).
- **P4 — Allowlist over denylist** *(`Declared`, with `Empirical` support for the failure mode; OWASP)*.
  Define exactly what **is** authorized; everything else is unauthorized by definition. A recognizer for a
  closed grammar **is** an allowlist by construction; a denylist is a shotgun parser enumerating the
  infinite complement. Includes **closed-key discipline** (duplicate-key rejection — the `// @key` header
  reader is the model).
- **P5 — Never-silent (G2)** *(house rule #1/#2)*. Out-of-range / malformed input is an explicit
  `Option`/`Result` **naming the offending input** — never a silent default, truncation, or guess. An
  *uncatchable abort is the most complete violation of never-silent there is* (the aborting parser breaks
  its own *"never crash"* contract more severely than any silent default).
- **P6 — Illegal-states-unrepresentable** *(`Declared` maxim; Minsky)*. Choose the data model so invalid
  states have **no representation** — a newtype with a private field + smart constructor, `Option<T>` not
  `Option<String>`, a sum type over the closed provenance. Necessary precondition for P1 ("parse into a
  type that can't be wrong" only works if such a type exists).

These reinforce each other: a **recognizer for a closed grammar** (P4) that **mints a refined,
unrepresentable-if-illegal type** (P1/P6) via the **sole constructor** (P6), **canonicalizes** its output
(P2), **bounds** every recursion/alloc (P3), and **never silently admits** the complement (P5).

---

## §3 The gap ledger — WHERE validation is needed (ranked)

> **Actionable security items lead.** The three rows at the top are the exploitable-now / identity-bearing
> gaps; fix them **first**. The mediums are real-but-bounded; the lows are defense-in-depth, each gated by
> an existing real gate (named inline). Severities are the findings' **own** honest tags — `Proven`
> where exhibited at a `file:line`, **not** inflated.

| # | Site (`file:line`) | Principle | Severity (status) | Fix direction |
|---|---|---|---|---|
| **A1** | `crates/mycelium-l1/src/parse.rs:685-700` `parse_type_ref` (`->` RHS) + `816-823` `parse_type_args_opt` (nested `<…>`) + `716-771` `parse_base_type` | **P3, P5/G2** | **CRITICAL** (`Proven` — un-guarded recursion exhibited at named lines; survey reports a working PoC) | The type subgrammar recurses on the host stack with **no charge against `MAX_EXPR_DEPTH`** (256, parse.rs:20), unlike `parse_expr`/`parse_unary` which **do** charge it; a ~20k-deep `A -> A -> … -> A` overflows the 8 MiB main-thread stack → **`SIGABRT`** — an **uncatchable process abort**, reached from `myc check`/`myc test` and the `myc-check`/`mycfmt`/`myc-lint`/`myc-sec` bins on attacker-supplied `.myc` source. Directly violates the module's own *"return an explicit error, never crash"* contract (parse.rs:14-20, A4-02/B2-01). **Fix:** charge the shared `self.depth` budget (increment-on-entry / compare to `MAX_EXPR_DEPTH` / refuse with the explicit `ParseError` / decrement-on-exit) in `parse_type_ref` + `parse_base_type`'s recursive descent; add a regression mirroring `deep_operator_nesting_is_refused_not_crashed` for nested `->` and nested `<…>`. **Fix this before anything else in the ledger.** |
| **A2** | `crates/mycelium-l1/src/parse.rs:1125-1146` `parse_pattern` (line `1134` nested ctor sub-patterns via `comma_separated(Self::parse_pattern)`) | **P3, P5/G2** | **HIGH** (`Proven` — un-guarded recursion exhibited at parse.rs:1134; same overflow shape, **no separate PoC**) | Same class as A1: `parse_pattern` recurses for nested constructor patterns `C(C(C(…)))` with no `MAX_EXPR_DEPTH` charge (`parse_match` charges depth once for the match expression, but the pattern nesting is independent host recursion). Structurally identical to the proven arrow case, reached from the same `parse` entry — `Proven` **by structure**, not by a separate PoC. **Fix:** route `parse_pattern` through the same depth-charge guard (one shared helper covers A1+A2); regression for deep nested ctor patterns; land **together** with A1. |
| **A3** | `crates/mycelium-proj/src/manifest.rs:332` `build_dependencies` (`hash = Some(as_str(v,"hash",line)?)`) + `Dependency.hash: Option<String>` (manifest.rs:93); consumed at `crates/mycelium-spore/src/lib.rs:235` `content_address`; **`ContentHash::parse` exists unused** at `crates/mycelium-core/src/id.rs:15` | **P1, P6, P5** | **HIGH** (`Proven` — fixture/test at `tests/manifest.rs:24,89` + `tests/fixtures/mycelium-proj.toml:18` accept `blake3:abc`; consequence bounded — the bogus pin still fails to resolve later, so not critical) | Parse-don't-validate **inverted on an identity-bearing field**: the dependency `hash` — the authoritative content-address pin (ADR-003), an input to spore DAG identity — is parsed only as a raw TOML `String` via `as_str` and **never checked against the `blake3:<hex>` shape**. `ContentHash::parse`/`from_parts` (id.rs:15-41) **do** enforce the `<algo>:<digest>` grammar but the manifest **does not call them**; `Dependency.hash` is `Option<String>`, so an illegal pin is **representable**. The v1 netstring encoder then *faithfully/injectively* commits the garbage pin — **injectivity of the ENCODING does not give validity of the INPUT** (the spore-finding class moved one boundary upstream). **Fix:** parse the dep hash into `mycelium_core::ContentHash` at the boundary (`ContentHash::parse(h).ok_or_else(\|\| ManifestError naming the offending value)`); change `Dependency.hash`/`ResolvedDep.hash` to `ContentHash` (mirrors `SourceFile.hash: ContentHash` at spore/lib.rs:32) so a malformed pin is unrepresentable downstream; parse at manifest, re-assert at publish. |
| M1 | `crates/mycelium-std-sys/src/io.rs:35-39` `read_to_end()` + `src/fs.rs:18-50` read/write/exists/create_dir_all/remove_file | **P3; P2/P4 (fs)** | MEDIUM (`Proven` — unbounded reads + bare-`&Path` passthrough exhibited; blast radius limited, no production `wild:` op wired) | `io::read_to_end` does an unbounded, input-driven `Vec` growth (hostile stdin → memory-exhaustion DoS); the fs floor passes `&Path` straight to `std::fs` with **no canonicalization, no allowlist/root confinement, no read cap** — nothing stops `../../etc/passwd` traversal, symlink escape, or an arbitrarily large read once reached from a `wild { }` block. **Fix:** add `read_to_end_capped(max)` (`stdin().take(max)` → explicit `Err(TooLarge{cap})`, never silent truncation); for fs, when wiring the bridge add explicit **root-confinement** (canonicalize-then-verify-prefix, reject traversal/symlink-escape with a named `Err`) decided as an **allowlist of intended roots**, plus a max-read cap. Do not let the bare passthrough be the boundary. |
| M2 | RFC-0028 §4.4 (`docs/rfcs/RFC-0028-FFI-and-System-Interface.md:174-177`) + `crates/mycelium-std-sys` + `mycelium-std-sys-host` (**no production `wild:` op exists** — only `wild:echo` test fixtures) | **P1/P6, P2, P3** | MEDIUM (`Declared` — a spec/architecture gap, not exhibited-exploitable code; the most external boundary is the least specified) | The actual foreign-data parse boundary (`Value`↔native host `PrimFn` bridge) **does not exist yet**; §4.4 explicitly **defers** it (*"v0 does not impose a canonical value-to-C encoding… the host `PrimFn` owns the `Value`↔native conversion"*). The danger: the *"deferred, no canonical encoding"* clause ships as the **permanent** answer — the exact ambiguity the spore v1 fix closed, re-opened at the most external trust edge. **Fix:** before **E14-1** lands, write its spec to **mandate** (1) every native→`Value` decode is a total `fn(&[native]) -> Result<Value, EvalError>` naming the offending bytes (P1/P5); (2) every `Value`→native encode is injective/length-prefixed (P2 — copy `push_field`'s netstring discipline); (3) buffer/length/pointer-shaped returns are bounds-checked against an explicit cap before alloc/copy (P3). |
| M3 | `crates/mycelium-proj/src/manifest.rs:183-208` table dispatch + `365-415` `build_project` (and `build_dependencies`/`build_toolchain`/`build_surface`/`build_spore`) — **no duplicate-key detection** | **P5/G2, P4** | MEDIUM (`Proven` — overwrite-without-check loops exhibited; author-local manifest, not a remote edge, so not high) | The manifest reader has **no duplicate-key detection**: `build_project` iterates and the **last** assignment for a repeated key silently wins; a repeated `[project]` header merely re-points `current`. So `name`/`kind`/`hash`/`certification` can be set twice with the second **silently shadowing** the first — asymmetric with the header parser, which **does** reject duplicates (`header.rs:152-159` `seen` set). Two conflicting `hash=` lines resolve to one without flagging — a never-silent violation at the identity-feeding layer. **Fix:** mirror the header parser's `seen` set; reject a duplicate key (and duplicate table) with a `ManifestError` naming the key and both line numbers. |
| M4 | `crates/mycelium-spore/src/lib.rs:267-297` `walk` (source collection) — symlink-following recursive walk, no depth/count/byte cap | **P3** | MEDIUM (`Proven` — symlink-following `is_dir()` recursion with no cap exhibited; bounded — build target usually operator-chosen) | The recursive source walk uses `Path::is_dir()` (stats the symlink **target**, not the link) and recurses with **no depth cap**, so a symlinked directory cycle (`a -> b -> a`) drives unbounded mutual recursion → stack overflow; a deep acyclic tree overflows too. No file-count/total-bytes cap (each `.myc` read whole via `std::fs::read` at line 283). Attacker-influenced in the build-untrusted-source supply-chain case. **Fix:** skip symlinked dir entries (`symlink_metadata`; don't recurse into symlinks) **or** canonicalize + track a visited real-path set; add explicit depth + file-count + total-bytes caps, each a named `SporeError`; property-test a symlink cycle and a deep tree. (cli-common `walk_into` has the same shape, lower exposure — low priority.) |
| M5 | `crates/mycelium-core/src/repr.rs:88-103` `well_formed` + `value.rs:174-182` `payload_matches` — `Repr` width/trits/dim `u32` with only `> 0`, **no upper bound** | **P3** | LOW-MEDIUM (`Proven` — `> 0`-only check exhibited; latent — present-payload requirement currently bounds allocation) | `Repr` width/trits/dim are `u32` and `well_formed()` enforces only `> 0`, **no upper bound**; a crafted `Repr` can declare dim/width up to `u32::MAX` on the serde path. Today serde_json bounds the practical alloc by input size (the matching payload must be present), capping the blast radius. **Latent risk:** any downstream that allocates proportional to `dim` **before** checking the payload over-allocates. **Fix:** add an explicit, documented upper bound on width/trits/dim in `well_formed` (or `Value::new`/the `Deserialize` impl), a `WfError` naming the offending dimension — a finite ceiling, not unbounded `u32`. (The deser path is otherwise exemplary: `deny_unknown_fields` + re-run `Value::new` + char-by-char bit/trit validation + a non-panic fuzz target.) |
| L1 | `crates/mycelium-spore/src/registry.rs:383-395` `parse_entry` (index read-back) + `340-352` `published_versions` | **P5/G2, G11** | LOW (`Proven` — last-wins/silent-ignore arms exhibited; **gated** by the resolve re-hash at registry.rs:288) | `parse_entry` is lenient three ways: (a) duplicate `spore_id`/`artifact` lines silently take the **last**; (b) any unrecognised line is silently ignored by `_ => {}`; (c) it returns `Option`, so the caller gets a generic *"malformed index entry"* that does **not** name the offending field (contrast `parse_ref` in std-content, which diagnoses the exact rule, G11). `published_versions` silently drops non-UTF8 filenames. **Gated:** resolve **re-hashes** the fetched object against the recorded artifact, so a forged entry can mislabel identity but **cannot** serve tampered bytes. **Fix (defense-in-depth):** make `parse_entry` total-and-strict — reject a repeated key (Conflict/Integrity, not last-wins), reject lines that are neither blank nor a known `key=value`, require exactly the two expected keys, return a typed reason naming the offending field (mirror `parse_ref`'s G11 dual projection). |
| L2 | `manifest.rs:517-634` `scan_value`/`scan_array`/`scan_inline_table` (mutual recursion); `core/src/id.rs:26-31` unbounded digest charset; `registry.rs:273,198` `read_to_string`/`read` no size cap; `std-sys/src/sys.rs:48-52` `args()` `filter_map` drop; `l1/src/lexer.rs:339-352` `lex_binary` empty `0b` | **P3, P5/P2/P1** | LOW cluster (`Proven` for each exhibited site; defense-in-depth / honesty nits, gated as noted) | (1) Manifest value scanning recurses through nested arrays/inline-tables with no depth/count cap — **gated** low because the v0 reader is single-line-only (depth bounded by line length). (2) `ContentHash::parse` accepts an unbounded-length digest; registry resolve/publish read index/object files with no size cap — **gated** by the local store + artifact re-hash. (3) `sys.args()` `filter_map` silently **drops** a non-Unicode OS arg, re-indexing the rest (mild P5/P2; honestly documented). (4) `lex_binary` emits an empty-payload `BinLit` for a bare `0b` rather than a lex-time error (minor P5/P1; validated downstream). **Fix:** (1) explicit recursion-depth param + element-count caps on `scan_value`; (2) cap digest length (e.g. reject > 512 chars) + a max-index/object-size guard before `read_to_string`/`read`; (3) offer `args_lossy() -> Vec<Result<String, OsString>>` so position is preserved + the non-Unicode case is explicit per-slot; (4) reject an empty `0b` at the lexer. All defense-in-depth; document the chosen bound as the contract. |

**Reference row (FIXED — the pattern to copy, not a gap).** The spore `content_address` v0→v1
length-prefixed encoding (DN-39 §5, PR #617): the canonical example of P2 done right — every variable-length
field spans exactly its byte count, no embedded delimiter can forge a boundary, a single canonical encoder
(DRY — the verify path cannot drift), an adversarial injectivity property test. **A3 is exactly the lesson
this fix did *not* close:** the encoder is now injective, but the *input* it commits is still unvalidated.

---

## §4 The architecture — only-intended-inputs across the stack *(Declared)*

**The spine — one closed recognizer per boundary, producing a typed value; then trust the value.** Each
external trust edge gets a single **total** parse function `untrusted -> Result<Typed, NamedError>` that:
(a) recognizes a **closed grammar / allowlist** (P4/P6 — accept exactly the intended language, reject its
complement); (b) emits the **refined type that carries the proof forward** (P1); and (c) is the **sole
construction path** for that type (P3/P6 — a newtype with a private field + smart constructor; illegal
states unrepresentable). This is LANGSEC's *full recognition before processing* (keep each grammar low in
the Chomsky hierarchy so it stays decidable — §6).

**Mycelium already exhibits this at model grade** — the architecture is *"make every boundary look like the
model spots"*, not a rewrite:

- **`ContentHash`** (`core/id.rs:9`) — newtype, **private** `String`, only `parse()`/`from_parts()` gate
  construction, `Deserialize` re-runs `parse()`. The canonical smart constructor; a `ContentHash` literally
  cannot contain a delimiter, so every downstream path/registry encoding is safe **by construction**.
  **(The A3 fix is just routing the dep hash through this existing gate.)**
- **The `// @key` header reader** (`proj/header.rs`) — closed `HEADER_KEYS`, **duplicate-key rejection**,
  each value parsed-into-a-type (semver / ISO-date / SPDX-over-allowlist / cert-mode). The template the
  manifest reader (M3) should copy.
- **`nodule` dotted-name + `parse_cert_mode`** (`l1/nodule.rs`, `proj/cert_scope.rs`) — segment-by-segment
  identifier validation, closed cert-mode match, sum-typed provenance.
- **`Value`/`Payload`/`ContentHash` `Deserialize`** (`core/value.rs`) — `deny_unknown_fields` + re-run
  `Value::new`, which closes the **P5 deserialization back-door**: a typed value reconstituted from bytes
  **re-enters the recognizer**, never a trusting blob-load. Backed by a non-panic fuzz target.

**Canonicalization discipline (P2) — the spore v1 fix is the reference; generalize it.** Every
content-addressed / concatenated pre-image MUST be **injective**: length-prefix or escape every
variable-length field. `push_field` (`spore/lib.rs:245` — `<len>:<bytes>\n` + per-section counts) and the
kernel `Canon` hasher (`core/content.rs:98-145` — length-prefixed blobs + one-byte domain-separation tags +
counts before every variadic) are the two reference encoders; std-spore `recompute_identity` **delegates**
to the single encoder (DRY — the verify path cannot drift). **Hard rule for ratification:** any new
content-addressed pre-image copies this length-prefix + count + domain-tag discipline, with a property test
asserting round-trip canonicality (`decode(x)` fails unless `encode(decode(x)) == x` byte-for-byte). **Critical
caveat (already finding A3):** injectivity of the ENCODING is **not** validity of the INPUT — canonicalization
and the upstream type-parse are **separate, both-required** obligations.

**Bounds at every recursive / allocating edge (P3) — the parser is the cautionary tale.** `parse_expr`/
`parse_unary` correctly charge `MAX_EXPR_DEPTH` and refuse with an explicit error (the model);
`parse_type_ref`/`parse_type_args_opt`/`parse_pattern` do **not** and overflow → `SIGABRT` (the gap, A1/A2).
The architecture mandates: every input-driven recursion charges a **shared** depth budget and returns a
named error past the limit (never a host-stack abort); every input-driven alloc/read is **capped before the
alloc** (digest length, read sizes, `Repr` dim/width, file-count/bytes on the build walk); every filesystem
descent either refuses symlinks or tracks a visited real-path set. The contract is the parser's own stated
one (parse.rs:14-20): *"must return an explicit error, never crash."*

**Never-silent (G2) end to end (P5).** Out-of-range / malformed input is an explicit `Option`/`Result`
naming the offending input — never a silent default/truncation/guess. The model spots set the bar
(`rand.fill_bytes` fill-all-or-`Err` with no zero-tail; `OsClock` checked `i128::try_from` with no silent
wrap; `OsEntropy` **no** fallback fill; the CLI closed-subcommand match → `usage()`; `init` refusing a bad
name without normalizing); the deviations (M3 duplicate-key last-wins, L1 `parse_entry` silent-ignore arms,
L2 `args()` drop / empty `0b`, the A1/A2 aborts) are exactly the never-silent debt to retire.

**Net:** the stack is **~70% already at the target pattern**. The architecture is (1) close the proven
parser-DoS gaps (A1/A2), (2) route the dependency hash through the existing `ContentHash` smart constructor
(A3), (3) **specify the unbuilt FFI host-encoding bridge to the same parse-into-typed + injective-encode +
bounded discipline BEFORE it is built** (M2), and (4) retire the never-silent debt. Tag every boundary's
guarantee per-op on the `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` lattice (VR-5).

---

## §5 Concurrency & throughput — the maintainer's emphasis *(Declared design fit; T2 backing Proven-at-memory-model level)*

**Validate-once-at-the-boundary, then trust the value — and value-semantics makes that concurrency-sound
for free.** The boundary recognizer is the **synchronization point**. Once the parse produces an
**immutable refined value** (P1/P5 — the typed value **is** a capability; holding it is proof the invariant
was checked), it carries its invariant intrinsically and needs **no further checking**, so it can be read
concurrently by N hyphae with **zero locks and zero re-validation**. Validation is a property of the
**value**, not a moment in a thread's execution — so it is neither repeated per consumer nor invalidatable
by another thread. Mycelium's core model is value-semantics + content-addressed immutable values, so this is
essentially free.

**This CLOSES the TOCTOU window.** If validated data were mutable + shared, a second thread could mutate it
*after* the check and *before* use (classic check-then-use re-admission of invalid state); immutable
value-semantics makes the check-to-use window **unbreakable** — there is no *"after the check"* mutation
possible (data-race-freedom is the memory-model precondition, `Proven` for Rust/C++/Java-class models). The
same immutability makes **lock-free data-parallel validation** sound: each input is independent and
immutable, so a **colony of hyphae** can parse/validate a **stream** of inputs in parallel with **zero
coordination** (embarrassingly parallel over independent inputs), and the resulting typed values fan out
lock-free.

**Throughput is NOT in tension with assurance — simdjson is the existence proof** (`Empirical`, peer-reviewed,
VLDB 2019): the first **standard-compliant** JSON parser to hit **GB/s** does **full** UTF-8 + full-spec
recognition at >2 GB/s single-core (~half the instructions of RapidJSON) by **fusing validation into
recognition** in two stages — SIMD/data-parallel structural indexing → typed-tape build — with **branchless**
inner loops. Two transferable lessons: (1) full validation can be made roughly **memory-bandwidth-bound** —
do **not** treat *complete recognition* as a speed tax to defer to a `certified` mode; (2) branchless /
data-parallel classification removes input-controlled branches, which is **both** a speed win **and** a
worst-case-input hardening bonus (fewer mispredict-DoS and timing-side-channel surfaces). **Honest caveat:**
the GB/s figures are hardware/corpus-specific to a 2019 Skylake; a Mycelium grammar would need its own
engineering to approach them — cite simdjson as proof that bandwidth-bound full validation is **possible**,
**not** as a guaranteed number.

**Where a validation layer must NOT become a serial bottleneck.** (a) The boundary must be the **only** gate
but **not** a single shared mutex — keep it a **pure total function over an immutable input** so it
parallelizes per-input. (b) **Streaming + bounded**: read-with-a-cap, never read-to-end-then-validate (the
M1/M4 unbounded reads are exactly the anti-pattern — an unbounded buffering stage serializes throughput on
the slowest/largest input **and** is a DoS). (c) The P3 depth/size caps **double as throughput guards**: a
bounded recognizer has a bounded per-input cost, so a hostile input cannot stall the colony.

**Two honest limits on *"validate once, trust everywhere"*** (both never-silent failure points to enforce,
not assume): it holds **only** across the **same** language/sink — a value safe as a Mycelium typed value is
**not** automatically safe re-serialized into a **different** downstream sink (SQL / shell / HTML / the native
FFI ABI), each of which is its own recognizer/encoder needing its own boundary (OWASP defense-in-depth, P6) —
**precisely why** the unbuilt `Value`↔native bridge (M2) needs its **own** injective encoder; and the
lock-free guarantee depends on **true** immutability — interior mutability / unsafe aliasing / a
deserialization back-door that reconstructs a typed value without re-parsing would silently break it (the
core deser path already guards this by re-running `Value::new`; the FFI bridge must too).

---

## §6 Prior art

| Principle | Source | Tag |
|---|---|---|
| **Parse, don't validate** — refined type at the boundary, the constructor is the sole gate | Alexis King, *"Parse, don't validate"* (2019) | `Declared` maxim (type-theory core `Proven`) |
| **LANGSEC — full recognition before processing**; input is a formal language; the **parser-equivalence undecidability** result (decidable only for regular / deterministic-CF) | Sassaman/Patterson/Bratus/Locasto, *Security Applications of Formal Language Theory* (IEEE Sys. J. 2013); *From shotgun parsers to more secure stacks* (2013) | recognizer principle `Declared`; **the equivalence theorem `Proven`-at-source** |
| **Make illegal states unrepresentable** | Y. Minsky, *Effective ML* (Jane Street, 2010) | `Declared` maxim |
| **Canonicalization / injective length-prefixed encoding** (leading zeros prohibited); the **non-canonical "quasi-encoding" acceptance** bug class | D. J. Bernstein, *Netstrings* (1997); RLP-in-ACL2 (arXiv:2009.13769) | spec-level `Declared`; **bug class `Empirical`** |
| **Capability-based security + parse-at-the-boundary** (the typed value is the capability; deserialization is the classic leak) | POLA / object-capability tradition | `Declared` |
| **Allowlist > denylist; syntactic + semantic** validation; defense-in-depth at each downstream sink | OWASP Input Validation Cheat Sheet / Proactive Controls C5/C3 | `Declared` (failure mode `Empirical`) |
| **simdjson** — full standard-compliant validation at GB/s; SIMD/branchless data-parallel; existence proof that full validation is **not** a throughput tax | Langdale & Lemire, *Parsing Gigabytes of JSON per Second* (VLDB J. 28(6), 2019) | **`Empirical`** (measured, peer-reviewed; hardware/corpus-specific) |
| **TOCTOU-freedom under immutability** — validate-once then share lock-free; no check-then-use window | Memory-model data-race-freedom (Rust/C++/Java-class); immutable/STM validation literature | **`Proven`-at-the-memory-model level** |

Full source URLs are carried in the review record; the load-bearing externals above are the ones the
architecture (§4) and concurrency fit (§5) depend on.

---

## §7 The generalizable principle *(Declared)*

> **Parse untrusted input into a typed, canonical, bounded value at a single closed-grammar recognizer per
> boundary — then the proof lives in the value, not in scattered downstream checks.**

The lesson the spore v1 fix taught (DN-39 §5), lifted to a stack-wide discipline:

1. A boundary either **PARSES into a type** whose smart constructor is the sole gate (illegal states
   unrepresentable) or it is a **shotgun parser** that re-admits invalid state downstream — there is **no
   middle *"string-checked-and-passed"*** (A3 is exactly that middle, and the proof of why it is unsafe).
2. An **injective / length-prefixed encoding is necessary but NOT sufficient** — it faithfully commits
   whatever it is handed, so it must sit **downstream** of an input-validating parse, never substitute for
   one (the `blake3:abc` pin proves the two obligations are independent).
3. Every input-driven recursion / allocation is **bounded with an explicit named error**, because an
   **uncatchable abort is the most complete violation of never-silent there is** (the parser that aborts
   breaks its own *"never crash"* contract more severely than any silent default — A1/A2).
4. Because the resulting value is an **immutable value-semantics capability**, validate-once-at-the-boundary
   is **simultaneously** the security architecture (single authority mint, no TOCTOU) **and** the concurrency
   architecture (lock-free data-parallel fan-out to a colony, full validation at memory bandwidth) —
   **assurance and throughput are the same design, not a trade.**

*"Only intended inputs can occur"* is achieved by **recognizing exactly the intended language** (allowlist /
closed grammar) and **minting an unforgeable typed value** for it — **never by enumerating the bad.**

---

## §8 Open-question ledger

- **OQ-1 (sequencing — the load-bearing one).** Land the parser DoS guards (A1/A2) **and** the dep-hash
  parse (A3) **before** the FFI host-encoding bridge (M2 / E14-1 / M-722 §4.4) is built. The §4.4 *"deferred,
  no canonical encoding"* clause must **not** ship as the permanent answer. **Who owns ratifying the
  replacement, and does the encoding spec block E14-1?** Should the `Value`↔native encoding be specified
  **now** (mandating total native→`Value` parse + injective `Value`→native encode + bounded buffers) as a
  **precondition** for the bridge landing, or is a separate DN the right vehicle?
- **OQ-2 (a shared boundary-parser toolkit?).** Is a single shared **depth-charge helper** (used by
  `parse_expr`/`parse_unary` **and** the to-be-fixed `parse_type_ref`/`parse_base_type`/`parse_pattern`) the
  right refactor — and more broadly, should the stack grow a **reusable boundary-parser toolkit** (the
  smart-constructor + closed-grammar + length-prefix + bounded-recursion discipline as shared scaffolding)?
  Should the downstream totality/type/elaborator passes **also** carry an independent depth guard, or is
  bounding the parser (hence AST depth) provably sufficient to protect them transitively, as parse.rs:14-20
  claims?
- **OQ-3 (`ContentHash` strictness).** Should `blake3:` digests be required to be **exactly 64 lowercase-hex**
  (vs the current permissive `[A-Za-z0-9_-]+` of any length)? Tighter = catches malformed pins earlier +
  bounds filename length; looser = forward-compatible with other algos. Decide the per-algo digest grammar.
- **OQ-4 (manifest duplicate-key policy).** Reject **all** duplicates strictly (matching `header.rs` and
  conformant TOML) — confirmed as the intended G2 posture (M3)? Any legitimate use of repeated keys/tables
  this would break?
- **OQ-5 (recognizer-equivalence).** If/when a `fast` parser and a `certified` parser ever exist for the
  **same** wire format, parser-equivalence is decidable **only** if the format is regular / deterministic-CF
  (LANGSEC, `Proven`-at-source). Should the policy be a **single generated recognizer** shared by both modes
  (so equivalence is trivial), and does any current format risk exceeding deterministic-context-free?
- **OQ-6 (throughput measurement).** simdjson proves bandwidth-bound full validation is possible, but a
  Mycelium grammar needs its own engineering. Is a **throughput benchmark corpus** (cf. the MEM-4 Q5 corpus
  precedent) warranted for the boundary recognizers, or premature in the design phase?
- **OQ-7 (build-walk symlink policy).** Refuse symlinked dirs outright, or follow-with-visited-set (M4)?
  Refusing is simpler/safer; following supports legitimate symlinked vendoring. Which is intended for the
  build-untrusted-source case?

---

## §9 Guarantee posture & house-rules note

**ENACTS NOTHING.** This is an advisory architecture-review **recommendation** the maintainer ratifies. It
**records gaps and recommends fixes** — it changes no code, no spec, and no decision status. **Append-only:**
nothing here supersedes or rewrites any ADR/RFC/DN; each fix it names (A1/A2/A3, the mediums, the lows, the
M2 FFI spec mandate) is a **separate, forward-only** decision/issue the maintainer ratifies/directs, and the
referenced future work (E14-1 / M-722 §4.4, RFC-0028) stays at its current status until separately ratified.

**Per-claim guarantee tags (transparency rule / lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`; VR-5 — no
tag upgraded past its basis):**

- **`Proven` (exhibited code gaps).** The parser stack-overflow gaps (A1 type — parse.rs:685-700/816-823, **PoC'd
  to `SIGABRT`**; A2 pattern — parse.rs:1125-1146, the same overflow shape but **`Proven`-by-structure, NOT
  separately PoC'd** — its *"exploitable"* claim is stated as structural, not as a witness); the dependency-hash
  parse-don't-validate gap (A3 — fixture/test at `tests/manifest.rs:24,89` accepting `blake3:abc`); the manifest
  duplicate-key last-wins (M3); the unbounded reads (M1, io.rs/fs.rs); the symlink-following build walk (M4); the
  `Repr` `> 0`-only bound (M5); and the `parse_entry` leniency arms (L1/L2) — **each tied to a `file:line` the
  survey read.**
- **`Empirical` / `Proven`-at-source (prior-art mechanisms — not this review's measurements).**
  netstrings / canonical-injective-encoding (Bernstein; the quasi-encoding bug class `Empirical`); LANGSEC
  full-recognition + the **parser-equivalence undecidability theorem** (`Proven`-at-source); simdjson GB/s
  full-validation (`Empirical`, peer-reviewed, hardware/corpus-specific — an **existence proof, not a
  guaranteed number**); TOCTOU / data-race-freedom under immutability (`Proven` at the memory-model level).
  Parse-don't-validate and allowlist>denylist are `Declared` design maxims (the type-theory is `Proven`, the
  maxim is a discipline).
- **`Declared` (this review's Mycelium-specific design synthesis).** The §4 architecture, the §5
  concurrency/throughput fit to value-semantics, and the §7 generalizable principle — **asserted design
  directions for ratification, NOT proven properties of the current code.**

**Honesty hold-the-line (severities NOT inflated).** Exploitable-now = critical/high (A1/A2 parser gaps, A3
identity-bearing hash). Real-but-bounded = medium (M1/M4 unbounded reads/walk not yet FFI-wired; M3
duplicate-key shadowing on an author-local file; M2 the unbuilt-but-under-specified FFI bridge). Defense-in-depth
= low (L1 `parse_entry` strictness, L2 digest-length cap / `args()` drop / `lex_binary` empty `0b` / manifest
recursion cap) — **each gated by an existing real gate** (the artifact re-hash, the single-line reader,
present-payload allocation bound) and labelled as such. The stack's many model spots are reported as **genuine
strengths**, not damned with faint praise; the one over-promise flagged honestly is `std-sys/lib.rs:1` naming
itself an *"audited FFI/syscall floor"* while the hard `Value`↔native bridge is unbuilt.

**Definition of Done.** The Draft → Accepted gate: the maintainer ratifies (a) the **six secure-input
principles** (§2); (b) the **gap ledger's prioritization** (§3 — A1/A2/A3 lead, fixed first, before E14-1);
(c) the **only-intended-inputs architecture** (§4) and its **concurrency/throughput fit** (§5); and (d) the
**generalizable principle** (§7). Accepting the *recommendation* neither enacts code nor upgrades any guarantee
tag past its stated basis (VR-5); each named fix is opened as its own forward issue. CHANGELOG / Doc-Index /
issues.yaml / docs/api-index owned by the integrating parent.

---

## Meta — changelog

- **2026-06-26 — Created (Draft, advisory) — authored.** Records the maintainer-commissioned **input-validation
  architecture review** (task #34): *devise a highly concurrent, high-throughput input-validation system so only
  intended inputs can occur*, and *consider where validation + secure best-practices are needed*. Answers both
  halves — **WHERE** (the §3 gap ledger over 5 boundaries / 10 finding clusters) and the **ARCHITECTURE** (§4 one
  closed recognizer per boundary minting an immutable/canonical/bounded typed value; §5 the lock-free
  data-parallel, no-TOCTOU, memory-bandwidth concurrency fit). Motivated by + patterned on the spore
  `content_address` injectivity fix (DN-39 §5 / PR #617) as the reference. Captures (§2) the **six secure-input
  principles** (P1 parse-don't-validate · P2 canonicalization/injective-encoding · P3 bounds/resource-limits · P4
  allowlist-over-denylist · P5 never-silent G2 · P6 illegal-states-unrepresentable); (§3) the ranked **gap
  ledger** with **3 actionable security items leading** — **A1 CRITICAL `Proven`** type-subgrammar stack-overflow
  DoS (`parse.rs:685-700/816-823`, no `MAX_EXPR_DEPTH` charge → `SIGABRT`, PoC'd), **A2 HIGH `Proven`-by-structure**
  pattern-subgrammar DoS (`parse.rs:1125-1146`, same shape, not separately PoC'd), **A3 HIGH `Proven`**
  parse-don't-validate gap on the identity-bearing dependency hash (`manifest.rs:332` free-text `String`;
  `ContentHash::parse` unused at `id.rs:15`) — plus the mediums (M1 unbounded stdin/no path confinement; M2 the
  unbuilt-and-under-specified FFI §4.4 bridge; M3 manifest duplicate-key silent last-wins; M4 symlink-cycle source
  walk; M5 `Repr` dimension no upper bound) and the lows (L1 registry `parse_entry` leniency; L2 manifest scan
  recursion / digest charset / `args()` drop / `lex_binary` empty `0b`); (§4) the **architecture** grounded in the
  existing model spots (`ContentHash`, the `// @key` header reader, the value serde re-parse) + the spore-v1
  canonicalization discipline + the parser depth-guard generalized; (§5) the **concurrency/throughput** synthesis
  (validate-once-then-trust the immutable value; simdjson as the bandwidth-bound full-validation existence proof,
  honestly caveated; the two honest limits — same-sink-only + true-immutability); (§6) the **prior art** (King;
  LANGSEC; Minsky; Bernstein netstrings; OWASP; simdjson) at `Empirical`/`Proven`-at-source; (§7) the
  **generalizable principle**; (§8) the **open-question ledger** (sequencing A1/A2/A3 before E14-1; a shared
  boundary-parser toolkit; `ContentHash` strictness; manifest duplicate-key policy; recognizer-equivalence;
  throughput measurement; symlink policy); and (§9) the **honest per-claim guarantee posture** (exhibited code
  gaps `Proven` — A2 stated `Proven`-by-structure, not PoC'd; architecture + Mycelium design `Declared`; prior-art
  mechanisms `Empirical`/`Proven`-at-source; severities **not** inflated, each low gated by its real gate). DoD =
  the Draft → Accepted gate (maintainer ratifies the principles + the ledger prioritization + the architecture +
  the principle; each named fix opened as its own forward issue). **Enacts nothing; moves no status; changes no
  normative text.** CHANGELOG / Doc-Index / issues.yaml / docs/api-index owned by the integrating parent.
  (Append-only; VR-5; G2.)
