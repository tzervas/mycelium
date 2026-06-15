//! Direct-LLVM-IR AOT backend for the kernel **bit/trit subset** (M-301; RFC-0004 §2 *direct-LLVM
//! fallback*; ADR-007/009; phase-3.md §1/§9.1).
//!
//! **Scope / honesty.** The ratified AOT path is `MLIR → LLVM` (RFC-0004 §2), but libMLIR is absent
//! in this environment while LLVM 18 tooling (`llc`, `clang`) is present. RFC-0004 §2 explicitly
//! anticipates *"a lighter direct-LLVM backend"* as the revisit; this module is that backend, scoped
//! to a **bit/trit subset**: `core.id`, `bit.not/and/or/xor` over `Binary{w}`, and `trit.neg` over
//! `Ternary{m}`. It is a *genuinely compiled native artifact* — not the textual `dialect::emit`
//! skeleton, and not the `aot::run` env-machine: [`emit_llvm_ir`] renders textual LLVM IR (one op
//! per output element, so nothing is opaque — RFC-0004 §6), and [`compile_and_run`] drives `llc` +
//! `clang` to a native executable, runs it, and reads the result back. This is the third,
//! *compiled*, execution path; the interp↔native differential (M-302) checks it against the
//! reference interpreter (NFR-7/RR-12).
//!
//! **Deliberately out of subset (explicit refusals, never silent — G2):** balanced-ternary **carry
//! arithmetic** (`trit.add/sub/mul` — the re-encode/overflow handling is the next M-301 slice),
//! swaps, and Dense/VSA representations. Each is an explicit [`AotError`]. The MLIR dialect path
//! stays the eventual home (`dialect::emit` is its dumpable skeleton), deferred until libMLIR exists.

use std::collections::HashMap;
use std::fmt;
use std::fmt::Write as _; // `writeln!` into a String never fails — call sites discard the Result.
use std::path::Path;
use std::process::Command;

use mycelium_core::lower::{self, Atom, Rhs};
use mycelium_core::{Meta, Node, Payload, Provenance, Repr, Trit, Value};

/// An explicit failure of the direct-LLVM AOT path. Every non-supported construct, missing tool, or
/// subprocess failure is one of these — the path is **never silent** (G2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AotError {
    /// A representation outside the subset (only `Binary{w}` / `Ternary{m}` are supported here).
    UnsupportedRepr(String),
    /// A primitive outside the subset (`core.id`, `bit.not/and/or/xor`, `trit.neg`).
    UnsupportedPrim(String),
    /// A Core IR construct the subset backend does not lower (e.g. a swap).
    UnsupportedNode(String),
    /// An operand atom with no prior binding (an ill-formed lowering).
    FreeVariable(String),
    /// A binary op over mismatched widths.
    WidthMismatch {
        /// The primitive name.
        prim: String,
        /// First operand width.
        a: usize,
        /// Second operand width.
        b: usize,
    },
    /// The native toolchain (`llc`/`clang`) is not installed — callers should **skip**, not fail
    /// (the house "skip gracefully when a tool is absent" idiom).
    ToolchainMissing(String),
    /// `llc`/`clang` ran but returned a non-zero status (compile failure).
    Compile(String),
    /// The compiled artifact failed to run or produced unreadable output.
    Run(String),
    /// The native stdout did not parse back into the expected payload shape.
    Parse(String),
    /// Reconstructing the result [`Value`] failed its well-formedness check.
    Wf(String),
}

impl fmt::Display for AotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AotError::UnsupportedRepr(r) => write!(f, "unsupported repr for the AOT subset: {r}"),
            AotError::UnsupportedPrim(p) => write!(f, "unsupported prim for the AOT subset: {p}"),
            AotError::UnsupportedNode(n) => write!(f, "unsupported node for the AOT subset: {n}"),
            AotError::FreeVariable(v) => write!(f, "free variable in lowered IR: {v}"),
            AotError::WidthMismatch { prim, a, b } => {
                write!(f, "{prim}: width mismatch {a} vs {b}")
            }
            AotError::ToolchainMissing(t) => write!(f, "native toolchain missing: {t}"),
            AotError::Compile(e) => write!(f, "native compile failed: {e}"),
            AotError::Run(e) => write!(f, "native run failed: {e}"),
            AotError::Parse(e) => write!(f, "native output parse failed: {e}"),
            AotError::Wf(e) => write!(f, "result not well-formed: {e}"),
        }
    }
}

