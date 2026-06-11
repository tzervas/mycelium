# Research Record 04 — Runtime & Concurrency Targets T4 (Pass 4)

> **What this file is.** A durable record of the fourth research pass, which grounds the Runtime
> tier that ADR-012 §7.3 flagged as aspirational and that **RFC-0008** (Runtime & Concurrency
> Execution Model) now defines: scope, results by target, the positions they support, and the
> source base. Conducted 2026-06-10 as three parallel deep-dives with primary-source
> verification; load-bearing claims were checked against primary sources, and unverified details
> are flagged in the per-target and end-of-record uncertainty registers. Findings are labeled
> **T4.1–T4.6** (continuing the T0–T3 scheme) and map onto RFC-0008's RT1–RT7 invariants and
> §4.5 vocabulary table.

## Scope

Ground, with evidence, the execution model the Runtime vocabulary presupposes (ADR-012 §7.3):
the concurrency unit and its scheduling/lifetime discipline (T4.1 → `hyph`, RT1/RT2/RT7), state
merge and decentralized coordination (T4.2 → `anas`/`cmn`, RT5/RT6), code/data mobility,
placement, and transport (T4.3 → `xloc`/`rhizo`/`forage`, RT3/RT4), durability and deployable
artifacts (T4.4 → `sclrt`/`spore`), failure and supervision (T4.5 → `reclaim`, RT4/RT5/RT7), and
execution-mode switching (T4.6 → `dimorph`). The question throughout: which disciplines let a
value-semantics, honesty-first, totality-aware substrate extend its guarantees across
concurrency and distribution instead of surrendering them.

## Results by target

### T4.1 — The concurrency unit and its semantics (→ `hyph`; RT1/RT2/RT7)

