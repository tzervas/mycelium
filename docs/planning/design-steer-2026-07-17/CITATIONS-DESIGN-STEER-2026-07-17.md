# Citations — Design-pack steer (2026-07-17)

| Field | Value |
|---|---|
| **Role** | External-evidence companion to `PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md`. The handoff cites entries as `[C#]`. |
| **Provenance** | Compiled from the maintainer-directed research pass (2026-07-17) run against the actual DESIGN-01..04 contents. Entries marked **◆** were identified as relevant but **not primary-source-verified** during the pass — the agent must verify before load-bearing use. Unmarked entries were verified against the named source. |
| **Honesty** | An external citation supports a *rationale*; it never substitutes for a repo ground (RFC/ADR/DN). Where evidence and corpus conflict, the corpus + maintainer steer govern. |

## A. External references

### Pack 01 — swaps, policy, conversion typing

1. **[C1] RFC 1542: TryFrom and TryInto — The Rust RFC Book.** rust-lang.github.io/rfcs/1542-try-from.html — Fallible-vs-infallible conversion split (`From` perfect, `TryFrom` → `Result`). Grounds P1 regime→result typing (A5) and the layer reconcile.
2. **[C2] `TryFrom` — Rust std docs.** doc.rust-lang.org/std/convert/trait.TryFrom.html — "`From` is intended for perfect conversions"; `as` "will silently truncate" = the `regime_type_lie` Mycelium refuses.
3. **[C3] Scala 3 implicit conversions: reference + SIP-71 `into`.** docs.scala-lang.org/scala3/reference/changed-features/implicit-conversions.html · docs.scala-lang.org/sips/71.html — Explicit opt-in restriction; removing implicit conversions *improves* expected-type inference. Grounds S1 (no auto-swap) and P1-Q3 elision gating.
4. **[C4] Odersky, feature warnings for implicit conversions (dotty PR #4229).** github.com/scala/scala3/pull/4229/files — "easily the most mis-used feature in Scala… more than 90% of the uses are mis-guided." Cautionary support for S1.
5. **[C5] Unison: the big idea — content-addressed code.** unison-lang.org/docs/the-big-idea/ — Definitions identified by syntax-tree hash; names are metadata. Grounds content-addressed PolicyRef/catalog + `expand ≡ longhand, same hash` conformance (with RFC-0012).
6. **[C6] Necula, *The Design and Implementation of a Certifying Compiler* (PLDI '98).** cs.cmu.edu/afs/cs/academic/class/15745-s06/web/handouts/necula-pldi98.pdf — Per-operation checkable certificates; consumer checks rather than proves. Grounds cert emit/check + P1-Q2 timing.
7. **[C7] Necula, Proof-Carrying Code (lecture notes).** asimod.in.tum.de/2005/abstracts/Necula_05.pdf — PCC framing; checkable-proof artifact.
8. **[C8] Certificate-size reduction in Abstraction-Carrying Code.** arxiv.org/pdf/1010.4533 — Certificate size/cost is the known bottleneck → handles + materialize-on-demand, CertMode dialing.
9. **[C9] King, *Parse, don't validate* (2019).** lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/ — Boundary parsing yields typed proof; partiality is the parser's nature. Grounds A5 + the predicate remint basis (P2-Q1).

### Pack 02 — lattices, containment, declassification

10. **[C10] Denning, *A Lattice Model of Secure Information Flow* (CACM 1976).** faculty.nps.edu/dedennin/publications/lattice76.pdf (DOI 10.1145/360051.360056) — Lattice meet/join foundation; ancestor of weakest-wins grade composition (with Biba integrity per RFC-0018).
11. **[C11] Stefan et al., *Flexible Dynamic Information Flow Control in Haskell* (LIO, 2011).** scs.stanford.edu/~dm/home/papers/stefan:lio.pdf — Floating-label creep = the poison problem; clearance = mode-firewall analog. Grounds P2-Q3/Q4 and export-only-first (P2-Q2).
12. **[C12] LIO.Core docs (clearance).** hackage.haskell.org/package/lio-0.10.0.0/docs/LIO-Core.html — Clearance "imposes an upper bound on the current label."
13. **[C13] Sabelfeld & Sands, *Declassification: Dimensions and Principles*.** cse.chalmers.se/~andrei/sabelfeld-sands-jcs07.pdf — What/who/where/when taxonomy; declassification at explicit reference points. Grounds airlock/remint discipline + deferring trial-basis remint (P2-Q1).
14. **[C14] Ruby issue #16131: Remove `$SAFE`, taint and trust.** bugs.ruby-lang.org/issues/16131 — Deprecate 2.7 / no-op 3.0 / remove 3.2 timeline. Cautionary tale for one pervasive trust dial → three-axis non-collapse.
15. **[C15] Saeloun, Ruby 2.7 taint deprecation writeup.** blog.saeloun.com/2020/02/18/ruby-2-7-access-and-setting-of-safe-warned-will-become-global-variable/ — Stated removal reasons (CGI-era, unsupported by input libs, `$SAFE` vulnerabilities).
16. **[C16] Ernst et al., *Collaborative Verification of Information Flow…* (CCS 2014).** homes.cs.washington.edu/~mernst/pubs/infoflow-ccs2014.pdf — "6 annotations per 100 lines… less than 1/4 of the annotation burden for Jif." Grounds zero-ceremony budget / modular bottom / export-only seal.
17. **[C17] Jif — Cornell.** cs.cornell.edu/jif/ — Canonical fine-grained IFC; adoption friction motivates the ceremony budget.
18. **[C18] Wadler & Findler, *Well-Typed Programs Can't Be Blamed* (ESOP 2009).** homepages.inf.ed.ac.uk/wadler/papers/blame/blame.pdf — Blame assignment across boundaries = `parent_event`/`child_cause` causality model.

### Pack 03 — diagnostics, localization, UX

19. **[C19] IBM WebSphere, First Failure Data Capture (FFDC).** ibm.com/docs/en/was-nd/8.5.5?topic=tools-first-failure-data-capture-ffdc — Capture-at-first-fault vs re-run debugging; the N9 term of art.
20. **[C20] rustc `Applicability`.** doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/enum.Applicability.html — `MachineApplicable` vs `MaybeIncorrect`/`HasPlaceholders` → "candidates only, never auto-apply" (X11, P1-Q3 refusal listings).
21. **[C21] Rust Compiler Development Guide — Errors and lints.** rustc-dev-guide.rust-lang.org/diagnostics.html — Structured diagnostics API, error codes, JSON output; grow-in-place precedent for P3-Q1.
22. **[C22] Elm, *Compiler Errors for Humans*.** elm-lang.org/news/compiler-errors-for-humans — Diagnostics quality as adoption driver; grounds shipping P0 surfaces early (P3-Q3).
23. **[C23] OpenTelemetry — Sampling (head vs tail).** opentelemetry.io/docs/concepts/sampling/ — Generation ≠ consumption precedent (P3-Q2).
24. **[C24] OpenTelemetry — Trace API / overview (Events, Links).** opentelemetry.io/docs/specs/otel/trace/api/ · opentelemetry.io/docs/specs/otel/overview/ — Links = causal relations across traces; Events = (timestamp, name, attrs). Confirms first-fault → OTel mapping stays clean (ranked option 4, later). Note: target logs/links/severity model, not the Span-Events *API* (announced future deprecation, 2026-03 blog).
25. **[C25] OpenTelemetry — Logs data model (SeverityNumber).** opentelemetry.io/docs/specs/otel/logs/data-model/ — 1–24 severity scale for lossless mapping.
26. **[C26] LSP 3.17 specification.** microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/ — `DiagnosticRelatedInformation` (parent/child), `DiagnosticTag`, `CodeAction` quickfix; one bus feeds hover/actions.

### Pack 04 — retention, digests, verifiable export

27. **[C27] OpenTelemetry Collector — batch processor README.** github.com/open-telemetry/opentelemetry-collector/blob/main/processor/batchprocessor/README.md — Bounded batching; drop ordering guidance.
28. **[C28] OpenTelemetry — Scaling the Collector.** opentelemetry.io/docs/collector/scaling/ — `otelcol_exporter_enqueue_failed_*` drop accounting; "Dropping data because sending_queue is full" → drop-made-visible = EXPLAIN-of-drop (RP3, P4-Q3).
29. **[C29] Dev.java — Configuring the JDK Flight Recorder.** dev.java/learn/jvm/jfr/configure/ — Always-on low overhead; defaults: maxsize 250 MB (JDK 11+ "using maxsize=250MB as default"), 12 MB chunk, 10 MB memorysize, 8 KB thread buffers; JDK-8 unlimited-default footgun. Anchors P4-Q1/Q2. Version-specific numbers — treat as anchors, not constants.
30. **[C30] Prometheus — Storage.** prometheus.io/docs/prometheus/latest/storage/ — Default retention 15d; 2h blocks; `remote_write` bounded-local/offloaded-durability pattern (RP5, P4-Q3).
31. **[C31] Masson, Rim & Lee, *DDSketch* (PVLDB 12(12), VLDB 2019).** vldb.org/pvldb/vol12/p2195-masson.pdf — Fully-mergeable quantile sketch with *formal relative-error guarantees* → declared-lossiness digests (P4-Q4).
32. **[C32] t-digest worst-case comparison (arXiv 2102.09299) · Apache DataSketches quantiles overview.** arxiv.org/pdf/2102.09299 · datasketches.apache.org/docs/QuantilesAll/QuantilesOverview.html — t-digest "has no mathematical basis for estimating its error" → prefer declared-bound shapes.
33. **[C33] RFC 6962 (Certificate Transparency); updated by RFC 9162.** rfc-editor.org/rfc/rfc6962.html · datatracker.ietf.org/doc/html/rfc9162 — Merkle logs, Signed Tree Heads, O(log n) inclusion/consistency proofs → "export digests that still verify" (P4-Q5); content-hash dedupe for cert handles.
34. **[C34] rfcs/1542-try-from.md (mirror) · Rust by Example: TryFrom/TryInto.** github.com/rust-lang/rfcs/blob/master/text/1542-try-from.md · doc.rust-lang.org/rust-by-example/conversion/try_from_try_into.html — Corroborates [C1]/[C2].

### ◆ Pointer-only (verify before load-bearing use)

- **◆ Trusted Types (W3C/MDN)** — sanitizer-gated remint of strings for DOM sinks; shipped airlock analog (P2-Q1). Pattern well-known; spec not re-fetched during the pass.
- **◆ Ariadne / miette (Rust diagnostic-rendering crates)** — implementation shortcuts for envelope rendering (W-A). Confirm current versions/APIs.
- **◆ SHErrLoc (Cornell)** — type-error root-cause localization research; academic backbone for N9.
- **◆ Pierce & Turner, local type inference; Dunfield & Krishnaswami, bidirectional typing survey** — foundation for P1-Q3 expected-type elision; Swift expression-inference compile-time blowups as the caution.
- **◆ Koka / OCaml 5 effect handlers** — the deferred language-effect export-hook form (P4-Q3).
- **◆ systemd-journald vacuum defaults; dmesg `log_buf_len`; PostgreSQL WAL checkpointing; RocksDB LSM compaction** — additional retention/compaction precedents; exact defaults not re-verified.
- **◆ W3C PROV; Green–Karvounarakis–Tannen semiring provenance** — candidate formal shapes for `basis_ref` and Meta propagation cost analysis (S1 spike input).

## B. Repo grounds index (internal — cite directly, not via [C#])

| Ground | Bears on |
|---|---|
| RFC-0002 §2–§5 (Accepted) | Single cert format/checker; legal-pair table (A1 source rows); `Option`-typed `dec`; TV-incompleteness → explicit fallback |
| RFC-0005 (Accepted) | Total/deterministic/content-addressed policies; mandatory EXPLAIN; fixed precedence → `policy: ambient` resolution law |
| RFC-0012 (NFR-7) | Sugar ≡ longhand, identical hash — conformance law for ambient spelling + `to:` elision |
| RFC-0013 §4 | `Diag` content-addressed dual projection; graded levels; additive-never-substitutive → P3-Q1 extension target |
| RFC-0018 §3–§4 (esp. §3.3, §4.5) | Lattice/meet; Swap = only endorsement; §4.5 implicit-flows open decision (couples to S2) |
| RFC-0033 / E20-1 / M-756 | `BigTernary` uncapped arithmetic; conversion-utility `i64` ceiling (m ≤ 40) → E-W1 |
| RFC-0034 §3–§8 (esp. §6, §7) | Mode invariants; scoping precedence (reused ×4); gen≠consumption normative (P3-Q2) |
| ADR-003 | Content addressing; mode/Meta excluded from hash; decomposition identity invariant |
| ADR-013 + RFC-0034 §8 | Spore = deployable unit; identity survives cert-off runtime |
| DN-16 + `docs/spec/stdlib/swap.md` §3/§7 | Landed `Swapped{value,cert}` surface; Result-only std pin; §7-Q2 explicit-until disposition |
| `mycelium-cert/src/mode.rs` | `GatedSwap`; trait-boundary cert discard ("the trait's `swap` discards it"); NotValidated → hard error — W-A seams |
| `mycelium-core/src/meta.rs` | `Meta{…, policy_used: Option<ContentHash>, cert_mode, wrapping_opt}` — handle-pattern precedent for `Meta.cert` |
| `mycelium-diag/src/lib.rs` | `Diag{severity,code,message,locus,trace,notes}` + `content_hash()` + `human()/machine()` — envelope base |
| `mycelium-select/src/lib.rs` | `SelectionPolicy`, `PolicyRegistry` (hash-keyed), total `explain()` — catalog substrate |
| `mycelium-transpile/src/emit.rs` | `u64`/`usize` → width 64 mapping — W-1 corroboration |
| `lib/std/swap.myc` · `docs/lib-index/INDEX.md` | De facto {8,6}/{4,3} export canon; `matrix_len => Binary{8}` inconsistency — W-1 sweep sites |
| `docs/planning/gap-analysis-2026-07-16/PARTITION.md` | 8 scope groups × 56 crates — Phase-3 component-repo seed |
| `maint-guide.md` · `Blocked-Decisions-Ratification-Map.md` (M-540) | Program frame; M-540 consumes the Swap Ergonomics DN |
