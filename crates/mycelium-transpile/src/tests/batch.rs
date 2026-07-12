//! Unit tests for directory/batch mode (`src/batch.rs`, M-873 follow-on) — no new dev-dependency
//! (e.g. `tempfile`) added for this, per the crate's kickoff-scoped minimal-deps stance (see
//! `Cargo.toml`'s `quote` comment): fixtures are written directly under `std::env::temp_dir()` in
//! a per-test unique subdirectory, cleaned up at the end of each test.

use crate::batch::{discover_rs_files, output_rel_path, summarize, transpile_batch};
use crate::gap::Category;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A fresh, empty temp directory scoped to one test (`tag` disambiguates by test name; the
/// counter disambiguates parallel test threads sharing a `tag`/pid).
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "mycelium-transpile-batch-test-{tag}-{}-{n}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        TempDir(dir)
    }

    fn write(&self, rel: &str, content: &str) {
        let path = self.0.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dir");
        }
        fs::write(&path, content).expect("write fixture file");
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// `discover_rs_files` recurses `*.rs` but skips any `tests` directory component (both a
/// crate-root `tests/` dir and the in-crate `src/tests/` layout) and any `tests.rs` file (the
/// older single-file test-module shape, e.g. `mycelium-std-fmt/src/tests.rs`).
#[test]
fn discover_skips_tests_dirs_and_files() {
    let tmp = TempDir::new("discover");
    tmp.write("lib.rs", "fn a(x: bool) -> bool { x }");
    tmp.write("helper.rs", "fn b(x: bool) -> bool { x }");
    tmp.write("tests.rs", "fn only_tests() {}");
    tmp.write("tests/integration.rs", "fn only_tests_2() {}");
    tmp.write("nested/mod_a.rs", "fn c(x: bool) -> bool { x }");
    tmp.write("nested/tests/deep.rs", "fn only_tests_3() {}");

    let found = discover_rs_files(tmp.path()).expect("discover succeeds");
    let names: Vec<String> = found
        .iter()
        .map(|p| {
            p.strip_prefix(tmp.path())
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();

    assert_eq!(
        names,
        vec![
            "helper.rs".to_string(),
            "lib.rs".to_string(),
            "nested/mod_a.rs".to_string(),
        ],
        "expected exactly the non-test .rs files, sorted; got {names:?}"
    );
}

/// `discover_rs_files` over an empty directory returns an empty (not missing/erroring) list —
/// never-silent for the degenerate case.
#[test]
fn discover_over_empty_dir_returns_empty() {
    let tmp = TempDir::new("discover-empty");
    let found = discover_rs_files(tmp.path()).expect("discover succeeds");
    assert!(found.is_empty(), "expected no files, got {found:?}");
}

/// `transpile_batch` + `summarize` over a small multi-file fixture: per-file summaries roll up
/// exactly into the batch totals (sum of counts, union of gaps), and the per-file never-silent
/// invariant (emitted + gaps >= total items) holds for every file in the batch — the batch-mode
/// analogue of `src/tests/invariant.rs`'s single-file check.
#[test]
fn batch_summary_totals_match_per_file_sums() {
    let tmp = TempDir::new("summary");
    // All-expressible file.
    tmp.write(
        "a.rs",
        "enum Ordering { Less, Equal, Greater }\nfn is_lt(o: bool) -> bool { o }",
    );
    // A file with a mix of emitted + gapped items (a known hard gap: named-field struct).
    tmp.write("b.rs", "struct Foo { x: u8 }\nfn ok(x: bool) -> bool { x }");
    // An all-gapped file (macro_rules! def).
    tmp.write("c.rs", "macro_rules! m { () => {}; }");

    let files = discover_rs_files(tmp.path()).expect("discover succeeds");
    assert_eq!(files.len(), 3, "expected all 3 fixture files discovered");

    let (results, failures) = transpile_batch(&files);
    assert!(
        failures.is_empty(),
        "expected every fixture file to parse, got failures={failures:?}"
    );
    assert_eq!(results.len(), 3);

    // Per-crate (per-file, here) never-silent invariant: emitted + gaps >= total items.
    for r in &results {
        let covered = r.report.emitted_items.len() + r.report.gaps.len();
        assert!(
            covered >= r.report.total_top_level_items,
            "never-silent invariant violated for {}: {} items but only {covered} \
             emitted+gap record(s)",
            r.path.display(),
            r.report.total_top_level_items
        );
    }

    let (batch_summary, union) = summarize(&results, tmp.path());
    assert_eq!(batch_summary.files.len(), 3);

    let sum_total_items: usize = batch_summary.files.iter().map(|f| f.total_items).sum();
    let sum_non_test: usize = batch_summary.files.iter().map(|f| f.non_test_items).sum();
    let sum_emitted: usize = batch_summary.files.iter().map(|f| f.emitted).sum();
    let sum_gaps: usize = batch_summary.files.iter().map(|f| f.gaps).sum();

    assert_eq!(batch_summary.totals.total_items, sum_total_items);
    assert_eq!(batch_summary.totals.non_test_items, sum_non_test);
    assert_eq!(batch_summary.totals.emitted, sum_emitted);
    assert_eq!(batch_summary.totals.gaps, sum_gaps);
    assert_eq!(
        union.gaps.len(),
        sum_gaps,
        "union.gap.json must carry every gap from every file, none dropped"
    );

    // At least one item landed (a.rs) and at least one gapped (b.rs's struct, c.rs's macro).
    assert!(
        sum_emitted > 0,
        "expected some emitted items across the batch"
    );
    assert!(sum_gaps > 0, "expected some gaps across the batch");

    // Per-category counts in the union must sum to the same total as `totals.category_counts`
    // (they're built from the same per-file counters) and must equal the raw gap count.
    let union_cat_sum: usize = union.category_counts.values().sum();
    assert_eq!(union_cat_sum, sum_gaps);
    let totals_cat_sum: usize = batch_summary.totals.category_counts.values().sum();
    assert_eq!(totals_cat_sum, sum_gaps);

    // Expressible percentage is a real percentage over the non-test denominator.
    assert!(
        (0.0..=100.0).contains(&batch_summary.totals.expressible_pct),
        "expressible_pct out of [0,100]: {}",
        batch_summary.totals.expressible_pct
    );

    // M-1044 / DN-109 §5.2: one pure-`Keep` remap entry per transpiled file, none dropped —
    // never-silent at the nodule-provenance level too, not just the gap level.
    assert_eq!(
        batch_summary.remap.nodules.len(),
        results.len(),
        "expected exactly one remap entry per transpiled file"
    );
    for n in &batch_summary.remap.nodules {
        assert_eq!(n.operation, crate::remap::RemapOperation::Keep);
        assert_eq!(n.safety, crate::remap::RemapSafety::Safe);
        assert!(n.identity_neutral, "a pure Keep is identity-neutral");
        assert!(
            !n.api_surface_changed,
            "a pure Keep must not claim an API-surface change"
        );
        assert_eq!(n.guarantee, "Declared");
        assert_eq!(n.sources.len(), 1, "a Keep has exactly one source file");
    }
    // v0 Mechanical-only: no idiom-choice instrumentation exists yet, so the field is honestly
    // empty rather than fabricated (see `src/remap.rs` module docs).
    assert!(batch_summary.remap.idiom_choices.is_empty());
}

/// A batch over zero files (e.g. a directory that discovers nothing) yields an honest all-zero
/// summary, not a divide-by-zero panic or a fabricated percentage.
#[test]
fn batch_summary_over_zero_files_is_all_zero_not_a_panic() {
    let (batch_summary, union) = summarize(&[], Path::new("crates/x/src"));
    assert!(batch_summary.files.is_empty());
    assert_eq!(batch_summary.totals.total_items, 0);
    assert_eq!(batch_summary.totals.emitted, 0);
    assert_eq!(batch_summary.totals.gaps, 0);
    assert_eq!(batch_summary.totals.expressible_pct, 0.0);
    assert!(union.gaps.is_empty());
    // Honest all-zero: no nodules recorded either (nothing was transpiled).
    assert!(batch_summary.remap.nodules.is_empty());
}

// ── M-1006 Phase-2: path-qualified batch output (`output_rel_path`) ──────────────────────────────

/// A file under the batch root maps to its **relative path** with `.rs` stripped, so a whole-corpus
/// run mirrors the source tree under the out-dir.
#[test]
fn output_rel_path_mirrors_the_tree_under_root() {
    let root = Path::new("crates");
    let got = output_rel_path(Path::new("crates/mycelium-core/src/lib.rs"), root)
        .expect("under root -> Ok");
    assert_eq!(got, PathBuf::from("mycelium-core/src/lib"));
}

/// The whole-corpus collision the fix targets: two crates' `lib.rs` must map to **distinct** outputs
/// (path-qualified), never the same flat `lib` — the property that makes the run non-lossy.
#[test]
fn same_stem_files_in_different_crates_get_distinct_outputs() {
    let root = Path::new("crates");
    let a = output_rel_path(Path::new("crates/mycelium-core/src/lib.rs"), root).unwrap();
    let b = output_rel_path(Path::new("crates/mycelium-std/src/lib.rs"), root).unwrap();
    assert_ne!(
        a, b,
        "two crates' lib.rs must not collide at the same output path"
    );
    assert_eq!(a, PathBuf::from("mycelium-core/src/lib"));
    assert_eq!(b, PathBuf::from("mycelium-std/src/lib"));
}

/// A flat single-crate `src/` root reduces to the bare stem — identical to the pre-Phase-2 flat
/// naming, which is why the committed `gen/myc-drafts/` 17-target manifest sees zero churn.
#[test]
fn flat_single_crate_root_reduces_to_bare_stem() {
    let root = Path::new("crates/mycelium-std-fs/src");
    let got = output_rel_path(Path::new("crates/mycelium-std-fs/src/lib.rs"), root).unwrap();
    assert_eq!(got, PathBuf::from("lib"));
}

/// A file not under the batch root falls back to the bare stem via `Err` (the caller warns — never
/// a silent mis-placement, G2).
#[test]
fn not_under_root_falls_back_to_bare_stem_via_err() {
    let root = Path::new("crates");
    let got = output_rel_path(Path::new("/elsewhere/foo.rs"), root);
    assert_eq!(got, Err(PathBuf::from("foo")));
}

/// Only the final `.rs` is stripped — a `foo.bar.rs` source keeps its `foo.bar` stem (so `append_ext`
/// in the CLI yields `foo.bar.myc`, not `foo.myc`).
#[test]
fn only_the_rs_extension_is_stripped() {
    let root = Path::new("crates/x/src");
    let got = output_rel_path(Path::new("crates/x/src/foo.bar.rs"), root).unwrap();
    assert_eq!(got, PathBuf::from("foo.bar"));
}

// ── Gap-close-2 (DN-34 §8.19/§8.20): the batch-scoped cross-nodule symbol table ─────────────────
//
// `transpile_batch`'s two real sibling files below: `checkty.rs` declares an emittable `Width`
// struct and a deliberately-unemittable `Env` struct (a named-field record whose field type has no
// mapping, so it stays a real `Category::Struct` gap — never in `checkty`'s `emitted_items`).
// `mono.rs` imports both, plus an external `std::` name, exercising: a full resolve (`Width`), an
// in-batch-sibling-but-gapped miss (`Env`), and an out-of-batch miss (`std::collections::BTreeMap`)
// side by side in the same run.

fn checkty_fixture() -> &'static str {
    "pub struct Width(u8);\nstruct Env { x: NotARealMappableType }\nfn helper(x: bool) -> bool { x }"
}

fn mono_fixture() -> &'static str {
    "use std::collections::BTreeMap;\nuse crate::checkty::{Width, Env};\nfn mono_helper(x: bool) -> bool { x }"
}

