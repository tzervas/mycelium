//! Differential tests for `std.time` (M-1021, kickoff `spw` / Wave-0 pilot, RFC-0031 D5) — the
//! `.myc` port of `crates/mycelium-std-time/src/lib.rs`'s EXPRESSIBLE-TODAY subset.
//!
//! # Scope (surface-check, D5 row 1 — see `lib/std/time.myc`'s module doc for the full writeup)
//! Ported + tested here: the three typed instant kinds + `Duration` as value ADTs, their bit-
//! preserving constructors/accessors, the SIGNED `Duration` comparison surface + the UNSIGNED instant
//! comparison surface (the kernel surfaces two's-complement compare via `lt_s`/`eq`, uncapped), the
//! deterministic `ManualClock` (value-semantic functional-update setters + its declared-effect reads),
//! and the RFC-0016 §4.5 guarantee matrix as checked data (the module's honesty crux). FLAGged, NOT
//! ported (VR-5/G2 — never a hollow port): the SIGNED i128 `Duration`/instant ARITHMETIC and the
//! instant differences (blocked on the kernel's TC_MAX_WIDTH=64 two's-complement prim cap — a
//! STRUCTURAL blocker, FLAG → enb), the OS `SystemClock`/`ClockSource`-trait floor (host FFI, deferred
//! M-541), the saturating `advance_mono`/`step_logical` (saturating_add ≠ never-silent checked add_u),
//! and the `TimeErr` diagnostic payloads + `Display` (no string/formatter/macro surface). See the
//! nodule doc for each FLAG's grounding.
//!
//! # Harness design
//! Execution/comparison machinery lives in the shared [`harness`] fixture (M-925) — this file supplies
//! the nodule's `include_str!`, the per-case drivers, and the **row-4 Rust-oracle wiring**. Every
//! `expected` value below is computed **live from the retained `mycelium_std_time` oracle** (its public
//! API + `GUARANTEE_MATRIX`), not a hardcoded constant — a real divergence between the Rust source and
//! the `.myc` transcription flips the computed oracle value and fails the corresponding case. That
//! oracle agreement is what earns the `Empirical` tag (VR-5).
//!
//! # Honesty tags (carried, never upgraded in translation — VR-5)
//! - **`Exact`** — the constructors/accessors/setters (bit-preserving/value-copying) and the
//!   comparison surface (`eq`/`lt`/`lt_s` are Exact over the finite Binary{N} domain).
//! - **`Declared`** — the declared-effect wrapper `new`/`into_inner`, the `ManualClock` reads (the
//!   effect is declared by the wrapper TYPE — structural, not a theorem), and the guarantee-matrix
//!   row data (asserted, hand-transcribed).
//! - **`Empirical`** — the three-way differential agreement (L1-eval ≡ L0-interp ≡ AOT) AND the
//!   Rust-oracle agreement below, validated by trial on the cases exercised.

mod harness;

use core::cmp::Ordering;
use mycelium_std_core::GuaranteeStrength;
use mycelium_std_time::{
    duration_cmp, ClockSource, Duration, LogicalInstant, ManualClock, MonoInstant, WallInstant,
    GUARANTEE_MATRIX,
};

/// The std.time nodule source, loaded at compile time — the single source of truth.
const TIME_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../lib/std/time.myc"
));

/// Build a full test program by appending a driver to the nodule source.
fn program(driver: &str) -> String {
    harness::program(TIME_SRC, driver)
}

/// Thin re-export of the shared [`harness::assert_three_way`].
fn assert_three_way(label: &str, src: &str, expected_src: &str) {
    harness::assert_three_way(label, src, expected_src);
}

// ── Rendering helpers (Rust value → the `.myc` literal that denotes the same Core value) ─────────────

/// Render a Rust `bool` as the `.myc` `Bool` literal.
fn myc_bool(b: bool) -> &'static str {
    if b {
        "True"
    } else {
        "False"
    }
}

