package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import com.microsoft.playwright.PlaywrightException;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.UUID;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

/** Tests the built Vue 3 artifact under /reader/, not only Vite dev at /. */
public class Vue3PreviewSubdirectoryTest {
    @Test
    public void builtUiRetainsRoutesAssetsAndRootApiProxy() {
        String url = System.getenv("READER_VUE3_SUBDIR_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(url != null && !executable.isEmpty()
                && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        URI base = URI.create(url);
        assertEquals("http", base.getScheme());
        assertTrue("Only loopback preview is allowed",
                "127.0.0.1".equals(base.getHost()) || "localhost".equals(base.getHost()));
        assertEquals("/reader", base.getPath());

        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                Page page = browser.newPage();
                page.setDefaultTimeout(30000);
                List<String> staticErrors = new ArrayList<>();
                page.onResponse(response -> {
                    String path = URI.create(response.url()).getPath();
                    if (path.startsWith("/reader/") && response.status() >= 400) {
                        staticErrors.add(path + " HTTP " + response.status());
                    }
                });
                page.onRequestFailed(request -> {
                    String path = URI.create(request.url()).getPath();
                    if (path.startsWith("/reader/static/") || path.startsWith("/reader/fonts/")) {
                        staticErrors.add(path + " failed");
                    }
                });

                page.navigate(url + "/login");
                page.locator(".login-page").waitFor();
                assertTrue((Boolean) page.evaluate("() => {" +
                        "const logo=document.querySelector('img.login-logo');" +
                        "return logo && logo.complete && logo.naturalWidth > 0 &&" +
                        "document.querySelector('link[rel=manifest]').href.endsWith('/reader/manifest.webmanifest');" +
                        "}"));
                @SuppressWarnings("unchecked")
                Map<String, Object> resources = (Map<String, Object>) page.evaluate("async () => {" +
                        "const paths=['/reader/manifest.webmanifest','/reader/fonts/lxgw-wenkai-regular.woff2'," +
                        "'/reader/fonts/source-han-serif-cn-regular.woff2','/reader/logo.svg'];" +
                        "const responses=await Promise.all(paths.map(p=>fetch(p)));" +
                        "const manifest=await responses[0].json();" +
                        "const api=await (await fetch('/reader3/getSystemInfo')).json();" +
                        "return {assets:responses.every(r=>r.ok),start:manifest.start_url," +
                        "scope:manifest.scope,api:api.isSuccess};" +
                        "}" );
                assertEquals(Boolean.TRUE, resources.get("assets"));
                assertEquals(".", resources.get("start"));
                assertEquals(".", resources.get("scope"));
                assertEquals(Boolean.TRUE, resources.get("api"));
                Map<String, Object> worker = null;
                for (int attempt = 0; attempt < 3; attempt++) {
                    try {
                        @SuppressWarnings("unchecked")
                        Map<String, Object> actual = (Map<String, Object>) page.evaluate("async () => {" +
                                "const ready=await Promise.race([navigator.serviceWorker.ready," +
                                "new Promise((_,reject)=>setTimeout(()=>reject(new Error('SW timeout')),10000))]);" +
                                "const keys=await caches.keys();" +
                                "return {scope:new URL(ready.scope).pathname," +
                                "shell:keys.some(k=>k.includes('%2Freader%2F')&&k.endsWith('-shell'))};" +
                                "}");
                        worker = actual;
                        break;
                    } catch (PlaywrightException error) {
                        if (attempt == 2 || !error.getMessage().contains("Execution context was destroyed")) {
                            throw error;
                        }
                        page.locator(".login-page").waitFor();
                    }
                }
                assertEquals("/reader/", worker.get("scope"));
                assertEquals(Boolean.TRUE, worker.get("shell"));

                page.locator(".mode-switch button").nth(1).click();
                page.locator("input[autocomplete=username]").fill("subdir" +
                        UUID.randomUUID().toString().replace("-", "").substring(0, 10));
                page.locator("input[autocomplete=current-password]").fill("SubdirProbe-2026");
                page.locator(".submit-btn").click();
                page.locator(".bookshelf-page").waitFor();
                assertTrue(URI.create(page.url()).getPath().startsWith("/reader/"));

                page.navigate(url + "/search");
                page.locator(".search-page").waitFor();
                page.reload();
                page.locator(".search-page").waitFor();
                assertTrue("Subdirectory static resources failed: " + staticErrors,
                        staticErrors.isEmpty());
            } finally {
                browser.close();
            }
        }
    }
}
