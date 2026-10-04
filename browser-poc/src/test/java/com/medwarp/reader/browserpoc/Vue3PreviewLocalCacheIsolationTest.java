package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Locator;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import com.microsoft.playwright.Response;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

/** Two generated users in one fresh browser context; never uses real data or credentials. */
public class Vue3PreviewLocalCacheIsolationTest {
    private static final String PASSWORD = "Generated-Cache-Probe-20261005";
    private static final String PRIVATE_MARKER = "生成甲账号的独有缓存正文，不应显示给乙账号。";
    private static final String LEGACY_MARKER = "无法确认归属的生成旧缓存，不能自动迁移。";

    private static void loopback(String url) {
        URI parsed = URI.create(url);
        assertEquals("http", parsed.getScheme());
        assertEquals("127.0.0.1", parsed.getHost());
    }

    private static void register(Page page, String preview, String username) {
        page.navigate(preview + "/login");
        page.locator(".mode-switch button").nth(1).click();
        page.locator("input[autocomplete=username]").fill(username);
        page.locator("input[autocomplete=current-password]").fill(PASSWORD);
        page.locator(".submit-btn").click();
        page.locator(".bookshelf-page").waitFor();
    }

    private static void logout(Page page, String preview) {
        page.navigate(preview + "/");
        page.locator(".bookshelf-page").waitFor();
        page.locator(".logout-btn").click();
        page.locator(".login-page").waitFor();
    }

    private static void login(Page page, String username) {
        page.locator("input[autocomplete=username]").fill(username);
        page.locator("input[autocomplete=current-password]").fill(PASSWORD);
        page.locator(".submit-btn").click();
        page.locator(".bookshelf-page").waitFor();
    }

