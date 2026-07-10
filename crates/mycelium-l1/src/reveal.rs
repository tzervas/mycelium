//! `reveal` — desugar-on-demand, Increment-1 (**M-1051**; DN-38 §5/§8.3; DN-110 §3.4/§8.4;
//! DN-110-8.2-hygiene-deepdive §5/§7 E3/§10 OQ-H3).
//!
//! `reveal` is the transparency inspector house rule #2 requires: it shows the **real L0 term the
//! kernel runs**, never a lossy text reconstruction (DN-38 §5). This module ships the
//! **E3-enabling core** — the four primitives a future E1/E3 hygiene-experiment harness (M-1055) or
//! `reveal` CLI / `certified`-mode round-trip check composes: [`reveal_l0`] (the shown L0 term),
//! [`render_surface`] (a best-effort, honestly-labelled surface pretty-printer), [`alpha_eq`] (the
//! structural alpha-equivalence [`Node`] needs but its derived-looking [`PartialEq`] does not
//! provide), and [`reelaborate`] (the closedness-re-derivation primitive the L0-level round-trip
//! witness composes from — **not yet wired to a genuine E3 regression corpus**; see the "honest
//! scoping" callout below).
//!
//! # Scope (Increment-1 — do not read more into this module than it claims)
//!
//! - **`site` is an entry symbol**, exactly like [`crate::elab::elaborate`]'s `entry: &str` — no
//!   source-span resolver (Increment-2).
//! - **No CLI** (Increment-2) and **no `certified`-mode round-trip *check*** wired into the checker
//!   (Increment-3, DN-38 §5's `delaborate ∘ lower = id` obligation *gated by* `certified`); this
//!   module only supplies the primitives those increments compose.
//! - **No `Node`/`Expr` type-definition changes** — this module reads the existing frozen grammar
//!   ([`mycelium_core::node`]) and the existing [`crate::elab::elaborate`] entry points; it adds no
//!   kernel surface (KC-3).
//!
//! # PINNED RULINGS (do not re-litigate here — see the cited sections for the deliberation)
//!
//! - **DN-38 §8.3 (v0 fidelity).** v0 is the **true L0-term view**: [`reveal_l0`] returns the
//!   literal [`Node`] [`crate::elab::elaborate`] produces — not a re-rendered/reconstructed
//!   approximation of it. The `certified` round-trip *check* (translation-validation over this view)
//!   is a later increment, out of scope here.
//! - **DN-110-8.2-hygiene-deepdive §10 OQ-H3 (surface fidelity for `%`-names).** Option **(a)**:
//!   the surface rendering ([`render_surface`]) declares `%`-freshened hygienic names
//!   **out-of-contract** — it shows them **raw** (never munged/hidden — house rule #2) and
//!   **honestly labels** the rendering non-re-parseable ([`Rendered::reparseable`]), rather than
//!   building a display-renaming pass. **The identity witness for the round-trip property is at the
//!   L0-*term* level ([`alpha_eq`] over [`reveal_l0`]'s output), never a surface re-parse** — this is
//!   the deep-dive §5 resolution the E3 experiment (§7) exists to force into the open, applied here.
//!   The same non-reparseable labelling is extended (consistently, not as a new ruling) to two other
//!   surface-token gaps the L0 grammar exposes that OQ-H3 did not separately enumerate: a
//!   [`Node::Swap`]'s `policy` (a resolved [`mycelium_core::ContentHash`], no surface `Path` spelling
//!   survives elaboration to invert it) and a [`Node::Construct`]/[`mycelium_core::Alt::Ctor`]'s
//!   `ctor` (a resolved [`mycelium_core::CtorRef`] `#<hash>#<i>`, likewise nameless at L0 — ADR-003).
//!   Both render via their own canonical `Display` (`CtorRef`'s Unison spelling) or an explicit
//!   `#`-prefixed marker, and both trip the same reparseable=false flag as a `%`-name (see
//!   [`render_surface`]'s doc for the exact marker scan).
//!
//! # Guarantee tags (VR-5 — no upgrade past what is checked here)
//!
//! - [`reveal_l0`] exposing the real, unmodified [`Node`] `elaborate` produced: **`Exact`** — it is
//!   the identity function composed with `elaborate` (no lossy step introduced), so "shows the real
//!   L0 term" is definitionally true at v0, not merely tested.
//! - [`alpha_eq`]/[`render_surface`]/[`reelaborate`] as *implementations of the properties they
//!   claim* (alpha-correctness, honest-labelling, closedness-preservation): **`Empirical`** — checked
//!   by the fixture-table + property test in `src/tests/reveal.rs`, **not** `Proven` (no mechanized
//!   proof of alpha-comparison correctness or of the pretty-printer's grammar coverage claim).
//! - The **closedness-preservation** property tested here (`src/tests/reveal.rs`'s
//!   `reveal_l0_output_is_closed_and_survives_reelaboration`),
//!   `alpha_eq(reelaborate(reveal_l0(x)), reveal_l0(x))` over the fixture corpus: **`Empirical`**,
//!   but see the honest-scoping callout immediately below — **this is not yet the DN-110-8.2-hygiene-
//!   deepdive §7 E3 regression test**, and no claim here should be read as E3 evidence.
//!
//! # `reelaborate` at v0 — what it actually checks, honestly
//!
//! Because v0's [`reveal_l0`] is a *direct* view of the already-elaborated [`Node`] (the pinned
//! DN-38 §8.3 ruling above — no lossy rendering step in between), there is no surface
//! print → re-lex → re-parse → re-check pipeline to invert (that pipeline is also independently
//! broken for `%`-names by OQ-H3's own finding — see [`render_surface`]). So Increment-1's
//! `reelaborate` is **not** "re-run the compiler front end on printed text"; it is a genuine
//! **re-derivation** of the term's own closedness witness — the invariant `elaborate` itself
//! establishes (a closed L0 program has no free [`Node::Var`] reference) — by an independent
//! structural walk over the shown [`Node`] alone (no access to the original `Env`/source). It
//! **recomputes**, rather than merely returns, that witness: [`reelaborate`] returns a clone of the
//! input **iff** the closedness re-derivation succeeds, and a never-silent [`RevealError::NotClosed`]
//! naming the offending free names otherwise (G2). This gives the closedness property real teeth (a
//! `reveal_l0` that ever leaked an unbound reference would fail it) while staying honest about not
//! being a full re-elaboration-from-surface — that stronger obligation is DN-38 §5's `certified`
//! round-trip, Increment-3.
//!
//! **Honest scoping (VR-5 — do not overclaim this as E3): the corpus round-trip test is NOT yet
//! the DN-110-8.2-hygiene-deepdive §7 E3 regression test, and must not be read/cited as such.**
//! [`reelaborate`] returns `shown.clone()` on the success path (see above — v0 has no lossy step to
//! invert, so re-derivation is a validated clone). Composed with `alpha_eq`, the corpus test therefore
//! compares a term to a **bit-identical clone of itself** — it would pass even with a broken
//! `alpha_eq` (a `false`-always or `true`-always comparator both happen to agree on identical
//! operands via different code paths; only a genuinely *differently-spelled-but-equivalent* pair
//! distinguishes a correct alpha-comparator from a broken one). What it *does* check, and check
//! validly, is **closedness-preservation**: that `reveal_l0`'s output survives an independent
//! closedness re-derivation with its structure intact. The real E3 — genuinely different-but-
//! alpha-equivalent pairs, produced by an actual sugar **expansion** with `%`-freshening
//! (`expand → reveal_l0 → reelaborate → alpha_eq`) — needs expression-position sugar rules that do
//! not exist yet; it is the E1/E3 experiment DN-110-8.2-hygiene-deepdive §7 names, tracked as a
//! follow-on (M-1055), not built in this increment. `alpha_eq` itself is separately unit-tested
//! against hand-built alpha-variant pairs (renamed binders across every binder-introducing `Node`
//! form — `Let`/`Lam`/`Fix`/`FixGroup`/`Alt::Ctor`) in `src/tests/reveal.rs`, which *is* a genuine
//! (if synthetic, non-sugar-derived) correctness check on the comparator in isolation.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use mycelium_core::{Alt, Node, Payload, Repr, ScalarKind, SparsityClass, Trit, Value};

