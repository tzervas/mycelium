//! The runnable AOT artifact (M-150 "runnable artifact for the subset"; M-151 differential target).
//!
//! Executes the **lowered A-normal form** (`mycelium-core::lower`) with a **big-step env-machine**:
//! bindings are evaluated in order into an environment, operands are looked up, primitives and swaps
//! are applied via the shared registries. This is an *independent execution path* from the M-110
//! reference interpreter (small-step substitution over the nested tree), so checking the two for
//! observable equivalence (M-151) is a real NFR-7 two-path test — it stands in for "interpreter vs
//! compiled native" until the libMLIR backend lands.
//!
//! **Full v0 calculus (M-342; RFC-0011 §4.4 Q5 closed).** [`run_core`] covers the whole calculus —
//! `Const/Var/Let/Op/Swap` plus the r3/r4 data + recursion nodes: `Construct` builds a [`Datum`],
//! `Match` selects an arm (binding constructor fields), `Lam` is a closure capturing its environment,
//! `App` applies it call-by-value, and `Fix` unfolds under a fuel clock. The three-way differential
//! (L1-eval ≡ L0-interp ≡ AOT) now spans this whole fragment. (The *native LLVM* backend stays the
//! bit/trit subset and refuses the rest with an explicit `UnsupportedNode` — VR-5; data/closure
//! codegen is the deferred MLIR→LLVM work.)
//!
//! **Stack-robust (M-347).** The machine is a **trampoline** over an *explicit heap control stack*
//! (`eval_machine`): `App`/`Match` push a continuation frame and switch blocks; a completed block
//! returns its value, unwinding the stack. So object-level recursion lives on the **heap**, and the
//! host call stack is **O(1)** — like the reference interpreter. Deep recursion is bounded by two
//! **explicit, graceful** budgets — `fuel` (Fix unfolds; time → [`EvalError::FuelExhausted`]) and a
//! control-stack depth ceiling (space → [`EvalError::DepthLimit`]) — never a host-stack abort and
//! never a hang. (Empirically: pre-trampoline this aborted at ~600 unfolds; post-trampoline it is
//! graceful — see `xtask recursion-probe`, DN-05 §1.1.) The depth ceiling is now resolved
//! **dynamically** from detected memory headroom ([`crate::budget`], DN-05 §2.4 / DN05-Q5): with the
//! control stack on the heap, the budget is a policy over memory, derived honestly and `EXPLAIN`-able,
//! with a conservative static fallback. [`run_core_with_budget`] still takes an explicit ceiling.
//!
//! **Submodule confinement (DN-21 §5 F-2):** zero `unsafe` — compiler-enforced; the crate's
//! only `unsafe` is the dynamic-linking FFI in `jit`/`bitnet`/`specialize`.
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::rc::Rc;

use mycelium_core::lower::{self, Anf, AnfAlt, Atom, Rhs};
use mycelium_core::{
    CoreValue, Datum, GuaranteeStrength, Node, PackScheme, Payload, PhysicalLayout, Repr, Value,
};
use mycelium_interp::{Budgets, EffectKind, EvalError, PrimRegistry, SwapEngine};

use crate::budget::{AutoDepthBudget, DepthBudget, DepthResolution, DEFAULT_PER_FRAME_BYTES};
use crate::pack;

/// The default fuel for the env-machine's `Fix` clock — generous; the guard is against a
/// non-productive recursion, surfaced as an explicit [`EvalError::FuelExhausted`], never a hang
/// (mirrors the reference interpreter, RFC-0007 §4.5).
const AOT_FUEL: u64 = 1_000_000;

/// Resolve the **control-stack depth** ceiling for the trampoline (M-347): the *space* analogue of
/// fuel. The machine refuses past this with an explicit [`EvalError::DepthLimit`] — a graceful limit
/// that bounds heap growth, never an OOM/abort. Resolved **dynamically** from detected memory
/// headroom ([`crate::budget`], DN-05 §2.4 / DN05-Q5): a fixed constant is too timid on a large host
/// and too bold on a small one, so the default policy derives it from `MemAvailable`/`RLIMIT_AS` with
/// a conservative static fallback. The basis is `EXPLAIN`-able ([`default_depth_budget`]).
fn resolve_max_depth() -> usize {
    AutoDepthBudget::default().resolve().max_depth
}

