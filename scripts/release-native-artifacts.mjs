import { createHash } from 'node:crypto'
import { createReadStream, existsSync, readFileSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const sha = /^[0-9a-f]{64}$/
const digest = /^sha256:[0-9a-f]{64}$/
export function validateInputs(arch, version, revision) {
  if (!['amd64', 'arm64'].includes(arch)) throw new Error('Unsupported native architecture')
  if (!/^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/.test(version)) throw new Error('Invalid stable version')
  if (!/^[0-9a-f]{40}$/.test(revision)) throw new Error('Invalid source revision')
}
export function validateImage(image, arch, version, revision) {
  validateInputs(arch, version, revision)
  if (!image || image.Os !== 'linux' || image.Architecture !== arch || !digest.test(image.Id)) throw new Error('Loaded image architecture, OS, or ID mismatch')
  if (image.Config?.User !== '10001:10001' ||
      JSON.stringify(image.Config?.Entrypoint) !== JSON.stringify(['/usr/local/bin/reader-entrypoint'])) throw new Error('Loaded image user or entrypoint mismatch')
  for (const entry of [
    'READER_RELEASE_VERSION=' + version, 'READER_BUILD_REVISION=' + revision,
    'READER_APP_WEBVIEWRENDERER=camoufox',
    'READER_APP_WEBUI=vue3',
  ]) {
    if (!Array.isArray(image.Config?.Env) || image.Config.Env.filter(value => typeof value === 'string' && value.startsWith(entry.split('=')[0] + '=')).length !== 1 ||
        !image.Config.Env.includes(entry)) throw new Error('Loaded image release identity or renderer mismatch')
  }
}
export function validateMetadata(metadata, expected, image) {
  validateInputs(expected.arch, expected.version, expected.revision)
  if (metadata?.schemaVersion !== 1 || metadata.architecture !== expected.arch ||
      metadata.version !== expected.version || metadata.revision !== expected.revision ||
      !sha.test(metadata.jarSha256) || !sha.test(metadata.archiveSha256) ||
      !digest.test(metadata.imageId) ||
      metadata.jarSha256 !== expected.jarSha256 || metadata.archiveSha256 !== expected.archiveSha256) throw new Error('Native artifact provenance or byte checksum mismatch')
  if (image !== undefined) {
    validateImage(image, expected.arch, expected.version, expected.revision)
    if (metadata.imageId !== image.Id) throw new Error('Loaded image differs from the native smoke-tested image')
  }
}
export async function hashFile(path) {
  const hash = createHash('sha256')
  // Archives may be GiB-sized. Never allocate the complete image in RAM.
  for await (const chunk of createReadStream(path)) hash.update(chunk)
  return hash.digest('hex')
}
async function main() {
  const [mode, arch, version, revision, jarDirectory, nativeDirectory] = process.argv.slice(2)
  if (!['record', 'check', 'loaded'].includes(mode) || !jarDirectory || !nativeDirectory || process.argv.length !== 8) throw new Error('Usage: release-native-artifacts.mjs record|check|loaded arch version revision jar-directory native-directory')
  validateInputs(arch, version, revision)
  const directory = resolve(nativeDirectory)
  const expected = {
    arch, version, revision,
    jarSha256: await hashFile(resolve(jarDirectory, 'reader-pro-v' + version + '.jar')),
    archiveSha256: await hashFile(resolve(directory, 'reader-image.tar.gz')),
  }
  const readImage = () => {
    const images = JSON.parse(readFileSync(resolve(directory, 'image-inspect.json'), 'utf8'))
    if (!Array.isArray(images) || images.length !== 1) throw new Error('Expected exactly one inspected image')
    return images[0]
  }
  const metadataPath = resolve(directory, 'metadata.json')
  let metadata
  if (mode === 'record') {
    if (existsSync(metadataPath)) throw new Error('Refusing to overwrite existing native provenance')
    const image = readImage()
    validateImage(image, arch, version, revision)
    metadata = {
      schemaVersion: 1, architecture: arch, version, revision,
      jarSha256: expected.jarSha256, archiveSha256: expected.archiveSha256, imageId: image.Id,
    }
    validateMetadata(metadata, expected, image)
    writeFileSync(metadataPath, JSON.stringify(metadata, null, 2) + '\n', {flag: 'wx'})
  } else {
    metadata = JSON.parse(readFileSync(metadataPath, 'utf8'))
    validateMetadata(metadata, expected, mode === 'loaded' ? readImage() : undefined)
  }
  console.log(JSON.stringify({phase: mode, architecture: arch, revision, jarSha256: metadata.jarSha256, imageId: metadata.imageId}))
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(error => { console.error(error.message); process.exitCode = 1 })
}
