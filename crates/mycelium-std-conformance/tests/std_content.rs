//! Differential tests for `std.content` (M-1022, kickoff `spw`, RFC-0031 D5/D6) — the `.myc` port
//! of `crates/mycelium-std-content/src/*.rs`, driven against the retained Rust oracle.
//!
//! # What this proves
//! Each case runs the shared three-way differential (L1-eval ≡ elaborate→L0-interp ≡ AOT, validated
//! by the M-210 checker) over a driver that exercises a ported op, AND compares the `.myc` result
//! against a value **computed live from the `mycelium_std_content` oracle** — never a hardcoded
//! literal. A real divergence between the Rust source and the `.myc` port flips the computed oracle
//! value and fails the corresponding case (the `std_core.rs` convention). Oracle agreement is what
//! earns `Empirical` (VR-5).
//!
//! # Scope (surface-check, D5 row 1 — see `lib/std/content.myc`'s module doc for the full writeup)
//! PORTED + differentiated here: `digest_eq`, `as_ref`/`ContentRef` accessors, `kind_prefix`/
//! `as_str_repr`, `ref_kind_from_prefix`, `MalformedDigest` + `display`, `parse_ref` +
//! `content_ref_from_str` (the hand-rolled shape scanners), the `guarantee_matrix::MATRIX` as
//! checked data + its structural checks, and the assoc-list `NameRegistry`.
//! NOT ported (FLAG-content-1, kernel structural-hash boundary — RFC-0031 D1): `hash_of_value` /
//! `hash_of_def`. They are excluded from this differential (their MATRIX rows are kept as pure data).
//!
//! # Honesty tags
//! - **`Exact`** — every op's own basis (total or explicit-`Option`/`Result`, deterministic,
//!   match/prim-defined; no selection/conversion/approximation — RFC-0016 C2).
//! - **`Declared`** — the 7-row MATRIX transcription (asserted data).
//! - **`Empirical`** — the three-way differential agreement + Rust-oracle comparison below.

mod harness;

use mycelium_std_content::guarantee_matrix::{Explainable, Fallibility, MATRIX};
use mycelium_std_content::{self as content, ContentHash, ContentRef, RefKind};
use std::str::FromStr;

/// The std.content nodule source, loaded at compile time — the single source of truth.
const CONTENT_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../lib/std/content.myc"
));

fn program(driver: &str) -> String {
    harness::program(CONTENT_SRC, driver)
}

fn assert_three_way(label: &str, src: &str, expected_src: &str) {
    harness::assert_three_way(label, src, expected_src);
}

/// Render a Rust `bool` as the `.myc` literal that denotes the same `Bool` value.
fn myc_bool(b: bool) -> &'static str {
    if b {
        "True"
    } else {
        "False"
    }
}

/// Render the `n`-deep `add_u(0b1, … 0b0 …)` chain the port's `Binary{8}` counters (`matrix_len`,
/// `fallible_count`, `name_registry_len`) expand to — the SAME primitive-op composition, so the
/// reference program carries the matching `Derived` provenance the `check_core` comparison requires
/// (the `std_core.rs` `myc_count_chain` precedent). An independent check of the count, not a literal.
fn myc_count_chain(n: u8) -> String {
    let mut expr = "0b0000_0000".to_owned();
    for _ in 0..n {
        expr = format!("add_u(0b0000_0001, {expr})");
    }
    expr
}

/// Escape a Rust `&str` into a `.myc` double-quoted `Bytes` literal (the lexer's minimal escape set
/// `\n \t \\ \" \0 \r`). Used to embed oracle-computed strings (addresses, repr forms, diagnostic
/// descriptions, the `Display` output — which itself carries `"` quotes) into drivers verbatim.
fn myc_str_lit(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\0' => out.push_str("\\0"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The `.myc` `RefKind` constructor name for a Rust `RefKind` (kept in lock-step with the port's
/// `type RefKind` and the oracle's enum).
fn myc_kind_ctor(k: RefKind) -> &'static str {
    match k {
        RefKind::Value => "Value",
        RefKind::Def => "Def",
        RefKind::Operation => "Operation",
        RefKind::Policy => "Policy",
        RefKind::Spore => "Spore",
        RefKind::Other => "Other",
    }
}

const ALL_KINDS: [RefKind; 6] = [
    RefKind::Value,
    RefKind::Def,
    RefKind::Operation,
    RefKind::Policy,
    RefKind::Spore,
    RefKind::Other,
];

