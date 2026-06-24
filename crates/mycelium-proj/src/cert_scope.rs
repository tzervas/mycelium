//! **`@certification` mode resolution & scoping** (M-790; RFC-0034 §6) — the declaration-site half
//! of tunable certification.
//!
//! RFC-0034 §6 says the active [`CertMode`] is **data in the
//! source** — declared by a `@certification` attribute on the `mycelium-proj.toml` manifest and/or a
//! nodule header — **not** a hidden build flag. It is resolved **most-specific-wins** over the scope
//! lattice `global ⊐ phylum ⊐ nodule`, *reusing the RFC-0012 ambient-representation + scoped-override
//! mechanism* (innermost-enclosing-wins, reified + EXPLAIN-able provenance — RFC-0012 §4.1) rather
//! than building new scoping machinery. This is the exact shape the [`mod@crate::resolve`]
//! module already gives the metadata fields (`local > manifest`, per-field
//! [`Origin`](crate::resolve::Origin)); the
//! certification mode is one more such scoped field, with one extra (most-specific) tier.
//!
//! **What is reused vs. added.** RFC-0012's mechanism is: *a declared default, scoped, with
//! innermost-wins resolution, that is reified and renderable and changes nothing about content
//! identity.* That is precisely [`resolve_mode`] below — a stack of `(scope, mode)` declarations
//! resolved most-specific-first, every result annotated with the [`CertScope`] it came from, and the
//! mode deliberately **excluded from the content hash** (it rides `Meta`, RFC-0001 §4.6 / ADR-003 —
//! exactly as RFC-0012's ambient is pure surface elaboration that never perturbs L0). No new scoping
//! algorithm is introduced; this composes the same innermost-wins fold.
//!
//! **Deferred (honest scope, VR-5).** RFC-0034 §6 also lists per-op `thaw`-style granularity and
//! per-knob overrides as **deferred** (YAGNI). They are not implemented here, and are named as
//! deferred rather than silently absent.
//!
//! ## FLAGs for maintainer ratification (M-790)
//! Two surface/semantic choices were under-specified by RFC-0034 §6 ("the exact `@certification`
//! surface syntax is sketched … its grammar lands with the surface-syntax work"). The smallest
//! defensible choice was made and is flagged here, not silently baked:
//!
//! - **FLAG-A (surface spelling).** The attribute value is the **lowercase** mode word
//!   `fast | balanced | certified` — matching the manifest's existing lowercase enum values
//!   (`kind = "phylum"`), *not* the `serde` capitalized form (`"Fast"`). Parsing is the closed,
//!   never-silent set (an unknown word is an explicit error, G2). If ratification prefers the
//!   capitalized serde spelling, [`parse_cert_mode`] is the single point to change.
//! - **FLAG-B (manifest tier = `global`/`phylum`).** The v0 single-manifest model carries **one**
//!   `@certification` declaration in `[project]`; RFC-0034 §6's distinct `global` vs `phylum` tiers
//!   are not yet separable in one manifest. This module models the manifest declaration as the
//!   [`CertScope::Phylum`] tier (a phylum *is* what a `mycelium-proj.toml` describes — `kind =
//!   "phylum"`), reserves [`CertScope::Global`] for a future workspace/global default, and resolves
//!   the full three-tier lattice so the precedence law is testable today. The separate global tier
//!   lands with the multi-manifest/workspace work (the same honest-scope deferral the
//!   [`mod@crate::resolve`] module already makes for the ancestor-nodule tier).

use mycelium_core::cert_mode::CertMode;
use mycelium_core::guarantee::GuaranteeStrength;

/// The scope a certification-mode declaration was made at — the RFC-0034 §6 lattice
/// `global ⊐ phylum ⊐ nodule`, ordered **least-specific → most-specific**. Resolution is
/// most-specific-wins, so a `Nodule` declaration overrides a `Phylum` one, which overrides `Global`.
///
/// This mirrors RFC-0012's innermost-enclosing-wins scope stack: `Global` is the outermost ambient,
/// `Nodule` the innermost. (See FLAG-B: in v0 the manifest declaration is the `Phylum` tier and
/// `Global` is reserved for a future workspace default.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CertScope {
    /// Project-/workspace-wide default — the least-specific tier (RFC-0034 §6 `global`). Reserved in
    /// v0 (FLAG-B); resolved so the precedence law holds end-to-end.
    Global,
    /// The phylum tier — a `mycelium-proj.toml` manifest's `@certification` (FLAG-B). More specific
    /// than `Global`, less than `Nodule`.
    Phylum,
    /// The nodule tier — an in-file `// @certification:` header line. The **most-specific** tier;
    /// overrides phylum and global (RFC-0034 §6).
    Nodule,
}

