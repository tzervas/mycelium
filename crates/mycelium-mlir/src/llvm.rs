//! Direct-LLVM-IR AOT backend for the kernel **bit/trit subset + non-recursive data fragment**
//! (M-301; M-373; RFC-0004 §2 *direct-LLVM fallback* / §11.2 *Increment-1 sanction*;
//! ADR-007/009; DN-15 §4.1; phase-3.md §1/§9.1).
//!
//! **Scope / honesty.** The ratified AOT path is `MLIR → LLVM` (RFC-0004 §2), but libMLIR is absent
//! in this environment while LLVM 18 tooling (`llc`, `clang`) is present. RFC-0004 §2 explicitly
//! anticipates *"a lighter direct-LLVM backend"* as the revisit; this module is that backend, scoped
//! to a **bit/trit + non-recursive data sub-fragment**: `core.id`, `bit.not/and/or/xor` over
//! `Binary{w}`, `trit.neg/add/sub/mul` over `Ternary{m}`, and (Increment-1) `Construct`/`Match` over
//! tagged stack structs. It is a *genuinely compiled native artifact* — not the textual `dialect::emit`
//! skeleton, and not the `aot::run` env-machine: [`emit_llvm_ir`] renders textual LLVM IR (one op
//! per output element, so nothing is opaque — RFC-0004 §6), and [`compile_and_run`] drives `llc` +
//! `clang` to a native executable, runs it, and reads the result back. This is the third,
//! *compiled*, execution path; the interp↔native differential (M-302) checks it against the
//! reference interpreter (NFR-7/RR-12).
//!
//! **Trit carry arithmetic (M-301 trit slice).** `trit.add/sub/mul` over `Ternary{m}` are lowered as
//! **ripple-carry** / **shifted-accumulate** IR that mirrors `mycelium_core::ternary` digit-for-digit
//! (`s + 4`, then `srem 3 − 1` for the balanced digit and `sdiv 3 − 1` for the carry — euclidean by
//! construction because `s + 4 ≥ 1`). Fixed-width overflow (a non-zero final carry, or non-zero high
//! trits of a product) is **detected at runtime** and signalled through the **read-back protocol**:
//! an out-of-range result prints the [`OVERFLOW_SENTINEL`] line (AOT) / returns a non-zero status
//! (JIT) and surfaces as an explicit [`AotError::Overflow`] — never a silent wrap (SC-3; G2). This
//! matches the interpreter's `EvalError::Overflow` so the M-302 differential stays honest.
//!
//! **Non-recursive data sub-fragment (Increment-1 — M-373; DN-15 §4.1; RFC-0004 §11.2).**
//! `Construct` and `Match` are now natively compiled for the **non-recursive, bounded** case (no
//! `Fix`/`FixGroup` in scope, so all allocations are statically bounded at codegen time). The
//! representation uses **stack `alloca`** (not `@malloc`) — a deliberate choice grounded in the
//! non-recursive/bounded restriction: because no heap recursion can produce unbounded allocation
//! depth, the alloca frame size is fixed at compile time, and an explicit OOM failure path is
//! unnecessary. Each constructed value is an `[N+1 x i64]` alloca (slot 0 = tag i64; slots 1..N =
//! field elements, one i64 per element laid out consecutively across all fields). `Match` emits an
//! LLVM `switch i64` on the tag with an explicit defined-trap default (never raw `unreachable` UB;
//! G2). Guarantee tag: **Declared** (hand-written textual-IR lowering; the differential against the
//! interpreter is empirical evidence, not a proof — VR-5).
//!
//! **Deliberately out of subset (explicit refusals, never silent — G2):** `App`, `Lam`, `Fix`,
//! `FixGroup` (closures + recursion need closure-conversion + heap, deferred to Increment-2/3),
//! `Swap` (swap to non-binary/ternary repr), Dense/VSA representations. Each is an explicit
//! [`AotError`]. The MLIR dialect path stays the eventual home (`dialect::emit` is its dumpable
//! skeleton), deferred until libMLIR exists.

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
    /// A balanced-ternary arithmetic result left the fixed `m`-trit range — the native path computed
    /// the overflow at runtime and signalled it through the read-back protocol (matches the
    /// interpreter's `EvalError::Overflow`; never a silent wrap, SC-3/G2).
    Overflow(String),
    /// A [`PackScheme`](mycelium_core::PackScheme) with no BitNet compute kernel (only the three
    /// bitnet packings I2_S/TL1/TL2 have one). An explicit refusal — never a silent misdecode.
    UnsupportedScheme(String),
}

/// The single byte the native artifact prints (AOT) when a fixed-width trit-arithmetic result
/// overflows the `m`-trit range. Chosen because it is **not** a valid element char (`'0'`/`'1'` for
/// bits, `'-'`/`'0'`/`'+'` for trits), so it can never be confused with a result line.
pub(crate) const OVERFLOW_SENTINEL: u8 = b'!';

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
            AotError::Overflow(e) => write!(f, "balanced-ternary overflow: {e}"),
            AotError::UnsupportedScheme(s) => write!(f, "no BitNet kernel for packing scheme: {s}"),
        }
    }
}

impl std::error::Error for AotError {}

/// One element (a bit or a trit), as an LLVM `i32` operand: a literal (`"0"`/`"1"`/`"-1"`) or an
/// SSA register (`"%r3"`).
type Operand = String;

/// Which representation a lane carries — fixes how its elements are computed and printed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LaneKind {
    /// `Binary{w}` — elements in `{0, 1}`, printed `'0'`/`'1'`.
    Binary,
    /// `Ternary{m}` — balanced-ternary elements in `{-1, 0, 1}`, printed `'-'`/`'0'`/`'+'`.
    Ternary,
}

/// A computed value lane: its representation kind and one `i32` operand per element.
#[derive(Debug, Clone)]
pub(crate) struct Lane {
    pub(crate) kind: LaneKind,
    pub(crate) vals: Vec<Operand>,
}

/// The layout of one field inside a [`Datum`] struct: kind + number of elements. Elements are stored
/// consecutively in the struct's i64 slots starting at `slot_start` (each element occupies one i64).
#[derive(Debug, Clone)]
pub(crate) struct FieldLayout {
    /// Binary or Ternary — determines how elements are interpreted.
    pub(crate) kind: LaneKind,
    /// Number of elements (the `w` of `Binary{w}` or the `m` of `Ternary{m}`).
    pub(crate) elems: usize,
    /// The i64 slot index (1-based; slot 0 is always the tag) of the first element of this field.
    pub(crate) slot_start: usize,
}

