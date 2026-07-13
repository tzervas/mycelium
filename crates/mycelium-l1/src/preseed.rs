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
use crate::checkty::{type_head, CheckError, Env, InstanceInfo, TraitInfo};

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

/// DN-138 §4.1 Alt A / §8 WU-2 — one seeded **primitive-instance** resolution fact: the parallel,
/// conditional counterpart to [`PreludeTraitSeed`], for an already-landed prelude TRAIT's
/// primitive-repr instance (`Show[Binary{64}]`, `Init[Bool]`, `Ord3[Bytes]`, …). Seeds only the
/// coherence key `(trait, type-head)` plus the concrete `for_ty`/`methods` — **no body**; the real
/// body lives in `lib/std` (`lib/std/fmt.myc` for `Show` — DN-127, already landed;
/// `lib/std/derive_prelude.myc` for `Init`/`Ord3` — DN-138 WU-1), pinned equal to this fact by the
/// sig-pin differential (`crates/mycelium-l1/src/tests/prelude_instance_seed.rs`, DN-138 §5
/// obligation 1 — the load-bearing soundness gate this whole mechanism rests on).
pub(crate) struct PreludeInstanceSeed {
    /// The trait this seeds a primitive instance of — one of [`crate::checkty::PRELUDE_TRAIT_SEEDS`]'s
    /// names (`Show`/`Init`/`Ord3`).
    pub(crate) trait_name: &'static str,
    /// A short surface-syntax hint for the redeclare-refusal message — purely diagnostic text,
    /// never parsed or otherwise load-bearing (mirrors [`PreludeTraitSeed::impl_hint`]).
    pub(crate) impl_hint: &'static str,
    /// Builds the concrete [`InstanceInfo`] this seed provides — hand-built, mirroring
    /// [`PreludeTraitSeed::prelude`]'s `fn() -> TraitInfo` shape (no allocation happens until this
    /// is actually called, so the enclosing `const` array stays a plain table of function pointers).
    pub(crate) instance: fn() -> InstanceInfo,
}

impl PreludeInstanceSeed {
    /// Per-nodule registration-pass seeding (DN-138 §5 obligation 4 — conditional-on-need,
    /// mirroring [`PreludeTraitSeed::seed_for_nodule`]'s exact textual trigger): seed this
    /// instance into `instances` **iff** `nodule.items` declares an `impl <trait_name>[...] for
    /// ...` for ANY head. This is the identical `used` test that already (conditionally) seeds the
    /// TRAIT itself — no new textual scan, and no new regression: a nodule that triggers this was
    /// ALREADY going to have a non-empty `instances` map (its own declared impl registers an entry
    /// there), so `crate::mono::is_already_monomorphic`'s `env.instances.is_empty()` fast-path
    /// invariant is unaffected for any trait-free program (DN-138 §2 fact 2).
    ///
    /// **Verify-first correction over the design note's own §5 obligation 5 wording (mitigation
    /// #14 / VR-5), found by this leaf's own oracle test
    /// (`crates/mycelium-l1/src/tests/prelude_instance_seed.rs`):** DN-138 literally reads "a file
    /// that both triggers the seed and declares the instance gets an explicit refusal" — but
    /// `lib/std/fmt.myc`/`lib/std/derive_prelude.myc` themselves are EXACTLY such files (each
    /// triggers its trait's seed AND hand-declares the very instance the seed also provides — they
    /// are the canonical bodies the seed is pinned against), and they MUST check clean. The checked
    /// distinction that makes both true at once: refuse only a genuine **coherence conflict** — an
    /// ALREADY-registered instance at this `(trait, head)` key whose `for_ty` **differs** from what
    /// this seed provides (e.g. a nodule hand-declaring `impl Show[Binary{32}] for Binary{32}`,
    /// colliding with the seed's `Binary{64}` at the same width-erased `"Binary"` head) — never an
    /// identical, coincidental self-provision (the `for_ty`s match exactly, so nothing conflicts;
    /// the already-real declaration simply wins and no seed insert is needed).
    ///
    /// # Errors
    /// A never-silent [`CheckError`] iff this nodule already registers a DIFFERENT concrete
    /// instance at the same `(trait, head)` key this seed would occupy (a genuine coherence
    /// conflict, RFC-0019 §4.5) — never for an identical (self-provided) instance.
    pub(crate) fn seed_instance_for_nodule(
        &self,
        instances: &mut BTreeMap<(String, String), InstanceInfo>,
        nodule: &Nodule,
    ) -> Result<(), CheckError> {
        let used = nodule
            .items
            .iter()
            .any(|item| matches!(item, Item::Impl(id) if id.trait_name == self.trait_name));
        if !used {
            return Ok(());
        }
        let info = (self.instance)();
        let Some(head) = type_head(&info.for_ty) else {
            // Unreachable by construction: every `PreludeInstanceSeed` in
            // `crate::checkty::PRELUDE_INSTANCE_SEEDS` seeds a concrete primitive-repr `for_ty`
            // (`Binary{64}`/`Bytes`/`Data:Bool`), never a bare type-variable — `type_head` only
            // returns `None` for `Ty::Var`/`Ty::Fn`. Refuse rather than silently drop the seed if
            // this invariant is ever violated (G2), instead of panicking on a malformed seed.
            return Err(CheckError::new(
                self.trait_name,
                format!(
                    "internal: prelude instance seed for `{}` has no concrete instance head \
                     (malformed seed — never a silent drop)",
                    self.trait_name
                ),
            ));
        };
        let key = (self.trait_name.to_owned(), head);
        if let Some(existing) = instances.get(&key) {
            if existing.for_ty == info.for_ty {
                // This nodule already legitimately provides the exact instance the seed would
                // (e.g. `lib/std/fmt.myc`'s own canonical `impl Show[Binary{64}]` body) — a
                // harmless, coincidental self-provision, not a conflict. Nothing to seed; the
                // real declaration already present wins.
                return Ok(());
            }
            // A DIFFERENT concrete type at the SAME width-erased head this seed occupies — a
            // genuine coherence conflict (RFC-0019 §4.5); refuse rather than silently letting
            // either side win.
            return Err(self.redeclare_error(&key.1));
        }
        instances.insert(key, info);
        Ok(())
    }

