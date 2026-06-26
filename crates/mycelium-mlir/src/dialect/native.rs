//! The **real** ternary-dialect lowering (M-601; M-725; RFC-0004 §2; RFC-0029 §7; ADR-009/ADR-019).
//!
//! Feature-gated (`mlir-dialect`, OFF by default). For the **bit/trit element-wise straight-line
//! fragment plus the balanced-ternary additive carry chain** (`trit.add`/`trit.sub`, M-725) this
//! emits a genuine MLIR module in the `func` + `arith` + `cf` dialects and drives it through the
//! verified libMLIR pipeline
//!
//! ```text
//! mlir-opt-<v> --convert-cf-to-llvm --convert-func-to-llvm --convert-arith-to-llvm
//!   --reconcile-unrealized-casts
//!   | mlir-translate-<v> --mlir-to-llvmir
//! ```
//!
//! to **real LLVM IR**, then `clang` → native executable → run → read-back. It is a fourth,
//! genuinely MLIR-compiled execution path (not the textual [`super::emit`] skeleton, not the
//! [`crate::llvm`] direct-LLVM emitter, not the [`crate::aot`] env-machine).
//!
//! **The fragment, and the moving honest boundary (RFC-0004 §2; M-725; VR-5/G2).** RFC-0004 §2
//! sequences the AOT path as "`ternary` first … lowering progressively to `linalg`/`vector`/`arith`".
//! The element-wise bit/trit ops (`core.id`, `bit.not/and/or/xor`, `trit.neg`) are the sub-fragment
//! the **standard** `arith` dialect carries faithfully — one `arith` op per element, every op
//! dumpable. **M-725 widens this one increment beyond element-wise**: the balanced-ternary *additive*
//! carry chain `trit.add`/`trit.sub` now lowers through the real dialect path as a fixed-width
//! ripple-carry over `arith` ops (`arith.addi`/`arith.remsi`/`arith.divsi`/`arith.subi`,
//! digit-for-digit the same `s + 4 → srem/sdiv 3 − 1` step the direct-LLVM path uses), with the
//! out-of-range result reported through the **shared** [`crate::llvm::OVERFLOW_SENTINEL`] read-back
//! (a `cf.cond_br` on the runtime overflow flag) — never a silent wrap. The new honest boundary is
//! `trit.mul` (the shifted-accumulate / 2m-trit-buffer fragment) and everything richer (the
//! `Construct`/`Match` data fragment, closures, recursion, `Swap`, Dense/VSA): each is an
//! **explicit, never-silent** [`DialectError::Unsupported`] routing the program to the direct-LLVM
//! backend ([`crate::llvm`]) or the interpreter — which already cover the full v0 calculus. The
//! carry *step* mirrors `mycelium_core::ternary::add_with_carry` digit-for-digit (one source of
//! truth for the carry semantics, re-emitted in `arith`, never a *divergent* second algorithm — DRY);
//! we still ship **no** divergent codegen for `trit.mul`/closures here just to widen further. The
//! honest boundary is an explicit refusal, not silent or fragile output.
//!
//! **Read-back protocol — shared with [`crate::llvm`] (single contract).** The emitted `@main`
//! `putchar`s each result element's ASCII char (`'0'`/`'1'` for bits; `'-'`/`'0'`/`'+'` for trits)
//! then a newline, and the result is decoded by the **same** [`crate::llvm::decode_result`] the
//! direct-LLVM path uses. On an additive overflow (`trit.add`/`trit.sub` leaving the `m`-trit range)
//! the artifact prints the **shared** [`crate::llvm::OVERFLOW_SENTINEL`] line instead, decoded to an
//! explicit [`DialectError::Overflow`] — byte-for-byte the same sentinel and meaning as the
//! direct-LLVM path's [`crate::llvm::AotError::Overflow`]. So the MLIR-dialect output and the
//! direct-LLVM output are read back identically — the three-way differential (M-602/M-725) compares
//! like with like, on the result *and* the overflow refusal.
//!
//! **Toolchain probing (skip-gracefully).** `mlir-opt`/`mlir-translate`/`clang` are probed at
//! runtime; absence is a graceful [`DialectError::ToolchainMissing`] (the caller skips, never
//! fails) — mirroring the `llc`/`clang` `ToolchainMissing` idiom in [`crate::llvm`]. So even
//! `cargo test --features mlir-dialect` is green on a box without libMLIR (ADR-019).
//!
//! **Guarantee tag:** `Empirical` — a real compiled artifact, correctness evidenced by the M-602
//! three-way differential over the corpus; never `Proven` without a checked equivalence proof
//! (VR-5).

use std::fmt;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use mycelium_core::lower::{self, Anf, Atom, Rhs};
use mycelium_core::{Node, Payload, Repr, Trit, Value};

use crate::llvm::{decode_result, LaneKind, OVERFLOW_SENTINEL};

/// An explicit failure of the real MLIR-dialect path. Every unsupported construct, missing tool, or
/// subprocess failure is one of these — the path is **never silent** (G2). Mirrors the contract of
/// [`crate::llvm::AotError`], specialized to the MLIR pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialectError {
    /// A node/prim/repr outside the fragment the standard `arith`/`func`/`cf` dialects lower here
    /// (the bit/trit element-wise ops **plus** the balanced-ternary additive carry chain
    /// `trit.add`/`trit.sub`; M-725). The message names what was refused and where it should run
    /// instead (the direct-LLVM backend [`crate::llvm`] or the interpreter) — an `EXPLAIN`-able
    /// routing, never a silent drop (G2/VR-5).
    Unsupported(String),
    /// A balanced-ternary additive result left the fixed `m`-trit range — the MLIR-compiled artifact
    /// computed the overflow at runtime and signalled it through the shared read-back protocol (the
    /// [`crate::llvm::OVERFLOW_SENTINEL`] line). Surfaced as an explicit error mirroring
    /// [`crate::llvm::AotError::Overflow`] and the interpreter's `EvalError::Overflow` — never a
    /// silent wrap (SC-3/G2). So the three-way differential stays honest on overflow too (M-725).
    Overflow(String),
    /// An operand atom with no prior binding (an ill-formed lowering — should not occur for a
    /// well-formed ANF program; surfaced explicitly rather than panicking).
    FreeVariable(String),
    /// The MLIR toolchain (`mlir-opt-<v>` / `mlir-translate-<v>`) or `clang` is not installed —
    /// callers should **skip**, not fail (the house "skip gracefully when a tool is absent" idiom;
    /// ADR-019). Carries the missing tool name.
    ToolchainMissing(String),
    /// A pipeline stage (`mlir-opt`, `mlir-translate`, or `clang`) ran but returned a non-zero
    /// status. Carries the stage name + captured stderr (no opaque failure).
    Compile(String),
    /// The compiled artifact failed to run or produced unreadable output.
    Run(String),
    /// The native stdout did not parse back into the expected payload shape.
    Parse(String),
    /// Reconstructing the result [`Value`] failed its well-formedness check.
    Wf(String),
}

