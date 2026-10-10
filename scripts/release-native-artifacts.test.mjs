import assert from 'node:assert/strict'
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { basename, dirname, join, resolve } from 'node:path'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import test from 'node:test'
import { validateImage, validateInputs, validateMetadata } from './release-native-artifacts.mjs'

const revision = 'a'.repeat(40)
const digest = 'sha256:' + 'b'.repeat(64)
const expected = {arch: 'arm64', version: '4.0.7', revision, jarSha256: 'c'.repeat(64), archiveSha256: 'd'.repeat(64)}
const image = () => ({Id: digest, Architecture: 'arm64', Os: 'linux', Config: {
  User: '10001:10001', Entrypoint: ['/usr/local/bin/reader-entrypoint'],
  Env: ['READER_RELEASE_VERSION=4.0.7', 'READER_BUILD_REVISION=' + revision, 'READER_APP_WEBVIEWRENDERER=camoufox', 'READER_APP_WEBUI=vue3'],
}})
const metadata = () => ({schemaVersion: 1, architecture: 'arm64', version: '4.0.7', revision,
  imageId: digest, jarSha256: expected.jarSha256, archiveSha256: expected.archiveSha256})
test('same source, shared JAR, archive, and loaded native image are accepted', () => validateMetadata(metadata(), expected, image()))
for (const [label, change] of [
  ['missing architecture', value => delete value.architecture],
  ['wrong architecture', value => {value.architecture = 'amd64'}],
  ['wrong revision', value => {value.revision = 'e'.repeat(40)}],
  ['different JAR bytes', value => {value.jarSha256 = 'e'.repeat(64)}],
  ['tampered image archive', value => {value.archiveSha256 = 'e'.repeat(64)}],
  ['different smoke-tested image ID', value => {value.imageId = 'sha256:' + 'e'.repeat(64)}],
  ['unsupported metadata schema', value => {value.schemaVersion = 2}],
]) {
  test('rejects ' + label, () => {const value = metadata(); change(value); assert.throws(() => validateMetadata(value, expected, image()))})
}
for (const [label, change] of [
  ['wrong loaded architecture', value => {value.Architecture = 'amd64'}],
  ['non-Linux image', value => {value.Os = 'windows'}],
  ['root execution', value => {value.Config.User = '0'}],
  ['different entrypoint', value => {value.Config.Entrypoint = ['sh']}],
  ['wrong release environment', value => {value.Config.Env[0] = 'READER_RELEASE_VERSION=4.0.8'}],
  ['duplicate identity environment', value => {value.Config.Env.push('READER_BUILD_REVISION=' + revision)}],
  ['missing default Vue 3 selection', value => {value.Config.Env.pop()}],
  ['old Vue 2 default', value => {value.Config.Env[value.Config.Env.length - 1] = 'READER_APP_WEBUI=vue2'}],
  ['duplicate default UI environment', value => {value.Config.Env.push('READER_APP_WEBUI=vue3')}],
]) {
  test('rejects ' + label, () => {const value = image(); change(value); assert.throws(() => validateImage(value, expected.arch, expected.version, expected.revision))})
}
test('rejects unsupported architecture, unsafe version, and invalid revision', () => {
  assert.throws(() => validateInputs('x86_64', '4.0.7', revision))
  assert.throws(() => validateInputs('arm64', '../4.0.7', revision))
  assert.throws(() => validateInputs('arm64', '04.0.7', revision))
  assert.throws(() => validateInputs('arm64', '4.0.7', 'legacy'))
})
test('CLI verifies generated file bytes before and after import; tampering fails closed', t => {
  const root = mkdtempSync(join(tmpdir(), 'reader-native-artifact-'))
  t.after(() => {assert.equal(resolve(dirname(root)), resolve(tmpdir())); assert.ok(basename(root).startsWith('reader-native-artifact-')); rmSync(root, {recursive: true, force: true})})
  const jarDirectory = join(root, 'jar'); const nativeDirectory = join(root, 'native')
  mkdirSync(jarDirectory); mkdirSync(nativeDirectory)
  writeFileSync(join(jarDirectory, 'reader-pro-v4.0.7.jar'), 'generated-jar-fixture')
  writeFileSync(join(nativeDirectory, 'reader-image.tar.gz'), 'generated-archive-fixture')
  writeFileSync(join(nativeDirectory, 'image-inspect.json'), JSON.stringify([image()]))
  const script = fileURLToPath(new URL('./release-native-artifacts.mjs', import.meta.url))
  const run = mode => spawnSync(process.execPath, [script, mode, 'arm64', '4.0.7', revision, jarDirectory, nativeDirectory], {encoding: 'utf8', timeout: 10000})
  for (const mode of ['record', 'check', 'loaded']) {const result = run(mode); assert.equal(result.status, 0, result.stderr)}
  const before = readFileSync(join(nativeDirectory, 'metadata.json'))
  assert.notEqual(run('record').status, 0)
  assert.deepEqual(readFileSync(join(nativeDirectory, 'metadata.json')), before)
  writeFileSync(join(nativeDirectory, 'reader-image.tar.gz'), 'tampered-archive')
  assert.notEqual(run('check').status, 0)
})