/// A constructed data value in the lowered env: a pointer to a stack-`alloca`'d struct (the tag in
/// slot 0, field elements in consecutive i64 slots after it) plus the field layout so that a
/// downstream `Match` can extract the fields without knowing the field types again.
///
/// Representation choice (DN-15 §4.1 / RFC-0004 §11.2): **stack `alloca`** is used instead of
/// `@malloc` because the non-recursive/bounded restriction (no `Fix`/`FixGroup`) means all
/// allocation depth is fixed at codegen time — there is no need for heap allocation or an explicit
/// OOM failure path. `alloca` is simpler, inspectable, and directly auditable in the emitted IR.
#[derive(Debug, Clone)]
pub(crate) struct Datum {
    /// The SSA register holding the `[N+1 x i64]*` alloca pointer.
    pub(crate) ptr: String,
    /// The constructor tag (an i64 discriminant, equal to the `CtorRef::index()`).
    /// Retained for auditability / future diagnostics; not read back during Match lowering because
    /// the tag is re-loaded from the alloca at runtime.
    pub(crate) _tag: u64,
    /// Layout of each field, in declaration order.
    pub(crate) fields: Vec<FieldLayout>,
    /// Total number of i64 slots (1 tag + sum of all field elem counts).
    pub(crate) slots: usize,
}

/// An environment value — either a repr-lane (bit/trit) or a constructed data value (tagged struct).
///
/// The `lower_program` env maps [`Atom`] → `EnvValue`. Repr-lane values flow into `emit_op`; datum
/// values are produced by `Construct` and consumed by `Match` arm bodies. A datum is never a final
/// result (the output protocol prints bits/trits; a program that leaves a datum on the result atom
/// is refused with an explicit [`AotError::UnsupportedNode`]).
#[derive(Debug, Clone)]
pub(crate) enum EnvValue {
    Repr(Lane),
    Datum(Datum),
}

impl EnvValue {
    /// Extract the repr lane, or return an explicit error if it is a datum.
    fn into_lane(self, ctx: &str) -> Result<Lane, AotError> {
        match self {
            EnvValue::Repr(l) => Ok(l),
            EnvValue::Datum(_) => Err(AotError::UnsupportedNode(format!(
                "{ctx}: expected a repr lane but found a data value (only repr \
                 values are valid here)"
            ))),
        }
    }
    fn as_lane(&self, ctx: &str) -> Result<&Lane, AotError> {
        match self {
            EnvValue::Repr(l) => Ok(l),
            EnvValue::Datum(_) => Err(AotError::UnsupportedNode(format!(
                "{ctx}: expected a repr lane but found a data value"
            ))),
        }
    }
    fn as_datum(&self, ctx: &str) -> Result<&Datum, AotError> {
        match self {
            EnvValue::Datum(d) => Ok(d),
            EnvValue::Repr(_) => Err(AotError::UnsupportedNode(format!(
                "{ctx}: expected a data value (Datum) but found a repr lane"
            ))),
        }
    }
}

/// SSA-name generator for the emitted IR (monotone counter → deterministic names).
pub(crate) struct Ssa(pub(crate) usize);
impl Ssa {
    pub(crate) fn fresh(&mut self) -> String {
        let n = self.0;
        self.0 += 1;
        format!("%r{n}")
    }
}

/// Basic-block label counter — gives every emitted control-flow label a unique name (monotone,
/// deterministic). Separate from the SSA counter so block names and register names never collide.
pub(crate) struct Bbc(pub(crate) usize);
impl Bbc {
    pub(crate) fn fresh(&mut self) -> String {
        let n = self.0;
        self.0 += 1;
        format!("bb{n}")
    }
}

/// The lowered program: the emitted op `body`, the `result` lane, and the SSA counter to continue
/// from. The **single source of truth** for [`emit_llvm_ir`], [`result_shape`], and the JIT
/// function emitter — so the shape used to parse the output can never disagree with what was emitted.
pub(crate) struct Lowered {
    pub(crate) body: String,
    pub(crate) result: Lane,
    pub(crate) ssa: Ssa,
    /// The combined runtime overflow flag — an `i1` SSA register that is the OR of every
    /// trit-arithmetic op's overflow condition, or `None` for a program that cannot overflow (no
    /// `trit.add/sub/mul`). The AOT/JIT emitters branch on it to drive the read-back protocol.
    pub(crate) overflow: Option<String>,
}

/// Lower a single field `Lane` into the struct at `ptr`, writing elements starting at `slot_start`
/// (each element occupies one i64 slot). Returns the `FieldLayout` for this field.
fn emit_store_field(
    lane: &Lane,
    ptr: &str,
    slots: usize,
    slot_start: usize,
    ssa: &mut Ssa,
    body: &mut String,
) -> FieldLayout {
    for (i, v) in lane.vals.iter().enumerate() {
        // Sign-extend / zero-extend the i32 element to i64 before storing.
        let ext = ssa.fresh();
        let _ = writeln!(body, "  {ext} = sext i32 {v} to i64");
        let gep = ssa.fresh();
        let slot = slot_start + i;
        let _ = writeln!(
            body,
            "  {gep} = getelementptr inbounds [{slots} x i64], [{slots} x i64]* {ptr}, i64 0, i64 {slot}"
        );
        let _ = writeln!(body, "  store i64 {ext}, i64* {gep}");
    }
    FieldLayout {
        kind: lane.kind,
        elems: lane.vals.len(),
        slot_start,
    }
}

/// Load one field from a struct at `ptr` given its `FieldLayout`, returning a `Lane` of i32
/// register operands (each element truncated from i64). The struct has `slots` total i64 slots.
fn emit_load_field(
    layout: &FieldLayout,
    ptr: &str,
    slots: usize,
    ssa: &mut Ssa,
    body: &mut String,
) -> Lane {
    let vals: Vec<Operand> = (0..layout.elems)
        .map(|i| {
            let slot = layout.slot_start + i;
            let gep = ssa.fresh();
            let _ = writeln!(
                body,
                "  {gep} = getelementptr inbounds [{slots} x i64], [{slots} x i64]* {ptr}, i64 0, i64 {slot}"
            );
            let loaded = ssa.fresh();
            let _ = writeln!(body, "  {loaded} = load i64, i64* {gep}");
            let trunc = ssa.fresh();
            let _ = writeln!(body, "  {trunc} = trunc i64 {loaded} to i32");
            trunc
        })
        .collect();
    Lane {
        kind: layout.kind,
        vals,
    }
}

/// Emit the `i32` ASCII char code for one result element of `kind` (operand `v`), returning the SSA
/// register holding it. Binary → `val + 48` (`'0'`/`'1'`); Ternary → `'-'`(45)/`'0'`(48)/`'+'`(43)
/// via a branch-free `select` chain. **Shared** by the AOT (`putchar`) and JIT (`store`) emitters so
/// their element encodings — and thus the read-back — can never diverge.
pub(crate) fn emit_char_code(kind: LaneKind, v: &str, ssa: &mut Ssa, body: &mut String) -> String {
    match kind {
        LaneKind::Binary => {
            let c = ssa.fresh();
            let _ = writeln!(body, "  {c} = add i32 {v}, 48");
            c
        }
        LaneKind::Ternary => {
            let isneg = ssa.fresh();
            let _ = writeln!(body, "  {isneg} = icmp eq i32 {v}, -1");
            let ispos = ssa.fresh();
            let _ = writeln!(body, "  {ispos} = icmp eq i32 {v}, 1");
            let t = ssa.fresh();
            let _ = writeln!(body, "  {t} = select i1 {ispos}, i32 43, i32 48");
            let c = ssa.fresh();
            let _ = writeln!(body, "  {c} = select i1 {isneg}, i32 45, i32 {t}");
            c
        }
    }
}