// ── digest_eq — byte-equality of the two addresses (the M-912 bytes_eq prim closes the gap) ───────

/// `digest_eq(h, h)` is reflexive — driven against `mycelium_std_content::digest_eq`.
#[test]
fn digest_eq_reflexive_matches_oracle() {
    let a = ContentHash::parse("blake3:abc").expect("well-formed");
    let expected = content::digest_eq(&a, &a);
    let driver =
        "fn main() => Bool = digest_eq(ContentHash(\"blake3:abc\"), ContentHash(\"blake3:abc\"));";
    let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
    assert_three_way("digest_eq reflexive", &program(driver), &expected_src);
}

/// `digest_eq` distinguishes distinct digests — driven against the oracle.
#[test]
fn digest_eq_distinct_matches_oracle() {
    let a = ContentHash::parse("blake3:abc").expect("well-formed");
    let b = ContentHash::parse("blake3:xyz").expect("well-formed");
    let expected = content::digest_eq(&a, &b);
    let driver =
        "fn main() => Bool = digest_eq(ContentHash(\"blake3:abc\"), ContentHash(\"blake3:xyz\"));";
    let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
    assert_three_way("digest_eq distinct", &program(driver), &expected_src);
}

// ── as_ref / ContentRef accessors ─────────────────────────────────────────────────────────────────

/// `content_ref_kind(as_ref(h, k))` round-trips the kind for every `RefKind` — oracle: `r.kind()`.
#[test]
fn as_ref_kind_preserved_matches_oracle() {
    let h = ContentHash::parse("blake3:abc").expect("well-formed");
    for k in ALL_KINDS {
        let r = content::as_ref(h.clone(), k);
        let expected = r.kind() == k;
        let ctor = myc_kind_ctor(k);
        let driver = format!(
            "fn main() => Bool = match content_ref_kind(as_ref(ContentHash(\"blake3:abc\"), {ctor})) {{ {ctor} => True, _ => False }};"
        );
        let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
        assert_three_way(
            &format!("as_ref kind {ctor}"),
            &program(&driver),
            &expected_src,
        );
    }
}

/// `content_ref_hash(as_ref(h, Def))` round-trips the hash — oracle: `r.hash() == &h`.
#[test]
fn as_ref_hash_preserved_matches_oracle() {
    let h = ContentHash::parse("blake3:abc").expect("well-formed");
    let r = content::as_ref(h.clone(), RefKind::Def);
    let expected = r.hash() == &h;
    let driver = "fn main() => Bool = digest_eq(content_ref_hash(as_ref(ContentHash(\"blake3:abc\"), Def)), ContentHash(\"blake3:abc\"));";
    let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
    assert_three_way("as_ref hash preserved", &program(driver), &expected_src);
}

/// `content_ref_as_str_repr` (hence `kind_prefix`) matches `ContentRef::as_str_repr()` for every
/// kind — the machine-side G11 projection. The oracle produces the expected repr string; the `.myc`
/// port computes its own; `bytes_eq` compares them.
#[test]
fn as_str_repr_matches_oracle_for_every_kind() {
    let addr = "blake3:abc";
    let h = ContentHash::parse(addr).expect("well-formed");
    for k in ALL_KINDS {
        let r = content::as_ref(h.clone(), k);
        let repr = r.as_str_repr();
        let ctor = myc_kind_ctor(k);
        let driver = format!(
            "fn main() => Bool = match bytes_eq(content_ref_as_str_repr(as_ref(ContentHash({}), {ctor})), {}) {{ 0b1 => True, _ => False }};",
            myc_str_lit(addr),
            myc_str_lit(&repr)
        );
        let expected_src = "nodule ref;\nfn main() => Bool = True;";
        assert_three_way(
            &format!("as_str_repr {ctor}"),
            &program(&driver),
            expected_src,
        );
    }
}

// ── parse_ref — the <algo>:<digest> shape scanner (hand-rolled recursive byte scan) ───────────────

/// `parse_ref` accept/reject matches the oracle over the accept cases + the 5 malformed shapes from
/// `lib.rs`'s own test module.
#[test]
fn parse_ref_accept_reject_matches_oracle() {
    let inputs = [
        "blake3:Hh3kQ_x-1A",
        "sha256:abcdef0123456789",
        "no-colon",
        "blake3:",
        ":digest",
        "UPPER:abc",
        "blake3:has space",
    ];
    for s in inputs {
        let expected = content::parse_ref(s).is_ok();
        let driver = format!("fn main() => Bool = parse_ref_ok({});", myc_str_lit(s));
        let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
        assert_three_way(
            &format!("parse_ref_ok {s:?}"),
            &program(&driver),
            &expected_src,
        );
    }
}