impl fmt::Display for DialectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DialectError::Unsupported(m) => {
                write!(f, "unsupported for the MLIR-dialect fragment: {m}")
            }
            DialectError::Overflow(m) => write!(f, "balanced-ternary overflow: {m}"),
            DialectError::FreeVariable(v) => write!(f, "free variable in lowered IR: {v}"),
            DialectError::ToolchainMissing(t) => write!(f, "MLIR toolchain missing: {t}"),
            DialectError::Compile(e) => write!(f, "MLIR pipeline compile failed: {e}"),
            DialectError::Run(e) => write!(f, "MLIR artifact run failed: {e}"),
            DialectError::Parse(e) => write!(f, "MLIR artifact output parse failed: {e}"),
            DialectError::Wf(e) => write!(f, "result not well-formed: {e}"),
        }
    }
}

impl std::error::Error for DialectError {}

/// The representation kind of a lowered result lane — the **public** shape descriptor for the
/// MLIR-dialect path (`Binary{w}` or `Ternary{m}`). Mirrors the internal `crate::llvm::LaneKind`
/// (which is `pub(crate)`); kept distinct so the public API does not leak a crate-private type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultKind {
    /// `Binary{w}` — elements in `{0, 1}`, printed `'0'`/`'1'`.
    Binary,
    /// `Ternary{m}` — balanced-ternary elements in `{-1, 0, 1}`, printed `'-'`/`'0'`/`'+'`.
    Ternary,
}

impl ResultKind {
    fn from_lane(k: LaneKind) -> Self {
        match k {
            LaneKind::Binary => ResultKind::Binary,
            LaneKind::Ternary => ResultKind::Ternary,
        }
    }
    fn to_lane(self) -> LaneKind {
        match self {
            ResultKind::Binary => LaneKind::Binary,
            ResultKind::Ternary => LaneKind::Ternary,
        }
    }
}

/// The resolved MLIR toolchain: the `mlir-opt`/`mlir-translate` binary names (version-matched to the
/// installed LLVM major) plus `clang`. Produced by [`resolve_tools`]; inspectable (no hidden tool
/// choice — the resolved binaries are queryable for `EXPLAIN`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MlirTools {
    /// The `mlir-opt` binary name, e.g. `mlir-opt-18`.
    pub mlir_opt: String,
    /// The `mlir-translate` binary name, e.g. `mlir-translate-18`.
    pub mlir_translate: String,
    /// The `clang` binary name (`clang-<v>` if present, else `clang`).
    pub clang: String,
    /// The detected LLVM major version the tools are matched to.
    pub llvm_major: u32,
}

impl MlirTools {
    /// Whether the MLIR toolchain resolves in this environment (a convenience over
    /// [`resolve_tools`] for tests/harnesses that want to assert non-vacuous coverage). `true` iff
    /// [`resolve_tools`] succeeds.
    #[must_use]
    pub fn is_available() -> bool {
        resolve_tools().is_ok()
    }
}

/// Parse the LLVM major version from a `--version` banner — either an `… LLVM version NN.…` line
/// (`llc`, `mlir-opt`, `mlir-translate`) **or** a `clang version NN.…` line (`clang`, which does not
/// print "LLVM version"). Returns `None` when no recognized banner is present.
fn parse_llvm_major(s: &str) -> Option<u32> {
    for line in s.lines() {
        for marker in ["LLVM version", "clang version"] {
            if let Some(idx) = line.find(marker) {
                let rest = &line[idx + marker.len()..];
                let tok: String = rest
                    .trim()
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                if let Ok(major) = tok.parse::<u32>() {
                    return Some(major);
                }
            }
        }
    }
    None
}

/// Detect the installed LLVM major version from `llc --version`, falling back to `clang --version`.
/// Returns `None` when neither tool is present or the version line cannot be parsed — the caller
/// turns that into a graceful skip.
fn detect_llvm_major() -> Option<u32> {
    for tool in ["llc", "clang"] {
        if let Ok(out) = Command::new(tool).arg("--version").output() {
            if let Some(major) = parse_llvm_major(&String::from_utf8_lossy(&out.stdout)) {
                return Some(major);
            }
        }
    }
    None
}

