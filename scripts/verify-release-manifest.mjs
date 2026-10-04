import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
const digest = /^sha256:[0-9a-f]{64}$/
const indexTypes = ['application/vnd.oci.image.index.v1+json', 'application/vnd.docker.distribution.manifest.list.v2+json']
const imageTypes = ['application/vnd.oci.image.manifest.v1+json', 'application/vnd.docker.distribution.manifest.v2+json']
export function verifyReleaseManifest(index, expected) {
  if (!digest.test(expected?.amd64) || !digest.test(expected?.arm64) || expected.amd64 === expected.arm64) throw new Error('Expected two distinct native image digests')
  if (index?.schemaVersion !== 2 || !indexTypes.includes(index.mediaType) ||
      !Array.isArray(index.manifests) || index.manifests.length !== 2) throw new Error('Release must be a two-platform image index')
  const seen = new Set()
  for (const descriptor of index.manifests) {
    const arch = descriptor.platform?.architecture
    if (!['amd64', 'arm64'].includes(arch) || seen.has(arch) || descriptor.platform?.os !== 'linux' ||
        (descriptor.platform?.variant && !(arch === 'arm64' && descriptor.platform.variant === 'v8'))) throw new Error('Release has duplicate, unsupported, or missing native platforms')
    if (!imageTypes.includes(descriptor.mediaType) || descriptor.digest !== expected[arch] ||
        !Number.isSafeInteger(descriptor.size) || descriptor.size <= 0 || descriptor.urls !== undefined) throw new Error('Release descriptor does not match the verified native image')
    seen.add(arch)
  }
  if (seen.size !== 2) throw new Error('Both amd64 and arm64 are required')
  return {platforms: ['linux/amd64', 'linux/arm64'], digests: {amd64: expected.amd64, arm64: expected.arm64}}
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    if (process.argv.length !== 5) throw new Error('Usage: verify-release-manifest.mjs index.json amd64-digest arm64-digest')
    console.log(JSON.stringify(verifyReleaseManifest(JSON.parse(readFileSync(process.argv[2], 'utf8')), {amd64: process.argv[3], arm64: process.argv[4]})))
  } catch (error) {console.error(error.message); process.exitCode = 1}
}