use crate::checkty::Env;
use crate::elab::{elaborate, ElabError};
use crate::totality::{WalkDepthExceeded, MAX_WALK_DEPTH};

// ---------------------------------------------------------------------------------------------
// reveal_l0
// ---------------------------------------------------------------------------------------------

/// Show the real, elaborated L0 [`Node`] for entry symbol `site` in `env` (DN-38 §5).
///
/// v0's `site` locator is exactly [`crate::elab::elaborate`]'s `entry: &str` — a nullary entry
/// symbol already checked into `env` (no arbitrary source-span resolver; Increment-2). This is a
/// **direct wrap**: the DN-38 §8.3 pinned v0-fidelity ruling (module doc) is that the shown term
/// *is* the literal `elaborate` output, so no rendering/reconstruction happens here — `Exact`
/// (identity composed with `elaborate`).
///
/// # Errors
/// [`RevealError::Elab`] wraps whatever [`ElabError`] `elaborate` itself would raise — most notably
/// [`ElabError::Residual`] (the entry falls outside the evaluation-complete fragment) or
/// [`ElabError::UnknownFn`] (no such entry). Never a partial/half-shown term (G2).
pub fn reveal_l0(env: &Env, site: &str) -> Result<Node, RevealError> {
    elaborate(env, site).map_err(RevealError::Elab)
}