/// Decode `width` printed element chars (Binary: `'0'`/`'1'`; Ternary: `'-'`/`'0'`/`'+'`) into an
/// `Exact` `Value`. **Shared** by the AOT stdout read-back and the JIT buffer read-back.
pub(crate) fn decode_result(
    kind: LaneKind,
    width: usize,
    chars: impl Iterator<Item = char>,
) -> Result<Value, AotError> {
    let chars: Vec<char> = chars.collect();
    if chars.len() != width {
        return Err(AotError::Parse(format!(
            "expected {width} elements, got {} ({chars:?})",
            chars.len()
        )));
    }
    match kind {
        LaneKind::Binary => {
            let bits: Vec<bool> = chars
                .into_iter()
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
        LaneKind::Ternary => {
            let trits: Vec<Trit> = chars
                .into_iter()
                .map(|c| match c {
                    '-' => Ok(Trit::Neg),
                    '0' => Ok(Trit::Zero),
                    '+' => Ok(Trit::Pos),
                    other => Err(AotError::Parse(format!("non-trit char {other:?}"))),
                })
                .collect::<Result<_, _>>()?;
            Value::new(
                Repr::Ternary {
                    trits: width as u32,
                },
                Payload::Trits(trits),
                Meta::exact(Provenance::Root),
            )
            .map_err(|e| AotError::Wf(e.to_string()))
        }
    }
}

/// Walk the lowered ANF, emitting one op per binding, and return the result lane. Returns an
/// explicit [`AotError`] for anything outside the bit/trit + non-recursive-data subset (M-301;
/// M-373). The env maps each bound atom to an [`EnvValue`] (either a repr lane or a datum struct).
pub(crate) fn lower_program(node: &Node) -> Result<Lowered, AotError> {
    let anf = lower::lower_to_anf(node);
    let mut env: HashMap<Atom, EnvValue> = HashMap::new();
    let mut ssa = Ssa(0);
    let mut bbc = Bbc(0);
    let mut body = String::new();
    // The per-op overflow `i1` registers, accumulated across the program. Any trit-arithmetic op
    // pushes its overflow condition here; the interpreter errors on the *first* overflow, so the
    // native path being conservative (OR of all of them ⇒ one explicit `Overflow`) gives the same
    // verdict — we never read the meaningless result either way.
    let mut flags: Vec<String> = Vec::new();

    for b in anf.bindings() {
        let ev = match &b.rhs {
            Rhs::Const(v) => EnvValue::Repr(const_lane(v)?),
            Rhs::Alias(a) => lookup_ev(&env, a)?.clone(),
            Rhs::Op { prim, args } => {
                let operands: Vec<&Lane> = args
                    .iter()
                    .map(|a| lookup_ev(&env, a)?.as_lane("op operand"))
                    .collect::<Result<_, _>>()?;
                EnvValue::Repr(emit_op(prim, &operands, &mut ssa, &mut body, &mut flags)?)
            }
            Rhs::Swap { target, .. } => {
                return Err(AotError::UnsupportedNode(format!(
                    "swap to {target:?} (the subset is straight-line bit/trit ops; M-301)"
                )));
            }
            // Increment-1 (M-373; DN-15 §4.1; RFC-0004 §11.2): Construct and Match are lowered for
            // the NON-RECURSIVE, BOUNDED case. Stack alloca is used (not malloc) because the
            // non-recursive/bounded restriction (no Fix/FixGroup in scope) makes all allocation depth
            // statically known at codegen time — no OOM path needed (G2 is satisfied by the explicit
            // UnsupportedNode refusal for Fix/FixGroup below). Guarantee: Declared (VR-5).
            Rhs::Construct { ctor, args } => {
                // Each field is a Lane; we store each element as one i64 slot after the tag.
                // Layout: [tag(i64), field_0_elem_0(i64), ..., field_0_elem_w-1, field_1_elem_0, ...]
                let field_lanes: Vec<Lane> = args
                    .iter()
                    .map(|a| lookup_ev(&env, a)?.as_lane("Construct field").cloned())
                    .collect::<Result<_, _>>()?;
                let total_elem: usize = field_lanes.iter().map(|l| l.vals.len()).sum();
                let slots = 1 + total_elem; // tag slot + one slot per element across all fields
                                            // Allocate the struct on the stack.
                let ptr = ssa.fresh();
                let _ = writeln!(body, "  {ptr} = alloca [{slots} x i64], align 8");
                // Store the tag (ctor.index() as i64) in slot 0.
                let tag_gep = ssa.fresh();
                let tag_val = ctor.index() as u64;
                let _ = writeln!(
                    body,
                    "  {tag_gep} = getelementptr inbounds [{slots} x i64], [{slots} x i64]* {ptr}, i64 0, i64 0"
                );
                let _ = writeln!(body, "  store i64 {tag_val}, i64* {tag_gep}");
                // Store each field's elements consecutively after the tag.
                let mut slot_start = 1usize;
                let mut field_layouts: Vec<FieldLayout> = Vec::with_capacity(field_lanes.len());
                for lane in &field_lanes {
                    let layout =
                        emit_store_field(lane, &ptr, slots, slot_start, &mut ssa, &mut body);
                    slot_start += lane.vals.len();
                    field_layouts.push(layout);
                }
                EnvValue::Datum(Datum {
                    ptr,
                    _tag: tag_val,
                    fields: field_layouts,
                    slots,
                })
            }
            Rhs::Match {
                scrutinee,
                alts,
                default: default_arm,
                // The ANF `default` arm (if `Some`) is lowered into the switch's default block so
                // that the native path returns the same value as the interpreter when no explicit
                // arm matches. If `None`, the default emits abort() — a defined-trap for the
                // provably-unreachable no-match case (WF7 checker proves coverage; G2: never UB).
            } => {
                // Load the tag from the scrutinee datum, then switch on it. Each arm loads its
                // binder fields from the struct and inlines the arm's ANF body (recursively).
                // The match must terminate at a repr Lane value (not a Datum) — the final result
                // must be printable by the read-back protocol. The switch has an explicit defined
                // default — either the lowered ANF default block (if `Some`) or a call to abort()
                // (if `None`) — never raw `unreachable` UB (G2).
                let datum = lookup_ev(&env, scrutinee)?
                    .as_datum("Match scrutinee")
                    .cloned()?;

                // Load the tag.
                let tag_gep = ssa.fresh();
                let slots = datum.slots;
                let ptr = &datum.ptr.clone();
                let _ = writeln!(
                    body,
                    "  {tag_gep} = getelementptr inbounds [{slots} x i64], [{slots} x i64]* {ptr}, i64 0, i64 0"
                );
                let tag_reg = ssa.fresh();
                let _ = writeln!(body, "  {tag_reg} = load i64, i64* {tag_gep}");

                // Generate unique labels for each arm and the merge block.
                let arm_labels: Vec<String> = (0..alts.len()).map(|_| bbc.fresh()).collect();
                let default_label = bbc.fresh();
                let merge_label = bbc.fresh();

                // Emit the switch instruction.
                let _ = write!(body, "  switch i64 {tag_reg}, label %{default_label} [");
                for (alt, label) in alts.iter().zip(&arm_labels) {
                    use mycelium_core::lower::AnfAlt;
                    let arm_tag = match alt {
                        AnfAlt::Ctor { ctor, .. } => ctor.index() as u64,
                        AnfAlt::Lit { .. } => {
                            return Err(AotError::UnsupportedNode(
                                "literal Match arms are not supported in the native LLVM data \
                                 fragment (Increment-1); use constructor arms only"
                                    .to_owned(),
                            ));
                        }
                    };
                    let _ = write!(body, " i64 {arm_tag}, label %{label}");
                }
                let _ = writeln!(body, " ]");

                // Collect (arm_label, result_lane) pairs for the phi at the merge.
                let mut phi_entries: Vec<(String, Lane)> = Vec::with_capacity(alts.len());

                for (alt, label) in alts.iter().zip(&arm_labels) {
                    use mycelium_core::lower::AnfAlt;
                    let _ = writeln!(body, "{label}:");
                    let AnfAlt::Ctor {
                        binders,
                        body: arm_body,
                        ..
                    } = alt
                    else {
                        unreachable!("literal arms filtered above")
                    };
                    // Issue 2 (never-silent / G2): check binder/field arity before zipping.
                    // The interpreter rejects arity mismatches with DataMalformed; we must too.
                    if binders.len() != datum.fields.len() {
                        return Err(AotError::UnsupportedNode(format!(
                            "Match arm binder arity ({}) != constructor field count ({}) \
                             — malformed Match (interpreter rejects with DataMalformed; G2/WF7)",
                            binders.len(),
                            datum.fields.len()
                        )));
                    }
                    // Bind field lanes into a child env extended from the current one.
                    let mut arm_env = env.clone();
                    for (binder, field_layout) in binders.iter().zip(&datum.fields) {
                        let field_lane =
                            emit_load_field(field_layout, ptr, slots, &mut ssa, &mut body);
                        arm_env.insert(
                            mycelium_core::lower::Atom::Named(binder.clone()),
                            EnvValue::Repr(field_lane),
                        );
                    }
                    // Lower the arm body (a nested ANF block) recursively into the current IR body.
                    let arm_result = lower_anf_block(
                        arm_body,
                        &mut arm_env,
                        &mut ssa,
                        &mut bbc,
                        &mut body,
                        &mut flags,
                    )?;
                    phi_entries.push((label.clone(), arm_result));
                    let _ = writeln!(body, "  br label %{merge_label}");
                }

                // Default block: if the ANF `default` arm is `Some`, lower it and merge its
                // result via the phi (matching the interpreter: the default body's value is
                // returned when no explicit arm matches). If `None`, emit abort() — a
                // defined-trap (WF7 proves exhaustive coverage; abort is the honest never-silent
                // trap; G2: never raw unreachable UB).
                let _ = writeln!(body, "{default_label}:");
                if let Some(default_block) = default_arm {
                    // The default arm has no binders; use the current env directly.
                    let default_result = lower_anf_block(
                        default_block,
                        &mut env.clone(),
                        &mut ssa,
                        &mut bbc,
                        &mut body,
                        &mut flags,
                    )?;
                    phi_entries.push((default_label.clone(), default_result));
                    let _ = writeln!(body, "  br label %{merge_label}");
                } else {
                    // No ANF default: WF7 guarantees the switch is exhaustive; abort() is the
                    // honest defined-trap (never raw `unreachable` UB — G2).
                    let _ = writeln!(body, "  call void @abort()");
                    let _ = writeln!(body, "  ret i32 0");
                }

                // Merge block: collect results from arms via phi.
                let _ = writeln!(body, "{merge_label}:");
                if phi_entries.is_empty() {
                    return Err(AotError::UnsupportedNode(
                        "Match with zero arms (exhaustive coverage requires at least one arm)"
                            .to_owned(),
                    ));
                }
                // All arms must yield the same kind/width Lane — check and emit phi per element.
                let first = &phi_entries[0].1;
                let kind = first.kind;
                let width = first.vals.len();
                for (_, lane) in &phi_entries[1..] {
                    if lane.kind != kind || lane.vals.len() != width {
                        return Err(AotError::UnsupportedNode(
                            "Match arms produce lanes of different kind or width — the native \
                             data fragment requires all arms to return the same repr shape"
                                .to_owned(),
                        ));
                    }
                }
                let mut result_vals: Vec<Operand> = Vec::with_capacity(width);
                for elem_idx in 0..width {
                    let phi_reg = ssa.fresh();
                    let phi_operands: Vec<String> = phi_entries
                        .iter()
                        .map(|(lbl, lane)| format!("[ {}, %{lbl} ]", lane.vals[elem_idx]))
                        .collect();
                    let _ = writeln!(body, "  {phi_reg} = phi i32 {}", phi_operands.join(", "));
                    result_vals.push(phi_reg);
                }
                EnvValue::Repr(Lane {
                    kind,
                    vals: result_vals,
                })
            }
            // App, Lam, Fix, FixGroup: closures + recursion — deferred to Increment-2/3. These run
            // on the AOT env-machine (M-342); the native LLVM path refuses them explicitly (VR-5; G2).
            // The refusal message is updated to no longer mention Construct/Match (which are now
            // supported for the non-recursive bounded case).
            Rhs::App { .. } => {
                return Err(AotError::UnsupportedNode(
                    "App: function application needs closure codegen (Increment-2, deferred). \
                     Runs on the AOT env-machine (M-342)"
                        .to_owned(),
                ));
            }
            Rhs::Lam { .. } => {
                return Err(AotError::UnsupportedNode(
                    "Lam: lambda abstraction needs closure-conversion (Increment-2, deferred). \
                     Runs on the AOT env-machine (M-342)"
                        .to_owned(),
                ));
            }
            Rhs::Fix { .. } => {
                return Err(AotError::UnsupportedNode(
                    "Fix: general recursion needs a heap trampoline (Increment-3 / DN-05 #1, \
                     deferred). Runs on the AOT env-machine (M-342)"
                        .to_owned(),
                ));
            }
            Rhs::FixGroup { .. } => {
                return Err(AotError::UnsupportedNode(
                    "FixGroup: mutual recursion needs a heap trampoline (Increment-3 / DN-05 #1, \
                     deferred). Runs on the AOT env-machine (M-342)"
                        .to_owned(),
                ));
            }
        };
        env.insert(b.name.clone(), ev);
    }

    let result_ev = lookup_ev(&env, anf.result())?.clone();
    let result = result_ev.into_lane("final program result")?;
    // Fold the per-op overflow flags into one `i1` (left-associative `or` chain), or `None`.
    let overflow = fold_or(&flags, &mut ssa, &mut body);
    Ok(Lowered {
        body,
        result,
        ssa,
        overflow,
    })
}

/// Lower a nested ANF block (a `Match` arm or similar nested scope) into the ongoing IR stream,
/// extending `env` with any new bindings. Returns the result `Lane` of the nested block.
/// This is the recursive workhorse for `Rhs::Match` arm bodies in [`lower_program`].
fn lower_anf_block(
    anf: &lower::Anf,
    env: &mut HashMap<Atom, EnvValue>,
    ssa: &mut Ssa,
    bbc: &mut Bbc,
    body: &mut String,
    flags: &mut Vec<String>,
) -> Result<Lane, AotError> {
    for b in anf.bindings() {
        let ev = match &b.rhs {
            Rhs::Const(v) => EnvValue::Repr(const_lane(v)?),
            Rhs::Alias(a) => lookup_ev(env, a)?.clone(),
            Rhs::Op { prim, args } => {
                let operands: Vec<&Lane> = args
                    .iter()
                    .map(|a| lookup_ev(env, a)?.as_lane("op operand"))
                    .collect::<Result<_, _>>()?;
                EnvValue::Repr(emit_op(prim, &operands, ssa, body, flags)?)
            }
            Rhs::Swap { target, .. } => {
                return Err(AotError::UnsupportedNode(format!(
                    "swap to {target:?} in a match arm (M-301)"
                )));
            }
            Rhs::Construct { ctor, args } => {
                let field_lanes: Vec<Lane> = args
                    .iter()
                    .map(|a| lookup_ev(env, a)?.as_lane("Construct field").cloned())
                    .collect::<Result<_, _>>()?;
                let total_elem: usize = field_lanes.iter().map(|l| l.vals.len()).sum();
                let slots = 1 + total_elem;
                let ptr = ssa.fresh();
                let _ = writeln!(body, "  {ptr} = alloca [{slots} x i64], align 8");
                let tag_gep = ssa.fresh();
                let tag_val = ctor.index() as u64;
                let _ = writeln!(
                    body,
                    "  {tag_gep} = getelementptr inbounds [{slots} x i64], [{slots} x i64]* {ptr}, i64 0, i64 0"
                );
                let _ = writeln!(body, "  store i64 {tag_val}, i64* {tag_gep}");
                let mut slot_start = 1usize;
                let mut field_layouts: Vec<FieldLayout> = Vec::with_capacity(field_lanes.len());
                for lane in &field_lanes {
                    let layout = emit_store_field(lane, &ptr, slots, slot_start, ssa, body);
                    slot_start += lane.vals.len();
                    field_layouts.push(layout);
                }
                EnvValue::Datum(Datum {
                    ptr,
                    _tag: tag_val,
                    fields: field_layouts,
                    slots,
                })
            }
            Rhs::Match {
                scrutinee,
                alts,
                default: default_arm,
                // Nested match — identical semantics to the top-level Match. The ANF `default`
                // arm (if `Some`) is lowered and merged via phi; if `None`, abort() is the
                // defined-trap (G2: never raw unreachable UB; WF7 proves exhaustive coverage).
            } => {
                // Nested match inside an arm body — identical logic to the top-level match.
                let datum = lookup_ev(env, scrutinee)?
                    .as_datum("Match scrutinee")
                    .cloned()?;
                let slots = datum.slots;
                let ptr = datum.ptr.clone();
                let tag_gep = ssa.fresh();
                let _ = writeln!(
                    body,
                    "  {tag_gep} = getelementptr inbounds [{slots} x i64], [{slots} x i64]* {ptr}, i64 0, i64 0"
                );
                let tag_reg = ssa.fresh();
                let _ = writeln!(body, "  {tag_reg} = load i64, i64* {tag_gep}");
                let arm_labels: Vec<String> = (0..alts.len()).map(|_| bbc.fresh()).collect();
                let default_label = bbc.fresh();
                let merge_label = bbc.fresh();
                let _ = write!(body, "  switch i64 {tag_reg}, label %{default_label} [");
                for (alt, label) in alts.iter().zip(&arm_labels) {
                    use mycelium_core::lower::AnfAlt;
                    let arm_tag = match alt {
                        AnfAlt::Ctor { ctor, .. } => ctor.index() as u64,
                        AnfAlt::Lit { .. } => {
                            return Err(AotError::UnsupportedNode(
                                "literal Match arms are not supported in the native LLVM data fragment"
                                    .to_owned(),
                            ));
                        }
                    };
                    let _ = write!(body, " i64 {arm_tag}, label %{label}");
                }
                let _ = writeln!(body, " ]");
                let mut phi_entries: Vec<(String, Lane)> = Vec::with_capacity(alts.len());
                for (alt, label) in alts.iter().zip(&arm_labels) {
                    use mycelium_core::lower::AnfAlt;
                    let _ = writeln!(body, "{label}:");
                    let AnfAlt::Ctor {
                        binders,
                        body: arm_body,
                        ..
                    } = alt
                    else {
                        unreachable!()
                    };
                    // Issue 2 (never-silent / G2): check binder/field arity before zipping.
                    if binders.len() != datum.fields.len() {
                        return Err(AotError::UnsupportedNode(format!(
                            "Match arm binder arity ({}) != constructor field count ({}) \
                             — malformed Match (interpreter rejects with DataMalformed; G2/WF7)",
                            binders.len(),
                            datum.fields.len()
                        )));
                    }
                    let mut arm_env = env.clone();
                    for (binder, field_layout) in binders.iter().zip(&datum.fields) {
                        let field_lane = emit_load_field(field_layout, &ptr, slots, ssa, body);
                        arm_env.insert(
                            mycelium_core::lower::Atom::Named(binder.clone()),
                            EnvValue::Repr(field_lane),
                        );
                    }
                    let arm_result =
                        lower_anf_block(arm_body, &mut arm_env, ssa, bbc, body, flags)?;
                    phi_entries.push((label.clone(), arm_result));
                    let _ = writeln!(body, "  br label %{merge_label}");
                }
                // Default block: lower ANF default if `Some`; abort() if `None` (G2/WF7).
                let _ = writeln!(body, "{default_label}:");
                if let Some(default_block) = default_arm {
                    let default_result =
                        lower_anf_block(default_block, &mut env.clone(), ssa, bbc, body, flags)?;
                    phi_entries.push((default_label.clone(), default_result));
                    let _ = writeln!(body, "  br label %{merge_label}");
                } else {
                    let _ = writeln!(body, "  call void @abort()");
                    let _ = writeln!(body, "  ret i32 0");
                }
                let _ = writeln!(body, "{merge_label}:");
                if phi_entries.is_empty() {
                    return Err(AotError::UnsupportedNode("Match with zero arms".to_owned()));
                }
                let first = &phi_entries[0].1;
                let kind = first.kind;
                let width = first.vals.len();
                for (_, lane) in &phi_entries[1..] {
                    if lane.kind != kind || lane.vals.len() != width {
                        return Err(AotError::UnsupportedNode(
                            "Match arms produce lanes of different kind or width".to_owned(),
                        ));
                    }
                }
                let mut result_vals: Vec<Operand> = Vec::with_capacity(width);
                for elem_idx in 0..width {
                    let phi_reg = ssa.fresh();
                    let phi_operands: Vec<String> = phi_entries
                        .iter()
                        .map(|(lbl, lane)| format!("[ {}, %{lbl} ]", lane.vals[elem_idx]))
                        .collect();
                    let _ = writeln!(body, "  {phi_reg} = phi i32 {}", phi_operands.join(", "));
                    result_vals.push(phi_reg);
                }
                EnvValue::Repr(Lane {
                    kind,
                    vals: result_vals,
                })
            }
            Rhs::App { .. } | Rhs::Lam { .. } | Rhs::Fix { .. } | Rhs::FixGroup { .. } => {
                return Err(AotError::UnsupportedNode(
                    "closure/recursion node in a match arm (App/Lam/Fix/FixGroup): deferred to \
                     Increment-2/3"
                        .to_owned(),
                ));
            }
        };
        env.insert(b.name.clone(), ev);
    }
    let result_ev = lookup_ev(env, anf.result())?.clone();
    result_ev.into_lane("match arm result")
}

/// Emit textual LLVM IR for the bit/trit + non-recursive-data program `node` — a `main` that
/// computes the result elements and writes them as a line to stdout (Binary: `'0'`/`'1'`;
/// Ternary: `'-'`/`'0'`/`'+'`). Deterministic. One op per output element (no opaque pass —
/// RFC-0004 §6). Returns an explicit [`AotError`] for anything outside the subset.
pub fn emit_llvm_ir(node: &Node) -> Result<String, AotError> {
    let Lowered {
        body,
        result,
        mut ssa,
        overflow,
    } = lower_program(node)?;
    let mut out =
        String::from("; mycelium direct-LLVM AOT (bit/trit + non-recursive data; M-301; M-373)\n");
    // `@putchar` for the read-back protocol; `@abort` for the match no-default trap (G2).
    out.push_str("declare i32 @putchar(i32)\n");
    out.push_str("declare void @abort()\n\n");
    out.push_str("define i32 @main() {\nentry:\n");
    out.push_str(&body);
    match overflow {
        // No trit arithmetic ⇒ no overflow path; emit the result line straight-line (unchanged IR).
        None => emit_result_line(result.kind, &result.vals, &mut ssa, &mut out),
        // Overflow possible ⇒ branch on the runtime flag: print the sentinel line on overflow, the
        // result line otherwise (the read-back protocol — never a silent wrap, G2).
        Some(ovf) => {
            let _ = writeln!(&mut out, "  br i1 {ovf}, label %ovf, label %ok");
            out.push_str("ovf:\n");
            let s = ssa.fresh();
            let _ = writeln!(
                &mut out,
                "  {s} = call i32 @putchar(i32 {})",
                OVERFLOW_SENTINEL
            );
            let snl = ssa.fresh();
            let _ = writeln!(&mut out, "  {snl} = call i32 @putchar(i32 10)");
            out.push_str("  ret i32 0\nok:\n");
            emit_result_line(result.kind, &result.vals, &mut ssa, &mut out);
        }
    }
    out.push_str("}\n");
    Ok(out)
}

/// Emit each result element as its ASCII char via `@putchar` (one op per element — a transparent
/// rendering of the computed lane, no opaque pass, RFC-0004 §6), then a trailing newline and `ret`.
fn emit_result_line(kind: LaneKind, vals: &[Operand], ssa: &mut Ssa, out: &mut String) {
    for v in vals {
        let c = emit_char_code(kind, v, ssa, out);
        let p = ssa.fresh();
        let _ = writeln!(out, "  {p} = call i32 @putchar(i32 {c})");
    }
    let nl = ssa.fresh();
    let _ = writeln!(out, "  {nl} = call i32 @putchar(i32 10)");
    out.push_str("  ret i32 0\n");
}

/// The result shape (lane kind + element count) of the program — **derived from the actual
/// lowering** ([`lower_program`]) so it can never disagree with what [`emit_llvm_ir`] emits. Used by
/// [`compile`] to know how to parse the native output.
fn result_shape(node: &Node) -> Result<(LaneKind, usize), AotError> {
    let l = lower_program(node)?;
    Ok((l.result.kind, l.result.vals.len()))
}

fn lookup_ev<'a>(env: &'a HashMap<Atom, EnvValue>, a: &Atom) -> Result<&'a EnvValue, AotError> {
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