/// The end-to-end cross-nodule resolution: `mono.rs`'s `use crate::checkty::{Width, Env};`
/// partially resolves (`Width` — `checkty` actually emitted it) and partially gaps (`Env` — a
/// batch sibling, but it gapped that name rather than emitting it), landing as ONE `Outcome::Emitted`
/// item carrying the unresolved leaf as a `sub_gaps` entry (both "emitted" and "honestly flagged" —
/// never neither, G2).
#[test]
fn cross_nodule_use_partially_resolves_against_a_batch_sibling() {
    let tmp = TempDir::new("cross-nodule-partial");
    tmp.write("checkty.rs", checkty_fixture());
    tmp.write("mono.rs", mono_fixture());

    let files = discover_rs_files(tmp.path()).expect("discover succeeds");
    let (results, failures) = transpile_batch(&files);
    assert!(failures.is_empty(), "unexpected failures: {failures:?}");

    let mono = results
        .iter()
        .find(|r| r.path.file_name().unwrap() == "mono.rs")
        .expect("mono.rs result present");

    // The resolved leaf landed as real emitted `.myc` text, home-qualified against checkty's own
    // derived nodule path — never a bare `Width` (no-bare-name-collapse, the M-1060 lesson).
    assert!(
        mono.myc.contains("use checkty.Width;") || mono.myc.contains(".Width;"),
        "expected a qualified `use ….Width;` line in mono.myc, got:\n{}",
        mono.myc
    );
    assert!(
        !mono.myc.lines().any(|l| l.trim() == "use Width;"),
        "must never emit a bare, unqualified `use Width;` — no-bare-name-collapse (VR-5/G2); \
         got:\n{}",
        mono.myc
    );
    assert!(
        mono.report
            .emitted_items
            .iter()
            .any(|n| n.starts_with("use:") && n.contains("Width")),
        "expected an emitted `use:…Width…` item, got {:?}",
        mono.report.emitted_items
    );

    // `Env` and the external `std::` import both still gap — never silently dropped, never
    // guessed — but now with the NEW, more precise reasons (a real symbol table exists, so the
    // old blanket "no cross-nodule symbol table" claim would itself be inaccurate).
    let import_gaps: Vec<&str> = mono
        .report
        .gaps
        .iter()
        .filter(|g| g.category == Category::Import)
        .map(|g| g.reason.as_str())
        .collect();
    assert!(
        import_gaps.iter().any(|r| r.contains("Env")
            && r.contains("checkty")
            && r.contains("gapped it rather than emitting it")),
        "expected an Env-naming, sibling-gapped-it reason among {import_gaps:?}"
    );
    assert!(
        import_gaps
            .iter()
            .any(|r| r.contains("BTreeMap") && r.contains("not a sibling module")),
        "expected a BTreeMap-naming, not-a-batch-sibling reason among {import_gaps:?}"
    );
}

