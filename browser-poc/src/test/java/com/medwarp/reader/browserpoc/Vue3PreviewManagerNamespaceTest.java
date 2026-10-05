package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.*;
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

import static org.junit.Assert.*;

/** Real legacy manager-password authorization; no fabricated isAdmin or user database edits. */
public class Vue3PreviewManagerNamespaceTest {
    private static final String PASSWORD = "Generated-Manager-Context-20261005";
    private static final String OWN = "生成的本人书架元数据";
    private static final String SYSTEM = "生成的系统配置书架元数据";

    private static void settled(Page page) {
        page.waitForCondition(() -> page.locator(".book-grid[aria-label='加载中']").count() == 0
                && page.locator(".refresh-btn.spinning").count() == 0);
        page.evaluate("() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))");
    }

    private static void screenshot(Page page, String suffix) {
        String directory = System.getenv("READER_UI_EVIDENCE_DIR");
        if (directory == null) directory = System.getenv("RUNNER_TEMP");
        if (directory != null) page.screenshot(new Page.ScreenshotOptions()
                .setPath(Path.of(directory, "vue3-manager-generated-" + suffix + ".png")));
    }

    private static void permissionLabelsAreReadable(Page page) {
        Locator labels = page.locator(".user-table tbody .perm-label");
        assertTrue("Generated manager list must have all four permission labels", labels.count() >= 4);
        for (int index = 0; index < labels.count(); index++) {
            Locator label = labels.nth(index);
            Number lines = (Number) label.evaluate("el => {const r=document.createRange();"
                    + "r.selectNodeContents(el);return Array.from(r.getClientRects())"
                    + ".filter(b=>b.width>0&&b.height>0).length;}");
            assertEquals("Manager permission label must not fragment into vertical characters: " + label.innerText(),
                    1, lines.intValue());
        }
        assertEquals("Each permission switch must keep its adjacent full label and accessible name", true,
                page.locator(".user-table tbody .perm-label").evaluateAll("labels => labels.every(label => {"
                        + "const button=label.previousElementSibling;"
                        + "return button?.getAttribute('role')==='switch'"
                        + "&&button.getAttribute('aria-label')===label.textContent.trim();})"));
    }

    private static void signIn(Page page, String account, boolean register) {
        page.locator(".mode-switch button").nth(register ? 1 : 0).click();
        page.locator("input[autocomplete=username]").fill(account);
        page.locator("input[autocomplete=current-password]").fill(PASSWORD);
        page.locator("input[type=checkbox]").check();
        Response response = page.waitForResponse(r -> r.url().contains("/reader3/login")
                        && "POST".equals(r.request().method()), () -> page.locator(".submit-btn").click());
        assertEquals(200, response.status());
        assertEquals(true, page.evaluate("body => JSON.parse(body).isSuccess === true", response.text()));
        page.locator(".bookshelf-page").waitFor();
        settled(page);
    }

    private static void noticesLeaveNavigationUsable(Page page) {
        settled(page);
        assertEquals("Visible notices must not cover navigation or the shelf search controls", false,
                page.evaluate("() => {const targets=Array.from(document.querySelectorAll('.topbar,.search-box'))"
                        + ".map(el=>el.getBoundingClientRect());return Array.from(document.querySelectorAll('.el-message'))"
                        + ".some(el=>{if(getComputedStyle(el).display==='none')return false;"
                        + "const b=el.getBoundingClientRect();const clip=el.closest('.reader-message-stack')?.getBoundingClientRect();"
                        + "const r={left:b.left,right:b.right,top:clip?Math.max(b.top,clip.top):b.top,"
                        + "bottom:clip?Math.min(b.bottom,clip.bottom):b.bottom};"
                        + "return r.bottom>r.top&&targets.some(t=>r.left<t.right&&r.right>t.left&&r.top<t.bottom&&r.bottom>t.top);});}"));
        Locator region = page.locator("#reader-message-region");
        assertEquals("1", String.valueOf(region.count()));
        assertTrue("Notice geometry must be measured while notices are actually present", region.isVisible());
        assertEquals("region", region.getAttribute("role"));
        assertEquals("操作消息，可滚动查看或关闭", region.getAttribute("aria-label"));
        assertEquals("Every notice stays inside the viewport with existing lower controls left clear", true,
                region.evaluate("el=>{const r=el.getBoundingClientRect();"
                        + "const lower=Array.from(document.querySelectorAll('.manage-bar,.reader-page .progress-bar,.update-banner'))"
                        + ".map(el=>el.getBoundingClientRect()).filter(b=>b.width>0&&b.height>0&&b.bottom>0&&b.top<innerHeight);"
                        + "return r.left>=0&&r.right<=innerWidth+1&&r.top>=0&&r.bottom<=innerHeight-8"
                        + "&&lower.every(b=>r.bottom<=b.top-8);}"));
        page.locator(".search-input").click();
        assertEquals("Notifications must not steal or block real search-input focus", true,
                page.locator(".search-input").evaluate("el=>document.activeElement===el"));
    }

