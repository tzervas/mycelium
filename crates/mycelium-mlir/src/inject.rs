//! In-process **hot-inject** prototype (M-341; ADR-017; ADR-016's call ABI; RFC-0004 §9 the
//! interpreted↔compiled continuum; phase-4).
//!
//! This is the *named first build step* of ADR-017, realized on the M-340 `dlopen` JIT (the
//! prototype substrate). It builds the three pieces ADR-016/017 specify:
//!
//! 1. **A hash-keyed dispatch table** ([`Image`]) — the running image holds a `ContentHash → entry`
//!    map (ADR-016's call ABI: a compiled stable component is invoked by the *content hash of the
//!    definition* it compiles). A [`call`](Image::call) **resolves to a compiled entry if present,
//!    else interprets** the definition (the continuum, RFC-0004 §9.1). A hash with neither a
//!    compiled entry nor an interpretable definition is an **explicit** [`InjectError::DispatchMiss`]
//!    — never a silent guess (G2/SC-3; ADR-017 decision 5).
//! 2. **Load-and-register injection** ([`inject`](Image::inject)) — "injecting" a recompiled
//!    definition compiles its unit (the `dlopen` JIT) and registers a *new* `hash → entry`. It
//!    **never mutates a live entry**: because identity *is* the content hash (ADR-003), a re-inject
//!    of the same definition is the same code under the same key (publish-once, idempotent), and an
//!    *edited* definition is a **new hash under a new entry** (ADR-017 decision 4). The atomicity
//!    hazard dissolves: an in-flight call to the old hash finishes on the old (still-loaded) code,
//!    while a new caller — referencing the new hash — dispatches to the new entry.
//! 3. **The recompile set by hash reachability** ([`recompile_closure`]) — editing a definition
//!    yields a new hash; its transitive *dependents* (which named the old hash) get new hashes too;
//!    everything else keeps its hash *and its already-compiled entry*. So the recompile set is
//!    **exactly the changed dependency-closure**, computed by reachability over the dependency graph
//!    — no AST/file diff (ADR-017 decision 3).
//!
//! **Verification (NFR-7).** The injected-compiled path is checked **observationally equivalent** to
//! the reference interpreter through the shared M-210 TV checker (`mycelium_cert::check`,
//! `ObservationalEquiv`) — the same checker that validates swaps and the interp↔AOT differential.
//! See `tests/inject_hotswap.rs`.
//!
//! **Scope / honesty (VR-5).** This is the *in-process* proof. A "definition/unit" here is a
//! **closed** bit/trit-subset program (the JIT's domain today, M-340) and the call boundary is
//! ADR-016's call ABI **restricted to nullary units** — the args-carrying value boundary (the
//! RFC-0001 §4.8 wire form) lands with the MLIR→LLVM backend (RFC-0004 §2). Cross-process / native
//! units and the cross-process unit format (RFC-0004 §10 OQ-3) stay deferred. What is proven *now*:
//! hash-keyed dispatch, never-silent resolution, load-and-register injection without live-entry
//! mutation, the dependency-closure recompile set, and interp≡injected-compiled equivalence.

use std::collections::{HashMap, HashSet};

use mycelium_core::{ContentHash, Node, Value};
use mycelium_interp::{EvalError, Interpreter};

use crate::jit::{compile_so, JitArtifact};
use crate::llvm::AotError;

/// How a [`ContentHash`] resolves in an [`Image`] — the inspectable/`EXPLAIN`-able dispatch decision
/// (ADR-017 decision 5: which hash resolves to which entry is queryable). Never a hidden choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    /// A compiled (injected) entry exists for this hash — the call dispatches to native code.
    Compiled,
    /// No compiled entry, but an interpretable definition is registered — the call interprets
    /// (the continuum, RFC-0004 §9.1).
    Interpreted,
    /// Neither a compiled entry nor an interpretable definition — a call would be an explicit
    /// [`InjectError::DispatchMiss`], never a guess.
    Miss,
}

/// A failure at the dispatch/injection boundary — every variant is explicit (never a silent pass or
/// a partial registration; G2/SC-3; ADR-017 decision 5). (`PartialEq` but not `Eq`: the wrapped
/// `EvalError` is only `PartialEq`.)
#[derive(Debug, Clone, PartialEq)]
pub enum InjectError {
    /// A call to a hash with no compiled entry and no interpretable definition.
    DispatchMiss(ContentHash),
    /// Compiling/loading the unit failed — no entry is registered (never a partial registration).
    /// Carries the underlying [`AotError`] (incl. a skippable `ToolchainMissing` when `clang` is
    /// absent, so callers can degrade to the interpreter rather than fail).
    Compile(AotError),
    /// The interpreter fallback refused the definition (an explicit `EvalError`, surfaced).
    Interp(EvalError),
}

impl std::fmt::Display for InjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InjectError::DispatchMiss(h) => {
                write!(
                    f,
                    "dispatch miss: no compiled entry or definition for {}",
                    h.as_str()
                )
            }
            InjectError::Compile(e) => write!(f, "unit compile/load failed: {e}"),
            InjectError::Interp(e) => write!(f, "interpreter fallback refused: {e}"),
        }
    }
}

