import assert from 'node:assert/strict'
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { basename, dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import test from 'node:test'
import { checkReaderVersion } from './check-reader-version.mjs'

const root = resolve(fileURLToPath(new URL('..', import.meta.url)))
const files = ['build.gradle.kts', 'cli.gradle', 'web/package.json', 'web/package-lock.json',
  'web-vue3/package.json', 'web-vue3/package-lock.json']

function fixture(t) {
  const directory = mkdtempSync(join(tmpdir(), 'reader-version-check-'))
  t.after(() => {
    assert.equal(resolve(dirname(directory)), resolve(tmpdir()))
    assert.ok(basename(directory).startsWith('reader-version-check-'))
    rmSync(directory, { recursive: true, force: true })
  })
  for (const file of files) {
    const destination = join(directory, file)
    mkdirSync(dirname(destination), { recursive: true })
    copyFileSync(join(root, file), destination)
  }
  return { directory, change(file, edit) {
    const path = join(directory, file)
    writeFileSync(path, edit(readFileSync(path, 'utf8')), 'utf8')
  } }
}

test('all project and root lock versions match without examining dependency versions', t => {
  const input = fixture(t)
  const version = checkReaderVersion(input.directory)
  assert.equal(checkReaderVersion(input.directory, version), version)
  assert.throws(() => checkReaderVersion(input.directory, '0.0.0'), /Requested version/)
  for (const directory of ['web', 'web-vue3']) {
    input.change(`${directory}/package-lock.json`, text => {
      const lock = JSON.parse(text)
      lock.packages['node_modules/generated-test-dependency'] = { version: '4.0.7' }
      return JSON.stringify(lock)
    })
  }
  assert.equal(checkReaderVersion(input.directory), version)
})

for (const directory of ['web', 'web-vue3']) {
  for (const target of ['manifest', 'lock', 'root-lock', 'name']) {
    test(`rejects ${directory} ${target} drift`, t => {
      const input = fixture(t)
      const file = `${directory}/${target === 'manifest' ? 'package.json' : 'package-lock.json'}`
      input.change(file, text => {
        const value = JSON.parse(text)
        if (target === 'root-lock') value.packages[''].version = '0.0.0'
        else if (target === 'name') value.packages[''].name = 'generated-wrong-identity'
        else value.version = '0.0.0'
        return JSON.stringify(value)
      })
      assert.throws(() => checkReaderVersion(input.directory), /versions must match|identities must match/)
    })
  }
}

test('rejects CLI drift and duplicate project version definitions', t => {
  const input = fixture(t)
  input.change('cli.gradle', text => text.replace(/^version = '[^']+'$/m, "version = '0.0.0'"))
  assert.throws(() => checkReaderVersion(input.directory), /CLI project version/)
  input.change('build.gradle.kts', text => text + '\nversion = "0.0.0"\n')
  assert.throws(() => checkReaderVersion(input.directory), /exactly one project version/)
})

for (const version of ['06.1.0', '6.1.0-rc.1', '6.1', '../6.1.0']) {
  test(`rejects unsafe or non-stable version ${version}`, t => {
    const input = fixture(t)
    input.change('build.gradle.kts', text => text.replace(/^version = "[^"]+"$/m, `version = "${version}"`))
    assert.throws(() => checkReaderVersion(input.directory), /stable SemVer/)
  })
}
