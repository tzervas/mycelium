//! The guarantee-strength lattice (RFC-0001 §3.4/§4.7; `guarantee.schema.json`).
//!
//! `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`, ordered strongest-to-weakest. The `meet`
//! (weakest-wins) composition and its algebraic laws are property-tested under **M-102**; this
//! module fixes the value space and the ordering they build on.

/// How trustworthy a value's representation/bound is. Honesty is monotone-downward: an operation's
/// result is never stronger than its weakest input (the `meet`, M-102).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuaranteeStrength {
    /// No approximation; `bound == None` (M-I1).
    Exact,
    /// Approximate with a machine-checked bound (basis `ProvenThm`).
    Proven,
    /// Approximate with an empirically-validated bound (basis `EmpiricalFit`).
    Empirical,
    /// Approximate with a user-asserted, unvalidated bound (basis `UserDeclared`); always flagged.
    Declared,
}

impl GuaranteeStrength {
    /// Lattice rank, `0` = strongest (`Exact`) … `3` = weakest (`Declared`). The `meet` of two
    /// strengths is the one with the larger rank (M-102 will expose `meet` and prove its laws).
    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            GuaranteeStrength::Exact => 0,
            GuaranteeStrength::Proven => 1,
            GuaranteeStrength::Empirical => 2,
            GuaranteeStrength::Declared => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GuaranteeStrength::{Declared, Empirical, Exact, Proven};

    #[test]
    fn ranks_are_strongest_to_weakest() {
        assert!(Exact.rank() < Proven.rank());
        assert!(Proven.rank() < Empirical.rank());
        assert!(Empirical.rank() < Declared.rank());
    }
}
