# Mycelium Agent Code Index

> **Honesty:** `Empirical/Declared — line/regex heuristic; source is ground truth. Use this index to find where to Read, not as an authoritative reference.`
> Use the index to find where to `Read`, not as an authoritative reference.

## mycelium-bench

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_bench::Backend` | enum | `crates/mycelium-bench/src/backend.rs:21` | One of the execution backends being measured. |
| `mycelium_bench::Case` | struct | `crates/mycelium-bench/src/corpus.rs:55` | One corpus entry: a stable id, the v0-calculus source, its fragment, and a one-line note on what |
| `mycelium_bench::CaseRecord` | struct | `crates/mycelium-bench/src/measure.rs:58` | All backends' results on one case. |
| `mycelium_bench::Engines` | struct | `crates/mycelium-bench/src/backend.rs:165` | The shared trusted engines every backend dispatches through (prim registry + certified swap). |
| `mycelium_bench::Fragment` | enum | `crates/mycelium-bench/src/corpus.rs:28` | Which evaluation-complete fragment a case lives in — the basis for whether a compiled backend can |
| `mycelium_bench::GrokLlmReport` | struct | `crates/mycelium-bench/src/llm.rs:243` | The top-level Grok/xAI harness report (`mycelium-grok-coauthor`). |
| `mycelium_bench::Honesty` | struct | `crates/mycelium-bench/src/report.rs:42` | The honesty posture block stamped into the report. |
| `mycelium_bench::LlmReport` | struct | `crates/mycelium-bench/src/llm.rs:38` | A parsed LLM-harness report (the subset of the schema the unified report needs; extra fields are |
| `mycelium_bench::LlmSection` | struct | `crates/mycelium-bench/src/report.rs:64` | The LLM-harness section of the unified report. |
| `mycelium_bench::Outcome` | enum | `crates/mycelium-bench/src/backend.rs:80` | The result of running one backend on one case — uniform across backends, never-silent. |
| `mycelium_bench::ParsedLlmSection` | struct | `crates/mycelium-bench/src/llm.rs:461` | Intermediate form produced by [`parse_any_llm_json`]; consumed by |
| `mycelium_bench::Report` | struct | `crates/mycelium-bench/src/report.rs:21` | Everything the report needs: the run record, optional ingested LLM-harness report, and run |
| `mycelium_bench::RunRecord` | struct | `crates/mycelium-bench/src/measure.rs:75` | The full execution-backend run: every case's record, in corpus order. |
| `mycelium_bench::Speed` | enum | `crates/mycelium-bench/src/verdict.rs:25` | The speed comparison band of a backend vs the interpreter, once both produced an *equal* value. |
| `mycelium_bench::Verdict` | enum | `crates/mycelium-bench/src/verdict.rs:37` | The full classification of one (backend, case) pair vs the trusted interpreter baseline. |
| `mycelium_bench::backend` | mod | `crates/mycelium-bench/src/lib.rs:44` | — |
| `mycelium_bench::backend::Backend::all` | fn | `crates/mycelium-bench/src/backend.rs:40` | All backends, in a stable order (interp first — it is the differential baseline). |
| `mycelium_bench::backend::Backend::is_baseline` | fn | `crates/mycelium-bench/src/backend.rs:64` | Whether this backend is the trusted differential baseline. |
| `mycelium_bench::backend::Backend::is_process_spawn_bound` | fn | `crates/mycelium-bench/src/backend.rs:73` | Whether this backend executes a freshly-spawned process per invocation (a microbench caveat |
| `mycelium_bench::backend::Backend::label` | fn | `crates/mycelium-bench/src/backend.rs:52` | A short, stable label for the report (matches the serde rename). |
| `mycelium_bench::backend::Outcome::reason` | fn | `crates/mycelium-bench/src/backend.rs:125` | The reason string for a non-value outcome (empty for a value). |
| `mycelium_bench::backend::Outcome::status` | fn | `crates/mycelium-bench/src/backend.rs:114` | A short status word for the report. |
| `mycelium_bench::backend::Outcome::value` | fn | `crates/mycelium-bench/src/backend.rs:105` | The produced value, if any. |
| `mycelium_bench::backend::Outcome::value_outcome` | fn | `crates/mycelium-bench/src/backend.rs:99` | A value outcome (boxes the value to keep the enum compact). |
| `mycelium_bench::backend::WarmRun` | struct | `crates/mycelium-bench/src/backend.rs:304` | A **warm runner**: a backend prepared so its *per-run* cost can be timed honestly, separating any |
| `mycelium_bench::backend::observable_eq` | fn | `crates/mycelium-bench/src/backend.rs:146` | **Observational equivalence** of two results — the honest differential equality. |
| `mycelium_bench::backend::run_aot_env` | fn | `crates/mycelium-bench/src/backend.rs:219` | Run the **AOT env-machine** (`mycelium_mlir::run_core`) on `node` — a big-step ANF evaluator, |
| `mycelium_bench::backend::run_direct_llvm` | fn | `crates/mycelium-bench/src/backend.rs:241` | Run the **direct-LLVM** backend (`mycelium_mlir::compile_and_run`) on `node`. |
| `mycelium_bench::backend::run_interp` | fn | `crates/mycelium-bench/src/backend.rs:202` | Run the **reference interpreter** (trusted base) on `node`. |
| `mycelium_bench::backend::run_jit` | fn | `crates/mycelium-bench/src/backend.rs:230` | Run the **JIT** backend (`mycelium_mlir::jit_run`) on `node`. |
| `mycelium_bench::backend::run_mlir_dialect` | fn | `crates/mycelium-bench/src/backend.rs:253` | Run the **MLIR-dialect** backend on `node`. |
| `mycelium_bench::backend::run_once` | fn | `crates/mycelium-bench/src/backend.rs:284` | Dispatch: run one backend on one node once, returning its [`Outcome`]. |
| `mycelium_bench::backend::warm_runner` | fn | `crates/mycelium-bench/src/backend.rs:336` | Build a [`WarmRun`] for one backend on one node — the compile-once/run-many split that makes the |
| `mycelium_bench::classify` | fn | `crates/mycelium-bench/src/verdict.rs:160` | Classify one (backend, case) pair. |
| `mycelium_bench::corpus` | mod | `crates/mycelium-bench/src/lib.rs:45` | — |
| `mycelium_bench::corpus` | fn | `crates/mycelium-bench/src/corpus.rs:104` | The full corpus, in a stable order. |
| `mycelium_bench::corpus::Case::elaborate` | fn | `crates/mycelium-bench/src/corpus.rs:94` | Parse, type-check and elaborate this case's source to a closed Core IR [`Node`] (the input |
| `mycelium_bench::corpus::CorpusError` | enum | `crates/mycelium-bench/src/corpus.rs:69` | An error from turning a corpus source into a Core IR term — kept explicit (never a silent skip) |
| `mycelium_bench::corpus::Fragment::label` | fn | `crates/mycelium-bench/src/corpus.rs:42` | A short human label for the report. |
| `mycelium_bench::llm` | mod | `crates/mycelium-bench/src/lib.rs:46` | — |
| `mycelium_bench::llm::GrokAblation` | struct | `crates/mycelium-bench/src/llm.rs:389` | The ablation block in the Grok report. |
| `mycelium_bench::llm::GrokHonestyPosture` | struct | `crates/mycelium-bench/src/llm.rs:288` | Honesty posture block in the Grok report. |
| `mycelium_bench::llm::GrokMetadata` | struct | `crates/mycelium-bench/src/llm.rs:263` | Harness metadata block in the Grok report. |
| `mycelium_bench::llm::GrokOutcome` | struct | `crates/mycelium-bench/src/llm.rs:358` | Per-task outcome in the Grok report. |
| `mycelium_bench::llm::GrokPerformance` | struct | `crates/mycelium-bench/src/llm.rs:331` | Aggregate performance metrics from the Grok report. |
| `mycelium_bench::llm::GrokQuality` | struct | `crates/mycelium-bench/src/llm.rs:307` | Aggregate quality metrics from the Grok report. |
| `mycelium_bench::llm::HonestyPosture` | struct | `crates/mycelium-bench/src/llm.rs:58` | The honesty posture the harness stamps into every report. |
| `mycelium_bench::llm::LlmIngestError` | enum | `crates/mycelium-bench/src/llm.rs:217` | A never-silent ingestion error. |
| `mycelium_bench::llm::LlmReport::from_json` | fn | `crates/mycelium-bench/src/llm.rs:178` | Parse a report from JSON text. |
| `mycelium_bench::llm::LlmReport::from_path` | fn | `crates/mycelium-bench/src/llm.rs:183` | Read + parse a report from a file path. |
| `mycelium_bench::llm::LlmReport::is_synthetic` | fn | `crates/mycelium-bench/src/llm.rs:150` | `true` when this report is a SYNTHETIC fixture run (no real model) — the primary honesty gate. |
| `mycelium_bench::llm::LlmReport::newest_in_dir` | fn | `crates/mycelium-bench/src/llm.rs:192` | Find the **newest** report under a harness reports directory (`*-report.json`, lexicographic |
| `mycelium_bench::llm::LlmReport::provenance` | fn | `crates/mycelium-bench/src/llm.rs:156` | A one-line provenance string for the unified report header. |
| `mycelium_bench::llm::Summary` | struct | `crates/mycelium-bench/src/llm.rs:71` | The report roll-up. |
| `mycelium_bench::llm::ValidationResult` | struct | `crates/mycelium-bench/src/llm.rs:96` | One validation result. |
| `mycelium_bench::llm::ValidationResult::is_mock` | fn | `crates/mycelium-bench/src/llm.rs:139` | Whether this result is a fixture (mock) result, not real-model evidence. |
| `mycelium_bench::llm::ValidationResult::token_counts` | fn | `crates/mycelium-bench/src/llm.rs:130` | The (prompt, generated) token counts this validation recorded, if present |
| `mycelium_bench::llm::ValidationResult::wall_seconds` | fn | `crates/mycelium-bench/src/llm.rs:116` | The wall-clock latency this validation recorded, in seconds, if present in `detail` |
| `mycelium_bench::measure` | mod | `crates/mycelium-bench/src/lib.rs:47` | — |
| `mycelium_bench::measure::BackendResult` | struct | `crates/mycelium-bench/src/measure.rs:38` | One backend's measured result on one case: its outcome, optional timing, and the classified |
| `mycelium_bench::measure::measure_case` | fn | `crates/mycelium-bench/src/measure.rs:104` | Measure all backends on one case and classify each vs the interpreter baseline. |
| `mycelium_bench::parse_any_llm_json` | fn | `crates/mycelium-bench/src/llm.rs:485` | Parse raw JSON text as *either* the Grok or the bench harness schema, returning a |
| `mycelium_bench::report` | mod | `crates/mycelium-bench/src/lib.rs:48` | — |
| `mycelium_bench::report::LlmSection::from_parsed` | fn | `crates/mycelium-bench/src/report.rs:129` | Build the section from the schema-agnostic [`crate::llm::ParsedLlmSection`] produced by |
| `mycelium_bench::report::LlmSection::from_report` | fn | `crates/mycelium-bench/src/report.rs:97` | Build the section from a parsed report + its source path / synthetic flag. |
| `mycelium_bench::report::LlmValidationRow` | struct | `crates/mycelium-bench/src/report.rs:77` | One per-validation row in the LLM section. |
| `mycelium_bench::report::LossRollup` | struct | `crates/mycelium-bench/src/report.rs:141` | A roll-up of losses for the "where we're losing" section. |
| `mycelium_bench::report::Report::loss_rollup` | fn | `crates/mycelium-bench/src/report.rs:153` | Roll up every loss across the run for the "where we're losing" section. |
| `mycelium_bench::report::Report::tallies` | fn | `crates/mycelium-bench/src/report.rs:186` | Count (wins, speed-losses, correctness-losses, capability-losses, skips) across the run. |
| `mycelium_bench::report::Report::to_json` | fn | `crates/mycelium-bench/src/report.rs:206` | The machine-readable JSON projection (pretty-printed, deterministic). |
| `mycelium_bench::report::Report::to_markdown` | fn | `crates/mycelium-bench/src/report.rs:213` | The human-readable markdown projection (deterministic — same run ⇒ same bytes, modulo the |
| `mycelium_bench::report::Tallies` | struct | `crates/mycelium-bench/src/report.rs:491` | Loss/win tallies across a run. |
| `mycelium_bench::report::neutral_band` | fn | `crates/mycelium-bench/src/report.rs:553` | The neutral-band constant, re-exported for the binary to stamp into the report metadata. |
| `mycelium_bench::run_corpus` | fn | `crates/mycelium-bench/src/measure.rs:146` | Run the whole corpus, in order. |
| `mycelium_bench::timing` | mod | `crates/mycelium-bench/src/lib.rs:49` | — |
| `mycelium_bench::timing::BATCHES:` | const | `crates/mycelium-bench/src/timing.rs:14` | Default number of timed batches; the fastest batch's mean is reported (least-noise estimate). |
| `mycelium_bench::timing::Timing` | struct | `crates/mycelium-bench/src/timing.rs:19` | A single backend/case timing result: the per-call nanoseconds and the trial accounting that makes |
| `mycelium_bench::timing::Timing::spread` | fn | `crates/mycelium-bench/src/timing.rs:34` | The observed best/worst spread ratio (`worst / best`), `1.0` when only one batch was timed or |
| `mycelium_bench::timing::bench` | fn | `crates/mycelium-bench/src/timing.rs:50` | Time `f`: `iters` warmup calls, then [`BATCHES`] timed batches of `iters` calls each; report the |
| `mycelium_bench::timing::is_debug_build` | fn | `crates/mycelium-bench/src/timing.rs:85` | `true` when this binary was compiled with debug assertions on (a debug build). |
| `mycelium_bench::timing::refuse_debug_build` | fn | `crates/mycelium-bench/src/timing.rs:92` | Refuse to produce perf numbers from a debug build — print the fix and exit `2`. |
| `mycelium_bench::verdict` | mod | `crates/mycelium-bench/src/lib.rs:50` | — |
| `mycelium_bench::verdict::NEUTRAL_BAND:` | const | `crates/mycelium-bench/src/verdict.rs:138` | The neutral band half-width: a backend within `[1/(1+NEUTRAL), 1+NEUTRAL]` of the interpreter's |
| `mycelium_bench::verdict::Verdict::guarantee_tag` | fn | `crates/mycelium-bench/src/verdict.rs:121` | The honest guarantee tag for this verdict. |
| `mycelium_bench::verdict::Verdict::is_loss` | fn | `crates/mycelium-bench/src/verdict.rs:103` | Whether this verdict counts as a LOSS (any of the three loss kinds) — for the "where we're |
| `mycelium_bench::verdict::Verdict::is_win` | fn | `crates/mycelium-bench/src/verdict.rs:114` | Whether this verdict counts as a WIN (a measured speed win). |
| `mycelium_bench::verdict::Verdict::status` | fn | `crates/mycelium-bench/src/verdict.rs:86` | A short status word for the report table. |

## mycelium-build

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_build::Arch` | enum | `crates/mycelium-build/src/target.rs:42` | A supported instruction-set architecture (the build-target arch dimension). |
| `mycelium_build::BuildCache` | struct | `crates/mycelium-build/src/cache.rs:43` | A content-addressed cache of build certificates, keyed by the build request's content address. |
| `mycelium_build::BuildCertificate` | struct | `crates/mycelium-build/src/lib.rs:141` | An inspectable, content-addressed record of one build decision (RFC-0004 §4; ADR-003). |
| `mycelium_build::BuildCertificate::blocked_on` | fn | `crates/mycelium-build/src/lib.rs:191` | The specific blocking reasons (empty when eligible). |
| `mycelium_build::BuildCertificate::cert_ref` | fn | `crates/mycelium-build/src/lib.rs:199` | The **content address** of this certificate (ADR-003 / RFC-0001 §4.6): the BLAKE3 hash of its |
| `mycelium_build::BuildCertificate::component` | fn | `crates/mycelium-build/src/lib.rs:161` | The component this certifies. |
| `mycelium_build::BuildCertificate::eligible` | fn | `crates/mycelium-build/src/lib.rs:176` | Whether the automatic §4 checks passed. |
| `mycelium_build::BuildCertificate::obligations` | fn | `crates/mycelium-build/src/lib.rs:171` | The recorded obligations. |
| `mycelium_build::BuildCertificate::promoted` | fn | `crates/mycelium-build/src/lib.rs:181` | Whether the component was explicitly promoted to stable. |
| `mycelium_build::BuildCertificate::route` | fn | `crates/mycelium-build/src/lib.rs:186` | The execution route. |
| `mycelium_build::BuildCertificate::spec_ratified` | fn | `crates/mycelium-build/src/lib.rs:166` | Whether the spec is ratified. |
| `mycelium_build::BuildError` | enum | `crates/mycelium-build/src/target.rs:157` | Why a profile's targets cannot be *realized* yet (RFC-0004 §9.3, honest scope). |
| `mycelium_build::BuildProfile` | enum | `crates/mycelium-build/src/target.rs:124` | A build's **target-set profile** (RFC-0004 §9.2): how many platforms to build for. |
| `mycelium_build::CacheOutcome` | enum | `crates/mycelium-build/src/cache.rs:18` | The outcome of a cached build — and whether it was served from cache. |
| `mycelium_build::Component` | struct | `crates/mycelium-build/src/lib.rs:91` | A candidate definition for the stable/experimental decision (RFC-0004 §4). |
| `mycelium_build::DispatchMiss` | struct | `crates/mycelium-build/src/target.rs:229` | A runtime dispatch **miss**: the host matched no present variant (RFC-0004 §9.3). |
| `mycelium_build::Eligibility` | enum | `crates/mycelium-build/src/lib.rs:103` | The automatic-check verdict (RFC-0004 §4): whether the §4 conditions hold. |
| `mycelium_build::ExecutionRoute` | enum | `crates/mycelium-build/src/lib.rs:35` | The execution route a component takes (RFC-0004 §4 / ADR-009). |
| `mycelium_build::Obligations` | struct | `crates/mycelium-build/src/lib.rs:48` | The RFC-0004 §4 verification obligations for a definition. |
| `mycelium_build::Os` | enum | `crates/mycelium-build/src/target.rs:31` | A supported operating system (the build-target OS dimension). |
| `mycelium_build::Target` | struct | `crates/mycelium-build/src/target.rs:53` | A build target: an `(os, arch)` pair. |
| `mycelium_build::VariantTable` | struct | `crates/mycelium-build/src/target.rs:221` | A **fat (multi-target) artifact's** per-target variant table (RFC-0004 §9.3): each compiled |
| `mycelium_build::cache` | mod | `crates/mycelium-build/src/lib.rs:25` | — |
| `mycelium_build::cache::BuildCache::build` | fn | `crates/mycelium-build/src/cache.rs:74` | Build `c` (promoting or not), serving the cached certificate on a hit or deciding-then-storing |
| `mycelium_build::cache::BuildCache::is_empty` | fn | `crates/mycelium-build/src/cache.rs:92` | Whether the cache is empty. |
| `mycelium_build::cache::BuildCache::len` | fn | `crates/mycelium-build/src/cache.rs:86` | The number of distinct requests cached. |
| `mycelium_build::cache::BuildCache::new` | fn | `crates/mycelium-build/src/cache.rs:50` | An empty cache. |
| `mycelium_build::cache::BuildCache::request_key` | fn | `crates/mycelium-build/src/cache.rs:58` | The content address of a build request: the component's identity hash folded with every |
| `mycelium_build::cache::CacheOutcome::certificate` | fn | `crates/mycelium-build/src/cache.rs:28` | The certificate, regardless of hit/miss. |
| `mycelium_build::cache::CacheOutcome::was_hit` | fn | `crates/mycelium-build/src/cache.rs:36` | Whether this was a cache hit. |
| `mycelium_build::check_eligibility` | fn | `crates/mycelium-build/src/lib.rs:114` | Run the automatic §4 eligibility checks. |
| `mycelium_build::decide` | fn | `crates/mycelium-build/src/lib.rs:266` | Decide a component's build and emit its certificate (RFC-0004 §4). |
| `mycelium_build::realizable_targets` | fn | `crates/mycelium-build/src/target.rs:197` | The targets a profile can be **realized** to *today* (RFC-0004 §9.3). |
| `mycelium_build::supported_targets` | fn | `crates/mycelium-build/src/target.rs:111` | All supported `(os, arch)` targets — the universe `--fat` builds for. |
| `mycelium_build::target` | mod | `crates/mycelium-build/src/lib.rs:26` | — |
| `mycelium_build::target::BuildProfile::is_compiled` | fn | `crates/mycelium-build/src/target.rs:150` | Whether this profile compiles anything at all (`false` for `Interpret`). |
| `mycelium_build::target::BuildProfile::targets` | fn | `crates/mycelium-build/src/target.rs:139` | The concrete target set this profile resolves to (`Interpret` → empty; `Fat` → |
| `mycelium_build::target::Target::host` | fn | `crates/mycelium-build/src/target.rs:71` | The target the build tool is itself running on, if it is a supported `(os, arch)` — `None` |
| `mycelium_build::target::Target::new` | fn | `crates/mycelium-build/src/target.rs:63` | Construct a target. |
| `mycelium_build::target::VariantTable::insert` | fn | `crates/mycelium-build/src/target.rs:255` | Record a target's compiled-variant artifact hash. |
| `mycelium_build::target::VariantTable::is_empty` | fn | `crates/mycelium-build/src/target.rs:272` | Whether the table is empty (an interpret-only artifact). |
| `mycelium_build::target::VariantTable::len` | fn | `crates/mycelium-build/src/target.rs:266` | The number of variants (1 for slim, \|targets\| for fat). |
| `mycelium_build::target::VariantTable::select` | fn | `crates/mycelium-build/src/target.rs:279` | **Runtime variant dispatch** (RFC-0004 §9.3): the artifact hash for `host`, or an explicit |

## mycelium-cert

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_cert::BF16_MIN_NORMAL:` | const | `crates/mycelium-cert/src/dense.rs:34` | Smallest positive *normal* bfloat16 (same exponent range as f32): `2^−126`. |
| `mycelium_cert::BF16_REL_EPS:` | const | `crates/mycelium-cert/src/dense.rs:30` | The proven per-element relative rounding bound: the unit roundoff `u = β^(1−p)/2 = 2^(1−8)/2 = |
| `mycelium_cert::BinTernParams` | struct | `crates/mycelium-cert/src/lib.rs:43` | Concrete parameters binding a bijection lemma to one use — `{ width, trits }` for binary↔ternary |
| `mycelium_cert::BinaryTernarySwapEngine` | struct | `crates/mycelium-cert/src/lib.rs:360` | A [`SwapEngine`] for the reference interpreter that performs the |
| `mycelium_cert::CertifiedSwapEngine` | struct | `crates/mycelium-cert/src/lib.rs:395` | A [`SwapEngine`] over the **complete certified swap surface** (SC-3 global, M-212): the |
| `mycelium_cert::CheckVerdict` | enum | `crates/mycelium-cert/src/check.rs:110` | The checker's verdict. |
| `mycelium_cert::DENSE_VSA_DEFAULT_DELTA:` | const | `crates/mycelium-cert/src/lib.rs:386` | The δ the engine requests for a Dense↔VSA swap when no policy channel supplies one — the same |
| `mycelium_cert::DENSE_VSA_EMP_DELTA:` | const | `crates/mycelium-cert/src/dense_vsa.rs:48` | Empirical profile — the validated δ. |
| `mycelium_cert::DENSE_VSA_MODEL:` | const | `crates/mycelium-cert/src/dense_vsa.rs:41` | The VSA model the swap targets (the atoms are bipolar and the encoding is the MAP-I additive |
| `mycelium_cert::Evidence` | enum | `crates/mycelium-cert/src/check.rs:58` | The evidence presented to the checker — the *certificate* of `(A, B, R, claimed, certificate)`. |
| `mycelium_cert::Fallback` | enum | `crates/mycelium-cert/src/check.rs:69` | The explicit fallback path when validation fails — required by RFC-0002 §2 (TV incompleteness |
| `mycelium_cert::NotValidatedReason` | enum | `crates/mycelium-cert/src/check.rs:79` | Why the checker did not validate. |
| `mycelium_cert::RefinementRelation` | enum | `crates/mycelium-cert/src/check.rs:45` | The relation `R` under which `B` claims to refine `A` (RFC-0002 §2). |
| `mycelium_cert::SwapCertificate` | enum | `crates/mycelium-cert/src/lib.rs:54` | The inspectable certificate every swap produces (RFC-0002 §3/§5; `swap-certificate.schema.json`). |
| `mycelium_cert::SwapError` | enum | `crates/mycelium-cert/src/lib.rs:83` | Why a swap could not be performed — always explicit (SC-3; G2), never a silent coercion. |
| `mycelium_cert::binary_to_ternary` | fn | `crates/mycelium-cert/src/lib.rs:270` | `enc`: encode an `n`-bit two's-complement [`Value`] into `m` balanced trits over a legal pair. |
| `mycelium_cert::check` | mod | `crates/mycelium-cert/src/lib.rs:22` | — |
| `mycelium_cert::check` | fn | `crates/mycelium-cert/src/check.rs:158` | The single shared checker: does artifact `B` refine reference `A` under `relation` within the |
| `mycelium_cert::check_core` | fn | `crates/mycelium-cert/src/check.rs:293` | Observational equivalence over a whole [`CoreValue`] (RFC-0011 §4.6; NFR-7) — the M-151/M-210 |
| `mycelium_cert::dense` | mod | `crates/mycelium-cert/src/lib.rs:23` | — |
| `mycelium_cert::dense_f32_to_bf16` | fn | `crates/mycelium-cert/src/dense.rs:81` | The Dense `F32 → BF16` rounding swap: returns the converted value and a |
| `mycelium_cert::dense_to_vsa` | fn | `crates/mycelium-cert/src/dense_vsa.rs:139` | Encode a bipolar `Dense{n, F32}` value into a `Vsa{MAP-I, vsa_dim}` superposition, emitting a |
| `mycelium_cert::dense_vsa` | mod | `crates/mycelium-cert/src/lib.rs:24` | — |
| `mycelium_cert::dense_vsa::DENSE_VSA_EMP_DIM_FACTOR:` | const | `crates/mycelium-cert/src/dense_vsa.rs:46` | Empirical profile — minimum `vsa_dim / components` ratio covered by the trials. |
| `mycelium_cert::dense_vsa::DENSE_VSA_EMP_MAX_COMPONENTS:` | const | `crates/mycelium-cert/src/dense_vsa.rs:44` | Empirical profile — maximum Dense components covered by the trials. |
| `mycelium_cert::dense_vsa::DENSE_VSA_EMP_METHOD:` | const | `crates/mycelium-cert/src/dense_vsa.rs:52` | Empirical profile — the method recorded in the `EmpiricalFit` basis. |
| `mycelium_cert::dense_vsa::DENSE_VSA_EMP_TRIALS:` | const | `crates/mycelium-cert/src/dense_vsa.rs:50` | Empirical profile — the trial count `tests/dense_vsa.rs` runs. |
| `mycelium_cert::legal_pair` | fn | `crates/mycelium-cert/src/lib.rs:234` | Whether `(n, m)` admits a lossless binary→ternary swap: `B_n ⊆ T_m ⇔ 2^(n-1) ≤ (3^m − 1)/2` |
| `mycelium_cert::roundtrip_lemma_ref` | fn | `crates/mycelium-cert/src/lib.rs:247` | The content hash of the once-per-swap-kind binary↔ternary round-trip lemma (P1/P2, |
| `mycelium_cert::ternary_to_binary` | fn | `crates/mycelium-cert/src/lib.rs:314` | `dec`: decode `m` balanced trits back into an `n`-bit two's-complement [`Value`]. |
| `mycelium_cert::vsa_to_dense` | fn | `crates/mycelium-cert/src/dense_vsa.rs:195` | Decode a `swap.dense_vsa.enc.v1` product back to a bipolar `Dense{components, F32}` value by |

## mycelium-check

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_check::Finding` | struct | `crates/mycelium-check/src/lib.rs:34` | One aggregated diagnostic. |
| `mycelium_check::Finding::with_route` | fn | `crates/mycelium-check/src/lib.rs:52` | Attach a baseline route, fluently (M-644 ergonomics). |
| `mycelium_check::FindingKind` | enum | `crates/mycelium-check/src/lib.rs:25` | What kind of refusal a finding records. |
| `mycelium_check::Report` | struct | `crates/mycelium-check/src/lib.rs:60` | The aggregated result of checking a set of sources. |
| `mycelium_check::Report::exit_code` | fn | `crates/mycelium-check/src/lib.rs:91` | The CI exit code: 2 if any parse error, else 3 if any check error, else 0. |
| `mycelium_check::Report::is_ok` | fn | `crates/mycelium-check/src/lib.rs:85` | Whether the report is clean (no findings). |
| `mycelium_check::Report::with_files_checked` | fn | `crates/mycelium-check/src/lib.rs:78` | Set the checked-file count, fluently (M-644 ergonomics). |
| `mycelium_check::Report::with_finding` | fn | `crates/mycelium-check/src/lib.rs:71` | Push a finding, fluently (M-644 ergonomics). |
| `mycelium_check::ResolveError` | struct | `crates/mycelium-check/src/lib.rs:104` | A project-resolution failure — no/ambiguous input (no sources, an unreadable file). |
| `mycelium_check::check_project` | fn | `crates/mycelium-check/src/lib.rs:189` | Resolve and check a whole project: every `.myc` under `dir`. |
| `mycelium_check::check_source` | fn | `crates/mycelium-check/src/lib.rs:116` | Check one source, appending any finding. |
| `mycelium_check::check_source_default` | fn | `crates/mycelium-check/src/lib.rs:162` | Check one source under the **default baseline policy** — the M-644 ergonomic convenience for the |
| `mycelium_check::check_sources` | fn | `crates/mycelium-check/src/lib.rs:170` | Check an explicit set of `(path, contents)` sources, aggregating findings deterministically. |

## mycelium-cli-common

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_cli_common::Args` | struct | `crates/mycelium-cli-common/src/lib.rs:133` | A transparent cursor over `std::env::args().skip(1)` that names the one repeated arg-parsing idiom: |
| `mycelium_cli_common::Args::from_args` | fn | `crates/mycelium-cli-common/src/lib.rs:148` | Cursor over an explicit argument list (skips nothing — pass the post-`argv[0]` tokens). |
| `mycelium_cli_common::Args::from_env` | fn | `crates/mycelium-cli-common/src/lib.rs:141` | Cursor over the process arguments, skipping `argv[0]` (the program name) — exactly the bins' |
| `mycelium_cli_common::Args::next` | fn | `crates/mycelium-cli-common/src/lib.rs:162` | The next raw token, or `None` at the end. |
| `mycelium_cli_common::Args::value` | fn | `crates/mycelium-cli-common/src/lib.rs:170` | The **value** for the just-seen value-flag: the next token, or `None` if the flag was last. |
| `mycelium_cli_common::ReadError` | struct | `crates/mycelium-cli-common/src/lib.rs:41` | The error from [`read_source`]: the diagnostic has **already been emitted** to stderr (never-silent, |
| `mycelium_cli_common::STDIN_SENTINEL:` | const | `crates/mycelium-cli-common/src/lib.rs:34` | The conventional sentinel that means "read standard input" across the toolchain CLIs. |
| `mycelium_cli_common::read_source` | fn | `crates/mycelium-cli-common/src/lib.rs:58` | Read one source: standard input when `path == "-"`, otherwise the file at `path`. |
| `mycelium_cli_common::walk_myc` | fn | `crates/mycelium-cli-common/src/lib.rs:84` | Collect every `.myc` file under `dir`, recursively, **sorted**. |

## mycelium-core

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_core::Alt` | enum | `crates/mycelium-core/src/node.rs:144` | One alternative of a flat [`Node::Match`] (RFC-0011 §4.1): a constructor arm (binding exactly the |
| `mycelium_core::Bound` | struct | `crates/mycelium-core/src/bound.rs:105` | A sound bound plus the basis by which it was obtained (ADR-011: `basis` is universal). |
| `mycelium_core::BoundBasis` | enum | `crates/mycelium-core/src/bound.rs:18` | How a bound was obtained — this determines the honest [`crate::GuaranteeStrength`]. |
| `mycelium_core::BoundKind` | enum | `crates/mycelium-core/src/bound.rs:68` | The bound payload, per kind (RFC-0001 §4.3). |
| `mycelium_core::CleanupShape` | enum | `crates/mycelium-core/src/recon.rs:59` | The per-slot cleanup projection a resonator decode uses (RFC-0003 §6.1; RFC-0009 §3/§9 Q2). |
| `mycelium_core::ContentHash` | struct | `crates/mycelium-core/src/id.rs:9` | A content address, e.g. |
| `mycelium_core::CoreValue` | enum | `crates/mycelium-core/src/datum.rs:90` | A runtime value: a representation [`Value`] (one of the four paradigm kinds) or an algebraic |
| `mycelium_core::CtorDecl` | struct | `crates/mycelium-core/src/data.rs:81` | One constructor of a resolved declaration: its field types, in declaration order. |
| `mycelium_core::CtorRef` | struct | `crates/mycelium-core/src/data.rs:37` | A constructor reference `#T#i` (RFC-0007 §4.2): the content hash of a data declaration and the |
| `mycelium_core::CtorSpec` | struct | `crates/mycelium-core/src/data.rs:108` | A build-time constructor spec: its fields, in declaration order. |
| `mycelium_core::DataDecl` | struct | `crates/mycelium-core/src/data.rs:89` | A resolved, content-addressed data declaration: its constructors in declaration order (the index |
| `mycelium_core::DataRegistry` | struct | `crates/mycelium-core/src/data.rs:152` | The content-addressed data registry `Σ` (RFC-0001 §4.3 r3): the resolved declarations keyed by |
| `mycelium_core::Datum` | struct | `crates/mycelium-core/src/datum.rs:29` | A constructed data value: a saturated constructor application (RFC-0011 §4.1, W6) with a |
| `mycelium_core::DeclSpec` | struct | `crates/mycelium-core/src/data.rs:115` | A build-time declaration spec: its constructors, in declaration order. |
| `mycelium_core::DecodeProcedure` | enum | `crates/mycelium-core/src/recon.rs:49` | The decoding procedure. |
| `mycelium_core::DecodeSpec` | struct | `crates/mycelium-core/src/recon.rs:79` | Decoding procedure + parameters: a cleanup threshold (indexed/cleanup) or a resonator factor |
| `mycelium_core::FieldSpec` | enum | `crates/mycelium-core/src/data.rs:99` | A build-time field spec: a representation field, or a data field referencing another declaration |
| `mycelium_core::FieldTy` | enum | `crates/mycelium-core/src/data.rs:72` | A field type within a resolved declaration: a representation type, or a (possibly cyclic) data |
| `mycelium_core::GuaranteeStrength` | enum | `crates/mycelium-core/src/guarantee.rs:16` | How trustworthy a value's representation/bound is. |
| `mycelium_core::InitStrategy` | enum | `crates/mycelium-core/src/recon.rs:68` | The resonator initialisation strategy (RFC-0003 §6.1; RFC-0009 §9 Q1). |
| `mycelium_core::Meta` | struct | `crates/mycelium-core/src/meta.rs:88` | Runtime, queryable metadata (RFC-0001 §4.3). |
| `mycelium_core::Names` | struct | `crates/mycelium-core/src/content.rs:453` | The separable `hash ↔ name` side-table (RFC-0001 §4.6, "names-as-metadata"). |
| `mycelium_core::Node` | enum | `crates/mycelium-core/src/node.rs:37` | A Core IR node. |
| `mycelium_core::NormKind` | enum | `crates/mycelium-core/src/bound.rs:53` | Norm in which an [`BoundKind::Error`] `eps` is expressed (extensible registry; RFC-0001 §4.3 r2). |
| `mycelium_core::PackScheme` | enum | `crates/mycelium-core/src/meta.rs:44` | Lossless physical packing schemes (extensible registry; RFC-0001 §4.3; DN-01). |
| `mycelium_core::Payload` | enum | `crates/mycelium-core/src/value.rs:55` | Representation-specific payload. |
| `mycelium_core::PhysicalLayout` | enum | `crates/mycelium-core/src/meta.rs:65` | The recorded schedule-staged packing (RFC-0001 §4.3; RFC-0004 §5). |
| `mycelium_core::PolicyRef` | type | `crates/mycelium-core/src/node.rs:33` | A reference to the selection policy a swap used (RFC-0005), as a content hash. |
| `mycelium_core::Prim` | type | `crates/mycelium-core/src/node.rs:31` | A primitive operator name; each declares its operand/result paradigms (RFC-0001 §4.5). |
| `mycelium_core::PrimDecl` | struct | `crates/mycelium-core/src/prim.rs:79` | A resolved, content-addressed prim declaration: its signature and the *intrinsic guarantee* `g_f` |
| `mycelium_core::PrimParadigm` | enum | `crates/mycelium-core/src/prim.rs:36` | The representation paradigm of a prim operand or result (the `τ`'s paradigm in `Π(p)`). |
| `mycelium_core::PrimRef` | struct | `crates/mycelium-core/src/prim.rs:103` | A prim reference `#p` (the prim analogue of CtorRef `#T#i`): the content hash |
| `mycelium_core::PrimSig` | struct | `crates/mycelium-core/src/prim.rs:58` | A prim's signature `Π(p) = (τ₁…τₙ) → τ` (RFC-0007 §4.4): the per-operand paradigms (arity is their |
| `mycelium_core::PrimTable` | struct | `crates/mycelium-core/src/prim.rs:134` | The content-addressed **prim table `Π`** (RFC-0007 §4.4; R7-Q4): resolved declarations keyed by |
| `mycelium_core::Provenance` | enum | `crates/mycelium-core/src/meta.rs:20` | Provenance: an acyclic derivation DAG (RFC-0001 §4.6). |
| `mycelium_core::Recipe` | struct | `crates/mycelium-core/src/recon.rs:40` | The compositional recipe / role schema: which ops combined which slots. |
| `mycelium_core::ReconInfo` | struct | `crates/mycelium-core/src/recon.rs:111` | The reconstruction manifest. |
| `mycelium_core::ReconMode` | enum | `crates/mycelium-core/src/recon.rs:30` | Which capability the manifest supports (RFC-0003 §6). |
| `mycelium_core::RegistryError` | enum | `crates/mycelium-core/src/data.rs:122` | Why building a [`DataRegistry`] from specs failed — always explicit (never a silent drop). |
| `mycelium_core::Repr` | enum | `crates/mycelium-core/src/repr.rs:57` | The four closed paradigm kinds (RFC-0001 §4.1). |
| `mycelium_core::ScalarKind` | enum | `crates/mycelium-core/src/repr.rs:14` | Scalar element kind for `Dense` values (extensible registry). |
| `mycelium_core::SparsityClass` | enum | `crates/mycelium-core/src/repr.rs:44` | Declared sparsity class of a VSA value (RFC-0001 §4.1; RFC-0003 §5). |
| `mycelium_core::SparsityObs` | struct | `crates/mycelium-core/src/meta.rs:34` | Measured (dynamic) sparsity — distinct from the declared [`crate::repr::SparsityClass`]. |
| `mycelium_core::Trit` | enum | `crates/mycelium-core/src/value.rs:19` | A balanced trit in `{-1, 0, +1}`. |
| `mycelium_core::Value` | struct | `crates/mycelium-core/src/value.rs:134` | A Mycelium value. |
| `mycelium_core::VarId` | type | `crates/mycelium-core/src/node.rs:29` | A variable identifier (a name; not part of content identity — RFC-0001 §4.6). |
| `mycelium_core::WfError` | enum | `crates/mycelium-core/src/lib.rs:49` | Well-formedness errors for Core IR construction (RFC-0001 §4.3/§4.5 invariants). |
| `mycelium_core::WidthRel` | enum | `crates/mycelium-core/src/prim.rs:50` | How a prim's operand and result *widths* relate. |
| `mycelium_core::binary` | mod | `crates/mycelium-core/src/lib.rs:13` | — |
| `mycelium_core::binary::bits_to_int` | fn | `crates/mycelium-core/src/binary.rs:10` | The signed two's-complement value of an MSB-first bit string. |
| `mycelium_core::binary::int_to_bits` | fn | `crates/mycelium-core/src/binary.rs:29` | The `n`-bit two's-complement representation of `value`, MSB-first — or `None` if `value` is |
| `mycelium_core::bound` | mod | `crates/mycelium-core/src/lib.rs:14` | — |
| `mycelium_core::bound::Bound::well_formed` | fn | `crates/mycelium-core/src/bound.rs:119` | Well-formedness per `bound.schema.json`: the payload ranges (magnitudes finite and in range) |
| `mycelium_core::bound::BoundBasis::strength` | fn | `crates/mycelium-core/src/bound.rs:42` | The honest [`GuaranteeStrength`] this basis implies (M-I2/M-I3/M-I4): the basis *is* the |
| `mycelium_core::content` | mod | `crates/mycelium-core/src/lib.rs:15` | — |
| `mycelium_core::content::Names::bind` | fn | `crates/mycelium-core/src/content.rs:468` | Bind a human name to a content hash, returning any previous name for that hash. |
| `mycelium_core::content::Names::is_empty` | fn | `crates/mycelium-core/src/content.rs:486` | Whether the table is empty. |
| `mycelium_core::content::Names::len` | fn | `crates/mycelium-core/src/content.rs:480` | Number of bound names. |
| `mycelium_core::content::Names::name_of` | fn | `crates/mycelium-core/src/content.rs:474` | The name bound to `hash`, if any. |
| `mycelium_core::content::Names::new` | fn | `crates/mycelium-core/src/content.rs:460` | An empty name table. |
| `mycelium_core::data` | mod | `crates/mycelium-core/src/lib.rs:16` | — |
| `mycelium_core::data::CtorRef::decl` | fn | `crates/mycelium-core/src/data.rs:51` | The referenced data declaration's content hash (`#T`). |
| `mycelium_core::data::CtorRef::index` | fn | `crates/mycelium-core/src/data.rs:57` | The constructor's index within its declaration (`#i`). |
| `mycelium_core::data::CtorRef::new` | fn | `crates/mycelium-core/src/data.rs:45` | Build a constructor reference from a declaration hash and a constructor index. |
| `mycelium_core::data::DataRegistry::build` | fn | `crates/mycelium-core/src/data.rs:163` | Build the registry from a set of named declaration specs, computing every declaration's |
| `mycelium_core::data::DataRegistry::ctor` | fn | `crates/mycelium-core/src/data.rs:267` | The constructor declaration a [`CtorRef`] points at, if registered and in range. |
| `mycelium_core::data::DataRegistry::ctor_count` | fn | `crates/mycelium-core/src/data.rs:281` | The number of constructors of the data type the [`CtorRef`] belongs to (for WF7 coverage). |
| `mycelium_core::data::DataRegistry::ctor_ref` | fn | `crates/mycelium-core/src/data.rs:249` | A [`CtorRef`] for constructor `index` of the declaration named `name`, if the declaration is |
| `mycelium_core::data::DataRegistry::decl_hash` | fn | `crates/mycelium-core/src/data.rs:242` | The content hash of the declaration registered under build-time name `name`, if any. |
| `mycelium_core::data::DataRegistry::field_count` | fn | `crates/mycelium-core/src/data.rs:275` | The number of fields the referenced constructor takes (its saturation arity, WF6). |
| `mycelium_core::datum` | mod | `crates/mycelium-core/src/lib.rs:17` | — |
| `mycelium_core::datum::CoreValue::as_data` | fn | `crates/mycelium-core/src/datum.rs:109` | The underlying datum, if this is a [`CoreValue::Data`]. |
| `mycelium_core::datum::CoreValue::as_repr` | fn | `crates/mycelium-core/src/datum.rs:100` | The underlying representation value, if this is a [`CoreValue::Repr`]. |
| `mycelium_core::datum::Datum::content_hash` | fn | `crates/mycelium-core/src/datum.rs:80` | The identity-bearing content hash of this datum: its constructor reference and its fields' |
| `mycelium_core::datum::Datum::ctor` | fn | `crates/mycelium-core/src/datum.rs:52` | The constructor reference (`#T#i`). |
| `mycelium_core::datum::Datum::fields` | fn | `crates/mycelium-core/src/datum.rs:58` | The field values, in declaration order. |
| `mycelium_core::datum::Datum::guarantee` | fn | `crates/mycelium-core/src/datum.rs:64` | The meet-summary guarantee. |
| `mycelium_core::datum::Datum::meet_guarantee` | fn | `crates/mycelium-core/src/datum.rs:71` | This datum with its summary guarantee met against `g` (weakest-wins). |
| `mycelium_core::datum::Datum::new` | fn | `crates/mycelium-core/src/datum.rs:41` | Construct a datum from a constructor reference and its field values, computing the |
| `mycelium_core::guarantee` | mod | `crates/mycelium-core/src/lib.rs:18` | — |
| `mycelium_core::guarantee::GuaranteeStrength::ALL:` | const | `crates/mycelium-core/src/guarantee.rs:33` | All four strengths, strongest-to-weakest — for exhaustive iteration in tests and tooling. |
| `mycelium_core::guarantee::GuaranteeStrength::TOP:` | const | `crates/mycelium-core/src/guarantee.rs:30` | The strongest strength — the identity of meet and the unit of |
| `mycelium_core::guarantee::GuaranteeStrength::meet` | fn | `crates/mycelium-core/src/guarantee.rs:61` | The lattice **meet** (greatest lower bound): the *weakest* of `self` and `other` |
| `mycelium_core::guarantee::GuaranteeStrength::meet_all` | fn | `crates/mycelium-core/src/guarantee.rs:88` | The meet of a sequence of strengths, weakest-wins, starting from TOP |
| `mycelium_core::guarantee::GuaranteeStrength::propagate` | fn | `crates/mycelium-core/src/guarantee.rs:77` | Propagate guarantees through an operation (RFC-0001 §4.7): |
| `mycelium_core::guarantee::GuaranteeStrength::rank` | fn | `crates/mycelium-core/src/guarantee.rs:43` | Lattice rank, `0` = strongest (`Exact`) … `3` = weakest (`Declared`). |
| `mycelium_core::id` | mod | `crates/mycelium-core/src/lib.rs:19` | — |
| `mycelium_core::id::ContentHash::algo` | fn | `crates/mycelium-core/src/id.rs:45` | The algorithm tag (the part before `:`), e.g. |
| `mycelium_core::id::ContentHash::as_str` | fn | `crates/mycelium-core/src/id.rs:57` | The address as a string slice. |
| `mycelium_core::id::ContentHash::digest` | fn | `crates/mycelium-core/src/id.rs:51` | The digest (the part after `:`). |
| `mycelium_core::id::ContentHash::from_parts` | fn | `crates/mycelium-core/src/id.rs:39` | Build a content address from an algorithm tag and digest, validating the shape (`algo` is |
| `mycelium_core::id::ContentHash::parse` | fn | `crates/mycelium-core/src/id.rs:15` | Parse a content address, validating its shape: `algo` is `[a-z0-9]+`, `digest` is |
| `mycelium_core::lower` | mod | `crates/mycelium-core/src/lib.rs:20` | — |
| `mycelium_core::lower::Anf` | struct | `crates/mycelium-core/src/lower.rs:567` | A flattened (A-normal-form) lowering of a Core IR node. |
| `mycelium_core::lower::Anf::bindings` | fn | `crates/mycelium-core/src/lower.rs:909` | The ordered bindings (for backends consuming the lowered IR — M-150). |
| `mycelium_core::lower::Anf::dump` | fn | `crates/mycelium-core/src/lower.rs:873` | The canonical, diffable dump of the substrate stage (SC-4). |
| `mycelium_core::lower::Anf::is_empty` | fn | `crates/mycelium-core/src/lower.rs:903` | Whether there are no bindings. |
| `mycelium_core::lower::Anf::len` | fn | `crates/mycelium-core/src/lower.rs:897` | Number of bindings (for tests/tooling). |
| `mycelium_core::lower::Anf::result` | fn | `crates/mycelium-core/src/lower.rs:915` | The result operand. |
| `mycelium_core::lower::AnfAlt` | enum | `crates/mycelium-core/src/lower.rs:533` | One alternative of a lowered [`Rhs::Match`] — the ANF analogue of [`crate::node::Alt`], with the |
| `mycelium_core::lower::Atom` | enum | `crates/mycelium-core/src/lower.rs:434` | An operand of a lowered binding: a reference to a named/temp binding. |
| `mycelium_core::lower::Atom::render` | fn | `crates/mycelium-core/src/lower.rs:444` | The canonical textual rendering of this operand (`name` or `%k`). |
| `mycelium_core::lower::Binding` | struct | `crates/mycelium-core/src/lower.rs:556` | One lowered binding: a name, its right-hand side, and (where statically known) its scheduled |
| `mycelium_core::lower::Rhs` | enum | `crates/mycelium-core/src/lower.rs:454` | The right-hand side of a lowered binding. |
| `mycelium_core::lower::Stage` | struct | `crates/mycelium-core/src/lower.rs:31` | One lowering stage: a name and its canonical, diffable textual dump. |
| `mycelium_core::lower::dump_node` | fn | `crates/mycelium-core/src/lower.rs:153` | The canonical, deterministic textual rendering of a Core IR node (the `core` stage). |
| `mycelium_core::lower::format` | fn | `crates/mycelium-core/src/lower.rs:169` | The **canonical formatter** (M-142; RFC-0001 §4.8; ADR-003). |
| `mycelium_core::lower::lower_to_anf` | fn | `crates/mycelium-core/src/lower.rs:586` | Lower a Core IR node into A-normal form (flatten nested nodes to a linear binding list). |
| `mycelium_core::lower::schedule` | fn | `crates/mycelium-core/src/lower.rs:41` | The default schedule-staged packing for a representation (RFC-0004 §5; DN-01). |
| `mycelium_core::lower::stages` | fn | `crates/mycelium-core/src/lower.rs:56` | Run the lowering pipeline, returning every stage in order (currently `core` → `substrate`). |
| `mycelium_core::meta` | mod | `crates/mycelium-core/src/lib.rs:21` | — |
| `mycelium_core::meta::Meta::bound` | fn | `crates/mycelium-core/src/meta.rs:188` | The bound, if approximate. |
| `mycelium_core::meta::Meta::exact` | fn | `crates/mycelium-core/src/meta.rs:164` | The common `Exact` metadata with no bound (M-I1). |
| `mycelium_core::meta::Meta::guarantee` | fn | `crates/mycelium-core/src/meta.rs:183` | The disclosed guarantee strength. |
| `mycelium_core::meta::Meta::new` | fn | `crates/mycelium-core/src/meta.rs:106` | Build a `Meta`, enforcing the guarantee/bound invariants: |
| `mycelium_core::meta::Meta::physical` | fn | `crates/mycelium-core/src/meta.rs:198` | The recorded physical layout, if any. |
| `mycelium_core::meta::Meta::policy_used` | fn | `crates/mycelium-core/src/meta.rs:208` | The policy that produced this value (set iff produced by a swap). |
| `mycelium_core::meta::Meta::provenance` | fn | `crates/mycelium-core/src/meta.rs:178` | The value's provenance. |
| `mycelium_core::meta::Meta::reconstruction` | fn | `crates/mycelium-core/src/meta.rs:203` | The reconstruction manifest, if attached (RFC-0003 §6). |
| `mycelium_core::meta::Meta::sparsity` | fn | `crates/mycelium-core/src/meta.rs:193` | Measured sparsity, if recorded. |
| `mycelium_core::meta::Meta::with_physical` | fn | `crates/mycelium-core/src/meta.rs:157` | Record the schedule-staged packing chosen at a lowering stage (RFC-0004 §5; DN-01; |
| `mycelium_core::meta::Meta::with_reconstruction` | fn | `crates/mycelium-core/src/meta.rs:142` | Attach a reconstruction manifest (RFC-0003 §6; M-260). |
| `mycelium_core::node` | mod | `crates/mycelium-core/src/lib.rs:22` | — |
| `mycelium_core::node::Node::content_hash` | fn | `crates/mycelium-core/src/content.rs:417` | The content hash of this value's *identity-bearing* content: its [`Repr`] and payload, with |
| `mycelium_core::node::Node::is_aot_lowerable` | fn | `crates/mycelium-core/src/node.rs:182` | Whether this whole node is in the **AOT-lowerable** fragment — i.e. |
| `mycelium_core::node::Node::is_repr_changing` | fn | `crates/mycelium-core/src/node.rs:169` | Whether this node is the (only) representation-changing node, [`Node::Swap`] (WF1). |
| `mycelium_core::operation_hash` | fn | `crates/mycelium-core/src/content.rs:442` | The content address of a *primitive operation* identified by its name — for the `op` field of a |
| `mycelium_core::prim` | mod | `crates/mycelium-core/src/lib.rs:23` | — |
| `mycelium_core::prim::PrimDecl::content_hash` | fn | `crates/mycelium-core/src/prim.rs:92` | The content hash of this declaration's identity-bearing content (signature + intrinsic |
| `mycelium_core::prim::PrimRef::decl` | fn | `crates/mycelium-core/src/prim.rs:114` | The referenced declaration's content hash. |
| `mycelium_core::prim::PrimRef::new` | fn | `crates/mycelium-core/src/prim.rs:108` | Build a prim reference from a declaration hash. |
| `mycelium_core::prim::PrimSig::arity` | fn | `crates/mycelium-core/src/prim.rs:70` | The prim's arity (operand count). |
| `mycelium_core::prim::PrimTable::builtins` | fn | `crates/mycelium-core/src/prim.rs:162` | The default table: the closed v0 kernel-prim set — the identity, the elementwise binary |
| `mycelium_core::prim::PrimTable::contains` | fn | `crates/mycelium-core/src/prim.rs:227` | Whether a prim named `name` is registered. |
| `mycelium_core::prim::PrimTable::decl_hash` | fn | `crates/mycelium-core/src/prim.rs:190` | The content hash of the prim registered under kernel name `name`, if any. |
| `mycelium_core::prim::PrimTable::entries` | fn | `crates/mycelium-core/src/prim.rs:241` | Every entry as `(name, #p, decl)`, in name order — the inspectable surface for EXPLAIN over |
| `mycelium_core::prim::PrimTable::get` | fn | `crates/mycelium-core/src/prim.rs:214` | The declaration registered under kernel name `name`, if any. |
| `mycelium_core::prim::PrimTable::insert` | fn | `crates/mycelium-core/src/prim.rs:150` | Register (or replace) a prim declaration under build-time kernel name `name`, returning its |
| `mycelium_core::prim::PrimTable::intrinsic` | fn | `crates/mycelium-core/src/prim.rs:221` | The intrinsic guarantee `g_f` of the prim named `name` (RFC-0001 §4.7), if registered. |
| `mycelium_core::prim::PrimTable::names` | fn | `crates/mycelium-core/src/prim.rs:233` | The registered kernel names, sorted. |
| `mycelium_core::prim::PrimTable::prim_ref` | fn | `crates/mycelium-core/src/prim.rs:196` | A [`PrimRef`] for the prim named `name`, if registered. |
| `mycelium_core::prim::PrimTable::resolve` | fn | `crates/mycelium-core/src/prim.rs:208` | The declaration a [`PrimRef`] points at, if registered. |
| `mycelium_core::recon` | mod | `crates/mycelium-core/src/lib.rs:24` | — |
| `mycelium_core::recon::ReconInfo::bound` | fn | `crates/mycelium-core/src/recon.rs:245` | The attached `{ε, δ, strength}` bound certificate. |
| `mycelium_core::recon::ReconInfo::codebooks` | fn | `crates/mycelium-core/src/recon.rs:230` | The content-addressed codebook references. |
| `mycelium_core::recon::ReconInfo::decode` | fn | `crates/mycelium-core/src/recon.rs:240` | The decode procedure + parameters. |
| `mycelium_core::recon::ReconInfo::dim` | fn | `crates/mycelium-core/src/recon.rs:225` | Hypervector dimensionality. |
| `mycelium_core::recon::ReconInfo::mode` | fn | `crates/mycelium-core/src/recon.rs:215` | Which capability this manifest supports. |
| `mycelium_core::recon::ReconInfo::model` | fn | `crates/mycelium-core/src/recon.rs:220` | The VSA model id (matches the producing `Repr.model`). |
| `mycelium_core::recon::ReconInfo::new` | fn | `crates/mycelium-core/src/recon.rs:132` | Build a manifest, enforcing the schema invariants (RFC-0003 §6; |
| `mycelium_core::recon::ReconInfo::recipe` | fn | `crates/mycelium-core/src/recon.rs:235` | The compositional recipe, if this manifest is compositional. |
| `mycelium_core::repr` | mod | `crates/mycelium-core/src/lib.rs:25` | — |
| `mycelium_core::repr::Repr::well_formed` | fn | `crates/mycelium-core/src/repr.rs:91` | Well-formed iff all widths/dims/trits (and any `max_active`) are positive and a VSA `model` |
| `mycelium_core::repr::ScalarKind::tag` | fn | `crates/mycelium-core/src/repr.rs:30` | A stable one-byte code for content-addressing (M-103). |
| `mycelium_core::ternary` | mod | `crates/mycelium-core/src/lib.rs:26` | — |
| `mycelium_core::ternary::add` | fn | `crates/mycelium-core/src/ternary.rs:107` | Ripple-carry add over two equal-length MSB-first trit strings, fixed-width. |
| `mycelium_core::ternary::digit` | fn | `crates/mycelium-core/src/ternary.rs:22` | The signed value of a single trit. |
| `mycelium_core::ternary::int_to_trits` | fn | `crates/mycelium-core/src/ternary.rs:69` | The unique `m`-trit balanced representation of `value`, MSB-first — or `None` if `value` lies |
| `mycelium_core::ternary::max_magnitude` | fn | `crates/mycelium-core/src/ternary.rs:51` | The maximum representable magnitude in `m` trits: `(3^m − 1) / 2`. |
| `mycelium_core::ternary::mul` | fn | `crates/mycelium-core/src/ternary.rs:141` | Fixed-width multiplication. |
| `mycelium_core::ternary::neg` | fn | `crates/mycelium-core/src/ternary.rs:92` | Digit-wise negation: `value(neg t) = −value(t)` exactly (balanced ternary is sign-symmetric, §1). |
| `mycelium_core::ternary::sub` | fn | `crates/mycelium-core/src/ternary.rs:130` | Fixed-width subtraction `a − b` = `add(a, neg(b))`. |
| `mycelium_core::ternary::trits_to_int` | fn | `crates/mycelium-core/src/ternary.rs:61` | The integer denoted by an MSB-first trit string (`value(t)`, §1). |
| `mycelium_core::value` | mod | `crates/mycelium-core/src/lib.rs:27` | — |
| `mycelium_core::value::Value::meta` | fn | `crates/mycelium-core/src/value.rs:169` | The metadata. |
| `mycelium_core::value::Value::new` | fn | `crates/mycelium-core/src/value.rs:143` | Build a value, checking `repr.well_formed()` and that `payload` matches `repr`. |
| `mycelium_core::value::Value::payload` | fn | `crates/mycelium-core/src/value.rs:164` | The payload. |
| `mycelium_core::value::Value::repr` | fn | `crates/mycelium-core/src/value.rs:159` | The representation descriptor. |

## mycelium-dense

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_dense::BF16_OP_REL_EPS:` | const | `crates/mycelium-dense/src/lib.rs:39` | Two-rounding relative bound for BF16 ops: the op is computed as a native `f32` op |
| `mycelium_dense::DENSE_MIN_NORMAL:` | const | `crates/mycelium-dense/src/lib.rs:43` | Smallest positive *normal* magnitude on both the `f32` and bfloat16 grids (`2^−126` — bf16 |
| `mycelium_dense::DenseError` | enum | `crates/mycelium-dense/src/lib.rs:71` | Why a Dense operation could not be performed — always explicit, never a silent coercion (G2). |
| `mycelium_dense::DenseOp` | enum | `crates/mycelium-dense/src/lib.rs:58` | The Dense operations this surface supplies (RFC-0001 §4.1 — the Dense analogue of |
| `mycelium_dense::DenseSpace` | struct | `crates/mycelium-dense/src/lib.rs:194` | A typed Dense space: every value it constructs or operates on has exactly this `dim` and |
| `mycelium_dense::DenseSpace::add_values` | fn | `crates/mycelium-dense/src/lib.rs:364` | Elementwise `a + b` (**`Proven`**, per-element relative ε — see crate docs). |
| `mycelium_dense::DenseSpace::dot` | fn | `crates/mycelium-dense/src/lib.rs:429` | Dot product in `f64` — a *measurement* helper (no `Meta` to tag), mirroring |
| `mycelium_dense::DenseSpace::neg_value` | fn | `crates/mycelium-dense/src/lib.rs:393` | Elementwise negation (**`Exact`** — the grids are symmetric, so no element ever rounds). |
| `mycelium_dense::DenseSpace::new` | fn | `crates/mycelium-dense/src/lib.rs:204` | A Dense space of `dim`-vectors over `dtype`. |
| `mycelium_dense::DenseSpace::op_guarantee` | fn | `crates/mycelium-dense/src/lib.rs:223` | The honest intrinsic guarantee per op: `neg` never rounds (`Exact`); `add`/`sub`/`scale` |
| `mycelium_dense::DenseSpace::op_rel_eps` | fn | `crates/mycelium-dense/src/lib.rs:232` | The per-element relative ε this space's rounding ops carry. |
| `mycelium_dense::DenseSpace::repr` | fn | `crates/mycelium-dense/src/lib.rs:213` | The `Repr` of this space's values. |
| `mycelium_dense::DenseSpace::scale_value` | fn | `crates/mycelium-dense/src/lib.rs:413` | Scalar multiplication `c · a` (**`Proven`**). |
| `mycelium_dense::DenseSpace::similarity` | fn | `crates/mycelium-dense/src/lib.rs:437` | Cosine similarity in `[-1, 1]` (`0` if either operand has zero norm) — a measurement |
| `mycelium_dense::DenseSpace::sub_values` | fn | `crates/mycelium-dense/src/lib.rs:369` | Elementwise `a − b` (**`Proven`**, same bound as `add`). |
| `mycelium_dense::DenseSpace::value` | fn | `crates/mycelium-dense/src/lib.rs:250` | Construct an **`Exact`** Dense value, checking every element is finite and exactly on the |
| `mycelium_dense::F32_OP_REL_EPS:` | const | `crates/mycelium-dense/src/lib.rs:34` | Single-rounding relative bound for native `f32` ops: the unit roundoff `u = β^(1−p)/2 = 2^−24` |

## mycelium-diag

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_diag::Code` | enum | `crates/mycelium-diag/src/lib.rs:144` | A stable diagnostic code / error class (RFC-0013 §4.2). |
| `mycelium_diag::Code::as_str` | fn | `crates/mycelium-diag/src/lib.rs:127` | The canonical name used in human/machine output. |
| `mycelium_diag::Diag` | struct | `crates/mycelium-diag/src/lib.rs:216` | A structured diagnostic record (RFC-0013 §4.1): a content-addressable value over an |
| `mycelium_diag::Diag::at` | fn | `crates/mycelium-diag/src/lib.rs:274` | Attach a source locus (explicit; absence stays `None` — never a fabricated zero, G2). |
| `mycelium_diag::Diag::code` | fn | `crates/mycelium-diag/src/lib.rs:303` | The diagnostic code / error class. |
| `mycelium_diag::Diag::content_hash` | fn | `crates/mycelium-diag/src/lib.rs:318` | The **content address** of this diagnostic (RFC-0013 §4.3; ADR-003) — a deterministic BLAKE3 |
| `mycelium_diag::Diag::error` | fn | `crates/mycelium-diag/src/lib.rs:236` | Build an `Error`-severity diagnostic with the given code (total builder). |
| `mycelium_diag::Diag::from_json` | fn | `crates/mycelium-diag/src/lib.rs:451` | Recover a `Diag` from its machine JSON projection (I3). |
| `mycelium_diag::Diag::human` | fn | `crates/mycelium-diag/src/lib.rs:375` | The **human projection** (G11 / RFC-0013 I3): a human-readable string. |
| `mycelium_diag::Diag::info` | fn | `crates/mycelium-diag/src/lib.rs:248` | Build an `Info`-severity diagnostic with the given code (total builder). |
| `mycelium_diag::Diag::machine` | fn | `crates/mycelium-diag/src/lib.rs:429` | The **machine projection** (G11 / RFC-0013 I3): a lossless JSON record with the content `id` |
| `mycelium_diag::Diag::message` | fn | `crates/mycelium-diag/src/lib.rs:267` | Set the human-readable message (value-semantic builder). |
| `mycelium_diag::Diag::note` | fn | `crates/mycelium-diag/src/lib.rs:281` | Attach a note (EXPLAIN payload). |
| `mycelium_diag::Diag::severity` | fn | `crates/mycelium-diag/src/lib.rs:297` | The typed severity (a `Warn` never silently becomes a pass — I1). |
| `mycelium_diag::Diag::trace` | fn | `crates/mycelium-diag/src/lib.rs:288` | Replace the trace (value-semantic builder). |
| `mycelium_diag::Diag::warn` | fn | `crates/mycelium-diag/src/lib.rs:242` | Build a `Warn`-severity diagnostic with the given code (total builder). |
| `mycelium_diag::Diag::with_severity` | fn | `crates/mycelium-diag/src/lib.rs:254` | The common total builder behind [`Self::error`]/[`Self::warn`]/[`Self::info`]. |
| `mycelium_diag::Locus` | struct | `crates/mycelium-diag/src/lib.rs:174` | A source locus — *where* a diagnostic points (RFC-0013 §4.2). |
| `mycelium_diag::Severity::ALL:` | const | `crates/mycelium-diag/src/lib.rs:118` | All severities, ordered weakest-to-strongest (`Debug < Info < Warn < Error`). |
| `mycelium_diag::Trace` | struct | `crates/mycelium-diag/src/lib.rs:188` | An ordered diagnostic trace — the chain of frames/notes that led to the failure (RFC-0013 §4.3). |
| `mycelium_diag::Trace::empty` | fn | `crates/mycelium-diag/src/lib.rs:196` | The empty trace (explicit absence — not a fabricated frame). |
| `mycelium_diag::Trace::with_frame` | fn | `crates/mycelium-diag/src/lib.rs:202` | Push a frame, returning the extended trace (value-semantic). |

## mycelium-doc

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_doc::BuildInput` | struct | `crates/mycelium-doc/src/build.rs:25` | What to ingest. |
| `mycelium_doc::CHECK_NAMES:` | const | `crates/mycelium-doc/src/doc_lint.rs:16` | The eight §4.1 checks, by canonical name — the single source of truth (`mycelium-lint` re-exports |
| `mycelium_doc::CheckOutcome` | struct | `crates/mycelium-doc/src/doc_lint.rs:76` | The outcome of one check. |
| `mycelium_doc::CheckStatus` | enum | `crates/mycelium-doc/src/doc_lint.rs:65` | Whether a check is fully active, partly dormant (named sub-aspects await machinery), or dormant. |
| `mycelium_doc::DocLintReport` | struct | `crates/mycelium-doc/src/doc_lint.rs:89` | The full §4.1 lint report. |
| `mycelium_doc::DocModel` | struct | `crates/mycelium-doc/src/ir.rs:320` | The whole projected corpus: top-level documents plus the navigable index over every node. |
| `mycelium_doc::Finding` | struct | `crates/mycelium-doc/src/doc_lint.rs:52` | One finding from a check. |
| `mycelium_doc::Level` | enum | `crates/mycelium-doc/src/ir.rs:22` | Graded depth (RFC-0013's `minimal / medium / detailed` levels, reused for docs — §4.1 progressive |
| `mycelium_doc::Node` | struct | `crates/mycelium-doc/src/ir.rs:224` | One node of the content-addressed doc-IR. |
| `mycelium_doc::Payload` | enum | `crates/mycelium-doc/src/ir.rs:147` | The kind-specific content of a node. |
| `mycelium_doc::Provenance` | struct | `crates/mycelium-doc/src/ir.rs:92` | Where a node was projected from (append-only provenance, §9 — "generated from"). |
| `mycelium_doc::Severity` | enum | `crates/mycelium-doc/src/doc_lint.rs:29` | Finding severity (mirrors the lattice's never-silent posture: an error gates, a warning advises). |
| `mycelium_doc::SourceKind` | enum | `crates/mycelium-doc/src/ir.rs:55` | Which corpus family a [`Payload::Document`] was projected from (drives ordering + the template's |
| `mycelium_doc::apiref` | mod | `crates/mycelium-doc/src/lib.rs:19` | — |
| `mycelium_doc::apiref::project_nodule` | fn | `crates/mycelium-doc/src/apiref.rs:21` | Project a `.myc` source into a [`Payload::Document`] (`source_kind: api`) of api-item nodes. |
| `mycelium_doc::apiref::project_schema` | fn | `crates/mycelium-doc/src/apiref.rs:107` | Project a JSON-schema file into a [`Payload::Document`] of api-item nodes (one per top-level |
| `mycelium_doc::build` | mod | `crates/mycelium-doc/src/lib.rs:20` | — |
| `mycelium_doc::build` | fn | `crates/mycelium-doc/src/build.rs:59` | Build the resolved doc model from the input. |
| `mycelium_doc::build::BuildInput::conventional` | fn | `crates/mycelium-doc/src/build.rs:44` | The conventional layout rooted at `repo_root`: `docs/`, `docs/spec/schemas/`, `examples/`, |
| `mycelium_doc::build::EPUB_DEFERRAL:` | const | `crates/mycelium-doc/src/build.rs:19` | EPUB is an honest deferral (spec §8 / §4.1 "never a half-build"). |
| `mycelium_doc::corpus` | mod | `crates/mycelium-doc/src/lib.rs:21` | — |
| `mycelium_doc::corpus::AnchorAlloc` | struct | `crates/mycelium-doc/src/corpus.rs:14` | Allocates globally-unique, stable anchor slugs (so deep links never collide — §4.1 navigability). |
| `mycelium_doc::corpus::AnchorAlloc::alloc` | fn | `crates/mycelium-doc/src/corpus.rs:26` | Slugify `base` (optionally namespaced under `ns`) and make it unique by `-N` suffixing. |
| `mycelium_doc::corpus::AnchorAlloc::new` | fn | `crates/mycelium-doc/src/corpus.rs:21` | A fresh allocator. |
| `mycelium_doc::corpus::extract_links` | fn | `crates/mycelium-doc/src/corpus.rs:170` | Extract inline `text` link targets from a paragraph (the cross-reference seed). |
| `mycelium_doc::corpus::ingest` | fn | `crates/mycelium-doc/src/corpus.rs:232` | Project a markdown source into a [`Payload::Document`] node. |
| `mycelium_doc::corpus::slugify` | fn | `crates/mycelium-doc/src/corpus.rs:52` | A GitHub-style anchor slug: lowercase, non-alphanumerics → `-`, collapsed, trimmed. |
| `mycelium_doc::doc_lint` | mod | `crates/mycelium-doc/src/lib.rs:22` | — |
| `mycelium_doc::doc_lint::DocLintReport::errors` | fn | `crates/mycelium-doc/src/doc_lint.rs:105` | Every error-severity finding, flattened. |
| `mycelium_doc::doc_lint::DocLintReport::has_errors` | fn | `crates/mycelium-doc/src/doc_lint.rs:97` | Whether any finding is error-severity (the gate condition). |
| `mycelium_doc::doc_lint::Severity::as_str` | fn | `crates/mycelium-doc/src/doc_lint.rs:41` | The canonical label. |
| `mycelium_doc::emit` | mod | `crates/mycelium-doc/src/lib.rs:23` | — |
| `mycelium_doc::emit::Artifacts` | struct | `crates/mycelium-doc/src/emit/mod.rs:15` | A set of generated artifacts: repo/out-relative path → file contents. |
| `mycelium_doc::emit::Artifacts::new` | fn | `crates/mycelium-doc/src/emit/mod.rs:23` | A fresh, empty artifact set. |
| `mycelium_doc::emit::Artifacts::put` | fn | `crates/mycelium-doc/src/emit/mod.rs:28` | Add (or overwrite) one artifact. |
| `mycelium_doc::emit::Artifacts::write_to` | fn | `crates/mycelium-doc/src/emit/mod.rs:37` | Write every artifact under `out_dir`, creating parent directories. |
| `mycelium_doc::emit::html` | mod | `crates/mycelium-doc/src/emit/mod.rs:7` | — |
| `mycelium_doc::emit::html::render` | fn | `crates/mycelium-doc/src/emit/html.rs:39` | Render the whole model to an HTML site: `index.html` plus one `pages/<anchor>.html` per document. |
| `mycelium_doc::emit::html::render_concat` | fn | `crates/mycelium-doc/src/emit/html.rs:50` | The concatenation of every page (for the parity/legibility lints, which scan the rendered output). |
| `mycelium_doc::emit::html::template_hash` | fn | `crates/mycelium-doc/src/emit/html.rs:30` | The pinned template content hash (provenance, §6) — the address of the shared template/style. |
| `mycelium_doc::emit::html_escape` | fn | `crates/mycelium-doc/src/emit/mod.rs:55` | Escape text for HTML body content / attribute values. |
| `mycelium_doc::emit::json` | mod | `crates/mycelium-doc/src/emit/mod.rs:8` | — |
| `mycelium_doc::emit::json::render` | fn | `crates/mycelium-doc/src/emit/json.rs:27` | Render the machine artifacts: the full model JSON + the JSONL search index. |
| `mycelium_doc::emit::json::render_model_json` | fn | `crates/mycelium-doc/src/emit/json.rs:36` | The whole model, serialized (pretty) — every node id is present (the parity hook). |
| `mycelium_doc::emit::json::render_search_index` | fn | `crates/mycelium-doc/src/emit/json.rs:42` | One JSON record per node, newline-delimited (a streamable search/tooling index). |
| `mycelium_doc::emit::typst` | mod | `crates/mycelium-doc/src/emit/mod.rs:9` | — |
| `mycelium_doc::emit::typst::render` | fn | `crates/mycelium-doc/src/emit/typst.rs:11` | Render the whole model to one Typst document source. |
| `mycelium_doc::emit_all` | fn | `crates/mycelium-doc/src/build.rs:141` | Emit every artifact (HTML site · Typst source · machine JSON · the EPUB deferral note). |
| `mycelium_doc::hash` | mod | `crates/mycelium-doc/src/lib.rs:24` | — |
| `mycelium_doc::hash::DocHasher` | struct | `crates/mycelium-doc/src/hash.rs:14` | A canonical, injective content hasher: tagged, length-prefixed writes feed a single BLAKE3 state. |
| `mycelium_doc::hash::DocHasher::child` | fn | `crates/mycelium-doc/src/hash.rs:64` | Absorb an already-computed child address (a content hash), length-prefixed. |
| `mycelium_doc::hash::DocHasher::finish` | fn | `crates/mycelium-doc/src/hash.rs:70` | Finalize into the kernel's `blake3:<hex>` content-address shape. |
| `mycelium_doc::hash::DocHasher::new` | fn | `crates/mycelium-doc/src/hash.rs:27` | A fresh hasher. |
| `mycelium_doc::hash::DocHasher::opt_str` | fn | `crates/mycelium-doc/src/hash.rs:53` | Absorb an optional string distinctly from the empty string (tag 0 = none, 1 = some). |
| `mycelium_doc::hash::DocHasher::str` | fn | `crates/mycelium-doc/src/hash.rs:46` | Absorb a length-prefixed string (the prefix makes the framing injective). |
| `mycelium_doc::hash::DocHasher::tag` | fn | `crates/mycelium-doc/src/hash.rs:34` | Absorb a one-byte domain/kind tag. |
| `mycelium_doc::hash::DocHasher::u64` | fn | `crates/mycelium-doc/src/hash.rs:40` | Absorb a `u64` (little-endian, fixed width — framing is injective). |
| `mycelium_doc::ir` | mod | `crates/mycelium-doc/src/lib.rs:25` | — |
| `mycelium_doc::ir::DocModel::all_nodes` | fn | `crates/mycelium-doc/src/ir.rs:345` | Every node across every document, depth-first (the order a reader meets them). |
| `mycelium_doc::ir::DocModel::id_set` | fn | `crates/mycelium-doc/src/ir.rs:355` | The set of content addresses present in the model (used by the dual-projection-parity lint). |
| `mycelium_doc::ir::Level::as_str` | fn | `crates/mycelium-doc/src/ir.rs:34` | The canonical label. |
| `mycelium_doc::ir::Node::new` | fn | `crates/mycelium-doc/src/ir.rs:246` | Build a node, computing its content address from its content + children (ADR-003). |
| `mycelium_doc::ir::Node::walk` | fn | `crates/mycelium-doc/src/ir.rs:310` | Depth-first pre-order visit of this node and its descendants. |
| `mycelium_doc::ir::Payload::kind_str` | fn | `crates/mycelium-doc/src/ir.rs:208` | The canonical kind label (for diagnostics / the machine projection). |
| `mycelium_doc::ir::XrefResolution` | enum | `crates/mycelium-doc/src/ir.rs:102` | How a cross-reference resolved against the model (the §4.1 `no-dead-xref` verdict). |
| `mycelium_doc::ir::XrefTarget` | struct | `crates/mycelium-doc/src/ir.rs:124` | The resolved-or-not target of a cross-reference. |
| `mycelium_doc::lint` | fn | `crates/mycelium-doc/src/doc_lint.rs:116` | Run all eight §4.1 checks over the model. |

## mycelium-fmt

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_fmt::FmtError::exit_code` | fn | `crates/mycelium-fmt/src/lib.rs:116` | The CLI exit code for this refusal (contract §5). |
| `mycelium_fmt::Formatted` | struct | `crates/mycelium-fmt/src/lib.rs:74` | A successful format result. |
| `mycelium_fmt::MYCFMT_VERSION:` | const | `crates/mycelium-fmt/src/lib.rs:66` | The formatter spelling/version this build implements. |
| `mycelium_fmt::format_source` | fn | `crates/mycelium-fmt/src/lib.rs:161` | Format `src` into its canonical form. |

## mycelium-interp

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_interp::Budgets` | struct | `crates/mycelium-interp/src/budget.rs:146` | The **budget ledger** — one enforcement mechanism over the separate named budgets (RFC-0014 §8 |
| `mycelium_interp::CancelToken` | struct | `crates/mycelium-interp/src/supervise.rs:33` | A **cooperative** cancellation token (RFC-0008 §4.7; structured-concurrency cancellation, RT7). |
| `mycelium_interp::Cancelled` | struct | `crates/mycelium-interp/src/supervise.rs:75` | A task observed its [`CancelToken`] cancelled — an **explicit, additive** outcome (RFC-0014 I1), |
| `mycelium_interp::EffectBudget` | enum | `crates/mycelium-interp/src/budget.rs:63` | A per-kind **budget** (RFC-0014 §4.5 I4) — distinct vocabulary (`max_attempts` / `max_depth` / a |
| `mycelium_interp::EffectBudgetExhausted` | struct | `crates/mycelium-interp/src/budget.rs:114` | Exceeding a budget — an **explicit, graceful** structured error (RFC-0014 §4.5 I4), never a hang / |
| `mycelium_interp::EffectKind` | enum | `crates/mycelium-interp/src/budget.rs:27` | A closed kernel of effect **kinds** (RFC-0014 §4.5 I3) plus user-declared names. |
| `mycelium_interp::Escalation` | enum | `crates/mycelium-interp/src/supervise.rs:135` | A supervisor escalated: a restart cascade hit a bound and the supervisor itself fails (its own |
| `mycelium_interp::EvalError` | enum | `crates/mycelium-interp/src/lib.rs:135` | Why evaluation could not proceed (always explicit — the interpreter is never silent; SC-3/G2). |
| `mycelium_interp::IdentitySwapEngine` | struct | `crates/mycelium-interp/src/swap.rs:27` | The trivial swap engine: a swap whose `target` equals the source `Repr` is the identity — exactly |
| `mycelium_interp::Interpreter` | struct | `crates/mycelium-interp/src/lib.rs:300` | The reference interpreter: a primitive registry + a swap engine. |
| `mycelium_interp::Interpreter::eval` | fn | `crates/mycelium-interp/src/lib.rs:508` | Evaluate `node` to a **representation** value by iterating step to a normal |
| `mycelium_interp::Interpreter::eval_core` | fn | `crates/mycelium-interp/src/lib.rs:520` | Evaluate `node` to a [`CoreValue`] — a representation value **or** a data value (the r3 data |
| `mycelium_interp::Interpreter::new` | fn | `crates/mycelium-interp/src/lib.rs:320` | Build an interpreter with a custom prim registry and swap engine (e.g. |
| `mycelium_interp::Interpreter::prim_names` | fn | `crates/mycelium-interp/src/lib.rs:337` | The registered primitive names (for tooling/EXPLAIN). |
| `mycelium_interp::Interpreter::step` | fn | `crates/mycelium-interp/src/lib.rs:345` | Perform exactly one small-step reduction on `node` (the `⟶` relation above). |
| `mycelium_interp::Interpreter::with_fuel` | fn | `crates/mycelium-interp/src/lib.rs:330` | Override the step budget. |
| `mycelium_interp::PrimRegistry` | struct | `crates/mycelium-interp/src/prims.rs:53` | The name→implementation table the interpreter dispatches `Op` nodes through. |
| `mycelium_interp::RestartIntensity` | struct | `crates/mycelium-interp/src/supervise.rs:125` | **Max-restart-intensity** for `reclaim` supervision (RFC-0008 §4.7; Erlang/OTP, Research Record 05 |
| `mycelium_interp::Step` | enum | `crates/mycelium-interp/src/lib.rs:125` | The result of one small-step attempt on a node. |
| `mycelium_interp::Supervisor` | struct | `crates/mycelium-interp/src/supervise.rs:179` | A `reclaim` **supervisor** (RFC-0008 §4.7; RT4/RT7): it restarts a failed child under a *bounded* |
| `mycelium_interp::SwapEngine` | trait | `crates/mycelium-interp/src/swap.rs:16` | Evaluates a `Swap` node. |
| `mycelium_interp::TaskOutcome` | enum | `crates/mycelium-interp/src/supervise.rs:94` | The **explicit, additive result of running a task** (RFC-0014 I1 lifted across the task boundary, |
| `mycelium_interp::budget` | mod | `crates/mycelium-interp/src/lib.rs:109` | — |
| `mycelium_interp::budget::Budgets::consume` | fn | `crates/mycelium-interp/src/budget.rs:183` | Consume `amount` of `kind`'s budget. |
| `mycelium_interp::budget::Budgets::new` | fn | `crates/mycelium-interp/src/budget.rs:153` | An empty ledger — no effect may run until a budget is declared (I5). |
| `mycelium_interp::budget::Budgets::remaining` | fn | `crates/mycelium-interp/src/budget.rs:173` | The remaining budget for `kind` (`None` if none was declared). |
| `mycelium_interp::budget::Budgets::set` | fn | `crates/mycelium-interp/src/budget.rs:167` | Declare (or reset) a budget for its effect kind. |
| `mycelium_interp::budget::Budgets::with` | fn | `crates/mycelium-interp/src/budget.rs:161` | Builder: declare a budget. |
| `mycelium_interp::budget::EffectBudget::amount` | fn | `crates/mycelium-interp/src/budget.rs:93` | The budget's scalar amount. |
| `mycelium_interp::budget::EffectBudget::kind` | fn | `crates/mycelium-interp/src/budget.rs:81` | The effect kind this budget bounds. |
| `mycelium_interp::prims` | mod | `crates/mycelium-interp/src/lib.rs:110` | — |
| `mycelium_interp::prims::PrimFn` | type | `crates/mycelium-interp/src/prims.rs:48` | A primitive implementation: a pure function from argument values to a result value (or an error). |
| `mycelium_interp::prims::PrimRegistry::empty` | fn | `crates/mycelium-interp/src/prims.rs:60` | An empty registry. |
| `mycelium_interp::prims::PrimRegistry::get` | fn | `crates/mycelium-interp/src/prims.rs:91` | Look up a primitive by name. |
| `mycelium_interp::prims::PrimRegistry::names` | fn | `crates/mycelium-interp/src/prims.rs:97` | The registered primitive names (sorted). |
| `mycelium_interp::prims::PrimRegistry::register` | fn | `crates/mycelium-interp/src/prims.rs:85` | Register (or replace) a primitive. |
| `mycelium_interp::prims::PrimRegistry::with_builtins` | fn | `crates/mycelium-interp/src/prims.rs:70` | The default registry: the exact built-ins — elementwise logical (`core.id`, |
| `mycelium_interp::supervise` | mod | `crates/mycelium-interp/src/lib.rs:111` | — |
| `mycelium_interp::supervise::CancelToken::cancel` | fn | `crates/mycelium-interp/src/supervise.rs:48` | Request cancellation. |
| `mycelium_interp::supervise::CancelToken::check` | fn | `crates/mycelium-interp/src/supervise.rs:63` | Observe the token at a checkpoint: an explicit [`Cancelled`] if cancellation was requested, else |
| `mycelium_interp::supervise::CancelToken::is_cancelled` | fn | `crates/mycelium-interp/src/supervise.rs:54` | Whether cancellation has been requested. |
| `mycelium_interp::supervise::Supervisor::new` | fn | `crates/mycelium-interp/src/supervise.rs:40` | A fresh, un-cancelled token. |
| `mycelium_interp::supervise::Supervisor::now` | fn | `crates/mycelium-interp/src/supervise.rs:202` | The current logical tick. |
| `mycelium_interp::supervise::Supervisor::record_restart` | fn | `crates/mycelium-interp/src/supervise.rs:230` | Record a restart at the current logical tick. |
| `mycelium_interp::supervise::Supervisor::restarts_remaining` | fn | `crates/mycelium-interp/src/supervise.rs:215` | The total restart budget remaining (the `cascade` cap). |
| `mycelium_interp::supervise::Supervisor::tick` | fn | `crates/mycelium-interp/src/supervise.rs:208` | Advance the logical clock by one tick and return the new value. |
| `mycelium_interp::swap` | mod | `crates/mycelium-interp/src/lib.rs:112` | — |

## mycelium-l1

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_l1::AmbientError` | enum | `crates/mycelium-l1/src/ambient.rs:50` | A never-silent refusal from the resolution pass (§4.3/§4.4) — always explicit, never a guess. |
| `mycelium_l1::CheckError` | struct | `crates/mycelium-l1/src/checkty.rs:115` | An explicit check failure (never a silent pass or a guess — S5/G2). |
| `mycelium_l1::ElabError` | enum | `crates/mycelium-l1/src/elab.rs:46` | Why a definition could not be elaborated to L0 — always explicit, never a partial artifact |
| `mycelium_l1::Env` | struct | `crates/mycelium-l1/src/checkty.rs:333` | The checked program environment: registry + function table. |
| `mycelium_l1::Evaluator` | struct | `crates/mycelium-l1/src/eval.rs:260` | The L1 evaluator over a checked [`Env`]. |
| `mycelium_l1::InstanceSelection` | struct | `crates/mycelium-l1/src/mono.rs:63` | A reified **instance selection** (RFC-0019 §4.4; house rule #2 — no black boxes). |
| `mycelium_l1::L1Error` | enum | `crates/mycelium-l1/src/eval.rs:93` | Why L1 evaluation could not produce a value — always explicit (S5/G2). |
| `mycelium_l1::L1Value` | enum | `crates/mycelium-l1/src/eval.rs:42` | An L1 runtime value: an L0 representation value, or a constructed datum. |
| `mycelium_l1::MonoSelections` | struct | `crates/mycelium-l1/src/mono.rs:83` | The **EXPLAIN record** of a monomorphization (M-673): every trait-method dispatch mono resolved, |
| `mycelium_l1::Nodule` | struct | `crates/mycelium-l1/src/ast.rs:74` | A whole program: a `nodule` header and its items. |
| `mycelium_l1::NoduleHeader` | struct | `crates/mycelium-l1/src/nodule.rs:25` | A recognised nodule header marker (DN-06 §6). |
| `mycelium_l1::NoduleHeaderError` | struct | `crates/mycelium-l1/src/nodule.rs:51` | An ill-formed nodule header marker — never-silent (G2): the author wrote `// nodule:` but the |
| `mycelium_l1::ParseError` | struct | `crates/mycelium-l1/src/error.rs:9` | A parse/lex failure at a source position. |
| `mycelium_l1::Phylum` | struct | `crates/mycelium-l1/src/ast.rs:16` | A **phylum** — the library-scale static grouping above `nodule` (DN-06; RFC-0006 §4.3; M-662). |
| `mycelium_l1::PhylumEnv` | struct | `crates/mycelium-l1/src/checkty.rs:503` | The checked environments of a whole **phylum** (M-662): one [`Env`] per nodule, paired with the |
| `mycelium_l1::Resolved` | struct | `crates/mycelium-l1/src/ambient.rs:130` | The resolved twin plus its provenance trace. |
| `mycelium_l1::Totality` | enum | `crates/mycelium-l1/src/totality.rs:31` | The divergence bit (RFC-0007 §4.5). |
| `mycelium_l1::Ty` | enum | `crates/mycelium-l1/src/checkty.rs:56` | A checked type. |
| `mycelium_l1::UsePath` | struct | `crates/mycelium-l1/src/ast.rs:63` | A `use` import target (`use a.b.Item` or the glob `use a.b.*`; M-662; RFC-0006 §4.3). |
| `mycelium_l1::Vis` | enum | `crates/mycelium-l1/src/ast.rs:42` | **Cross-nodule visibility** of a top-level item (M-662; RFC-0006 §4.3). |
| `mycelium_l1::ambient` | mod | `crates/mycelium-l1/src/lib.rs:37` | — |
| `mycelium_l1::ambient::ResolutionNote` | struct | `crates/mycelium-l1/src/ambient.rs:119` | A record of one ambient fill, for EXPLAIN / "where did this paradigm come from?" (§4.3). |
| `mycelium_l1::ast` | mod | `crates/mycelium-l1/src/lib.rs:38` | — |
| `mycelium_l1::ast::AmbientParams` | enum | `crates/mycelium-l1/src/ast.rs:117` | The written params of a **paradigm-less repr** `{ … }` (RFC-0012 §4.2): the size/shape is still |
| `mycelium_l1::ast::Arm` | struct | `crates/mycelium-l1/src/ast.rs:546` | A `match` arm. |
| `mycelium_l1::ast::BaseType` | enum | `crates/mycelium-l1/src/ast.rs:325` | A base (un-annotated) type. |
| `mycelium_l1::ast::Ctor` | struct | `crates/mycelium-l1/src/ast.rs:169` | One constructor of a [`TypeDecl`]. |
| `mycelium_l1::ast::Expr` | enum | `crates/mycelium-l1/src/ast.rs:435` | An expression. |
| `mycelium_l1::ast::FnDecl` | struct | `crates/mycelium-l1/src/ast.rs:266` | A function definition. |
| `mycelium_l1::ast::FnSig` | struct | `crates/mycelium-l1/src/ast.rs:231` | A function signature (shared by trait requirements and `fn` definitions). |
| `mycelium_l1::ast::FnSig::param_names` | fn | `crates/mycelium-l1/src/ast.rs:258` | The **names** of this signature's type parameters (dropping any bounds) — the form the |
| `mycelium_l1::ast::Hypha` | struct | `crates/mycelium-l1/src/ast.rs:560` | One `hypha <expr>` spawn inside a [`Expr::Colony`] block — a single concurrent execution unit |
| `mycelium_l1::ast::ImplDecl` | struct | `crates/mycelium-l1/src/ast.rs:196` | A trait-instance declaration `impl Trait<args> for T { fn … }` (RFC-0019 §4.1; RFC-0007 §12.1). |
| `mycelium_l1::ast::Item` | enum | `crates/mycelium-l1/src/ast.rs:135` | A top-level item. |
| `mycelium_l1::ast::Literal::binary` | fn | `crates/mycelium-l1/src/ast.rs:609` | A binary literal from its verbatim digit/`_` string (the `…` of `0b…`). |
| `mycelium_l1::ast::Literal::ternary` | fn | `crates/mycelium-l1/src/ast.rs:617` | A ternary literal from its verbatim `+0-` string, MSB-first (the inner text of `<…>`). |
| `mycelium_l1::ast::Paradigm` | enum | `crates/mycelium-l1/src/ast.rs:91` | A representation **paradigm** tag (RFC-0001 §4.2): the granularity of the RFC-0012 ambient. |
| `mycelium_l1::ast::Param` | struct | `crates/mycelium-l1/src/ast.rs:282` | A value parameter `name: type`. |
| `mycelium_l1::ast::Path` | struct | `crates/mycelium-l1/src/ast.rs:6` | A dotted path (`signals.demo`, `core.binary`); also a bare name. |
| `mycelium_l1::ast::Pattern` | enum | `crates/mycelium-l1/src/ast.rs:567` | A pattern. |
| `mycelium_l1::ast::Phylum::of_one` | fn | `crates/mycelium-l1/src/ast.rs:29` | A **phylum-of-one** wrapping a single bare nodule (no `phylum` header). |
| `mycelium_l1::ast::Scalar` | enum | `crates/mycelium-l1/src/ast.rs:370` | A scalar element kind. |
| `mycelium_l1::ast::Sparsity` | enum | `crates/mycelium-l1/src/ast.rs:361` | Declared sparsity of a VSA type. |
| `mycelium_l1::ast::Strength` | enum | `crates/mycelium-l1/src/ast.rs:383` | A guarantee-lattice strength. |
| `mycelium_l1::ast::Strength::meet` | fn | `crates/mycelium-l1/src/ast.rs:415` | The **meet** `g₁ ∧ g₂` — the *weaker* (less trusted) of the two grades (RFC-0018 §4.1: the |
| `mycelium_l1::ast::Strength::rank` | fn | `crates/mycelium-l1/src/ast.rs:400` | The **trust rank** on the integrity lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` |
| `mycelium_l1::ast::Strength::satisfies` | fn | `crates/mycelium-l1/src/ast.rs:428` | `self ⊒ demand` — is `self` **at least as trusted** as `demand`? The honesty rule as a |
| `mycelium_l1::ast::TraitDecl` | struct | `crates/mycelium-l1/src/ast.rs:180` | `trait Name<params> { fn … }` (LR-2; conventional term). |
| `mycelium_l1::ast::TraitRef` | struct | `crates/mycelium-l1/src/ast.rs:211` | A reference to a trait in a bound position — `Cmp` or `Cmp<Binary{8}>` (RFC-0019 §4.1 `bound`). |
| `mycelium_l1::ast::TypeDecl` | struct | `crates/mycelium-l1/src/ast.rs:155` | `type Name<params> = Ctor \| Ctor(field, …) \| …` (LR-1). |
| `mycelium_l1::ast::TypeParam` | struct | `crates/mycelium-l1/src/ast.rs:222` | A (possibly **bounded**) type parameter on a **function** — `T` or `T: Cmp + Ord<T>` (RFC-0019 |
| `mycelium_l1::ast::TypeRef` | struct | `crates/mycelium-l1/src/ast.rs:291` | A type with an optional guarantee-strength index (`T @ Exact`; LR-6). |
| `mycelium_l1::ast::TypeRef::unguaranteed` | fn | `crates/mycelium-l1/src/ast.rs:304` | A type with **no** written guarantee index — the surface `T` form (the index is then |
| `mycelium_l1::ast::TypeRef::with_guarantee` | fn | `crates/mycelium-l1/src/ast.rs:315` | A type carrying an explicit guarantee-strength index — the surface `T @ g` form. |
| `mycelium_l1::ast::Vis::is_pub` | fn | `crates/mycelium-l1/src/ast.rs:53` | Is this item exported to other nodules of the phylum (`pub`)? |
| `mycelium_l1::check_and_resolve` | fn | `crates/mycelium-l1/src/checkty.rs:1010` | Like [`check_nodule`], but also returns the **fully-resolved longhand twin** of the program |
| `mycelium_l1::check_nodule` | fn | `crates/mycelium-l1/src/checkty.rs:595` | Check a whole nodule: build the registry (prelude + declarations), then type every function |
| `mycelium_l1::check_nodule_matured` | fn | `crates/mycelium-l1/src/checkty.rs:951` | Like [`check_nodule`] but with an explicit `matured_scope` flag (RFC-0017 §4.2): when `true`, |
| `mycelium_l1::check_phylum` | fn | `crates/mycelium-l1/src/checkty.rs:613` | Check a whole **phylum** (M-662; RFC-0006 §4.3): build the phylum-wide `pub` **export table** and |
| `mycelium_l1::check_phylum_matured` | fn | `crates/mycelium-l1/src/checkty.rs:623` | Like [`check_phylum`] but with the explicit `matured_scope` gate applied to **every** nodule |
| `mycelium_l1::checkty` | mod | `crates/mycelium-l1/src/lib.rs:39` | — |
| `mycelium_l1::checkty::CheckError::at` | fn | `crates/mycelium-l1/src/checkty.rs:135` | Public, ergonomic constructor: a check failure at `site` with `message`. |
| `mycelium_l1::checkty::CtorInfo` | struct | `crates/mycelium-l1/src/checkty.rs:170` | One constructor of a registered data type. |
| `mycelium_l1::checkty::DataInfo` | struct | `crates/mycelium-l1/src/checkty.rs:182` | A registered data type. |
| `mycelium_l1::checkty::Env::ctor` | fn | `crates/mycelium-l1/src/checkty.rs:364` | Find the data type owning constructor `ctor`, with its index — `None` if no type has it. |
| `mycelium_l1::checkty::Env::fn_decl` | fn | `crates/mycelium-l1/src/checkty.rs:380` | The function declaration named `name`, if any. |
| `mycelium_l1::checkty::Env::fn_totality` | fn | `crates/mycelium-l1/src/checkty.rs:388` | The totality verdict for function `name`, if it has been classified. |
| `mycelium_l1::checkty::Env::instance` | fn | `crates/mycelium-l1/src/checkty.rs:402` | The registered instance for `(trait_name, head)`, if any (RFC-0019 §4.5). |
| `mycelium_l1::checkty::Env::trait_info` | fn | `crates/mycelium-l1/src/checkty.rs:395` | The registered trait named `name`, if any (RFC-0019 §4.2). |
| `mycelium_l1::checkty::Env::type_info` | fn | `crates/mycelium-l1/src/checkty.rs:373` | The registered data type named `name`, if any. |
| `mycelium_l1::checkty::InstanceInfo` | struct | `crates/mycelium-l1/src/checkty.rs:210` | A registered **instance** `impl Trait<args> for T` (RFC-0019 §4.5). |
| `mycelium_l1::checkty::MAX_CHECK_DEPTH:` | const | `crates/mycelium-l1/src/checkty.rs:40` | The checker's **explicit expression-nesting budget** (the "banked guard 4" discipline; A4-02). |
| `mycelium_l1::checkty::PhylumEnv::nodule` | fn | `crates/mycelium-l1/src/checkty.rs:521` | The checked [`Env`] of the nodule whose path equals `path`, if present. |
| `mycelium_l1::checkty::PhylumEnv::single` | fn | `crates/mycelium-l1/src/checkty.rs:512` | The single nodule's [`Env`] when this is a phylum-of-one, else `None`. |
| `mycelium_l1::checkty::TraitInfo` | struct | `crates/mycelium-l1/src/checkty.rs:197` | A registered **trait** (RFC-0019 §4.2; LR-2). |
| `mycelium_l1::checkty::prim_kernel_name` | fn | `crates/mycelium-l1/src/checkty.rs:3502` | The surface→kernel prim-name mapping (the `Op` node's `prim` — RFC-0007 §4.1). |
| `mycelium_l1::checkty::prim_sig` | fn | `crates/mycelium-l1/src/checkty.rs:3488` | The builtin prim signature table `Π` (RFC-0007 §4.4 T-Op), width-polymorphic. |
| `mycelium_l1::checkty::type_head` | fn | `crates/mycelium-l1/src/checkty.rs:229` | The **coherence key** of a type (RFC-0019 §4.5): the head a `(trait, type-head)` instance key is |
| `mycelium_l1::elab` | mod | `crates/mycelium-l1/src/lib.rs:41` | — |
| `mycelium_l1::elab::build_registry` | fn | `crates/mycelium-l1/src/elab.rs:516` | Build the content-addressed data registry `Σ` (RFC-0001 §4.3 r3) from the checked environment's |
| `mycelium_l1::elab::lit_value` | fn | `crates/mycelium-l1/src/elab.rs:84` | Build the L0 [`Value`] of a representation literal (Q6: a literal *is* its representation — |
| `mycelium_l1::elab::policy_name_ref` | fn | `crates/mycelium-l1/src/elab.rs:192` | The v0 **policy-name reference**: a deterministic, domain-separated content address derived |
| `mycelium_l1::elab::type_repr` | fn | `crates/mycelium-l1/src/elab.rs:143` | Resolve a surface [`TypeRef`] to a kernel [`Repr`] (swap targets). |
| `mycelium_l1::elaborate` | fn | `crates/mycelium-l1/src/elab.rs:213` | Elaborate the nullary function `entry` of a checked nodule to a closed L0 [`Node`]. |
| `mycelium_l1::elaborate_colony` | fn | `crates/mycelium-l1/src/elab.rs:250` | **Per-hypha elaboration of a `colony` entry** for the *real-concurrency* execution path |
| `mycelium_l1::error` | mod | `crates/mycelium-l1/src/lib.rs:42` | — |
| `mycelium_l1::error::ParseError::at` | fn | `crates/mycelium-l1/src/error.rs:27` | Ergonomic alias for [`ParseError::new`] taking any `impl Into<String>` message (so a `&str` |
| `mycelium_l1::error::ParseError::new` | fn | `crates/mycelium-l1/src/error.rs:19` | Build an error at `pos`. |
| `mycelium_l1::eval` | mod | `crates/mycelium-l1/src/lib.rs:43` | — |
| `mycelium_l1::eval::EvaluatorOpts` | struct | `crates/mycelium-l1/src/eval.rs:217` | The tunable **budgets** of an [`Evaluator`] — the step (`fuel`) and recursion-depth guards — as |
| `mycelium_l1::eval::EvaluatorOpts::depth` | fn | `crates/mycelium-l1/src/eval.rs:247` | Set the recursion-depth budget (builder-style), leaving `fuel` untouched. |
| `mycelium_l1::eval::EvaluatorOpts::fuel` | fn | `crates/mycelium-l1/src/eval.rs:240` | Set the step budget (builder-style), leaving `depth` untouched. |
| `mycelium_l1::eval::L1Value::as_repr` | fn | `crates/mycelium-l1/src/eval.rs:59` | The underlying L0 value, if this is a representation value. |
| `mycelium_l1::eval::L1Value::to_core` | fn | `crates/mycelium-l1/src/eval.rs:74` | Project this L1 value onto the L0 [`CoreValue`] domain, resolving each constructor's |
| `mycelium_l1::eval::strength_of` | fn | `crates/mycelium-l1/src/eval.rs:175` | The surface strength keyword's kernel lattice point. |
| `mycelium_l1::expand_phylum_to_source` | fn | `crates/mycelium-l1/src/ambient.rs:227` | Render a whole [`Phylum`] back to canonical surface text (M-662): the optional `phylum <path>` |
| `mycelium_l1::expand_to_source` | fn | `crates/mycelium-l1/src/ambient.rs:198` | Render a (resolved or partly-resolved) [`Nodule`] back to canonical surface text — the M-142/LSP |
| `mycelium_l1::lexer` | mod | `crates/mycelium-l1/src/lib.rs:48` | — |
| `mycelium_l1::lexer::Comment` | struct | `crates/mycelium-l1/src/lexer.rs:32` | A captured `//` line comment, produced by [`lex_with_comments`]. |
| `mycelium_l1::lexer::lex` | fn | `crates/mycelium-l1/src/lexer.rs:70` | Tokenize `src` into a [`Spanned`] stream terminated by [`Tok::Eof`]. |
| `mycelium_l1::lexer::lex_with_comments` | fn | `crates/mycelium-l1/src/lexer.rs:84` | Tokenize `src`, returning the [`Spanned`] token stream **and** an ordered [`Vec<Comment>`] |
| `mycelium_l1::mono` | mod | `crates/mycelium-l1/src/lib.rs:49` | — |
| `mycelium_l1::mono::HofSpecialization` | struct | `crates/mycelium-l1/src/mono.rs:245` | The **EXPLAIN record** of a single HOF defunctionalization (RFC-0024 §4, M-687): which |
| `mycelium_l1::mono::MonoSelections::get` | fn | `crates/mycelium-l1/src/mono.rs:95` | The selection mono made for the mangled callee `mangled`, if any. |
| `mycelium_l1::mono::MonoSelections::hof` | fn | `crates/mycelium-l1/src/mono.rs:120` | The HOF defunctionalization record for the mangled specialization `mangled`, if any |
| `mycelium_l1::mono::MonoSelections::hof_iter` | fn | `crates/mycelium-l1/src/mono.rs:125` | Every recorded HOF specialization, in deterministic (mangled-name) order. |
| `mycelium_l1::mono::MonoSelections::is_empty` | fn | `crates/mycelium-l1/src/mono.rs:113` | Were no trait-method selections recorded? (A non-trait program monomorphizes with an empty |
| `mycelium_l1::mono::MonoSelections::iter` | fn | `crates/mycelium-l1/src/mono.rs:100` | Every recorded selection, in deterministic (mangled-name) order. |
| `mycelium_l1::mono::MonoSelections::len` | fn | `crates/mycelium-l1/src/mono.rs:106` | How many distinct trait-method instances were resolved (0 for a non-trait program). |
| `mycelium_l1::monomorphize` | fn | `crates/mycelium-l1/src/mono.rs:140` | Monomorphize a checked `Env` from nullary monomorphic `entry`, returning a closed monomorphic |
| `mycelium_l1::monomorphize_with_selections` | fn | `crates/mycelium-l1/src/mono.rs:149` | Like [`monomorphize`] but also returns the [`MonoSelections`] EXPLAIN record of every trait-method |
| `mycelium_l1::nodule` | mod | `crates/mycelium-l1/src/lib.rs:50` | — |
| `mycelium_l1::nodule::NoduleHeader::canonical` | fn | `crates/mycelium-l1/src/nodule.rs:40` | The canonical one-line spelling of this marker — what the formatter (M-142) emits. |
| `mycelium_l1::nodule::NoduleHeader::dotted` | fn | `crates/mycelium-l1/src/nodule.rs:34` | The dotted name as written (`"geometry.shapes"`), or `None` for the bare marker. |
| `mycelium_l1::parse` | mod | `crates/mycelium-l1/src/lib.rs:51` | — |
| `mycelium_l1::parse` | fn | `crates/mycelium-l1/src/parse.rs:26` | Parse a complete **single-`nodule`** program from source — the v0 entry point, unchanged by the |
| `mycelium_l1::parse_nodule_header` | fn | `crates/mycelium-l1/src/nodule.rs:75` | Recognise the optional nodule header marker on the first non-blank line of `src`. |
| `mycelium_l1::parse_phylum` | fn | `crates/mycelium-l1/src/parse.rs:48` | Parse a complete **phylum** program (M-662; RFC-0006 §4.3): an optional `phylum <path>` header |
| `mycelium_l1::resolve` | fn | `crates/mycelium-l1/src/ambient.rs:144` | Resolve a parsed [`Nodule`] to its longhand twin (RFC-0012 §4.3/§4.4). |
| `mycelium_l1::resolve_report` | fn | `crates/mycelium-l1/src/ambient.rs:152` | Like [`resolve`], but also returns the provenance trace ([`ResolutionNote`]s) for EXPLAIN (§4.3). |
| `mycelium_l1::token` | mod | `crates/mycelium-l1/src/lib.rs:52` | — |
| `mycelium_l1::token::Pos` | struct | `crates/mycelium-l1/src/token.rs:5` | A 1-based source position, for never-silent parse diagnostics. |
| `mycelium_l1::token::ScalarTok` | enum | `crates/mycelium-l1/src/token.rs:247` | Scalar-kind keyword payload. |
| `mycelium_l1::token::Spanned` | struct | `crates/mycelium-l1/src/token.rs:273` | A token with its starting position. |
| `mycelium_l1::token::StrengthTok` | enum | `crates/mycelium-l1/src/token.rs:260` | Guarantee-strength keyword payload. |
| `mycelium_l1::token::Tok` | enum | `crates/mycelium-l1/src/token.rs:22` | A lexical token. |
| `mycelium_l1::token::keyword` | fn | `crates/mycelium-l1/src/token.rs:282` | Resolve an identifier-shaped lexeme to its keyword token, or `None` if it is a plain identifier. |
| `mycelium_l1::totality` | mod | `crates/mycelium-l1/src/lib.rs:53` | — |
| `mycelium_l1::totality::classify_all` | fn | `crates/mycelium-l1/src/totality.rs:45` | Classify every function in the table. |

## mycelium-lint

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_lint::Fix` | struct | `crates/mycelium-lint/src/lib.rs:56` | A reified fix offer for a finding. |
| `mycelium_lint::FixTier::as_str` | fn | `crates/mycelium-lint/src/lib.rs:45` | The canonical label. |
| `mycelium_lint::LintFinding` | struct | `crates/mycelium-lint/src/lib.rs:79` | One lint finding with its (optional) reified fix. |
| `mycelium_lint::LintFinding::with_fix` | fn | `crates/mycelium-lint/src/lib.rs:97` | Attach a reified fix offer, fluently (M-644 ergonomics). |
| `mycelium_lint::LintReport` | struct | `crates/mycelium-lint/src/lib.rs:105` | The aggregated lint result. |
| `mycelium_lint::LintReport::has_errors` | fn | `crates/mycelium-lint/src/lib.rs:130` | Whether any finding is an error-severity house-rule violation. |
| `mycelium_lint::LintReport::tier_counts` | fn | `crates/mycelium-lint/src/lib.rs:136` | Counts by tier: (apply, suggest, scaffold). |
| `mycelium_lint::LintReport::with_files` | fn | `crates/mycelium-lint/src/lib.rs:123` | Set the linted-file count, fluently (M-644 ergonomics). |
| `mycelium_lint::LintReport::with_finding` | fn | `crates/mycelium-lint/src/lib.rs:116` | Push a finding, fluently (M-644 ergonomics). |
| `mycelium_lint::doc_lint_status` | fn | `crates/mycelium-lint/src/lib.rs:230` | The status line for the §4.1 doc lint — now **active** (it runs over the M-363 doc-IR via `myc-doc`, |
| `mycelium_lint::lint_source` | fn | `crates/mycelium-lint/src/lib.rs:241` | Lint one source, appending findings. |
| `mycelium_lint::lint_sources` | fn | `crates/mycelium-lint/src/lib.rs:277` | Lint an explicit set of `(file, contents)` sources, deterministically. |
| `mycelium_lint::recovery_scaffold` | fn | `crates/mycelium-lint/src/lib.rs:203` | Generate an RFC-0014 **recovery scaffold** for an error `class` under a named, bounded [`RecoveryProfile`] |

## mycelium-lsp

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_lsp::AuditView` | struct | `crates/mycelium-lsp/src/diagnostics/audit.rs:34` | The audit view: every crossing in a program, in deterministic traversal order. |
| `mycelium_lsp::BaselineRule` | struct | `crates/mycelium-lsp/src/baseline.rs:32` | The auto-derived baseline for one error class: its presentation level + route, and the *rationale* |
| `mycelium_lsp::ClassRegistry` | struct | `crates/mycelium-lsp/src/diagnostics/registry.rs:60` | The known set of error-class names a policy may name (RFC-0013 §4.5). |
| `mycelium_lsp::CompletionItem` | struct | `crates/mycelium-lsp/src/completions.rs:47` | A single LSP completion item (minimal fields: `label`, `kind`, `insertText`, |
| `mycelium_lsp::Crossing` | struct | `crates/mycelium-lsp/src/diagnostics/audit.rs:18` | One representation crossing (`swap` site) and what the audit can read off it. |
| `mycelium_lsp::DEPTH_LIMIT:` | const | `crates/mycelium-lsp/src/llm_canonical_parser.rs:40` | Maximum nesting depth (banked guard #4 — depth limit prevents stack overflow). |
| `mycelium_lsp::Diagnostic` | struct | `crates/mycelium-lsp/src/lint.rs:35` | A single lint finding. |
| `mycelium_lsp::DiagnosticPolicy` | struct | `crates/mycelium-lsp/src/diagnostics/policy.rs:86` | A reified error-handling policy: a map from a **registry-resolved** [`ClassName`] to its [`Rule`]. |
| `mycelium_lsp::DiagnosticRecord` | struct | `crates/mycelium-lsp/src/diagnostics/record.rs:109` | One **content-addressed diagnostic** (§4.3) — the canonical truth. |
| `mycelium_lsp::DocumentStore` | struct | `crates/mycelium-lsp/src/sync.rs:27` | An in-memory store of open documents (`uri → source text`), the minimal state full-sync requires. |
| `mycelium_lsp::ExplainSite` | struct | `crates/mycelium-lsp/src/feedback.rs:86` | A surfaced selection EXPLAIN (M-221; RFC-0005 §4): the swap site and the re-derived trace. |
| `mycelium_lsp::Feedback` | struct | `crates/mycelium-lsp/src/feedback.rs:95` | The aggregated feedback surface (SC-5 channel) for one Core IR program. |
| `mycelium_lsp::FeedbackSummary` | struct | `crates/mycelium-lsp/src/feedback.rs:117` | A structured, at-a-glance rollup of a [`Feedback`] (M-310): per-artifact-kind counts and the |
| `mycelium_lsp::GuaranteeAnnotation` | struct | `crates/mycelium-lsp/src/feedback.rs:42` | A per-value honesty annotation: where it is, its guarantee tag, and its bound (if approximate). |
| `mycelium_lsp::KEYWORD_COMPLETIONS:` | const | `crates/mycelium-lsp/src/completions.rs:90` | The complete set of active keyword completions. |
| `mycelium_lsp::Level` | enum | `crates/mycelium-lsp/src/diagnostics/record.rs:24` | A graded context **level** — a verbosity knob over *one* truth (§4.2). |
| `mycelium_lsp::Outcome` | enum | `crates/mycelium-lsp/src/recover/mod.rs:67` | The result sum `Ok(τ) \| Err(ε)` (RFC-0014 §4.1). |
| `mycelium_lsp::ParseError` | enum | `crates/mycelium-lsp/src/llm_canonical_parser.rs:44` | Errors returned by [`parse_llm_canonical`] (G2: always explicit, never silent). |
| `mycelium_lsp::Presentation` | struct | `crates/mycelium-lsp/src/diagnostics/record.rs:136` | The result of presenting an error: the **additive** diagnostic *and* the explicit error, **still |
| `mycelium_lsp::RESILIENT_MAX_ATTEMPTS:` | const | `crates/mycelium-lsp/src/baseline.rs:183` | The bounded retry ceiling the `resilient` profile applies (RFC-0015 §4.1 example `retry(<=3)`; I4). |
| `mycelium_lsp::ReasonedError` | struct | `crates/mycelium-lsp/src/diagnostics/record.rs:64` | The **explicit, already-emitted reasoned error** this layer *presents* — never replaces (I1). |
| `mycelium_lsp::RecoveryPolicy` | struct | `crates/mycelium-lsp/src/recover/policy.rs:50` | A reified recovery policy: a map from a **registry-resolved** [`ClassName`] to its [`RecoveryAction`]. |
| `mycelium_lsp::RecoveryProfile` | enum | `crates/mycelium-lsp/src/baseline.rs:152` | The **closed v0** set of named, opt-in, bounded recovery profiles (RFC-0015 §8-Q2; A2). |
| `mycelium_lsp::Resolution` | enum | `crates/mycelium-lsp/src/recover/mod.rs:79` | The outcome of handling: an error is **either recovered** (an explicit value with an honest |
| `mycelium_lsp::Rule` | struct | `crates/mycelium-lsp/src/diagnostics/policy.rs:28` | A single `on <ErrorClass> => { … }` rule. |
| `mycelium_lsp::SNIPPET_COMPLETIONS:` | const | `crates/mycelium-lsp/src/completions.rs:462` | The set of high-value scaffolding snippets. |
| `mycelium_lsp::Severity` | enum | `crates/mycelium-lsp/src/lint.rs:26` | Severity of a [`Diagnostic`]. |
| `mycelium_lsp::StructuredError` | struct | `crates/mycelium-lsp/src/recover/mod.rs:43` | The structured error value — the `Err` payload of the result sum (RFC-0001; the same structured |
| `mycelium_lsp::SwapSite` | struct | `crates/mycelium-lsp/src/feedback.rs:53` | A swap site and the certificate it emits (when statically resolvable). |
| `mycelium_lsp::UnknownClass` | struct | `crates/mycelium-lsp/src/diagnostics/registry.rs:36` | Resolving an error-class name not in the registry — an **explicit** configuration error (X1: never |
| `mycelium_lsp::analyze` | fn | `crates/mycelium-lsp/src/feedback.rs:183` | Analyze a Core IR program and return the feedback artifact kinds over one surface. |
| `mycelium_lsp::analyze_with` | fn | `crates/mycelium-lsp/src/feedback.rs:192` | [`analyze`], plus the **EXPLAIN channel** (M-221; SC-5): every swap site whose `PolicyRef` |
| `mycelium_lsp::baseline` | mod | `crates/mycelium-lsp/src/lib.rs:10` | — |
| `mycelium_lsp::baseline::RecoveryProfile::all` | fn | `crates/mycelium-lsp/src/baseline.rs:171` | The closed v0 set, for enumeration / exhaustive tests. |
| `mycelium_lsp::baseline::RecoveryProfile::as_str` | fn | `crates/mycelium-lsp/src/baseline.rs:162` | The canonical profile name. |
| `mycelium_lsp::baseline::RecoveryProfile::resolve` | fn | `crates/mycelium-lsp/src/baseline.rs:177` | Resolve a profile name against the closed set (looked up, never evaluated). |
| `mycelium_lsp::baseline_for_class` | fn | `crates/mycelium-lsp/src/baseline.rs:45` | The **total** baseline derivation (A4): a deterministic function of the class name — a closed table |
| `mycelium_lsp::check_effects` | fn | `crates/mycelium-lsp/src/recover/effect.rs:58` | The **compositional no-undeclared-effect check** (I3): every effect a definition *performs* (its own |
| `mycelium_lsp::completion_list` | fn | `crates/mycelium-lsp/src/completions.rs:538` | Return the full list of completion items (keywords + snippets) as an LSP |
| `mycelium_lsp::completions` | mod | `crates/mycelium-lsp/src/lib.rs:11` | — |
| `mycelium_lsp::completions::CompletionItem::to_lsp_value` | fn | `crates/mycelium-lsp/src/completions.rs:65` | Serialize this item to an LSP `CompletionItem` JSON value. |
| `mycelium_lsp::derive_baseline` | fn | `crates/mycelium-lsp/src/baseline.rs:97` | Derive the baseline [`DiagnosticPolicy`] for **every** class in `registry` (the broadest scope). |
| `mycelium_lsp::derive_baseline_for` | fn | `crates/mycelium-lsp/src/baseline.rs:110` | Derive the baseline scoped to a **definition's declared effect classes** (the classes it can raise; |
| `mycelium_lsp::diagnostics` | mod | `crates/mycelium-lsp/src/lib.rs:12` | — |
| `mycelium_lsp::diagnostics::ClassName` | struct | `crates/mycelium-lsp/src/diagnostics/registry.rs:17` | A **resolved** error-class name. |
| `mycelium_lsp::diagnostics::DETAILED_ALLOWLIST:` | const | `crates/mycelium-lsp/src/diagnostics/record.rs:36` | The **allowlist** for the detailed tier (§4.5, exclusion X2): the *only* context-field names a |
| `mycelium_lsp::diagnostics::Delivery` | enum | `crates/mycelium-lsp/src/diagnostics/sink.rs:161` | The **honest delivery semantics** of a sink (RT5). |
| `mycelium_lsp::diagnostics::PolicyFile` | struct | `crates/mycelium-lsp/src/diagnostics/policy.rs:158` | A serializable projection of a policy (RFC-0013 §4.7: a file is a *projection of* the canonical |
| `mycelium_lsp::diagnostics::Route` | enum | `crates/mycelium-lsp/src/diagnostics/sink.rs:32` | The **closed v0 set** of diagnostic routes (RFC-0013 §8). |
| `mycelium_lsp::diagnostics::SinkBinding` | struct | `crates/mycelium-lsp/src/diagnostics/sink.rs:215` | A resolved binding of a [`Route`] to its RFC-0008 sink and the sink's honest [`Delivery`] guarantee. |
| `mycelium_lsp::diagnostics::UnknownRoute` | struct | `crates/mycelium-lsp/src/diagnostics/sink.rs:138` | A `route` string that is not in the closed v0 [`Route`] set — an explicit configuration error |
| `mycelium_lsp::diagnostics::audit` | mod | `crates/mycelium-lsp/src/diagnostics/mod.rs:26` | — |
| `mycelium_lsp::diagnostics::audit::AuditView::of` | fn | `crates/mycelium-lsp/src/diagnostics/audit.rs:42` | Build the audit view for a Core IR program — enumerating **every** `swap` (I5). |
| `mycelium_lsp::diagnostics::audit::AuditView::to_human` | fn | `crates/mycelium-lsp/src/diagnostics/audit.rs:56` | The human projection: one line per crossing, honesty bound named (or `unknown`, never faked). |
| `mycelium_lsp::diagnostics::audit::AuditView::to_json` | fn | `crates/mycelium-lsp/src/diagnostics/audit.rs:50` | The JSON projection (§4.3 dual-projection form — this view is read-only structured output). |
| `mycelium_lsp::diagnostics::policy` | mod | `crates/mycelium-lsp/src/diagnostics/mod.rs:27` | — |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::content_id` | fn | `crates/mycelium-lsp/src/diagnostics/policy.rs:134` | The **content address** of this policy (RFC-0005 `PolicyRef`; ADR-006) — a deterministic |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::from_file` | fn | `crates/mycelium-lsp/src/diagnostics/policy.rs:183` | Ingest a [`PolicyFile`], **resolving every class name through the registry** (X1). |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::is_empty` | fn | `crates/mycelium-lsp/src/diagnostics/policy.rs:126` | Whether the policy has no rules. |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::on` | fn | `crates/mycelium-lsp/src/diagnostics/policy.rs:103` | Add a rule for `class`, **resolving the class name through the registry** (X1). |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::rule_for` | fn | `crates/mycelium-lsp/src/diagnostics/policy.rs:115` | The rule for a resolved class, if any. |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::rules` | fn | `crates/mycelium-lsp/src/diagnostics/policy.rs:120` | The rules, in deterministic (class-sorted) order. |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::to_file` | fn | `crates/mycelium-lsp/src/diagnostics/policy.rs:167` | Project this policy to a serializable [`PolicyFile`] (one on-disk form; §4.7). |
| `mycelium_lsp::diagnostics::policy::Rule::level` | fn | `crates/mycelium-lsp/src/diagnostics/policy.rs:63` | Set the level. |
| `mycelium_lsp::diagnostics::policy::Rule::message` | fn | `crates/mycelium-lsp/src/diagnostics/policy.rs:51` | Set the presentation message. |
| `mycelium_lsp::diagnostics::policy::Rule::new` | fn | `crates/mycelium-lsp/src/diagnostics/policy.rs:46` | An empty rule (all defaults). |
| `mycelium_lsp::diagnostics::policy::Rule::route` | fn | `crates/mycelium-lsp/src/diagnostics/policy.rs:72` | Set the route from a free-form string (the on-the-wire/`PolicyFile` projection form). |
| `mycelium_lsp::diagnostics::policy::Rule::route_to` | fn | `crates/mycelium-lsp/src/diagnostics/policy.rs:78` | Set the route from the **closed v0** Route vocabulary (the checked path). |
| `mycelium_lsp::diagnostics::policy::Rule::tag` | fn | `crates/mycelium-lsp/src/diagnostics/policy.rs:57` | Add a tag. |
| `mycelium_lsp::diagnostics::record` | mod | `crates/mycelium-lsp/src/diagnostics/mod.rs:28` | — |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::content_id` | fn | `crates/mycelium-lsp/src/diagnostics/record.rs:237` | The **content address** of this diagnostic (§4.3; ADR-003) — a deterministic BLAKE3 over its |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::from_json` | fn | `crates/mycelium-lsp/src/diagnostics/record.rs:283` | Recover a record from its JSON projection (I3). |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::sink` | fn | `crates/mycelium-lsp/src/diagnostics/record.rs:227` | Resolve this diagnostic's `route` to its RFC-0008 [`SinkBinding`] (M-354, RFC-0013 §8). |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::to_human` | fn | `crates/mycelium-lsp/src/diagnostics/record.rs:292` | The **human projection** (§4.3), graded by self.level. |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::to_json` | fn | `crates/mycelium-lsp/src/diagnostics/record.rs:267` | The **JSON projection** (§4.3): the lossless, round-trippable machine record, with its |
| `mycelium_lsp::diagnostics::record::ReasonedError::new` | fn | `crates/mycelium-lsp/src/diagnostics/record.rs:81` | A minimal reasoned error (class + message + site), no reason or context. |
| `mycelium_lsp::diagnostics::record::ReasonedError::with_context` | fn | `crates/mycelium-lsp/src/diagnostics/record.rs:100` | Attach a candidate detailed-tier context field (allowlist-filtered at projection). |
| `mycelium_lsp::diagnostics::record::ReasonedError::with_reason` | fn | `crates/mycelium-lsp/src/diagnostics/record.rs:93` | Attach a medium-tier reason. |
| `mycelium_lsp::diagnostics::registry` | mod | `crates/mycelium-lsp/src/diagnostics/mod.rs:29` | — |
| `mycelium_lsp::diagnostics::registry::ClassName::as_str` | fn | `crates/mycelium-lsp/src/diagnostics/registry.rs:22` | The class name as a string. |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::classes` | fn | `crates/mycelium-lsp/src/diagnostics/registry.rs:134` | The known class names, sorted (deterministic). |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::contains` | fn | `crates/mycelium-lsp/src/diagnostics/registry.rs:114` | Whether `name` is a known class. |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::new` | fn | `crates/mycelium-lsp/src/diagnostics/registry.rs:92` | An empty registry — resolves nothing until classes are registered. |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::register` | fn | `crates/mycelium-lsp/src/diagnostics/registry.rs:108` | Register a downstream error class. |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::resolve` | fn | `crates/mycelium-lsp/src/diagnostics/registry.rs:123` | Resolve a name to a [`ClassName`] **through the registry** — the only way to obtain one. |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::with_builtins` | fn | `crates/mycelium-lsp/src/diagnostics/registry.rs:100` | The registry seeded with the v0 built-in classes (`BUILTIN_CLASSES`). |
| `mycelium_lsp::diagnostics::sink` | mod | `crates/mycelium-lsp/src/diagnostics/mod.rs:30` | — |
| `mycelium_lsp::diagnostics::sink::Delivery::delivers` | fn | `crates/mycelium-lsp/src/diagnostics/sink.rs:185` | Whether the sink actually **delivers** the presentation. |
| `mycelium_lsp::diagnostics::sink::Delivery::guarantee` | fn | `crates/mycelium-lsp/src/diagnostics/sink.rs:193` | The honest **delivery guarantee** on the lattice (RT5/VR-5): `None` for the null sink (nothing |
| `mycelium_lsp::diagnostics::sink::Delivery::probability_bound` | fn | `crates/mycelium-lsp/src/diagnostics/sink.rs:205` | The probabilistic delivery bound, if this is a probabilistic sink (the mesh δ; RT5). |
| `mycelium_lsp::diagnostics::sink::Route::all` | fn | `crates/mycelium-lsp/src/diagnostics/sink.rs:64` | The closed v0 set, in declaration order (for enumeration / exhaustive tests). |
| `mycelium_lsp::diagnostics::sink::Route::as_str` | fn | `crates/mycelium-lsp/src/diagnostics/sink.rs:52` | The canonical route string (the on-the-wire/`PolicyFile` projection name). |
| `mycelium_lsp::diagnostics::sink::Route::binding` | fn | `crates/mycelium-lsp/src/diagnostics/sink.rs:91` | The RFC-0008 sink this route binds to, with its **honest delivery guarantee** (RT5). |
| `mycelium_lsp::diagnostics::sink::Route::resolve` | fn | `crates/mycelium-lsp/src/diagnostics/sink.rs:80` | Resolve a `route` string to its [`Route`] — **checked against the closed v0 set** (the §4.5 X1 |
| `mycelium_lsp::expand` | mod | `crates/mycelium-lsp/src/lib.rs:13` | — |
| `mycelium_lsp::expand_ambient` | fn | `crates/mycelium-lsp/src/expand.rs:26` | Render `text`'s fully-resolved longhand twin (paradigm tags filled, `with paradigm` blocks |
| `mycelium_lsp::explain_baseline` | fn | `crates/mycelium-lsp/src/baseline.rs:134` | The `EXPLAIN` of the baseline derivation over `registry` (A3): every class with its derived level, |
| `mycelium_lsp::feedback` | mod | `crates/mycelium-lsp/src/lib.rs:14` | — |
| `mycelium_lsp::feedback::Feedback::summary` | fn | `crates/mycelium-lsp/src/feedback.rs:148` | Summarize this feedback into a [`FeedbackSummary`] (M-310). |
| `mycelium_lsp::feedback::FeedbackSummary::is_clean` | fn | `crates/mycelium-lsp/src/feedback.rs:140` | Clean = no `Error`-severity diagnostics — the gate [`crate::lint::has_errors`] checks, lifted |
| `mycelium_lsp::feedback::PrimSite` | struct | `crates/mycelium-lsp/src/feedback.rs:71` | A surfaced **prim declaration** at an `Op` site (M-390; R7-Q4; DN-10 §3.2 step 4): the |
| `mycelium_lsp::fmt` | mod | `crates/mycelium-lsp/src/lib.rs:15` | — |
| `mycelium_lsp::format` | fn | `crates/mycelium-lsp/src/fmt.rs:16` | Format a Core IR node into its canonical textual normal form (α-normalized binders). |
| `mycelium_lsp::handle` | fn | `crates/mycelium-lsp/src/recover/mod.rs:110` | Handle an [`Outcome`] under a reified recovery `policy`, drawing on a budget ledger and an |
| `mycelium_lsp::has_errors` | fn | `crates/mycelium-lsp/src/lint.rs:71` | Whether `lint` found at least one `Error`-severity diagnostic. |
| `mycelium_lsp::lint` | mod | `crates/mycelium-lsp/src/lib.rs:16` | — |
| `mycelium_lsp::lint` | fn | `crates/mycelium-lsp/src/lint.rs:62` | Lint a (closed) Core IR program, returning all findings in deterministic traversal order. |
| `mycelium_lsp::lint::Diagnostic::path` | fn | `crates/mycelium-lsp/src/lint.rs:51` | The breadcrumb [`Self::at`] as a structured, navigable path (split on `/`) — so a client can |
| `mycelium_lsp::lint_nodule_header` | fn | `crates/mycelium-lsp/src/lint.rs:81` | The **source-text** companion lint (M-141; DN-06 §6): recognise the `// nodule:` header marker |
| `mycelium_lsp::lint_structured_header` | fn | `crates/mycelium-lsp/src/lint.rs:103` | The **structured-header** lint (M-141; M-359 / spec §3): parse the `// @key: value` header and |
| `mycelium_lsp::llm_canonical_parser` | mod | `crates/mycelium-lsp/src/lib.rs:17` | — |
| `mycelium_lsp::parse_llm_canonical` | fn | `crates/mycelium-lsp/src/llm_canonical_parser.rs:94` | Parse a LlmCanonical source string into a normalized Core-IR S-expression string. |
| `mycelium_lsp::present` | fn | `crates/mycelium-lsp/src/diagnostics/record.rs:151` | Present an explicit [`ReasonedError`] as a [`DiagnosticRecord`], optionally shaped by a policy. |
| `mycelium_lsp::project` | mod | `crates/mycelium-lsp/src/lib.rs:18` | — |
| `mycelium_lsp::project::llm_canonical` | fn | `crates/mycelium-lsp/src/project.rs:40` | Render a closed Core IR [`Node`] as the `LlmCanonical` s-expression surface (RFC-0021 §4.6). |
| `mycelium_lsp::publish_diagnostics_notification` | fn | `crates/mycelium-lsp/src/wire.rs:79` | Build the full `textDocument/publishDiagnostics` JSON-RPC **notification** (server → client) that |
| `mycelium_lsp::publish_for_source` | fn | `crates/mycelium-lsp/src/sync.rs:83` | The full `textDocument/publishDiagnostics` notification for `uri`'s `text` (parse → check). |
| `mycelium_lsp::read_message` | fn | `crates/mycelium-lsp/src/wire.rs:115` | Read one JSON-RPC message off `reader`, decoding the `Content-Length` header framing. |
| `mycelium_lsp::recover` | mod | `crates/mycelium-lsp/src/lib.rs:19` | — |
| `mycelium_lsp::recover::EffectSet` | type | `crates/mycelium-lsp/src/recover/effect.rs:27` | A definition's **declared** effect set (§4.5 I3) — what it says it can do, on its signature. |
| `mycelium_lsp::recover::StructuredError::new` | fn | `crates/mycelium-lsp/src/recover/mod.rs:55` | A structured error. |
| `mycelium_lsp::recover::UndeclaredEffect` | struct | `crates/mycelium-lsp/src/recover/effect.rs:33` | An effect a definition performs but did **not** declare (I3) — an explicit checker error, never |
| `mycelium_lsp::recover::effect` | mod | `crates/mycelium-lsp/src/recover/mod.rs:25` | — |
| `mycelium_lsp::recover::policy` | mod | `crates/mycelium-lsp/src/recover/mod.rs:26` | — |
| `mycelium_lsp::recover::policy::RecoveryAction` | enum | `crates/mycelium-lsp/src/recover/policy.rs:20` | The **closed** v0 recovery-action set (§4.4; §8 resolved). |
| `mycelium_lsp::recover::policy::RecoveryPolicy::action_for` | fn | `crates/mycelium-lsp/src/recover/policy.rs:78` | The recovery action for a resolved class, if any. |
| `mycelium_lsp::recover::policy::RecoveryPolicy::content_id` | fn | `crates/mycelium-lsp/src/recover/policy.rs:96` | The **content address** of this policy (RFC-0005 `PolicyRef`; ADR-006) — a deterministic BLAKE3 |
| `mycelium_lsp::recover::policy::RecoveryPolicy::is_empty` | fn | `crates/mycelium-lsp/src/recover/policy.rs:89` | Whether the policy has no rules. |
| `mycelium_lsp::recover::policy::RecoveryPolicy::new` | fn | `crates/mycelium-lsp/src/recover/policy.rs:57` | An empty policy. |
| `mycelium_lsp::recover::policy::RecoveryPolicy::on` | fn | `crates/mycelium-lsp/src/recover/policy.rs:66` | Add an action for `class`, **resolving the class name through the registry** (X1). |
| `mycelium_lsp::recover::policy::RecoveryPolicy::rules` | fn | `crates/mycelium-lsp/src/recover/policy.rs:83` | The rules, in deterministic (class-sorted) order. |
| `mycelium_lsp::recovery_profile` | fn | `crates/mycelium-lsp/src/baseline.rs:191` | Build a [`RecoveryPolicy`] from a named [`RecoveryProfile`] over the **explicitly supplied** classes |
| `mycelium_lsp::resilient_publish_for_source` | fn | `crates/mycelium-lsp/src/sync.rs:110` | The resilient counterpart of [`publish_for_source`]: the server-boundary builder that the |
| `mycelium_lsp::resilient_source_diagnostics` | fn | `crates/mycelium-lsp/src/sync.rs:102` | Like [`source_diagnostics`], but **isolating an internal analysis panic** as a structured |
| `mycelium_lsp::serve` | fn | `crates/mycelium-lsp/src/wire.rs:182` | Drive the LSP lifecycle **with document sync** (M-310) over `reader`/`writer` (stdio in the real |
| `mycelium_lsp::serve_stdio` | fn | `crates/mycelium-lsp/src/wire.rs:254` | Run [`serve`] over the process's **real stdio** — the entry point an editor launches |
| `mycelium_lsp::source_diagnostics` | fn | `crates/mycelium-lsp/src/sync.rs:71` | Analyze a document's source through the text → `Node` pipeline and return its LSP diagnostics |
| `mycelium_lsp::sync` | mod | `crates/mycelium-lsp/src/lib.rs:20` | — |
| `mycelium_lsp::sync::DocumentStore::is_empty` | fn | `crates/mycelium-lsp/src/sync.rs:62` | Whether the store is empty. |
| `mycelium_lsp::sync::DocumentStore::len` | fn | `crates/mycelium-lsp/src/sync.rs:56` | Number of open documents. |
| `mycelium_lsp::sync::DocumentStore::new` | fn | `crates/mycelium-lsp/src/sync.rs:34` | An empty store. |
| `mycelium_lsp::sync::DocumentStore::remove` | fn | `crates/mycelium-lsp/src/sync.rs:44` | Drop a document (`didClose`). |
| `mycelium_lsp::sync::DocumentStore::set` | fn | `crates/mycelium-lsp/src/sync.rs:39` | Record (or replace) a document's full text (`didOpen` / `didChange` full sync). |
| `mycelium_lsp::sync::DocumentStore::text` | fn | `crates/mycelium-lsp/src/sync.rs:50` | The stored text for `uri`, if open. |
| `mycelium_lsp::to_lsp_diagnostic` | fn | `crates/mycelium-lsp/src/wire.rs:47` | Map a [`Diagnostic`] to an LSP-`Diagnostic` JSON value. |
| `mycelium_lsp::wire` | mod | `crates/mycelium-lsp/src/lib.rs:21` | — |
| `mycelium_lsp::wire::SERVER_NAME:` | const | `crates/mycelium-lsp/src/wire.rs:31` | The advertised server name (LSP `serverInfo.name`). |
| `mycelium_lsp::wire::initialize_result` | fn | `crates/mycelium-lsp/src/wire.rs:97` | The `initialize` result: the server's advertised capabilities. |
| `mycelium_lsp::wire::lsp_severity` | fn | `crates/mycelium-lsp/src/wire.rs:36` | LSP `DiagnosticSeverity` code for a [`Severity`] (LSP spec: Error=1, Warning=2, Information=3, |
| `mycelium_lsp::wire::publish_diagnostics_params` | fn | `crates/mycelium-lsp/src/wire.rs:64` | The `params` of a `textDocument/publishDiagnostics` notification for `feedback` at `uri`. |
| `mycelium_lsp::write_message` | fn | `crates/mycelium-lsp/src/wire.rs:154` | Write one JSON-RPC message to `writer` with the `Content-Length` framing, then flush. |

## mycelium-mlir

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_mlir::AotError` | enum | `crates/mycelium-mlir/src/llvm.rs:60` | An explicit failure of the direct-LLVM AOT path. |
| `mycelium_mlir::AutoDepthBudget` | struct | `crates/mycelium-mlir/src/budget.rs:168` | The default policy: derive the ceiling from **detected memory headroom**, conservative fallback |
| `mycelium_mlir::Backend` | enum | `crates/mycelium-mlir/src/vr4.rs:49` | A backend whose lowering VR-4 requires to be dumpable. |
| `mycelium_mlir::BackendStage` | struct | `crates/mycelium-mlir/src/vr4.rs:114` | One backend's VR-4 obligation result: the backend, its dumpable-stage status, and the honest |
| `mycelium_mlir::BitnetDotKernel` | struct | `crates/mycelium-mlir/src/bitnet.rs:281` | A compiled, in-process BitNet dot kernel: the `.so` (in a per-artifact temp dir, cleaned on drop), |
| `mycelium_mlir::Colony` | type | `crates/mycelium-mlir/src/runtime.rs:116` | A **`colony`** — the DN-06 dynamic runtime grouping of active `hypha` (a cooperating set of |
| `mycelium_mlir::ColonyError` | enum | `crates/mycelium-mlir/src/runtime.rs:340` | Why running a `colony` through the concurrent executor refused — **always explicit, never a silent |
| `mycelium_mlir::CompiledArtifact` | struct | `crates/mycelium-mlir/src/llvm.rs:2410` | A compiled native artifact for a bit/trit-subset program: the executable on disk (in a |
| `mycelium_mlir::CrossBackendGate` | struct | `crates/mycelium-mlir/src/vr4.rs:146` | The full VR-4 cross-backend gate result for one program: a [`BackendStage`] for **every** backend. |
| `mycelium_mlir::Deadlock` | struct | `crates/mycelium-mlir/src/runtime.rs:106` | A dataflow schedule made **no progress** over a full sweep — every remaining task is parked on a |
| `mycelium_mlir::DeployError` | enum | `crates/mycelium-mlir/src/deploy.rs:60` | Why producing a deployable native artifact failed — always explicit (G2), never a guessed default, |
| `mycelium_mlir::DepthBasis` | enum | `crates/mycelium-mlir/src/budget.rs:106` | The inspectable derivation of a [`DepthResolution`] — the no-black-box record (G2). |
| `mycelium_mlir::DepthBudget` | trait | `crates/mycelium-mlir/src/budget.rs:67` | Resolves a control-stack **depth ceiling** for the AOT env-machine, with an inspectable basis. |
| `mycelium_mlir::DepthBudget::resolve` | fn | `crates/mycelium-mlir/src/inject.rs:172` | How `hash` resolves — the `EXPLAIN`-able dispatch decision (ADR-017 decision 5). |
| `mycelium_mlir::DepthResolution` | struct | `crates/mycelium-mlir/src/budget.rs:75` | A resolved depth ceiling plus the [`DepthBasis`] explaining how it was chosen. |
| `mycelium_mlir::Image` | struct | `crates/mycelium-mlir/src/inject.rs:103` | The running **image**: a hash-keyed dispatch table over a compiled overlay + an interpretable |
| `mycelium_mlir::InjectError` | enum | `crates/mycelium-mlir/src/inject.rs:69` | A failure at the dispatch/injection boundary — every variant is explicit (never a silent pass or |
| `mycelium_mlir::JitArtifact` | struct | `crates/mycelium-mlir/src/jit.rs:103` | A JIT-compiled kernel: the `.so` on disk (in a per-artifact temp dir, cleaned on drop) + the |
| `mycelium_mlir::KernelLayout` | struct | `crates/mycelium-mlir/src/bitnet.rs:56` | The **inspectable physical-layout record** a packed-ternary kernel decodes (M-610; NFR-1/NFR-4; |
| `mycelium_mlir::MemSource` | enum | `crates/mycelium-mlir/src/budget.rs:97` | Which kernel accounting figure the detected headroom came from. |
| `mycelium_mlir::NativeArtifact` | struct | `crates/mycelium-mlir/src/deploy.rs:98` | The inspectable, content-addressed descriptor of one natively-compiled program — the unit a |
| `mycelium_mlir::Network` | struct | `crates/mycelium-mlir/src/channel.rs:75` | A **Kahn process network** (RFC-0008 §4.3): the grouping whose typed SPSC channels form a |
| `mycelium_mlir::Poll` | enum | `crates/mycelium-mlir/src/runtime.rs:37` | The result of advancing a task one cooperative step. |
| `mycelium_mlir::Receiver` | struct | `crates/mycelium-mlir/src/channel.rs:121` | The **single consumer** end of a channel. |
| `mycelium_mlir::Resolution` | enum | `crates/mycelium-mlir/src/inject.rs:54` | How a [`ContentHash`] resolves in an [`Image`] — the inspectable/`EXPLAIN`-able dispatch decision |
| `mycelium_mlir::STATIC_FALLBACK_DEPTH:` | const | `crates/mycelium-mlir/src/budget.rs:60` | The conservative static fallback ceiling: the prior fixed default (M-347's `AOT_MAX_DEPTH`), |
| `mycelium_mlir::Scope` | struct | `crates/mycelium-mlir/src/runtime.rs:84` | A **structured concurrency scope** (RT7): tasks spawned here are all joined before the scope |
| `mycelium_mlir::Sender` | struct | `crates/mycelium-mlir/src/channel.rs:116` | The **single producer** end of a channel. |
| `mycelium_mlir::SpecializedDotKernel` | struct | `crates/mycelium-mlir/src/specialize.rs:92` | A compiled, in-process **weight-specialized** dot kernel: the `.so` (in a per-artifact temp dir, |
| `mycelium_mlir::StageStatus` | enum | `crates/mycelium-mlir/src/vr4.rs:94` | Whether a backend's stage was dumped, or skipped (with a reason) — never a fabricated dump (G2). |
| `mycelium_mlir::StaticDepthBudget` | struct | `crates/mycelium-mlir/src/budget.rs:239` | An explicit, fixed depth ceiling — for tests and callers that want a deterministic budget. |
| `mycelium_mlir::StaticReason` | enum | `crates/mycelium-mlir/src/budget.rs:86` | Why a [`DepthBasis::Static`] budget was used (detection not attempted, failed, or explicit). |
| `mycelium_mlir::SweepOrder` | enum | `crates/mycelium-mlir/src/runtime.rs:94` | The order a **dataflow** sweep visits still-pending children. |
| `mycelium_mlir::Task` | trait | `crates/mycelium-mlir/src/runtime.rs:60` | A cooperative task: `poll` advances it by one step. |
| `mycelium_mlir::TaskCtx` | struct | `crates/mycelium-mlir/src/runtime.rs:47` | The per-step context a task observes (the same cadence it would check fuel/depth): its cancel token |
| `mycelium_mlir::TryRecv` | enum | `crates/mycelium-mlir/src/channel.rs:138` | Why a [`Receiver::try_recv`] yielded no value. |
| `mycelium_mlir::TrySend` | enum | `crates/mycelium-mlir/src/channel.rs:128` | Why a [`Sender::try_send`] could not complete *right now*. |
| `mycelium_mlir::aot` | mod | `crates/mycelium-mlir/src/lib.rs:44` | — |
| `mycelium_mlir::aot::run_core_with_budget` | fn | `crates/mycelium-mlir/src/aot.rs:171` | [`run_core`] with **both** budgets explicit (M-347): `fuel` bounds `Fix` unfolds (time), `max_depth` |
| `mycelium_mlir::bitnet` | mod | `crates/mycelium-mlir/src/lib.rs:45` | — |
| `mycelium_mlir::bitnet::BitnetDotKernel::bind` | fn | `crates/mycelium-mlir/src/bitnet.rs:337` | **Bind once, call many** (M-682): resolve the entry point a single time into a lifetime-bound |
| `mycelium_mlir::bitnet::BitnetDotKernel::call` | fn | `crates/mycelium-mlir/src/bitnet.rs:348` | Run the kernel over `packed_weights` and `activations`, summing the first `n` ternary products. |
| `mycelium_mlir::bitnet::BitnetDotKernel::layout` | fn | `crates/mycelium-mlir/src/bitnet.rs:328` | The kernel's **inspectable physical-layout record** (M-610): the reified `Meta.physical` |
| `mycelium_mlir::bitnet::BoundBitnetDot` | struct | `crates/mycelium-mlir/src/bitnet.rs:362` | A [`BitnetDotKernel`] with its entry point resolved into a lifetime-bound `Sym` (M-682). |
| `mycelium_mlir::bitnet::KERNEL_SCHEME:` | const | `crates/mycelium-mlir/src/bitnet.rs:128` | The packing this kernel decodes inline by default. |
| `mycelium_mlir::bitnet::KernelLayout::bits_per_element` | fn | `crates/mycelium-mlir/src/bitnet.rs:92` | The **actual** bits-per-element the kernel's loads assume, measured from the byte buffer the |
| `mycelium_mlir::bitnet::KernelLayout::explain` | fn | `crates/mycelium-mlir/src/bitnet.rs:104` | A human-readable `EXPLAIN` of the physical layout — what the kernel actually reads, so the |
| `mycelium_mlir::bitnet::KernelLayout::new` | fn | `crates/mycelium-mlir/src/bitnet.rs:66` | The layout for `scheme`. |
| `mycelium_mlir::bitnet::KernelLayout::physical` | fn | `crates/mycelium-mlir/src/bitnet.rs:80` | The reified `Meta.physical` record — the [`PhysicalLayout`] that travels on a result's `Meta` |
| `mycelium_mlir::bitnet::KernelLayout::scheme` | fn | `crates/mycelium-mlir/src/bitnet.rs:72` | The packing scheme. |
| `mycelium_mlir::budget` | mod | `crates/mycelium-mlir/src/lib.rs:46` | — |
| `mycelium_mlir::budget::DEFAULT_PER_FRAME_BYTES:` | const | `crates/mycelium-mlir/src/budget.rs:44` | Conservative per-frame heap estimate (bytes). |
| `mycelium_mlir::channel` | mod | `crates/mycelium-mlir/src/lib.rs:47` | — |
| `mycelium_mlir::channel::Network::channel` | fn | `crates/mycelium-mlir/src/channel.rs:98` | Create a typed SPSC channel on this network with explicit, finite capacity `cap` (no |
| `mycelium_mlir::channel::Network::epoch` | fn | `crates/mycelium-mlir/src/channel.rs:91` | The number of successful channel sends + recvs across this network so far — monotone, |
| `mycelium_mlir::channel::Network::new` | fn | `crates/mycelium-mlir/src/channel.rs:82` | A fresh network with its progress clock at zero. |
| `mycelium_mlir::compile` | fn | `crates/mycelium-mlir/src/dialect/native.rs:694` | Compile `node` through the MLIR pipeline to a native executable (MLIR → LLVM IR → `clang`) |
| `mycelium_mlir::compile_and_run` | fn | `crates/mycelium-mlir/src/dialect/native.rs:724` | Compile + run `node` through the MLIR pipeline and read the result back. |
| `mycelium_mlir::compile_bitnet_dot` | fn | `crates/mycelium-mlir/src/bitnet.rs:408` | Compile the **I2_S** BitNet dot kernel to a shared object and load it in-process. |
| `mycelium_mlir::compile_bitnet_dot_for` | fn | `crates/mycelium-mlir/src/bitnet.rs:415` | Compile the BitNet dot kernel for `scheme` to a shared object and load it in-process. |
| `mycelium_mlir::compile_bitnet_dot_simd` | fn | `crates/mycelium-mlir/src/simd.rs:131` | Compile the hand-vectorized I2_S BitNet dot kernel to a shared object and load it in-process, |
| `mycelium_mlir::compile_bitnet_dot_simd_tl1` | fn | `crates/mycelium-mlir/src/simd.rs:248` | Compile the hand-vectorized TL1 BitNet dot kernel to a shared object and load it in-process, |
| `mycelium_mlir::compile_bitnet_dot_simd_tl2` | fn | `crates/mycelium-mlir/src/simd.rs:595` | Compile the hand-vectorized TL2 BitNet dot kernel to a shared object and load it in-process, |
| `mycelium_mlir::compile_so` | fn | `crates/mycelium-mlir/src/jit.rs:322` | Compile the bit/trit-subset program to a shared object without calling it. |
| `mycelium_mlir::compile_specialized_dot` | fn | `crates/mycelium-mlir/src/specialize.rs:168` | Compile a kernel **specialized on `weights`** (baked in as constants) to a shared object and load |
| `mycelium_mlir::cross_backend_gate` | fn | `crates/mycelium-mlir/src/vr4.rs:195` | Run the **VR-4 cross-backend gate** over `node`: for every backend, produce its dumpable lowering |
| `mycelium_mlir::default_depth_budget` | fn | `crates/mycelium-mlir/src/aot.rs:64` | The default depth-budget resolution — the resolved ceiling **and** its `EXPLAIN`-able basis (no |
| `mycelium_mlir::deploy` | mod | `crates/mycelium-mlir/src/lib.rs:48` | — |
| `mycelium_mlir::deploy::NativeArtifact::build` | fn | `crates/mycelium-mlir/src/deploy.rs:128` | Build the native-artifact descriptor for `node`. |
| `mycelium_mlir::deploy::NativeArtifact::explain` | fn | `crates/mycelium-mlir/src/deploy.rs:192` | A human-readable `EXPLAIN` of the deployable artifact: its content identity, the carried-IR |
| `mycelium_mlir::deploy::NativeArtifact::faithfulness` | fn | `crates/mycelium-mlir/src/deploy.rs:175` | The honest faithfulness strength — `Empirical` (the differentials), never `Proven` (VR-5). |
| `mycelium_mlir::deploy::NativeArtifact::id` | fn | `crates/mycelium-mlir/src/deploy.rs:157` | The canonical content-addressed identity (the program's hash; ADR-003). |
| `mycelium_mlir::deploy::NativeArtifact::lowered_ir` | fn | `crates/mycelium-mlir/src/deploy.rs:163` | The dumpable lowered LLVM IR carried into the deployment (VR-4 evidence). |
| `mycelium_mlir::deploy::NativeArtifact::same_identity_as` | fn | `crates/mycelium-mlir/src/deploy.rs:184` | Whether two artifacts have the **same content-addressed identity** (ADR-003) — i.e. |
| `mycelium_mlir::deploy::NativeArtifact::vr4` | fn | `crates/mycelium-mlir/src/deploy.rs:169` | The VR-4 cross-backend attestation travelling with the deployed unit (no opaque pass anywhere). |
| `mycelium_mlir::dialect` | mod | `crates/mycelium-mlir/src/lib.rs:49` | — |
| `mycelium_mlir::emit` | fn | `crates/mycelium-mlir/src/dialect.rs:75` | Emit the textual `ternary`-dialect module for `node` (one op per lowered binding). |
| `mycelium_mlir::emit_bitnet_dot_ir` | fn | `crates/mycelium-mlir/src/bitnet.rs:147` | Emit the textual LLVM IR for the **I2_S** packed-ternary dot kernel — the default scheme. |
| `mycelium_mlir::emit_bitnet_dot_ir_for` | fn | `crates/mycelium-mlir/src/bitnet.rs:159` | Emit the textual LLVM IR for the packed-ternary dot kernel |
| `mycelium_mlir::emit_bitnet_dot_simd_ir` | fn | `crates/mycelium-mlir/src/simd.rs:59` | Emit the textual LLVM IR for the **hand-vectorized I2_S** packed-ternary dot kernel |
| `mycelium_mlir::emit_bitnet_dot_simd_tl1_ir` | fn | `crates/mycelium-mlir/src/simd.rs:167` | Emit the textual LLVM IR for the **hand-vectorized TL1** packed-ternary dot kernel |
| `mycelium_mlir::emit_bitnet_dot_simd_tl2_ir` | fn | `crates/mycelium-mlir/src/simd.rs:300` | Emit the textual LLVM IR for the **hand-vectorized TL2** packed-ternary dot kernel |
| `mycelium_mlir::emit_llvm_ir` | fn | `crates/mycelium-mlir/src/llvm.rs:1924` | Emit textual LLVM IR for the bit/trit + non-recursive-data program `node` — a `main` that |
| `mycelium_mlir::emit_specialized_dot_ir` | fn | `crates/mycelium-mlir/src/specialize.rs:57` | Emit the textual LLVM IR for a **weight-specialized** ternary dot kernel |
| `mycelium_mlir::inject` | mod | `crates/mycelium-mlir/src/lib.rs:50` | — |
| `mycelium_mlir::inject::Image::call` | fn | `crates/mycelium-mlir/src/inject.rs:185` | Dispatch a call by content hash (ADR-016's call ABI, nullary-unit restriction). |
| `mycelium_mlir::inject::Image::define` | fn | `crates/mycelium-mlir/src/inject.rs:135` | Register a definition **interpret-only** under its content hash (RFC-0001 §4.6), returning the |
| `mycelium_mlir::inject::Image::defined_count` | fn | `crates/mycelium-mlir/src/inject.rs:210` | The number of known (interpretable) definitions. |
| `mycelium_mlir::inject::Image::inject` | fn | `crates/mycelium-mlir/src/inject.rs:153` | **Inject** a recompiled definition: compile its unit (the `dlopen` JIT) and register a |
| `mycelium_mlir::inject::Image::injected_count` | fn | `crates/mycelium-mlir/src/inject.rs:204` | The number of compiled (injected) entries — the dispatch table never shrinks on a re-inject |
| `mycelium_mlir::inject::Image::is_injected` | fn | `crates/mycelium-mlir/src/inject.rs:197` | Whether a compiled (injected) entry exists for `hash`. |
| `mycelium_mlir::inject::Image::new` | fn | `crates/mycelium-mlir/src/inject.rs:118` | An empty image with the default reference interpreter. |
| `mycelium_mlir::inject::Image::with_interpreter` | fn | `crates/mycelium-mlir/src/inject.rs:125` | Build an image with a specific interpreter for the fallback path (e.g. |
| `mycelium_mlir::jit` | mod | `crates/mycelium-mlir/src/lib.rs:51` | — |
| `mycelium_mlir::jit::JitArtifact::call` | fn | `crates/mycelium-mlir/src/jit.rs:113` | Call the kernel in-process (`dlopen` → `dlsym` → call) and read the result back as an `Exact` |
| `mycelium_mlir::jit_run` | fn | `crates/mycelium-mlir/src/jit.rs:346` | Compile the program to a shared object and call it once, in-process. |
| `mycelium_mlir::jit_specialized_dot` | fn | `crates/mycelium-mlir/src/specialize.rs:205` | Convenience: specialize on `weights`, compile, and run the dot product against `activations` once. |
| `mycelium_mlir::jit_ternary_dot` | fn | `crates/mycelium-mlir/src/bitnet.rs:445` | Convenience: pack `weights` under [`KERNEL_SCHEME`] (I2_S), compile the kernel, and run the dot |
| `mycelium_mlir::jit_ternary_dot_for` | fn | `crates/mycelium-mlir/src/bitnet.rs:451` | As [`jit_ternary_dot`], but for an explicit `scheme` — packs `weights` under `scheme` and runs |
| `mycelium_mlir::llvm` | mod | `crates/mycelium-mlir/src/lib.rs:52` | — |
| `mycelium_mlir::llvm::CompiledArtifact::run` | fn | `crates/mycelium-mlir/src/llvm.rs:2420` | Execute the compiled artifact and read its result back as an `Exact` `Binary{w}`/`Ternary{m}` |
| `mycelium_mlir::llvm::compile` | fn | `crates/mycelium-mlir/src/llvm.rs:2454` | Compile the bit/trit-subset program to a native executable (emit LLVM IR → `llc` → `clang`) |
| `mycelium_mlir::llvm::compile_and_run` | fn | `crates/mycelium-mlir/src/llvm.rs:2480` | Compile the bit/trit-subset program to a native executable, run it once, and read the result |
| `mycelium_mlir::pack` | mod | `crates/mycelium-mlir/src/lib.rs:53` | — |
| `mycelium_mlir::pack::PackError` | enum | `crates/mycelium-mlir/src/pack.rs:47` | A packing-codec error. |
| `mycelium_mlir::pack::needed_bytes` | fn | `crates/mycelium-mlir/src/pack.rs:96` | Bytes required to hold `count` trits under `scheme` — the buffer-bound model. |
| `mycelium_mlir::pack_trits` | fn | `crates/mycelium-mlir/src/pack.rs:201` | Encode `trits` to bytes under `scheme` (bijective; the AOT path's physical buffer). |
| `mycelium_mlir::recompile_closure` | fn | `crates/mycelium-mlir/src/inject.rs:226` | The **recompile set** of a change, by hash reachability (ADR-017 decision 3 — no AST/file diff). |
| `mycelium_mlir::relayout_trits` | fn | `crates/mycelium-mlir/src/pack.rs:291` | Re-materialize trits through a pack-then-read round-trip where the buffer is **packed as** |
| `mycelium_mlir::run` | fn | `crates/mycelium-mlir/src/aot.rs:213` | Run a Core IR program through the AOT path to a representation [`Value`]. |
| `mycelium_mlir::run_colony` | fn | `crates/mycelium-mlir/src/runtime.rs:428` | Run an L1 `colony { hypha e1, …, hypha eN }` as **real concurrent execution**, validated equal to |
| `mycelium_mlir::run_core` | fn | `crates/mycelium-mlir/src/aot.rs:147` | Run a Core IR program through the AOT path to a [`CoreValue`] (the full v0 calculus — repr, data, |
| `mycelium_mlir::run_core_with_effects` | fn | `crates/mycelium-mlir/src/aot.rs:196` | [`run_core_with_budget`] with a shared **effect-budget ledger** threaded through the env-machine |
| `mycelium_mlir::run_core_with_fuel` | fn | `crates/mycelium-mlir/src/aot.rs:156` | [`run_core`] with an explicit `Fix`-unfold (fuel) budget and the dynamically-resolved depth ceiling. |
| `mycelium_mlir::run_with_layout` | fn | `crates/mycelium-mlir/src/aot.rs:547` | Run a Core IR program through the AOT path **with a schedule-staged packing layout** (M-251; |
| `mycelium_mlir::runtime` | mod | `crates/mycelium-mlir/src/lib.rs:54` | — |
| `mycelium_mlir::simd` | mod | `crates/mycelium-mlir/src/lib.rs:55` | — |
| `mycelium_mlir::specialize` | mod | `crates/mycelium-mlir/src/lib.rs:56` | — |
| `mycelium_mlir::specialize::BoundSpecializedDot` | struct | `crates/mycelium-mlir/src/specialize.rs:139` | A [`SpecializedDotKernel`] with its entry point resolved into a lifetime-bound `Sym` (M-682). |
| `mycelium_mlir::specialize::SpecializedDotKernel::bind` | fn | `crates/mycelium-mlir/src/specialize.rs:119` | **Bind once, call many** (M-682): resolve the `myc_bitnet_dot_spec` entry point a single time |
| `mycelium_mlir::specialize::SpecializedDotKernel::call` | fn | `crates/mycelium-mlir/src/specialize.rs:130` | Run the specialized kernel over `activations`, returning `Σ digit(wᵢ)·activations[i]` for the |
| `mycelium_mlir::specialize::SpecializedDotKernel::n` | fn | `crates/mycelium-mlir/src/specialize.rs:104` | The logical number of lanes (weight length) compiled into this kernel. |
| `mycelium_mlir::specialize::SpecializedDotKernel::nonzero` | fn | `crates/mycelium-mlir/src/specialize.rs:111` | The number of nonzero (surviving) lanes — the straight-line `add`/`sub` count, exposed for |
| `mycelium_mlir::ternary_dot_ref` | fn | `crates/mycelium-mlir/src/bitnet.rs:135` | The reference (oracle) ternary dot product `Σ digit(wᵢ)·xᵢ` over `i64`, the exact semantics the |
| `mycelium_mlir::unpack_trits` | fn | `crates/mycelium-mlir/src/pack.rs:243` | Decode `count` trits from `bytes` under `scheme`. |
| `mycelium_mlir::vr4` | mod | `crates/mycelium-mlir/src/lib.rs:57` | — |
| `mycelium_mlir::vr4::Backend::all` | fn | `crates/mycelium-mlir/src/vr4.rs:67` | All backends, in lowering order — the exhaustive set the VR-4 gate must cover. |
| `mycelium_mlir::vr4::Backend::name` | fn | `crates/mycelium-mlir/src/vr4.rs:80` | A stable human-readable name (for `EXPLAIN` / reports). |
| `mycelium_mlir::vr4::BackendStage::explain` | fn | `crates/mycelium-mlir/src/vr4.rs:127` | A short `EXPLAIN` line: backend, covered/skipped, the faithfulness tag, and the dump size / |
| `mycelium_mlir::vr4::CrossBackendGate::covered` | fn | `crates/mycelium-mlir/src/vr4.rs:164` | How many backends produced an actual dump (vs. |
| `mycelium_mlir::vr4::CrossBackendGate::fully_covered` | fn | `crates/mycelium-mlir/src/vr4.rs:156` | Whether **every** backend produced a dumpable stage (no skips) — the strongest gate verdict. |
| `mycelium_mlir::vr4::StageStatus::is_dumped` | fn | `crates/mycelium-mlir/src/vr4.rs:105` | Whether this stage was actually dumped (covered), vs. |

## mycelium-numerics

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_numerics::AffineForm` | struct | `crates/mycelium-numerics/src/error.rs:46` | An affine form `x₀ + Σ xᵢ·εᵢ` with noise symbols `εᵢ ∈ [−1, +1]` (ADR-010 §1). |
| `mycelium_numerics::ApRhlJudgment` | struct | `crates/mycelium-numerics/src/prob.rs:69` | An apRHL `⟨ε, δ⟩` relational judgment (ADR-010 §2): "the implementation refines the reference up |
| `mycelium_numerics::Certificate` | struct | `crates/mycelium-numerics/src/cert.rs:159` | The shared certificate both kernels reduce to (ADR-010 §3): an ε side, a δ side, and a `strength` |
| `mycelium_numerics::CheckOutcome` | enum | `crates/mycelium-numerics/src/cert.rs:91` | The verdict of a tier-i re-validation (ADR-010 "Trusted base"). |
| `mycelium_numerics::ComposedBound` | struct | `crates/mycelium-numerics/src/cert.rs:244` | A bound composed by the kernel, with the honest `strength` its inputs' bases justify — the |
| `mycelium_numerics::ErrorBound` | struct | `crates/mycelium-numerics/src/error.rs:221` | A scalar ε-magnitude bound `{eps ≥ 0, norm}` — the [`AffineForm::radius`] projection that travels |
| `mycelium_numerics::ErrorOp` | enum | `crates/mycelium-numerics/src/cert.rs:42` | The error-kernel operation a composition records — re-evaluated by the tier-i checker and used by |
| `mycelium_numerics::NoiseSym` | type | `crates/mycelium-numerics/src/error.rs:33` | A noise-symbol identifier. |
| `mycelium_numerics::ProbBound` | struct | `crates/mycelium-numerics/src/prob.rs:16` | A scalar failure-probability bound `δ ∈ [0, 1]` — travels in a [`mycelium_core::Bound`] |
| `mycelium_numerics::accuracy_to_probability` | fn | `crates/mycelium-numerics/src/cert.rs:148` | The single sanctioned **cross-kernel inference** (ADR-010 §4): an [`ErrorBound`] feeds a |
| `mycelium_numerics::basis_strength` | fn | `crates/mycelium-numerics/src/cert.rs:255` | The strength implied by a bound's basis (M-I2/M-I3/M-I4): the basis *is* the evidence class. |
| `mycelium_numerics::cert` | mod | `crates/mycelium-numerics/src/lib.rs:25` | — |
| `mycelium_numerics::cert::Certificate::delta` | fn | `crates/mycelium-numerics/src/cert.rs:190` | The δ failure-probability side (`0` if no probabilistic component). |
| `mycelium_numerics::cert::Certificate::eps` | fn | `crates/mycelium-numerics/src/cert.rs:185` | The ε-magnitude side (`0` if no error component). |
| `mycelium_numerics::cert::Certificate::from_error` | fn | `crates/mycelium-numerics/src/cert.rs:222` | Lift an [`ErrorBound`] to a certificate at the given `strength` (δ side `0`). |
| `mycelium_numerics::cert::Certificate::from_prob` | fn | `crates/mycelium-numerics/src/cert.rs:232` | Lift a [`ProbBound`] to a certificate at the given `strength` (ε side `0`). |
| `mycelium_numerics::cert::Certificate::new` | fn | `crates/mycelium-numerics/src/cert.rs:201` | A well-formed certificate, or `None` if `eps`/`delta` are out of range (never silent). |
| `mycelium_numerics::cert::Certificate::strength` | fn | `crates/mycelium-numerics/src/cert.rs:195` | The honest guarantee strength (`meet` of contributors). |
| `mycelium_numerics::check_error_claim` | fn | `crates/mycelium-numerics/src/cert.rs:109` | Re-validate a claimed ε bound for `op` over `inputs`: **Valid** iff the claim is `≥` the |
| `mycelium_numerics::check_union_claim` | fn | `crates/mycelium-numerics/src/cert.rs:129` | Re-validate a claimed δ against the **union bound** of `inputs`: **Valid** iff the claim is `≥` |
| `mycelium_numerics::compose_error_bound` | fn | `crates/mycelium-numerics/src/cert.rs:322` | Compose the **`Error` bounds of approximate inputs** under `op` into a result bound whose |
| `mycelium_numerics::error` | mod | `crates/mycelium-numerics/src/lib.rs:26` | — |
| `mycelium_numerics::error::AffineForm::add` | fn | `crates/mycelium-numerics/src/error.rs:140` | Addition — *exact* on the form's structure (shared noise symbols combine, so correlated |
| `mycelium_numerics::error::AffineForm::center` | fn | `crates/mycelium-numerics/src/error.rs:86` | The central value `x₀`. |
| `mycelium_numerics::error::AffineForm::constant` | fn | `crates/mycelium-numerics/src/error.rs:56` | The exact constant `c` (no uncertainty; `radius == 0`). |
| `mycelium_numerics::error::AffineForm::eval` | fn | `crates/mycelium-numerics/src/error.rs:104` | Evaluate the form at a noise assignment `ε(sym) ∈ [−1, +1]`. |
| `mycelium_numerics::error::AffineForm::mul` | fn | `crates/mycelium-numerics/src/error.rs:186` | Multiplication — *nonlinear*. |
| `mycelium_numerics::error::AffineForm::neg` | fn | `crates/mycelium-numerics/src/error.rs:128` | Negation — exact (`−x̂`). |
| `mycelium_numerics::error::AffineForm::radius` | fn | `crates/mycelium-numerics/src/error.rs:94` | The total deviation `radius = Σ\|xᵢ\|` — the sound ε on `\|value − center\|` (ADR-010 §1). |
| `mycelium_numerics::error::AffineForm::scale` | fn | `crates/mycelium-numerics/src/error.rs:166` | Scaling by a constant `c` (`c·x̂`), with the round-off of the center and each scaled |
| `mycelium_numerics::error::AffineForm::sub` | fn | `crates/mycelium-numerics/src/error.rs:158` | Subtraction — exact (`x̂ − ŷ`); `x̂ − x̂ == 0` with `radius 0` (the correlation advantage). |
| `mycelium_numerics::error::AffineForm::uncertain` | fn | `crates/mycelium-numerics/src/error.rs:69` | `center ± radius` carried on a single noise symbol `sym`, or `None` if `center` is non-finite, |
| `mycelium_numerics::error::ErrorBound::eps` | fn | `crates/mycelium-numerics/src/error.rs:232` | The error magnitude (`>= 0`, finite). |
| `mycelium_numerics::error::ErrorBound::new` | fn | `crates/mycelium-numerics/src/error.rs:250` | A well-formed bound, or `None` if `eps` is negative or non-finite (never a silent coercion). |
| `mycelium_numerics::error::ErrorBound::norm` | fn | `crates/mycelium-numerics/src/error.rs:238` | The norm `eps` is measured in. |
| `mycelium_numerics::error::ROUNDOFF_SYM:` | const | `crates/mycelium-numerics/src/error.rs:40` | The reserved noise symbol carrying the **accumulated floating-point round-off of the affine |
| `mycelium_numerics::error_norm` | fn | `crates/mycelium-numerics/src/cert.rs:355` | The norm of a `BoundKind::Error`, for callers assembling [`ErrorOp`]s. |
| `mycelium_numerics::prob` | mod | `crates/mycelium-numerics/src/lib.rs:27` | — |
| `mycelium_numerics::prob::ApRhlJudgment::eps` | fn | `crates/mycelium-numerics/src/prob.rs:77` | The log privacy factor `ε ≥ 0` (the factor is `e^ε`). |
| `mycelium_numerics::prob::ApRhlJudgment::seq` | fn | `crates/mycelium-numerics/src/prob.rs:98` | The apRHL **`[SEQ]`** rule: sequencing two relational steps composes **multiplicatively in the |
| `mycelium_numerics::prob::ProbBound::delta` | fn | `crates/mycelium-numerics/src/prob.rs:24` | Failure probability, always in `[0, 1]`. |
| `mycelium_numerics::prob::ProbBound::new` | fn | `crates/mycelium-numerics/src/prob.rs:36` | A well-formed bound, or `None` if `delta ∉ [0, 1]` or is non-finite (never silent). |
| `mycelium_numerics::prob::ProbBound::or` | fn | `crates/mycelium-numerics/src/prob.rs:60` | Combine with another failure mode by the union bound — the binary form of union. |
| `mycelium_numerics::prob::ProbBound::union` | fn | `crates/mycelium-numerics/src/prob.rs:45` | The **union bound**: `P(⋃ Eᵢ) ≤ min(1, Σ δᵢ)` (ADR-010 §2). |
| `mycelium_numerics::recompute_error` | fn | `crates/mycelium-numerics/src/cert.rs:64` | Re-derive the composed [`ErrorBound`] of `inputs` under `op` from the kernel — the checker's and |

## mycelium-proj

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_proj::Dependency` | struct | `crates/mycelium-proj/src/manifest.rs:80` | One `[dependencies]` entry (M-368): another phylum, **content-addressed** (ADR-003) — pinned by |
| `mycelium_proj::Deprecated` | enum | `crates/mycelium-proj/src/header.rs:31` | A `@deprecated` value: a bare flag or a reason string (spec §3). |
| `mycelium_proj::HEADER_KEYS:` | const | `crates/mycelium-proj/src/header.rs:16` | The closed v0 metadata key set (spec §7.3). |
| `mycelium_proj::HeaderError` | struct | `crates/mycelium-proj/src/header.rs:74` | An explicit header error (G2): a malformed marker, an unknown/duplicate key, or a bad value. |
| `mycelium_proj::HeaderFields` | struct | `crates/mycelium-proj/src/header.rs:40` | The parsed `@key` metadata of a header (all optional; absent fields inherit per the resolver). |
| `mycelium_proj::Manifest` | struct | `crates/mycelium-proj/src/manifest.rs:102` | A parsed `mycelium-proj.toml` (v0: the typed `[project]` table + the optional `[toolchain]`, |
| `mycelium_proj::ManifestError` | struct | `crates/mycelium-proj/src/manifest.rs:117` | An explicit manifest error (G2): a syntax error, an out-of-subset construct, or a bad value. |
| `mycelium_proj::Origin` | enum | `crates/mycelium-proj/src/resolve.rs:22` | Where a resolved field's value came from. |
| `mycelium_proj::Project` | struct | `crates/mycelium-proj/src/manifest.rs:33` | The typed `[project]` table (the v0 closed key set). |
| `mycelium_proj::ProjectKind` | enum | `crates/mycelium-proj/src/manifest.rs:22` | The shape of a Mycelium project (spec §2 — `[project].kind`). |
| `mycelium_proj::Resolved` | struct | `crates/mycelium-proj/src/resolve.rs:40` | A resolved field: its effective value and where it came from. |
| `mycelium_proj::ResolvedHeader` | struct | `crates/mycelium-proj/src/resolve.rs:49` | The fully-resolved header — each inherited field annotated with its [`Origin`]. |
| `mycelium_proj::SporeConfig` | struct | `crates/mycelium-proj/src/manifest.rs:94` | The typed `[spore]` table (M-368): how the project publishes as a deployable (ADR-013). |
| `mycelium_proj::StructuredHeader` | struct | `crates/mycelium-proj/src/header.rs:65` | A parsed structured header: the `// nodule:` marker plus its `@key` metadata. |
| `mycelium_proj::Surface` | struct | `crates/mycelium-proj/src/manifest.rs:71` | The typed `[surface]` table (M-368): a phylum's **public exports** — the germination boundary. |
| `mycelium_proj::Toolchain` | struct | `crates/mycelium-proj/src/manifest.rs:60` | The typed `[toolchain]` table (M-364): the optional pins the toolchain reads. |
| `mycelium_proj::explain` | fn | `crates/mycelium-proj/src/resolve.rs:145` | The `EXPLAIN` of a resolved header — every field with its value and source, so nothing about the |
| `mycelium_proj::header` | mod | `crates/mycelium-proj/src/lib.rs:18` | — |
| `mycelium_proj::header::KNOWN_SPDX:` | const | `crates/mycelium-proj/src/header.rs:326` | The v0 known-SPDX subset — common OSI/FSF identifiers. |
| `mycelium_proj::header::is_iso_date` | fn | `crates/mycelium-proj/src/header.rs:281` | A `YYYY-MM-DD` ISO-8601 calendar date with a plausible month/day (cheap, dependency-free; the |
| `mycelium_proj::header::is_semver` | fn | `crates/mycelium-proj/src/header.rs:303` | A `MAJOR.MINOR.PATCH` semver core, with an optional `-prerelease` and/or `+build` suffix. |
| `mycelium_proj::header::is_spdx` | fn | `crates/mycelium-proj/src/header.rs:354` | A recognised SPDX identifier or a simple expression over [`KNOWN_SPDX`] (operators `OR`/`AND`/ |
| `mycelium_proj::header::is_url` | fn | `crates/mycelium-proj/src/header.rs:315` | A non-empty, URL-shaped string (scheme-prefixed or `git@`-style). |
| `mycelium_proj::manifest` | mod | `crates/mycelium-proj/src/lib.rs:19` | — |
| `mycelium_proj::parse_header` | fn | `crates/mycelium-proj/src/header.rs:107` | Parse the structured header from `src`. |
| `mycelium_proj::parse_manifest` | fn | `crates/mycelium-proj/src/manifest.rs:160` | Parse a `mycelium-proj.toml` source into a [`Manifest`]. |
| `mycelium_proj::resolve` | mod | `crates/mycelium-proj/src/lib.rs:20` | — |
| `mycelium_proj::resolve` | fn | `crates/mycelium-proj/src/resolve.rs:76` | Resolve a parsed header against an optional project manifest. |

## mycelium-sec

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_sec::Finding` | struct | `crates/mycelium-sec/src/lib.rs:61` | A security finding — always cites *why* (G2). |
| `mycelium_sec::Severity` | enum | `crates/mycelium-sec/src/lib.rs:19` | A finding's severity — a **fixed, declared** map (looked up, never heuristically scored; VR-5). |
| `mycelium_sec::Severity::as_str` | fn | `crates/mycelium-sec/src/lib.rs:35` | The canonical label. |
| `mycelium_sec::WildAudit` | struct | `crates/mycelium-sec/src/lib.rs:76` | The `wild`-audit result over a set of sources: the full inventory + the (unjustified) findings. |
| `mycelium_sec::WildAudit::justified` | fn | `crates/mycelium-sec/src/lib.rs:86` | How many blocks are justified. |
| `mycelium_sec::WildAudit::unjustified` | fn | `crates/mycelium-sec/src/lib.rs:91` | How many are unjustified. |
| `mycelium_sec::WildBlock` | struct | `crates/mycelium-sec/src/lib.rs:48` | One `wild` block found by the audit — located, and justified-or-not. |
| `mycelium_sec::audit_wild` | fn | `crates/mycelium-sec/src/lib.rs:98` | Audit a set of `(file, contents)` sources for `wild` blocks (LR-9/S6). |
| `mycelium_sec::collect_myc` | fn | `crates/mycelium-sec/src/lib.rs:229` | Collect every `.myc` under `dir` (recursively, sorted); skipping hidden entries and `target/`. |
| `mycelium_sec::explain_wild` | fn | `crates/mycelium-sec/src/lib.rs:198` | Render the `wild`-audit `EXPLAIN` (no black box): the inventory + each unjustified finding's *why*. |

## mycelium-select

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_select::Action` | enum | `crates/mycelium-select/src/lib.rs:256` | What a matched rule does. |
| `mycelium_select::BITNET_PACKINGS:` | const | `crates/mycelium-select/src/lib.rs:788` | The fixed **bitnet.cpp** ternary packing candidate set (RFC-0004 §5; Wang et al.): `I2_S` |
| `mycelium_select::Candidate` | enum | `crates/mycelium-select/src/lib.rs:245` | A selectable candidate — the three RFC-0005 §4 sites share one vocabulary (one mechanism). |
| `mycelium_select::CandidateCost` | struct | `crates/mycelium-select/src/lib.rs:509` | The per-candidate cost line of an [`Explanation`]. |
| `mycelium_select::CostModel` | struct | `crates/mycelium-select/src/lib.rs:285` | The **explicit cost function** (RFC-0005 §2.1): cost = `storage_weight ×` the candidate's |
| `mycelium_select::CostModel::cost` | fn | `crates/mycelium-select/src/lib.rs:336` | The deterministic cost of `candidate` given `inputs` — total, finite for every well-formed |
| `mycelium_select::DecodeFacts` | struct | `crates/mycelium-select/src/lib.rs:76` | The **exact decode facts** the RFC-0010 decode site queries — generic integers/booleans about a |
| `mycelium_select::DecodeMethod` | enum | `crates/mycelium-select/src/lib.rs:232` | A decode methodology — the **third** RFC-0005 §4 site (RFC-0010): how a value is decoded back to |
| `mycelium_select::Explanation` | struct | `crates/mycelium-select/src/lib.rs:520` | The **mandatory EXPLAIN record** (M-221; RFC-0005 §2.2/§4): emitted on *every* selection — |
| `mycelium_select::ParadigmKind` | enum | `crates/mycelium-select/src/lib.rs:50` | The four closed paradigm kinds, as a predicate-level discriminator (RFC-0001 §4.1). |
| `mycelium_select::PolicyError` | enum | `crates/mycelium-select/src/lib.rs:357` | Why a policy could not be constructed — validated up front so every constructed policy is |
| `mycelium_select::PolicyRegistry` | struct | `crates/mycelium-select/src/lib.rs:732` | A registry resolving a recorded `PolicyRef` back to the policy that decided — the operational |
| `mycelium_select::PolicyRegistry::get` | fn | `crates/mycelium-select/src/lib.rs:752` | Resolve a `PolicyRef` to its policy, if registered. |
| `mycelium_select::PolicyRegistry::is_empty` | fn | `crates/mycelium-select/src/lib.rs:764` | Whether the registry is empty. |
| `mycelium_select::PolicyRegistry::len` | fn | `crates/mycelium-select/src/lib.rs:758` | Number of registered policies. |
| `mycelium_select::PolicyRegistry::new` | fn | `crates/mycelium-select/src/lib.rs:431` | Build a policy, validating totality up front: at least one candidate, every `Choose(i)` |
| `mycelium_select::PolicyRegistry::register` | fn | `crates/mycelium-select/src/lib.rs:744` | Register a policy under its content address; returns the `PolicyRef`. |
| `mycelium_select::Predicate` | enum | `crates/mycelium-select/src/lib.rs:138` | The predicate language — small, closed, **not Turing-complete**: no loops, no recursion in the |
| `mycelium_select::Predicate::eval` | fn | `crates/mycelium-select/src/lib.rs:172` | Evaluate against the queryable inputs — total: every predicate yields a boolean on every |
| `mycelium_select::Predicate::literals_finite` | fn | `crates/mycelium-select/src/lib.rs:210` | True iff every floating-point literal in the predicate tree is finite (A5-01/B2-02). |
| `mycelium_select::Rule` | struct | `crates/mycelium-select/src/lib.rs:266` | One row of the decision table: `when` (a [`Predicate`]) → `action`. |
| `mycelium_select::SelectError` | enum | `crates/mycelium-select/src/lib.rs:542` | Why a selection call failed — always explicit (G2), never a silent fallback choice. |
| `mycelium_select::SelectionInputs` | struct | `crates/mycelium-select/src/lib.rs:92` | The **queryable inputs** a policy may inspect — drawn from a value's [`Repr`] + [`Meta`] |
| `mycelium_select::SelectionInputs::from_meta` | fn | `crates/mycelium-select/src/lib.rs:109` | The queryable projection of a `(Repr, Meta)` pair (no decode facts — swap/packing sites). |
| `mycelium_select::SelectionInputs::of_value` | fn | `crates/mycelium-select/src/lib.rs:121` | The queryable projection of a [`Value`]. |
| `mycelium_select::SelectionInputs::with_decode` | fn | `crates/mycelium-select/src/lib.rs:127` | Attach decode-site facts (RFC-0010) for the [`select_decode_method`] adapter. |
| `mycelium_select::SelectionPolicy` | struct | `crates/mycelium-select/src/lib.rs:403` | A **reified selection policy** (ADR-006; RFC-0005 §2/§3): an ordered decision table over a |
| `mycelium_select::SelectionPolicy::candidates` | fn | `crates/mycelium-select/src/lib.rs:476` | The finite candidate set. |
| `mycelium_select::SelectionPolicy::cost_model` | fn | `crates/mycelium-select/src/lib.rs:491` | The explicit cost model. |
| `mycelium_select::SelectionPolicy::default_choice` | fn | `crates/mycelium-select/src/lib.rs:486` | The mandatory default arm (totality). |
| `mycelium_select::SelectionPolicy::name` | fn | `crates/mycelium-select/src/lib.rs:471` | The policy's display name (not part of selection semantics, but part of its identity). |
| `mycelium_select::SelectionPolicy::policy_ref` | fn | `crates/mycelium-select/src/lib.rs:500` | The **content address** of this policy (RFC-0005 §3; RFC-0001 §4.6): the hash of its |
| `mycelium_select::SelectionPolicy::rules` | fn | `crates/mycelium-select/src/lib.rs:481` | The ordered decision table. |
| `mycelium_select::bitnet_packing_policy` | fn | `crates/mycelium-select/src/lib.rs:807` | Build the **default schedule-staged packing policy** (M-250): the three [`BITNET_PACKINGS`] |
| `mycelium_select::explain` | fn | `crates/mycelium-select/src/lib.rs:669` | `explain(policy, meta) → trace` (RFC-0005 §4): the mandatory EXPLAIN, **total and |
| `mycelium_select::layout_of` | fn | `crates/mycelium-select/src/lib.rs:794` | Map a chosen ternary [`PackScheme`] to the [`PhysicalLayout`] recorded on `Meta.physical`. |
| `mycelium_select::record_packing_layout` | fn | `crates/mycelium-select/src/lib.rs:863` | One-call convenience: select the packing layout for a value's `(Repr, Meta)` and **record it** |
| `mycelium_select::select` | fn | `crates/mycelium-select/src/lib.rs:597` | The **single selection entry point** (RFC-0005 §2; one mechanism for both §4 sites): evaluate |
| `mycelium_select::select_decode_method` | fn | `crates/mycelium-select/src/lib.rs:713` | Decode-method site adapter (RFC-0005 §4 site 3; RFC-0010): the chosen candidate must be a |
| `mycelium_select::select_layout` | fn | `crates/mycelium-select/src/lib.rs:842` | The **packing-schedule selector** (M-250; RFC-0004 §5; one mechanism — RFC-0005 §4): evaluate |
| `mycelium_select::select_packing` | fn | `crates/mycelium-select/src/lib.rs:694` | Packing-schedule site adapter (RFC-0005 §4 site 2; RFC-0004 §5 — consumed by E2-7/M-250): the |
| `mycelium_select::select_swap_target` | fn | `crates/mycelium-select/src/lib.rs:677` | Swap-target site adapter (RFC-0005 §4 site 1; RFC-0002): the chosen candidate must be a |

## mycelium-spore

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_spore::ResolvedDep` | struct | `crates/mycelium-spore/src/lib.rs:33` | A resolved dependency edge — pinned by content hash (authoritative, ADR-003). |
| `mycelium_spore::SourceFile` | struct | `crates/mycelium-spore/src/lib.rs:24` | A project source file, content-addressed (raw-byte BLAKE3; ADR-003). |
| `mycelium_spore::Spore` | struct | `crates/mycelium-spore/src/lib.rs:47` | A built spore: its content-addressed identity plus the components that define it and the metadata that |
| `mycelium_spore::SporeError` | enum | `crates/mycelium-spore/src/lib.rs:67` | A spore-build refusal — never a partial artifact (G2). |
| `mycelium_spore::SporeError::exit_code` | fn | `crates/mycelium-spore/src/lib.rs:77` | The CLI exit code for this refusal. |
| `mycelium_spore::build_spore` | fn | `crates/mycelium-spore/src/lib.rs:102` | Build a [`Spore`] from a parsed manifest and the project root directory. |
| `mycelium_spore::explain` | fn | `crates/mycelium-spore/src/lib.rs:269` | The `EXPLAIN` of a built spore (no black box): the identity receipt, the surface, the code by hash, the |
| `mycelium_spore::kind_str` | fn | `crates/mycelium-spore/src/lib.rs:218` | The canonical `[project].kind` spelling. |

## mycelium-stack

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_stack::with_deep_stack` | fn | `crates/mycelium-stack/src/lib.rs:61` | Run `f` on a worker thread with a large explicit stack ([`DEEP_STACK_BYTES`]) and return its value. |

## mycelium-std-cmp

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_cmp::Bf16Bits` | struct | `crates/mycelium-std-cmp/src/lib.rs:538` | A BF16 value stored as its bit pattern in a `u16`. |
| `mycelium_std_cmp::Bf16Bits::INFINITY:` | const | `crates/mycelium-std-cmp/src/lib.rs:550` | The BF16 bit-pattern for positive infinity. |
| `mycelium_std_cmp::Bf16Bits::NAN:` | const | `crates/mycelium-std-cmp/src/lib.rs:548` | The BF16 bit-pattern for NaN (a quiet NaN in f32 bit layout). |
| `mycelium_std_cmp::Bf16Bits::NEG_INFINITY:` | const | `crates/mycelium-std-cmp/src/lib.rs:552` | The BF16 bit-pattern for negative infinity. |
| `mycelium_std_cmp::Bf16Bits::NEG_ONE:` | const | `crates/mycelium-std-cmp/src/lib.rs:546` | The BF16 bit-pattern for negative one. |
| `mycelium_std_cmp::Bf16Bits::ONE:` | const | `crates/mycelium-std-cmp/src/lib.rs:544` | The BF16 bit-pattern for positive one. |
| `mycelium_std_cmp::Bf16Bits::ZERO:` | const | `crates/mycelium-std-cmp/src/lib.rs:542` | The BF16 bit-pattern for positive zero. |
| `mycelium_std_cmp::Bf16Bits::to_f32` | fn | `crates/mycelium-std-cmp/src/lib.rs:560` | Widen this BF16 value to an f32 by zero-filling the lower 16 mantissa bits. |
| `mycelium_std_cmp::ClampError` | enum | `crates/mycelium-std-cmp/src/lib.rs:251` | The explicit error set for `clamp` (spec §3). |
| `mycelium_std_cmp::GUARANTEE_MATRIX:` | const | `crates/mycelium-std-cmp/src/lib.rs:845` | The `std.cmp`/`convert` guarantee matrix (spec §4). |
| `mycelium_std_cmp::MatrixRow` | struct | `crates/mycelium-std-cmp/src/lib.rs:823` | One row of the `std.cmp`/`convert` guarantee matrix (RFC-0016 §4.5; spec §4). |
| `mycelium_std_cmp::MycEq` | trait | `crates/mycelium-std-cmp/src/lib.rs:119` | Total equality — respects content-addressed identity where it applies (ADR-003). |
| `mycelium_std_cmp::MycOrd:` | trait | `crates/mycelium-std-cmp/src/lib.rs:133` | Total ordering — for types with a well-defined total order. |
| `mycelium_std_cmp::MycPartialOrd:` | trait | `crates/mycelium-std-cmp/src/lib.rs:168` | Partial ordering — for types where some pairs may be incomparable (e.g. |
| `mycelium_std_cmp::Narrow` | trait | `crates/mycelium-std-cmp/src/lib.rs:363` | Explicitly-fallible narrowing conversion — the value may not fit in the target type. |
| `mycelium_std_cmp::NarrowError` | enum | `crates/mycelium-std-cmp/src/lib.rs:318` | The explicit error set for a narrowing conversion (spec §3 / §4). |
| `mycelium_std_cmp::Ordering` | enum | `crates/mycelium-std-cmp/src/lib.rs:70` | The result of a comparison — Less, Equal, or Greater. |
| `mycelium_std_cmp::Ordering::reverse` | fn | `crates/mycelium-std-cmp/src/lib.rs:82` | Reverse the ordering: `Less ↔ Greater`, `Equal ↔ Equal`. |
| `mycelium_std_cmp::Widen` | trait | `crates/mycelium-std-cmp/src/lib.rs:307` | Lossless widening conversion — the domain is a subset of the codomain by construction. |
| `mycelium_std_cmp::assert_matrix_invariants` | fn | `crates/mycelium-std-cmp/src/lib.rs:924` | Assert the structural invariants of the guarantee matrix — called from tests. |
| `mycelium_std_cmp::myc_clamp` | fn | `crates/mycelium-std-cmp/src/lib.rs:285` | Clamp `x` to `[lo, hi]` under total order. |
| `mycelium_std_cmp::myc_max` | fn | `crates/mycelium-std-cmp/src/lib.rs:238` | Return the maximum of two values under total order. |
| `mycelium_std_cmp::myc_min` | fn | `crates/mycelium-std-cmp/src/lib.rs:226` | Return the minimum of two values under total order. |

## mycelium-std-collections

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_collections::CollErr` | enum | `crates/mycelium-std-collections/src/error.rs:23` | Out-of-bounds or invalid range on a [`crate::Seq`] operation (spec §3 `CollErr`). |
| `mycelium_std_collections::Map` | struct | `crates/mycelium-std-collections/src/map.rs:56` | An immutable persistent key→value map (value-semantic; spec §3). |
| `mycelium_std_collections::Seq` | struct | `crates/mycelium-std-collections/src/seq.rs:36` | An immutable persistent indexed sequence (value-semantic; spec §3). |
| `mycelium_std_collections::Set` | struct | `crates/mycelium-std-collections/src/set.rs:45` | An immutable persistent membership set (value-semantic; spec §3). |
| `mycelium_std_collections::error` | mod | `crates/mycelium-std-collections/src/lib.rs:109` | — |
| `mycelium_std_collections::guarantee_matrix` | mod | `crates/mycelium-std-collections/src/lib.rs:110` | — |
| `mycelium_std_collections::guarantee_matrix::Explainable` | enum | `crates/mycelium-std-collections/src/guarantee_matrix.rs:39` | Whether an op has an EXPLAIN obligation (C3). |
| `mycelium_std_collections::guarantee_matrix::Fallibility` | enum | `crates/mycelium-std-collections/src/guarantee_matrix.rs:30` | Fallibility classification for an exported op. |
| `mycelium_std_collections::guarantee_matrix::MATRIX:` | const | `crates/mycelium-std-collections/src/guarantee_matrix.rs:71` | The `std.collections` guarantee matrix. |
| `mycelium_std_collections::guarantee_matrix::MatrixRow` | struct | `crates/mycelium-std-collections/src/guarantee_matrix.rs:48` | One row in the guarantee matrix (RFC-0016 §4.5). |
| `mycelium_std_collections::map` | mod | `crates/mycelium-std-collections/src/lib.rs:111` | — |
| `mycelium_std_collections::seq` | mod | `crates/mycelium-std-collections/src/lib.rs:112` | — |
| `mycelium_std_collections::set` | mod | `crates/mycelium-std-collections/src/lib.rs:113` | — |

## mycelium-std-content

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_content::ContentRef` | struct | `crates/mycelium-std-content/src/content_ref.rs:48` | A typed, opaque content reference — a `(kind, hash)` pair that cert / policy / provenance / |
| `mycelium_std_content::MalformedDigest` | struct | `crates/mycelium-std-content/src/error.rs:18` | The content-address string is not well-formed (`<algo>:<digest>` shape; RFC-0001 §4.6). |
| `mycelium_std_content::NameRegistry` | struct | `crates/mycelium-std-content/src/name_registry.rs:36` | A read/write `hash ↔ name` registry (RFC-0001 §4.6 "names-as-metadata"). |
| `mycelium_std_content::RefKind` | enum | `crates/mycelium-std-content/src/content_ref.rs:27` | The role a [`ContentRef`] points to (the explicit kind tag). |
| `mycelium_std_content::as_ref` | fn | `crates/mycelium-std-content/src/lib.rs:172` | Build a typed [`ContentRef`] that cert / policy / provenance / `spore` artifacts embed to |
| `mycelium_std_content::content_ref` | mod | `crates/mycelium-std-content/src/lib.rs:77` | — |
| `mycelium_std_content::content_ref::ContentRef::as_str_repr` | fn | `crates/mycelium-std-content/src/content_ref.rs:89` | The canonical string form of this reference: `<kind-prefix>+<algo>:<digest>`. |
| `mycelium_std_content::content_ref::ContentRef::into_hash` | fn | `crates/mycelium-std-content/src/content_ref.rs:76` | Consume the ref, returning the inner [`ContentHash`]. |
| `mycelium_std_content::content_ref::ContentRef::kind` | fn | `crates/mycelium-std-content/src/content_ref.rs:64` | The role this reference designates. |
| `mycelium_std_content::content_ref::ContentRef::new` | fn | `crates/mycelium-std-content/src/content_ref.rs:58` | Build a `ContentRef` from an explicit kind and hash. |
| `mycelium_std_content::content_ref::RefKind::hash` | fn | `crates/mycelium-std-content/src/content_ref.rs:70` | The content-addressed identity this reference points to. |
| `mycelium_std_content::digest_eq` | fn | `crates/mycelium-std-content/src/lib.rs:154` | Identity equality by digest: two content hashes are **the same identity** iff their digests |
| `mycelium_std_content::error` | mod | `crates/mycelium-std-content/src/lib.rs:78` | — |
| `mycelium_std_content::guarantee_matrix` | mod | `crates/mycelium-std-content/src/lib.rs:79` | — |
| `mycelium_std_content::guarantee_matrix::Explainable` | enum | `crates/mycelium-std-content/src/guarantee_matrix.rs:40` | Whether an op has an EXPLAIN obligation (C3). |
| `mycelium_std_content::guarantee_matrix::Fallibility` | enum | `crates/mycelium-std-content/src/guarantee_matrix.rs:31` | Fallibility classification for an exported op. |
| `mycelium_std_content::guarantee_matrix::MATRIX:` | const | `crates/mycelium-std-content/src/guarantee_matrix.rs:71` | The `std.content` guarantee matrix. |
| `mycelium_std_content::guarantee_matrix::MatrixRow` | struct | `crates/mycelium-std-content/src/guarantee_matrix.rs:49` | One row in the guarantee matrix (RFC-0016 §4.5). |
| `mycelium_std_content::hash_of_def` | fn | `crates/mycelium-std-content/src/lib.rs:134` | The content hash of a definition (hash-of-AST; RFC-0001 §4.6 `hash(def)`): |
| `mycelium_std_content::hash_of_value` | fn | `crates/mycelium-std-content/src/lib.rs:112` | The content hash of a runtime *value*: its identity-bearing `Repr` + payload, with all dynamic |
| `mycelium_std_content::name_registry` | mod | `crates/mycelium-std-content/src/lib.rs:80` | — |
| `mycelium_std_content::name_registry::NameRegistry::bind` | fn | `crates/mycelium-std-content/src/name_registry.rs:53` | Bind a human name to a content hash. |
| `mycelium_std_content::name_registry::NameRegistry::is_empty` | fn | `crates/mycelium-std-content/src/name_registry.rs:92` | Whether the registry is empty. |
| `mycelium_std_content::name_registry::NameRegistry::len` | fn | `crates/mycelium-std-content/src/name_registry.rs:86` | Number of names currently bound in the registry. |
| `mycelium_std_content::name_registry::NameRegistry::names_of` | fn | `crates/mycelium-std-content/src/name_registry.rs:77` | All names bound to `hash`, as a list (0 or 1 entries with the current kernel; see module |
| `mycelium_std_content::name_registry::NameRegistry::new` | fn | `crates/mycelium-std-content/src/name_registry.rs:43` | Create an empty registry. |
| `mycelium_std_content::name_registry::NameRegistry::resolve_name` | fn | `crates/mycelium-std-content/src/name_registry.rs:65` | Look up the name bound to `hash`, returning `None` when the name is unbound. |
| `mycelium_std_content::names_of` | fn | `crates/mycelium-std-content/src/lib.rs:250` | All names bound to `hash` in `registry`, as a list (0 or 1 entries with the current kernel; |
| `mycelium_std_content::parse_ref` | fn | `crates/mycelium-std-content/src/lib.rs:190` | Parse a content-address string (`<algo>:<digest>`) into a [`ContentHash`]. |
| `mycelium_std_content::resolve_name` | fn | `crates/mycelium-std-content/src/lib.rs:230` | Look up the name bound to a content hash in `registry`, returning `None` when the name is |

## mycelium-std-core

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_core::GUARANTEE_MATRIX:` | const | `crates/mycelium-std-core/src/lib.rs:139` | The `std.core` guarantee matrix (spec §4). |
| `mycelium_std_core::GuaranteeRow` | struct | `crates/mycelium-std-core/src/lib.rs:121` | One row of the module guarantee matrix (RFC-0016 §4.5): an exported item, its |
| `mycelium_std_core::bound_of` | fn | `crates/mycelium-std-core/src/lib.rs:105` | The bound attached to `v`, or `None` when there is no metadata or no bound. |
| `mycelium_std_core::error_scaffold` | mod | `crates/mycelium-std-core/src/lib.rs:47` | — |
| `mycelium_std_core::error_scaffold::StdError:` | trait | `crates/mycelium-std-core/src/error_scaffold.rs:68` | Marker super-trait: *this value is an honest stdlib error*. |
| `mycelium_std_core::guarantee_of` | fn | `crates/mycelium-std-core/src/lib.rs:99` | The guarantee tag of `v` (total — every value carries one). |
| `mycelium_std_core::meta_of` | fn | `crates/mycelium-std-core/src/lib.rs:93` | The metadata of `v`, or `None` if `v` is algebraic data (no `Meta`). |
| `mycelium_std_core::prelude` | mod | `crates/mycelium-std-core/src/lib.rs:62` | The curated default prelude (spec §3 / FLAG Q1). |
| `mycelium_std_core::provenance_of` | fn | `crates/mycelium-std-core/src/lib.rs:111` | The provenance of `v`, or `None` if `v` is algebraic data (no `Meta`). |
| `mycelium_std_core::repr_of` | fn | `crates/mycelium-std-core/src/lib.rs:87` | The representation of `v`, or `None` if `v` is algebraic data (no `Repr`). |

## mycelium-std-dense

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_dense::ACCUMULATION_BF16_EMPIRICAL_BASIS:` | const | `crates/mycelium-std-dense/src/lib.rs:454` | Empirical basis string for BF16 accumulation ops (FLAG Q1). |
| `mycelium_std_dense::ACCUMULATION_EMPIRICAL_BASIS:` | const | `crates/mycelium-std-dense/src/lib.rs:448` | Empirical basis string for accumulation ops (FLAG Q1). |
| `mycelium_std_dense::GUARANTEE_MATRIX:` | const | `crates/mycelium-std-dense/src/lib.rs:239` | The guarantee matrix for `std.dense` (RFC-0016 §4.5). |
| `mycelium_std_dense::GuaranteeRow` | struct | `crates/mycelium-std-dense/src/lib.rs:217` | One row of the guarantee matrix (§4 of `docs/spec/stdlib/dense.md`). |
| `mycelium_std_dense::OpBound` | struct | `crates/mycelium-std-dense/src/lib.rs:166` | A reified ε-bound artifact: the inspectable record of a float op's accuracy claim (C3/EXPLAIN; |
| `mycelium_std_dense::OpBound::to_core_bound` | fn | `crates/mycelium-std-dense/src/lib.rs:185` | Convert to a [`Bound`] suitable for attaching to a [`mycelium_core::Meta`]. |
| `mycelium_std_dense::SQRT_COMPOSITION_EMPIRICAL_BASIS:` | const | `crates/mycelium-std-dense/src/lib.rs:459` | Empirical basis string for L2-norm / cosine ops (FLAG Q2). |
| `mycelium_std_dense::StdDense` | struct | `crates/mycelium-std-dense/src/lib.rs:492` | The ergonomic Ring-1 capability surface over a typed `Dense{dim, dtype}` space (M-518). |
| `mycelium_std_dense::StdDense::add` | fn | `crates/mycelium-std-dense/src/lib.rs:589` | Elementwise `a + b` — float DT: `Proven` (FLAG Q1 — uses kernel bound). |
| `mycelium_std_dense::StdDense::cosine` | fn | `crates/mycelium-std-dense/src/lib.rs:873` | Cosine similarity — float DT: `Empirical` (FLAG Q2: sqrt + division composition). |
| `mycelium_std_dense::StdDense::dim` | fn | `crates/mycelium-std-dense/src/lib.rs:515` | The dimensionality. |
| `mycelium_std_dense::StdDense::dot` | fn | `crates/mycelium-std-dense/src/lib.rs:845` | Dot product `⟨a, b⟩` — float DT: `Empirical` (FLAG Q1: accumulation bound). |
| `mycelium_std_dense::StdDense::dtype` | fn | `crates/mycelium-std-dense/src/lib.rs:521` | The element dtype. |
| `mycelium_std_dense::StdDense::from_slice` | fn | `crates/mycelium-std-dense/src/lib.rs:560` | Construct a value from a slice, checking length and grid alignment. |
| `mycelium_std_dense::StdDense::full` | fn | `crates/mycelium-std-dense/src/lib.rs:544` | Construct an **Exact** constant vector with every element equal to `x`. |
| `mycelium_std_dense::StdDense::hadamard` | fn | `crates/mycelium-std-dense/src/lib.rs:630` | Elementwise (Hadamard) product `a ⊙ b` — float DT: `Proven` (FLAG Q1). |
| `mycelium_std_dense::StdDense::l1_norm` | fn | `crates/mycelium-std-dense/src/lib.rs:798` | L1 norm (sum of \|xᵢ\|) — float DT: `Empirical` (same accumulation argument as `sum`, |
| `mycelium_std_dense::StdDense::l2_norm` | fn | `crates/mycelium-std-dense/src/lib.rs:818` | L2 (Euclidean) norm — float DT: `Empirical` (FLAG Q2: sqrt composition not fully checked). |
| `mycelium_std_dense::StdDense::map` | fn | `crates/mycelium-std-dense/src/lib.rs:710` | Map a function `f` over every element (tag = meet of input tag and `f_tag` — VR-5). |
| `mycelium_std_dense::StdDense::neg` | fn | `crates/mycelium-std-dense/src/lib.rs:615` | Elementwise negation — **Exact** (the dtype grid is symmetric; no rounding). |
| `mycelium_std_dense::StdDense::new` | fn | `crates/mycelium-std-dense/src/lib.rs:501` | Construct a `StdDense` surface for a `dim`-dimensional space over `dtype`. |
| `mycelium_std_dense::StdDense::scale` | fn | `crates/mycelium-std-dense/src/lib.rs:678` | Scalar multiplication `s · a` — float DT: `Proven` (FLAG Q1). |
| `mycelium_std_dense::StdDense::space` | fn | `crates/mycelium-std-dense/src/lib.rs:509` | The underlying [`DenseSpace`] descriptor. |
| `mycelium_std_dense::StdDense::sub` | fn | `crates/mycelium-std-dense/src/lib.rs:599` | Elementwise `a − b` — same contract as add. |
| `mycelium_std_dense::StdDense::sum` | fn | `crates/mycelium-std-dense/src/lib.rs:777` | Sum all elements — float DT: `Empirical` (FLAG Q1: accumulation bound, conservative |
| `mycelium_std_dense::StdDense::zeros` | fn | `crates/mycelium-std-dense/src/lib.rs:529` | Construct an **Exact** zero vector (guarantee matrix: `zeros` — `Exact`, total). |
| `mycelium_std_dense::StdDenseError` | enum | `crates/mycelium-std-dense/src/lib.rs:92` | Errors from the `std.dense` capability surface (C1/G2: explicit typed errors, never sentinels). |
| `mycelium_std_dense::accumulation_eps_bf16` | fn | `crates/mycelium-std-dense/src/lib.rs:443` | BF16 analogue of [`accumulation_eps_f32`]. |
| `mycelium_std_dense::accumulation_eps_f32` | fn | `crates/mycelium-std-dense/src/lib.rs:436` | Conservative empirical ε for floating-point accumulation ops (`sum`, `l1_norm`, `dot`). |

## mycelium-std-diag

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_diag::guarantee_matrix` | mod | `crates/mycelium-std-diag/src/lib.rs:42` | The §4.5 guarantee matrix — encoded as data, asserted in tests (RFC-0016 §4.5; spec §4). |
| `mycelium_std_diag::guarantee_matrix::Explainable` | enum | `crates/mycelium-std-diag/src/guarantee_matrix.rs:44` | Whether an op exposes a C3 EXPLAIN artifact. |
| `mycelium_std_diag::guarantee_matrix::Fallibility` | enum | `crates/mycelium-std-diag/src/guarantee_matrix.rs:35` | Fallibility classification for a `std.diag` exported op. |
| `mycelium_std_diag::guarantee_matrix::MATRIX:` | const | `crates/mycelium-std-diag/src/guarantee_matrix.rs:81` | The `std.diag` guarantee matrix (spec §4; RFC-0016 §4.5). |
| `mycelium_std_diag::guarantee_matrix::MatrixRow` | struct | `crates/mycelium-std-diag/src/guarantee_matrix.rs:59` | One row in the `std.diag` guarantee matrix (RFC-0016 §4.5; spec §4). |

## mycelium-std-error

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_error::RefusalRecord` | struct | `crates/mycelium-std-error/src/combinators.rs:38` | The refusal record emitted when a named partial accessor (`unwrap`/`expect`/`unwrap_err`) |
| `mycelium_std_error::SubstitutionRecord` | struct | `crates/mycelium-std-error/src/combinators.rs:69` | The substitution record for `unwrap_or` / `unwrap_or_else`: records that a default was |
| `mycelium_std_error::and_then` | fn | `crates/mycelium-std-error/src/combinators.rs:111` | Monadic bind: apply `f` to the `Ok` value; `Err` short-circuits and **propagates** |
| `mycelium_std_error::combinators` | mod | `crates/mycelium-std-error/src/lib.rs:64` | — |
| `mycelium_std_error::expect` | fn | `crates/mycelium-std-error/src/combinators.rs:362` | Extract the `Ok` value with a caller-supplied reason for the expected state. |
| `mycelium_std_error::filter` | fn | `crates/mycelium-std-error/src/combinators.rs:139` | Filter an `Option`: `Some(x)` where `predicate(x)` is `false` becomes `None`. |
| `mycelium_std_error::flatten` | fn | `crates/mycelium-std-error/src/combinators.rs:236` | Flatten `Result<Result<T, E>, E>` to `Result<T, E>`. |
| `mycelium_std_error::guarantee_matrix` | mod | `crates/mycelium-std-error/src/lib.rs:65` | — |
| `mycelium_std_error::guarantee_matrix::Explainable` | enum | `crates/mycelium-std-error/src/guarantee_matrix.rs:51` | Whether an op has a C3 EXPLAIN obligation (selects / converts / approximates). |
| `mycelium_std_error::guarantee_matrix::Fallibility` | enum | `crates/mycelium-std-error/src/guarantee_matrix.rs:36` | Fallibility classification for an exported op. |
| `mycelium_std_error::guarantee_matrix::MATRIX:` | const | `crates/mycelium-std-error/src/guarantee_matrix.rs:94` | The `std.error` guarantee matrix. |
| `mycelium_std_error::guarantee_matrix::MatrixRow` | struct | `crates/mycelium-std-error/src/guarantee_matrix.rs:68` | One row in the `std.error` guarantee matrix (RFC-0016 §4.5; spec §4). |
| `mycelium_std_error::inspect` | fn | `crates/mycelium-std-error/src/combinators.rs:152` | Peek the `Ok` side with an effectful closure; the value and sum shape are **unchanged**. |
| `mycelium_std_error::inspect_err` | fn | `crates/mycelium-std-error/src/combinators.rs:165` | Peek the `Err` side with an effectful closure; the value and propagation are **unchanged**. |
| `mycelium_std_error::map` | fn | `crates/mycelium-std-error/src/combinators.rs:84` | Map the `Ok`-side value; `Err` passes through unchanged (error preserved in the sum). |
| `mycelium_std_error::map_err` | fn | `crates/mycelium-std-error/src/combinators.rs:97` | Map the `Err`-side value; `Ok` passes through unchanged. |
| `mycelium_std_error::ok` | fn | `crates/mycelium-std-error/src/combinators.rs:214` | Convert `Result<T, E>` to `Option<T>`: `Ok(t) → Some(t)`, `Err(e) → None`. |
| `mycelium_std_error::ok_or` | fn | `crates/mycelium-std-error/src/combinators.rs:181` | Convert `Option<T>` to `Result<T, E>` by naming the `None` case: `None → Err(err)`. |
| `mycelium_std_error::ok_or_else` | fn | `crates/mycelium-std-error/src/combinators.rs:192` | Convert `Option<T>` to `Result<T, E>` with a lazily-computed error value. |
| `mycelium_std_error::or_else` | fn | `crates/mycelium-std-error/src/combinators.rs:125` | Explicit recovery hook: apply `f` to the `Err` value; `Ok` passes through. |
| `mycelium_std_error::transpose` | fn | `crates/mycelium-std-error/src/combinators.rs:225` | Transpose `Option<Result<T, E>>` to `Result<Option<T>, E>`. |
| `mycelium_std_error::unwrap` | fn | `crates/mycelium-std-error/src/combinators.rs:335` | Extract the `Ok` value. |
| `mycelium_std_error::unwrap_err` | fn | `crates/mycelium-std-error/src/combinators.rs:387` | Extract the `Err` value: symmetric to `unwrap`. |
| `mycelium_std_error::unwrap_or` | fn | `crates/mycelium-std-error/src/combinators.rs:260` | Recover an `Err`/`None` with an explicitly-supplied default value. |
| `mycelium_std_error::unwrap_or_else` | fn | `crates/mycelium-std-error/src/combinators.rs:276` | Recover an `Err`/`None` with a computed default from a closure. |
| `mycelium_std_error::unwrap_or_else_option` | fn | `crates/mycelium-std-error/src/combinators.rs:307` | Recover an `Option<T>` with a computed default from a closure. |
| `mycelium_std_error::unwrap_or_option` | fn | `crates/mycelium-std-error/src/combinators.rs:293` | Recover an `Option<T>` with an explicitly-supplied default value. |
| `mycelium_std_error::zip` | fn | `crates/mycelium-std-error/src/combinators.rs:246` | Zip two `Option`s: both must be `Some`; either `None` short-circuits to `None`. |

## mycelium-std-fmt

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_fmt::Budget` | struct | `crates/mycelium-std-fmt/src/lib.rs:122` | A budget for `display_bounded`: the maximum number of *elements* (bits, trits, scalars, |
| `mycelium_std_fmt::FromJsonError` | enum | `crates/mycelium-std-fmt/src/lib.rs:187` | Errors the `from_json` machine projection can raise. |
| `mycelium_std_fmt::GUARANTEE_MATRIX:` | const | `crates/mycelium-std-fmt/src/lib.rs:364` | The `std.fmt` guarantee matrix (spec §4 / RFC-0016 §4.5). |
| `mycelium_std_fmt::Json` | struct | `crates/mycelium-std-fmt/src/lib.rs:172` | The machine-projection JSON view of a [`Value`] (spec §3 / G11). |
| `mycelium_std_fmt::Json::inner` | fn | `crates/mycelium-std-fmt/src/lib.rs:177` | Borrow the inner `serde_json::Value` for inspection. |
| `mycelium_std_fmt::MatrixRow` | struct | `crates/mycelium-std-fmt/src/lib.rs:346` | One row of the `std.fmt` guarantee matrix (RFC-0016 §4.5; spec §4). |
| `mycelium_std_fmt::Rendering` | struct | `crates/mycelium-std-fmt/src/lib.rs:152` | The result of `display_bounded`: a rendered text paired with its truncation record. |
| `mycelium_std_fmt::Text` | struct | `crates/mycelium-std-fmt/src/lib.rs:96` | A rendered text string (the output of a human projection). |
| `mycelium_std_fmt::Text::as_str` | fn | `crates/mycelium-std-fmt/src/lib.rs:101` | Borrow the inner string. |
| `mycelium_std_fmt::ToJsonError` | enum | `crates/mycelium-std-fmt/src/lib.rs:214` | Error the `to_json` machine projection can raise. |
| `mycelium_std_fmt::Truncation` | enum | `crates/mycelium-std-fmt/src/lib.rs:131` | Whether a [`Rendering`] is complete or whether some content was elided. |
| `mycelium_std_fmt::assert_matrix_invariants` | fn | `crates/mycelium-std-fmt/src/lib.rs:412` | Assert the structural invariants of the guarantee matrix — called from tests. |
| `mycelium_std_fmt::debug` | fn | `crates/mycelium-std-fmt/src/lib.rs:274` | Render `v` as a structural debug string (more detailed than `display`). |
| `mycelium_std_fmt::display` | fn | `crates/mycelium-std-fmt/src/lib.rs:260` | Render `v` as a human-readable string. |
| `mycelium_std_fmt::display_bounded` | fn | `crates/mycelium-std-fmt/src/lib.rs:290` | Render `v` within `limit` elements, emitting a typed `Truncation` record when content is |
| `mycelium_std_fmt::from_json` | fn | `crates/mycelium-std-fmt/src/lib.rs:336` | Reconstruct a [`Value`] from its machine JSON view (the `from_json` half). |
| `mycelium_std_fmt::to_json` | fn | `crates/mycelium-std-fmt/src/lib.rs:313` | Project `v` to a machine-faithful JSON view (the `to_json` half of the dual projection, G11). |

## mycelium-std-fs

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_fs::DirIter` | struct | `crates/mycelium-std-fs/src/lib.rs:110` | An open directory iterator handle. |
| `mycelium_std_fs::Effects` | enum | `crates/mycelium-std-fs/src/guarantee_matrix.rs:58` | Declared effects for an op (C6). |
| `mycelium_std_fs::ErrnoClass` | enum | `crates/mycelium-std-fs/src/error.rs:23` | The classified OS errno — never a bare raw code (C3: no opaque error codes). |
| `mycelium_std_fs::Explainable` | enum | `crates/mycelium-std-fs/src/guarantee_matrix.rs:49` | Whether an op has an EXPLAIN obligation (C3). |
| `mycelium_std_fs::Fallibility` | enum | `crates/mycelium-std-fs/src/guarantee_matrix.rs:29` | Fallibility classification for an exported op. |
| `mycelium_std_fs::File` | struct | `crates/mycelium-std-fs/src/lib.rs:87` | An affine open-file handle (LR-8: consumed exactly once). |
| `mycelium_std_fs::File::is_consumed` | fn | `crates/mycelium-std-fs/src/lib.rs:100` | Whether this handle has been consumed. |
| `mycelium_std_fs::FileKind` | enum | `crates/mycelium-std-fs/src/metadata.rs:15` | The kind of filesystem entry. |
| `mycelium_std_fs::Fs` | struct | `crates/mycelium-std-fs/src/lib.rs:131` | The filesystem context: holds the substrate and exposes all effectful fs ops. |
| `mycelium_std_fs::Fs::close` | fn | `crates/mycelium-std-fs/src/lib.rs:255` | Close (consume) a `File` handle. |
| `mycelium_std_fs::Fs::copy` | fn | `crates/mycelium-std-fs/src/lib.rs:327` | Copy `from` to `to`. |
| `mycelium_std_fs::Fs::create_dir` | fn | `crates/mycelium-std-fs/src/lib.rs:282` | Create a directory at `path`. |
| `mycelium_std_fs::Fs::exists` | fn | `crates/mycelium-std-fs/src/lib.rs:178` | Check whether a path exists. |
| `mycelium_std_fs::Fs::flush` | fn | `crates/mycelium-std-fs/src/lib.rs:242` | Flush deferred write state for a `File` handle. |
| `mycelium_std_fs::Fs::in_memory` | fn | `crates/mycelium-std-fs/src/lib.rs:141` | Create a new `Fs` over a fresh in-memory substrate. |
| `mycelium_std_fs::Fs::in_memory_with_limit` | fn | `crates/mycelium-std-fs/src/lib.rs:149` | Create a new `Fs` with a simulated disk limit (for testing `DiskFull` paths). |
| `mycelium_std_fs::Fs::open` | fn | `crates/mycelium-std-fs/src/lib.rs:203` | Open a path to an affine `File` handle under an explicit `OpenOptions`. |
| `mycelium_std_fs::Fs::path` | fn | `crates/mycelium-std-fs/src/lib.rs:94` | The original path this handle was opened for (for diagnostics). |
| `mycelium_std_fs::Fs::read` | fn | `crates/mycelium-std-fs/src/lib.rs:217` | Read bytes from an open `File` handle into `buf`. |
| `mycelium_std_fs::Fs::read_dir` | fn | `crates/mycelium-std-fs/src/lib.rs:269` | List the entries in a directory. |
| `mycelium_std_fs::Fs::remove_dir` | fn | `crates/mycelium-std-fs/src/lib.rs:302` | Remove an **empty** directory at `path`. |
| `mycelium_std_fs::Fs::remove_file` | fn | `crates/mycelium-std-fs/src/lib.rs:292` | Remove a regular file at `path`. |
| `mycelium_std_fs::Fs::rename` | fn | `crates/mycelium-std-fs/src/lib.rs:315` | Rename / move `from` to `to`. |
| `mycelium_std_fs::Fs::stat` | fn | `crates/mycelium-std-fs/src/lib.rs:188` | Get filesystem metadata for a path. |
| `mycelium_std_fs::Fs::write` | fn | `crates/mycelium-std-fs/src/lib.rs:229` | Write bytes to an open `File` handle. |
| `mycelium_std_fs::FsErr` | enum | `crates/mycelium-std-fs/src/error.rs:85` | The explicit, traceable filesystem error (RFC-0013 diagnostic record). |
| `mycelium_std_fs::MATRIX:` | const | `crates/mycelium-std-fs/src/guarantee_matrix.rs:92` | The `std.fs` guarantee matrix. |
| `mycelium_std_fs::MatrixRow` | struct | `crates/mycelium-std-fs/src/guarantee_matrix.rs:67` | One row in the `std.fs` guarantee matrix (RFC-0016 §4.5 / spec §4). |
| `mycelium_std_fs::Metadata` | struct | `crates/mycelium-std-fs/src/metadata.rs:97` | A read-only snapshot of filesystem entry metadata (C4 / ADR-003 — metadata is a value). |
| `mycelium_std_fs::OpenOptions` | struct | `crates/mycelium-std-fs/src/options.rs:28` | Declared open intent for `open` (spec §3). |
| `mycelium_std_fs::Path` | struct | `crates/mycelium-std-fs/src/path.rs:34` | An immutable, content-addressable UTF-8 filesystem path (C4 / ADR-003). |
| `mycelium_std_fs::Permissions` | struct | `crates/mycelium-std-fs/src/metadata.rs:32` | Read/write/execute permission bits for owner, group, and others. |
| `mycelium_std_fs::Wild` | enum | `crates/mycelium-std-fs/src/guarantee_matrix.rs:40` | Whether an op reaches the audited OS syscall floor. |
| `mycelium_std_fs::error` | mod | `crates/mycelium-std-fs/src/lib.rs:57` | — |
| `mycelium_std_fs::error::FsErr::errno_class` | fn | `crates/mycelium-std-fs/src/error.rs:184` | The classified errno — `None` for `UseAfterConsume` (caught above the OS floor). |
| `mycelium_std_fs::error::FsErr::path` | fn | `crates/mycelium-std-fs/src/error.rs:142` | The path that was attempted, if applicable. |
| `mycelium_std_fs::error::FsErr::why` | fn | `crates/mycelium-std-fs/src/error.rs:163` | The human-readable why-string (G11 dual projection). |
| `mycelium_std_fs::guarantee_matrix` | mod | `crates/mycelium-std-fs/src/lib.rs:58` | — |
| `mycelium_std_fs::metadata` | mod | `crates/mycelium-std-fs/src/lib.rs:59` | — |
| `mycelium_std_fs::metadata::Metadata::is_dir` | fn | `crates/mycelium-std-fs/src/metadata.rs:132` | Whether this entry is a directory. |
| `mycelium_std_fs::metadata::Metadata::is_file` | fn | `crates/mycelium-std-fs/src/metadata.rs:126` | Whether this entry is a regular file. |
| `mycelium_std_fs::metadata::Metadata::is_symlink` | fn | `crates/mycelium-std-fs/src/metadata.rs:138` | Whether this entry is a symbolic link. |
| `mycelium_std_fs::metadata::Metadata::new` | fn | `crates/mycelium-std-fs/src/metadata.rs:115` | Construct a metadata value directly (used by the in-memory substrate). |
| `mycelium_std_fs::metadata::Permissions::from_mode` | fn | `crates/mycelium-std-fs/src/metadata.rs:40` | Construct from raw Unix mode bits. |
| `mycelium_std_fs::metadata::Permissions::group_read` | fn | `crates/mycelium-std-fs/src/metadata.rs:70` | Whether the group has read permission. |
| `mycelium_std_fs::metadata::Permissions::is_readonly` | fn | `crates/mycelium-std-fs/src/metadata.rs:82` | Whether this entry is read-only for the owner (write bit not set). |
| `mycelium_std_fs::metadata::Permissions::others_read` | fn | `crates/mycelium-std-fs/src/metadata.rs:76` | Whether others have read permission. |
| `mycelium_std_fs::metadata::Permissions::owner_execute` | fn | `crates/mycelium-std-fs/src/metadata.rs:64` | Whether the owner has execute permission. |
| `mycelium_std_fs::metadata::Permissions::owner_read` | fn | `crates/mycelium-std-fs/src/metadata.rs:52` | Whether the owner has read permission. |
| `mycelium_std_fs::metadata::Permissions::owner_write` | fn | `crates/mycelium-std-fs/src/metadata.rs:58` | Whether the owner has write permission. |
| `mycelium_std_fs::metadata::Permissions::raw_mode` | fn | `crates/mycelium-std-fs/src/metadata.rs:46` | The raw mode bits (preserved for tooling; not the primary interface — C3). |
| `mycelium_std_fs::options` | mod | `crates/mycelium-std-fs/src/lib.rs:60` | — |
| `mycelium_std_fs::options::OpenOptions::new` | fn | `crates/mycelium-std-fs/src/options.rs:49` | All-false options: pure open (no create, no truncate, no write). |
| `mycelium_std_fs::options::OpenOptions::read_only` | fn | `crates/mycelium-std-fs/src/options.rs:64` | A read-only open (the most common case made ergonomic, while staying honest). |
| `mycelium_std_fs::options::OpenOptions::validate` | fn | `crates/mycelium-std-fs/src/options.rs:122` | Validate that the option combination is coherent. |
| `mycelium_std_fs::options::OpenOptions::wants_write` | fn | `crates/mycelium-std-fs/src/options.rs:134` | Whether this intent requests any write capability (write or append). |
| `mycelium_std_fs::options::OpenOptions::with_append` | fn | `crates/mycelium-std-fs/src/options.rs:87` | Builder: enable append mode. |
| `mycelium_std_fs::options::OpenOptions::with_create` | fn | `crates/mycelium-std-fs/src/options.rs:94` | Builder: enable create-if-absent. |
| `mycelium_std_fs::options::OpenOptions::with_create_new` | fn | `crates/mycelium-std-fs/src/options.rs:101` | Builder: enable create-and-fail-if-exists. |
| `mycelium_std_fs::options::OpenOptions::with_read` | fn | `crates/mycelium-std-fs/src/options.rs:73` | Builder: enable reading. |
| `mycelium_std_fs::options::OpenOptions::with_truncate` | fn | `crates/mycelium-std-fs/src/options.rs:108` | Builder: enable truncate. |
| `mycelium_std_fs::options::OpenOptions::with_write` | fn | `crates/mycelium-std-fs/src/options.rs:80` | Builder: enable writing. |
| `mycelium_std_fs::path` | mod | `crates/mycelium-std-fs/src/lib.rs:61` | — |
| `mycelium_std_fs::path::Path::as_str` | fn | `crates/mycelium-std-fs/src/path.rs:54` | The path as a string slice. |
| `mycelium_std_fs::path::Path::file_name` | fn | `crates/mycelium-std-fs/src/path.rs:113` | The final component of the path (the file/directory name), or `None` for root. |
| `mycelium_std_fs::path::Path::is_absolute` | fn | `crates/mycelium-std-fs/src/path.rs:136` | Whether this path starts with `/` (i.e. |
| `mycelium_std_fs::path::Path::join` | fn | `crates/mycelium-std-fs/src/path.rs:68` | Lexically join a child component onto this path. |
| `mycelium_std_fs::path::Path::new` | fn | `crates/mycelium-std-fs/src/path.rs:48` | Construct a `Path` from a UTF-8 string slice. |
| `mycelium_std_fs::path::Path::parent` | fn | `crates/mycelium-std-fs/src/path.rs:96` | The parent directory of this path, or `None` at the root. |

## mycelium-std-io

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_io::Budget` | enum | `crates/mycelium-std-io/src/io.rs:52` | A declared byte-read budget (C6/RFC-0014 §4.5; ADR-015). |
| `mycelium_std_io::ByteCount` | struct | `crates/mycelium-std-io/src/error.rs:149` | The number of bytes successfully read before an error. |
| `mycelium_std_io::ByteOffset` | struct | `crates/mycelium-std-io/src/error.rs:27` | A byte offset into the input: the **locus** of a serialization failure (C1 / |
| `mycelium_std_io::FieldPath` | struct | `crates/mycelium-std-io/src/error.rs:42` | A field path into a structured value, naming *where* inside a composite the |
| `mycelium_std_io::Format` | enum | `crates/mycelium-std-io/src/serialize.rs:59` | The two supported serialization formats (spec §3). |
| `mycelium_std_io::IoError` | enum | `crates/mycelium-std-io/src/error.rs:162` | The explicit error set for byte-movement failures (C1/RFC-0013; spec §3). |
| `mycelium_std_io::ReadValueError` | enum | `crates/mycelium-std-io/src/error.rs:206` | A combined error from [`crate::io::read_value`]: either a byte-movement failure |
| `mycelium_std_io::SerError` | enum | `crates/mycelium-std-io/src/error.rs:71` | The explicit error set for (de)serialization failures (C1/RFC-0013; spec §3). |
| `mycelium_std_io::Sink` | struct | `crates/mycelium-std-io/src/io.rs:170` | An abstract byte **sink**: a write target wrapped in an affine handle. |
| `mycelium_std_io::Source` | struct | `crates/mycelium-std-io/src/io.rs:115` | An abstract byte **source**: a `Substrate` wrapped in an affine handle. |
| `mycelium_std_io::Substrate` | struct | `crates/mycelium-std-io/src/io.rs:76` | The in-memory substrate: a `Vec<u8>` cursor. |
| `mycelium_std_io::deserialize` | fn | `crates/mycelium-std-io/src/serialize.rs:155` | Recover a `Value` from `bytes` serialized in the given `format`. |
| `mycelium_std_io::error` | mod | `crates/mycelium-std-io/src/lib.rs:101` | — |
| `mycelium_std_io::error::FieldPath::from_static` | fn | `crates/mycelium-std-io/src/error.rs:47` | Construct from a static description. |
| `mycelium_std_io::from_json` | fn | `crates/mycelium-std-io/src/serialize.rs:194` | Recover a `Value` from canonical JSON text. |
| `mycelium_std_io::guarantee_matrix` | mod | `crates/mycelium-std-io/src/lib.rs:102` | — |
| `mycelium_std_io::guarantee_matrix::Explainable` | enum | `crates/mycelium-std-io/src/guarantee_matrix.rs:75` | Whether the op surfaces an EXPLAIN artifact (C3). |
| `mycelium_std_io::guarantee_matrix::Fallibility` | enum | `crates/mycelium-std-io/src/guarantee_matrix.rs:65` | Fallibility classification for an exported op (C1). |
| `mycelium_std_io::guarantee_matrix::GuaranteeTag` | enum | `crates/mycelium-std-io/src/guarantee_matrix.rs:36` | Guarantee tag on the honesty lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` |
| `mycelium_std_io::guarantee_matrix::GuaranteeTag::as_str` | fn | `crates/mycelium-std-io/src/guarantee_matrix.rs:53` | Human-readable name matching the lattice notation (`"Exact"`, etc.). |
| `mycelium_std_io::guarantee_matrix::MATRIX:` | const | `crates/mycelium-std-io/src/guarantee_matrix.rs:105` | The `std.io` + `serialize` guarantee matrix. |
| `mycelium_std_io::guarantee_matrix::MatrixRow` | struct | `crates/mycelium-std-io/src/guarantee_matrix.rs:86` | One row in the `std.io` + `serialize` guarantee matrix (RFC-0016 §4.5). |
| `mycelium_std_io::io` | mod | `crates/mycelium-std-io/src/lib.rs:103` | — |
| `mycelium_std_io::io::Sink::into_bytes` | fn | `crates/mycelium-std-io/src/io.rs:188` | Consume the sink and return the bytes written into it. |
| `mycelium_std_io::io::Sink::new` | fn | `crates/mycelium-std-io/src/io.rs:124` | Wrap a substrate as an affine `Source`. |
| `mycelium_std_io::io::Source::from_bytes` | fn | `crates/mycelium-std-io/src/io.rs:89` | Construct a new in-memory substrate from a byte slice. |
| `mycelium_std_io::io::Source::remaining` | fn | `crates/mycelium-std-io/src/io.rs:98` | How many bytes remain unread. |
| `mycelium_std_io::read` | fn | `crates/mycelium-std-io/src/io.rs:250` | Read up to `budget` bytes from `src`, returning the bytes and the remaining |
| `mycelium_std_io::read_all` | fn | `crates/mycelium-std-io/src/io.rs:226` | Read all remaining bytes from `src`, consuming it exactly once (LR-8). |
| `mycelium_std_io::read_value` | fn | `crates/mycelium-std-io/src/io.rs:291` | Deserialize a `Value` directly from `src` in the given `format`, joining the |
| `mycelium_std_io::serialize` | mod | `crates/mycelium-std-io/src/lib.rs:104` | — |
| `mycelium_std_io::serialize` | fn | `crates/mycelium-std-io/src/serialize.rs:118` | Project `v` to the wire/JSON byte form for the given `format`. |
| `mycelium_std_io::to_json` | fn | `crates/mycelium-std-io/src/serialize.rs:179` | The **one canonical JSON projection**: project `v` to compact UTF-8 JSON text. |
| `mycelium_std_io::write` | fn | `crates/mycelium-std-io/src/io.rs:274` | Write `bytes` to `snk`, consuming the handle and returning the updated one |

## mycelium-std-iter

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_iter::AnyAllWitness` | struct | `crates/mycelium-std-iter/src/lib.rs:471` | The reified done-flag witness for [`any_with_witness`] and [`all_with_witness`] (C3). |
| `mycelium_std_iter::Foldable` | struct | `crates/mycelium-std-iter/src/foldable.rs:15` | — |
| `mycelium_std_iter::Lazy` | struct | `crates/mycelium-std-iter/src/lazy.rs:30` | A demand-driven, potentially-unbounded sequence. |
| `mycelium_std_iter::Transducer` | struct | `crates/mycelium-std-iter/src/transducer.rs:29` | A composable, source-independent step transformer. |
| `mycelium_std_iter::ZeroStep` | struct | `crates/mycelium-std-iter/src/error.rs:12` | Error returned by step_by when `k = 0`. |
| `mycelium_std_iter::ZipOutcome` | struct | `crates/mycelium-std-iter/src/zip_outcome.rs:17` | Records the outcome of a zip call — specifically, which side (if any) was |
| `mycelium_std_iter::all_with_witness` | fn | `crates/mycelium-std-iter/src/lib.rs:262` | Return `true` if all elements satisfy `pred`, together with an [`AnyAllWitness`]. |
| `mycelium_std_iter::any_with_witness` | fn | `crates/mycelium-std-iter/src/lib.rs:231` | Return `true` if any element satisfies `pred`, together with an [`AnyAllWitness`]. |
| `mycelium_std_iter::chain` | fn | `crates/mycelium-std-iter/src/lib.rs:375` | Append all elements of `right` after `left` — two finite spines remain finite. |
| `mycelium_std_iter::count` | fn | `crates/mycelium-std-iter/src/lib.rs:210` | Count the number of elements in `source`. |
| `mycelium_std_iter::enumerate` | fn | `crates/mycelium-std-iter/src/lib.rs:157` | Pair each element with its zero-based index. |
| `mycelium_std_iter::error` | mod | `crates/mycelium-std-iter/src/lib.rs:86` | — |
| `mycelium_std_iter::error::ZipLengthMismatch` | struct | `crates/mycelium-std-iter/src/error.rs:29` | Error returned by zip_exact when the left and right `Foldable`s have |
| `mycelium_std_iter::filter` | fn | `crates/mycelium-std-iter/src/lib.rs:121` | Keep only elements for which `pred` returns `true`. |
| `mycelium_std_iter::find` | fn | `crates/mycelium-std-iter/src/lib.rs:295` | Return the first element satisfying `pred`, or `None` if no element matches. |
| `mycelium_std_iter::flat_map` | fn | `crates/mycelium-std-iter/src/lib.rs:167` | Map each element to a `Foldable<F>` and flatten — finite-of-finite is finite (§4.7). |
| `mycelium_std_iter::fold` | fn | `crates/mycelium-std-iter/src/lib.rs:188` | The §4.8 `for` fold, surfaced directly. |
| `mycelium_std_iter::foldable` | mod | `crates/mycelium-std-iter/src/lib.rs:87` | — |
| `mycelium_std_iter::guarantee_matrix` | mod | `crates/mycelium-std-iter/src/lib.rs:88` | — |
| `mycelium_std_iter::guarantee_matrix::GuaranteeRow` | struct | `crates/mycelium-std-iter/src/guarantee_matrix.rs:31` | One row of the `std.iter` guarantee matrix (spec §4 / RFC-0016 §4.5). |
| `mycelium_std_iter::guarantee_matrix::MATRIX:` | const | `crates/mycelium-std-iter/src/guarantee_matrix.rs:50` | The full `std.iter` guarantee matrix — all ops (spec §4, 18 spec rows; 22 implementation rows |
| `mycelium_std_iter::lazy` | mod | `crates/mycelium-std-iter/src/lib.rs:89` | — |
| `mycelium_std_iter::lazy_take` | fn | `crates/mycelium-std-iter/src/lib.rs:456` | Convert a [`Lazy<E>`] back into a bounded, total [`Foldable<E>`] by applying a `Nat` bound. |
| `mycelium_std_iter::map` | fn | `crates/mycelium-std-iter/src/lib.rs:111` | Apply `f` to every element, producing a new `Foldable<F>`. |
| `mycelium_std_iter::position` | fn | `crates/mycelium-std-iter/src/lib.rs:311` | Return the zero-based index of the first element satisfying `pred`, or `None` if none. |
| `mycelium_std_iter::reduce` | fn | `crates/mycelium-std-iter/src/lib.rs:198` | Reduce a non-empty `Foldable` with a combining function, returning `None` on empty input. |
| `mycelium_std_iter::scan` | fn | `crates/mycelium-std-iter/src/lib.rs:133` | Running accumulator fold — length-preserving (one output element per input element). |
| `mycelium_std_iter::skip` | fn | `crates/mycelium-std-iter/src/lib.rs:399` | Drop the first `n` elements, returning the remainder. |
| `mycelium_std_iter::step_by` | fn | `crates/mycelium-std-iter/src/lib.rs:410` | Keep every `k`-th element (0-indexed). |
| `mycelium_std_iter::take` | fn | `crates/mycelium-std-iter/src/lib.rs:389` | Keep at most the first `n` elements. |
| `mycelium_std_iter::transduce` | fn | `crates/mycelium-std-iter/src/lib.rs:434` | Apply a [`Transducer<E, F>`] to `source`, reducing into `init` with `f`. |
| `mycelium_std_iter::transducer` | mod | `crates/mycelium-std-iter/src/lib.rs:90` | — |
| `mycelium_std_iter::zip` | fn | `crates/mycelium-std-iter/src/lib.rs:336` | Pair elements from two `Foldable`s, truncating to the shorter spine. |
| `mycelium_std_iter::zip_exact` | fn | `crates/mycelium-std-iter/src/lib.rs:353` | Pair elements from two `Foldable`s; return `Err(ZipLengthMismatch)` if lengths differ. |
| `mycelium_std_iter::zip_outcome` | mod | `crates/mycelium-std-iter/src/lib.rs:91` | — |
| `mycelium_std_iter::zip_outcome::ZipOutcome::left_excess` | fn | `crates/mycelium-std-iter/src/zip_outcome.rs:60` | The number of elements dropped from the left side (0 if left was the shorter or equal). |
| `mycelium_std_iter::zip_outcome::ZipOutcome::left_len` | fn | `crates/mycelium-std-iter/src/zip_outcome.rs:36` | The number of elements in the left input. |
| `mycelium_std_iter::zip_outcome::ZipOutcome::result_len` | fn | `crates/mycelium-std-iter/src/zip_outcome.rs:48` | The number of pairs produced (= `min(left_len, right_len)`). |
| `mycelium_std_iter::zip_outcome::ZipOutcome::right_excess` | fn | `crates/mycelium-std-iter/src/zip_outcome.rs:66` | The number of elements dropped from the right side (0 if right was the shorter or equal). |
| `mycelium_std_iter::zip_outcome::ZipOutcome::right_len` | fn | `crates/mycelium-std-iter/src/zip_outcome.rs:42` | The number of elements in the right input. |
| `mycelium_std_iter::zip_outcome::ZipOutcome::was_truncated` | fn | `crates/mycelium-std-iter/src/zip_outcome.rs:54` | `true` if the two inputs had different lengths (some elements were dropped). |

## mycelium-std-math

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_math::Approx` | struct | `crates/mycelium-std-math/src/approx.rs:69` | `Approx<f64>` — a thin carrier for an approximate `f64` result with its attached bound. |
| `mycelium_std_math::ApproxExplain` | struct | `crates/mycelium-std-math/src/approx.rs:85` | The dual human/machine EXPLAIN record for an [`Approx`] result (G11; C3). |
| `mycelium_std_math::GUARANTEE_MATRIX:` | const | `crates/mycelium-std-math/src/matrix.rs:49` | The `std.math` guarantee matrix (spec §4; RFC-0016 §4.5). |
| `mycelium_std_math::GuaranteeRow` | struct | `crates/mycelium-std-math/src/matrix.rs:27` | One row of the `std.math` guarantee matrix (RFC-0016 §4.5; spec §4). |
| `mycelium_std_math::MathErr` | enum | `crates/mycelium-std-math/src/lib.rs:92` | The explicit error set for fallible `std.math` ops (spec §3; C1 / G2). |
| `mycelium_std_math::RoundMode` | enum | `crates/mycelium-std-math/src/exact.rs:30` | The reified rounding mode for [`round`] (spec §3; C3 / SC-3 / G11). |
| `mycelium_std_math::approx` | mod | `crates/mycelium-std-math/src/lib.rs:77` | — |
| `mycelium_std_math::approx::acos` | fn | `crates/mycelium-std-math/src/approx.rs:398` | `acos(x)` — approximate arccosine. |
| `mycelium_std_math::approx::asin` | fn | `crates/mycelium-std-math/src/approx.rs:383` | `asin(x)` — approximate arcsine. |
| `mycelium_std_math::approx::atan` | fn | `crates/mycelium-std-math/src/approx.rs:413` | `atan(x)` — approximate arctangent. |
| `mycelium_std_math::approx::atan2` | fn | `crates/mycelium-std-math/src/approx.rs:430` | `atan2(y, x)` — approximate four-quadrant arctangent. |
| `mycelium_std_math::approx::cbrt` | fn | `crates/mycelium-std-math/src/approx.rs:192` | `cbrt(x)` — approximate cube root. |
| `mycelium_std_math::approx::cos` | fn | `crates/mycelium-std-math/src/approx.rs:342` | `cos(x)` — approximate cosine. |
| `mycelium_std_math::approx::declared_error_bound` | fn | `crates/mycelium-std-math/src/approx.rs:50` | The `Declared` ε bound attached to all approximate ops in this implementation. |
| `mycelium_std_math::approx::exp` | fn | `crates/mycelium-std-math/src/approx.rs:208` | `exp(x)` — approximate natural exponential `eˣ`. |
| `mycelium_std_math::approx::hypot` | fn | `crates/mycelium-std-math/src/approx.rs:305` | `hypot(x, y)` — approximate `√(x² + y²)`. |
| `mycelium_std_math::approx::log` | fn | `crates/mycelium-std-math/src/approx.rs:227` | `log(x)` — approximate natural logarithm `ln(x)`. |
| `mycelium_std_math::approx::logb` | fn | `crates/mycelium-std-math/src/approx.rs:243` | `logb(b, x)` — approximate base-`b` logarithm `log_b(x)`. |
| `mycelium_std_math::approx::pow` | fn | `crates/mycelium-std-math/src/approx.rs:278` | `pow(x, y)` — approximate `xʸ`. |
| `mycelium_std_math::approx::sin` | fn | `crates/mycelium-std-math/src/approx.rs:327` | `sin(x)` — approximate sine. |
| `mycelium_std_math::approx::sqrt` | fn | `crates/mycelium-std-math/src/approx.rs:173` | `sqrt(x)` — approximate square root. |
| `mycelium_std_math::approx::tan` | fn | `crates/mycelium-std-math/src/approx.rs:363` | `tan(x)` — approximate tangent. |
| `mycelium_std_math::assert_matrix_invariants` | fn | `crates/mycelium-std-math/src/matrix.rs:283` | Assert structural invariants on [`GUARANTEE_MATRIX`] — the RFC-0016 §4.5 obligation. |
| `mycelium_std_math::exact` | mod | `crates/mycelium-std-math/src/lib.rs:78` | — |
| `mycelium_std_math::exact::abs` | fn | `crates/mycelium-std-math/src/exact.rs:165` | `abs(x)` — absolute value of a signed integer. |
| `mycelium_std_math::exact::ceil` | fn | `crates/mycelium-std-math/src/exact.rs:82` | `ceil(x)` — round toward positive infinity (exact under the `Ceil` mode). |
| `mycelium_std_math::exact::checked_div` | fn | `crates/mycelium-std-math/src/exact.rs:290` | `checked_div(a, b)` — exact integer division. |
| `mycelium_std_math::exact::checked_rem` | fn | `crates/mycelium-std-math/src/exact.rs:305` | `checked_rem(a, b)` — exact integer remainder (`a mod b`, truncated toward zero). |
| `mycelium_std_math::exact::floor` | fn | `crates/mycelium-std-math/src/exact.rs:67` | `floor(x)` — round toward negative infinity (exact under the `Floor` mode). |
| `mycelium_std_math::exact::gcd` | fn | `crates/mycelium-std-math/src/exact.rs:227` | `gcd(a, b)` — greatest common divisor (always non-negative). |
| `mycelium_std_math::exact::lcm` | fn | `crates/mycelium-std-math/src/exact.rs:262` | `lcm(a, b)` — least common multiple (always non-negative). |
| `mycelium_std_math::exact::max` | fn | `crates/mycelium-std-math/src/exact.rs:209` | `max(a, b)` — maximum of two signed integers. |
| `mycelium_std_math::exact::min` | fn | `crates/mycelium-std-math/src/exact.rs:195` | `min(a, b)` — minimum of two signed integers. |
| `mycelium_std_math::exact::neg` | fn | `crates/mycelium-std-math/src/exact.rs:176` | `neg(x)` — negation of a signed integer. |
| `mycelium_std_math::exact::ratio` | fn | `crates/mycelium-std-math/src/exact.rs:324` | `ratio(a, b)` — exact rational representation of `a/b`. |
| `mycelium_std_math::exact::round` | fn | `crates/mycelium-std-math/src/exact.rs:116` | `round(x, mode)` — round `x` to the nearest integer under the named, reified [`RoundMode`]. |
| `mycelium_std_math::exact::signum` | fn | `crates/mycelium-std-math/src/exact.rs:184` | `signum(x)` — signum of a signed integer: -1, 0, or 1. |
| `mycelium_std_math::exact::trunc` | fn | `crates/mycelium-std-math/src/exact.rs:97` | `trunc(x)` — round toward zero (exact under the `TruncTowardZero` mode). |
| `mycelium_std_math::matrix` | mod | `crates/mycelium-std-math/src/lib.rs:79` | — |

## mycelium-std-numerics

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_numerics::Approx` | struct | `crates/mycelium-std-numerics/src/lib.rs:246` | A thin view pairing a value with its `{Bound, strength}` (RFC-0001 §4.3 `Meta`) — **not** a |
| `mycelium_std_numerics::CheckErr` | enum | `crates/mycelium-std-numerics/src/lib.rs:196` | Structured verdict for the tier-i re-validation checker (spec §3 `CheckErr`; RFC-0013). |
| `mycelium_std_numerics::DECLARED_FLOAT_EPS:` | const | `crates/mycelium-std-numerics/src/lib.rs:73` | The `Declared`-strength ε upper bound for `f64` operations whose compute floor is the |
| `mycelium_std_numerics::Explanation` | struct | `crates/mycelium-std-numerics/src/lib.rs:414` | The `explain` artifact for an [`Approx<T>`] (C3; G11 dual human/machine projection). |
| `mycelium_std_numerics::NumErr` | enum | `crates/mycelium-std-numerics/src/lib.rs:154` | Structured refusal record for `std.numerics` helpers (C1; RFC-0013; spec §3 `NumErr`). |
| `mycelium_std_numerics::ProvenThm` | struct | `crates/mycelium-std-numerics/src/lib.rs:117` | A checked-theorem witness required to construct an [`Approx`] with `Proven` strength (FR-N3). |
| `mycelium_std_numerics::ProvenThm::new` | fn | `crates/mycelium-std-numerics/src/lib.rs:134` | Construct a [`ProvenThm`] witness with the given `citation`. |
| `mycelium_std_numerics::accuracy_to_probability` | fn | `crates/mycelium-std-numerics/src/lib.rs:585` | The single sanctioned cross-kernel inference (spec §3 `accuracy_to_probability`; ADR-010 §4). |
| `mycelium_std_numerics::check_error` | fn | `crates/mycelium-std-numerics/src/lib.rs:643` | Re-validate a claimed ε bound for `op` over `input_bounds` via the M-203 tier-i checker. |
| `mycelium_std_numerics::check_union` | fn | `crates/mycelium-std-numerics/src/lib.rs:670` | Re-validate a claimed δ union bound over `input_bounds` via the M-203 tier-i checker. |
| `mycelium_std_numerics::error_bound` | fn | `crates/mycelium-std-numerics/src/lib.rs:493` | Construct an `ErrorBound{eps, norm, basis}` (spec §3 `error_bound`). |
| `mycelium_std_numerics::matrix` | mod | `crates/mycelium-std-numerics/src/lib.rs:50` | — |
| `mycelium_std_numerics::matrix::GUARANTEE_MATRIX:` | const | `crates/mycelium-std-numerics/src/matrix.rs:48` | The `std.numerics` guarantee matrix (spec §4; RFC-0016 §4.5). |
| `mycelium_std_numerics::matrix::GuaranteeRow` | struct | `crates/mycelium-std-numerics/src/matrix.rs:30` | One row of the `std.numerics` guarantee matrix (RFC-0016 §4.5; spec §4). |
| `mycelium_std_numerics::matrix::assert_matrix_invariants` | fn | `crates/mycelium-std-numerics/src/matrix.rs:195` | Assert structural invariants on [`GUARANTEE_MATRIX`] — the RFC-0016 §4.5 obligation. |
| `mycelium_std_numerics::prob_bound` | fn | `crates/mycelium-std-numerics/src/lib.rs:510` | Construct a `ProbabilityBound{delta, basis}` (spec §3 `prob_bound`). |
| `mycelium_std_numerics::union_delta` | fn | `crates/mycelium-std-numerics/src/lib.rs:537` | Compose the **δ union bound** of a slice of `Probability`-kind bounds, taking the **meet** of |

## mycelium-std-rand

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_rand::EntropyEffect` | struct | `crates/mycelium-std-rand/src/lib.rs:142` | The reified `entropy` declared effect (C6 / RT3 / RFC-0014). |
| `mycelium_std_rand::EntropyRng` | struct | `crates/mycelium-std-rand/src/lib.rs:569` | An entropy-backed generator. |
| `mycelium_std_rand::EntropySource` | trait | `crates/mycelium-std-rand/src/lib.rs:152` | Injectable entropy source — the seam between pure `std.rand` and the `std-sys` phylum. |
| `mycelium_std_rand::GUARANTEE_MATRIX:` | const | `crates/mycelium-std-rand/src/lib.rs:740` | The `std.rand` guarantee matrix (spec §4 / RFC-0016 §4.5). |
| `mycelium_std_rand::MatrixRow` | struct | `crates/mycelium-std-rand/src/lib.rs:705` | One row of the `std.rand` guarantee matrix (RFC-0016 §4.5; rand.md §4). |
| `mycelium_std_rand::RandErr` | enum | `crates/mycelium-std-rand/src/lib.rs:85` | Errors returned by `std.rand` operations (C1 — every fallible op returns this |
| `mycelium_std_rand::Rng` | struct | `crates/mycelium-std-rand/src/lib.rs:195` | A seeded, deterministic generator **value** (spec §3). |
| `mycelium_std_rand::Rng::algo` | fn | `crates/mycelium-std-rand/src/lib.rs:227` | The algorithm this generator uses (inspectable; C3). |
| `mycelium_std_rand::Rng::state` | fn | `crates/mycelium-std-rand/src/lib.rs:221` | The current raw state (inspectable; C3). |
| `mycelium_std_rand::RngAlgo` | enum | `crates/mycelium-std-rand/src/lib.rs:171` | The PRNG algorithm used by a [`Rng`] — the inspectable algorithm tag (C3). |
| `mycelium_std_rand::StubEntropy` | struct | `crates/mycelium-std-rand/src/lib.rs:670` | A deterministic, injectable [`EntropySource`] for tests. |
| `mycelium_std_rand::StubEntropy::new` | fn | `crates/mycelium-std-rand/src/lib.rs:591` | Construct an `EntropyRng` by seeding from the given entropy source. |
| `mycelium_std_rand::assert_matrix_invariants` | fn | `crates/mycelium-std-rand/src/lib.rs:843` | Assert the structural invariants of the guarantee matrix — called from tests. |
| `mycelium_std_rand::bernoulli` | fn | `crates/mycelium-std-rand/src/lib.rs:439` | Draw a `bool` from a Bernoulli distribution with success probability `p`. |
| `mycelium_std_rand::choice` | fn | `crates/mycelium-std-rand/src/lib.rs:463` | Choose one element uniformly at random from a non-empty slice. |
| `mycelium_std_rand::exponential` | fn | `crates/mycelium-std-rand/src/lib.rs:529` | Draw from an Exponential(λ) distribution using the inverse-CDF method. |
| `mycelium_std_rand::next_u64` | fn | `crates/mycelium-std-rand/src/lib.rs:317` | Draw the next raw `u64` from a seeded generator. |
| `mycelium_std_rand::normal` | fn | `crates/mycelium-std-rand/src/lib.rs:508` | Draw from a Normal(μ, σ) distribution using the Box–Muller transform. |
| `mycelium_std_rand::seed` | fn | `crates/mycelium-std-rand/src/lib.rs:300` | Build an [`Rng`] from a `u64` seed. |
| `mycelium_std_rand::seed_from_entropy` | fn | `crates/mycelium-std-rand/src/lib.rs:634` | Mint a single reproducible seed from entropy, then return a pure [`Rng`]. |
| `mycelium_std_rand::shuffle` | fn | `crates/mycelium-std-rand/src/lib.rs:482` | Produce a uniformly-random permutation of the input slice (Fisher–Yates shuffle). |
| `mycelium_std_rand::split` | fn | `crates/mycelium-std-rand/src/lib.rs:338` | Derive two independent sub-stream generators from one (the "split" operation). |
| `mycelium_std_rand::uniform_int` | fn | `crates/mycelium-std-rand/src/lib.rs:374` | Draw a uniformly-distributed `i64` in the half-open range `[lo, hi)`. |
| `mycelium_std_rand::uniform_u64` | fn | `crates/mycelium-std-rand/src/lib.rs:394` | Draw a uniformly-distributed `u64` in the half-open range `[lo, hi)`. |

## mycelium-std-recover

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_recover::ClassName` | struct | `crates/mycelium-std-recover/src/registry.rs:19` | A registry-resolved error class name (RFC-0013 §4.5 — X1). |
| `mycelium_std_recover::ClassRegistry` | struct | `crates/mycelium-std-recover/src/registry.rs:69` | A simple, append-only **error-class registry** (RFC-0013 §4.5 / X1). |
| `mycelium_std_recover::DiagError` | struct | `crates/mycelium-std-recover/src/outcome.rs:63` | A diagnosed error value: the error payload `E` bundled with its [`Diag`] record (FR-R5). |
| `mycelium_std_recover::EffectSet` | type | `crates/mycelium-std-recover/src/effect.rs:24` | A definition's **declared** effect set (§4.5 I3) — the set it names on its signature. |
| `mycelium_std_recover::Outcome` | enum | `crates/mycelium-std-recover/src/outcome.rs:21` | The input result sum `Ok(T) \| Err(E)` (RFC-0014 §4.1). |
| `mycelium_std_recover::PolicyHashError` | struct | `crates/mycelium-std-recover/src/policy.rs:50` | An error computing the content address of a [`RecoveryPolicy`] (banked guard #5). |
| `mycelium_std_recover::PolicyRef` | type | `crates/mycelium-std-recover/src/policy.rs:35` | The content address of a `RecoveryPolicy` (RFC-0001 §4.6 / ADR-006 / `PolicyRef`). |
| `mycelium_std_recover::RecoverOutcome` | type | `crates/mycelium-std-recover/src/lib.rs:89` | `RecoverOutcome<T, E>` is `Resolution<T, E>` — the concrete shape that resolves |
| `mycelium_std_recover::RecoveryAction` | enum | `crates/mycelium-std-recover/src/action.rs:33` | The **closed** v0 recovery-action set (RFC-0014 §4.4; §8 resolved). |
| `mycelium_std_recover::RecoveryPolicy` | struct | `crates/mycelium-std-recover/src/policy.rs:86` | A reified, content-addressed recovery policy. |
| `mycelium_std_recover::Resolution` | enum | `crates/mycelium-std-recover/src/outcome.rs:90` | The **outcome of handling** an [`Outcome`] under a recovery policy (RFC-0014 §4.2). |
| `mycelium_std_recover::UndeclaredEffect` | struct | `crates/mycelium-std-recover/src/effect.rs:32` | A performed-but-undeclared effect (I3) — an explicit checker error, never silent. |
| `mycelium_std_recover::UnknownClass` | struct | `crates/mycelium-std-recover/src/registry.rs:39` | The explicit error returned by [`ClassRegistry::resolve`] when a name is not registered (X1). |
| `mycelium_std_recover::action` | mod | `crates/mycelium-std-recover/src/lib.rs:61` | — |
| `mycelium_std_recover::check_effects` | fn | `crates/mycelium-std-recover/src/effect.rs:62` | The **compositional no-undeclared-effect check** (I3). |
| `mycelium_std_recover::effect` | mod | `crates/mycelium-std-recover/src/lib.rs:62` | — |
| `mycelium_std_recover::guarantee_matrix` | mod | `crates/mycelium-std-recover/src/lib.rs:63` | — |
| `mycelium_std_recover::guarantee_matrix::Explainable` | enum | `crates/mycelium-std-recover/src/guarantee_matrix.rs:26` | Whether an op carries an EXPLAIN obligation (C3 — no black boxes). |
| `mycelium_std_recover::guarantee_matrix::Fallibility` | enum | `crates/mycelium-std-recover/src/guarantee_matrix.rs:15` | Fallibility classification for a `std.recover` exported op (I1 — explicit outcome set). |
| `mycelium_std_recover::guarantee_matrix::MatrixRow` | struct | `crates/mycelium-std-recover/src/guarantee_matrix.rs:37` | One row in the `std.recover` guarantee matrix (RFC-0016 §4.5; spec §4). |
| `mycelium_std_recover::handle` | mod | `crates/mycelium-std-recover/src/lib.rs:64` | — |
| `mycelium_std_recover::handle_classified` | fn | `crates/mycelium-std-recover/src/handle.rs:75` | Handle an [`Outcome`] under a recovery policy, providing the error's class for rule lookup. |
| `mycelium_std_recover::outcome` | mod | `crates/mycelium-std-recover/src/lib.rs:65` | — |
| `mycelium_std_recover::policy` | mod | `crates/mycelium-std-recover/src/lib.rs:66` | — |
| `mycelium_std_recover::policy_effects` | fn | `crates/mycelium-std-recover/src/policy.rs:242` | The declared, closed effect set for a policy (I3 / RFC-0014 §4.5). |
| `mycelium_std_recover::recover_classified` | fn | `crates/mycelium-std-recover/src/handle.rs:198` | Convenience: bridge a `Result<T, E>` into a [`Resolution<T, E>`] under a policy. |
| `mycelium_std_recover::registry` | mod | `crates/mycelium-std-recover/src/lib.rs:67` | — |
| `mycelium_std_recover::registry::ClassName::as_str` | fn | `crates/mycelium-std-recover/src/registry.rs:24` | The string representation of this name (for display and hashing only — not for equality). |
| `mycelium_std_recover::registry::ClassRegistry::contains` | fn | `crates/mycelium-std-recover/src/registry.rs:108` | Whether a name is registered. |
| `mycelium_std_recover::registry::ClassRegistry::new` | fn | `crates/mycelium-std-recover/src/registry.rs:76` | An empty registry (no classes registered yet). |
| `mycelium_std_recover::registry::ClassRegistry::register` | fn | `crates/mycelium-std-recover/src/registry.rs:81` | Register a class name. |
| `mycelium_std_recover::registry::ClassRegistry::resolve` | fn | `crates/mycelium-std-recover/src/registry.rs:96` | Resolve a string to a [`ClassName`] if it is registered. |
| `mycelium_std_recover::registry::ClassRegistry::with` | fn | `crates/mycelium-std-recover/src/registry.rs:87` | Builder: register a name. |

## mycelium-std-runtime

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_runtime::colony` | mod | `crates/mycelium-std-runtime/src/lib.rs:29` | — |
| `mycelium_std_runtime::colony::COLONY_KAHN_STRENGTH:` | const | `crates/mycelium-std-runtime/src/colony.rs:24` | Guarantee strength for `Colony` Kahn-determinism (channel-mediated communication). |
| `mycelium_std_runtime::colony::Colony` | struct | `crates/mycelium-std-runtime/src/colony.rs:129` | Colony: a group of scopes sharing a supervision tree and a `Network`. |
| `mycelium_std_runtime::colony::SCOPE_JOIN_STRENGTH:` | const | `crates/mycelium-std-runtime/src/colony.rs:21` | Guarantee strength for `Scope` join semantics (RT2 sequentialization differential). |
| `mycelium_std_runtime::colony::Scope` | struct | `crates/mycelium-std-runtime/src/colony.rs:43` | Structured concurrency scope: all tasks complete or are cancelled before scope exit. |
| `mycelium_std_runtime::colony::ScopeError` | enum | `crates/mycelium-std-runtime/src/colony.rs:28` | Error type for scope exits with active tasks. |
| `mycelium_std_runtime::guarantee_matrix` | mod | `crates/mycelium-std-runtime/src/lib.rs:30` | — |
| `mycelium_std_runtime::guarantee_matrix::GaugeRow` | struct | `crates/mycelium-std-runtime/src/guarantee_matrix.rs:10` | One row in the guarantee matrix. |
| `mycelium_std_runtime::network` | mod | `crates/mycelium-std-runtime/src/lib.rs:31` | — |
| `mycelium_std_runtime::network::CHANNEL_FIFO_STRENGTH:` | const | `crates/mycelium-std-runtime/src/network.rs:21` | Guarantee strength for single-channel FIFO ordering. |
| `mycelium_std_runtime::network::ChannelError` | enum | `crates/mycelium-std-runtime/src/network.rs:49` | Errors returned by `Network` construction operations. |
| `mycelium_std_runtime::network::KAHN_DETERMINISM_STRENGTH:` | const | `crates/mycelium-std-runtime/src/network.rs:24` | Guarantee strength for cross-channel Kahn-determinism. |
| `mycelium_std_runtime::network::Network` | struct | `crates/mycelium-std-runtime/src/network.rs:60` | A named network of typed channels within a `Colony`. |
| `mycelium_std_runtime::network::Network::channel` | fn | `crates/mycelium-std-runtime/src/network.rs:81` | Create a bounded FIFO channel with the given capacity. |
| `mycelium_std_runtime::network::Network::new` | fn | `crates/mycelium-std-runtime/src/network.rs:68` | Create a new network. |
| `mycelium_std_runtime::network::Receiver` | struct | `crates/mycelium-std-runtime/src/network.rs:157` | Receiving end of a typed channel. |
| `mycelium_std_runtime::network::Sender` | struct | `crates/mycelium-std-runtime/src/network.rs:109` | Sending end of a typed channel. |
| `mycelium_std_runtime::network::TryRecv` | enum | `crates/mycelium-std-runtime/src/network.rs:206` | Result of a non-blocking receive attempt. |
| `mycelium_std_runtime::network::TrySend` | enum | `crates/mycelium-std-runtime/src/network.rs:195` | Result of a non-blocking send attempt. |
| `mycelium_std_runtime::task` | mod | `crates/mycelium-std-runtime/src/lib.rs:32` | — |
| `mycelium_std_runtime::task::Deadlock` | struct | `crates/mycelium-std-runtime/src/task.rs:97` | Deadlock descriptor: returned when a scope cannot make progress. |
| `mycelium_std_runtime::task::Poll` | enum | `crates/mycelium-std-runtime/src/task.rs:75` | Poll result for an async task step. |
| `mycelium_std_runtime::task::SweepOrder` | enum | `crates/mycelium-std-runtime/src/task.rs:84` | Order in which tasks are swept from a scope's run queue. |
| `mycelium_std_runtime::task::TASK_PURITY_STRENGTH:` | const | `crates/mycelium-std-runtime/src/task.rs:12` | Guarantee strength for the `Task` purity contract. |
| `mycelium_std_runtime::task::Task` | struct | `crates/mycelium-std-runtime/src/task.rs:21` | A computation that can be spawned into a `Scope`. |
| `mycelium_std_runtime::task::Task::new` | fn | `crates/mycelium-std-runtime/src/task.rs:27` | Construct a task from a closure. |
| `mycelium_std_runtime::task::Task::run` | fn | `crates/mycelium-std-runtime/src/task.rs:34` | Run the task closure exactly once. |
| `mycelium_std_runtime::task::TaskCtx` | struct | `crates/mycelium-std-runtime/src/task.rs:47` | Context passed to a running task — carries cancellation signal and scope ref. |
| `mycelium_std_runtime::task::TaskCtx::cancel` | fn | `crates/mycelium-std-runtime/src/task.rs:62` | Cancel this context (used by `Scope` when cancellation is requested). |
| `mycelium_std_runtime::task::TaskCtx::is_cancelled` | fn | `crates/mycelium-std-runtime/src/task.rs:57` | Returns `true` if this task's scope has been cancelled. |

## mycelium-std-select

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_select::ExplainAble` | enum | `crates/mycelium-std-select/src/lib.rs:350` | Whether an op emits a valid, inspectable `Explanation` (the C3 / SC-3 crux). |
| `mycelium_std_select::GuaranteeRow` | struct | `crates/mycelium-std-select/src/lib.rs:309` | One row of the guarantee matrix (RFC-0016 §4.5; spec §4). |
| `mycelium_std_select::GuaranteeTag` | enum | `crates/mycelium-std-select/src/lib.rs:331` | The honest guarantee tag — C2 / VR-5. |
| `mycelium_std_select::PolicyRef` | type | `crates/mycelium-std-select/src/lib.rs:91` | A **content hash** that identifies a [`SelectionPolicy`] — recorded in `Meta.policy_used` so |
| `mycelium_std_select::build` | fn | `crates/mycelium-std-select/src/lib.rs:130` | Build and validate a [`SelectionPolicy`] from a name, candidates, rules, a default arm, and |
| `mycelium_std_select::explain` | fn | `crates/mycelium-std-select/src/lib.rs:239` | The **explain capability** (RFC-0005 §4): derive the mandatory [`Explanation`] for a |
| `mycelium_std_select::policy_ref` | fn | `crates/mycelium-std-select/src/lib.rs:145` | Return the content address of a [`SelectionPolicy`] — its [`PolicyRef`] (RFC-0005 §3). |
| `mycelium_std_select::select` | fn | `crates/mycelium-std-select/src/lib.rs:195` | The **one selection mechanism** (RFC-0005 §4; C3): evaluate the decision table and return |
| `mycelium_std_select::select_with_override` | fn | `crates/mycelium-std-select/src/lib.rs:291` | A **first-class deterministic override**: force a specific candidate by index and record |

## mycelium-std-spore

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_spore::DeployError` | enum | `crates/mycelium-std-spore/src/deploy.rs:137` | An explicit deploy error — never a silent fallback (G2 / VR-4 / VR-5). |
| `mycelium_std_spore::DeployResult` | enum | `crates/mycelium-std-spore/src/deploy.rs:102` | The outcome of a germination attempt (ADR-013 native path — M-620). |
| `mycelium_std_spore::DeployTarget` | enum | `crates/mycelium-std-spore/src/deploy.rs:48` | Where to deploy a spore. |
| `mycelium_std_spore::DeployVerification` | struct | `crates/mycelium-std-spore/src/deploy.rs:76` | What was verified during a successful deploy (VR-4 / C4 / ADR-003). |
| `mycelium_std_spore::MATRIX:` | const | `crates/mycelium-std-spore/src/guarantee_matrix.rs:69` | The `std.spore` guarantee matrix (spec §4.5), encoded as data (RFC-0016 §4.5). |
| `mycelium_std_spore::MalformedManifest` | enum | `crates/mycelium-std-spore/src/recon_manifest.rs:190` | A refusal from manifest validation — explicitly named, never silent (C1/G2). |
| `mycelium_std_spore::ReconManifest` | struct | `crates/mycelium-std-spore/src/recon_manifest.rs:37` | A validated reconstruction manifest — the RFC-0003 §6 record: mode, model, dim, codebooks, |
| `mycelium_std_spore::RegrowthResult` | struct | `crates/mycelium-std-spore/src/recon_manifest.rs:233` | The result of a probabilistic regrowth attempt via `std.vsa`. |
| `mycelium_std_spore::SporeErr` | enum | `crates/mycelium-std-spore/src/spore_ops.rs:29` | An explicit spore error — never a silent accept (C1/G2). |
| `mycelium_std_spore::SporeUnit` | struct | `crates/mycelium-std-spore/src/spore_ops.rs:96` | A content-addressed, value-semantic spore handle (ADR-013). |
| `mycelium_std_spore::deploy` | mod | `crates/mycelium-std-spore/src/lib.rs:60` | — |
| `mycelium_std_spore::explain_deploy` | fn | `crates/mycelium-std-spore/src/deploy.rs:339` | The EXPLAIN of a germination outcome — VR-4 / SC-3 / C3 / G11. |
| `mycelium_std_spore::explain_spore` | fn | `crates/mycelium-std-spore/src/spore_ops.rs:344` | The `EXPLAIN` of a built spore: the identity receipt, the surface, the code by hash, the |
| `mycelium_std_spore::germinate` | fn | `crates/mycelium-std-spore/src/deploy.rs:235` | Deploy a [`SporeUnit`] to a [`DeployTarget`] — the ADR-013 native germination entry point. |
| `mycelium_std_spore::guarantee_matrix` | mod | `crates/mycelium-std-spore/src/lib.rs:61` | — |
| `mycelium_std_spore::guarantee_matrix::GuaranteeTag` | type | `crates/mycelium-std-spore/src/guarantee_matrix.rs:31` | Guarantee tag string — the lattice position (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`). |
| `mycelium_std_spore::guarantee_matrix::MatrixRow` | struct | `crates/mycelium-std-spore/src/guarantee_matrix.rs:35` | One row of the `std.spore` guarantee matrix (RFC-0016 §4.5 / spec §4). |
| `mycelium_std_spore::identity` | fn | `crates/mycelium-std-spore/src/spore_ops.rs:227` | The spore's canonical content-addressed identity (ADR-003). |
| `mycelium_std_spore::manifest_of` | fn | `crates/mycelium-std-spore/src/spore_ops.rs:321` | The reconstruction manifest of a `SporeUnit`, if any — `None` for project spores without one. |
| `mycelium_std_spore::recon_manifest` | mod | `crates/mycelium-std-spore/src/lib.rs:62` | — |
| `mycelium_std_spore::recon_manifest::ReconManifest::declared_strength` | fn | `crates/mycelium-std-spore/src/recon_manifest.rs:126` | The declared guarantee strength from the manifest's bound certificate. |
| `mycelium_std_spore::recon_manifest::ReconManifest::delta` | fn | `crates/mycelium-std-spore/src/recon_manifest.rs:164` | The bound's failure-probability δ, if this is a `ProbabilityBound` (the common case for |
| `mycelium_std_spore::recon_manifest::ReconManifest::inner` | fn | `crates/mycelium-std-spore/src/recon_manifest.rs:151` | Access the inner [`ReconInfo`] for callers that need the kernel representation (e.g. |
| `mycelium_std_spore::recon_manifest::ReconManifest::manifest_hash` | fn | `crates/mycelium-std-spore/src/recon_manifest.rs:139` | The content hash of the manifest, computed by hashing its canonical representation. |
| `mycelium_std_spore::recon_manifest::ReconManifest::mode` | fn | `crates/mycelium-std-spore/src/recon_manifest.rs:114` | The reconstruction mode (`IndexedRetrieval` or `CompositionalReconstruction`). |
| `mycelium_std_spore::recon_manifest::ReconManifest::new` | fn | `crates/mycelium-std-spore/src/recon_manifest.rs:63` | Build and validate a reconstruction manifest from its components. |
| `mycelium_std_spore::recon_manifest::ReconManifest::validate` | fn | `crates/mycelium-std-spore/src/recon_manifest.rs:97` | Validate an existing [`ReconInfo`] from the kernel, wrapping it as a [`ReconManifest`]. |
| `mycelium_std_spore::recon_manifest::RegrowthResult::as_approx` | fn | `crates/mycelium-std-spore/src/recon_manifest.rs:295` | Project to the stdlib's honest carrier `std.numerics::Approx<Factorization>` (FLAG Q4a — |
| `mycelium_std_spore::recon_manifest::RegrowthResult::bound` | fn | `crates/mycelium-std-spore/src/recon_manifest.rs:266` | The certificate bound (read-only — construction enforces the FR-C2 ceiling). |
| `mycelium_std_spore::recon_manifest::RegrowthResult::is_declared` | fn | `crates/mycelium-std-spore/src/recon_manifest.rs:307` | True iff the strength is `Declared` (the weakest; user-asserted only). |
| `mycelium_std_spore::recon_manifest::RegrowthResult::is_empirical` | fn | `crates/mycelium-std-spore/src/recon_manifest.rs:301` | True iff the strength is exactly `Empirical` (the expected case for the resonator path). |
| `mycelium_std_spore::recon_manifest::RegrowthResult::strength` | fn | `crates/mycelium-std-spore/src/recon_manifest.rs:277` | The honest guarantee strength — **derived** from the bound's basis (never fabricated, |
| `mycelium_std_spore::spore_ops` | mod | `crates/mycelium-std-spore/src/lib.rs:63` | — |
| `mycelium_std_spore::spore_ops::SporeUnit::from_manifest` | fn | `crates/mycelium-std-spore/src/spore_ops.rs:127` | Build a `SporeUnit` from a parsed `Manifest` and the project directory. |
| `mycelium_std_spore::spore_ops::SporeUnit::from_value` | fn | `crates/mycelium-std-spore/src/spore_ops.rs:156` | The degenerate `spore(v)` case (ADR-013 §2): build a spore whose payload is a single value |
| `mycelium_std_spore::spore_ops::SporeUnit::manifest` | fn | `crates/mycelium-std-spore/src/spore_ops.rs:240` | The reconstruction manifest, if this spore carries one. |
| `mycelium_std_spore::spore_ops::SporeUnit::raw` | fn | `crates/mycelium-std-spore/src/spore_ops.rs:246` | The raw M-368 spore (for consumers that need the full project representation). |
| `mycelium_std_spore::verify` | fn | `crates/mycelium-std-spore/src/spore_ops.rs:208` | Verify the spore: recompute the component-DAG hash and compare to the declared identity. |

## mycelium-std-swap

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_swap::CheckError` | enum | `crates/mycelium-std-swap/src/lib.rs:103` | Why a certificate check did not produce a `Validated` verdict (RFC-0002 §2). |
| `mycelium_std_swap::ExplainRecord` | struct | `crates/mycelium-std-swap/src/lib.rs:143` | A dual human/machine projection of a [`SwapCertificate`] (G11; C3). |
| `mycelium_std_swap::GUARANTEE_MATRIX:` | const | `crates/mycelium-std-swap/src/lib.rs:468` | The guarantee matrix for `std.swap` (RFC-0016 §4.5; swap.md §4). |
| `mycelium_std_swap::MatrixRow` | struct | `crates/mycelium-std-swap/src/lib.rs:448` | One row of the guarantee matrix (RFC-0016 §4.5; swap.md §4). |
| `mycelium_std_swap::PolicyRef` | type | `crates/mycelium-std-swap/src/lib.rs:59` | A PolicyRef is a [`ContentHash`] that records which swap policy was applied (RFC-0005; ADR-006). |
| `mycelium_std_swap::Swapped` | struct | `crates/mycelium-std-swap/src/lib.rs:71` | The result of a successful swap: the target **value** together with its inspectable |
| `mycelium_std_swap::assert_matrix_invariants` | fn | `crates/mycelium-std-swap/src/lib.rs:537` | Assert the structural invariants of the guarantee matrix — called from tests. |
| `mycelium_std_swap::bin_to_tern` | fn | `crates/mycelium-std-swap/src/lib.rs:245` | `bin_to_tern` — encode an `n`-bit two's-complement [`Value`] into `m` balanced trits. |
| `mycelium_std_swap::check_swap` | fn | `crates/mycelium-std-swap/src/lib.rs:378` | Validate that value `b` refines value `a` under the swap described by `cert` (M-210). |
| `mycelium_std_swap::dense_to_vsa` | fn | `crates/mycelium-std-swap/src/lib.rs:320` | `dense_to_vsa` — encode a bipolar `Dense{n, F32}` value into a `Vsa{MAP-I, vsa_dim}` |
| `mycelium_std_swap::explain` | fn | `crates/mycelium-std-swap/src/lib.rs:88` | Project the certificate to a human/machine dual EXPLAIN record (G11; C3). |
| `mycelium_std_swap::f32_to_bf16` | fn | `crates/mycelium-std-swap/src/lib.rs:295` | `f32_to_bf16` — round a `Dense{F32}` value to `Dense{BF16}` under round-to-nearest (M-211). |
| `mycelium_std_swap::tern_to_bin` | fn | `crates/mycelium-std-swap/src/lib.rs:267` | `tern_to_bin` — decode `m` balanced trits back into an `n`-bit two's-complement [`Value`]. |
| `mycelium_std_swap::vsa_to_dense` | fn | `crates/mycelium-std-swap/src/lib.rs:348` | `vsa_to_dense` — decode a `swap.dense_vsa.enc.v1` product back to a bipolar `Dense{F32}` value |

## mycelium-std-sys

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_sys::fs` | mod | `crates/mycelium-std-sys/src/lib.rs:42` | — |
| `mycelium_std_sys::fs::create_dir_all` | fn | `crates/mycelium-std-sys/src/fs.rs:41` | \[Declared\] Create a directory and all its parents. |
| `mycelium_std_sys::fs::exists` | fn | `crates/mycelium-std-sys/src/fs.rs:34` | \[Declared\] Check whether a path exists on the filesystem. |
| `mycelium_std_sys::fs::read` | fn | `crates/mycelium-std-sys/src/fs.rs:18` | \[Declared\] Read the entire contents of a file at `path`. |
| `mycelium_std_sys::fs::remove_file` | fn | `crates/mycelium-std-sys/src/fs.rs:48` | \[Declared\] Remove a file. |
| `mycelium_std_sys::fs::write` | fn | `crates/mycelium-std-sys/src/fs.rs:25` | \[Declared\] Write `contents` to a file at `path`, creating or truncating it. |
| `mycelium_std_sys::math` | mod | `crates/mycelium-std-sys/src/lib.rs:43` | — |
| `mycelium_std_sys::math::acos` | fn | `crates/mycelium-std-sys/src/math.rs:37` | \[Declared\] `acos(x)`. |
| `mycelium_std_sys::math::asin` | fn | `crates/mycelium-std-sys/src/math.rs:32` | \[Declared\] `asin(x)`. |
| `mycelium_std_sys::math::atan` | fn | `crates/mycelium-std-sys/src/math.rs:42` | \[Declared\] `atan(x)`. |
| `mycelium_std_sys::math::atan2` | fn | `crates/mycelium-std-sys/src/math.rs:47` | \[Declared\] `atan2(y, x)`. |
| `mycelium_std_sys::math::cbrt` | fn | `crates/mycelium-std-sys/src/math.rs:82` | \[Declared\] `cbrt(x)`. |
| `mycelium_std_sys::math::cos` | fn | `crates/mycelium-std-sys/src/math.rs:22` | \[Declared\] `cos(x)`. |
| `mycelium_std_sys::math::exp` | fn | `crates/mycelium-std-sys/src/math.rs:52` | \[Declared\] `exp(x)`. |
| `mycelium_std_sys::math::exp2` | fn | `crates/mycelium-std-sys/src/math.rs:57` | \[Declared\] `exp2(x)`. |
| `mycelium_std_sys::math::ln` | fn | `crates/mycelium-std-sys/src/math.rs:62` | \[Declared\] `ln(x)`. |
| `mycelium_std_sys::math::log10` | fn | `crates/mycelium-std-sys/src/math.rs:72` | \[Declared\] `log10(x)`. |
| `mycelium_std_sys::math::log2` | fn | `crates/mycelium-std-sys/src/math.rs:67` | \[Declared\] `log2(x)`. |
| `mycelium_std_sys::math::sin` | fn | `crates/mycelium-std-sys/src/math.rs:17` | \[Declared\] `sin(x)`. |
| `mycelium_std_sys::math::sqrt` | fn | `crates/mycelium-std-sys/src/math.rs:77` | \[Declared\] `sqrt(x)`. |
| `mycelium_std_sys::math::tan` | fn | `crates/mycelium-std-sys/src/math.rs:27` | \[Declared\] `tan(x)`. |
| `mycelium_std_sys::rand` | mod | `crates/mycelium-std-sys/src/lib.rs:44` | — |
| `mycelium_std_sys::rand::EntropyError` | enum | `crates/mycelium-std-sys/src/rand.rs:37` | Errors from platform entropy operations. |
| `mycelium_std_sys::rand::fill_bytes` | fn | `crates/mycelium-std-sys/src/rand.rs:70` | \[Declared\] Fill `buf` with bytes from the OS entropy source (`/dev/urandom`). |
| `mycelium_std_sys::time` | mod | `crates/mycelium-std-sys/src/lib.rs:45` | — |
| `mycelium_std_sys::time::mono_nanos` | fn | `crates/mycelium-std-sys/src/time.rs:41` | \[Declared\] Returns monotonic nanoseconds since an unspecified process-local epoch. |
| `mycelium_std_sys::time::sleep_nanos` | fn | `crates/mycelium-std-sys/src/time.rs:57` | \[Declared\] Pause the current thread for approximately `nanos` nanoseconds. |
| `mycelium_std_sys::time::wall_nanos` | fn | `crates/mycelium-std-sys/src/time.rs:26` | \[Declared\] Returns nanoseconds since the Unix epoch from the wall clock. |

## mycelium-std-ternary

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_ternary::Bit` | enum | `crates/mycelium-std-ternary/src/primitives.rs:110` | A binary digit in `{0, 1}` (FR-M2). |
| `mycelium_std_ternary::ExplainRecord` | struct | `crates/mycelium-std-ternary/src/packing.rs:119` | The inspectable EXPLAIN record attached to a packed value (C3/NFR-1/SC-3/G11). |
| `mycelium_std_ternary::PackError` | enum | `crates/mycelium-std-ternary/src/packing.rs:91` | Explicit errors returned by [`pack`] (C1/G2 — no silent failure, no sentinel). |
| `mycelium_std_ternary::Packed` | struct | `crates/mycelium-std-ternary/src/packing.rs:163` | A packed trit sequence: bytes + the inspectable `Meta.physical` scheme record (C3/C4/NFR-1). |
| `mycelium_std_ternary::Scheme` | enum | `crates/mycelium-std-ternary/src/packing.rs:38` | The packing scheme chosen at a lowering stage (RFC-0004 §5; `physical-layout.schema.json`). |
| `mycelium_std_ternary::Trit` | enum | `crates/mycelium-std-ternary/src/primitives.rs:21` | A balanced trit in `{−1, 0, +1}` (FR-M2; M-111). |
| `mycelium_std_ternary::add` | fn | `crates/mycelium-std-ternary/src/arithmetic.rs:90` | Fixed-width balanced-ternary addition `a + b`. |
| `mycelium_std_ternary::arithmetic` | mod | `crates/mycelium-std-ternary/src/lib.rs:51` | — |
| `mycelium_std_ternary::explain` | fn | `crates/mycelium-std-ternary/src/packing.rs:205` | The full EXPLAIN record for this packed value (C3/G11/NFR-1/SC-3). |
| `mycelium_std_ternary::guarantee_matrix` | mod | `crates/mycelium-std-ternary/src/lib.rs:52` | — |
| `mycelium_std_ternary::guarantee_matrix::Explainable` | enum | `crates/mycelium-std-ternary/src/guarantee_matrix.rs:52` | Whether the op exposes an inspectable artifact for its selection/conversion (C3/G11). |
| `mycelium_std_ternary::guarantee_matrix::Fallibility` | enum | `crates/mycelium-std-ternary/src/guarantee_matrix.rs:41` | Whether an op is total or returns an explicit error on some inputs. |
| `mycelium_std_ternary::guarantee_matrix::MATRIX:` | const | `crates/mycelium-std-ternary/src/guarantee_matrix.rs:78` | The complete guarantee matrix for `std.ternary` (RFC-0016 §4.5). |
| `mycelium_std_ternary::guarantee_matrix::OpGuarantee` | struct | `crates/mycelium-std-ternary/src/guarantee_matrix.rs:61` | One row of the guarantee matrix (RFC-0016 §4.5; `docs/spec/stdlib/ternary.md` §4). |
| `mycelium_std_ternary::guarantee_matrix::Tag` | enum | `crates/mycelium-std-ternary/src/guarantee_matrix.rs:28` | A guarantee-lattice tag (C2/VR-5). |
| `mycelium_std_ternary::guarantee_matrix::assert_matrix_invariants` | fn | `crates/mycelium-std-ternary/src/guarantee_matrix.rs:222` | Structural invariants on the matrix — asserted in tests. |
| `mycelium_std_ternary::int_to_trits` | fn | `crates/mycelium-std-ternary/src/arithmetic.rs:54` | The unique `m`-trit balanced representation of `value`, MSB-first. |
| `mycelium_std_ternary::max_magnitude` | fn | `crates/mycelium-std-ternary/src/arithmetic.rs:65` | The maximum representable magnitude in `m` trits: `(3^m − 1) / 2`. |
| `mycelium_std_ternary::mul` | fn | `crates/mycelium-std-ternary/src/arithmetic.rs:112` | Fixed-width balanced-ternary multiplication `a × b`. |
| `mycelium_std_ternary::neg` | fn | `crates/mycelium-std-ternary/src/arithmetic.rs:79` | Digit-wise negation of an `m`-trit balanced-ternary number. |
| `mycelium_std_ternary::pack` | fn | `crates/mycelium-std-ternary/src/packing.rs:411` | Pack a trit sequence under the given scheme. |
| `mycelium_std_ternary::packing` | mod | `crates/mycelium-std-ternary/src/lib.rs:53` | — |
| `mycelium_std_ternary::packing::Packed::bytes` | fn | `crates/mycelium-std-ternary/src/packing.rs:197` | The packed bytes, read-only (lossless re-encoding of the trits; RFC-0004 §5). |
| `mycelium_std_ternary::packing::Packed::scheme` | fn | `crates/mycelium-std-ternary/src/packing.rs:182` | The scheme used to pack these bytes (the `Meta.physical` inspectable record; C3/NFR-1). |
| `mycelium_std_ternary::packing::Packed::trit_count` | fn | `crates/mycelium-std-ternary/src/packing.rs:188` | The number of trits originally packed (total; needed for reconstructing the last group). |
| `mycelium_std_ternary::packing::Scheme::group_size` | fn | `crates/mycelium-std-ternary/src/packing.rs:64` | The alignment group size (number of trits that must be present for a complete group). |
| `mycelium_std_ternary::packing::Scheme::trits_per_byte` | fn | `crates/mycelium-std-ternary/src/packing.rs:53` | The number of trits packed per byte for this scheme. |
| `mycelium_std_ternary::packing::SelectionNote` | enum | `crates/mycelium-std-ternary/src/packing.rs:132` | How the scheme was selected (for the EXPLAIN record). |
| `mycelium_std_ternary::primitives` | mod | `crates/mycelium-std-ternary/src/lib.rs:54` | — |
| `mycelium_std_ternary::primitives::Bit::and` | fn | `crates/mycelium-std-ternary/src/primitives.rs:146` | Boolean AND. |
| `mycelium_std_ternary::primitives::Bit::digit` | fn | `crates/mycelium-std-ternary/src/primitives.rs:50` | The signed integer value of this trit: `Neg↦−1, Zero↦0, Pos↦+1`. |
| `mycelium_std_ternary::primitives::Bit::new` | fn | `crates/mycelium-std-ternary/src/primitives.rs:37` | Construct a `Trit` from an integer. |
| `mycelium_std_ternary::primitives::Bit::or` | fn | `crates/mycelium-std-ternary/src/primitives.rs:157` | Boolean OR. |
| `mycelium_std_ternary::primitives::Bit::xor` | fn | `crates/mycelium-std-ternary/src/primitives.rs:168` | Boolean XOR. |
| `mycelium_std_ternary::primitives::Trit::from_wire_char` | fn | `crates/mycelium-std-ternary/src/primitives.rs:93` | Parse a wire glyph back into a `Trit`. |
| `mycelium_std_ternary::primitives::Trit::to_wire_char` | fn | `crates/mycelium-std-ternary/src/primitives.rs:81` | The MSB-first wire glyph for this trit: `-` / `0` / `+` |
| `mycelium_std_ternary::scheme_of` | fn | `crates/mycelium-std-ternary/src/packing.rs:221` | The scheme used to pack `p` (the inspectable `Meta.physical` record; C3/NFR-1). |
| `mycelium_std_ternary::sub` | fn | `crates/mycelium-std-ternary/src/arithmetic.rs:98` | Fixed-width balanced-ternary subtraction `a − b = add(a, neg(b))`. |
| `mycelium_std_ternary::trits_to_int` | fn | `crates/mycelium-std-ternary/src/arithmetic.rs:43` | The integer denoted by an MSB-first trit string. |
| `mycelium_std_ternary::unpack` | fn | `crates/mycelium-std-ternary/src/packing.rs:443` | Unpack a [`Packed`] trit sequence back to a `Vec<Trit>`. |

## mycelium-std-testing

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_testing::Budget` | struct | `crates/mycelium-std-testing/src/lib.rs:165` | A declared, bounded trial budget for a property run (C6 — effects are bounded). |
| `mycelium_std_testing::Budget::DEFAULT:` | const | `crates/mycelium-std-testing/src/lib.rs:169` | The default budget when no specific value is required (100 trials). |
| `mycelium_std_testing::Budget::MIN:` | const | `crates/mycelium-std-testing/src/lib.rs:172` | The minimum budget (1 trial). |
| `mycelium_std_testing::Budget::trials` | fn | `crates/mycelium-std-testing/src/lib.rs:186` | The number of trials this budget permits. |
| `mycelium_std_testing::FailRecord` | struct | `crates/mycelium-std-testing/src/verdict.rs:30` | A structured failure record carried by [`Verdict::Fail`]. |
| `mycelium_std_testing::Gen` | trait | `crates/mycelium-std-testing/src/lib.rs:147` | A type that can produce values of type `T` given an `Rng`. |
| `mycelium_std_testing::GoldenBaseline` | struct | `crates/mycelium-std-testing/src/lib.rs:304` | A golden baseline: an identifier (the "name") and its expected serialized form. |
| `mycelium_std_testing::Rng` | struct | `crates/mycelium-std-testing/src/lib.rs:84` | A deterministic, seeded pseudo-random generator for property-test inputs (RT3 / C6). |
| `mycelium_std_testing::Rng::new` | fn | `crates/mycelium-std-testing/src/lib.rs:93` | Construct a generator from a fixed seed (RT3: no undeclared entropy). |
| `mycelium_std_testing::Rng::next_u32` | fn | `crates/mycelium-std-testing/src/lib.rs:116` | Advance and return a `u32`. |
| `mycelium_std_testing::Rng::next_u64` | fn | `crates/mycelium-std-testing/src/lib.rs:106` | Advance the state and return the next `u64` (Xorshift64). |
| `mycelium_std_testing::Rng::next_usize_below` | fn | `crates/mycelium-std-testing/src/lib.rs:123` | Advance and return a value in `[0, n)`. |
| `mycelium_std_testing::SkipReason` | enum | `crates/mycelium-std-testing/src/verdict.rs:67` | The reason a test was skipped (spec §3). |
| `mycelium_std_testing::Summary` | struct | `crates/mycelium-std-testing/src/verdict.rs:157` | The aggregated outcome of a collection of verdicts (spec §3 / [`crate::summarize`]). |
| `mycelium_std_testing::UndetReason` | enum | `crates/mycelium-std-testing/src/verdict.rs:94` | The reason a test result is undetermined (ran but could not decide — spec §3). |
| `mycelium_std_testing::Verdict` | enum | `crates/mycelium-std-testing/src/verdict.rs:120` | The outcome of a single test case (spec §3 / §4 guarantee matrix). |
| `mycelium_std_testing::differential` | fn | `crates/mycelium-std-testing/src/lib.rs:426` | Run a differential (oracle) test: require `lhs(input) == rhs(input)`. |
| `mycelium_std_testing::for_all` | fn | `crates/mycelium-std-testing/src/lib.rs:213` | Run a property test: generate `budget` inputs from `gen` and check `prop` for each. |
| `mycelium_std_testing::golden` | fn | `crates/mycelium-std-testing/src/lib.rs:338` | Run a golden / snapshot test: compare `produced` against the stored baseline. |
| `mycelium_std_testing::guarantee_matrix` | mod | `crates/mycelium-std-testing/src/lib.rs:67` | — |
| `mycelium_std_testing::guarantee_matrix::MATRIX:` | const | `crates/mycelium-std-testing/src/guarantee_matrix.rs:49` | The `std.testing` guarantee matrix (spec §4). |
| `mycelium_std_testing::guarantee_matrix::Row` | struct | `crates/mycelium-std-testing/src/guarantee_matrix.rs:23` | One row of the `std.testing` guarantee matrix. |
| `mycelium_std_testing::is_green` | fn | `crates/mycelium-std-testing/src/lib.rs:511` | True only if there are no failures **and** skipped/undetermined counts are surfaced (i.e., |
| `mycelium_std_testing::summarize` | fn | `crates/mycelium-std-testing/src/lib.rs:473` | Aggregate a slice of verdicts into a [`Summary`]. |
| `mycelium_std_testing::verdict` | mod | `crates/mycelium-std-testing/src/lib.rs:68` | — |
| `mycelium_std_testing::verdict::FailRecord::to_diag` | fn | `crates/mycelium-std-testing/src/verdict.rs:50` | Project this failure to the canonical [`mycelium_diag::Diag`] record (the testing↔diag |
| `mycelium_std_testing::verdict::Summary::total` | fn | `crates/mycelium-std-testing/src/verdict.rs:171` | Total number of verdicts in this summary. |

## mycelium-std-text

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_text::BoundaryError` | enum | `crates/mycelium-std-text/src/error.rs:82` | An index into a `Text` was out of range or fell on an invalid boundary. |
| `mycelium_std_text::EncodeError` | enum | `crates/mycelium-std-text/src/error.rs:222` | A character in the `Text` is not representable in the target encoding. |
| `mycelium_std_text::Lossy` | struct | `crates/mycelium-std-text/src/types.rs:164` | The **type-level opt-in** to lossy transcoding (spec §3 / C1 / G2). |
| `mycelium_std_text::ParseErr` | enum | `crates/mycelium-std-text/src/error.rs:160` | A parse failure: the input string was empty, malformed, or the lexically-valid value was |
| `mycelium_std_text::Text` | struct | `crates/mycelium-std-text/src/types.rs:37` | An immutable, UTF-8 encoded string value (spec §1 / §3). |
| `mycelium_std_text::TranscodeError` | enum | `crates/mycelium-std-text/src/error.rs:266` | A UTF-16 unit sequence is invalid (unpaired surrogate or otherwise invalid unit). |
| `mycelium_std_text::Utf8Error` | enum | `crates/mycelium-std-text/src/error.rs:43` | An invalid UTF-8 byte sequence was found at a known byte index. |
| `mycelium_std_text::char_at` | fn | `crates/mycelium-std-text/src/ops.rs:277` | Return the `char` at char index `i` (0-based codepoint index). |
| `mycelium_std_text::chars` | fn | `crates/mycelium-std-text/src/ops.rs:218` | Return a `Vec` of `char`s in `s`, in order (total). |
| `mycelium_std_text::concat` | fn | `crates/mycelium-std-text/src/ops.rs:79` | Concatenate two `Text` values (total), returning a new `Text`. |
| `mycelium_std_text::encode_utf8` | fn | `crates/mycelium-std-text/src/ops.rs:392` | Return the UTF-8 byte encoding of `s` (total — `Text` is already UTF-8). |
| `mycelium_std_text::error` | mod | `crates/mycelium-std-text/src/lib.rs:80` | — |
| `mycelium_std_text::from_chars` | fn | `crates/mycelium-std-text/src/ops.rs:43` | Construct a `Text` from a slice of `char`s (total: every char sequence is valid UTF-8). |
| `mycelium_std_text::from_utf16` | fn | `crates/mycelium-std-text/src/ops.rs:497` | Transcode a UTF-16 `u16` sequence to a `Text` (fallible). |
| `mycelium_std_text::from_utf8` | fn | `crates/mycelium-std-text/src/ops.rs:57` | Construct a `Text` from a byte slice, verifying UTF-8 validity (fallible). |
| `mycelium_std_text::guarantee_matrix` | mod | `crates/mycelium-std-text/src/lib.rs:81` | — |
| `mycelium_std_text::guarantee_matrix::MATRIX:` | const | `crates/mycelium-std-text/src/guarantee_matrix.rs:51` | The `std.text` guarantee matrix — one row per exported op, encoded as data and asserted |
| `mycelium_std_text::guarantee_matrix::MatrixRow` | struct | `crates/mycelium-std-text/src/guarantee_matrix.rs:30` | One row in the `std.text` guarantee matrix (RFC-0016 §4.5). |
| `mycelium_std_text::join` | fn | `crates/mycelium-std-text/src/ops.rs:94` | Join a slice of `Text` values with a separator (total), returning a new `Text`. |
| `mycelium_std_text::len_bytes` | fn | `crates/mycelium-std-text/src/ops.rs:159` | The length of `s` in bytes (total). |
| `mycelium_std_text::len_chars` | fn | `crates/mycelium-std-text/src/ops.rs:171` | The length of `s` in Unicode scalar values (codepoints; total). |
| `mycelium_std_text::len_graphemes` | fn | `crates/mycelium-std-text/src/ops.rs:193` | The length of `s` in Unicode grapheme clusters (total — see FLAG Q2). |
| `mycelium_std_text::ops` | mod | `crates/mycelium-std-text/src/lib.rs:82` | — |
| `mycelium_std_text::parse_bool` | fn | `crates/mycelium-std-text/src/ops.rs:367` | Parse a boolean from `s` (fallible — `Result`, **never a sentinel**). |
| `mycelium_std_text::parse_int` | fn | `crates/mycelium-std-text/src/ops.rs:310` | Parse a decimal integer from `s` (fallible — `Result`, **never a sentinel**). |
| `mycelium_std_text::replace` | fn | `crates/mycelium-std-text/src/ops.rs:145` | Return a new `Text` with every non-overlapping occurrence of `from` replaced by `to` (total). |
| `mycelium_std_text::slice` | fn | `crates/mycelium-std-text/src/ops.rs:242` | Extract the substring of `s` given by the byte range `[start, end)`, returning a new `Text`. |
| `mycelium_std_text::to_latin1` | fn | `crates/mycelium-std-text/src/ops.rs:423` | Encode `s` in Latin-1 (ISO-8859-1), strict — `Err` on any non-Latin-1 character. |
| `mycelium_std_text::to_latin1_lossy` | fn | `crates/mycelium-std-text/src/ops.rs:456` | Encode `s` in Latin-1, substituting non-Latin-1 characters with `marker` (opt-in lossy). |
| `mycelium_std_text::to_lower` | fn | `crates/mycelium-std-text/src/ops.rs:109` | Return a new `Text` with every ASCII uppercase letter mapped to lowercase (total). |
| `mycelium_std_text::to_upper` | fn | `crates/mycelium-std-text/src/ops.rs:121` | Return a new `Text` with every ASCII lowercase letter mapped to uppercase (total). |
| `mycelium_std_text::to_utf16` | fn | `crates/mycelium-std-text/src/ops.rs:405` | Transcode `s` from UTF-8 to UTF-16 (lossless; total). |
| `mycelium_std_text::trim` | fn | `crates/mycelium-std-text/src/ops.rs:133` | Return a new `Text` with leading and trailing whitespace removed (total). |
| `mycelium_std_text::types` | mod | `crates/mycelium-std-text/src/lib.rs:83` | — |
| `mycelium_std_text::types::Text::as_bytes` | fn | `crates/mycelium-std-text/src/types.rs:63` | View the internal bytes. |
| `mycelium_std_text::types::Text::as_str` | fn | `crates/mycelium-std-text/src/types.rs:57` | View the internal UTF-8 bytes as a `&str` (total, by-invariant). |
| `mycelium_std_text::types::Text::into_inner` | fn | `crates/mycelium-std-text/src/types.rs:81` | Decompose into the inner `String`, consuming the `Text`. |
| `mycelium_std_text::types::Text::is_empty` | fn | `crates/mycelium-std-text/src/types.rs:75` | Is the text empty (zero bytes)? |
| `mycelium_std_text::types::Text::len_bytes` | fn | `crates/mycelium-std-text/src/types.rs:69` | The length in bytes (C2: `Exact`; total). |
| `mycelium_std_text::types::Text::new` | fn | `crates/mycelium-std-text/src/types.rs:49` | Construct a `Text` from a `&str` slice (total — any `&str` is valid UTF-8). |

## mycelium-std-time

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_time::ClockSource` | trait | `crates/mycelium-std-time/src/lib.rs:547` | The injectable clock-source surface (C6 / RFC-0014 declared effects). |
| `mycelium_std_time::DeclaredTime` | struct | `crates/mycelium-std-time/src/lib.rs:139` | Declared-effect wrapper for a MONOTONIC or LOGICAL clock read (effect: `time`). |
| `mycelium_std_time::DeclaredTimeEntropy` | struct | `crates/mycelium-std-time/src/lib.rs:166` | Declared-effect wrapper for a WALL-CLOCK read (effect: `{ time, entropy }`). |
| `mycelium_std_time::Duration` | struct | `crates/mycelium-std-time/src/lib.rs:205` | A signed nanosecond span (C4 / RFC-0001 value-semantic). |
| `mycelium_std_time::Duration::MAX:` | const | `crates/mycelium-std-time/src/lib.rs:218` | The largest representable (most-positive) span. |
| `mycelium_std_time::Duration::MIN:` | const | `crates/mycelium-std-time/src/lib.rs:215` | The smallest representable (most-negative) span. |
| `mycelium_std_time::Duration::ZERO:` | const | `crates/mycelium-std-time/src/lib.rs:212` | The zero span. |
| `mycelium_std_time::Duration::checked_abs` | fn | `crates/mycelium-std-time/src/lib.rs:297` | Absolute value of the span. |
| `mycelium_std_time::Duration::checked_neg` | fn | `crates/mycelium-std-time/src/lib.rs:287` | Negate the span. |
| `mycelium_std_time::Duration::from_micros` | fn | `crates/mycelium-std-time/src/lib.rs:243` | Construct from microseconds. |
| `mycelium_std_time::Duration::from_millis` | fn | `crates/mycelium-std-time/src/lib.rs:235` | Construct from milliseconds. |
| `mycelium_std_time::Duration::from_secs` | fn | `crates/mycelium-std-time/src/lib.rs:227` | Construct from whole seconds. |
| `mycelium_std_time::GUARANTEE_MATRIX:` | const | `crates/mycelium-std-time/src/lib.rs:788` | The `std.time` guarantee matrix (spec §4 / RFC-0016 §4.5). |
| `mycelium_std_time::GuaranteeRow` | struct | `crates/mycelium-std-time/src/lib.rs:763` | One row of the `std.time` guarantee matrix (RFC-0016 §4.5 / spec §4). |
| `mycelium_std_time::LogicalInstant` | struct | `crates/mycelium-std-time/src/lib.rs:386` | A point on the RFC-0008 LOGICAL clock (a deterministic monotonic tick the runtime advances). |
| `mycelium_std_time::ManualClock` | struct | `crates/mycelium-std-time/src/lib.rs:668` | A [`ClockSource`] with manually-settable time values — for deterministic tests. |
| `mycelium_std_time::ManualClock::advance_mono` | fn | `crates/mycelium-std-time/src/lib.rs:702` | Advance the MONOTONIC clock by `delta_ns` nanoseconds (for tests that simulate time |
| `mycelium_std_time::ManualClock::set_logical` | fn | `crates/mycelium-std-time/src/lib.rs:696` | Set the LOGICAL tick returned by `logical_now`. |
| `mycelium_std_time::ManualClock::set_mono` | fn | `crates/mycelium-std-time/src/lib.rs:686` | Set the MONOTONIC clock value returned by `mono_now`. |
| `mycelium_std_time::ManualClock::set_wall` | fn | `crates/mycelium-std-time/src/lib.rs:691` | Set the WALL-CLOCK value returned by `wall_now`. |
| `mycelium_std_time::ManualClock::step_logical` | fn | `crates/mycelium-std-time/src/lib.rs:707` | Advance the LOGICAL clock by one tick (for tests that simulate a runtime step). |
| `mycelium_std_time::MonoInstant` | struct | `crates/mycelium-std-time/src/lib.rs:318` | A point on the MONOTONIC clock (never-backward, no civil meaning). |
| `mycelium_std_time::SystemClock` | struct | `crates/mycelium-std-time/src/lib.rs:582` | A [`ClockSource`] backed by Rust's `std::time` — the **std-sys placeholder** (FLAG §7-Q3). |
| `mycelium_std_time::TimeErr` | enum | `crates/mycelium-std-time/src/lib.rs:93` | Every explicit failure from a `std.time` operation (C1 / G2 / RFC-0013 diagnostic shape). |
| `mycelium_std_time::WallInstant` | struct | `crates/mycelium-std-time/src/lib.rs:348` | A point on the WALL-CLOCK (civil/UTC time, an entropy source). |
| `mycelium_std_time::assert_matrix_invariants` | fn | `crates/mycelium-std-time/src/lib.rs:883` | Assert the structural invariants of the guarantee matrix — called from tests. |
| `mycelium_std_time::duration_add` | fn | `crates/mycelium-std-time/src/lib.rs:414` | Add two durations. |
| `mycelium_std_time::duration_as_unit` | fn | `crates/mycelium-std-time/src/lib.rs:467` | Convert a duration to a coarser unit (truncating), or return `Err(Overflow)` if the truncated |
| `mycelium_std_time::duration_cmp` | fn | `crates/mycelium-std-time/src/lib.rs:451` | Compare two durations. |
| `mycelium_std_time::duration_scale` | fn | `crates/mycelium-std-time/src/lib.rs:442` | Scale a duration by a signed integer factor. |
| `mycelium_std_time::duration_sub` | fn | `crates/mycelium-std-time/src/lib.rs:428` | Subtract two durations (`a - b`). |
| `mycelium_std_time::logical_diff` | fn | `crates/mycelium-std-time/src/lib.rs:525` | Compute the duration between two LOGICAL instants (`later − earlier`). |
| `mycelium_std_time::logical_now` | fn | `crates/mycelium-std-time/src/lib.rs:755` | Read the LOGICAL clock from `source`. |
| `mycelium_std_time::mono_diff` | fn | `crates/mycelium-std-time/src/lib.rs:488` | Compute the signed duration between two MONOTONIC instants (`later − earlier`). |
| `mycelium_std_time::mono_now` | fn | `crates/mycelium-std-time/src/lib.rs:735` | Read the MONOTONIC clock from `source`. |
| `mycelium_std_time::wall_diff` | fn | `crates/mycelium-std-time/src/lib.rs:505` | Compute the signed duration between two WALL-CLOCK instants (`later − earlier`). |
| `mycelium_std_time::wall_now` | fn | `crates/mycelium-std-time/src/lib.rs:747` | Read the WALL-CLOCK from `source`. |

## mycelium-std-vsa

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_std_vsa::GUARANTEE_MATRIX:` | const | `crates/mycelium-std-vsa/src/matrix.rs:62` | The `std.vsa` guarantee matrix (vsa.md §4), encoded as data (RFC-0016 §4.5). |
| `mycelium_std_vsa::OpGuarantee` | struct | `crates/mycelium-std-vsa/src/matrix.rs:24` | One row of the `std.vsa` guarantee matrix (RFC-0016 §4.5 / vsa.md §4). |
| `mycelium_std_vsa::bind` | fn | `crates/mycelium-std-vsa/src/ops.rs:30` | Bind two hypervectors (associate). |
| `mycelium_std_vsa::bind_role` | fn | `crates/mycelium-std-vsa/src/ops.rs:113` | Role–filler binding: `bind(role, filler)`. |
| `mycelium_std_vsa::bundle` | fn | `crates/mycelium-std-vsa/src/ops.rs:65` | Bundle (superpose) a non-empty set of hypervectors. |
| `mycelium_std_vsa::cleanup` | fn | `crates/mycelium-std-vsa/src/ops.rs:145` | Cleanup: nearest-atom indexed retrieval against an item memory. |
| `mycelium_std_vsa::encode_seq` | fn | `crates/mycelium-std-vsa/src/encoding.rs:32` | Sequence encoding: `bundle( permute^0(items[0]), permute^1(items[1]), … )`. |
| `mycelium_std_vsa::encode_set` | fn | `crates/mycelium-std-vsa/src/encoding.rs:58` | Set encoding: `bundle(items[0], items[1], …)` — superpose atoms without positional encoding. |
| `mycelium_std_vsa::encoding` | mod | `crates/mycelium-std-vsa/src/lib.rs:46` | — |
| `mycelium_std_vsa::matrix` | mod | `crates/mycelium-std-vsa/src/lib.rs:47` | — |
| `mycelium_std_vsa::matrix::std_matrix_tag` | fn | `crates/mycelium-std-vsa/src/matrix.rs:226` | Look up a row in [`GUARANTEE_MATRIX`] by model id and op. |
| `mycelium_std_vsa::ops` | mod | `crates/mycelium-std-vsa/src/lib.rs:48` | — |
| `mycelium_std_vsa::permute` | fn | `crates/mycelium-std-vsa/src/ops.rs:77` | Permute (cyclically shift) a hypervector by `shift` positions. |
| `mycelium_std_vsa::recon` | mod | `crates/mycelium-std-vsa/src/lib.rs:49` | — |
| `mycelium_std_vsa::reconstruct_factors` | fn | `crates/mycelium-std-vsa/src/recon.rs:93` | Resonator factorization: recover the unknown factor atoms of a bind product. |
| `mycelium_std_vsa::reconstruct_role` | fn | `crates/mycelium-std-vsa/src/recon.rs:50` | Compositional reconstruction: unbind `record` by a named `role`, then clean up against |
| `mycelium_std_vsa::similarity` | fn | `crates/mycelium-std-vsa/src/ops.rs:101` | Cosine similarity of two hypervectors in `[-1, 1]`. |
| `mycelium_std_vsa::unbind` | fn | `crates/mycelium-std-vsa/src/ops.rs:44` | Unbind (recover a factor from a bind product). |
| `mycelium_std_vsa::unpermute` | fn | `crates/mycelium-std-vsa/src/ops.rs:88` | The inverse of [`permute`] by the same `shift` — exactly undoes the cyclic rotation. |

## mycelium-vsa

| Symbol | Kind | File:Line | Summary |
|---|---|---|---|
| `mycelium_vsa::Bsc` | struct | `crates/mycelium-vsa/src/bsc.rs:42` | The BSC model at a fixed dimensionality. |
| `mycelium_vsa::Cleanup` | enum | `crates/mycelium-vsa/src/resonator.rs:63` | Per-slot cleanup projection (RFC-0009 §3 / §9 Q2 / §10.3 ablation). |
| `mycelium_vsa::CleanupMemory` | struct | `crates/mycelium-vsa/src/cleanup.rs:28` | A labelled item memory at a fixed dimensionality. |
| `mycelium_vsa::DEFAULT_ENUM_BUDGET:` | const | `crates/mycelium-vsa/src/decode_select.rs:53` | The default enumeration budget: brute force is chosen when `∏ᵢ kᵢ ≤` this. |
| `mycelium_vsa::DecodeSelection` | struct | `crates/mycelium-vsa/src/decode_select.rs:142` | A reified decode-method selection result (RFC-0010): the chosen methodology, the mandatory EXPLAIN |
| `mycelium_vsa::EmpiricalProfile` | struct | `crates/mycelium-vsa/src/lib.rs:364` | A **trial-validated empirical profile**: the regime over which a crate-declared `Empirical` |
| `mycelium_vsa::EmpiricalProfile::bound` | fn | `crates/mycelium-vsa/src/lib.rs:403` | The δ bound this profile backs, with its honest `EmpiricalFit` basis (M-I3). |
| `mycelium_vsa::EmpiricalProfile::check` | fn | `crates/mycelium-vsa/src/lib.rs:382` | Check the profile's side-conditions for an op over `items` operands at `dim`; a violation |
| `mycelium_vsa::Factorization` | struct | `crates/mycelium-vsa/src/resonator.rs:196` | A clean, gate-passing factorization: the per-slot recovered atom plus its confidence/margin, and |
| `mycelium_vsa::Fhrr` | struct | `crates/mycelium-vsa/src/fhrr.rs:48` | The FHRR model at a fixed dimensionality. |
| `mycelium_vsa::Hrr` | struct | `crates/mycelium-vsa/src/hrr.rs:54` | The HRR model at a fixed dimensionality. |
| `mycelium_vsa::Init` | enum | `crates/mycelium-vsa/src/resonator.rs:85` | Initialisation strategy (RFC-0009 §9 Q1). |
| `mycelium_vsa::IterationRecord` | struct | `crates/mycelium-vsa/src/resonator.rs:167` | One sweep's decoded snapshot, for `EXPLAIN` (RFC-0009 §4 run trace / similarity trajectory). |
| `mycelium_vsa::MAPI_RESONATOR_PROFILE:` | const | `crates/mycelium-vsa/src/resonator.rs:301` | The trial-validated MAP-I regime (RFC-0009 §9 Q4 / §10.2 / §10.3). |
| `mycelium_vsa::MapB` | struct | `crates/mycelium-vsa/src/mapb.rs:46` | The MAP-B model at a fixed dimensionality. |
| `mycelium_vsa::MapI` | struct | `crates/mycelium-vsa/src/mapi.rs:20` | The MAP-I model at a fixed dimensionality. |
| `mycelium_vsa::Match` | struct | `crates/mycelium-vsa/src/cleanup.rs:14` | A cleanup hit: the recovered codebook item plus how confident the match is. |
| `mycelium_vsa::RFC0003_MATRIX:` | const | `crates/mycelium-vsa/src/matrix.rs:34` | The §4 matrix: `(model id, op, normative tag)` for every implemented model × operation. |
| `mycelium_vsa::ResonatorParams` | struct | `crates/mycelium-vsa/src/resonator.rs:95` | All resonator run parameters. |
| `mycelium_vsa::ResonatorProfile` | struct | `crates/mycelium-vsa/src/resonator.rs:209` | Trial-validated operational regime for resonator factorization (RFC-0009 §5.2 / §9 Q4). |
| `mycelium_vsa::ResonatorTrace` | struct | `crates/mycelium-vsa/src/resonator.rs:181` | The full inspectable trace + verdict — returned on **any** stop (success or error), so `EXPLAIN` |
| `mycelium_vsa::Sbc` | struct | `crates/mycelium-vsa/src/sbc.rs:36` | The SBC model: `blocks` blocks of `block_len` components (`dim = blocks · block_len`). |
| `mycelium_vsa::StopReason` | enum | `crates/mycelium-vsa/src/resonator.rs:137` | The terminal verdict of a run (RFC-0009 §3/§6). |
| `mycelium_vsa::VsaError` | enum | `crates/mycelium-vsa/src/lib.rs:71` | Why a VSA operation could not be performed — always explicit, never a silent coercion (G2). |
| `mycelium_vsa::VsaModel` | trait | `crates/mycelium-vsa/src/lib.rs:325` | A composition-style VSA model (RFC-0003 §3): the `bind`/`unbind` (+ self-inverse flag), |
| `mycelium_vsa::VsaOp` | enum | `crates/mycelium-vsa/src/lib.rs:58` | The VSA operations a model supplies (RFC-0003 §3). |
| `mycelium_vsa::bsc` | mod | `crates/mycelium-vsa/src/lib.rs:22` | — |
| `mycelium_vsa::bsc::BSC_BUNDLE_PROFILE:` | const | `crates/mycelium-vsa/src/bsc.rs:30` | The trial-validated regime backing the Value-level BSC bundle's `Empirical` δ |
| `mycelium_vsa::bsc::Bsc::bind_values` | fn | `crates/mycelium-vsa/src/bsc.rs:75` | Value-level `bind` (Exact, XOR). |
| `mycelium_vsa::bsc::Bsc::bundle_values_empirical` | fn | `crates/mycelium-vsa/src/bsc.rs:119` | Value-level **`Empirical` bundle**: majority superposition carrying the |
| `mycelium_vsa::bsc::Bsc::new` | fn | `crates/mycelium-vsa/src/bsc.rs:50` | A BSC model of dimension `dim`. |
| `mycelium_vsa::bsc::Bsc::permute_value` | fn | `crates/mycelium-vsa/src/bsc.rs:105` | Value-level `permute` (Exact). |
| `mycelium_vsa::bsc::Bsc::unbind_values` | fn | `crates/mycelium-vsa/src/bsc.rs:90` | Value-level `unbind` (Exact; XOR is self-inverse). |
| `mycelium_vsa::capacity` | mod | `crates/mycelium-vsa/src/lib.rs:23` | — |
| `mycelium_vsa::capacity::CAPACITY_CITATION:` | const | `crates/mycelium-vsa/src/capacity.rs:44` | The citation string for the MAP-I bundle capacity theorem (T0.2). |
| `mycelium_vsa::capacity::MARGIN_MU:` | const | `crates/mycelium-vsa/src/capacity.rs:21` | The illustrative margin `μ` the M-001 probe fixes (so `2/μ² = 200`). |
| `mycelium_vsa::capacity::proven_capacity_bound` | fn | `crates/mycelium-vsa/src/capacity.rs:51` | Issue a **`Proven`** capacity [`Bound`] for bundling `items` into `dim`, targeting failure |
| `mycelium_vsa::capacity::required_dim` | fn | `crates/mycelium-vsa/src/capacity.rs:26` | The cited theorem `requiredDim(m, δ) = ⌈(2/μ²)·ln(m/δ)⌉` (RFC-0003 §5). |
| `mycelium_vsa::cleanup` | mod | `crates/mycelium-vsa/src/lib.rs:24` | — |
| `mycelium_vsa::cleanup::CleanupMemory::atoms` | fn | `crates/mycelium-vsa/src/cleanup.rs:76` | The codebook atoms in index order, as `(label, atom)` pairs. |
| `mycelium_vsa::cleanup::CleanupMemory::cleanup` | fn | `crates/mycelium-vsa/src/cleanup.rs:87` | Clean up `query` against the codebook using `model`'s similarity: return the best-matching |
| `mycelium_vsa::cleanup::CleanupMemory::dim` | fn | `crates/mycelium-vsa/src/cleanup.rs:69` | Dimensionality of the stored atoms. |
| `mycelium_vsa::cleanup::CleanupMemory::insert` | fn | `crates/mycelium-vsa/src/cleanup.rs:44` | Store an atom under `label`. |
| `mycelium_vsa::cleanup::CleanupMemory::is_empty` | fn | `crates/mycelium-vsa/src/cleanup.rs:63` | Whether the memory is empty. |
| `mycelium_vsa::cleanup::CleanupMemory::len` | fn | `crates/mycelium-vsa/src/cleanup.rs:57` | Number of stored items. |
| `mycelium_vsa::cleanup::CleanupMemory::new` | fn | `crates/mycelium-vsa/src/cleanup.rs:36` | An empty memory for `dim`-dimensional atoms. |
| `mycelium_vsa::decode_method_policy` | fn | `crates/mycelium-vsa/src/decode_select.rs:70` | Build the **default decode-method policy** (RFC-0010 §4): three candidates |
| `mycelium_vsa::decode_select` | mod | `crates/mycelium-vsa/src/lib.rs:25` | — |
| `mycelium_vsa::explain_decode_method` | fn | `crates/mycelium-vsa/src/decode_select.rs:127` | The mandatory **EXPLAIN** for a decode-method choice (RFC-0005 §4), without executing the decode: |
| `mycelium_vsa::factorize` | fn | `crates/mycelium-vsa/src/resonator.rs:326` | Factorize `s` into one atom per slot of `codebooks`, running the RFC-0009 §3 loop with `params`. |
| `mycelium_vsa::fhrr` | mod | `crates/mycelium-vsa/src/lib.rs:26` | — |
| `mycelium_vsa::fhrr::FHRR_UNBIND_PROFILE:` | const | `crates/mycelium-vsa/src/fhrr.rs:26` | The trial-validated regime backing the Value-level FHRR unbind's `Empirical` δ |
| `mycelium_vsa::fhrr::Fhrr::bind_values` | fn | `crates/mycelium-vsa/src/fhrr.rs:84` | Value-level `bind` (deterministic phasor algebra). |
| `mycelium_vsa::fhrr::Fhrr::new` | fn | `crates/mycelium-vsa/src/fhrr.rs:56` | An FHRR model of dimension `dim`. |
| `mycelium_vsa::fhrr::Fhrr::unbind_values` | fn | `crates/mycelium-vsa/src/fhrr.rs:102` | Value-level **`Empirical` unbind** (the RFC-0003 §4 weak-link tag, like HRR): the decoded |
| `mycelium_vsa::hrr` | mod | `crates/mycelium-vsa/src/lib.rs:27` | — |
| `mycelium_vsa::hrr::HRR_UNBIND_PROFILE:` | const | `crates/mycelium-vsa/src/hrr.rs:42` | The trial-validated regime backing the Value-level HRR unbind's `Empirical` δ |
| `mycelium_vsa::hrr::Hrr::bind_values` | fn | `crates/mycelium-vsa/src/hrr.rs:100` | Value-level `bind` (deterministic algebra; binding is where HRR is honest — the |
| `mycelium_vsa::hrr::Hrr::new` | fn | `crates/mycelium-vsa/src/hrr.rs:62` | An HRR model of dimension `dim`. |
| `mycelium_vsa::hrr::Hrr::unbind_values` | fn | `crates/mycelium-vsa/src/hrr.rs:119` | Value-level **`Empirical` unbind** — the documented weak link (RFC-0003 §4). |
| `mycelium_vsa::mapb` | mod | `crates/mycelium-vsa/src/lib.rs:28` | — |
| `mycelium_vsa::mapb::MAPB_BUNDLE_PROFILE:` | const | `crates/mycelium-vsa/src/mapb.rs:34` | The trial-validated regime backing the Value-level MAP-B bundle's `Empirical` δ |
| `mycelium_vsa::mapb::MapB::bind_values` | fn | `crates/mycelium-vsa/src/mapb.rs:83` | Value-level `bind` (Exact). |
| `mycelium_vsa::mapb::MapB::bundle_values_empirical` | fn | `crates/mycelium-vsa/src/mapb.rs:134` | Value-level **`Empirical` bundle**: sign-rounded superposition carrying the |
| `mycelium_vsa::mapb::MapB::new` | fn | `crates/mycelium-vsa/src/mapb.rs:54` | A MAP-B model of dimension `dim`. |
| `mycelium_vsa::mapb::MapB::permute_value` | fn | `crates/mycelium-vsa/src/mapb.rs:118` | Value-level `permute` (Exact). |
| `mycelium_vsa::mapb::MapB::unbind_values` | fn | `crates/mycelium-vsa/src/mapb.rs:102` | Value-level `unbind` (Exact; self-inverse). |
| `mycelium_vsa::mapi` | mod | `crates/mycelium-vsa/src/lib.rs:29` | — |
| `mycelium_vsa::mapi::MapI::bind_values` | fn | `crates/mycelium-vsa/src/mapi.rs:94` | Value-level `bind` (Exact): `bind(a, b)` with `Derived` provenance over both inputs. |
| `mycelium_vsa::mapi::MapI::bundle_values_certified` | fn | `crates/mycelium-vsa/src/mapi.rs:135` | Value-level **certified `bundle`** (M-131): superpose `items` and attach a **`Proven`** |
| `mycelium_vsa::mapi::MapI::new` | fn | `crates/mycelium-vsa/src/mapi.rs:28` | A MAP-I model of dimension `dim`. |
| `mycelium_vsa::mapi::MapI::permute_value` | fn | `crates/mycelium-vsa/src/mapi.rs:123` | Value-level `permute` (Exact): cyclic shift by `shift`. |
| `mycelium_vsa::mapi::MapI::unbind_values` | fn | `crates/mycelium-vsa/src/mapi.rs:110` | Value-level `unbind` (Exact): recover a factor (self-inverse for MAP-I). |
| `mycelium_vsa::matrix` | mod | `crates/mycelium-vsa/src/lib.rs:30` | — |
| `mycelium_vsa::matrix_tag` | fn | `crates/mycelium-vsa/src/matrix.rs:70` | Look up the normative tag for `(model_id, op)`; `None` for a model the matrix does not cover |
| `mycelium_vsa::recon` | mod | `crates/mycelium-vsa/src/lib.rs:31` | — |
| `mycelium_vsa::reconstruct_factors` | fn | `crates/mycelium-vsa/src/recon.rs:90` | Factorize `record` — a bind product `s = x₁ ⊛ … ⊛ x_F` — into one codebook atom per slot, following |
| `mycelium_vsa::reconstruct_factors_auto` | fn | `crates/mycelium-vsa/src/decode_select.rs:164` | **Automatic factor reconstruction** (RFC-0010): select the decode methodology for `s` against |
| `mycelium_vsa::reconstruct_factors_selected` | fn | `crates/mycelium-vsa/src/recon.rs:158` | Value-level **auto-selected** factor decode (RFC-0010): like [`reconstruct_factors`], but routes |
| `mycelium_vsa::reconstruct_role` | fn | `crates/mycelium-vsa/src/recon.rs:24` | Compositionally reconstruct the filler bound under `role` inside `record`, following the |
| `mycelium_vsa::resonator` | mod | `crates/mycelium-vsa/src/lib.rs:32` | — |
| `mycelium_vsa::resonator::ResonatorParams::mapi_default` | fn | `crates/mycelium-vsa/src/resonator.rs:121` | The recommended MAP-I defaults (Hebbian bipolar cleanup, uniform superposition init, τ_lock=0.9, |
| `mycelium_vsa::resonator::ResonatorProfile::bound` | fn | `crates/mycelium-vsa/src/resonator.rs:281` | The δ bound this profile backs, with its honest `EmpiricalFit` basis (RFC-0009 §5.2). |
| `mycelium_vsa::resonator::ResonatorProfile::check` | fn | `crates/mycelium-vsa/src/resonator.rs:229` | Check the regime side-conditions for a concrete request; a violation is an explicit |
| `mycelium_vsa::sbc` | mod | `crates/mycelium-vsa/src/lib.rs:33` | — |
| `mycelium_vsa::sbc::Sbc::bind_values` | fn | `crates/mycelium-vsa/src/sbc.rs:167` | Value-level `bind`: per-block index addition; the result keeps the one-hot refinement, |
| `mycelium_vsa::sbc::Sbc::dim` | fn | `crates/mycelium-vsa/src/sbc.rs:52` | Total dimensionality. |
| `mycelium_vsa::sbc::Sbc::new` | fn | `crates/mycelium-vsa/src/sbc.rs:46` | An SBC model with `blocks` blocks of `block_len` components. |
| `mycelium_vsa::sbc::Sbc::repr` | fn | `crates/mycelium-vsa/src/sbc.rs:106` | The SBC `Repr`: the declared sparsity class is the static refinement |
| `mycelium_vsa::sbc::Sbc::unbind_values` | fn | `crates/mycelium-vsa/src/sbc.rs:179` | Value-level `unbind`: per-block index subtraction (the exact algebraic inverse of `bind`). |
| `mycelium_vsa::sbc::Sbc::value` | fn | `crates/mycelium-vsa/src/sbc.rs:126` | Construct an **`Exact`** SBC value from per-block active indices, carrying the declared |

## Flagged items

Items the heuristic could not locate (G2: never silently dropped):

| Symbol | Reason |
|---|---|
| `bool::myc_cmp` | definition not found via regex heuristic (kind='fn', name='myc_cmp') — possibly macro-generated or cfg-gated |
| `bool::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `bool::myc_ge` | definition not found via regex heuristic (kind='fn', name='myc_ge') — possibly macro-generated or cfg-gated |
| `bool::myc_gt` | definition not found via regex heuristic (kind='fn', name='myc_gt') — possibly macro-generated or cfg-gated |
| `bool::myc_le` | definition not found via regex heuristic (kind='fn', name='myc_le') — possibly macro-generated or cfg-gated |
| `bool::myc_lt` | definition not found via regex heuristic (kind='fn', name='myc_lt') — possibly macro-generated or cfg-gated |
| `bool::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `bool::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
| `bool::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `bool::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `bool::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `bool::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `char::myc_cmp` | definition not found via regex heuristic (kind='fn', name='myc_cmp') — possibly macro-generated or cfg-gated |
| `char::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `char::myc_ge` | definition not found via regex heuristic (kind='fn', name='myc_ge') — possibly macro-generated or cfg-gated |
| `char::myc_gt` | definition not found via regex heuristic (kind='fn', name='myc_gt') — possibly macro-generated or cfg-gated |
| `char::myc_le` | definition not found via regex heuristic (kind='fn', name='myc_le') — possibly macro-generated or cfg-gated |
| `char::myc_lt` | definition not found via regex heuristic (kind='fn', name='myc_lt') — possibly macro-generated or cfg-gated |
| `char::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `char::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
| `core::cmp::Ordering::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `f32::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `f32::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `f32::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
| `f32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `f32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `f32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `f32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `f32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `f32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `f32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `f32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `f32::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `f64::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `f64::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `f64::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
| `f64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `f64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `f64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `f64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `f64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `f64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `f64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `f64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `f64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i128::myc_cmp` | definition not found via regex heuristic (kind='fn', name='myc_cmp') — possibly macro-generated or cfg-gated |
| `i128::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `i128::myc_ge` | definition not found via regex heuristic (kind='fn', name='myc_ge') — possibly macro-generated or cfg-gated |
| `i128::myc_gt` | definition not found via regex heuristic (kind='fn', name='myc_gt') — possibly macro-generated or cfg-gated |
| `i128::myc_le` | definition not found via regex heuristic (kind='fn', name='myc_le') — possibly macro-generated or cfg-gated |
| `i128::myc_lt` | definition not found via regex heuristic (kind='fn', name='myc_lt') — possibly macro-generated or cfg-gated |
| `i128::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `i128::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
| `i128::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i128::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i128::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i128::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i16::myc_cmp` | definition not found via regex heuristic (kind='fn', name='myc_cmp') — possibly macro-generated or cfg-gated |
| `i16::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `i16::myc_ge` | definition not found via regex heuristic (kind='fn', name='myc_ge') — possibly macro-generated or cfg-gated |
| `i16::myc_gt` | definition not found via regex heuristic (kind='fn', name='myc_gt') — possibly macro-generated or cfg-gated |
| `i16::myc_le` | definition not found via regex heuristic (kind='fn', name='myc_le') — possibly macro-generated or cfg-gated |
| `i16::myc_lt` | definition not found via regex heuristic (kind='fn', name='myc_lt') — possibly macro-generated or cfg-gated |
| `i16::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `i16::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
| `i16::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i16::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i16::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i16::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i16::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i16::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `i16::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `i16::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `i32::myc_cmp` | definition not found via regex heuristic (kind='fn', name='myc_cmp') — possibly macro-generated or cfg-gated |
| `i32::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `i32::myc_ge` | definition not found via regex heuristic (kind='fn', name='myc_ge') — possibly macro-generated or cfg-gated |
| `i32::myc_gt` | definition not found via regex heuristic (kind='fn', name='myc_gt') — possibly macro-generated or cfg-gated |
| `i32::myc_le` | definition not found via regex heuristic (kind='fn', name='myc_le') — possibly macro-generated or cfg-gated |
| `i32::myc_lt` | definition not found via regex heuristic (kind='fn', name='myc_lt') — possibly macro-generated or cfg-gated |
| `i32::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `i32::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
| `i32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i32::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `i32::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `i64::myc_cmp` | definition not found via regex heuristic (kind='fn', name='myc_cmp') — possibly macro-generated or cfg-gated |
| `i64::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `i64::myc_ge` | definition not found via regex heuristic (kind='fn', name='myc_ge') — possibly macro-generated or cfg-gated |
| `i64::myc_gt` | definition not found via regex heuristic (kind='fn', name='myc_gt') — possibly macro-generated or cfg-gated |
| `i64::myc_le` | definition not found via regex heuristic (kind='fn', name='myc_le') — possibly macro-generated or cfg-gated |
| `i64::myc_lt` | definition not found via regex heuristic (kind='fn', name='myc_lt') — possibly macro-generated or cfg-gated |
| `i64::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `i64::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
| `i64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i64::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `i8::myc_cmp` | definition not found via regex heuristic (kind='fn', name='myc_cmp') — possibly macro-generated or cfg-gated |
| `i8::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `i8::myc_ge` | definition not found via regex heuristic (kind='fn', name='myc_ge') — possibly macro-generated or cfg-gated |
| `i8::myc_gt` | definition not found via regex heuristic (kind='fn', name='myc_gt') — possibly macro-generated or cfg-gated |
| `i8::myc_le` | definition not found via regex heuristic (kind='fn', name='myc_le') — possibly macro-generated or cfg-gated |
| `i8::myc_lt` | definition not found via regex heuristic (kind='fn', name='myc_lt') — possibly macro-generated or cfg-gated |
| `i8::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `i8::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
| `i8::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i8::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i8::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i8::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `i8::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `i8::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `i8::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `i8::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `isize::myc_cmp` | definition not found via regex heuristic (kind='fn', name='myc_cmp') — possibly macro-generated or cfg-gated |
| `isize::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `isize::myc_ge` | definition not found via regex heuristic (kind='fn', name='myc_ge') — possibly macro-generated or cfg-gated |
| `isize::myc_gt` | definition not found via regex heuristic (kind='fn', name='myc_gt') — possibly macro-generated or cfg-gated |
| `isize::myc_le` | definition not found via regex heuristic (kind='fn', name='myc_le') — possibly macro-generated or cfg-gated |
| `isize::myc_lt` | definition not found via regex heuristic (kind='fn', name='myc_lt') — possibly macro-generated or cfg-gated |
| `isize::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `isize::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_bench::backend::Backend` | dedup-alias: same definition as `mycelium_bench::Backend` at crates/mycelium-bench/src/backend.rs:21 — one canonical row kept |
| `mycelium_bench::backend::Backend::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::backend::Backend::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::backend::Backend::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_bench::backend::Backend::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_bench::backend::Backend::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::backend::Backend::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::backend::Backend::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_bench::backend::Backend::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_bench::backend::Backend::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::backend::Backend::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::backend::Engines` | dedup-alias: same definition as `mycelium_bench::Engines` at crates/mycelium-bench/src/backend.rs:165 — one canonical row kept |
| `mycelium_bench::backend::Engines::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_bench::backend::Engines::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_bench::backend::Outcome` | dedup-alias: same definition as `mycelium_bench::Outcome` at crates/mycelium-bench/src/backend.rs:80 — one canonical row kept |
| `mycelium_bench::backend::Outcome::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::backend::Outcome::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::backend::Outcome::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::backend::Outcome::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::Case` | dedup-alias: same definition as `mycelium_bench::Case` at crates/mycelium-bench/src/corpus.rs:55 — one canonical row kept |
| `mycelium_bench::corpus::Case::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::Case::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::Case::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::Case::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::CorpusError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::CorpusError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::Fragment` | dedup-alias: same definition as `mycelium_bench::Fragment` at crates/mycelium-bench/src/corpus.rs:28 — one canonical row kept |
| `mycelium_bench::corpus::Fragment::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::Fragment::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::Fragment::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::Fragment::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::Fragment::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::Fragment::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::Fragment::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::Fragment::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::Fragment::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::Fragment::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::Fragment::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::Fragment::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::corpus::corpus` | dedup-alias: same definition as `mycelium_bench::corpus` at crates/mycelium-bench/src/corpus.rs:104 — one canonical row kept |
| `mycelium_bench::llm::GrokAblation::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokAblation::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokAblation::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokHonestyPosture::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokHonestyPosture::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokHonestyPosture::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokLlmReport` | dedup-alias: same definition as `mycelium_bench::GrokLlmReport` at crates/mycelium-bench/src/llm.rs:243 — one canonical row kept |
| `mycelium_bench::llm::GrokLlmReport::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokLlmReport::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokLlmReport::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokLlmReport::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokLlmReport::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokLlmReport::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokLlmReport::from_json` | dedup-alias: same definition as `mycelium_bench::llm::LlmReport::from_json` at crates/mycelium-bench/src/llm.rs:178 — one canonical row kept |
| `mycelium_bench::llm::GrokLlmReport::from_json` | dedup-alias: same definition as `mycelium_bench::llm::LlmReport::from_json` at crates/mycelium-bench/src/llm.rs:178 — one canonical row kept |
| `mycelium_bench::llm::GrokLlmReport::is_synthetic` | dedup-alias: same definition as `mycelium_bench::llm::LlmReport::is_synthetic` at crates/mycelium-bench/src/llm.rs:150 — one canonical row kept |
| `mycelium_bench::llm::GrokLlmReport::is_synthetic` | dedup-alias: same definition as `mycelium_bench::llm::LlmReport::is_synthetic` at crates/mycelium-bench/src/llm.rs:150 — one canonical row kept |
| `mycelium_bench::llm::GrokLlmReport::provenance` | dedup-alias: same definition as `mycelium_bench::llm::LlmReport::provenance` at crates/mycelium-bench/src/llm.rs:156 — one canonical row kept |
| `mycelium_bench::llm::GrokLlmReport::provenance` | dedup-alias: same definition as `mycelium_bench::llm::LlmReport::provenance` at crates/mycelium-bench/src/llm.rs:156 — one canonical row kept |
| `mycelium_bench::llm::GrokMetadata::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokMetadata::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokMetadata::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokOutcome::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokOutcome::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokOutcome::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokPerformance::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokPerformance::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokPerformance::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokQuality::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokQuality::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::GrokQuality::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::HonestyPosture::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::HonestyPosture::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::HonestyPosture::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::LlmIngestError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::LlmIngestError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::LlmReport` | dedup-alias: same definition as `mycelium_bench::LlmReport` at crates/mycelium-bench/src/llm.rs:38 — one canonical row kept |
| `mycelium_bench::llm::LlmReport::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::LlmReport::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::LlmReport::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::LlmReport::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::LlmReport::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::LlmReport::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::ParsedLlmSection` | dedup-alias: same definition as `mycelium_bench::ParsedLlmSection` at crates/mycelium-bench/src/llm.rs:461 — one canonical row kept |
| `mycelium_bench::llm::ParsedLlmSection::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::ParsedLlmSection::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::Summary::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::Summary::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::Summary::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::ValidationResult::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::ValidationResult::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::ValidationResult::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::llm::parse_any_llm_json` | dedup-alias: same definition as `mycelium_bench::parse_any_llm_json` at crates/mycelium-bench/src/llm.rs:485 — one canonical row kept |
| `mycelium_bench::measure::BackendResult::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::measure::BackendResult::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::measure::BackendResult::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::measure::CaseRecord` | dedup-alias: same definition as `mycelium_bench::CaseRecord` at crates/mycelium-bench/src/measure.rs:58 — one canonical row kept |
| `mycelium_bench::measure::CaseRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::measure::CaseRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::measure::CaseRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::measure::CaseRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::measure::CaseRecord::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::measure::CaseRecord::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::measure::RunRecord` | dedup-alias: same definition as `mycelium_bench::RunRecord` at crates/mycelium-bench/src/measure.rs:75 — one canonical row kept |
| `mycelium_bench::measure::RunRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::measure::RunRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::measure::RunRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::measure::RunRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::measure::RunRecord::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::measure::RunRecord::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::measure::run_corpus` | dedup-alias: same definition as `mycelium_bench::run_corpus` at crates/mycelium-bench/src/measure.rs:146 — one canonical row kept |
| `mycelium_bench::report::Honesty` | dedup-alias: same definition as `mycelium_bench::Honesty` at crates/mycelium-bench/src/report.rs:42 — one canonical row kept |
| `mycelium_bench::report::Honesty::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Honesty::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Honesty::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Honesty::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Honesty::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Honesty::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Honesty::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Honesty::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::LlmSection` | dedup-alias: same definition as `mycelium_bench::LlmSection` at crates/mycelium-bench/src/report.rs:64 — one canonical row kept |
| `mycelium_bench::report::LlmSection::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::LlmSection::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::LlmSection::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::LlmSection::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::LlmSection::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::LlmSection::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::LlmValidationRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::LlmValidationRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::LlmValidationRow::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::LossRollup::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::LossRollup::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::LossRollup::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::LossRollup::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Report` | dedup-alias: same definition as `mycelium_bench::Report` at crates/mycelium-bench/src/report.rs:21 — one canonical row kept |
| `mycelium_bench::report::Report::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Report::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Report::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Report::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Report::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Report::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Tallies::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Tallies::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Tallies::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::report::Tallies::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::timing::Timing::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::timing::Timing::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_bench::timing::Timing::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::timing::Timing::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::Speed` | dedup-alias: same definition as `mycelium_bench::Speed` at crates/mycelium-bench/src/verdict.rs:25 — one canonical row kept |
| `mycelium_bench::verdict::Speed::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::Speed::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::Speed::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::Speed::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::Speed::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::Speed::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::Speed::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::Speed::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::Verdict` | dedup-alias: same definition as `mycelium_bench::Verdict` at crates/mycelium-bench/src/verdict.rs:37 — one canonical row kept |
| `mycelium_bench::verdict::Verdict::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::Verdict::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::Verdict::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::Verdict::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::Verdict::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::Verdict::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::Verdict::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::Verdict::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_bench::verdict::classify` | dedup-alias: same definition as `mycelium_bench::classify` at crates/mycelium-bench/src/verdict.rs:160 — one canonical row kept |
| `mycelium_build::BuildCertificate::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::BuildCertificate::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::BuildCertificate::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::BuildCertificate::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::BuildCertificate::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::Component::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::Component::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::Component::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::Eligibility::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::Eligibility::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::Eligibility::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::ExecutionRoute::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::ExecutionRoute::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::ExecutionRoute::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::ExecutionRoute::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::ExecutionRoute::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::Obligations::all` | definition not found via regex heuristic (kind='fn', name='all') — possibly macro-generated or cfg-gated |
| `mycelium_build::Obligations::all_discharged` | definition not found via regex heuristic (kind='fn', name='all_discharged') — possibly macro-generated or cfg-gated |
| `mycelium_build::Obligations::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::Obligations::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::Obligations::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::Obligations::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::Obligations::none` | definition not found via regex heuristic (kind='fn', name='none') — possibly macro-generated or cfg-gated |
| `mycelium_build::Obligations::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::cache::BuildCache` | dedup-alias: same definition as `mycelium_build::BuildCache` at crates/mycelium-build/src/cache.rs:43 — one canonical row kept |
| `mycelium_build::cache::BuildCache::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::cache::BuildCache::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::cache::BuildCache::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_build::cache::BuildCache::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_build::cache::BuildCache::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::cache::BuildCache::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::cache::CacheOutcome` | dedup-alias: same definition as `mycelium_build::CacheOutcome` at crates/mycelium-build/src/cache.rs:18 — one canonical row kept |
| `mycelium_build::cache::CacheOutcome::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::cache::CacheOutcome::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::cache::CacheOutcome::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::cache::CacheOutcome::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::cache::CacheOutcome::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::cache::CacheOutcome::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Arch` | dedup-alias: same definition as `mycelium_build::Arch` at crates/mycelium-build/src/target.rs:42 — one canonical row kept |
| `mycelium_build::target::Arch::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Arch::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Arch::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Arch::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Arch::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Arch::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Arch::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Arch::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Arch::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Arch::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Arch::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Arch::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Arch::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Arch::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Arch::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Arch::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::BuildError` | dedup-alias: same definition as `mycelium_build::BuildError` at crates/mycelium-build/src/target.rs:157 — one canonical row kept |
| `mycelium_build::target::BuildError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::BuildError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::BuildError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::BuildError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::BuildError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::BuildError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::BuildError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::BuildError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::BuildProfile` | dedup-alias: same definition as `mycelium_build::BuildProfile` at crates/mycelium-build/src/target.rs:124 — one canonical row kept |
| `mycelium_build::target::BuildProfile::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::BuildProfile::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::BuildProfile::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::BuildProfile::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::BuildProfile::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::BuildProfile::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::DispatchMiss` | dedup-alias: same definition as `mycelium_build::DispatchMiss` at crates/mycelium-build/src/target.rs:229 — one canonical row kept |
| `mycelium_build::target::DispatchMiss::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::DispatchMiss::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::DispatchMiss::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::DispatchMiss::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::DispatchMiss::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::DispatchMiss::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::DispatchMiss::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::DispatchMiss::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Os` | dedup-alias: same definition as `mycelium_build::Os` at crates/mycelium-build/src/target.rs:31 — one canonical row kept |
| `mycelium_build::target::Os::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Os::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Os::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Os::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Os::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Os::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Os::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Os::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Os::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Os::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Os::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Os::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Os::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Os::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Os::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Os::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target` | dedup-alias: same definition as `mycelium_build::Target` at crates/mycelium-build/src/target.rs:53 — one canonical row kept |
| `mycelium_build::target::Target::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::Target::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::VariantTable` | dedup-alias: same definition as `mycelium_build::VariantTable` at crates/mycelium-build/src/target.rs:221 — one canonical row kept |
| `mycelium_build::target::VariantTable::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::VariantTable::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::VariantTable::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::VariantTable::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::VariantTable::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::VariantTable::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::VariantTable::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::VariantTable::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::VariantTable::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::VariantTable::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::VariantTable::new` | dedup-alias: same definition as `mycelium_build::target::Target::new` at crates/mycelium-build/src/target.rs:63 — one canonical row kept |
| `mycelium_build::target::VariantTable::new` | dedup-alias: same definition as `mycelium_build::target::Target::new` at crates/mycelium-build/src/target.rs:63 — one canonical row kept |
| `mycelium_build::target::VariantTable::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::VariantTable::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_build::target::VariantTable::targets` | dedup-alias: same definition as `mycelium_build::target::BuildProfile::targets` at crates/mycelium-build/src/target.rs:139 — one canonical row kept |
| `mycelium_build::target::VariantTable::targets` | dedup-alias: same definition as `mycelium_build::target::BuildProfile::targets` at crates/mycelium-build/src/target.rs:139 — one canonical row kept |
| `mycelium_build::target::realizable_targets` | dedup-alias: same definition as `mycelium_build::realizable_targets` at crates/mycelium-build/src/target.rs:197 — one canonical row kept |
| `mycelium_build::target::supported_targets` | dedup-alias: same definition as `mycelium_build::supported_targets` at crates/mycelium-build/src/target.rs:111 — one canonical row kept |
| `mycelium_cert::BinTernParams::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_cert::BinTernParams::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_cert::BinTernParams::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_cert::BinTernParams::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_cert::BinTernParams::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_cert::BinaryTernarySwapEngine::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_cert::BinaryTernarySwapEngine::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_cert::BinaryTernarySwapEngine::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_cert::BinaryTernarySwapEngine::swap` | definition not found via regex heuristic (kind='fn', name='swap') — possibly macro-generated or cfg-gated |
| `mycelium_cert::CertifiedSwapEngine::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_cert::CertifiedSwapEngine::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_cert::CertifiedSwapEngine::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_cert::CertifiedSwapEngine::swap` | definition not found via regex heuristic (kind='fn', name='swap') — possibly macro-generated or cfg-gated |
| `mycelium_cert::SwapCertificate::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_cert::SwapCertificate::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_cert::SwapCertificate::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_cert::SwapCertificate::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_cert::SwapCertificate::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_cert::SwapError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_cert::SwapError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_cert::SwapError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_cert::SwapError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::CheckVerdict` | dedup-alias: same definition as `mycelium_cert::CheckVerdict` at crates/mycelium-cert/src/check.rs:110 — one canonical row kept |
| `mycelium_cert::check::CheckVerdict::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::CheckVerdict::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::CheckVerdict::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::CheckVerdict::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::CheckVerdict::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::CheckVerdict::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::Evidence` | definition not found via regex heuristic (kind='fn', name='Evidence') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::Evidence` | definition not found via regex heuristic (kind='fn', name='Evidence') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::Evidence` | definition not found via regex heuristic (kind='fn', name='Evidence') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::Evidence` | definition not found via regex heuristic (kind='fn', name='Evidence') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::Evidence` | definition not found via regex heuristic (kind='fn', name='Evidence') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::Evidence` | definition not found via regex heuristic (kind='fn', name='Evidence') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::Evidence` | dedup-alias: same definition as `mycelium_cert::Evidence` at crates/mycelium-cert/src/check.rs:58 — one canonical row kept |
| `mycelium_cert::check::Fallback` | dedup-alias: same definition as `mycelium_cert::Fallback` at crates/mycelium-cert/src/check.rs:69 — one canonical row kept |
| `mycelium_cert::check::Fallback::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::Fallback::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::Fallback::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::Fallback::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::Fallback::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::Fallback::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::NotValidatedReason` | dedup-alias: same definition as `mycelium_cert::NotValidatedReason` at crates/mycelium-cert/src/check.rs:79 — one canonical row kept |
| `mycelium_cert::check::NotValidatedReason::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::NotValidatedReason::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::NotValidatedReason::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::NotValidatedReason::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::NotValidatedReason::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::NotValidatedReason::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::RefinementRelation` | dedup-alias: same definition as `mycelium_cert::RefinementRelation` at crates/mycelium-cert/src/check.rs:45 — one canonical row kept |
| `mycelium_cert::check::RefinementRelation::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::RefinementRelation::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::RefinementRelation::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::RefinementRelation::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::RefinementRelation::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::RefinementRelation::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_cert::check::check` | dedup-alias: same definition as `mycelium_cert::check` at crates/mycelium-cert/src/check.rs:158 — one canonical row kept |
| `mycelium_cert::check::check_core` | dedup-alias: same definition as `mycelium_cert::check_core` at crates/mycelium-cert/src/check.rs:293 — one canonical row kept |
| `mycelium_cert::dense::BF16_MIN_NORMAL:` | dedup-alias: same definition as `mycelium_cert::BF16_MIN_NORMAL:` at crates/mycelium-cert/src/dense.rs:34 — one canonical row kept |
| `mycelium_cert::dense::BF16_REL_EPS:` | dedup-alias: same definition as `mycelium_cert::BF16_REL_EPS:` at crates/mycelium-cert/src/dense.rs:30 — one canonical row kept |
| `mycelium_cert::dense::dense_f32_to_bf16` | dedup-alias: same definition as `mycelium_cert::dense_f32_to_bf16` at crates/mycelium-cert/src/dense.rs:81 — one canonical row kept |
| `mycelium_cert::dense_vsa::DENSE_VSA_EMP_DELTA:` | dedup-alias: same definition as `mycelium_cert::DENSE_VSA_EMP_DELTA:` at crates/mycelium-cert/src/dense_vsa.rs:48 — one canonical row kept |
| `mycelium_cert::dense_vsa::DENSE_VSA_MODEL:` | dedup-alias: same definition as `mycelium_cert::DENSE_VSA_MODEL:` at crates/mycelium-cert/src/dense_vsa.rs:41 — one canonical row kept |
| `mycelium_cert::dense_vsa::dense_to_vsa` | dedup-alias: same definition as `mycelium_cert::dense_to_vsa` at crates/mycelium-cert/src/dense_vsa.rs:139 — one canonical row kept |
| `mycelium_cert::dense_vsa::vsa_to_dense` | dedup-alias: same definition as `mycelium_cert::vsa_to_dense` at crates/mycelium-cert/src/dense_vsa.rs:195 — one canonical row kept |
| `mycelium_check::Finding::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_check::Finding::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_check::Finding::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_check::FindingKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_check::FindingKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_check::FindingKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_check::Report::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_check::Report::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_check::Report::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_check::ResolveError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_check::ResolveError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_check::ResolveError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_check::ResolveError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_cli_common::ReadError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_cli_common::ReadError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_cli_common::ReadError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::WfError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::WfError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::WfError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::WfError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::Bound` | dedup-alias: same definition as `mycelium_core::Bound` at crates/mycelium-core/src/bound.rs:105 — one canonical row kept |
| `mycelium_core::bound::Bound::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::Bound::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::Bound::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::Bound::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::Bound::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::Bound::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::Bound::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::Bound::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::Bound::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::Bound::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundBasis` | dedup-alias: same definition as `mycelium_core::BoundBasis` at crates/mycelium-core/src/bound.rs:18 — one canonical row kept |
| `mycelium_core::bound::BoundBasis::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundBasis::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundBasis::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundBasis::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundBasis::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundBasis::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundBasis::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundBasis::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundBasis::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundBasis::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundKind` | dedup-alias: same definition as `mycelium_core::BoundKind` at crates/mycelium-core/src/bound.rs:68 — one canonical row kept |
| `mycelium_core::bound::BoundKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundKind::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundKind::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundKind::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::BoundKind::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::NormKind` | dedup-alias: same definition as `mycelium_core::NormKind` at crates/mycelium-core/src/bound.rs:53 — one canonical row kept |
| `mycelium_core::bound::NormKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::NormKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::NormKind::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::NormKind::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::NormKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::NormKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::NormKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::NormKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::NormKind::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::bound::NormKind::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::content::Names` | dedup-alias: same definition as `mycelium_core::Names` at crates/mycelium-core/src/content.rs:453 — one canonical row kept |
| `mycelium_core::content::Names::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::content::Names::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::content::Names::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_core::content::Names::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_core::content::Names::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::content::Names::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::content::Names::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::content::Names::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::content::operation_hash` | dedup-alias: same definition as `mycelium_core::operation_hash` at crates/mycelium-core/src/content.rs:442 — one canonical row kept |
| `mycelium_core::data::CtorDecl` | dedup-alias: same definition as `mycelium_core::CtorDecl` at crates/mycelium-core/src/data.rs:81 — one canonical row kept |
| `mycelium_core::data::CtorDecl::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorDecl::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorDecl::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorDecl::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorDecl::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorDecl::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorRef` | dedup-alias: same definition as `mycelium_core::CtorRef` at crates/mycelium-core/src/data.rs:37 — one canonical row kept |
| `mycelium_core::data::CtorRef::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorRef::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorRef::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorRef::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorRef::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorRef::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorRef::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorRef::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorRef::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorRef::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorRef::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorRef::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorRef::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorRef::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorSpec` | dedup-alias: same definition as `mycelium_core::CtorSpec` at crates/mycelium-core/src/data.rs:108 — one canonical row kept |
| `mycelium_core::data::CtorSpec::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorSpec::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorSpec::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorSpec::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorSpec::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::CtorSpec::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DataDecl` | dedup-alias: same definition as `mycelium_core::DataDecl` at crates/mycelium-core/src/data.rs:89 — one canonical row kept |
| `mycelium_core::data::DataDecl::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DataDecl::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DataDecl::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DataDecl::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DataDecl::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DataDecl::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DataRegistry` | dedup-alias: same definition as `mycelium_core::DataRegistry` at crates/mycelium-core/src/data.rs:152 — one canonical row kept |
| `mycelium_core::data::DataRegistry::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DataRegistry::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DataRegistry::decl` | dedup-alias: same definition as `mycelium_core::data::CtorRef::decl` at crates/mycelium-core/src/data.rs:51 — one canonical row kept |
| `mycelium_core::data::DataRegistry::decl` | dedup-alias: same definition as `mycelium_core::data::CtorRef::decl` at crates/mycelium-core/src/data.rs:51 — one canonical row kept |
| `mycelium_core::data::DataRegistry::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DataRegistry::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DataRegistry::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DataRegistry::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DataRegistry::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DataRegistry::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DeclSpec` | dedup-alias: same definition as `mycelium_core::DeclSpec` at crates/mycelium-core/src/data.rs:115 — one canonical row kept |
| `mycelium_core::data::DeclSpec::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DeclSpec::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DeclSpec::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DeclSpec::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DeclSpec::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::DeclSpec::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::FieldSpec` | dedup-alias: same definition as `mycelium_core::FieldSpec` at crates/mycelium-core/src/data.rs:99 — one canonical row kept |
| `mycelium_core::data::FieldSpec::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::FieldSpec::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::FieldSpec::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::FieldSpec::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::FieldSpec::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::FieldSpec::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::FieldTy` | dedup-alias: same definition as `mycelium_core::FieldTy` at crates/mycelium-core/src/data.rs:72 — one canonical row kept |
| `mycelium_core::data::FieldTy::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::FieldTy::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::FieldTy::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::FieldTy::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::FieldTy::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::FieldTy::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::RegistryError` | dedup-alias: same definition as `mycelium_core::RegistryError` at crates/mycelium-core/src/data.rs:122 — one canonical row kept |
| `mycelium_core::data::RegistryError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::RegistryError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::RegistryError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::RegistryError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::RegistryError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::RegistryError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::RegistryError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::data::RegistryError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::CoreValue` | dedup-alias: same definition as `mycelium_core::CoreValue` at crates/mycelium-core/src/datum.rs:90 — one canonical row kept |
| `mycelium_core::datum::CoreValue::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::CoreValue::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::CoreValue::content_hash` | dedup-alias: same definition as `mycelium_core::datum::Datum::content_hash` at crates/mycelium-core/src/datum.rs:80 — one canonical row kept |
| `mycelium_core::datum::CoreValue::content_hash` | dedup-alias: same definition as `mycelium_core::datum::Datum::content_hash` at crates/mycelium-core/src/datum.rs:80 — one canonical row kept |
| `mycelium_core::datum::CoreValue::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::CoreValue::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::CoreValue::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::CoreValue::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::CoreValue::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::CoreValue::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::CoreValue::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::CoreValue::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::CoreValue::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::CoreValue::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::CoreValue::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::CoreValue::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::CoreValue::guarantee` | dedup-alias: same definition as `mycelium_core::datum::Datum::guarantee` at crates/mycelium-core/src/datum.rs:64 — one canonical row kept |
| `mycelium_core::datum::CoreValue::guarantee` | dedup-alias: same definition as `mycelium_core::datum::Datum::guarantee` at crates/mycelium-core/src/datum.rs:64 — one canonical row kept |
| `mycelium_core::datum::Datum` | dedup-alias: same definition as `mycelium_core::Datum` at crates/mycelium-core/src/datum.rs:29 — one canonical row kept |
| `mycelium_core::datum::Datum::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::Datum::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::Datum::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::Datum::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::Datum::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::datum::Datum::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::guarantee::GuaranteeStrength` | dedup-alias: same definition as `mycelium_core::GuaranteeStrength` at crates/mycelium-core/src/guarantee.rs:16 — one canonical row kept |
| `mycelium_core::guarantee::GuaranteeStrength::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::guarantee::GuaranteeStrength::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::guarantee::GuaranteeStrength::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::guarantee::GuaranteeStrength::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::guarantee::GuaranteeStrength::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::guarantee::GuaranteeStrength::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::guarantee::GuaranteeStrength::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::guarantee::GuaranteeStrength::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::guarantee::GuaranteeStrength::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::guarantee::GuaranteeStrength::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::id::ContentHash` | dedup-alias: same definition as `mycelium_core::ContentHash` at crates/mycelium-core/src/id.rs:9 — one canonical row kept |
| `mycelium_core::id::ContentHash::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::id::ContentHash::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::id::ContentHash::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_core::id::ContentHash::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_core::id::ContentHash::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::id::ContentHash::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::id::ContentHash::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::id::ContentHash::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::id::ContentHash::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::id::ContentHash::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::id::ContentHash::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_core::id::ContentHash::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_core::id::ContentHash::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_core::id::ContentHash::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_core::id::ContentHash::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::id::ContentHash::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::Anf::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::Anf::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::Anf::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::AnfAlt::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::AnfAlt::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::AnfAlt::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::Atom::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::Atom::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::Atom::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::Atom::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::Binding::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::Binding::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::Binding::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::Rhs::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::Rhs::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::Rhs::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::Stage::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::Stage::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::lower::Stage::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Meta` | dedup-alias: same definition as `mycelium_core::Meta` at crates/mycelium-core/src/meta.rs:88 — one canonical row kept |
| `mycelium_core::meta::Meta::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Meta::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Meta::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Meta::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Meta::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Meta::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Meta::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Meta::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Meta::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Meta::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PackScheme` | dedup-alias: same definition as `mycelium_core::PackScheme` at crates/mycelium-core/src/meta.rs:44 — one canonical row kept |
| `mycelium_core::meta::PackScheme::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PackScheme::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PackScheme::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PackScheme::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PackScheme::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PackScheme::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PackScheme::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PackScheme::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PackScheme::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PackScheme::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PhysicalLayout` | dedup-alias: same definition as `mycelium_core::PhysicalLayout` at crates/mycelium-core/src/meta.rs:65 — one canonical row kept |
| `mycelium_core::meta::PhysicalLayout::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PhysicalLayout::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PhysicalLayout::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PhysicalLayout::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PhysicalLayout::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PhysicalLayout::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PhysicalLayout::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PhysicalLayout::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PhysicalLayout::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::PhysicalLayout::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Provenance` | dedup-alias: same definition as `mycelium_core::Provenance` at crates/mycelium-core/src/meta.rs:20 — one canonical row kept |
| `mycelium_core::meta::Provenance::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Provenance::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Provenance::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Provenance::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Provenance::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Provenance::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Provenance::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Provenance::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Provenance::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::Provenance::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::SparsityObs` | dedup-alias: same definition as `mycelium_core::SparsityObs` at crates/mycelium-core/src/meta.rs:34 — one canonical row kept |
| `mycelium_core::meta::SparsityObs::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::SparsityObs::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::SparsityObs::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::SparsityObs::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::SparsityObs::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::SparsityObs::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::SparsityObs::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::SparsityObs::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::SparsityObs::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::meta::SparsityObs::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::node::Alt` | dedup-alias: same definition as `mycelium_core::Alt` at crates/mycelium-core/src/node.rs:144 — one canonical row kept |
| `mycelium_core::node::Alt::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::node::Alt::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::node::Alt::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::node::Alt::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::node::Alt::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::node::Alt::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::node::Node` | dedup-alias: same definition as `mycelium_core::Node` at crates/mycelium-core/src/node.rs:37 — one canonical row kept |
| `mycelium_core::node::Node::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::node::Node::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::node::Node::content_hash` | ambiguous: short name 'content_hash' is defined in multiple modules; attributed to crates/mycelium-core/src/content.rs by heuristic — verify against source (ground truth) |
| `mycelium_core::node::Node::content_hash` | ambiguous: short name 'content_hash' is defined in multiple modules; attributed to crates/mycelium-core/src/content.rs by heuristic — verify against source (ground truth) |
| `mycelium_core::node::Node::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::node::Node::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::node::Node::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::node::Node::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::node::PolicyRef` | dedup-alias: same definition as `mycelium_core::PolicyRef` at crates/mycelium-core/src/node.rs:33 — one canonical row kept |
| `mycelium_core::node::Prim` | dedup-alias: same definition as `mycelium_core::Prim` at crates/mycelium-core/src/node.rs:31 — one canonical row kept |
| `mycelium_core::node::VarId` | dedup-alias: same definition as `mycelium_core::VarId` at crates/mycelium-core/src/node.rs:29 — one canonical row kept |
| `mycelium_core::prim::PrimDecl` | dedup-alias: same definition as `mycelium_core::PrimDecl` at crates/mycelium-core/src/prim.rs:79 — one canonical row kept |
| `mycelium_core::prim::PrimDecl::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimDecl::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimDecl::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimDecl::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimDecl::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimDecl::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimParadigm` | dedup-alias: same definition as `mycelium_core::PrimParadigm` at crates/mycelium-core/src/prim.rs:36 — one canonical row kept |
| `mycelium_core::prim::PrimParadigm::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimParadigm::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimParadigm::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimParadigm::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimParadigm::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimParadigm::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimRef` | dedup-alias: same definition as `mycelium_core::PrimRef` at crates/mycelium-core/src/prim.rs:103 — one canonical row kept |
| `mycelium_core::prim::PrimRef::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimRef::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimRef::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimRef::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimRef::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimRef::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimRef::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimRef::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimRef::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimRef::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimRef::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimRef::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimRef::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimRef::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimSig` | dedup-alias: same definition as `mycelium_core::PrimSig` at crates/mycelium-core/src/prim.rs:58 — one canonical row kept |
| `mycelium_core::prim::PrimSig::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimSig::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimSig::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimSig::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimSig::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimSig::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimTable` | dedup-alias: same definition as `mycelium_core::PrimTable` at crates/mycelium-core/src/prim.rs:134 — one canonical row kept |
| `mycelium_core::prim::PrimTable::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimTable::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimTable::decl` | dedup-alias: same definition as `mycelium_core::prim::PrimRef::decl` at crates/mycelium-core/src/prim.rs:114 — one canonical row kept |
| `mycelium_core::prim::PrimTable::decl` | dedup-alias: same definition as `mycelium_core::prim::PrimRef::decl` at crates/mycelium-core/src/prim.rs:114 — one canonical row kept |
| `mycelium_core::prim::PrimTable::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimTable::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimTable::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimTable::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimTable::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimTable::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::PrimTable::new` | dedup-alias: same definition as `mycelium_core::prim::PrimRef::new` at crates/mycelium-core/src/prim.rs:108 — one canonical row kept |
| `mycelium_core::prim::PrimTable::new` | dedup-alias: same definition as `mycelium_core::prim::PrimRef::new` at crates/mycelium-core/src/prim.rs:108 — one canonical row kept |
| `mycelium_core::prim::WidthRel` | dedup-alias: same definition as `mycelium_core::WidthRel` at crates/mycelium-core/src/prim.rs:50 — one canonical row kept |
| `mycelium_core::prim::WidthRel::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::WidthRel::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::WidthRel::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::WidthRel::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::WidthRel::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::prim::WidthRel::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::CleanupShape` | dedup-alias: same definition as `mycelium_core::CleanupShape` at crates/mycelium-core/src/recon.rs:59 — one canonical row kept |
| `mycelium_core::recon::CleanupShape::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::CleanupShape::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::CleanupShape::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::CleanupShape::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::CleanupShape::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::CleanupShape::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::CleanupShape::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::CleanupShape::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::CleanupShape::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::CleanupShape::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeProcedure` | dedup-alias: same definition as `mycelium_core::DecodeProcedure` at crates/mycelium-core/src/recon.rs:49 — one canonical row kept |
| `mycelium_core::recon::DecodeProcedure::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeProcedure::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeProcedure::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeProcedure::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeProcedure::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeProcedure::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeProcedure::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeProcedure::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeProcedure::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeProcedure::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeSpec` | dedup-alias: same definition as `mycelium_core::DecodeSpec` at crates/mycelium-core/src/recon.rs:79 — one canonical row kept |
| `mycelium_core::recon::DecodeSpec::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeSpec::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeSpec::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeSpec::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeSpec::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeSpec::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeSpec::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeSpec::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeSpec::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::DecodeSpec::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::InitStrategy` | dedup-alias: same definition as `mycelium_core::InitStrategy` at crates/mycelium-core/src/recon.rs:68 — one canonical row kept |
| `mycelium_core::recon::InitStrategy::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::InitStrategy::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::InitStrategy::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::InitStrategy::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::InitStrategy::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::InitStrategy::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::InitStrategy::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::InitStrategy::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::InitStrategy::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::InitStrategy::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::Recipe` | dedup-alias: same definition as `mycelium_core::Recipe` at crates/mycelium-core/src/recon.rs:40 — one canonical row kept |
| `mycelium_core::recon::Recipe::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::Recipe::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::Recipe::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::Recipe::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::Recipe::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::Recipe::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::Recipe::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::Recipe::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::Recipe::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::Recipe::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconInfo` | dedup-alias: same definition as `mycelium_core::ReconInfo` at crates/mycelium-core/src/recon.rs:111 — one canonical row kept |
| `mycelium_core::recon::ReconInfo::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconInfo::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconInfo::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconInfo::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconInfo::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconInfo::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconInfo::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconInfo::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconInfo::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconInfo::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconMode` | dedup-alias: same definition as `mycelium_core::ReconMode` at crates/mycelium-core/src/recon.rs:30 — one canonical row kept |
| `mycelium_core::recon::ReconMode::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconMode::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconMode::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconMode::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconMode::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconMode::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconMode::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconMode::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconMode::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::recon::ReconMode::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::Repr` | dedup-alias: same definition as `mycelium_core::Repr` at crates/mycelium-core/src/repr.rs:57 — one canonical row kept |
| `mycelium_core::repr::Repr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::Repr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::Repr::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::Repr::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::Repr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::Repr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::Repr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::Repr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::Repr::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::Repr::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::ScalarKind` | dedup-alias: same definition as `mycelium_core::ScalarKind` at crates/mycelium-core/src/repr.rs:14 — one canonical row kept |
| `mycelium_core::repr::ScalarKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::ScalarKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::ScalarKind::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::ScalarKind::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::ScalarKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::ScalarKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::ScalarKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::ScalarKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::ScalarKind::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::ScalarKind::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::SparsityClass` | dedup-alias: same definition as `mycelium_core::SparsityClass` at crates/mycelium-core/src/repr.rs:44 — one canonical row kept |
| `mycelium_core::repr::SparsityClass::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::SparsityClass::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::SparsityClass::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::SparsityClass::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::SparsityClass::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::SparsityClass::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::SparsityClass::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::SparsityClass::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::SparsityClass::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::repr::SparsityClass::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Payload` | dedup-alias: same definition as `mycelium_core::Payload` at crates/mycelium-core/src/value.rs:55 — one canonical row kept |
| `mycelium_core::value::Payload::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Payload::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Payload::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Payload::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Payload::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Payload::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Payload::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Payload::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Payload::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Payload::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Trit` | dedup-alias: same definition as `mycelium_core::Trit` at crates/mycelium-core/src/value.rs:19 — one canonical row kept |
| `mycelium_core::value::Trit::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Trit::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Trit::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Trit::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Trit::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Trit::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Value` | dedup-alias: same definition as `mycelium_core::Value` at crates/mycelium-core/src/value.rs:134 — one canonical row kept |
| `mycelium_core::value::Value::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Value::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Value::content_hash` | ambiguous: short name 'content_hash' is defined in multiple modules; attributed to crates/mycelium-core/src/content.rs by heuristic — verify against source (ground truth) |
| `mycelium_core::value::Value::content_hash` | ambiguous: short name 'content_hash' is defined in multiple modules; attributed to crates/mycelium-core/src/content.rs by heuristic — verify against source (ground truth) |
| `mycelium_core::value::Value::content_hash` | dedup-alias: same definition as `mycelium_core::node::Node::content_hash` at crates/mycelium-core/src/content.rs:417 — one canonical row kept |
| `mycelium_core::value::Value::content_hash` | dedup-alias: same definition as `mycelium_core::node::Node::content_hash` at crates/mycelium-core/src/content.rs:417 — one canonical row kept |
| `mycelium_core::value::Value::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Value::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Value::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Value::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Value::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Value::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Value::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_core::value::Value::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_dense::DenseError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_dense::DenseError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_dense::DenseError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_dense::DenseError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_dense::DenseOp::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_dense::DenseOp::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_dense::DenseOp::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_dense::DenseSpace::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_dense::DenseSpace::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_dense::DenseSpace::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Code::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Code::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Code::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Code::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Code::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Code::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_diag::ContentHash` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_diag::Diag::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Diag::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Diag::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Diag::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Diag::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Locus::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Locus::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Locus::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Locus::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Locus::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Locus::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Locus::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Severity::as_str` | dedup-alias: same definition as `mycelium_diag::Code::as_str` at crates/mycelium-diag/src/lib.rs:127 — one canonical row kept |
| `mycelium_diag::Severity::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Severity::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Severity::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Severity::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Severity::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Severity::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Severity::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Severity::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Trace::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Trace::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Trace::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Trace::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Trace::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Trace::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_diag::Trace::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_doc::build::BuildInput` | dedup-alias: same definition as `mycelium_doc::BuildInput` at crates/mycelium-doc/src/build.rs:25 — one canonical row kept |
| `mycelium_doc::build::BuildInput::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::build::BuildInput::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::build::BuildInput::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::build::BuildInput::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::build::build` | dedup-alias: same definition as `mycelium_doc::build` at crates/mycelium-doc/src/build.rs:59 — one canonical row kept |
| `mycelium_doc::build::emit_all` | dedup-alias: same definition as `mycelium_doc::emit_all` at crates/mycelium-doc/src/build.rs:141 — one canonical row kept |
| `mycelium_doc::corpus::AnchorAlloc::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_doc::corpus::AnchorAlloc::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::CHECK_NAMES:` | dedup-alias: same definition as `mycelium_doc::CHECK_NAMES:` at crates/mycelium-doc/src/doc_lint.rs:16 — one canonical row kept |
| `mycelium_doc::doc_lint::CheckOutcome` | dedup-alias: same definition as `mycelium_doc::CheckOutcome` at crates/mycelium-doc/src/doc_lint.rs:76 — one canonical row kept |
| `mycelium_doc::doc_lint::CheckOutcome::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::CheckOutcome::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::CheckOutcome::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::CheckOutcome::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::CheckOutcome::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::CheckOutcome::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::CheckStatus` | dedup-alias: same definition as `mycelium_doc::CheckStatus` at crates/mycelium-doc/src/doc_lint.rs:65 — one canonical row kept |
| `mycelium_doc::doc_lint::CheckStatus::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::CheckStatus::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::CheckStatus::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::CheckStatus::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::CheckStatus::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::CheckStatus::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::DocLintReport` | dedup-alias: same definition as `mycelium_doc::DocLintReport` at crates/mycelium-doc/src/doc_lint.rs:89 — one canonical row kept |
| `mycelium_doc::doc_lint::DocLintReport::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::DocLintReport::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::DocLintReport::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::DocLintReport::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::DocLintReport::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::DocLintReport::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::Finding` | dedup-alias: same definition as `mycelium_doc::Finding` at crates/mycelium-doc/src/doc_lint.rs:52 — one canonical row kept |
| `mycelium_doc::doc_lint::Finding::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::Finding::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::Finding::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::Finding::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::Finding::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::Finding::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::Severity` | dedup-alias: same definition as `mycelium_doc::Severity` at crates/mycelium-doc/src/doc_lint.rs:29 — one canonical row kept |
| `mycelium_doc::doc_lint::Severity::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::Severity::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::Severity::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::Severity::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::Severity::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::Severity::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::doc_lint::lint` | dedup-alias: same definition as `mycelium_doc::lint` at crates/mycelium-doc/src/doc_lint.rs:116 — one canonical row kept |
| `mycelium_doc::emit::Artifacts::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::emit::Artifacts::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_doc::emit::Artifacts::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::hash::DocHasher::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::DocModel` | dedup-alias: same definition as `mycelium_doc::DocModel` at crates/mycelium-doc/src/ir.rs:320 — one canonical row kept |
| `mycelium_doc::ir::DocModel::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::DocModel::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::DocModel::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::DocModel::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::DocModel::new` | dedup-alias: same definition as `mycelium_doc::ir::Node::new` at crates/mycelium-doc/src/ir.rs:246 — one canonical row kept |
| `mycelium_doc::ir::DocModel::new` | dedup-alias: same definition as `mycelium_doc::ir::Node::new` at crates/mycelium-doc/src/ir.rs:246 — one canonical row kept |
| `mycelium_doc::ir::DocModel::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::DocModel::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Level` | dedup-alias: same definition as `mycelium_doc::Level` at crates/mycelium-doc/src/ir.rs:22 — one canonical row kept |
| `mycelium_doc::ir::Level::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Level::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Level::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Level::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Level::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Level::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Level::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Level::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Node` | dedup-alias: same definition as `mycelium_doc::Node` at crates/mycelium-doc/src/ir.rs:224 — one canonical row kept |
| `mycelium_doc::ir::Node::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Node::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Node::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Node::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Node::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Node::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Node::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Node::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Payload` | dedup-alias: same definition as `mycelium_doc::Payload` at crates/mycelium-doc/src/ir.rs:147 — one canonical row kept |
| `mycelium_doc::ir::Payload::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Payload::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Payload::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Payload::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Payload::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Payload::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Payload::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Payload::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Provenance` | dedup-alias: same definition as `mycelium_doc::Provenance` at crates/mycelium-doc/src/ir.rs:92 — one canonical row kept |
| `mycelium_doc::ir::Provenance::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Provenance::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Provenance::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Provenance::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Provenance::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Provenance::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Provenance::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::Provenance::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::SourceKind` | dedup-alias: same definition as `mycelium_doc::SourceKind` at crates/mycelium-doc/src/ir.rs:55 — one canonical row kept |
| `mycelium_doc::ir::SourceKind::as_str` | dedup-alias: same definition as `mycelium_doc::ir::Level::as_str` at crates/mycelium-doc/src/ir.rs:34 — one canonical row kept |
| `mycelium_doc::ir::SourceKind::as_str` | dedup-alias: same definition as `mycelium_doc::ir::Level::as_str` at crates/mycelium-doc/src/ir.rs:34 — one canonical row kept |
| `mycelium_doc::ir::SourceKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::SourceKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::SourceKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::SourceKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::SourceKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::SourceKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::SourceKind::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::SourceKind::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::XrefResolution::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::XrefResolution::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::XrefResolution::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::XrefResolution::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::XrefTarget::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::XrefTarget::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::XrefTarget::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_doc::ir::XrefTarget::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_fmt::FmtError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_fmt::FmtError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_fmt::FmtError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_fmt::FmtError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_fmt::Formatted::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_fmt::Formatted::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_fmt::Formatted::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_fmt::Formatted::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_fmt::Formatted::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_interp::EvalError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::EvalError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_interp::EvalError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::EvalError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::EvalError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_interp::EvalError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_interp::EvalError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_interp::EvalError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_interp::Interpreter::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_interp::Step::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::Step::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_interp::Step::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::SwapEngine::swap` | definition not found via regex heuristic (kind='fn', name='swap') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::Budgets` | dedup-alias: same definition as `mycelium_interp::Budgets` at crates/mycelium-interp/src/budget.rs:146 — one canonical row kept |
| `mycelium_interp::budget::Budgets::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::Budgets::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::Budgets::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::Budgets::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::Budgets::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::Budgets::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::Budgets::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::Budgets::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectBudget` | dedup-alias: same definition as `mycelium_interp::EffectBudget` at crates/mycelium-interp/src/budget.rs:63 — one canonical row kept |
| `mycelium_interp::budget::EffectBudget::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectBudget::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectBudget::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectBudget::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectBudget::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectBudget::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectBudgetExhausted` | dedup-alias: same definition as `mycelium_interp::EffectBudgetExhausted` at crates/mycelium-interp/src/budget.rs:114 — one canonical row kept |
| `mycelium_interp::budget::EffectBudgetExhausted::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectBudgetExhausted::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectBudgetExhausted::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectBudgetExhausted::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectBudgetExhausted::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectBudgetExhausted::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectBudgetExhausted::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectBudgetExhausted::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectKind` | dedup-alias: same definition as `mycelium_interp::EffectKind` at crates/mycelium-interp/src/budget.rs:27 — one canonical row kept |
| `mycelium_interp::budget::EffectKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectKind::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectKind::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectKind::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectKind::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectKind::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_interp::budget::EffectKind::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_interp::prims::PrimRegistry` | dedup-alias: same definition as `mycelium_interp::PrimRegistry` at crates/mycelium-interp/src/prims.rs:53 — one canonical row kept |
| `mycelium_interp::prims::PrimRegistry::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::prims::PrimRegistry::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::prims::PrimRegistry::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_interp::prims::PrimRegistry::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::CancelToken` | dedup-alias: same definition as `mycelium_interp::CancelToken` at crates/mycelium-interp/src/supervise.rs:33 — one canonical row kept |
| `mycelium_interp::supervise::CancelToken::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::CancelToken::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::CancelToken::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::CancelToken::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::CancelToken::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::CancelToken::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::CancelToken::new` | dedup-alias: same definition as `mycelium_interp::supervise::Supervisor::new` at crates/mycelium-interp/src/supervise.rs:40 — one canonical row kept |
| `mycelium_interp::supervise::CancelToken::new` | dedup-alias: same definition as `mycelium_interp::supervise::Supervisor::new` at crates/mycelium-interp/src/supervise.rs:40 — one canonical row kept |
| `mycelium_interp::supervise::Cancelled` | dedup-alias: same definition as `mycelium_interp::Cancelled` at crates/mycelium-interp/src/supervise.rs:75 — one canonical row kept |
| `mycelium_interp::supervise::Cancelled::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Cancelled::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Cancelled::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Cancelled::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Cancelled::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Cancelled::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Cancelled::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Cancelled::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Escalation` | dedup-alias: same definition as `mycelium_interp::Escalation` at crates/mycelium-interp/src/supervise.rs:135 — one canonical row kept |
| `mycelium_interp::supervise::Escalation::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Escalation::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Escalation::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Escalation::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Escalation::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Escalation::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Escalation::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Escalation::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::RestartIntensity` | dedup-alias: same definition as `mycelium_interp::RestartIntensity` at crates/mycelium-interp/src/supervise.rs:125 — one canonical row kept |
| `mycelium_interp::supervise::RestartIntensity::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::RestartIntensity::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::RestartIntensity::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::RestartIntensity::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::RestartIntensity::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::RestartIntensity::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Supervisor` | dedup-alias: same definition as `mycelium_interp::Supervisor` at crates/mycelium-interp/src/supervise.rs:179 — one canonical row kept |
| `mycelium_interp::supervise::Supervisor::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Supervisor::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Supervisor::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::Supervisor::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::TaskOutcome` | definition not found via regex heuristic (kind='fn', name='TaskOutcome') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::TaskOutcome` | definition not found via regex heuristic (kind='fn', name='TaskOutcome') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::TaskOutcome` | definition not found via regex heuristic (kind='fn', name='TaskOutcome') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::TaskOutcome` | definition not found via regex heuristic (kind='fn', name='TaskOutcome') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::TaskOutcome` | definition not found via regex heuristic (kind='fn', name='TaskOutcome') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::TaskOutcome` | definition not found via regex heuristic (kind='fn', name='TaskOutcome') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::TaskOutcome` | definition not found via regex heuristic (kind='fn', name='TaskOutcome') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::TaskOutcome` | definition not found via regex heuristic (kind='fn', name='TaskOutcome') — possibly macro-generated or cfg-gated |
| `mycelium_interp::supervise::TaskOutcome` | dedup-alias: same definition as `mycelium_interp::TaskOutcome` at crates/mycelium-interp/src/supervise.rs:94 — one canonical row kept |
| `mycelium_interp::swap::IdentitySwapEngine` | dedup-alias: same definition as `mycelium_interp::IdentitySwapEngine` at crates/mycelium-interp/src/swap.rs:27 — one canonical row kept |
| `mycelium_interp::swap::IdentitySwapEngine::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::swap::IdentitySwapEngine::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_interp::swap::IdentitySwapEngine::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_interp::swap::IdentitySwapEngine::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_interp::swap::IdentitySwapEngine::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::swap::IdentitySwapEngine::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_interp::swap::IdentitySwapEngine::swap` | definition not found via regex heuristic (kind='fn', name='swap') — possibly macro-generated or cfg-gated |
| `mycelium_interp::swap::IdentitySwapEngine::swap` | definition not found via regex heuristic (kind='fn', name='swap') — possibly macro-generated or cfg-gated |
| `mycelium_interp::swap::IdentitySwapEngine::swap` | definition not found via regex heuristic (kind='fn', name='swap') — possibly macro-generated or cfg-gated |
| `mycelium_interp::swap::IdentitySwapEngine::swap` | definition not found via regex heuristic (kind='fn', name='swap') — possibly macro-generated or cfg-gated |
| `mycelium_interp::swap::SwapEngine` | dedup-alias: same definition as `mycelium_interp::SwapEngine` at crates/mycelium-interp/src/swap.rs:16 — one canonical row kept |
| `mycelium_interp::swap::SwapEngine::swap` | definition not found via regex heuristic (kind='fn', name='swap') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::AmbientError` | dedup-alias: same definition as `mycelium_l1::AmbientError` at crates/mycelium-l1/src/ambient.rs:50 — one canonical row kept |
| `mycelium_l1::ambient::AmbientError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::AmbientError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::AmbientError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::AmbientError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::AmbientError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::AmbientError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::AmbientError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::AmbientError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::ResolutionNote::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::ResolutionNote::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::ResolutionNote::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::Resolved` | dedup-alias: same definition as `mycelium_l1::Resolved` at crates/mycelium-l1/src/ambient.rs:130 — one canonical row kept |
| `mycelium_l1::ambient::Resolved::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::Resolved::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::Resolved::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::Resolved::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::Resolved::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::Resolved::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ambient::expand_phylum_to_source` | dedup-alias: same definition as `mycelium_l1::expand_phylum_to_source` at crates/mycelium-l1/src/ambient.rs:227 — one canonical row kept |
| `mycelium_l1::ambient::expand_to_source` | dedup-alias: same definition as `mycelium_l1::expand_to_source` at crates/mycelium-l1/src/ambient.rs:198 — one canonical row kept |
| `mycelium_l1::ambient::resolve` | dedup-alias: same definition as `mycelium_l1::resolve` at crates/mycelium-l1/src/ambient.rs:144 — one canonical row kept |
| `mycelium_l1::ambient::resolve_report` | dedup-alias: same definition as `mycelium_l1::resolve_report` at crates/mycelium-l1/src/ambient.rs:152 — one canonical row kept |
| `mycelium_l1::ast::AmbientParams::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::AmbientParams::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::AmbientParams::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Arm::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Arm::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Arm::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::BaseType::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::BaseType::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::BaseType::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Ctor::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Ctor::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Ctor::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Expr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Expr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Expr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::FnDecl::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::FnDecl::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::FnDecl::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::FnSig::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::FnSig::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::FnSig::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Hypha::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Hypha::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Hypha::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::ImplDecl::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::ImplDecl::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::ImplDecl::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Item::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Item::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Item::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Literal::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Literal::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Literal::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Nodule` | dedup-alias: same definition as `mycelium_l1::Nodule` at crates/mycelium-l1/src/ast.rs:74 — one canonical row kept |
| `mycelium_l1::ast::Nodule::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Nodule::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Nodule::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Nodule::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Nodule::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Nodule::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Paradigm::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Paradigm::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Paradigm::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Paradigm::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Param::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Param::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Param::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Path::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Path::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Path::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Pattern::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Pattern::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Pattern::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Phylum` | dedup-alias: same definition as `mycelium_l1::Phylum` at crates/mycelium-l1/src/ast.rs:16 — one canonical row kept |
| `mycelium_l1::ast::Phylum::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Phylum::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Phylum::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Phylum::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Phylum::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Phylum::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Scalar::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Scalar::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Scalar::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Scalar::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Sparsity::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Sparsity::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Sparsity::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Strength::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Strength::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Strength::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Strength::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::TraitDecl::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::TraitDecl::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::TraitDecl::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::TraitRef::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::TraitRef::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::TraitRef::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::TypeDecl::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::TypeDecl::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::TypeDecl::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::TypeParam::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::TypeParam::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::TypeParam::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::TypeRef::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::TypeRef::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::TypeRef::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::UsePath` | dedup-alias: same definition as `mycelium_l1::UsePath` at crates/mycelium-l1/src/ast.rs:63 — one canonical row kept |
| `mycelium_l1::ast::UsePath::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::UsePath::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::UsePath::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::UsePath::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::UsePath::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::UsePath::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Vis` | dedup-alias: same definition as `mycelium_l1::Vis` at crates/mycelium-l1/src/ast.rs:42 — one canonical row kept |
| `mycelium_l1::ast::Vis::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Vis::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Vis::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Vis::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Vis::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::ast::Vis::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::CheckError` | dedup-alias: same definition as `mycelium_l1::CheckError` at crates/mycelium-l1/src/checkty.rs:115 — one canonical row kept |
| `mycelium_l1::checkty::CheckError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::CheckError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::CheckError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::CheckError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::CheckError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::CheckError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::CheckError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::CheckError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::CheckError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::CheckError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::CheckError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::CheckError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::CtorInfo::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::CtorInfo::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::CtorInfo::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::DataInfo::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::DataInfo::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::DataInfo::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::Env` | dedup-alias: same definition as `mycelium_l1::Env` at crates/mycelium-l1/src/checkty.rs:333 — one canonical row kept |
| `mycelium_l1::checkty::Env::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::Env::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::Env::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::Env::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::InstanceInfo::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::InstanceInfo::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::InstanceInfo::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::PhylumEnv` | dedup-alias: same definition as `mycelium_l1::PhylumEnv` at crates/mycelium-l1/src/checkty.rs:503 — one canonical row kept |
| `mycelium_l1::checkty::PhylumEnv::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::PhylumEnv::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::PhylumEnv::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::PhylumEnv::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::TraitInfo::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::TraitInfo::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::TraitInfo::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::Ty` | dedup-alias: same definition as `mycelium_l1::Ty` at crates/mycelium-l1/src/checkty.rs:56 — one canonical row kept |
| `mycelium_l1::checkty::Ty::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::Ty::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::Ty::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::Ty::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::Ty::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::Ty::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::Ty::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::Ty::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::checkty::check_and_resolve` | dedup-alias: same definition as `mycelium_l1::check_and_resolve` at crates/mycelium-l1/src/checkty.rs:1010 — one canonical row kept |
| `mycelium_l1::checkty::check_nodule` | dedup-alias: same definition as `mycelium_l1::check_nodule` at crates/mycelium-l1/src/checkty.rs:595 — one canonical row kept |
| `mycelium_l1::checkty::check_nodule_matured` | dedup-alias: same definition as `mycelium_l1::check_nodule_matured` at crates/mycelium-l1/src/checkty.rs:951 — one canonical row kept |
| `mycelium_l1::checkty::check_phylum` | dedup-alias: same definition as `mycelium_l1::check_phylum` at crates/mycelium-l1/src/checkty.rs:613 — one canonical row kept |
| `mycelium_l1::checkty::check_phylum_matured` | dedup-alias: same definition as `mycelium_l1::check_phylum_matured` at crates/mycelium-l1/src/checkty.rs:623 — one canonical row kept |
| `mycelium_l1::elab::ElabError` | dedup-alias: same definition as `mycelium_l1::ElabError` at crates/mycelium-l1/src/elab.rs:46 — one canonical row kept |
| `mycelium_l1::elab::ElabError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::elab::ElabError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::elab::ElabError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::elab::ElabError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::elab::ElabError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::elab::ElabError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::elab::ElabError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::elab::ElabError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::elab::elaborate` | dedup-alias: same definition as `mycelium_l1::elaborate` at crates/mycelium-l1/src/elab.rs:213 — one canonical row kept |
| `mycelium_l1::elab::elaborate_colony` | dedup-alias: same definition as `mycelium_l1::elaborate_colony` at crates/mycelium-l1/src/elab.rs:250 — one canonical row kept |
| `mycelium_l1::error::ParseError` | dedup-alias: same definition as `mycelium_l1::ParseError` at crates/mycelium-l1/src/error.rs:9 — one canonical row kept |
| `mycelium_l1::error::ParseError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::error::ParseError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::error::ParseError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::error::ParseError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::error::ParseError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::error::ParseError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::error::ParseError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::error::ParseError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::Evaluator` | definition not found via regex heuristic (kind='fn', name='Evaluator') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::Evaluator` | definition not found via regex heuristic (kind='fn', name='Evaluator') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::Evaluator` | definition not found via regex heuristic (kind='fn', name='Evaluator') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::Evaluator` | definition not found via regex heuristic (kind='fn', name='Evaluator') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::Evaluator` | definition not found via regex heuristic (kind='fn', name='Evaluator') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::Evaluator` | definition not found via regex heuristic (kind='fn', name='Evaluator') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::Evaluator` | definition not found via regex heuristic (kind='fn', name='Evaluator') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::Evaluator` | definition not found via regex heuristic (kind='fn', name='Evaluator') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::Evaluator` | definition not found via regex heuristic (kind='fn', name='Evaluator') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::Evaluator` | definition not found via regex heuristic (kind='fn', name='Evaluator') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::Evaluator` | definition not found via regex heuristic (kind='fn', name='Evaluator') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::Evaluator` | definition not found via regex heuristic (kind='fn', name='Evaluator') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::Evaluator` | dedup-alias: same definition as `mycelium_l1::Evaluator` at crates/mycelium-l1/src/eval.rs:260 — one canonical row kept |
| `mycelium_l1::eval::EvaluatorOpts::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::EvaluatorOpts::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::EvaluatorOpts::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::EvaluatorOpts::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::L1Error` | dedup-alias: same definition as `mycelium_l1::L1Error` at crates/mycelium-l1/src/eval.rs:93 — one canonical row kept |
| `mycelium_l1::eval::L1Error::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::L1Error::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::L1Error::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::L1Error::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::L1Error::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::L1Error::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::L1Error::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::L1Error::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::L1Error::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::L1Error::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::L1Value` | dedup-alias: same definition as `mycelium_l1::L1Value` at crates/mycelium-l1/src/eval.rs:42 — one canonical row kept |
| `mycelium_l1::eval::L1Value::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::L1Value::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::L1Value::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::L1Value::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::L1Value::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::eval::L1Value::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::lexer::Comment::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::lexer::Comment::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::lexer::Comment::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::HofSpecialization::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::HofSpecialization::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::HofSpecialization::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::InstanceSelection` | dedup-alias: same definition as `mycelium_l1::InstanceSelection` at crates/mycelium-l1/src/mono.rs:63 — one canonical row kept |
| `mycelium_l1::mono::InstanceSelection::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::InstanceSelection::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::InstanceSelection::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::InstanceSelection::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::InstanceSelection::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::InstanceSelection::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::MonoSelections` | dedup-alias: same definition as `mycelium_l1::MonoSelections` at crates/mycelium-l1/src/mono.rs:83 — one canonical row kept |
| `mycelium_l1::mono::MonoSelections::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::MonoSelections::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::MonoSelections::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::MonoSelections::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::MonoSelections::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::MonoSelections::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::MonoSelections::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::MonoSelections::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::mono::monomorphize` | dedup-alias: same definition as `mycelium_l1::monomorphize` at crates/mycelium-l1/src/mono.rs:140 — one canonical row kept |
| `mycelium_l1::mono::monomorphize_with_selections` | dedup-alias: same definition as `mycelium_l1::monomorphize_with_selections` at crates/mycelium-l1/src/mono.rs:149 — one canonical row kept |
| `mycelium_l1::nodule::NoduleHeader` | dedup-alias: same definition as `mycelium_l1::NoduleHeader` at crates/mycelium-l1/src/nodule.rs:25 — one canonical row kept |
| `mycelium_l1::nodule::NoduleHeader::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::nodule::NoduleHeader::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::nodule::NoduleHeader::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::nodule::NoduleHeader::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::nodule::NoduleHeader::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::nodule::NoduleHeader::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::nodule::NoduleHeaderError` | dedup-alias: same definition as `mycelium_l1::NoduleHeaderError` at crates/mycelium-l1/src/nodule.rs:51 — one canonical row kept |
| `mycelium_l1::nodule::NoduleHeaderError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::nodule::NoduleHeaderError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::nodule::NoduleHeaderError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::nodule::NoduleHeaderError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::nodule::NoduleHeaderError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::nodule::NoduleHeaderError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::nodule::NoduleHeaderError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::nodule::NoduleHeaderError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::nodule::parse_nodule_header` | dedup-alias: same definition as `mycelium_l1::parse_nodule_header` at crates/mycelium-l1/src/nodule.rs:75 — one canonical row kept |
| `mycelium_l1::parse::parse` | dedup-alias: same definition as `mycelium_l1::parse` at crates/mycelium-l1/src/parse.rs:26 — one canonical row kept |
| `mycelium_l1::parse::parse_phylum` | dedup-alias: same definition as `mycelium_l1::parse_phylum` at crates/mycelium-l1/src/parse.rs:48 — one canonical row kept |
| `mycelium_l1::token::Pos::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::token::Pos::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::token::Pos::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::token::Pos::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::token::ScalarTok::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::token::ScalarTok::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::token::ScalarTok::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::token::Spanned::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::token::Spanned::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::token::Spanned::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::token::StrengthTok::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::token::StrengthTok::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::token::StrengthTok::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::token::Tok::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::token::Tok::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::token::Tok::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::totality::Totality` | dedup-alias: same definition as `mycelium_l1::Totality` at crates/mycelium-l1/src/totality.rs:31 — one canonical row kept |
| `mycelium_l1::totality::Totality::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::totality::Totality::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_l1::totality::Totality::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::totality::Totality::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_l1::totality::Totality::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_l1::totality::Totality::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lint::DOC_QUALITY_CHECKS` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_lint::Fix::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lint::Fix::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lint::Fix::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lint::Fix::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lint::FixTier::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lint::FixTier::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lint::FixTier::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lint::LintFinding::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lint::LintFinding::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lint::LintFinding::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lint::LintReport::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lint::LintReport::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lint::LintReport::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::EffectBudget` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_lsp::EffectBudgetExhausted` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_lsp::EffectKind` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_lsp::RecoverUndeclaredEffect` | definition not found via regex heuristic (kind='struct', name='RecoverUndeclaredEffect') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::baseline::BaselineRule` | dedup-alias: same definition as `mycelium_lsp::BaselineRule` at crates/mycelium-lsp/src/baseline.rs:32 — one canonical row kept |
| `mycelium_lsp::baseline::BaselineRule::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::baseline::BaselineRule::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::baseline::BaselineRule::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::baseline::BaselineRule::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::baseline::BaselineRule::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::baseline::BaselineRule::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::baseline::RESILIENT_MAX_ATTEMPTS:` | dedup-alias: same definition as `mycelium_lsp::RESILIENT_MAX_ATTEMPTS:` at crates/mycelium-lsp/src/baseline.rs:183 — one canonical row kept |
| `mycelium_lsp::baseline::RecoveryProfile` | dedup-alias: same definition as `mycelium_lsp::RecoveryProfile` at crates/mycelium-lsp/src/baseline.rs:152 — one canonical row kept |
| `mycelium_lsp::baseline::RecoveryProfile::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::baseline::RecoveryProfile::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::baseline::RecoveryProfile::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::baseline::RecoveryProfile::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::baseline::RecoveryProfile::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::baseline::RecoveryProfile::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::baseline::baseline_for_class` | dedup-alias: same definition as `mycelium_lsp::baseline_for_class` at crates/mycelium-lsp/src/baseline.rs:45 — one canonical row kept |
| `mycelium_lsp::baseline::derive_baseline` | dedup-alias: same definition as `mycelium_lsp::derive_baseline` at crates/mycelium-lsp/src/baseline.rs:97 — one canonical row kept |
| `mycelium_lsp::baseline::derive_baseline_for` | dedup-alias: same definition as `mycelium_lsp::derive_baseline_for` at crates/mycelium-lsp/src/baseline.rs:110 — one canonical row kept |
| `mycelium_lsp::baseline::explain_baseline` | dedup-alias: same definition as `mycelium_lsp::explain_baseline` at crates/mycelium-lsp/src/baseline.rs:134 — one canonical row kept |
| `mycelium_lsp::baseline::recovery_profile` | dedup-alias: same definition as `mycelium_lsp::recovery_profile` at crates/mycelium-lsp/src/baseline.rs:191 — one canonical row kept |
| `mycelium_lsp::completions::CompletionItem` | dedup-alias: same definition as `mycelium_lsp::CompletionItem` at crates/mycelium-lsp/src/completions.rs:47 — one canonical row kept |
| `mycelium_lsp::completions::CompletionItem::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::completions::CompletionItem::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::completions::CompletionItem::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::completions::CompletionItem::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::completions::CompletionItem::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::completions::CompletionItem::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::completions::KEYWORD_COMPLETIONS:` | dedup-alias: same definition as `mycelium_lsp::KEYWORD_COMPLETIONS:` at crates/mycelium-lsp/src/completions.rs:90 — one canonical row kept |
| `mycelium_lsp::completions::SNIPPET_COMPLETIONS:` | dedup-alias: same definition as `mycelium_lsp::SNIPPET_COMPLETIONS:` at crates/mycelium-lsp/src/completions.rs:462 — one canonical row kept |
| `mycelium_lsp::completions::completion_list` | dedup-alias: same definition as `mycelium_lsp::completion_list` at crates/mycelium-lsp/src/completions.rs:538 — one canonical row kept |
| `mycelium_lsp::diagnostics::AuditView` | dedup-alias: same definition as `mycelium_lsp::AuditView` at crates/mycelium-lsp/src/diagnostics/audit.rs:34 — one canonical row kept |
| `mycelium_lsp::diagnostics::ClassRegistry` | dedup-alias: same definition as `mycelium_lsp::ClassRegistry` at crates/mycelium-lsp/src/diagnostics/registry.rs:60 — one canonical row kept |
| `mycelium_lsp::diagnostics::Crossing` | dedup-alias: same definition as `mycelium_lsp::Crossing` at crates/mycelium-lsp/src/diagnostics/audit.rs:18 — one canonical row kept |
| `mycelium_lsp::diagnostics::DiagnosticPolicy` | dedup-alias: same definition as `mycelium_lsp::DiagnosticPolicy` at crates/mycelium-lsp/src/diagnostics/policy.rs:86 — one canonical row kept |
| `mycelium_lsp::diagnostics::DiagnosticRecord` | dedup-alias: same definition as `mycelium_lsp::DiagnosticRecord` at crates/mycelium-lsp/src/diagnostics/record.rs:109 — one canonical row kept |
| `mycelium_lsp::diagnostics::Level` | dedup-alias: same definition as `mycelium_lsp::Level` at crates/mycelium-lsp/src/diagnostics/record.rs:24 — one canonical row kept |
| `mycelium_lsp::diagnostics::Presentation` | dedup-alias: same definition as `mycelium_lsp::Presentation` at crates/mycelium-lsp/src/diagnostics/record.rs:136 — one canonical row kept |
| `mycelium_lsp::diagnostics::ReasonedError` | dedup-alias: same definition as `mycelium_lsp::ReasonedError` at crates/mycelium-lsp/src/diagnostics/record.rs:64 — one canonical row kept |
| `mycelium_lsp::diagnostics::Rule` | dedup-alias: same definition as `mycelium_lsp::Rule` at crates/mycelium-lsp/src/diagnostics/policy.rs:28 — one canonical row kept |
| `mycelium_lsp::diagnostics::UnknownClass` | dedup-alias: same definition as `mycelium_lsp::UnknownClass` at crates/mycelium-lsp/src/diagnostics/registry.rs:36 — one canonical row kept |
| `mycelium_lsp::diagnostics::audit::AuditView` | dedup-alias: same definition as `mycelium_lsp::AuditView` at crates/mycelium-lsp/src/diagnostics/audit.rs:34 — one canonical row kept |
| `mycelium_lsp::diagnostics::audit::AuditView::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::AuditView::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::AuditView::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::AuditView::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::AuditView::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::AuditView::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::AuditView::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::AuditView::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::AuditView::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::AuditView::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::AuditView::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::AuditView::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::Crossing` | dedup-alias: same definition as `mycelium_lsp::Crossing` at crates/mycelium-lsp/src/diagnostics/audit.rs:18 — one canonical row kept |
| `mycelium_lsp::diagnostics::audit::Crossing::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::Crossing::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::Crossing::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::Crossing::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::Crossing::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::Crossing::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::Crossing::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::Crossing::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::Crossing::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::Crossing::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::Crossing::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::audit::Crossing::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy` | dedup-alias: same definition as `mycelium_lsp::DiagnosticPolicy` at crates/mycelium-lsp/src/diagnostics/policy.rs:86 — one canonical row kept |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::new` | dedup-alias: same definition as `mycelium_lsp::diagnostics::policy::Rule::new` at crates/mycelium-lsp/src/diagnostics/policy.rs:46 — one canonical row kept |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::new` | dedup-alias: same definition as `mycelium_lsp::diagnostics::policy::Rule::new` at crates/mycelium-lsp/src/diagnostics/policy.rs:46 — one canonical row kept |
| `mycelium_lsp::diagnostics::policy::DiagnosticPolicy::new` | dedup-alias: same definition as `mycelium_lsp::diagnostics::policy::Rule::new` at crates/mycelium-lsp/src/diagnostics/policy.rs:46 — one canonical row kept |
| `mycelium_lsp::diagnostics::policy::PolicyFile` | dedup-alias: same definition as `mycelium_lsp::diagnostics::PolicyFile` at crates/mycelium-lsp/src/diagnostics/policy.rs:158 — one canonical row kept |
| `mycelium_lsp::diagnostics::policy::PolicyFile::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::PolicyFile::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::PolicyFile::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::PolicyFile::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::PolicyFile::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::PolicyFile::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::PolicyFile::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::PolicyFile::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::PolicyFile::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::PolicyFile::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::PolicyFile::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::PolicyFile::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule` | dedup-alias: same definition as `mycelium_lsp::Rule` at crates/mycelium-lsp/src/diagnostics/policy.rs:28 — one canonical row kept |
| `mycelium_lsp::diagnostics::policy::Rule::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::policy::Rule::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::present` | dedup-alias: same definition as `mycelium_lsp::present` at crates/mycelium-lsp/src/diagnostics/record.rs:151 — one canonical row kept |
| `mycelium_lsp::diagnostics::record::DETAILED_ALLOWLIST:` | dedup-alias: same definition as `mycelium_lsp::diagnostics::DETAILED_ALLOWLIST:` at crates/mycelium-lsp/src/diagnostics/record.rs:36 — one canonical row kept |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord` | dedup-alias: same definition as `mycelium_lsp::DiagnosticRecord` at crates/mycelium-lsp/src/diagnostics/record.rs:109 — one canonical row kept |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::DiagnosticRecord::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level` | dedup-alias: same definition as `mycelium_lsp::Level` at crates/mycelium-lsp/src/diagnostics/record.rs:24 — one canonical row kept |
| `mycelium_lsp::diagnostics::record::Level::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Level::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Presentation` | dedup-alias: same definition as `mycelium_lsp::Presentation` at crates/mycelium-lsp/src/diagnostics/record.rs:136 — one canonical row kept |
| `mycelium_lsp::diagnostics::record::Presentation::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Presentation::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Presentation::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Presentation::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Presentation::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Presentation::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Presentation::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Presentation::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::Presentation::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::ReasonedError` | dedup-alias: same definition as `mycelium_lsp::ReasonedError` at crates/mycelium-lsp/src/diagnostics/record.rs:64 — one canonical row kept |
| `mycelium_lsp::diagnostics::record::ReasonedError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::ReasonedError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::ReasonedError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::ReasonedError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::ReasonedError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::ReasonedError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::ReasonedError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::ReasonedError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::ReasonedError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::record::present` | dedup-alias: same definition as `mycelium_lsp::present` at crates/mycelium-lsp/src/diagnostics/record.rs:151 — one canonical row kept |
| `mycelium_lsp::diagnostics::registry::ClassName` | dedup-alias: same definition as `mycelium_lsp::diagnostics::ClassName` at crates/mycelium-lsp/src/diagnostics/registry.rs:17 — one canonical row kept |
| `mycelium_lsp::diagnostics::registry::ClassName::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassName::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassName::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassName::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassName::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassName::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassName::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassName::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassName::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassName::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassName::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassName::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassName::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassName::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassRegistry` | dedup-alias: same definition as `mycelium_lsp::ClassRegistry` at crates/mycelium-lsp/src/diagnostics/registry.rs:60 — one canonical row kept |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::ClassRegistry::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::UnknownClass` | dedup-alias: same definition as `mycelium_lsp::UnknownClass` at crates/mycelium-lsp/src/diagnostics/registry.rs:36 — one canonical row kept |
| `mycelium_lsp::diagnostics::registry::UnknownClass::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::UnknownClass::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::UnknownClass::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::UnknownClass::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::UnknownClass::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::UnknownClass::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::UnknownClass::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::UnknownClass::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::UnknownClass::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::UnknownClass::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::UnknownClass::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::registry::UnknownClass::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Delivery` | dedup-alias: same definition as `mycelium_lsp::diagnostics::Delivery` at crates/mycelium-lsp/src/diagnostics/sink.rs:161 — one canonical row kept |
| `mycelium_lsp::diagnostics::sink::Delivery::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Delivery::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Delivery::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Delivery::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Delivery::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Delivery::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Route` | dedup-alias: same definition as `mycelium_lsp::diagnostics::Route` at crates/mycelium-lsp/src/diagnostics/sink.rs:32 — one canonical row kept |
| `mycelium_lsp::diagnostics::sink::Route::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Route::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Route::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Route::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Route::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Route::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Route::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Route::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Route::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Route::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Route::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Route::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Route::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::Route::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::SinkBinding` | dedup-alias: same definition as `mycelium_lsp::diagnostics::SinkBinding` at crates/mycelium-lsp/src/diagnostics/sink.rs:215 — one canonical row kept |
| `mycelium_lsp::diagnostics::sink::SinkBinding::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::SinkBinding::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::SinkBinding::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::SinkBinding::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::SinkBinding::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::SinkBinding::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::UnknownRoute` | dedup-alias: same definition as `mycelium_lsp::diagnostics::UnknownRoute` at crates/mycelium-lsp/src/diagnostics/sink.rs:138 — one canonical row kept |
| `mycelium_lsp::diagnostics::sink::UnknownRoute::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::UnknownRoute::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::UnknownRoute::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::UnknownRoute::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::UnknownRoute::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::UnknownRoute::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::UnknownRoute::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::diagnostics::sink::UnknownRoute::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::expand::expand_ambient` | dedup-alias: same definition as `mycelium_lsp::expand_ambient` at crates/mycelium-lsp/src/expand.rs:26 — one canonical row kept |
| `mycelium_lsp::feedback::ExplainSite` | dedup-alias: same definition as `mycelium_lsp::ExplainSite` at crates/mycelium-lsp/src/feedback.rs:86 — one canonical row kept |
| `mycelium_lsp::feedback::ExplainSite::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::ExplainSite::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::ExplainSite::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::ExplainSite::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::ExplainSite::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::ExplainSite::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::Feedback` | dedup-alias: same definition as `mycelium_lsp::Feedback` at crates/mycelium-lsp/src/feedback.rs:95 — one canonical row kept |
| `mycelium_lsp::feedback::Feedback::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::Feedback::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::Feedback::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::Feedback::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::Feedback::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::Feedback::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::FeedbackSummary` | dedup-alias: same definition as `mycelium_lsp::FeedbackSummary` at crates/mycelium-lsp/src/feedback.rs:117 — one canonical row kept |
| `mycelium_lsp::feedback::FeedbackSummary::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::FeedbackSummary::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::FeedbackSummary::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::FeedbackSummary::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::FeedbackSummary::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::FeedbackSummary::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::GuaranteeAnnotation` | dedup-alias: same definition as `mycelium_lsp::GuaranteeAnnotation` at crates/mycelium-lsp/src/feedback.rs:42 — one canonical row kept |
| `mycelium_lsp::feedback::GuaranteeAnnotation::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::GuaranteeAnnotation::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::GuaranteeAnnotation::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::GuaranteeAnnotation::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::GuaranteeAnnotation::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::GuaranteeAnnotation::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::PrimSite::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::PrimSite::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::PrimSite::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::SwapSite` | dedup-alias: same definition as `mycelium_lsp::SwapSite` at crates/mycelium-lsp/src/feedback.rs:53 — one canonical row kept |
| `mycelium_lsp::feedback::SwapSite::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::SwapSite::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::SwapSite::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::SwapSite::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::SwapSite::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::SwapSite::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::feedback::analyze` | dedup-alias: same definition as `mycelium_lsp::analyze` at crates/mycelium-lsp/src/feedback.rs:183 — one canonical row kept |
| `mycelium_lsp::feedback::analyze_with` | dedup-alias: same definition as `mycelium_lsp::analyze_with` at crates/mycelium-lsp/src/feedback.rs:192 — one canonical row kept |
| `mycelium_lsp::fmt::format` | dedup-alias: same definition as `mycelium_lsp::format` at crates/mycelium-lsp/src/fmt.rs:16 — one canonical row kept |
| `mycelium_lsp::lint::Diagnostic` | dedup-alias: same definition as `mycelium_lsp::Diagnostic` at crates/mycelium-lsp/src/lint.rs:35 — one canonical row kept |
| `mycelium_lsp::lint::Diagnostic::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::lint::Diagnostic::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::lint::Diagnostic::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::lint::Diagnostic::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::lint::Diagnostic::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::lint::Diagnostic::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::lint::Severity` | dedup-alias: same definition as `mycelium_lsp::Severity` at crates/mycelium-lsp/src/lint.rs:26 — one canonical row kept |
| `mycelium_lsp::lint::Severity::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::lint::Severity::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::lint::Severity::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::lint::Severity::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::lint::Severity::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::lint::Severity::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::lint::has_errors` | dedup-alias: same definition as `mycelium_lsp::has_errors` at crates/mycelium-lsp/src/lint.rs:71 — one canonical row kept |
| `mycelium_lsp::lint::lint` | dedup-alias: same definition as `mycelium_lsp::lint` at crates/mycelium-lsp/src/lint.rs:62 — one canonical row kept |
| `mycelium_lsp::lint::lint_nodule_header` | dedup-alias: same definition as `mycelium_lsp::lint_nodule_header` at crates/mycelium-lsp/src/lint.rs:81 — one canonical row kept |
| `mycelium_lsp::lint::lint_structured_header` | dedup-alias: same definition as `mycelium_lsp::lint_structured_header` at crates/mycelium-lsp/src/lint.rs:103 — one canonical row kept |
| `mycelium_lsp::llm_canonical_parser::DEPTH_LIMIT:` | dedup-alias: same definition as `mycelium_lsp::DEPTH_LIMIT:` at crates/mycelium-lsp/src/llm_canonical_parser.rs:40 — one canonical row kept |
| `mycelium_lsp::llm_canonical_parser::ParseError` | dedup-alias: same definition as `mycelium_lsp::ParseError` at crates/mycelium-lsp/src/llm_canonical_parser.rs:44 — one canonical row kept |
| `mycelium_lsp::llm_canonical_parser::ParseError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::llm_canonical_parser::ParseError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::llm_canonical_parser::ParseError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::llm_canonical_parser::ParseError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::llm_canonical_parser::ParseError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::llm_canonical_parser::ParseError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::llm_canonical_parser::parse_llm_canonical` | dedup-alias: same definition as `mycelium_lsp::parse_llm_canonical` at crates/mycelium-lsp/src/llm_canonical_parser.rs:94 — one canonical row kept |
| `mycelium_lsp::recover::Action` | definition not found via regex heuristic (kind='enum', name='Action') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::EffectBudget` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_lsp::recover::EffectBudgetExhausted` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_lsp::recover::EffectBudgets` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_lsp::recover::EffectKind` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_lsp::recover::Outcome` | dedup-alias: same definition as `mycelium_lsp::Outcome` at crates/mycelium-lsp/src/recover/mod.rs:67 — one canonical row kept |
| `mycelium_lsp::recover::Outcome::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::Outcome::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::Outcome::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::Outcome::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::Outcome::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::Outcome::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::RecoveryPolicy` | dedup-alias: same definition as `mycelium_lsp::RecoveryPolicy` at crates/mycelium-lsp/src/recover/policy.rs:50 — one canonical row kept |
| `mycelium_lsp::recover::Resolution` | dedup-alias: same definition as `mycelium_lsp::Resolution` at crates/mycelium-lsp/src/recover/mod.rs:79 — one canonical row kept |
| `mycelium_lsp::recover::Resolution::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::Resolution::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::Resolution::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::Resolution::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::Resolution::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::Resolution::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::StructuredError` | dedup-alias: same definition as `mycelium_lsp::StructuredError` at crates/mycelium-lsp/src/recover/mod.rs:43 — one canonical row kept |
| `mycelium_lsp::recover::StructuredError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::StructuredError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::StructuredError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::StructuredError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::StructuredError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::StructuredError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::check_effects` | dedup-alias: same definition as `mycelium_lsp::check_effects` at crates/mycelium-lsp/src/recover/effect.rs:58 — one canonical row kept |
| `mycelium_lsp::recover::effect::Budgets` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_lsp::recover::effect::EffectBudget` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_lsp::recover::effect::EffectBudgetExhausted` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_lsp::recover::effect::EffectKind` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_lsp::recover::effect::EffectSet` | dedup-alias: same definition as `mycelium_lsp::recover::EffectSet` at crates/mycelium-lsp/src/recover/effect.rs:27 — one canonical row kept |
| `mycelium_lsp::recover::effect::UndeclaredEffect` | dedup-alias: same definition as `mycelium_lsp::recover::UndeclaredEffect` at crates/mycelium-lsp/src/recover/effect.rs:33 — one canonical row kept |
| `mycelium_lsp::recover::effect::UndeclaredEffect::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::effect::UndeclaredEffect::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::effect::UndeclaredEffect::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::effect::UndeclaredEffect::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::effect::UndeclaredEffect::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::effect::UndeclaredEffect::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::effect::UndeclaredEffect::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::effect::UndeclaredEffect::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::effect::UndeclaredEffect::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::effect::UndeclaredEffect::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::effect::UndeclaredEffect::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::effect::UndeclaredEffect::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::effect::check_effects` | dedup-alias: same definition as `mycelium_lsp::check_effects` at crates/mycelium-lsp/src/recover/effect.rs:58 — one canonical row kept |
| `mycelium_lsp::recover::handle` | dedup-alias: same definition as `mycelium_lsp::handle` at crates/mycelium-lsp/src/recover/mod.rs:110 — one canonical row kept |
| `mycelium_lsp::recover::policy::RecoveryAction::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryAction::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryAction::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryAction::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryAction::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryAction::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryPolicy` | dedup-alias: same definition as `mycelium_lsp::RecoveryPolicy` at crates/mycelium-lsp/src/recover/policy.rs:50 — one canonical row kept |
| `mycelium_lsp::recover::policy::RecoveryPolicy::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryPolicy::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryPolicy::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryPolicy::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryPolicy::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryPolicy::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryPolicy::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryPolicy::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryPolicy::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryPolicy::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryPolicy::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::recover::policy::RecoveryPolicy::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::sync::DocumentStore` | dedup-alias: same definition as `mycelium_lsp::DocumentStore` at crates/mycelium-lsp/src/sync.rs:27 — one canonical row kept |
| `mycelium_lsp::sync::DocumentStore::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::sync::DocumentStore::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::sync::DocumentStore::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::sync::DocumentStore::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::sync::DocumentStore::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::sync::DocumentStore::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_lsp::sync::publish_for_source` | dedup-alias: same definition as `mycelium_lsp::publish_for_source` at crates/mycelium-lsp/src/sync.rs:83 — one canonical row kept |
| `mycelium_lsp::sync::resilient_publish_for_source` | dedup-alias: same definition as `mycelium_lsp::resilient_publish_for_source` at crates/mycelium-lsp/src/sync.rs:110 — one canonical row kept |
| `mycelium_lsp::sync::resilient_source_diagnostics` | dedup-alias: same definition as `mycelium_lsp::resilient_source_diagnostics` at crates/mycelium-lsp/src/sync.rs:102 — one canonical row kept |
| `mycelium_lsp::sync::source_diagnostics` | dedup-alias: same definition as `mycelium_lsp::source_diagnostics` at crates/mycelium-lsp/src/sync.rs:71 — one canonical row kept |
| `mycelium_lsp::wire::publish_diagnostics_notification` | dedup-alias: same definition as `mycelium_lsp::publish_diagnostics_notification` at crates/mycelium-lsp/src/wire.rs:79 — one canonical row kept |
| `mycelium_lsp::wire::read_message` | dedup-alias: same definition as `mycelium_lsp::read_message` at crates/mycelium-lsp/src/wire.rs:115 — one canonical row kept |
| `mycelium_lsp::wire::serve` | dedup-alias: same definition as `mycelium_lsp::serve` at crates/mycelium-lsp/src/wire.rs:182 — one canonical row kept |
| `mycelium_lsp::wire::serve_stdio` | dedup-alias: same definition as `mycelium_lsp::serve_stdio` at crates/mycelium-lsp/src/wire.rs:254 — one canonical row kept |
| `mycelium_lsp::wire::to_lsp_diagnostic` | dedup-alias: same definition as `mycelium_lsp::to_lsp_diagnostic` at crates/mycelium-lsp/src/wire.rs:47 — one canonical row kept |
| `mycelium_lsp::wire::write_message` | dedup-alias: same definition as `mycelium_lsp::write_message` at crates/mycelium-lsp/src/wire.rs:154 — one canonical row kept |
| `mycelium_mlir::Task::Error` | definition not found via regex heuristic (kind='type', name='Error') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::Task::Output` | definition not found via regex heuristic (kind='type', name='Output') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::Task::poll` | definition not found via regex heuristic (kind='fn', name='poll') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::aot::default_depth_budget` | dedup-alias: same definition as `mycelium_mlir::default_depth_budget` at crates/mycelium-mlir/src/aot.rs:64 — one canonical row kept |
| `mycelium_mlir::aot::run` | dedup-alias: same definition as `mycelium_mlir::run` at crates/mycelium-mlir/src/aot.rs:213 — one canonical row kept |
| `mycelium_mlir::aot::run_core` | dedup-alias: same definition as `mycelium_mlir::run_core` at crates/mycelium-mlir/src/aot.rs:147 — one canonical row kept |
| `mycelium_mlir::aot::run_core_with_effects` | dedup-alias: same definition as `mycelium_mlir::run_core_with_effects` at crates/mycelium-mlir/src/aot.rs:196 — one canonical row kept |
| `mycelium_mlir::aot::run_core_with_fuel` | dedup-alias: same definition as `mycelium_mlir::run_core_with_fuel` at crates/mycelium-mlir/src/aot.rs:156 — one canonical row kept |
| `mycelium_mlir::aot::run_with_layout` | dedup-alias: same definition as `mycelium_mlir::run_with_layout` at crates/mycelium-mlir/src/aot.rs:547 — one canonical row kept |
| `mycelium_mlir::bitnet::BitnetDotKernel` | dedup-alias: same definition as `mycelium_mlir::BitnetDotKernel` at crates/mycelium-mlir/src/bitnet.rs:281 — one canonical row kept |
| `mycelium_mlir::bitnet::BitnetDotKernel::scheme` | dedup-alias: same definition as `mycelium_mlir::bitnet::KernelLayout::scheme` at crates/mycelium-mlir/src/bitnet.rs:72 — one canonical row kept |
| `mycelium_mlir::bitnet::BitnetDotKernel::scheme` | dedup-alias: same definition as `mycelium_mlir::bitnet::KernelLayout::scheme` at crates/mycelium-mlir/src/bitnet.rs:72 — one canonical row kept |
| `mycelium_mlir::bitnet::BoundBitnetDot` | definition not found via regex heuristic (kind='fn', name='BoundBitnetDot') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::bitnet::KernelLayout` | dedup-alias: same definition as `mycelium_mlir::KernelLayout` at crates/mycelium-mlir/src/bitnet.rs:56 — one canonical row kept |
| `mycelium_mlir::bitnet::KernelLayout::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::bitnet::KernelLayout::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::bitnet::KernelLayout::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::bitnet::KernelLayout::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::bitnet::KernelLayout::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::bitnet::KernelLayout::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::bitnet::compile_bitnet_dot` | dedup-alias: same definition as `mycelium_mlir::compile_bitnet_dot` at crates/mycelium-mlir/src/bitnet.rs:408 — one canonical row kept |
| `mycelium_mlir::bitnet::compile_bitnet_dot_for` | dedup-alias: same definition as `mycelium_mlir::compile_bitnet_dot_for` at crates/mycelium-mlir/src/bitnet.rs:415 — one canonical row kept |
| `mycelium_mlir::bitnet::emit_bitnet_dot_ir` | dedup-alias: same definition as `mycelium_mlir::emit_bitnet_dot_ir` at crates/mycelium-mlir/src/bitnet.rs:147 — one canonical row kept |
| `mycelium_mlir::bitnet::emit_bitnet_dot_ir_for` | dedup-alias: same definition as `mycelium_mlir::emit_bitnet_dot_ir_for` at crates/mycelium-mlir/src/bitnet.rs:159 — one canonical row kept |
| `mycelium_mlir::bitnet::jit_ternary_dot` | dedup-alias: same definition as `mycelium_mlir::jit_ternary_dot` at crates/mycelium-mlir/src/bitnet.rs:445 — one canonical row kept |
| `mycelium_mlir::bitnet::jit_ternary_dot_for` | dedup-alias: same definition as `mycelium_mlir::jit_ternary_dot_for` at crates/mycelium-mlir/src/bitnet.rs:451 — one canonical row kept |
| `mycelium_mlir::bitnet::ternary_dot_ref` | dedup-alias: same definition as `mycelium_mlir::ternary_dot_ref` at crates/mycelium-mlir/src/bitnet.rs:135 — one canonical row kept |
| `mycelium_mlir::budget::AutoDepthBudget` | dedup-alias: same definition as `mycelium_mlir::AutoDepthBudget` at crates/mycelium-mlir/src/budget.rs:168 — one canonical row kept |
| `mycelium_mlir::budget::AutoDepthBudget::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::AutoDepthBudget::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::AutoDepthBudget::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::AutoDepthBudget::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::AutoDepthBudget::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::AutoDepthBudget::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::AutoDepthBudget::resolve` | dedup-alias: same definition as `mycelium_mlir::DepthBudget::resolve` at crates/mycelium-mlir/src/inject.rs:172 — one canonical row kept |
| `mycelium_mlir::budget::AutoDepthBudget::resolve` | dedup-alias: same definition as `mycelium_mlir::DepthBudget::resolve` at crates/mycelium-mlir/src/inject.rs:172 — one canonical row kept |
| `mycelium_mlir::budget::AutoDepthBudget::resolve` | dedup-alias: same definition as `mycelium_mlir::DepthBudget::resolve` at crates/mycelium-mlir/src/inject.rs:172 — one canonical row kept |
| `mycelium_mlir::budget::AutoDepthBudget::resolve` | dedup-alias: same definition as `mycelium_mlir::DepthBudget::resolve` at crates/mycelium-mlir/src/inject.rs:172 — one canonical row kept |
| `mycelium_mlir::budget::DepthBasis` | dedup-alias: same definition as `mycelium_mlir::DepthBasis` at crates/mycelium-mlir/src/budget.rs:106 — one canonical row kept |
| `mycelium_mlir::budget::DepthBasis::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::DepthBasis::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::DepthBasis::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::DepthBasis::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::DepthBasis::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::DepthBasis::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::DepthBudget` | dedup-alias: same definition as `mycelium_mlir::DepthBudget` at crates/mycelium-mlir/src/budget.rs:67 — one canonical row kept |
| `mycelium_mlir::budget::DepthBudget::resolve` | dedup-alias: same definition as `mycelium_mlir::DepthBudget::resolve` at crates/mycelium-mlir/src/inject.rs:172 — one canonical row kept |
| `mycelium_mlir::budget::DepthResolution` | dedup-alias: same definition as `mycelium_mlir::DepthResolution` at crates/mycelium-mlir/src/budget.rs:75 — one canonical row kept |
| `mycelium_mlir::budget::DepthResolution::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::DepthResolution::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::DepthResolution::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::DepthResolution::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::DepthResolution::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::DepthResolution::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::DepthResolution::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::DepthResolution::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::MemSource` | dedup-alias: same definition as `mycelium_mlir::MemSource` at crates/mycelium-mlir/src/budget.rs:97 — one canonical row kept |
| `mycelium_mlir::budget::MemSource::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::MemSource::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::MemSource::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::MemSource::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::MemSource::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::MemSource::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::STATIC_FALLBACK_DEPTH:` | dedup-alias: same definition as `mycelium_mlir::STATIC_FALLBACK_DEPTH:` at crates/mycelium-mlir/src/budget.rs:60 — one canonical row kept |
| `mycelium_mlir::budget::StaticDepthBudget` | dedup-alias: same definition as `mycelium_mlir::StaticDepthBudget` at crates/mycelium-mlir/src/budget.rs:239 — one canonical row kept |
| `mycelium_mlir::budget::StaticDepthBudget::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::StaticDepthBudget::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::StaticDepthBudget::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::StaticDepthBudget::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::StaticDepthBudget::resolve` | dedup-alias: same definition as `mycelium_mlir::DepthBudget::resolve` at crates/mycelium-mlir/src/inject.rs:172 — one canonical row kept |
| `mycelium_mlir::budget::StaticDepthBudget::resolve` | dedup-alias: same definition as `mycelium_mlir::DepthBudget::resolve` at crates/mycelium-mlir/src/inject.rs:172 — one canonical row kept |
| `mycelium_mlir::budget::StaticDepthBudget::resolve` | dedup-alias: same definition as `mycelium_mlir::DepthBudget::resolve` at crates/mycelium-mlir/src/inject.rs:172 — one canonical row kept |
| `mycelium_mlir::budget::StaticDepthBudget::resolve` | dedup-alias: same definition as `mycelium_mlir::DepthBudget::resolve` at crates/mycelium-mlir/src/inject.rs:172 — one canonical row kept |
| `mycelium_mlir::budget::StaticReason` | dedup-alias: same definition as `mycelium_mlir::StaticReason` at crates/mycelium-mlir/src/budget.rs:86 — one canonical row kept |
| `mycelium_mlir::budget::StaticReason::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::StaticReason::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::StaticReason::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::StaticReason::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::StaticReason::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::budget::StaticReason::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Network` | dedup-alias: same definition as `mycelium_mlir::Network` at crates/mycelium-mlir/src/channel.rs:75 — one canonical row kept |
| `mycelium_mlir::channel::Network::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Network::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Network::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Network::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Receiver` | definition not found via regex heuristic (kind='fn', name='Receiver') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Receiver` | definition not found via regex heuristic (kind='fn', name='Receiver') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Receiver` | definition not found via regex heuristic (kind='fn', name='Receiver') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Receiver` | definition not found via regex heuristic (kind='fn', name='Receiver') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Receiver` | definition not found via regex heuristic (kind='fn', name='Receiver') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Receiver` | definition not found via regex heuristic (kind='fn', name='Receiver') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Receiver` | dedup-alias: same definition as `mycelium_mlir::Receiver` at crates/mycelium-mlir/src/channel.rs:121 — one canonical row kept |
| `mycelium_mlir::channel::Sender` | definition not found via regex heuristic (kind='fn', name='Sender') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Sender` | definition not found via regex heuristic (kind='fn', name='Sender') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Sender` | definition not found via regex heuristic (kind='fn', name='Sender') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Sender` | definition not found via regex heuristic (kind='fn', name='Sender') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Sender` | definition not found via regex heuristic (kind='fn', name='Sender') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Sender` | definition not found via regex heuristic (kind='fn', name='Sender') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::Sender` | dedup-alias: same definition as `mycelium_mlir::Sender` at crates/mycelium-mlir/src/channel.rs:116 — one canonical row kept |
| `mycelium_mlir::channel::TryRecv` | dedup-alias: same definition as `mycelium_mlir::TryRecv` at crates/mycelium-mlir/src/channel.rs:138 — one canonical row kept |
| `mycelium_mlir::channel::TryRecv::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::TryRecv::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::TryRecv::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::TryRecv::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::TryRecv::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::TryRecv::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::TrySend` | definition not found via regex heuristic (kind='fn', name='TrySend') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::TrySend` | definition not found via regex heuristic (kind='fn', name='TrySend') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::TrySend` | definition not found via regex heuristic (kind='fn', name='TrySend') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::TrySend` | definition not found via regex heuristic (kind='fn', name='TrySend') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::channel::TrySend` | dedup-alias: same definition as `mycelium_mlir::TrySend` at crates/mycelium-mlir/src/channel.rs:128 — one canonical row kept |
| `mycelium_mlir::compile` | ambiguous: short name 'compile' is defined in multiple modules; attributed to crates/mycelium-mlir/src/dialect/native.rs by heuristic — verify against source (ground truth) |
| `mycelium_mlir::compile_and_run` | ambiguous: short name 'compile_and_run' is defined in multiple modules; attributed to crates/mycelium-mlir/src/dialect/native.rs by heuristic — verify against source (ground truth) |
| `mycelium_mlir::deploy::DeployError` | dedup-alias: same definition as `mycelium_mlir::DeployError` at crates/mycelium-mlir/src/deploy.rs:60 — one canonical row kept |
| `mycelium_mlir::deploy::DeployError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::deploy::DeployError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::deploy::DeployError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::deploy::DeployError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::deploy::DeployError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::deploy::DeployError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::deploy::DeployError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::deploy::DeployError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::deploy::NativeArtifact` | dedup-alias: same definition as `mycelium_mlir::NativeArtifact` at crates/mycelium-mlir/src/deploy.rs:98 — one canonical row kept |
| `mycelium_mlir::deploy::NativeArtifact::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::deploy::NativeArtifact::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::deploy::NativeArtifact::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::deploy::NativeArtifact::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::deploy::NativeArtifact::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::deploy::NativeArtifact::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::dialect::emit` | dedup-alias: same definition as `mycelium_mlir::emit` at crates/mycelium-mlir/src/dialect.rs:75 — one canonical row kept |
| `mycelium_mlir::inject::Image` | dedup-alias: same definition as `mycelium_mlir::Image` at crates/mycelium-mlir/src/inject.rs:103 — one canonical row kept |
| `mycelium_mlir::inject::Image::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::inject::Image::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::inject::Image::resolve` | dedup-alias: same definition as `mycelium_mlir::DepthBudget::resolve` at crates/mycelium-mlir/src/inject.rs:172 — one canonical row kept |
| `mycelium_mlir::inject::Image::resolve` | dedup-alias: same definition as `mycelium_mlir::DepthBudget::resolve` at crates/mycelium-mlir/src/inject.rs:172 — one canonical row kept |
| `mycelium_mlir::inject::InjectError` | dedup-alias: same definition as `mycelium_mlir::InjectError` at crates/mycelium-mlir/src/inject.rs:69 — one canonical row kept |
| `mycelium_mlir::inject::InjectError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::inject::InjectError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::inject::InjectError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::inject::InjectError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::inject::InjectError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::inject::InjectError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::inject::InjectError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::inject::InjectError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::inject::Resolution` | dedup-alias: same definition as `mycelium_mlir::Resolution` at crates/mycelium-mlir/src/inject.rs:54 — one canonical row kept |
| `mycelium_mlir::inject::Resolution::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::inject::Resolution::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::inject::Resolution::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::inject::Resolution::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::inject::Resolution::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::inject::Resolution::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::inject::recompile_closure` | dedup-alias: same definition as `mycelium_mlir::recompile_closure` at crates/mycelium-mlir/src/inject.rs:226 — one canonical row kept |
| `mycelium_mlir::jit::JitArtifact` | dedup-alias: same definition as `mycelium_mlir::JitArtifact` at crates/mycelium-mlir/src/jit.rs:103 — one canonical row kept |
| `mycelium_mlir::jit::compile_so` | dedup-alias: same definition as `mycelium_mlir::compile_so` at crates/mycelium-mlir/src/jit.rs:322 — one canonical row kept |
| `mycelium_mlir::jit::jit_run` | dedup-alias: same definition as `mycelium_mlir::jit_run` at crates/mycelium-mlir/src/jit.rs:346 — one canonical row kept |
| `mycelium_mlir::llvm::AotError` | dedup-alias: same definition as `mycelium_mlir::AotError` at crates/mycelium-mlir/src/llvm.rs:60 — one canonical row kept |
| `mycelium_mlir::llvm::AotError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::llvm::AotError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::llvm::AotError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::llvm::AotError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::llvm::AotError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::llvm::AotError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::llvm::AotError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::llvm::AotError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::llvm::CompiledArtifact` | dedup-alias: same definition as `mycelium_mlir::CompiledArtifact` at crates/mycelium-mlir/src/llvm.rs:2410 — one canonical row kept |
| `mycelium_mlir::llvm::emit_llvm_ir` | dedup-alias: same definition as `mycelium_mlir::emit_llvm_ir` at crates/mycelium-mlir/src/llvm.rs:1924 — one canonical row kept |
| `mycelium_mlir::needed_bytes_for` | definition not found via regex heuristic (kind='fn', name='needed_bytes_for') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::pack::PackError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::pack::PackError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::pack::PackError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::pack::PackError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::pack::pack_trits` | dedup-alias: same definition as `mycelium_mlir::pack_trits` at crates/mycelium-mlir/src/pack.rs:201 — one canonical row kept |
| `mycelium_mlir::pack::relayout_trits` | dedup-alias: same definition as `mycelium_mlir::relayout_trits` at crates/mycelium-mlir/src/pack.rs:291 — one canonical row kept |
| `mycelium_mlir::pack::unpack_trits` | dedup-alias: same definition as `mycelium_mlir::unpack_trits` at crates/mycelium-mlir/src/pack.rs:243 — one canonical row kept |
| `mycelium_mlir::run` | ambiguous: short name 'run' is defined in multiple modules; attributed to crates/mycelium-mlir/src/aot.rs by heuristic — verify against source (ground truth) |
| `mycelium_mlir::runtime::Colony` | dedup-alias: same definition as `mycelium_mlir::Colony` at crates/mycelium-mlir/src/runtime.rs:116 — one canonical row kept |
| `mycelium_mlir::runtime::ColonyError` | dedup-alias: same definition as `mycelium_mlir::ColonyError` at crates/mycelium-mlir/src/runtime.rs:340 — one canonical row kept |
| `mycelium_mlir::runtime::ColonyError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::ColonyError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::ColonyError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::ColonyError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::ColonyError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::ColonyError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::ColonyError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::ColonyError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Deadlock` | dedup-alias: same definition as `mycelium_mlir::Deadlock` at crates/mycelium-mlir/src/runtime.rs:106 — one canonical row kept |
| `mycelium_mlir::runtime::Deadlock::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Deadlock::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Deadlock::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Deadlock::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Deadlock::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Deadlock::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Poll` | dedup-alias: same definition as `mycelium_mlir::Poll` at crates/mycelium-mlir/src/runtime.rs:37 — one canonical row kept |
| `mycelium_mlir::runtime::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Scope` | dedup-alias: same definition as `mycelium_mlir::Scope` at crates/mycelium-mlir/src/runtime.rs:84 — one canonical row kept |
| `mycelium_mlir::runtime::SweepOrder` | dedup-alias: same definition as `mycelium_mlir::SweepOrder` at crates/mycelium-mlir/src/runtime.rs:94 — one canonical row kept |
| `mycelium_mlir::runtime::SweepOrder::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::SweepOrder::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::SweepOrder::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::SweepOrder::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::SweepOrder::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::SweepOrder::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Task` | dedup-alias: same definition as `mycelium_mlir::Task` at crates/mycelium-mlir/src/runtime.rs:60 — one canonical row kept |
| `mycelium_mlir::runtime::Task::Error` | definition not found via regex heuristic (kind='type', name='Error') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Task::Output` | definition not found via regex heuristic (kind='type', name='Output') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::Task::poll` | definition not found via regex heuristic (kind='fn', name='poll') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::runtime::TaskCtx` | dedup-alias: same definition as `mycelium_mlir::TaskCtx` at crates/mycelium-mlir/src/runtime.rs:47 — one canonical row kept |
| `mycelium_mlir::runtime::run_colony` | dedup-alias: same definition as `mycelium_mlir::run_colony` at crates/mycelium-mlir/src/runtime.rs:428 — one canonical row kept |
| `mycelium_mlir::simd::compile_bitnet_dot_simd` | dedup-alias: same definition as `mycelium_mlir::compile_bitnet_dot_simd` at crates/mycelium-mlir/src/simd.rs:131 — one canonical row kept |
| `mycelium_mlir::simd::compile_bitnet_dot_simd_tl1` | dedup-alias: same definition as `mycelium_mlir::compile_bitnet_dot_simd_tl1` at crates/mycelium-mlir/src/simd.rs:248 — one canonical row kept |
| `mycelium_mlir::simd::compile_bitnet_dot_simd_tl2` | dedup-alias: same definition as `mycelium_mlir::compile_bitnet_dot_simd_tl2` at crates/mycelium-mlir/src/simd.rs:595 — one canonical row kept |
| `mycelium_mlir::simd::emit_bitnet_dot_simd_ir` | dedup-alias: same definition as `mycelium_mlir::emit_bitnet_dot_simd_ir` at crates/mycelium-mlir/src/simd.rs:59 — one canonical row kept |
| `mycelium_mlir::simd::emit_bitnet_dot_simd_tl1_ir` | dedup-alias: same definition as `mycelium_mlir::emit_bitnet_dot_simd_tl1_ir` at crates/mycelium-mlir/src/simd.rs:167 — one canonical row kept |
| `mycelium_mlir::simd::emit_bitnet_dot_simd_tl2_ir` | dedup-alias: same definition as `mycelium_mlir::emit_bitnet_dot_simd_tl2_ir` at crates/mycelium-mlir/src/simd.rs:300 — one canonical row kept |
| `mycelium_mlir::specialize::BoundSpecializedDot` | definition not found via regex heuristic (kind='fn', name='BoundSpecializedDot') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::specialize::SpecializedDotKernel` | dedup-alias: same definition as `mycelium_mlir::SpecializedDotKernel` at crates/mycelium-mlir/src/specialize.rs:92 — one canonical row kept |
| `mycelium_mlir::specialize::compile_specialized_dot` | dedup-alias: same definition as `mycelium_mlir::compile_specialized_dot` at crates/mycelium-mlir/src/specialize.rs:168 — one canonical row kept |
| `mycelium_mlir::specialize::emit_specialized_dot_ir` | dedup-alias: same definition as `mycelium_mlir::emit_specialized_dot_ir` at crates/mycelium-mlir/src/specialize.rs:57 — one canonical row kept |
| `mycelium_mlir::specialize::jit_specialized_dot` | dedup-alias: same definition as `mycelium_mlir::jit_specialized_dot` at crates/mycelium-mlir/src/specialize.rs:205 — one canonical row kept |
| `mycelium_mlir::vr4::Backend` | dedup-alias: same definition as `mycelium_mlir::Backend` at crates/mycelium-mlir/src/vr4.rs:49 — one canonical row kept |
| `mycelium_mlir::vr4::Backend::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::Backend::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::Backend::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::Backend::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::Backend::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::Backend::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::Backend::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::Backend::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::BackendStage` | dedup-alias: same definition as `mycelium_mlir::BackendStage` at crates/mycelium-mlir/src/vr4.rs:114 — one canonical row kept |
| `mycelium_mlir::vr4::BackendStage::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::BackendStage::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::BackendStage::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::BackendStage::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::BackendStage::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::BackendStage::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::CrossBackendGate` | dedup-alias: same definition as `mycelium_mlir::CrossBackendGate` at crates/mycelium-mlir/src/vr4.rs:146 — one canonical row kept |
| `mycelium_mlir::vr4::CrossBackendGate::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::CrossBackendGate::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::CrossBackendGate::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::CrossBackendGate::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::CrossBackendGate::explain` | dedup-alias: same definition as `mycelium_mlir::vr4::BackendStage::explain` at crates/mycelium-mlir/src/vr4.rs:127 — one canonical row kept |
| `mycelium_mlir::vr4::CrossBackendGate::explain` | dedup-alias: same definition as `mycelium_mlir::vr4::BackendStage::explain` at crates/mycelium-mlir/src/vr4.rs:127 — one canonical row kept |
| `mycelium_mlir::vr4::CrossBackendGate::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::CrossBackendGate::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::StageStatus` | dedup-alias: same definition as `mycelium_mlir::StageStatus` at crates/mycelium-mlir/src/vr4.rs:94 — one canonical row kept |
| `mycelium_mlir::vr4::StageStatus::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::StageStatus::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::StageStatus::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::StageStatus::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::StageStatus::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::StageStatus::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_mlir::vr4::cross_backend_gate` | dedup-alias: same definition as `mycelium_mlir::cross_backend_gate` at crates/mycelium-mlir/src/vr4.rs:195 — one canonical row kept |
| `mycelium_numerics::cert::Certificate` | dedup-alias: same definition as `mycelium_numerics::Certificate` at crates/mycelium-numerics/src/cert.rs:159 — one canonical row kept |
| `mycelium_numerics::cert::Certificate::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::Certificate::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::Certificate::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::Certificate::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::Certificate::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::Certificate::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::Certificate::exact` | definition not found via regex heuristic (kind='fn', name='exact') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::Certificate::exact` | definition not found via regex heuristic (kind='fn', name='exact') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::Certificate::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::Certificate::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::Certificate::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::Certificate::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::CheckOutcome` | dedup-alias: same definition as `mycelium_numerics::CheckOutcome` at crates/mycelium-numerics/src/cert.rs:91 — one canonical row kept |
| `mycelium_numerics::cert::CheckOutcome::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::CheckOutcome::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::CheckOutcome::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::CheckOutcome::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::CheckOutcome::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::CheckOutcome::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::ComposedBound` | dedup-alias: same definition as `mycelium_numerics::ComposedBound` at crates/mycelium-numerics/src/cert.rs:244 — one canonical row kept |
| `mycelium_numerics::cert::ComposedBound::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::ComposedBound::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::ComposedBound::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::ComposedBound::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::ComposedBound::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::ComposedBound::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::ErrorOp` | dedup-alias: same definition as `mycelium_numerics::ErrorOp` at crates/mycelium-numerics/src/cert.rs:42 — one canonical row kept |
| `mycelium_numerics::cert::ErrorOp::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::ErrorOp::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::ErrorOp::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::ErrorOp::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::ErrorOp::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::ErrorOp::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::cert::accuracy_to_probability` | dedup-alias: same definition as `mycelium_numerics::accuracy_to_probability` at crates/mycelium-numerics/src/cert.rs:148 — one canonical row kept |
| `mycelium_numerics::cert::basis_strength` | dedup-alias: same definition as `mycelium_numerics::basis_strength` at crates/mycelium-numerics/src/cert.rs:255 — one canonical row kept |
| `mycelium_numerics::cert::check_error_claim` | dedup-alias: same definition as `mycelium_numerics::check_error_claim` at crates/mycelium-numerics/src/cert.rs:109 — one canonical row kept |
| `mycelium_numerics::cert::check_union_claim` | dedup-alias: same definition as `mycelium_numerics::check_union_claim` at crates/mycelium-numerics/src/cert.rs:129 — one canonical row kept |
| `mycelium_numerics::cert::compose_error_bound` | dedup-alias: same definition as `mycelium_numerics::compose_error_bound` at crates/mycelium-numerics/src/cert.rs:322 — one canonical row kept |
| `mycelium_numerics::cert::error_norm` | dedup-alias: same definition as `mycelium_numerics::error_norm` at crates/mycelium-numerics/src/cert.rs:355 — one canonical row kept |
| `mycelium_numerics::cert::recompute_error` | dedup-alias: same definition as `mycelium_numerics::recompute_error` at crates/mycelium-numerics/src/cert.rs:64 — one canonical row kept |
| `mycelium_numerics::error::AffineForm` | dedup-alias: same definition as `mycelium_numerics::AffineForm` at crates/mycelium-numerics/src/error.rs:46 — one canonical row kept |
| `mycelium_numerics::error::AffineForm::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::error::AffineForm::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::error::AffineForm::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::error::AffineForm::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::error::AffineForm::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::error::AffineForm::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::error::ErrorBound` | dedup-alias: same definition as `mycelium_numerics::ErrorBound` at crates/mycelium-numerics/src/error.rs:221 — one canonical row kept |
| `mycelium_numerics::error::ErrorBound::add` | dedup-alias: same definition as `mycelium_numerics::error::AffineForm::add` at crates/mycelium-numerics/src/error.rs:140 — one canonical row kept |
| `mycelium_numerics::error::ErrorBound::add` | dedup-alias: same definition as `mycelium_numerics::error::AffineForm::add` at crates/mycelium-numerics/src/error.rs:140 — one canonical row kept |
| `mycelium_numerics::error::ErrorBound::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::error::ErrorBound::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::error::ErrorBound::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::error::ErrorBound::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::error::ErrorBound::exact` | definition not found via regex heuristic (kind='fn', name='exact') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::error::ErrorBound::exact` | definition not found via regex heuristic (kind='fn', name='exact') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::error::ErrorBound::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::error::ErrorBound::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::error::ErrorBound::mul` | dedup-alias: same definition as `mycelium_numerics::error::AffineForm::mul` at crates/mycelium-numerics/src/error.rs:186 — one canonical row kept |
| `mycelium_numerics::error::ErrorBound::mul` | dedup-alias: same definition as `mycelium_numerics::error::AffineForm::mul` at crates/mycelium-numerics/src/error.rs:186 — one canonical row kept |
| `mycelium_numerics::error::ErrorBound::neg` | dedup-alias: same definition as `mycelium_numerics::error::AffineForm::neg` at crates/mycelium-numerics/src/error.rs:128 — one canonical row kept |
| `mycelium_numerics::error::ErrorBound::neg` | dedup-alias: same definition as `mycelium_numerics::error::AffineForm::neg` at crates/mycelium-numerics/src/error.rs:128 — one canonical row kept |
| `mycelium_numerics::error::ErrorBound::scale` | dedup-alias: same definition as `mycelium_numerics::error::AffineForm::scale` at crates/mycelium-numerics/src/error.rs:166 — one canonical row kept |
| `mycelium_numerics::error::ErrorBound::scale` | dedup-alias: same definition as `mycelium_numerics::error::AffineForm::scale` at crates/mycelium-numerics/src/error.rs:166 — one canonical row kept |
| `mycelium_numerics::error::ErrorBound::sub` | dedup-alias: same definition as `mycelium_numerics::error::AffineForm::sub` at crates/mycelium-numerics/src/error.rs:158 — one canonical row kept |
| `mycelium_numerics::error::ErrorBound::sub` | dedup-alias: same definition as `mycelium_numerics::error::AffineForm::sub` at crates/mycelium-numerics/src/error.rs:158 — one canonical row kept |
| `mycelium_numerics::error::NoiseSym` | dedup-alias: same definition as `mycelium_numerics::NoiseSym` at crates/mycelium-numerics/src/error.rs:33 — one canonical row kept |
| `mycelium_numerics::prob::ApRhlJudgment` | dedup-alias: same definition as `mycelium_numerics::ApRhlJudgment` at crates/mycelium-numerics/src/prob.rs:69 — one canonical row kept |
| `mycelium_numerics::prob::ApRhlJudgment::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::prob::ApRhlJudgment::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::prob::ApRhlJudgment::delta` | dedup-alias: same definition as `mycelium_numerics::prob::ProbBound::delta` at crates/mycelium-numerics/src/prob.rs:24 — one canonical row kept |
| `mycelium_numerics::prob::ApRhlJudgment::delta` | dedup-alias: same definition as `mycelium_numerics::prob::ProbBound::delta` at crates/mycelium-numerics/src/prob.rs:24 — one canonical row kept |
| `mycelium_numerics::prob::ApRhlJudgment::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::prob::ApRhlJudgment::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::prob::ApRhlJudgment::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::prob::ApRhlJudgment::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::prob::ApRhlJudgment::new` | dedup-alias: same definition as `mycelium_numerics::prob::ProbBound::new` at crates/mycelium-numerics/src/prob.rs:36 — one canonical row kept |
| `mycelium_numerics::prob::ApRhlJudgment::new` | dedup-alias: same definition as `mycelium_numerics::prob::ProbBound::new` at crates/mycelium-numerics/src/prob.rs:36 — one canonical row kept |
| `mycelium_numerics::prob::ProbBound` | dedup-alias: same definition as `mycelium_numerics::ProbBound` at crates/mycelium-numerics/src/prob.rs:16 — one canonical row kept |
| `mycelium_numerics::prob::ProbBound::certain` | definition not found via regex heuristic (kind='fn', name='certain') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::prob::ProbBound::certain` | definition not found via regex heuristic (kind='fn', name='certain') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::prob::ProbBound::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::prob::ProbBound::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::prob::ProbBound::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::prob::ProbBound::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::prob::ProbBound::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_numerics::prob::ProbBound::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::Deprecated` | dedup-alias: same definition as `mycelium_proj::Deprecated` at crates/mycelium-proj/src/header.rs:31 — one canonical row kept |
| `mycelium_proj::header::Deprecated::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::Deprecated::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::Deprecated::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::Deprecated::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::Deprecated::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::Deprecated::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HEADER_KEYS:` | dedup-alias: same definition as `mycelium_proj::HEADER_KEYS:` at crates/mycelium-proj/src/header.rs:16 — one canonical row kept |
| `mycelium_proj::header::HeaderError` | dedup-alias: same definition as `mycelium_proj::HeaderError` at crates/mycelium-proj/src/header.rs:74 — one canonical row kept |
| `mycelium_proj::header::HeaderError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderFields` | dedup-alias: same definition as `mycelium_proj::HeaderFields` at crates/mycelium-proj/src/header.rs:40 — one canonical row kept |
| `mycelium_proj::header::HeaderFields::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderFields::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderFields::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderFields::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderFields::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderFields::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderFields::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::HeaderFields::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::StructuredHeader` | dedup-alias: same definition as `mycelium_proj::StructuredHeader` at crates/mycelium-proj/src/header.rs:65 — one canonical row kept |
| `mycelium_proj::header::StructuredHeader::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::StructuredHeader::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::StructuredHeader::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::StructuredHeader::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::StructuredHeader::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::StructuredHeader::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::header::parse_header` | dedup-alias: same definition as `mycelium_proj::parse_header` at crates/mycelium-proj/src/header.rs:107 — one canonical row kept |
| `mycelium_proj::manifest::Dependency` | dedup-alias: same definition as `mycelium_proj::Dependency` at crates/mycelium-proj/src/manifest.rs:80 — one canonical row kept |
| `mycelium_proj::manifest::Dependency::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Dependency::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Dependency::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Dependency::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Dependency::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Dependency::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Manifest` | dedup-alias: same definition as `mycelium_proj::Manifest` at crates/mycelium-proj/src/manifest.rs:102 — one canonical row kept |
| `mycelium_proj::manifest::Manifest::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Manifest::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Manifest::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Manifest::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Manifest::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Manifest::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::ManifestError` | dedup-alias: same definition as `mycelium_proj::ManifestError` at crates/mycelium-proj/src/manifest.rs:117 — one canonical row kept |
| `mycelium_proj::manifest::ManifestError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::ManifestError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::ManifestError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::ManifestError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::ManifestError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::ManifestError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::ManifestError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::ManifestError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Project` | dedup-alias: same definition as `mycelium_proj::Project` at crates/mycelium-proj/src/manifest.rs:33 — one canonical row kept |
| `mycelium_proj::manifest::Project::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Project::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Project::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Project::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Project::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Project::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::ProjectKind` | dedup-alias: same definition as `mycelium_proj::ProjectKind` at crates/mycelium-proj/src/manifest.rs:22 — one canonical row kept |
| `mycelium_proj::manifest::ProjectKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::ProjectKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::ProjectKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::ProjectKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::ProjectKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::ProjectKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::SporeConfig` | dedup-alias: same definition as `mycelium_proj::SporeConfig` at crates/mycelium-proj/src/manifest.rs:94 — one canonical row kept |
| `mycelium_proj::manifest::SporeConfig::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::SporeConfig::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::SporeConfig::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::SporeConfig::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::SporeConfig::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::SporeConfig::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::SporeConfig::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::SporeConfig::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Surface` | dedup-alias: same definition as `mycelium_proj::Surface` at crates/mycelium-proj/src/manifest.rs:71 — one canonical row kept |
| `mycelium_proj::manifest::Surface::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Surface::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Surface::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Surface::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Surface::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Surface::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Surface::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Surface::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Toolchain` | dedup-alias: same definition as `mycelium_proj::Toolchain` at crates/mycelium-proj/src/manifest.rs:60 — one canonical row kept |
| `mycelium_proj::manifest::Toolchain::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Toolchain::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Toolchain::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Toolchain::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Toolchain::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Toolchain::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Toolchain::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::Toolchain::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::manifest::parse_manifest` | dedup-alias: same definition as `mycelium_proj::parse_manifest` at crates/mycelium-proj/src/manifest.rs:160 — one canonical row kept |
| `mycelium_proj::resolve::Origin` | dedup-alias: same definition as `mycelium_proj::Origin` at crates/mycelium-proj/src/resolve.rs:22 — one canonical row kept |
| `mycelium_proj::resolve::Origin::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::Origin::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::Origin::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::Origin::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::Origin::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::Origin::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::Resolved` | definition not found via regex heuristic (kind='fn', name='Resolved') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::Resolved` | definition not found via regex heuristic (kind='fn', name='Resolved') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::Resolved` | definition not found via regex heuristic (kind='fn', name='Resolved') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::Resolved` | definition not found via regex heuristic (kind='fn', name='Resolved') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::Resolved` | definition not found via regex heuristic (kind='fn', name='Resolved') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::Resolved` | definition not found via regex heuristic (kind='fn', name='Resolved') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::Resolved` | dedup-alias: same definition as `mycelium_proj::Resolved` at crates/mycelium-proj/src/resolve.rs:40 — one canonical row kept |
| `mycelium_proj::resolve::ResolvedHeader` | dedup-alias: same definition as `mycelium_proj::ResolvedHeader` at crates/mycelium-proj/src/resolve.rs:49 — one canonical row kept |
| `mycelium_proj::resolve::ResolvedHeader::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::ResolvedHeader::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::ResolvedHeader::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::ResolvedHeader::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::ResolvedHeader::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::ResolvedHeader::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::ResolvedHeader::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::ResolvedHeader::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_proj::resolve::explain` | dedup-alias: same definition as `mycelium_proj::explain` at crates/mycelium-proj/src/resolve.rs:145 — one canonical row kept |
| `mycelium_proj::resolve::resolve` | dedup-alias: same definition as `mycelium_proj::resolve` at crates/mycelium-proj/src/resolve.rs:76 — one canonical row kept |
| `mycelium_sec::Finding::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_sec::Finding::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_sec::Finding::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_sec::Severity::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_sec::Severity::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_sec::Severity::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_sec::Severity::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_sec::Severity::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_sec::WildAudit::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_sec::WildAudit::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_sec::WildAudit::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_sec::WildAudit::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_sec::WildBlock::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_sec::WildBlock::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_sec::WildBlock::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::Action::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_select::Action::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::Action::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_select::Action::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::Action::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::Candidate::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_select::Candidate::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::Candidate::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_select::Candidate::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::Candidate::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::CandidateCost::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_select::CandidateCost::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::CandidateCost::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_select::CandidateCost::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::CandidateCost::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::CostModel::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_select::CostModel::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::CostModel::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_select::CostModel::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::CostModel::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::DecodeFacts::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_select::DecodeFacts::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::DecodeFacts::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_select::DecodeFacts::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::DecodeFacts::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::DecodeMethod::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_select::DecodeMethod::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::DecodeMethod::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_select::DecodeMethod::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::DecodeMethod::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::Explanation::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_select::Explanation::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::Explanation::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_select::Explanation::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::Explanation::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::ParadigmKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_select::ParadigmKind::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::ParadigmKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_select::ParadigmKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::ParadigmKind::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::PolicyError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_select::PolicyError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_select::PolicyError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::PolicyError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::PolicyRegistry::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_select::PolicyRegistry::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::Predicate::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_select::Predicate::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::Predicate::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_select::Predicate::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::Predicate::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::Rule::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_select::Rule::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::Rule::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_select::Rule::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::Rule::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::SelectError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_select::SelectError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_select::SelectError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::SelectError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::SelectionInputs::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_select::SelectionInputs::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::SelectionInputs::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_select::SelectionInputs::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::SelectionInputs::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::SelectionPolicy::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_select::SelectionPolicy::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_select::SelectionPolicy::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_select::SelectionPolicy::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_select::SelectionPolicy::new` | dedup-alias: same definition as `mycelium_select::PolicyRegistry::new` at crates/mycelium-select/src/lib.rs:431 — one canonical row kept |
| `mycelium_select::SelectionPolicy::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_spore::ResolvedDep::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_spore::ResolvedDep::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_spore::ResolvedDep::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_spore::SourceFile::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_spore::SourceFile::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_spore::SourceFile::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_spore::Spore::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_spore::Spore::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_spore::Spore::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_spore::SporeError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_spore::SporeError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_spore::SporeError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_spore::SporeError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::Bf16Bits::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::Bf16Bits::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::Bf16Bits::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::Bf16Bits::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::Bf16Bits::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::Bf16Bits::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::ClampError` | definition not found via regex heuristic (kind='fn', name='ClampError') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::ClampError` | definition not found via regex heuristic (kind='fn', name='ClampError') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::ClampError` | definition not found via regex heuristic (kind='fn', name='ClampError') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::ClampError` | definition not found via regex heuristic (kind='fn', name='ClampError') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::MatrixRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::MatrixRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::MatrixRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::MycEq::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::MycEq::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::MycOrd::myc_cmp` | definition not found via regex heuristic (kind='fn', name='myc_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::MycOrd::myc_ge` | definition not found via regex heuristic (kind='fn', name='myc_ge') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::MycOrd::myc_gt` | definition not found via regex heuristic (kind='fn', name='myc_gt') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::MycOrd::myc_le` | definition not found via regex heuristic (kind='fn', name='myc_le') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::MycOrd::myc_lt` | definition not found via regex heuristic (kind='fn', name='myc_lt') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::MycPartialOrd::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::Narrow::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::NarrowError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::NarrowError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::NarrowError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::NarrowError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::Ordering::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::Ordering::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::Ordering::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::Ordering::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::Ordering::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_cmp::Widen::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::error::CollErr` | dedup-alias: same definition as `mycelium_std_collections::CollErr` at crates/mycelium-std-collections/src/error.rs:23 — one canonical row kept |
| `mycelium_std_collections::error::CollErr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::error::CollErr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::error::CollErr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::error::CollErr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::error::CollErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::error::CollErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::error::CollErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::error::CollErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::guarantee_matrix::Explainable::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::guarantee_matrix::Explainable::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::guarantee_matrix::Explainable::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::guarantee_matrix::Fallibility::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::guarantee_matrix::Fallibility::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::guarantee_matrix::Fallibility::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::guarantee_matrix::MatrixRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::guarantee_matrix::MatrixRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::guarantee_matrix::MatrixRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | definition not found via regex heuristic (kind='fn', name='Map') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::map::Map` | dedup-alias: same definition as `mycelium_std_collections::Map` at crates/mycelium-std-collections/src/map.rs:56 — one canonical row kept |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | definition not found via regex heuristic (kind='fn', name='Seq') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::seq::Seq` | dedup-alias: same definition as `mycelium_std_collections::Seq` at crates/mycelium-std-collections/src/seq.rs:36 — one canonical row kept |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | definition not found via regex heuristic (kind='fn', name='Set') — possibly macro-generated or cfg-gated |
| `mycelium_std_collections::set::Set` | dedup-alias: same definition as `mycelium_std_collections::Set` at crates/mycelium-std-collections/src/set.rs:45 — one canonical row kept |
| `mycelium_std_content::ContentHash` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_content::Names` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_content::content_ref::ContentRef` | dedup-alias: same definition as `mycelium_std_content::ContentRef` at crates/mycelium-std-content/src/content_ref.rs:48 — one canonical row kept |
| `mycelium_std_content::content_ref::ContentRef::Err` | definition not found via regex heuristic (kind='type', name='Err') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::ContentRef::Err` | definition not found via regex heuristic (kind='type', name='Err') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::ContentRef::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::ContentRef::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::ContentRef::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::ContentRef::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::ContentRef::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::ContentRef::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::ContentRef::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::ContentRef::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::ContentRef::from_str` | definition not found via regex heuristic (kind='fn', name='from_str') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::ContentRef::from_str` | definition not found via regex heuristic (kind='fn', name='from_str') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::ContentRef::hash` | dedup-alias: same definition as `mycelium_std_content::content_ref::RefKind::hash` at crates/mycelium-std-content/src/content_ref.rs:70 — one canonical row kept |
| `mycelium_std_content::content_ref::ContentRef::hash` | dedup-alias: same definition as `mycelium_std_content::content_ref::RefKind::hash` at crates/mycelium-std-content/src/content_ref.rs:70 — one canonical row kept |
| `mycelium_std_content::content_ref::ContentRef::hash` | dedup-alias: same definition as `mycelium_std_content::content_ref::RefKind::hash` at crates/mycelium-std-content/src/content_ref.rs:70 — one canonical row kept |
| `mycelium_std_content::content_ref::ContentRef::hash` | dedup-alias: same definition as `mycelium_std_content::content_ref::RefKind::hash` at crates/mycelium-std-content/src/content_ref.rs:70 — one canonical row kept |
| `mycelium_std_content::content_ref::ContentRef::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::ContentRef::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::RefKind` | dedup-alias: same definition as `mycelium_std_content::RefKind` at crates/mycelium-std-content/src/content_ref.rs:27 — one canonical row kept |
| `mycelium_std_content::content_ref::RefKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::RefKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::RefKind::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::RefKind::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::RefKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::RefKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::RefKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::RefKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::RefKind::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::content_ref::RefKind::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::error::MalformedDigest` | dedup-alias: same definition as `mycelium_std_content::MalformedDigest` at crates/mycelium-std-content/src/error.rs:18 — one canonical row kept |
| `mycelium_std_content::error::MalformedDigest::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::error::MalformedDigest::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::error::MalformedDigest::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::error::MalformedDigest::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::error::MalformedDigest::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::error::MalformedDigest::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::error::MalformedDigest::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::error::MalformedDigest::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::guarantee_matrix::Explainable::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::guarantee_matrix::Explainable::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::guarantee_matrix::Explainable::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::guarantee_matrix::Fallibility::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::guarantee_matrix::Fallibility::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::guarantee_matrix::Fallibility::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::guarantee_matrix::MatrixRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::guarantee_matrix::MatrixRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::guarantee_matrix::MatrixRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::name_registry::NameRegistry` | dedup-alias: same definition as `mycelium_std_content::NameRegistry` at crates/mycelium-std-content/src/name_registry.rs:36 — one canonical row kept |
| `mycelium_std_content::name_registry::NameRegistry::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::name_registry::NameRegistry::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::name_registry::NameRegistry::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::name_registry::NameRegistry::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::name_registry::NameRegistry::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_content::name_registry::NameRegistry::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_core::Bound` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::BoundBasis` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::BoundKind` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::ContentHash` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::CoreValue` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::Datum` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::GuaranteeRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_core::GuaranteeRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_core::GuaranteeRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_core::GuaranteeStrength` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::Meta` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::NormKind` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::PackScheme` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::Payload` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::PhysicalLayout` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::Provenance` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::Repr` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::ScalarKind` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::SparsityClass` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::SparsityObs` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::Trit` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::Value` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::prelude::Bound` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::prelude::BoundBasis` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::prelude::BoundKind` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::prelude::CoreValue` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::prelude::Datum` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::prelude::GuaranteeStrength` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::prelude::Meta` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::prelude::NormKind` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::prelude::Payload` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::prelude::Provenance` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::prelude::Repr` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::prelude::Trit` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::prelude::Value` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_core::prelude::bound_of` | dedup-alias: same definition as `mycelium_std_core::bound_of` at crates/mycelium-std-core/src/lib.rs:105 — one canonical row kept |
| `mycelium_std_core::prelude::guarantee_of` | dedup-alias: same definition as `mycelium_std_core::guarantee_of` at crates/mycelium-std-core/src/lib.rs:99 — one canonical row kept |
| `mycelium_std_core::prelude::meta_of` | dedup-alias: same definition as `mycelium_std_core::meta_of` at crates/mycelium-std-core/src/lib.rs:93 — one canonical row kept |
| `mycelium_std_core::prelude::provenance_of` | dedup-alias: same definition as `mycelium_std_core::provenance_of` at crates/mycelium-std-core/src/lib.rs:111 — one canonical row kept |
| `mycelium_std_core::prelude::repr_of` | dedup-alias: same definition as `mycelium_std_core::repr_of` at crates/mycelium-std-core/src/lib.rs:87 — one canonical row kept |
| `mycelium_std_dense::BF16_OP_REL_EPS` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_dense::Bound` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_dense::BoundBasis` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_dense::BoundKind` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_dense::DENSE_MIN_NORMAL` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_dense::DenseError` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_dense::DenseOp` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_dense::DenseSpace` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_dense::F32_OP_REL_EPS` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_dense::GuaranteeRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_dense::GuaranteeRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_dense::GuaranteeRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_dense::GuaranteeStrength` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_dense::NormKind` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_dense::OpBound::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_dense::OpBound::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_dense::OpBound::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_dense::ScalarKind` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_dense::StdDense::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_dense::StdDenseError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_dense::StdDenseError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_dense::StdDenseError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_dense::StdDenseError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_dense::StdDenseError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_diag::Code` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_diag::ContentHash` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_diag::Diag` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_diag::Locus` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_diag::Severity` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_diag::Trace` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_diag::guarantee_matrix::Explainable::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_diag::guarantee_matrix::Explainable::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_diag::guarantee_matrix::Explainable::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_diag::guarantee_matrix::Fallibility::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_diag::guarantee_matrix::Fallibility::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_diag::guarantee_matrix::Fallibility::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_diag::guarantee_matrix::MatrixRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_diag::guarantee_matrix::MatrixRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_diag::guarantee_matrix::MatrixRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::GuaranteeStrength` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_error::Outcome` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_error::PolicyRef` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_error::RecoverOutcome` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_error::Resolution` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_error::combinators::RefusalRecord` | dedup-alias: same definition as `mycelium_std_error::RefusalRecord` at crates/mycelium-std-error/src/combinators.rs:38 — one canonical row kept |
| `mycelium_std_error::combinators::RefusalRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::combinators::RefusalRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::combinators::RefusalRecord::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::combinators::RefusalRecord::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::combinators::RefusalRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::combinators::RefusalRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::combinators::RefusalRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::combinators::RefusalRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::combinators::SubstitutionRecord` | dedup-alias: same definition as `mycelium_std_error::SubstitutionRecord` at crates/mycelium-std-error/src/combinators.rs:69 — one canonical row kept |
| `mycelium_std_error::combinators::SubstitutionRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::combinators::SubstitutionRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::combinators::SubstitutionRecord::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::combinators::SubstitutionRecord::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::combinators::SubstitutionRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::combinators::SubstitutionRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::combinators::and_then` | dedup-alias: same definition as `mycelium_std_error::and_then` at crates/mycelium-std-error/src/combinators.rs:111 — one canonical row kept |
| `mycelium_std_error::combinators::expect` | dedup-alias: same definition as `mycelium_std_error::expect` at crates/mycelium-std-error/src/combinators.rs:362 — one canonical row kept |
| `mycelium_std_error::combinators::filter` | dedup-alias: same definition as `mycelium_std_error::filter` at crates/mycelium-std-error/src/combinators.rs:139 — one canonical row kept |
| `mycelium_std_error::combinators::flatten` | dedup-alias: same definition as `mycelium_std_error::flatten` at crates/mycelium-std-error/src/combinators.rs:236 — one canonical row kept |
| `mycelium_std_error::combinators::inspect` | dedup-alias: same definition as `mycelium_std_error::inspect` at crates/mycelium-std-error/src/combinators.rs:152 — one canonical row kept |
| `mycelium_std_error::combinators::inspect_err` | dedup-alias: same definition as `mycelium_std_error::inspect_err` at crates/mycelium-std-error/src/combinators.rs:165 — one canonical row kept |
| `mycelium_std_error::combinators::map` | dedup-alias: same definition as `mycelium_std_error::map` at crates/mycelium-std-error/src/combinators.rs:84 — one canonical row kept |
| `mycelium_std_error::combinators::map_err` | dedup-alias: same definition as `mycelium_std_error::map_err` at crates/mycelium-std-error/src/combinators.rs:97 — one canonical row kept |
| `mycelium_std_error::combinators::ok` | dedup-alias: same definition as `mycelium_std_error::ok` at crates/mycelium-std-error/src/combinators.rs:214 — one canonical row kept |
| `mycelium_std_error::combinators::ok_or` | dedup-alias: same definition as `mycelium_std_error::ok_or` at crates/mycelium-std-error/src/combinators.rs:181 — one canonical row kept |
| `mycelium_std_error::combinators::ok_or_else` | dedup-alias: same definition as `mycelium_std_error::ok_or_else` at crates/mycelium-std-error/src/combinators.rs:192 — one canonical row kept |
| `mycelium_std_error::combinators::or_else` | dedup-alias: same definition as `mycelium_std_error::or_else` at crates/mycelium-std-error/src/combinators.rs:125 — one canonical row kept |
| `mycelium_std_error::combinators::transpose` | dedup-alias: same definition as `mycelium_std_error::transpose` at crates/mycelium-std-error/src/combinators.rs:225 — one canonical row kept |
| `mycelium_std_error::combinators::unwrap` | dedup-alias: same definition as `mycelium_std_error::unwrap` at crates/mycelium-std-error/src/combinators.rs:335 — one canonical row kept |
| `mycelium_std_error::combinators::unwrap_err` | dedup-alias: same definition as `mycelium_std_error::unwrap_err` at crates/mycelium-std-error/src/combinators.rs:387 — one canonical row kept |
| `mycelium_std_error::combinators::unwrap_or` | dedup-alias: same definition as `mycelium_std_error::unwrap_or` at crates/mycelium-std-error/src/combinators.rs:260 — one canonical row kept |
| `mycelium_std_error::combinators::unwrap_or_else` | dedup-alias: same definition as `mycelium_std_error::unwrap_or_else` at crates/mycelium-std-error/src/combinators.rs:276 — one canonical row kept |
| `mycelium_std_error::combinators::unwrap_or_else_option` | dedup-alias: same definition as `mycelium_std_error::unwrap_or_else_option` at crates/mycelium-std-error/src/combinators.rs:307 — one canonical row kept |
| `mycelium_std_error::combinators::unwrap_or_option` | dedup-alias: same definition as `mycelium_std_error::unwrap_or_option` at crates/mycelium-std-error/src/combinators.rs:293 — one canonical row kept |
| `mycelium_std_error::combinators::zip` | dedup-alias: same definition as `mycelium_std_error::zip` at crates/mycelium-std-error/src/combinators.rs:246 — one canonical row kept |
| `mycelium_std_error::guarantee_matrix::Explainable::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::guarantee_matrix::Explainable::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::guarantee_matrix::Explainable::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::guarantee_matrix::Fallibility::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::guarantee_matrix::Fallibility::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::guarantee_matrix::Fallibility::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::guarantee_matrix::MatrixRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::guarantee_matrix::MatrixRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::guarantee_matrix::MatrixRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_error::handle_classified` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_fmt::Budget::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Budget::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Budget::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Budget::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Budget::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::FromJsonError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::FromJsonError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::FromJsonError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::FromJsonError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Json::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Json::deserialize` | definition not found via regex heuristic (kind='fn', name='deserialize') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Json::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Json::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Json::serialize` | definition not found via regex heuristic (kind='fn', name='serialize') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::MatrixRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::MatrixRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::MatrixRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Payload` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_fmt::Rendering::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Rendering::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Rendering::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Text::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Text::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Text::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Text::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::ToJsonError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::ToJsonError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::ToJsonError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::ToJsonError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Trit` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_fmt::Truncation::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Truncation::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fmt::Truncation::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::DirIter::Item` | definition not found via regex heuristic (kind='type', name='Item') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::DirIter::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::DirIter::next` | definition not found via regex heuristic (kind='fn', name='next') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::File::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::File::path` | dedup-alias: same definition as `mycelium_std_fs::Fs::path` at crates/mycelium-std-fs/src/lib.rs:94 — one canonical row kept |
| `mycelium_std_fs::error::ErrnoClass` | dedup-alias: same definition as `mycelium_std_fs::ErrnoClass` at crates/mycelium-std-fs/src/error.rs:23 — one canonical row kept |
| `mycelium_std_fs::error::ErrnoClass::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::error::ErrnoClass::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::error::ErrnoClass::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::error::ErrnoClass::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::error::ErrnoClass::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::error::ErrnoClass::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::error::ErrnoClass::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::error::ErrnoClass::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::error::FsErr` | dedup-alias: same definition as `mycelium_std_fs::FsErr` at crates/mycelium-std-fs/src/error.rs:85 — one canonical row kept |
| `mycelium_std_fs::error::FsErr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::error::FsErr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::error::FsErr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::error::FsErr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::error::FsErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::error::FsErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::error::FsErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::error::FsErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Effects` | dedup-alias: same definition as `mycelium_std_fs::Effects` at crates/mycelium-std-fs/src/guarantee_matrix.rs:58 — one canonical row kept |
| `mycelium_std_fs::guarantee_matrix::Effects::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Effects::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Effects::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Effects::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Effects::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Effects::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Explainable` | dedup-alias: same definition as `mycelium_std_fs::Explainable` at crates/mycelium-std-fs/src/guarantee_matrix.rs:49 — one canonical row kept |
| `mycelium_std_fs::guarantee_matrix::Explainable::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Explainable::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Explainable::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Explainable::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Explainable::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Explainable::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Fallibility` | dedup-alias: same definition as `mycelium_std_fs::Fallibility` at crates/mycelium-std-fs/src/guarantee_matrix.rs:29 — one canonical row kept |
| `mycelium_std_fs::guarantee_matrix::Fallibility::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Fallibility::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Fallibility::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Fallibility::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Fallibility::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Fallibility::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::MATRIX:` | dedup-alias: same definition as `mycelium_std_fs::MATRIX:` at crates/mycelium-std-fs/src/guarantee_matrix.rs:92 — one canonical row kept |
| `mycelium_std_fs::guarantee_matrix::MatrixRow` | dedup-alias: same definition as `mycelium_std_fs::MatrixRow` at crates/mycelium-std-fs/src/guarantee_matrix.rs:67 — one canonical row kept |
| `mycelium_std_fs::guarantee_matrix::MatrixRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::MatrixRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::MatrixRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::MatrixRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::MatrixRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::MatrixRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Wild` | dedup-alias: same definition as `mycelium_std_fs::Wild` at crates/mycelium-std-fs/src/guarantee_matrix.rs:40 — one canonical row kept |
| `mycelium_std_fs::guarantee_matrix::Wild::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Wild::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Wild::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Wild::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Wild::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::guarantee_matrix::Wild::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::FileKind` | dedup-alias: same definition as `mycelium_std_fs::FileKind` at crates/mycelium-std-fs/src/metadata.rs:15 — one canonical row kept |
| `mycelium_std_fs::metadata::FileKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::FileKind::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::FileKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::FileKind::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::FileKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::FileKind::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::FileKind::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::FileKind::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::Metadata` | dedup-alias: same definition as `mycelium_std_fs::Metadata` at crates/mycelium-std-fs/src/metadata.rs:97 — one canonical row kept |
| `mycelium_std_fs::metadata::Metadata::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::Metadata::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::Metadata::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::Metadata::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::Metadata::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::Metadata::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::Permissions` | dedup-alias: same definition as `mycelium_std_fs::Permissions` at crates/mycelium-std-fs/src/metadata.rs:32 — one canonical row kept |
| `mycelium_std_fs::metadata::Permissions::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::Permissions::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::Permissions::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::Permissions::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::Permissions::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::Permissions::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::Permissions::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::metadata::Permissions::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::options::OpenOptions` | dedup-alias: same definition as `mycelium_std_fs::OpenOptions` at crates/mycelium-std-fs/src/options.rs:28 — one canonical row kept |
| `mycelium_std_fs::options::OpenOptions::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::options::OpenOptions::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::options::OpenOptions::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::options::OpenOptions::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::options::OpenOptions::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::options::OpenOptions::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::options::OpenOptions::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::options::OpenOptions::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::path::Path` | dedup-alias: same definition as `mycelium_std_fs::Path` at crates/mycelium-std-fs/src/path.rs:34 — one canonical row kept |
| `mycelium_std_fs::path::Path::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::path::Path::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::path::Path::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::path::Path::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::path::Path::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::path::Path::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::path::Path::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::path::Path::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::path::Path::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::path::Path::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::path::Path::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::path::Path::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::path::Path::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_fs::path::Path::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::GUARANTEE_MATRIX:` | definition not found via regex heuristic (kind='const', name='GUARANTEE_MATRIX:') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ByteCount` | dedup-alias: same definition as `mycelium_std_io::ByteCount` at crates/mycelium-std-io/src/error.rs:149 — one canonical row kept |
| `mycelium_std_io::error::ByteCount::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ByteCount::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ByteCount::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ByteCount::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ByteCount::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ByteCount::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ByteCount::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ByteCount::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ByteOffset` | dedup-alias: same definition as `mycelium_std_io::ByteOffset` at crates/mycelium-std-io/src/error.rs:27 — one canonical row kept |
| `mycelium_std_io::error::ByteOffset::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ByteOffset::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ByteOffset::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ByteOffset::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ByteOffset::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ByteOffset::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ByteOffset::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ByteOffset::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::FieldPath` | dedup-alias: same definition as `mycelium_std_io::FieldPath` at crates/mycelium-std-io/src/error.rs:42 — one canonical row kept |
| `mycelium_std_io::error::FieldPath::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::FieldPath::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::FieldPath::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::FieldPath::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::FieldPath::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::FieldPath::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::FieldPath::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::FieldPath::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::IoError` | dedup-alias: same definition as `mycelium_std_io::IoError` at crates/mycelium-std-io/src/error.rs:162 — one canonical row kept |
| `mycelium_std_io::error::IoError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::IoError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::IoError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::IoError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::IoError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::IoError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::IoError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::IoError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError` | dedup-alias: same definition as `mycelium_std_io::ReadValueError` at crates/mycelium-std-io/src/error.rs:206 — one canonical row kept |
| `mycelium_std_io::error::ReadValueError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::source` | definition not found via regex heuristic (kind='fn', name='source') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::ReadValueError::source` | definition not found via regex heuristic (kind='fn', name='source') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::SerError` | dedup-alias: same definition as `mycelium_std_io::SerError` at crates/mycelium-std-io/src/error.rs:71 — one canonical row kept |
| `mycelium_std_io::error::SerError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::SerError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::SerError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::SerError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::SerError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::SerError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::SerError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::error::SerError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::guarantee_matrix::Explainable::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::guarantee_matrix::Explainable::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::guarantee_matrix::Explainable::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::guarantee_matrix::Fallibility::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::guarantee_matrix::Fallibility::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::guarantee_matrix::Fallibility::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::guarantee_matrix::GuaranteeTag::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::guarantee_matrix::GuaranteeTag::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::guarantee_matrix::GuaranteeTag::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::guarantee_matrix::MatrixRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::guarantee_matrix::MatrixRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::guarantee_matrix::MatrixRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::io::Budget` | dedup-alias: same definition as `mycelium_std_io::Budget` at crates/mycelium-std-io/src/io.rs:52 — one canonical row kept |
| `mycelium_std_io::io::Budget::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::io::Budget::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::io::Budget::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::io::Budget::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::io::Budget::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::io::Budget::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::io::Sink` | dedup-alias: same definition as `mycelium_std_io::Sink` at crates/mycelium-std-io/src/io.rs:170 — one canonical row kept |
| `mycelium_std_io::io::Sink::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::io::Sink::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::io::Sink::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::io::Sink::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::io::Source` | dedup-alias: same definition as `mycelium_std_io::Source` at crates/mycelium-std-io/src/io.rs:115 — one canonical row kept |
| `mycelium_std_io::io::Source::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::io::Source::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::io::Source::new` | dedup-alias: same definition as `mycelium_std_io::io::Sink::new` at crates/mycelium-std-io/src/io.rs:124 — one canonical row kept |
| `mycelium_std_io::io::Source::new` | dedup-alias: same definition as `mycelium_std_io::io::Sink::new` at crates/mycelium-std-io/src/io.rs:124 — one canonical row kept |
| `mycelium_std_io::io::Substrate` | dedup-alias: same definition as `mycelium_std_io::Substrate` at crates/mycelium-std-io/src/io.rs:76 — one canonical row kept |
| `mycelium_std_io::io::Substrate::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::io::Substrate::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::io::Substrate::from_bytes` | dedup-alias: same definition as `mycelium_std_io::io::Source::from_bytes` at crates/mycelium-std-io/src/io.rs:89 — one canonical row kept |
| `mycelium_std_io::io::Substrate::from_bytes` | dedup-alias: same definition as `mycelium_std_io::io::Source::from_bytes` at crates/mycelium-std-io/src/io.rs:89 — one canonical row kept |
| `mycelium_std_io::io::Substrate::remaining` | dedup-alias: same definition as `mycelium_std_io::io::Source::remaining` at crates/mycelium-std-io/src/io.rs:98 — one canonical row kept |
| `mycelium_std_io::io::Substrate::remaining` | dedup-alias: same definition as `mycelium_std_io::io::Source::remaining` at crates/mycelium-std-io/src/io.rs:98 — one canonical row kept |
| `mycelium_std_io::io::read` | dedup-alias: same definition as `mycelium_std_io::read` at crates/mycelium-std-io/src/io.rs:250 — one canonical row kept |
| `mycelium_std_io::io::read_all` | dedup-alias: same definition as `mycelium_std_io::read_all` at crates/mycelium-std-io/src/io.rs:226 — one canonical row kept |
| `mycelium_std_io::io::read_value` | dedup-alias: same definition as `mycelium_std_io::read_value` at crates/mycelium-std-io/src/io.rs:291 — one canonical row kept |
| `mycelium_std_io::io::write` | dedup-alias: same definition as `mycelium_std_io::write` at crates/mycelium-std-io/src/io.rs:274 — one canonical row kept |
| `mycelium_std_io::serialize::Format` | dedup-alias: same definition as `mycelium_std_io::Format` at crates/mycelium-std-io/src/serialize.rs:59 — one canonical row kept |
| `mycelium_std_io::serialize::Format::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::serialize::Format::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::serialize::Format::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::serialize::Format::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::serialize::Format::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::serialize::Format::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_io::serialize::deserialize` | dedup-alias: same definition as `mycelium_std_io::deserialize` at crates/mycelium-std-io/src/serialize.rs:155 — one canonical row kept |
| `mycelium_std_io::serialize::from_json` | dedup-alias: same definition as `mycelium_std_io::from_json` at crates/mycelium-std-io/src/serialize.rs:194 — one canonical row kept |
| `mycelium_std_io::serialize::serialize` | dedup-alias: same definition as `mycelium_std_io::serialize` at crates/mycelium-std-io/src/serialize.rs:118 — one canonical row kept |
| `mycelium_std_io::serialize::to_json` | dedup-alias: same definition as `mycelium_std_io::to_json` at crates/mycelium-std-io/src/serialize.rs:179 — one canonical row kept |
| `mycelium_std_iter::AnyAllWitness::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::AnyAllWitness::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::AnyAllWitness::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::GuaranteeStrength` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_iter::error::ZeroStep` | dedup-alias: same definition as `mycelium_std_iter::ZeroStep` at crates/mycelium-std-iter/src/error.rs:12 — one canonical row kept |
| `mycelium_std_iter::error::ZeroStep::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::error::ZeroStep::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::error::ZeroStep::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::error::ZeroStep::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::error::ZeroStep::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::error::ZeroStep::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::error::ZeroStep::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::error::ZeroStep::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::error::ZipLengthMismatch::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::error::ZipLengthMismatch::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::error::ZipLengthMismatch::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::error::ZipLengthMismatch::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | definition not found via regex heuristic (kind='fn', name='Foldable') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::foldable::Foldable` | dedup-alias: same definition as `mycelium_std_iter::Foldable` at crates/mycelium-std-iter/src/foldable.rs:15 — one canonical row kept |
| `mycelium_std_iter::guarantee_matrix::GuaranteeRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::guarantee_matrix::GuaranteeRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::guarantee_matrix::GuaranteeRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::lazy::Lazy` | definition not found via regex heuristic (kind='fn', name='Lazy') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::lazy::Lazy` | definition not found via regex heuristic (kind='fn', name='Lazy') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::lazy::Lazy` | definition not found via regex heuristic (kind='fn', name='Lazy') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::lazy::Lazy` | definition not found via regex heuristic (kind='fn', name='Lazy') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::lazy::Lazy` | dedup-alias: same definition as `mycelium_std_iter::Lazy` at crates/mycelium-std-iter/src/lazy.rs:30 — one canonical row kept |
| `mycelium_std_iter::transducer::Transducer` | definition not found via regex heuristic (kind='fn', name='Transducer') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::transducer::Transducer` | definition not found via regex heuristic (kind='fn', name='Transducer') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::transducer::Transducer` | definition not found via regex heuristic (kind='fn', name='Transducer') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::transducer::Transducer` | definition not found via regex heuristic (kind='fn', name='Transducer') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::transducer::Transducer` | definition not found via regex heuristic (kind='fn', name='Transducer') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::transducer::Transducer` | definition not found via regex heuristic (kind='fn', name='Transducer') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::transducer::Transducer` | definition not found via regex heuristic (kind='fn', name='Transducer') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::transducer::Transducer` | definition not found via regex heuristic (kind='fn', name='Transducer') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::transducer::Transducer` | definition not found via regex heuristic (kind='fn', name='Transducer') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::transducer::Transducer` | definition not found via regex heuristic (kind='fn', name='Transducer') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::transducer::Transducer` | dedup-alias: same definition as `mycelium_std_iter::Transducer` at crates/mycelium-std-iter/src/transducer.rs:29 — one canonical row kept |
| `mycelium_std_iter::zip_outcome::ZipOutcome` | dedup-alias: same definition as `mycelium_std_iter::ZipOutcome` at crates/mycelium-std-iter/src/zip_outcome.rs:17 — one canonical row kept |
| `mycelium_std_iter::zip_outcome::ZipOutcome::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::zip_outcome::ZipOutcome::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::zip_outcome::ZipOutcome::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::zip_outcome::ZipOutcome::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::zip_outcome::ZipOutcome::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_iter::zip_outcome::ZipOutcome::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::MathErr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::MathErr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::MathErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::MathErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::Approx` | dedup-alias: same definition as `mycelium_std_math::Approx` at crates/mycelium-std-math/src/approx.rs:69 — one canonical row kept |
| `mycelium_std_math::approx::ApproxExplain` | dedup-alias: same definition as `mycelium_std_math::ApproxExplain` at crates/mycelium-std-math/src/approx.rs:85 — one canonical row kept |
| `mycelium_std_math::approx::ApproxExplain::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::ApproxExplain::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::ApproxExplain::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::ApproxExplain::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::ApproxExplain::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::ApproxExplain::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::approx::DECLARED_FLOAT_EPS` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_math::exact::RoundMode` | dedup-alias: same definition as `mycelium_std_math::RoundMode` at crates/mycelium-std-math/src/exact.rs:30 — one canonical row kept |
| `mycelium_std_math::exact::RoundMode::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::exact::RoundMode::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::exact::RoundMode::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::exact::RoundMode::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::exact::RoundMode::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::exact::RoundMode::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::matrix::GUARANTEE_MATRIX:` | dedup-alias: same definition as `mycelium_std_math::GUARANTEE_MATRIX:` at crates/mycelium-std-math/src/matrix.rs:49 — one canonical row kept |
| `mycelium_std_math::matrix::GuaranteeRow` | dedup-alias: same definition as `mycelium_std_math::GuaranteeRow` at crates/mycelium-std-math/src/matrix.rs:27 — one canonical row kept |
| `mycelium_std_math::matrix::GuaranteeRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::matrix::GuaranteeRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::matrix::GuaranteeRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::matrix::GuaranteeRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::matrix::GuaranteeRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::matrix::GuaranteeRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_math::matrix::assert_matrix_invariants` | dedup-alias: same definition as `mycelium_std_math::assert_matrix_invariants` at crates/mycelium-std-math/src/matrix.rs:283 — one canonical row kept |
| `mycelium_std_numerics::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::Approx` | definition not found via regex heuristic (kind='fn', name='Approx') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::CheckErr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::CheckErr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::CheckErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::CheckErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::ErrorBound` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_numerics::ErrorOp` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_numerics::Explanation::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::Explanation::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::Explanation::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::KernelProbBound` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_numerics::NumErr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::NumErr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::NumErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::NumErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::ProvenThm::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::ProvenThm::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::ProvenThm::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::matrix::GuaranteeRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::matrix::GuaranteeRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_numerics::matrix::GuaranteeRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::EntropyEffect::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::EntropyEffect::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::EntropyEffect::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::EntropyRng` | definition not found via regex heuristic (kind='fn', name='EntropyRng') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::EntropyRng` | definition not found via regex heuristic (kind='fn', name='EntropyRng') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::EntropySource::fill_bytes` | definition not found via regex heuristic (kind='fn', name='fill_bytes') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::MatrixRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::MatrixRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::MatrixRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::RandErr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::RandErr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::RandErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::RandErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::Rng::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::Rng::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::Rng::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::RngAlgo::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::RngAlgo::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::RngAlgo::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::StubEntropy::fill_bytes` | definition not found via regex heuristic (kind='fn', name='fill_bytes') — possibly macro-generated or cfg-gated |
| `mycelium_std_rand::StubEntropy::fill_bytes` | definition not found via regex heuristic (kind='fn', name='fill_bytes') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::Budgets` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_recover::EffectBudget` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_recover::EffectBudgetExhausted` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_recover::EffectKind` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_recover::action::RecoveryAction` | definition not found via regex heuristic (kind='fn', name='RecoveryAction') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::action::RecoveryAction` | definition not found via regex heuristic (kind='fn', name='RecoveryAction') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::action::RecoveryAction` | definition not found via regex heuristic (kind='fn', name='RecoveryAction') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::action::RecoveryAction` | definition not found via regex heuristic (kind='fn', name='RecoveryAction') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::action::RecoveryAction` | definition not found via regex heuristic (kind='fn', name='RecoveryAction') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::action::RecoveryAction` | definition not found via regex heuristic (kind='fn', name='RecoveryAction') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::action::RecoveryAction` | dedup-alias: same definition as `mycelium_std_recover::RecoveryAction` at crates/mycelium-std-recover/src/action.rs:33 — one canonical row kept |
| `mycelium_std_recover::effect::Budgets` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_recover::effect::EffectBudget` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_recover::effect::EffectBudgetExhausted` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_recover::effect::EffectKind` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_recover::effect::EffectSet` | dedup-alias: same definition as `mycelium_std_recover::EffectSet` at crates/mycelium-std-recover/src/effect.rs:24 — one canonical row kept |
| `mycelium_std_recover::effect::UndeclaredEffect` | dedup-alias: same definition as `mycelium_std_recover::UndeclaredEffect` at crates/mycelium-std-recover/src/effect.rs:32 — one canonical row kept |
| `mycelium_std_recover::effect::UndeclaredEffect::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::effect::UndeclaredEffect::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::effect::UndeclaredEffect::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::effect::UndeclaredEffect::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::effect::UndeclaredEffect::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::effect::UndeclaredEffect::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::effect::UndeclaredEffect::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::effect::UndeclaredEffect::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::effect::check_effects` | dedup-alias: same definition as `mycelium_std_recover::check_effects` at crates/mycelium-std-recover/src/effect.rs:62 — one canonical row kept |
| `mycelium_std_recover::guarantee_matrix::Explainable::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::guarantee_matrix::Explainable::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::guarantee_matrix::Explainable::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::guarantee_matrix::Fallibility::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::guarantee_matrix::Fallibility::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::guarantee_matrix::Fallibility::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::guarantee_matrix::MatrixRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::guarantee_matrix::MatrixRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::guarantee_matrix::MatrixRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::handle::handle_classified` | dedup-alias: same definition as `mycelium_std_recover::handle_classified` at crates/mycelium-std-recover/src/handle.rs:75 — one canonical row kept |
| `mycelium_std_recover::handle::recover_classified` | dedup-alias: same definition as `mycelium_std_recover::recover_classified` at crates/mycelium-std-recover/src/handle.rs:198 — one canonical row kept |
| `mycelium_std_recover::outcome::DiagError` | definition not found via regex heuristic (kind='fn', name='DiagError') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::DiagError` | definition not found via regex heuristic (kind='fn', name='DiagError') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::DiagError` | definition not found via regex heuristic (kind='fn', name='DiagError') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::DiagError` | definition not found via regex heuristic (kind='fn', name='DiagError') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::DiagError` | definition not found via regex heuristic (kind='fn', name='DiagError') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::DiagError` | definition not found via regex heuristic (kind='fn', name='DiagError') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::DiagError` | definition not found via regex heuristic (kind='fn', name='DiagError') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::DiagError` | definition not found via regex heuristic (kind='fn', name='DiagError') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::DiagError` | dedup-alias: same definition as `mycelium_std_recover::DiagError` at crates/mycelium-std-recover/src/outcome.rs:63 — one canonical row kept |
| `mycelium_std_recover::outcome::Outcome` | definition not found via regex heuristic (kind='fn', name='Outcome') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Outcome` | definition not found via regex heuristic (kind='fn', name='Outcome') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Outcome` | definition not found via regex heuristic (kind='fn', name='Outcome') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Outcome` | definition not found via regex heuristic (kind='fn', name='Outcome') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Outcome` | definition not found via regex heuristic (kind='fn', name='Outcome') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Outcome` | definition not found via regex heuristic (kind='fn', name='Outcome') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Outcome` | definition not found via regex heuristic (kind='fn', name='Outcome') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Outcome` | definition not found via regex heuristic (kind='fn', name='Outcome') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Outcome` | definition not found via regex heuristic (kind='fn', name='Outcome') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Outcome` | definition not found via regex heuristic (kind='fn', name='Outcome') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Outcome` | definition not found via regex heuristic (kind='fn', name='Outcome') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Outcome` | definition not found via regex heuristic (kind='fn', name='Outcome') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Outcome` | definition not found via regex heuristic (kind='fn', name='Outcome') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Outcome` | definition not found via regex heuristic (kind='fn', name='Outcome') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Outcome` | dedup-alias: same definition as `mycelium_std_recover::Outcome` at crates/mycelium-std-recover/src/outcome.rs:21 — one canonical row kept |
| `mycelium_std_recover::outcome::Resolution` | definition not found via regex heuristic (kind='fn', name='Resolution') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Resolution` | definition not found via regex heuristic (kind='fn', name='Resolution') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Resolution` | definition not found via regex heuristic (kind='fn', name='Resolution') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Resolution` | definition not found via regex heuristic (kind='fn', name='Resolution') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Resolution` | definition not found via regex heuristic (kind='fn', name='Resolution') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Resolution` | definition not found via regex heuristic (kind='fn', name='Resolution') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Resolution` | definition not found via regex heuristic (kind='fn', name='Resolution') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Resolution` | definition not found via regex heuristic (kind='fn', name='Resolution') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Resolution` | definition not found via regex heuristic (kind='fn', name='Resolution') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Resolution` | definition not found via regex heuristic (kind='fn', name='Resolution') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Resolution` | definition not found via regex heuristic (kind='fn', name='Resolution') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Resolution` | definition not found via regex heuristic (kind='fn', name='Resolution') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::outcome::Resolution` | dedup-alias: same definition as `mycelium_std_recover::Resolution` at crates/mycelium-std-recover/src/outcome.rs:90 — one canonical row kept |
| `mycelium_std_recover::policy::PolicyHashError` | dedup-alias: same definition as `mycelium_std_recover::PolicyHashError` at crates/mycelium-std-recover/src/policy.rs:50 — one canonical row kept |
| `mycelium_std_recover::policy::PolicyHashError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::PolicyHashError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::PolicyHashError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::PolicyHashError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::PolicyHashError::source` | definition not found via regex heuristic (kind='fn', name='source') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::PolicyHashError::source` | definition not found via regex heuristic (kind='fn', name='source') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::PolicyRef` | dedup-alias: same definition as `mycelium_std_recover::PolicyRef` at crates/mycelium-std-recover/src/policy.rs:35 — one canonical row kept |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | definition not found via regex heuristic (kind='fn', name='RecoveryPolicy') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::policy::RecoveryPolicy` | dedup-alias: same definition as `mycelium_std_recover::RecoveryPolicy` at crates/mycelium-std-recover/src/policy.rs:86 — one canonical row kept |
| `mycelium_std_recover::policy::policy_effects` | dedup-alias: same definition as `mycelium_std_recover::policy_effects` at crates/mycelium-std-recover/src/policy.rs:242 — one canonical row kept |
| `mycelium_std_recover::registry::ClassName` | dedup-alias: same definition as `mycelium_std_recover::ClassName` at crates/mycelium-std-recover/src/registry.rs:19 — one canonical row kept |
| `mycelium_std_recover::registry::ClassName::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassName::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassName::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassName::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassName::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassName::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassName::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassName::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassName::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassName::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassName::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassName::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassName::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassName::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassRegistry` | dedup-alias: same definition as `mycelium_std_recover::ClassRegistry` at crates/mycelium-std-recover/src/registry.rs:69 — one canonical row kept |
| `mycelium_std_recover::registry::ClassRegistry::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassRegistry::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassRegistry::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassRegistry::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassRegistry::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::ClassRegistry::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::UnknownClass` | dedup-alias: same definition as `mycelium_std_recover::UnknownClass` at crates/mycelium-std-recover/src/registry.rs:39 — one canonical row kept |
| `mycelium_std_recover::registry::UnknownClass::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::UnknownClass::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::UnknownClass::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::UnknownClass::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::UnknownClass::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::UnknownClass::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::UnknownClass::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_recover::registry::UnknownClass::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::colony::Colony` | definition not found via regex heuristic (kind='fn', name='Colony') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::colony::Colony` | definition not found via regex heuristic (kind='fn', name='Colony') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::colony::Colony` | definition not found via regex heuristic (kind='fn', name='Colony') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::colony::Colony` | definition not found via regex heuristic (kind='fn', name='Colony') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::colony::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::colony::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::colony::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::colony::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::colony::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::colony::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::colony::Scope` | definition not found via regex heuristic (kind='fn', name='Scope') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::colony::ScopeError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::colony::ScopeError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::guarantee_matrix::GaugeRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::guarantee_matrix::GaugeRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::guarantee_matrix::GaugeRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::network::ChannelError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::network::ChannelError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::network::Network::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::network::Network::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::network::Receiver` | definition not found via regex heuristic (kind='fn', name='Receiver') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::network::Receiver` | definition not found via regex heuristic (kind='fn', name='Receiver') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::network::Sender` | definition not found via regex heuristic (kind='fn', name='Sender') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::network::Sender` | definition not found via regex heuristic (kind='fn', name='Sender') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::network::Sender` | definition not found via regex heuristic (kind='fn', name='Sender') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::network::TryRecv` | definition not found via regex heuristic (kind='fn', name='TryRecv') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::network::TryRecv` | definition not found via regex heuristic (kind='fn', name='TryRecv') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::network::TrySend` | definition not found via regex heuristic (kind='fn', name='TrySend') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::network::TrySend` | definition not found via regex heuristic (kind='fn', name='TrySend') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::task::Deadlock::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::task::Deadlock::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::task::Deadlock::new` | dedup-alias: same definition as `mycelium_std_runtime::task::Task::new` at crates/mycelium-std-runtime/src/task.rs:27 — one canonical row kept |
| `mycelium_std_runtime::task::Poll` | definition not found via regex heuristic (kind='fn', name='Poll') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::task::Poll` | definition not found via regex heuristic (kind='fn', name='Poll') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::task::SweepOrder::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::task::SweepOrder::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::task::SweepOrder::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::task::SweepOrder::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::task::Task::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::task::TaskCtx::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::task::TaskCtx::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_runtime::task::TaskCtx::new` | dedup-alias: same definition as `mycelium_std_runtime::task::Task::new` at crates/mycelium-std-runtime/src/task.rs:27 — one canonical row kept |
| `mycelium_std_select::Action` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::BITNET_PACKINGS` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::Candidate` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::CandidateCost` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::ContentHash` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::CostModel` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::DecodeFacts` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::DecodeMethod` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::ExplainAble::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_select::ExplainAble::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_select::ExplainAble::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_select::Explanation` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::GuaranteeRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_select::GuaranteeRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_select::GuaranteeRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_select::GuaranteeTag::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_select::GuaranteeTag::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_select::GuaranteeTag::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_select::Meta` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::PackScheme` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::ParadigmKind` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::PhysicalLayout` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::PolicyError` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::PolicyRegistry` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::Predicate` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::Provenance` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::Repr` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::Rule` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::SelectError` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::SelectionInputs` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::SelectionPolicy` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::Value` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::bitnet_packing_policy` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::layout_of` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::record_packing_layout` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::select_decode_method` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::select_layout` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::select_packing` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_select::select_swap_target` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_spore::ContentHash` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_spore::GuaranteeStrength` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_spore::RawSpore` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_spore::ReconMode` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_spore::SporeError` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_spore::deploy::DeployError` | dedup-alias: same definition as `mycelium_std_spore::DeployError` at crates/mycelium-std-spore/src/deploy.rs:137 — one canonical row kept |
| `mycelium_std_spore::deploy::DeployError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployResult` | dedup-alias: same definition as `mycelium_std_spore::DeployResult` at crates/mycelium-std-spore/src/deploy.rs:102 — one canonical row kept |
| `mycelium_std_spore::deploy::DeployResult::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployResult::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployResult::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployResult::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployResult::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployResult::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployTarget` | dedup-alias: same definition as `mycelium_std_spore::DeployTarget` at crates/mycelium-std-spore/src/deploy.rs:48 — one canonical row kept |
| `mycelium_std_spore::deploy::DeployTarget::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployTarget::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployTarget::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployTarget::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployTarget::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployTarget::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployVerification` | dedup-alias: same definition as `mycelium_std_spore::DeployVerification` at crates/mycelium-std-spore/src/deploy.rs:76 — one canonical row kept |
| `mycelium_std_spore::deploy::DeployVerification::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployVerification::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployVerification::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployVerification::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployVerification::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::DeployVerification::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::deploy::explain_deploy` | dedup-alias: same definition as `mycelium_std_spore::explain_deploy` at crates/mycelium-std-spore/src/deploy.rs:339 — one canonical row kept |
| `mycelium_std_spore::deploy::germinate` | dedup-alias: same definition as `mycelium_std_spore::germinate` at crates/mycelium-std-spore/src/deploy.rs:235 — one canonical row kept |
| `mycelium_std_spore::guarantee_matrix::MATRIX:` | dedup-alias: same definition as `mycelium_std_spore::MATRIX:` at crates/mycelium-std-spore/src/guarantee_matrix.rs:69 — one canonical row kept |
| `mycelium_std_spore::guarantee_matrix::MatrixRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::guarantee_matrix::MatrixRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::guarantee_matrix::MatrixRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::MalformedManifest` | dedup-alias: same definition as `mycelium_std_spore::MalformedManifest` at crates/mycelium-std-spore/src/recon_manifest.rs:190 — one canonical row kept |
| `mycelium_std_spore::recon_manifest::MalformedManifest::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::MalformedManifest::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::MalformedManifest::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::MalformedManifest::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::MalformedManifest::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::MalformedManifest::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::MalformedManifest::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::MalformedManifest::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::ReconManifest` | dedup-alias: same definition as `mycelium_std_spore::ReconManifest` at crates/mycelium-std-spore/src/recon_manifest.rs:37 — one canonical row kept |
| `mycelium_std_spore::recon_manifest::ReconManifest::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::ReconManifest::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::ReconManifest::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::ReconManifest::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::ReconManifest::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::ReconManifest::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::ReconManifest::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::ReconManifest::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::ReconMode` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_spore::recon_manifest::RegrowthResult` | dedup-alias: same definition as `mycelium_std_spore::RegrowthResult` at crates/mycelium-std-spore/src/recon_manifest.rs:233 — one canonical row kept |
| `mycelium_std_spore::recon_manifest::RegrowthResult::delta` | dedup-alias: same definition as `mycelium_std_spore::recon_manifest::ReconManifest::delta` at crates/mycelium-std-spore/src/recon_manifest.rs:164 — one canonical row kept |
| `mycelium_std_spore::recon_manifest::RegrowthResult::delta` | dedup-alias: same definition as `mycelium_std_spore::recon_manifest::ReconManifest::delta` at crates/mycelium-std-spore/src/recon_manifest.rs:164 — one canonical row kept |
| `mycelium_std_spore::recon_manifest::RegrowthResult::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::RegrowthResult::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::recon_manifest::RegrowthResult::new` | dedup-alias: same definition as `mycelium_std_spore::recon_manifest::ReconManifest::new` at crates/mycelium-std-spore/src/recon_manifest.rs:63 — one canonical row kept |
| `mycelium_std_spore::recon_manifest::RegrowthResult::new` | dedup-alias: same definition as `mycelium_std_spore::recon_manifest::ReconManifest::new` at crates/mycelium-std-spore/src/recon_manifest.rs:63 — one canonical row kept |
| `mycelium_std_spore::spore_ops::SporeErr` | dedup-alias: same definition as `mycelium_std_spore::SporeErr` at crates/mycelium-std-spore/src/spore_ops.rs:29 — one canonical row kept |
| `mycelium_std_spore::spore_ops::SporeErr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::spore_ops::SporeErr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::spore_ops::SporeErr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::spore_ops::SporeErr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::spore_ops::SporeErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::spore_ops::SporeErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::spore_ops::SporeErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::spore_ops::SporeErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::spore_ops::SporeErr::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::spore_ops::SporeErr::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::spore_ops::SporeUnit` | dedup-alias: same definition as `mycelium_std_spore::SporeUnit` at crates/mycelium-std-spore/src/spore_ops.rs:96 — one canonical row kept |
| `mycelium_std_spore::spore_ops::SporeUnit::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::spore_ops::SporeUnit::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::spore_ops::SporeUnit::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::spore_ops::SporeUnit::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::spore_ops::SporeUnit::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::spore_ops::SporeUnit::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_spore::spore_ops::SporeUnit::identity` | dedup-alias: same definition as `mycelium_std_spore::identity` at crates/mycelium-std-spore/src/spore_ops.rs:227 — one canonical row kept |
| `mycelium_std_spore::spore_ops::SporeUnit::identity` | dedup-alias: same definition as `mycelium_std_spore::identity` at crates/mycelium-std-spore/src/spore_ops.rs:227 — one canonical row kept |
| `mycelium_std_spore::spore_ops::SporeUnit::verify` | dedup-alias: same definition as `mycelium_std_spore::verify` at crates/mycelium-std-spore/src/spore_ops.rs:208 — one canonical row kept |
| `mycelium_std_spore::spore_ops::SporeUnit::verify` | dedup-alias: same definition as `mycelium_std_spore::verify` at crates/mycelium-std-spore/src/spore_ops.rs:208 — one canonical row kept |
| `mycelium_std_spore::spore_ops::explain_spore` | dedup-alias: same definition as `mycelium_std_spore::explain_spore` at crates/mycelium-std-spore/src/spore_ops.rs:344 — one canonical row kept |
| `mycelium_std_spore::spore_ops::identity` | dedup-alias: same definition as `mycelium_std_spore::identity` at crates/mycelium-std-spore/src/spore_ops.rs:227 — one canonical row kept |
| `mycelium_std_spore::spore_ops::manifest_of` | dedup-alias: same definition as `mycelium_std_spore::manifest_of` at crates/mycelium-std-spore/src/spore_ops.rs:321 — one canonical row kept |
| `mycelium_std_spore::spore_ops::verify` | dedup-alias: same definition as `mycelium_std_spore::verify` at crates/mycelium-std-spore/src/spore_ops.rs:208 — one canonical row kept |
| `mycelium_std_swap::BF16_MIN_NORMAL` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::BF16_REL_EPS` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::Bound` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::BoundBasis` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::BoundKind` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::CheckError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_swap::CheckError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_swap::CheckError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_swap::CheckError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_swap::CheckVerdict` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::ContentHash` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::DENSE_VSA_EMP_DELTA` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::DENSE_VSA_MODEL` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::Evidence` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::ExplainRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_swap::ExplainRecord::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_swap::ExplainRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_swap::Fallback` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::GuaranteeStrength` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::MatrixRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_swap::MatrixRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_swap::MatrixRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_swap::NormKind` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::NotValidatedReason` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::RefinementRelation` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::Repr` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::SwapCertificate` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::SwapError` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::Swapped::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_swap::Swapped::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_swap::Swapped::explain` | dedup-alias: same definition as `mycelium_std_swap::explain` at crates/mycelium-std-swap/src/lib.rs:88 — one canonical row kept |
| `mycelium_std_swap::Swapped::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_swap::Value` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::check` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::check` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::legal_pair` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_swap::roundtrip_lemma_ref` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_sys::rand::EntropyError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_sys::rand::EntropyError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_sys::rand::EntropyError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_sys::rand::EntropyError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::arithmetic::add` | dedup-alias: same definition as `mycelium_std_ternary::add` at crates/mycelium-std-ternary/src/arithmetic.rs:90 — one canonical row kept |
| `mycelium_std_ternary::arithmetic::int_to_trits` | dedup-alias: same definition as `mycelium_std_ternary::int_to_trits` at crates/mycelium-std-ternary/src/arithmetic.rs:54 — one canonical row kept |
| `mycelium_std_ternary::arithmetic::max_magnitude` | dedup-alias: same definition as `mycelium_std_ternary::max_magnitude` at crates/mycelium-std-ternary/src/arithmetic.rs:65 — one canonical row kept |
| `mycelium_std_ternary::arithmetic::mul` | dedup-alias: same definition as `mycelium_std_ternary::mul` at crates/mycelium-std-ternary/src/arithmetic.rs:112 — one canonical row kept |
| `mycelium_std_ternary::arithmetic::neg` | dedup-alias: same definition as `mycelium_std_ternary::neg` at crates/mycelium-std-ternary/src/arithmetic.rs:79 — one canonical row kept |
| `mycelium_std_ternary::arithmetic::sub` | dedup-alias: same definition as `mycelium_std_ternary::sub` at crates/mycelium-std-ternary/src/arithmetic.rs:98 — one canonical row kept |
| `mycelium_std_ternary::arithmetic::trits_to_int` | dedup-alias: same definition as `mycelium_std_ternary::trits_to_int` at crates/mycelium-std-ternary/src/arithmetic.rs:43 — one canonical row kept |
| `mycelium_std_ternary::guarantee_matrix::Explainable::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::guarantee_matrix::Explainable::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::guarantee_matrix::Explainable::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::guarantee_matrix::Fallibility::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::guarantee_matrix::Fallibility::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::guarantee_matrix::Fallibility::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::guarantee_matrix::OpGuarantee::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::guarantee_matrix::OpGuarantee::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::guarantee_matrix::OpGuarantee::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::guarantee_matrix::Tag::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::guarantee_matrix::Tag::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::guarantee_matrix::Tag::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::ExplainRecord` | dedup-alias: same definition as `mycelium_std_ternary::ExplainRecord` at crates/mycelium-std-ternary/src/packing.rs:119 — one canonical row kept |
| `mycelium_std_ternary::packing::ExplainRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::ExplainRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::ExplainRecord::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::ExplainRecord::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::ExplainRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::ExplainRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::ExplainRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::ExplainRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::PackError` | dedup-alias: same definition as `mycelium_std_ternary::PackError` at crates/mycelium-std-ternary/src/packing.rs:91 — one canonical row kept |
| `mycelium_std_ternary::packing::PackError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::PackError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::PackError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::PackError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::PackError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::PackError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::PackError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::PackError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::Packed` | dedup-alias: same definition as `mycelium_std_ternary::Packed` at crates/mycelium-std-ternary/src/packing.rs:163 — one canonical row kept |
| `mycelium_std_ternary::packing::Packed::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::Packed::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::Packed::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::Packed::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::Packed::explain` | dedup-alias: same definition as `mycelium_std_ternary::explain` at crates/mycelium-std-ternary/src/packing.rs:205 — one canonical row kept |
| `mycelium_std_ternary::packing::Packed::explain` | dedup-alias: same definition as `mycelium_std_ternary::explain` at crates/mycelium-std-ternary/src/packing.rs:205 — one canonical row kept |
| `mycelium_std_ternary::packing::Packed::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::Packed::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::Scheme` | dedup-alias: same definition as `mycelium_std_ternary::Scheme` at crates/mycelium-std-ternary/src/packing.rs:38 — one canonical row kept |
| `mycelium_std_ternary::packing::Scheme::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::Scheme::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::Scheme::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::Scheme::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::Scheme::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::Scheme::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::Scheme::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::Scheme::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::Scheme::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::Scheme::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::SelectionNote::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::SelectionNote::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::SelectionNote::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::packing::explain` | dedup-alias: same definition as `mycelium_std_ternary::explain` at crates/mycelium-std-ternary/src/packing.rs:205 — one canonical row kept |
| `mycelium_std_ternary::packing::pack` | dedup-alias: same definition as `mycelium_std_ternary::pack` at crates/mycelium-std-ternary/src/packing.rs:411 — one canonical row kept |
| `mycelium_std_ternary::packing::scheme_of` | dedup-alias: same definition as `mycelium_std_ternary::scheme_of` at crates/mycelium-std-ternary/src/packing.rs:221 — one canonical row kept |
| `mycelium_std_ternary::packing::unpack` | dedup-alias: same definition as `mycelium_std_ternary::unpack` at crates/mycelium-std-ternary/src/packing.rs:443 — one canonical row kept |
| `mycelium_std_ternary::primitives::Bit` | dedup-alias: same definition as `mycelium_std_ternary::Bit` at crates/mycelium-std-ternary/src/primitives.rs:110 — one canonical row kept |
| `mycelium_std_ternary::primitives::Bit::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Bit::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Bit::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Bit::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Bit::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Bit::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Bit::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Bit::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Bit::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Bit::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Trit` | dedup-alias: same definition as `mycelium_std_ternary::Trit` at crates/mycelium-std-ternary/src/primitives.rs:21 — one canonical row kept |
| `mycelium_std_ternary::primitives::Trit::Output` | definition not found via regex heuristic (kind='type', name='Output') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Trit::Output` | definition not found via regex heuristic (kind='type', name='Output') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Trit::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Trit::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Trit::digit` | dedup-alias: same definition as `mycelium_std_ternary::primitives::Bit::digit` at crates/mycelium-std-ternary/src/primitives.rs:50 — one canonical row kept |
| `mycelium_std_ternary::primitives::Trit::digit` | dedup-alias: same definition as `mycelium_std_ternary::primitives::Bit::digit` at crates/mycelium-std-ternary/src/primitives.rs:50 — one canonical row kept |
| `mycelium_std_ternary::primitives::Trit::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Trit::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Trit::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Trit::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Trit::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Trit::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Trit::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Trit::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_ternary::primitives::Trit::neg` | dedup-alias: same definition as `mycelium_std_ternary::neg` at crates/mycelium-std-ternary/src/arithmetic.rs:79 — one canonical row kept |
| `mycelium_std_ternary::primitives::Trit::neg` | dedup-alias: same definition as `mycelium_std_ternary::neg` at crates/mycelium-std-ternary/src/arithmetic.rs:79 — one canonical row kept |
| `mycelium_std_ternary::primitives::Trit::new` | dedup-alias: same definition as `mycelium_std_ternary::primitives::Bit::new` at crates/mycelium-std-ternary/src/primitives.rs:37 — one canonical row kept |
| `mycelium_std_ternary::primitives::Trit::new` | dedup-alias: same definition as `mycelium_std_ternary::primitives::Bit::new` at crates/mycelium-std-ternary/src/primitives.rs:37 — one canonical row kept |
| `mycelium_std_testing::Budget::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::Budget::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::Budget::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::Budget::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::Budget::new` | dedup-alias: same definition as `mycelium_std_testing::Rng::new` at crates/mycelium-std-testing/src/lib.rs:93 — one canonical row kept |
| `mycelium_std_testing::Budget::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::Gen::generate` | definition not found via regex heuristic (kind='fn', name='generate') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::Gen::shrink` | definition not found via regex heuristic (kind='fn', name='shrink') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::GoldenBaseline::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::GoldenBaseline::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::GoldenBaseline::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::GoldenBaseline::new` | dedup-alias: same definition as `mycelium_std_testing::Rng::new` at crates/mycelium-std-testing/src/lib.rs:93 — one canonical row kept |
| `mycelium_std_testing::Rng::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::Rng::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::guarantee_matrix::Row::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::guarantee_matrix::Row::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::guarantee_matrix::Row::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::FailRecord` | dedup-alias: same definition as `mycelium_std_testing::FailRecord` at crates/mycelium-std-testing/src/verdict.rs:30 — one canonical row kept |
| `mycelium_std_testing::verdict::FailRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::FailRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::FailRecord::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::FailRecord::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::FailRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::FailRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::SkipReason` | dedup-alias: same definition as `mycelium_std_testing::SkipReason` at crates/mycelium-std-testing/src/verdict.rs:67 — one canonical row kept |
| `mycelium_std_testing::verdict::SkipReason::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::SkipReason::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::SkipReason::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::SkipReason::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::SkipReason::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::SkipReason::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::Summary` | dedup-alias: same definition as `mycelium_std_testing::Summary` at crates/mycelium-std-testing/src/verdict.rs:157 — one canonical row kept |
| `mycelium_std_testing::verdict::Summary::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::Summary::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::Summary::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::Summary::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::Summary::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::Summary::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::Summary::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::Summary::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::UndetReason` | dedup-alias: same definition as `mycelium_std_testing::UndetReason` at crates/mycelium-std-testing/src/verdict.rs:94 — one canonical row kept |
| `mycelium_std_testing::verdict::UndetReason::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::UndetReason::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::UndetReason::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::UndetReason::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::UndetReason::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::UndetReason::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::Verdict` | dedup-alias: same definition as `mycelium_std_testing::Verdict` at crates/mycelium-std-testing/src/verdict.rs:120 — one canonical row kept |
| `mycelium_std_testing::verdict::Verdict::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::Verdict::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::Verdict::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::Verdict::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::Verdict::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_testing::verdict::Verdict::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::BoundaryError` | dedup-alias: same definition as `mycelium_std_text::BoundaryError` at crates/mycelium-std-text/src/error.rs:82 — one canonical row kept |
| `mycelium_std_text::error::BoundaryError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::BoundaryError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::BoundaryError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::BoundaryError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::BoundaryError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::BoundaryError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::BoundaryError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::BoundaryError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::EncodeError` | dedup-alias: same definition as `mycelium_std_text::EncodeError` at crates/mycelium-std-text/src/error.rs:222 — one canonical row kept |
| `mycelium_std_text::error::EncodeError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::EncodeError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::EncodeError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::EncodeError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::EncodeError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::EncodeError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::EncodeError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::EncodeError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::ParseErr` | dedup-alias: same definition as `mycelium_std_text::ParseErr` at crates/mycelium-std-text/src/error.rs:160 — one canonical row kept |
| `mycelium_std_text::error::ParseErr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::ParseErr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::ParseErr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::ParseErr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::ParseErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::ParseErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::ParseErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::ParseErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::TranscodeError` | dedup-alias: same definition as `mycelium_std_text::TranscodeError` at crates/mycelium-std-text/src/error.rs:266 — one canonical row kept |
| `mycelium_std_text::error::TranscodeError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::TranscodeError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::TranscodeError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::TranscodeError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::TranscodeError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::TranscodeError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::TranscodeError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::TranscodeError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::Utf8Error` | dedup-alias: same definition as `mycelium_std_text::Utf8Error` at crates/mycelium-std-text/src/error.rs:43 — one canonical row kept |
| `mycelium_std_text::error::Utf8Error::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::Utf8Error::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::Utf8Error::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::Utf8Error::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::Utf8Error::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::Utf8Error::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::Utf8Error::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::error::Utf8Error::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::guarantee_matrix::MatrixRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::guarantee_matrix::MatrixRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::guarantee_matrix::MatrixRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::len_bytes` | ambiguous: short name 'len_bytes' is defined in multiple modules; attributed to crates/mycelium-std-text/src/ops.rs by heuristic — verify against source (ground truth) |
| `mycelium_std_text::ops::char_at` | dedup-alias: same definition as `mycelium_std_text::char_at` at crates/mycelium-std-text/src/ops.rs:277 — one canonical row kept |
| `mycelium_std_text::ops::chars` | dedup-alias: same definition as `mycelium_std_text::chars` at crates/mycelium-std-text/src/ops.rs:218 — one canonical row kept |
| `mycelium_std_text::ops::concat` | dedup-alias: same definition as `mycelium_std_text::concat` at crates/mycelium-std-text/src/ops.rs:79 — one canonical row kept |
| `mycelium_std_text::ops::encode_utf8` | dedup-alias: same definition as `mycelium_std_text::encode_utf8` at crates/mycelium-std-text/src/ops.rs:392 — one canonical row kept |
| `mycelium_std_text::ops::from_chars` | dedup-alias: same definition as `mycelium_std_text::from_chars` at crates/mycelium-std-text/src/ops.rs:43 — one canonical row kept |
| `mycelium_std_text::ops::from_utf16` | dedup-alias: same definition as `mycelium_std_text::from_utf16` at crates/mycelium-std-text/src/ops.rs:497 — one canonical row kept |
| `mycelium_std_text::ops::from_utf8` | dedup-alias: same definition as `mycelium_std_text::from_utf8` at crates/mycelium-std-text/src/ops.rs:57 — one canonical row kept |
| `mycelium_std_text::ops::join` | dedup-alias: same definition as `mycelium_std_text::join` at crates/mycelium-std-text/src/ops.rs:94 — one canonical row kept |
| `mycelium_std_text::ops::len_bytes` | dedup-alias: same definition as `mycelium_std_text::len_bytes` at crates/mycelium-std-text/src/ops.rs:159 — one canonical row kept |
| `mycelium_std_text::ops::len_chars` | dedup-alias: same definition as `mycelium_std_text::len_chars` at crates/mycelium-std-text/src/ops.rs:171 — one canonical row kept |
| `mycelium_std_text::ops::len_graphemes` | dedup-alias: same definition as `mycelium_std_text::len_graphemes` at crates/mycelium-std-text/src/ops.rs:193 — one canonical row kept |
| `mycelium_std_text::ops::parse_bool` | dedup-alias: same definition as `mycelium_std_text::parse_bool` at crates/mycelium-std-text/src/ops.rs:367 — one canonical row kept |
| `mycelium_std_text::ops::parse_int` | dedup-alias: same definition as `mycelium_std_text::parse_int` at crates/mycelium-std-text/src/ops.rs:310 — one canonical row kept |
| `mycelium_std_text::ops::replace` | dedup-alias: same definition as `mycelium_std_text::replace` at crates/mycelium-std-text/src/ops.rs:145 — one canonical row kept |
| `mycelium_std_text::ops::slice` | dedup-alias: same definition as `mycelium_std_text::slice` at crates/mycelium-std-text/src/ops.rs:242 — one canonical row kept |
| `mycelium_std_text::ops::to_latin1` | dedup-alias: same definition as `mycelium_std_text::to_latin1` at crates/mycelium-std-text/src/ops.rs:423 — one canonical row kept |
| `mycelium_std_text::ops::to_latin1_lossy` | dedup-alias: same definition as `mycelium_std_text::to_latin1_lossy` at crates/mycelium-std-text/src/ops.rs:456 — one canonical row kept |
| `mycelium_std_text::ops::to_lower` | dedup-alias: same definition as `mycelium_std_text::to_lower` at crates/mycelium-std-text/src/ops.rs:109 — one canonical row kept |
| `mycelium_std_text::ops::to_upper` | dedup-alias: same definition as `mycelium_std_text::to_upper` at crates/mycelium-std-text/src/ops.rs:121 — one canonical row kept |
| `mycelium_std_text::ops::to_utf16` | dedup-alias: same definition as `mycelium_std_text::to_utf16` at crates/mycelium-std-text/src/ops.rs:405 — one canonical row kept |
| `mycelium_std_text::ops::trim` | dedup-alias: same definition as `mycelium_std_text::trim` at crates/mycelium-std-text/src/ops.rs:133 — one canonical row kept |
| `mycelium_std_text::types::Lossy` | definition not found via regex heuristic (kind='fn', name='Lossy') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Lossy` | definition not found via regex heuristic (kind='fn', name='Lossy') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Lossy` | definition not found via regex heuristic (kind='fn', name='Lossy') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Lossy` | definition not found via regex heuristic (kind='fn', name='Lossy') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Lossy` | definition not found via regex heuristic (kind='fn', name='Lossy') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Lossy` | definition not found via regex heuristic (kind='fn', name='Lossy') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Lossy` | definition not found via regex heuristic (kind='fn', name='Lossy') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Lossy` | definition not found via regex heuristic (kind='fn', name='Lossy') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Lossy` | definition not found via regex heuristic (kind='fn', name='Lossy') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Lossy` | definition not found via regex heuristic (kind='fn', name='Lossy') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Lossy` | definition not found via regex heuristic (kind='fn', name='Lossy') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Lossy` | definition not found via regex heuristic (kind='fn', name='Lossy') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Lossy` | dedup-alias: same definition as `mycelium_std_text::Lossy` at crates/mycelium-std-text/src/types.rs:164 — one canonical row kept |
| `mycelium_std_text::types::Text` | dedup-alias: same definition as `mycelium_std_text::Text` at crates/mycelium-std-text/src/types.rs:37 — one canonical row kept |
| `mycelium_std_text::types::Text::Err` | definition not found via regex heuristic (kind='type', name='Err') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::Err` | definition not found via regex heuristic (kind='type', name='Err') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::as_ref` | definition not found via regex heuristic (kind='fn', name='as_ref') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::as_ref` | definition not found via regex heuristic (kind='fn', name='as_ref') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::from` | definition not found via regex heuristic (kind='fn', name='from') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::from_str` | definition not found via regex heuristic (kind='fn', name='from_str') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::from_str` | definition not found via regex heuristic (kind='fn', name='from_str') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_text::types::Text::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::ClockSource::logical_now` | dedup-alias: same definition as `mycelium_std_time::logical_now` at crates/mycelium-std-time/src/lib.rs:755 — one canonical row kept |
| `mycelium_std_time::ClockSource::mono_now` | dedup-alias: same definition as `mycelium_std_time::mono_now` at crates/mycelium-std-time/src/lib.rs:735 — one canonical row kept |
| `mycelium_std_time::ClockSource::wall_now` | dedup-alias: same definition as `mycelium_std_time::wall_now` at crates/mycelium-std-time/src/lib.rs:747 — one canonical row kept |
| `mycelium_std_time::DeclaredTime` | definition not found via regex heuristic (kind='fn', name='DeclaredTime') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::DeclaredTime` | definition not found via regex heuristic (kind='fn', name='DeclaredTime') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::DeclaredTime` | definition not found via regex heuristic (kind='fn', name='DeclaredTime') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::DeclaredTime` | definition not found via regex heuristic (kind='fn', name='DeclaredTime') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::DeclaredTime` | definition not found via regex heuristic (kind='fn', name='DeclaredTime') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::DeclaredTime` | definition not found via regex heuristic (kind='fn', name='DeclaredTime') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::DeclaredTimeEntropy` | definition not found via regex heuristic (kind='fn', name='DeclaredTimeEntropy') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::DeclaredTimeEntropy` | definition not found via regex heuristic (kind='fn', name='DeclaredTimeEntropy') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::DeclaredTimeEntropy` | definition not found via regex heuristic (kind='fn', name='DeclaredTimeEntropy') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::DeclaredTimeEntropy` | definition not found via regex heuristic (kind='fn', name='DeclaredTimeEntropy') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::DeclaredTimeEntropy` | definition not found via regex heuristic (kind='fn', name='DeclaredTimeEntropy') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::DeclaredTimeEntropy` | definition not found via regex heuristic (kind='fn', name='DeclaredTimeEntropy') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::Duration::as_nanos` | definition not found via regex heuristic (kind='fn', name='as_nanos') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::Duration::as_secs_trunc` | definition not found via regex heuristic (kind='fn', name='as_secs_trunc') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::Duration::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::Duration::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::Duration::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::Duration::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::Duration::from_nanos` | definition not found via regex heuristic (kind='fn', name='from_nanos') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::Duration::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::Duration::is_negative` | definition not found via regex heuristic (kind='fn', name='is_negative') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::Duration::is_zero` | definition not found via regex heuristic (kind='fn', name='is_zero') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::Duration::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::GuaranteeRow::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::GuaranteeRow::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::GuaranteeRow::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::LogicalInstant::as_tick` | definition not found via regex heuristic (kind='fn', name='as_tick') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::LogicalInstant::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::LogicalInstant::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::LogicalInstant::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::LogicalInstant::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::LogicalInstant::from_tick` | definition not found via regex heuristic (kind='fn', name='from_tick') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::LogicalInstant::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::LogicalInstant::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::ManualClock::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::ManualClock::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::ManualClock::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::ManualClock::logical_now` | dedup-alias: same definition as `mycelium_std_time::logical_now` at crates/mycelium-std-time/src/lib.rs:755 — one canonical row kept |
| `mycelium_std_time::ManualClock::logical_now` | dedup-alias: same definition as `mycelium_std_time::logical_now` at crates/mycelium-std-time/src/lib.rs:755 — one canonical row kept |
| `mycelium_std_time::ManualClock::mono_now` | dedup-alias: same definition as `mycelium_std_time::mono_now` at crates/mycelium-std-time/src/lib.rs:735 — one canonical row kept |
| `mycelium_std_time::ManualClock::mono_now` | dedup-alias: same definition as `mycelium_std_time::mono_now` at crates/mycelium-std-time/src/lib.rs:735 — one canonical row kept |
| `mycelium_std_time::ManualClock::wall_now` | dedup-alias: same definition as `mycelium_std_time::wall_now` at crates/mycelium-std-time/src/lib.rs:747 — one canonical row kept |
| `mycelium_std_time::ManualClock::wall_now` | dedup-alias: same definition as `mycelium_std_time::wall_now` at crates/mycelium-std-time/src/lib.rs:747 — one canonical row kept |
| `mycelium_std_time::MonoInstant::as_nanos` | definition not found via regex heuristic (kind='fn', name='as_nanos') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::MonoInstant::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::MonoInstant::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::MonoInstant::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::MonoInstant::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::MonoInstant::from_nanos` | definition not found via regex heuristic (kind='fn', name='from_nanos') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::MonoInstant::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::MonoInstant::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::SystemClock::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::SystemClock::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::SystemClock::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::SystemClock::logical_now` | dedup-alias: same definition as `mycelium_std_time::logical_now` at crates/mycelium-std-time/src/lib.rs:755 — one canonical row kept |
| `mycelium_std_time::SystemClock::logical_now` | dedup-alias: same definition as `mycelium_std_time::logical_now` at crates/mycelium-std-time/src/lib.rs:755 — one canonical row kept |
| `mycelium_std_time::SystemClock::mono_now` | dedup-alias: same definition as `mycelium_std_time::mono_now` at crates/mycelium-std-time/src/lib.rs:735 — one canonical row kept |
| `mycelium_std_time::SystemClock::mono_now` | dedup-alias: same definition as `mycelium_std_time::mono_now` at crates/mycelium-std-time/src/lib.rs:735 — one canonical row kept |
| `mycelium_std_time::SystemClock::wall_now` | dedup-alias: same definition as `mycelium_std_time::wall_now` at crates/mycelium-std-time/src/lib.rs:747 — one canonical row kept |
| `mycelium_std_time::SystemClock::wall_now` | dedup-alias: same definition as `mycelium_std_time::wall_now` at crates/mycelium-std-time/src/lib.rs:747 — one canonical row kept |
| `mycelium_std_time::TimeErr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::TimeErr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::TimeErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::TimeErr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::WallInstant::as_nanos_since_epoch` | definition not found via regex heuristic (kind='fn', name='as_nanos_since_epoch') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::WallInstant::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::WallInstant::cmp` | definition not found via regex heuristic (kind='fn', name='cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::WallInstant::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::WallInstant::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::WallInstant::from_nanos_since_epoch` | definition not found via regex heuristic (kind='fn', name='from_nanos_since_epoch') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::WallInstant::hash` | definition not found via regex heuristic (kind='fn', name='hash') — possibly macro-generated or cfg-gated |
| `mycelium_std_time::WallInstant::partial_cmp` | definition not found via regex heuristic (kind='fn', name='partial_cmp') — possibly macro-generated or cfg-gated |
| `mycelium_std_vsa::CleanupMemory` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_vsa::Factorization` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_vsa::GuaranteeStrength` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_vsa::Match` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_vsa::ResonatorTrace` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_vsa::VsaError` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_vsa::VsaModel` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_vsa::VsaOp` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_std_vsa::encoding::encode_seq` | dedup-alias: same definition as `mycelium_std_vsa::encode_seq` at crates/mycelium-std-vsa/src/encoding.rs:32 — one canonical row kept |
| `mycelium_std_vsa::encoding::encode_set` | dedup-alias: same definition as `mycelium_std_vsa::encode_set` at crates/mycelium-std-vsa/src/encoding.rs:58 — one canonical row kept |
| `mycelium_std_vsa::matrix::GUARANTEE_MATRIX:` | dedup-alias: same definition as `mycelium_std_vsa::GUARANTEE_MATRIX:` at crates/mycelium-std-vsa/src/matrix.rs:62 — one canonical row kept |
| `mycelium_std_vsa::matrix::OpGuarantee` | dedup-alias: same definition as `mycelium_std_vsa::OpGuarantee` at crates/mycelium-std-vsa/src/matrix.rs:24 — one canonical row kept |
| `mycelium_std_vsa::matrix::OpGuarantee::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_vsa::matrix::OpGuarantee::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_std_vsa::matrix::OpGuarantee::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_vsa::matrix::OpGuarantee::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_std_vsa::matrix::OpGuarantee::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_vsa::matrix::OpGuarantee::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_std_vsa::ops::bind` | dedup-alias: same definition as `mycelium_std_vsa::bind` at crates/mycelium-std-vsa/src/ops.rs:30 — one canonical row kept |
| `mycelium_std_vsa::ops::bind_role` | dedup-alias: same definition as `mycelium_std_vsa::bind_role` at crates/mycelium-std-vsa/src/ops.rs:113 — one canonical row kept |
| `mycelium_std_vsa::ops::bundle` | dedup-alias: same definition as `mycelium_std_vsa::bundle` at crates/mycelium-std-vsa/src/ops.rs:65 — one canonical row kept |
| `mycelium_std_vsa::ops::cleanup` | dedup-alias: same definition as `mycelium_std_vsa::cleanup` at crates/mycelium-std-vsa/src/ops.rs:145 — one canonical row kept |
| `mycelium_std_vsa::ops::permute` | dedup-alias: same definition as `mycelium_std_vsa::permute` at crates/mycelium-std-vsa/src/ops.rs:77 — one canonical row kept |
| `mycelium_std_vsa::ops::similarity` | dedup-alias: same definition as `mycelium_std_vsa::similarity` at crates/mycelium-std-vsa/src/ops.rs:101 — one canonical row kept |
| `mycelium_std_vsa::ops::unbind` | dedup-alias: same definition as `mycelium_std_vsa::unbind` at crates/mycelium-std-vsa/src/ops.rs:44 — one canonical row kept |
| `mycelium_std_vsa::ops::unpermute` | dedup-alias: same definition as `mycelium_std_vsa::unpermute` at crates/mycelium-std-vsa/src/ops.rs:88 — one canonical row kept |
| `mycelium_std_vsa::recon::reconstruct_factors` | dedup-alias: same definition as `mycelium_std_vsa::reconstruct_factors` at crates/mycelium-std-vsa/src/recon.rs:93 — one canonical row kept |
| `mycelium_std_vsa::recon::reconstruct_role` | dedup-alias: same definition as `mycelium_std_vsa::reconstruct_role` at crates/mycelium-std-vsa/src/recon.rs:50 — one canonical row kept |
| `mycelium_vsa::DecodeMethod` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_vsa::EmpiricalProfile::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::EmpiricalProfile::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::EmpiricalProfile::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::Explanation` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_vsa::VsaError::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::VsaError::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::VsaError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::VsaError::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::VsaModel::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::VsaModel::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::VsaModel::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::VsaModel::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::VsaModel::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::VsaModel::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::VsaModel::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::VsaModel::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::VsaModel::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::VsaOp::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::VsaOp::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::VsaOp::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc` | dedup-alias: same definition as `mycelium_vsa::Bsc` at crates/mycelium-vsa/src/bsc.rs:42 — one canonical row kept |
| `mycelium_vsa::bsc::Bsc::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::bsc::Bsc::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::cleanup::CleanupMemory` | dedup-alias: same definition as `mycelium_vsa::CleanupMemory` at crates/mycelium-vsa/src/cleanup.rs:28 — one canonical row kept |
| `mycelium_vsa::cleanup::CleanupMemory::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::cleanup::CleanupMemory::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::cleanup::CleanupMemory::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::cleanup::CleanupMemory::default` | definition not found via regex heuristic (kind='fn', name='default') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::cleanup::CleanupMemory::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::cleanup::CleanupMemory::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::cleanup::Match` | dedup-alias: same definition as `mycelium_vsa::Match` at crates/mycelium-vsa/src/cleanup.rs:14 — one canonical row kept |
| `mycelium_vsa::cleanup::Match::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::cleanup::Match::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::cleanup::Match::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::cleanup::Match::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::cleanup::Match::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::cleanup::Match::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::decode_select::DEFAULT_ENUM_BUDGET:` | dedup-alias: same definition as `mycelium_vsa::DEFAULT_ENUM_BUDGET:` at crates/mycelium-vsa/src/decode_select.rs:53 — one canonical row kept |
| `mycelium_vsa::decode_select::DecodeMethod` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_vsa::decode_select::DecodeSelection` | dedup-alias: same definition as `mycelium_vsa::DecodeSelection` at crates/mycelium-vsa/src/decode_select.rs:142 — one canonical row kept |
| `mycelium_vsa::decode_select::DecodeSelection::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::decode_select::DecodeSelection::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::decode_select::DecodeSelection::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::decode_select::DecodeSelection::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::decode_select::DecodeSelection::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::decode_select::DecodeSelection::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::decode_select::Explanation` | re-export (pub use) — cannot locate definition without type resolution |
| `mycelium_vsa::decode_select::decode_method_policy` | dedup-alias: same definition as `mycelium_vsa::decode_method_policy` at crates/mycelium-vsa/src/decode_select.rs:70 — one canonical row kept |
| `mycelium_vsa::decode_select::explain_decode_method` | dedup-alias: same definition as `mycelium_vsa::explain_decode_method` at crates/mycelium-vsa/src/decode_select.rs:127 — one canonical row kept |
| `mycelium_vsa::decode_select::reconstruct_factors_auto` | dedup-alias: same definition as `mycelium_vsa::reconstruct_factors_auto` at crates/mycelium-vsa/src/decode_select.rs:164 — one canonical row kept |
| `mycelium_vsa::fhrr::Fhrr` | dedup-alias: same definition as `mycelium_vsa::Fhrr` at crates/mycelium-vsa/src/fhrr.rs:48 — one canonical row kept |
| `mycelium_vsa::fhrr::Fhrr::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::fhrr::Fhrr::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr` | dedup-alias: same definition as `mycelium_vsa::Hrr` at crates/mycelium-vsa/src/hrr.rs:54 — one canonical row kept |
| `mycelium_vsa::hrr::Hrr::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::hrr::Hrr::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB` | dedup-alias: same definition as `mycelium_vsa::MapB` at crates/mycelium-vsa/src/mapb.rs:46 — one canonical row kept |
| `mycelium_vsa::mapb::MapB::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapb::MapB::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI` | dedup-alias: same definition as `mycelium_vsa::MapI` at crates/mycelium-vsa/src/mapi.rs:20 — one canonical row kept |
| `mycelium_vsa::mapi::MapI::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::mapi::MapI::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::matrix::RFC0003_MATRIX:` | dedup-alias: same definition as `mycelium_vsa::RFC0003_MATRIX:` at crates/mycelium-vsa/src/matrix.rs:34 — one canonical row kept |
| `mycelium_vsa::matrix::matrix_tag` | dedup-alias: same definition as `mycelium_vsa::matrix_tag` at crates/mycelium-vsa/src/matrix.rs:70 — one canonical row kept |
| `mycelium_vsa::recon::reconstruct_factors` | dedup-alias: same definition as `mycelium_vsa::reconstruct_factors` at crates/mycelium-vsa/src/recon.rs:90 — one canonical row kept |
| `mycelium_vsa::recon::reconstruct_factors_selected` | dedup-alias: same definition as `mycelium_vsa::reconstruct_factors_selected` at crates/mycelium-vsa/src/recon.rs:158 — one canonical row kept |
| `mycelium_vsa::recon::reconstruct_role` | dedup-alias: same definition as `mycelium_vsa::reconstruct_role` at crates/mycelium-vsa/src/recon.rs:24 — one canonical row kept |
| `mycelium_vsa::resonator::Cleanup` | dedup-alias: same definition as `mycelium_vsa::Cleanup` at crates/mycelium-vsa/src/resonator.rs:63 — one canonical row kept |
| `mycelium_vsa::resonator::Cleanup::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Cleanup::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Cleanup::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Cleanup::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Cleanup::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Cleanup::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Factorization` | dedup-alias: same definition as `mycelium_vsa::Factorization` at crates/mycelium-vsa/src/resonator.rs:196 — one canonical row kept |
| `mycelium_vsa::resonator::Factorization::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Factorization::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Factorization::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Factorization::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Factorization::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Factorization::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Init` | dedup-alias: same definition as `mycelium_vsa::Init` at crates/mycelium-vsa/src/resonator.rs:85 — one canonical row kept |
| `mycelium_vsa::resonator::Init::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Init::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Init::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Init::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Init::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::Init::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::IterationRecord` | dedup-alias: same definition as `mycelium_vsa::IterationRecord` at crates/mycelium-vsa/src/resonator.rs:167 — one canonical row kept |
| `mycelium_vsa::resonator::IterationRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::IterationRecord::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::IterationRecord::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::IterationRecord::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::IterationRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::IterationRecord::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::MAPI_RESONATOR_PROFILE:` | dedup-alias: same definition as `mycelium_vsa::MAPI_RESONATOR_PROFILE:` at crates/mycelium-vsa/src/resonator.rs:301 — one canonical row kept |
| `mycelium_vsa::resonator::ResonatorParams` | dedup-alias: same definition as `mycelium_vsa::ResonatorParams` at crates/mycelium-vsa/src/resonator.rs:95 — one canonical row kept |
| `mycelium_vsa::resonator::ResonatorParams::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorParams::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorParams::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorParams::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorParams::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorParams::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorProfile` | dedup-alias: same definition as `mycelium_vsa::ResonatorProfile` at crates/mycelium-vsa/src/resonator.rs:209 — one canonical row kept |
| `mycelium_vsa::resonator::ResonatorProfile::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorProfile::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorProfile::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorProfile::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorProfile::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorProfile::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorTrace` | dedup-alias: same definition as `mycelium_vsa::ResonatorTrace` at crates/mycelium-vsa/src/resonator.rs:181 — one canonical row kept |
| `mycelium_vsa::resonator::ResonatorTrace::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorTrace::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorTrace::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorTrace::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorTrace::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::ResonatorTrace::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::StopReason` | dedup-alias: same definition as `mycelium_vsa::StopReason` at crates/mycelium-vsa/src/resonator.rs:137 — one canonical row kept |
| `mycelium_vsa::resonator::StopReason::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::StopReason::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::StopReason::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::StopReason::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::StopReason::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::StopReason::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::resonator::factorize` | dedup-alias: same definition as `mycelium_vsa::factorize` at crates/mycelium-vsa/src/resonator.rs:326 — one canonical row kept |
| `mycelium_vsa::sbc::Sbc` | dedup-alias: same definition as `mycelium_vsa::Sbc` at crates/mycelium-vsa/src/sbc.rs:36 — one canonical row kept |
| `mycelium_vsa::sbc::Sbc::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::bind` | definition not found via regex heuristic (kind='fn', name='bind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::bundle` | definition not found via regex heuristic (kind='fn', name='bundle') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::clone` | definition not found via regex heuristic (kind='fn', name='clone') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::eq` | definition not found via regex heuristic (kind='fn', name='eq') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::fmt` | definition not found via regex heuristic (kind='fn', name='fmt') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::intrinsic_guarantee` | definition not found via regex heuristic (kind='fn', name='intrinsic_guarantee') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::model_id` | definition not found via regex heuristic (kind='fn', name='model_id') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::permute` | definition not found via regex heuristic (kind='fn', name='permute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::self_inverse` | definition not found via regex heuristic (kind='fn', name='self_inverse') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::similarity` | definition not found via regex heuristic (kind='fn', name='similarity') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::unbind` | definition not found via regex heuristic (kind='fn', name='unbind') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `mycelium_vsa::sbc::Sbc::unpermute` | definition not found via regex heuristic (kind='fn', name='unpermute') — possibly macro-generated or cfg-gated |
| `u128::myc_cmp` | definition not found via regex heuristic (kind='fn', name='myc_cmp') — possibly macro-generated or cfg-gated |
| `u128::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `u128::myc_ge` | definition not found via regex heuristic (kind='fn', name='myc_ge') — possibly macro-generated or cfg-gated |
| `u128::myc_gt` | definition not found via regex heuristic (kind='fn', name='myc_gt') — possibly macro-generated or cfg-gated |
| `u128::myc_le` | definition not found via regex heuristic (kind='fn', name='myc_le') — possibly macro-generated or cfg-gated |
| `u128::myc_lt` | definition not found via regex heuristic (kind='fn', name='myc_lt') — possibly macro-generated or cfg-gated |
| `u128::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `u128::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
| `u128::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u128::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u128::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u128::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u128::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u128::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u128::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u128::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u128::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u16::myc_cmp` | definition not found via regex heuristic (kind='fn', name='myc_cmp') — possibly macro-generated or cfg-gated |
| `u16::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `u16::myc_ge` | definition not found via regex heuristic (kind='fn', name='myc_ge') — possibly macro-generated or cfg-gated |
| `u16::myc_gt` | definition not found via regex heuristic (kind='fn', name='myc_gt') — possibly macro-generated or cfg-gated |
| `u16::myc_le` | definition not found via regex heuristic (kind='fn', name='myc_le') — possibly macro-generated or cfg-gated |
| `u16::myc_lt` | definition not found via regex heuristic (kind='fn', name='myc_lt') — possibly macro-generated or cfg-gated |
| `u16::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `u16::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
| `u16::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u16::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u16::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u16::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `u16::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `u16::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `u32::myc_cmp` | definition not found via regex heuristic (kind='fn', name='myc_cmp') — possibly macro-generated or cfg-gated |
| `u32::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `u32::myc_ge` | definition not found via regex heuristic (kind='fn', name='myc_ge') — possibly macro-generated or cfg-gated |
| `u32::myc_gt` | definition not found via regex heuristic (kind='fn', name='myc_gt') — possibly macro-generated or cfg-gated |
| `u32::myc_le` | definition not found via regex heuristic (kind='fn', name='myc_le') — possibly macro-generated or cfg-gated |
| `u32::myc_lt` | definition not found via regex heuristic (kind='fn', name='myc_lt') — possibly macro-generated or cfg-gated |
| `u32::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `u32::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
| `u32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u32::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u32::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `u32::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `u64::myc_cmp` | definition not found via regex heuristic (kind='fn', name='myc_cmp') — possibly macro-generated or cfg-gated |
| `u64::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `u64::myc_ge` | definition not found via regex heuristic (kind='fn', name='myc_ge') — possibly macro-generated or cfg-gated |
| `u64::myc_gt` | definition not found via regex heuristic (kind='fn', name='myc_gt') — possibly macro-generated or cfg-gated |
| `u64::myc_le` | definition not found via regex heuristic (kind='fn', name='myc_le') — possibly macro-generated or cfg-gated |
| `u64::myc_lt` | definition not found via regex heuristic (kind='fn', name='myc_lt') — possibly macro-generated or cfg-gated |
| `u64::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `u64::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
| `u64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u64::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u64::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `u8::myc_cmp` | definition not found via regex heuristic (kind='fn', name='myc_cmp') — possibly macro-generated or cfg-gated |
| `u8::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `u8::myc_ge` | definition not found via regex heuristic (kind='fn', name='myc_ge') — possibly macro-generated or cfg-gated |
| `u8::myc_gt` | definition not found via regex heuristic (kind='fn', name='myc_gt') — possibly macro-generated or cfg-gated |
| `u8::myc_le` | definition not found via regex heuristic (kind='fn', name='myc_le') — possibly macro-generated or cfg-gated |
| `u8::myc_lt` | definition not found via regex heuristic (kind='fn', name='myc_lt') — possibly macro-generated or cfg-gated |
| `u8::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `u8::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
| `u8::narrow` | definition not found via regex heuristic (kind='fn', name='narrow') — possibly macro-generated or cfg-gated |
| `u8::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `u8::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `u8::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `u8::widen` | definition not found via regex heuristic (kind='fn', name='widen') — possibly macro-generated or cfg-gated |
| `usize::myc_cmp` | definition not found via regex heuristic (kind='fn', name='myc_cmp') — possibly macro-generated or cfg-gated |
| `usize::myc_eq` | definition not found via regex heuristic (kind='fn', name='myc_eq') — possibly macro-generated or cfg-gated |
| `usize::myc_ge` | definition not found via regex heuristic (kind='fn', name='myc_ge') — possibly macro-generated or cfg-gated |
| `usize::myc_gt` | definition not found via regex heuristic (kind='fn', name='myc_gt') — possibly macro-generated or cfg-gated |
| `usize::myc_le` | definition not found via regex heuristic (kind='fn', name='myc_le') — possibly macro-generated or cfg-gated |
| `usize::myc_lt` | definition not found via regex heuristic (kind='fn', name='myc_lt') — possibly macro-generated or cfg-gated |
| `usize::myc_ne` | definition not found via regex heuristic (kind='fn', name='myc_ne') — possibly macro-generated or cfg-gated |
| `usize::myc_partial_cmp` | definition not found via regex heuristic (kind='fn', name='myc_partial_cmp') — possibly macro-generated or cfg-gated |
