//! The **L1 fuel-guarded evaluator** (RFC-0007 §4.6): a big-step environment machine mirroring
//! the M-110 reference interpreter's contract — explicit errors only, a step budget instead of a
//! termination assumption (CakeML-style clocked semantics, T3.4), and the *same* trusted
//! primitive registry and swap engine the L0 interpreter and the AOT path dispatch through, so
//! "two execution paths" can never mean "two semantics" (NFR-7).
//!
//! Programs **inside** the evaluation-complete fragment also elaborate to L0
//! ([`crate::elab::elaborate`]) and must agree with this evaluator on the observable
//! (`repr + payload + guarantee`) — the §4.6 differential obligation, validated through the M-210
//! shared checker (`tests/differential.rs`). Programs **outside** the fragment (recursion, match,
//! data values, dynamic guarantee indices) run *only* here.
//!
//! Honesty:
//! - exhausting the step budget is an explicit [`L1Error::FuelExhausted`], never a hang — and
//!   "checked total" means precisely "terminates for every sufficiently large fuel" (§4.5);
//! - a guarantee index `@ g` is checked **dynamically against `Meta`** (stage 0, RFC-0007 §4.3):
//!   asserting `@ g` on a value whose tag is weaker than `g` is an explicit
//!   [`L1Error::GuaranteeTooWeak`] — the assertion never upgrades the tag (VR-5), and a passing
//!   check leaves the value's own (possibly stronger) tag untouched;
//! - states the typechecker proves unreachable still fail as explicit [`L1Error::Stuck`] errors,
//!   never panics or defaults (S5/G2);
//! - [`Evaluator::call`] runs the recursive evaluation on a deep worker stack (256 MiB, lazily
//!   committed) via [`mycelium_stack::with_deep_stack`], so the **explicit depth budget** — not
//!   the caller's thread stack — is always what bounds a pathological input. Raising
//!   [`DEFAULT_DEPTH`] via [`Evaluator::with_depth`] is now host-stack-safe: the budget refuses
//!   cleanly well before the physical stack limit (banked guard 4; see `DEFAULT_DEPTH`). The
//!   worker stack is the transitional Rust-host adapter; the explicit budget is the portable
//!   primitive that will carry to the self-hosted Mycelium frontend (RFC-0007 §4.5/§4.6).

use mycelium_cert::BinaryTernarySwapEngine;
use mycelium_core::{CoreValue, DataRegistry, Datum, GuaranteeStrength, Value};
use mycelium_interp::{EvalError as KernelError, PrimRegistry, SwapEngine};

use crate::ast::{Expr, Literal, Pattern, Strength};
use crate::checkty::{prim_kernel_name, Env};
use crate::elab::{lit_value, policy_name_ref, type_repr, ElabError};

/// An L1 runtime value: an L0 representation value, or a constructed datum. Data values are
/// immutable and acyclic by construction — a `Construct` value can only contain values that
/// existed before it (RFC-0007 §4.7).
#[derive(Debug, Clone, PartialEq)]
pub enum L1Value {
    /// An L0 value (`repr + payload + Meta`).
    Repr(Value),
    /// A saturated constructor application (W6).
    Data {
        /// The data type's name (v0 keys the registry by name; RFC-0007 §4.2).
        ty: String,
        /// The constructor's name.
        ctor: String,
        /// The constructor's field values, in declaration order.
        fields: Vec<L1Value>,
    },
}

impl L1Value {
    /// The underlying L0 value, if this is a representation value.
    #[must_use]
    pub fn as_repr(&self) -> Option<&Value> {
        match self {
            L1Value::Repr(v) => Some(v),
            L1Value::Data { .. } => None,
        }
    }

    /// Project this L1 value onto the L0 [`CoreValue`] domain, resolving each constructor's
    /// name-keyed identity (`ty`/`ctor`) to its content-addressed `#T#i` [`mycelium_core::CtorRef`]
    /// through `registry` — the **same** registry the elaborator built (RFC-0011 §4.3). This is the
    /// bridge that makes the M-210 differential meaningful on the data fragment: an L1-eval result
    /// and an elaborate→L0-interp result become comparable *as the same L0 value* (NFR-7). The data
    /// guarantee is the meet-summary [`Datum::new`] computes from the fields, identical on both
    /// paths. Returns `None` if a constructor is not in the registry (outside the r3 fragment).
    #[must_use]
    pub fn to_core(&self, env: &crate::checkty::Env, registry: &DataRegistry) -> Option<CoreValue> {
        match self {
            L1Value::Repr(v) => Some(CoreValue::Repr(v.clone())),
            L1Value::Data { ty, ctor, fields } => {
                let decl = env.types.get(ty)?;
                let index = decl.ctors.iter().position(|c| c.name == *ctor)?;
                let ctor_ref = registry.ctor_ref(ty, u32::try_from(index).ok()?)?;
                let core_fields = fields
                    .iter()
                    .map(|f| f.to_core(env, registry))
                    .collect::<Option<Vec<_>>>()?;
                Some(CoreValue::Data(Datum::new(ctor_ref, core_fields)))
            }
        }
    }
}