/// The default depth-budget resolution — the resolved ceiling **and** its `EXPLAIN`-able basis (no
/// black box, G2). Exposed for tooling/diagnostics (the xtask probe, a future `EXPLAIN`) so the
/// chosen limit and *why* are always inspectable, never an opaque magic number (DN-05 §2.4 / DN05-Q5).
pub fn default_depth_budget() -> DepthResolution {
    AutoDepthBudget::default().resolve()
}

/// A value in the AOT env-machine: a fully-evaluated [`CoreValue`] (repr value or datum), or a
/// **closure** / **recursive suspension** for the r4 function fragment. Closures capture their
/// defining environment by value (the v0 surface is first-order, so this is a finite capture). Bodies
/// are [`Rc`]-shared so closures/continuation frames don't clone the block tree.
//
// `CoreValue` is intentionally inlined (not boxed): it is the common case and on the hot evaluation
// path, so boxing every value to equalise variant sizes would add an allocation per value. The size
// asymmetry is an accepted, deliberate trade-off.
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
enum AotVal {
    /// A representation value or a datum (a normal form).
    Core(CoreValue),
    /// A lambda closure: parameter, body block, and the captured environment.
    Closure {
        param: String,
        body: Rc<Anf>,
        env: Env,
    },
    /// A `Fix` suspension: unfolds on application under the fuel clock.
    Fix {
        name: String,
        body: Rc<Anf>,
        env: Env,
    },
    /// A mutual-recursion group member (RFC-0001 r5): on application it re-binds every member name to
    /// its own suspension (so siblings can call each other) and enters `which`'s body, under the fuel
    /// clock — the env-machine analogue of the interpreter's `FixGroup` focus unfold.
    FixGroup {
        /// All members of the group, shared across the per-member suspensions.
        defs: Rc<Vec<(String, Anf)>>,
        /// Which member this suspension resolves to on application.
        which: String,
        /// The environment captured at the group's binding site.
        env: Env,
    },
}

type Env = HashMap<Atom, AotVal>;

fn lookup(env: &Env, a: &Atom) -> Result<AotVal, EvalError> {
    env.get(a).cloned().ok_or_else(|| match a {
        Atom::Named(x) => EvalError::FreeVariable(x.clone()),
        Atom::Temp(k) => EvalError::FreeVariable(format!("%{k}")),
    })
}

/// Coerce an [`AotVal`] to a [`CoreValue`], or an explicit refusal for a bare function value (a v0
/// entry returns a repr/data value, never a function — mirrors the interpreter's `FunctionResult`).
fn as_core(v: AotVal) -> Result<CoreValue, EvalError> {
    match v {
        AotVal::Core(cv) => Ok(cv),
        AotVal::Closure { .. } | AotVal::Fix { .. } | AotVal::FixGroup { .. } => {
            Err(EvalError::FunctionResult)
        }
    }
}

/// Coerce an [`AotVal`] to a representation [`Value`] (for an `Op`/`Swap` operand): a datum or a
/// function in that position is a type error the checker prevents — explicit, never a guess.
fn as_repr_value(v: AotVal) -> Result<Value, EvalError> {
    match v {
        AotVal::Core(CoreValue::Repr(rv)) => Ok(rv),
        AotVal::Core(CoreValue::Data(_)) => Err(EvalError::DataMalformed {
            why: "a primitive/swap operand reduced to a data value, not a representation value"
                .to_owned(),
        }),
        AotVal::Closure { .. } | AotVal::Fix { .. } | AotVal::FixGroup { .. } => {
            Err(EvalError::DataMalformed {
                why: "a primitive/swap operand reduced to a function value".to_owned(),
            })
        }
    }
}

