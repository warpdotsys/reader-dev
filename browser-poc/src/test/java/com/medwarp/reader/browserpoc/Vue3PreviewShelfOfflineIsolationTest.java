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
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.Assert.*;

/** Generated metadata only: real login/save/get, with deliberately failed shelf transport. */
public class Vue3PreviewShelfOfflineIsolationTest {
    private static final String PASSWORD = "Generated-Offline-Shelf-20261005";
    private static final String FIRST_BOOK = "生成账号甲的离线书架";
    private static final String SECOND_BOOK = "生成账号乙的离线书架";
    private static final String UNKNOWN_BOOK = "归属未知的生成旧缓存，不得自动迁移";
    private static final String LEGACY = "{\"books\":[{\"bookUrl\":\"generated-legacy\",\"name\":\""
            + UNKNOWN_BOOK + "\",\"author\":\"generated\"}],\"groups\":[],\"ts\":1}";

    private static void signIn(Page page, String username, boolean register) {
        page.locator(".mode-switch button").nth(register ? 1 : 0).click();
        page.locator("input[autocomplete=username]").fill(username);
        page.locator("input[autocomplete=current-password]").fill(PASSWORD);
        page.locator("input[type=checkbox]").check();
        Response login = page.waitForResponse(r -> r.url().contains("/reader3/login")
                        && "POST".equals(r.request().method()),
                () -> page.locator(".submit-btn").click());
        assertEquals(200, login.status());
        assertTrue(Boolean.TRUE.equals(((Map<?, ?>) page.evaluate("body => JSON.parse(body)", login.text())).get("isSuccess")));
        page.locator(".bookshelf-page").waitFor();
        settled(page);
        assertEquals(username, page.evaluate("localStorage.getItem('reader_username')"));
    }

    private static void settled(Page page) {
        page.waitForCondition(() -> page.locator(".book-grid[aria-label='加载中']").count() == 0
                && page.locator(".refresh-btn.spinning").count() == 0);
        page.evaluate("() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))");
    }

    private static void logout(Page page) {
        page.locator(".logout-btn").click();
        page.locator(".login-page").waitFor();
    }

    private static void saveGeneratedMetadata(Page page, String name) {
        Object ok = page.evaluate("async name => { const token = localStorage.getItem('reader_access_token');"
                + " const r = await fetch('/reader3/saveBook?accessToken=' + encodeURIComponent(token), {"
                + " method: 'POST', headers: {'Content-Type':'application/json'},"
                + " body: JSON.stringify({bookUrl:'generated/' + name + '.txt', origin:'loc_book',"
                + " originName:'generated-metadata-only', name, author:'生成夹具', canUpdate:false})});"
                + " const b = await r.json(); return r.status === 200 && b.isSuccess === true; }", name);
        assertEquals("Actual metadata save, not a stub implementation", true, ok);
    }

    private static void screenshot(Page page, String suffix) {
        String evidence = System.getenv("READER_UI_EVIDENCE_DIR");
        if (evidence == null) evidence = System.getenv("RUNNER_TEMP");
        if (evidence != null) page.screenshot(new Page.ScreenshotOptions()
                .setPath(Path.of(evidence, "vue3-shelf-offline-generated-" + suffix + ".png")));
    }

