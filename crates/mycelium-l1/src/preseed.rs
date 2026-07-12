//! DN-129 §5 — the shared **prelude-trait seeding spine** every built-in, conditionally-seeded
//! trait rides: `Fuse` (M-965 F-A1), `Ord3` (DN-122 §13 / M-1080 WU-B), `Show` (DN-127),
//! `Init`/`Fault` (DN-129). Factored out of the three copy-pasted `Fuse`/`Ord3` conditionals that
//! previously lived at [`crate::checkty::register_nodule_decls`] (per-nodule registration +
//! redeclare refusal), the [`crate::checkty::PhylumEnv::link`] phylum-wide runtime merge, and the
//! [`crate::checkty::OwnDecls`] exclusion filter — a pure DRY refactor of already-landed logic
//! (KC-3-neutral: no new mechanism, one shared implementation instead of N hand-copied ones).
//!
//! **Behavior for `Fuse`/`Ord3` is byte-identical after this refactor.** Their own regression
//! suites (`tests/fuse.rs`/`tests/ord3.rs`) only assert `err.message.contains(name) &&
//! err.message.contains("built-in")` — never the exact wording of the redeclare-refusal message —
//! so unifying the message text under one shared template is a safe substitution, verified by
//! re-running those suites unchanged (mitigation #14: verify, don't assume).

use std::collections::BTreeMap;

use crate::ast::{Item, Nodule, Path};
use crate::checkty::{CheckError, Env, TraitInfo};

/// One prelude trait's registration bundle — the small interface [`PreludeTraitSeed::seed_for_nodule`]
/// / [`PreludeTraitSeed::seed_for_link`] are written against **once**, instead of being
/// re-implemented per trait. Every prelude trait module (`fuse`, `ord3`, `show`, `init`, `fault`)
/// exposes a `const SEED: PreludeTraitSeed` (or an equivalent constructor) built from this shape.
pub(crate) struct PreludeTraitSeed {
    /// This trait's name — the one string every registration/lookup/exclusion site agrees on
    /// (Law of Demeter — a single named constant beats a scattered literal).
    pub(crate) name: &'static str,
    /// A short surface-syntax hint for the redeclare-refusal message, e.g.
    /// `"impl Fuse[T] for T { fn join(a: T, b: T) => T = … }"` — purely diagnostic text, never
    /// parsed or otherwise load-bearing.
    pub(crate) impl_hint: &'static str,
    /// Builds the hand-built [`TraitInfo`] this trait seeds into a registry.
    pub(crate) prelude: fn() -> TraitInfo,
}

impl PreludeTraitSeed {
    /// Per-nodule registration-pass seeding (mirrors the landed `Fuse`/`Ord3` conditional
    /// previously inlined in [`crate::checkty::register_nodule_decls`]): seed `self.name` into
    /// `traits` **iff** `nodule.items` declares an `impl <name>[...] for ...`, refusing any
    /// attempt to shadow the built-in trait with a local `trait <name> ...` declaration (never a
    /// silent shadow of the prelude — G2).
    pub(crate) fn seed_for_nodule(
        &self,
        traits: &mut BTreeMap<String, TraitInfo>,
        nodule: &Nodule,
    ) -> Result<(), CheckError> {
        let used = nodule
            .items
            .iter()
            .any(|item| matches!(item, Item::Impl(id) if id.trait_name == self.name));
        if used {
            if traits.contains_key(self.name) {
                return Err(self.redeclare_error());
            }
            traits.insert(self.name.to_owned(), (self.prelude)());
        } else if traits.contains_key(self.name) {
            return Err(self.redeclare_error());
        }
        Ok(())
    }

    /// Phylum-wide runtime-link seeding (mirrors the landed `Fuse`/`Ord3` conditional previously
    /// inlined in [`crate::checkty::PhylumEnv::link`]): present in the linked env **iff** some
    /// nodule's already-checked [`Env`] actually declared an instance of it.
    pub(crate) fn seed_for_link(
        &self,
        traits: &mut BTreeMap<String, TraitInfo>,
        nodules: &[(Path, Env)],
    ) {
        if nodules
            .iter()
            .any(|(_, e)| e.traits.contains_key(self.name))
        {
            traits.insert(self.name.to_owned(), (self.prelude)());
        }
    }

    /// The never-silent (G2) redeclare-refusal `CheckError`: naming the trait, that it is
    /// built-in, and a corrected surface-syntax hint — generalized from the `Fuse`/`Ord3`-specific
    /// wording, but still specific enough to be actionable per trait.
    fn redeclare_error(&self) -> CheckError {
        CheckError::new(
            self.name,
            format!(
                "cannot redeclare the built-in prelude trait `{}` — its contract is already \
                 provided by the prelude; remove this declaration and `{}` directly",
                self.name, self.impl_hint
            ),
        )
    }
}
