# Per-repo scorecard — decomposition alignment assessment 2026-07-18

Columns: seed commit count · three-way pin match (local HEAD == origin/main == components.lock
pin, plus live remote tip at assessment time) · parity classes
(identical-expected/identical-elsewhere/modified/new-scaffold/new-unexplained/missing) ·
cross-repo path-dep count · workspace root present · CROSS-REF.md · docs/ · CI workflow.
All rows `Exact` except the live-tip column (`Empirical`, 2026-07-18). Full row data:
`parity/` + generator scripts.

| Repo | Seed=1 | 3-way pin | Parity (ie/el/mod/ns/nu/miss) | Path deps | Workspace root | CROSS-REF | docs/ | CI |
|---|---|---|---|---|---|---|---|---|
| mycelium-bench | 1 | yes | 22/1/1/1/0/0 | 6 | — | no | no | ci.yml |
| mycelium-build | 1 | yes | 4/1/1/1/0/0 | 1 | — | no | no | ci.yml |
| mycelium-check | 1 | yes | 4/1/1/1/0/0 | 4 | — | no | no | ci.yml |
| mycelium-cli | 1 | yes | 19/1/1/1/0/0 | 5 | — | no | no | ci.yml |
| mycelium-cli-common | 1 | yes | 2/1/1/1/0/0 | 0 | — | no | no | ci.yml |
| mycelium-cli-myc | 1 | yes | 1/0/1/1/10/0 | 0 | — | no | no | ci.yml |
| mycelium-codegen | 1 | yes | 100/0/1/1/0/0 | 13 | — | no | no | ci.yml |
| mycelium-core | 1 | yes | 59/0/1/1/0/0 | 1 | — | no | no | ci.yml |
| mycelium-doc | 1 | yes | 34/1/1/1/0/0 | 4 | — | no | no | ci.yml |
| mycelium-fmt | 1 | yes | 7/1/1/1/0/0 | 4 | — | no | no | ci.yml |
| mycelium-l1 | 1 | yes | 119/1/1/1/0/0 | 9 | — | no | no | ci.yml |
| mycelium-lang | 1 | n/a-lang | 0/1/0/3/0/0 | 0 | — | no | no | ci.yml |
| mycelium-lint | 1 | yes | 3/1/1/1/0/0 | 4 | — | no | no | ci.yml |
| mycelium-lsp | 1 | yes | 38/1/1/1/0/0 | 7 | — | no | no | ci.yml |
| mycelium-proj | 1 | yes | 16/1/1/1/0/0 | 2 | — | no | no | ci.yml |
| mycelium-runtime | 1 | yes | 47/0/1/1/0/0 | 15 | — | no | no | ci.yml |
| mycelium-sec | 1 | yes | 5/1/1/1/0/0 | 1 | — | no | no | ci.yml |
| mycelium-spore | 1 | yes | 9/1/1/1/0/0 | 2 | — | no | no | ci.yml |
| mycelium-std-cmp | 1 | yes | 2/1/1/1/0/0 | 2 | — | no | no | ci.yml |
| mycelium-std-collections | 1 | yes | 7/1/1/1/0/0 | 2 | — | no | no | ci.yml |
| mycelium-std-conformance | 1 | yes | 18/1/0/2/0/0 | 17 | — | no | no | ci.yml |
| mycelium-std-content | 1 | yes | 6/1/1/1/0/0 | 1 | — | no | no | ci.yml |
| mycelium-std-core | 1 | yes | 4/1/1/1/0/0 | 1 | — | no | no | ci.yml |
| mycelium-std-dense | 1 | yes | 2/1/1/1/0/0 | 2 | — | no | no | ci.yml |
| mycelium-std-diag | 1 | yes | 3/1/1/1/0/0 | 1 | — | no | no | ci.yml |
| mycelium-std-error | 1 | yes | 4/1/1/1/0/0 | 3 | — | no | no | ci.yml |
| mycelium-std-fmt | 1 | yes | 3/1/1/1/0/0 | 3 | — | no | no | ci.yml |
| mycelium-std-fs | 1 | yes | 8/1/1/1/0/0 | 2 | — | no | no | ci.yml |
| mycelium-std-io | 1 | yes | 8/1/1/1/0/0 | 2 | — | no | no | ci.yml |
| mycelium-std-iter | 1 | yes | 8/1/1/1/0/0 | 2 | — | no | no | ci.yml |
| mycelium-std-math | 1 | yes | 5/1/1/1/0/0 | 4 | — | no | no | ci.yml |
| mycelium-std-numerics | 1 | yes | 3/1/1/1/0/0 | 3 | — | no | no | ci.yml |
| mycelium-std-rand | 1 | yes | 2/1/1/1/0/0 | 2 | — | no | no | ci.yml |
| mycelium-std-recover | 1 | yes | 10/1/1/1/0/0 | 4 | — | no | no | ci.yml |
| mycelium-std-runtime | 1 | yes | 26/1/1/1/0/0 | 5 | — | no | no | ci.yml |
| mycelium-std-select | 1 | yes | 2/1/1/1/0/0 | 2 | — | no | no | ci.yml |
| mycelium-std-spore | 1 | yes | 6/1/1/1/0/0 | 8 | — | no | no | ci.yml |
| mycelium-std-swap | 1 | yes | 2/1/1/1/0/0 | 3 | — | no | no | ci.yml |
| mycelium-std-sys | 1 | yes | 11/1/1/1/0/0 | 0 | — | no | no | ci.yml |
| mycelium-std-sys-host | 1 | yes | 2/1/1/1/0/0 | 3 | — | no | no | ci.yml |
| mycelium-std-ternary | 1 | yes | 6/1/1/1/0/0 | 1 | — | no | no | ci.yml |
| mycelium-std-testing | 1 | yes | 10/1/1/1/0/0 | 4 | — | no | no | ci.yml |
| mycelium-std-text | 1 | yes | 6/1/1/1/0/0 | 2 | — | no | no | ci.yml |
| mycelium-std-time | 1 | yes | 2/1/1/1/0/0 | 2 | — | no | no | ci.yml |
| mycelium-std-vsa | 1 | yes | 10/1/1/1/0/0 | 2 | — | no | no | ci.yml |
| mycelium-transpile | 1 | yes | 55/1/1/1/0/0 | 2 | — | no | no | ci.yml |
| mycelium-value | 1 | yes | 48/0/1/1/0/0 | 8 | — | no | no | ci.yml |

Notes: mycelium-lang has no self-pin in its own lock (mirror carries one) — its pin column
reads n/a. mycelium-cli-myc parity: 10 new-unexplained files (the superset exception, F14).
