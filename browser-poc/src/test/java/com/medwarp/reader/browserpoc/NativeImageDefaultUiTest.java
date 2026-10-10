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
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.UUID;

import static org.junit.Assert.*;

/** Real client against the complete image's default UI, never Vite or production. */
public class NativeImageDefaultUiTest {
    private static final String FIRST = "完整镜像生成正文第一段，中文与 UTF-8。";
    private static final String LAST = "完整镜像生成正文第二段，翻章和刷新。";

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
        List<String> unexpectedNetwork = new ArrayList<>();
        List<String> pageErrors = new ArrayList<>();
        List<String> requestedPaths = new ArrayList<>();
        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")));
             Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                     .setExecutablePath(Path.of(executable)).setHeadless(true));
             BrowserContext context = browser.newContext(new Browser.NewContextOptions()
                     .setViewportSize(1280, 900))) {
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
        }
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
