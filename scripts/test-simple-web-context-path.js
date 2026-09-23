const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

const root = path.resolve(__dirname, "..");
const bundleDir = path.join(root, "src", "main", "resources", "simple-web");
const shim = fs.readFileSync(
  path.join(bundleDir, "assets", "js", "context-path-20260923.js"),
  "utf8"
);

for (const page of ["index.html", "reader.html", "search.html", "rss.html"]) {
  const html = fs.readFileSync(path.join(bundleDir, page), "utf8");
  assert.ok(
    html.includes('src="assets/js/common-eebd186870.js"></script>' +
      '<script type="text/javascript" src="assets/js/context-path-20260923.js"></script>'),
    `${page} must load the context-path shim immediately after the common bundle`
  );
}

function run(pathname, options = {}) {
  const ajax = { baseURL: options.initial || "/reader3" };
  const $ = {
    ajax,
    getUrlPra: () => options.queryApi || "",
    cookie: { get: () => options.cookieApi || "" }
  };
  const window = {
    $,
    location: { pathname },
    myStorage: { getItem: () => options.storedApi || "" }
  };
  vm.runInNewContext(shim, { window, $ });
  return ajax.baseURL;
}

assert.equal(run("/simple-web/"), "/reader3");
assert.equal(run("/reader/simple-web/"), "/reader/reader3");
assert.equal(run("/one/two/simple-web/search.html"), "/one/two/reader3");
assert.equal(run("/other/"), "/reader3");

const custom = "https://example.invalid/reader3";
assert.equal(run("/reader/simple-web/", { initial: custom, queryApi: custom }), custom);
assert.equal(run("/reader/simple-web/", { initial: custom, cookieApi: custom }), custom);
assert.equal(run("/reader/simple-web/", { initial: custom, storedApi: custom }), custom);

console.log("simple-web context-path compatibility checks passed");
