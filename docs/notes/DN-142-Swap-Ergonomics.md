# Design Note DN-142 — Swap Ergonomics: Ambient Policy, Handle+Sink Certificates, and `to:` Elision

| Field | Value |
|---|---|
| **Note** | DN-142 |
| **Status** | **Draft** (2026-07-18; captures the maintainer's steered decisions in `PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.1 P1-Q1..Q4 for ratification. No ratification date — Draft to Accepted is the maintainer's append-only call, house rule H1/H2; this note does not self-ratify.) |
| **Audience** | Implementing agent(s) executing Phase-2 waves W-A/W-B (`PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §5); the maintainer, at ratification. |
| **Decides** | *Proposes, for ratification (already maintainer-steered — this note is the citable capture, not a fresh proposal):* the `policy: ambient` spelling law and its RFC-0012/RFC-0034 §6-style scoped resolution (P1-Q1); the handle-plus-sink certificate architecture that closes the cert-discard gap without widening `SwapEngine` (P1-Q2); the three mechanical gates on `to:` elision (P1-Q3); and the kernel/std-surface "regime layer reconcile" table (P1-Q4). |
| **Grounds** | `docs/planning/design-steer-2026-07-17/PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.1 (P1-Q1..Q4, the binding steer text this note captures) and §2 (the width-canon corrective this note's §8 cross-references) · `docs/planning/gap-analysis-2026-07-16/DESIGN-01-SWAPS-AND-POLICY.md` §3-§7 (the pack this steer answers) · RFC-0002 §2-§5 (the certificate, the checker, the bijection semantics, the legal-pair table) · RFC-0005 §2-§5 (`SelectionPolicy`/`PolicyRef`, mandatory EXPLAIN, the "one mechanism, [multiple] sites" rule) · RFC-0012 §4.1/§4.3/§4.6 (the ambient-is-a-reified-scoped-selection pattern, the never-silent resolution errors, the meaning-preservation differential) · RFC-0034 §6 (the `global ⊐ phylum ⊐ nodule` most-specific-wins scoping this note's §3.2 resolution law reuses) · DN-16 §7-Q4 / 2026-06-19 and 2026-06-21 changelog entries (the landed `mycelium-std-swap` surface pinned `Result`-only) · `docs/spec/stdlib/swap.md` §3/§4/§7 (the landed surface, the open ergonomics questions this note answers) · `docs/spec/swaps/binary-ternary.md` §2-§4 (the kernel `Option`-typed `dec`, cited in §6 below) · `crates/mycelium-cert/src/mode.rs:152-219` (`ModeGatedSwapEngine`, the cert-discard site) · `crates/mycelium-core/src/meta.rs:90-149` (the `Meta` field-addition precedent — `cert_mode`/`wrapping_opt` beside `policy_used`, all excluded from the content hash). |
| **Task** | Phase-1 ratifiable capture, item 1 of `PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §4 ("Swap Ergonomics DN"). Tracking id **not minted for this note itself** — it is the citable design vehicle, not an implementation issue. **Free-ID check (mitigation #1):** `docs/notes/` carries no `DN-141` or `DN-142` file as of 2026-07-18. `DESIGN-02-TAGS-META-AND-CONTAINMENT.md` names `DN-142` as one candidate promotion target for **pack 02** ("containment topology," its own header line 11 and §8 item 5), but the steer explicitly declines that use ("DN-142 not minted (speculative granularity)," `PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.1 P2-Q5) — pack 02 instead ratifies as a **DN-141 rewrite**. `DN-142` is therefore genuinely free and is minted here for the *unrelated* Swap Ergonomics DN P1-Q4 calls for, not for pack 02's containment topology. |
| **Related** | DN-141 rewrite (pack 02, containment — a sibling Phase-1 capture, not authored by this note) · the `LanguageRetentionPolicy` spec (Phase-1 item 5 — the fourth ambient instance, §3.1 below) · `docs/spec/stdlib/swap.md` (the surface this note's ergonomics land on) · `docs/spec/swaps/binary-ternary.md` (the bijective swap class §5's elision gates apply to first; also amended 2026-07-18 for the W-1 width corrective, a sibling capture this note cross-references in §8 but does not restate). |
| **Definition of Done (this note's ratification gate)** | (1) the maintainer moves Status Draft to Accepted per house rule H1/H2 (append-only; no agent does this); (2) §3's ambient-policy conformance law (`expand(policy: ambient) ≡ longhand`, identical content hash) has a landed CI golden per RFC-0012 §4.6/NFR-7 before `policy: ambient` ships in wave W-B; (3) §4's `Meta.cert` field plus mode-gated store lands with the cap from the `LanguageRetentionPolicy` spec (Phase-1 item 5) before any `certified`-mode swap emits a cert handle; (4) §5's three elision gates each have a positive and a negative differential case before X9 ships `to:` elision. |

> **Posture (transparency rule / VR-5 / G2 / house rule #4).** This note **captures** binding maintainer
> steers already recorded in `PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.1 — it is the citable
> ratification vehicle P1-Q4 calls for, not a fresh design proposal. Every normative sentence below
> traces to that steer table, to the RFC/ADR/DN it cites, or to source read at `file:line` (marked
> inline). Nothing here is `Accepted`; per house rule H2 an agent never asserts a ratification date.
> Where this note extrapolates beyond the steer's literal words (e.g. the `Meta.cert` field shape,
> drawn from the existing `cert_mode`/`wrapping_opt` pattern), that extrapolation is marked `Declared`
> and flagged, not presented as already-decided.

---

## §1 Purpose

`std.swap`'s spec (`docs/spec/stdlib/swap.md` §7-Q2/Q3) and the DESIGN-01 gap-analysis pack both left the
call-site ergonomics of swaps open — policy spelling, certificate packaging, and destination elision —
pending the maintainer's steer. The maintainer has now steered all four DESIGN-01 §6 questions
(P1-Q1..Q4, `PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.1). This note is the **dedicated Swap
Ergonomics DN** P1-Q4 calls for — the citable home for that capture, replacing "extend M-540" as the
vehicle (DESIGN-01 §6 Q4, resolved: dedicated DN). M-540's per-ring ergonomics pass (already landed,
`issues.yaml` `status:done`) **consumes** this DN going forward; it was never the design vehicle for the
questions answered here.

