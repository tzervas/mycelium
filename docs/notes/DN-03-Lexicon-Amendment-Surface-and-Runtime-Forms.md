# Design Note DN-03 — Lexicon Amendment: Surface Additions, the Alias Rule & Runtime Names

| Field | Value |
|---|---|
| **Note** | DN-03 |
| **Status** | **Resolved** (2026-06-10) — ratified by the maintainer through the DN-02 three-test gate; the agreed set feeds the grammar artifacts (RFC-0006 §4.3) |
| **Amends** | DN-02 (Resolved, append-only) — adds Surface terms and the canonical-long-form/one-short-alias rule; does **not** rewrite DN-02 (supersede-don't-edit) |
| **Feeds** | RFC-0006 (surface vocabulary); the grammar/conformance corpus; the L1 prototype reserved-word set; the M-142 formatter's canonical spelling |
| **Date** | June 10, 2026 |
| **Decides** | (a) the Surface-tier additions `consume`/`grow` (adopt) and `embody` (decline → keep `impl`); (b) `for`'s reservation (RFC-0007 §4.8 r2); (c) the **canonical-long-form + one-short-alias** rule; (d) the Runtime-tier short-form refinements (ADR-012 §7.6) against the RFC-0008-grounded meanings |
| **Depends on** | DN-02 (the naming law + three-test gate); ADR-012 §7.5/§7.6 (the flags); RFC-0007 §4.8 (`for`); RFC-0008 §4.5 (the grounded Runtime meanings — the T-map test these names are now scored against); ADR-003 (content addressing makes aliases free) |

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
defined, `Total` by construction, *provisional spelling* (KC-2-gated). It is **conventional**
(not themed): an iteration head is universal scaffolding, high on T-learn, and theming it would
cost machine familiarity for no teaching gain — the same reasoning that kept `let`/`if`/`match`
in DN-02 §3.

**Still not reserved** (DN-02 §6 reaffirmed): `while`, `loop`, `break`, `continue`, `return` —
unbounded iteration would undermine the divergence bit (RFC-0007 §4.5), and the L1 prototype
already emits *teaching diagnostics* where these words appear (parse-level juxtaposition,
check-level unknown name) pointing at recursion / `for`.

## 3. The canonical-name + one-alias rule (ADR-012 §7.6)

**Rule (normative for the lexicon).** Each themed term has **one canonical name** and **at most
one sanctioned alias**. Because identity is content-addressed (ADR-003), the two spellings denote
the **same definition — the same hash** — and the one canonical formatter (M-142, S3) renders
whichever the configured audience reads; a beginner sees `anastomose`, an expert types `fuse`,
with **zero identity cost** and no second definition. The reserved-word set contains both
spellings (a collision with either is an explicit diagnostic, never a silent shadow).

A precision the projection metaphor must not overstate: where the alias is a genuine
*abbreviation* it is literally one word shortened; where it is a chosen *synonym*
(`anastomose`/`fuse`, `mycorrhizal-network`/`mesh`, `sclerotium`/`cyst`) the two are different
words bound to one content-addressed definition — same *meaning*, same hash, **not** "the same
word projected two ways". The rule is "one canonical name + at most one sanctioned synonym", and
the alias must earn its single slot by the mnemonic/collision-free/pronounceable test (DN-02 §1).

This bounds vocabulary growth: no term may sprout a *family* of forms (the `sclrt`/`sclerotize`/
`sclerotium` sprawl is exactly what the rule forbids), and a term whose canonical name is already
short and pronounceable (`hypha`, `forage`) takes **no alias** — "at most one" includes zero.

## 4. Runtime-tier names (ADR-012 §7.6, against the RFC-0008 meanings)

Now that RFC-0008 §4.5 gives each Runtime term an operational meaning, the T-map test is
runnable. The ratified set (canonical name ⟶ **at most one** sanctioned alias):

| Concept (RFC-0008) | Canonical | Old short | **Ratified alias** | Why |
|---|---|---|---|---|
| concurrent execution unit | `hypha` | `hyph` | **none** | `hypha` is already short + pronounceable; "at most one" is *zero* here — no abbreviation earns its slot |
| lawful state fusion (RT6) | `anastomose` | `anas` | **`fuse`** | RT6 is genuine *merge* (CRDT join — two states converge into one); `weave` was rejected on **T-map** (woven threads stay distinct, implying the wrong behavior — the DN-02 `spawn`-for-a-pure-fn disqualifier); `fuse` is honest to "two become one" and equally pronounceable |
| decentralized mesh (RT5) | `mycorrhizal-network` | `cmn` | **`mesh`** | a gossip/pub-sub overlay has a *universal* name; keeping it conventional is what the gate demands (theme the unique, keep the universal). `cmn` read as "common", not a mesh |
| capability contract w/ infra | `mycorrhize` | `myco` | **`graft`** | `myco` **collides with the language family name** (Mycelium/"Myco"), forcing a rename; `graft` is pronounceable and teaches host-binding (eyes open: botanical not fungal, and implies more permanence than RT4's revocable affine contract — accepted as clean-word-beats-ugly-accurate, `symb` being a vowel-strip we penalize elsewhere) |
| durable checkpoint | `sclerotium` | `sclrt` | **`cyst`** (verb `encyst`) | `sclrt` was an unpronounceable vowel-strip; **encystment is the biological process of becoming a dormant, resistant, *resumable* form** — the single most behaviorally-accurate T-map in the Runtime set (a cyst *is* an RT2 checkpoint). Reads as noun (`a cyst`) and verb (`encyst(x)`). `dorm` was the runner-up; `cyst` wins on theme fidelity |
| explicit value movement | `translocate` | `xloc` | **`xloc`** | kept — `x` = cross/trans is an established abbreviation convention (xfer, xlat, xchg), so `xloc` *teaches* "trans-locate"; not a bare vowel-strip |
| adaptive placement policy | `forage` | `forage` | **none** | already a clean pronounceable word; a reified RFC-0005 policy (RT3). No alias earns its slot |
| priority transport path | `rhizomorph` | `rhizo` | **none** | appears rarely; the canonical name suffices (`rhizo` retired as a reflex abbreviation under "at most one = zero") |
| execution-mode switch | `dimorph` | `dimorph` | **`dimorph`** | kept (tiering = RFC-0004 `ExecutionMode`; repr-switch = `Swap`, S1) |
| runtime-unit reclamation | `reclaim` | `reclaim` | **`reclaim`** | kept — clear; **scope clarified** (RFC-0008 RT7): reclaims *stale runtime units*, **never memory** (LR-9 makes memory reclamation automatic; a memory-`reclaim` would contradict it) |

**Status (unchanged from RFC-0008 §4.5):** these remain **reserved vocabulary, not active
syntax** — DN-03 ratifies the *names*; activation still requires each construct's implementation
RFC (RFC-0008 §4.6 R1/R2). The names are recorded now so the name-stable vocabulary is stable at
the *right* names.

## 5. What this changes in the artifacts

- **Reserved-word set** (lexer + M-141 linter) gains: `consume`, `grow`, `for` (active Surface);
  the Runtime canonical names + their single aliases (`hypha`, `anastomose`/`fuse`,
  `mycorrhizal-network`/`mesh`, `mycorrhize`/`graft`, `sclerotium`/`cyst` (+ verb `encyst`),
  `translocate`/`xloc`, `forage`, `rhizomorph`, `dimorph`, `reclaim`) as **reserved-but-inactive**
  (a parse-time "reserved for the runtime model (RFC-0008), not yet active" diagnostic, never a
  silent accept).
- **`impl`** is the inherent-method keyword (conventional) when methods enter the grammar; `embody`
  is **not** reserved.
- **Lexicon Reference** updates its Surface table (drop `embody`, the `consume`/`grow` notes), the
  Runtime table (the ratified names), and adds the alias rule (§3).
- **The Example Programs Reference** `loop` note (Example #17) is superseded by `for` (§2);
  Runtime examples keep their "intent, not runnable" marking until activation.

## 6. What is deliberately left open

- **`grow` vs `derive`** spelling-overlap with the eventual macro/derive system — flagged, not
  decided; revisit when generative features are specified.
- **Activation** of any Runtime term — each needs its RFC-0008 implementation-stage RFC (R1/R2).
- **The alias projection UX** (which audience sees which spelling, how the formatter is
  configured) — an M-142/L3 tooling matter, not a vocabulary decision.

---

## Meta — changelog

- **2026-06-10 — Resolved.** Amends DN-02 (append-only): adopt `consume`/`grow`; decline
  `embody` (keep `impl`); reserve `for` (RFC-0007 §4.8); ratify the **canonical-name +
  one-alias** rule (with the abbreviation-vs-synonym precision, §3) and the Runtime names against
  RFC-0008 §4.5's grounded meanings — `anas`→`fuse` (genuine merge; `weave` rejected on T-map),
  `cmn`→`mesh`, `myco`→`graft`, `sclrt`→`cyst`/`encyst` (encystment = dormant resumable form);
  `xloc`/`dimorph`/`reclaim` kept; `hypha`/`forage`/`rhizomorph` take **no** alias ("at most one"
  = zero). Still reserved-not-active. Superseding any term is a new note, not a rewrite.