/// Emit the LLVM IR for one bit/trit-subset op, returning the result lane. Trit-arithmetic ops also
/// push their runtime overflow `i1` register(s) onto `flags` (the caller folds them into the
/// program-level overflow flag that drives the read-back protocol).
fn emit_op(
    prim: &str,
    operands: &[&Lane],
    ssa: &mut Ssa,
    body: &mut String,
    flags: &mut Vec<String>,
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
        // Balanced-ternary addition: a fixed-width ripple-carry over the trits (LSB→MSB), with a
        // runtime overflow flag (non-zero final carry ⇒ out of m-trit range). Mirrors
        // `mycelium_core::ternary::add` digit-for-digit.
        "trit.add" => {
            let (a, b) = arity2(prim, operands)?;
            require_kind(prim, a.kind, LaneKind::Ternary)?;
            require_kind(prim, b.kind, LaneKind::Ternary)?;
            require_width(prim, a, b)?;
            let (lane, ovf) = emit_trit_add(&a.vals, &b.vals, ssa, body);
            flags.push(ovf);
            Ok(lane)
        }
        // Subtraction `a − b` = `add(a, neg(b))`: negate `b`'s trits, then the same ripple adder.
        "trit.sub" => {
            let (a, b) = arity2(prim, operands)?;
            require_kind(prim, a.kind, LaneKind::Ternary)?;
            require_kind(prim, b.kind, LaneKind::Ternary)?;
            require_width(prim, a, b)?;
            let neg_b = map1(b, ssa, body, |x, r| format!("  {r} = sub i32 0, {x}"));
            let (lane, ovf) = emit_trit_add(&a.vals, &neg_b.vals, ssa, body);
            flags.push(ovf);
            Ok(lane)
        }
        // Multiplication: shifted accumulation in a 2m-trit buffer (mirrors
        // `mycelium_core::ternary::mul`), then overflow iff any high trit is non-zero. Each `b` digit
        // scales `a` by an `i32 mul` (the digit is ±1/0, so this is exactly ±a / 0 per position).
        "trit.mul" => {
            let (a, b) = arity2(prim, operands)?;
            require_kind(prim, a.kind, LaneKind::Ternary)?;
            require_kind(prim, b.kind, LaneKind::Ternary)?;
            require_width(prim, a, b)?;
            let (lane, ovfs) = emit_trit_mul(&a.vals, &b.vals, ssa, body);
            flags.extend(ovfs);
            Ok(lane)
        }
        other => Err(AotError::UnsupportedPrim(other.to_owned())),
    }
}

