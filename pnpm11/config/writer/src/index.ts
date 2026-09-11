import path from 'node:path'

import { allowBuildsToAllowScripts, mergeAllowScriptsIntoAllowBuilds, readAllowScripts } from '@pnpm/building.policy'
import type { PnpmSettings, ProjectManifest } from '@pnpm/types'
import { tryReadProjectManifest } from '@pnpm/workspace.project-manifest-reader'
import { updateWorkspaceManifest } from '@pnpm/workspace.workspace-manifest-writer'

export interface WriteSettingsOptions {
  updatedSettings?: PnpmSettings
  updatedOverrides?: Record<string, string>
  updatedAuditIgnoreGhsas?: string[]
  addedMinimumReleaseAgeExcludes?: string[]
  deletedLegacyKeys?: string[]
  rootProjectManifest?: ProjectManifest
  rootProjectManifestDir: string
  workspaceDir: string
}

type ProjectManifestWithRawAllowScripts =
  & Omit<ProjectManifest, 'allowScripts'>
  & {
    allowScripts?: Record<string, unknown>
  }

export async function writeSettings (opts: WriteSettingsOptions): Promise<void> {
  const updatedSettings = opts.updatedSettings == null ? undefined : { ...opts.updatedSettings }
  let rootProjectManifestUpdate: undefined | {
    manifest: ProjectManifestWithRawAllowScripts
    writeProjectManifest: (manifest: ProjectManifest, force?: boolean) => Promise<void>
  }
  if (
    updatedSettings?.allowBuilds != null &&
    path.resolve(opts.workspaceDir) === path.resolve(opts.rootProjectManifestDir)
  ) {
    const { fileName, manifest, writeProjectManifest } = await tryReadProjectManifest(opts.rootProjectManifestDir)
    if (fileName === 'package.json' && manifest != null) {
      const { allowScripts, hasObject } = readAllowScripts((manifest as ProjectManifestWithRawAllowScripts).allowScripts)
      if (hasObject) {
        updatedSettings.allowBuilds = mergeAllowScriptsIntoAllowBuilds(updatedSettings.allowBuilds, allowScripts).allowBuilds
        rootProjectManifestUpdate = {
          manifest: manifest as ProjectManifestWithRawAllowScripts,
          writeProjectManifest,
        }
      }
    }
  }
  await updateWorkspaceManifest(opts.workspaceDir, {
    updatedFields: updatedSettings,
    updatedOverrides: opts.updatedOverrides,
    updatedAuditIgnoreGhsas: opts.updatedAuditIgnoreGhsas,
    addedMinimumReleaseAgeExcludes: opts.addedMinimumReleaseAgeExcludes,
    deletedLegacyKeys: opts.deletedLegacyKeys,
  })
  if (rootProjectManifestUpdate?.manifest.allowScripts != null && updatedSettings?.allowBuilds != null) {
    rootProjectManifestUpdate.manifest.allowScripts = {
      ...rootProjectManifestUpdate.manifest.allowScripts,
      ...allowBuildsToAllowScripts(updatedSettings.allowBuilds),
    }
    await rootProjectManifestUpdate.writeProjectManifest(rootProjectManifestUpdate.manifest)
  }
}