/// Render a `u64` as the exact-width `Binary{64}` literal (leading zeros preserved — the `.myc` lexer
/// widths a `0b…` literal by its digit count, so the full 64 digits are required).
fn myc_u64_bin(n: u64) -> String {
    format!("0b{n:064b}")
}

/// Render an `i128` as the exact-width `Binary{128}` two's-complement literal. `n as u128`
/// reinterprets the signed bit pattern (two's complement), which is exactly how the port stores a
/// signed span/instant (bit-preserving, sign uninterpreted by the ctor/accessor).
fn myc_i128_bin(n: i128) -> String {
    format!("0b{:0128b}", n as u128)
}

/// Render the `n`-deep nested `add_u(0b1, …, 0b0)…` chain the port's recursive `Binary{8}` counters
/// (`matrix_len`, `count_exact`, `count_declared`, `explainable_count`) expand to for `n` matching
/// elements — recomputing via the SAME primitive-op composition gives the reference program the
/// matching `Derived` provenance the M-925 harness's `check_core` comparison requires (the
/// `std_core.rs` precedent), while remaining an INDEPENDENT check of the count.
fn myc_count_chain(n: u8) -> String {
    let mut expr = "0b0000_0000".to_owned();
    for _ in 0..n {
        expr = format!("add_u(0b0000_0001, {expr})");
    }
    expr
}

/// Render a `core::cmp::Ordering` as the `.myc` `Ordering` constructor name.
fn myc_ordering(o: Ordering) -> &'static str {
    match o {
        Ordering::Less => "Lt",
        Ordering::Equal => "Eq",
        Ordering::Greater => "Gt",
    }
}

/// A `nodule ref;` program whose `main` returns a `Bool`.
fn ref_bool(b: bool) -> String {
    format!("nodule ref;\nfn main() => Bool = {};", myc_bool(b))
}

/// A `nodule ref;` program whose `main` returns a `Binary{8}` count (via the same `add_u` chain).
fn ref_count(n: u8) -> String {
    format!(
        "nodule ref;\nfn main() => Binary{{8}} = {};",
        myc_count_chain(n)
    )
}

/// A `nodule ref;` program whose `main` returns an `Ordering` (the type is redeclared locally, exactly
/// as the port declares it, so both elaborate to the same Core sum-constructor).
fn ref_ordering(o: Ordering) -> String {
    format!(
        "nodule ref;\ntype Ordering = Lt | Eq | Gt;\nfn main() => Ordering = {};",
        myc_ordering(o)
    )
}

// ── §A. Constructor/accessor round-trips (Exact, bit-preserving — driven against the oracle) ─────────

/// `mono_as_nanos(mono_from_nanos(n)) == n` — the oracle's `MonoInstant::from_nanos(n).as_nanos()`.
#[test]
fn mono_instant_roundtrip_matches_oracle() {
    let n: u64 = 1_000_000_000;
    let expected = MonoInstant::from_nanos(n).as_nanos();
    let driver = format!(
        "fn main() => Binary{{64}} = mono_as_nanos(mono_from_nanos({}));",
        myc_u64_bin(n)
    );
    let expected_src = format!(
        "nodule ref;\nfn main() => Binary{{64}} = {};",
        myc_u64_bin(expected)
    );
    assert_three_way("mono roundtrip == oracle", &program(&driver), &expected_src);
}

/// `logical_as_tick(logical_from_tick(n)) == n` — the oracle's `LogicalInstant::from_tick(n).as_tick()`.
#[test]
fn logical_instant_roundtrip_matches_oracle() {
    let n: u64 = 42;
    let expected = LogicalInstant::from_tick(n).as_tick();
    let driver = format!(
        "fn main() => Binary{{64}} = logical_as_tick(logical_from_tick({}));",
        myc_u64_bin(n)
    );
    let expected_src = format!(
        "nodule ref;\nfn main() => Binary{{64}} = {};",
        myc_u64_bin(expected)
    );
    assert_three_way(
        "logical roundtrip == oracle",
        &program(&driver),
        &expected_src,
    );
}

