# Spec (Draft) — `LanguageRetentionPolicy`: bounding the language/runtime's own audit memory

| Field | Value |
|---|---|
| **Status** | **Draft** (2026-07-18 — captured from maintainer steer P4-Q1..Q5; not yet ratified). Recorded per the Phase-1 ratifiable-capture worklist; implementation (the concrete struct + the mode-gated cert store + the sizing pass) is Phase-2/Phase-W-D work, not this capture (§12). |
| **Scope** | The `LanguageRetentionPolicy` record: what bounds the **language/runtime's own** perpetual in-process surfaces (swap certificates, the `EXPLAIN`-generation signal incl. the RFC-0013 first-fault bus, `Meta`/provenance on live values) so a long-running process cannot OOM itself on its own honesty machinery, while every drop/compaction/export stays non-silent (G2) and no summarization ever upgrades a grade (VR-5). |
| **Out of scope** | Application logs, product analytics, user-defined append-only stores, CI artifacts, and the append-only decision corpus (ADR/RFC/DN) — all **app/ops/tooling** owned, not language-internal (DESIGN-04 §1). |
| **Depends on** | [RFC-0012](../rfcs/RFC-0012-Ambient-Representation-and-Scoped-Overrides.md) §4.1/§6-analog (the scoped-ambient resolution mechanism this policy reuses as its fourth instance); [RFC-0013](../rfcs/RFC-0013-Structured-Diagnostics-and-Reified-Error-Policy.md) + its 2026-07-18 Amendment A1 (the first-fault bus this policy bounds, L4 below); [RFC-0034](../rfcs/RFC-0034-Tunable-Certification-and-Transparency-Modes.md) §5 (`CertMode`) / §6 (the `global ⊐ phylum ⊐ nodule` scoping this policy's resolution mirrors) / §7 (generation ≠ consumption — this policy bounds *retention*, a third, orthogonal axis); [RFC-0002](../rfcs/RFC-0002-Swap-Certificate-and-Split-Regime.md) (swap certificates, L1 below); [RFC-0001](../rfcs/RFC-0001-Core-IR-and-Metadata-Schema.md) §4.6 / ADR-003 (content-addressing; a policy is metadata, excluded from content hash exactly like `Meta`); KC-3 (no kernel log daemon — RP6); G2 (never-silent drop/compact); VR-5 (never upgrade a grade to make a summary look stronger). |
| **Grounds on** | `docs/planning/design-steer-2026-07-17/PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.4 (P4-Q1..Q5, binding maintainer steers) and §4 item 5 (this spec's own capture instruction); `docs/planning/gap-analysis-2026-07-16/DESIGN-04-LEDGER-RETENTION-AND-OFFLOAD.md` (the full design source, §1–§11); `docs/planning/design-steer-2026-07-17/CITATIONS-DESIGN-STEER-2026-07-17.md` [C29]–[C33] (external evidence anchors cited below). |

---

## 0. Placement note (why this file is here, not `docs/spec/diagnostics/retention.md`)

`docs/spec/` was checked before minting a path: `stdlib/` is reserved for the RFC-0016 `std` module
taxonomy (a per-module spec for a `crates/mycelium-std-*` crate — see `docs/spec/stdlib/README.md`
§1); `swaps/` holds swap-mechanism specs (`binary-ternary.md`); `grammar/`/`schemas`/`api/` are
generated/structural. `LanguageRetentionPolicy` is **none of these** — DESIGN-04 §5.3 is explicit that
it is "a **runtime** policy … **not** an app logging config," analogous to `CertMode` (RFC-0034 §5),
which itself lives in `docs/rfcs/`, not `docs/spec/stdlib/`. There is also no existing
`docs/spec/diagnostics/` directory, and minting one for a policy whose scope spans certs (L1),
`EXPLAIN` signal (L2), and `Meta` (L3) — not diagnostics (L4) alone — would misname the directory for
three of its four surfaces. The closest existing convention is `docs/spec/`'s **top-level,
Title-Case-hyphenated contract docs** (`Security-Checks-Contract.md`, `Surface-Stability-Declaration.md`,
`Spore-Build-and-Publish-Contract.md`) — cross-cutting language/runtime contracts that are not tied to
one stdlib module. This file follows that convention: **`docs/spec/Language-Retention-Policy.md`**.
Flagged for the integrating parent in case a dedicated `docs/spec/runtime/` subdirectory is later
judged the better long-term home once more runtime-policy specs accumulate (this is the first one).

## 1. User stories

- **As a Mycelium runtime operator** running a long-lived `certified` colony, I want the language's own
  audit/diagnostic memory (certs, `EXPLAIN` signal, `Meta`, first-fault records) to stay bounded and
  never silently dropped, so my process does not OOM itself on its own honesty machinery while I still
  get a non-silent, `EXPLAIN`-able account of what was compacted or exported.
- **As a developer** running the `fast` default, I want small, sane retention defaults out of the box —
  first-fault localization without paying `certified`-grade memory cost — and a warning, not a hard
  failure, if I never think about retention at all.
- **As an auditor** who needs an admissible record of what a `certified` run did, I want the policy's
  exported evidence to be independently **verifiable** (content-hashed digests + inclusion proofs)
  rather than asked to trust an in-process claim that "everything was retained."

## 2. In-scope surfaces (DESIGN-04 §3 inventory, condensed)

This policy governs exactly the **language-internal perpetual** surfaces — nothing an application
chose to log is in scope (DESIGN-04 §1):

| ID | Surface | Growth driver | Bounded by this policy? |
|---|---|---|---|
| **L1** | Swap **certificates** (RFC-0002) | per successful/attempted swap | yes — `hot_cert_handle_cap` (§5) |
| **L2** | **Inspectability signal** (`EXPLAIN` input; RFC-0034 §7 generation) | per selection/swap/mode event | yes, qualitatively (RP1/RP2, §4); no dedicated numeric field yet — **FLAGGED**, §11 |
| **L3** | **`Meta`/provenance** on live `Value`s | per value + meet | yes, qualitatively (mode-aware; RP2); no dedicated numeric field yet — **FLAGGED**, §11 |
| **L4** | **First-fault bus** (RFC-0013 + Amendment A1) | per refuse (+ optional crumbs) | yes — `hot_first_fault_cap` (§5) |
| L5–L7 | Live value graph (GC, not a ledger), spore/deploy identity (compile-time, not a long-run ledger), future language-event streams | — | not this policy's concern today (L5/L6); L7 inherits this policy at birth once it exists (DESIGN-04 §6) |

## 3. The `LanguageRetentionPolicy` record (DESIGN-04 §5.3 fields)

A **runtime** policy, RFC-0005-shaped (a reified, inspectable, content-addressed selection — like a
`CertMode`/policy-catalog declaration), **not** an app logging config:

| Field | Meaning |
|---|---|
| `hot_first_fault_cap` | Max first-fault records (L4) retained in-process (dual cap: count **and** bytes — P4-Q1). |
| `hot_cert_handle_cap` | Max swap-certificate **handles** retained (L1); `0` = only the live-attached handle, no extra ring. |
| `epoch` | Count-or-time boundary for hot → warm compaction. |
| `on_overflow` | `drop_oldest` \| `compact_to_digest` \| `export_then_drop` — what happens at a cap/epoch boundary. |
| `export` | Optional host hook for **language-audit blobs only** (§7); absent = no export configured. |

**Resolution mirrors `CertMode`.** All five fields are overridable via a declared
`LanguageRetentionPolicy` at the same granularity RFC-0034 §6 uses (§4 below); none of them enters a
value's content hash (ADR-003) — the policy is dynamic `Meta`-class metadata, exactly as `Meta.policy_used`
and the active `CertMode` already are (RFC-0001 §4.6).

## 4. Resolution — the fourth RFC-0012 scoped-ambient instance

The steer names this policy's resolution mechanism explicitly (handoff §1.1 P1-Q1 implementation
notes): *"RFC-0034 §6 already reuses the ambient mechanism for `CertMode`; policy becomes the third
instance, retention (§1.4) the fourth."* The lineage, in order:

1. **RFC-0012** itself — the original ambient **paradigm** default (`default paradigm P`).
2. **RFC-0034 §6** — `CertMode` scoping (`global ⊐ phylum ⊐ nodule`, most-specific-wins, declared via a
   `@certification` attribute).
3. **`policy: ambient`** (handoff P1-Q1; the pending Swap Ergonomics DN, not yet minted) — selection
   policy resolution over the same precedence.
4. **`LanguageRetentionPolicy`** (this spec) — the fourth instance, same mechanism, no new scoping
   machinery invented.

**Resolution law (reusing RFC-0012 §4.1 / RFC-0034 §6, unchanged mechanism):** `global ⊐ phylum ⊐
nodule`, resolving most-specific-wins, declared at the project (`mycelium-proj` manifest), phylum, or
nodule level. Declaration site: an attribute mirroring `@certification` — sketched here as `@retention`
(**Declared** name; not a ratified surface, and not this spec's to fix — the concrete attribute
grammar is Phase-2/surface-layer work). No hash perturbation (ADR-003): the resolved policy is metadata,
excluded from content hash exactly as `CertMode`/`policy_used` already are.

**One deliberate divergence from instances 2 and 3 — flagged, not silently harmonized.** `CertMode`
and `policy: ambient` both hard-error when unresolved ("no ambient policy declared for this pair in
scope," handoff P1-Q1). `LanguageRetentionPolicy` does **not** inherit that hard-error default — P4-Q2
(§6 below) is a **bounded default + warning**, escalating to required-explicit only under a declared
audit obligation. The scoping *mechanism* is shared (instance four of the same precedence); the
*unresolved-case* policy is not, and this spec states that difference explicitly rather than assuming
uniformity across the four instances.

## 5. P4-Q1 — dual caps (count + bytes), per-surface, mode-scaled — `Declared` placeholder defaults

**Guarantee tag: `Declared`.** These are placeholder numbers, not yet sized against real workloads —
honest per VR-5: they carry no checked or empirical basis today. They ride the honesty lattice
explicitly: `Declared` → `Empirical` only after the Phase-2 sizing pass (DESIGN-04 §10 P0 item —
"Empirical inventory of L1–L4 sizes in current Rust runtime under load"; static struct-footprint sizing
plus a synthetic load run, per handoff P4-Q1 implementation notes). Anchors: JDK Flight Recorder's
always-on low-overhead maxsize/memorysize defaults and Prometheus's time+size dual cap
([C29], [C30] — version-specific numbers, treated as anchors not constants, per the citation doc's own
caveat).

| Mode | `hot_first_fault_cap` (L4) | `hot_cert_handle_cap` (L1) | warm epochs |
|---|---|---|---|
| **`fast`** | 64 records / 256 KiB | 0 | not specified by the steer — **FLAGGED**: `fast`'s tiny ring plausibly has no warm tier at all (nothing to compact), but this is an *inference*, not a steered number; left open for the sizing pass rather than fabricated here |
| **`balanced`** | not specified by the steer — **FLAGGED**, §11 | not specified by the steer — **FLAGGED**, §11 | not specified by the steer — **FLAGGED**, §11 |
| **`certified`** | 1024 records / 8 MiB hot | 256 | 16 |

All numbers are **overridable** per §4's resolution (the whole point of a `LanguageRetentionPolicy`
declaration is to override the mode default). L2 (`EXPLAIN`-signal storage) and L3 (`Meta` on live
values) have **no dedicated numeric cap field** in the DESIGN-04 §5.3 record as captured — they are
bounded only *qualitatively* today (RP1/RP2, §8 below); see §11 for this residual.

## 6. P4-Q2 — bounded default + warning when unset; required-explicit only under a declared audit obligation

**Normative rule (captured, `Declared`):** absent an explicit `LanguageRetentionPolicy` declaration at
any scope, the runtime uses the §5 mode-default caps and emits a **non-silent warning** (never a hard
`check` error) that no policy was declared. This is the **JFR bounded-default** posture, deliberately
avoiding the JDK-8 unlimited-default footgun ([C29]). The one escalation path: a colony/spore that
**declares an audit obligation** (a separate, explicit declaration — not this policy's own presence)
makes an absent `LanguageRetentionPolicy` a **`check` error** instead of a warning. This mirrors G2's
general shape (a reduced-coverage/absent-declaration state is always reported, never silently passed —
`/security-review`'s "a skip is named, never a silent pass" posture, `docs/spec/Security-Checks-Contract.md`
§2) applied to retention specifically. The concrete "audit obligation" declaration surface is
**not fixed here** — it is a Phase-2/surface-layer question, FLAGGED §11.

## 7. P4-Q3 — host FFI export hook (C-ABI; at-least-once ack; drop counters `EXPLAIN`ed)

**Direction (captured, `Declared`):** the export hook is **host FFI first** — matching the C-backend
bootstrap and KC-3's no-daemon posture (no kernel logging dependency is introduced; the hook is a
process-boundary callback, not a new trusted subsystem). The signature below is an **illustrative
sketch** (mirrors the "illustrative signatures, not a committed surface" convention used elsewhere in
this spec family, e.g. `docs/spec/stdlib/diag.md` §3) — not a committed ABI:

```c
/* Illustrative sketch only — NOT a committed ABI (P4-Q3). */

/* At-least-once ack: the runtime retries a bounded number of times (policy-configured, never
 * unbounded — RP1 forbids an unbounded accumulator anywhere, incl. a retry queue) and, only after
 * exhausting that bound, EXPLAINs an explicit drop and increments a drop counter. Never a silent
 * fire-and-forget. */
typedef enum { MYC_RETAIN_ACK = 0, MYC_RETAIN_NACK = 1 } myc_retention_ack;

typedef myc_retention_ack (*myc_retention_export_fn)(
    const uint8_t *blob,              /* opaque LANGUAGE-AUDIT blob only (warm digest or a
                                        * cert-handle batch) — never an app-log payload */
    size_t         blob_len,
    const uint8_t  content_hash[32],  /* BLAKE3 digest of blob, ADR-003 framing (RFC-0001 §4.6) */
    void          *user_data
);
```

**Forward compatibility (explicit design constraint, P4-Q3):** *"Hook signature designed so a later
language-effect form wraps the same callback"* — i.e. when a future `RFC-0014`-style declared, bounded
effect surface (the `Code::Budget` shape already in `crates/mycelium-diag/src/lib.rs:147` names the
precedent: *"a declared, bounded effect budget was exhausted"*) wraps this export, it wraps **this same
C-ABI callback**, not a second export mechanism. **Drop accounting** follows the OTel bounded-queue
drop-accounting pattern ([C27]/[C28] per the handoff's citation list — the same anchors DESIGN-04
implicitly draws on for "drop counters `EXPLAIN`ed"): every drop increments a counter that is itself
`EXPLAIN`-able, never a silently-shrinking, unaccounted queue.

## 8. P4-Q4 — lossy warm digests, declared lossiness, and the `EXPLAIN`-of-drop record shape

**Yes, lossy warm digests are permitted** — but only with **declared lossiness**: the digest states
what was dropped and what error semantics survive. Anchoring evidence, honestly graded (`Declared`
choice of shape, not a proof that the chosen sketch is optimal): DDSketch's formal *relative-error*
guarantee is preferred over t-digest's *unbounded worst case* ([C31], [C32]) — i.e. **prefer a
declared-bound sketch shape over an unbounded-error one** when a warm tier compresses hot data.

**The `EXPLAIN`-of-drop record shape (captured verbatim from DESIGN-04 §5.3's sequence diagram):**

```text
{ retained = digest, export = hash?, dropped = N, loss_semantics }
```

- `retained` — what survives in the warm tier (a digest, never a claim of full retention).
- `export` — the content hash of an exported blob, if `on_overflow = export_then_drop` succeeded; absent
  otherwise.
- `dropped` — the count of records the compaction/drop actually removed (never silently unaccounted).
- `loss_semantics` — what error/accuracy semantics the digest still honestly supports (e.g. "exact
  membership lost; count preserved exact; percentile estimate DDSketch-bounded").

**RP7, restated at this policy's level (DESIGN-04 §4):** *a digest never upgrades a grade or fabricates
a checked cert.* A warm digest of dropped `Empirical`/`Declared` data stays at or below its inputs'
weakest tag (VR-5) — summarizing cannot manufacture a `Proven`/`Exact` claim that was never checked, and
a compacted cert-handle batch is never presented as if the original certificates were still individually
retained.

**Open, flagged (not decided here):** whether the `EXPLAIN`-of-drop record is itself emitted as a `Diag`
instance (reusing RFC-0013's extended envelope, Amendment A1) or as a distinct lightweight record is a
Phase-2 implementation call. Reusing `Diag` would be the G-9-consistent choice ("no third diagnostic
system," RFC-0013 Amendment A1 §10.1) — recorded as the natural direction, not committed here, since
this spec fixes the drop-record's **field shape** per P4-Q4, not its concrete emission mechanism.

## 9. P4-Q5 — no surface may claim "full in-process audit retained" under `certified`

**Normative rule (captured, `Declared`):** no surface of `certified` mode may claim that it retains a
full in-process audit trail forever. `certified` means the runtime **can produce checked, exportable,
verifiable evidence** — content-hashed export digests plus **inclusion proofs** — not that every
certificate/`Meta`/first-fault record stays pinned in memory for the process's lifetime. This follows
the Certificate Transparency model directly: verifiability comes from **exported Merkle digests with
O(log n) inclusion/consistency proofs** (RFC 6962, updated by RFC 9162 — [C33]), not from pinning
everything in RAM. Concretely: a `certified` process's hot/warm tiers stay **capped** (§5) exactly like
`fast`'s, just at larger defaults; what makes `certified` *certified* is that its export path (§7) plus
digest chain (§8) let a third party verify an exported claim was actually included in the run, without
requiring the runtime to have kept the whole run in memory to answer that question.

**The one sanctioned exception:** **append-all survives only as an explicit opt-in audit mode that
states its own bound.** A colony/spore may opt into append-all retention for L1–L4, but that mode must
itself declare a bound (a disk quota, a rotation policy, an explicit "unbounded — operator's
responsibility" acknowledgement) — it does not get to claim unbounded retention *by default* under
`certified`, and choosing it is a visible, `EXPLAIN`-able declaration (G2), not an ambient consequence of
picking `certified`.

## 10. Interaction with RFC-0013 Amendment A1 (the first-fault bus, L4)

This policy's `hot_first_fault_cap` bounds exactly the surface RFC-0013's 2026-07-18 Amendment A1
extends (`docs/rfcs/RFC-0013-Structured-Diagnostics-and-Reified-Error-Policy.md` §10): the `Diag`
record, generated ≥ middle tier always (RFC-0034 §7; the Amendment A1 §10.6 cross-reference), retained
here under a **ring** discipline by default (RP4, DESIGN-04 §4: *"first-fault ≠ full history — default
ring/epoch inside the runtime"*). **Generation ≠ retention** (RP1) is the orthogonal-axis restatement of
RFC-0034 §7's generation ≠ consumption: a first-fault record can be generated (RFC-0013 Amendment A1)
and consumed at a dialable tier (RFC-0013 §4.2 `minimal`/`medium`/`detailed`) while *this* policy
separately governs how long the generated record stays resident before it is compacted, exported, or
dropped. None of the three axes — generation, consumption, retention — implies the other two.

## 11. Open questions / FLAGs (genuinely open — not silently decided)

- **L2/L3 dedicated numeric caps.** DESIGN-04 §5.3's five named fields cover L1 (`hot_cert_handle_cap`)
  and L4 (`hot_first_fault_cap`) explicitly; L2 (`EXPLAIN`-signal storage) and L3 (live-value `Meta`) are
  bounded only qualitatively by mode (DESIGN-04 §5.2) today. Whether they get their own
  `LanguageRetentionPolicy` fields or share the L1/L4 caps by convention is left to the Phase-2 sizing
  pass — not invented here.
- **`balanced` mode's concrete numbers.** P4-Q1 gave concrete placeholder numbers for `fast` and
  `certified` only; `balanced`'s dual caps are unspecified. §5's table marks every `balanced` cell
  FLAGGED rather than interpolating a number the steer did not give.
- **The `@retention`-style declaration surface's exact grammar.** Sketched by analogy to
  `@certification` (§4) but not fixed — surface-layer work, out of this capture's scope.
- **The "declared audit obligation" surface (§6).** What declares an audit obligation, and where, is
  not fixed here; P4-Q2 names the *escalation rule*, not the *declaration mechanism*.
- **Whether the `EXPLAIN`-of-drop record reuses `Diag`/Amendment A1** (§8) — recorded as the
  G-9-consistent natural direction, not decided.
- **`fast`'s warm-epoch count** (§5) — plausibly zero/no-warm-tier by inference, not a steered number;
  left to the sizing pass.

## 12. Definition of Done (this spec's own capture, house rule #6)

- [x] Fields captured per DESIGN-04 §5.3 (§3).
- [x] Resolution stated as the fourth RFC-0012 scoped-ambient instance, with the P4-Q2 divergence from
      instances 2/3 made explicit, not silently harmonized (§4).
- [x] P4-Q1 placeholder defaults recorded `Declared`, with the sizing-pass upgrade path to `Empirical`
      named (§5).
- [x] P4-Q2 bounded-default-plus-warning rule + the audit-obligation escalation captured (§6).
- [x] P4-Q3 FFI export-hook signature sketched (illustrative, not committed) with the at-least-once-ack
      + `EXPLAIN`ed-drop-counter contract and the later-effect-form forward-compatibility constraint (§7).
- [x] P4-Q4 lossy-warm-digest rule + the exact `EXPLAIN`-of-drop record shape + RP7 restated (§8).
- [x] P4-Q5 no-full-retention-under-certified rule + the verifiable-evidence (export digest + inclusion
      proof) model + the append-all opt-in exception, each stating its own bound (§9).
- [x] Every genuinely-open item flagged rather than guessed (§11) — G2/VR-5.
- [ ] Maintainer ratifies this capture (gate, per the handoff §4 Phase-1 stop point).
- [ ] Phase-2 sizing pass (DESIGN-04 §10 P0) upgrades the `Declared` defaults toward `Empirical`.
- [ ] The concrete `LanguageRetentionPolicy` Rust/Mycelium-lang struct, the mode-gated cert store, and
      the `@retention`-style declaration surface land in a later wave — not this capture (no code lands
      with this spec, mirroring RFC-0013 Amendment A1's own "no code lands with this capture" posture).

## Meta — changelog

- **2026-07-18 — Created (Draft, pending ratification).** Mints `docs/spec/Language-Retention-Policy.md`
  as the Phase-1 ratifiable capture of `LanguageRetentionPolicy` (handoff §4 item 5; steer §1.4
  P4-Q1..Q5). Captures the DESIGN-04 §5.3 field set, the fourth-RFC-0012-scoped-ambient-instance
  resolution law (with the P4-Q2 unresolved-case divergence from `CertMode`/`policy: ambient` made
  explicit), the `Declared` placeholder mode defaults (with L2/L3 and `balanced` gaps flagged rather
  than fabricated), the bounded-default-plus-warning rule, the host-FFI export-hook sketch, the lossy
  warm-digest rule + `EXPLAIN`-of-drop record shape + RP7 restatement, and the no-full-retention-under-
  `certified` rule with its verifiable-evidence model and append-all opt-in exception. No code lands
  with this capture; the maintainer's ratification and the Phase-2 sizing pass remain open (§12).
  Append-only (new file; nothing here supersedes existing text). Grounds: DESIGN-04 §1–§11;
  `PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §1.4/§4 item 5; RFC-0012/0013/0034/0002/0001; ADR-003;
  KC-3; G2; VR-5.