## §2 User story and scope

*As an author writing a swap-heavy Mycelium program, I want a short, honest spelling for "use the policy
this scope already declared" and a certificate I can inspect without threading it through every call, so
that I get the ergonomics of an implicit default without losing the never-silent audit trail S1/G2
require.*

*As a reviewer, or the checker itself, I want every elided spelling to expand to byte-identical L0 (same
content hash) and every certificate discard to instead be an inspectable handle, so that ergonomics never
become a black box (RFC-0012 I1/I2, ADR-006).*

In scope: the `policy: ambient` spelling and its resolution law (§3); the certificate handle-plus-sink
architecture (§4); the `to:` elision gates (§5); the kernel/std-surface regime reconcile (§6); a pointer
to the A1 legal-pair matrix (§7); and the W-1 width-canon cross-reference (§8). Out of scope: the DN-141
grade/mode/typing containment work (pack 02, its own DN), the RFC-0013 diagnostic-envelope amendment and
the `LanguageRetentionPolicy` spec (separate Phase-1 items), and any code landing — this note is design
capture, not an implementation PR (house rule H1: no code lands with a Draft DN).

---

## §3 The `policy: ambient` spelling law (P1-Q1)

### §3.1 Vocabulary

- The spelling is **`policy: ambient`**. This is the corpus's ratified word for a *declared, scoped*
  default (`PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.1 P1-Q1): RFC-0012 uses it for the paradigm
  default, and RFC-0034 §6 already reuses the same ambient/scoped-override mechanism for `CertMode`
  resolution; `policy: ambient` is the **third instance** of the one mechanism (the
  `LanguageRetentionPolicy` spec, Phase-1 item 5, is the fourth) — a generalization of RFC-0005 §4's
  "one mechanism, [multiple] sites" rule.
- The **explicit catalog-name spelling** (`policy: <name>`) stays legal at every call site — ambient is
  an ergonomic default, not a replacement for a named policy (DESIGN-01 §4.1, rows A2/A3).
- `policy: _`, `policy: auto`, and `policy: default` are **rejected vocabulary**. `default` in particular
  imports an unexamined-fallback prior that contradicts ADR-006's reified-selection mandate and G2's
  never-silent floor (`PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.1 P1-Q1 rationale). Phase-0 audit
  item G-6 greps for all three strings across code/docs/tests and requires zero post-capture hits.

