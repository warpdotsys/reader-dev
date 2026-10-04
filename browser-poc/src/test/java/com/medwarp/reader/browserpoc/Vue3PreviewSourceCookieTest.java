package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import org.junit.Assume;
import org.junit.Test;

import java.net.URI;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.UUID;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

/** New Vue 3 cookie-management API: real JSON array, persistence and user isolation. */
public class Vue3PreviewSourceCookieTest {
    @Test
    public void setListIsolateAndClearSourceCookie() {
        String previewUrl = System.getenv("READER_VUE3_PREVIEW_URL");
        String executable = System.getProperty("browser.executable", "");
        Assume.assumeTrue(previewUrl != null && !previewUrl.isEmpty()
                && !executable.isEmpty() && Files.isRegularFile(Path.of(executable))
                && "1".equals(System.getenv("READER_VUE3_ISOLATED")));
        URI preview = URI.create(previewUrl);
        assertEquals("http", preview.getScheme());
        assertTrue("Only isolated loopback Reader is allowed",
                "127.0.0.1".equals(preview.getHost()) || "localhost".equals(preview.getHost()));

        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Map.of("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            Browser browser = playwright.chromium().launch(new BrowserType.LaunchOptions()
                    .setExecutablePath(Path.of(executable)).setHeadless(true));
            try {
                Page owner = browser.newPage();
                Page stranger = browser.newPage();
                register(owner, previewUrl);
                register(stranger, previewUrl);
                String domain = "cookie-" + UUID.randomUUID().toString().replace("-", "") + ".example";
                String sourceA = "https://a." + domain + "/path";
                String sourceB = "https://b." + domain + "/path";
                Map<String, String> sources = Map.of("a", sourceA, "b", sourceB);
                Object ownerResult = owner.evaluate("async (sources) => {" +
                        "const token=localStorage.getItem('reader_access_token');" +
                        "const endpoint=(name)=>'/reader3/'+name+'?accessToken='+encodeURIComponent(token);" +
                        "const post=async(name,body)=>(await fetch(endpoint(name),{method:'POST'," +
                        "headers:{'Content-Type':'application/json'},body:JSON.stringify(body)})).json();" +
                        "const read=async()=>(await fetch(endpoint('getBookSourceCookie'))).json();" +
                        "const savedA=await post('saveBookSource',{bookSourceUrl:sources.a,bookSourceName:'Cookie A'});" +
                        "const savedB=await post('saveBookSource',{bookSourceUrl:sources.b,bookSourceName:'Cookie B'});" +
                        "const denied=await post('setBookSourceCookie',{bookSource:JSON.stringify({bookSourceUrl:'https://unimported.example'}),cookie:'x=y'});" +
                        "const set=await post('setBookSourceCookie',{bookSource:sources.a,cookie:'session=probe'});" +
                        "const listed=await read();" +
                        "return {saved:savedA.isSuccess&&savedB.isSuccess,denied:!denied.isSuccess," +
                        "set:set.isSuccess,array:Array.isArray(listed.data)," +
                        "found:Array.isArray(listed.data)&&[sources.a,sources.b].every(url=>" +
                        "listed.data.some(row=>row.sourceUrl===url&&row.hasCookie===true&&" +
                        "row.cookie.includes('***')&&!row.cookie.includes('session=probe')))};" +
                        "}", sources);
                assertEquals(Map.of("saved", true, "denied", true, "set", true,
                        "array", true, "found", true), ownerResult);

                Object isolated = stranger.evaluate("async (sources) => {" +
                        "const token=localStorage.getItem('reader_access_token');" +
                        "const response=await fetch('/reader3/getBookSourceCookie?accessToken='+encodeURIComponent(token));" +
                        "const result=await response.json();" +
                        "return result.isSuccess&&Array.isArray(result.data)&&" +
                        "!result.data.some(row=>row.sourceUrl===sources.a||row.sourceUrl===sources.b);" +
                        "}", sources);
                assertEquals(true, isolated);

                Object cleared = owner.evaluate("async (sources) => {" +
                        "const token=localStorage.getItem('reader_access_token');" +
                        "const endpoint=(name)=>'/reader3/'+name+'?accessToken='+encodeURIComponent(token);" +
                        "const response=await fetch(endpoint('setBookSourceCookie'),{method:'POST'," +
                        "headers:{'Content-Type':'application/json'}," +
                        "body:JSON.stringify({bookSource:sources.b,cookie:''})});" +
                        "const set=await response.json();" +
                        "const listed=await (await fetch(endpoint('getBookSourceCookie'))).json();" +
                        "return set.isSuccess&&set.data.cleared===true&&" +
                        "[sources.a,sources.b].every(url=>set.data.clearedSourceUrls.includes(url))&&" +
                        "Array.isArray(listed.data)&&" +
                        "!listed.data.some(row=>row.sourceUrl===sources.a||row.sourceUrl===sources.b);" +
                        "}", sources);
                assertEquals(true, cleared);

                // Old versions wrote this unscoped flag. It must not override the
                // current account's server state, including after cookie revocation.
                owner.evaluate("url => localStorage.setItem('reader_src_login_'+url,'1')", sourceA);
                owner.navigate(previewUrl + "/sources");
                owner.locator(".source-row").first().waitFor();
                assertEquals("An obsolete local marker is not server authentication", 0,
                        owner.locator(".source-badge.logged").count());

                owner.locator(".source-row").filter(new com.microsoft.playwright.Locator.FilterOptions()
                        .setHasText("Cookie A")).locator("button[title^='登录书源']").click();
                owner.locator("[aria-label='书源登录'] .manual-box textarea").waitFor();
                // The last empty value deliberately has no final newline: trimming
                // TABs would destroy its seventh Netscape field before submission.
                String generatedCredential = "# Netscape HTTP Cookie File\n" +
                        "#HttpOnly_." + domain + "\tTRUE\t/auth\tTRUE\t0\tcredentialProbe\tNotReal-Qidian-12345\n" +
                        "." + domain + "\tTRUE\t/\tFALSE\t0\temptyProbe\t";
                owner.locator("[aria-label='书源登录'] .manual-box textarea").fill(generatedCredential);
                com.microsoft.playwright.Response importResponse = owner.waitForResponse(
                        response -> URI.create(response.url()).getPath().endsWith("/reader3/setBookSourceCookie"),
                        () -> owner.locator("[aria-label='书源登录'] .manual-box .accent-btn").click());
                assertEquals(200, importResponse.status());
                Map<?, ?> importedResult = (Map<?, ?>) owner.evaluate("text => JSON.parse(text)", importResponse.text());
                assertEquals("Netscape import must succeed", true, importedResult.get("isSuccess"));
                Map<?, ?> importedData = (Map<?, ?>) importedResult.get("data");
                assertEquals("Netscape metadata must reach the backend unchanged",
                        "netscape", importedData.get("format"));
                assertEquals("Both Secure/path-scoped and empty-value records must survive",
                        2, ((Number) importedData.get("imported")).intValue());
                owner.waitForFunction("document.querySelector('.login-msg')?.textContent.includes('未验证')");
                assertEquals("Saving a cookie is not proof of a successful site login",
                        "Cookie 已保存（未验证）", owner.locator(".login-state-text").innerText());
                assertFalse("Do not expose even a prefix of the submitted credential",
                        owner.locator("body").innerText().contains("NotReal-Qidian"));
                String runnerTemp = System.getenv("RUNNER_TEMP");
                if (runnerTemp != null && !runnerTemp.isEmpty()) {
                    owner.screenshot(new Page.ScreenshotOptions()
                            .setPath(Path.of(runnerTemp).resolve("vue3-source-cookie-stored.png")));
                }
                owner.locator("[aria-label='书源登录'] .dlg-close").click();
                owner.reload();
                owner.locator(".source-row").first().waitFor();
                owner.waitForFunction("document.querySelectorAll('.source-badge.logged').length===2");
                assertTrue(owner.locator(".source-badge.logged").first().innerText().contains("Cookie 已保存"));

                owner.locator(".source-row").filter(new com.microsoft.playwright.Locator.FilterOptions()
                        .setHasText("Cookie A")).locator("button[title^='编辑书源']").click();
                com.microsoft.playwright.Locator editorCookie = owner.locator(
                        "[aria-label='编辑书源'] [placeholder^='粘贴普通 Cookie 头或 Netscape 导出']");
                assertEquals("A single-line input would silently discard Netscape line breaks",
                        "TEXTAREA", editorCookie.evaluate("element => element.tagName"));
                editorCookie.fill(generatedCredential);
                assertEquals("Both editor lines and final empty field must survive", generatedCredential,
                        editorCookie.inputValue());
                com.microsoft.playwright.Response editedImport = owner.waitForResponse(
                        response -> URI.create(response.url()).getPath().endsWith("/reader3/setBookSourceCookie"),
                        () -> owner.locator("[aria-label='编辑书源'] .accent-btn[type=submit]").click());
                assertEquals(200, editedImport.status());
                Map<?, ?> editedResult = (Map<?, ?>) owner.evaluate("text => JSON.parse(text)", editedImport.text());
                assertEquals(true, editedResult.get("isSuccess"));
                Map<?, ?> editedData = (Map<?, ?>) editedResult.get("data");
                assertEquals("netscape", editedData.get("format"));
                assertEquals(2, ((Number) editedData.get("imported")).intValue());
                owner.waitForFunction("!document.querySelector('[aria-label=\"编辑书源\"]')");
                assertFalse(owner.locator("body").innerText().contains("NotReal-Qidian"));

                assertEquals(true, owner.evaluate("async url => {" +
                        "const token=localStorage.getItem('reader_access_token');" +
                        "const result=await (await fetch('/reader3/setBookSourceCookie?accessToken='+" +
                        "encodeURIComponent(token),{method:'POST',headers:{'Content-Type':'application/json'}," +
                        "body:JSON.stringify({bookSource:url,cookie:''})})).json();" +
                        "return result.isSuccess&&result.data.cleared===true;}", sourceB));
                owner.waitForResponse(response -> URI.create(response.url()).getPath().endsWith("/reader3/getBookSourceCookie"),
                        owner::reload);
                owner.locator(".source-row").first().waitFor();
                assertEquals("Server revocation must win over any surviving local flag", 0,
                        owner.locator(".source-badge.logged").count());
                // Restore only generated credentials to make the account-switch
                // check meaningful: the first account has cookies, the second does not.
                assertEquals(true, owner.evaluate("async url => {" +
                        "const token=localStorage.getItem('reader_access_token');" +
                        "const result=await (await fetch('/reader3/setBookSourceCookie?accessToken='+" +
                        "encodeURIComponent(token),{method:'POST',headers:{'Content-Type':'application/json'}," +
                        "body:JSON.stringify({bookSource:url,cookie:'session=generated-switch-test'})})).json();" +
                        "return result.isSuccess&&result.data.success===true;}", sourceA));
                owner.reload();
                owner.waitForFunction("document.querySelectorAll('.source-badge.logged').length===2");

                // Same browser origin, different authenticated account. No old UI
                // marker may carry authentication over to the second namespace.
                assertEquals(true, stranger.evaluate("async (sources) => {" +
                        "const token=localStorage.getItem('reader_access_token');" +
                        "const post=async(source)=>(await fetch('/reader3/saveBookSource?accessToken='+" +
                        "encodeURIComponent(token),{method:'POST',headers:{'Content-Type':'application/json'}," +
                        "body:JSON.stringify(source)})).json();" +
                        "return (await post({bookSourceUrl:sources.a,bookSourceName:'Cookie A'})).isSuccess&&" +
                        "(await post({bookSourceUrl:sources.b,bookSourceName:'Cookie B'})).isSuccess;}", sources));
                String otherUsername = (String) stranger.evaluate("() => localStorage.getItem('reader_username')");
                // The legacy backend prefers its session cookie to a query token.
                // Exercise an actual logout/login, not an artificial token swap
                // while keeping the previous account's authenticated session.
                owner.navigate(previewUrl + "/");
                owner.locator(".bookshelf-page").waitFor();
                owner.locator(".logout-btn").click();
                owner.locator(".login-page").waitFor();
                owner.locator("input[autocomplete=username]").fill(otherUsername);
                owner.locator("input[autocomplete=current-password]").fill("CookieProbe-2026");
                owner.locator(".submit-btn").click();
                owner.locator(".bookshelf-page").waitFor();
                owner.waitForResponse(response -> URI.create(response.url()).getPath().endsWith("/reader3/getBookSourceCookie"),
                        () -> owner.navigate(previewUrl + "/sources"));
                owner.locator(".source-row").first().waitFor();
                assertEquals("Account switching must not inherit a source cookie badge", 0,
                        owner.locator(".source-badge.logged").count());
                owner.locator("button[title^='Cookie 管理']").click();
                owner.waitForFunction("document.querySelector('.cookie-mgr-note')?.textContent.includes('0 个')");
                assertEquals(0, owner.locator(".cookie-list .cookie-row").count());
            } finally {
                browser.close();
            }
        }
    }

    private static void register(Page page, String previewUrl) {
        page.navigate(previewUrl + "/login");
        page.locator(".mode-switch button").nth(1).click();
        page.locator("input[autocomplete=username]").fill("vue" +
                UUID.randomUUID().toString().replace("-", "").substring(0, 10));
        page.locator("input[autocomplete=current-password]").fill("CookieProbe-2026");
        page.locator(".submit-btn").click();
        page.locator(".bookshelf-page").waitFor();
    }
}
