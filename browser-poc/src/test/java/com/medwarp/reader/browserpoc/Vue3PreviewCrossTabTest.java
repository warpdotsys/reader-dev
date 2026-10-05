package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.*;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
import java.net.URLDecoder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.UUID;

import static org.junit.Assert.*;

/** Generated accounts in two real pages with shared cookies but independent tab sessions. */
public class Vue3PreviewCrossTabTest {
    private static final String PASSWORD = "Generated-Cross-Tab-20261005";
    private static final String FIRST_BOOK = "生成跨标签账号甲的书架元数据";
    private static final String SECOND_BOOK = "生成跨标签账号乙的书架元数据";

    private static void settled(Page page) {
        page.waitForCondition(() -> page.locator(".book-grid[aria-label='加载中']").count() == 0
                && page.locator(".refresh-btn.spinning").count() == 0);
        page.evaluate("() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))");
    }

    private static void register(Page page, String preview, String account) {
        page.navigate(preview + "/login");
        page.locator(".mode-switch button").nth(1).click();
        page.locator("input[autocomplete=username]").fill(account);
        page.locator("input[autocomplete=current-password]").fill(PASSWORD);
        page.locator("input[type=checkbox]").uncheck();
        Response login = page.waitForResponse(r -> r.url().contains("/reader3/login")
                        && "POST".equals(r.request().method()), () -> page.locator(".submit-btn").click());
        assertEquals(200, login.status());
        assertEquals(true, page.evaluate("body => JSON.parse(body).isSuccess === true", login.text()));
        page.locator(".bookshelf-page").waitFor();
        settled(page);
        assertEquals(account, page.evaluate("sessionStorage.getItem('reader_username')"));
        assertNull(page.evaluate("localStorage.getItem('reader_access_token')"));
    }

    private static void seed(Page page, String name) {
        assertEquals(true, page.evaluate("async name => {"
                + "const query=new URLSearchParams({accessToken:sessionStorage.getItem('reader_access_token')});"
                + "const r=await fetch('/reader3/saveBook?'+query,{method:'POST',"
                + "headers:{'Content-Type':'application/json'},body:JSON.stringify({"
                + "bookUrl:'generated/'+name+'.txt',origin:'loc_book',originName:'generated-metadata-only',"
                + "name,author:'生成夹具',canUpdate:false})}); const b=await r.json();"
                + "return r.status===200&&b.isSuccess===true;}", name));
        page.reload();
        page.locator(".book-card").first().waitFor();
        settled(page);
        assertTrue(page.locator("body").innerText().contains(name));
    }

    private static void screenshot(Page page, String name) {
        String dir = System.getenv("READER_UI_EVIDENCE_DIR");
        if (dir == null) dir = System.getenv("RUNNER_TEMP");
        if (dir != null) page.screenshot(new Page.ScreenshotOptions()
                .setPath(Path.of(dir, "vue3-cross-tab-generated-" + name + ".png")));
    }

    @SuppressWarnings("unchecked")
    private static Map<String, Object> read(Page page, String path, boolean explicit, String token) {
        Map<String, Object> args = new java.util.HashMap<>();
        args.put("path", path);
        args.put("explicit", explicit);
        args.put("token", token);
        return (Map<String, Object>) page.evaluate("async ({path,explicit,token}) => {"
                + "const params=new URLSearchParams();if(explicit)params.set('readerAuth','access-token');"
                + "if(token!==null)params.set('accessToken',token);"
                + "const r=await fetch('/reader3/'+path+'?'+params,{cache:'no-store',method:path==='logout'?'POST':'GET'});"
                + "return {status:r.status,namespace:r.headers.get('X-Reader-Namespace'),body:await r.json()};}",
                args);
    }

    @SuppressWarnings("unchecked")
    private static Map<String, Object> body(Map<String, Object> response) {
        assertEquals(200, ((Number) response.get("status")).intValue());
        return (Map<String, Object>) response.get("body");
    }

    @SuppressWarnings("unchecked")
    private static String infoUsername(Map<String, Object> response) {
        Map<String, Object> payload = body(response);
        assertEquals(true, payload.get("isSuccess"));
        return (String) ((Map<String, Object>) ((Map<String, Object>) payload.get("data")).get("userInfo")).get("username");
    }

    private static void relogin(Page page, String account) {
        assertEquals(true, page.evaluate("async ({username,password})=>{"
                + "const r=await fetch('/reader3/login',{method:'POST',headers:{'Content-Type':'application/json'},"
                + "body:JSON.stringify({username,password,isLogin:true})});const b=await r.json();"
                + "return r.status===200&&b.isSuccess===true;}", Map.of("username", account, "password", PASSWORD)));
    }

