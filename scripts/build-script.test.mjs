/** Execute the actual local build entry with a generated Gradle/JDK fixture.
 * These are script contract tests, not proof that the Reader JAR builds.
 */
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { chmodSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { basename, dirname, join, resolve } from 'node:path'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import test from 'node:test'

const root = resolve(fileURLToPath(new URL('..', import.meta.url)))
const windows = process.platform === 'win32'
// Use the same PowerShell 7 runtime on Windows and the hosted Linux runner;
// inheriting PS7's PSModulePath into Windows PowerShell 5 loses Get-FileHash.
const powershell = 'pwsh'
const quote = value => "'" + value.replaceAll("'", "''") + "'"

function fixture(t, { version = '6.1.0', artifact = '6.1.0', failure = false, changeVersion = false, empty = false } = {}) {
  const directory = mkdtempSync(join(tmpdir(), 'reader-build-script-'))
  t.after(() => {
    assert.equal(resolve(dirname(directory)), resolve(tmpdir()))
    assert.ok(basename(directory).startsWith('reader-build-script-'))
    rmSync(directory, { recursive: true, force: true })
  })
  for (const child of ['scripts', '.tools/jdk-11.0.8/bin', 'build/libs', 'build/authorized-fixture']) {
    mkdirSync(join(directory, child), { recursive: true })
  }
  copyFileSync(join(root, 'scripts/build.ps1'), join(directory, 'scripts/build.ps1'))
  copyFileSync(join(root, 'scripts/reader-project-version.ps1'), join(directory, 'scripts/reader-project-version.ps1'))
  writeFileSync(join(directory, 'build.gradle.kts'), `version = "${version}"\n`)
  writeFileSync(join(directory, 'changed.gradle'), 'version = "6.2.0"\n')
  writeFileSync(join(directory, '.tools/jdk-11.0.8/bin/java.exe'), 'generated-placeholder-never-executed')
  writeFileSync(join(directory, 'fixture.jar'), empty ? '' : 'generated-test-artifact-not-a-real-reader')
  writeFileSync(join(directory, 'build/authorized-fixture/sentinel.txt'), 'generated-input-must-survive')
  const stub = windows
    ? `@echo off\r\necho %* > "%~dp0args.txt"\r\n${changeVersion ? 'copy /b "%~dp0changed.gradle" "%~dp0build.gradle.kts" >nul\r\n' : ''}${failure ? 'exit /b 7\r\n' : `copy /b "%~dp0fixture.jar" "%~dp0build\\libs\\reader-${artifact}.jar" >nul\r\nexit /b 0\r\n`}`
    : `#!/bin/sh\nprintf '%s\\n' "$@" > args.txt\n${changeVersion ? 'cp changed.gradle build.gradle.kts\n' : ''}${failure ? 'exit 7\n' : `cp fixture.jar build/libs/reader-${artifact}.jar\nexit 0\n`}`
  writeFileSync(join(directory, 'gradlew.bat'), stub)
  if (!windows) chmodSync(join(directory, 'gradlew.bat'), 0o700)
  return { directory, run() {
    const command = `$ErrorActionPreference='Stop'; $record = & ${quote(join(directory, 'scripts/build.ps1'))}; $record | ConvertTo-Json -Depth 3`
    const result = spawnSync(powershell, ['-NoLogo', '-NoProfile', '-NonInteractive', '-Command', command],
      { cwd: directory, encoding: 'utf8', timeout: 20000 })
    assert.equal(result.error, undefined, `PowerShell script execution unavailable: ${result.error}`)
    assert.equal(readFileSync(join(directory, 'build/authorized-fixture/sentinel.txt'), 'utf8'), 'generated-input-must-survive')
    return result
  } }
}

test('actual PowerShell build entry uses exact version, finite workers, and preserves build inputs', t => {
  const input = fixture(t)
  const result = input.run()
  assert.equal(result.status, 0, result.stderr)
  assert.ok(existsSync(join(input.directory, 'build/libs/reader-6.1.0.jar')))
  const args = readFileSync(join(input.directory, 'args.txt'), 'utf8').trim().split(/\s+/)
  assert.deepEqual(args, ['test', 'bootJar', '--rerun-tasks', '--max-workers=2', '--no-build-cache', '--no-daemon'])
  const record = JSON.parse(result.stdout)
  assert.equal(record.FullName, join(input.directory, 'build/libs/reader-6.1.0.jar'))
  assert.equal(record.Length, 41)
  assert.equal(record.Algorithm, 'SHA256')
  assert.equal(record.SHA256.toLowerCase(), createHash('sha256').update('generated-test-artifact-not-a-real-reader').digest('hex'))
})

for (const [name, options, expected] of [
  ['wrong artifact version', { artifact: '4.0.7' }, /Expected artifact was not produced/],
  ['empty artifact', { empty: true }, /Expected artifact was not produced/],
  ['Gradle failure', { failure: true }, /Gradle build failed with exit code 7/],
  ['project version changing during build', { changeVersion: true }, /Project version changed during the build/],
  ['unsafe project version before any build', { version: '../6.1.0' }, /Expected exactly one stable project version/],
  ['pre-release project version before any build', { version: '6.1.0-rc.1' }, /Expected exactly one stable project version/],
]) {
  test(`actual PowerShell build entry rejects ${name}`, t => {
    const input = fixture(t, options)
    const result = input.run()
    assert.notEqual(result.status, 0)
    assert.match(result.stderr + result.stdout, expected)
    if (options.version) assert.equal(existsSync(join(input.directory, 'args.txt')), false)
  })
}
