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
import java.util.Map;
import java.util.UUID;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

/** The file page's rename action must reach a real, scoped backend operation. */
public class Vue3PreviewFileTest {
    @Test
    public void renameAndImportThroughTheFilePage() {
        String previewUrl = System.getenv("READER_VUE3_PREVIEW_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(previewUrl != null && !executable.isEmpty()
                && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        URI uri = URI.create(previewUrl);
        assertEquals("http", uri.getScheme());
        assertTrue("Only an isolated loopback preview is allowed",
                "127.0.0.1".equals(uri.getHost()) || "localhost".equals(uri.getHost()));

        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                Page page = browser.newPage();
                page.setDefaultTimeout(30000);
                page.navigate(previewUrl + "/login");
                page.locator(".mode-switch button").nth(1).click();
                page.locator("input[autocomplete=username]").fill("file" +
                        UUID.randomUUID().toString().replace("-", "").substring(0, 10));
                page.locator("input[autocomplete=current-password]").fill("FileProbe-2026");
                page.locator(".submit-btn").click();
                page.locator(".bookshelf-page").waitFor();
                page.navigate(previewUrl + "/files");
                page.locator(".file-page").waitFor();
                page.locator(".home-pills button:has-text('用户数据')").click();

                String suffix = UUID.randomUUID().toString().substring(0, 8);
                String oldName = "rename-before-" + suffix + ".txt";
                String newName = "rename-after-" + suffix + ".txt";
                String collisionName = "rename-existing-" + suffix + ".txt";
                String importDir = "import-dir-" + suffix;
                assertTrue(api(page, "/file/save", Map.of("path", "/" + oldName,
                        "content", "reader-rename-preserves-content", "home", "__HOME__")));
                assertTrue(api(page, "/file/save", Map.of("path", "/" + collisionName,
                        "content", "do-not-overwrite", "home", "__HOME__")));
                try {
                    page.locator(".home-pills button:has-text('根')").click();
                    page.locator(".home-pills button:has-text('用户数据')").click();
                    page.locator(".row").filter(new com.microsoft.playwright.Locator.FilterOptions()
                            .setHasText(oldName)).locator(".row-select").click();
                    page.locator(".toolbar button:has-text('重命名')").click();
                    page.locator(".dlg-overlay .dlg-input").fill(newName);
                    page.locator(".dlg-overlay .btn-primary").click();
                    page.locator(".row-name:text-is('" + newName + "')").waitFor();

                    assertEquals("reader-rename-preserves-content",
                            apiData(page, "/file/get", "/" + newName));
                    assertFalse(api(page, "/file/rename", Map.of("path", "/" + newName,
                            "name", "../escape.txt", "home", "__HOME__")));
                    assertFalse(api(page, "/file/rename", Map.of("path", "/" + newName,
                            "name", collisionName, "home", "__HOME__")));
                    assertEquals("do-not-overwrite",
                            apiData(page, "/file/get", "/" + collisionName));

                    page.locator(".row").filter(new com.microsoft.playwright.Locator.FilterOptions()
                            .setHasText(newName)).locator(".row-select").click();
                    page.locator(".toolbar button:has-text('导入书架')").click();
                    Response imported = page.waitForResponse(
                            response -> URI.create(response.url()).getPath()
                                    .endsWith("/reader3/scanLocalBookDir"),
                            () -> page.locator(".dlg-overlay .btn-primary").click());
                    @SuppressWarnings("unchecked")
                    Map<String, Object> result = (Map<String, Object>)
                            page.evaluate("text => JSON.parse(text)", imported.text());
                    assertTrue(Boolean.TRUE.equals(result.get("isSuccess")));
                    @SuppressWarnings("unchecked")
                    Map<String, Object> counts = (Map<String, Object>) result.get("data");
                    assertEquals(1, ((Number) counts.get("imported")).intValue());
                    assertEquals(0, ((Number) counts.get("failed")).intValue());

                    assertTrue(api(page, "/file/save", Map.of("path", "/" + importDir + "/one.txt",
                            "content", "第一章 起始\n正文一", "home", "__HOME__")));
                    assertTrue(api(page, "/file/save", Map.of("path", "/" + importDir + "/nested/two.txt",
                            "content", "第一章 起始\n正文二", "home", "__HOME__")));
                    page.locator(".home-pills button:has-text('根')").click();
                    page.locator(".home-pills button:has-text('用户数据')").click();
                    page.locator(".row").filter(new com.microsoft.playwright.Locator.FilterOptions()
                            .setHasText(importDir)).locator(".row-main").click();
                    page.locator(".toolbar button:has-text('导入目录')").click();
                    Response directoryImport = page.waitForResponse(
                            response -> URI.create(response.url()).getPath()
                                    .endsWith("/reader3/scanLocalBookDir"),
                            () -> page.locator(".dlg-overlay .btn-primary").click());
                    @SuppressWarnings("unchecked")
                    Map<String, Object> directoryResult = (Map<String, Object>)
                            page.evaluate("text => JSON.parse(text)", directoryImport.text());
                    assertTrue(Boolean.TRUE.equals(directoryResult.get("isSuccess")));
                    @SuppressWarnings("unchecked")
                    Map<String, Object> directoryCounts = (Map<String, Object>) directoryResult.get("data");
                    assertEquals(2, ((Number) directoryCounts.get("imported")).intValue());
                    assertEquals(0, ((Number) directoryCounts.get("failed")).intValue());
                } finally {
                    api(page, "/file/delete", Map.of("path", "/" + oldName, "home", "__HOME__"));
                    api(page, "/file/delete", Map.of("path", "/" + newName, "home", "__HOME__"));
                    api(page, "/file/delete", Map.of("path", "/" + collisionName, "home", "__HOME__"));
                    api(page, "/file/delete", Map.of("path", "/" + importDir, "home", "__HOME__"));
                }
            } finally {
                browser.close();
            }
        }
    }

    private static boolean api(Page page, String path, Map<String, String> body) {
        Object result = page.evaluate("async ({path, body}) => {" +
                "const token = localStorage.getItem('reader_access_token');" +
                "const response = await fetch('/reader3' + path + '?accessToken=' + encodeURIComponent(token)," +
                "{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(body)});" +
                "return (await response.json()).isSuccess; }", Map.of("path", path, "body", body));
        return Boolean.TRUE.equals(result);
    }

    private static String apiData(Page page, String path, String file) {
        Object result = page.evaluate("async ({path, file}) => {" +
                "const token = localStorage.getItem('reader_access_token');" +
                "const query = new URLSearchParams({accessToken:token,path:file,home:'__HOME__'});" +
                "const response = await fetch('/reader3' + path + '?' + query);" +
                "const result = await response.json();" +
                "if (!result.isSuccess) throw new Error(result.errorMsg);" +
                "return result.data; }", Map.of("path", path, "file", file));
        return String.valueOf(result);
    }
}