### §3.2 Resolution law

`policy: ambient` resolves through the same **scoped precedence** RFC-0034 §6 already established for
`CertMode` — `global ⊐ phylum ⊐ nodule`, most-specific-wins, reusing RFC-0012's ambient plus
scoped-override mechanism (no new scoping machinery) — evaluated against a **declared ambient policy** at
the resolving scope plus the swap's **catalog pair-registration** (the `std.swap.policy` catalog,
DESIGN-01 §4.1 row A2).

- **Unresolved is a hard error, never a fallback.** If no scope declares an ambient policy for the pair in
  question, resolution fails with an explicit error ("no ambient policy declared for this pair in scope")
  — never a silent substitute (`PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.1 P1-Q1 implementation
  notes; mirrors RFC-0012 §4.3's `UnresolvedAmbient`/`UnresolvedWidth` pattern — no implicit global
  fallback).
- **Elision is spelling only.** Exactly as RFC-0012 §4.3 invariant I1 (the ambient emits no `Swap`) and
  RFC-0005 §3 (every swap records the `PolicyRef` it used) already require, L0 always stores the
  **resolved** `PolicyRef` — never the literal token `ambient`. This is "resolve-and-record" (DESIGN-01
  §4.1 rule 1).
- **Always record the resolved `PolicyRef` hash plus the EXPLAIN origin** — one of `declared@nodule`,
  `declared@phylum`, or `catalog` (`PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.1 P1-Q1). This is the
  first-fault `policy_resolve` emitter site (DESIGN-01 §4.3); the envelope schema itself is a separate
  Phase-1 item (the RFC-0013 amendment, DESIGN-03 §3) — this note names the site, it does not define the
  envelope.

### §3.3 Conformance law

`expand(policy: ambient) ≡ longhand`, with an **identical content hash** — the same free-conformance
proof RFC-0012 §4.6/NFR-7 already delivers for the paradigm ambient (identical L0 implies identical
`content_hash`, RFC-0001 §4.6). This is a **CI golden**, not a claim: a corpus of swap sites written both
with `policy: ambient` and with the fully-resolved longhand `policy: <name>` must elaborate to the same
hash, mirroring `tests/ambient.rs`'s existing methodology (RFC-0012 §4.6). Landing this golden is DoD item
(2) above and gates `policy: ambient` shipping in wave W-B (`PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md`
§5).

---

## §4 Handle-plus-sink certificate architecture (P1-Q2)

### §4.1 Disposition: explicit `Swapped` stays the authoring model until X15

`std.swap`'s landed surface (`docs/spec/stdlib/swap.md` §3, pinned to
`crates/mycelium-std-swap/src/lib.rs` per the DN-16 2026-06-19 resolution) returns
`Swapped { value: Value, cert: SwapCertificate }` — an explicit second return, never an implicit ambient
value. `docs/spec/stdlib/swap.md` §7-Q2 left "cert ambient" open; DESIGN-01 §6 Q2 asked whether to make it
the default authoring model now or keep explicit `Swapped` until the diagnostic bus lands. The steer
**confirms the standing disposition**: keep explicit `Swapped` until X15 (the first-fault bus) lands
(`PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.1 P1-Q2). `swap.md` §7-Q2's "default to explicit
`Swapped<T>` until §8-Q3 resolves" is therefore **not superseded** — it is reaffirmed with a concrete gate
(X15), not a date.

### §4.2 Closing the cert-discard gap without widening `SwapEngine`

The concrete gap: `ModeGatedSwapEngine`'s `SwapEngine::swap` (`crates/mycelium-cert/src/mode.rs:202-213`)
computes the full `GatedSwap` (value plus certificate plus check verdict, via `swap_gated`) but the trait
method's contract returns only the `Value` — the doc comment at `mode.rs:218` says so explicitly: "keeps
the certificate (the trait's `swap` discards it)." Widening the `SwapEngine` trait to return the
certificate would re-import the threading ceremony §4.1 just deferred
(`PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.1 P1-Q2 rationale). The steered fix instead **closes the
gap via a diagnostic sink plus a certificate handle**:

