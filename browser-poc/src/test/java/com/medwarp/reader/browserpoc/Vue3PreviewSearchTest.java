package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import com.microsoft.playwright.Response;
import com.microsoft.playwright.Route;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.UUID;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

/** Exercises multi-source SSE and selected-source search through actual Vue pages. */
public class Vue3PreviewSearchTest {
    @Test
    public void exactSearchThenSelectedSourceSearch() throws Exception {
        String previewUrl = System.getenv("READER_VUE3_PREVIEW_URL");
        String fixtureUrl = System.getenv("READER_BOOK_FIXTURE_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(previewUrl != null && fixtureUrl != null
                && !executable.isEmpty() && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        for (String value : new String[]{previewUrl, fixtureUrl}) {
            URI uri = URI.create(value);
            assertEquals("http", uri.getScheme());
            assertTrue("Only loopback fixtures are allowed", "127.0.0.1".equals(uri.getHost())
                    || "localhost".equals(uri.getHost()));
        }

        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                Page page = browser.newPage();
                page.setDefaultTimeout(30000);
                page.navigate(previewUrl + "/login");
                page.locator(".mode-switch button").nth(1).click();
                page.locator("input[autocomplete=username]").fill("vue" +
                        UUID.randomUUID().toString().replace("-", "").substring(0, 10));
                page.locator("input[autocomplete=current-password]").fill("SearchProbe-2026");
                page.locator(".submit-btn").click();
                page.locator(".bookshelf-page").waitFor();

                byte[] script = getClass().getResourceAsStream("/vue3-reading-setup.js").readAllBytes();
                page.evaluate(new String(script, StandardCharsets.UTF_8),
                        Map.of("base", fixtureUrl, "saveBook", false));
                page.navigate(previewUrl + "/search");
                page.locator(".search-page").waitFor();
                page.locator(".search-input").fill("差分测试书");
                page.locator(".exact-toggle").click();
                page.locator(".search-btn").click();
                page.locator(".result-item").first().waitFor();
                assertTrue(page.locator(".result-name").first().innerText().contains("差分测试书"));
                page.waitForFunction("!document.querySelector('.search-btn').disabled");
                page.locator("select[aria-label='搜索范围']").selectOption(fixtureUrl);
                Response selectedSource = page.waitForResponse(
                        response -> URI.create(response.url()).getPath().endsWith("/reader3/searchBook"),
                        () -> page.locator(".search-btn").click());
                assertTrue(selectedSource.text().contains("\"isSuccess\":true"));
                page.locator(".result-item").first().waitFor();
                assertTrue(page.locator(".result-name").first().innerText().contains("差分测试书"));
                page.locator(".result-item").first().click();
                page.locator(".detail-page").waitFor();
                assertTrue(page.locator(".book-name").innerText().contains("差分测试书"));
                page.locator(".tabs .tab").nth(1).click();
                page.locator(".toc-item").first().waitFor();
                page.locator(".toc-item").first().click();
                try {
                    page.getByText("第一段，中文与 UTF-8。").waitFor(
                            new com.microsoft.playwright.Locator.WaitForOptions().setTimeout(10000));
                } catch (RuntimeException failure) {
                    throw new AssertionError("Unshelved reading at " + page.url() + ": " +
                            page.locator(".reader-page").innerText().substring(0,
                                    Math.min(1000, page.locator(".reader-page").innerText().length())), failure);
                }

                // Force an SSE transport failure: the UI must parse the legacy
                // batch envelope {lastIndex,list}, not treat data as an array.
                page.route("**/reader3/searchBookMultiSSE**", route -> route.fulfill(
                        new Route.FulfillOptions().setStatus(503).setContentType("text/plain")
                                .setBody("fixture SSE unavailable")));
                page.navigate(previewUrl + "/search");
                page.locator(".search-input").fill("差分测试书");
                Response batch = page.waitForResponse(
                        response -> URI.create(response.url()).getPath().endsWith("/reader3/searchBookMulti"),
                        () -> page.locator(".search-btn").click());
                assertTrue(batch.text().contains("\"lastIndex\""));
                page.locator(".result-item").first().waitFor();
                assertTrue(page.locator(".result-name").first().innerText().contains("差分测试书"));

                page.locator("select[aria-label='搜索范围']").selectOption(fixtureUrl);
                page.locator(".search-input").fill(fixtureUrl + "/book");
                page.locator(".url-open-btn").click();
                page.locator(".detail-page").waitFor();
                assertTrue(page.locator(".book-name").innerText().contains("差分测试书"));
            } finally {
                browser.close();
            }
        }
    }
}