/// Errors from the `reveal` primitives ([`reveal_l0`], [`reelaborate`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevealError {
    /// `site` could not be elaborated to L0 at all (see [`ElabError`]).
    Elab(ElabError),
    /// A [`reelaborate`] closedness re-derivation exceeded its own recursion-depth budget
    /// ([`MAX_WALK_DEPTH`], shared with [`crate::totality`]/[`crate::mono`]) — a clean refusal
    /// rather than a host-stack overflow on a pathologically-nested shown term (G2).
    DepthExceeded(WalkDepthExceeded),
    /// A [`reelaborate`] closedness re-derivation found the shown [`Node`] is **not** closed — it
    /// references at least one [`Node::Var`] with no enclosing binder. Naming (sorted,
    /// deterministic) the offending free names; never a silent accept of a malformed "shown" term.
    NotClosed(Vec<String>),
}

impl std::fmt::Display for RevealError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RevealError::Elab(e) => write!(f, "reveal: {e}"),
            RevealError::DepthExceeded(e) => write!(f, "reveal: reelaborate: {e}"),
            RevealError::NotClosed(names) => write!(
                f,
                "reveal: reelaborate: the shown L0 term is not closed — free reference(s): {}",
                names.join(", ")
            ),
        }
    }
}

impl std::error::Error for RevealError {}

// ---------------------------------------------------------------------------------------------
// reelaborate
// ---------------------------------------------------------------------------------------------

/// Re-derive (not merely inspect) the closedness witness of a shown L0 [`Node`] — the v0
/// `reelaborate` primitive the module doc's "`reelaborate` at v0" section explains. Returns a
/// clone of `shown` when the independent structural walk confirms every [`Node::Var`] is bound by
/// an enclosing binder within `shown` itself; a never-silent [`RevealError`] otherwise.
///
/// # Errors
/// [`RevealError::NotClosed`] naming the free reference(s); [`RevealError::DepthExceeded`] on a
/// pathologically-nested `shown` (never a host-stack overflow, G2).
pub fn reelaborate(shown: &Node) -> Result<Node, RevealError> {
    let mut bound: Vec<String> = Vec::new();
    let mut free: BTreeSet<String> = BTreeSet::new();
    collect_free_vars(shown, &mut bound, &mut free, 0).map_err(RevealError::DepthExceeded)?;
    if !free.is_empty() {
        return Err(RevealError::NotClosed(free.into_iter().collect()));
    }
    Ok(shown.clone())
}