/// `parse_ref`'s diagnostic `description` matches the oracle byte-for-byte over each malformed shape
/// — the strong differential on the hand-ported 5-way diagnostic branching.
#[test]
fn parse_ref_description_matches_oracle() {
    let malformed = [
        "no-colon",
        "blake3:",
        ":digest",
        "UPPER:abc",
        "blake3:has space",
    ];
    for s in malformed {
        let err = content::parse_ref(s).expect_err("must reject");
        let desc = err.description;
        let driver = format!(
            "fn main() => Bool = match bytes_eq(parse_ref_description({}), {}) {{ 0b1 => True, _ => False }};",
            myc_str_lit(s),
            myc_str_lit(desc)
        );
        let expected_src = "nodule ref;\nfn main() => Bool = True;";
        assert_three_way(
            &format!("parse_ref_description {s:?}"),
            &program(&driver),
            expected_src,
        );
    }
}

/// A well-formed address round-trips: `parse_ref(s)` yields a hash whose address equals `s` — oracle:
/// `parse_ref(s).unwrap().as_str() == s`.
#[test]
fn parse_ref_roundtrip_matches_oracle() {
    let s = "blake3:Hh3kQ_x-1A";
    let h = content::parse_ref(s).expect("well-formed");
    let expected = h.as_str() == s;
    let driver = format!(
        "fn main() => Bool = match parse_ref({s}) {{ Ok(h) => digest_eq(h, ContentHash({s})), Err(_) => False }};",
        s = myc_str_lit(s)
    );
    let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
    assert_three_way("parse_ref round-trip", &program(&driver), &expected_src);
}

// ── content_ref_from_str — the <kind-prefix>+<algo>:<digest> parser ───────────────────────────────

/// A canonical string round-trips through `content_ref_from_str` back to itself via `as_str_repr` —
/// oracle: `ContentRef::from_str(s).unwrap().as_str_repr() == s`.
#[test]
fn content_ref_from_str_roundtrip_matches_oracle() {
    let s = "def+blake3:abc";
    let r = ContentRef::from_str(s).expect("well-formed");
    let expected = r.as_str_repr() == s;
    let driver = format!(
        "fn main() => Bool = match content_ref_from_str({s}) {{ Ok(r) => match bytes_eq(content_ref_as_str_repr(r), {s}) {{ 0b1 => True, _ => False }}, Err(_) => False }};",
        s = myc_str_lit(s)
    );
    let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
    assert_three_way(
        "content_ref_from_str round-trip",
        &program(&driver),
        &expected_src,
    );
}

/// `content_ref_from_str` accept/reject matches the oracle over an accept + the 3 malformed shapes
/// from `content_ref.rs`'s own test module (missing `+`, unknown prefix, malformed tail).
#[test]
fn content_ref_from_str_accept_reject_matches_oracle() {
    let inputs = [
        "def+blake3:abc",
        "nokind",
        "bogus+blake3:abc",
        "value+nocolon",
    ];
    for s in inputs {
        let expected = ContentRef::from_str(s).is_ok();
        let driver = format!(
            "fn main() => Bool = content_ref_from_str_ok({});",
            myc_str_lit(s)
        );
        let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
        assert_three_way(
            &format!("content_ref_from_str_ok {s:?}"),
            &program(&driver),
            &expected_src,
        );
    }
}

// ── MalformedDigest::Display ──────────────────────────────────────────────────────────────────────

/// `malformed_digest_display` reproduces the Rust `Display` impl byte-for-byte (incl. the `{:?}`
/// quotes around the input) — oracle: `parse_ref("no-colon").unwrap_err().to_string()`.
#[test]
fn malformed_digest_display_matches_oracle() {
    let err = content::parse_ref("no-colon").expect_err("must reject");
    let disp = err.to_string();
    let driver = format!(
        "fn main() => Bool = match parse_ref(\"no-colon\") {{ Err(e) => match bytes_eq(malformed_digest_display(e), {}) {{ 0b1 => True, _ => False }}, Ok(_) => False }};",
        myc_str_lit(&disp)
    );
    let expected_src = "nodule ref;\nfn main() => Bool = True;";
    assert_three_way("malformed_digest_display", &program(&driver), expected_src);
}

