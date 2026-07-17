# Mycelium discussion template

| Field | Value |
|---|---|
| **Status** | Living process template (not an ADR/RFC/DN) |
| **Use when** | Opening a design/research/product discussion for maintainer + agents |
| **Complements** | ADR/RFC/DN templates · `maint-guide.md` · design pack `DESIGN-01`…`04` |
| **Honesty** | Discussion claims are `Declared` until captured into a Draft DN/RFC/ADR |

Use this as the **shape** of a discussion — in GitHub Discussions, a design council session,
Telegram orch notes, or an agent swarm kickoff. Copy the body below; fill every section or mark
`N/A` with a one-line why.

---

## 0. Header (fill first)

```markdown
### Discussion: <short imperative title>

| Field | Value |
|---|---|
| **Kind** | design · research · process · product · incident · other |
| **Opened** | YYYY-MM-DD |
| **Owner** | @handle / L0 |
| **Participants** | maintainer · L0 · agents (models) |
| **Related** | M-… · RFC-… · ADR-… · DN-… · DESIGN-0N · PR #… |
| **Urgency** | P0 blocking · P1 next wave · P2 backlog · P3 someday |
| **Mode** | explore (no decision) · steer (need choices) · ratify (ready for Draft DN/RFC) |
```

---

## 1. Context (what is true now)

- **Background** (3–8 sentences; link corpus, not re-litigate Enacted RFCs).
- **Checked basis** — what is `Exact` / `Proven` / `Empirical` / `Declared` today?
- **Out of scope** for this discussion (explicit).

---

## 2. Problem / opportunity

- **User-visible pain** (UX/DX) or **system risk** (correctness, growth, safety).
- **Who hurts** (author · operator · porter · runtime · maintainer · agent).
- **What “done” would feel like** (one paragraph, no implementation detail yet).

---

## 3. Non-negotiables (gates)

List house rules and prior decisions that bind this thread (cite):

| Gate | Citation | Implication here |
|---|---|---|
| Never-silent (G2) | … | … |
| No tag upgrade without basis (VR-5) | … | … |
| No silent / auto `swap` (S1) | … | … |
| Append-only decisions | house rule #3 | capture as Draft → Accepted later |
| … | … | … |

If a proposal would violate a gate, mark it **REJECT** or require an explicit supersession path.

---

## 4. Options (ranked)

At least two options + **status quo**. For each:

| # | Option | Pros | Cons / risks | Fits gates? |
|---|---|---|---|---|
| 0 | Status quo | … | … | … |
| 1 | … | … | … | … |
| 2 | … | … | … | … |

**Recommendation** (discussion owner): option N — one paragraph why.

**Adversarial note:** strongest argument *against* the recommendation.

---

## 5. Consequences if we choose the recommendation

- **Authors / DX**
- **Runtime / language internals** (not app-layer concerns unless necessary)
- **Tooling / CI**
- **Docs / corpus capture** (which DN/RFC/ADR would carry it?)
- **What we explicitly will not do**

---

## 6. Open questions for the maintainer

Numbered list. Prefer binary or few-way steers over open essays.

1. …
2. …

---

## 7. Capture & next steps

| If mode is… | Then… |
|---|---|
| **explore** | Park notes; no corpus status change |
| **steer** | Record steers in this thread; open Draft DN/RFC when stable |
| **ratify** | Author Draft DN/RFC/ADR from this template; PR → `dev`; do not skip `Accepted` |

**Follow-on work (Declared until filed):** wave items / M-ids (verify free slots before minting).

---

## 8. Session log (append during discussion)

| When | Who | Note |
|---|---|---|
| … | … | … |

---

## Quick checklist before closing

- [ ] Gates table filled
- [ ] Options include status quo
- [ ] Recommendation + adversarial note
- [ ] Maintainer questions answered or deferred with owner
- [ ] Capture vehicle named (DN/RFC/ADR/none)
- [ ] No silent upgrade of guarantee claims past their basis