/// Why L1 evaluation could not produce a value — always explicit (S5/G2).
#[derive(Debug, Clone, PartialEq)]
pub enum L1Error {
    /// The step budget ran out — the non-termination guard (RFC-0007 §4.5/§4.6).
    FuelExhausted,
    /// The recursion-depth budget ran out. This is the **explicit semantic ceiling** (banked guard
    /// 4; see [`DEFAULT_DEPTH`]): the evaluator recurses on the deep worker stack
    /// ([`mycelium_stack`]), so the budget — not a host-stack overflow — is always what stops a
    /// pathological input. Raise with [`Evaluator::with_depth`]; the host stack will not be the
    /// limit.
    DepthExceeded {
        /// The configured depth budget.
        limit: u32,
    },
    /// The trusted kernel (prim registry / swap engine) refused — the refusal is forwarded
    /// verbatim, never softened.
    Kernel(KernelError),
    /// A dynamic guarantee-index check failed: the asserted `@ g` is *stronger* than the value's
    /// actual tag — an assertion may only weaken, never upgrade (VR-5; RFC-0007 §4.3).
    GuaranteeTooWeak {
        /// The function in which the assertion appears.
        site: String,
        /// The asserted strength.
        asserted: Strength,
        /// The value's actual (weaker) strength.
        actual: GuaranteeStrength,
    },
    /// A construct the v0 evaluator does not support (`wild`, `spore`, bare-integer/list
    /// literals…) — refused with its reason, mirroring the typechecker's refusals.
    Unsupported {
        /// The function in which the construct appears.
        site: String,
        /// What was refused, and why.
        what: String,
    },
    /// An evaluation state the typechecker proves unreachable (unknown name, non-exhaustive
    /// match, arity mismatch…). Reported explicitly rather than panicking, so a checker bug can
    /// never become silent misbehavior.
    Stuck {
        /// The function in which evaluation got stuck.
        site: String,
        /// What went wrong.
        why: String,
    },
}

impl core::fmt::Display for L1Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            L1Error::FuelExhausted => write!(f, "evaluation exceeded its step budget"),
            L1Error::DepthExceeded { limit } => write!(
                f,
                "evaluation exceeded its recursion-depth budget ({limit}) — explicit by design \
                 (raise with `Evaluator::with_depth`; the host stack is not the limit)"
            ),
            L1Error::Kernel(e) => write!(f, "kernel refusal: {e}"),
            L1Error::GuaranteeTooWeak {
                site,
                asserted,
                actual,
            } => write!(
                f,
                "in `{site}`: asserted `@ {asserted:?}` but the value's tag is {actual:?} — an \
                 annotation may only weaken (VR-5)"
            ),
            L1Error::Unsupported { site, what } => write!(f, "in `{site}`: {what}"),
            L1Error::Stuck { site, why } => write!(
                f,
                "in `{site}`: stuck — {why} (the typechecker should have refused this program)"
            ),
        }
    }
}

impl std::error::Error for L1Error {}

impl From<KernelError> for L1Error {
    fn from(e: KernelError) -> Self {
        L1Error::Kernel(e)
    }
}

/// The surface strength keyword's kernel lattice point.
#[must_use]
pub fn strength_of(s: Strength) -> GuaranteeStrength {
    match s {
        Strength::Exact => GuaranteeStrength::Exact,
        Strength::Proven => GuaranteeStrength::Proven,
        Strength::Empirical => GuaranteeStrength::Empirical,
        Strength::Declared => GuaranteeStrength::Declared,
    }
}

/// Default step budget — mirrors the reference interpreter's (M-110).
const DEFAULT_FUEL: u64 = 1_000_000;

/// Default recursion-depth budget — conservative enough for an unoptimized (debug) build.
///
/// [`Evaluator::call`] runs the recursive evaluation on a deep worker stack (256 MiB, lazily
/// committed, via [`mycelium_stack::with_deep_stack`]), so this budget is the **always-binding
/// semantic ceiling** (banked guard 4) — not a stand-in for the host stack. Deep but terminating
/// programs can safely raise it via [`Evaluator::with_depth`]; the host stack will not be the
/// limit. Default is 64 — conservative by design and unchanged. A raised budget refuses cleanly
/// once it trips; the worker stack is the transitional Rust-host adapter (see
/// [`mycelium_stack`]) and is expected to disappear when the frontend self-hosts (the budget
/// carries to the Mycelium-native clocked-computation model; RFC-0007 §4.5/§4.6).
///
/// **Grounding (measured, not guessed).** The 256 MiB worker stack is the same one the checker
/// and elaborator use. The evaluator's `eval` frame is smaller than the checker's (~10.9 KiB):
/// it carries a `u64` fuel counter, a `u32` depth counter, a `&str` site, a `&mut Vec<…>` scope
/// pointer, and a `&Expr` — roughly 2–4 KiB in a debug build. At ~4 KiB/frame the 256 MiB
/// stack supports **~65,000** levels physically; at ~2 KiB/frame **~130,000**. The default
/// budget (64) is therefore a **~1,000× safety margin** below the physical ceiling, and raising
/// it to 4,096 (matching the checker) is safe with ample headroom. An in-process measurement
/// of the *clean-DepthExceeded* property is the regression guard; the physical ceiling estimate
/// is `Empirical` (frame size varies with the Rust optimizer and the IR structure).
pub(crate) const DEFAULT_DEPTH: u32 = 64;

