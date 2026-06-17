<!--
PR title grammar (Conventional Commits): `type(scope): subject`
  type  -> label: feat→type:feature · fix→type:bug · docs→type:docs · test→type:verification
                  build|ci→type:infra · refactor|perf|style|chore|revert→type:chore
                  (repo friends: spec→type:spec · research→type:research · design→type:design)
  scope -> area:<scope> only when <scope> is an area:* label (core-ir/swap/vsa/execution/
           numerics/selection/toolchain/project/language); otherwise omit it (never invented).
The labels/milestone are reconciled from this title by `gh-issues-sync.py --prs` (add-only).
Example: `feat(swap): certify the binary↔ternary round-trip (M-012)`
-->

## Description

Please include a summary of the changes and which issue is fixed.

Fixes # (issue)

## Type of change (mirrors the PR-title `type`)

- [ ] `design`/`spec` — RFC / ADR / decision or specification change
- [ ] `research` — research update / new source
- [ ] `docs` — documentation improvement
- [ ] `fix` — bug fix (non-breaking change which fixes an issue)
- [ ] `feat` — new feature (non-breaking change which adds functionality)
- [ ] `feat!`/`fix!` — breaking change (would change existing behavior; note it in the subject)

## Checklist

- [ ] My code follows the style guidelines of this project
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes

## Additional context

Add any other context about the PR here.