impl std::error::Error for InjectError {}

/// The running **image**: a hash-keyed dispatch table over a compiled overlay + an interpretable
/// base (RFC-0004 §9 continuum). Definitions are registered interpret-only with [`define`](Self::define);
/// [`inject`](Self::inject) adds a compiled entry on top. A [`call`](Self::call) prefers the compiled
/// entry, else interprets — never a silent miss.
#[derive(Default)]
pub struct Image {
    /// The interpretable base: every known definition, keyed by its content hash (the continuum's
    /// safe default — ADR-009).
    defs: HashMap<ContentHash, Node>,
    /// The compiled (injected) overlay: `ContentHash → entry`. Injection registers here; a key is
    /// **published once, never overwritten** (ADR-017 decision 4 — content-addressing guarantees a
    /// re-inject under the same key is the same code).
    compiled: HashMap<ContentHash, JitArtifact>,
    /// The trusted reference interpreter for the fallback path (ADR-007).
    interp: Interpreter,
}

impl Image {
    /// An empty image with the default reference interpreter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build an image with a specific interpreter for the fallback path (e.g. one wired with a
    /// certified swap engine).
    #[must_use]
    pub fn with_interpreter(interp: Interpreter) -> Self {
        Image {
            interp,
            ..Self::default()
        }
    }

    /// Register a definition **interpret-only** under its content hash (RFC-0001 §4.6), returning the
    /// hash. Re-defining the same definition is idempotent (the same hash maps to the same node).
    /// This is the continuum's safe default: a definition runs interpreted until it is injected.
    pub fn define(&mut self, node: Node) -> ContentHash {
        let hash = node.content_hash();
        self.defs.entry(hash.clone()).or_insert(node);
        hash
    }

    /// **Inject** a recompiled definition: compile its unit (the `dlopen` JIT) and register a
    /// `hash → entry` in the dispatch table, *also* recording it as an interpretable definition.
    /// Returns the definition's content hash (the dispatch key).
    ///
    /// **Never mutates a live entry (ADR-017 decision 4).** If a compiled entry already exists for
    /// this hash, the existing entry is kept (publish-once) and no recompile happens — by
    /// content-addressing it is byte-for-byte the same code. Injecting an *edited* definition is a
    /// **new hash**, so it lands under a new key and the old entry is untouched; an in-flight call to
    /// the old hash continues to dispatch to the old, still-loaded code.
    ///
    /// A failed unit compile/load is an explicit [`InjectError::Compile`] with **no** registration
    /// (never a partial entry); when `clang` is absent it is a skippable `Compile(ToolchainMissing)`.
    pub fn inject(&mut self, node: &Node) -> Result<ContentHash, InjectError> {
        let hash = node.content_hash();
        // The definition is always interpretable (the continuum base) — record it first so a later
        // resolution can fall back even if the compiled overlay is dropped.
        self.defs
            .entry(hash.clone())
            .or_insert_with(|| node.clone());
        if self.compiled.contains_key(&hash) {
            // Publish-once: the key already holds this exact code (content-addressed). Do not
            // recompile and do not overwrite the live entry.
            return Ok(hash);
        }
        let artifact = compile_so(node).map_err(InjectError::Compile)?;
        self.compiled.insert(hash.clone(), artifact);
        Ok(hash)
    }

    /// How `hash` resolves — the `EXPLAIN`-able dispatch decision (ADR-017 decision 5).
    #[must_use]
    pub fn resolve(&self, hash: &ContentHash) -> Resolution {
        if self.compiled.contains_key(hash) {
            Resolution::Compiled
        } else if self.defs.contains_key(hash) {
            Resolution::Interpreted
        } else {
            Resolution::Miss
        }
    }

    /// Dispatch a call by content hash (ADR-016's call ABI, nullary-unit restriction). Resolves to
    /// the compiled entry if present, else interprets the registered definition; a hash with neither
    /// is an explicit [`InjectError::DispatchMiss`] (never a silent guess).
    pub fn call(&self, hash: &ContentHash) -> Result<Value, InjectError> {
        if let Some(entry) = self.compiled.get(hash) {
            return entry.call().map_err(InjectError::Compile);
        }
        if let Some(node) = self.defs.get(hash) {
            return self.interp.eval(node).map_err(InjectError::Interp);
        }
        Err(InjectError::DispatchMiss(hash.clone()))
    }

    /// Whether a compiled (injected) entry exists for `hash`.
    #[must_use]
    pub fn is_injected(&self, hash: &ContentHash) -> bool {
        self.compiled.contains_key(hash)
    }

    /// The number of compiled (injected) entries — the dispatch table never shrinks on a re-inject
    /// of an existing hash (publish-once), so a stable count witnesses the no-overwrite property.
    #[must_use]
    pub fn injected_count(&self) -> usize {
        self.compiled.len()
    }

    /// The number of known (interpretable) definitions.
    #[must_use]
    pub fn defined_count(&self) -> usize {
        self.defs.len()
    }
}