/// The tunable **budgets** of an [`Evaluator`] — the step (`fuel`) and recursion-depth guards — as
/// a single options struct, an alternative to threading the fluent [`Evaluator::with_fuel`] /
/// [`Evaluator::with_depth`] chain. Applied via [`Evaluator::with_opts`]; the fluent setters stay.
///
/// Only the `Copy` budget knobs live here: the *engines* (`PrimRegistry`, `Box<dyn SwapEngine>`)
/// are not part of `EvaluatorOpts` — they are not `Clone`/`Default` and stay set through
/// [`Evaluator::with_engines`], so this struct is a plain, copyable, defaultable bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvaluatorOpts {
    /// The step budget (as [`Evaluator::with_fuel`]). [`Default`] is `DEFAULT_FUEL`.
    pub fuel: u64,
    /// The recursion-depth budget (as [`Evaluator::with_depth`]). [`Default`] is `DEFAULT_DEPTH`.
    /// Evaluation runs on the deep worker stack ([`mycelium_stack`]), so a raised budget is
    /// host-stack-safe — the budget, not the host stack, is the ceiling.
    pub depth: u32,
}

/// The defaults mirror [`Evaluator::new`] exactly — `DEFAULT_FUEL` / `DEFAULT_DEPTH` — so
/// `Evaluator::new(env).with_opts(EvaluatorOpts::default())` is a no-op (the budgets are unchanged).
impl Default for EvaluatorOpts {
    fn default() -> Self {
        EvaluatorOpts {
            fuel: DEFAULT_FUEL,
            depth: DEFAULT_DEPTH,
        }
    }
}

impl EvaluatorOpts {
    /// Set the step budget (builder-style), leaving `depth` untouched.
    #[must_use]
    pub fn fuel(mut self, fuel: u64) -> Self {
        self.fuel = fuel;
        self
    }

    /// Set the recursion-depth budget (builder-style), leaving `fuel` untouched.
    #[must_use]
    pub fn depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }
}

/// The L1 evaluator over a checked [`Env`]. Construction wires the same trusted engines the
/// L0 paths use: the built-in prim registry and the certified binary↔ternary swap engine
/// (M-120/M-210) — override with [`Evaluator::with_engines`] for tests or extensions.
///
/// [`Evaluator::call`] runs the recursive pass on a deep worker stack (see [`DEFAULT_DEPTH`]);
/// the swap engine must be `Send + Sync` so `&Evaluator` can be shared across the scoped worker
/// thread (all built-in engines are `Copy`, hence `Send + Sync`).
pub struct Evaluator<'e> {
    env: &'e Env,
    prims: PrimRegistry,
    swap: Box<dyn SwapEngine + Send + Sync>,
    fuel: u64,
    depth: u32,
}

impl<'e> Evaluator<'e> {
    /// An evaluator over `env` with the trusted default engines and the default budgets.
    #[must_use]
    pub fn new(env: &'e Env) -> Self {
        Evaluator {
            env,
            prims: PrimRegistry::with_builtins(),
            swap: Box::new(BinaryTernarySwapEngine),
            fuel: DEFAULT_FUEL,
            depth: DEFAULT_DEPTH,
        }
    }

    /// Replace the prim registry and swap engine. The swap engine must be `Send + Sync` (all
    /// built-in engines are `Copy`, hence `Send + Sync`; a custom engine for tests likewise).
    #[must_use]
    pub fn with_engines(
        mut self,
        prims: PrimRegistry,
        swap: Box<dyn SwapEngine + Send + Sync>,
    ) -> Self {
        self.prims = prims;
        self.swap = swap;
        self
    }

    /// Override the step budget.
    #[must_use]
    pub fn with_fuel(mut self, fuel: u64) -> Self {
        self.fuel = fuel;
        self
    }

    /// Override the recursion-depth budget. Evaluation runs on the deep worker stack
    /// ([`mycelium_stack`]), so a raised budget is host-stack-safe — the budget is the ceiling,
    /// not the host stack.
    #[must_use]
    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    /// Apply a budget [`EvaluatorOpts`] in one call — equivalent to
    /// `self.with_fuel(opts.fuel).with_depth(opts.depth)`. Additive convenience; the engines are
    /// untouched (configure those with [`Evaluator::with_engines`]).
    #[must_use]
    pub fn with_opts(self, opts: EvaluatorOpts) -> Self {
        self.with_fuel(opts.fuel).with_depth(opts.depth)
    }