// ── guarantee_matrix — the RFC-0016 §4.5 matrix as checked data (driven against the live oracle) ──

/// `matrix_len(matrix())` equals the live Rust `MATRIX.len()` (7).
#[test]
fn matrix_len_matches_oracle() {
    let n = u8::try_from(MATRIX.len()).expect("fits u8");
    let driver = "fn main() => Binary{8} = matrix_len(matrix());";
    let expected_src = format!(
        "nodule ref;\nfn main() => Binary{{8}} = {};",
        myc_count_chain(n)
    );
    assert_three_way(
        "matrix_len == MATRIX.len()",
        &program(driver),
        &expected_src,
    );
}

/// Every MATRIX row's guarantee is `"Exact"` — driven against the live oracle.
#[test]
fn all_guarantee_exact_matches_oracle() {
    let expected = MATRIX.iter().all(|r| r.guarantee == "Exact");
    let driver = "fn main() => Bool = all_guarantee_exact(matrix());";
    let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
    assert_three_way("all_guarantee_exact", &program(driver), &expected_src);
}

/// Every MATRIX row is effect-free (`"none"`) — driven against the live oracle.
#[test]
fn all_effects_none_matches_oracle() {
    let expected = MATRIX.iter().all(|r| r.effects == "none");
    let driver = "fn main() => Bool = all_effects_none(matrix());";
    let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
    assert_three_way("all_effects_none", &program(driver), &expected_src);
}

/// Exactly two ops are `Fallible` (`parse_ref`, `resolve_name`) — driven against the live oracle.
#[test]
fn fallible_count_matches_oracle() {
    let n = u8::try_from(
        MATRIX
            .iter()
            .filter(|r| matches!(r.fallibility, Fallibility::Fallible))
            .count(),
    )
    .expect("fits u8");
    let driver = "fn main() => Binary{8} = fallible_count(matrix());";
    let expected_src = format!(
        "nodule ref;\nfn main() => Binary{{8}} = {};",
        myc_count_chain(n)
    );
    assert_three_way("fallible_count == 2", &program(driver), &expected_src);
}

/// Every MATRIX row is `NotApplicable` for EXPLAIN — driven against the live oracle.
#[test]
fn all_explain_na_matches_oracle() {
    let expected = MATRIX
        .iter()
        .all(|r| r.explainable == Explainable::NotApplicable);
    let driver = "fn main() => Bool = all_explain_na(matrix());";
    let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
    assert_three_way("all_explain_na", &program(driver), &expected_src);
}

// ── NameRegistry — the assoc-list redesign (FLAG-content-2), read/write surface ───────────────────

/// `resolve_name` on an empty registry is `None` — oracle: `NameRegistry::new().resolve_name(&h)`.
#[test]
fn resolve_name_unbound_matches_oracle() {
    let h = ContentHash::parse("blake3:abc").expect("well-formed");
    let reg = content::NameRegistry::new();
    let expected = reg.resolve_name(&h).is_none();
    let driver = "fn main() => Bool = match name_registry_resolve_name(name_registry_new(), ContentHash(\"blake3:abc\")) { None => True, Some(_) => False };";
    let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
    assert_three_way(
        "resolve_name unbound == None",
        &program(driver),
        &expected_src,
    );
}

/// Binding then resolving round-trips the name — oracle: `reg.bind(h, "my_const");
/// reg.resolve_name(&h) == Some("my_const")`.
#[test]
fn resolve_name_after_bind_matches_oracle() {
    let h = ContentHash::parse("blake3:abc").expect("well-formed");
    let mut reg = content::NameRegistry::new();
    reg.bind(h.clone(), "my_const");
    let expected = reg.resolve_name(&h) == Some("my_const");
    let driver = "fn main() => Bool = match name_registry_resolve_name(name_registry_insert(name_registry_new(), ContentHash(\"blake3:abc\"), \"my_const\"), ContentHash(\"blake3:abc\")) { Some(n) => match bytes_eq(n, \"my_const\") { 0b1 => True, _ => False }, None => False };";
    let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
    assert_three_way("resolve_name after bind", &program(driver), &expected_src);
}

