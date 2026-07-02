//! The static affine **use-once** tracker for `Substrate` bindings (M-903; DN-71 Model S §4.2).
//!
//! `Substrate{tag}` is the affine external-resource kind (RFC-0006 LR-8): a binding of that type
//! must be **moved at most once along any control path** (DN-71 §4.2). A *move* is any reference to
//! the binding — `consume`, an argument pass, a return, a constructor/field capture — since this
//! language has no borrow/reference concept: every value use is a move (RT1). This module tracks
//! exactly that, indexed against the type-checker's own lexical `scope: Vec<(String, Ty)>`.
//!
//! # Design — piggybacked on the checker's existing scope, not a parallel pass
//! DN-71 §4.2 directs this be *"built on the checker's existing linearity machinery
//! (`check_linear` precedent), not a parallel analysis"* (DRY/KC-3). [`Tracker`] mirrors
//! [`crate::checkty::Cx`]'s `scope: Vec<(String, Ty)>` **by index**, not by name — a shadowing
//! rebinding of the same name is a fresh, independently-tracked slot, exactly as `scope` itself
//! treats shadowing ("a lexical stack; shadowing = later wins"). Every `scope.push`/`pop`/
//! `truncate` call site in `checkty.rs` pairs a matching [`Tracker`] call, so the two stacks always
//! have equal length and the same index always names the same binding in both.
//!
//! A [`Tracker`] is constructed **inert** ([`Tracker::inert`]) for the two `Cx` contexts that are
//! not a whole-function-body check — `check_lower_rule_rhs_type` (a `lower` rule's RHS has no value
//! parameters, so no `Substrate` binding is ever in scope there) and `infer_type` (the elaborator's
//! **post-check** re-inference over an already-validated term, RFC-0011, invoked repeatedly over
//! partial/overlapping fragments with a scope the elaborator threads itself — running the affine
//! pass again there could false-positive on a fragment that is not the whole original walk). Only
//! `check_fn_body`'s `Cx` — the one full walk of one function body from its parameter scope — gets
//! an **active** tracker ([`Tracker::seeded`]). Every push/pop/truncate/use hook on an inert tracker
//! is a guaranteed no-op (never a silent behavior change to the two non-affine contexts — G2).
//!
//! # Branch merging — conservative union, sound over precise (`Empirical`, not `Proven`)
//! `if`/`match` alternatives are mutually exclusive at runtime, so each is checked from the **same**
//! pre-branch snapshot ([`Tracker::snapshot`] / [`Tracker::restore`]); the states after each
//! alternative are then combined as the **union** of "moved" outcomes
//! ([`Tracker::merge_alt`]/[`union_merge_into`]) — a slot moved in *any* alternative is treated as
//! moved going forward. This can reject a handful of programs a fully path-sensitive analysis would
//! accept (e.g. consuming a handle in only one arm, with no further use afterward, is still fine —
//! the merge only bites when a *later* reference exists), but it can **never** let a real
//! double-consume slip through undetected: VR-5 — sound over precise for v0.
//!
//! # A known, honest limitation — loop/closure bodies run a statically-unknown number of times
//! A `for` body and a `lambda` body are each checked **once**, lexically, but may *execute* zero,
//! one, or many times at runtime (a `for` iterates its spine; a closure may be called repeatedly).
//! A `Substrate` reference inside such a body is tracked as **one** lexical use — sound only when
//! that body in fact runs at most once. This is a real gap in the *static* pass for v0 (not silently
//! hidden — recorded here), and it is exactly why DN-71 §4.2 keeps the **runtime consumed-flag
//! backstop** ([`crate::substrate::SubstrateHandle::try_consume`]) as more than defense in depth: a
//! double-consume that escapes this lexical approximation still traps, never silently, at the first
//! repeated move. Closing this gap statically (an effect/multiplicity system over closures and
//! `for`) is future work, not required for the `Empirical` guarantee this pass ships at.

use crate::checkty::Ty;
use std::cell::{Cell, RefCell};

/// Where a use (move) happened, for the both-sites double-consume diagnostic. This checker has no
/// source-span machinery (a [`crate::checkty::CheckError`] carries only a function-name `site` and a
/// free-form message — see `check_linear`'s precedent), so a "site" here is **honestly** just a
/// stable, monotonically increasing **use ordinal** within the function body being checked — never a
/// fabricated line/column (VR-5: don't claim a precision this checker doesn't have).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UseSite {
    pub(crate) ordinal: u32,
}

/// One scope slot's affine state, mirroring [`crate::checkty::Cx`]'s `scope` by index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Slot {
    /// Not a `Substrate`-typed binding — nothing to track (every non-affine binding is `Skip`, so
    /// the tracker never has an opinion about it).
    Skip,
    /// A live, unmoved `Substrate{tag}` binding.
    Live { tag: String },
    /// Already moved (used) once — `first_use` names where, for the both-sites diagnostic.
    Moved { tag: String, first_use: UseSite },
}

impl Slot {
    /// The slot a fresh binding of type `ty` starts in: [`Slot::Live`] for a `Substrate{tag}`,
    /// [`Slot::Skip`] for everything else.
    fn for_ty(ty: &Ty) -> Self {
        match ty {
            Ty::Substrate(tag) => Slot::Live { tag: tag.clone() },
            _ => Slot::Skip,
        }
    }
}