/// Probe whether a binary exists + responds to `--version`.
fn tool_present(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// The LLVM major a tool reports via `--version`, or `None` if it's absent/unparsable. Used to
/// confirm an *unversioned* fallback binary actually matches the detected LLVM major before it is
/// accepted — never a silent mismatched substitution (G2).
fn tool_major(name: &str) -> Option<u32> {
    let out = Command::new(name).arg("--version").output().ok()?;
    parse_llvm_major(&String::from_utf8_lossy(&out.stdout))
}

/// Resolve the MLIR toolchain, version-matched to the installed LLVM major.
///
/// Tries the versioned binaries first (`mlir-opt-<major>`, `mlir-translate-<major>` — how the distro
/// packages them; ADR-019), then the unversioned fallbacks (`mlir-opt`, `mlir-translate`). Returns
/// [`DialectError::ToolchainMissing`] (a *skip*, not a failure) when the LLVM major cannot be
/// detected or a required tool is absent. **Never** silently substitutes a mismatched-version tool
/// (G2; no silent toolchain bump — CLAUDE.md).
pub fn resolve_tools() -> Result<MlirTools, DialectError> {
    let major = detect_llvm_major().ok_or_else(|| {
        DialectError::ToolchainMissing("llc/clang (LLVM version undetectable)".to_owned())
    })?;

    let opt_versioned = format!("mlir-opt-{major}");
    let tr_versioned = format!("mlir-translate-{major}");
    // Versioned binary first; otherwise an unversioned fallback ONLY when its own `--version`
    // reports the same major (never silently substitute a mismatched toolchain — G2).
    let mlir_opt = if tool_present(&opt_versioned) {
        opt_versioned
    } else if tool_major("mlir-opt") == Some(major) {
        "mlir-opt".to_owned()
    } else {
        return Err(DialectError::ToolchainMissing(format!(
            "mlir-opt-{major} (unversioned `mlir-opt` absent or a different LLVM major — never \
             silently substituted, G2) — run scripts/setup-mlir.sh"
        )));
    };
    let mlir_translate = if tool_present(&tr_versioned) {
        tr_versioned
    } else if tool_major("mlir-translate") == Some(major) {
        "mlir-translate".to_owned()
    } else {
        return Err(DialectError::ToolchainMissing(format!(
            "mlir-translate-{major} (unversioned `mlir-translate` absent or a different LLVM major \
             — never silently substituted, G2) — run scripts/setup-mlir.sh"
        )));
    };
    let clang_versioned = format!("clang-{major}");
    let clang = if tool_present(&clang_versioned) {
        clang_versioned
    } else if tool_major("clang") == Some(major) {
        "clang".to_owned()
    } else {
        return Err(DialectError::ToolchainMissing(format!(
            "clang-{major} (unversioned `clang` absent or a different LLVM major — never silently \
             substituted, G2)"
        )));
    };

    Ok(MlirTools {
        mlir_opt,
        mlir_translate,
        clang,
        llvm_major: major,
    })
}

// ─── SSA naming for the emitted MLIR ──────────────────────────────────────────────────────────

/// A monotone counter minting fresh MLIR SSA names (`%v0`, `%v1`, …). MLIR SSA values are textual
/// `%name`s exactly like LLVM, so the emitter mirrors [`crate::llvm`]'s `Ssa` shape.
#[derive(Default)]
struct Ssa(usize);
impl Ssa {
    fn fresh(&mut self) -> String {
        let n = self.0;
        self.0 += 1;
        format!("%v{n}")
    }
}

/// A computed value lane in the MLIR body: its representation kind and one `i32`-typed SSA operand
/// (or `i32` literal `arith.constant`) per element. The element model is **identical** to
/// [`crate::llvm`]'s `Lane` (Binary in `{0,1}`, Ternary in `{-1,0,1}`), so the read-back is shared.
#[derive(Debug, Clone)]
struct Lane {
    kind: LaneKind,
    /// SSA names of `i32` values, one per element.
    vals: Vec<String>,
}

/// Emit an `arith.constant` for one `i32` element value, returning its SSA name.
fn emit_const_i32(v: i32, ssa: &mut Ssa, body: &mut String) -> String {
    let r = ssa.fresh();
    let _ = writeln!(body, "    {r} = arith.constant {v} : i32");
    r
}

/// Materialize a constant `Value`'s elements as `arith.constant` SSA values (the entry point for a
/// `Rhs::Const`). Refuses Dense/VSA reprs explicitly (they are not in the element-wise fragment).
fn const_lane(v: &Value, ssa: &mut Ssa, body: &mut String) -> Result<Lane, DialectError> {
    match (v.repr(), v.payload()) {
        (Repr::Binary { .. }, Payload::Bits(b)) => {
            let vals = b
                .iter()
                .map(|&x| emit_const_i32(i32::from(x), ssa, body))
                .collect();
            Ok(Lane {
                kind: LaneKind::Binary,
                vals,
            })
        }
        (Repr::Ternary { .. }, Payload::Trits(t)) => {
            let vals = t
                .iter()
                .map(|&x| {
                    let e = match x {
                        Trit::Neg => -1,
                        Trit::Zero => 0,
                        Trit::Pos => 1,
                    };
                    emit_const_i32(e, ssa, body)
                })
                .collect();
            Ok(Lane {
                kind: LaneKind::Ternary,
                vals,
            })
        }
        (repr, _) => Err(DialectError::Unsupported(format!(
            "repr {repr:?} is not in the element-wise dialect fragment (Dense/VSA stay on the \
             interpreter / direct-LLVM path)"
        ))),
    }
}

/// Require a lane to be of the expected kind, else an explicit refusal (a `bit.*` op on a ternary
/// lane, or `trit.*` on a binary one, is a type error — never silently mis-lowered; G2).
fn require_kind(prim: &str, got: LaneKind, want: LaneKind) -> Result<(), DialectError> {
    if got == want {
        Ok(())
    } else {
        Err(DialectError::Unsupported(format!(
            "{prim} expects a {want:?} operand, got {got:?}"
        )))
    }
}

/// Map a unary `arith` op over a lane's elements (one op per element — dumpable, no opaque pass),
/// `mk` rendering the op line for element SSA `x` into result SSA `r`.
fn map1(a: &Lane, ssa: &mut Ssa, body: &mut String, mk: impl Fn(&str, &str) -> String) -> Lane {
    let vals = a
        .vals
        .iter()
        .map(|x| {
            let r = ssa.fresh();
            let _ = writeln!(body, "{}", mk(x, &r));
            r
        })
        .collect();
    Lane { kind: a.kind, vals }
}

/// Map a binary `arith` op over two equal-width lanes' elements, `mk` rendering the op line for
/// element SSAs `x`,`y` into result SSA `r`. Width mismatch is an explicit refusal (G2).
fn map2(
    prim: &str,
    a: &Lane,
    b: &Lane,
    ssa: &mut Ssa,
    body: &mut String,
    mk: impl Fn(&str, &str, &str) -> String,
) -> Result<Lane, DialectError> {
    if a.vals.len() != b.vals.len() {
        return Err(DialectError::Unsupported(format!(
            "{prim}: width mismatch {} vs {}",
            a.vals.len(),
            b.vals.len()
        )));
    }
    let vals = a
        .vals
        .iter()
        .zip(&b.vals)
        .map(|(x, y)| {
            let r = ssa.fresh();
            let _ = writeln!(body, "{}", mk(x, y, &r));
            r
        })
        .collect();
    Ok(Lane { kind: a.kind, vals })
}

/// Lower one primitive over its operand lanes to `arith` ops, returning the result lane. Covers the
/// element-wise fragment **and** the balanced-ternary additive carry chain `trit.add`/`trit.sub`
/// (M-725) — the latter push their runtime overflow `i1` SSA name onto `flags` (the caller folds
/// them into the program-level overflow flag that drives the read-back). The new honest boundary is
/// `trit.mul` (and everything richer): an explicit [`DialectError::Unsupported`] routing to
/// [`crate::llvm`] / the interpreter (G2). The carry *step* re-emits
/// `mycelium_core::ternary::add_with_carry` digit-for-digit (one source of truth, never a divergent
/// second algorithm — DRY).
fn emit_op(
    prim: &str,
    operands: &[&Lane],
    ssa: &mut Ssa,
    body: &mut String,
    flags: &mut Vec<String>,
) -> Result<Lane, DialectError> {
    let arity1 = |p: &str| -> Result<&Lane, DialectError> {
        match operands {
            [a] => Ok(*a),
            _ => Err(DialectError::Unsupported(format!(
                "{p} expects 1 operand, got {}",
                operands.len()
            ))),
        }
    };
    let arity2 = |p: &str| -> Result<(&Lane, &Lane), DialectError> {
        match operands {
            [a, b] => Ok((*a, *b)),
            _ => Err(DialectError::Unsupported(format!(
                "{p} expects 2 operands, got {}",
                operands.len()
            ))),
        }
    };
    match prim {
        // Identity passes the lane through unchanged, any kind.
        "core.id" => Ok(arity1(prim)?.clone()),
        // bit.not x = xor(x, 1) per bit.
        "bit.not" => {
            let a = arity1(prim)?;
            require_kind(prim, a.kind, LaneKind::Binary)?;
            let one = emit_const_i32(1, ssa, body);
            Ok(map1(a, ssa, body, |x, r| {
                format!("    {r} = arith.xori {x}, {one} : i32")
            }))
        }
        "bit.and" | "bit.or" | "bit.xor" => {
            let (a, b) = arity2(prim)?;
            require_kind(prim, a.kind, LaneKind::Binary)?;
            require_kind(prim, b.kind, LaneKind::Binary)?;
            let op = match prim {
                "bit.and" => "arith.andi",
                "bit.or" => "arith.ori",
                _ => "arith.xori",
            };
            map2(prim, a, b, ssa, body, |x, y, r| {
                format!("    {r} = {op} {x}, {y} : i32")
            })
        }
        // Balanced-ternary negation is digit-wise (`-t`), exact, no carry — `0 - x` per trit.
        "trit.neg" => {
            let a = arity1(prim)?;
            require_kind(prim, a.kind, LaneKind::Ternary)?;
            let zero = emit_const_i32(0, ssa, body);
            Ok(map1(a, ssa, body, |x, r| {
                format!("    {r} = arith.subi {zero}, {x} : i32")
            }))
        }
        // Balanced-ternary addition (M-725): a fixed-width ripple-carry over the trits (LSB→MSB),
        // with a runtime overflow `i1` (non-zero final carry ⇒ out of m-trit range). Mirrors
        // `mycelium_core::ternary::add` digit-for-digit — the same `s + 4 → srem/sdiv 3 − 1` step
        // the direct-LLVM path (`crate::llvm::emit_trit_add`) emits, re-expressed in `arith` ops.
        "trit.add" => {
            let (a, b) = arity2(prim)?;
            require_kind(prim, a.kind, LaneKind::Ternary)?;
            require_kind(prim, b.kind, LaneKind::Ternary)?;
            require_width(prim, a, b)?;
            let (lane, ovf) = emit_trit_add(&a.vals, &b.vals, ssa, body);
            flags.push(ovf);
            Ok(lane)
        }
        // Subtraction `a − b` = `add(a, neg(b))`: negate `b`'s trits, then the same ripple adder
        // (exactly `crate::llvm`'s `trit.sub`; DRY).
        "trit.sub" => {
            let (a, b) = arity2(prim)?;
            require_kind(prim, a.kind, LaneKind::Ternary)?;
            require_kind(prim, b.kind, LaneKind::Ternary)?;
            require_width(prim, a, b)?;
            let zero = emit_const_i32(0, ssa, body);
            let neg_b = map1(b, ssa, body, |x, r| {
                format!("    {r} = arith.subi {zero}, {x} : i32")
            });
            let (lane, ovf) = emit_trit_add(&a.vals, &neg_b.vals, ssa, body);
            flags.push(ovf);
            Ok(lane)
        }
        // The NEW honest boundary (M-725): `trit.mul` is the shifted-accumulate / 2m-trit-buffer
        // fragment — materially richer than the additive ripple, and already implemented and
        // differential-proven on the direct-LLVM path (`crate::llvm::emit_trit_mul`). We do NOT ship
        // a second, divergent multiply codegen here just to widen further: that would be two sources
        // of truth for the same semantics (DRY) and a fragility risk (G2). Explicit refusal.
        "trit.mul" => Err(DialectError::Unsupported(format!(
            "{prim}: balanced-ternary multiply (shifted-accumulate) is the MLIR-dialect fragment's \
             new boundary — it runs on the direct-LLVM backend (crate::llvm::emit_trit_mul), which \
             carries the 2m-trit accumulation + overflow read-back"
        ))),
        other => Err(DialectError::Unsupported(format!(
            "primitive {other:?} is not in the MLIR-dialect fragment (bit.not/and/or/xor, trit.neg, \
             trit.add, trit.sub, core.id) — it runs on the direct-LLVM backend / interpreter"
        ))),
    }
}

/// Require two lanes to have equal element count, else an explicit width-mismatch refusal (G2).
/// Mirrors [`crate::llvm`]'s `require_width`.
fn require_width(prim: &str, a: &Lane, b: &Lane) -> Result<(), DialectError> {
    if a.vals.len() == b.vals.len() {
        Ok(())
    } else {
        Err(DialectError::Unsupported(format!(
            "{prim}: width mismatch {} vs {}",
            a.vals.len(),
            b.vals.len()
        )))
    }
}

/// Emit a fixed-width balanced-ternary ripple-carry add over MSB-first trit operands `a`/`b` (equal
/// length, caller-checked) in `arith` ops. Returns the sum lane (MSB-first) and the SSA name of an
/// `i1` register set iff the final carry is non-zero (overflow). Digit-for-digit identical to
/// [`crate::llvm`]'s `emit_trit_add` (and thus to `mycelium_core::ternary::add`): with
/// `x = aᵢ + bᵢ + carry + 4` (always ≥ 1 so `arith.remsi`/`arith.divsi` are euclidean), the balanced
/// digit is `x remsi 3 − 1` and the next carry is `x divsi 3 − 1`.
fn emit_trit_add(a: &[String], b: &[String], ssa: &mut Ssa, body: &mut String) -> (Lane, String) {
    let m = a.len();
    // The incoming carry of the LSB step is the constant 0 trit.
    let mut carry = emit_const_i32(0, ssa, body);
    let mut sum_lsb: Vec<String> = Vec::with_capacity(m);
    // Process least-significant first (the tail of the MSB-first strings).
    for i in (0..m).rev() {
        let (digit, next_carry) = emit_trit_add_step(&a[i], &b[i], &carry, ssa, body);
        sum_lsb.push(digit);
        carry = next_carry;
    }
    // Overflow iff the final carry out of the most-significant trit is non-zero (an `i1`).
    let zero = emit_const_i32(0, ssa, body);
    let ovf = ssa.fresh();
    let _ = writeln!(body, "    {ovf} = arith.cmpi ne, {carry}, {zero} : i32");
    let vals: Vec<String> = sum_lsb.into_iter().rev().collect(); // back to MSB-first
    (
        Lane {
            kind: LaneKind::Ternary,
            vals,
        },
        ovf,
    )
}

/// One balanced-ternary add step in `arith`: given operand trits `a`/`b` and the incoming `carry`
/// (all `i32` SSA in `{−1,0,1}`), emit the balanced digit + outgoing carry. Returns
/// `(digit_reg, carry_reg)`. Byte-for-byte the `arith` analogue of [`crate::llvm`]'s
/// `emit_trit_add_step` (the single shared carry primitive — DRY).
fn emit_trit_add_step(
    a: &str,
    b: &str,
    carry: &str,
    ssa: &mut Ssa,
    body: &mut String,
) -> (String, String) {
    let four = emit_const_i32(4, ssa, body);
    let three = emit_const_i32(3, ssa, body);
    let one = emit_const_i32(1, ssa, body);
    let s1 = ssa.fresh();
    let _ = writeln!(body, "    {s1} = arith.addi {a}, {b} : i32");
    let s2 = ssa.fresh();
    let _ = writeln!(body, "    {s2} = arith.addi {s1}, {carry} : i32");
    // x = s + 4 ∈ [1,7], strictly positive ⇒ remsi/divsi coincide with euclidean rem/div by 3.
    let x = ssa.fresh();
    let _ = writeln!(body, "    {x} = arith.addi {s2}, {four} : i32");
    let rem = ssa.fresh();
    let _ = writeln!(body, "    {rem} = arith.remsi {x}, {three} : i32");
    let digit = ssa.fresh();
    let _ = writeln!(body, "    {digit} = arith.subi {rem}, {one} : i32");
    let q = ssa.fresh();
    let _ = writeln!(body, "    {q} = arith.divsi {x}, {three} : i32");
    let next_carry = ssa.fresh();
    let _ = writeln!(body, "    {next_carry} = arith.subi {q}, {one} : i32");
    (digit, next_carry)
}

/// Fold a list of `i1` overflow flags into one (`arith.ori` chain), or `None` if empty.
/// Deterministic. Mirrors [`crate::llvm`]'s `fold_or`.
fn fold_or(flags: &[String], ssa: &mut Ssa, body: &mut String) -> Option<String> {
    let mut it = flags.iter();
    let mut acc = it.next()?.clone();
    for f in it {
        let r = ssa.fresh();
        let _ = writeln!(body, "    {r} = arith.ori {acc}, {f} : i1");
        acc = r;
    }
    Some(acc)
}

/// Walk the lowered ANF, emitting one `arith` op per binding into `@main`'s body, and return the
/// result lane **plus** the program-level overflow flag (`Some(i1)` iff any `trit.add`/`trit.sub`
/// binding can overflow at runtime, else `None`). Returns an explicit [`DialectError::Unsupported`]
/// for any node outside the fragment — routing the program to the direct-LLVM backend / interpreter
/// (G2).
fn lower_program(
    node: &Node,
    ssa: &mut Ssa,
    body: &mut String,
) -> Result<(Lane, Option<String>), DialectError> {
    let anf = lower::lower_to_anf(node);
    lower_block(&anf, ssa, body)
}

/// Lower one ANF block (its bindings + result) into MLIR ops, returning the result lane and the
/// folded program-level overflow flag (`None` when no trit additive op is present, so an
/// overflow-free program emits exactly the M-601 module). The data / closure / recursion / swap
/// nodes are explicit refusals here (they live on the richer paths).
fn lower_block(
    anf: &Anf,
    ssa: &mut Ssa,
    body: &mut String,
) -> Result<(Lane, Option<String>), DialectError> {
    use std::collections::HashMap;
    let mut env: HashMap<Atom, Lane> = HashMap::new();
    // The per-op overflow `i1` registers, accumulated across the program. Any trit additive op
    // pushes its overflow condition here; the interpreter errors on the *first* overflow, so the
    // native path being conservative (OR of all of them ⇒ one explicit overflow) gives the same
    // verdict — the meaningless result is never read either way. Mirrors `crate::llvm`'s flags.
    let mut flags: Vec<String> = Vec::new();
    let lookup = |env: &HashMap<Atom, Lane>, a: &Atom| -> Result<Lane, DialectError> {
        env.get(a)
            .cloned()
            .ok_or_else(|| DialectError::FreeVariable(a.render()))
    };

    for b in anf.bindings() {
        let lane = match &b.rhs {
            Rhs::Const(v) => const_lane(v, ssa, body)?,
            Rhs::Alias(a) => lookup(&env, a)?,
            Rhs::Op { prim, args } => {
                let operands: Vec<Lane> = args
                    .iter()
                    .map(|a| lookup(&env, a))
                    .collect::<Result<_, _>>()?;
                let refs: Vec<&Lane> = operands.iter().collect();
                emit_op(prim, &refs, ssa, body, &mut flags)?
            }
            // Everything below is an explicit, never-silent refusal — it runs on the direct-LLVM
            // backend (`crate::llvm`, which covers the full v0 calculus) or the interpreter. The
            // message routes it there (EXPLAIN-able; G2/VR-5).
            Rhs::Swap { target, .. } => {
                return Err(DialectError::Unsupported(format!(
                    "Swap to {target:?}: representation swaps are not in the MLIR-dialect fragment \
                     (they run on the interpreter / direct-LLVM path)"
                )));
            }
            Rhs::Construct { .. } | Rhs::Match { .. } => {
                return Err(DialectError::Unsupported(
                    "the data fragment (Construct/Match) is not in the MLIR-dialect fragment — it is \
                     lowered by the direct-LLVM backend (crate::llvm; M-373) or interpreted"
                        .to_owned(),
                ));
            }
            Rhs::App { .. } | Rhs::Lam { .. } => {
                return Err(DialectError::Unsupported(
                    "closures (App/Lam) are not in the MLIR-dialect fragment — they are lowered by \
                     the direct-LLVM backend (crate::llvm; M-378) or interpreted"
                        .to_owned(),
                ));
            }
            Rhs::Fix { .. } | Rhs::FixGroup { .. } => {
                return Err(DialectError::Unsupported(
                    "recursion (Fix/FixGroup) is not in the MLIR-dialect fragment — tail recursion \
                     is lowered by the direct-LLVM backend (crate::llvm; M-379); the rest is \
                     interpreted"
                        .to_owned(),
                ));
            }
        };
        env.insert(b.name.clone(), lane);
    }
    let result = lookup(&env, anf.result())?;
    // Fold every per-op overflow `i1` into one program-level flag (or `None` — no trit additive op).
    let overflow = fold_or(&flags, ssa, body);
    Ok((result, overflow))
}

/// Emit the print sequence: one `func.call @putchar` per result element (its ASCII char), then a
/// newline `putchar`. The char codes match [`crate::llvm`]'s `emit_char_code` (Binary → `val + 48`;
/// Ternary → `-1→45 ('-')`, `0→48 ('0')`, `1→43 ('+')`) so the read-back is identical across paths.
fn emit_print(lane: &Lane, ssa: &mut Ssa, body: &mut String) {
    for v in &lane.vals {
        let code = emit_char_code(lane.kind, v, ssa, body);
        let r = ssa.fresh();
        let _ = writeln!(body, "    {r} = func.call @putchar({code}) : (i32) -> i32");
    }
    let nl = emit_const_i32(10, ssa, body);
    let r = ssa.fresh();
    let _ = writeln!(body, "    {r} = func.call @putchar({nl}) : (i32) -> i32");
}

/// Emit the `i32` ASCII char code for one element of `kind` (SSA `v`) using `arith` ops, returning
/// the SSA holding the code. The encoding is byte-for-byte the same as [`crate::llvm`]'s
/// `emit_char_code`, so a Binary/Ternary element prints the identical char on both compiled paths.
fn emit_char_code(kind: LaneKind, v: &str, ssa: &mut Ssa, body: &mut String) -> String {
    match kind {
        LaneKind::Binary => {
            let off = emit_const_i32(48, ssa, body);
            let c = ssa.fresh();
            let _ = writeln!(body, "    {c} = arith.addi {v}, {off} : i32");
            c
        }
        LaneKind::Ternary => {
            // isneg = (v == -1); ispos = (v == 1).
            let neg1 = emit_const_i32(-1, ssa, body);
            let pos1 = emit_const_i32(1, ssa, body);
            let isneg = ssa.fresh();
            let _ = writeln!(body, "    {isneg} = arith.cmpi eq, {v}, {neg1} : i32");
            let ispos = ssa.fresh();
            let _ = writeln!(body, "    {ispos} = arith.cmpi eq, {v}, {pos1} : i32");
            // t = ispos ? 43 ('+') : 48 ('0');  c = isneg ? 45 ('-') : t.
            let c43 = emit_const_i32(43, ssa, body);
            let c48 = emit_const_i32(48, ssa, body);
            let c45 = emit_const_i32(45, ssa, body);
            let t = ssa.fresh();
            let _ = writeln!(body, "    {t} = arith.select {ispos}, {c43}, {c48} : i32");
            let c = ssa.fresh();
            let _ = writeln!(body, "    {c} = arith.select {isneg}, {c45}, {t} : i32");
            c
        }
    }
}

/// Emit the full MLIR module for `node`: a `func.func private @putchar` declaration + a
/// `func.func @main` that computes the result lane, prints each element, and returns 0. Deterministic.
/// Returns an explicit [`DialectError::Unsupported`] for an out-of-fragment node (the program then
/// runs on the direct-LLVM backend / interpreter).
///
/// **Overflow read-back (M-725).** When the program contains a `trit.add`/`trit.sub` that can
/// overflow at runtime, `@main` branches (`cf.cond_br`) on the folded overflow `i1`: on overflow it
/// prints the shared [`OVERFLOW_SENTINEL`] line and returns 0; otherwise it prints the result line —
/// exactly mirroring [`crate::llvm`]'s read-back, so the artifact's stdout means the same on both
/// compiled paths. An **overflow-free** program (no trit additive op) emits the single-block,
/// straight-line module unchanged (byte-for-byte the M-601 shape) — the branch is added only when it
/// is needed.
///
/// The returned `(module, kind, width)` triple carries the lane shape so the read-back
/// ([`crate::llvm::decode_result`]) can parse `@main`'s stdout. Every op is explicit, dumpable MLIR
/// text — no opaque pass (RFC-0004 §6 / VR-4).
pub fn emit_mlir(node: &Node) -> Result<(String, ResultKind, usize), DialectError> {
    let mut ssa = Ssa::default();
    let mut body = String::new();
    let (result, overflow) = lower_program(node, &mut ssa, &mut body)?;

    let kind = ResultKind::from_lane(result.kind);
    let width = result.vals.len();

    let mut module = String::new();
    module.push_str("module {\n");
    module.push_str("  func.func private @putchar(i32) -> i32\n");
    module.push_str("  func.func @main() -> i32 {\n");

    match overflow {
        // No trit additive op ⇒ no overflow path; straight-line print + return (the M-601 module
        // unchanged, so element-wise programs emit byte-for-byte as before).
        None => {
            emit_print(&result, &mut ssa, &mut body);
            module.push_str(&body);
            let r = ssa.fresh();
            let _ = writeln!(module, "    {r} = arith.constant 0 : i32");
            let _ = writeln!(module, "    func.return {r} : i32");
        }
        // Overflow possible ⇒ branch on the runtime flag (`cf.cond_br`): print the sentinel line on
        // overflow, the result line otherwise. The read-back protocol — never a silent wrap (G2).
        // The entry block ends with the body (which computed the `ovf` i1) + the conditional branch;
        // `^ovf` prints the sentinel, `^ok` prints the result. Both return 0.
        Some(ovf) => {
            module.push_str(&body);
            let _ = writeln!(module, "    cf.cond_br {ovf}, ^ovf, ^ok");
            // ^ovf: print the OVERFLOW_SENTINEL char + newline, return 0.
            module.push_str("  ^ovf:\n");
            let sentinel = emit_const_i32(i32::from(OVERFLOW_SENTINEL), &mut ssa, &mut module);
            let s = ssa.fresh();
            let _ = writeln!(
                module,
                "    {s} = func.call @putchar({sentinel}) : (i32) -> i32"
            );
            let nl = emit_const_i32(10, &mut ssa, &mut module);
            let snl = ssa.fresh();
            let _ = writeln!(
                module,
                "    {snl} = func.call @putchar({nl}) : (i32) -> i32"
            );
            let zo = ssa.fresh();
            let _ = writeln!(module, "    {zo} = arith.constant 0 : i32");
            let _ = writeln!(module, "    func.return {zo} : i32");
            // ^ok: print the result line, return 0.
            module.push_str("  ^ok:\n");
            let mut ok = String::new();
            emit_print(&result, &mut ssa, &mut ok);
            module.push_str(&ok);
            let zk = ssa.fresh();
            let _ = writeln!(module, "    {zk} = arith.constant 0 : i32");
            let _ = writeln!(module, "    func.return {zk} : i32");
        }
    }

    module.push_str("  }\n}\n");
    Ok((module, kind, width))
}

// ─── The pipeline: MLIR module → real LLVM IR → native → read-back ────────────────────────────

/// Lower `node` through the real MLIR pipeline to **LLVM IR text**, without compiling/running it.
/// Emits the `arith`/`func`/`cf` MLIR module ([`emit_mlir`]), then runs
/// `mlir-opt --convert-cf-to-llvm --convert-func-to-llvm --convert-arith-to-llvm
/// --reconcile-unrealized-casts | mlir-translate --mlir-to-llvmir`. The `--convert-cf-to-llvm` pass
/// lowers the M-725 overflow-read-back `cf.cond_br` (a no-op for an overflow-free element-wise
/// program, which contains no `cf` ops). Each stage is a real libMLIR pass; the intermediate MLIR and
/// the resulting IR are both dumpable (no opaque pass — VR-4). Returns the LLVM IR text + lane shape,
/// or an explicit [`DialectError`] (skip on `ToolchainMissing`).
pub fn lower_to_llvm_ir(node: &Node) -> Result<(String, ResultKind, usize), DialectError> {
    let (mlir, kind, width) = emit_mlir(node)?;
    let tools = resolve_tools()?;

    let dir = unique_tmp_dir()?;
    let mlir_path = dir.join("kernel.mlir");
    let guard = TmpDir(dir);
    std::fs::write(&mlir_path, mlir.as_bytes())
        .map_err(|e| DialectError::Run(format!("write MLIR: {e}")))?;

    // Stage 1: mlir-opt lowers cf+func+arith → the LLVM dialect. `--convert-cf-to-llvm` handles the
    // M-725 overflow-read-back branch; it is a no-op for an overflow-free element-wise module.
    let lowered_mlir = run_capture(
        &tools.mlir_opt,
        &[
            "--convert-cf-to-llvm",
            "--convert-func-to-llvm",
            "--convert-arith-to-llvm",
            "--reconcile-unrealized-casts",
            path(&mlir_path)?,
        ],
        "mlir-opt",
    )?;

    // Stage 2: mlir-translate renders the LLVM-dialect module as textual LLVM IR.
    let lowered_path = guard.0.join("kernel.lowered.mlir");
    std::fs::write(&lowered_path, lowered_mlir.as_bytes())
        .map_err(|e| DialectError::Run(format!("write lowered MLIR: {e}")))?;
    let llvm_ir = run_capture(
        &tools.mlir_translate,
        &["--mlir-to-llvmir", path(&lowered_path)?],
        "mlir-translate",
    )?;

    Ok((llvm_ir, kind, width))
}

/// A compiled native artifact from the MLIR-dialect path: the executable on disk (cleaned up on
/// drop) plus the result shape needed to parse its output. Produced by [`compile`]; run with
/// [`Compiled::run`]. The **compile-once / run-many** split mirrors [`crate::llvm::CompiledArtifact`]
/// so a harness can time the one-time AOT cost separately from warm per-invocation cost (M-602).
pub struct Compiled {
    _dir: TmpDir,
    bin: std::path::PathBuf,
    kind: ResultKind,
    width: usize,
    llvm_major: u32,
}

impl Compiled {
    /// The LLVM major version the MLIR toolchain was matched to (for `EXPLAIN`/captions).
    #[must_use]
    pub fn llvm_major(&self) -> u32 {
        self.llvm_major
    }
    /// Execute the compiled artifact and read its result back as an `Exact` `Binary{w}`/`Ternary{m}`
    /// [`Value`] via the **shared** [`crate::llvm::decode_result`] (same read-back as the
    /// direct-LLVM path). A `trit.add`/`trit.sub` that overflowed prints the shared
    /// [`OVERFLOW_SENTINEL`] line ⇒ an explicit [`DialectError::Overflow`], never a silent wrap
    /// (M-725; mirrors [`crate::llvm::AotError::Overflow`] and the interpreter's `EvalError::Overflow`).
    pub fn run(&self) -> Result<Value, DialectError> {
        let output = Command::new(&self.bin)
            .output()
            .map_err(|e| DialectError::Run(format!("exec {}: {e}", self.bin.display())))?;
        if !output.status.success() {
            return Err(DialectError::Run(format!(
                "artifact exited {}",
                output.status
            )));
        }
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| DialectError::Parse(format!("non-utf8 output: {e}")))?;
        let line = stdout.lines().next().unwrap_or("");
        // Read-back protocol: the sentinel line means the native arithmetic overflowed the m-trit
        // range — an explicit error, never a silently-wrapped result (matches the interpreter's
        // `EvalError::Overflow` and the direct-LLVM path, so the three-way differential stays honest).
        if line.as_bytes() == [OVERFLOW_SENTINEL] {
            return Err(DialectError::Overflow(format!(
                "fixed-width result out of {}-trit range",
                self.width
            )));
        }
        decode_result(self.kind.to_lane(), self.width, line.chars())
            .map_err(|e| DialectError::Parse(e.to_string()))
    }
}

