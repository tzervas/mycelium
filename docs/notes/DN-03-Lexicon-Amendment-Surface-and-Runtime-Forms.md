# Design Note DN-03 — Lexicon Amendment: Surface Additions & Runtime Names (one name each)

| Field | Value |
|---|---|
| **Note** | DN-03 |
| **Status** | **Resolved** (2026-06-10) — ratified by the maintainer through the DN-02 three-test gate; the agreed set feeds the grammar artifacts (RFC-0006 §4.3) |
| **Amends** | DN-02 (Resolved, append-only) — adds Surface terms and the one-name-per-term rule; does **not** rewrite DN-02 (supersede-don't-edit) |
| **Feeds** | RFC-0006 (surface vocabulary); the grammar/conformance corpus; the L1 prototype reserved-word set; the M-142 formatter |
| **Date** | June 10, 2026 |
| **Decides** | (a) the Surface-tier additions `consume`/`grow` (adopt) and `embody` (decline → keep `impl`); (b) `for`'s reservation (RFC-0007 §4.8 r2); (c) **one name per term** — flat, rejecting ADR-012 §7.6's canonical+alias scheme; (d) the single Runtime-tier names against the RFC-0008-grounded meanings |
| **Depends on** | DN-02 (the naming law + three-test gate); ADR-012 §7.5/§7.6 (the flags — §7.6's alias scheme is superseded here); RFC-0007 §4.8 (`for`); RFC-0008 §4.5 (the grounded Runtime meanings the names are scored against) |

> Like DN-02, this note narrows and the maintainer ratifies. The three-test gate (DN-02 §1 —
> **T-map** fidelity, **T-illuminate** teaching value, **T-learn** dual readability) is applied
> verbatim; a term ships themed only if it passes, and keeps the conventional term otherwise.

---

## 1. Surface-tier additions (ADR-012 §7.5, through the gate)

| Candidate | Concept | Gate result | Decision |
|---|---|---|---|
| **`consume`** | acquire + take exclusive ownership of an affine `substrate` (LR-8) | **T-map:** a fungus consumes substrate *exactly once* = affinity — accurate. **T-illuminate:** teaches single-use. **T-learn:** reads cleanly for both audiences. **Passes.** | **Adopt** (themed). |
| **`grow`** | derive-like / generative capability extension (`grow Debug for T`) | **T-map:** the system *grows* new capability — fair. **T-illuminate:** acceptable, mildly generic. **T-learn:** fine. **Passes (with note).** | **Adopt** (themed; note the mild genericness). |
| **`embody`** | inherent methods on a type (≈ Rust `impl`) | **T-map:** "the type embodies its capabilities" — decorative, not behavioral. **T-illuminate:** weak — teaches little about the behavior. **T-learn:** `impl` is highly machine-/human-familiar; theming *costs* dual readability for no teaching gain. **Fails T-illuminate; loses T-learn.** | **Decline → keep `impl`** (the conventional term, by the same logic that kept `trait`/`type`/`use` in DN-02 §3). |

`consume`'s operand type is `substrate` (DN-02, ratified). Inherent-method blocks are **not in
the v0 grammar** (no methods exist yet); `impl` is recorded here as the chosen term **for when
they enter the grammar** — the binding is decided, the grammar work is later.

## 2. Control flow — `for` reserved (RFC-0007 §4.8 r2)

`for` is **reserved** as the keyword of bounded-iteration sugar (RFC-0007 §4.8): elaboration-
defined, `Total` by construction, spelling **adopted** (RFC-0007 r3; under RFC-0006's global
KC-2 gate like all v0 surface syntax). It is **conventional**
(not themed): an iteration head is universal scaffolding, high on T-learn, and theming it would
cost machine familiarity for no teaching gain — the same reasoning that kept `let`/`if`/`match`
in DN-02 §3.

**Still not reserved** (DN-02 §6 reaffirmed): `while`, `loop`, `break`, `continue`, `return` —
unbounded iteration would undermine the divergence bit (RFC-0007 §4.5), and the L1 prototype
already emits *teaching diagnostics* where these words appear (parse-level juxtaposition,
check-level unknown name) pointing at recursion / `for`.

## 3. One name per term — flat (supersedes ADR-012 §7.6's canonical+alias scheme)

ADR-012 §7.6 proposed a **canonical long form + one short alias** per Runtime term, on the
reasoning that content-addressing (ADR-003) makes a second spelling "free" (same hash, projected
differently by the formatter). **Rejected** here in favor of a flat rule:

> **Each term has exactly one name.** No canonical/alias pairs, no per-audience projection.

The "free" benefit is speculative (there are no users, and no formatter yet), while the cost is
real *now*: two spellings per concept to keep in sync, a normative projection rule, and the
honesty wrinkle that a "synonym alias" (`anastomose`/`fuse`) is two different words dressed as
one. Flat is DRY/KISS — pick the single clearest name and stop. Where a concept's *themed* word
is itself the clearest, it is themed (`hypha`, `cyst`, `graft`); where a plain word is clearer,
the plain word is the name (`fuse`, `mesh`, `reclaim`) — the DN-02 gate applied once, to one
name. Content-addressing still underlies *definition* identity; the lexicon just doesn't mint two
labels for it.

This still bounds growth (the `sclrt`/`sclerotize`/`sclerotium` sprawl is exactly what one-name
forbids) and keeps the reserved-word set minimal.

## 4. Runtime-tier names (one each, against the RFC-0008 meanings)

Now that RFC-0008 §4.5 gives each Runtime term an operational meaning, the T-map test is
runnable. The ratified **single name** per concept:

| Concept (RFC-0008) | **Name** | Why this name |
|---|---|---|
| concurrent execution unit | `hypha` | the fungal unit; short, pronounceable, the signature concept of the runtime |
| lawful state fusion (RT6) | `fuse` | RT6 is genuine *merge* (CRDT join — two states converge into one); `fuse` is honest to "two become one". (`anastomose` was the obscure long form; `weave` was rejected on T-map — woven threads stay distinct) |
| decentralized mesh (RT5) | `mesh` | a gossip/pub-sub overlay has a *universal* name; the gate keeps the universal word where it is clearest (`cmn`/`mycorrhizal-network` were opaque/ornamental) |
| capability contract w/ infra | `graft` | `myco`/`mycorrhize` **collide with the language family name** (Mycelium/"Myco"); `graft` is pronounceable and teaches host-binding (eyes open: botanical not fungal, implies more permanence than RT4's revocable affine contract — accepted as clean-beats-ugly, `symb` being a vowel-strip) |
| durable checkpoint | `cyst` | **encystment is becoming a dormant, resistant, *resumable* form** — the most behaviorally-accurate T-map in the set (a cyst *is* an RT2 checkpoint); used constructor-style `cyst(computation)`, matching `spore(value)`. (`sclerotium` was the obscure long form; `sclrt` an unpronounceable vowel-strip; `dorm` the runner-up) |
| explicit value movement | `xloc` | `x` = cross/trans is an established abbreviation convention (xfer, xlat, xchg), so `xloc` *teaches* "trans-locate"; chosen over the longer `translocate` for the flat single name |
| adaptive placement policy | `forage` | a clean pronounceable word; a reified RFC-0005 policy (RT3) |
| priority transport path | `backbone` | the standard term for a declared high-bandwidth/long-distance path — exactly RT3's definition (the obscure `rhizomorph`/`rhizo` is dropped) |
| execution-mode switch | `tier` | the canonical behavior *is* tiering (interpreted↔native `ExecutionMode`, RFC-0004); standard compiler vocabulary, and more precise than `dimorph` (the dense↔sparse sense is a `Swap`, S1, not this) |
| runtime-unit reclamation | `reclaim` | clear; **scope clarified** (RFC-0008 RT7): reclaims *stale runtime units*, **never memory** (LR-9 makes memory reclamation automatic; a memory-`reclaim` would contradict it) |

**Status (unchanged from RFC-0008 §4.5):** these remain **reserved vocabulary, not active
syntax** — DN-03 ratifies the *names*; activation still requires each construct's implementation
RFC (RFC-0008 §4.6 R1/R2). The names are recorded now so the name-stable vocabulary is stable at
the *right* names.

## 5. What this changes in the artifacts

- **Reserved-word set** (lexer + M-141 linter) gains: `consume`, `grow`, `for` (active Surface);
  the Runtime names `hypha`, `fuse`, `mesh`, `graft`, `cyst`, `xloc`, `forage`, `backbone`,
  `tier`, `reclaim` — **one each** — as **reserved-but-inactive** (a parse-time "reserved for
  the runtime model (RFC-0008), not yet active" diagnostic, never a silent accept).
- **`impl`** is the inherent-method keyword (conventional) when methods enter the grammar; `embody`
  is **not** reserved.
- **Lexicon Reference** updates its Surface table (drop `embody`, the `consume`/`grow` notes) and
  collapses the Runtime table to one name per concept (the alias rule is retired, §3).
- **The Example Programs Reference** `loop` note (Example #17) is superseded by `for` (§2);
  Runtime examples keep their "intent, not runnable" marking until activation.

## 6. What is deliberately left open

- **`grow` vs `derive`** spelling-overlap with the eventual macro/derive system — flagged, not
  decided; revisit when generative features are specified.
- **Activation** of any Runtime term — each needs its RFC-0008 implementation-stage RFC (R1/R2).

---

## Meta — changelog

- **2026-06-10 — Resolved.** Amends DN-02 (append-only): adopt `consume`/`grow`; decline
  `embody` (keep `impl`); reserve `for` (RFC-0007 §4.8); adopt **one name per term** (flat —
  ADR-012 §7.6's canonical+alias scheme is rejected as needless surface area, §3) and the single
  Runtime names against RFC-0008 §4.5's grounded meanings: `hypha`, `fuse` (genuine merge —
  `anastomose`/`weave` dropped), `mesh`, `graft`, `cyst` (encystment = dormant resumable form;
  `cyst(…)` constructor-style like `spore`), `xloc`, `forage`, `backbone` (was `rhizomorph`),
  `tier` (was `dimorph`), `reclaim`. Still reserved-not-active. Superseding any term is a new
  note, not a rewrite.