    /// Call function `name` with `args`, big-step, under the configured budgets. The result
    /// honors the signature's dynamic guarantee index, if any (RFC-0007 §4.3).
    ///
    /// The recursive evaluation runs on a deep worker stack (256 MiB, lazily committed) via
    /// [`mycelium_stack::with_deep_stack`], so the **explicit [`DEFAULT_DEPTH`] budget** — not
    /// the caller's thread stack — is always the bound. The host stack never overflows for any
    /// budget value: [`L1Error::DepthExceeded`] is always what trips first (banked guard 4). Cost:
    /// one worker-thread spawn per call (~tens of µs); shallow programs touch only a few stack
    /// pages (lazily committed). The worker stack is the transitional Rust-host adapter; the
    /// budget is the portable primitive for the future self-hosted frontend.
    pub fn call(&self, name: &str, args: Vec<L1Value>) -> Result<L1Value, L1Error> {
        // Run the recursive evaluation on the deep worker stack so the explicit depth budget —
        // not the caller's thread stack — is the bound for any budget value. The closure captures
        // `&self`; this is safe because `Evaluator: Sync` (all fields are `Sync`: `&Env`,
        // `PrimRegistry` — a `BTreeMap<String, fn(…)>` — and `Box<dyn SwapEngine + Send + Sync>`).
        mycelium_stack::with_deep_stack(|| {
            let mut fuel = self.fuel;
            self.invoke(&mut fuel, self.depth, name, args)
        })
    }

    /// One function invocation: bind parameters, evaluate the body, check the return index.
    fn invoke(
        &self,
        fuel: &mut u64,
        depth: u32,
        name: &str,
        args: Vec<L1Value>,
    ) -> Result<L1Value, L1Error> {
        let Some(fd) = self.env.fns.get(name) else {
            return Err(L1Error::Stuck {
                site: name.to_owned(),
                why: format!("unknown function `{name}`"),
            });
        };
        if fd.sig.value_params.len() != args.len() {
            return Err(L1Error::Stuck {
                site: name.to_owned(),
                why: format!(
                    "`{name}` takes {} argument(s), got {}",
                    fd.sig.value_params.len(),
                    args.len()
                ),
            });
        }
        let mut scope: Vec<(String, L1Value)> = fd
            .sig
            .value_params
            .iter()
            .map(|p| p.name.clone())
            .zip(args)
            .collect();
        let result = self.eval(fuel, depth, name, &mut scope, &fd.body)?;
        if let Some(g) = fd.sig.ret.guarantee {
            self.assert_guarantee(name, &result, g)?;
        }
        Ok(result)
    }