/// Compile `node` through the MLIR pipeline to a native executable (MLIR → LLVM IR → `clang`)
/// without running it. Returns [`DialectError::ToolchainMissing`] when the toolchain is absent so
/// callers can skip; any out-of-fragment construct is an explicit [`DialectError::Unsupported`].
pub fn compile(node: &Node) -> Result<Compiled, DialectError> {
    let (llvm_ir, kind, width) = lower_to_llvm_ir(node)?;
    let tools = resolve_tools()?;

    let dir = unique_tmp_dir()?;
    let ll = dir.join("kernel.ll");
    let bin = dir.join("kernel");
    let guard = TmpDir(dir);
    std::fs::write(&ll, llvm_ir.as_bytes())
        .map_err(|e| DialectError::Run(format!("write LLVM IR: {e}")))?;

    // clang compiles + links the textual LLVM IR directly to a native executable.
    run_ok(
        &tools.clang,
        &[path(&ll)?, "-o", path(&bin)?, "-Wno-override-module"],
        "clang",
    )?;

    Ok(Compiled {
        _dir: guard,
        bin,
        kind,
        width,
        llvm_major: tools.llvm_major,
    })
}

/// Compile + run `node` through the MLIR pipeline and read the result back. The convenience wrapper
/// over [`compile`] + [`Compiled::run`] — the **MLIR-dialect** execution path the M-602 three-way
/// differential checks against the interpreter and the direct-LLVM backend.
pub fn compile_and_run(node: &Node) -> Result<Value, DialectError> {
    compile(node)?.run()
}