/// The other half of the correctness bar the task names explicitly: a resolved cross-nodule `use`
/// is only the checker-accepted form when the referenced item is itself `pub` in its home nodule
/// (DN-113/M-1060's `resolve_imports` is `pub`-gated) — so `checkty.rs`'s `Width` (referenced by
/// `mono.rs` above) must be emitted `pub`, while `helper` (never referenced by any sibling) stays
/// unmarked, exactly as before this lever landed.
#[test]
fn resolved_cross_nodule_reference_marks_the_sibling_item_pub() {
    let tmp = TempDir::new("pub-propagation");
    tmp.write("checkty.rs", checkty_fixture());
    tmp.write("mono.rs", mono_fixture());

    let files = discover_rs_files(tmp.path()).expect("discover succeeds");
    let (results, failures) = transpile_batch(&files);
    assert!(failures.is_empty(), "unexpected failures: {failures:?}");

    let checkty = results
        .iter()
        .find(|r| r.path.file_name().unwrap() == "checkty.rs")
        .expect("checkty.rs result present");

    assert!(
        checkty.myc.contains("pub type Width"),
        "Width is referenced by a sibling's resolved `use` — expected a `pub` prefix; got:\n{}",
        checkty.myc
    );
    // `helper` was never imported by any sibling in this batch, so it stays exactly as before —
    // no spurious `pub` on every item (only the genuinely-referenced ones).
    assert!(
        !checkty.myc.contains("pub fn helper"),
        "helper is never cross-nodule-referenced — must not be marked pub; got:\n{}",
        checkty.myc
    );
}

