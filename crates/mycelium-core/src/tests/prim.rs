//! White-box tests for [`crate::prim`]. Extracted from the logic file (test-layout rule, M-797).

use crate::guarantee::GuaranteeStrength;
use crate::prim::{PrimDecl, PrimParadigm, PrimSig, PrimTable, WidthRel};

fn xor() -> PrimDecl {
    PrimDecl {
        sig: PrimSig {
            operands: vec![PrimParadigm::Binary, PrimParadigm::Binary],
            result: PrimParadigm::Binary,
            width: WidthRel::Uniform,
        },
        intrinsic: GuaranteeStrength::Exact,
    }
}

#[test]
fn hash_is_well_shaped_blake3_and_name_independent() {
    let h = xor().content_hash();
    assert_eq!(h.algo(), "blake3");
    assert_eq!(h.digest().len(), 64);
    // The same declaration under two different kernel names has the same identity (ADR-003).
    let mut t = PrimTable::new();
    let a = t.insert("bit.xor", xor());
    let b = t.insert("bit.xor_alias", xor());
    assert_eq!(a, b, "identity is the signature+intrinsic, not the name");
}

#[test]
fn distinct_signatures_get_distinct_identities() {
    let not = PrimDecl {
        sig: PrimSig {
            operands: vec![PrimParadigm::Binary],
            result: PrimParadigm::Binary,
            width: WidthRel::Uniform,
        },
        intrinsic: GuaranteeStrength::Exact,
    };
    assert_ne!(
        xor().content_hash(),
        not.content_hash(),
        "different arity/paradigm ⇒ different identity"
    );
}

#[test]
fn intrinsic_is_identity_bearing() {
    // A prim whose only difference is the intrinsic guarantee is a *different* declaration —
    // the honesty tag is part of identity (so an Exact prim can never alias an Empirical one).
    let mut declared = xor();
    declared.intrinsic = GuaranteeStrength::Declared;
    assert_ne!(xor().content_hash(), declared.content_hash());
}

#[test]
fn builtins_are_present_and_resolvable() {
    let t = PrimTable::builtins();
    for name in [
        "core.id",
        "bit.not",
        "bit.and",
        "bit.or",
        "bit.xor",
        "trit.neg",
        "trit.add",
        "trit.sub",
        "trit.mul",
        "cmp.eq",
        "cmp.lt",
        "bit.add",
        "bit.sub",
        "bin.mul",
        "bin.div",
        "bin.rem",
        "bit.width_cast",
        "seq.len",
        "seq.get",
        "bytes.len",
        "bytes.get",
        "bytes.slice",
        "bytes.concat",
        "fuse_join:binary",
    ] {
        let r = t.prim_ref(name).expect("builtin registered");
        let d = t.resolve(&r).expect("ref resolves");
        assert_eq!(d.intrinsic, GuaranteeStrength::Exact);
        assert_eq!(t.intrinsic(name), Some(GuaranteeStrength::Exact));
    }
    // `entries()` is the EXPLAIN surface: one inspectable entry per builtin (RFC-0032 D1/D2
    // added cmp.eq/cmp.lt/bit.add/bit.sub to the original nine; D3/M-749 added seq.len/seq.get;
    // D4/M-750 added bytes.len/get/slice/concat; DN-41/M-798 added bit.width_cast; DN-58/M-817
    // added the `Binary` `Fuse` meet `fuse_join:binary`; RFC-0033/M-887 added `bin.mul`; RFC-0033/
    // M-888 added `bin.div`/`bin.rem`).
    assert_eq!(t.entries().len(), 24);
}

#[test]
fn build_is_deterministic() {
    // Two independent builds produce the same hashes (content-addressing is a pure function).
    assert_eq!(PrimTable::builtins(), PrimTable::builtins());
}

