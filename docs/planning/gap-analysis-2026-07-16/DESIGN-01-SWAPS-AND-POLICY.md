# Design pack 01 — Swaps & policy ergonomics

| Field | Value |
|---|---|
| **Status** | **Draft** design package — not Accepted · not implement |
| **Pack** | 1 of 3 · with [02 Tags & containment](./DESIGN-02-TAGS-META-AND-CONTAINMENT.md) · [03 Machinery, diagnostics & UX](./DESIGN-03-MACHINERY-DIAGNOSTICS-AND-UX.md) |
| **Honesty** | Design positions `Declared` until ratified |
| **Sources distilled** | Agent A · council · `stdlib/swap.md` Q2 · RFC-0001/0002 · DN-29 |

## 1. Why this document exists

Mycelium’s signature operation is the **never-silent `swap`**: every representation change is
lexical, certified, and auditable. That power currently costs **call-site ceremony** (policy
threading, cert packaging, fallibility under-typed) and can make **failures hard to localize**.

This pack answers: *how do we keep S1/G2 honesty while making swaps usable every day?*

## 2. Mental model

```mermaid
flowchart LR
  V1["Value @ Repr A"] --> S["swap · written"]
  S --> P["PolicyRef\nresolve + record"]
  S --> C["Certificate\nemit / check by mode"]
  S --> V2["Value @ Repr B"]
  P --> X["EXPLAIN"]
  C --> X
  C --> F{"check OK?"}
  F -->|yes| V2
  F -->|no| R["Option / Result / NotValidated\nnever looks Exact"]
```

| Piece | Role | Must stay explicit? |
|---|---|---|
| **`swap` keyword / `std.swap.*`** | Marks Repr change | **Yes forever** (S1) |
| **`to:` target** | Destination Repr | Prefer written; optional elision only if unique expected type |
| **`policy:`** | Selection rule identity | **Yes as identity** — may elide *spelling* if resolved hash is recorded |
| **Certificate** | Audit of the change | Mode-gated emit/check; must remain **queryable** on failure |
| **Result shape** | Total vs partial | Typed by **regime** — do not type partial as total |

## 3. Pain (author-facing)

| ID | Pain | Wanted outcome |
|---|---|---|
| **P1** | Every site needs verbose `to:` + `policy:` | Short path for common pairs; still EXPLAIN-able |
| **P2** | `Swapped { value, cert }` forces threading | Value-forward default; cert still inspectable |
| **P3** | Policy authoring is a second subsystem | **Catalog** of content-addressed policies |
| **P4** | Legal pairs known late / at runtime | Static matrix where possible |
| **P5** | Tutorial types total when pair is partial | Regime → `Option`/`Result` |
| **P6** | Failed check / dig for “which swap?” | First-fault diagnostic (see pack 03) |
| **P7** | `fast` drops cert check but not syntax tax | Modes gate *machinery*, not honesty of writing swap |

**Hard rejects:** auto-insert `swap`; omit policy with **no** recorded identity; treat `NotValidated` as success.

## 4. Recommended package (Draft)

### 4.1 Policy streamline (first-class — maintainer priority)

| Step | Mechanism | Effect |
|---|---|---|
| **A1** | **Legal-pair matrix** in checker (RFC-0002 data) | Early refuse illegal pairs |
| **A2** | **`std.swap.policy` catalog** — content-addressed defaults | Authors pick by name/intent, not reinvent tables |
| **A3** | **`policy: default` (or `_`)** → resolve → record `PolicyRef` + EXPLAIN origin | Cuts P1/P3 without black-box policy |
| **A4** | Optional nodule/phylum **ambient policy** for *written* swaps only | Same pattern as ambient paradigm (RFC-0012) |

```mermaid
sequenceDiagram
  participant Author
  participant Surface
  participant Catalog
  participant EXPLAIN
  Author->>Surface: swap(x, to: T, policy: default)
  Surface->>Catalog: resolve default for pair
  Catalog-->>Surface: PolicyRef hash
  Surface->>EXPLAIN: pair, policy hash, origin
  Surface-->>Author: value (+ cert handle by mode)
```

**Rule:** elision is *spelling* only. If resolution fails → **hard error**, never silent fallback policy.

### 4.2 Typing & cert packaging

| Step | Mechanism | Effect |
|---|---|---|
| **A5** | **Regime → result type** (total / Option / Result) | Honest fallibility |
| **A6** | **Cert ambient** (value-forward; cert queryable) *or* keep explicit `Swapped` until failure is always materializable | Cuts P2 without hiding fail |
| **A7** | Named std ops desugar to keyword `swap` | One type story |

**Joint gate with packs 02/03:** if cert is ambient, **failed check must still surface** as typed failure + first-fault event — never Exact success.

### 4.3 Localize swap failures

Attachment points for diagnostics (full design in pack 03): policy resolve, pair legality, cert emit, cert check, out-of-range. Each emits a **first-fault** record with source span + why.

## 5. Ranked options (summary)

| Rank | Option | Verdict |
|---:|---|---|
| **1** | Catalog + default policy + legal matrix + regime types | **Recommend** |
| **2** | Cert ambient after failure materializability lands | Follow-on |
| **3** | `to:` elision + named-op sugar | Follow-on |
| **4** | Tooling candidates only (LSP insert-swap) | Parallel, never auto-apply |
| **REJECT** | Auto-swap · policy-less form · greenwash metrics | Disqualified |

## 6. Open questions for you

1. Spelling: `policy: default` vs `_` vs catalog name only?
2. Cert ambient as default authoring model, or keep explicit `Swapped` until the diagnostic bus lands?
3. Allow `to:` elision under unique expected type?
4. Vehicle: extend M-540 vs dedicated Swap Ergonomics DN after steer?

## 7. DoD before implement waves

- [ ] Maintainer steers §6
- [ ] Normative capture (DN/RFC amend) for A1–A3
- [ ] Conformance: expand(`policy: default`) ≡ longhand with same hash
- [ ] Fail paths never type as Exact under certified mode

## 8. See also

- Pack [02](./DESIGN-02-TAGS-META-AND-CONTAINMENT.md) — grades, meet, seals
- Pack [03](./DESIGN-03-MACHINERY-DIAGNOSTICS-AND-UX.md) — AX ranks, emitters, UX backlog
