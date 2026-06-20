//! The **real** ternary-dialect lowering (M-601; RFC-0004 §2; ADR-009/ADR-019).
//!
//! Feature-gated (`mlir-dialect`, OFF by default). For the **bit/trit element-wise straight-line
//! fragment** this emits a genuine MLIR module in the `func` + `arith` dialects and drives it
//! through the verified libMLIR pipeline
//!
//! ```text
//! mlir-opt-<v> --convert-func-to-llvm --convert-arith-to-llvm --reconcile-unrealized-casts
//!   | mlir-translate-<v> --mlir-to-llvmir
//! ```
//!
//! to **real LLVM IR**, then `clang` → native executable → run → read-back. It is a fourth,
//! genuinely MLIR-compiled execution path (not the textual [`super::emit`] skeleton, not the
//! [`crate::llvm`] direct-LLVM emitter, not the [`crate::aot`] env-machine).
//!
//! **Why this fragment, and only this fragment (RFC-0004 §2; VR-5/G2).** RFC-0004 §2 sequences the
//! AOT path as "`ternary` first … lowering progressively to `linalg`/`vector`/`arith`". The
//! element-wise bit/trit ops (`core.id`, `bit.not/and/or/xor`, `trit.neg`) are exactly the
//! sub-fragment the **standard** `arith` dialect carries faithfully — one `arith` op per element,
//! every op dumpable, the `mlir-opt` passes doing the real lowering. Everything richer (trit
//! *carry* arithmetic with its overflow read-back, the `Construct`/`Match` data fragment, closures,
//! recursion, `Swap`, Dense/VSA) is an **explicit, never-silent** [`DialectError::Unsupported`]
//! that routes the program to the direct-LLVM backend ([`crate::llvm`]) or the interpreter — which
//! already cover the full v0 calculus. We do **not** ship a second, divergent carry/closure codegen
//! here just to widen coverage: that would be two sources of truth for the same semantics (DRY) and
//! a fragility risk (G2). The honest boundary is an explicit refusal, not silent or fragile output.
//!
//! **Read-back protocol — shared with [`crate::llvm`] (single contract).** The emitted `@main`
//! `putchar`s each result element's ASCII char (`'0'`/`'1'` for bits; `'-'`/`'0'`/`'+'` for trits)
//! then a newline, and the result is decoded by the **same** [`crate::llvm::decode_result`] the
//! direct-LLVM path uses. So the MLIR-dialect output and the direct-LLVM output are read back
//! identically — the three-way differential (M-602) compares like with like.
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

use crate::llvm::{decode_result, LaneKind};

/// An explicit failure of the real MLIR-dialect path. Every unsupported construct, missing tool, or
/// subprocess failure is one of these — the path is **never silent** (G2). Mirrors the contract of
/// [`crate::llvm::AotError`], specialized to the MLIR pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialectError {
    /// A node/prim/repr outside the bit/trit element-wise fragment the standard `arith` dialect
    /// lowers here. The message names what was refused and where it should run instead (the
    /// direct-LLVM backend [`crate::llvm`] or the interpreter) — an `EXPLAIN`-able routing, never a
    /// silent drop (G2/VR-5).
    Unsupported(String),
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

