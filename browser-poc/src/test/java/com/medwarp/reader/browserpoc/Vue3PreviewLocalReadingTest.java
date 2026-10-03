package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.FileChooser;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import com.microsoft.playwright.Response;
import com.microsoft.playwright.Request;
import com.microsoft.playwright.Route;
import com.microsoft.playwright.TimeoutError;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Consumer;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import static org.junit.Assert.*;

/** Only generated fixtures: no private book, production account or remote source. */
public class Vue3PreviewLocalReadingTest {
    private static final String EPUB_TEXT = "合成 EPUB 第一段，中文 & 标点 <明>。";
    private static final String EPUB_LAST = "合成 EPUB 第二段，翻章与刷新。";
    private static final String TXT_TEXT = "合成 TXT 第一段，中文与 UTF-8。";
    private static final String TXT_LAST = "合成 TXT 第二段，翻章与刷新。";

    @Test
    public void generatedTxtAndEpubImportReadNavigateAndRefresh() throws Exception {
        String base = System.getenv("READER_VUE3_PREVIEW_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(base != null && !executable.isEmpty()
                && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        URI uri = URI.create(base);
        assertEquals("http", uri.getScheme());
        assertTrue("Only isolated loopback data is permitted",
                "127.0.0.1".equals(uri.getHost()) || "localhost".equals(uri.getHost()));
        Path txt = Files.createTempFile("synthetic-local-", ".txt");
        Path epub = Files.createTempFile("synthetic-local-", ".epub");
        Path fragments = Files.createTempFile("synthetic-fragments-", ".epub");
        Path navFragments = Files.createTempFile("synthetic-nav-fragments-", ".epub");
        Path cssFixture = Files.createTempFile("synthetic-nested-css-", ".epub");
        Path budgetFixture = Files.createTempFile("synthetic-budget-", ".epub");
        Files.writeString(txt, "第一章 起点\n" + TXT_TEXT + "\n第二章 终点\n" + TXT_LAST,
                StandardCharsets.UTF_8);
        writeEpub(epub);
        writeFragmentEpub(fragments);
        writeNavFragmentEpub(navFragments);
        writeCssEpub(cssFixture, false);
        writeCssEpub(budgetFixture, true);
        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                exportGeneratedFixture(fragments, "generated-ncx-fragments.epub");
                exportGeneratedFixture(navFragments, "generated-nav-fragments.epub");
                Page page = browser.newPage();
                page.setDefaultTimeout(30000);
                page.navigate(base + "/login");
                page.locator(".mode-switch button").nth(1).click();
                page.locator("input[autocomplete=username]").fill("local" +
                        UUID.randomUUID().toString().replace("-", "").substring(0, 10));
                page.locator("input[autocomplete=current-password]").fill("LocalProbe-2026");
                page.locator(".submit-btn").click();
                page.locator(".bookshelf-page").waitFor();
                String txtUrl = importBook(page, txt);
                verifyReading(page, base, txtUrl, TXT_TEXT, TXT_LAST, false);
                page.navigate(base);
                page.locator(".bookshelf-page").waitFor();
                String epubUrl = importBook(page, epub);
                verifyReading(page, base, epubUrl, EPUB_TEXT, EPUB_LAST, true);
                Path screenshot = Path.of(System.getenv().getOrDefault("RUNNER_TEMP",
                        System.getProperty("java.io.tmpdir")), "vue3-local-reading-synthetic.png");
                page.screenshot(new Page.ScreenshotOptions().setPath(screenshot));
                page.navigate(base);
                page.locator(".bookshelf-page").waitFor();
                String fragmentUrl = importBook(page, fragments);
                verifyFragmentReading(page, base, fragmentUrl);
                // The NCX journey finishes in raw mode; reset using the real UI.
                page.locator("button[title^='EPUB 排版模式']").click();
                page.locator(".reader-content:not(.epub-html)").waitFor();
                page.navigate(base);
                page.locator(".bookshelf-page").waitFor();
                String navUrl = importBook(page, navFragments);
                verifyNavFragmentReading(page, base, navUrl);
                page.navigate(base);
                page.locator(".bookshelf-page").waitFor();
                String cssUrl = importBook(page, cssFixture);
                verifyNestedCss(page, base, cssUrl);
                page.navigate(base);
                page.locator(".bookshelf-page").waitFor();
                String budgetUrl = importBook(page, budgetFixture);
                verifyBudgetRejection(page, base, budgetUrl);
                verifyCancelledDownload(page, base, cssUrl);
                verifyDownloadDeadlineAndRetry(page, base, cssUrl);
            } finally {
                browser.close();
            }
        } finally {
            Files.deleteIfExists(txt);
            Files.deleteIfExists(epub);
            Files.deleteIfExists(fragments);
            Files.deleteIfExists(navFragments);
            Files.deleteIfExists(cssFixture);
            Files.deleteIfExists(budgetFixture);
        }
    }

    private static String importBook(Page page, Path fixture) {
        page.locator(".import-btn").click();
        page.locator("[aria-label='导入本地书籍']").waitFor();
        FileChooser chooser = page.waitForFileChooser(() -> page.locator(".dropzone").click());
        Response preview = page.waitForResponse(response -> URI.create(response.url()).getPath()
                .endsWith("/reader3/importBookPreview"), () -> chooser.setFiles(fixture));
        assertEquals(200, preview.status());
        assertTrue(preview.text().contains("\"isSuccess\":true"));
        page.locator(".preview-panel").waitFor();
        Response saved = page.waitForResponse(response -> URI.create(response.url()).getPath()
                .endsWith("/reader3/saveBook"),
                () -> page.locator("[aria-label='导入本地书籍'] .accent-btn").click());
        assertEquals(200, saved.status());
        assertTrue(saved.text().contains("\"isSuccess\":true"));
        if (fixture.toString().endsWith(".epub")) {
            // Only our generated fixture: explicitly choose NCX order, unlike its reversed spine.
            Boolean configured = (Boolean) page.evaluate("async raw => {"
                    + "const book = JSON.parse(raw).data; book.tocUrl = 'toc';"
                    + "const token = localStorage.getItem('reader_access_token') || sessionStorage.getItem('reader_access_token');"
                    + "const response = await fetch('/reader3/saveBook?accessToken=' + encodeURIComponent(token || ''),"
                    + "{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(book)});"
                    + "return response.ok && (await response.json()).isSuccess === true; }", saved.text());
            assertEquals(Boolean.TRUE, configured);
        }
        String url = (String) page.evaluate("raw => JSON.parse(raw).data.bookUrl", saved.text());
        assertTrue(url.startsWith("storage/data/"));
        page.locator(".bookshelf-page").waitFor();
        return url;
    }

    private static void verifyReading(Page page, String base, String bookUrl,
                                      String first, String last, boolean epub) throws Exception {
        // URLEncoder is for form queries; Vue Router path segments need %20,
        // not '+'. The generated EPUB intentionally has spaces in its title.
        String reader = base + "/reader/" + URLEncoder.encode(bookUrl, StandardCharsets.UTF_8)
                .replace("+", "%20");
        Response initial;
        try {
            initial = page.waitForResponse(response -> URI.create(response.url()).getPath()
                    .endsWith("/reader3/getBookContent"), () -> page.navigate(reader + "?chapter=0"));
        } catch (RuntimeException failure) {
            throw new AssertionError("Generated " + (epub ? "EPUB" : "TXT")
                    + " page did not request content; page=" + page.url()
                    + "; visible=" + page.locator(".reader-page").innerText(), failure);
        }
        assertEquals(200, initial.status());
        if (epub) assertTrue("Even default text mode must request XHTML, never display an asset URL",
                URI.create(initial.url()).getQuery().contains("epubContent=1"));
        page.getByText(first, new Page.GetByTextOptions().setExact(true)).waitFor();
        assertFalse(page.locator(".reader-content").innerText().contains("/book-assets/"));
        assertFalse(page.locator(".reader-content").innerText().contains("HEAD-ONLY"));
        assertFalse(page.locator(".reader-content").innerText().contains("SCRIPT-ONLY"));
        assertFalse(page.locator(".reader-content").innerText().contains("\ufffd"));
        assertTrue("First chapter navigation must be bounded",
                !page.locator(".chapter-nav button").first().isEnabled());
        page.locator(".chapter-nav button").last().click();
        page.getByText(last, new Page.GetByTextOptions().setExact(true)).waitFor();
        page.waitForCondition(() -> page.url().contains("chapter=1"));
        page.reload();
        page.getByText(last, new Page.GetByTextOptions().setExact(true)).waitFor();
        assertFalse("Final chapter navigation must be bounded",
                page.locator(".chapter-nav button").last().isEnabled());
        page.locator(".chapter-nav button").first().click();
        page.getByText(first, new Page.GetByTextOptions().setExact(true)).waitFor();
        page.waitForCondition(() -> page.url().contains("chapter=0"));
        page.reload();
        page.getByText(first, new Page.GetByTextOptions().setExact(true)).waitFor();
        if (epub) {
            page.locator("button[title^='EPUB 排版模式']").click();
            page.locator(".reader-content.epub-html").waitFor();
            assertTrue(page.locator(".reader-content.epub-html").innerText().contains(first));
            assertEquals(0, page.locator(".reader-content.epub-html script").count());
            page.reload();
            page.locator(".reader-content.epub-html").waitFor();
            assertTrue(page.locator(".reader-content.epub-html").innerText().contains(first));
            Response downloaded = page.waitForResponse(response -> URI.create(response.url()).getPath()
                    .endsWith("/reader3/file/download"),
                    () -> page.locator("button[title^='EPUB 排版模式']").click());
            assertEquals(200, downloaded.status());
            assertTrue(URI.create(downloaded.url()).getQuery().contains("index.epub"));
            page.frameLocator("iframe.epub-frame").getByText(first,
                    new com.microsoft.playwright.FrameLocator.GetByTextOptions().setExact(true)).waitFor();
            assertEquals(0, page.frameLocator("iframe.epub-frame").locator("script").count());
            assertEquals("allow-same-origin", page.locator("iframe.epub-frame").getAttribute("sandbox"));
            assertTrue("Raw EPUB must not collapse to the browser's default 150px iframe",
                    ((Number) page.locator("iframe.epub-frame").evaluate("el => el.clientHeight")).intValue() >= 320);
            assertEquals("rgb(17, 34, 51)", page.frameLocator("iframe.epub-frame").locator("p").first()
                    .evaluate("el => getComputedStyle(el).color"));
            page.screenshot(new Page.ScreenshotOptions().setPath(Path.of(
                    System.getenv().getOrDefault("RUNNER_TEMP", System.getProperty("java.io.tmpdir")),
                    "vue3-local-reading-raw-synthetic.png")));
            verifyRawScrollRestore(page, bookUrl);
            page.frameLocator("iframe.epub-frame").getByText("书内下一章").click();
            page.waitForCondition(() -> page.url().contains("chapter=1"));
            page.frameLocator("iframe.epub-frame").getByText(last,
                    new com.microsoft.playwright.FrameLocator.GetByTextOptions().setExact(true)).waitFor();
            verifyRawScrollRestore(page, bookUrl);
            try {
                page.waitForResponse(response -> URI.create(response.url()).getPath()
                        .endsWith("/reader3/getBookContent"),
                        () -> page.locator(".chapter-nav button").first().click());
                page.waitForCondition(() -> page.url().contains("chapter=0"));
                page.frameLocator("iframe.epub-frame").getByText(first,
                        new com.microsoft.playwright.FrameLocator.GetByTextOptions().setExact(true)).waitFor();
            } catch (RuntimeException failure) {
                page.screenshot(new Page.ScreenshotOptions().setPath(Path.of(
                        System.getenv().getOrDefault("RUNNER_TEMP", System.getProperty("java.io.tmpdir")),
                        "vue3-local-reading-raw-synthetic.png")));
                throw new AssertionError("Generated EPUB previous chapter after raw refresh failed; page="
                        + page.url() + "; navigation=" + page.locator(".chapter-nav").innerText()
                        + "; frame=" + page.frameLocator("iframe.epub-frame").locator("body").innerText(), failure);
            }
            page.locator("button[title^='EPUB 排版模式']").click();
            page.locator(".reader-content:not(.epub-html)").waitFor();
            assertTrue("Switching back to text must not require a refresh",
                    page.locator(".reader-content").innerText().contains(first));
        }
    }

    private static void verifyRawScrollRestore(Page page, String bookUrl) {
        // This evaluates only our generated fixture in a dedicated test browser, never personal tabs.
        page.waitForCondition(() -> !"true".equals(page.locator("iframe.epub-frame").getAttribute("aria-busy")));
        String key = (String) page.evaluate("url => { const owner = localStorage.getItem('reader_username')"
                + " || sessionStorage.getItem('reader_username') || 'default';"
                + "return 'reader-epub-progress-' + encodeURIComponent(JSON.stringify([owner,url])); }", bookUrl);
        Response saved = page.waitForResponse(response -> URI.create(response.url()).getPath()
                .endsWith("/reader3/saveBookProgress"), () -> page.frameLocator("iframe.epub-frame")
                .locator("body").evaluate("el => el.ownerDocument.defaultView.scrollTo(0, 600)"));
        assertEquals(200, saved.status());
        assertTrue(saved.text().contains("\"isSuccess\":true"));
        assertEquals("Legacy progress transport remains {url,index}, not invented pixel fields",
                Boolean.TRUE, page.evaluate("raw => { const b = JSON.parse(raw);"
                        + "return typeof b.url === 'string' && Number.isInteger(b.index)"
                        + " && Object.keys(b).sort().join(',') === 'index,url'; }", saved.request().postData()));
        page.waitForCondition(() -> Boolean.TRUE.equals(page.evaluate("key => {"
                + "try { return JSON.parse(localStorage.getItem(key)).scrollY >= 580; } catch { return false; } }", key)));
        page.reload();
        page.locator("iframe.epub-frame").waitFor();
        page.waitForCondition(() -> "false".equals(page.locator("iframe.epub-frame").getAttribute("aria-busy")));
        Number position = (Number) page.frameLocator("iframe.epub-frame").locator("body")
                .evaluate("el => el.ownerDocument.defaultView.scrollY");
        assertTrue("Reload must restore inside the iframe, not scroll its host page; actual=" + position,
                position.doubleValue() >= 580 && position.doubleValue() <= 620);
        assertTrue("Host position must not be confused with iframe position",
                ((Number) page.evaluate("() => window.scrollY")).doubleValue() < 80);
    }

    private static String longSyntheticBody() {
        StringBuilder body = new StringBuilder();
        for (int i = 0; i < 50; i++) body.append("<p>合成滚动定位段落 ").append(i)
                .append("：仅用于验证滚动保存、字体排版及刷新恢复。</p>");
        return body.toString();
    }

    private static void verifyFragmentReading(Page page, String base, String bookUrl) {
        String reader = base + "/reader/" + URLEncoder.encode(bookUrl, StandardCharsets.UTF_8)
                .replace("+", "%20");
        Response toc = page.waitForResponse(response -> URI.create(response.url()).getPath()
                .endsWith("/reader3/getChapterList"), () -> page.navigate(reader + "?chapter=0"));
        assertEquals("Legacy NCX parser collapses shared resources; never fabricate another backend chapter",
                Boolean.TRUE, page.evaluate("raw => { const data=JSON.parse(raw).data;"
                        + "return data.length === 1 && data[0].url === 'Text/shared.xhtml'; }", toc.text()));
        page.getByText("合成锚点第二节正文", new Page.GetByTextOptions().setExact(true)).waitFor();
        // The preceding journey finishes in text mode. Keep this generated book's selection explicit.
        page.locator("button[title^='EPUB 排版模式']").click();
        page.locator(".reader-content.epub-html").waitFor();
        page.locator("button[title^='EPUB 排版模式']").click();
        waitRawReady(page);
        page.frameLocator("iframe.epub-frame").getByText("同页下一节").click();
        assertAnchorAtTop(page, "h2[id='中文目标']");
        page.frameLocator("iframe.epub-frame").getByText("同文件返回首节").click();
        assertAnchorAtTop(page, "h2[id='intro']");
        page.locator(".toc-btn").click();
        assertEquals(2, page.locator(".epub-toc-list button").count());
        page.locator(".epub-toc-list").getByText("第二节锚点", new com.microsoft.playwright.Locator.GetByTextOptions().setExact(true)).click();
        page.waitForCondition(() -> page.url().contains("epubAnchor="));
        waitRawReady(page);
        assertAnchorAtTop(page, "h2[id='中文目标']");
        assertTrue("The second TOC anchor is not the first file's start",
                innerScroll(page) > 1000);
        assertEquals(0, page.frameLocator("iframe.epub-frame").locator("script").count());
        assertEquals("allow-same-origin", page.locator("iframe.epub-frame").getAttribute("sandbox"));
        page.reload();
        waitRawReady(page);
        assertAnchorAtTop(page, "h2[id='中文目标']");
        // File-qualified and fragment-only links preserve distinct anchors at the same legacy index.
        page.frameLocator("iframe.epub-frame").getByText("同文件返回首节").click();
        page.waitForCondition(() -> page.url().contains("chapter=0"));
        waitRawReady(page);
        assertAnchorAtTop(page, "h2[id='intro']");
        page.frameLocator("iframe.epub-frame").getByText("同页未列目录注记").click();
        waitRawReady(page);
        assertAnchorAtTop(page, "a[name='note']");
        assertTrue("A non-TOC anchor must retain the current legacy chapter", page.url().contains("chapter=0"));
        page.frameLocator("iframe.epub-frame").getByText("同页下一节").click();
        page.waitForCondition(() -> page.url().contains("epubAnchor=%E4%B8%AD"));
        waitRawReady(page);
        assertAnchorAtTop(page, "h2[id='中文目标']");
        Response saved = page.waitForResponse(response -> URI.create(response.url()).getPath()
                .endsWith("/reader3/saveBookProgress"), () -> page.frameLocator("iframe.epub-frame")
                .locator("body").evaluate("el => el.ownerDocument.defaultView.scrollBy({top:180,behavior:'instant'})"));
        assertEquals(200, saved.status());
        assertTrue(saved.text().contains("\"isSuccess\":true"));
        assertEquals(Boolean.TRUE, page.evaluate("raw => JSON.parse(raw).index === 0", saved.request().postData()));
        double savedPosition = innerScroll(page);
        page.reload();
        waitRawReady(page);
        assertEquals("A saved position takes precedence over the chapter's initial anchor",
                savedPosition, innerScroll(page), 2);
        page.screenshot(new Page.ScreenshotOptions().setPath(Path.of(
                System.getenv().getOrDefault("RUNNER_TEMP", System.getProperty("java.io.tmpdir")),
                "vue3-local-reading-fragments-synthetic.png")));
        page.frameLocator("iframe.epub-frame").getByText("不存在锚点的合成链接").click();
        page.getByText("目录锚点不存在，已回到本页顶部", new Page.GetByTextOptions().setExact(true)).waitFor();
        waitRawReady(page);
        assertEquals(0, innerScroll(page), 2);
        assertTrue(page.url().contains("chapter=0"));
    }

    private static void waitRawReady(Page page) {
        page.locator("iframe.epub-frame").waitFor();
        page.waitForCondition(() -> "false".equals(page.locator("iframe.epub-frame").getAttribute("aria-busy")));
    }

    private static void verifyNavFragmentReading(Page page, String base, String bookUrl) {
        String reader = base + "/reader/" + URLEncoder.encode(bookUrl, StandardCharsets.UTF_8)
                .replace("+", "%20");
        Response toc = page.waitForResponse(response -> URI.create(response.url()).getPath()
                .endsWith("/reader3/getChapterList"), () -> page.navigate(reader + "?chapter=0"));
        assertEquals(200, toc.status());
        assertEquals("EPUB3 nested navigation still uses the two legacy resource indices",
                Boolean.TRUE, page.evaluate("raw => { const b=JSON.parse(raw); const d=b.data;"
                        + "return b.isSuccess === true && d.length === 2"
                        + " && d[0].index === 0 && d[0].url === 'Text/shared.xhtml'"
                        + " && d[1].index === 1 && d[1].url === 'Text/later.xhtml'; }", toc.text()));
        page.getByText("EPUB3 合成首节正文", new Page.GetByTextOptions().setExact(true)).waitFor();
        page.locator("button[title^='EPUB 排版模式']").click();
        page.locator(".reader-content.epub-html").waitFor();
        page.locator("button[title^='EPUB 排版模式']").click();
        waitRawReady(page);
        page.locator(".toc-btn").click();
        assertEquals("Nested nav entries are flattened without duplicates or landmark/NCX entries",
                3, page.locator(".epub-toc-list button").count());
        assertFalse(page.locator(".epub-toc-list").innerText().contains("重复目录"));
        assertFalse(page.locator(".epub-toc-list").innerText().contains("地标不是目录"));
        assertFalse(page.locator(".epub-toc-list").innerText().contains("旧 NCX 不应显示"));
        page.locator(".epub-toc-list").getByText("NAV 中文小节",
                new com.microsoft.playwright.Locator.GetByTextOptions().setExact(true)).click();
        waitRawReady(page);
        assertAnchorAtTop(page, "h2[id='中文目标']");
        assertTrue(page.url().contains("chapter=0"));
        assertTrue(innerScroll(page) > 1000);
        page.reload();
        waitRawReady(page);
        assertAnchorAtTop(page, "h2[id='中文目标']");

        page.locator(".toc-btn").click();
        Response loaded;
        try {
            loaded = readAfterOldProgressSettles(page, bookUrl,
                () -> page.locator(".epub-toc-list").getByText("NAV 后一文件",
                        new com.microsoft.playwright.Locator.GetByTextOptions().setExact(true)).click());
        } catch (RuntimeException failure) {
            // Dedicated generated-book account only: do not inspect any personal browser or shelf.
            Object storedIndex = page.evaluate("async url => { const token=localStorage.getItem('reader_access_token')"
                    + " || sessionStorage.getItem('reader_access_token');"
                    + "const r=await fetch('/reader3/getBookshelf?refresh=0&accessToken=' + encodeURIComponent(token || ''));"
                    + "const b=await r.json(); return b.data.find(book => book.bookUrl === url)?.durChapterIndex ?? null; }", bookUrl);
            page.screenshot(new Page.ScreenshotOptions().setPath(Path.of(
                    System.getenv().getOrDefault("RUNNER_TEMP", System.getProperty("java.io.tmpdir")),
                    "vue3-local-reading-nav-fragments-timeout-synthetic.png")));
            throw new AssertionError("Generated EPUB3 cross-file content was not requested; page=" + page.url()
                    + "; storedIndex=" + storedIndex + "; busy=" + page.locator("iframe.epub-frame").getAttribute("aria-busy")
                    + "; innerScroll=" + innerScroll(page)
                    + "; frameHeading=" + page.frameLocator("iframe.epub-frame").locator("h2").innerText(), failure);
        }
        assertEquals(200, loaded.status());
        assertTrue(loaded.text().contains("\"isSuccess\":true"));
        assertTrue(URI.create(loaded.url()).getQuery().contains("epubContent=1"));
        // Legacy content reads also update the shelf index; do not invent a required extra POST.
        assertEquals(1, ((Number) page.evaluate("async url => { const token=localStorage.getItem('reader_access_token')"
                + " || sessionStorage.getItem('reader_access_token');"
                + "const r=await fetch('/reader3/getBookshelf?refresh=0&accessToken=' + encodeURIComponent(token || ''));"
                + "const b=await r.json(); return b.data.find(book => book.bookUrl === url).durChapterIndex; }", bookUrl)).intValue());
        page.waitForCondition(() -> page.url().contains("chapter=1") && page.url().contains("epubAnchor=finale"));
        waitRawReady(page);
        assertAnchorAtTop(page, "h2[id='finale']");
        page.reload();
        waitRawReady(page);
        assertAnchorAtTop(page, "h2[id='finale']");
        Response saved = page.waitForResponse(response -> URI.create(response.url()).getPath()
                .endsWith("/reader3/saveBookProgress"), () -> page.frameLocator("iframe.epub-frame")
                .locator("body").evaluate("el => el.ownerDocument.defaultView.scrollBy({top:180,behavior:'instant'})"));
        assertEquals(200, saved.status());
        assertTrue(saved.text().contains("\"isSuccess\":true"));
        assertEquals("Explicit scrolling keeps the old {url,index} transport at the second file",
                Boolean.TRUE, page.evaluate("raw => { const b=JSON.parse(raw);"
                        + "return b.index === 1 && Object.keys(b).sort().join(',') === 'index,url'; }",
                        saved.request().postData()));
        double savedPosition = innerScroll(page);
        page.reload();
        waitRawReady(page);
        assertEquals(savedPosition, innerScroll(page), 2);
        page.frameLocator("iframe.epub-frame").getByText("跨文件返回中文小节").click();
        page.waitForCondition(() -> page.url().contains("chapter=0"));
        waitRawReady(page);
        assertAnchorAtTop(page, "h2[id='中文目标']");
        assertEquals(0, page.frameLocator("iframe.epub-frame").locator("script").count());
        assertEquals("allow-same-origin", page.locator("iframe.epub-frame").getAttribute("sandbox"));
        assertEquals(0, page.getByText("目录锚点不存在，已回到本页顶部",
                new Page.GetByTextOptions().setExact(true)).count());
        page.screenshot(new Page.ScreenshotOptions().setPath(Path.of(
                System.getenv().getOrDefault("RUNNER_TEMP", System.getProperty("java.io.tmpdir")),
                "vue3-local-reading-nav-fragments-synthetic.png")));
    }

    private static void verifyNestedCss(Page page, String base, String bookUrl) {
        AtomicBoolean outsideZipRequest = new AtomicBoolean();
        Consumer<Request> observer = request -> {
            if (URI.create(request.url()).getPort() == 18899) outsideZipRequest.set(true);
        };
        page.onRequest(observer);
        try {
            openGeneratedCssInTextMode(page, base, bookUrl);
            selectEpubMode(page, "原版排版");
            page.frameLocator("iframe.epub-frame").locator("#css-proof").waitFor();
            page.waitForCondition(() -> "rgb(17, 34, 51)".equals(page.frameLocator("iframe.epub-frame")
                .locator("#css-proof").evaluate("el => getComputedStyle(el).color")));
            assertEquals("700", page.frameLocator("iframe.epub-frame").locator("#css-proof")
                .evaluate("el => getComputedStyle(el).fontWeight"));
            assertEquals("7px", page.frameLocator("iframe.epub-frame").locator("#css-proof")
                .evaluate("el => getComputedStyle(el).marginLeft"));
            assertEquals("11px", page.frameLocator("iframe.epub-frame").locator("#css-proof")
                .evaluate("el => getComputedStyle(el).paddingLeft"));
            assertTrue(String.valueOf(page.frameLocator("iframe.epub-frame").locator("#css-proof")
                .evaluate("el => getComputedStyle(el).backgroundImage")).contains("blob:"));
            assertTrue((Boolean) page.frameLocator("iframe.epub-frame").locator("#inline-proof")
                .evaluate("el => getComputedStyle(el).backgroundImage.includes('blob:')"));
            page.waitForCondition(() -> (Boolean) page.frameLocator("iframe.epub-frame").locator("#image-proof")
                .evaluate("el => el.complete && el.naturalWidth === 12"));
            assertEquals(0, page.frameLocator("iframe.epub-frame").locator("script,iframe,object,embed").count());
            assertEquals("allow-same-origin", page.locator("iframe.epub-frame").getAttribute("sandbox"));
            assertFalse("Book CSS must not request the synthetic outside-ZIP endpoint", outsideZipRequest.get());
            page.screenshot(new Page.ScreenshotOptions().setPath(Path.of(
                System.getenv().getOrDefault("RUNNER_TEMP", System.getProperty("java.io.tmpdir")),
                "vue3-local-reading-css-synthetic.png")));
        } catch (RuntimeException failure) {
            page.screenshot(new Page.ScreenshotOptions().setPath(Path.of(
                System.getenv().getOrDefault("RUNNER_TEMP", System.getProperty("java.io.tmpdir")),
                "vue3-local-reading-css-timeout-synthetic.png")));
            String observed = page.locator("iframe.epub-frame").count() > 0
                && page.frameLocator("iframe.epub-frame").locator("#css-proof").count() > 0
                ? String.valueOf(page.frameLocator("iframe.epub-frame").locator("#css-proof")
                    .evaluate("el => getComputedStyle(el).color")) : "no CSS proof";
            String mode = page.locator("button[title^='EPUB 排版模式']").count() > 0
                ? page.locator("button[title^='EPUB 排版模式']").getAttribute("title") : "no mode button";
            throw new AssertionError("Generated nested CSS failed; observed color=" + observed
                + "; mode=" + mode + "; page=" + page.url(), failure);
        } finally { page.offRequest(observer); }
    }

    private static void verifyBudgetRejection(Page page, String base, String bookUrl) {
        openGeneratedCssInTextMode(page, base, bookUrl);
        selectEpubMode(page, "原版排版");
        page.locator(".epub-error").getByText("EPUB 单个资源超过原版排版上限",
            new com.microsoft.playwright.Locator.GetByTextOptions().setExact(true)).waitFor();
        assertEquals(0, page.locator("iframe.epub-frame").count());
        assertEquals("Inline errors must not duplicate a toast over the reading controls",
            0, page.locator(".el-message__content").getByText("EPUB 单个资源超过原版排版上限",
                new com.microsoft.playwright.Locator.GetByTextOptions().setExact(true)).count());
        page.screenshot(new Page.ScreenshotOptions().setPath(Path.of(
            System.getenv().getOrDefault("RUNNER_TEMP", System.getProperty("java.io.tmpdir")),
            "vue3-local-reading-budget-synthetic.png")));
        // 原文件/后端没有被截断或改写；原版排版被拒绝仍可主动切到普通模式。
        page.locator(".epub-error").getByRole(com.microsoft.playwright.options.AriaRole.BUTTON,
            new com.microsoft.playwright.Locator.GetByRoleOptions().setName("切换普通阅读").setExact(true)).click();
        page.locator(".reader-content:not(.epub-html)").getByText("生成预算样本正文仍可普通阅读",
            new com.microsoft.playwright.Locator.GetByTextOptions().setExact(true)).waitFor();
        assertEquals(0, page.locator(".epub-error").count());
        page.screenshot(new Page.ScreenshotOptions().setPath(Path.of(
            System.getenv().getOrDefault("RUNNER_TEMP", System.getProperty("java.io.tmpdir")),
            "vue3-local-reading-budget-fallback-synthetic.png")));
    }

    private static void verifyCancelledDownload(Page page, String base, String bookUrl) {
        openGeneratedCssInTextMode(page, base, bookUrl);
        AtomicReference<Route> held = new AtomicReference<>();
        AtomicBoolean aborted = new AtomicBoolean();
        Consumer<Route> delay = route -> { if (!held.compareAndSet(null, route)) route.resume(); };
        Consumer<Request> failed = request -> {
            if (URI.create(request.url()).getPath().endsWith("/reader3/file/download")
                && String.valueOf(request.failure()).contains("ERR_ABORTED")) aborted.set(true);
        };
        page.onRequestFailed(failed);
        page.route("**/reader3/file/download**", delay);
        try {
            selectEpubMode(page, "净化排版");
            page.locator(".reader-content.epub-html").waitFor();
            page.waitForRequest(request -> URI.create(request.url()).getPath().endsWith("/reader3/file/download"),
                () -> page.locator("button[title^='EPUB 排版模式']").click());
            page.waitForCondition(() -> held.get() != null);
            page.locator("button[title^='EPUB 排版模式']").click();
            page.locator(".reader-content:not(.epub-html)").waitFor();
            // Release the intercepted route so Chromium can report the real fetch cancellation.
            Route pending = held.getAndSet(null);
            if (pending != null) pending.resume();
            page.waitForCondition(aborted::get);
            assertEquals(0, page.locator("iframe.epub-frame,.epub-error").count());
        } finally {
            page.unroute("**/reader3/file/download**", delay);
            page.offRequestFailed(failed);
            Route leftover = held.getAndSet(null);
            if (leftover != null) leftover.resume();
        }
    }

    private static void verifyDownloadDeadlineAndRetry(Page page, String base, String bookUrl) {
        openGeneratedCssInTextMode(page, base, bookUrl);
        AtomicReference<Route> held = new AtomicReference<>();
        AtomicBoolean aborted = new AtomicBoolean();
        Consumer<Route> delay = route -> { if (!held.compareAndSet(null, route)) route.resume(); };
        Consumer<Request> failed = request -> {
            if (URI.create(request.url()).getPath().endsWith("/reader3/file/download")
                    && String.valueOf(request.failure()).contains("ERR_ABORTED")) aborted.set(true);
        };
        // Exercise the real browser timer without sleeping for a minute or adding a production test bypass.
        page.clock().install();
        page.onRequestFailed(failed);
        page.route("**/reader3/file/download**", delay);
        try {
            selectEpubMode(page, "净化排版");
            page.locator(".reader-content.epub-html").waitFor();
            page.waitForRequest(request -> URI.create(request.url()).getPath().endsWith("/reader3/file/download"),
                    () -> page.locator("button[title^='EPUB 排版模式']").click());
            page.waitForCondition(() -> held.get() != null);
            page.clock().fastForward(60001);
            page.locator(".epub-error").getByText("EPUB 下载超时（60 秒），已终止原版排版加载",
                    new com.microsoft.playwright.Locator.GetByTextOptions().setExact(true)).waitFor();
            assertEquals(0, page.locator("iframe.epub-frame").count());
            Route pending = held.getAndSet(null);
            if (pending != null) pending.resume();
            page.waitForCondition(aborted::get);
            page.screenshot(new Page.ScreenshotOptions().setPath(Path.of(System.getenv()
                    .getOrDefault("RUNNER_TEMP", System.getProperty("java.io.tmpdir")),
                    "vue3-local-reading-download-deadline-synthetic.png")));
            page.locator(".epub-error").getByRole(com.microsoft.playwright.options.AriaRole.BUTTON,
                    new com.microsoft.playwright.Locator.GetByRoleOptions().setName("切换普通阅读").setExact(true)).click();
            page.locator(".reader-content:not(.epub-html)").getByText("生成预算样本正文仍可普通阅读",
                    new com.microsoft.playwright.Locator.GetByTextOptions().setExact(true)).waitFor();
            assertEquals(0, page.locator(".epub-error").count());
        } finally {
            page.unroute("**/reader3/file/download**", delay);
            page.offRequestFailed(failed);
            Route leftover = held.getAndSet(null);
            if (leftover != null) leftover.resume();
        }
        // Same source can really retry after the aborted response: no poisoned cache or late iframe.
        selectEpubMode(page, "原版排版");
        page.frameLocator("iframe.epub-frame").locator("#css-proof").waitFor();
        assertEquals(0, page.locator(".epub-error").count());
        System.out.println("Generated EPUB: stalled download aborted at the 60-second browser-clock deadline, ordinary reading and real retry PASS");
    }

    private static void openGeneratedCssInTextMode(Page page, String base, String bookUrl) {
        String reader = base + "/reader/" + URLEncoder.encode(bookUrl, StandardCharsets.UTF_8)
            .replace("+", "%20");
        Response toc = page.waitForResponse(response -> URI.create(response.url()).getPath()
            .endsWith("/reader3/getChapterList"), () -> page.navigate(reader + "?chapter=0"));
        assertEquals(200, toc.status());
        assertTrue(toc.text().contains("\"isSuccess\":true"));
        selectEpubMode(page, "纯文本");
        page.locator(".reader-content:not(.epub-html)").getByText("生成预算样本正文仍可普通阅读",
            new com.microsoft.playwright.Locator.GetByTextOptions().setExact(true)).waitFor();
    }

    private static void selectEpubMode(Page page, String target) {
        com.microsoft.playwright.Locator button = page.locator("button[title^='EPUB 排版模式']");
        button.waitFor();
        for (int attempt = 0; attempt < 3; attempt++) {
            String current = button.innerText().trim();
            if (target.equals(current)) return;
            button.click();
            page.waitForCondition(() -> !current.equals(button.innerText().trim()));
        }
        assertEquals("Select the actual mode through the user-visible control", target, button.innerText().trim());
    }

    private static void writeCssEpub(Path path, boolean overBudget) throws Exception {
        try (ZipOutputStream zip = new ZipOutputStream(Files.newOutputStream(path))) {
            entry(zip, "mimetype", "application/epub+zip");
            entry(zip, "META-INF/container.xml", "<container><rootfiles><rootfile full-path='OEBPS/book.opf'/></rootfiles></container>");
            entry(zip, "OEBPS/book.opf", "<package xmlns='http://www.idpf.org/2007/opf' version='2.0' unique-identifier='uid'>"
                + "<metadata xmlns:dc='http://purl.org/dc/elements/1.1/'><dc:identifier id='uid'>synthetic-css-" + overBudget + "</dc:identifier>"
                + "<dc:title>" + (overBudget ? "生成原版预算 EPUB" : "生成嵌套 CSS EPUB")
                + "</dc:title><dc:creator>测试作者</dc:creator></metadata>"
                + "<manifest><item id='ncx' href='toc.ncx' media-type='application/x-dtbncx+xml'/>"
                + "<item id='one' href='Text/one.xhtml' media-type='application/xhtml+xml'/>"
                + "<item id='css' href='Styles/main.css' media-type='text/css'/></manifest>"
                + "<spine toc='ncx'><itemref idref='one'/></spine></package>");
            entry(zip, "OEBPS/toc.ncx", "<ncx xmlns='http://www.daisy.org/z3986/2005/ncx/' version='2005-1'>"
                + "<head><meta name='dtb:uid' content='synthetic-css-" + overBudget + "'/></head><docTitle><text>生成样式</text></docTitle>"
                + "<navMap><navPoint id='one' playOrder='1'><navLabel><text>生成章节</text></navLabel>"
                + "<content src='Text/one.xhtml'/></navPoint></navMap></ncx>");
            entry(zip, "OEBPS/Text/one.xhtml", "<html xmlns='http://www.w3.org/1999/xhtml'><head><title>生成样式</title>"
                + "<link rel='stylesheet' href='../Styles/main.css' media='screen'/>"
                + "<style>#inline-proof{background-image:url('../Images/图.svg')}</style></head><body>"
                + "<p id='css-proof' class='root leaf cycle-a cycle-b'>生成嵌套样式中文正文</p>"
                + "<p id='inline-proof'>生成行内样式正文</p><img id='image-proof' src='../Images/图.svg' alt='生成图片'/>"
                + "<p>生成预算样本正文仍可普通阅读</p><script>/* SCRIPT-ONLY */</script></body></html>");
            entry(zip, "OEBPS/Styles/main.css", overBudget ? "/*" + "x".repeat(4 * 1024 * 1024) + "*/"
                : "@import 'nested/palette.css' layer(book) supports(display: grid) screen;"
                + "@import url('cycle.css');@import 'http://127.0.0.1:18899/outside.css';"
                + ".root{font-weight:700}.cycle-a{margin-left:7px}");
            if (!overBudget) {
                entry(zip, "OEBPS/Styles/nested/palette.css", "@import '../leaf/%E4%B8%AD%E6%96%87.css' screen;");
                entry(zip, "OEBPS/Styles/leaf/中文.css", ".leaf{color:rgb(17,34,51);background-image:url('../../Images/图.svg')}");
                entry(zip, "OEBPS/Styles/cycle.css", "@import 'main.css';.cycle-b{padding-left:11px}");
            }
            entry(zip, "OEBPS/Images/图.svg", "<svg xmlns='http://www.w3.org/2000/svg' width='12' height='12'><rect width='12' height='12' fill='#193'/></svg>");
        }
    }

    /** Opt-in export of these generated fixtures only; never reads a private book. */
    private static void exportGeneratedFixture(Path fixture, String name) throws Exception {
        String directory = System.getenv("READER_GENERATED_EPUB_EXPORT_DIR");
        if (directory == null || directory.isBlank()) return;
        Path destination = Path.of(directory);
        Files.createDirectories(destination);
        Files.copy(fixture, destination.resolve(name)); // Fail instead of overwriting existing evidence.
    }

    /** Hold a real old-chapter POST: a new progress-writing GET must not overtake it. */
    private static Response readAfterOldProgressSettles(Page page, String bookUrl, Runnable navigate) {
        AtomicReference<Route> held = new AtomicReference<>();
        AtomicBoolean prematureRead = new AtomicBoolean();
        Consumer<Request> observer = request -> {
            URI uri = URI.create(request.url());
            if (uri.getPath().endsWith("/reader3/getBookContent") && uri.getQuery().contains("index=1")
                    && held.get() != null) prematureRead.set(true);
        };
        Consumer<Route> delayOldWrite = route -> {
            String body = route.request().postData();
            if (body != null && body.contains(bookUrl) && body.contains("\"index\":0")
                    && held.compareAndSet(null, route)) return;
            route.resume();
        };
        page.onRequest(observer);
        page.route("**/reader3/saveBookProgress**", delayOldWrite);
        try {
            page.waitForRequest(request -> URI.create(request.url()).getPath().endsWith("/reader3/saveBookProgress")
                    && request.postData() != null && request.postData().contains(bookUrl)
                    && request.postData().contains("\"index\":0"), navigate);
            page.waitForCondition(() -> held.get() != null);
            try {
                page.waitForCondition(prematureRead::get, new Page.WaitForConditionOptions().setTimeout(1000));
            } catch (TimeoutError expected) { /* The old POST is deliberately held for this bounded interval. */ }
            assertFalse("New chapter read overtook an unresolved old progress POST", prematureRead.get());
            return page.waitForResponse(response -> URI.create(response.url()).getPath().endsWith("/reader3/getBookContent")
                    && URI.create(response.url()).getQuery().contains("index=1"), () -> held.getAndSet(null).resume());
        } finally {
            Route leftover = held.getAndSet(null);
            if (leftover != null) leftover.resume();
            page.unroute("**/reader3/saveBookProgress**", delayOldWrite);
            page.offRequest(observer);
        }
    }

    private static double innerScroll(Page page) {
        return ((Number) page.frameLocator("iframe.epub-frame").locator("body")
                .evaluate("el => el.ownerDocument.defaultView.scrollY")).doubleValue();
    }

    private static void assertAnchorAtTop(Page page, String selector) {
        page.waitForCondition(() -> Math.abs(((Number) page.frameLocator("iframe.epub-frame").locator(selector)
                .evaluate("el => el.getBoundingClientRect().top")).doubleValue()) <= 2);
        double top = ((Number) page.frameLocator("iframe.epub-frame").locator(selector)
                .evaluate("el => el.getBoundingClientRect().top")).doubleValue();
        assertEquals("Exact EPUB anchor must be positioned at the iframe top: " + selector, 0, top, 2);
    }

    private static void writeFragmentEpub(Path path) throws Exception {
        try (ZipOutputStream zip = new ZipOutputStream(Files.newOutputStream(path))) {
            entry(zip, "mimetype", "application/epub+zip");
            entry(zip, "META-INF/container.xml", "<container><rootfiles><rootfile full-path='OEBPS/book.opf'/></rootfiles></container>");
            entry(zip, "OEBPS/book.opf", "<package xmlns='http://www.idpf.org/2007/opf' version='2.0' unique-identifier='uid'>"
                    + "<metadata xmlns:dc='http://purl.org/dc/elements/1.1/'><dc:identifier id='uid'>synthetic-fragments</dc:identifier>"
                    + "<dc:title>合成目录锚点 EPUB</dc:title><dc:creator>测试作者</dc:creator></metadata>"
                    + "<manifest><item id='ncx' href='toc.ncx' media-type='application/x-dtbncx+xml'/>"
                    + "<item id='shared' href='Text/shared.xhtml' media-type='application/xhtml+xml'/></manifest>"
                    + "<spine toc='ncx'><itemref idref='shared'/></spine></package>");
            entry(zip, "OEBPS/toc.ncx", "<ncx xmlns='http://www.daisy.org/z3986/2005/ncx/' version='2005-1'>"
                    + "<head><meta name='dtb:uid' content='synthetic-fragments'/></head><docTitle><text>合成目录锚点 EPUB</text></docTitle><navMap>"
                    + "<navPoint id='intro' playOrder='1'><navLabel><text>首节锚点</text></navLabel><content src='Text/shared.xhtml#intro'/></navPoint>"
                    + "<navPoint id='second' playOrder='2'><navLabel><text>第二节锚点</text></navLabel><content src='Text/shared.xhtml#中文目标'/></navPoint>"
                    + "</navMap></ncx>");
            entry(zip, "OEBPS/Text/shared.xhtml", "<html xmlns='http://www.w3.org/1999/xhtml'><head><title>合成锚点测试</title>"
                    + "<style>html{scroll-behavior:smooth}</style></head><body>"
                    + "<h2 id='intro'>首节锚点</h2><p>合成锚点首节正文</p>"
                    + "<a href='#%E4%B8%AD%E6%96%87%E7%9B%AE%E6%A0%87'>同页下一节</a>"
                    + "<a href='#note'>同页未列目录注记</a>" + longSyntheticBody()
                    + "<h2 id='中文目标'>第二节锚点</h2><p>合成锚点第二节正文</p>"
                    + "<a href='shared.xhtml#intro'>同文件返回首节</a>"
                    + "<a href='#missing'>不存在锚点的合成链接</a>" + longSyntheticBody()
                    + "<a name='note'>合成旧式命名锚点</a>" + longSyntheticBody()
                    + "<script>/* SCRIPT-ONLY */</script></body></html>");
        }
    }

    private static void writeNavFragmentEpub(Path path) throws Exception {
        try (ZipOutputStream zip = new ZipOutputStream(Files.newOutputStream(path))) {
            entry(zip, "mimetype", "application/epub+zip");
            entry(zip, "META-INF/container.xml", "<container><rootfiles><rootfile full-path='OEBPS/book.opf'/></rootfiles></container>");
            entry(zip, "OEBPS/book.opf", "<package xmlns='http://www.idpf.org/2007/opf' version='3.0' unique-identifier='uid'>"
                    + "<metadata xmlns:dc='http://purl.org/dc/elements/1.1/'><dc:identifier id='uid'>synthetic-nav-fragments</dc:identifier>"
                    + "<dc:title>合成 EPUB3 导航锚点</dc:title><dc:creator>测试作者</dc:creator></metadata>"
                    + "<manifest><item id='nav' href='Nav/toc.xhtml' media-type='application/xhtml+xml' properties='nav'/>"
                    + "<item id='ncx' href='toc.ncx' media-type='application/x-dtbncx+xml'/>"
                    + "<item id='shared' href='Text/shared.xhtml' media-type='application/xhtml+xml'/>"
                    + "<item id='later' href='Text/later.xhtml' media-type='application/xhtml+xml'/></manifest>"
                    + "<spine toc='ncx'><itemref idref='shared'/><itemref idref='later'/></spine></package>");
            // 'ops' is deliberately an alias for the EPUB namespace, not the literal prefix 'epub'.
            entry(zip, "OEBPS/Nav/toc.xhtml", "<html xmlns='http://www.w3.org/1999/xhtml' xmlns:ops='http://www.idpf.org/2007/ops'>"
                    + "<head><title>生成导航</title></head><body><nav ops:type='toc'><ol>"
                    + "<li><a href='../Text/shared.xhtml#intro'>NAV 首节</a><ol>"
                    + "<li><a href='../Text/shared.xhtml#%E4%B8%AD%E6%96%87%E7%9B%AE%E6%A0%87'>NAV 中文小节</a></li>"
                    + "</ol></li><li><a href='../Text/later.xhtml#finale'>NAV 后一文件</a></li>"
                    + "<li><a href='../Text/shared.xhtml#intro'>重复目录</a></li></ol></nav>"
                    + "<nav ops:type='landmarks'><ol><li><a href='../Text/shared.xhtml#note'>地标不是目录</a></li></ol></nav>"
                    + "</body></html>");
            entry(zip, "OEBPS/toc.ncx", "<ncx xmlns='http://www.daisy.org/z3986/2005/ncx/' version='2005-1'>"
                    + "<head><meta name='dtb:uid' content='synthetic-nav-fragments'/></head><docTitle><text>生成 EPUB3 导航</text></docTitle>"
                    + "<navMap><navPoint id='decoy' playOrder='1'><navLabel><text>旧 NCX 不应显示</text></navLabel>"
                    + "<content src='Text/shared.xhtml#intro'/></navPoint></navMap></ncx>");
            entry(zip, "OEBPS/Text/shared.xhtml", "<html xmlns='http://www.w3.org/1999/xhtml'><head><title>EPUB3 生成正文</title></head><body>"
                    + "<h2 id='intro'>NAV 首节</h2><p>EPUB3 合成首节正文</p>" + longSyntheticBody()
                    + "<h2 id='中文目标'>NAV 中文小节</h2><p>EPUB3 合成中文小节正文</p>"
                    + "<a href='later.xhtml#finale'>跨文件去后一节</a>" + longSyntheticBody()
                    + "<a name='note'>生成注记</a><script>/* SCRIPT-ONLY */</script></body></html>");
            entry(zip, "OEBPS/Text/later.xhtml", "<html xmlns='http://www.w3.org/1999/xhtml'><head><title>EPUB3 后一正文文件</title></head><body>"
                    + longSyntheticBody() + "<h2 id='finale'>NAV 后一文件</h2><p>EPUB3 合成后一文件正文</p>"
                    + "<a href='shared.xhtml#%E4%B8%AD%E6%96%87%E7%9B%AE%E6%A0%87'>跨文件返回中文小节</a>"
                    + longSyntheticBody() + "</body></html>");
        }
    }

    private static void writeEpub(Path path) throws Exception {
        try (ZipOutputStream zip = new ZipOutputStream(Files.newOutputStream(path))) {
            entry(zip, "mimetype", "application/epub+zip");
            entry(zip, "META-INF/container.xml", "<?xml version='1.0' encoding='UTF-8'?>"
                    + "<container xmlns='urn:oasis:names:tc:opendocument:xmlns:container' version='1.0'>"
                    + "<rootfiles><rootfile full-path='OEBPS/content.opf' "
                    + "media-type='application/oebps-package+xml'/></rootfiles></container>");
            entry(zip, "OEBPS/content.opf", "<?xml version='1.0' encoding='UTF-8'?>"
                    + "<package xmlns='http://www.idpf.org/2007/opf' version='2.0' unique-identifier='uid'>"
                    + "<metadata xmlns:dc='http://purl.org/dc/elements/1.1/'><dc:identifier id='uid'>synthetic</dc:identifier>"
                    + "<dc:title>合成本地 EPUB</dc:title><dc:creator>测试作者</dc:creator><dc:language>zh-CN</dc:language></metadata>"
                    + "<manifest><item id='ncx' href='toc.ncx' media-type='application/x-dtbncx+xml'/>"
                    + "<item id='one' href='Text/one.xhtml' media-type='application/xhtml+xml'/>"
                    + "<item id='two' href='Text/two.xhtml' media-type='application/xhtml+xml'/>"
                    + "<item id='css' href='Styles/main.css' media-type='text/css'/></manifest>"
                    // Deliberately different from NCX order: TOC index is not a spine index.
                    + "<spine toc='ncx'><itemref idref='two'/><itemref idref='one'/></spine></package>");
            entry(zip, "OEBPS/toc.ncx", "<?xml version='1.0' encoding='UTF-8'?>"
                    + "<ncx xmlns='http://www.daisy.org/z3986/2005/ncx/' version='2005-1'>"
                    + "<head><meta name='dtb:uid' content='synthetic'/></head><docTitle><text>合成本地 EPUB</text></docTitle><navMap>"
                    + "<navPoint id='one' playOrder='1'><navLabel><text>第一章</text></navLabel><content src='Text/one.xhtml'/></navPoint>"
                    + "<navPoint id='two' playOrder='2'><navLabel><text>第二章</text></navLabel><content src='Text/two.xhtml'/></navPoint>"
                    + "</navMap></ncx>");
            entry(zip, "OEBPS/Text/one.xhtml", "<html xmlns='http://www.w3.org/1999/xhtml'>"
                    + "<head><title>HEAD-ONLY</title><link rel='stylesheet' href='../Styles/main.css'/></head><body><h1>第一章</h1>"
                    + "<p>合成 EPUB 第一段，中文 &amp; 标点 &lt;明&gt;。</p>"
                    + "<a href='two.xhtml'>书内下一章</a>"
                    + longSyntheticBody() + "<script>/* SCRIPT-ONLY */</script></body></html>");
            entry(zip, "OEBPS/Text/two.xhtml", "<html xmlns='http://www.w3.org/1999/xhtml'>"
                    + "<head><title>HEAD-ONLY</title></head><body><h1>第二章</h1><p>" + EPUB_LAST + "</p>"
                    + longSyntheticBody() + "</body></html>");
            entry(zip, "OEBPS/Styles/main.css", "p{color:rgb(17,34,51)}");
        }
    }

    private static void entry(ZipOutputStream zip, String name, String content) throws Exception {
        ZipEntry entry = new ZipEntry(name);
        entry.setTime(1577836800000L);
        // EPUB's first mimetype entry must be uncompressed.
        byte[] bytes = content.getBytes(StandardCharsets.UTF_8);
        if (name.equals("mimetype")) {
            java.util.zip.CRC32 crc = new java.util.zip.CRC32();
            crc.update(bytes);
            entry.setMethod(ZipEntry.STORED);
            entry.setSize(bytes.length);
            entry.setCrc(crc.getValue());
        }
        zip.putNextEntry(entry);
        zip.write(bytes);
        zip.closeEntry();
    }
}
