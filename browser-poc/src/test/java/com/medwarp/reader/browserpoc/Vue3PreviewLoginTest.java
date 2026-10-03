package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Locator;
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
    public void registerLogoutLoginReloadAndLoadEmptyBookshelf() {
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

                // Registration's automatic sign-in is not the existing-user
                // login journey. Exercise the actual form after logging out.
                page.locator(".logout-btn").click();
                page.locator(".login-page").waitFor();
                assertEquals(null, page.evaluate("localStorage.getItem('reader_access_token')"));
                page.locator("input[autocomplete=username]").fill(username);
                page.locator("input[autocomplete=current-password]").fill("WrongPreviewProbe-2026");
                Response refusedLogin = page.waitForResponse(
                        response -> response.url().contains("/reader3/login")
                                && "POST".equals(response.request().method()),
                        () -> page.locator(".submit-btn").click());
                assertEquals("Legacy authentication failures still use HTTP 200", 200, refusedLogin.status());
                assertTrue("Wrong password must be a failed ReturnData, not a usable session",
                        refusedLogin.text().matches("(?s).*\"isSuccess\"\\s*:\\s*false.*"));
                page.locator(".el-message--error").filter(new Locator.FilterOptions()
                        .setHasText("密码错误")).waitFor();
                assertTrue("A refused login must leave the form usable", page.locator(".login-page").isVisible());
                assertEquals(null, page.evaluate("localStorage.getItem('reader_access_token')"));
                assertEquals(null, page.evaluate("sessionStorage.getItem('reader_access_token')"));

                // A retry must recover without reloading the page. In this
                // branch remember=false means tab-local storage, not no login.
                page.locator("input[autocomplete=current-password]").fill("PreviewProbe-2026");
                page.locator("input[type=checkbox]").uncheck();
                page.locator(".submit-btn").click();
                page.locator(".bookshelf-page").waitFor();
                assertEquals(null, page.evaluate("localStorage.getItem('reader_access_token')"));
                assertEquals(null, page.evaluate("localStorage.getItem('reader_username')"));
                assertEquals(username, page.evaluate("sessionStorage.getItem('reader_username')"));
                assertEquals("0", page.evaluate("localStorage.getItem('reader_remember')"));
                String loginToken = (String) page.evaluate("sessionStorage.getItem('reader_access_token')");
                assertTrue("Existing-user login token must be namespaced to this account",
                        loginToken.startsWith(username + ":"));

                Response shelf = page.waitForResponse(
                        response -> response.url().contains("/reader3/getBookshelf"),
                        page::reload);
                assertEquals(200, shelf.status());
                page.locator(".bookshelf-page").waitFor();
                assertEquals("Reload must retain a tab-local authenticated session", loginToken,
                        page.evaluate("sessionStorage.getItem('reader_access_token')"));
                assertEquals(null, page.evaluate("localStorage.getItem('reader_access_token')"));
                // Chromium can discard a reload response body after navigation;
                // inspect the envelope through a fresh authenticated request.
                assertEquals(true, page.evaluate("async () => {" +
                        "const token = sessionStorage.getItem('reader_access_token');" +
                        "const response = await fetch('/reader3/getBookshelf?accessToken=' + encodeURIComponent(token));" +
                        "const result = await response.json();" +
                        "return response.status === 200 && result.isSuccess === true &&" +
                        "Array.isArray(result.data) && result.data.length === 0; }"));

                page.locator(".logout-btn").click();
                page.locator(".login-page").waitFor();
                assertEquals(null, page.evaluate("localStorage.getItem('reader_access_token')"));
                assertEquals(null, page.evaluate("sessionStorage.getItem('reader_access_token')"));
                page.navigate(previewUrl + "/");
                page.locator(".login-page").waitFor();
                assertTrue("The protected bookshelf must not remain visible after logout",
                        page.locator(".bookshelf-page").count() == 0);
            } finally {
                browser.close();
            }
        }
    }
}
