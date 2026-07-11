# DN-113 — The Cross-Phylum (Crate→Crate) Import / Resolution Subsystem

| Field | Value |
|---|---|
| **Note** | DN-113 |
| **Status** | **Accepted** (2026-07-10, delegated ratification — see the dated "Ratification (maintainer-delegated, orchestrator-selected on the merits, 2026-07-10)" section below). **Accepted ratifies the v1 DESIGN only, NOT Enacted** (house rule #3: `Enacted` requires stepping through `Accepted` first and means *fully implemented/landed*; this note has no implementation to land — M-1060 tracks that). Originally **Draft** (2026-07-10) — a design-reasoner PLAN in the plan → review → improve → ratify → implement cycle. |
| **Grounding basis** | `Empirical` where read against the codebase at `dev` **`45927ea4`** (commands and file:line cited inline). The proposed design is **`Declared`** until implemented and differential-witnessed (VR-5) — ratifying the design does not itself build or witness it. This is genuinely green-field: **0 % is wired today** (§1 audit). Where the design is undetermined it is flagged as an open question, never guessed (G2/VR-5). |
| **Author** | design-reasoner (Opus). Owns only this note. |

---

## §0 The question, in one line

A Mycelium **nodule** can already `use` a `pub` symbol from **another nodule of the same phylum**
(M-662/M-1024, real fixtures). It **cannot** reference a symbol in a **dependency phylum** — the
crate→crate boundary. This note designs that subsystem: the **reference syntax**, the **dependency-
graph (`Phyla`) model**, **content-addressed version-pinned resolution** (ADR-003), **separate
compilation + re-exports**, and how it **layers over the one canonical linker** (`PhylumEnv::link`,
M-1024) rather than becoming a third parallel linker. It also supplies the cross-phylum def-site
resolution that DN-110 §8.2 **OQ-H1** left open (`docs/notes/DN-110-8.2-hygiene-deepdive.md` §10).

---

## §1 Ground truth — the empirical import audit (`Empirical`, `dev 45927ea4`)

Verified against the codebase before designing (mitigation #14 — the tracker is `Declared`, the
codebase is ground truth):

| # | Fact | Basis |
|---|---|---|
| 1 | **Project detection works.** `mycelium-proj.toml` parses `[project] kind=phylum\|program\|script`, `[surface].exports`, `[dependencies]`, `[toolchain]`, `[spore]`. Only `[project]` is fully typed+validated in v0; the others are typed but their *consumers* were staged. | `crates/mycelium-proj/src/manifest.rs:20-121` |
| 2 | **Intra-phylum cross-nodule `use` works.** `use a.b.Item` / `use a.b.*` → `resolve_imports` against the phylum-wide `Exports` (pub-only, keyed by **qualified name**) → `PhylumEnv::link` folds every nodule's *checked home decl* into one `Env` (**M-1024**, the canonical L1 linker). Never-silent on unknown/private/ambiguous. | `checkty.rs:1086` (`link`), `:1211` (`Exports`), `:1231` (`NoduleImports`), `:1327` (`check_phylum`) |
| 3 | **Cross-phylum does not exist.** `Manifest::dependencies: Vec<Dependency>` parses (`Dependency { name, phylum, version: Option<String>, hash: Option<ContentHash> }`, `manifest.rs:84-97`), **but nothing in `mycelium-l1` / `mycelium-cli` / `mycelium-check` ever reads `manifest.dependencies`.** Only `mycelium-spore::build_spore` reads deps — as **packaging metadata** (records the resolved edges; never loads their exports, never resolves against them). | grep: `manifest.dependencies` consumed only in `mycelium-spore/src/lib.rs:167-196` |
| 4 | **No dependency-graph type.** `ast::Phylum { path: Option<Path>, nodules: Vec<Nodule> }` is **one** phylum; `check_phylum` takes **one** `&Phylum`. There is no `Phyla` / dep-graph / multi-phylum type. | `ast.rs:16-22`; `checkty.rs:1327` |
| 5 | **No cross-phylum `use` syntax.** `UsePath { path: Path, glob: bool }` names an **intra-phylum** dotted path only. No phylum-qualifier anywhere in the grammar or corpus. Only `DN-110-8.2` mentions "cross-phylum" — and only as the OQ-H1 open question. | `ast.rs:63-70`; grep `cross-phylum` → `DN-110-8.2` only |
| 6 | **The dep pin is already identity-bearing + typed.** `Dependency.hash: Option<ContentHash>` is **parsed-not-validated** (DN-40 A3): a malformed pin is an explicit `ManifestError` at manifest-build time, a well-formed `ContentHash` is `Exact` by construction (`manifest.rs:529-564`). ADR-003: metadata is not identity; the **hash** is. | `manifest.rs:92-96`, `:536-564` |
| 7 | **Content-addressing is uneven across symbol kinds.** **Types** are content-addressed (`elab::build_registry` → `#T#i` refs). **Values** carry a `ContentHash`. **Cross-nodule function** resolution is a **flat-NAME merge**, not content-hash (`link`, `checkty.rs:1122-1150`). | `checkty.rs:1122-1183`; DN-110-8.2 §6 |
| 8 | **The spore artifact carries source-by-hash, not checked exports.** `Spore { id, kind, surface: Vec<String>, sources: Vec<SourceFile{path, hash}>, deps: Vec<ResolvedDep>, name, version }`. `sources` are BLAKE3 of file **bytes** (not the text, not a checked interface); `surface` is bare **names** (no types). `build_spore` refuses a hashless dep at publish. | `mycelium-spore/src/lib.rs:34-75`, `:114-196` |
| 9 | **The registry can fetch + integrity-verify an artifact by name+constraint.** `resolve(root, name, constraint) -> Resolved { spore_id, artifact, bytes }`, bytes hash-verified before return. v0 constraints are **exact only** — a range/caret/tilde is `RegistryError::Unsupported` (exit 64). | `registry.rs:57-68`, `:267`, `:83` |

**Net:** the declaration layer (manifest `[dependencies]`, the typed content-pinned `hash`) and the
same-phylum resolution/link machinery (`Exports` / `resolve_imports` / `link`) both exist and work.
The gap is exactly the **layer between them**: read the deps, load their exports, resolve a cross-
phylum `use` against them through the *same* machinery, and pin it to the content hash. That is what
this note designs.

---

## §2 User stories (house rule #6)

- **US-1 (the headline).** *As a phylum author, I want to `use` a dependency phylum's `pub` symbol,
  so that I can build on a published library without copying its code into my phylum.*
- **US-2 (never-silent supply chain).** *As a phylum author, I want a hash / version mismatch between
  what my manifest pins and what actually resolves to be a **loud, explicit** error, so that I never
  silently link the wrong bytes* (ADR-003 / G2).
- **US-3 (the facility / OQ-H1).** *As a sugar-facility author (DN-110/M-1054), I want a cross-phylum
  def-site reference to expand to a **content-addressed, stable, unforgeable** ref, so that a `sugar`
  rule defined in phylum P and used in phylum Q captures **P's** intended symbol across separate
  compilation — not whatever Q happens to have in scope.*
- **US-4 (one linker).** *As the maintainer, I want cross-phylum resolution to be a **layer over the
  single canonical linker** (M-1024), so that I maintain **one** resolution path, not a third parallel
  one* (DRY, the maintainer's Decision-1 directive).
- **US-5 (small kernel).** *As a kernel reviewer, I want v1 to add the **minimum** surface that makes
  US-1 work and defer everything else never-silently, so that the auditable kernel stays small*
  (KC-3 / YAGNI).

---

## §3 The objective function (the criteria every option is scored against)

Weighted by the house rules; used in the ranked tables (§4–§6).

| Criterion | What it rewards | House-rule basis |
|---|---|---|
| **C1 DRY-over-canonical-linker** | Reuses `Exports`/`resolve_imports`/`link`; no third linker | Decision 1; KC-3; US-4 |
| **C2 Content-addressing / ADR-003** | Resolution is driven by, and pinned to, the content hash; identity follows origin | ADR-003; rule #2 |
| **C3 KC-3 small kernel** | Minimum new surface; v1 vs deferred is explicit | KC-3; rule #5; YAGNI |
| **C4 Transparency / never-silent** | Every mismatch/ambiguity/cycle is an explicit refusal | rule #2; G2 |
| **C5 Reproducibility / separate-comp** | Same inputs → same result; a dep is checkable without rebuilding the world | ADR-003; ADR-013 |
| **C6 Feasibility on today's tree** | Buildable on the landed manifest/spore/linker without a large prerequisite | mitigation #14 |

---

## §4 Design axis 1 — cross-phylum reference **syntax**

How does a nodule name a symbol in a **dependency** phylum? The reference must be **never-silent**:
you can tell a cross-phylum reference from an intra-phylum one **by looking at it** (G2), and the
head resolves against the manifest `[dependencies]` local name (the existing resolution key).

| Option | Mechanism | C1 | C3 | C4 | Verdict |
|---|---|---|---|---|---|
| **A1 — extend `use` with a phylum-boundary marker** `use <dep>::<nodule>.<name>` (`::` crosses the phylum boundary; `.` stays the nodule/path separator; glob `use <dep>::<nodule>.*`) | `UsePath` gains an optional `phylum: Option<String>` head (the `[dependencies]` key); the checker routes a `Some(_)` head to the cross-phylum resolver, a `None` head to the existing intra-phylum path. Reuses the whole `resolve_imports` merge. | ✅ same `use`, same merge | ✅ one field + one lexer token | ✅ boundary is lexically visible | **Rank 1** |
| **A2 — a distinct keyword** `from <dep> use <nodule>.<name>` or `extern use …` | New keyword + new grammar production; a second import form to teach/lint/format. | ➖ new production | ❌ two import syntaxes | ✅ explicit | Rank 3 |
| **A3 — overload `.` and disambiguate by lookup** `use <dep>.<nodule>.<name>`; the resolver tries the head against the dep table, else the local nodule set | No new token, but the **head is ambiguous** when a local nodule root shares a dep's name; forces a disambiguation rule (a collision must be a never-silent error, else a silent winner). | ✅ zero new syntax | ✅ zero new field | ➖ needs a collision rule to stay never-silent | **Rank 2** |

**Recommendation — Rank 1 (A1, `::` boundary marker).** The `::` phylum-boundary separator makes a
cross-phylum reference **self-evident** (never-silent by *construction*, not by a disambiguation
rule), mirrors the widely-understood Rust `crate::` intuition, and is a **bounded** grammar addition
— one lexer token and one optional `phylum: Option<String>` field on `UsePath` (`ast.rs:63`). The
head is the manifest `[dependencies]` **local name** (already the resolution key, `manifest.rs:87`),
so `use collections::seq.List` reads "from the dependency I locally call `collections`, its `seq`
nodule's `List`." A2 doubles the import surface (fails C3). A3 saves the token but must *invent* a
never-silent collision rule to stay honest, and reintroduces exactly the head-ambiguity `::` removes;
it is the acceptable fallback if the maintainer wants **zero** grammar change in v1.

> **Note (append-only status of the grammar).** Activating the `::` token / `phylum` head is a
> **surface-grammar** change; this DN *recommends* it, and its ratification should be reflected in the
> owning grammar RFC (RFC-0006 §4.3, the `use` production) — **FLAG-SYNTAX**, not edited here.
> `phylum` is a **reserved-not-active** keyword today (`lang-lexicon-syntax.md:132`); `::` gives the
> cross-phylum boundary without consuming that keyword, which stays available for a future
> `phylum <path>` header block.

---

## §5 Design axis 2 — the `Phyla` dependency-graph model + **how deps are loaded**

Two sub-decisions: **(a)** the new type shape, and **(b)** where a dependency's `Exports` come from.

### §5.1 The `Phyla` type (additive over `check_phylum`)

A new type in `mycelium-l1` (`Declared`), **additive** — `check_phylum(&Phylum)` is unchanged:

```
Phyla {
  // dep-local-name  ->  the resolved, checked dependency phylum
  deps: BTreeMap<String, ResolvedPhylum>,
  // the DAG edges from the root manifest (+ transitively), for cycle detection + ordering
  edges: …,
}
ResolvedPhylum {
  phylum_hash: ContentHash,     // the authoritative content pin (ADR-003) this resolved to
  exports:     Exports,          // the SAME pub-only import registry check_phylum already builds
  env:         Env,              // the SAME linked runtime env PhylumEnv::link already produces
}
```

The new entry point is **additive**: `check_phylum_with_deps(phylum: &Phylum, deps: &Phyla) ->
Result<PhylumEnv, CheckError>`. It is `check_phylum` (§1 fact 2) with **one added step** — before
resolving intra-phylum `use`s, it makes each dep's `Exports` available under its phylum-qualified key
(§7). A phylum with no `[dependencies]` gets an empty `Phyla` and behaves **exactly** as today
(backward-compatible, the M-662 "additive layer" pattern).

### §5.2 Where a dependency's `Exports` come from

| Option | Mechanism | C5 | C6 | ADR/notes |
|---|---|---|---|---|
| **B1 — load from the resolved spore artifact** | `registry::resolve` the dep by name→pin, then fetch each `SourceFile` blob from the object store, re-parse + `check_phylum`. | ➖ still recompiles (spore carries source-by-**hash**, not checked exports — §1 fact 8) | ➖ needs the object store to hold source *blobs*; more moving parts | reproducible; content-pinned |
| **B2 — load from a verified source tree** (workspace path-dep, or a fetched tree **verified against the manifest pin**) | Resolve the dep to a source directory; `check_phylum` it; **assert its content hash == the manifest `hash`** (never-silent mismatch, §1 fact 6/9). | ➖ recompiles the dep each build (correct, not fast) | ✅ `lib/std` is source *right here*; smallest step | reproducible **via the pin check** |
| **B3 — a spore-carried checked-exports interface blob** (an "rlib"-like `Exports` serialization) | Extend `Spore` with a serialized, versioned checked-`Exports` section; a consumer loads it **without** recompiling the dep — true separate compilation. | ✅ genuine separate compilation | ❌ serialize+version an identity-bearing interface = its **own ADR** (what's in the blob, ABI/version, trust) | the real v2 |
| **B4 — a `mycelium-proj.lock`** pinning the whole transitive graph (name→hash) | Orthogonal: the deterministic **graph-resolution** layer feeding B1/B2/B3. | ✅ deterministic graph | ✅ small (a generated file) | complements any of B1–B3 |

**Recommendation — v1 = B2 + B4; defer B3.** Load each dep from a **source tree** (a workspace/path
dependency now; a fetched tree later) and **verify the checked phylum's content hash against the
manifest pin** — a mismatch is a never-silent `Integrity`-class refusal (reuse the exit-5 pattern,
`registry.rs:83`). Add a generated **lock** (B4) so the transitive graph resolves deterministically.
Rationale against the objective: B2 scores highest on **C6** (`lib/std` is source in-tree — the
dogfooding target is immediately reachable) and keeps **C5** honest via the pin check, at the cost of
recompiling deps each build — an **optimization**, not a correctness gap (KISS/YAGNI). **B3 is the
right v2** but it serializes an **identity-bearing interface**, which is a decision of its own (an
ADR: blob contents, versioning/ABI, trust boundary) — pulling it into v1 violates C3 and stalls the
"must get working" goal. **Be explicit (VR-5): v1 is *whole-graph compilation with content-pinned
inputs*, not separate compilation.** Say so; don't claim the separate-comp property B3 buys.

---

## §6 Design axis 3 — content-addressed, version-pinned resolution (ADR-003 / OQ-H1)

**How the pin drives resolution.** For each `[dependencies]` entry, resolution produces a
`ResolvedPhylum` whose `phylum_hash` **must equal** the manifest's `hash` pin (`ContentHash`,
identity-bearing, `manifest.rs:92-96`). A mismatch — the tree/artifact that resolved does not hash to
the pin — is a **never-silent** refusal (G2). The human `version` (`^2`, …) is checked **against** the
pin as an advisory consistency signal, never as the identity (ADR-003: metadata is not identity).
v1 honors **exact pins only** — a range constraint is `Unsupported` today (§1 fact 9); ranges are
deferred (§8).

**The OQ-H1 capture-of-hash (US-3).** A cross-phylum reference `use dep::nod.sym` resolves, **at
check time**, to a content-addressed **def-site ref**:

```
CrossPhylumRef = (phylum_hash: ContentHash, qualified_name: "nod.sym")
```

This is **stable + unforgeable across separate compilation**: `phylum_hash` pins the exact source
bytes of the dependency, so `(phylum_hash, "nod.sym")` denotes one specific definition that no
use-site can shadow or forge — the def-site-resolution answer OQ-H1 asked for, at **phylum
granularity**.

> **The honest granularity boundary (VR-5).** Today **types** are content-addressed (`#T#i`) but
> **functions** are flat-name-merged (§1 fact 7). So a *per-function* content hash does not exist yet.
> v1's def-site ref is therefore **`(phylum_hash, qname)`** — content-**stable** (the phylum hash
> fixes the fn's bytes transitively) even though the function is not *independently* hashed.
> Extending content-addressing to function symbols (a per-symbol hash) is **deferred** (§8) and is
> the precise residual OQ-H1 flags. v1 does **not** claim per-symbol content identity for functions;
> it claims phylum-pinned stability, which is enough for the facility's def-site capture to be
> unforgeable. This resolves the **cross-phylum half of OQ-H1** to a `Declared` position for
> maintainer ratification; it does **not** close OQ-H1 (house rule #3).

---

## §7 Design axis 4 — layering over the **one** canonical linker (DRY / US-4)

The maintainer's Decision-1 directive: cross-phylum is a **layer**, not a third linker. This is
achievable because the same-phylum machinery is already the right shape, keyed by **qualified name**:

1. **Check-time (resolution).** `Exports` (`checkty.rs:1211`) is the pub-only import registry keyed by
   qualified name; `resolve_imports` (M-662) merges a nodule's `use`s against it at a documented
   precedence, refusing unknown/private/ambiguous never-silently. The cross-phylum layer builds a
   **phylum-qualified export view**: each dep's `Exports` namespaced under its dep-local-name (so
   `collections::seq.List` is a distinct key from a local `seq.List`). A cross-phylum `use` resolves
   through the **same** `resolve_imports` merge — it just draws candidates from the qualified view.
   **No second resolver.**
2. **Runtime (execution).** `PhylumEnv::link` (`checkty.rs:1086`, M-1024) folds a phylum's checked
   home decls into one `Env`. The cross-phylum dual links each dep's `Env` in under its **phylum-
   qualified** key. Crucially, the same-phylum `link` is a **flat** namespace (one decl per simple
   name; a cross-nodule collision is a never-silent `CheckError`; qualified disambiguation deferred to
   **M-982**). Cross-phylum **cannot** be flat (every phylum has a `map`), so the cross-phylum layer
   **must** carry the phylum qualifier on the key — which is exactly the **same M-982 residual**,
   surfaced one level up. v1 therefore requires an **explicit (non-glob)** cross-phylum `use` that
   binds one simple name, and refuses a cross-phylum simple-name collision never-silently, deferring
   glob + disambiguation to M-982. **Same linker, one added qualifier dimension — DRY satisfied.**

This is why cross-phylum is a **layer over M-1024**, not a rewrite: `Exports`/`resolve_imports`/`link`
each gain a phylum-qualifier on their key space; the merge/precedence/never-silent logic is untouched.

---

## §8 The v1 / deferred boundary (KC-3 — the small-kernel line)

**v1 (what "get it working" is):**

- **Syntax:** extend `use` with the `::` phylum-boundary head (§4 Rank 1) — `use dep::nod.sym`; the
  head is the `[dependencies]` local name. **Explicit imports only** (no glob).
- **`Phyla` type + `check_phylum_with_deps`** (§5.1), additive over `check_phylum`.
- **Loading:** B2 (verified source tree) + B4 (a generated lock) (§5.2). **Exact pins only.**
- **Resolution:** content-pinned; `phylum_hash` == manifest pin or a never-silent refusal (§6);
  the def-site ref is `(phylum_hash, qname)`.
- **Layering:** through the existing `Exports`/`resolve_imports`/`link` with a phylum qualifier (§7).
- **Never-silent everywhere:** unknown dep, unknown/private symbol, hash mismatch, version skew,
  cross-phylum name collision, **cycle** → each an explicit `CheckError`/refusal (G2).
- **Check-time only.**

**Deferred (flagged, never a silent assumption — VR-5):**

| Deferred | Why | Where it lands |
|---|---|---|
| **Separate compilation** (B3 spore interface blob) | serializes an identity-bearing interface — its own ADR | v2 / new ADR |
| **Re-export** (`pub use dep::…`) | YAGNI until the corpus needs it; the **origin-hash rule is specified now** (§9.4) so it is not a silent gap | v2 |
| **Per-function content hash** | functions are flat-name today (§1 fact 7); phylum-hash granularity suffices for v1 | separate work item |
| **Glob cross-phylum `use dep::nod.*`** + cross-phylum name disambiguation | the same residual as intra-phylum | folds into **M-982** |
| **Version *range* constraints** (caret/tilde) | registry `resolve` is exact-only (`Unsupported`, §1 fact 9) | v2 |
| **Runtime / dynamic multi-spore linking** (colony germination) | check-time is the v1 scope | later, ADR-013/ADR-020 territory |

---

## §9 Adversarial stress-test (house rule #4 / VR-5 — argue against the recommendation)

### §9.1 Diamond / version conflict — **the sharpest finding**

Root `P` depends on `A@h1` and on `B`; `B` depends on `A@h2` (`h1 ≠ h2`). Under the content-addressed
design, **`A@h1` and `A@h2` are two different phyla** (different `phylum_hash`). v1 **does not**
force-unify them and **does not** pick a SemVer winner. They coexist; a symbol from `A@h1` has a
different content-identity than the "same" symbol from `A@h2`, so a value of `A@h1`'s type flowing
where `A@h2`'s type is expected is a **type mismatch** — a **never-silent** refusal.

**This is safe and honest, but it is strict**, and it is the design's sharpest surprise: a user with a
Cargo mental model expects `A` to be *unified* to one compatible version across the diamond. v1's
answer is "they are different phyla; their types do not cross." That is **correct** (it never silently
links mismatched bytes — US-2) but may reject a program a SemVer-coalescing resolver would accept.
**The verdict: the strict-reject is the right *default* (never-silent, content-honest), but whether
v1 additionally needs a SemVer-compatibility *coalescing* policy is a real decision the maintainer
must make** — flagged as **OQ-CP-1** (§10). Do not paper it over with a silent "highest version wins":
that would violate rule #2 outright.

### §9.2 Hash mismatch

The resolved source tree / artifact does not hash to the manifest pin → an explicit `Integrity`-class
refusal naming the dep, the expected pin, and the computed hash (reuse `as_content_hash`'s never-
silent style, `manifest.rs:542-549`, and the registry exit-5 pattern). **Passes** (C4). No partial
link is produced (mirrors `build_spore`'s no-partial-artifact rule, `lib.rs:77-84`).

### §9.3 Cyclic phyla

`A` depends on `B` depends on `A`. Content-addressing **forbids** this by construction — a content
hash cannot transitively depend on itself (`A`'s hash would have to be known to compute `B`'s, which
is needed to compute `A`'s). v1 **detects the cycle during `Phyla` construction and refuses
never-silently** with the cycle path. **Passes** (C2/C4). This is not merely a policy choice; it is
forced by ADR-003 — a strong correctness argument *for* the content-addressed design.

### §9.4 A dep re-exporting a transitive symbol

`P` uses `B`; `B` re-exports `C`'s symbol; `P` references it. **v1 defers re-export** (§8), so the v1
behavior is a **never-silent "symbol not in `B`'s own `pub` exports; depend on `C` directly"**
refusal — *not* a silent transitive leak. The **origin-hash rule is specified now** so v2 is not a
silent assumption: *a re-exported symbol keeps its **origin** phylum's ref* `(origin_phylum_hash,
origin_qname)`, never the re-exporter's. Consequence: a chain `A → B → C` re-export resolves to `C`'s
origin identity, so a diamond that reaches the same origin `C` through two paths **is** the same
symbol (diamond-safe) — the mirror of types' content-addressing. This keeps v2 coherent and makes the
v1 deferral safe.

### §9.5 Undeclared / hashless dep

A `use dep::…` whose `dep` is not a `[dependencies]` key → never-silent "no such dependency `dep` in
the manifest." A dep with no `hash` → reuse `build_spore`'s existing hashless-dep refusal
(`lib.rs:167-181`) at resolution time. **Passes** (C4).

### §9.6 The DRY claim, tested against itself

Could cross-phylum secretly become a second linker? The risk: the flat-vs-qualified namespace gap
(§7) tempts a bespoke cross-phylum merge. The mitigation is structural — the qualifier is an added
**key dimension** on `Exports`/`link`, and the merge/precedence/never-silent code is the *same*. If an
implementer finds themselves writing a second `resolve_imports`, that is the signal the layering was
abandoned — call it out in `/pr-review`. **The claim holds only if the qualifier is threaded through
the existing types, not forked into new ones** — stated so the implementer cannot drift silently.

---

## §10 Open questions (NOT decided here — house rule #3 / VR-5)

- **OQ-CP-1 (diamond coalescing).** Is v1's strict "different hash ⇒ different phylum, types don't
  cross" (§9.1) the accepted default, or does v1 need a SemVer-compatibility **coalescing** policy?
  The strict answer is never-silent and content-honest; coalescing is a convenience that must **not**
  become a silent winner. **Sharpest — needs a maintainer disposition.** **RESOLVED at ratification
  (2026-07-10): the strict-reject default is accepted — NO silent SemVer-coalescing. Two different
  pinned hashes are two different phyla; a value crossing between them is a never-silent type
  mismatch. SemVer-coalescing stays deferred as a possible explicit opt-in v2 policy — see the
  Ratification section below.**
- **OQ-CP-2 (syntax marker).** `::` boundary (§4 Rank 1) vs `.`-plus-dep-table (Rank 2, zero grammar
  change but needs a never-silent collision rule). Grammar activation is FLAG-SYNTAX to RFC-0006.
  **RESOLVED at ratification (2026-07-10): Rank 1 (`::`) accepted — see the Ratification section
  below; FLAG-SYNTAX routing to RFC-0006 §4.3 stands, unchanged.**
- **OQ-CP-3 (loading).** v1 B2 (verified source) — accepted? Or commission B3 (spore interface blob /
  separate compilation) now, with its own ADR? **RESOLVED at ratification (2026-07-10): v1 = B2
  (verified source tree) + B4 (generated content-pinned lock), accepted as specified in §5.2/§8; B3
  (separate-compilation interface blob) stays deferred to v2/its own ADR — see the Ratification
  section below.**
- **OQ-CP-4 (OQ-H1 granularity).** Accept `(phylum_hash, qname)` as the v1 capture unit (§6), with
  per-function content-addressing deferred? Or require function-content-addressing in v1? This is the
  cross-phylum half of **DN-110-8.2 §10 OQ-H1**; this note recommends the phylum-granular answer.
  **RESOLVED at ratification (2026-07-10): `(phylum_hash, qualified_name)` accepted as the v1 capture
  unit; per-function content-addressing DEFERRED, honestly — see the Ratification section below. This
  resolves the cross-phylum half of DN-110-8.2 §10 OQ-H1 to this design; it does not itself close
  OQ-H1 (house rule #3).**
- **OQ-CP-5 (lock format).** Does v1 ship a `mycelium-proj.lock` (B4), and what is its shape /
  ownership (generated, committed)? **NOT dispositioned by this ratification** — v1 shipping a lock
  (B4) is accepted as part of the v1/deferred boundary (§8, OQ-CP-3 above), but the lock's exact
  shape/ownership is left open, genuinely unresolved — flagged, not guessed (G2/VR-5).

---

## §11 Definition of Done (house rule #6 — what ratification requires of the maintainer)

This note reaches **Accepted** when the maintainer:

1. **Picks the syntax** — §4 Rank 1 (`::`) or Rank 2 (`.`+dep-table) (OQ-CP-2), and FLAG-SYNTAX is
   routed to RFC-0006 §4.3 for the grammar activation.
2. **Picks the loading model** — v1 B2+B4 (§5.2) vs commissioning B3 now (OQ-CP-3).
3. **Dispositions OQ-H1's granularity** — accept `(phylum_hash, qname)` as v1's cross-phylum def-site
   ref (§6/OQ-CP-4), or require per-function content hashing in v1. On acceptance, this note becomes
   the **resolution basis for OQ-H1's cross-phylum half** (still not closing OQ-H1 — rule #3).
4. **Dispositions the diamond policy** — OQ-CP-1 (§9.1): strict-reject default vs a coalescing policy.
5. **Confirms the v1/deferred boundary** (§8) — re-export, glob, ranges, separate-comp, runtime all
   deferred with their never-silent v1 refusals.
6. **Mints the implementation issue** — see **FLAG-ISSUE (M-1060)** below — and this note moves to
   **Accepted** (never straight to Enacted; rule #3). Guarantees stay `Declared` until the v1
   subsystem lands and is differential-witnessed (a real cross-phylum `use` fixture over `lib/std`
   plus a second phylum, the M-662/M-1024 witnessing pattern).

**Verification the implementation must show (the "how verified"):** a two-phylum fixture (a dep
phylum and a consumer) where `myc check` resolves a cross-phylum `use`, a **hash-mismatch** fixture that
refuses never-silently, a **cycle** fixture that refuses, and a differential that the linked `Env` a
cross-phylum reference resolves through is the **same** one `PhylumEnv::link` produces (the DRY proof,
§7). Advances **FR/NFR** for the phylum/library capability (RFC-0006 §4.3) and grounds **OQ-H1**.

---

## Ratification (maintainer-delegated, orchestrator-selected on the merits, 2026-07-10)

**Recorded decision (append-only — this note's original §1–§11 text above is unchanged; this section
adds the ratification, per house rule #3).** The maintainer delegated the choice among this note's
ranked options ("ratify the options best fit objectively speaking for this project"); the integrating
orchestrator selected the recommended design below on the merits stated in §3–§9, and this section
records that selection as the ratification. This is a **delegated ratification, not a self-ratification
by the reasoner** — the maintainer authorized the delegation; the selection is grounded entirely in this
note's own objective-function analysis (§3's criteria table) and its adversarial stress-test (§9), not
asserted without basis.

1. **Syntax (§4/OQ-CP-2) — Rank 1 (A1) accepted.** `use dep::nod.sym` — the `::` phylum-boundary head
   on the existing `use` production, the local `[dependencies]` name as the head, `.` staying the
   nodule/path separator. Self-evidently never-silent by construction (G2), the smallest grammar
   addition (one token, one optional `phylum` field), and it avoids A3's head-ambiguity collision rule.
   **FLAG-SYNTAX stands** — this is a design ratification; the surface-grammar activation still routes
   to RFC-0006 §4.3 (unchanged from the note's own framing).
2. **Loading (§5.2/OQ-CP-3) — v1 = B2 + B4 accepted; B3 deferred.** Load each dependency from a
   **verified source tree** and assert its checked phylum's content hash equals the manifest pin (a
   never-silent `Integrity`-class refusal on mismatch); add a generated **lock** (B4) for deterministic
   transitive-graph resolution. **B3 (a spore-carried checked-exports interface blob / true separate
   compilation) is explicitly DEFERRED** — it serializes an identity-bearing interface, which is its
   own ADR-level decision (blob contents, versioning/ABI, trust boundary), not a v1 requirement (KISS/
   YAGNI, per the note's own §5.2 rationale). v1 is honestly **whole-graph compilation with
   content-pinned inputs**, not separate compilation — say so, don't claim the B3 property (VR-5).
3. **OQ-H1 granularity (§6/OQ-CP-4) — PHYLUM-level accepted; per-function hashing DEFERRED.** The v1
   cross-phylum def-site ref is `(phylum_hash: ContentHash, qualified_name: "nod.sym")` — content-
   **stable** (the phylum hash fixes the referenced symbol's bytes transitively) even though a function
   is not *independently* content-hashed (only types are, today — §1 fact 7). Extending content-
   addressing to per-symbol/per-function hashes is **deferred, honestly** — flagged as future work, not
   claimed as done. This resolves the cross-phylum half of DN-110-8.2 §10 **OQ-H1** to this design; it
   does **not** itself close OQ-H1 (house rule #3 — a design note does not retroactively close a
   different note's open question, it only supplies the resolution basis for one facet of it).
4. **Diamond policy (§9.1/OQ-CP-1) — NO silent SemVer-coalescing.** Two different pinned content hashes
   for "the same" dependency name (`A@h1` vs `A@h2` in a diamond) are **two different phyla** — their
   types do not cross, and a value of one flowing where the other is expected is a **never-silent type
   mismatch**, never a "highest version wins" resolution. House rule #2 forbids a silent coalescing
   default outright, and this ratifies the note's own §9.1 finding that the strict-reject is the
   correct, content-honest v1 default. **A SemVer-compatibility coalescing policy stays DEFERRED as a
   possible EXPLICIT opt-in v2 mechanism** — never a silent default, never assumed without a future
   ratification of its own.
5. **The v1/deferred boundary (§8) accepted as specified.** v1 = explicit (non-glob) cross-phylum
   `use`, the `Phyla`/`check_phylum_with_deps` additive type, B2+B4 loading, content-pinned resolution,
   layered over the existing `Exports`/`resolve_imports`/`link` machinery (DRY, §7), and never-silent
   refusals for every named failure mode (unknown dep, unknown/private symbol, hash mismatch, version
   skew, name collision, cycle). Deferred, never-silently: separate compilation (B3), re-export (with
   the §9.4 origin-hash rule specified now so v2 is not a silent gap), per-function content hashing,
   glob cross-phylum `use` (folds into M-982), version-range constraints, and runtime/dynamic
   multi-spore linking.
6. **Implementation issue minted.** **M-1060** — see `tools/github/issues.yaml` (applied by the
   integrating parent, per FLAG-ISSUE below).
7. **CRITICAL — Accepted ratifies the v1 DESIGN only, NOT Enacted (house rule #3 / VR-5).** This is
   genuinely green-field (§1: 0% wired today). No code has landed for the `Phyla` type, the `::` syntax,
   the B2/B4 loading path, or any cross-phylum resolution. Every guarantee this note's design implies
   stays **`Declared`** until M-1060 implements and differential-witnesses it (the §11 "how verified"
   criteria: a two-phylum fixture, a hash-mismatch refusal, a cycle refusal, and the DRY-linked-`Env`
   differential). Nothing here is upgraded past its checked basis by this ratification.

---

## §12 FLAGs (append-only rows the integrating parent applies — I do not edit these files)

- **FLAG-DOCINDEX.** Add a `docs/Doc-Index.md` row for **DN-113** (this note), in the DN block after
  the highest existing DN row (DN-112 the ctor-seal note, or DN-111 if the DN-112 row is not yet
  present). Suggested summary: *"Draft — designs the green-field
  cross-phylum import/resolution subsystem: a `::` phylum-boundary `use` head, an additive `Phyla`
  dep-graph + `check_phylum_with_deps`, content-pinned resolution to a `(phylum_hash, qname)`
  def-site ref (grounds DN-110-8.2 OQ-H1's cross-phylum half), loaded B2-from-verified-source with a
  lock, layered over the M-1024 canonical linker via a phylum qualifier (DRY, no third linker). v1
  defers separate-comp/re-export/glob/ranges never-silently. Recommends, does not ratify."* Status
  column: **Draft (2026-07-10)**.
- **FLAG-ISSUE (M-1060).** Mint **M-1060** in `tools/github/issues.yaml` (next free id; max is
  M-1059) — *"implement the DN-113 v1 cross-phylum import/resolution subsystem"* — with user stories
  US-1…US-5 (§2) and the §11 DoD. Suggested `depends_on`: the DN-113 ratification; relates to
  **M-1054** (the facility whose OQ-H1 cross-phylum def-site resolution this grounds) and **M-982**
  (the qualified-scoping residual §7 shares). Milestone: *Phase 5 — Self-Hosting & Core Library*
  (same as M-1054). `doc_refs`: `corpus:DN-113`, `src:crates/mycelium-l1/src/checkty.rs:1086`,
  `src:crates/mycelium-proj/src/manifest.rs:84`.
- **FLAG-SYNTAX.** The `::` phylum-boundary `use` head (§4 Rank 1) is a surface-grammar change —
  route to **RFC-0006 §4.3** (the `use` production) for activation on ratification. No grammar file
  edited by this note.
- **FLAG-CHANGELOG.** Add an append-only `CHANGELOG.md` entry under the design-phase Unreleased
  section noting DN-113 (Draft) filed.

---

## Changelog (this note)

- **2026-07-10 (later same day) — Ratified (maintainer-delegated, orchestrator-selected on the merits,
  house rule #3).** Status **Draft → Accepted** (v1 design ratification, **NOT Enacted** — VR-5,
  guarantees stay `Declared` until M-1060 implements and differential-witnesses the design). Accepts:
  the `::` phylum-boundary `use` syntax (Rank 1); B2 (verified source tree) + B4 (generated lock)
  loading, B3 deferred; the `(phylum_hash, qualified_name)` OQ-H1 capture-unit granularity,
  per-function hashing deferred; the strict no-silent-SemVer-coalescing diamond policy (OQ-CP-1), with
  an explicit-opt-in coalescing policy deferred to a possible v2; the §8 v1/deferred boundary as
  specified. Mints **M-1060** (the v1 implementation issue). OQ-CP-2/CP-3/CP-4/CP-1 resolved; OQ-CP-5
  (lock format/shape) stays genuinely open. See the "Ratification (maintainer-delegated,
  orchestrator-selected on the merits, 2026-07-10)" section above for the full recorded decision.
- **2026-07-10 — Draft filed.** design-reasoner (Opus), read against `dev 45927ea4`. Enumerates the
  cross-phylum import/resolution subsystem: syntax (§4), the `Phyla` model + loading (§5), content-
  pinned resolution + OQ-H1 (§6), layering over the canonical linker (§7), the v1/deferred boundary
  (§8), the adversarial stress-test (§9), open questions (§10), DoD (§11), FLAGs (§12). Recommends,
  ranked; ratifies nothing; moves no other doc's status (house rule #3). `Empirical` for the §1
  audit; `Declared` for the design.