impl std::error::Error for AotError {}

/// One element (a bit or a trit), as an LLVM `i32` operand: a literal (`"0"`/`"1"`/`"-1"`) or an
/// SSA register (`"%r3"`).
type Operand = String;

/// Which representation a lane carries — fixes how its elements are computed and printed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LaneKind {
    /// `Binary{w}` — elements in `{0, 1}`, printed `'0'`/`'1'`.
    Binary,
    /// `Ternary{m}` — balanced-ternary elements in `{-1, 0, 1}`, printed `'-'`/`'0'`/`'+'`.
    Ternary,
}

/// A computed value lane: its representation kind and one `i32` operand per element.
#[derive(Debug, Clone)]
struct Lane {
    kind: LaneKind,
    vals: Vec<Operand>,
}

/// SSA-name generator for the emitted IR (monotone counter → deterministic names).
struct Ssa(usize);
impl Ssa {
    fn fresh(&mut self) -> String {
        let n = self.0;
        self.0 += 1;
        format!("%r{n}")
    }
}

/// The lowered program: the emitted op `body`, the `result` lane, and the SSA counter to continue
/// from. The **single source of truth** for both [`emit_llvm_ir`] and [`result_shape`] — so the
/// shape used to parse the native output can never disagree with what was emitted.
struct Lowered {
    body: String,
    result: Lane,
    ssa: Ssa,
}

/// Walk the lowered ANF, emitting one op per binding, and return the result lane. Returns an
/// explicit [`AotError`] for anything outside the bit/trit subset.
fn lower_program(node: &Node) -> Result<Lowered, AotError> {
    let anf = lower::lower_to_anf(node);
    let mut env: HashMap<Atom, Lane> = HashMap::new();
    let mut ssa = Ssa(0);
    let mut body = String::new();

    for b in anf.bindings() {
        let lane = match &b.rhs {
            Rhs::Const(v) => const_lane(v)?,
            Rhs::Alias(a) => lookup(&env, a)?.clone(),
            Rhs::Op { prim, args } => {
                let operands: Vec<&Lane> = args
                    .iter()
                    .map(|a| lookup(&env, a))
                    .collect::<Result<_, _>>()?;
                emit_op(prim, &operands, &mut ssa, &mut body)?
            }
            Rhs::Swap { target, .. } => {
                return Err(AotError::UnsupportedNode(format!(
                    "swap to {target:?} (the subset is straight-line bit/trit ops; M-301)"
                )));
            }
        };
        env.insert(b.name.clone(), lane);
    }

    let result = lookup(&env, anf.result())?.clone();
    Ok(Lowered { body, result, ssa })
}

/// Emit textual LLVM IR for the bit/trit-subset program `node` — a `main` that computes the result
/// elements and writes them as a line to stdout (Binary: `'0'`/`'1'`; Ternary: `'-'`/`'0'`/`'+'`).
/// Deterministic. One op per output element (no opaque pass — RFC-0004 §6). Returns an explicit
/// [`AotError`] for anything outside the subset.
pub fn emit_llvm_ir(node: &Node) -> Result<String, AotError> {
    let Lowered {
        body,
        result,
        mut ssa,
    } = lower_program(node)?;
    let mut out = String::from("; mycelium direct-LLVM AOT (bit/trit subset; M-301)\n");
    out.push_str("declare i32 @putchar(i32)\n\n");
    out.push_str("define i32 @main() {\nentry:\n");
    out.push_str(&body);
    // Emit each result element as its ASCII char, then a newline. Binary: '0'/'1' (val+48).
    // Ternary: '-'(45)/'0'(48)/'+'(43) via a branch-free select chain — one op per element, so the
    // printout stays a transparent rendering of the computed lane (no opaque pass, RFC-0004 §6).
    for v in &result.vals {
        let c = match result.kind {
            LaneKind::Binary => {
                let c = ssa.fresh();
                let _ = writeln!(&mut out, "  {c} = add i32 {v}, 48");
                c
            }
            LaneKind::Ternary => {
                let isneg = ssa.fresh();
                let _ = writeln!(&mut out, "  {isneg} = icmp eq i32 {v}, -1");
                let ispos = ssa.fresh();
                let _ = writeln!(&mut out, "  {ispos} = icmp eq i32 {v}, 1");
                let t = ssa.fresh();
                let _ = writeln!(&mut out, "  {t} = select i1 {ispos}, i32 43, i32 48");
                let c = ssa.fresh();
                let _ = writeln!(&mut out, "  {c} = select i1 {isneg}, i32 45, i32 {t}");
                c
            }
        };
        let p = ssa.fresh();
        let _ = writeln!(&mut out, "  {p} = call i32 @putchar(i32 {c})");
    }
    let nl = ssa.fresh();
    let _ = writeln!(&mut out, "  {nl} = call i32 @putchar(i32 10)");
    out.push_str("  ret i32 0\n}\n");
    Ok(out)
}

