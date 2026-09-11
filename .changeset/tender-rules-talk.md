---
"@pnpm/building.policy": patch
"@pnpm/config.reader": patch
"@pnpm/config.writer": patch
"@pnpm/types": patch
"pnpm": patch
"pacquet": patch
---

pnpm now reads top-level `allowScripts` from `package.json` when `pnpm-workspace.yaml` does not decide a package's `allowBuilds`, warns on conflicts, and syncs existing `allowScripts` objects with boolean `allowBuilds` entries when approvals are written [pnpm/pnpm#12486](https://github.com/pnpm/pnpm/issues/12486).
