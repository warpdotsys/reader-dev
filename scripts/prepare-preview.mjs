#!/usr/bin/env node
import { constants, copyFileSync, existsSync, lstatSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { hashFile, validateMetadata } from './release-native-artifacts.mjs'
import { checkReaderVersion } from './check-reader-version.mjs'

export function validatePreview(config, version) {
  if (config?.schemaVersion !== 1 || config.version !== version ||
      !/^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/.test(version) ||
      typeof config.tag !== 'string' ||
      !config.tag.startsWith(`v${version}-preview.`) ||
      !/^[1-9][0-9]*$/.test(config.tag.slice(`v${version}-preview.`.length))) {
    throw new Error('Expected a matching immutable vX.Y.Z-preview.N identity')
  }
  return config
}

function regular(path, limit = 2 * 1024 ** 3 - 1) {
  const stat = lstatSync(path)
  if (!stat.isFile() || stat.isSymbolicLink() || stat.size <= 0 || stat.size > limit) {
    throw new Error('Invalid or oversized candidate input')
  }
  return stat.size
}

function json(path) {
  regular(path, 2 * 1024 ** 2)
  return JSON.parse(readFileSync(path, 'utf8'))
}

export async function preparePreview({ config, version, revision, runUrl, inputs, output, notes }) {
  validatePreview(config, version)
  if (!/^[0-9a-f]{40}$/.test(revision) ||
      !/^https:\/\/github\.com\/warpdotsys\/reader-dev\/actions\/runs\/[1-9][0-9]*$/.test(runUrl)) {
    throw new Error('Invalid source or hosted run identity')
  }
  if (existsSync(output)) throw new Error('Refusing to overwrite an existing candidate bundle')
  const source = name => resolve(inputs, name)
  const jar = source(`jar/reader-pro-v${version}.jar`)
  regular(jar); regular(source('ci/first.jar'))
  const jarSha = await hashFile(jar)
  if (await hashFile(source('ci/first.jar')) !== jarSha) throw new Error('CI and native image JAR differ')
  const repro = json(source('repro/REPRODUCIBILITY.json'))
  if (repro.schemaVersion !== 1 || repro.passed !== true || repro.archiveBytesMatch !== true ||
      repro.firstJarSha256 !== jarSha || repro.secondJarSha256 !== jarSha) {
    throw new Error('Missing exact-byte Reader reproducibility proof')
  }
  const licenseJar = source('license/first.jar')
  regular(licenseJar)
  const licenseSha = await hashFile(licenseJar)
  const smoke = json(source('license/SMOKE.json'))
  if (smoke.schemaVersion !== 1 || smoke.passed !== true || smoke.jarSha256 !== licenseSha ||
      smoke.smtpMessageCount !== 1 || !['eightDigitCodeReceived', 'licenseSupplied', 'licenseActivated',
        'duplicateClaimRejected', 'restartClaimRejected'].every(key => smoke[key] === true)) {
    throw new Error('Missing tested license JAR provenance')
  }
  regular(notes, 256 * 1024)
  const releaseNotes = readFileSync(notes, 'utf8')
  for (const heading of ['## 已知问题', '## 验证范围', '## 升级与回滚']) {
    if (!releaseNotes.includes(heading)) throw new Error('Required release note section missing')
  }
  const entries = [[jar, `reader-pro-${config.tag}.jar`],
    [licenseJar, `reader-license-${config.tag}.jar`], [notes, 'RELEASE-NOTES.md'],
    [source('repro/REPRODUCIBILITY.json'), 'REPRODUCIBILITY.json'],
    [source('license/SMOKE.json'), 'LICENSE-SMOKE.json']]
  for (const arch of ['amd64', 'arm64']) {
    const directory = source(`native/reader-tested-native-${arch}-${revision}`)
    const archive = resolve(directory, 'reader-image.tar.gz')
    regular(archive)
    const metadataFile = resolve(directory, 'metadata.json')
    validateMetadata(json(metadataFile), { arch, version, revision,
      jarSha256: jarSha, archiveSha256: await hashFile(archive) })
    entries.push([archive, `reader-pro-${config.tag}-linux-${arch}.tar.gz`],
      [metadataFile, `linux-${arch}-metadata.json`])
  }
  // All identities and byte proofs are checked before creating any output.
  mkdirSync(output)
  const assets = []
  for (const [from, name] of entries) {
    const destination = resolve(output, name)
    copyFileSync(from, destination, constants.COPYFILE_EXCL)
    assets.push({ name, bytes: regular(destination), sha256: await hashFile(destination) })
  }
  const manifest = { schemaVersion: 1, tag: config.tag, applicationVersion: version, revision, runUrl,
    channel: 'preview', productionDeployment: false, assets }
  writeFileSync(resolve(output, 'PREVIEW.json'), JSON.stringify(manifest, null, 2) + '\n', { flag: 'wx' })
  const sums = [...assets.map(asset => `${asset.sha256}  ${asset.name}`),
    `${await hashFile(resolve(output, 'PREVIEW.json'))}  PREVIEW.json`]
  writeFileSync(resolve(output, 'SHA256SUMS'), sums.join('\n') + '\n', { flag: 'wx' })
  return manifest
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const root = resolve(fileURLToPath(new URL('..', import.meta.url)))
    const config = validatePreview(json(resolve(root, 'preview.json')), checkReaderVersion(root))
    if (process.argv[2] === '--check' && process.argv.length === 3) {
      console.log(JSON.stringify(config))
    } else {
      if (process.argv.length !== 6) throw new Error('Usage: prepare-preview.mjs inputs output revision run-url')
      const [, , inputs, output, revision, runUrl] = process.argv
      const manifest = await preparePreview({ config, version: config.version, revision, runUrl,
        inputs: resolve(inputs), output: resolve(output), notes: resolve(root, `docs/releases/${config.tag}.md`) })
      console.log(JSON.stringify({ tag: manifest.tag, revision, assetCount: manifest.assets.length }))
    }
  } catch (error) {
    console.error(error.message)
    process.exitCode = 1
  }
}