- **Lightweight isolated processes (Erlang/BEAM).** Share-nothing processes with process-local
  heaps (default initial size 233 words, designed for "hundreds of thousands or even millions of
  processes"); per-process heaps let GC reclaim memory without global synchronization; the cost
  is copying terms between heaps (Erlang/OTP Efficiency Guide, "Processes"). Scheduling is
  **reduction-counted** — each process runs a budget of reductions before descheduling
  (Stenman, *The BEAM Book*, "Scheduling") — an industrial **fuel-metered scheduler**.
- **Goroutines (contrast case).** 2 KB-minimum growable stacks make spawn cheap, but goroutines
  share a mutable heap (races possible) and are **unscoped** — `go` returns immediately and the
  child may outlive its caller, exactly the pattern structured concurrency rejects.
- **Effect-handler fibers.** OCaml 5 retrofits fibers via effect handlers at ~1% mean overhead
  for non-handler code (Sivaramakrishnan et al., *Retrofitting Effect Handlers onto OCaml*,
  PLDI 2021); Leijen shows async/await, interleaving, cancellation, and timeouts as a *library*
  over algebraic effects with block-scoped interleaving (*Structured Asynchrony with Algebraic
  Effects*, TyDe 2017) — the scheduler is user-definable and reifiable, no black-box runtime.
- **Structured concurrency.** Coined by Sústrik for libdill (2016): "lifetimes of concurrent
  functions are cleanly nested," a tree of coroutines with explicit cancellation; generalized by
  N. J. Smith's **nursery** (2018): every spawn happens inside an explicitly passed scope that
  does not exit until all children complete — restoring the "black box rule" (a function's
  concurrent side effects are over when it returns), by analogy with eliminating `goto`.
  Industrial adoption: Kotlin made scope mandatory (kotlinx.coroutines 0.26.0, 2018); Java's
  virtual threads are final (JEP 444, JDK 21) but `StructuredTaskScope` is **still in preview as
  of June 2026** (JEP 428→…→525, sixth preview targeted to JDK 26) — the lifetime rule is
  settled, the join/policy *surface* is hard to finalize (eight preview rounds and counting).
- **Deterministic parallelism.** Kahn process networks (Kahn, IFIP Congress 1974): processes as
  continuous functions on stream histories; network behavior is the least fixed point, hence
  **independent of scheduling/timing**, *provided* reads block and emptiness cannot be tested —
  determinism is bought by banning observation of nondeterministic facts. Haskell's Par monad
  (Marlow, Newton, Peyton Jones, Haskell Symposium 2011): deterministic dataflow via
  single-assignment IVars. DPJ (Bocchino, Adve et al., OOPSLA 2009): determinism by a
  region-based type-and-effect system. **LVars** (Kuper & Newton, *LVars: Lattice-based Data
  Structures for Deterministic Parallelism*, FHPC '13 / ICFP 2013) — the strongest precedent:
  shared variables whose states form a user-specified lattice, writes are least-upper-bound
  (monotonic) updates, reads are threshold reads that block until the state passes a designated
  lower-bound set, and the calculus (λLVar) is **proven deterministic**.
- **Fuel/clock interpreters extend to concurrency.** CakeML's trusted semantics is a clocked
  functional big-step interpreter (Kumar, Myreen, Norrish, Owens, POPL 2014; Owens et al., ESOP
  2016). Two precedented routes compose: (a) fuel-as-scheduling-quantum (BEAM's reduction
  counting) — a deterministic round-robin over cooperative fuel-metered steps yields a total,
  reproducible interpreter; (b) schedule-independence — if communication is KPN/LVar-shaped, the
  observable is provably schedule-independent, so the trusted base need exhibit only *one* fair
  schedule and "parallel" becomes a pure performance mode.

**Position.** A `hyph` is a *scoped, fuel-metered, isolated* execution context. Erlang-grade
isolation is free in Mycelium (immutable acyclic values: copying degenerates to reference
sharing). Nursery scoping is what makes concurrency compatible with precise reclamation (a
scope's values die with it — no orphan can resurrect them) and with the totality judgment (a
`total` definition that spawns must be total *inclusive of its children*; the black-box rule is
the condition under which totality composes — an unscoped `go` would make `total` a lie). LVars
is the load-bearing communication precedent: monotone writes + threshold reads over a lattice
give determinism **by theorem** — on the honesty lattice that means inter-`hyph` programs can
carry `Proven`-deterministic tags with *checked* side-conditions (monotonicity, threshold
well-formedness) rather than `Declared` ones, and Mycelium already owns semilattice machinery.
The trusted base extends with no new trust story (clocked interpretation + deterministic
cooperative stepping; Kahn/LVar discipline makes the scheduler semantically irrelevant — RT2).
Java's preview history counsels: fix the lifetime rule (RT7) now; keep the join/policy surface
explicitly open (R8-Q1).

### T4.2 — State merge & decentralized coordination (→ `anas`/`cmn`; RT5/RT6)

- **CRDTs / strong eventual consistency** (verified verbatim from the author PDF): Shapiro,
  Preguiça, Baquero, Zawirski, *Conflict-free Replicated Data Types*, SSS 2011 (LNCS 6976,
  pp. 386–400). SEC = eventual delivery + termination + strong convergence ("correct replicas
  that have delivered the same updates have equivalent state"). **Theorem 1 (CvRDT):** any
  state-based object satisfying the *monotonic semilattice property* — (i) states form a
  join-semilattice, (ii) merge computes the LUB, (iii) updates are monotonically non-decreasing
  — is SEC. **Theorem 2 (CmRDT):** causal delivery + commutativity of concurrent updates ⇒ SEC.
  **Important precision:** the paper proves these conditions **sufficient**, not "iff" — never
  write "converges iff join-semilattice". Proofs live in the companion INRIA report RR-7506
  (verified as the SSS paper's own reference; the report itself was unreachable this pass).
  Stronger basis available for a `Proven` tag: SEC is **mechanically verified in Isabelle/HOL**
  (Gomes, Kleppmann, Mulligan, Beresford, OOPSLA 2017).
- **Join/meet duality** (project-internal observation, not an external claim): CRDT payload
  merge is a **join** (LUB) on the state semilattice; Mycelium's guarantee composition is a
  **meet** (weakest-wins) on the honesty lattice. Same algebraic laws (commutative, associative,
  idempotent), so a fused value is well-defined as ⟨payload ⊔, guarantee ⊓⟩, order-insensitively
  — which is what makes fusion compatible with the deterministic posture (RT2/RT6).
- **Session types**: Honda (CONCUR '93); Honda, Vasconcelos, Kubo (ESOP '98); multiparty: Honda,
  Yoshida, Carbone (POPL 2008; JACM 63(1) 2016). Static guarantees: communication safety (no
  send/receive mismatch) and session fidelity (the interaction follows the declared protocol);
  MPST projects a global type onto per-participant local types. **Progress/deadlock-freedom is
  not free** — it needs extra conditions under session interleaving and must be tagged
  separately. Maturity: 30 years of theory, library-level implementations; limited mainstream
  production adoption.
- **Gossip/epidemic protocols** (verified verbatim): Demers et al., *Epidemic Algorithms for
  Replicated Database Maintenance*, PODC 1987 — anti-entropy is "extremely reliable" but
  expensive; rumor mongering trades cost for a nonzero residue; "the probability that the
  information has not converged is exponentially decreasing with time" — **exactly a δ-bounded
  guarantee, not a certainty**. HyParView (DSN 2007) and Plumtree/epidemic broadcast trees
  (SRDS 2007): partial-view membership + eager-tree/lazy-gossip broadcast, with *experimental*
  (i.e. `Empirical`) reliability claims. libp2p **gossipsub** (spec repo, verified): v1.0
  normative spec; v1.1 security hardening (peer scoring, flood publishing, outbound quotas,
  PRUNE backoff, spam penalties, opportunistic grafting); delivery remains best-effort/
  probabilistic; scoring is a heuristic defense, not a proof.

**Position.** `anas` (fusion) is specified as the pair **⟨payload join, `Meta` meet⟩** (RT6):
the CRDT discipline yields a `Proven` convergence tag *only* with its side-conditions checked —
semilattice laws + monotonicity for state-based merge (property-testable, per house rule
SC-2-style), or causal delivery + commutativity for op-based merge, where **causal delivery is a
runtime obligation the mesh must certify, not an axiom**. The fusion protocol is a reified,
EXPLAINable object (which merge, which lattice, which delivery assumption). Session/MPST typing
is the right shape for `anas` handshake protocols — safety/fidelity as checked static guarantees,
deadlock-freedom tagged separately (it does not come free). For `cmn`, the literature is
unambiguous: gossip guarantees are probabilistic — analytic epidemic bounds fit
Proven-with-δ via the existing `ProbabilityBound` machinery (ADR-010/011) when the model's
assumptions hold; protocol-engineering resilience numbers are `Empirical` (RT5).

### T4.3 — Code/data mobility, placement, transport (→ `xloc`/`rhizo`/`forage`; RT3/RT4)

- **Unison: content-addressed code makes computations relocatable** (verified against the
  primary source): "Each Unison definition is identified by a hash of its syntax tree";
  "arbitrary computations can just be moved from one location to another, with missing
  dependencies deployed on the fly" — the recipient inspects received code for hashes it is
  missing, requests them on demand, and caches them (unison-lang.org, "The big idea"; Unison
  Cloud, "Our approach"). Mobility is a *consequence* of content addressing, not a subsystem —
  and Mycelium already adopted Unison identity (ADR-003).
- **Cloud Haskell — the weaker baseline.** Epstein, Black & Peyton Jones, *Towards Haskell in
  the Cloud* (Haskell Symposium 2011) serializes closures via `static` pointers — code *keys*,
  not code; verified limitation: "only processes launched from the same program binary are
  guaranteed to use the same set of keys" (GHC User's Guide §static pointers). Same-binary
  deployment everywhere is exactly the constraint content addressing dissolves.
- **Erlang distribution — transparency's honesty costs.** Message passing/links/monitors are
  transparent across nodes when pids are used, but registered names are node-local, and the
  default distribution protocol is clear text — security is "against accidental misuse"
  (Erlang/OTP, "Distributed Erlang"). Transparent-ish distribution leaks its seams.
- **Legion: placement is policy, never semantics** (verified verbatim from the SC 2012 paper):
  "This mapping API is designed so that any user-supplied mapping strategy can only affect the
  performance of applications, not their correctness"; "program correctness is unaffected by
  mapper decisions, which can only impact performance" (Bauer, Treichler, Slaughter, Aiken,
  *Legion: Expressing Locality and Independence with Logical Regions*, SC 2012). The paper
  explicitly contrasts Chapel, where user domain maps must be correct for the program to be.
- **Dataflow engines.** Naiad/timely dataflow: logical timestamps unify batch, streaming, and
  iteration (Murray et al., SOSP 2013). Ray: dynamic task graphs over an **immutable**
  shared-memory object store with lineage-based fault tolerance (Moritz et al., OSDI 2018) —
  the immutable store is the value-semantics-adjacent piece.
- **Backpressure.** Reactive Streams: exactly four interfaces (`Publisher`/`Subscriber`/
  `Subscription`/`Processor`) with demand-credit flow — "the total number of onNext's signalled
  … MUST be less than or equal to the total number of elements requested" (reactive-streams.org
  spec 1.0.4); TCP receive windows (RFC 9293) are the transport-level ancestor.
- **Work stealing** (→ `forage` bounds): expected execution time **T₁/P + O(T∞)** for *fully
  strict* computations — verified (Blumofe & Leiserson, JACM 46(5), 1999). The side-conditions
  matter: under the honesty rule this imports as `Proven` only with the fully-strict condition
  checked, else `Empirical`.

**Position.** The Legion result is load-bearing for RT3: placement/routing (`xloc` routes,
`rhizo` priority paths, `forage` decisions) must be reified *mapping policy* that can change
performance but provably cannot change meaning — the runtime analogue of certified swaps.
Unison proves content-addressed identity suffices for ship-by-hash mobility with on-demand
dependency sync, strictly dominating static-pointer schemes; Erlang shows what transparency
costs in honesty, so remoteness stays explicit in types (RT4). Backpressure has a small precise
spec precedent. `forage` may be signal-driven, but per RFC-0005 the deciding artifact stays a
total, non-learned, content-addressed, EXPLAINable policy — *learned placement as inspectable
policy* has **no found precedent** (flagged novel, open question), and work-stealing bounds are
citable only with their side-conditions tagged.

### T4.4 — Durability, checkpoint/restore, deployable artifacts (→ `sclrt`/`dorm`, `spore`)

- **CRIU — OS-level checkpointing is inherently leaky** (verified): non-checkpointable
  categories include open devices, debugger-attached tasks, most socket types, files passed
  over UNIX sockets, X/graphical state; external resources need explicit flags
  (criu.org/What_cannot_be_checkpointed). Checkpointing *ambient OS state* is a catalogue of
  exceptions. Lineage: Condor's user-level checkpoint/migration (Litzkow, Livny & Mutka, ICDCS
  1988); Sprite process migration (Douglis & Ousterhout, SP&E 1991 — background, not re-fetched).
- **Durable execution: determinism is the price of replay** (verified): Temporal — "Workflow
  code must be deterministic to support replay … given the same input"; replayed commands are
  compared against the event history and a mismatch is a non-determinism *error*
  (docs.temporal.io). Azure Durable Functions — event sourcing + full re-execution on resume;
  "orchestrator function code must be deterministic" (MS Learn). Both systems **bolt determinism
  onto** nondeterministic host languages via code constraints and runtime policing.
- **Content-addressed artifacts.** Nix: input-addressed vs content-addressed store paths (RFC
  0062; CA-derivations still experimental). OCI: an image is a DAG of content-addressable blobs,
  every node identified by the SHA-256 digest of its content (OCI Image Format Specification).
  Wasm: WASI 0.3 shipped Feb 2026 with native async; WASI 1.0 targeted 2026 (roadmap-declared,
  not ratified).
- **Live migration.** Pre-copy VM migration with 60–210 ms downtimes (Clark et al., NSDI 2005);
  Wasm-level snapshot/migration exists (Wanco, POPL 2025 SRC; EdgeSys '24) — linear memory and
  stack discipline make snapshots portable in a way process images are not.

**Position.** The durable-execution finding is the strongest pro-corpus argument in the pass:
Temporal/Azure must *impose and police* determinism, whereas a total, deterministic,
value-semantics computation satisfies the replay precondition **by construction** — so a
`sclrt`/`dorm` checkpoint degenerates to values + a continuation reference, all
content-addressable, and "resume" is honest replay rather than best-effort process surgery
(CRIU's exception catalogue is the cautionary contrast: ambient effects are exactly what cannot
be checkpointed, supporting reified effect boundaries). `spore` has three independent precedents
converging on hash-identified DAGs of code+config+state (Nix, OCI, Unison); the RFC-0003
reconstruction manifest slots in as one digest-referenced component of the spore DAG (the
ADR-012 §7.4 reconciliation, owned by the RFC-0003 revision). The `dorm` rename is apt: a
checkpoint is a dormant, durable, content-addressed *value*, not a frozen process.

### T4.5 — Failure, supervision, partial-failure honesty (→ `reclaim`; RT4/RT5/RT7)

- **Erlang/OTP supervision** (verified verbatim from the thesis PDF): Armstrong, *Making
  reliable distributed systems in the presence of software errors*, PhD thesis, KTH/SICS 2003.
  "Let some other process do the error recovery. If you can't do what you want to do, die. Let
  it crash. Do not program defensively." Links propagate exit signals through the link set;
  workers/supervisors form restart hierarchies; the thesis connects this to Gray's fail-fast
  modules and Candea–Fox crash-only software.
- **FLP impossibility** (verified verbatim): Fischer, Lynch, Paterson, JACM 32(2), 1985 — "no
  completely asynchronous consensus protocol can tolerate even a single unannounced process
  death", and the operative line: **"it is impossible for one process to tell whether another
  has died (stopped entirely) or is just running very slowly."** A liveness impossibility for
  deterministic protocols in the fully asynchronous model (not a safety result).
- **Unreliable failure detectors**: Chandra & Toueg, JACM 43(2), 1996 — detectors characterized
  by completeness and accuracy; consensus is solvable atop detectors that make infinitely many
  mistakes; the framing concedes FLP and relocates the assumption into an explicit, admittedly
  unreliable oracle. **φ-accrual** (Hayashibara, Défago, Yared, Katayama, SRDS 2004): a
  *continuous suspicion level* (φ from the heartbeat inter-arrival distribution) instead of
  binary up/down — "decouple monitoring and interpretation"; productionized by Akka.
- **Partial failure** (verified): Waldo, Wyant, Wollrath, Kendall, *A Note on Distributed
  Computing*, Sun SMLI TR-94-29, 1994. Four irreducible local/remote differences — latency,
  memory access, **partial failure**, concurrency; partial failure has no local analog; systems
  that "paper over the distinction between local and remote objects … fail to support basic
  requirements of robustness and reliability". Interfaces must reflect locality: remote objects
  need different failure semantics.

**Position.** Three normative points. (1) **Failure detection can never be `Exact`** (RT5): FLP
makes crashed-vs-slow formally indistinguishable in the asynchronous model, so a liveness
judgment about a runtime unit is intrinsically *suspicion*; φ-accrual is the honest interface —
a confidence value mapping to `Empirical` + `ProbabilityBound` (a bare timeout threshold is
`Declared`); any terminating-coordination claim is `Proven` only relative to surfaced
environment assumptions (partial synchrony / detector class), else honestly downgraded.
(2) **`reclaim` = supervision as reified policy** (RT7): restart strategy, link set, and
escalation path are inspectable values you can EXPLAIN, never hidden runtime behavior — the
OTP discipline in Mycelium house style. (3) **Waldo is the never-silent rule applied to
distribution** (RT4): a remote interaction must be typed as fallible, never given a local
call's signature — distribution transparency is precisely the "silent swap" Mycelium forbids.

### T4.6 — Mode switching (→ `dimorph`; RT2/S1)

- **Deoptimization as semantics-preserving mode switch.** Hölzle, Chambers, Ungar (PLDI 1992):
  SELF transparently converts activations of optimized code back to unoptimized form on demand —
  the ancestor of JVM/V8 tier-up/deopt (optimized code runs under recorded assumptions, falls
  back when they break).
- **The obligation made formal.** Flückiger et al., *Correctness of Speculative Optimizations
  with Dynamic Deoptimization* (POPL 2018): assumptions and deopt points reified in the IR;
  deoptimization = finding a semantically equivalent fragment not relying on invalid assumptions
  plus a verified state translation. Barrière et al., *Formally Verified Speculation and
  Deoptimization in a JIT Compiler* (POPL 2021, CoreJIT, Coq) — machine-checked proof that tier
  transitions (incl. on-stack replacement) preserve semantics.

**Position.** `dimorph` is the JIT tier-transition problem with assumptions made first-class —
Mycelium's house style anyway. The 1992→2018→2021 arc shows the equivalence obligation of a mode
switch is carryable at every lattice rung: `Empirical` (industrial deopt, tested), `Proven`
(CoreJIT, machine-checked), with the side-condition structure (reified assumption + state
translation + transfer point) known and checkable. Two normative consequences: (1) interpreted ⇄
native via `matured` is the *speculation-free* special case — the interpreter is the reference
and AOT code for checked-total definitions needs no deopt guard, only the once-per-definition
equivalence obligation (RFC-0004 §3, already built); (2) a dense ⇄ sparse *representation*
switch is a certified `Swap` (S1), never a hidden tier.

## Decisions this pass supports (pointers)

- **RFC-0008 (Draft, 2026-06-10)** — the RT1–RT7 runtime invariants; the
  deterministic-fragment-first posture with sequential reference semantics (T4.1); fusion as
  lawful semilattice merge with meet-composed guarantees (T4.2); placement as the third RFC-0005
  policy site, semantics-free by the Legion separation (T4.3); checkpoints as content-addressed
  values gated on the deterministic/total fragment (T4.4); explicit partial failure and
  suspicion-not-certainty failure detection (T4.5); `dimorph` = RFC-0004 `ExecutionMode` tiering
  or certified `Swap`, nothing else (T4.6).
- The **Runtime vocabulary's T-map test** (DN-02) can now actually be run — RFC-0008 §4.5 maps
  each term to an operational meaning; ratification of names (incl. ADR-012 §7.6 short-form
  refinements) is DN-03's.
- The `spore` reconciliation (ADR-012 §7.4) gains its shape: manifest = one digest-referenced
  component of a content-addressed deployable DAG (T4.4) — recorded for the RFC-0003 revision.

## Key sources

(Representative; per-finding citations inline above.) Erlang/OTP Efficiency Guide + *The BEAM
Book*; Sivaramakrishnan et al. PLDI 2021; Leijen TyDe 2017; Sústrik libdill docs; N. J. Smith
*Notes on structured concurrency* 2018; JEP 444/505/525; Kahn IFIP 1974; Marlow et al. Haskell
Symposium 2011; Bocchino et al. OOPSLA 2009; Kuper & Newton FHPC '13 (LVars); Kumar/Owens et al.
POPL 2014 / ESOP 2016 (CakeML clocked semantics); Shapiro et al. SSS 2011 + INRIA RR-7506
(CRDTs); Honda et al. ESOP 1998 / POPL 2008 (session types); Demers et al. PODC 1987; Leitão et
al. DSN/SRDS 2007 (HyParView/Plumtree); libp2p gossipsub; Armstrong thesis 2003; FLP JACM 1985;
Chandra & Toueg JACM 1996; Hayashibara et al. SRDS 2004 (φ-accrual); Waldo et al. SMLI TR-94-29;
unison-lang.org; Epstein et al. Haskell Symposium 2011; Erlang "Distributed Erlang" docs; Bauer
et al. SC 2012 (Legion); Murray et al. SOSP 2013 (Naiad); Moritz et al. OSDI 2018 (Ray);
Reactive Streams 1.0.4; Blumofe & Leiserson JACM 1999; criu.org; docs.temporal.io; MS Learn
(Durable Functions); NixOS RFC 0062; OCI image-spec; wasi.dev roadmap; Clark et al. NSDI 2005;
Hölzle et al. PLDI 1992; Flückiger et al. POPL 2018; Barrière et al. POPL 2021 (CoreJIT).

## Honest-uncertainty register

- **Kahn 1974 primary text** unavailable during the pass (mirrors 503); the determinism
  statement (continuity, least fixed point, blocking-read/no-peek side conditions) verified via
  secondary sources only — re-verify exact phrasing before quoting it normatively.
- **Sústrik's original post** (250bpm.com/blog:71) is dead; the 2016 coining date rests on
  secondary sources; definitional content verified against live libdill docs.
- **Go stack-size history** (1.3 contiguous stacks, 1.4 default change) partly from memory; the
  2 KB current minimum is corroborated.
- **BEAM's ~4000-reduction slice** is an implementation constant from *The BEAM Book*, not
  normative OTP documentation — illustrative only.
- **JEP statuses** verified via inside.java/InfoQ (openjdk.org 403'd the fetcher); structured
  concurrency **not final** as of 2026-06; re-check after the JDK 27 cycle.
- **LVars proof scope**: FHPC '13 covers λLVar (monotone writes + threshold reads); the
  freeze/handler extension is only *quasi*-deterministic (POPL 2014, cited from memory, not
  re-verified). Par-monad determinism is by construction, not mechanized — tag claims resting on
  it `Empirical`.
- **Sprite, Erlang hot code loading, TCP flow control** cited from background, not re-fetched.
- **CRDT sufficiency, not necessity**: SSS 2011 proves the semilattice/causal-delivery
  conditions *sufficient* for SEC — necessity is not claimed; never write "converges iff".
  RR-7506 (the proofs report) was unreachable this pass (cited via the SSS paper's own
  bibliography); the Isabelle/HOL mechanization (Gomes et al., OOPSLA 2017) was verified at the
  metadata level only.
- **Session-type guarantee statements** (safety/fidelity wording) come from secondary sources
  (PLS Lab, surveys), not extracted from the primary PDFs.
- **gossipsub v1.1 hardening** verified from the spec text; the empirical evaluation of its
  effectiveness was not. HyParView's quantitative resilience figures not extracted from the body.
- **◇W weakest-failure-detector** is the *separate* Chandra–Hadzilacos–Toueg JACM 1996 paper —
  not verified this pass; cite separately if used. FLP escapes (Ben-Or randomization;
  Dwork–Lynch–Stockmeyer partial synchrony) stated from background, unverified.
- **φ-accrual formula** confirmed from Akka's production docs, not the SRDS PDF itself. Page
  numbers for the Waldo quotes were not recorded during extraction.
- **Nix CA-derivations** experimental; **WASI 1.0 in 2026** is roadmap-`Declared`, not fact;
  **Wanco/EdgeSys** are SRC/workshop tier — existence proofs, not performance claims.
- **Serializable continuations**: content-addressing a checkpoint's continuation presupposes a
  defunctionalized/serializable continuation representation — a *design obligation* for the
  RFC-0008 implementation stages, provided by no cited system at the value level except Unison
  (whose serialized-continuation stability across runtime versions is itself unverified).
- **No found precedent** (novelty flags, absence-of-evidence): totality/determinism gating
  *checkpointability*; learned placement as a reified inspectable policy; per-value guarantee
  tags crossing a distribution boundary. Treat all three as novel contributions needing their
  own soundness arguments, not citations.
