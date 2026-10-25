---
"pacquet": minor
---

Top-level `allowScripts` in `package.json` now supplies build approvals for packages without a workspace build decision. Conflicting entries are ignored with a warning. Build approval writes sync boolean decisions into an existing `allowScripts` object [pnpm/pnpm#12486](https://github.com/pnpm/pnpm/issues/12486).
