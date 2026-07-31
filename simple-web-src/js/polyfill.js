/**
 * polyfill.js - Promise Polyfill Detection
 *
 * This is a minimal browserify bundle stub that checks for Promise support.
 * In practice, modern browsers already have Promise, so this file is essentially
 * a no-op placeholder that ensures the module system bootstrap runs.
 */
(function browserifyBundle(modules, cache, entries) {
    /**
     * Require function for the browserify module system.
     * Loads and caches modules by their numeric ID.
     */
    function require(id, fromParent) {
        if (!cache[id]) {
            if (!modules[id]) {
                var nativeRequire = typeof require === 'function' && require;
                if (!fromParent && nativeRequire) {
                    return nativeRequire(id, true);
                }
                if (globalRequire) {
                    return globalRequire(id, true);
                }
                var error = new Error("Cannot find module '" + id + "'");
                error.code = 'MODULE_NOT_FOUND';
                throw error;
            }

            var module = cache[id] = { exports: {} };
            modules[id][0].call(
                module.exports,
                function (dep) {
                    return require(modules[id][1][dep] || dep);
                },
                module,
                module.exports,
                browserifyBundle,
                modules,
                cache,
                entries
            );
        }
        return cache[id].exports;
    }

    var globalRequire = typeof require === 'function' && require;

    for (var i = 0; i < entries.length; i++) {
        require(entries[i]);
    }

    return require;
})(
    {
        // Module 1: empty module (Promise polyfill not needed in modern environments)
        1: [function (require, module, exports) {
            // No-op: Promise is natively available
        }, {}]
    },
    {},
    [1]
);