/// The **recompile set** of a change, by hash reachability (ADR-017 decision 3 — no AST/file diff).
///
/// `deps` is the dependency graph: `deps[h]` is the set of hashes that definition `h` *directly
/// references*. `changed` is the set of edited definitions (each already a *new* hash). The result is
/// the closure that must be recompiled: every `changed` definition **plus** every definition that
/// transitively depends on a changed one (its callers, by reverse reachability) — because each such
/// dependent named a now-changed hash and is therefore itself a new definition. Everything outside
/// the set keeps its hash and its already-compiled entry (never re-injected).
///
/// Pure and deterministic; depends only on the hash graph, never on definition bodies.
#[must_use]
pub fn recompile_closure(
    deps: &HashMap<ContentHash, Vec<ContentHash>>,
    changed: &[ContentHash],
) -> HashSet<ContentHash> {
    // Invert the dependency edges to reverse edges (dependency → its dependents/callers).
    let mut dependents: HashMap<&ContentHash, Vec<&ContentHash>> = HashMap::new();
    for (h, references) in deps {
        for r in references {
            dependents.entry(r).or_default().push(h);
        }
    }
    // BFS the reverse graph from every changed hash; the closure includes the changed set itself.
    let mut closure: HashSet<ContentHash> = HashSet::new();
    let mut frontier: Vec<ContentHash> = changed.to_vec();
    while let Some(h) = frontier.pop() {
        if !closure.insert(h.clone()) {
            continue; // already visited
        }
        if let Some(callers) = dependents.get(&h) {
            for c in callers {
                if !closure.contains(*c) {
                    frontier.push((*c).clone());
                }
            }
        }
    }
    closure
}

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_core::{Meta, Payload, Provenance, Repr};

    fn binary(bits: Vec<bool>) -> Value {
        let width = bits.len() as u32;
        Value::new(
            Repr::Binary { width },
            Payload::Bits(bits),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    /// `not(<bits>)` — a closed bit-subset program (the JIT's domain).
    fn not_prog(bits: Vec<bool>) -> Node {
        Node::Op {
            prim: "bit.not".into(),
            args: vec![Node::Const(binary(bits))],
        }
    }

    fn h(s: &str) -> ContentHash {
        ContentHash::parse(&format!("blake3:{s}")).unwrap()
    }

    #[test]
    fn defined_definition_resolves_to_interpreted_and_calls() {
        // No toolchain needed: an interpret-only definition dispatches to the interpreter.
        let mut img = Image::new();
        let prog = not_prog(vec![true, false, true, true]);
        let hash = img.define(prog);
        assert_eq!(img.resolve(&hash), Resolution::Interpreted);
        let v = img.call(&hash).expect("interpreted call runs");
        assert_eq!(v.payload(), &Payload::Bits(vec![false, true, false, false]));
    }

    #[test]
    fn an_unknown_hash_is_an_explicit_dispatch_miss() {
        let img = Image::new();
        let miss = h("deadbeef");
        assert_eq!(img.resolve(&miss), Resolution::Miss);
        assert_eq!(img.call(&miss), Err(InjectError::DispatchMiss(miss)));
    }

    #[test]
    fn different_programs_get_different_hashes() {
        // The injection key is the content hash — an edit is a new hash (ADR-017 decision 4).
        let a = not_prog(vec![true, false]).content_hash();
        let b = not_prog(vec![false, false]).content_hash();
        assert_ne!(a, b);
    }

    #[test]
    fn recompile_closure_is_the_reverse_reachable_set() {
        // Graph: main -> helper -> leaf ; other (independent).
        let (main, helper, leaf, other) = (h("main"), h("helper"), h("leaf"), h("other"));
        let mut deps: HashMap<ContentHash, Vec<ContentHash>> = HashMap::new();
        deps.insert(main.clone(), vec![helper.clone()]);
        deps.insert(helper.clone(), vec![leaf.clone()]);
        deps.insert(leaf.clone(), vec![]);
        deps.insert(other.clone(), vec![]);

        // Editing `leaf` must recompile leaf + helper + main, but not the independent `other`.
        let set = recompile_closure(&deps, std::slice::from_ref(&leaf));
        assert_eq!(
            set,
            HashSet::from([leaf.clone(), helper.clone(), main.clone()])
        );
        assert!(!set.contains(&other));

        // Editing a leaf with no dependents recompiles only itself.
        assert_eq!(
            recompile_closure(&deps, std::slice::from_ref(&other)),
            HashSet::from([other])
        );
    }

    #[test]
    fn recompile_closure_terminates_on_a_cycle() {
        // Mutual reference (a hash cycle) must not loop forever — closure is still finite.
        let (a, b) = (h("a"), h("b"));
        let mut deps: HashMap<ContentHash, Vec<ContentHash>> = HashMap::new();
        deps.insert(a.clone(), vec![b.clone()]);
        deps.insert(b.clone(), vec![a.clone()]);
        assert_eq!(
            recompile_closure(&deps, std::slice::from_ref(&a)),
            HashSet::from([a, b])
        );
    }
}
