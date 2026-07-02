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
        // RFC-0032 D4 (M-750): the byte-string repr.
        Repr::Bytes => "Bytes".to_owned(),
        // ADR-040 (M-896): the scalar-float repr renders its frozen width by name (F64-only today).
        Repr::Float { .. } => "Float{F64}".to_owned(),
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
