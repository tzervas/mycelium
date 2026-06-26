# Design Note DN-25 — Road to Full-Language 1.0.0 (program map)

| Field | Value |
|---|---|
| **Note** | DN-25 |
| **Status** | **Draft** (2026-06-23; planning capture, DN-17 posture) |
| **Feeds** | ADR-022 (the full-language 1.0.0 gate of record); DN-19 (Road to 1.0.0 — kernel/core, now track T1); DN-14 (self-hosting gate) |
| **Date** | June 23, 2026 |
| **Decides** | *Nothing normatively* — advisory. The **operational map** for ADR-022: every gap to a full-language 1.0.0, mapped to a track → epic → child issues → doc stub → kickoff, with the dependency graph + sequencing. ADR-022 is the gate; this is the route. |
| **Task** | The full-language 1.0.0 program (epics E10-1…E18-1) |

> **Posture (honesty rule / VR-5).** Advisory planning capture. Status of each track is reported at the
> strength evidenced. All new specs are `Draft`, all new issues `needs-design`. Nothing here enacts a
> release or upgrades a tag. The binding gate is ADR-022 §5.

---

## 1. Purpose

ADR-022 defines *what* a full-language 1.0.0 is (the dual-version model + the T1–T9 Definition of
Done). This note is *how we get there*: the complete gap map turned into trackable work — so a fresh
session can pick up any track from a kickoff and know its scope, dependencies, and done-criteria.

The north star (ADR-022 §1): the **core** may stay Rust and reach `core 1.0.0` first; the **language**
reaches `lang 1.0.0` only when the **stdlib + every library/phylum beyond the bare core is written in
Mycelium (`.myc`), stable, and fully usable**.

## 2. Track → epic → doc → kickoff map

| Track | Epic | New design doc (stub) | Kickoff | Phase |
|---|---|---|---|---|
| **T1** Core/kernel 1.0.0 sub-gate (honesty-integrity durability) | **E10-1** | — (ADR-021/022, DN-19) | `c10` | 8 |
| **T2** Surface-language completeness & grammar | **E11-1** | RFC-0025 (operator syntax), RFC-0030 (L3 grammar) | `s10` | 5 |
| **T3** Runtime & concurrency execution maturity | **E12-1** | RFC-0027 (memory mgmt & reclamation) | `r10` | 7 |
| **T4** Standard library **in Mycelium** | **E13-1** | RFC-0031 (self-hosted stdlib composition) | `lib10` | 5 |
| **T5** FFI & system interface | **E14-1** | RFC-0028 (FFI & system interface) | `ffi10` | 7 |
| **T6** Native AOT maturity, optimization & accel — **→ `1.1` (post-1.0.0; ADR-022 §8 Q4)** | **E15-1** | RFC-0029 (AOT opt, codegen maturity & JIT) | `aot10` | 6 |
| **T7** Toolchain, IDE & package distribution | **E16-1** | RFC-0026 (editor highlighting grammar) | `tool10` | 8 |
| **T8** Documentation, stability & release | **E17-1** | ADR-023 (stability & API-compat guarantees) | `rel10` | 8 |
| **T9** Self-hosting capstone | **E18-1** | DN-26 (self-hosting bootstrap plan) | `boot10` | 5 |

Existing epics subsumed/extended by the tracks: E6-1 (native MLIR → T6), E7-1/E7-3/E7-5 (surface →
T2), E7-2 (runtime vocabulary → T3), E9-1 (highlighting → T7). They are referenced via `depends_on`,
not duplicated.

## 3. Issue inventory (M-700 – M-743)

- **E10-1 (T1):** M-700 Medium-findings ledger · M-701 WS8 durability (mutants/proptest/fuzz) · M-702 cargo-deny/audit wiring · M-703 core 1.0.0 tag + ADR-021→Enacted.
- **E11-1 (T2):** M-704 full HOF/closures · M-705 operator syntax · M-706 L3 EBNF grammar · M-707 RFC-0020 L2 surface · M-708 generics/traits/effects stabilization.
- **E12-1 (T3):** M-709 real scheduler · M-710 runtime vocabulary execution · M-711 deadlock-freedom · M-712 memory reclamation · M-713 supervision/cancellation.
- **E13-1 (T4):** M-714 stdlib composition/layout · M-715 core/prelude in `.myc` · M-716 collections in `.myc` · M-717 text/fmt in `.myc` · M-718 math/numerics in `.myc` · M-719 stdlib conformance + stability.
- **E14-1 (T5):** M-720 FFI surface · M-721 `wild` execution · M-722 syscall binding · M-723 time/rand bindings · M-724 FFI safety audit.
- **E15-1 (T6):** M-725 libMLIR integration · M-726 optimization passes · M-727 JIT · M-728 BitNet accel · M-729 codegen differential durability.
- **E16-1 (T7):** M-730 full LSP · M-731 highlighting delivery · M-732 package manager · M-733 toolchain UX · M-734 reproducible distribution.
- **E17-1 (T8):** M-735 language reference + tutorial · M-736 stdlib API docs · M-737 stability/API-compat guarantees · M-738 full-language 1.0.0 release act · **M-743 MIT-only licensing audit + enforcement**.
- **E18-1 (T9):** M-739 self-hosting bootstrap plan · M-740 port L1 frontend to `.myc` · M-741 self-hosted toolchain ratification · M-742 self-hosting CI gate.

