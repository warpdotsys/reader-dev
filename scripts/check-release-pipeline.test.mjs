import assert from 'node:assert/strict'
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { basename, dirname, join, resolve } from 'node:path'
import { spawnSync } from 'node:child_process'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

const sourceRoot = resolve(fileURLToPath(new URL('..', import.meta.url)))
const fixtureFiles = [
  'scripts/check-release-pipeline.mjs',
  '.github/workflows/release.yml', '.github/workflows/ci.yml',
  '.github/workflows/release-native.yml', 'scripts/smoke-native-release.sh',
  'scripts/import-native-release.sh',
  '.github/workflows/browser-image.yml', '.github/workflows/vue3-preview.yml',
  'deploy/reader-pro/compose.production.yaml', 'deploy/reader-pro/Dockerfile',
  'deploy/reader-pro/base-images.lock',
]

function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), 'reader-release-guard-'))
  t.after(() => {
    // Only remove the tiny fixture directory this test just created. Never
    // recurse into a repository, the temp root, or a path supplied by a caller.
    assert.equal(resolve(dirname(root)), resolve(tmpdir()))
    assert.ok(basename(root).startsWith('reader-release-guard-'))
    rmSync(root, { recursive: true, force: true })
  })
  for (const file of fixtureFiles) {
    const destination = join(root, file)
    mkdirSync(dirname(destination), { recursive: true })
    copyFileSync(join(sourceRoot, file), destination)
  }
  return {
    change(file, mutate) {
      const target = join(root, file)
      writeFileSync(target, mutate(readFileSync(target, 'utf8').replace(/\r\n/g, '\n')), 'utf8')
    },
    check() {
      const result = spawnSync(process.execPath, [join(root, 'scripts/check-release-pipeline.mjs')],
        { cwd: root, encoding: 'utf8', timeout: 10000 })
      assert.equal(result.error, undefined)
      return result
    },
  }
}

test('current release and CI satisfy the structural guard', (t) => {
  const result = fixture(t).check()
  assert.equal(result.status, 0, result.stderr)
  assert.match(result.stdout, /static checks passed/)
})

test('removing the loopback bind is rejected', (t) => {
  const current = fixture(t)
  current.change('scripts/smoke-native-release.sh', (text) =>
    text.replace('-e READER_SERVER_BINDADDRESS=127.0.0.1', ''))
  const result = current.check()
  assert.notEqual(result.status, 0)
  assert.match(result.stderr, /READER_SERVER_BINDADDRESS/)
})

test('removing the release resource assertion is rejected', (t) => {
  const current = fixture(t)
  current.change('scripts/smoke-native-release.sh', (text) =>
    text.replace('scripts/report-browser-cgroup.py', 'scripts/omitted-budget.py'))
  const result = current.check()
  assert.notEqual(result.status, 0)
  assert.match(result.stderr, /report-browser-cgroup/)
})

test('exporting the native image before the browser and resource checks is rejected', (t) => {
  const current = fixture(t)
  current.change('.github/workflows/release-native.yml', (text) =>
    text.replace('          docker image inspect "reader-pro:camoufox-smoke-$arch" > exported/image-inspect.json', '')
      .replace('      - name: Run generated browser contracts under the release resource budget',
        '      - name: Incorrect early export\n        run: docker save untested-image\n      - name: Run generated browser contracts under the release resource budget'))
  const result = current.check()
  assert.notEqual(result.status, 0)
  assert.match(result.stderr, /before exporting an image/)
})

test('not running the release guard in ordinary CI is rejected', (t) => {
  const current = fixture(t)
  current.change('.github/workflows/ci.yml', (text) =>
    text.replace('node scripts/check-release-pipeline.mjs', 'node scripts/omitted-release-guard.mjs'))
  const result = current.check()
  assert.notEqual(result.status, 0)
  assert.match(result.stderr, /CI and formal release/)
})

