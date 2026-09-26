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
import static org.junit.Assert.assertTrue;

/** Real Vue 3 pages against a deterministic loopback book source and isolated Reader. */
public class Vue3PreviewReadingTest {
    private static void requireLoopback(String value) {
        URI uri = URI.create(value);
        assertEquals("http", uri.getScheme());
        assertTrue("Only loopback fixtures are allowed", "127.0.0.1".equals(uri.getHost())
                || "localhost".equals(uri.getHost()));
    }

    @Test
    public void bookDetailTocAndFirstChapterRender() throws Exception {
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
                page.navigate(previewUrl + "/login");
                page.locator(".mode-switch button").nth(1).click();
                page.locator("input[autocomplete=username]").fill("vue" +
                        UUID.randomUUID().toString().replace("-", "").substring(0, 10));
                page.locator("input[autocomplete=current-password]").fill("ReadingProbe-2026");
                page.locator(".submit-btn").click();
                page.locator(".bookshelf-page").waitFor();

                byte[] scriptBytes = getClass().getResourceAsStream("/vue3-reading-setup.js").readAllBytes();
                String setupScript = new String(scriptBytes, java.nio.charset.StandardCharsets.UTF_8);
                String bookUrl = (String) page.evaluate(setupScript, fixtureUrl);

                page.navigate(previewUrl + "/book/" + java.net.URLEncoder.encode(bookUrl,
                        java.nio.charset.StandardCharsets.UTF_8));
                page.locator(".detail-page").waitFor();
                page.locator(".tabs .tab").nth(1).click();
                page.locator(".toc-item").first().waitFor();
                assertEquals(2, page.locator(".toc-item").count());
                page.locator(".toc-item").first().click();
                page.locator(".reader-page").waitFor();
                try {
                    page.getByText("第一段，中文与 UTF-8。").waitFor(
                            new com.microsoft.playwright.Locator.WaitForOptions().setTimeout(12000));
                } catch (RuntimeException failure) {
                    Object apiReply = page.evaluate("" +
                            "(args) => fetch('/reader3/getBookContent?url=' + encodeURIComponent(args.bookUrl)" +
                            " + '&chapterUrl=' + encodeURIComponent(args.fixtureUrl + '/chapter/1'))" +
                            ".then(response => response.text())",
                            Map.of("bookUrl", bookUrl, "fixtureUrl", fixtureUrl));
                    throw new AssertionError("Reader at " + page.url() + " displays: " +
                            page.locator(".reader-page").innerText().substring(0,
                                    Math.min(1200, page.locator(".reader-page").innerText().length())) +
                            "; direct API reply=" + apiReply, failure);
                }
                assertTrue(page.locator(".reader-content").innerText()
                        .contains("第一段，中文与 UTF-8。"));
                Response progress = page.waitForResponse(response -> URI.create(response.url()).getPath()
                                .endsWith("/reader3/saveBookProgress"),
                        () -> page.locator(".chapter-nav button").last().click());
                assertEquals(200, progress.status());
                assertTrue("The legacy progress endpoint must accept the Vue 3 chapter update",
                        progress.text().contains("\"isSuccess\":true"));
                page.getByText("终章内容固定。").waitFor();
                assertTrue(page.locator(".reader-content").innerText()
                        .contains("终章内容固定。"));

                page.navigate(previewUrl + "/book/" + java.net.URLEncoder.encode(bookUrl,
                        java.nio.charset.StandardCharsets.UTF_8));
                page.locator(".detail-page").waitFor();
                page.locator(".detail-page button.search-btn:has-text('导出')").click();
                page.locator(".dlg-export").waitFor();
                Response txt = page.waitForResponse(response -> URI.create(response.url()).getPath()
                                .endsWith("/reader3/exportBook"),
                        () -> page.locator(".dlg-export .accent-btn:has-text('导出')").click());
                assertEquals(200, txt.status());
                assertTrue("TXT export must send the legacy isEpub=0 contract",
                        URI.create(txt.url()).getQuery().contains("isEpub=0"));
                assertTrue(txt.body().length > 20);

                page.locator(".dlg-export .fmt-btn").nth(1).click();
                Response epub = page.waitForResponse(response -> URI.create(response.url()).getPath()
                                .endsWith("/reader3/exportBook"),
                        () -> page.locator(".dlg-export .accent-btn:has-text('导出')").click());
                assertEquals(200, epub.status());
                assertTrue("EPUB export must send the legacy isEpub=1 contract",
                        URI.create(epub.url()).getQuery().contains("isEpub=1"));
                byte[] epubBytes = epub.body();
                assertTrue("EPUB export must be a ZIP archive", epubBytes.length > 100
                        && epubBytes[0] == 'P' && epubBytes[1] == 'K');
                assertEquals(2, page.locator(".dlg-export .fmt-btn").count());
            } finally {
                browser.close();
            }
        }
    }
}