- **A certificate handle beside `policy_used`.** `Meta` already carries
  `policy_used: Option<ContentHash>` (`crates/mycelium-core/src/meta.rs:97`) and two later-added,
  content-hash-excluded fields following the identical pattern (`cert_mode`, `wrapping_opt`,
  `meta.rs:98-109`, both documented "excluded from the content hash... the exclusion is by construction").
  `Meta.cert: Option<ContentHash>` is proposed **beside** `policy_used`, on the same excluded-from-hash
  basis (RFC-0001 §4.6) — a value's identity never depends on whether or how it was certified. The steer's
  own implementation-notes column states this directly: "Add `cert: Option<ContentHash>` to `Meta` beside
  `policy_used` (excluded from content hash like all `Meta` — RFC-0001 §4.6)"
  (`PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.1 P1-Q2). The precise field placement relative to the
  other `Meta` fields is `Declared` — an extrapolation from the existing extension pattern, flagged as
  such, not a literal quote.
- **Certificate bodies go to a mode-gated content-addressed store.** Emit modes only (i.e. not `fast`'s
  default), capped per the `LanguageRetentionPolicy` spec's §1.4 P4-Q1 dual caps (Phase-1 item 5 — a
  sibling capture this note does not restate). The handle (`ContentHash`) is what rides `Meta`; the body
  lives in the store, addressable by that hash.
- **First-fault emitter siting, not schema.** The `ModeGatedSwapEngine`'s `NotValidated` branch
  (`mode.rs:206-210`, already never-silent — it returns an `EvalError`, never a pass) is the **first
  `swap_check` emitter site**; the validated path emits a first-fault-bus `swap_check` crumb plus the cert
  handle. This note names the site; the envelope schema itself is the separate RFC-0013-amendment Phase-1
  item, not defined here.
- **No `SwapEngine` widening.** The trait's `swap` signature is unchanged; `swap_gated` (already present,
  `mode.rs:191-199`) remains the full-fidelity path for callers that want the certificate directly, and
  the new handle is how a `swap`-only caller still gets an inspectable pointer to it.
- **X8 (cert-ambient) reduces to sugar over this later.** Once the handle-plus-sink lands, a future
  cert-ambient surface (X8, held per `PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §5 wave W-E) is pure
  ergonomic sugar over an already-present handle — it does not need new machinery.

### §4.3 Joint gate with the containment pack

Per DESIGN-01 §4.2's joint gate (unchanged by this steer): if a certificate is ever made ambient, a
failed check must still surface as a typed failure plus a first-fault event — never an `Exact` success.
The `NotValidated` branch already enforces this at the trait boundary (`mode.rs:206-210`); the
handle-plus-sink design in §4.2 does not relax it.

---

## §5 `to:` elision (P1-Q3)

