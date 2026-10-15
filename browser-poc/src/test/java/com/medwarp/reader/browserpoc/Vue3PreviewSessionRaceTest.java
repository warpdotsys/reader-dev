package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.*;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
import java.net.URLDecoder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.Assert.*;

/** Real generated-account forms plus deliberately delayed, synthetic authentication errors. */
public class Vue3PreviewSessionRaceTest {
    private static final String PASSWORD = "Generated-Session-Race-20261005";
    private static final String MARKER = "生成的延迟认证失败";
    private static final String OLD_BOOK = "生成的旧会话书架，不得晚到写入新会话";

    private static void signIn(Page page, String username, boolean register) {
        page.locator(".mode-switch button").nth(register ? 1 : 0).click();
        page.locator("input[autocomplete=username]").fill(username);
        page.locator("input[autocomplete=current-password]").fill(PASSWORD);
        page.locator("input[type=checkbox]").check();
        Response login = page.waitForResponse(
                response -> response.url().contains("/reader3/login") && "POST".equals(response.request().method()),
                () -> page.locator(".submit-btn").click());
        assertEquals("Actual generated login must reach the backend", 200, login.status());
        assertTrue("Actual login ReturnData must succeed (do not log its token)",
                Boolean.TRUE.equals(((Map<?, ?>) page.evaluate("body => JSON.parse(body)", login.text())).get("isSuccess")));
        page.locator(".bookshelf-page").waitFor();
        assertEquals(username, page.evaluate("localStorage.getItem('reader_username')"));
    }

    private static void logout(Page page) {
        // SPA navigation only: a document navigation would abort the old XHR and hide this race.
        page.locator(".logout-btn").click();
        page.locator(".login-page").waitFor();
    }

    private static String token(Page page) {
        String value = (String) page.evaluate("localStorage.getItem('reader_access_token')");
        assertTrue("Generated account must have a token", value != null && !value.isEmpty());
        return value;
    }

    private static void release(Page page, Route held, int mode) {
        String body = "{\"isSuccess\":false,\"errorMsg\":\"" + MARKER
                + (mode == 2 ? "，请登录后使用" : "") + "\",\"data\":"
                + (mode == 1 ? "\"NEED_LOGIN\"" : "null") + "}";
        Response response = page.waitForResponse(r -> r.request().equals(held.request()),
                () -> held.fulfill(new Route.FulfillOptions().setStatus(mode == 0 ? 401 : 200)
                        .setContentType("application/json; charset=utf-8").setBody(body)));
        assertEquals(mode == 0 ? 401 : 200, response.status());
        assertFalse(Boolean.TRUE.equals(((Map<?, ?>) page.evaluate("body => JSON.parse(body)", response.text())).get("isSuccess")));
        // Drain the response callback and Vue/router updates, not a network-only green check.
        response.finished();
        page.evaluate("() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))");
    }