// Mutant-witness (prim.rs:71:9): PrimSig::arity() must return operands.len(), not 0 or 1.
// Tests against unary, binary, and zero-arity signatures to cover all three replacement constants.
#[test]
fn arity_reflects_operand_count() {
    // Zero-arity (no operands).
    let zero = PrimSig {
        operands: vec![],
        result: PrimParadigm::Any,
        width: WidthRel::Uniform,
    };
    assert_eq!(zero.arity(), 0);
    // Unary.
    let unary = PrimSig {
        operands: vec![PrimParadigm::Binary],
        result: PrimParadigm::Binary,
        width: WidthRel::Uniform,
    };
    assert_eq!(unary.arity(), 1);
    // Binary.
    let binary = PrimSig {
        operands: vec![PrimParadigm::Ternary, PrimParadigm::Ternary],
        result: PrimParadigm::Ternary,
        width: WidthRel::Uniform,
    };
    assert_eq!(binary.arity(), 2);
    // From builtins: core.id is unary, bit.xor is binary.
    let t = PrimTable::builtins();
    assert_eq!(t.get("core.id").unwrap().sig.arity(), 1);
    assert_eq!(t.get("bit.xor").unwrap().sig.arity(), 2);
}

// Mutant-witness (prim.rs:122:9): Display for PrimRef must emit a non-empty, `#`-prefixed
// string, not Ok(Default::default()) (which would emit nothing).
#[test]
fn prim_ref_display_is_hash_prefixed() {
    let t = PrimTable::builtins();
    let r = t.prim_ref("bit.xor").unwrap();
    let s = r.to_string();
    // Must start with `#` (the Unison-style prim reference spelling).
    assert!(
        s.starts_with('#'),
        "PrimRef display must start with '#': got {s:?}"
    );
    // Must be non-empty and carry the algo prefix from the hash.
    assert!(
        s.len() > 1,
        "PrimRef display must be non-trivial: got {s:?}"
    );
}

// Mutant-witness (prim.rs:191:9 and prim.rs:203:9): decl_hash and decl must return the
// actual registered entry for a known name, not always None.
#[test]
fn decl_hash_and_decl_return_entries_for_known_names() {
    let t = PrimTable::builtins();
    // decl_hash returns Some for a registered name.
    let h = t.decl_hash("bit.not");
    assert!(
        h.is_some(),
        "decl_hash must return Some for a registered prim"
    );
    // decl resolves the hash to the actual declaration.
    let d = t.decl(h.unwrap());
    assert!(d.is_some(), "decl must resolve a registered hash");
    assert_eq!(d.unwrap().intrinsic, GuaranteeStrength::Exact);
    // Unknown names return None.
    assert!(t.decl_hash("nonexistent").is_none());
}

// Mutant-witness (prim.rs:228:9 both true/false replacements): contains() must return true
// for registered names and false for unregistered names — both sides kill both replacements.
#[test]
fn contains_returns_true_iff_registered() {
    let t = PrimTable::builtins();
    assert!(
        t.contains("trit.mul"),
        "contains must be true for a registered prim"
    );
    assert!(t.contains("bit.and"));
    assert!(
        !t.contains("nonexistent"),
        "contains must be false for an unknown prim"
    );
    assert!(!t.contains(""));
}

// Mutant-witness (prim.rs:234:9 all three replacements: vec![], vec![""], vec!["xyzzy"]):
// names() must return exactly the registered kernel names, sorted — neither empty, nor
// containing blank strings, nor containing sentinel strings.
#[test]
fn names_returns_registered_sorted_names() {
    let t = PrimTable::builtins();
    let ns = t.names();
    // Exactly 24 builtins (the original 9 + RFC-0032 cmp.eq/cmp.lt/bit.add/bit.sub + D3
    // seq.len/seq.get + D4 bytes.len/get/slice/concat + DN-41 bit.width_cast + DN-58/M-817
    // fuse_join:binary + RFC-0033/M-887 bin.mul + RFC-0033/M-888 bin.div/bin.rem).
    assert_eq!(
        ns.len(),
        24,
        "names() count must match the builtin count: {ns:?}"
    );
    // Sorted (BTreeMap iteration is sorted).
    let mut sorted = ns.clone();
    sorted.sort();
    assert_eq!(ns, sorted, "names() must be in sorted order");
    // Must contain specific known names, not blank/sentinel strings.
    assert!(ns.contains(&"bit.xor"), "must contain 'bit.xor'");
    assert!(ns.contains(&"core.id"), "must contain 'core.id'");
    assert!(!ns.contains(&""), "must not contain empty string");
    assert!(!ns.contains(&"xyzzy"), "must not contain sentinel 'xyzzy'");
}