    /// Big-step evaluation of `e` under `scope`. Every node costs one unit of fuel, so an
    /// unproductive recursion is an explicit [`L1Error::FuelExhausted`], never a hang.
    ///
    /// **Depth is charged per AST node, not per call frame (A4-03).** `eval` recurses on the host
    /// stack for *every* sub-expression — an operand of an `App`, the bound of a `Let`, an `if`
    /// branch — not only at a function `invoke`. The depth budget is a *host-stack* guard (see
    /// [`L1Error::DepthExceeded`]), so it must count exactly the recursion that consumes host
    /// stack: a deeply **nested expression** (e.g. `not(not(… not(x) …))`) overflows the stack just
    /// as a deep call chain does, and charging only at `invoke` would leave it unguarded. The
    /// honest consequence is that [`DEFAULT_DEPTH`] = 64 is a *nesting* ceiling, not a call-depth
    /// ceiling: an expression whose AST is more than ~64 nodes deep along any single path is
    /// refused with an explicit [`L1Error::DepthExceeded`] even if it makes no recursive call.
    /// This is a deliberate over-approximation in favor of the termination/no-crash guarantee
    /// (S5/G2) — raise the budget via [`Evaluator::with_depth`] when a legitimately deep but
    /// terminating expression needs it (the host stack is not the limit; see [`DEFAULT_DEPTH`]).
    /// (`for`-folds walk their spine
    /// iteratively and so are *not* subject to this ceiling per element — see [`Self::eval_for`].)
    fn eval(
        &self,
        fuel: &mut u64,
        depth: u32,
        site: &str,
        scope: &mut Vec<(String, L1Value)>,
        e: &Expr,
    ) -> Result<L1Value, L1Error> {
        *fuel = fuel.checked_sub(1).ok_or(L1Error::FuelExhausted)?;
        // Per-node (not per-call-frame) on purpose: this counts host-stack recursion, which a wide
        // *and* a deep AST both incur. See the method doc for why the per-node charge is the safe
        // choice and what the resulting 64-node nesting ceiling means (A4-03).
        let depth = depth
            .checked_sub(1)
            .ok_or(L1Error::DepthExceeded { limit: self.depth })?;
        match e {
            // RFC-0032 D4 (M-750): `0x…` byte-string literals share the `lit_value` lowering with the
            // binary/ternary repr literals (all are context-free repr literals).
            Expr::Lit(l @ (Literal::Bin(_) | Literal::Trit(_) | Literal::Bytes(_))) => Ok(
                L1Value::Repr(lit_value(site, l).map_err(|e| unsupported(site, &e))?),
            ),
            // RFC-0032 D3 (M-749): a list literal `[e1, …]` evaluates to a `Repr::Seq` value. Each
            // element is evaluated to a repr value; the element repr (from the first) anchors the
            // descriptor. The checker has already verified homogeneity; the `Value::new`
            // well-formedness check is the never-silent final guard (a heterogeneous/malformed seq is
            // refused, never silently built — G2). A non-repr element (a data value) is an explicit
            // refusal. An empty `[]` has no element repr at eval time (its width came from an
            // ascription the value form does not carry) — refused, never silently defaulted.
            Expr::Lit(Literal::List(elems)) => {
                let mut vals = Vec::with_capacity(elems.len());
                for el in elems {
                    let v = self.eval(fuel, depth, site, scope, el)?;
                    match v.as_repr() {
                        Some(rv) => vals.push(rv.clone()),
                        None => {
                            return Err(L1Error::Unsupported {
                                site: site.to_owned(),
                                what: "a list literal element is not a representation value — a \
                                       v0 `Seq` is built from repr elements only (RFC-0032 D3)"
                                    .to_owned(),
                            })
                        }
                    }
                }
                let Some(first) = vals.first() else {
                    return Err(L1Error::Unsupported {
                        site: site.to_owned(),
                        what: "an empty list literal `[]` has no element repr to anchor the `Seq` \
                               descriptor at eval (RFC-0032 D3)"
                            .to_owned(),
                    });
                };
                let elem = first.repr().clone();
                let len = u32::try_from(vals.len()).map_or(u32::MAX, |n| n);
                let seq = mycelium_core::Value::new(
                    mycelium_core::Repr::Seq {
                        elem: Box::new(elem),
                        len,
                    },
                    mycelium_core::Payload::Seq(vals),
                    mycelium_core::Meta::exact(mycelium_core::Provenance::Root),
                )
                .map_err(|e| L1Error::Stuck {
                    site: site.to_owned(),
                    why: format!("malformed sequence literal: {e}"),
                })?;
                Ok(L1Value::Repr(seq))
            }
            Expr::Lit(_) => Err(L1Error::Unsupported {
                site: site.to_owned(),
                what: "bare-integer literals have no v0 value form (Q6)".to_owned(),
            }),

            Expr::Path(p) => {
                if p.0.len() == 1 {
                    let name = &p.0[0];
                    if let Some((_, v)) = scope.iter().rev().find(|(n, _)| n == name) {
                        return Ok(v.clone());
                    }
                    if let Some((d, i)) = self.env.ctor(name) {
                        if d.ctors[i].fields.is_empty() {
                            return Ok(L1Value::Data {
                                ty: d.name.clone(),
                                ctor: name.clone(),
                                fields: vec![],
                            });
                        }
                    }
                }
                Err(L1Error::Stuck {
                    site: site.to_owned(),
                    why: format!("unresolved name `{}`", p.0.join(".")),
                })
            }

            Expr::Let {
                name,
                ty,
                bound,
                body,
            } => {
                let bv = self.eval(fuel, depth, site, scope, bound)?;
                if let Some(g) = ty.as_ref().and_then(|t| t.guarantee) {
                    self.assert_guarantee(site, &bv, g)?;
                }
                scope.push((name.clone(), bv));
                let r = self.eval(fuel, depth, site, scope, body);
                scope.pop();
                r
            }

            Expr::If { cond, conseq, alt } => {
                let c = self.eval(fuel, depth, site, scope, cond)?;
                match c {
                    L1Value::Data { ref ctor, .. } if ctor == "True" => {
                        self.eval(fuel, depth, site, scope, conseq)
                    }
                    L1Value::Data { ref ctor, .. } if ctor == "False" => {
                        self.eval(fuel, depth, site, scope, alt)
                    }
                    other => Err(L1Error::Stuck {
                        site: site.to_owned(),
                        why: format!("if-condition evaluated to a non-Bool: {other:?}"),
                    }),
                }
            }

            Expr::Match { scrutinee, arms } => {
                self.eval_match(fuel, depth, site, scope, scrutinee, arms)
            }

            Expr::For {
                x,
                xs,
                acc,
                init,
                body,
            } => self.eval_for(fuel, depth, site, scope, x, xs, acc, init, body),

            Expr::Swap {
                value,
                target,
                policy,
            } => {
                let v = self.eval(fuel, depth, site, scope, value)?;
                let Some(src) = v.as_repr() else {
                    return Err(L1Error::Stuck {
                        site: site.to_owned(),
                        why: "swap source is not a representation value".to_owned(),
                    });
                };
                let repr = type_repr(site, target).map_err(|e| unsupported(site, &e))?;
                let out = self.swap.swap(src, &repr, &policy_name_ref(policy))?;
                let out = L1Value::Repr(out);
                if let Some(g) = target.guarantee {
                    self.assert_guarantee(site, &out, g)?;
                }
                Ok(out)
            }

            Expr::WithParadigm { .. } => Err(L1Error::Unsupported {
                site: site.to_owned(),
                what: "internal: a `with paradigm` block reached the evaluator — the ambient \
                       resolution pass strips it (RFC-0012 §4.4)"
                    .to_owned(),
            }),
            // A `wild` block (the audited FFI floor — M-661/M-721) executes by dispatching its
            // host-call form through the prim registry under the reserved `wild:` namespace
            // (RFC-0028 §4.3) — the capability handle. The default registry grants no `wild:` op, so
            // an ungranted host op is an explicit, never-silent refusal (G2). This mirrors the
            // elaborator's `wild → Op{wild:…}` lowering, so the L1 surface evaluator and the
            // L0/AOT paths agree on a `wild`-backed operation (the three-way differential).
            Expr::Wild(body) => self.eval_wild(fuel, depth, site, scope, body),
            Expr::Spore(_) => Err(L1Error::Unsupported {
                site: site.to_owned(),
                what: "`spore` is deferred to the reconstruction-manifest work (E2-5/M-260)"
                    .to_owned(),
            }),
            Expr::Lambda { .. } => Err(L1Error::Unsupported {
                site: site.to_owned(),
                what: "`lambda` (closures) is deferred to M-704 / RFC-0024 §5 — RFC-0037 D5 \
                       reserves the surface, it does not yet evaluate"
                    .to_owned(),
            }),

            // `colony { hypha e1, …, hypha eN }` (RFC-0008 §4.7; M-666). The trusted base evaluates
            // the **RT2 spawn-order sequentialization** — the reference semantics of any deterministic
            // concurrent program (RFC-0008 §4.2/RT2): each hypha body is evaluated in spawn order,
            // and the colony's value is the **last** hypha's (matching the type rule). This is the
            // honest reference run — *not* a real scheduler (the trusted base stays sequential; KC-3).
            // A real concurrent executor (`mycelium-mlir::runtime`, M-357) is a performance path
            // validated *against* this sequentialization (the RT2 differential), never a new meaning.
            // The parser guarantees ≥ 1 hypha; an empty list is still an explicit refusal here.
            Expr::Colony(hyphae) => {
                let Some((last, leading)) = hyphae.split_last() else {
                    return Err(L1Error::Unsupported {
                        site: site.to_owned(),
                        what: "internal: an empty `colony` reached the evaluator — the parser \
                               requires ≥ 1 hypha (RFC-0008 §4.7)"
                            .to_owned(),
                    });
                };
                // Evaluate each leading hypha for its (sequentialized) effect, in order; a refusal in
                // any one propagates (never silently dropped — RT4/I1). Their values are not the
                // colony's observable in this no-product v0 (only the last hypha's is).
                for h in leading {
                    self.eval(fuel, depth, site, scope, &h.body)?;
                }
                self.eval(fuel, depth, site, scope, &last.body)
            }

            Expr::Ascribe(inner, t) => {
                let v = self.eval(fuel, depth, site, scope, inner)?;
                if let Some(g) = t.guarantee {
                    self.assert_guarantee(site, &v, g)?;
                }
                Ok(v)
            }

            Expr::App { head, args } => self.eval_app(fuel, depth, site, scope, head, args),
        }
    }