/// A batch with **no** in-batch cross-referencing `use` (every file is import-independent) is
/// byte-identical to the pre-gap-close-2 driver: every `use` still gaps, nothing is ever marked
/// `pub`. Guards against the two-pass driver silently changing behavior for the common case.
#[test]
fn batch_with_no_cross_file_use_is_unaffected() {
    let tmp = TempDir::new("no-cross-file-use");
    tmp.write(
        "a.rs",
        "pub struct Foo(u8);\nfn helper(x: bool) -> bool { x }",
    );
    tmp.write("b.rs", "pub struct Bar(u8);\nuse std::fmt;\n");

    let files = discover_rs_files(tmp.path()).expect("discover succeeds");
    let (results, _failures) = transpile_batch(&files);

    let a = results
        .iter()
        .find(|r| r.path.file_name().unwrap() == "a.rs")
        .unwrap();
    let b = results
        .iter()
        .find(|r| r.path.file_name().unwrap() == "b.rs")
        .unwrap();
    assert!(
        !a.myc.contains("pub "),
        "no sibling references Foo/helper — nothing should be pub-marked; got:\n{}",
        a.myc
    );
    assert!(
        b.report.gaps.iter().any(|g| g.category == Category::Import),
        "the unresolvable `use std::fmt;` must still gap"
    );
}

/// A rename/self/glob leaf on an in-batch head never resolves (scoped OUT of this increment —
/// deliberately, not a bug): a solitary `use crate::checkty::Width as W;` still gaps the whole
/// item (the only leaf is a `Rename`), never silently emitting the aliased form.
#[test]
fn renamed_glob_and_self_leaves_on_an_in_batch_head_still_gap() {
    let tmp = TempDir::new("scoped-out-leaves");
    tmp.write("checkty.rs", checkty_fixture());
    tmp.write(
        "consumer.rs",
        "use crate::checkty::Width as W;\nuse crate::checkty::*;\nfn f(x: bool) -> bool { x }",
    );

    let files = discover_rs_files(tmp.path()).expect("discover succeeds");
    let (results, failures) = transpile_batch(&files);
    assert!(failures.is_empty(), "unexpected failures: {failures:?}");

    let consumer = results
        .iter()
        .find(|r| r.path.file_name().unwrap() == "consumer.rs")
        .unwrap();
    let import_gap_count = consumer
        .report
        .gaps
        .iter()
        .filter(|g| g.category == Category::Import)
        .count();
    assert_eq!(
        import_gap_count, 2,
        "both the rename and the glob must gap (scoped out, never guessed); gaps: {:?}",
        consumer.report.gaps
    );
    assert!(
        !consumer.myc.contains("use "),
        "neither leaf resolves, so consumer.myc must carry no `use` line at all; got:\n{}",
        consumer.myc
    );
}
