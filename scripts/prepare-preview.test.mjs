import assert from 'node:assert/strict'
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync, existsSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import test from 'node:test'
import { hashFile } from './release-native-artifacts.mjs'
import { preparePreview, validatePreview } from './prepare-preview.mjs'

const version = '6.1.1'
const config = { schemaVersion: 1, version, tag: 'v6.1.1-preview.1' }
const revision = 'a'.repeat(40)
const runUrl = 'https://github.com/warpdotsys/reader-dev/actions/runs/1234'
const root = resolve(fileURLToPath(new URL('..', import.meta.url)))

async function fixture(t) {
  const directory = mkdtempSync(join(tmpdir(), 'reader-generated-preview-'))
  t.after(() => rmSync(directory, { recursive: true, force: true }))
  const put = (name, content) => {
    const file = join(directory, name)
    mkdirSync(resolve(file, '..'), { recursive: true })
    writeFileSync(file, typeof content === 'string' ? content : JSON.stringify(content))
    return file
  }
  const jarSha = await hashFile(put(`jar/reader-pro-v${version}.jar`, 'generated-reader-jar'))
  put('ci/first.jar', 'generated-reader-jar')
  put('repro/REPRODUCIBILITY.json', { schemaVersion: 1, passed: true, archiveBytesMatch: true,
    firstJarSha256: jarSha, secondJarSha256: jarSha })
  const licenseSha = await hashFile(put('license/first.jar', 'generated-license-jar'))
  put('license/SMOKE.json', { schemaVersion: 1, passed: true, jarSha256: licenseSha, smtpMessageCount: 1,
    eightDigitCodeReceived: true, licenseSupplied: true, licenseActivated: true,
    duplicateClaimRejected: true, restartClaimRejected: true })
  for (const architecture of ['amd64', 'arm64']) {
    const name = `native/reader-tested-native-${architecture}-${revision}`
    const archiveSha256 = await hashFile(put(`${name}/reader-image.tar.gz`, `generated-image-${architecture}`))
    put(`${name}/metadata.json`, { schemaVersion: 1, architecture, version, revision, jarSha256: jarSha,
      archiveSha256, imageId: 'sha256:' + 'b'.repeat(64) })
  }
  const notes = put('notes.md', '# Generated preview\n## 已知问题\n未完成。\n## 验证范围\n生成样本。\n## 升级与回滚\n先备份。\n')
  const args = { config, version, revision, runUrl, inputs: directory, output: join(directory, 'output'), notes }
  return { put, args }
}

test('assembles only exact tested software bytes and binds every asset hash', async t => {
  const { args } = await fixture(t)
  const result = await preparePreview(args)
  assert.equal(result.channel, 'preview')
  assert.equal(result.productionDeployment, false)
  assert.equal(result.assets.length, 9)
  for (const asset of result.assets) assert.equal(await hashFile(join(args.output, asset.name)), asset.sha256)
  assert.match(readFileSync(join(args.output, 'SHA256SUMS'), 'utf8'), /PREVIEW.json/)
  await assert.rejects(preparePreview(args), /overwrite/)
})

for (const tag of ['v6.1.1', 'v6.1.0-preview.1', 'v6.1.1-preview.0', 'v6.1.1-preview.01', '../preview.1', 'v6.1.1-preview.1\n']) {
  test(`refuses non-candidate identity ${JSON.stringify(tag)}`, () => {
    assert.throws(() => validatePreview({ ...config, tag }, version), /immutable/)
  })
}

for (const [name, corrupt, expected] of [
  ['CI bytes', f => f.put('ci/first.jar', 'other-jar'), /JAR differ/],
  ['rebuild result', f => f.put('repro/REPRODUCIBILITY.json', { passed: false }), /reproducibility/],
  ['license result', f => f.put('license/SMOKE.json', { passed: true }), /license JAR/],
  ['image bytes', f => f.put(`native/reader-tested-native-arm64-${revision}/reader-image.tar.gz`, 'different-image'), /checksum mismatch/],
  ['release notes', f => f.put('notes.md', '# No known issues section'), /section missing/],
]) {
  test(`fails before output creation when ${name} is inconsistent`, async t => {
    const f = await fixture(t); corrupt(f)
    await assert.rejects(preparePreview(f.args), expected)
    assert.equal(existsSync(f.args.output), false)
  })
}

test('candidate workflow cannot publish latest, deploy, or bypass its three gates', () => {
  const workflow = readFileSync(join(root, '.github/workflows/preview-release.yml'), 'utf8')
  assert.match(workflow, /needs: \[checks, ui, native\]/)
  for (const file of ['ci.yml', 'release-native.yml', 'vue3-preview.yml']) {
    assert.ok(workflow.includes(`uses: ./.github/workflows/${file}`))
  }
  assert.match(workflow, /github.event_name == 'workflow_dispatch' && inputs.create_draft/)
  assert.match(workflow, /--draft --prerelease --latest=false/)
  assert.doesNotMatch(workflow, /self-hosted|packages: write|secrets: inherit|\bssh\b|\bscp\b|docker push|continue-on-error/)
  assert.match(readFileSync(join(root, '.github/workflows/release.yml'), 'utf8'), /- '!v\*-\*'/)
  assert.match(readFileSync(join(root, '.github/workflows/release.yml'), 'utf8'), /if: \$\{\{ vars.READER_ENABLE_PRODUCTION_DEPLOY == 'true' \}\}/)
})

test('release gate executes both browser suites and preserves their actual XML', () => {
  const workflow = readFileSync(join(root, '.github/workflows/release-native.yml'), 'utf8')
  for (const name of ['CamoufoxWebviewRendererTest', 'LocalWebviewRendererTest']) {
    assert.ok(workflow.includes(`--tests 'com.htmake.reader.utils.${name}'`))
    assert.ok(workflow.includes(`build/test-results/test/TEST-com.htmake.reader.utils.${name}.xml`))
  }
  assert.ok(workflow.includes('python3 scripts/verify-browser-jvm-contracts.py --results-dir build/test-results/test'))
})
