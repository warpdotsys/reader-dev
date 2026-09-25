import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import ts from 'typescript'

// Static inventory, not a claim that a matching route's parameters or response
// have been verified. Only literal first arguments to API calls are counted.
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')
const apiDir = path.join(root, 'web-vue3/src/api')
const backendFile = path.join(root, 'src/main/java/com/htmake/reader/api/YueduApi.kt')
const backend = new Map()
const backendSource = fs.readFileSync(backendFile, 'utf8')
for (const match of backendSource.matchAll(/router\.(get|post|route)\("(\/reader3\/[A-Za-z0-9/_-]+)"\)/g)) {
  const methods = backend.get(match[2]) ?? new Set()
  methods.add(match[1].toUpperCase())
  backend.set(match[2], methods)
}

const seen = new Map()
const dynamicCalls = []
const callers = new Set(['get', 'post', 'fetch'])
for (const filename of fs.readdirSync(apiDir).filter((name) => name.endsWith('.ts') && !name.endsWith('.test.ts'))) {
  const filenameAbs = path.join(apiDir, filename)
  const source = fs.readFileSync(filenameAbs, 'utf8')
  const ast = ts.createSourceFile(filename, source, ts.ScriptTarget.Latest, true)
  const visit = (node) => {
    if (ts.isCallExpression(node) || ts.isNewExpression(node)) {
      const callee = node.expression.getText(ast).split('.').at(-1)
      if (callers.has(callee) || callee === 'EventSource') {
        const first = node.arguments?.[0]
        if (first && (ts.isStringLiteral(first) || ts.isNoSubstitutionTemplateLiteral(first))) {
          const raw = first.text.split('?')[0]
          if (raw.startsWith('/')) {
            const route = raw.startsWith('/reader3/') ? raw : `/reader3${raw}`
            if (/^\/reader3\/[A-Za-z0-9/_-]+$/.test(route)) {
              const refs = seen.get(route) ?? new Set()
              refs.add(filename)
              seen.set(route, refs)
            }
          }
        } else if (first) {
          dynamicCalls.push(`${filename}:${ast.getLineAndCharacterOfPosition(node.getStart(ast)).line + 1}`)
        }
      }
    }
    ts.forEachChild(node, visit)
  }
  visit(ast)
}

const routes = [...seen].sort(([a], [b]) => a.localeCompare(b)).map(([route, refs]) => ({
  route,
  frontendFiles: [...refs].sort(),
  backendMethods: [...(backend.get(route) ?? [])].sort(),
  registeredInYueduApi: backend.has(route),
}))
const report = {
  frontendLiteralRoutes: routes.length,
  registeredInYueduApi: routes.filter((row) => row.registeredInYueduApi).length,
  notRegisteredInYueduApi: routes.filter((row) => !row.registeredInYueduApi),
  dynamicCalls,
  limitations: 'Static route names only; dynamic calls, other controllers, parameters, permissions and response shapes need separate review.',
}
console.log(JSON.stringify(report, null, 2))