    @Test(timeout = 180000)
    public void twoTabAccountsMustNotFollowTheOtherTabsCookieOrCacheItsShelf() throws Exception {
        String preview = System.getenv("READER_VUE3_PREVIEW_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(preview != null && !executable.isEmpty()
                && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        assertEquals("http", URI.create(preview).getScheme());
        assertEquals("127.0.0.1", URI.create(preview).getHost());

        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")));
             Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                     .setExecutablePath(Path.of(executable)).setHeadless(true));
             BrowserContext context = browser.newContext()) {
            Page firstPage = context.newPage();
            Page secondPage = context.newPage();
            firstPage.setDefaultTimeout(10000);
            secondPage.setDefaultTimeout(10000);
            String suffix = UUID.randomUUID().toString().replace("-", "").substring(0, 10);
            String first = "taba" + suffix;
            String second = "tabb" + suffix;
            register(firstPage, preview, first);
            seed(firstPage, FIRST_BOOK);
            register(secondPage, preview, second);
            seed(secondPage, SECOND_BOOK);
            assertEquals(first, firstPage.evaluate("sessionStorage.getItem('reader_username')"));
            assertEquals(second, secondPage.evaluate("sessionStorage.getItem('reader_username')"));
            String firstToken = (String) firstPage.evaluate("sessionStorage.getItem('reader_access_token')");
            // Valid old account-scoped envelope containing a wrong Cookie account: preserve, never trust it.
            String oldCache = (String) firstPage.evaluate("({account,marker}) => {"
                    + "const scope=JSON.stringify([location.origin+'/',account,account]);"
                    + "const key='reader_shelf_offline_v2:'+scope;localStorage.setItem(key,JSON.stringify({"
                    + "version:2,scope,books:[{bookUrl:'generated-poison',tocUrl:'',origin:'loc_book',"
                    + "originName:'generated',name:marker,author:'generated'}],groups:[],ts:1}));return key;}",
                    Map.of("account", first, "marker", SECOND_BOOK));
            Object oldCacheValue = firstPage.evaluate("key=>localStorage.getItem(key)", oldCache);

            Response reply = firstPage.waitForResponse(r -> r.url().contains("/reader3/getBookshelf"),
                    () -> firstPage.locator(".refresh-btn").click());
            settled(firstPage);
            boolean sentFirstIdentity = URLDecoder.decode(URI.create(reply.url()).getRawQuery(),
                    StandardCharsets.UTF_8).contains("accessToken=" + first + ":");
            boolean renderedSecond = firstPage.locator("body").innerText().contains(SECOND_BOOK);
            boolean cachedSecondUnderFirst = Boolean.TRUE.equals(firstPage.evaluate("({account,marker}) => "
                    + "Object.keys(localStorage).filter(k=>k.startsWith('reader_shelf_offline_v2:'))"
                    + ".some(k=>{const b=JSON.parse(localStorage.getItem(k));return JSON.parse(b.scope)[1]===account"
                    + "&&JSON.parse(b.scope).includes('token-auth-v1')&&JSON.stringify(b.books).includes(marker);})", Map.of("account", first, "marker", SECOND_BOOK)));
            screenshot(firstPage, "first-after-second-login");
            System.out.printf("CROSS_TAB sentFirstIdentity=%s visibleFirstIdentity=%s actualNamespace=%s renderedSecond=%s cachedSecondUnderFirst=%s%n",
                    sentFirstIdentity, first.equals(firstPage.locator(".user-chip").innerText()),
                    reply.headers().get("x-reader-namespace"), renderedSecond, cachedSecondUnderFirst);
            assertEquals(200, reply.status());
            assertEquals(true, firstPage.evaluate("body=>JSON.parse(body).isSuccess===true", reply.text()));
            assertTrue("The actual first-tab request must still contain the first account's token", sentFirstIdentity);
            assertTrue("Actual UI requests must opt in explicitly", reply.url().contains("readerAuth=access-token"));
            assertEquals("Actual backend namespace must match the tab, not the shared cookie", first,
                    reply.headers().get("x-reader-namespace"));
            assertFalse("First tab must not render the second account's shelf", renderedSecond);
            assertFalse("First tab must not cache the second shelf under its own scope", cachedSecondUnderFirst);
            assertTrue(firstPage.locator("body").innerText().contains(FIRST_BOOK));
            assertTrue(secondPage.locator("body").innerText().contains(SECOND_BOOK));
            // Tokens held by both pages become historical, still valid under the original expiry map.
            relogin(firstPage, first);
            relogin(secondPage, second);
            assertEquals(first, infoUsername(read(firstPage, "getUserInfo", true, firstToken)));
            assertEquals("Old clients keep Cookie-first behavior", second,
                    infoUsername(read(firstPage, "getUserInfo", false, firstToken)));
            for (String invalid : new String[] {null, "generated-invalid-token"}) {
                for (String endpoint : new String[] {"getBookshelf", "getUserInfo"}) {
                    Map<String, Object> denied = body(read(firstPage, endpoint, true, invalid));
                    assertEquals(false, denied.get("isSuccess"));
                    assertEquals("NEED_LOGIN", denied.get("data"));
                    assertFalse(String.valueOf(denied).contains(SECOND_BOOK));
                }
            }

            // Real Vue write with Cookie B must persist only in token account A.
            String groupName = "生成跨标签分组甲" + suffix;
            firstPage.locator(".group-manage").first().click();
            firstPage.locator(".group-create input.group-input").fill(groupName);
            Response groupWrite = firstPage.waitForResponse(r -> r.url().contains("/reader3/saveBookGroup"),
                    () -> firstPage.locator(".group-create .accent-btn").click());
            firstPage.locator(".group-row").filter(new Locator.FilterOptions().setHasText(groupName)).waitFor();
            assertEquals(200, groupWrite.status());
            assertEquals(first, groupWrite.headers().get("x-reader-namespace"));
            assertEquals(true, body(read(firstPage, "getBookGroups", true, firstToken)).get("isSuccess"));
            assertTrue(String.valueOf(body(read(firstPage, "getBookGroups", true, firstToken)).get("data")).contains(groupName));
            String secondToken = (String) secondPage.evaluate("sessionStorage.getItem('reader_access_token')");
            assertFalse(String.valueOf(body(read(secondPage, "getBookGroups", true, secondToken)).get("data")).contains(groupName));
            firstPage.locator("[aria-label='分组管理'] button[title='关闭']").click();
            assertEquals(second, infoUsername(read(firstPage, "getUserInfo", false, null)));

            // Remove only newly generated A snapshots: network failure must not revive the preserved old envelope.
            firstPage.evaluate("account=>Object.keys(localStorage).filter(k=>k.startsWith('reader_shelf_offline_v2:'))"
                    + ".forEach(k=>{const scope=JSON.parse(k.slice('reader_shelf_offline_v2:'.length));"
                    + "if(scope[1]===account&&scope.includes('token-auth-v1'))localStorage.removeItem(k);})", first);
            firstPage.route("**/reader3/getBookshelf?*", route -> route.abort("failed"));
            firstPage.reload();
            firstPage.locator(".bookshelf-page").waitFor();
            settled(firstPage);
            assertFalse(firstPage.locator("body").innerText().contains(SECOND_BOOK));
            assertEquals(0, firstPage.locator(".offline-shelf-banner").count());
            assertEquals(oldCacheValue, firstPage.evaluate("key=>localStorage.getItem(key)", oldCache));
            screenshot(firstPage, "old-cookie-cache-refused");
            firstPage.unroute("**/reader3/getBookshelf?*");
            firstPage.locator(".refresh-btn").click();
            settled(firstPage);
            assertTrue(firstPage.locator("body").innerText().contains(FIRST_BOOK));

            // Actual backend logout revokes A, not the shared B Cookie or B's token.
            Map<String, Object> logout = body(read(firstPage, "logout", true, firstToken));
            // Preserve the legacy chained ReturnData contract, not an assumed conventional logout shape.
            System.out.printf("CROSS_TAB_LOGOUT isSuccess=%s errorMsg=%s data=%s%n",
                    logout.get("isSuccess"), logout.get("errorMsg"), logout.get("data"));
            assertEquals(true, logout.get("isSuccess"));
            assertEquals("", logout.get("errorMsg"));
            assertEquals("NEED_LOGIN", logout.get("data"));
            assertEquals("NEED_LOGIN", body(read(firstPage, "getBookshelf", true, firstToken)).get("data"));
            assertEquals(second, infoUsername(read(secondPage, "getUserInfo", true, secondToken)));
            assertEquals(second, infoUsername(read(secondPage, "getUserInfo", false, null)));
            firstPage.locator(".logout-btn").click();
            firstPage.locator(".login-page").waitFor();
            secondPage.locator(".refresh-btn").click();
            settled(secondPage);
            assertTrue(secondPage.locator("body").innerText().contains(SECOND_BOOK));
            assertEquals(second, secondPage.locator(".user-chip").innerText());
            assertEquals(oldCacheValue, secondPage.evaluate("key=>localStorage.getItem(key)", oldCache));
            screenshot(secondPage, "second-survives-first-logout");
            System.out.println("CROSS_TAB_COMPLETE read/write/info/missing/invalid/legacy-cookie/history/old-cache/logout verified with generated accounts");
        }
    }
}