/// Require two lanes to have equal element count, else an explicit [`AotError::WidthMismatch`].
fn require_width(prim: &str, a: &Lane, b: &Lane) -> Result<(), AotError> {
    if a.vals.len() == b.vals.len() {
        Ok(())
    } else {
        Err(AotError::WidthMismatch {
            prim: prim.to_owned(),
            a: a.vals.len(),
            b: b.vals.len(),
        })
    }
}

/// Emit a fixed-width balanced-ternary ripple-carry add over MSB-first trit operands `a`/`b` (equal
/// length, caller-checked). Returns the sum lane (MSB-first) and an `i1` register that is set iff the
/// final carry is non-zero (overflow). Each digit follows `mycelium_core::ternary::add`: with
/// `x = aᵢ + bᵢ + carry + 4` (always ≥ 1 so `srem`/`sdiv` are euclidean), the balanced digit is
/// `x srem 3 − 1` and the next carry is `x sdiv 3 − 1`.
fn emit_trit_add(a: &[Operand], b: &[Operand], ssa: &mut Ssa, body: &mut String) -> (Lane, String) {
    let m = a.len();
    let mut carry = "0".to_owned();
    let mut sum_lsb: Vec<Operand> = Vec::with_capacity(m);
    // Process least-significant first (the tail of the MSB-first strings).
    for i in (0..m).rev() {
        let (digit, next_carry) = emit_trit_add_step(&a[i], &b[i], &carry, ssa, body);
        sum_lsb.push(digit);
        carry = next_carry;
    }
    // Overflow iff the final carry out of the most-significant trit is non-zero.
    let ovf = ssa.fresh();
    let _ = writeln!(body, "  {ovf} = icmp ne i32 {carry}, 0");
    let vals: Vec<Operand> = sum_lsb.into_iter().rev().collect(); // back to MSB-first
    (
        Lane {
            kind: LaneKind::Ternary,
            vals,
        },
        ovf,
    )
}