/// The depth-tracked free-variable walk behind [`reelaborate`] — mirrors
/// [`crate::mono::free_vars`]/[`crate::totality::walk_expr`]'s M-674/M-866 discipline (shared
/// [`MAX_WALK_DEPTH`] budget, DRY) but retargeted at the elaborated [`Node`] grammar rather than
/// the surface [`crate::ast::Expr`].
fn collect_free_vars(
    node: &Node,
    bound: &mut Vec<String>,
    free: &mut BTreeSet<String>,
    depth: u32,
) -> Result<(), WalkDepthExceeded> {
    let depth = depth + 1;
    if depth > MAX_WALK_DEPTH {
        return Err(WalkDepthExceeded {
            limit: MAX_WALK_DEPTH,
        });
    }
    match node {
        Node::Const(_) => {}
        Node::Var(id) => {
            if !bound.iter().any(|b| b == id) {
                free.insert(id.clone());
            }
        }
        Node::Let { id, bound: b, body } => {
            collect_free_vars(b, bound, free, depth)?;
            bound.push(id.clone());
            collect_free_vars(body, bound, free, depth)?;
            bound.pop();
        }
        Node::Op { args, .. } | Node::Construct { args, .. } => {
            for a in args {
                collect_free_vars(a, bound, free, depth)?;
            }
        }
        Node::Swap { src, .. } => collect_free_vars(src, bound, free, depth)?,
        Node::Match {
            scrutinee,
            alts,
            default,
        } => {
            collect_free_vars(scrutinee, bound, free, depth)?;
            for alt in alts {
                match alt {
                    Alt::Ctor { binders, body, .. } => {
                        for b in binders {
                            bound.push(b.clone());
                        }
                        collect_free_vars(body, bound, free, depth)?;
                        for _ in binders {
                            bound.pop();
                        }
                    }
                    Alt::Lit { body, .. } => collect_free_vars(body, bound, free, depth)?,
                }
            }
            if let Some(d) = default {
                collect_free_vars(d, bound, free, depth)?;
            }
        }
        Node::Lam { param, body } => {
            bound.push(param.clone());
            collect_free_vars(body, bound, free, depth)?;
            bound.pop();
        }
        Node::App { func, arg } => {
            collect_free_vars(func, bound, free, depth)?;
            collect_free_vars(arg, bound, free, depth)?;
        }
        Node::Fix { name, body } => {
            bound.push(name.clone());
            collect_free_vars(body, bound, free, depth)?;
            bound.pop();
        }
        Node::FixGroup { defs, body } => {
            for (name, _) in defs {
                bound.push(name.clone());
            }
            for (_, d) in defs {
                collect_free_vars(d, bound, free, depth)?;
            }
            collect_free_vars(body, bound, free, depth)?;
            for _ in defs {
                bound.pop();
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// alpha_eq
// ---------------------------------------------------------------------------------------------

/// Structural **alpha-equivalence** over [`Node`].
///
/// **Why this exists (critical, per the task grounding):** [`Node`]'s [`PartialEq`] is *literal*
/// structural equality — binder names (`Let::id`, `Lam::param`, `Fix::name`, `FixGroup` def names,
/// `Alt::Ctor::binders`) are compared **by spelling** (`crates/mycelium-core/src/node.rs`'s manual
/// iterative `PartialEq`, e.g. the `Let` arm's `if i1 != i2 { return false; }`), with **no**
/// pre-comparison alpha-canonicalization anywhere in the read path
/// (DN-110-8.2-hygiene-deepdive §6's "false half" finding). So `λx.x` and `λy.y` are `!=` under
/// `==` even though they denote the same term — `==` cannot be used as the E3 round-trip oracle.
///
/// This is an explicit, independent structural alpha-comparison: a paired walk over both terms
/// that tracks each side's own binder-introduction order as a de-Bruijn-style positional stack
/// (`actx`/`bctx`) and, at a [`Node::Var`], compares **bound-position** (not spelling) when both
/// sides' occurrences are bound, or **spelling** when both are free (an out-of-scope reference —
/// never expected on a closed `elaborate` output, but handled honestly rather than assumed away).
/// A bound-vs-free mismatch is unequal. `FixGroup`/`Match`-arm binder groups are compared
/// **positionally** (index `i` in one side's group corresponds to index `i` in the other's) —
/// the same convention `Node`'s own structural `PartialEq` already uses for those groups, just with
/// alpha-aware `Var` occurrences layered on top.
///
/// **Depth budget (honest scoping note).** Bounded by the shared [`MAX_WALK_DEPTH`] (4096); past it
/// this returns `false` rather than a host-stack overflow — a conservative approximation on a
/// pathologically-nested adversarial pair, not a claim of exactness there. Every fixture in this
/// increment's corpus (single elaborated function bodies) sits far under the budget, mirroring the
/// existing crate convention (`crate::totality::MAX_WALK_DEPTH`'s own measured ~500× margin).
#[must_use]
pub fn alpha_eq(a: &Node, b: &Node) -> bool {
    let mut actx: Vec<String> = Vec::new();
    let mut bctx: Vec<String> = Vec::new();
    alpha_eq_at(a, b, &mut actx, &mut bctx, 0)
}

/// The innermost (rightmost) index of `name` in `ctx` — respects shadowing the same way ordinary
/// lexical scoping does (a re-bound name shadows the outer one; both sides must shadow at the same
/// relative position for `alpha_eq` to hold there).
fn binder_index(ctx: &[String], name: &str) -> Option<usize> {
    ctx.iter().rposition(|s| s == name)
}

fn alpha_eq_at(
    a: &Node,
    b: &Node,
    actx: &mut Vec<String>,
    bctx: &mut Vec<String>,
    depth: u32,
) -> bool {
    let depth = depth + 1;
    if depth > MAX_WALK_DEPTH {
        return false;
    }
    match (a, b) {
        (Node::Const(x), Node::Const(y)) => x == y,
        (Node::Var(x), Node::Var(y)) => match (binder_index(actx, x), binder_index(bctx, y)) {
            (Some(ia), Some(ib)) => ia == ib,
            (None, None) => x == y,
            _ => false,
        },
        (
            Node::Let {
                id: i1,
                bound: b1,
                body: y1,
            },
            Node::Let {
                id: i2,
                bound: b2,
                body: y2,
            },
        ) => {
            if !alpha_eq_at(b1, b2, actx, bctx, depth) {
                return false;
            }
            actx.push(i1.clone());
            bctx.push(i2.clone());
            let r = alpha_eq_at(y1, y2, actx, bctx, depth);
            actx.pop();
            bctx.pop();
            r
        }
        (Node::Op { prim: p1, args: a1 }, Node::Op { prim: p2, args: a2 }) => {
            p1 == p2
                && a1.len() == a2.len()
                && a1
                    .iter()
                    .zip(a2)
                    .all(|(x, y)| alpha_eq_at(x, y, actx, bctx, depth))
        }
        (
            Node::Swap {
                src: s1,
                target: t1,
                policy: pol1,
            },
            Node::Swap {
                src: s2,
                target: t2,
                policy: pol2,
            },
        ) => t1 == t2 && pol1 == pol2 && alpha_eq_at(s1, s2, actx, bctx, depth),
        (Node::Construct { ctor: c1, args: a1 }, Node::Construct { ctor: c2, args: a2 }) => {
            c1 == c2
                && a1.len() == a2.len()
                && a1
                    .iter()
                    .zip(a2)
                    .all(|(x, y)| alpha_eq_at(x, y, actx, bctx, depth))
        }
        (
            Node::Match {
                scrutinee: s1,
                alts: al1,
                default: d1,
            },
            Node::Match {
                scrutinee: s2,
                alts: al2,
                default: d2,
            },
        ) => {
            if !alpha_eq_at(s1, s2, actx, bctx, depth) || al1.len() != al2.len() {
                return false;
            }
            for (x, y) in al1.iter().zip(al2) {
                if !alpha_eq_alt(x, y, actx, bctx, depth) {
                    return false;
                }
            }
            match (d1, d2) {
                (None, None) => true,
                (Some(x), Some(y)) => alpha_eq_at(x, y, actx, bctx, depth),
                _ => false,
            }
        }
        (
            Node::Lam {
                param: p1,
                body: b1,
            },
            Node::Lam {
                param: p2,
                body: b2,
            },
        ) => {
            actx.push(p1.clone());
            bctx.push(p2.clone());
            let r = alpha_eq_at(b1, b2, actx, bctx, depth);
            actx.pop();
            bctx.pop();
            r
        }
        (Node::App { func: f1, arg: a1 }, Node::App { func: f2, arg: a2 }) => {
            alpha_eq_at(f1, f2, actx, bctx, depth) && alpha_eq_at(a1, a2, actx, bctx, depth)
        }
        (Node::Fix { name: n1, body: b1 }, Node::Fix { name: n2, body: b2 }) => {
            actx.push(n1.clone());
            bctx.push(n2.clone());
            let r = alpha_eq_at(b1, b2, actx, bctx, depth);
            actx.pop();
            bctx.pop();
            r
        }
        (Node::FixGroup { defs: d1, body: b1 }, Node::FixGroup { defs: d2, body: b2 }) => {
            if d1.len() != d2.len() {
                return false;
            }
            for (n, _) in d1 {
                actx.push(n.clone());
            }
            for (n, _) in d2 {
                bctx.push(n.clone());
            }
            let mut ok = d1
                .iter()
                .zip(d2)
                .all(|((_, x), (_, y))| alpha_eq_at(x, y, actx, bctx, depth));
            if ok {
                ok = alpha_eq_at(b1, b2, actx, bctx, depth);
            }
            for _ in d1 {
                actx.pop();
            }
            for _ in d2 {
                bctx.pop();
            }
            ok
        }
        // Different Node variants are unequal (mirrors Node::PartialEq's own catch-all).
        _ => false,
    }
}

fn alpha_eq_alt(
    a: &Alt,
    b: &Alt,
    actx: &mut Vec<String>,
    bctx: &mut Vec<String>,
    depth: u32,
) -> bool {
    match (a, b) {
        (
            Alt::Ctor {
                ctor: c1,
                binders: bd1,
                body: bo1,
            },
            Alt::Ctor {
                ctor: c2,
                binders: bd2,
                body: bo2,
            },
        ) => {
            if c1 != c2 || bd1.len() != bd2.len() {
                return false;
            }
            for n in bd1 {
                actx.push(n.clone());
            }
            for n in bd2 {
                bctx.push(n.clone());
            }
            let r = alpha_eq_at(bo1, bo2, actx, bctx, depth);
            for _ in bd1 {
                actx.pop();
            }
            for _ in bd2 {
                bctx.pop();
            }
            r
        }
        (
            Alt::Lit {
                value: v1,
                body: bo1,
            },
            Alt::Lit {
                value: v2,
                body: bo2,
            },
        ) => v1 == v2 && alpha_eq_at(bo1, bo2, actx, bctx, depth),
        _ => false,
    }
}

// ---------------------------------------------------------------------------------------------
// render_surface — the Node -> surface pretty-printer
// ---------------------------------------------------------------------------------------------

/// The output of [`render_surface`]: best-effort surface text for a closed L0 [`Node`], plus an
/// honest, **mechanically computed** reparseability flag (OQ-H3 option (a), module doc).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
    /// The rendered surface-syntax text.
    pub text: String,
    /// `false` whenever `text` embeds a token the surface lexer/grammar has no rule for — a raw
    /// `%`-freshened hygienic name (OQ-H3), a resolved `#<hash>`/`#<hash>#<i>` content-address
    /// (`Swap::policy`/`Construct::ctor`, no surviving surface spelling), or a non-bare-identifier
    /// [`mycelium_core::Prim`] name (a `wild:name` host-call spelling, `#op[..]`-marked). Computed
    /// mechanically by scanning `text` for the two marker characters (`%`, `#`) neither of which the
    /// surface lexer ever accepts inside a real identifier/keyword token (`lexer.rs` — `#` is
    /// unhandled entirely, `%` is only ever `Tok::Percent`, never an identifier constituent) — so
    /// this flag is a **sound, conservative** (never a false "reparseable") proxy for "is this
    /// literally the token stream a `reveal`-showing human could paste back into a `.myc` file",
    /// never a claim that the rest of `text` round-trips through the *checker* (only through the
    /// *lexer/parser* — see the module doc's `reelaborate` section for where the real identity
    /// witness lives).
    pub reparseable: bool,
}

/// A [`render_surface`] refusal — never a silent gap or a `todo!()` (house rule #2/G2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    /// The traversal's own recursion exceeded [`MAX_WALK_DEPTH`] (shared budget, DRY).
    DepthExceeded(WalkDepthExceeded),
    /// No known surface-syntax form exists for this piece of the term. Named honestly rather than
    /// approximated: `node` identifies the offending construct kind, `detail` says why.
    Unrenderable {
        /// The offending construct kind (e.g. `"Const(Dense)"`, `"Const(NaN/±inf Float)"`).
        node: &'static str,
        /// Why no surface form exists.
        detail: String,
    },
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::DepthExceeded(e) => write!(f, "render_surface: {e}"),
            RenderError::Unrenderable { node, detail } => {
                write!(f, "render_surface: cannot render {node}: {detail}")
            }
        }
    }
}

