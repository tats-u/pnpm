import fs from 'node:fs'
import path from 'node:path'

import { expect, test } from '@jest/globals'
import { prepareEmpty } from '@pnpm/prepare'
import type { DepPath } from '@pnpm/types'
import { readYamlFileSync } from 'read-yaml-file'
import { writeYamlFileSync } from 'write-yaml-file'

import { handleIgnoredBuilds } from '../lib/handleIgnoredBuilds.js'

test('handleIgnoredBuilds does not update pnpm-workspace.yaml when workspace is ignored', async () => {
  prepareEmpty()

  const workspaceManifestFile = path.resolve('pnpm-workspace.yaml')
  const workspaceManifest = {
    allowBuilds: {
      esbuild: false,
    },
  }
  writeYamlFileSync(workspaceManifestFile, workspaceManifest)
  const workspaceManifestBefore = fs.readFileSync(workspaceManifestFile, 'utf8')

  await handleIgnoredBuilds({
    ignoreWorkspace: true,
    rootProjectManifestDir: process.cwd(),
  }, new Set(['esbuild@0.25.0' as DepPath]))

  expect(fs.readFileSync(workspaceManifestFile, 'utf8')).toBe(workspaceManifestBefore)
  expect(readYamlFileSync(workspaceManifestFile)).toStrictEqual(workspaceManifest)
})

test('handleIgnoredBuilds syncs existing package.json allowScripts entries into allowBuilds', async () => {
  prepareEmpty()

  fs.writeFileSync('package.json', JSON.stringify({
    allowScripts: {
      esbuild: true,
    },
  }, null, 2))

  await handleIgnoredBuilds({
    allowBuilds: {
      esbuild: true,
    },
    rootProjectManifestDir: process.cwd(),
    workspaceDir: process.cwd(),
  }, new Set(['sharp@0.33.0' as DepPath]))

  expect(readYamlFileSync(path.resolve('pnpm-workspace.yaml'))).toStrictEqual({
    allowBuilds: {
      esbuild: true,
      sharp: 'set this to true or false',
    },
  })
  expect(JSON.parse(fs.readFileSync(path.resolve('package.json'), 'utf8'))).toStrictEqual({
    allowScripts: {
      esbuild: true,
    },
  })
})