/// Run a Core IR program through the AOT path to a [`CoreValue`] (the full v0 calculus — repr, data,
/// and recursion; M-342). Lowers to ANF, then evaluates with a **trampolined** environment machine
/// (an explicit heap control stack — *O(1) host stack*, M-347), an independent path from the M-110
/// small-step interpreter (the NFR-7 two-path check).
pub fn run_core(
    node: &Node,
    prims: &PrimRegistry,
    swap: &dyn SwapEngine,
) -> Result<CoreValue, EvalError> {
    run_core_with_budget(node, prims, swap, AOT_FUEL, resolve_max_depth())
}

/// [`run_core`] with an explicit `Fix`-unfold (fuel) budget and the dynamically-resolved depth ceiling.
pub fn run_core_with_fuel(
    node: &Node,
    prims: &PrimRegistry,
    swap: &dyn SwapEngine,
    fuel: u64,
) -> Result<CoreValue, EvalError> {
    run_core_with_budget(node, prims, swap, fuel, resolve_max_depth())
}

/// [`run_core`] with **both** budgets explicit (M-347): `fuel` bounds `Fix` unfolds (time), `max_depth`
/// bounds the control-stack depth (space). Each overrun is an **explicit, graceful** error
/// ([`EvalError::FuelExhausted`] / [`EvalError::DepthLimit`]) — never a hang and never a host-stack
/// abort. This is the **explicit override**: `max_depth` is whatever the caller passes; the
/// `run_core`/`run_core_with_fuel` entries instead resolve it *dynamically* from detected memory
/// headroom ([`crate::budget`], DN-05 §2.4 / DN05-Q5).
pub fn run_core_with_budget(
    node: &Node,
    prims: &PrimRegistry,
    swap: &dyn SwapEngine,
    fuel: u64,
    max_depth: usize,
) -> Result<CoreValue, EvalError> {
    // The default entry carries an *empty* effect ledger: no `alloc` effect budget is declared, so the
    // depth ceiling remains the sole space guard (identical pre-RFC-0014-§4.8 behaviour).
    run_core_with_effects(node, prims, swap, fuel, max_depth, &mut Budgets::new())
}

/// [`run_core_with_budget`] with a shared **effect-budget ledger** threaded through the env-machine
/// (RFC-0014 §4.8 — completing the deferred integration). The ledger is the *same*
/// [`mycelium_interp::Budgets`] the recovery driver consumes, and an overrun surfaces as
/// [`EvalError::EffectBudget`] — the effect sibling of `FuelExhausted`/`DepthLimit`, on the **one
/// runtime refusal channel** (RFC-0014 §8: separate named budgets, one enforcement mechanism).
///
/// v0 L0 has **no effect node** (KC-3 — no kernel hook), so the machine spends only the budgets that
/// correspond to costs it *intrinsically* incurs: a declared **`alloc`** budget is charged
/// [`DEFAULT_PER_FRAME_BYTES`] per control-stack frame, at the same frame-push site the depth ceiling
/// guards — making the `alloc` effect budget the **opt-in** sibling of the DN-05 depth ceiling. Absent
/// (the default empty ledger) ⇒ no charge ⇒ unchanged behaviour (I5: a broader bound is opt-in). The
/// `retry`/`cascade` budgets are spent by the recovery *driver* over this same ledger and channel; the
/// concurrency wave (RFC-0008) layers *per-task* ledgers on this seam.
pub fn run_core_with_effects(
    node: &Node,
    prims: &PrimRegistry,
    swap: &dyn SwapEngine,
    fuel: u64,
    max_depth: usize,
    budgets: &mut Budgets,
) -> Result<CoreValue, EvalError> {
    let top = Rc::new(lower::lower_to_anf(node));
    let mut fuel = fuel;
    let result = eval_machine(top, Env::new(), prims, swap, &mut fuel, max_depth, budgets)?;
    as_core(result)
}