    /// Bounded iteration (RFC-0007 §4.8): walk the linearly recursive spine head-to-tail,
    /// folding the accumulator through the body. The walk is **iterative** — a `for` over a long
    /// list costs fuel per element (each body evaluation is clocked) but never host stack, so it
    /// cannot trip the depth guard the way the equivalent hand-written recursion would.
    #[allow(clippy::too_many_arguments)] // the machine threads its budgets + the form's five parts
    fn eval_for(
        &self,
        fuel: &mut u64,
        depth: u32,
        site: &str,
        scope: &mut Vec<(String, L1Value)>,
        x: &str,
        xs: &Expr,
        acc: &str,
        init: &Expr,
        body: &Expr,
    ) -> Result<L1Value, L1Error> {
        let mut spine = self.eval(fuel, depth, site, scope, xs)?;
        let mut accv = self.eval(fuel, depth, site, scope, init)?;
        loop {
            let L1Value::Data { ty, ctor, fields } = spine else {
                return Err(L1Error::Stuck {
                    site: site.to_owned(),
                    why: "`for` spine is not a data value".to_owned(),
                });
            };
            if fields.is_empty() {
                return Ok(accv); // a nil — the spine ends, the fold is the accumulator
            }
            // A cons: exactly one spine field (type == ty) and one element field (checked).
            let Some(d) = self.env.types.get(&ty) else {
                return Err(L1Error::Stuck {
                    site: site.to_owned(),
                    why: format!("`for` over unregistered type `{ty}`"),
                });
            };
            let Some(ci) = d.ctors.iter().position(|c| c.name == ctor) else {
                return Err(L1Error::Stuck {
                    site: site.to_owned(),
                    why: format!("`for` met unknown constructor `{ctor}` of `{ty}`"),
                });
            };
            let mut elem = None;
            let mut rest = None;
            for (f, v) in d.ctors[ci].fields.iter().zip(fields) {
                if matches!(f, crate::checkty::Ty::Data(n, _) if *n == ty) {
                    rest = Some(v);
                } else {
                    elem = Some(v);
                }
            }
            let (Some(elem), Some(rest)) = (elem, rest) else {
                return Err(L1Error::Stuck {
                    site: site.to_owned(),
                    why: format!(
                        "`{ctor}` is not nil/cons-shaped — the checker should have refused"
                    ),
                });
            };
            // Each element's body evaluation is clocked like any other expression.
            *fuel = fuel.checked_sub(1).ok_or(L1Error::FuelExhausted)?;
            scope.push((x.to_owned(), elem));
            scope.push((acc.to_owned(), accv));
            let next = self.eval(fuel, depth, site, scope, body);
            scope.pop();
            scope.pop();
            accv = next?;
            spine = rest;
        }
    }

