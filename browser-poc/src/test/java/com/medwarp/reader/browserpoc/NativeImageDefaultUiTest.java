package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.*;
import org.junit.Assume;
import org.junit.Test;

import java.io.InputStream;
import java.net.URI;
import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.LinkedHashSet;
import java.util.Set;
import java.util.zip.CRC32;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;
import java.util.zip.ZipFile;

import static org.junit.Assert.*;

/** Real client against the complete image's default UI, never Vite or production. */
public class NativeImageDefaultUiTest {
    private static final String FIRST = "完整镜像生成正文第一段，中文与 UTF-8。";
    private static final String LAST = "完整镜像生成正文第二段，翻章和刷新。";
    private static final String EPUB_NAME = "完整镜像生成 EPUB";
    private static final String EPUB_FIRST = "完整镜像生成 EPUB 第一章，三种模式与中文。";
    private static final String EPUB_LAST = "完整镜像生成 EPUB 第二章，返回与刷新。";

    @Test(timeout = 180000)
    public void defaultImageRegisterLoginImportReadReloadAndLogout() throws Exception {
        // Generic unit builds may omit this integration fixture. The image job
        // explicitly selects it and separately rejects any skip or missing XML.
        Assume.assumeTrue("1".equals(System.getenv("READER_NATIVE_UI_ISOLATED")));
        String base = System.getenv("READER_NATIVE_UI_URL");
        assertEquals("Only the fresh complete-image fixture is permitted", "http://127.0.0.1:18890", base);
        String revision = System.getenv("READER_NATIVE_UI_REVISION");
        assertNotNull(revision);
        assertTrue(revision.matches("[0-9a-f]{40}"));
        String version = "ci-" + revision.substring(0, 12);
        String executable = System.getProperty("browser.executable", "");
        assertFalse(executable.isEmpty());
        assertTrue(Files.isRegularFile(Path.of(executable)));
        String evidence = System.getenv("READER_NATIVE_UI_EVIDENCE_DIR");
        assertNotNull(evidence);
        Path directory = Path.of(evidence);
        Files.createDirectories(directory);
        String username = "imageui" + UUID.randomUUID().toString().replace("-", "").substring(0, 10);
        String password = "GeneratedImageProbe-2026";
        Path fixture = Files.createTempFile("reader-native-generated-", ".txt");
        Files.writeString(fixture, "第一章 起点\n" + FIRST + "\n第二章 终点\n" + LAST, StandardCharsets.UTF_8);
        Path epubFixture = Files.createTempFile("reader-native-generated-", ".epub");
        List<String> unexpectedNetwork = new ArrayList<>();
        List<String> pageErrors = new ArrayList<>();
        List<String> requestedPaths = new ArrayList<>();
        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")));
             Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                     .setExecutablePath(Path.of(executable)).setHeadless(true));
             BrowserContext context = browser.newContext(new Browser.NewContextOptions()
                     .setViewportSize(1280, 900))) {
            writeGeneratedEpub(epubFixture);
            validateGeneratedEpub(epubFixture);
            // No fake responses. Only the actual isolated Reader origin is
            // allowed; this generated-book journey needs no external resources.
            context.route("**/*", route -> {
                URI target = URI.create(route.request().url());
                if ("http".equals(target.getScheme()) && "127.0.0.1".equals(target.getHost())
                        && target.getPort() == 18890) route.resume();
                else { unexpectedNetwork.add(target.getScheme() + ":" + target.getHost()); route.abort(); }
            });
            Page page = context.newPage();
            page.setDefaultTimeout(30000);
            page.onPageError(pageErrors::add);
            page.onRequest(request -> requestedPaths.add(URI.create(request.url()).getPath()));
            page.navigate(base + "/login");
            page.locator(".login-page").waitFor();
            APIResponse release = context.request().get(base + "/assets/reader-release.json");
            assertEquals(200, release.status());
            assertEquals(Boolean.TRUE, page.evaluate("args => { const r=JSON.parse(args[0]);"
                    + "return r.version===args[1] && r.buildRevision===args[2]; }",
                    List.of(release.text(), version, revision)));
            capture(page, directory, "login");

            page.locator(".mode-switch button").nth(1).click();
            page.locator("input[autocomplete=username]").fill(username);
            page.locator("input[autocomplete=current-password]").fill(password);
            page.locator(".submit-btn").click();
            page.locator(".bookshelf-page").waitFor();
            assertEquals(username, page.evaluate("localStorage.getItem('reader_username')"));
            assertNamespacedToken(page, "localStorage", username);
            page.locator(".logout-btn").click();
            page.locator(".login-page").waitFor();
            assertNoTokens(page);

            page.locator("input[autocomplete=username]").fill(username);
            page.locator("input[autocomplete=current-password]").fill("WrongGeneratedImageProbe-2026");
            Response refused = page.waitForResponse(response -> endpoint(response, "login"),
                    () -> page.locator(".submit-btn").click());
            assertEnvelope(page, refused, false);
            assertEquals(Boolean.TRUE, page.evaluate("raw => JSON.parse(raw).errorMsg.includes('密码错误')", refused.text()));
            page.locator(".el-message--error").filter(new Locator.FilterOptions().setHasText("密码错误")).waitFor();
            assertTrue(page.locator(".login-page").isVisible());
            assertNoTokens(page);
            // Dismiss the already asserted error using its real visible control,
            // so later screenshots cannot misattribute a previous login toast.
            for (Locator close : page.locator(".el-message__closeBtn").all()) close.click();
            page.waitForFunction("() => document.querySelectorAll('.el-message').length === 0");
            page.locator("input[autocomplete=current-password]").fill(password);
            page.locator("input[type=checkbox]").uncheck();
            Response loggedIn = page.waitForResponse(response -> endpoint(response, "login"),
                    () -> page.locator(".submit-btn").click());
            assertEnvelope(page, loggedIn, true);
            page.locator(".bookshelf-page").waitFor();
            assertEquals(null, page.evaluate("localStorage.getItem('reader_username')"));
            assertEquals(null, page.evaluate("localStorage.getItem('reader_access_token')"));
            assertEquals("0", page.evaluate("localStorage.getItem('reader_remember')"));
            String token = assertNamespacedToken(page, "sessionStorage", username);
            Response emptyShelf = page.waitForResponse(response -> endpoint(response, "getBookshelf"), page::reload);
            assertEquals(200, emptyShelf.status());
            page.locator(".bookshelf-page").waitFor();
            assertTrue("Tab-local token must survive reload", token.equals(page.evaluate("sessionStorage.getItem('reader_access_token')")));
            assertEquals(0, shelfCount(page));

            page.locator(".import-btn").click();
            page.locator("[aria-label='导入本地书籍']").waitFor();
            FileChooser chooser = page.waitForFileChooser(() -> page.locator(".dropzone").click());
            Response parsed = page.waitForResponse(response -> endpoint(response, "importBookPreview"),
                    () -> chooser.setFiles(fixture));
            assertEnvelope(page, parsed, true);
            page.locator(".preview-panel").waitFor();
            assertTrue(page.locator(".preview-panel").innerText().contains(fixture.getFileName().toString().replace(".txt", "")));
            assertEquals(0, shelfCount(page));
            Response saved = page.waitForResponse(response -> endpoint(response, "saveBook"),
                    () -> page.locator("[aria-label='导入本地书籍'] .accent-btn").click());
            assertEnvelope(page, saved, true);
            String bookUrl = (String) page.evaluate("raw => JSON.parse(raw).data.bookUrl", saved.text());
            assertTrue(bookUrl.startsWith("storage/data/"));
            // A save response precedes Vue's awaited shelf refresh. Do not
            // navigate away while that real user-visible workflow is pending.
            page.locator("[aria-label='导入本地书籍']")
                    .waitFor(new Locator.WaitForOptions().setState(com.microsoft.playwright.options.WaitForSelectorState.HIDDEN));
            String savedName = (String) page.evaluate("raw => JSON.parse(raw).data.name", saved.text());
            page.locator(".book-card").filter(new Locator.FilterOptions().setHasText(savedName)).waitFor();
            page.locator(".bookshelf-page").waitFor();
            assertEquals(1, shelfCount(page));
            capture(page, directory, "shelf");
            String reader = base + "/reader/" + URLEncoder.encode(bookUrl, StandardCharsets.UTF_8).replace("+", "%20");
            Response content = page.waitForResponse(response -> endpoint(response, "getBookContent"),
                    () -> page.navigate(reader + "?chapter=0"));
            assertEnvelope(page, content, true);
            page.getByText(FIRST, new Page.GetByTextOptions().setExact(true)).waitFor();
            assertFalse(page.locator(".reader-content").innerText().contains("\ufffd"));
            assertFalse(page.locator(".chapter-nav button").first().isEnabled());
            Response progress = page.waitForResponse(response -> endpoint(response, "saveBookProgress"),
                    () -> page.locator(".chapter-nav button").last().click());
            assertEnvelope(page, progress, true);
            page.getByText(LAST, new Page.GetByTextOptions().setExact(true)).waitFor();
            page.waitForCondition(() -> page.url().contains("chapter=1"));
            page.reload();
            page.getByText(LAST, new Page.GetByTextOptions().setExact(true)).waitFor();
            assertFalse(page.locator(".chapter-nav button").last().isEnabled());
            assertFalse(page.locator(".reader-content").innerText().contains("\ufffd"));
            capture(page, directory, "reading");
            page.locator(".chapter-nav button").first().click();
            page.getByText(FIRST, new Page.GetByTextOptions().setExact(true)).waitFor();
            page.waitForCondition(() -> page.url().contains("chapter=0"));
            page.reload();
            page.getByText(FIRST, new Page.GetByTextOptions().setExact(true)).waitFor();
            page.navigate(base + "/");
            page.locator(".bookshelf-page").waitFor();
            importGeneratedEpubAndReadModes(page, epubFixture, base);
            page.navigate(base + "/");
            page.locator(".bookshelf-page").waitFor();
            assertEquals(2, shelfCount(page));
            page.locator(".logout-btn").click();
            page.locator(".login-page").waitFor();
            assertNoTokens(page);
            page.navigate(base + "/");
            page.locator(".login-page").waitFor();
            assertEquals(0, page.locator(".bookshelf-page").count());
            capture(page, directory, "logout");
            assertFalse(requestedPaths.stream().anyMatch(path -> path.endsWith("/reader3/uploadLocalBook")));
            assertTrue("No external network may be attempted", unexpectedNetwork.isEmpty());
            assertTrue("Generated image journey must have no page errors", pageErrors.isEmpty());
        } finally {
            Files.deleteIfExists(fixture); // Only this exclusive generated fixture, never an input/user book.
            Files.deleteIfExists(epubFixture);
        }
    }

    private static void importGeneratedEpubAndReadModes(Page page, Path fixture, String base) {
        page.locator(".import-btn").click();
        Locator dialog = page.locator("[aria-label='导入本地书籍']");
        dialog.waitFor();
        FileChooser chooser = page.waitForFileChooser(() -> page.locator(".dropzone").click());
        Response preview = page.waitForResponse(response -> endpoint(response, "importBookPreview"),
                () -> chooser.setFiles(fixture));
        assertEnvelope(page, preview, true);
        page.locator(".preview-panel").waitFor();
        assertTrue(page.locator(".preview-panel").innerText().contains(EPUB_NAME));
        assertEquals(1, shelfCount(page));
        Response saved = page.waitForResponse(response -> endpoint(response, "saveBook"),
                () -> dialog.locator(".accent-btn").click());
        assertEnvelope(page, saved, true);
        String bookUrl = (String) page.evaluate("raw => JSON.parse(raw).data.bookUrl", saved.text());
        assertTrue(bookUrl.startsWith("storage/data/"));
        dialog.waitFor(new Locator.WaitForOptions().setState(com.microsoft.playwright.options.WaitForSelectorState.HIDDEN));
        page.locator(".book-card").filter(new Locator.FilterOptions().setHasText(EPUB_NAME)).waitFor();
        assertEquals(2, shelfCount(page));
        String reader = base + "/reader/" + URLEncoder.encode(bookUrl, StandardCharsets.UTF_8).replace("+", "%20");
        page.navigate(reader + "?chapter=0");
        page.getByText(EPUB_FIRST, new Page.GetByTextOptions().setExact(true)).waitFor();
        assertFalse(page.locator(".reader-content").innerText().contains("\ufffd"));
        assertEquals(0, page.locator(".reader-content script").count());
        assertTrue(page.locator("button[title^='EPUB 排版模式']").innerText().contains("文本"));

        page.locator("button[title^='EPUB 排版模式']").click();
        page.locator(".reader-content.epub-html").waitFor();
        page.getByText(EPUB_FIRST, new Page.GetByTextOptions().setExact(true)).waitFor();
        assertFalse(page.locator(".reader-content").innerText().contains("\ufffd"));
        assertEquals(0, page.locator(".reader-content script").count());

        page.locator("button[title^='EPUB 排版模式']").click();
        RawEpubReadiness.waitFor(page);
        assertEquals("allow-same-origin", page.locator("iframe.epub-frame").getAttribute("sandbox"));
        Locator rawBody = page.frameLocator("iframe.epub-frame").locator("body");
        page.frameLocator("iframe.epub-frame").getByText(EPUB_FIRST,
                new FrameLocator.GetByTextOptions().setExact(true)).waitFor();
        assertFalse(rawBody.innerText().contains("\ufffd"));
        assertEquals(0, page.frameLocator("iframe.epub-frame").locator("script").count());
        page.locator(".chapter-nav button").last().click();
        page.frameLocator("iframe.epub-frame").getByText(EPUB_LAST,
                new FrameLocator.GetByTextOptions().setExact(true)).waitFor();
        page.waitForCondition(() -> page.url().contains("chapter=1"));
        RawEpubReadiness.waitFor(page);
        page.reload();
        page.frameLocator("iframe.epub-frame").getByText(EPUB_LAST,
                new FrameLocator.GetByTextOptions().setExact(true)).waitFor();
        RawEpubReadiness.waitFor(page);
        assertFalse(page.frameLocator("iframe.epub-frame").locator("body").innerText().contains("\ufffd"));
    }

    static void writeGeneratedEpub(Path path) throws Exception {
        String container = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>"
                + "<container version=\"1.0\" xmlns=\"urn:oasis:names:tc:opendocument:xmlns:container\">"
                + "<rootfiles><rootfile full-path=\"OEBPS/content.opf\" media-type=\"application/oebps-package+xml\"/>"
                + "</rootfiles></container>";
        String opf = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>"
                + "<package version=\"2.0\" unique-identifier=\"id\" xmlns=\"http://www.idpf.org/2007/opf\">"
                + "<metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\"><dc:identifier id=\"id\">urn:generated-native-ui</dc:identifier>"
                + "<dc:title>" + EPUB_NAME + "</dc:title><dc:creator>Generated Fixture</dc:creator><dc:language>zh-CN</dc:language></metadata>"
                + "<manifest><item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\"/>"
                + "<item id=\"c1\" href=\"chapter1.xhtml\" media-type=\"application/xhtml+xml\"/>"
                + "<item id=\"c2\" href=\"chapter2.xhtml\" media-type=\"application/xhtml+xml\"/></manifest>"
                + "<spine toc=\"ncx\"><itemref idref=\"c1\"/><itemref idref=\"c2\"/></spine></package>";
        String ncx = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>"
                + "<ncx version=\"2005-1\" xmlns=\"http://www.daisy.org/z3986/2005/ncx/\">"
                + "<head><meta name=\"dtb:uid\" content=\"urn:generated-native-ui\"/></head><docTitle><text>" + EPUB_NAME + "</text></docTitle>"
                + "<navMap><navPoint id=\"c1\" playOrder=\"1\"><navLabel><text>第一章 生成 EPUB</text></navLabel><content src=\"chapter1.xhtml\"/></navPoint>"
                + "<navPoint id=\"c2\" playOrder=\"2\"><navLabel><text>第二章 生成 EPUB</text></navLabel><content src=\"chapter2.xhtml\"/></navPoint></navMap></ncx>";
        try (ZipOutputStream zip = new ZipOutputStream(Files.newOutputStream(path), StandardCharsets.UTF_8)) {
            byte[] mimetype = "application/epub+zip".getBytes(StandardCharsets.US_ASCII);
            CRC32 crc = new CRC32();
            crc.update(mimetype);
            ZipEntry first = new ZipEntry("mimetype");
            first.setMethod(ZipEntry.STORED);
            first.setSize(mimetype.length);
            first.setCompressedSize(mimetype.length);
            first.setCrc(crc.getValue());
            first.setTimeLocal(LocalDateTime.of(1980, 1, 1, 0, 0));
            zip.putNextEntry(first);
            zip.write(mimetype);
            zip.closeEntry();
            zipText(zip, "META-INF/container.xml", container);
            zipText(zip, "OEBPS/content.opf", opf);
            zipText(zip, "OEBPS/toc.ncx", ncx);
            zipText(zip, "OEBPS/chapter1.xhtml", generatedXhtml("第一章 生成 EPUB", EPUB_FIRST));
            zipText(zip, "OEBPS/chapter2.xhtml", generatedXhtml("第二章 生成 EPUB", EPUB_LAST));
        }
    }

    static void validateGeneratedEpub(Path path) throws Exception {
        assertTrue(Files.size(path) > 0 && Files.size(path) < 65536);
        try (ZipFile zip = new ZipFile(path.toFile(), StandardCharsets.UTF_8)) {
            assertEquals(6, zip.size());
            ZipEntry first = zip.entries().nextElement();
            assertEquals("mimetype", first.getName());
            assertEquals(ZipEntry.STORED, first.getMethod());
            Set<String> names = new LinkedHashSet<>();
            for (ZipEntry entry : java.util.Collections.list(zip.entries())) {
                assertTrue(names.add(entry.getName()));
                byte[] bytes;
                try (InputStream stream = zip.getInputStream(entry)) {
                    bytes = stream.readAllBytes();
                }
                CRC32 crc = new CRC32();
                crc.update(bytes);
                assertEquals(entry.getSize(), (long) bytes.length);
                assertEquals(entry.getCrc(), crc.getValue());
                String text = new String(bytes, StandardCharsets.UTF_8);
                assertFalse(text.contains("\ufffd"));
                if (entry.getName().equals("mimetype")) assertEquals("application/epub+zip", text);
                if (entry.getName().equals("META-INF/container.xml")) assertTrue(text.contains("OEBPS/content.opf"));
                if (entry.getName().equals("OEBPS/content.opf")) assertTrue(text.contains("<dc:title>" + EPUB_NAME + "</dc:title>"));
                if (entry.getName().equals("OEBPS/chapter1.xhtml")) assertTrue(text.contains(EPUB_FIRST));
                if (entry.getName().equals("OEBPS/chapter2.xhtml")) assertTrue(text.contains(EPUB_LAST));
            }
            assertEquals(Set.of("mimetype", "META-INF/container.xml", "OEBPS/content.opf", "OEBPS/toc.ncx",
                    "OEBPS/chapter1.xhtml", "OEBPS/chapter2.xhtml"), names);
        }
    }

    private static String generatedXhtml(String title, String content) {
        return "<?xml version=\"1.0\" encoding=\"UTF-8\"?>"
                + "<html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title>" + title + "</title></head>"
                + "<body><h1>" + title + "</h1><p>" + content + "</p></body></html>";
    }

    private static void zipText(ZipOutputStream zip, String name, String text) throws Exception {
        ZipEntry entry = new ZipEntry(name);
        entry.setTimeLocal(LocalDateTime.of(1980, 1, 1, 0, 0));
        zip.putNextEntry(entry);
        zip.write(text.getBytes(StandardCharsets.UTF_8));
        zip.closeEntry();
    }

    private static boolean endpoint(Response response, String name) {
        return URI.create(response.url()).getPath().equals("/reader3/" + name);
    }

    private static void assertEnvelope(Page page, Response response, boolean success) {
        assertEquals(200, response.status());
        assertEquals(success, page.evaluate("raw => JSON.parse(raw).isSuccess", response.text()));
    }

    private static String assertNamespacedToken(Page page, String storage, String username) {
        assertEquals(username, page.evaluate(storage + ".getItem('reader_username')"));
        Object value = page.evaluate(storage + ".getItem('reader_access_token')");
        assertTrue("Generated session must have a namespaced token", value instanceof String && ((String) value).startsWith(username + ":"));
        return (String) value;
    }

    private static void assertNoTokens(Page page) {
        assertEquals(null, page.evaluate("localStorage.getItem('reader_access_token')"));
        assertEquals(null, page.evaluate("sessionStorage.getItem('reader_access_token')"));
    }

    private static int shelfCount(Page page) {
        return ((Number) page.evaluate("async () => { const token=sessionStorage.getItem('reader_access_token')"
                + " || localStorage.getItem('reader_access_token'); const response=await fetch('/reader3/getBookshelf?accessToken='"
                + " + encodeURIComponent(token)); const result=await response.json();"
                + "if(response.status!==200 || result.isSuccess!==true || !Array.isArray(result.data)) throw Error('Invalid generated shelf envelope');"
                + "return result.data.length; }" )).intValue();
    }

    private static void capture(Page page, Path directory, String stage) throws Exception {
        try (InputStream input = NativeImageDefaultUiTest.class.getResourceAsStream("/ui-screenshot-readiness.js")) {
            assertNotNull(input);
            page.waitForFunction(new String(input.readAllBytes(), StandardCharsets.UTF_8), false);
        }
        page.screenshot(new Page.ScreenshotOptions().setPath(directory.resolve("native-default-ui-generated-" + stage + ".png")));
    }
}