/// Run a Core IR program through the AOT path to a representation [`Value`]. Convenience over
/// [`run_core`] for the repr fragment: a data result is the explicit [`EvalError::DataResult`]
/// (mirrors `Interpreter::eval`), never a silent mishandling.
pub fn run(node: &Node, prims: &PrimRegistry, swap: &dyn SwapEngine) -> Result<Value, EvalError> {
    match run_core(node, prims, swap)? {
        CoreValue::Repr(v) => Ok(v),
        CoreValue::Data(_) => Err(EvalError::DataResult),
    }
}

/// A continuation: where to bind a returned value and resume. The reified caller context.
struct Cont {
    block: Rc<Anf>,
    idx: usize,
    env: Env,
    name: Atom,
}

/// A frame on the explicit **heap** control stack — what makes the machine O(1) host stack.
// `ApplyThen` carries an inlined `AotVal` (see the note on `AotVal`); the size asymmetry with
// `Resume` is the same accepted trade-off.
#[allow(clippy::large_enum_variant)]
enum Frame {
    /// Bind the returned value to `name`, then resume `block` at `idx` in `env`.
    Resume(Cont),
    /// The returned value is a function; **apply** it to `arg`, then resume per the continuation.
    /// (The two-step shape of a `Fix` application: unfold the body to a closure, then call it.)
    ApplyThen { arg: AotVal, cont: Cont },
}

/// Enter an application `f arg` whose result should resume `ret`: push the right frame and return the
/// `(block, env)` to evaluate next. Closures call their body; a `Fix` unfolds under the fuel clock
/// (binding its name to itself) and re-applies. The depth ceiling is checked here (the only place the
/// control stack grows on a call). Applying a non-function is an explicit refusal.
fn enter_apply(
    f: AotVal,
    arg: AotVal,
    ret: Cont,
    stack: &mut Vec<Frame>,
    fuel: &mut u64,
    max_depth: usize,
    budgets: &mut Budgets,
) -> Result<(Rc<Anf>, Env), EvalError> {
    if stack.len() >= max_depth {
        return Err(EvalError::DepthLimit { limit: max_depth });
    }
    // A declared `alloc` effect budget bounds the control-stack *memory* — charged per frame at the
    // DN-05 per-frame rate, the opt-in sibling of the depth ceiling (RFC-0014 §4.8). Absent ⇒ skip
    // (the depth ceiling is the default space guard). An overrun is the unified, graceful
    // `EvalError::EffectBudget` (`?` converts via `From<EffectBudgetExhausted>`) — never an OOM.
    if budgets.remaining(&EffectKind::Alloc).is_some() {
        budgets.consume(EffectKind::Alloc, DEFAULT_PER_FRAME_BYTES)?;
    }
    match f {
        AotVal::Closure { param, body, env } => {
            stack.push(Frame::Resume(ret));
            let mut call_env = env;
            call_env.insert(Atom::Named(param), arg);
            Ok((body, call_env))
        }
        AotVal::Fix { name, body, env } => {
            *fuel = fuel.checked_sub(1).ok_or(EvalError::FuelExhausted)?;
            stack.push(Frame::ApplyThen { arg, cont: ret });
            let selfval = AotVal::Fix {
                name: name.clone(),
                body: Rc::clone(&body),
                env: env.clone(),
            };
            let mut unfold_env = env;
            unfold_env.insert(Atom::Named(name), selfval);
            Ok((body, unfold_env))
        }
        AotVal::FixGroup { defs, which, env } => {
            *fuel = fuel.checked_sub(1).ok_or(EvalError::FuelExhausted)?;
            stack.push(Frame::ApplyThen { arg, cont: ret });
            // Re-bind every member name to its own focus suspension (so a sibling call resolves the
            // whole group), then enter the focused member's body — mirrors the interpreter's
            // `FixGroup` focus unfold under the same fuel clock.
            let mut unfold_env = env.clone();
            for (member, _) in defs.iter() {
                unfold_env.insert(
                    Atom::Named(member.clone()),
                    AotVal::FixGroup {
                        defs: Rc::clone(&defs),
                        which: member.clone(),
                        env: env.clone(),
                    },
                );
            }
            let body = defs
                .iter()
                .find(|(n, _)| *n == which)
                .map(|(_, b)| Rc::new(b.clone()))
                .ok_or(EvalError::FreeVariable(which))?;
            Ok((body, unfold_env))
        }
        AotVal::Core(_) => Err(EvalError::ApplyNonFunction),
    }
}

