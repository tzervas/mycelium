//! The **`LlmCanonical` projection** (RFC-0021 §4.6; M-380) — a v0 prototype renderer.
//!
//! A *projection* (RFC-0021 §3.1) is a total, inspectable function from a content-addressed L1 node
//! tree to a rendered surface; identity stays the content hash, never the rendering (P4). This module
//! is the **LLM-facing canonical** projection (FR-S5): an s-expression rendering chosen for low
//! out-of-distribution token overhead and maximal regularity, intended as the machine-co-authoring
//! surface. It lives **above the kernel** (KC-3) — in the dual-intelligibility surface crate, not in
//! `mycelium-core`.
//!
//! # What this prototype demonstrates (the RFC-0021 §9 ergonomics gate / RP-4 sub-q 1)
//! `research/11` *assessed* that authoring a total, honesty-preserving projection over the closed
//! ~11-node L1 grammar is feasible at single-engineer scale (T11.4). This module turns that assessment
//! into **demonstrated, tested** evidence: it is one total `match` over every [`Node`] kind (the
//! compiler enforces totality), it preserves the honesty overlay by construction, and its tests check
//! that overlay holds. It does **not** address the *empirical* LLM-leverage gate (RP-1 / RFC-0021's
//! §9 second prompt) — that needs LLM runs and is honestly out of scope (`research/11` T11.6).
//!
//! # The honesty overlay (RFC-0021 §4.3 P1–P6), enforced here
//! - **P2 (honest tags survive):** every `Const`'s guarantee tag is rendered (`@Exact`/`@Declared`/…),
//!   and an approximate value's bound presence is surfaced (`:bound`).
//! - **P3 (`Swap` never elided):** the `Swap` node renders explicitly as `(swap! …)`, always carrying
//!   its target and policy — it can never be dropped (it is a match arm).
//! - **P1/P4:** this is a *view*; it changes no node and re-uses the kernel's `CtorRef`/`PrimRef`
//!   `#…` content-addresses as identity.
//!
//! # Honest scope of the prototype (VR-5)
//! The mapping rules here are a Rust `match`, not the *declared, dumpable rule table* RFC-0021 §4.2
//! ultimately wants (so the rules are inspectable as source, but not yet as data); and this is a
//! **read-only** projection (no `RoundTrip` parse-back — RFC-0021 §4.1 `EditCapability::ReadOnly`).
//! Both are noted as follow-ups, not claimed as done.

use mycelium_core::{
    Alt, GuaranteeStrength, Node, Payload, Repr, ScalarKind, SparsityClass, Trit, Value,
};

/// Render a closed Core IR [`Node`] as the `LlmCanonical` s-expression surface (RFC-0021 §4.6).
/// **Total** over every node kind (enforced by the exhaustive `match`) and **deterministic** (a pure
/// function of the node). Preserves the honesty overlay (P2/P3).
#[must_use]
pub fn llm_canonical(node: &Node) -> String {
    render_node(node)
}

fn render_node(n: &Node) -> String {
    match n {
        Node::Const(v) => format!("(const {})", render_value(v)),
        Node::Var(x) => x.clone(),
        Node::Let { id, bound, body } => {
            format!("(let [{id} {}] {})", render_node(bound), render_node(body))
        }
        Node::Op { prim, args } => format!("(op {prim}{})", render_args(args)),
        // P3: a Swap is ALWAYS rendered, explicitly, with its target + policy — never elided.
        Node::Swap {
            src,
            target,
            policy,
        } => format!(
            "(swap! {} :to {} :policy {})",
            render_node(src),
            render_repr(target),
            policy.as_str()
        ),
        Node::Construct { ctor, args } => format!("(make {ctor}{})", render_args(args)),
        Node::Match {
            scrutinee,
            alts,
            default,
        } => render_match(scrutinee, alts, default.as_deref()),
        Node::Lam { param, body } => format!("(fn [{param}] {})", render_node(body)),
        Node::App { func, arg } => format!("({} {})", render_node(func), render_node(arg)),
        Node::Fix { name, body } => format!("(fix {name} {})", render_node(body)),
        Node::FixGroup { defs, body } => {
            let binds: String = defs
                .iter()
                .map(|(name, def)| format!("[{name} {}]", render_node(def)))
                .collect::<Vec<_>>()
                .join(" ");
            format!("(fix-group ({binds}) {})", render_node(body))
        }
    }
}

