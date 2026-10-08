package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Locator;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import com.microsoft.playwright.Response;
import com.microsoft.playwright.options.WaitForSelectorState;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.UUID;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

/**
 * Isolated end-to-end contract for the restored Java/Kotlin TXT toc-rule API.
 * It verifies the on-disk legacy API shape indirectly through CRUD, default-rule
 * protection, idempotent default import, and a second user's isolated namespace.
 */
public class Vue3PreviewTxtTocRuleTest {
    @Test
    public void customRulesAreCrudableAndNeverLeakAcrossUsers() throws Exception {
        String previewUrl = System.getenv("READER_VUE3_PREVIEW_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(previewUrl != null && !previewUrl.isEmpty()
                && executable != null && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        URI preview = URI.create(previewUrl);
        assertEquals("http", preview.getScheme());
        assertTrue("Only an isolated loopback preview is allowed",
                "127.0.0.1".equals(preview.getHost()) || "localhost".equals(preview.getHost()));

        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                Page alice = browser.newPage();
                Page bob = browser.newPage();
                alice.setDefaultTimeout(15000);
                bob.setDefaultTimeout(15000);
                register(alice, previewUrl, "toca");
                register(bob, previewUrl, "tocb");

                String name = "E2E-TXT-" + UUID.randomUUID().toString().substring(0, 8);
                long id = 1_900_000_000_000L + (System.nanoTime() & 0x00ff_ffffL);
                Map<String, Object> saved = api(alice, "POST", "/saveTxtTocRule",
                        "{\"id\":" + id + ",\"name\":\"" + name
                                + "\",\"rule\":\"^第.+章$\",\"enable\":true,\"serialNumber\":99}");
                assertTrue((Boolean) saved.get("isSuccess"));
                @SuppressWarnings("unchecked")
                Map<String, Object> savedData = (Map<String, Object>) saved.get("data");
                assertEquals(id, ((Number) savedData.get("id")).longValue());

                assertTrue(ruleNames(api(alice, "GET", "/getTxtTocRules", null)).contains(name));
                assertFalse("规则只能属于当前登录用户",
                        ruleNames(api(bob, "GET", "/getTxtTocRules", null)).contains(name));

                Map<String, Object> firstImport = api(alice, "POST", "/importDefaultTxtTocRules", "{}");
                Map<String, Object> secondImport = api(alice, "POST", "/importDefaultTxtTocRules", "{}");
                assertTrue((Boolean) firstImport.get("isSuccess"));
                assertEquals(18, ((Number) dataMap(firstImport).get("count")).intValue());
                assertEquals("重复导入不得覆盖已编辑用户规则", 0,
                        ((Number) dataMap(secondImport).get("count")).intValue());

                Map<String, Object> defaultDelete = api(alice, "POST", "/deleteTxtTocRule", "{\"id\":-1}");
                assertFalse("JAR 内置负数 id 不允许通过用户接口删除", (Boolean) defaultDelete.get("isSuccess"));
                Map<String, Object> deleted = api(alice, "POST", "/deleteTxtTocRule", "{\"id\":" + id + "}");
                assertTrue((Boolean) deleted.get("isSuccess"));
                assertFalse(ruleNames(api(alice, "GET", "/getTxtTocRules", null)).contains(name));
                verifySettingsDialog(alice, bob, previewUrl, name + "-UI");
            } finally {
                browser.close();
            }
        }
    }

    private static void verifySettingsDialog(Page alice, Page bob, String base, String name) throws Exception {
        alice.navigate(base + "/settings");
        alice.locator(".settings-page").waitFor();
        alice.locator("button").filter(new Locator.FilterOptions().setHasText("新增规则")).click();
        Locator dialog = alice.locator("[aria-label='新增 txtTocRule']");
        dialog.waitFor();
        dialog.locator("input").nth(0).fill(name);
        dialog.locator("input").nth(1).fill("[");
        Response rejected = alice.waitForResponse(response -> isRuleWrite(response, "/saveTxtTocRule"),
                () -> dialog.locator("button[type=submit]").click());
        assertEquals(200, rejected.status());
        assertTrue(rejected.text().contains("\"isSuccess\":false"));
        assertTrue(rejected.text().contains("正则表达式无效"));
        alice.waitForFunction("() => { const d = document.querySelector('[aria-label=\"新增 txtTocRule\"]');"
                + "return d && !d.querySelector('button[type=submit]').disabled; }");
        assertTrue("A rejected save must retain the settings page, not invoke ErrorBoundary",
                alice.locator(".settings-page").isVisible());
        assertEquals(0, alice.locator(".error-boundary").count());
        assertEquals(name, dialog.locator("input").nth(0).inputValue());
        assertEquals("[", dialog.locator("input").nth(1).inputValue());
        screenshot(alice, "invalid");

        dialog.locator("input").nth(1).fill("^第.+章$");
        Response saved = alice.waitForResponse(response -> isRuleWrite(response, "/saveTxtTocRule"),
                () -> dialog.locator("button[type=submit]").click());
        assertEquals(200, saved.status());
        assertTrue(saved.text().contains("\"isSuccess\":true"));
        dialog.waitFor(new Locator.WaitForOptions().setState(WaitForSelectorState.HIDDEN));
        assertEquals("Successful save must restore page scrolling", "",
                alice.evaluate("() => document.body.style.overflow"));
        Locator row = alice.locator(".toc-list .tts-row").filter(new Locator.FilterOptions().setHasText(name));
        row.waitFor();
        row.scrollIntoViewIfNeeded();
        Map<String, Object> persisted = namedRule(api(alice, "GET", "/getTxtTocRules", null), name);
        assertTrue(((Number) persisted.get("id")).longValue() > 0);
        assertEquals("^第.+章$", persisted.get("rule"));
        assertEquals(true, persisted.get("enable"));
        assertFalse(ruleNames(api(bob, "GET", "/getTxtTocRules", null)).contains(name));
        screenshot(alice, "created");

        Response toggled = alice.waitForResponse(response -> isRuleWrite(response, "/saveTxtTocRule"),
                () -> row.locator("button[role=switch]").click());
        assertEquals(200, toggled.status());
        assertTrue(toggled.text().contains("\"isSuccess\":true"));
        assertEquals(false, namedRule(api(alice, "GET", "/getTxtTocRules", null), name).get("enable"));
        assertEquals("false", row.locator("button[role=switch]").getAttribute("aria-checked"));
        screenshot(alice, "disabled");

        row.locator("button[title='删除规则']").click();
        Locator confirmation = alice.locator("[aria-label='删除 txtTocRule']");
        confirmation.waitFor();
        Response removed = alice.waitForResponse(response -> isRuleWrite(response, "/deleteTxtTocRule"),
                () -> confirmation.locator(".danger-btn").click());
        assertEquals(200, removed.status());
        assertTrue(removed.text().contains("\"isSuccess\":true"));
        confirmation.waitFor(new Locator.WaitForOptions().setState(WaitForSelectorState.HIDDEN));
        assertEquals(0, row.count());
        assertFalse(ruleNames(api(alice, "GET", "/getTxtTocRules", null)).contains(name));
        assertFalse(ruleNames(api(bob, "GET", "/getTxtTocRules", null)).contains(name));
        screenshot(alice, "deleted");
    }

    private static boolean isRuleWrite(Response response, String path) {
        return "POST".equals(response.request().method())
                && URI.create(response.url()).getPath().endsWith("/reader3" + path);
    }

    @SuppressWarnings("unchecked")
    private static Map<String, Object> namedRule(Map<String, Object> response, String name) {
        assertTrue((Boolean) response.get("isSuccess"));
        return ((java.util.List<Map<String, Object>>) response.get("data")).stream()
                .filter(rule -> name.equals(rule.get("name"))).findFirst()
                .orElseThrow(() -> new AssertionError("Generated UI rule was not persisted"));
    }

    private static void screenshot(Page page, String scene) throws Exception {
        String ready;
        try (InputStream input = Vue3PreviewTxtTocRuleTest.class.getResourceAsStream("/ui-screenshot-readiness.js")) {
            assertTrue("Read-only screenshot guard must be packaged", input != null);
            ready = new String(input.readAllBytes(), StandardCharsets.UTF_8);
        }
        page.waitForFunction(ready, false);
        String directory = System.getenv("RUNNER_TEMP");
        if (directory == null || directory.isEmpty()) return;
        page.screenshot(new Page.ScreenshotOptions().setPath(
                Path.of(directory, "vue3-txt-rule-generated-" + scene + ".png")));
    }

    private static void register(Page page, String previewUrl, String prefix) {
        page.navigate(previewUrl + "/login");
        page.locator(".mode-switch button").nth(1).click();
        page.locator("input[autocomplete=username]").fill(prefix + UUID.randomUUID().toString().replace("-", "").substring(0, 10));
        page.locator("input[autocomplete=current-password]").fill("TocProbe-2026");
        page.locator(".submit-btn").click();
        page.locator(".bookshelf-page").waitFor();
    }

    @SuppressWarnings("unchecked")
    private static Map<String, Object> api(Page page, String method, String path, String body) {
        return (Map<String, Object>) page.evaluate("async ({method, path, body}) => {"
                + "const token = localStorage.getItem('reader_access_token');"
                + "const response = await fetch('/reader3' + path + '?accessToken=' + encodeURIComponent(token), {"
                + "method, headers: body ? {'content-type':'application/json'} : {}, body: body || undefined});"
                + "return await response.json(); }", Map.of("method", method, "path", path, "body", body == null ? "" : body));
    }

    @SuppressWarnings("unchecked")
    private static java.util.List<String> ruleNames(Map<String, Object> response) {
        assertTrue((Boolean) response.get("isSuccess"));
        return ((java.util.List<Map<String, Object>>) response.get("data")).stream()
                .map(rule -> String.valueOf(rule.get("name")))
                .collect(java.util.stream.Collectors.toList());
    }

    @SuppressWarnings("unchecked")
    private static Map<String, Object> dataMap(Map<String, Object> response) {
        return (Map<String, Object>) response.get("data");
    }
}