/// One balanced-ternary add step: given operand trits `a`/`b` and the incoming `carry` (all `i32` in
/// `{−1,0,1}`), emit the digit + outgoing carry. Returns `(digit_reg, carry_reg)`.
fn emit_trit_add_step(
    a: &str,
    b: &str,
    carry: &str,
    ssa: &mut Ssa,
    body: &mut String,
) -> (String, String) {
    let s1 = ssa.fresh();
    let _ = writeln!(body, "  {s1} = add i32 {a}, {b}");
    let s2 = ssa.fresh();
    let _ = writeln!(body, "  {s2} = add i32 {s1}, {carry}");
    // x = s + 4 ∈ [1,7], strictly positive ⇒ srem/sdiv coincide with euclidean rem/div by 3.
    let x = ssa.fresh();
    let _ = writeln!(body, "  {x} = add i32 {s2}, 4");
    let rem = ssa.fresh();
    let _ = writeln!(body, "  {rem} = srem i32 {x}, 3");
    let digit = ssa.fresh();
    let _ = writeln!(body, "  {digit} = sub i32 {rem}, 1");
    let q = ssa.fresh();
    let _ = writeln!(body, "  {q} = sdiv i32 {x}, 3");
    let next_carry = ssa.fresh();
    let _ = writeln!(body, "  {next_carry} = sub i32 {q}, 1");
    (digit, next_carry)
}