impl std::error::Error for RenderError {}

/// Render `node` toward surface grammar, best-effort, covering the whole closed [`Node`] grammar
/// (every variant has a render arm below); anything with genuinely no surface-syntax form is an
/// explicit [`RenderError::Unrenderable`] (never a silent gap — house rule #2). See [`Rendered`]
/// for what `reparseable` does and does not promise.
///
/// # Errors
/// [`RenderError::DepthExceeded`] on a pathologically-nested `node` (G2, never a host-stack
/// overflow); [`RenderError::Unrenderable`] for a [`Node::Const`] whose [`Value`] has no surface
/// literal grammar today — a [`Repr::Dense`]/[`Repr::Vsa`] payload (no dense/hypervector literal
/// syntax exists in the L1 surface grammar), or a non-finite [`Repr::Float`] value (the lexer's
/// `FloatLit` only ever accepts a finite decimal form, ADR-040 §2.4 — `NaN`/`±inf` values can arise
/// from evaluation but never from a literal).
pub fn render_surface(node: &Node) -> Result<Rendered, RenderError> {
    let text = render_node(node, 0)?;
    let reparseable = !text.contains('%') && !text.contains('#');
    Ok(Rendered { text, reparseable })
}

fn charge_depth(depth: u32) -> Result<u32, RenderError> {
    let depth = depth + 1;
    if depth > MAX_WALK_DEPTH {
        return Err(RenderError::DepthExceeded(WalkDepthExceeded {
            limit: MAX_WALK_DEPTH,
        }));
    }
    Ok(depth)
}

