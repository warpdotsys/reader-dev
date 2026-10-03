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
      writeFileSync(target, mutate(readFileSync(target, 'utf8')), 'utf8')
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
  current.change('.github/workflows/release.yml', (text) =>
    text.replace('-e READER_SERVER_BINDADDRESS=127.0.0.1', ''))
  const result = current.check()
  assert.notEqual(result.status, 0)
  assert.match(result.stderr, /READER_SERVER_BINDADDRESS/)
})

test('removing the release resource assertion is rejected', (t) => {
  const current = fixture(t)
  current.change('.github/workflows/release.yml', (text) =>
    text.replace('scripts/report-browser-cgroup.py', 'scripts/omitted-budget.py'))
  const result = current.check()
  assert.notEqual(result.status, 0)
  assert.match(result.stderr, /report-browser-cgroup/)
})

test('checking resource budgets only after registry login is rejected', (t) => {
  const current = fixture(t)
  current.change('.github/workflows/release.yml', (text) => {
    const start = text.indexOf('              docker exec -i reader-pro-camoufox-smoke python -')
    const end = text.indexOf('              exit 0', start)
    assert.ok(start >= 0 && end > start)
    const budget = text.slice(start, end)
    const withoutBudget = text.replace(budget, '')
    const position = withoutBudget.indexOf('      - name: Log in to Docker Hub')
    assert.ok(position >= 0)
    const lateStep = '      - name: Incorrectly late resource check\n        run: |\n' +
      budget.split(/\r?\n/).filter(Boolean).map((line) => '          ' + line.trim()).join('\n') + '\n'
    return withoutBudget.slice(0, position) + lateStep + withoutBudget.slice(position)
  })
  const result = current.check()
  assert.notEqual(result.status, 0)
  assert.match(result.stderr, /before registry writes/)
})

test('not running the release guard in ordinary CI is rejected', (t) => {
  const current = fixture(t)
  current.change('.github/workflows/ci.yml', (text) =>
    text.replace('node scripts/check-release-pipeline.mjs', 'node scripts/omitted-release-guard.mjs'))
  const result = current.check()
  assert.notEqual(result.status, 0)
  assert.match(result.stderr, /CI and formal release/)
})