/// `names_of` after a bind is a 1-element list carrying the name (the one-name limitation) —
/// oracle: `reg.names_of(&h) == vec!["my_const"]`.
#[test]
fn names_of_after_bind_matches_oracle() {
    let h = ContentHash::parse("blake3:abc").expect("well-formed");
    let mut reg = content::NameRegistry::new();
    reg.bind(h.clone(), "my_const");
    let expected = reg.names_of(&h) == vec!["my_const".to_owned()];
    let driver = "fn main() => Bool = match name_registry_names_of(name_registry_insert(name_registry_new(), ContentHash(\"blake3:abc\"), \"my_const\"), ContentHash(\"blake3:abc\")) { Cons(n, Nil) => match bytes_eq(n, \"my_const\") { 0b1 => True, _ => False }, _ => False };";
    let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
    assert_three_way("names_of after bind", &program(driver), &expected_src);
}

/// `names_of` on an unbound hash is empty — oracle: `NameRegistry::new().names_of(&h).is_empty()`.
#[test]
fn names_of_unbound_matches_oracle() {
    let h = ContentHash::parse("blake3:abc").expect("well-formed");
    let reg = content::NameRegistry::new();
    let expected = reg.names_of(&h).is_empty();
    let driver = "fn main() => Bool = match name_registry_names_of(name_registry_new(), ContentHash(\"blake3:abc\")) { Nil => True, Cons(_, _) => False };";
    let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
    assert_three_way("names_of unbound == empty", &program(driver), &expected_src);
}

/// The first bind of a hash returns `None` (no previous name) — oracle: `reg.bind(h, "first")`.
#[test]
fn bind_first_returns_none_matches_oracle() {
    let h = ContentHash::parse("blake3:abc").expect("well-formed");
    let mut reg = content::NameRegistry::new();
    let expected = reg.bind(h, "first").is_none();
    let driver = "fn main() => Bool = match name_registry_bind_prev(name_registry_new(), ContentHash(\"blake3:abc\"), \"first\") { None => True, Some(_) => False };";
    let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
    assert_three_way("bind first returns None", &program(driver), &expected_src);
}

/// Re-binding a hash returns the previous name (identity unchanged — ADR-003) — oracle:
/// `reg.bind(h, "first"); reg.bind(h, "second") == Some("first")`.
#[test]
fn rebind_returns_previous_matches_oracle() {
    let h = ContentHash::parse("blake3:abc").expect("well-formed");
    let mut reg = content::NameRegistry::new();
    reg.bind(h.clone(), "first");
    let expected = reg.bind(h, "second") == Some("first".to_owned());
    let driver = "fn main() => Bool = match name_registry_bind_prev(name_registry_insert(name_registry_new(), ContentHash(\"blake3:abc\"), \"first\"), ContentHash(\"blake3:abc\"), \"second\") { Some(n) => match bytes_eq(n, \"first\") { 0b1 => True, _ => False }, None => False };";
    let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
    assert_three_way("rebind returns previous", &program(driver), &expected_src);
}

/// `len` counts distinct bindings; a fresh registry `is_empty` — oracle: `reg.len()` after two
/// distinct-hash binds is 2.
#[test]
fn name_registry_len_matches_oracle() {
    let h1 = ContentHash::parse("blake3:aaa").expect("well-formed");
    let h2 = ContentHash::parse("blake3:bbb").expect("well-formed");
    let mut reg = content::NameRegistry::new();
    reg.bind(h1, "alpha");
    reg.bind(h2, "beta");
    let n = u8::try_from(reg.len()).expect("fits u8");
    let driver = "fn main() => Binary{8} = name_registry_len(name_registry_insert(name_registry_insert(name_registry_new(), ContentHash(\"blake3:aaa\"), \"alpha\"), ContentHash(\"blake3:bbb\"), \"beta\"));";
    let expected_src = format!(
        "nodule ref;\nfn main() => Binary{{8}} = {};",
        myc_count_chain(n)
    );
    assert_three_way("name_registry_len == 2", &program(driver), &expected_src);
}

/// A fresh registry is empty — oracle: `NameRegistry::new().is_empty()`.
#[test]
fn name_registry_is_empty_matches_oracle() {
    let reg = content::NameRegistry::new();
    let expected = reg.is_empty();
    let driver = "fn main() => Bool = name_registry_is_empty(name_registry_new());";
    let expected_src = format!("nodule ref;\nfn main() => Bool = {};", myc_bool(expected));
    assert_three_way("name_registry is_empty", &program(driver), &expected_src);
}