Every epic + issue carries **user stories** + a **Definition of Done** (ADR-022 §7 convention).

## 4. Dependency graph & sequencing

```
core axis:   T1 (E10-1) ───────────────► core 1.0.0 tag  (independent; first)

lang axis:   T2 (E11-1) ─┬─► T4 (E13-1) ─┬─► T9 (E18-1) ─► lang 1.0.0 (E17-1/M-738)
                         │               │
             T3 (E12-1) ─┘               │   (T8 docs/stability + T7 tooling: continuous)
             T5 (E14-1) ─► (system libs in T4)
             T6 (E15-1)  (perf — DEFERRED to 1.1, ADR-022 §8 Q4; post-1.0.0, off the gate)
```

- **Wave A (now, parallel):** T1 (core gate) ∥ T2 (surface) ∥ T7/T8 continuous tooling/docs.
- **Wave B:** T3 (runtime) ∥ T5 (FFI) — unblock the system-touching stdlib modules.
- **Wave C:** T4 (stdlib in Mycelium) — the heart; depends on T2 (+ T3/T5 for system modules).
- **Wave D:** T9 (self-hosting capstone) — depends on T2 + T4; then T8/M-738 cuts `lang 1.0.0`.
- **Post-1.0.0 (not a gate):** T6 (native AOT/perf — `aot10`) is **rolled to `1.1`** (ADR-022 §8 Q4) — patched in as a QoL/perf enhancement; the interpreter (+ direct-LLVM kernel subset) is the trusted 1.0.0 base. Runs post-1.0.0 alongside the T9 self-host capstone.

## 5. Kickoffs (the parallelizable heads)

Each track has a stowed kickoff in `.claude/kickoffs/` driven on its protected head branch
(`claude/head/<uid>`) per the Wave-N workflow: `c10` · `s10` · `r10` · `lib10` · `ffi10` · `aot10` ·
`tool10` · `rel10` · `boot10`. The README indexes them with status + dependency order. Disjoint file
ownership keeps them collision-free; `lib10`/`boot10` are the long poles (they define `lang 1.0.0`).

## 6. Conventions (ADR-022 §7)

User stories + Definition of Done on every epic/issue; MIT-only first-party licensing. See ADR-022 §7
and CONTRIBUTING.md.

## 7. Honesty & grounding

Today's state is grounded: `core` is capability-complete and near its gate (ADR-021: A1/A5/B1/B2 met,
A2/A3/A4 open); the surface has generics/traits/HOF/phylum landed (E7-x) but operators/L3/closures
remain; the stdlib is **Rust** save `lib/std/result.myc`; runtime is the skeletal v0 R1 API; the
native path is direct-LLVM on a kernel subset. The gap to `lang 1.0.0` is therefore real and large —
this note maps it without overclaiming any of it as done (VR-5/G2).

---

## 8. Changelog

- **2026-06-25 — Sequencing note (D5/D3; post corpus-alignment audit; advisory, no status move).**
  - **D5 — sequencing confirmed.** **T4 (stdlib-in-Mycelium, M-714–719)** and **T9 (self-hosting capstone)**
    stay **post-core surface work**, sequenced **behind grammar → E19-1 → runtime Phase-7**: the surface
    grammar must complete (T2 — operators landed via RFC-0025/M-705; L3/closures + the `[]`-for-type-args
    grammar wave per DN-31 still pending), then **E19-1** (`Repr::Seq`/`Repr::Bytes` value-model additions,
    ADR-024/025/026/027) must land + verify, then runtime **Phase-7** activation, before T4's `.myc` stdlib
    and T9's self-hosting are on the critical path. This matches the §4 wave graph (Wave A → B → C → D); it
    upgrades nothing as done (VR-5).
  - **D3 — strict-sequence gate (also recorded in ADR-024).** The **core 1.0.0 (T1) tag act (M-703)** is
    committed **only after E19-1 is implemented + verified** (M-703 `depends_on E19-1`; the three-way
    differential + M-752 conformance gate green). The §3 inventory lists M-703 under E10-1/T1 and E19-1's
    M-749/M-750 under E13-1/T4 — D3 makes the cross-track ordering explicit: the core tag waits on E19-1.
  - **§7 snapshot note (Low locator).** §7's "stdlib is **Rust** save `lib/std/result.myc`" was accurate at
    2026-06-23; `option.myc` + `cmp.myc` landed 2026-06-24 (append-only; the §7 prose is left intact).