/// Select the first-matching arm (or default) of a lowered `Match`, returning the arm's block (as a
/// fresh [`Rc`]) and the environment to evaluate it in (constructor fields bound left-to-right). No
/// match + no default is an explicit [`EvalError::NonExhaustiveMatch`].
fn select_arm(
    cv: &CoreValue,
    alts: &[AnfAlt],
    default: Option<&Anf>,
    env: &Env,
) -> Result<(Rc<Anf>, Env), EvalError> {
    for alt in alts {
        match alt {
            AnfAlt::Ctor {
                ctor,
                binders,
                body,
            } => {
                if let CoreValue::Data(d) = cv {
                    if d.ctor() == ctor {
                        if binders.len() != d.fields().len() {
                            return Err(EvalError::DataMalformed {
                                why: format!(
                                    "constructor arm binds {} of {} field(s) (WF6/WF7)",
                                    binders.len(),
                                    d.fields().len()
                                ),
                            });
                        }
                        let mut arm_env = env.clone();
                        for (binder, field) in binders.iter().zip(d.fields()) {
                            arm_env
                                .insert(Atom::Named(binder.clone()), AotVal::Core(field.clone()));
                        }
                        return Ok((Rc::new(body.clone()), arm_env));
                    }
                }
            }
            AnfAlt::Lit { value, body } => {
                if let CoreValue::Repr(rv) = cv {
                    if rv.repr() == value.repr() && rv.payload() == value.payload() {
                        return Ok((Rc::new(body.clone()), env.clone()));
                    }
                }
            }
        }
    }
    match default {
        Some(d) => Ok((Rc::new(d.clone()), env.clone())),
        None => Err(EvalError::NonExhaustiveMatch),
    }
}

/// The result of evaluating one binding's RHS: bind a value and advance, or switch to a new block
/// (a call / match descent) whose continuation is already on the stack.
// `Bind` carries an inlined `AotVal` (see the note on `AotVal`) — same accepted size trade-off.
#[allow(clippy::large_enum_variant)]
enum Step {
    Bind(Atom, AotVal),
    Switch(Rc<Anf>, Env),
}