/// Space-prefixed operand list, e.g. ` a b c` (empty for no args).
fn render_args(args: &[Node]) -> String {
    args.iter()
        .map(|a| format!(" {}", render_node(a)))
        .collect()
}

fn render_match(scrutinee: &Node, alts: &[Alt], default: Option<&Node>) -> String {
    let mut arms: Vec<String> = alts.iter().map(render_alt).collect();
    if let Some(d) = default {
        arms.push(format!("(_ {})", render_node(d)));
    }
    format!("(match {} {})", render_node(scrutinee), arms.join(" "))
}

fn render_alt(alt: &Alt) -> String {
    match alt {
        Alt::Ctor {
            ctor,
            binders,
            body,
        } => {
            let pat = if binders.is_empty() {
                format!("{ctor}")
            } else {
                format!("({ctor} {})", binders.join(" "))
            };
            format!("({pat} {})", render_node(body))
        }
        Alt::Lit { value, body } => format!("({} {})", render_value(value), render_node(body)),
    }
}

/// Render a constant value: its literal (repr + payload) plus the honesty overlay — the guarantee tag
/// always (P2), and bound presence when approximate (P6-adjacent).
fn render_value(v: &Value) -> String {
    let lit = render_payload(v.repr(), v.payload());
    let guar = guar_str(v.meta().guarantee());
    let bound = if v.meta().bound().is_some() {
        " :bound"
    } else {
        ""
    };
    format!("{lit} @{guar}{bound}")
}

fn guar_str(g: GuaranteeStrength) -> &'static str {
    match g {
        GuaranteeStrength::Exact => "Exact",
        GuaranteeStrength::Proven => "Proven",
        GuaranteeStrength::Empirical => "Empirical",
        GuaranteeStrength::Declared => "Declared",
    }
}

/// Above this width, a literal payload is summarized by its length rather than inlined element-wise.
/// `Binary{width}`/`Ternary{trits}` are only constrained `> 0` (`mycelium-core::repr`), so a `Const`
/// can carry an arbitrarily large payload; rendering every element would be O(width). The summary is
/// honest (the value is never dropped — it states its shape), mirroring the VSA-hypervector case.
const INLINE_MAX: usize = 64;

fn render_payload(repr: &Repr, payload: &Payload) -> String {
    match (repr, payload) {
        (Repr::Binary { .. }, Payload::Bits(bits)) => {
            if bits.len() > INLINE_MAX {
                format!("0b<{} bits>", bits.len())
            } else {
                let s: String = bits.iter().map(|&b| if b { '1' } else { '0' }).collect();
                format!("0b{s}")
            }
        }
        (Repr::Ternary { .. }, Payload::Trits(trits)) => {
            if trits.len() > INLINE_MAX {
                format!("<{} trits>", trits.len())
            } else {
                let s: String = trits
                    .iter()
                    .map(|t| match t {
                        Trit::Neg => '-',
                        Trit::Zero => '0',
                        Trit::Pos => '+',
                    })
                    .collect();
                format!("<{s}>")
            }
        }
        (Repr::Dense { dtype, .. }, Payload::Scalars(xs)) => {
            let s: String = xs
                .iter()
                .map(|x| format!("{x}"))
                .collect::<Vec<_>>()
                .join(" ");
            format!("[{s}]:{}", scalar_str(*dtype))
        }
        (
            Repr::Vsa {
                model, sparsity, ..
            },
            Payload::Hypervector(xs),
        ) => {
            // The hypervector content is not literal-inlined (it is high-dimensional); render its
            // shape honestly so the projection stays total and never silently drops the value.
            format!("<hv:{model}/{}{}>", xs.len(), sparsity_str(sparsity))
        }
        // A payload that does not match its repr cannot occur for a well-formed `Value` (constructed
        // through `Value::new`); render it explicitly rather than panicking (never silent).
        (r, _) => format!("<malformed-value:{}>", render_repr(r)),
    }
}