    @Test
    public void chapterCacheDoesNotCrossAccountsAtTheSameBookAndChapterUrl() throws Exception {
        String preview = System.getenv("READER_VUE3_PREVIEW_URL");
        String fixture = System.getenv("READER_BOOK_FIXTURE_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(preview != null && fixture != null && !executable.isEmpty()
                && Files.isRegularFile(Path.of(executable)) && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        loopback(preview);
        loopback(fixture);
        String suffix = UUID.randomUUID().toString().replace("-", "").substring(0, 10);
        String first = "cachea" + suffix;
        String second = "cacheb" + suffix;
        String setup = new String(getClass().getResourceAsStream("/vue3-reading-setup.js").readAllBytes(), StandardCharsets.UTF_8);
        String bookUrl = fixture + "/book";
        String readerPath = preview + "/reader/" + java.net.URLEncoder.encode(bookUrl, StandardCharsets.UTF_8) + "?chapter=0";

        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                Page page = browser.newPage(); // Fresh context; the two generated users intentionally share it.
                AtomicInteger bodyRequests = new AtomicInteger();
                page.onRequest(request -> {
                    URI uri = URI.create(request.url());
                    // Next-chapter image preloading is legitimate; only the current chapter proves its cache hit.
                    if (uri.getPath().endsWith("/reader3/getBookContent") && uri.getRawQuery() != null
                            && java.util.Arrays.asList(uri.getRawQuery().split("&")).contains("index=0")) bodyRequests.incrementAndGet();
                });
                page.setDefaultTimeout(15000);
                register(page, preview, first);
                assertEquals(bookUrl, page.evaluate(setup, fixture));
                // An unowned old-version cache must be preserved, but never used by either account.
                page.evaluate("async ({bookUrl, marker}) => {"
                        + "const db = await new Promise((yes,no) => {const r=indexedDB.open('reader-local-cache',1);"
                        + "r.onupgradeneeded=()=>{const s=r.result.createObjectStore('chapters',{keyPath:'key'});s.createIndex('bookUrl','bookUrl');};"
                        + "r.onsuccess=()=>yes(r.result);r.onerror=()=>no(r.error);});"
                        + "await new Promise((yes,no)=>{const t=db.transaction('chapters','readwrite');"
                        + "t.objectStore('chapters').put({key:'ch:'+bookUrl+'\\u0000'+bookUrl.replace('/book','/chapter/1'),"
                        + "bookUrl,chapterUrl:bookUrl.replace('/book','/chapter/1'),title:'生成旧缓存',index:0,content:marker,updatedAt:1});"
                        + "t.oncomplete=yes;t.onerror=()=>no(t.error);});db.close();}",
                        Map.of("bookUrl", bookUrl, "marker", LEGACY_MARKER));
                page.navigate(readerPath);
                page.locator(".reader-content").getByText("第一段，中文与 UTF-8。").waitFor();
                page.locator("button[title='编辑本章正文并保存（服务器 + 本机缓存）']").click();
                page.locator(".edit-textarea").fill(PRIVATE_MARKER);
                Response saved = page.waitForResponse(response -> URI.create(response.url()).getPath()
                        .endsWith("/reader3/saveBookContent"), () -> page.locator(".edit-card .pop-btn").click());
                assertEquals(200, saved.status());
                assertTrue("Editing must use the real legacy saveBookContent contract", saved.text().contains("\"isSuccess\":true"));
                assertTrue(saved.request().postData().contains("\"url\":\"" + bookUrl + "\""));
                assertTrue(saved.request().postData().contains("\"index\":0"));
                page.locator(".reader-content").getByText(PRIVATE_MARKER).waitFor();
                page.reload();
                page.locator(".reader-content").getByText(PRIVATE_MARKER).waitFor();

                logout(page, preview);
                register(page, preview, second);
                assertEquals(bookUrl, page.evaluate(setup, fixture));
                Object backend = page.evaluate("async ({bookUrl, fixture}) => {"
                        + "const p = new URLSearchParams({url:bookUrl,bookSourceUrl:fixture,index:'0'});"
                        + "const response = await fetch('/reader3/getBookContent?' + p, {credentials:'same-origin'});"
                        + "const data = await response.json();"
                        + "return {status:response.status, success:data.isSuccess, content:typeof data.data === 'string' ? data.data : data.data?.content || ''};}",
                        Map.of("bookUrl", bookUrl, "fixture", fixture));
                assertTrue("The second user's backend must return its own generated public fixture body", backend instanceof Map);
                Map<?, ?> reply = (Map<?, ?>) backend;
                assertEquals(200, ((Number) reply.get("status")).intValue());
                assertEquals(Boolean.TRUE, reply.get("success"));
                assertTrue(((String) reply.get("content")).contains("第一段，中文与 UTF-8。"));
                assertFalse(((String) reply.get("content")).contains(PRIVATE_MARKER));

                page.navigate(readerPath);
                Locator body = page.locator(".reader-content");
                try {
                    body.getByText("第一段，中文与 UTF-8。").waitFor();
                } catch (RuntimeException failure) {
                    String directory = System.getenv("READER_UI_EVIDENCE_DIR");
                    if (directory != null) page.screenshot(new Page.ScreenshotOptions()
                            .setPath(Path.of(directory, "generated-cross-account-cache-failure.png")));
                    throw new AssertionError("Second user must not reuse first user's IndexedDB body; displayed generated text="
                            + body.innerText(), failure);
                }
                assertFalse(body.innerText().contains(PRIVATE_MARKER));
                String directory = System.getenv("READER_UI_EVIDENCE_DIR");
                if (directory != null) page.screenshot(new Page.ScreenshotOptions()
                        .setPath(Path.of(directory, "generated-second-user-cache-isolated.png")));
                page.reload();
                body.getByText("第一段，中文与 UTF-8。").waitFor();
                assertFalse(body.innerText().contains(PRIVATE_MARKER));

                page.navigate(preview + "/book/" + java.net.URLEncoder.encode(bookUrl, StandardCharsets.UTF_8));
                page.locator(".detail-page").waitFor();
                page.getByText("清缓存", new Page.GetByTextOptions().setExact(true)).click();
                Response cleared = page.waitForResponse(response -> URI.create(response.url()).getPath()
                        .endsWith("/reader3/deleteBookCache"), () -> page.locator(".el-message-box__btns button")
                        .filter(new Locator.FilterOptions().setHasText("清除")).click());
                assertEquals(200, cleared.status());
                assertTrue(cleared.text().contains("\"isSuccess\":true"));
                page.getByText("已清除本书缓存（本机 1 条）", new Page.GetByTextOptions().setExact(true)).waitFor();

                logout(page, preview);
                login(page, first);
                bodyRequests.set(0);
                page.navigate(readerPath);
                page.locator(".reader-content").getByText(PRIVATE_MARKER).waitFor();
                assertEquals("Returning to the first user must hit its preserved current-chapter cache", 0, bodyRequests.get());
                assertEquals(Boolean.TRUE, page.evaluate("async marker => {"
                        + "const names=(await indexedDB.databases()).map(d=>d.name);"
                        + "const token=localStorage.getItem('reader_access_token')||sessionStorage.getItem('reader_access_token');"
                        + "if(names.some(n=>token&&n.includes(token)))return false;"
                        + "if(names.filter(n=>n.startsWith('reader-local-cache-v2:')).length!==2)return false;"
                        + "const db=await new Promise((yes,no)=>{const r=indexedDB.open('reader-local-cache',1);r.onsuccess=()=>yes(r.result);r.onerror=()=>no(r.error);});"
                        + "const records=await new Promise((yes,no)=>{const r=db.transaction('chapters').objectStore('chapters').getAll();r.onsuccess=()=>yes(r.result);r.onerror=()=>no(r.error);});"
                        + "db.close();return records.length===1&&records[0].content===marker;}", LEGACY_MARKER));
            } finally {
                browser.close();
            }
        }
    }
}