/// The result shape (lane kind + element count) of the program — **derived from the actual
/// lowering** ([`lower_program`]) so it can never disagree with what [`emit_llvm_ir`] emits. Used by
/// [`compile`] to know how to parse the native output.
fn result_shape(node: &Node) -> Result<(LaneKind, usize), AotError> {
    let l = lower_program(node)?;
    Ok((l.result.kind, l.result.vals.len()))
}

fn lookup<'a, T>(env: &'a HashMap<Atom, T>, a: &Atom) -> Result<&'a T, AotError> {
    env.get(a).ok_or_else(|| AotError::FreeVariable(a.render()))
}

/// The const's elements as `i32` literal operands + its lane kind, or an explicit refusal for an
/// unsupported repr (Dense/VSA).
fn const_lane(v: &Value) -> Result<Lane, AotError> {
    match (v.repr(), v.payload()) {
        (Repr::Binary { .. }, Payload::Bits(b)) => Ok(Lane {
            kind: LaneKind::Binary,
            vals: b
                .iter()
                .map(|&x| if x { "1" } else { "0" }.to_owned())
                .collect(),
        }),
        (Repr::Ternary { .. }, Payload::Trits(t)) => Ok(Lane {
            kind: LaneKind::Ternary,
            vals: t
                .iter()
                .map(|&x| {
                    match x {
                        Trit::Neg => "-1",
                        Trit::Zero => "0",
                        Trit::Pos => "1",
                    }
                    .to_owned()
                })
                .collect(),
        }),
        (repr, _) => Err(AotError::UnsupportedRepr(format!("{repr:?}"))),
    }
}

/// Require a lane to be of the expected kind, else an explicit refusal (a `bit.*` op on a ternary
/// lane, or `trit.*` on a binary one, is a type error — never silently mis-lowered).
fn require_kind(prim: &str, got: LaneKind, want: LaneKind) -> Result<(), AotError> {
    if got == want {
        Ok(())
    } else {
        Err(AotError::UnsupportedPrim(format!(
            "{prim} expects a {want:?} operand, got {got:?}"
        )))
    }
}

/// Emit the elementwise LLVM IR for one bit/trit-subset op, returning the result lane.
fn emit_op(
    prim: &str,
    operands: &[&Lane],
    ssa: &mut Ssa,
    body: &mut String,
) -> Result<Lane, AotError> {
    match prim {
        // Identity passes the lane through unchanged, any kind (M-I1 passthrough).
        "core.id" => {
            let [a] = arity1(prim, operands)?;
            Ok((*a).clone())
        }
        "bit.not" => {
            let [a] = arity1(prim, operands)?;
            require_kind(prim, a.kind, LaneKind::Binary)?;
            Ok(map1(a, ssa, body, |x, r| format!("  {r} = xor i32 {x}, 1")))
        }
        "bit.and" | "bit.or" | "bit.xor" => {
            let (a, b) = arity2(prim, operands)?;
            require_kind(prim, a.kind, LaneKind::Binary)?;
            require_kind(prim, b.kind, LaneKind::Binary)?;
            let instr = match prim {
                "bit.and" => "and",
                "bit.or" => "or",
                _ => "xor",
            };
            map2(prim, a, b, ssa, body, |x, y, r| {
                format!("  {r} = {instr} i32 {x}, {y}")
            })
        }
        // Balanced-ternary negation is digit-wise (`-t`), exact, no carry — `0 - x` per trit.
        "trit.neg" => {
            let [a] = arity1(prim, operands)?;
            require_kind(prim, a.kind, LaneKind::Ternary)?;
            Ok(map1(a, ssa, body, |x, r| format!("  {r} = sub i32 0, {x}")))
        }
        // Trit arithmetic with carry/overflow (add/sub/mul) is the next M-301 slice — refused
        // explicitly here, never half-lowered (G2).
        "trit.add" | "trit.sub" | "trit.mul" => Err(AotError::UnsupportedPrim(format!(
            "{prim}: balanced-ternary carry arithmetic is the next M-301 slice (not yet lowered)"
        ))),
        other => Err(AotError::UnsupportedPrim(other.to_owned())),
    }
}