/// The outcome of recording a use ([`Tracker::use_at`]) at some scope index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UseOutcome {
    /// The binding is not `Substrate`-typed (or the tracker is inert) — nothing to enforce.
    NotAffine,
    /// The binding's first move — now `Moved`.
    FirstUse,
    /// A **double-consume**: the binding was already moved. Carries both sites for the
    /// RFC-0013-style both-sites diagnostic — the earlier move's ordinal and this violating use's.
    DoubleUse {
        tag: String,
        first_ordinal: u32,
        this_ordinal: u32,
    },
}

/// Union-merge `other` into `acc` **in place**: a slot that is [`Slot::Live`] in `acc` but
/// [`Slot::Moved`] in `other` becomes `Moved` (a slot moved in *either* alternative is moved
/// afterward — the conservative branch-merge rule, module docs above). `acc` and `other` must be
/// the same length (both are snapshots of the same pre-branch scope depth); a length mismatch is an
/// internal-invariant violation the caller is responsible for not producing (every call site merges
/// two snapshots taken at the same scope depth).
pub(crate) fn union_merge_into(acc: &mut [Slot], other: &[Slot]) {
    debug_assert_eq!(
        acc.len(),
        other.len(),
        "union_merge_into: branch snapshots must share the same pre-branch scope depth"
    );
    for (a, o) in acc.iter_mut().zip(other.iter()) {
        if matches!(a, Slot::Live { .. }) {
            if let Slot::Moved { .. } = o {
                *a = o.clone();
            }
        }
    }
}

/// The affine tracker for one `Cx::check` walk (M-903; DN-71 §4.2). See the module docs for the
/// inert-vs-active split, the index-lockstep-with-`scope` invariant, and the branch-merge rule.
#[derive(Debug)]
pub(crate) struct Tracker {
    /// `None` ⇒ **inert** (every operation below is a no-op / [`UseOutcome::NotAffine`]).
    /// `Some` ⇒ **active**, seeded from `check_fn_body`'s initial parameter scope.
    slots: Option<RefCell<Vec<Slot>>>,
    next_ordinal: Cell<u32>,
}

impl Tracker {
    /// An inert tracker — used by the two `Cx` contexts that are not a whole-function-body walk
    /// (module docs). Every subsequent call is a guaranteed no-op.
    pub(crate) fn inert() -> Self {
        Tracker {
            slots: None,
            next_ordinal: Cell::new(0),
        }
    }

    /// An active tracker seeded with one slot per entry of `initial_scope` (a function's value
    /// parameters — `check_fn_body`'s only pre-populated scope), so a `Substrate`-typed parameter is
    /// tracked from the body's very first statement (a parameter pass already counts as a use — the
    /// caller moved it in).
    pub(crate) fn seeded(initial_scope: &[(String, Ty)]) -> Self {
        Tracker {
            slots: Some(RefCell::new(
                initial_scope
                    .iter()
                    .map(|(_, ty)| Slot::for_ty(ty))
                    .collect(),
            )),
            next_ordinal: Cell::new(0),
        }
    }

    /// Push one fresh slot for a newly-bound `ty` — call at every `scope.push` site.
    pub(crate) fn push(&self, ty: &Ty) {
        if let Some(slots) = &self.slots {
            slots.borrow_mut().push(Slot::for_ty(ty));
        }
    }

    /// Pop one slot — call at every `scope.pop` site.
    pub(crate) fn pop(&self) {
        if let Some(slots) = &self.slots {
            slots.borrow_mut().pop();
        }
    }

    /// Truncate to `len` slots — call at every `scope.truncate(len)` site.
    pub(crate) fn truncate(&self, len: usize) {
        if let Some(slots) = &self.slots {
            slots.borrow_mut().truncate(len);
        }
    }

    /// A snapshot of the current state, for `if`/`match` branch forking. `None` for an inert
    /// tracker (nothing to snapshot; [`Self::restore`]/[`Self::merge_alt`] are then also no-ops).
    pub(crate) fn snapshot(&self) -> Option<Vec<Slot>> {
        self.slots.as_ref().map(|s| s.borrow().clone())
    }

    /// Restore a prior [`Self::snapshot`] — rewinds to a branch's shared pre-state.
    pub(crate) fn restore(&self, snap: &Option<Vec<Slot>>) {
        if let (Some(slots), Some(snap)) = (&self.slots, snap) {
            *slots.borrow_mut() = snap.clone();
        }
    }

    /// Union-merge an alternative branch's post-state ([`Self::snapshot`]) into the tracker's
    /// **current** state (module docs — the conservative branch-merge rule).
    pub(crate) fn merge_alt(&self, alt: Option<Vec<Slot>>) {
        if let (Some(slots), Some(alt)) = (&self.slots, alt) {
            union_merge_into(&mut slots.borrow_mut(), &alt);
        }
    }

    /// Record a **use** (move) of the binding at scope index `idx`. See [`UseOutcome`].
    pub(crate) fn use_at(&self, idx: usize) -> UseOutcome {
        let Some(slots) = &self.slots else {
            return UseOutcome::NotAffine;
        };
        let ordinal = self.next_ordinal.get();
        self.next_ordinal.set(ordinal + 1);
        let mut slots = slots.borrow_mut();
        match &slots[idx] {
            Slot::Skip => UseOutcome::NotAffine,
            Slot::Live { tag } => {
                let tag = tag.clone();
                slots[idx] = Slot::Moved {
                    tag,
                    first_use: UseSite { ordinal },
                };
                UseOutcome::FirstUse
            }
            Slot::Moved { tag, first_use } => UseOutcome::DoubleUse {
                tag: tag.clone(),
                first_ordinal: first_use.ordinal,
                this_ordinal: ordinal,
            },
        }
    }
}
