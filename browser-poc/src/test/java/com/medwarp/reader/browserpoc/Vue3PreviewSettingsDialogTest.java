package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Locator;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import com.microsoft.playwright.Response;
import com.microsoft.playwright.Route;
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
                assertEquals(originalUrl, named(list(alice), name).get("url"));
                assertEquals(0, list(bob).size());
                screenshot(alice, "tts-added");

                row.locator("button[title='编辑听书源（完整字段）']").click();
                Locator editor = alice.locator("[aria-label='编辑听书源']");
                editor.waitFor();
                editor.locator("input").nth(0).fill(editedUrl);
                editor.locator("textarea").nth(0).fill("{\"X-Generated-UI\":\"one\"}");
                Response edited = alice.waitForResponse(Vue3PreviewSettingsDialogTest::isSave,
                        () -> editor.locator("button[type=submit]").click());
                assertEquals(200, edited.status());
                assertTrue(edited.text().contains("\"isSuccess\":true"));
                editor.waitFor(new Locator.WaitForOptions().setState(WaitForSelectorState.HIDDEN));
                assertEquals("", alice.evaluate("() => document.body.style.overflow"));
                assertEquals(1, list(alice).size());
                assertEquals(editedUrl, named(list(alice), name).get("url"));
                assertEquals("{\"X-Generated-UI\":\"one\"}", named(list(alice), name).get("header"));
                assertEquals(0, list(bob).size());
                alice.reload();
                alice.locator(".settings-page").waitFor();
                row.waitFor();
                assertTrue(row.textContent().contains(editedUrl));
                row.scrollIntoViewIfNeeded();
                screenshot(alice, "tts-edited");
                assertEquals("This storage journey must not synthesize or download audio", 0, synthesisRequests.get());
                verifyUnavailableOpds(alice, opdsRequests);
            } finally {
                browser.close();
            }
        }
    }

    private static void verifyUnavailableOpds(Page page, AtomicInteger requests) throws Exception {
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