/// `wall_as_nanos_since_epoch(wall_from_nanos_since_epoch(n)) == n` (signed i128, positive value).
#[test]
fn wall_instant_roundtrip_matches_oracle() {
    let n: i128 = 1_717_000_000_000_000_000;
    let expected = WallInstant::from_nanos_since_epoch(n).as_nanos_since_epoch();
    let driver = format!(
        "fn main() => Binary{{128}} = wall_as_nanos_since_epoch(wall_from_nanos_since_epoch({}));",
        myc_i128_bin(n)
    );
    let expected_src = format!(
        "nodule ref;\nfn main() => Binary{{128}} = {};",
        myc_i128_bin(expected)
    );
    assert_three_way("wall roundtrip == oracle", &program(&driver), &expected_src);
}

/// `dur_as_nanos(dur_from_nanos(n)) == n` for a NEGATIVE span (the signed bit pattern round-trips).
#[test]
fn duration_roundtrip_negative_matches_oracle() {
    let n: i128 = -2_500_000_000;
    let expected = Duration::from_nanos(n).as_nanos();
    let driver = format!(
        "fn main() => Binary{{128}} = dur_as_nanos(dur_from_nanos({}));",
        myc_i128_bin(n)
    );
    let expected_src = format!(
        "nodule ref;\nfn main() => Binary{{128}} = {};",
        myc_i128_bin(expected)
    );
    assert_three_way(
        "duration roundtrip (negative) == oracle",
        &program(&driver),
        &expected_src,
    );
}

// ── §B. Duration comparison surface (Exact, SIGNED — driven against the oracle's duration_cmp) ───────

/// `dur_cmp` returns the same `Ordering` the oracle's `duration_cmp` does — SIGNED, incl. negatives.
#[test]
fn duration_cmp_ordering_matches_oracle() {
    // a = -5 (signed) < b = 3: a naive UNSIGNED compare would call it Greater (0xF…B > 3) — this
    // asserts the SIGNED prim `lt_s` is used, matching the oracle's `i128::cmp`.
    let a: i128 = -5;
    let b: i128 = 3;
    let expected = duration_cmp(Duration::from_nanos(a), Duration::from_nanos(b));
    let driver = format!(
        "fn main() => Ordering = dur_cmp(dur_from_nanos({}), dur_from_nanos({}));",
        myc_i128_bin(a),
        myc_i128_bin(b)
    );
    assert_three_way(
        "dur_cmp(-5, 3) == oracle Ordering",
        &program(&driver),
        &ref_ordering(expected),
    );
}

/// `dur_lt(a, b)` agrees with `duration_cmp(a, b) == Less` for a signed pair (a < b).
#[test]
fn duration_lt_signed_matches_oracle() {
    let a: i128 = -5;
    let b: i128 = 3;
    let expected = duration_cmp(Duration::from_nanos(a), Duration::from_nanos(b)) == Ordering::Less;
    let driver = format!(
        "fn main() => Bool = dur_lt(dur_from_nanos({}), dur_from_nanos({}));",
        myc_i128_bin(a),
        myc_i128_bin(b)
    );
    assert_three_way(
        "dur_lt(-5, 3) == oracle",
        &program(&driver),
        &ref_bool(expected),
    );
}

/// `dur_lt` is FALSE when a > b (signed) — the negative-vs-positive orientation both ways.
#[test]
fn duration_lt_false_when_greater_matches_oracle() {
    let a: i128 = 3;
    let b: i128 = -5;
    let expected = duration_cmp(Duration::from_nanos(a), Duration::from_nanos(b)) == Ordering::Less;
    let driver = format!(
        "fn main() => Bool = dur_lt(dur_from_nanos({}), dur_from_nanos({}));",
        myc_i128_bin(a),
        myc_i128_bin(b)
    );
    assert_three_way(
        "dur_lt(3, -5) == oracle",
        &program(&driver),
        &ref_bool(expected),
    );
}