impl CertScope {
    /// All three scopes, least-specific → most-specific — for exhaustive iteration in tests/tooling.
    pub const ALL: [CertScope; 3] = [CertScope::Global, CertScope::Phylum, CertScope::Nodule];

    /// Specificity rank, `0` = [`Global`](CertScope::Global) (least) … `2` =
    /// [`Nodule`](CertScope::Nodule) (most). Higher wins in [`resolve_mode`]. This *is* the derived
    /// [`Ord`] (the variants are declared least→most specific); the method names the contract.
    #[must_use]
    pub fn specificity(self) -> u8 {
        match self {
            CertScope::Global => 0,
            CertScope::Phylum => 1,
            CertScope::Nodule => 2,
        }
    }

    /// A stable, lower-case label for `EXPLAIN` output (RFC-0012 renderability; G2 — never ambient).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            CertScope::Global => "global",
            CertScope::Phylum => "phylum",
            CertScope::Nodule => "nodule",
        }
    }
}

/// One `@certification` declaration: a [`CertMode`] declared at a given [`CertScope`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CertDecl {
    /// The scope the declaration was made at.
    pub scope: CertScope,
    /// The declared mode.
    pub mode: CertMode,
}

/// The resolved certification mode plus its provenance — the analogue of
/// [`Resolved`](crate::resolve::Resolved) for the certification field. Never ambient: a resolved
/// mode always names the [`CertScope`] it came from (G2 / RFC-0012 renderability).
///
/// [`Default`] is the [`defaulted`](ResolvedMode::defaulted) value — `Fast`, `source: None` — so a
/// [`ResolvedHeader`](crate::resolve::ResolvedHeader) with no `@certification` anywhere still has a
/// well-defined effective mode (the project default, RFC-0034 §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResolvedMode {
    /// The effective mode after most-specific-wins resolution.
    pub mode: CertMode,
    /// Where the winning declaration was made — `None` iff the mode is the
    /// [`CertMode::default`] fallback (no declaration at any scope).
    pub source: Option<CertScope>,
}

impl ResolvedMode {
    /// The default resolution: no declaration at any scope ⇒ the project default
    /// [`CertMode::Fast`] (RFC-0034 §5), `source: None`.
    #[must_use]
    pub fn defaulted() -> Self {
        ResolvedMode {
            mode: CertMode::default(),
            source: None,
        }
    }
}

/// Parse the `@certification` attribute value into a [`CertMode`] — the closed, never-silent set
/// `fast | balanced | certified` (FLAG-A: lowercase surface spelling). An unrecognised word is an
/// **explicit** error (G2 / VR-5), never a silent guess.
///
/// # Errors
/// Returns the offending word (caller wraps it in its own error type with a line number).
pub fn parse_cert_mode(value: &str) -> Result<CertMode, String> {
    match value.trim() {
        "fast" => Ok(CertMode::Fast),
        "balanced" => Ok(CertMode::Balanced),
        "certified" => Ok(CertMode::Certified),
        other => Err(format!(
            "unknown @certification mode {other:?} — the closed set is `fast`, `balanced`, \
             `certified` (RFC-0034 §6; G2)"
        )),
    }
}

/// The surface spelling of a mode (the inverse of [`parse_cert_mode`]) — for `EXPLAIN`/round-trip.
#[must_use]
pub fn cert_mode_word(mode: CertMode) -> &'static str {
    match mode {
        CertMode::Fast => "fast",
        CertMode::Balanced => "balanced",
        CertMode::Certified => "certified",
    }
}

/// **Resolve the active certification mode most-specific-wins** over a set of `@certification`
/// declarations (RFC-0034 §6), reusing RFC-0012's innermost-enclosing-wins fold: the declaration at
/// the highest [`CertScope::specificity`] wins; ties at a scope are not possible by construction (one
/// declaration per scope — the parser rejects duplicates upstream). With **no** declaration the
/// result is the [`CertMode::default`] fallback ([`ResolvedMode::defaulted`]).
///
/// The result carries its [`CertScope`] provenance so the choice is never ambient (G2). Resolution is
/// pure, deterministic, and order-independent (it picks the max-specificity scope, not the last
/// element).
#[must_use]
pub fn resolve_mode(decls: &[CertDecl]) -> ResolvedMode {
    decls
        .iter()
        .max_by_key(|d| d.scope.specificity())
        .map_or_else(ResolvedMode::defaulted, |winner| ResolvedMode {
            mode: winner.mode,
            source: Some(winner.scope),
        })
}

