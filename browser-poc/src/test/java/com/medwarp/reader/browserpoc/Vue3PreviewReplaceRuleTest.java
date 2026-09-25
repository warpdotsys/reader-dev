package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Locator;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.UUID;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;

/** Verifies the Vue 3 replacement-rule adapter against the isolated legacy API. */
public class Vue3PreviewReplaceRuleTest {
    @Test
    @SuppressWarnings("unchecked")
    public void ruleCrudUsesLegacyLongPatternAndEntityDeleteContract() throws Exception {
        String previewUrl = System.getenv("READER_VUE3_PREVIEW_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(previewUrl != null && !executable.isEmpty()
                && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        requireLoopback(previewUrl);

        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                Page page = browser.newPage();
                page.setDefaultTimeout(30000);
                register(page, previewUrl);
                page.navigate(previewUrl + "/rules");
                page.locator(".rules-page").waitFor();
                page.getByText("暂无规则，点击右上角新增").waitFor();

                String suffix = UUID.randomUUID().toString().replace("-", "").substring(0, 9);
                String originalName = "契约规则" + suffix;
                String renamedName = "改名规则" + suffix;
                createRule(page, originalName, "旧广告" + suffix, "已替换");

                Map<String, Object> stored = findRule(getRules(page), originalName);
                assertNotNull("Rule must be persisted by the legacy endpoint", stored);
                assertTrue("Legacy id must be a JSON number accepted by Kotlin Long",
                        stored.get("id") instanceof Number);
                assertEquals("旧广告" + suffix, stored.get("pattern"));
                assertEquals("已替换", stored.get("replacement"));
                assertEquals(Boolean.TRUE, stored.get("isEnabled"));
                assertFalse("Vue-only names must not leak into the legacy payload", stored.containsKey("find"));

                Locator originalRow = row(page, originalName);
                originalRow.locator("button[role='switch']").click();
                stored = null;
                long toggleDeadline = System.currentTimeMillis() + 10000;
                while (System.currentTimeMillis() < toggleDeadline) {
                    stored = findRule(getRules(page), originalName);
                    if (stored != null && Boolean.FALSE.equals(stored.get("isEnabled"))) break;
                    Thread.sleep(100);
                }
                assertNotNull("Toggle request must reach the legacy controller", stored);
                assertEquals("Toggle should persist the legacy isEnabled property", Boolean.FALSE,
                        stored.get("isEnabled"));

                originalRow = row(page, originalName);
                originalRow.locator("button").filter(new Locator.FilterOptions().setHasText("编辑")).click();
                Locator dialog = page.locator(".dlg[aria-label='编辑替换规则']");
                dialog.locator(".field-input").nth(0).fill(renamedName);
                dialog.locator(".dlg-actions .accent-btn").click();
                row(page, renamedName).waitFor();
                List<Map<String, Object>> afterRename = getRules(page);
                Map<String, Object> renamedStored = findRule(afterRename, renamedName);
                assertNotNull(renamedStored);
                assertFalse("Rename must not reuse an id that could evict its local mirror",
                        stored.get("id").equals(renamedStored.get("id")));
                assertEquals("Rename must remove its previous name-keyed row", null,
                        findRule(afterRename, originalName));

                // Full reload proves the page maps actual legacy rows back to the Vue view model.
                page.reload();
                row(page, renamedName).waitFor();
                String bulkOne = "批量甲" + suffix;
                String bulkTwo = "批量乙" + suffix;
                createRule(page, bulkOne, "批量查找甲" + suffix, "A");
                createRule(page, bulkTwo, "批量查找乙" + suffix, "B");

                row(page, renamedName).locator("input.row-check").check();
                row(page, bulkOne).locator("input.row-check").check();
                row(page, bulkTwo).locator("input.row-check").check();
                page.locator(".bulk-bar .danger-btn").click();
                page.locator(".dlg-confirm .danger-btn").click();
                row(page, renamedName).waitFor(new Locator.WaitForOptions()
                        .setState(com.microsoft.playwright.options.WaitForSelectorState.DETACHED));
                assertEquals("Batch delete sends entity array accepted by CURD.deleteMulti", null,
                        findRule(getRules(page), renamedName));
                assertEquals(null, findRule(getRules(page), bulkOne));
                assertEquals(null, findRule(getRules(page), bulkTwo));
            } finally {
                browser.close();
            }
        }
    }

    private static void requireLoopback(String value) {
        URI uri = URI.create(value);
        assertEquals("http", uri.getScheme());
        assertTrue("Only isolated loopback previews are allowed", "127.0.0.1".equals(uri.getHost())
                || "localhost".equals(uri.getHost()));
    }

    private static void register(Page page, String previewUrl) {
        page.navigate(previewUrl + "/login");
        page.locator(".mode-switch button").nth(1).click();
        page.locator("input[autocomplete=username]").fill("rule" +
                UUID.randomUUID().toString().replace("-", "").substring(0, 10));
        page.locator("input[autocomplete=current-password]").fill("RulesProbe-2026");
        page.locator(".submit-btn").click();
        page.locator(".bookshelf-page").waitFor();
    }

    private static void createRule(Page page, String name, String find, String replacement) {
        page.locator(".section-head .add-btn").click();
        Locator dialog = page.locator(".dlg[aria-label='编辑替换规则']");
        dialog.locator(".field-input").nth(0).fill(name);
        dialog.locator(".field-input").nth(1).fill(find);
        dialog.locator(".field-input").nth(2).fill(replacement);
        dialog.locator(".dlg-actions .accent-btn").click();
        row(page, name).waitFor();
    }

    private static Locator row(Page page, String name) {
        return page.locator(".rule-table tbody tr")
                .filter(new Locator.FilterOptions().setHasText(name));
    }

    @SuppressWarnings("unchecked")
    private static List<Map<String, Object>> getRules(Page page) {
        return (List<Map<String, Object>>) page.evaluate("async () => {" +
                "const token = localStorage.getItem('reader_access_token');" +
                "const response = await fetch('/reader3/getReplaceRules?accessToken=' + encodeURIComponent(token));" +
                "const result = await response.json();" +
                "if (!response.ok || !result.isSuccess) throw new Error(result.errorMsg || response.status);" +
                "return result.data; }");
    }

    private static Map<String, Object> findRule(List<Map<String, Object>> rules, String name) {
        return rules.stream().filter(rule -> name.equals(rule.get("name"))).findFirst().orElse(null);
    }
}
