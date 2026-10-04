import assert from 'node:assert/strict'
import test from 'node:test'
import { verifyReleaseManifest } from './verify-release-manifest.mjs'
const expected = {amd64: 'sha256:' + 'a'.repeat(64), arm64: 'sha256:' + 'b'.repeat(64)}
const fixture = () => ({schemaVersion: 2, mediaType: 'application/vnd.oci.image.index.v1+json',
  manifests: ['amd64', 'arm64'].map(arch => ({mediaType: 'application/vnd.oci.image.manifest.v1+json',
    digest: expected[arch], size: 1000, platform: {os: 'linux', architecture: arch}})),
})
test('accepts exactly the two verified native image descriptors', () => assert.deepEqual(verifyReleaseManifest(fixture(), expected).platforms, ['linux/amd64', 'linux/arm64']))
test('Docker manifest-list media types and reversed platform order are accepted', () => {
  const index = fixture(); index.mediaType = 'application/vnd.docker.distribution.manifest.list.v2+json'
  index.manifests.reverse().forEach(item => {item.mediaType = 'application/vnd.docker.distribution.manifest.v2+json'})
  verifyReleaseManifest(index, expected)
})
test('ARM64 v8 variant is accepted', () => {const index = fixture(); index.manifests[1].platform.variant = 'v8'; verifyReleaseManifest(index, expected)})
for (const [label, change] of [
  ['missing arm64', index => {index.manifests.pop()}],
  ['duplicated amd64', index => {index.manifests[1] = {...index.manifests[0]}}],
  ['unexpected third descriptor', index => {index.manifests.push({...index.manifests[0]})}],
  ['Windows platform', index => {index.manifests[1].platform.os = 'windows'}],
  ['wrong native digest', index => {index.manifests[1].digest = 'sha256:' + 'c'.repeat(64)}],
  ['nested index instead of native image', index => {index.manifests[1].mediaType = index.mediaType}],
  ['invalid descriptor size', index => {index.manifests[1].size = 0}],
  ['untrusted external blob URL', index => {index.manifests[1].urls = ['https://example.invalid/blob']}],
  ['unsupported ARM variant', index => {index.manifests[1].platform.variant = 'v7'}],
  ['wrong schema', index => {index.schemaVersion = 1}],
]) {
  test('rejects ' + label, () => {const index = fixture(); change(index); assert.throws(() => verifyReleaseManifest(index, expected))})
}
test('rejects missing, non-digest, and shared expected digests', () => {
  for (const value of [{}, {amd64:'latest',arm64:expected.arm64}, {amd64:expected.amd64,arm64:expected.amd64}]) assert.throws(() => verifyReleaseManifest(fixture(), value))
})
