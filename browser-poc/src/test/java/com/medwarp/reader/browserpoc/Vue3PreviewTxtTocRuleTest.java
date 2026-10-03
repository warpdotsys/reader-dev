package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
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
    public void customRulesAreCrudableAndNeverLeakAcrossUsers() {
        String previewUrl = System.getenv("READER_VUE3_PREVIEW_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(previewUrl != null && !previewUrl.isEmpty()
                && executable != null && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        URI preview = URI.create(previewUrl);
        assertTrue("Only an isolated loopback preview is allowed",
                "127.0.0.1".equals(preview.getHost()) || "localhost".equals(preview.getHost()));

        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                Page alice = browser.newPage();
                Page bob = browser.newPage();
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
            } finally {
                browser.close();
            }
        }
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