/// Emit one IR instruction per element of `a`, returning the result lane (same kind as `a`).
fn map1(a: &Lane, ssa: &mut Ssa, body: &mut String, f: impl Fn(&str, &str) -> String) -> Lane {
    let vals = a
        .vals
        .iter()
        .map(|x| {
            let r = ssa.fresh();
            let _ = writeln!(body, "{}", f(x, &r));
            r
        })
        .collect();
    Lane { kind: a.kind, vals }
}

/// Emit one IR instruction per element pair of `a`/`b` (widths must match), returning the result
/// lane (same kind as `a`).
fn map2(
    prim: &str,
    a: &Lane,
    b: &Lane,
    ssa: &mut Ssa,
    body: &mut String,
    f: impl Fn(&str, &str, &str) -> String,
) -> Result<Lane, AotError> {
    if a.vals.len() != b.vals.len() {
        return Err(AotError::WidthMismatch {
            prim: prim.to_owned(),
            a: a.vals.len(),
            b: b.vals.len(),
        });
    }
    let vals = a
        .vals
        .iter()
        .zip(&b.vals)
        .map(|(x, y)| {
            let r = ssa.fresh();
            let _ = writeln!(body, "{}", f(x, y, &r));
            r
        })
        .collect();
    Ok(Lane { kind: a.kind, vals })
}

fn arity1<'a>(prim: &str, ops: &[&'a Lane]) -> Result<[&'a Lane; 1], AotError> {
    match ops {
        [a] => Ok([a]),
        _ => Err(AotError::UnsupportedPrim(format!(
            "{prim} expects 1 operand, got {}",
            ops.len()
        ))),
    }
}

fn arity2<'a>(prim: &str, ops: &[&'a Lane]) -> Result<(&'a Lane, &'a Lane), AotError> {
    match ops {
        [a, b] => Ok((a, b)),
        _ => Err(AotError::UnsupportedPrim(format!(
            "{prim} expects 2 operands, got {}",
            ops.len()
        ))),
    }
}

/// A compiled native artifact for a bit/trit-subset program: the executable on disk (in a
/// per-artifact temp dir, cleaned up on drop) plus the result shape (lane kind + element count)
/// needed to parse its output. Produced by [`compile`]; run any number of times with
/// [`CompiledArtifact::run`]. The **compile-once / run-many** split is the natural AOT shape and lets
/// a harness time the one-time AOT cost separately from warm per-invocation cost (the E1 perf
/// measurement, M-303).
pub struct CompiledArtifact {
    _dir: TmpDir,
    bin: std::path::PathBuf,
    kind: LaneKind,
    width: usize,
}

impl CompiledArtifact {
    /// Execute the compiled artifact and read its result back as an `Exact` `Binary{w}`/`Ternary{m}`
    /// [`Value`] (bit/`neg` ops are exact; the subset refuses approximate inputs).
    pub fn run(&self) -> Result<Value, AotError> {
        let output = Command::new(&self.bin)
            .output()
            .map_err(|e| AotError::Run(format!("exec {}: {e}", self.bin.display())))?;
        if !output.status.success() {
            return Err(AotError::Run(format!("artifact exited {}", output.status)));
        }
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| AotError::Parse(format!("non-utf8 output: {e}")))?;
        let line = stdout.lines().next().unwrap_or("");
        if line.chars().count() != self.width {
            return Err(AotError::Parse(format!(
                "expected {} elements, got {} ({line:?})",
                self.width,
                line.chars().count()
            )));
        }
        match self.kind {
            LaneKind::Binary => {
                let bits: Vec<bool> = line
                    .chars()
                    .map(|c| match c {
                        '0' => Ok(false),
                        '1' => Ok(true),
                        other => Err(AotError::Parse(format!("non-bit char {other:?}"))),
                    })
                    .collect::<Result<_, _>>()?;
                Value::new(
                    Repr::Binary {
                        width: self.width as u32,
                    },
                    Payload::Bits(bits),
                    Meta::exact(Provenance::Root),
                )
                .map_err(|e| AotError::Wf(e.to_string()))
            }
            LaneKind::Ternary => {
                let trits: Vec<Trit> = line
                    .chars()
                    .map(|c| match c {
                        '-' => Ok(Trit::Neg),
                        '0' => Ok(Trit::Zero),
                        '+' => Ok(Trit::Pos),
                        other => Err(AotError::Parse(format!("non-trit char {other:?}"))),
                    })
                    .collect::<Result<_, _>>()?;
                Value::new(
                    Repr::Ternary {
                        trits: self.width as u32,
                    },
                    Payload::Trits(trits),
                    Meta::exact(Provenance::Root),
                )
                .map_err(|e| AotError::Wf(e.to_string()))
            }
        }
    }
}

