//! Direct-LLVM-IR AOT backend for the kernel **bit subset** (M-301; RFC-0004 §2 *direct-LLVM
//! fallback*; ADR-007/009; phase-3.md §1).
//!
//! **Scope / honesty.** The ratified AOT path is `MLIR → LLVM` (RFC-0004 §2), but libMLIR is absent
//! in this environment while LLVM 18 tooling (`llc`, `clang`) is present. RFC-0004 §2 explicitly
//! anticipates *"a lighter direct-LLVM backend"* as the revisit; this module is that backend, scoped
//! to the **bit subset** (`core.id`, `bit.not/and/or/xor` over `Binary{w}`). It is a *genuinely
//! compiled native artifact* — not the textual `dialect::emit` skeleton, and not the `aot::run`
//! env-machine: [`emit_llvm_ir`] renders textual LLVM IR (one SSA op per output bit, so nothing is
//! opaque — RFC-0004 §6), and [`compile_and_run`] drives `llc` + `clang` to a native executable,
//! runs it, and reads the result back. This is the third, *compiled*, execution path; the
//! interp↔native differential (M-302) checks it against the reference interpreter (NFR-7/RR-12).
//!
//! **Deliberately out of subset (explicit refusals, never silent — G2):** the trit arithmetic
//! prims (`trit.*`, balanced-ternary carry chains), swaps, and any non-`Binary` representation. Each
//! is an explicit [`AotError`]. Trit lowering is the next slice; the MLIR dialect path stays the
//! eventual home (`dialect::emit` is its dumpable skeleton) and is deferred until libMLIR exists.

use std::collections::HashMap;
use std::fmt;
use std::fmt::Write as _; // `writeln!` into a String never fails — call sites discard the Result.
use std::path::Path;
use std::process::Command;

use mycelium_core::lower::{self, Atom, Rhs};
use mycelium_core::{Meta, Node, Payload, Provenance, Repr, Value};

/// An explicit failure of the direct-LLVM AOT path. Every non-supported construct, missing tool, or
/// subprocess failure is one of these — the path is **never silent** (G2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AotError {
    /// A representation outside the bit subset (only `Binary{w}` is supported here).
    UnsupportedRepr(String),
    /// A primitive outside the bit subset (`core.id`, `bit.not/and/or/xor`).
    UnsupportedPrim(String),
    /// A Core IR construct the bit-subset backend does not lower (e.g. a swap).
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
            AotError::UnsupportedRepr(r) => write!(f, "unsupported repr for the bit subset: {r}"),
            AotError::UnsupportedPrim(p) => write!(f, "unsupported prim for the bit subset: {p}"),
            AotError::UnsupportedNode(n) => write!(f, "unsupported node for the bit subset: {n}"),
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

/// One bit, as an LLVM `i32` operand: either a literal (`"0"`/`"1"`) or an SSA register (`"%r3"`).
type BitOperand = String;

/// SSA-name generator for the emitted IR (monotone counter → deterministic names).
struct Ssa(usize);
impl Ssa {
    fn fresh(&mut self) -> String {
        let n = self.0;
        self.0 += 1;
        format!("%r{n}")
    }
}

/// Emit textual LLVM IR for the bit-subset program `node` — a `main` that computes the result bits
/// and writes them as a line of `'0'`/`'1'` to stdout. Deterministic. One SSA op per output bit
/// (no opaque pass — RFC-0004 §6). Returns an explicit [`AotError`] for anything outside the subset.
pub fn emit_llvm_ir(node: &Node) -> Result<String, AotError> {
    let anf = lower::lower_to_anf(node);
    let mut env: HashMap<Atom, Vec<BitOperand>> = HashMap::new();
    let mut ssa = Ssa(0);
    let mut body = String::new();

    for b in anf.bindings() {
        let bits = match &b.rhs {
            Rhs::Const(v) => const_bits(v)?,
            Rhs::Alias(a) => lookup(&env, a)?.clone(),
            Rhs::Op { prim, args } => {
                let operands: Vec<&Vec<BitOperand>> = args
                    .iter()
                    .map(|a| lookup(&env, a))
                    .collect::<Result<_, _>>()?;
                emit_op(prim, &operands, &mut ssa, &mut body)?
            }
            Rhs::Swap { target, .. } => {
                return Err(AotError::UnsupportedNode(format!(
                    "swap to {target:?} (bit subset is straight-line bit logic; M-301)"
                )));
            }
        };
        env.insert(b.name.clone(), bits);
    }

    let result = lookup(&env, anf.result())?;
    let mut out = String::from("; mycelium direct-LLVM AOT (bit subset; M-301)\n");
    out.push_str("declare i32 @putchar(i32)\n\n");
    out.push_str("define i32 @main() {\nentry:\n");
    out.push_str(&body);
    // Emit the result bits as ASCII '0'/'1', then a newline.
    for bit in result {
        let c = ssa.fresh();
        let _ = writeln!(&mut out, "  {c} = add i32 {bit}, 48");
        let p = ssa.fresh();
        let _ = writeln!(&mut out, "  {p} = call i32 @putchar(i32 {c})");
    }
    let nl = ssa.fresh();
    let _ = writeln!(&mut out, "  {nl} = call i32 @putchar(i32 10)");
    out.push_str("  ret i32 0\n}\n");
    Ok(out)
}

