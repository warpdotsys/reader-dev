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
import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.UUID;
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
        Files.writeString(txt, "第一章 起点\n" + TXT_TEXT + "\n第二章 终点\n" + TXT_LAST,
                StandardCharsets.UTF_8);
        writeEpub(epub);
        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
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
            } finally {
                browser.close();
            }
        } finally {
            Files.deleteIfExists(txt);
            Files.deleteIfExists(epub);
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
        String url = (String) page.evaluate("raw => JSON.parse(raw).data.bookUrl", saved.text());
        assertTrue(url.startsWith("storage/data/"));
        page.locator(".bookshelf-page").waitFor();
        return url;
    }

    private static void verifyReading(Page page, String base, String bookUrl,
                                      String first, String last, boolean epub) {
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
                    + "<item id='two' href='Text/two.xhtml' media-type='application/xhtml+xml'/></manifest>"
                    + "<spine toc='ncx'><itemref idref='one'/><itemref idref='two'/></spine></package>");
            entry(zip, "OEBPS/toc.ncx", "<?xml version='1.0' encoding='UTF-8'?>"
                    + "<ncx xmlns='http://www.daisy.org/z3986/2005/ncx/' version='2005-1'>"
                    + "<head><meta name='dtb:uid' content='synthetic'/></head><docTitle><text>合成本地 EPUB</text></docTitle><navMap>"
                    + "<navPoint id='one' playOrder='1'><navLabel><text>第一章</text></navLabel><content src='Text/one.xhtml'/></navPoint>"
                    + "<navPoint id='two' playOrder='2'><navLabel><text>第二章</text></navLabel><content src='Text/two.xhtml'/></navPoint>"
                    + "</navMap></ncx>");
            entry(zip, "OEBPS/Text/one.xhtml", "<html xmlns='http://www.w3.org/1999/xhtml'>"
                    + "<head><title>HEAD-ONLY</title></head><body><h1>第一章</h1>"
                    + "<p>合成 EPUB 第一段，中文 &amp; 标点 &lt;明&gt;。</p>"
                    + "<script>/* SCRIPT-ONLY */</script></body></html>");
            entry(zip, "OEBPS/Text/two.xhtml", "<html xmlns='http://www.w3.org/1999/xhtml'>"
                    + "<head><title>HEAD-ONLY</title></head><body><h1>第二章</h1><p>" + EPUB_LAST + "</p></body></html>");
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
