package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Locator;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import com.microsoft.playwright.Response;
import com.microsoft.playwright.Route;
import com.microsoft.playwright.options.AriaRole;
import com.microsoft.playwright.options.WaitForSelectorState;
import org.junit.Assume;
import org.junit.Test;

import java.io.InputStream;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.Assert.*;

/** Generated accounts only: one explicit rejected-response fixture, then real backend writes. */
public class Vue3PreviewSettingsDialogTest {
    @Test
    public void rejectedAddRetainsFormThenRealTtsAddAndEditCloseAndPersist() throws Exception {
        String base = System.getenv("READER_VUE3_PREVIEW_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(base != null && !executable.isEmpty() && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        URI uri = URI.create(base);
        assertEquals("http", uri.getScheme());
        assertTrue("Only the generated loopback Reader may be modified",
                "127.0.0.1".equals(uri.getHost()) || "localhost".equals(uri.getHost()));

        String name = "GeneratedTTS-" + UUID.randomUUID().toString().substring(0, 8);
        String renamed = name + "-renamed";
        String occupied = "OccupiedTTS-" + UUID.randomUUID().toString().substring(0, 8);
        String originalUrl = base + "/generated-speech-only";
        String editedUrl = base + "/generated-speech-revised";
        AtomicInteger synthesisRequests = new AtomicInteger();
        AtomicInteger opdsRequests = new AtomicInteger();
        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                Page alice = browser.newPage();
                Page bob = browser.newPage();
                alice.setDefaultTimeout(15000);
                bob.setDefaultTimeout(15000);
                alice.onRequest(request -> {
                    if (request.url().startsWith(base + "/generated-speech")) synthesisRequests.incrementAndGet();
                    String path = URI.create(request.url()).getPath();
                    if ("/opds".equals(path) || "/reader3/getOpdsSettings".equals(path)
                            || "/reader3/saveOpdsSettings".equals(path)) opdsRequests.incrementAndGet();
                });
                register(alice, base, "diala");
                register(bob, base, "dialb");
                assertEquals(0, list(alice).size());
                alice.navigate(base + "/settings");
                alice.locator(".settings-page").waitFor();
                alice.locator("button").filter(new Locator.FilterOptions().setHasText("新增听书源")).click();
                Locator dialog = alice.locator("[aria-label='新增听书源']");
                dialog.waitFor();
                dialog.locator("input").nth(0).fill(originalUrl);
                dialog.locator("input").nth(1).fill(name);

                AtomicBoolean rejectedOnce = new AtomicBoolean();
                alice.route("**/reader3/httpTTS/save?*", route -> {
                    if (rejectedOnce.compareAndSet(false, true)) {
                        route.fulfill(new Route.FulfillOptions().setStatus(200).setContentType("application/json")
                                .setBody("{\"isSuccess\":false,\"errorMsg\":\"生成的保存失败夹具\",\"data\":\"\"}"));
                    } else route.resume();
                });
                Response rejected = alice.waitForResponse(Vue3PreviewSettingsDialogTest::isSave,
                        () -> dialog.locator("button[type=submit]").click());
                assertEquals(200, rejected.status());
                assertTrue(rejected.text().contains("生成的保存失败夹具"));
                alice.waitForFunction("() => { const d = document.querySelector('[aria-label=\"新增听书源\"]');"
                        + "return d && !d.querySelector('button[type=submit]').disabled; }");
                assertTrue(rejectedOnce.get());
                assertTrue(alice.locator(".settings-page").isVisible());
                assertEquals(0, alice.locator(".error-boundary").count());
                assertEquals(name, dialog.locator("input").nth(1).inputValue());
                assertEquals(originalUrl, dialog.locator("input").nth(0).inputValue());
                assertEquals(0, list(alice).size());
                assertEquals(0, ((Number) alice.evaluate(
                        "() => JSON.parse(localStorage.getItem('reader_http_tts_list') || '[]').length")).intValue());
                screenshot(alice, "tts-rejected");

                Response added = alice.waitForResponse(Vue3PreviewSettingsDialogTest::isSave,
                        () -> dialog.locator("button[type=submit]").click());
                assertEquals(200, added.status());
                assertTrue(added.text().contains("\"isSuccess\":true"));
                dialog.waitFor(new Locator.WaitForOptions().setState(WaitForSelectorState.HIDDEN));
                assertEquals("", alice.evaluate("() => document.body.style.overflow"));
                Locator row = alice.locator(".tts-list .tts-row").filter(new Locator.FilterOptions().setHasText(name));
                row.waitFor();
                row.scrollIntoViewIfNeeded();
                assertEquals("Legacy HTTP sources must not display a missing Vue-only field",
                        "在线合成", row.locator(".tts-type").textContent());
                assertEquals(originalUrl, named(list(alice), name).get("url"));
                assertEquals(0, list(bob).size());
                screenshot(alice, "tts-added");

                Map<String, Object> firstRecord = named(list(alice), name);
                // Playwright Java's browser argument serializer accepts Double,
                // not Long; this generated integer is exactly representable in JS.
                assertTrue(post(alice, "/httpTTS/save", Map.of("id", 1900000000002d,
                        "name", occupied, "url", base + "/generated-other-only")).get("isSuccess").equals(true));
                Map<String, Object> occupiedRecord = named(list(alice), occupied);
                row.locator("button[title='编辑听书源（完整字段）']").click();
                Locator editor = alice.locator("[aria-label='编辑听书源']");
                editor.waitFor();
                editor.locator("input").nth(0).fill(editedUrl);
                editor.locator("input").nth(1).fill(occupied);
                editor.locator("textarea").nth(0).fill("{\"X-Generated-UI\":\"one\"}");
                editor.locator("textarea").nth(1).fill("generated-library-not-executed");
                editor.locator("input[type=checkbox]").check();
                Response conflict = alice.waitForResponse(Vue3PreviewSettingsDialogTest::isUpdate,
                        () -> editor.locator("button[type=submit]").click());
                assertEquals(200, conflict.status());
                assertTrue(conflict.text().contains("\"isSuccess\":false"));
                assertTrue("Generated collision response: " + conflict.text(), conflict.text().contains("名称已存在"));
                alice.waitForFunction("() => !document.querySelector('[aria-label=\"编辑听书源\"] button[type=submit]').disabled");
                assertTrue(editor.isVisible());
                assertEquals(occupied, editor.locator("input").nth(1).inputValue());
                assertEquals(firstRecord, named(list(alice), name));
                assertEquals(occupiedRecord, named(list(alice), occupied));
                assertEquals(0, list(bob).size());
                screenshot(alice, "tts-rename-conflict");

                editor.locator("input").nth(1).fill(renamed);
                Response edited = alice.waitForResponse(Vue3PreviewSettingsDialogTest::isUpdate,
                        () -> editor.locator("button[type=submit]").click());
                assertEquals(200, edited.status());
                assertTrue(edited.text().contains("\"isSuccess\":true"));
                editor.waitFor(new Locator.WaitForOptions().setState(WaitForSelectorState.HIDDEN));
                assertEquals("", alice.evaluate("() => document.body.style.overflow"));
                assertEquals(2, list(alice).size());
                assertFalse(list(alice).stream().anyMatch(value -> name.equals(value.get("name"))));
                Map<String, Object> renamedRecord = named(list(alice), renamed);
                assertEquals(firstRecord.get("id"), renamedRecord.get("id"));
                assertEquals(editedUrl, renamedRecord.get("url"));
                assertEquals("{\"X-Generated-UI\":\"one\"}", renamedRecord.get("header"));
                assertEquals("generated-library-not-executed", renamedRecord.get("jsLib"));
                assertEquals(true, renamedRecord.get("enabledCookieJar"));
                assertEquals(occupiedRecord, named(list(alice), occupied));
                assertEquals(false, post(bob, "/httpTTS/update", Map.of("original", renamedRecord,
                        "updated", Map.of("id", renamedRecord.get("id"), "name", renamed, "url", editedUrl))).get("isSuccess"));
                assertEquals(0, list(bob).size());
                Page anonymous = browser.newPage();
                anonymous.navigate(base + "/login");
                Map<String, Object> noLogin = post(anonymous, "/httpTTS/update", Map.of("original", renamedRecord,
                        "updated", Map.of("id", renamedRecord.get("id"), "name", renamed, "url", editedUrl)));
                assertEquals(false, noLogin.get("isSuccess"));
                assertEquals("NEED_LOGIN", noLogin.get("data"));
                anonymous.close();
                alice.reload();
                alice.locator(".settings-page").waitFor();
                row.waitFor();
                assertTrue(row.textContent().contains(editedUrl));
                assertEquals("在线合成", row.locator(".tts-type").textContent());
                row.scrollIntoViewIfNeeded();
                screenshot(alice, "tts-edited");
                verifyStaleAndOfflineEdit(alice, base, renamed, occupiedRecord);
                assertEquals("This storage journey must not synthesize or download audio", 0, synthesisRequests.get());
                verifyUnavailableOpds(alice, opdsRequests);
            } finally {
                browser.close();
            }
        }
    }

    private static void verifyStaleAndOfflineEdit(Page page, String base, String name,
                                                  Map<String, Object> occupiedRecord) throws Exception {
        Locator row = page.locator(".tts-list .tts-row").filter(new Locator.FilterOptions().setHasText(name));
        row.locator("button[title='编辑听书源（完整字段）']").click();
        Locator editor = page.locator("[aria-label='编辑听书源']");
        editor.waitFor();
        Map<String, Object> original = named(list(page), name);
        String concurrentUrl = base + "/generated-speech-concurrent";
        assertEquals(true, post(page, "/httpTTS/update", Map.of("original", original,
                "updated", Map.of("id", original.get("id"), "name", name, "url", concurrentUrl))).get("isSuccess"));
        editor.locator("input").nth(0).fill(base + "/generated-speech-stale-must-not-persist");
        Response stale = page.waitForResponse(Vue3PreviewSettingsDialogTest::isUpdate,
                () -> editor.locator("button[type=submit]").click());
        assertEquals(200, stale.status());
        assertTrue(stale.text().contains("\"isSuccess\":false"));
        assertTrue(stale.text().contains("已被修改"));
        page.waitForFunction("() => !document.querySelector('[aria-label=\"编辑听书源\"] button[type=submit]').disabled");
        assertTrue(editor.isVisible());
        assertEquals(concurrentUrl, named(list(page), name).get("url"));
        assertEquals(occupiedRecord, named(list(page), String.valueOf(occupiedRecord.get("name"))));
        screenshot(page, "tts-edit-stale");
        editor.locator(".ghost-btn").click();
        editor.waitFor(new Locator.WaitForOptions().setState(WaitForSelectorState.HIDDEN));
        page.reload();
        row.waitFor();
        page.waitForFunction("() => JSON.parse(localStorage.getItem('reader_http_tts_list') || '[]')"
                + ".some(item => item.url.endsWith('/generated-speech-concurrent'))");
        Object cacheBefore = page.evaluate("() => localStorage.getItem('reader_http_tts_list')");
        row.locator("button[title='编辑听书源（完整字段）']").click();
        editor.waitFor();
        editor.locator("input").nth(1).fill(name + "-offline-must-not-persist");
        page.route("**/reader3/httpTTS/update?*", Route::abort);
        page.waitForRequest(request -> "POST".equals(request.method())
                        && URI.create(request.url()).getPath().equals("/reader3/httpTTS/update"),
                () -> editor.locator("button[type=submit]").click());
        page.locator(".el-message").filter(new Locator.FilterOptions().setHasText("未确认保存")).waitFor();
        assertTrue(editor.isVisible());
        assertEquals(cacheBefore, page.evaluate("() => localStorage.getItem('reader_http_tts_list')"));
        assertEquals(concurrentUrl, named(list(page), name).get("url"));
        assertEquals(2, list(page).size());
        screenshot(page, "tts-edit-offline");
        page.unroute("**/reader3/httpTTS/update?*");
        editor.locator(".ghost-btn").click();
        editor.waitFor(new Locator.WaitForOptions().setState(WaitForSelectorState.HIDDEN));
        assertEquals("", page.evaluate("() => document.body.style.overflow"));
    }

    private static void verifyUnavailableOpds(Page page, AtomicInteger requests) throws Exception {
        // The TTS offline failure was asserted and captured in its own scenario.
        // Dismiss its notifications through the visible control before collecting
        // unrelated OPDS evidence; never clear them before the TTS assertions.
        if (page.locator("#reader-message-stack .el-message").count() > 0) {
            page.getByRole(AriaRole.BUTTON, new Page.GetByRoleOptions().setName("关闭消息")).click();
        }
        page.waitForFunction("() => document.querySelectorAll('#reader-message-stack .el-message').length === 0");
        Locator card = page.locator("section.card").filter(new Locator.FilterOptions().setHasText("OPDS 访问"));
        card.locator("button").filter(new Locator.FilterOptions().setHasText("配置")).click();
        Locator dialog = page.locator("[aria-label='OPDS 账号']");
        dialog.waitFor();
        dialog.locator("input").nth(0).fill("generated-opds");
        dialog.locator("input").nth(1).fill("GeneratedOpds-NoPersist-2026");
        dialog.locator("button[type=submit]").click();
        dialog.locator(".field-tip.error").waitFor();
        assertTrue(dialog.locator(".field-tip.error").textContent()
                .contains("当前 Java/Kotlin 服务端未实现 OPDS 独立账号配置"));
        assertTrue(dialog.isVisible());
        assertEquals(0, page.locator(".error-boundary").count());
        assertEquals("The unavailable adapter must not send a substitute credential API call", 0, requests.get());
        assertEquals("The OPDS screenshot must not inherit the completed TTS offline failure", 0,
                page.locator("#reader-message-stack .el-message")
                        .filter(new Locator.FilterOptions().setHasText("网络连接失败")).count());
        assertEquals(false, page.evaluate("() => [localStorage, sessionStorage].some(storage =>"
                + " Object.values(storage).some(value => value.includes('GeneratedOpds-NoPersist-2026')))"));
        screenshot(page, "opds-unavailable");
        dialog.locator(".ghost-btn").click();
        dialog.waitFor(new Locator.WaitForOptions().setState(WaitForSelectorState.HIDDEN));
        assertEquals("", page.evaluate("() => document.body.style.overflow"));
    }

    private static boolean isSave(Response response) {
        return "POST".equals(response.request().method())
                && URI.create(response.url()).getPath().equals("/reader3/httpTTS/save");
    }

    private static boolean isUpdate(Response response) {
        return "POST".equals(response.request().method())
                && URI.create(response.url()).getPath().equals("/reader3/httpTTS/update");
    }

    @SuppressWarnings("unchecked")
    private static Map<String, Object> post(Page page, String route, Map<String, Object> body) {
        return (Map<String, Object>) page.evaluate("async args => {"
                + "const token = localStorage.getItem('reader_access_token');"
                + "const response = await fetch('/reader3' + args.route + '?accessToken=' + encodeURIComponent(token),"
                + "{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(args.body)});"
                + "if (response.status !== 200) throw new Error('Generated edit status failed');"
                + "return await response.json(); }", Map.of("route", route, "body", body));
    }

    private static void register(Page page, String base, String prefix) {
        page.navigate(base + "/login");
        page.locator(".mode-switch button").nth(1).click();
        page.locator("input[autocomplete=username]").fill(prefix
                + UUID.randomUUID().toString().replace("-", "").substring(0, 10));
        page.locator("input[autocomplete=current-password]").fill("GeneratedDialog-2026");
        page.locator(".submit-btn").click();
        page.locator(".bookshelf-page").waitFor();
    }

    @SuppressWarnings("unchecked")
    private static List<Map<String, Object>> list(Page page) {
        return (List<Map<String, Object>>) page.evaluate("async () => {"
                + "const token = localStorage.getItem('reader_access_token');"
                + "const response = await fetch('/reader3/httpTTS/list?accessToken=' + encodeURIComponent(token));"
                + "const result = await response.json();"
                + "if (!result.isSuccess || !Array.isArray(result.data)) throw new Error('Generated TTS list failed');"
                + "return result.data; }");
    }

    private static Map<String, Object> named(List<Map<String, Object>> values, String name) {
        return values.stream().filter(value -> name.equals(value.get("name"))).findFirst()
                .orElseThrow(() -> new AssertionError("Generated TTS was not persisted"));
    }

    private static void screenshot(Page page, String scene) throws Exception {
        String ready;
        try (InputStream input = Vue3PreviewSettingsDialogTest.class.getResourceAsStream("/ui-screenshot-readiness.js")) {
            assertTrue("Screenshot guard must be packaged", input != null);
            ready = new String(input.readAllBytes(), StandardCharsets.UTF_8);
        }
        page.waitForFunction(ready, false);
        String directory = System.getenv("RUNNER_TEMP");
        if (directory == null || directory.isEmpty()) return;
        page.screenshot(new Page.ScreenshotOptions().setPath(
                Path.of(directory, "vue3-settings-generated-" + scene + ".png")));
    }
}
