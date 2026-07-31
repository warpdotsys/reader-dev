/**
 * Welcome to your Workbox-powered service worker!
 *
 * You'll need to register this file in your web app and you should
 * disable HTTP caching for this file too.
 * See https://goo.gl/nhQhGp
 *
 * The rest of the code is auto-generated. Please don't update this file
 * directly; instead, make changes to your Workbox build configuration
 * and re-run your build process.
 * See https://goo.gl/2aRDsh
 */

importScripts("https://storage.googleapis.com/workbox-cdn/releases/4.3.1/workbox-sw.js");

importScripts(
  "sw.js",
  "precache-manifest.0d903434eaa73f94acefeef5d39c6628.js"
);

workbox.core.setCacheNameDetails({prefix: "reader"});

self.addEventListener('message', (event) => {
  if (event.data && event.data.type === 'SKIP_WAITING') {
    self.skipWaiting();
  }
});

/**
 * The workboxSW.precacheAndRoute() method efficiently caches and responds to
 * requests for URLs in the manifest.
 * See https://goo.gl/S9QRab
 */
self.__precacheManifest = [].concat(self.__precacheManifest || []);
workbox.precaching.precacheAndRoute(self.__precacheManifest, {});

workbox.precaching.cleanupOutdatedCaches();

workbox.routing.registerRoute(/^https?:\/\/[^/]*\/?$/, new workbox.strategies.NetworkFirst({ "cacheName":"home", plugins: [new workbox.cacheableResponse.Plugin({ statuses: [ 200 ] })] }), 'GET');
workbox.routing.registerRoute(/^https?:\/\/[^/]*\/reader3\/cover/, new workbox.strategies.CacheFirst({ "cacheName":"bookCover", plugins: [{ generateCacheKey: ({ request }) => { const searchParams = new URL(request.url).searchParams; return searchParams.get("path"); }, checkResponse: ({ response }) => { if (response.status === 200) { return response; } return null; }, cacheWillUpdate: async function cacheWillUpdate({ request, response, event, state }) { const resCopy = response.clone(); if (this.checkResponse) { return await this.checkResponse({ request, response: resCopy, event, state }); } const body = await resCopy.json().catch(() => false); if (body && body.isSuccess) { return response; } else { return null; } }, cacheKeyWillBeUsed: async function cacheKeyWillBeUsed({ request, mode, params, event, state }) { if (this.generateCacheKey) { const cacheKey = this.generateCacheKey({ request, mode, params, event, state }); return cacheKey || request; } else { return request; } } }, new workbox.cacheableResponse.Plugin({ statuses: [ 200 ] }), new workbox.expiration.Plugin({ maxAgeSeconds: 2592000, maxEntries: 1000, purgeOnQuotaError: false })] }), 'GET');
