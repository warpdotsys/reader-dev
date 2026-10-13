import assert from 'node:assert/strict'
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { basename, dirname, join, resolve } from 'node:path'
import { spawnSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { fileURLToPath } from 'node:url'
import test from 'node:test'
import { validateImage, validateInputs, validateMetadata, validateLoadedMetadata } from './release-native-artifacts.mjs'

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
test('same immutable loaded ID keeps legacy output and does not need the relationship reader', () => {
  assert.equal(validateLoadedMetadata(metadata(), expected, image(), () => {throw new Error('must not inspect a different object')}), undefined)
})
function manifestFixture() {
  const loaded = image(); loaded.Id = 'sha256:' + 'f'.repeat(64)
  const proof = {schemaVersion: 1, loadedImageId: loaded.Id, originalConfigDigest: digest,
    loadedIdObjectKind: 'manifest', architecture: expected.arch, expectedRevision: revision,
    expectedJarSha256: expected.jarSha256, configBytesSha256Verified: true,
    manifestBytesSha256Verified: true, allOriginalLayerDiffIdsMatch: true, readOnlyIdentityCheck: true,
    runtimeExecuted: false, packagedJarBytesIndependentlyRead: false, layerCount: 14}
  return {loaded, proof}
}
test('different immutable ID requires a live inspection and keeps both object identities', () => {
  const {loaded, proof} = manifestFixture(); let calls = 0
  assert.equal(validateLoadedMetadata(metadata(), expected, loaded, (runtimeId, configId, values) => {
    calls++; assert.equal(runtimeId, loaded.Id); assert.equal(configId, digest); assert.equal(values, expected); return proof
  }), proof)
  assert.equal(calls, 1); assert.notEqual(proof.loadedImageId, proof.originalConfigDigest)
  assert.throws(() => validateLoadedMetadata(metadata(), expected, loaded), /live original-config/)
})
test('a plain acceptance boolean or wrong relationship receipt never bypasses provenance', () => {
  const {loaded, proof} = manifestFixture()
  for (const change of [() => true, () => ({...proof, originalConfigDigest: loaded.Id}),
    () => ({...proof, expectedJarSha256: 'e'.repeat(64)}), () => ({...proof, manifestBytesSha256Verified: false}),
    () => ({...proof, allOriginalLayerDiffIdsMatch: false}), () => ({...proof, layerCount: true}),
    () => ({...proof, runtimeExecuted: true}), () => ({...proof, expectedRevision: 'e'.repeat(40)})]) {
    assert.throws(() => validateLoadedMetadata(metadata(), expected, loaded, change), /relationship proof/)
  }
})
test('tampered archives and unsafe loaded runtime fail before any content inspection', () => {
  const {loaded, proof} = manifestFixture(); let calls = 0
  const inspect = () => {calls++; return proof}
  assert.throws(() => validateLoadedMetadata({...metadata(), archiveSha256: 'e'.repeat(64)}, expected, loaded, inspect))
  loaded.Config.User = '0'
  assert.throws(() => validateLoadedMetadata(metadata(), expected, loaded, inspect))
  assert.equal(calls, 0)
})
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

test('Linux CLI bridge executes the real Python byte/layer verifier; generated content tampering refuses',
  {skip: process.platform === 'win32'}, t => {
  const root = mkdtempSync(join(tmpdir(), 'reader-native-bridge-'))
  t.after(() => {
    assert.equal(resolve(dirname(root)), resolve(tmpdir()))
    assert.ok(basename(root).startsWith('reader-native-bridge-'))
    rmSync(root, {recursive: true, force: true})
  })
  const binaries = join(root, 'bin'), jar = join(root, 'jar'), native = join(root, 'native')
  for (const path of [binaries, jar, native]) mkdirSync(path)
  const checksum = value => createHash('sha256').update(value).digest('hex')
  const cfg = {os: 'linux', architecture: 'arm64', config: {...image().Config, WorkingDir: '/app'},
    rootfs: {type: 'layers', diff_ids: ['sha256:' + '1'.repeat(64)]}}
  const config = Buffer.from(JSON.stringify(cfg)), configId = 'sha256:' + checksum(config)
  const manifestObject = {schemaVersion: 2, mediaType: 'application/vnd.oci.image.manifest.v1+json',
    config: {mediaType: 'application/vnd.oci.image.config.v1+json', digest: configId, size: config.length},
    layers: [{mediaType: 'application/vnd.oci.image.layer.v1.tar', digest: cfg.rootfs.diff_ids[0], size: 1024}]}
  const manifest = Buffer.from(JSON.stringify(manifestObject)), manifestId = 'sha256:' + checksum(manifest)
  const inspected = {...image(), Id: manifestId, Config: cfg.config,
    RootFS: {Type: 'layers', Layers: cfg.rootfs.diff_ids},
    Descriptor: {digest: manifestId, mediaType: manifestObject.mediaType}}
  const jarBytes = 'generated-jar', archiveBytes = 'generated-archive'
  writeFileSync(join(jar, 'reader-pro-v4.0.7.jar'), jarBytes)
  writeFileSync(join(native, 'reader-image.tar.gz'), archiveBytes)
  writeFileSync(join(native, 'image-inspect.json'), JSON.stringify([inspected]))
  writeFileSync(join(native, 'metadata.json'), JSON.stringify({...metadata(), imageId: configId,
    jarSha256: checksum(jarBytes), archiveSha256: checksum(archiveBytes)}))
  const encodedConfig = config.toString('base64'), encodedManifest = manifest.toString('base64')
  const shim = `#!/usr/bin/env python3\nimport base64, json, sys\n`
  writeFileSync(join(binaries, 'docker'), shim + `assert sys.argv[1:]==['image','inspect','${manifestId}']\nsys.stdout.write(${JSON.stringify(JSON.stringify([inspected]))})\n`, {mode: 0o700})
  writeFileSync(join(binaries, 'ctr'), shim + `assert sys.argv[1:4]==['--namespace','moby','content'] and sys.argv[4]=='get'\nvalue={'${configId}':'${encodedConfig}','${manifestId}':'${encodedManifest}'}[sys.argv[5]]\nsys.stdout.buffer.write(base64.b64decode(value))\n`, {mode: 0o700})
  const script = fileURLToPath(new URL('./release-native-artifacts.mjs', import.meta.url))
  const run = () => spawnSync(process.execPath, [script, 'loaded', 'arm64', '4.0.7', revision, jar, native],
    {encoding: 'utf8', timeout: 15000, env: {...process.env, PATH: binaries + ':' + process.env.PATH}})
  const before = readFileSync(join(native, 'metadata.json'))
  const success = run(); assert.equal(success.status, 0, success.stderr)
  const receipt = JSON.parse(success.stdout)
  assert.equal(receipt.imageId, configId); assert.equal(receipt.loadedImageId, manifestId)
  assert.equal(receipt.originalConfigIdentity.configBytesSha256Verified, true)
  assert.equal(receipt.originalConfigIdentity.packagedJarBytesIndependentlyRead, false)
  assert.deepEqual(readFileSync(join(native, 'metadata.json')), before)
  writeFileSync(join(binaries, 'ctr'), shim + `sys.stdout.buffer.write(b'tampered-content')\n`, {mode: 0o700})
  const failed = run(); assert.notEqual(failed.status, 0)
  assert.match(failed.stderr, /original-config relationship inspection failed/)
  assert.deepEqual(readFileSync(join(native, 'metadata.json')), before)
})