fn is_bare_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn render_node(node: &Node, depth: u32) -> Result<String, RenderError> {
    let depth = charge_depth(depth)?;
    match node {
        Node::Const(v) => render_value(v),
        Node::Var(id) => Ok(id.clone()),
        Node::Let { id, bound, body } => Ok(format!(
            "let {id} = {} in {}",
            render_node(bound, depth)?,
            render_node(body, depth)?
        )),
        Node::Op { prim, args } => {
            let rendered = render_args(args, depth)?;
            if is_bare_ident(prim) {
                Ok(format!("{prim}({rendered})"))
            } else {
                // e.g. a `wild:name` host-call prim — no bare-identifier surface spelling.
                Ok(format!("#op[{prim}]({rendered})"))
            }
        }
        Node::Swap {
            src,
            target,
            policy,
        } => Ok(format!(
            "swap({}, to: {}, policy: #{})",
            render_node(src, depth)?,
            render_repr(target),
            policy.as_str()
        )),
        Node::Construct { ctor, args } => {
            let rendered = render_args(args, depth)?;
            // `CtorRef`'s own canonical `Display` is the Unison spelling `#<hash>#<i>` — already
            // `#`-prefixed, so it naturally trips the reparseable=false scan (no surface spelling
            // survives elaboration to invert it back to a constructor name — ADR-003).
            Ok(format!("{ctor}({rendered})"))
        }
        Node::Match {
            scrutinee,
            alts,
            default,
        } => {
            let mut arms = Vec::with_capacity(alts.len() + usize::from(default.is_some()));
            for alt in alts {
                arms.push(render_alt(alt, depth)?);
            }
            if let Some(d) = default {
                arms.push(format!("_ => {}", render_node(d, depth)?));
            }
            Ok(format!(
                "match {} {{ {} }}",
                render_node(scrutinee, depth)?,
                arms.join(", ")
            ))
        }
        Node::Lam { param, body } => {
            Ok(format!("lambda({param}) => {}", render_node(body, depth)?))
        }
        Node::App { func, arg } => {
            // Flatten a left-nested curried App chain App(App(f,a1),a2)... into f(a1, a2, ...) —
            // an iterative (non-recursive) walk down the spine, so only the head/args themselves
            // are charged against the recursion-depth budget.
            let mut rev_args: Vec<&Node> = vec![arg.as_ref()];
            let mut head: &Node = func.as_ref();
            while let Node::App { func: f2, arg: a2 } = head {
                rev_args.push(a2.as_ref());
                head = f2.as_ref();
            }
            let head_text = render_node(head, depth)?;
            let mut arg_texts = Vec::with_capacity(rev_args.len());
            for a in rev_args.into_iter().rev() {
                arg_texts.push(render_node(a, depth)?);
            }
            Ok(format!("{head_text}({})", arg_texts.join(", ")))
        }
        Node::Fix { name, body } => {
            // No surface expression-position spelling — `Fix` arises from a top-level recursive
            // `fn` binding, not an expression the surface writes directly. `#`-marked, honest.
            Ok(format!("#fix[{name}]({})", render_node(body, depth)?))
        }
        Node::FixGroup { defs, body } => {
            let mut parts = Vec::with_capacity(defs.len());
            for (name, d) in defs {
                parts.push(format!("{name} = {}", render_node(d, depth)?));
            }
            Ok(format!(
                "#fixgroup[{}]({})",
                parts.join("; "),
                render_node(body, depth)?
            ))
        }
    }
}

