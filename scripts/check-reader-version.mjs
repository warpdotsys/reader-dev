#!/usr/bin/env node
/** Read-only synchronization check; dependency versions are intentionally untouched. */
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const stable = /^(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)$/
const sourceRoot = resolve(fileURLToPath(new URL('..', import.meta.url)))

export function checkReaderVersion(root = sourceRoot, expected) {
  const read = file => readFileSync(resolve(root, file), 'utf8')
  const definitions = (file, pattern) => {
    const matches = [...read(file).matchAll(pattern)]
    if (matches.length !== 1) throw new Error(`Expected exactly one project version: ${file}`)
    return matches[0][1]
  }
  const version = definitions('build.gradle.kts', /^version\s*=\s*"([^"]+)"\s*$/gm)
  if (!stable.test(version)) throw new Error('Project version must be stable SemVer')
  if (expected !== undefined && (!stable.test(expected) || expected !== version)) {
    throw new Error(`Requested version does not match project version ${version}`)
  }
  if (definitions('cli.gradle', /^version\s*=\s*'([^']+)'\s*$/gm) !== version) {
    throw new Error('CLI project version does not match the main project')
  }
  for (const directory of ['web', 'web-vue3']) {
    const manifest = JSON.parse(read(`${directory}/package.json`))
    const lock = JSON.parse(read(`${directory}/package-lock.json`))
    if (manifest.version !== version || lock.version !== version || lock.packages?.['']?.version !== version) {
      throw new Error(`Package and root lock versions must match: ${directory}`)
    }
    if (manifest.name !== lock.name || manifest.name !== lock.packages?.['']?.name) {
      throw new Error(`Package and root lock identities must match: ${directory}`)
    }
  }
  return version
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    if (process.argv.length > 3) throw new Error('Usage: check-reader-version.mjs [expected-version]')
    console.log(`Reader project versions synchronized: ${checkReaderVersion(sourceRoot, process.argv[2])}`)
  } catch (error) {
    console.error(error.message)
    process.exitCode = 1
  }
}
