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