fn render_args(args: &[Node], depth: u32) -> Result<String, RenderError> {
    let mut parts = Vec::with_capacity(args.len());
    for a in args {
        parts.push(render_node(a, depth)?);
    }
    Ok(parts.join(", "))
}

fn render_alt(alt: &Alt, depth: u32) -> Result<String, RenderError> {
    match alt {
        Alt::Ctor {
            ctor,
            binders,
            body,
        } => Ok(format!(
            "{ctor}({}) => {}",
            binders.join(", "),
            render_node(body, depth)?
        )),
        Alt::Lit { value, body } => Ok(format!(
            "{} => {}",
            render_value(value)?,
            render_node(body, depth)?
        )),
    }
}

/// Render a [`Repr`] type-descriptor toward its surface `TypeRef` spelling (`Binary{8}`,
/// `Ternary{6}`, `Dense{128, F32}`, `VSA{model, 1024, Sparse{16}}`, `Seq{Binary{8}, 3}`, `Bytes`,
/// `Float`) — total: every `Repr` variant has real surface type-syntax (`parse.rs::parse_base_type`),
/// unlike a `Repr`'s *literal payload* rendering ([`render_value`]), which is not total.
fn render_repr(repr: &Repr) -> String {
    match repr {
        Repr::Binary { width } => format!("Binary{{{width}}}"),
        Repr::Ternary { trits } => format!("Ternary{{{trits}}}"),
        Repr::Dense { dim, dtype } => format!("Dense{{{dim}, {}}}", render_scalar_kind(dtype)),
        Repr::Vsa {
            model,
            dim,
            sparsity,
        } => format!("VSA{{{model}, {dim}, {}}}", render_sparsity(sparsity)),
        Repr::Seq { elem, len } => format!("Seq{{{}, {len}}}", render_repr(elem)),
        Repr::Float { .. } => "Float".to_owned(),
        Repr::Bytes => "Bytes".to_owned(),
    }
}

