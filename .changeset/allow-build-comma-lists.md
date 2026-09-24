---
"@pnpm/cli.utils": patch
"@pnpm/exec.commands": patch
"@pnpm/installing.commands": patch
"pnpm": patch
"pacquet": patch
---

`pnpm add`, `pnpm dlx`, and `pnpm create` now accept comma-separated values in `--allow-build` [pnpm/pnpm#11759](https://github.com/pnpm/pnpm/issues/11759).