    private static void key(Page page, String key, boolean success) {
        Locator dialog = page.locator("[role=dialog][aria-label='输入管理密码']");
        dialog.waitFor();
        dialog.locator("input[type=password]").fill(key);
        Response response = page.waitForResponse(r -> r.url().contains("/reader3/getUserList"),
                () -> dialog.locator("button[type=submit]").click());
        assertEquals(200, response.status());
        assertEquals(success, page.evaluate("body => JSON.parse(body).isSuccess === true", response.text()));
        if (success) {
            assertEquals("Legacy has no persistent per-user administrator field", false,
                    page.evaluate("body => JSON.parse(body).data.some(u => 'isAdmin' in u)", response.text()));
            page.waitForCondition(() -> page.locator("[role=dialog][aria-label='输入管理密码']").count() == 0);
        } else dialog.waitFor();
    }

    @Test
    public void verifiedManagerUsesRealUserNSAndCannotLeakToTheNextAccount() throws Exception {
        String preview = System.getenv("READER_VUE3_SECURE_URL");
        String managerKey = System.getenv("READER_TEST_MANAGER_KEY");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(preview != null && managerKey != null && !managerKey.isEmpty()
                && !executable.isEmpty() && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        assertEquals("http", URI.create(preview).getScheme());
        assertEquals("127.0.0.1", URI.create(preview).getHost());
        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")));
             Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                     .setExecutablePath(Path.of(executable)).setHeadless(true));
             BrowserContext context = browser.newContext(new Browser.NewContextOptions().setViewportSize(1280, 720))) {
            Page page = context.newPage();
            page.setDefaultTimeout(10000);
            String suffix = UUID.randomUUID().toString().replace("-", "").substring(0, 10);
            String first = "managera" + suffix;
            String second = "managerb" + suffix;
            AtomicBoolean offline = new AtomicBoolean();
            AtomicBoolean wrongHeader = new AtomicBoolean();
            page.route("**/reader3/getBookshelf?*", route -> {
                if (offline.get()) route.abort("failed");
                else if (wrongHeader.get()) {
                    java.util.HashMap<String, String> headers = new java.util.HashMap<>(route.request().headers());
                    headers.put("x-reader-secure-key", "generated-wrong-manager-key");
                    route.resume(new Route.ResumeOptions().setHeaders(headers));
                } else route.resume();
            });
            page.navigate(preview + "/login");
            signIn(page, first, true);
            Object saved = page.evaluate("async ({own,system,key}) => { const token=localStorage.getItem('reader_access_token');"
                    + " for (const [name,ns] of [[own,''],[system,'default']]) {"
                    + " const query=new URLSearchParams({accessToken:token}); if(ns)query.set('userNS',ns);"
                    + " const r=await fetch('/reader3/saveBook?'+query,{method:'POST',"
                    + " headers:{'Content-Type':'application/json',...(ns?{'X-Reader-Secure-Key':key}:{})},"
                    + " body:JSON.stringify({bookUrl:'generated/'+name+'.txt',origin:'loc_book',"
                    + " originName:'generated-metadata-only',name,author:'生成夹具',canUpdate:false})});"
                    + " const b=await r.json(); if(r.status!==200||b.isSuccess!==true)return false; } return true; }",
                    Map.of("own", OWN, "system", SYSTEM, "key", managerKey));
            assertEquals(true, saved);
            String token = (String) page.evaluate("localStorage.getItem('reader_access_token')");
            APIResponse legacy = context.request().get(preview + "/reader3/getBookshelf?accessToken="
                    + URLEncoder.encode(token, StandardCharsets.UTF_8) + "&userNS=default",
                    com.microsoft.playwright.options.RequestOptions.create().setHeader("X-Reader-Secure-Key", managerKey));
            assertEquals(200, legacy.status());
            assertEquals("default", legacy.headers().get("x-reader-namespace"));
            assertEquals(true, page.evaluate("body => {const b=JSON.parse(body);return b.isSuccess===true&&b.data.length===1&&b.data[0].name==='" + SYSTEM + "';}", legacy.text()));
            APIResponse fallback = context.request().get(preview + "/reader3/getBookshelf?accessToken="
                    + URLEncoder.encode(token, StandardCharsets.UTF_8) + "&userNS=default",
                    com.microsoft.playwright.options.RequestOptions.create().setHeader("X-Reader-Secure-Key", "generated-wrong-manager-key"));
            assertEquals(200, fallback.status());
            assertEquals(first, fallback.headers().get("x-reader-namespace"));
            assertEquals("Backend preserves legacy fallback semantics but identifies its actual namespace", true,
                    page.evaluate("body => {const b=JSON.parse(body);return b.isSuccess===true&&b.data.length===1&&b.data[0].name==='" + OWN + "';}", fallback.text()));
            page.locator(".refresh-btn").click(); settled(page);
            assertTrue(page.locator("body").innerText().contains(OWN));
            assertFalse(page.locator("body").innerText().contains(SYSTEM));
            page.navigate(preview + "/users");
            page.locator(".users-page").waitFor();
            key(page, "generated-wrong-manager-key", false);
            assertEquals(0, page.locator(".default-config-btn:visible").count());
            key(page, managerKey, true);
            screenshot(page, "authorized");
            permissionLabelsAreReadable(page);
            page.setViewportSize(375, 812);
            permissionLabelsAreReadable(page);
            assertEquals("Narrow user list must scroll inside its table region, not the whole page", true,
                    page.evaluate("() => document.documentElement.scrollWidth <= innerWidth+1"));
            Locator table = page.locator(".table-wrap");
            assertEquals("0", table.getAttribute("tabindex"));
            assertEquals(true, table.evaluate("el => el.scrollWidth > el.clientWidth"));
            table.focus();
            table.press("ArrowRight");
            page.waitForCondition(() -> Boolean.TRUE.equals(table.evaluate("el => el.scrollLeft>0")));
            screenshot(page, "permissions-narrow");
            page.setViewportSize(1280, 720);
            assertTrue("Real manager-password success must expose the system-configuration button, without fake isAdmin",
                    page.locator(".default-config-btn:visible").count() > 0);
            assertFalse("Legacy user management cannot pretend to persist administrator roles",
                    page.locator(".users-page").innerText().contains("管理员"));
            page.locator(".nav-link").filter(new Locator.FilterOptions().setHasText("书架")).click();
            page.locator(".bookshelf-page").waitFor(); settled(page);
            Response systemShelf = page.waitForResponse(r -> r.url().contains("/reader3/getBookshelf"),
                    () -> page.locator(".default-config-btn:visible").click());
            assertTrue(systemShelf.url().contains("userNS=default"));
            assertFalse(systemShelf.url().contains("ns=default"));
            assertFalse(systemShelf.url().contains("secureKey="));
            assertEquals("default", systemShelf.headers().get("x-reader-namespace"));
            settled(page);
            assertTrue(page.locator("body").innerText().contains(SYSTEM));
            assertFalse(page.locator("body").innerText().contains(OWN));
            page.reload();
            page.waitForCondition(() -> page.locator(".default-config-btn.active:visible").count() == 1
                    && page.locator("body").innerText().contains(SYSTEM));
            settled(page);
            wrongHeader.set(true);
            page.locator(".refresh-btn").click(); settled(page);
            page.waitForCondition(() -> page.locator(".offline-shelf-banner").count() == 0);
            assertFalse("Wrong manager header must not render the server's own-user fallback", page.locator("body").innerText().contains(OWN));
            assertFalse(page.locator("body").innerText().contains(SYSTEM));
            screenshot(page, "wrong-space-rejected");
            wrongHeader.set(false);
            offline.set(true);
            page.locator(".refresh-btn").click(); settled(page);
            assertEquals(1, page.locator(".offline-shelf-banner").count());
            assertTrue(page.locator("body").innerText().contains(SYSTEM));
            page.locator(".default-config-btn:visible").click(); settled(page);
            assertTrue(page.locator("body").innerText().contains(OWN));
            assertFalse(page.locator("body").innerText().contains(SYSTEM));
            page.waitForCondition(() -> page.locator(".el-message--error:visible").count() > 0);
            assertEquals("Repeated identical transport errors must share one visible notice", 1,
                    page.locator(".el-message--error:visible").count());
            assertEquals("Transport failures must be displayed in Chinese without changing namespace rejection", true,
                    page.locator(".el-message--error:visible").evaluateAll("messages => messages.every(message => "
                            + "!message.textContent.includes('Network Error'))"
                            + "&&messages.some(message=>message.textContent.includes('网络连接失败，请检查连接后重试'))"));
            screenshot(page, "own-offline");
            noticesLeaveNavigationUsable(page);
            // Namespace changes trigger non-silent loads; a manual refresh intentionally stays silent.
            // Preserve all resulting notices instead of discarding them with a message-count limit.
            for (int round = 0; round < 4; round++) {
                page.locator(".default-config-btn:visible").click(); settled(page);
            }
            assertTrue("Real offline namespace changes must retain several warning notices",
                    page.locator(".el-message--warning:visible").count() >= 3);
            page.locator(".manage-btn").click(); settled(page);
            noticesLeaveNavigationUsable(page);
            screenshot(page, "notices-manage-footer");
            page.locator(".manage-btn").click(); settled(page);
            page.locator(".manage-bar").waitFor(new Locator.WaitForOptions()
                    .setState(com.microsoft.playwright.options.WaitForSelectorState.HIDDEN));
            page.setViewportSize(640, 360);
            noticesLeaveNavigationUsable(page);
            screenshot(page, "notices-short");
            page.setViewportSize(375, 320);
            noticesLeaveNavigationUsable(page);
            Locator stack = page.locator(".reader-message-stack");
            screenshot(page, "notices-narrow-short-before-scroll");
            System.out.println("NOTICE_VIEWPORT " + stack.evaluate("el=>JSON.stringify({"
                    + "width:innerWidth,height:innerHeight,clientHeight:el.clientHeight,scrollHeight:el.scrollHeight,"
                    + "regionHeight:el.parentElement.getBoundingClientRect().height,"
                    + "safeTop:el.parentElement.style.getPropertyValue('--reader-message-safe-top'),"
                    + "bottom:el.parentElement.style.getPropertyValue('--reader-message-bottom')})"));
            assertEquals("0", stack.getAttribute("tabindex"));
            assertEquals("A short viewport must still expose a readable notification line", true,
                    stack.evaluate("el=>el.clientHeight>=40"));
            assertEquals("All retained notices must remain scrollable on a short narrow screen", true,
                    stack.evaluate("el=>el.scrollHeight>el.clientHeight"));
            Locator navigation = page.locator(".user-area");
            assertEquals("Short-screen navigation keeps readable labels and its own scroll area", true,
                    navigation.evaluate("el=>el.scrollWidth>el.clientWidth"));
            assertEquals("0", navigation.getAttribute("tabindex"));
            navigation.focus();
            Number navigationBefore = (Number) navigation.evaluate("el=>el.scrollLeft");
            navigation.press(navigationBefore.doubleValue() > 0 ? "ArrowLeft" : "ArrowRight");
            page.waitForCondition(() -> ((Number) navigation.evaluate("el=>el.scrollLeft")).doubleValue()
                    != navigationBefore.doubleValue());
            stack.focus(); stack.press("End");
            page.waitForCondition(() -> Boolean.TRUE.equals(stack.evaluate("el=>el.scrollTop>0")));
            // Focusing/scanning scroll regions may also move the document: recheck actual geometry.
            noticesLeaveNavigationUsable(page);
            screenshot(page, "notices-narrow-short");
            page.locator(".reader-message-dismiss").click();
            page.waitForCondition(() -> page.locator(".el-message:visible").count() == 0
                    && !page.locator("#reader-message-region").isVisible());
            page.setViewportSize(1280, 720);
            offline.set(false);
            page.locator(".logout-btn").click(); page.locator(".login-page").waitFor();
            signIn(page, second, true);
            assertEquals(0, page.locator(".default-config-btn:visible").count());
            assertFalse(page.locator("body").innerText().contains(OWN));
            assertFalse(page.locator("body").innerText().contains(SYSTEM));
            page.navigate(preview + "/users");
            page.locator("[role=dialog][aria-label='输入管理密码']").waitFor();
            assertEquals("A's verified manager key cannot be inherited by B", "",
                    page.locator("[role=dialog][aria-label='输入管理密码'] input[type=password]").inputValue());
            assertEquals(false, page.evaluate("secret => performance.getEntriesByType('resource').some(r => r.name.includes(secret)||r.name.includes('secureKey='))", managerKey));
            System.out.println("MANAGER_NAMESPACE legacyUserNS=actual noFakeIsAdmin=verified wrongKey=denied namespaceProof=actual silentFallback=rejected reload=reverified systemShelf=isolated ownOffline=isolated nextAccount=reverify credentialUrl=absent");
        }
    }
}
