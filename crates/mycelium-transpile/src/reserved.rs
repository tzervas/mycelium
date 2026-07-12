//! Mycelium reserved-word snapshot + the identifier collision guard (M-1001).
//!
//! A Rust identifier that is a Mycelium **reserved word**, emitted verbatim into
//! constructor/variant/pattern/type/fn position, fails to **parse** — the lexer tokenizes it as a
//! keyword, not an `Ident` (observed by the M-1000 vet loop as
//! `parse-error: expected a pattern, found Strength(Exact)` on `mycelium-l1/src/eval.rs`, and
//! `expected an identifier, found Binary` on `checkty.rs`). That is a "plausible but wrong" emission
//! the DN-34 §4/§8 flag-don't-guess principle forbids. The transpiler has **no sanctioned renaming
//! scheme** — the self-hosted port's per-type ctor prefixing (`lib/compiler/README.md`
//! FLAG-ast-5/FLAG-parse-2) is a *human* decision, not a mechanical one — so a collision is
//! **gapped** ([`crate::gap::Category::ReservedWord`]), never silently emitted or auto-renamed
//! (G2/VR-5).
//!
//! # Guarantee: `Declared`
//!
//! [`RESERVED`] is a verbatim **snapshot** of `mycelium-l1`'s lexer keyword table
//! (`crates/mycelium-l1/src/token.rs` `fn keyword`) as of **2026-07-12** (refreshed from the
//! 2026-07-06 snapshot — gap-close-2 Phase-0 re-measure — to add `priv`/`wrapping`, landed on the
//! real lexer since the prior snapshot date but missed here; see `src/tests/reserved.rs`'s
//! `snapshot_words_are_all_still_reserved_in_the_lexer` for the drift guard this refresh keeps
//! green), copied row-for-row. It is `Declared`, not authoritative — the lexer is ground truth. A
//! drift test (`src/tests/reserved.rs`, a dev-dependency on `mycelium-l1`) asserts every word here
//! is still rejected by the real `mycelium_l1::token::keyword`, so a snapshot that drifts to a
//! *non*-reserved word is caught (the **over-gap** direction — the one that would regress a valid
//! emission). The **under-gap** direction — a *new* keyword `l1` adds that this list misses — is a
//! residual the vet loop catches as a parse error, never a silent bad emission (this crate has no
//! `mycelium-l1` runtime dependency to introspect its keyword table programmatically — only a
//! dev-dependency for the drift test — so an exhaustive cross-check isn't wired here; when this
//! snapshot is touched again, diff it by eye against `crates/mycelium-l1/src/token.rs`'s `fn
//! keyword`, the cited source of truth).

use crate::gap::{Category, GapReason};

/// The Mycelium reserved-word set — a verbatim snapshot of `mycelium-l1`'s `token::keyword` table
/// (2026-07-12). Grouped as in the source: active keywords, reserved-not-active runtime/surface
/// terms, the repr-type keywords, the scalar-float keywords, and the guarantee-strength keywords.
pub const RESERVED: &[&str] = &[
    // Active + reserved-not-active structural/surface keywords.
    "nodule",
    "phylum",
    "colony",
    "hypha",
    "fuse",
    "mesh",
    "graft",
    "cyst",
    "xloc",
    "forage",
    "backbone",
    "tier",
    "reclaim",
    "consume",
    "grow",
    "lambda",
    "object",
    "via",
    "lower",
    "derive",
    "use",
    "pub",
    // `priv` — the M-1027/DN-104 per-constructor seal marker (missed by the 2026-07-06 snapshot;
    // added in this 2026-07-12 refresh — mycelium-l1/src/token.rs `fn keyword`).
    "priv",
    "type",
    "trait",
    "impl",
    "fn",
    "matured",
    "thaw",
    "let",
    "in",
    "if",
    "then",
    "else",
    "match",
    "for",
    "swap",
    "default",
    "paradigm",
    "with",
    "wild",
    "spore",
    // `wrapping` — RFC-0034 §10/§10.1 (CU-5) named modular-arithmetic opt-out (missed by the
    // 2026-07-06 snapshot; added in this 2026-07-12 refresh).
    "wrapping",
    "to",
    "policy",
    // Repr-type keywords + their RFC-0037 short aliases.
    "Binary",
    "Ternary",
    "Dense",
    "VSA",
    "bin",
    "tern",
    "emb",
    "hvec",
    "Seq",
    "Bytes",
    "Float",
    "Substrate",
    "Sparse",
    // Scalar-float keywords.
    "F16",
    "BF16",
    "F32",
    "F64",
    // Guarantee-strength keywords.
    "Exact",
    "Proven",
    "Empirical",
    "Declared",
];

/// Whether `word` is a Mycelium reserved word (would not lex as an `Ident`).
pub fn is_reserved(word: &str) -> bool {
    RESERVED.contains(&word)
}

/// Guard an identifier the emitter is about to place into `.myc` surface text. `Ok(())` when it is a
/// legal identifier; `Err(GapReason)` (category [`Category::ReservedWord`]) when it collides with a
/// reserved word — so the caller gaps the construct rather than emit un-parseable text. `context`
/// names the position (e.g. `"enum variant"`, `"match pattern"`, `"type name"`) for the diagnostic.
pub fn guard_ident(name: &str, context: &str) -> Result<(), GapReason> {
    if is_reserved(name) {
        Err(GapReason::new(
            Category::ReservedWord,
            format!(
                "{context} `{name}` collides with a Mycelium reserved word — emitting it verbatim \
                 would fail to parse (the lexer tokenizes it as a keyword, not an identifier); no \
                 sanctioned auto-rename in this PoC, so flagged rather than emitted (G2/VR-5)"
            ),
        ))
    } else {
        Ok(())
    }
}

