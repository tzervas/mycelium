# Design Note DN-98 — Specialist Agent Personas (reusable `.claude/agents/` roles that pre-scope model tier, tools, skills, and discipline for the swarm workflow)

| Field | Value |
|---|---|
| **Note** | DN-98 |
| **Status** | **Draft** (2026-07-09) — proposes a **specialist-agent-persona system** for the Mycelium swarm workflow: reusable `.claude/agents/*.md` definitions that pre-scope **model tier + tool set + skills + discipline** per recurring role, so spawning a swarm agent is "invoke the persona + a task delta" rather than re-writing a full brief. **Enacts nothing** and **moves no other doc's status** (house rule #3, append-only). Requires maintainer ratification to move Draft to Accepted. All tags `Declared` unless a cited source holds them higher (VR-5). |
| **Decides** | *Proposes, for ratification:* (a) a catalog of **six specialist personas** — `myc-porter`, `myc-leaf`, `pr-reviewer`, `security-reviewer`, `design-reasoner`, `integrator` — each pinning model tier, a least-privilege tool set, the skills it drives, and its role loop; (b) the **swarm-mode composition + precedence** (mode fixes the model; a persona's frontmatter model is the out-of-swarm default; one persona carries a hard Opus floor); (c) the **orchestrator invocation** convention (`subagent_type: <persona>` + a short task delta); (d) `.claude/agents/` as a new, deliberately-small registry alongside `.claude/skills/`. Goal (the maintainer's explicit ask): **token/usage efficiency and better-optimized swarms.** |
| **Consumes** | `CLAUDE.md` §Swarm modes, §Fractal Swarm Development System (Role prompt blocks), §Concurrent-PR development, §Ratified branch/merge/propagation workflow (DN-97), the Skills list; the mitigations (#1/#9/#11/#12/#14). The Claude Code **sub-agents** feature (`.claude/agents/*.md`). |
| **Feeds** | `.claude/agents/*.md` (the six persona files, landed with this note); on ratification, `CLAUDE.md` §Skills/§Swarm wiring + a `CHANGELOG` + `Doc-Index` row (FLAGged, not edited here). |
| **Date** | July 9, 2026 |
| **Task** | agent-personas — design + implement the specialist-persona system. |

> **Posture (transparency rule / VR-5 / G2 / house rule #4).** This note **works up a decision** and
> ships the persona files as **Draft** operational material; it does **not** ratify the system (house
> rule #3 — a maintainer does). **One load-bearing fact is `Declared`, not independently reproduced:**
> the Claude Code docs state that **custom subagents load `CLAUDE.md`** (unlike the built-in Explore/Plan
> agents) — relayed from `code.claude.com/docs/en/sub-agents.md` via the `claude-code-guide` agent, not
> verified by spawning one and observing. The design is built to be **correct either way**: each persona
> body also carries a compact "non-negotiables" paragraph, so the house rules bind **whether or not** the
> auto-load holds (§7). The frontmatter schema (§2) is `Declared`-from-docs. The persona catalog and the
> token-efficiency argument are `Declared`-with-argument. **No sycophancy:** §7 states the real
> trade-offs — persona drift against `CLAUDE.md`/the skills, a second registry to maintain, and the fact
> that tool-scoping bounds accidental edits, not all writes — plainly, and §9 says which roles are
> deliberately **not** personas and why.

---

## §1 The system in one picture

```
   orchestrator (any tier)                          .claude/agents/  (the persona registry)
   ─────────────────────────                        ──────────────────────────────────────
   Agent(                                            myc-porter.md       (Sonnet, port loop)
     subagent_type: "myc-leaf",   ◄── names a ROLE   myc-leaf.md         (Sonnet/Haiku, impl leaf)
     model: <mode-resolved>,      ◄── mode fixes it   pr-reviewer.md      (Sonnet, read-only review)
     isolation: "worktree",       ◄── spawn-time      security-reviewer.md(Sonnet, read-only)
     prompt: "<1–2 line delta>")  ◄── the ONLY bespoke design-reasoner.md   (Opus, hard floor)
                                       text per spawn integrator.md       (Sonnet/Opus, full tools)
                                                         │
   each persona file = frontmatter (name·description·    └─ body = the compact system prompt:
   model·tools) + a compact system-prompt body             role loop · skills it drives ·
                                                            non-negotiables · report format
```

A **persona** is a `.claude/agents/*.md` file: YAML frontmatter (`name`, `description`, `model`, `tools`)
plus a Markdown body that is the agent's **system prompt**. The orchestrator spawns it via the Agent tool
with `subagent_type: <name>` and adds only a **task-specific delta** — the fixed role scaffolding
(discipline, skills, tool scope, house-rule reminders) lives in the file, authored once. That is the
token-efficiency win (§6).

---

## §2 The confirmed `.claude/agents/` schema (verify-first — mitigation #14)

Verified against the Claude Code sub-agents docs (`code.claude.com/docs/en/sub-agents.md`) before
authoring, via the `claude-code-guide` agent. `Declared`-from-docs.

- **Location.** `.claude/agents/*.md` (project-level, checked in) or `~/.claude/agents/` (user-level);
  scanned recursively. Identity is the frontmatter `name`, not the filename (we match them anyway).
- **Required frontmatter:** `name` (lowercase + hyphens), `description` (the when-to-use / auto-delegation
  trigger).
- **Optional frontmatter we use:** `model` (`sonnet` / `opus` / `haiku` / `fable` / a full ID / `inherit`;
  omit ⇒ `inherit`), `tools` (comma-separated allowlist; **omit ⇒ inherit all**; an allowlist is
  default-deny, so MCP servers like `mcp__github` must be listed explicitly to be granted).
- **Optional frontmatter the docs also confirm but we deliberately DO NOT use** (portability — §7.5):
  `disallowedTools`, `skills`, `isolation`, `permissionMode`, `color`, `maxTurns`, `mcpServers`, `hooks`,
  `memory`, `background`, `effort`, `initialPrompt`. We hold to the four core fields (`name`,
  `description`, `model`, `tools`) that are universally supported and were explicitly blessed for this
  task; the richer fields are documented here as an opt-in the maintainer may adopt later.
- **Body = system prompt.** Everything after the frontmatter is the agent's system prompt.
- **CLAUDE.md loading (`Declared`, load-bearing — see Posture):** the docs state **custom subagents load
  `CLAUDE.md`** (Explore/Plan do not). This lets persona bodies **reference** the house rules rather than
  re-paste them. We do **not** rely on it for correctness (§7.1).

Consequences we design around: skills are driven **from the body** (an agent invokes `/skill`), not via a
`skills:` frontmatter key; worktree isolation is passed **at spawn time** (`isolation: "worktree"`), not
via frontmatter — both the robust, version-independent path.

---

## §3 The persona catalog

Six personas, each mapping a **genuinely recurring** role this workflow exercises. Quality over quantity:
a persona earns its place only by removing real per-spawn brief-writing overhead **and** adding
specialization the generic `CLAUDE.md` Role blocks (§9) don't.

| Persona | Model (default) | Tools | Role (one line) |
|---|---|---|---|
| **`myc-porter`** | Sonnet | R/G/G, Edit, Write, Bash, Skill | The M-993 differential-witnessed `.myc` porter — triage + gap-profile, live-oracle differential, `Empirical`-tagged emissions, graduate into `lib/` only with a witness. |
| **`myc-leaf`** | Sonnet (Haiku under Haiku/Hybrid) | R/G/G, Edit, Write, Bash, Skill | The change-scoped implementation leaf — one disjoint crate/dir, `/dev-workflow`, tests-first, honest tags, change-scoped gates, FLAG shared files up. |
| **`pr-reviewer`** | Sonnet | R/G/G, Bash, Skill (**read-only**) | The `/pr-review` specialist — transparency/grounding/append-only/hallucination lens; returns severity-ranked findings; no patch, no merge. |
| **`security-reviewer`** | Sonnet | R/G/G, Bash, Skill (**read-only**) | The `/security-review` specialist — secrets, supply-chain, shell/CI safety, input handling; returns severity-ranked findings. |
| **`design-reasoner`** | Opus (**hard floor**) | R/G/G, Edit, Write, Bash, Skill | The DN/ADR design-evaluation reasoner — enumerate, evaluate, recommend-**not**-ratify, adversarial stress-test; authors a Draft DN/ADR only. |
| **`integrator`** | Sonnet (Opus preferred) | full set + `mcp__github`/`mcp__tero` | The integration-tier close-out — reconcile the shared surface once, regen indices (`/doc-index` + `/tero-refresh`), full `just check`, close-out, `/sync-down`. |

(R/G/G = `Read, Grep, Glob`.) Full loops, skills, and report formats are in each file's body.

**Tool-scoping rationale (least-privilege where it is real).** The two **reviewers** hold **no
`Edit`/`Write`/`Agent`/MCP-write** — a reviewer cannot accidentally mutate the tree; it returns findings.
The **leaves** (`myc-porter`, `myc-leaf`) hold `Edit`/`Write` but **no `Agent`** — a leaf is terminal and
cannot spawn sub-agents, enforcing the fractal invariant by construction. The **integrator** alone holds
the full set (it owns the shared files, regenerates indices, and drives protected-branch PR merges) — an
honest exception, not a claim we scope everything (§7.4).

---

## §4 Swarm-mode composition and precedence

A persona names a **role**; the active swarm mode (`CLAUDE.md` §Swarm modes: Sonnet / Haiku / Opus /
Hybrid) fixes the **model**. They compose by three rules:

- **SM-1 — the mode fixes the model.** In an active swarm, the orchestrator passes the mode-resolved
  model **explicitly** on each spawn (the Agent tool's per-invocation `model`), which **overrides** the
  persona's frontmatter `model` (Claude Code resolution order: per-invocation > frontmatter > inherit).
  This honors §Swarm modes' "pass the resolved model explicitly — never substitute silently." So
  `myc-leaf` runs as Haiku in a Hybrid/Haiku swarm, Sonnet in a Sonnet swarm, Opus in an Opus swarm —
  the persona does not fight the mode.
- **SM-2 — a documented hard floor escalates, never-silently.** A role that is ineffective below a tier
  carries a **hard model floor** in its body. **`design-reasoner` floors at Opus.** If the active mode
  resolves lower, the spawning parent **escalates that one spawn to the floor and says so** (G2 —
  never-silent). This is an orchestrator **discipline** (there is no "min-model" frontmatter field), made
  visible so a mis-set model is caught. `design-reasoner` is the **only** hard-floor persona; `integrator`
  carries a **soft** preference (Opus for a heavy batch) — a recommendation, not an escalation.
- **SM-3 — the frontmatter model is the out-of-swarm default.** For a one-off spawn with no mode set, the
  frontmatter `model` applies, so a bare `subagent_type: pr-reviewer` gets Sonnet with **zero** extra
  words. This is the token-efficiency win for ad-hoc use.

**Precedence (highest wins):** per-invocation model (mode, or the SM-2 floor escalation) > persona
frontmatter default > `inherit`.

---

## §5 How the orchestrator invokes a persona

```
Agent(
  subagent_type: "myc-leaf",          # the persona name (frontmatter `name`)
  model: "haiku",                     # SM-1: the active swarm mode's leaf tier (explicit)
  isolation: "worktree",              # mitigation #11: one isolated worktree per concurrent agent
  description: "impl M-1042 dense-pack",
  prompt: "Implement M-1042 in crates/mycelium-std-dense only. Branch \
           claude/leaf/<EPIC>-<LEAF>-dense-pack off the epic tip. Advances SC-2; \
           ship the pack/unpack bound + property test. FLAG the CHANGELOG line up.")
```

Everything fixed by the role — the `/dev-workflow` loop, honest tags, change-scoped gates, worktree
guard, commit/push discipline, the non-negotiables — is **in the persona file**. The `prompt` carries
**only** the task delta (issue id, the disjoint directory, the branch, the `FR/NFR/VR/SC` advanced, the
FLAG). For a review, the delta is just the PR/branch ref: `subagent_type: "pr-reviewer", prompt: "Review
PR #1103 against dev."`

---

## §6 Token-efficiency rationale (the maintainer's explicit ask)

The objective is **token/usage efficiency and better-optimized swarms**. The system delivers it three
ways:

1. **The per-spawn brief shrinks from a role-block to a task delta.** Today an orchestrator pastes a full
   brief per spawn (role + model + tools + skills + discipline + house-rule reminders) — hundreds of
   tokens, re-authored each time and drift-prone. With a persona it writes `subagent_type: <name>` + one
   or two lines. The fixed scaffolding is amortized into a file authored once.
2. **The house rules are not paid for in the brief.** Because custom subagents load `CLAUDE.md`
   (`Declared`, §2), and because the skills carry their own discipline, the persona body **references**
   the rules (by number) rather than re-pasting them — so neither the orchestrator's brief nor the
   persona file re-encodes the rule text.
3. **Least-privilege is by construction, not by reminder.** The orchestrator no longer needs to tell a
   reviewer "don't edit" or a leaf "don't spawn" — the `tools` allowlist enforces it, removing that text
   from every brief and closing the failure mode structurally.

The scaffolding cost moves from **O(spawns)** (re-written every time) to **O(personas)** (six files,
authored once). In a wave of 20–30 spawns that is the dominant saving.

---

## §7 Honest trade-offs (VR-5)

- **§7.1 Persona drift vs `CLAUDE.md` + the skills — the single biggest trade-off.** The persona bodies
  duplicate a thin slice of role discipline that also lives in `CLAUDE.md` and the skills. If the house
  rules evolve, the bodies can lag → two sources of truth. **Mitigation:** personas **reference**
  `CLAUDE.md` as authoritative and **name** skills rather than re-pasting their content, so the drift
  surface is the thin role-specific delta, not the rule set. **Residual risk remains** and is a real
  maintenance cost — this is the trade-off to weigh at ratification. (It is *also* why the compact
  non-negotiables paragraph names rules by **number** with a one-clause reminder: short enough to keep
  current, explicit enough to bind if the auto-load fails.)
- **§7.2 Another registry to maintain.** `.claude/agents/` joins `.claude/skills/` as a place that must
  stay current. **Mitigation (YAGNI):** only six personas, each earning its place; the structural
  orchestrator/epic roles are **deliberately not** personas (§9).
- **§7.3 The model-precedence subtlety.** SM-1/SM-2 is a discipline, not a mechanism. An orchestrator that
  forgets SM-2 could run `design-reasoner` as Haiku in a Hybrid swarm, defeating its purpose.
  **Mitigation:** the floor is stated in the persona body **and** here; the never-silent escalation makes
  a mis-set model visible.
- **§7.4 Tool-scoping is not sandboxing.** A read-only reviewer still holds `Bash` (needed for `git
  diff`), so "read-only" bounds **accidental `Edit`/`Write`**, not all writes — a determined `Bash`
  command could still mutate. **Mitigation / honest scope:** the branch-guard PreToolUse hook still blocks
  protected-branch git ops; true confinement would need `permissionMode`/hooks, deferred. The win is real
  but **partial** — we do not claim more.
- **§7.5 Frontmatter portability.** We use only the four core fields; the richer confirmed fields
  (`skills`/`isolation`/`disallowedTools`/`permissionMode`/`color`) are documented (§2) but unused, to
  stay portable across Claude Code versions and to honor the verify-first "don't rely on unconfirmed-in-
  this-version fields." **Cost:** skills are driven from the body and isolation is passed at spawn time,
  slightly more verbose than a `skills:`/`isolation:` frontmatter would be. A maintainer who pins a known
  version may opt into the richer fields.

---

## §8 Adversarial stress-test (VR-5 / house rule #4)

- **S1 — a Hybrid swarm spawns `design-reasoner`.** Mode says Haiku (leaf tier); SM-2 floors it at Opus;
  the orchestrator escalates and says so. **Survives** (the floor is the reason SM-2 exists).
- **S2 — `CLAUDE.md` does not auto-load for a custom subagent** (the `Declared` fact is wrong). The
  persona body's non-negotiables paragraph still binds the agent to rules #1–#6/#9/#11/#12/#14. **Survives
  — the design is correct either way** (§7.1); only the token saving of reference-vs-paste is reduced.
- **S3 — a reviewer is asked to fix what it found.** It has no `Edit`/`Write`; it returns the fix for a
  patcher (the `/pr-land` split). **Survives** — least-privilege holds; no silent self-patch.
- **S4 — a leaf tries to spawn sub-agents.** No `Agent` tool → it cannot; it FLAGs the need up. **Survives
  — the fractal "leaves are terminal" invariant is structural.**
- **S5 — the persona set drifts from `CLAUDE.md`.** Caught by review (the bodies point at `CLAUDE.md` as
  authoritative, so a reviewer reconciles the delta), and bounded because the bodies reference rather than
  copy. **Survives, with the residual maintenance cost named (§7.1).**
- **S6 — the integrator lacks `mcp__tero` in-session.** Its body falls back (`just tero-index` for the
  committed index; `gh` CLI for PR merges). **Survives** — the MCP is a convenience, not a dependency.

**Honest verdict:** the system survives every sequence; its correctness does **not** hinge on the one
`Declared` fact (S2), and its one genuine ongoing cost is drift (§7.1), mitigated but not eliminated.

---

## §9 What is deliberately NOT a persona (and why)

The **structural hierarchy roles** — Orchestrator, Epic Agent, Leaf Agent — already have **Role prompt
blocks** in `CLAUDE.md` §Fractal Swarm. A generic `orchestrator`/`epic-agent` persona would **duplicate**
those blocks without adding specialization → pure drift surface, no new capability (fails the §7.2 bar).
So personas are added exactly where `CLAUDE.md` **lacks** a Role block — the **specialist** roles
(porter, reviewer, reasoner, integrator). The two specialist **leaf** personas (`myc-porter`, `myc-leaf`)
are the one overlap: they **compose** the generic Leaf Agent Role (inherited via `CLAUDE.md`) **plus**
their domain loop — additive value, not duplication. If the maintainer later finds the Epic Role block is
re-pasted often enough to hurt, an `epic-agent` persona is a clean follow-on; this note defers it (YAGNI)
rather than pay the drift cost now.

---

## §10 User stories

- **As the orchestrator of a swarm,** I want to spawn a role by name plus a one-line delta instead of
  re-writing a full brief each time, so I spend my scarce context budget on the work, not the scaffolding.
- **As the maintainer,** I want swarm agents pre-scoped to least-privilege tools (a reviewer that cannot
  edit, a leaf that cannot spawn), so a whole class of mis-scoped-agent failures cannot happen.
- **As a swarm agent,** I want my model tier fixed by the active mode (with a documented, never-silent
  floor where the role needs it), so I run at the right cost/capability without per-spawn negotiation.
- **As a reviewer,** I want the `/pr-review` or `/security-review` lens and its rubric baked into my
  persona, so every review I do is consistent and I return severity-ranked findings, not ad-hoc prose.
- **As the maintainer,** I want the persona system's trade-offs (drift, a second registry, partial
  scoping) stated honestly, so I ratify it with eyes open — not because it was proposed.

---

## §11 Definition of Done

**Draft.** **Accepted** when the maintainer (a) approves the six-persona catalog (§3) and the model
composition/precedence (§4), and (b) confirms the four-core-field frontmatter choice (§2/§7.5) or opts
into the richer fields. **Resolved** when, after acceptance:

1. `CLAUDE.md` gains a short **§Agent personas** note (under §Skills or §Swarm) pointing at
   `.claude/agents/` and stating the SM-1/SM-2/SM-3 precedence — maintainer/integrator-owned (**FLAG-1**);
2. `CHANGELOG.md` records the addition (Added — Draft persona system) (**FLAG-2**);
3. `docs/Doc-Index.md` registers the DN-98 row (**FLAG-3**);
4. optionally, the `claude-code-guide`-relayed **"custom subagents load `CLAUDE.md`"** fact is verified
   in-session (spawn a persona, confirm the rules are in its context) and this note's `Declared` tag on it
   is upgraded to `Empirical` (**FLAG-4**).

A maintainer amendment supersedes by dated section — never a rewrite (append-only).

---

## FLAGs (up to the integrating parent — not edited from this leaf)

| FLAG | What it is | Who |
|---|---|---|
| **FLAG-1** | **`CLAUDE.md`** — add a §Agent personas note (registry pointer + SM-1/2/3 precedence), append-only, on ratification. | Maintainer / Integrator |
| **FLAG-2** | **`CHANGELOG.md`** — entry for DN-98 + the six persona files (Added — Draft persona system). | Integrator |
| **FLAG-3** | **`docs/Doc-Index.md`** — register the DN-98 row (table row: `\| **DN-98 — Specialist Agent Personas** (path) \| summary \| **Draft** (2026-07-09; agent-personas) \|`). | Integrator |
| **FLAG-4** | **Verify the `CLAUDE.md`-auto-load fact** in-session and upgrade its tag `Declared → Empirical` (§2 / Posture). | Maintainer / Integrator |
| **FLAG-5** | **DN slot verification.** DN-98 verified free at authoring (2026-07-09 — mitigation #1; DN-97 was highest); integrator re-verifies at merge. | Integrator |

---

## Meta — changelog

- **2026-07-09 — Created (Draft; agent-personas).** Proposes the **specialist-agent-persona system**:
  six `.claude/agents/*.md` personas (`myc-porter`, `myc-leaf`, `pr-reviewer`, `security-reviewer`,
  `design-reasoner`, `integrator`) that pre-scope **model tier + least-privilege tools + skills +
  discipline** per recurring role (§3), the **swarm-mode composition + precedence** (SM-1 mode fixes the
  model; SM-2 a documented never-silent hard floor — `design-reasoner` at Opus; SM-3 the frontmatter
  default for one-off spawns, §4), the **orchestrator invocation** convention (`subagent_type` + task
  delta, §5), and the **token-efficiency rationale** (per-spawn brief shrinks to a delta; scaffolding
  amortized O(spawns)→O(personas), §6). Frontmatter uses only the four **verified** core fields
  (`name`/`description`/`model`/`tools`, §2 — `Declared`-from-docs); richer confirmed fields documented
  but deliberately unused for portability (§7.5). **Honest trade-offs** stated (§7): persona drift vs
  `CLAUDE.md`/skills (**the biggest**, mitigated by reference-not-paste), a second registry (bounded by
  six-only + §9's deliberate non-personas), model-precedence subtlety, and **tool-scoping bounds
  accidental edits, not all writes**. The one **load-bearing `Declared` fact** — custom subagents load
  `CLAUDE.md` — is **not** relied on for correctness (each body carries a compact non-negotiables
  paragraph; S2). **Enacts nothing; requires maintainer ratification to move Draft to Accepted (house
  rule #3).** (VR-5; G2.)
