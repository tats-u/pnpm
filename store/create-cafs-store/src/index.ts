import { promises as fs } from 'fs'
import path from 'path'
import {
  createCafs,
  getFilePathByModeInCafs,
} from '@pnpm/cafs'
import type { Cafs, PackageFilesResponse } from '@pnpm/cafs-types'
import { createIndexedPkgImporter } from '@pnpm/fs.indexed-pkg-importer'
import {
  type ImportIndexedPackage,
  type ImportPackageFunction,
  type PackageFileInfo,
} from '@pnpm/store-controller-types'
import memoize from 'mem'
import pathTemp from 'path-temp'

function createPackageImporter (
  opts: {
    importIndexedPackage?: ImportIndexedPackage
    packageImportMethod?: 'auto' | 'hardlink' | 'copy' | 'clone' | 'clone-or-copy'
    cafsDir: string
  }
): ImportPackageFunction {
  const cachedImporterCreator = opts.importIndexedPackage
    ? () => opts.importIndexedPackage!
    : memoize(createIndexedPkgImporter)
  const packageImportMethod = opts.packageImportMethod
  const gfm = getFlatMap.bind(null, opts.cafsDir)
  return async (to, opts) => {
    const { filesMap, isBuilt } = gfm(opts.filesResponse, opts.sideEffectsCacheKey)
    const pkgImportMethod = (opts.requiresBuild && !isBuilt)
      ? 'clone-or-copy'
      : (opts.filesResponse.packageImportMethod ?? packageImportMethod)
    const impPkg = cachedImporterCreator(pkgImportMethod)
    const importMethod = await impPkg(to, {
      filesMap,
      fromStore: opts.filesResponse.fromStore,
      force: opts.force,
      keepModulesDir: Boolean(opts.keepModulesDir),
    })
    return { importMethod, isBuilt }
  }
}

function getFlatMap (
  cafsDir: string,
  filesResponse: PackageFilesResponse,
  targetEngine?: string
): { filesMap: Map<string, string>, isBuilt: boolean } {
  if (filesResponse.local) {
    return {
      filesMap: filesResponse.filesIndex,
      isBuilt: false,
    }
  }
  let isBuilt!: boolean
  let filesIndex!: Map<string, PackageFileInfo>
  if (targetEngine && filesResponse.sideEffects?.has(targetEngine)) {
    filesIndex = filesResponse.sideEffects.get(targetEngine)!
    isBuilt = true
  } else {
    filesIndex = filesResponse.filesIndex
    isBuilt = false
  }
  const filesMap = new Map<string, string>()
  for (const [filename, { integrity, mode }] of filesIndex.entries()) {
    filesMap.set(filename, getFilePathByModeInCafs(cafsDir, integrity, mode))
  }
  return { filesMap, isBuilt }
}

export function createCafsStore (
  storeDir: string,
  opts?: {
    ignoreFile?: (filename: string) => boolean
    importPackage?: ImportIndexedPackage
    packageImportMethod?: 'auto' | 'hardlink' | 'copy' | 'clone' | 'clone-or-copy'
  }
): Cafs {
  const cafsDir = path.join(storeDir, 'files')
  const baseTempDir = path.join(storeDir, 'tmp')
  const importPackage = createPackageImporter({
    importIndexedPackage: opts?.importPackage,
    packageImportMethod: opts?.packageImportMethod,
    cafsDir,
  })
  return {
    ...createCafs(cafsDir, opts?.ignoreFile),
    cafsDir,
    importPackage,
    tempDir: async () => {
      const tmpDir = pathTemp(baseTempDir)
      await fs.mkdir(tmpDir, { recursive: true })
      return tmpDir
    },
  }
}
