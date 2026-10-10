package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.FileChooser;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import com.microsoft.playwright.Response;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.UUID;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

/**
 * Browser-level contract for the Java/Kotlin local-book flow:
 * importBookPreview persists/parses the file, then saveBook confirms shelf import.
 * It deliberately rejects the retired Rust-only /uploadLocalBook contract.
 */
public class Vue3PreviewBookshelfImportTest {
    @Test
    public void previewThenConfirmAddsTxtBookToShelfWithoutUploadLocalBook() throws Exception {
        String previewUrl = System.getenv("READER_VUE3_PREVIEW_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(previewUrl != null && !previewUrl.isEmpty()
                && !executable.isEmpty() && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        URI preview = URI.create(previewUrl);
        assertEquals("http", preview.getScheme());
        assertTrue("Only an isolated loopback preview is allowed",
                "127.0.0.1".equals(preview.getHost()) || "localhost".equals(preview.getHost()));

        String title = "Vue3 导入 " + UUID.randomUUID().toString().substring(0, 8);
        Path fixture = Files.createTempFile("reader-vue3-import-", ".txt");
        Files.writeString(fixture, "第一章 开始\n" + title + " 正文\n第二章 继续\n结束", StandardCharsets.UTF_8);
        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                Page page = browser.newPage();
                page.setDefaultTimeout(30000);
                List<String> requestedPaths = new ArrayList<>();
                page.onRequest(request -> requestedPaths.add(URI.create(request.url()).getPath()));
                register(page, previewUrl);
                assertEquals(0, shelfCount(page));

                page.locator(".import-btn").click();
                page.locator("[aria-label='导入本地书籍']").waitFor();
                FileChooser chooser = page.waitForFileChooser(
                        () -> page.locator(".dropzone").click());
                Response parsed = page.waitForResponse(
                        response -> URI.create(response.url()).getPath().endsWith("/reader3/importBookPreview"),
                        () -> chooser.setFiles(fixture));
                assertEquals(200, parsed.status());
                assertTrue(parsed.text().contains("\"isSuccess\":true"));
                try {
                    page.locator(".preview-panel").waitFor(
                            new com.microsoft.playwright.Locator.WaitForOptions().setTimeout(8000));
                } catch (RuntimeException failure) {
                    throw new AssertionError("Import preview was not shown; API=" + parsed.text()
                            + "; dialog=" + page.locator("[aria-label='导入本地书籍']").innerText(), failure);
                }
                assertTrue(page.locator(".preview-panel").textContent()
                        .contains(fixture.getFileName().toString().replace(".txt", "")));
                assertEquals(0, shelfCount(page));

                Response saved = page.waitForResponse(
                        response -> URI.create(response.url()).getPath().endsWith("/reader3/saveBook"),
                        () -> page.locator("[aria-label='导入本地书籍'] .accent-btn").click());
                assertEquals(200, saved.status());
                assertTrue(saved.text().contains("\"isSuccess\":true"));
                page.locator(".bookshelf-page").waitFor();
                assertEquals(1, shelfCount(page));
                assertFalse("Vue3 must not call the nonexistent Rust upload route",
                        requestedPaths.stream().anyMatch(path -> path.endsWith("/reader3/uploadLocalBook")));
            } finally {
                browser.close();
            }
        } finally {
            Files.deleteIfExists(fixture);
        }
    }

    private static void register(Page page, String previewUrl) {
        page.navigate(previewUrl + "/login");
        page.locator(".mode-switch button").nth(1).click();
        page.locator("input[autocomplete=username]").fill("import" +
                UUID.randomUUID().toString().replace("-", "").substring(0, 10));
        page.locator("input[autocomplete=current-password]").fill("ImportProbe-2026");
        page.locator(".submit-btn").click();
        page.locator(".bookshelf-page").waitFor();
    }

    private static int shelfCount(Page page) {
        Object value = page.evaluate("async () => {" +
                "const token = localStorage.getItem('reader_access_token');" +
                "const response = await fetch('/reader3/getBookshelf?accessToken=' + encodeURIComponent(token));" +
                "const body = await response.json();" +
                "if (!body.isSuccess || !Array.isArray(body.data)) throw new Error(body.errorMsg || 'bookshelf failed');" +
                "return body.data.length; }");
        return ((Number) value).intValue();
    }
}
