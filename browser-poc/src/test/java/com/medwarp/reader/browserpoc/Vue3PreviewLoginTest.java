package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import com.microsoft.playwright.Response;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.UUID;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

/**
 * Opt-in end-to-end probe. Point the Vite preview proxy at an isolated Reader
 * workdir; this test registers a random account and must never target production.
 */
public class Vue3PreviewLoginTest {
    @Test
    public void registerReloadAndLoadEmptyBookshelf() {
        String previewUrl = System.getenv("READER_VUE3_PREVIEW_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(previewUrl != null && !previewUrl.isEmpty()
                && !executable.isEmpty() && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        URI preview = URI.create(previewUrl);
        assertEquals("Preview must be HTTP", "http", preview.getScheme());
        assertTrue("Preview must be loopback", "127.0.0.1".equals(preview.getHost())
                || "localhost".equals(preview.getHost()));

        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(java.util.Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                Page page = browser.newPage();
                page.navigate(previewUrl + "/login");
                page.locator(".mode-switch button").nth(1).click();
                String username = "vue" + UUID.randomUUID().toString().replace("-", "").substring(0, 10);
                page.locator("input[autocomplete=username]").fill(username);
                page.locator("input[autocomplete=current-password]").fill("PreviewProbe-2026");
                page.locator(".submit-btn").click();
                page.locator(".bookshelf-page").waitFor();
                assertEquals(username, page.evaluate("localStorage.getItem('reader_username')"));
                String token = (String) page.evaluate("localStorage.getItem('reader_access_token')");
                assertTrue("Login token must be namespaced to this account", token.startsWith(username + ":"));

                Response shelf = page.waitForResponse(
                        response -> response.url().contains("/reader3/getBookshelf"),
                        page::reload);
                assertEquals(200, shelf.status());
                assertTrue(shelf.text().contains("\"isSuccess\":true"));
                page.locator(".bookshelf-page").waitFor();
            } finally {
                browser.close();
            }
        }
    }
}