/// Emit fixed-width balanced-ternary multiplication over MSB-first trit operands `a`/`b` (equal
/// length, caller-checked). Mirrors `mycelium_core::ternary::mul`: shifted accumulation of `±a` into
/// a 2m-trit buffer, returning the low `m` trits (MSB-first) and the overflow `i1` flags — one per
/// non-zero high trit, plus each accumulation's carry (provably zero, OR-ed in as an honest net).
fn emit_trit_mul(
    a: &[Operand],
    b: &[Operand],
    ssa: &mut Ssa,
    body: &mut String,
) -> (Lane, Vec<String>) {
    let m = a.len();
    if m == 0 {
        return (
            Lane {
                kind: LaneKind::Ternary,
                vals: Vec::new(),
            },
            Vec::new(),
        );
    }
    let wide = 2 * m;
    // LSB-first views of the operands and a 2m-wide accumulator initialised to zero.
    let a_lsb: Vec<&Operand> = a.iter().rev().collect();
    let b_lsb: Vec<&Operand> = b.iter().rev().collect();
    let mut acc: Vec<Operand> = vec!["0".to_owned(); wide];
    let mut flags: Vec<String> = Vec::new();

    for (k, &bk) in b_lsb.iter().enumerate() {
        // Partial = (a scaled by digit bk) shifted left by k, in a 2m-wide LSB-first buffer. The
        // digit is ±1/0, so `aⱼ * bk` is exactly ±aⱼ / 0 — the per-digit factor, no carry yet.
        let mut partial: Vec<Operand> = vec!["0".to_owned(); wide];
        for (j, &aj) in a_lsb.iter().enumerate() {
            let p = ssa.fresh();
            let _ = writeln!(body, "  {p} = mul i32 {aj}, {bk}");
            partial[k + j] = p;
        }
        let (next_acc, carry) = emit_ripple_add_lsb(&acc, &partial, ssa, body);
        acc = next_acc;
        // The 2m-wide sum cannot truly overflow for m-trit operands; OR the carry in anyway so a
        // codegen slip can never pass silently (honest net, never a fabricated guarantee).
        let c = ssa.fresh();
        let _ = writeln!(body, "  {c} = icmp ne i32 {carry}, 0");
        flags.push(c);
    }
    // The product fits in m trits iff the high half (positions [m, 2m)) is all zero.
    for hi in &acc[m..] {
        let f = ssa.fresh();
        let _ = writeln!(body, "  {f} = icmp ne i32 {hi}, 0");
        flags.push(f);
    }
    let vals: Vec<Operand> = acc[..m].iter().rev().cloned().collect(); // low m trits, MSB-first
    (
        Lane {
            kind: LaneKind::Ternary,
            vals,
        },
        flags,
    )
}

/// Ripple-carry add over two equal-length **LSB-first** trit-operand vectors. Returns the sum
/// (LSB-first) and the final carry register. The shared inner adder for [`emit_trit_mul`].
fn emit_ripple_add_lsb(
    a: &[Operand],
    b: &[Operand],
    ssa: &mut Ssa,
    body: &mut String,
) -> (Vec<Operand>, String) {
    let mut carry = "0".to_owned();
    let mut sum: Vec<Operand> = Vec::with_capacity(a.len());
    for (ai, bi) in a.iter().zip(b) {
        let (digit, next_carry) = emit_trit_add_step(ai, bi, &carry, ssa, body);
        sum.push(digit);
        carry = next_carry;
    }
    (sum, carry)
}

