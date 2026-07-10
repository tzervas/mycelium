# Design Note DN-104 — Per-Constructor Visibility Seal (the ENB-4 grammar-slot close)

| Field | Value |
|---|---|
| **Note** | DN-104 |
| **Status** | **Draft** (2026-07-10). Authored alongside the **first landable increment** of M-1027 (ENB-4). It records the design of the per-constructor **visibility seal** (`pub type T = priv Mk(..)`) — surface, grammar, AST, export-table semantics, the never-silent cross-nodule construction refusal — resolves the DN-53 §B.6 Q1 fork **for ratification**, and **enacts nothing** and **moves no other doc's status** (house rule #3, append-only). Tags are `Empirical` where read against the code / witnessed by a running differential, `Declared` for any design not yet ratified (VR-5). |
| **Decides** | *Proposes, for ratification:* (1) a `priv` **seal marker** on the `constructor` production, reserved as a keyword (`Tok::Priv`, lexed `"priv"`) so it can never be a silent identifier (G2), parsed **immediately before a constructor name** (`= priv Mk(T1, …)`); (2) the marker is **threaded into the AST** (`Ctor.sealed: bool`), not flattened — so the type declaration stays faithful for printing / round-trip and every AST consumer; (3) **export-table semantics** — a `pub type` with a sealed constructor exports the **type NAME** (usable in a client nodule's signatures, `use`, and pattern position) but **WITHHOLDS the constructor from cross-nodule CONSTRUCTION**: constructing a sealed ctor from a foreign nodule is a never-silent `CheckError` (the capability-gate, FR-N3); (4) the seal is enforced by a per-nodule **withheld set** derived from the phylum export table (reusing the M-1024 / M-662 cross-nodule resolution machinery), checked at the two constructor-application sites; (5) **`priv` is only meaningful on a `pub type`** — a seal on a nodule-private type is a never-silent refusal (redundant: the whole type is already unimportable); (6) the **DN-53 §B.6 Q1 fork resolves to the BINARY seal** (sealed / not-sealed), **not** a full Rust-style `pub(path)` scoped-visibility grammar (KISS/YAGNI; §3). It does **not** edit `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (the integrating session owns those). |
| **Feeds** | DN-99 §A3 / register rows #37 (sealed-constructor visibility) + #69 (field-visibility sub-case), ENB-4; M-1027; DN-53 §B.5 (field-level visibility deferral — the encapsulation-by-type pattern this note makes first-class) + §B.6 Q1 (the binary-vs-scoped-visibility open sub-question this note resolves); M-1023 (the spw Wave-0 `Approx::proven` capability gate that was OMITTED rather than ported ungated — VR-5 — and which a sealed constructor now lets port faithfully); M-662 / M-1024 (the `pub` export table + cross-nodule link this note reuses, Enacted); DN-26 (SCC self-hosting, the Rust↔`.myc` dual). |
| **Grounds on** | KC-3 (small kernel — no new L0 node, no new checking pass; the seal is a boolean on an existing AST node + one predicate at the existing construction sites), DRY (reuse the M-662 export table + the `NoduleImports` ambiguity-set machinery — the withheld set is its exact twin), G2 (never-silent — a foreign construction, a redundant seal on a private type each print the fix), VR-5 (no tag upgraded past its basis — the seal is `Declared`, earned `Empirical` by the witnesses in §7), KISS/YAGNI (binary seal over a full `pub(path)` scoped-visibility grammar). |
| **Date** | July 10, 2026 |
| **Task** | M-1027 (ENB-4) — per-constructor visibility seal. |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This note records a design and a running
> increment; it does **not** take a decision (house rule #3 — the maintainer ratifies). Empirical claims
> are witnessed by the differential/conformance witnesses named in §7. The seal semantics, the marker
> keyword, and the §B.6 Q1 resolution are `Declared` until ratified. **No sycophancy:** §3 confronts the
> genuine fork (binary seal vs full `pub(path)` scoped visibility) on its merits and §6 states the
> residuals (pattern-position sealing, named-field visibility, the `.myc` enforcement mirror) plainly
> rather than claiming a finished "full visibility system".
>
> **CRITICAL, added at integration review (2026-07-10, dev-lane `/pr-review` + independently
> reproduced by the integrator — house rule #4, no claim upgraded past a checked basis).** The
> mechanism below is **NOT an enforced security/capability boundary**, contrary to §1's and §3's
> "capability-gate"/"unforgeable" framing as originally drafted. Mycelium resolves types and
> constructors **by bare name, re-resolved in the caller's own scope** (own local decls shadow
> imports — the pre-existing RFC-0006 §4.3 / M-662 precedence rule), not by nodule-qualified
> nominal identity. A foreign nodule can therefore declare **its own unsealed type/ctor of the same
> name** — never importing the real sealed one — and the checker accepts it: `check_phylum` returns
> `Ok` for a same-named local shadow that passes a locally-forged value where the sealed type is
> expected. Reproduced and **pinned** as
> `tests/ctor_seal.rs::known_gap_a_same_named_local_shadow_type_bypasses_the_seal`. The seal, as
> landed, is an **opt-in API-discipline nudge that refuses a well-behaved caller going through
> `use home.Ctor`** — it is not unforgeable and does not defend against an adversarial or even
> accidentally-colliding same name. **§1/§3/§4 below are left as originally drafted (append-only —
> the design record is not silently rewritten) but must be read through this correction; §6 records
> it as a residual and tracks the real fix as M-1036 (nodule-qualified type identity).** Until
> M-1036 lands, the maintainer should **not** ratify this note as delivering the FR-N3 capability
> guarantee — only the never-silent, opt-in-discipline mechanism.

---

## §1 Purpose

Close the sealed-constructor surface gap (DN-99 register row #37 / §A3, ENB-4). A semcore/stdlib porter
translating a Rust *unforgeable capability token* today has no faithful target. The concrete driver
(FR-N3, the numerics port): a `ProvenThm`-style value whose whole guarantee is that **only its home
nodule can mint one** (via checked logic), while any nodule may **receive, name, and pass** it. In Rust
this is the opaque-type / private-constructor idiom (`mod sealed { pub struct Proven(()); }` — the type
is `pub`, the field/constructor is not). Mycelium v0 has **no such marker**: `Vis` is item-level only
(`enum Vis { Private, Pub }`, `crates/mycelium-l1/src/ast.rs:42`), so a `pub type` exports its type name
**and** makes **every** constructor freely constructible cross-nodule — there is no way to export the
name while withholding construction. The spw Wave-0 port hit exactly this: `Approx::proven` was
**OMITTED** rather than ported with a fabricated ungated escape hatch (M-1023, VR-5 — an honest gap, not a
silent downgrade). This note adds the seal as **surface + AST boolean + one export-table predicate over
the already-Enacted M-662 `pub` machinery** — **no new kernel semantics, no new L0 node, no new checking
pass** (KC-3).

## §2 The surface + grammar

```
type_decl   ::= 'pub'? 'type' ident type_params? '=' ctor ('|' ctor)*
ctor        ::= 'priv'? ident ctor_fields?
ctor_fields ::= '(' type_ref (',' type_ref)* ')'
```

- **`pub type ProvenThm = priv Mk(())`** — the type name `ProvenThm` is exported to the phylum; its sole
  constructor `Mk` is **sealed** (withheld from cross-nodule construction). The `priv` marker sits
  **immediately before the constructor name**, mirroring how `pub` sits immediately before `type`/`fn`.
- **Per-constructor, not per-type** — a multi-constructor type may seal a subset:
  `pub type T = Open(A) | priv Closed(B)` exports both constructor *names* for pattern-matching but
  withholds only `Closed` from foreign construction. (The realistic capability case is single-constructor;
  the grammar does not special-case it.)
- **`priv` is reserved** (`Tok::Priv`, lexed `"priv"`) so it can never silently become an identifier (G2),
  exactly as `pub` is reserved. It is meaningful **only** at constructor position; a `priv` anywhere else
  is a parse error (the parser only consults it in `parse_ctor`).
- **Backward compatible** — an unmarked constructor is `sealed: false`; every existing `type` declaration
  parses and checks unchanged (the marker is an *optional* prefix, absent by default).

### §2.1 Why `priv` (the marker keyword)

`priv` is the marker the DN-99 register (#37) and the M-1027 issue both write (`= priv Mk(..)`). It reads
correctly for the semantics under Mycelium's default-private model: the **type** is `pub`, the
**constructor** is `priv` (private) — a reader coming from Rust's `pub struct Foo(())` with a private
field reads "private constructor" and is exactly right. `seal`/`sealed` was considered (more
intent-revealing — "a sealed constructor") and is a legitimate alternative the maintainer may prefer; this
note recommends **`priv`** for register-fidelity and the `pub`/`priv` symmetry, and flags the choice
(§6). Either way the AST field is named `sealed` (the *property*), decoupling the surface keyword from the
internal representation.

## §3 The design fork (DN-53 §B.6 Q1 — confronted, not glossed)

**Binary seal vs full `pub(path)` scoped visibility.** Rust expresses this capability with
`pub(crate)` / `pub(super)` / `pub(in path)` — a *scoped-visibility* grammar where an item is visible
within a named sub-scope. Mycelium's current model is **binary** (`pub` = phylum-wide export; absent =
nodule-private). Two ways to close row #37:

- **(A) Binary seal (RECOMMENDED).** A single boolean per constructor: exported-name-but-withheld-
  construction, keyed off the **nodule boundary** (the one visibility boundary Mycelium has today). No new
  scope grammar, no `pub(...)` parse surface, no path-resolution against a scope. It expresses the FR-N3
  capability-gate exactly — "only the home nodule constructs" — because the home nodule *is* the natural
  minting boundary. Cost: it cannot express "visible to this sub-tree of nodules but not that one"; but
  Mycelium has no sub-nodule scope grammar for such a target to name (there is no `super`/`crate`
  analogue with sub-scopes — the phylum is flat over nodules).
- **(B) Full `pub(path)` scoped visibility.** A general grammar: `pub(nodule)`, `pub(phylum)`,
  `pub(in a.b)`. Strictly more expressive, and it would subsume the binary seal (the seal is
  `pub(nodule)` on a constructor). Cost: a whole new visibility grammar, path resolution against the
  nodule tree, and a scope model Mycelium does not otherwise have — a large surface for a capability whose
  **only witnessed demand today is the binary home-nodule mint-gate** (M-1023 / FR-N3). DN-53 §B.6 Q1
  already leans "keep binary for v1" (KISS); registry/package work (multi-phylum visibility) is where
  scoped visibility would earn its keep, and it has not landed.

**Resolution (for ratification): (A) — the binary seal.** It closes the witnessed gap with the smallest
surface (KC-3/YAGNI), and it is **forward-compatible**: should scoped visibility later land, `priv Mk` is
exactly `pub(nodule) Mk` and can be re-expressed without a semantic change (the seal is the
`nodule`-scoped point of the future lattice). We do **not** build (B) speculatively (YAGNI); the choice is
flagged for maintainer override (§6). This is the row #37 close; the row #69 field-visibility sub-case is
addressed by §5 (encapsulation-by-type, DN-53 §B.5 — sealing the *constructor* is the positional-field
analogue that does not require named-field syntax).

## §4 AST + checking (the mechanism)

**AST.** `Ctor` gains `sealed: bool` (`crates/mycelium-l1/src/ast.rs`). The parser sets it from the
optional `priv` prefix in `parse_ctor`. Every other `Ctor` construction site (ambient rebuild, the test
fixtures) threads it through; the surface printers (`ambient::print_type_decl`, `mycelium-fmt`'s
`render_ctor`) emit a leading `priv` marker for a sealed ctor so `parse → expand_to_source → parse`
round-trips.

**Export table (M-662 reuse).** The phylum-wide export table (`Exports`) gains a
`sealed: BTreeMap<qualified-type-name, {sealed ctor names}>`, populated from each `pub type`'s AST while
the table is built. When a nodule imports a type, its sealed constructor names are folded into a per-nodule
**withheld set** (`NoduleImports.sealed`) — the exact twin of the existing `NoduleImports.ambiguous`
glob-collision set (DRY: the withheld set answers "which imported ctor names may not be constructed here",
parallel to "which imported names may not be referenced here"). The nodule's **own** constructor names are
subtracted from the withheld set (an own constructor is always constructible in its home; own decls shadow
imports).

**Refusal (never-silent, G2).** At each of the two constructor-application sites in the body checker (the
nullary-ctor-as-value path and the saturated `App` path), a resolved constructor whose name is in the
withheld set is refused with an explicit `CheckError` naming the sealed constructor, its home, and the
fix ("construct it in its home nodule, or expose a `pub fn` factory"). A **redundant** seal — `priv` on a
constructor of a **non-`pub`** type — is refused at registration (`resolve_ctors`): the whole type is
already nodule-private, so the marker is meaningless and likely a mistake.

**What is NOT withheld (the design choice, §3(3)).** The type **name** stays importable and usable in a
client's signatures and `use`. **Pattern position is permitted**: a foreign nodule that holds a
`ProvenThm` may `match` / destructure it — this is safe for the capability property, whose invariant is
**unforgeability** (you cannot *mint* one), not opacity (destructuring a `Mk(())` reveals only `()`). A
stricter *fully-opaque* variant (seal pattern-matching too) is a genuine follow-on, flagged in §6 — this
note deliberately scopes the seal to **construction** because that is the FR-N3 capability-gate, and says
so rather than silently choosing (G2).

## §5 Interaction with named-field visibility (DN-53 §B.5)

DN-53 §B.5 **deferred** field-level `pub` (fields are positional/unnamed in v0; the encapsulation-by-type
pattern — a `pub` type with private constructor helpers — was named the pragmatic alternative). The
per-constructor seal **is that pattern made first-class**: `pub type T = priv Mk(F1, F2)` is the direct
surface for "the type is exported, its fields are reachable only through the home nodule's `pub` accessor
functions". It composes cleanly with the future named-fields extension: when named fields land, a
per-field `pub` can be designed **on top of** the constructor seal (a sealed constructor with a subset of
`pub` fields), and nothing here forecloses it. This note therefore addresses the row #69 field-visibility
**sub-case** (positional) without the premature named-field syntax DN-53 §B.5 warned against — it is the
DN-53-sanctioned near-term close, not a contradiction of the deferral.

## §6 Residuals (stated plainly, not glossed — house rule #4)

- **CRITICAL — not an enforced security/capability boundary (M-1036, added at integration review
  2026-07-10).** The withheld set (`NoduleImports.sealed`) is keyed by **bare constructor name** and
  populated only along the `use`-import path; it is never consulted for a caller's **own locally
  declared** type/ctor. Because Mycelium resolves types/ctors by bare name in the *caller's own scope*
  (own decls shadow imports — RFC-0006 §4.3 / M-662, pre-existing and unrelated to this note), a foreign
  nodule that declares a **same-named local (unsealed) type**, without ever importing the real sealed
  one, bypasses the seal entirely — `check_phylum` accepts the resulting program. This falsifies the
  §1/§3 "unforgeable capability-gate" framing for anything but a well-behaved caller that goes through
  `use home.Ctor`; it is **not** a defense against an adversarial or accidentally-colliding same name,
  and **M-1023's `Approx::proven` port must not rely on it as a real security boundary** until fixed.
  Pinned by `tests/ctor_seal.rs::known_gap_a_same_named_local_shadow_type_bypasses_the_seal` (asserts
  the current, unsound `Ok` — update to assert the refusal once fixed). The real fix needs
  **nodule-qualified type identity** (types resolved against their *declaring* nodule, not the caller's
  bare-name lookup) — a materially larger change than this note's scope, tracked as **M-1036**. The
  maintainer should ratify this note as delivering the **never-silent, opt-in-discipline** mechanism
  only, not the FR-N3 capability guarantee, until M-1036 lands.
- **Marker keyword `priv` vs `seal`/`sealed`** (§2.1) — recommended `priv` (register-fidelity, `pub`/`priv`
  symmetry); the maintainer may prefer the more intent-revealing `sealed`. The AST field (`sealed`) is
  keyword-independent, so a later keyword swap is a lexer + parser + printer change only.
- **Fully-opaque variant (seal pattern-matching too)** (§4) — this note seals **construction only**;
  sealing destructuring/pattern-position is a separate, stricter capability a future note may add on
  witnessed demand. Never-silent: the current scope is documented, not implied.
- **Scoped visibility `pub(path)`** (§3) — deferred (YAGNI) until multi-phylum / registry work witnesses
  demand; `priv` is forward-compatible as the `nodule`-scoped point.
- **The `.myc` enforcement mirror** — the `.myc` self-hosted frontend (`lib/compiler/*.myc`) mirrors the
  **surface** (`priv` token + `Ctor.sealed` + parse) and the **type vocabulary** (semcore.myc's
  `Ctor`/`TypeDecl`), but the **cross-nodule export-table enforcement** (`resolve_imports` + the withheld
  set) is **not yet ported to `.myc`** — the whole M-662/M-1024 cross-nodule checking layer is Rust-only
  in the current port (semcore.myc grades individual `FnDecl` bodies). The seal's enforcement `.myc`
  mirror **rides the checkty cross-nodule port**, and is FLAGGED as a residual (DN-26 parity is at the
  surface + vocabulary layer this increment, not the enforcement layer). This is honest scope, not a
  silent omission.
- **`.myc` object-body seal parity gap (added at integration review 2026-07-10).** The Rust frontend
  refuses `priv` inside an `object` body at parse (`crates/mycelium-l1/src/parse.rs`, §2 scope); the
  `.myc` self-hosted `parse_object_decl` (`lib/compiler/parse.myc`) currently has **no matching
  refusal** — it accepts `priv` there and threads the (unused) `sealed` flag through `OD(...)` silently.
  A minor parity gap, not a soundness issue (the Rust frontend is the checked oracle); a one-line
  symmetric refusal in `parse_object_decl` would close it.

## §7 Definition of Done + witnesses

- **Accept** — a sealed constructor is constructible **in its home nodule** (`pub type T = priv Mk(A)`;
  the home nodule's `fn mk() => T = Mk(a)` checks); the type **name** is importable and usable in a foreign
  nodule's signature (`fn f(x: T) => T = x` checks after `use home.T`).
- **Reject (never-silent, G2)** — (a) a foreign nodule constructing the sealed ctor **via an imported
  name** (`use home.Mk; fn forge() => T = Mk(a)`) → the withheld-construction refusal; (b) `priv` on a
  non-`pub` type → the redundant-seal refusal.
- **NOT rejected (the §6 known gap, M-1036)** — a foreign nodule constructing a **same-named local
  (unsealed) type/ctor** it never imported, then passing that value where the sealed type is expected,
  type-checks (`check_phylum` returns `Ok`). This is the gap that falsifies the "unforgeable" framing;
  pinned by `tests/ctor_seal.rs::known_gap_a_same_named_local_shadow_type_bypasses_the_seal`.
- **Differential (DN-26 / `/myc-dogfood`)** — the Rust-oracle differential witnesses the three intended
  behaviours (home-construct OK · foreign-construct-via-import refused · cross-nodule type-use OK) **and**
  pins the known gap above as an 11th witness; the `.myc` frontend `myc check`-clean over the touched
  nodules (dogfood parity, surface/AST/fingerprint layer; enforcement mirror FLAGGED as riding the
  checkty port — §6).
- **Guarantee** — `Declared` (surface + a boolean on an existing node + one predicate at the existing
  construction sites), upgraded to `Empirical` by the running conformance + differential witnesses above
  **for the never-silent, opt-in-discipline mechanism this note actually delivers**; **not** upgraded to
  a `Proven` or enforced-capability claim (no such claim is checked — VR-5; §6 known gap M-1036).

## §8 Changelog

- **2026-07-10** — DN-104 created (**Draft**). Recorded the per-constructor visibility-seal design
  (surface/grammar §2; the marker-keyword choice §2.1; the binary-seal-vs-`pub(path)` fork resolved to
  binary §3; the AST-boolean + export-table-withheld-set + never-silent-refusal mechanism §4; the DN-53
  §B.5 field-visibility interaction §5; residuals §6; the DoD/witnesses §7). Authored READ + DN + the
  M-1027 increment only — no edit to `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (integration-owned;
  FLAGGED up). `Empirical` where cited against the tree (dev `30a0ff3f`) or witnessed by a running test;
  `Declared` for the unratified seal semantics + the §B.6 Q1 resolution. Append-only; status advances only
  by maintainer ratification (house rule #3).
- **2026-07-10 (integration review)** — a dev-lane `/pr-review` pass found, and the integrator
  independently reproduced (`cargo run` against a standalone same-named-shadow program), a **Critical**
  soundness gap: the withheld set is bypassed by a foreign nodule's same-named local (unsealed) type
  declaration, because Mycelium resolves types/ctors by bare name in the caller's own scope rather than
  by nodule-qualified nominal identity — a pre-existing property of the type-resolution model, not a bug
  introduced by this note's mechanism. Added the CRITICAL callout after the grounding block, a new §6
  residual bullet (+ the M-1036 tracking issue for the real fix — nodule-qualified type identity), a
  pinned "NOT rejected" case in §7, and softened §1/§3/§4's "unforgeable capability-gate" language via
  the callout (the original prose is left intact below it, append-only — read through the correction).
  Also added the `.myc` object-body seal parity gap to §6 (previously disclosed only in the PR commit
  message, not the design record). Status **stays Draft**; the maintainer ratifies knowing the corrected,
  narrower scope (never-silent opt-in discipline, not an enforced capability boundary) — never silently
  upgraded (VR-5/G2, house rule #4).
