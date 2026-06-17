# Design Note DN-07 — RFC-0016 (Core/Standard Library) Ratification-Readiness

| Field | Value |
|---|---|
| **Note** | DN-07 |
| **Status** | **Draft** (2026-06-17) — a ratification-*readiness* assessment, presented for the maintainer's append-only call. It **does not** flip RFC-0016 `Draft → Accepted` (that is the maintainer's decision per RFC-0016 §8 and the append-only rule); it frames *what remains to decide* so the pass is fast and honest. |
| **Feeds** | **RFC-0016** (Core Library & Standard Library — the contract + taxonomy keystone, `Draft`); `docs/spec/stdlib/` (the 23 module specs + their cross-module reconciliation §5); `docs/spec/stdlib/self-hosting-readiness.md` (M-502, the migration gate); `docs/planning/phase-5.md` (the M-510…M-534 decomposition) |
| **Date** | June 17, 2026 |
| **Decides** | *Nothing normatively* — it **recommends** a disposition for each RFC-0016 §8 open question (resolve-with-a-recommendation vs formally-defer), states which items genuinely need maintainer ratification, and records the one residual cross-module FLAG + the standing pre-ratification grounding obligation (§7). Grounded throughout; never invents a resolution where the corpus has none (the planning analogue of G2). |
| **Task** | M-501 (#149) — the design-first keystone's readiness framing |

> **Posture (honesty rule).** This note is *advisory*. Where it says "recommend", it means a
> grounded proposal for the maintainer to accept or reject — not a decision. Where the corpus does
> not yet ground an answer, the question is **formally deferred** with its gating reason, never
> silently closed. RFC-0016's status is untouched by this note.

---

## 1. Why this note

RFC-0016's acceptance (M-501, #149) names the gate the rest of Phase 5 depends on: *"the Core
Library RFC presented + ratified (Draft→Accepted, maintainer) before any stdlib code; it fixes the
module set, the per-op guarantee/EXPLAIN contract, and the Rust→Mycelium migration order."* As of
2026-06-17 the RFC is `Draft (Proposed)` and **all 23 of its taxonomy modules now have a `Draft`
spec** (`docs/spec/stdlib/`, two design waves — PRs #180/#181), each carrying the §4.1 contract
(C1–C6) and the §4.5 guarantee-matrix obligation, with the cross-module seams reconciled in the
spec index §5.

That is the corpus the maintainer ratifies *against*. This note does the readiness bookkeeping: it
(a) summarizes the 23 specs by Ring/Tier, (b) gives each RFC-0016 §8 open question a recommended
disposition with its grounding, (c) flags the one residual cross-module FLAG, and (d) records the
standing §7 research-tracing obligation. The intent is that the maintainer's ratification pass is a
*review of recommendations*, not a fresh design exercise.

## 2. The 23 specs at a glance (by Ring / Tier)

The authoritative roster is the spec index (`docs/spec/stdlib/README.md` §4); reproduced here grouped
by **ring** (RFC-0016 §4.2 — the KC-3-preserving layering) and **tier** (§4.3/§4.4 — *intent*:
A = Mycelium-shaped differentiator, B = table-stakes common). Every row is `Draft — landed` (authored and
integrated, awaiting maintainer ratification with RFC-0016). Every spec meets the same §4.1 contract;
the tier split is roadmap legibility, not a quality split.

**Ring 0 — kernel-adjacent re-exports** (no new trusted code; KC-3):

| Module | Task | Tier | Grounding |
|---|---|---|---|
| `core` / prelude | M-515 (#157) | A | RFC-0001 (value model, guarantee lattice, content-addressing) |

**Ring 1 — capability surfaces** (Tier A; certificate/EXPLAIN *consumers* over landed crates):

| Module | Task | Grounding |
|---|---|---|
| `numerics` | M-512 (#153) | ADR-010/011; M-201/202/203 |
| `swap` | M-516 (#158) | RFC-0002; M-120/210/211/231 |
| `vsa` / `hdc` | M-513 (#154) | RFC-0003/0009; M-130/240–242/260 |
| `dense` | M-518 (#160) | RFC-0001 §4.1; M-230 |
| `select` / `explain` | M-519 (#161) | RFC-0005/ADR-006; M-220/221/222 |
| `diag` | M-510 (#151) | RFC-0013; M-345 |
| `recover` | M-520 (#156) | RFC-0014; M-352/353 |
| `content` / `hash` | M-523 (#164) | ADR-003; RFC-0001 §4.6 |
| `spore` | M-522 (#163) | ADR-013; RFC-0003 §6; M-368 |

**Ring 2 — general library** (written to the contract over Ring 0/1):

| Module | Task | Tier | Honesty crux |
|---|---|---|---|
| `collections` | M-511 (#152) | B | value-semantic; a rehash/rebalance is not a silent reorder |
| `text` / `string` | M-524 (#165) | B | `parse` → `Result`; lossy encoding explicit (C1) |
| `math` | M-525 (#166) | B | rounding/approx ops carry their tag (C2); domain errors explicit |
| `iter` | M-526 (#167) | B | total/terminating where the kernel guarantees it; laziness explicit |
| `error`/`option`/`result` | M-527 (#168) | B | propagation is the floor (I1); suppression impossible |
| `io` + `serialize` | M-514 (#155) | B | substrate single-consumption (LR-8); one canonical JSON |
| `fs` | M-528 (#169) | B | every path/permission failure explicit; `wild` floor (§8-Q6) |
| `time` | M-529 (#170) | B | monotonic vs wall a *typed* distinction; RT3 declared effect |
| `rand` | M-531 (#171) | B | nondeterminism reified/named (RT3); no silent entropy |
| `cmp` / `convert` | M-532 (#172) | B | a lossy convert is explicit + fallible, never silent narrowing |
| `fmt` | M-533 (#173) | B | dual human/machine (JSON) projection (G11), one canonical form |
| `testing` | M-534 (#174) | B | a skipped/undetermined check is reported, never a silent pass |
| `runtime` / `colony` | M-521 (#162) | A | **reserved-not-active** vocabulary; Phase-7-gated (§8-Q4) |

`runtime`/`colony` is tier-A by intent (the fungal concurrency surface, RFC-0008) but is sequenced in
Ring 2 of the layering because it is *reserved vocabulary* until the RFC-0008 constructs land — a
FLAGGED cross-phase dependency (§8-Q4), not active surface.

**The load-bearing common spine (every spec, both tiers):** the §4.1 contract C1–C6 — never-silent
(C1/G2), honest per-op guarantee tag (C2/VR-5), no black boxes / EXPLAIN (C3/SC-3/G11),
content-addressed value-semantics (C4/ADR-003), above the small kernel (C5/KC-3), declared+bounded
effects (C6/RFC-0014) — verified per-op by the module's **guarantee matrix** (§4.5), *data asserted in
tests, never prose only*. The spec index §5 establishes that, with all 23 drafted, **no two specs
conflict on an owned surface**.

## 3. Disposition of the RFC-0016 §8 open questions

Each §8 question below gets a **recommended disposition**. Two outcomes are possible: **Resolve** (the
corpus grounds a recommendation the maintainer can accept at ratification) or **Defer** (a real
dependency blocks resolution; recorded with its gate). The recommendations are grounded; none is a
silent decision.

### Q1 — the v0 module set + priority order → **RESOLVE (with a recommendation)**
All 23 taxonomy modules have a `Draft` spec with no owned-surface conflict (spec index §5 "Net").
The M-346 five-candidate floor — `diag`/`collections`/`numerics`/`vsa`/`io+serialize` — is the safe
v0 set, and RFC-0016 §4.6 already names `diag` + `recover` as the *first migration* targets (the most
honesty-load-bearing). **Recommendation:** ratify the full 23-module taxonomy as the v0 *scope*, with
the five-candidate floor sequenced first and `diag`/`recover` leading the Rust→Mycelium migration. The
**priority order** is the maintainer's to confirm. *Ratification item.*

### Q2 — naming (phylum `std`? crate-mirrored module names? lexicon) → **RESOLVE (DN-level call)**
Grounding: DN-06 fixed `phylum` (library-scale) / `nodule` (basic unit); the spec index §5 records
that `core`/`ternary`/`swap`/`content` defer the `Bit`/`Trit`/`std`/error-value names to the DN-02/06
lexicon and that **`core` and `error` must agree the one error-value identifier** (DN-03's
one-name-per-term rule). **Recommendation:** name the phylum `std`; mirror module names to the
capability-crate names (`mycelium-vsa → std.vsa`, `mycelium-swap → std.swap`) for traceability, using
a themed lexicon name only where the corpus already has one (`spore`, `runtime`/`colony`). Close the
`core`↔`error` error-value name as a single DN decision. This is an **append-only lexicon decision**
(DN-02/03/06 discipline) — the maintainer's call, recommended-not-flipped. *Ratification item (DN).*

### Q3 — ergonomics vs the contract (tension A) → **DEFER (needs one per-ring design pass)**
This is the **single most-recurrent** tension: ~10 specs FLAG it (`core`/`cmp`/`error`/`fmt`/`iter`/
`math`/`recover`/`select`/`testing`/`text` per the per-spec §7 review), and the spec index §5 names
the same seven-module cluster. The question — is the tag/EXPLAIN/certificate machinery
*implicit-by-default-but-inspectable* or *always-explicit at the call site*? — is genuinely undecided
and load-bearing for adoption. **Recommendation (direction, not resolution):** adopt the **RFC-0012
ambient-representation precedent** library-wide — implicit at the call site, but always reified,
queryable, and EXPLAIN-able (never a black box, C3) — and discharge it as **one per-ring design pass**
(a dedicated follow-on task), not seven per-module answers. *This is the chief genuine open design
question; it does not block ratifying the contract + taxonomy, but it should be the maintainer's
explicit "direction accepted, pass scheduled" rather than left implicit.* *Defer with a named follow-on.*

### Q4 — `runtime`/`colony` sequencing (std vs separate phylum, Phase-7) → **DEFER (cross-phase gate)**
Grounding: the `runtime` spec is reserved-not-active vocabulary (Glossary ⟂), gated on the RFC-0008
constructs activating (Phase 7); spec index §5 records this as a FLAGGED cross-phase dependency.
**Recommendation:** home the runtime surface in a **separate `runtime` phylum** (or a gated sub-phylum)
so pure `std` carries no inactive surface, and activate it construct-by-construct at the Phase-7 gate.
**This does not block RFC-0016 ratification** — the contract + taxonomy stand without it; only
`runtime`'s *placement* is deferred. *Defer (Phase-7).*

### Q5 — the migration differential's bar → **RESOLVE (recommendation), ratify with M-502**
Grounding: 14 specs reference a self-hosting differential; `swap`/`testing`/`self-hosting-readiness`
(M-502) tie it to one shared bar (spec index §5). The interp↔AOT differential (NFR-7) already runs
through the M-210 `ObservationalEquiv` checker on observable results. **Recommendation:** a **two-level
bar** — (1) *observable-result equivalence through M-210* as the universal floor (matches the existing
NFR-7 differential), and (2) *tag + EXPLAIN equivalence* as a **stronger, per-module-declared** bar for
honesty-load-bearing modules whose tags/certificates are themselves observable (`swap` certificates,
`select` EXPLAIN, `numerics` bounds). M-502 makes the verdict *checkable*; it stays honestly *not
established* until a concrete L3 surface (M-320 / task-D track) can author a stdlib module. *Ratification
item, jointly with M-502.*

### Q6 — the `wild`/FFI floor (`std` vs `std-sys`) → **RESOLVE (with a recommendation)**
Grounding: corroborated from three modules — `fs` (syscalls), `rand` (platform entropy), `math` (libm
transcendentals) — each bottoming out in an audited `wild` block (ADR-014); spec index §5 consolidates
them into "one `std-sys` question, three call sites." **Recommendation:** split the audited `wild`
floor into a **separate `std-sys` phylum** so pure `std` stays leak-free *by construction* (LR-9) and
can publish a `wild`-free certification badge (RFC-0016 §9). The **minimal audited FFI inventory** is
the deliverable the maintainer must approve. *Ratification item.*

### Residual cross-module FLAG (not a §8 item) — the `BF16→F32` placement
The one unsettled *owned-surface* seam (spec index §5; swap §7-Q3 / cmp §7-Q2): the *lossless reverse*
`BF16→F32` widening — `swap` (certificate-carrying) or `cmp`/`convert` (lossless widening, no
certificate)? No op is double-owned; only this reverse's owner is open. **Recommendation:** lift it to
`cmp`/`convert` as lossless float widening (a certificate certifies a *bounded/lossy* change; a lossless
widen needs none), leaving `swap` to own only the certified/lossy direction. *Ratification item (small).*

## 4. The standing pre-ratification grounding obligation (RFC-0016 §7)

RFC-0016 §7 records a gate the maintainer must clear (or knowingly waive) *before* flipping
Draft→Accepted: the cross-language stdlib comparison (Rust/Python/Go/OCaml module sets) and the
"honest stdlib" prior art (refinement-typed / verified standard libraries) are **to be traced into
`research/` as a Research Record**, so the taxonomy choices are grounded, not asserted. This note
**flags it as outstanding** — it is not yet a Research Record in `research/`. It does not change the
*contract* (C1–C6, which are lifted from the already-grounded house rules), but it is the §7 obligation
on the *taxonomy*'s external grounding and should be discharged or explicitly waived at ratification.

## 5. Readiness verdict

**The RFC-0016 core is ratification-ready.** The §4.1 contract (C1–C6), the ring layering (§4.2), the
Tier-A/Tier-B taxonomy (§4.3/§4.4), the per-module guarantee-matrix obligation (§4.5), and the
Rust-first→Mycelium migration order (§4.6) are each corroborated by 23 independent `Draft` specs with
**no owned-surface conflict** (spec index §5). The §8 questions do **not** destabilize that core; they
split cleanly into:

- **Resolvable at ratification** (a grounded recommendation to accept/reject): **Q1** (full taxonomy,
  five-candidate floor first), **Q2** (`std` + crate-mirrored names; one error-value name — a DN call),
  **Q5** (two-level differential bar, with M-502), **Q6** (`std-sys` split for the `wild` floor), and
  the **`BF16→F32`** FLAG (lift to `cmp`/`convert`).
- **Genuinely deferred** (a real dependency, not a blocker for the contract+taxonomy): **Q3** (the
  ergonomics-vs-contract per-ring design pass — the chief open design question; recommend the RFC-0012
  ambient precedent + schedule the pass) and **Q4** (the `runtime` phylum's Phase-7-gated placement).
- **A standing grounding gate** (§4): the §7 cross-language/"honest stdlib" Research Record is not yet
  in `research/` — discharge or knowingly waive at ratification.

**What the maintainer must decide to flip RFC-0016 `Draft → Accepted`:** (1) accept/adjust the v0
module set + priority (Q1); (2) ratify the naming/lexicon (Q2, a DN decision); (3) accept the
differential bar (Q5) and the `std-sys` split (Q6) and the `BF16→F32` placement; (4) **accept the
*direction* for Q3 and schedule the per-ring ergonomics pass**, and note Q4 as a Phase-7 deferral; (5)
discharge or waive the §7 Research Record gate. None of these requires inventing scope — every input is
grounded in the corpus or explicitly FLAGGED. **This note does not flip the status** (RFC-0016 §8 / the
append-only rule); it is the maintainer's call.

## Meta — changelog

- **2026-06-17 — Draft (created).** Ratification-readiness assessment for RFC-0016 (M-501, #149): the
  23 module specs summarized by Ring/Tier (Ring 0 `core`; Ring 1 the nine Tier-A capability surfaces;
  Ring 2 the Tier-B commons + reserved `runtime`), a recommended disposition for each §8 open question
  (Q1/Q2/Q5/Q6 + the `BF16→F32` FLAG → resolve-with-a-recommendation; Q3 ergonomics + Q4 runtime →
  formally deferred with their gates), the standing §7 Research-Record grounding obligation flagged, and
  an explicit "what the maintainer must decide" list. Advisory; **does not** move RFC-0016's status
  (append-only — the maintainer's call). Grounds every recommendation in the corpus; FLAGs, never
  invents, where the corpus is silent (planning analogue of G2). No code; no kernel change (KC-3).