/// Fold a list of `i1` overflow flags into one (`or i1` chain), or `None` if empty. Deterministic.
fn fold_or(flags: &[String], ssa: &mut Ssa, body: &mut String) -> Option<String> {
    let mut it = flags.iter();
    let mut acc = it.next()?.clone();
    for f in it {
        let r = ssa.fresh();
        let _ = writeln!(body, "  {r} = or i1 {acc}, {f}");
        acc = r;
    }
    Some(acc)
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
        // Read-back protocol: the sentinel line means the native arithmetic overflowed the m-trit
        // range — an explicit error, never a silently-wrapped result (matches `EvalError::Overflow`).
        if line.as_bytes() == [OVERFLOW_SENTINEL] {
            return Err(AotError::Overflow(format!(
                "fixed-width result out of {}-trit range",
                self.width
            )));
        }
        decode_result(self.kind, self.width, line.chars())
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

pub(crate) fn run_tool(tool: &str, args: &[&str]) -> Result<(), AotError> {
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

pub(crate) fn path(p: &Path) -> Result<&str, AotError> {
    p.to_str()
        .ok_or_else(|| AotError::Run(format!("non-utf8 path {}", p.display())))
}

pub(crate) fn unique_tmp_dir() -> Result<std::path::PathBuf, AotError> {
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
pub(crate) struct TmpDir(pub(crate) std::path::PathBuf);
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

    fn binop(prim: &str, a: Vec<Trit>, b: Vec<Trit>) -> Node {
        Node::Op {
            prim: prim.into(),
            args: vec![Node::Const(ternary(a)), Node::Const(ternary(b))],
        }
    }

    #[test]
    fn trit_add_emits_ripple_carry_ir() {
        // Mutant-witness: a non-carry (elementwise) add would not emit the srem/sdiv-by-3 balancing
        // or the icmp overflow flag the read-back protocol branches on.
        let ir = emit_llvm_ir(&binop(
            "trit.add",
            vec![Trit::Pos, Trit::Neg, Trit::Neg],
            vec![Trit::Zero, Trit::Pos, Trit::Pos],
        ))
        .unwrap();
        assert!(ir.contains("srem i32") && ir.contains("sdiv i32")); // balanced-digit normalisation
        assert!(ir.contains("icmp ne i32")); // overflow flag
        assert!(ir.contains("br i1")); // read-back branch
        assert!(ir.contains("putchar(i32 33)")); // overflow sentinel '!'
    }

    #[test]
    fn arithmetic_emission_is_deterministic() {
        let p = binop(
            "trit.mul",
            vec![Trit::Zero, Trit::Pos, Trit::Neg],
            vec![Trit::Zero, Trit::Pos, Trit::Zero],
        );
        assert_eq!(emit_llvm_ir(&p), emit_llvm_ir(&p));
    }

    #[test]
    fn refuses_arithmetic_width_mismatch() {
        // Mutant-witness: dropping the width check would emit a ragged ripple-carry.
        let prog = binop("trit.add", vec![Trit::Pos, Trit::Zero], vec![Trit::Pos]);
        assert!(matches!(
            emit_llvm_ir(&prog),
            Err(AotError::WidthMismatch { .. })
        ));
    }

    #[test]
    fn refuses_bit_arithmetic_on_binary_lane() {
        // Mutant-witness: dropping require_kind would let trit.add ripple over a binary lane.
        let prog = Node::Op {
            prim: "trit.add".into(),
            args: vec![
                Node::Const(binary(vec![true, false])),
                Node::Const(binary(vec![false, true])),
            ],
        };
        assert!(matches!(
            emit_llvm_ir(&prog),
            Err(AotError::UnsupportedPrim(_))
        ));
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

    #[test]
    fn native_trit_add_matches_oracle() {
        // 5 + 4 = 9 in 3 trits: [+,-,-] + [0,+,+] = [+,0,0]. Mutant-witness: a missing carry would
        // yield the elementwise (wrong) sum, and a wrong balancing constant would mis-encode.
        let prog = binop(
            "trit.add",
            vec![Trit::Pos, Trit::Neg, Trit::Neg],
            vec![Trit::Zero, Trit::Pos, Trit::Pos],
        );
        match compile_and_run(&prog) {
            Ok(v) => assert_eq!(
                v.payload(),
                &Payload::Trits(vec![Trit::Pos, Trit::Zero, Trit::Zero])
            ),
            Err(AotError::ToolchainMissing(_)) => {}
            Err(e) => panic!("unexpected AOT error: {e}"),
        }
    }

    #[test]
    fn native_trit_sub_matches_oracle() {
        // 9 - 4 = 5 in 3 trits: [+,0,0] - [0,+,+] = [+,-,-].
        let prog = binop(
            "trit.sub",
            vec![Trit::Pos, Trit::Zero, Trit::Zero],
            vec![Trit::Zero, Trit::Pos, Trit::Pos],
        );
        match compile_and_run(&prog) {
            Ok(v) => assert_eq!(
                v.payload(),
                &Payload::Trits(vec![Trit::Pos, Trit::Neg, Trit::Neg])
            ),
            Err(AotError::ToolchainMissing(_)) => {}
            Err(e) => panic!("unexpected AOT error: {e}"),
        }
    }

    #[test]
    fn native_trit_mul_matches_oracle() {
        // 2 * 3 = 6 in 3 trits: [0,+,-] * [0,+,0] = [+,-,0]. Mutant-witness: a wrong shift in the
        // shifted-accumulate, or reading the high (overflow) half, would diverge.
        let prog = binop(
            "trit.mul",
            vec![Trit::Zero, Trit::Pos, Trit::Neg],
            vec![Trit::Zero, Trit::Pos, Trit::Zero],
        );
        match compile_and_run(&prog) {
            Ok(v) => assert_eq!(
                v.payload(),
                &Payload::Trits(vec![Trit::Pos, Trit::Neg, Trit::Zero])
            ),
            Err(AotError::ToolchainMissing(_)) => {}
            Err(e) => panic!("unexpected AOT error: {e}"),
        }
    }

    #[test]
    fn native_trit_add_overflow_is_explicit() {
        // 4 + 4 = 8 in 2 trits (max magnitude 4) overflows. The native path must report it through
        // the read-back protocol — an explicit Overflow, never a silent wrap. Mutant-witness:
        // dropping the final-carry flag would print a wrapped result instead.
        let prog = binop(
            "trit.add",
            vec![Trit::Pos, Trit::Pos],
            vec![Trit::Pos, Trit::Pos],
        );
        match compile_and_run(&prog) {
            Ok(v) => panic!("overflow must not produce a value, got {:?}", v.payload()),
            Err(AotError::Overflow(_)) => { /* expected */ }
            Err(AotError::ToolchainMissing(_)) => {}
            Err(e) => panic!("unexpected AOT error: {e}"),
        }
    }

    #[test]
    fn native_trit_mul_overflow_is_explicit() {
        // 4 * 4 = 16 in 2 trits overflows (high trits non-zero).
        let prog = binop(
            "trit.mul",
            vec![Trit::Pos, Trit::Pos],
            vec![Trit::Pos, Trit::Pos],
        );
        match compile_and_run(&prog) {
            Ok(v) => panic!("overflow must not produce a value, got {:?}", v.payload()),
            Err(AotError::Overflow(_)) => {}
            Err(AotError::ToolchainMissing(_)) => {}
            Err(e) => panic!("unexpected AOT error: {e}"),
        }
    }
}
