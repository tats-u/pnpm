---
"@pnpm/installing.commands": minor
"@pnpm/installing.deps-installer": minor
"@pnpm/installing.deps-resolver": patch
"pacquet": minor
"pnpm": minor
---

Added `pnpm add --types`, which saves matching `@types/*` packages to `devDependencies`, skips packages that bundle `types` or `typings`, and prefers a same-major `@types/*` release before falling back to `latest`.