fn render_scalar_kind(k: &ScalarKind) -> &'static str {
    match k {
        ScalarKind::F16 => "F16",
        ScalarKind::Bf16 => "BF16",
        ScalarKind::F32 => "F32",
        ScalarKind::F64 => "F64",
    }
}

fn render_sparsity(s: &SparsityClass) -> String {
    match s {
        SparsityClass::Dense => "Dense".to_owned(),
        SparsityClass::Sparse { max_active } => format!("Sparse{{{max_active}}}"),
    }
}

/// Render a [`Value`] toward its surface literal spelling — **not total** (see [`render_surface`]'s
/// `# Errors`): a [`Repr::Dense`]/[`Repr::Vsa`] payload and a non-finite [`Repr::Float`] have no
/// literal surface form, and are an explicit [`RenderError::Unrenderable`] rather than a fabricated
/// approximation (G2).
fn render_value(v: &Value) -> Result<String, RenderError> {
    match (v.repr(), v.payload()) {
        (Repr::Binary { .. }, Payload::Bits(bits)) => Ok(format!(
            "0b{}",
            bits.iter()
                .map(|&b| if b { '1' } else { '0' })
                .collect::<String>()
        )),
        (Repr::Ternary { .. }, Payload::Trits(trits)) => Ok(format!(
            "0t{}",
            trits
                .iter()
                .map(|t| match t {
                    Trit::Neg => '-',
                    Trit::Zero => '0',
                    Trit::Pos => '+',
                })
                .collect::<String>()
        )),
        (Repr::Bytes, Payload::Bytes(bytes)) => Ok(format!("0x{}", hex_encode(bytes))),
        (Repr::Float { .. }, Payload::Float(x)) => {
            if x.is_finite() {
                // `{:?}` always carries a decimal point (`1.0`, not `1`) — the surface `FloatLit`
                // grammar requires `digits '.' digits` (matches `value.rs`'s own wire-form
                // convention for the same reason, DRY).
                Ok(format!("{x:?}"))
            } else {
                Err(RenderError::Unrenderable {
                    node: "Const(NaN/±inf Float)",
                    detail: "the surface FloatLit grammar (ADR-040 §2.4) only accepts a finite \
                              decimal literal; a NaN/±inf value can only arise from evaluation, \
                              never from a literal, so it has no surface spelling to show"
                        .to_owned(),
                })
            }
        }
        (Repr::Seq { .. }, Payload::Seq(elems)) => {
            let mut parts = Vec::with_capacity(elems.len());
            for e in elems {
                parts.push(render_value(e)?);
            }
            Ok(format!("[{}]", parts.join(", ")))
        }
        (Repr::Dense { .. }, _) => Err(RenderError::Unrenderable {
            node: "Const(Dense)",
            detail: "no dense-embedding literal syntax exists in the L1 surface grammar (v0 gap, \
                      Declared) — a Dense value is only ever produced by evaluation/ops, never \
                      written as a literal"
                .to_owned(),
        }),
        (Repr::Vsa { .. }, _) => Err(RenderError::Unrenderable {
            node: "Const(Vsa)",
            detail: "no hypervector literal syntax exists in the L1 surface grammar (v0 gap, \
                      Declared) — a VSA value is only ever produced by evaluation/ops, never \
                      written as a literal"
                .to_owned(),
        }),
        (repr, _) => Err(RenderError::Unrenderable {
            node: "Const",
            detail: format!(
                "repr/payload combination has no known surface literal form (repr = {repr:?})"
            ),
        }),
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}