/// Compile the bit/trit-subset program to a native executable (emit LLVM IR → `llc` → `clang`)
/// without running it. Returns [`AotError::ToolchainMissing`] when `llc`/`clang` are absent so
/// callers can skip; any out-of-subset construct is the same explicit refusal as [`emit_llvm_ir`].
pub fn compile(node: &Node) -> Result<CompiledArtifact, AotError> {
    let ir = emit_llvm_ir(node)?;
    let (kind, width) = result_shape(node)?;
    ensure_toolchain()?;

    let dir = unique_tmp_dir()?;
    let ll = dir.join("kernel.ll");
    let obj = dir.join("kernel.o");
    let bin = dir.join("kernel");
    let guard = TmpDir(dir);

    std::fs::write(&ll, ir.as_bytes()).map_err(|e| AotError::Run(format!("write IR: {e}")))?;
    run_tool("llc", &["-filetype=obj", path(&ll)?, "-o", path(&obj)?])?;
    run_tool("clang", &[path(&obj)?, "-o", path(&bin)?])?;

    Ok(CompiledArtifact {
        _dir: guard,
        bin,
        kind,
        width,
    })
}

/// Compile the bit/trit-subset program to a native executable, run it once, and read the result
/// back. The convenience wrapper over [`compile`] + [`CompiledArtifact::run`]; this is the
/// **compiled** execution path the M-302 differential checks against the interpreter.
pub fn compile_and_run(node: &Node) -> Result<Value, AotError> {
    compile(node)?.run()
}

fn ensure_toolchain() -> Result<(), AotError> {
    for tool in ["llc", "clang"] {
        Command::new(tool)
            .arg("--version")
            .output()
            .map_err(|_| AotError::ToolchainMissing(tool.to_owned()))?;
    }
    Ok(())
}