/// The trampoline: iterate over blocks with an explicit control stack, so object-level recursion
/// uses **heap**, not the host call stack (O(1) host stack — the M-347 fix). `App`/`Match` push a
/// continuation and switch blocks; a completed block returns its result value, unwinding the stack
/// (an `ApplyThen` frame re-applies). Deep recursion is bounded by `fuel` (time) and `max_depth`
/// (space) — both explicit graceful errors, never an abort.
fn eval_machine(
    top: Rc<Anf>,
    top_env: Env,
    prims: &PrimRegistry,
    swap: &dyn SwapEngine,
    fuel: &mut u64,
    max_depth: usize,
    budgets: &mut Budgets,
) -> Result<AotVal, EvalError> {
    let mut block = top;
    let mut env = top_env;
    let mut idx = 0usize;
    let mut stack: Vec<Frame> = Vec::new();

    loop {
        if idx >= block.bindings().len() {
            // Block complete: produce its result and resume the top control-stack frame.
            let val = lookup(&env, block.result())?;
            match stack.pop() {
                None => return Ok(val),
                Some(Frame::Resume(c)) => {
                    let mut e = c.env;
                    e.insert(c.name, val);
                    block = c.block;
                    env = e;
                    idx = c.idx;
                }
                Some(Frame::ApplyThen { arg, cont }) => {
                    // The returned value is the unfolded closure; apply it to the saved arg (its
                    // result flows to `cont`, the frame enter_apply pushes).
                    let (nb, ne) =
                        enter_apply(val, arg, cont, &mut stack, fuel, max_depth, budgets)?;
                    block = nb;
                    env = ne;
                    idx = 0;
                }
            }
            continue;
        }

        // Evaluate binding `idx`. Compute an owned `Step` inside a scope that borrows `block`, so we
        // can reassign `block`/`env` afterwards without an outstanding borrow.
        let step: Step = {
            let binding = &block.bindings()[idx];
            let name = binding.name.clone();
            match &binding.rhs {
                Rhs::Const(v) => Step::Bind(name, AotVal::Core(CoreValue::Repr(v.clone()))),
                Rhs::Alias(a) => Step::Bind(name, lookup(&env, a)?),
                Rhs::Op { prim, args } => {
                    let vals: Vec<Value> = args
                        .iter()
                        .map(|a| as_repr_value(lookup(&env, a)?))
                        .collect::<Result<_, _>>()?;
                    let refs: Vec<&Value> = vals.iter().collect();
                    let f = prims
                        .get(prim)
                        .ok_or_else(|| EvalError::UnknownPrim(prim.clone()))?;
                    Step::Bind(name, AotVal::Core(CoreValue::Repr(f(prim, &refs)?)))
                }
                Rhs::Swap {
                    src,
                    target,
                    policy,
                } => {
                    let s = as_repr_value(lookup(&env, src)?)?;
                    Step::Bind(
                        name,
                        AotVal::Core(CoreValue::Repr(swap.swap(&s, target, policy)?)),
                    )
                }
                Rhs::Construct { ctor, args } => {
                    let fields: Vec<CoreValue> = args
                        .iter()
                        .map(|a| as_core(lookup(&env, a)?))
                        .collect::<Result<_, _>>()?;
                    Step::Bind(
                        name,
                        AotVal::Core(CoreValue::Data(Datum::new(ctor.clone(), fields))),
                    )
                }
                Rhs::Lam { param, body } => Step::Bind(
                    name,
                    AotVal::Closure {
                        param: param.clone(),
                        body: Rc::new(body.clone()),
                        env: env.clone(),
                    },
                ),
                Rhs::Fix { name: fname, body } => Step::Bind(
                    name,
                    AotVal::Fix {
                        name: fname.clone(),
                        body: Rc::new(body.clone()),
                        env: env.clone(),
                    },
                ),
                Rhs::FixGroup { defs, which } => Step::Bind(
                    name,
                    AotVal::FixGroup {
                        defs: Rc::new(defs.clone()),
                        which: which.clone(),
                        env: env.clone(),
                    },
                ),
                Rhs::App { func, arg } => {
                    let f = lookup(&env, func)?;
                    let a = lookup(&env, arg)?;
                    let ret = Cont {
                        block: Rc::clone(&block),
                        idx: idx + 1,
                        env: std::mem::take(&mut env),
                        name,
                    };
                    let (nb, ne) = enter_apply(f, a, ret, &mut stack, fuel, max_depth, budgets)?;
                    Step::Switch(nb, ne)
                }
                Rhs::Match {
                    scrutinee,
                    alts,
                    default,
                } => {
                    let cv = as_core(lookup(&env, scrutinee)?)?;
                    // r3 boundary (RFC-0011 §4.6): the guarantee-meet through Match is the identity
                    // only when the scrutinee is Exact; a non-Exact scrutinee is the explicit deferral
                    // (never a fabricated bound) — mirrors the reference interpreter.
                    if cv.guarantee() != GuaranteeStrength::Exact {
                        return Err(EvalError::GuaranteeMeetUnsupported {
                            scrutinee: cv.guarantee(),
                        });
                    }
                    let (arm_block, arm_env) = select_arm(&cv, alts, default.as_ref(), &env)?;
                    if stack.len() >= max_depth {
                        return Err(EvalError::DepthLimit { limit: max_depth });
                    }
                    stack.push(Frame::Resume(Cont {
                        block: Rc::clone(&block),
                        idx: idx + 1,
                        env: std::mem::take(&mut env),
                        name,
                    }));
                    Step::Switch(arm_block, arm_env)
                }
            }
        };

        match step {
            Step::Bind(name, v) => {
                env.insert(name, v);
                idx += 1;
            }
            Step::Switch(nb, ne) => {
                block = nb;
                env = ne;
                idx = 0;
            }
        }
    }
}