/// `dur_eq(a, a)` is TRUE — reflexive equality on a negative span.
#[test]
fn duration_eq_reflexive_matches_oracle() {
    let a: i128 = -123_456_789;
    let expected =
        duration_cmp(Duration::from_nanos(a), Duration::from_nanos(a)) == Ordering::Equal;
    let driver = format!(
        "fn main() => Bool = dur_eq(dur_from_nanos({}), dur_from_nanos({}));",
        myc_i128_bin(a),
        myc_i128_bin(a)
    );
    assert_three_way(
        "dur_eq reflexive == oracle",
        &program(&driver),
        &ref_bool(expected),
    );
}

/// `dur_is_zero(Duration::from_nanos(0))` is TRUE; a nonzero span is FALSE — the oracle's `is_zero`.
#[test]
fn duration_is_zero_matches_oracle() {
    for n in [0i128, 1, -1, 1_000_000_000] {
        let expected = Duration::from_nanos(n).is_zero();
        let driver = format!(
            "fn main() => Bool = dur_is_zero(dur_from_nanos({}));",
            myc_i128_bin(n)
        );
        assert_three_way(
            &format!("dur_is_zero({n}) == oracle"),
            &program(&driver),
            &ref_bool(expected),
        );
    }
}

/// `dur_is_negative` matches the oracle's `is_negative` across the sign boundary (the SIGNED test).
#[test]
fn duration_is_negative_matches_oracle() {
    for n in [0i128, 1, -1, i128::MIN, i128::MAX] {
        let expected = Duration::from_nanos(n).is_negative();
        let driver = format!(
            "fn main() => Bool = dur_is_negative(dur_from_nanos({}));",
            myc_i128_bin(n)
        );
        assert_three_way(
            &format!("dur_is_negative({n}) == oracle"),
            &program(&driver),
            &ref_bool(expected),
        );
    }
}

// ── §C. Instant comparison surface (Exact — mono/logical UNSIGNED u64, wall SIGNED i128) ─────────────

/// `mono_cmp` is the UNSIGNED ordering the oracle derives on `MonoInstant`'s `u64` tick.
#[test]
fn mono_cmp_unsigned_matches_oracle() {
    let a: u64 = 5;
    let b: u64 = 10;
    let expected = MonoInstant::from_nanos(a).cmp(&MonoInstant::from_nanos(b));
    let driver = format!(
        "fn main() => Ordering = mono_cmp(mono_from_nanos({}), mono_from_nanos({}));",
        myc_u64_bin(a),
        myc_u64_bin(b)
    );
    assert_three_way(
        "mono_cmp(5, 10) == oracle Ordering",
        &program(&driver),
        &ref_ordering(expected),
    );
}

/// `wall_cmp` is the SIGNED ordering the oracle derives on `WallInstant`'s `i128` field (negative <
/// positive), matching the `lt_s` prim — a witness the wall order is signed, not unsigned.
#[test]
fn wall_cmp_signed_matches_oracle() {
    let a: i128 = -5;
    let b: i128 = 3;
    let expected =
        WallInstant::from_nanos_since_epoch(a).cmp(&WallInstant::from_nanos_since_epoch(b));
    let driver = format!(
        "fn main() => Ordering = wall_cmp(wall_from_nanos_since_epoch({}), wall_from_nanos_since_epoch({}));",
        myc_i128_bin(a),
        myc_i128_bin(b)
    );
    assert_three_way(
        "wall_cmp(-5, 3) == oracle Ordering",
        &program(&driver),
        &ref_ordering(expected),
    );
}

// ── §D. ManualClock — value-semantic setters + declared-effect reads (driven against the oracle) ─────