/// **Gap-close-2 Phase-0 regression fix, revised (PR #1517 review HIGH — cross-file nodule-path
/// collision).** Sanitize a derived nodule path (`transpile::derive_nodule_path`, M-1042) against
/// [`RESERVED`]. M-1042 extended nodule-path derivation to include a file's **intra-crate module
/// path** as dotted segments (`crates/mycelium-l1/src/fuse.rs` -> `l1.fuse`,
/// `crates/mycelium-std-runtime/src/colony.rs` -> `std.runtime.colony`) — but a segment that is
/// itself a reserved word (`fuse`, `colony`, …) was never run through [`guard_ident`], so it
/// leaked verbatim into the `.myc` header (`nodule l1.fuse;`), which the real lexer tokenizes as a
/// **keyword**, not a path identifier — a hard `myc check` **parse error** ("expected an
/// identifier, found Fuse"), not a clean gap. That regressed the `checked_fraction`'s G2 "zero
/// hard parse failures" invariant for every file whose file/dir-name happens to be a reserved
/// word.
///
/// The original fix (2026-07-11) **dropped** the colliding segment. That reintroduced a *silent*
/// collision one level up: `crates/mycelium-l1/src/fuse.rs` (`l1.fuse`) and `crates/mycelium-l1/
/// src/nodule.rs` (`l1.nodule`) both drop their sole reserved segment and sanitize to the
/// identical `l1` nodule path — two distinct source files emitting the same `nodule l1;` header.
/// Each file myc-checks "Clean" individually, so the per-file vet loop cannot see the collision
/// (G2: an undisclosed possible-collision is exactly the "no black boxes" rule exists to prevent).
///
/// The fix here instead **escapes** each colliding segment in place — `word` becomes
/// `word{RESERVED_SEGMENT_SUFFIX}` (`fuse` -> `fuse_kw`) — rather than deleting it. This is
/// **collision-free among the reserved words themselves, by construction**: the suffix is a
/// constant appended verbatim, so the map `word -> word + RESERVED_SEGMENT_SUFFIX` is injective
/// (two different reserved words can never produce the same escaped segment), and no entry in
/// [`RESERVED`] ends in `RESERVED_SEGMENT_SUFFIX` (checked by
/// `escaped_reserved_words_are_never_themselves_reserved` in `src/tests/reserved.rs`), so an
/// escaped segment can never re-trigger the very guard it exists to satisfy. Every other segment
/// (the non-colliding crate/module prefix) is passed through unchanged, so `l1.fuse` -> `l1.fuse_kw`
/// and `l1.nodule` -> `l1.nodule_kw` are now distinct.
///
/// **Documented residual (`Declared`, not `Proven` — VR-5):** because both Rust and Mycelium
/// identifiers are ASCII-only (`is_ident_continue` in `mycelium-l1/src/lexer.rs` — no
/// non-ASCII-marker escape is available to either language), this is not a mathematical proof of
/// global uniqueness against *every* possible source tree — a hypothetical sibling file literally
/// named `fuse_kw.rs` alongside `fuse.rs` in the same directory would still collide post-escape.
/// That shape is not present in this corpus (checked against `crates/mycelium-l1/src/token.rs`'s
/// keyword list at the 2026-07-12 snapshot) and is vanishingly unlikely by convention (`_kw` is not
/// a real Rust module-naming pattern in this codebase); it remains a residual, not a silent one —
/// the gap reason below always names the exact original path and the exact escaped segment(s), so
/// a future occurrence is diagnosable, not invisible.
///
/// A nodule-path segment is transpiler-derived file-layout metadata, not a *program* identifier —
/// unlike [`guard_ident`]'s callers (which gap rather than guess a rename for a real symbol, since
/// there is no sanctioned auto-rename for program surface), escaping file-layout metadata with a
/// fixed, disclosed marker is a deterministic, EXPLAIN-traceable transform, not a guess. Returns
/// `(sanitized_path, Some(reason))` when a segment collided, or `(nodule_path.to_owned(), None)`
/// unchanged when it did not.
pub const RESERVED_SEGMENT_SUFFIX: &str = "_kw";

pub fn sanitize_nodule_path(nodule_path: &str) -> (String, Option<GapReason>) {
    let segments: Vec<&str> = nodule_path.split('.').collect();
    let colliding: Vec<&str> = segments
        .iter()
        .copied()
        .filter(|s| is_reserved(s))
        .collect();
    if colliding.is_empty() {
        return (nodule_path.to_string(), None);
    }
    let escaped: Vec<String> = segments
        .into_iter()
        .map(|s| {
            if is_reserved(s) {
                format!("{s}{RESERVED_SEGMENT_SUFFIX}")
            } else {
                s.to_string()
            }
        })
        .collect();
    let sanitized = escaped.join(".");
    let reason = GapReason::new(
        Category::ReservedWord,
        format!(
            "derived nodule path `{nodule_path}` has segment(s) [{}] colliding with a Mycelium \
             reserved word — emitting them verbatim would fail to parse (`nodule {nodule_path};` \
             tokenizes the colliding word as a keyword, not an identifier); each colliding \
             segment is escaped with the `{RESERVED_SEGMENT_SUFFIX}` suffix (now `{sanitized}`) \
             rather than dropped, so distinct source files whose only differing segment is a \
             reserved word (e.g. `l1.fuse` vs `l1.nodule`) stay distinguishable instead of both \
             collapsing onto the same nodule path (G2/VR-5)",
            colliding.join(", ")
        ),
    );
    (sanitized, Some(reason))
}