/// Run a Core IR program through the AOT path **with a schedule-staged packing layout** (M-251;
/// RFC-0004 §5/§8). The result is first computed by [`run`], then — for a ternary result — its
/// trits are materialized into a physical buffer **packed under `packed_as`** and **read back under
/// the recorded tag `read_as`** (the `Meta.physical` claim), and the layout is recorded on the
/// result's `Meta` (M-I5 lossless, [`Value`]'s `with_physical`).
///
/// When the tag is correct (`packed_as == read_as`) the read-back is the identity, so the result is
/// observably equal to the layout-agnostic reference (the interpreter / [`run`]) — and the M-210
/// observational-equivalence check validates. A **mislabeled** tag (`packed_as != read_as`)
/// misreads the buffer, producing a different payload that the same check rejects (NFR-7) — the E3
/// soundness property: the layout record is trusted *only because a wrong one is caught*.
///
/// Non-ternary results carry no trit-packing layout, so they pass through unchanged.
pub fn run_with_layout(
    node: &Node,
    prims: &PrimRegistry,
    swap: &dyn SwapEngine,
    packed_as: PackScheme,
    read_as: PackScheme,
) -> Result<Value, EvalError> {
    let v = run(node, prims, swap)?;
    match (v.repr(), v.payload()) {
        (Repr::Ternary { .. }, Payload::Trits(trits)) => {
            let read = pack::relayout_trits(trits, packed_as, read_as);
            let meta = v
                .meta()
                .clone()
                .with_physical(PhysicalLayout::TritPacked { scheme: read_as });
            Value::new(v.repr().clone(), Payload::Trits(read), meta)
                .map_err(|e| EvalError::Swap(e.to_string()))
        }
        _ => Ok(v),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_core::{Meta, Payload, Provenance, Repr};
    use mycelium_interp::{EffectBudget, IdentitySwapEngine};

    fn byte() -> Value {
        Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(vec![true, false, true, true, false, false, true, false]),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    #[test]
    fn runs_a_let_op_program() {
        // let a = byte in bit.not(a)
        let node = Node::Let {
            id: "a".into(),
            bound: Box::new(Node::Const(byte())),
            body: Box::new(Node::Op {
                prim: "bit.not".into(),
                args: vec![Node::Var("a".into())],
            }),
        };
        let out = run(&node, &PrimRegistry::with_builtins(), &IdentitySwapEngine).unwrap();
        let expected: Vec<bool> = match byte().payload() {
            Payload::Bits(b) => b.iter().map(|&x| !x).collect(),
            _ => unreachable!(),
        };
        assert_eq!(out.payload(), &Payload::Bits(expected));
    }

    #[test]
    fn free_variable_is_explicit() {
        let node = Node::Var("nope".into());
        assert_eq!(
            run(&node, &PrimRegistry::with_builtins(), &IdentitySwapEngine),
            Err(EvalError::FreeVariable("nope".into()))
        );
    }

    #[test]
    fn applies_a_closure_in_the_env_machine() {
        // (λx. bit.not(x)) byte  — exercises Lam capture + App + closure-body eval (M-342).
        let node = Node::App {
            func: Box::new(Node::Lam {
                param: "x".into(),
                body: Box::new(Node::Op {
                    prim: "bit.not".into(),
                    args: vec![Node::Var("x".into())],
                }),
            }),
            arg: Box::new(Node::Const(byte())),
        };
        let out = run(&node, &PrimRegistry::with_builtins(), &IdentitySwapEngine).unwrap();
        let expected: Vec<bool> = match byte().payload() {
            Payload::Bits(b) => b.iter().map(|&x| !x).collect(),
            _ => unreachable!(),
        };
        assert_eq!(out.payload(), &Payload::Bits(expected));
    }

    /// `(fix f => λx. f x) c` — unfolds forever.
    fn spin() -> Node {
        Node::App {
            func: Box::new(Node::Fix {
                name: "f".into(),
                body: Box::new(Node::Lam {
                    param: "x".into(),
                    body: Box::new(Node::App {
                        func: Box::new(Node::Var("f".into())),
                        arg: Box::new(Node::Var("x".into())),
                    }),
                }),
            }),
            arg: Box::new(Node::Const(byte())),
        }
    }

    #[test]
    fn a_nonproductive_recursion_is_an_explicit_budget_error_not_an_abort() {
        // M-347: with the trampoline the env-machine is O(1) host stack, so even at a HUGE fuel this
        // is a graceful explicit budget error (the depth ceiling or fuel — whichever first), never a
        // host-stack abort and never a hang. (Pre-trampoline, fuel this large overflowed the stack.)
        let r = run_core_with_fuel(
            &spin(),
            &PrimRegistry::with_builtins(),
            &IdentitySwapEngine,
            50_000_000,
        );
        assert!(
            matches!(
                r,
                Err(EvalError::DepthLimit { .. }) | Err(EvalError::FuelExhausted)
            ),
            "expected a graceful budget error, got {r:?}"
        );
    }

    #[test]
    fn the_depth_ceiling_is_an_explicit_graceful_error() {
        // With a small control-stack ceiling and ample fuel, deep recursion hits the *depth* budget
        // first — an explicit DepthLimit (the space analogue of fuel), never a host-stack abort.
        let r = run_core_with_budget(
            &spin(),
            &PrimRegistry::with_builtins(),
            &IdentitySwapEngine,
            1_000_000, // fuel ≫ depth, so the depth ceiling bites first
            64,
        );
        assert_eq!(r, Err(EvalError::DepthLimit { limit: 64 }));
    }

    #[test]
    fn a_declared_alloc_effect_budget_overruns_gracefully_at_runtime() {
        // RFC-0014 §4.8 (completed): the recovery `Budgets` ledger is now wired into the env-machine's
        // budget enforcement. A declared `alloc` effect budget bounds control-stack *memory* (the
        // opt-in sibling of the depth ceiling) and an overrun is the unified, graceful
        // `EvalError::EffectBudget` — the runtime-path extension of the RFC-0014 I4 bounded-overrun
        // test, on the *same* channel as `FuelExhausted`/`DepthLimit`, never an OOM/hang.
        let frames = 10u64; // allow 10 frames' worth of alloc, then the 11th frame overruns
        let mut budgets =
            Budgets::new().with(EffectBudget::Bytes(frames * DEFAULT_PER_FRAME_BYTES));
        let r = run_core_with_effects(
            &spin(),
            &PrimRegistry::with_builtins(),
            &IdentitySwapEngine,
            1_000_000, // fuel ≫ alloc budget
            1_000_000, // depth ceiling ≫ alloc budget, so the *effect* budget bites first
            &mut budgets,
        );
        match r {
            Err(EvalError::EffectBudget(e)) => {
                assert_eq!(e.kind, EffectKind::Alloc);
                assert_eq!(e.requested, DEFAULT_PER_FRAME_BYTES);
                assert_eq!(e.remaining, 0);
            }
            other => panic!("expected a graceful EffectBudget overrun, got {other:?}"),
        }
    }

    #[test]
    fn an_absent_alloc_budget_leaves_runtime_behaviour_unchanged() {
        // I5 (opt-in): the default empty ledger declares no `alloc` budget, so the env-machine charges
        // nothing and the depth ceiling remains the sole space guard — identical to pre-§4.8 behaviour.
        let r = run_core_with_effects(
            &spin(),
            &PrimRegistry::with_builtins(),
            &IdentitySwapEngine,
            1_000_000,
            64,
            &mut Budgets::new(),
        );
        assert_eq!(r, Err(EvalError::DepthLimit { limit: 64 }));
    }
}