/// A `set_mono` then `mono_now().into_inner()` read reproduces the set tick — the oracle's own
/// `ManualClock::default().set_mono(...).mono_now().into_inner().unwrap().as_nanos()`.
#[test]
fn manual_clock_mono_read_matches_oracle() {
    let n: u64 = 1_000_000_000;
    let mut clock = ManualClock::default();
    clock.set_mono(MonoInstant::from_nanos(n));
    let expected = clock
        .mono_now()
        .into_inner()
        .expect("ManualClock mono read is infallible")
        .as_nanos();
    let driver = format!(
        "fn main() => Binary{{64}} = mono_as_nanos(declared_time_into_inner(manual_mono_now(manual_set_mono(manual_clock_default(), mono_from_nanos({})))));",
        myc_u64_bin(n)
    );
    let expected_src = format!(
        "nodule ref;\nfn main() => Binary{{64}} = {};",
        myc_u64_bin(expected)
    );
    assert_three_way(
        "manual mono read == oracle",
        &program(&driver),
        &expected_src,
    );
}

/// The WALL read flows through the `DeclaredTimeEntropy` wrapper (the typed-distinction crux) and
/// reproduces the set signed nanos-since-epoch — the oracle's `wall_now().into_inner()`.
#[test]
fn manual_clock_wall_read_matches_oracle() {
    let n: i128 = 1_717_000_000_000_000_000;
    let mut clock = ManualClock::default();
    clock.set_wall(WallInstant::from_nanos_since_epoch(n));
    let expected = clock
        .wall_now()
        .into_inner()
        .expect("ManualClock wall read is infallible")
        .as_nanos_since_epoch();
    let driver = format!(
        "fn main() => Binary{{128}} = wall_as_nanos_since_epoch(declared_entropy_into_inner(manual_wall_now(manual_set_wall(manual_clock_default(), wall_from_nanos_since_epoch({})))));",
        myc_i128_bin(n)
    );
    let expected_src = format!(
        "nodule ref;\nfn main() => Binary{{128}} = {};",
        myc_i128_bin(expected)
    );
    assert_three_way(
        "manual wall read == oracle",
        &program(&driver),
        &expected_src,
    );
}

/// The LOGICAL read reproduces the set tick — the oracle's `logical_now().into_inner()`.
#[test]
fn manual_clock_logical_read_matches_oracle() {
    let n: u64 = 42;
    let mut clock = ManualClock::default();
    clock.set_logical(LogicalInstant::from_tick(n));
    let expected = clock.logical_now().into_inner().as_tick();
    let driver = format!(
        "fn main() => Binary{{64}} = logical_as_tick(declared_time_into_inner(manual_logical_now(manual_set_logical(manual_clock_default(), logical_from_tick({})))));",
        myc_u64_bin(n)
    );
    let expected_src = format!(
        "nodule ref;\nfn main() => Binary{{64}} = {};",
        myc_u64_bin(expected)
    );
    assert_three_way(
        "manual logical read == oracle",
        &program(&driver),
        &expected_src,
    );
}

/// The DEFAULT clock reads zero on every source — the oracle's `ManualClock::default()` reads.
#[test]
fn manual_clock_default_reads_zero_matches_oracle() {
    let clock = ManualClock::default();
    let expected = clock
        .mono_now()
        .into_inner()
        .expect("default mono read")
        .as_nanos();
    let driver =
        "fn main() => Binary{64} = mono_as_nanos(declared_time_into_inner(manual_mono_now(manual_clock_default())));";
    let expected_src = format!(
        "nodule ref;\nfn main() => Binary{{64}} = {};",
        myc_u64_bin(expected)
    );
    assert_three_way(
        "manual default mono read == oracle (0)",
        &program(driver),
        &expected_src,
    );
}

// ── §E. Guarantee matrix — checked data driven against the LIVE Rust GUARANTEE_MATRIX (the crux) ─────

/// `matrix_len(matrix())` equals the live oracle's `GUARANTEE_MATRIX.len()` (not a hardcoded 11).
#[test]
fn matrix_len_matches_oracle_row_count() {
    let expected = u8::try_from(GUARANTEE_MATRIX.len()).expect("row count fits u8");
    let driver = "fn main() => Binary{8} = matrix_len(matrix());";
    assert_three_way(
        "matrix_len == oracle GUARANTEE_MATRIX.len()",
        &program(driver),
        &ref_count(expected),
    );
}