`to:` elision is **allowed**, shipping as **AX-sugar (X9)**, **after walls** (i.e. after the AX-iso wave
W-E's containment boundaries land — `PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.1 P1-Q3, §5). It is
gated by **three** mechanical conditions, none of which is optional:

1. **Uniqueness-or-refuse.** Exactly one legal pair (RFC-0002 §5) is consistent with the expected type at
   the elided call site; if more than one legal pair matches, the elision is a **hard error** listing the
   candidates as suggestions that are **never auto-applied** — the X11 posture, an analog of rustc's
   `Applicability::MaybeIncorrect`-style suggestions (cited in the DESIGN-01/steer discussion) that a tool
   may *offer* but never silently *apply*.
2. **Elaboration-hash CI goldens.** `elided ≡ longhand` under the same content-hash test discipline as
   §3.3's ambient-policy golden — a second, independent conformance corpus, not a reuse of the same test
   (the two elisions are orthogonal: one elides the policy identity, the other the destination `Repr`).
3. **Regime typing from the resolved pair, never the spelling.** The checker computes the result type
   (total / `Option` / `Result`, per §6 below) from the **resolved** `(R_src → R_target)` pair — a partial
   pair types `Result` (or `Option`, per the layer) whether or not `to:` was written. Eliding the
   destination must never let a partial pair masquerade as total by hiding behind the elided spelling.

Each gate is independently testable and each must have a positive and a negative differential case before
X9 ships (DoD item (4) above).

---

## §6 Regime layer reconcile (P1-Q4)

A residual cross-layer inconsistency, flagged by the steer for explicit capture so A1/A5 wording (the
legal-pair matrix and its regime-typing rule) cannot drift: **the kernel and the std surface disagree on
which type carries "off the image."**

| Layer | Fallibility shape for `LosslessWithinRange` (binary/ternary) | Source |
|---|---|---|
| **Kernel** (`enc`/`dec`, `mycelium-core`/`mycelium-cert`) | `dec : Tern_m → Option Bin_n` — **`Option`-typed**; `None` off the image, explicit and never silent | RFC-0002 §4 ("`dec : Tern_m → Option Bin_n`"); `docs/spec/swaps/binary-ternary.md` §3 (same signature, worked example §5) |
| **Std surface** (`mycelium-std-swap`, `tern_to_bin`) | `Result<Swapped, SwapError>` — **`Result`-only**; `Err(OutOfRange)` off the image, no `Option` return anywhere in the surface | DN-16 §7-Q4 resolution, 2026-06-19 ("there are **no** `Option` returns — every fallible op is `Result`"); `docs/spec/stdlib/swap.md` §3 header ("every fallible op returns `Result` (no `Option`)") |

Both are honest and never-silent — the divergence is **which explicit failure type**, not whether failure
is explicit. DN-16's 2026-06-21 re-audit already logged this exact drift as a "minor spec-text drift"
(`tern_to_bin` returns `Err(OutOfRange)` where §4's prose says `Option`/`None`) without resolving *why*
the layers differ. This note states the **why**, so future ergonomics work (elision, ambient) does not
silently pick one shape and call it the whole story:

- The **kernel** signature is the one RFC-0002 §4 proves (P1/P2, SMT-dischargeable for fixed widths) —
  `Option` is the natural shape for "this function may have no answer" at the bijection-math layer, and
  changing it would touch a proof obligation (M-121), not just a wrapper.
- The **std surface** is a library convenience layer (`docs/spec/stdlib/swap.md` §1: "Ring 1... a
  certificate consumer... adds no trusted code") that standardized on `Result` uniformly across *all* of
  `std.swap`'s exported ops (bijective and bounded alike) so callers do not need two different fallibility
  idioms depending on swap kind — a DN-16-pinned ergonomic choice, not a proof-shape choice.

**Rule (normative for future A5/regime-typing wording).** A `Result`-vs-`Option` choice is a
**layer-presentation** decision, never a **regime** decision. The regime (`LosslessWithinRange`,
`Bounded`, etc., RFC-0002 §5) determines *that* a pair is partial; which explicit failure type a given
layer surfaces it as is that layer's own, separately-stated convention. §5's elision gate 3 ("regime
typing from the resolved pair") binds to the **regime**, not to either layer's chosen wrapper — an elided
`to:` must still type `Result` at the std surface and would still be provable `Option`-shaped at the
kernel, without the two disagreeing about which pairs are partial.

---

## §7 A1 legal-pair matrix (reference)

DESIGN-01 §4.1 row A1 ("legal-pair matrix in checker") is a **checker materialization** of RFC-0002 §5's
legal-pair table — this DN does not restate that table (restating it would fork a normative source;
RFC-0002 §5 remains canonical). The checker's job is early refusal of illegal pairs (`legal_pair_refuse`,
DESIGN-01 §4.3) using exactly RFC-0002 §5's rows; any pair the checker admits must already be one of
RFC-0002 §5's legal rows, and any pair RFC-0002 §5 marks a type error / rejected / explicit error / never
silent must refuse at the checker, not later at runtime.

---

## §8 W-1 width-canon cross-reference

The binary width canon corrective (`PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §2, "corrective W-1") is
captured separately, as its own append-only amendments, in `docs/spec/swaps/binary-ternary.md` and
`docs/spec/stdlib/swap.md` (this same batch, dated 2026-07-18). It intersects this DN at one point: §5's
`to:`-elision uniqueness gate and §3's ambient-policy catalog both presume a **canonical** pair per
paradigm crossing to resolve unambiguously where more than one legal width could apply; W-1 fixes that
canon at `Binary{64}` (`Binary{32}` recognized fallback), so the elision/ambient machinery designed here
should read "the canonical pair" as W-1 defines it once enablement item E-W1 lands, not the legacy
`Binary{8}`/`Ternary{6}` de facto default it replaces. No other coupling — the two captures are otherwise
independent and this note does not restate W-1's content.

---

## §9 Open questions / FLAGs

- **Certificate-store cap sizing** is deferred to the `LanguageRetentionPolicy` spec's sizing pass
  (Phase-1 item 5, `PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.4 P4-Q1) — this note names the
  handle-plus-sink shape but does not set the cap.
- **The RFC-0013 envelope schema** for the `policy_resolve`/`swap_check` first-fault sites named in §3.2
  and §4.2 is the separate Phase-1 item 3 amendment (DESIGN-03 §3.3a's site-kind catalog) — this note
  names the sites, not the schema.
- **X8 cert-ambient's exact surface** is left to its own future design pass (§4.2, last bullet) — only its
  reduction-to-sugar property is stated here.

## §10 Grounding summary

`PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.1 (P1-Q1..Q4, the binding steer this note captures) and
§2 (the W-1 corrective cross-referenced in §8) · `DESIGN-01-SWAPS-AND-POLICY.md` §3-§7 (the pain points,
the ranked package, the DoD this note helps close) · RFC-0002 §2-§5 (checker, certificate content,
bijection semantics, legal-pair table) · RFC-0005 §2-§5 (`SelectionPolicy`/`PolicyRef`, EXPLAIN,
one-mechanism-multiple-sites) · RFC-0012 §4.1/§4.3/§4.6 (ambient as reified scoped selection; never-silent
resolution errors; the meaning-preservation differential this note's §3.3 golden mirrors) · RFC-0034 §6
(the `global ⊐ phylum ⊐ nodule` scoping law this note's §3.2 resolution law reuses) · DN-16 (the landed
`mycelium-std-swap` `Result`-only surface pin, §6 of this note) · `docs/spec/stdlib/swap.md` §3/§4/§7 ·
`docs/spec/swaps/binary-ternary.md` §3/§4 · source: `crates/mycelium-cert/src/mode.rs:152-219`,
`crates/mycelium-core/src/meta.rs:90-149`.

## Meta — changelog

- **2026-07-18 — Draft.** Mints DN-142 as the dedicated Swap Ergonomics DN called for by
  `PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §4 item 1 (P1-Q4). Captures the four steered decisions
  (P1-Q1..Q4): the `policy: ambient` spelling law with its RFC-0034 §6-style `global ⊐ phylum ⊐ nodule`
  resolution precedence, hard-error-on-unresolved rule, and RFC-0012 §4.6-style content-hash conformance
  golden; the handle-plus-sink certificate architecture that closes the `ModeGatedSwapEngine` cert-discard
  gap (`mode.rs:218`) via a `Meta.cert: Option<ContentHash>` handle beside `policy_used` plus a mode-gated
  content-addressed store, without widening the `SwapEngine` trait, while reaffirming explicit `Swapped`
  as the authoring model until X15; the three mechanical `to:`-elision gates (uniqueness-or-refuse,
  elaboration-hash goldens, regime-from-resolved-pair typing); and the kernel-`Option`-vs-std-`Result`
  regime layer reconcile table (RFC-0002 §4 vs the DN-16 pin), stated as a layer-presentation distinction
  that must never be conflated with the regime itself. No status is moved to Accepted (house rule H1/H2);
  no code lands with this note. Append-only henceforth.
