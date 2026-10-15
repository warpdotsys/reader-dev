package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.*;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.Assert.*;

/** Real UI logout and mixed remembered/tab-local accounts; generated accounts only. */
public class Vue3PreviewLogoutTest {
    private static final String PASSWORD = "Generated-Logout-20261005";

    private static void signIn(Page page, String preview, String account, boolean register, boolean remember) {
        page.navigate(preview + "/login");
        page.locator(".mode-switch button").nth(register ? 1 : 0).click();
        page.locator("input[autocomplete=username]").fill(account);
        page.locator("input[autocomplete=current-password]").fill(PASSWORD);
        page.locator("input[type=checkbox]").setChecked(remember);
        Response response = page.waitForResponse(r -> r.url().contains("/reader3/login")
                && "POST".equals(r.request().method()), () -> page.locator(".submit-btn").click());
        assertEquals(200, response.status());
        assertEquals(true, page.evaluate("body=>JSON.parse(body).isSuccess===true", response.text()));
        page.locator(".bookshelf-page").waitFor();
        assertEquals(account, page.locator(".user-chip").innerText());
    }

    private static String token(Page page) {
        return (String) page.evaluate("sessionStorage.getItem('reader_access_token') || localStorage.getItem('reader_access_token')");
    }

    private static void assertToken(Page page, String token, boolean accepted) {
        assertEquals(true, page.evaluate("async ({token,accepted})=>{"
                + "const query=new URLSearchParams({readerAuth:'access-token',accessToken:token});"
                + "const r=await fetch('/reader3/getUserInfo?'+query,{cache:'no-store'});const b=await r.json();"
                + "return r.status===200&&(accepted?b.isSuccess===true&&b.data.userInfo.username===token.split(':')[0]"
                + ":b.isSuccess===false&&b.data==='NEED_LOGIN');}", Map.of("token", token, "accepted", accepted)));
    }

    private static void verifyLogout(Page page, Response response, String token) {
        assertEquals(200, response.status());
        assertEquals(true, page.evaluate("({url,body,token})=>{const q=new URL(url).searchParams;const b=JSON.parse(body);"
                + "return q.get('readerAuth')==='access-token'&&q.get('accessToken')===token"
                + "&&b.isSuccess===true&&b.errorMsg===''&&b.data==='NEED_LOGIN';}",
                Map.of("url", response.url(), "body", response.text(), "token", token)));
        page.locator(".login-page").waitFor();
        assertToken(page, token, false);
    }

    private static void screenshot(Page page, String name) {
        String dir = System.getenv("READER_UI_EVIDENCE_DIR");
        if (dir == null) dir = System.getenv("RUNNER_TEMP");
        if (dir != null) page.screenshot(new Page.ScreenshotOptions()
                .setPath(Path.of(dir, "vue3-logout-generated-" + name + ".png")));
    }

    @Test(timeout = 180000)
    public void actualButtonsRevokeOnlyTheirTokenAndPreserveOtherTabsRememberedSession() {
        String preview = System.getenv("READER_VUE3_PREVIEW_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(preview != null && !executable.isEmpty() && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        assertEquals("http", URI.create(preview).getScheme());
        assertEquals("127.0.0.1", URI.create(preview).getHost());
        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")));
             Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                     .setExecutablePath(Path.of(executable)).setHeadless(true));
             BrowserContext context = browser.newContext()) {
            Page first = context.newPage();
            Page second = context.newPage();
            first.setDefaultTimeout(10000);
            second.setDefaultTimeout(10000);
            String suffix = UUID.randomUUID().toString().replace("-", "").substring(0, 10);
            String accountA = "logouta" + suffix;
            String accountB = "logoutb" + suffix;
            signIn(first, preview, accountA, true, false);
            String tokenA = token(first);
            signIn(second, preview, accountB, true, true);
            String tokenB = token(second);
            try {
                Response logout = first.waitForResponse(r -> r.url().contains("/reader3/logout")
                        && "POST".equals(r.request().method()), () -> first.locator(".logout-btn").click());
                verifyLogout(first, logout, tokenA);
            } finally { screenshot(first, "first-button"); }
            assertEquals(tokenB, second.evaluate("localStorage.getItem('reader_access_token')"));
            assertEquals(accountB, second.evaluate("localStorage.getItem('reader_username')"));
            assertToken(second, tokenB, true);
            first.reload();
            first.locator(".login-page").waitFor();
            assertEquals(0, first.locator(".bookshelf-page").count());

            // The transient login must not erase B's saved login; reload must prefer A's complete pair.
            signIn(first, preview, accountA, false, false);
            String retryTokenA = token(first);
            assertEquals(tokenB, first.evaluate("localStorage.getItem('reader_access_token')"));
            Response reloaded = first.waitForResponse(r -> r.url().contains("/reader3/getBookshelf"), first::reload);
            assertEquals(200, reloaded.status());
            assertEquals(accountA, reloaded.headers().get("x-reader-namespace"));
            first.locator(".bookshelf-page").waitFor();
            assertEquals(accountA, first.locator(".user-chip").innerText());
            second.reload();
            second.locator(".bookshelf-page").waitFor();
            assertEquals(accountB, second.locator(".user-chip").innerText());

            // A real settings cancel must send nothing. Confirm must use the same remote revocation.
            first.locator(".topbar .user-area button").filter(new Locator.FilterOptions().setHasText("设置")).click();
            first.locator(".settings-page").waitFor();
            AtomicInteger requests = new AtomicInteger();
            first.onRequest(r -> { if (r.url().contains("/reader3/logout")) requests.incrementAndGet(); });
            Locator settingsLogout = first.locator("button.danger-btn").filter(new Locator.FilterOptions().setHasText("退出登录"));
            settingsLogout.click();
            first.locator(".el-message-box__btns button").filter(new Locator.FilterOptions().setHasText("取消")).click();
            assertEquals(0, requests.get());
            assertToken(first, retryTokenA, true);
            settingsLogout.click();
            Response settingsResponse = first.waitForResponse(r -> r.url().contains("/reader3/logout")
                    && "POST".equals(r.request().method()), () -> first.locator(".el-message-box__btns button")
                    .filter(new Locator.FilterOptions().setHasText("退出")).click());
            verifyLogout(first, settingsResponse, retryTokenA);
            assertEquals(1, requests.get());
            assertEquals(tokenB, second.evaluate("localStorage.getItem('reader_access_token')"));

            // No false claim of server revocation when only the logout request is offline.
            signIn(first, preview, accountA, false, false);
            String offlineToken = token(first);
            first.route("**/reader3/logout?*", route -> route.abort("failed"));
            first.locator(".logout-btn").click();
            first.locator(".login-page").waitFor();
            first.locator(".el-message--warning").filter(new Locator.FilterOptions()
                    .setHasText("服务端令牌尚未确认撤销")).waitFor();
            first.unroute("**/reader3/logout?*");
            assertToken(first, offlineToken, true);
            assertEquals(tokenB, second.evaluate("localStorage.getItem('reader_access_token')"));
            assertToken(second, tokenB, true);
            screenshot(first, "offline-local-only");
            screenshot(second, "remembered-survives");
            System.out.println("LOGOUT_COMPLETE bookshelf/settings/cancel/mixed-reload/signed-out/offline-warning/other-token preserved; generated accounts only");
        }
    }
}