fn render_repr(r: &Repr) -> String {
    match r {
        Repr::Binary { width } => format!("Binary{{{width}}}"),
        Repr::Ternary { trits } => format!("Ternary{{{trits}}}"),
        Repr::Dense { dim, dtype } => format!("Dense{{{dim},{}}}", scalar_str(*dtype)),
        Repr::Vsa {
            model,
            dim,
            sparsity,
        } => format!("VSA{{{model},{dim}{}}}", sparsity_str(sparsity)),
        // RFC-0032 D3 (M-749): the indexed-sequence repr renders its element type and length.
        Repr::Seq { elem, len } => format!("Seq{{{},{len}}}", render_repr(elem)),
    }
}

fn scalar_str(k: ScalarKind) -> &'static str {
    match k {
        ScalarKind::F16 => "F16",
        ScalarKind::Bf16 => "BF16",
        ScalarKind::F32 => "F32",
        ScalarKind::F64 => "F64",
    }
}

fn sparsity_str(s: &SparsityClass) -> String {
    match s {
        SparsityClass::Dense => String::new(),
        SparsityClass::Sparse { max_active } => format!(",sparse<={max_active}"),
    }
}

#[cfg(test)]
mod tests {
    use super::llm_canonical;
    use mycelium_core::{
        Bound, BoundBasis, BoundKind, CtorRef, GuaranteeStrength, Meta, Node, NormKind, Payload,
        Provenance, Repr, ScalarKind, Trit, Value,
    };