    @Test
    public void offlineMetadataCannotCrossAccountsAndBusinessErrorsCannotMasqueradeAsOffline() throws Exception {
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
                     .setExecutablePath(Path.of(executable)).setHeadless(true));
             BrowserContext context = browser.newContext()) {
            Page page = context.newPage();
            page.setDefaultTimeout(10000);
            String suffix = UUID.randomUUID().toString().replace("-", "").substring(0, 10);
            String first = "offa" + suffix;
            String second = "offb" + suffix;
            AtomicReference<String> blocked = new AtomicReference<>();
            AtomicReference<String> failure = new AtomicReference<>("network");
            AtomicReference<String> holdAccount = new AtomicReference<>();
            AtomicReference<Route> held = new AtomicReference<>();
            AtomicBoolean groupFailure = new AtomicBoolean();
            page.route("**/reader3/getBookGroups?*", route -> {
                if (!groupFailure.get()) route.resume();
                else route.fulfill(new Route.FulfillOptions().setStatus(200)
                        .setContentType("application/json; charset=utf-8")
                        .setBody("{\"isSuccess\":false,\"errorMsg\":\"生成分组业务失败\",\"data\":null}"));
            });
            page.route("**/reader3/getBookshelf?*", route -> {
                String query = URI.create(route.request().url()).getRawQuery();
                String hold = holdAccount.get();
                if (hold != null && query != null && java.util.Arrays.stream(query.split("&"))
                        .map(part -> URLDecoder.decode(part, StandardCharsets.UTF_8))
                        .anyMatch(part -> part.startsWith("accessToken=" + hold + ":"))
                        && held.compareAndSet(null, route)) {
                    holdAccount.set(null);
                    return;
                }
                String account = blocked.get();
                boolean matches = account != null && query != null && java.util.Arrays.stream(query.split("&"))
                        .map(part -> URLDecoder.decode(part, StandardCharsets.UTF_8))
                        .anyMatch(part -> part.startsWith("accessToken=" + account + ":"));
                if (!matches) route.resume();
                else if ("network".equals(failure.get())) route.abort("failed");
                else route.fulfill(new Route.FulfillOptions().setStatus("503".equals(failure.get()) ? 503 : 200)
                        .setContentType("application/json; charset=utf-8")
                        .setBody("503".equals(failure.get())
                                ? "{\"isSuccess\":false,\"errorMsg\":\"生成服务不可用\",\"data\":null}"
                                : "{\"isSuccess\":false,\"errorMsg\":\"生成业务拒绝\",\"data\":\"NEED_SECURE_KEY\"}"));
            });
            page.navigate(preview + "/login");
            page.evaluate("raw => localStorage.setItem('reader_shelf_offline', raw)", LEGACY);
            signIn(page, first, true);
            saveGeneratedMetadata(page, FIRST_BOOK);
            page.locator(".refresh-btn").click();
            settled(page);
            assertTrue(page.locator(".book-card").filter(new Locator.FilterOptions().setHasText(FIRST_BOOK)).isVisible());
            logout(page);

            blocked.set(second);
            signIn(page, second, true);
            // The unmodified backend remains reachable and its new-account shelf is empty.
            String token = (String) page.evaluate("localStorage.getItem('reader_access_token')");
            APIResponse real = context.request().get(preview + "/reader3/getBookshelf?accessToken="
                    + java.net.URLEncoder.encode(token, StandardCharsets.UTF_8));
            assertEquals(200, real.status());
            assertEquals(true, page.evaluate("body => { const b=JSON.parse(body); return b.isSuccess === true && b.data.length === 0; }", real.text()));
            screenshot(page, "other-account-empty");
            assertFalse("Network failure must not expose the previous account's cached shelf",
                    page.locator("body").innerText().contains(FIRST_BOOK));
            assertFalse(page.locator("body").innerText().contains(UNKNOWN_BOOK));
            assertEquals(0, page.locator(".offline-shelf-banner").count());

            blocked.set(null);
            saveGeneratedMetadata(page, SECOND_BOOK);
            page.locator(".refresh-btn").click();
            settled(page);
            assertTrue(page.locator(".book-card").filter(new Locator.FilterOptions().setHasText(SECOND_BOOK)).isVisible());
            blocked.set(second);
            page.locator(".refresh-btn").click();
            settled(page);
            assertEquals(1, page.locator(".offline-shelf-banner").count());
            assertTrue(page.locator("body").innerText().contains(SECOND_BOOK));
            assertFalse(page.locator("body").innerText().contains(FIRST_BOOK));
            screenshot(page, "own-account-offline");
            logout(page);

            blocked.set(first);
            signIn(page, first, false);
            assertEquals(1, page.locator(".offline-shelf-banner").count());
            assertTrue(page.locator("body").innerText().contains(FIRST_BOOK));
            assertFalse(page.locator("body").innerText().contains(SECOND_BOOK));
            failure.set("business");
            page.reload();
            page.locator(".bookshelf-page").waitFor();
            settled(page);
            assertEquals("A business rejection is not an offline transport failure", 0,
                    page.locator(".offline-shelf-banner").count());
            assertFalse(page.locator("body").innerText().contains(FIRST_BOOK));

            failure.set("503");
            page.reload();
            page.locator(".bookshelf-page").waitFor();
            settled(page);
            assertEquals(1, page.locator(".offline-shelf-banner").count());
            assertTrue(page.locator("body").innerText().contains(FIRST_BOOK));
            blocked.set(null);
            page.locator(".offline-shelf-banner button").click();
            settled(page);
            // Vue's leave transition retains the DOM briefly after offlineShelf becomes false.
            // Wait for the actual banner to detach, not just completion of its HTTP request.
            page.waitForCondition(() -> page.locator(".offline-shelf-banner").count() == 0);
            assertEquals(0, page.locator(".offline-shelf-banner").count());
            assertTrue(page.locator("body").innerText().contains(FIRST_BOOK));
            assertEquals("Unknown legacy cache must be preserved, not migrated or removed", LEGACY,
                    page.evaluate("localStorage.getItem('reader_shelf_offline')"));
            assertEquals(false, page.evaluate("() => Object.keys(localStorage).filter(k => k.startsWith('reader_shelf_offline_v2:'))"
                    + ".some(k => k.includes(localStorage.getItem('reader_access_token')))"));

            // Two refreshes in the same session: an older success cannot replace the latest snapshot.
            holdAccount.set(first);
            page.locator(".refresh-btn").click();
            page.waitForCondition(() -> held.get() != null);
            page.locator(".refresh-btn").click();
            settled(page);
            Route delayed = held.getAndSet(null);
            Response late = page.waitForResponse(r -> r.request().equals(delayed.request()),
                    () -> delayed.fulfill(new Route.FulfillOptions().setStatus(200)
                            .setContentType("application/json; charset=utf-8")
                            .setBody("{\"isSuccess\":true,\"errorMsg\":\"\",\"data\":[{\"bookUrl\":\"generated-stale\","
                                    + "\"tocUrl\":\"\",\"origin\":\"loc_book\",\"originName\":\"generated\","
                                    + "\"name\":\"" + UNKNOWN_BOOK + "\",\"author\":\"generated\"}]}")));
            assertEquals(200, late.status());
            late.finished();
            settled(page);
            assertTrue(page.locator("body").innerText().contains(FIRST_BOOK));
            assertFalse(page.locator("body").innerText().contains(UNKNOWN_BOOK));
            assertEquals(false, page.evaluate("marker => Object.keys(localStorage).filter(k => k.startsWith('reader_shelf_offline_v2:'))"
                    + ".some(k => localStorage.getItem(k).includes(marker))", UNKNOWN_BOOK));

            // Partial success must not overwrite the last complete shelf+group snapshot with fake empty groups.
            Object cacheBefore = page.evaluate("() => Object.keys(localStorage).filter(k => k.startsWith('reader_shelf_offline_v2:')).sort()"
                    + ".map(k => [k,localStorage.getItem(k)])");
            String added = "生成分组失败期间的新元数据";
            saveGeneratedMetadata(page, added);
            groupFailure.set(true);
            page.locator(".refresh-btn").click();
            settled(page);
            assertTrue(page.locator("body").innerText().contains(added));
            assertEquals(cacheBefore, page.evaluate("() => Object.keys(localStorage).filter(k => k.startsWith('reader_shelf_offline_v2:')).sort()"
                    + ".map(k => [k,localStorage.getItem(k)])"));
            groupFailure.set(false);
            blocked.set(first);
            failure.set("network");
            page.locator(".refresh-btn").click();
            settled(page);
            assertTrue(page.locator("body").innerText().contains(FIRST_BOOK));
            assertFalse(page.locator("body").innerText().contains(added));
            assertEquals(1, page.locator(".offline-shelf-banner").count());
            blocked.set(null);
            page.locator(".offline-shelf-banner button").click();
            settled(page);
            page.waitForCondition(() -> page.locator(".offline-shelf-banner").count() == 0);
            assertTrue(page.locator("body").innerText().contains(added));
            assertEquals(0, page.locator(".offline-shelf-banner").count());
            System.out.println("SHELF_OFFLINE crossAccount=hidden ownAccount=restored business=notOffline server503=restored recovery=online legacy=untouched credentials=notInKey lateRefresh=discarded partialGroups=preserved");
        }
    }
}