/// The result width (number of output bits) of the bit-subset program, without emitting IR — used
/// by [`compile_and_run`] to parse the native output. Shares the same refusal surface as
/// [`emit_llvm_ir`].
fn result_width(node: &Node) -> Result<usize, AotError> {
    let anf = lower::lower_to_anf(node);
    let mut env: HashMap<Atom, usize> = HashMap::new();
    for b in anf.bindings() {
        let w = match &b.rhs {
            Rhs::Const(v) => const_bits(v)?.len(),
            Rhs::Alias(a) => *env
                .get(a)
                .ok_or_else(|| AotError::FreeVariable(a.render()))?,
            Rhs::Op { prim, args } => {
                let widths: Vec<usize> = args
                    .iter()
                    .map(|a| {
                        env.get(a)
                            .copied()
                            .ok_or_else(|| AotError::FreeVariable(a.render()))
                    })
                    .collect::<Result<_, _>>()?;
                op_width(prim, &widths)?
            }
            Rhs::Swap { target, .. } => {
                return Err(AotError::UnsupportedNode(format!("swap to {target:?}")));
            }
        };
        env.insert(b.name.clone(), w);
    }
    env.get(anf.result())
        .copied()
        .ok_or_else(|| AotError::FreeVariable(anf.result().render()))
}

fn lookup<'a, T>(env: &'a HashMap<Atom, T>, a: &Atom) -> Result<&'a T, AotError> {
    env.get(a).ok_or_else(|| AotError::FreeVariable(a.render()))
}

/// The const's bits as `i32` literal operands, or an explicit refusal for a non-`Binary` repr.
fn const_bits(v: &Value) -> Result<Vec<BitOperand>, AotError> {
    match (v.repr(), v.payload()) {
        (Repr::Binary { .. }, Payload::Bits(b)) => Ok(b
            .iter()
            .map(|&x| if x { "1" } else { "0" }.to_owned())
            .collect()),
        (repr, _) => Err(AotError::UnsupportedRepr(format!("{repr:?}"))),
    }
}

/// The output width of a bit-subset op (for parsing), with the same arity/width/prim checks as
/// emission so a refusal here matches a refusal there.
fn op_width(prim: &str, widths: &[usize]) -> Result<usize, AotError> {
    match prim {
        "core.id" | "bit.not" => exactly1(prim, widths),
        "bit.and" | "bit.or" | "bit.xor" => same2(prim, widths),
        other => Err(AotError::UnsupportedPrim(other.to_owned())),
    }
}

fn exactly1(prim: &str, widths: &[usize]) -> Result<usize, AotError> {
    match widths {
        [w] => Ok(*w),
        _ => Err(AotError::UnsupportedPrim(format!(
            "{prim} expects 1 operand, got {}",
            widths.len()
        ))),
    }
}

fn same2(prim: &str, widths: &[usize]) -> Result<usize, AotError> {
    match widths {
        [a, b] if a == b => Ok(*a),
        [a, b] => Err(AotError::WidthMismatch {
            prim: prim.to_owned(),
            a: *a,
            b: *b,
        }),
        _ => Err(AotError::UnsupportedPrim(format!(
            "{prim} expects 2 operands, got {}",
            widths.len()
        ))),
    }
}

/// Emit the elementwise LLVM IR for one bit-subset op, returning the result bit operands.
fn emit_op(
    prim: &str,
    operands: &[&Vec<BitOperand>],
    ssa: &mut Ssa,
    body: &mut String,
) -> Result<Vec<BitOperand>, AotError> {
    match prim {
        "core.id" => {
            let [a] = arity1(prim, operands)?;
            Ok(a.clone())
        }
        "bit.not" => {
            let [a] = arity1(prim, operands)?;
            Ok(a.iter()
                .map(|x| {
                    let r = ssa.fresh();
                    let _ = writeln!(body, "  {r} = xor i32 {x}, 1");
                    r
                })
                .collect())
        }
        "bit.and" | "bit.or" | "bit.xor" => {
            let (a, b) = arity2(prim, operands)?;
            if a.len() != b.len() {
                return Err(AotError::WidthMismatch {
                    prim: prim.to_owned(),
                    a: a.len(),
                    b: b.len(),
                });
            }
            let instr = match prim {
                "bit.and" => "and",
                "bit.or" => "or",
                _ => "xor",
            };
            Ok(a.iter()
                .zip(b)
                .map(|(x, y)| {
                    let r = ssa.fresh();
                    let _ = writeln!(body, "  {r} = {instr} i32 {x}, {y}");
                    r
                })
                .collect())
        }
        other => Err(AotError::UnsupportedPrim(other.to_owned())),
    }
}