fn run_tool(tool: &str, args: &[&str]) -> Result<(), AotError> {
    let out = Command::new(tool)
        .args(args)
        .output()
        .map_err(|_| AotError::ToolchainMissing(tool.to_owned()))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(AotError::Compile(format!(
            "{tool} {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        )))
    }
}

fn path(p: &Path) -> Result<&str, AotError> {
    p.to_str()
        .ok_or_else(|| AotError::Run(format!("non-utf8 path {}", p.display())))
}

fn unique_tmp_dir() -> Result<std::path::PathBuf, AotError> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("myc-aot-{}-{nanos}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| AotError::Run(format!("mkdir tmp: {e}")))?;
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
    use mycelium_core::Repr;

    fn binary(bits: Vec<bool>) -> Value {
        let width = bits.len() as u32;
        Value::new(
            Repr::Binary { width },
            Payload::Bits(bits),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    fn ternary(trits: Vec<Trit>) -> Value {
        let m = trits.len() as u32;
        Value::new(
            Repr::Ternary { trits: m },
            Payload::Trits(trits),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    fn not_program() -> Node {
        Node::Op {
            prim: "bit.not".into(),
            args: vec![Node::Const(binary(vec![true, false, true, true]))],
        }
    }

    fn neg_program() -> Node {
        Node::Op {
            prim: "trit.neg".into(),
            args: vec![Node::Const(ternary(vec![Trit::Pos, Trit::Zero, Trit::Neg]))],
        }
    }

    #[test]
    fn emits_module_for_bit_not() {
        let ir = emit_llvm_ir(&not_program()).unwrap();
        assert!(ir.contains("declare i32 @putchar(i32)"));
        assert!(ir.contains("define i32 @main()"));
        assert!(ir.contains("xor i32")); // bit.not lowers to xor with 1
        assert!(ir.contains("call i32 @putchar"));
        assert!(ir.contains("ret i32 0"));
    }

    #[test]
    fn emission_is_deterministic() {
        assert_eq!(emit_llvm_ir(&not_program()), emit_llvm_ir(&not_program()));
    }

    #[test]
    fn emits_module_for_trit_neg() {
        let ir = emit_llvm_ir(&neg_program()).unwrap();
        assert!(ir.contains("sub i32 0,")); // trit.neg lowers to 0 - x per trit
                                            // Ternary output uses the '-'(45)/'0'(48)/'+'(43) select chain.
        assert!(ir.contains("select i1") && ir.contains("i32 45") && ir.contains("i32 43"));
        assert!(ir.contains("ret i32 0"));
    }

    #[test]
    fn ternary_const_is_supported() {
        // M-301 trit slice: a Ternary const is now lowered (was UnsupportedRepr in the bit-only
        // slice). Mutant-witness: reverting const_lane to Binary-only would refuse this.
        let v = ternary(vec![Trit::Pos, Trit::Zero, Trit::Neg]);
        assert!(emit_llvm_ir(&Node::Const(v)).is_ok());
    }

    #[test]
    fn refuses_trit_arithmetic_as_next_slice() {
        // Mutant-witness: lowering trit.add/sub/mul without the carry/overflow handling would
        // silently mis-compute; they must refuse explicitly until that slice lands.
        let t = ternary(vec![Trit::Pos, Trit::Zero]);
        for prim in ["trit.add", "trit.sub", "trit.mul"] {
            let prog = Node::Op {
                prim: prim.into(),
                args: vec![Node::Const(t.clone()), Node::Const(t.clone())],
            };
            assert!(
                matches!(emit_llvm_ir(&prog), Err(AotError::UnsupportedPrim(_))),
                "{prim} must refuse as the next slice"
            );
        }
    }

    #[test]
    fn refuses_bit_op_on_ternary_lane() {
        // Mutant-witness: dropping require_kind would let bit.not mis-lower a ternary lane.
        let prog = Node::Op {
            prim: "bit.not".into(),
            args: vec![Node::Const(ternary(vec![Trit::Pos, Trit::Neg]))],
        };
        assert!(matches!(
            emit_llvm_ir(&prog),
            Err(AotError::UnsupportedPrim(_))
        ));
    }

    #[test]
    fn refuses_swap() {
        // Mutant-witness: a swap is not straight-line bit logic; it must be refused, not ignored.
        let prog = Node::Swap {
            src: Box::new(Node::Const(binary(vec![true, false]))),
            target: Repr::Ternary { trits: 2 },
            policy: mycelium_core::ContentHash::parse("blake3:x").unwrap(),
        };
        assert!(matches!(
            emit_llvm_ir(&prog),
            Err(AotError::UnsupportedNode(_))
        ));
    }

    #[test]
    fn refuses_width_mismatch() {
        // Mutant-witness: dropping the width check would emit a ragged elementwise op.
        let prog = Node::Op {
            prim: "bit.and".into(),
            args: vec![
                Node::Const(binary(vec![true, false, true])),
                Node::Const(binary(vec![true, false])),
            ],
        };
        assert!(matches!(
            emit_llvm_ir(&prog),
            Err(AotError::WidthMismatch { .. })
        ));
    }

    // --- compiled-path smoke test (skips when llc/clang are absent) ---------------------------

    #[test]
    fn native_bit_not_matches_interpreter() {
        let prog = not_program();
        match compile_and_run(&prog) {
            Ok(v) => {
                // Mutant-witness: if bit.not lowered to `or`/`and` instead of `xor _, 1`, the
                // payload would differ from the complemented input.
                assert_eq!(v.payload(), &Payload::Bits(vec![false, true, false, false]));
                assert_eq!(v.repr(), &Repr::Binary { width: 4 });
            }
            Err(AotError::ToolchainMissing(_)) => { /* environment skip — house idiom */ }
            Err(e) => panic!("unexpected AOT error: {e}"),
        }
    }

    #[test]
    fn native_trit_neg_matches_interpreter() {
        // Mutant-witness: if trit.neg lowered to anything but `0 - x` (or the output select chain
        // mapped the wrong char), the negated payload `[-,0,+]` would differ.
        match compile_and_run(&neg_program()) {
            Ok(v) => {
                assert_eq!(
                    v.payload(),
                    &Payload::Trits(vec![Trit::Neg, Trit::Zero, Trit::Pos])
                );
                assert_eq!(v.repr(), &Repr::Ternary { trits: 3 });
            }
            Err(AotError::ToolchainMissing(_)) => { /* environment skip */ }
            Err(e) => panic!("unexpected AOT error: {e}"),
        }
    }
}