    @Test
    public void oldAccountAndOldSessionErrorsCannotLogoutTheCurrentSession() throws Exception {
        String preview = System.getenv("READER_VUE3_PREVIEW_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(preview != null && !executable.isEmpty() && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        URI uri = URI.create(preview);
        assertEquals("http", uri.getScheme());
        assertEquals("127.0.0.1", uri.getHost());

        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")));
             Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                     .setExecutablePath(Path.of(executable)).setHeadless(true))) {
            for (int mode = 0; mode < 3; mode++) {
                // Each error kind uses a new context; only the two generated accounts share storage.
                try (BrowserContext context = browser.newContext()) {
                    Page page = context.newPage();
                    page.setDefaultTimeout(10000);
                    AtomicReference<Route> pending = new AtomicReference<>();
                    AtomicReference<String> holdUsername = new AtomicReference<>();
                    String suffix = UUID.randomUUID().toString().replace("-", "").substring(0, 10);
                    String first = "racea" + suffix;
                    String second = "raceb" + suffix;
                    holdUsername.set(first);
                    page.route("**/reader3/getBookshelf?*", route -> {
                        String query = URI.create(route.request().url()).getRawQuery();
                        String name = holdUsername.get();
                        if (name != null && query != null
                                && java.util.Arrays.stream(query.split("&"))
                                .map(part -> URLDecoder.decode(part, StandardCharsets.UTF_8))
                                .anyMatch(part -> part.startsWith("accessToken=" + name + ":"))
                                && pending.compareAndSet(null, route)) {
                            holdUsername.set(null);
                        } else {
                            route.resume();
                        }
                    });
                    page.navigate(preview + "/login");
                    signIn(page, first, true);
                    page.waitForCondition(() -> pending.get() != null);
                    Route oldAccount = pending.getAndSet(null);
                    logout(page);
                    signIn(page, second, true);
                    String currentToken = token(page);
                    assertEquals(true, page.evaluate("async () => { const r = await fetch('/reader3/getBookshelf?accessToken='"
                            + " + encodeURIComponent(localStorage.getItem('reader_access_token')));"
                            + " const b = await r.json(); return r.status === 200 && b.isSuccess === true; }"));
                    release(page, oldAccount, mode);
                    String evidence = System.getenv("READER_UI_EVIDENCE_DIR");
                    if (evidence == null) evidence = System.getenv("RUNNER_TEMP");
                    if (evidence != null) page.screenshot(new Page.ScreenshotOptions()
                            .setPath(Path.of(evidence, "vue3-session-race-generated-" + mode + ".png")));
                    assertTrue("Old-account error mode " + mode + " must not remove the new account token",
                            currentToken.equals(page.evaluate("localStorage.getItem('reader_access_token')")));
                    assertEquals(second, page.evaluate("localStorage.getItem('reader_username')"));
                    assertTrue("Old-account error must keep the real bookshelf visible",
                            page.locator(".bookshelf-page").isVisible());
                    assertEquals(0, page.locator(".el-message--error").filter(
                            new Locator.FilterOptions().setHasText(MARKER)).count());

                    // A late success must also be discarded before obsolete callers save a shelf.
                    holdUsername.set(second);
                    page.locator(".refresh-btn").click();
                    page.waitForCondition(() -> pending.get() != null);
                    Route oldSuccess = pending.getAndSet(null);
                    logout(page);
                    signIn(page, second, false);
                    String successSessionToken = token(page);
                    Response lateSuccess = page.waitForResponse(r -> r.request().equals(oldSuccess.request()),
                            () -> oldSuccess.fulfill(new Route.FulfillOptions().setStatus(200)
                                    .setContentType("application/json; charset=utf-8")
                                    .setBody("{\"isSuccess\":true,\"errorMsg\":\"\",\"data\":[{\"name\":\""
                                            + OLD_BOOK + "\",\"bookUrl\":\"" + preview
                                            + "/generated-old-book\",\"author\":\"generated\"}]}")));
                    assertEquals(200, lateSuccess.status());
                    lateSuccess.finished();
                    page.evaluate("() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))");
                    assertTrue(successSessionToken.equals(page.evaluate("localStorage.getItem('reader_access_token')")));
                    assertFalse("Old successful shelf must not reach the current page", page.locator("body").innerText().contains(OLD_BOOK));
                    assertEquals(false, page.evaluate("marker => (localStorage.getItem('reader_shelf_offline') || '').includes(marker)", OLD_BOOK));

                    // Same account, new successful sign-in, still no document reload.
                    holdUsername.set(second);
                    page.locator(".refresh-btn").click();
                    page.waitForCondition(() -> pending.get() != null);
                    Route oldSession = pending.getAndSet(null);
                    logout(page);
                    signIn(page, second, false);
                    String renewedToken = token(page);
                    release(page, oldSession, mode);
                    assertTrue("Old session error must not remove a new sign-in for the same account",
                            renewedToken.equals(page.evaluate("localStorage.getItem('reader_access_token')")));
                    assertTrue(page.locator(".bookshelf-page").isVisible());

                    // The guard must not suppress a current session's real expiry behavior.
                    holdUsername.set(second);
                    page.locator(".refresh-btn").click();
                    page.waitForCondition(() -> pending.get() != null);
                    release(page, pending.getAndSet(null), mode);
                    page.locator(".login-page").waitFor();
                    assertNull(page.evaluate("localStorage.getItem('reader_access_token')"));
                    assertNull(page.evaluate("sessionStorage.getItem('reader_access_token')"));
                    signIn(page, second, false);
                    assertEquals(second, page.evaluate("localStorage.getItem('reader_username')"));
                    System.out.printf("SESSION_RACE mode=%d oldAccount=ignored oldSuccess=discarded oldSession=ignored currentExpiry=login retry=bookshelf%n", mode);
                }
            }
        }
    }
}