    /// Phylum-wide runtime-link seeding — the instance-seed analogue of
    /// [`PreludeTraitSeed::seed_for_link`]: insert this seed's fact into the merged map **once**,
    /// iff *some* nodule's already-checked [`Env`] carries it. This is what lets two or more
    /// nodules that each independently need the SAME seeded primitive instance link together
    /// without a false collision — mirrors why a prelude TRAIT is excluded from `OwnDecls.traits`'s
    /// per-nodule collision set in [`crate::checkty::PhylumEnv`]'s own doc comment; see
    /// [`Self::provides`] for the matching per-nodule-merge skip this pairs with.
    pub(crate) fn seed_instance_for_link(
        &self,
        instances: &mut BTreeMap<(String, String), InstanceInfo>,
        nodules: &[(Path, Env)],
    ) {
        let info = (self.instance)();
        let Some(head) = type_head(&info.for_ty) else {
            return;
        };
        let key = (self.trait_name.to_owned(), head);
        if nodules.iter().any(|(_, e)| e.instances.contains_key(&key)) {
            instances.insert(key, info);
        }
    }

    /// Does `key` name the `(trait, head)` fact THIS seed provides? Used by
    /// [`crate::checkty::PhylumEnv::link`]'s per-nodule instance merge loop to skip a seeded fact
    /// there (never double-insert / never falsely collide on it across nodules) — the instance
    /// analogue of `OwnDecls.traits` excluding a prelude trait name from its own per-nodule set.
    #[must_use]
    pub(crate) fn provides(&self, key: &(String, String)) -> bool {
        if key.0 != self.trait_name {
            return false;
        }
        let info = (self.instance)();
        type_head(&info.for_ty).is_some_and(|h| key.1 == h)
    }

    /// The never-silent (G2) redeclare-refusal `CheckError` for an instance-seed collision, naming
    /// the trait + head and a corrected surface-syntax hint (mirrors [`PreludeTraitSeed`]'s own
    /// trait-level redeclare message).
    fn redeclare_error(&self, head: &str) -> CheckError {
        CheckError::new(
            self.trait_name,
            format!(
                "cannot redeclare the built-in prelude instance `{}` for type-head `{head}` — its \
                 resolution fact is already seeded by the prelude (DN-138 Alt A); remove this \
                 declaration, or use `{}` if you intended a genuinely different concrete type",
                self.trait_name, self.impl_hint
            ),
        )
    }
}