/// The `EXPLAIN` of a certification-mode resolution — the effective mode and its source scope, so the
/// active mode is never ambient (G2 / RFC-0012 renderability). Stable and deterministic.
#[must_use]
pub fn explain_mode(r: &ResolvedMode) -> String {
    let src = r.source.map_or("default", CertScope::label);
    format!("certification: {}  [{src}]", cert_mode_word(r.mode))
}

// --- cross-mode composition: the explicit, visible boundary (RFC-0034 §3.1, §6) ---

/// The **explicit, visible event** raised when a value produced under one [`CertMode`] enters a
/// computation running under a *stronger* (higher-[`depth`](CertMode::depth)) mode — RFC-0034 §3.1
/// / §6. A `fast` value composed into a `certified` computation **must not silently inherit**
/// `certified` strength it did not earn (VR-5). This struct is that boundary made data: it records
/// the producer and consumer modes and the **floored** guarantee the value is *honestly* allowed to
/// claim once it crosses in.
///
/// It is deliberately a *value*, not a panic or a log line: the boundary is surfaced to the caller as
/// an inspectable artifact (no black box, G2), the same posture RFC-0012 takes for a paradigm
/// crossing (an explicit `swap`, never an inserted conversion).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrossModeEvent {
    /// The mode the incoming value was produced under.
    pub producer: CertMode,
    /// The mode the consuming computation runs under.
    pub consumer: CertMode,
    /// The guarantee strength the value carried before crossing.
    pub incoming: GuaranteeStrength,
    /// The guarantee strength the value is honestly allowed to claim **after** crossing — floored by
    /// the *producer's* mode, never upgraded to the consumer's (VR-5). This is exactly
    /// [`CertMode::gate_guarantee`] applied with the **producer** mode: a `Fast`-produced value stays
    /// at its structural/`Declared` strength even inside a `Certified` computation.
    pub effective: GuaranteeStrength,
}

impl CrossModeEvent {
    /// Whether this crossing is a genuine **mode boundary** — the producer ran *less* certification
    /// than the consumer. Only an up-crossing (`producer.depth() < consumer.depth()`) is a boundary
    /// where a silent upgrade would be a defect; a same-or-stronger producer needs no flag.
    #[must_use]
    pub fn is_boundary(self) -> bool {
        self.producer.depth() < self.consumer.depth()
    }

    /// Whether the crossing **upgraded** the value's guarantee strength. This must **always** be
    /// `false` (VR-5): the effective strength is floored by the producer, never raised by the
    /// consumer. Provided so the never-silent-upgrade law is directly assertable.
    ///
    /// Strength is ranked `0` = strongest (`Exact`) … `3` = weakest (`Declared`), so an *upgrade* is
    /// a strictly **lower** [`rank`](GuaranteeStrength::rank) on the effective side.
    #[must_use]
    pub fn upgraded_strength(self) -> bool {
        self.effective.rank() < self.incoming.rank()
    }
}

/// Compose a value produced under `producer` into a computation running under `consumer`, surfacing
/// the cross-mode boundary as an explicit [`CrossModeEvent`] (RFC-0034 §3.1 / §6).
///
/// The effective guarantee is **floored by the producer's mode** (`producer.gate_guarantee(incoming)`)
/// — a value never gains strength it did not earn just by entering a stronger computation (VR-5). The
/// returned event makes the crossing inspectable (G2): the caller can see producer/consumer modes,
/// the incoming strength, and the honest effective strength, and can branch on
/// [`CrossModeEvent::is_boundary`].
///
/// This is the never-silent twin of RFC-0012's paradigm crossing: just as the ambient never inserts a
/// conversion (a `swap` must be written), composing across cert modes never inserts an upgrade — the
/// boundary is always an explicit event.
#[must_use]
pub fn compose(
    producer: CertMode,
    consumer: CertMode,
    incoming: GuaranteeStrength,
) -> CrossModeEvent {
    CrossModeEvent {
        producer,
        consumer,
        incoming,
        // Floored by the PRODUCER, never the consumer — the value keeps only the strength its own
        // mode established (VR-5). `gate_guarantee` is monotone-down: it never raises a strength.
        effective: producer.gate_guarantee(incoming),
    }
}