    fn byte() -> Value {
        Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(vec![true, false, true, true, false, false, true, false]),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    fn trits() -> Value {
        Value::new(
            Repr::Ternary { trits: 4 },
            Payload::Trits(vec![Trit::Zero, Trit::Zero, Trit::Pos, Trit::Neg]),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    // P2 witness: an approximate value carrying a Declared guarantee + a bound.
    fn declared_dense() -> Value {
        let bound = Bound {
            kind: BoundKind::Error {
                eps: 0.5,
                norm: NormKind::Linf,
            },
            basis: BoundBasis::UserDeclared,
        };
        Value::new(
            Repr::Dense {
                dim: 2,
                dtype: ScalarKind::F32,
            },
            Payload::Scalars(vec![1.0, 2.0]),
            Meta::new(
                Provenance::Root,
                GuaranteeStrength::Declared,
                Some(bound),
                None,
                None,
                None,
            )
            .expect("well-formed meta"),
        )
        .unwrap()
    }

    /// Totality (RFC-0021 §4.2): every one of the 11 node kinds renders without panicking and yields
    /// a non-empty s-expression. The exhaustive `match` makes this true by construction; this test is
    /// the witness that the feasibility assessment (`research/11` T11.4) holds for the full grammar.
    #[test]
    fn total_over_every_node_kind() {
        let ctor = CtorRef::new(
            mycelium_core::ContentHash::parse("blake3:00ctor00").unwrap(),
            0,
        );
        let nodes: Vec<Node> = vec![
            Node::Const(byte()),
            Node::Var("x".into()),
            Node::Let {
                id: "a".into(),
                bound: Box::new(Node::Const(byte())),
                body: Box::new(Node::Var("a".into())),
            },
            Node::Op {
                prim: "bit.not".into(),
                args: vec![Node::Const(byte())],
            },
            Node::Swap {
                src: Box::new(Node::Const(byte())),
                target: Repr::Ternary { trits: 6 },
                policy: mycelium_core::ContentHash::parse("blake3:po1icy00").unwrap(),
            },
            Node::Construct {
                ctor: ctor.clone(),
                args: vec![Node::Const(byte())],
            },
            Node::Match {
                scrutinee: Box::new(Node::Const(byte())),
                alts: vec![],
                default: Some(Box::new(Node::Const(byte()))),
            },
            Node::Lam {
                param: "x".into(),
                body: Box::new(Node::Var("x".into())),
            },
            Node::App {
                func: Box::new(Node::Var("f".into())),
                arg: Box::new(Node::Const(byte())),
            },
            Node::Fix {
                name: "f".into(),
                body: Box::new(Node::Var("f".into())),
            },
            Node::FixGroup {
                defs: vec![
                    ("f".into(), Box::new(Node::Var("g".into()))),
                    ("g".into(), Box::new(Node::Var("f".into()))),
                ],
                body: Box::new(Node::Var("f".into())),
            },
        ];
        // One rule per node kind; the closed v0 grammar is exactly these 11.
        assert_eq!(nodes.len(), 11, "the v0 L1 grammar is 11 node kinds");
        for n in &nodes {
            let s = llm_canonical(n);
            assert!(!s.is_empty(), "every node kind renders non-empty: {n:?}");
        }
    }

    /// P3 (RFC-0021 §4.3): a `Swap` is rendered explicitly and never elided — its marker and policy
    /// always appear.
    #[test]
    fn swap_is_never_elided() {
        let prog = Node::Let {
            id: "a".into(),
            bound: Box::new(Node::Const(byte())),
            body: Box::new(Node::Swap {
                src: Box::new(Node::Var("a".into())),
                target: Repr::Ternary { trits: 6 },
                policy: mycelium_core::ContentHash::parse("blake3:po1icy00").unwrap(),
            }),
        };
        let s = llm_canonical(&prog);
        assert!(
            s.contains("(swap!"),
            "P3: the Swap node must be rendered: {s}"
        );
        assert!(s.contains(":to Ternary{6}"), "the target survives: {s}");
        assert!(s.contains(":policy"), "the policy reference survives: {s}");
    }

    /// P2 (RFC-0021 §4.3): the guarantee tag is part of every rendered constant, and an approximate
    /// value surfaces its bound — the honesty tag can never be silently dropped.
    #[test]
    fn guarantee_tags_survive() {
        assert!(
            llm_canonical(&Node::Const(byte())).contains("@Exact"),
            "an Exact value renders its tag"
        );
        let s = llm_canonical(&Node::Const(declared_dense()));
        assert!(
            s.contains("@Declared"),
            "a Declared value renders its tag: {s}"
        );
        assert!(
            s.contains(":bound"),
            "an approximate value surfaces its bound: {s}"
        );
    }

    /// Determinism (RFC-0021 §3.1): same node in, same surface out.
    #[test]
    fn deterministic() {
        let prog = Node::Op {
            prim: "trit.add".into(),
            args: vec![Node::Const(trits()), Node::Const(trits())],
        };
        assert_eq!(llm_canonical(&prog), llm_canonical(&prog));
    }

    /// A large `Const` payload is summarized by length, not inlined element-wise — bounded output on
    /// arbitrarily wide values, and never silently dropped (it states its shape).
    #[test]
    fn large_payloads_are_summarized() {
        let wide = Value::new(
            Repr::Binary { width: 256 },
            Payload::Bits(vec![false; 256]),
            Meta::exact(Provenance::Root),
        )
        .unwrap();
        let s = llm_canonical(&Node::Const(wide));
        assert!(s.contains("0b<256 bits>"), "wide binary is summarized: {s}");
        // A small value is still inlined verbatim (the byte fixture is 8 bits).
        assert!(llm_canonical(&Node::Const(byte())).contains("0b10110010"));
    }
}