    /// The W7 flat-match machine (split out of [`Self::eval`] to keep the recursion frame small —
    /// the depth guard's budget is host stack, so frame size is part of the contract).
    fn eval_match(
        &self,
        fuel: &mut u64,
        depth: u32,
        site: &str,
        scope: &mut Vec<(String, L1Value)>,
        scrutinee: &Expr,
        arms: &[crate::ast::Arm],
    ) -> Result<L1Value, L1Error> {
        let sv = self.eval(fuel, depth, site, scope, scrutinee)?;
        // The checker has verified exhaustiveness, redundancy, types, and arity (W7), so the first
        // arm whose (possibly nested) pattern matches fires. The trailing `Stuck` is unreachable for
        // checked programs but kept as the honest never-silent fallback (G2).
        for arm in arms {
            let mut binds: Vec<(String, L1Value)> = Vec::new();
            if self.try_match(site, &arm.pattern, &sv, &mut binds)? {
                let mark = scope.len();
                scope.extend(binds);
                let r = self.eval(fuel, depth, site, scope, &arm.body);
                scope.truncate(mark);
                return r;
            }
        }
        Err(L1Error::Stuck {
            site: site.to_owned(),
            why: "no arm matched the scrutinee (W7 — the checker requires coverage)".to_owned(),
        })
    }