// ─── subprocess plumbing (mirrors crate::llvm's tool-probe pattern) ───────────────────────────

/// Run a tool capturing stdout; a missing binary is [`DialectError::ToolchainMissing`] (skip), a
/// non-zero exit is [`DialectError::Compile`] with the captured stderr (no opaque failure).
fn run_capture(tool: &str, args: &[&str], stage: &str) -> Result<String, DialectError> {
    let out = Command::new(tool)
        .args(args)
        .output()
        .map_err(|_| DialectError::ToolchainMissing(tool.to_owned()))?;
    if out.status.success() {
        String::from_utf8(out.stdout)
            .map_err(|e| DialectError::Parse(format!("{stage}: non-utf8 stdout: {e}")))
    } else {
        Err(DialectError::Compile(format!(
            "{stage} ({tool} {}): {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        )))
    }
}

/// Run a tool for its side effect (no stdout needed); same never-silent error contract.
fn run_ok(tool: &str, args: &[&str], stage: &str) -> Result<(), DialectError> {
    let out = Command::new(tool)
        .args(args)
        .output()
        .map_err(|_| DialectError::ToolchainMissing(tool.to_owned()))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(DialectError::Compile(format!(
            "{stage} ({tool} {}): {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        )))
    }
}

fn path(p: &Path) -> Result<&str, DialectError> {
    p.to_str()
        .ok_or_else(|| DialectError::Run(format!("non-utf8 path {}", p.display())))
}

fn unique_tmp_dir() -> Result<std::path::PathBuf, DialectError> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("myc-mlir-{}-{nanos}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| DialectError::Run(format!("mkdir tmp: {e}")))?;
    Ok(dir)
}

/// Best-effort cleanup of the per-run temp dir.
struct TmpDir(std::path::PathBuf);
impl Drop for TmpDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
