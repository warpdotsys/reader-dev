#!/usr/bin/env node
/** Structural guard for the formal release workflow.
 *
 * It intentionally does not contact registries or production. GitHub Actions
 * performs those checks after a real stable release tag exists.
 */
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(fileURLToPath(new URL('..', import.meta.url)))
const read = (path) => readFileSync(resolve(root, path), 'utf8').replace(/\r\n/g, '\n')
const workflow = read('.github/workflows/release.yml')
const nativeWorkflow = read('.github/workflows/release-native.yml')
const nativeSmoke = read('scripts/smoke-native-release.sh')
const nativeImporter = read('scripts/import-native-release.sh')
const buildContract = nativeWorkflow + '\n' + nativeSmoke
const compose = read('deploy/reader-pro/compose.production.yaml')
const dockerfile = read('deploy/reader-pro/Dockerfile')
const baseImagesLock = read('deploy/reader-pro/base-images.lock')

for (const token of [
  'verify-release-inputs', 'verify-vue3-e2e', 'build-and-publish-images', 'deploy-production',
  'publish-github-release', 'origin/legacy', 'GHCR_IMAGE', 'DOCKERHUB_IMAGE',
  'DOCKERHUB_USERNAME', 'DOCKERHUB_PASSWORD', 'docker push',
  'docker buildx imagetools inspect', 'READER_IMAGE', 'https://read.medwarp.cn',
  'state-before-${RELEASE_TAG}.tar', 'rescue-after-failure-${RELEASE_TAG}.tar',
  'test "$PROD_SSH_HOST" = cdn.medwarp.cn',
  'READER_IMAGE="$rollback_image"', 'scripts/verify_camoufox_wheel_lock.py',
  'KexAlgorithms=diffie-hellman-group14-sha256', 'IdentitiesOnly=yes',
  'StrictHostKeyChecking=yes', 'snapshot_ready=false', 'snapshot_ready=true',
  "grep -Eq '^##[[:space:]]+已知问题'", 'actions/setup-node@49933ea5288caeca8642d1e84afbd3f7d6820020',
  'reader-anonymous-docker-config',
]) {
  if (!workflow.includes(token)) throw new Error(`release workflow missing required token: ${token}`)
}
for (const token of ['v*-restored', 'macos-15', 'reader-4.0.7']) {
  if ((workflow + nativeWorkflow).includes(token)) throw new Error(`release workflow contains forbidden legacy token: ${token}`)
}
for (const token of [
  'npm --prefix web-vue3 run build', 'Read and validate locked base image digests',
  'base-images.lock', 'REQUIRE_WHEEL_HASHES=true', 'fc-list :lang=zh family',
  'READER_SERVER_BINDADDRESS=127.0.0.1', '--tmpfs /tmp:size=256m,mode=1777',
  'reader-release-listener.txt', 'scripts/report-browser-cgroup.py',
  'BROWSER_RESOURCE_BUDGET.json', 'READER_BUILD_REVISION=${GITHUB_SHA}',
  'sha256sum /app/reader.jar', 'test "$actual_jar" = "$expected_jar"',
]) {
  if (!buildContract.includes(token)) throw new Error(`native build missing release safety token: ${token}`)
}
for (const token of [
  'storage_bytes * 3 + 2147483648', 'Insufficient rollback space',
  'assets/reader-release.json', 'rescue-after-public-failure-${RELEASE_TAG}.tar',
  'expected-image-ref', "{{.Config.Image}}",
]) {
  if (!workflow.includes(token)) throw new Error(`release workflow missing production safety token: ${token}`)
}
const approvedActions = new Map([
  ['actions/checkout', '11bd71901bbe5b1630ceea73d27597364c9af683'],
  ['actions/setup-node', '49933ea5288caeca8642d1e84afbd3f7d6820020'],
  ['actions/setup-java', 'cf277c60eb25467037889841efdb72551f06f6c3'],
  ['actions/setup-python', 'ece7cb06caefa5fff74198d8649806c4678c61a1'],
  ['docker/setup-buildx-action', '8d2750c68a42422c14e847fe6c8ac0403b4cbd6f'],
  ['docker/login-action', 'c94ce9fb468520275223c153574b00df6fe4bcc9'],
  ['actions/upload-artifact', 'ea165f8d65b6e75b540449e92b4886f43607fa02'],
  ['actions/download-artifact', 'd3f86a106a0bac45b974a628896c90dbdf5c8093'],
])
for (const [, action, ref] of (workflow + nativeWorkflow).matchAll(/^\s*(?:-\s+)?uses:\s+([^@\s]+)@([^\s#]+)/gm)) {
  const expected = approvedActions.get(action)
  if (!expected || ref !== expected || !/^[0-9a-f]{40}$/.test(ref)) {
    throw new Error(`release workflow action is not an approved full SHA: ${action}@${ref}`)
  }
}
if (!compose.includes('image: ${READER_IMAGE:?READER_IMAGE must be an immutable image reference}')) {
  throw new Error('production compose must require immutable READER_IMAGE')
}
if (!compose.includes('READER_APP_WEBVIEWRENDERER: camoufox')) {
  throw new Error('production compose must select the in-image Camoufox renderer')
}
if (!compose.includes('init: true')) {
  throw new Error('production compose must reap browser child processes with Docker init')
}
if (!dockerfile.includes('ARG READER_JAR') || !dockerfile.includes('COPY ${READER_JAR} /app/reader.jar')) {
  throw new Error('Dockerfile must receive versioned JAR through READER_JAR')
}
if (!dockerfile.includes('ARG TEMURIN_JRE_IMAGE\n') || !dockerfile.includes('ARG PLAYWRIGHT_PYTHON_IMAGE\n') ||
    !dockerfile.includes('ARG REQUIRE_WHEEL_HASHES=true') || !dockerfile.includes('--require-hashes')) {
  throw new Error('Dockerfile must accept digest-pinned bases and verify wheels by default')
}
for (const token of ['docker-entrypoint.sh /usr/local/bin/reader-entrypoint', 'READER_RELEASE_VERSION=${READER_VERSION}', 'ENTRYPOINT ["/usr/local/bin/reader-entrypoint"]']) {
  if (!dockerfile.includes(token)) throw new Error(`Dockerfile missing reproducibility/release proof token: ${token}`)
}
if (dockerfile.includes('apt-get') || dockerfile.includes('COPY apt-sources.list')) {
  throw new Error('Dockerfile must not require live apt repositories during the image build')
}
for (const name of ['TEMURIN_JRE_IMAGE', 'PLAYWRIGHT_PYTHON_IMAGE']) {
  if (!new RegExp(`^${name}=.+@sha256:[0-9a-f]{64}$`, 'm').test(baseImagesLock)) {
    throw new Error(`base image lock missing immutable ${name}`)
  }
}
const browserWorkflow = read('.github/workflows/browser-image.yml')
const imageUiSelections = [...dockerfile.matchAll(/^\s+READER_APP_WEBUI=(\w+)\s*\\\s*$/gm)].map(match => match[1])
if (JSON.stringify(imageUiSelections) !== JSON.stringify(['vue3']) ||
    !compose.includes('READER_APP_WEBUI: ${READER_APP_WEBUI:-vue3}')) {
  throw new Error('complete image must default to Vue 3 with an explicit Vue 2 rollback selector')
}
const defaultUiProbe = 'python3 scripts/verify-reader-default-ui.py probe'
const defaultUiCheck = 'python3 scripts/verify-reader-default-ui.py check'
for (const [label, content] of [['native', nativeSmoke], ['full image', browserWorkflow]]) {
  const probe = content.indexOf(defaultUiProbe)
  const check = content.indexOf(defaultUiCheck)
  const browser = content.indexOf('python3 scripts/smoke-local-webview.py', probe)
  if (probe < 0 || check <= probe || browser <= check ||
      /-e\s+["']?READER_APP_WEBUI=/.test(content) ||
      /\|\|\s*(?:true|:)/.test(content.slice(probe, browser))) {
    throw new Error(`${label} must fail closed on actual default Vue 3 static bytes without a smoke-only UI override`)
  }
}
if (!nativeImporter.includes(defaultUiCheck + ' "$report_directory/DEFAULT_UI.json"') ||
    !nativeImporter.includes('--expected-jar-sha "$jar_sha"') ||
    !nativeWorkflow.includes('exported/DEFAULT_UI.json') ||
    !nativeWorkflow.includes('dist/*-DEFAULT_UI.json') ||
    !browserWorkflow.includes('${{ runner.temp }}/bundled-default-ui.json')) {
  throw new Error('default UI observations must survive native transfer and bind publisher checks to the shared JAR')
}
if (!browserWorkflow.includes('fc-list :lang=zh family')) {
  throw new Error('browser image smoke test must verify CJK font coverage')
}
const ciWorkflow = read('.github/workflows/ci.yml')
if (!ciWorkflow.includes('node scripts/check-release-pipeline.mjs') ||
    !workflow.includes('node scripts/check-release-pipeline.mjs')) {
  throw new Error('CI and formal release must execute release safety checks')
}
const section = (text, start, end) => {
  const begin = text.indexOf('\n  ' + start + ':\n')
  const finish = end ? text.indexOf('\n  ' + end + ':\n', begin) : text.length
  if (begin < 0 || finish <= begin) throw new Error(`missing workflow job: ${start}`)
  return text.slice(begin, finish)
}
const nativeBuild = section(nativeWorkflow, 'native-images', 'verify-transferred-images')
const sharedJar = section(nativeWorkflow, 'build-jar', 'native-images')
const nativeTransfer = section(nativeWorkflow, 'verify-transferred-images', 'verify-publisher-imports')
const publisherRehearsal = section(nativeWorkflow, 'verify-publisher-imports')
const publisher = section(workflow, 'build-and-publish-images', 'deploy-production')
const defaultGate = sharedJar.indexOf('- name: Verify packaged default engine contracts before exporting the shared JAR')
const realDefaultRun = sharedJar.indexOf("./gradlew -PreaderWebUi=vue3 test --tests 'com.htmake.reader.utils.CamoufoxWebviewRendererTest'")
const defaultReportCheck = sharedJar.indexOf('python3 scripts/verify-camoufox-contracts.py')
const defaultWorkerCompare = sharedJar.indexOf('cmp <(unzip -p')
const jarExport = sharedJar.indexOf('name: reader-release-jar-${{ github.sha }}')
if (defaultGate < 0 || realDefaultRun < defaultGate || defaultReportCheck < realDefaultRun ||
    defaultWorkerCompare < defaultReportCheck || jarExport < defaultWorkerCompare) {
  throw new Error('shared release JAR must not be exported before actual default-engine contracts and packaged worker identity pass')
}
const defaultStepEnd = sharedJar.indexOf('\n      - name:', defaultGate + 1)
const defaultStep = sharedJar.slice(defaultGate, defaultStepEnd)
if (!defaultStep.includes('set -euo pipefail') ||
    /^\s*(?:if:|continue-on-error:)/m.test(defaultStep) || /\|\|\s*(?:true|:)/.test(defaultStep)) {
  throw new Error('default-engine release gate must fail closed without skip conditions or ignored errors')
}
for (const token of [
  'actions/setup-python@ece7cb06caefa5fff74198d8649806c4678c61a1', "python-version: '3.10'",
  '--require-hashes', 'deploy/reader-pro/install_camoufox_pinned.py',
  "READER_CAMOUFOX_BROWSER_VERSION: '152.0.4-beta.30'", 'READER_CAMOUFOX_PYTHON:',
  'name: reader-release-camoufox-contract-${{ github.sha }}', 'if: always()', 'if-no-files-found: error',
]) {
  if (!sharedJar.includes(token)) throw new Error(`release default-engine gate missing required token: ${token}`)
}
if (!browserWorkflow.includes('python3 scripts/verify-camoufox-contracts.py')) {
  throw new Error('browser integration and native release must share the strict default-engine report verifier')
}
for (const token of [
  'workflow_call:', 'reader-release-jar-${{ github.sha }}',
  'reader-tested-native-${{ matrix.arch }}-${{ github.sha }}',
  'reader-native-transfer-${{ matrix.arch }}-${{ github.sha }}',
  'amd64:X64:x86_64|arm64:ARM64:aarch64',
]) {
  if (!nativeWorkflow.includes(token)) throw new Error(`native release missing artifact or native-runner contract: ${token}`)
}
for (const content of [nativeBuild, nativeTransfer]) {
  for (const token of ['arch: amd64', 'arch: arm64', 'runner: ubuntu-24.04\n', 'runner: ubuntu-24.04-arm\n']) {
    if (!content.includes(token)) throw new Error(`both native hosted runners are required: ${token}`)
  }
}
if (!nativeTransfer.includes('needs: [build-jar, native-images]') ||
    !workflow.includes('uses: ./.github/workflows/release-native.yml') ||
    !publisher.includes('needs: [verify-release-inputs, verify-vue3-e2e, build-native-images]')) {
  throw new Error('registry writes must wait for both native builds and both fresh-runner round trips')
}
for (const token of ['self-hosted', 'setup-qemu', 'docker push', 'docker/login-action', 'packages: write', '${{ secrets.']) {
  if (nativeWorkflow.includes(token)) throw new Error(`native rehearsal must not use registry credentials or non-native execution: ${token}`)
}
const smokeCall = nativeBuild.indexOf('bash scripts/smoke-native-release.sh')
const archiveExport = nativeBuild.indexOf('docker save ')
const browserSmoke = nativeSmoke.indexOf('python3 scripts/smoke-local-webview.py')
const budgetCheck = nativeSmoke.indexOf('scripts/report-browser-cgroup.py')
const asyncReaderCheck = nativeSmoke.indexOf('python3 scripts/verify-reader-async-smoke.py')
if (asyncReaderCheck < browserSmoke || asyncReaderCheck >= budgetCheck ||
    !browserWorkflow.includes('python3 scripts/verify-reader-async-smoke.py') ||
    !nativeImporter.includes('python3 scripts/verify-reader-async-smoke.py "$report_directory/BROWSER_SYNTHETIC.json"')) {
  throw new Error('native, full-image, and publisher stages must verify actual async Reader API observations')
}
for (const [label, content, invocation] of [
  ['native', nativeSmoke, 'python3 scripts/verify-reader-metadata-smoke.py "$output/BROWSER_SYNTHETIC.json"'],
  ['full image', browserWorkflow, 'python3 scripts/verify-reader-metadata-smoke.py "$RUNNER_TEMP/bundled-browser-synthetic.json"'],
  ['publisher', nativeImporter, 'python3 scripts/verify-reader-metadata-smoke.py "$report_directory/BROWSER_SYNTHETIC.json"'],
]) {
  const exact = content.split('\n').filter(line => line.trim() === invocation)
  if (exact.length !== 1) throw new Error(`${label} must fail closed on actual browser-backed metadata Reader API observations`)
}
const metadataReaderCheck = nativeSmoke.indexOf('python3 scripts/verify-reader-metadata-smoke.py')
if (metadataReaderCheck <= asyncReaderCheck || metadataReaderCheck >= budgetCheck) {
  throw new Error('actual browser-backed metadata Reader API observations must precede resource checks and image export')
}
if (smokeCall < 0 || archiveExport < smokeCall || browserSmoke < 0 || budgetCheck < browserSmoke ||
    !nativeBuild.includes('node scripts/release-native-artifacts.mjs record ')) {
  throw new Error('native browser and resource checks must finish before exporting an image for registry writes')
}
const transferredCheck = nativeTransfer.indexOf('node scripts/release-native-artifacts.mjs check ')
const transferredLoad = nativeTransfer.indexOf('docker load --input')
const transferredIdentity = nativeTransfer.indexOf('node scripts/release-native-artifacts.mjs loaded ')
const transferredRun = nativeTransfer.indexOf('bash scripts/smoke-native-release.sh')
if (transferredCheck < 0 || transferredLoad < transferredCheck || transferredIdentity < transferredLoad || transferredRun < transferredIdentity) {
  throw new Error('fresh native runners must verify archive bytes, load image identity, and actually run the same image')
}
if (!nativeTransfer.includes('cp imported/metadata.json transferred/metadata.json') ||
    !nativeTransfer.includes('path: transferred/\n')) {
  throw new Error('round-trip evidence must have a single flat artifact root matching the formal publisher')
}
const registryLogin = publisher.indexOf('- name: Log in to GitHub Container Registry')
const registryPush = publisher.indexOf('docker push ')
for (const token of ['reader-tested-native-amd64-${{ github.sha }}', 'reader-tested-native-arm64-${{ github.sha }}']) {
  if (!publisher.includes(token)) throw new Error(`publisher must consume same-commit tested native artifacts: ${token}`)
}
for (const arch of ['amd64', 'arm64']) {
  const call = 'bash scripts/import-native-release.sh ' + arch + ' '
  const calls = [...publisher.matchAll(new RegExp(call.replaceAll('.', '\\.'), 'g'))]
  if (calls.length !== 1 || calls[0].index >= registryLogin || registryPush < registryLogin) {
    throw new Error('both native artifact checks and loaded identities must pass before registry login and push')
  }
  if (!publisherRehearsal.includes(call)) throw new Error('publisher import rehearsal must exercise both same-commit native artifacts')
}
const importCheck = nativeImporter.indexOf('node scripts/release-native-artifacts.mjs check ')
const importLoad = nativeImporter.indexOf('docker load --input ')
const importIdentity = nativeImporter.indexOf('node scripts/release-native-artifacts.mjs loaded ')
if (importCheck < 0 || importLoad < importCheck || importIdentity < importLoad) {
  throw new Error('both native artifact checks and loaded identities must pass before registry login and push')
}
if (!publisherRehearsal.includes('needs: [build-jar, native-images, verify-transferred-images]') ||
    !nativeImporter.includes('cmp "$evidence/metadata.json" "$directory/metadata.json"')) {
  throw new Error('publisher import rehearsal must consume both real fresh-runner evidence artifacts')
}
if (!nativeSmoke.includes('select(.version == $version and .buildRevision == $revision)') ||
    !nativeImporter.includes('for report_directory in "$directory" "$evidence"; do') ||
    !nativeImporter.includes('type == "object" and .version == $version and .buildRevision == $revision') ||
    !nativeImporter.includes('type == "object" and .architecture == $arch and .revision == $revision and .jarSha256 == $jar_sha')) {
  throw new Error('publisher artifact identity validation must retain and check actual runtime objects, not boolean placeholders')
}
if (/^\s*docker (?:build|buildx build)\b/m.test(publisher + '\n' + nativeImporter)) {
  throw new Error('publisher must never rebuild an image after native smoke tests')
}
for (const token of [
  'docker buildx imagetools create --tag', 'dist/NATIVE_DIGEST-amd64', 'dist/NATIVE_DIGEST-arm64',
  'node scripts/verify-release-manifest.mjs dist/GHCR_IMAGE_INDEX.json',
  'node scripts/verify-release-manifest.mjs dist/DOCKERHUB_IMAGE_INDEX.json',
]) {
  if (!publisher.includes(token)) throw new Error(`both registry manifests must match both tested native images: ${token}`)
}
for (const [name, content] of [['native release', nativeWorkflow], ['browser image', browserWorkflow], ['CI', ciWorkflow]]) {
  if (!content.includes("unittest discover -s src/test/python -p '*_test.py'")) {
    throw new Error(`${name} workflow must execute browser and differential CLI safety tests`)
  }
}
const vue3Workflow = read('.github/workflows/vue3-preview.yml')
if (!vue3Workflow.includes('workflow_call:') ||
    !workflow.includes('uses: ./.github/workflows/vue3-preview.yml') ||
    !workflow.includes('needs: [verify-release-inputs, verify-vue3-e2e]')) {
  throw new Error('release images must wait for the same-commit Vue 3 browser journeys')
}
for (const [name, content] of [['browser image', browserWorkflow], ['CI', ciWorkflow]]) {
  if (!content.includes('npm run build --prefix web-vue3') || !content.includes('-PreaderWebUi=vue3')) {
    throw new Error(`${name} workflow must build and package Vue 3`)
  }
}
if (browserWorkflow.match(/reader-4\.0\.7\.jar/) || ciWorkflow.match(/reader-4\.0\.7\.jar/) || vue3Workflow.match(/reader-4\.0\.7\.jar/)) {
  throw new Error('versioned workflows must not hard-code reader-4.0.7.jar')
}
const imageRead = workflow.indexOf("previous_image=$(docker inspect")
const trapRegistration = workflow.indexOf('trap rollback ERR')
const serviceStop = workflow.indexOf('if [ "$initial_deploy" = false ]; then docker stop reader-pro-restored; fi')
if (imageRead < 0 || trapRegistration < imageRead || serviceStop < trapRegistration) {
  throw new Error('release workflow must read prior image and register rollback before stopping a service')
}
for (const token of [
  'initial_deploy=false', 'docker image tag "$previous_image" "$rollback_image"',
  "{{.State.Running}}", 'rescue_ok=true', 'if [ "$rescue_ok" = false ]; then return 1; fi',
  'Failed to archive the fault state; restoring the already verified pre-deploy snapshot.',
]) {
  if (!workflow.includes(token)) throw new Error(`release workflow missing rollback branch guard: ${token}`)
}
console.log('release pipeline static checks passed')
