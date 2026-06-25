# Wiki source (`docs/wiki/`)

These markdown files are the **source** for the Mycelium GitHub wiki. They live in the main
repository (so they are versioned, reviewed, and diffable alongside the code) and are **published to
the GitHub wiki** by a manual-dispatch GitHub Action
([`.github/workflows/publish-docs.yml`](../../.github/workflows/publish-docs.yml)).

## Why source-in-repo

The GitHub wiki is a separate git repository (`<repo>.wiki.git`) that cannot be reviewed via pull
requests. Keeping the wiki source here means every wiki change goes through the normal
review/transparency discipline; the Action only mirrors the approved content out.

## Pages

- `Home.md` — landing page.
- `_Sidebar.md` — wiki navigation.
- `Architecture.md`, `Crate-Index.md`, `API-Reference.md` — understanding the workspace.
- `Memory-Model.md`, `Tunable-Certification.md`, `Decision-Records.md` — core concepts.

`Crate-Index.md` is regenerated from the per-crate READMEs; the rest are authored. Filenames map to
wiki page titles (`Memory-Model.md` → the *Memory Model* page); `_Sidebar.md` is the wiki sidebar.

## Publishing

Trigger the **publish-docs** workflow (Actions tab → *Publish wiki & API docs* → *Run workflow*). It
mirrors `docs/wiki/*.md` to the wiki repo and builds + deploys the rustdoc to GitHub Pages. Required
repo Settings:

- **Wiki** feature enabled, and **Pages** set to *GitHub Actions*.
- A repository secret **`DOCS_PUBLISH_TOKEN`** (Settings → Secrets and variables → Actions) — a
  fine-grained PAT with *Contents: write* (covers the wiki), stored + masked by GitHub's Secrets
  feature so it is never exposed in logs. The wiki job falls back to the built-in `GITHUB_TOKEN` if
  the secret is absent; Pages deploy uses OIDC and needs no extra secret.

See [API Reference](API-Reference) for the docs side.

## Local preview

Markdown renders directly on GitHub; for a local check run the repo's markdown gate
(`bash scripts/checks/markdown.sh`).