fn arity1<'a>(
    prim: &str,
    ops: &[&'a Vec<BitOperand>],
) -> Result<[&'a Vec<BitOperand>; 1], AotError> {
    match ops {
        [a] => Ok([a]),
        _ => Err(AotError::UnsupportedPrim(format!(
            "{prim} expects 1 operand, got {}",
            ops.len()
        ))),
    }
}

fn arity2<'a>(
    prim: &str,
    ops: &[&'a Vec<BitOperand>],
) -> Result<(&'a Vec<BitOperand>, &'a Vec<BitOperand>), AotError> {
    match ops {
        [a, b] => Ok((a, b)),
        _ => Err(AotError::UnsupportedPrim(format!(
            "{prim} expects 2 operands, got {}",
            ops.len()
        ))),
    }
}

/// Compile the bit-subset program to a native executable (via `llc` + `clang`), run it, and read the
/// result back as an `Exact` `Binary{w}` [`Value`] (bit ops are exact; the subset refuses approximate
/// inputs). Returns [`AotError::ToolchainMissing`] when `llc`/`clang` are absent so callers can skip.
///
/// This is the **compiled** execution path the M-302 differential checks against the interpreter.
pub fn compile_and_run(node: &Node) -> Result<Value, AotError> {
    let ir = emit_llvm_ir(node)?;
    let width = result_width(node)?;
    ensure_toolchain()?;

    let dir = unique_tmp_dir()?;
    let ll = dir.join("kernel.ll");
    let obj = dir.join("kernel.o");
    let bin = dir.join("kernel");
    let _guard = TmpDir(dir.clone());

    std::fs::write(&ll, ir.as_bytes()).map_err(|e| AotError::Run(format!("write IR: {e}")))?;

    run_tool("llc", &["-filetype=obj", path(&ll)?, "-o", path(&obj)?])?;
    run_tool("clang", &[path(&obj)?, "-o", path(&bin)?])?;

    let output = Command::new(&bin)
        .output()
        .map_err(|e| AotError::Run(format!("exec {}: {e}", bin.display())))?;
    if !output.status.success() {
        return Err(AotError::Run(format!("artifact exited {}", output.status)));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| AotError::Parse(format!("non-utf8 output: {e}")))?;
    let line = stdout.lines().next().unwrap_or("");
    if line.len() != width {
        return Err(AotError::Parse(format!(
            "expected {width} bits, got {} ({line:?})",
            line.len()
        )));
    }
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
            width: width as u32,
        },
        Payload::Bits(bits),
        Meta::exact(Provenance::Root),
    )
    .map_err(|e| AotError::Wf(e.to_string()))
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

    fn not_program() -> Node {
        Node::Op {
            prim: "bit.not".into(),
            args: vec![Node::Const(binary(vec![true, false, true, true]))],
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
    fn refuses_ternary_const() {
        // Mutant-witness: if const_bits silently treated a non-Binary repr as bits, this would
        // emit garbage IR instead of refusing.
        use mycelium_core::Trit;
        let v = Value::new(
            Repr::Ternary { trits: 3 },
            Payload::Trits(vec![Trit::Pos, Trit::Zero, Trit::Neg]),
            Meta::exact(Provenance::Root),
        )
        .unwrap();
        assert!(matches!(
            emit_llvm_ir(&Node::Const(v)),
            Err(AotError::UnsupportedRepr(_))
        ));
    }

    #[test]
    fn refuses_trit_op() {
        // Mutant-witness: dropping the prim allowlist would let trit.add through and mis-lower it.
        use mycelium_core::Trit;
        let t = Value::new(
            Repr::Ternary { trits: 2 },
            Payload::Trits(vec![Trit::Pos, Trit::Zero]),
            Meta::exact(Provenance::Root),
        )
        .unwrap();
        let prog = Node::Op {
            prim: "trit.add".into(),
            args: vec![Node::Const(t.clone()), Node::Const(t)],
        };
        assert!(matches!(
            emit_llvm_ir(&prog),
            Err(AotError::UnsupportedRepr(_)) | Err(AotError::UnsupportedPrim(_))
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
}
