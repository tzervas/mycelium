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

/// **Gap-close-2 Phase-0 regression fix.** Sanitize a derived nodule path (`transpile::
/// derive_nodule_path`, M-1042) against [`RESERVED`]. M-1042 extended nodule-path derivation to
/// include a file's **intra-crate module path** as dotted segments (`crates/mycelium-l1/src/
/// fuse.rs` -> `l1.fuse`, `crates/mycelium-std-runtime/src/colony.rs` -> `std.runtime.colony`) —
/// but a segment that is itself a reserved word (`fuse`, `colony`, …) was never run through
/// [`guard_ident`], so it leaked verbatim into the `.myc` header (`nodule l1.fuse;`), which the
/// real lexer tokenizes as a **keyword**, not a path identifier — a hard `myc check` **parse
/// error** ("expected an identifier, found Fuse"), not a clean gap. That regressed the
/// `checked_fraction`'s G2 "zero hard parse failures" invariant for every file whose file/dir-name
/// happens to be a reserved word.
///
/// A nodule-path segment is transpiler-derived file-layout metadata, not a *program* identifier —
/// unlike [`guard_ident`]'s callers (which gap rather than guess a rename for a real symbol, since
/// there is no sanctioned auto-rename for program surface), the safe, deterministic,
/// EXPLAIN-traceable choice here is to **drop** the colliding segment(s) (falling back to the
/// un-suffixed crate-prefix — historically what every file emitted before M-1042 added the extra
/// segment — if every segment collides) while **always recording** a [`Category::ReservedWord`]
/// [`GapReason`] naming exactly what collided (never silent, G2/VR-5). Returns
/// `(sanitized_path, Some(reason))` when a segment collided, or `(nodule_path.to_owned(), None)`
/// unchanged when it did not.
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
    let kept: Vec<&str> = segments.into_iter().filter(|s| !is_reserved(s)).collect();
    let sanitized = if kept.is_empty() {
        // Pathological: every segment collided (never observed in the corpus — crate prefixes are
        // not reserved words). Fall back to a fixed, still-legal placeholder rather than emit an
        // empty/invalid nodule path (never a silent panic — G2).
        "unknown".to_string()
    } else {
        kept.join(".")
    };
    let reason = GapReason::new(
        Category::ReservedWord,
        format!(
            "derived nodule path `{nodule_path}` has segment(s) [{}] colliding with a Mycelium \
             reserved word — emitting them verbatim would fail to parse (`nodule {nodule_path};` \
             tokenizes the colliding word as a keyword, not an identifier); the colliding \
             segment(s) are dropped from the emitted nodule path (now `{sanitized}`) rather than \
             guessed at a rename (G2/VR-5)",
            colliding.join(", ")
        ),
    );
    (sanitized, Some(reason))
}