for (const [name, file, mutate, expected] of [
  ['old Vue 2 image default', 'deploy/reader-pro/Dockerfile', text => text.replace('READER_APP_WEBUI=vue3', 'READER_APP_WEBUI=vue2'), /default to Vue 3/],
  ['a comment cannot impersonate default Vue 3', 'deploy/reader-pro/Dockerfile', text => text.replace('READER_APP_WEBUI=vue3', 'READER_APP_WEBUI=vue2') + '\n# READER_APP_WEBUI=vue3\n', /default to Vue 3/],
  ['Compose without explicit legacy UI rollback', 'deploy/reader-pro/compose.production.yaml', text => text.replace('READER_APP_WEBUI: ${READER_APP_WEBUI:-vue3}', 'READER_APP_WEBUI: vue3'), /Vue 2 rollback/],
  ['native default UI observation omitted', 'scripts/smoke-native-release.sh', text => text.replace('python3 scripts/verify-reader-default-ui.py probe', 'echo omitted-default-ui-probe'), /actual default Vue 3/],
  ['full image default UI guard omitted', '.github/workflows/browser-image.yml', text => text.replace('python3 scripts/verify-reader-default-ui.py check', 'echo omitted-default-ui-check'), /actual default Vue 3/],
  ['smoke-only Vue 3 override hides old image default', 'scripts/smoke-native-release.sh', text => text.replace('-e READER_APP_WORKDIR=/', '-e READER_APP_WEBUI=vue3 -e READER_APP_WORKDIR=/'), /without a smoke-only UI override/],
  ['default UI failures ignored', 'scripts/smoke-native-release.sh', text => text.replace('--expected-jar-sha "$expected_jar"', '--expected-jar-sha "$expected_jar" || true'), /fail closed/],
  ['publisher default UI evidence omitted', 'scripts/import-native-release.sh', text => text.replace('python3 scripts/verify-reader-default-ui.py check', 'echo omitted-publisher-default-ui'), /survive native transfer/],
  ['default UI observations missing from export', '.github/workflows/release-native.yml', text => text.replace('exported/DEFAULT_UI.json', 'exported/omitted-default-ui.json'), /survive native transfer/],
  ['native async Reader guard omitted', 'scripts/smoke-native-release.sh', text => text.replace('python3 scripts/verify-reader-async-smoke.py', 'echo omitted-native-async-reader'), /actual async Reader API/],
  ['publisher async Reader guard omitted', 'scripts/import-native-release.sh', text => text.replace('python3 scripts/verify-reader-async-smoke.py', 'echo omitted-publisher-async-reader'), /actual async Reader API/],
  ['full image async Reader guard omitted', '.github/workflows/browser-image.yml', text => text.replace('python3 scripts/verify-reader-async-smoke.py', 'echo omitted-image-async-reader'), /actual async Reader API/],
  ['native metadata Reader guard omitted', 'scripts/smoke-native-release.sh', text => text.replace('python3 scripts/verify-reader-metadata-smoke.py', 'echo omitted-native-metadata-reader'), /actual browser-backed metadata Reader API/],
  ['publisher metadata Reader guard omitted', 'scripts/import-native-release.sh', text => text.replace('python3 scripts/verify-reader-metadata-smoke.py', 'echo omitted-publisher-metadata-reader'), /actual browser-backed metadata Reader API/],
  ['full image metadata Reader guard omitted', '.github/workflows/browser-image.yml', text => text.replace('python3 scripts/verify-reader-metadata-smoke.py', 'echo omitted-image-metadata-reader'), /actual browser-backed metadata Reader API/],
  ['native metadata Reader failures ignored', 'scripts/smoke-native-release.sh', text => text.replace('verify-reader-metadata-smoke.py "$output/BROWSER_SYNTHETIC.json"', 'verify-reader-metadata-smoke.py "$output/BROWSER_SYNTHETIC.json" || true'), /fail closed.*actual browser-backed metadata Reader API/],
  ['publisher metadata Reader failures ignored', 'scripts/import-native-release.sh', text => text.replace('verify-reader-metadata-smoke.py "$report_directory/BROWSER_SYNTHETIC.json"', 'verify-reader-metadata-smoke.py "$report_directory/BROWSER_SYNTHETIC.json" || true'), /fail closed.*actual browser-backed metadata Reader API/],
  ['full image metadata Reader failures ignored', '.github/workflows/browser-image.yml', text => text.replace('verify-reader-metadata-smoke.py "$RUNNER_TEMP/bundled-browser-synthetic.json"', 'verify-reader-metadata-smoke.py "$RUNNER_TEMP/bundled-browser-synthetic.json" || true'), /fail closed.*actual browser-backed metadata Reader API/],
  ['boolean release assertion instead of the observed identity object', 'scripts/smoke-native-release.sh', text => text.replace('select(.version == $version and .buildRevision == $revision)', '.version == $version and .buildRevision == $revision'), /publisher artifact identity validation/],
  ['publisher omitting the runtime identity report validation', 'scripts/import-native-release.sh', text => text.replace('type == "object" and .version == $version and .buildRevision == $revision', 'true'), /publisher artifact identity validation/],
  ['publisher rehearsal without the real transferred evidence dependency', '.github/workflows/release-native.yml', text => text.replace('needs: [build-jar, native-images, verify-transferred-images]', 'needs: [build-jar, native-images]'), /real fresh-runner evidence/],
  ['mixed report directories that introduce an unexpected artifact root', '.github/workflows/release-native.yml', text => text.replace('path: transferred/', 'path: |\n            imported/metadata.json\n            transferred/IMAGE_IDENTITY.json'), /single flat artifact root/],
  ['missing hash comparison for the JAR actually running inside the image', 'scripts/smoke-native-release.sh', text => text.replace('test "$actual_jar" = "$expected_jar"', 'echo unchecked'), /actual_jar/],
  ['missing native ARM64 runner', '.github/workflows/release-native.yml', text => text.replaceAll('runner: ubuntu-24.04-arm', 'runner: ubuntu-24.04'), /both native hosted runners/],
  ['missing all-native completion dependency', '.github/workflows/release.yml', text => text.replace('needs: [verify-release-inputs, verify-vue3-e2e, build-native-images]', 'needs: [verify-release-inputs, verify-vue3-e2e]'), /both native builds/],
  ['post-smoke image rebuild', '.github/workflows/release.yml', text => text.replace('          for arch in amd64 arm64; do', '          docker build -t replacement .\n          for arch in amd64 arm64; do'), /never rebuild/],
  ['missing loaded-image identity check', 'scripts/import-native-release.sh', text => text.replace('node scripts/release-native-artifacts.mjs loaded ', 'node scripts/omitted-loaded.mjs loaded '), /before registry login/],
  ['missing Docker Hub manifest check', '.github/workflows/release.yml', text => text.replace('node scripts/verify-release-manifest.mjs dist/DOCKERHUB_IMAGE_INDEX.json', 'node scripts/omitted-manifest.mjs dist/DOCKERHUB_IMAGE_INDEX.json'), /both registry manifests/],
  ['registry login in the rehearsal', '.github/workflows/release-native.yml', text => text + '\n# docker/login-action is an invalid rehearsal dependency\n', /must not use registry credentials/],
  ['round-trip check without actually running the reloaded image', '.github/workflows/release-native.yml', text => text.replace('      - name: Actually run the reloaded image with fresh generated accounts and storage\n        run: bash scripts/smoke-native-release.sh', '      - name: Incorrectly omit the reloaded runtime check\n        run: echo skipped'), /actually run the same image/],
  ['default-engine runtime contracts omitted before JAR export', '.github/workflows/release-native.yml', text => text.replace("./gradlew -PreaderWebUi=vue3 test --tests 'com.htmake.reader.utils.CamoufoxWebviewRendererTest'", 'echo omitted-real-default-contracts'), /before actual default-engine contracts/],
  ['default-engine XML verifier omitted', '.github/workflows/release-native.yml', text => text.replace('python3 scripts/verify-camoufox-contracts.py', 'echo omitted-report-verifier'), /before actual default-engine contracts/],
  ['packaged worker identity omitted after runtime tests', '.github/workflows/release-native.yml', text => text.replace('cmp <(unzip -p', 'echo omitted-worker-check <(unzip -p'), /before actual default-engine contracts/],
  ['unlocked default-engine Python inputs', '.github/workflows/release-native.yml', text => text.replace('--require-hashes', ''), /default-engine gate missing required token/],
  ['browser integration using a different report guard', '.github/workflows/browser-image.yml', text => text.replace('python3 scripts/verify-camoufox-contracts.py', 'echo omitted-browser-report-guard'), /share the strict default-engine report verifier/],
  ['default-engine gate treating failures as optional', '.github/workflows/release-native.yml', text => text.replace('      - name: Verify packaged default engine contracts before exporting the shared JAR\n', '      - name: Verify packaged default engine contracts before exporting the shared JAR\n        continue-on-error: true\n'), /must fail closed/],
  ['default-engine gate silently skipped', '.github/workflows/release-native.yml', text => text.replace('      - name: Verify packaged default engine contracts before exporting the shared JAR\n', '      - name: Verify packaged default engine contracts before exporting the shared JAR\n        if: false\n'), /must fail closed/],
]) {
  test('rejects ' + name, (t) => {
    const current = fixture(t)
    current.change(file, mutate)
    const result = current.check()
    assert.notEqual(result.status, 0)
    assert.match(result.stderr, expected)
  })
}
