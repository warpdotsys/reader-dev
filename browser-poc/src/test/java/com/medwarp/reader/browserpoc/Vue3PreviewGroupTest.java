package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Locator;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.stream.Collectors;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

/** Exercises the Vue 3 group path against an isolated legacy Reader. */
public class Vue3PreviewGroupTest {
    @Test
    @SuppressWarnings("unchecked")
    public void groupCrudAndMultiMembershipUseLegacyStorageContract() throws Exception {
        String previewUrl = System.getenv("READER_VUE3_PREVIEW_URL");
        String fixtureUrl = System.getenv("READER_BOOK_FIXTURE_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(previewUrl != null && fixtureUrl != null
                && !executable.isEmpty() && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        requireLoopback(previewUrl);
        requireLoopback(fixtureUrl);

        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                Page page = browser.newPage();
                page.setDefaultTimeout(60000);
                register(page, previewUrl);

                String username = (String) page.evaluate("localStorage.getItem('reader_username')");
                byte[] scriptBytes = getClass().getResourceAsStream("/vue3-reading-setup.js").readAllBytes();
                String setupScript = new String(scriptBytes, StandardCharsets.UTF_8);
                String bookUrl = (String) page.evaluate(setupScript, fixtureUrl);
                page.reload();
                page.locator(".book-card").first().waitFor();

                String suffix = UUID.randomUUID().toString().replace("-", "").substring(0, 8);
                String firstName = "分组甲" + suffix;
                String secondName = "分组乙" + suffix;
                page.locator(".group-manage").first().click();
                createGroup(page, firstName);
                createGroup(page, secondName);

                Locator firstRow = page.locator(".group-row")
                        .filter(new Locator.FilterOptions().setHasText(firstName));
                firstRow.locator("button[title='重命名']").click();
                String renamed = "已重命名" + suffix;
                page.locator(".group-rename-input").fill(renamed);
                page.locator(".group-row button[title='保存重命名']").click();
                page.locator(".group-row").filter(new Locator.FilterOptions().setHasText(renamed)).waitFor();

                // Built-in negative groups are filters, not mutable user groups.
                Locator allGroup = page.locator(".group-row")
                        .filter(new Locator.FilterOptions().setHasText("全部"));
                assertEquals(0, allGroup.locator("button[title='删除分组']").count());

                page.locator("[aria-label='分组管理'] button[title='关闭']").click();
                page.locator("[aria-label='分组管理']").waitFor(new Locator.WaitForOptions()
                        .setState(com.microsoft.playwright.options.WaitForSelectorState.DETACHED));
                page.locator(".book-card").first().locator(".card-menu-btn").click();
                page.locator(".ctx-item").filter(new Locator.FilterOptions().setHasText("设置分组")).click();
                checkGroup(page, renamed);
                checkGroup(page, secondName);
                page.locator("[aria-label='设置分组'] .dlg-foot .accent-btn").click();

                Map<String, Object> groupedBook = getBook(page, bookUrl);
                long firstId = getGroupId(page, renamed);
                long secondId = getGroupId(page, secondName);
                assertEquals("Selected group bits must be combined in legacy Book.group",
                        firstId | secondId, ((Number) groupedBook.get("group")).longValue());

                // Reopening after a full reload checks mask decoding, not only local UI state.
                page.reload();
                page.locator(".book-card").first().waitFor();
                page.locator(".book-card").first().locator(".card-menu-btn").click();
                page.locator(".ctx-item").filter(new Locator.FilterOptions().setHasText("设置分组")).click();
                assertTrue(groupCheckbox(page, renamed).isChecked());
                assertTrue(groupCheckbox(page, secondName).isChecked());
                page.locator("[aria-label='设置分组'] .dlg-close").click();

                // Deleting an assigned group first clears that bit through the legacy XOR endpoint.
                page.locator(".group-manage").first().click();
                Locator secondRow = page.locator(".group-row")
                        .filter(new Locator.FilterOptions().setHasText(secondName));
                secondRow.locator("button[title='删除分组']").click();
                page.locator(".el-message-box__btns button.el-button--primary").click();
                page.locator(".group-row").filter(new Locator.FilterOptions().setHasText(secondName)).waitFor(
                        new Locator.WaitForOptions().setState(com.microsoft.playwright.options.WaitForSelectorState.DETACHED));
                Map<String, Object> remainingBook = getBook(page, bookUrl);
                assertEquals(firstId, ((Number) remainingBook.get("group")).longValue());

                assertFalse("Renamed group replaces the old name", getGroupNames(page).contains(firstName));
                assertTrue("Renamed group is persisted", getGroupNames(page).contains(renamed));
                assertTrue("The isolated account remains active", username.equals(
                        page.evaluate("localStorage.getItem('reader_username')")));
            } finally {
                browser.close();
            }
        }
    }

    private static void requireLoopback(String value) {
        URI uri = URI.create(value);
        assertEquals("http", uri.getScheme());
        assertTrue("Only isolated loopback fixtures are allowed", "127.0.0.1".equals(uri.getHost())
                || "localhost".equals(uri.getHost()));
    }

    private static void register(Page page, String previewUrl) {
        page.navigate(previewUrl + "/login");
        page.locator(".mode-switch button").nth(1).click();
        page.locator("input[autocomplete=username]").fill("vue" +
                UUID.randomUUID().toString().replace("-", "").substring(0, 10));
        page.locator("input[autocomplete=current-password]").fill("GroupsProbe-2026");
        page.locator(".submit-btn").click();
        page.locator(".bookshelf-page").waitFor();
    }

    private static void createGroup(Page page, String name) {
        page.locator(".group-create input.group-input").fill(name);
        page.locator(".group-create .accent-btn").click();
        page.locator(".group-row").filter(new Locator.FilterOptions().setHasText(name)).waitFor();
    }

    private static void checkGroup(Page page, String name) {
        Locator row = page.locator(".book-group-panel-row")
                .filter(new Locator.FilterOptions().setHasText(name));
        assertEquals("Expected exactly one group option: " + name, 1, row.count());
        row.locator("input[type=checkbox]").check();
    }

    private static Locator groupCheckbox(Page page, String name) {
        return page.locator(".book-group-panel-row")
                .filter(new Locator.FilterOptions().setHasText(name))
                .locator("input[type=checkbox]");
    }

    private static long getGroupId(Page page, String name) {
        List<Map<String, Object>> groups = getGroups(page);
        return groups.stream()
                .filter(group -> name.equals(group.get("groupName")))
                .map(group -> ((Number) group.get("groupId")).longValue())
                .findFirst()
                .orElseThrow(() -> new AssertionError("Missing persisted legacy group " + name));
    }

    private static List<String> getGroupNames(Page page) {
        return getGroups(page).stream().map(group -> String.valueOf(group.get("groupName")))
                .collect(Collectors.toList());
    }

    @SuppressWarnings("unchecked")
    private static List<Map<String, Object>> getGroups(Page page) {
        return (List<Map<String, Object>>) page.evaluate("async () => {" +
                "const token = localStorage.getItem('reader_access_token');" +
                "const response = await fetch('/reader3/getBookGroups?accessToken=' + encodeURIComponent(token));" +
                "const result = await response.json();" +
                "if (!response.ok || !result.isSuccess) throw new Error(result.errorMsg || response.status);" +
                "return result.data; }");
    }

    @SuppressWarnings("unchecked")
    private static Map<String, Object> getBook(Page page, String bookUrl) {
        return (Map<String, Object>) page.evaluate("async (url) => {" +
                "const token = localStorage.getItem('reader_access_token');" +
                "const response = await fetch('/reader3/getBookshelf?accessToken=' + encodeURIComponent(token));" +
                "const result = await response.json();" +
                "if (!response.ok || !result.isSuccess) throw new Error(result.errorMsg || response.status);" +
                "return result.data.find(book => book.bookUrl === url); }", bookUrl);
    }
}
