import fs from 'node:fs'
import path from 'node:path'

import { parseWantedDependency } from '@pnpm/resolving.parse-wanted-dependency'

export function splitCommaSeparatedSelectors (selectors: string[] | undefined, baseDir: string): string[] | undefined {
  return selectors?.flatMap((selector) => splitCommaSeparated(selector, baseDir))
}

function splitCommaSeparated (selector: string, baseDir: string): string[] {
  const specifier = parseWantedDependency(selector)?.bareSpecifier ?? selector

  if (!specifier.includes(',')) return [selector]
  if (specifier.includes('://')) return [selector]
  if (refersToExistingLocalPath(specifier, baseDir)) return [selector]

  return selector.split(',').map((token) => token.trim()).filter(Boolean)
}

function refersToExistingLocalPath (specifier: string, baseDir: string): boolean {
  let pathPart: string
  if (specifier.startsWith('file:')) {
    pathPart = specifier.slice('file:'.length)
  } else if (specifier.startsWith('link:')) {
    pathPart = specifier.slice('link:'.length)
  } else if (specifier[0] === '.' || specifier[0] === '/' || specifier[0] === '~') {
    pathPart = specifier
  } else if (/^[a-z]:[/\\]/i.test(specifier)) {
    pathPart = specifier
  } else {
    return false
  }

  const resolved = path.isAbsolute(pathPart) ? pathPart : path.resolve(baseDir, pathPart)
  return fs.existsSync(resolved)
}