/// `count_exact(matrix())` equals the live oracle's count of `Exact`-tagged rows (the pure-arithmetic
/// invariant — a `Proven`/`Empirical` tag on any of these would itself violate VR-5).
#[test]
fn count_exact_matches_oracle() {
    let expected = u8::try_from(
        GUARANTEE_MATRIX
            .iter()
            .filter(|r| r.tag == GuaranteeStrength::Exact)
            .count(),
    )
    .expect("count fits u8");
    let driver = "fn main() => Binary{8} = count_exact(matrix());";
    assert_three_way(
        "count_exact == oracle",
        &program(driver),
        &ref_count(expected),
    );
}

/// `count_declared(matrix())` equals the live oracle's count of `Declared`-tagged rows (the clock-read
/// invariant — the crux of this module: clock reads are `Declared`, never dressed up as `Exact`).
#[test]
fn count_declared_matches_oracle() {
    let expected = u8::try_from(
        GUARANTEE_MATRIX
            .iter()
            .filter(|r| r.tag == GuaranteeStrength::Declared)
            .count(),
    )
    .expect("count fits u8");
    let driver = "fn main() => Binary{8} = count_declared(matrix());";
    assert_three_way(
        "count_declared == oracle",
        &program(driver),
        &ref_count(expected),
    );
}

/// `explainable_count(matrix())` equals the live oracle's count of EXPLAIN-able rows.
#[test]
fn explainable_count_matches_oracle() {
    let expected =
        u8::try_from(GUARANTEE_MATRIX.iter().filter(|r| r.explainable).count()).expect("fits u8");
    let driver = "fn main() => Binary{8} = explainable_count(matrix());";
    assert_three_way(
        "explainable_count == oracle",
        &program(driver),
        &ref_count(expected),
    );
}

/// `crux_holds()` — the honesty crux in one predicate: all three clock reads are `Declared` AND the
/// wall row's effect declaration differs from the mono row's (the typed `{ time, entropy }` vs `time`
/// distinction). Driven against the live oracle's own rows.
#[test]
fn crux_holds_matches_oracle() {
    let row = |op: &str| {
        GUARANTEE_MATRIX
            .iter()
            .find(|r| r.op == op)
            .unwrap_or_else(|| panic!("oracle has a {op} row"))
    };
    let wall = row("wall_now");
    let mono = row("mono_now");
    let logical = row("logical_now");
    let expected = wall.tag == GuaranteeStrength::Declared
        && mono.tag == GuaranteeStrength::Declared
        && logical.tag == GuaranteeStrength::Declared
        && wall.effects != mono.effects;
    let driver = "fn main() => Bool = crux_holds();";
    assert_three_way(
        "crux_holds == oracle",
        &program(driver),
        &ref_bool(expected),
    );
}

/// The `wall_now` row is the ONLY entropy-declaring row — the oracle's typed distinction, checked here
/// by the byte-length difference of the two effect declarations (`"{ time, entropy }"` vs `"time"`).
#[test]
fn entropy_distinction_matches_oracle() {
    let wall = GUARANTEE_MATRIX
        .iter()
        .find(|r| r.op == "wall_now")
        .expect("wall_now row");
    let mono = GUARANTEE_MATRIX
        .iter()
        .find(|r| r.op == "mono_now")
        .expect("mono_now row");
    // The oracle fact the .myc `entropy_distinction_holds` witnesses (via byte-length inequality).
    let expected = wall.effects.len() != mono.effects.len() && wall.effects.contains("entropy");
    let driver = "fn main() => Bool = entropy_distinction_holds();";
    assert_three_way(
        "entropy_distinction_holds == oracle",
        &program(driver),
        &ref_bool(expected),
    );
}