    /// Try to match `val` against `pat`, accumulating the pattern's binders into `binds`
    /// (left-to-right, recursively for nested patterns). Returns whether it matched; on a partial
    /// nested failure the caller discards `binds`, so no rollback is needed. The
    /// constructor/literal/binder resolution mirrors the typechecker's `check_pattern` exactly, so a
    /// checked program never gets stuck (RFC-0007 §4.7).
    fn try_match(
        &self,
        site: &str,
        pat: &Pattern,
        val: &L1Value,
        binds: &mut Vec<(String, L1Value)>,
    ) -> Result<bool, L1Error> {
        match pat {
            Pattern::Wildcard => Ok(true),
            // A bare name is a nullary-constructor alternative iff it names one of the value's data
            // type's constructors; otherwise it binds the whole value.
            Pattern::Ident(n) => match val {
                L1Value::Data { ty, ctor, .. }
                    if self
                        .env
                        .types
                        .get(ty)
                        .is_some_and(|d| d.ctors.iter().any(|c| c.name == *n)) =>
                {
                    Ok(ctor == n)
                }
                _ => {
                    binds.push((n.clone(), val.clone()));
                    Ok(true)
                }
            },
            Pattern::Ctor(n, subs) => match val {
                L1Value::Data { ctor, fields, .. } => {
                    if ctor != n {
                        return Ok(false);
                    }
                    for (sub, fv) in subs.iter().zip(fields) {
                        if !self.try_match(site, sub, fv, binds)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                L1Value::Repr(_) => Ok(false),
            },
            Pattern::Lit(lit) => match val {
                L1Value::Repr(v) => {
                    let lv = crate::elab::lit_value(site, lit).map_err(|e| L1Error::Stuck {
                        site: site.to_owned(),
                        why: format!("malformed literal pattern: {e}"),
                    })?;
                    Ok(lv.repr() == v.repr() && lv.payload() == v.payload())
                }
                L1Value::Data { .. } => Ok(false),
            },
        }
    }

    /// Execute a `wild { name(args…) }` block — the audited FFI floor (M-661/M-721; RFC-0028 §4.3).
    /// The host operation is resolved through the prim registry under its reserved `wild:<name>` key
    /// (the capability handle, §4.3); the registry registers no `wild:` op by default, so an
    /// ungranted host op is an explicit [`KernelError::UnknownPrim`] (never silent — G2). The body
    /// is the trusted/opaque escape (M-661): only its *shape* is interpreted (a host-call form
    /// `name(args…)` or a bare `name`); any other shape is an explicit refusal. Mirrors the
    /// elaborator's `wild → Op{wild:…}` lowering so this surface path and the L0/AOT paths agree
    /// (the three-way differential).
    fn eval_wild(
        &self,
        fuel: &mut u64,
        depth: u32,
        site: &str,
        scope: &mut Vec<(String, L1Value)>,
        body: &Expr,
    ) -> Result<L1Value, L1Error> {
        let (name, args): (&str, &[Expr]) =
            match body {
                Expr::App { head, args } => match head.as_ref() {
                    Expr::Path(p) if p.0.len() == 1 => (p.0[0].as_str(), args.as_slice()),
                    _ => return Err(L1Error::Unsupported {
                        site: site.to_owned(),
                        what: "a v0 `wild` block body must be a host-call form `name(args…)` with \
                               a single, undotted host-operation name (RFC-0028 §4.2)"
                            .to_owned(),
                    }),
                },
                Expr::Path(p) if p.0.len() == 1 => (p.0[0].as_str(), &[]),
                _ => return Err(L1Error::Unsupported {
                    site: site.to_owned(),
                    what:
                        "a v0 `wild` block body must be a host-call form `name(args…)` or a bare \
                           `name` (RFC-0028 §4.2)"
                            .to_owned(),
                }),
            };
        let key = format!("wild:{name}");
        // CBV: the host-call arguments evaluate left-to-right before dispatch.
        let mut argv = Vec::with_capacity(args.len());
        for a in args {
            argv.push(self.eval(fuel, depth, site, scope, a)?);
        }
        let vals: Vec<&Value> = argv
            .iter()
            .map(|v| {
                v.as_repr().ok_or_else(|| L1Error::Stuck {
                    site: site.to_owned(),
                    why: format!("`wild` host op `{name}` applied to a data value (RFC-0028 §4.4)"),
                })
            })
            .collect::<Result<_, _>>()?;
        let f = self
            .prims
            .get(&key)
            .ok_or_else(|| L1Error::Kernel(KernelError::UnknownPrim(key.clone())))?;
        Ok(L1Value::Repr(f(&key, &vals)?))
    }

    /// First-order application: user functions, saturated constructors (W6), and prims — split
    /// out of [`Self::eval`] for the same frame-size reason as [`Self::eval_match`].
    fn eval_app(
        &self,
        fuel: &mut u64,
        depth: u32,
        site: &str,
        scope: &mut Vec<(String, L1Value)>,
        head: &Expr,
        args: &[Expr],
    ) -> Result<L1Value, L1Error> {
        let Expr::Path(p) = head else {
            return Err(L1Error::Stuck {
                site: site.to_owned(),
                why: "v0 application head must be a name (first-order)".to_owned(),
            });
        };
        if p.0.len() != 1 {
            return Err(L1Error::Stuck {
                site: site.to_owned(),
                why: format!("dotted call `{}`", p.0.join(".")),
            });
        }
        let name = &p.0[0];
        // CBV: arguments evaluate left-to-right before any application.
        let mut argv = Vec::with_capacity(args.len());
        for a in args {
            argv.push(self.eval(fuel, depth, site, scope, a)?);
        }
        if self.env.fns.contains_key(name) {
            return self.invoke(fuel, depth, name, argv);
        }
        if let Some((d, i)) = self.env.ctor(name) {
            if d.ctors[i].fields.len() != argv.len() {
                return Err(L1Error::Stuck {
                    site: site.to_owned(),
                    why: format!("unsaturated constructor `{name}` (W6)"),
                });
            }
            return Ok(L1Value::Data {
                ty: d.name.clone(),
                ctor: name.clone(),
                fields: argv,
            });
        }
        if let Some(kernel) = prim_kernel_name(name) {
            let vals: Vec<&Value> = argv
                .iter()
                .map(|v| {
                    v.as_repr().ok_or_else(|| L1Error::Stuck {
                        site: site.to_owned(),
                        why: format!("prim `{name}` applied to a data value"),
                    })
                })
                .collect::<Result<_, _>>()?;
            let f = self
                .prims
                .get(kernel)
                .ok_or_else(|| L1Error::Kernel(KernelError::UnknownPrim(kernel.to_owned())))?;
            return Ok(L1Value::Repr(f(kernel, &vals)?));
        }
        Err(L1Error::Stuck {
            site: site.to_owned(),
            why: format!("unknown function/constructor/prim `{name}`"),
        })
    }

    /// The stage-0 dynamic guarantee check (RFC-0007 §4.3): the value's actual tag must be **at
    /// least as strong** as the asserted index — an annotation may only weaken (VR-5). The check
    /// never modifies the value: a passing assertion leaves the (possibly stronger) tag in place,
    /// and a failing one is an explicit error, never a downgrade-and-continue.
    pub(crate) fn assert_guarantee(
        &self,
        site: &str,
        v: &L1Value,
        asserted: Strength,
    ) -> Result<(), L1Error> {
        match v {
            L1Value::Repr(value) => {
                let actual = value.meta().guarantee();
                if actual.rank() > strength_of(asserted).rank() {
                    return Err(L1Error::GuaranteeTooWeak {
                        site: site.to_owned(),
                        asserted,
                        actual,
                    });
                }
                Ok(())
            }
            L1Value::Data { .. } => Err(L1Error::Unsupported {
                site: site.to_owned(),
                what: "a guarantee index on a data-typed value has no Meta to check in v0"
                    .to_owned(),
            }),
        }
    }
}

/// Forward a bridge refusal (shared with elaboration) as an explicit evaluator refusal.
fn unsupported(site: &str, e: &ElabError) -> L1Error {
    L1Error::Unsupported {
        site: site.to_owned(),
        what: e.to_string(),
    }
}
