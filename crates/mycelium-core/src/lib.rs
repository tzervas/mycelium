//! `mycelium-core` — Core IR: Value<Repr,Meta>, the guarantee lattice, content-addressing (RFC-0001 r2). Types land in M-101/M-102/M-103/M-104.
//!
//! Skeleton crate (M-091). No public API yet; the design lives in the RFCs and the
//! ratified data-contract schemas (`docs/spec/schemas/`). Implementation lands per the
//! crate's tracked issue.

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles() {
        // Smoke test: the skeleton builds and links under the pinned MSRV (1.92).
        let answer: u32 = 2 + 2;
        assert_eq!(answer, 4);
    }
}