/// Detect the installed LLVM major version from `llc --version` (the line `… LLVM version NN.…`),
/// falling back to `clang --version`. Returns `None` when neither tool is present or the version
/// line cannot be parsed — the caller turns that into a graceful skip.
fn detect_llvm_major() -> Option<u32> {
    fn parse_major(s: &str) -> Option<u32> {
        // Find a token of the form "NN.MM.PP" or "NN.MM" on a "LLVM version" line.
        for line in s.lines() {
            if let Some(idx) = line.find("LLVM version") {
                let rest = &line[idx + "LLVM version".len()..];
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
        None
    }
    for tool in ["llc", "clang"] {
        if let Ok(out) = Command::new(tool).arg("--version").output() {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(major) = parse_major(&text) {
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
    let mlir_opt = if tool_present(&opt_versioned) {
        opt_versioned
    } else if tool_present("mlir-opt") {
        "mlir-opt".to_owned()
    } else {
        return Err(DialectError::ToolchainMissing(format!(
            "mlir-opt-{major} (and unversioned mlir-opt) — run scripts/setup-mlir.sh"
        )));
    };
    let mlir_translate = if tool_present(&tr_versioned) {
        tr_versioned
    } else if tool_present("mlir-translate") {
        "mlir-translate".to_owned()
    } else {
        return Err(DialectError::ToolchainMissing(format!(
            "mlir-translate-{major} (and unversioned mlir-translate) — run scripts/setup-mlir.sh"
        )));
    };
    let clang_versioned = format!("clang-{major}");
    let clang = if tool_present(&clang_versioned) {
        clang_versioned
    } else if tool_present("clang") {
        "clang".to_owned()
    } else {
        return Err(DialectError::ToolchainMissing("clang".to_owned()));
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

/// Lower one element-wise primitive over its operand lanes to `arith` ops, returning the result
/// lane. Anything outside the element-wise fragment — notably the trit *carry* ops
/// (`trit.add/sub/mul`, which need a ripple-carry + an overflow read-back already proven on the
/// direct-LLVM path) — is an explicit [`DialectError::Unsupported`] routing to [`crate::llvm`] (G2).
fn emit_op(
    prim: &str,
    operands: &[&Lane],
    ssa: &mut Ssa,
    body: &mut String,
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
        // Trit *carry* arithmetic is deliberately NOT lowered here: it needs a fixed-width
        // ripple-carry + a runtime overflow read-back, already implemented and differential-proven
        // on the direct-LLVM path (`crate::llvm`). Re-emitting it in `arith` would be a second
        // source of truth for the carry semantics (DRY) and a fragility risk (G2). Explicit refusal.
        "trit.add" | "trit.sub" | "trit.mul" => Err(DialectError::Unsupported(format!(
            "{prim}: trit carry arithmetic is not in the MLIR element-wise fragment — it runs on the \
             direct-LLVM backend (crate::llvm), which carries the ripple-carry + overflow read-back"
        ))),
        other => Err(DialectError::Unsupported(format!(
            "primitive {other:?} is not in the MLIR element-wise fragment (bit.not/and/or/xor, \
             trit.neg, core.id) — it runs on the direct-LLVM backend / interpreter"
        ))),
    }
}

/// Walk the lowered ANF, emitting one `arith` op per binding into `@main`'s body, and return the
/// result lane. Returns an explicit [`DialectError::Unsupported`] for any node outside the
/// element-wise fragment — routing the program to the direct-LLVM backend / interpreter (G2).
fn lower_program(node: &Node, ssa: &mut Ssa, body: &mut String) -> Result<Lane, DialectError> {
    let anf = lower::lower_to_anf(node);
    lower_block(&anf, ssa, body)
}

/// Lower one ANF block (its bindings + result) into MLIR ops, returning the result lane. The data /
/// closure / recursion / swap nodes are explicit refusals here (they live on the richer paths).
fn lower_block(anf: &Anf, ssa: &mut Ssa, body: &mut String) -> Result<Lane, DialectError> {
    use std::collections::HashMap;
    let mut env: HashMap<Atom, Lane> = HashMap::new();
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
                emit_op(prim, &refs, ssa, body)?
            }
            // Everything below is an explicit, never-silent refusal — it runs on the direct-LLVM
            // backend (`crate::llvm`, which covers the full v0 calculus) or the interpreter. The
            // message routes it there (EXPLAIN-able; G2/VR-5).
            Rhs::Swap { target, .. } => {
                return Err(DialectError::Unsupported(format!(
                    "Swap to {target:?}: representation swaps are not in the MLIR element-wise \
                     fragment (they run on the interpreter / direct-LLVM path)"
                )));
            }
            Rhs::Construct { .. } | Rhs::Match { .. } => {
                return Err(DialectError::Unsupported(
                    "the data fragment (Construct/Match) is not in the MLIR element-wise fragment — \
                     it is lowered by the direct-LLVM backend (crate::llvm; M-373) or interpreted"
                        .to_owned(),
                ));
            }
            Rhs::App { .. } | Rhs::Lam { .. } => {
                return Err(DialectError::Unsupported(
                    "closures (App/Lam) are not in the MLIR element-wise fragment — they are \
                     lowered by the direct-LLVM backend (crate::llvm; M-378) or interpreted"
                        .to_owned(),
                ));
            }
            Rhs::Fix { .. } | Rhs::FixGroup { .. } => {
                return Err(DialectError::Unsupported(
                    "recursion (Fix/FixGroup) is not in the MLIR element-wise fragment — tail \
                     recursion is lowered by the direct-LLVM backend (crate::llvm; M-379); the rest \
                     is interpreted"
                        .to_owned(),
                ));
            }
        };
        env.insert(b.name.clone(), lane);
    }
    lookup(&env, anf.result())
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

/// Emit the full MLIR module for `node` (the element-wise fragment): a `func.func private @putchar`
/// declaration + a `func.func @main` that computes the result lane, prints each element, and
/// returns 0. Deterministic. Returns an explicit [`DialectError::Unsupported`] for an out-of-fragment
/// node (the program then runs on the direct-LLVM backend / interpreter).
///
/// The returned `(module, kind, width)` triple carries the lane shape so the read-back
/// ([`crate::llvm::decode_result`]) can parse `@main`'s stdout. Every op is explicit, dumpable MLIR
/// text — no opaque pass (RFC-0004 §6 / VR-4).
pub fn emit_mlir(node: &Node) -> Result<(String, ResultKind, usize), DialectError> {
    let mut ssa = Ssa::default();
    let mut body = String::new();
    let result = lower_program(node, &mut ssa, &mut body)?;
    emit_print(&result, &mut ssa, &mut body);

    let mut module = String::new();
    module.push_str("module {\n");
    module.push_str("  func.func private @putchar(i32) -> i32\n");
    module.push_str("  func.func @main() -> i32 {\n");
    module.push_str(&body);
    let zero = {
        // The return value 0 — emitted last so it cannot clash with body SSA names.
        let r = ssa.fresh();
        let _ = writeln!(module, "    {r} = arith.constant 0 : i32");
        format!("    func.return {r} : i32\n")
    };
    module.push_str(&zero);
    module.push_str("  }\n}\n");

    let kind = ResultKind::from_lane(result.kind);
    let width = result.vals.len();
    Ok((module, kind, width))
}

// ─── The pipeline: MLIR module → real LLVM IR → native → read-back ────────────────────────────

/// Lower `node` through the real MLIR pipeline to **LLVM IR text**, without compiling/running it.
/// Emits the `arith`/`func` MLIR module ([`emit_mlir`]), then runs
/// `mlir-opt --convert-func-to-llvm --convert-arith-to-llvm --reconcile-unrealized-casts
/// | mlir-translate --mlir-to-llvmir`. Each stage is a real libMLIR pass; the intermediate MLIR and
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

    // Stage 1: mlir-opt lowers func+arith → the LLVM dialect.
    let lowered_mlir = run_capture(
        &tools.mlir_opt,
        &[
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
    /// direct-LLVM path).
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

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_core::{Meta, Provenance};

    fn byte(bits: [bool; 8]) -> Value {
        Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(bits.to_vec()),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    const A: [bool; 8] = [true, false, true, true, false, false, true, false];

    fn not_a_xor_b() -> Node {
        let b = byte([false, false, true, false, true, false, true, true]);
        Node::Op {
            prim: "bit.not".into(),
            args: vec![Node::Op {
                prim: "bit.xor".into(),
                args: vec![Node::Const(byte(A)), Node::Const(b)],
            }],
        }
    }

    #[test]
    fn emits_a_real_arith_func_module() {
        let (m, kind, width) = emit_mlir(&not_a_xor_b()).expect("emit");
        assert!(m.starts_with("module {"));
        assert!(m.contains("func.func @main()"));
        assert!(m.contains("func.func private @putchar"));
        // Real arith ops (the lowering, not the textual skeleton):
        assert!(m.contains("arith.xori"), "expected arith.xori in:\n{m}");
        assert!(m.contains("func.call @putchar"));
        assert!(m.contains("func.return"));
        assert_eq!(kind, ResultKind::Binary);
        assert_eq!(width, 8);
    }

    #[test]
    fn emission_is_deterministic() {
        assert_eq!(
            emit_mlir(&not_a_xor_b()).unwrap().0,
            emit_mlir(&not_a_xor_b()).unwrap().0
        );
    }

    #[test]
    fn out_of_fragment_nodes_are_explicitly_refused() {
        // A Swap is refused (routed to interp / direct-LLVM), never silently lowered.
        let swap = Node::Swap {
            src: Box::new(Node::Const(byte(A))),
            target: Repr::Ternary { trits: 6 },
            policy: mycelium_core::ContentHash::parse("blake3:round_trip_safe").unwrap(),
        };
        match emit_mlir(&swap) {
            Err(DialectError::Unsupported(_)) => {}
            other => panic!("Swap must be Unsupported, got {other:?}"),
        }
        // Trit carry arithmetic is refused (stays on the direct-LLVM path).
        let add = Node::Op {
            prim: "trit.add".into(),
            args: vec![
                Node::Const(
                    Value::new(
                        Repr::Ternary { trits: 3 },
                        Payload::Trits(vec![Trit::Pos, Trit::Neg, Trit::Neg]),
                        Meta::exact(Provenance::Root),
                    )
                    .unwrap(),
                ),
                Node::Const(
                    Value::new(
                        Repr::Ternary { trits: 3 },
                        Payload::Trits(vec![Trit::Zero, Trit::Pos, Trit::Pos]),
                        Meta::exact(Provenance::Root),
                    )
                    .unwrap(),
                ),
            ],
        };
        match emit_mlir(&add) {
            Err(DialectError::Unsupported(_)) => {}
            other => panic!("trit.add must be Unsupported, got {other:?}"),
        }
    }

    #[test]
    fn toolchain_resolves_or_skips() {
        // Either the tools resolve (this container) or we get a graceful ToolchainMissing — never a
        // panic, never a silent mismatch.
        match resolve_tools() {
            Ok(t) => {
                assert!(t.mlir_opt.contains("mlir-opt"));
                assert!(t.mlir_translate.contains("mlir-translate"));
                assert!(t.llvm_major >= 1);
            }
            Err(DialectError::ToolchainMissing(_)) => {}
            Err(e) => panic!("unexpected toolchain error: {e}"),
        }
    }
}
